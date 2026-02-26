use anyhow::Context;
use clap::{Parser, Subcommand};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Semaphore;
use tracing::{error, info};

use loglens_api::{build_router, routes::service, AppState, MAX_CONCURRENT_PARSE_JOBS};

#[derive(Debug, Parser)]
#[command(name = "loglens-api")]
#[command(about = "LogLens Rust API service")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://loglens:loglens@localhost:5432/loglens"
    )]
    database_url: String,

    #[arg(long, env = "APP_HOST", default_value = "0.0.0.0")]
    host: String,

    #[arg(long, env = "APP_PORT", default_value_t = 8080)]
    port: u16,

    #[arg(
        long,
        env = "CORS_ALLOWED_ORIGIN",
        default_value = "http://localhost:3000"
    )]
    cors_allowed_origin: String,
}

#[derive(Debug, Clone, Copy, Subcommand)]
enum Command {
    Serve,
    Migrate,
    PrintOpenapi,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Serve);

    match command {
        Command::Serve => serve(&cli).await,
        Command::Migrate => migrate(&cli.database_url).await,
        Command::PrintOpenapi => {
            use utoipa::OpenApi;
            let json = serde_json::to_string_pretty(&service::ApiDoc::openapi())
                .context("failed to serialize OpenAPI contract")?;
            println!("{json}");
            Ok(())
        }
    }
}

async fn serve(cli: &Cli) -> anyhow::Result<()> {
    let pool = connect_pool(&cli.database_url).await?;
    run_migrations(&pool).await?;

    let version =
        std::env::var("APP_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
    let state = Arc::new(AppState {
        pool: pool.clone(),
        version,
        parse_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_PARSE_JOBS)),
    });
    let app = build_router(state, &cli.cors_allowed_origin);

    tokio::spawn(expired_job_reaper(pool));

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port)
        .parse()
        .context("failed to parse listen address")?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind TCP listener")?;

    info!("API listening on http://{addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("axum server error")?;

    Ok(())
}

/// Background task that periodically deletes parse_jobs whose 30-day
/// retention window has elapsed. CASCADE foreign keys on parsed_log_events
/// and benchmark_snapshots handle child-row cleanup automatically.
async fn expired_job_reaper(pool: PgPool) {
    let interval = std::time::Duration::from_secs(60 * 60); // hourly
    loop {
        match sqlx::query_scalar::<_, i64>(
            "DELETE FROM parse_jobs WHERE expires_at <= NOW() RETURNING id",
        )
        .fetch_all(&pool)
        .await
        {
            Ok(ids) if !ids.is_empty() => {
                info!("expired-job reaper deleted {} job(s): {:?}", ids.len(), ids);
            }
            Ok(_) => {}
            Err(e) => {
                error!("expired-job reaper failed: {e}");
            }
        }
        tokio::time::sleep(interval).await;
    }
}

async fn migrate(database_url: &str) -> anyhow::Result<()> {
    let pool = connect_pool(database_url).await?;
    run_migrations(&pool).await?;
    info!("migrations completed");
    Ok(())
}

async fn connect_pool(database_url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await
        .context("failed to connect to PostgreSQL")
}

async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("failed to run migrations")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        if let Ok(mut stream) = signal(SignalKind::terminate()) {
            let _ = stream.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use loglens_api::helpers::{require_permission, role_allows};
    use loglens_api::models::*;
    use axum::http::StatusCode;

    #[test]
    fn role_policy_enforces_expected_permissions() {
        assert!(role_allows(&OrgRole::Owner, OrgPermission::ManageBilling));
        assert!(role_allows(&OrgRole::Admin, OrgPermission::ManageMembers));
        assert!(!role_allows(&OrgRole::Member, OrgPermission::ManageBilling));
        assert!(!role_allows(&OrgRole::Viewer, OrgPermission::ManageMembers));
    }

    #[test]
    fn cross_org_access_is_denied_without_membership() {
        let result = require_permission(None, OrgPermission::View);
        assert!(matches!(
            result,
            Err(loglens_api::error::ApiError {
                status: StatusCode::FORBIDDEN,
                ..
            })
        ));
    }

    #[test]
    fn license_status_controls_entitlements() {
        let active = entitlement_keys(&LicenseTier::Enterprise, &LicenseStatus::Active);
        assert!(active
            .iter()
            .any(|feature| feature == "advanced_limits_insights"));

        let suspended = entitlement_keys(&LicenseTier::Enterprise, &LicenseStatus::PastDue);
        assert!(suspended.is_empty());
    }
}

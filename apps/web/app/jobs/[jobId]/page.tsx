import { getJob, listJobBenchmarks, listJobEvents } from "@loglens/api-client";
import { buildApiAuthContext, requireSession } from "../../../lib/auth";
import JobDetailClient from "./JobDetailClient";

export const dynamic = "force-dynamic";

type PageProps = {
  params: Promise<{ jobId: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function readServerApiBaseUrl() {
  return process.env.API_INTERNAL_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
}

function readBrowserApiBaseUrl() {
  return process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
}

export default async function JobDetailPage({ params, searchParams }: PageProps) {
  const { jobId: jobIdParam } = await params;
  const resolvedSearchParams = searchParams ? await searchParams : {};
  const orgIdParam = Array.isArray(resolvedSearchParams.org)
    ? resolvedSearchParams.org[0]
    : resolvedSearchParams.org;

  const jobId = Number(jobIdParam);
  const orgId = Number(orgIdParam);

  if (!Number.isFinite(jobId) || !Number.isFinite(orgId)) {
    return <p>Invalid job or organization ID.</p>;
  }

  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const serverApiUrl = readServerApiBaseUrl();
  const browserApiUrl = readBrowserApiBaseUrl();

  const [job, benchmarksRes, eventsRes] = await Promise.all([
    getJob(auth, orgId, jobId, serverApiUrl),
    listJobBenchmarks(auth, orgId, jobId, serverApiUrl),
    listJobEvents(auth, orgId, jobId, { limit: 200 }, serverApiUrl),
  ]);

  return (
    <main style={{ padding: "2rem", maxWidth: "1200px" }}>
      <header style={{ marginBottom: "1.5rem" }}>
        <h1>Job #{job.job_id}</h1>
        <p style={{ color: "#888" }}>{job.file_name}</p>
      </header>

      <section style={{ marginBottom: "1rem" }}>
        <dl
          style={{
            display: "grid",
            gridTemplateColumns: "auto 1fr",
            gap: "0.25rem 1rem",
            fontSize: "0.875rem",
          }}
        >
          <dt>Status</dt>
          <dd>{job.status}</dd>
          <dt>Total lines</dt>
          <dd>{job.total_lines.toLocaleString()}</dd>
          <dt>Parsed lines</dt>
          <dd>{job.parsed_lines.toLocaleString()}</dd>
          <dt>Benchmarks</dt>
          <dd>{job.benchmark_count}</dd>
          {job.error_message && (
            <>
              <dt>Error</dt>
              <dd style={{ color: "#ef4444" }}>{job.error_message}</dd>
            </>
          )}
        </dl>
      </section>

      <JobDetailClient
        orgId={orgId}
        jobId={jobId}
        auth={auth}
        benchmarks={benchmarksRes.benchmarks}
        initialEvents={eventsRes.events}
        totalEvents={eventsRes.total}
        apiBaseUrl={browserApiUrl}
      />
    </main>
  );
}

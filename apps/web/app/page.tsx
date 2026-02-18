import { getHealth, getVersion } from "@loglens/api-client";

export const dynamic = "force-dynamic";

function readServerApiBaseUrl() {
  return process.env.API_INTERNAL_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
}

export default async function HomePage() {
  const apiBaseUrl = readServerApiBaseUrl();

  try {
    const [health, version] = await Promise.all([getHealth(apiBaseUrl), getVersion(apiBaseUrl)]);

    return (
      <main>
        <h1>LogLens</h1>
        <p>
          <a href="/login">Sign in</a> | <a href="/app">Open app dashboard</a>
        </p>
        <p>API base URL: {apiBaseUrl}</p>
        <ul>
          <li>Health status: {health.status}</li>
          <li>Database status: {health.database}</li>
          <li>API version: {version.version}</li>
        </ul>
      </main>
    );
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown API error";

    return (
      <main>
        <h1>LogLens</h1>
        <p>
          <a href="/login">Sign in</a> | <a href="/app">Open app dashboard</a>
        </p>
        <p>API base URL: {apiBaseUrl}</p>
        <p>Unable to reach API: {message}</p>
      </main>
    );
  }
}

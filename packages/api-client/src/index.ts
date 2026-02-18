import type { components, paths } from "./generated.js";

type HealthResponse = paths["/health"]["get"]["responses"]["200"]["content"]["application/json"];
type VersionResponse = paths["/version"]["get"]["responses"]["200"]["content"]["application/json"];
type MeResponse = components["schemas"]["MeResponse"];
type ListOrgsResponse = components["schemas"]["ListOrgsResponse"];
type OrgSummary = components["schemas"]["OrgSummary"];
type OrgMembersResponse = components["schemas"]["OrgMembersResponse"];
type LicenseSnapshot = components["schemas"]["LicenseSnapshot"];
type MutationResponse = components["schemas"]["MutationResponse"];
type CreateOrgRequest = components["schemas"]["CreateOrgRequest"];
type UpdateLicenseRequest = components["schemas"]["UpdateLicenseRequest"];
type UpdateMemberRoleRequest = components["schemas"]["UpdateMemberRoleRequest"];

export type OrgRole = components["schemas"]["OrgRole"];
export type LicenseTier = components["schemas"]["LicenseTier"];
export type LicenseStatus = components["schemas"]["LicenseStatus"];
export type { MeResponse, ListOrgsResponse, OrgSummary, OrgMembersResponse, LicenseSnapshot };

export type JobStatus = "queued" | "running" | "done" | "failed";

export type ParseJobResponse = {
  job_id: number;
  org_id: number;
  file_name: string;
  status: JobStatus;
  total_lines: number;
  parsed_lines: number;
  benchmark_count: number;
  error_message: string | null;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
};

export type UploadResponse = {
  jobs: ParseJobResponse[];
};

export type ListJobsResponse = {
  jobs: ParseJobResponse[];
};

export type ListJobsParams = {
  status?: string;
};

export type BenchmarkSnapshot = {
  sequence: number;
  label: string;
  query_rows: number;
  query_rows_limit: number;
  query_rows_delta: number;
  heap_size_pct: number;
  heap_size_bytes_limit: number;
  heap_size_delta: number;
  cpu_time_ms: number;
  cpu_time_limit: number;
  cpu_time_delta: number;
  dml_statements: number;
  dml_statements_limit: number;
  soql_queries: number;
  soql_queries_limit: number;
};

export type ListBenchmarksResponse = {
  benchmarks: BenchmarkSnapshot[];
};

export type LogEvent = {
  line_index: number;
  timestamp: string;
  nanos: number | null;
  event_type: string;
  line_number: number | null;
  log_level: string | null;
  message: string;
};

export type ListEventsResponse = {
  events: LogEvent[];
  total: number;
};

export type ListEventsParams = {
  offset?: number;
  limit?: number;
  event_type?: string;
  log_level?: string;
  search?: string;
};

export type AuthContext = {
  authSubject: string;
  email?: string;
};

type RequestOptions = {
  method?: "GET" | "POST" | "PATCH";
  auth?: AuthContext;
  body?: unknown;
};

const defaultBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";

function buildAuthHeaders(auth?: AuthContext): Record<string, string> {
  if (!auth) {
    return {};
  }

  const headers: Record<string, string> = {
    "x-loglens-auth-sub": auth.authSubject,
  };

  if (auth.email) {
    headers["x-loglens-auth-email"] = auth.email;
  }

  return headers;
}

async function requestJson<T>(url: string, options: RequestOptions = {}): Promise<T> {
  const method = options.method ?? "GET";
  const hasBody = options.body !== undefined;
  const response = await fetch(url, {
    method,
    cache: "no-store",
    headers: {
      ...buildAuthHeaders(options.auth),
      ...(hasBody ? { "content-type": "application/json" } : {}),
    },
    ...(hasBody ? { body: JSON.stringify(options.body) } : {}),
  });

  if (!response.ok) {
    const fallback = `Request failed: ${response.status} ${response.statusText}`;
    let message = fallback;
    try {
      const error = (await response.json()) as { error?: string };
      if (error.error) {
        message = error.error;
      }
    } catch {
      // Response body was not valid JSON; use the fallback
    }
    throw new Error(message);
  }

  return (await response.json()) as T;
}

export function buildUrl(baseUrl: string, path: string): string {
  const normalizedBase = baseUrl.replace(/\/+$/, "");
  return `${normalizedBase}${path}`;
}

export function getHealth(baseUrl = defaultBaseUrl): Promise<HealthResponse> {
  return requestJson<HealthResponse>(buildUrl(baseUrl, "/health"));
}

export function getVersion(baseUrl = defaultBaseUrl): Promise<VersionResponse> {
  return requestJson<VersionResponse>(buildUrl(baseUrl, "/version"));
}

export function getMe(auth: AuthContext, baseUrl = defaultBaseUrl): Promise<MeResponse> {
  return requestJson<MeResponse>(buildUrl(baseUrl, "/v1/me"), { auth });
}

export function updateMeLicense(
  auth: AuthContext,
  license: UpdateLicenseRequest,
  baseUrl = defaultBaseUrl,
): Promise<LicenseSnapshot> {
  return requestJson<LicenseSnapshot>(buildUrl(baseUrl, "/v1/me/license"), {
    method: "PATCH",
    auth,
    body: license,
  });
}

export function listOrgs(auth: AuthContext, baseUrl = defaultBaseUrl): Promise<ListOrgsResponse> {
  return requestJson<ListOrgsResponse>(buildUrl(baseUrl, "/v1/orgs"), { auth });
}

export function createOrg(
  auth: AuthContext,
  payload: CreateOrgRequest,
  baseUrl = defaultBaseUrl,
): Promise<OrgSummary> {
  return requestJson<OrgSummary>(buildUrl(baseUrl, "/v1/orgs"), {
    method: "POST",
    auth,
    body: payload,
  });
}

export function getOrg(
  auth: AuthContext,
  orgId: number,
  baseUrl = defaultBaseUrl,
): Promise<OrgSummary> {
  return requestJson<OrgSummary>(buildUrl(baseUrl, `/v1/orgs/${orgId}`), { auth });
}

export function listOrgMembers(
  auth: AuthContext,
  orgId: number,
  baseUrl = defaultBaseUrl,
): Promise<OrgMembersResponse> {
  return requestJson<OrgMembersResponse>(buildUrl(baseUrl, `/v1/orgs/${orgId}/members`), { auth });
}

export function updateOrgLicense(
  auth: AuthContext,
  orgId: number,
  license: UpdateLicenseRequest,
  baseUrl = defaultBaseUrl,
): Promise<LicenseSnapshot> {
  return requestJson<LicenseSnapshot>(buildUrl(baseUrl, `/v1/orgs/${orgId}/license`), {
    method: "PATCH",
    auth,
    body: license,
  });
}

export function updateOrgMemberRole(
  auth: AuthContext,
  orgId: number,
  memberUserId: number,
  payload: UpdateMemberRoleRequest,
  baseUrl = defaultBaseUrl,
): Promise<MutationResponse> {
  return requestJson<MutationResponse>(
    buildUrl(baseUrl, `/v1/orgs/${orgId}/members/${memberUserId}/role`),
    {
      method: "PATCH",
      auth,
      body: payload,
    },
  );
}

export async function uploadLogs(
  auth: AuthContext,
  orgId: number,
  files: File[],
  baseUrl = defaultBaseUrl,
): Promise<UploadResponse> {
  const formData = new FormData();
  for (const file of files) {
    formData.append("file", file);
  }

  const response = await fetch(buildUrl(baseUrl, `/v1/orgs/${orgId}/uploads`), {
    method: "POST",
    cache: "no-store",
    headers: buildAuthHeaders(auth),
    body: formData,
  });

  if (!response.ok) {
    const fallback = `Upload failed: ${response.status} ${response.statusText}`;
    let message = fallback;
    try {
      const error = (await response.json()) as { error?: string };
      if (error.error) {
        message = error.error;
      }
    } catch {
      // Response body was not valid JSON; use the fallback
    }
    throw new Error(message);
  }

  return (await response.json()) as UploadResponse;
}

export function listJobs(
  auth: AuthContext,
  orgId: number,
  params: ListJobsParams = {},
  baseUrl = defaultBaseUrl,
): Promise<ListJobsResponse> {
  const searchParams = new URLSearchParams();
  if (params.status) searchParams.set("status", params.status);
  const qs = searchParams.toString();
  const path = `/v1/orgs/${orgId}/jobs${qs ? `?${qs}` : ""}`;
  return requestJson<ListJobsResponse>(buildUrl(baseUrl, path), { auth });
}

export function getJob(
  auth: AuthContext,
  orgId: number,
  jobId: number,
  baseUrl = defaultBaseUrl,
): Promise<ParseJobResponse> {
  return requestJson<ParseJobResponse>(
    buildUrl(baseUrl, `/v1/orgs/${orgId}/jobs/${jobId}`),
    { auth },
  );
}

export function listJobBenchmarks(
  auth: AuthContext,
  orgId: number,
  jobId: number,
  baseUrl = defaultBaseUrl,
): Promise<ListBenchmarksResponse> {
  return requestJson<ListBenchmarksResponse>(
    buildUrl(baseUrl, `/v1/orgs/${orgId}/jobs/${jobId}/benchmarks`),
    { auth },
  );
}

export function listJobEvents(
  auth: AuthContext,
  orgId: number,
  jobId: number,
  params: ListEventsParams = {},
  baseUrl = defaultBaseUrl,
): Promise<ListEventsResponse> {
  const searchParams = new URLSearchParams();
  if (params.offset !== undefined) searchParams.set("offset", String(params.offset));
  if (params.limit !== undefined) searchParams.set("limit", String(params.limit));
  if (params.event_type) searchParams.set("event_type", params.event_type);
  if (params.log_level) searchParams.set("log_level", params.log_level);
  if (params.search) searchParams.set("search", params.search);

  const qs = searchParams.toString();
  const path = `/v1/orgs/${orgId}/jobs/${jobId}/events${qs ? `?${qs}` : ""}`;
  return requestJson<ListEventsResponse>(buildUrl(baseUrl, path), { auth });
}

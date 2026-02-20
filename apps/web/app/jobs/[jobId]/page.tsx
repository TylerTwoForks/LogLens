import {
  getJob,
  listJobBenchmarks,
  listJobEvents,
  getEventSummary,
} from "@loglens/api-client";
import type {
  JobStatus,
  ListBenchmarksResponse,
  ListEventsResponse,
  EventSummaryResponse,
} from "@loglens/api-client";
import { buildApiAuthContext, requireSession } from "../../../lib/auth";
import JobDetailClient from "./JobDetailClient";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

export const dynamic = "force-dynamic";

type PageProps = {
  params: Promise<{ jobId: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function readServerApiBaseUrl() {
  return (
    process.env.API_INTERNAL_URL ??
    process.env.NEXT_PUBLIC_API_BASE_URL ??
    "http://localhost:8080"
  );
}

function readBrowserApiBaseUrl() {
  return "/api/proxy";
}

const STATUS_VARIANT: Record<
  JobStatus,
  { variant: "default" | "secondary" | "outline" | "destructive"; label: string; className?: string }
> = {
  queued: { variant: "secondary", label: "Queued" },
  running: { variant: "outline", label: "Processing", className: "border-success text-success" },
  done: { variant: "outline", label: "Done", className: "border-success text-success" },
  failed: { variant: "destructive", label: "Failed" },
};

function StatusBadge({ status }: { status: JobStatus }) {
  const s = STATUS_VARIANT[status] ?? STATUS_VARIANT.queued;
  return (
    <Badge variant={s.variant} className={s.className}>
      {s.label}
    </Badge>
  );
}

export default async function JobDetailPage({
  params,
  searchParams,
}: PageProps) {
  const { jobId: jobIdParam } = await params;
  const resolvedSearchParams = searchParams ? await searchParams : {};
  const orgIdParam = Array.isArray(resolvedSearchParams.org)
    ? resolvedSearchParams.org[0]
    : resolvedSearchParams.org;

  const jobId = Number(jobIdParam);
  const orgId = Number(orgIdParam);

  if (!Number.isFinite(jobId) || !Number.isFinite(orgId)) {
    return (
      <p className="p-8 text-sm text-destructive">
        Invalid job or organization ID.
      </p>
    );
  }

  const session = await requireSession();
  const auth = buildApiAuthContext(session);
  const serverApiUrl = readServerApiBaseUrl();
  const browserApiUrl = readBrowserApiBaseUrl();

  const job = await getJob(auth, orgId, jobId, serverApiUrl);
  const isDone = job.status === "done";
  const isFailed = job.status === "failed";
  const isPending = job.status === "queued" || job.status === "running";

  const emptySummary: EventSummaryResponse = {
    event_type_counts: [],
    timeline: [],
    total_events: 0,
    class_names: [],
  };

  let benchmarksRes: ListBenchmarksResponse = { benchmarks: [] };
  let eventsRes: ListEventsResponse = { events: [], total: 0 };
  let summaryRes = emptySummary;

  if (isDone) {
    const [benchResult, eventsResult, summaryResult] =
      await Promise.allSettled([
        listJobBenchmarks(auth, orgId, jobId, serverApiUrl),
        listJobEvents(auth, orgId, jobId, { limit: 200 }, serverApiUrl),
        getEventSummary(auth, orgId, jobId, 50, serverApiUrl),
      ]);

    if (benchResult.status === "fulfilled")
      benchmarksRes = benchResult.value;
    if (eventsResult.status === "fulfilled")
      eventsRes = eventsResult.value;
    if (summaryResult.status === "fulfilled")
      summaryRes = summaryResult.value;
  }

  return (
    <main className="mx-auto max-w-[1200px] p-6">
      <header className="mb-6 flex flex-wrap items-center gap-4">
        <div>
          <h1 className="text-2xl font-bold text-foreground">
            Job #{job.job_id}
          </h1>
          <p className="mt-1 text-sm text-muted-foreground">
            {job.file_name}
          </p>
        </div>
        <StatusBadge status={job.status} />
      </header>

      <Card className="mb-6">
        <CardContent className="pt-6">
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
            <dt className="text-muted-foreground">Total lines</dt>
            <dd className="text-foreground">
              {job.total_lines.toLocaleString()}
            </dd>
            <dt className="text-muted-foreground">Parsed lines</dt>
            <dd className="text-foreground">
              {job.parsed_lines.toLocaleString()}
            </dd>
            <dt className="text-muted-foreground">Benchmarks</dt>
            <dd className="text-foreground">{job.benchmark_count}</dd>
            {job.error_message && (
              <>
                <dt className="text-muted-foreground">Error</dt>
                <dd className="text-destructive">{job.error_message}</dd>
              </>
            )}
          </dl>
        </CardContent>
      </Card>

      {isFailed && (
        <Card className="mb-6 border-destructive">
          <CardContent className="pt-6">
            <p className="font-semibold text-destructive">Job failed</p>
            {job.error_message && (
              <p className="mt-2 text-sm text-destructive">
                {job.error_message}
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {isPending && (
        <div className="flex flex-col gap-6">
          <Card>
            <CardContent className="py-8 text-center">
              <p className="text-muted-foreground">
                {job.status === "queued"
                  ? "This job is queued and waiting to be processed..."
                  : "This job is currently being processed..."}
              </p>
              <p className="mt-2 text-xs text-muted-foreground/60">
                Refresh the page to check for updates.
              </p>
            </CardContent>
          </Card>
          <Skeleton className="h-[200px] w-full" />
          <Skeleton className="h-[300px] w-full" />
        </div>
      )}

      {isDone && (
        <JobDetailClient
          orgId={orgId}
          jobId={jobId}
          auth={auth}
          benchmarks={benchmarksRes.benchmarks}
          initialEvents={eventsRes.events}
          totalEvents={eventsRes.total}
          apiBaseUrl={browserApiUrl}
          fileName={job.file_name}
          eventSummary={summaryRes}
        />
      )}
    </main>
  );
}

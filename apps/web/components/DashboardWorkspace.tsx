"use client";

import {
  useState,
  useCallback,
  useRef,
  useEffect,
} from "react";
import type {
  ParseJobResponse,
  AuthContext,
  BenchmarkSnapshot,
  LogEvent,
  EventSummaryResponse,
  ListBenchmarksResponse,
  ListEventsResponse,
} from "@loglens/api-client";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { Upload, FileText, Loader2, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import JobDetailClient from "@/app/jobs/[jobId]/JobDetailClient";

type Props = {
  orgId: number;
  auth: AuthContext;
  initialJobs: ParseJobResponse[];
  apiBaseUrl: string;
};

type JobDetailData = {
  benchmarks: BenchmarkSnapshot[];
  events: LogEvent[];
  totalEvents: number;
  summary: EventSummaryResponse;
};

const STATUS_BADGE: Record<
  string,
  {
    variant: "default" | "secondary" | "outline" | "destructive";
    className?: string;
  }
> = {
  queued: { variant: "secondary" },
  running: { variant: "outline", className: "border-info text-info" },
  done: { variant: "outline", className: "border-success text-success" },
  failed: { variant: "destructive" },
};

export default function DashboardWorkspace({
  orgId,
  auth,
  initialJobs,
  apiBaseUrl,
}: Props) {
  const [jobs, setJobs] = useState<ParseJobResponse[]>(initialJobs);
  const [uploading, setUploading] = useState(false);
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const [selectedJobId, setSelectedJobId] = useState<number | null>(null);
  const [jobDetail, setJobDetail] = useState<JobDetailData | null>(null);
  const [jobDetailLoading, setJobDetailLoading] = useState(false);
  const [jobDetailError, setJobDetailError] = useState<string | null>(null);

  const retainedFiles = useRef<Map<string, string>>(new Map());

  const authHeaders: Record<string, string> = {
    "x-loglens-auth-sub": auth.authSubject,
    ...(auth.email ? { "x-loglens-auth-email": auth.email } : {}),
  };

  // ── Job polling ──────────────────────────────────────────────────────

  const fetchJobs = useCallback(async () => {
    try {
      const res = await fetch(`${apiBaseUrl}/v1/orgs/${orgId}/jobs`, {
        cache: "no-store",
        headers: authHeaders,
      });
      if (res.ok) {
        const data = (await res.json()) as { jobs: ParseJobResponse[] };
        setJobs(data.jobs);
      }
    } catch {
      // polling failure is non-critical
    }
  }, [apiBaseUrl, orgId, auth.authSubject, auth.email]);

  const hasActiveJobs = jobs.some(
    (j) => j.status === "queued" || j.status === "running",
  );

  useEffect(() => {
    if (hasActiveJobs) {
      if (!pollingRef.current) {
        pollingRef.current = setInterval(fetchJobs, 2000);
      }
    } else if (pollingRef.current) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, [hasActiveJobs, fetchJobs]);

  // ── Upload handling ──────────────────────────────────────────────────

  const handleUpload = async (files: FileList | File[]) => {
    const fileArray = Array.from(files).filter((f) => f.size > 0);
    if (fileArray.length === 0) return;

    setUploading(true);
    setUploadError(null);

    try {
      const textResults = await Promise.all(
        fileArray.map((f) => f.text()),
      );
      for (let i = 0; i < fileArray.length; i++) {
        retainedFiles.current.set(fileArray[i].name, textResults[i]);
      }

      const formData = new FormData();
      for (const file of fileArray) {
        formData.append("file", file);
      }

      const res = await fetch(`${apiBaseUrl}/v1/orgs/${orgId}/uploads`, {
        method: "POST",
        headers: authHeaders,
        body: formData,
      });

      if (!res.ok) {
        const body = await res.json().catch(() => null);
        throw new Error(body?.error ?? `Upload failed: ${res.status}`);
      }

      await fetchJobs();
    } catch (err) {
      setUploadError(err instanceof Error ? err.message : "Upload failed");
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  const onFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) handleUpload(e.target.files);
  };

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    if (e.dataTransfer.files.length > 0) handleUpload(e.dataTransfer.files);
  };

  // ── Fetch job detail data ────────────────────────────────────────────

  const fetchJobDetail = useCallback(
    async (jobId: number) => {
      setJobDetailLoading(true);
      setJobDetailError(null);
      setJobDetail(null);

      try {
        const [benchRes, eventsRes, summaryRes] = await Promise.all([
          fetch(
            `${apiBaseUrl}/v1/orgs/${orgId}/jobs/${jobId}/benchmarks`,
            { cache: "no-store", headers: authHeaders },
          ).then((r) => {
            if (!r.ok) throw new Error(`Benchmarks: ${r.status}`);
            return r.json() as Promise<ListBenchmarksResponse>;
          }),
          fetch(
            `${apiBaseUrl}/v1/orgs/${orgId}/jobs/${jobId}/events?limit=200`,
            { cache: "no-store", headers: authHeaders },
          ).then((r) => {
            if (!r.ok) throw new Error(`Events: ${r.status}`);
            return r.json() as Promise<ListEventsResponse>;
          }),
          fetch(
            `${apiBaseUrl}/v1/orgs/${orgId}/jobs/${jobId}/event-summary`,
            { cache: "no-store", headers: authHeaders },
          ).then((r) => {
            if (!r.ok) throw new Error(`Summary: ${r.status}`);
            return r.json() as Promise<EventSummaryResponse>;
          }),
        ]);

        setJobDetail({
          benchmarks: benchRes.benchmarks,
          events: eventsRes.events,
          totalEvents: eventsRes.total,
          summary: summaryRes,
        });
      } catch (err) {
        setJobDetailError(
          err instanceof Error ? err.message : "Failed to load job details",
        );
      } finally {
        setJobDetailLoading(false);
      }
    },
    [apiBaseUrl, orgId, auth.authSubject, auth.email],
  );

  // ── Job selection ────────────────────────────────────────────────────

  const handleSelectJob = useCallback(
    (job: ParseJobResponse) => {
      if (selectedJobId === job.job_id) {
        setSelectedJobId(null);
        setJobDetail(null);
        return;
      }

      setSelectedJobId(job.job_id);

      if (job.status === "done") {
        fetchJobDetail(job.job_id);
      } else {
        setJobDetail(null);
        setJobDetailError(null);
      }
    },
    [selectedJobId, fetchJobDetail],
  );

  // Auto-refresh detail when a selected job transitions to done
  const selectedJob = jobs.find((j) => j.job_id === selectedJobId) ?? null;
  const prevStatusRef = useRef<string | null>(null);

  useEffect(() => {
    if (!selectedJob) {
      prevStatusRef.current = null;
      return;
    }
    const prev = prevStatusRef.current;
    prevStatusRef.current = selectedJob.status;

    if (
      prev &&
      prev !== "done" &&
      selectedJob.status === "done"
    ) {
      fetchJobDetail(selectedJob.job_id);
    }
  }, [selectedJob?.status, selectedJob?.job_id, fetchJobDetail]);

  // ── Retained file content for selected job ───────────────────────────

  const retainedContent =
    selectedJob && retainedFiles.current.has(selectedJob.file_name)
      ? retainedFiles.current.get(selectedJob.file_name)!
      : undefined;

  // ── Delete job ───────────────────────────────────────────────────────

  const handleDeleteJob = async (jobId: number, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const res = await fetch(
        `${apiBaseUrl}/v1/orgs/${orgId}/jobs/${jobId}`,
        { method: "DELETE", headers: authHeaders },
      );
      if (!res.ok) {
        const body = await res.json().catch(() => null);
        throw new Error(body?.error ?? `Delete failed: ${res.status}`);
      }
      if (selectedJobId === jobId) {
        setSelectedJobId(null);
        setJobDetail(null);
      }
      await fetchJobs();
    } catch {
      // non-critical — next poll will reconcile
    }
  };

  // ── Render ───────────────────────────────────────────────────────────

  return (
    <div className="space-y-6">
      {/* Top row: upload zone + job list */}
      <div className="grid grid-cols-1 items-start gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,2fr)]">
        {/* Left: Upload drop zone */}
        <div className="space-y-3">
          <h3 className="text-lg font-semibold text-foreground">
            Upload Logs
          </h3>
          <Card
            className={cn(
              "flex cursor-pointer items-center justify-center border-2 border-dashed transition-colors",
              dragOver
                ? "border-primary bg-primary/5"
                : "border-border bg-card hover:border-muted-foreground",
              uploading && "cursor-wait opacity-70",
            )}
            onDragOver={(e) => {
              e.preventDefault();
              setDragOver(true);
            }}
            onDragLeave={() => setDragOver(false)}
            onDrop={onDrop}
            onClick={() => !uploading && fileInputRef.current?.click()}
          >
            <CardContent className="py-10 text-center">
              <input
                ref={fileInputRef}
                type="file"
                multiple
                accept=".log,.txt"
                onChange={onFileChange}
                className="hidden"
              />
              {uploading ? (
                <div className="flex flex-col items-center gap-2">
                  <Loader2 className="h-8 w-8 animate-spin text-primary" />
                  <p className="text-sm text-muted-foreground">
                    Uploading...
                  </p>
                </div>
              ) : (
                <div className="flex flex-col items-center gap-2">
                  <Upload className="h-8 w-8 text-muted-foreground" />
                  <p className="text-sm text-muted-foreground">
                    Drop Salesforce debug log files here, or click to browse.
                  </p>
                  <span className="text-xs text-muted-foreground/70">
                    Accepts .log and .txt files
                  </span>
                </div>
              )}
            </CardContent>
          </Card>
          {uploadError && (
            <p className="text-sm text-destructive">{uploadError}</p>
          )}
        </div>

        {/* Right: Job list */}
        <div className="space-y-3">
          <h3 className="text-lg font-semibold text-foreground">
            Parse Jobs
          </h3>
          <Card className="h-[320px]">
            {jobs.length === 0 ? (
              <CardContent className="flex h-full items-center justify-center">
                <div className="flex flex-col items-center gap-2 text-center">
                  <FileText className="h-8 w-8 text-muted-foreground/50" />
                  <p className="text-sm text-muted-foreground">
                    No jobs yet. Upload a log to get started.
                  </p>
                </div>
              </CardContent>
            ) : (
              <div className="h-full overflow-y-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>File</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead className="hidden sm:table-cell">
                        Lines
                      </TableHead>
                      <TableHead className="hidden md:table-cell">
                        Created
                      </TableHead>
                      <TableHead className="w-10" />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {jobs.map((job) => {
                      const badge =
                        STATUS_BADGE[job.status] ?? STATUS_BADGE.queued;
                      const isSelected = selectedJobId === job.job_id;
                      return (
                        <TableRow
                          key={job.job_id}
                          className={cn(
                            "cursor-pointer transition-colors",
                            isSelected && "bg-accent/10",
                          )}
                          onClick={() => handleSelectJob(job)}
                        >
                          <TableCell className="max-w-[180px] truncate font-medium">
                            {job.file_name}
                          </TableCell>
                          <TableCell>
                            <Badge
                              variant={badge.variant}
                              className={badge.className}
                            >
                              {job.status}
                            </Badge>
                          </TableCell>
                          <TableCell className="hidden text-muted-foreground sm:table-cell">
                            {job.status === "done"
                              ? `${job.parsed_lines.toLocaleString()} / ${job.total_lines.toLocaleString()}`
                              : "-"}
                          </TableCell>
                          <TableCell className="hidden text-muted-foreground md:table-cell">
                            {new Date(job.created_at)
                              .toISOString()
                              .replace("T", " ")
                              .slice(0, 19) + " UTC"}
                          </TableCell>
                          <TableCell className="w-10 px-2">
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7 text-muted-foreground hover:text-destructive"
                              onClick={(e) =>
                                handleDeleteJob(job.job_id, e)
                              }
                            >
                              <Trash2 className="h-4 w-4" />
                              <span className="sr-only">Delete job</span>
                            </Button>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
            )}
          </Card>
        </div>
      </div>

      {/* Bottom: Inline job detail */}
      {selectedJob && (
        <SelectedJobPanel
          job={selectedJob}
          loading={jobDetailLoading}
          error={jobDetailError}
          detail={jobDetail}
          orgId={orgId}
          auth={auth}
          apiBaseUrl={apiBaseUrl}
          retainedFileContent={retainedContent}
        />
      )}
    </div>
  );
}

// ── Selected Job Panel ───────────────────────────────────────────────────

type SelectedJobPanelProps = {
  job: ParseJobResponse;
  loading: boolean;
  error: string | null;
  detail: JobDetailData | null;
  orgId: number;
  auth: AuthContext;
  apiBaseUrl: string;
  retainedFileContent?: string;
};

function SelectedJobPanel({
  job,
  loading,
  error,
  detail,
  orgId,
  auth,
  apiBaseUrl,
  retainedFileContent,
}: SelectedJobPanelProps) {
  const badge = STATUS_BADGE[job.status] ?? STATUS_BADGE.queued;

  return (
    <Card>
      <CardHeader className="pb-4">
        <div className="flex items-center gap-3">
          <CardTitle className="text-lg">{job.file_name}</CardTitle>
          <Badge variant={badge.variant} className={badge.className}>
            {job.status}
          </Badge>
        </div>
        <p className="text-xs text-muted-foreground">
          Job #{job.job_id} &middot; Created{" "}
          {new Date(job.created_at).toLocaleString()}
        </p>
      </CardHeader>
      <CardContent>
        <JobDetailBody
          job={job}
          loading={loading}
          error={error}
          detail={detail}
          orgId={orgId}
          auth={auth}
          apiBaseUrl={apiBaseUrl}
          retainedFileContent={retainedFileContent}
        />
      </CardContent>
    </Card>
  );
}

function JobDetailBody({
  job,
  loading,
  error,
  detail,
  orgId,
  auth,
  apiBaseUrl,
  retainedFileContent,
}: SelectedJobPanelProps) {
  if (job.status === "queued" || job.status === "running") {
    return (
      <div className="flex items-center gap-3 py-8 text-center">
        <Loader2 className="mx-auto h-6 w-6 animate-spin text-info" />
        <p className="text-sm text-muted-foreground">
          {job.status === "queued" ? "Waiting in queue..." : "Processing..."}
        </p>
      </div>
    );
  }

  if (job.status === "failed") {
    return (
      <div className="rounded-md border border-destructive bg-destructive/5 px-4 py-6 text-center">
        <p className="text-sm font-medium text-destructive">Parse failed</p>
        {job.error_message && (
          <p className="mt-1 text-xs text-destructive/80">
            {job.error_message}
          </p>
        )}
      </div>
    );
  }

  if (loading) {
    return (
      <div className="space-y-4 py-4">
        <Skeleton className="h-8 w-full" />
        <Skeleton className="h-[200px] w-full" />
        <Skeleton className="h-[300px] w-full" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-md border border-destructive bg-destructive/5 px-4 py-6 text-center">
        <p className="text-sm text-destructive">{error}</p>
      </div>
    );
  }

  if (!detail) return null;

  return (
    <JobDetailClient
      orgId={orgId}
      jobId={job.job_id}
      auth={auth}
      benchmarks={detail.benchmarks}
      initialEvents={detail.events}
      totalEvents={detail.totalEvents}
      apiBaseUrl={apiBaseUrl}
      fileName={job.file_name}
      eventSummary={detail.summary}
      retainedFileContent={retainedFileContent}
    />
  );
}

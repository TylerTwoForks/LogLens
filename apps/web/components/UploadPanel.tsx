"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import type { ParseJobResponse, AuthContext } from "@loglens/api-client";
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
import { cn } from "@/lib/utils";
import Link from "next/link";

type Props = {
  orgId: number;
  auth: AuthContext;
  initialJobs: ParseJobResponse[];
  apiBaseUrl: string;
};

const STATUS_BADGE: Record<
  string,
  { variant: "default" | "secondary" | "outline" | "destructive"; className?: string }
> = {
  queued: { variant: "secondary" },
  running: { variant: "outline", className: "border-info text-info" },
  done: { variant: "outline", className: "border-success text-success" },
  failed: { variant: "destructive" },
};

export default function UploadPanel({
  orgId,
  auth,
  initialJobs,
  apiBaseUrl,
}: Props) {
  const [jobs, setJobs] = useState<ParseJobResponse[]>(initialJobs);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const authHeaders: Record<string, string> = {
    "x-loglens-auth-sub": auth.authSubject,
    ...(auth.email ? { "x-loglens-auth-email": auth.email } : {}),
  };

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
      // Silently fail polling; jobs will refresh next cycle
    }
  }, [apiBaseUrl, orgId, auth.authSubject, auth.email]);

  const hasActiveJobs = jobs.some(
    (j) => j.status === "queued" || j.status === "running"
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

  const handleUpload = async (files: FileList | File[]) => {
    const fileArray = Array.from(files).filter((f) => f.size > 0);
    if (fileArray.length === 0) return;

    setUploading(true);
    setError(null);

    const formData = new FormData();
    for (const file of fileArray) {
      formData.append("file", file);
    }

    try {
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
      setError(err instanceof Error ? err.message : "Upload failed");
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  const onFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      handleUpload(e.target.files);
    }
  };

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    if (e.dataTransfer.files.length > 0) {
      handleUpload(e.dataTransfer.files);
    }
  };

  return (
    <div className="mt-6 space-y-4">
      <h3 className="text-lg font-semibold text-foreground">Log Upload</h3>

      <Card
        className={cn(
          "cursor-pointer border-2 border-dashed transition-colors",
          dragOver
            ? "border-primary bg-primary/5"
            : "border-border bg-card hover:border-muted-foreground",
          uploading && "cursor-wait opacity-70"
        )}
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={onDrop}
        onClick={() => !uploading && fileInputRef.current?.click()}
      >
        <CardContent className="py-8 text-center">
          <input
            ref={fileInputRef}
            type="file"
            multiple
            accept=".log,.txt"
            onChange={onFileChange}
            className="hidden"
          />
          {uploading ? (
            <p className="text-sm text-muted-foreground">Uploading...</p>
          ) : (
            <p className="text-sm text-muted-foreground">
              Drop Salesforce debug log files here, or click to browse.
              <br />
              <span className="text-xs">Accepts .log and .txt files</span>
            </p>
          )}
        </CardContent>
      </Card>

      {error && (
        <p className="text-sm text-destructive">{error}</p>
      )}

      {jobs.length > 0 && (
        <div>
          <h4 className="mb-2 text-sm font-semibold text-foreground">
            Parse Jobs
          </h4>
          <div className="overflow-x-auto rounded-md border border-border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>ID</TableHead>
                  <TableHead>File</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Lines</TableHead>
                  <TableHead>Benchmarks</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead />
                </TableRow>
              </TableHeader>
              <TableBody>
                {jobs.map((job) => {
                  const badge = STATUS_BADGE[job.status] ?? STATUS_BADGE.queued;
                  return (
                    <TableRow key={job.job_id}>
                      <TableCell>{job.job_id}</TableCell>
                      <TableCell className="max-w-[200px] truncate">
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
                      <TableCell>
                        {job.status === "done"
                          ? `${job.parsed_lines.toLocaleString()} / ${job.total_lines.toLocaleString()}`
                          : "-"}
                      </TableCell>
                      <TableCell>
                        {job.status === "done" ? job.benchmark_count : "-"}
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {new Date(job.created_at)
                          .toISOString()
                          .replace("T", " ")
                          .slice(0, 19) + " UTC"}
                      </TableCell>
                      <TableCell>
                        {job.status === "done" && (
                          <Link
                            href={`/jobs/${job.job_id}?org=${orgId}`}
                            className="text-sm text-primary hover:underline"
                          >
                            View
                          </Link>
                        )}
                        {job.status === "failed" && job.error_message && (
                          <span
                            title={job.error_message}
                            className="cursor-help text-sm text-destructive"
                          >
                            Error
                          </span>
                        )}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        </div>
      )}
    </div>
  );
}

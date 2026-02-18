"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import type { ParseJobResponse, AuthContext } from "@loglens/api-client";

type Props = {
  orgId: number;
  auth: AuthContext;
  initialJobs: ParseJobResponse[];
  apiBaseUrl: string;
};

function statusBadge(status: string) {
  const colors: Record<string, { bg: string; fg: string }> = {
    queued: { bg: "#3b3b00", fg: "#facc15" },
    running: { bg: "#002a3b", fg: "#38bdf8" },
    done: { bg: "#003b1a", fg: "#4ade80" },
    failed: { bg: "#3b0000", fg: "#f87171" },
  };
  const c = colors[status] ?? { bg: "#333", fg: "#aaa" };
  return (
    <span
      style={{
        display: "inline-block",
        padding: "0.125rem 0.5rem",
        borderRadius: "9999px",
        fontSize: "0.75rem",
        fontWeight: 600,
        backgroundColor: c.bg,
        color: c.fg,
      }}
    >
      {status}
    </span>
  );
}

export default function UploadPanel({ orgId, auth, initialJobs, apiBaseUrl }: Props) {
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

  const hasActiveJobs = jobs.some((j) => j.status === "queued" || j.status === "running");

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
    <section style={{ marginTop: "1.5rem" }}>
      <h3 style={{ marginBottom: "0.75rem" }}>Log Upload</h3>

      <div
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={onDrop}
        style={{
          border: `2px dashed ${dragOver ? "#6366f1" : "#444"}`,
          borderRadius: "8px",
          padding: "1.5rem",
          textAlign: "center",
          backgroundColor: dragOver ? "#1e1b4b" : "#111",
          transition: "border-color 150ms, background-color 150ms",
          cursor: uploading ? "wait" : "pointer",
        }}
        onClick={() => !uploading && fileInputRef.current?.click()}
      >
        <input
          ref={fileInputRef}
          type="file"
          multiple
          accept=".log,.txt"
          onChange={onFileChange}
          style={{ display: "none" }}
        />
        {uploading ? (
          <p style={{ color: "#888", margin: 0 }}>Uploading...</p>
        ) : (
          <p style={{ color: "#888", margin: 0 }}>
            Drop Salesforce debug log files here, or click to browse.
            <br />
            <span style={{ fontSize: "0.75rem" }}>Accepts .log and .txt files</span>
          </p>
        )}
      </div>

      {error && (
        <p style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.5rem" }}>{error}</p>
      )}

      {jobs.length > 0 && (
        <div style={{ marginTop: "1rem" }}>
          <h4 style={{ marginBottom: "0.5rem" }}>Parse Jobs</h4>
          <table
            style={{
              width: "100%",
              borderCollapse: "collapse",
              fontSize: "0.875rem",
            }}
          >
            <thead>
              <tr
                style={{
                  borderBottom: "1px solid #333",
                  textAlign: "left",
                }}
              >
                <th style={{ padding: "0.375rem 0.5rem" }}>ID</th>
                <th style={{ padding: "0.375rem 0.5rem" }}>File</th>
                <th style={{ padding: "0.375rem 0.5rem" }}>Status</th>
                <th style={{ padding: "0.375rem 0.5rem" }}>Lines</th>
                <th style={{ padding: "0.375rem 0.5rem" }}>Benchmarks</th>
                <th style={{ padding: "0.375rem 0.5rem" }}>Created</th>
                <th style={{ padding: "0.375rem 0.5rem" }}></th>
              </tr>
            </thead>
            <tbody>
              {jobs.map((job) => (
                <tr key={job.job_id} style={{ borderBottom: "1px solid #222" }}>
                  <td style={{ padding: "0.375rem 0.5rem" }}>{job.job_id}</td>
                  <td
                    style={{
                      padding: "0.375rem 0.5rem",
                      maxWidth: "200px",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {job.file_name}
                  </td>
                  <td style={{ padding: "0.375rem 0.5rem" }}>
                    {statusBadge(job.status)}
                  </td>
                  <td style={{ padding: "0.375rem 0.5rem" }}>
                    {job.status === "done"
                      ? `${job.parsed_lines.toLocaleString()} / ${job.total_lines.toLocaleString()}`
                      : "-"}
                  </td>
                  <td style={{ padding: "0.375rem 0.5rem" }}>
                    {job.status === "done" ? job.benchmark_count : "-"}
                  </td>
                  <td style={{ padding: "0.375rem 0.5rem", color: "#888" }}>
                    {new Date(job.created_at).toLocaleString()}
                  </td>
                  <td style={{ padding: "0.375rem 0.5rem" }}>
                    {job.status === "done" && (
                      <a
                        href={`/jobs/${job.job_id}?org=${orgId}`}
                        style={{ color: "#6366f1", textDecoration: "none" }}
                      >
                        View
                      </a>
                    )}
                    {job.status === "failed" && job.error_message && (
                      <span
                        title={job.error_message}
                        style={{ color: "#f87171", cursor: "help" }}
                      >
                        Error
                      </span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

"use client";

import { useState, useCallback } from "react";
import type { AuthContext, BenchmarkSnapshot, LogEvent } from "@loglens/api-client";
import GovernorLimitCharts from "../../../components/GovernorLimitCharts";
import BenchmarkTable from "../../../components/BenchmarkTable";
import LogViewer from "../../../components/LogViewer";

type Tab = "charts" | "table" | "logs";

type Props = {
  orgId: number;
  jobId: number;
  auth: AuthContext;
  benchmarks: BenchmarkSnapshot[];
  initialEvents: LogEvent[];
  totalEvents: number;
  apiBaseUrl: string;
};

export default function JobDetailClient({
  orgId,
  jobId,
  auth,
  benchmarks,
  initialEvents,
  totalEvents,
  apiBaseUrl,
}: Props) {
  const [tab, setTab] = useState<Tab>("charts");

  const authHeaders: Record<string, string> = {
    "x-loglens-auth-sub": auth.authSubject,
    ...(auth.email ? { "x-loglens-auth-email": auth.email } : {}),
  };

  const handleLoadMore = useCallback(
    async (params: {
      offset: number;
      limit: number;
      event_type?: string;
      log_level?: string;
      search?: string;
    }) => {
      const searchParams = new URLSearchParams();
      searchParams.set("offset", String(params.offset));
      searchParams.set("limit", String(params.limit));
      if (params.event_type) searchParams.set("event_type", params.event_type);
      if (params.log_level) searchParams.set("log_level", params.log_level);
      if (params.search) searchParams.set("search", params.search);

      const res = await fetch(
        `${apiBaseUrl}/v1/orgs/${orgId}/jobs/${jobId}/events?${searchParams.toString()}`,
        { cache: "no-store", headers: authHeaders },
      );

      if (!res.ok) {
        throw new Error(`Failed to load events: ${res.status}`);
      }

      return (await res.json()) as { events: LogEvent[]; total: number };
    },
    [apiBaseUrl, orgId, jobId, auth.authSubject, auth.email],
  );

  const tabStyle = (t: Tab) => ({
    padding: "0.5rem 1.25rem",
    cursor: "pointer" as const,
    background: "none",
    color: tab === t ? "#e5e5e5" : "#888",
    fontWeight: tab === t ? (600 as const) : (400 as const),
    fontSize: "0.875rem",
    borderTop: "none",
    borderLeft: "none",
    borderRight: "none",
    borderBottomWidth: "2px",
    borderBottomStyle: "solid" as const,
    borderBottomColor: tab === t ? "#6366f1" : "transparent",
  });

  return (
    <>
      <nav
        style={{
          display: "flex",
          gap: "0",
          borderBottom: "1px solid #333",
          marginBottom: "1.5rem",
        }}
      >
        <button onClick={() => setTab("charts")} style={tabStyle("charts")}>
          Governor Limit Charts
        </button>
        <button onClick={() => setTab("table")} style={tabStyle("table")}>
          Benchmark Data
        </button>
        <button onClick={() => setTab("logs")} style={tabStyle("logs")}>
          Log Viewer
        </button>
      </nav>

      {tab === "charts" && <GovernorLimitCharts benchmarks={benchmarks} />}
      {tab === "table" && <BenchmarkTable benchmarks={benchmarks} />}
      {tab === "logs" && (
        <LogViewer
          initialEvents={initialEvents}
          totalEvents={totalEvents}
          onLoadMore={handleLoadMore}
        />
      )}
    </>
  );
}

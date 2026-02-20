"use client";

import { useCallback, useState } from "react";
import type {
  AuthContext,
  BenchmarkSnapshot,
  LogEvent,
  EventSummaryResponse,
} from "@loglens/api-client";
import GovernorLimitCharts from "../../../components/GovernorLimitCharts";
import BenchmarkTable from "../../../components/BenchmarkTable";
import LogViewer from "../../../components/LogViewer";
import TimelineChart from "../../../components/TimelineChart";
import HotspotsChart from "../../../components/HotspotsChart";
import type { DisplayEvent } from "../../../components/LogViewer";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";

type Props = {
  orgId: number;
  jobId: number;
  auth: AuthContext;
  benchmarks: BenchmarkSnapshot[];
  initialEvents: LogEvent[];
  totalEvents: number;
  apiBaseUrl: string;
  fileName: string;
  eventSummary: EventSummaryResponse;
  retainedFileContent?: string;
};

const LOG_LEVELS = new Set([
  "ERROR",
  "WARN",
  "INFO",
  "DEBUG",
  "FINE",
  "FINER",
  "FINEST",
  "INTERNAL",
]);

function parseLogContent(content: string): DisplayEvent[] {
  const lines = content.split("\n");
  const events: DisplayEvent[] = [];

  for (let i = 0; i < lines.length; i++) {
    const trimmed = lines[i].trim();
    if (!trimmed) continue;

    const pipeIdx = trimmed.indexOf("|");
    if (pipeIdx === -1) continue;

    const timestampPart = trimmed.substring(0, pipeIdx);
    const tMatch = timestampPart.match(
      /^(\d{2}:\d{2}:\d{2}\.\d+)\s*\((\d+)\)/
    );
    if (!tMatch) continue;

    const rest = trimmed.substring(pipeIdx + 1);
    const segments = rest.split("|");
    const eventType = segments[0] ?? "";

    let lineNumber: number | null = null;
    let logLevel: string | null = null;
    let message = "";
    let messageStartIdx = 1;

    if (segments.length >= 2) {
      const bracketMatch = segments[1].match(/^\[(\d+)\]$/);
      if (bracketMatch) {
        lineNumber = parseInt(bracketMatch[1], 10);
        messageStartIdx = 2;
      }
    }

    if (messageStartIdx < segments.length) {
      const candidate = segments[messageStartIdx].trim();
      if (LOG_LEVELS.has(candidate)) {
        logLevel = candidate;
        messageStartIdx += 1;
      }
    }

    if (messageStartIdx < segments.length) {
      message = segments.slice(messageStartIdx).join("|");
    }

    events.push({
      line_index: i,
      timestamp: tMatch[1],
      nanos: parseInt(tMatch[2], 10),
      event_type: eventType,
      line_number: lineNumber,
      log_level: logLevel,
      class_name: null,
      message,
    });
  }

  return events;
}

export default function JobDetailClient({
  orgId,
  jobId,
  auth,
  benchmarks,
  initialEvents,
  totalEvents,
  apiBaseUrl,
  fileName,
  eventSummary,
  retainedFileContent,
}: Props) {
  const [reuploadedEvents, setReuploadedEvents] = useState<
    DisplayEvent[] | null
  >(() => (retainedFileContent ? parseLogContent(retainedFileContent) : null));
  const [reuploadError, setReuploadError] = useState<string | null>(null);

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
      class_name?: string;
    }) => {
      const searchParams = new URLSearchParams();
      searchParams.set("offset", String(params.offset));
      searchParams.set("limit", String(params.limit));
      if (params.event_type)
        searchParams.set("event_type", params.event_type);
      if (params.log_level)
        searchParams.set("log_level", params.log_level);
      if (params.search) searchParams.set("search", params.search);
      if (params.class_name)
        searchParams.set("class_name", params.class_name);

      const res = await fetch(
        `${apiBaseUrl}/v1/orgs/${orgId}/jobs/${jobId}/events?${searchParams.toString()}`,
        { cache: "no-store", headers: authHeaders }
      );

      if (!res.ok) {
        throw new Error(`Failed to load events (${res.status})`);
      }

      return (await res.json()) as { events: DisplayEvent[]; total: number };
    },
    [apiBaseUrl, orgId, jobId, auth.authSubject, auth.email]
  );

  const handleReupload = useCallback(
    (file: File) => {
      setReuploadError(null);

      if (file.name !== fileName) {
        setReuploadError(
          `File name mismatch: expected "${fileName}" but received "${file.name}". ` +
            "Please upload the same log file that was originally processed for this job."
        );
        return;
      }

      const reader = new FileReader();
      reader.onload = () => {
        const content = reader.result as string;
        const events = parseLogContent(content);
        setReuploadedEvents(events);
      };
      reader.onerror = () => {
        setReuploadError("Failed to read file.");
      };
      reader.readAsText(file);
    },
    [fileName]
  );

  const handleReuploadLoadMore = useCallback(
    async (params: {
      offset: number;
      limit: number;
      event_type?: string;
      log_level?: string;
      search?: string;
      class_name?: string;
    }) => {
      if (!reuploadedEvents)
        return { events: [] as DisplayEvent[], total: 0 };

      let filtered = reuploadedEvents;
      if (params.event_type) {
        filtered = filtered.filter(
          (e) => e.event_type === params.event_type
        );
      }
      if (params.log_level) {
        filtered = filtered.filter(
          (e) => e.log_level === params.log_level
        );
      }
      if (params.search) {
        const lower = params.search.toLowerCase();
        filtered = filtered.filter((e) =>
          (e.message ?? "").toLowerCase().includes(lower)
        );
      }

      return {
        events: filtered.slice(params.offset, params.offset + params.limit),
        total: filtered.length,
      };
    },
    [reuploadedEvents]
  );

  const isReuploadActive = reuploadedEvents !== null;
  const activeEvents = reuploadedEvents ?? initialEvents;
  const activeTotal = isReuploadActive
    ? reuploadedEvents.length
    : totalEvents;
  const activeLoadMore = isReuploadActive
    ? handleReuploadLoadMore
    : handleLoadMore;

  return (
    <Tabs defaultValue="overview">
      <TabsList variant="line">
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="charts">Governor Limits</TabsTrigger>
        <TabsTrigger value="table">Benchmark Data</TabsTrigger>
        <TabsTrigger value="logs">Log Viewer</TabsTrigger>
      </TabsList>

      <TabsContent value="overview" className="mt-6">
        <div className="flex flex-col gap-10">
          <TimelineChart timeline={eventSummary.timeline} />
          <HotspotsChart
            eventTypeCounts={eventSummary.event_type_counts}
            totalEvents={eventSummary.total_events}
          />
        </div>
      </TabsContent>

      <TabsContent value="charts" className="mt-6">
        <GovernorLimitCharts benchmarks={benchmarks} />
      </TabsContent>

      <TabsContent value="table" className="mt-6">
        <BenchmarkTable benchmarks={benchmarks} />
      </TabsContent>

      <TabsContent value="logs" className="mt-6">
        {reuploadError && (
          <p className="mb-2 text-sm text-destructive">{reuploadError}</p>
        )}
        <LogViewer
          initialEvents={activeEvents}
          totalEvents={activeTotal}
          fileName={fileName}
          classNames={eventSummary.class_names}
          onLoadMore={activeLoadMore}
          onReupload={handleReupload}
          hasMessages={isReuploadActive}
        />
      </TabsContent>
    </Tabs>
  );
}

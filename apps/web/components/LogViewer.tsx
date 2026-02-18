"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { LogEvent } from "@loglens/api-client";

type Props = {
  initialEvents: LogEvent[];
  totalEvents: number;
  onLoadMore: (params: {
    offset: number;
    limit: number;
    event_type?: string;
    log_level?: string;
    search?: string;
  }) => Promise<{ events: LogEvent[]; total: number }>;
};

const LOG_LEVEL_COLORS: Record<string, string> = {
  ERROR: "#ef4444",
  WARN: "#f59e0b",
  INFO: "#3b82f6",
  DEBUG: "#a3a3a3",
  FINE: "#6b7280",
  FINER: "#6b7280",
  FINEST: "#6b7280",
};

const EVENT_TYPE_OPTIONS = [
  "",
  "USER_DEBUG",
  "EXECUTION_STARTED",
  "EXECUTION_FINISHED",
  "CODE_UNIT_STARTED",
  "CODE_UNIT_FINISHED",
  "SOQL_EXECUTE_BEGIN",
  "SOQL_EXECUTE_END",
  "DML_BEGIN",
  "DML_END",
  "METHOD_ENTRY",
  "METHOD_EXIT",
];

const LOG_LEVEL_OPTIONS = [
  "",
  "ERROR",
  "WARN",
  "INFO",
  "DEBUG",
  "FINE",
  "FINER",
  "FINEST",
];

const PAGE_SIZE = 200;
const ROW_HEIGHT = 24;
const SEARCH_DEBOUNCE_MS = 400;

export default function LogViewer({
  initialEvents,
  totalEvents,
  onLoadMore,
}: Props) {
  const [events, setEvents] = useState(initialEvents);
  const [total, setTotal] = useState(totalEvents);
  const [loading, setLoading] = useState(false);
  const [wrap, setWrap] = useState(false);

  const [eventTypeFilter, setEventTypeFilter] = useState("");
  const [logLevelFilter, setLogLevelFilter] = useState("");
  const [searchText, setSearchText] = useState("");

  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(600);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latestFiltersRef = useRef({ eventType: "", logLevel: "", search: "" });

  useEffect(() => {
    if (containerRef.current) {
      setContainerHeight(containerRef.current.clientHeight);
    }
  }, []);

  const fetchFiltered = useCallback(
    async (eventType: string, logLevel: string, search: string) => {
      setLoading(true);
      try {
        const result = await onLoadMore({
          offset: 0,
          limit: PAGE_SIZE,
          event_type: eventType || undefined,
          log_level: logLevel || undefined,
          search: search || undefined,
        });
        setEvents(result.events);
        setTotal(result.total);
        setScrollTop(0);
        if (containerRef.current) {
          containerRef.current.scrollTop = 0;
        }
      } finally {
        setLoading(false);
      }
    },
    [onLoadMore],
  );

  const handleEventTypeChange = useCallback(
    (value: string) => {
      setEventTypeFilter(value);
      latestFiltersRef.current.eventType = value;
      fetchFiltered(value, latestFiltersRef.current.logLevel, latestFiltersRef.current.search);
    },
    [fetchFiltered],
  );

  const handleLogLevelChange = useCallback(
    (value: string) => {
      setLogLevelFilter(value);
      latestFiltersRef.current.logLevel = value;
      fetchFiltered(latestFiltersRef.current.eventType, value, latestFiltersRef.current.search);
    },
    [fetchFiltered],
  );

  const handleSearchChange = useCallback(
    (value: string) => {
      setSearchText(value);
      latestFiltersRef.current.search = value;
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
      searchTimerRef.current = setTimeout(() => {
        fetchFiltered(
          latestFiltersRef.current.eventType,
          latestFiltersRef.current.logLevel,
          value,
        );
      }, SEARCH_DEBOUNCE_MS);
    },
    [fetchFiltered],
  );

  useEffect(() => {
    return () => {
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    };
  }, []);

  const loadMoreEvents = useCallback(async () => {
    if (loading || events.length >= total) return;
    setLoading(true);
    try {
      const result = await onLoadMore({
        offset: events.length,
        limit: PAGE_SIZE,
        event_type: eventTypeFilter || undefined,
        log_level: logLevelFilter || undefined,
        search: searchText || undefined,
      });
      setEvents((prev) => [...prev, ...result.events]);
      setTotal(result.total);
    } finally {
      setLoading(false);
    }
  }, [loading, events.length, total, eventTypeFilter, logLevelFilter, searchText, onLoadMore]);

  const handleScroll = useCallback(() => {
    if (!containerRef.current) return;
    const el = containerRef.current;
    setScrollTop(el.scrollTop);

    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 400;
    if (nearBottom) {
      loadMoreEvents();
    }
  }, [loadMoreEvents]);

  const startIdx = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - 5);
  const visibleCount = Math.ceil(containerHeight / ROW_HEIGHT) + 10;
  const endIdx = Math.min(events.length, startIdx + visibleCount);
  const visibleEvents = wrap ? events : events.slice(startIdx, endIdx);

  const renderRow = (evt: LogEvent, globalIdx: number) => {
    const levelColor = LOG_LEVEL_COLORS[evt.log_level ?? ""] ?? "#a3a3a3";

    const positionStyle: React.CSSProperties = wrap
      ? { borderBottom: "1px solid #222", display: "flex", padding: "2px 0.5rem", minHeight: ROW_HEIGHT }
      : {
          position: "absolute",
          top: globalIdx * ROW_HEIGHT,
          left: 0,
          height: ROW_HEIGHT,
          display: "flex",
          padding: "0 0.5rem",
          borderBottom: "1px solid #222",
        };

    return (
      <div
        key={evt.line_index}
        style={{
          ...positionStyle,
          whiteSpace: wrap ? "pre-wrap" : "nowrap",
          wordBreak: wrap ? "break-all" : undefined,
          alignItems: "flex-start",
        }}
      >
        <span
          style={{
            width: "50px",
            minWidth: "50px",
            flexShrink: 0,
            color: "#555",
            textAlign: "right",
            marginRight: "0.75rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
        >
          {evt.line_index}
        </span>
        <span
          style={{
            width: "90px",
            minWidth: "90px",
            flexShrink: 0,
            color: "#6b7280",
            marginRight: "0.5rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
        >
          {evt.timestamp}
        </span>
        <span
          style={{
            width: "160px",
            minWidth: "160px",
            flexShrink: 0,
            color: "#a78bfa",
            marginRight: "0.5rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
        >
          {evt.event_type}
        </span>
        {evt.log_level && (
          <span
            style={{
              width: "50px",
              minWidth: "50px",
              flexShrink: 0,
              color: levelColor,
              fontWeight: evt.log_level === "ERROR" ? 700 : 400,
              marginRight: "0.5rem",
              lineHeight: `${ROW_HEIGHT}px`,
            }}
          >
            {evt.log_level}
          </span>
        )}
        <span style={{ color: "#d4d4d4", lineHeight: `${ROW_HEIGHT}px` }}>
          {evt.message}
        </span>
      </div>
    );
  };

  return (
    <div>
      <div
        style={{
          display: "flex",
          gap: "0.5rem",
          flexWrap: "wrap",
          marginBottom: "0.75rem",
          alignItems: "center",
        }}
      >
        <select
          value={eventTypeFilter}
          onChange={(e) => handleEventTypeChange(e.target.value)}
          style={{ fontSize: "0.8125rem", padding: "0.3rem" }}
        >
          <option value="">All Event Types</option>
          {EVENT_TYPE_OPTIONS.filter(Boolean).map((et) => (
            <option key={et} value={et}>
              {et}
            </option>
          ))}
        </select>

        <select
          value={logLevelFilter}
          onChange={(e) => handleLogLevelChange(e.target.value)}
          style={{ fontSize: "0.8125rem", padding: "0.3rem" }}
        >
          <option value="">All Levels</option>
          {LOG_LEVEL_OPTIONS.filter(Boolean).map((ll) => (
            <option key={ll} value={ll}>
              {ll}
            </option>
          ))}
        </select>

        <input
          type="text"
          placeholder="Search messages..."
          value={searchText}
          onChange={(e) => handleSearchChange(e.target.value)}
          style={{
            fontSize: "0.8125rem",
            padding: "0.3rem 0.5rem",
            width: "200px",
            border: "1px solid #555",
            borderRadius: "4px",
            background: "inherit",
            color: "inherit",
          }}
        />

        <label
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "0.35rem",
            fontSize: "0.8125rem",
            color: "#ccc",
            cursor: "pointer",
            userSelect: "none",
          }}
        >
          <input
            type="checkbox"
            checked={wrap}
            onChange={(e) => setWrap(e.target.checked)}
            style={{ accentColor: "#6366f1" }}
          />
          Wrap lines
        </label>

        {loading && (
          <span style={{ fontSize: "0.75rem", color: "#f59e0b" }}>Loading...</span>
        )}

        <span style={{ fontSize: "0.75rem", color: "#888", marginLeft: "auto" }}>
          {events.length} of {total} events loaded
        </span>
      </div>

      <div
        ref={containerRef}
        onScroll={handleScroll}
        style={{
          height: "600px",
          overflow: "auto",
          border: "1px solid #333",
          borderRadius: "4px",
          fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
          fontSize: "0.75rem",
          lineHeight: `${ROW_HEIGHT}px`,
          background: "#111",
        }}
      >
        {wrap ? (
          <div>{visibleEvents.map((evt, i) => renderRow(evt, i))}</div>
        ) : (
          <div
            style={{
              height: events.length * ROW_HEIGHT,
              position: "relative",
              minWidth: "fit-content",
            }}
          >
            {visibleEvents.map((evt, i) => renderRow(evt, startIdx + i))}
          </div>
        )}
      </div>
    </div>
  );
}

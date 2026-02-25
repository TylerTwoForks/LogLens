"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { LogEvent } from "@loglens/api-client";
import { Bookmark } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

export type DisplayEvent = LogEvent & { message?: string };

type Props = {
  initialEvents: DisplayEvent[];
  totalEvents: number;
  fileName?: string;
  classNames?: string[];
  onLoadMore: (params: {
    offset: number;
    limit: number;
    event_type?: string;
    log_level?: string;
    search?: string;
    class_name?: string;
  }) => Promise<{ events: DisplayEvent[]; total: number }>;
  onReupload?: (file: File) => void;
  hasMessages?: boolean;
};

const LOG_LEVEL_COLORS: Record<string, string> = {
  ERROR: "text-destructive",
  WARN: "text-chart-1",
  INFO: "text-info",
  DEBUG: "text-muted-foreground",
  FINE: "text-muted-foreground/70",
  FINER: "text-muted-foreground/70",
  FINEST: "text-muted-foreground/70",
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

const selectClasses =
  "h-8 rounded-md border border-input bg-background px-2 text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-ring";

export default function LogViewer({
  initialEvents,
  totalEvents,
  fileName,
  classNames = [],
  onLoadMore,
  onReupload,
  hasMessages = false,
}: Props) {
  const [events, setEvents] = useState<DisplayEvent[]>(initialEvents);
  const [total, setTotal] = useState(totalEvents);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wrap, setWrap] = useState(false);

  const [eventTypeFilter, setEventTypeFilter] = useState("");
  const [logLevelFilter, setLogLevelFilter] = useState("");
  const [classNameFilter, setClassNameFilter] = useState("");
  const [searchText, setSearchText] = useState("");

  const [bookmarked, setBookmarked] = useState<Map<number, DisplayEvent>>(new Map());
  const [showBookmarkedOnly, setShowBookmarkedOnly] = useState(false);

  const containerRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latestFiltersRef = useRef({
    eventType: "",
    logLevel: "",
    className: "",
    search: "",
  });
  const loadingRef = useRef(false);

  useEffect(() => {
    setEvents(initialEvents);
    setTotal(totalEvents);
  }, [initialEvents, totalEvents]);

  useEffect(() => {
    return () => {
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    };
  }, []);

  const displayedEvents = showBookmarkedOnly
    ? Array.from(bookmarked.values()).sort((a, b) => a.line_index - b.line_index)
    : events;

  const virtualizer = useVirtualizer({
    count: displayedEvents.length,
    getScrollElement: () => containerRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 20,
  });

  const toggleBookmark = useCallback((evt: DisplayEvent) => {
    setBookmarked((prev) => {
      const next = new Map(prev);
      if (next.has(evt.line_index)) {
        next.delete(evt.line_index);
      } else {
        next.set(evt.line_index, evt);
      }
      return next;
    });
  }, []);

  const clearBookmarks = useCallback(() => {
    setBookmarked(new Map());
    setShowBookmarkedOnly(false);
  }, []);

  const fetchFiltered = useCallback(
    async (
      eventType: string,
      logLevel: string,
      className: string,
      search: string
    ) => {
      setLoading(true);
      setError(null);
      loadingRef.current = true;
      try {
        const result = await onLoadMore({
          offset: 0,
          limit: PAGE_SIZE,
          event_type: eventType || undefined,
          log_level: logLevel || undefined,
          class_name: className || undefined,
          search: search || undefined,
        });
        setEvents(result.events);
        setTotal(result.total);
        if (containerRef.current) {
          containerRef.current.scrollTop = 0;
        }
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load events");
      } finally {
        setLoading(false);
        loadingRef.current = false;
      }
    },
    [onLoadMore]
  );

  const applyFilters = useCallback(() => {
    const f = latestFiltersRef.current;
    fetchFiltered(f.eventType, f.logLevel, f.className, f.search);
  }, [fetchFiltered]);

  const handleEventTypeChange = useCallback(
    (value: string) => {
      setEventTypeFilter(value);
      latestFiltersRef.current.eventType = value;
      applyFilters();
    },
    [applyFilters]
  );

  const handleLogLevelChange = useCallback(
    (value: string) => {
      setLogLevelFilter(value);
      latestFiltersRef.current.logLevel = value;
      applyFilters();
    },
    [applyFilters]
  );

  const handleClassNameChange = useCallback(
    (value: string) => {
      setClassNameFilter(value);
      latestFiltersRef.current.className = value;
      applyFilters();
    },
    [applyFilters]
  );

  const handleSearchChange = useCallback(
    (value: string) => {
      setSearchText(value);
      latestFiltersRef.current.search = value;
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
      searchTimerRef.current = setTimeout(() => {
        applyFilters();
      }, SEARCH_DEBOUNCE_MS);
    },
    [applyFilters]
  );

  const loadMoreEvents = useCallback(async () => {
    if (loadingRef.current || events.length >= total || showBookmarkedOnly)
      return;
    setLoading(true);
    loadingRef.current = true;
    try {
      const result = await onLoadMore({
        offset: events.length,
        limit: PAGE_SIZE,
        event_type: eventTypeFilter || undefined,
        log_level: logLevelFilter || undefined,
        class_name: classNameFilter || undefined,
        search: searchText || undefined,
      });
      setEvents((prev) => [...prev, ...result.events]);
      setTotal(result.total);
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to load more events"
      );
    } finally {
      setLoading(false);
      loadingRef.current = false;
    }
  }, [
    events.length,
    total,
    eventTypeFilter,
    logLevelFilter,
    classNameFilter,
    searchText,
    onLoadMore,
    showBookmarkedOnly,
  ]);

  const handleScroll = useCallback(() => {
    if (!containerRef.current) return;
    const el = containerRef.current;
    const nearBottom =
      el.scrollHeight - el.scrollTop - el.clientHeight < 400;
    if (nearBottom) {
      loadMoreEvents();
    }
  }, [loadMoreEvents]);

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file && onReupload) {
        onReupload(file);
      }
      e.target.value = "";
    },
    [onReupload]
  );

  const renderRow = (evt: DisplayEvent) => {
    const levelClass = LOG_LEVEL_COLORS[evt.log_level ?? ""] ?? "text-muted-foreground";
    const isBookmarked = bookmarked.has(evt.line_index);

    return (
      <>
        <button
          onClick={(e) => {
            e.stopPropagation();
            toggleBookmark(evt);
          }}
          className={cn(
            "shrink-0 flex items-center justify-center transition-colors",
            isBookmarked
              ? "text-amber-400"
              : "text-muted-foreground/20 hover:text-muted-foreground/50"
          )}
          style={{
            width: 20,
            minWidth: 20,
            marginRight: "0.25rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
          title={isBookmarked ? "Remove bookmark" : "Bookmark this line"}
        >
          <Bookmark
            className="h-3 w-3"
            fill={isBookmarked ? "currentColor" : "none"}
          />
        </button>
        <span
          className="shrink-0 text-right text-muted-foreground/50"
          style={{
            width: 50,
            minWidth: 50,
            marginRight: "0.75rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
        >
          {evt.line_index}
        </span>
        <span
          className="shrink-0 text-muted-foreground"
          style={{
            width: 90,
            minWidth: 90,
            marginRight: "0.5rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
        >
          {evt.timestamp}
        </span>
        <span
          className="shrink-0 text-accent"
          style={{
            width: 160,
            minWidth: 160,
            marginRight: "0.5rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
        >
          {evt.event_type}
        </span>
        <span
          className={cn(
            "shrink-0 overflow-hidden",
            evt.log_level ? levelClass : "text-transparent",
            evt.log_level === "ERROR" && "font-bold"
          )}
          style={{
            width: 60,
            minWidth: 60,
            marginRight: "0.5rem",
            lineHeight: `${ROW_HEIGHT}px`,
          }}
        >
          {evt.log_level ?? ""}
        </span>
        {hasMessages && (
          <span
            className="text-foreground"
            style={{ lineHeight: `${ROW_HEIGHT}px` }}
          >
            {evt.message ?? ""}
          </span>
        )}
      </>
    );
  };

  const virtualItems = virtualizer.getVirtualItems();

  return (
    <div>
      {/* Privacy re-upload banner */}
      {!hasMessages && (
        <div className="mb-3 flex flex-wrap items-center gap-3 rounded-md border border-border bg-muted px-4 py-3">
          <span className="flex-1 text-sm text-muted-foreground">
            Log messages are not stored for privacy.
            {fileName
              ? ` Re-upload "${fileName}" to view full message content.`
              : " Re-upload the same log file to view full message content."}
          </span>
          {onReupload && (
            <>
              <input
                ref={fileInputRef}
                type="file"
                accept=".log,.txt"
                onChange={handleFileSelect}
                className="hidden"
              />
              <Button
                size="sm"
                onClick={() => fileInputRef.current?.click()}
              >
                Re-upload log
              </Button>
            </>
          )}
        </div>
      )}

      {/* Active re-upload banner */}
      {hasMessages && (
        <div className="mb-3 flex flex-wrap items-center gap-3 rounded-md border border-success bg-success/10 px-4 py-3">
          <span className="flex-1 text-sm text-success">
            Viewing log messages from re-uploaded file (in-memory only, not
            stored).
          </span>
          {onReupload && (
            <>
              <input
                ref={fileInputRef}
                type="file"
                accept=".log,.txt"
                onChange={handleFileSelect}
                className="hidden"
              />
              <Button
                variant="outline"
                size="sm"
                className="border-success text-success hover:bg-success/10"
                onClick={() => fileInputRef.current?.click()}
              >
                Upload different file
              </Button>
            </>
          )}
        </div>
      )}

      {/* Filters toolbar */}
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <select
          value={eventTypeFilter}
          onChange={(e) => handleEventTypeChange(e.target.value)}
          className={selectClasses}
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
          className={selectClasses}
        >
          <option value="">All Levels</option>
          {LOG_LEVEL_OPTIONS.filter(Boolean).map((ll) => (
            <option key={ll} value={ll}>
              {ll}
            </option>
          ))}
        </select>

        {classNames.length > 0 && (
          <select
            value={classNameFilter}
            onChange={(e) => handleClassNameChange(e.target.value)}
            className={selectClasses}
          >
            <option value="">All Classes</option>
            {classNames.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        )}

        <Input
          type="text"
          placeholder={
            hasMessages ? "Search messages..." : "Search event types..."
          }
          value={searchText}
          onChange={(e) => handleSearchChange(e.target.value)}
          className="h-8 w-[200px] text-xs"
        />

        <label className="inline-flex cursor-pointer select-none items-center gap-1.5 text-xs text-muted-foreground">
          <input
            type="checkbox"
            checked={wrap}
            onChange={(e) => setWrap(e.target.checked)}
            className="accent-primary"
          />
          Wrap lines
        </label>

        {/* Bookmark filter controls */}
        <button
          disabled={bookmarked.size === 0}
          onClick={() => setShowBookmarkedOnly((prev) => !prev)}
          className={cn(
            "inline-flex h-8 items-center gap-1.5 rounded-md border px-2 text-xs transition-colors",
            showBookmarkedOnly
              ? "border-amber-400 bg-amber-400/10 text-amber-400"
              : bookmarked.size > 0
                ? "border-input bg-background text-muted-foreground hover:text-foreground"
                : "cursor-not-allowed border-input bg-background text-muted-foreground/40"
          )}
        >
          <Bookmark
            className="h-3 w-3"
            fill={showBookmarkedOnly ? "currentColor" : "none"}
          />
          {bookmarked.size > 0 ? `Bookmarks (${bookmarked.size})` : "Bookmarks"}
        </button>

        {bookmarked.size > 0 && (
          <button
            onClick={clearBookmarks}
            className="inline-flex h-8 items-center rounded-md border border-input bg-background px-2 text-xs text-muted-foreground transition-colors hover:text-destructive"
            title="Clear all bookmarks"
          >
            Clear
          </button>
        )}

        {loading && (
          <span className="text-xs text-chart-1">Loading...</span>
        )}
        {error && (
          <span className="text-xs text-destructive">{error}</span>
        )}

        <span className="ml-auto text-xs text-muted-foreground">
          {showBookmarkedOnly
            ? `${displayedEvents.length} of ${bookmarked.size} bookmarks`
            : `${events.length} of ${total} events loaded`}
        </span>
      </div>

      {/* Virtualized event list */}
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="h-[600px] overflow-auto rounded border border-border bg-card font-mono text-xs"
        style={{ lineHeight: `${ROW_HEIGHT}px` }}
      >
        {/* Sticky header */}
        <div
          className="sticky top-0 z-10 flex whitespace-nowrap border-b-2 border-border bg-muted px-2 font-semibold text-muted-foreground"
          style={{ height: ROW_HEIGHT, minWidth: "fit-content" }}
        >
          <span
            className="shrink-0"
            style={{ width: 20, minWidth: 20, marginRight: "0.25rem" }}
          />
          <span
            className="shrink-0 text-right"
            style={{ width: 50, minWidth: 50, marginRight: "0.75rem" }}
          >
            Line
          </span>
          <span
            className="shrink-0"
            style={{ width: 90, minWidth: 90, marginRight: "0.5rem" }}
          >
            Time
          </span>
          <span
            className="shrink-0"
            style={{ width: 160, minWidth: 160, marginRight: "0.5rem" }}
          >
            Event Type
          </span>
          <span
            className="shrink-0"
            style={{ width: 60, minWidth: 60, marginRight: "0.5rem" }}
          >
            Level
          </span>
          {hasMessages && <span>Message</span>}
        </div>

        {/* Virtualized rows */}
        <div
          style={{
            height: virtualizer.getTotalSize(),
            width: "100%",
            position: "relative",
            minWidth: "fit-content",
          }}
        >
          {virtualItems.map((virtualRow) => {
            const evt = displayedEvents[virtualRow.index];
            if (!evt) return null;
            const isBookmarked = bookmarked.has(evt.line_index);
            return (
              <div
                key={virtualRow.key}
                ref={virtualizer.measureElement}
                data-index={virtualRow.index}
                className={cn(
                  "flex border-b border-border px-2",
                  wrap ? "whitespace-pre-wrap break-all" : "whitespace-nowrap",
                  isBookmarked && "border-l-2 border-l-amber-400 bg-amber-400/5"
                )}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  transform: `translateY(${virtualRow.start}px)`,
                  alignItems: "flex-start",
                }}
              >
                {renderRow(evt)}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

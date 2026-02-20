"use client";

import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import type { TimelineBucket } from "@loglens/api-client";

type Props = {
  timeline: TimelineBucket[];
};

function formatNanos(nanos: number): string {
  const totalMs = nanos / 1_000_000;
  if (totalMs < 1_000) return `${totalMs.toFixed(0)}ms`;
  const seconds = totalMs / 1_000;
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  const minutes = Math.floor(seconds / 60);
  const remainSec = seconds % 60;
  return `${minutes}m ${remainSec.toFixed(0)}s`;
}

export default function TimelineChart({ timeline }: Props) {
  if (timeline.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        No timeline data to display.
      </p>
    );
  }

  const data = timeline.map((b) => ({
    midpoint: (b.nanos_start + b.nanos_end) / 2,
    start: b.nanos_start,
    end: b.nanos_end,
    count: b.count,
  }));

  return (
    <div>
      <h4 className="mb-2 text-sm font-semibold text-foreground">
        Event Density Over Time
      </h4>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart
          data={data}
          margin={{ top: 10, right: 20, left: 10, bottom: 30 }}
        >
          <defs>
            <linearGradient id="timelineGrad" x1="0" y1="0" x2="0" y2="1">
              <stop
                offset="5%"
                stopColor="var(--chart-1)"
                stopOpacity={0.4}
              />
              <stop
                offset="95%"
                stopColor="var(--chart-1)"
                stopOpacity={0.05}
              />
            </linearGradient>
          </defs>
          <CartesianGrid
            strokeDasharray="3 3"
            stroke="var(--border)"
            opacity={0.5}
          />
          <XAxis
            dataKey="midpoint"
            type="number"
            tickFormatter={formatNanos}
            tick={{
              fontSize: 11,
              fill: "var(--muted-foreground)",
            }}
            label={{
              value: "Elapsed time",
              position: "insideBottom",
              offset: -10,
              fontSize: 12,
              fill: "var(--muted-foreground)",
            }}
          />
          <YAxis
            tick={{
              fontSize: 11,
              fill: "var(--muted-foreground)",
            }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "var(--popover)",
              borderColor: "var(--border)",
              color: "var(--popover-foreground)",
              borderRadius: "0.375rem",
              fontSize: "0.75rem",
            }}
            labelFormatter={(_, payload) => {
              const item = payload?.[0]?.payload;
              if (!item) return "";
              return `${formatNanos(item.start)} – ${formatNanos(item.end)}`;
            }}
            formatter={(value) => [
              `${Number(value).toLocaleString()} events`,
              "Count",
            ]}
          />
          <Area
            type="monotone"
            dataKey="count"
            stroke="var(--chart-1)"
            strokeWidth={2}
            fill="url(#timelineGrad)"
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

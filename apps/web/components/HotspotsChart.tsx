"use client";

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Cell,
} from "recharts";
import type { EventTypeBucket } from "@loglens/api-client";

type Props = {
  eventTypeCounts: EventTypeBucket[];
  totalEvents: number;
};

const CHART_COLORS = [
  "var(--chart-1)",
  "var(--chart-2)",
  "var(--chart-3)",
  "var(--chart-4)",
  "var(--chart-5)",
];

const MAX_BARS = 15;

export default function HotspotsChart({
  eventTypeCounts,
  totalEvents,
}: Props) {
  if (eventTypeCounts.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        No event type data to display.
      </p>
    );
  }

  const top = eventTypeCounts.slice(0, MAX_BARS);
  const otherCount = eventTypeCounts
    .slice(MAX_BARS)
    .reduce((sum, b) => sum + b.count, 0);
  const data =
    otherCount > 0
      ? [...top, { event_type: "OTHER", count: otherCount }]
      : top;

  const barHeight = 32;
  const chartHeight = Math.max(250, data.length * barHeight + 60);

  return (
    <div>
      <h4 className="mb-2 text-sm font-semibold text-foreground">
        Event Type Distribution
        <span className="ml-3 text-xs font-normal text-muted-foreground">
          {totalEvents.toLocaleString()} total events
        </span>
      </h4>
      <ResponsiveContainer width="100%" height={chartHeight}>
        <BarChart
          data={data}
          layout="vertical"
          margin={{ top: 5, right: 20, left: 10, bottom: 5 }}
        >
          <CartesianGrid
            strokeDasharray="3 3"
            stroke="var(--border)"
            opacity={0.5}
            horizontal={false}
          />
          <XAxis
            type="number"
            tick={{
              fontSize: 11,
              fill: "var(--muted-foreground)",
            }}
          />
          <YAxis
            type="category"
            dataKey="event_type"
            tick={{
              fontSize: 11,
              fill: "var(--foreground)",
            }}
            width={180}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "var(--popover)",
              borderColor: "var(--border)",
              color: "var(--popover-foreground)",
              borderRadius: "0.375rem",
              fontSize: "0.75rem",
            }}
            formatter={(value) => {
              const n = Number(value);
              const pct =
                totalEvents > 0
                  ? ((n / totalEvents) * 100).toFixed(1)
                  : "0";
              return [`${n.toLocaleString()} (${pct}%)`, "Events"];
            }}
          />
          <Bar
            dataKey="count"
            isAnimationActive={false}
            radius={[0, 4, 4, 0]}
          >
            {data.map((_, idx) => (
              <Cell
                key={idx}
                fill={CHART_COLORS[idx % CHART_COLORS.length]}
              />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}

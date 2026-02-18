"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import type { BenchmarkSnapshot } from "@loglens/api-client";

type ChartConfig = {
  title: string;
  dataKey: keyof BenchmarkSnapshot;
  limitKey?: keyof BenchmarkSnapshot;
  color: string;
  unit?: string;
};

const CHARTS: ChartConfig[] = [
  {
    title: "SOQL Queries vs Trigger Event",
    dataKey: "soql_queries",
    limitKey: "soql_queries_limit",
    color: "#6366f1",
    unit: " queries",
  },
  {
    title: "Query Rows vs Trigger Event",
    dataKey: "query_rows",
    limitKey: "query_rows_limit",
    color: "#f59e0b",
    unit: " rows",
  },
  {
    title: "Heap Size (%) vs Trigger Event",
    dataKey: "heap_size_pct",
    color: "#10b981",
    unit: "%",
  },
  {
    title: "DML Statements vs Trigger Event",
    dataKey: "dml_statements",
    limitKey: "dml_statements_limit",
    color: "#ef4444",
    unit: " ops",
  },
  {
    title: "CPU Time vs Trigger Event",
    dataKey: "cpu_time_ms",
    limitKey: "cpu_time_limit",
    color: "#8b5cf6",
    unit: " ms",
  },
];

type Props = {
  benchmarks: BenchmarkSnapshot[];
};

export default function GovernorLimitCharts({ benchmarks }: Props) {
  if (benchmarks.length === 0) {
    return <p style={{ color: "#888" }}>No benchmark data to display.</p>;
  }

  const data = benchmarks.map((b, i) => ({
    ...b,
    index: i,
    shortLabel:
      b.label.length > 30 ? b.label.slice(0, 27) + "..." : b.label,
  }));

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "2rem" }}>
      {CHARTS.map((chart) => (
        <div key={chart.dataKey}>
          <h4 style={{ margin: "0 0 0.5rem" }}>{chart.title}</h4>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart
              data={data}
              margin={{ top: 5, right: 20, left: 10, bottom: 80 }}
            >
              <CartesianGrid strokeDasharray="3 3" opacity={0.3} />
              <XAxis
                dataKey="shortLabel"
                angle={-90}
                textAnchor="end"
                interval={0}
                tick={{ fontSize: 10 }}
                height={100}
              />
              <YAxis tick={{ fontSize: 11 }} />
              <Tooltip
                labelFormatter={(_, payload) => {
                  const item = payload?.[0]?.payload;
                  return item?.label ?? "";
                }}
                formatter={(value) => [
                  `${value}${chart.unit ?? ""}`,
                  chart.title.split(" vs")[0],
                ]}
              />
              <Line
                type="monotone"
                dataKey={chart.dataKey}
                stroke={chart.color}
                strokeWidth={2}
                dot={{ r: 3 }}
                activeDot={{ r: 5 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      ))}
    </div>
  );
}

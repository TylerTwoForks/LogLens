"use client";

import { useState } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  ResponsiveContainer,
} from "recharts";
import type { BenchmarkSnapshot } from "@loglens/api-client";
import { Card, CardContent } from "@/components/ui/card";

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
    color: "var(--chart-1)",
    unit: " queries",
  },
  {
    title: "Query Rows vs Trigger Event",
    dataKey: "query_rows",
    limitKey: "query_rows_limit",
    color: "var(--chart-2)",
    unit: " rows",
  },
  {
    title: "Heap Size (%) vs Trigger Event",
    dataKey: "heap_size_pct",
    color: "var(--chart-4)",
    unit: "%",
  },
  {
    title: "DML Statements vs Trigger Event",
    dataKey: "dml_statements",
    limitKey: "dml_statements_limit",
    color: "var(--chart-3)",
    unit: " ops",
  },
  {
    title: "CPU Time vs Trigger Event",
    dataKey: "cpu_time_ms",
    limitKey: "cpu_time_limit",
    color: "var(--chart-5)",
    unit: " ms",
  },
];

const LABEL_TRUNCATE = 40;

type Props = {
  benchmarks: BenchmarkSnapshot[];
};

type ChartPoint = BenchmarkSnapshot & {
  seq: number;
  xIndex: number;
};

type HoveredDot = {
  point: ChartPoint;
  cx: number;
  cy: number;
};

function truncateLabel(label: string): string {
  return label.length > LABEL_TRUNCATE
    ? label.slice(0, LABEL_TRUNCATE - 3) + "..."
    : label;
}

function buildChartData(benchmarks: BenchmarkSnapshot[]) {
  const uniqueLabels: string[] = [];
  const labelIndexMap = new Map<string, number>();

  for (const b of benchmarks) {
    if (!labelIndexMap.has(b.label)) {
      labelIndexMap.set(b.label, uniqueLabels.length);
      uniqueLabels.push(b.label);
    }
  }

  const points: ChartPoint[] = benchmarks.map((b, seq) => ({
    ...b,
    seq,
    xIndex: labelIndexMap.get(b.label)!,
  }));

  const tickLabels = uniqueLabels.map(truncateLabel);

  return { points, uniqueLabels, tickLabels };
}

function AngledTick({
  x,
  y,
  payload,
  tickLabels,
}: {
  x: number;
  y: number;
  payload: { value: number };
  tickLabels: string[];
}) {
  const label = tickLabels[payload.value] ?? "";
  return (
    <g transform={`translate(${x},${y})`}>
      <text
        x={0}
        y={0}
        dy={14}
        textAnchor="end"
        fill="var(--foreground)"
        fontSize={12}
        fontWeight={500}
        transform="rotate(-45)"
      >
        {label}
      </text>
    </g>
  );
}

function SingleGovernorChart({
  chart,
  points,
  uniqueLabels,
  tickLabels,
  xTicks,
  bottomSpace,
}: {
  chart: ChartConfig;
  points: ChartPoint[];
  uniqueLabels: string[];
  tickLabels: string[];
  xTicks: number[];
  bottomSpace: number;
}) {
  const [hovered, setHovered] = useState<HoveredDot | null>(null);

  return (
    <div>
      <h4 className="mb-2 text-sm font-semibold text-foreground">
        {chart.title}
      </h4>
      <div className="relative">
        <ResponsiveContainer width="100%" height={500 + bottomSpace}>
          <LineChart
            data={points}
            margin={{
              top: 10,
              right: 20,
              left: 10,
              bottom: bottomSpace,
            }}
          >
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="var(--border)"
              opacity={0.5}
            />
            <XAxis
              dataKey="xIndex"
              type="number"
              domain={[0, uniqueLabels.length - 1]}
              ticks={xTicks}
              interval={0}
              tick={(props: Record<string, unknown>) => (
                <AngledTick
                  x={props.x as number}
                  y={props.y as number}
                  payload={props.payload as { value: number }}
                  tickLabels={tickLabels}
                />
              )}
              height={bottomSpace}
            />
            <YAxis
              tick={{
                fontSize: 11,
                fill: "var(--muted-foreground)",
              }}
            />
            <Line
              type="linear"
              dataKey={chart.dataKey}
              stroke={chart.color}
              strokeWidth={2}
              dot={(dotProps: Record<string, unknown>) => {
                const cx = dotProps.cx as number;
                const cy = dotProps.cy as number;
                const payload = dotProps.payload as ChartPoint;
                const index = dotProps.index as number;
                return (
                  <g key={`dot-${index}`}>
                    <circle
                      cx={cx}
                      cy={cy}
                      r={10}
                      fill="transparent"
                      stroke="none"
                      style={{ cursor: "pointer" }}
                      onMouseEnter={() =>
                        setHovered({ point: payload, cx, cy })
                      }
                      onMouseLeave={() => setHovered(null)}
                    />
                    <circle
                      cx={cx}
                      cy={cy}
                      r={3}
                      fill={chart.color}
                      stroke={chart.color}
                      strokeWidth={1}
                    />
                  </g>
                );
              }}
              activeDot={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
        {hovered && (
          <div
            className="pointer-events-none absolute z-10 max-w-xs rounded-md border border-border bg-popover px-3 py-2 text-xs text-popover-foreground shadow-md"
            style={{
              left: hovered.cx + 14,
              top: hovered.cy - 16,
            }}
          >
            <p className="mb-0.5 font-medium">{hovered.point.label}</p>
            <p>
              {chart.title.split(" vs")[0]}:{" "}
              {String(hovered.point[chart.dataKey])}
              {chart.unit ?? ""}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

export default function GovernorLimitCharts({ benchmarks }: Props) {
  if (benchmarks.length === 0) {
    return (
      <Card>
        <CardContent className="py-8 text-center">
          <p className="text-base text-muted-foreground">
            No benchmark data to display
          </p>
          <p className="mt-2 text-sm text-muted-foreground/60">
            Governor limit benchmarks are extracted from LIMIT_USAGE and
            LIMIT_USAGE_FOR_NS events in the log.
          </p>
        </CardContent>
      </Card>
    );
  }

  const { points, uniqueLabels, tickLabels } = buildChartData(benchmarks);

  const xTicks = uniqueLabels.map((_, i) => i);

  const maxLabelLen = Math.max(...tickLabels.map((l) => l.length));
  const bottomSpace = Math.min(250, Math.max(100, maxLabelLen * 4.5));

  return (
    <div className="flex flex-col gap-10">
      {CHARTS.map((chart) => (
        <SingleGovernorChart
          key={chart.dataKey}
          chart={chart}
          points={points}
          uniqueLabels={uniqueLabels}
          tickLabels={tickLabels}
          xTicks={xTicks}
          bottomSpace={bottomSpace}
        />
      ))}
    </div>
  );
}

"use client";

import { useState } from "react";
import type { BenchmarkSnapshot } from "@loglens/api-client";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";

type SortKey = keyof BenchmarkSnapshot;
type SortDir = "asc" | "desc";

const COLUMNS: {
  key: SortKey;
  label: string;
  format?: (v: number) => string;
}[] = [
  { key: "sequence", label: "#" },
  { key: "label", label: "Label" },
  { key: "query_rows", label: "Query Rows" },
  {
    key: "heap_size_pct",
    label: "Heap Size (%)",
    format: (v) => v.toFixed(1),
  },
  { key: "cpu_time_ms", label: "CPU Time (ms)" },
  { key: "dml_statements", label: "DML Statements" },
  { key: "soql_queries", label: "SOQL Queries" },
];

type Props = {
  benchmarks: BenchmarkSnapshot[];
};

export default function BenchmarkTable({ benchmarks }: Props) {
  const [sortKey, setSortKey] = useState<SortKey>("sequence");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [filter, setFilter] = useState("");

  if (benchmarks.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        No benchmark data to display.
      </p>
    );
  }

  const filtered = filter
    ? benchmarks.filter((b) =>
        b.label.toLowerCase().includes(filter.toLowerCase())
      )
    : benchmarks;

  const sorted = [...filtered].sort((a, b) => {
    const aVal = a[sortKey];
    const bVal = b[sortKey];
    if (typeof aVal === "string" && typeof bVal === "string") {
      return sortDir === "asc"
        ? aVal.localeCompare(bVal)
        : bVal.localeCompare(aVal);
    }
    const aNum = Number(aVal);
    const bNum = Number(bVal);
    return sortDir === "asc" ? aNum - bNum : bNum - aNum;
  });

  function handleSort(key: SortKey) {
    if (key === sortKey) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  }

  return (
    <div>
      <Input
        type="text"
        placeholder="Filter by label..."
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        className="mb-3 max-w-[260px]"
      />
      <div className="overflow-x-auto rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              {COLUMNS.map((col) => (
                <TableHead
                  key={col.key}
                  onClick={() => handleSort(col.key)}
                  className="cursor-pointer select-none whitespace-nowrap"
                >
                  {col.label}
                  {sortKey === col.key
                    ? sortDir === "asc"
                      ? " ▲"
                      : " ▼"
                    : ""}
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {sorted.map((row) => (
              <TableRow key={`${row.sequence}-${row.label}`}>
                {COLUMNS.map((col) => {
                  const raw = row[col.key];
                  const display =
                    col.format && typeof raw === "number"
                      ? col.format(raw)
                      : String(raw);
                  return (
                    <TableCell
                      key={col.key}
                      className={cn(
                        col.key === "label"
                          ? "whitespace-normal"
                          : "whitespace-nowrap"
                      )}
                    >
                      {display}
                    </TableCell>
                  );
                })}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
      <p className="mt-2 text-xs text-muted-foreground">
        {sorted.length} of {benchmarks.length} rows
      </p>
    </div>
  );
}

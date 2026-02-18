"use client";

import { useState } from "react";
import type { BenchmarkSnapshot } from "@loglens/api-client";

type SortKey = keyof BenchmarkSnapshot;
type SortDir = "asc" | "desc";

const COLUMNS: { key: SortKey; label: string; format?: (v: number) => string }[] = [
  { key: "sequence", label: "#" },
  { key: "label", label: "Label" },
  { key: "query_rows", label: "Query Rows" },
  { key: "heap_size_pct", label: "Heap Size (%)", format: (v) => v.toFixed(1) },
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
    return <p style={{ color: "#888" }}>No benchmark data to display.</p>;
  }

  const filtered = filter
    ? benchmarks.filter((b) =>
        b.label.toLowerCase().includes(filter.toLowerCase()),
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
      <input
        type="text"
        placeholder="Filter by label..."
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        style={{
          marginBottom: "0.75rem",
          padding: "0.4rem 0.6rem",
          fontSize: "0.875rem",
          border: "1px solid #555",
          borderRadius: "4px",
          background: "inherit",
          color: "inherit",
          width: "260px",
        }}
      />
      <div style={{ overflowX: "auto" }}>
        <table
          style={{
            width: "100%",
            borderCollapse: "collapse",
            fontSize: "0.8125rem",
          }}
        >
          <thead>
            <tr>
              {COLUMNS.map((col) => (
                <th
                  key={col.key}
                  onClick={() => handleSort(col.key)}
                  style={{
                    cursor: "pointer",
                    textAlign: "left",
                    padding: "0.5rem 0.75rem",
                    borderBottom: "2px solid #444",
                    whiteSpace: "nowrap",
                    userSelect: "none",
                  }}
                >
                  {col.label}
                  {sortKey === col.key ? (sortDir === "asc" ? " \u25B2" : " \u25BC") : ""}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {sorted.map((row) => (
              <tr key={`${row.sequence}-${row.label}`}>
                {COLUMNS.map((col) => {
                  const raw = row[col.key];
                  const display =
                    col.format && typeof raw === "number"
                      ? col.format(raw)
                      : String(raw);
                  return (
                    <td
                      key={col.key}
                      style={{
                        padding: "0.4rem 0.75rem",
                        borderBottom: "1px solid #333",
                        whiteSpace: col.key === "label" ? "normal" : "nowrap",
                      }}
                    >
                      {display}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p style={{ fontSize: "0.75rem", color: "#888", margin: "0.5rem 0 0" }}>
        {sorted.length} of {benchmarks.length} rows
      </p>
    </div>
  );
}

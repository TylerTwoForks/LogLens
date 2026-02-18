use parser_core::{parse_log, ParsedEvent};
use parser_sfdc_benchmarks::{extract_benchmarks, BenchmarkSnapshot};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParseMetrics {
    pub total_lines: usize,
    pub parsed_lines: usize,
    pub benchmark_count: usize,
    pub peak_query_rows: i64,
    pub peak_heap_size_pct: f64,
    pub peak_cpu_time_ms: i64,
    pub peak_dml_statements: i64,
    pub peak_soql_queries: i64,
}

/// Produce summary statistics from raw log content.
///
/// Parses the log, extracts benchmarks, and derives peak governor-limit values.
pub fn summarize(content: &str) -> ParseMetrics {
    let total_lines = content.lines().count();
    let events = parse_log(content);
    let parsed_lines = events.len();
    let benchmarks = extract_benchmarks(&events);

    build_metrics(total_lines, parsed_lines, &benchmarks)
}

/// Produce summary statistics from already-parsed events.
pub fn summarize_from_events(total_lines: usize, events: &[ParsedEvent]) -> ParseMetrics {
    let parsed_lines = events.len();
    let benchmarks = extract_benchmarks(events);

    build_metrics(total_lines, parsed_lines, &benchmarks)
}

fn build_metrics(
    total_lines: usize,
    parsed_lines: usize,
    benchmarks: &[BenchmarkSnapshot],
) -> ParseMetrics {
    let benchmark_count = benchmarks.len();

    let peak_query_rows = benchmarks.iter().map(|b| b.query_rows).max().unwrap_or(0);
    let peak_heap_size_pct = benchmarks
        .iter()
        .map(|b| b.heap_size_pct)
        .fold(0.0_f64, f64::max);
    let peak_cpu_time_ms = benchmarks.iter().map(|b| b.cpu_time_ms).max().unwrap_or(0);
    let peak_dml_statements = benchmarks
        .iter()
        .map(|b| b.dml_statements)
        .max()
        .unwrap_or(0);
    let peak_soql_queries = benchmarks
        .iter()
        .map(|b| b.soql_queries)
        .max()
        .unwrap_or(0);

    ParseMetrics {
        total_lines,
        parsed_lines,
        benchmark_count,
        peak_query_rows,
        peak_heap_size_pct,
        peak_cpu_time_ms,
        peak_dml_statements,
        peak_soql_queries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_empty_log() {
        let metrics = summarize("");
        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.parsed_lines, 0);
        assert_eq!(metrics.benchmark_count, 0);
        assert_eq!(metrics.peak_query_rows, 0);
    }

    #[test]
    fn summarizes_log_without_benchmarks() {
        let content = "\
14:51:26.0 (100)|EXECUTION_STARTED
14:51:26.0 (200)|USER_DEBUG|[1]|DEBUG|hello world
14:51:26.0 (300)|EXECUTION_FINISHED";

        let metrics = summarize(content);
        assert_eq!(metrics.total_lines, 3);
        assert_eq!(metrics.parsed_lines, 3);
        assert_eq!(metrics.benchmark_count, 0);
    }

    #[test]
    fn summarizes_log_with_benchmarks() {
        let content = "\
14:51:26.0 (100)|USER_DEBUG|[98]|DEBUG|Trigger Event: Handler - Before Query Rows:  50/50,000 {+50}
14:51:26.0 (101)|USER_DEBUG|[129]|DEBUG|Trigger Event: Handler - Before Heap Size:  2.5% of 6000000 bytes {+2.5}
14:51:26.0 (102)|USER_DEBUG|[112]|DEBUG|Trigger Event: Handler - Before CPU Time:  300 out of 60000 {+300}
14:51:26.0 (103)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - Before DML Statement Limit: 5 / 100 DML Operations
14:51:26.0 (104)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - Before SOQL Limit: 12 / 100 SOQL Queries
14:51:26.0 (200)|USER_DEBUG|[98]|DEBUG|Trigger Event: Handler - After Query Rows:  120/50,000 {+70}
14:51:26.0 (201)|USER_DEBUG|[129]|DEBUG|Trigger Event: Handler - After Heap Size:  5.0% of 6000000 bytes {+2.5}
14:51:26.0 (202)|USER_DEBUG|[112]|DEBUG|Trigger Event: Handler - After CPU Time:  800 out of 60000 {+500}
14:51:26.0 (203)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - After DML Statement Limit: 10 / 100 DML Operations
14:51:26.0 (204)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - After SOQL Limit: 25 / 100 SOQL Queries";

        let metrics = summarize(content);
        assert_eq!(metrics.total_lines, 10);
        assert_eq!(metrics.parsed_lines, 10);
        assert_eq!(metrics.benchmark_count, 2);
        assert_eq!(metrics.peak_query_rows, 120);
        assert!((metrics.peak_heap_size_pct - 5.0).abs() < 0.001);
        assert_eq!(metrics.peak_cpu_time_ms, 800);
        assert_eq!(metrics.peak_dml_statements, 10);
        assert_eq!(metrics.peak_soql_queries, 25);
    }
}

use parser_core::{ParsedEvent, SfdcEventType};
use serde::Serialize;

/// A single governor-limit snapshot captured from a `Debug.benchmark('Limit:...')`
/// checkpoint in a Salesforce debug log.
///
/// Each snapshot groups 5 metrics (Query Rows, Heap Size, CPU Time, DML Statements,
/// SOQL Queries) that were emitted together for a single labeled checkpoint.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkSnapshot {
    pub sequence: usize,
    pub label: String,
    pub query_rows: i64,
    pub query_rows_limit: i64,
    pub query_rows_delta: i64,
    pub heap_size_pct: f64,
    pub heap_size_bytes_limit: i64,
    pub heap_size_delta: f64,
    pub cpu_time_ms: i64,
    pub cpu_time_limit: i64,
    pub cpu_time_delta: i64,
    pub dml_statements: i64,
    pub dml_statements_limit: i64,
    pub soql_queries: i64,
    pub soql_queries_limit: i64,
}

/// Intermediate accumulator while collecting the 5 metrics for a single checkpoint.
#[derive(Debug, Default)]
struct SnapshotBuilder {
    label: String,
    query_rows: Option<(i64, i64, i64)>,       // (current, limit, delta)
    heap_size: Option<(f64, i64, f64)>,         // (pct, bytes_limit, delta)
    cpu_time: Option<(i64, i64, i64)>,          // (current, limit, delta)
    dml_statements: Option<(i64, i64)>,         // (current, limit)
    soql_queries: Option<(i64, i64)>,           // (current, limit)
}

impl SnapshotBuilder {
    fn is_complete(&self) -> bool {
        self.query_rows.is_some()
            && self.heap_size.is_some()
            && self.cpu_time.is_some()
            && self.dml_statements.is_some()
            && self.soql_queries.is_some()
    }

    fn build(self, sequence: usize) -> BenchmarkSnapshot {
        let (qr, qr_lim, qr_delta) = self.query_rows.unwrap_or_default();
        let (hs_pct, hs_lim, hs_delta) = self.heap_size.unwrap_or_default();
        let (cpu, cpu_lim, cpu_delta) = self.cpu_time.unwrap_or_default();
        let (dml, dml_lim) = self.dml_statements.unwrap_or_default();
        let (soql, soql_lim) = self.soql_queries.unwrap_or_default();

        BenchmarkSnapshot {
            sequence,
            label: self.label,
            query_rows: qr,
            query_rows_limit: qr_lim,
            query_rows_delta: qr_delta,
            heap_size_pct: hs_pct,
            heap_size_bytes_limit: hs_lim,
            heap_size_delta: hs_delta,
            cpu_time_ms: cpu,
            cpu_time_limit: cpu_lim,
            cpu_time_delta: cpu_delta,
            dml_statements: dml,
            dml_statements_limit: dml_lim,
            soql_queries: soql,
            soql_queries_limit: soql_lim,
        }
    }
}

/// Extract benchmark snapshots from a sequence of already-parsed log events.
///
/// This mirrors the logic of the Python notebook: scan for `USER_DEBUG` lines
/// containing `Trigger Event:`, parse out the 5 governor-limit metrics, group
/// them by label in the order they appear, and return one `BenchmarkSnapshot`
/// per complete group.
pub fn extract_benchmarks(events: &[ParsedEvent]) -> Vec<BenchmarkSnapshot> {
    let mut snapshots = Vec::new();
    let mut builder: Option<SnapshotBuilder> = None;
    let mut seq: usize = 0;

    for event in events {
        if event.event_type != SfdcEventType::UserDebug {
            if let Some(b) = builder.take() {
                snapshots.push(b.build(seq));
                seq += 1;
            }
            continue;
        }

        let msg = &event.message;
        let Some(te_pos) = msg.find("Trigger Event:") else {
            if let Some(b) = builder.take() {
                snapshots.push(b.build(seq));
                seq += 1;
            }
            continue;
        };

        let after_te = &msg[te_pos + "Trigger Event:".len()..].trim_start();

        if let Some(parsed) = try_parse_query_rows(after_te) {
            let b = flush_if_complete_and_new_label(&mut builder, &parsed.0, &mut snapshots, &mut seq);
            let b = b.unwrap_or_else(|| new_builder(&parsed.0));
            let mut b = b;
            b.query_rows = Some((parsed.1, parsed.2, parsed.3));
            builder = Some(b);
        } else if let Some(parsed) = try_parse_heap_size(after_te) {
            let b = flush_if_complete_and_new_label(&mut builder, &parsed.0, &mut snapshots, &mut seq);
            let b = b.unwrap_or_else(|| new_builder(&parsed.0));
            let mut b = b;
            b.heap_size = Some((parsed.1, parsed.2, parsed.3));
            builder = Some(b);
        } else if let Some(parsed) = try_parse_cpu_time(after_te) {
            let b = flush_if_complete_and_new_label(&mut builder, &parsed.0, &mut snapshots, &mut seq);
            let b = b.unwrap_or_else(|| new_builder(&parsed.0));
            let mut b = b;
            b.cpu_time = Some((parsed.1, parsed.2, parsed.3));
            builder = Some(b);
        } else if let Some(parsed) = try_parse_dml_statements(after_te) {
            let b = flush_if_complete_and_new_label(&mut builder, &parsed.0, &mut snapshots, &mut seq);
            let b = b.unwrap_or_else(|| new_builder(&parsed.0));
            let mut b = b;
            b.dml_statements = Some((parsed.1, parsed.2));
            builder = Some(b);
        } else if let Some(parsed) = try_parse_soql_queries(after_te) {
            let b = flush_if_complete_and_new_label(&mut builder, &parsed.0, &mut snapshots, &mut seq);
            let b = b.unwrap_or_else(|| new_builder(&parsed.0));
            let mut b = b;
            b.soql_queries = Some((parsed.1, parsed.2));
            builder = Some(b);
        } else if let Some(b) = builder.take() {
            snapshots.push(b.build(seq));
            seq += 1;
        }
    }

    if let Some(b) = builder.take() {
        snapshots.push(b.build(seq));
    }

    snapshots
}

/// Convenience wrapper: parse raw log text and extract benchmarks in one call.
pub fn extract_benchmarks_from_log(content: &str) -> Vec<BenchmarkSnapshot> {
    let events = parser_core::parse_log(content);
    extract_benchmarks(&events)
}

fn new_builder(label: &str) -> SnapshotBuilder {
    SnapshotBuilder {
        label: label.to_owned(),
        ..Default::default()
    }
}

/// If the current builder is complete (has all 5 metrics), flush it and return
/// `None` so the caller creates a fresh builder. Otherwise return the existing
/// builder if the label matches, or flush + return `None` if the label changed.
fn flush_if_complete_and_new_label(
    builder: &mut Option<SnapshotBuilder>,
    label: &str,
    snapshots: &mut Vec<BenchmarkSnapshot>,
    seq: &mut usize,
) -> Option<SnapshotBuilder> {
    let Some(b) = builder.take() else {
        return None;
    };

    if b.is_complete() || b.label != label {
        snapshots.push(b.build(*seq));
        *seq += 1;
        None
    } else {
        Some(b)
    }
}

// ---------------------------------------------------------------------------
// Metric parsers
//
// Each returns Option<(label, ...metric fields)>.
//
// Format examples from the Salesforce logs:
//   "DayHandler - BeforeUpdate Query Rows:  0/50,000 {0}"
//   "DayHandler - BeforeUpdate Heap Size:  0.1% of 6000000 bytes {+0.1}"
//   "DayHandler - BeforeUpdate CPU Time:  24 out of 60000 {+24}"
//   "DayHandler - BeforeUpdate DML Statement Limit: 1 / 100 DML Operations"
//   "DayHandler - BeforeUpdate SOQL Limit: 0 / 100 SOQL Queries"
// ---------------------------------------------------------------------------

fn try_parse_query_rows(text: &str) -> Option<(String, i64, i64, i64)> {
    let marker = "Query Rows:";
    let idx = text.find(marker)?;
    let label = text[..idx].trim().to_owned();
    let rest = text[idx + marker.len()..].trim();

    // "0/50,000 {0}" or "53/50,000 {+53}"
    let slash = rest.find('/')?;
    let current = parse_int_with_commas(rest[..slash].trim())?;

    let after_slash = &rest[slash + 1..];
    let (limit_str, delta_part) = split_at_brace(after_slash);
    let limit = parse_int_with_commas(limit_str.trim())?;
    let delta = parse_int_delta(delta_part.trim());

    Some((label, current, limit, delta))
}

fn try_parse_heap_size(text: &str) -> Option<(String, f64, i64, f64)> {
    let marker = "Heap Size:";
    let idx = text.find(marker)?;
    let label = text[..idx].trim().to_owned();
    let rest = text[idx + marker.len()..].trim();

    // "0.1% of 6000000 bytes {+0.1}"
    let pct_idx = rest.find('%')?;
    let pct: f64 = rest[..pct_idx].trim().parse().ok()?;

    let of_idx = rest.find("of")?;
    let after_of = &rest[of_idx + 2..];
    let bytes_idx = after_of.find("bytes")?;
    let bytes_limit: i64 = after_of[..bytes_idx].trim().parse().ok()?;

    let (_, delta_part) = split_at_brace(after_of);
    let delta = parse_float_delta(delta_part.trim());

    Some((label, pct, bytes_limit, delta))
}

fn try_parse_cpu_time(text: &str) -> Option<(String, i64, i64, i64)> {
    let marker = "CPU Time:";
    let idx = text.find(marker)?;
    let label = text[..idx].trim().to_owned();
    let rest = text[idx + marker.len()..].trim();

    // "24 out of 60000 {+24}"
    let out_idx = rest.find("out of")?;
    let current: i64 = rest[..out_idx].trim().parse().ok()?;

    let after_out = &rest[out_idx + "out of".len()..];
    let (limit_str, delta_part) = split_at_brace(after_out);
    let limit: i64 = limit_str.trim().parse().ok()?;
    let delta = parse_int_delta(delta_part.trim());

    Some((label, current, limit, delta))
}

fn try_parse_dml_statements(text: &str) -> Option<(String, i64, i64)> {
    let marker = "DML Statement Limit:";
    let idx = text.find(marker)?;
    let label = text[..idx].trim().to_owned();
    let rest = text[idx + marker.len()..].trim();

    // "1 / 100 DML Operations"
    let slash = rest.find('/')?;
    let current: i64 = rest[..slash].trim().parse().ok()?;

    let after_slash = &rest[slash + 1..];
    let dml_idx = after_slash.find("DML").unwrap_or(after_slash.len());
    let limit: i64 = after_slash[..dml_idx].trim().parse().ok()?;

    Some((label, current, limit))
}

fn try_parse_soql_queries(text: &str) -> Option<(String, i64, i64)> {
    let marker = "SOQL Limit:";
    let idx = text.find(marker)?;
    let label = text[..idx].trim().to_owned();
    let rest = text[idx + marker.len()..].trim();

    // "0 / 100 SOQL Queries"
    let slash = rest.find('/')?;
    let current: i64 = rest[..slash].trim().parse().ok()?;

    let after_slash = &rest[slash + 1..];
    let soql_idx = after_slash.find("SOQL").unwrap_or(after_slash.len());
    let limit: i64 = after_slash[..soql_idx].trim().parse().ok()?;

    Some((label, current, limit))
}

/// Split text at the first `{` to separate the value portion from the delta.
fn split_at_brace(text: &str) -> (&str, &str) {
    if let Some(brace) = text.find('{') {
        (&text[..brace], &text[brace..])
    } else {
        (text, "")
    }
}

fn parse_int_with_commas(s: &str) -> Option<i64> {
    let cleaned: String = s.chars().filter(|c| *c != ',').collect();
    cleaned.parse().ok()
}

fn parse_int_delta(s: &str) -> i64 {
    // "{+53}" or "{0}" or "{-10}"
    let inner = s.trim_start_matches('{').trim_end_matches('}').trim();
    let cleaned = inner.trim_start_matches('+');
    cleaned.parse().unwrap_or(0)
}

fn parse_float_delta(s: &str) -> f64 {
    let inner = s.trim_start_matches('{').trim_end_matches('}').trim();
    let cleaned = inner.trim_start_matches('+');
    cleaned.parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LOG: &str = "\
14:51:26.0 (64969664)|USER_DEBUG|[98]|DEBUG|Trigger Event: DayHandler - BeforeUpdate Query Rows:  0/50,000 {0}
14:51:26.0 (65401713)|USER_DEBUG|[129]|DEBUG|Trigger Event: DayHandler - BeforeUpdate Heap Size:  0.1% of 6000000 bytes {+0.1}
14:51:26.0 (65490690)|USER_DEBUG|[112]|DEBUG|Trigger Event: DayHandler - BeforeUpdate CPU Time:  24 out of 60000 {+24}
14:51:26.0 (65701492)|USER_DEBUG|[40]|DEBUG|Trigger Event: DayHandler - BeforeUpdate DML Statement Limit: 1 / 100 DML Operations
14:51:26.0 (65803771)|USER_DEBUG|[40]|DEBUG|Trigger Event: DayHandler - BeforeUpdate SOQL Limit: 0 / 100 SOQL Queries
14:51:26.0 (318754101)|USER_DEBUG|[98]|DEBUG|Trigger Event: DayHandler - Before->AndFinally Query Rows:  53/50,000 {+53}
14:51:26.0 (320035588)|USER_DEBUG|[129]|DEBUG|Trigger Event: DayHandler - Before->AndFinally Heap Size:  1.5% of 6000000 bytes {+1.4}
14:51:26.0 (320093469)|USER_DEBUG|[112]|DEBUG|Trigger Event: DayHandler - Before->AndFinally CPU Time:  139 out of 60000 {+115}
14:51:26.0 (320187809)|USER_DEBUG|[40]|DEBUG|Trigger Event: DayHandler - Before->AndFinally DML Statement Limit: 1 / 100 DML Operations
14:51:26.0 (320238799)|USER_DEBUG|[40]|DEBUG|Trigger Event: DayHandler - Before->AndFinally SOQL Limit: 5 / 100 SOQL Queries";

    #[test]
    fn extracts_two_complete_snapshots() {
        let snapshots = extract_benchmarks_from_log(SAMPLE_LOG);
        assert_eq!(snapshots.len(), 2);

        let first = &snapshots[0];
        assert_eq!(first.sequence, 0);
        assert_eq!(first.label, "DayHandler - BeforeUpdate");
        assert_eq!(first.query_rows, 0);
        assert_eq!(first.query_rows_limit, 50000);
        assert_eq!(first.query_rows_delta, 0);
        assert!((first.heap_size_pct - 0.1).abs() < 0.001);
        assert_eq!(first.heap_size_bytes_limit, 6000000);
        assert!((first.heap_size_delta - 0.1).abs() < 0.001);
        assert_eq!(first.cpu_time_ms, 24);
        assert_eq!(first.cpu_time_limit, 60000);
        assert_eq!(first.cpu_time_delta, 24);
        assert_eq!(first.dml_statements, 1);
        assert_eq!(first.dml_statements_limit, 100);
        assert_eq!(first.soql_queries, 0);
        assert_eq!(first.soql_queries_limit, 100);

        let second = &snapshots[1];
        assert_eq!(second.sequence, 1);
        assert_eq!(second.label, "DayHandler - Before->AndFinally");
        assert_eq!(second.query_rows, 53);
        assert_eq!(second.query_rows_delta, 53);
        assert!((second.heap_size_pct - 1.5).abs() < 0.001);
        assert_eq!(second.cpu_time_ms, 139);
        assert_eq!(second.cpu_time_delta, 115);
        assert_eq!(second.dml_statements, 1);
        assert_eq!(second.soql_queries, 5);
    }

    #[test]
    fn handles_negative_heap_delta() {
        let log = "\
14:51:26.0 (100)|USER_DEBUG|[98]|DEBUG|Trigger Event: Handler - After Heap Size:  1.2% of 6000000 bytes {-0.3}";
        let events = parser_core::parse_log(log);
        let snapshots = extract_benchmarks(&events);

        assert_eq!(snapshots.len(), 1);
        assert!((snapshots[0].heap_size_delta - (-0.3)).abs() < 0.001);
    }

    #[test]
    fn handles_duplicate_labels_for_reentrant_triggers() {
        let log = "\
14:51:26.0 (100)|USER_DEBUG|[98]|DEBUG|Trigger Event: Handler - Before Query Rows:  0/50,000 {0}
14:51:26.0 (101)|USER_DEBUG|[129]|DEBUG|Trigger Event: Handler - Before Heap Size:  0.1% of 6000000 bytes {+0.1}
14:51:26.0 (102)|USER_DEBUG|[112]|DEBUG|Trigger Event: Handler - Before CPU Time:  10 out of 60000 {+10}
14:51:26.0 (103)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - Before DML Statement Limit: 1 / 100 DML Operations
14:51:26.0 (104)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - Before SOQL Limit: 0 / 100 SOQL Queries
14:51:26.0 (200)|USER_DEBUG|[98]|DEBUG|Trigger Event: Handler - Before Query Rows:  50/50,000 {+50}
14:51:26.0 (201)|USER_DEBUG|[129]|DEBUG|Trigger Event: Handler - Before Heap Size:  2.0% of 6000000 bytes {+1.9}
14:51:26.0 (202)|USER_DEBUG|[112]|DEBUG|Trigger Event: Handler - Before CPU Time:  100 out of 60000 {+90}
14:51:26.0 (203)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - Before DML Statement Limit: 3 / 100 DML Operations
14:51:26.0 (204)|USER_DEBUG|[40]|DEBUG|Trigger Event: Handler - Before SOQL Limit: 5 / 100 SOQL Queries";

        let snapshots = extract_benchmarks_from_log(log);
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].label, "Handler - Before");
        assert_eq!(snapshots[1].label, "Handler - Before");
        assert_eq!(snapshots[0].query_rows, 0);
        assert_eq!(snapshots[1].query_rows, 50);
    }

    #[test]
    fn skips_non_benchmark_user_debug() {
        let log = "\
14:51:26.0 (100)|USER_DEBUG|[5]|DEBUG|Some random debug message
14:51:26.0 (200)|USER_DEBUG|[98]|DEBUG|Trigger Event: X Query Rows:  0/50,000 {0}
14:51:26.0 (201)|USER_DEBUG|[129]|DEBUG|Trigger Event: X Heap Size:  0.5% of 6000000 bytes {+0.5}
14:51:26.0 (202)|USER_DEBUG|[112]|DEBUG|Trigger Event: X CPU Time:  5 out of 60000 {+5}
14:51:26.0 (203)|USER_DEBUG|[40]|DEBUG|Trigger Event: X DML Statement Limit: 0 / 100 DML Operations
14:51:26.0 (204)|USER_DEBUG|[40]|DEBUG|Trigger Event: X SOQL Limit: 0 / 100 SOQL Queries";

        let snapshots = extract_benchmarks_from_log(log);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].label, "X");
    }

    #[test]
    fn empty_input_yields_no_snapshots() {
        let snapshots = extract_benchmarks_from_log("");
        assert!(snapshots.is_empty());
    }

    #[test]
    fn parse_helpers_work() {
        assert_eq!(parse_int_with_commas("50,000"), Some(50000));
        assert_eq!(parse_int_with_commas("0"), Some(0));
        assert_eq!(parse_int_delta("{+53}"), 53);
        assert_eq!(parse_int_delta("{0}"), 0);
        assert_eq!(parse_int_delta("{-10}"), -10);
        assert!((parse_float_delta("{+1.4}") - 1.4).abs() < 0.001);
        assert!((parse_float_delta("{-0.3}") - (-0.3)).abs() < 0.001);
    }
}

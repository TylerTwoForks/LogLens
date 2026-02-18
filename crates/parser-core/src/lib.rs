use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfdcEventType {
    UserDebug,
    ExecutionStarted,
    ExecutionFinished,
    CodeUnitStarted,
    CodeUnitFinished,
    MethodEntry,
    MethodExit,
    SoqlExecuteBegin,
    SoqlExecuteEnd,
    DmlBegin,
    DmlEnd,
    LimitUsage,
    LimitUsageForNs,
    SystemDebug,
    Other(String),
}

impl SfdcEventType {
    pub fn from_str_tag(s: &str) -> Self {
        match s {
            "USER_DEBUG" => Self::UserDebug,
            "EXECUTION_STARTED" => Self::ExecutionStarted,
            "EXECUTION_FINISHED" => Self::ExecutionFinished,
            "CODE_UNIT_STARTED" => Self::CodeUnitStarted,
            "CODE_UNIT_FINISHED" => Self::CodeUnitFinished,
            "METHOD_ENTRY" => Self::MethodEntry,
            "METHOD_EXIT" => Self::MethodExit,
            "SOQL_EXECUTE_BEGIN" => Self::SoqlExecuteBegin,
            "SOQL_EXECUTE_END" => Self::SoqlExecuteEnd,
            "DML_BEGIN" => Self::DmlBegin,
            "DML_END" => Self::DmlEnd,
            "LIMIT_USAGE" => Self::LimitUsage,
            "LIMIT_USAGE_FOR_NS" => Self::LimitUsageForNs,
            "SYSTEM_DEBUG" => Self::SystemDebug,
            other => Self::Other(other.to_owned()),
        }
    }
}

impl fmt::Display for SfdcEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::UserDebug => "USER_DEBUG",
            Self::ExecutionStarted => "EXECUTION_STARTED",
            Self::ExecutionFinished => "EXECUTION_FINISHED",
            Self::CodeUnitStarted => "CODE_UNIT_STARTED",
            Self::CodeUnitFinished => "CODE_UNIT_FINISHED",
            Self::MethodEntry => "METHOD_ENTRY",
            Self::MethodExit => "METHOD_EXIT",
            Self::SoqlExecuteBegin => "SOQL_EXECUTE_BEGIN",
            Self::SoqlExecuteEnd => "SOQL_EXECUTE_END",
            Self::DmlBegin => "DML_BEGIN",
            Self::DmlEnd => "DML_END",
            Self::LimitUsage => "LIMIT_USAGE",
            Self::LimitUsageForNs => "LIMIT_USAGE_FOR_NS",
            Self::SystemDebug => "SYSTEM_DEBUG",
            Self::Other(tag) => tag.as_str(),
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Fine,
    Finer,
    Finest,
    Other(String),
}

impl LogLevel {
    pub fn from_str_tag(s: &str) -> Self {
        match s {
            "ERROR" => Self::Error,
            "WARN" => Self::Warn,
            "INFO" => Self::Info,
            "DEBUG" => Self::Debug,
            "FINE" => Self::Fine,
            "FINER" => Self::Finer,
            "FINEST" => Self::Finest,
            other => Self::Other(other.to_owned()),
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Fine => "FINE",
            Self::Finer => "FINER",
            Self::Finest => "FINEST",
            Self::Other(tag) => tag.as_str(),
        };
        write!(f, "{label}")
    }
}

/// A single parsed line from a Salesforce debug log.
///
/// Salesforce log lines follow the format:
///   `TIMESTAMP (NANOS)|EVENT_TYPE|[LINE_NO]|LOG_LEVEL|MESSAGE`
///
/// Not all lines have every segment (e.g. `EXECUTION_STARTED` has no
/// line number or log level), so optional fields may be `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedEvent {
    pub timestamp: String,
    pub nanos: Option<u64>,
    pub event_type: SfdcEventType,
    pub line_number: Option<u32>,
    pub level: Option<LogLevel>,
    pub message: String,
    pub raw_line: String,
}

/// Attempt to parse a single Salesforce debug log line.
///
/// Returns `None` for lines that don't match the expected pipe-delimited
/// structure (header lines, blank lines, etc.).
pub fn parse_line(line: &str) -> Option<ParsedEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parts: Vec<&str> = trimmed.splitn(5, '|').collect();
    if parts.len() < 2 {
        return None;
    }

    let timestamp_raw = parts[0].trim();
    let (timestamp, nanos) = parse_timestamp(timestamp_raw)?;
    let event_type = SfdcEventType::from_str_tag(parts[1].trim());

    let (line_number, level, message) = match parts.len() {
        // timestamp|event_type
        2 => (None, None, String::new()),
        // timestamp|event_type|rest
        3 => {
            let rest = parts[2].trim();
            if rest.starts_with('[') {
                (parse_line_number(rest), None, String::new())
            } else {
                (None, None, rest.to_owned())
            }
        }
        // timestamp|event_type|line_no|level_or_msg
        4 => {
            let seg2 = parts[2].trim();
            let seg3 = parts[3].trim();
            if seg2.starts_with('[') {
                let ln = parse_line_number(seg2);
                if looks_like_level(seg3) {
                    (ln, Some(LogLevel::from_str_tag(seg3)), String::new())
                } else {
                    (ln, None, seg3.to_owned())
                }
            } else if looks_like_level(seg2) {
                (None, Some(LogLevel::from_str_tag(seg2)), seg3.to_owned())
            } else {
                (None, None, format!("{seg2}|{seg3}"))
            }
        }
        // timestamp|event_type|line_no|level|message
        _ => {
            let seg2 = parts[2].trim();
            let seg3 = parts[3].trim();
            let seg4 = parts[4]; // preserve whitespace in message body
            if seg2.starts_with('[') {
                let ln = parse_line_number(seg2);
                (ln, Some(LogLevel::from_str_tag(seg3)), seg4.to_owned())
            } else if looks_like_level(seg2) {
                let message = format!("{seg3}|{seg4}");
                (None, Some(LogLevel::from_str_tag(seg2)), message)
            } else {
                let message = format!("{seg2}|{seg3}|{seg4}");
                (None, None, message)
            }
        }
    };

    Some(ParsedEvent {
        timestamp,
        nanos,
        event_type,
        line_number,
        level,
        message,
        raw_line: trimmed.to_owned(),
    })
}

/// Parse all lines in a log body, returning events for every parseable line.
pub fn parse_log(content: &str) -> Vec<ParsedEvent> {
    content.lines().filter_map(parse_line).collect()
}

fn parse_timestamp(raw: &str) -> Option<(String, Option<u64>)> {
    // Format: "14:51:26.0 (64969664)" or just "14:51:26.0"
    if let Some(paren_start) = raw.find('(') {
        let ts = raw[..paren_start].trim().to_owned();
        let nanos_str = raw[paren_start + 1..].trim_end_matches(')').trim();
        let nanos = nanos_str.parse::<u64>().ok();
        if ts.is_empty() {
            return None;
        }
        Some((ts, nanos))
    } else {
        let ts = raw.trim().to_owned();
        if ts.is_empty() || !ts.contains(':') {
            return None;
        }
        Some((ts, None))
    }
}

fn parse_line_number(seg: &str) -> Option<u32> {
    let inner = seg.trim_start_matches('[').trim_end_matches(']').trim();
    inner.parse().ok()
}

fn looks_like_level(s: &str) -> bool {
    matches!(
        s,
        "ERROR" | "WARN" | "INFO" | "DEBUG" | "FINE" | "FINER" | "FINEST"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_debug_line() {
        let line = "14:51:26.0 (64969664)|USER_DEBUG|[98]|DEBUG|Trigger Event: DayHandler - BeforeUpdate Query Rows:  0/50,000 {0}";
        let event = parse_line(line).unwrap();

        assert_eq!(event.timestamp, "14:51:26.0");
        assert_eq!(event.nanos, Some(64969664));
        assert_eq!(event.event_type, SfdcEventType::UserDebug);
        assert_eq!(event.line_number, Some(98));
        assert_eq!(event.level, Some(LogLevel::Debug));
        assert!(event.message.contains("Trigger Event:"));
        assert!(event.message.contains("Query Rows"));
    }

    #[test]
    fn parses_execution_started() {
        let line = "14:51:25.0 (1)|EXECUTION_STARTED";
        let event = parse_line(line).unwrap();

        assert_eq!(event.event_type, SfdcEventType::ExecutionStarted);
        assert_eq!(event.nanos, Some(1));
        assert!(event.message.is_empty());
    }

    #[test]
    fn parses_code_unit_started() {
        let line = "14:51:26.0 (100)|CODE_UNIT_STARTED|[EXTERNAL]|01q...";
        let event = parse_line(line).unwrap();

        assert_eq!(event.event_type, SfdcEventType::CodeUnitStarted);
    }

    #[test]
    fn parses_line_without_nanos() {
        let line = "14:51:26.0|USER_DEBUG|[10]|INFO|some message";
        let event = parse_line(line).unwrap();

        assert_eq!(event.timestamp, "14:51:26.0");
        assert_eq!(event.nanos, None);
        assert_eq!(event.event_type, SfdcEventType::UserDebug);
        assert_eq!(event.line_number, Some(10));
        assert_eq!(event.level, Some(LogLevel::Info));
        assert_eq!(event.message, "some message");
    }

    #[test]
    fn rejects_blank_line() {
        assert_eq!(parse_line(""), None);
        assert_eq!(parse_line("   "), None);
    }

    #[test]
    fn rejects_header_lines() {
        assert_eq!(parse_line("This is a log header with no pipes"), None);
    }

    #[test]
    fn rejects_line_without_timestamp() {
        assert_eq!(parse_line("nocolon|USER_DEBUG|[1]|INFO|msg"), None);
    }

    #[test]
    fn parse_log_extracts_multiple_events() {
        let content = "\
14:51:26.0 (100)|EXECUTION_STARTED
14:51:26.0 (200)|USER_DEBUG|[1]|DEBUG|hello
not a valid line
14:51:26.0 (300)|EXECUTION_FINISHED";

        let events = parse_log(content);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].event_type, SfdcEventType::ExecutionStarted);
        assert_eq!(events[1].event_type, SfdcEventType::UserDebug);
        assert_eq!(events[2].event_type, SfdcEventType::ExecutionFinished);
    }

    #[test]
    fn message_preserves_pipes_in_body() {
        let line = "14:51:26.0 (100)|USER_DEBUG|[5]|DEBUG|some|piped|content";
        let event = parse_line(line).unwrap();
        assert_eq!(event.message, "some|piped|content");
    }
}

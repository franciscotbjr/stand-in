//! SSE (Server-Sent Events) parser for the client-side HTTP transport.
//!
//! Incremental, chunk-safe parser that processes `text/event-stream` bytes
//! and emits every complete data event. Handles:
//! - `data:` lines (single or multi-line, joined with `\n`)
//! - `event:` / `id:` / `retry:` lines (silently ignored)
//! - Comment lines `:` (keep-alive heartbeats)
//! - Partial line boundaries across chunks (buffered internally)
//! - Multiple events in a single feed call
//!
//! # Anti-regression
//!
//! This parser **never discards events** — every completed event is yielded
//! in order. The old bug (only emitting the last event) is provably absent.

/// Incremental SSE parser that accumulates data across chunks and emits
/// complete event payloads on each empty-line boundary.
#[derive(Debug, Default)]
pub(crate) struct SseParser {
    /// Accumulated bytes for the current line (may span multiple chunks).
    line_buf: Vec<u8>,
    /// Accumulated data payload for the event currently being built.
    /// `None` until the first `data:` line of the current event.
    data_buf: Option<String>,
}

impl SseParser {
    /// Create a fresh parser with no buffered state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a chunk of raw bytes from the SSE stream.
    ///
    /// Returns every complete event payload parsed from this and any
    /// previously buffered partial chunks.  Each returned `String` is the
    /// value of the `data:` field(s) for one event.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut events = Vec::new();
        self.line_buf.extend_from_slice(chunk);

        loop {
            let newline_pos = self.line_buf.iter().position(|&b| b == b'\n');
            let Some(nl) = newline_pos else {
                // No complete line yet — wait for more bytes.
                break;
            };

            // Extract the line bytes, stripping optional trailing \r
            let is_cr = nl > 0 && self.line_buf[nl - 1] == b'\r';
            let line_end = if is_cr { nl - 1 } else { nl };
            // Clone into owned String before mutating line_buf below
            let line = String::from_utf8_lossy(&self.line_buf[..line_end]).into_owned();

            // Drain the processed line (including \n and optional \r)
            self.line_buf.drain(..=nl);

            if line.is_empty() {
                // Empty line = event boundary. Emit accumulated data.
                if let Some(data) = self.data_buf.take() {
                    events.push(data);
                }
            } else if line.starts_with(':') {
                // Comment / keep-alive — ignore.
            } else if let Some(data) = line.strip_prefix("data:") {
                // Strip optional single leading space after "data:"
                let data = data.strip_prefix(' ').unwrap_or(data);
                match self.data_buf.as_mut() {
                    Some(buf) => {
                        buf.push('\n');
                        buf.push_str(data);
                    }
                    None => {
                        self.data_buf = Some(data.to_string());
                    }
                }
            }
            // "event:", "id:", "retry:" — silently ignored per spec.
        }

        events
    }

    /// Flush any partially-built event after the stream ends.
    ///
    /// If the stream terminates without a final empty line, the last
    /// accumulated data event is returned here.
    pub fn finish(&mut self) -> Option<String> {
        self.data_buf.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_event_simple() {
        let mut p = SseParser::new();
        let events = p.feed(b"data: hello\n\n");
        assert_eq!(events, vec!["hello"]);
        assert!(p.finish().is_none());
    }

    #[test]
    fn test_multi_event_in_one_feed() {
        let mut p = SseParser::new();
        let input = b"data: first\n\ndata: second\n\n";
        let events = p.feed(input);
        assert_eq!(events, vec!["first", "second"]);
        assert!(p.finish().is_none());
    }

    #[test]
    fn test_keep_alive_comment_ignored() {
        let mut p = SseParser::new();
        let input = b": keep-alive\n\ndata: real\n\n";
        let events = p.feed(input);
        assert_eq!(events, vec!["real"]);
    }

    #[test]
    fn test_event_line_ignored() {
        let mut p = SseParser::new();
        let input = b"event: message\ndata: payload\n\n";
        let events = p.feed(input);
        assert_eq!(events, vec!["payload"]);
    }

    #[test]
    fn test_multi_line_data_joined_with_newline() {
        let mut p = SseParser::new();
        let input = b"data: line1\ndata: line2\ndata: line3\n\n";
        let events = p.feed(input);
        assert_eq!(events, vec!["line1\nline2\nline3"]);
    }

    #[test]
    fn test_chunk_split_mid_line() {
        let mut p = SseParser::new();
        // First chunk: partial line "hel"
        let events = p.feed(b"da");
        assert!(events.is_empty());
        // Second chunk: complete the line and event
        let events = p.feed(b"ta: value split\n\n");
        assert_eq!(events, vec!["value split"]);
    }

    #[test]
    fn test_chunk_split_after_data_colon() {
        let mut p = SseParser::new();
        let events = p.feed(b"data:");
        assert!(events.is_empty());
        let events = p.feed(b" value\n\n");
        assert_eq!(events, vec!["value"]);
    }

    #[test]
    fn test_chunk_split_at_newline() {
        let mut p = SseParser::new();
        let events = p.feed(b"data: slow\n");
        assert!(events.is_empty());
        let events = p.feed(b"\n");
        assert_eq!(events, vec!["slow"]);
    }

    #[test]
    fn test_does_not_emit_until_empty_line() {
        let mut p = SseParser::new();
        let events = p.feed(b"data: pending\ndata: more\n");
        assert!(events.is_empty());
        let events = p.feed(b"\n");
        assert_eq!(events, vec!["pending\nmore"]);
    }

    #[test]
    fn test_data_prefix_without_space() {
        let mut p = SseParser::new();
        let events = p.feed(b"data:compact\n\n");
        assert_eq!(events, vec!["compact"]);
    }

    #[test]
    fn test_crlf_line_endings() {
        let mut p = SseParser::new();
        let events = p.feed(b"data: value\r\n\r\n");
        assert_eq!(events, vec!["value"]);
    }

    #[test]
    fn test_mixed_newline_styles_across_events() {
        let mut p = SseParser::new();
        let input = b"data: first\r\n\r\ndata: second\n\n";
        let events = p.feed(input);
        assert_eq!(events, vec!["first", "second"]);
    }

    #[test]
    fn test_finish_flushes_partial_event() {
        let mut p = SseParser::new();
        let events = p.feed(b"data: unfinished\ndata: event\n");
        assert!(events.is_empty());
        assert_eq!(p.finish(), Some("unfinished\nevent".to_string()));
    }

    #[test]
    fn test_finish_returns_none_when_empty() {
        let mut p = SseParser::new();
        let events = p.feed(b"data: done\n\n");
        assert_eq!(events, vec!["done"]);
        assert!(p.finish().is_none());
    }

    #[test]
    fn test_multiple_feeds_across_events() {
        let mut p = SseParser::new();
        let events = p.feed(b"data: 1\n\n");
        assert_eq!(events, vec!["1"]);
        let events = p.feed(b": keep-alive\n\n");
        assert!(events.is_empty());
        let events = p.feed(b"data: 2\n\n");
        assert_eq!(events, vec!["2"]);
        let events = p.feed(b"event: ignored\n\n");
        assert!(events.is_empty());
    }

    #[test]
    fn test_many_events_no_discard() {
        // Regression test: the old bug discarded all but the last event.
        let mut p = SseParser::new();
        let body = (1..=10)
            .map(|i| format!("data: event-{i}\n\n"))
            .collect::<String>();
        let events = p.feed(body.as_bytes());
        let expected: Vec<String> = (1..=10).map(|i| format!("event-{i}")).collect();
        assert_eq!(events, expected);
    }

    #[test]
    fn test_only_comments_produce_no_events() {
        let mut p = SseParser::new();
        let events = p.feed(b": keep-alive\n: another\n\n");
        assert!(events.is_empty());
        assert!(p.finish().is_none());
    }

    #[test]
    fn test_empty_input_produces_no_events() {
        let mut p = SseParser::new();
        assert!(p.feed(b"").is_empty());
        assert!(p.finish().is_none());
    }
}

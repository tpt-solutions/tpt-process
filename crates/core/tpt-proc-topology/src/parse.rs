//! A minimal textual PFD (process flow diagram) format.
//!
//! Lines have the form
//!
//! ```text
//! U1 -> U2 [S1]        # unit U1 feeds unit U2 through stream S1
//! ```
//!
//! Blank lines and `#` comments are ignored. Whitespace is insignificant.
//! Self-loops (`U1 -> U1`) are rejected — a unit's recycle must pass through
//! at least one other unit (in `tpt-proc-core`, connections between a unit
//! and itself are structurally invalid).

use tpt_proc_core::{CoreError, Result, StreamId, UnitId};

use crate::ProcessGraph;

/// Parses PFD edge-list text into a [`ProcessGraph`].
///
/// # Errors
/// [`CoreError::InvalidFlowsheet`] for malformed lines, zero ids, or
/// self-loops.
pub fn parse_pfd(text: &str) -> Result<ProcessGraph> {
    let mut graph = ProcessGraph::new();
    for (line_no, raw) in text.lines().enumerate() {
        // Normalize bracket and arrow spacing so `U2->U3[S2]` tokenizes the
        // same as `U2 -> U3 [ S2 ]`.
        let line = raw
            .split('#')
            .next()
            .unwrap_or("")
            .replace("->", " -> ")
            .replace('[', " [ ")
            .replace(']', " ] ");
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fail = |detail: String| {
            Err(CoreError::InvalidFlowsheet(format!(
                "line {}: {detail} ({raw:?})",
                line_no + 1
            )))
        };
        let mut tokens = line.split_whitespace();
        let Some(from_tok) = tokens.next() else {
            return fail("expected 'U<n> -> U<m> [S<k>]'".into());
        };
        let Some(from) = parse_prefixed_id(from_tok, 'U') else {
            return fail(format!("expected unit id, got {from_tok:?}"));
        };
        if tokens.next() != Some("->") {
            return fail("expected '->'".into());
        }
        let Some(to_tok) = tokens.next() else {
            return fail("expected unit id after '->'".into());
        };
        let Some(to) = parse_prefixed_id(to_tok, 'U') else {
            return fail(format!("expected unit id, got {to_tok:?}"));
        };
        if tokens.next() != Some("[") {
            return fail("expected '['".into());
        }
        let Some(stream_tok) = tokens.next() else {
            return fail("expected stream id in brackets".into());
        };
        let Some(stream) = parse_prefixed_id(stream_tok, 'S').map(|u| StreamId(u.value())) else {
            return fail(format!("expected stream id, got {stream_tok:?}"));
        };
        if tokens.next() != Some("]") || tokens.next().is_some() {
            return fail("expected ']' and end of line".into());
        }
        if from.value() == 0 || to.value() == 0 || stream.value() == 0 {
            return fail("ids must be >= 1".into());
        }
        if from == to {
            return fail(format!("self-loop U{} is not allowed", from.value()));
        }
        graph.add_stream(stream, from, to);
    }
    Ok(graph)
}

/// Parses `P<n>` with prefix `P` into a [`UnitId`]; `None` when malformed.
fn parse_prefixed_id(token: &str, prefix: char) -> Option<UnitId> {
    let digits = token.strip_prefix(prefix)?;
    digits.parse::<u64>().ok().map(UnitId)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multigraph_edges() {
        let g = parse_pfd(
            "# comment line\n\nU1 -> U2 [S1]  # trailing comment\nU2->U3[S2]\nU1->U2 [S3]\n",
        )
        .unwrap();
        assert_eq!(g.edges().len(), 3);
        assert_eq!(g.units().count(), 3);
    }

    #[test]
    fn rejects_malformed() {
        assert!(parse_pfd("U1 U2 [S1]").is_err());
        assert!(parse_pfd("U1 -> U2 S1").is_err());
        assert!(parse_pfd("U1 -> U2 [S1] extra").is_err());
        assert!(parse_pfd("X1 -> U2 [S1]").is_err());
        assert!(parse_pfd("U0 -> U2 [S1]").is_err());
    }

    #[test]
    fn prefixed_ids() {
        assert!(parse_prefixed_id("U12", 'U').is_some());
        assert!(parse_prefixed_id("X12", 'U').is_none());
        assert!(parse_prefixed_id("Uabc", 'U').is_none());
    }
}

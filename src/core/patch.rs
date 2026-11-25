//! Non-destructive write modes for Spark `[[files]]` entries.
//!
//! By default a rendered file **overwrites** its destination. Setting a `mode`
//! on a `[[files]]` entry switches it to a strategy that merges the rendered
//! content into the file that is already on disk, so a template can ship
//! incremental changes — extra `CLAUDE.md` rules, a new config block, a patch —
//! that other people apply on top of their own files without losing them.
//!
//! ```toml
//! [[files]]
//! path = "CLAUDE.md"
//! mode = "append"
//! content = """
//! ## New agent rules
//! - Always run the test suite before committing.
//! """
//! ```
//!
//! The merge itself is pure (`FileMode::apply`): it takes the current file
//! contents (if any) plus the rendered fragment and returns the bytes to write.
//! Reading the destination is the caller's job, which keeps this module free of
//! I/O and easy to test.

use serde::{Deserialize, Serialize};

/// How a rendered `[[files]]` entry is written to its destination.
///
/// The default, [`FileMode::Overwrite`], reproduces Spark's original behaviour
/// (replace the file). Every other variant reads the existing file and merges
/// the rendered content into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileMode {
    /// Replace the destination with the rendered content (default).
    #[default]
    Overwrite,
    /// Append the rendered content to the end of the destination.
    Append,
    /// Insert the rendered content at the start of the destination.
    Prepend,
    /// Insert the rendered content relative to an `anchor` line (see
    /// [`Position`]). Falls back to [`FileMode::Append`] when the anchor is not
    /// found, so content is never silently dropped.
    Insert,
    /// Maintain an idempotent, marker-delimited block in the destination.
    /// Re-applying the template replaces the block in place instead of
    /// duplicating it.
    Block,
    /// Apply the rendered content to the destination as a unified diff.
    Patch,
}

/// Where [`FileMode::Insert`] places content relative to the matched anchor line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    /// Insert on the line(s) before the anchor.
    Before,
    /// Insert on the line(s) after the anchor (default).
    #[default]
    After,
}

impl FileMode {
    /// Whether this mode needs to read the current destination before writing.
    /// Only [`FileMode::Overwrite`] does not.
    pub fn reads_existing(&self) -> bool {
        !matches!(self, FileMode::Overwrite)
    }

    /// Merges `rendered` into `existing` according to this mode and returns the
    /// bytes to write to the destination.
    ///
    /// `existing` is the current content of the destination (`None` when it does
    /// not exist). `anchor`/`position` are consumed by [`FileMode::Insert`] and
    /// `marker` by [`FileMode::Block`]; the other modes ignore them.
    ///
    /// Errors are returned as a message so the caller can attach the path.
    pub fn apply(
        &self,
        existing: Option<&str>,
        rendered: &str,
        anchor: Option<&str>,
        position: Position,
        marker: Option<&str>,
    ) -> std::result::Result<String, String> {
        let existing = existing.unwrap_or("");
        match self {
            FileMode::Overwrite => Ok(rendered.to_string()),
            FileMode::Append => Ok(join_append(existing, rendered)),
            FileMode::Prepend => Ok(join_prepend(existing, rendered)),
            FileMode::Insert => insert_at_anchor(existing, rendered, anchor, position),
            FileMode::Block => block(existing, rendered, marker),
            FileMode::Patch => apply_unified(existing, rendered),
        }
    }
}

/// Detects the dominant line ending of `s`: `"\r\n"` when any CRLF is present,
/// otherwise `"\n"`. Used so merges keep a file's existing newline convention
/// instead of normalizing it.
fn line_ending(s: &str) -> &'static str {
    if s.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Appends `rendered` after `existing`, guaranteeing a newline between them.
/// The separator follows `existing`'s line-ending convention.
fn join_append(existing: &str, rendered: &str) -> String {
    if existing.is_empty() {
        return rendered.to_string();
    }
    let nl = line_ending(existing);
    let mut out = String::with_capacity(existing.len() + rendered.len() + nl.len());
    out.push_str(existing);
    if !existing.ends_with('\n') {
        out.push_str(nl);
    }
    out.push_str(rendered);
    out
}

/// Prepends `rendered` before `existing`, guaranteeing a newline between them.
/// The separator follows `existing`'s line-ending convention.
fn join_prepend(existing: &str, rendered: &str) -> String {
    if existing.is_empty() {
        return rendered.to_string();
    }
    let nl = line_ending(existing);
    let mut out = String::with_capacity(existing.len() + rendered.len() + nl.len());
    out.push_str(rendered);
    if !rendered.ends_with('\n') {
        out.push_str(nl);
    }
    out.push_str(existing);
    out
}

/// Inserts `rendered` as whole lines before/after the first line that contains
/// `anchor`. Falls back to an append when the anchor is absent.
fn insert_at_anchor(
    existing: &str,
    rendered: &str,
    anchor: Option<&str>,
    position: Position,
) -> std::result::Result<String, String> {
    let anchor = match anchor {
        Some(a) if !a.is_empty() => a,
        _ => return Err("`insert` mode requires a non-empty `anchor`".to_string()),
    };

    let lines: Vec<&str> = existing.lines().collect();
    let Some(idx) = lines.iter().position(|line| line.contains(anchor)) else {
        // Anchor not found: never drop the content, append it instead.
        return Ok(join_append(existing, rendered));
    };

    let insert_at = match position {
        Position::Before => idx,
        Position::After => idx + 1,
    };

    let rendered_lines: Vec<&str> = rendered.lines().collect();
    let mut out_lines: Vec<&str> = Vec::with_capacity(lines.len() + rendered_lines.len());
    out_lines.extend_from_slice(&lines[..insert_at]);
    out_lines.extend_from_slice(&rendered_lines);
    out_lines.extend_from_slice(&lines[insert_at..]);

    // Rejoin with the file's existing newline style so a CRLF file is not
    // silently normalized to LF (and vice versa).
    let nl = line_ending(existing);
    let mut out = out_lines.join(nl);
    if existing.ends_with('\n') {
        out.push_str(nl);
    }
    Ok(out)
}

/// Builds the start/end sentinels for a [`FileMode::Block`] with the given id.
fn block_markers(marker: Option<&str>) -> (String, String) {
    let id = marker.unwrap_or("spark");
    (
        format!("<!-- spark:{id} start -->"),
        format!("<!-- spark:{id} end -->"),
    )
}

/// Maintains an idempotent, marker-delimited block. Marker combinations are
/// handled explicitly so a corrupt managed section is reported rather than
/// silently duplicated:
///
/// - neither marker present → append a fresh block;
/// - exactly one matched pair in order → replace the region between them;
/// - a lone marker, reversed order, or multiple pairs → error.
fn block(
    existing: &str,
    rendered: &str,
    marker: Option<&str>,
) -> std::result::Result<String, String> {
    let (start, end) = block_markers(marker);
    let body = rendered.trim_end_matches('\n');
    let fresh_block = format!("{start}\n{body}\n{end}");

    let start_count = existing.matches(start.as_str()).count();
    let end_count = existing.matches(end.as_str()).count();

    match (start_count, end_count) {
        (0, 0) => Ok(join_append(existing, &fresh_block)),
        (1, 1) => {
            let s = existing.find(start.as_str()).unwrap();
            let e = existing.find(end.as_str()).unwrap();
            if s < e {
                let end_idx = e + end.len();
                let mut out = String::with_capacity(existing.len() + fresh_block.len());
                out.push_str(&existing[..s]);
                out.push_str(&fresh_block);
                out.push_str(&existing[end_idx..]);
                Ok(out)
            } else {
                Err(format!(
                    "block markers are out of order in the destination (`{end}` appears before `{start}`)"
                ))
            }
        }
        _ => Err(format!(
            "destination has malformed block markers (found {start_count} `{start}` / \
             {end_count} `{end}`; expected a single matched pair)"
        )),
    }
}

/// Applies `rendered` to `existing` as a unified diff via `diffy`.
fn apply_unified(existing: &str, rendered: &str) -> std::result::Result<String, String> {
    let patch =
        diffy::Patch::from_str(rendered).map_err(|e| format!("invalid unified diff: {e}"))?;
    diffy::apply(existing, &patch).map_err(|e| format!("could not apply unified diff: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overwrite_ignores_existing() {
        let out = FileMode::Overwrite
            .apply(Some("old"), "new", None, Position::After, None)
            .unwrap();
        assert_eq!(out, "new");
    }

    #[test]
    fn append_adds_newline_between() {
        let out = FileMode::Append
            .apply(Some("a"), "b", None, Position::After, None)
            .unwrap();
        assert_eq!(out, "a\nb");
    }

    #[test]
    fn append_keeps_single_separator_when_existing_ends_with_newline() {
        let out = FileMode::Append
            .apply(Some("a\n"), "b", None, Position::After, None)
            .unwrap();
        assert_eq!(out, "a\nb");
    }

    #[test]
    fn append_to_missing_file_is_just_rendered() {
        let out = FileMode::Append
            .apply(None, "fresh", None, Position::After, None)
            .unwrap();
        assert_eq!(out, "fresh");
    }

    #[test]
    fn prepend_adds_before_existing() {
        let out = FileMode::Prepend
            .apply(Some("world"), "hello", None, Position::After, None)
            .unwrap();
        assert_eq!(out, "hello\nworld");
    }

    #[test]
    fn insert_after_anchor_line() {
        let existing = "# Title\n## Conventions\n- keep it tidy\n";
        let out = FileMode::Insert
            .apply(
                Some(existing),
                "- new rule",
                Some("## Conventions"),
                Position::After,
                None,
            )
            .unwrap();
        assert_eq!(out, "# Title\n## Conventions\n- new rule\n- keep it tidy\n");
    }

    #[test]
    fn insert_before_anchor_line() {
        let existing = "alpha\nomega\n";
        let out = FileMode::Insert
            .apply(
                Some(existing),
                "beta",
                Some("omega"),
                Position::Before,
                None,
            )
            .unwrap();
        assert_eq!(out, "alpha\nbeta\nomega\n");
    }

    #[test]
    fn insert_preserves_crlf_line_endings() {
        let existing = "[core]\r\neditor = \"vim\"\r\n[net]\r\n";
        let out = FileMode::Insert
            .apply(
                Some(existing),
                "timeout = 30",
                Some("[net]"),
                Position::After,
                None,
            )
            .unwrap();
        assert_eq!(
            out,
            "[core]\r\neditor = \"vim\"\r\n[net]\r\ntimeout = 30\r\n"
        );
    }

    #[test]
    fn append_preserves_crlf_separator() {
        let out = FileMode::Append
            .apply(Some("a\r\nb"), "c", None, Position::After, None)
            .unwrap();
        assert_eq!(out, "a\r\nb\r\nc");
    }

    #[test]
    fn insert_falls_back_to_append_when_anchor_missing() {
        let out = FileMode::Insert
            .apply(Some("a\n"), "b", Some("NOPE"), Position::After, None)
            .unwrap();
        assert_eq!(out, "a\nb");
    }

    #[test]
    fn insert_without_anchor_errors() {
        let err = FileMode::Insert
            .apply(Some("a"), "b", None, Position::After, None)
            .unwrap_err();
        assert!(err.contains("anchor"));
    }

    #[test]
    fn block_appends_when_absent() {
        let out = FileMode::Block
            .apply(
                Some("# Doc\n"),
                "rules",
                None,
                Position::After,
                Some("rules"),
            )
            .unwrap();
        assert_eq!(
            out,
            "# Doc\n<!-- spark:rules start -->\nrules\n<!-- spark:rules end -->"
        );
    }

    #[test]
    fn block_is_idempotent_and_replaces_in_place() {
        let first = FileMode::Block
            .apply(Some("intro\n"), "v1", None, Position::After, Some("r"))
            .unwrap();
        // Re-applying with new content must replace the block, not duplicate it.
        let second = FileMode::Block
            .apply(Some(&first), "v2", None, Position::After, Some("r"))
            .unwrap();
        assert_eq!(
            second,
            "intro\n<!-- spark:r start -->\nv2\n<!-- spark:r end -->"
        );
        assert_eq!(second.matches("spark:r start").count(), 1);
    }

    #[test]
    fn block_errors_on_lone_start_marker() {
        let existing = "before\n<!-- spark:x start -->\norphan\n";
        let err = FileMode::Block
            .apply(Some(existing), "new", None, Position::After, Some("x"))
            .unwrap_err();
        assert!(err.contains("malformed"), "unexpected message: {err}");
    }

    #[test]
    fn block_errors_on_lone_end_marker() {
        let existing = "before\n<!-- spark:x end -->\n";
        let err = FileMode::Block
            .apply(Some(existing), "new", None, Position::After, Some("x"))
            .unwrap_err();
        assert!(err.contains("malformed"), "unexpected message: {err}");
    }

    #[test]
    fn block_errors_on_multiple_pairs() {
        let existing = concat!(
            "<!-- spark:x start -->\na\n<!-- spark:x end -->\n",
            "<!-- spark:x start -->\nb\n<!-- spark:x end -->\n",
        );
        let err = FileMode::Block
            .apply(Some(existing), "new", None, Position::After, Some("x"))
            .unwrap_err();
        assert!(err.contains("malformed"), "unexpected message: {err}");
    }

    #[test]
    fn block_errors_on_reversed_markers() {
        let existing = "<!-- spark:x end -->\nmiddle\n<!-- spark:x start -->\n";
        let err = FileMode::Block
            .apply(Some(existing), "new", None, Position::After, Some("x"))
            .unwrap_err();
        assert!(err.contains("out of order"), "unexpected message: {err}");
    }

    #[test]
    fn block_preserves_text_after_the_markers() {
        let existing = "before\n<!-- spark:x start -->\nold\n<!-- spark:x end -->\nafter\n";
        let out = FileMode::Block
            .apply(Some(existing), "new", None, Position::After, Some("x"))
            .unwrap();
        assert_eq!(
            out,
            "before\n<!-- spark:x start -->\nnew\n<!-- spark:x end -->\nafter\n"
        );
    }

    #[test]
    fn patch_applies_unified_diff() {
        let existing = "line one\nline two\nline three\n";
        let diff = diffy::create_patch(existing, "line one\nline TWO\nline three\n").to_string();
        let out = FileMode::Patch
            .apply(Some(existing), &diff, None, Position::After, None)
            .unwrap();
        assert_eq!(out, "line one\nline TWO\nline three\n");
    }

    #[test]
    fn patch_reports_when_diff_does_not_apply() {
        // A hunk whose context does not match the base must surface an error
        // rather than silently corrupting the file.
        let diff = "--- a\n+++ b\n@@ -1 +1 @@\n-nonexistent line\n+replacement\n";
        let err = FileMode::Patch
            .apply(
                Some("totally different\n"),
                diff,
                None,
                Position::After,
                None,
            )
            .unwrap_err();
        assert!(err.contains("unified diff"), "unexpected message: {err}");
    }

    #[test]
    fn mode_deserializes_from_lowercase() {
        let m: FileMode = serde_json::from_str("\"append\"").unwrap();
        assert_eq!(m, FileMode::Append);
        let p: Position = serde_json::from_str("\"before\"").unwrap();
        assert_eq!(p, Position::Before);
    }

    #[test]
    fn reads_existing_is_false_only_for_overwrite() {
        assert!(!FileMode::Overwrite.reads_existing());
        assert!(FileMode::Append.reads_existing());
        assert!(FileMode::Patch.reads_existing());
    }
}

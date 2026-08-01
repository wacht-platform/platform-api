use super::{AgentFilesystem, EditFileResult, ReadFileResult, WriteFileResult};
use crate::sandbox::{ExecRequest, SandboxError};
use commands::WriteToDeploymentStorageCommand;
use common::error::AppError;
use common::ResultExt;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Duration;

fn map_sandbox_error(path: &str, op: &str, err: SandboxError) -> AppError {
    match err {
        SandboxError::NotFound(msg) => AppError::NotFound(format!("{op} {path}: {msg}")),
        SandboxError::Timeout(msg) => AppError::Internal(format!("{op} {path}: timed out: {msg}")),
        SandboxError::Cancelled => AppError::Internal(format!("{op} {path}: cancelled")),
        SandboxError::Transient(msg) => {
            AppError::Internal(format!("{op} {path}: transient sandbox error: {msg}"))
        }
        SandboxError::Config(msg) => {
            AppError::Internal(format!("{op} {path}: sandbox config error: {msg}"))
        }
        SandboxError::Other(msg) => AppError::Internal(format!("{op} {path}: {msg}")),
    }
}

impl AgentFilesystem {
    pub async fn save_upload(&self, filename: &str, data: &[u8]) -> Result<String, AppError> {
        let key = format!(
            "{}/persistent/{}/uploads/{}",
            self.deployment_id, self.thread_id, filename
        );
        WriteToDeploymentStorageCommand::new(self.deployment_id, key, data.to_vec())
            .execute_with_deps(&common::deps::from_app(&self.app_state).db().enc())
            .await?;
        Ok(format!("/uploads/{}", filename))
    }

    /// Cheap existence check — runs `test -e` in the sandbox. Returns `Ok(true)`
    /// for files and directories alike, `Ok(false)` if absent. Used by terminal
    /// status validation to confirm declared artifacts actually exist on disk.
    pub async fn exists(&self, path: &str) -> Result<bool, AppError> {
        let result = self
            .sandbox_handle
            .exec(ExecRequest {
                command: vec![
                    "bash".into(),
                    "-c".into(),
                    "test -e \"$1\"".into(),
                    "_".into(),
                    sandbox_path(path),
                ],
                cwd: None,
                env: BTreeMap::new(),
                timeout: Some(Duration::from_secs(5)),
                exec_id: None,
            })
            .await
            .map_err(|e| map_sandbox_error(path, "exists", e))?;
        Ok(result.exit_code == 0)
    }

    pub async fn read_file_bytes(&self, path: &str) -> Result<Vec<u8>, AppError> {
        self.sandbox_handle
            .read_file(&sandbox_path(path))
            .await
            .map_err(|e| map_sandbox_error(path, "read", e))
    }

    pub async fn read_file(
        &self,
        path: &str,
        start_line: Option<usize>,
        end_line: Option<usize>,
        start_char: Option<usize>,
        end_char: Option<usize>,
    ) -> Result<ReadFileResult, AppError> {
        let bytes = self.read_file_bytes(path).await?;
        let content = String::from_utf8(bytes)
            .map_err(|e| AppError::Internal(format!("file {} is not valid utf-8: {}", path, e)))?;

        let total_lines = content.lines().count();
        let total_chars = content.chars().count();

        if start_char.is_some() || end_char.is_some() {
            let start = start_char.unwrap_or(1).saturating_sub(1).min(total_chars);
            let end = end_char.unwrap_or(total_chars).min(total_chars).max(start);
            let slice: String = content.chars().skip(start).take(end - start).collect();
            let slice_hash = Self::slice_hash(&slice);
            self.mark_read(path);
            return Ok(ReadFileResult {
                content: slice,
                total_lines,
                total_chars,
                start_line: 0,
                end_line: 0,
                start_char: Some(start + 1),
                end_char: Some(end),
                slice_hash,
            });
        }

        let lines: Vec<&str> = content.lines().collect();
        let start = start_line.unwrap_or(1).saturating_sub(1);
        let end = end_line.unwrap_or(total_lines).min(total_lines);

        let raw_slice = lines
            .iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let slice_hash = Self::slice_hash(&raw_slice);
        self.mark_read(path);

        Ok(ReadFileResult {
            content: raw_slice,
            total_lines,
            total_chars,
            start_line: start + 1,
            end_line: end,
            start_char: None,
            end_char: None,
            slice_hash,
        })
    }

    pub async fn write_file(
        &self,
        path: &str,
        content: &str,
        append: bool,
    ) -> Result<WriteFileResult, AppError> {
        let final_bytes = if append {
            let existing = self
                .sandbox_handle
                .read_file(&sandbox_path(path))
                .await
                .unwrap_or_default();
            let mut buf = existing;
            if !buf.is_empty() && buf.last() != Some(&b'\n') && !content.starts_with('\n') {
                buf.push(b'\n');
            }
            buf.extend_from_slice(content.as_bytes());
            if buf.last() != Some(&b'\n') {
                buf.push(b'\n');
            }
            buf
        } else {
            content.as_bytes().to_vec()
        };

        self.sandbox_handle
            .write_file(&sandbox_path(path), &final_bytes)
            .await
            .map_err(|e| map_sandbox_error(path, "write", e))?;

        self.unmark_read(path);

        let total_lines = String::from_utf8_lossy(&final_bytes).lines().count();
        Ok(WriteFileResult {
            lines_written: content.lines().count(),
            total_lines,
            partial: false,
        })
    }

    pub async fn edit_file(
        &self,
        path: &str,
        old_string: &str,
        new_string: &str,
        replace_all: bool,
    ) -> Result<EditFileResult, AppError> {
        if old_string.is_empty() {
            return Err(AppError::BadRequest(
                "edit_file: old_string must not be empty. To create or fully overwrite a file use write_file; to add content at end-of-file use append_file.".to_string(),
            ));
        }

        if old_string == new_string {
            return Err(AppError::BadRequest(
                "edit_file: old_string and new_string are identical — this would be a no-op."
                    .to_string(),
            ));
        }

        if !self.has_read_path(path) {
            return Err(AppError::BadRequest(format!(
                "edit_file: must read_file({path}) at least once this turn before editing. The Read step is what proves you've seen the current file state."
            )));
        }

        let existing_bytes = self.read_file_bytes(path).await.map_err(|e| match e {
            AppError::NotFound(_) => AppError::BadRequest(format!(
                "edit_file: {path} does not exist. Use write_file to create it."
            )),
            other => other,
        })?;
        let existing = String::from_utf8(existing_bytes)
            .map_err(|e| AppError::Internal(format!("file {} is not valid utf-8: {}", path, e)))?;

        let count = existing.matches(old_string).count();
        let (new_content, replacements) = if count > 0 {
            if count > 1 && !replace_all {
                return Err(AppError::BadRequest(format!(
                    "edit_file: old_string matched {count} times in {path}. Add surrounding context to make it unique, or set replace_all=true to replace every occurrence."
                )));
            }

            let new_content = if replace_all {
                existing.replace(old_string, new_string)
            } else {
                existing.replacen(old_string, new_string, 1)
            };
            (new_content, if replace_all { count } else { 1 })
        } else if !replace_all {
            match flexible_replace(&existing, old_string, new_string) {
                FlexibleEdit::Replaced(new_content) => (new_content, 1),
                FlexibleEdit::Ambiguous(lines) => {
                    return Err(AppError::BadRequest(format!(
                        "edit_file: old_string matched {} places in {path} after ignoring source whitespace (matches begin on lines {:?}). Add surrounding context to make it unique, or set replace_all=true for exact matches.",
                        lines.len(),
                        lines
                    )));
                }
                FlexibleEdit::NoMatch => {
                    return Err(AppError::BadRequest(format!(
                        "edit_file: old_string not found in {path}. Matching ignores only source whitespace (indentation, line breaks, and repeated spaces); copy the exact non-whitespace text from read_file and keep it unique."
                    )));
                }
            }
        } else {
            return Err(AppError::BadRequest(format!(
                "edit_file: old_string not found in {path}. The file may have changed since you last read it, or your old_string contains paraphrased whitespace. Re-read the file and copy the exact bytes."
            )));
        };

        self.sandbox_handle
            .write_file(&sandbox_path(path), new_content.as_bytes())
            .await
            .map_err(|e| map_sandbox_error(path, "write", e))?;

        self.unmark_read(path);

        Ok(EditFileResult {
            replacements,
            total_lines: new_content.lines().count(),
        })
    }

    fn has_read_path(&self, path: &str) -> bool {
        let tracked_path = Self::tracked_path(path);
        self.read_paths
            .read()
            .ok()
            .map(|read_paths| read_paths.contains(&tracked_path))
            .unwrap_or(false)
    }

    pub async fn list_dir(&self, path: &str) -> Result<Vec<String>, AppError> {
        let result = self
            .sandbox_handle
            .exec(ExecRequest {
                command: vec![
                    "bash".into(),
                    "-c".into(),
                    "ls -1AF \"$1\"".into(),
                    "_".into(),
                    sandbox_path(path),
                ],
                cwd: None,
                env: BTreeMap::new(),
                timeout: Some(Duration::from_secs(30)),
                exec_id: None,
            })
            .await
            .map_err(|e| AppError::Internal(format!("list_dir {}: {}", path, e)))?;

        if result.exit_code != 0 {
            return Err(AppError::NotFound(format!("Directory not found: {}", path)));
        }

        Ok(String::from_utf8_lossy(&result.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect())
    }

    pub async fn search(&self, query: &str, path: &str) -> Result<String, AppError> {
        let result = self
            .sandbox_handle
            .exec(ExecRequest {
                command: vec![
                    "bash".into(),
                    "-c".into(),
                    "rg --json -i --follow \"$1\" \"$2\"".into(),
                    "_".into(),
                    query.to_string(),
                    sandbox_path(path),
                ],
                cwd: None,
                env: BTreeMap::new(),
                timeout: Some(Duration::from_secs(60)),
                exec_id: None,
            })
            .await
            .map_err_internal("search")?;

        if result.exit_code != 0 && result.exit_code != 1 {
            return Err(AppError::Internal(format!(
                "Search error: {}",
                String::from_utf8_lossy(&result.stderr)
            )));
        }
        Ok(String::from_utf8_lossy(&result.stdout).to_string())
    }

    fn unmark_read(&self, path: &str) {
        let tracked_path = Self::tracked_path(path);
        if let Ok(mut read_paths) = self.read_paths.write() {
            read_paths.remove(&tracked_path);
        }
    }

    fn mark_read(&self, path: &str) {
        let tracked_path = Self::tracked_path(path);
        if let Ok(mut read_paths) = self.read_paths.write() {
            read_paths.insert(tracked_path);
        }
    }

    pub(crate) fn format_numbered_line(line_number: usize, line: &str) -> String {
        format!("{}: {}", line_number, line)
    }

    fn tracked_path(path: &str) -> String {
        path.trim_start_matches('/').to_string()
    }

    fn slice_hash(value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        let digest = hasher.finalize();
        let mut encoded = String::with_capacity(digest.len() * 2);
        for byte in digest {
            encoded.push_str(&format!("{:02x}", byte));
        }
        encoded
    }
}

fn sandbox_path(path: &str) -> String {
    format!("/{}", path.trim_start_matches('/'))
}

struct FlexibleSource {
    /// Whitespace is represented by one separator only when it separates two
    /// word characters. All non-whitespace source bytes are preserved exactly.
    text: String,
    /// Source byte start/end offsets for each byte in `text`.
    starts: Vec<usize>,
    ends: Vec<usize>,
}

enum FlexibleEdit {
    Replaced(String),
    /// 1-based line number for each normalized match.
    Ambiguous(Vec<usize>),
    NoMatch,
}

/// Normalize only source whitespace while retaining the original byte span for
/// every normalized byte. Whitespace inside quoted strings is preserved so a
/// lenient edit cannot change string contents accidentally.
fn normalize_source(source: &str) -> FlexibleSource {
    let mut text = String::new();
    let mut starts = Vec::new();
    let mut ends = Vec::new();
    let mut pending_whitespace: Option<(usize, usize)> = None;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut previous_word = false;

    let mut emit = |ch: char, start: usize, end: usize| {
        text.push(ch);
        for _ in 0..ch.len_utf8() {
            starts.push(start);
            ends.push(end);
        }
    };

    for (start, ch) in source.char_indices() {
        let end = start + ch.len_utf8();
        if let Some(active_quote) = quote {
            emit(ch, start, end);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            previous_word = false;
            continue;
        }

        let current_word = ch.is_alphanumeric() || ch == '_';
        if matches!(ch, '\'' | '"' | '`') {
            pending_whitespace = None;
            quote = Some(ch);
            emit(ch, start, end);
            previous_word = false;
            continue;
        }

        if ch.is_whitespace() {
            pending_whitespace = Some(match pending_whitespace {
                Some((whitespace_start, _)) => (whitespace_start, end),
                None => (start, end),
            });
            continue;
        }

        if let Some((whitespace_start, whitespace_end)) = pending_whitespace.take() {
            if previous_word && current_word {
                emit(' ', whitespace_start, whitespace_end);
            }
        }
        emit(ch, start, end);
        previous_word = current_word;
    }

    FlexibleSource { text, starts, ends }
}

/// Replace one unique source span after ignoring insignificant whitespace.
/// Exact matching remains the fast path in `edit_file`; this fallback only
/// relaxes indentation, line wrapping, and blank-line differences while keeping
/// non-whitespace source tokens exact.
fn flexible_replace(content: &str, old: &str, new: &str) -> FlexibleEdit {
    let needle = normalize_source(old).text;
    if needle.is_empty() {
        return FlexibleEdit::NoMatch;
    }

    let source = normalize_source(content);
    let mut matches = Vec::new();
    let mut from = 0usize;
    while let Some(position) = source.text[from..].find(&needle) {
        let start = from + position;
        let end = start + needle.len();
        matches.push((source.starts[start], source.ends[end - 1]));
        from = start + needle.len().max(1);
    }

    match matches.as_slice() {
        [] => FlexibleEdit::NoMatch,
        &[(start, end)] => {
            let mut updated = String::with_capacity(content.len() + new.len());
            updated.push_str(&content[..start]);
            updated.push_str(new);
            updated.push_str(&content[end..]);
            FlexibleEdit::Replaced(updated)
        }
        multiple => FlexibleEdit::Ambiguous(
            multiple
                .iter()
                .map(|(start, _)| content[..*start].matches('\n').count() + 1)
                .collect(),
        ),
    }
}

#[cfg(test)]
mod flexible_edit_tests {
    use super::{flexible_replace, FlexibleEdit};

    #[test]
    fn matches_source_with_different_whitespace() {
        let source = "const value = build(\n    first,\tsecond\n);\n";
        let old = "  const   value = build(first, second);  ";

        let FlexibleEdit::Replaced(updated) = flexible_replace(source, old, "replacement") else {
            panic!("whitespace-normalized source should match");
        };
        assert_eq!(updated, "replacement\n");
    }

    #[test]
    fn rejects_changed_non_whitespace_tokens() {
        let source = "const value = build(first, second);\n";
        let old = "const value = build(first, changed);";

        assert!(matches!(
            flexible_replace(source, old, "replacement"),
            FlexibleEdit::NoMatch
        ));
    }

    #[test]
    fn rejects_ambiguous_normalized_matches() {
        let source = "const value = build(first, second);\n\nconst value = build(first, second);\n";
        let old = "const value = build(first, second);";

        let FlexibleEdit::Ambiguous(lines) = flexible_replace(source, old, "replacement") else {
            panic!("repeated normalized source should be ambiguous");
        };
        assert_eq!(lines, vec![1, 3]);
    }

    #[test]
    fn preserves_whitespace_inside_quoted_strings() {
        let source = "let value = \"hello  world\";\n";
        let old = "let value = \"hello world\";";

        assert!(matches!(
            flexible_replace(source, old, "replacement"),
            FlexibleEdit::NoMatch
        ));
    }
}

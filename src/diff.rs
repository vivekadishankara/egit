use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub old_path: String,
    pub new_path: String,
    pub status: String,
    pub extension: String,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
    pub highlighted: String,
    pub inline_diff: Option<String>,
}

#[cfg(feature = "ssr")]
pub fn parse_diff(raw: &str) -> Vec<DiffFile> {
    let mut files: Vec<DiffFile> = Vec::new();
    let mut current_file: Option<DiffFileBuilder> = None;

    for line in raw.lines() {
        if line.starts_with("diff --git ") {
            if let Some(builder) = current_file.take() {
                if let Some(file) = builder.build() {
                    files.push(file);
                }
            }
            current_file = Some(DiffFileBuilder::new(line));
        } else if let Some(ref mut builder) = current_file {
            builder.process_line(line);
        }
    }

    if let Some(builder) = current_file.take() {
        if let Some(file) = builder.build() {
            files.push(file);
        }
    }

    files
}

#[cfg(feature = "ssr")]
struct DiffFileBuilder {
    old_path: String,
    new_path: String,
    extension: String,
    hunks: Vec<DiffHunkBuilder>,
    current_hunk: Option<DiffHunkBuilder>,
    has_content: bool,
    is_binary: bool,
}

#[cfg(feature = "ssr")]
impl DiffFileBuilder {
    fn new(header: &str) -> Self {
        let (old, new) = parse_diff_git_header(header);
        let ext = new.rsplit('.').next()
            .map(|s| s.to_string())
            .unwrap_or_default();

        Self {
            old_path: old,
            new_path: new,
            extension: ext,
            hunks: Vec::new(),
            current_hunk: None,
            has_content: false,
            is_binary: false,
        }
    }

    fn process_line(&mut self, line: &str) {
        if line.starts_with("Binary files ") {
            self.is_binary = true;
            return;
        }

        if line.starts_with("--- ") {
            self.old_path = line[4..].trim_start_matches("a/").to_string();
            return;
        }
        if line.starts_with("+++ ") {
            self.new_path = line[4..].trim_start_matches("b/").to_string();
            let parts: Vec<&str> = self.new_path.rsplit('.').collect();
            if parts.len() > 1 {
                self.extension = parts[0].to_string();
            }
            return;
        }

        if line.starts_with("new file mode") {
            self.has_content = true;
            return;
        }
        if line.starts_with("deleted file mode") {
            self.has_content = true;
            return;
        }
        if line.starts_with("rename from ") || line.starts_with("rename to ") {
            self.has_content = true;
            return;
        }

        if line.starts_with("@@") {
            if let Some(hunk) = self.current_hunk.take() {
                self.hunks.push(hunk);
            }
            self.current_hunk = Some(DiffHunkBuilder::new(line));
            return;
        }

        if let Some(ref mut hunk) = self.current_hunk {
            if line.starts_with('+') {
                hunk.add_line("add", &line[1..]);
                self.has_content = true;
            } else if line.starts_with('-') {
                hunk.add_line("delete", &line[1..]);
                self.has_content = true;
            } else if line.starts_with(' ') {
                hunk.add_line("context", &line[1..]);
            }
        }
    }

    fn build(mut self) -> Option<DiffFile> {
        if let Some(hunk) = self.current_hunk.take() {
            self.hunks.push(hunk);
        }

        if !self.has_content && !self.is_binary {
            return None;
        }

        let status = if self.is_binary {
            "binary".to_string()
        } else if self.old_path == "/dev/null" || self.old_path.starts_with("dev/null") {
            "added".to_string()
        } else if self.new_path == "/dev/null" || self.new_path.starts_with("dev/null") {
            "deleted".to_string()
        } else {
            "modified".to_string()
        };

        let mut total_add = 0u32;
        let mut total_del = 0u32;

        let mut hunks: Vec<DiffHunk> = Vec::new();
        for raw_hunk in self.hunks {
            let (hunk, add, del) = raw_hunk.build();
            total_add += add;
            total_del += del;
            hunks.push(hunk);
        }

        // Apply inline word diffs to matched +/- pairs within each hunk
        for hunk in &mut hunks {
            apply_inline_word_diff(&mut hunk.lines);
        }

        Some(DiffFile {
            old_path: self.old_path,
            new_path: self.new_path,
            status,
            extension: self.extension,
            hunks,
            stats: DiffStats {
                additions: total_add,
                deletions: total_del,
            },
        })
    }
}

#[cfg(feature = "ssr")]
struct DiffHunkBuilder {
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    header: String,
    lines: Vec<(String, String)>,
}

#[cfg(feature = "ssr")]
impl DiffHunkBuilder {
    fn new(header_line: &str) -> Self {
        let (old_start, old_lines, new_start, new_lines) = parse_hunk_header(header_line);

        Self {
            old_start,
            old_lines,
            new_start,
            new_lines,
            header: header_line.to_string(),
            lines: Vec::new(),
        }
    }

    fn add_line(&mut self, line_type: &str, content: &str) {
        self.lines.push((line_type.to_string(), content.to_string()));
    }

    fn build(self) -> (DiffHunk, u32, u32) {
        let mut old_lineno = self.old_start;
        let mut new_lineno = self.new_start;
        let mut add = 0u32;
        let mut del = 0u32;

        let diff_lines: Vec<DiffLine> = self
            .lines
            .iter()
            .map(|(line_type, content)| {
                match line_type.as_str() {
                    "add" => {
                        let n = new_lineno;
                        new_lineno += 1;
                        add += 1;
                        DiffLine {
                            line_type: "add".to_string(),
                            old_lineno: None,
                            new_lineno: Some(n),
                            content: content.clone(),
                            highlighted: String::new(),
                            inline_diff: None,
                        }
                    }
                    "delete" => {
                        let o = old_lineno;
                        old_lineno += 1;
                        del += 1;
                        DiffLine {
                            line_type: "delete".to_string(),
                            old_lineno: Some(o),
                            new_lineno: None,
                            content: content.clone(),
                            highlighted: String::new(),
                            inline_diff: None,
                        }
                    }
                    _ => {
                        let o = old_lineno;
                        let n = new_lineno;
                        old_lineno += 1;
                        new_lineno += 1;
                        DiffLine {
                            line_type: "context".to_string(),
                            old_lineno: Some(o),
                            new_lineno: Some(n),
                            content: content.clone(),
                            highlighted: String::new(),
                            inline_diff: None,
                        }
                    }
                }
            })
            .collect();

        (
            DiffHunk {
                old_start: self.old_start,
                old_lines: self.old_lines,
                new_start: self.new_start,
                new_lines: self.new_lines,
                header: self.header,
                lines: diff_lines,
            },
            add,
            del,
        )
    }
}

#[cfg(feature = "ssr")]
fn parse_diff_git_header(line: &str) -> (String, String) {
    let rest = line.strip_prefix("diff --git ").unwrap_or(line);
    let parts: Vec<&str> = rest.split(' ').collect();
    let old = parts
        .first()
        .map(|s| s.strip_prefix("a/").unwrap_or(s).to_string())
        .unwrap_or_default();
    let new = parts
        .last()
        .map(|s| s.strip_prefix("b/").unwrap_or(s).to_string())
        .unwrap_or_default();
    (old, new)
}

#[cfg(feature = "ssr")]
fn parse_hunk_header(line: &str) -> (u32, u32, u32, u32) {
    let rest = line
        .strip_prefix("@@ ")
        .and_then(|s| s.split(" @@").next())
        .unwrap_or("");

    let parts: Vec<&str> = rest.split(' ').collect();

    let old_part = parts.first().unwrap_or(&"-0,0");
    let new_part = parts.get(1).unwrap_or(&"+0,0");

    let (old_start, old_len) = parse_range(old_part);
    let (new_start, new_len) = parse_range(new_part);

    (old_start, old_len, new_start, new_len)
}

#[cfg(feature = "ssr")]
fn parse_range(part: &str) -> (u32, u32) {
    let part = if part.starts_with('-') || part.starts_with('+') {
        &part[1..]
    } else {
        part
    };

    if let Some((start, count)) = part.split_once(',') {
        (start.parse().unwrap_or(1), count.parse().unwrap_or(1))
    } else {
        (part.parse().unwrap_or(1), 1)
    }
}

#[cfg(feature = "ssr")]
fn apply_inline_word_diff(lines: &mut Vec<DiffLine>) {
    let mut i = 0;
    while i < lines.len() {
        if lines[i].line_type == "delete" {
            if let Some(add_idx) = find_matching_add(lines, i + 1) {
                let old_text = &lines[i].content;
                let new_text = &lines[add_idx].content;
                if old_text != new_text {
                    let word_diff = word_diff_html(old_text, new_text);
                    lines[add_idx].inline_diff = Some(word_diff);
                }
                i = add_idx + 1;
                continue;
            }
        }
        i += 1;
    }
}

#[cfg(feature = "ssr")]
fn find_matching_add(lines: &[DiffLine], start: usize) -> Option<usize> {
    for j in start..lines.len() {
        if lines[j].line_type == "add" {
            return Some(j);
        }
        if lines[j].line_type != "delete" {
            return None;
        }
    }
    None
}

#[cfg(feature = "ssr")]
fn word_diff_html(old_text: &str, new_text: &str) -> String {
    let old_tokens = tokenize(old_text);
    let new_tokens = tokenize(new_text);

    let lcs = lcs_indices(&old_tokens, &new_tokens);
    let mut old_i = 0usize;
    let mut new_i = 0usize;
    let mut html = String::new();

    while old_i < old_tokens.len() || new_i < new_tokens.len() {
        let in_lcs = lcs.contains(&(old_i, new_i));

        if in_lcs {
            html.push_str(old_tokens[old_i]);
            old_i += 1;
            new_i += 1;
        } else {
            let mut del_group = String::new();
            while old_i < old_tokens.len()
                && !lcs.contains(&(old_i, new_i))
            {
                del_group.push_str(old_tokens[old_i]);
                old_i += 1;
            }
            if !del_group.is_empty() {
                html.push_str("<span class=\"diff-word-del\">");
                html.push_str(&escape_html(&del_group));
                html.push_str("</span>");
            }

            let mut add_group = String::new();
            while new_i < new_tokens.len()
                && !lcs.contains(&(old_i, new_i))
            {
                add_group.push_str(new_tokens[new_i]);
                new_i += 1;
            }
            if !add_group.is_empty() {
                html.push_str("<span class=\"diff-word-add\">");
                html.push_str(&escape_html(&add_group));
                html.push_str("</span>");
            }
        }
    }

    html
}

#[cfg(feature = "ssr")]
fn tokenize(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = 0usize;
    let bytes = s.as_bytes();
    let len = bytes.len();

    while start < len {
        if bytes[start].is_ascii_alphanumeric() || bytes[start] == b'_' {
            let mut end = start + 1;
            while end < len
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
            {
                end += 1;
            }
            tokens.push(&s[start..end]);
            start = end;
        } else {
            let mut end = start + 1;
            while end < len
                && !bytes[end].is_ascii_alphanumeric()
                && bytes[end] != b'_'
            {
                end += 1;
            }
            tokens.push(&s[start..end]);
            start = end;
        }
    }

    tokens
}

#[cfg(feature = "ssr")]
fn lcs_indices<'a>(
    old: &[&'a str],
    new: &[&'a str],
) -> Vec<(usize, usize)> {
    let m = old.len();
    let n = new.len();

    if m == 0 || n == 0 {
        return Vec::new();
    }

    let mut dp = vec![vec![0u32; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            result.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

#[cfg(feature = "ssr")]
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

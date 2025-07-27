/// Determina se un commento contiene codice sorgente in percentuale superiore alla soglia
pub fn is_code_comment(comment: &str, threshold: u8) -> bool {
    // Semplice euristica: conta le linee che sembrano codice (contengono ;, {, }, #include, #define, ecc)
    let lines: Vec<&str> = comment.lines().collect();
    if lines.is_empty() {
        return false;
    }
    let code_lines = lines.iter().filter(|l| is_code_line(l)).count();
    let percent = (code_lines as f32 / lines.len() as f32) * 100.0;
    percent >= threshold as f32
}

fn is_code_line(line: &str) -> bool {
    let l = line.trim();
    l.contains(';')
        || l.contains('{')
        || l.contains('}')
        || l.starts_with("#include")
        || l.starts_with("#define")
        || l.starts_with("#ifdef")
        || l.starts_with("#ifndef")
        || l.starts_with("#endif")
        || l.starts_with("//")
        || l.starts_with("/*")
        || l.starts_with("* ")
}

/// Unisce commenti adiacenti (senza linee di codice tra di loro) in un'unica sezione
pub fn merge_adjacent_comments(src: &str, comments: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if comments.is_empty() {
        return vec![];
    }
    let mut merged = Vec::new();
    let mut cur_start = comments[0].0;
    let mut cur_end = comments[0].1;
    for window in comments.windows(2) {
        let (prev_start, prev_end) = window[0];
        let (next_start, next_end) = window[1];
        let prev_is_single = is_single_line_comment(src, (prev_start, prev_end));
        let next_is_single = is_single_line_comment(src, (next_start, next_end));
        let between = &src[prev_end..next_start];
        // Merge solo se entrambi sono single-line, consecutivi e tra di loro solo whitespace
        if prev_is_single && next_is_single && between.trim().is_empty() {
            cur_end = next_end;
        } else {
            merged.push((cur_start, cur_end));
            cur_start = next_start;
            cur_end = next_end;
        }
    }
    merged.push((cur_start, cur_end));
    merged
}

pub fn is_single_line_comment(src: &str, (start, end): (usize, usize)) -> bool {
    let text = &src[start..end];
    let lines: Vec<&str> = text.lines().collect();
    lines.len() == 1
}
use tree_sitter::Node;

/// Estrae tutte le sezioni di commento dal nodo root
pub fn collect_comments(node: Node, out: &mut Vec<(usize, usize)>) {
    if node.kind() == "comment" {
        out.push((node.start_byte(), node.end_byte()));
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_comments(child, out);
        }
    }
}

/// Converte un offset di byte in (riga, colonna) (0-based)
pub fn offset_to_linecol(src: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    let mut count = 0;
    for c in src.chars() {
        if count >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        count += c.len_utf8();
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::{is_code_comment, merge_adjacent_comments};
    use crate::utils::comment_utils::is_single_line_comment;

    #[test]
    fn test_merge_adjacent_single_line() {
        let src = "// a\n// b\nint x;\n// c\n// d\n/* multi\nline */\n// e\n";
        // commenti: [(0,5),(6,10),(18,22),(23,27),(24,44),(45,49)]
        let comments = vec![(0, 5), (6, 10), (18, 22), (23, 27), (28 ,44), (45, 49)];
        let merged = merge_adjacent_comments(src, &comments);
        // I primi due e i successivi due single-line vanno uniti, il multi-line resta separato
        assert_eq!(merged, vec![(0, 10), (18, 27), (30, 44), (45, 50)]);
    }

    #[test]
    fn test_merge_adjacent_with_code_between() {
        let src = "// a\nint x;\n// b\n";
        let comments = vec![(0, 5), (12, 17)];
        let merged = merge_adjacent_comments(src, &comments);
        // Non devono essere uniti perché c'è codice tra loro
        assert_eq!(merged, comments);
    }

    #[test]
    fn test_merge_adjacent_multiline() {
        let src = "/* multi\nline */\n// a\n// b\n";
        let comments = vec![(0, 14), (15, 20), (21, 26)];
        let merged = merge_adjacent_comments(src, &comments);
        // Solo i due single-line vanno uniti
        assert_eq!(merged, vec![(0, 14), (15, 26)]);
    }

    #[test]
    fn test_is_single_line_comment() {
        let src = "// a\n/* multi\nline */\n";
        assert!(is_single_line_comment(src, (0, 5)));
        assert!(!is_single_line_comment(src, (6, 20)));
    }
    #[test]
    fn test_is_code_comment_basic() {
        let c = "// int a = 0;\n// printf(\"hello\");\n// just text";

        assert!(is_code_comment(c, 50)); // 2/3 righe sono codice
        assert!(!is_code_comment(c, 80));
    }
    #[test]
    fn test_is_code_comment_empty() {
        assert!(!is_code_comment("", 10));
    }
    #[test]
    fn test_is_code_comment_define() {
        let c = "/*\n#define X 10\n#define Y 20\n*/";
        assert!(is_code_comment(c, 50));
    }
    #[test]
    fn test_is_code_comment_text() {
        let c = "/*\nQuesto è solo testo\nancora testo\n*/";
        assert!(!is_code_comment(c, 10));
    }
}

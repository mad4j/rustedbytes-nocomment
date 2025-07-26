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

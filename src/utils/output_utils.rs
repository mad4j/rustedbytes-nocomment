use std::path::PathBuf;
use owo_colors::OwoColorize;
use owo_colors::colors::{Green, BrightBlack};
use crate::utils::comment_utils::offset_to_linecol;

/// Stampa il file con i commenti evidenziati in verde e il resto in grigio scuro
pub fn print_colored(src: &str, comments: &[(usize, usize)]) {
    let mut idx = 0;
    let mut cidx = 0;
    let mut in_comment = false;
    let mut out = String::new();
    while idx < src.len() {
        if cidx < comments.len() && idx == comments[cidx].0 {
            in_comment = true;
        }
        if cidx < comments.len() && idx == comments[cidx].1 {
            in_comment = false;
            cidx += 1;
        }
        let ch = src[idx..].chars().next().unwrap();
        let s = ch.to_string();
        if in_comment {
            out.push_str(&s.fg::<Green>().to_string());
        } else {
            out.push_str(&s.fg::<BrightBlack>().to_string());
        }
        idx += ch.len_utf8();
    }
    print!("{}", out);
}

/// Stampa la lista delle sezioni di commento
pub fn print_comment_sections(
    src: &str,
    comments: &[(usize, usize)],
    linecol: bool,
    file: &PathBuf,
) {
    println!("File: {}", file.display());
    for (start, end) in comments {
        if linecol {
            let (sl, sc) = offset_to_linecol(src, *start);
            let (el, ec) = offset_to_linecol(src, *end);
            println!("({}:{})-({}:{})", sl + 1, sc + 1, el + 1, ec + 1);
        } else {
            println!("{}-{}", start, end);
        }
    }
}

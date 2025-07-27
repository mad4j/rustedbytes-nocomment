use clap::Parser;
use std::fs;
use std::path::PathBuf;
use tree_sitter::Parser as TsParser;
mod utils {
    pub mod comment_utils;
    pub mod output_utils;
}
use utils::comment_utils::{collect_comments, is_code_comment, merge_adjacent_comments};
use utils::output_utils::{print_colored, print_comment_sections};

/// CLI per estrarre sezioni di commento da file C/C++
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File da analizzare (C/C++ source o header)
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Fornisci output come (riga:carattere),(riga:carattere) invece che come offset carattere
    #[arg(long)]
    linecol: bool,
    /// Forza il parser C++ (default: auto)
    #[arg(long)]
    cpp: bool,
    /// Stampa il file con i commenti evidenziati in verde e il resto in grigio scuro
    #[arg(long)]
    color_output: bool,
    /// Analizza e indica quali commenti contengono codice sorgente
    #[arg(long)]
    detect_code_comments: bool,
    /// Percentuale minima di codice per considerare un commento come "contenente codice"
    #[arg(long, default_value_t = 40)]
    code_threshold: u8,
}

fn main() {
    let args = Args::parse();
    for file in &args.files {
        let Ok(src) = fs::read_to_string(file) else {
            eprintln!("Impossibile leggere il file: {}", file.display());
            continue;
        };
        let lang = if args.cpp
            || file.extension().map_or(false, |e| {
                e == "cpp" || e == "hpp" || e == "cc" || e == "cxx"
            }) {
            tree_sitter_cpp::LANGUAGE
        } else {
            tree_sitter_c::LANGUAGE
        };
        let mut parser = TsParser::new();
        parser
            .set_language(&lang.into())
            .expect("Impossibile impostare il linguaggio");
        let Some(tree) = parser.parse(&src, None) else {
            eprintln!("Parsing fallito per: {}", file.display());
            continue;
        };
        let mut comments = Vec::new();
        let root = tree.root_node();
        collect_comments(root, &mut comments);
        let merged_comments = merge_adjacent_comments(&src, &comments);
        if args.detect_code_comments {
            println!("File: {}", file.display());
            for (start, end) in &merged_comments {
                let comment_text = &src[*start..*end];
                let is_code = is_code_comment(comment_text, args.code_threshold);
                if args.linecol {
                    let (sl, sc) = utils::comment_utils::offset_to_linecol(&src, *start);
                    let (el, ec) = utils::comment_utils::offset_to_linecol(&src, *end);
                    println!(
                        "({}:{})-({}:{}) {}",
                        sl + 1,
                        sc + 1,
                        el + 1,
                        ec + 1,
                        if is_code { "[CODE]" } else { "" }
                    );
                } else {
                    println!("{}-{} {}", start, end, if is_code { "[CODE]" } else { "" });
                }
            }
        } else if args.color_output {
            print_colored(&src, &merged_comments);
        } else {
            print_comment_sections(&src, &merged_comments, args.linecol, &file);
        }
    }
}

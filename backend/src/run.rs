use crate::backend;
use crate::pbn;
use crate::syntax;

use chumsky::Parser;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

fn parse_error(
    title: &str,
    code: i32,
    filename: &'static str,
    src: &'static str,
    err: &chumsky::error::Simple<char>,
) {
    use ariadne::*;

    let err_span = err.span();
    let err_expected = err
        .expected()
        .filter_map(|mtok| mtok.map(|tok| format!("`{}`", tok)))
        .collect::<Vec<_>>();

    let error_color = Color::Red;

    let mut report = Report::build(ReportKind::Error, filename, err_span.start)
        .with_code(code)
        .with_message(title)
        .with_label(
            Label::new((filename, err_span))
                .with_message(
                    format!("{}", "Unexpected token".fg(error_color),),
                )
                .with_color(error_color),
        );

    if !err_expected.is_empty() {
        report = report.with_note(format!(
            "{}{}",
            if err_expected.len() == 1 {
                format!("Expected {}", err_expected[0])
            } else {
                format!("Expected one of {}", err_expected.join(", "))
            },
            match err.found() {
                Some(tok) => format!(", but found `{}`", tok),
                None => "".to_owned(),
            }
        ));
    }

    report
        .finish()
        .eprint(sources(vec![(filename, src)]))
        .unwrap();
}

pub fn run(
    lib_filename: &PathBuf,
    imp_filename: &PathBuf,
    prog_filename: &PathBuf,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let lib_src = std::fs::read_to_string(lib_filename).unwrap();
    let imp_src = std::fs::read_to_string(imp_filename).unwrap();
    let prog_src = std::fs::read_to_string(prog_filename).unwrap();

    let lib = match syntax::parse::library().parse(lib_src.to_owned()) {
        Ok(lib) => lib,
        Err(errs) => {
            parse_error(
                "Library parse error",
                0,
                lib_filename.display().to_string().leak(),
                lib_src.leak(),
                &errs[0],
            );
            return Err("Library parse error".into());
        }
    };

    let prog = match syntax::parse::program().parse(prog_src.to_owned()) {
        Ok(prog) => prog,
        Err(errs) => {
            parse_error(
                "Program parse error",
                1,
                prog_filename.display().to_string().leak(),
                prog_src.leak(),
                &errs[0],
            );
            return Err("Program parse error".into());
        }
    };

    match pbn::run(&lib, &prog, true) {
        Some(tree) => {
            println!(
                "\n{}",
                ansi_term::Color::Cyan.bold().paint("[ All done! ]")
            );
            println!(
                "\n{}",
                backend::Python::new(&tree).emit().plain_text(&imp_src)
            );
            if json {
                let json_filename = prog_filename.with_extension("json");
                let mut json_file = File::create(json_filename)?;
                write!(
                    json_file,
                    "{}",
                    serde_json::to_string_pretty(&tree).unwrap()
                )?;
            }
        }
        None => {
            println!(
                "{}",
                ansi_term::Color::Red.bold().paint("[ Not possible! ]")
            );
        }
    }

    Ok(())
}

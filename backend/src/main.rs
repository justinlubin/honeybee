mod analysis;
mod backend;
mod derivation;
mod egglog_adapter;
mod hybrid_oracle;
mod ir;
mod pbn;
mod syntax;
mod synthesis;
mod top_level;

use chumsky::Parser;
use std::fs::File;
use std::io::prelude::*;

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

fn main() -> std::io::Result<()> {
    env_logger::init();

    if std::env::args().len() != 5 {
        println!(
            "usage: {} <library> <implementation> <program> <output>",
            std::env::args().nth(0).unwrap()
        );
        std::process::exit(1)
    }

    // Library signature

    let lib_filename = &*std::env::args().nth(1).unwrap().leak();
    let lib_src = &*std::fs::read_to_string(lib_filename).unwrap().leak();

    let lib = match syntax::parse::library().parse(lib_src.to_owned()) {
        Ok(lib) => lib,
        Err(errs) => {
            parse_error(
                "Library parse error",
                0,
                lib_filename,
                lib_src,
                &errs[0],
            );
            std::process::exit(1);
        }
    };

    // Library implementation

    let imp_filename = &*std::env::args().nth(2).unwrap().leak();
    let imp_src = &*std::fs::read_to_string(imp_filename).unwrap().leak();

    // Program

    let prog_filename = &*std::env::args().nth(3).unwrap().leak();
    let prog_src = &*std::fs::read_to_string(prog_filename).unwrap().leak();

    let prog = match syntax::parse::program().parse(prog_src.to_owned()) {
        Ok(prog) => prog,
        Err(errs) => {
            parse_error(
                "Program parse error",
                1,
                prog_filename,
                prog_src,
                &errs[0],
            );
            return Ok(());
        }
    };

    // Output path

    let output_filename = std::env::args().nth(4).unwrap();

    // Main

    let runner = top_level::Runner { interactive: true };

    match runner.run(lib, imp_src, prog) {
        Some(output) => {
            let mut output_file = File::create(output_filename)?;
            write!(output_file, "{}", output)?;
            if runner.interactive {
                println!(
                    "\n{}",
                    ansi_term::Color::Cyan.bold().paint("[ All done! ]")
                );
            }
        }
        None => {
            println!(
                "{}",
                ansi_term::Color::Red.bold().paint("[ Not possible! ]")
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

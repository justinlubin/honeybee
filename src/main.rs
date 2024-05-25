mod analysis;
mod backend;
mod derivation;
mod egglog_adapter;
mod ir;
mod syntax;
mod synthesis;

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

    let mut colors = ColorGenerator::new();

    let error_color = Color::Red;

    let mut report = Report::build(ReportKind::Error, filename, err_span.start)
        .with_code(1)
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
            return Ok(());
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

    if !egglog_adapter::check_possible(&lib, &prog) {
        println!(">>> Not possible <<<");
        return Ok(());
    }

    let mut synthesizer = synthesis::Synthesizer::new(&lib, &prog);
    let analyzer = analysis::CLI {
        mode: analysis::CLIMode::Auto,
        print: false,
    };

    let mut step = 1;
    loop {
        println!(
            "{} {} {}\n\n{}",
            ansi_term::Color::Fixed(8).paint("═".repeat(2)),
            ansi_term::Color::Fixed(8).paint(format!("Step {}", step)),
            ansi_term::Color::Fixed(8).paint("═".repeat(40)),
            ansi_term::Style::new().bold().paint("Derivation tree:")
        );
        print!("{}", synthesizer.tree.pretty());
        let options = synthesizer.options();
        if options.is_empty() {
            break;
        }
        println!();
        let choice = analyzer.analyze(options);
        synthesizer.step(&choice);
        step += 1;
    }

    let nb = backend::Python::new(&synthesizer.tree)
        .emit()
        .nbformat(&imp_src);

    let mut output_file = File::create(output_filename)?;
    write!(output_file, "{}", nb)?;

    println!(
        "\n{}{}\n",
        " ".repeat(20),
        ansi_term::Color::Cyan.paint("All done!")
    );

    Ok(())
}

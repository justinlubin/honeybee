use crate::analysis;
use crate::derivation;
use crate::egglog_adapter;
use crate::ir::*;
use crate::synthesis;

pub fn run(
    lib: &Library,
    prog: &Program,
    interactive: bool,
) -> Option<derivation::Tree> {
    if !egglog_adapter::check_possible(lib, prog) {
        return None;
    }

    let mut synthesizer = synthesis::Synthesizer::new(lib, prog);
    let analyzer = if interactive {
        analysis::CLI {
            // mode: analysis::CLIMode::FastForward,
            mode: analysis::CLIMode::Manual,
            print: true,
        }
    } else {
        analysis::CLI {
            mode: analysis::CLIMode::Auto,
            print: false,
        }
    };

    let mut step = 1;
    loop {
        if interactive {
            println!(
                "{} {} {}\n\n{}",
                ansi_term::Color::Fixed(8).paint("═".repeat(2)),
                ansi_term::Color::Fixed(8).paint(format!("Step {}", step)),
                ansi_term::Color::Fixed(8).paint("═".repeat(40)),
                ansi_term::Style::new().bold().paint("Derivation tree:")
            );
            print!("{}", synthesizer.tree.pretty());
        }
        let options = synthesizer.options();
        if options.is_empty() {
            break;
        }
        if interactive {
            println!();
        }
        let choice = analyzer.analyze(options);
        synthesizer.step(&choice);
        step += 1;
    }
    Some(synthesizer.tree)
}

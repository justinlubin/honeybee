use crate::analysis;
use crate::backend;
use crate::egglog_adapter;
use crate::synthesis;

use crate::ir::*;

pub struct Runner {
    pub interactive: bool,
}

impl Runner {
    pub fn run(
        &self,
        lib: Library,
        imp_src: &str,
        prog: Program,
    ) -> Option<String> {
        if !egglog_adapter::check_possible(&lib, &prog) {
            return None;
        }

        let mut synthesizer = synthesis::Synthesizer::new(&lib, &prog);
        let analyzer = if self.interactive {
            analysis::CLI {
                mode: analysis::CLIMode::FastForward,
                // mode: analysis::CLIMode::Manual,
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
            if self.interactive {
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
            if self.interactive {
                println!();
            }
            let choice = analyzer.analyze(options);
            synthesizer.step(&choice);
            step += 1;
        }

        return Some(
            backend::Python::new(&synthesizer.tree)
                .emit()
                .nbformat(&imp_src),
        );
    }
}

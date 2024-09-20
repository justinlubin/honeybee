use crate::derivation;
use crate::syntax;
use crate::synthesis;

use crate::ir::*;

#[derive(Debug, Clone)]
pub enum CLIMode {
    Manual,
    FastForward,
    Auto,
}

pub struct CLI {
    pub mode: CLIMode,
    pub print: bool,
}

impl CLI {
    fn select<T>(&self, title: &str, mut options: Vec<(String, T)>) -> T {
        use std::io::Write;

        if options.is_empty() {
            panic!("Empty options");
        }

        let auto = match self.mode {
            CLIMode::Auto => true,
            CLIMode::FastForward => options.len() == 1,
            CLIMode::Manual => false,
        };

        if self.print {
            println!("{}", title);
            for (i, (label, _)) in options.iter().enumerate() {
                println!(
                    "  {}) {}{}",
                    i,
                    label,
                    if auto && i == 0 {
                        ansi_term::Color::Red
                            .paint(" (auto-selected)")
                            .to_string()
                    } else {
                        "".to_owned()
                    }
                );
            }
        }

        loop {
            if !auto {
                if self.print {
                    print!("> ");
                }
                let _ = std::io::stdout().flush();
            }

            let idx = if auto {
                0
            } else {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                if input == "q" {
                    std::process::exit(1);
                }
                match input.trim().parse::<usize>() {
                    Ok(idx) => idx,
                    Err(_) => continue,
                }
            };

            if idx >= options.len() {
                continue;
            }

            if self.print {
                println!();
            }

            return options.swap_remove(idx).1;
        }
    }

    fn select_assignment(
        &self,
        path: &Vec<String>,
        assignment_options: Vec<Assignment>,
    ) -> Assignment {
        use ansi_term::Color::*;
        use ansi_term::Style;

        self.select(
            &format!(
                "{}{}{}",
                &Style::new().bold().paint("Assignments"),
                if path.is_empty() {
                    " (on root)".to_owned()
                } else {
                    format!(" (on {})", Blue.paint(path.join(".")))
                },
                &Style::new().bold().paint(":"),
            ),
            assignment_options
                .into_iter()
                .map(|a| {
                    (Cyan.paint(syntax::unparse::assignment(&a)).to_string(), a)
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn analyze(
        &self,
        options: Vec<synthesis::GoalOption>,
    ) -> synthesis::Choice {
        use ansi_term::Color::*;
        use ansi_term::Style;

        let goal_option = self.select(
            &format!(
                "{} {}",
                Style::new().bold().paint("Goals:"),
                Fixed(8).paint("('q' to quit)"),
            ),
            options
                .into_iter()
                .map(|opt| match &opt {
                    synthesis::GoalOption::Output { path, tag, .. }
                    | synthesis::GoalOption::Input { path, tag, .. } => (
                        Yellow
                            .paint(
                                path.iter()
                                    .map(|pe| &pe.tag)
                                    .chain(std::iter::once(tag))
                                    .map(|id| id.clone())
                                    .collect::<Vec<_>>()
                                    .join("."),
                            )
                            .to_string(),
                        opt,
                    ),
                })
                .collect::<Vec<_>>(),
        );

        match goal_option {
            synthesis::GoalOption::Output {
                path,
                tag,
                computation_options,
            } => {
                let computations = derivation::computations(&path);
                let synthesis::ComputationOption {
                    name,
                    assignment_options,
                } = self.select(
                    &Style::new().bold().paint("Computations:").to_string(),
                    computation_options
                        .into_iter()
                        // .filter(|c| !computations.contains(&c.name))
                        .map(|c| (Green.paint(c.name.clone()).to_string(), c))
                        .collect::<Vec<_>>(),
                );

                let path = derivation::into_tags(path);

                let assignment =
                    self.select_assignment(&path, assignment_options);

                synthesis::Choice::Output {
                    path,
                    tag,
                    computation_name: name,
                    assignment,
                }
            }
            synthesis::GoalOption::Input {
                path,
                tag,
                fact_name,
                assignment_options,
            } => {
                let path = derivation::into_tags(path);
                synthesis::Choice::Input {
                    tag,
                    fact_name,
                    assignment: self
                        .select_assignment(&path, assignment_options),
                    path,
                }
            }
        }
    }
}

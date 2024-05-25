use crate::synthesis;

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
                match input.trim().parse::<usize>() {
                    Ok(idx) => idx,
                    Err(_) => continue,
                }
            };

            if idx > options.len() {
                continue;
            }

            if self.print {
                println!();
            }

            return options.swap_remove(idx).1;
        }
    }

    pub fn analyze(
        &self,
        options: Vec<synthesis::GoalOption>,
    ) -> synthesis::Choice {
        use ansi_term::Color::*;
        use ansi_term::Style;

        let goal_option = self.select(
            &Style::new().bold().paint("Goals:").to_string(),
            options
                .into_iter()
                .map(|opt| match &opt {
                    synthesis::GoalOption::Analysis { path, tag, .. }
                    | synthesis::GoalOption::Annotation { path, tag, .. } => (
                        Yellow
                            .paint(
                                path.iter()
                                    .chain(std::iter::once(tag))
                                    .map(|id| format!(".{}", id))
                                    .collect::<Vec<_>>()
                                    .join(""),
                            )
                            .to_string(),
                        opt,
                    ),
                })
                .collect::<Vec<_>>(),
        );

        match goal_option {
            synthesis::GoalOption::Analysis {
                path,
                tag,
                computation_options,
            } => {
                let computation_option = self.select(
                    &Style::new().bold().paint("Computations:").to_string(),
                    computation_options
                        .into_iter()
                        .map(|c| (Green.paint(c.name.clone()).to_string(), c))
                        .collect::<Vec<_>>(),
                );

                synthesis::Choice::Analysis {
                    path,
                    tag,
                    computation_name: computation_option.name,
                    assignment: computation_option
                        .assignment_options
                        .into_iter()
                        .next()
                        .unwrap(),
                }
            }
            synthesis::GoalOption::Annotation {
                path,
                tag,
                fact_name,
                assignment_options,
            } => synthesis::Choice::Annotation {
                path,
                tag,
                fact_name,
                assignment: assignment_options.into_iter().next().unwrap(),
            },
        }
    }
}

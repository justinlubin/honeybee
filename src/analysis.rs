use crate::synthesis;

enum Mode {
    Manual,
    FastForward,
    Auto,
}

fn select_cli<T>(mode: &Mode, title: &str, mut options: Vec<(String, T)>) -> T {
    use std::io::Write;

    if options.is_empty() {
        panic!("Empty options");
    }

    let auto = match mode {
        Mode::Auto => true,
        Mode::FastForward => options.len() == 1,
        Mode::Manual => false,
    };

    println!("{}", title);
    for (i, (label, _)) in options.iter().enumerate() {
        println!(
            "  {}) {}{}",
            i,
            label,
            if auto && i == 0 {
                " (auto-selected)"
            } else {
                ""
            }
        );
    }

    loop {
        if !auto {
            print!("> ");
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

        println!();

        return options.swap_remove(idx).1;
    }
}

fn cli(mode: &Mode, options: Vec<synthesis::GoalOption>) -> synthesis::Choice {
    use ansi_term::Color::*;
    use ansi_term::Style;

    let goal_option = select_cli(
        mode,
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
            let computation_option = select_cli(
                mode,
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

pub fn manual(options: Vec<synthesis::GoalOption>) -> synthesis::Choice {
    cli(&Mode::Manual, options)
}

pub fn fast_forward(options: Vec<synthesis::GoalOption>) -> synthesis::Choice {
    cli(&Mode::FastForward, options)
}

pub fn auto(options: Vec<synthesis::GoalOption>) -> synthesis::Choice {
    cli(&Mode::Auto, options)
}

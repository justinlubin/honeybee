use crate::synthesis;

pub fn auto(options: Vec<synthesis::GoalOption>) -> synthesis::Choice {
    match options.into_iter().next().unwrap() {
        synthesis::GoalOption::Analysis {
            path,
            tag,
            computation_options,
        } => {
            let computation_option =
                computation_options.into_iter().next().unwrap();
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

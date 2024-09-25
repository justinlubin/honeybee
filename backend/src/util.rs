use std::time::Instant;

pub fn timed_cartesian_product<T: Clone>(
    choices_sequence: Vec<Vec<T>>,
    soft_timeout: u128,
) -> Option<Vec<Vec<T>>> {
    let now = Instant::now();
    let mut result = vec![vec![]];
    for choices in choices_sequence {
        let mut new_result = vec![];
        for choice in choices {
            if now.elapsed().as_millis() > soft_timeout {
                return None;
            }
            for mut r in result.clone() {
                r.push(choice.clone());
                new_result.push(r);
            }
        }
        result = new_result;
    }
    Some(result)
}

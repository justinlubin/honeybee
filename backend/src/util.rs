pub fn cartesian_product<T: Clone>(
    choices_sequence: Vec<Vec<T>>,
) -> Vec<Vec<T>> {
    let mut result = vec![vec![]];
    for choices in choices_sequence {
        let mut new_result = vec![];
        for choice in choices {
            for mut r in result.clone() {
                r.push(choice.clone());
                new_result.push(r);
            }
        }
        result = new_result;
    }
    result
}

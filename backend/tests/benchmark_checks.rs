use honeybee::*;

use std::collections::HashMap;

#[test]
fn generic_counts_match() {
    let records =
        benchmark::run(&"../compendium/suites/generic/".into(), 1, 5000, true)
            .unwrap();

    let mut h: HashMap<
        String,
        (
            (benchmark::Algorithm, usize),
            Vec<(benchmark::Algorithm, usize)>,
        ),
    > = HashMap::new();

    for r in records {
        if r.completed && r.task == "All" {
            let data = (r.algorithm.clone(), r.solution_count);
            match h.get_mut(&r.entry) {
                Some((_, vec)) => vec.push(data),
                None => {
                    h.insert(r.entry, (data, vec![]));
                    ()
                }
            }
        }
    }

    for (task, ((a, c), es)) in h {
        for (aa, cc) in es {
            assert_eq!(
                c, cc,
                "Algorithms '{:?}' and '{:?}' got different results for task '{}'",
                a, aa, task
            );
        }
    }
}

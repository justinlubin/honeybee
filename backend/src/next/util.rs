use crate::next::timer::*;

use indexmap::IndexMap;

pub fn cartesian_product<E, K: Clone + Eq + std::hash::Hash, T: Clone>(
    timer: &impl Timer<E>,
    choices: IndexMap<K, Vec<T>>,
) -> Result<Vec<IndexMap<K, T>>, E> {
    let mut results = vec![IndexMap::new()];
    for (k, vs) in choices.iter() {
        let mut new_results = vec![];
        for map in results {
            timer.tick()?;
            for v in vs {
                let mut new_map = map.clone();
                new_map.insert(k.clone(), v.clone());
                new_results.push(new_map)
            }
        }
        results = new_results;
    }
    Ok(results)
}

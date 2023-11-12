use crate::data::Portfolio;
use ndarray::Array1;
use std::collections::HashMap;

type Array = Array1<f64>;

fn zero_where_neg((x, a): (&mut f64, f64)) {
    if a < 0. {
        *x = 0.;
    }
}
fn solve_impl(mut current: Array, new_budget: f64, mut shares: Array) -> Array {
    loop {
        let current_tot = current.sum();
        let alloc = shares.clone() * (current_tot + new_budget) - current.clone();
        if alloc.iter().all(|&a| a >= 0.) {
            return alloc;
        }
        current
            .iter_mut()
            .zip(alloc.clone())
            .for_each(zero_where_neg);
        shares
            .iter_mut()
            .zip(alloc.clone())
            .for_each(zero_where_neg);
        let shares_sum = shares.sum();
        shares /= shares_sum;
    }
}

pub fn solve(p: &Portfolio, new_budget: f64) -> Vec<u32> {
    let mut ret = vec![0; p.securities.len()];
    let mut values = HashMap::new();
    for s in &p.securities {
        *values.entry(s.category.as_str()).or_insert(0.0) += s.amount as f64 * s.latest_value;
    }
    dbg!(values.clone());
    let shares = Array::from_iter(p.categories.iter().map(|c| c.target_percentage));
    let current = Array::from_iter(
        p.categories
            .iter()
            .map(|c| *values.get(c.name.as_str()).unwrap()),
    );
    let new_amounts = solve_impl(current, new_budget, shares);

    // now solve each category
    for (c, &a) in p.categories.iter().zip(new_amounts.iter()) {
        let new_budget = a;
        let filtered = p.securities.iter().filter(|s| s.category == c.name);
        let shares = Array::from_iter(filtered.clone().map(|s| s.target_percentage));
        let current = Array::from_iter(filtered.clone().map(|s| s.amount as f64 * s.latest_value));
        let new_amounts = solve_impl(current, new_budget, shares);
        ret.iter_mut()
            .zip(p.securities.iter())
            .filter(|(_, s)| s.category == c.name)
            .zip(new_amounts)
            .for_each(|((r, s), a)| *r = (a / s.latest_value).floor() as u32);
    }
    ret
}

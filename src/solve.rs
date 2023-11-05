use ndarray::Array1;

pub type Array = Array1<f64>;

fn zero_where_neg((x, a): (&mut f64, f64)) {
    if a < 0. {
        *x = 0.;
    }
}
pub fn solve(mut current: Array, new_budget: f64, mut shares: Array) -> Array {
    loop {
        let current_tot = current.sum();
        let alloc = shares.clone()*(current_tot+new_budget) - current.clone();
        if alloc.iter().all(|&a| a >= 0.) {
            return alloc;
        }
        current.iter_mut().zip(alloc.clone()).for_each(zero_where_neg);
        shares.iter_mut().zip(alloc.clone()).for_each(zero_where_neg);
        let shares_sum = shares.sum();
        shares = shares / shares_sum;
    }
}

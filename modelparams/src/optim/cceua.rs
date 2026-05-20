use rand::Rng;

/// Competitive Complex Evolution step.
/// Returns (best_x, best_f, n_evals).
pub fn cceua<F>(
    f: &F,
    s: &mut Vec<Vec<f64>>,   // simplex points [size_simplex × n_param]
    sf: &mut Vec<f64>,        // simplex costs
    bl: &[f64],
    bu: &[f64],
    rng: &mut impl Rng,
) -> (Vec<f64>, f64, usize)
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    let n_param = bl.len();
    let size_simplex = s.len();
    let mut n_evals = 0usize;

    let beta = 0.5_f64;

    let worst_idx = size_simplex - 1;

    // centroid of all points except the worst
    let mut ce = vec![0.0_f64; n_param];
    for i in 0..worst_idx {
        for j in 0..n_param {
            ce[j] += s[i][j];
        }
    }
    for j in 0..n_param { ce[j] /= worst_idx as f64; }

    // reflection
    let mut snew: Vec<f64> = (0..n_param)
        .map(|j| (2.0 * ce[j] - s[worst_idx][j]).clamp(bl[j], bu[j]))
        .collect();
    let fnew = f(&snew);
    n_evals += 1;

    if fnew < sf[worst_idx] {
        s[worst_idx] = snew;
        sf[worst_idx] = fnew;
        sort_simplex(s, sf);
        return (s[0].clone(), sf[0], n_evals);
    }

    // contraction
    snew = (0..n_param)
        .map(|j| (beta * s[worst_idx][j] + (1.0 - beta) * ce[j]).clamp(bl[j], bu[j]))
        .collect();
    let fnew = f(&snew);
    n_evals += 1;

    if fnew < sf[worst_idx] {
        s[worst_idx] = snew;
        sf[worst_idx] = fnew;
        sort_simplex(s, sf);
        return (s[0].clone(), sf[0], n_evals);
    }

    // random point in feasible space
    snew = (0..n_param)
        .map(|j| rng.gen_range(bl[j]..=bu[j]))
        .collect();
    let fnew = f(&snew);
    n_evals += 1;

    s[worst_idx] = snew;
    sf[worst_idx] = fnew;
    sort_simplex(s, sf);

    (s[0].clone(), sf[0], n_evals)
}

pub fn sort_simplex(s: &mut Vec<Vec<f64>>, sf: &mut Vec<f64>) {
    let mut indices: Vec<usize> = (0..sf.len()).collect();
    indices.sort_by(|&a, &b| sf[a].partial_cmp(&sf[b]).unwrap());
    let s_sorted: Vec<Vec<f64>> = indices.iter().map(|&i| s[i].clone()).collect();
    let sf_sorted: Vec<f64> = indices.iter().map(|&i| sf[i]).collect();
    *s = s_sorted;
    *sf = sf_sorted;
}

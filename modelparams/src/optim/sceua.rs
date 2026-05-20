use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rayon::prelude::*;
use super::cceua::{cceua, sort_simplex};

#[derive(Debug, Clone, PartialEq)]
pub enum ReturnCode {
    MaxIters,
    Success,
    Stalled,
    Failure,
}

#[derive(Debug, Clone)]
pub struct SceOptions {
    pub max_evals: usize,
    pub n_complex: usize,
    pub size_complex: usize,  // 0 = auto: 2*n_param+1
    pub size_simplex: usize,  // 0 = auto: n_param+1
    pub n_evolu: usize,       // 0 = auto: size_complex
    pub kstop: usize,
    pub f_reltol: f64,
    pub x_reltol: f64,
    pub seed: u64,
    pub verbose: bool,
    pub parallel: bool,
}

impl SceOptions {
    pub fn new(n_param: usize) -> Self {
        let size_complex = 2 * n_param + 1;
        Self {
            max_evals: 1000,
            n_complex: 5,
            size_complex,
            size_simplex: n_param + 1,
            n_evolu: size_complex,
            kstop: 5,
            f_reltol: 1e-4,
            x_reltol: 1e-4,
            seed: 1,
            verbose: true,
            parallel: true,
        }
    }
}

#[derive(Debug)]
pub struct SceResult {
    pub best_x: Vec<f64>,
    pub best_f: f64,
    pub n_evals: usize,
    pub code: ReturnCode,
}

fn rng_seed(seed: u64, nloop: u64, igs: u64, loop_: u64) -> u64 {
    seed.wrapping_mul(1_000_003)
        .wrapping_add(nloop.wrapping_mul(997))
        .wrapping_add(igs.wrapping_mul(101))
        .wrapping_add(loop_)
}

fn geometric_range(x: &[Vec<f64>], bound: &[f64]) -> f64 {
    let n_param = bound.len();
    let mut gr = 0.0_f64;
    for j in 0..n_param {
        if bound[j] == 0.0 { continue; }
        let col_max = x.iter().map(|row| row[j]).fold(f64::NEG_INFINITY, f64::max);
        let col_min = x.iter().map(|row| row[j]).fold(f64::INFINITY, f64::min);
        gr += ((col_max - col_min) / bound[j]).ln();
    }
    (gr / n_param as f64).exp()
}

/// Shuffled Complex Evolution – University of Arizona
pub fn sceua<F>(
    f: F,
    x0: &[f64],
    bl: &[f64],
    bu: &[f64],
    opts: SceOptions,
) -> SceResult
where
    F: Fn(&[f64]) -> f64 + Sync + Send,
{
    let n_param = x0.len();
    assert_eq!(bl.len(), n_param);
    assert_eq!(bu.len(), n_param);

    let size_complex  = if opts.size_complex  == 0 { 2 * n_param + 1 } else { opts.size_complex };
    let size_simplex  = if opts.size_simplex  == 0 { n_param + 1     } else { opts.size_simplex };
    let n_evolu       = if opts.n_evolu       == 0 { size_complex     } else { opts.n_evolu };
    let n_complex     = opts.n_complex;
    let n_pop = size_complex * n_complex;

    let bound: Vec<f64> = bl.iter().zip(bu).map(|(l, u)| u - l).collect();

    // initial population
    let mut main_rng = SmallRng::seed_from_u64(rng_seed(opts.seed, 0, 0, 0));
    let mut x: Vec<Vec<f64>> = (0..n_pop)
        .map(|_| (0..n_param).map(|j| bl[j] + main_rng.gen::<f64>() * bound[j]).collect())
        .collect();
    x[0] = x0.to_vec();

    let sanitize = |v: f64| if v.is_finite() { v } else { f64::MAX };

    let mut xf: Vec<f64> = if opts.parallel {
        x.par_iter().map(|xi| sanitize(f(xi))).collect()
    } else {
        x.iter().map(|xi| sanitize(f(xi))).collect()
    };

    let mut n_evals = n_pop;
    sort_pop(&mut x, &mut xf);

    let mut gnrng = geometric_range(&x, &bound);
    let mut criter: Vec<f64> = Vec::new();
    let mut criter_change = 1e5_f64;
    let mut nloop = 0usize;

    if opts.verbose {
        println!("Iteration = {:3}, nEvals = {:5}, Best Cost = {:.5}", nloop, n_evals, xf[0]);
    }

    while n_evals < opts.max_evals && gnrng > opts.x_reltol && criter_change > opts.f_reltol {
        nloop += 1;

        // evolve each complex
        let complex_results: Vec<(Vec<Vec<f64>>, Vec<f64>, usize)> = if opts.parallel {
            (0..n_complex).into_par_iter().map(|igs| {
                evolve_complex(&f, &x, &xf, igs, n_complex, size_complex, size_simplex,
                               n_evolu, bl, bu, opts.seed, nloop as u64)
            }).collect()
        } else {
            (0..n_complex).map(|igs| {
                evolve_complex(&f, &x, &xf, igs, n_complex, size_complex, size_simplex,
                               n_evolu, bl, bu, opts.seed, nloop as u64)
            }).collect()
        };

        for (igs, (cx, cf, evals)) in complex_results.into_iter().enumerate() {
            let k2: Vec<usize> = (0..size_complex).map(|k| k * n_complex + igs).collect();
            for (k, &pop_i) in k2.iter().enumerate() {
                x[pop_i] = cx[k].clone();
                xf[pop_i] = cf[k];
            }
            n_evals += evals;
        }

        sort_pop(&mut x, &mut xf);
        gnrng = geometric_range(&x, &bound);

        criter.push(xf[0]);
        if nloop >= opts.kstop {
            let window = &criter[nloop - opts.kstop..=nloop - 1];
            let mean_abs: f64 = window.iter().map(|v| v.abs()).sum::<f64>() / opts.kstop as f64;
            criter_change = (criter[nloop - 1] - criter[nloop - opts.kstop]).abs() / mean_abs;
        }

        if opts.verbose {
            println!("Iteration = {:3}, nEvals = {:5}, Best Cost = {:.5}", nloop, n_evals, xf[0]);
        }
    }

    let code = if n_evals >= opts.max_evals {
        ReturnCode::MaxIters
    } else if gnrng <= opts.x_reltol {
        ReturnCode::Success
    } else if criter_change <= opts.f_reltol {
        ReturnCode::Stalled
    } else {
        ReturnCode::Failure
    };

    if opts.verbose {
        println!("Exit: {:?}", code);
    }

    SceResult { best_x: x[0].clone(), best_f: xf[0], n_evals, code }
}

fn evolve_complex<F>(
    f: &F,
    x: &[Vec<f64>],
    xf: &[f64],
    igs: usize,
    n_complex: usize,
    size_complex: usize,
    size_simplex: usize,
    n_evolu: usize,
    bl: &[f64],
    bu: &[f64],
    seed: u64,
    nloop: u64,
) -> (Vec<Vec<f64>>, Vec<f64>, usize)
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    let k2: Vec<usize> = (0..size_complex).map(|k| k * n_complex + igs).collect();
    let mut cx: Vec<Vec<f64>> = k2.iter().map(|&i| x[i].clone()).collect();
    let mut cf: Vec<f64>      = k2.iter().map(|&i| xf[i]).collect();
    let mut total_evals = 0usize;

    for loop_ in 0..n_evolu {
        let mut rng = SmallRng::seed_from_u64(rng_seed(seed, nloop, igs as u64, loop_ as u64));

        // sample simplex indices from complex with triangular probability
        let mut lcs = vec![0usize; size_simplex];
        lcs[0] = 0;
        for k3 in 1..size_simplex {
            let mut lpos;
            loop {
                let r: f64 = rng.gen();
                let sc = size_complex as f64;
                lpos = (1.0 + (sc + 0.5 - ((sc + 0.5).powi(2) - sc * (sc + 1.0) * r).sqrt()))
                    as usize;
                lpos = lpos.saturating_sub(1).min(size_complex - 1);
                if !lcs[..k3].contains(&lpos) { break; }
            }
            lcs[k3] = lpos;
        }
        lcs.sort_unstable();

        let mut s:  Vec<Vec<f64>> = lcs.iter().map(|&i| cx[i].clone()).collect();
        let mut sf: Vec<f64>      = lcs.iter().map(|&i| cf[i]).collect();

        let (_, _, evals) = cceua(f, &mut s, &mut sf, bl, bu, &mut rng);
        total_evals += evals;

        // put evolved simplex back
        let worst = lcs[size_simplex - 1];
        cx[worst] = s[size_simplex - 1].clone();
        cf[worst] = sf[size_simplex - 1];

        sort_simplex(&mut cx, &mut cf);
    }

    (cx, cf, total_evals)
}

fn sort_pop(x: &mut Vec<Vec<f64>>, xf: &mut Vec<f64>) {
    let mut idx: Vec<usize> = (0..xf.len()).collect();
    idx.sort_by(|&a, &b| xf[a].partial_cmp(&xf[b]).unwrap());
    let xs: Vec<Vec<f64>> = idx.iter().map(|&i| x[i].clone()).collect();
    let fs: Vec<f64>      = idx.iter().map(|&i| xf[i]).collect();
    *x  = xs;
    *xf = fs;
}

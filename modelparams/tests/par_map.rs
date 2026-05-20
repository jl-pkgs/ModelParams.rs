use modelparams::{par_map, par_map_cloned};
use std::time::Instant;
use std::thread;

// ── simple map tests ──────────────────────────────────────────────────────────

#[test]
fn test_serial_map() {
    let a: Vec<i32> = (1..=20).collect();
    let r = par_map(&a, false, |x| x * x);
    let expected: Vec<i32> = a.iter().map(|x| x * x).collect();
    assert_eq!(r, expected);
}

#[test]
fn test_parallel_map() {
    let a: Vec<i32> = (1..=20).collect();
    let r = par_map(&a, true, |x| x + 1);
    let expected: Vec<i32> = a.iter().map(|x| x + 1).collect();
    assert_eq!(r, expected);
}

#[test]
fn test_serial_parallel_agree() {
    let a: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let rs = par_map(&a, false, |x| x * x + x.sqrt());
    let rp = par_map(&a, true,  |x| x * x + x.sqrt());
    for (s, p) in rs.iter().zip(rp.iter()) {
        assert!((s - p).abs() < 1e-12, "serial={s}, parallel={p}");
    }
}

// ── state-cloning tests ───────────────────────────────────────────────────────

#[derive(Clone)]
struct SolverState {
    counter: i32,
    scratch: Vec<f64>,
}

#[derive(Clone)]
struct SolverConfig {
    state: SolverState,
}

fn simulate_step(x: &i32, mut cfg: SolverConfig) -> i32 {
    cfg.state.counter += 1;
    cfg.state.scratch[0] += *x as f64;
    x * x
}

#[test]
fn test_cloned_serial_state_unchanged() {
    let a: Vec<i32> = (1..=20).collect();
    let cfg = SolverConfig { state: SolverState { counter: 0, scratch: vec![0.0] } };
    let r = par_map_cloned(&a, &cfg, false, simulate_step);
    assert_eq!(cfg.state.counter, 0, "original state must not be modified");
    assert_eq!(cfg.state.scratch[0], 0.0);
    let expected: Vec<i32> = a.iter().map(|x| x * x).collect();
    assert_eq!(r, expected);
}

#[test]
fn test_cloned_parallel_state_unchanged() {
    let a: Vec<i32> = (1..=20).collect();
    let cfg = SolverConfig { state: SolverState { counter: 0, scratch: vec![0.0] } };
    let r = par_map_cloned(&a, &cfg, true, simulate_step);
    assert_eq!(cfg.state.counter, 0, "original state must not be modified");
    assert_eq!(cfg.state.scratch[0], 0.0);
    let expected: Vec<i32> = a.iter().map(|x| x * x).collect();
    assert_eq!(r, expected);
}

#[test]
fn test_cloned_serial_increments_per_clone() {
    // Each clone starts counter=0, increments to 1; result == item index + 1
    let a: Vec<i32> = (1..=20).collect();
    let cfg = SolverConfig { state: SolverState { counter: 0, scratch: vec![0.0] } };
    // closure returns counter after increment (which is always 1 per clone)
    let r = par_map_cloned(&a, &cfg, false, |_x, mut c| { c.state.counter += 1; c.state.counter });
    assert!(r.iter().all(|&v| v == 1), "each clone counter should reach exactly 1");
}

// ── timing / speedup test ─────────────────────────────────────────────────────

fn sleep_and_square(x: &i32, _cfg: SolverConfig) -> i32 {
    thread::sleep(std::time::Duration::from_millis(10));
    x * x
}

#[test]
#[ignore]  // run explicitly with: cargo test timing -- --ignored --nocapture
fn test_parallel_speedup() {
    let a: Vec<i32> = (1..=48).collect();
    let cfg = SolverConfig { state: SolverState { counter: 0, scratch: vec![0.0] } };
    let expected: Vec<i32> = a.iter().map(|x| x * x).collect();

    let t0 = Instant::now();
    let rs = par_map_cloned(&a, &cfg, false, sleep_and_square);
    let t_serial = t0.elapsed();

    let t1 = Instant::now();
    let rp = par_map_cloned(&a, &cfg, true,  sleep_and_square);
    let t_parallel = t1.elapsed();

    assert_eq!(rs, expected);
    assert_eq!(rp, expected);

    let speedup = t_serial.as_secs_f64() / t_parallel.as_secs_f64();
    println!(
        "par_map: serial={:.2}s, parallel={:.2}s, speedup={:.2}x",
        t_serial.as_secs_f64(), t_parallel.as_secs_f64(), speedup
    );

    let n_cpus = rayon::current_num_threads();
    if n_cpus > 1 {
        assert!(speedup > 1.5, "expected speedup > 1.5x on {n_cpus} threads, got {speedup:.2}x");
    }
}

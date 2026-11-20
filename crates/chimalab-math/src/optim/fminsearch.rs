//! Mínimo sem restrições pelo simplex de Nelder–Mead.
//!
//! J. A. Nelder & R. Mead, Comput. J. 7 (1965) 308–313; regras de ordenação, desempate e
//! aceitação de J. C. Lagarias, J. A. Reeds, M. H. Wright & P. E. Wright, SIAM J. Optim. 9 (1998)
//! 112–147; parâmetros adaptativos de F. Gao & L. Han, Comput. Optim. Appl. 51 (2012) 259–277;
//! reinício contra a convergência para pontos não estacionários (K. I. M. McKinnon, SIAM J.
//! Optim. 9, 1998; C. T. Kelley, SIAM J. Optim. 10, 1999).

use super::{ExitFlag, OptimError, OptimResult, Options, Procedure, Tracker};

/// Mínimo local de `f: ℝⁿ → ℝ` a partir de `x0`, sem derivadas.
///
/// - Parâmetros: reflexão 1, expansão 2, contração ½ e encolhimento ½; com
///   [`Options::adaptive`], 1, 1 + 2/n, ¾ − 1/(2n) e 1 − 1/n (Gao & Han), melhores para n ≥ 5.
/// - Simplex inicial: x0 e x0 + hᵢ·eᵢ, com hᵢ = 0,05·|x0ᵢ| (ou 0,05·max(1, ‖x0‖∞) se x0ᵢ = 0).
///   [`Options::initial_step`] e [`Options::initial_simplex`] substituem a regra.
/// - Convergência: max ‖xᵢ − x₀‖∞ ≤ tol_x·max(1, ‖x₀‖∞) **e** max |fᵢ − f₀| ≤ tol_fun·max(1, |f₀|).
/// - Reinício: ao convergir, reinicia do melhor ponto com um simplex novo, até
///   [`Options::restarts`] vezes (padrão 1) enquanto o valor melhorar além de tol_fun.
/// - NaN conta como +∞, o que permite funções com domínio restrito.
///
/// Padrões: `tol_x` = `tol_fun` = 1e-8, `max_iter` = 1000·n, `max_fun_evals` = 2000·n.
///
/// Erros: `pontoInicialInvalido` (vazio, não finito, f(x0) NaN), `opcaoInvalida` (passo ou
/// simplex com dimensão errada, ou simplex degenerado), `toleranciaInvalida`.
pub fn fminsearch(
    mut f: impl FnMut(&[f64]) -> f64,
    x0: &[f64],
    opts: &mut Options<'_>,
) -> Result<OptimResult<Vec<f64>>, OptimError> {
    let n = x0.len();
    if n == 0 {
        return Err(OptimError::InvalidStart("ponto inicial vazio"));
    }
    if x0.iter().any(|v| !v.is_finite()) {
        return Err(OptimError::InvalidStart("ponto inicial não finito"));
    }
    let tol_x = opts.tol_x.unwrap_or(1e-8);
    let tol_f = opts.tol_fun.unwrap_or(1e-8);
    let restarts = opts.restarts.unwrap_or(1);
    let nf = n as f64;
    let (rho, chi, gamma, sigma) = if opts.adaptive {
        (1.0, 1.0 + 2.0 / nf, 0.75 - 0.5 / nf, 1.0 - 1.0 / nf)
    } else {
        (1.0, 2.0, 0.5, 0.5)
    };
    let step = match &opts.initial_step {
        Some(h) if h.len() != n => return Err(OptimError::InvalidOption("initial_step: dimensão")),
        Some(h) if h.iter().any(|v| !v.is_finite() || *v == 0.0) => {
            return Err(OptimError::InvalidOption(
                "initial_step: passos finitos e não nulos",
            ));
        }
        Some(h) => Some(h.clone()),
        None => None,
    };
    let given = match opts.initial_simplex.take() {
        Some(s) => {
            if s.len() != n + 1 || s.iter().any(|p| p.len() != n) {
                return Err(OptimError::InvalidOption(
                    "initial_simplex: n + 1 pontos de dimensão n",
                ));
            }
            if s.iter().flatten().any(|v| !v.is_finite()) {
                return Err(OptimError::InvalidOption("initial_simplex: não finito"));
            }
            if degenerate(&s) {
                return Err(OptimError::InvalidOption("initial_simplex: degenerado"));
            }
            Some(s)
        }
        None => None,
    };
    let mut t = Tracker::new(opts, 1000 * n, 2000 * n)?;
    let f0 = f(x0);
    t.fun_evals += 1;
    if f0.is_nan() {
        return Err(OptimError::InvalidStart("f(x0) é NaN"));
    }
    let mut eval = |t: &mut Tracker, x: &[f64]| {
        t.fun_evals += 1;
        let y = f(x);
        if y.is_nan() { f64::INFINITY } else { y }
    };
    let mut pts: Vec<Vec<f64>>;
    let mut fs: Vec<f64>;
    match given {
        Some(s) => {
            fs = s.iter().map(|p| eval(&mut t, p)).collect();
            pts = s;
        }
        None => {
            pts = initial(x0, step.as_deref());
            fs = vec![f0];
            for p in &pts[1..] {
                fs.push(eval(&mut t, p));
            }
        }
    }
    sort(&mut pts, &mut fs);
    let mut exit = t.report(&pts[0], fs[0], Procedure::Initial, None);
    let mut restarts_left = restarts;
    let mut last_restart_f = f64::INFINITY;
    while exit.is_none() {
        if converged(&pts, &fs, tol_x, tol_f) {
            let fbest = fs[0];
            let improved = last_restart_f - fbest > tol_f * fbest.abs().max(1.0);
            if restarts_left == 0 || (last_restart_f.is_finite() && !improved) {
                exit = Some(ExitFlag::Converged);
                break;
            }
            restarts_left -= 1;
            last_restart_f = fbest;
            let best = pts[0].clone();
            pts = initial(&best, step.as_deref());
            fs.truncate(1);
            fs[0] = fbest;
            for p in &pts[1..] {
                fs.push(eval(&mut t, p));
            }
            sort(&mut pts, &mut fs);
            t.iterations += 1;
            exit = t.report(&pts[0], fs[0], Procedure::Restart, None);
            continue;
        }
        let worst = &pts[n];
        let mut xbar = vec![0.0; n];
        for p in &pts[..n] {
            for (c, v) in xbar.iter_mut().zip(p) {
                *c += v / nf;
            }
        }
        let along =
            |coef: f64| -> Vec<f64> { xbar.iter().zip(worst).map(|(c, w)| c + coef * (c - w)).collect() };
        let xr = along(rho);
        let fr = eval(&mut t, &xr);
        let accepted = if fr < fs[0] {
            let xe = along(rho * chi);
            let fe = eval(&mut t, &xe);
            if fe < fr {
                Some((xe, fe, Procedure::Expand))
            } else {
                Some((xr, fr, Procedure::Reflect))
            }
        } else if fr < fs[n - 1] {
            Some((xr, fr, Procedure::Reflect))
        } else if fr < fs[n] {
            let xc = along(rho * gamma);
            let fc = eval(&mut t, &xc);
            (fc <= fr).then_some((xc, fc, Procedure::ContractOutside))
        } else {
            let xcc = along(-gamma);
            let fcc = eval(&mut t, &xcc);
            (fcc < fs[n]).then_some((xcc, fcc, Procedure::ContractInside))
        };
        let proc = match accepted {
            Some((x, fx, proc)) => {
                // Insere após os pontos de valor igual (Lagarias et al.).
                pts.pop();
                fs.pop();
                let k = fs.partition_point(|&v| v <= fx);
                pts.insert(k, x);
                fs.insert(k, fx);
                proc
            }
            None => {
                let best = pts[0].clone();
                for i in 1..=n {
                    for (v, b) in pts[i].iter_mut().zip(&best) {
                        *v = b + sigma * (*v - b);
                    }
                    fs[i] = eval(&mut t, &pts[i]);
                }
                sort(&mut pts, &mut fs);
                Procedure::Shrink
            }
        };
        t.iterations += 1;
        exit = t.report(&pts[0], fs[0], proc, None);
    }
    let x = pts.swap_remove(0);
    Ok(t.finish(x, fs[0], exit.unwrap_or(ExitFlag::Converged), None))
}

fn initial(x0: &[f64], step: Option<&[f64]>) -> Vec<Vec<f64>> {
    let scale = x0.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1.0);
    let mut pts = vec![x0.to_vec()];
    for i in 0..x0.len() {
        let h = match step {
            Some(h) => h[i],
            None if x0[i] != 0.0 => 0.05 * x0[i],
            None => 0.05 * scale,
        };
        let mut p = x0.to_vec();
        p[i] += h;
        // Passo absorvido pelo arredondamento (|h| < ulp(x0ᵢ)): usa o vizinho seguinte.
        if p[i] == x0[i] {
            p[i] = next_away(x0[i], h);
        }
        pts.push(p);
    }
    pts
}

/// Vizinho de x ≠ 0 em ponto flutuante no sentido de `dir` (0 + h nunca é absorvido).
fn next_away(x: f64, dir: f64) -> f64 {
    let b = x.to_bits();
    let up = (dir > 0.0) == (x > 0.0);
    if up {
        f64::from_bits(b + 1)
    } else {
        f64::from_bits(b - 1)
    }
}

/// Ordena por valor; empates mantêm a ordem anterior (ordenação estável).
fn sort(pts: &mut Vec<Vec<f64>>, fs: &mut Vec<f64>) {
    let mut idx: Vec<usize> = (0..fs.len()).collect();
    idx.sort_by(|&i, &j| fs[i].total_cmp(&fs[j]));
    *pts = idx.iter().map(|&i| pts[i].clone()).collect();
    *fs = idx.iter().map(|&i| fs[i]).collect();
}

fn converged(pts: &[Vec<f64>], fs: &[f64], tol_x: f64, tol_f: f64) -> bool {
    let x0 = &pts[0];
    let sx = x0.iter().fold(1.0f64, |m, v| m.max(v.abs()));
    let dx = pts[1..]
        .iter()
        .flat_map(|p| p.iter().zip(x0).map(|(a, b)| (a - b).abs()))
        .fold(0.0f64, f64::max);
    let df = fs[1..].iter().map(|v| (v - fs[0]).abs()).fold(0.0f64, f64::max);
    dx <= tol_x * sx && df <= tol_f * fs[0].abs().max(1.0)
}

/// Simplex degenerado: as arestas a partir do primeiro ponto são linearmente dependentes
/// (eliminação gaussiana com pivotamento e tolerância relativa).
fn degenerate(s: &[Vec<f64>]) -> bool {
    let n = s.len() - 1;
    let mut m: Vec<Vec<f64>> = s[1..]
        .iter()
        .map(|p| p.iter().zip(&s[0]).map(|(a, b)| a - b).collect())
        .collect();
    let scale = m.iter().flatten().fold(0.0f64, |a, v| a.max(v.abs()));
    if scale == 0.0 {
        return true;
    }
    for col in 0..n {
        let piv = (col..n)
            .max_by(|&i, &j| m[i][col].abs().total_cmp(&m[j][col].abs()))
            .unwrap_or(col);
        if m[piv][col].abs() <= 1e-12 * scale {
            return true;
        }
        m.swap(col, piv);
        for i in col + 1..n {
            let r = m[i][col] / m[col][col];
            for k in col..n {
                m[i][k] -= r * m[col][k];
            }
        }
    }
    false
}

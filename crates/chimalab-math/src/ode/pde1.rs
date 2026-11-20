//! EDP parabólica e elíptica 1-D pelo método das linhas (F050):
//!
//! c(x, t, u, uₓ)·∂u/∂t = x^{−m}·∂/∂x(x^m·f(x, t, u, uₓ)) + s(x, t, u, uₓ),  x ∈ [a, b],
//!
//! com m = 0 (placa), 1 (cilindro) ou 2 (esfera), e contorno p(x, t, u) + q(x, t)·f = 0 em cada
//! extremo.
//!
//! Discretização conservativa em volumes finitos com os pontos médios de R. D. Skeel & M. Berzins,
//! SIAM J. Sci. Stat. Comput. 11 (1990) 1–32: ξ = média aritmética (m = 0), logarítmica (m = 1)
//! ou a correspondente esférica (m = 2), o que mantém a ordem 2 perto de x = 0 em geometria
//! cilíndrica ou esférica. Contorno de Dirichlet (q = 0) vira equação algébrica (linha nula da
//! matriz de massa), e a integração no tempo é a do Radau IIA com massa (F049).

use super::{AbsTol, Method, OdeError, OdeOptions, OdeStatus, solve_ivp};
use crate::linalg::Matrix;

/// Coeficientes (c, f, s) da EDP num ponto: `pde(x, t, u, ux)`.
pub type PdeFn<'a> = &'a dyn Fn(f64, f64, f64, f64) -> (f64, f64, f64);
/// Contorno: `bc(xa, ua, xb, ub, t)` → (pa, qa, pb, qb).
pub type BcFn<'a> = &'a dyn Fn(f64, f64, f64, f64, f64) -> (f64, f64, f64, f64);

/// Ponto médio de Skeel–Berzins entre xl e xr para a simetria m.
fn xi(m: u32, xl: f64, xr: f64) -> f64 {
    match m {
        0 => 0.5 * (xl + xr),
        1 if xl > 0.0 => (xr - xl) / (xr / xl).ln(),
        2 if xl > 0.0 => xl * xr * (xr / xl).ln() / (xr - xl),
        // Com xl = 0 o limite das médias logarítmicas é 0, o que zeraria o peso ξ^m e
        // desacoplaria o nó central; usa-se o ponto médio aritmético (ordem 2 mantida).
        _ => 0.5 * (xl + xr),
    }
}

/// Resolve a EDP escalar e devolve u exatamente nos instantes pedidos, `u[k][i]` = u(x_i, t_k),
/// também quando `tspan` tem só dois instantes. Se a integração parar antes (estado diferente de
/// `Completed`), só vêm os instantes alcançados.
///
/// Erros: `opcaoInvalida` (m > 2; m > 0 com a < 0), `intervaloInvalido` (malha com < 3 pontos ou
/// fora de ordem; instantes), e os de [`solve_ivp`]. Falhas na integração voltam como
/// `StepTooSmall`/`NonFinite` no estado.
pub fn pde1(
    m: u32,
    pde: PdeFn<'_>,
    ic: &dyn Fn(f64) -> f64,
    bc: BcFn<'_>,
    xmesh: &[f64],
    tspan: &[f64],
    rel_tol: f64,
) -> Result<(Vec<Vec<f64>>, OdeStatus), OdeError> {
    if m > 2 {
        return Err(OdeError::InvalidOption("pde1: m ∈ {0, 1, 2}"));
    }
    let n = xmesh.len();
    if n < 3 || xmesh.iter().any(|v| !v.is_finite()) || xmesh.windows(2).any(|w| w[1] <= w[0]) {
        return Err(OdeError::InvalidTspan(
            "pde1: malha com ≥ 3 pontos, crescente e finita",
        ));
    }
    if m > 0 && xmesh[0] < 0.0 {
        return Err(OdeError::InvalidOption(
            "pde1: com m > 0, a malha deve começar em x ≥ 0",
        ));
    }
    let x = xmesh.to_vec();
    let t0 = tspan.first().copied().unwrap_or(0.0);
    let u0: Vec<f64> = x.iter().map(|&xi| ic(xi)).collect();
    // Pontos médios e volumes (∫ x^m dx entre os pontos médios).
    let xm: Vec<f64> = (0..n - 1).map(|i| xi(m, x[i], x[i + 1])).collect();
    let pot = |v: f64| v.powi(m as i32 + 1) / (m as f64 + 1.0);
    let vol: Vec<f64> = (0..n)
        .map(|i| {
            let l = if i == 0 { x[0] } else { xm[i - 1] };
            let r = if i == n - 1 { x[n - 1] } else { xm[i] };
            pot(r) - pot(l)
        })
        .collect();
    let peso = |v: f64| if m == 0 { 1.0 } else { v.powi(m as i32) };
    let (a, b) = (x[0], x[n - 1]);
    // Contorno de Dirichlet (q = 0) em cada extremo: decidido em t₀ (q só depende de x e t).
    let (_, qa0, _, qb0) = bc(a, u0[0], b, u0[n - 1], t0);
    let (dir_a, dir_b) = (qa0 == 0.0, qb0 == 0.0);
    let rhs = |t: f64, u: &[f64], du: &mut [f64]| {
        // Fluxos nos pontos médios.
        let fl: Vec<f64> = (0..n - 1)
            .map(|i| {
                let h = x[i + 1] - x[i];
                let um = 0.5 * (u[i] + u[i + 1]);
                pde(xm[i].max(x[i]).min(x[i + 1]), t, um, (u[i + 1] - u[i]) / h).1
            })
            .collect();
        let (pa, qa, pb, qb) = bc(a, u[0], b, u[n - 1], t);
        for i in 0..n {
            let ux = if i == 0 {
                (u[1] - u[0]) / (x[1] - x[0])
            } else if i == n - 1 {
                (u[n - 1] - u[n - 2]) / (x[n - 1] - x[n - 2])
            } else {
                (u[i + 1] - u[i - 1]) / (x[i + 1] - x[i - 1])
            };
            let (c, _, s) = pde(x[i], t, u[i], ux);
            // Fluxo à esquerda e à direita do volume i.
            let f_esq = if i == 0 {
                if dir_a { 0.0 } else { -pa / qa }
            } else {
                fl[i - 1]
            };
            let f_dir = if i == n - 1 {
                if dir_b { 0.0 } else { -pb / qb }
            } else {
                fl[i]
            };
            let w_esq = if i == 0 { peso(x[0]) } else { peso(xm[i - 1]) };
            let w_dir = if i == n - 1 { peso(x[n - 1]) } else { peso(xm[i]) };
            let _ = c; // c entra na matriz de massa
            du[i] = w_dir * f_dir - w_esq * f_esq + s * vol[i];
        }
        // Dirichlet: equação algébrica p = 0 (com massa nula na linha).
        if dir_a {
            du[0] = pa;
        }
        if dir_b {
            du[n - 1] = pb;
        }
    };
    // Matriz de massa: c·volume na diagonal; zero nas linhas de Dirichlet. c avaliado em t₀ com
    // u₀ (este incremento supõe c independente de t e u; massa variável fica para depois).
    let mut mass = Matrix::zeros(n, n);
    for i in 0..n {
        let ux = if i == 0 || i == n - 1 {
            0.0
        } else {
            (u0[i + 1] - u0[i - 1]) / (x[i + 1] - x[i - 1])
        };
        let (c, _, _) = pde(x[i], t0, u0[i], ux);
        mass.set(i, i, c * vol[i]);
    }
    if dir_a {
        mass.set(0, 0, 0.0);
    }
    if dir_b {
        mass.set(n - 1, n - 1, 0.0);
    }
    // Condição inicial consistente nos extremos de Dirichlet: resolve p(u) = 0 por secante.
    let mut y0 = u0.clone();
    for (lado, ativo) in [(0usize, dir_a), (n - 1, dir_b)] {
        if !ativo {
            continue;
        }
        let p = |v: f64| {
            let mut w = y0.clone();
            w[lado] = v;
            let (pa, _, pb, _) = bc(a, w[0], b, w[n - 1], t0);
            if lado == 0 { pa } else { pb }
        };
        let (mut v0, mut v1) = (y0[lado], y0[lado] + 1e-3 * y0[lado].abs().max(1.0));
        for _ in 0..50 {
            let (p0, p1) = (p(v0), p(v1));
            if p1 == p0 || p1.abs() <= 1e-15 {
                break;
            }
            let v2 = v1 - p1 * (v1 - v0) / (p1 - p0);
            v0 = v1;
            v1 = v2;
        }
        y0[lado] = v1;
    }
    let mut opts = OdeOptions {
        method: Method::Radau,
        rel_tol: Some(rel_tol),
        abs_tol: Some(AbsTol::Scalar(rel_tol * 1e-3)),
        mass: Some(mass),
        dense: true,
        ..Default::default()
    };
    let s = solve_ivp(rhs, tspan, &y0, &mut opts)?;
    // Sempre nos instantes pedidos (com dois instantes, `solve_ivp` devolveria todos os passos).
    let u = tspan.iter().filter_map(|&t| s.eval(t)).collect();
    Ok((u, s.status))
}

//! Polinômios (funcionalidade F045, RF-045).
//!
//! Convenção: coeficientes em ordem **decrescente** de grau, `p[0]·xⁿ + … + p[n]`.

use num_complex::Complex64;

use crate::linalg::{Eig, LinalgError, LinalgWarning, Matrix, MatrixExt, eig, solve};
use crate::signal::{ConvShape, conv, deconv};

/// Avalia o polinômio em cada ponto (método de Horner).
pub fn polyval(p: &[f64], x: &[f64]) -> Vec<f64> {
    x.iter()
        .map(|&v| p.iter().fold(0.0, |acc, &c| acc * v + c))
        .collect()
}

/// Avalia o polinômio (coeficientes reais) em pontos complexos.
pub fn polyval_complex(p: &[f64], x: &[Complex64]) -> Vec<Complex64> {
    x.iter()
        .map(|&v| p.iter().fold(Complex64::new(0.0, 0.0), |acc, &c| acc * v + c))
        .collect()
}

/// Raízes do polinômio, pelos autovalores da matriz companheira.
///
/// Zeros à esquerda são ignorados; zeros à direita produzem raízes nulas. Polinômio constante
/// (ou nulo) não tem raízes.
pub fn roots(p: &[f64]) -> Result<Vec<Complex64>, LinalgError> {
    if p.iter().any(|c| !c.is_finite()) {
        return Err(LinalgError::NonFinite { operation: "roots" });
    }
    let first = match p.iter().position(|&c| c != 0.0) {
        Some(i) => i,
        None => return Ok(vec![]),
    };
    let last = p
        .iter()
        .rposition(|&c| c != 0.0)
        .expect("há coeficiente não nulo");
    let core = &p[first..=last];
    let zeros = p.len() - 1 - last;
    let n = core.len() - 1;
    let mut r = Vec::with_capacity(n + zeros);
    if n > 0 {
        let c = Matrix::from_fn(n, n, |i, j| {
            if i == 0 {
                -core[j + 1] / core[0]
            } else if i == j + 1 {
                1.0
            } else {
                0.0
            }
        });
        match eig(&c)? {
            Eig::General { values, .. } => r.extend(values),
            Eig::Symmetric { values, .. } => r.extend(values.iter().map(|&v| Complex64::new(v, 0.0))),
        }
    }
    r.extend(std::iter::repeat_n(Complex64::new(0.0, 0.0), zeros));
    Ok(r)
}

/// Coeficientes (mônicos) do polinômio com as raízes dadas. Se as raízes complexas vierem em
/// pares conjugados, as partes imaginárias residuais são descartadas e o resultado é real.
pub fn poly(roots: &[Complex64]) -> (Vec<Complex64>, Option<Vec<f64>>) {
    let mut c = vec![Complex64::new(1.0, 0.0)];
    for &r in roots {
        let mut next = c.clone();
        next.push(Complex64::new(0.0, 0.0));
        for k in 1..next.len() {
            next[k] -= r * c[k - 1];
        }
        c = next;
    }
    // Real quando o multiconjunto de raízes é fechado por conjugação.
    let mut unmatched: Vec<Complex64> = roots.iter().filter(|z| z.im != 0.0).copied().collect();
    let mut closed = true;
    while let Some(z) = unmatched.pop() {
        let scale = z.norm().max(1.0);
        match unmatched
            .iter()
            .position(|w| (w - z.conj()).norm() <= 1e-10 * scale)
        {
            Some(i) => {
                unmatched.remove(i);
            }
            None => {
                closed = false;
                break;
            }
        }
    }
    let real = closed.then(|| c.iter().map(|z| z.re).collect());
    (c, real)
}

/// Centralização e escala `x̂ = (x − μ₀)/μ₁` usadas por [`polyfit_scaled`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Centering {
    /// Média dos pontos.
    pub mean: f64,
    /// Desvio padrão amostral dos pontos.
    pub std: f64,
}

/// Resultado de um ajuste polinomial.
#[derive(Clone, Debug)]
pub struct PolyFit {
    /// Coeficientes (ordem decrescente).
    pub p: Vec<f64>,
    /// Fator triangular `R` da QR da matriz de Vandermonde (útil para intervalos de previsão).
    pub r: Matrix,
    /// Graus de liberdade: `npontos − (grau + 1)` (mínimo 0).
    pub df: usize,
    /// Norma 2 do resíduo.
    pub normr: f64,
    /// Centralização aplicada (só em [`polyfit_scaled`]).
    pub centering: Option<Centering>,
    /// Avisos (ex.: problema mal condicionado ou com posto deficiente).
    pub warnings: Vec<LinalgWarning>,
}

/// Ajuste polinomial de grau `n` por mínimos quadrados.
pub fn polyfit(x: &[f64], y: &[f64], n: usize) -> Result<PolyFit, LinalgError> {
    fit(x, y, n, None)
}

/// Ajuste polinomial com centralização e escala de `x` (melhora o condicionamento). Para avaliar,
/// use `polyval(p, (x − mean)/std)`.
pub fn polyfit_scaled(x: &[f64], y: &[f64], n: usize) -> Result<PolyFit, LinalgError> {
    let m = x.len() as f64;
    let mean = x.iter().sum::<f64>() / m;
    let std = if x.len() > 1 {
        (x.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (m - 1.0)).sqrt()
    } else {
        1.0
    };
    let std = if std == 0.0 { 1.0 } else { std };
    fit(x, y, n, Some(Centering { mean, std }))
}

fn fit(x: &[f64], y: &[f64], n: usize, c: Option<Centering>) -> Result<PolyFit, LinalgError> {
    if x.len() != y.len() {
        return Err(LinalgError::DimensionMismatch {
            operation: "polyfit",
            left: (x.len(), 1),
            right: (y.len(), 1),
        });
    }
    let xs: Vec<f64> = match c {
        Some(c) => x.iter().map(|v| (v - c.mean) / c.std).collect(),
        None => x.to_vec(),
    };
    let m = xs.len();
    let v = Matrix::from_fn(m, n + 1, |i, j| xs[i].powi((n - j) as i32));
    let b = Matrix::column(y);
    let sol = solve(&v, &b)?;
    let p: Vec<f64> = (0..=n).map(|j| sol.x.get(j, 0)).collect();
    let mut warnings = sol.warnings;
    if m < n + 1 && warnings.is_empty() {
        // Sistema subdeterminado: infinitas soluções; foi devolvida uma solução básica.
        warnings.push(LinalgWarning::RankDeficient { rank: m, tol: 0.0 });
    }
    let fitted = polyval(&p, &xs);
    let normr = fitted
        .iter()
        .zip(y)
        .map(|(f, t)| (f - t).powi(2))
        .sum::<f64>()
        .sqrt();
    let qr = crate::linalg::qr(&v)?;
    let k = (n + 1).min(m);
    let r = qr.r.submatrix(0, k, 0, n + 1);
    Ok(PolyFit {
        p,
        r,
        df: m.saturating_sub(n + 1),
        normr,
        centering: c,
        warnings,
    })
}

/// Derivada do polinômio.
pub fn polyder(p: &[f64]) -> Vec<f64> {
    let n = p.len();
    if n <= 1 {
        return vec![0.0];
    }
    let d: Vec<f64> = (0..n - 1).map(|i| p[i] * (n - 1 - i) as f64).collect();
    trim(d)
}

/// Derivada do produto `a·b`.
pub fn polyder_product(a: &[f64], b: &[f64]) -> Vec<f64> {
    polyder(&conv(a, b, ConvShape::Full))
}

/// Derivada do quociente `a/b`: devolve `(numerador, denominador)` = `(a'b − ab', b²)`.
pub fn polyder_quotient(a: &[f64], b: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let t1 = conv(&polyder(a), b, ConvShape::Full);
    let t2 = conv(a, &polyder(b), ConvShape::Full);
    (trim(sub_aligned(&t1, &t2)), trim(conv(b, b, ConvShape::Full)))
}

/// Integral do polinômio com constante de integração `k`.
pub fn polyint(p: &[f64], k: f64) -> Vec<f64> {
    let n = p.len();
    let mut out: Vec<f64> = (0..n).map(|i| p[i] / (n - i) as f64).collect();
    out.push(k);
    out
}

fn sub_aligned(a: &[f64], b: &[f64]) -> Vec<f64> {
    let n = a.len().max(b.len());
    (0..n)
        .map(|i| {
            let ai = (i + a.len()).checked_sub(n).map_or(0.0, |j| a[j]);
            let bi = (i + b.len()).checked_sub(n).map_or(0.0, |j| b[j]);
            ai - bi
        })
        .collect()
}

fn trim(mut p: Vec<f64>) -> Vec<f64> {
    while p.len() > 1 && p[0] == 0.0 {
        p.remove(0);
    }
    if p.is_empty() { vec![0.0] } else { p }
}

/// Expansão em frações parciais de `B(s)/A(s)`.
#[derive(Clone, Debug)]
pub struct Residue {
    /// Resíduos, na mesma ordem de `poles`.
    pub residues: Vec<Complex64>,
    /// Polos; um polo de multiplicidade `m` aparece `m` vezes seguidas, e o `j`-ésimo resíduo
    /// do grupo (j = 1..m) multiplica `1/(s − p)ʲ`.
    pub poles: Vec<Complex64>,
    /// Termo direto (quociente polinomial), vazio se `grau(B) < grau(A)`.
    pub direct: Vec<f64>,
}

/// Frações parciais: `B/A = Σ rᵢⱼ/(s − pᵢ)ʲ + k(s)`. Polos numericamente próximos são agrupados
/// como polo repetido, com tolerância relativa que cresce com a multiplicidade (de 1e-5 para
/// polos duplos a 1e-2; ver `cluster_tol`), e o grupo é representado pela média das raízes.
pub fn residue(b: &[f64], a: &[f64]) -> Result<Residue, LinalgError> {
    let a = trim(a.to_vec());
    let b = trim(b.to_vec());
    if a == [0.0] {
        return Err(LinalgError::InvalidArgument {
            operation: "residue",
            reason: "denominador nulo",
        });
    }
    let (direct, rem) = if b.len() >= a.len() {
        let (q, r) = deconv(&b, &a).map_err(|_| LinalgError::NonFinite { operation: "residue" })?;
        (q, r[r.len() - (a.len() - 1)..].to_vec())
    } else {
        (vec![], b.clone())
    };
    let poles = roots(&a)?;
    let groups = group_poles(&poles);
    let lead = a[0];
    let mut residues = vec![];
    let mut out_poles = vec![];
    for (gi, &(p, m)) in groups.iter().enumerate() {
        // D(s) = lead·Π_{outros grupos}(s − q)^mq ; N(s) = resto.
        // Coeficientes de Taylor de N/D em torno de p até ordem m−1:
        // B/A = (1/(s−p)^m)·Σ cₖ(s−p)ᵏ ⇒ resíduo de 1/(s−p)^(m−k) é cₖ.
        let n_t = taylor(
            &rem.iter().map(|&c| Complex64::new(c, 0.0)).collect::<Vec<_>>(),
            p,
            m,
        );
        let mut d_poly = vec![Complex64::new(lead, 0.0)];
        for (gj, &(q, mq)) in groups.iter().enumerate() {
            if gj != gi {
                for _ in 0..mq {
                    d_poly = mul_linear(&d_poly, q);
                }
            }
        }
        let d_t = taylor(&d_poly, p, m);
        let c = series_div(&n_t, &d_t, m);
        for j in 1..=m {
            out_poles.push(p);
            residues.push(c[m - j]);
        }
    }
    Ok(Residue {
        residues,
        poles: out_poles,
        direct,
    })
}

/// Tolerância relativa para reconhecer `k` raízes numéricas como uma raiz de multiplicidade `k`.
/// A companheira espalha uma raiz `k`-upla a uma distância relativa ~ε^(1/k) do valor exato; o
/// fator 8 dá margem para o condicionamento dos demais fatores. O piso 1e-5 vale para polos
/// duplos (pares distintos a mais de 1e-5 nunca são fundidos); o teto 1e-2 cobre multiplicidade
/// até 6 e limita a fusão de polos distintos próximos quando `k` é grande.
fn cluster_tol(k: usize) -> f64 {
    (8.0 * f64::EPSILON.powf(1.0 / k as f64)).clamp(1e-5, 1e-2)
}

/// Agrupa polos repetidos: `(centro, multiplicidade)`. Para cada polo ainda livre, escolhe o
/// maior `k` tal que os `k` polos livres mais próximos dele estejam todos a no máximo
/// `cluster_tol(k)·max(|centro|, 1)` do centro (a média do grupo).
fn group_poles(poles: &[Complex64]) -> Vec<(Complex64, usize)> {
    let mut free: Vec<Complex64> = poles.to_vec();
    let mut groups = vec![];
    while let Some(&p) = free.first() {
        let mut order: Vec<usize> = (0..free.len()).collect();
        order.sort_by(|&i, &j| (free[i] - p).norm().total_cmp(&(free[j] - p).norm()));
        let mut best = (p, 1, vec![order[0]]);
        for k in 2..=free.len() {
            let members = &order[..k];
            let center = members.iter().map(|&i| free[i]).sum::<Complex64>() / k as f64;
            let tol = cluster_tol(k) * center.norm().max(1.0);
            if members.iter().all(|&i| (free[i] - center).norm() <= tol) {
                best = (center, k, members.to_vec());
            }
        }
        let (center, k, mut members) = best;
        groups.push((center, k));
        members.sort_unstable_by(|a, b| b.cmp(a));
        for i in members {
            free.remove(i);
        }
    }
    groups
}

/// Multiplica polinômio complexo por `(s − q)`.
fn mul_linear(p: &[Complex64], q: Complex64) -> Vec<Complex64> {
    let mut out = p.to_vec();
    out.push(Complex64::new(0.0, 0.0));
    for k in (1..out.len()).rev() {
        out[k] -= q * p[k - 1];
    }
    out
}

/// Primeiros `m` coeficientes de Taylor de `p(s)` em torno de `s0` (divisão sintética repetida).
fn taylor(p: &[Complex64], s0: Complex64, m: usize) -> Vec<Complex64> {
    let mut coeffs = p.to_vec();
    let mut out = Vec::with_capacity(m);
    for _ in 0..m {
        if coeffs.is_empty() {
            out.push(Complex64::new(0.0, 0.0));
            continue;
        }
        // Horner: quociente e resto da divisão por (s − s0).
        let mut q = Vec::with_capacity(coeffs.len().saturating_sub(1));
        let mut acc = Complex64::new(0.0, 0.0);
        for (i, &c) in coeffs.iter().enumerate() {
            acc = acc * s0 + c;
            if i + 1 < coeffs.len() {
                q.push(acc);
            }
        }
        out.push(acc);
        coeffs = q;
    }
    out
}

/// Divisão de séries de potências `n/d` até `m` termos (`d[0] ≠ 0`).
fn series_div(n: &[Complex64], d: &[Complex64], m: usize) -> Vec<Complex64> {
    let mut c = vec![Complex64::new(0.0, 0.0); m];
    for k in 0..m {
        let s: Complex64 = (1..=k)
            .map(|i| d.get(i).copied().unwrap_or_default() * c[k - i])
            .sum();
        c[k] = (n.get(k).copied().unwrap_or_default() - s) / d[0];
    }
    c
}

/// Reconstrói `B(s)/A(s)` a partir de uma expansão em frações parciais (inverso de [`residue`]).
/// Devolve `(b, a)` com `a` mônico. Partes imaginárias residuais são descartadas quando polos
/// e resíduos vêm em pares conjugados.
pub fn residue_inverse(r: &Residue) -> (Vec<f64>, Vec<f64>) {
    let (a_c, _) = poly(&r.poles);
    let mut b = vec![Complex64::new(0.0, 0.0); a_c.len()];
    // Para cada termo r/(s − p)ʲ: numerador = r · A(s)/(s − p)ʲ.
    let mut i = 0;
    while i < r.poles.len() {
        let p = r.poles[i];
        let mut m = 1;
        while i + m < r.poles.len() && (r.poles[i + m] - p).norm() <= 1e-12 * p.norm().max(1.0) {
            m += 1;
        }
        for j in 1..=m {
            let mut q = a_c.clone();
            for _ in 0..j {
                q = divide_linear(&q, p);
            }
            let off = b.len() - q.len();
            for (k, c) in q.iter().enumerate() {
                b[off + k] += r.residues[i + j - 1] * c;
            }
        }
        i += m;
    }
    let mut num: Vec<Complex64> = b;
    if !r.direct.is_empty() {
        let kd: Vec<Complex64> = r.direct.iter().map(|&c| Complex64::new(c, 0.0)).collect();
        let prod = mul_complex(&kd, &a_c);
        let off = prod.len() - num.len();
        let mut out = prod;
        for (k, c) in num.iter().enumerate() {
            out[off + k] += c;
        }
        num = out;
    }
    // Coeficientes líderes desprezíveis (≤ 1e-12 do maior) são resíduos de arredondamento.
    let mut num_r: Vec<f64> = num.iter().map(|z| z.re).collect();
    let scale = num_r.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    while num_r.len() > 1 && num_r[0].abs() <= 1e-12 * scale {
        num_r.remove(0);
    }
    (num_r, a_c.iter().map(|z| z.re).collect())
}

/// Divide polinômio complexo por `(s − p)` (descarta o resto, nulo quando `p` é raiz).
fn divide_linear(c: &[Complex64], p: Complex64) -> Vec<Complex64> {
    let mut q = Vec::with_capacity(c.len().saturating_sub(1));
    let mut acc = Complex64::new(0.0, 0.0);
    for &x in &c[..c.len() - 1] {
        acc = acc * p + x;
        q.push(acc);
    }
    q
}

fn mul_complex(a: &[Complex64], b: &[Complex64]) -> Vec<Complex64> {
    let mut out = vec![Complex64::new(0.0, 0.0); a.len() + b.len() - 1];
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            out[i + j] += x * y;
        }
    }
    out
}

/// Avalia o polinômio em uma matriz quadrada: `p(X) = p₀·Xⁿ + … + pₙ·I` (Horner matricial).
pub fn polyvalm(p: &[f64], x: &Matrix) -> Result<Matrix, LinalgError> {
    if !x.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "polyvalm",
            shape: x.shape(),
        });
    }
    let n = x.rows();
    let id = Matrix::identity(n);
    let mut acc = Matrix::zeros(n, n);
    for &c in p {
        acc = acc.matmul(x)?.add(&id.scale(c))?;
    }
    Ok(acc)
}

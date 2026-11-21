//! Interpolação (funcionalidade F044, RF-044; DT-27).
//!
//! [`Interpolant1`] constrói um interpolante 1-D reutilizável a partir de pontos (x, y); [`interp1`]
//! é o atalho para uma avaliação. Métodos ([`Method1`]):
//! - `Linear`, `Nearest`, `Previous`, `Next`;
//! - `Pchip`: Hermite cúbica que preserva a monotonia (Fritsch & Butland, 1984; Fritsch & Carlson,
//!   1980);
//! - `Akima` (Akima, 1970) e `Makima` (pesos modificados que evitam ultrapassagem em platôs);
//! - `Spline(end)`: spline cúbica C² com condição de extremo not-a-knot, natural, derivadas dadas
//!   ou periódica (de Boor, *A Practical Guide to Splines*, cap. IV).
//!
//! Os métodos cúbicos produzem um [`PPoly`], com derivada e integral exatas.
//! Fora do intervalo dos dados vale a política [`Extrap`], igual para todos os métodos.

mod ppoly;
mod scattered;

use std::fmt;

pub use ppoly::PPoly;
pub use scattered::{ScatteredInterpolant, ScatteredMethod};

/// Condição de extremo da spline cúbica.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SplineEnd {
    /// Terceira derivada contínua na segunda e na penúltima quebra (padrão).
    #[default]
    NotAKnot,
    /// Segunda derivada nula nos extremos.
    Natural,
    /// Primeiras derivadas dadas nos extremos.
    Clamped(f64, f64),
    /// Periódica: exige y₀ = y_{n−1}; derivadas até a segunda coincidem nos extremos.
    Periodic,
}

/// Método de interpolação 1-D.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Method1 {
    /// Linear por partes (padrão).
    #[default]
    Linear,
    /// Vizinho mais próximo; no ponto médio, o da direita.
    Nearest,
    /// Ponto anterior (degrau à esquerda).
    Previous,
    /// Ponto seguinte (degrau à direita).
    Next,
    /// Hermite cúbica monótona.
    Pchip,
    /// Akima (1970).
    Akima,
    /// Akima modificado.
    Makima,
    /// Spline cúbica.
    Spline(SplineEnd),
}

/// O que fazer fora de [x_min, x_max].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Extrap {
    /// NaN (padrão).
    #[default]
    Nan,
    /// Prolonga o método (último polinômio, último degrau ou reta extrema).
    Extrapolate,
    /// Valor constante.
    Value(f64),
    /// Valor no extremo mais próximo.
    Clamp,
}

/// Erros de interpolação.
#[derive(Clone, Debug, PartialEq)]
pub enum InterpError {
    /// Menos pontos que o método exige.
    TooFewPoints {
        /// Mínimo.
        needed: usize,
        /// Recebidos.
        got: usize,
    },
    /// x e y com tamanhos diferentes, ou forma incompatível.
    ShapeMismatch {
        /// Esperado.
        expected: usize,
        /// Recebido.
        actual: usize,
    },
    /// Abscissas repetidas.
    DuplicatePoints(f64),
    /// Abscissas não finitas, ou quebras fora de ordem.
    NotMonotonic,
    /// Spline periódica com y₀ ≠ y_{n−1}, ou condição de extremo não finita.
    InvalidOption(&'static str),
}

impl InterpError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::TooFewPoints { .. } => "chimalab:interp:pontosInsuficientes",
            Self::ShapeMismatch { .. } => "chimalab:interp:formaIncompativel",
            Self::DuplicatePoints(_) => "chimalab:interp:pontosDuplicados",
            Self::NotMonotonic => "chimalab:interp:naoMonotono",
            Self::InvalidOption(_) => "chimalab:interp:opcaoInvalida",
        }
    }
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewPoints { needed, got } => {
                write!(f, "pontos insuficientes: {got}; o método exige {needed}")
            }
            Self::ShapeMismatch { expected, actual } => {
                write!(f, "tamanhos incompatíveis: {actual}; esperado {expected}")
            }
            Self::DuplicatePoints(x) => write!(f, "abscissa repetida: {x}"),
            Self::NotMonotonic => write!(f, "abscissas não finitas ou fora de ordem"),
            Self::InvalidOption(m) => write!(f, "opção inválida: {m}"),
        }
    }
}

impl std::error::Error for InterpError {}

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    /// Linear, degraus: os próprios pontos.
    Table(Method1),
    /// Métodos cúbicos.
    Poly(PPoly),
}

/// Interpolante 1-D reutilizável.
#[derive(Clone, Debug, PartialEq)]
pub struct Interpolant1 {
    x: Vec<f64>,
    y: Vec<f64>,
    kind: Kind,
}

impl Interpolant1 {
    /// Constrói o interpolante. x é ordenado (com y) se preciso; x deve ser finito e sem
    /// repetições. y pode conter NaN (propaga localmente nos métodos locais).
    ///
    /// Mínimo de pontos: 1 para os degraus e o vizinho mais próximo, 2 para os demais (com 2
    /// pontos os cúbicos dão a reta; a spline not-a-knot com 3 dá a parábola).
    pub fn new(x: &[f64], y: &[f64], method: Method1) -> Result<Self, InterpError> {
        if x.len() != y.len() {
            return Err(InterpError::ShapeMismatch {
                expected: x.len(),
                actual: y.len(),
            });
        }
        let needed = match method {
            Method1::Nearest | Method1::Previous | Method1::Next => 1,
            _ => 2,
        };
        if x.len() < needed {
            return Err(InterpError::TooFewPoints { needed, got: x.len() });
        }
        if x.iter().any(|v| !v.is_finite()) {
            return Err(InterpError::NotMonotonic);
        }
        let mut idx: Vec<usize> = (0..x.len()).collect();
        idx.sort_by(|&a, &b| x[a].total_cmp(&x[b]));
        let xs: Vec<f64> = idx.iter().map(|&i| x[i]).collect();
        let ys: Vec<f64> = idx.iter().map(|&i| y[i]).collect();
        if let Some(w) = xs.windows(2).find(|w| w[0] == w[1]) {
            return Err(InterpError::DuplicatePoints(w[0]));
        }
        let kind = match method {
            Method1::Linear | Method1::Nearest | Method1::Previous | Method1::Next => Kind::Table(method),
            Method1::Pchip => Kind::Poly(hermite(&xs, &ys, &pchip_slopes(&xs, &ys))),
            Method1::Akima => Kind::Poly(hermite(&xs, &ys, &akima_slopes(&xs, &ys, false))),
            Method1::Makima => Kind::Poly(hermite(&xs, &ys, &akima_slopes(&xs, &ys, true))),
            Method1::Spline(end) => Kind::Poly(hermite(&xs, &ys, &spline_slopes(&xs, &ys, end)?)),
        };
        Ok(Self { x: xs, y: ys, kind })
    }

    /// Abscissas ordenadas.
    pub fn x(&self) -> &[f64] {
        &self.x
    }

    /// Ordenadas na ordem de [`Self::x`].
    pub fn y(&self) -> &[f64] {
        &self.y
    }

    /// Polinômio por partes (métodos cúbicos; `None` para linear e degraus).
    pub fn ppoly(&self) -> Option<&PPoly> {
        match &self.kind {
            Kind::Poly(p) => Some(p),
            Kind::Table(_) => None,
        }
    }

    /// Valor em `xq`.
    pub fn eval(&self, xq: f64, extrap: Extrap) -> f64 {
        if xq.is_nan() {
            return f64::NAN;
        }
        let (lo, hi) = (self.x[0], self.x[self.x.len() - 1]);
        let fora = xq < lo || xq > hi;
        if fora {
            match extrap {
                Extrap::Nan => return f64::NAN,
                Extrap::Value(v) => return v,
                Extrap::Clamp => return self.eval(xq.clamp(lo, hi), Extrap::Nan),
                Extrap::Extrapolate => {}
            }
        }
        // Nos nós, o valor dado exatamente (o polinômio avaliado no extremo do intervalo pode
        // errar por arredondamento).
        if let Ok(i) = self.x.binary_search_by(|v| v.total_cmp(&xq)) {
            return self.y[i];
        }
        match &self.kind {
            Kind::Poly(p) => p.eval(xq, Extrap::Extrapolate),
            Kind::Table(m) => self.eval_table(*m, xq),
        }
    }

    /// Valores em vários pontos.
    pub fn eval_many(&self, xq: &[f64], extrap: Extrap) -> Vec<f64> {
        xq.iter().map(|&v| self.eval(v, extrap)).collect()
    }

    fn eval_table(&self, m: Method1, xq: f64) -> f64 {
        let (x, y) = (&self.x, &self.y);
        let n = x.len();
        // Primeiro índice com x > xq.
        let j = x.partition_point(|&v| v <= xq);
        match m {
            Method1::Previous => y[j.saturating_sub(1)],
            Method1::Next => {
                // Os nós já foram tratados em `eval`: aqui xq está entre nós.
                y[j.min(n - 1)]
            }
            Method1::Nearest => {
                if j == 0 {
                    y[0]
                } else if j == n {
                    y[n - 1]
                } else if xq - x[j - 1] < x[j] - xq {
                    y[j - 1]
                } else {
                    y[j]
                }
            }
            _ => {
                // Linear: o segmento que contém xq (os extremos estendem a reta).
                let i = j.clamp(1, n - 1) - 1;
                let (x0, x1) = (x[i], x[i + 1]);
                let t = (xq - x0) / (x1 - x0);
                y[i] + t * (y[i + 1] - y[i])
            }
        }
    }
}

/// Interpola (x, y) em `xq` pelo método dado.
pub fn interp1(
    x: &[f64],
    y: &[f64],
    xq: &[f64],
    method: Method1,
    extrap: Extrap,
) -> Result<Vec<f64>, InterpError> {
    Ok(Interpolant1::new(x, y, method)?.eval_many(xq, extrap))
}

/// Polinômio de Hermite cúbico por partes a partir dos valores e das derivadas nos nós.
fn hermite(x: &[f64], y: &[f64], s: &[f64]) -> PPoly {
    let coefs = (0..x.len() - 1)
        .map(|i| {
            let h = x[i + 1] - x[i];
            let d = (y[i + 1] - y[i]) / h;
            vec![
                y[i],
                s[i],
                (3.0 * d - 2.0 * s[i] - s[i + 1]) / h,
                (s[i] + s[i + 1] - 2.0 * d) / (h * h),
            ]
        })
        .collect();
    PPoly::from_parts(x.to_vec(), coefs)
}

fn secants(x: &[f64], y: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let h: Vec<f64> = x.windows(2).map(|w| w[1] - w[0]).collect();
    let d: Vec<f64> = (0..h.len()).map(|i| (y[i + 1] - y[i]) / h[i]).collect();
    (h, d)
}

/// Derivadas do pchip: média harmônica ponderada de Fritsch & Butland nos nós internos (zero
/// em extremos locais) e fórmula de três pontos nos extremos, limitada para preservar a forma.
fn pchip_slopes(x: &[f64], y: &[f64]) -> Vec<f64> {
    let n = x.len();
    let (h, d) = secants(x, y);
    if n == 2 {
        return vec![d[0], d[0]];
    }
    let mut s = vec![0.0; n];
    for k in 1..n - 1 {
        // Mesmo sinal, comparado sem multiplicar (o produto pode sofrer underflow e simular um
        // extremo local).
        if d[k - 1] != 0.0 && d[k] != 0.0 && d[k - 1].is_sign_positive() == d[k].is_sign_positive() {
            let w1 = 2.0 * h[k] + h[k - 1];
            let w2 = h[k] + 2.0 * h[k - 1];
            s[k] = (w1 + w2) / (w1 / d[k - 1] + w2 / d[k]);
        }
    }
    let end = |h0: f64, h1: f64, d0: f64, d1: f64| -> f64 {
        let v = ((2.0 * h0 + h1) * d0 - h0 * d1) / (h0 + h1);
        if v.signum() != d0.signum() || d0 == 0.0 {
            0.0
        } else if d0.signum() != d1.signum() && v.abs() > 3.0 * d0.abs() {
            3.0 * d0
        } else {
            v
        }
    };
    s[0] = end(h[0], h[1], d[0], d[1]);
    s[n - 1] = end(h[n - 2], h[n - 3], d[n - 2], d[n - 3]);
    s
}

/// Derivadas de Akima: média das secantes vizinhas ponderada pelas diferenças das secantes
/// do outro lado; nos extremos, secantes fictícias por extrapolação quadrática (Akima, 1970).
/// Modificado: wᵢ = |δᵢ₊₁ − δᵢ| + |δᵢ₊₁ + δᵢ|/2, que zera a derivada em platôs.
fn akima_slopes(x: &[f64], y: &[f64], modified: bool) -> Vec<f64> {
    let n = x.len();
    let (_, d) = secants(x, y);
    if n == 2 {
        return vec![d[0], d[0]];
    }
    // m[k + 2] = δ_k, com dois valores fictícios em cada extremo.
    let m_ = n - 1;
    let mut m = vec![0.0; m_ + 4];
    m[2..m_ + 2].copy_from_slice(&d);
    m[1] = 2.0 * m[2] - m[3];
    m[0] = 2.0 * m[1] - m[2];
    m[m_ + 2] = 2.0 * m[m_ + 1] - m[m_];
    m[m_ + 3] = 2.0 * m[m_ + 2] - m[m_ + 1];
    (0..n)
        .map(|i| {
            // Secantes em volta do nó i: m[i], m[i+1] | m[i+2], m[i+3].
            let (a, b, c, e) = (m[i], m[i + 1], m[i + 2], m[i + 3]);
            let (w1, w2) = if modified {
                (
                    (e - c).abs() + (e + c).abs() / 2.0,
                    (b - a).abs() + (b + a).abs() / 2.0,
                )
            } else {
                ((e - c).abs(), (b - a).abs())
            };
            let w = w1 + w2;
            if w == 0.0 {
                (b + c) / 2.0
            } else {
                // Pesos normalizados antes de multiplicar: w·δ sofreria underflow em escalas
                // pequenas (|δ| ~ 1e-230 dá w·δ ~ 1e-460).
                (w1 / w) * b + (w2 / w) * c
            }
        })
        .collect()
}

/// Derivadas da spline cúbica C²: sistema tridiagonal nas derivadas sᵢ (de Boor, cap. IV):
/// hᵢ·sᵢ₋₁ + 2(hᵢ₋₁ + hᵢ)·sᵢ + hᵢ₋₁·sᵢ₊₁ = 3(hᵢ·δᵢ₋₁ + hᵢ₋₁·δᵢ), mais as linhas dos extremos.
fn spline_slopes(x: &[f64], y: &[f64], end: SplineEnd) -> Result<Vec<f64>, InterpError> {
    let n = x.len();
    // Resolve com x normalizado pelo comprimento H do intervalo e converte as derivadas no fim
    // (s = s_u/H): os termos h²·δ do sistema sofreriam underflow ou overflow com abscissas em
    // escalas extremas.
    let big_h = x[n - 1] - x[0];
    let (h, d): (Vec<f64>, Vec<f64>) = {
        let h: Vec<f64> = x.windows(2).map(|w| (w[1] - w[0]) / big_h).collect();
        let d = (0..n - 1).map(|i| (y[i + 1] - y[i]) / h[i]).collect();
        (h, d)
    };
    let end = match end {
        SplineEnd::Clamped(a, b) => SplineEnd::Clamped(a * big_h, b * big_h),
        e => e,
    };
    let s = spline_slopes_unit(&h, &d, y, end)?;
    Ok(s.into_iter().map(|v| v / big_h).collect())
}

fn spline_slopes_unit(h: &[f64], d: &[f64], y: &[f64], end: SplineEnd) -> Result<Vec<f64>, InterpError> {
    let n = y.len();
    if let SplineEnd::Clamped(a, b) = end
        && !(a.is_finite() && b.is_finite())
    {
        return Err(InterpError::InvalidOption("derivadas de extremo não finitas"));
    }
    if end == SplineEnd::Periodic {
        if y[0] != y[n - 1] {
            return Err(InterpError::InvalidOption("spline periódica exige y₀ = y_{n−1}"));
        }
        if n == 2 {
            return Ok(vec![0.0, 0.0]);
        }
    }
    if n == 2 {
        return Ok(match end {
            SplineEnd::Clamped(a, b) => vec![a, b],
            _ => vec![d[0], d[0]],
        });
    }
    if n == 3 && end == SplineEnd::NotAKnot {
        // Not-a-knot com 3 pontos: a parábola interpoladora.
        let c = (d[1] - d[0]) / (h[0] + h[1]);
        return Ok(vec![d[0] - c * h[0], d[0] + c * h[0], d[1] + c * h[1]]);
    }
    let mut sub = vec![0.0; n];
    let mut diag = vec![0.0; n];
    let mut sup = vec![0.0; n];
    let mut rhs = vec![0.0; n];
    for i in 1..n - 1 {
        sub[i] = h[i];
        diag[i] = 2.0 * (h[i - 1] + h[i]);
        sup[i] = h[i - 1];
        rhs[i] = 3.0 * (h[i] * d[i - 1] + h[i - 1] * d[i]);
    }
    match end {
        SplineEnd::NotAKnot => {
            let (h0, h1) = (h[0], h[1]);
            diag[0] = h1;
            sup[0] = h0 + h1;
            rhs[0] = ((h0 + 2.0 * (h0 + h1)) * h1 * d[0] + h0 * h0 * d[1]) / (h0 + h1);
            let (a, b) = (h[n - 2], h[n - 3]);
            sub[n - 1] = a + b;
            diag[n - 1] = b;
            rhs[n - 1] = ((a + 2.0 * (a + b)) * b * d[n - 2] + a * a * d[n - 3]) / (a + b);
        }
        SplineEnd::Natural => {
            diag[0] = 2.0;
            sup[0] = 1.0;
            rhs[0] = 3.0 * d[0];
            sub[n - 1] = 1.0;
            diag[n - 1] = 2.0;
            rhs[n - 1] = 3.0 * d[n - 2];
        }
        SplineEnd::Clamped(a, b) => {
            diag[0] = 1.0;
            rhs[0] = a;
            diag[n - 1] = 1.0;
            rhs[n - 1] = b;
        }
        SplineEnd::Periodic => return Ok(periodic_slopes(h, d)),
    }
    Ok(tridiagonal(sub, diag, sup, rhs))
}

/// Sistema tridiagonal por eliminação de Gauss com pivotamento parcial (o pivotamento troca
/// linhas adjacentes e cria um segundo superdiagonal). `sub[0]` e `sup[n−1]` são ignorados.
fn tridiagonal(sub: Vec<f64>, diag: Vec<f64>, sup: Vec<f64>, rhs: Vec<f64>) -> Vec<f64> {
    let n = diag.len();
    let (mut b, mut c, mut r) = (diag, sup, rhs);
    let a = sub;
    // c[n−1] não existe na matriz; zerado, as escritas nele (e o e[n−2] que dele vem) nunca são
    // lidas, e o laço dispensa casos especiais na última linha.
    c[n - 1] = 0.0;
    let mut e = vec![0.0; n]; // segundo superdiagonal criado pelas trocas
    for i in 0..n - 1 {
        // Linha i: [b_i, c_i, e_i]; linha i+1: [a_{i+1}, b_{i+1}, c_{i+1}].
        let mut ai = a[i + 1];
        if ai.abs() > b[i].abs() {
            // Troca as linhas i e i+1.
            std::mem::swap(&mut b[i], &mut ai);
            std::mem::swap(&mut c[i], &mut b[i + 1]);
            std::mem::swap(&mut e[i], &mut c[i + 1]);
            r.swap(i, i + 1);
        }
        let f = ai / b[i];
        b[i + 1] -= f * c[i];
        c[i + 1] -= f * e[i];
        r[i + 1] -= f * r[i];
    }
    let mut s = vec![0.0; n];
    for i in (0..n).rev() {
        let mut v = r[i];
        if i + 1 < n {
            v -= c[i] * s[i + 1];
        }
        if i + 2 < n {
            v -= e[i] * s[i + 2];
        }
        s[i] = v / b[i];
    }
    s
}

/// Spline periódica: sistema cíclico em s₀..s_{n−2} (s_{n−1} = s₀), resolvido por
/// Sherman–Morrison sobre o tridiagonal.
fn periodic_slopes(h: &[f64], d: &[f64]) -> Vec<f64> {
    let m = h.len(); // número de incógnitas
    // Linha i (i = 0..m−1), com índices cíclicos: h_i s_{i−1} + 2(h_{i−1}+h_i) s_i + h_{i−1} s_{i+1}.
    let hm = |i: usize| h[(i + m - 1) % m];
    let dm = |i: usize| d[(i + m - 1) % m];
    let mut sub = vec![0.0; m];
    let mut diag = vec![0.0; m];
    let mut sup = vec![0.0; m];
    let mut rhs = vec![0.0; m];
    for i in 0..m {
        sub[i] = h[i];
        diag[i] = 2.0 * (hm(i) + h[i]);
        sup[i] = hm(i);
        rhs[i] = 3.0 * (h[i] * dm(i) + hm(i) * d[i]);
    }
    // m ≥ 2 (n ≥ 3 pontos; n = 2 é tratado antes).
    let mut s = if m == 2 {
        // Matriz 2×2 cheia: [[b0, a0 + c0], [a1 + c1, b1]].
        let (p, q, r_, t) = (diag[0], sub[0] + sup[0], sub[1] + sup[1], diag[1]);
        let det = p * t - q * r_;
        vec![(rhs[0] * t - q * rhs[1]) / det, (p * rhs[1] - r_ * rhs[0]) / det]
    } else {
        // A = T + u vᵀ com u = (γ, 0, …, 0, a₀)ᵀ... forma de Sherman–Morrison com γ = −b₀.
        let alpha = sub[0]; // A[0][m−1]
        let beta = sup[m - 1]; // A[m−1][0]
        let gamma = -diag[0];
        let mut bb = diag.clone();
        bb[0] -= gamma;
        bb[m - 1] -= alpha * beta / gamma;
        let x = tridiagonal(sub.clone(), bb.clone(), sup.clone(), rhs.clone());
        let mut u = vec![0.0; m];
        u[0] = gamma;
        u[m - 1] = beta;
        let z = tridiagonal(sub, bb, sup, u);
        let fact = (x[0] + alpha * x[m - 1] / gamma) / (1.0 + z[0] + alpha * z[m - 1] / gamma);
        x.iter().zip(&z).map(|(xi, zi)| xi - fact * zi).collect()
    };
    s.push(s[0]);
    s
}

/// Método em grades N-D.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MethodN {
    /// Multilinear (padrão): combinação dos 2ᴺ vértices da célula.
    #[default]
    Linear,
    /// Vértice mais próximo (no ponto médio, o de índice maior).
    Nearest,
    /// Produto tensorial de um método cúbico 1-D (pchip, Akima, makima ou spline), aplicado
    /// dimensão a dimensão.
    Tensor(Method1),
}

/// Grade retilínea N-D com valores column-major: o índice da dimensão 1 varia mais rápido.
#[derive(Clone, Debug, PartialEq)]
pub struct GridInterpolant {
    axes: Vec<Vec<f64>>,
    values: Vec<f64>,
    method: MethodN,
}

impl GridInterpolant {
    /// `axes[d]`: abscissas estritamente crescentes da dimensão d; `values`: Π len(axes[d])
    /// valores em ordem column-major.
    ///
    /// Erros: `formaIncompativel` (número de valores), `pontosInsuficientes` (eixo com menos de 2
    /// pontos, ou nenhum eixo), `naoMonotono` (eixo não estritamente crescente ou não finito),
    /// `opcaoInvalida` (tensor com método não cúbico).
    pub fn new(axes: Vec<Vec<f64>>, values: Vec<f64>, method: MethodN) -> Result<Self, InterpError> {
        if axes.is_empty() {
            return Err(InterpError::TooFewPoints { needed: 1, got: 0 });
        }
        for a in &axes {
            if a.len() < 2 {
                return Err(InterpError::TooFewPoints {
                    needed: 2,
                    got: a.len(),
                });
            }
            if a.iter().any(|v| !v.is_finite()) || a.windows(2).any(|w| w[1] <= w[0]) {
                return Err(InterpError::NotMonotonic);
            }
        }
        let total: usize = axes.iter().map(Vec::len).product();
        if values.len() != total {
            return Err(InterpError::ShapeMismatch {
                expected: total,
                actual: values.len(),
            });
        }
        if let MethodN::Tensor(m) = method
            && matches!(
                m,
                Method1::Linear | Method1::Nearest | Method1::Previous | Method1::Next
            )
        {
            return Err(InterpError::InvalidOption(
                "tensor exige método cúbico; use Linear ou Nearest",
            ));
        }
        Ok(Self { axes, values, method })
    }

    /// Número de dimensões.
    pub fn ndims(&self) -> usize {
        self.axes.len()
    }

    /// Valor no ponto `p` (N coordenadas). Fora da grade, segue `extrap` (`Extrapolate`
    /// prolonga as células ou os polinômios extremos de cada dimensão).
    ///
    /// # Panics
    /// Se `p.len()` ≠ número de dimensões.
    pub fn eval(&self, p: &[f64], extrap: Extrap) -> f64 {
        assert_eq!(p.len(), self.axes.len(), "ponto com dimensão errada");
        if p.iter().any(|v| v.is_nan()) {
            return f64::NAN;
        }
        let fora = p
            .iter()
            .zip(&self.axes)
            .any(|(v, a)| *v < a[0] || *v > a[a.len() - 1]);
        let mut q = p.to_vec();
        if fora {
            match extrap {
                Extrap::Nan => return f64::NAN,
                Extrap::Value(v) => return v,
                Extrap::Clamp => {
                    for (v, a) in q.iter_mut().zip(&self.axes) {
                        *v = v.clamp(a[0], a[a.len() - 1]);
                    }
                }
                Extrap::Extrapolate => {}
            }
        }
        match self.method {
            MethodN::Linear => self.multilinear(&q),
            MethodN::Nearest => self.nearest(&q),
            MethodN::Tensor(m) => self.tensor(&q, m),
        }
    }

    fn strides(&self) -> Vec<usize> {
        let mut s = vec![1usize; self.axes.len()];
        for d in 1..self.axes.len() {
            s[d] = s[d - 1] * self.axes[d - 1].len();
        }
        s
    }

    fn multilinear(&self, q: &[f64]) -> f64 {
        let nd = self.axes.len();
        let st = self.strides();
        // Célula e peso por dimensão.
        let mut base = 0usize;
        let mut w = vec![0.0; nd];
        for d in 0..nd {
            let a = &self.axes[d];
            let i = a.partition_point(|&v| v <= q[d]).clamp(1, a.len() - 1) - 1;
            w[d] = (q[d] - a[i]) / (a[i + 1] - a[i]);
            base += i * st[d];
        }
        let mut acc = 0.0;
        for corner in 0..(1usize << nd) {
            let mut idx = base;
            let mut peso = 1.0;
            for d in 0..nd {
                if corner >> d & 1 == 1 {
                    idx += st[d];
                    peso *= w[d];
                } else {
                    peso *= 1.0 - w[d];
                }
            }
            // Peso nulo não contamina com NaN de vértices que não contam (nós exatos).
            if peso != 0.0 {
                acc += peso * self.values[idx];
            }
        }
        acc
    }

    fn nearest(&self, q: &[f64]) -> f64 {
        let st = self.strides();
        let mut idx = 0usize;
        for (d, a) in self.axes.iter().enumerate() {
            let j = a.partition_point(|&v| v <= q[d]);
            let i = if j == 0 {
                0
            } else if j == a.len() || q[d] - a[j - 1] < a[j] - q[d] {
                j - 1
            } else {
                j
            };
            idx += i * st[d];
        }
        self.values[idx]
    }

    /// Produto tensorial: interpola ao longo da dimensão 1 em cada linha, depois da 2 nos
    /// resultados, e assim por diante (o resultado não depende da ordem).
    fn tensor(&self, q: &[f64], m: Method1) -> f64 {
        let mut vals = self.values.clone();
        for (d, a) in self.axes.iter().enumerate() {
            let n = a.len();
            let linhas = vals.len() / n;
            // Column-major com a dimensão d agora a mais rápida.
            let mut prox = Vec::with_capacity(linhas);
            for l in 0..linhas {
                let y: Vec<f64> = (0..n).map(|k| vals[l * n + k]).collect();
                let it = Interpolant1::new(a, &y, m).expect("eixo validado no construtor");
                prox.push(it.eval(q[d], Extrap::Extrapolate));
            }
            vals = prox;
        }
        vals[0]
    }
}

#[cfg(test)]
mod testes_tridiagonal {
    use super::tridiagonal;

    /// Eliminação de Gauss densa com pivotamento parcial (oráculo independente).
    fn denso(a: &mut [Vec<f64>], mut b: Vec<f64>) -> Vec<f64> {
        let n = b.len();
        for k in 0..n {
            let p = (k..n)
                .max_by(|&i, &j| a[i][k].abs().total_cmp(&a[j][k].abs()))
                .unwrap();
            a.swap(k, p);
            b.swap(k, p);
            for i in k + 1..n {
                let f = a[i][k] / a[k][k];
                for j in k..n {
                    a[i][j] -= f * a[k][j];
                }
                b[i] -= f * b[k];
            }
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let s: f64 = (i + 1..n).map(|j| a[i][j] * x[j]).sum();
            x[i] = (b[i] - s) / a[i][i];
        }
        x
    }

    /// Sistemas tridiagonais não dominantes (o pivotamento troca linhas e cria o segundo
    /// superdiagonal), n de 2 a 9, contra a eliminação densa.
    #[test]
    fn tridiagonal_com_pivotamento_contra_denso() {
        let mut s = 7u64;
        let mut u = || {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (s >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        };
        let mut trocas = 0;
        for caso in 0..400 {
            let n = 2 + caso % 8;
            let sub: Vec<f64> = (0..n).map(|i| if i == 0 { 0.0 } else { 3.0 * u() }).collect();
            // Zeros exatos na diagonal: sem pivotamento, a eliminação divide por zero.
            let diag: Vec<f64> = (0..n)
                .map(|i| {
                    let v = 0.2 * u() + if caso % 3 == 0 { 2.0 } else { 0.0 };
                    if (i + caso) % 3 == 1 { 0.0 } else { v }
                })
                .collect();
            let sup: Vec<f64> = (0..n).map(|i| if i + 1 == n { 0.0 } else { 3.0 * u() }).collect();
            let rhs: Vec<f64> = (0..n).map(|_| u()).collect();
            trocas += (1..n).filter(|&i| sub[i].abs() > diag[i - 1].abs()).count();
            let mut a = vec![vec![0.0; n]; n];
            for i in 0..n {
                a[i][i] = diag[i];
                if i > 0 {
                    a[i][i - 1] = sub[i];
                }
                if i + 1 < n {
                    a[i][i + 1] = sup[i];
                }
            }
            let esp = denso(&mut a, rhs.clone());
            let x = tridiagonal(sub, diag, sup, rhs);
            let escala = esp.iter().fold(1.0f64, |m, v| m.max(v.abs()));
            for i in 0..n {
                assert!(
                    (x[i] - esp[i]).abs() <= 1e-11 * escala,
                    "caso {caso} n={n}: {x:?} vs {esp:?}"
                );
            }
        }
        assert!(trocas > 500, "{trocas}");
    }
}

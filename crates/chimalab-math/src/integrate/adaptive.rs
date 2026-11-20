//! Quadratura adaptativa global de Gauss–Kronrod 7–15.
//!
//! O intervalo [a, b] é cortado nos pontos de quebra. Cada trecho é levado a um intervalo finito
//! do parâmetro t por uma mudança de variável (Shampine, 2008):
//! - finito [p, q]: x = (q − p)/4·t(3 − t²) + (q + p)/2, t ∈ [−1, 1]; dx/dt = 3(q − p)/4·(1 − t²)
//!   se anula nos extremos e enfraquece singularidades integráveis ali. x é calculado a partir
//!   do extremo mais próximo, para não perder precisão perto dele;
//! - [p, +∞): x = p + t/(1 − t); (−∞, q]: x = q − t/(1 − t), t ∈ [0, 1);
//! - (−∞, +∞): x = t/(1 − t²), t ∈ (−1, 1).
//!
//! Cada trecho começa com 10 subintervalos em t. A cada passo o subintervalo de maior erro
//! estimado (|Kronrod − Gauss|) é dividido ao meio, até que a soma dos erros fique abaixo de
//! max(AbsTol, RelTol·|Q|) ou que se atinja o número máximo de subintervalos. Os extremos dos
//! subintervalos nunca são avaliados.

use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use chimalab_array::Scalar;

use super::IntegrateError;
use super::gk::gk15;

/// Subintervalos iniciais por trecho.
const INITIAL_PANELS: usize = 10;

/// Opções de [`integral`].
#[derive(Clone, Debug, PartialEq)]
pub struct IntegralOptions {
    /// Tolerância absoluta (≥ 0). Padrão 1e-10.
    pub abs_tol: f64,
    /// Tolerância relativa (≥ 0). Padrão 1e-6.
    pub rel_tol: f64,
    /// Número máximo de subintervalos (≥ 1). Padrão 650.
    pub max_intervals: usize,
    /// Pontos de quebra (descontinuidades, singularidades interiores). Os que caem fora de
    /// (a, b) são ignorados.
    pub waypoints: Vec<f64>,
}

impl Default for IntegralOptions {
    fn default() -> Self {
        Self {
            abs_tol: 1e-10,
            rel_tol: 1e-6,
            max_intervals: 650,
            waypoints: vec![],
        }
    }
}

/// Aviso: o resultado foi devolvido, mas sem garantia da tolerância pedida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegralWarning {
    /// Número máximo de subintervalos atingido.
    MaxIntervals,
    /// O refinamento chegou ao limite de resolução do ponto flutuante: um subintervalo não pode
    /// mais ser dividido, ou os nós caíram sobre um extremo (integrando não integrável ou com
    /// singularidade forte demais, como x^{−0,9} em 0).
    MinimumStep,
    /// O integrando devolveu Inf ou NaN.
    NonFinite,
}

impl IntegralWarning {
    /// Identificador estável.
    pub fn id(self) -> &'static str {
        match self {
            Self::MaxIntervals => "chimalab:integral:limiteDeIntervalos",
            Self::MinimumStep => "chimalab:integral:passoMinimo",
            Self::NonFinite => "chimalab:integral:valorNaoFinito",
        }
    }
}

/// Resultado de [`integral`].
#[derive(Clone, Debug, PartialEq)]
pub struct IntegralResult<T> {
    /// Valor aproximado da integral.
    pub value: T,
    /// Estimativa do erro absoluto (soma de |Kronrod − Gauss| dos subintervalos). Em integrandos
    /// singulares, o arredondamento de x perto da singularidade pode fazer o erro real passar um
    /// pouco desta estimativa.
    pub error_bound: f64,
    /// Avaliações do integrando.
    pub evaluations: usize,
    /// Aviso, se a tolerância não foi garantida.
    pub warning: Option<IntegralWarning>,
}

/// Mudança de variável de um trecho.
#[derive(Clone, Copy, Debug)]
enum Map {
    /// [p, q] finito.
    Finite { lo: f64, hi: f64, quarter: f64 },
    /// [p, +∞).
    Upper(f64),
    /// (−∞, q].
    Lower(f64),
    /// (−∞, +∞).
    Whole,
}

impl Map {
    /// Intervalo do parâmetro t.
    fn domain(self) -> (f64, f64) {
        match self {
            Map::Finite { .. } | Map::Whole => (-1.0, 1.0),
            Map::Upper(_) | Map::Lower(_) => (0.0, 1.0),
        }
    }

    /// (x(t), dx/dt).
    fn apply(self, t: f64) -> (f64, f64) {
        match self {
            Map::Finite { lo, hi, quarter } => {
                // x − p = q(1+t)²(2−t) e q − x = q(1−t)²(2+t), com q = (hi − lo)/4: a partir do
                // extremo mais próximo, x perto de um extremo singular não perde precisão (e não
                // arredonda para o próprio extremo, onde o integrando pode ser infinito).
                let (u, v) = (1.0 + t, 1.0 - t);
                let x = if t < 0.0 {
                    lo + quarter * u * u * (2.0 - t)
                } else {
                    hi - quarter * v * v * (2.0 + t)
                };
                (x, 3.0 * quarter * u * v)
            }
            Map::Upper(p) => {
                let u = 1.0 - t;
                (p + t / u, 1.0 / (u * u))
            }
            Map::Lower(q) => {
                let u = 1.0 - t;
                (q - t / u, 1.0 / (u * u))
            }
            Map::Whole => {
                let u = 1.0 - t * t;
                (t / u, (1.0 + t * t) / (u * u))
            }
        }
    }
}

/// Subintervalo em t com sua estimativa.
struct Panel<T> {
    piece: usize,
    t0: f64,
    t1: f64,
    value: T,
    error: f64,
}

impl<T> PartialEq for Panel<T> {
    fn eq(&self, other: &Self) -> bool {
        self.error.total_cmp(&other.error) == Ordering::Equal
    }
}
impl<T> Eq for Panel<T> {}
impl<T> PartialOrd for Panel<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Panel<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.error.total_cmp(&other.error)
    }
}

fn check_options(opts: &IntegralOptions) -> Result<(), IntegrateError> {
    let ok = |v: f64| v.is_finite() && v >= 0.0;
    if !ok(opts.abs_tol) {
        return Err(IntegrateError::InvalidTolerance("abs_tol deve ser finita e ≥ 0"));
    }
    if !ok(opts.rel_tol) {
        return Err(IntegrateError::InvalidTolerance("rel_tol deve ser finita e ≥ 0"));
    }
    if opts.max_intervals == 0 {
        return Err(IntegrateError::InvalidTolerance("max_intervals deve ser ≥ 1"));
    }
    if opts.waypoints.iter().any(|w| !w.is_finite()) {
        return Err(IntegrateError::InvalidLimits(
            "pontos de quebra devem ser finitos",
        ));
    }
    Ok(())
}

/// Trechos de [lo, hi] (lo < hi) cortados nos pontos de quebra interiores.
fn pieces(lo: f64, hi: f64, waypoints: &[f64]) -> Vec<Map> {
    let mut cuts: Vec<f64> = waypoints.iter().copied().filter(|&w| w > lo && w < hi).collect();
    cuts.sort_by(f64::total_cmp);
    cuts.dedup();
    let mut ends = vec![lo];
    ends.extend(cuts);
    ends.push(hi);
    ends.windows(2)
        .map(|w| match (w[0].is_finite(), w[1].is_finite()) {
            (true, true) => Map::Finite {
                lo: w[0],
                hi: w[1],
                quarter: 0.25 * (w[1] - w[0]),
            },
            (true, false) => Map::Upper(w[0]),
            (false, true) => Map::Lower(w[1]),
            (false, false) => Map::Whole,
        })
        .collect()
}

/// ∫ₐᵇ f(x) dx por quadratura adaptativa global (ver o cabeçalho do módulo). Aceita limites
/// infinitos, a > b (o sinal se inverte) e integrandos reais ou complexos.
///
/// Erros: limite NaN (`limitesInvalidos`), tolerâncias inválidas (`toleranciaInvalida`).
/// Quando a tolerância não é alcançada, o valor vem com um [`IntegralWarning`].
pub fn integral<T: Scalar>(
    mut f: impl FnMut(f64) -> T,
    a: f64,
    b: f64,
    opts: &IntegralOptions,
) -> Result<IntegralResult<T>, IntegrateError> {
    if a.is_nan() || b.is_nan() {
        return Err(IntegrateError::InvalidLimits("limite NaN"));
    }
    check_options(opts)?;
    if a > b {
        let mut r = integral(f, b, a, opts)?;
        r.value = -r.value;
        return Ok(r);
    }
    if a == b {
        return Ok(IntegralResult {
            value: T::zero(),
            error_bound: 0.0,
            evaluations: 0,
            warning: None,
        });
    }
    let maps = pieces(a, b, &opts.waypoints);
    let mut evaluations = 0;
    let mut warning = None;
    let mut heap = BinaryHeap::new();
    // Marca se algum nó chegou ao limite de resolução do parâmetro t (ver abaixo).
    let resolution_limit = Cell::new(false);
    let mut eval_panel = |piece: usize, t0: f64, t1: f64, evaluations: &mut usize| {
        let map = maps[piece];
        let mut g = |t: f64| {
            let (x, dx) = map.apply(t);
            // Perto de t = ±1 nos intervalos infinitos, x pode virar ±Inf por arredondamento; e,
            // num subintervalo minúsculo junto a um extremo finito, um nó pode cair no próprio
            // extremo, onde dx/dt = 0 e o integrando pode ser infinito. O ponto é descartado (sem
            // 0·Inf = NaN), mas o refinamento só chega aqui quando a massa da integral perto do
            // extremo não é resolúvel em ponto flutuante (integrando não integrável ou com
            // singularidade forte demais): o resultado sai com o aviso `MinimumStep`.
            // (Com dx finito, x também é: nos mapas infinitos |x| ≤ dx.)
            if dx == 0.0 || !dx.is_finite() {
                resolution_limit.set(true);
                return T::zero();
            }
            f(x).scale(dx)
        };
        let (k, gauss) = gk15(&mut g, t0, t1);
        *evaluations += 15;
        Panel {
            piece,
            t0,
            t1,
            value: k,
            error: (k - gauss).abs(),
        }
    };
    for (piece, map) in maps.iter().enumerate() {
        let (lo, hi) = map.domain();
        let step = (hi - lo) / INITIAL_PANELS as f64;
        for i in 0..INITIAL_PANELS {
            let t0 = lo + step * i as f64;
            // Nos dois domínios ([−1, 1] e [0, 1]) step·10 dá o extremo exato.
            let t1 = lo + step * (i + 1) as f64;
            heap.push(eval_panel(piece, t0, t1, &mut evaluations));
        }
    }
    loop {
        let (q, e) = totals(&heap);
        if !all_finite(q.abs(), e) {
            warning = Some(IntegralWarning::NonFinite);
            break;
        }
        if e <= opts.abs_tol.max(opts.rel_tol * q.abs()) {
            break;
        }
        if heap.len() >= opts.max_intervals {
            warning = Some(IntegralWarning::MaxIntervals);
            break;
        }
        let worst = heap.pop().expect("há subintervalos");
        let mid = 0.5 * (worst.t0 + worst.t1);
        if mid <= worst.t0 || mid >= worst.t1 {
            heap.push(worst);
            warning = Some(IntegralWarning::MinimumStep);
            break;
        }
        heap.push(eval_panel(worst.piece, worst.t0, mid, &mut evaluations));
        heap.push(eval_panel(worst.piece, mid, worst.t1, &mut evaluations));
    }
    // O limite de resolução é o diagnóstico mais específico: prevalece sobre o de subintervalos.
    if warning != Some(IntegralWarning::NonFinite) && resolution_limit.get() {
        warning = Some(IntegralWarning::MinimumStep);
    }
    let (value, error_bound) = totals(&heap);
    Ok(IntegralResult {
        value,
        error_bound,
        evaluations,
        warning,
    })
}

/// |Q| e E finitos. A soma é finita exatamente quando as duas parcelas são (Inf − Inf e NaN dão
/// NaN), de modo que trocar `+` por `−` ou `·` dá o mesmo veredito (mutantes equivalentes
/// registrados em `.cargo/mutants.toml`).
pub(super) fn all_finite(q_abs: f64, e: f64) -> bool {
    (q_abs + e).is_finite()
}

/// (Σ valores, Σ erros), somados em ordem crescente de erro para reduzir o arredondamento.
fn totals<T: Scalar>(heap: &BinaryHeap<Panel<T>>) -> (T, f64) {
    let mut panels: Vec<&Panel<T>> = heap.iter().collect();
    panels.sort_by(|p, q| p.error.total_cmp(&q.error));
    let mut v = T::zero();
    let mut e = 0.0;
    for p in panels {
        v += p.value;
        e += p.error;
    }
    (v, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subintervalos_ordenados_pelo_erro() {
        let p = |e: f64| Panel {
            piece: 0,
            t0: 0.0,
            t1: 1.0,
            value: 0.0,
            error: e,
        };
        assert!(p(1.0) == p(1.0) && p(1.0) != p(2.0));
        assert!(p(1.0) < p(2.0) && p(f64::NAN) > p(f64::INFINITY));
        assert_eq!(p(3.0).partial_cmp(&p(2.0)), Some(Ordering::Greater));
    }
}

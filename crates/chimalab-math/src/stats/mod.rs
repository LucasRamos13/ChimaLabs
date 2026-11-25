//! Estatística descritiva e pré-processamento de dados (funcionalidade F043, RF-043; DT-29).
//!
//! As funções operam em vetores. [`along_dim`] as aplica ao longo de uma dimensão de um array N-D
//! column-major. NaN é incluído (propaga) ou omitido conforme [`NanFlag`].
//!
//! - Resumos: [`sum`], [`prod`], [`mean`], [`median`], [`mode`], [`var`], [`std`], [`min`],
//!   [`max`], [`range`], [`quantile`], [`prctile`].
//! - Cumulativos: [`cumsum`], [`cumprod`], [`cummax`], [`cummin`].
//! - Janelas móveis: [`moving`] com [`Window`], [`Endpoints`] e [`MovingStat`].
//! - Preparação: [`normalize`], [`rescale`], [`isoutlier`], [`rmoutliers`], [`filloutliers`],
//!   [`histcounts`], [`histcounts_auto`].
//! - Suavização e detecção: [`smoothdata`], [`local_extrema`] (proeminência), [`gesd`]
//!   (Grubbs/GESD).

mod change;
mod moving;
mod prep;
mod smooth;
mod weighted;

use std::fmt;

pub use change::{ChangeKind, changepoints, changepoints_exact};
pub use moving::{Endpoints, MovingStat, Window, moving};
pub use prep::{
    BinRule, CenterBy, FillMethod, NormalizeMethod, OutlierBounds, OutlierMethod, ScaleBy, filloutliers,
    histcounts, histcounts_auto, isoutlier, normalize, rescale, rmoutliers,
};
pub use smooth::{SmoothMethod, gesd, local_extrema, smoothdata};
pub use weighted::{WeightNorm, mean_weighted, median_weighted, std_weighted, var_weighted};

/// Tratamento de NaN.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NanFlag {
    /// NaN propaga (padrão).
    #[default]
    Include,
    /// NaN é ignorado.
    Omit,
}

/// Normalização da variância.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VarNorm {
    /// Divide por n − 1 (amostral, não enviesada; padrão).
    #[default]
    Sample,
    /// Divide por n (populacional; segundo momento).
    Population,
}

/// Tipo de quantil de Hyndman & Fan (Am. Stat. 50, 1996), de 1 a 9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuantileType(u8);

impl QuantileType {
    /// Tipo k ∈ 1..=9.
    pub fn new(k: u8) -> Result<Self, StatsError> {
        if (1..=9).contains(&k) {
            Ok(Self(k))
        } else {
            Err(StatsError::InvalidOption("tipo de quantil entre 1 e 9"))
        }
    }

    /// Número do tipo.
    pub fn get(self) -> u8 {
        self.0
    }
}

impl Default for QuantileType {
    /// Tipo 7: interpolação linear com p(k) = (k − 1)/(n − 1), o mais difundido.
    fn default() -> Self {
        Self(7)
    }
}

/// Erros.
#[derive(Clone, Debug, PartialEq)]
pub enum StatsError {
    /// Opção ou argumento inválido.
    InvalidOption(&'static str),
    /// Forma incompatível com o número de elementos, ou dimensão inexistente.
    ShapeMismatch {
        /// Esperado.
        expected: usize,
        /// Recebido.
        actual: usize,
    },
}

impl StatsError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::InvalidOption(_) => "chimalab:stats:opcaoInvalida",
            Self::ShapeMismatch { .. } => "chimalab:stats:formaIncompativel",
        }
    }
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOption(m) => write!(f, "opção inválida: {m}"),
            Self::ShapeMismatch { expected, actual } => {
                write!(f, "forma incompatível: {actual}; esperado {expected}")
            }
        }
    }
}

impl std::error::Error for StatsError {}

/// Valores a considerar: com `Omit`, sem NaN; com `Include`, `None` se houver NaN.
fn usable(x: &[f64], nan: NanFlag) -> Option<Vec<f64>> {
    match nan {
        NanFlag::Omit => Some(x.iter().copied().filter(|v| !v.is_nan()).collect()),
        NanFlag::Include => {
            if x.iter().any(|v| v.is_nan()) {
                None
            } else {
                Some(x.to_vec())
            }
        }
    }
}

/// Soma compensada de Neumaier (erro ~2ε·Σ|xᵢ|, independente de n). Vazio: 0.
pub(crate) fn neumaier(x: impl IntoIterator<Item = f64>) -> f64 {
    let (s, c) = neumaier_par(x);
    // Com ±∞ na entrada, c vira NaN (∞ − ∞); a soma correta é s.
    if s.is_finite() { s + c } else { s }
}

/// Soma de Neumaier sem o arredondamento final: (s, c) com s + c a soma compensada. Para quem
/// continua acumulando (somas móveis), juntar s + c antes perderia a parte baixa.
pub(crate) fn neumaier_par(x: impl IntoIterator<Item = f64>) -> (f64, f64) {
    let mut s = 0.0f64;
    let mut c = 0.0f64;
    for v in x {
        let t = s + v;
        if s.abs() >= v.abs() {
            c += (s - t) + v;
        } else {
            c += (v - t) + s;
        }
        s = t;
    }
    (s, c)
}

/// Soma (compensada). Vazio: 0.
pub fn sum(x: &[f64], nan: NanFlag) -> f64 {
    match nan {
        NanFlag::Omit => neumaier(x.iter().copied().filter(|v| !v.is_nan())),
        NanFlag::Include => neumaier(x.iter().copied()),
    }
}

/// Produto. Vazio: 1.
pub fn prod(x: &[f64], nan: NanFlag) -> f64 {
    match nan {
        NanFlag::Omit => x.iter().filter(|v| !v.is_nan()).product(),
        NanFlag::Include => x.iter().product(),
    }
}

/// Média aritmética compensada, sem overflow intermediário. Vazio (ou só NaN com `Omit`): NaN.
pub fn mean(x: &[f64], nan: NanFlag) -> f64 {
    let Some(v) = usable(x, nan) else {
        return f64::NAN;
    };
    mean_of(&v)
}

fn mean_of(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let n = v.len() as f64;
    let s = neumaier(v.iter().copied());
    if s.is_finite() || v.iter().any(|x| x.is_infinite()) {
        return s / n;
    }
    // Overflow da soma com dados finitos: soma reescalada por 2⁻⁵⁴⁰ (exato) e desfeita no fim.
    let k = 2f64.powi(-540);
    neumaier(v.iter().map(|x| x * k)) / n / k
}

/// Variância (dois passos corrigidos, Chan, Golub & LeVeque). Um único valor: 0 (amostral e
/// populacional). Vazio: NaN.
pub fn var(x: &[f64], norm: VarNorm, nan: NanFlag) -> f64 {
    let Some(v) = usable(x, nan) else {
        return f64::NAN;
    };
    var_of(&v, norm)
}

fn var_of(v: &[f64], norm: VarNorm) -> f64 {
    let n = v.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return if v[0].is_finite() { 0.0 } else { f64::NAN };
    }
    let m = mean_of(v);
    if !m.is_finite() {
        return f64::NAN;
    }
    // Σ(x − m)² − (Σ(x − m))²/n: o segundo termo corrige o erro de arredondamento de m.
    let d: Vec<f64> = v.iter().map(|x| x - m).collect();
    let den = match norm {
        VarNorm::Sample => (n - 1) as f64,
        VarNorm::Population => n as f64,
    };
    let corrigida = |k: f64| {
        let s2 = neumaier(d.iter().map(|e| (e * k) * (e * k)));
        let s1 = neumaier(d.iter().map(|e| e * k));
        (s2, (s2 - s1 * s1 / n as f64).max(0.0))
    };
    let (s2, ss) = corrigida(1.0);
    if s2.is_finite() {
        return ss / den;
    }
    // Desvios enormes: Σe² estoura (e ∞ − ∞ daria NaN). Reescala por 2⁻⁵⁴⁰, exata, e desfaz no
    // fim; o resultado é ∞ só se a variância verdadeira passar do maior f64 (regressão).
    let k = 2f64.powi(-540);
    let (_, ssk) = corrigida(k);
    ssk / den / k / k
}

/// Desvio-padrão: √var.
pub fn std(x: &[f64], norm: VarNorm, nan: NanFlag) -> f64 {
    var(x, norm, nan).sqrt()
}

/// Mínimo. Vazio: NaN.
pub fn min(x: &[f64], nan: NanFlag) -> f64 {
    usable(x, nan).map_or(f64::NAN, |v| v.into_iter().reduce(f64::min).unwrap_or(f64::NAN))
}

/// Máximo. Vazio: NaN.
pub fn max(x: &[f64], nan: NanFlag) -> f64 {
    usable(x, nan).map_or(f64::NAN, |v| v.into_iter().reduce(f64::max).unwrap_or(f64::NAN))
}

/// Amplitude max − min.
pub fn range(x: &[f64], nan: NanFlag) -> f64 {
    max(x, nan) - min(x, nan)
}

/// k-ésimo menor (0-based) por seleção em O(n).
fn kth(v: &mut [f64], k: usize) -> f64 {
    let (_, x, _) = v.select_nth_unstable_by(k, f64::total_cmp);
    *x
}

/// Média de dois valores sem overflow.
fn midpoint(a: f64, b: f64) -> f64 {
    let m = (a + b) / 2.0;
    if m.is_finite() || !(a.is_finite() && b.is_finite()) {
        m
    } else {
        a / 2.0 + b / 2.0
    }
}

/// Mediana (seleção em O(n); n par: média dos dois centrais). Vazio: NaN.
pub fn median(x: &[f64], nan: NanFlag) -> f64 {
    let Some(mut v) = usable(x, nan) else {
        return f64::NAN;
    };
    median_mut(&mut v)
}

pub(crate) fn median_mut(v: &mut [f64]) -> f64 {
    let n = v.len();
    if n == 0 {
        return f64::NAN;
    }
    let hi = kth(v, n / 2);
    if n % 2 == 1 {
        hi
    } else {
        // O maior da metade inferior está na parte esquerda após a seleção.
        let lo = v[..n / 2].iter().copied().fold(f64::NEG_INFINITY, f64::max);
        midpoint(lo, hi)
    }
}

/// Moda: o valor mais frequente; empate, o menor. NaN é sempre ignorado. Vazio: NaN.
pub fn mode(x: &[f64]) -> f64 {
    let mut v: Vec<f64> = x.iter().copied().filter(|a| !a.is_nan()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let (mut best, mut best_n) = (v[0], 0usize);
    let mut i = 0;
    while i < v.len() {
        let mut j = i;
        // −0 e +0 contam como o mesmo valor.
        while j < v.len() && v[j] == v[i] {
            j += 1;
        }
        if j - i > best_n {
            best = v[i];
            best_n = j - i;
        }
        i = j;
    }
    best
}

/// Quantis nas probabilidades p ∈ [0, 1] pelo tipo de Hyndman & Fan. Vazio: NaN.
///
/// Erros: `opcaoInvalida` (p fora de [0, 1] ou NaN).
pub fn quantile(x: &[f64], p: &[f64], kind: QuantileType, nan: NanFlag) -> Result<Vec<f64>, StatsError> {
    if p.iter().any(|q| !(0.0..=1.0).contains(q)) {
        return Err(StatsError::InvalidOption("probabilidades em [0, 1]"));
    }
    let Some(mut v) = usable(x, nan) else {
        return Ok(vec![f64::NAN; p.len()]);
    };
    if v.is_empty() {
        return Ok(vec![f64::NAN; p.len()]);
    }
    v.sort_by(f64::total_cmp);
    Ok(p.iter().map(|&q| quantile_sorted(&v, q, kind)).collect())
}

/// Percentis em p ∈ [0, 100].
pub fn prctile(x: &[f64], p: &[f64], kind: QuantileType, nan: NanFlag) -> Result<Vec<f64>, StatsError> {
    if p.iter().any(|q| !(0.0..=100.0).contains(q)) {
        return Err(StatsError::InvalidOption("percentis em [0, 100]"));
    }
    let pp: Vec<f64> = p.iter().map(|q| q / 100.0).collect();
    quantile(x, &pp, kind, nan)
}

/// Quantil de dados ordenados (n ≥ 1), definições de Hyndman & Fan. Índices 1-based:
/// tipos 1–3 descontínuos; tipos 4–9 interpolam entre x₍ⱼ₎ e x₍ⱼ₊₁₎ com h = n·p + m.
pub(crate) fn quantile_sorted(v: &[f64], p: f64, kind: QuantileType) -> f64 {
    let n = v.len();
    let nf = n as f64;
    // Probabilidades decimais (0,1, 0,25…) não são exatas em binário: um n·p que fica a poucos ε
    // de um inteiro conta como inteiro, como pretendido (mesma ideia da tolerância de R).
    let snap = |h: f64| {
        let r = h.round();
        if (h - r).abs() <= 8.0 * f64::EPSILON * r.abs().max(1.0) {
            r
        } else {
            h
        }
    };
    let at = |j: i64| v[(j.clamp(1, n as i64) - 1) as usize];
    match kind.0 {
        1..=3 => {
            let np = snap(nf * p);
            match kind.0 {
                1 => at(np.ceil() as i64),
                2 => {
                    let j = np.floor();
                    if np > j {
                        at(j as i64 + 1)
                    } else {
                        // Média nos pontos de descontinuidade.
                        midpoint(at(j as i64), at(j as i64 + 1))
                    }
                }
                _ => {
                    // Tipo 3: h = n·p − ½; se h é inteiro, o par entre h e h + 1; senão ⌈h⌉.
                    let h = snap(np - 0.5);
                    let j = h.floor();
                    if h > j {
                        at(j as i64 + 1)
                    } else if (j as i64) % 2 == 0 {
                        at(j as i64)
                    } else {
                        at(j as i64 + 1)
                    }
                }
            }
        }
        k => {
            let (a, b) = match k {
                4 => (0.0, 1.0),
                5 => (0.5, 0.5),
                6 => (0.0, 0.0),
                7 => (1.0, 1.0),
                8 => (1.0 / 3.0, 1.0 / 3.0),
                _ => (3.0 / 8.0, 3.0 / 8.0),
            };
            // h = (n + 1 − a − b)p + a, 1-based.
            let h = snap((nf + 1.0 - a - b) * p + a);
            let j = h.floor();
            let g = h - j;
            let (lo, hi) = (at(j as i64), at(j as i64 + 1));
            if g == 0.0 || lo == hi {
                lo
            } else {
                lo + g * (hi - lo)
            }
        }
    }
}

/// Soma cumulativa (compensada). Com `Omit`, NaN conta como 0.
pub fn cumsum(x: &[f64], nan: NanFlag) -> Vec<f64> {
    let mut s = 0.0f64;
    let mut c = 0.0f64;
    x.iter()
        .map(|&v| {
            let v = if nan == NanFlag::Omit && v.is_nan() {
                0.0
            } else {
                v
            };
            let t = s + v;
            if s.abs() >= v.abs() {
                c += (s - t) + v;
            } else {
                c += (v - t) + s;
            }
            s = t;
            if s.is_finite() { s + c } else { s }
        })
        .collect()
}

/// Produto cumulativo. Com `Omit`, NaN conta como 1.
pub fn cumprod(x: &[f64], nan: NanFlag) -> Vec<f64> {
    let mut p = 1.0;
    x.iter()
        .map(|&v| {
            if !(nan == NanFlag::Omit && v.is_nan()) {
                p *= v;
            }
            p
        })
        .collect()
}

fn cum_ext(x: &[f64], nan: NanFlag, pick: fn(f64, f64) -> f64) -> Vec<f64> {
    let mut acc = f64::NAN;
    let mut viu_nan = false;
    x.iter()
        .map(|&v| {
            if v.is_nan() {
                if nan == NanFlag::Include {
                    viu_nan = true;
                }
            } else {
                acc = if acc.is_nan() { v } else { pick(acc, v) };
            }
            if viu_nan { f64::NAN } else { acc }
        })
        .collect()
}

/// Máximo cumulativo. Com `Omit`, NaN é pulado (NaN só antes do primeiro número).
pub fn cummax(x: &[f64], nan: NanFlag) -> Vec<f64> {
    cum_ext(x, nan, f64::max)
}

/// Mínimo cumulativo.
pub fn cummin(x: &[f64], nan: NanFlag) -> Vec<f64> {
    cum_ext(x, nan, f64::min)
}

/// Aplica `f` a cada linha ao longo da dimensão `dim` (0-based) de um array column-major com
/// forma `shape`. Todas as linhas devem produzir o mesmo número de valores m, e a saída tem a
/// forma original com `shape[dim]` = m.
///
/// Erros: `formaIncompativel` (produto da forma ≠ número de dados, ou dim fora da forma, ou
/// linhas de tamanhos diferentes).
pub fn along_dim(
    data: &[f64],
    shape: &[usize],
    dim: usize,
    mut f: impl FnMut(&[f64]) -> Vec<f64>,
) -> Result<(Vec<f64>, Vec<usize>), StatsError> {
    let total: usize = shape.iter().product();
    if total != data.len() {
        return Err(StatsError::ShapeMismatch {
            expected: total,
            actual: data.len(),
        });
    }
    if dim >= shape.len() {
        return Err(StatsError::ShapeMismatch {
            expected: shape.len(),
            actual: dim + 1,
        });
    }
    let n = shape[dim];
    let stride: usize = shape[..dim].iter().product();
    let outer: usize = shape[dim + 1..].iter().product();
    let mut linha = vec![0.0; n];
    let mut out: Vec<f64> = Vec::new();
    let mut m: Option<usize> = None;
    let mut resultados: Vec<Vec<f64>> = Vec::with_capacity(stride * outer);
    for o in 0..outer {
        for i in 0..stride {
            for (k, l) in linha.iter_mut().enumerate() {
                *l = data[i + stride * (k + n * o)];
            }
            let r = f(&linha);
            match m {
                None => m = Some(r.len()),
                Some(mm) if mm != r.len() => {
                    return Err(StatsError::ShapeMismatch {
                        expected: mm,
                        actual: r.len(),
                    });
                }
                _ => {}
            }
            resultados.push(r);
        }
    }
    let m = m.unwrap_or(0);
    out.resize(stride * m * outer, 0.0);
    for o in 0..outer {
        for i in 0..stride {
            let r = &resultados[o * stride + i];
            for (k, v) in r.iter().enumerate() {
                out[i + stride * (k + m * o)] = *v;
            }
        }
    }
    let mut forma = shape.to_vec();
    forma[dim] = m;
    Ok((out, forma))
}

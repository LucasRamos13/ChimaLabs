//! Polinômios por partes na forma local: no intervalo [bᵢ, bᵢ₊₁),
//! p(x) = Σₖ cᵢₖ·(x − bᵢ)ᵏ, k = 0..grau.

use super::{Extrap, InterpError};

/// Polinômio por partes.
#[derive(Clone, Debug, PartialEq)]
pub struct PPoly {
    /// Quebras estritamente crescentes (m + 1 valores para m intervalos).
    breaks: Vec<f64>,
    /// Coeficientes locais de cada intervalo, do grau 0 ao grau máximo.
    coefs: Vec<Vec<f64>>,
}

impl PPoly {
    /// Cria a partir das quebras e dos coeficientes locais (`coefs[i][k]` multiplica (x − bᵢ)ᵏ).
    ///
    /// Erros: `pontosInsuficientes` (menos de 2 quebras), `formaIncompativel` (número de
    /// intervalos ≠ quebras − 1 ou graus diferentes), `naoMonotono` (quebras não estritamente
    /// crescentes ou não finitas).
    pub fn new(breaks: Vec<f64>, coefs: Vec<Vec<f64>>) -> Result<Self, InterpError> {
        if breaks.len() < 2 {
            return Err(InterpError::TooFewPoints {
                needed: 2,
                got: breaks.len(),
            });
        }
        if coefs.len() != breaks.len() - 1 {
            return Err(InterpError::ShapeMismatch {
                expected: breaks.len() - 1,
                actual: coefs.len(),
            });
        }
        let k = coefs[0].len();
        if k == 0 || coefs.iter().any(|c| c.len() != k) {
            return Err(InterpError::ShapeMismatch {
                expected: k.max(1),
                actual: coefs.iter().map(Vec::len).find(|&l| l != k).unwrap_or(0),
            });
        }
        if breaks.iter().any(|b| !b.is_finite()) || breaks.windows(2).any(|w| w[1] <= w[0]) {
            return Err(InterpError::NotMonotonic);
        }
        Ok(Self { breaks, coefs })
    }

    pub(crate) fn from_parts(breaks: Vec<f64>, coefs: Vec<Vec<f64>>) -> Self {
        Self { breaks, coefs }
    }

    /// Quebras.
    pub fn breaks(&self) -> &[f64] {
        &self.breaks
    }

    /// Coeficientes locais.
    pub fn coefs(&self) -> &[Vec<f64>] {
        &self.coefs
    }

    /// Grau (número de coeficientes − 1).
    pub fn degree(&self) -> usize {
        self.coefs[0].len() - 1
    }

    /// Índice do intervalo de x (os extremos usam o primeiro e o último intervalo).
    fn interval(&self, x: f64) -> usize {
        let m = self.coefs.len();
        // Primeira quebra > x, menos 1, limitada a [0, m − 1].
        self.breaks
            .partition_point(|&b| b <= x)
            .saturating_sub(1)
            .min(m - 1)
    }

    fn eval_in(&self, i: usize, x: f64) -> f64 {
        let dx = x - self.breaks[i];
        self.coefs[i].iter().rev().fold(0.0, |acc, c| acc * dx + c)
    }

    /// Valor em x. Fora de [b₀, b_m], segue `extrap`; NaN dá NaN.
    pub fn eval(&self, x: f64, extrap: Extrap) -> f64 {
        if x.is_nan() {
            return f64::NAN;
        }
        let (lo, hi) = (self.breaks[0], *self.breaks.last().unwrap_or(&self.breaks[0]));
        if x < lo || x > hi {
            return match extrap {
                Extrap::Nan => f64::NAN,
                Extrap::Value(v) => v,
                Extrap::Clamp => self.eval_in(self.interval(x.clamp(lo, hi)), x.clamp(lo, hi)),
                Extrap::Extrapolate => self.eval_in(self.interval(x), x),
            };
        }
        self.eval_in(self.interval(x), x)
    }

    /// Valores em vários pontos.
    pub fn eval_many(&self, xs: &[f64], extrap: Extrap) -> Vec<f64> {
        xs.iter().map(|&x| self.eval(x, extrap)).collect()
    }

    /// Derivada (grau − 1; a derivada de uma constante é a função nula de grau 0).
    pub fn derivative(&self) -> Self {
        let coefs = self
            .coefs
            .iter()
            .map(|c| {
                if c.len() == 1 {
                    vec![0.0]
                } else {
                    c.iter().enumerate().skip(1).map(|(k, v)| k as f64 * v).collect()
                }
            })
            .collect();
        Self {
            breaks: self.breaks.clone(),
            coefs,
        }
    }

    /// Primitiva que vale 0 em b₀ (contínua: cada intervalo começa no valor acumulado).
    pub fn antiderivative(&self) -> Self {
        let mut acc = 0.0;
        let mut coefs = Vec::with_capacity(self.coefs.len());
        for (i, c) in self.coefs.iter().enumerate() {
            let mut n = vec![acc];
            n.extend(c.iter().enumerate().map(|(k, v)| v / (k as f64 + 1.0)));
            let h = self.breaks[i + 1] - self.breaks[i];
            acc = n.iter().rev().fold(0.0, |s, v| s * h + v);
            coefs.push(n);
        }
        Self {
            breaks: self.breaks.clone(),
            coefs,
        }
    }

    /// ∫ₐᵇ p(x) dx (a e b dentro de [b₀, b_m]; fora, segue a extrapolação polinomial).
    pub fn integral(&self, a: f64, b: f64) -> f64 {
        let p = self.antiderivative();
        p.eval(b, Extrap::Extrapolate) - p.eval(a, Extrap::Extrapolate)
    }
}

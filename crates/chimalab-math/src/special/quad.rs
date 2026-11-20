//! Quadratura de dupla exponencial (Takahasi e Mori, "Double exponential formulas for numerical
//! integration", Publ. RIMS 9, 1974): tanh-sinh em [a, b] e exp-sinh em [0, ∞).
//! Uso interno (funções especiais); a integração pública é a funcionalidade F048.

use std::f64::consts::FRAC_PI_2;
use std::ops::{Add, AddAssign, Mul, Sub};

use num_complex::Complex64;

const MAX_LEVEL: usize = 13;

/// Valores que a quadratura soma: reais ou complexos.
trait DeValue: Copy + Add<Output = Self> + AddAssign + Sub<Output = Self> + Mul<f64, Output = Self> {
    const ZERO: Self;
    fn magnitude(self) -> f64;
}

impl DeValue for f64 {
    const ZERO: Self = 0.0;
    fn magnitude(self) -> f64 {
        self.abs()
    }
}

impl DeValue for Complex64 {
    const ZERO: Self = Complex64::new(0.0, 0.0);
    fn magnitude(self) -> f64 {
        self.norm()
    }
}

/// Soma refinando o passo pela metade até convergir; `node(t)` devolve `w(t)·f(x(t))`.
fn de_sum<T: DeValue>(node: impl Fn(f64) -> T, tmax: f64, tol: f64, scale: f64) -> T {
    let mut h = 0.5;
    // Nível 0: todos os pontos k·h
    let mut sum = node(0.0);
    let mut k = 1.0;
    while k * h <= tmax {
        sum += node(k * h) + node(-k * h);
        k += 1.0;
    }
    let mut est = sum * h;
    for _ in 1..MAX_LEVEL {
        h *= 0.5;
        // Novos pontos: múltiplos ímpares de h
        let mut add = T::ZERO;
        let mut j = 1.0;
        while j * h <= tmax {
            add += node(j * h) + node(-j * h);
            j += 2.0;
        }
        sum += add;
        let new = sum * h;
        if (new - est).magnitude() <= tol * new.magnitude().max(scale).max(f64::MIN_POSITIVE) {
            return new;
        }
        est = new;
    }
    est
}

/// ∫ₐᵇ f(x) dx por tanh-sinh (o integrando não é avaliado nas extremidades).
/// `scale` é o piso absoluto do critério de convergência (útil quando a integral pode ser ≈ 0).
pub(crate) fn tanh_sinh(f: impl Fn(f64) -> f64, a: f64, b: f64, tol: f64, scale: f64) -> f64 {
    let half = 0.5 * (b - a);
    de_sum(
        |t| {
            let s = FRAC_PI_2 * t.sinh();
            let e = (-2.0 * s.abs()).exp();
            // distância à extremidade mais próxima, sem cancelamento: (b−a)·e/(1+e)
            let d = (b - a) * e / (1.0 + e);
            let x = if s >= 0.0 { b - d } else { a + d };
            if d == 0.0 {
                return 0.0;
            }
            let ch = s.cosh();
            let w = half * FRAC_PI_2 * t.cosh() / (ch * ch);
            w * f(x)
        },
        3.2,
        tol,
        scale,
    )
}

/// ∫₀^∞ f(x) dx por exp-sinh (para integrandos com decaimento ao menos exponencial).
pub(crate) fn exp_sinh(f: impl Fn(f64) -> f64, tol: f64, scale: f64) -> f64 {
    exp_sinh_generic(f, tol, scale)
}

/// [`exp_sinh`] para integrando complexo.
pub(crate) fn exp_sinh_complex(f: impl Fn(f64) -> Complex64, tol: f64, scale: f64) -> Complex64 {
    exp_sinh_generic(f, tol, scale)
}

fn exp_sinh_generic<T: DeValue>(f: impl Fn(f64) -> T, tol: f64, scale: f64) -> T {
    de_sum(
        |t| {
            let x = (FRAC_PI_2 * t.sinh()).exp();
            if x == 0.0 || !x.is_finite() {
                return T::ZERO;
            }
            let w = FRAC_PI_2 * t.cosh() * x;
            let v = f(x);
            if v.magnitude() == 0.0 { T::ZERO } else { v * w }
        },
        4.5,
        tol,
        scale,
    )
}

//! Funções elementares e especiais (funcionalidade F042, RF-042).
//!
//! Implementação independente a partir de fontes primárias: NIST Digital Library of
//! Mathematical Functions (DLMF, <https://dlmf.nist.gov>) e artigos originais citados em cada
//! função. Nenhum código derivado de "Numerical Recipes" (ver LICENCAS_E_RESTRICOES.txt D12).

mod bessel;
mod bessel_complex;
mod dd;
mod elementary;
mod elliptic;
mod erf;
mod erf_complex;
mod expint;
mod gamma;
mod gamma_complex;
mod integer;
mod legendre;
mod quad;
mod zeta;

pub use bessel::{HankelKind, airy, airy_prime, besselh, besseli, besselj, besselk, bessely};
pub use bessel_complex::{
    airy_complex, airy_prime_complex, besselh_complex, besseli_complex, besselj_complex, besselk_complex,
    bessely_complex,
};
pub use elementary::{
    acosd, asind, atan2d, atand, cosd, cotd, cscd, deg2rad, fix, frexp, ldexp, mod_floor, nthroot, rad2deg,
    reallog, realpow, realsqrt, rem_trunc, round_digits, secd, sign, sind, tand,
};
pub use elliptic::{ellipj, ellipke};
pub use erf::{erf, erfc, erfcinv, erfcx, erfinv};
pub use erf_complex::{erf_complex, erfc_complex, erfcx_complex, faddeeva};
pub use expint::{ei, expint, expint_complex};
pub use gamma::{beta, betainc, betaln, factorial, gamma, gammainc, gammaln, psi};
pub use gamma_complex::{gamma_complex, gammaln_complex};
pub use integer::{
    Bezout, combinations, factor, gcd, gcd_bezout, isprime, lcm, nchoosek, perms, primes, primes_between,
};
pub use legendre::{LegendreNorm, legendre};
pub use zeta::{hurwitz_zeta, polygamma, zeta};

use std::fmt;

/// Erros das funções especiais.
#[derive(Clone, Debug, PartialEq)]
pub enum SpecialError {
    /// Argumento fora do domínio (ex.: inteiro exigido, valor negativo onde não se admite).
    Domain(&'static str),
    /// O resultado seria complexo (funções "reais" como `realsqrt`).
    ComplexResult(&'static str),
}

impl SpecialError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Domain(_) => "chimalab:special:dominio",
            Self::ComplexResult(_) => "chimalab:special:resultadoComplexo",
        }
    }
}

impl fmt::Display for SpecialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(m) => write!(f, "argumento fora do domínio: {m}"),
            Self::ComplexResult(m) => write!(f, "o resultado seria complexo: {m}"),
        }
    }
}

impl std::error::Error for SpecialError {}

/// Fração contínua pelo método de Lentz modificado (Thompson e Barnett, J. Comput. Phys. 64, 1986):
/// avalia `b0 + a1/(b1 + a2/(b2 + …))`, com `terms(k)` devolvendo `(a_k, b_k)` para k ≥ 1.
pub(crate) fn lentz(b0: f64, terms: impl Fn(usize) -> (f64, f64)) -> f64 {
    lentz_n(b0, 10_000, terms)
}

/// [`lentz`] com número máximo de termos dado.
pub(crate) fn lentz_n(b0: f64, max_terms: usize, terms: impl Fn(usize) -> (f64, f64)) -> f64 {
    const TINY: f64 = 1e-300;
    let mut f = if b0 == 0.0 { TINY } else { b0 };
    let (mut c, mut d) = (f, 0.0);
    for k in 1..max_terms {
        let (a, b) = terms(k);
        d = b + a * d;
        if d == 0.0 {
            d = TINY;
        }
        c = b + a / c;
        if c == 0.0 {
            c = TINY;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if (delta - 1.0).abs() < 1e-16 {
            break;
        }
    }
    f
}

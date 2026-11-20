//! Trigonometria em graus (exata nos múltiplos de 90°), arredondamentos, restos e raízes reais.

use super::SpecialError;
use std::f64::consts::PI;

/// Reduz `x` graus a `(n, t)` com `x ≡ 90·n + t (mod 360)`, `|t| ≤ 45`, `n ∈ 0..4`. Exato.
fn reduce_deg(x: f64) -> (u8, f64) {
    let r = x % 360.0; // fmod é exato
    let q = (r / 90.0).round();
    let t = r - 90.0 * q; // exato: q·90 e r são múltiplos representáveis da mesma escala
    (q.rem_euclid(4.0) as u8, t)
}

fn sin_t(t: f64) -> f64 {
    (t * (PI / 180.0)).sin()
}
fn cos_t(t: f64) -> f64 {
    (t * (PI / 180.0)).cos()
}

/// Zero com sinal positivo, exceto quando a entrada é ±0 (preserva o sinal da entrada).
fn clean_zero(v: f64, x: f64) -> f64 {
    if v == 0.0 {
        if x == 0.0 { x } else { 0.0 }
    } else {
        v
    }
}

/// Seno de `x` em graus. Exato nos múltiplos de 90° (ex.: `sind(180) == 0`).
pub fn sind(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let (n, t) = reduce_deg(x);
    let v = match n {
        0 => sin_t(t),
        1 => cos_t(t),
        2 => -sin_t(t),
        _ => -cos_t(t),
    };
    clean_zero(v, x)
}

/// Cosseno de `x` em graus. Exato nos múltiplos de 90°.
pub fn cosd(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let (n, t) = reduce_deg(x);
    let v = match n {
        0 => cos_t(t),
        1 => -sin_t(t),
        2 => -cos_t(t),
        _ => sin_t(t),
    };
    clean_zero(v, 1.0)
}

/// Tangente em graus (±Inf nos múltiplos ímpares de 90°; exata nos múltiplos de 45°).
pub fn tand(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let (_, t) = reduce_deg(x);
    if t.abs() == 45.0 {
        let (s, c) = (sind(x), cosd(x));
        return if s.signum() == c.signum() { 1.0 } else { -1.0 };
    }
    let (s, c) = (sind(x), cosd(x));
    if c == 0.0 {
        f64::INFINITY.copysign(s)
    } else {
        clean_zero(s / c, x)
    }
}

/// Cotangente em graus.
pub fn cotd(x: f64) -> f64 {
    1.0 / tand(x)
}
/// Secante em graus.
pub fn secd(x: f64) -> f64 {
    1.0 / cosd(x)
}
/// Cossecante em graus.
pub fn cscd(x: f64) -> f64 {
    1.0 / sind(x)
}

/// Arco-seno em graus (exato em 0, ±1/2, ±1).
pub fn asind(x: f64) -> f64 {
    match x {
        1.0 => 90.0,
        -1.0 => -90.0,
        0.5 => 30.0,
        -0.5 => -30.0,
        _ => rad2deg(x.asin()),
    }
}
/// Arco-cosseno em graus (exato em 0, ±1/2, ±1).
pub fn acosd(x: f64) -> f64 {
    match x {
        1.0 => 0.0,
        -1.0 => 180.0,
        0.0 => 90.0,
        0.5 => 60.0,
        -0.5 => 120.0,
        _ => rad2deg(x.acos()),
    }
}
/// Arco-tangente em graus (exato em 0, ±1, ±Inf).
pub fn atand(x: f64) -> f64 {
    match x {
        1.0 => 45.0,
        -1.0 => -45.0,
        _ if x.is_infinite() => 90.0f64.copysign(x),
        _ => rad2deg(x.atan()),
    }
}
/// Arco-tangente de quatro quadrantes em graus.
pub fn atan2d(y: f64, x: f64) -> f64 {
    if y.abs() == x.abs() && x != 0.0 && x.is_finite() {
        return match (y > 0.0, x > 0.0) {
            (true, true) => 45.0,
            (true, false) => 135.0,
            (false, true) => -45.0,
            (false, false) => -135.0,
        };
    }
    rad2deg(y.atan2(x))
}

/// Graus para radianos.
pub fn deg2rad(x: f64) -> f64 {
    x * (PI / 180.0)
}
/// Radianos para graus.
pub fn rad2deg(x: f64) -> f64 {
    x * (180.0 / PI)
}

/// Truncamento em direção a zero.
pub fn fix(x: f64) -> f64 {
    x.trunc()
}

/// Arredonda para `n` casas decimais (meio para longe do zero); `n` negativo arredonda à esquerda
/// da vírgula. Resultado é o `f64` mais próximo do decimal arredondado.
pub fn round_digits(x: f64, n: i32) -> f64 {
    if !x.is_finite() {
        return x;
    }
    if n >= 0 {
        let p = 10f64.powi(n);
        let y = x * p;
        if !y.is_finite() || y.abs() >= 2f64.powi(52) {
            return x; // já sem casas a arredondar
        }
        y.round() / p
    } else {
        let p = 10f64.powi(-n);
        (x / p).round() * p
    }
}

/// Resto com o sinal do divisor (`x − floor(x/y)·y`). Exato. `mod_floor(x, 0) = x`.
pub fn mod_floor(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return x;
    }
    if x.is_infinite() || y.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    if y.is_infinite() {
        return if x == 0.0 || x.signum() == y.signum() {
            x
        } else {
            y
        };
    }
    let r = x % y;
    if r != 0.0 && (r < 0.0) != (y < 0.0) {
        r + y
    } else if r == 0.0 {
        0.0f64.copysign(y)
    } else {
        r
    }
}

/// Resto com o sinal do dividendo (`x − fix(x/y)·y`). Exato. `rem_trunc(x, 0) = NaN`.
pub fn rem_trunc(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return f64::NAN;
    }
    x % y
}

/// Sinal: −1, 0 ou 1 (NaN para NaN).
pub fn sign(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x == 0.0 {
        0.0
    } else {
        x.signum()
    }
}

/// Raiz n-ésima real. Para `x < 0` exige `n` ímpar.
pub fn nthroot(x: f64, n: i64) -> Result<f64, SpecialError> {
    if n == 0 {
        return Err(SpecialError::Domain("nthroot: n não pode ser 0"));
    }
    if x < 0.0 && n % 2 == 0 {
        return Err(SpecialError::ComplexResult(
            "nthroot: raiz par de número negativo",
        ));
    }
    let mut y = x.abs().powf(1.0 / n as f64).copysign(x);
    // Um passo de Newton corrige o arredondamento de powf (raízes exatas ficam exatas).
    if y.is_finite() && y != 0.0 && n.unsigned_abs() <= 64 {
        let nn = n as f64;
        let yn = y.powi(n as i32);
        y -= (yn - x) / (nn * yn / y);
    }
    Ok(y)
}

/// Raiz quadrada que falha para resultados complexos.
pub fn realsqrt(x: f64) -> Result<f64, SpecialError> {
    if x < 0.0 {
        Err(SpecialError::ComplexResult("realsqrt de número negativo"))
    } else {
        Ok(x.sqrt())
    }
}

/// Logaritmo natural que falha para resultados complexos.
pub fn reallog(x: f64) -> Result<f64, SpecialError> {
    if x < 0.0 {
        Err(SpecialError::ComplexResult("reallog de número negativo"))
    } else {
        Ok(x.ln())
    }
}

/// Potência que falha para resultados complexos (base negativa com expoente não inteiro).
pub fn realpow(x: f64, y: f64) -> Result<f64, SpecialError> {
    if x < 0.0 && y.fract() != 0.0 {
        Err(SpecialError::ComplexResult(
            "realpow: base negativa com expoente não inteiro",
        ))
    } else {
        Ok(x.powf(y))
    }
}

/// Decompõe `x = f·2^e` com `0,5 ≤ |f| < 1` (f = e = 0 para x = 0; NaN/Inf com e = 0).
pub fn frexp(x: f64) -> (f64, i32) {
    if x == 0.0 || !x.is_finite() {
        return (x, 0);
    }
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i32;
    if exp == 0 {
        // subnormal: normaliza multiplicando por 2^54
        let (f, e) = frexp(x * 2f64.powi(54));
        return (f, e - 54);
    }
    let f = f64::from_bits((bits & !(0x7ffu64 << 52)) | (1022u64 << 52));
    (f, exp - 1022)
}

/// `f·2^e` exato (salvo overflow/underflow).
pub fn ldexp(f: f64, e: i32) -> f64 {
    // Divide em passos para não transbordar 2^e no meio.
    let mut x = f;
    let mut e = e;
    while e > 1000 {
        x *= 2f64.powi(1000);
        e -= 1000;
    }
    while e < -1000 {
        x *= 2f64.powi(-1000);
        e += 1000;
    }
    x * 2f64.powi(e)
}

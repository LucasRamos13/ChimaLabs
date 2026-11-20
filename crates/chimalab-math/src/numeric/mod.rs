//! Precisão e aritmética especial (funcionalidade F054, RF-054; DT-31).
//!
//! - Representação de ponto flutuante: [`eps`], [`eps_f32`], [`flintmax`], [`flintmax_f32`],
//!   [`realmax`], [`realmin`], [`next_after`].
//! - Inteiros: [`IntType`] (limites), [`idivide`] com [`Rounding`].
//! - Bases: [`to_base`], [`from_base`], [`dec2bin`], [`bin2dec`], [`dec2hex`], [`hex2dec`],
//!   [`dec2bin_signed`]; bits IEEE: [`num2hex`], [`hex2num`].
//! - Precisão estendida: [`DoubleDouble`] (~32 dígitos, operadores, sqrt, exp, ln, potência,
//!   leitura e escrita decimal).

use std::fmt;

mod dd;
pub use dd::DoubleDouble;

/// Erros.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumericError {
    /// Base fora de 2..=36.
    InvalidBase(u32),
    /// Caractere inválido para a base, ou cadeia vazia.
    InvalidDigit(String),
    /// Valor não cabe no tipo.
    Overflow,
    /// Número de bits fora do intervalo.
    InvalidBits(u32),
}

impl NumericError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::InvalidBase(_) => "chimalab:numeric:baseInvalida",
            Self::InvalidDigit(_) => "chimalab:numeric:digitoInvalido",
            Self::Overflow => "chimalab:numeric:estouro",
            Self::InvalidBits(_) => "chimalab:numeric:bitsInvalidos",
        }
    }
}

impl fmt::Display for NumericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBase(b) => write!(f, "base {b} fora de 2..=36"),
            Self::InvalidDigit(s) => write!(f, "dígito inválido em {s:?}"),
            Self::Overflow => write!(f, "valor não cabe no tipo"),
            Self::InvalidBits(b) => write!(f, "número de bits {b} fora de 1..=128"),
        }
    }
}

impl std::error::Error for NumericError {}

/// Distância de |x| ao próximo f64 maior (o ulp de x). eps(1) = 2⁻⁵²; eps(0) = 2⁻¹⁰⁷⁴ (menor
/// subnormal); eps(±∞) e eps(NaN) = NaN; eps(f64::MAX) = 2⁹⁷¹ (a distância ao anterior).
pub fn eps(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let a = x.abs();
    if a == f64::MAX {
        return a - f64::from_bits(a.to_bits() - 1);
    }
    f64::from_bits(a.to_bits() + 1) - a
}

/// Como [`eps`], em precisão simples.
pub fn eps_f32(x: f32) -> f32 {
    if !x.is_finite() {
        return f32::NAN;
    }
    let a = x.abs();
    if a == f32::MAX {
        return a - f32::from_bits(a.to_bits() - 1);
    }
    f32::from_bits(a.to_bits() + 1) - a
}

/// Maior inteiro tal que ele e todos os menores em módulo são exatos em f64: 2⁵³.
pub fn flintmax() -> f64 {
    9_007_199_254_740_992.0
}

/// Maior inteiro consecutivo exato em f32: 2²⁴.
pub fn flintmax_f32() -> f32 {
    16_777_216.0
}

/// Maior f64 finito.
pub fn realmax() -> f64 {
    f64::MAX
}

/// Menor f64 normal positivo (2⁻¹⁰²²); abaixo dele há os subnormais.
pub fn realmin() -> f64 {
    f64::MIN_POSITIVE
}

/// Próximo f64 depois de x na direção de y (IEEE 754 nextAfter). x = y devolve y; NaN propaga.
pub fn next_after(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if x == y {
        return y;
    }
    if x == 0.0 {
        // Menor subnormal com o sinal de y.
        return f64::from_bits(1).copysign(y);
    }
    let b = x.to_bits();
    let afasta = (y > x) == (x > 0.0);
    f64::from_bits(if afasta { b + 1 } else { b - 1 })
}

/// Tipos inteiros.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntType {
    /// 8 bits com sinal.
    I8,
    /// 16 bits com sinal.
    I16,
    /// 32 bits com sinal.
    I32,
    /// 64 bits com sinal.
    I64,
    /// 8 bits sem sinal.
    U8,
    /// 16 bits sem sinal.
    U16,
    /// 32 bits sem sinal.
    U32,
    /// 64 bits sem sinal.
    U64,
}

impl IntType {
    /// Maior valor.
    pub fn max(self) -> i128 {
        match self {
            Self::I8 => i8::MAX.into(),
            Self::I16 => i16::MAX.into(),
            Self::I32 => i32::MAX.into(),
            Self::I64 => i64::MAX.into(),
            Self::U8 => u8::MAX.into(),
            Self::U16 => u16::MAX.into(),
            Self::U32 => u32::MAX.into(),
            Self::U64 => u64::MAX.into(),
        }
    }

    /// Menor valor.
    pub fn min(self) -> i128 {
        match self {
            Self::I8 => i8::MIN.into(),
            Self::I16 => i16::MIN.into(),
            Self::I32 => i32::MIN.into(),
            Self::I64 => i64::MIN.into(),
            _ => 0,
        }
    }

    /// Satura v no intervalo do tipo.
    pub fn saturate(self, v: i128) -> i128 {
        v.clamp(self.min(), self.max())
    }
}

/// Arredondamento da divisão inteira.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rounding {
    /// Em direção a zero (padrão).
    #[default]
    Fix,
    /// Ao mais próximo; empate para longe de zero.
    Round,
    /// Para −∞.
    Floor,
    /// Para +∞.
    Ceil,
}

/// a/b arredondado e saturado no tipo. Exato (sem passar por ponto flutuante).
///
/// Divisão por zero: a > 0 dá o máximo do tipo, a < 0 o mínimo e 0/0 dá 0 (a convenção de
/// aritmética saturada).
pub fn idivide(a: i128, b: i128, rounding: Rounding, ty: IntType) -> i128 {
    if b == 0 {
        return match a.signum() {
            1 => ty.max(),
            -1 => ty.min(),
            _ => 0,
        };
    }
    // Quociente truncado e resto (|r| < |b|). O único overflow possível é i128::MIN/−1, cujo
    // valor verdadeiro (2¹²⁷) satura no máximo de qualquer tipo.
    let Some(q) = a.checked_div(b) else {
        return ty.max();
    };
    let r = a % b;
    let q = if r == 0 {
        q
    } else {
        let negativo = (a < 0) != (b < 0);
        match rounding {
            Rounding::Fix => q,
            Rounding::Floor => {
                if negativo {
                    q - 1
                } else {
                    q
                }
            }
            Rounding::Ceil => {
                if negativo {
                    q
                } else {
                    q + 1
                }
            }
            Rounding::Round => {
                // |r|/|b| ≥ ½ ⇔ |r| ≥ |b| − |r| (sem dobrar, que poderia estourar): afasta de zero.
                if r.unsigned_abs() >= b.unsigned_abs() - r.unsigned_abs() {
                    if negativo { q - 1 } else { q + 1 }
                } else {
                    q
                }
            }
        }
    };
    ty.saturate(q)
}

const DIGITOS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Representação de v na base (2..=36), com pelo menos `min_digits` dígitos (zeros à esquerda).
pub fn to_base(v: u128, base: u32, min_digits: usize) -> Result<String, NumericError> {
    if !(2..=36).contains(&base) {
        return Err(NumericError::InvalidBase(base));
    }
    let mut d = Vec::new();
    let mut x = v;
    while x > 0 {
        d.push(DIGITOS[(x % u128::from(base)) as usize]);
        x /= u128::from(base);
    }
    while d.len() < min_digits.max(1) {
        d.push(b'0');
    }
    d.reverse();
    Ok(String::from_utf8(d).expect("ASCII"))
}

/// Valor de uma cadeia na base (2..=36). Ignora espaços nas pontas e `_` entre dígitos; aceita
/// maiúsculas e minúsculas, e os prefixos 0b, 0o, 0x quando coincidem com a base.
pub fn from_base(s: &str, base: u32) -> Result<u128, NumericError> {
    if !(2..=36).contains(&base) {
        return Err(NumericError::InvalidBase(base));
    }
    let t = s.trim();
    let sem_prefixo = match (base, t.get(..2).map(str::to_ascii_lowercase).as_deref()) {
        (2, Some("0b")) | (8, Some("0o")) | (16, Some("0x")) => &t[2..],
        _ => t,
    };
    let invalido = || NumericError::InvalidDigit(s.to_string());
    if sem_prefixo.is_empty() || sem_prefixo.starts_with('_') || sem_prefixo.ends_with('_') {
        return Err(invalido());
    }
    let mut v: u128 = 0;
    for c in sem_prefixo.chars() {
        if c == '_' {
            continue;
        }
        let d = c.to_digit(base).ok_or_else(invalido)?;
        v = v
            .checked_mul(u128::from(base))
            .and_then(|x| x.checked_add(u128::from(d)))
            .ok_or(NumericError::Overflow)?;
    }
    Ok(v)
}

/// Binário com pelo menos `min_digits` dígitos.
pub fn dec2bin(v: u128, min_digits: usize) -> String {
    to_base(v, 2, min_digits).expect("base válida")
}

/// Valor de uma cadeia binária.
pub fn bin2dec(s: &str) -> Result<u128, NumericError> {
    from_base(s, 2)
}

/// Hexadecimal (maiúsculas) com pelo menos `min_digits` dígitos.
pub fn dec2hex(v: u128, min_digits: usize) -> String {
    to_base(v, 16, min_digits).expect("base válida")
}

/// Valor de uma cadeia hexadecimal.
pub fn hex2dec(s: &str) -> Result<u128, NumericError> {
    from_base(s, 16)
}

/// Binário de um inteiro com sinal em complemento de dois com `bits` dígitos (1..=128).
///
/// Erros: `bitsInvalidos`; `estouro` se v não cabe em `bits` bits com sinal.
pub fn dec2bin_signed(v: i128, bits: u32) -> Result<String, NumericError> {
    if !(1..=128).contains(&bits) {
        return Err(NumericError::InvalidBits(bits));
    }
    let (lo, hi) = if bits == 128 {
        (i128::MIN, i128::MAX)
    } else {
        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
    };
    if v < lo || v > hi {
        return Err(NumericError::Overflow);
    }
    let mask: u128 = if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    Ok(dec2bin(v as u128 & mask, bits as usize))
}

/// Os 64 bits IEEE 754 de x em hexadecimal (16 dígitos, minúsculas).
pub fn num2hex(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

/// f64 a partir dos bits em hexadecimal. Com menos de 16 dígitos, completa com zeros à direita
/// (os dígitos dados são os mais significativos: "3ff" é 1,0).
pub fn hex2num(s: &str) -> Result<f64, NumericError> {
    let t = s.trim();
    let t = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
    if t.is_empty() || t.len() > 16 || !t.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(NumericError::InvalidDigit(s.to_string()));
    }
    let completo = format!("{t:0<16}");
    let bits = u64::from_str_radix(&completo, 16).expect("até 16 dígitos hexadecimais validados acima");
    Ok(f64::from_bits(bits))
}

//! Aritmética dupla-dupla para o usuário: x = hi + lo com |lo| ≤ ½ulp(hi), ~106 bits de
//! mantissa (~32 dígitos decimais) e a mesma faixa de expoentes do f64.
//!
//! Transformações exatas de T. J. Dekker, Numer. Math. 18 (1971) 224–242, e os algoritmos de
//! Y. Hida, X. S. Li & D. H. Bailey, "Algorithms for quad-double precision floating point
//! arithmetic", ARITH-15 (2001), §2–3, para a forma dupla-dupla:
//! - soma e produto com renormalização;
//! - divisão por três quocientes parciais;
//! - raiz por um passo de Newton;
//! - exp com redução por k·ln 2 e 2⁻¹⁰, Taylor e duplicações;
//! - ln por Newton sobre exp.
//!
//! Erros medidos contra mpmath a 300 bits (`ferramentas/oraculos/f054_dd.py`), em ulp de
//! dupla-dupla (2⁻¹⁰⁴ relativo):
//! - soma, produto, divisão, raiz e exp: ≤ 0,5;
//! - leitura decimal: 3,3;
//! - ln: 6,1;
//! - potência inteira com até 40 produtos: 14.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

use super::NumericError;

/// Número dupla-dupla normalizado.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DoubleDouble {
    hi: f64,
    lo: f64,
}

/// Terceiro termo de ln 2: ln 2 − LN_2.hi − LN_2.lo (mpmath, `ferramentas/oraculos/f054_dd.py`).
const LN_2_TERCEIRO: f64 = 5.707_708_438_416_212e-34;

#[inline]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

#[inline]
fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    (s, b - (s - a))
}

#[inline]
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    (p, a.mul_add(b, -p))
}

impl DoubleDouble {
    /// π a ~32 dígitos.
    pub const PI: Self = Self {
        hi: std::f64::consts::PI,
        lo: 1.224_646_799_147_353_2e-16,
    };
    /// e a ~32 dígitos.
    pub const E: Self = Self {
        hi: std::f64::consts::E,
        lo: 1.445_646_891_729_250_2e-16,
    };
    /// ln 2 a ~32 dígitos.
    pub const LN_2: Self = Self {
        hi: std::f64::consts::LN_2,
        lo: 2.319_046_813_846_299_6e-17,
    };
    /// Espaçamento relativo: 2⁻¹⁰⁴.
    pub const EPSILON: f64 = 4.930_380_657_631_324e-32;

    /// Valor exato de um f64.
    pub const fn from_f64(x: f64) -> Self {
        Self { hi: x, lo: 0.0 }
    }

    /// hi + lo (soma exata, renormalizada).
    pub fn from_parts(hi: f64, lo: f64) -> Self {
        let (h, l) = two_sum(hi, lo);
        Self { hi: h, lo: l }
    }

    /// Parte alta (o f64 mais próximo do valor).
    pub fn hi(self) -> f64 {
        self.hi
    }

    /// Parte baixa.
    pub fn lo(self) -> f64 {
        self.lo
    }

    /// Arredonda para f64: normalizado, hi já é o f64 mais próximo (hi + lo arredonda para hi).
    pub fn to_f64(self) -> f64 {
        self.hi
    }

    /// Verdadeiro se não for ±∞ nem NaN.
    pub fn is_finite(self) -> bool {
        self.hi.is_finite()
    }

    /// Verdadeiro se for NaN.
    pub fn is_nan(self) -> bool {
        self.hi.is_nan()
    }

    /// Valor absoluto (pelo sinal de hi; normalizado, hi = 0 implica lo = 0).
    pub fn abs(self) -> Self {
        if self.hi.is_sign_negative() { -self } else { self }
    }

    fn mul_f(self, b: f64) -> Self {
        let (p, e) = two_prod(self.hi, b);
        let (h, l) = quick_two_sum(p, e + self.lo * b);
        Self { hi: h, lo: l }
    }

    /// Multiplica por 2ᵏ (exato, salvo estouro ou subnormais).
    pub fn ldexp(self, k: i32) -> Self {
        let f = |v: f64| {
            // Em dois passos para k fora da faixa de um único fator.
            let k1 = k / 2;
            v * 2f64.powi(k1) * 2f64.powi(k - k1)
        };
        Self {
            hi: f(self.hi),
            lo: f(self.lo),
        }
    }

    /// Raiz quadrada: x₀ = √hi em f64 e um passo de Newton em dupla-dupla,
    /// x = x₀ + (a − x₀²)/(2x₀).
    pub fn sqrt(self) -> Self {
        if self.hi <= 0.0 {
            return if self.hi == 0.0 && self.lo == 0.0 {
                self
            } else {
                Self::from_f64(f64::NAN)
            };
        }
        if self.hi.is_infinite() {
            return self;
        }
        let x = self.hi.sqrt();
        let (p, e) = two_prod(x, x);
        let r = (self - Self { hi: p, lo: e }).hi;
        Self::from_parts(x, r / (2.0 * x))
    }

    /// Potência inteira por quadrados sucessivos (inverso no fim para n < 0).
    pub fn powi(self, n: i32) -> Self {
        if n == 0 {
            return Self::from_f64(1.0);
        }
        let mut base = self;
        let mut m = n.unsigned_abs();
        let mut acc = Self::from_f64(1.0);
        while m > 0 {
            if m & 1 == 1 {
                acc = acc * base;
            }
            base = base * base;
            m >>= 1;
        }
        if n < 0 { Self::from_f64(1.0) / acc } else { acc }
    }

    /// e^x − 1 para |x| ≤ ln2/2: redução por 2¹⁰ (exata), Taylor com 11 termos (o primeiro
    /// omitido é < 1e-40) e 10 duplicações m ← m(2 + m), que mantêm a precisão relativa de m.
    fn expm1_reduzido(self) -> Self {
        let r = self.ldexp(-10);
        let um = Self::from_f64(1.0);
        let mut m = um;
        for k in (2..=12).rev() {
            m = um + r * m / Self::from_f64(k as f64);
        }
        let mut m = r * m;
        for _ in 0..10 {
            m = m * (Self::from_f64(2.0) + m);
        }
        m
    }

    /// Exponencial: x = k·ln2 + r com |r| ≤ ln2/2, e^x = 2ᵏ·(1 + expm1(r)).
    pub fn exp(self) -> Self {
        if self.hi.is_nan() {
            return self;
        }
        if self.hi > 709.8 {
            return Self::from_f64(f64::INFINITY);
        }
        if self.hi < -745.2 {
            return Self::from_f64(0.0);
        }
        let k = (self.hi / std::f64::consts::LN_2).round();
        // k·ln 2 como soma exata de termos f64 (dois produtos exatos e o terceiro termo de ln 2),
        // subtraídos um a um: o primeiro cancela x quase todo sem erro, e os demais erram
        // relativamente a |r|. Regressão: com LN_2·k em dupla-dupla, o arredondamento de lo·k
        // (~3e-30 com k ≈ 1000) dava até 143 ulp em exp(696).
        let (p, e) = two_prod(Self::LN_2.hi, k);
        let (q, f) = two_prod(Self::LN_2.lo, k);
        let mut r = self;
        for termo in [p, e, q, f, LN_2_TERCEIRO * k] {
            r = r - Self::from_f64(termo);
        }
        let e = Self::from_f64(1.0) + r.expm1_reduzido();
        // 2ᵏ em dois fatores, para k perto dos extremos (resultado subnormal ou perto do máximo).
        e.ldexp(k as i32)
    }

    /// Logaritmo natural: l₀ = ln(hi) em f64 e um passo de Newton l = l₀ + x·e^{−l₀} − 1, que
    /// leva o erro de ~1e-16 a ~1e-32.
    pub fn ln(self) -> Self {
        // NaN e negativos numa só condição.
        if matches!(self.hi.partial_cmp(&0.0), None | Some(Ordering::Less)) {
            return Self::from_f64(f64::NAN);
        }
        if self.hi == 0.0 {
            return Self::from_f64(f64::NEG_INFINITY);
        }
        if self.hi.is_infinite() {
            return self;
        }
        let l0 = Self::from_f64(self.hi.ln());
        l0 + self * (-l0).exp() - Self::from_f64(1.0)
    }

    /// Representação decimal com `digitos` algarismos significativos (1 a 34), em notação
    /// científica: "d.ddd…e±x".
    pub fn to_string_digits(self, digitos: usize) -> String {
        let digitos = digitos.clamp(1, 34);
        if self.hi.is_nan() {
            return "NaN".into();
        }
        if self.hi.is_infinite() {
            return if self.hi > 0.0 {
                "Inf".into()
            } else {
                "-Inf".into()
            };
        }
        if self.hi == 0.0 {
            return format!("{}0e+00", if self.hi.is_sign_negative() { "-" } else { "" });
        }
        let neg = self.hi < 0.0;
        let v = self.abs();
        // Expoente decimal: estimativa por log10(hi), corrigida para 1 ≤ m < 10.
        let mut e10 = v.hi.log10().floor() as i32;
        let mut m = v / Self::from_f64(10.0).powi(e10);
        // Comparações com o valor inteiro (hi e lo): com hi = 1 e lo < 0 o valor é < 1.
        // Regressão: comparando só hi, 1 − 1e-20 saía "1.000…e+00". (O ramo ≥ 10 é defensivo:
        // log10 de hi bem arredondado não fica abaixo do expoente.)
        let (um, dez) = (Self::from_f64(1.0), Self::from_f64(10.0));
        if m >= dez {
            m = m / dez;
            e10 += 1;
        } else if m < um {
            m = m * Self::from_f64(10.0);
            e10 -= 1;
        }
        // Dígitos por extração, com um dígito de guarda para arredondar.
        let mut ds: Vec<u8> = Vec::with_capacity(digitos + 1);
        for _ in 0..=digitos {
            // Piso do valor inteiro: hi inteiro com lo < 0 está abaixo de hi.
            let mut d = m.hi.floor();
            if d == m.hi && m.lo < 0.0 {
                d -= 1.0;
            }
            let d = d.clamp(0.0, 9.0);
            ds.push(d as u8);
            m = (m - Self::from_f64(d)) * Self::from_f64(10.0);
        }
        let guarda = ds.pop().unwrap_or(0);
        if guarda >= 5 {
            let mut i = ds.len();
            loop {
                if i == 0 {
                    ds.insert(0, 1);
                    ds.pop();
                    e10 += 1;
                    break;
                }
                i -= 1;
                if ds[i] == 9 {
                    ds[i] = 0;
                } else {
                    ds[i] += 1;
                    break;
                }
            }
        }
        let mut s = String::new();
        if neg {
            s.push('-');
        }
        s.push((b'0' + ds[0]) as char);
        if ds.len() > 1 {
            s.push('.');
            for d in &ds[1..] {
                s.push((b'0' + d) as char);
            }
        }
        s.push_str(&format!("e{}{:02}", if e10 < 0 { '-' } else { '+' }, e10.abs()));
        s
    }
}

impl From<f64> for DoubleDouble {
    fn from(x: f64) -> Self {
        Self::from_f64(x)
    }
}

impl Neg for DoubleDouble {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }
}

impl Add for DoubleDouble {
    type Output = Self;
    /// Soma precisa (Hida–Li–Bailey, "IEEE add"): erro ≤ 2 ulp de dupla-dupla.
    fn add(self, o: Self) -> Self {
        let (s, e) = two_sum(self.hi, o.hi);
        if !s.is_finite() {
            return Self::from_f64(s);
        }
        let (t, f) = two_sum(self.lo, o.lo);
        let (s, e) = quick_two_sum(s, e + t);
        let (h, l) = quick_two_sum(s, e + f);
        Self { hi: h, lo: l }
    }
}

impl Sub for DoubleDouble {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self + (-o)
    }
}

impl Mul for DoubleDouble {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        let (p, e) = two_prod(self.hi, o.hi);
        if !p.is_finite() {
            return Self::from_f64(p);
        }
        let (h, l) = quick_two_sum(p, e + (self.hi * o.lo + self.lo * o.hi));
        Self { hi: h, lo: l }
    }
}

impl Div for DoubleDouble {
    type Output = Self;
    /// Três quocientes parciais q₁ + q₂ + q₃ com restos exatos.
    fn div(self, o: Self) -> Self {
        let q1 = self.hi / o.hi;
        if !q1.is_finite() || o.hi.is_infinite() {
            return Self::from_f64(q1);
        }
        let r = self - o.mul_f(q1);
        let q2 = r.hi / o.hi;
        let r = r - o.mul_f(q2);
        let q3 = r.hi / o.hi;
        let (h, l) = quick_two_sum(q1, q2);
        Self { hi: h, lo: l } + Self::from_f64(q3)
    }
}

impl PartialOrd for DoubleDouble {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        match self.hi.partial_cmp(&o.hi)? {
            Ordering::Equal => self.lo.partial_cmp(&o.lo),
            c => Some(c),
        }
    }
}

impl fmt::Display for DoubleDouble {
    /// 32 algarismos significativos (ou `{:.N}` para N + 1 algarismos).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = f.precision().map_or(32, |p| p + 1);
        f.write_str(&self.to_string_digits(d))
    }
}

impl FromStr for DoubleDouble {
    type Err = NumericError;

    /// Lê decimal com sinal, ponto e expoente opcionais ("−1.25e-3"), e também "inf", "nan".
    /// Os algarismos são acumulados exatamente em dupla-dupla e escalados por 10^e (erro de
    /// poucos ulp de dupla-dupla).
    fn from_str(s: &str) -> Result<Self, NumericError> {
        let t = s.trim();
        let (neg, corpo) = match t.as_bytes().first() {
            Some(b'-') => (true, &t[1..]),
            Some(b'+') => (false, &t[1..]),
            _ => (false, t),
        };
        let sinal = |v: Self| if neg { -v } else { v };
        match corpo.to_ascii_lowercase().as_str() {
            "inf" | "infinity" => return Ok(sinal(Self::from_f64(f64::INFINITY))),
            "nan" => return Ok(Self::from_f64(f64::NAN)),
            _ => {}
        }
        let (mant, exp) = match corpo.find(['e', 'E']) {
            Some(i) => {
                let e: i32 = corpo[i + 1..]
                    .parse()
                    .map_err(|_| NumericError::InvalidDigit(corpo[i + 1..].to_string()))?;
                (&corpo[..i], e)
            }
            None => (corpo, 0),
        };
        let mut v = Self::default();
        let mut casas = 0i32;
        let mut viu_ponto = false;
        let mut algarismos = 0usize;
        let dez = Self::from_f64(10.0);
        for c in mant.chars() {
            match c {
                '.' if !viu_ponto => viu_ponto = true,
                '0'..='9' => {
                    // Além de 36 algarismos significativos, o resto não muda a dupla-dupla.
                    if algarismos < 36 {
                        v = v * dez + Self::from_f64(f64::from(c as u8 - b'0'));
                        if v.hi != 0.0 {
                            algarismos += 1;
                        }
                        if viu_ponto {
                            casas += 1;
                        }
                    } else if !viu_ponto {
                        casas -= 1;
                    }
                }
                '_' => {}
                _ => return Err(NumericError::InvalidDigit(c.to_string())),
            }
        }
        if mant.chars().all(|c| !c.is_ascii_digit()) {
            return Err(NumericError::InvalidDigit(s.to_string()));
        }
        let e = exp.saturating_sub(casas);
        let v = if e >= 0 {
            v * dez.powi(e.min(400))
        } else if e >= -300 {
            v / dez.powi(-e)
        } else {
            // Em duas etapas, para não estourar 10^(−e).
            v / dez.powi(300) / dez.powi((-e - 300).min(400))
        };
        Ok(sinal(v))
    }
}

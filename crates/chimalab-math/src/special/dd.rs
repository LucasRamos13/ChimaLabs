//! Aritmética dupla-dupla (valor = hi + lo, ~32 dígitos), só para expoentes de funções especiais
//! em que somas de termos grandes perderiam dígitos em f64 (ln B(a, b) com argumentos grandes).
//!
//! Algoritmos clássicos de transformações exatas: soma de Knuth (two-sum), produto exato por FMA
//! (two-prod) e normalização (quick-two-sum); ver T. J. Dekker, "A floating-point technique for
//! extending the available precision", Numer. Math. 18 (1971) 224–242. `exp` em argumento pequeno
//! por Taylor com redução 2⁻¹⁰ e duplicações; `ln` por um passo de Newton sobre `exp`.

use super::elementary::{frexp, ldexp};

/// Número dupla-dupla normalizado: |lo| ≤ ½ulp(hi).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Dd {
    pub(crate) hi: f64,
    pub(crate) lo: f64,
}

/// ln 2 em dupla-dupla (conferido em ferramentas/oraculos/f042_beta_gammainc.py, mpmath).
pub(crate) const LN2: Dd = Dd {
    hi: std::f64::consts::LN_2,
    lo: 2.319_046_813_846_299_6e-17,
};
/// ln √(2π) em dupla-dupla (idem).
pub(crate) const LN_SQRT_2PI: Dd = Dd {
    hi: 0.918_938_533_204_672_8,
    lo: -3.878_294_158_067_241_4e-17,
};

/// Soma exata: a + b = s + e (Knuth).
fn two_sum(a: f64, b: f64) -> Dd {
    let s = a + b;
    let bb = s - a;
    Dd {
        hi: s,
        lo: (a - (s - bb)) + (b - bb),
    }
}

/// Soma exata quando |a| ≥ |b|.
fn quick_two_sum(a: f64, b: f64) -> Dd {
    let s = a + b;
    Dd {
        hi: s,
        lo: b - (s - a),
    }
}

/// Produto exato: a·b = p + e (FMA).
fn two_prod(a: f64, b: f64) -> Dd {
    let p = a * b;
    Dd {
        hi: p,
        lo: a.mul_add(b, -p),
    }
}

impl Dd {
    pub(crate) fn from(x: f64) -> Self {
        Dd { hi: x, lo: 0.0 }
    }

    /// a + b exato em dupla-dupla.
    pub(crate) fn sum(a: f64, b: f64) -> Self {
        two_sum(a, b)
    }

    pub(crate) fn add(self, o: Dd) -> Dd {
        let s = two_sum(self.hi, o.hi);
        let t = two_sum(self.lo, o.lo);
        let s = quick_two_sum(s.hi, s.lo + t.hi);
        quick_two_sum(s.hi, s.lo + t.lo)
    }

    pub(crate) fn neg(self) -> Dd {
        Dd {
            hi: -self.hi,
            lo: -self.lo,
        }
    }

    pub(crate) fn sub(self, o: Dd) -> Dd {
        self.add(o.neg())
    }

    pub(crate) fn mul(self, o: Dd) -> Dd {
        let p = two_prod(self.hi, o.hi);
        quick_two_sum(p.hi, p.lo + (self.hi * o.lo + self.lo * o.hi))
    }

    pub(crate) fn mul_f(self, b: f64) -> Dd {
        let p = two_prod(self.hi, b);
        quick_two_sum(p.hi, p.lo + self.lo * b)
    }

    /// Divisão por um f64: q₁ = hi/b, resto exato r = self − q₁·b, q₂ = r/b.
    pub(crate) fn div_f(self, b: f64) -> Dd {
        let q1 = self.hi / b;
        let r = self.sub(two_prod(q1, b));
        quick_two_sum(q1, r.hi / b)
    }

    /// Converte para f64 (arredondado).
    /// Normalizado (|lo| ≤ ½ulp(hi)), hi + lo arredonda para hi.
    pub(crate) fn to_f64(self) -> f64 {
        self.hi
    }
}

/// e^x − 1 para |x| ≤ 0,75: redução x/2¹⁰ (exata), Taylor com 11 termos (o primeiro omitido é
/// < 1e-38) e 10 duplicações e^{2y} − 1 = m(2 + m), que preservam a precisão relativa de m.
fn expm1_small(x: f64) -> Dd {
    let r = Dd::from(x / 1024.0);
    // Horner: m = r(1 + r/2(1 + r/3(…)))
    let mut m = Dd::from(1.0);
    for k in (2..=12).rev() {
        // Divisão dupla-dupla: 1/k em f64 não é exato e erraria ~1e-23 relativo.
        m = Dd::from(1.0).add(r.mul(m).div_f(k as f64));
    }
    let mut m = r.mul(m);
    for _ in 0..10 {
        m = m.mul(Dd::from(2.0).add(m));
    }
    m
}

/// ln y em dupla-dupla, y > 0 finito: y = f·2ᵉ com f ∈ [½, 1), ln f por Newton
/// l₁ = l₀ + (f·e^{−l₀} − 1), que leva o erro de l₀ (~1e-16) a ~1e-32.
pub(crate) fn ln(y: Dd) -> Dd {
    let (f, e) = frexp(y.hi);
    let s = Dd {
        hi: f,
        lo: ldexp(y.lo, -e),
    };
    let l0 = f.ln();
    let em = expm1_small(-l0); // e^{−l₀} − 1
    // f·e^{−l₀} − 1 = f·em + (f − 1)
    let t = s.mul(em).add(s.add(Dd::from(-1.0)));
    Dd::from(l0).add(t).add(LN2.mul_f(e as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transformacoes_exatas() {
        let s = two_sum(1.0, 1e-17);
        assert_eq!((s.hi, s.lo), (1.0, 1e-17));
        let p = two_prod(1.0 + 2f64.powi(-30), 1.0 + 2f64.powi(-30));
        assert_eq!((p.hi, p.lo), (1.0 + 2f64.powi(-29), 2f64.powi(-60)));
        let q = quick_two_sum(1.0, 1e-17);
        assert_eq!((q.hi, q.lo), (1.0, 1e-17));
        let a = Dd { hi: 1.0, lo: 1e-17 };
        assert_eq!(a.sub(a).to_f64(), 0.0);
        assert_eq!(a.mul_f(2.0), Dd { hi: 2.0, lo: 2e-17 });
        assert_eq!(a.mul(Dd::from(3.0)), Dd { hi: 3.0, lo: 3e-17 });
        assert_eq!(
            Dd::sum(1.0, 2f64.powi(-60)),
            Dd {
                hi: 1.0,
                lo: 2f64.powi(-60)
            }
        );
    }

    #[test]
    fn ln_de_potencias_de_2_e_de_valores_conhecidos() {
        // ln 2ᵏ = k·ln 2 exatamente na dupla-dupla.
        for k in [-1074i32, -20, 1, 7, 1023] {
            let l = ln(Dd::from(ldexp(1.0, k)));
            let want = LN2.mul_f(k as f64);
            assert!(
                (l.sub(want).to_f64()).abs() < 1e-29 * want.hi.abs().max(1.0),
                "k = {k}: {l:?} {want:?}"
            );
        }
        // ln(e) a 1e-31: e em dupla-dupla (mpmath).
        let e = Dd {
            hi: std::f64::consts::E,
            lo: 1.445_646_891_729_250_2e-16,
        };
        assert!(ln(e).sub(Dd::from(1.0)).to_f64().abs() < 1e-31);
        // ln(√(2π)) = ½ ln(2π), com 2π em dupla-dupla.
        let two_pi = Dd {
            hi: 2.0 * std::f64::consts::PI,
            lo: 2.449_293_598_294_706_4e-16,
        };
        let half = ln(two_pi).mul_f(0.5);
        assert!(half.sub(LN_SQRT_2PI).to_f64().abs() < 1e-30);
    }

    #[test]
    fn soma_no_nivel_de_1e_30() {
        // (1 + 2⁻⁶⁰) + (2⁻⁵⁵ + 2⁻¹⁰⁰) = 1 + (2⁻⁵⁵ + 2⁻⁶⁰ + 2⁻¹⁰⁰), exato.
        let a = Dd {
            hi: 1.0,
            lo: 2f64.powi(-60),
        };
        let b = Dd {
            hi: 2f64.powi(-55),
            lo: 2f64.powi(-100),
        };
        let s = a.add(b);
        assert_eq!(s.hi, 1.0);
        assert_eq!(s.lo, 2f64.powi(-55) + 2f64.powi(-60) + 2f64.powi(-100));
        // Produto com parte baixa: (1 + 2⁻⁶⁰)(3 + 2⁻⁵⁴) = 3 + 2⁻⁵⁴ + 3·2⁻⁶⁰ + 2⁻¹¹⁴.
        let p = a.mul(Dd {
            hi: 3.0,
            lo: 2f64.powi(-54),
        });
        assert_eq!(p.hi, 3.0);
        assert!((p.lo - (2f64.powi(-54) + 3.0 * 2f64.powi(-60))).abs() <= 2f64.powi(-105));
        assert_eq!(Dd { hi: 2.0, lo: 1e-17 }.to_f64(), 2.0);
        // Cancelamento: (1 + 2⁻⁶⁰) + (−1 + 2⁻¹²⁰) = 2⁻⁶⁰ + 2⁻¹²⁰; só a segunda correção guarda 2⁻¹²⁰.
        let c = a.add(Dd {
            hi: -1.0,
            lo: 2f64.powi(-120),
        });
        assert_eq!((c.hi, c.lo), (2f64.powi(-60), 2f64.powi(-120)));
        // ln com parte baixa: ln(1 + 2⁻⁶⁰) = 2⁻⁶⁰ − 2⁻¹²¹ + …. A precisão do ln dupla-dupla é
        // absoluta (~1e-32: perto de 1 ele soma ln ½ + ln 2), que é o que os expoentes usam.
        let l = ln(a);
        assert!(l.sub(Dd::from(2f64.powi(-60))).to_f64().abs() <= 1e-31, "{l:?}");
    }

    #[test]
    fn divisao() {
        // Regressão: 1/k em f64 na série de Taylor deixava ln com erro de ~3e-24.
        let third = Dd::from(1.0).div_f(3.0);
        assert!(third.mul_f(3.0).sub(Dd::from(1.0)).to_f64().abs() < 1e-32);
    }

    #[test]
    fn expm1_pequeno() {
        let m = expm1_small(1e-20);
        assert!((m.to_f64() - 1e-20).abs() < 1e-36);
        let m = expm1_small(0.5);
        assert!((m.to_f64() - 0.5f64.exp_m1()).abs() < 2e-16);
    }
}

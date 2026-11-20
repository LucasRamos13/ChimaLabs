//! Casos de borda complementares (CLAUDE.md §7.4), escritos a partir das linhas que a
//! cobertura apontou como não executadas.

use chimalab_math::signal::nextpow2;
use chimalab_math::special::{expint, lcm};

/// Invariante de `nextpow2`: `2^(p−1) < |x| ≤ 2^p`, verificado em todas as potências de 2
/// representáveis (inclusive subnormais) e nos vizinhos imediatos de cada uma.
#[test]
fn nextpow2_em_todas_as_potencias_e_vizinhos() {
    for e in -1074..=1023 {
        // 2^e exato, construído pelos bits (powi perde os subnormais).
        let pot = if e >= -1022 {
            f64::from_bits(((e + 1023) as u64) << 52)
        } else {
            f64::from_bits(1u64 << (e + 1074))
        };
        for x in [
            pot,
            f64::from_bits(pot.to_bits().saturating_sub(1)),
            f64::from_bits(pot.to_bits() + 1),
        ] {
            if x == 0.0 || !x.is_finite() {
                continue;
            }
            for v in [x, -x] {
                let p = nextpow2(v);
                assert_eq!(p, p.trunc(), "x = {v:e}");
                assert!(2f64.powf(p) >= x, "x = {v:e}, p = {p}");
                assert!(2f64.powf(p - 1.0) < x, "x = {v:e}, p = {p}");
            }
        }
    }
    assert_eq!(nextpow2(f64::MIN_POSITIVE / 4.0), -1024.0);
    assert_eq!(nextpow2(5e-324), -1074.0);
    assert_eq!(nextpow2(f64::MAX), 1024.0);
    assert_eq!(nextpow2(-0.0), 0.0);
    assert_eq!(nextpow2(f64::NEG_INFINITY), f64::INFINITY);
}

#[test]
fn lcm_com_zero() {
    assert_eq!(lcm(0.0, 0.0).unwrap(), 0.0);
    assert_eq!(lcm(0.0, 7.0).unwrap(), 0.0);
    assert_eq!(lcm(-4.0, 6.0).unwrap(), 12.0);
}

#[test]
fn expint_em_zero_e_negativos() {
    assert_eq!(expint(0.0), f64::INFINITY);
    assert_eq!(expint(-0.0), f64::INFINITY);
    assert!(expint(-1.0).is_nan());
    assert!(expint(f64::NAN).is_nan());
    assert_eq!(expint(f64::INFINITY), 0.0);
}

//! Integrais elípticas completas e funções elípticas de Jacobi pela média aritmético-geométrica
//! (DLMF 19.8.1, 19.8.6, 22.20.1).

use std::f64::consts::PI;

/// Integrais elípticas completas `(K(m), E(m))` para m ≤ 1 (parâmetro m = k²).
pub fn ellipke(m: f64) -> (f64, f64) {
    if m.is_nan() || m > 1.0 {
        return (f64::NAN, f64::NAN);
    }
    if m == 1.0 {
        return (f64::INFINITY, 1.0);
    }
    let (mut a, mut b) = (1.0f64, (1.0 - m).sqrt());
    let mut sum = 0.5 * m; // 2^(n−1)·c_n² com c₀² = m
    let mut pow2 = 0.5;
    for _ in 0..60 {
        let c = 0.5 * (a - b);
        let an = 0.5 * (a + b);
        b = (a * b).sqrt();
        a = an;
        pow2 *= 2.0;
        sum += pow2 * c * c;
        if c.abs() <= f64::EPSILON * a {
            break;
        }
    }
    let k = PI / (2.0 * a);
    (k, k * (1.0 - sum))
}

/// Funções elípticas de Jacobi `(sn, cn, dn)` de `u` com parâmetro 0 ≤ m ≤ 1
/// (transformação descendente de Landen pela MAG).
pub fn ellipj(u: f64, m: f64) -> (f64, f64, f64) {
    if u.is_nan() || m.is_nan() || !(0.0..=1.0).contains(&m) {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    if m == 0.0 {
        return (u.sin(), u.cos(), 1.0);
    }
    if m == 1.0 {
        let s = 1.0 / u.cosh();
        return (u.tanh(), s, s);
    }
    // Redução pelo período 4K(m) de sn e cn (dn tem período 2K): o erro absoluto deixa de
    // crescer com |u| além do condicionamento intrínseco (regressão encontrada por proptest).
    let (k, _) = ellipke(m);
    let period = 4.0 * k;
    let u = u - (u / period).round() * period;
    let mut a = vec![1.0f64];
    let mut c = vec![m.sqrt()];
    let mut b = (1.0 - m).sqrt();
    while c.last().unwrap().abs() > f64::EPSILON && a.len() < 64 {
        let an = a.last().unwrap();
        let cn = 0.5 * (an - b);
        let bn = (an * b).sqrt();
        a.push(0.5 * (an + b));
        c.push(cn);
        b = bn;
    }
    let n = a.len() - 1;
    let mut phi = 2f64.powi(n as i32) * a[n] * u;
    for k in (1..=n).rev() {
        phi = 0.5 * (phi + (c[k] / a[k] * phi.sin()).asin());
    }
    let (s, co) = (phi.sin(), phi.cos());
    // dn² = 1 − m·sn² = (1 − m) + m·cn². Para m ≤ ½ a primeira forma não tem cancelamento; para
    // m > ½, 1 − m é exata (Sterbenz) e a segunda só soma termos positivos. Assim dn² + m·sn² = 1
    // vale até o arredondamento (a fórmula pelo ângulo, co/cos(φ₁ − φ₀), deixava resíduo de
    // ~1e-12 perto de m = 1: regressão encontrada por proptest).
    let dn = if m <= 0.5 {
        (1.0 - m * s * s).sqrt()
    } else {
        ((1.0 - m) + m * co * co).sqrt()
    };
    (s, co, dn)
}

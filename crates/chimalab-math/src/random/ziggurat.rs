//! Normal padrão pelo ziggurat de 256 camadas (Marsaglia & Tsang, "The ziggurat method for
//! generating random variables", J. Stat. Softw. 5, 2000).
//!
//! As camadas têm a mesma área v; x₁ = r é o início da cauda, x₀ = v/f(r) é a largura da camada da
//! base (que inclui a cauda), x_{i+1} = f⁻¹(f(xᵢ) + v/xᵢ) e x₂₅₆ = 0, com f(x) = e^{−x²/2}.
//! r e v foram obtidos resolvendo a recorrência a 40 dígitos (mpmath): o resíduo de fechamento
//! em x₂₅₆ é ~1e-39 (ver o teste `tabelas_fecham`). O índice da camada vem dos 8 bits baixos e a
//! uniforme dos 53 bits altos de uma mesma palavra de 64 bits: são independentes (correção de
//! Doornik, 2005, ao uso dos mesmos bits no artigo original).

use std::sync::OnceLock;

use super::Rng;

pub(crate) const R: f64 = 3.654_152_885_361_009;
pub(crate) const V: f64 = 0.004_928_673_233_974_655;

/// (x₀…x₂₅₆, f(x₀)…f(x₂₅₆)).
pub(crate) fn tables() -> &'static ([f64; 257], [f64; 257]) {
    static T: OnceLock<([f64; 257], [f64; 257])> = OnceLock::new();
    T.get_or_init(|| {
        let f = |x: f64| (-0.5 * x * x).exp();
        let mut x = [0.0; 257];
        x[0] = V / f(R);
        x[1] = R;
        for i in 1..255 {
            // Com r e v exatos, f(xᵢ) + v/xᵢ < 1 para i ≤ 254 (teste `tabelas_fecham`).
            let y = f(x[i]) + V / x[i];
            x[i + 1] = (-2.0 * y.ln()).sqrt();
        }
        x[256] = 0.0;
        let mut fx = [0.0; 257];
        for (fi, xi) in fx.iter_mut().zip(&x) {
            *fi = f(*xi);
        }
        (x, fx)
    })
}

pub(crate) fn normal(rng: &mut Rng) -> f64 {
    let (x, fx) = tables();
    loop {
        let b = rng.next_u64();
        let i = (b & 0xff) as usize;
        let sign = if b & 0x100 == 0 { 1.0 } else { -1.0 };
        let u = ((b >> 11) as f64 + 0.5) * (1.0 / (1u64 << 53) as f64);
        let z = u * x[i];
        if z < x[i + 1] {
            return sign * z;
        }
        if i == 0 {
            // Cauda além de r (Marsaglia, 1964): x = −ln(u₁)/r, y = −ln(u₂), aceita se 2y > x².
            loop {
                let t = -rng.uniform().ln() / R;
                let y = -rng.uniform().ln();
                if 2.0 * y > t * t {
                    return sign * (R + t);
                }
            }
        }
        // Faixa da camada i: altura uniforme entre f(xᵢ) e f(xᵢ₊₁); aceita abaixo da curva.
        let y = fx[i] + rng.uniform() * (fx[i + 1] - fx[i]);
        if y < (-0.5 * z * z).exp() {
            return sign * z;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabelas_fecham() {
        let (x, fx) = tables();
        // Camadas estritamente decrescentes e finitas; x₁ = r; x₂₅₆ = 0.
        assert_eq!(x[1], R);
        assert!(x.windows(2).all(|w| w[0] > w[1]), "não decrescente");
        assert!(x.iter().all(|v| v.is_finite()) && x[256] == 0.0);
        // A última camada fecha em f = 1: f(x₂₅₅) + v/x₂₅₅ = 1 (a erro de arredondamento
        // acumulado na recorrência).
        let fecho = fx[255] + V / x[255];
        assert!((fecho - 1.0).abs() < 1e-11, "{fecho}");
        // Área de cada camada = v: xᵢ·(f(xᵢ₊₁) − f(xᵢ)) para i ≥ 1.
        for i in 1..255 {
            assert!((x[i] * (fx[i + 1] - fx[i]) - V).abs() < 1e-14, "camada {i}");
        }
        // Camada da base: x₀·f(r) = v.
        assert!((x[0] * fx[1] - V).abs() < 1e-17);
        assert_eq!(fx[256], 1.0);
    }
}

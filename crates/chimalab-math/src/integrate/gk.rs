//! Regra de Gauss–Kronrod 7–15 em [−1, 1].

use chimalab_array::Scalar;

// Gerado por ferramentas/oraculos/f048_gauss_kronrod.py (decimal, 60 dígitos);
// exatidão conferida: K15 até x²², G7 até x¹³.
/// Nós de Kronrod em [0, 1], decrescentes; os de índice ímpar são os nós de Gauss.
const XK: [f64; 8] = [
    0.9914553711208126,
    0.9491079123427585,
    0.8648644233597691,
    0.7415311855993945,
    0.5860872354676911,
    0.4058451513773972,
    0.20778495500789848,
    0.0,
];
/// Pesos de Kronrod correspondentes a `XK`.
const WK: [f64; 8] = [
    0.022935322010529224,
    0.06309209262997856,
    0.10479001032225019,
    0.14065325971552592,
    0.1690047266392679,
    0.19035057806478542,
    0.20443294007529889,
    0.20948214108472782,
];
/// Pesos de Gauss dos nós `XK[1]`, `XK[3]`, `XK[5]` e `XK[7]` (= 0).
const WG: [f64; 4] = [
    0.1294849661688697,
    0.27970539148927664,
    0.3818300505051189,
    0.4179591836734694,
];

/// Aplica a regra 7–15 a `f` no intervalo [t0, t1]. Devolve (Kronrod, Gauss); os extremos não são
/// avaliados.
pub(crate) fn gk15<T: Scalar>(f: &mut impl FnMut(f64) -> T, t0: f64, t1: f64) -> (T, T) {
    let c = 0.5 * (t0 + t1);
    let h = 0.5 * (t1 - t0);
    let fc = f(c);
    let mut k = fc.scale(WK[7]);
    let mut g = fc.scale(WG[3]);
    for i in 0..7 {
        let dx = h * XK[i];
        let s = f(c - dx) + f(c + dx);
        k += s.scale(WK[i]);
        if i % 2 == 1 {
            g += s.scale(WG[i / 2]);
        }
    }
    (k.scale(h), g.scale(h))
}

/// Os 15 nós de Kronrod em [−1, 1] (crescentes), com os pesos de Kronrod e os de Gauss (zero
/// nos nós que não são de Gauss), para regras produto.
pub(crate) fn gk15_nodes() -> ([f64; 15], [f64; 15], [f64; 15]) {
    let mut x = [0.0; 15];
    let mut wk = [0.0; 15];
    let mut wg = [0.0; 15];
    for i in 0..8 {
        // XK é decrescente em [0, 1]: índice i ↦ −XK[i] à esquerda e +XK[i] à direita.
        let (l, r) = (i, 14 - i);
        x[l] = -XK[i];
        x[r] = XK[i];
        wk[l] = WK[i];
        wk[r] = WK[i];
        if i % 2 == 1 {
            wg[l] = WG[i / 2];
            wg[r] = WG[i / 2];
        }
    }
    (x, wk, wg)
}

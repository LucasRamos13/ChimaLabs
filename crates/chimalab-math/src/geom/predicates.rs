//! Predicados geométricos com sinal exato: orient2d e incircle.
//!
//! O cálculo segue o esquema de J. R. Shewchuk (Discrete Comput. Geom. 18, 1997):
//! 1. Avalia o determinante em ponto flutuante.
//! 2. Compara-o com um limite rigoroso do erro de arredondamento. O sinal é certo se o valor estiver acima do limite.
//! 3. Caso contrário, reavalia exatamente por expansões: somas não sobrepostas de f64, por soma e produto exatos (two-sum e FMA).
//!
//! A etapa 3 é exata para qualquer entrada finita sem overflow.

const EPS: f64 = f64::EPSILON / 2.0; // 2⁻⁵³, unidade de arredondamento
const ORIENT_ERR: f64 = (3.0 + 16.0 * EPS) * EPS;
const INCIRCLE_ERR: f64 = (10.0 + 96.0 * EPS) * EPS;

/// a + b = s + e exatamente (Knuth).
#[inline]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    let e = (a - (s - bb)) + (b - bb);
    (s, e)
}

/// a·b = p + e exatamente (com FMA).
#[inline]
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    (p, a.mul_add(b, -p))
}

/// Soma de uma expansão com um escalar (Shewchuk, Grow-Expansion), sem zeros.
fn grow(e: &[f64], b: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(e.len() + 1);
    let mut q = b;
    for &ei in e {
        let (s, h) = two_sum(q, ei);
        if h != 0.0 {
            out.push(h);
        }
        q = s;
    }
    if q != 0.0 || out.is_empty() {
        out.push(q);
    }
    out
}

/// Soma de expansões.
fn add(e: &[f64], f: &[f64]) -> Vec<f64> {
    let mut out = e.to_vec();
    for &fi in f {
        out = grow(&out, fi);
    }
    out
}

/// Expansão vezes escalar (Shewchuk, Scale-Expansion), sem zeros.
fn scale(e: &[f64], b: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(2 * e.len());
    let mut q = 0.0;
    for &ei in e {
        let (p, pe) = two_prod(ei, b);
        let (s1, h1) = two_sum(q, pe);
        if h1 != 0.0 {
            out.push(h1);
        }
        let (s2, h2) = two_sum(p, s1);
        if h2 != 0.0 {
            out.push(h2);
        }
        q = s2;
    }
    if q != 0.0 || out.is_empty() {
        out.push(q);
    }
    out
}

/// Produto de expansões.
fn mul(e: &[f64], f: &[f64]) -> Vec<f64> {
    let mut acc = vec![0.0];
    for &fi in f {
        acc = add(&acc, &scale(e, fi));
    }
    acc
}

fn neg(e: &[f64]) -> Vec<f64> {
    e.iter().map(|v| -v).collect()
}

/// Sinal de uma expansão: o do componente de maior magnitude (o último não nulo).
fn sign(e: &[f64]) -> f64 {
    e.iter().rev().find(|v| **v != 0.0).map_or(0.0, |v| v.signum())
}

/// a − b exato como expansão.
fn diff(a: f64, b: f64) -> Vec<f64> {
    let (s, e) = two_sum(a, -b);
    if e == 0.0 { vec![s] } else { vec![e, s] }
}

/// Sinal de det[[ax − cx, ay − cy], [bx − cx, by − cy]]: > 0 se a, b, c estão em sentido
/// anti-horário, < 0 se horário, 0 se colineares. Exato.
pub fn orient2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    let detleft = (a[0] - c[0]) * (b[1] - c[1]);
    let detright = (a[1] - c[1]) * (b[0] - c[0]);
    let det = detleft - detright;
    let bound = ORIENT_ERR * (detleft.abs() + detright.abs());
    if det > bound || -det > bound {
        return det.signum();
    }
    orient2d_exact(a, b, c)
}

fn orient2d_exact(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    let acx = diff(a[0], c[0]);
    let bcy = diff(b[1], c[1]);
    let acy = diff(a[1], c[1]);
    let bcx = diff(b[0], c[0]);
    sign(&add(&mul(&acx, &bcy), &neg(&mul(&acy, &bcx))))
}

/// Sinal de incircle: > 0 se d está estritamente dentro do círculo por a, b, c (com a, b, c em
/// sentido anti-horário), < 0 se fora, 0 se cocircular. Com a, b, c horários, o sinal se
/// inverte. Exato.
pub fn incircle(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> f64 {
    let (adx, ady) = (a[0] - d[0], a[1] - d[1]);
    let (bdx, bdy) = (b[0] - d[0], b[1] - d[1]);
    let (cdx, cdy) = (c[0] - d[0], c[1] - d[1]);
    let bdxcdy = bdx * cdy;
    let cdxbdy = cdx * bdy;
    let alift = adx * adx + ady * ady;
    let cdxady = cdx * ady;
    let adxcdy = adx * cdy;
    let blift = bdx * bdx + bdy * bdy;
    let adxbdy = adx * bdy;
    let bdxady = bdx * ady;
    let clift = cdx * cdx + cdy * cdy;
    let det = alift * (bdxcdy - cdxbdy) + blift * (cdxady - adxcdy) + clift * (adxbdy - bdxady);
    let permanent = (bdxcdy.abs() + cdxbdy.abs()) * alift
        + (cdxady.abs() + adxcdy.abs()) * blift
        + (adxbdy.abs() + bdxady.abs()) * clift;
    let bound = INCIRCLE_ERR * permanent;
    if det > bound || -det > bound {
        return det.signum();
    }
    incircle_exact(a, b, c, d)
}

fn incircle_exact(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> f64 {
    let (adx, ady) = (diff(a[0], d[0]), diff(a[1], d[1]));
    let (bdx, bdy) = (diff(b[0], d[0]), diff(b[1], d[1]));
    let (cdx, cdy) = (diff(c[0], d[0]), diff(c[1], d[1]));
    let lift = |x: &[f64], y: &[f64]| add(&mul(x, x), &mul(y, y));
    let cross = |x1: &[f64], y1: &[f64], x2: &[f64], y2: &[f64]| add(&mul(x1, y2), &neg(&mul(y1, x2)));
    let t1 = mul(&lift(&adx, &ady), &cross(&bdx, &bdy, &cdx, &cdy));
    let t2 = mul(&lift(&bdx, &bdy), &cross(&cdx, &cdy, &adx, &ady));
    let t3 = mul(&lift(&cdx, &cdy), &cross(&adx, &ady, &bdx, &bdy));
    sign(&add(&add(&t1, &t2), &t3))
}

const ORIENT3_ERR: f64 = (7.0 + 56.0 * EPS) * EPS;

/// Sinal de det[a − d; b − d; c − d]: > 0 se d está abaixo do plano por a, b, c (orientados em
/// sentido anti-horário vistos de cima), < 0 acima, 0 se coplanares. Exato.
pub fn orient3d(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> f64 {
    let (adx, ady, adz) = (a[0] - d[0], a[1] - d[1], a[2] - d[2]);
    let (bdx, bdy, bdz) = (b[0] - d[0], b[1] - d[1], b[2] - d[2]);
    let (cdx, cdy, cdz) = (c[0] - d[0], c[1] - d[1], c[2] - d[2]);
    let (bdxcdy, cdxbdy) = (bdx * cdy, cdx * bdy);
    let (cdxady, adxcdy) = (cdx * ady, adx * cdy);
    let (adxbdy, bdxady) = (adx * bdy, bdx * ady);
    let det = adz * (bdxcdy - cdxbdy) + bdz * (cdxady - adxcdy) + cdz * (adxbdy - bdxady);
    let permanent = (bdxcdy.abs() + cdxbdy.abs()) * adz.abs()
        + (cdxady.abs() + adxcdy.abs()) * bdz.abs()
        + (adxbdy.abs() + bdxady.abs()) * cdz.abs();
    let bound = ORIENT3_ERR * permanent;
    if det > bound || -det > bound {
        return det.signum();
    }
    orient3d_exact(a, b, c, d)
}

fn orient3d_exact(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> f64 {
    let e = |u: f64, v: f64| diff(u, v);
    let (ax, ay, az) = (e(a[0], d[0]), e(a[1], d[1]), e(a[2], d[2]));
    let (bx, by, bz) = (e(b[0], d[0]), e(b[1], d[1]), e(b[2], d[2]));
    let (cx, cy, cz) = (e(c[0], d[0]), e(c[1], d[1]), e(c[2], d[2]));
    let cross = |x1: &[f64], y1: &[f64], x2: &[f64], y2: &[f64]| add(&mul(x1, y2), &neg(&mul(y1, x2)));
    let t1 = mul(&az, &cross(&bx, &by, &cx, &cy));
    let t2 = mul(&bz, &cross(&cx, &cy, &ax, &ay));
    let t3 = mul(&cz, &cross(&ax, &ay, &bx, &by));
    sign(&add(&add(&t1, &t2), &t3))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O caminho exato e o filtrado concordam onde o filtro decide.
    #[test]
    fn exato_concorda_com_filtro() {
        let pts = [
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.3, 0.7],
            [2.5, -1.0],
            [-1.5, 0.25],
        ];
        for &a in &pts {
            for &b in &pts {
                for &c in &pts {
                    assert_eq!(orient2d(a, b, c), orient2d_exact(a, b, c));
                    for &d in &pts {
                        assert_eq!(incircle(a, b, c, d), incircle_exact(a, b, c, d));
                    }
                }
            }
        }
        assert_eq!(sign(&[]), 0.0);
    }

    /// Avaliação ingênua (sem filtro) dos três determinantes, só para medir o perigo.
    fn ingenuo2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
        ((a[0] - c[0]) * (b[1] - c[1]) - (a[1] - c[1]) * (b[0] - c[0])).signum()
    }

    /// Entradas quase degeneradas (L. Kettner et al., "Classroom examples of robustness problems
    /// in geometric computations", Comput. Geom. 40, 2008): grades de pontos a poucos ulp de uma
    /// reta, de um círculo e de um plano, com coordenadas deslocadas para amplificar o
    /// cancelamento. O predicado filtrado tem de concordar sempre com o exato; a avaliação ingênua
    /// erra o sinal numa parte dos casos (o teste confere que o perigo existe). Um limite de erro
    /// menor que o de Shewchuk deixaria passar esses sinais errados.
    #[test]
    fn filtro_em_entradas_quase_degeneradas() {
        let ulp = |x: f64, k: i64| f64::from_bits((x.to_bits() as i64 + k) as u64);
        // orient2d: a perto da reta y = x, b e c sobre ela (Kettner et al., figura 1).
        let mut erros_ingenuos = 0;
        for i in 0..64 {
            for j in 0..64 {
                let a = [ulp(0.5, i), ulp(0.5, j)];
                for (b, c) in [
                    ([12.0, 12.0], [24.0, 24.0]),
                    ([24.0, 24.0], [12.0, 12.0]),
                    ([17.3, 17.3], [0.7, 0.7]),
                ] {
                    let exato = orient2d_exact(a, b, c);
                    assert_eq!(orient2d(a, b, c), exato, "{a:?}");
                    assert_eq!(orient2d(b, c, a), exato, "{a:?}");
                    erros_ingenuos += (ingenuo2(a, b, c) != exato) as usize;
                }
            }
        }
        assert!(erros_ingenuos > 100, "{erros_ingenuos}");
        // incircle: d a poucos ulp do círculo unitário por a, b, c, tudo deslocado de (o, o).
        for o in [0.0, 1e3, 1e8] {
            let t = |p: [f64; 2]| [p[0] + o, p[1] + o];
            let (a, b, c) = (t([1.0, 0.0]), t([0.0, 1.0]), t([-1.0, 0.0]));
            for i in -20..20 {
                for j in -20..20 {
                    let d = t([ulp(0.6, i), ulp(0.8, j)]);
                    assert_eq!(incircle(a, b, c, d), incircle_exact(a, b, c, d), "o={o} {d:?}");
                    assert_eq!(incircle(b, c, a, d), incircle_exact(b, c, a, d), "o={o} {d:?}");
                }
            }
        }
        // incircle e orient3d com pontos "no" círculo e "no" plano por construção em ponto
        // flutuante (o arredondamento os tira de lá por poucos ulp), a distâncias grandes da
        // origem: aqui o determinante ingênuo erra o sinal com frequência.
        let mut s = 99u64;
        let mut u = || {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (s >> 11) as f64 / (1u64 << 53) as f64
        };
        let (mut ing2, mut ing3) = (0, 0);
        for k in 0..20_000 {
            let o = [1.0, 1e3, 1e6, 3e7][k % 4];
            // Círculo de centro (o, o) e raio r: quatro pontos por ângulos sorteados.
            // Raio e extensão de ~1 a ~1e3 (termos do permanente longe de 1).
            let ext = [1.0, 1e3][k / 4 % 2];
            let r = ext * (0.5 + u());
            let ponto = |t: f64| [o + r * t.cos(), o + r * t.sin()];
            let (a, b, c, d) = (
                ponto(std::f64::consts::TAU * u()),
                ponto(std::f64::consts::TAU * u()),
                ponto(std::f64::consts::TAU * u()),
                ponto(std::f64::consts::TAU * u()),
            );
            let ex = incircle_exact(a, b, c, d);
            assert_eq!(incircle(a, b, c, d), ex, "{a:?} {b:?} {c:?} {d:?}");
            let (adx, ady, bdx, bdy, cdx, cdy) = (
                a[0] - d[0],
                a[1] - d[1],
                b[0] - d[0],
                b[1] - d[1],
                c[0] - d[0],
                c[1] - d[1],
            );
            let ing = ((adx * adx + ady * ady) * (bdx * cdy - cdx * bdy)
                + (bdx * bdx + bdy * bdy) * (cdx * ady - adx * cdy)
                + (cdx * cdx + cdy * cdy) * (adx * bdy - bdx * ady))
                .signum();
            ing2 += (ing != ex) as usize;
            // Plano por a, b, c; d = a + σ(b − a) + τ(c − a).
            let mut p3 = |_: ()| [o + 3.0 * ext * u(), o + 3.0 * ext * u(), o + 3.0 * ext * u()];
            let (a, b, c) = (p3(()), p3(()), p3(()));
            let (sg, ta) = (u(), u());
            let d = [0, 1, 2].map(|i| a[i] + sg * (b[i] - a[i]) + ta * (c[i] - a[i]));
            let ex = orient3d_exact(a, b, c, d);
            assert_eq!(orient3d(a, b, c, d), ex, "{a:?} {b:?} {c:?} {d:?}");
            let m = [0, 1, 2].map(|i| [a[i] - d[i], b[i] - d[i], c[i] - d[i]]);
            let ing = (m[2][0] * (m[0][1] * m[1][2] - m[0][2] * m[1][1])
                + m[2][1] * (m[0][2] * m[1][0] - m[0][0] * m[1][2])
                + m[2][2] * (m[0][0] * m[1][1] - m[0][1] * m[1][0]))
                .signum();
            ing3 += (ing != ex) as usize;
        }
        // Medido: 1064 e 253 (o perigo existe; o filtro tem de cobri-lo).
        assert!(ing2 > 300 && ing3 > 50, "{ing2} {ing3}");
        // orient3d: d a poucos ulp do plano x + y + z = 1 por a, b, c, deslocado de (o, o, o).
        for o in [0.0, 1e3, 1e8] {
            let t = |p: [f64; 3]| [p[0] + o, p[1] + o, p[2] + o];
            let (a, b, c) = (t([1.0, 0.0, 0.0]), t([0.0, 1.0, 0.0]), t([0.0, 0.0, 1.0]));
            let terco = 1.0 / 3.0;
            for i in -16..16 {
                for j in -16..16 {
                    let d = t([ulp(terco, i), ulp(terco, j), 1.0 - terco - terco]);
                    assert_eq!(orient3d(a, b, c, d), orient3d_exact(a, b, c, d), "o={o} {d:?}");
                    assert_eq!(orient3d(c, a, b, d), orient3d_exact(c, a, b, d), "o={o} {d:?}");
                }
            }
        }
    }

    /// Produto de expansões exato: componentes inteiros com muitos bits, conferidos em i128.
    #[test]
    fn produto_de_expansoes_exato() {
        let casos: [(&[f64], &[f64]); 4] = [
            (&[3.0, 1152921504606846976.0], &[1.0, 36028797018963968.0]),
            (&[-7.0, 4503599627370497.0], &[4503599627370497.0]),
            (&[1.0, 9007199254740992.0], &[-3.0, 9007199254740992.0]),
            (&[12345.0], &[-67891.0, 2251799813685248.0]),
        ];
        let soma = |e: &[f64]| e.iter().map(|&v| v as i128).sum::<i128>();
        for (e, f) in casos {
            let p = mul(e, f);
            assert_eq!(soma(&p), soma(e) * soma(f), "{e:?} × {f:?} = {p:?}");
            // Não sobreposta e crescente em magnitude.
            assert!(p.windows(2).all(|w| w[0].abs() < w[1].abs()), "{p:?}");
        }
    }
}

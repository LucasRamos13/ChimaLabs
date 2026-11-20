//! Padrão rigoroso (CLAUDE.md §7.3): testes de propriedade com entradas aleatórias (proptest,
//! 256 casos por propriedade). Cada propriedade é um invariante matemático da função.

use chimalab_math::linalg::*;
use chimalab_math::poly::*;
use chimalab_math::signal::*;
use chimalab_math::special::*;
use num_complex::Complex64;
use proptest::prelude::*;

fn matriz(n: usize) -> impl Strategy<Value = Matrix> {
    proptest::collection::vec(-1.0f64..1.0, n * n).prop_map(move |v| Matrix::from_col_major(n, n, v).unwrap())
}
fn retangular() -> impl Strategy<Value = Matrix> {
    (1usize..9, 1usize..9).prop_flat_map(|(m, n)| {
        proptest::collection::vec(-1.0f64..1.0, m * n)
            .prop_map(move |v| Matrix::from_col_major(m, n, v).unwrap())
    })
}
/// Matriz quadrada bem condicionada (diagonal dominante).
fn bem_condicionada() -> impl Strategy<Value = Matrix> {
    (1usize..12).prop_flat_map(|n| {
        matriz(n).prop_map(move |mut a| {
            for i in 0..n {
                a.set(i, i, a.get(i, i) + n as f64 + 1.0);
            }
            a
        })
    })
}
fn dif(a: &Matrix, b: &Matrix) -> f64 {
    a.sub(b).unwrap().max_abs()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // ---------------------------------------------------------------- álgebra linear (F040)

    #[test]
    fn solve_tem_residuo_pequeno(a in bem_condicionada()) {
        let n = a.rows();
        let b = Matrix::from_fn(n, 2, |i, j| (i + 3 * j) as f64 - 2.0);
        let s = solve(&a, &b).unwrap();
        prop_assert!(s.warnings.is_empty());
        prop_assert!(dif(&a.matmul(&s.x).unwrap(), &b) < 1e-12 * (n as f64 + 1.0));
    }

    #[test]
    fn lu_qr_svd_reconstroem(a in retangular()) {
        let (m, n) = a.shape();
        let f = lu(&a).unwrap();
        prop_assert!(dif(&f.p_matrix().matmul(&a).unwrap(), &f.l.matmul(&f.u).unwrap()) < 1e-13);
        let q = qr(&a).unwrap();
        prop_assert!(dif(&q.q.matmul(&q.r).unwrap(), &a) < 1e-13);
        prop_assert!(dif(&q.q.transpose().matmul(&q.q).unwrap(), &Matrix::identity(m)) < 1e-13);
        let s = svd(&a, true).unwrap();
        let k = m.min(n);
        let us = Matrix::from_fn(m, k, |i, j| s.u.get(i, j) * s.s[j]);
        prop_assert!(dif(&us.matmul(&s.v.transpose()).unwrap(), &a) < 1e-13);
        prop_assert!(s.s.windows(2).all(|w| w[0] >= w[1]) && s.s.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn determinante_e_multiplicativo(a in matriz(5), b in matriz(5)) {
        let (da, db) = (det(&a).unwrap(), det(&b).unwrap());
        let dab = det(&a.matmul(&b).unwrap()).unwrap();
        prop_assert!((dab - da * db).abs() <= 1e-12 * (1.0 + (da * db).abs()));
    }

    #[test]
    fn inversa_da_inversa(a in bem_condicionada()) {
        let (ai, _) = inv(&a).unwrap();
        let (aii, _) = inv(&ai).unwrap();
        prop_assert!(dif(&aii, &a) < 1e-11 * a.max_abs());
    }

    #[test]
    fn expm_de_a_vezes_expm_de_menos_a(a in matriz(4)) {
        let p = expm(&a).unwrap().matmul(&expm(&a.scale(-1.0)).unwrap()).unwrap();
        prop_assert!(dif(&p, &Matrix::identity(4)) < 1e-12);
    }

    #[test]
    fn autovalores_simetricos(g in matriz(6)) {
        let s = Matrix::from_fn(6, 6, |i, j| g.get(i, j) + g.get(j, i));
        let Eig::Symmetric { values, vectors } = eig(&s).unwrap() else { panic!() };
        let d = Matrix::from_fn(6, 6, |i, j| if i == j { values[i] } else { 0.0 });
        prop_assert!(dif(&s.matmul(&vectors).unwrap(), &vectors.matmul(&d).unwrap()) < 1e-12);
        prop_assert!(values.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn moore_penrose(a in retangular()) {
        let p = pinv(&a, None).unwrap();
        prop_assert!(dif(&a.matmul(&p).unwrap().matmul(&a).unwrap(), &a) < 1e-10 * (1.0 + a.max_abs()));
        let ap = a.matmul(&p).unwrap();
        prop_assert!(dif(&ap, &ap.transpose()) < 1e-10);
    }

    #[test]
    fn schur_reconstroi(a in (1usize..10).prop_flat_map(matriz)) {
        let s = schur(&a).unwrap();
        prop_assert!(dif(&s.u.matmul(&s.t).unwrap().matmul(&s.u.transpose()).unwrap(), &a) < 1e-12);
        let c = schur_complex(&a).unwrap();
        let back = c.u.matmul(&c.t).unwrap().matmul(&c.u.adjoint()).unwrap();
        prop_assert!(back.sub(&ComplexMatrix::from_real(&a)).unwrap().max_abs() < 1e-12);
    }

    #[test]
    fn funm_exp_igual_expm(a in matriz(4)) {
        let RealOrComplex::Real(f) = funm_elementary(&a, Elementary::Exp).unwrap().value else { panic!() };
        let e = expm(&a).unwrap();
        prop_assert!(dif(&f, &e) < 1e-11 * e.max_abs());
    }

    // ---------------------------------------------------------------- sinais (F046)

    #[test]
    fn fft_ida_e_volta_e_parseval(v in proptest::collection::vec((-1.0f64..1.0, -1.0f64..1.0), 1..300)) {
        let x: Vec<Complex64> = v.iter().map(|&(a, b)| Complex64::new(a, b)).collect();
        let n = x.len();
        let (y, _) = fft(&x, &[n], None, 0).unwrap();
        let (z, _) = ifft(&y, &[n], None, 0).unwrap();
        prop_assert!(x.iter().zip(&z).all(|(a, b)| (a - b).norm() < 1e-12));
        let ex: f64 = x.iter().map(|c| c.norm_sqr()).sum();
        let ey: f64 = y.iter().map(|c| c.norm_sqr()).sum::<f64>() / n as f64;
        prop_assert!((ex - ey).abs() <= 1e-12 * (1.0 + ex));
    }

    #[test]
    fn convolucao_comutativa_e_deconvolucao(a in proptest::collection::vec(-1.0f64..1.0, 1..40), b in proptest::collection::vec(-1.0f64..1.0, 1..40)) {
        let ab = conv(&a, &b, ConvShape::Full);
        let ba = conv(&b, &a, ConvShape::Full);
        prop_assert!(ab.iter().zip(&ba).all(|(x, y)| (x - y).abs() < 1e-14));
        // A divisão longa é mal condicionada para divisores com raízes de módulo grande (erro
        // ~ (max|aᵢ|/|a₀|)ᵏ); a propriedade vale para divisores bem condicionados: a₀ = 1 e
        // Σ|aᵢ| ≤ 1/2 (raízes de módulo > 1/2 do recíproco, divisão estável).
        let s: f64 = a[1..].iter().map(|v| v.abs()).sum::<f64>().max(1e-300);
        let a: Vec<f64> = std::iter::once(1.0).chain(a[1..].iter().map(|v| 0.5 * v / s.max(1.0))).collect();
        let ab = conv(&a, &b, ConvShape::Full);
        let (q, r) = deconv(&ab, &a).unwrap();
        prop_assert!(q.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-8));
        prop_assert!(r.iter().all(|v| v.abs() < 1e-8));
    }

    #[test]
    fn filtro_fir_igual_a_convolucao(b in proptest::collection::vec(-1.0f64..1.0, 1..8), x in proptest::collection::vec(-1.0f64..1.0, 1..60)) {
        let y = filter(&b, &[1.0], &x, None).unwrap().y;
        let c = conv(&b, &x, ConvShape::Full);
        prop_assert!(y.iter().zip(&c).all(|(p, q)| (p - q).abs() < 1e-14));
    }

    #[test]
    fn fftshift_e_inverso(v in proptest::collection::vec(0i32..100, 0..50)) {
        let n = v.len();
        let s = fftshift(&v, &[n], None).unwrap();
        prop_assert_eq!(ifftshift(&s, &[n], None).unwrap(), v);
    }

    // ---------------------------------------------------------------- polinômios (F045)

    #[test]
    fn polyint_e_polyder_sao_inversas(p in proptest::collection::vec(-5.0f64..5.0, 1..10), k in -3.0f64..3.0) {
        let d = polyder(&polyint(&p, k));
        prop_assert!(d.len() == p.len() && d.iter().zip(&p).all(|(a, b)| (a - b).abs() < 1e-12));
    }

    #[test]
    fn raizes_anulam_o_polinomio(r in proptest::collection::vec(-3.0f64..3.0, 1..7)) {
        let rr: Vec<Complex64> = r.iter().map(|&x| Complex64::new(x, 0.0)).collect();
        let (_, p) = poly(&rr);
        let p = p.unwrap();
        let found = roots(&p).unwrap();
        let vals = polyval_complex(&p, &found);
        let scale: f64 = p.iter().map(|c| c.abs()).sum();
        prop_assert!(vals.iter().all(|v| v.norm() < 1e-8 * scale));
    }

    #[test]
    fn polyfit_recupera_polinomio(c in proptest::collection::vec(-2.0f64..2.0, 1..5)) {
        let x: Vec<f64> = (0..20).map(|i| i as f64 / 10.0 - 1.0).collect();
        let y = polyval(&c, &x);
        let f = polyfit(&x, &y, c.len() - 1).unwrap();
        prop_assert!(f.p.iter().zip(&c).all(|(a, b)| (a - b).abs() < 1e-9));
    }

    // ---------------------------------------------------------------- especiais (F042)

    #[test]
    fn gamma_recorrencia(x in 0.05f64..150.0) {
        prop_assert!((gamma(x + 1.0) / (x * gamma(x)) - 1.0).abs() < 1e-13);
        prop_assert!((gammaln(x + 1.0) - gammaln(x) - x.ln()).abs() < 1e-12 * (1.0 + gammaln(x + 1.0).abs()));
    }

    #[test]
    fn erro_e_inversas(y in -0.999_999f64..0.999_999, x in -6.0f64..6.0) {
        prop_assert!((erf(erfinv(y)) - y).abs() <= 4.0 * f64::EPSILON);
        prop_assert!((erf(x) + erfc(x) - 1.0).abs() <= 4.0 * f64::EPSILON);
        prop_assert!((erfc(-x) - (2.0 - erfc(x))).abs() <= 4.0 * f64::EPSILON);
    }

    #[test]
    fn incompletas_complementares(a in 0.1f64..60.0, x in 0.0f64..80.0, b in 0.1f64..60.0, t in 0.0f64..1.0) {
        prop_assert!((gammainc(x, a, false) + gammainc(x, a, true) - 1.0).abs() < 1e-13);
        prop_assert!((betainc(t, a, b, false) + betainc(t, a, b, true) - 1.0).abs() < 1e-13);
        prop_assert!((betainc(t, a, b, false) - betainc(1.0 - t, b, a, true)).abs() < 1e-12);
    }

    #[test]
    fn bessel_recorrencia(nu in 0.0f64..15.0, x in 0.05f64..60.0) {
        let lhs = besselj(nu, x) + besselj(nu + 2.0, x);
        let rhs = 2.0 * (nu + 1.0) / x * besselj(nu + 1.0, x);
        prop_assert!((lhs - rhs).abs() < 1e-12 * (1.0 + rhs.abs()));
        let w = besseli(nu, x, true) * besselk(nu + 1.0, x, true) + besseli(nu + 1.0, x, true) * besselk(nu, x, true);
        prop_assert!((w * x - 1.0).abs() < 1e-12);
    }

    #[test]
    fn jacobi_identidades(u in -20.0f64..20.0, m in 0.0f64..1.0) {
        let (sn, cn, dn) = ellipj(u, m);
        prop_assert!((sn * sn + cn * cn - 1.0).abs() < 1e-14);
        prop_assert!((dn * dn + m * sn * sn - 1.0).abs() < 1e-14);
    }

    #[test]
    fn graus_e_restos(x in -1e6f64..1e6, y in 0.001f64..1e3) {
        let s = sind(x);
        let c = cosd(x);
        prop_assert!((s * s + c * c - 1.0).abs() < 4.0 * f64::EPSILON);
        let r = mod_floor(x, y);
        prop_assert!((0.0..y).contains(&r));
        prop_assert!((rem_trunc(x, y).abs()) < y);
    }

    #[test]
    fn inteiros(a in 1u64..1_000_000_000_000, b in 1u64..1_000_000) {
        let f = factor(a);
        prop_assert_eq!(f.iter().product::<u64>(), a);
        prop_assert!(f.iter().all(|&p| p == 1 || isprime(p)));
        let g = gcd(a as f64, b as f64).unwrap();
        prop_assert!((a as f64) % g == 0.0 && (b as f64) % g == 0.0);
        let bz = gcd_bezout(a as f64, b as f64).unwrap();
        // Conferência em inteiros de 128 bits: em f64, u·a + v·b passa de 2⁵³ e arredonda.
        prop_assert_eq!(bz.u as i128 * a as i128 + bz.v as i128 * b as i128, g as i128);
    }

    #[test]
    fn isprime_concorda_com_divisao_por_tentativa(n in 0u64..200_000) {
        let trial = n >= 2 && (2..).take_while(|d| d * d <= n).all(|d| n % d != 0);
        prop_assert_eq!(isprime(n), trial);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// H⁽²⁾ = conj(H⁽¹⁾) para ν e x reais; a escala só muda a fase (|H·e^{∓ix}| = |H|);
    /// Wronskiano cruzado H⁽¹⁾_{ν+1}H⁽²⁾_ν − H⁽¹⁾_ν H⁽²⁾_{ν+1} = −4i/(πx) (DLMF 10.5.5).
    #[test]
    fn hankel_conjugacao_escala_e_wronskiano(nu in -3.0f64..3.0, x in 0.5f64..100.0) {
        use chimalab_math::special::{HankelKind::*, besselh};
        for s in [false, true] {
            let h1 = besselh(nu, First, x, s);
            let h2 = besselh(nu, Second, x, s);
            prop_assert_eq!(h2, h1.conj());
        }
        let h = besselh(nu, First, x, false);
        let hs = besselh(nu, First, x, true);
        prop_assert!((h.norm() - hs.norm()).abs() <= 1e-14 * h.norm());
        let (a1, a2) = (besselh(nu, First, x, false), besselh(nu, Second, x, false));
        let (b1, b2) = (besselh(nu + 1.0, First, x, false), besselh(nu + 1.0, Second, x, false));
        let w = b1 * a2 - a1 * b2;
        let esperado = -4.0 / (std::f64::consts::PI * x);
        let escala = 2.0 * a1.norm() * b1.norm();
        prop_assert!(w.re.abs() <= 1e-13 * escala, "{w}");
        prop_assert!((w.im - esperado).abs() <= 1e-13 * escala, "{w} vs {esperado}");
    }
}

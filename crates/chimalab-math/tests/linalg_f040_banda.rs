//! F040 — solvers de banda escolhidos por `solve` (RF-040: "escolha automática de algoritmo por
//! estrutura (triangular, simétrica, banda)").
//!
//! Oráculos: inversa fechada da matriz de Laplace 1-D, solução conhecida e resíduo relativo.

use chimalab_math::linalg::*;
use num_complex::Complex64;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

/// Matriz de banda (kl, ku) com elementos pseudoaleatórios em [−1, 1) (sem dominância diagonal).
fn banda(rng: &mut Rng, n: usize, kl: usize, ku: usize) -> Matrix {
    Matrix::from_fn(n, n, |i, j| {
        if i <= j + kl && j <= i + ku {
            rng.next()
        } else {
            0.0
        }
    })
}

/// Matriz de Laplace 1-D: tridiag(−1, 2, −1).
fn laplace(n: usize) -> Matrix {
    Matrix::from_fn(n, n, |i, j| match i.abs_diff(j) {
        0 => 2.0,
        1 => -1.0,
        _ => 0.0,
    })
}

fn residuo_relativo<T: LinScalar>(a: &NdArray<T>, x: &NdArray<T>, b: &NdArray<T>) -> f64 {
    let r = a.matmul(x).unwrap().sub(b).unwrap();
    r.norm1() / (a.norm1() * x.norm1() + b.norm1())
}

#[test]
fn laplace_usa_cholesky_de_banda_e_confere_com_a_inversa_fechada() {
    // (T⁻¹)ᵢⱼ = min(i, j)·(n + 1 − max(i, j))/(n + 1), índices a partir de 1.
    let n = 200;
    let s = solve(&laplace(n), &Matrix::identity(n)).unwrap();
    assert_eq!(s.method, SolveMethod::BandCholesky);
    assert!(s.warnings.is_empty());
    let mut err: f64 = 0.0;
    for i in 1..=n {
        for j in 1..=n {
            let exato = (i.min(j) * (n + 1 - i.max(j))) as f64 / (n + 1) as f64;
            err = err.max((s.x.get(i - 1, j - 1) - exato).abs() / exato);
        }
    }
    // cond₂(T) ≈ 4(n+1)²/π² ≈ 1,6e4: erro relativo ≤ cond·ε com folga.
    assert!(err < 1e-11, "{err:e}");
}

#[test]
fn ca1_residuo_pequeno_com_lu_de_banda() {
    let mut rng = Rng(0xba4d);
    for (n, kl, ku) in [(32, 1, 1), (100, 2, 5), (300, 7, 3), (500, 0, 9), (64, 15, 0)] {
        let a = banda(&mut rng, n, kl, ku);
        let x_true = Matrix::from_fn(n, 3, |_, _| rng.next());
        let b = a.matmul(&x_true).unwrap();
        let s = solve(&a, &b).unwrap();
        // kl = 0 ou ku = 0 é triangular e não chega à banda.
        let esperado = if kl == 0 {
            SolveMethod::UpperTriangular
        } else if ku == 0 {
            SolveMethod::LowerTriangular
        } else {
            SolveMethod::BandLu
        };
        assert_eq!(s.method, esperado, "n={n} kl={kl} ku={ku}");
        let r = residuo_relativo(&a, &s.x, &b);
        assert!(r < 1e-12, "n={n} kl={kl} ku={ku}: {r:e}");
        // Erro direto contra a solução conhecida, limitado pela condição: ‖Δx‖/‖x‖ ≲ c·ε/rcond.
        let rc = rcond(&a).unwrap();
        let erro = x_true.sub(&s.x).unwrap().max_abs() / x_true.max_abs();
        assert!(
            erro < 1e3 * f64::EPSILON / rc,
            "n={n}: erro {erro:e}, rcond {rc:e}"
        );
    }
}

#[test]
fn limiares_da_escolha_por_banda() {
    // n ≥ 32 e kl + ku + 1 ≤ n/4.
    let mut rng = Rng(7);
    let m = |rng: &mut Rng, n, kl, ku| {
        let a = banda(rng, n, kl, ku);
        solve(&a, &Matrix::from_fn(n, 1, |i, _| i as f64)).unwrap().method
    };
    assert_eq!(m(&mut rng, 32, 1, 1), SolveMethod::BandLu);
    assert_eq!(m(&mut rng, 31, 1, 1), SolveMethod::Lu);
    assert_eq!(m(&mut rng, 32, 4, 3), SolveMethod::BandLu); // 8 = 32/4
    assert_eq!(m(&mut rng, 32, 4, 4), SolveMethod::Lu); // 9 > 8
    assert_eq!(m(&mut rng, 40, 3, 6), SolveMethod::BandLu); // 10 = 40/4
    assert_eq!(m(&mut rng, 40, 4, 6), SolveMethod::Lu);
    // Mesma fronteira no caso hermitiano definido positivo.
    assert_eq!(
        solve(&laplace(32), &Matrix::identity(32)).unwrap().method,
        SolveMethod::BandCholesky
    );
    assert_eq!(
        solve(&laplace(31), &Matrix::identity(31)).unwrap().method,
        SolveMethod::Cholesky
    );
}

#[test]
fn hermitiana_indefinida_cai_para_lu_de_banda() {
    // Diagonal positiva, mas indefinida (tridiag(3, 1, 3)): Cholesky falha.
    let n = 50;
    let a = Matrix::from_fn(n, n, |i, j| match i.abs_diff(j) {
        0 => 1.0,
        1 => 3.0,
        _ => 0.0,
    });
    let b = Matrix::from_fn(n, 2, |i, k| (i + k) as f64);
    let s = solve(&a, &b).unwrap();
    assert_eq!(s.method, SolveMethod::BandLu);
    assert!(residuo_relativo(&a, &s.x, &b) < 1e-12);
}

#[test]
fn entradas_complexas() {
    let mut rng = Rng(99);
    let n = 80;
    let a = ComplexMatrix::from_fn(n, n, |i, j| {
        if i.abs_diff(j) <= 2 {
            Complex64::new(rng.next(), rng.next())
        } else {
            Complex64::new(0.0, 0.0)
        }
    });
    let b = ComplexMatrix::from_fn(n, 2, |i, k| Complex64::new(i as f64, k as f64));
    let s = solve(&a, &b).unwrap();
    assert_eq!(s.method, SolveMethod::BandLu);
    assert!(residuo_relativo(&a, &s.x, &b) < 1e-12);
    // Hermitiana definida positiva: Aᴴ·A + I tem banda 4 e usa Cholesky de banda.
    let h = a
        .adjoint()
        .matmul(&a)
        .unwrap()
        .add(&ComplexMatrix::identity(n))
        .unwrap();
    // Simetriza exatamente (o produto em ponto flutuante não é hermitiano bit a bit).
    let h = ComplexMatrix::from_fn(n, n, |i, j| match i.cmp(&j) {
        std::cmp::Ordering::Less => h.get(i, j),
        std::cmp::Ordering::Equal => Complex64::new(h.get(i, i).re, 0.0),
        std::cmp::Ordering::Greater => h.get(j, i).conj(),
    });
    let s = solve(&h, &b).unwrap();
    assert_eq!(s.method, SolveMethod::BandCholesky);
    assert!(residuo_relativo(&h, &s.x, &b) < 1e-12);
}

#[test]
fn ca3_banda_singular_cai_para_lu_densa_com_aviso() {
    let mut rng = Rng(3);
    let mut a = banda(&mut rng, 64, 2, 2);
    for i in 0..64 {
        a.set(i, 10, 0.0);
    }
    let s = solve(&a, &Matrix::from_fn(64, 1, |_, _| 1.0)).unwrap();
    assert_eq!(s.method, SolveMethod::Lu);
    assert_eq!(s.warnings, vec![LinalgWarning::Singular]);
}

#[test]
fn ca3_aviso_de_mal_condicionamento_na_banda() {
    // Bloco [1 2; ½ 1+ε] (det = ε) no canto de uma tridiagonal bem condicionada: rcond ~ ε/10.
    let n = 64;
    let mut a = Matrix::from_fn(n, n, |i, j| match i.abs_diff(j) {
        0 => 4.0,
        1 => 1.0,
        _ => 0.0,
    });
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 0.5);
    a.set(1, 1, 1.0 + f64::EPSILON);
    a.set(1, 2, 0.0);
    a.set(2, 1, 0.0);
    let s = solve(&a, &Matrix::from_fn(n, 1, |_, _| 1.0)).unwrap();
    assert_eq!(s.method, SolveMethod::BandLu);
    let [LinalgWarning::NearlySingular { rcond: rc }] = s.warnings[..] else {
        panic!("{:?}", s.warnings)
    };
    // rcond exato = 1/(‖A‖₁‖A⁻¹‖₁), com A⁻¹ do bloco = [1+ε −2; −½ 1]/ε.
    let exato = 1.0 / (a.norm1() * ((3.0 + f64::EPSILON) / f64::EPSILON).max(1.0));
    assert!(rc <= exato * 1.0001 && rc >= exato / 3.0, "{rc:e} vs {exato:e}");

    // Versão hermitiana (Cholesky de banda): [1 1; 1 1+ε].
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    let s = solve(&a, &Matrix::from_fn(n, 1, |_, _| 1.0)).unwrap();
    assert_eq!(s.method, SolveMethod::BandCholesky);
    assert!(matches!(s.warnings[..], [LinalgWarning::NearlySingular { .. }]));
}

#[test]
fn bandwidth_publica() {
    assert_eq!(bandwidth(&laplace(5)), (1, 1));
    let mut rng = Rng(5);
    assert_eq!(bandwidth(&banda(&mut rng, 30, 4, 7)), (4, 7));
    let c = ComplexMatrix::from_fn(3, 3, |i, j| Complex64::new(0.0, (i == j + 2) as u8 as f64));
    assert_eq!(bandwidth(&c), (2, 0));
}

#[test]
fn regressao_banda_de_matriz_larga() {
    // 2×5 com um só não nulo na diagonal: a varredura acima da diagonal da coluna 4 passava das
    // linhas existentes e entrava em pânico (índice (2, 4) fora de [2, 5]).
    let mut w = Matrix::zeros(2, 5);
    w.set(1, 1, 3.0);
    assert_eq!(bandwidth(&w), (0, 0));
    w.set(0, 4, 1.0);
    assert_eq!(bandwidth(&w), (0, 4));
    w.set(1, 0, 1.0);
    assert_eq!(bandwidth(&w), (1, 4));
}

/// Desempenho (RF-040: banda). Registrado em docs/progresso/F040.md.
#[test]
#[ignore = "medição de desempenho; rodar com --release --ignored"]
fn desempenho_banda_vs_densa() {
    use std::time::Instant;
    let mut rng = Rng(11);
    let n = 2000;
    let a = banda(&mut rng, n, 3, 3);
    let b = Matrix::from_fn(n, 1, |_, _| rng.next());
    let t = Instant::now();
    let s = solve(&a, &b).unwrap();
    let banda_ms = t.elapsed().as_secs_f64() * 1e3;
    assert_eq!(s.method, SolveMethod::BandLu);
    let mut cheia = a.clone();
    cheia.set(n - 1, 0, 1e-300); // força a LU densa
    let t = Instant::now();
    let d = solve(&cheia, &b).unwrap();
    let densa_ms = t.elapsed().as_secs_f64() * 1e3;
    assert_eq!(d.method, SolveMethod::Lu);
    println!("n = {n}, kl = ku = 3: banda {banda_ms:.2} ms; densa {densa_ms:.2} ms");
}

mod propriedades {
    use super::*;
    use proptest::prelude::*;

    fn banda_aleatoria() -> impl Strategy<Value = (Matrix, Matrix)> {
        // n ≥ max(32, 4·(kl + ku + 1)): sempre banda estreita.
        (1usize..6, 1usize..6, 0usize..80, any::<u64>()).prop_map(|(kl, ku, extra, seed)| {
            let n = 32.max(4 * (kl + ku + 1)) + extra;
            let mut rng = Rng(seed | 1);
            let a = banda(&mut rng, n, kl, ku);
            let b = Matrix::from_fn(n, 2, |_, _| rng.next());
            (a, b)
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// LU de banda com pivotamento parcial é retroestável: resíduo relativo pequeno para
        /// qualquer banda estreita, sem exigir dominância diagonal.
        #[test]
        fn solve_de_banda_tem_residuo_pequeno((a, b) in banda_aleatoria()) {
            let s = solve(&a, &b).unwrap();
            prop_assert_eq!(s.method, SolveMethod::BandLu);
            let r = residuo_relativo(&a, &s.x, &b);
            prop_assert!(r < 1e-12, "{:e}", r);
        }

        /// Hermitiana definida positiva (BᵀB + I, B de banda): Cholesky de banda e mesma solução
        /// que a LU de banda da matriz levemente não simétrica.
        #[test]
        fn cholesky_de_banda_tem_residuo_pequeno(n in 32usize..100, kd in 1usize..4, seed: u64) {
            let mut rng = Rng(seed | 1);
            let b0 = banda(&mut rng, n, 0, kd);
            let mut h = b0.transpose().matmul(&b0).unwrap().add(&Matrix::identity(n)).unwrap();
            for i in 0..n {
                for j in 0..i {
                    h.set(i, j, h.get(j, i));
                }
            }
            let rhs = Matrix::from_fn(n, 1, |_, _| rng.next());
            let s = solve(&h, &rhs).unwrap();
            prop_assert_eq!(s.method, SolveMethod::BandCholesky);
            prop_assert!(residuo_relativo(&h, &s.x, &rhs) < 1e-12);
        }

        /// Balanceamento: B = D⁻¹AD exato, diagonal preservada e autovalores ordenados com vetores
        /// coerentes (A·v = λ·v).
        #[test]
        fn balanceamento_e_ordenacao(n in 1usize..8, seed: u64, expo in proptest::collection::vec(-30i32..30, 8)) {
            let mut rng = Rng(seed | 1);
            let a = Matrix::from_fn(n, n, |i, j| rng.next() * 2f64.powi(expo[i] - expo[j]));
            let bal = balance(&a).unwrap();
            for i in 0..n {
                prop_assert_eq!(bal.b.get(i, i), a.get(i, i));
                for j in 0..n {
                    prop_assert_eq!(bal.b.get(i, j), a.get(i, j) / bal.scale[i] * bal.scale[j]);
                }
            }
            let e = eig_with(&a, EigOptions { balance: true, order: EigOrder::Ascending }).unwrap();
            let (vals, vecs) = match e {
                Eig::General { values, vectors } => (values, vectors),
                Eig::Symmetric { .. } => return Ok(()),
            };
            for w in vals.windows(2) {
                prop_assert!(w[0].re < w[1].re || (w[0].re == w[1].re && w[0].im <= w[1].im));
            }
            let ac = a.to_complex();
            let escala = bal.b.as_slice().iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-300);
            for (j, &lam) in vals.iter().enumerate() {
                // Resíduo medido na escala balanceada: D⁻¹(A v − λ v) = B w − λ w.
                let r = (0..n)
                    .map(|i| {
                        let av: Complex64 = (0..n).map(|k| ac.get(i, k) * vecs.get(k, j)).sum();
                        ((av - lam * vecs.get(i, j)) / bal.scale[i]).norm_sqr()
                    })
                    .sum::<f64>()
                    .sqrt();
                let w = (0..n).map(|i| (vecs.get(i, j) / bal.scale[i]).norm_sqr()).sum::<f64>().sqrt();
                prop_assert!(r <= 1e-12 * escala * w, "j={}: {:e}", j, r / (escala * w));
            }
        }
    }
}

//! F041 — matrizes esparsas e solvers iterativos (incremento 1).
//!
//! Oráculos: Poisson 2-D com solução exata da equação discreta (b = A·x* com x* conhecido),
//! resolução densa (F040) e resíduos verdadeiros recalculados.

// Comparações elemento a elemento com matrizes densas ficam mais legíveis com índices.
#![allow(clippy::needless_range_loop)]

use chimalab_math::linalg::{Matrix, solve};
use chimalab_math::random::{Algorithm, Rng};
use chimalab_math::sparse::{
    DirectMethod, EigsOptions, IterFlag, IterOptions, Preconditioner, SparseError, SparseMatrix, Which,
    bicgstab, eigs, gmres, minres, pcg, spsolve,
};
use proptest::prelude::*;

/// Laplaciano 2-D de 5 pontos numa grade k×k (Dirichlet): SPD, n = k².
fn poisson2d(k: usize) -> SparseMatrix {
    let idx = |i: usize, j: usize| i + k * j;
    let mut t = Vec::new();
    for j in 0..k {
        for i in 0..k {
            t.push((idx(i, j), idx(i, j), 4.0));
            if i > 0 {
                t.push((idx(i, j), idx(i - 1, j), -1.0));
            }
            if i + 1 < k {
                t.push((idx(i, j), idx(i + 1, j), -1.0));
            }
            if j > 0 {
                t.push((idx(i, j), idx(i, j - 1), -1.0));
            }
            if j + 1 < k {
                t.push((idx(i, j), idx(i, j + 1), -1.0));
            }
        }
    }
    SparseMatrix::from_triplets(k * k, k * k, &t).unwrap()
}

fn exata(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| ((i as f64) * 0.37).sin() + 0.1 * i as f64 / n as f64)
        .collect()
}

fn erro_rel(x: &[f64], y: &[f64]) -> f64 {
    let num: f64 = x.iter().zip(y).map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt();
    let den: f64 = y.iter().map(|b| b * b).sum::<f64>().sqrt();
    num / den
}

#[test]
fn ca1_pcg_converge_em_poisson_2d() {
    let k = 100;
    let a = poisson2d(k);
    let xs = exata(k * k);
    let b = a.mul_vec(&xs).unwrap();
    // κ(A) ~ 4k²/π² ≈ 4000: CG sem precondicionador precisa de O(k) iterações; IC(0) reduz.
    let mut iters = Vec::new();
    for pc in [Preconditioner::None, Preconditioner::Jacobi, Preconditioner::Ic0] {
        let o = IterOptions {
            tol: 1e-10,
            precond: pc,
            ..Default::default()
        };
        let r = pcg(&a, &b, &o).unwrap();
        assert_eq!(r.flag, IterFlag::Converged, "{pc:?}");
        assert!(r.relres <= 1e-10, "{pc:?}: {}", r.relres);
        // Erro ≤ κ·relres.
        assert!(erro_rel(&r.x, &xs) <= 1e-6, "{pc:?}: {}", erro_rel(&r.x, &xs));
        assert_eq!(r.resvec.len(), r.iterations + 1);
        iters.push(r.iterations);
        println!("PCG {pc:?}: {} iterações, relres {:e}", r.iterations, r.relres);
    }
    // Jacobi não muda nada (diagonal constante); IC(0) reduz as iterações.
    assert!(iters[2] * 2 < iters[0], "{iters:?}");
}

/// A memória é proporcional ao nnz: os vetores do CSC têm exatamente nnz e ncols + 1 elementos.
#[test]
fn ca3_memoria_proporcional_ao_nnz() {
    let a = poisson2d(300); // n = 90 000, nnz = 5n − 4k
    let (n, nnz) = (300 * 300, 5 * 300 * 300 - 4 * 300);
    assert_eq!(a.nnz(), nnz);
    assert_eq!(a.values().len(), nnz);
    assert_eq!(a.row_idx().len(), nnz);
    assert_eq!(a.col_ptr().len(), n + 1);
    // A densa teria n² = 8,1e9 valores; a esparsa guarda 2·nnz + n + 1 ≈ 9,9e5 números.
    assert!(2 * a.nnz() + n + 1 < n * n / 1000);
}

#[test]
fn construcao_e_operacoes() {
    // Repetidos somados, zeros (inclusive resultantes) descartados.
    let a = SparseMatrix::from_triplets(
        3,
        4,
        &[
            (0, 0, 1.0),
            (2, 1, 5.0),
            (0, 0, 2.0),
            (1, 3, 0.0),
            (2, 2, 4.0),
            (2, 2, -4.0),
        ],
    )
    .unwrap();
    assert_eq!(a.nnz(), 2);
    assert_eq!(a.get(0, 0), 3.0);
    assert_eq!(a.get(2, 2), 0.0);
    assert_eq!(a.shape(), (3, 4));
    assert_eq!(a.triplets(), vec![(0, 0, 3.0), (2, 1, 5.0)]);
    // Densa ida e volta, transposta, diagonal, identidade.
    let d = a.to_dense();
    assert_eq!(SparseMatrix::from_dense(&d), a);
    assert_eq!(a.transpose().get(1, 2), 5.0);
    assert_eq!(a.transpose().transpose(), a);
    assert_eq!(a.diagonal(), vec![3.0, 0.0, 0.0]);
    let i = SparseMatrix::identity(4);
    assert_eq!(a.mul(&i).unwrap(), a);
    // Produtos contra os densos.
    let mut r = Rng::new(Algorithm::Philox, 5);
    let aleat = |r: &mut Rng, m: usize, n: usize| {
        let t: Vec<(usize, usize, f64)> = (0..m * n / 3)
            .map(|_| {
                (
                    (r.uniform() * m as f64) as usize,
                    (r.uniform() * n as f64) as usize,
                    r.uniform() - 0.5,
                )
            })
            .collect();
        SparseMatrix::from_triplets(m, n, &t).unwrap()
    };
    let (p, q) = (aleat(&mut r, 7, 5), aleat(&mut r, 5, 6));
    let pq = p.mul(&q).unwrap().to_dense();
    let (pd, qd) = (p.to_dense(), q.to_dense());
    for ii in 0..7 {
        for jj in 0..6 {
            let e: f64 = (0..5).map(|k| pd.get(ii, k) * qd.get(k, jj)).sum();
            assert!((pq.get(ii, jj) - e).abs() <= 1e-15);
        }
    }
    let x: Vec<f64> = (0..5).map(|k| k as f64 - 2.0).collect();
    let y = p.mul_vec(&x).unwrap();
    for ii in 0..7 {
        let e: f64 = (0..5).map(|k| pd.get(ii, k) * x[k]).sum();
        assert!((y[ii] - e).abs() <= 1e-15);
    }
    let z = p.mul_vec_t(&[1.0; 7]).unwrap();
    for jj in 0..5 {
        let e: f64 = (0..7).map(|k| pd.get(k, jj)).sum();
        assert!((z[jj] - e).abs() <= 1e-15);
    }
    let s = p.add_scaled(2.0, &p, -2.0).unwrap();
    assert_eq!(s.nnz(), 0);
    assert!(poisson2d(5).is_symmetric());
    assert!(!p.is_symmetric());
    // Erros.
    assert_eq!(
        SparseMatrix::from_triplets(2, 2, &[(2, 0, 1.0)])
            .unwrap_err()
            .id(),
        "chimalab:sparse:indiceForaDaForma"
    );
    assert_eq!(
        p.mul_vec(&[1.0]).unwrap_err().id(),
        "chimalab:sparse:formaIncompativel"
    );
    assert_eq!(
        p.mul_vec_t(&[1.0]).unwrap_err().id(),
        "chimalab:sparse:formaIncompativel"
    );
    assert_eq!(p.mul(&p).unwrap_err().id(), "chimalab:sparse:formaIncompativel");
    assert_eq!(
        p.add_scaled(1.0, &q, 1.0).unwrap_err().id(),
        "chimalab:sparse:formaIncompativel"
    );
}

#[test]
#[should_panic(expected = "índice fora da forma")]
fn get_fora_da_forma() {
    SparseMatrix::identity(2).get(2, 0);
}

#[test]
fn resolucao_direta_contra_densa() {
    let a = poisson2d(30);
    let xs = exata(900);
    let b = a.mul_vec(&xs).unwrap();
    let (x, m) = spsolve(&a, &b).unwrap();
    assert_eq!(m, DirectMethod::Cholesky);
    assert!(erro_rel(&x, &xs) <= 1e-12);
    // Não simétrica: LU; compara com a densa.
    let mut r = Rng::new(Algorithm::Philox, 9);
    let n = 60;
    let mut t: Vec<(usize, usize, f64)> = (0..n).map(|i| (i, i, 4.0 + r.uniform())).collect();
    for _ in 0..4 * n {
        t.push((
            (r.uniform() * n as f64) as usize,
            (r.uniform() * n as f64) as usize,
            r.uniform() - 0.5,
        ));
    }
    let a = SparseMatrix::from_triplets(n, n, &t).unwrap();
    let b: Vec<f64> = (0..n).map(|i| (i as f64).cos()).collect();
    let (x, m) = spsolve(&a, &b).unwrap();
    assert_eq!(m, DirectMethod::Lu);
    let xd = solve(&a.to_dense(), &Matrix::column(&b)).unwrap().x;
    for i in 0..n {
        assert!((x[i] - xd.get(i, 0)).abs() <= 1e-12 * xd.get(i, 0).abs().max(1.0));
    }
    // Simétrica indefinida com diagonal positiva: Cholesky falha, LU resolve.
    let ind =
        SparseMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 3.0), (1, 0, 3.0), (1, 1, 1.0)]).unwrap();
    let (x, m) = spsolve(&ind, &[4.0, 4.0]).unwrap();
    assert_eq!(m, DirectMethod::Lu);
    assert!((x[0] - 1.0).abs() <= 1e-15 && (x[1] - 1.0).abs() <= 1e-15);
    // Bordas e erros.
    assert_eq!(
        spsolve(&SparseMatrix::identity(0), &[]).unwrap().0,
        Vec::<f64>::new()
    );
    let sing =
        SparseMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 1.0)]).unwrap();
    assert_eq!(
        spsolve(&sing, &[1.0, 2.0]).unwrap_err().id(),
        "chimalab:sparse:singular"
    );
    let ret = SparseMatrix::from_triplets(2, 3, &[]).unwrap();
    assert_eq!(
        spsolve(&ret, &[1.0, 2.0]).unwrap_err().id(),
        "chimalab:sparse:naoQuadrada"
    );
    assert_eq!(
        spsolve(&sing, &[1.0]).unwrap_err().id(),
        "chimalab:sparse:formaIncompativel"
    );
    assert_eq!(
        spsolve(&sing, &[1.0, f64::NAN]).unwrap_err().id(),
        "chimalab:sparse:naoFinito"
    );
}

/// Convecção–difusão 2-D (não simétrica): GMRES e BiCGSTAB, com e sem ILU(0).
#[test]
fn nao_simetricos_gmres_bicgstab() {
    let k = 40;
    let idx = |i: usize, j: usize| i + k * j;
    let mut t = Vec::new();
    for j in 0..k {
        for i in 0..k {
            t.push((idx(i, j), idx(i, j), 4.0));
            if i > 0 {
                t.push((idx(i, j), idx(i - 1, j), -1.4));
            }
            if i + 1 < k {
                t.push((idx(i, j), idx(i + 1, j), -0.6));
            }
            if j > 0 {
                t.push((idx(i, j), idx(i, j - 1), -1.0));
            }
            if j + 1 < k {
                t.push((idx(i, j), idx(i, j + 1), -1.0));
            }
        }
    }
    let a = SparseMatrix::from_triplets(k * k, k * k, &t).unwrap();
    let xs = exata(k * k);
    let b = a.mul_vec(&xs).unwrap();
    for pc in [Preconditioner::None, Preconditioner::Jacobi, Preconditioner::Ilu0] {
        let o = IterOptions {
            tol: 1e-10,
            precond: pc,
            ..Default::default()
        };
        let mut o_g = o.clone();
        o_g.max_iter = Some(2000);
        for (nome, r) in [
            ("gmres", gmres(&a, &b, &o_g).unwrap()),
            ("bicgstab", bicgstab(&a, &b, &o).unwrap()),
        ] {
            assert_eq!(r.flag, IterFlag::Converged, "{nome} {pc:?}: {}", r.relres);
            assert!(r.relres <= 1e-10, "{nome} {pc:?}");
            assert!(erro_rel(&r.x, &xs) <= 1e-7, "{nome} {pc:?}");
            println!("{nome} {pc:?}: {} iterações", r.iterations);
        }
    }
    // GMRES sem reinício (m = n) converge em ≤ n iterações em aritmética exata.
    let o = IterOptions {
        tol: 1e-12,
        restart: Some(400),
        ..Default::default()
    };
    let small = poisson2d(12);
    let bs = small.mul_vec(&exata(144)).unwrap();
    let r = gmres(&small, &bs, &o).unwrap();
    assert_eq!(r.flag, IterFlag::Converged);
    assert!(r.iterations <= 144);
}

/// MINRES em matriz simétrica indefinida (Poisson deslocado), onde o CG colapsa.
#[test]
fn minres_simetrica_indefinida() {
    let k = 30;
    let p = poisson2d(k);
    let shift = SparseMatrix::identity(k * k);
    let a = p.add_scaled(1.0, &shift, -2.5).unwrap(); // autovalores em (−2,5, 5,5)
    let xs = exata(k * k);
    let b = a.mul_vec(&xs).unwrap();
    let r = minres(
        &a,
        &b,
        &IterOptions {
            tol: 1e-10,
            max_iter: Some(5000),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r.flag, IterFlag::Converged, "{}", r.relres);
    assert!(erro_rel(&r.x, &xs) <= 1e-6);
    let cg = pcg(&a, &b, &IterOptions::default()).unwrap();
    assert_ne!(cg.flag, IterFlag::Converged);
    assert_eq!(cg.flag.warning_id(), Some("chimalab:sparse:colapso"));
    // MINRES com precondicionador SPD (Jacobi) em SPD.
    let bp = p.mul_vec(&xs).unwrap();
    let r = minres(
        &p,
        &bp,
        &IterOptions {
            tol: 1e-10,
            precond: Preconditioner::Jacobi,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r.flag, IterFlag::Converged);
    assert!(erro_rel(&r.x, &xs) <= 1e-7);
}

#[test]
fn iterativos_bordas_e_erros() {
    let a = poisson2d(10);
    let n = 100;
    // b = 0: x = 0 imediatamente.
    for f in [pcg, gmres, bicgstab, minres] {
        let r = f(&a, &vec![0.0; n], &IterOptions::default()).unwrap();
        assert_eq!((r.flag, r.iterations, r.relres), (IterFlag::Converged, 0, 0.0));
        assert!(r.x.iter().all(|v| *v == 0.0));
    }
    // Chute inicial exato: 0 iterações.
    let xs = exata(n);
    let b = a.mul_vec(&xs).unwrap();
    for f in [pcg, bicgstab, minres] {
        let r = f(
            &a,
            &b,
            &IterOptions {
                x0: Some(xs.clone()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(r.iterations, 0);
    }
    // Limite de iterações: melhor iterado, flag e aviso.
    for f in [pcg, gmres, bicgstab, minres] {
        let r = f(
            &a,
            &b,
            &IterOptions {
                max_iter: Some(3),
                tol: 1e-14,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(r.flag, IterFlag::MaxIterations);
        assert_eq!(r.flag.warning_id(), Some("chimalab:sparse:naoConvergiu"));
        assert!(r.relres < 1.0);
    }
    assert_eq!(IterFlag::Converged.warning_id(), None);
    assert_eq!(
        IterFlag::NonFinite.warning_id(),
        Some("chimalab:sparse:naoFinito")
    );
    // Erros de entrada.
    let ret = SparseMatrix::from_triplets(2, 3, &[]).unwrap();
    let e = |r: Result<chimalab_math::sparse::IterResult, SparseError>| r.unwrap_err().id();
    assert_eq!(
        e(pcg(&ret, &[1.0, 2.0], &IterOptions::default())),
        "chimalab:sparse:naoQuadrada"
    );
    assert_eq!(
        e(pcg(&a, &[1.0], &IterOptions::default())),
        "chimalab:sparse:formaIncompativel"
    );
    assert_eq!(
        e(pcg(
            &a,
            &b,
            &IterOptions {
                tol: 0.0,
                ..Default::default()
            }
        )),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        e(pcg(
            &a,
            &b,
            &IterOptions {
                x0: Some(vec![0.0; 3]),
                ..Default::default()
            }
        )),
        "chimalab:sparse:formaIncompativel"
    );
    let mut bn = b.clone();
    bn[3] = f64::NAN;
    assert_eq!(
        e(gmres(&a, &bn, &IterOptions::default())),
        "chimalab:sparse:naoFinito"
    );
    // Precondicionadores impossíveis: diagonal nula; IC(0) com pivô negativo.
    let zd = SparseMatrix::from_triplets(2, 2, &[(0, 1, 1.0), (1, 0, 1.0)]).unwrap();
    for pc in [Preconditioner::Jacobi, Preconditioner::Ilu0] {
        assert_eq!(
            e(gmres(
                &zd,
                &[1.0, 1.0],
                &IterOptions {
                    precond: pc,
                    ..Default::default()
                }
            )),
            "chimalab:sparse:singular"
        );
    }
    let neg =
        SparseMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 2.0), (1, 0, 2.0), (1, 1, 1.0)]).unwrap();
    assert_eq!(
        e(pcg(
            &neg,
            &[1.0, 1.0],
            &IterOptions {
                precond: Preconditioner::Ic0,
                ..Default::default()
            }
        )),
        "chimalab:sparse:singular"
    );
    // MINRES com precondicionador não SPD (Jacobi com diagonal negativa): colapso.
    let negd = SparseMatrix::from_triplets(2, 2, &[(0, 0, -1.0), (1, 1, 2.0)]).unwrap();
    let r = minres(
        &negd,
        &[1.0, 1.0],
        &IterOptions {
            precond: Preconditioner::Jacobi,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r.flag, IterFlag::Breakdown);
    for err in [
        SparseError::ShapeMismatch {
            expected: (1, 1),
            actual: (2, 2),
        },
        SparseError::IndexOutOfBounds { row: 1, col: 2 },
        SparseError::NonFinite,
        SparseError::Singular,
        SparseError::NotSquare((1, 2)),
        SparseError::InvalidOption("x"),
    ] {
        assert!(!err.to_string().is_empty());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Matrizes aleatórias diagonalmente dominantes: todos os métodos aplicáveis concordam com a
    /// resolução direta; o resíduo verdadeiro informado confere.
    #[test]
    fn iterativos_contra_direta(n in 2usize..40, semente in 0u64..1000, sim in any::<bool>()) {
        let mut r = Rng::new(Algorithm::Philox, semente);
        let mut t: Vec<(usize, usize, f64)> = Vec::new();
        for _ in 0..3 * n {
            let (i, j, v) = ((r.uniform() * n as f64) as usize, (r.uniform() * n as f64) as usize, r.uniform() - 0.5);
            t.push((i, j, v));
            if sim {
                t.push((j, i, v));
            }
        }
        for i in 0..n {
            t.push((i, i, 4.0 * n as f64));
        }
        let a = SparseMatrix::from_triplets(n, n, &t).unwrap();
        let b: Vec<f64> = (0..n).map(|i| (i as f64 * 1.3).sin()).collect();
        let (xd, _) = spsolve(&a, &b).unwrap();
        let o = IterOptions { tol: 1e-12, precond: Preconditioner::Ilu0, ..Default::default() };
        let mut casos = vec![gmres(&a, &b, &o).unwrap(), bicgstab(&a, &b, &o).unwrap()];
        if sim {
            let oi = IterOptions { tol: 1e-12, precond: Preconditioner::Ic0, ..Default::default() };
            casos.push(pcg(&a, &b, &oi).unwrap());
            casos.push(minres(&a, &b, &IterOptions { tol: 1e-12, ..Default::default() }).unwrap());
        }
        for res in casos {
            prop_assert_eq!(res.flag, IterFlag::Converged);
            prop_assert!(erro_rel(&res.x, &xd) <= 1e-9);
            let ax = a.mul_vec(&res.x).unwrap();
            let rr: f64 = ax.iter().zip(&b).map(|(p, q)| (p - q).powi(2)).sum::<f64>().sqrt();
            let nb: f64 = b.iter().map(|v| v * v).sum::<f64>().sqrt();
            prop_assert!((rr / nb - res.relres).abs() <= 1e-14);
        }
    }
}

/// Casos-limite das recorrências: substituição do resíduo, colapsos e esgotamento do espaço de
/// Krylov. Nenhum deles pode terminar com flag de convergência sem o resíduo verdadeiro.
#[test]
fn recorrencias_casos_limite() {
    let diag = |d: &[f64]| {
        SparseMatrix::from_triplets(
            d.len(),
            d.len(),
            &d.iter().enumerate().map(|(i, v)| (i, i, *v)).collect::<Vec<_>>(),
        )
        .unwrap()
    };
    // κ = 1e12 e tol 1e-15: o resíduo da recorrência continua caindo, mas o verdadeiro estagna em
    // ~ε·κ. A substituição do resíduo impede declarar convergência.
    let d: Vec<f64> = (0..60).map(|k| 10f64.powf(12.0 * k as f64 / 59.0)).collect();
    let a = diag(&d);
    let b: Vec<f64> = (0..60).map(|k| 1.0 + (k as f64).sin()).collect();
    for f in [pcg, bicgstab] {
        let r = f(
            &a,
            &b,
            &IterOptions {
                tol: 1e-15,
                max_iter: Some(400),
                ..Default::default()
            },
        )
        .unwrap();
        assert_ne!(r.flag, IterFlag::Converged);
        assert!(r.relres > 1e-15);
    }
    // Matriz cheia mal condicionada (κ = 1e10, A = Q·D·Q com Q reflexão de Householder): o
    // resíduo verdadeiro estagna em ~ε·κ ≈ 1e-6, enquanto o da recorrência continua caindo.
    // Com tol 1e-9, a substituição do resíduo é acionada e nenhum método declara convergência.
    let n = 20;
    let v: Vec<f64> = (0..n).map(|k| (k as f64 * 0.7).cos() + 0.3).collect();
    let vv: f64 = v.iter().map(|x| x * x).sum();
    let dd: Vec<f64> = (0..n)
        .map(|k| 10f64.powf(10.0 * k as f64 / (n - 1) as f64))
        .collect();
    let q = |i: usize, j: usize| f64::from(u8::from(i == j)) - 2.0 * v[i] * v[j] / vv;
    let mut t = Vec::new();
    for i in 0..n {
        for j in 0..n {
            t.push((i, j, (0..n).map(|k| q(i, k) * dd[k] * q(k, j)).sum::<f64>()));
        }
    }
    let mal = SparseMatrix::from_triplets(n, n, &t).unwrap();
    let bm: Vec<f64> = (0..n).map(|k| (k as f64).sin() + 2.0).collect();
    for f in [pcg, bicgstab, minres] {
        let r = f(
            &mal,
            &bm,
            &IterOptions {
                tol: 1e-9,
                max_iter: Some(300),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            r.flag != IterFlag::Converged || r.relres <= 1e-9,
            "{:?} {}",
            r.flag,
            r.relres
        );
    }
    // BiCGSTAB: r̂·v = 0 (permutação) e ω = 0 seguido de ρ = 0 ([[1, 1], [1, 0]] com b = e₁).
    let perm = SparseMatrix::from_triplets(2, 2, &[(0, 1, 1.0), (1, 0, 1.0)]).unwrap();
    assert_eq!(
        bicgstab(&perm, &[1.0, 0.0], &IterOptions::default())
            .unwrap()
            .flag,
        IterFlag::Breakdown
    );
    let w0 = SparseMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 1.0), (1, 0, 1.0)]).unwrap();
    assert_eq!(
        bicgstab(&w0, &[1.0, 0.0], &IterOptions::default()).unwrap().flag,
        IterFlag::Breakdown
    );
    // GMRES: A·v = 0 (singular) dá colapso "feliz" sem resolver.
    let sing = diag(&[0.0, 1.0]);
    let r = gmres(&sing, &[1.0, 0.0], &IterOptions::default()).unwrap();
    assert_eq!(r.flag, IterFlag::Breakdown);
    // GMRES no limite exato de iterações: converge na última.
    let p = SparseMatrix::from_triplets(3, 3, &[(0, 0, 2.0), (1, 1, 3.0), (2, 2, 5.0), (0, 1, 1.0)]).unwrap();
    let livre = gmres(&p, &[1.0, 2.0, 3.0], &IterOptions::default()).unwrap();
    let justo = gmres(
        &p,
        &[1.0, 2.0, 3.0],
        &IterOptions {
            max_iter: Some(livre.iterations),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(justo.flag, IterFlag::Converged);
    // MINRES em sistema inconsistente: não converge (o resíduo mínimo é 1/√2 do de b).
    let r = minres(&diag(&[1.0, 0.0]), &[1.0, 1.0], &IterOptions::default()).unwrap();
    assert_ne!(r.flag, IterFlag::Converged);
    assert!(r.relres >= 0.7);
    // β = 0 exato com resíduo grande (matriz nula 1×1): colapso.
    let zero = SparseMatrix::from_triplets(1, 1, &[]).unwrap();
    assert_eq!(
        minres(&zero, &[1.0], &IterOptions::default()).unwrap().flag,
        IterFlag::Breakdown
    );
    // MINRES com precondicionador indefinido que só falha no meio.
    let ind =
        SparseMatrix::from_triplets(2, 2, &[(0, 0, 2.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, -3.0)]).unwrap();
    let r = minres(
        &ind,
        &[2.0, 0.5],
        &IterOptions {
            precond: Preconditioner::Jacobi,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        matches!(r.flag, IterFlag::Breakdown | IterFlag::Converged),
        "{:?}",
        r.flag
    );
    // Overflow: a solução não é representável.
    let tiny = diag(&[1e-300]);
    let r = pcg(&tiny, &[1e300], &IterOptions::default()).unwrap();
    assert_eq!(r.flag, IterFlag::NonFinite);
    assert_eq!(
        spsolve(&tiny, &[1e300]).unwrap_err().id(),
        "chimalab:sparse:singular"
    );
}

/// Autovalores do laplaciano 2-D k×k: 4 − 2cos(iπ/(k+1)) − 2cos(jπ/(k+1)), i, j = 1..k.
fn espectro_poisson(k: usize) -> Vec<f64> {
    let c = |i: usize| 2.0 * (i as f64 * std::f64::consts::PI / (k + 1) as f64).cos();
    let mut v: Vec<f64> = (1..=k)
        .flat_map(|i| (1..=k).map(move |j| 4.0 - c(i) - c(j)))
        .collect();
    v.sort_by(f64::total_cmp);
    v
}

#[test]
fn ca2_eigs_retorna_os_k_maiores() {
    let k = 30;
    let a = poisson2d(k);
    let esp = espectro_poisson(k);
    let n = k * k;
    let o = EigsOptions::default();
    // Maiores (em módulo e algébricos coincidem: SPD).
    for w in [Which::LargestMagnitude, Which::LargestAlgebraic] {
        let r = eigs(&a, 6, w, &o).unwrap();
        assert!(r.converged, "{w:?}");
        for (i, l) in r.values.iter().enumerate() {
            assert!(
                (l - esp[n - 1 - i]).abs() <= 1e-9,
                "{w:?} {i}: {l} vs {}",
                esp[n - 1 - i]
            );
        }
        println!("EIGS {w:?}: {} reinícios, {} produtos", r.restarts, r.operations);
    }
    // Menores (mal separados: convergência lenta pelo Lanczos direto, rápida pela inversão).
    let r = eigs(
        &a,
        4,
        Which::SmallestAlgebraic,
        &EigsOptions {
            max_restarts: 2000,
            ..o.clone()
        },
    )
    .unwrap();
    for (i, l) in r.values.iter().enumerate() {
        assert!((l - esp[i]).abs() <= 1e-8, "SA {i}: {l} vs {}", esp[i]);
    }
    let r = eigs(&a, 4, Which::NearestTo(0.0), &o).unwrap();
    assert!(r.converged);
    for (i, l) in r.values.iter().enumerate() {
        assert!((l - esp[i]).abs() <= 1e-10, "σ=0 {i}");
    }
    println!("EIGS σ=0: {} reinícios, {} soluções", r.restarts, r.operations);
    // Interior: os 3 mais próximos de σ = 3,3.
    let mut perto = esp.clone();
    perto.sort_by(|x, y| (x - 3.3).abs().total_cmp(&(y - 3.3).abs()));
    let r = eigs(&a, 3, Which::NearestTo(3.3), &o).unwrap();
    for (i, l) in r.values.iter().enumerate() {
        assert!((l - perto[i]).abs() <= 1e-9, "σ=3,3 {i}: {l} vs {}", perto[i]);
    }
    // Vetores: resíduo pequeno e ortonormais.
    let r = eigs(&a, 5, Which::LargestMagnitude, &o).unwrap();
    for (i, x) in r.vectors.iter().enumerate() {
        assert!(r.residuals[i] <= 1e-8);
        for (j, y) in r.vectors.iter().enumerate() {
            let d: f64 = x.iter().zip(y).map(|(p, q)| p * q).sum();
            assert!((d - f64::from(u8::from(i == j))).abs() <= 1e-8, "{i} {j}: {d}");
        }
    }
}

#[test]
fn eigs_bordas_e_erros() {
    // Indefinida: maiores em módulo pegam os dois lados.
    let d: Vec<(usize, usize, f64)> = (0..50).map(|i| (i, i, i as f64 - 30.0)).collect();
    let a = SparseMatrix::from_triplets(50, 50, &d).unwrap();
    let r = eigs(&a, 3, Which::LargestMagnitude, &EigsOptions::default()).unwrap();
    assert_eq!(r.values, vec![-30.0, -29.0, -28.0]);
    let r = eigs(&a, 2, Which::LargestAlgebraic, &EigsOptions::default()).unwrap();
    assert_eq!(r.values, vec![19.0, 18.0]);
    // Subespaço invariante cedo (matriz com poucos autovalores distintos).
    let i = SparseMatrix::identity(40);
    let r = eigs(&i, 2, Which::LargestMagnitude, &EigsOptions::default()).unwrap();
    assert!(r.values.len() == 2 && r.values.iter().all(|v| (v - 1.0).abs() <= 4.0 * f64::EPSILON));
    let e = |r: Result<chimalab_math::sparse::EigsResult, SparseError>| r.unwrap_err().id();
    let ns = SparseMatrix::from_triplets(3, 3, &[(0, 1, 1.0)]).unwrap();
    assert_eq!(
        e(eigs(&ns, 1, Which::default(), &EigsOptions::default())),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        e(eigs(&i, 0, Which::default(), &EigsOptions::default())),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        e(eigs(&i, 40, Which::default(), &EigsOptions::default())),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        e(eigs(
            &i,
            1,
            Which::default(),
            &EigsOptions {
                tol: 0.0,
                ..Default::default()
            }
        )),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        e(eigs(&i, 1, Which::NearestTo(f64::NAN), &EigsOptions::default())),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        e(eigs(&i, 1, Which::NearestTo(1.0), &EigsOptions::default())),
        "chimalab:sparse:singular"
    );
    let ret = SparseMatrix::from_triplets(2, 3, &[]).unwrap();
    assert_eq!(
        e(eigs(&ret, 1, Which::default(), &EigsOptions::default())),
        "chimalab:sparse:naoQuadrada"
    );
    let inf = SparseMatrix::from_triplets(2, 2, &[(0, 0, f64::INFINITY), (1, 1, 1.0)]).unwrap();
    assert_eq!(
        e(eigs(&inf, 1, Which::default(), &EigsOptions::default())),
        "chimalab:sparse:naoFinito"
    );
}

#[test]
fn svds_contra_svd_densa() {
    use chimalab_math::linalg::singular_values;
    use chimalab_math::sparse::svds;
    let mut r = Rng::new(Algorithm::Philox, 12);
    let (m, n) = (40, 25);
    let t: Vec<(usize, usize, f64)> = (0..200)
        .map(|_| {
            (
                (r.uniform() * m as f64) as usize,
                (r.uniform() * n as f64) as usize,
                r.uniform() - 0.5,
            )
        })
        .collect();
    let a = SparseMatrix::from_triplets(m, n, &t).unwrap();
    let sd = singular_values(&a.to_dense()).unwrap();
    let s = svds(&a, 4, &EigsOptions::default()).unwrap();
    assert!(s.converged);
    for (i, sv) in s.values.iter().enumerate() {
        assert!((sv - sd[i]).abs() <= 1e-9 * sd[0], "{i}: {sv} vs {}", sd[i]);
        // A·v = σ·u e Aᵀ·u = σ·v.
        let av = a.mul_vec(&s.right[i]).unwrap();
        let atu = a.mul_vec_t(&s.left[i]).unwrap();
        let e1: f64 = av
            .iter()
            .zip(&s.left[i])
            .map(|(p, q)| (p - sv * q).powi(2))
            .sum::<f64>()
            .sqrt();
        let e2: f64 = atu
            .iter()
            .zip(&s.right[i])
            .map(|(p, q)| (p - sv * q).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(e1 <= 1e-8 && e2 <= 1e-8, "{i}: {e1} {e2}");
    }
    assert_eq!(
        svds(&a, 0, &EigsOptions::default()).unwrap_err().id(),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        svds(&a, 26, &EigsOptions::default()).unwrap_err().id(),
        "chimalab:sparse:opcaoInvalida"
    );
}

#[test]
fn lsqr_contra_minimos_quadrados_densos() {
    use chimalab_math::linalg::pinv;
    use chimalab_math::sparse::lsqr;
    let mut r = Rng::new(Algorithm::Philox, 21);
    let aleat = |r: &mut Rng, m: usize, n: usize| {
        let mut t: Vec<(usize, usize, f64)> = (0..3 * (m + n))
            .map(|_| {
                (
                    (r.uniform() * m as f64) as usize,
                    (r.uniform() * n as f64) as usize,
                    r.uniform() - 0.5,
                )
            })
            .collect();
        for i in 0..m.min(n) {
            t.push((i, i, 2.0));
        }
        SparseMatrix::from_triplets(m, n, &t).unwrap()
    };
    let o = IterOptions {
        tol: 1e-12,
        ..Default::default()
    };
    // Sobredeterminado (incompatível) e subdeterminado (mínima norma): contra a pseudoinversa.
    for (m, n) in [(60, 25), (20, 45)] {
        let a = aleat(&mut r, m, n);
        let b: Vec<f64> = (0..m).map(|i| (i as f64 * 0.37).sin()).collect();
        let res = lsqr(&a, &b, 0.0, &o).unwrap();
        assert_eq!(res.flag, IterFlag::Converged, "{m}×{n}");
        let p = pinv(&a.to_dense(), None).unwrap();
        let xr: Vec<f64> = (0..n).map(|j| (0..m).map(|i| p.get(j, i) * b[i]).sum()).collect();
        assert!(
            erro_rel(&res.x, &xr) <= 1e-8,
            "{m}×{n}: {}",
            erro_rel(&res.x, &xr)
        );
    }
    // Amortecido: min ‖Ax − b‖² + λ²‖x‖² = mínimos quadrados de [A; λI] x ≈ [b; 0].
    let (m, n) = (40, 30);
    let a = aleat(&mut r, m, n);
    let b: Vec<f64> = (0..m).map(|i| (i as f64 * 0.11).cos()).collect();
    let lam = 0.5;
    let res = lsqr(&a, &b, lam, &o).unwrap();
    let mut t = a.triplets();
    t.extend((0..n).map(|j| (m + j, j, lam)));
    let aug = SparseMatrix::from_triplets(m + n, n, &t).unwrap();
    let mut bb = b.clone();
    bb.extend(vec![0.0; n]);
    let p = pinv(&aug.to_dense(), None).unwrap();
    let xr: Vec<f64> = (0..n)
        .map(|j| (0..m + n).map(|i| p.get(j, i) * bb[i]).sum())
        .collect();
    assert!(erro_rel(&res.x, &xr) <= 1e-8, "{}", erro_rel(&res.x, &xr));
    // Compatível: resíduo ~0; b = 0; b ⟂ imagem; erros.
    let sq = poisson2d(10);
    let xs = exata(100);
    let bs = sq.mul_vec(&xs).unwrap();
    let res = lsqr(&sq, &bs, 0.0, &o).unwrap();
    assert!(res.relres <= 1e-10 && erro_rel(&res.x, &xs) <= 1e-8);
    assert_eq!(lsqr(&sq, &vec![0.0; 100], 0.0, &o).unwrap().iterations, 0);
    let col = SparseMatrix::from_triplets(2, 1, &[(0, 0, 1.0)]).unwrap();
    let r0 = lsqr(&col, &[0.0, 1.0], 0.0, &o).unwrap();
    assert_eq!((r0.iterations, r0.x[0]), (0, 0.0));
    assert_eq!(
        lsqr(&sq, &[1.0], 0.0, &o).unwrap_err().id(),
        "chimalab:sparse:formaIncompativel"
    );
    assert_eq!(
        lsqr(&sq, &bs, -1.0, &o).unwrap_err().id(),
        "chimalab:sparse:opcaoInvalida"
    );
    assert_eq!(
        lsqr(
            &sq,
            &bs,
            0.0,
            &IterOptions {
                x0: Some(vec![0.0; 3]),
                ..Default::default()
            }
        )
        .unwrap_err()
        .id(),
        "chimalab:sparse:formaIncompativel"
    );
    let mut bn = bs.clone();
    bn[0] = f64::NAN;
    assert_eq!(
        lsqr(&sq, &bn, 0.0, &o).unwrap_err().id(),
        "chimalab:sparse:naoFinito"
    );
    let lim = lsqr(
        &sq,
        &bs,
        0.0,
        &IterOptions {
            tol: 1e-14,
            max_iter: Some(2),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(lim.flag, IterFlag::MaxIterations);
}

/// Preenchimento de Cholesky simbólico (eliminação no grafo, oráculo independente): número de
/// arestas novas ao eliminar os nós na ordem p.
fn preenchimento(a: &SparseMatrix, p: &[usize]) -> usize {
    use std::collections::BTreeSet;
    let n = p.len();
    let mut pos = vec![0; n];
    for (k, &v) in p.iter().enumerate() {
        pos[v] = k;
    }
    let mut viz: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
    for (i, j, _) in a.triplets() {
        if i != j {
            viz[pos[i]].insert(pos[j]);
            viz[pos[j]].insert(pos[i]);
        }
    }
    let mut novas = 0;
    for k in 0..n {
        let maiores: Vec<usize> = viz[k].iter().copied().filter(|&v| v > k).collect();
        for (x, &u) in maiores.iter().enumerate() {
            for &w in &maiores[x + 1..] {
                if viz[u].insert(w) {
                    viz[w].insert(u);
                    novas += 1;
                }
            }
        }
    }
    novas
}

#[test]
fn ordenacoes_amd_e_colamd() {
    use chimalab_math::sparse::{amd, colamd};
    let valida = |p: &[usize], n: usize| {
        let mut q = p.to_vec();
        q.sort_unstable();
        q == (0..n).collect::<Vec<_>>()
    };
    // Seta com a linha/coluna densa em 0: na ordem natural, preenche tudo (n−1 escolhe 2); AMD
    // elimina o nó denso por último e não preenche nada.
    let n = 60;
    let mut t: Vec<(usize, usize, f64)> = (0..n).map(|i| (i, i, 4.0)).collect();
    for i in 1..n {
        t.push((0, i, 1.0));
        t.push((i, 0, 1.0));
    }
    let seta = SparseMatrix::from_triplets(n, n, &t).unwrap();
    let natural: Vec<usize> = (0..n).collect();
    assert_eq!(preenchimento(&seta, &natural), (n - 1) * (n - 2) / 2);
    let p = amd(&seta).unwrap();
    assert!(valida(&p, n));
    assert_eq!(preenchimento(&seta, &p), 0);
    // Poisson 2-D 30 × 30: AMD preenche bem menos que a ordem natural (banda).
    let a = poisson2d(30);
    let p = amd(&a).unwrap();
    assert!(valida(&p, 900));
    let (fn_, fa) = (
        preenchimento(&a, &(0..900).collect::<Vec<_>>()),
        preenchimento(&a, &p),
    );
    assert!(fa * 2 < fn_, "AMD {fa} contra natural {fn_}");
    // COLAMD: permutação válida de colunas, inclusive retangular; vazia e erros.
    let r = SparseMatrix::from_triplets(5, 3, &[(0, 0, 1.0), (1, 1, 1.0), (4, 2, 1.0), (2, 0, 1.0)]).unwrap();
    assert!(valida(&colamd(&r), 3));
    assert!(valida(&colamd(&a), 900));
    assert_eq!(
        amd(&SparseMatrix::from_triplets(0, 0, &[]).unwrap()).unwrap(),
        Vec::<usize>::new()
    );
    assert_eq!(amd(&r).unwrap_err().id(), "chimalab:sparse:naoQuadrada");
}

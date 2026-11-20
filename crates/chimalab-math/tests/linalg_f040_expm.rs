//! F040: expm pelo algoritmo de Al-Mohy & Higham (2009). Oráculos: fórmulas fechadas da
//! exponencial de blocos 2×2 triangulares e de Jordan, e semelhanças exatas em f64.

use chimalab_math::linalg::{ComplexMatrix, Matrix, expm};

use num_complex::Complex64;

fn rel(x: f64, exact: f64) -> f64 {
    if exact == 0.0 {
        x.abs()
    } else {
        ((x - exact) / exact).abs()
    }
}

/// Erro relativo máximo por componente.
fn max_rel(e: &Matrix, ex: &Matrix) -> f64 {
    let mut worst: f64 = 0.0;
    for i in 0..e.rows() {
        for j in 0..e.cols() {
            worst = worst.max(rel(e.get(i, j), ex.get(i, j)));
        }
    }
    worst
}

#[test]
fn ca_triangular_superior_nao_normal_com_precisao_por_componente() {
    // exp([[1, b], [0, −1]]) = [[e, b·sinh 1], [0, e⁻¹]]. O algoritmo de 2005 escalonava por
    // ‖A‖₁ = b + 1 e perdia até 7,5e-9 (b = 1e10); medido em 2026-11-18.
    for b in [1e2, 1e4, 1e6, 1e8, 1e10, 1e14] {
        let a = Matrix::from_rows(&[&[1.0, b], &[0.0, -1.0]]).unwrap();
        let ex = Matrix::from_rows(&[&[1f64.exp(), b * 1f64.sinh()], &[0.0, (-1f64).exp()]]).unwrap();
        let err = max_rel(&expm(&a).unwrap(), &ex);
        assert!(err < 2e-16, "b={b}: {err:e}");
    }
}

#[test]
fn ca_triangular_inferior_nao_superescalona() {
    // Sem a correção da diagonal (só se aplica à triangular superior): o ganho vem só da escolha
    // de s pelas normas de potências. Antes: até 7,5e-9; agora ≤ 6,5e-16 (2026-11-18).
    for b in [1e2, 1e4, 1e6, 1e8, 1e10] {
        let a = Matrix::from_rows(&[&[-1.0, 0.0], &[b, 1.0]]).unwrap();
        let ex = Matrix::from_rows(&[&[(-1f64).exp(), 0.0], &[b * 1f64.sinh(), 1f64.exp()]]).unwrap();
        let err = max_rel(&expm(&a).unwrap(), &ex);
        assert!(err < 2e-15, "b={b}: {err:e}");
    }
}

#[test]
fn bloco_de_jordan_com_superdiagonal_grande() {
    // exp(a·I + t·N), N nilpotente 3×3: e^a·[[1, t, t²/2], [0, 1, t], [0, 0, 1]]. Diagonal
    // repetida: superdiagonal pelo limite sinhc(0) = 1.
    for (a, t) in [(0.5, 1e3), (-2.0, 1e6), (3.0, 1e-3)] {
        let m = Matrix::from_rows(&[&[a, t, 0.0], &[0.0, a, t], &[0.0, 0.0, a]]).unwrap();
        let ea = f64::exp(a);
        let ex = Matrix::from_rows(&[
            &[ea, ea * t, ea * t * t / 2.0],
            &[0.0, ea, ea * t],
            &[0.0, 0.0, ea],
        ])
        .unwrap();
        let err = max_rel(&expm(&m).unwrap(), &ex);
        assert!(err < 1e-14, "a={a}, t={t}: {err:e}");
    }
}

#[test]
fn superdiagonal_sem_cancelamento_com_autovalores_proximos() {
    // f₁₂ = (e^λ₂ − e^λ₁)/(λ₂ − λ₁) com λ₂ − λ₁ = h pequeno: e^λ₁·expm1(h)/h, calculado aqui
    // de forma estável.
    for h in [1e-4, 1e-8, 1e-12] {
        let a = Matrix::from_rows(&[&[1.0, 1.0], &[0.0, 1.0 + h]]).unwrap();
        let exact = 1f64.exp() * h.exp_m1() / h;
        let e = expm(&a).unwrap();
        assert!(
            rel(e.get(0, 1), exact) < 4e-16,
            "h={h}: {:e}",
            rel(e.get(0, 1), exact)
        );
        assert!(rel(e.get(1, 1), (1.0 + h).exp()) < 2e-16);
    }
}

#[test]
fn triangular_complexa() {
    // exp([[i, b], [0, −i]]) = [[e^i, b·sin 1], [0, e^{−i}]].
    let i = Complex64::new(0.0, 1.0);
    let b = 1e8;
    let a = ComplexMatrix::from_fn(2, 2, |r, c| match (r, c) {
        (0, 0) => i,
        (0, 1) => Complex64::new(b, 0.0),
        (1, 1) => -i,
        _ => Complex64::new(0.0, 0.0),
    });
    let e = expm(&a).unwrap();
    assert!((e.get(0, 0) - i.exp()).norm() < 2e-16);
    assert!((e.get(1, 1) - (-i).exp()).norm() < 2e-16);
    assert!((e.get(0, 1) - Complex64::new(b * 1f64.sin(), 0.0)).norm() < 2e-16 * b);
    assert_eq!(e.get(1, 0), Complex64::new(0.0, 0.0));
}

#[test]
fn semelhanca_exata_segue_o_condicionamento() {
    // A = S·T·S⁻¹ com S = [[1, 0], [1, 1]] (exata em f64): exp(A) = S·exp(T)·S⁻¹. O problema tem
    // condicionamento ~ b²·u, e o erro observado acompanha esse limite (2,4e-7 com b = 1e4).
    for b in [1e1, 1e2, 1e3, 1e4] {
        let a = Matrix::from_rows(&[&[1.0 - b, b], &[2.0 - b, b - 1.0]]).unwrap();
        let (e1, em1, sh) = (1f64.exp(), (-1f64).exp(), b * 1f64.sinh());
        let ex = Matrix::from_rows(&[&[e1 - sh, sh], &[e1 - sh - em1, sh + em1]]).unwrap();
        let err = expm(&a).unwrap().sub(&ex).unwrap().max_abs() / ex.max_abs();
        assert!(err < 100.0 * b * b * f64::EPSILON, "b={b}: {err:e}");
    }
}

#[test]
fn regressao_norma_que_estoura_termina() {
    // Antes: ‖A‖₁ = ∞ dava s = i32::MAX e ~2·10⁹ quadraturas (travamento). Agora s é finito e o
    // resultado estoura, como deve (exp de autovalor 2e308).
    let a = Matrix::from_rows(&[&[1e308, 1e308], &[1e308, 1e308]]).unwrap();
    let e = expm(&a).unwrap();
    assert!(e.as_slice().iter().all(|x| x.is_nan() || x.is_infinite()));
}

#[test]
fn potencias_que_estouram_sao_recalculadas() {
    // A = t·[[0, −1], [1, 0]] com t = 1e200: A² estoura, e as potências são recalculadas a partir
    // de 2⁻ˢA. exp(A) é uma rotação, mas o problema tem condicionamento ~t: nas ~668 quadraturas
    // o erro de módulo (1 ± ε) é elevado a 2^668 e o resultado tende a 0. Confere-se só que o
    // caminho termina sem NaN e com valores limitados pelo módulo da rotação.
    let t = 1e200;
    let a = Matrix::from_rows(&[&[0.0, -t], &[t, 0.0]]).unwrap();
    let e = expm(&a).unwrap();
    assert!(e.all_finite());
    assert!(e.max_abs() <= 1.0 + 1e-12);
}

#[test]
#[ignore = "medição de desempenho; rodar com --release -- --ignored --nocapture"]
fn desempenho_expm_densa() {
    // Matriz aleatória densa (LCG), ‖A‖ moderada: grau 13 com s pequeno. Medido em 2026-11-18,
    // nesta máquina (mínimo de 9): n = 300 ≈ 10 ms, n = 600 ≈ 53 ms; o algoritmo de 2005 dava
    // ≈ 7 e ≈ 42 ms sob a mesma carga. O custo extra (estimativas de ‖Aᵏ‖ e ℓ) é O(n²) por
    // produto matriz-vetor; o perfil por fase deu ~5–10% do total.
    let mut seed = 7u64;
    for n in [300usize, 600] {
        let a = Matrix::from_fn(n, n, |_, _| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((seed >> 11) as f64 / (1u64 << 53) as f64 - 0.5) * 4.0 / (n as f64).sqrt()
        });
        let best = (0..9)
            .map(|_| {
                let t = std::time::Instant::now();
                std::hint::black_box(expm(&a).unwrap());
                t.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min);
        println!("expm n={n}: {best:.1} ms");
    }
}

//! F040 — balanceamento e opções de `eig` (RF-040: "ordem dos autovalores não garantida salvo
//! opção").
//!
//! Oráculo: matrizes com autovalores exatamente conhecidos, A = D·(P·S·Λ·S⁻¹·P⁻¹)·D⁻¹ com S, P
//! inteiras unimodulares (inversas inteiras exatas), Λ = diag(1, …, n) e D = diag(2^eᵢ): todos os
//! produtos são exatos em ponto flutuante, então os autovalores de A são exatamente 1, …, n.

use chimalab_math::linalg::*;
use num_complex::Complex64;

fn mal_escalada(e: &[i32]) -> Matrix {
    let n = e.len();
    let s = Matrix::from_fn(n, n, |i, j| if i == j || j == i + 1 { 1.0 } else { 0.0 });
    let si = Matrix::from_fn(n, n, |i, j| match j.checked_sub(i) {
        Some(d) if d % 2 == 0 => 1.0,
        Some(_) => -1.0,
        None => 0.0,
    });
    let l = Matrix::from_fn(n, n, |i, j| if i == j { (i + 1) as f64 } else { 0.0 });
    let acopla = |sinal: f64| {
        Matrix::from_fn(n, n, |i, j| {
            if i == j {
                1.0
            } else if (i == n - 1 && j == 0) || (i == n - 2 && j == 1) {
                sinal
            } else {
                0.0
            }
        })
    };
    let m = acopla(1.0)
        .matmul(&s)
        .unwrap()
        .matmul(&l)
        .unwrap()
        .matmul(&si)
        .unwrap()
        .matmul(&acopla(-1.0))
        .unwrap();
    Matrix::from_fn(n, n, |i, j| m.get(i, j) * 2f64.powi(e[i] - e[j]))
}

fn erro_autovalores(e: &Eig<f64>) -> f64 {
    let Eig::General { values, .. } = e else {
        panic!("esperado caso geral")
    };
    let mut v: Vec<Complex64> = values.clone();
    v.sort_by(|a, b| a.re.total_cmp(&b.re));
    v.iter()
        .enumerate()
        .map(|(i, z)| (z - Complex64::new((i + 1) as f64, 0.0)).norm())
        .fold(0.0, f64::max)
}

/// max_j ‖A·vⱼ − λⱼ·vⱼ‖₂ / (‖A‖_F · ‖vⱼ‖₂).
fn residuo_autovetores<T: LinScalar>(a: &NdArray<T>, e: &Eig<T>) -> f64 {
    let a = a.to_complex();
    let (vals, vecs): (Vec<Complex64>, ComplexMatrix) = match e {
        Eig::General { values, vectors } => (values.clone(), vectors.clone()),
        Eig::Symmetric { values, vectors } => (
            values.iter().map(|&x| Complex64::new(x, 0.0)).collect(),
            vectors.to_complex(),
        ),
    };
    let n = a.rows();
    let fro = a.as_slice().iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt();
    (0..n)
        .map(|j| {
            let r = (0..n)
                .map(|i| {
                    ((0..n).map(|k| a.get(i, k) * vecs.get(k, j)).sum::<Complex64>()
                        - vals[j] * vecs.get(i, j))
                    .norm_sqr()
                })
                .sum::<f64>()
                .sqrt();
            let v = (0..n).map(|i| vecs.get(i, j).norm_sqr()).sum::<f64>().sqrt();
            r / (fro * v)
        })
        .fold(0.0, f64::max)
}

const SEM: EigOptions = EigOptions {
    balance: false,
    order: EigOrder::Unspecified,
};

#[test]
fn balanceamento_corrige_matriz_mal_escalada() {
    for e in [
        vec![0, 20, -20, 40, -10, 30],
        vec![0, 40, -40, 60, -30, 50],
        vec![0, -60, 30, 0, 60, -30],
    ] {
        let a = mal_escalada(&e);
        let com = eig(&a).unwrap();
        let sem = eig_with(&a, SEM).unwrap();
        // Medido em 2026-11-17: sem balanceamento 4, 6e3 e 2e16; com, ≤ 1,2e-14.
        assert!(
            erro_autovalores(&com) < 1e-12,
            "{e:?}: {:e}",
            erro_autovalores(&com)
        );
        assert!(
            erro_autovalores(&sem) > 1.0,
            "{e:?}: o caso deixou de ser mal escalado?"
        );
        // Autovetores voltam à escala de A e têm norma 2 unitária.
        assert!(residuo_autovetores(&a, &com) < 1e-13, "{e:?}");
        let Eig::General { vectors, .. } = &com else {
            unreachable!()
        };
        for j in 0..6 {
            let nrm = (0..6).map(|i| vectors.get(i, j).norm_sqr()).sum::<f64>().sqrt();
            assert!((nrm - 1.0).abs() < 1e-14);
        }
    }
}

#[test]
fn balance_propriedades() {
    let a = mal_escalada(&[0, 40, -40, 60, -30, 50]);
    let bal = balance(&a).unwrap();
    // Escalas são potências de 2 e B = D⁻¹·A·D exatamente.
    for &s in &bal.scale {
        assert_eq!(s.log2().fract(), 0.0, "{s}");
    }
    for i in 0..6 {
        for j in 0..6 {
            assert_eq!(bal.b.get(i, j), a.get(i, j) / bal.scale[i] * bal.scale[j]);
        }
    }
    // A norma cai drasticamente (de ~2^90 para O(10)).
    let fro = |m: &Matrix| m.as_slice().iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!(
        fro(&bal.b) < 100.0 && fro(&a) > 1e20,
        "{} {}",
        fro(&bal.b),
        fro(&a)
    );
    // Ponto fixo: balancear de novo não muda nada.
    let de_novo = balance(&bal.b).unwrap();
    assert!(de_novo.scale.iter().all(|&s| s == 1.0), "{:?}", de_novo.scale);
    // Após o balanceamento, cada par (coluna, linha) fora da diagonal tem normas a menos de 4×.
    for i in 0..6 {
        let c: f64 = (0..6)
            .filter(|&k| k != i)
            .map(|k| bal.b.get(k, i).powi(2))
            .sum::<f64>()
            .sqrt();
        let r: f64 = (0..6)
            .filter(|&k| k != i)
            .map(|k| bal.b.get(i, k).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(c / r < 4.0 && r / c < 4.0, "i={i}: c={c} r={r}");
    }
}

#[test]
fn balance_casos_especiais() {
    // Simétrica e hermitiana: já balanceadas.
    let s = Matrix::from_rows(&[&[1.0, 1e6, 0.0], &[1e6, 2.0, 3.0], &[0.0, 3.0, 4.0]]).unwrap();
    assert_eq!(balance(&s).unwrap().scale, vec![1.0; 3]);
    let h = ComplexMatrix::from_fn(2, 2, |i, j| match (i, j) {
        (0, 1) => Complex64::new(1e8, 3.0),
        (1, 0) => Complex64::new(1e8, -3.0),
        _ => Complex64::new(1.0, 0.0),
    });
    assert_eq!(balance(&h).unwrap().scale, vec![1.0; 2]);
    // Triangular (linha ou coluna nula fora da diagonal): nada a fazer nessa posição.
    let t = Matrix::from_rows(&[&[1.0, 1e10], &[0.0, 2.0]]).unwrap();
    assert_eq!(balance(&t).unwrap().scale, vec![1.0, 1.0]);
    // Vazia, escalar.
    assert!(balance(&Matrix::zeros(0, 0)).unwrap().scale.is_empty());
    assert_eq!(
        balance(&Matrix::from_rows(&[&[5.0]]).unwrap()).unwrap().scale,
        vec![1.0]
    );
    // Complexa não hermitiana mal escalada.
    let c = ComplexMatrix::from_fn(2, 2, |i, j| match (i, j) {
        (0, 1) => Complex64::new(0.0, 2f64.powi(40)),
        (1, 0) => Complex64::new(2f64.powi(-40), 0.0),
        _ => Complex64::new(1.0, 1.0),
    });
    let bc = balance(&c).unwrap();
    // |b₀₁| = |b₁₀| exige s₀/s₁ = √(|a₀₁|/|a₁₀|) = 2⁴⁰.
    assert_eq!(bc.scale[0] / bc.scale[1], 2f64.powi(40));
    assert_eq!(bc.b.get(0, 1).norm(), bc.b.get(1, 0).norm());
    // Escalas limitadas: elementos de magnitudes extremas não estouram.
    let x = Matrix::from_rows(&[&[1.0, 1e300], &[1e-300, 1.0]]).unwrap();
    let bx = balance(&x).unwrap();
    assert!(
        bx.scale.iter().all(|s| (1e-151..=1e151).contains(s)),
        "{:?}",
        bx.scale
    );
    assert!(bx.b.all_finite());
}

#[test]
fn balance_erros() {
    assert_eq!(
        balance(&Matrix::zeros(2, 3)).unwrap_err().id(),
        "chimalab:linalg:matrizNaoQuadrada"
    );
    let nan = Matrix::from_rows(&[&[1.0, f64::NAN], &[0.0, 1.0]]).unwrap();
    assert_eq!(
        balance(&nan).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    let inf = Matrix::from_rows(&[&[f64::INFINITY]]).unwrap();
    assert_eq!(
        balance(&inf).unwrap_err().id(),
        "chimalab:linalg:valoresNaoFinitos"
    );
    // eig_with mantém os mesmos erros de eig, com e sem balanceamento.
    for o in [EigOptions::default(), SEM] {
        assert_eq!(
            eig_with(&Matrix::zeros(2, 3), o).unwrap_err().id(),
            "chimalab:linalg:matrizNaoQuadrada"
        );
        assert_eq!(
            eig_with(&nan, o).unwrap_err().id(),
            "chimalab:linalg:valoresNaoFinitos"
        );
    }
}

#[test]
fn ordenacao_dos_autovalores() {
    // Autovalores 3, −1 ± 2i, 0,5 e −4 (blocos 2×2 e 1×1, conjugado por P unimodular).
    let blocos = Matrix::from_rows(&[
        &[3.0, 0.0, 0.0, 0.0, 0.0],
        &[0.0, -1.0, 2.0, 0.0, 0.0],
        &[0.0, -2.0, -1.0, 0.0, 0.0],
        &[0.0, 0.0, 0.0, 0.5, 0.0],
        &[0.0, 0.0, 0.0, 0.0, -4.0],
    ])
    .unwrap();
    let p = Matrix::from_fn(5, 5, |i, j| if i == j || j == i + 1 { 1.0 } else { 0.0 });
    let pi = Matrix::from_fn(5, 5, |i, j| match j.checked_sub(i) {
        Some(d) if d % 2 == 0 => 1.0,
        Some(_) => -1.0,
        None => 0.0,
    });
    let a = p.matmul(&blocos).unwrap().matmul(&pi).unwrap();
    let ordem = |o: EigOrder| -> Vec<(f64, f64)> {
        let e = eig_with(
            &a,
            EigOptions {
                balance: true,
                order: o,
            },
        )
        .unwrap();
        assert!(
            residuo_autovetores(&a, &e) < 1e-13,
            "{o:?}: vetores fora de ordem"
        );
        let Eig::General { values, .. } = e else { panic!() };
        values
            .iter()
            .map(|z| ((z.re * 1e10).round() / 1e10, (z.im * 1e10).round() / 1e10))
            .collect()
    };
    assert_eq!(
        ordem(EigOrder::Ascending),
        vec![(-4.0, 0.0), (-1.0, -2.0), (-1.0, 2.0), (0.5, 0.0), (3.0, 0.0)]
    );
    assert_eq!(
        ordem(EigOrder::Descending),
        vec![(3.0, 0.0), (0.5, 0.0), (-1.0, 2.0), (-1.0, -2.0), (-4.0, 0.0)]
    );
    assert_eq!(
        ordem(EigOrder::DescendingMagnitude),
        vec![(-4.0, 0.0), (3.0, 0.0), (-1.0, -2.0), (-1.0, 2.0), (0.5, 0.0)]
    );
    // Simétrica: crescente por padrão; decrescente sob opção, com os vetores acompanhando.
    let s = Matrix::from_rows(&[&[2.0, 1.0, 0.0], &[1.0, 2.0, 1.0], &[0.0, 1.0, 2.0]]).unwrap();
    for (o, esperado) in [
        (EigOrder::Unspecified, [2.0 - 2f64.sqrt(), 2.0, 2.0 + 2f64.sqrt()]),
        (EigOrder::Ascending, [2.0 - 2f64.sqrt(), 2.0, 2.0 + 2f64.sqrt()]),
        (EigOrder::Descending, [2.0 + 2f64.sqrt(), 2.0, 2.0 - 2f64.sqrt()]),
        (
            EigOrder::DescendingMagnitude,
            [2.0 + 2f64.sqrt(), 2.0, 2.0 - 2f64.sqrt()],
        ),
    ] {
        let e = eig_with(
            &s,
            EigOptions {
                balance: true,
                order: o,
            },
        )
        .unwrap();
        assert!(residuo_autovetores(&s, &e) < 1e-14, "{o:?}");
        let Eig::Symmetric { values, .. } = e else {
            panic!()
        };
        for (v, x) in values.iter().zip(esperado) {
            assert!((v - x).abs() < 1e-14, "{o:?}: {values:?}");
        }
    }
    // Desempate do módulo pela parte real: −2 antes de 2.
    let d = Matrix::from_rows(&[&[2.0, 1.0], &[0.0, -2.0]]).unwrap();
    let Eig::General { values, .. } = eig_with(
        &d,
        EigOptions {
            balance: false,
            order: EigOrder::DescendingMagnitude,
        },
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(values, vec![Complex64::new(-2.0, 0.0), Complex64::new(2.0, 0.0)]);
}

#[test]
fn opcoes_padrao_e_casos_degenerados() {
    assert_eq!(
        EigOptions::default(),
        EigOptions {
            balance: true,
            order: EigOrder::Unspecified
        }
    );
    // Vazia e escalar, com qualquer opção.
    for o in [
        EigOrder::Unspecified,
        EigOrder::Ascending,
        EigOrder::DescendingMagnitude,
    ] {
        let opts = EigOptions {
            balance: true,
            order: o,
        };
        assert!(
            matches!(eig_with(&Matrix::zeros(0, 0), opts).unwrap(), Eig::Symmetric { values, .. } if values.is_empty())
        );
        let Eig::Symmetric { values, .. } = eig_with(&Matrix::from_rows(&[&[-3.0]]).unwrap(), opts).unwrap()
        else {
            panic!()
        };
        assert_eq!(values, vec![-3.0]);
    }
    // Complexa mal escalada: o balanceamento também vale para entrada complexa.
    let i = Complex64::new(0.0, 1.0);
    let k = 2f64.powi(60);
    let c = ComplexMatrix::from_fn(3, 3, |r, s| match (r, s) {
        (0, 0) => Complex64::new(1.0, 0.0),
        (0, 1) => i * k,
        (1, 0) => Complex64::new(1.0 / k, 0.0),
        (1, 1) => Complex64::new(2.0, 0.0),
        (1, 2) => Complex64::new(k, 0.0),
        (2, 1) => Complex64::new(1.0 / k, 0.0),
        (2, 2) => Complex64::new(3.0, 0.0),
        _ => Complex64::new(0.0, 0.0),
    });
    let e = eig(&c).unwrap();
    assert!(residuo_autovetores(&c, &e) < 1e-13);
    // Traço e determinante conferem com a soma e o produto dos autovalores.
    let Eig::General { values, .. } = e else { panic!() };
    let soma: Complex64 = values.iter().sum();
    assert!((soma - Complex64::new(6.0, 0.0)).norm() < 1e-13);
    let prod: Complex64 = values.iter().product();
    let d = chimalab_math::linalg::det(&c).unwrap();
    assert!((prod - d).norm() < 1e-12 * d.norm().max(1.0), "{prod} vs {d}");
}

/// Fatores exatos para A = [1 x; y 1] com |x|/|y| = R. Para i = 0 (c = |y|, r = |x|), o critério
/// escolhe f = 2^k com c·f ∈ [r/(2f), 2r/f), isto é, 2^{2k} ∈ [R/2, 2R), e só aplica se
/// c·f + r/f < 0,95·(c + r). Depois, i = 1 vê a razão 2^{2k}/R ∈ [½, 2) e não muda nada.
#[test]
fn balance_fatores_exatos_2x2() {
    let caso = |x: f64, y: f64| {
        balance(&Matrix::from_rows(&[&[1.0, x], &[y, 1.0]]).unwrap())
            .unwrap()
            .scale
    };
    let p = |e: i32| 2f64.powi(e);
    // R = 2^d: k = ⌈(d − 1)/2⌉ para d ≥ 1; k = ⌈(d + 1)/2⌉ − 1 para d ≤ −1 (salvo a regra de 95%).
    for (d, k) in [
        (81, 40),
        (80, 40),
        (3, 1),
        (2, 1),
        (-2, -1),
        (-3, -2),
        (-80, -40),
        (-81, -41),
    ] {
        assert_eq!(caso(p(d), 1.0), vec![p(k), 1.0], "d = {d}");
        assert_eq!(caso(p(d - 7), p(-7)), vec![p(k), 1.0], "d = {d} (deslocado)");
    }
    // d ∈ {−1, 0, 1}: f = 1 ou a melhoria é < 5% (c = 2r → c/2 + 2r = c + r).
    for d in [-1, 0, 1] {
        assert_eq!(caso(p(d), 1.0), vec![1.0, 1.0], "d = {d}");
    }
    // Regra dos 95%: R = 2,2 → f = 2 daria 2 + 1,1 = 3,1 ≥ 0,95·3,2 = 3,04: não aplica.
    assert_eq!(caso(2.2, 1.0), vec![1.0, 1.0]);
    // R = 2,6 → 2 + 1,3 = 3,3 < 0,95·3,6 = 3,42: aplica.
    assert_eq!(caso(2.6, 1.0), vec![2.0, 1.0]);
    // Sinais e complexos não importam (só módulos).
    assert_eq!(caso(-p(81), -1.0), vec![p(40), 1.0]);
}

// ---------------------------------------------------------------------------------------------
// Etapa de permutação (isolamento de autovalores)

/// Matriz redutível plantada: M = [T₁ X Y; 0 B₂₂ Z; 0 0 T₃], com T₁ (k₁×k₁) e T₃ (k₃×k₃)
/// triangulares superiores de diagonal conhecida e B₂₂ = `mal_escalada` (autovalores 1…m),
/// escondida por uma permutação q: A[q[i]][q[j]] = M[i][j].
fn redutivel(k1: usize, m: usize, k3: usize, q: &[usize]) -> (Matrix, Vec<f64>) {
    let n = k1 + m + k3;
    let b22 = mal_escalada(
        &(0..m as i32)
            .map(|i| if i % 2 == 0 { 20 * i } else { -15 * i })
            .collect::<Vec<_>>(),
    );
    let mut isolados = vec![];
    let mm = Matrix::from_fn(n, n, |i, j| {
        let bloco = |x: usize| {
            if x < k1 {
                0
            } else if x < k1 + m {
                1
            } else {
                2
            }
        };
        match (bloco(i), bloco(j)) {
            (1, 1) => b22.get(i - k1, j - k1),
            (0, 0) | (2, 2) if i == j => 100.0 + i as f64,
            (0, 0) | (2, 2) if i < j => 0.5 + (i * 7 + j) as f64,
            (bi, bj) if bi < bj => 1.0 + ((i + 2 * j) % 5) as f64,
            _ => 0.0,
        }
    });
    for i in (0..k1).chain(k1 + m..n) {
        isolados.push(100.0 + i as f64);
    }
    let a = Matrix::from_fn(n, n, |i, j| {
        let (ii, jj) = (
            q.iter().position(|&x| x == i).unwrap(),
            q.iter().position(|&x| x == j).unwrap(),
        );
        mm.get(ii, jj)
    });
    (a, isolados)
}

#[test]
fn permutacao_isola_autovalores_plantados() {
    let q = [5usize, 2, 8, 0, 7, 3, 1, 6, 4];
    let (a, mut isolados) = redutivel(2, 4, 3, &q);
    let bal = balance(&a).unwrap();
    let n = 9;
    // Bloco ativo do tamanho de B₂₂; fora dele, escala 1 e diagonais = autovalores isolados.
    assert_eq!(bal.ihi - bal.ilo, 4, "ilo = {}, ihi = {}", bal.ilo, bal.ihi);
    let mut diag: Vec<f64> = (0..bal.ilo).chain(bal.ihi..n).map(|i| bal.b.get(i, i)).collect();
    diag.sort_by(f64::total_cmp);
    isolados.sort_by(f64::total_cmp);
    assert_eq!(diag, isolados);
    assert!((0..bal.ilo).chain(bal.ihi..n).all(|i| bal.scale[i] == 1.0));
    // Estrutura: abaixo da diagonal só há não nulos dentro do bloco ativo.
    for i in 0..n {
        for k in 0..i {
            if !(bal.ilo..bal.ihi).contains(&i) || !(bal.ilo..bal.ihi).contains(&k) {
                assert_eq!(bal.b.get(i, k), 0.0, "({i}, {k})");
            }
        }
    }
    // A·T = T·B exatamente (T = P·D, potências de 2).
    let t = bal.transform();
    assert_eq!(a.matmul(&t).unwrap(), t.matmul(&bal.b).unwrap());
    // perm é uma permutação.
    let mut p = bal.perm.clone();
    p.sort_unstable();
    assert_eq!(p, (0..n).collect::<Vec<_>>());
    // Autovalores: os isolados e 1..4; autovetores devolvidos para a base original.
    let e = eig(&a).unwrap();
    let Eig::General { values, .. } = &e else { panic!() };
    let mut v: Vec<f64> = values.iter().map(|z| z.re).collect();
    v.sort_by(f64::total_cmp);
    let mut want = isolados.clone();
    want.extend([1.0, 2.0, 3.0, 4.0]);
    want.sort_by(f64::total_cmp);
    for (x, w) in v.iter().zip(&want) {
        assert!((x - w).abs() <= 1e-12 * w.abs(), "{x} vs {w}");
    }
    assert!(
        residuo_autovetores(&a, &e) < 1e-14,
        "{}",
        residuo_autovetores(&a, &e)
    );
}

#[test]
fn opcoes_de_balanceamento() {
    let q = [3usize, 0, 4, 1, 2, 6, 5];
    let (a, _) = redutivel(1, 4, 2, &q);
    let so_escala = balance_with(
        &a,
        BalanceOptions {
            permute: false,
            scale: true,
        },
    )
    .unwrap();
    assert_eq!((so_escala.ilo, so_escala.ihi), (0, 7));
    assert_eq!(so_escala.perm, (0..7).collect::<Vec<_>>());
    let so_perm = balance_with(
        &a,
        BalanceOptions {
            permute: true,
            scale: false,
        },
    )
    .unwrap();
    assert!(so_perm.scale.iter().all(|&s| s == 1.0));
    assert_eq!(so_perm.ihi - so_perm.ilo, 4);
    let nada = balance_with(
        &a,
        BalanceOptions {
            permute: false,
            scale: false,
        },
    )
    .unwrap();
    assert_eq!(nada.b, a);
    assert_eq!(
        BalanceOptions::default(),
        BalanceOptions {
            permute: true,
            scale: true
        }
    );
    // As três variantes satisfazem A·T = T·B exatamente.
    for bal in [so_escala, so_perm, nada] {
        let t = bal.transform();
        assert_eq!(a.matmul(&t).unwrap(), t.matmul(&bal.b).unwrap());
    }
}

#[test]
fn triangulares_e_diagonais_isolam_tudo() {
    // Triangular superior grande: todas as linhas isoladas, sem trocas (busca de baixo para cima
    // acha a próxima de imediato: O(n²)).
    let n = 400;
    let up = Matrix::from_fn(n, n, |i, j| {
        if i <= j {
            1.0 + ((i * 31 + j * 17) % 13) as f64
        } else {
            0.0
        }
    });
    let bal = balance(&up).unwrap();
    assert_eq!((bal.ilo, bal.ihi), (0, 0));
    assert_eq!(bal.perm, (0..n).collect::<Vec<_>>());
    assert_eq!(bal.b, up);
    // Triangular inferior: vira triangular superior por permutação.
    let low = Matrix::from_fn(6, 6, |i, j| if i >= j { (1 + i + 2 * j) as f64 } else { 0.0 });
    let bal = balance(&low).unwrap();
    assert_eq!(bal.ihi, bal.ilo);
    for i in 0..6 {
        for k in 0..i {
            assert_eq!(bal.b.get(i, k), 0.0);
        }
    }
    let t = bal.transform();
    assert_eq!(low.matmul(&t).unwrap(), t.matmul(&bal.b).unwrap());
    // Uma coluna isolada (só a diagonal na coluna 0) vai para o início.
    let c = Matrix::from_rows(&[&[7.0, 1.0, 2.0], &[0.0, 1.0, 3.0], &[0.0, 4.0, 5.0]]).unwrap();
    let bal = balance(&c).unwrap();
    assert_eq!((bal.ilo, bal.ihi, bal.b.get(0, 0)), (1, 3, 7.0));
    // Complexa: o critério de zero vale para complexos.
    let z = ComplexMatrix::from_fn(3, 3, |i, j| {
        if i == 2 && j < 2 {
            Complex64::new(0.0, 0.0)
        } else {
            Complex64::new(1.0 + i as f64, j as f64)
        }
    });
    let bal = balance(&z).unwrap();
    assert_eq!((bal.ilo, bal.ihi), (0, 2));
}

mod propriedades_permutacao {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        // Matrizes esparsas aleatórias: A·T = T·B exatamente, perm é permutação, escala 1 fora do
        // bloco ativo e B triangular fora dele.
        #[test]
        fn invariantes(n in 1usize..9, sem in any::<u64>()) {
            let mut x = sem;
            let mut prox = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x };
            let a = Matrix::from_fn(n, n, |_, _| {
                let r = prox();
                if r % 3 == 0 { ((r >> 8) % 1000) as f64 * 2f64.powi(((r >> 20) % 60) as i32 - 30) } else { 0.0 }
            });
            let bal = balance(&a).unwrap();
            let t = bal.transform();
            prop_assert_eq!(a.matmul(&t).unwrap(), t.matmul(&bal.b).unwrap());
            let mut p = bal.perm.clone();
            p.sort_unstable();
            prop_assert_eq!(p, (0..n).collect::<Vec<_>>());
            for i in 0..n {
                if !(bal.ilo..bal.ihi).contains(&i) {
                    prop_assert_eq!(bal.scale[i], 1.0);
                }
                for k in 0..i {
                    if !(bal.ilo..bal.ihi).contains(&i) || !(bal.ilo..bal.ihi).contains(&k) {
                        prop_assert_eq!(bal.b.get(i, k), 0.0);
                    }
                }
            }
        }
    }
}

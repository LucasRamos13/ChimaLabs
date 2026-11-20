//! Testes da F045 escritos a partir do teste de mutação (cargo-mutants, CLAUDE.md §7.7).
//! Cada teste cita o(s) mutante(s) que deve detectar. Oráculos: valores calculados à mão
//! (frações parciais, derivadas e estatísticas elementares).

use chimalab_math::linalg::LinalgWarning;
use chimalab_math::poly::*;
use num_complex::Complex64;

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}

fn sorted_re(mut v: Vec<Complex64>) -> Vec<Complex64> {
    v.sort_by(|a, b| a.re.total_cmp(&b.re).then(a.im.total_cmp(&b.im)));
    v
}

fn binom(n: usize, k: usize) -> f64 {
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// Resíduo associado ao polo `pole` e à potência `j` (1 = termo 1/(s − p)).
fn residuo(r: &Residue, pole: Complex64, j: usize) -> Complex64 {
    let start = r
        .poles
        .iter()
        .position(|p| (p - pole).norm() < 1e-4 * pole.norm().max(1.0))
        .unwrap_or_else(|| panic!("polo {pole} ausente em {:?}", r.poles));
    r.residues[start + j - 1]
}

/// Mutante `roots`: `-core[j+1] / core[0]` → `*` (só aparece com polinômio não mônico).
#[test]
fn raizes_de_polinomio_nao_monico() {
    let r = sorted_re(roots(&[2.0, -6.0, 4.0]).unwrap());
    assert!(
        (r[0] - 1.0).norm() < 1e-14 && (r[1] - 2.0).norm() < 1e-14,
        "{r:?}"
    );
    let r = sorted_re(roots(&[-0.5, 0.0, 2.0]).unwrap());
    assert!(
        (r[0] + 2.0).norm() < 1e-14 && (r[1] - 2.0).norm() < 1e-14,
        "{r:?}"
    );
}

/// Mutantes `poly`: tolerância de conjugação `1e-10 * scale` → `+` / `/`.
#[test]
fn poly_tolerancia_de_conjugacao_e_relativa() {
    // Par conjugado com erro absoluto 1e-8 em módulo ~1414: dentro de 1e-10·|z| → real.
    let z = c(1000.0, 1000.0);
    let (_, real) = poly(&[z, c(1000.0 + 1e-8, -1000.0)]);
    assert!(
        real.is_some(),
        "par quase conjugado em escala grande deve dar resultado real"
    );
    // Raízes que não são conjugadas (distância 0,5) não podem ser aceitas.
    let (_, real) = poly(&[c(1.0, 1.0), c(1.0, -1.5)]);
    assert!(real.is_none());
}

/// Mutantes `polyfit_scaled`: média e desvio padrão amostral da centralização.
#[test]
fn polyfit_escalonado_centralizacao_exata() {
    let x = [1.0, 2.0, 3.0, 4.0];
    let y = [1.0, 4.0, 9.0, 16.0];
    let f = polyfit_scaled(&x, &y, 2).unwrap();
    let cen = f.centering.expect("centralização");
    assert_eq!(cen.mean, 2.5);
    assert!((cen.std - (5.0_f64 / 3.0).sqrt()).abs() < 1e-15, "{}", cen.std);
    // Avaliação com x̂ reproduz y.
    let xs: Vec<f64> = x.iter().map(|v| (v - cen.mean) / cen.std).collect();
    for (v, t) in polyval(&f.p, &xs).iter().zip(&y) {
        assert!((v - t).abs() < 1e-12);
    }
    // Um único ponto: desvio 1 (sem divisão por zero).
    let f = polyfit_scaled(&[3.0], &[7.0], 0).unwrap();
    let cen = f.centering.unwrap();
    assert_eq!((cen.mean, cen.std), (3.0, 1.0));
}

/// Mutantes `fit`: condição `m < n + 1` do aviso de posto deficiente.
#[test]
fn polyfit_aviso_so_quando_subdeterminado() {
    // m = n: dois pontos, grau 2 → subdeterminado, precisa de aviso.
    let f = polyfit(&[1.0, 2.0], &[1.0, 4.0], 2).unwrap();
    assert!(
        f.warnings
            .iter()
            .any(|w| matches!(w, LinalgWarning::RankDeficient { .. })),
        "{:?}",
        f.warnings
    );
    // m = n + 1: determinado, sem aviso.
    let f = polyfit(&[1.0, 2.0, 3.0], &[1.0, 4.0, 9.0], 2).unwrap();
    assert!(f.warnings.is_empty(), "{:?}", f.warnings);
}

/// Mutante `sub_aligned` → `vec![1.0]`: derivada do quociente (a'b − ab', b²).
#[test]
fn derivada_do_quociente() {
    // d/dx [x / (x + 1)] = 1 / (x + 1)²
    let (num, den) = polyder_quotient(&[1.0, 0.0], &[1.0, 1.0]);
    assert_eq!(num, vec![1.0]);
    assert_eq!(den, vec![1.0, 2.0, 1.0]);
    // d/dx [x² / (x − 1)] = (x² − 2x) / (x − 1)²; comprimentos diferentes em sub_aligned.
    let (num, den) = polyder_quotient(&[1.0, 0.0, 0.0], &[1.0, -1.0]);
    assert_eq!(num, vec![1.0, -2.0, 0.0]);
    assert_eq!(den, vec![1.0, -2.0, 1.0]);
}

/// Mutantes `residue` no corte do resto: `r[r.len() - (a.len() - 1)..]`.
#[test]
fn residue_impropria_com_denominador_linear_e_mesmo_grau() {
    // s³/(s + 1) = s² − s + 1 − 1/(s + 1)
    let r = residue(&[1.0, 0.0, 0.0, 0.0], &[1.0, 1.0]).unwrap();
    assert_eq!(r.direct, vec![1.0, -1.0, 1.0]);
    assert!((residuo(&r, c(-1.0, 0.0), 1) + 1.0).norm() < 1e-14, "{r:?}");
    // s/(s + 1) = 1 − 1/(s + 1): grau(B) = grau(A).
    let r = residue(&[1.0, 0.0], &[1.0, 1.0]).unwrap();
    assert_eq!(r.direct, vec![1.0]);
    assert!((residuo(&r, c(-1.0, 0.0), 1) + 1.0).norm() < 1e-14, "{r:?}");
}

/// Mutante `residue`: tolerância de agrupamento `1e-5 * |c|` → `/` (polo duplo grande).
#[test]
fn residue_polo_duplo_de_modulo_grande() {
    // 1/(s − 100)² : raízes numéricas separadas por ~1e-6, agrupadas pela tolerância relativa.
    let r = residue(&[1.0], &[1.0, -200.0, 10000.0]).unwrap();
    assert_eq!(r.poles.len(), 2);
    assert!((r.poles[0] - r.poles[1]).norm() == 0.0, "{:?}", r.poles);
    assert!((r.poles[0] - 100.0).norm() < 1e-6);
    assert!(residuo(&r, c(100.0, 0.0), 1).norm() < 1e-6, "{r:?}");
    assert!((residuo(&r, c(100.0, 0.0), 2) - 1.0).norm() < 1e-6, "{r:?}");
}

/// Mutantes `residue` (média do grupo, `*c * m` → `/`) e `series_div` (`d[i] * c` → `/`).
#[test]
fn residue_polo_triplo_e_duplo_com_outro_polo() {
    // Regressão: polos de multiplicidade 3 a 6 não eram agrupados (as raízes numéricas se
    // espalham ~ε^(1/k), acima da antiga tolerância fixa 1e-5) e os resíduos saíam ~1e10.
    // O centro do grupo é a média das raízes; 1/(s + 1)ᵏ tem resíduo 1 só na potência k.
    for k in 3..=6 {
        let a: Vec<f64> = (0..=k).map(|i| binom(k, i)).collect();
        let r = residue(&[1.0], &a).unwrap();
        assert_eq!(r.poles.len(), k);
        assert!(r.poles.iter().all(|&p| p == r.poles[0]), "k = {k}: {:?}", r.poles);
        assert!((r.poles[0] + 1.0).norm() < 1e-12, "k = {k}: {:?}", r.poles);
        for j in 1..=k {
            let esperado = if j == k { 1.0 } else { 0.0 };
            assert!(
                (residuo(&r, c(-1.0, 0.0), j) - esperado).norm() < 1e-9,
                "k = {k}, j = {j}: {r:?}"
            );
        }
    }
    // 1/((s − 1)²(s + 2)) = (1/9)/(s + 2) − (1/9)/(s − 1) + (1/3)/(s − 1)²
    let r = residue(&[1.0], &[1.0, 0.0, -3.0, 2.0]).unwrap();
    let um = c(1.0, 0.0);
    assert!((residuo(&r, c(-2.0, 0.0), 1) - 1.0 / 9.0).norm() < 1e-7, "{r:?}");
    assert!((residuo(&r, um, 1) + 1.0 / 9.0).norm() < 1e-7, "{r:?}");
    assert!((residuo(&r, um, 2) - 1.0 / 3.0).norm() < 1e-7, "{r:?}");
}

/// Mutantes `residue_inverse`: deslocamento do termo de polo repetido (`b.len() - q.len()`),
/// do termo direto (`prod.len() - num.len()`) e tolerância de agrupamento (`1e-12 * |p|`).
#[test]
fn residue_inverse_polo_duplo_termo_direto_e_agrupamento() {
    // 1/(s − 1)²  (grau 2, j = 2)
    let r = Residue {
        residues: vec![c(0.0, 0.0), c(1.0, 0.0)],
        poles: vec![c(1.0, 0.0), c(1.0, 0.0)],
        direct: vec![],
    };
    let (b, a) = residue_inverse(&r);
    assert_eq!(b, vec![1.0]);
    assert_eq!(a, vec![1.0, -2.0, 1.0]);
    // 2 + 1/(s + 1) = (2s + 3)/(s + 1): termo direto de comprimento 1.
    let r = Residue {
        residues: vec![c(1.0, 0.0)],
        poles: vec![c(-1.0, 0.0)],
        direct: vec![2.0],
    };
    let (b, a) = residue_inverse(&r);
    assert_eq!(b, vec![2.0, 3.0]);
    assert_eq!(a, vec![1.0, 1.0]);
    // Polos 100 e 100 + 1e-11 (distância relativa 1e-13) contam como polo duplo:
    // 0/(s − 100) + 1/(s − 100)² → numerador 1.
    let r = Residue {
        residues: vec![c(0.0, 0.0), c(1.0, 0.0)],
        poles: vec![c(100.0, 0.0), c(100.0 + 1e-11, 0.0)],
        direct: vec![],
    };
    let (b, _) = residue_inverse(&r);
    assert_eq!(b.len(), 1, "{b:?}");
    assert!((b[0] - 1.0).abs() < 1e-9, "{b:?}");
}

/// Mutantes `polyval_complex` (`acc * v + c` → `-` / `*`): valores calculados à mão.
#[test]
fn polyval_em_pontos_complexos() {
    // p(z) = 2z² − 3z + 5
    let p = [2.0, -3.0, 5.0];
    let v = polyval_complex(&p, &[c(1.0, 1.0), c(0.0, 0.0), c(-2.0, 0.5)]);
    // z = 1 + i: 2·(2i) − 3 − 3i + 5 = 2 + i
    assert_eq!(v[0], c(2.0, 1.0));
    assert_eq!(v[1], c(5.0, 0.0));
    // z = −2 + 0,5i: z² = 3,75 − 2i → 7,5 − 4i + 6 − 1,5i + 5 = 18,5 − 5,5i
    assert_eq!(v[2], c(18.5, -5.5));
    // Polinômio vazio avalia a zero.
    assert_eq!(polyval_complex(&[], &[c(3.0, 1.0)]), vec![c(0.0, 0.0)]);
}

/// Mutantes `cluster_tol` (`8.0 * ε^(1/k)` → `+` / `/`): dois polos simples distintos a
/// distância relativa 1e-4 não podem ser fundidos num polo duplo.
#[test]
fn residue_polos_proximos_mas_distintos() {
    // 1/((s − 1)(s − 1,0001)) = 1e4/(s − 1,0001) − 1e4/(s − 1)
    let a = [1.0, -2.0001, 1.0001];
    let r = residue(&[1.0], &a).unwrap();
    assert_eq!(r.poles.len(), 2);
    assert!((r.poles[0] - r.poles[1]).norm() > 5e-5, "{:?}", r.poles);
    // Tolerância relativa 1e-7: 2,0001 e 1,0001 não são exatos em binário, e o resíduo
    // 1/(p₁ − p₂) amplifica esse arredondamento por ~1e4.
    assert!((residuo(&r, c(1.0001, 0.0), 1) - 1e4).norm() < 1e-3, "{r:?}");
    assert!((residuo(&r, c(1.0, 0.0), 1) + 1e4).norm() < 1e-3, "{r:?}");
}

fn conta_distintos(poles: &[Complex64]) -> Vec<usize> {
    let mut grupos: Vec<(Complex64, usize)> = vec![];
    for &p in poles {
        match grupos.iter_mut().find(|(q, _)| *q == p) {
            Some((_, m)) => *m += 1,
            None => grupos.push((p, 1)),
        }
    }
    let mut m: Vec<usize> = grupos.into_iter().map(|(_, m)| m).collect();
    m.sort_unstable();
    m
}

/// Mutantes `group_poles` (chave de ordenação `|polo − p|` → `+` / `/`): estruturas de
/// multiplicidade com polos simétricos (±p) e de módulos diferentes. Confere as multiplicidades
/// e a ida e volta `residue_inverse(residue(B, A)) = (B, A)`.
#[test]
fn residue_estrutura_de_multiplicidades_e_ida_e_volta() {
    let casos: [(&[f64], Vec<usize>); 4] = [
        // (s − 1)²(s + 1)
        (&[1.0, 1.0, -1.0], vec![1, 2]),
        // (s + 1)²(s − 1)
        (&[-1.0, -1.0, 1.0], vec![1, 2]),
        // (s − 2)²(s + 2)(s − 0,5)
        (&[2.0, 2.0, -2.0, 0.5], vec![1, 1, 2]),
        // (s − 1)²(s + 1)²(s − 3)
        (&[1.0, 1.0, -1.0, -1.0, 3.0], vec![1, 2, 2]),
    ];
    for (raizes, mult) in casos {
        let rz: Vec<Complex64> = raizes.iter().map(|&x| c(x, 0.0)).collect();
        let a = poly(&rz).1.unwrap();
        let b = [1.0, -0.5];
        let r = residue(&b, &a).unwrap();
        assert_eq!(conta_distintos(&r.poles), mult, "{raizes:?}: {:?}", r.poles);
        let (bb, aa) = residue_inverse(&r);
        assert!(
            aa.iter().zip(&a).all(|(x, y)| (x - y).abs() < 1e-8),
            "{raizes:?}: {aa:?} vs {a:?}"
        );
        assert_eq!(bb.len(), b.len(), "{raizes:?}: {bb:?}");
        assert!(
            bb.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-6),
            "{raizes:?}: {bb:?}"
        );
    }
}

/// Mutantes `residue_inverse` no corte de coeficientes líderes desprezíveis:
/// `num_r.len() > 1` → `>=` e `1e-12 * scale` → `/`.
#[test]
fn residue_inverse_corta_lideres_desprezíveis() {
    // Todos os resíduos nulos: numerador é o polinômio nulo [0], nunca vazio.
    let r = Residue {
        residues: vec![c(0.0, 0.0), c(0.0, 0.0)],
        poles: vec![c(1.0, 0.0), c(2.0, 0.0)],
        direct: vec![],
    };
    assert_eq!(residue_inverse(&r).0, vec![0.0]);
    // Resíduos de soma ~0 por arredondamento: o coeficiente líder (soma dos resíduos) deve
    // ser descartado relativamente à escala (≈ 1e3) do numerador.
    let res = [100.1, 200.2, -300.3];
    let soma: f64 = res.iter().sum();
    assert!(soma != 0.0 && soma.abs() < 1e-11, "{soma:e}");
    let r = Residue {
        residues: res.iter().map(|&x| c(x, 0.0)).collect(),
        poles: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        direct: vec![],
    };
    let (b, _) = residue_inverse(&r);
    assert_eq!(b.len(), 2, "{b:?}");
}

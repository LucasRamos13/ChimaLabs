//! Álgebra linear pela linguagem (F040 no interpretador): cada função contra identidades
//! (reconstruções, ortogonalidade, condições de Moore–Penrose) e valores exatos conhecidos.

use chimalab_array::Complex64;
use chimalab_runtime::{Interpretador, Obj, Value};
use proptest::prelude::*;

fn roda(src: &str) -> Interpretador {
    let mut it = Interpretador::new();
    let r = it.executa(src);
    assert!(r.erro.is_none(), "{src}: {}", r.erro.unwrap());
    it
}

fn valor(it: &Interpretador, nome: &str) -> Value {
    match it.variavel(nome) {
        Some(Obj::Array(v)) => v.clone(),
        o => panic!("{nome}: {o:?}"),
    }
}

fn num(it: &Interpretador, nome: &str) -> Vec<f64> {
    valor(it, nome).to_f64s().unwrap()
}

/// Maior módulo dos elementos de uma variável (para resíduos calculados na linguagem).
fn maior(it: &Interpretador, nome: &str) -> f64 {
    valor(it, nome)
        .to_complex()
        .iter()
        .map(|z| z.norm())
        .fold(0.0, f64::max)
}

fn perto(a: &[f64], b: &[f64], tol: f64) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| (x - y).abs() <= tol * (1.0 + y.abs()))
}

fn erro(src: &str) -> (String, String) {
    let mut it = Interpretador::new();
    let e = it
        .executa(src)
        .erro
        .unwrap_or_else(|| panic!("{src} deveria falhar"));
    (e.id, e.mensagem)
}

#[test]
fn determinante_inversa_e_traco() {
    let it = roda(
        "a = det([1 2; 3 4])\nb = det([2 1 0; 0 3 1; 0 0 4])\nc = det(eye(3))\nd = det([true false; false true])\n\
         e = det([])\nX = inv([4 7; 2 6])\nR = inv([4 7; 2 6]) * [4 7; 2 6] - eye(2)\nt = trace([1 2; 3 4])\n",
    );
    assert!(perto(&num(&it, "a"), &[-2.0], 1e-15));
    assert!(perto(&num(&it, "b"), &[24.0], 1e-15));
    assert_eq!(num(&it, "c"), [1.0]);
    assert_eq!(num(&it, "d"), [1.0]);
    assert_eq!(num(&it, "e"), [1.0], "det da matriz vazia é 1 (produto vazio)");
    assert!(perto(&num(&it, "X"), &[0.6, -0.2, -0.7, 0.4], 1e-15));
    assert!(maior(&it, "R") < 1e-15);
    assert_eq!(num(&it, "t"), [5.0]);
    // Complexo.
    let it = roda("z = det([1i 0; 0 2])\nW = inv([2i 0; 0 4])\ntz = trace([1i 2; 3 1])\n");
    assert_eq!(valor(&it, "z").to_complex(), [Complex64::new(0.0, 2.0)]);
    assert_eq!(valor(&it, "W").to_complex()[0], Complex64::new(0.0, -0.5));
    assert_eq!(valor(&it, "tz").to_complex(), [Complex64::new(1.0, 1.0)]);
    // Inversa de singular: aviso no console, não erro.
    let mut it = Interpretador::new();
    let r = it.executa("Y = inv([1 2; 2 4]);");
    assert!(r.erro.is_none());
    assert!(
        r.texto().contains("chimalab:runtime:matrizSingular"),
        "{}",
        r.texto()
    );
}

#[test]
fn normas_posto_e_condicionamento() {
    let it = roda(
        "a = norm([3 4])\nb = norm([3 4], 1)\nc = norm([3 4], Inf)\nd = norm([1 2; 3 4], 'fro')\n\
         e = norm([1 2; 3 4], 1)\nf = norm([1 2; 3 4], Inf)\ng = norm([1 1 1 1], 3)\nh = norm([3 0; 0 -5])\n\
         r1 = rank([1 2; 2 4])\nr2 = rank(eye(3))\nr3 = rank(zeros(2))\nr4 = rank([1 0; 0 1e-10], 1e-5)\n\
         r5 = rank([1 0; 0 1e-10])\nk = cond([1 0; 0 10])\nn0 = norm([])\nnz = norm([3i 4])\nm2 = norm([3 0; 0 -5], 2)\nv2 = norm([3 4], 2)\n",
    );
    for (v, esperado) in [
        ("a", 5.0),
        ("b", 7.0),
        ("c", 4.0),
        ("d", 30f64.sqrt()),
        ("e", 6.0),
        ("f", 7.0),
        ("g", 4f64.powf(1.0 / 3.0)),
        ("h", 5.0),
        ("r1", 1.0),
        ("r2", 3.0),
        ("r3", 0.0),
        ("r4", 1.0),
        ("r5", 2.0),
        ("k", 10.0),
        ("n0", 0.0),
        ("nz", 5.0),
        ("m2", 5.0),
        ("v2", 5.0),
    ] {
        assert!(
            perto(&num(&it, v), &[esperado], 1e-14),
            "{v}: {:?} ≠ {esperado}",
            num(&it, v)
        );
    }
    for src in [
        "norm([1 2], 'x')",
        "norm([1 2], -1)",
        "norm([1 2], 0)",
        "norm([1 2], [1 2])",
        "norm([1 2], x -> x)",
        "rank([1 2], -1)",
        "pinv([1 2], [1 2])",
    ] {
        assert_eq!(erro(src).0, "chimalab:execucao:argumentos", "{src}");
    }
}

#[test]
fn autovalores_e_valores_singulares() {
    // Simétrica: autovalores reais crescentes, autovetores ortonormais.
    let it = roda("A = [2 1; 1 2]\nl = eig(A)\nV, D = eig(A)\nR = A * V - V * D\nO = V' * V - eye(2)\n");
    assert!(perto(&num(&it, "l"), &[1.0, 3.0], 1e-15));
    assert_eq!(valor(&it, "l").dims(), [2, 1]);
    assert!(maior(&it, "R") < 1e-14 && maior(&it, "O") < 1e-14);
    // Rotação: ±i; reconstrução complexa.
    let it = roda("A = [0 1; -1 0]\nl = eig(A)\nV, D = eig(A)\nR = A * V - V * D\n");
    let mut l = valor(&it, "l").to_complex();
    l.sort_by(|a, b| a.im.total_cmp(&b.im));
    assert!(
        (l[0] - Complex64::new(0.0, -1.0)).norm() < 1e-15 && (l[1] - Complex64::new(0.0, 1.0)).norm() < 1e-15
    );
    assert!(maior(&it, "R") < 1e-14);
    // Não simétrica com autovalores reais: o resultado volta a ser real.
    let it = roda("l = eig([1 2; 0 3])\n");
    assert!(!valor(&it, "l").is_complex());
    let mut l = num(&it, "l");
    l.sort_by(f64::total_cmp);
    assert!(perto(&l, &[1.0, 3.0], 1e-15));
    // SVD: valores em coluna, decrescentes; reconstrução retangular.
    let it = roda(
        "s = svd([3 0; 0 -4])\nA = [1 2; 3 4; 5 6]\nU, S, V = svd(A)\nR = U * S * V' - A\nOu = U' * U - eye(3)\nOv = V' * V - eye(2)\n\
         U2, S2 = svd(A)\nz = svd([1i 0; 0 2])\n",
    );
    assert_eq!(num(&it, "s"), [4.0, 3.0]);
    assert_eq!(valor(&it, "S").dims(), [3, 2]);
    assert!(maior(&it, "R") < 1e-14 && maior(&it, "Ou") < 1e-14 && maior(&it, "Ov") < 1e-14);
    assert_eq!(valor(&it, "S2"), valor(&it, "S"));
    assert!(perto(&num(&it, "z"), &[2.0, 1.0], 1e-15));
}

#[test]
fn fatoracoes() {
    let it = roda(
        "A = [4 2; 2 3]\nR = chol(A)\nE = R' * R - A\n\
         B = [1 2 3; 4 5 6; 7 8 10]\nL, U, P = lu(B)\nF = P * B - L * U\nL2, U2 = lu(B)\nG = L2 * U2 - B\n\
         Q, R2 = qr(B)\nH = Q * R2 - B\nO = Q' * Q - eye(3)\nR1 = qr(B)\n",
    );
    assert!(maior(&it, "E") < 1e-14);
    let r = valor(&it, "R").to_f64s().unwrap();
    assert_eq!(r[1], 0.0, "R triangular superior");
    assert!(maior(&it, "F") < 1e-14 && maior(&it, "G") < 1e-14);
    let l = num(&it, "L");
    assert_eq!((l[0], l[4], l[8]), (1.0, 1.0, 1.0), "diagonal unitária");
    assert_eq!((l[3], l[6], l[7]), (0.0, 0.0, 0.0), "triangular inferior");
    let p = num(&it, "P");
    assert_eq!(p.iter().filter(|&&x| x == 1.0).count(), 3, "permutação");
    assert!(maior(&it, "H") < 1e-14 && maior(&it, "O") < 1e-14);
    assert_eq!(valor(&it, "R1"), valor(&it, "R2"));
    let r2 = num(&it, "R2");
    assert_eq!((r2[1], r2[2], r2[5]), (0.0, 0.0, 0.0));
    // Retangular e complexa.
    let it = roda("C = [1 1i; 2 3; 0 1]\nQ, R = qr(C)\nH = Q * R - C\nL, U, P = lu(C)\nF = P * C - L * U\n");
    assert!(maior(&it, "H") < 1e-14 && maior(&it, "F") < 1e-14);
    assert_eq!(
        erro("R = chol([1 2; 2 1])").0,
        "chimalab:linalg:naoDefinidaPositiva"
    );
    let (id, msg) = erro("x = lu([1 2; 3 4])");
    assert_eq!(id, "chimalab:execucao:saidasDemais");
    assert!(msg.contains("L, U = lu(A)"));
}

#[test]
fn exponencial_pseudoinversa_e_subespacos() {
    let it = roda(
        "E1 = expm([0 1; 0 0])\nE0 = expm(zeros(2))\nE2 = expm([1 0; 0 2])\n\
         A = [1 2; 2 4; 3 6]\nX = pinv(A)\nc1 = A * X * A - A\nc2 = X * A * X - X\nc3 = (A * X)' - A * X\nc4 = (X * A)' - X * A\n\
         N = null([1 1])\nr = [1 1] * N\nnn = N' * N\nT = orth([1 1; 1 1])\ntt = T' * T\nXc = pinv([1i 0])\n",
    );
    assert_eq!(num(&it, "E1"), [1.0, 0.0, 1.0, 1.0]);
    assert_eq!(num(&it, "E0"), [1.0, 0.0, 0.0, 1.0]);
    assert!(perto(&num(&it, "E2"), &[1f64.exp(), 0.0, 0.0, 2f64.exp()], 1e-15));
    for c in ["c1", "c2", "c3", "c4"] {
        assert!(maior(&it, c) < 1e-14, "Moore–Penrose {c}: {}", maior(&it, c));
    }
    assert_eq!(valor(&it, "N").dims(), [2, 1]);
    assert!(maior(&it, "r") < 1e-15);
    assert!(perto(&num(&it, "nn"), &[1.0], 1e-15));
    assert_eq!(valor(&it, "T").dims(), [2, 1]);
    assert!(perto(&num(&it, "tt"), &[1.0], 1e-15));
    assert_eq!(valor(&it, "Xc").to_complex()[0], Complex64::new(0.0, -1.0));
}

#[test]
fn erros_de_algebra() {
    for (src, id) in [
        ("det(ones(2, 3))", "chimalab:linalg:matrizNaoQuadrada"),
        ("inv(ones(2, 3))", "chimalab:linalg:matrizNaoQuadrada"),
        ("trace(ones(2, 3))", "chimalab:linalg:matrizNaoQuadrada"),
        ("expm(ones(2, 3))", "chimalab:linalg:matrizNaoQuadrada"),
        ("eig([1 NaN; 0 1])", "chimalab:linalg:valoresNaoFinitos"),
        ("det(int8(eye(2)))", "chimalab:execucao:classeNaoSuportada"),
        ("det(single(eye(2)))", "chimalab:execucao:classeNaoSuportada"),
        ("det(ones(2, 2, 2))", "chimalab:execucao:argumentos"),
        ("det()", "chimalab:execucao:argumentos"),
        ("det(1, 2)", "chimalab:execucao:argumentos"),
        ("norm(1, 2, 3)", "chimalab:execucao:argumentos"),
        ("det(x -> x)", "chimalab:execucao:funcaoComoValor"),
        ("svd(eye(2); econ=true)", "chimalab:execucao:argumentos"),
    ] {
        assert_eq!(erro(&format!("r = {src}")).0, id, "{src}");
    }
    assert!(erro("r = det(int8(eye(2)))").1.contains("double(…)"));
    assert!(
        erro("r = det(1, 2)")
            .1
            .contains("`det` aceita 1 argumento(s), recebeu 2")
    );
    assert!(
        erro("r = norm(1, 2, 3)")
            .1
            .contains("`norm` aceita 1 ou 2 argumento(s), recebeu 3")
    );
}

proptest! {
    /// Matrizes 3×3 diagonalmente dominantes aleatórias: inv·A = I, det(A)·det(inv(A)) = 1,
    /// soma e produto dos autovalores iguais a traço e determinante, Σσ² = ‖A‖²_F.
    #[test]
    fn identidades_em_matrizes_aleatorias(v in prop::collection::vec(-5i8..6, 9)) {
        let elems: Vec<String> = v.iter().enumerate().map(|(k, &x)| {
            let d = if k % 4 == 0 { 30 } else { 0 };
            (i32::from(x) + d).to_string()
        }).collect();
        let a = format!("[{} {} {}; {} {} {}; {} {} {}]", elems[0], elems[3], elems[6], elems[1], elems[4], elems[7], elems[2], elems[5], elems[8]);
        let src = format!(
            "A = {a}\nI = inv(A) * A - eye(3)\np = det(A) * det(inv(A))\nl = eig(A)\nsl = sum(l) - trace(A)\npl = prod_(l) - det(A)\n\
             s = svd(A)\nsf = sum(s .^ 2) - norm(A, 'fro')^2\ndef prod_(x):\n    r = 1\n    for k in 1:numel(x):\n        r = r * x[k]\n    return r\n"
        );
        let it = roda(&src);
        prop_assert!(maior(&it, "I") < 1e-13);
        prop_assert!((num(&it, "p")[0] - 1.0).abs() < 1e-12);
        let escala = num(&it, "s")[0];
        prop_assert!(maior(&it, "sl") < 1e-12 * escala);
        prop_assert!(maior(&it, "pl") < 1e-12 * escala.powi(3));
        prop_assert!(maior(&it, "sf") < 1e-12 * escala * escala);
    }
}

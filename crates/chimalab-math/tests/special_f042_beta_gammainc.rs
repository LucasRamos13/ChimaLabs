//! F042: beta, betaln e gammainc nos regimes difíceis (argumentos grandes, um argumento pequeno e
//! outro enorme, transição a ≈ x). Oráculo: tests/dados/F042/beta_gammainc.csv (mpmath a 60
//! dígitos, ferramentas/oraculos/f042_beta_gammainc.py; LICENCAS D13).

use chimalab_math::special::*;

fn referencias() -> Vec<(String, f64, f64, f64)> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/beta_gammainc.csv"
    );
    std::fs::read_to_string(path)
        .expect("arquivo de referências")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("funcao"))
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            (
                c[0].to_string(),
                c[1].parse().unwrap(),
                c[2].parse().unwrap(),
                c[3].parse().unwrap(),
            )
        })
        .collect()
}

/// (função, pior erro relativo, argumentos)
fn piores() -> Vec<(String, f64, f64, f64)> {
    let mut out: Vec<(String, f64, f64, f64)> = vec![];
    for (f, a, b, want) in referencias() {
        let got = match f.as_str() {
            "beta" => beta(a, b),
            "betaln" => betaln(a, b),
            "gammainc_lower" => gammainc(a, b, false),
            "gammainc_upper" => gammainc(a, b, true),
            _ => unreachable!(),
        };
        // betaln perto de zero (B ≈ 1): só o erro absoluto tem sentido; escala max(1, |v|).
        let escala = if f == "betaln" {
            want.abs().max(1.0)
        } else {
            want.abs()
        };
        // NaN conta como erro infinito (`NaN > pior` seria falso e o caso passaria em silêncio).
        let e = if got == want {
            0.0
        } else if got.is_nan() {
            f64::INFINITY
        } else {
            (got - want).abs() / escala
        };
        match out.iter_mut().find(|p| p.0 == f) {
            Some(p) if e > p.1 => *p = (f.clone(), e, a, b),
            Some(_) => {}
            None => out.push((f.clone(), e, a, b)),
        }
    }
    out
}

#[test]
fn regimes_dificeis_contra_mpmath() {
    // Antes da reescrita (2026-11-18): beta 9,6e-10 em (0,5; 10⁶), betaln 1,5e-10 e gammainc
    // 2,2e-10 em a = x = 10⁵ (regressão).
    for (f, e, a, b) in piores() {
        eprintln!("{f}: pior {e:e} em ({a}, {b})");
        let tol = if f.starts_with("gammainc") { 8e-15 } else { 4e-15 };
        assert!(e < tol, "{f}: {e:e} em ({a}, {b})");
    }
}

#[test]
fn bordas_de_beta() {
    assert_eq!(beta(f64::INFINITY, 2.0), 0.0);
    assert_eq!(beta(3.0, f64::INFINITY), 0.0);
    assert_eq!(betaln(f64::INFINITY, 2.0), f64::NEG_INFINITY);
    assert!(beta(f64::NAN, 2.0).is_nan() && betaln(-1.0, 2.0).is_nan());
    // Fora do domínio (a ≤ 0 ou b ≤ 0): NaN, inclusive a = 0 e um só argumento negativo.
    for (a, b) in [
        (0.0, 2.0),
        (2.0, 0.0),
        (-1.5, 2.0),
        (2.0, -1.5),
        (-0.5, 300.0),
        (300.0, 0.0),
        (0.0, 300.0),
    ] {
        assert!(beta(a, b).is_nan() && betaln(a, b).is_nan(), "({a}, {b})");
    }
    // Simetria exata e continuidade na troca de método (a + b = 170).
    assert_eq!(beta(300.0, 14.9), beta(14.9, 300.0));
    let (lo, hi) = (beta(85.0, 85.0f64.next_down()), beta(85.0, 85.0));
    assert!((lo - hi).abs() / hi < 1e-14);
    let (lo, hi) = (betaln(169.0, 1.0f64.next_down()), betaln(169.0, 1.0));
    assert!((lo - hi).abs() / hi.abs() < 1e-14);
    // B estoura f64 mas ln B é finito: B(1e-310, 400) ≈ 1/a, ln B ≈ 713,8 (só a dupla-dupla acerta;
    // pelo caminho das gamas daria ln(Inf)).
    // ln B(a, b) = ln Γ(a) + ln Γ(b) − ln Γ(a+b) ≈ −ln a − ψ(b)·a… = −ln a + O(a): com a = 1e-310 o
    // termo O(a) some e ln B = −ln(1e-310) a 1 ulp.
    let l = betaln(1e-310, 400.0);
    assert!(beta(1e-310, 400.0).is_infinite(), "{}", beta(1e-310, 400.0));
    assert!((l + (1e-310f64).ln()).abs() <= 2e-13, "{l}");
    // Regressão: ln Γ de subnormal estourava na reflexão.
    assert!((gammaln(1e-310) + (1e-310f64).ln()).abs() <= 2e-13);
    assert!((gammaln(5e-324) + (5e-324f64).ln()).abs() <= 2e-13);
    // Subestouro e valores extremos.
    assert_eq!(beta(1e6, 1e6), 0.0);
    assert!(betaln(1e6, 1e6) < -1.3e6);
}

/// a de 10⁵ a 10⁹ perto da transição x ≈ a, e caudas até 1e-91. Regressão: a série e a fração
/// contínua paravam em 10⁴ termos e davam P(10⁹, 10⁹) = 0,124 (correto: 0,500004). Para a ≥ 10⁷
/// vale a expansão de Temme; abaixo, série e fração contínua com até 100 + 15√a termos.
#[test]
fn gammainc_com_a_grande() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/gammainc_temme.csv"
    );
    let txt = std::fs::read_to_string(path).expect("arquivo de referências");
    let mut pior = (0.0f64, 0.0, 0.0);
    for l in txt.lines().filter(|l| !l.starts_with('#') && !l.starts_with('a')) {
        let v: Vec<f64> = l.split(',').map(|s| s.parse().unwrap()).collect();
        let (a, x, p, q) = (v[0], v[1], v[2], v[3]);
        for (got, want) in [(gammainc(x, a, false), p), (gammainc(x, a, true), q)] {
            let e = if got == want {
                0.0
            } else {
                (got - want).abs() / want.abs()
            };
            if e > pior.0 {
                pior = (e, a, x);
            }
        }
    }
    eprintln!(
        "gammainc a grande: pior {:e} em a = {}, x = {}",
        pior.0, pior.1, pior.2
    );
    assert!(pior.0 < 2e-14, "{pior:?}");
}

#[test]
fn bordas_de_gammainc() {
    for (x, a) in [
        (f64::NAN, 2.0),
        (1.0, f64::NAN),
        (-1.0, 2.0),
        (1.0, 0.0),
        (1.0, -3.0),
        (-1.0, 2e7),
    ] {
        assert!(
            gammainc(x, a, false).is_nan() && gammainc(x, a, true).is_nan(),
            "({x}, {a})"
        );
    }
    // x = 0 e x = ∞, inclusive no ramo de Temme (a ≥ 10⁷).
    for a in [3.0, 2e7] {
        assert_eq!((gammainc(0.0, a, false), gammainc(0.0, a, true)), (0.0, 1.0));
        assert_eq!(
            (
                gammainc(f64::INFINITY, a, false),
                gammainc(f64::INFINITY, a, true)
            ),
            (1.0, 0.0)
        );
    }
    // Longe da transição no ramo de Temme: P e Q valem 0 ou 1 em f64, sem NaN mesmo com x enorme.
    assert_eq!(
        (gammainc(1e300, 2e7, false), gammainc(1e300, 2e7, true)),
        (1.0, 0.0)
    );
    assert_eq!(
        (gammainc(1e-300, 2e7, false), gammainc(1e-300, 2e7, true)),
        (0.0, 1.0)
    );
    assert_eq!((gammainc(3e7, 2e7, false), gammainc(1e7, 2e7, true)), (1.0, 1.0));
}

/// ln|Γ| com erro relativo puro em pontos que exercitam cada ramo: perto das raízes 1 e 2 (onde o
/// erro absoluto do oráculo principal não distingue os ramos), x pequeno e subnormal, as fronteiras
/// 15 e 1e300, e negativos (reflexão). Oráculo: tests/dados/F042/gammaln_ramos.csv (mpmath).
#[test]
fn gammaln_por_ramo() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F042/gammaln_ramos.csv"
    );
    let txt = std::fs::read_to_string(path).expect("arquivo de referências");
    let mut pior = (0.0f64, 0.0);
    let linhas: Vec<&str> = txt
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with('x'))
        .collect();
    assert!(linhas.len() >= 25, "referências faltando: {}", linhas.len());
    for l in linhas {
        let v: Vec<f64> = l.split(',').map(|s| s.parse().unwrap()).collect();
        let (x, want) = (v[0], v[1]);
        let got = gammaln(x);
        // Negativos: a reflexão ln(π/|sen πx|) − ln Γ(1 − x) cancela quando |Γ(x)| ≈ 1 (ln|Γ(−2,3)|
        // ≈ 0,37 a partir de 1,24 − 0,87), e só o erro absoluto tem sentido ali.
        let escala = if x < 0.0 { want.abs().max(1.0) } else { want.abs() };
        let e = if got == want {
            0.0
        } else if got.is_nan() {
            f64::INFINITY
        } else {
            (got - want).abs() / escala
        };
        if e > pior.0 {
            pior = (e, x);
        }
    }
    eprintln!("gammaln por ramo: pior {:e} em x = {}", pior.0, pior.1);
    assert!(pior.0 < 4e-15, "{pior:?}");
}

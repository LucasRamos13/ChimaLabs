//! F043 — estatística descritiva e pré-processamento (incremento 1).
//!
//! Oráculo: `tests/dados/F043/*.csv`, de `ferramentas/oraculos/f043_estatisticas.py`. São valores
//! exatos em aritmética racional, arredondados no fim; os quantis seguem a definição de Hyndman &
//! Fan implementada de forma independente.

use chimalab_math::stats::{
    BinRule, CenterBy, Endpoints, FillMethod, MovingStat, NanFlag, NormalizeMethod, OutlierMethod,
    QuantileType, ScaleBy, StatsError, VarNorm, Window, along_dim, cummax, cummin, cumprod, cumsum,
    filloutliers, histcounts, histcounts_auto, isoutlier, max, mean, median, min, mode, moving, normalize,
    prctile, prod, quantile, range, rescale, rmoutliers, std, sum, var,
};
use proptest::prelude::*;

const NAN: f64 = f64::NAN;

fn dados() -> Vec<(String, Vec<f64>)> {
    let csv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F043/dados.csv"
    ))
    .unwrap();
    let mut out: Vec<(String, Vec<f64>)> = Vec::new();
    for l in csv.lines().skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        if out.last().is_none_or(|o| o.0 != c[0]) {
            out.push((c[0].to_string(), Vec::new()));
        }
        out.last_mut().unwrap().1.push(c[2].parse().unwrap());
    }
    out
}

fn refs() -> Vec<(String, String, f64, f64)> {
    let csv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F043/referencias.csv"
    ))
    .unwrap();
    csv.lines()
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split(',').collect();
            (
                c[0].to_string(),
                c[1].to_string(),
                c[2].parse().unwrap(),
                c[3].parse().unwrap(),
            )
        })
        .collect()
}

fn ulps(a: f64, b: f64) -> u64 {
    if a == b {
        return 0;
    }
    if a.is_nan() || b.is_nan() || a.signum() != b.signum() {
        return u64::MAX;
    }
    (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs()
}

#[test]
fn contra_oraculo_exato() {
    let d = dados();
    let r = refs();
    assert_eq!(d.len(), 6);
    assert_eq!(r.len(), 6 * (4 + 63));
    for (conj, est, p, v) in &r {
        let x = &d.iter().find(|(n, _)| n == conj).unwrap().1;
        let (obtido, lim) = match est.as_str() {
            // Soma compensada: arredondamento final correto (≤ 1 ulp) mesmo com cancelamento.
            "soma" => (sum(x, NanFlag::Include), 1),
            "media" => (mean(x, NanFlag::Include), 2),
            // Dois passos corrigidos: erro relativo ~ n·ε, aqui ≤ 64 ulps.
            "var_amostral" => (var(x, VarNorm::Sample, NanFlag::Include), 64),
            "var_populacional" => (var(x, VarNorm::Population, NanFlag::Include), 64),
            q => {
                let k: u8 = q[1..].parse().unwrap();
                (
                    quantile(x, &[*p], QuantileType::new(k).unwrap(), NanFlag::Include).unwrap()[0],
                    4,
                )
            }
        };
        // Quantis interpolados: o erro de g (vindo de p binário) multiplica a amplitude dos dados.
        if est.starts_with('q') {
            let amp = max(x, NanFlag::Include) - min(x, NanFlag::Include);
            assert!(
                (obtido - v).abs() <= 8.0 * f64::EPSILON * (v.abs() + amp),
                "{conj} {est} {p}: {obtido} vs {v}"
            );
            continue;
        }
        assert!(
            ulps(obtido, *v) <= lim,
            "{conj} {est} {p}: {obtido} vs {v} ({} ulps)",
            ulps(obtido, *v)
        );
    }
}

#[test]
fn ca1_mean_omitnan_ignora_nan() {
    let x = [1.0, NAN, 3.0, NAN, 8.0];
    assert_eq!(mean(&x, NanFlag::Omit), 4.0);
    assert!(mean(&x, NanFlag::Include).is_nan());
    // Só NaN, ou vazio: NaN.
    assert!(mean(&[NAN, NAN], NanFlag::Omit).is_nan());
    assert!(mean(&[], NanFlag::Omit).is_nan());
    // As demais reduções seguem a mesma regra.
    assert_eq!(sum(&x, NanFlag::Omit), 12.0);
    assert_eq!(prod(&x, NanFlag::Omit), 24.0);
    assert_eq!(median(&x, NanFlag::Omit), 3.0);
    assert_eq!(min(&x, NanFlag::Omit), 1.0);
    assert_eq!(max(&x, NanFlag::Omit), 8.0);
    assert_eq!(range(&x, NanFlag::Omit), 7.0);
    assert!((var(&x, VarNorm::Sample, NanFlag::Omit) - 13.0).abs() <= 1e-14);
    for f in [sum, prod, median, min, max] {
        assert!(f(&x, NanFlag::Include).is_nan());
    }
}

#[test]
fn ca2_movmean_com_janela_par_e_impar() {
    let x = [4.0, 8.0, 6.0, -1.0, -2.0, -3.0, -1.0, 3.0, 4.0, 5.0];
    // Ímpar (3): um de cada lado; bordas encolhidas.
    let m3 = moving(
        &x,
        Window::Centered(3),
        MovingStat::Mean,
        Endpoints::Shrink,
        NanFlag::Include,
    )
    .unwrap();
    let esperado3 = [6.0, 6.0, 13.0 / 3.0, 1.0, -2.0, -2.0, -1.0 / 3.0, 2.0, 4.0, 4.5];
    for (a, b) in m3.iter().zip(esperado3) {
        assert!((a - b).abs() <= 1e-15, "{a} vs {b}");
    }
    // Par (4): dois antes e um depois (centrada entre i − 1 e i).
    let m4 = moving(
        &x,
        Window::Centered(4),
        MovingStat::Mean,
        Endpoints::Shrink,
        NanFlag::Include,
    )
    .unwrap();
    let esperado4 = [6.0, 6.0, 4.25, 2.75, 0.0, -1.75, -0.75, 0.75, 2.75, 4.0];
    for (a, b) in m4.iter().zip(esperado4) {
        assert!((a - b).abs() <= 1e-15, "{a} vs {b}");
    }
    // Assimétrica (2 antes, 0 depois) e bordas descartadas ou preenchidas.
    let ma = moving(
        &x,
        Window::Asymmetric(2, 0),
        MovingStat::Mean,
        Endpoints::Discard,
        NanFlag::Include,
    )
    .unwrap();
    assert_eq!(ma.len(), 8);
    assert_eq!(ma[0], 6.0);
    let mf = moving(
        &x,
        Window::Centered(5),
        MovingStat::Mean,
        Endpoints::Fill(-9.0),
        NanFlag::Include,
    )
    .unwrap();
    assert_eq!((mf[0], mf[1], mf[8], mf[9]), (-9.0, -9.0, -9.0, -9.0));
    assert_eq!(mf[2], 3.0);
    // NaN: propaga na janela, ou é ignorado.
    let y = [1.0, NAN, 3.0, 5.0];
    let inc = moving(
        &y,
        Window::Centered(3),
        MovingStat::Mean,
        Endpoints::Shrink,
        NanFlag::Include,
    )
    .unwrap();
    assert!(inc[0].is_nan() && inc[2].is_nan() && inc[3] == 4.0);
    let om = moving(
        &y,
        Window::Centered(3),
        MovingStat::Mean,
        Endpoints::Shrink,
        NanFlag::Omit,
    )
    .unwrap();
    assert_eq!(om, vec![1.0, 2.0, 4.0, 4.0]);
}

#[test]
fn ca3_rmoutliers_metodo_mediana() {
    // Mediana 10, MAD = 1, limiar 3·1,4826 = 4,448: só 25 e −3 são outliers.
    let x = [9.0, 10.0, 11.0, 10.0, 25.0, 9.0, 11.0, -3.0, 10.0, NAN];
    let (kept, m) = rmoutliers(&x, OutlierMethod::Median(3.0)).unwrap();
    assert_eq!(
        m,
        vec![false, false, false, false, true, false, false, true, false, false]
    );
    assert_eq!(kept.len(), 8);
    assert!(kept[7].is_nan());
    let (_, b) = isoutlier(&x, OutlierMethod::default()).unwrap();
    assert_eq!(b.center, 10.0);
    assert!((b.upper - (10.0 + 3.0 * 1.482_602_218_505_602)).abs() <= 1e-14);
    // Um valor no limite exato não é outlier (comparação estrita).
    let (m, _) = isoutlier(&[0.0, 0.0, 1.0, 1.0, 1.0], OutlierMethod::Quartiles(0.0)).unwrap();
    assert_eq!(m, vec![false; 5]);
}

#[test]
fn mediana_moda_quantis_bordas() {
    assert_eq!(median(&[3.0, 1.0, 2.0], NanFlag::Include), 2.0);
    assert_eq!(median(&[4.0, 1.0, 3.0, 2.0], NanFlag::Include), 2.5);
    // Média dos centrais sem overflow.
    assert_eq!(median(&[f64::MAX, f64::MAX], NanFlag::Include), f64::MAX);
    assert_eq!(median(&[-f64::MAX, f64::MAX], NanFlag::Include), 0.0);
    assert!(median(&[], NanFlag::Include).is_nan());
    assert_eq!(median(&[f64::INFINITY, 1.0, 2.0], NanFlag::Include), 2.0);
    // Moda: empate fica com o menor; NaN ignorado; −0 e +0 iguais.
    assert_eq!(mode(&[3.0, 1.0, 3.0, 1.0, 2.0]), 1.0);
    assert_eq!(mode(&[NAN, 5.0, NAN]), 5.0);
    assert!(mode(&[NAN]).is_nan());
    assert_eq!(mode(&[-0.0, 0.0, 1.0]), 0.0);
    // Quantis: tipo padrão 7, percentis, vazio e NaN.
    let x = [1.0, 2.0, 3.0, 4.0];
    assert_eq!(
        quantile(&x, &[0.5], QuantileType::default(), NanFlag::Include).unwrap(),
        vec![2.5]
    );
    assert_eq!(
        prctile(&x, &[0.0, 100.0], QuantileType::default(), NanFlag::Include).unwrap(),
        vec![1.0, 4.0]
    );
    assert!(quantile(&[], &[0.5], QuantileType::default(), NanFlag::Include).unwrap()[0].is_nan());
    assert!(quantile(&[1.0, NAN], &[0.5], QuantileType::default(), NanFlag::Include).unwrap()[0].is_nan());
    assert_eq!(
        quantile(&[1.0, NAN], &[0.5], QuantileType::default(), NanFlag::Omit).unwrap(),
        vec![1.0]
    );
    assert_eq!(QuantileType::new(5).unwrap().get(), 5);
    // Um único valor: todos os tipos devolvem o próprio valor.
    for k in 1..=9 {
        assert_eq!(
            quantile(
                &[7.0],
                &[0.0, 0.3, 1.0],
                QuantileType::new(k).unwrap(),
                NanFlag::Include
            )
            .unwrap(),
            vec![7.0; 3]
        );
    }
}

#[test]
fn somas_medias_variancias_bordas() {
    assert_eq!(sum(&[], NanFlag::Include), 0.0);
    assert_eq!(prod(&[], NanFlag::Include), 1.0);
    // Soma compensada: 1e100 + 1 − 1e100 = 1 (a ingênua dá 0).
    assert_eq!(sum(&[1e100, 1.0, -1e100], NanFlag::Include), 1.0);
    assert_eq!(sum(&[f64::INFINITY, 1.0], NanFlag::Include), f64::INFINITY);
    assert!(sum(&[f64::INFINITY, -f64::INFINITY], NanFlag::Include).is_nan());
    // Média sem overflow intermediário.
    assert_eq!(mean(&[f64::MAX, f64::MAX], NanFlag::Include), f64::MAX);
    assert_eq!(mean(&[f64::INFINITY, 1.0], NanFlag::Include), f64::INFINITY);
    // Subnormais.
    assert_eq!(mean(&[4e-320, 2e-320], NanFlag::Include), 3e-320);
    // Variância: um valor dá 0; ∞ dá NaN; desvios enormes não estouram.
    assert_eq!(var(&[5.0], VarNorm::Sample, NanFlag::Include), 0.0);
    assert!(var(&[f64::INFINITY], VarNorm::Sample, NanFlag::Include).is_nan());
    assert!(var(&[f64::INFINITY, 1.0], VarNorm::Sample, NanFlag::Include).is_nan());
    assert!(var(&[], VarNorm::Sample, NanFlag::Include).is_nan());
    // Desvios de 1e154: a soma dos quadrados (2e308) estouraria; a reescala dá 1e308.
    let v = var(&[-1e154, 1e154], VarNorm::Population, NanFlag::Include);
    assert!((v / 1e308 - 1.0).abs() <= 1e-15, "{v}");
    assert_eq!(
        var(&[-1e200, 1e200], VarNorm::Population, NanFlag::Include),
        f64::INFINITY
    );
    assert_eq!(
        std(
            &[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0],
            VarNorm::Population,
            NanFlag::Include
        ),
        2.0
    );
    // min/max vazios e com ±∞.
    assert!(min(&[], NanFlag::Omit).is_nan());
    assert_eq!(max(&[f64::NEG_INFINITY, -3.0], NanFlag::Include), -3.0);
}

#[test]
fn cumulativos() {
    let x = [1.0, NAN, 2.0, 0.5, 4.0];
    assert_eq!(cumsum(&x, NanFlag::Omit), vec![1.0, 1.0, 3.0, 3.5, 7.5]);
    let c = cumsum(&x, NanFlag::Include);
    assert!(c[0] == 1.0 && c[1..].iter().all(|v| v.is_nan()));
    assert_eq!(cumprod(&x, NanFlag::Omit), vec![1.0, 1.0, 2.0, 1.0, 4.0]);
    assert_eq!(cummax(&x, NanFlag::Omit), vec![1.0, 1.0, 2.0, 2.0, 4.0]);
    assert_eq!(cummin(&x, NanFlag::Omit), vec![1.0, 1.0, 1.0, 0.5, 0.5]);
    let cm = cummax(&x, NanFlag::Include);
    assert!(cm[0] == 1.0 && cm[1..].iter().all(|v| v.is_nan()));
    let cm = cummin(&[NAN, 2.0, 1.0], NanFlag::Omit);
    assert!(cm[0].is_nan() && cm[1] == 2.0 && cm[2] == 1.0);
    // Soma cumulativa compensada.
    assert_eq!(cumsum(&[1e100, 1.0, -1e100], NanFlag::Include)[2], 1.0);
    assert_eq!(cumsum(&[f64::INFINITY, 1.0], NanFlag::Include)[1], f64::INFINITY);
}

#[test]
fn ao_longo_da_dimensao() {
    // 2×3 column-major: [[1, 3, 5], [2, 4, 6]].
    let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let (m, f) = along_dim(&a, &[2, 3], 0, |l| vec![mean(l, NanFlag::Include)]).unwrap();
    assert_eq!((m, f), (vec![1.5, 3.5, 5.5], vec![1, 3]));
    let (m, f) = along_dim(&a, &[2, 3], 1, |l| vec![sum(l, NanFlag::Include)]).unwrap();
    assert_eq!((m, f), (vec![9.0, 12.0], vec![2, 1]));
    let (c, f) = along_dim(&a, &[2, 3], 1, |l| cumsum(l, NanFlag::Include)).unwrap();
    assert_eq!((c, f), (vec![1.0, 2.0, 4.0, 6.0, 9.0, 12.0], vec![2, 3]));
    // 3-D.
    let b: Vec<f64> = (0..24).map(f64::from).collect();
    let (s, f) = along_dim(&b, &[2, 3, 4], 2, |l| vec![sum(l, NanFlag::Include)]).unwrap();
    assert_eq!(f, vec![2, 3, 1]);
    assert_eq!(s[0], 0.0 + 6.0 + 12.0 + 18.0);
    // Erros.
    assert_eq!(
        along_dim(&a, &[4, 2], 0, |l| l.to_vec()).unwrap_err().id(),
        "chimalab:stats:formaIncompativel"
    );
    assert_eq!(
        along_dim(&a, &[2, 3], 2, |l| l.to_vec()).unwrap_err().id(),
        "chimalab:stats:formaIncompativel"
    );
    let mut k = 0;
    let e = along_dim(&a, &[2, 3], 0, |l| {
        k += 1;
        vec![l[0]; k]
    })
    .unwrap_err();
    assert_eq!(e.id(), "chimalab:stats:formaIncompativel");
    // Dimensão vazia: média de vazio é NaN por coluna.
    let (m, f) = along_dim(&[], &[0, 3], 0, |l| vec![mean(l, NanFlag::Include)]).unwrap();
    assert_eq!(f, vec![1, 3]);
    assert!(m.iter().all(|v| v.is_nan()));
}

#[test]
fn normalizacao_e_reescala() {
    let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let z = normalize(&x, NormalizeMethod::ZScore(VarNorm::Population)).unwrap();
    assert_eq!(z[0], -1.5);
    assert!((mean(&z, NanFlag::Include)).abs() <= 1e-15);
    assert_eq!(normalize(&x, NormalizeMethod::default()).unwrap().len(), 8);
    assert_eq!(normalize(&x, NormalizeMethod::Range(0.0, 1.0)).unwrap()[7], 1.0);
    let n2 = normalize(&[3.0, 4.0], NormalizeMethod::Norm(2.0)).unwrap();
    assert_eq!(n2, vec![0.6, 0.8]);
    assert_eq!(
        normalize(&[3.0, -4.0], NormalizeMethod::Norm(f64::INFINITY)).unwrap(),
        vec![0.75, -1.0]
    );
    assert_eq!(
        normalize(&[1e300, 1e300], NormalizeMethod::Norm(2.0)).unwrap()[0],
        1.0 / 2f64.sqrt()
    );
    assert_eq!(
        normalize(&[0.0, 0.0], NormalizeMethod::Norm(2.0)).unwrap(),
        vec![0.0, 0.0]
    );
    assert_eq!(
        normalize(&[1.0, 3.0, 8.0], NormalizeMethod::Center(CenterBy::Median)).unwrap(),
        vec![-2.0, 0.0, 5.0]
    );
    assert_eq!(
        normalize(&[1.0, 3.0, 8.0], NormalizeMethod::Center(CenterBy::Mean)).unwrap()[0],
        -3.0
    );
    assert_eq!(
        normalize(&[1.0, 3.0], NormalizeMethod::Center(CenterBy::Value(1.0))).unwrap(),
        vec![0.0, 2.0]
    );
    assert_eq!(
        normalize(&[NAN, 2.0, 4.0], NormalizeMethod::Scale(ScaleBy::First)).unwrap()[2],
        2.0
    );
    assert_eq!(
        normalize(&[2.0, 4.0], NormalizeMethod::Scale(ScaleBy::Value(4.0))).unwrap(),
        vec![0.5, 1.0]
    );
    let s = normalize(&x, NormalizeMethod::Scale(ScaleBy::Std)).unwrap();
    assert!((std(&s, VarNorm::Sample, NanFlag::Include) - 1.0).abs() <= 1e-15);
    let _ = normalize(&x, NormalizeMethod::Scale(ScaleBy::Mad)).unwrap();
    let iq = normalize(&[1.0, 2.0, 3.0, 4.0, 5.0], NormalizeMethod::Scale(ScaleBy::Iqr)).unwrap();
    assert_eq!(iq[1], 1.0);
    // Dados constantes: z-score dá zeros (sem dividir por 0); NaN preservado.
    assert_eq!(
        normalize(&[3.0, 3.0, NAN], NormalizeMethod::default()).unwrap()[..2],
        [0.0, 0.0]
    );
    // Reescala: extremos exatos; constante vai ao ponto médio.
    let r = rescale(&[1.0, 2.0, 11.0], -1.0, 1.0).unwrap();
    assert_eq!(r, vec![-1.0, -0.8, 1.0]);
    assert_eq!(rescale(&[5.0, 5.0], 0.0, 1.0).unwrap(), vec![0.5, 0.5]);
    assert!(rescale(&[NAN], 0.0, 1.0).unwrap()[0].is_nan());
    for e in [
        normalize(&x, NormalizeMethod::Norm(0.5)).unwrap_err(),
        normalize(&x, NormalizeMethod::Norm(NAN)).unwrap_err(),
        rescale(&x, 1.0, 0.0).unwrap_err(),
        rescale(&x, 0.0, f64::INFINITY).unwrap_err(),
    ] {
        assert_eq!(e.id(), "chimalab:stats:opcaoInvalida");
    }
}

#[test]
fn outliers_metodos_e_preenchimento() {
    let x = [1.0, 2.0, 3.0, 100.0, 4.0, 5.0, -80.0, 6.0];
    let (m, b) = isoutlier(&x, OutlierMethod::Mean(1.0)).unwrap();
    assert!(m[3] && m[6]);
    assert!(b.lower < b.center && b.center < b.upper);
    let (m, _) = isoutlier(&x, OutlierMethod::Quartiles(1.5)).unwrap();
    assert_eq!(m.iter().filter(|v| **v).count(), 2);
    let (f, _) = filloutliers(&x, FillMethod::Linear, OutlierMethod::default()).unwrap();
    assert_eq!((f[3], f[6]), (3.5, 5.5));
    let (f, _) = filloutliers(&x, FillMethod::Previous, OutlierMethod::default()).unwrap();
    assert_eq!((f[3], f[6]), (3.0, 5.0));
    let (f, _) = filloutliers(&x, FillMethod::Next, OutlierMethod::default()).unwrap();
    assert_eq!((f[3], f[6]), (4.0, 6.0));
    let (f, _) = filloutliers(&x, FillMethod::Nearest, OutlierMethod::default()).unwrap();
    assert_eq!((f[3], f[6]), (3.0, 5.0));
    let (f, _) = filloutliers(&x, FillMethod::Constant(0.0), OutlierMethod::default()).unwrap();
    assert_eq!((f[3], f[6]), (0.0, 0.0));
    let (f, b) = isoutlier(&x, OutlierMethod::default())
        .map(|(_, b)| {
            (
                filloutliers(&x, FillMethod::Clip, OutlierMethod::default())
                    .unwrap()
                    .0,
                b,
            )
        })
        .unwrap();
    assert_eq!((f[3], f[6]), (b.upper, b.lower));
    let (f, _) = filloutliers(&x, FillMethod::Center, OutlierMethod::default()).unwrap();
    assert_eq!(f[3], 3.5);
    // Outliers nas pontas: os posicionais usam o único vizinho bom.
    let y = [100.0, 1.0, 2.0, 1.0, 2.0, -100.0];
    for fm in [
        FillMethod::Previous,
        FillMethod::Next,
        FillMethod::Nearest,
        FillMethod::Linear,
    ] {
        let (f, _) = filloutliers(&y, fm, OutlierMethod::default()).unwrap();
        assert_eq!((f[0], f[5]), (1.0, 2.0), "{fm:?}");
    }
    // Sem nenhum valor bom, os posicionais dão NaN.
    let (f, _) = filloutliers(
        &[0.0, 0.0, 0.0, 5.0],
        FillMethod::Previous,
        OutlierMethod::Median(0.0),
    )
    .unwrap();
    assert_eq!(f, vec![0.0, 0.0, 0.0, 0.0]);
    let (f, _) = filloutliers(&[5.0], FillMethod::Nearest, OutlierMethod::Mean(0.0)).unwrap();
    assert_eq!(f, vec![5.0]);
    assert_eq!(
        isoutlier(&x, OutlierMethod::Median(-1.0)).unwrap_err().id(),
        "chimalab:stats:opcaoInvalida"
    );
    let (m, _) = isoutlier(&[], OutlierMethod::Quartiles(1.5)).unwrap();
    assert!(m.is_empty());
}

#[test]
fn histogramas() {
    let x = [0.0, 0.5, 1.0, 1.5, 2.0, 2.0, NAN, 5.0, -1.0];
    assert_eq!(histcounts(&x, &[0.0, 1.0, 2.0]).unwrap(), vec![2, 4]);
    for e in [&[0.0][..], &[1.0, 0.0], &[0.0, NAN], &[0.0, 0.0]] {
        assert_eq!(
            histcounts(&x, e).unwrap_err().id(),
            "chimalab:stats:opcaoInvalida"
        );
    }
    let y: Vec<f64> = (0..1000).map(|k| ((k * 7919) % 1000) as f64 / 10.0).collect();
    for rule in [
        BinRule::FreedmanDiaconis,
        BinRule::Scott,
        BinRule::Sturges,
        BinRule::Sqrt,
    ] {
        let (c, e) = histcounts_auto(&y, rule);
        assert_eq!(c.iter().sum::<usize>(), 1000, "{rule:?}");
        assert!(e[0] <= 0.0 && *e.last().unwrap() >= 99.9);
        // Largura "redonda" e uniforme.
        let w = e[1] - e[0];
        assert!(e.windows(2).all(|p| ((p[1] - p[0]) - w).abs() <= 1e-9 * w));
        let m = w / 10f64.powf(w.log10().floor());
        assert!(
            [1.0, 2.0, 2.5, 5.0, 10.0].iter().any(|r| (m - r).abs() <= 1e-9),
            "{rule:?}: {w}"
        );
    }
    assert_eq!(
        histcounts_auto(&[], BinRule::default()),
        (vec![0], vec![0.0, 1.0])
    );
    assert_eq!(
        histcounts_auto(&[3.0, 3.0], BinRule::default()),
        (vec![2], vec![2.5, 3.5])
    );
    // IQR nulo: cai na regra de Scott.
    let (c, _) = histcounts_auto(&[1.0, 1.0, 1.0, 1.0, 1.0, 2.0], BinRule::FreedmanDiaconis);
    assert_eq!(c.iter().sum::<usize>(), 6);
}

#[test]
fn erros_com_identificador() {
    let casos: Vec<(StatsError, &str)> = vec![
        (QuantileType::new(0).unwrap_err(), "chimalab:stats:opcaoInvalida"),
        (QuantileType::new(10).unwrap_err(), "chimalab:stats:opcaoInvalida"),
        (
            quantile(&[1.0], &[1.5], QuantileType::default(), NanFlag::Include).unwrap_err(),
            "chimalab:stats:opcaoInvalida",
        ),
        (
            quantile(&[1.0], &[NAN], QuantileType::default(), NanFlag::Include).unwrap_err(),
            "chimalab:stats:opcaoInvalida",
        ),
        (
            prctile(&[1.0], &[101.0], QuantileType::default(), NanFlag::Include).unwrap_err(),
            "chimalab:stats:opcaoInvalida",
        ),
        (
            moving(
                &[1.0],
                Window::Centered(0),
                MovingStat::Mean,
                Endpoints::Shrink,
                NanFlag::Include,
            )
            .unwrap_err(),
            "chimalab:stats:opcaoInvalida",
        ),
    ];
    for (e, id) in casos {
        assert_eq!(e.id(), id, "{e}");
        assert!(!e.to_string().is_empty());
    }
    assert!(
        along_dim(&[1.0], &[2], 0, |l| l.to_vec())
            .unwrap_err()
            .to_string()
            .contains('2')
    );
    // NaN propaga na variância com Include; média móvel sem overflow intermediário.
    assert!(var(&[1.0, NAN], VarNorm::Sample, NanFlag::Include).is_nan());
    let m = moving(
        &[1.7e308, 1.7e308, 1.7e308],
        Window::Centered(3),
        MovingStat::Mean,
        Endpoints::Shrink,
        NanFlag::Include,
    )
    .unwrap();
    assert!(m.iter().all(|v| ulps(*v, 1.7e308) <= 2), "{m:?}");
}

/// Estatística de referência de uma janela, por força bruta.
fn bruto(w: &[f64], stat: MovingStat, nan: NanFlag) -> f64 {
    if nan == NanFlag::Include && w.iter().any(|v| v.is_nan()) {
        return NAN;
    }
    let v: Vec<f64> = w.iter().copied().filter(|a| !a.is_nan()).collect();
    match stat {
        MovingStat::Sum => sum(&v, NanFlag::Include),
        MovingStat::Mean => mean(&v, NanFlag::Include),
        MovingStat::Median => median(&v, NanFlag::Include),
        MovingStat::Max => max(&v, NanFlag::Include),
        MovingStat::Min => min(&v, NanFlag::Include),
        MovingStat::Prod => prod(&v, NanFlag::Include),
        MovingStat::Var(n) => var(&v, n, NanFlag::Include),
        MovingStat::Std(n) => std(&v, n, NanFlag::Include),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// Estatísticas móveis contra força bruta janela a janela, em todas as combinações.
    #[test]
    fn moveis_contra_forca_bruta(
        x in prop::collection::vec(prop_oneof![9 => -100.0f64..100.0, 1 => Just(f64::NAN)], 0..60),
        k in 1usize..9,
        antes in 0usize..5,
        depois in 0usize..5,
        assim in any::<bool>(),
        omit in any::<bool>(),
    ) {
        let nan = if omit { NanFlag::Omit } else { NanFlag::Include };
        let w = if assim { Window::Asymmetric(antes, depois) } else { Window::Centered(k) };
        let (b, a) = match w {
            Window::Centered(k) if k % 2 == 1 => ((k - 1) / 2, (k - 1) / 2),
            Window::Centered(k) => (k / 2, k / 2 - 1),
            Window::Asymmetric(b, a) => (b, a),
        };
        let n = x.len();
        let stats = [MovingStat::Sum, MovingStat::Mean, MovingStat::Median, MovingStat::Max, MovingStat::Min,
                     MovingStat::Prod, MovingStat::Var(VarNorm::Sample), MovingStat::Std(VarNorm::Population)];
        for st in stats {
            let r = moving(&x, w, st, Endpoints::Shrink, nan).unwrap();
            prop_assert_eq!(r.len(), n);
            for (i, ri) in r.iter().enumerate() {
                let lo = i.saturating_sub(b);
                let hi = (i + a).min(n - 1);
                let e = bruto(&x[lo..=hi], st, nan);
                prop_assert!(
                    (ri.is_nan() && e.is_nan()) || (ri - e).abs() <= 1e-9 * e.abs().max(1.0),
                    "{:?} i={} {} vs {}", st, i, ri, e
                );
            }
            let d = moving(&x, w, st, Endpoints::Discard, nan).unwrap();
            prop_assert_eq!(d.len(), n.saturating_sub(a + b));
        }
    }

    /// Média e variância invariantes por translação; variância escala com o quadrado.
    #[test]
    fn media_variancia_afins(x in prop::collection::vec(-1e3f64..1e3, 2..50), c in -1e6f64..1e6, s in 0.5f64..4.0) {
        let y: Vec<f64> = x.iter().map(|v| s * v + c).collect();
        let (mx, my) = (mean(&x, NanFlag::Include), mean(&y, NanFlag::Include));
        prop_assert!((my - (s * mx + c)).abs() <= 1e-9 * (c.abs() + 1.0));
        let (vx, vy) = (var(&x, VarNorm::Sample, NanFlag::Include), var(&y, VarNorm::Sample, NanFlag::Include));
        prop_assert!((vy - s * s * vx).abs() <= 1e-6 * vy.max(1.0));
    }

    /// Quantis: monótonos em p e entre o mínimo e o máximo, em todos os tipos.
    #[test]
    fn quantis_monotonos(x in prop::collection::vec(-50.0f64..50.0, 1..40)) {
        let ps: Vec<f64> = (0..=20).map(|k| k as f64 / 20.0).collect();
        for k in 1..=9 {
            let q = quantile(&x, &ps, QuantileType::new(k).unwrap(), NanFlag::Include).unwrap();
            prop_assert!(q.windows(2).all(|w| w[0] <= w[1]), "tipo {}", k);
            let (lo, hi) = (min(&x, NanFlag::Include), max(&x, NanFlag::Include));
            prop_assert!(q.iter().all(|v| *v >= lo && *v <= hi));
        }
    }
}

// ================================================================ incremento 2

#[test]
fn suavizacao_reproduz_polinomios() {
    use chimalab_math::stats::{SmoothMethod, smoothdata};
    let n = 40;
    let cub: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64;
            0.5 - 0.3 * t + 0.02 * t * t - 0.0004 * t * t * t
        })
        .collect();
    // Savitzky–Golay de grau 3 reproduz cúbicas em toda parte (inclusive nas bordas).
    let s = smoothdata(&cub, SmoothMethod::SavitzkyGolay { window: 7, degree: 3 }).unwrap();
    for (a, b) in s.iter().zip(&cub) {
        assert!((a - b).abs() <= 1e-10 * b.abs().max(1.0), "{a} vs {b}");
    }
    // LOESS (parábola local) reproduz parábolas; LOWESS reproduz retas, robusto ou não.
    let par: Vec<f64> = (0..n)
        .map(|i| 1.0 + 0.5 * i as f64 - 0.01 * (i * i) as f64)
        .collect();
    let lin: Vec<f64> = (0..n).map(|i| 2.0 - 0.25 * i as f64).collect();
    for robust in [false, true] {
        let s = smoothdata(&par, SmoothMethod::Loess { span: 0.3, robust }).unwrap();
        assert!(
            s.iter().zip(&par).all(|(a, b)| (a - b).abs() <= 1e-9),
            "loess {robust}"
        );
        let s = smoothdata(&lin, SmoothMethod::Lowess { span: 0.2, robust }).unwrap();
        assert!(
            s.iter().zip(&lin).all(|(a, b)| (a - b).abs() <= 1e-9),
            "lowess {robust}"
        );
    }
    // Gaussiana preserva constantes e, no interior, retas (pesos simétricos).
    let c = smoothdata(&[3.0; 20], SmoothMethod::Gaussian(5)).unwrap();
    assert!(c.iter().all(|v| (v - 3.0).abs() <= 1e-15));
    let g = smoothdata(&lin, SmoothMethod::Gaussian(5)).unwrap();
    for i in 3..n - 3 {
        assert!((g[i] - lin[i]).abs() <= 1e-12);
    }
}

#[test]
fn lowess_robusto_resiste_a_outliers() {
    use chimalab_math::stats::{SmoothMethod, smoothdata};
    let mut y: Vec<f64> = (0..50).map(|i| (i as f64 * 0.2).sin()).collect();
    let limpo = y.clone();
    y[20] = 10.0;
    y[35] = -8.0;
    let comum = smoothdata(
        &y,
        SmoothMethod::Lowess {
            span: 0.15,
            robust: false,
        },
    )
    .unwrap();
    let rob = smoothdata(
        &y,
        SmoothMethod::Lowess {
            span: 0.15,
            robust: true,
        },
    )
    .unwrap();
    // O robusto fica perto do sinal limpo nos outliers; o comum é puxado por eles.
    assert!((rob[20] - limpo[20]).abs() < 0.1, "{}", rob[20]);
    assert!((comum[20] - limpo[20]).abs() > 0.5);
    // NaN é ignorado e preservado como NaN.
    let mut z = limpo.clone();
    z[10] = f64::NAN;
    let s = smoothdata(
        &z,
        SmoothMethod::Lowess {
            span: 0.2,
            robust: false,
        },
    )
    .unwrap();
    assert!(s[10].is_nan() && s[11].is_finite());
    let s = smoothdata(&z, SmoothMethod::SavitzkyGolay { window: 5, degree: 2 }).unwrap();
    assert!(s.iter().all(|v| v.is_finite()));
    let s = smoothdata(&z, SmoothMethod::Gaussian(3)).unwrap();
    assert!(s[10].is_finite());
    // Só NaN; erros de opção.
    assert!(
        smoothdata(
            &[f64::NAN; 4],
            SmoothMethod::Lowess {
                span: 0.5,
                robust: true
            }
        )
        .unwrap()
        .iter()
        .all(|v| v.is_nan())
    );
    assert!(
        smoothdata(&[f64::NAN; 4], SmoothMethod::Gaussian(3))
            .unwrap()
            .iter()
            .all(|v| v.is_nan())
    );
    assert!(
        smoothdata(
            &[f64::NAN; 6],
            SmoothMethod::SavitzkyGolay { window: 3, degree: 1 }
        )
        .unwrap()
        .iter()
        .all(|v| v.is_nan())
    );
    for m in [
        SmoothMethod::Gaussian(0),
        SmoothMethod::SavitzkyGolay { window: 4, degree: 1 },
        SmoothMethod::SavitzkyGolay { window: 3, degree: 3 },
        SmoothMethod::Lowess {
            span: 0.0,
            robust: false,
        },
        SmoothMethod::Loess {
            span: 1.5,
            robust: false,
        },
    ] {
        assert_eq!(
            smoothdata(&limpo, m).unwrap_err().id(),
            "chimalab:stats:opcaoInvalida"
        );
    }
    // Resíduos todos nulos no robusto (dados exatamente lineares): pesos 1, e o ajuste local
    // reproduz a reta. A igualdade é a menos de alguns ulp: os pesos tricúbicos usam `powi`,
    // cuja precisão varia com a plataforma (no Windows, 4.999999999999999).
    let lin: Vec<f64> = (0..10).map(f64::from).collect();
    let v = smoothdata(
        &lin,
        SmoothMethod::Lowess {
            span: 1.0,
            robust: true,
        },
    )
    .unwrap()[5];
    assert!((v - 5.0).abs() <= 4.0 * f64::EPSILON * 5.0, "{v}");
}

#[test]
fn extremos_locais_com_proeminencia() {
    use chimalab_math::stats::local_extrema;
    //            0    1    2    3    4    5    6    7    8    9   10
    let y = [0.0, 3.0, 1.0, 5.0, 2.0, 2.5, 2.0, 4.0, 4.0, 4.0, 0.0];
    let p = local_extrema(&y, false, 0.0, 0);
    // Picos em 1 (prom. 3 − 1 = 2... à esquerda o mínimo é 0 e à direita até 5 é 1: base 1),
    // 3 (o mais alto: base max(0, 0) = 0, prom. 5), 5 (prom. 0,5) e o platô 7–9 (centro 8,
    // base max(2, 0) = 2, prom. 2).
    assert_eq!(p, vec![(1, 2.0), (3, 5.0), (5, 0.5), (8, 2.0)]);
    // Filtro de proeminência e de separação (o mais alto tem prioridade).
    assert_eq!(
        local_extrema(&y, false, 1.0, 0)
            .iter()
            .map(|p| p.0)
            .collect::<Vec<_>>(),
        vec![1, 3, 8]
    );
    assert_eq!(
        local_extrema(&y, false, 0.0, 3)
            .iter()
            .map(|p| p.0)
            .collect::<Vec<_>>(),
        vec![3, 8]
    );
    // Mínimos.
    let m = local_extrema(&y, true, 0.0, 0);
    assert_eq!(m.iter().map(|p| p.0).collect::<Vec<_>>(), vec![2, 4, 6]);
    // Bordas e NaN não são extremos; vetor curto.
    assert!(local_extrema(&[1.0, 0.0, 1.0], false, 0.0, 0).is_empty());
    assert!(local_extrema(&[0.0, f64::NAN, 0.0], false, 0.0, 0).is_empty());
    assert!(local_extrema(&[1.0], false, 0.0, 0).is_empty());
    // NaN ao lado de um candidato impede a comparação (não é pico); no vale de outro pico, é
    // ignorado no cálculo da proeminência.
    let q = local_extrema(&[0.0, 2.0, f64::NAN, 1.0, 3.0, 0.0], false, 0.0, 0);
    assert_eq!(q, vec![(4, 3.0)]);
}

#[test]
fn gesd_e_grubbs() {
    use chimalab_math::stats::gesd;
    // Dados quase normais (quantis de N(0,1)) com dois outliers injetados.
    let mut x: Vec<f64> = (1..=30)
        .map(|k| {
            let p = (k as f64 - 0.5) / 30.0;
            // Aproximação de Φ⁻¹ suficiente para gerar dados simétricos.
            let t = (-2.0 * (p.min(1.0 - p)).ln()).sqrt();
            let z = t
                - (2.515517 + 0.802853 * t + 0.010328 * t * t)
                    / (1.0 + 1.432788 * t + 0.189269 * t * t + 0.001308 * t * t * t);
            if p < 0.5 { -z } else { z }
        })
        .collect();
    x[5] = 9.0;
    x[17] = -7.5;
    let m = gesd(&x, 5, 0.05).unwrap();
    assert!(m[5] && m[17]);
    assert_eq!(m.iter().filter(|b| **b).count(), 2);
    // Grubbs (um outlier no máximo).
    let g = gesd(&x, 1, 0.05).unwrap();
    assert_eq!(g.iter().filter(|b| **b).count(), 1);
    assert!(g[5]);
    // Sem outliers: nada marcado; constante: nada; NaN ignorado; poucos pontos.
    let limpo: Vec<f64> = (0..20).map(|k| (k as f64 * 0.7).sin()).collect();
    assert!(gesd(&limpo, 3, 0.05).unwrap().iter().all(|b| !b));
    assert!(gesd(&[2.0; 10], 3, 0.05).unwrap().iter().all(|b| !b));
    let mut com_nan = x.clone();
    com_nan[0] = f64::NAN;
    assert!(!gesd(&com_nan, 5, 0.05).unwrap()[0]);
    assert!(gesd(&[1.0, 100.0], 1, 0.05).unwrap().iter().all(|b| !b));
    assert_eq!(gesd(&x, 1, 1.5).unwrap_err().id(), "chimalab:stats:opcaoInvalida");
}

// ================================================================ incremento 3: mudanças

/// Ruído determinístico pequeno (sequência de baixa discrepância centrada).
fn ruido(i: usize, a: f64) -> f64 {
    a * (((i as f64) * 0.618_033_988_749_895).fract() - 0.5)
}

#[test]
fn mudancas_de_media_reta_e_variancia() {
    use chimalab_math::stats::{ChangeKind, changepoints};
    let mut y: Vec<f64> = Vec::new();
    for (k, nivel) in [(30, 0.0), (30, 5.0), (40, -2.0)] {
        for _ in 0..k {
            let i = y.len();
            y.push(nivel + ruido(i, 0.5));
        }
    }
    assert_eq!(changepoints(&y, ChangeKind::Mean, 5.0, 1).unwrap(), vec![30, 60]);
    // Penalidade enorme: nenhuma mudança; zero: muitas.
    assert!(changepoints(&y, ChangeKind::Mean, 1e9, 1).unwrap().is_empty());
    assert!(changepoints(&y, ChangeKind::Mean, 0.0, 1).unwrap().len() > 10);
    // Reta com quebra de inclinação em 50.
    let z: Vec<f64> = (0..100).map(|i| if i < 50 { 0.1 * i as f64 } else { 5.0 - 0.3 * (i - 50) as f64 } + ruido(i, 0.05)).collect();
    assert_eq!(changepoints(&z, ChangeKind::Linear, 1.0, 3).unwrap(), vec![50]);
    // Variância que muda em 60 (média constante).
    let w: Vec<f64> = (0..120)
        .map(|i| ruido(i, if i < 60 { 0.2 } else { 4.0 }))
        .collect();
    assert_eq!(
        changepoints(&w, ChangeKind::MeanVariance, 10.0, 5).unwrap(),
        vec![60]
    );
    // Segmento mínimo: poucos pontos não comportam duas partes.
    assert!(
        changepoints(&[1.0, 9.0, 1.0], ChangeKind::Mean, 0.0, 2)
            .unwrap()
            .is_empty()
    );
    // Erros.
    for (p, m) in [(-1.0, 1), (f64::NAN, 1), (1.0, 0)] {
        assert_eq!(
            changepoints(&y, ChangeKind::Mean, p, m).unwrap_err().id(),
            "chimalab:stats:opcaoInvalida"
        );
    }
    assert_eq!(
        changepoints(&[1.0, f64::NAN, 2.0], ChangeKind::Mean, 1.0, 1)
            .unwrap_err()
            .id(),
        "chimalab:stats:opcaoInvalida"
    );
}

/// Custo total de uma partição (oráculo independente: média e reta por fórmulas diretas).
fn custo_particao(y: &[f64], cps: &[usize], kind: chimalab_math::stats::ChangeKind, pen: f64) -> f64 {
    use chimalab_math::stats::ChangeKind;
    let mut bordas = vec![0];
    bordas.extend_from_slice(cps);
    bordas.push(y.len());
    let mut total = pen * cps.len() as f64;
    for w in bordas.windows(2) {
        let seg = &y[w[0]..w[1]];
        let k = seg.len() as f64;
        let m = seg.iter().sum::<f64>() / k;
        total += match kind {
            ChangeKind::Mean => seg.iter().map(|v| (v - m).powi(2)).sum::<f64>(),
            ChangeKind::Linear => {
                if seg.len() < 2 {
                    0.0
                } else {
                    let tm = (w[0] + w[1] - 1) as f64 / 2.0;
                    let sxx: f64 = (w[0]..w[1]).map(|t| (t as f64 - tm).powi(2)).sum();
                    let sxy: f64 = (w[0]..w[1])
                        .zip(seg)
                        .map(|(t, v)| (t as f64 - tm) * (v - m))
                        .sum();
                    let b = sxy / sxx;
                    (w[0]..w[1])
                        .zip(seg)
                        .map(|(t, v)| (v - m - b * (t as f64 - tm)).powi(2))
                        .sum::<f64>()
                }
            }
            ChangeKind::MeanVariance => unreachable!(),
        };
    }
    total
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// PELT (com poda) dá exatamente a partição da programação dinâmica completa.
    #[test]
    fn pelt_igual_a_particao_otima(y in prop::collection::vec(-5.0f64..5.0, 2..60), pen in 0.0f64..20.0, min_len in 1usize..4, linear in any::<bool>()) {
        use chimalab_math::stats::{ChangeKind, changepoints, changepoints_exact};
        let kind = if linear { ChangeKind::Linear } else { ChangeKind::Mean };
        let a = changepoints(&y, kind, pen, min_len).unwrap();
        let b = changepoints_exact(&y, kind, pen, min_len).unwrap();
        prop_assert_eq!(a, b);
    }

    /// A partição ótima tem o menor custo entre todas as partições (força bruta, n ≤ 10).
    #[test]
    fn particao_otima_contra_forca_bruta(y in prop::collection::vec(-5.0f64..5.0, 2..11), pen in 0.0f64..10.0, linear in any::<bool>()) {
        use chimalab_math::stats::{ChangeKind, changepoints_exact};
        let kind = if linear { ChangeKind::Linear } else { ChangeKind::Mean };
        let n = y.len();
        let cps = changepoints_exact(&y, kind, pen, 1).unwrap();
        let achado = custo_particao(&y, &cps, kind, pen);
        let mut melhor = f64::INFINITY;
        for mask in 0u32..(1 << (n - 1)) {
            let c: Vec<usize> = (1..n).filter(|i| mask >> (i - 1) & 1 == 1).collect();
            melhor = melhor.min(custo_particao(&y, &c, kind, pen));
        }
        prop_assert!(achado <= melhor + 1e-9 * melhor.abs().max(1.0), "{} vs {}", achado, melhor);
    }
}

// ================================================================ pesos

#[test]
fn ponderadas_casos_e_erros() {
    use chimalab_math::stats::{
        NanFlag, WeightNorm, mean_weighted, median_weighted, std_weighted, var_weighted,
    };
    let x = [1.0, 2.0, 4.0];
    let w = [1.0, 2.0, 1.0];
    // Média (1 + 4 + 4)/4 = 2,25; Σw(x − m)² = 1,5625 + 0,125 + 3,0625 = 4,75.
    assert_eq!(mean_weighted(&x, &w, NanFlag::Include).unwrap(), 2.25);
    assert!(
        (var_weighted(&x, &w, WeightNorm::Population, NanFlag::Include).unwrap() - 4.75 / 4.0).abs() <= 1e-15
    );
    assert!(
        (var_weighted(&x, &w, WeightNorm::Frequency, NanFlag::Include).unwrap() - 4.75 / 3.0).abs() <= 1e-15
    );
    // Confiabilidade: Σw − Σw²/Σw = 4 − 6/4 = 2,5.
    assert!(
        (var_weighted(&x, &w, WeightNorm::Reliability, NanFlag::Include).unwrap() - 4.75 / 2.5).abs()
            <= 1e-15
    );
    assert!(
        (std_weighted(&x, &w, WeightNorm::Population, NanFlag::Include).unwrap() - (4.75f64 / 4.0).sqrt())
            .abs()
            <= 1e-15
    );
    // Mediana: acumulado 1, 3, 4; metade 2 → 2. Pesos [1, 1, 2]: acumulado 2 = metade → (2 + 4)/2.
    assert_eq!(median_weighted(&x, &w, NanFlag::Include).unwrap(), 2.0);
    assert_eq!(
        median_weighted(&x, &[1.0, 1.0, 2.0], NanFlag::Include).unwrap(),
        3.0
    );
    assert_eq!(
        median_weighted(&[5.0, 1.0], &[0.3, 0.7], NanFlag::Include).unwrap(),
        1.0
    );
    // Peso zero não conta; todos zero: NaN; um só par: variância 0.
    assert_eq!(
        mean_weighted(&[1.0, 1e300], &[1.0, 0.0], NanFlag::Include).unwrap(),
        1.0
    );
    assert!(mean_weighted(&x, &[0.0; 3], NanFlag::Include).unwrap().is_nan());
    assert!(median_weighted(&x, &[0.0; 3], NanFlag::Include).unwrap().is_nan());
    assert!(
        var_weighted(&x, &[0.0; 3], WeightNorm::Frequency, NanFlag::Include)
            .unwrap()
            .is_nan()
    );
    assert_eq!(
        var_weighted(&[3.0], &[0.5], WeightNorm::Frequency, NanFlag::Include).unwrap(),
        0.0
    );
    // Frequência com Σw ≤ 1 e dispersão: denominador ≤ 0, NaN.
    assert!(
        var_weighted(
            &[1.0, 2.0],
            &[0.25, 0.25],
            WeightNorm::Frequency,
            NanFlag::Include
        )
        .unwrap()
        .is_nan()
    );
    // NaN: propaga ou é omitido.
    let xn = [1.0, f64::NAN, 3.0];
    assert!(mean_weighted(&xn, &[1.0; 3], NanFlag::Include).unwrap().is_nan());
    assert!(
        var_weighted(&xn, &[1.0; 3], WeightNorm::Population, NanFlag::Include)
            .unwrap()
            .is_nan()
    );
    assert!(
        median_weighted(&xn, &[1.0; 3], NanFlag::Include)
            .unwrap()
            .is_nan()
    );
    assert_eq!(mean_weighted(&xn, &[1.0; 3], NanFlag::Omit).unwrap(), 2.0);
    assert_eq!(median_weighted(&xn, &[1.0; 3], NanFlag::Omit).unwrap(), 2.0);
    // Infinito na média: variância NaN.
    assert!(
        var_weighted(
            &[1.0, f64::INFINITY],
            &[1.0; 2],
            WeightNorm::Population,
            NanFlag::Include
        )
        .unwrap()
        .is_nan()
    );
    // Erros.
    assert_eq!(
        mean_weighted(&x, &[1.0; 2], NanFlag::Include).unwrap_err().id(),
        "chimalab:stats:formaIncompativel"
    );
    for wr in [[-1.0, 1.0, 1.0], [f64::NAN, 1.0, 1.0], [f64::INFINITY, 1.0, 1.0]] {
        assert_eq!(
            var_weighted(&x, &wr, WeightNorm::Population, NanFlag::Include)
                .unwrap_err()
                .id(),
            "chimalab:stats:opcaoInvalida"
        );
        assert_eq!(
            median_weighted(&x, &wr, NanFlag::Include).unwrap_err().id(),
            "chimalab:stats:opcaoInvalida"
        );
        assert_eq!(
            std_weighted(&x, &wr, WeightNorm::Population, NanFlag::Include)
                .unwrap_err()
                .id(),
            "chimalab:stats:opcaoInvalida"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Pesos inteiros = dados replicados (média, variância de frequência e populacional,
    /// mediana); confiabilidade e populacional invariantes a w → c·w.
    #[test]
    fn ponderadas_contra_replicacao(dados in prop::collection::vec((-100.0f64..100.0, 0u32..5), 1..30), c in 0.01f64..100.0) {
        use chimalab_math::stats::{NanFlag, VarNorm, WeightNorm, mean, mean_weighted, median, median_weighted, var, var_weighted};
        let x: Vec<f64> = dados.iter().map(|d| d.0).collect();
        let w: Vec<f64> = dados.iter().map(|d| f64::from(d.1)).collect();
        let rep: Vec<f64> = dados.iter().flat_map(|&(v, k)| std::iter::repeat_n(v, k as usize)).collect();
        prop_assume!(!rep.is_empty());
        let n = NanFlag::Include;
        let rel = |a: f64, b: f64| (a - b).abs() <= 1e-12 * b.abs().max(1.0);
        prop_assert!(rel(mean_weighted(&x, &w, n).unwrap(), mean(&rep, n)));
        prop_assert!(rel(var_weighted(&x, &w, WeightNorm::Population, n).unwrap(), var(&rep, VarNorm::Population, n)));
        if rep.len() > 1 {
            prop_assert!(rel(var_weighted(&x, &w, WeightNorm::Frequency, n).unwrap(), var(&rep, VarNorm::Sample, n)));
        }
        prop_assert_eq!(median_weighted(&x, &w, n).unwrap(), median(&rep, n));
        let wc: Vec<f64> = w.iter().map(|v| v * c).collect();
        for norm in [WeightNorm::Population, WeightNorm::Reliability] {
            let a = var_weighted(&x, &w, norm, n).unwrap();
            let b = var_weighted(&x, &wc, norm, n).unwrap();
            prop_assert!((a.is_nan() && b.is_nan()) || (a - b).abs() <= 1e-10 * a.abs().max(1.0), "{:?} {} {}", norm, a, b);
        }
        prop_assert!(rel(mean_weighted(&x, &wc, n).unwrap(), mean(&rep, n)));
    }
}

/// Mutação da detecção de mudanças: empates exatos, segmentos de variância nula e média e
/// variância contra força bruta.
#[test]
fn mudancas_empates_e_variancia_nula() {
    use chimalab_math::stats::{ChangeKind, changepoints, changepoints_exact};
    // Dados constantes com penalidade 0: todas as partições custam 0; o desempate escolhe o maior
    // início do último segmento, o que dá uma mudança em cada ponto (regra documentada).
    let c = [3.0; 6];
    assert_eq!(
        changepoints(&c, ChangeKind::Mean, 0.0, 1).unwrap(),
        vec![1, 2, 3, 4, 5]
    );
    assert_eq!(
        changepoints_exact(&c, ChangeKind::Mean, 0.0, 1).unwrap(),
        vec![1, 2, 3, 4, 5]
    );
    // Com penalidade positiva, nenhuma mudança.
    assert!(changepoints(&c, ChangeKind::Mean, 0.1, 1).unwrap().is_empty());
    // Média e variância com segmentos exatamente constantes: o piso da variância decide, e a
    // mudança fica exatamente na emenda.
    let d = [1.0, 1.0, 1.0, 1.0, 5.0, 5.0, 5.0, 5.0];
    assert_eq!(
        changepoints(&d, ChangeKind::MeanVariance, 1.0, 2).unwrap(),
        vec![4]
    );
    let e = [1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 0.0, 2.0, 0.0, 2.0];
    assert_eq!(
        changepoints(&e, ChangeKind::MeanVariance, 1.0, 2).unwrap(),
        vec![5]
    );
}

/// Custo k·ln(σ̂²) com o mesmo piso do código (oráculo direto, sem somas acumuladas).
fn custo_mv(seg: &[f64]) -> f64 {
    let k = seg.len() as f64;
    let m = seg.iter().sum::<f64>() / k;
    let ss: f64 = seg.iter().map(|v| (v - m).powi(2)).sum();
    let s2: f64 = seg.iter().map(|v| v * v).sum();
    k * (ss / k).max(1e-12 * (s2 / k).max(f64::MIN_POSITIVE)).ln()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Média e variância: a partição devolvida (sem poda) é a de menor custo entre todas, por
    /// força bruta (n ≤ 10, segmentos ≥ 2).
    #[test]
    fn media_variancia_contra_forca_bruta(y in prop::collection::vec(-5.0f64..5.0, 4..11), pen in 0.0f64..5.0) {
        use chimalab_math::stats::{ChangeKind, changepoints};
        let n = y.len();
        let custo = |cps: &[usize]| {
            let mut b = vec![0];
            b.extend_from_slice(cps);
            b.push(n);
            b.windows(2).map(|w| custo_mv(&y[w[0]..w[1]])).sum::<f64>() + pen * cps.len() as f64
        };
        let achado = changepoints(&y, ChangeKind::MeanVariance, pen, 2).unwrap();
        let mut melhor = f64::INFINITY;
        for mask in 0u32..(1 << (n - 1)) {
            let c: Vec<usize> = (1..n).filter(|i| mask >> (i - 1) & 1 == 1).collect();
            let mut b = vec![0];
            b.extend_from_slice(&c);
            b.push(n);
            if b.windows(2).all(|w| w[1] - w[0] >= 2) {
                melhor = melhor.min(custo(&c));
            }
        }
        prop_assert!(custo(&achado) <= melhor + 1e-9 * melhor.abs().max(1.0), "{} vs {}", custo(&achado), melhor);
    }
}

/// Histograma automático: largura de cada regra pelas definições e o arredondamento "redondo"
/// (mutação; antes só a soma das contagens era conferida). Valores esperados à mão:
/// x = 0..99 (n = 100, amplitude 99, σ = 29,0115, IQR entre 49,5 e 50 conforme a interpolação):
/// Sturges 99/8 = 12,4 → 20; √n: 9,9 → 10; Scott 3,49·σ·n^{−1/3} = 21,8 → 25; Freedman–Diaconis
/// 2·IQR·n^{−1/3} ≈ 21,4 → 25.
#[test]
fn histograma_automatico_larguras_das_regras() {
    let larg = |e: &[f64]| e[1] - e[0];
    let x: Vec<f64> = (0..100).map(f64::from).collect();
    for (regra, w) in [
        (BinRule::Sturges, 20.0),
        (BinRule::Sqrt, 10.0),
        (BinRule::Scott, 25.0),
        (BinRule::FreedmanDiaconis, 25.0),
    ] {
        let (c, e) = histcounts_auto(&x, regra);
        assert_eq!(larg(&e), w, "{regra:?}: {e:?}");
        assert_eq!(e[0], 0.0, "{regra:?}");
        assert!(
            e[e.len() - 2] < 99.0 && *e.last().unwrap() >= 99.0,
            "{regra:?}: {e:?}"
        );
        assert_eq!(c.iter().sum::<usize>(), 100);
    }
    // Deslocado para negativos: a primeira borda é o múltiplo da largura logo abaixo do mínimo.
    let xd: Vec<f64> = x.iter().map(|v| v - 37.2).collect();
    let (_, e) = histcounts_auto(&xd, BinRule::Sturges);
    assert_eq!((e[0], larg(&e)), (-40.0, 20.0), "{e:?}");
    // IQR nulo no Freedman–Diaconis: cai no Scott (3,49·0,408·6^{−1/3} = 0,784 → 1).
    let (_, e) = histcounts_auto(&[1.0, 1.0, 1.0, 1.0, 1.0, 2.0], BinRule::FreedmanDiaconis);
    assert_eq!(larg(&e), 1.0, "{e:?}");
    // Arredondamento: n = 4 com √n → amplitude/2. 2,5 fica 2,5; 0,0123 vai a 0,02; 7 vai a 10.
    for (amp, w) in [(5.0, 2.5), (0.0246, 0.02), (14.0, 10.0), (3.0, 2.0), (0.2, 0.1)] {
        let (_, e) = histcounts_auto(&[0.0, amp / 3.0, amp / 2.0, amp], BinRule::Sqrt);
        assert!((larg(&e) - w).abs() <= 1e-15 * w, "amplitude {amp}: {e:?}");
    }
    // No máximo 10⁶ intervalos: um valor extremo força a largura mínima amplitude/10⁶ = 1000.
    let mut y: Vec<f64> = (0..1000).map(|i| i as f64 / 1000.0).collect();
    y.push(1e9);
    let (c, e) = histcounts_auto(&y, BinRule::FreedmanDiaconis);
    assert_eq!(larg(&e), 1000.0);
    assert!(c.len() <= 1_000_001 && c.iter().sum::<usize>() == 1001);
}

/// Valores de `isoutlier`, `filloutliers`, `normalize` e `rescale` (mutação; antes só as
/// máscaras eram conferidas). Os limites são recompostos aqui a partir da mediana, do MAD
/// (mediana de |x − mediana|, por ordenação), da média, do desvio e dos quartis.
#[test]
fn outliers_e_normalizacao_valores() {
    let mad_normal = 1.0 / (std::f64::consts::SQRT_2 * 0.476_936_276_204_469_9); // 1/(√2·erfinv(½))
    let x = [2.0, 3.0, 3.5, 4.0, 5.0, 5.5, 40.0, 6.0, 1.0];
    let mut s = x.to_vec();
    s.sort_by(f64::total_cmp);
    let med = s[4];
    let mut d: Vec<f64> = x.iter().map(|v| (v - med).abs()).collect();
    d.sort_by(f64::total_cmp);
    let mad = d[4];
    for t in [1.5, 3.0] {
        let (_, b) = isoutlier(&x, OutlierMethod::Median(t)).unwrap();
        let sc = mad_normal * mad;
        assert!(
            (b.lower - (med - t * sc)).abs() <= 1e-13
                && (b.upper - (med + t * sc)).abs() <= 1e-13
                && b.center == med,
            "{b:?}"
        );
        let (_, b) = isoutlier(&x, OutlierMethod::Mean(t)).unwrap();
        let (m, sd) = (mean(&x, NanFlag::Omit), std(&x, VarNorm::Sample, NanFlag::Omit));
        assert!(
            (b.lower - (m - t * sd)).abs() <= 1e-13
                && (b.upper - (m + t * sd)).abs() <= 1e-13
                && b.center == m,
            "{b:?}"
        );
        let (_, b) = isoutlier(&x, OutlierMethod::Quartiles(t)).unwrap();
        let q = quantile(&x, &[0.25, 0.75], QuantileType::default(), NanFlag::Omit).unwrap();
        let iq = q[1] - q[0];
        assert!(
            (b.lower - (q[0] - t * iq)).abs() <= 1e-13 && (b.upper - (q[1] + t * iq)).abs() <= 1e-13,
            "{b:?}"
        );
        assert!((b.center - (q[0] + q[1]) / 2.0).abs() <= 1e-15, "{b:?}");
    }
    // Escalas por MAD e IQR no normalize.
    let z = normalize(&x, NormalizeMethod::Scale(ScaleBy::Mad)).unwrap();
    assert!((z[6] - 40.0 / (mad_normal * mad)).abs() <= 1e-13, "{}", z[6]);
    let q = quantile(&x, &[0.25, 0.75], QuantileType::default(), NanFlag::Omit).unwrap();
    let z = normalize(&x, NormalizeMethod::Scale(ScaleBy::Iqr)).unwrap();
    assert!((z[6] - 40.0 / (q[1] - q[0])).abs() <= 1e-13, "{}", z[6]);
    // Norma p, inclusive p = 1 e valores enormes (escala pelo maior |x| antes de elevar).
    for (p, nrm) in [(1.0, 7.0), (2.0, 5.0), (3.0, (27.0f64 + 64.0).cbrt())] {
        for e in [1.0, 1e200] {
            let z = normalize(&[3.0 * e, -4.0 * e], NormalizeMethod::Norm(p)).unwrap();
            assert!(
                (z[0] - 3.0 / nrm).abs() <= 1e-15 && (z[1] + 4.0 / nrm).abs() <= 1e-15,
                "p={p} e={e}: {z:?}"
            );
        }
    }
    // Reescala de dados constantes: ponto médio.
    assert_eq!(rescale(&[7.0, 7.0], 2.0, 6.0).unwrap(), vec![4.0, 4.0]);
    // Preenchimento: linear entre vizinhos (dois outliers seguidos), mais próximo com empate
    // (vence o anterior) e recorte no limite correto.
    let y = [1.0, 2.0, 100.0, 200.0, 5.0, 6.0, 7.0, 8.0, -90.0, 9.0];
    let (f, m) = filloutliers(&y, FillMethod::Linear, OutlierMethod::Median(3.0)).unwrap();
    assert_eq!(
        m,
        vec![false, false, true, true, false, false, false, false, true, false]
    );
    assert!(
        (f[2] - 3.0).abs() <= 1e-15 && (f[3] - 4.0).abs() <= 1e-15 && (f[8] - 8.5).abs() <= 1e-15,
        "{f:?}"
    );
    let (f, _) = filloutliers(&y, FillMethod::Nearest, OutlierMethod::Median(3.0)).unwrap();
    assert_eq!((f[2], f[3], f[8]), (2.0, 5.0, 8.0));
    let (f, _) = filloutliers(&y, FillMethod::Clip, OutlierMethod::Median(3.0)).unwrap();
    let (_, b) = isoutlier(&y, OutlierMethod::Median(3.0)).unwrap();
    assert_eq!((f[2], f[8]), (b.upper, b.lower));
}

/// LOWESS/LOESS de referência (W. S. Cleveland, JASA 74, 1979), escrito à parte por força bruta:
/// vizinhança de q pontos pela janela contígua de menor alcance máximo (empate: a mais à
/// esquerda), pesos tricúbicos com alcance ·1,000001, mínimos quadrados ponderados pelas equações
/// normais em t − t₀ e, no robusto, 4 reponderações bi-quadradas com escala 6·mediana|r|.
fn loess_referencia(y: &[f64], span: f64, grau: usize, robusto: bool) -> Vec<f64> {
    let n = y.len();
    let q = ((span * n as f64).ceil() as usize).clamp((grau + 1).min(n), n);
    let mut rob = vec![1.0; n];
    let mut fit = vec![0.0; n];
    for it in 0..if robusto { 5 } else { 1 } {
        for (i, fi) in fit.iter_mut().enumerate() {
            let alcance = |c: usize| (i as f64 - c as f64).abs().max((c + q - 1) as f64 - i as f64);
            let l = (0..=n - q).fold(0, |m, c| if alcance(c) < alcance(m) { c } else { m });
            let dmax = (l..l + q)
                .map(|j| (j as f64 - i as f64).abs())
                .fold(0.0, f64::max)
                * 1.000_001;
            let p = grau.min(q - 1) + 1;
            let (mut a, mut b) = (vec![vec![0.0; p]; p], vec![0.0; p]);
            for j in l..l + q {
                let u = if dmax == 0.0 {
                    0.0
                } else {
                    (j as f64 - i as f64).abs() / dmax
                };
                let w = (1.0 - u * u * u).powi(3) * rob[j];
                let d = j as f64 - i as f64;
                let pot: Vec<f64> = (0..2 * p).map(|k| d.powi(k as i32)).collect();
                for r in 0..p {
                    b[r] += w * pot[r] * y[j];
                    for c in 0..p {
                        a[r][c] += w * pot[r + c];
                    }
                }
            }
            // Eliminação de Gauss; o valor ajustado em t₀ é o coeficiente constante.
            for k in 0..p {
                for r in k + 1..p {
                    let f = a[r][k] / a[k][k];
                    for c in k..p {
                        a[r][c] -= f * a[k][c];
                    }
                    b[r] -= f * b[k];
                }
            }
            let mut x = vec![0.0; p];
            for r in (0..p).rev() {
                x[r] = (b[r] - (r + 1..p).map(|c| a[r][c] * x[c]).sum::<f64>()) / a[r][r];
            }
            *fi = x[0];
        }
        if it < 4 && robusto {
            let mut res: Vec<f64> = (0..n).map(|i| (y[i] - fit[i]).abs()).collect();
            let r0 = res.clone();
            res.sort_by(f64::total_cmp);
            let med = if n % 2 == 1 {
                res[n / 2]
            } else {
                0.5 * (res[n / 2 - 1] + res[n / 2])
            };
            let s = 6.0 * med;
            for i in 0..n {
                rob[i] = (1.0 - (r0[i] / s).min(1.0).powi(2)).powi(2);
            }
        }
    }
    fit
}

/// smoothdata contra oráculos (mutação): resposta ao impulso do gaussiano (pesos
/// exp(−½(d/σ)²) normalizados, σ = k/5), coeficientes publicados do Savitzky–Golay (janela 5,
/// grau 2: (−3, 12, 17, 12, −3)/35; A. Savitzky & M. Golay, Anal. Chem. 36, 1964) e reprodução de
/// polinômios nas bordas, e LOWESS/LOESS contra a referência acima.
#[test]
fn suavizacao_contra_oraculos() {
    use chimalab_math::stats::{SmoothMethod, smoothdata};
    let mut imp = vec![0.0; 21];
    imp[10] = 1.0;
    // Gaussiano, k = 5 (meia janela 2, σ = 1).
    let g = smoothdata(&imp, SmoothMethod::Gaussian(5)).unwrap();
    let w: Vec<f64> = (-2..=2).map(|d: i32| (-0.5 * f64::from(d * d)).exp()).collect();
    let sw: f64 = w.iter().sum();
    for (d, wd) in (-2..=2).zip(&w) {
        let i = (10 + d) as usize;
        assert!((g[i] - wd / sw).abs() <= 1e-15, "gauss {d}: {}", g[i]);
    }
    assert_eq!((g[7], g[13]), (0.0, 0.0));
    // Savitzky–Golay, janela 5, grau 2.
    let sg = smoothdata(&imp, SmoothMethod::SavitzkyGolay { window: 5, degree: 2 }).unwrap();
    for (d, c) in (-2..=2).zip([-3.0, 12.0, 17.0, 12.0, -3.0]) {
        let i = (10 - d) as usize;
        assert!((sg[i] - c / 35.0).abs() <= 1e-14, "sg {d}: {}", sg[i]);
    }
    // Reprodução de polinômios de grau ≤ grau, inclusive nas bordas (janela deslocada).
    let cub: Vec<f64> = (0..15)
        .map(|i| {
            let t = i as f64;
            1.0 - 2.0 * t + 0.3 * t * t - 0.02 * t * t * t
        })
        .collect();
    let s3 = smoothdata(&cub, SmoothMethod::SavitzkyGolay { window: 7, degree: 3 }).unwrap();
    assert!(s3.iter().zip(&cub).all(|(a, b)| (a - b).abs() <= 1e-11), "{s3:?}");
    // LOWESS/LOESS, robustos ou não, com outliers, contra a referência.
    let mut y: Vec<f64> = (0..40)
        .map(|i| {
            let t = i as f64 / 4.0;
            t.sin() + 0.1 * t
        })
        .collect();
    y[7] += 3.0;
    y[25] -= 2.5;
    for (span, grau, robusto) in [
        (0.3, 1, false),
        (0.3, 1, true),
        (0.5, 2, false),
        (0.25, 2, true),
        (1.0, 1, true),
    ] {
        let m = if grau == 1 {
            SmoothMethod::Lowess {
                span,
                robust: robusto,
            }
        } else {
            SmoothMethod::Loess {
                span,
                robust: robusto,
            }
        };
        let s = smoothdata(&y, m).unwrap();
        let r = loess_referencia(&y, span, grau, robusto);
        for i in 0..y.len() {
            assert!((s[i] - r[i]).abs() <= 1e-10, "{m:?} i={i}: {} vs {}", s[i], r[i]);
        }
    }
}

/// Extremos locais nas bordas e nos limiares (mutação) e GESD contra o valor crítico tabelado de
/// Grubbs (n = 10, α = 0,05 bilateral: G = 2,290; tabelas de F. E. Grubbs, Technometrics 11,
/// 1969). A estatística G = max|x − x̄|/s é calculada aqui; dados com G logo abaixo e logo acima
/// do limite.
#[test]
fn extremos_e_gesd_nas_bordas() {
    use chimalab_math::stats::{gesd, local_extrema};
    // Pico no penúltimo índice (base: o maior dos mínimos dos dois lados, max(0, 1) = 1) e platô
    // que chega ao fim (não é pico).
    assert_eq!(
        local_extrema(&[0.0, 1.0, 0.0, 2.0, 1.0], false, 0.0, 0),
        vec![(1, 1.0), (3, 1.0)]
    );
    assert_eq!(
        local_extrema(&[0.0, 1.0, 0.0, 2.0, 2.0], false, 0.0, 0),
        vec![(1, 1.0)]
    );
    assert_eq!(
        local_extrema(&[0.0, 3.0, 3.0, 3.0, 1.0], false, 0.0, 0),
        vec![(2, 2.0)]
    );
    // Proeminência igual ao mínimo entra; separação igual à mínima é permitida.
    let y = [0.0, 2.0, 1.0, 3.0, 0.0, 1.5, 0.0];
    assert_eq!(
        local_extrema(&y, false, 1.0, 0),
        vec![(1, 1.0), (3, 3.0), (5, 1.5)]
    );
    assert_eq!(local_extrema(&y, false, 1.5, 0), vec![(3, 3.0), (5, 1.5)]);
    assert_eq!(
        local_extrema(&y, false, 0.0, 2),
        vec![(1, 1.0), (3, 3.0), (5, 1.5)]
    );
    assert_eq!(local_extrema(&y, false, 0.0, 3), vec![(3, 3.0)]);
    // Mínimos: espelho.
    let neg: Vec<f64> = y.iter().map(|v| -v).collect();
    assert_eq!(local_extrema(&neg, true, 1.5, 0), vec![(3, 3.0), (5, 1.5)]);
    // GESD/Grubbs: 9 valores fixos e um que se afasta.
    let base = [10.0, 10.4, 9.7, 10.1, 9.9, 10.3, 9.8, 10.2, 10.0];
    let g = |v: f64| {
        let mut x = base.to_vec();
        x.push(v);
        let m = x.iter().sum::<f64>() / 10.0;
        let s = (x.iter().map(|a| (a - m) * (a - m)).sum::<f64>() / 9.0).sqrt();
        x.iter().map(|a| (a - m).abs()).fold(0.0, f64::max) / s
    };
    // v com G = alvo, por bissecção (G cresce com v acima da média).
    let com_g = |alvo: f64| {
        let (mut lo, mut hi) = (10.5, 100.0);
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if g(mid) < alvo { lo = mid } else { hi = mid }
        }
        lo
    };
    for (alvo, outlier) in [(2.27, false), (2.31, true), (2.6, true), (1.9, false)] {
        let mut x = base.to_vec();
        x.push(com_g(alvo));
        let m = gesd(&x, 1, 0.05).unwrap();
        assert_eq!(m[9], outlier, "G = {alvo}");
        assert!(m[..9].iter().all(|b| !b));
    }
    // Três pontos ainda são testados (G máximo 2/√3 ≈ 1,155 contra o crítico ≈ 1,115 com
    // α = 0,5); dois não. α fora de (0, 1) é erro.
    assert_eq!(
        gesd(&[0.0, 0.0, 1000.0], 1, 0.5).unwrap(),
        vec![false, false, true]
    );
    assert_eq!(gesd(&[1.0, 2.0], 1, 0.05).unwrap(), vec![false, false]);
    for a in [0.0, 1.0] {
        assert_eq!(
            gesd(&base, 1, a).unwrap_err().id(),
            "chimalab:stats:opcaoInvalida"
        );
    }
}

/// Moda, quantil com infinitos, somas compensadas e estatísticas ponderadas (mutação).
#[test]
fn moda_somas_compensadas_e_ponderadas() {
    use chimalab_math::stats::{WeightNorm, median_weighted, var_weighted};
    // Moda: não necessariamente o menor; empate fica com o menor; grupos crescentes.
    assert_eq!(mode(&[1.0, 2.0, 2.0]), 2.0);
    assert_eq!(mode(&[1.0, 2.0, 2.0, 3.0, 3.0, 3.0]), 3.0);
    assert_eq!(mode(&[3.0, 1.0, 3.0, 1.0, 2.0]), 1.0);
    assert_eq!(mode(&[5.0, 4.0, 4.0, 9.0, 9.0, 9.0, 4.0, 4.0]), 4.0);
    // Quantil entre dois infinitos iguais: ∞ (sem ∞ − ∞).
    let q = quantile(
        &[1.0, f64::INFINITY, f64::INFINITY],
        &[0.75],
        QuantileType::default(),
        NanFlag::Omit,
    )
    .unwrap();
    assert_eq!(q[0], f64::INFINITY);
    // Somas compensadas: com cancelamento catastrófico, o resultado é a soma exata arredondada
    // uma só vez (conferido com frações exatas em Python; 1e16 + 2 é representável, 1e16 + 3
    // arredonda para 1e16 + 4).
    let x = [1e16, 1.0, -1e16, 1.0, 1e16, 1.0, -1e16, 1.0];
    assert_eq!(
        cumsum(&x, NanFlag::Include),
        vec![
            1e16,
            1e16,
            1.0,
            2.0,
            1.000_000_000_000_000_2e16,
            1.000_000_000_000_000_4e16,
            3.0,
            4.0
        ]
    );
    let m = moving(
        &x,
        Window::Centered(3),
        MovingStat::Sum,
        Endpoints::Shrink,
        NanFlag::Include,
    )
    .unwrap();
    assert_eq!(
        m,
        vec![
            1e16,
            1.0,
            -9_999_999_999_999_998.0,
            1.0,
            1.000_000_000_000_000_2e16,
            1.0,
            -9_999_999_999_999_998.0,
            -1e16
        ]
    );
    // ±∞ que entra e sai da janela: depois de sair, a soma volta a ser finita.
    let y = [1.0, 2.0, f64::INFINITY, 3.0, 4.0, 5.0, 6.0];
    let m = moving(
        &y,
        Window::Asymmetric(1, 0),
        MovingStat::Sum,
        Endpoints::Shrink,
        NanFlag::Include,
    )
    .unwrap();
    assert_eq!(m, vec![1.0, 3.0, f64::INFINITY, f64::INFINITY, 7.0, 9.0, 11.0]);
    let y = [1.0, f64::NEG_INFINITY, f64::INFINITY, 3.0, 4.0, 5.0];
    let m = moving(
        &y,
        Window::Asymmetric(1, 0),
        MovingStat::Sum,
        Endpoints::Shrink,
        NanFlag::Include,
    )
    .unwrap();
    assert!(m[2].is_nan());
    assert_eq!(&m[3..], &[f64::INFINITY, 7.0, 9.0]);
    // Variância ponderada por frequência com Σw = 1: denominador nulo, NaN (não ∞).
    assert!(
        var_weighted(&[1.0, 3.0], &[0.5, 0.5], WeightNorm::Frequency, NanFlag::Include)
            .unwrap()
            .is_nan()
    );
    // Mediana ponderada contra a mediana dos dados replicados (pesos inteiros).
    let mut s = 5u64;
    let mut u = || {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (s >> 33) as usize
    };
    for caso in 0..300 {
        let n = 1 + caso % 7;
        let xs: Vec<f64> = (0..n).map(|_| (u() % 20) as f64 - 10.0).collect();
        let ws: Vec<f64> = (0..n).map(|_| (u() % 4) as f64).collect();
        if ws.iter().all(|&w| w == 0.0) {
            continue;
        }
        let rep: Vec<f64> = xs
            .iter()
            .zip(&ws)
            .flat_map(|(&x, &w)| std::iter::repeat_n(x, w as usize))
            .collect();
        let esp = median(&rep, NanFlag::Include);
        let got = median_weighted(&xs, &ws, NanFlag::Include).unwrap();
        assert_eq!(got, esp, "caso {caso}: x {xs:?} w {ws:?}");
    }
    // Pesos fracionários com soma acumulada exatamente na metade (0,1 × 10).
    let xs: Vec<f64> = (1..=10).map(f64::from).collect();
    assert_eq!(median_weighted(&xs, &[0.1; 10], NanFlag::Include).unwrap(), 5.5);
}

/// `along_dim` em 3-D ao longo da dimensão do meio (passo e blocos externos > 1), com funções
/// que mudam o tamanho do eixo, contra o cálculo por índices; erro de dimensão com os tamanhos.
/// Quantil com posição a poucos ε de um inteiro: o ajuste evita interpolar com o vizinho
/// (mutação).
#[test]
fn along_dim_3d_e_ajuste_do_quantil() {
    let shape = [2usize, 3, 2];
    let a: Vec<f64> = (0..12).map(|k| (k * k % 7) as f64 + 0.5 * k as f64).collect();
    let at = |i: usize, j: usize, l: usize| a[i + 2 * (j + 3 * l)];
    // Soma ao longo da dimensão 2 (índice 1): forma 2×1×2.
    let (s, forma) = along_dim(&a, &shape, 1, |v| vec![v.iter().sum()]).unwrap();
    assert_eq!(forma, vec![2, 1, 2]);
    for l in 0..2 {
        for i in 0..2 {
            let esp: f64 = (0..3).map(|j| at(i, j, l)).sum();
            assert_eq!(s[i + 2 * l], esp, "i={i} l={l}");
        }
    }
    // Primeiro e último ao longo da dimensão 2: forma 2×2×2.
    let (r, forma) = along_dim(&a, &shape, 1, |v| vec![v[0], v[v.len() - 1]]).unwrap();
    assert_eq!(forma, vec![2, 2, 2]);
    for l in 0..2 {
        for i in 0..2 {
            assert_eq!(
                (r[i + 2 * (2 * l)], r[i + 2 * (1 + 2 * l)]),
                (at(i, 0, l), at(i, 2, l))
            );
        }
    }
    match along_dim(&a, &shape, 5, |v| v.to_vec()) {
        Err(StatsError::ShapeMismatch { expected, actual }) => assert_eq!((expected, actual), (3, 6)),
        o => panic!("{o:?}"),
    }
    // Tipo 4 (h = n·p) com n = 25 e p = 0,28: em ponto flutuante h = 7,000000000000001, a poucos
    // ε do inteiro 7 (achado por busca). Sem o ajuste, o peso 9e-16 do vizinho 8e17 somaria ~700.
    let x: Vec<f64> = (1..=25)
        .map(|i| if i <= 7 { i as f64 } else { i as f64 * 1e17 })
        .collect();
    let q = quantile(&x, &[0.28], QuantileType::new(4).unwrap(), NanFlag::Include).unwrap();
    assert_eq!(q, vec![7.0]);
}

/// Preenchimento pelo mais próximo com distâncias em que diferença e razão discordam
/// (mutação): outliers em 6..=12 com vizinhos bons em 5 e 13 (no índice 10: 5 contra 3,
/// vence o seguinte), e outliers em 2..=7 com bons em 1 e 8 (no índice 4: 3 contra 4, vence o
/// anterior).
#[test]
fn preenchimento_mais_proximo_distancias() {
    let serie = |ruins: std::ops::RangeInclusive<usize>| -> Vec<f64> {
        (0..20)
            .map(|i| {
                if ruins.contains(&i) {
                    1000.0
                } else {
                    1.0 + 0.01 * i as f64
                }
            })
            .collect()
    };
    let y = serie(6..=12);
    let (f, m) = filloutliers(&y, FillMethod::Nearest, OutlierMethod::Median(3.0)).unwrap();
    assert!(m[6..=12].iter().all(|&b| b) && !m[5] && !m[13]);
    assert_eq!((f[10], f[8], f[9]), (y[13], y[5], y[5]));
    let y = serie(2..=7);
    let (f, _) = filloutliers(&y, FillMethod::Nearest, OutlierMethod::Median(3.0)).unwrap();
    assert_eq!((f[4], f[5]), (y[1], y[8]));
}

//! F053: números aleatórios. Oráculos:
//! - KAT oficiais do Random123 (tests/dados/F053/kat_random123.txt) para Philox4x32-10 e
//!   Threefry4x64-20;
//! - MT19937: 10 000º valor com semente 5489 = 4123659995 (exigido pelo padrão C++ para
//!   `std::mt19937`) e as saídas do gerador em C do módulo `random` do Python
//!   (tests/dados/F053/geradores.csv);
//! - MRG32k3a e xoshiro256++: aritmética inteira exata em Python a partir dos artigos;
//! - saltos de matriz do MRG32k3a conferidos contra passos sucessivos;
//! - testes estatísticos com valores-p por `gammainc` e `erfc` (F042).

use chimalab_math::random::*;
use chimalab_math::special::{erfc, gammainc};

const TODOS: [Algorithm; 5] = [
    Algorithm::Philox,
    Algorithm::Threefry,
    Algorithm::Twister,
    Algorithm::Mrg32k3a,
    Algorithm::Xoshiro256pp,
];

fn hex32(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn hex64(s: &str) -> u64 {
    u64::from_str_radix(s, 16).unwrap()
}

#[test]
fn kat_random123() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F053/kat_random123.txt"
    );
    let mut n = 0;
    for l in std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let w: Vec<&str> = l.split_whitespace().collect();
        match w[0] {
            "philox4x32" => {
                let c = [hex32(w[2]), hex32(w[3]), hex32(w[4]), hex32(w[5])];
                let k = [hex32(w[6]), hex32(w[7])];
                let want = [hex32(w[8]), hex32(w[9]), hex32(w[10]), hex32(w[11])];
                assert_eq!(raw::philox4x32_10(c, k), want, "{l}");
            }
            "threefry4x64" => {
                let c = [hex64(w[2]), hex64(w[3]), hex64(w[4]), hex64(w[5])];
                let k = [hex64(w[6]), hex64(w[7]), hex64(w[8]), hex64(w[9])];
                let want = [hex64(w[10]), hex64(w[11]), hex64(w[12]), hex64(w[13])];
                assert_eq!(raw::threefry4x64_20(c, k), want, "{l}");
            }
            _ => unreachable!(),
        }
        n += 1;
    }
    assert_eq!(n, 6);
}

#[test]
fn mt19937_contra_padrao_cpp_e_python() {
    // Padrão C++ ([rand.predef]): o 10 000º valor de mt19937 com a semente padrão 5489.
    assert_eq!(raw::mt19937_u32(5489, 10_000)[9_999], 4_123_659_995);
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/dados/F053/geradores.csv"
    );
    let txt = std::fs::read_to_string(path).unwrap();
    let mut n = 0;
    for l in txt
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("gerador"))
    {
        let c: Vec<&str> = l.split(',').collect();
        let (seed, i, v): (u64, usize, u64) = (
            c[1].parse().unwrap(),
            c[2].parse().unwrap(),
            c[3].parse().unwrap(),
        );
        match c[0] {
            "twister" => {
                let mut r = Rng::new(Algorithm::Twister, seed);
                let got = (0..=i).map(|_| r.next_u32()).last().unwrap();
                assert_eq!(u64::from(got), v, "twister {seed} {i}");
            }
            "mrg32k3a" => {
                let mut r = Rng::new(Algorithm::Mrg32k3a, seed);
                // next_u32 do MRG devolve z ∈ [0, m₁) diretamente.
                let got = (0..=i).map(|_| r.next_u32()).last().unwrap();
                assert_eq!(u64::from(got), v, "mrg {seed} {i}");
            }
            "xoshiro256pp" => {
                let mut r = Rng::new(Algorithm::Xoshiro256pp, seed);
                let got = (0..=i).map(|_| r.next_u64()).last().unwrap();
                assert_eq!(got, v, "xoshiro {seed} {i}");
            }
            _ => unreachable!(),
        }
        n += 1;
    }
    assert!(n > 250);
}

#[test]
fn ca1_semente_reprodutivel() {
    for alg in TODOS {
        let a: Vec<u64> = {
            let mut r = Rng::new(alg, 42);
            (0..100).map(|_| r.next_u64()).collect()
        };
        let mut r = Rng::new(alg, 42);
        let b: Vec<u64> = (0..100).map(|_| r.next_u64()).collect();
        assert_eq!(a, b, "{alg:?}");
        let mut r = Rng::new(alg, 43);
        let c: Vec<u64> = (0..100).map(|_| r.next_u64()).collect();
        assert_ne!(a, c, "{alg:?}");
        // Distribuições também reprodutíveis.
        let (mut x, mut y) = (Rng::new(alg, 7), Rng::new(alg, 7));
        assert_eq!(x.normals(50), y.normals(50));
        assert_eq!(x.permutation(30), y.permutation(30));
    }
    assert_eq!(Rng::default().algorithm(), Algorithm::Philox);
    assert_eq!(Rng::default(), Rng::new(Algorithm::Philox, 0));
}

#[test]
fn ca3_fluxos_e_subfluxos_nao_se_sobrepoem() {
    // Philox e Threefry: contadores disjuntos por construção; confere que 4 fluxos não repetem
    // nenhuma palavra de 64 bits entre si em 2000 saídas.
    for alg in [
        Algorithm::Philox,
        Algorithm::Threefry,
        Algorithm::Mrg32k3a,
        Algorithm::Xoshiro256pp,
    ] {
        let mut todos = std::collections::HashSet::new();
        for s in 0..4 {
            let mut r = Rng::with_stream(alg, 9, s, 0).unwrap();
            for _ in 0..2000 {
                assert!(todos.insert(r.next_u64()), "{alg:?}: repetição entre fluxos");
            }
        }
    }
    // MRG32k3a: o salto de matriz coincide com passos sucessivos (A^k), base dos saltos de
    // 2⁷⁶ e 2¹²⁷.
    for k in [1u64, 2, 3, 7, 64, 1000] {
        let (passo, salto) = raw::mrg32k3a_step_vs_jump(5, k);
        assert_eq!(passo, salto, "k = {k}");
        // E coincide com o gerador público avançado k passos.
        let mut r = Rng::new(Algorithm::Mrg32k3a, 5);
        for _ in 0..k {
            r.next_u32();
        }
        assert_eq!(r.state().words, passo.to_vec(), "k = {k}");
    }
    // Subfluxos distintos e fluxo 0/subfluxo 0 igual ao gerador simples.
    let mut a = Rng::with_stream(Algorithm::Mrg32k3a, 3, 0, 0).unwrap();
    let mut b = Rng::new(Algorithm::Mrg32k3a, 3);
    assert_eq!(
        (0..10).map(|_| a.next_u32()).collect::<Vec<_>>(),
        (0..10).map(|_| b.next_u32()).collect::<Vec<_>>()
    );
    let mut s1 = Rng::with_stream(Algorithm::Mrg32k3a, 3, 0, 1).unwrap();
    let mut s2 = Rng::with_stream(Algorithm::Mrg32k3a, 3, 0, 2).unwrap();
    assert_ne!(s1.next_u64(), s2.next_u64());
    let mut p0 = Rng::with_stream(Algorithm::Philox, 3, 0, 0).unwrap();
    assert_eq!(p0.next_u64(), Rng::new(Algorithm::Philox, 3).next_u64());
}

#[test]
fn erros_com_identificador() {
    assert_eq!(
        Algorithm::from_name("mt").unwrap_err().id(),
        "chimalab:random:geradorDesconhecido"
    );
    for alg in TODOS {
        assert_eq!(Algorithm::from_name(alg.name()).unwrap(), alg);
    }
    let e = Rng::with_stream(Algorithm::Twister, 1, 1, 0).unwrap_err();
    assert_eq!(e.id(), "chimalab:random:argumentoInvalido");
    assert!(Rng::with_stream(Algorithm::Xoshiro256pp, 1, 0, 1).is_err());
    assert!(Rng::with_stream(Algorithm::Philox, 1, 0, 1).is_err());
    assert!(Rng::with_stream(Algorithm::Threefry, 1, 2, 3).is_err());
    let mut r = Rng::default();
    assert_eq!(
        r.integer(5, 4).unwrap_err().id(),
        "chimalab:random:argumentoInvalido"
    );
    assert_eq!(
        r.sample(3, 4).unwrap_err().id(),
        "chimalab:random:argumentoInvalido"
    );
    let st = |algorithm, words: Vec<u64>| RngState { algorithm, words };
    let m1 = 4_294_967_087u64;
    let m2 = 4_294_944_443u64;
    for bad in [
        st(Algorithm::Philox, vec![1, 2, 3, 4]),
        st(Algorithm::Philox, vec![1, 2, 3, 4, 0]),
        st(Algorithm::Philox, vec![1, 2, 3, 4, 5]),
        st(Algorithm::Philox, vec![1 << 32, 2, 3, 4, 1]),
        st(Algorithm::Philox, vec![1, 1 << 32, 3, 4, 1]),
        st(Algorithm::Threefry, vec![1, 2, 3, 0, 0, 0]),
        st(Algorithm::Threefry, vec![1, 2, 3, 5, 0, 0]),
        st(Algorithm::Threefry, vec![1, 2, 3, 2, 2, 0]),
        st(Algorithm::Threefry, vec![1, 2, 3, 2, 0, 7]),
        st(Algorithm::Threefry, vec![1, 2, 3, 2, 1, 1 << 32]),
        st(Algorithm::Twister, vec![0; 10]),
        st(
            Algorithm::Twister,
            [vec![1u64 << 32], vec![0; 623], vec![0]].concat(),
        ),
        st(Algorithm::Twister, [vec![0; 624], vec![625]].concat()),
        st(Algorithm::Mrg32k3a, vec![0, 0, 0, 1, 1, 1]),
        st(Algorithm::Mrg32k3a, vec![1, 1, 1, 0, 0, 0]),
        st(Algorithm::Mrg32k3a, vec![m1, 1, 1, 1, 1, 1]),
        st(Algorithm::Mrg32k3a, vec![1, 1, m1, 1, 1, 1]),
        st(Algorithm::Mrg32k3a, vec![1, 1, 1, m2, 1, 1]),
        st(Algorithm::Mrg32k3a, vec![1, 1, 1, 1, 1, m2]),
        st(Algorithm::Xoshiro256pp, vec![0, 0, 0, 0, 0, 0]),
        st(Algorithm::Xoshiro256pp, vec![1, 0, 0, 0, 0]),
        st(Algorithm::Xoshiro256pp, vec![1, 0, 0, 0, 0, 5]),
    ] {
        assert_eq!(
            Rng::from_state(&bad).unwrap_err(),
            RandomError::InvalidState,
            "{bad:?}"
        );
    }
    assert!(RandomError::InvalidArgument("x").to_string().contains('x'));
    assert!(!RandomError::UnknownGenerator.to_string().is_empty());
    assert!(!RandomError::InvalidState.to_string().is_empty());
    assert_eq!(RandomError::InvalidState.id(), "chimalab:random:estadoInvalido");
}

#[test]
fn uniforme_inteiros_e_permutacoes() {
    for alg in TODOS {
        let mut r = Rng::new(alg, 2024);
        let u = r.uniforms(100_000);
        assert!(u.iter().all(|&x| x > 0.0 && x < 1.0), "{alg:?}");
        let m = u.iter().sum::<f64>() / u.len() as f64;
        let v = u.iter().map(|x| (x - m).powi(2)).sum::<f64>() / u.len() as f64;
        // σ da média = √(1/12/n) ≈ 9e-4: 5σ.
        assert!(
            (m - 0.5).abs() < 5e-3 && (v - 1.0 / 12.0).abs() < 2e-3,
            "{alg:?}: {m} {v}"
        );
        // Inteiros: limites inclusivos atingidos, faixa respeitada, intervalo de um ponto.
        let xs: Vec<i64> = (0..10_000).map(|_| r.integer(-3, 3).unwrap()).collect();
        assert!(xs.iter().all(|&x| (-3..=3).contains(&x)));
        assert!(xs.contains(&-3) && xs.contains(&3));
        assert_eq!(r.integer(7, 7).unwrap(), 7);
        let _ = r.integer(i64::MIN, i64::MAX).unwrap();
        // Permutações e amostras.
        let mut p = r.permutation(50);
        p.sort_unstable();
        assert_eq!(p, (0..50).collect::<Vec<_>>());
        let s = r.sample(1000, 10).unwrap();
        let mut d = s.clone();
        d.sort_unstable();
        d.dedup();
        assert_eq!((s.len(), d.len()), (10, 10));
        assert!(s.iter().all(|&x| x < 1000));
        assert!(r.sample(5, 0).unwrap().is_empty() && r.permutation(0).is_empty());
        assert_eq!(r.sample(4, 4).unwrap().len(), 4);
    }
}

/// Valor-p do qui-quadrado com `gl` graus de liberdade: Q(gl/2, χ²/2).
fn p_qui2(chi2: f64, gl: f64) -> f64 {
    gammainc(chi2 / 2.0, gl / 2.0, true)
}

/// Bateria própria (sementes fixas, sem instabilidade): todos os valores-p em [1e-4, 1 − 1e-4].
#[test]
fn bateria_estatistica_propria() {
    for alg in TODOS {
        let mut r = Rng::new(alg, 31337);
        // 1. Uniformidade: 1000 caixas, 10⁶ amostras.
        let n = 1_000_000;
        let mut cx = vec![0u32; 1000];
        for _ in 0..n {
            cx[(r.uniform() * 1000.0) as usize] += 1;
        }
        let e = n as f64 / 1000.0;
        let chi2: f64 = cx.iter().map(|&o| (o as f64 - e).powi(2) / e).sum();
        let p1 = p_qui2(chi2, 999.0);
        // 2. Pares (x, y) em grade 32×32: independência serial.
        let mut cp = vec![0u32; 1024];
        for _ in 0..n / 2 {
            let (a, b) = ((r.uniform() * 32.0) as usize, (r.uniform() * 32.0) as usize);
            cp[a * 32 + b] += 1;
        }
        let e2 = (n / 2) as f64 / 1024.0;
        let p2 = p_qui2(cp.iter().map(|&o| (o as f64 - e2).powi(2) / e2).sum(), 1023.0);
        // 3. Inteiros sem viés em [0, 6]: 7 caixas.
        let mut ci = [0u32; 7];
        for _ in 0..700_000 {
            ci[r.integer(0, 6).unwrap() as usize] += 1;
        }
        let p3 = p_qui2(ci.iter().map(|&o| (o as f64 - 1e5).powi(2) / 1e5).sum(), 6.0);
        // 4. Normal: Kolmogorov–Smirnov contra Φ com 10⁵ amostras.
        let mut z = r.normals(100_000);
        z.sort_by(f64::total_cmp);
        let nn = z.len() as f64;
        let d = z
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let phi = 0.5 * erfc(-x / std::f64::consts::SQRT_2);
                (phi - i as f64 / nn).abs().max(((i + 1) as f64 / nn - phi).abs())
            })
            .fold(0.0, f64::max);
        // Valor-p assintótico de KS: 2Σ(−1)^{k−1}e^{−2k²λ²}, λ = (√n + 0,12 + 0,11/√n)·D.
        let lam = (nn.sqrt() + 0.12 + 0.11 / nn.sqrt()) * d;
        let p4: f64 = (1..100)
            .map(|k| 2.0 * (-1f64).powi(k - 1) * (-2.0 * (k * k) as f64 * lam * lam).exp())
            .sum();
        // 5. Cauda da normal: P(|Z| > 4) = 6,334e-5; 2·10⁶ amostras → esperado 126,7.
        let cauda = (0..2_000_000).filter(|_| r.normal().abs() > 4.0).count() as f64;
        let esp = 2e6 * erfc(4.0 / std::f64::consts::SQRT_2);
        let p5 = erfc((cauda - esp).abs() / esp.sqrt() / std::f64::consts::SQRT_2);
        for (nome, p) in [
            ("uniforme", p1),
            ("pares", p2),
            ("inteiros", p3),
            ("ks normal", p4),
            ("cauda", p5),
        ] {
            eprintln!("{alg:?} {nome}: p = {p:.4}");
            assert!(
                (1e-4..=1.0 - 1e-4).contains(&p) || nome == "cauda" && p > 1e-4,
                "{alg:?} {nome}: p = {p}"
            );
        }
    }
}

mod propriedades {
    use super::*;
    // Importação explícita: o prelúdio do proptest também exporta um `Rng`.
    use chimalab_math::random::Rng;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        // Salvar e restaurar em qualquer posição, inclusive no meio de um bloco de Philox ou de
        // um u64 dividido em dois u32.
        #[test]
        fn estado_salvo_e_restaurado(alg in 0usize..5, seed in any::<u64>(), k in 0usize..40, m in 1usize..40, meio in any::<bool>()) {
            let mut r = Rng::new(TODOS[alg], seed);
            for _ in 0..k {
                r.next_u64();
            }
            if meio {
                r.next_u32();
            }
            let st = r.state();
            let a: Vec<u32> = (0..m).map(|_| r.next_u32()).collect();
            let mut r2 = Rng::from_state(&st).unwrap();
            prop_assert_eq!(r2.state(), st);
            let b: Vec<u32> = (0..m).map(|_| r2.next_u32()).collect();
            prop_assert_eq!(a, b);
        }

        // Inteiros em [lo, hi] para faixas quaisquer, inclusive enormes.
        #[test]
        fn inteiros_na_faixa(lo in any::<i64>(), w in any::<u64>(), seed in any::<u64>()) {
            let hi = (lo as i128 + (w >> 1) as i128).min(i64::MAX as i128) as i64;
            let mut r = Rng::new(Algorithm::Philox, seed);
            for _ in 0..20 {
                let x = r.integer(lo, hi).unwrap();
                prop_assert!(lo <= x && x <= hi);
            }
        }
    }
}

/// Medição: `cargo test --release -- --ignored desempenho_random --nocapture`.
#[test]
#[ignore]
fn desempenho_random() {
    for alg in TODOS {
        let mut r = Rng::new(alg, 1);
        let t = std::time::Instant::now();
        let mut acc = 0u64;
        for _ in 0..10_000_000 {
            acc ^= r.next_u64();
        }
        let tu = t.elapsed();
        let t = std::time::Instant::now();
        let s: f64 = (0..10_000_000).map(|_| r.normal()).sum();
        eprintln!(
            "{alg:?}: 10⁷ u64 em {tu:?}, 10⁷ normais em {:?} ({acc:x} {s:.1})",
            t.elapsed()
        );
    }
}

/// Regressão: com o estado do MRG32k3a perto de m₁ e m₂, a soma dos produtos passava de 2⁶⁴ (em
/// release dava a volta e o valor saía errado). Referência: a recorrência em aritmética exata.
#[test]
fn mrg32k3a_com_estado_no_limite() {
    let (m1, m2) = (4_294_967_087u128, 4_294_944_443u128);
    let s1 = [m1 - 1, m1 - 2, m1 - 3];
    let s2 = [m2 - 1, m2 - 2, m2 - 3];
    let words: Vec<u64> = s1.iter().chain(&s2).map(|&x| x as u64).collect();
    let mut r = Rng::from_state(&RngState {
        algorithm: Algorithm::Mrg32k3a,
        words,
    })
    .unwrap();
    let (mut a, mut b) = (s1, s2);
    for _ in 0..20 {
        let p1 = (1_403_580 * a[1] + (m1 - 810_728) * a[0]) % m1;
        a = [a[1], a[2], p1];
        let p2 = (527_612 * b[2] + (m2 - 1_370_589) * b[0]) % m2;
        b = [b[1], b[2], p2];
        let z = (p1 + m1 - p2) % m1;
        assert_eq!(u128::from(r.next_u32()), z);
    }
}

/// Fórmulas exatas das conversões, conferidas contra um clone do gerador.
#[test]
fn conversoes_exatas() {
    let escala = 1.0 / (1u64 << 53) as f64;
    for alg in TODOS {
        let mut r = Rng::new(alg, 99);
        for _ in 0..50 {
            let mut c = r.clone();
            let u = r.uniform();
            let want = if alg == Algorithm::Mrg32k3a {
                (f64::from(c.next_u32()) + 0.5) / 4_294_967_087.0
            } else {
                ((c.next_u64() >> 11) as f64 + 0.5) * escala
            };
            assert_eq!(u, want, "{alg:?}");
        }
        // u64 ↔ dois u32: geradores de 64 bits dão a metade baixa primeiro e guardam a alta;
        // os de 32 bits montam o u64 com a primeira palavra na parte alta.
        let mut c = r.clone();
        let (a, b) = (r.next_u32(), r.next_u32());
        let x = c.next_u64();
        match alg {
            Algorithm::Threefry | Algorithm::Xoshiro256pp => assert_eq!((a, b), (x as u32, (x >> 32) as u32)),
            _ => assert_eq!(x, (u64::from(a) << 32) | u64::from(b)),
        }
        // Depois de um u32 avulso, o u64 seguinte de um gerador de 64 bits usa o u32 guardado
        // (parte alta) e um u32 novo.
        if matches!(alg, Algorithm::Threefry | Algorithm::Xoshiro256pp) {
            let mut c = r.clone();
            let lo = r.next_u32();
            let y = r.next_u64();
            let full = c.next_u64();
            let next = c.next_u64();
            assert_eq!(lo, full as u32);
            assert_eq!(y, ((full >> 32) << 32) | (next & 0xffff_ffff));
        }
        assert_eq!(r.normals(37).len(), 37);
        assert_eq!(r.uniforms(5).len(), 5);
    }
}

/// Inteiros pelo método de Lemire: sem rejeição o valor é lo + ⌊x·s/2⁶⁴⌋; com s = 2⁶³ + 1 a
/// rejeição (parte baixa < 2⁶⁴ mod s = 2⁶³ − 1) ocorre em ~metade das tentativas e é exercida.
#[test]
fn inteiros_pelo_metodo_de_lemire() {
    let lemire = |c: &mut Rng, s: u64| -> u64 {
        let t = s.wrapping_neg() % s;
        loop {
            let m = u128::from(c.next_u64()) * u128::from(s);
            if (m as u64) >= t {
                return (m >> 64) as u64;
            }
        }
    };
    for (lo, hi) in [
        (0i64, 6i64),
        (-5, 5),
        (10, 10),
        (0, i64::MAX),
        (i64::MIN, -1),
        (-(1 << 62), 1 << 62),
    ] {
        let s = (hi as i128 - lo as i128 + 1) as u64;
        let mut r = Rng::new(Algorithm::Philox, 77);
        let mut c = r.clone();
        for _ in 0..200 {
            assert_eq!(
                r.integer(lo, hi).unwrap() as i128,
                lo as i128 + lemire(&mut c, s) as i128,
                "[{lo}, {hi}]"
            );
        }
    }
    // Faixa completa: o u64 inteiro, reinterpretado.
    let mut r = Rng::new(Algorithm::Philox, 3);
    let mut c = r.clone();
    for _ in 0..20 {
        assert_eq!(r.integer(i64::MIN, i64::MAX).unwrap(), c.next_u64() as i64);
    }
}

/// Fisher–Yates uniforme: as 6 permutações de 3 elementos com frequências iguais (qui-quadrado),
/// e as amostras de 2 entre 4 também (12 arranjos).
#[test]
fn permutacoes_uniformes() {
    let mut r = Rng::new(Algorithm::Philox, 5);
    let mut cont = std::collections::HashMap::new();
    for _ in 0..60_000 {
        *cont.entry(r.permutation(3)).or_insert(0u32) += 1;
    }
    assert_eq!(cont.len(), 6);
    let chi2: f64 = cont.values().map(|&o| (o as f64 - 1e4).powi(2) / 1e4).sum();
    assert!(p_qui2(chi2, 5.0) > 1e-4, "{cont:?}");
    let mut cont = std::collections::HashMap::new();
    for _ in 0..60_000 {
        *cont.entry(r.sample(4, 2).unwrap()).or_insert(0u32) += 1;
    }
    assert_eq!(cont.len(), 12);
    let chi2: f64 = cont.values().map(|&o| (o as f64 - 5e3).powi(2) / 5e3).sum();
    assert!(p_qui2(chi2, 11.0) > 1e-4, "{cont:?}");
}

/// Estado inicial de cada gerador: ida e volta exata, inclusive o índice 624 do MT19937 recém
/// iniciado e um fluxo do MRG32k3a.
#[test]
fn estado_inicial_ida_e_volta() {
    for alg in TODOS {
        let r = Rng::new(alg, 11);
        let back = Rng::from_state(&r.state()).unwrap();
        assert_eq!(back.state(), r.state(), "{alg:?}");
        let (mut a, mut b) = (r.clone(), back);
        assert_eq!(
            (0..10).map(|_| a.next_u64()).collect::<Vec<_>>(),
            (0..10).map(|_| b.next_u64()).collect::<Vec<_>>()
        );
    }
    let r = Rng::new(Algorithm::Twister, 1);
    assert_eq!(*r.state().words.last().unwrap(), 624);
    // Valores válidos no limite: chave do Philox e palavras do MT19937 iguais a u32::MAX.
    let r = Rng::new(Algorithm::Philox, u64::MAX);
    assert_eq!(&r.state().words[..2], &[u64::from(u32::MAX); 2]);
    assert_eq!(Rng::from_state(&r.state()).unwrap(), r);
    let mut w = Rng::new(Algorithm::Twister, 1).state().words;
    w[0] = u64::from(u32::MAX);
    w[623] = u64::from(u32::MAX);
    let st = RngState {
        algorithm: Algorithm::Twister,
        words: w.clone(),
    };
    assert_eq!(Rng::from_state(&st).unwrap().state().words, w);
    let r = Rng::with_stream(Algorithm::Mrg32k3a, 1, 2, 3).unwrap();
    assert_eq!(Rng::from_state(&r.state()).unwrap(), r);
}

/// Contadores grandes no Philox e no Threefry: fluxo ≥ 2³² e bloco ≥ 2³² usam as palavras altas do
/// contador. Referência: a função de bloco, validada pelos KAT do Random123.
#[test]
fn contadores_grandes() {
    let fluxo = (1u64 << 32) + 5;
    let mut r = Rng::with_stream(Algorithm::Philox, 0x1234_5678_9abc_def0, fluxo, 0).unwrap();
    let key = [0x9abc_def0, 0x1234_5678];
    let bloco = raw::philox4x32_10([0, 0, 5, 1], key);
    assert_eq!((0..4).map(|_| r.next_u32()).collect::<Vec<_>>(), bloco.to_vec());
    // Bloco 2³² + 3 restaurado de um estado salvo.
    let mut w = r.state().words;
    w[2] = (1u64 << 32) + 3;
    w[4] = 4;
    let mut r = Rng::from_state(&RngState {
        algorithm: Algorithm::Philox,
        words: w,
    })
    .unwrap();
    let bloco = raw::philox4x32_10([3, 1, 5, 1], key);
    assert_eq!((0..4).map(|_| r.next_u32()).collect::<Vec<_>>(), bloco.to_vec());
    // Threefry: fluxo e bloco nas palavras 2 e 0 do contador.
    let mut t = Rng::with_stream(Algorithm::Threefry, 77, fluxo, 0).unwrap();
    let b = raw::threefry4x64_20([0, 0, fluxo, 0], [77, 0, 0, 0]);
    assert_eq!((0..4).map(|_| t.next_u64()).collect::<Vec<_>>(), b.to_vec());
}

/// Ziggurat bit a bit contra uma implementação de referência escrita aqui a partir do artigo de
/// Marsaglia & Tsang (2000), com as tabelas recalculadas de forma independente. Os testes
/// estatísticos não distinguem erros nos caminhos raros (cauda além de r, em ~0,03% dos sorteios;
/// cunha de rejeição, em ~1%; sinal, simétrico), e este teste confere todos eles exatamente.
#[test]
fn ziggurat_bit_a_bit() {
    let r = 3.654_152_885_361_009_f64;
    let v = 0.004_928_673_233_974_655_f64;
    let f = |x: f64| (-0.5 * x * x).exp();
    let mut x = vec![v / f(r), r];
    while x.len() < 256 {
        let xi = *x.last().unwrap();
        x.push((-2.0 * (f(xi) + v / xi).ln()).sqrt());
    }
    x.push(0.0);
    let fx: Vec<f64> = x.iter().map(|&t| f(t)).collect();
    let referencia = |g: &mut Rng| -> f64 {
        loop {
            let b = g.next_u64();
            let i = (b & 0xff) as usize;
            let sinal = if b & 0x100 == 0 { 1.0 } else { -1.0 };
            let u = ((b >> 11) as f64 + 0.5) / (1u64 << 53) as f64;
            let z = u * x[i];
            if z < x[i + 1] {
                return sinal * z;
            }
            if i == 0 {
                loop {
                    let t = -g.uniform().ln() / r;
                    let y = -g.uniform().ln();
                    if 2.0 * y > t * t {
                        return sinal * (r + t);
                    }
                }
            }
            if fx[i] + g.uniform() * (fx[i + 1] - fx[i]) < f(z) {
                return sinal * z;
            }
        }
    };
    let (mut cauda, mut grandes) = (0, 0);
    for alg in [Algorithm::Philox, Algorithm::Xoshiro256pp] {
        let mut a = Rng::new(alg, 2026);
        let mut b = a.clone();
        for _ in 0..200_000 {
            let (za, zb) = (a.normal(), referencia(&mut b));
            assert_eq!(za.to_bits(), zb.to_bits(), "{alg:?}");
            cauda += usize::from(za.abs() > r);
            grandes += usize::from(za.abs() > 2.0);
        }
    }
    // Caminhos raros de fato exercidos: P(|Z| > r) ≈ 2,6e-4 → ~100 em 4·10⁵.
    assert!(cauda > 40 && grandes > 10_000, "{cauda} {grandes}");
}

/// Saltos do xoshiro256++ por matriz em GF(2): Tᵏ coincide com k passos; T^(2²⁵⁶) = T (período
/// máximo); o fluxo 0 é o gerador sem salto; fluxos grandes pelo caminho de potência de matriz
/// coincidem com saltos repetidos.
#[test]
fn xoshiro_saltos_por_matriz_gf2() {
    let mut vistos = Vec::new();
    for k in [0u64, 1, 2, 3, 63, 64, 65, 1000, 4097] {
        let (passo, matriz) = raw::xoshiro_step_vs_matrix(11, k);
        assert_eq!(passo, matriz, "k = {k}");
        // Estados distintos para cada k (o período é 2²⁵⁶ − 1).
        assert!(!vistos.contains(&passo), "k = {k}");
        vistos.push(passo);
    }
    // O passo um a um é o próprio gerador: k = 1 a partir do estado da semente.
    let mut g = Rng::new(Algorithm::Xoshiro256pp, 11);
    let s0 = g.state();
    g.next_u64();
    assert_ne!(g.state(), s0);
    assert!(raw::xoshiro_period_identity(256));
    assert!(!raw::xoshiro_period_identity(255));
    assert!(!raw::xoshiro_period_identity(128));
    // Igualdade de geradores compara o estado.
    assert!(Rng::new(Algorithm::Xoshiro256pp, 7) == Rng::new(Algorithm::Xoshiro256pp, 7));
    assert!(Rng::new(Algorithm::Xoshiro256pp, 7) != Rng::new(Algorithm::Xoshiro256pp, 8));
    let mut a = Rng::with_stream(Algorithm::Xoshiro256pp, 7, 0, 0).unwrap();
    let mut b = Rng::new(Algorithm::Xoshiro256pp, 7);
    assert_eq!(a.next_u64(), b.next_u64());
    // Fluxos grandes (potência da matriz de salto) contra saltos sucessivos.
    for n in [65u64, 100, 257] {
        let (potencia, repetido) = raw::xoshiro_jump_paths(7, n);
        assert_eq!(potencia, repetido, "n = {n}");
        assert!(!vistos.contains(&potencia), "n = {n}");
        vistos.push(potencia);
    }
    // Fluxos distintos dão estados distintos, inclusive o último.
    let ultimo = Rng::with_stream(Algorithm::Xoshiro256pp, 7, u64::MAX, 0).unwrap();
    assert_ne!(ultimo.state(), Rng::new(Algorithm::Xoshiro256pp, 7).state());
}

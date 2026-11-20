//! Testes da funcionalidade F046 (TF-046). Oráculos: DFT pela definição O(n²), sinais com
//! espectro analítico, equações de diferenças calculadas à mão e multiplicação de polinômios.

use chimalab_math::linalg::Matrix;
use chimalab_math::signal::*;
use num_complex::Complex64;
use std::f64::consts::PI;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
    fn complex(&mut self, n: usize) -> Vec<Complex64> {
        (0..n).map(|_| Complex64::new(self.next(), self.next())).collect()
    }
    fn real(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.next()).collect()
    }
}

/// DFT pela definição: X[k] = Σ x[n]·e^{−2πikn/N}.
fn dft(x: &[Complex64]) -> Vec<Complex64> {
    let n = x.len();
    (0..n)
        .map(|k| {
            x.iter()
                .enumerate()
                .map(|(j, &v)| v * Complex64::from_polar(1.0, -2.0 * PI * ((k * j) % n) as f64 / n as f64))
                .sum()
        })
        .collect()
}

fn max_err(a: &[Complex64], b: &[Complex64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| (x - y).norm()).fold(0.0, f64::max)
}

fn max_err_r(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

// ---------- FFT 1-D ----------

#[test]
fn fft_bate_com_dft_por_definicao_em_varios_tamanhos() {
    let mut rng = Rng(1);
    for n in [1usize, 2, 3, 4, 5, 7, 8, 12, 16, 17, 31, 64, 97, 100, 128, 255] {
        let x = rng.complex(n);
        let (y, s) = fft(&x, &[n], None, 0).unwrap();
        assert_eq!(s, vec![n]);
        let e = max_err(&y, &dft(&x));
        assert!(e < 1e-12 * n as f64, "n={n}: {e:e}");
    }
}

#[test]
fn ca2_fft_de_tamanho_primo_10007() {
    let n = 10007;
    let mut rng = Rng(2);
    let x = rng.complex(n);
    let (y, _) = fft(&x, &[n], None, 0).unwrap();
    // Alguns coeficientes conferidos pela definição (cada um é O(n)).
    for &k in &[0usize, 1, 2, 5003, 10006] {
        let exact: Complex64 = x
            .iter()
            .enumerate()
            .map(|(j, &v)| v * Complex64::from_polar(1.0, -2.0 * PI * ((k * j) % n) as f64 / n as f64))
            .sum();
        assert!((y[k] - exact).norm() < 1e-9, "k={k}");
    }
    // Ida e volta, e identidade de Parseval: Σ|x|² = Σ|X|²/N.
    let (z, _) = ifft(&y, &[n], None, 0).unwrap();
    assert!(max_err(&z, &x) < 1e-12);
    let ex: f64 = x.iter().map(|v| v.norm_sqr()).sum();
    let ey: f64 = y.iter().map(|v| v.norm_sqr()).sum::<f64>() / n as f64;
    assert!((ex - ey).abs() < 1e-9 * ex);
}

#[test]
fn ca1_ida_e_volta() {
    let mut rng = Rng(3);
    for n in [1usize, 6, 50, 1000, 4096] {
        let x = rng.complex(n);
        let (y, _) = fft(&x, &[n], None, 0).unwrap();
        let (z, _) = ifft(&y, &[n], None, 0).unwrap();
        assert!(max_err(&z, &x) < 1e-12, "n={n}");
    }
}

#[test]
fn espectros_analiticos() {
    // Impulso → constante 1
    let mut imp = vec![Complex64::new(0.0, 0.0); 8];
    imp[0] = Complex64::new(1.0, 0.0);
    let (y, _) = fft(&imp, &[8], None, 0).unwrap();
    assert!(y.iter().all(|v| (v - 1.0).norm() < 1e-15));
    // cos(2π·3n/N) → N/2 em k = 3 e k = N − 3
    let n = 32;
    let x: Vec<f64> = (0..n)
        .map(|j| (2.0 * PI * 3.0 * j as f64 / n as f64).cos())
        .collect();
    let (y, _) = fft_real(&x, &[n], None, 0).unwrap();
    for (k, v) in y.iter().enumerate() {
        let expected = if k == 3 || k == n - 3 { 16.0 } else { 0.0 };
        assert!((v - expected).norm() < 1e-12, "k={k}: {v}");
    }
}

#[test]
fn preenchimento_e_truncamento() {
    let x = vec![
        Complex64::new(1.0, 0.0),
        Complex64::new(2.0, 0.0),
        Complex64::new(3.0, 0.0),
    ];
    let (y, s) = fft(&x, &[3], Some(5), 0).unwrap();
    assert_eq!(s, vec![5]);
    let mut padded = x.clone();
    padded.extend([Complex64::new(0.0, 0.0); 2]);
    assert!(max_err(&y, &dft(&padded)) < 1e-14);
    let (y, s) = fft(&x, &[3], Some(2), 0).unwrap();
    assert_eq!(s, vec![2]);
    assert!(max_err(&y, &dft(&x[..2])) < 1e-15);
}

#[test]
fn ifft_simetrica_devolve_real() {
    let mut rng = Rng(4);
    let x = rng.real(20);
    let (y, _) = fft_real(&x, &[20], None, 0).unwrap();
    let (z, _) = ifft_symmetric(&y, &[20], None, 0).unwrap();
    assert!(max_err_r(&z, &x) < 1e-14);
}

// ---------- N-D ----------

#[test]
fn fft_ao_longo_de_cada_dimensao_de_array_3d() {
    let shape = [3usize, 4, 5];
    let mut rng = Rng(5);
    let x = rng.complex(60);
    let at = |i: usize, j: usize, k: usize| i + 3 * j + 12 * k;
    for dim in 0..3 {
        let (y, s) = fft(&x, &shape, None, dim).unwrap();
        assert_eq!(s, shape.to_vec());
        // Confere cada linha ao longo de `dim` pela definição.
        for a in 0..shape[(dim + 1) % 3] {
            for b in 0..shape[(dim + 2) % 3] {
                let idx = |t: usize| {
                    let mut c = [0usize; 3];
                    c[dim] = t;
                    c[(dim + 1) % 3] = a;
                    c[(dim + 2) % 3] = b;
                    at(c[0], c[1], c[2])
                };
                let line: Vec<Complex64> = (0..shape[dim]).map(|t| x[idx(t)]).collect();
                let exact = dft(&line);
                for t in 0..shape[dim] {
                    assert!((y[idx(t)] - exact[t]).norm() < 1e-13, "dim={dim}");
                }
            }
        }
    }
    // Dimensão além das existentes é singleton: transformada de tamanho 1 = identidade.
    let (y, s) = fft(&x, &shape, None, 3).unwrap();
    assert_eq!(s, vec![3, 4, 5, 1]);
    assert!(max_err(&y, &x) < 1e-15);
}

#[test]
fn fftn_ida_e_volta_e_igual_a_sequencia() {
    let shape = [6usize, 5, 4];
    let mut rng = Rng(6);
    let x = rng.complex(120);
    let (y, s) = fftn(&x, &shape, None).unwrap();
    let mut seq = x.clone();
    for d in 0..3 {
        seq = fft(&seq, &shape, None, d).unwrap().0;
    }
    assert!(max_err(&y, &seq) < 1e-12);
    let (z, _) = ifftn(&y, &s, None).unwrap();
    assert!(max_err(&z, &x) < 1e-13);
}

#[test]
fn fftshift_e_ifftshift() {
    let odd: Vec<i32> = (0..5).collect();
    assert_eq!(fftshift(&odd, &[5], None).unwrap(), vec![3, 4, 0, 1, 2]);
    assert_eq!(ifftshift(&[3, 4, 0, 1, 2], &[5], None).unwrap(), odd);
    let even: Vec<i32> = (0..6).collect();
    assert_eq!(fftshift(&even, &[6], None).unwrap(), vec![3, 4, 5, 0, 1, 2]);
    assert_eq!(
        ifftshift(&fftshift(&even, &[6], None).unwrap(), &[6], None).unwrap(),
        even
    );
    // 2-D (2×3, column-major), deslocando as duas dimensões.
    let m: Vec<i32> = vec![1, 4, 2, 5, 3, 6]; // [[1,2,3],[4,5,6]]
    let s = fftshift(&m, &[2, 3], None).unwrap();
    assert_eq!(s, vec![6, 3, 4, 1, 5, 2]); // [[6,4,5],[3,1,2]]
    // Só na dimensão 1 (colunas)
    let s = fftshift(&m, &[2, 3], Some(&[1])).unwrap();
    assert_eq!(s, vec![3, 6, 1, 4, 2, 5]); // [[3,1,2],[6,4,5]]
}

#[test]
fn nextpow2_valores() {
    assert_eq!(nextpow2(0.0), 0.0);
    assert_eq!(nextpow2(1.0), 0.0);
    assert_eq!(nextpow2(2.0), 1.0);
    assert_eq!(nextpow2(3.0), 2.0);
    assert_eq!(nextpow2(1024.0), 10.0);
    assert_eq!(nextpow2(1025.0), 11.0);
    assert_eq!(nextpow2(-7.0), 3.0);
    assert_eq!(nextpow2(0.3), -1.0);
}

// ---------- filter ----------

#[test]
fn ca3_filter_equivale_a_equacao_de_diferencas() {
    let mut rng = Rng(7);
    let x = rng.real(50);
    let b = [0.2, -0.1, 0.4];
    let a = [2.0, -0.6, 0.2, 0.1];
    let y = filter(&b, &a, &x, None).unwrap().y;
    // a0·y[n] = Σ b_k x[n−k] − Σ_{k≥1} a_k y[n−k]
    let mut yr = vec![0.0; 50];
    for n in 0..50 {
        let mut s = 0.0;
        for (k, bk) in b.iter().enumerate() {
            if n >= k {
                s += bk * x[n - k];
            }
        }
        for (k, ak) in a.iter().enumerate().skip(1) {
            if n >= k {
                s -= ak * yr[n - k];
            }
        }
        yr[n] = s / a[0];
    }
    assert!(max_err_r(&y, &yr) < 1e-14);
}

#[test]
fn filter_resposta_ao_impulso_e_fir_igual_a_convolucao() {
    // y[n] = x[n] + 0.5·y[n−1] → resposta ao impulso 0.5ⁿ
    let mut x = vec![0.0; 10];
    x[0] = 1.0;
    let y = filter(&[1.0], &[1.0, -0.5], &x, None).unwrap().y;
    for (n, v) in y.iter().enumerate() {
        assert!((v - 0.5f64.powi(n as i32)).abs() < 1e-16);
    }
    // FIR: filter(b, 1, x) = primeiros len(x) termos de conv(b, x)
    let mut rng = Rng(8);
    let b = rng.real(5);
    let x = rng.real(30);
    let y = filter(&b, &[1.0], &x, None).unwrap().y;
    let c = conv(&b, &x, ConvShape::Full);
    assert!(max_err_r(&y, &c[..30]) < 1e-14);
}

#[test]
fn filter_em_blocos_com_estado_continua_igual() {
    let mut rng = Rng(9);
    let x = rng.real(40);
    let (b, a) = ([0.3, 0.3], [1.0, -0.8, 0.15]);
    let whole = filter(&b, &a, &x, None).unwrap().y;
    let first = filter(&b, &a, &x[..17], None).unwrap();
    let second = filter(&b, &a, &x[17..], Some(&first.zf)).unwrap();
    let joined: Vec<f64> = first.y.iter().chain(&second.y).copied().collect();
    assert!(max_err_r(&joined, &whole) < 1e-15);
}

#[test]
fn filter_ao_longo_das_colunas_de_matriz() {
    // Matriz 4×2 column-major: filtra cada coluna (dim 0) e cada linha (dim 1).
    let x = [1.0, 2.0, 3.0, 4.0, 10.0, 20.0, 30.0, 40.0];
    let r = filter_along(&[1.0, 1.0], &[1.0], &x, &[4, 2], 0, None).unwrap();
    assert_eq!(r.y, vec![1.0, 3.0, 5.0, 7.0, 10.0, 30.0, 50.0, 70.0]);
    assert_eq!(r.zf, vec![4.0, 40.0]);
    let r = filter_along(&[1.0, 1.0], &[1.0], &x, &[4, 2], 1, None).unwrap();
    assert_eq!(r.y, vec![1.0, 2.0, 3.0, 4.0, 11.0, 22.0, 33.0, 44.0]);
}

#[test]
fn filter_erros() {
    assert_eq!(
        filter(&[1.0], &[0.0, 1.0], &[1.0], None).unwrap_err().id(),
        "chimalab:signal:argumentoInvalido"
    );
    assert_eq!(
        filter(&[1.0, 2.0], &[1.0], &[1.0], Some(&[1.0, 2.0]))
            .unwrap_err()
            .id(),
        "chimalab:signal:argumentoInvalido"
    );
    assert_eq!(
        fft(&[Complex64::new(1.0, 0.0)], &[2], None, 0).unwrap_err().id(),
        "chimalab:signal:formaIncompativel"
    );
}

// ---------- convolução ----------

#[test]
fn conv_multiplica_polinomios_e_respeita_formas() {
    // (1 + x)(1 − x) = 1 − x²  (coeficientes em ordem crescente ou decrescente: tanto faz)
    assert_eq!(
        conv(&[1.0, 1.0], &[1.0, -1.0], ConvShape::Full),
        vec![1.0, 0.0, -1.0]
    );
    let u = [1.0, 2.0, 3.0, 4.0, 5.0];
    let v = [1.0, 1.0, 1.0];
    assert_eq!(
        conv(&u, &v, ConvShape::Full),
        vec![1.0, 3.0, 6.0, 9.0, 12.0, 9.0, 5.0]
    );
    assert_eq!(conv(&u, &v, ConvShape::Same), vec![3.0, 6.0, 9.0, 12.0, 9.0]);
    assert_eq!(conv(&u, &v, ConvShape::Valid), vec![6.0, 9.0, 12.0]);
    // Núcleo par em 'same': começa em ⌊nb/2⌋ da convolução completa.
    assert_eq!(
        conv(&u, &[1.0, 1.0], ConvShape::Same),
        vec![3.0, 5.0, 7.0, 9.0, 5.0]
    );
    assert!(conv(&v, &u, ConvShape::Valid).is_empty());
    assert!(conv(&[], &u, ConvShape::Full).is_empty());
}

#[test]
fn conv_por_fft_igual_a_direta() {
    let mut rng = Rng(10);
    let a = rng.real(3000);
    let b = rng.real(500);
    let fast = conv(&a, &b, ConvShape::Full); // acima do limiar: caminho FFT
    let mut direct = vec![0.0; 3499];
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            direct[i + j] += x * y;
        }
    }
    assert!(max_err_r(&fast, &direct) < 1e-10);
}

#[test]
fn conv2_e_filter2() {
    let a = Matrix::from_rows(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0], &[7.0, 8.0, 9.0]]).unwrap();
    let k = Matrix::from_rows(&[&[1.0, 0.0], &[0.0, -1.0]]).unwrap();
    let full = conv2(&a, &k, ConvShape::Full);
    let exact = Matrix::from_rows(&[
        &[1.0, 2.0, 3.0, 0.0],
        &[4.0, 4.0, 4.0, -3.0],
        &[7.0, 4.0, 4.0, -6.0],
        &[0.0, -7.0, -8.0, -9.0],
    ])
    .unwrap();
    assert_eq!(full, exact);
    assert_eq!(
        conv2(&a, &k, ConvShape::Valid),
        Matrix::from_rows(&[&[4.0, 4.0], &[4.0, 4.0]]).unwrap()
    );
    assert_eq!(conv2(&a, &k, ConvShape::Same).shape(), (3, 3));
    // Correlação: filter2(h, x) = conv2(x, rot180(h))
    let corr = filter2(&k, &a, ConvShape::Valid);
    assert_eq!(corr, Matrix::from_rows(&[&[-4.0, -4.0], &[-4.0, -4.0]]).unwrap());
}

#[test]
fn convn_3d_bate_com_soma_direta() {
    let mut rng = Rng(11);
    let (sa, sb) = ([3usize, 2, 4], [2usize, 3, 2]);
    let a = rng.real(24);
    let b = rng.real(12);
    let (c, s) = convn(&a, &sa, &b, &sb, ConvShape::Full).unwrap();
    assert_eq!(s, vec![4, 4, 5]);
    let ia = |i: usize, j: usize, k: usize| i + 3 * j + 6 * k;
    let ib = |i: usize, j: usize, k: usize| i + 2 * j + 6 * k;
    let ic = |i: usize, j: usize, k: usize| i + 4 * j + 16 * k;
    let mut exact = vec![0.0; 80];
    for (i, j, k) in (0..3).flat_map(|i| (0..2).flat_map(move |j| (0..4).map(move |k| (i, j, k)))) {
        for (p, q, r) in (0..2).flat_map(|p| (0..3).flat_map(move |q| (0..2).map(move |r| (p, q, r)))) {
            exact[ic(i + p, j + q, k + r)] += a[ia(i, j, k)] * b[ib(p, q, r)];
        }
    }
    assert!(max_err_r(&c, &exact) < 1e-14);
}

#[test]
fn deconv_inverte_conv() {
    let a = [1.0, -3.0, 2.0];
    let q = [2.0, 0.5, -1.0, 4.0];
    let mut b = conv(&a, &q, ConvShape::Full);
    b[5] += 0.25; // resto
    let (qq, r) = deconv(&b, &a).unwrap();
    assert!(max_err_r(&qq, &q) < 1e-13);
    let back: Vec<f64> = conv(&a, &qq, ConvShape::Full)
        .iter()
        .zip(&r)
        .map(|(x, y)| x + y)
        .collect();
    assert!(max_err_r(&back, &b) < 1e-13);
    assert_eq!(
        deconv(&[1.0, 2.0], &[1.0, 2.0, 3.0]).unwrap(),
        (vec![0.0], vec![1.0, 2.0])
    );
    assert_eq!(
        (deconv(&[1.0], &[0.0, 1.0])).unwrap_err().id(),
        "chimalab:signal:argumentoInvalido"
    );
}

#[test]
fn fft2_transforma_so_duas_dimensoes() {
    let shape = [4usize, 3, 2];
    let mut rng = Rng(12);
    let x = rng.complex(24);
    let (y, s) = fft2(&x, &shape, None).unwrap();
    assert_eq!(s, shape.to_vec());
    let mut seq = fft(&x, &shape, None, 0).unwrap().0;
    seq = fft(&seq, &shape, None, 1).unwrap().0;
    assert!(max_err(&y, &seq) < 1e-14);
    // Diferente de fftn (que também transforma a dimensão 2).
    assert!(max_err(&y, &fftn(&x, &shape, None).unwrap().0) > 1e-3);
    let (z, _) = ifft2(&y, &s, None).unwrap();
    assert!(max_err(&z, &x) < 1e-14);
    let (_, s) = fft2(&x, &shape, Some((8, 5))).unwrap();
    assert_eq!(s, vec![8, 5, 2]);
}

#[test]
fn conv2_separavel_valores_a_mao() {
    // u = [1, 1] (soma de linhas vizinhas), v = [1, −1] (diferença de colunas).
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]).unwrap();
    let full = conv2_separable(&[1.0, 1.0], &[1.0, -1.0], &a, ConvShape::Full);
    // Colunas com u: [1 2; 4 6; 3 4]; linhas com v: [1 1 −2; 4 2 −6; 3 1 −4].
    let exact = Matrix::from_rows(&[&[1.0, 1.0, -2.0], &[4.0, 2.0, -6.0], &[3.0, 1.0, -4.0]]).unwrap();
    assert_eq!(full, exact);
    assert_eq!(
        conv2_separable(&[1.0, 1.0], &[1.0, -1.0], &a, ConvShape::Same),
        Matrix::from_rows(&[&[2.0, -6.0], &[1.0, -4.0]]).unwrap()
    );
    assert_eq!(
        conv2_separable(&[1.0, 1.0], &[1.0, -1.0], &a, ConvShape::Valid),
        Matrix::from_rows(&[&[2.0]]).unwrap()
    );
}

#[test]
fn conv2_separavel_equivale_ao_nucleo_produto_externo() {
    let mut rng = Rng(0x5eed_c0de);
    for (m, n, nu, nv) in [
        (5, 7, 3, 2),
        (4, 4, 1, 1),
        (3, 6, 4, 5),
        (8, 1, 2, 3),
        (1, 9, 1, 4),
        (0, 3, 2, 2),
        (3, 3, 0, 2),
        (2, 2, 5, 5),
        (70, 60, 40, 35), // caminho por FFT na segunda etapa
    ] {
        let a = Matrix::from_col_major(m, n, rng.real(m * n)).unwrap();
        let u = rng.real(nu);
        let v = rng.real(nv);
        let k = Matrix::from_fn(nu, nv, |i, j| u[i] * v[j]);
        for shape in [ConvShape::Full, ConvShape::Same, ConvShape::Valid] {
            let s = conv2_separable(&u, &v, &a, shape);
            let r = conv2(&a, &k, shape);
            assert_eq!(s.shape(), r.shape(), "{m}×{n}, {nu}, {nv}, {shape:?}");
            let err = s
                .as_slice()
                .iter()
                .zip(r.as_slice())
                .fold(0.0_f64, |e, (x, y)| e.max((x - y).abs()));
            assert!(err < 1e-10, "{m}×{n}, {nu}, {nv}, {shape:?}: {err:e}");
        }
    }
}

/// Escolha do caminho (direto × FFT): abaixo do limiar, ou com um dos lados < 32, a convolução
/// é direta e, para inteiros pequenos, **exata** (somas de inteiros < 2⁵³). Detecta mutantes
/// que desviam casos pequenos para a FFT, cujo resultado só é exato a menos de arredondamento.
#[test]
fn convolucao_pequena_de_inteiros_e_exata() {
    let inteiros =
        |n: usize, s: i64| -> Vec<f64> { (0..n as i64).map(|i| ((i * s) % 17 - 8) as f64).collect() };
    let exata = |u: &[f64], v: &[f64]| -> Vec<f64> {
        let mut w = vec![0.0; u.len() + v.len() - 1];
        for (i, a) in u.iter().enumerate() {
            for (j, b) in v.iter().enumerate() {
                w[i + j] += a * b;
            }
        }
        w
    };
    // Dois lados ≥ 32, mas produto abaixo do limiar.
    let (u, v) = (inteiros(40, 7), inteiros(40, 5));
    assert_eq!(conv(&u, &v, ConvShape::Full), exata(&u, &v));
    // Um lado < 32 (núcleo curto) e produto acima do limiar.
    let (u, v) = (inteiros(70_000, 3), vec![3.0, -1.0, 2.0]);
    assert_eq!(conv(&u, &v, ConvShape::Full), exata(&u, &v));
    let (u, v) = (inteiros(70_000, 11), vec![2.0]);
    assert_eq!(conv(&u, &v, ConvShape::Full), exata(&u, &v));
}

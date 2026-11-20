//! F046 com sinais complexos (DT-18): filter, conv/convn (direto e por FFT), deconv e filter2.

use chimalab_math::linalg::ComplexMatrix;
use chimalab_math::signal::*;
use num_complex::Complex64 as C;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
    fn vec(&mut self, n: usize) -> Vec<C> {
        (0..n).map(|_| C::new(self.next(), self.next())).collect()
    }
}

fn err(a: &[C], b: &[C]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| (x - y).norm()).fold(0.0, f64::max)
}

fn direct_conv(a: &[C], b: &[C]) -> Vec<C> {
    let mut out = vec![C::new(0.0, 0.0); a.len() + b.len() - 1];
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            out[i + j] += x * y;
        }
    }
    out
}

#[test]
fn filter_complexo_equivale_a_equacao_de_diferencas() {
    let mut rng = Rng(1);
    let x = rng.vec(40);
    let b = rng.vec(3);
    let a = [C::new(1.0, 0.5), C::new(-0.3, 0.1), C::new(0.05, -0.02)];
    let y = filter(&b, &a, &x, None).unwrap().y;
    let mut yr = vec![C::new(0.0, 0.0); 40];
    for n in 0..40 {
        let mut s = C::new(0.0, 0.0);
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
    assert!(err(&y, &yr) < 1e-13);
}

#[test]
fn conv_complexa_direta_e_por_fft() {
    let mut rng = Rng(2);
    let (a, b) = (rng.vec(7), rng.vec(4));
    assert!(err(&conv(&a, &b, ConvShape::Full), &direct_conv(&a, &b)) < 1e-14);
    let (a, b) = (rng.vec(2000), rng.vec(300)); // acima do limiar: FFT
    assert!(err(&conv(&a, &b, ConvShape::Full), &direct_conv(&a, &b)) < 1e-10);
}

#[test]
fn deconv_complexa() {
    let mut rng = Rng(3);
    let a = rng.vec(3);
    let q = rng.vec(5);
    let b = conv(&a, &q, ConvShape::Full);
    let (qq, r) = deconv(&b, &a).unwrap();
    assert!(err(&qq, &q) < 1e-12);
    assert!(r.iter().all(|z| z.norm() < 1e-12));
}

#[test]
fn conv2_e_filter2_complexos() {
    let i = C::new(0.0, 1.0);
    let one = C::new(1.0, 0.0);
    let z = C::new(0.0, 0.0);
    let a = ComplexMatrix::from_rows(&[&[one, i], &[i, one]]).unwrap();
    let k = ComplexMatrix::from_rows(&[&[one, z], &[z, i]]).unwrap();
    // Convolução completa calculada à mão:
    // [[1, i, 0], [i, 1+i·1, i·i], [0, i·i, i·1]] = [[1, i, 0], [i, 1+i, −1], [0, −1, i]]
    let full = conv2(&a, &k, ConvShape::Full);
    let exact = ComplexMatrix::from_rows(&[&[one, i, z], &[i, one + i, -one], &[z, -one, i]]).unwrap();
    assert!(full.sub(&exact).unwrap().max_abs() < 1e-15);
    // Correlação = convolução com núcleo rotacionado
    let corr = filter2(&k, &a, ConvShape::Valid);
    assert_eq!(corr.shape(), (1, 1));
    assert!((corr.get(0, 0) - (i * one + one * one)).norm() < 1e-15);
}

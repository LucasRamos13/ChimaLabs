//! Triangulação de Delaunay 2-D incremental (Bowyer–Watson) com triângulos fantasmas.
//!
//! Cada aresta do fecho tem um triângulo "fantasma" com um vértice no infinito (G). Assim, um
//! ponto fora do fecho entra pelo mesmo mecanismo que um ponto interno, sem o supertriângulo,
//! cujos vértices finitos estragam a triangulação perto do fecho. Os testes de conflito usam os
//! predicados exatos.
//!
//! Conflito de um ponto p com um triângulo:
//! - triângulo real (a, b, c), anti-horário: incircle(a, b, c, p) > 0;
//! - fantasma (b, a, G), do lado de fora da aresta a→b do fecho: p estritamente à esquerda de
//!   b→a, ou sobre a reta e estritamente dentro do segmento (Shewchuk, 1996).

use std::collections::HashMap;

use super::GeomError;
use super::predicates::{incircle, orient2d};

const G: usize = usize::MAX;
/// Vizinho inexistente (aresta do fecho) na triangulação final.
pub const NONE: usize = usize::MAX;

/// Triangulação de Delaunay 2-D.
#[derive(Clone, Debug)]
pub struct Delaunay {
    points: Vec<[f64; 2]>,
    triangles: Vec<[usize; 3]>,
    neighbors: Vec<[usize; 3]>,
    duplicates: Vec<(usize, usize)>,
}

struct Builder<'p> {
    p: &'p [[f64; 2]],
    tri: Vec<[usize; 3]>,
    nbr: Vec<[usize; 3]>,
    alive: Vec<bool>,
    free: Vec<usize>,
    last: usize,
}

impl Builder<'_> {
    fn pt(&self, v: usize) -> [f64; 2] {
        self.p[v]
    }

    fn new_tri(&mut self, v: [usize; 3]) -> usize {
        if let Some(t) = self.free.pop() {
            self.tri[t] = v;
            self.nbr[t] = [NONE; 3];
            self.alive[t] = true;
            t
        } else {
            self.tri.push(v);
            self.nbr.push([NONE; 3]);
            self.alive.push(true);
            self.tri.len() - 1
        }
    }

    fn conflict(&self, t: usize, q: [f64; 2]) -> bool {
        let [a, b, c] = self.tri[t];
        if c == G {
            // Fantasma (a, b, G): aresta real a→b vista de fora do fecho.
            let (pa, pb) = (self.pt(a), self.pt(b));
            let o = orient2d(pa, pb, q);
            if o != 0.0 {
                return o > 0.0;
            }
            // Colinear: dentro do segmento aberto?
            let dentro = |u: f64, v: f64, w: f64| (u < w && w < v) || (v < w && w < u);
            return if pa[0] != pb[0] {
                dentro(pa[0], pb[0], q[0])
            } else {
                dentro(pa[1], pb[1], q[1])
            };
        }
        incircle(self.pt(a), self.pt(b), self.pt(c), q) > 0.0
    }

    /// Caminhada de visibilidade até um triângulo em conflito com q; `Err(v)` se q coincide com o
    /// vértice v.
    fn locate(&self, q: [f64; 2]) -> Result<usize, usize> {
        let mut t = self.last;
        let limite = 4 * self.tri.len() + 16;
        let mut gira = 0usize;
        for _ in 0..limite {
            let [a, b, c] = self.tri[t];
            if c == G {
                if self.conflict(t, q) {
                    return Ok(t);
                }
                // Volta para o triângulo real da aresta e continua a caminhada.
                t = self.nbr[t][2];
                continue;
            }
            for v in [a, b, c] {
                if self.pt(v) == q {
                    return Err(v);
                }
            }
            let mut moveu = false;
            gira = gira.wrapping_add(1);
            for s in 0..3 {
                let k = (s + gira) % 3;
                let (u, w) = (self.tri[t][(k + 1) % 3], self.tri[t][(k + 2) % 3]);
                if orient2d(self.pt(u), self.pt(w), q) < 0.0 {
                    t = self.nbr[t][k];
                    moveu = true;
                    break;
                }
            }
            if !moveu {
                return Ok(t);
            }
        }
        // Salvaguarda: busca linear.
        (0..self.tri.len())
            .find(|&t| self.alive[t] && self.conflict(t, q))
            .ok_or(usize::MAX)
    }

    fn insert(&mut self, i: usize) -> Option<usize> {
        let q = self.pt(i);
        let t0 = match self.locate(q) {
            Ok(t) => t,
            Err(v) => return Some(v),
        };
        // Cavidade: triângulos em conflito alcançáveis a partir de t0.
        let mut cav = vec![t0];
        let mut na_cav: HashMap<usize, ()> = HashMap::new();
        na_cav.insert(t0, ());
        let mut borda: Vec<(usize, usize, usize, usize)> = Vec::new(); // (u, v, fora, k_fora)
        let mut k = 0;
        while k < cav.len() {
            let t = cav[k];
            k += 1;
            for e in 0..3 {
                let n = self.nbr[t][e];
                if na_cav.contains_key(&n) {
                    continue;
                }
                if self.conflict(n, q) {
                    na_cav.insert(n, ());
                    cav.push(n);
                } else {
                    let (u, v) = (self.tri[t][(e + 1) % 3], self.tri[t][(e + 2) % 3]);
                    let kf = (0..3)
                        .find(|&j| self.nbr[n][j] == t)
                        .expect("adjacência simétrica");
                    borda.push((u, v, n, kf));
                }
            }
        }
        for &t in &cav {
            self.alive[t] = false;
            self.free.push(t);
        }
        // Novos triângulos (u, v, i), com G sempre na última posição.
        let mut raios: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut algum = 0;
        for &(u, v, fora, kf) in &borda {
            let verts = if u == G {
                [v, i, G]
            } else if v == G {
                [i, u, G]
            } else {
                [u, v, i]
            };
            let t = self.new_tri(verts);
            algum = t;
            // Vizinho do outro lado da aresta (u, v).
            let ke = (0..3)
                .find(|&j| verts[j] != u && verts[j] != v)
                .expect("vértice oposto");
            self.nbr[t][ke] = fora;
            self.nbr[fora][kf] = t;
            // Arestas que tocam i: ligadas aos outros novos triângulos pela aresta orientada.
            for j in 0..3 {
                if j == ke {
                    continue;
                }
                let (a, b) = (verts[(j + 1) % 3], verts[(j + 2) % 3]);
                if let Some((t2, j2)) = raios.remove(&(b, a)) {
                    self.nbr[t][j] = t2;
                    self.nbr[t2][j2] = t;
                } else {
                    raios.insert((a, b), (t, j));
                }
            }
        }
        debug_assert!(raios.is_empty());
        self.last = algum;
        None
    }
}

/// Índice na curva de Hilbert (ordem 16) de um ponto na grade [0, 2¹⁶)².
fn hilbert(mut x: u32, mut y: u32) -> u64 {
    let n: u32 = 1 << 16;
    let mut d: u64 = 0;
    let mut s = n / 2;
    while s > 0 {
        let rx = u32::from(x & s > 0);
        let ry = u32::from(y & s > 0);
        d += u64::from(s) * u64::from(s) * u64::from((3 * rx) ^ ry);
        if ry == 0 {
            if rx == 1 {
                x = n - 1 - x;
                y = n - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        s /= 2;
    }
    d
}

impl Delaunay {
    /// Triangula os pontos. Pontos repetidos são ignorados (ver [`Self::duplicates`]).
    ///
    /// Erros: `pontosInsuficientes` (menos de 3 pontos distintos), `naoFinito` (coordenada NaN ou
    /// infinita), `degenerado` (todos colineares: não há triângulo).
    pub fn new(points: &[[f64; 2]]) -> Result<Self, GeomError> {
        if points.iter().flatten().any(|v| !v.is_finite()) {
            return Err(GeomError::NonFinite);
        }
        if points.len() < 3 {
            return Err(GeomError::TooFewPoints {
                needed: 3,
                got: points.len(),
            });
        }
        // Ordem de Hilbert: pontos próximos inseridos em sequência deixam a caminhada curta.
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in points {
            for d in 0..2 {
                lo[d] = lo[d].min(p[d]);
                hi[d] = hi[d].max(p[d]);
            }
        }
        let escala = |v: f64, d: usize| -> u32 {
            let w = hi[d] - lo[d];
            if w > 0.0 {
                (((v - lo[d]) / w) * 65535.0) as u32
            } else {
                0
            }
        };
        let mut ordem: Vec<usize> = (0..points.len()).collect();
        ordem.sort_by_key(|&i| hilbert(escala(points[i][0], 0), escala(points[i][1], 1)));
        // Semente: três pontos não colineares.
        let i0 = ordem[0];
        let i1 = ordem.iter().copied().find(|&i| points[i] != points[i0]);
        let Some(i1) = i1 else {
            return Err(GeomError::TooFewPoints { needed: 3, got: 1 });
        };
        let i2 = ordem
            .iter()
            .copied()
            .find(|&i| orient2d(points[i0], points[i1], points[i]) != 0.0);
        let Some(i2) = i2 else {
            return Err(GeomError::Degenerate("todos os pontos são colineares"));
        };
        let mut b = Builder {
            p: points,
            tri: Vec::new(),
            nbr: Vec::new(),
            alive: Vec::new(),
            free: Vec::new(),
            last: 0,
        };
        let (a, c) = if orient2d(points[i0], points[i1], points[i2]) > 0.0 {
            (i1, i2)
        } else {
            (i2, i1)
        };
        let real = b.new_tri([i0, a, c]);
        // Fantasmas das três arestas, com G na última posição: aresta u→v do real dá (v, u, G).
        let v = [i0, a, c];
        let mut fantasmas = [0usize; 3];
        for k in 0..3 {
            let (u, w) = (v[(k + 1) % 3], v[(k + 2) % 3]);
            fantasmas[k] = b.new_tri([w, u, G]);
            b.nbr[real][k] = fantasmas[k];
            b.nbr[fantasmas[k]][2] = real;
        }
        // Entre fantasmas: (w, u, G) e o fantasma que começa em u compartilham a aresta (u, G).
        for &f in &fantasmas {
            let u = b.tri[f][1];
            for &g in &fantasmas {
                if b.tri[g][0] == u {
                    // f tem a aresta (u, G) na posição 0; g, que começa em u, tem (G, u) na 1.
                    b.nbr[f][0] = g;
                    b.nbr[g][1] = f;
                }
            }
        }
        b.last = real;
        let mut duplicates = Vec::new();
        for &i in &ordem {
            if i == i0 || i == a || i == c {
                continue;
            }
            if let Some(orig) = b.insert(i) {
                duplicates.push((i, orig));
            }
        }
        // Triângulos reais, renumerados, e vizinhos (NONE nas arestas do fecho).
        let mut novo = vec![NONE; b.tri.len()];
        let mut triangles = Vec::new();
        for t in 0..b.tri.len() {
            if b.alive[t] && b.tri[t][2] != G {
                novo[t] = triangles.len();
                triangles.push(b.tri[t]);
            }
        }
        let neighbors = (0..b.tri.len())
            .filter(|&t| novo[t] != NONE)
            .map(|t| b.nbr[t].map(|n| novo[n]))
            .collect();
        duplicates.sort_unstable();
        Ok(Self {
            points: points.to_vec(),
            triangles,
            neighbors,
            duplicates,
        })
    }

    /// Pontos de entrada (inclusive os repetidos).
    pub fn points(&self) -> &[[f64; 2]] {
        &self.points
    }

    /// Triângulos (índices nos pontos), em sentido anti-horário.
    pub fn triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }

    /// `neighbors()[t][k]`: triângulo do outro lado da aresta oposta ao vértice k de t
    /// ([`NONE`] no fecho).
    pub fn neighbors(&self) -> &[[usize; 3]] {
        &self.neighbors
    }

    /// Pares (repetido, original) dos pontos ignorados por coincidirem com outro.
    pub fn duplicates(&self) -> &[(usize, usize)] {
        &self.duplicates
    }

    /// Confere o critério de Delaunay em cada aresta interna, com o incircle exato: o vértice
    /// oposto do vizinho não está estritamente dentro do círculo circunscrito.
    pub fn is_delaunay(&self) -> bool {
        self.triangles.iter().zip(&self.neighbors).all(|(t, nb)| {
            nb.iter().all(|&n| {
                if n == NONE {
                    return true;
                }
                let oposto = self.triangles[n].iter().find(|v| !t.contains(v)).copied();
                oposto.is_none_or(|o| {
                    let p = |i: usize| self.points[i];
                    incircle(p(t[0]), p(t[1]), p(t[2]), p(o)) <= 0.0
                })
            })
        })
    }

    /// Triângulo que contém q e as coordenadas baricêntricas (λ₀, λ₁, λ₂) em relação aos seus
    /// vértices; `None` fora do fecho. Pontos sobre arestas pertencem a um dos triângulos.
    pub fn locate(&self, q: [f64; 2]) -> Option<(usize, [f64; 3])> {
        if self.triangles.is_empty() || q.iter().any(|v| !v.is_finite()) {
            return None;
        }
        let mut t = 0usize;
        let mut gira = 0usize;
        for _ in 0..4 * self.triangles.len() + 16 {
            let v = self.triangles[t];
            let p = |i: usize| self.points[v[i]];
            let mut moveu = false;
            gira += 1;
            for s in 0..3 {
                let k = (s + gira) % 3;
                if orient2d(p((k + 1) % 3), p((k + 2) % 3), q) < 0.0 {
                    let n = self.neighbors[t][k];
                    if n == NONE {
                        return None;
                    }
                    t = n;
                    moveu = true;
                    break;
                }
            }
            if !moveu {
                return Some((t, self.barycentric(t, q)));
            }
        }
        // Salvaguarda: busca linear.
        (0..self.triangles.len())
            .find(|&t| {
                let v = self.triangles[t];
                (0..3).all(|k| orient2d(self.points[v[(k + 1) % 3]], self.points[v[(k + 2) % 3]], q) >= 0.0)
            })
            .map(|t| (t, self.barycentric(t, q)))
    }

    fn barycentric(&self, t: usize, q: [f64; 2]) -> [f64; 3] {
        let [a, b, c] = self.triangles[t].map(|i| self.points[i]);
        let det = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        let l0 = ((b[1] - c[1]) * (q[0] - c[0]) + (c[0] - b[0]) * (q[1] - c[1])) / det;
        let l1 = ((c[1] - a[1]) * (q[0] - c[0]) + (a[0] - c[0]) * (q[1] - c[1])) / det;
        [l0, l1, 1.0 - l0 - l1]
    }
}

#[cfg(test)]
mod testes_criterio {
    use super::*;

    /// `is_delaunay` recusa uma triangulação válida que viola o critério: a diagonal A–C do
    /// quadrilátero, com D dentro do círculo por A, B, C (mutação: a verificação nunca era
    /// exercitada com resposta falsa).
    #[test]
    fn criterio_recusa_diagonal_errada() {
        let points = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 1.0], [0.5, 1.2]];
        assert!(incircle(points[0], points[1], points[2], points[3]) > 0.0);
        // T0 = [A, B, C], T1 = [A, C, D]; aresta comum A–C (oposta a B em T0 e a D em T1).
        let errada = Delaunay {
            points: points.clone(),
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            neighbors: vec![[NONE, 1, NONE], [NONE, NONE, 0]],
            duplicates: vec![],
        };
        assert!(!errada.is_delaunay());
        let certa = Delaunay::new(&points).unwrap();
        assert!(certa.is_delaunay());
        assert_eq!(certa.triangles().len(), 2);
    }
}

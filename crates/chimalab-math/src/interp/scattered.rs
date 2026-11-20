//! Interpolação de dados dispersos 2-D sobre a triangulação de Delaunay (F047).
//!
//! - Linear: coordenadas baricêntricas no triângulo que contém o ponto (contínua, exata para
//!   funções afins).
//! - Vizinho mais próximo: caminhada gulosa no grafo de Delaunay, que sempre termina no ponto
//!   mais próximo. Pela dualidade com o diagrama de Voronoi, se v não é o mais próximo de q, o
//!   segmento de v a q sai da célula de v por uma aresta compartilhada com um vizinho u de
//!   Delaunay, e q está do lado de u da mediatriz: |uq| < |vq|.
//!
//! - Vizinho natural (Sibson, 1981): pesos proporcionais às áreas que a célula de Voronoi de q
//!   tomaria de cada vizinho se q fosse inserido. Cálculo pelos circuncentros, sem inserir:
//!   - a cavidade são os triângulos cujo círculo circunscrito contém q (incircle exato);
//!   - para cada vértice i da borda da cavidade, com vizinhos p e n na borda, a área tomada de i
//!     é a do polígono [cc(p, i, q), circuncentros dos triângulos da cavidade em volta de i,
//!     cc(i, n, q)] (D. F. Watson, *Contouring*, Pergamon, 1992; J.-D. Boissonnat & F. Cazals,
//!     Comput. Geom. 22 (2002) 185–203).
//!
//!   Coordenadas relativas a q, para reduzir o cancelamento. Contínua, exata para funções
//!   afins, C¹ fora dos pontos de dado. Sobre a borda do fecho (triângulo novo degenerado),
//!   coincide com a linear ao longo da aresta, que é o seu limite.
//!
//! Pontos repetidos recebem a média dos seus valores.

use super::{Extrap, InterpError};
use std::collections::HashMap;

use crate::geom::{Delaunay, GeomError, NONE, incircle, orient2d};

/// Método para dados dispersos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScatteredMethod {
    /// Linear por triângulo (padrão).
    #[default]
    Linear,
    /// Valor do ponto de dado mais próximo.
    Nearest,
    /// Vizinho natural de Sibson.
    Natural,
}

/// Interpolante de dados dispersos 2-D.
#[derive(Clone, Debug)]
pub struct ScatteredInterpolant {
    tri: Delaunay,
    values: Vec<f64>,
    adjacency: Vec<Vec<usize>>,
    method: ScatteredMethod,
}

impl From<GeomError> for InterpError {
    fn from(e: GeomError) -> Self {
        match e {
            GeomError::TooFewPoints { needed, got } => InterpError::TooFewPoints { needed, got },
            GeomError::NonFinite => InterpError::NotMonotonic,
            GeomError::Degenerate(m) => InterpError::InvalidOption(m),
        }
    }
}

impl ScatteredInterpolant {
    /// Constrói a partir de pontos (x, y) e valores.
    ///
    /// Erros: `formaIncompativel`, `pontosInsuficientes` (menos de 3 pontos distintos),
    /// `naoMonotono` (coordenada não finita), `opcaoInvalida` (todos os pontos colineares).
    pub fn new(points: &[[f64; 2]], values: &[f64], method: ScatteredMethod) -> Result<Self, InterpError> {
        if points.len() != values.len() {
            return Err(InterpError::ShapeMismatch {
                expected: points.len(),
                actual: values.len(),
            });
        }
        let tri = Delaunay::new(points)?;
        // Média dos valores dos repetidos, guardada no original.
        let mut v = values.to_vec();
        let mut soma: std::collections::HashMap<usize, (f64, usize)> = std::collections::HashMap::new();
        for &(dup, orig) in tri.duplicates() {
            let e = soma.entry(orig).or_insert((values[orig], 1));
            e.0 += values[dup];
            e.1 += 1;
        }
        for (orig, (s, k)) in soma {
            v[orig] = s / k as f64;
        }
        let mut adjacency = vec![Vec::new(); points.len()];
        for t in tri.triangles() {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                if !adjacency[a].contains(&b) {
                    adjacency[a].push(b);
                }
                if !adjacency[b].contains(&a) {
                    adjacency[b].push(a);
                }
            }
        }
        Ok(Self {
            tri,
            values: v,
            adjacency,
            method,
        })
    }

    /// Valor em q. Fora do fecho convexo, segue `extrap`; `Extrapolate` e `Clamp` usam o valor do
    /// ponto de dado mais próximo.
    pub fn eval(&self, q: [f64; 2], extrap: Extrap) -> f64 {
        if q.iter().any(|v| v.is_nan()) {
            return f64::NAN;
        }
        let Some((t, l)) = self.tri.locate(q) else {
            return match extrap {
                Extrap::Nan => f64::NAN,
                Extrap::Value(v) => v,
                Extrap::Extrapolate | Extrap::Clamp => {
                    self.values[self.nearest_from(self.tri.triangles()[0][0], q)]
                }
            };
        };
        let v = self.tri.triangles()[t];
        match self.method {
            ScatteredMethod::Linear => {
                let mut acc = 0.0;
                for k in 0..3 {
                    // Peso nulo não propaga NaN de vértices que não contam.
                    if l[k] != 0.0 {
                        acc += l[k] * self.values[v[k]];
                    }
                }
                acc
            }
            ScatteredMethod::Nearest => self.values[self.nearest_from(v[0], q)],
            ScatteredMethod::Natural => match self.sibson(t, q) {
                Some(w) => w.iter().map(|&(i, wi)| wi * self.values[i]).sum(),
                None => (0..3)
                    .filter(|&k| l[k] != 0.0)
                    .map(|k| l[k] * self.values[v[k]])
                    .sum(),
            },
        }
    }

    /// Coordenadas de vizinho natural (Sibson) de q: pares (índice do ponto, peso), pesos ≥ 0
    /// com soma 1 e Σ wᵢ·xᵢ = q. `None` fora do fecho convexo ou sobre a sua borda (onde as
    /// coordenadas são as lineares da aresta).
    pub fn sibson_coordinates(&self, q: [f64; 2]) -> Option<Vec<(usize, f64)>> {
        let (t, _) = self.tri.locate(q)?;
        self.sibson(t, q)
    }

    fn sibson(&self, t0: usize, q: [f64; 2]) -> Option<Vec<(usize, f64)>> {
        let pts = self.tri.points();
        let tris = self.tri.triangles();
        let nb = self.tri.neighbors();
        if let Some(&v) = tris[t0].iter().find(|&&v| pts[v] == q) {
            return Some(vec![(v, 1.0)]);
        }
        // Cavidade por busca a partir do triângulo que contém q.
        let mut na_cav: HashMap<usize, bool> = HashMap::new();
        let mut pilha = vec![t0];
        na_cav.insert(t0, true);
        let mut cav = Vec::new();
        while let Some(t) = pilha.pop() {
            cav.push(t);
            for &n in &nb[t] {
                if n == NONE || na_cav.contains_key(&n) {
                    continue;
                }
                let v = tris[n];
                let dentro = incircle(pts[v[0]], pts[v[1]], pts[v[2]], q) > 0.0;
                na_cav.insert(n, dentro);
                if dentro {
                    pilha.push(n);
                }
            }
        }
        // Arestas orientadas da cavidade → triângulo; borda: aresta cujo vizinho não é da cavidade.
        let mut aresta_tri: HashMap<(usize, usize), usize> = HashMap::new();
        let mut prox: HashMap<usize, usize> = HashMap::new();
        let mut ant: HashMap<usize, usize> = HashMap::new();
        for &t in &cav {
            let v = tris[t];
            for k in 0..3 {
                let (a, b) = (v[(k + 1) % 3], v[(k + 2) % 3]);
                aresta_tri.insert((a, b), t);
                let n = nb[t][k];
                if n == NONE || !na_cav.get(&n).copied().unwrap_or(false) {
                    // q sobre a borda do fecho: triângulo (a, b, q) degenerado.
                    if orient2d(pts[a], pts[b], q) <= 0.0 {
                        return None;
                    }
                    prox.insert(a, b);
                    ant.insert(b, a);
                }
            }
        }
        let rel = |i: usize| [pts[i][0] - q[0], pts[i][1] - q[1]];
        let cc = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
            let (bx, by, cx, cy) = (b[0] - a[0], b[1] - a[1], c[0] - a[0], c[1] - a[1]);
            let d = 2.0 * (bx * cy - by * cx);
            let (b2, c2) = (bx * bx + by * by, cx * cx + cy * cy);
            [a[0] + (cy * b2 - by * c2) / d, a[1] + (bx * c2 - cx * b2) / d]
        };
        let origem = [0.0, 0.0];
        let mut pesos = Vec::with_capacity(prox.len());
        let mut total = 0.0;
        // Ordem fixa (por índice): a soma das áreas não depende da ordem do HashMap.
        let mut borda: Vec<(usize, usize)> = prox.iter().map(|(&i, &n)| (i, n)).collect();
        borda.sort_unstable();
        for (i, n) in borda {
            let p = *ant.get(&i)?;
            let mut poli = vec![cc(rel(p), rel(i), origem)];
            let mut t = *aresta_tri.get(&(p, i))?;
            for _ in 0..=cav.len() {
                let v = tris[t];
                poli.push(cc(rel(v[0]), rel(v[1]), rel(v[2])));
                let k = v.iter().position(|&x| x == i)?;
                let x = v[(k + 1) % 3];
                if x == n {
                    break;
                }
                t = *aresta_tri.get(&(x, i))?;
            }
            poli.push(cc(rel(i), rel(n), origem));
            let m = poli.len();
            let area = 0.5
                * (0..m)
                    .map(|j| {
                        let (u, w) = (poli[j], poli[(j + 1) % m]);
                        u[0] * w[1] - u[1] * w[0]
                    })
                    .sum::<f64>()
                    .abs();
            total += area;
            pesos.push((i, area));
        }
        if !(total.is_finite() && total > 0.0) {
            return None;
        }
        for p in &mut pesos {
            p.1 /= total;
        }
        Some(pesos)
    }

    /// Valores em vários pontos.
    pub fn eval_many(&self, qs: &[[f64; 2]], extrap: Extrap) -> Vec<f64> {
        qs.iter().map(|&q| self.eval(q, extrap)).collect()
    }

    /// Índice do ponto de dado mais próximo de q (caminhada gulosa no grafo de Delaunay; empate:
    /// o de menor índice).
    pub fn nearest_point(&self, q: [f64; 2]) -> usize {
        self.nearest_from(self.tri.triangles()[0][0], q)
    }

    fn nearest_from(&self, start: usize, q: [f64; 2]) -> usize {
        let p = self.tri.points();
        let d2 = |i: usize| (p[i][0] - q[0]).powi(2) + (p[i][1] - q[1]).powi(2);
        let mut cur = start;
        loop {
            let mut best = cur;
            for &n in &self.adjacency[cur] {
                if d2(n) < d2(best) || (d2(n) == d2(best) && n < best) {
                    best = n;
                }
            }
            if best == cur {
                return cur;
            }
            cur = best;
        }
    }
}

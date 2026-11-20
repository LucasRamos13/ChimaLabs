//! Geometria computacional (funcionalidade F047, RF-047; DT-28).
//!
//! - [`Delaunay`]: triangulação de Delaunay 2-D, com vizinhanças e localização de pontos.
//! - [`convex_hull`]: fecho convexo 2-D (cadeia monótona de Andrew).
//! - Polígonos: [`polygon_area`], [`polygon_centroid`], [`polygon_perimeter`] e
//!   [`point_in_polygon`].
//! - Voronoi 2-D: [`voronoi`] (células com raios) e [`voronoi_clipped`] (recortadas).
//! - Regiões com buracos e operações booleanas: [`Polygon`], [`polygon_boolean`], [`BoolOp`].
//! - [`orient2d`] e [`incircle`]: predicados com sinal exato (Shewchuk, 1997).

mod boolean;
mod delaunay;
mod hull3;
mod predicates;
mod voronoi;

use std::fmt;

pub use boolean::{BoolOp, Polygon, polygon_boolean};
pub use delaunay::{Delaunay, NONE};
pub use hull3::{convex_hull_3d, hull_volume};
pub use predicates::{incircle, orient2d, orient3d};
pub use voronoi::{VoronoiCell, circumcenter, voronoi, voronoi_clipped};

/// Erros de geometria.
#[derive(Clone, Debug, PartialEq)]
pub enum GeomError {
    /// Menos pontos (distintos) que o necessário.
    TooFewPoints {
        /// Mínimo.
        needed: usize,
        /// Recebidos.
        got: usize,
    },
    /// Coordenada NaN ou infinita.
    NonFinite,
    /// Configuração degenerada (todos colineares, polígono sem área).
    Degenerate(&'static str),
}

impl GeomError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::TooFewPoints { .. } => "chimalab:geom:pontosInsuficientes",
            Self::NonFinite => "chimalab:geom:naoFinito",
            Self::Degenerate(_) => "chimalab:geom:degenerado",
        }
    }
}

impl fmt::Display for GeomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewPoints { needed, got } => write!(f, "pontos insuficientes: {got}; exigidos {needed}"),
            Self::NonFinite => write!(f, "coordenada não finita"),
            Self::Degenerate(m) => write!(f, "configuração degenerada: {m}"),
        }
    }
}

impl std::error::Error for GeomError {}

/// Fecho convexo 2-D: índices dos vértices em sentido anti-horário, começando pelo ponto de menor
/// x (e menor y no empate). Pontos sobre as arestas não entram; repetidos contam uma vez.
///
/// Com todos os pontos colineares, devolve os dois extremos (ou um, se todos coincidem).
/// Erros: `naoFinito`. A entrada vazia dá um fecho vazio.
pub fn convex_hull(points: &[[f64; 2]]) -> Result<Vec<usize>, GeomError> {
    if points.iter().flatten().any(|v| !v.is_finite()) {
        return Err(GeomError::NonFinite);
    }
    let mut idx: Vec<usize> = (0..points.len()).collect();
    idx.sort_by(|&a, &b| {
        points[a][0]
            .total_cmp(&points[b][0])
            .then(points[a][1].total_cmp(&points[b][1]))
            .then(a.cmp(&b))
    });
    idx.dedup_by(|a, b| points[*a] == points[*b]);
    if idx.len() < 3 {
        return Ok(idx);
    }
    let mut lower: Vec<usize> = Vec::new();
    for &i in &idx {
        while lower.len() >= 2
            && orient2d(
                points[lower[lower.len() - 2]],
                points[lower[lower.len() - 1]],
                points[i],
            ) <= 0.0
        {
            lower.pop();
        }
        lower.push(i);
    }
    let mut upper: Vec<usize> = Vec::new();
    for &i in idx.iter().rev() {
        while upper.len() >= 2
            && orient2d(
                points[upper[upper.len() - 2]],
                points[upper[upper.len() - 1]],
                points[i],
            ) <= 0.0
        {
            upper.pop();
        }
        upper.push(i);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    Ok(lower)
}

/// Área com sinal (positiva se os vértices estão em sentido anti-horário), pela fórmula de Gauss
/// com coordenadas relativas ao primeiro vértice (reduz o cancelamento longe da origem).
pub fn polygon_area(poly: &[[f64; 2]]) -> f64 {
    if poly.len() < 3 {
        return 0.0;
    }
    let o = poly[0];
    let mut s = 0.0;
    for i in 1..poly.len() - 1 {
        let (a, b) = (poly[i], poly[i + 1]);
        s += (a[0] - o[0]) * (b[1] - o[1]) - (b[0] - o[0]) * (a[1] - o[1]);
    }
    s / 2.0
}

/// Perímetro (polígono fechado implicitamente).
pub fn polygon_perimeter(poly: &[[f64; 2]]) -> f64 {
    let n = poly.len();
    (0..n)
        .map(|i| {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            (b[0] - a[0]).hypot(b[1] - a[1])
        })
        .sum()
}

/// Centroide da região do polígono simples.
///
/// Erros: `degenerado` (área nula).
pub fn polygon_centroid(poly: &[[f64; 2]]) -> Result<[f64; 2], GeomError> {
    let area = polygon_area(poly);
    if area == 0.0 {
        return Err(GeomError::Degenerate("polígono sem área"));
    }
    let o = poly[0];
    let (mut cx, mut cy) = (0.0, 0.0);
    for i in 1..poly.len() - 1 {
        let (a, b) = (
            [poly[i][0] - o[0], poly[i][1] - o[1]],
            [poly[i + 1][0] - o[0], poly[i + 1][1] - o[1]],
        );
        let cr = a[0] * b[1] - b[0] * a[1];
        cx += (a[0] + b[0]) * cr;
        cy += (a[1] + b[1]) * cr;
    }
    Ok([o[0] + cx / (6.0 * area), o[1] + cy / (6.0 * area)])
}

/// Posição de um ponto em relação a um polígono.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    /// Dentro (número de voltas ≠ 0).
    Inside,
    /// Fora.
    Outside,
    /// Sobre uma aresta ou vértice (decidido com o predicado exato).
    Boundary,
}

/// Ponto no polígono pelo número de voltas (D. Sunday; inclui polígonos não simples: regiões com
/// número de voltas ≠ 0 são "dentro"). A borda é detectada exatamente.
pub fn point_in_polygon(q: [f64; 2], poly: &[[f64; 2]]) -> Location {
    let n = poly.len();
    if n == 0 || q.iter().any(|v| v.is_nan()) {
        return Location::Outside;
    }
    let mut voltas = 0i64;
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let o = orient2d(a, b, q);
        if o == 0.0
            && q[0] >= a[0].min(b[0])
            && q[0] <= a[0].max(b[0])
            && q[1] >= a[1].min(b[1])
            && q[1] <= a[1].max(b[1])
        {
            return Location::Boundary;
        }
        if a[1] <= q[1] {
            if b[1] > q[1] && o > 0.0 {
                voltas += 1;
            }
        } else if b[1] <= q[1] && o < 0.0 {
            voltas -= 1;
        }
    }
    if voltas != 0 {
        Location::Inside
    } else {
        Location::Outside
    }
}

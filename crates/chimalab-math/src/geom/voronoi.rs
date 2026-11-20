//! Diagrama de Voronoi 2-D como dual da triangulação de Delaunay.
//!
//! A célula de um ponto é a cadeia dos circuncentros dos triângulos em volta dele, em sentido
//! anti-horário. Pontos do fecho têm célula ilimitada: a cadeia começa e termina em raios
//! perpendiculares às arestas do fecho, apontando para fora.

use super::boolean::{BoolOp, Polygon, polygon_boolean};
use super::delaunay::{Delaunay, NONE};

/// Célula de Voronoi.
#[derive(Clone, Debug, PartialEq)]
pub struct VoronoiCell {
    /// Índice do ponto (sítio).
    pub site: usize,
    /// Vértices (circuncentros), em sentido anti-horário em volta do sítio.
    pub vertices: Vec<[f64; 2]>,
    /// Raios das células ilimitadas: direção que sai do primeiro vértice e direção que sai do
    /// último (vetores unitários). `None` para células limitadas.
    pub rays: Option<([f64; 2], [f64; 2])>,
}

impl VoronoiCell {
    /// Verdadeiro se a célula é limitada.
    pub fn is_bounded(&self) -> bool {
        self.rays.is_none()
    }
}

/// Circuncentro do triângulo (a, b, c) (não colineares).
pub fn circumcenter(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> [f64; 2] {
    // Coordenadas relativas a a reduzem o cancelamento.
    let (bx, by) = (b[0] - a[0], b[1] - a[1]);
    let (cx, cy) = (c[0] - a[0], c[1] - a[1]);
    let d = 2.0 * (bx * cy - by * cx);
    let (b2, c2) = (bx * bx + by * by, cx * cx + cy * cy);
    [a[0] + (cy * b2 - by * c2) / d, a[1] + (bx * c2 - cx * b2) / d]
}

fn unit(v: [f64; 2]) -> [f64; 2] {
    let n = v[0].hypot(v[1]);
    [v[0] / n, v[1] / n]
}

/// Células de Voronoi de todos os pontos distintos (os repetidos não têm célula própria).
pub fn voronoi(d: &Delaunay) -> Vec<VoronoiCell> {
    let pts = d.points();
    let tris = d.triangles();
    let nb = d.neighbors();
    let centros: Vec<[f64; 2]> = tris
        .iter()
        .map(|t| circumcenter(pts[t[0]], pts[t[1]], pts[t[2]]))
        .collect();
    // Um triângulo incidente por vértice.
    let mut incid = vec![NONE; pts.len()];
    for (k, t) in tris.iter().enumerate() {
        for &v in t {
            incid[v] = k;
        }
    }
    let mut cells = Vec::new();
    for (v, &t0) in incid.iter().enumerate() {
        if t0 == NONE {
            continue;
        }
        // Gira em volta de v: do triângulo t, o próximo no sentido anti-horário é o vizinho do
        // outro lado da aresta (v, sucessor de v em t).
        let pos = |t: usize| tris[t].iter().position(|&x| x == v).expect("v pertence a t");
        let ccw = |t: usize| nb[t][(pos(t) + 2) % 3];
        let cw = |t: usize| nb[t][(pos(t) + 1) % 3];
        let mut ordem = vec![t0];
        let mut t = t0;
        let mut fechado = false;
        loop {
            let n = ccw(t);
            if n == NONE {
                break;
            }
            if n == t0 {
                fechado = true;
                break;
            }
            ordem.push(n);
            t = n;
        }
        let rays = if fechado {
            None
        } else {
            // Célula ilimitada: completa girando no sentido horário a partir de t0.
            let mut ini = Vec::new();
            let mut t = t0;
            loop {
                let n = cw(t);
                if n == NONE {
                    break;
                }
                ini.push(n);
                t = n;
            }
            ini.reverse();
            ini.extend(ordem);
            ordem = ini;
            // Girar no sentido anti-horário em volta de v atravessa a aresta (v, sucessor).
            // O triângulo mais horário não tem vizinho do outro lado de (antecessor, v): essa é
            // uma aresta do fecho, e o mais anti-horário tem a aresta (v, sucessor) no fecho.
            // Cada raio é a normal externa da sua aresta (à direita, com o interior à esquerda).
            let first = ordem[0];
            let last = *ordem.last().expect("não vazio");
            let tf = tris[first];
            let u1 = tf[(tf.iter().position(|&x| x == v).expect("v") + 2) % 3];
            let tl = tris[last];
            let w = tl[(tl.iter().position(|&x| x == v).expect("v") + 1) % 3];
            let (pv, pu, pw) = (pts[v], pts[u1], pts[w]);
            let n1 = unit([pv[1] - pu[1], -(pv[0] - pu[0])]);
            let n2 = unit([pw[1] - pv[1], -(pw[0] - pv[0])]);
            Some((n1, n2))
        };
        cells.push(VoronoiCell {
            site: v,
            vertices: ordem.iter().map(|&t| centros[t]).collect(),
            rays,
        });
    }
    cells
}

/// Células recortadas pelo retângulo [x0, x1] × [y0, y1]: polígonos convexos que particionam o
/// retângulo (cada ponto do retângulo fica na célula do sítio mais próximo).
pub fn voronoi_clipped(d: &Delaunay, bbox: [f64; 4]) -> Vec<(usize, Polygon)> {
    let [x0, y0, x1, y1] = bbox;
    let caixa = Polygon::new(vec![vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]]).expect("finito");
    // Comprimento dos raios: bem maior que a caixa e que a nuvem de pontos.
    let mut ext = (x1 - x0).abs() + (y1 - y0).abs();
    for p in d.points() {
        ext = ext.max(p[0].abs() + p[1].abs());
    }
    let longe = 4.0 * ext + 1.0;
    voronoi(d)
        .into_iter()
        .map(|c| {
            let mut anel = c.vertices.clone();
            if let Some((r1, r2)) = c.rays {
                let (f, l) = (anel[0], *anel.last().expect("não vazio"));
                anel.insert(0, [f[0] + longe * r1[0], f[1] + longe * r1[1]]);
                anel.push([l[0] + longe * r2[0], l[1] + longe * r2[1]]);
                // Fecha pelo lado de fora: um ponto ainda mais longe na bissetriz dos raios
                // (a cunha no infinito é convexa, com ângulo menor que 180°).
                let m = unit([r1[0] + r2[0], r1[1] + r2[1]]);
                let s = d.points()[c.site];
                anel.push([s[0] + 3.0 * longe * m[0], s[1] + 3.0 * longe * m[1]]);
            }
            let p = Polygon::new(vec![anel]).expect("finito");
            (c.site, polygon_boolean(&p, &caixa, BoolOp::Intersection))
        })
        .collect()
}

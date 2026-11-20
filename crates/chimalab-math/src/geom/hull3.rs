//! Fecho convexo 3-D incremental com o predicado orient3d exato.
//!
//! Começa por um tetraedro não degenerado. Cada ponto novo remove as faces que o "veem" (o ponto
//! está do lado de fora do plano da face) e liga o horizonte (arestas entre faces vistas e não
//! vistas) ao ponto. Custo O(n·h) no pior caso, suficiente para dezenas de milhares de pontos;
//! uma versão com listas de conflito (Clarkson & Shor, 1989) fica para depois.

use std::collections::HashMap;

use super::GeomError;
use super::predicates::{orient2d, orient3d};

/// Faces triangulares do fecho (índices nos pontos), orientadas para fora: vistas de fora, os
/// vértices estão em sentido anti-horário.
///
/// Erros: `naoFinito`, `pontosInsuficientes` (menos de 4 pontos distintos), `degenerado` (todos
/// coplanares).
pub fn convex_hull_3d(p: &[[f64; 3]]) -> Result<Vec<[usize; 3]>, GeomError> {
    if p.iter().flatten().any(|v| !v.is_finite()) {
        return Err(GeomError::NonFinite);
    }
    let n = p.len();
    if n < 4 {
        return Err(GeomError::TooFewPoints { needed: 4, got: n });
    }
    // Tetraedro inicial: i0, i1 distintos; i2 não colinear; i3 não coplanar.
    let i0 = 0;
    let i1 = (1..n)
        .find(|&i| p[i] != p[i0])
        .ok_or(GeomError::TooFewPoints { needed: 4, got: 1 })?;
    // Colineares exatamente ⇔ as três projeções coordenadas são colineares (orient2d exato).
    let colinear = |i: usize| {
        let pr = |q: [f64; 3], a: usize, b: usize| [q[a], q[b]];
        [(0, 1), (1, 2), (0, 2)]
            .iter()
            .all(|&(a, b)| orient2d(pr(p[i0], a, b), pr(p[i1], a, b), pr(p[i], a, b)) == 0.0)
    };
    let i2 = (0..n)
        .find(|&i| !colinear(i))
        .ok_or(GeomError::Degenerate("todos os pontos são colineares"))?;
    let i3 = (0..n)
        .find(|&i| orient3d(p[i0], p[i1], p[i2], p[i]) != 0.0)
        .ok_or(GeomError::Degenerate("todos os pontos são coplanares"))?;
    // Orienta as 4 faces para fora (o quarto vértice fica do lado de dentro: orient3d > 0).
    let mut faces: Vec<[usize; 3]> = Vec::new();
    for (f, oposto) in [
        ([i0, i1, i2], i3),
        ([i0, i1, i3], i2),
        ([i0, i2, i3], i1),
        ([i1, i2, i3], i0),
    ] {
        faces.push(if orient3d(p[f[0]], p[f[1]], p[f[2]], p[oposto]) > 0.0 {
            f
        } else {
            [f[0], f[2], f[1]]
        });
    }
    for (i, q) in p.iter().enumerate() {
        if [i0, i1, i2, i3].contains(&i) {
            continue;
        }
        // Faces que veem q: q estritamente do lado de fora.
        let vistas: Vec<bool> = faces
            .iter()
            .map(|f| orient3d(p[f[0]], p[f[1]], p[f[2]], *q) < 0.0)
            .collect();
        if !vistas.iter().any(|&v| v) {
            continue; // dentro ou sobre o fecho
        }
        // Horizonte: arestas orientadas de faces vistas cuja reversa não pertence a face vista.
        let mut arestas_vistas: HashMap<(usize, usize), ()> = HashMap::new();
        for (f, &v) in faces.iter().zip(&vistas) {
            if v {
                for k in 0..3 {
                    arestas_vistas.insert((f[k], f[(k + 1) % 3]), ());
                }
            }
        }
        let mut novas = Vec::new();
        for (f, &v) in faces.iter().zip(&vistas) {
            if v {
                for k in 0..3 {
                    let (a, b) = (f[k], f[(k + 1) % 3]);
                    if !arestas_vistas.contains_key(&(b, a)) {
                        novas.push([a, b, i]);
                    }
                }
            }
        }
        faces = faces
            .into_iter()
            .zip(&vistas)
            .filter(|(_, v)| !**v)
            .map(|(f, _)| f)
            .collect();
        faces.extend(novas);
    }
    Ok(faces)
}

/// Volume do sólido limitado por faces orientadas para fora (teorema da divergência).
pub fn hull_volume(p: &[[f64; 3]], faces: &[[usize; 3]]) -> f64 {
    let o = faces.first().map_or([0.0; 3], |f| p[f[0]]);
    faces
        .iter()
        .map(|f| {
            let (a, b, c) = (sub(p[f[0]], o), sub(p[f[1]], o), sub(p[f[2]], o));
            let x = cross(b, c);
            (a[0] * x[0] + a[1] * x[1] + a[2] * x[2]) / 6.0
        })
        .sum()
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

//! Operações booleanas de polígonos (união, interseção, diferença, ou-exclusivo) por
//! sobreposição de arestas com classificação.
//!
//! 1. Cada anel é orientado pela profundidade de aninhamento: os de contorno ficam anti-horários
//!    e os buracos horários, de modo que a região fica sempre à esquerda das arestas.
//! 2. As arestas de A e B são cortadas em todos os pontos de interseção. Cada ponto é calculado
//!    uma vez e inserido nas duas arestas, com coordenadas idênticas.
//! 3. Cada pedaço é classificado pelo ponto médio: dentro ou fora do outro polígono (par-ímpar),
//!    ou compartilhado com um pedaço do outro, no mesmo sentido ou no oposto.
//! 4. A operação escolhe os pedaços, invertendo alguns, e eles são religados em anéis. Em
//!    vértices com várias saídas, toma a curva mais à direita, o que mantém os anéis simples.
//!
//! Custo O((n + m)² + k): suficiente para polígonos de até alguns milhares de vértices. Uma
//! varredura (Martínez, Rueda & Feito, 2009/2013) fica para entradas grandes. A entrada deve ter
//! anéis simples (sem autointerseção); anéis podem se tocar.

use std::collections::HashMap;

use super::predicates::orient2d;
use super::{GeomError, Location, point_in_polygon, polygon_area};

/// Região plana com anéis (regra par-ímpar: um ponto está dentro se um número ímpar de anéis o
/// contém). Depois de [`Polygon::new`], contornos são anti-horários e buracos horários.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    rings: Vec<Vec<[f64; 2]>>,
}

/// Operação booleana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoolOp {
    /// A ∪ B.
    Union,
    /// A ∩ B.
    Intersection,
    /// A − B.
    Difference,
    /// (A − B) ∪ (B − A).
    Xor,
}

impl Polygon {
    /// Cria a região a partir de anéis (fechados implicitamente; o último vértice não repete o
    /// primeiro). Remove vértices repetidos consecutivos e anéis sem área.
    ///
    /// Erros: `naoFinito`.
    pub fn new(rings: Vec<Vec<[f64; 2]>>) -> Result<Self, GeomError> {
        if rings.iter().flatten().flatten().any(|v| !v.is_finite()) {
            return Err(GeomError::NonFinite);
        }
        let mut limpos: Vec<Vec<[f64; 2]>> = Vec::new();
        for r in rings {
            let mut c: Vec<[f64; 2]> = Vec::with_capacity(r.len());
            for p in r {
                if c.last() != Some(&p) {
                    c.push(p);
                }
            }
            while c.len() > 1 && c.first() == c.last() {
                c.pop();
            }
            if c.len() >= 3 && polygon_area(&c) != 0.0 {
                limpos.push(c);
            }
        }
        // Profundidade de aninhamento pelo primeiro vértice que não está sobre outro anel.
        let mut out = Vec::with_capacity(limpos.len());
        for (i, r) in limpos.iter().enumerate() {
            let mut prof = 0;
            for (j, s) in limpos.iter().enumerate() {
                if i == j {
                    continue;
                }
                let dentro = r
                    .iter()
                    .map(|p| point_in_polygon(*p, s))
                    .find(|l| *l != Location::Boundary)
                    .unwrap_or(Location::Outside);
                if dentro == Location::Inside {
                    prof += 1;
                }
            }
            let mut r = r.clone();
            let ccw = polygon_area(&r) > 0.0;
            if ccw != (prof % 2 == 0) {
                r.reverse();
            }
            out.push(r);
        }
        Ok(Self { rings: out })
    }

    /// Anéis orientados.
    pub fn rings(&self) -> &[Vec<[f64; 2]>] {
        &self.rings
    }

    /// Área da região (soma das áreas com sinal dos anéis orientados).
    pub fn area(&self) -> f64 {
        self.rings.iter().map(|r| polygon_area(r)).sum()
    }

    /// Posição de q (par-ímpar sobre os anéis; borda se estiver sobre algum anel).
    pub fn locate(&self, q: [f64; 2]) -> Location {
        let mut dentro = false;
        for r in &self.rings {
            match point_in_polygon(q, r) {
                Location::Boundary => return Location::Boundary,
                Location::Inside => dentro = !dentro,
                Location::Outside => {}
            }
        }
        if dentro {
            Location::Inside
        } else {
            Location::Outside
        }
    }

    fn edges(&self) -> Vec<([f64; 2], [f64; 2])> {
        self.rings
            .iter()
            .flat_map(|r| (0..r.len()).map(move |i| (r[i], r[(i + 1) % r.len()])))
            .collect()
    }
}

/// Interseções do segmento p→q com r→s: devolve os pontos (com o parâmetro em p→q e em r→s).
/// Colineares: as pontas de cada um que caem no outro.
fn crossings(p: [f64; 2], q: [f64; 2], r: [f64; 2], s: [f64; 2]) -> Vec<([f64; 2], f64, f64)> {
    let (o1, o2) = (orient2d(p, q, r), orient2d(p, q, s));
    let (o3, o4) = (orient2d(r, s, p), orient2d(r, s, q));
    let param = |a: [f64; 2], b: [f64; 2], x: [f64; 2]| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        if dx.abs() >= dy.abs() {
            (x[0] - a[0]) / dx
        } else {
            (x[1] - a[1]) / dy
        }
    };
    let mut out = Vec::new();
    if o1 == 0.0 && o2 == 0.0 {
        // Colineares: pontas dentro do outro segmento (inclusive extremos).
        for x in [r, s] {
            let t = param(p, q, x);
            if (0.0..=1.0).contains(&t) {
                out.push((x, t, param(r, s, x)));
            }
        }
        for x in [p, q] {
            let u = param(r, s, x);
            if (0.0..=1.0).contains(&u) {
                out.push((x, param(p, q, x), u));
            }
        }
        return out;
    }
    if o1 * o2 > 0.0 || o3 * o4 > 0.0 {
        return out;
    }
    // Ponto de contato numa ponta: usa a própria ponta (coordenadas exatas).
    let x = if o1 == 0.0 {
        r
    } else if o2 == 0.0 {
        s
    } else if o3 == 0.0 {
        p
    } else if o4 == 0.0 {
        q
    } else {
        // Cruzamento próprio: interpolação ao longo de r→s com |o1|/(|o1| + |o2|) em ponto flutuante.
        let d1 = (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
        let d2 = (q[0] - p[0]) * (s[1] - p[1]) - (q[1] - p[1]) * (s[0] - p[0]);
        let u = d1 / (d1 - d2);
        let x = [r[0] + u * (s[0] - r[0]), r[1] + u * (s[1] - r[1])];
        // O cruzamento é próprio (predicados exatos), mas o ponto arredondado pode cair numa
        // ponta, além dela, ou a poucos ulp dela. Então usa-se a própria ponta (a mais próxima,
        // a até 32 ulp da escala das coordenadas), para os dois segmentos receberem o mesmo corte
        // e não surgirem dois vértices quase iguais.
        // Regressões:
        // - um vértice de A quase sobre uma aresta de B dava t ≥ 1 em A (corte descartado) e
        //   u ∈ (0, 1) em B (corte feito); a cadeia quebrava e o anel saía fechado por uma
        //   corda, com 60% da área perdida;
        // - um cruzamento a 1 ulp de um vértice criava um vértice quase duplicado, com o mesmo
        //   efeito (proptest de células de Voronoi).
        let escala = [p, q, r, s].iter().flatten().fold(0.0f64, |m, v| m.max(v.abs()));
        let tol = 32.0 * f64::EPSILON * escala;
        let dist = |a: [f64; 2]| (a[0] - x[0]).abs().max((a[1] - x[1]).abs());
        let perto = [p, q, r, s]
            .into_iter()
            .filter(|&c| dist(c) <= tol)
            .min_by(|&a, &b| dist(a).total_cmp(&dist(b)));
        // Sem ponta próxima, os testes de parâmetro são defensivos: segmentos quase paralelos,
        // em que o ponto calculado pode se afastar mais que a tolerância.
        let (t, uu) = (param(p, q, x), param(r, s, x));
        if let Some(c) = perto {
            c
        } else if t <= 0.0 {
            p
        } else if t >= 1.0 {
            q
        } else if uu <= 0.0 {
            r
        } else if uu >= 1.0 {
            s
        } else {
            x
        }
    };
    out.push((x, param(p, q, x), param(r, s, x)));
    out
}

/// Corta as arestas nos pontos dados (parâmetro, ponto) e devolve os pedaços.
fn split(edges: &[([f64; 2], [f64; 2])], cortes: &[Vec<(f64, [f64; 2])>]) -> Vec<([f64; 2], [f64; 2])> {
    let mut out = Vec::new();
    for (e, c) in edges.iter().zip(cortes) {
        let mut pts: Vec<(f64, [f64; 2])> = c.iter().filter(|(t, _)| *t > 0.0 && *t < 1.0).copied().collect();
        pts.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut cur = e.0;
        for (_, x) in pts {
            if x != cur && x != e.1 {
                out.push((cur, x));
                cur = x;
            }
        }
        out.push((cur, e.1));
    }
    out
}

/// A op B.
pub fn polygon_boolean(a: &Polygon, b: &Polygon, op: BoolOp) -> Polygon {
    let (ea, eb) = (a.edges(), b.edges());
    let mut ca: Vec<Vec<(f64, [f64; 2])>> = vec![Vec::new(); ea.len()];
    let mut cb: Vec<Vec<(f64, [f64; 2])>> = vec![Vec::new(); eb.len()];
    for (i, &(p, q)) in ea.iter().enumerate() {
        for (j, &(r, s)) in eb.iter().enumerate() {
            for (x, t, u) in crossings(p, q, r, s) {
                ca[i].push((t, x));
                cb[j].push((u, x));
            }
        }
    }
    let pa = split(&ea, &ca);
    let pb = split(&eb, &cb);
    // Pedaços compartilhados (mesmas pontas): sentido igual ou oposto.
    let mut chave_b: HashMap<([u64; 2], [u64; 2]), usize> = HashMap::new();
    let bits = |p: [f64; 2]| [p[0].to_bits(), p[1].to_bits()];
    for (j, &(r, s)) in pb.iter().enumerate() {
        chave_b.insert((bits(r), bits(s)), j);
    }
    let meio = |p: [f64; 2], q: [f64; 2]| [p[0] / 2.0 + q[0] / 2.0, p[1] / 2.0 + q[1] / 2.0];
    let mut sel: Vec<([f64; 2], [f64; 2])> = Vec::new();
    let mut usado_b = vec![false; pb.len()];
    for &(p, q) in &pa {
        let mesmo = chave_b.get(&(bits(p), bits(q))).copied();
        let oposto = chave_b.get(&(bits(q), bits(p))).copied();
        if let Some(j) = mesmo {
            usado_b[j] = true;
            if matches!(op, BoolOp::Union | BoolOp::Intersection) {
                sel.push((p, q));
            }
            continue;
        }
        if let Some(j) = oposto {
            usado_b[j] = true;
            if op == BoolOp::Difference {
                sel.push((p, q));
            }
            continue;
        }
        let fora = b.locate(meio(p, q)) != Location::Inside;
        match (op, fora) {
            (BoolOp::Union | BoolOp::Difference | BoolOp::Xor, true) => sel.push((p, q)),
            (BoolOp::Intersection, false) => sel.push((p, q)),
            (BoolOp::Xor, false) => sel.push((q, p)),
            _ => {}
        }
    }
    for (j, &(r, s)) in pb.iter().enumerate() {
        if usado_b[j] {
            continue;
        }
        let fora = a.locate(meio(r, s)) != Location::Inside;
        match (op, fora) {
            (BoolOp::Union | BoolOp::Xor, true) => sel.push((r, s)),
            (BoolOp::Intersection, false) => sel.push((r, s)),
            (BoolOp::Difference | BoolOp::Xor, false) => sel.push((s, r)),
            _ => {}
        }
    }
    Polygon { rings: assemble(sel) }
}

/// Religa arestas orientadas em anéis: em cada vértice, segue a saída mais à direita em relação
/// à entrada.
fn assemble(edges: Vec<([f64; 2], [f64; 2])>) -> Vec<Vec<[f64; 2]>> {
    let bits = |p: [f64; 2]| [p[0].to_bits(), p[1].to_bits()];
    let mut saidas: HashMap<[u64; 2], Vec<usize>> = HashMap::new();
    for (k, e) in edges.iter().enumerate() {
        saidas.entry(bits(e.0)).or_default().push(k);
    }
    let mut usado = vec![false; edges.len()];
    let mut aneis = Vec::new();
    for inicio in 0..edges.len() {
        if usado[inicio] {
            continue;
        }
        let mut anel = vec![edges[inicio].0];
        usado[inicio] = true;
        let mut atual = inicio;
        loop {
            let (p, q) = edges[atual];
            if q == anel[0] {
                break;
            }
            anel.push(q);
            let cands: Vec<usize> = saidas
                .get(&bits(q))
                .map_or(Vec::new(), |v| v.iter().copied().filter(|&k| !usado[k]).collect());
            let Some(prox) = cands.into_iter().min_by(|&x, &y| {
                // Ângulo de giro em (−π, π]: o menor é a curva mais à direita.
                let giro = |k: usize| {
                    let r = edges[k].1;
                    let (ux, uy) = (q[0] - p[0], q[1] - p[1]);
                    let (vx, vy) = (r[0] - q[0], r[1] - q[1]);
                    (ux * vy - uy * vx).atan2(ux * vx + uy * vy)
                };
                giro(x).total_cmp(&giro(y))
            }) else {
                break;
            };
            usado[prox] = true;
            atual = prox;
        }
        if anel.len() >= 3 && polygon_area(&anel) != 0.0 {
            aneis.push(anel);
        }
    }
    aneis
}

//! F047 — geometria computacional (incremento 1, 2-D): aceitação, predicados exatos,
//! propriedades, bordas e erros.

use chimalab_math::geom::{
    BoolOp, Delaunay, GeomError, Location, NONE, Polygon, circumcenter, convex_hull, incircle, orient2d,
    point_in_polygon, polygon_area, polygon_boolean, polygon_centroid, polygon_perimeter, voronoi,
    voronoi_clipped,
};
use chimalab_math::random::{Algorithm, Rng};
use proptest::prelude::*;

fn aleatorios(n: usize, semente: u64) -> Vec<[f64; 2]> {
    let mut r = Rng::new(Algorithm::Philox, semente);
    (0..n).map(|_| [r.uniform(), r.uniform()]).collect()
}

/// Pontos sobre a fronteira do fecho (inclusive os colineares nas arestas).
fn na_borda_do_fecho(pts: &[[f64; 2]]) -> usize {
    let h: Vec<[f64; 2]> = convex_hull(pts).unwrap().iter().map(|&i| pts[i]).collect();
    let mut distintos = pts.to_vec();
    distintos.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    distintos.dedup();
    distintos
        .iter()
        .filter(|p| point_in_polygon(**p, &h) == Location::Boundary)
        .count()
}

fn valida(d: &Delaunay, pts: &[[f64; 2]]) {
    assert!(d.is_delaunay());
    // Todos anti-horários.
    for t in d.triangles() {
        assert!(orient2d(pts[t[0]], pts[t[1]], pts[t[2]]) > 0.0);
    }
    // Euler: 2n − 2 − h triângulos, com n distintos e h na fronteira do fecho.
    let n = pts.len() - d.duplicates().len();
    let h = na_borda_do_fecho(pts);
    assert_eq!(d.triangles().len(), 2 * n - 2 - h);
    // Vizinhança simétrica.
    for (t, nb) in d.neighbors().iter().enumerate() {
        for &v in nb {
            if v != NONE {
                assert!(d.neighbors()[v].contains(&t));
            }
        }
    }
}

#[test]
fn ca1_delaunay_de_1e5_pontos_satisfaz_circulo_vazio() {
    let pts = aleatorios(100_000, 2026);
    let t = std::time::Instant::now();
    let d = Delaunay::new(&pts).unwrap();
    println!("1e5 pontos: {:?} (debug)", t.elapsed());
    valida(&d, &pts);
    // Todo ponto é vértice de algum triângulo.
    let mut usado = vec![false; pts.len()];
    for t in d.triangles() {
        for &v in t {
            usado[v] = true;
        }
    }
    assert!(usado.iter().all(|&u| u));
}

#[test]
fn predicados_exatos_em_casos_degenerados() {
    // Colineares exatos com deslocamento grande: o cálculo ingênuo em f64 tem ruído; o exato dá 0.
    let o = 1e15 + 0.5;
    for k in 0..50 {
        let k = k as f64;
        let a = [o, o];
        let b = [o + 1.0 + k, o + 1.0 + k];
        let c = [o + 7.0 * k + 3.0, o + 7.0 * k + 3.0];
        assert_eq!(orient2d(a, b, c), 0.0);
        // Um ulp para cima: c fica à esquerda de a→b (anti-horário).
        let up = f64::from_bits(c[1].to_bits() + 1);
        assert_eq!(orient2d(a, b, [c[0], up]), 1.0);
        let dn = f64::from_bits(c[1].to_bits() - 1);
        assert_eq!(orient2d(a, b, [c[0], dn]), -1.0);
    }
    // Cocirculares exatos (x² + y² = 25 em inteiros) deslocados por 1e6: incircle = 0.
    let o = 1e6;
    let circ = [
        [5.0, 0.0],
        [3.0, 4.0],
        [0.0, 5.0],
        [-4.0, 3.0],
        [-5.0, 0.0],
        [-3.0, -4.0],
        [4.0, -3.0],
    ];
    let p = |v: [f64; 2]| [o + v[0], o + v[1]];
    for i in 0..circ.len() {
        for j in 0..circ.len() {
            for k in 0..circ.len() {
                for l in 0..circ.len() {
                    let (a, b, c, d) = (p(circ[i]), p(circ[j]), p(circ[k]), p(circ[l]));
                    assert_eq!(incircle(a, b, c, d), 0.0, "{i}{j}{k}{l}");
                }
            }
        }
    }
    // (5, 0) deslocado um ulp para fora (x maior) fica fora do círculo; para dentro, dentro.
    let (a, b, c) = (p(circ[1]), p(circ[2]), p(circ[4])); // anti-horários
    assert!(orient2d(a, b, c) > 0.0);
    let x = o + 5.0;
    assert_eq!(incircle(a, b, c, [f64::from_bits(x.to_bits() + 1), o]), -1.0);
    assert_eq!(incircle(a, b, c, [f64::from_bits(x.to_bits() - 1), o]), 1.0);
    // Inverter a orientação inverte o sinal.
    assert_eq!(incircle(a, c, b, [f64::from_bits(x.to_bits() - 1), o]), -1.0);
}

#[test]
fn delaunay_grade_cocircular_e_casos_pequenos() {
    // Grade 60×60: todos os quadrados têm 4 pontos cocirculares.
    let g: Vec<[f64; 2]> = (0..3600).map(|k| [(k % 60) as f64, (k / 60) as f64]).collect();
    let d = Delaunay::new(&g).unwrap();
    valida(&d, &g);
    // Grade deslocada para longe da origem e reduzida: predicados exatos continuam decidindo.
    let g2: Vec<[f64; 2]> = g
        .iter()
        .map(|p| [1e9 + p[0] * 1e-3, -1e9 + p[1] * 1e-3])
        .collect();
    let d2 = Delaunay::new(&g2).unwrap();
    valida(&d2, &g2);
    // Três pontos: um triângulo, anti-horário mesmo dados em sentido horário.
    let tri = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]];
    let d = Delaunay::new(&tri).unwrap();
    assert_eq!(d.triangles().len(), 1);
    assert_eq!(d.neighbors()[0], [NONE; 3]);
    assert_eq!(d.points().len(), 3);
    // Quadrado: dois triângulos.
    let q = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    assert_eq!(Delaunay::new(&q).unwrap().triangles().len(), 2);
    // Repetidos: ignorados e registrados.
    let r = [
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 0.0],
        [0.3, 0.3],
    ];
    let d = Delaunay::new(&r).unwrap();
    assert_eq!(d.duplicates().len(), 2);
    for &(dup, orig) in d.duplicates() {
        assert_eq!(r[dup], r[orig]);
        assert_ne!(dup, orig);
    }
    valida(&d, &r);
    // Colineares seguidos de um ponto fora: os colineares viram arestas do fecho.
    let mut c: Vec<[f64; 2]> = (0..20).map(|k| [k as f64, 2.0 * k as f64]).collect();
    c.push([5.0, 0.0]);
    let d = Delaunay::new(&c).unwrap();
    valida(&d, &c);
    // Pontos fora do fecho entrando em sequência (fecho cresce): círculo de pontos.
    let circ: Vec<[f64; 2]> = (0..200)
        .map(|k| {
            let a = k as f64 * 0.1;
            [
                a.cos() * (1.0 + 0.001 * k as f64),
                a.sin() * (1.0 + 0.001 * k as f64),
            ]
        })
        .collect();
    valida(&Delaunay::new(&circ).unwrap(), &circ);
}

#[test]
fn localizacao_e_baricentricas() {
    let pts = aleatorios(2000, 7);
    let d = Delaunay::new(&pts).unwrap();
    let mut r = Rng::new(Algorithm::Philox, 99);
    for _ in 0..500 {
        let q = [r.uniform() * 1.2 - 0.1, r.uniform() * 1.2 - 0.1];
        match d.locate(q) {
            Some((t, l)) => {
                let v = d.triangles()[t];
                // Reconstrução de q pelas baricêntricas, e todas ≥ 0 (a menos de arredondamento).
                let x = l[0] * pts[v[0]][0] + l[1] * pts[v[1]][0] + l[2] * pts[v[2]][0];
                let y = l[0] * pts[v[0]][1] + l[1] * pts[v[1]][1] + l[2] * pts[v[2]][1];
                assert!((x - q[0]).abs() <= 1e-12 && (y - q[1]).abs() <= 1e-12);
                assert!(l.iter().all(|&w| w >= -1e-12));
            }
            None => {
                let h: Vec<[f64; 2]> = convex_hull(&pts).unwrap().iter().map(|&i| pts[i]).collect();
                assert_eq!(point_in_polygon(q, &h), Location::Outside);
            }
        }
    }
    // Nos vértices: baricêntrica 1 no próprio vértice.
    let (t, l) = d.locate(pts[17]).unwrap();
    let k = d.triangles()[t].iter().position(|&v| v == 17).unwrap();
    assert!((l[k] - 1.0).abs() <= 1e-12);
    assert!(d.locate([f64::NAN, 0.5]).is_none());
}

#[test]
fn fecho_convexo() {
    let pts = [
        [0.0, 0.0],
        [2.0, 0.0],
        [1.0, 0.0],
        [2.0, 2.0],
        [1.0, 1.0],
        [0.0, 2.0],
        [0.0, 1.0],
        [2.0, 2.0],
    ];
    let h = convex_hull(&pts).unwrap();
    // Anti-horário a partir do menor x (e menor y): sem colineares nem repetidos.
    assert_eq!(h, vec![0, 1, 3, 5]);
    assert_eq!(convex_hull(&[]).unwrap(), Vec::<usize>::new());
    assert_eq!(convex_hull(&[[1.0, 1.0], [1.0, 1.0]]).unwrap(), vec![0]);
    // Colineares: os dois extremos.
    assert_eq!(
        convex_hull(&[[0.0, 0.0], [2.0, 2.0], [1.0, 1.0]]).unwrap(),
        vec![0, 1]
    );
    assert_eq!(
        convex_hull(&[[0.0, f64::NAN]]).unwrap_err().id(),
        "chimalab:geom:naoFinito"
    );
}

#[test]
fn poligonos() {
    // L de área 3 (unidades inteiras: exato).
    let l = [
        [0.0, 0.0],
        [2.0, 0.0],
        [2.0, 1.0],
        [1.0, 1.0],
        [1.0, 2.0],
        [0.0, 2.0],
    ];
    assert_eq!(polygon_area(&l), 3.0);
    let mut horario = l;
    horario.reverse();
    assert_eq!(polygon_area(&horario), -3.0);
    assert_eq!(polygon_perimeter(&l), 8.0);
    // Centroide do L: média ponderada de [0,2]×[0,1] (área 2, (1, 0,5)) e [0,1]×[1,2] (área 1,
    // (0,5, 1,5)): (5/6, 5/6).
    let c = polygon_centroid(&l).unwrap();
    assert!((c[0] - 5.0 / 6.0).abs() <= 1e-15 && (c[1] - 5.0 / 6.0).abs() <= 1e-15);
    // Longe da origem: a área continua exata (coordenadas relativas ao primeiro vértice).
    let longe: Vec<[f64; 2]> = l.iter().map(|p| [p[0] + 1e9, p[1] - 1e9]).collect();
    assert_eq!(polygon_area(&longe), 3.0);
    // Ponto no polígono (côncavo), borda e vértice.
    assert_eq!(point_in_polygon([0.5, 1.5], &l), Location::Inside);
    assert_eq!(point_in_polygon([1.5, 1.5], &l), Location::Outside);
    assert_eq!(point_in_polygon([1.5, 1.0], &l), Location::Boundary);
    assert_eq!(point_in_polygon([2.0, 0.0], &l), Location::Boundary);
    assert_eq!(point_in_polygon([0.5, 1.5], &horario), Location::Inside);
    assert_eq!(point_in_polygon([f64::NAN, 0.5], &l), Location::Outside);
    assert_eq!(point_in_polygon([0.0, 0.0], &[]), Location::Outside);
    // Estrela (autointersectante): o centro tem número de voltas 2, dentro.
    let estrela: Vec<[f64; 2]> = (0..5)
        .map(|k| {
            let a = std::f64::consts::FRAC_PI_2 + 4.0 * std::f64::consts::PI * k as f64 / 5.0;
            [a.cos(), a.sin()]
        })
        .collect();
    assert_eq!(point_in_polygon([0.0, 0.0], &estrela), Location::Inside);
    // Bordas de entrada.
    assert_eq!(polygon_area(&[[0.0, 0.0], [1.0, 1.0]]), 0.0);
    assert_eq!(
        polygon_centroid(&[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]])
            .unwrap_err()
            .id(),
        "chimalab:geom:degenerado"
    );
}

#[test]
fn erros_com_identificador() {
    let casos: Vec<(GeomError, &str)> = vec![
        (
            Delaunay::new(&[[0.0, 0.0], [1.0, 0.0]]).unwrap_err(),
            "chimalab:geom:pontosInsuficientes",
        ),
        (
            Delaunay::new(&[[0.0, 0.0]; 5]).unwrap_err(),
            "chimalab:geom:pontosInsuficientes",
        ),
        (
            Delaunay::new(&[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]).unwrap_err(),
            "chimalab:geom:degenerado",
        ),
        (
            Delaunay::new(&[[0.0, 0.0], [1.0, f64::INFINITY], [2.0, 0.0]]).unwrap_err(),
            "chimalab:geom:naoFinito",
        ),
    ];
    for (e, id) in casos {
        assert_eq!(e.id(), id, "{e}");
        assert!(!e.to_string().is_empty());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Pontos inteiros pequenos (muitos colineares, cocirculares e repetidos): triangulação
    /// válida sempre que não forem todos colineares.
    #[test]
    fn delaunay_em_pontos_degenerados(pts in prop::collection::vec((0i32..8, 0i32..8), 3..60)) {
        let p: Vec<[f64; 2]> = pts.iter().map(|&(x, y)| [x as f64, y as f64]).collect();
        match Delaunay::new(&p) {
            Ok(d) => valida(&d, &p),
            Err(e) => {
                let esperado = matches!(e, GeomError::Degenerate(_) | GeomError::TooFewPoints { .. });
                prop_assert!(esperado, "{}", e);
            }
        }
    }

    /// Fecho convexo contra força bruta: convexo, e nenhum ponto fora.
    #[test]
    fn fecho_contra_forca_bruta(pts in prop::collection::vec((-50i32..50, -50i32..50), 3..40)) {
        let p: Vec<[f64; 2]> = pts.iter().map(|&(x, y)| [x as f64, y as f64]).collect();
        let h = convex_hull(&p).unwrap();
        if h.len() >= 3 {
            let poly: Vec<[f64; 2]> = h.iter().map(|&i| p[i]).collect();
            for i in 0..poly.len() {
                let (a, b, c) = (poly[i], poly[(i + 1) % poly.len()], poly[(i + 2) % poly.len()]);
                prop_assert!(orient2d(a, b, c) > 0.0);
            }
            for q in &p {
                prop_assert_ne!(point_in_polygon(*q, &poly), Location::Outside);
            }
        }
    }
}

// ================================================================ booleanas (incremento 3)

fn quadrado(x0: f64, y0: f64, l: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x0 + l, y0], [x0 + l, y0 + l], [x0, y0 + l]]
}

fn pol(aneis: Vec<Vec<[f64; 2]>>) -> Polygon {
    Polygon::new(aneis).unwrap()
}

#[test]
fn ca2_uniao_e_intersecao_com_area_correta() {
    // Dois quadrados 2×2 sobrepostos em 1×1.
    let a = pol(vec![quadrado(0.0, 0.0, 2.0)]);
    let b = pol(vec![quadrado(1.0, 1.0, 2.0)]);
    let area = |op| polygon_boolean(&a, &b, op).area();
    assert_eq!(area(BoolOp::Union), 7.0);
    assert_eq!(area(BoolOp::Intersection), 1.0);
    assert_eq!(area(BoolOp::Difference), 3.0);
    assert_eq!(area(BoolOp::Xor), 6.0);
    // Quadrado [0,2]² e triângulo (1,−1), (3,1), (1,1): dentro do quadrado, o triângulo cobre
    // x ∈ [1, 2], y ∈ [0, 1] (a hipotenusa y = x − 2 fica abaixo de y = 0 ali): área 1.
    let t = pol(vec![vec![[1.0, -1.0], [3.0, 1.0], [1.0, 1.0]]]);
    let i = polygon_boolean(&a, &t, BoolOp::Intersection);
    assert!((i.area() - 1.0).abs() <= 1e-15, "{}", i.area());
    assert!((polygon_boolean(&a, &t, BoolOp::Union).area() - (4.0 + 2.0 - 1.0)).abs() <= 1e-15);
    // Lados compartilhados: quadrados vizinhos unidos viram um retângulo (sem aresta interna).
    let c = pol(vec![quadrado(2.0, 0.0, 2.0)]);
    let u = polygon_boolean(&a, &c, BoolOp::Union);
    assert_eq!(u.area(), 8.0);
    assert_eq!(u.rings().len(), 1);
    assert_eq!(polygon_boolean(&a, &c, BoolOp::Intersection).area(), 0.0);
    assert_eq!(polygon_boolean(&a, &c, BoolOp::Difference).area(), 4.0);
    // Idênticos.
    assert_eq!(polygon_boolean(&a, &a, BoolOp::Union).area(), 4.0);
    assert_eq!(polygon_boolean(&a, &a, BoolOp::Intersection).area(), 4.0);
    assert_eq!(polygon_boolean(&a, &a, BoolOp::Difference).area(), 0.0);
    assert_eq!(polygon_boolean(&a, &a, BoolOp::Xor).area(), 0.0);
    // Contido: buraco na diferença.
    let dentro = pol(vec![quadrado(0.5, 0.5, 1.0)]);
    let d = polygon_boolean(&a, &dentro, BoolOp::Difference);
    assert_eq!(d.area(), 3.0);
    assert_eq!(d.rings().len(), 2);
    assert_eq!(d.locate([1.0, 1.0]), Location::Outside);
    assert_eq!(d.locate([0.25, 0.25]), Location::Inside);
    // Disjuntos.
    let longe = pol(vec![quadrado(10.0, 10.0, 1.0)]);
    assert_eq!(polygon_boolean(&a, &longe, BoolOp::Union).area(), 5.0);
    assert_eq!(polygon_boolean(&a, &longe, BoolOp::Intersection).area(), 0.0);
}

#[test]
fn regioes_com_buracos_e_orientacao() {
    // Anel (quadrado 4×4 com buraco 2×2), dados em qualquer orientação.
    let mut fora = quadrado(0.0, 0.0, 4.0);
    fora.reverse();
    let anel = pol(vec![fora, quadrado(1.0, 1.0, 2.0)]);
    assert_eq!(anel.area(), 12.0);
    assert!(polygon_area(&anel.rings()[0]) > 0.0 && polygon_area(&anel.rings()[1]) < 0.0);
    assert_eq!(anel.locate([2.0, 2.0]), Location::Outside);
    assert_eq!(anel.locate([0.5, 2.0]), Location::Inside);
    assert_eq!(anel.locate([1.0, 2.0]), Location::Boundary);
    // Um quadrado que tapa o buraco: a união preenche tudo.
    let tampa = pol(vec![quadrado(0.5, 0.5, 3.0)]);
    let u = polygon_boolean(&anel, &tampa, BoolOp::Union);
    assert_eq!(u.area(), 16.0);
    let i = polygon_boolean(&anel, &tampa, BoolOp::Intersection);
    assert_eq!(i.area(), 9.0 - 4.0);
    // Limpeza: vértices repetidos e anéis degenerados.
    let p = pol(vec![
        vec![[0.0, 0.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]],
        vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]],
    ]);
    assert_eq!(p.rings().len(), 1);
    assert_eq!(p.area(), 0.5);
    assert_eq!(
        Polygon::new(vec![vec![[0.0, f64::NAN]; 3]]).unwrap_err().id(),
        "chimalab:geom:naoFinito"
    );
    // Vazio.
    let vazio = pol(vec![]);
    assert_eq!(polygon_boolean(&anel, &vazio, BoolOp::Union).area(), 12.0);
    assert_eq!(polygon_boolean(&anel, &vazio, BoolOp::Intersection).area(), 0.0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Polígonos convexos aleatórios (fechos de pontos inteiros): inclusão–exclusão e
    /// pertinência ponto a ponto contra A e B.
    #[test]
    fn booleanas_inclusao_exclusao(
        pa in prop::collection::vec((-20i32..20, -20i32..20), 3..15),
        pb in prop::collection::vec((-20i32..20, -20i32..20), 3..15),
        pts in prop::collection::vec((-21.0f64..21.0, -21.0f64..21.0), 50),
    ) {
        let fecho = |v: &[(i32, i32)]| {
            let p: Vec<[f64; 2]> = v.iter().map(|&(x, y)| [x as f64, y as f64]).collect();
            convex_hull(&p).unwrap().iter().map(|&i| p[i]).collect::<Vec<_>>()
        };
        let (ha, hb) = (fecho(&pa), fecho(&pb));
        prop_assume!(ha.len() >= 3 && hb.len() >= 3);
        let (a, b) = (pol(vec![ha]), pol(vec![hb]));
        let un = polygon_boolean(&a, &b, BoolOp::Union);
        let it = polygon_boolean(&a, &b, BoolOp::Intersection);
        let df = polygon_boolean(&a, &b, BoolOp::Difference);
        let xo = polygon_boolean(&a, &b, BoolOp::Xor);
        let tol = 1e-9 * (a.area() + b.area());
        prop_assert!((un.area() - (a.area() + b.area() - it.area())).abs() <= tol);
        prop_assert!((df.area() - (a.area() - it.area())).abs() <= tol);
        prop_assert!((xo.area() - (un.area() - it.area())).abs() <= tol);
        for &(x, y) in &pts {
            let q = [x, y];
            let (la, lb) = (a.locate(q), b.locate(q));
            if la == Location::Boundary || lb == Location::Boundary {
                continue;
            }
            let (ia, ib) = (la == Location::Inside, lb == Location::Inside);
            let checar = |p: &Polygon, esperado: bool| {
                let l = p.locate(q);
                l == Location::Boundary || (l == Location::Inside) == esperado
            };
            prop_assert!(checar(&un, ia || ib));
            prop_assert!(checar(&it, ia && ib));
            prop_assert!(checar(&df, ia && !ib));
            prop_assert!(checar(&xo, ia != ib));
        }
    }
}

// ================================================================ Voronoi (incremento 2)

fn mais_proximo(pts: &[[f64; 2]], q: [f64; 2]) -> (usize, f64) {
    pts.iter()
        .enumerate()
        .map(|(i, p)| (i, (p[0] - q[0]).hypot(p[1] - q[1])))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap()
}

#[test]
fn voronoi_vertices_raios_e_particao() {
    let pts = aleatorios(300, 31);
    let d = Delaunay::new(&pts).unwrap();
    let cells = voronoi(&d);
    assert_eq!(cells.len(), 300);
    let h = convex_hull(&pts).unwrap();
    for c in &cells {
        let s = pts[c.site];
        // Cada vértice é equidistante do sítio e não há sítio mais próximo (círculo vazio).
        for v in &c.vertices {
            let ds = (s[0] - v[0]).hypot(s[1] - v[1]);
            let (_, dmin) = mais_proximo(&pts, *v);
            assert!((ds - dmin).abs() <= 1e-9 * ds.max(1.0), "{} {ds} {dmin}", c.site);
        }
        // Células ilimitadas exatamente nos pontos do fecho.
        assert_eq!(!c.is_bounded(), h.contains(&c.site), "{}", c.site);
        if let Some((r1, r2)) = c.rays {
            for (v, r) in [(c.vertices[0], r1), (*c.vertices.last().unwrap(), r2)] {
                // Ao longo do raio, o sítio continua entre os mais próximos (empate com o vizinho do fecho).
                let q = [v[0] + 0.7 * r[0], v[1] + 0.7 * r[1]];
                let ds = (s[0] - q[0]).hypot(s[1] - q[1]);
                let (_, dmin) = mais_proximo(&pts, q);
                assert!((ds - dmin).abs() <= 1e-9 * ds.max(1.0), "raio de {}", c.site);
                assert!((r[0].hypot(r[1]) - 1.0).abs() <= 1e-15);
            }
        }
    }
    // Recortadas: particionam a caixa, e cada ponto da caixa está na célula do sítio mais próximo.
    let caixa = [-0.2, -0.1, 1.1, 1.3];
    let rec = voronoi_clipped(&d, caixa);
    let soma: f64 = rec.iter().map(|(_, p)| p.area()).sum();
    assert!((soma - 1.3 * 1.4).abs() <= 1e-9, "{soma}");
    let mut r = Rng::new(Algorithm::Philox, 77);
    for _ in 0..300 {
        let q = [-0.2 + 1.3 * r.uniform(), -0.1 + 1.4 * r.uniform()];
        let (perto, _) = mais_proximo(&pts, q);
        let (_, cel) = rec.iter().find(|(site, _)| *site == perto).unwrap();
        assert_ne!(cel.locate(q), Location::Outside, "{q:?}");
    }
    // Circuncentro equidistante.
    let c = circumcenter([0.0, 0.0], [4.0, 0.0], [0.0, 2.0]);
    assert_eq!(c, [2.0, 1.0]);
    // Pontos repetidos não têm célula própria.
    let d = Delaunay::new(&[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 0.0], [0.4, 0.4]]).unwrap();
    assert_eq!(voronoi(&d).len(), 4);
}

// ================================================================ fecho 3-D

#[test]
fn fecho_3d_definicao_euler_e_volumes() {
    use chimalab_math::geom::{convex_hull_3d, hull_volume, orient3d};
    let mut r = Rng::new(Algorithm::Philox, 3);
    // Pontos aleatórios numa bola: toda face deixa todos os pontos do lado de dentro.
    let pts: Vec<[f64; 3]> = (0..2000)
        .map(|_| {
            [
                r.uniform() * 2.0 - 1.0,
                r.uniform() * 2.0 - 1.0,
                r.uniform() * 2.0 - 1.0,
            ]
        })
        .filter(|p| p[0] * p[0] + p[1] * p[1] + p[2] * p[2] <= 1.0)
        .collect();
    let f = convex_hull_3d(&pts).unwrap();
    for t in &f {
        for q in &pts {
            assert!(orient3d(pts[t[0]], pts[t[1]], pts[t[2]], *q) >= 0.0);
        }
    }
    // Euler para poliedro triangulado: V − A + F = 2 com A = 3F/2.
    let mut v: Vec<usize> = f.iter().flatten().copied().collect();
    v.sort_unstable();
    v.dedup();
    assert_eq!(v.len() as i64 - (3 * f.len() / 2) as i64 + f.len() as i64, 2);
    // Volume abaixo do da bola e perto dele.
    let vol = hull_volume(&pts, &f);
    assert!(vol < 4.0 / 3.0 * std::f64::consts::PI && vol > 3.5, "{vol}");
    // Cubo com pontos internos, nas faces e repetidos: volume exato 8.
    let mut cubo: Vec<[f64; 3]> = (0..8)
        .map(|k| {
            [
                if k & 1 == 1 { 1.0 } else { -1.0 },
                if k & 2 == 2 { 1.0 } else { -1.0 },
                if k & 4 == 4 { 1.0 } else { -1.0 },
            ]
        })
        .collect();
    cubo.extend([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.5, 1.0], [1.0, 1.0, 1.0]]);
    let fc = convex_hull_3d(&cubo).unwrap();
    // Volume em ponto flutuante: soma de 12 tetraedros, erro de arredondamento ~ε.
    assert!((hull_volume(&cubo, &fc) - 8.0).abs() <= 1e-14);
    // O fecho só usa os 8 vértices (internos, de face e repetidos ficam de fora).
    assert!(fc.iter().flatten().all(|&i| i < 8));
    // Tetraedro: volume 1/6.
    let tet = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let ft = convex_hull_3d(&tet).unwrap();
    assert_eq!(ft.len(), 4);
    assert!((hull_volume(&tet, &ft) - 1.0 / 6.0).abs() <= 1e-16);
    // orient3d exato: coplanares deslocados para longe dão 0.
    let o = 1e8;
    assert_eq!(
        orient3d([o, o, o], [o + 1.0, o, o], [o, o + 1.0, o], [o + 3.0, o + 7.0, o]),
        0.0
    );
    assert_eq!(
        orient3d(
            [o, o, o],
            [o + 1.0, o, o],
            [o, o + 1.0, o],
            [o + 3.0, o + 7.0, f64::from_bits(o.to_bits() + 1)]
        ),
        -1.0
    );
    // Erros: poucos pontos, coplanares, colineares, não finitos.
    assert_eq!(
        convex_hull_3d(&tet[..3]).unwrap_err().id(),
        "chimalab:geom:pontosInsuficientes"
    );
    let plano: Vec<[f64; 3]> = (0..10).map(|k| [k as f64, (k * k) as f64, 0.0]).collect();
    assert_eq!(
        convex_hull_3d(&plano).unwrap_err().id(),
        "chimalab:geom:degenerado"
    );
    let reta: Vec<[f64; 3]> = (0..10)
        .map(|k| [k as f64 * 0.1, k as f64 * 0.2, k as f64 * 0.3])
        .collect();
    assert_eq!(
        convex_hull_3d(&reta).unwrap_err().id(),
        "chimalab:geom:degenerado"
    );
    assert_eq!(
        convex_hull_3d(&[[0.0; 3]; 5]).unwrap_err().id(),
        "chimalab:geom:pontosInsuficientes"
    );
    assert_eq!(
        convex_hull_3d(&[[0.0, 0.0, f64::NAN]; 5]).unwrap_err().id(),
        "chimalab:geom:naoFinito"
    );
}

// ================================================================ regressão: cortes arredondados

/// Recorte de Sutherland–Hodgman de um polígono por um convexo anti-horário (oráculo
/// independente da sobreposição usada em `polygon_boolean`).
fn recorte_convexo(sujeito: &[[f64; 2]], recorte: &[[f64; 2]]) -> f64 {
    let mut out = sujeito.to_vec();
    let m = recorte.len();
    for k in 0..m {
        let (a, b) = (recorte[k], recorte[(k + 1) % m]);
        let dentro = |p: [f64; 2]| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]) >= 0.0;
        let corte = |p: [f64; 2], q: [f64; 2]| {
            let (dx, dy, ex, ey) = (q[0] - p[0], q[1] - p[1], b[0] - a[0], b[1] - a[1]);
            let t = (ex * (p[1] - a[1]) - ey * (p[0] - a[0])) / (ey * dx - ex * dy);
            [p[0] + t * dx, p[1] + t * dy]
        };
        let ent = std::mem::take(&mut out);
        for i in 0..ent.len() {
            let (p, q) = (ent[i], ent[(i + 1) % ent.len()]);
            match (dentro(p), dentro(q)) {
                (true, true) => out.push(q),
                (true, false) => out.push(corte(p, q)),
                (false, true) => {
                    out.push(corte(p, q));
                    out.push(q);
                }
                _ => {}
            }
        }
        if out.is_empty() {
            return 0.0;
        }
    }
    let n = out.len();
    0.5 * (0..n)
        .map(|i| out[i][0] * out[(i + 1) % n][1] - out[(i + 1) % n][0] * out[i][1])
        .sum::<f64>()
}

/// Regressão: vértice de A quase sobre uma aresta de B (células de Voronoi vizinhas). O ponto de
/// cruzamento arredondado caía além da ponta em A e dentro de B; a cadeia quebrava e a interseção
/// perdia 60% da área (0,00109 em vez de 0,00276).
#[test]
fn intersecao_com_vertice_quase_sobre_aresta() {
    let a = Polygon::new(vec![vec![
        [0.4943224456628077, 0.4425471412815096],
        [0.3783386644616425, 0.5083509792893468],
        [0.36092115986952006, 0.5107602191715642],
        [0.23703168808834563, 0.36178395369905614],
        [0.25682111368322036, 0.30938285720124126],
        [0.3547576517525804, 0.30038507349101473],
        [0.46334216682055085, 0.34180306174253267],
        [0.4989044790854512, 0.40240407269509587],
    ]])
    .unwrap();
    let b = Polygon::new(vec![vec![
        [0.44911388006325575, 0.36721602878176485],
        [0.35158460669055425, 0.32120862957363305],
        [0.36191418331567526, 0.2534193249913781],
        [0.5428568397853627, 0.0628123225989185],
        [0.5409106735979101, 0.2032589109504635],
    ]])
    .unwrap();
    let inter = polygon_boolean(&a, &b, BoolOp::Intersection);
    let sh = recorte_convexo(&a.rings()[0], &b.rings()[0]);
    assert!((inter.area() - sh).abs() <= 1e-15, "{} vs {sh}", inter.area());
    assert!((inter.area() - 0.002_762_825_780_396_86).abs() <= 1e-15);
    // O mesmo par escalado por 2^±30 (exato): a tolerância de encaixe nas pontas é relativa à
    // escala das coordenadas, e o resultado escala com s² (mutação).
    for e in [-30i32, 30] {
        let s = 2f64.powi(e);
        let esc = |p: &Polygon| {
            Polygon::new(vec![p.rings()[0].iter().map(|v| [v[0] * s, v[1] * s]).collect()]).unwrap()
        };
        let r = polygon_boolean(&esc(&a), &esc(&b), BoolOp::Intersection).area();
        assert!(
            (r - inter.area() * s * s).abs() <= 1e-15 * s * s,
            "2^{e}: {r} vs {}",
            inter.area() * s * s
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Célula de Voronoi de um ponto inserido contra as células antigas (contatos quase
    /// degenerados por construção): cada interseção coincide com Sutherland–Hodgman, e as partes
    /// somam a área da célula nova.
    #[test]
    fn intersecoes_de_celulas_de_voronoi(sem in any::<u64>(), n in 8usize..40, qx in 0.2f64..0.8, qy in 0.2f64..0.8, e in prop_oneof![Just(0i32), Just(30), Just(-30)]) {
        // Escala 2^e exata: a tolerância de encaixe nas pontas é relativa às coordenadas.
        let s = 2f64.powi(e);
        let pts: Vec<[f64; 2]> = aleatorios(n, sem).iter().map(|p| [p[0] * s, p[1] * s]).collect();
        let (qx, qy) = (qx * s, qy * s);
        let caixa = [-3.0 * s, -3.0 * s, 4.0 * s, 4.0 * s];
        let Ok(d) = Delaunay::new(&pts) else { return Ok(()) };
        let antigas = voronoi_clipped(&d, caixa);
        let mut mais = pts.clone();
        mais.push([qx, qy]);
        let novas = voronoi_clipped(&Delaunay::new(&mais).unwrap(), caixa);
        let cq = &novas.iter().find(|c| c.0 == n).unwrap().1;
        let mut soma = 0.0;
        for (_, cel) in &antigas {
            let a = polygon_boolean(cq, cel, BoolOp::Intersection).area();
            let sh = recorte_convexo(&cq.rings()[0], &cel.rings()[0]);
            prop_assert!((a - sh).abs() <= 1e-12 * cq.area().max(1e-300) + 1e-15 * s * s, "{} vs {}", a, sh);
            soma += a;
        }
        prop_assert!((soma - cq.area()).abs() <= 1e-12 * cq.area(), "{} vs {}", soma, cq.area());
    }
}

/// Booleanas com vértices de toque (regressão e mutação): quando duas regiões se tocam só num
/// vértice (dois quadrados pelo canto, ou um furo que toca a borda externa), o resultado sai em
/// anéis simples, sem vértice repetido; antes saía um anel em forma de 8. Arestas comuns sem área
/// não geram anéis.
#[test]
fn booleana_com_toque_em_vertice_e_aresta() {
    let quad = |x: f64, y: f64| {
        Polygon::new(vec![vec![[x, y], [x + 1.0, y], [x + 1.0, y + 1.0], [x, y + 1.0]]]).unwrap()
    };
    let simples = |r: &Vec<[f64; 2]>| {
        // Sem vértice repetido no anel.
        let mut v: Vec<[u64; 2]> = r.iter().map(|p| [p[0].to_bits(), p[1].to_bits()]).collect();
        v.sort_unstable();
        v.windows(2).all(|w| w[0] != w[1])
    };
    // Toque num canto, nas duas diagonais e com o segundo quadrado em cada posição.
    for (x, y) in [(1.0, 1.0), (-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0)] {
        let (a, b) = (quad(0.0, 0.0), quad(x, y));
        for op in [BoolOp::Union, BoolOp::Xor] {
            let r = polygon_boolean(&a, &b, op);
            assert_eq!(r.rings().len(), 2, "({x}, {y}) {op:?}: {:?}", r.rings());
            assert!(
                r.rings().iter().all(simples),
                "({x}, {y}) {op:?}: {:?}",
                r.rings()
            );
            assert!((r.area() - 2.0).abs() <= 1e-15, "({x}, {y}) {op:?}");
        }
        let r = polygon_boolean(&a, &b, BoolOp::Intersection);
        assert!(r.rings().is_empty(), "({x}, {y}): {:?}", r.rings());
    }
    // Aresta comum: interseção vazia (sem anel degenerado), união com área 2.
    let (a, b) = (quad(0.0, 0.0), quad(1.0, 0.0));
    assert!(polygon_boolean(&a, &b, BoolOp::Intersection).rings().is_empty());
    assert!((polygon_boolean(&a, &b, BoolOp::Union).area() - 2.0).abs() <= 1e-15);
    // Furo que toca a borda externa num ponto: quadrado [0, 2]² menos o triângulo com vértice
    // em (1, 0) sobre a borda. Área 4 − 1/2, anéis simples, e o ponto de toque fica na borda.
    let q2 = Polygon::new(vec![vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]]).unwrap();
    let tri = Polygon::new(vec![vec![[1.0, 0.0], [1.5, 1.0], [0.5, 1.0]]]).unwrap();
    let r = polygon_boolean(&q2, &tri, BoolOp::Difference);
    assert!((r.area() - 3.5).abs() <= 1e-15, "{}", r.area());
    assert!(r.rings().iter().all(simples), "{:?}", r.rings());
    assert_eq!(r.locate([1.0, 0.5]), Location::Outside);
    assert_eq!(r.locate([1.0, 0.0]), Location::Boundary);
    assert_eq!(r.locate([0.3, 0.3]), Location::Inside);
    // Três quadrados em L tocando-se em canto e aresta: XOR com o do meio.
    let l = polygon_boolean(&quad(0.0, 0.0), &quad(1.0, 1.0), BoolOp::Union);
    let r = polygon_boolean(&l, &quad(2.0, 0.0), BoolOp::Union);
    assert!((r.area() - 3.0).abs() <= 1e-15);
    assert!(r.rings().iter().all(simples), "{:?}", r.rings());
}

/// Regressão: células de Voronoi recortadas em escala pequena. Os raios das células ilimitadas
/// tinham comprimento 4·ext + 1 (parcela absoluta); em coordenadas ~1e-9 ficavam 1e8 vezes
/// maiores que a caixa, e as áreas erravam em até 1e-8 relativo. Agora o diagrama é invariante
/// por escala: com coordenadas multiplicadas por 2^±30 (exato), as áreas escalam exatamente.
#[test]
fn voronoi_recortado_invariante_por_escala() {
    let base = aleatorios(8, 1_385_169_509_088_931_791);
    let areas = |e: i32| -> Vec<f64> {
        let s = 2f64.powi(e);
        let mut pts: Vec<[f64; 2]> = base.iter().map(|p| [p[0] * s, p[1] * s]).collect();
        pts.push([0.2 * s, 0.2 * s]);
        let caixa = [-3.0 * s, -3.0 * s, 4.0 * s, 4.0 * s];
        voronoi_clipped(&Delaunay::new(&pts).unwrap(), caixa)
            .iter()
            .map(|c| c.1.area() / (s * s))
            .collect()
    };
    let a0 = areas(0);
    assert!((a0.iter().sum::<f64>() - 49.0).abs() <= 1e-12);
    // Até 2^±100: abaixo de ~1e-75 os termos do incircle (quarta potência das coordenadas)
    // sofrem underflow e os predicados deixam de ser exatos (limite das expansões em f64).
    for e in [-30, 30, -100, 100] {
        assert_eq!(areas(e), a0, "2^{e}");
    }
}

/// Centroide fora da origem, ponto no polígono com vértices na altura do ponto e cobertura da
/// caixa pelo Voronoi recortado (mutação).
#[test]
fn centroide_raio_por_vertices_e_cobertura_da_caixa() {
    // Centroide de um L deslocado: forma fechada por decomposição em dois retângulos.
    for (ox, oy) in [(0.0, 0.0), (5.0, -7.0), (1e6, 3e6)] {
        let l = [
            [0.0, 0.0],
            [3.0, 0.0],
            [3.0, 1.0],
            [1.0, 1.0],
            [1.0, 2.0],
            [0.0, 2.0],
        ];
        let p: Vec<[f64; 2]> = l.iter().map(|v| [v[0] + ox, v[1] + oy]).collect();
        // Retângulos [0,3]×[0,1] (área 3, centro (1,5; 0,5)) e [0,1]×[1,2] (área 1, centro (0,5; 1,5)).
        let c = polygon_centroid(&p).unwrap();
        let esp = [ox + (3.0 * 1.5 + 0.5) / 4.0, oy + (3.0 * 0.5 + 1.5) / 4.0];
        assert!(
            (c[0] - esp[0]).abs() <= 1e-12 * (1.0 + ox.abs())
                && (c[1] - esp[1]).abs() <= 1e-12 * (1.0 + oy.abs()),
            "{c:?} vs {esp:?}"
        );
    }
    // Losango |x| + |y| ≤ 1: pontos na altura dos vértices (0, ±1) e (±1, 0), em que o raio
    // horizontal passa exatamente por um vértice; oráculo pela desigualdade.
    let los = [[0.0, -1.0], [1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]];
    for i in -8..=8 {
        for y in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let x = i as f64 / 4.0;
            let s = x.abs() + f64::abs(y);
            let esp = if s < 1.0 {
                Location::Inside
            } else if s == 1.0 {
                Location::Boundary
            } else {
                Location::Outside
            };
            assert_eq!(point_in_polygon([x, y], &los), esp, "({x}, {y})");
        }
    }
    // Polígono com arestas horizontais na altura do ponto (degraus).
    let degraus = [
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [3.0, 2.0],
        [3.0, 1.0],
        [1.0, 1.0],
        [1.0, 2.0],
        [0.0, 2.0],
    ];
    for (q, esp) in [
        ([2.0, 1.0], Location::Boundary),
        ([2.0, 1.5], Location::Outside),
        ([0.5, 1.0], Location::Inside),
        ([3.5, 1.0], Location::Inside),
        ([-1.0, 1.0], Location::Outside),
        ([5.0, 1.0], Location::Outside),
        ([5.0, 2.0], Location::Outside),
        ([-1.0, 0.0], Location::Outside),
        ([2.0, 0.5], Location::Inside),
    ] {
        assert_eq!(point_in_polygon(q, &degraus), esp, "{q:?}");
    }
    // Voronoi recortado: as células cobrem a caixa (áreas somam a da caixa), mesmo com a caixa
    // muito maior que a nuvem, alongada, ou longe dos pontos.
    let pts = aleatorios(12, 77);
    let longe: Vec<[f64; 2]> = pts.iter().map(|p| [p[0] + 1e3, p[1] + 1e3]).collect();
    for (nuvem, caixa) in [
        (&pts, [-100.0, -100.0, 100.0, 100.0]),
        (&pts, [-1000.0, -1.0, 1000.0, 1.0]),
        (&pts, [-3.0, 0.2, 5.0, 0.3]),
        (&longe, [0.0, 0.0, 1.0, 1.0]),
        (&longe, [-50.0, -50.0, 50.0, 50.0]),
    ] {
        let d = Delaunay::new(nuvem).unwrap();
        let soma: f64 = voronoi_clipped(&d, caixa).iter().map(|c| c.1.area()).sum();
        let area = (caixa[2] - caixa[0]) * (caixa[3] - caixa[1]);
        assert!((soma - area).abs() <= 1e-9 * area, "{caixa:?}: {soma} vs {area}");
    }
}

/// Ponto no polígono com o raio passando por um vértice de pico ou de vale na altura do ponto,
/// nos dois sentidos de percurso (mutação): contar duas vezes esse vértice muda o número de voltas
/// para 0 num dos sentidos e o ponto sairia "fora".
#[test]
fn ponto_no_poligono_com_entalhes() {
    // Quadrado 4×4 com entalhe por baixo até o pico (2, 2) e outro com entalhe por cima até o
    // vale (2, 2). O ponto (1, 2) está dentro nos dois; (2, 2) é borda; (2, 1) está fora no
    // primeiro e dentro no segundo.
    let pico = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [2.0, 2.0],
        [3.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [0.0, 4.0],
    ];
    let vale = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [2.0, 2.0],
        [1.0, 4.0],
        [0.0, 4.0],
    ];
    for (poli, dentro_21) in [(pico, false), (vale, true)] {
        for sentido in [false, true] {
            let mut p = poli.clone();
            if sentido {
                p.reverse();
            }
            assert_eq!(point_in_polygon([1.0, 2.0], &p), Location::Inside, "{p:?}");
            assert_eq!(point_in_polygon([3.5, 2.0], &p), Location::Inside, "{p:?}");
            assert_eq!(point_in_polygon([2.0, 2.0], &p), Location::Boundary, "{p:?}");
            assert_eq!(point_in_polygon([-1.0, 2.0], &p), Location::Outside, "{p:?}");
            let esp = if dentro_21 {
                Location::Inside
            } else {
                Location::Outside
            };
            assert_eq!(point_in_polygon([2.0, 1.0], &p), esp, "{p:?}");
        }
    }
}

/// Raio que passa exatamente por vértices (mutação): no losango, pontos fora na altura de um
/// vértice não podem contar o cruzamento duas vezes (a aresta que sobe inclui o início e exclui
/// o fim).
#[test]
fn ponto_no_poligono_raio_por_vertice() {
    let losango = [[0.0, -1.0], [1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]];
    for q in [
        [-2.0, 0.0],
        [2.0, 0.0],
        [-3.0, 1.0],
        [3.0, -1.0],
        [0.5, 1.0],
        [-0.5, -1.0],
    ] {
        assert_eq!(point_in_polygon(q, &losango), Location::Outside, "{q:?}");
    }
    for q in [[0.0, 0.0], [0.5, 0.0], [-0.5, 0.0], [0.0, 0.5]] {
        assert_eq!(point_in_polygon(q, &losango), Location::Inside, "{q:?}");
    }
    for q in [[1.0, 0.0], [0.0, 1.0], [0.5, 0.5]] {
        assert_eq!(point_in_polygon(q, &losango), Location::Boundary, "{q:?}");
    }
    // Sentido horário dá o mesmo.
    let horario: Vec<[f64; 2]> = losango.iter().rev().copied().collect();
    assert_eq!(point_in_polygon([-2.0, 0.0], &horario), Location::Outside);
    assert_eq!(point_in_polygon([0.0, 0.0], &horario), Location::Inside);
}

/// Pontos sobre as arestas do fecho, inseridos depois das pontas (mutação): o teste de
/// colinearidade dos triângulos fantasmas é alcançável. Sem ele, 216 de 4000 conjuntos deram
/// triangulação inválida.
#[test]
fn delaunay_com_pontos_sobre_o_fecho() {
    let mut s: u64 = 12345;
    let mut rnd = |m: u64| {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s % m
    };
    for _ in 0..4000 {
        let n = 10 + rnd(10);
        let nf = n as f64;
        let mut pts = vec![[0.0, 0.0], [nf, 0.0], [0.0, nf]];
        for _ in 0..(3 + rnd(12)) {
            let k = (1 + rnd(n - 1)) as f64;
            match rnd(4) {
                0 => pts.push([k, 0.0]),
                1 => pts.push([0.0, k]),
                2 => pts.push([k, nf - k]),
                _ => pts.push([rnd(n) as f64 * 0.5, rnd(n) as f64 * 0.5]),
            }
        }
        for i in (1..pts.len()).rev() {
            let j = rnd(i as u64 + 1) as usize;
            pts.swap(i, j);
        }
        let d = Delaunay::new(&pts).unwrap();
        assert!(d.is_delaunay(), "{pts:?}");
        assert!(
            d.triangles()
                .iter()
                .all(|t| orient2d(pts[t[0]], pts[t[1]], pts[t[2]]) > 0.0),
            "{pts:?}"
        );
    }
}

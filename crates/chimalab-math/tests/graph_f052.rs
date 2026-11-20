//! F052 — grafos e redes (incremento 1): aceitação, força bruta, propriedades e erros.

// Matrizes de distância por índice ficam mais legíveis assim.
#![allow(clippy::needless_range_loop)]

use chimalab_math::graph::{Graph, GraphError, PathMethod};
use proptest::prelude::*;

/// Floyd–Warshall (força bruta O(n³)).
fn floyd(g: &Graph) -> Vec<Vec<f64>> {
    let n = g.num_nodes();
    let mut d = vec![vec![f64::INFINITY; n]; n];
    for (i, row) in d.iter_mut().enumerate() {
        row[i] = 0.0;
    }
    for e in g.edges() {
        d[e.from][e.to] = d[e.from][e.to].min(e.weight);
        if !g.is_directed() {
            d[e.to][e.from] = d[e.to][e.from].min(e.weight);
        }
    }
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                if d[i][k] + d[k][j] < d[i][j] {
                    d[i][j] = d[i][k] + d[k][j];
                }
            }
        }
    }
    d
}

#[test]
fn ca1_dijkstra_correto() {
    // Rede clássica (Dijkstra, 1959 / livros-texto): s=0 … 5.
    let g = Graph::from_edges(
        6,
        true,
        &[
            (0, 1, 7.0),
            (0, 2, 9.0),
            (0, 5, 14.0),
            (1, 2, 10.0),
            (1, 3, 15.0),
            (2, 3, 11.0),
            (2, 5, 2.0),
            (3, 4, 6.0),
            (5, 4, 9.0),
        ],
    )
    .unwrap();
    let (d, _) = g.shortest_paths(0, PathMethod::Dijkstra).unwrap();
    assert_eq!(d, vec![0.0, 7.0, 9.0, 20.0, 20.0, 11.0]);
    let (p, c) = g.shortest_path(0, 4, PathMethod::Dijkstra).unwrap().unwrap();
    assert_eq!((p, c), (vec![0, 2, 5, 4], 20.0));
    // Inalcançável.
    let g2 = Graph::from_edges(3, true, &[(0, 1, 1.0)]).unwrap();
    assert!(g2.shortest_path(0, 2, PathMethod::Auto).unwrap().is_none());
    assert_eq!(
        g2.shortest_path(1, 1, PathMethod::Auto).unwrap(),
        Some((vec![1], 0.0))
    );
}

#[test]
fn ca2_fluxo_maximo_em_rede_de_teste() {
    // Rede de Cormen, Leiserson, Rivest & Stein (Introduction to Algorithms, fig. 26.1): fluxo 23.
    let g = Graph::from_edges(
        6,
        true,
        &[
            (0, 1, 16.0),
            (0, 2, 13.0),
            (1, 3, 12.0),
            (2, 1, 4.0),
            (2, 4, 14.0),
            (3, 2, 9.0),
            (3, 5, 20.0),
            (4, 3, 7.0),
            (4, 5, 4.0),
        ],
    )
    .unwrap();
    let (f, lado) = g.max_flow(0, 5).unwrap();
    assert_eq!(f, 23.0);
    // O corte informado tem capacidade igual ao fluxo (dualidade).
    let corte: f64 = g
        .edges()
        .iter()
        .filter(|e| lado[e.from] && !lado[e.to])
        .map(|e| e.weight)
        .sum();
    assert_eq!(corte, 23.0);
    assert!(lado[0] && !lado[5]);
    // Não dirigido: cada aresta nos dois sentidos.
    let u = Graph::from_edges(
        4,
        false,
        &[(0, 1, 3.0), (1, 3, 2.0), (0, 2, 2.0), (2, 3, 3.0), (1, 2, 1.0)],
    )
    .unwrap();
    assert_eq!(u.max_flow(0, 3).unwrap().0, 5.0);
    assert_eq!(u.max_flow(2, 2).unwrap().0, f64::INFINITY);
}

#[test]
fn bellman_ford_e_ciclos_negativos() {
    let g = Graph::from_edges(
        5,
        true,
        &[
            (0, 1, 6.0),
            (0, 3, 7.0),
            (1, 2, 5.0),
            (1, 3, 8.0),
            (1, 4, -4.0),
            (2, 1, -2.0),
            (3, 2, -3.0),
            (3, 4, 9.0),
            (4, 0, 2.0),
            (4, 2, 7.0),
        ],
    )
    .unwrap();
    // Exemplo de CLRS (fig. 24.4): distâncias 0, 2, 4, 7, −2.
    let (d, _) = g.shortest_paths(0, PathMethod::Auto).unwrap();
    assert_eq!(d, vec![0.0, 2.0, 4.0, 7.0, -2.0]);
    let ciclo = Graph::from_edges(3, true, &[(0, 1, 1.0), (1, 2, -2.0), (2, 1, 1.0)]).unwrap();
    assert_eq!(
        ciclo.shortest_paths(0, PathMethod::Auto).unwrap_err(),
        GraphError::NegativeCycle
    );
    // Dijkstra recusa pesos negativos; aresta negativa não dirigida é ciclo negativo.
    assert_eq!(
        g.shortest_paths(0, PathMethod::Dijkstra).unwrap_err().id(),
        "chimalab:graph:pesoInvalido"
    );
    let und = Graph::from_edges(2, false, &[(0, 1, -1.0)]).unwrap();
    assert_eq!(
        und.shortest_paths(0, PathMethod::BellmanFord).unwrap_err().id(),
        "chimalab:graph:pesoInvalido"
    );
    // Sem pesos (BFS) ignora os valores.
    let (d, _) = g.shortest_paths(0, PathMethod::Unweighted).unwrap();
    assert_eq!(d, vec![0.0, 1.0, 2.0, 1.0, 2.0]);
}

#[test]
fn percursos_componentes_topologica() {
    let g = Graph::from_edges(6, false, &[(0, 1, 1.0), (0, 2, 1.0), (1, 3, 1.0), (4, 5, 1.0)]).unwrap();
    assert_eq!(g.bfs(0).unwrap(), vec![0, 1, 2, 3]);
    assert_eq!(g.dfs(0).unwrap(), vec![0, 1, 3, 2]);
    assert_eq!(g.components(), vec![0, 0, 0, 0, 1, 1]);
    assert_eq!(g.neighbors(0).unwrap(), vec![1, 2]);
    // Fortemente conexos: dois ciclos ligados por uma aresta.
    let d = Graph::from_edges(
        6,
        true,
        &[
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 0, 1.0),
            (2, 3, 1.0),
            (3, 4, 1.0),
            (4, 3, 1.0),
            (5, 5, 1.0),
        ],
    )
    .unwrap();
    let c = d.strong_components();
    assert!(c[0] == c[1] && c[1] == c[2] && c[3] == c[4] && c[0] != c[3] && c[5] != c[0] && c[5] != c[3]);
    // Topológica (Kahn, menor índice primeiro) e ciclo.
    let dag = Graph::from_edges(5, true, &[(3, 1, 1.0), (1, 0, 1.0), (4, 0, 1.0), (2, 4, 1.0)]).unwrap();
    assert_eq!(dag.toposort().unwrap(), vec![2, 3, 1, 4, 0]);
    assert_eq!(d.toposort().unwrap_err().id(), "chimalab:graph:naoAciclico");
    assert_eq!(g.toposort().unwrap_err().id(), "chimalab:graph:tipoDeGrafo");
    // Laço e multiaresta no grau.
    let m = Graph::from_edges(2, false, &[(0, 0, 1.0), (0, 1, 1.0), (0, 1, 2.0)]).unwrap();
    assert_eq!(m.degree_centrality(), vec![4.0, 2.0]);
    // Grafo vazio.
    let v = Graph::new(0, false);
    assert!(v.components().is_empty() && v.strong_components().is_empty());
    assert_eq!(v.min_spanning_tree().unwrap(), (vec![], 0.0));
}

#[test]
fn arvore_geradora_e_centralidades() {
    let g = Graph::from_edges(
        5,
        false,
        &[
            (0, 1, 2.0),
            (0, 3, 6.0),
            (1, 2, 3.0),
            (1, 3, 8.0),
            (1, 4, 5.0),
            (2, 4, 7.0),
            (3, 4, 9.0),
        ],
    )
    .unwrap();
    let (arestas, peso) = g.min_spanning_tree().unwrap();
    assert_eq!(peso, 16.0);
    assert_eq!(arestas.len(), 4);
    assert_eq!(
        Graph::new(2, true).min_spanning_tree().unwrap_err().id(),
        "chimalab:graph:tipoDeGrafo"
    );
    // Intermediação: caminho 0−1−2−3: 0, 2, 2, 0; estrela de 5 folhas: centro (5·4)/2 = 10.
    let p = Graph::from_edges(4, false, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0)]).unwrap();
    assert_eq!(p.betweenness().unwrap(), vec![0.0, 2.0, 2.0, 0.0]);
    let e = Graph::from_edges(6, false, &(1..6).map(|k| (0, k, 1.0)).collect::<Vec<_>>()).unwrap();
    assert_eq!(e.betweenness().unwrap()[0], 10.0);
    // Dois caminhos mínimos iguais: o nó do meio de cada um recebe ½ por par.
    let q = Graph::from_edges(4, false, &[(0, 1, 1.0), (1, 3, 1.0), (0, 2, 1.0), (2, 3, 1.0)]).unwrap();
    assert_eq!(q.betweenness().unwrap(), vec![0.5, 0.5, 0.5, 0.5]);
    // Dirigido: 0→1→2 dá 1 ao nó 1.
    let dd = Graph::from_edges(3, true, &[(0, 1, 1.0), (1, 2, 1.0)]).unwrap();
    assert_eq!(dd.betweenness().unwrap(), vec![0.0, 1.0, 0.0]);
    // Proximidade: caminho 0−1−2: nó 1 tem (2)/(1+1) = 1; pontas 2/3. Isolado: 0.
    let c = Graph::from_edges(4, false, &[(0, 1, 1.0), (1, 2, 1.0)]).unwrap();
    let cl = c.closeness().unwrap();
    // Com 4 nós e r = 2 alcançáveis: (2/3)·(2/Σd).
    assert!((cl[1] - (2.0 / 3.0) * (2.0 / 2.0)).abs() <= 1e-15);
    assert!((cl[0] - (2.0 / 3.0) * (2.0 / 3.0)).abs() <= 1e-15);
    assert_eq!(cl[3], 0.0);
    assert_eq!(
        Graph::from_edges(2, true, &[(0, 1, -1.0)])
            .unwrap()
            .betweenness()
            .unwrap_err()
            .id(),
        "chimalab:graph:pesoInvalido"
    );
}

#[test]
fn erros_com_identificador() {
    let mut g = Graph::new(2, true);
    assert_eq!(
        g.add_edge(0, 2, 1.0).unwrap_err().id(),
        "chimalab:graph:noInexistente"
    );
    assert_eq!(
        g.add_edge(0, 1, f64::NAN).unwrap_err().id(),
        "chimalab:graph:pesoInvalido"
    );
    assert_eq!(g.bfs(3).unwrap_err().id(), "chimalab:graph:noInexistente");
    assert_eq!(g.dfs(3).unwrap_err().id(), "chimalab:graph:noInexistente");
    assert_eq!(g.neighbors(3).unwrap_err().id(), "chimalab:graph:noInexistente");
    assert_eq!(
        g.shortest_path(0, 9, PathMethod::Auto).unwrap_err().id(),
        "chimalab:graph:noInexistente"
    );
    assert_eq!(g.max_flow(0, 9).unwrap_err().id(), "chimalab:graph:noInexistente");
    let neg = Graph::from_edges(2, true, &[(0, 1, -1.0)]).unwrap();
    assert_eq!(
        neg.max_flow(0, 1).unwrap_err().id(),
        "chimalab:graph:pesoInvalido"
    );
    for e in [
        GraphError::NodeNotFound(1),
        GraphError::InvalidWeight(-1.0),
        GraphError::NegativeCycle,
        GraphError::NotAcyclic,
        GraphError::WrongKind("x"),
    ] {
        assert!(!e.to_string().is_empty());
    }
}

fn grafo(n: usize, dirigido: bool, arestas: &[(usize, usize, u8)]) -> Graph {
    let e: Vec<(usize, usize, f64)> = arestas
        .iter()
        .map(|&(u, v, w)| (u % n, v % n, f64::from(w)))
        .collect();
    Graph::from_edges(n, dirigido, &e).unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// Dijkstra, Bellman–Ford e BFS contra Floyd–Warshall.
    #[test]
    fn caminhos_contra_floyd(n in 1usize..9, dir in any::<bool>(), ar in prop::collection::vec((0usize..9, 0usize..9, 0u8..20), 0..25)) {
        let g = grafo(n, dir, &ar);
        let f = floyd(&g);
        for s in 0..n {
            for m in [PathMethod::Dijkstra, PathMethod::BellmanFord, PathMethod::Auto] {
                let (d, pred) = g.shortest_paths(s, m).unwrap();
                prop_assert_eq!(&d, &f[s]);
                // O caminho reconstruído tem o comprimento informado.
                for t in 0..n {
                    if let Some((c, len)) = g.shortest_path(s, t, m).unwrap() {
                        prop_assert_eq!(c[0], s);
                        prop_assert_eq!(*c.last().unwrap(), t);
                        prop_assert_eq!(len, d[t]);
                    }
                    let _ = &pred;
                }
            }
        }
    }

    /// Fluxo máximo contra o corte mínimo por força bruta (todos os 2^(n−2) cortes s–t).
    #[test]
    fn fluxo_contra_corte_minimo(n in 2usize..8, dir in any::<bool>(), ar in prop::collection::vec((0usize..8, 0usize..8, 0u8..10), 0..20)) {
        let g = grafo(n, dir, &ar);
        let (s, t) = (0, n - 1);
        let (f, lado) = g.max_flow(s, t).unwrap();
        let cap = |lado: &dyn Fn(usize) -> bool| -> f64 {
            g.edges().iter().map(|e| {
                let mut c = 0.0;
                if lado(e.from) && !lado(e.to) && e.from != e.to { c += e.weight; }
                if !g.is_directed() && lado(e.to) && !lado(e.from) { c += e.weight; }
                c
            }).sum()
        };
        let mut melhor = f64::INFINITY;
        for mask in 0u32..(1 << n) {
            let dentro = |v: usize| mask >> v & 1 == 1;
            if dentro(s) && !dentro(t) {
                melhor = melhor.min(cap(&dentro));
            }
        }
        prop_assert_eq!(f, melhor);
        prop_assert_eq!(cap(&|v| lado[v]), f);
    }

    /// Kruskal contra o mínimo sobre todas as florestas geradoras (subconjuntos acíclicos de
    /// tamanho máximo).
    #[test]
    fn kruskal_contra_forca_bruta(n in 1usize..6, ar in prop::collection::vec((0usize..6, 0usize..6, 0u8..20), 0..9)) {
        let g = grafo(n, false, &ar);
        let (esc, peso) = g.min_spanning_tree().unwrap();
        let m = g.edges().len();
        let ncomp = { let c = g.components(); c.iter().max().map_or(0, |x| x + 1) };
        let alvo = n - ncomp;
        prop_assert_eq!(esc.len(), alvo);
        let mut melhor = f64::INFINITY;
        for mask in 0u32..(1 << m) {
            if mask.count_ones() as usize != alvo { continue; }
            let sub: Vec<(usize, usize, f64)> = (0..m).filter(|k| mask >> k & 1 == 1).map(|k| { let e = g.edges()[k]; (e.from, e.to, e.weight) }).collect();
            let h = Graph::from_edges(n, false, &sub).unwrap();
            let c = h.components();
            if c.iter().max().map_or(0, |x| x + 1) == ncomp {
                melhor = melhor.min(sub.iter().map(|e| e.2).sum());
            }
        }
        prop_assert_eq!(peso, melhor);
    }

    /// Componentes fortes contra alcançabilidade mútua.
    #[test]
    fn fortes_contra_alcancabilidade(n in 1usize..9, ar in prop::collection::vec((0usize..9, 0usize..9, 1u8..2), 0..20)) {
        let g = grafo(n, true, &ar);
        let f = floyd(&g);
        let c = g.strong_components();
        for u in 0..n {
            for v in 0..n {
                let mutuo = f[u][v].is_finite() && f[v][u].is_finite();
                prop_assert_eq!(c[u] == c[v], mutuo);
            }
        }
    }
}

#[test]
fn isomorfismo_casos_classicos() {
    // C6 contra dois triângulos: mesma sequência de graus, não isomorfos.
    let c6 = Graph::from_edges(
        6,
        false,
        &(0..6).map(|i| (i, (i + 1) % 6, 1.0)).collect::<Vec<_>>(),
    )
    .unwrap();
    let tri = Graph::from_edges(
        6,
        false,
        &[
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 0, 1.0),
            (3, 4, 1.0),
            (4, 5, 1.0),
            (5, 3, 1.0),
        ],
    )
    .unwrap();
    assert!(c6.isomorphism(&tri).is_none());
    // Petersen: dois desenhos (externo/interno contra rotulagem por pares disjuntos de {0..4}).
    let mut pe = Vec::new();
    for i in 0..5 {
        pe.push((i, (i + 1) % 5, 1.0));
        pe.push((i, i + 5, 1.0));
        pe.push((i + 5, (i + 2) % 5 + 5, 1.0));
    }
    let p1 = Graph::from_edges(10, false, &pe).unwrap();
    let pares: Vec<(usize, usize)> = (0..5).flat_map(|a| (a + 1..5).map(move |b| (a, b))).collect();
    let mut kn = Vec::new();
    for i in 0..10 {
        for j in i + 1..10 {
            let (a, b) = (pares[i], pares[j]);
            if a.0 != b.0 && a.0 != b.1 && a.1 != b.0 && a.1 != b.1 {
                kn.push((i, j, 1.0));
            }
        }
    }
    let p2 = Graph::from_edges(10, false, &kn).unwrap();
    let m = p1.isomorphism(&p2).expect("Petersen é Kneser(5, 2)");
    for e in p1.edges() {
        assert!(p2.neighbors(m[e.from]).unwrap().contains(&m[e.to]));
    }
    // Dirigido: sentido importa; tamanhos diferentes.
    let a = Graph::from_edges(3, true, &[(0, 1, 1.0), (1, 2, 1.0)]).unwrap();
    let b = Graph::from_edges(3, true, &[(1, 0, 1.0), (2, 1, 1.0)]).unwrap();
    assert!(a.isomorphism(&b).is_some());
    let c = Graph::from_edges(3, true, &[(0, 1, 1.0), (0, 2, 1.0)]).unwrap();
    assert!(a.isomorphism(&c).is_none());
    assert!(a.isomorphism(&Graph::new(4, true)).is_none());
    assert!(
        a.isomorphism(&Graph::from_edges(3, false, &[(0, 1, 1.0), (1, 2, 1.0)]).unwrap())
            .is_none()
    );
    // Laços e multiarestas contam.
    let l1 = Graph::from_edges(2, false, &[(0, 0, 1.0), (0, 1, 1.0)]).unwrap();
    let l2 = Graph::from_edges(2, false, &[(1, 1, 1.0), (0, 1, 1.0)]).unwrap();
    assert_eq!(l1.isomorphism(&l2), Some(vec![1, 0]));
    let mm = Graph::from_edges(2, false, &[(0, 1, 1.0), (0, 1, 1.0)]).unwrap();
    assert!(l1.isomorphism(&mm).is_none());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Reetiquetagem aleatória é sempre isomorfa (e o mapa achado é válido); o resultado
    /// coincide com a força bruta sobre todas as permutações.
    #[test]
    fn isomorfismo_contra_forca_bruta(n in 1usize..7, ar in prop::collection::vec((0usize..7, 0usize..7), 0..12), perm_semente in any::<u64>(), outro in prop::collection::vec((0usize..7, 0usize..7), 0..12)) {
        let g = Graph::from_edges(n, false, &ar.iter().map(|&(u, v)| (u % n, v % n, 1.0)).collect::<Vec<_>>()).unwrap();
        // Permutação pseudoaleatória.
        let mut p: Vec<usize> = (0..n).collect();
        let mut s = perm_semente;
        for i in (1..n).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            p.swap(i, (s >> 33) as usize % (i + 1));
        }
        let h = Graph::from_edges(n, false, &g.edges().iter().map(|e| (p[e.from], p[e.to], 1.0)).collect::<Vec<_>>()).unwrap();
        let m = g.isomorphism(&h);
        prop_assert!(m.is_some());
        // Contra força bruta num par qualquer.
        let k = Graph::from_edges(n, false, &outro.iter().map(|&(u, v)| (u % n, v % n, 1.0)).collect::<Vec<_>>()).unwrap();
        let conta = |gr: &Graph, q: &[usize]| {
            let mut c = std::collections::HashMap::new();
            for e in gr.edges() {
                let (a, b) = (q[e.from], q[e.to]);
                *c.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
            c
        };
        let id: Vec<usize> = (0..n).collect();
        let alvo = conta(&k, &id);
        let mut existe = false;
        let mut perm: Vec<usize> = (0..n).collect();
        // Heap's algorithm sobre todas as permutações.
        fn todas(k: usize, perm: &mut Vec<usize>, f: &mut dyn FnMut(&[usize]) -> bool) -> bool {
            if k <= 1 { return f(perm); }
            for i in 0..k {
                if todas(k - 1, perm, f) { return true; }
                if k.is_multiple_of(2) { perm.swap(i, k - 1) } else { perm.swap(0, k - 1) }
            }
            false
        }
        todas(n, &mut perm, &mut |q| { if conta(&g, q) == alvo { existe = true; } existe });
        prop_assert_eq!(g.isomorphism(&k).is_some(), existe);
    }
}

// ================================================================ mutação: lacunas fechadas

#[test]
fn mutacao_bordas_e_valores_dos_erros() {
    // Nó igual a n (o primeiro inválido).
    let g = Graph::from_edges(3, false, &[(0, 1, 1.0), (1, 2, 1.0)]).unwrap();
    assert_eq!(g.neighbors(3).unwrap_err(), GraphError::NodeNotFound(3));
    assert_eq!(g.neighbors(2).unwrap(), vec![1]);
    // O peso informado no erro é o negativo (não um zero que venha antes).
    let neg = Graph::from_edges(3, false, &[(0, 1, 0.0), (1, 2, -2.0)]).unwrap();
    assert_eq!(
        neg.shortest_paths(0, PathMethod::BellmanFord).unwrap_err(),
        GraphError::InvalidWeight(-2.0)
    );
    assert_eq!(neg.betweenness().unwrap_err(), GraphError::InvalidWeight(-2.0));
    // Pesos zero são válidos na intermediação.
    let z = Graph::from_edges(3, false, &[(0, 1, 0.0), (1, 2, 1.0)]).unwrap();
    assert_eq!(z.betweenness().unwrap(), vec![0.0, 1.0, 0.0]);
    // Proximidade com todas as distâncias nulas: 0 por convenção (não ∞).
    // Wasserman–Faust no nó 0: distâncias {0, 1}, (2/2)·(2/1) = 2.
    assert_eq!(z.closeness().unwrap()[0], 2.0);
    let zz = Graph::from_edges(2, false, &[(0, 1, 0.0)]).unwrap();
    assert_eq!(zz.closeness().unwrap(), vec![0.0, 0.0]);
    // s = t: fluxo infinito e o corte contém só s.
    let (f, corte) = g.max_flow(1, 1).unwrap();
    assert_eq!((f, corte), (f64::INFINITY, vec![false, true, false]));
    // Intermediação ponderada: caminho mais longo até um nó ainda não finalizado não conta.
    let tri = Graph::from_edges(3, false, &[(0, 1, 1.0), (0, 2, 1.0), (1, 2, 5.0)]).unwrap();
    // 1 → 2 passa por 0 (2 < 5); a partir de 0, a aresta 1–2 (6 > 1) não pode somar caminhos.
    assert_eq!(tri.betweenness().unwrap(), vec![1.0, 0.0, 0.0]);
}

/// Fluxo máximo contra o corte mínimo por força bruta (teorema fluxo máximo = corte mínimo):
/// exige o resíduo reverso correto em grafos em que um caminho guloso bloqueia o ótimo.
#[test]
fn fluxo_maximo_igual_ao_corte_minimo_forca_bruta() {
    // Exemplo clássico em que é preciso desfazer fluxo: s→a→b→t é o primeiro caminho.
    let g = Graph::from_edges(
        6,
        true,
        &[
            (0, 1, 1.0),
            (0, 2, 1.0),
            (1, 3, 1.0),
            (2, 3, 1.0),
            (3, 4, 1.0),
            (1, 4, 1.0),
            (4, 5, 2.0),
            (3, 5, 1.0),
        ],
    )
    .unwrap();
    assert_eq!(g.max_flow(0, 5).unwrap().0, 2.0);
    // Caso que exige o resíduo reverso no Dinic: o único caminho curto s→a→b→t (comprimento 3)
    // bloqueia o ótimo, e a segunda unidade só passa por s→c→z→w→b, desfazendo a→b (aresta
    // reversa b→a), e por a→x→y→t. Fluxo 2; sem o reverso, 1. Nós: s=0, a=1, b=2, t=3, x=4,
    // y=5, c=6, z=7, w=8. Regressão de mutação: os testes aleatórios não exercitavam o reverso.
    let h = Graph::from_edges(
        9,
        true,
        &[
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 3, 1.0),
            (1, 4, 1.0),
            (4, 5, 1.0),
            (5, 3, 1.0),
            (0, 6, 1.0),
            (6, 7, 1.0),
            (7, 8, 1.0),
            (8, 2, 1.0),
        ],
    )
    .unwrap();
    assert_eq!(h.max_flow(0, 3).unwrap().0, 2.0);
    let mut r = chimalab_math::random::Rng::new(chimalab_math::random::Algorithm::Philox, 52);
    for _ in 0..300 {
        let n = 2 + (r.uniform() * 6.0) as usize;
        let m = (r.uniform() * 14.0) as usize;
        let dirigido = r.uniform() < 0.7;
        let ar: Vec<(usize, usize, f64)> = (0..m)
            .map(|_| {
                (
                    (r.uniform() * n as f64) as usize,
                    (r.uniform() * n as f64) as usize,
                    (1.0 + (r.uniform() * 5.0).floor()),
                )
            })
            .filter(|e| e.0 != e.1)
            .collect();
        let g = Graph::from_edges(n, dirigido, &ar).unwrap();
        let (fluxo, lado_s) = g.max_flow(0, n - 1).unwrap();
        let cap = |lado: &dyn Fn(usize) -> bool| -> f64 {
            ar.iter()
                .map(|&(u, v, w)| {
                    let mut c = 0.0;
                    if lado(u) && !lado(v) {
                        c += w;
                    }
                    if !dirigido && lado(v) && !lado(u) {
                        c += w;
                    }
                    c
                })
                .sum()
        };
        let mut minimo = f64::INFINITY;
        for mask in 0u32..(1 << n) {
            let lado = |v: usize| mask >> v & 1 == 1;
            if lado(0) && !lado(n - 1) {
                minimo = minimo.min(cap(&lado));
            }
        }
        assert_eq!(fluxo, minimo, "{ar:?} dirigido {dirigido}");
        // O corte devolvido tem exatamente essa capacidade.
        assert_eq!(cap(&|v: usize| lado_s[v]), minimo);
    }
}

/// Mutação (isomorfismo): orientação diferente com mesmas multiplicidades. Um laço em grafo não
/// dirigido e em grafo dirigido dá a mesma matriz de multiplicidades e o mesmo número de arestas;
/// só a checagem de orientação separa os dois.
#[test]
fn isomorfismo_orientacao_diferente_com_lacos() {
    let nd = Graph::from_edges(2, false, &[(0, 0, 1.0), (1, 1, 1.0)]).unwrap();
    let d = Graph::from_edges(2, true, &[(0, 0, 1.0), (1, 1, 1.0)]).unwrap();
    assert!(nd.isomorphism(&d).is_none());
    assert!(d.isomorphism(&nd).is_none());
    assert!(d.isomorphism(&d).is_some());
}

/// Isomorfismo dirigido contra força bruta: todos os pares de dígrafos simples com 3 nós (64 × 64)
/// e uma amostra fixa de pares com 4 nós e o mesmo número de arcos. Cada mapeamento devolvido
/// precisa preservar os arcos nos dois sentidos.
#[test]
fn isomorfismo_dirigido_forca_bruta() {
    fn digrafo(n: usize, mascara: u32) -> (Graph, Vec<Vec<bool>>) {
        let pares: Vec<(usize, usize)> = (0..n)
            .flat_map(|u| (0..n).filter(move |&v| v != u).map(move |v| (u, v)))
            .collect();
        let mut adj = vec![vec![false; n]; n];
        let mut arcos = Vec::new();
        for (i, &(u, v)) in pares.iter().enumerate() {
            if mascara >> i & 1 == 1 {
                adj[u][v] = true;
                arcos.push((u, v, 1.0));
            }
        }
        (Graph::from_edges(n, true, &arcos).unwrap(), adj)
    }
    fn permutacoes(n: usize) -> Vec<Vec<usize>> {
        if n == 0 {
            return vec![vec![]];
        }
        let mut out = Vec::new();
        for p in permutacoes(n - 1) {
            for i in 0..n {
                let mut q = p.clone();
                q.insert(i, n - 1);
                out.push(q);
            }
        }
        out
    }
    let valida = |a: &[Vec<bool>], b: &[Vec<bool>], p: &[usize]| {
        (0..a.len()).all(|u| (0..a.len()).all(|v| a[u][v] == b[p[u]][p[v]]))
    };
    let confere = |n: usize, x: u32, y: u32, perms: &[Vec<usize>]| {
        let ((ga, aa), (gb, ab)) = (digrafo(n, x), digrafo(n, y));
        let existe = perms.iter().any(|p| valida(&aa, &ab, p));
        match ga.isomorphism(&gb) {
            Some(p) => assert!(
                valida(&aa, &ab, &p),
                "n={n} {x:b} {y:b}: mapeamento inválido {p:?}"
            ),
            None => assert!(!existe, "n={n} {x:b} {y:b}: isomorfos, mas devolveu None"),
        }
    };
    let p3 = permutacoes(3);
    for x in 0..64u32 {
        for y in 0..64u32 {
            confere(3, x, y, &p3);
        }
    }
    let p4 = permutacoes(4);
    let mut s = 12345u64;
    let mut pares = 0;
    while pares < 3000 {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let (x, y) = ((s >> 20) as u32 & 0xfff, (s >> 40) as u32 & 0xfff);
        if x.count_ones() == y.count_ones() {
            confere(4, x, y, &p4);
            pares += 1;
        }
    }
}

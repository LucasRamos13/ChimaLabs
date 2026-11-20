//! Grafos e redes (funcionalidade F052, RF-052; DT-33).
//!
//! [`Graph`] guarda n nós (0..n) e arestas ponderadas, dirigidas ou não, com multiarestas e laços.
//! Algoritmos: [`Graph::bfs`], [`Graph::dfs`], [`Graph::components`],
//! [`Graph::strong_components`] (Tarjan), [`Graph::toposort`] (Kahn), [`Graph::shortest_path`]
//! (Dijkstra, Bellman–Ford ou BFS), [`Graph::min_spanning_tree`] (Kruskal),
//! [`Graph::max_flow`] (Dinic, com o corte mínimo) e as centralidades
//! [`Graph::degree_centrality`], [`Graph::closeness`] e [`Graph::betweenness`] (Brandes).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::fmt;

/// Erros.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphError {
    /// Nó inexistente.
    NodeNotFound(usize),
    /// Peso inválido (NaN, ou negativo onde não é permitido).
    InvalidWeight(f64),
    /// Ciclo de peso negativo alcançável (caminhos mínimos indefinidos).
    NegativeCycle,
    /// O grafo tem ciclo (ordenação topológica impossível).
    NotAcyclic,
    /// Operação exige grafo dirigido ou não dirigido.
    WrongKind(&'static str),
}

impl GraphError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::NodeNotFound(_) => "chimalab:graph:noInexistente",
            Self::InvalidWeight(_) => "chimalab:graph:pesoInvalido",
            Self::NegativeCycle => "chimalab:graph:cicloNegativo",
            Self::NotAcyclic => "chimalab:graph:naoAciclico",
            Self::WrongKind(_) => "chimalab:graph:tipoDeGrafo",
        }
    }
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(v) => write!(f, "nó {v} inexistente"),
            Self::InvalidWeight(w) => write!(f, "peso inválido {w}"),
            Self::NegativeCycle => write!(f, "ciclo de peso negativo"),
            Self::NotAcyclic => write!(f, "o grafo tem ciclo"),
            Self::WrongKind(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for GraphError {}

/// Aresta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    /// Origem.
    pub from: usize,
    /// Destino.
    pub to: usize,
    /// Peso.
    pub weight: f64,
}

/// Grafo.
#[derive(Clone, Debug, PartialEq)]
pub struct Graph {
    n: usize,
    directed: bool,
    edges: Vec<Edge>,
    /// adj[u] = (vizinho, índice da aresta).
    adj: Vec<Vec<(usize, usize)>>,
}

/// Algoritmo de caminho mínimo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PathMethod {
    /// Escolhe: BFS sem pesos (todos 1), Dijkstra com pesos ≥ 0, Bellman–Ford com negativos.
    #[default]
    Auto,
    /// Ignora os pesos (cada aresta vale 1).
    Unweighted,
    /// Dijkstra (pesos ≥ 0).
    Dijkstra,
    /// Bellman–Ford (aceita negativos; detecta ciclo negativo).
    BellmanFord,
}

impl Graph {
    /// Grafo com n nós, sem arestas.
    pub fn new(n: usize, directed: bool) -> Self {
        Self {
            n,
            directed,
            edges: Vec::new(),
            adj: vec![Vec::new(); n],
        }
    }

    /// A partir de arestas (origem, destino, peso).
    pub fn from_edges(n: usize, directed: bool, edges: &[(usize, usize, f64)]) -> Result<Self, GraphError> {
        let mut g = Self::new(n, directed);
        for &(u, v, w) in edges {
            g.add_edge(u, v, w)?;
        }
        Ok(g)
    }

    /// Acrescenta uma aresta. Erros: `noInexistente`, `pesoInvalido` (NaN).
    pub fn add_edge(&mut self, u: usize, v: usize, w: f64) -> Result<(), GraphError> {
        for x in [u, v] {
            if x >= self.n {
                return Err(GraphError::NodeNotFound(x));
            }
        }
        if w.is_nan() {
            return Err(GraphError::InvalidWeight(w));
        }
        let k = self.edges.len();
        self.edges.push(Edge {
            from: u,
            to: v,
            weight: w,
        });
        self.adj[u].push((v, k));
        if !self.directed && u != v {
            self.adj[v].push((u, k));
        }
        Ok(())
    }

    /// Número de nós.
    pub fn num_nodes(&self) -> usize {
        self.n
    }

    /// Arestas.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Dirigido.
    pub fn is_directed(&self) -> bool {
        self.directed
    }

    /// Vizinhos de saída de u (com repetição em multigrafos).
    pub fn neighbors(&self, u: usize) -> Result<Vec<usize>, GraphError> {
        self.check(u)?;
        Ok(self.adj[u].iter().map(|&(v, _)| v).collect())
    }

    fn check(&self, u: usize) -> Result<(), GraphError> {
        if u < self.n {
            Ok(())
        } else {
            Err(GraphError::NodeNotFound(u))
        }
    }

    /// Ordem de visita em largura a partir de s.
    pub fn bfs(&self, s: usize) -> Result<Vec<usize>, GraphError> {
        self.check(s)?;
        let mut visto = vec![false; self.n];
        let mut ordem = Vec::new();
        let mut fila = VecDeque::from([s]);
        visto[s] = true;
        while let Some(u) = fila.pop_front() {
            ordem.push(u);
            for &(v, _) in &self.adj[u] {
                if !visto[v] {
                    visto[v] = true;
                    fila.push_back(v);
                }
            }
        }
        Ok(ordem)
    }

    /// Ordem de visita em profundidade (pré-ordem) a partir de s, vizinhos na ordem de inserção.
    pub fn dfs(&self, s: usize) -> Result<Vec<usize>, GraphError> {
        self.check(s)?;
        let mut visto = vec![false; self.n];
        let mut ordem = Vec::new();
        let mut pilha = vec![s];
        while let Some(u) = pilha.pop() {
            if visto[u] {
                continue;
            }
            visto[u] = true;
            ordem.push(u);
            for &(v, _) in self.adj[u].iter().rev() {
                if !visto[v] {
                    pilha.push(v);
                }
            }
        }
        Ok(ordem)
    }

    /// Componentes conexos (fracos, se dirigido): rótulo por nó, numerados em ordem do menor nó.
    pub fn components(&self) -> Vec<usize> {
        let mut pai: Vec<usize> = (0..self.n).collect();
        fn raiz(p: &mut [usize], mut x: usize) -> usize {
            while p[x] != x {
                p[x] = p[p[x]];
                x = p[x];
            }
            x
        }
        for e in &self.edges {
            let (a, b) = (raiz(&mut pai, e.from), raiz(&mut pai, e.to));
            if a != b {
                pai[a.max(b)] = a.min(b);
            }
        }
        let mut rotulo = vec![usize::MAX; self.n];
        let mut prox = 0;
        (0..self.n)
            .map(|v| {
                let r = raiz(&mut pai, v);
                if rotulo[r] == usize::MAX {
                    rotulo[r] = prox;
                    prox += 1;
                }
                rotulo[r]
            })
            .collect()
    }

    /// Componentes fortemente conexos (Tarjan, iterativo): rótulo por nó, em ordem topológica
    /// reversa da condensação (Tarjan numera os sumidouros primeiro).
    pub fn strong_components(&self) -> Vec<usize> {
        let n = self.n;
        let (mut index, mut low) = (vec![usize::MAX; n], vec![0usize; n]);
        let mut na_pilha = vec![false; n];
        let mut pilha = Vec::new();
        let mut comp = vec![usize::MAX; n];
        let (mut contador, mut ncomp) = (0usize, 0usize);
        for s in 0..n {
            if index[s] != usize::MAX {
                continue;
            }
            // Pilha de chamadas: (nó, próximo vizinho a examinar).
            let mut chamadas = vec![(s, 0usize)];
            index[s] = contador;
            low[s] = contador;
            contador += 1;
            pilha.push(s);
            na_pilha[s] = true;
            while let Some(&mut (u, ref mut i)) = chamadas.last_mut() {
                if *i < self.adj[u].len() {
                    let v = self.adj[u][*i].0;
                    *i += 1;
                    if index[v] == usize::MAX {
                        index[v] = contador;
                        low[v] = contador;
                        contador += 1;
                        pilha.push(v);
                        na_pilha[v] = true;
                        chamadas.push((v, 0));
                    } else if na_pilha[v] {
                        low[u] = low[u].min(index[v]);
                    }
                } else {
                    chamadas.pop();
                    if let Some(&(p, _)) = chamadas.last() {
                        low[p] = low[p].min(low[u]);
                    }
                    if low[u] == index[u] {
                        while let Some(w) = pilha.pop() {
                            na_pilha[w] = false;
                            comp[w] = ncomp;
                            if w == u {
                                break;
                            }
                        }
                        ncomp += 1;
                    }
                }
            }
        }
        comp
    }

    /// Ordenação topológica (Kahn; empate pelo menor índice). Erros: `tipoDeGrafo` (não dirigido),
    /// `naoAciclico`.
    pub fn toposort(&self) -> Result<Vec<usize>, GraphError> {
        if !self.directed {
            return Err(GraphError::WrongKind("ordenação topológica exige grafo dirigido"));
        }
        let mut grau = vec![0usize; self.n];
        for e in &self.edges {
            grau[e.to] += 1;
        }
        let mut prontos: BinaryHeap<Reverse<usize>> =
            (0..self.n).filter(|&v| grau[v] == 0).map(Reverse).collect();
        let mut ordem = Vec::with_capacity(self.n);
        while let Some(Reverse(u)) = prontos.pop() {
            ordem.push(u);
            for &(v, _) in &self.adj[u] {
                grau[v] -= 1;
                if grau[v] == 0 {
                    prontos.push(Reverse(v));
                }
            }
        }
        if ordem.len() == self.n {
            Ok(ordem)
        } else {
            Err(GraphError::NotAcyclic)
        }
    }

    /// Distâncias mínimas de s a todos (∞ se inalcançável) e o predecessor de cada nó.
    ///
    /// Erros: `noInexistente`, `pesoInvalido` (negativo com Dijkstra; negativo em grafo não
    /// dirigido, onde a aresta é um ciclo negativo), `cicloNegativo`.
    pub fn shortest_paths(
        &self,
        s: usize,
        method: PathMethod,
    ) -> Result<(Vec<f64>, Vec<Option<usize>>), GraphError> {
        self.check(s)?;
        let tem_negativo = self.edges.iter().any(|e| e.weight < 0.0);
        let m = match method {
            PathMethod::Auto if self.edges.iter().all(|e| e.weight == 1.0) => PathMethod::Unweighted,
            PathMethod::Auto if tem_negativo => PathMethod::BellmanFord,
            PathMethod::Auto => PathMethod::Dijkstra,
            outro => outro,
        };
        let n = self.n;
        let mut dist = vec![f64::INFINITY; n];
        let mut pred = vec![None; n];
        dist[s] = 0.0;
        match m {
            PathMethod::Unweighted => {
                let mut fila = VecDeque::from([s]);
                while let Some(u) = fila.pop_front() {
                    for &(v, _) in &self.adj[u] {
                        if dist[v].is_infinite() {
                            dist[v] = dist[u] + 1.0;
                            pred[v] = Some(u);
                            fila.push_back(v);
                        }
                    }
                }
            }
            PathMethod::Dijkstra => {
                if let Some(e) = self.edges.iter().find(|e| e.weight < 0.0) {
                    return Err(GraphError::InvalidWeight(e.weight));
                }
                // Fila de prioridade com chaves inteiras ordenáveis (bits de f64 ≥ 0 preservam a ordem).
                let mut fila: BinaryHeap<Reverse<(u64, usize)>> =
                    BinaryHeap::from([Reverse((0f64.to_bits(), s))]);
                let mut feito = vec![false; n];
                while let Some(Reverse((_, u))) = fila.pop() {
                    if feito[u] {
                        continue;
                    }
                    feito[u] = true;
                    for &(v, k) in &self.adj[u] {
                        let nd = dist[u] + self.edges[k].weight;
                        if nd < dist[v] {
                            dist[v] = nd;
                            pred[v] = Some(u);
                            fila.push(Reverse((nd.to_bits(), v)));
                        }
                    }
                }
            }
            _ => {
                if !self.directed && tem_negativo {
                    let w = self
                        .edges
                        .iter()
                        .find(|e| e.weight < 0.0)
                        .map_or(0.0, |e| e.weight);
                    return Err(GraphError::InvalidWeight(w));
                }
                for _ in 1..n.max(1) {
                    let mut mudou = false;
                    for e in &self.edges {
                        for (a, b) in self.directions(e) {
                            if dist[a].is_finite() && dist[a] + e.weight < dist[b] {
                                dist[b] = dist[a] + e.weight;
                                pred[b] = Some(a);
                                mudou = true;
                            }
                        }
                    }
                    if !mudou {
                        break;
                    }
                }
                for e in &self.edges {
                    for (a, b) in self.directions(e) {
                        if dist[a].is_finite() && dist[a] + e.weight < dist[b] {
                            return Err(GraphError::NegativeCycle);
                        }
                    }
                }
            }
        }
        Ok((dist, pred))
    }

    fn directions(&self, e: &Edge) -> Vec<(usize, usize)> {
        if self.directed {
            vec![(e.from, e.to)]
        } else {
            vec![(e.from, e.to), (e.to, e.from)]
        }
    }

    /// Caminho mínimo de s a t: (nós do caminho, comprimento). `None` se t é inalcançável.
    pub fn shortest_path(
        &self,
        s: usize,
        t: usize,
        method: PathMethod,
    ) -> Result<Option<(Vec<usize>, f64)>, GraphError> {
        self.check(t)?;
        let (dist, pred) = self.shortest_paths(s, method)?;
        if dist[t].is_infinite() {
            return Ok(None);
        }
        let mut caminho = vec![t];
        let mut v = t;
        while let Some(p) = pred[v] {
            caminho.push(p);
            v = p;
        }
        caminho.reverse();
        Ok(Some((caminho, dist[t])))
    }

    /// Árvore (floresta) geradora mínima por Kruskal: índices das arestas e peso total.
    /// Erros: `tipoDeGrafo` (dirigido).
    pub fn min_spanning_tree(&self) -> Result<(Vec<usize>, f64), GraphError> {
        if self.directed {
            return Err(GraphError::WrongKind("árvore geradora exige grafo não dirigido"));
        }
        let mut ordem: Vec<usize> = (0..self.edges.len()).collect();
        ordem.sort_by(|&a, &b| {
            self.edges[a]
                .weight
                .total_cmp(&self.edges[b].weight)
                .then(a.cmp(&b))
        });
        let mut pai: Vec<usize> = (0..self.n).collect();
        fn raiz(p: &mut [usize], mut x: usize) -> usize {
            while p[x] != x {
                p[x] = p[p[x]];
                x = p[x];
            }
            x
        }
        let mut escolhidas = Vec::new();
        let mut total = 0.0;
        for k in ordem {
            let e = self.edges[k];
            let (a, b) = (raiz(&mut pai, e.from), raiz(&mut pai, e.to));
            if a != b {
                pai[a] = b;
                escolhidas.push(k);
                total += e.weight;
            }
        }
        Ok((escolhidas, total))
    }

    /// Fluxo máximo de s a t (Dinic), com os pesos como capacidades (≥ 0). Em grafo não dirigido,
    /// cada aresta vale nos dois sentidos. Devolve o valor e o lado de s do corte mínimo.
    ///
    /// Erros: `noInexistente`, `pesoInvalido` (capacidade negativa).
    pub fn max_flow(&self, s: usize, t: usize) -> Result<(f64, Vec<bool>), GraphError> {
        self.check(s)?;
        self.check(t)?;
        if let Some(e) = self.edges.iter().find(|e| e.weight < 0.0) {
            return Err(GraphError::InvalidWeight(e.weight));
        }
        // Rede residual: arestas (destino, capacidade, índice da reversa).
        let mut g: Vec<Vec<(usize, f64, usize)>> = vec![Vec::new(); self.n];
        let add = |g: &mut Vec<Vec<(usize, f64, usize)>>, u: usize, v: usize, c: f64, cr: f64| {
            let (iu, iv) = (g[u].len(), g[v].len());
            g[u].push((v, c, iv));
            g[v].push((u, cr, iu));
        };
        for e in &self.edges {
            if e.from != e.to {
                add(
                    &mut g,
                    e.from,
                    e.to,
                    e.weight,
                    if self.directed { 0.0 } else { e.weight },
                );
            }
        }
        let mut total = 0.0;
        if s == t {
            return Ok((f64::INFINITY, (0..self.n).map(|v| v == s).collect()));
        }
        loop {
            // BFS de níveis.
            let mut nivel = vec![usize::MAX; self.n];
            nivel[s] = 0;
            let mut fila = VecDeque::from([s]);
            while let Some(u) = fila.pop_front() {
                for &(v, c, _) in &g[u] {
                    if c > 0.0 && nivel[v] == usize::MAX {
                        nivel[v] = nivel[u] + 1;
                        fila.push_back(v);
                    }
                }
            }
            if nivel[t] == usize::MAX {
                let lado: Vec<bool> = nivel.iter().map(|&l| l != usize::MAX).collect();
                return Ok((total, lado));
            }
            // Fluxo bloqueante por DFS iterativa com ponteiros.
            let mut ptr = vec![0usize; self.n];
            loop {
                let f = dinic_dfs(&mut g, &nivel, &mut ptr, s, t);
                if f == 0.0 {
                    break;
                }
                total += f;
            }
        }
    }

    /// Grau (número de arestas incidentes; laços contam 2 em não dirigidos, como na soma dos graus).
    pub fn degree_centrality(&self) -> Vec<f64> {
        let mut d = vec![0.0; self.n];
        for e in &self.edges {
            d[e.from] += 1.0;
            d[e.to] += 1.0;
        }
        d
    }

    /// Proximidade de Wasserman–Faust: (r/(n − 1))·(r/Σd), com r os nós alcançáveis e d as
    /// distâncias (pesos como comprimentos). Em grafo conexo, é (n − 1)/Σd. Nó isolado: 0.
    pub fn closeness(&self) -> Result<Vec<f64>, GraphError> {
        (0..self.n)
            .map(|s| {
                let (d, _) = self.shortest_paths(s, PathMethod::Auto)?;
                let alc: Vec<f64> = d
                    .iter()
                    .enumerate()
                    .filter(|(v, x)| *v != s && x.is_finite())
                    .map(|(_, x)| *x)
                    .collect();
                let soma: f64 = alc.iter().sum();
                if alc.is_empty() || soma == 0.0 {
                    return Ok(0.0);
                }
                // Wasserman–Faust: (r/(n − 1))·(r/Σd); com todos alcançáveis, (n − 1)/Σd.
                let r = alc.len() as f64;
                let nn = (self.n - 1) as f64;
                Ok((r / nn) * (r / soma))
            })
            .collect()
    }

    /// Intermediação (Brandes): soma, sobre pares (s, t), da fração de caminhos mínimos que passam
    /// pelo nó. Sem pesos (BFS) se todos os pesos são 1; senão Dijkstra. Em grafos não dirigidos,
    /// cada par conta uma vez.
    pub fn betweenness(&self) -> Result<Vec<f64>, GraphError> {
        if let Some(e) = self.edges.iter().find(|e| e.weight < 0.0) {
            return Err(GraphError::InvalidWeight(e.weight));
        }
        let n = self.n;
        let mut cb = vec![0.0; n];
        for s in 0..n {
            let mut pilha = Vec::new();
            let mut preds: Vec<Vec<usize>> = vec![Vec::new(); n];
            let mut sigma = vec![0.0f64; n];
            let mut dist = vec![f64::INFINITY; n];
            sigma[s] = 1.0;
            dist[s] = 0.0;
            let mut fila: BinaryHeap<Reverse<(u64, usize)>> =
                BinaryHeap::from([Reverse((0f64.to_bits(), s))]);
            let mut feito = vec![false; n];
            while let Some(Reverse((_, u))) = fila.pop() {
                if feito[u] {
                    continue;
                }
                feito[u] = true;
                pilha.push(u);
                for &(v, k) in &self.adj[u] {
                    let nd = dist[u] + self.edges[k].weight;
                    if nd < dist[v] {
                        dist[v] = nd;
                        sigma[v] = sigma[u];
                        preds[v] = vec![u];
                        fila.push(Reverse((nd.to_bits(), v)));
                    } else if nd == dist[v] && !feito[v] {
                        sigma[v] += sigma[u];
                        preds[v].push(u);
                    }
                }
            }
            let mut delta = vec![0.0; n];
            while let Some(w) = pilha.pop() {
                for &v in &preds[w] {
                    delta[v] += sigma[v] / sigma[w] * (1.0 + delta[w]);
                }
                if w != s {
                    cb[w] += delta[w];
                }
            }
        }
        if !self.directed {
            cb.iter_mut().for_each(|x| *x /= 2.0);
        }
        Ok(cb)
    }
}

fn dinic_dfs(
    g: &mut [Vec<(usize, f64, usize)>],
    nivel: &[usize],
    ptr: &mut [usize],
    s: usize,
    t: usize,
) -> f64 {
    // DFS iterativa: caminho de arestas (nó, índice da aresta).
    let mut caminho: Vec<(usize, usize)> = Vec::new();
    let mut u = s;
    loop {
        if u == t {
            let f = caminho
                .iter()
                .map(|&(a, k)| g[a][k].1)
                .fold(f64::INFINITY, f64::min);
            for &(a, k) in &caminho {
                let (v, _, r) = g[a][k];
                g[a][k].1 -= f;
                g[v][r].1 += f;
            }
            return f;
        }
        let mut avancou = false;
        while ptr[u] < g[u].len() {
            let (v, c, _) = g[u][ptr[u]];
            if c > 0.0 && nivel[v] == nivel[u] + 1 {
                caminho.push((u, ptr[u]));
                u = v;
                avancou = true;
                break;
            }
            ptr[u] += 1;
        }
        if !avancou {
            // Beco sem saída: recua e descarta a aresta que levou até aqui.
            match caminho.pop() {
                Some((a, _)) => {
                    ptr[a] += 1;
                    u = a;
                }
                None => return 0.0,
            }
        }
    }
}

impl Graph {
    /// Contagem de arestas entre u e v (no sentido u → v se dirigido; multiarestas somam).
    fn multiplicidade(&self) -> Vec<Vec<u32>> {
        let mut m = vec![vec![0u32; self.n]; self.n];
        for e in &self.edges {
            m[e.from][e.to] += 1;
            if !self.directed && e.from != e.to {
                m[e.to][e.from] += 1;
            }
        }
        m
    }

    /// Isomorfismo (ignora pesos): `Some(p)` com p[u] = nó de `other` correspondente a u, tal que
    /// as multiplicidades de arestas coincidem para todo par; `None` se não forem isomorfos.
    /// Busca com retrocesso no estilo VF2 (Cordella et al., IEEE TPAMI 26, 2004): ordem dos nós
    /// por grau decrescente e poda por grau e pelas adjacências já mapeadas.
    pub fn isomorphism(&self, other: &Graph) -> Option<Vec<usize>> {
        let n = self.n;
        if n != other.n || self.directed != other.directed || self.edges.len() != other.edges.len() {
            return None;
        }
        let (ma, mb) = (self.multiplicidade(), other.multiplicidade());
        let assinatura = |m: &[Vec<u32>], v: usize| -> (u32, u32, u32) {
            let saida: u32 = m[v].iter().sum();
            let entrada: u32 = (0..n).map(|u| m[u][v]).sum();
            (saida, entrada, m[v][v])
        };
        let sa: Vec<_> = (0..n).map(|v| assinatura(&ma, v)).collect();
        let sb: Vec<_> = (0..n).map(|v| assinatura(&mb, v)).collect();
        let (mut ca, mut cb) = (sa.clone(), sb.clone());
        ca.sort_unstable();
        cb.sort_unstable();
        if ca != cb {
            return None;
        }
        let mut ordem: Vec<usize> = (0..n).collect();
        ordem.sort_by_key(|&v| std::cmp::Reverse(sa[v].0 + sa[v].1));
        let mut mapa = vec![usize::MAX; n];
        let mut usado = vec![false; n];
        #[allow(clippy::too_many_arguments)]
        fn busca(
            k: usize,
            ordem: &[usize],
            ma: &[Vec<u32>],
            mb: &[Vec<u32>],
            sa: &[(u32, u32, u32)],
            sb: &[(u32, u32, u32)],
            mapa: &mut [usize],
            usado: &mut [bool],
        ) -> bool {
            if k == ordem.len() {
                return true;
            }
            let u = ordem[k];
            for w in 0..mapa.len() {
                if usado[w] || sa[u] != sb[w] {
                    continue;
                }
                // Consistência com os já mapeados (nos dois sentidos).
                let ok = ordem[..k]
                    .iter()
                    .all(|&x| ma[u][x] == mb[w][mapa[x]] && ma[x][u] == mb[mapa[x]][w]);
                if !ok {
                    continue;
                }
                mapa[u] = w;
                usado[w] = true;
                if busca(k + 1, ordem, ma, mb, sa, sb, mapa, usado) {
                    return true;
                }
                usado[w] = false;
                mapa[u] = usize::MAX;
            }
            false
        }
        busca(0, &ordem, &ma, &mb, &sa, &sb, &mut mapa, &mut usado).then_some(mapa)
    }
}

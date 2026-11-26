//! Avaliador que percorre a AST (DT-39, resposta 48.A; semântica no DT-40).
//!
//! - O workspace do console é um quadro de variáveis; cada chamada de função abre outro,
//!   isolado (a função só vê os parâmetros e o que cria).
//! - Funções anônimas capturam os **valores** das variáveis livres no momento da criação.
//! - As funções `def` do nível de cima de um programa valem no programa inteiro (são
//!   registradas antes da execução); as `def` aninhadas, quando a instrução executa.
//! - Condições de `if`/`while` e operandos de `and`/`or` precisam ser escalares (vazio é falso).

use std::collections::{HashMap, HashSet};
use std::fmt;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chimalab_array::{Complex64, NdArray};
use chimalab_lang::sintaxe::{
    Arg, BinOp as Op, Expr, FSeg, Node, Param, Stmt, StmtKind, SyntaxError, UnOp, parse_program,
};

use crate::exibicao;
use crate::{
    BinOp, CmpOp, Index, LogicOp, RuntimeError, Value, Warning, colon, colon_elemento, colon_n, compare,
    elementwise, end_index, horzcat, logical, mldivide, mpower, mrdivide, mtimes, neg, not, subsasgn,
    subsdelete, subsref, transpose, verifica_tamanho, vertcat,
};

/// Profundidade padrão de chamadas aninhadas (configurável com
/// [`Interpretador::define_limite_de_recursao`]).
pub const MAX_CHAMADAS: usize = 1000;

/// Pilha da thread em que cada execução roda (espaço virtual reservado; só as páginas tocadas
/// ocupam memória).
pub const PILHA_DA_THREAD: usize = 256 << 20;

/// Orçamento padrão de pilha da avaliação, dentro da thread própria (sobram 32 MiB de margem).
pub const PILHA_PADRAO: usize = PILHA_DA_THREAD - (32 << 20);

/// Orçamento quando a thread própria não pode ser criada e a avaliação roda na thread de quem
/// chama (1 MiB: cabe nas threads de 2 MiB).
pub const PILHA_SEM_THREAD: usize = 1 << 20;

/// Função definida com `def`.
#[derive(Debug, PartialEq)]
pub struct Funcao {
    /// Nome.
    pub nome: String,
    /// Parâmetros.
    pub params: Vec<Param>,
    /// Corpo.
    pub corpo: Vec<Stmt>,
}

/// Função anônima com os valores capturados.
#[derive(Debug, PartialEq)]
pub struct Anonima {
    /// Parâmetros.
    pub params: Vec<String>,
    /// Corpo.
    pub corpo: Node,
    /// Valores capturados na criação.
    pub capturas: HashMap<String, Obj>,
}

/// Valor do interpretador: um array ou uma função.
#[derive(Clone, Debug, PartialEq)]
pub enum Obj {
    /// Array (todas as classes do runtime).
    Array(Value),
    /// Função anônima.
    Anonima(Arc<Anonima>),
    /// Função `def`.
    Funcao(Arc<Funcao>),
    /// Função embutida, pelo nome.
    Embutida(&'static str),
}

/// O que a execução produz para o console.
#[derive(Clone, Debug, PartialEq)]
pub enum Saida {
    /// Valor a exibir com o nome (`x = …`).
    Valor {
        /// Nome (variável ou `ans`).
        nome: String,
        /// Valor.
        valor: Obj,
    },
    /// Texto (de `disp`).
    Texto(String),
    /// Aviso.
    Aviso {
        /// Identificador.
        id: &'static str,
        /// Mensagem.
        mensagem: String,
    },
}

impl fmt::Display for Saida {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Saida::Valor { nome, valor } => write!(f, "{}", exibicao::com_nome(nome, valor)),
            Saida::Texto(t) => write!(f, "{t}"),
            Saida::Aviso { id, mensagem } => write!(f, "aviso ({id}): {mensagem}"),
        }
    }
}

/// Erro de execução (ou de sintaxe) com identificador estável e posição.
#[derive(Clone, Debug, PartialEq)]
pub struct ErroExec {
    /// Identificador (`chimalab:…`).
    pub id: String,
    /// Mensagem.
    pub mensagem: String,
    /// Linha (1-based).
    pub linha: usize,
    /// Coluna (1-based).
    pub coluna: usize,
}

impl fmt::Display for ErroExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "linha {}, coluna {}: {}",
            self.linha, self.coluna, self.mensagem
        )
    }
}

impl std::error::Error for ErroExec {}

impl From<SyntaxError> for ErroExec {
    fn from(e: SyntaxError) -> Self {
        let mensagem = e.to_string();
        let mensagem = mensagem
            .split_once(": ")
            .map_or(mensagem.clone(), |(_, m)| m.to_string());
        ErroExec {
            id: e.id().to_string(),
            mensagem,
            linha: e.line,
            coluna: e.col,
        }
    }
}

/// Resultado de uma execução: o que foi produzido até o fim ou até o erro.
#[derive(Debug, Default)]
pub struct Resultado {
    /// Saídas, na ordem.
    pub saidas: Vec<Saida>,
    /// Erro que interrompeu a execução.
    pub erro: Option<ErroExec>,
}

impl Resultado {
    /// Texto do console: as saídas, uma por linha, e o erro no fim.
    pub fn texto(&self) -> String {
        let mut t: Vec<String> = self.saidas.iter().map(|s| s.to_string()).collect();
        if let Some(e) = &self.erro {
            t.push(format!("erro: {e}"));
        }
        t.join("\n")
    }
}

pub(crate) fn erro_em(id: &str, mensagem: impl Into<String>, n: &Node) -> ErroExec {
    ErroExec {
        id: id.to_string(),
        mensagem: mensagem.into(),
        linha: n.line,
        coluna: n.col,
    }
}

pub(crate) fn de_runtime(e: RuntimeError, n: &Node) -> ErroExec {
    erro_em(e.id(), e.to_string(), n)
}

enum Fluxo {
    Normal,
    Break,
    Continue,
    Return(Vec<Obj>),
}

/// Interpretador com o workspace do console.
#[derive(Debug, Default)]
pub struct Interpretador {
    quadros: Vec<HashMap<String, Obj>>,
    funcoes: HashMap<String, Arc<Funcao>>,
    pub(crate) saidas: Vec<Saida>,
    chamadas: usize,
    /// Limite de chamadas aninhadas (0 = o padrão).
    limite_chamadas: usize,
    /// Orçamento de pilha em bytes (0 = o padrão) e o endereço de referência da execução atual.
    limite_pilha: usize,
    base_pilha: usize,
    /// Orçamento em vigor na execução atual.
    pilha_em_vigor: usize,
    /// Valores de `end` dos índices em avaliação.
    ends: Vec<usize>,
    /// Pedido de interrupção (Ctrl+C), consultado a cada volta de laço e a cada chamada.
    interrupcao: Arc<AtomicBool>,
    /// Gerador de números aleatórios (criado no primeiro uso, com semente 0).
    gerador: Option<chimalab_math::random::Rng>,
}

impl Interpretador {
    /// Interpretador com o workspace vazio.
    pub fn new() -> Self {
        Self {
            quadros: vec![HashMap::new()],
            ..Self::default()
        }
    }

    /// Variável do workspace.
    pub fn variavel(&self, nome: &str) -> Option<&Obj> {
        self.quadros[0].get(nome)
    }

    /// Define uma variável do workspace.
    pub fn define(&mut self, nome: &str, v: Value) {
        self.quadros[0].insert(nome.to_string(), Obj::Array(v));
    }

    /// Gerador de números aleatórios da sessão (Philox, semente 0, no primeiro uso).
    pub(crate) fn gerador(&mut self) -> &mut chimalab_math::random::Rng {
        self.gerador.get_or_insert_with(|| {
            chimalab_math::random::Rng::new(chimalab_math::random::Algorithm::Philox, 0)
        })
    }

    /// Variáveis do quadro atual (o workspace, ou o da função em execução), em ordem alfabética.
    pub(crate) fn variaveis_atuais(&self) -> Vec<(String, Obj)> {
        let mut v: Vec<(String, Obj)> = self
            .quadros
            .last()
            .expect("workspace")
            .iter()
            .map(|(k, o)| (k.clone(), o.clone()))
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    /// Apaga uma variável do quadro atual; devolve se existia.
    pub(crate) fn apaga(&mut self, nome: &str) -> bool {
        self.quadro().remove(nome).is_some()
    }

    /// Apaga todas as variáveis do quadro atual.
    pub(crate) fn apaga_todas(&mut self) {
        self.quadro().clear();
    }

    /// Nomes das variáveis do workspace, em ordem alfabética.
    pub fn nomes(&self) -> Vec<String> {
        let mut v: Vec<String> = self.quadros[0].keys().cloned().collect();
        v.sort();
        v
    }

    /// Limite de chamadas aninhadas (padrão [`MAX_CHAMADAS`]); passar dele é
    /// `chimalab:execucao:recursaoProfunda`.
    pub fn define_limite_de_recursao(&mut self, n: usize) {
        self.limite_chamadas = n.max(1);
    }

    /// Orçamento de pilha da avaliação, em bytes (padrão [`PILHA_PADRAO`], dentro da thread
    /// própria de [`PILHA_DA_THREAD`]); acima dele, `chimalab:execucao:recursaoProfunda` em vez de
    /// estourar a pilha. Valores acima de [`PILHA_PADRAO`] são reduzidos a ele.
    pub fn define_limite_de_pilha(&mut self, bytes: usize) {
        self.limite_pilha = bytes;
    }

    /// Endereço aproximado do topo da pilha (de uma variável local), sem `unsafe`.
    #[inline(never)]
    fn marca_de_pilha() -> usize {
        let marca = 0u8;
        std::hint::black_box(&marca) as *const u8 as usize
    }

    /// Recusa continuar se a pilha desta execução passou do orçamento (a pilha cresce para baixo
    /// nas plataformas suportadas; se crescesse para cima, a guarda só não atuaria).
    fn verifica_pilha(&self, n: &Node) -> Result<(), ErroExec> {
        if self.base_pilha.saturating_sub(Self::marca_de_pilha()) > self.pilha_em_vigor {
            return Err(erro_em(
                "chimalab:execucao:recursaoProfunda",
                "a avaliação aninhou fundo demais (recursão ou expressão profunda demais para a pilha)",
                n,
            ));
        }
        Ok(())
    }

    /// Sinal de interrupção: outra thread (o tratador de Ctrl+C) o liga, e a execução para na
    /// próxima volta de laço ou chamada com `chimalab:execucao:interrompido`, preservando o
    /// workspace.
    pub fn sinal_de_interrupcao(&self) -> Arc<AtomicBool> {
        self.interrupcao.clone()
    }

    fn verifica_interrupcao(&self, n: &Node) -> Result<(), ErroExec> {
        if self.interrupcao.swap(false, Ordering::Relaxed) {
            return Err(erro_em(
                "chimalab:execucao:interrompido",
                "execução interrompida",
                n,
            ));
        }
        Ok(())
    }

    /// Executa um programa no workspace (entrada do console ou script).
    pub fn executa(&mut self, src: &str) -> Resultado {
        self.executa_modo(src, false)
    }

    /// Executa como documento (DT-34): nada é exibido, exceto o valor da última instrução do
    /// nível de cima, se ela for uma expressão (sem `;`) ou uma atribuição (sem `;`). `disp` e
    /// avisos continuam aparecendo.
    pub fn executa_documento(&mut self, src: &str) -> Resultado {
        self.executa_modo(src, true)
    }

    /// Roda a execução numa thread com pilha própria grande; se a thread não puder ser criada,
    /// roda aqui mesmo, com o orçamento conservador [`PILHA_SEM_THREAD`].
    fn executa_modo(&mut self, src: &str, documento: bool) -> Resultado {
        let pedido = if self.limite_pilha == 0 {
            PILHA_PADRAO
        } else {
            self.limite_pilha.min(PILHA_PADRAO)
        };
        let r = std::thread::scope(|s| {
            let h = std::thread::Builder::new()
                .name("chimalab-avaliador".into())
                .stack_size(PILHA_DA_THREAD)
                .spawn_scoped(s, || {
                    self.pilha_em_vigor = pedido;
                    self.executa_aqui(src, documento)
                });
            h.ok()
                .map(|h| h.join().unwrap_or_else(|p| std::panic::resume_unwind(p)))
        });
        match r {
            Some(r) => r,
            None => {
                self.pilha_em_vigor = pedido.min(PILHA_SEM_THREAD);
                self.executa_aqui(src, documento)
            }
        }
    }

    fn executa_aqui(&mut self, src: &str, documento: bool) -> Resultado {
        self.saidas.clear();
        self.base_pilha = Self::marca_de_pilha();
        // Um pedido feito antes de a execução começar (no prompt) não vale para ela.
        self.interrupcao.store(false, Ordering::Relaxed);
        let erro = match parse_program(src) {
            Err(e) => Some(ErroExec::from(e)),
            Ok(mut prog) => {
                if documento {
                    let n = prog.len();
                    for (k, s) in prog.iter_mut().enumerate() {
                        esconde(s, k + 1 == n);
                    }
                }
                // `def` do nível de cima valem no programa inteiro.
                for s in &prog {
                    if let StmtKind::Def(nome, params, corpo) = &s.kind {
                        self.registra(nome, params, corpo);
                    }
                }
                self.chamadas = 0;
                self.ends.clear();
                self.quadros.truncate(1);
                self.bloco(&prog).err()
            }
        };
        Resultado {
            saidas: std::mem::take(&mut self.saidas),
            erro,
        }
    }

    fn registra(&mut self, nome: &str, params: &[Param], corpo: &[Stmt]) {
        let f = Funcao {
            nome: nome.to_string(),
            params: params.to_vec(),
            corpo: corpo.to_vec(),
        };
        self.funcoes.insert(nome.to_string(), Arc::new(f));
    }

    fn quadro(&mut self) -> &mut HashMap<String, Obj> {
        self.quadros.last_mut().expect("há sempre o workspace")
    }

    fn le(&self, nome: &str) -> Option<&Obj> {
        self.quadros.last().expect("workspace").get(nome)
    }

    fn mostra(&mut self, nome: &str) {
        if let Some(v) = self.le(nome).cloned() {
            self.saidas.push(Saida::Valor {
                nome: nome.to_string(),
                valor: v,
            });
        }
    }

    pub(crate) fn avisos(&mut self, w: Vec<Warning>) {
        for a in w {
            let mensagem = match &a {
                Warning::SingularMatrix { rcond } => {
                    format!(
                        "matriz singular ou mal condicionada; o resultado pode ser impreciso (rcond ≈ {rcond:.3e})"
                    )
                }
            };
            self.saidas.push(Saida::Aviso { id: a.id(), mensagem });
        }
    }

    // ------------------------------------------------------------ instruções

    fn bloco(&mut self, corpo: &[Stmt]) -> Result<Fluxo, ErroExec> {
        for s in corpo {
            match self.instrucao(s)? {
                Fluxo::Normal => {}
                f => return Ok(f),
            }
        }
        Ok(Fluxo::Normal)
    }

    fn instrucao(&mut self, s: &Stmt) -> Result<Fluxo, ErroExec> {
        match &s.kind {
            StmtKind::Expr(e) => {
                // Um nome de variável sozinho mostra a variável sem mexer em `ans`.
                if let Expr::Ident(n) = &e.expr
                    && matches!(self.le(n), Some(Obj::Array(_)))
                {
                    if s.show {
                        self.mostra(n);
                    }
                    return Ok(Fluxo::Normal);
                }
                let vals = self.saidas_de(e, 0)?;
                if let Some(v) = vals.into_iter().next() {
                    self.quadro().insert("ans".into(), v);
                    if s.show {
                        self.mostra("ans");
                    }
                }
            }
            StmtKind::Assign(alvos, valor) => {
                let vals = if alvos.len() == 1 {
                    vec![self.avalia(valor)?]
                } else {
                    if !matches!(valor.expr, Expr::Call(..)) {
                        return Err(erro_em(
                            "chimalab:execucao:variasSaidas",
                            "só uma chamada de função produz várias saídas",
                            valor,
                        ));
                    }
                    self.saidas_de(valor, alvos.len())?
                };
                for (a, v) in alvos.iter().zip(vals) {
                    self.atribui(a, v)?;
                    if s.show
                        && let Some(n) = nome_base(a).filter(|n| *n != "_")
                    {
                        self.mostra(n);
                    }
                }
            }
            StmtKind::OpAssign(alvo, op, valor) => {
                let atual = self.avalia(alvo)?;
                let r = self.avalia(valor)?;
                let v = self.binaria(*op, atual, r, alvo, valor)?;
                self.atribui(alvo, Obj::Array(v))?;
                if s.show
                    && let Some(n) = nome_base(alvo)
                {
                    self.mostra(n);
                }
            }
            StmtKind::If(ramos, senao) => {
                for (c, corpo) in ramos {
                    if self.condicao(c)? {
                        return self.bloco(corpo);
                    }
                }
                if let Some(corpo) = senao {
                    return self.bloco(corpo);
                }
            }
            StmtKind::For(var, it, corpo) => {
                // Intervalo literal: percorre sem montar o vetor (`for k in 1:1e12:` não aloca),
                // com os mesmos valores que `colon` daria.
                if let Expr::Range { start, step, stop } = &it.expr {
                    let (a, p, b) = self.intervalo(start, step.as_deref(), stop)?;
                    let n = colon_n(a, p, b);
                    for k in 0..n {
                        self.verifica_interrupcao(it)?;
                        let x = Value::scalar(colon_elemento(a, p, b, n, k));
                        self.quadro().insert(var.clone(), Obj::Array(x));
                        match self.bloco(corpo)? {
                            Fluxo::Break => break,
                            Fluxo::Return(r) => return Ok(Fluxo::Return(r)),
                            Fluxo::Normal | Fluxo::Continue => {}
                        }
                    }
                    return Ok(Fluxo::Normal);
                }
                let v = self.array(it)?;
                let linhas = v.dims()[0];
                let ncols = if v.numel() == 0 { 0 } else { v.numel() / linhas };
                for j in 0..ncols {
                    self.verifica_interrupcao(it)?;
                    let col = if linhas == 1 {
                        subsref(&v, &[Index::Scalar(j + 1)])
                    } else {
                        subsref(&v, &[Index::All, Index::Scalar(j + 1)])
                    }
                    .map_err(|e| de_runtime(e, it))?;
                    self.quadro().insert(var.clone(), Obj::Array(col));
                    match self.bloco(corpo)? {
                        Fluxo::Break => break,
                        Fluxo::Return(r) => return Ok(Fluxo::Return(r)),
                        Fluxo::Normal | Fluxo::Continue => {}
                    }
                }
            }
            StmtKind::While(c, corpo) => {
                while {
                    self.verifica_interrupcao(c)?;
                    self.condicao(c)?
                } {
                    match self.bloco(corpo)? {
                        Fluxo::Break => break,
                        Fluxo::Return(r) => return Ok(Fluxo::Return(r)),
                        Fluxo::Normal | Fluxo::Continue => {}
                    }
                }
            }
            StmtKind::Break => return Ok(Fluxo::Break),
            StmtKind::Continue => return Ok(Fluxo::Continue),
            StmtKind::Pass => {}
            StmtKind::Return(vs) => {
                let vals = vs.iter().map(|v| self.avalia(v)).collect::<Result<Vec<_>, _>>()?;
                return Ok(Fluxo::Return(vals));
            }
            StmtKind::Try(corpo, var, tratador) => {
                let (chamadas, ends) = (self.chamadas, self.ends.len());
                let quadros = self.quadros.len();
                match self.bloco(corpo) {
                    Ok(f) => return Ok(f),
                    Err(e) => {
                        // O erro pode ter saído do meio de chamadas e índices: restaura.
                        self.chamadas = chamadas;
                        self.ends.truncate(ends);
                        self.quadros.truncate(quadros);
                        if let Some(v) = var {
                            self.quadro()
                                .insert(v.clone(), Obj::Array(Value::text(&e.mensagem)));
                        }
                        return self.bloco(tratador);
                    }
                }
            }
            StmtKind::Match(sujeito, casos, padrao) => {
                let v = self.array(sujeito)?;
                for c in casos {
                    for alt in &c.values {
                        let a = self.array(alt)?;
                        if iguais(&v, &a) {
                            return self.bloco(&c.body);
                        }
                    }
                }
                if let Some(corpo) = padrao {
                    return self.bloco(corpo);
                }
            }
            StmtKind::Def(nome, params, corpo) => self.registra(nome, params, corpo),
        }
        Ok(Fluxo::Normal)
    }

    /// Condição de `if`/`while`: escalar, sem NaN.
    fn condicao(&mut self, c: &Node) -> Result<bool, ErroExec> {
        let v = self.array(c)?;
        escalar_logico(&v, c)
    }

    fn atribui(&mut self, alvo: &Node, v: Obj) -> Result<(), ErroExec> {
        match &alvo.expr {
            Expr::Ident(n) if n == "_" => Ok(()),
            Expr::Ident(n) => {
                self.quadro().insert(n.clone(), v);
                Ok(())
            }
            Expr::Index(base, args) => {
                let Expr::Ident(nome) = &base.expr else {
                    return Err(erro_em(
                        "chimalab:execucao:naoSuportado",
                        "atribuição a índice de índice ou de membro ainda não é suportada",
                        alvo,
                    ));
                };
                let Obj::Array(rhs) = v else {
                    return Err(erro_em(
                        "chimalab:execucao:funcaoComoValor",
                        "uma função não pode ser elemento de um array",
                        alvo,
                    ));
                };
                let dims = match self.le(nome) {
                    Some(Obj::Array(a)) => a.dims().to_vec(),
                    Some(_) => {
                        return Err(erro_em(
                            "chimalab:execucao:funcaoComoValor",
                            format!("`{nome}` é uma função, não um array"),
                            base,
                        ));
                    }
                    None => vec![0, 0],
                };
                let idx = self.indices(&dims, args)?;
                verifica_tamanho(&tamanho_apos(&dims, &idx)).map_err(|e| de_runtime(e, alvo))?;
                // No lugar, sem copiar o array (o runtime deixa o alvo intacto se der erro). Uma
                // variável nova só entra no workspace se a atribuição der certo.
                let apaga = rhs.dims() == [0, 0] && matches!(rhs, Value::Double(_));
                let aplica = |v: &mut Value| {
                    if apaga {
                        subsdelete(v, &idx)
                    } else {
                        subsasgn(v, &idx, &rhs)
                    }
                };
                match self.quadro().get_mut(nome) {
                    Some(Obj::Array(v)) => aplica(v).map_err(|e| de_runtime(e, alvo)),
                    _ => {
                        let mut novo = Value::doubles(&[0, 0], Vec::new());
                        aplica(&mut novo).map_err(|e| de_runtime(e, alvo))?;
                        self.quadro().insert(nome.clone(), Obj::Array(novo));
                        Ok(())
                    }
                }
            }
            _ => Err(erro_em(
                "chimalab:execucao:naoSuportado",
                "atribuição a membro ainda não é suportada (estruturas entram depois)",
                alvo,
            )),
        }
    }

    // ------------------------------------------------------------ expressões

    /// Avalia uma expressão que produz um valor.
    pub(crate) fn avalia(&mut self, n: &Node) -> Result<Obj, ErroExec> {
        self.verifica_pilha(n)?;
        match &n.expr {
            Expr::Num(v) => Ok(Obj::Array(Value::scalar(*v))),
            Expr::Imag(v) => Ok(Obj::Array(Value::Complex(NdArray::filled_nd(
                &[1, 1],
                Complex64::new(0.0, *v),
            )))),
            Expr::Str(s) => Ok(Obj::Array(Value::text(s))),
            Expr::FStr(segs) => {
                let mut out = String::new();
                for s in segs {
                    match s {
                        FSeg::Lit(t) => out.push_str(t),
                        FSeg::Expr(e, formato) => {
                            let v = self.avalia(e)?;
                            let t = exibicao::em_fstring(&v, formato.as_deref())
                                .map_err(|m| erro_em("chimalab:execucao:formatoInvalido", m, e))?;
                            out.push_str(&t);
                        }
                    }
                }
                Ok(Obj::Array(Value::text(&out)))
            }
            Expr::Ident(nome) => self.nome(nome, n),
            Expr::End => match self.ends.last() {
                Some(&e) => Ok(Obj::Array(Value::scalar(e as f64))),
                None => Err(erro_em(
                    "chimalab:sintaxe:endForaDeIndice",
                    "`end` fora de índice",
                    n,
                )),
            },
            Expr::All => Err(erro_em(
                "chimalab:execucao:naoSuportado",
                "`:` sozinho só vale em índices",
                n,
            )),
            Expr::Unary(op, a) => {
                let v = self.array(a)?;
                let r = match op {
                    UnOp::Neg => neg(&v),
                    UnOp::Plus => Ok(v),
                    UnOp::Not => not(&v),
                };
                r.map(Obj::Array).map_err(|e| de_runtime(e, n))
            }
            Expr::Binary(Op::AndAlso, a, b) | Expr::Binary(Op::OrElse, a, b) => {
                let e = matches!(n.expr, Expr::Binary(Op::AndAlso, ..));
                let va = self.array(a)?;
                let x = escalar_logico(&va, a)?;
                // Curto-circuito: `a and b` com a falso não avalia b.
                let r = if x != e {
                    x
                } else {
                    let vb = self.array(b)?;
                    escalar_logico(&vb, b)?
                };
                Ok(Obj::Array(Value::Logical(NdArray::filled_nd(&[1, 1], r))))
            }
            Expr::Binary(op, a, b) => {
                let x = self.avalia(a)?;
                let y = self.avalia(b)?;
                self.binaria(*op, x, y, a, b).map(Obj::Array)
            }
            Expr::Range { start, step, stop } => {
                let (a, p, b) = self.intervalo(start, step.as_deref(), stop)?;
                verifica_tamanho(&[1, colon_n(a, p, b)]).map_err(|e| de_runtime(e, n))?;
                Ok(Obj::Array(colon(a, p, b)))
            }
            Expr::Transpose { expr, conjugate } => {
                let v = self.array(expr)?;
                transpose(&v, *conjugate)
                    .map(Obj::Array)
                    .map_err(|e| de_runtime(e, n))
            }
            Expr::Index(base, args) => self.indexa(base, args, n),
            Expr::Call(..) => {
                let mut v = self.saidas_de(n, 1)?;
                if v.is_empty() {
                    return Err(erro_em(
                        "chimalab:execucao:semValor",
                        "a função não devolveu valor",
                        n,
                    ));
                }
                Ok(v.swap_remove(0))
            }
            Expr::Member(base, campo) => {
                let v = self.array(base)?;
                match campo.as_str() {
                    "T" => transpose(&v, false).map(Obj::Array).map_err(|e| de_runtime(e, n)),
                    "H" => transpose(&v, true).map(Obj::Array).map_err(|e| de_runtime(e, n)),
                    _ => Err(erro_em(
                        "chimalab:execucao:naoSuportado",
                        format!("membro `{campo}` desconhecido (estruturas entram depois)"),
                        n,
                    )),
                }
            }
            Expr::Matrix(linhas) => {
                let mut rows = Vec::with_capacity(linhas.len());
                for l in linhas {
                    let partes = l.iter().map(|x| self.array(x)).collect::<Result<Vec<_>, _>>()?;
                    rows.push(horzcat(&partes).map_err(|e| de_runtime(e, n))?);
                }
                if rows.is_empty() {
                    return Ok(Obj::Array(Value::doubles(&[0, 0], Vec::new())));
                }
                vertcat(&rows).map(Obj::Array).map_err(|e| de_runtime(e, n))
            }
            Expr::Lambda(params, corpo) => {
                let mut livres = HashSet::new();
                variaveis_livres(corpo, &params.iter().cloned().collect(), &mut livres);
                let quadro = self.quadros.last().expect("workspace");
                let capturas = livres
                    .into_iter()
                    .filter_map(|v| quadro.get(&v).map(|o| (v, o.clone())))
                    .collect();
                Ok(Obj::Anonima(Arc::new(Anonima {
                    params: params.clone(),
                    corpo: (**corpo).clone(),
                    capturas,
                })))
            }
        }
    }

    /// Avalia e exige um array.
    pub(crate) fn array(&mut self, n: &Node) -> Result<Value, ErroExec> {
        match self.avalia(n)? {
            Obj::Array(v) => Ok(v),
            _ => Err(erro_em(
                "chimalab:execucao:funcaoComoValor",
                "esperava um array, veio uma função",
                n,
            )),
        }
    }

    fn intervalo(
        &mut self,
        start: &Node,
        step: Option<&Node>,
        stop: &Node,
    ) -> Result<(f64, f64, f64), ErroExec> {
        let a = self.escalar_real(start, "início do intervalo")?;
        let p = match step {
            Some(s) => self.escalar_real(s, "passo do intervalo")?,
            None => 1.0,
        };
        let b = self.escalar_real(stop, "fim do intervalo")?;
        Ok((a, p, b))
    }

    fn escalar_real(&mut self, n: &Node, papel: &str) -> Result<f64, ErroExec> {
        let v = self.array(n)?;
        let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
        if x.len() != 1 {
            return Err(erro_em(
                "chimalab:execucao:intervaloNaoEscalar",
                format!(
                    "o {papel} precisa ser um escalar (veio {})",
                    exibicao::tamanho(v.dims())
                ),
                n,
            ));
        }
        Ok(x[0])
    }

    fn nome(&mut self, nome: &str, n: &Node) -> Result<Obj, ErroExec> {
        if let Some(v) = self.le(nome) {
            return Ok(v.clone());
        }
        if let Some(f) = self.funcoes.get(nome) {
            return Ok(Obj::Funcao(f.clone()));
        }
        if let Some(c) = crate::embutidas::constante(nome) {
            return Ok(Obj::Array(c));
        }
        if let Some(e) = crate::embutidas::nome(nome) {
            return Ok(Obj::Embutida(e));
        }
        let sugestao = self
            .sugestao(nome)
            .map_or(String::new(), |s| format!("; você quis dizer `{s}`?"));
        Err(erro_em(
            "chimalab:execucao:nomeIndefinido",
            format!("`{nome}` não está definido{sugestao}"),
            n,
        ))
    }

    /// Nome conhecido mais próximo (variável, `def` ou embutida), se estiver perto o bastante:
    /// distância de edição 1 para nomes de até 4 letras, 2 para os maiores. Empate: o primeiro em
    /// ordem alfabética.
    fn sugestao(&self, nome: &str) -> Option<String> {
        let limite = if nome.chars().count() <= 4 { 1 } else { 2 };
        let mut candidatos: Vec<String> = self.quadros.last().expect("workspace").keys().cloned().collect();
        candidatos.extend(self.funcoes.keys().cloned());
        candidatos.extend(crate::embutidas::todas().iter().map(|s| s.to_string()));
        candidatos.sort();
        candidatos
            .into_iter()
            .map(|c| (distancia(nome, &c), c))
            .filter(|(d, _)| *d <= limite)
            .min_by_key(|(d, _)| *d)
            .map(|(_, c)| c)
    }

    fn binaria(&mut self, op: Op, x: Obj, y: Obj, a: &Node, b: &Node) -> Result<Value, ErroExec> {
        let (Obj::Array(x), Obj::Array(y)) = (x, y) else {
            return Err(erro_em(
                "chimalab:execucao:funcaoComoValor",
                "operação aritmética com uma função (para chamá-la, use f(x))",
                a,
            ));
        };
        let r = match op {
            Op::Add => elementwise(BinOp::Add, &x, &y),
            Op::Sub => elementwise(BinOp::Sub, &x, &y),
            Op::Mul => elementwise(BinOp::Mul, &x, &y),
            Op::Div => elementwise(BinOp::Div, &x, &y),
            Op::LeftDiv => elementwise(BinOp::LeftDiv, &x, &y),
            Op::Pow => elementwise(BinOp::Pow, &x, &y),
            Op::MatMul => mtimes(&x, &y),
            Op::MatDiv | Op::MatLeftDiv | Op::MatPow => {
                let r = match op {
                    Op::MatDiv => mrdivide(&x, &y),
                    Op::MatLeftDiv => mldivide(&x, &y),
                    _ => mpower(&x, &y),
                };
                r.map(|(v, w)| {
                    self.avisos(w);
                    v
                })
            }
            Op::Eq => compare(CmpOp::Eq, &x, &y),
            Op::Ne => compare(CmpOp::Ne, &x, &y),
            Op::Lt => compare(CmpOp::Lt, &x, &y),
            Op::Le => compare(CmpOp::Le, &x, &y),
            Op::Gt => compare(CmpOp::Gt, &x, &y),
            Op::Ge => compare(CmpOp::Ge, &x, &y),
            Op::And => logical(LogicOp::And, &x, &y),
            Op::Or => logical(LogicOp::Or, &x, &y),
            Op::AndAlso | Op::OrElse => unreachable!("curto-circuito tratado antes"),
        };
        // O erro aponta o operando esquerdo (início da operação); o direito fica como contexto.
        let _ = b;
        r.map_err(|e| de_runtime(e, a))
    }

    /// Subscritos avaliados com `end` valendo para um array de forma `dims`.
    fn indices(&mut self, dims: &[usize], args: &[Node]) -> Result<Vec<Index>, ErroExec> {
        let n = args.len();
        let mut out = Vec::with_capacity(n);
        for (k, a) in args.iter().enumerate() {
            if a.expr == Expr::All {
                out.push(Index::All);
                continue;
            }
            self.ends.push(end_index(dims, k, n));
            let v = self.array(a);
            self.ends.pop();
            out.push(para_indice(&v?, a)?);
        }
        Ok(out)
    }

    fn indexa(&mut self, base: &Node, args: &[Node], n: &Node) -> Result<Obj, ErroExec> {
        // Variável: lê sem copiar o array inteiro (só a forma, para o `end`).
        if let Expr::Ident(nome) = &base.expr
            && let Some(Obj::Array(v)) = self.le(nome)
        {
            let dims = v.dims().to_vec();
            let idx = self.indices(&dims, args)?;
            let Some(Obj::Array(v)) = self.le(nome) else {
                unreachable!("os subscritos não alteram o quadro atual")
            };
            return subsref(v, &idx).map(Obj::Array).map_err(|e| de_runtime(e, n));
        }
        let v = match self.avalia(base)? {
            Obj::Array(v) => v,
            _ => {
                return Err(erro_em(
                    "chimalab:execucao:indexarFuncao",
                    "funções se chamam com parênteses: f(x)",
                    n,
                ));
            }
        };
        let idx = self.indices(v.dims(), args)?;
        subsref(&v, &idx).map(Obj::Array).map_err(|e| de_runtime(e, n))
    }

    /// Saídas de uma expressão (chamadas podem dar várias; o resto dá uma).
    fn saidas_de(&mut self, n: &Node, nargout: usize) -> Result<Vec<Obj>, ErroExec> {
        let Expr::Call(alvo, args) = &n.expr else {
            return Ok(vec![self.avalia(n)?]);
        };
        let f = match &alvo.expr {
            Expr::Ident(nome) => match self.nome(nome, alvo)? {
                Obj::Array(_) => {
                    return Err(erro_em(
                        "chimalab:execucao:chamarArray",
                        format!("`{nome}` é um array: para indexar, use {nome}[…]"),
                        n,
                    ));
                }
                f => f,
            },
            _ => match self.avalia(alvo)? {
                Obj::Array(_) => {
                    return Err(erro_em(
                        "chimalab:execucao:chamarArray",
                        "só funções se chamam com parênteses; arrays se indexam com colchetes",
                        n,
                    ));
                }
                f => f,
            },
        };
        let mut pos = Vec::new();
        let mut nomeados: Vec<(String, Obj)> = Vec::new();
        for Arg { name, value } in args {
            let v = self.avalia(value)?;
            match name {
                Some(nm) => {
                    if nomeados.iter().any(|(k, _)| k == nm) {
                        return Err(erro_em(
                            "chimalab:execucao:argumentos",
                            format!("argumento `{nm}` repetido"),
                            value,
                        ));
                    }
                    nomeados.push((nm.clone(), v));
                }
                None => pos.push(v),
            }
        }
        self.chama(&f, pos, nomeados, nargout, n)
    }

    /// Chama uma função com argumentos já avaliados.
    pub(crate) fn chama(
        &mut self,
        f: &Obj,
        pos: Vec<Obj>,
        nomeados: Vec<(String, Obj)>,
        nargout: usize,
        n: &Node,
    ) -> Result<Vec<Obj>, ErroExec> {
        if let Obj::Embutida(nome) = f {
            let vals = crate::embutidas::chama(self, nome, pos, nomeados, nargout, n)?;
            if nargout > 1 && vals.len() < nargout {
                return Err(erro_em(
                    "chimalab:execucao:saidasDemais",
                    format!(
                        "`{nome}` devolve {} valor(es), foram pedidos {nargout}",
                        vals.len()
                    ),
                    n,
                ));
            }
            return Ok(vals);
        }
        self.verifica_interrupcao(n)?;
        self.verifica_pilha(n)?;
        let limite = if self.limite_chamadas == 0 {
            MAX_CHAMADAS
        } else {
            self.limite_chamadas
        };
        if self.chamadas >= limite {
            return Err(erro_em(
                "chimalab:execucao:recursaoProfunda",
                format!("mais de {limite} chamadas aninhadas (recursão sem fim?)"),
                n,
            ));
        }
        let (quadro, executa): (HashMap<String, Obj>, Execucao) = match f {
            Obj::Anonima(a) => {
                let mut q = a.capturas.clone();
                let params: Vec<Param> = a
                    .params
                    .iter()
                    .map(|p| Param {
                        name: p.clone(),
                        default: None,
                    })
                    .collect();
                liga(&mut q, &params, pos, nomeados, n, "função anônima")?;
                (q, Execucao::Expr(a.clone()))
            }
            Obj::Funcao(fd) => {
                let mut q = HashMap::new();
                let faltam = liga(&mut q, &fd.params, pos, nomeados, n, &fd.nome)?;
                (q, Execucao::Def(fd.clone(), faltam))
            }
            Obj::Embutida(_) | Obj::Array(_) => unreachable!("tratados antes"),
        };
        self.chamadas += 1;
        self.quadros.push(quadro);
        let r = self.executa_corpo(&executa);
        self.quadros.pop();
        self.chamadas -= 1;
        let vals = r?;
        // Uma anônima sempre devolve um valor; uma `def` sem `return` devolve nenhum (e o uso
        // como valor vira `semValor` em quem pediu).
        if nargout > vals.len().max(1) {
            let nome = match &executa {
                Execucao::Def(fd, _) => fd.nome.clone(),
                Execucao::Expr(_) => "função anônima".into(),
            };
            return Err(erro_em(
                "chimalab:execucao:saidasDemais",
                format!(
                    "`{nome}` devolve {} valor(es), foram pedidos {nargout}",
                    vals.len()
                ),
                n,
            ));
        }
        Ok(vals)
    }

    fn executa_corpo(&mut self, e: &Execucao) -> Result<Vec<Obj>, ErroExec> {
        match e {
            Execucao::Expr(a) => Ok(vec![self.avalia(&a.corpo)?]),
            Execucao::Def(fd, faltam) => {
                // Valores padrão: avaliados no quadro da função, podendo usar parâmetros
                // anteriores.
                for k in faltam {
                    let p = &fd.params[*k];
                    let d = p.default.as_ref().expect("só faltam os que têm padrão");
                    let v = self.avalia(d)?;
                    self.quadro().insert(p.name.clone(), v);
                }
                match self.bloco(&fd.corpo)? {
                    Fluxo::Return(v) => Ok(v),
                    _ => Ok(Vec::new()),
                }
            }
        }
    }
}

enum Execucao {
    Expr(Arc<Anonima>),
    Def(Arc<Funcao>, Vec<usize>),
}

/// Liga os argumentos aos parâmetros; devolve os índices dos parâmetros que usam o padrão.
fn liga(
    q: &mut HashMap<String, Obj>,
    params: &[Param],
    pos: Vec<Obj>,
    nomeados: Vec<(String, Obj)>,
    n: &Node,
    nome: &str,
) -> Result<Vec<usize>, ErroExec> {
    if pos.len() > params.len() {
        return Err(erro_em(
            "chimalab:execucao:argumentos",
            format!(
                "`{nome}` aceita {} argumento(s), recebeu {}",
                params.len(),
                pos.len()
            ),
            n,
        ));
    }
    let mut ligado = vec![false; params.len()];
    for (k, v) in pos.into_iter().enumerate() {
        q.insert(params[k].name.clone(), v);
        ligado[k] = true;
    }
    for (nm, v) in nomeados {
        let Some(k) = params.iter().position(|p| p.name == nm) else {
            return Err(erro_em(
                "chimalab:execucao:argumentos",
                format!("`{nome}` não tem parâmetro `{nm}`"),
                n,
            ));
        };
        if ligado[k] {
            return Err(erro_em(
                "chimalab:execucao:argumentos",
                format!("parâmetro `{nm}` de `{nome}` recebeu dois valores"),
                n,
            ));
        }
        q.insert(nm, v);
        ligado[k] = true;
    }
    let mut faltam = Vec::new();
    for (k, p) in params.iter().enumerate() {
        if !ligado[k] {
            if p.default.is_none() {
                return Err(erro_em(
                    "chimalab:execucao:argumentoFaltando",
                    format!("falta o argumento `{}` de `{nome}`", p.name),
                    n,
                ));
            }
            faltam.push(k);
        }
    }
    Ok(faltam)
}

/// Modo documento: apaga a exibição de tudo, menos da última instrução do nível de cima.
fn esconde(s: &mut Stmt, ultima: bool) {
    if !ultima {
        s.show = false;
    }
    let blocos: Vec<&mut Vec<Stmt>> = match &mut s.kind {
        StmtKind::If(ramos, senao) => ramos.iter_mut().map(|(_, b)| b).chain(senao.iter_mut()).collect(),
        StmtKind::For(_, _, b) | StmtKind::While(_, b) => vec![b],
        StmtKind::Try(b, _, t) => vec![b, t],
        StmtKind::Match(_, casos, padrao) => casos
            .iter_mut()
            .map(|c| &mut c.body)
            .chain(padrao.iter_mut())
            .collect(),
        // O corpo das funções é avaliado à parte e segue as próprias marcas.
        _ => Vec::new(),
    };
    for b in blocos {
        for x in b.iter_mut() {
            esconde(x, false);
        }
    }
}

/// Distância de edição entre dois nomes, por caracteres, com troca de vizinhos custando 1
/// (Damerau, alinhamento ótimo de cadeias): `medai` está a 1 de `media`.
fn distancia(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, linha) in d.iter_mut().enumerate() {
        linha[0] = i;
    }
    for (j, x) in d[0].iter_mut().enumerate() {
        *x = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let custo = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + custo);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[n][m]
}

/// Limite superior da forma depois de atribuir com os subscritos `idx` (crescimento).
fn tamanho_apos(dims: &[usize], idx: &[Index]) -> Vec<usize> {
    let n = idx.len();
    idx.iter()
        .enumerate()
        .map(|(k, ix)| {
            let atual = end_index(dims, k, n);
            let maior = match ix {
                Index::Scalar(i) => *i,
                Index::List(v) => v.iter().copied().max().unwrap_or(0),
                Index::All | Index::Logical(_) => 0,
            };
            atual.max(maior)
        })
        .collect()
}

/// Nome da variável que uma atribuição altera.
fn nome_base(n: &Node) -> Option<&str> {
    match &n.expr {
        Expr::Ident(s) => Some(s),
        Expr::Index(b, _) | Expr::Member(b, _) => nome_base(b),
        _ => None,
    }
}

/// Valor lógico de uma condição (e dos operandos de `and`/`or`): vazio é falso (critério do
/// F006); escalar vale pelo valor; array com mais de um elemento é erro (DT-40, pergunta 49).
fn escalar_logico(v: &Value, n: &Node) -> Result<bool, ErroExec> {
    if v.numel() == 0 {
        return Ok(false);
    }
    if v.numel() != 1 {
        return Err(erro_em(
            "chimalab:execucao:condicaoNaoEscalar",
            format!(
                "a condição precisa ser um escalar (veio {}); use all(…) ou any(…)",
                exibicao::tamanho(v.dims())
            ),
            n,
        ));
    }
    let t = crate::ops::truth(v).map_err(|e| de_runtime(e, n))?;
    Ok(t[0])
}

/// Converte um valor em subscrito.
fn para_indice(v: &Value, n: &Node) -> Result<Index, ErroExec> {
    if let Value::Logical(m) = v {
        return Ok(Index::Logical(m.as_slice().to_vec()));
    }
    let x = v.to_f64s().map_err(|e| de_runtime(e, n))?;
    let mut out = Vec::with_capacity(x.len());
    for &e in &x {
        if !(e.is_finite() && e.fract() == 0.0 && e >= 0.0) {
            return Err(erro_em(
                "chimalab:runtime:indiceInvalido",
                format!(
                    "índice inválido: {} (índices são inteiros a partir de 1)",
                    exibicao::numero(e)
                ),
                n,
            ));
        }
        out.push(e as usize);
    }
    if v.dims() == [1, 1] {
        Ok(Index::Scalar(out[0]))
    } else {
        Ok(Index::List(out))
    }
}

/// Igualdade de valores (mesma forma e mesmos elementos; NaN ≠ NaN).
pub(crate) fn iguais(a: &Value, b: &Value) -> bool {
    a.dims() == b.dims() && a.to_complex() == b.to_complex()
}

/// Nomes livres de uma expressão (fora os ligados).
fn variaveis_livres(n: &Node, ligados: &HashSet<String>, out: &mut HashSet<String>) {
    let filhos: Vec<&Node> = match &n.expr {
        Expr::Ident(s) => {
            if !ligados.contains(s) {
                out.insert(s.clone());
            }
            Vec::new()
        }
        Expr::Unary(_, a) | Expr::Transpose { expr: a, .. } | Expr::Member(a, _) => vec![a],
        Expr::Binary(_, a, b) => vec![a, b],
        Expr::Range { start, step, stop } => {
            let mut v: Vec<&Node> = vec![start, stop];
            v.extend(step.as_deref());
            v
        }
        Expr::Index(a, args) => std::iter::once(&**a).chain(args.iter()).collect(),
        Expr::Call(a, args) => std::iter::once(&**a)
            .chain(args.iter().map(|x| &x.value))
            .collect(),
        Expr::Matrix(l) => l.iter().flatten().collect(),
        Expr::FStr(segs) => segs
            .iter()
            .filter_map(|s| match s {
                FSeg::Expr(x, _) => Some(x),
                FSeg::Lit(_) => None,
            })
            .collect(),
        Expr::Lambda(ps, corpo) => {
            let mut dentro = ligados.clone();
            dentro.extend(ps.iter().cloned());
            variaveis_livres(corpo, &dentro, out);
            Vec::new()
        }
        Expr::Num(_) | Expr::Imag(_) | Expr::Str(_) | Expr::End | Expr::All => Vec::new(),
    };
    for f in filhos {
        variaveis_livres(f, ligados, out);
    }
}

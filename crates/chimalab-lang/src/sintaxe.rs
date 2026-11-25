//! Expressões da linguagem: AST e parser (DT-37).
//!
//! Parser descendente com precedência por níveis sobre os tokens do [`crate::lexico`]. Este
//! incremento cobre só expressões; instruções e blocos (DT-39) vêm no próximo incremento.

use std::fmt;

use crate::lexico::{FPart, LexError, Op, Tok, Token, tokenize};

/// Operador unário.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    /// `-x`
    Neg,
    /// `+x`
    Plus,
}

/// Operador binário.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*` (produto matricial)
    MatMul,
    /// `/` (divisão matricial à direita)
    MatDiv,
    /// `\` (divisão matricial à esquerda)
    MatLeftDiv,
    /// `.*`
    Mul,
    /// `./`
    Div,
    /// `.\`
    LeftDiv,
    /// `^` (potência matricial)
    MatPow,
    /// `.^`
    Pow,
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
}

impl BinOp {
    fn text(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::MatMul => "*",
            BinOp::MatDiv => "/",
            BinOp::MatLeftDiv => "\\",
            BinOp::Mul => ".*",
            BinOp::Div => "./",
            BinOp::LeftDiv => ".\\",
            BinOp::MatPow => "^",
            BinOp::Pow => ".^",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
        }
    }
}

/// Argumento de chamada.
#[derive(Clone, Debug, PartialEq)]
pub struct Arg {
    /// Nome, para argumentos nomeados.
    pub name: Option<String>,
    /// Valor.
    pub value: Node,
}

/// Segmento de f-string.
#[derive(Clone, Debug, PartialEq)]
pub enum FSeg {
    /// Texto literal.
    Lit(String),
    /// Expressão, com o formato opcional.
    Expr(Node, Option<String>),
}

/// Expressão.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// Número real.
    Num(f64),
    /// Número imaginário (parte imaginária).
    Imag(f64),
    /// Texto.
    Str(String),
    /// f-string.
    FStr(Vec<FSeg>),
    /// Nome.
    Ident(String),
    /// `end` dentro de um índice.
    End,
    /// `:` sozinho num índice (todos).
    All,
    /// Operação unária.
    Unary(UnOp, Box<Node>),
    /// Operação binária.
    Binary(BinOp, Box<Node>, Box<Node>),
    /// Intervalo `início:fim` ou `início:passo:fim`.
    Range {
        /// Início.
        start: Box<Node>,
        /// Passo, se dado.
        step: Option<Box<Node>>,
        /// Fim (incluído).
        stop: Box<Node>,
    },
    /// Transposta: conjugada (`'`) ou simples (`.'`).
    Transpose {
        /// Operando.
        expr: Box<Node>,
        /// `'` (conjugada) ou `.'`.
        conjugate: bool,
    },
    /// Indexação `A[…]`.
    Index(Box<Node>, Vec<Node>),
    /// Chamada `f(…)`.
    Call(Box<Node>, Vec<Arg>),
    /// Membro `a.nome`.
    Member(Box<Node>, String),
    /// Literal de matriz: linhas de elementos (concatenados na horizontal, linhas na vertical).
    Matrix(Vec<Vec<Node>>),
}

/// Expressão com posição (linha e coluna, 1-based).
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// A expressão.
    pub expr: Expr,
    /// Linha.
    pub line: usize,
    /// Coluna.
    pub col: usize,
}

/// Tipo de erro de sintaxe.
#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxErrorKind {
    /// Erro do léxico.
    Lex(LexError),
    /// Token fora do lugar; `expected` descreve o que se esperava.
    Unexpected {
        /// O que se esperava.
        expected: &'static str,
        /// O que veio.
        found: String,
    },
    /// O texto acabou no meio da expressão.
    UnexpectedEnd,
    /// `a < b < c`.
    ChainedComparison,
    /// `end` fora de índice.
    EndOutsideIndex,
    /// Argumento posicional depois de um nomeado.
    PositionalAfterNamed,
    /// `~=` (pergunta 44.A: o diferente é `!=`).
    TildeNotEqual,
}

/// Erro de sintaxe com posição.
#[derive(Clone, Debug, PartialEq)]
pub struct SyntaxError {
    /// Tipo.
    pub kind: SyntaxErrorKind,
    /// Linha (1-based).
    pub line: usize,
    /// Coluna (1-based).
    pub col: usize,
}

impl SyntaxError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match &self.kind {
            SyntaxErrorKind::Lex(e) => e.id(),
            SyntaxErrorKind::Unexpected { .. } => "chimalab:sintaxe:tokenInesperado",
            SyntaxErrorKind::UnexpectedEnd => "chimalab:sintaxe:fimInesperado",
            SyntaxErrorKind::ChainedComparison => "chimalab:sintaxe:comparacaoEncadeada",
            SyntaxErrorKind::EndOutsideIndex => "chimalab:sintaxe:endForaDeIndice",
            SyntaxErrorKind::PositionalAfterNamed => "chimalab:sintaxe:argumentoPosicionalDepoisDeNomeado",
            SyntaxErrorKind::TildeNotEqual => "chimalab:sintaxe:diferenteComTil",
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let SyntaxErrorKind::Lex(e) = &self.kind {
            return write!(f, "{e}");
        }
        let msg = match &self.kind {
            SyntaxErrorKind::Unexpected { expected, found } => format!("esperava {expected}, veio {found}"),
            SyntaxErrorKind::UnexpectedEnd => "a expressão terminou antes do esperado".into(),
            SyntaxErrorKind::ChainedComparison => {
                "comparações não se encadeiam: escreva (a < b) e (b < c) separadamente".into()
            }
            SyntaxErrorKind::EndOutsideIndex => "`end` só vale dentro de um índice A[…]".into(),
            SyntaxErrorKind::PositionalAfterNamed => "argumento posicional depois de um nomeado".into(),
            SyntaxErrorKind::TildeNotEqual => "o diferente se escreve `!=`".into(),
            SyntaxErrorKind::Lex(_) => unreachable!(),
        };
        write!(f, "linha {}, coluna {}: {msg}", self.line, self.col)
    }
}

impl std::error::Error for SyntaxError {}

impl From<LexError> for SyntaxError {
    fn from(e: LexError) -> Self {
        let (line, col) = (e.line, e.col);
        SyntaxError {
            kind: SyntaxErrorKind::Lex(e),
            line,
            col,
        }
    }
}

/// Contexto de análise: dentro de colchetes de matriz o espaço separa elementos; dentro de
/// índices `end` e `:` sozinho valem.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Normal,
    Matrix,
    Index,
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
    ctx: Vec<Ctx>,
}

fn descreve(t: &Tok) -> String {
    match t {
        Tok::Ident(s) => format!("o nome `{s}`"),
        Tok::Number { text, .. } => format!("o número {text}"),
        Tok::Imag { text, .. } => format!("o imaginário {text}i"),
        Tok::Str(_) | Tok::FStr(_) => "um texto".into(),
        Tok::Op(o) => format!("`{}`", o.text()),
        Tok::Newline => "o fim da linha".into(),
        Tok::Indent | Tok::Dedent => "uma mudança de indentação".into(),
        Tok::Eof => "o fim do texto".into(),
    }
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.toks[self.i.min(self.toks.len() - 1)]
    }

    fn peek_at(&self, k: usize) -> &Token {
        &self.toks[(self.i + k).min(self.toks.len() - 1)]
    }

    fn next(&mut self) -> Token {
        let t = self.peek().clone();
        if self.i < self.toks.len() - 1 {
            self.i += 1;
        }
        t
    }

    fn ctx(&self) -> Ctx {
        *self.ctx.last().unwrap_or(&Ctx::Normal)
    }

    fn is_op(&self, o: Op) -> bool {
        self.peek().tok == Tok::Op(o)
    }

    /// Só restam fins de linha, fechamentos de bloco e o fim do texto.
    fn at_end(&self) -> bool {
        self.toks[self.i..]
            .iter()
            .all(|t| matches!(t.tok, Tok::Newline | Tok::Dedent | Tok::Eof))
    }

    fn unexpected(&self, expected: &'static str) -> SyntaxError {
        let t = self.peek();
        let kind = if self.at_end() {
            SyntaxErrorKind::UnexpectedEnd
        } else {
            SyntaxErrorKind::Unexpected {
                expected,
                found: descreve(&t.tok),
            }
        };
        SyntaxError {
            kind,
            line: t.line,
            col: t.col,
        }
    }

    fn expect(&mut self, o: Op, expected: &'static str) -> Result<Token, SyntaxError> {
        if self.is_op(o) {
            Ok(self.next())
        } else {
            Err(self.unexpected(expected))
        }
    }

    /// Dentro de colchetes de matriz, um token depois de espaço que começa outro elemento
    /// encerra o elemento atual.
    fn starts_element(&self) -> bool {
        if self.ctx() != Ctx::Matrix {
            return false;
        }
        let t = self.peek();
        if !t.space_before {
            return false;
        }
        match &t.tok {
            // `[1 -2]`: sinal colado ao operando depois de espaço é outro elemento.
            // `(` e `[` depois de espaço já não se colam ao elemento anterior (ver `postfix`).
            Tok::Op(Op::Plus | Op::Minus) => !self.peek_at(1).space_before,
            _ => false,
        }
    }

    fn expr(&mut self) -> Result<Node, SyntaxError> {
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Node, SyntaxError> {
        let l = self.range()?;
        if self.is_op(Op::TildeEq) {
            let t = self.peek();
            return Err(SyntaxError {
                kind: SyntaxErrorKind::TildeNotEqual,
                line: t.line,
                col: t.col,
            });
        }
        let Some(op) = self.cmp_op() else { return Ok(l) };
        self.next();
        let r = self.range()?;
        if self.cmp_op().is_some() {
            let t = self.peek();
            return Err(SyntaxError {
                kind: SyntaxErrorKind::ChainedComparison,
                line: t.line,
                col: t.col,
            });
        }
        Ok(bin(op, l, r))
    }

    fn cmp_op(&self) -> Option<BinOp> {
        if self.starts_element() {
            return None;
        }
        Some(match self.peek().tok {
            Tok::Op(Op::Eq) => BinOp::Eq,
            Tok::Op(Op::Ne) => BinOp::Ne,
            Tok::Op(Op::Lt) => BinOp::Lt,
            Tok::Op(Op::Le) => BinOp::Le,
            Tok::Op(Op::Gt) => BinOp::Gt,
            Tok::Op(Op::Ge) => BinOp::Ge,
            _ => return None,
        })
    }

    fn range(&mut self) -> Result<Node, SyntaxError> {
        // `:` sozinho num índice: todos.
        if self.ctx() == Ctx::Index && self.is_op(Op::Colon) {
            let fim = matches!(self.peek_at(1).tok, Tok::Op(Op::Comma | Op::RBracket));
            if fim {
                let t = self.next();
                return Ok(node(Expr::All, &t));
            }
        }
        let start = self.additive()?;
        if !self.is_op(Op::Colon) || self.starts_element() {
            return Ok(start);
        }
        self.next();
        let b = self.additive()?;
        let (step, stop) = if self.is_op(Op::Colon) {
            self.next();
            (Some(Box::new(b)), self.additive()?)
        } else {
            (None, b)
        };
        let (line, col) = (start.line, start.col);
        Ok(Node {
            expr: Expr::Range {
                start: Box::new(start),
                step,
                stop: Box::new(stop),
            },
            line,
            col,
        })
    }

    fn additive(&mut self) -> Result<Node, SyntaxError> {
        let mut l = self.multiplicative()?;
        loop {
            if self.starts_element() {
                return Ok(l);
            }
            let op = match self.peek().tok {
                Tok::Op(Op::Plus) => BinOp::Add,
                Tok::Op(Op::Minus) => BinOp::Sub,
                _ => return Ok(l),
            };
            self.next();
            let r = self.multiplicative()?;
            l = bin(op, l, r);
        }
    }

    fn multiplicative(&mut self) -> Result<Node, SyntaxError> {
        let mut l = self.unary()?;
        loop {
            let op = match self.peek().tok {
                Tok::Op(Op::Star) => BinOp::MatMul,
                Tok::Op(Op::Slash) => BinOp::MatDiv,
                Tok::Op(Op::Backslash) => BinOp::MatLeftDiv,
                Tok::Op(Op::DotStar) => BinOp::Mul,
                Tok::Op(Op::DotSlash) => BinOp::Div,
                Tok::Op(Op::DotBackslash) => BinOp::LeftDiv,
                _ => return Ok(l),
            };
            self.next();
            let r = self.unary()?;
            l = bin(op, l, r);
        }
    }

    fn unary(&mut self) -> Result<Node, SyntaxError> {
        let op = match self.peek().tok {
            Tok::Op(Op::Minus) => UnOp::Neg,
            Tok::Op(Op::Plus) => UnOp::Plus,
            _ => return self.power(),
        };
        let t = self.next();
        let e = self.unary()?;
        Ok(node(Expr::Unary(op, Box::new(e)), &t))
    }

    fn power(&mut self) -> Result<Node, SyntaxError> {
        let base = self.postfix()?;
        let op = match self.peek().tok {
            Tok::Op(Op::Caret) => BinOp::MatPow,
            Tok::Op(Op::DotCaret) => BinOp::Pow,
            _ => return Ok(base),
        };
        self.next();
        // À direita, e o expoente pode ter sinal: 2^-1, 2^3^2 = 2^(3^2).
        let exp = self.unary()?;
        Ok(bin(op, base, exp))
    }

    fn postfix(&mut self) -> Result<Node, SyntaxError> {
        let mut e = self.primary()?;
        loop {
            let t = self.peek().clone();
            let colado = !(self.ctx() == Ctx::Matrix && t.space_before);
            match t.tok {
                Tok::Op(Op::Quote) | Tok::Op(Op::DotQuote) => {
                    self.next();
                    e = Node {
                        expr: Expr::Transpose {
                            expr: Box::new(e.clone()),
                            conjugate: t.tok == Tok::Op(Op::Quote),
                        },
                        line: e.line,
                        col: e.col,
                    };
                }
                Tok::Op(Op::LBracket) if colado => {
                    self.next();
                    let args = self.list(Op::RBracket, Ctx::Index)?;
                    e = Node {
                        line: e.line,
                        col: e.col,
                        expr: Expr::Index(Box::new(e), args),
                    };
                }
                Tok::Op(Op::LParen) if colado => {
                    self.next();
                    let args = self.call_args()?;
                    e = Node {
                        line: e.line,
                        col: e.col,
                        expr: Expr::Call(Box::new(e), args),
                    };
                }
                Tok::Op(Op::Dot) => {
                    self.next();
                    let Tok::Ident(name) = self.peek().tok.clone() else {
                        return Err(self.unexpected("um nome depois de `.`"));
                    };
                    self.next();
                    e = Node {
                        line: e.line,
                        col: e.col,
                        expr: Expr::Member(Box::new(e), name),
                    };
                }
                _ => return Ok(e),
            }
        }
    }

    /// Lista separada por vírgulas até `fim`, no contexto dado.
    fn list(&mut self, fim: Op, ctx: Ctx) -> Result<Vec<Node>, SyntaxError> {
        self.ctx.push(ctx);
        let mut out = Vec::new();
        if !self.is_op(fim) {
            loop {
                out.push(self.expr()?);
                if self.is_op(Op::Comma) {
                    self.next();
                    continue;
                }
                break;
            }
        }
        self.ctx.pop();
        self.expect(fim, "`,` ou o fechamento")?;
        Ok(out)
    }

    fn call_args(&mut self) -> Result<Vec<Arg>, SyntaxError> {
        self.ctx.push(Ctx::Normal);
        let mut out: Vec<Arg> = Vec::new();
        let mut so_nomeados = false;
        while !self.is_op(Op::RParen) {
            if self.is_op(Op::Semicolon) {
                // `range(a, b; length=n)`: depois de `;`, só nomeados.
                self.next();
                so_nomeados = true;
                continue;
            }
            let nomeado =
                matches!(self.peek().tok, Tok::Ident(_)) && self.peek_at(1).tok == Tok::Op(Op::Assign);
            if nomeado {
                let Tok::Ident(name) = self.next().tok else {
                    unreachable!()
                };
                self.next();
                out.push(Arg {
                    name: Some(name),
                    value: self.expr()?,
                });
            } else {
                let t = self.peek().clone();
                if so_nomeados || out.iter().any(|a| a.name.is_some()) {
                    self.ctx.pop();
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::PositionalAfterNamed,
                        line: t.line,
                        col: t.col,
                    });
                }
                out.push(Arg {
                    name: None,
                    value: self.expr()?,
                });
            }
            if self.is_op(Op::Comma) {
                self.next();
            } else if !self.is_op(Op::RParen) && !self.is_op(Op::Semicolon) {
                self.ctx.pop();
                return Err(self.unexpected("`,`, `;` ou `)`"));
            }
        }
        self.ctx.pop();
        self.next();
        Ok(out)
    }

    fn matrix(&mut self, abre: &Token) -> Result<Node, SyntaxError> {
        self.ctx.push(Ctx::Matrix);
        let mut linhas: Vec<Vec<Node>> = Vec::new();
        let mut linha: Vec<Node> = Vec::new();
        let mut so_virgulas = true;
        loop {
            match self.peek().tok {
                Tok::Op(Op::RBracket) => {
                    self.next();
                    break;
                }
                Tok::Op(Op::Semicolon) | Tok::Newline => {
                    self.next();
                    if !linha.is_empty() {
                        linhas.push(std::mem::take(&mut linha));
                    }
                    continue;
                }
                Tok::Op(Op::Comma) => {
                    self.next();
                    continue;
                }
                _ => {}
            }
            if !linha.is_empty() && !matches!(self.toks[self.i - 1].tok, Tok::Op(Op::Comma)) {
                so_virgulas = false;
            }
            linha.push(self.expr()?);
        }
        if !linha.is_empty() {
            linhas.push(linha);
        }
        self.ctx.pop();
        // Forma aninhada `[[1, 2], [3, 4]]`: uma linha, separada por vírgulas, só de literais de
        // matriz: empilha (DT-34, item 28).
        if linhas.len() == 1
            && linhas[0].len() > 1
            && so_virgulas
            && linhas[0].iter().all(|n| matches!(n.expr, Expr::Matrix(_)))
        {
            let unica = linhas.pop().expect("uma linha");
            linhas = unica.into_iter().map(|n| vec![n]).collect();
        }
        Ok(node(Expr::Matrix(linhas), abre))
    }

    fn primary(&mut self) -> Result<Node, SyntaxError> {
        let t = self.peek().clone();
        let consome = matches!(
            t.tok,
            Tok::Number { .. }
                | Tok::Imag { .. }
                | Tok::Str(_)
                | Tok::FStr(_)
                | Tok::Ident(_)
                | Tok::Op(Op::LParen | Op::LBracket)
        );
        if !consome {
            return Err(self.unexpected("uma expressão"));
        }
        self.next();
        let e = match &t.tok {
            Tok::Number { value, .. } => Expr::Num(*value),
            Tok::Imag { value, .. } => Expr::Imag(*value),
            Tok::Str(s) => Expr::Str(s.clone()),
            Tok::FStr(partes) => Expr::FStr(fstring(partes)?),
            Tok::Ident(s) if s == "end" => {
                if !self.ctx.contains(&Ctx::Index) {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::EndOutsideIndex,
                        line: t.line,
                        col: t.col,
                    });
                }
                Expr::End
            }
            Tok::Ident(s) => Expr::Ident(s.clone()),
            Tok::Op(Op::LParen) => {
                self.ctx.push(Ctx::Normal);
                let mut e = self.expr()?;
                self.ctx.pop();
                self.expect(Op::RParen, "`)`")?;
                // A posição é a do parêntese.
                e.line = t.line;
                e.col = t.col;
                return Ok(e);
            }
            Tok::Op(Op::LBracket) => return self.matrix(&t),
            _ => unreachable!("filtrado acima"),
        };
        Ok(node(e, &t))
    }
}

fn node(expr: Expr, t: &Token) -> Node {
    Node {
        expr,
        line: t.line,
        col: t.col,
    }
}

fn bin(op: BinOp, l: Node, r: Node) -> Node {
    let (line, col) = (l.line, l.col);
    Node {
        expr: Expr::Binary(op, Box::new(l), Box::new(r)),
        line,
        col,
    }
}

/// Analisa as expressões de uma f-string, ajustando as posições para o texto original.
fn fstring(partes: &[FPart]) -> Result<Vec<FSeg>, SyntaxError> {
    partes
        .iter()
        .map(|p| match p {
            FPart::Lit(s) => Ok(FSeg::Lit(s.clone())),
            FPart::Expr {
                src,
                format,
                line,
                col,
            } => {
                // A expressão de uma f-string não atravessa linhas (o léxico recusa): só a coluna
                // muda.
                let desloca = |_l: usize, c: usize| (*line, col + c - 1);
                let mut n = parse_expr(src).map_err(|mut e| {
                    (e.line, e.col) = desloca(e.line, e.col);
                    e
                })?;
                reposiciona(&mut n, &desloca);
                Ok(FSeg::Expr(n, format.clone()))
            }
        })
        .collect()
}

fn reposiciona(n: &mut Node, f: &dyn Fn(usize, usize) -> (usize, usize)) {
    (n.line, n.col) = f(n.line, n.col);
    match &mut n.expr {
        Expr::Unary(_, a) | Expr::Transpose { expr: a, .. } | Expr::Member(a, _) => reposiciona(a, f),
        Expr::Binary(_, a, b) => {
            reposiciona(a, f);
            reposiciona(b, f);
        }
        Expr::Range { start, step, stop } => {
            reposiciona(start, f);
            if let Some(s) = step {
                reposiciona(s, f);
            }
            reposiciona(stop, f);
        }
        Expr::Index(a, args) => {
            reposiciona(a, f);
            args.iter_mut().for_each(|x| reposiciona(x, f));
        }
        Expr::Call(a, args) => {
            reposiciona(a, f);
            args.iter_mut().for_each(|x| reposiciona(&mut x.value, f));
        }
        Expr::Matrix(linhas) => linhas.iter_mut().flatten().for_each(|x| reposiciona(x, f)),
        Expr::FStr(segs) => segs.iter_mut().for_each(|s| {
            if let FSeg::Expr(x, _) = s {
                reposiciona(x, f);
            }
        }),
        _ => {}
    }
}

/// Analisa o texto inteiro como uma única expressão (quebras de linha no fim são aceitas).
///
/// Erros: ver [`SyntaxErrorKind`], com identificadores `chimalab:sintaxe:…` (e os
/// `chimalab:lexico:…` do léxico).
pub fn parse_expr(src: &str) -> Result<Node, SyntaxError> {
    let toks = tokenize(src)?;
    let mut p = Parser {
        toks,
        i: 0,
        ctx: Vec::new(),
    };
    let e = p.expr()?;
    while p.peek().tok == Tok::Newline {
        p.next();
    }
    if p.peek().tok != Tok::Eof {
        return Err(p.unexpected("o fim da expressão"));
    }
    Ok(e)
}

/// Forma canônica em S-expressão (para testes e depuração): `(+ 1 (* 2 3))`.
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.expr {
            Expr::Num(v) => write!(f, "{v}"),
            Expr::Imag(v) => write!(f, "{v}i"),
            Expr::Str(s) => write!(f, "{s:?}"),
            Expr::FStr(segs) => {
                write!(f, "(fstr")?;
                for s in segs {
                    match s {
                        FSeg::Lit(t) => write!(f, " {t:?}")?,
                        FSeg::Expr(n, None) => write!(f, " {{{n}}}")?,
                        FSeg::Expr(n, Some(fmt)) => write!(f, " {{{n}:{fmt}}}")?,
                    }
                }
                write!(f, ")")
            }
            Expr::Ident(s) => write!(f, "{s}"),
            Expr::End => write!(f, "end"),
            Expr::All => write!(f, ":"),
            Expr::Unary(UnOp::Neg, a) => write!(f, "(neg {a})"),
            Expr::Unary(UnOp::Plus, a) => write!(f, "(pos {a})"),
            Expr::Binary(op, a, b) => write!(f, "({} {a} {b})", op.text()),
            Expr::Range {
                start,
                step: None,
                stop,
            } => write!(f, "(: {start} {stop})"),
            Expr::Range {
                start,
                step: Some(s),
                stop,
            } => write!(f, "(: {start} {s} {stop})"),
            Expr::Transpose { expr, conjugate } => {
                write!(f, "({} {expr})", if *conjugate { "'" } else { ".'" })
            }
            Expr::Index(a, args) => {
                write!(f, "(idx {a}")?;
                for x in args {
                    write!(f, " {x}")?;
                }
                write!(f, ")")
            }
            Expr::Call(a, args) => {
                write!(f, "(call {a}")?;
                for x in args {
                    match &x.name {
                        Some(n) => write!(f, " {n}={}", x.value)?,
                        None => write!(f, " {}", x.value)?,
                    }
                }
                write!(f, ")")
            }
            Expr::Member(a, n) => write!(f, "(. {a} {n})"),
            Expr::Matrix(linhas) => {
                write!(f, "[")?;
                for (k, l) in linhas.iter().enumerate() {
                    if k > 0 {
                        write!(f, ";")?;
                    }
                    for (j, x) in l.iter().enumerate() {
                        if j > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{x}")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}

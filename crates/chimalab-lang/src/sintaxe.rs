//! Expressões e instruções da linguagem: AST e parser (DT-37 e DT-39).
//!
//! Parser descendente com precedência por níveis sobre os tokens do [`crate::lexico`]. Os blocos
//! seguem a indentação (`Indent`/`Dedent` do léxico), como no Python.

use std::fmt;

use crate::lexico::{FPart, LexError, Op, Tok, Token, tokenize};

/// Operador unário.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    /// `-x`
    Neg,
    /// `+x`
    Plus,
    /// `not x` ou `!x` (negação lógica)
    Not,
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
    /// `&` (e lógico elemento a elemento)
    And,
    /// `|` (ou lógico elemento a elemento)
    Or,
    /// `and` ou `&&` (com curto-circuito)
    AndAlso,
    /// `or` ou `||` (com curto-circuito)
    OrElse,
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
            BinOp::And => "&",
            BinOp::Or => "|",
            BinOp::AndAlso => "and",
            BinOp::OrElse => "or",
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
    /// Função anônima `x -> corpo` ou `(x, y) -> corpo`.
    Lambda(Vec<String>, Box<Node>),
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
    /// Palavra reservada usada como nome.
    ReservedWord(String),
    /// Atribuição a algo que não é nome, índice `A[…]` nem membro `a.b`.
    InvalidTarget,
    /// Faltou o corpo indentado depois de `:`.
    ExpectedBlock,
    /// Indentação sem um `:` que abra bloco.
    UnexpectedIndent,
    /// `break` ou `continue` fora de laço.
    OutsideLoop(&'static str),
    /// `return` fora de função.
    ReturnOutsideFunction,
    /// `elif`, `else`, `except` ou `case` sem a instrução que os introduz.
    OrphanClause(&'static str),
    /// Parâmetro obrigatório depois de um com valor padrão.
    RequiredAfterDefault,
    /// Parâmetro repetido.
    DuplicateParam(String),
    /// `case _:` que não é o último.
    DefaultCaseNotLast,
    /// Aninhamento além de [`MAX_ANINHAMENTO`] níveis (protege a pilha).
    NestingTooDeep,
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
            SyntaxErrorKind::ReservedWord(_) => "chimalab:sintaxe:palavraReservada",
            SyntaxErrorKind::InvalidTarget => "chimalab:sintaxe:alvoInvalido",
            SyntaxErrorKind::ExpectedBlock => "chimalab:sintaxe:blocoEsperado",
            SyntaxErrorKind::UnexpectedIndent => "chimalab:sintaxe:indentacaoInesperada",
            SyntaxErrorKind::OutsideLoop(_) => "chimalab:sintaxe:foraDeLaco",
            SyntaxErrorKind::ReturnOutsideFunction => "chimalab:sintaxe:returnForaDeFuncao",
            SyntaxErrorKind::OrphanClause(_) => "chimalab:sintaxe:clausulaSemInstrucao",
            SyntaxErrorKind::RequiredAfterDefault => "chimalab:sintaxe:obrigatorioDepoisDePadrao",
            SyntaxErrorKind::DuplicateParam(_) => "chimalab:sintaxe:parametroRepetido",
            SyntaxErrorKind::DefaultCaseNotLast => "chimalab:sintaxe:casoPadraoNaoUltimo",
            SyntaxErrorKind::NestingTooDeep => "chimalab:sintaxe:aninhamentoProfundo",
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
            SyntaxErrorKind::UnexpectedEnd => "o texto terminou antes do esperado".into(),
            SyntaxErrorKind::ChainedComparison => {
                "comparações não se encadeiam: escreva (a < b) e (b < c) separadamente".into()
            }
            SyntaxErrorKind::EndOutsideIndex => "`end` só vale dentro de um índice A[…]".into(),
            SyntaxErrorKind::PositionalAfterNamed => "argumento posicional depois de um nomeado".into(),
            SyntaxErrorKind::TildeNotEqual => "o diferente se escreve `!=`".into(),
            SyntaxErrorKind::ReservedWord(w) => {
                format!("`{w}` é palavra reservada e não pode ser usada como nome")
            }
            SyntaxErrorKind::InvalidTarget => {
                "só se atribui a um nome, a um índice A[…] ou a um membro a.b (índices usam colchetes)".into()
            }
            SyntaxErrorKind::ExpectedBlock => {
                "esperava o corpo do bloco, indentado, na linha seguinte".into()
            }
            SyntaxErrorKind::UnexpectedIndent => {
                "indentação inesperada: só se indenta depois de uma linha terminada em `:`".into()
            }
            SyntaxErrorKind::OutsideLoop(w) => format!("`{w}` só vale dentro de `for` ou `while`"),
            SyntaxErrorKind::ReturnOutsideFunction => "`return` só vale dentro de `def`".into(),
            SyntaxErrorKind::OrphanClause(w) => {
                let dono = match *w {
                    "elif" | "else" => "`if`",
                    "except" => "`try`",
                    _ => "`match`",
                };
                format!("`{w}` sem o {dono} correspondente, no mesmo nível de indentação")
            }
            SyntaxErrorKind::RequiredAfterDefault => {
                "parâmetro sem valor padrão depois de um com valor padrão".into()
            }
            SyntaxErrorKind::DuplicateParam(n) => format!("parâmetro `{n}` repetido"),
            SyntaxErrorKind::DefaultCaseNotLast => "`case _:` precisa ser o último caso".into(),
            SyntaxErrorKind::NestingTooDeep => {
                format!("mais de {MAX_ANINHAMENTO} níveis de aninhamento; divida a expressão ou o bloco")
            }
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
/// No cabeçalho de um bloco (`if c:`, `for k in a:b:`), o `:` seguido do fim da linha fecha o
/// cabeçalho em vez de continuar um intervalo.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Normal,
    Matrix,
    Index,
    Header,
}

/// Níveis máximos de aninhamento (parênteses, colchetes, sinais, `not` e blocos): acima disso o
/// parser recusa em vez de estourar a pilha.
pub const MAX_ANINHAMENTO: usize = 100;

pub use crate::lexico::KEYWORDS;

/// Se `w` é palavra reservada.
pub fn is_keyword(w: &str) -> bool {
    KEYWORDS.contains(&w)
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
    ctx: Vec<Ctx>,
    /// Laços abertos na função atual (para `break` e `continue`).
    loops: usize,
    /// Funções abertas (para `return`).
    funcs: usize,
    /// Aninhamento atual.
    prof: usize,
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

    fn is_kw(&self, w: &str) -> bool {
        matches!(&self.peek().tok, Tok::Ident(s) if s == w)
    }

    fn err_at(&self, kind: SyntaxErrorKind, t: &Token) -> SyntaxError {
        SyntaxError {
            kind,
            line: t.line,
            col: t.col,
        }
    }

    /// Um nome que não seja palavra reservada.
    fn name(&mut self, expected: &'static str) -> Result<String, SyntaxError> {
        let t = self.peek().clone();
        match &t.tok {
            Tok::Ident(s) if is_keyword(s) => Err(self.err_at(SyntaxErrorKind::ReservedWord(s.clone()), &t)),
            Tok::Ident(s) => {
                self.next();
                Ok(s.clone())
            }
            _ => Err(self.unexpected(expected)),
        }
    }

    /// Entra num nível de aninhamento.
    fn entra(&mut self) -> Result<(), SyntaxError> {
        self.prof += 1;
        if self.prof > MAX_ANINHAMENTO {
            let t = self.peek().clone();
            return Err(self.err_at(SyntaxErrorKind::NestingTooDeep, &t));
        }
        Ok(())
    }

    /// Expressão completa: função anônima ou `or`.
    fn expr(&mut self) -> Result<Node, SyntaxError> {
        self.entra()?;
        let r = self.expr_sem_limite();
        self.prof -= 1;
        r
    }

    fn expr_sem_limite(&mut self) -> Result<Node, SyntaxError> {
        if let Some(params) = self.lambda_params() {
            let t = self.peek().clone();
            // Consome os parâmetros e a seta.
            while !self.is_op(Op::Arrow) {
                self.next();
            }
            self.next();
            let corpo = self.expr()?;
            return Ok(node(Expr::Lambda(params, Box::new(corpo)), &t));
        }
        self.or_expr()
    }

    /// Reconhece `x ->`, `(x, y) ->` e `() ->` sem consumir nada.
    fn lambda_params(&self) -> Option<Vec<String>> {
        let toks = &self.toks[self.i..];
        let nome = |t: &Token| match &t.tok {
            Tok::Ident(s) if !is_keyword(s) => Some(s.clone()),
            _ => None,
        };
        if let (Some(n), Some(seta)) = (toks.first().and_then(nome), toks.get(1)) {
            return (seta.tok == Tok::Op(Op::Arrow)).then(|| vec![n]);
        }
        if toks.first()?.tok != Tok::Op(Op::LParen) {
            return None;
        }
        let mut params = Vec::new();
        let mut k = 1;
        if toks.get(k)?.tok != Tok::Op(Op::RParen) {
            loop {
                params.push(nome(toks.get(k)?)?);
                k += 1;
                match toks.get(k)?.tok {
                    Tok::Op(Op::Comma) => k += 1,
                    Tok::Op(Op::RParen) => break,
                    _ => return None,
                }
            }
        }
        (toks.get(k + 1)?.tok == Tok::Op(Op::Arrow)).then_some(params)
    }

    fn or_expr(&mut self) -> Result<Node, SyntaxError> {
        let mut l = self.and_expr()?;
        while self.is_kw("or") || self.is_op(Op::PipePipe) {
            self.next();
            let r = self.and_expr()?;
            l = bin(BinOp::OrElse, l, r);
        }
        Ok(l)
    }

    fn and_expr(&mut self) -> Result<Node, SyntaxError> {
        let mut l = self.elementwise_or()?;
        while self.is_kw("and") || self.is_op(Op::AmpAmp) {
            self.next();
            let r = self.elementwise_or()?;
            l = bin(BinOp::AndAlso, l, r);
        }
        Ok(l)
    }

    fn elementwise_or(&mut self) -> Result<Node, SyntaxError> {
        let mut l = self.elementwise_and()?;
        while self.is_op(Op::Pipe) {
            self.next();
            let r = self.elementwise_and()?;
            l = bin(BinOp::Or, l, r);
        }
        Ok(l)
    }

    fn elementwise_and(&mut self) -> Result<Node, SyntaxError> {
        let mut l = self.not_expr()?;
        while self.is_op(Op::Amp) {
            self.next();
            let r = self.not_expr()?;
            l = bin(BinOp::And, l, r);
        }
        Ok(l)
    }

    /// `not`/`!` ficam abaixo da comparação: `not a == b` é `not (a == b)`, e acima de `&`:
    /// `!a & b` é `(!a) & b`.
    fn not_expr(&mut self) -> Result<Node, SyntaxError> {
        if self.is_kw("not") || self.is_op(Op::Bang) {
            let t = self.next();
            self.entra()?;
            let e = self.not_expr()?;
            self.prof -= 1;
            return Ok(node(Expr::Unary(UnOp::Not, Box::new(e)), &t));
        }
        self.comparison()
    }

    /// O `:` em `pos` fecha o cabeçalho de um bloco.
    fn closes_header(&self) -> bool {
        self.ctx() == Ctx::Header
            && self.is_op(Op::Colon)
            && matches!(self.peek_at(1).tok, Tok::Newline | Tok::Eof)
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
        if !self.is_op(Op::Colon) || self.starts_element() || self.closes_header() {
            return Ok(start);
        }
        self.next();
        let b = self.additive()?;
        let (step, stop) = if self.is_op(Op::Colon) && !self.closes_header() {
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
        self.entra()?;
        let e = self.unary()?;
        self.prof -= 1;
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
        self.entra()?;
        let exp = self.unary()?;
        self.prof -= 1;
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
            Tok::Ident(s) if is_keyword(s) => {
                return Err(self.err_at(SyntaxErrorKind::ReservedWord(s.clone()), &t));
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
        Expr::Unary(_, a) | Expr::Transpose { expr: a, .. } | Expr::Member(a, _) | Expr::Lambda(_, a) => {
            reposiciona(a, f)
        }
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
    let mut p = Parser::new(toks);
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
            Expr::Unary(UnOp::Not, a) => write!(f, "(not {a})"),
            Expr::Lambda(ps, corpo) => write!(f, "(-> ({}) {corpo})", ps.join(" ")),
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

// ------------------------------------------------------------------ instruções (DT-39)

/// Parâmetro de função.
#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    /// Nome.
    pub name: String,
    /// Valor padrão.
    pub default: Option<Node>,
}

/// Caso de `match`: valores alternativos e corpo.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    /// Valores (o caso vale se o sujeito for igual a qualquer um).
    pub values: Vec<Node>,
    /// Corpo.
    pub body: Vec<Stmt>,
}

/// Tipo de instrução.
#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    /// Expressão (no console, o valor vai para `ans` e é exibido).
    Expr(Node),
    /// `a = e` ou `a, b = f(x)` (vários alvos recebem as saídas da chamada).
    Assign(Vec<Node>, Node),
    /// `a += e` e afins: o operador é `+`, `-`, `*` ou `/`.
    OpAssign(Node, BinOp, Node),
    /// `if`/`elif`/`else`: ramos (condição, corpo) e o `else`.
    If(Vec<(Node, Vec<Stmt>)>, Option<Vec<Stmt>>),
    /// `for nome in expr:` (percorre as colunas, DT-39).
    For(String, Node, Vec<Stmt>),
    /// `while cond:`.
    While(Node, Vec<Stmt>),
    /// `break`.
    Break,
    /// `continue`.
    Continue,
    /// `pass`.
    Pass,
    /// `return` com zero ou mais valores.
    Return(Vec<Node>),
    /// `try:` … `except nome:` …
    Try(Vec<Stmt>, Option<String>, Vec<Stmt>),
    /// `match sujeito:` com os casos e o `case _:` opcional.
    Match(Node, Vec<Case>, Option<Vec<Stmt>>),
    /// `def nome(parâmetros):` corpo.
    Def(String, Vec<Param>, Vec<Stmt>),
}

/// Instrução com posição e exibição.
#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    /// A instrução.
    pub kind: StmtKind,
    /// Linha (1-based).
    pub line: usize,
    /// Coluna (1-based).
    pub col: usize,
    /// Se o resultado é exibido no console: expressões e atribuições sem `;` no fim.
    pub show: bool,
}

impl Parser {
    fn new(toks: Vec<Token>) -> Self {
        Self {
            toks,
            i: 0,
            ctx: Vec::new(),
            loops: 0,
            funcs: 0,
            prof: 0,
        }
    }

    /// Instruções até `Dedent` (bloco) ou `Eof`.
    fn statements(&mut self) -> Result<Vec<Stmt>, SyntaxError> {
        let mut out = Vec::new();
        loop {
            let t = self.peek().clone();
            match t.tok {
                Tok::Newline | Tok::Op(Op::Semicolon) => {
                    self.next();
                }
                Tok::Dedent | Tok::Eof => return Ok(out),
                Tok::Indent => return Err(self.err_at(SyntaxErrorKind::UnexpectedIndent, &t)),
                _ => self.statement(&mut out)?,
            }
        }
    }

    /// Uma instrução composta, ou as instruções simples de uma linha (separadas por `;`).
    fn statement(&mut self, out: &mut Vec<Stmt>) -> Result<(), SyntaxError> {
        let t = self.peek().clone();
        if let Tok::Ident(w) = &t.tok {
            let atribui = matches!(
                self.peek_at(1).tok,
                Tok::Op(Op::Assign | Op::PlusAssign | Op::MinusAssign | Op::StarAssign | Op::SlashAssign)
            );
            if atribui && is_keyword(w) {
                return Err(self.err_at(SyntaxErrorKind::ReservedWord(w.clone()), &t));
            }
            let composta = match w.as_str() {
                "if" => Some(self.if_stmt()?),
                "for" => Some(self.for_stmt()?),
                "while" => Some(self.while_stmt()?),
                "try" => Some(self.try_stmt()?),
                "match" => Some(self.match_stmt()?),
                "def" => Some(self.def_stmt()?),
                "elif" => return Err(self.err_at(SyntaxErrorKind::OrphanClause("elif"), &t)),
                "else" => return Err(self.err_at(SyntaxErrorKind::OrphanClause("else"), &t)),
                "except" => return Err(self.err_at(SyntaxErrorKind::OrphanClause("except"), &t)),
                "case" => return Err(self.err_at(SyntaxErrorKind::OrphanClause("case"), &t)),
                _ => None,
            };
            if let Some(s) = composta {
                out.push(s);
                return Ok(());
            }
        }
        loop {
            let mut s = self.simple()?;
            if self.is_op(Op::Semicolon) {
                self.next();
                s.show = false;
                out.push(s);
                if matches!(self.peek().tok, Tok::Newline | Tok::Dedent | Tok::Eof) {
                    return Ok(());
                }
                continue;
            }
            out.push(s);
            return match self.peek().tok {
                // O léxico sempre emite `Newline` antes de `Dedent` e de `Eof`.
                Tok::Newline => {
                    self.next();
                    Ok(())
                }
                _ => Err(self.unexpected("`;` ou o fim da linha")),
            };
        }
    }

    fn at_simple_end(&self) -> bool {
        matches!(
            self.peek().tok,
            Tok::Newline | Tok::Dedent | Tok::Eof | Tok::Op(Op::Semicolon)
        )
    }

    fn simple(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.peek().clone();
        let mk = |kind, show| Stmt {
            kind,
            line: t.line,
            col: t.col,
            show,
        };
        for (w, kind) in [("break", StmtKind::Break), ("continue", StmtKind::Continue)] {
            if self.is_kw(w) {
                if self.loops == 0 {
                    return Err(self.err_at(SyntaxErrorKind::OutsideLoop(w), &t));
                }
                self.next();
                return Ok(mk(kind, false));
            }
        }
        if self.is_kw("pass") {
            self.next();
            return Ok(mk(StmtKind::Pass, false));
        }
        if self.is_kw("return") {
            if self.funcs == 0 {
                return Err(self.err_at(SyntaxErrorKind::ReturnOutsideFunction, &t));
            }
            self.next();
            let mut vals = Vec::new();
            if !self.at_simple_end() {
                vals.push(self.expr()?);
                while self.is_op(Op::Comma) {
                    self.next();
                    vals.push(self.expr()?);
                }
            }
            return Ok(mk(StmtKind::Return(vals), false));
        }
        let primeiro = self.expr()?;
        let mut alvos = vec![primeiro];
        while self.is_op(Op::Comma) {
            self.next();
            alvos.push(self.expr()?);
        }
        let op = match self.peek().tok {
            Tok::Op(Op::Assign) => None,
            Tok::Op(Op::PlusAssign) => Some(BinOp::Add),
            Tok::Op(Op::MinusAssign) => Some(BinOp::Sub),
            Tok::Op(Op::StarAssign) => Some(BinOp::MatMul),
            Tok::Op(Op::SlashAssign) => Some(BinOp::MatDiv),
            _ if alvos.len() > 1 => return Err(self.unexpected("`=` depois dos alvos")),
            _ => return Ok(mk(StmtKind::Expr(alvos.pop().expect("um")), true)),
        };
        if op.is_some() && alvos.len() > 1 {
            return Err(self.unexpected("`=` (atribuição composta tem um alvo só)"));
        }
        for a in &alvos {
            if !alvo_valido(a) {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::InvalidTarget,
                    line: a.line,
                    col: a.col,
                });
            }
        }
        self.next();
        let valor = self.expr()?;
        Ok(match op {
            None => mk(StmtKind::Assign(alvos, valor), true),
            Some(o) => mk(StmtKind::OpAssign(alvos.pop().expect("um"), o, valor), true),
        })
    }

    fn header_expr(&mut self) -> Result<Node, SyntaxError> {
        self.ctx.push(Ctx::Header);
        let e = self.expr();
        self.ctx.pop();
        e
    }

    /// `:`, fim de linha e o corpo indentado; consome o `Dedent` final.
    fn block(&mut self) -> Result<Vec<Stmt>, SyntaxError> {
        self.open_block()?;
        self.entra()?;
        let corpo = self.statements()?;
        self.prof -= 1;
        self.next();
        Ok(corpo)
    }

    /// `:`, fim de linha e `Indent`.
    fn open_block(&mut self) -> Result<(), SyntaxError> {
        if !self.is_op(Op::Colon) {
            // Mesmo no fim da linha, o problema é o `:` que falta, não o fim do texto.
            let t = self.peek();
            return Err(self.err_at(
                SyntaxErrorKind::Unexpected {
                    expected: "`:` no fim da linha",
                    found: descreve(&t.tok),
                },
                t,
            ));
        }
        self.next();
        if self.peek().tok != Tok::Newline {
            return Err(
                self.unexpected("o fim da linha depois de `:` (o corpo vai indentado nas linhas seguintes)")
            );
        }
        self.next();
        let t = self.peek().clone();
        match t.tok {
            Tok::Indent => {
                self.next();
                Ok(())
            }
            Tok::Eof => Err(self.err_at(SyntaxErrorKind::UnexpectedEnd, &t)),
            _ => Err(self.err_at(SyntaxErrorKind::ExpectedBlock, &t)),
        }
    }

    fn stmt_at(t: &Token, kind: StmtKind) -> Stmt {
        Stmt {
            kind,
            line: t.line,
            col: t.col,
            show: false,
        }
    }

    fn if_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.next();
        let c = self.header_expr()?;
        let mut ramos = vec![(c, self.block()?)];
        let mut senao = None;
        loop {
            if self.is_kw("elif") {
                self.next();
                let c = self.header_expr()?;
                ramos.push((c, self.block()?));
            } else if self.is_kw("else") {
                self.next();
                senao = Some(self.block()?);
                break;
            } else {
                break;
            }
        }
        Ok(Self::stmt_at(&t, StmtKind::If(ramos, senao)))
    }

    fn loop_body(&mut self) -> Result<Vec<Stmt>, SyntaxError> {
        self.loops += 1;
        let corpo = self.block();
        self.loops -= 1;
        corpo
    }

    fn for_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.next();
        let var = self.name("o nome da variável do laço")?;
        if !self.is_kw("in") {
            return Err(self.unexpected("`in`"));
        }
        self.next();
        let it = self.header_expr()?;
        let corpo = self.loop_body()?;
        Ok(Self::stmt_at(&t, StmtKind::For(var, it, corpo)))
    }

    fn while_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.next();
        let c = self.header_expr()?;
        let corpo = self.loop_body()?;
        Ok(Self::stmt_at(&t, StmtKind::While(c, corpo)))
    }

    fn try_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.next();
        let corpo = self.block()?;
        if !self.is_kw("except") {
            return Err(self.unexpected("`except`"));
        }
        self.next();
        let var = if self.is_op(Op::Colon) {
            None
        } else {
            Some(self.name("o nome do erro ou `:`")?)
        };
        let tratador = self.block()?;
        Ok(Self::stmt_at(&t, StmtKind::Try(corpo, var, tratador)))
    }

    fn match_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.next();
        let sujeito = self.header_expr()?;
        self.open_block()?;
        let mut casos = Vec::new();
        let mut padrao: Option<Vec<Stmt>> = None;
        while self.is_kw("case") {
            let c = self.next();
            if padrao.is_some() {
                return Err(self.err_at(SyntaxErrorKind::DefaultCaseNotLast, &c));
            }
            let curinga = matches!(&self.peek().tok, Tok::Ident(s) if s == "_")
                && self.peek_at(1).tok == Tok::Op(Op::Colon);
            if curinga {
                self.next();
                padrao = Some(self.block()?);
                continue;
            }
            let mut valores = vec![self.header_expr()?];
            while self.is_op(Op::Comma) {
                self.next();
                valores.push(self.header_expr()?);
            }
            casos.push(Case {
                values: valores,
                body: self.block()?,
            });
        }
        if casos.is_empty() && padrao.is_none() || self.peek().tok != Tok::Dedent {
            return Err(self.unexpected("`case`"));
        }
        self.next();
        Ok(Self::stmt_at(&t, StmtKind::Match(sujeito, casos, padrao)))
    }

    fn def_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let t = self.next();
        let nome = self.name("o nome da função")?;
        self.expect(Op::LParen, "`(`")?;
        let mut params: Vec<Param> = Vec::new();
        while !self.is_op(Op::RParen) {
            let pt = self.peek().clone();
            let n = self.name("o nome de um parâmetro")?;
            if params.iter().any(|p| p.name == n) {
                return Err(self.err_at(SyntaxErrorKind::DuplicateParam(n), &pt));
            }
            let default = if self.is_op(Op::Assign) {
                self.next();
                self.ctx.push(Ctx::Normal);
                let e = self.expr();
                self.ctx.pop();
                Some(e?)
            } else {
                if params.iter().any(|p| p.default.is_some()) {
                    return Err(self.err_at(SyntaxErrorKind::RequiredAfterDefault, &pt));
                }
                None
            };
            params.push(Param { name: n, default });
            if self.is_op(Op::Comma) {
                self.next();
            } else if !self.is_op(Op::RParen) {
                return Err(self.unexpected("`,` ou `)`"));
            }
        }
        self.next();
        let (laços, funcs) = (self.loops, self.funcs);
        self.loops = 0;
        self.funcs += 1;
        let corpo = self.block();
        (self.loops, self.funcs) = (laços, funcs);
        Ok(Self::stmt_at(&t, StmtKind::Def(nome, params, corpo?)))
    }
}

/// Alvo de atribuição: nome, ou índice/membro de um alvo.
fn alvo_valido(n: &Node) -> bool {
    match &n.expr {
        Expr::Ident(_) => true,
        Expr::Index(base, _) | Expr::Member(base, _) => alvo_valido(base),
        _ => false,
    }
}

/// Analisa um programa (script, função ou entrada do console).
///
/// Erros: ver [`SyntaxErrorKind`], com identificadores `chimalab:sintaxe:…` (e os
/// `chimalab:lexico:…` do léxico).
pub fn parse_program(src: &str) -> Result<Vec<Stmt>, SyntaxError> {
    let mut p = Parser::new(tokenize(src)?);
    let prog = p.statements()?;
    // O léxico fecha todos os blocos antes do fim: só pode restar o `Eof`.
    debug_assert_eq!(p.peek().tok, Tok::Eof);
    Ok(prog)
}

fn bloco(f: &mut fmt::Formatter<'_>, corpo: &[Stmt]) -> fmt::Result {
    write!(f, "(")?;
    for (k, s) in corpo.iter().enumerate() {
        if k > 0 {
            write!(f, " ")?;
        }
        write!(f, "{s}")?;
    }
    write!(f, ")")
}

/// Forma canônica em S-expressão; `;` no fim marca instrução sem exibição.
impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            StmtKind::Expr(e) => write!(f, "{e}")?,
            StmtKind::Assign(alvos, v) if alvos.len() == 1 => write!(f, "(= {} {v})", alvos[0])?,
            StmtKind::Assign(alvos, v) => {
                let a: Vec<String> = alvos.iter().map(|x| x.to_string()).collect();
                write!(f, "(= ({}) {v})", a.join(" "))?
            }
            StmtKind::OpAssign(a, op, v) => write!(f, "({}= {a} {v})", op.text())?,
            StmtKind::If(ramos, senao) => {
                write!(f, "(if")?;
                for (c, corpo) in ramos {
                    write!(f, " ({c} ")?;
                    bloco(f, corpo)?;
                    write!(f, ")")?;
                }
                if let Some(s) = senao {
                    write!(f, " (else ")?;
                    bloco(f, s)?;
                    write!(f, ")")?;
                }
                write!(f, ")")?
            }
            StmtKind::For(v, it, corpo) => {
                write!(f, "(for {v} {it} ")?;
                bloco(f, corpo)?;
                write!(f, ")")?
            }
            StmtKind::While(c, corpo) => {
                write!(f, "(while {c} ")?;
                bloco(f, corpo)?;
                write!(f, ")")?
            }
            StmtKind::Break => write!(f, "break")?,
            StmtKind::Continue => write!(f, "continue")?,
            StmtKind::Pass => write!(f, "pass")?,
            StmtKind::Return(vs) => {
                write!(f, "(return")?;
                for v in vs {
                    write!(f, " {v}")?;
                }
                write!(f, ")")?
            }
            StmtKind::Try(corpo, var, trat) => {
                write!(f, "(try ")?;
                bloco(f, corpo)?;
                write!(f, " (except {} ", var.as_deref().unwrap_or("_"))?;
                bloco(f, trat)?;
                write!(f, "))")?
            }
            StmtKind::Match(suj, casos, padrao) => {
                write!(f, "(match {suj}")?;
                for c in casos {
                    let v: Vec<String> = c.values.iter().map(|x| x.to_string()).collect();
                    write!(f, " (case ({}) ", v.join(" "))?;
                    bloco(f, &c.body)?;
                    write!(f, ")")?;
                }
                if let Some(p) = padrao {
                    write!(f, " (case _ ")?;
                    bloco(f, p)?;
                    write!(f, ")")?;
                }
                write!(f, ")")?
            }
            StmtKind::Def(nome, params, corpo) => {
                write!(f, "(def {nome} (")?;
                for (k, p) in params.iter().enumerate() {
                    if k > 0 {
                        write!(f, " ")?;
                    }
                    match &p.default {
                        Some(d) => write!(f, "{}={d}", p.name)?,
                        None => write!(f, "{}", p.name)?,
                    }
                }
                write!(f, ") ")?;
                bloco(f, corpo)?;
                write!(f, ")")?
            }
        }
        if !self.show
            && matches!(
                self.kind,
                StmtKind::Expr(_) | StmtKind::Assign(..) | StmtKind::OpAssign(..)
            )
        {
            write!(f, ";")?;
        }
        Ok(())
    }
}

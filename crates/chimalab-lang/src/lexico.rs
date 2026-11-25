//! Analisador léxico da linguagem ChimaLab (DT-36).
//!
//! Converte o texto-fonte em tokens com posição. Decisões (DT-22, DT-34, DT-36):
//! - todas as palavras saem como identificadores; as palavras-chave são do parser;
//! - blocos por indentação, com `Indent`/`Dedent` como no Python;
//! - dentro de `(` e `{` a quebra de linha é ignorada; dentro de `[` vira `Newline`
//!   (separador de linhas da matriz);
//! - cada token guarda se veio depois de espaço (dentro de colchetes, o espaço separa elementos);
//! - `'` é transposta depois de um valor (e, dentro de colchetes, sem espaço antes); senão abre
//!   um texto.

use std::fmt;

/// Operadores e pontuação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `\`
    Backslash,
    /// `^`
    Caret,
    /// `.*`
    DotStar,
    /// `./`
    DotSlash,
    /// `.\`
    DotBackslash,
    /// `.^`
    DotCaret,
    /// `'` (transposta conjugada)
    Quote,
    /// `.'` (transposta simples)
    DotQuote,
    /// `=`
    Assign,
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `~=`
    TildeEq,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `.`
    Dot,
    /// `@`
    At,
    /// `&`
    Amp,
    /// `&&`
    AmpAmp,
    /// `|`
    Pipe,
    /// `||`
    PipePipe,
    /// `!`
    Bang,
    /// `~`
    Tilde,
    /// `+=`
    PlusAssign,
    /// `-=`
    MinusAssign,
    /// `*=`
    StarAssign,
    /// `/=`
    SlashAssign,
}

impl Op {
    /// Grafia do operador.
    pub fn text(self) -> &'static str {
        match self {
            Op::Plus => "+",
            Op::Minus => "-",
            Op::Star => "*",
            Op::Slash => "/",
            Op::Backslash => "\\",
            Op::Caret => "^",
            Op::DotStar => ".*",
            Op::DotSlash => "./",
            Op::DotBackslash => ".\\",
            Op::DotCaret => ".^",
            Op::Quote => "'",
            Op::DotQuote => ".'",
            Op::Assign => "=",
            Op::Eq => "==",
            Op::Ne => "!=",
            Op::TildeEq => "~=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::Colon => ":",
            Op::Comma => ",",
            Op::Semicolon => ";",
            Op::LParen => "(",
            Op::RParen => ")",
            Op::LBracket => "[",
            Op::RBracket => "]",
            Op::LBrace => "{",
            Op::RBrace => "}",
            Op::Dot => ".",
            Op::At => "@",
            Op::Amp => "&",
            Op::AmpAmp => "&&",
            Op::Pipe => "|",
            Op::PipePipe => "||",
            Op::Bang => "!",
            Op::Tilde => "~",
            Op::PlusAssign => "+=",
            Op::MinusAssign => "-=",
            Op::StarAssign => "*=",
            Op::SlashAssign => "/=",
        }
    }
}

/// Parte de uma f-string.
#[derive(Clone, Debug, PartialEq)]
pub enum FPart {
    /// Texto literal (escapes já resolvidos).
    Lit(String),
    /// Expressão entre chaves: o texto-fonte, o formato opcional (depois de `:`) e a posição
    /// (linha e coluna) do primeiro caractere da expressão.
    Expr {
        /// Texto-fonte da expressão.
        src: String,
        /// Especificação de formato, sem o `:`.
        format: Option<String>,
        /// Linha (1-based).
        line: usize,
        /// Coluna (1-based, em caracteres).
        col: usize,
    },
}

/// Tipo de token.
#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    /// Identificador (inclui as futuras palavras-chave).
    Ident(String),
    /// Número real; `text` é a grafia original (sem `_`).
    Number {
        /// Valor (arredondado para f64).
        value: f64,
        /// Grafia original, sem os `_`.
        text: String,
    },
    /// Número imaginário (sufixo `i` ou `j`): o valor da parte imaginária.
    Imag {
        /// Parte imaginária.
        value: f64,
        /// Grafia original, sem os `_` e sem o sufixo.
        text: String,
    },
    /// Texto.
    Str(String),
    /// f-string.
    FStr(Vec<FPart>),
    /// Operador ou pontuação.
    Op(Op),
    /// Fim de linha lógica (ou separador de linhas dentro de `[ ]`).
    Newline,
    /// Abre um bloco.
    Indent,
    /// Fecha um bloco.
    Dedent,
    /// Fim do texto.
    Eof,
}

/// Token com posição.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    /// O token.
    pub tok: Tok,
    /// Linha (1-based).
    pub line: usize,
    /// Coluna (1-based, em caracteres).
    pub col: usize,
    /// Intervalo em bytes no texto-fonte.
    pub span: (usize, usize),
    /// Se havia espaço (ou comentário) antes, na mesma linha.
    pub space_before: bool,
}

/// Tipo de erro léxico.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    /// Texto sem a aspa de fechamento na mesma linha.
    UnterminatedString,
    /// Comentário `#=` sem `=#`.
    UnterminatedComment,
    /// Caractere que não pertence à linguagem.
    InvalidChar(char),
    /// Recuo para uma coluna que não abriu bloco.
    InconsistentIndent,
    /// Número malformado (ex.: `2x`, `0x` sem dígitos).
    InvalidNumber,
    /// Escape desconhecido num texto.
    InvalidEscape,
    /// f-string com chave sem par ou expressão vazia.
    InvalidFString,
    /// Fechamento sem a abertura correspondente.
    Unbalanced,
    /// Abertura sem fechamento até o fim do texto.
    Unclosed,
}

/// Erro léxico com posição.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexError {
    /// Tipo.
    pub kind: LexErrorKind,
    /// Linha (1-based).
    pub line: usize,
    /// Coluna (1-based, em caracteres).
    pub col: usize,
}

impl LexError {
    /// Identificador estável do erro.
    pub fn id(&self) -> &'static str {
        match self.kind {
            LexErrorKind::UnterminatedString => "chimalab:lexico:textoNaoTerminado",
            LexErrorKind::UnterminatedComment => "chimalab:lexico:comentarioNaoTerminado",
            LexErrorKind::InvalidChar(_) => "chimalab:lexico:caractereInvalido",
            LexErrorKind::InconsistentIndent => "chimalab:lexico:indentacaoInconsistente",
            LexErrorKind::InvalidNumber => "chimalab:lexico:numeroInvalido",
            LexErrorKind::InvalidEscape => "chimalab:lexico:escapeInvalido",
            LexErrorKind::InvalidFString => "chimalab:lexico:fstringInvalida",
            LexErrorKind::Unbalanced => "chimalab:lexico:delimitadorDesbalanceado",
            LexErrorKind::Unclosed => "chimalab:lexico:delimitadorNaoFechado",
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match &self.kind {
            LexErrorKind::UnterminatedString => "texto sem aspa de fechamento".to_string(),
            LexErrorKind::UnterminatedComment => "comentário #= sem =#".to_string(),
            LexErrorKind::InvalidChar(c) => format!("caractere inválido {c:?}"),
            LexErrorKind::InconsistentIndent => {
                "indentação não corresponde a nenhum bloco aberto".to_string()
            }
            LexErrorKind::InvalidNumber => "número malformado".to_string(),
            LexErrorKind::InvalidEscape => "escape desconhecido".to_string(),
            LexErrorKind::InvalidFString => "f-string malformada".to_string(),
            LexErrorKind::Unbalanced => "fechamento sem a abertura correspondente".to_string(),
            LexErrorKind::Unclosed => "abertura sem fechamento".to_string(),
        };
        write!(f, "linha {}, coluna {}: {msg}", self.line, self.col)
    }
}

impl std::error::Error for LexError {}

/// Largura de tabulação na indentação (próximo múltiplo).
const TAB: usize = 4;

struct Lexer<'s> {
    src: &'s str,
    chars: Vec<(usize, char)>,
    i: usize,
    line: usize,
    col: usize,
    out: Vec<Token>,
    brackets: Vec<(char, usize, usize)>,
    indents: Vec<usize>,
    pending_indent: Option<usize>,
    line_has_token: bool,
    space: bool,
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

impl<'s> Lexer<'s> {
    fn new(src: &'s str) -> Self {
        Self {
            src,
            chars: src.char_indices().collect(),
            i: 0,
            line: 1,
            col: 1,
            out: Vec::new(),
            brackets: Vec::new(),
            indents: vec![0],
            pending_indent: Some(0),
            line_has_token: false,
            space: false,
        }
    }

    fn peek(&self, k: usize) -> Option<char> {
        self.chars.get(self.i + k).map(|&(_, c)| c)
    }

    fn byte(&self) -> usize {
        self.chars.get(self.i).map_or(self.src.len(), |&(b, _)| b)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek(0)?;
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn err(&self, kind: LexErrorKind, line: usize, col: usize) -> LexError {
        LexError { kind, line, col }
    }

    /// Emite um token real: antes, resolve a indentação pendente da linha.
    fn emit(&mut self, tok: Tok, line: usize, col: usize, start: usize) -> Result<(), LexError> {
        if let Some(w) = self.pending_indent.take() {
            let topo = *self.indents.last().expect("pilha com a base 0");
            if w > topo {
                self.indents.push(w);
                self.push_raw(Tok::Indent, line, col, start);
            } else {
                while w < *self.indents.last().expect("base") {
                    self.indents.pop();
                    self.push_raw(Tok::Dedent, line, col, start);
                }
                if w != *self.indents.last().expect("base") {
                    return Err(self.err(LexErrorKind::InconsistentIndent, line, col));
                }
            }
        }
        let end = self.byte();
        self.out.push(Token {
            tok,
            line,
            col,
            span: (start, end),
            space_before: self.space,
        });
        self.space = false;
        self.line_has_token = true;
        Ok(())
    }

    fn push_raw(&mut self, tok: Tok, line: usize, col: usize, at: usize) {
        self.out.push(Token {
            tok,
            line,
            col,
            span: (at, at),
            space_before: false,
        });
    }

    /// O último token termina um valor (para decidir se `'` é transposta).
    fn prev_is_value(&self) -> bool {
        matches!(
            self.out.last().map(|t| &t.tok),
            Some(
                Tok::Ident(_)
                    | Tok::Number { .. }
                    | Tok::Imag { .. }
                    | Tok::Str(_)
                    | Tok::FStr(_)
                    | Tok::Op(Op::RParen | Op::RBracket | Op::RBrace | Op::Quote | Op::DotQuote)
            )
        )
    }

    fn in_brackets(&self) -> bool {
        self.brackets.last().is_some_and(|b| b.0 == '[')
    }

    fn run(mut self) -> Result<Vec<Token>, LexError> {
        while let Some(c) = self.peek(0) {
            let (line, col, start) = (self.line, self.col, self.byte());
            match c {
                '\n' => {
                    self.bump();
                    match self.brackets.last().map(|b| b.0) {
                        None => {
                            if self.line_has_token {
                                self.push_raw(Tok::Newline, line, col, start);
                            }
                            self.line_has_token = false;
                            self.pending_indent = Some(self.measure_indent());
                        }
                        Some('[') => {
                            self.push_raw(Tok::Newline, line, col, start);
                            self.space = false;
                        }
                        Some(_) => {}
                    }
                    self.space = false;
                }
                ' ' | '\t' | '\r' => {
                    self.bump();
                    self.space = true;
                }
                '#' => {
                    self.comment()?;
                    self.space = true;
                }
                '0'..='9' => self.number(line, col, start)?,
                '.' if self.peek(1).is_some_and(|d| d.is_ascii_digit()) => self.number(line, col, start)?,
                '"' => {
                    self.bump();
                    let s = self.string('"', false, line, col)?;
                    self.emit(Tok::Str(s), line, col, start)?;
                }
                '\'' => {
                    let transposta = self.prev_is_value() && !(self.in_brackets() && self.space);
                    self.bump();
                    if transposta {
                        self.emit(Tok::Op(Op::Quote), line, col, start)?;
                    } else {
                        let s = self.string('\'', false, line, col)?;
                        self.emit(Tok::Str(s), line, col, start)?;
                    }
                }
                c if is_ident_start(c) => self.word(line, col, start)?,
                _ => self.operator(line, col, start)?,
            }
        }
        if let Some(&(_, l, c)) = self.brackets.last() {
            return Err(self.err(LexErrorKind::Unclosed, l, c));
        }
        let (line, col, at) = (self.line, self.col, self.src.len());
        if self.line_has_token {
            self.push_raw(Tok::Newline, line, col, at);
        }
        while self.indents.len() > 1 {
            self.indents.pop();
            self.push_raw(Tok::Dedent, line, col, at);
        }
        self.push_raw(Tok::Eof, line, col, at);
        Ok(self.out)
    }

    /// Largura da indentação no início da linha (consome espaços e tabulações).
    fn measure_indent(&mut self) -> usize {
        let mut w = 0;
        loop {
            match self.peek(0) {
                Some(' ') => {
                    self.bump();
                    w += 1;
                }
                Some('\t') => {
                    self.bump();
                    w = (w / TAB + 1) * TAB;
                }
                _ => return w,
            }
        }
    }

    fn comment(&mut self) -> Result<(), LexError> {
        let (line, col) = (self.line, self.col);
        self.bump(); // '#'
        if self.peek(0) == Some('=') {
            self.bump();
            let mut prof = 1;
            while prof > 0 {
                match (self.peek(0), self.peek(1)) {
                    (Some('#'), Some('=')) => {
                        self.bump();
                        self.bump();
                        prof += 1;
                    }
                    (Some('='), Some('#')) => {
                        self.bump();
                        self.bump();
                        prof -= 1;
                    }
                    (Some(_), _) => {
                        self.bump();
                    }
                    (None, _) => return Err(self.err(LexErrorKind::UnterminatedComment, line, col)),
                }
            }
        } else {
            while self.peek(0).is_some_and(|c| c != '\n') {
                self.bump();
            }
        }
        Ok(())
    }

    fn digits(&mut self, radix: u32, s: &mut String) -> bool {
        let mut algum = false;
        while let Some(c) = self.peek(0) {
            if c.is_digit(radix) {
                s.push(c);
                algum = true;
                self.bump();
            } else if c == '_' && algum && self.peek(1).is_some_and(|d| d.is_digit(radix)) {
                self.bump();
            } else {
                break;
            }
        }
        algum
    }

    fn number(&mut self, line: usize, col: usize, start: usize) -> Result<(), LexError> {
        let mut text = String::new();
        let value;
        let prefixo = match (self.peek(0), self.peek(1)) {
            (Some('0'), Some('x' | 'X')) => Some(16),
            (Some('0'), Some('o' | 'O')) => Some(8),
            (Some('0'), Some('b' | 'B')) => Some(2),
            _ => None,
        };
        if let Some(radix) = prefixo {
            self.bump();
            self.bump();
            let mut d = String::new();
            if !self.digits(radix, &mut d) {
                return Err(self.err(LexErrorKind::InvalidNumber, line, col));
            }
            text = format!(
                "0{}{d}",
                match radix {
                    16 => 'x',
                    8 => 'o',
                    _ => 'b',
                }
            );
            value = u128::from_str_radix(&d, radix)
                .map_err(|_| self.err(LexErrorKind::InvalidNumber, line, col))? as f64;
        } else {
            self.digits(10, &mut text);
            // Ponto decimal, exceto quando pertence a um operador elemento a elemento (1.*2)
            // ou a `..`.
            if self.peek(0) == Some('.') {
                let depois = self.peek(1);
                let operador = matches!(depois, Some('*' | '/' | '\\' | '^' | '\'' | '.'));
                if !operador {
                    self.bump();
                    text.push('.');
                    self.digits(10, &mut text);
                }
            }
            if matches!(self.peek(0), Some('e' | 'E')) {
                let (s1, s2) = (self.peek(1), self.peek(2));
                let expoente = s1.is_some_and(|c| c.is_ascii_digit())
                    || (matches!(s1, Some('+' | '-')) && s2.is_some_and(|c| c.is_ascii_digit()));
                if expoente {
                    text.push('e');
                    self.bump();
                    if let Some(c @ ('+' | '-')) = self.peek(0) {
                        text.push(c);
                        self.bump();
                    }
                    self.digits(10, &mut text);
                }
            }
            value = text
                .parse::<f64>()
                .map_err(|_| self.err(LexErrorKind::InvalidNumber, line, col))?;
        }
        // Sufixo imaginário colado, sem continuar um identificador.
        if matches!(self.peek(0), Some('i' | 'j')) && !self.peek(1).is_some_and(is_ident_char) {
            self.bump();
            return self.emit(Tok::Imag { value, text }, line, col, start);
        }
        if self.peek(0).is_some_and(is_ident_char) {
            return Err(self.err(LexErrorKind::InvalidNumber, line, col));
        }
        self.emit(Tok::Number { value, text }, line, col, start)
    }

    fn word(&mut self, line: usize, col: usize, start: usize) -> Result<(), LexError> {
        let mut w = String::new();
        while let Some(c) = self.peek(0).filter(|&c| is_ident_char(c)) {
            w.push(c);
            self.bump();
        }
        // Prefixos de texto: f"…", r"…", rf"…", fr"…" (colados à aspa).
        if let Some(q @ ('"' | '\'')) = self.peek(0) {
            let (f, r) = match w.as_str() {
                "f" => (true, false),
                "r" => (false, true),
                "rf" | "fr" => (true, true),
                _ => (false, false),
            };
            if f || r {
                self.bump();
                let tok = if f {
                    Tok::FStr(self.fstring(q, r, line, col)?)
                } else {
                    Tok::Str(self.string(q, true, line, col)?)
                };
                return self.emit(tok, line, col, start);
            }
        }
        self.emit(Tok::Ident(w), line, col, start)
    }

    /// Escape depois de `\` (o `\` já consumido).
    fn escape(&mut self, out: &mut String) -> Result<(), LexError> {
        let (line, col) = (self.line, self.col.saturating_sub(1));
        let c = self
            .bump()
            .ok_or(self.err(LexErrorKind::UnterminatedString, line, col))?;
        match c {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '0' => out.push('\0'),
            '\\' => out.push('\\'),
            '\'' => out.push('\''),
            '"' => out.push('"'),
            'u' => {
                if self.bump() != Some('{') {
                    return Err(self.err(LexErrorKind::InvalidEscape, line, col));
                }
                let mut h = String::new();
                while let Some(d) = self.peek(0).filter(|d| d.is_ascii_hexdigit()) {
                    h.push(d);
                    self.bump();
                }
                if self.bump() != Some('}') || h.is_empty() || h.len() > 6 {
                    return Err(self.err(LexErrorKind::InvalidEscape, line, col));
                }
                let v = u32::from_str_radix(&h, 16).expect("hexadecimal");
                out.push(char::from_u32(v).ok_or(self.err(LexErrorKind::InvalidEscape, line, col))?);
            }
            _ => return Err(self.err(LexErrorKind::InvalidEscape, line, col)),
        }
        Ok(())
    }

    /// Texto até a aspa `q` (a de abertura já consumida). Não atravessa linhas.
    fn string(&mut self, q: char, raw: bool, line: usize, col: usize) -> Result<String, LexError> {
        let mut s = String::new();
        loop {
            match self.bump() {
                None | Some('\n') => return Err(self.err(LexErrorKind::UnterminatedString, line, col)),
                Some(c) if c == q => return Ok(s),
                Some('\\') if raw => {
                    // Cru: a barra fica, e uma aspa logo depois não fecha o texto.
                    s.push('\\');
                    if let Some(c) = self.peek(0).filter(|&c| c == q || c == '\\') {
                        s.push(c);
                        self.bump();
                    }
                }
                Some('\\') => self.escape(&mut s)?,
                Some(c) => s.push(c),
            }
        }
    }

    /// f-string até a aspa `q`.
    fn fstring(&mut self, q: char, raw: bool, line: usize, col: usize) -> Result<Vec<FPart>, LexError> {
        let mut partes = Vec::new();
        let mut lit = String::new();
        loop {
            let (l, c) = (self.line, self.col);
            match self.bump() {
                None | Some('\n') => return Err(self.err(LexErrorKind::UnterminatedString, line, col)),
                Some(ch) if ch == q => break,
                Some('{') if self.peek(0) == Some('{') => {
                    self.bump();
                    lit.push('{');
                }
                Some('}') if self.peek(0) == Some('}') => {
                    self.bump();
                    lit.push('}');
                }
                Some('}') => return Err(self.err(LexErrorKind::InvalidFString, l, c)),
                Some('{') => {
                    if !lit.is_empty() {
                        partes.push(FPart::Lit(std::mem::take(&mut lit)));
                    }
                    partes.push(self.fexpr(q, l, c)?);
                }
                Some('\\') if !raw => self.escape(&mut lit)?,
                Some(ch) => lit.push(ch),
            }
        }
        if !lit.is_empty() {
            partes.push(FPart::Lit(lit));
        }
        Ok(partes)
    }

    /// `{expr}` ou `{expr:formato}` (a chave de abertura já consumida).
    fn fexpr(&mut self, q: char, l0: usize, c0: usize) -> Result<FPart, LexError> {
        let (line, col) = (self.line, self.col);
        let mut src = String::new();
        let mut format = None;
        let mut prof = 0usize;
        let mut aspa: Option<char> = None;
        loop {
            let ch = match self.peek(0) {
                None | Some('\n') => return Err(self.err(LexErrorKind::InvalidFString, l0, c0)),
                Some(ch) => ch,
            };
            if let Some(a) = aspa {
                if ch == q {
                    return Err(self.err(LexErrorKind::InvalidFString, l0, c0));
                }
                self.bump();
                src.push(ch);
                if ch == a {
                    aspa = None;
                }
                continue;
            }
            match ch {
                '}' if prof == 0 => {
                    self.bump();
                    break;
                }
                ':' if prof == 0 && format.is_none() => {
                    self.bump();
                    let mut f = String::new();
                    while let Some(x) = self.peek(0).filter(|&x| x != '}' && x != '\n' && x != q) {
                        f.push(x);
                        self.bump();
                    }
                    format = Some(f);
                }
                c if c == q => return Err(self.err(LexErrorKind::InvalidFString, l0, c0)),
                // Na expressão, `'` depois de um valor é transposta (regra do DT-34), não texto.
                // Regressão: f"{A'}" dava erro de f-string.
                '\'' if src
                    .trim_end()
                    .chars()
                    .last()
                    .is_some_and(|p| is_ident_char(p) || matches!(p, ')' | ']' | '}' | '\'')) =>
                {
                    src.push(ch);
                    self.bump();
                }
                '\'' | '"' => {
                    aspa = Some(ch);
                    src.push(ch);
                    self.bump();
                }
                '(' | '[' | '{' => {
                    prof += 1;
                    src.push(ch);
                    self.bump();
                }
                ')' | ']' | '}' => {
                    prof = prof.saturating_sub(1);
                    src.push(ch);
                    self.bump();
                }
                _ => {
                    src.push(ch);
                    self.bump();
                }
            }
        }
        if src.trim().is_empty() {
            return Err(self.err(LexErrorKind::InvalidFString, l0, c0));
        }
        Ok(FPart::Expr {
            src,
            format,
            line,
            col,
        })
    }

    fn operator(&mut self, line: usize, col: usize, start: usize) -> Result<(), LexError> {
        let c = self.peek(0).expect("há caractere");
        let d = self.peek(1);
        let dois = match (c, d) {
            ('.', Some('*')) => Some(Op::DotStar),
            ('.', Some('/')) => Some(Op::DotSlash),
            ('.', Some('\\')) => Some(Op::DotBackslash),
            ('.', Some('^')) => Some(Op::DotCaret),
            ('.', Some('\'')) => Some(Op::DotQuote),
            ('=', Some('=')) => Some(Op::Eq),
            ('!', Some('=')) => Some(Op::Ne),
            ('~', Some('=')) => Some(Op::TildeEq),
            ('<', Some('=')) => Some(Op::Le),
            ('>', Some('=')) => Some(Op::Ge),
            ('&', Some('&')) => Some(Op::AmpAmp),
            ('|', Some('|')) => Some(Op::PipePipe),
            ('+', Some('=')) => Some(Op::PlusAssign),
            ('-', Some('=')) => Some(Op::MinusAssign),
            ('*', Some('=')) => Some(Op::StarAssign),
            ('/', Some('=')) => Some(Op::SlashAssign),
            _ => None,
        };
        if let Some(op) = dois {
            self.bump();
            self.bump();
            return self.emit(Tok::Op(op), line, col, start);
        }
        let op = match c {
            '+' => Op::Plus,
            '-' => Op::Minus,
            '*' => Op::Star,
            '/' => Op::Slash,
            '\\' => Op::Backslash,
            '^' => Op::Caret,
            '=' => Op::Assign,
            '<' => Op::Lt,
            '>' => Op::Gt,
            ':' => Op::Colon,
            ',' => Op::Comma,
            ';' => Op::Semicolon,
            '.' => Op::Dot,
            '@' => Op::At,
            '&' => Op::Amp,
            '|' => Op::Pipe,
            '!' => Op::Bang,
            '~' => Op::Tilde,
            '(' | '[' | '{' => {
                self.brackets.push((c, line, col));
                match c {
                    '(' => Op::LParen,
                    '[' => Op::LBracket,
                    _ => Op::LBrace,
                }
            }
            ')' | ']' | '}' => {
                let par = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if self.brackets.pop().map(|b| b.0) != Some(par) {
                    return Err(self.err(LexErrorKind::Unbalanced, line, col));
                }
                match c {
                    ')' => Op::RParen,
                    ']' => Op::RBracket,
                    _ => Op::RBrace,
                }
            }
            other => return Err(self.err(LexErrorKind::InvalidChar(other), line, col)),
        };
        self.bump();
        self.emit(Tok::Op(op), line, col, start)
    }
}

/// Analisa o texto-fonte inteiro.
///
/// Erros: ver [`LexErrorKind`], com identificadores `chimalab:lexico:…`.
pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(src).run()
}

/// Se o texto termina no meio de uma construção que continua na próxima linha (para o console
/// pedir mais entrada): delimitador ou comentário `#=` abertos, linha final terminada em `:`
/// (abre bloco), ou bloco aberto sem a linha em branco que o encerra no console. Outros erros
/// devolvem `false`: aparecem ao executar.
pub fn incomplete(src: &str) -> bool {
    match tokenize(src) {
        Err(e) => matches!(e.kind, LexErrorKind::Unclosed | LexErrorKind::UnterminatedComment),
        Ok(toks) => {
            let ultimo_real = toks
                .iter()
                .rev()
                .map(|t| &t.tok)
                .find(|t| !matches!(t, Tok::Newline | Tok::Dedent | Tok::Eof));
            if matches!(ultimo_real, Some(Tok::Op(Op::Colon))) {
                return true;
            }
            // Bloco aberto (houve Indent) e a entrada não termina em linha em branco.
            let abriu = toks.iter().any(|t| t.tok == Tok::Indent);
            abriu && !src.trim_end_matches([' ', '\t', '\r']).ends_with("\n\n")
        }
    }
}

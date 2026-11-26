# 3. Instruções e blocos

Decididas no DT-39 (respostas 40–48). Testes de conformidade em `crates/chimalab-lang/tests/instrucoes.rs`; o exemplo da §3.8 é analisado por um teste.

## 3.1 Linhas, `;` e exibição
- Cada linha tem uma instrução composta ou uma ou mais instruções simples separadas por `;`.
- No console, expressões e atribuições exibem o resultado; um `;` no fim suprime a exibição (DT-34). `for`, `if`, `def` e as demais não exibem nada.
- Não há corpo na mesma linha do cabeçalho: `if x: y = 1` é erro. O corpo vai nas linhas seguintes, indentado (§1.1).

```
>> x = 1            # exibe x
>> y = 2; z = 3;    # nada exibido
```

## 3.2 Atribuições
| Forma | Significado |
|---|---|
| `x = e` | atribui |
| `A[i, j] = e`, `s.campo = e` | atribui a parte de um valor (índices com colchetes) |
| `m, i = max(v)` | vários alvos recebem as saídas de uma chamada; `_` descarta uma saída |
| `x += e`, `-=`, `*=`, `/=` | o mesmo que `x = x + e` etc. (`*=` e `/=` são matriciais) |

Atribuir a outra coisa (`f(1) = 2`, `x + 1 = 2`) é erro `chimalab:sintaxe:alvoInvalido`.

## 3.3 Operadores lógicos
| Grafia | Significado |
|---|---|
| `a and b`, `a && b` | e com curto-circuito (b só é avaliado se preciso) |
| `a or b`, `a \|\| b` | ou com curto-circuito |
| `not a`, `!a` | negação (elemento a elemento em arrays) |
| `a & b`, `a \| b` | e/ou elemento a elemento |

Precedência, da mais fraca para a mais forte: `->` (função anônima), `or`, `and`, `|`, `&`, `not`, comparação, intervalo, `+ -`, `* / \`, sinal, potência. Assim `x > 0 & x < 1` dispensa parênteses, `not a == b` é `not (a == b)` e `!a & b` é `(!a) & b`. Depois de uma comparação ou de um operador aritmético, a negação pede parênteses: `a != (!b)`.

## 3.4 Funções anônimas
`x -> x^2`, `(x, y) -> x + y` e `() -> 1`. O corpo é uma expressão e se estende o máximo possível: `x -> y -> x + y` devolve outra função.

## 3.5 Condicionais e laços
```
if x > 0:
    s = 1
elif x < 0:
    s = -1
else:
    s = 0

for k in 1:n:          # o último `:` fecha o cabeçalho
    total += k

while erro > tol:
    ...
```
- `for` percorre as **colunas** do valor (num vetor linha, os elementos).
- `break` e `continue` só valem dentro de `for` e `while` da mesma função.
- No cabeçalho, um `:` seguido do fim da linha (ou de comentário) fecha o cabeçalho; os outros `:` são intervalos: `for k in 1:2:n:`.

## 3.6 Erros e `match`
```
try:
    r = calcula(x)
except e:            # `except:` sem nome também vale
    r = []

match op:
    case "+":
        r = a + b
    case "-", "menos":  # vários valores
        r = a - b
    case _:             # padrão: precisa ser o último
        r = 0
```

## 3.7 Funções
```
def norma_p(x, p=2):
    return sum(abs(x) .^ p) ^ (1 / p)

def minmax(v):
    return min(v), max(v)    # várias saídas: a, b = minmax(v)
```
- Parâmetros com valor padrão vêm depois dos obrigatórios; nomes não se repetem.
- `return` só vale dentro de `def`.

## 3.8 Exemplo
```
def media_movel(x, k=3):
    n = numel(x)
    y = zeros(1, n)
    for i in 1:n:
        a = max(1, i - k + 1)
        y[i] = mean(x[a:i])
    return y

def classifica(v):
    match sign(v):
        case 1:
            return "positivo"
        case -1:
            return "negativo"
        case _:
            return "zero"

total = 0
k = 1
while k <= 10:
    if mod(k, 2) == 0 and k != 6:
        total += k
    elif k == 9 or k == 7:
        k += 1
        continue
    k += 1
try:
    r = media_movel(1:10)
except erro:
    r = []
quadrado = x -> x .^ 2
ok = !isempty(r) && all(r > 0);
```

## 3.9 Palavras reservadas
`and break case continue def elif else except for if in match not or pass return try while`. Usá-las como nome é erro `chimalab:sintaxe:palavraReservada`. `end` não é reservada: dentro de índices é o último índice; fora deles, um erro próprio.

## 3.10 Erros
| Identificador | Exemplo |
|---|---|
| `chimalab:sintaxe:alvoInvalido` | `f(1) = 2` |
| `chimalab:sintaxe:blocoEsperado` | `if x:` sem corpo indentado |
| `chimalab:sintaxe:indentacaoInesperada` | linha indentada sem `:` antes |
| `chimalab:sintaxe:foraDeLaco` | `break` fora de laço |
| `chimalab:sintaxe:returnForaDeFuncao` | `return` fora de `def` |
| `chimalab:sintaxe:clausulaSemInstrucao` | `else:` sem `if` |
| `chimalab:sintaxe:obrigatorioDepoisDePadrao` | `def f(x=1, y):` |
| `chimalab:sintaxe:parametroRepetido` | `def f(x, x):` |
| `chimalab:sintaxe:casoPadraoNaoUltimo` | `case _:` antes de outro `case` |
| `chimalab:sintaxe:palavraReservada` | `if = 3` |

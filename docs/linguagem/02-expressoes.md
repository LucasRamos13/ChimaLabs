# 2. Expressões

## 2.1 Precedência
Da menor para a maior:

| Nível | Operadores | Associatividade |
|---|---|---|
| 1 | `==` `!=` `<` `<=` `>` `>=` | nenhuma: `a < b < c` é erro |
| 2 | `a:b`, `a:passo:b` (intervalo) | — |
| 3 | `+` `-` | à esquerda |
| 4 | `*` `/` `\` `.*` `./` `.\` | à esquerda |
| 5 | sinal `-x`, `+x` | prefixo |
| 6 | `^` `.^` | à direita |
| 7 | `'` `.'`, `A[…]`, `f(…)`, `.nome` | pós-fixos |

Consequências:
- `-2^2` é −4 e `2^-1` é 0,5;
- `2^3^2` é 2^9 = 512, como na notação matemática;
- `1:n+1` vai de 1 a n + 1;
- `a < b < c` é recusado com a sugestão de escrever as duas comparações (o erro clássico de comparar um resultado lógico com `c` não acontece).

Operadores lógicos (DT-39): `and`/`&&` e `or`/`||` com curto-circuito, `not`/`!`, e `&`, `|` elemento a elemento; entram no parser com as instruções.

## 2.2 Operações de matriz e elemento a elemento
- `*`, `/`, `\` e `^` são de álgebra linear: produto matricial, divisões por sistema linear e potência de matriz quadrada; a potência aceita expoente inteiro (exata para matrizes inteiras), negativo (pela inversa) ou fracionário (ramo principal), e um escalar elevado a uma matriz.
- `.*`, `./`, `.\` e `.^` operam elemento a elemento, com expansão implícita das dimensões unitárias: uma coluna 2×1 com uma linha 1×3 dá 2×3.
- Com um escalar, as duas formas coincidem.

## 2.3 Intervalos
`a:b` e `a:passo:b` são vetores linha com o fim **incluído**: `1:5` é `[1 2 3 4 5]` e `0:0.1:1` tem 11 elementos e termina exatamente em 1. Passo 0, NaN ou sentido contrário dão vetor vazio. A função `range` (DT-34; a chamada entra com as funções da linguagem) só aceitará a forma com argumento nomeado: `range(0, 1; length=5)` dá 5 pontos igualmente espaçados com os dois extremos exatos (no runtime, `linspace`), e `range(0, 1; step=0.25)` é o mesmo que `0:0.25:1`.

## 2.4 Matrizes literais
- `[1 2; 3 4]`: elementos separados por espaço ou `,`, linhas por `;` ou quebra de linha.
- **O espaço separa elementos:** `[1 -2]` tem dois elementos (o sinal colado ao número depois de um espaço começa outro elemento), `[1 - 2]` tem um. `[a (1)]` e `[a [1]]` também têm dois; sem espaço, `a(1)` é chamada e `a[1]` é índice.
- **Forma aninhada:** `[[1, 2], [3, 4]]` é o mesmo que `[1 2; 3 4]`: quando todos os elementos de uma única linha são matrizes literais separadas por vírgula, cada uma vira uma linha. Com espaços, `[[1 2] [3 4]]` concatena na horizontal (`[1 2 3 4]`).
- **Concatenação** (DT-38): as linhas são concatenadas na horizontal e depois empilhadas; os tamanhos precisam ser compatíveis. `[]` é ignorado. A classe do resultado segue a aritmética (inteiro domina, `single` domina `double`); só lógicos ou só caracteres mantêm a classe; caractere com número é erro (converta explicitamente).

## 2.5 Índices e chamadas
- `A[i, j]` indexa e `f(x)` chama. A base é 1.
- Dentro de `A[…]`, `end` é o último índice da dimensão (`A[end]`, `A[end-1, :]`), e `:` sozinho seleciona tudo. Fora de um índice, `end` é erro.
- Argumentos nomeados: `f(x, y=2)`, ou depois de `;`: `range(0, 1; length=5)`. Um argumento posicional depois de um nomeado é erro.
- Membros: `x.campo`; `A.T` e `A.H` são as transpostas simples e conjugada.

## 2.6 Transpostas
`A'` é a transposta conjugada e `A.'` a simples (iguais para reais). `A.H` e `A.T` são sinônimos (reconhecidos pelo parser; a avaliação vem com o interpretador).

## 2.7 Erros de sintaxe
| Identificador | Quando |
|---|---|
| `chimalab:sintaxe:tokenInesperado` | token fora do lugar, com a descrição do que se esperava |
| `chimalab:sintaxe:fimInesperado` | a expressão acabou no meio (`1 +`) |
| `chimalab:sintaxe:comparacaoEncadeada` | `a < b < c` |
| `chimalab:sintaxe:endForaDeIndice` | `end` fora de `A[…]` |
| `chimalab:sintaxe:argumentoPosicionalDepoisDeNomeado` | `f(y=2, x)` |
| `chimalab:sintaxe:diferenteComTil` | `a ~= b` (use `!=`) |

Os erros léxicos (seção 1.8) também aparecem ao analisar uma expressão. Erros dentro de uma f-string apontam a posição no texto original.

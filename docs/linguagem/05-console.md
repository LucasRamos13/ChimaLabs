# 5. Console, exibição e execução

Regras do primeiro avaliador (DT-39, 48.A; DT-40). Testes em `crates/chimalab-runtime/tests/interpretador.rs`.

## 5.1 O que o console mostra
| Entrada | Exibe | `ans` |
|---|---|---|
| `x = 1 + 2` | `x = 3` | não muda |
| `x = 1 + 2;` | nada | não muda |
| `x` | `x = 3` | não muda |
| `2 * x` | `ans = 6` | recebe 6 |
| `m, i = max(v)` | `m = …` e `i = …` | não muda |
| `A[2] = 7` | a variável `A` inteira | não muda |
| `f()` sem `return` | nada | não muda |

Instruções compostas (`if`, `for`, `def`…) não exibem nada por si; o que está dentro delas exibe conforme as regras acima. Um erro interrompe a execução, mas o que já foi feito permanece no workspace.

## 5.2 Números
- Até **10 algarismos significativos** (7 em `single`), sem zeros à direita: `1/3` → `0.3333333333`, `0.1 + 0.2` → `0.3`.
- Inteiros exatos abaixo de 10¹⁵ aparecem sem casa decimal: `2^40` → `1099511627776`.
- Notação científica quando o expoente sai de [−5, 10): `1.5e-7`, `1e20`.
- Um número não inteiro nunca parece inteiro. Se o arredondamento comeria as casas decimais, aparecem todos os algarismos (`1 - eps/2` → `1.000000000`, como num resultado de `A \ b` que deveria dar 1); se nem assim couber, notação científica (`12345678901.5` → `1.23456789e10`).
- Empates exatos arredondam para o par. `-0` mostra o sinal; `Inf`, `-Inf` e `NaN` por extenso.
- Complexos: `3 - 4i`.

## 5.3 Arrays
```
>> A = [1 -2; 30 4]
A = 2×2
   1  -2
  30   4
>> b = [1 2 3] > 1
b = 1×3 logical
  false   true   true
>> e = zeros(0, 3)
e = [] (0×3)
>> T = ones(2, 2, 2)
T = 2×2×2
  [:, :, 1]
    1  1
    1  1
  [:, :, 2]
    1  1
    1  1
```
- O cabeçalho traz o tamanho e, se não for `double`, a classe. Colunas alinhadas à direita.
- Acima de 1000 elementos, a exibição é resumida: as 3 primeiras e as 3 últimas linhas e colunas, com `…`, `⋮` e `⋱` no lugar do resto (e as 3 primeiras e 3 últimas fatias N-D). O cabeçalho mantém o tamanho completo, e só os elementos mostrados são formatados.
- Textos entre aspas duplas: `s = "olá"`. Funções: `f = função anônima (x)`, `h = função g(a, b=…)`.

## 5.4 f-strings
Formato `[<>^][largura][.precisão][f|e|g|d|s]`:

| Exemplo | Resultado |
|---|---|
| `f"{1/3:.3f}"` | `0.333` |
| `f"{12345.678:.2e}"` | `1.23e+04` |
| `f"{1/3:.4}"` ou `:.4g` | `0.3333` |
| `f"{42:>5}"` | `   42` |
| `f"{'ab':^6}"` | `  ab  ` |
| `f"{[1 2; 3 4]}"` | `[1 2; 3 4]` |

`d` exige inteiro; textos só aceitam largura e alinhamento; um formato desconhecido é `chimalab:execucao:formatoInvalido`.

## 5.5 Erros de execução
| Identificador | Exemplo |
|---|---|
| `chimalab:execucao:nomeIndefinido` | `y = z` sem `z` |
| `chimalab:execucao:chamarArray` | `x(1)` com `x` array (use `x[1]`) |
| `chimalab:execucao:indexarFuncao` | `f[1]` com `f` função |
| `chimalab:execucao:funcaoComoValor` | `sin + 1` |
| `chimalab:execucao:condicaoNaoEscalar` | `if [1 1]:` |
| `chimalab:execucao:argumentos` / `argumentoFaltando` | número ou nome de argumentos errado |
| `chimalab:execucao:semValor` | `x = f()` com `f` sem `return` |
| `chimalab:execucao:saidasDemais` | `a, b = f()` com `f` de uma saída |
| `chimalab:execucao:variasSaidas` | `a, b = 1` |
| `chimalab:execucao:recursaoProfunda` | mais de 1000 chamadas aninhadas (configurável) ou pilha esgotada |
| `chimalab:execucao:intervaloNaoEscalar` | `1:[1 2]` |
| `chimalab:execucao:classeNaoSuportada` | `exp(int8(1))` |
| `chimalab:execucao:naoSuportado` | estruturas e atribuições aninhadas (incremento futuro) |
| `chimalab:execucao:formatoInvalido` | `f"{1.5:d}"` |

Um nome indefinido sugere o nome conhecido mais próximo (variável, `def` ou embutida; distância de edição em que trocar duas letras vizinhas conta 1; até 1 para nomes de até 4 letras e 2 para os maiores; empate pela ordem alfabética): `` `summ` não está definido; você quis dizer `sum`? ``. No console e nos scripts, o erro vem com a linha do código e um `^` na coluna:

```
erro: linha 2, coluna 5: `nada` não está definido
  y = nada
      ^
```

Os erros do runtime (`chimalab:runtime:…`) e da sintaxe (`chimalab:sintaxe:…`, `chimalab:lexico:…`) chegam pelo mesmo caminho, com linha e coluna. `error(id, mensagem)` levanta um erro com o identificador do usuário.

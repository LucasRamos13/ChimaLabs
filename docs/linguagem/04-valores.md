# 4. Valores e tipos

Todo valor é um array N-D com pelo menos duas dimensões (um escalar é 1×1), guardado em ordem de coluna. As regras abaixo são as do runtime (DT-32 e DT-38), com testes em `crates/chimalab-runtime/tests/`.

## 4.1 Classes
| Classe | Conteúdo |
|---|---|
| `double` | ponto flutuante de 64 bits, real ou complexo (padrão dos literais) |
| `single` | ponto flutuante de 32 bits, real |
| `int8` … `int64`, `uint8` … `uint64` | inteiros com e sem sinal |
| `logical` | verdadeiro ou falso |
| `char` | caracteres |

## 4.2 Conversões para inteiro
- Arredondam o meio para longe de zero: `uint8(3.5)` é 4 e `int8(-2.5)` é −3.
- Saturam nos limites: `int8(300)` é 127 e `uint8(-5)` é 0.
- NaN vira 0. Converter NaN para lógico é erro.

## 4.3 Aritmética mista
- Inteiro com `double` dá a classe inteira: a conta é feita em ponto flutuante e depois arredondada e saturada (`int8(127) + 1` é 127).
- `int64` e `uint64` com outro inteiro (ou `double` de valor inteiro) somam, subtraem e multiplicam em aritmética exata, sem perder bits acima de 2⁵³.
- Divisão inteira arredonda o meio para longe de zero: `int32(7)/int32(2)` é 4 e `int32(-7)/int32(2)` é −4. Divisão por zero satura (0/0 é 0).
- Inteiros de classes diferentes, ou inteiro com complexo, são erro.
- `single` com `double` dá `single`. `logical` e `char` contam como `double`.
- A negação segue o IEEE em ponto flutuante: `-0.0` tem sinal (1/(−0) é −∞).

## 4.4 Complexos automáticos
`sqrt(-4)` é `2i`, e `(-8)^(1/3)` é a raiz principal `1 + 1,732i`. Um resultado complexo cuja parte imaginária some volta a ser real. A potência de base negativa só é complexa quando o expoente é fracionário: `(-8)^2` é 64.

## 4.5 Expansão implícita
Nas operações elemento a elemento, dimensões iguais ou unitárias se combinam: uma coluna 2×1 com uma linha 1×3 dá 2×3. Dimensões diferentes e não unitárias são erro (`chimalab:runtime:dimensoesIncompativeis`).

## 4.6 Concatenação
- `[A B]` concatena na horizontal e `[A; B]` na vertical; os tamanhos nas outras dimensões precisam ser iguais. `[]` é ignorado.
- A classe segue a aritmética mista (inteiro domina, `single` domina `double`). Só lógicos ou só caracteres mantêm a classe.
- **Caractere com número é erro**: `['a' 66]` não vira `"aB"` em silêncio; converta explicitamente.

## 4.7 Operações matriciais
- `A*B` é o produto matricial (também complexo; `single` acumula em `double` e arredonda uma vez). Inteiros não têm produto matricial.
- `A\b` resolve A·x = b escolhendo o método pela estrutura da matriz; `A/B` resolve X·B = A. Matriz singular ou mal condicionada gera o aviso `chimalab:runtime:matrizSingular`, com a estimativa do recíproco do número de condição.
- `A^p`: expoente inteiro por quadrados sucessivos (exato para matrizes inteiras enquanto os valores couberem em 2⁵³), negativo pela inversa, fracionário pela função de matriz (ramo principal); `s^B` com escalar s é exp(B·ln s).

## 4.8 Indexação
- Base 1. Formas: subscrito `A[i, j]`, linear `A[k]`, lógica (máscara), intervalo e `:` (tudo).
- Atribuir fora dos limites cresce o array, preenchendo com o zero da classe; a atribuição linear só cresce vetores. Atribuir vazio remove elementos.
- Índice zero é erro, com mensagem que lembra a base 1.

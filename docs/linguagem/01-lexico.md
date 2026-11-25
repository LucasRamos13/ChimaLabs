# 1. Estrutura léxica

O texto-fonte é UTF-8. O analisador léxico o divide em tokens: nomes, números, textos, operadores, fins de linha e mudanças de indentação. Cada token sabe a linha e a coluna (em caracteres) onde começa, e os erros apontam essa posição.

## 1.1 Linhas e blocos
- Uma linha lógica termina na quebra de linha. Linhas em branco e linhas só com comentário são ignoradas.
- **Blocos por indentação.** Uma linha mais recuada que a anterior abre um bloco; voltar a um recuo anterior fecha os blocos até ele. Voltar para uma coluna que nunca abriu bloco é erro (`chimalab:lexico:indentacaoInconsistente`):

  ```
  if x > 0:
      y = 1
    z = 2        # erro: a coluna 3 não abriu bloco
  ```

- A tabulação avança até o próximo múltiplo de 4 colunas. Misturar tabulações e espaços funciona, mas não é recomendado.
- **Continuação de linha:** dentro de `( )` e `{ }` a quebra de linha é ignorada. Dentro de `[ ]` ela separa linhas da matriz, como `;`:

  ```
  A = [1 2
       3 4]        # o mesmo que [1 2; 3 4]
  ```

- No console, a entrada continua enquanto houver delimitador aberto, comentário `#=` aberto ou bloco aberto; uma linha em branco encerra o bloco.

## 1.2 Comentários
- `#` comenta até o fim da linha.
- `#= … =#` comenta um trecho, que pode atravessar linhas e conter outros `#= … =#`.
- Um comentário conta como espaço (dentro de colchetes, separa elementos).

## 1.3 Nomes
Começam com letra (inclusive letras não latinas, como `α`) ou `_`, seguidos de letras, dígitos ou `_`. Maiúsculas e minúsculas são diferentes. As palavras-chave são reconhecidas pelo parser, não pelo léxico.

## 1.4 Números
| Forma | Exemplos |
|---|---|
| Decimal, com `_` entre dígitos | `42`, `1_000_000`, `3.25`, `.5`, `1.` |
| Expoente | `1e3`, `2.5E-3`, `1e+2` |
| Hexadecimal, octal, binário | `0xFF`, `0o17`, `0b101` |
| Imaginário (sufixo colado) | `2i`, `3.5j`, `1e2i` |

- `1.*2` é `1 .* 2`: o ponto pertence ao operador elemento a elemento.
- Um número colado a letras é erro: `2x` dá `chimalab:lexico:numeroInvalido` (escreva `2*x`).
- Inteiros acima de 2⁵³ guardam a grafia original, para as classes inteiras de 64 bits.

## 1.5 Textos
- Aspas simples e duplas são equivalentes: `'abc'` e `"abc"`. Um texto não atravessa linhas.
- Escapes: `\n`, `\t`, `\r`, `\0`, `\\`, `\'`, `\"` e `\u{…}` (código Unicode em hexadecimal, até 6 dígitos). Outro escape é erro.
- **Textos crus** com o prefixo `r` não processam escapes: `r"C:\temp\nova"`.
- **`'` depois de um valor é transposta.** Depois de um nome, número, texto, `)`, `]`, `}` ou outra transposta, `'` transpõe; nos demais lugares, abre um texto. Dentro de colchetes, um espaço antes de `'` abre texto:

  ```
  >> B = A'            # transposta
  >> s = 'texto'       # texto
  >> [a' b']           # duas transpostas
  >> [a 'b']           # a e o texto "b"
  ```

## 1.6 f-strings
`f"…"` intercala expressões entre chaves:

```
>> f"x = {x:.3f}, n = {n}"
>> f"{{literal}}"          # chaves duplas são literais
```

- O primeiro `:` fora de parênteses e colchetes começa a especificação de formato; um intervalo dentro da expressão precisa de parênteses: `f"{(1:3)}"`.
- Dentro da expressão valem textos e transpostas: `f"{d['chave']}"`, `f"{A'}"`.
- `rf"…"` combina f-string e texto cru.

## 1.7 Operadores e pontuação
`+ - * / \ ^`, `.* ./ .\ .^`, `' .'`, `= += -= *= /=`, `== != < <= > >=`, `: , ; .`, `( ) [ ] { }`, `@ & | ! ~ && || ~=`.

`&&`, `||` e `!` equivalem a `and`, `or` e `not` (DT-39). `~=` é recusado pelo parser com a sugestão `!=`. `~` e `@` ficam reservados.

## 1.8 Erros léxicos
| Identificador | Quando |
|---|---|
| `chimalab:lexico:textoNaoTerminado` | texto sem aspa de fechamento na mesma linha |
| `chimalab:lexico:comentarioNaoTerminado` | `#=` sem `=#` |
| `chimalab:lexico:caractereInvalido` | caractere fora da linguagem (ex.: `$`) |
| `chimalab:lexico:indentacaoInconsistente` | recuo para coluna que não abriu bloco |
| `chimalab:lexico:numeroInvalido` | `2x`, `0x` sem dígitos |
| `chimalab:lexico:escapeInvalido` | `\q`, `\u{110000}` |
| `chimalab:lexico:fstringInvalida` | chave sem par, expressão vazia |
| `chimalab:lexico:delimitadorDesbalanceado` | fechamento sem a abertura correspondente |
| `chimalab:lexico:delimitadorNaoFechado` | abertura sem fechamento até o fim do texto |

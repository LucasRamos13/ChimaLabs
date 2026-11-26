# Especificação da linguagem ChimaLab

Referência normativa da linguagem, escrita do zero para o projeto. Cada regra aqui tem teste de conformidade em `crates/chimalab-lang/tests/` ou `crates/chimalab-runtime/tests/`. Quando a especificação e um teste discordarem, o defeito é registrado e corrigido no mesmo incremento.

| Parte | Conteúdo | Estado |
|---|---|---|
| [Guia rápido](00-guia-rapido.md) | Uma volta pela linguagem no console, com respostas conferidas por teste | ✅ |
| [1. Estrutura léxica](01-lexico.md) | Caracteres, linhas e indentação, comentários, nomes, números, textos, f-strings, operadores | ✅ implementada (DT-36) |
| [2. Expressões](02-expressoes.md) | Precedência, intervalos, matrizes, índices, chamadas, transpostas | ✅ análise (DT-37) e avaliação (DT-40) |
| [3. Instruções e blocos](03-instrucoes.md) | Atribuição, lógicos, funções anônimas, `if`, laços, `try`, `match`, `def` | ✅ análise (DT-39) e avaliação (DT-40) |
| [4. Valores e tipos](04-valores.md) | Classes, conversões, aritmética mista, concatenação, operações matriciais, indexação | ✅ no runtime (DT-32, DT-38); funções de construção na linguagem com o interpretador |
| [5. Console e execução](05-console.md) | Exibição, `ans`, `;`, f-strings, erros de execução | ✅ avaliador da árvore (DT-40); documentos executáveis depois |
| [6. Funções embutidas](06-funcoes.md) | Construção, reduções, elementares, classes, álgebra linear | ✅ primeiro conjunto (DT-40) |

Convenções: a indexação é na base 1 (DT-22); os exemplos mostram a entrada depois de `>>`.

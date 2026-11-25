# Especificação da linguagem ChimaLab

Referência normativa da linguagem, escrita do zero para o projeto. Cada regra aqui tem teste de conformidade em `crates/chimalab-lang/tests/` ou `crates/chimalab-runtime/tests/`. Quando a especificação e um teste discordarem, o defeito é registrado e corrigido no mesmo incremento.

| Parte | Conteúdo | Estado |
|---|---|---|
| [1. Estrutura léxica](01-lexico.md) | Caracteres, linhas e indentação, comentários, nomes, números, textos, f-strings, operadores | ✅ implementada (DT-36) |
| [2. Expressões](02-expressoes.md) | Precedência, intervalos, matrizes, índices, chamadas, transpostas | ✅ análise sintática (DT-37); avaliação depende do interpretador |
| 3. Instruções e blocos | Atribuição, `if`, laços, funções, exceções | ⏳ decidido no DT-39; implementação no próximo incremento |
| [4. Valores e tipos](04-valores.md) | Classes, conversões, aritmética mista, concatenação, operações matriciais, indexação | ✅ no runtime (DT-32, DT-38); funções de construção na linguagem com o interpretador |
| 5. Console e documentos | Exibição, `ans`, `;`, entrada incompleta | parcialmente no DT-34 |

Convenções: a indexação é na base 1 (DT-22); os exemplos mostram a entrada depois de `>>`.

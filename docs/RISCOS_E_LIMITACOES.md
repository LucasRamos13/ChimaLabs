# Riscos e Limitações

## Riscos principais (probabilidade / impacto)

| ID | Risco | P | I | Mitigação |
|---|---|---|---|---|
| R02 | **Compatibilidade de linguagem** pode ser contestada (Q2) | Média | Muito alto | Parser isolado (DT-09); opção B (linguagem própria) |
| R03 | **Escopo:** paridade com núcleo + Fluxo já é um esforço de vários anos-pessoa; extensões futuras multiplicam esse esforço | Alta | Muito alto | Fases e portões; priorização por dependência; não reduzir escopo em silêncio (CLAUDE.md) |
| R05 | **Contaminação de sala limpa** por conhecimento prévio (do assistente de IA ou de desenvolvedores que leram código GPL como o Octave) | Média | Alto | Processo formal (D7), registro de proveniência em PR, revisão jurídica (Q5) |
| R06 | Patentes em solvers, renderização, codecs (H.264/AAC) e geração de código | Baixa/Média | Alto | Busca de patentes (Q4); algoritmos publicados antigos; codecs do SO |
| R07 | Lacunas no ecossistema permissivo (LAPACK completo, XSLT, corretor ortográfico, serial) | Média | Médio | Implementação própria planejada; aprovações pontuais |
| R08 | Desempenho do interpretador abaixo do esperado | Média | Alto | Bytecode especializado, JIT (DT-03), benchmarks contínuos |
| R09 | Diferenças entre webviews por plataforma | Média | Médio | Testes E2E nas 3 plataformas; renderização nativa de figuras |
| R11 | Serviços externos equivalentes (catálogo de pacotes, mapas, atualização) exigem infraestrutura e custos | Alta | Médio | Funcionar offline primeiro; serviços como componentes opcionais |
| R12 | Fidelidade numérica: usuários esperam resultados idênticos ao original | Alta | Médio | Documentar tolerâncias; não prometer igualdade bit a bit |

## Limitações
1. O catálogo descreve necessidades funcionais redigidas de forma independente; o comportamento de cada item é fixado pelos critérios de aceitação e pelos testes, não por observação de outro produto.
2. Nenhuma avaliação jurídica formal foi feita. As questões em aberto estão em `LICENCAS_E_RESTRICOES.txt` §15.

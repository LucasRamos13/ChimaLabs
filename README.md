# ChimaLab

Ambiente aberto e independente de computação numérica (linguagem de arrays, bibliotecas científicas, gráficos, IDE, documentos executáveis, apps), com o **ChimaLab Fluxo** para modelagem e simulação por diagramas de blocos.

> **Estado:** em desenvolvimento. As bibliotecas numéricas (`crates/chimalab-math`) e o modelo de valores do runtime (`crates/chimalab-runtime`) já têm implementação e testes; a linguagem, a IDE e o Fluxo ainda não. O estado de cada funcionalidade está em [docs/MATRIZ_PARIDADE_FUNCIONAL.md](docs/MATRIZ_PARIDADE_FUNCIONAL.md) e em `docs/progresso/`.

## Objetivos
- Atender às necessidades de engenheiros, cientistas, estudantes e docentes que hoje dependem de ambientes comerciais de computação numérica e simulação.
- Ser uma *melhoria*, não um clone: linguagem com ergonomia inspirada no Python, rigor numérico suficiente para artigos científicos e uma interface própria.
- Implementação **em sala limpa**: nenhum código, binário, texto, documentação ou recurso visual de produto proprietário; nenhum código de projetos copyleft. Cada algoritmo vem de publicações citadas no código e em `docs/progresso/`.

## O que já existe (`chimalab-math`)
Álgebra linear densa e esparsa (fatorações, autovalores, Schur, funções de matriz, métodos iterativos), funções especiais reais e complexas, estatística descritiva e robusta, FFT e filtragem, interpolação 1-D e de dados dispersos, polinômios, geometria computacional (predicados exatos, Delaunay, Voronoi, fecho convexo 2-D e 3-D, operações booleanas), integração numérica adaptativa, EDOs (explícitos, implícitos, Rosenbrock), equações com retardo, problemas de contorno e EDPs parabólicas 1-D, otimização e raízes, grafos, geradores aleatórios e aritmética dupla-dupla.

## Testes
O padrão de testes está em [docs/PLANO_DE_TESTES.md](docs/PLANO_DE_TESTES.md): critérios de aceitação, oráculos independentes de alta precisão (mpmath, soluções analíticas, identidades), testes de propriedade (proptest), casos de borda, cobertura ≥ 90% e teste de mutação (cargo-mutants).

```bash
ferramentas/verificar.sh          # formatação, clippy, testes, licenças e documentos gerados
cargo test --workspace            # só os testes
```

Requer o toolchain Rust fixado em `rust-toolchain.toml`.

## Documentos

| Documento | Conteúdo |
|---|---|
| [CLAUDE.md](CLAUDE.md) | Regras permanentes do projeto |
| [LICENCAS_E_RESTRICOES.txt](LICENCAS_E_RESTRICOES.txt) | Política de licenças e dependências aprovadas |
| [docs/CATALOGO_FUNCIONALIDADES.md](docs/CATALOGO_FUNCIONALIDADES.md) | 162 funcionalidades com critérios de aceitação |
| [docs/REQUISITOS.md](docs/REQUISITOS.md) | Requisitos não funcionais e funcionais |
| [docs/MATRIZ_PARIDADE_FUNCIONAL.md](docs/MATRIZ_PARIDADE_FUNCIONAL.md) | Funcionalidade → requisito → módulo → teste → status |
| [docs/linguagem/](docs/linguagem/README.md) | Especificação da linguagem (estrutura léxica e expressões) |
| [docs/ARQUITETURA.md](docs/ARQUITETURA.md) | Arquitetura |
| [docs/DECISOES_TECNICAS.md](docs/DECISOES_TECNICAS.md) | Decisões técnicas (DT-xx) |
| [docs/PLANO_DE_DESENVOLVIMENTO.md](docs/PLANO_DE_DESENVOLVIMENTO.md) | Fases e portões |
| [docs/PLANO_DE_TESTES.md](docs/PLANO_DE_TESTES.md) / [docs/TESTES_POR_FUNCIONALIDADE.md](docs/TESTES_POR_FUNCIONALIDADE.md) | Estratégia e casos de teste |
| [docs/RISCOS_E_LIMITACOES.md](docs/RISCOS_E_LIMITACOES.md) | Riscos |
| [docs/PENDENCIAS_DE_INVESTIGACAO.md](docs/PENDENCIAS_DE_INVESTIGACAO.md) | Pendências e decisões em aberto |
| [docs/IDENTIDADE.md](docs/IDENTIDADE.md) | Nome e identidade visual |

## Estrutura
```
Cargo.toml    workspace Rust (crates/chimalab-*, crates/fluxo-*)
crates/       chimalab-math, chimalab-array e chimalab-runtime implementados; os demais são esqueletos
docs/         planejamento, decisões e progresso por funcionalidade
ferramentas/  verificação, gerador de documentos e geradores de oráculos (tests/dados/)
tests/dados/  valores de referência de alta precisão
assets/       logotipo e ícones próprios (proveniência em assets/PROVENIENCIA.md)
deny.toml     política de licenças aplicada pelo cargo-deny
```

Os documentos de catálogo, matriz, requisitos e testes são gerados a partir de `docs/dados/funcionalidades.json`:
```bash
python3 -I ferramentas/gerar_docs.py
```

## Desenvolvimento assistido por IA
Grande parte do código e da documentação foi escrita com o assistente Claude (Anthropic), sob as regras de sala limpa de `CLAUDE.md`; cada commit declara a proveniência (artigos e especificações consultados).

## Licença
Licenciado sob sua escolha de [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE) (SPDX: `MIT OR Apache-2.0`).

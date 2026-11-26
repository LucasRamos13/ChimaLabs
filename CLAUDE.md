# CLAUDE.md — Regras permanentes do projeto ChimaLab

Este arquivo orienta **todas** as sessões futuras (humanas ou de IA). As regras aqui têm prioridade sobre preferências de conveniência.

## 0. Estado atual
- **Planejamento APROVADO em 2026-11-17** (DT-15). G0 fechado; licença do projeto: MIT OR Apache-2.0; nome **ChimaLab** definido pelo responsável (DT-17); projeto de portfólio, de código aberto, sem registro de marca (DT-25).
- **Repositório público** (DT-35): `github.com/LucasRamos13/ChimaLabs`. Tudo o que é commitado fica público. O material da investigação em sala limpa (inventário, listas de componentes, tabelas de terceiros, análise e riscos contratuais) fica **fora** do repositório, em `../ChimaLab-privado/`, e nunca deve ser copiado para cá.
- **Em andamento:** F001 (console: léxico, parser e avaliador, em `crates/chimalab-lang`, `chimalab-runtime` e `chimalab-cli`), F002 (scripts), F006 (controle de fluxo), F016 (funções anônimas e de ordem superior), F013 (workspace), F020 (conjuntos, ordenação e busca), F003–F005 (tipos, indexação e operadores no runtime), F040 (álgebra linear densa), F041 (matrizes esparsas), F042 (funções especiais), F043 (estatística descritiva), F046 (FFT e filtragem), F044 (interpolação), F047 (geometria computacional), F048 (integração numérica), F049 (EDOs), F050 (retardos, contorno e EDP 1-D), F051 (otimização básica e raízes), F052 (grafos), F053 (números aleatórios) e F054 (precisão e aritmética especial); **implementada:** F045 (polinômios). Numéricos em `crates/chimalab-math`, modelo de valores em `crates/chimalab-runtime`; progresso em `docs/progresso/Fxxx.md`. Cada funcionalidade em desenvolvimento tem um arquivo `docs/progresso/Fxxx.md`.
- Portões G1 e G2: liberados em 2026-11-18 por decisão do responsável com base em pesquisa, sujeita a parecer jurídico (DT-19 e DT-21).
  - G1 (DT-09): linguagem própria, mais um modo de compatibilidade opcional e isolado.
  - G2 (DT-10): só `.mat` v5/v7.3, implementado a partir de fontes independentes.
  - Superfície da linguagem (DT-22): indexação base 1, blocos por indentação (estilo Python), modo de compatibilidade só depois do núcleo.
  - Sintaxe (DT-34): `*` matricial e `.*` elemento a elemento; `[1 2; 3 4]` e `[[1, 2], [3, 4]]`; `a:b` com o fim incluído; `def`/`return a, b`; `#` e `#= =#`; aspas simples ou duplas e f-strings; `A[i]` indexa e `f(x)` chama; console mostra tudo (`;` suprime) e documentos só a última expressão; `A'`, `A.'`, `A.H`, `A.T`; `end` como último índice.
  - Construções (DT-39): blocos como no Python (`if`/`elif`/`else`, `for k in …:`, `while`, `try`/`except`, `match`/`case`); `for` percorre colunas; lógicos `and`/`&&`, `or`/`||`, `not`/`!`; imaginário `2i`/`2j`; diferente `!=` (`~=` é erro); só continuação implícita; linha em branco fecha bloco no console; anônimas `x -> x^2`; primeiro interpretador percorre a AST.
  - Avaliador (DT-40, aprovado: 49.A, 50.A, 51.A, 53.A): condições exigem escalar (vazio é falso); anônimas capturam valores na criação; `def` com escopo isolado; exibição com 10 algarismos e sem inteiros falsos. A fazer: Ctrl+C com `ctrlc` (52.A), `+` entre textos concatena (54.A), pesquisar estruturas/células/dicionários (55.P); texto indexado por escalar Unicode (56.A, DT-06 revisto).
  - Reserva de nomes, registro de marca, logotipo e busca de marca: dispensados (DT-25). O projeto é de portfólio e de código aberto.
- Toolchain: Rust 1.100.0 via rustup em `~/.cargo/bin` (fora do PATH do shell — usar `export PATH=$HOME/.cargo/bin:$PATH`). Verificações locais: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check`, `python3 -I ferramentas/gerar_docs.py --verificar`.

## 1. Identidade
- **Nome:** ChimaLab (ambiente de computação numérica) e ChimaLab Fluxo (modelagem e simulação por blocos). Logo: `assets/logo.png` (ver `docs/IDENTIDADE.md`).
- **Objetivo:** alternativa independente, aberta e multiplataforma que atenda **todas** as necessidades funcionais catalogadas de um ambiente comercial de computação numérica e simulação por blocos.
- **Visão (DT-20):** ser uma *melhoria* do original, não um clone.
  - Linguagem com ergonomia de Python: funções amigáveis que adiantam tarefas complexas, sem perder o caminho avançado.
  - Rigor para artigos científicos e livros.
  - UI profissional com visual substancialmente melhor, e recursos que o original não tem (ex.: editor em grade ao criar uma matriz).
  - Ao fim de cada etapa, revisar se ela pode ser expandida, melhorada ou otimizada sem perdas.
  - A cada fase, pesquisar algoritmos e trabalhos recentes, com ceticismo e respeitando as licenças.
- **Respostas do responsável (DT-19):**
  - não executar o original;
  - não citar o nome do original na documentação de usuário;
  - push autorizado após incrementos verificados, mas **agrupado** (pergunta 37.B, 2026-11-20): commitar a cada incremento e fazer push no fim de um bloco de trabalho (no máximo a cada ~3 h, e sempre no fim da sessão), não a cada commit. Cada push consome minutos do CI;
  - repositório público desde 2026-11-20 (DT-35).
  - Pesquisas que gerarem novas perguntas voltam no formato numerado com opção P.
- **Público-alvo:** engenheiros, cientistas, estudantes, docentes e equipes de P&D.
- **Capacidades pretendidas:** linguagem de arrays interativa, bibliotecas numéricas, tipos de dados de alto nível, gráficos 2-D/3-D, IDE completa, documentos executáveis, construção de apps, E/S ampla, interoperabilidade, testes e build, pacotes e simulação por diagramas de blocos. Lista completa em `docs/CATALOGO_FUNCIONALIDADES.md`.

## 2. Paridade funcional
- Buscar implementar **todas** as funcionalidades aprovadas do catálogo (162 F-xxx; extensões futuras catalogadas fora do repositório).
- **Proibido** remover, ignorar ou simplificar funcionalidades sem justificativa documentada em `DECISOES_TECNICAS.md` **e** autorização do responsável.
- Nenhuma funcionalidade pode sumir da matriz. Para mudar status, edite `docs/dados/funcionalidades.json` (`status_forcado`) e rode `python3 -I ferramentas/gerar_docs.py`.
- Nunca declarar paridade de 100% sem todas as funcionalidades em **VALIDADA**. "Catalogado" não é "existente no original".

## 3. Independência de implementação (sala limpa)
- Todo código é escrito de forma independente, a partir de `docs/REQUISITOS.md`, especificações públicas e algoritmos publicados.
- **Proibido:**
  - usar código, binários, bibliotecas, textos, documentação, ícones, imagens ou recursos do produto original ou da mídia de instalação investigada (mantida fora do repositório);
  - usar código descompilado;
  - consultar ou copiar código de projetos copyleft equivalentes (ex.: GNU Octave) para implementar funcionalidades;
  - mascarar material copiado (renomear, traduzir, reformatar, ofuscar).
- **Não** abrir, descompilar ou executar arquivos da mídia original. A investigação foi feita em sala limpa e não haverá análise adicional (DT-19, item 3).
- Padrões, protocolos e formatos públicos e algoritmos conhecidos podem ser usados, respeitando suas licenças.
- Cada PR deve declarar a proveniência: fontes consultadas (especificações, artigos) e confirmação de que nada do original foi usado.

## 4. Arquitetura
- Seguir `docs/ARQUITETURA.md`: kernel Rust separado da UI Tauri/TypeScript; módulos `chimalab-*` e `fluxo-*`.
- Mudanças arquiteturais relevantes (novo módulo, troca de framework, mudança de protocolo ou formato) exigem um registro DT-xx em `docs/DECISOES_TECNICAS.md` **antes** do código.

## 5. Identidade visual
- Usar exclusivamente os recursos próprios descritos em `docs/IDENTIDADE.md`. Todo recurso em `assets/` precisa de registro em `assets/PROVENIENCIA.md`.
- Não reproduzir a expressão visual do original (faixa de opções, ícones, aparência de blocos, paletas).

## 6. Dependências
Antes de adicionar qualquer biblioteca, crate, pacote npm ou ferramenta de build:
1. Identificar nome e versão exata.
2. Verificar a licença no repositório upstream (arquivo LICENSE + manifesto).
3. Verificar as dependências transitivas (`cargo deny check licenses`, `license-checker`).
4. Avaliar riscos (manutenção, segurança, patentes).
5. Atualizar `LICENCAS_E_RESTRICOES.txt`, Parte B (estado → APROVADA).
6. Documentar a decisão em `docs/DECISOES_TECNICAS.md` se for estrutural.
- Permitidas: MIT, ISC, BSD-2-Clause, BSD-3-Clause, Apache-2.0, Zlib, 0BSD, sempre cumprindo as obrigações. Todo o resto exige aprovação explícita registrada.
- Nunca reutilizar automaticamente uma dependência só porque o original a usa.

## 7. Testes — padrão RIGOROSO (exigência do responsável, 2026-11-17)
**Toda parte concluída precisa ser rigorosamente testada.** Uma parte só é considerada concluída (e um item da matriz só passa de EM DESENVOLVIMENTO) quando cumprir **todas** as camadas abaixo. O detalhamento e as metas numéricas estão em `docs/PLANO_DE_TESTES.md` §6.

1. **Critérios de aceitação:** cada CA da funcionalidade tem pelo menos um teste nomeado `caN_…`.
2. **Oráculo independente:** valores de referência de alta precisão (ex.: mpmath, via `ferramentas/oraculos/`), soluções analíticas ou identidades matemáticas. **Nunca** a saída do produto original. Tolerâncias documentadas e justificadas.
3. **Testes de propriedade** (proptest) com entradas aleatórias para os invariantes: reconstruções, idas e voltas, identidades e simetrias.
4. **Casos de borda obrigatórios:** vazio, escalar, 0, ±0, NaN, ±Inf, negativos, valores enormes e minúsculos (inclusive subnormais), formas degeneradas e incompatíveis.
5. **Caminhos de erro:** cada erro tem teste que confere o identificador estável.
6. **Cobertura** (cargo-llvm-cov): ≥ 90% de linhas em cada módulo concluído; exceções justificadas no `docs/progresso/Fxxx.md`.
7. **Teste de mutação** (cargo-mutants): nenhum mutante "sobrevivente" sem justificativa registrada; os testes precisam detectar defeitos, não só executar código.
8. **Regressão:** todo bug encontrado ganha um teste que falharia antes da correção.
9. **Desempenho:** CAs de desempenho têm medição registrada (benchmark ou teste `#[ignore]` com tempo anotado).
10. **CI** nas três plataformas antes de marcar TESTADA.

Os resultados de cobertura e de mutação de cada funcionalidade ficam registrados em `docs/progresso/Fxxx.md` (seção "Rigor dos testes").
- Oráculos independentes: nunca usar saída do produto original.
- Código escrito não significa funcionalidade concluída. Os estados seguem `docs/PLANO_DE_TESTES.md` §5.

## 8. Documentação
Manter atualizados, no mesmo PR da mudança: requisitos, arquitetura, catálogo, matriz de paridade (via gerador), licenças, decisões técnicas, plano de testes e pendências. A documentação de usuário deve ser escrita do zero, sem traduzir nem parafrasear a do original.

## 9. Rotina obrigatória das sessões
Antes de trabalhar:
1. Ler este `CLAUDE.md`.
2. Ler o `README.md`.
3. Consultar `docs/CATALOGO_FUNCIONALIDADES.md` (ou `docs/dados/funcionalidades.json`).
4. Consultar `docs/MATRIZ_PARIDADE_FUNCIONAL.md`.
5. Consultar `docs/PLANO_DE_DESENVOLVIMENTO.md` (fase e portões).
6. Verificar as dependências declaradas (Cargo.toml/package.json, quando existirem).
7. Verificar `LICENCAS_E_RESTRICOES.txt`.
8. Identificar as pendências em `docs/PENDENCIAS_DE_INVESTIGACAO.md`.

Nunca presumir que uma funcionalidade está implementada sem consultar o status na matriz **e** os testes correspondentes.

## 10. Ferramentas do repositório
- `ferramentas/oraculos/`: scripts que geram valores de referência de alta precisão (mpmath, só em desenvolvimento; ver LICENCAS D13) gravados em `tests/dados/Fxxx/`.
- `ferramentas/verificar.sh`: roda formatação, clippy, testes, `cargo deny` e a sincronia dos documentos. **Obrigatório antes de todo commit**; não commitar se falhar.
- `ferramentas/gerar_docs.py`: regenera catálogo, matriz, requisitos, testes e resumo a partir de `docs/dados/funcionalidades.json`.
- Rodar Python com `python3 -I`.

# Decisões Técnicas

Formato: contexto → decisão → alternativas → vantagens/desvantagens → custo → licença → riscos → compatibilidade. Estado: **PROPOSTA** (aguarda aprovação do responsável) / **APROVADA** / **SUBSTITUÍDA**.
Mudanças arquiteturais importantes **devem** entrar aqui antes da implementação (CLAUDE.md).

---

## DT-01 — Linguagem do núcleo: Rust · APROVADA (2026-11-17)
- **Contexto:** o runtime precisa de desempenho de nível C, segurança de memória (parsers expostos a arquivos não confiáveis) e portabilidade.
- **Alternativas:** C++20 (ecossistema numérico maduro, mas risco de memória e build complexo); Zig (imaturo); Julia como hospedeiro (GC e latência de arranque, além de licenças de dependências variadas); Go (GC, numérico fraco).
- **Vantagens:** segurança de memória, `cargo` + `cargo-deny` para auditar licenças, ecossistema numérico permissivo (faer, rustfft), FFI C fácil.
- **Desvantagens:** curva de aprendizado; menos bibliotecas numéricas maduras que em C/Fortran.
- **Custo:** médio. **Licença:** MIT OR Apache-2.0. **Riscos:** compilação lenta em workspace grande (mitigação: crates pequenos). **Compatibilidade:** Linux, Windows, macOS, arm64 e x86-64.

## DT-02 — Shell desktop: Tauri + frontend TypeScript · APROVADA (2026-11-17)
- **Alternativas:** Qt (LGPL/comercial, fora da política); Electron (MIT, mas redistribui Chromium com pacote grande e muitos avisos de licença); egui/iced nativos em Rust (MIT/Apache-2.0, mas editor de código e rich text imaturos).
- **Vantagens:** webview do SO (pacote pequeno), licenças permissivas, editores web maduros (CodeMirror), a mesma UI serve o modo navegador (F104).
- **Desvantagens:** diferenças entre webviews (WebKitGTK, WebView2, WKWebView); WebGPU desigual.
- **Riscos:** WebKitGTK no Linux tem desempenho gráfico inferior. Mitigação: renderização de figuras via wgpu nativo em janela própria ou textura compartilhada (DT-05).

## DT-03 — Estratégia de execução: interpretador sobre bytecode + JIT opcional · APROVADA (2026-11-17)
- **Alternativas:** só AST (simples e lento); transpilar para Julia/LLVM (LLVM pesado; Apache-2.0 WITH LLVM-exception); Cranelift.
- **Decisão:** bytecode próprio com especialização por tipo; JIT com Cranelift na Fase 4, se a licença com exceção for aprovada (`LICENCAS_E_RESTRICOES.txt` D4). Senão, fica o interpretador otimizado.

## DT-04 — Backend numérico: faer como padrão; OpenBLAS opcional · APROVADA (2026-11-17)
- **Alternativas:** Intel MKL (proprietária: fora); OpenBLAS (BSD-3, C/Fortran no build); BLIS (BSD-3); nalgebra (Apache-2.0, menos desempenho em grandes matrizes).
- **Decisão:** trait `LinalgBackend`; faer (MIT, puro Rust, build simples) como padrão; OpenBLAS + LAPACK de referência atrás de *feature flag*.
- **Risco:** cobertura incompleta de LAPACK no faer (ex.: QZ generalizado, Schur ordenado). Mitigação: implementar ou usar LAPACK (BSD-3) para essas rotinas.

## DT-05 — Gráficos: grafo de cena no kernel + renderização wgpu/tiny-skia · APROVADA (2026-11-17)
- **Alternativas:** Plotly/ECharts no frontend (rápido, mas acopla o modelo de objetos à UI e não roda headless); VTK (BSD-3, pesado em C++); Skia (BSD-3, build pesado).
- **Decisão:** grafo de cena próprio (F060). Renderização interativa via WebGPU/WebGL no frontend a partir de *diffs*. Exportação headless via tiny-skia (raster) e escritores vetoriais próprios (PDF/SVG/EPS).

## DT-06 — Representação de texto char · APROVADA (2026-11-17)
- Arrays char indexáveis por unidade de código **UTF-16**, para semântica de indexação previsível e interoperabilidade com Java/.NET. `string` usa UTF-8 internamente. Conversões explícitas.
- **Risco:** pares substitutos. Documentar o comportamento.

## DT-07 — Memória de objetos de referência · APROVADA (2026-11-17)
- Contagem de referência + coleta de ciclos *trial deletion* (ex.: biblioteca `bacon_rajan_cc`, MIT/Apache-2.0, ou implementação própria). A destruição determinística é exigida pelo F007 (destrutores).

## DT-08 — Motor de simulação Fluxo · APROVADA (2026-11-17)
- **Alternativas:** usar um motor Modelica (OpenModelica é OSMC-PL/GPL: fora); FMI-only; motor próprio.
- **Decisão:** motor próprio orientado a blocos (fases: compilar → plano de execução → laço do solver), com reaproveitamento dos solvers de `chimalab-math` (F049) e SUNDIALS (BSD-3).

## DT-09 — Linguagem própria com camada de compatibilidade opcional · APROVADA (2026-11-18, DT-19 item 1 por pesquisa; sujeita a parecer)
- **Opções consideradas:**
  - A: compatibilidade total de sintaxe e de nomes com o original;
  - B: linguagem própria com importação ou tradução;
  - C: linguagem própria como superfície principal, mais um modo de compatibilidade opcional e isolado para executar e migrar scripts legados.
- **Decisão: C.**
- **Base jurídica pesquisada** (não substitui parecer; fontes no DT-21):
  - **UE:** no TJUE C-406/10, *SAS Institute v. World Programming* (2012), funcionalidade, linguagem de programação e formatos de arquivo de dados não são protegidos por direito autoral; só a expressão (código-fonte e objeto).
  - **EUA:** *Google v. Oracle* (2021) tratou a reimplementação de uma interface como uso justo. A decisão é estreita, caso a caso, e não decide a proteção.
  - **Precedente prático:** implementações compatíveis de terceiros existem há décadas sem litígio público sobre sintaxe.
  - **Brasil:** a Lei 9.609/98 protege o programa como expressão.
  - **Conclusão:** o risco jurídico da camada de compatibilidade é baixo, desde que nada do código, da documentação ou dos arquivos de ajuda do original seja usado.
- **Por que C e não A:**
  - a visão de produto (DT-20) pede uma linguagem melhor, com ergonomia de Python, e não um clone;
  - C concentra o risco residual numa camada isolada e removível;
  - a parte própria pode evoluir sem amarras de compatibilidade.
- **Implementação:** parser isolado em `chimalab-lang`; o modo de compatibilidade é um front-end separado que traduz para a mesma representação intermediária.
- **Detalhes da superfície própria** (indexação, blocos): perguntas novas ao responsável, ver DT-21.

## DT-10 — Formatos do original: só `.mat`, por fontes independentes · APROVADA (2026-11-18, DT-19 item 2 por pesquisa; sujeita a parecer)
- **Base:** TJUE C-406/10 (formatos de arquivos de dados não são protegidos por direito autoral) e a finalidade de interoperabilidade.
- **Leitura e escrita de `.mat` v5 e v7.3** (v7.3 via HDF5, licença aprovada em D18).
- **Fontes de implementação:** descrições independentes e implementações de licença permissiva, como scipy.io (BSD-3-Clause) e matio (BSD-2-Clause), com atribuição. A especificação em PDF do fabricante **não** será usada, porque vem acompanhada de termos de licença próprios.
- **Testes:** arquivos gerados pelas próprias bibliotecas permissivas, pois o original não pode ser executado (DT-19 item 3).
- **Fora de escopo:** os formatos proprietários de modelos, documentos vivos, apps e figuras. Não há documentação independente nem corpus de teste lícito sem executar o original. Reavaliar se surgir documentação pública independente.

## DT-11 — Formato de dados nativo `.chimad` · APROVADA (2026-11-17)
- HDF5 (licença BSD-like; requer aprovação) vs. formato próprio (ZIP + chunks binários + JSON). Decisão final na Fase 6.

## DT-12 — Gerência de licenças no CI · APROVADA (2026-11-17)
- `cargo-deny` com lista de permissões igual à política; `license-checker` para npm; `cargo-about` gera THIRD_PARTY_NOTICES. O build falha quando aparece licença fora da lista.

## DT-13 — Nome e identidade · SUBSTITUÍDA por DT-17
- Nome original proposto: "Numora". Ver DT-17.

## DT-14 — Sem licenciamento ou ativação; telemetria opt-in · APROVADA (2026-11-17)
- Projeto pensado como software livre permissivo (licença do próprio ChimaLab a decidir: Apache-2.0 ou MIT OR Apache-2.0). Pendência P-12.

## DT-15 — Registro de aprovação do planejamento · APROVADA (2026-11-17)
- O responsável respondeu "Aprovado" ao planejamento completo (docs/ e LICENCAS_E_RESTRICOES.txt).
- **Liberado:** Fase 1 (Preparação). Fases 2+ de `chimalab-math`/`chimalab-data` podem avançar sem G1; `chimalab-lang` aguarda G1.
- **Itens que estavam abertos e foram resolvidos pela recomendação do plano:** licença do projeto = **MIT OR Apache-2.0** (P-12; padrão do ecossistema Rust, Apache-2.0 traz concessão de patentes); nome **Numora** provisório — depois substituído por ChimaLab, ver DT-17 (Q6 pendente — não publicar/registrar antes).
- **Continuam pendentes:** DT-09 (G1/Q2) e DT-10 (G2/Q3); aprovações D4 (LLVM-exception, HDF5/NetCDF, OFL); Q1–Q8.

## DT-16 — Toolchain fixado: Rust 1.100.0, edição 2024 · APROVADA (2026-11-17)
- `rust-toolchain.toml` fixa 1.100.0 (stable de 2026-11-09) para builds reprodutíveis; atualizações por PR dedicado.
- Lints de workspace: `unsafe_code = "deny"` (exceções por crate exigem registro aqui), `missing_docs = "warn"`, clippy `-D warnings` no CI.

## DT-17 — Nome definitivo: ChimaLab · APROVADA (2026-11-17, decisão do responsável)
- Sequência: "Numora" (proposta original) → "MateFoundry" → **"ChimaLab"** (decisão final do responsável, no mesmo dia).
- Consequências aplicadas: crates `numora-*` → `chimalab-*`; binário `chimalab`; módulo de simulação "ChimaLab Fluxo" (crates `fluxo-*` mantidos); extensões próprias `.chima` (script), `.chimad` (dados), `.chimanb` (documento vivo), `.chimaapp` (app), `.chimapkg` (pacote); `chimalab-project.toml` e `.chimalab/`.
- Logotipo: conceito gerado pelo responsável no ChatGPT (`assets/conceito/logo-chatgpt.png`); versão oficial minimalista em SVG com monograma "C" (`assets/logo.svg`), autorizada pelo responsável; paleta em `IDENTIDADE.md`.
- Diretório local do repositório renomeado de `Numora` para `ChimaLab` (2026-11-17, a pedido do responsável).
- Pendências: Q6 (marca, incluindo confundibilidade do sufixo "-Lab"), autoria e direitos da logo.

## DT-18 — Array N-D genérico compartilhado (`chimalab-array`) · APROVADA (2026-11-17, "prossiga" do responsável após proposta)
- **Contexto:** F040 e F046 precisam aceitar números complexos, e o runtime (Fase 2) vai precisar de um array N-D único para valores numéricos. Sem um tipo comum, cada biblioteca teria seu próprio formato, com conversões custosas na fronteira.
- **Decisão:** novo crate `chimalab-array` com `NdArray<T>`:
  - ordem column-major;
  - forma com ≥ 2 dimensões, sem singletons finais além da segunda;
  - dados em `Arc<Vec<T>>` com cópia sob demanda (*copy-on-write*), barato de clonar;
  - trait `Scalar`, implementado para `f64` e `Complex64` (outros tipos numéricos virão com o runtime);
  - conveniências 2-D (`zeros(linhas, colunas)`, `get(i, j)`) ao lado da API N-D (`zeros_nd`, `get_nd`).
  - Em `chimalab-math`: `Matrix = NdArray<f64>`, `ComplexMatrix = NdArray<Complex64>`, e as funções de álgebra linear são genéricas no escalar.
- **Alternativas:** `ndarray` (MIT/Apache-2.0, maduro, mas é row-major por padrão e sua API não reflete as semânticas que o runtime precisa: forma mínima 2-D, expansão implícita, cópia sob demanda); `faer::Mat` como tipo de valor (só 2-D); manter `Matrix` própria só para f64 (duplicaria toda a API para complexos).
- **Relação com G1:** a representação interna não depende da sintaxe da linguagem. Regras de indexação de usuário (F004) continuam sujeitas a G1 e ficarão no runtime.
- **Riscos:** refatoração de `chimalab-math` (mitigada: a suíte de testes existente tem de passar sem mudanças de semântica).

## DT-19 — Respostas do responsável às 17 decisões pendentes · APROVADA (2026-11-18)
Formato: item → resposta → situação. "P" significa que o responsável delegou a pesquisa e a decisão mais racional com base nos dados. Nas questões jurídicas, decisões tomadas por P valem como **decisão do responsável com base em pesquisa, sujeita a parecer jurídico**. Pesquisas que gerarem novas perguntas voltam ao responsável no mesmo formato numerado.

| # | Tema | Resposta | Situação |
|---|---|---|---|
| 1 | Q2/G1/DT-09: compatibilidade de linguagem | P | decidido: C (DT-09); G1 liberado sujeito a parecer; perguntas novas no DT-21 |
| 2 | Q3/G2/DT-10: formatos do original | P | decidido: só `.mat`, por fontes independentes (DT-10); G2 liberado nesse escopo |
| 3 | Q1/P-08: executar o instalador ou analisar o original em execução | A: nunca | decidido; a sala limpa continua sem análise dinâmica |
| 4 | Q4: patentes | P | decidido: política de triagem por funcionalidade (DT-21) |
| 5 | Q5: sala limpa e assistentes de IA | A: manter o processo atual | decidido (proveniência em cada commit, nada do original) |
| 6 | Q6: nome e logotipo | P | decidido: manter ChimaLab (DT-21); perguntas novas sobre reserva, registro e logo |
| 7 | Q7: citar o nome do original | B: nunca citar na documentação | decidido |
| 8 | Q8: dados de terceiros em serviços | A: adiar | decidido; volta na fase de serviços |
| 9 | Licença Apache-2.0 WITH LLVM-exception | A: aprovar | decidido (LICENCAS D14) |
| 10 | Licença OFL-1.1 | A: aprovar | decidido (LICENCAS D15) |
| 11 | Licenças BSD-like do HDF5 e do NetCDF | P | decidido: aprovadas (LICENCAS D18) |
| 12 | mpmath nos oráculos | P | decidido: já aprovado em LICENCAS D13; reinstalar num ambiente virtual local, fora do git |
| 13 | OpenBLAS para o CA2 do F040 | A: instalar só para benchmark | decidido (LICENCAS D16) |
| 14 | Push e CI | B: push após cada incremento verificado | decidido; autorização permanente |
| 15 | Visibilidade do repositório | A: privado até Q6 | decidido; confirmado PRIVATE em 2026-11-18 |
| 16 | Prioridade do próximo bloco | P | decidido: ordem no DT-21 |
| 17 | Melhorias opcionais (beta/gammainc, Schur com deflação agressiva, permutação do balanceamento) | A: fazer agora | em execução |

- **Consequência do item 7:** a documentação de usuário e os materiais públicos não citam o nome do produto original. Documentos internos de investigação (inventário, tabelas de terceiros) mantêm as referências necessárias à rastreabilidade da sala limpa, e não são publicados.

## DT-20 — Visão de produto: melhoria, não clone · APROVADA (2026-11-18, decisão do responsável)
- **Meta:** o ChimaLab deve ser uma *melhoria* do original, não apenas uma alternativa.
- **Linguagem:** o modelo de ergonomia é o Python. Funções amigáveis resolvem tarefas complexas numa chamada, com padrões sensatos, e o caminho avançado (opções, objetos, controle fino) continua disponível. Sujeita ao resultado do item 1 de DT-19 quanto à superfície sintática.
- **Rigor:** os resultados precisam servir a artigos científicos e livros. Isso significa precisão documentada, reprodutibilidade (sementes, versões, ambiente registrados) e saídas gráficas e numéricas publicáveis.
- **UI/UX:** manter o profissionalismo do original com visual substancialmente melhor, e acrescentar o que ele não tem e é útil. Exemplo: um editor em grade (popup) ao iniciar uma matriz, porque digitar matrizes em código é tedioso.
- **Processo:**
  - Ao concluir cada etapa, revisar se ela pode ser expandida, melhorada ou otimizada sem perdas.
  - A cada fase, pesquisar trabalhos e algoritmos recentes ainda não implementados em outros lugares, avaliando a utilidade com ceticismo e respeitando as regras de licença do §6 do CLAUDE.md.

## DT-21 — Resultados das pesquisas delegadas (itens P do DT-19) · APROVADA (2026-11-18)
Decisões tomadas por pesquisa, por delegação do responsável. As jurídicas estão sujeitas a parecer.

**1 e 2 — linguagem e formatos:** ver DT-09 e DT-10. Fontes:
- TJUE C-406/10, *SAS Institute v. World Programming* (2012), resumos em scl.org e fieldfisher.com;
- *Google LLC v. Oracle America*, 593 U.S. (2021), resumo do Copyright Office em copyright.gov/fair-use/summaries.

**4 — patentes:**
- **Risco baixo:** algoritmos numéricos de literatura publicada há décadas (o que foi implementado até aqui).
- **Risco concentrado:** em simulação por blocos, geração de código, máquinas de estado e recursos de IDE, o fabricante do original detém patentes ativas (há exemplos públicos em patents.justia.com, por titular). O direito autoral protege a expressão; a patente protege a invenção, e a sala limpa não protege contra ela.
- **Política adotada:**
  - antes de implementar cada funcionalidade dessas áreas (Fluxo, geração de código, recursos não triviais de IDE), fazer uma busca dirigida em bases públicas (Google Patents, USPTO, INPI) por patentes ativas (menos de 20 anos) e registrar o resultado no `docs/progresso/Fxxx.md`;
  - preferir métodos de literatura anterior às patentes encontradas, e contornar quando necessário.

**6 — nome e logotipo:**
- **Disponibilidade** (2026-11-18):
  - crates.io, npm e PyPI: o nome está livre (404);
  - GitHub: só o repositório do projeto e um projeto sem relação ("chimalabora-web");
  - os domínios .com, .org, .io, .dev e .com.br não resolvem no DNS (provavelmente livres; confirmar no registrador);
  - a busca web não achou marca ou produto "ChimaLab". Os nomes próximos são de áreas distintas (Cima Lab no MIT, médica; Chimaera, imagens médicas).
- **Sufixo "-Lab":** é um elemento comum e fraco em software científico, presente em vários produtos que coexistem. O risco de confusão é baixo, mas a busca formal no INPI, no USPTO e no EUIPO ainda não foi feita.
- **Logotipo:** obras geradas só por IA não têm direito autoral nos EUA (*Thaler v. Perlmutter*, D.C. Cir. 2025; certiorari negado em 2026); obras com contribuição criativa humana podem ter. A proteção de marca, por outro lado, vem do uso e da distintividade, não da autoria.
- **Decisão:** manter ChimaLab. Perguntas novas ao responsável: reserva de nomes, registro de marca e logotipo.

**11 — HDF5 e NetCDF:**
- HDF5: licença própria de estilo BSD-3 (identificador SPDX "HDF5"). Exige manter os avisos em fonte e binário, não usar os nomes dos titulares para endossar e manter o aviso da parte LBNL.
- netCDF-C: BSD-3-Clause desde 2018.
- As duas são permissivas e compatíveis com MIT OR Apache-2.0: aprovadas em LICENCAS D18.

**16 — ordem dos próximos blocos:**
1. As melhorias do item 17 (já autorizadas).
2. F053, números aleatórios: reprodutibilidade científica; base de estatística, otimização estocástica e testes.
3. F051, raízes e otimização.
4. F049, EDOs: alto valor científico; usa F048 e F040.
5. F047 e F044, geometria e interpolação (F044 depende de F047).
6. F043, estatística.
7. F041, esparsas.
8. F054, precisão estendida.

Justificativa: as dependências do catálogo e o valor para uso científico. A linguagem entra assim que as perguntas novas de superfície forem respondidas; ela não bloqueia os blocos acima.

## DT-22 — Superfície da linguagem própria e pendências de lançamento · APROVADA (2026-11-18, decisão do responsável)
Respostas às perguntas geradas pelo DT-21:

| # | Tema | Resposta |
|---|---|---|
| 18 | Indexação | A: **base 1** (notação matemática; facilita a migração e o uso com livros de álgebra linear) |
| 19 | Delimitação de blocos | A: **indentação**, como no Python |
| 20 | Modo de compatibilidade com scripts legados | B: depois do núcleo da linguagem própria |
| 21–24 | Reserva de nomes, registro de marca, logotipo e busca formal de marca | **adiadas** até a fase de lançamento público, depois de implementar as funcionalidades |

- **Consequências técnicas de 18 e 19:**
  - o parser precisa de um analisador léxico sensível à indentação (tokens INDENT/DEDENT);
  - o console interativo precisa tratar blocos incompletos (linha de continuação) e colagem de código;
  - a indexação base 1 vale para a linguagem; a API Rust interna continua base 0.
- **21–24:** não têm efeito técnico. São proteção jurídica e de marca para o lançamento. Lembrar o responsável antes de qualquer publicação ou quando as funcionalidades estiverem implementadas.

## DT-23 — Números aleatórios (F053): módulo `random`, geradores e padrões · APROVADA (2026-11-18, decisão técnica)
- **Local:** submódulo `chimalab-math::random`, sem dependências externas. O crate `rand` (PROPOSTA em LICENCAS) **não** será usado: os geradores do catálogo (MT, Philox, Threefry, MRG com subfluxos) exigem estado salvável e saltos que o `rand` não oferece em forma estável.
- **Geradores,** implementados a partir das publicações:

| Gerador | Fonte | Uso |
|---|---|---|
| MT19937 | Matsumoto & Nishimura, ACM TOMACS 8 (1998) | compatibilidade de prática; período 2¹⁹⁹³⁷−1 |
| Philox4x32-10 | Salmon, Moraes, Dror & Shaw, SC'11 (2011) | **padrão**: baseado em contador, fluxos independentes pela chave e salto O(1) |
| Threefry4x64-20 | idem | baseado em contador, 64 bits |
| MRG32k3a | L'Ecuyer, Oper. Res. 47 (1999); L'Ecuyer, Simard, Chen & Kelton, Oper. Res. 50 (2002) | fluxos e subfluxos por saltos de matriz (2¹²⁷ e 2⁷⁶) |
| xoshiro256++ | Blackman & Vigna, ACM TOMS 47 (2021) | rápido; saltos de 2¹²⁸ e 2¹⁹² |
| lagged Fibonacci multiplicativo e SIMD-MT (dSFMT) | Marsaglia (1985); Saito & Matsumoto (2009) | incremento posterior |

- **Por que Philox como padrão:**
  - é reprodutível em paralelo sem coordenação (um fluxo por chave, salto pelo contador);
  - passa no BigCrush;
  - tem estado pequeno;
  - o MT19937 falha nos testes de complexidade linear do BigCrush.
- **Sementes:** um inteiro de 64 bits expandido por SplitMix64 (Steele, Lea & Flood, OOPSLA 2014) para o estado de cada gerador. O MT19937 usa a inicialização `init_genrand` do artigo, por compatibilidade com sequências publicadas.
- **Estado:** `state()` devolve as palavras do estado mais o identificador do gerador, e `from_state` restaura. Requisito "estado salvável e restaurável".
- **Distribuições:**
  - **uniforme** em (0, 1) aberto, com 53 bits: ((x >> 11) + ½)·2⁻⁵³, que nunca dá 0 nem 1 (útil para ln u);
  - **inteiros** em [a, b] sem viés, pelo método de Lemire (ACM TOMACS 29, 2019);
  - **normal** pelo ziggurat de Marsaglia & Tsang (J. Stat. Softw. 5, 2000), com a correção de Doornik (2005): índice da camada e uniforme vêm de bits independentes;
  - **permutações** por Fisher–Yates, inclusive k de n.
- **Oráculos:**
  - KAT oficiais do Random123 para Philox e Threefry (arquivo `tests/kat_vectors`, BSD-3-Clause, usado só como dados de teste, com atribuição);
  - para o MT19937, o 10 000º valor com semente 5489 = 4123659995 (exigido pelo padrão C++ para `std::mt19937`) e a implementação em C do módulo `random` do Python;
  - MRG32k3a e xoshiro256++ em aritmética inteira exata em Python;
  - saltos de matriz conferidos contra passos sucessivos.
- **Testes estatísticos:**
  - bateria própria (qui-quadrado, KS da normal, correlação serial, espaçamentos de aniversário, corridas);
  - **TestU01 SmallCrush** como ferramenta de desenvolvimento (Apache-2.0 desde 2020; LICENCAS D19), por um programa C que lê a saída do gerador.
  - Resultado (2026-11-19): SmallCrush aprovada pelos 5 geradores. Na Crush, Philox, Threefry, MRG32k3a e xoshiro256++ passaram (os marginais isolados sumiram com outra semente), e o MT19937 falhou em LinearComp, como esperado. A ferramenta está em `ferramentas/testu01/` e a tabela em `docs/progresso/F053.md`.


## DT-24 — Otimização básica e raízes (F051): módulo `optim` · APROVADA (2026-11-19, decisão técnica)
- **Local:** submódulo `chimalab-math::optim`, sem dependências novas. Os subproblemas de mínimos quadrados usam `linalg::solve` (QR com pivotamento de colunas).
- **Algoritmos,** implementados a partir das publicações:

| Função | Algoritmo | Fonte |
|---|---|---|
| zero escalar | **Brent–Dekker** (padrão): bissecção, secante e interpolação quadrática inversa | R. P. Brent, *Algorithms for Minimization without Derivatives* (1973), cap. 4 |
| zero escalar | **ITP** (opção): bissecção interpolada, truncada e projetada; nunca usa mais avaliações que a bissecção mais n₀ | I. F. D. Oliveira & R. H. C. Takahashi, ACM TOMS 47(1), 2020 |
| minimização escalar limitada | Brent: seção áurea com interpolação parabólica | Brent (1973), cap. 5 |
| minimização sem restrições | Nelder–Mead com as regras de ordenação e desempate de Lagarias et al., mais os parâmetros adaptativos de Gao & Han para dimensão alta (opção) | J. A. Nelder & R. Mead, Comput. J. 7 (1965); J. C. Lagarias, J. A. Reeds, M. H. Wright & P. E. Wright, SIAM J. Optim. 9 (1998); F. Gao & L. Han, Comput. Optim. Appl. 51 (2012) |
| mínimos quadrados não negativos | Lawson–Hanson (conjunto ativo) | C. L. Lawson & R. J. Hanson, *Solving Least Squares Problems* (1974), cap. 23 |

- **Pesquisa crítica (DT-20):**
  - **ITP** entra como opção, não como padrão. Garante o pior caso da bissecção e tem convergência superlinear. Mas, nos testes de Alefeld–Potra–Shi, a vantagem média sobre Brent é pequena, e Brent é a referência que os leitores de artigos conhecem.
  - **Reinício do Nelder–Mead.** O método pode convergir para um ponto que não é estacionário (McKinnon, SIAM J. Optim. 9, 1998). A defesa simples e documentada é reiniciar a partir do melhor ponto com um simplex novo (Kelley, SIAM J. Optim. 10, 1999). O padrão é um reinício, que só é aceito se melhorar o valor. Custa algumas centenas de avaliações e corrige o contraexemplo de McKinnon (teste).
  - **NNLS em escala grande:** o TNT-NN (Myre et al., 2017) e variantes de gradiente projetado ficam para quando houver matrizes esparsas (F040). Lawson–Hanson é finito, exato e o padrão de referência.
- **Melhorias sobre o comportamento clássico:**
  - o zero com chute escalar busca o intervalo dos dois lados e contorna regiões onde f é NaN;
  - uma mudança de sinal que não é raiz (polo, descontinuidade) é **sinalizada** no resultado, em vez de virar uma raiz falsa;
  - a minimização limitada confere os extremos do intervalo, porque o método de Brent nunca os avalia;
  - no Nelder–Mead, valores NaN contam como +∞, o que aceita funções com domínio restrito.
- **Interface:**
  - `Options` reúne as tolerâncias, os limites de iterações e de avaliações, o histórico e uma função de saída chamada a cada iteração, que pode interromper;
  - o resultado traz a solução, o valor, um `ExitFlag` padronizado com código numérico, as contagens e o histórico, usado pela exibição iterativa e pelos gráficos de iteração (CA3).
- **Erros** com identificadores estáveis `chimalab:otimizacao:*`: intervalo inválido, sem mudança de sinal, ponto inicial inválido, tolerância ou opção inválida e formas incompatíveis.

## DT-25 — Finalidade do projeto: portfólio de código aberto · APROVADA (2026-11-19, decisão do responsável)
- O ChimaLab é um projeto de portfólio, de código aberto (MIT OR Apache-2.0, DT-15). Não haverá registro de marca nem outros registros legais.
- **Dispensados:**
  - itens 21–24 do DT-22: reserva de nomes, registro no INPI, logotipo com autoria humana documentada e busca formal de marca;
  - Q6 e as pendências P-13 e P-15.
- **Continua valendo:**
  - sala limpa (CLAUDE.md §3);
  - não citar o nome do original na documentação de usuário;
  - política de licenças das dependências;
  - não usar nome nem identidade visual que se confundam com marcas de terceiros. Isso não depende de registro próprio.
- **Repositório:** continua privado até o responsável pedir a publicação.

## DT-26 — Equações diferenciais ordinárias (F049): módulo `ode` · APROVADA (2026-11-19, decisão técnica)
- **Local:** submódulo `chimalab-math::ode`, sem dependências novas. Usa `linalg` (LU, nos métodos implícitos) e `optim::fzero` (localização de eventos).
- **Incrementos:**
  1. Infraestrutura e métodos explícitos:
     - Bogacki–Shampine 3(2), por L. F. Shampine & P. Bogacki, Appl. Math. Lett. 2 (1989);
     - Dormand–Prince 5(4), por J. R. Dormand & P. J. Prince, J. Comput. Appl. Math. 6 (1980);
     - saída densa, saída em instantes pedidos, eventos com direção e término.
  2. Métodos para problemas rígidos (feito em 2026-11-20):
     - NDF e BDF de ordem variável 1–5 (Shampine & Reichelt, 1997);
     - Jacobiano fornecido ou por diferenças finitas.
  3. Restante dos métodos:
     - Radau IIA de ordem 5 (Hairer & Wanner, *Solving ODE II*), com matriz de massa e DAE de índice 1 (feito em 2026-11-20; o sistema complexo é resolvido na forma real 2n×2n);
     - Rosenbrock;
     - pares de alta ordem de Verner, para tolerâncias apertadas;
     - detecção de rigidez e problemas totalmente implícitos.
     - Pesquisa (DT-20), a confirmar no incremento: os coeficientes Rodas5P (G. Steinebach, 2023) corrigem a perda de ordem do Rodas5 em problemas não autônomos e DAE. São candidatos para o Rosenbrock, depois de conferir as condições de ordem como no `f049_tabelas_rk.py`.
- **Melhorias sobre a prática comum (DT-20):**
  - **Tolerâncias padrão:** relativa 1e-6 e absoluta 1e-9, em vez de 1e-3 e 1e-6. Os padrões frouxos dão gráficos visivelmente errados em problemas comuns, como órbitas e osciladores, e o custo extra é pequeno. A relativa é limitada por baixo a 100·ε_máq, com aviso.
  - **Norma do erro:** RMS de eᵢ/(atolᵢ + rtol·max(|yᵢ|, |ŷᵢ|)), como em Hairer, Nørsett & Wanner. Com ela, o resultado não depende de como as componentes estão ordenadas ou duplicadas.
  - **Controle de passo:** PI (Gustafsson, 1991; Hairer, Nørsett & Wanner, §IV.2), com β = 0,04 no DOPRI5, o que reduz oscilações de passo. Passo inicial pelo algoritmo de Hairer, Nørsett & Wanner (§II.4).
  - **Eventos:**
    - localizados por Brent sobre a saída densa, com precisão total em t;
    - todos os eventos de um mesmo passo são processados em ordem de tempo;
    - eventos simultâneos são todos registrados;
    - um evento terminal encerra a integração exatamente no instante localizado, e esse ponto entra na saída.
  - **Falhas:**
    - f com NaN ou ±∞ rejeita o passo e o reduz;
    - passo abaixo de 16·ε·|t| encerra com o resultado parcial e o estado `PassoMinimo` ou `NaoFinito`, nunca com um erro que descarte o que já foi calculado.
- **Saída:**
  - com dois instantes em `tspan`: cada passo, mais `refine` − 1 pontos interpolados (padrão 4 no DOPRI5 e 1 no BS23);
  - com mais instantes: exatamente nesses instantes;
  - `dense: true` guarda a solução contínua, avaliável em qualquer t do intervalo.
- **Oráculos:**
  - condições de ordem de Butcher sobre árvores enraizadas, em aritmética racional exata, para as tabelas e as saídas densas (`ferramentas/oraculos/f049_tabelas_rk.py`);
  - soluções analíticas;
  - referências mpmath (`odefun`, série de Taylor em alta precisão).

## DT-27 — Interpolação (F044): módulo `interp` · APROVADA (2026-11-20, decisão técnica)
- **Local:** submódulo `chimalab-math::interp`, sem dependências novas.
- **Incremento 1 (1-D):** interpolante reutilizável e polinômio por partes (`PPoly`) com derivada e integral.
- **Métodos:**
  - linear;
  - vizinho mais próximo, anterior e seguinte;
  - **pchip:** Hermite cúbica monótona, com as derivadas de F. N. Fritsch & J. Butland, SIAM J. Sci. Stat. Comput. 5 (1984) 300–304, e a fórmula de três pontos nos extremos com as restrições de F. N. Fritsch & R. E. Carlson, SIAM J. Numer. Anal. 17 (1980) 238–246;
  - **Akima:** H. Akima, J. ACM 17 (1970) 589–602, e a variante modificada (makima), com pesos wᵢ = |δᵢ₊₁ − δᵢ| + |δᵢ₊₁ + δᵢ|/2. Ela evita a ultrapassagem em platôs e é descrita na documentação pública do SciPy (BSD-3);
  - **spline cúbica:** condições not-a-knot (padrão), natural, derivadas dadas nos extremos e periódica (C. de Boor, *A Practical Guide to Splines*, ed. rev. 2001, cap. IV). O sistema tridiagonal é resolvido com pivotamento parcial, e o periódico pela fórmula de Sherman–Morrison.
- **Incremento 2:** grades N-D (linear, cúbica tensorial). **Incremento 3:** dados dispersos, que dependem da triangulação de Delaunay de F047.
- **Melhorias sobre a prática comum (DT-20):**
  - **Extrapolação explícita e uniforme:** `Extrap::{Nan (padrão), Extrapolate, Value(v), Clamp}`, igual em todos os métodos. A prática comum muda o padrão conforme o método e surpreende o usuário.
  - **Pontos fora de ordem:** são ordenados automaticamente, junto com os valores. A ordem decrescente é aceita.
  - **Pontos duplicados:** erro explícito com identificador, nunca resultado silencioso.
  - **Derivadas e integrais exatas** do interpolante, pelo `PPoly`.
- **Oráculos:**
  - reprodução exata de polinômios (cúbicos pela spline not-a-knot, lineares por todos os métodos);
  - continuidade C¹/C² conferida nas quebras;
  - monotonia do pchip;
  - sistema da spline resolvido em alta precisão pelo mpmath com eliminação densa (`ferramentas/oraculos/f044_splines.py`), independente do algoritmo tridiagonal.

## DT-28 — Geometria computacional (F047): módulo `geom`, implementação própria · APROVADA (2026-11-20, decisão técnica)
- **Local:** submódulo `chimalab-math::geom`, sem dependências novas.
  - O crate `spade` (MIT OR Apache-2.0, PROPOSTA em LICENCAS) **não** será usado nesta etapa.
  - Motivos: a triangulação precisa expor os índices dos triângulos, as vizinhanças e as consultas de localização no formato da biblioteca (arrays de índices), precisa de predicados exatos auditáveis e precisa ser estendida a 3-D e aos alfa-shapes.
  - Uma implementação própria, curta e testada contra predicados exatos, cobre isso sem adaptadores.
- **Incremento 1 (2-D):**
  - **Predicados geométricos robustos** orient2d e incircle: filtro em ponto flutuante com limite de erro, e cálculo exato por expansões quando o filtro não decide (J. R. Shewchuk, *Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates*, Discrete Comput. Geom. 18 (1997) 305–363; aqui, a avaliação exata usa expansões sem a etapa adaptativa intermediária).
  - **Delaunay 2-D** incremental de Bowyer–Watson (A. Bowyer, Comput. J. 24 (1981); D. F. Watson, Comput. J. 24 (1981)), com localização por caminhada e inserção em ordem espacial (curva de Hilbert), o que dá custo esperado ~O(n log n).
  - **Fecho convexo 2-D** pela cadeia monótona de A. M. Andrew, Inf. Process. Lett. 9 (1979).
  - **Polígonos:** área com sinal e centroide (fórmula de Gauss), perímetro, e ponto no polígono pelo número de voltas, com os casos de borda decididos pelo orient2d exato.
  - **Voronoi 2-D** como dual da triangulação.
- **Incrementos seguintes:**
  - Delaunay e fecho 3-D;
  - operações booleanas de polígonos: algoritmo de Vatti ou de Martinez–Rueda–Feito, a escolher com pesquisa;
  - alfa-shapes;
  - buffer.
- **Oráculos:**
  - critério do círculo vazio conferido com o incircle exato em toda aresta interna, para 1e5 pontos aleatórios (CA1);
  - predicados contra aritmética racional exata (frações de inteiros) em casos quase degenerados;
  - áreas de polígonos com coordenadas inteiras (exatas);
  - fecho contra força bruta.

## DT-29 — Estatística descritiva e pré-processamento (F043): módulo `stats` · APROVADA (2026-11-20, decisão técnica)
- **Local:** submódulo `chimalab-math::stats`, sem dependências novas.
- **Forma:**
  - as funções operam em vetores `&[f64]`;
  - `along_dim` aplica qualquer redução ou transformação ao longo de uma dimensão de um array N-D column-major `(dados, forma)`, como em `trapz` (F048);
  - a dimensão padrão é a primeira não unitária. A regra fica com o runtime, que conhece a forma.
- **Incremento 1:**
  - soma, produto, média, mediana, moda, variância e desvio, mínimo, máximo e amplitude, com NaN incluído ou omitido;
  - quantis pelos 9 tipos de Hyndman & Fan;
  - somas, produtos, máximos e mínimos cumulativos;
  - estatísticas móveis com janela par ou ímpar, simétrica ou assimétrica, e bordas encolhidas, preenchidas ou descartadas;
  - normalização (z-score, intervalo, norma, centro e escala) e reescala;
  - outliers pelos métodos da mediana, da média e dos quartis, com detecção, remoção e preenchimento;
  - contagem em histograma com bordas dadas ou automáticas.
- **Incremento 2:**
  - suavização (gaussiana, Savitzky–Golay, loess/lowess robusto);
  - detecção de mudanças e de extremos locais (proeminência);
  - outliers por Grubbs/GESD e por janela móvel;
  - pesos.
- **Rigor e melhorias sobre a prática comum (DT-20):**
  - **Soma:** compensada de Neumaier (Z. Angew. Math. Mech. 54, 1974), com erro O(ε) independente de n, enquanto o erro da soma ingênua cresce com n;
  - **Média:** sem overflow intermediário: a média de [1e308, 1e308] é 1e308, não ∞;
  - **Variância:** dois passos com correção (T. F. Chan, G. H. Golub & R. J. LeVeque, Am. Stat. 37 (1983) 242–247), estável quando a média é grande diante do desvio;
  - **Mediana e quantis:** seleção em O(n), sem ordenação completa, e média dos dois centrais sem overflow;
  - **Quantis:** o padrão é o tipo 7 de Hyndman & Fan, Am. Stat. 50 (1996), o mais difundido em R, NumPy e planilhas. Os 9 tipos estão disponíveis;
  - **Média móvel:** soma deslizante compensada, recalculada periodicamente para não acumular deriva;
  - **Máximo e mínimo móveis:** fila monótona, O(n) independentemente da janela.
- **Oráculos:**
  - valores calculados à mão;
  - identidades (média e variância de sequências aritméticas em forma fechada; quantis de tipo k contra as fórmulas de Hyndman & Fan aplicadas por força bruta);
  - mpmath para somas e variâncias mal condicionadas.

## DT-30 — Matrizes esparsas e solvers iterativos (F041): módulo `sparse` · APROVADA (2026-11-20, decisão técnica)
- **Local:** submódulo `chimalab-math::sparse`.
- **Dependência:** liga a feature `sparse-linalg` do `faer` 0.24.4. Não entra nenhum crate novo: a feature só compila código que já está no crate aprovado (MIT, LICENCAS). Ela fornece:
  - ordenações AMD e COLAMD, implementações próprias do faer;
  - Cholesky supernodal e LU esparsa com pivotamento parcial, para a resolução direta.
  - Os componentes do SuiteSparse (GPL/LGPL em parte, catálogo F041) **não** são usados.
- **Incremento 1:**
  - **formato:** CSC com índices de linha ordenados por coluna, construído a partir de triplas, com soma de repetidos e remoção de zeros (decisão: zeros explícitos não são guardados);
  - **operações:**
    - conversão de e para denso;
    - transposta;
    - produtos matriz–vetor e matriz–matriz esparsa (Gustavson, ACM TOMS 4, 1978);
    - soma e escala;
    - diagonal;
    - identidade;
    - triplas;
  - **resolução direta:** Cholesky quando a matriz é simétrica com diagonal positiva e a fatoração tem sucesso; senão, LU. O método usado é informado;
  - **iterativos:** gradiente conjugado precondicionado (Hestenes & Stiefel, 1952), GMRES(m) com reinício (Saad & Schultz, 1986), BiCGSTAB (van der Vorst, 1992) e MINRES (Paige & Saunders, 1975);
  - **precondicionadores:** Jacobi, IC(0) e ILU(0) no padrão de A (Saad, *Iterative Methods for Sparse Linear Systems*, 2.ª ed., 2003, cap. 10);
  - **resultado dos iterativos:** flag de convergência, resíduo relativo, número de iterações e histórico do resíduo. A não convergência devolve o melhor iterado com um aviso, nunca um erro.
- **Incremento 2:**
  - autovalores e valores singulares parciais (eigs/svds) por Lanczos e Arnoldi com reinício implícito (Sorensen, 1992), implementação própria;
  - LSQR (Paige & Saunders, 1982);
  - complexos.
- **Oráculos:**
  - Poisson 2-D com solução conhecida;
  - resolução esparsa contra a densa (F040);
  - resíduos verdadeiros recalculados;
  - memória proporcional ao nnz, pelos tamanhos dos vetores do CSC.

## DT-31 — Precisão e aritmética especial (F054): módulo `numeric` · APROVADA (2026-11-20, decisão técnica)
- **Local:** submódulo `chimalab-math::numeric`, sem dependências novas.
- **Incremento 1:**
  - **representação:** espaçamento (eps(x), o ulp), maior inteiro consecutivo exato (flintmax), maior e menor normal (realmax e realmin), em f64 e f32; vizinho em ponto flutuante (nextafter); limites de cada tipo inteiro;
  - **divisão inteira** com 4 arredondamentos (truncar, arredondar, piso, teto), com saturação e regras explícitas para divisão por zero;
  - **conversões de base** de 2 a 36 para inteiros sem sinal de até 128 bits, com o número mínimo de dígitos, mais binário com complemento de dois para negativos;
  - **ida e volta** entre f64 e a cadeia hexadecimal dos bits IEEE 754.
- **Incremento 2:** precisão estendida exposta ao usuário: o tipo dupla-dupla, já usado internamente em F042 (Dekker, 1971), e um decimal de precisão arbitrária próprio, sem GMP/MPFR (LGPL).
- **Melhorias:**
  - as conversões rejeitam entradas inválidas com erro e identificador (caractere fora da base, overflow), em vez de devolver NaN silenciosamente;
  - aceitam separadores `_` e o prefixo `0x`/`0b`/`0o`, como em linguagens modernas.

## DT-32 — Modelo de valores do runtime (F003–F005, semântica independente da sintaxe) · APROVADA (2026-11-20, decisão técnica)
- **Local:** `chimalab-runtime::value`, sobre `chimalab-array::NdArray`. A semântica não depende das perguntas 27–36 de sintaxe: tipos, conversões, aritmética mista, expansão e indexação são regras de comportamento, fixadas pelos CAs de F003–F005.
- **Classes:**
  - `double` e `single`, reais ou complexos; complexo `single` fica para depois;
  - inteiros com e sem sinal de 8 a 64 bits;
  - `logical` e `char`.
  - Cada classe tem armazenamento próprio (1 byte por elemento em `int8`), gerado por macros.
- **Regras (F003):**
  - conversão para inteiro: arredonda o meio para longe de zero, satura e leva NaN a 0;
  - inteiro com `double` dá a classe inteira, calculando em ponto flutuante, depois arredondando e saturando;
  - inteiros de classes diferentes, ou inteiro com complexo, dão erro `chimalab:runtime:classesIncompativeis`;
  - `single` com `double` dá `single`; `logical` e `char` em aritmética valem como `double`;
  - divisão inteira por zero satura no máximo ou no mínimo, e 0/0 dá 0;
  - `int64` e `uint64` somam, subtraem e multiplicam em aritmética exata (i128) quando o outro operando é inteiro ou `double` com valor inteiro. Assim não perdem bits acima de 2⁵³, o que a prática comum perde em alguns casos;
  - raiz, logaritmo e potência de reais negativos dão complexo automaticamente.
- **Expansão implícita (F005):** dimensões iguais ou unitárias, pela `broadcast_shape` de `chimalab-array`.
- **Indexação (F004):** a API do runtime recebe índices na base 1 (DT-22) e os converte; as formas são subscrito, linear, lógica, intervalo e "tudo". A atribuição fora dos limites cresce o array preenchendo com zero da classe, e a atribuição de vazio remove.

## DT-33 — Grafos e redes (F052): módulo `graph` · APROVADA (2026-11-20, decisão técnica)
- **Local:** `chimalab-math::graph`, sem dependências novas.
- **Forma:**
  - grafo dirigido ou não, com n nós numerados e arestas ponderadas em listas de adjacência;
  - multigrafos e laços são aceitos;
  - as propriedades em tabela de nós e arestas dependem de F024 e virão com as tabelas;
  - a plotagem com layout depende de F060 (gráficos).
- **Algoritmos do incremento 1, das publicações:**
  - BFS e DFS;
  - componentes conexos, e fortemente conexos por Tarjan (SIAM J. Comput. 1, 1972);
  - ordenação topológica por Kahn (Commun. ACM 5, 1962);
  - caminhos mínimos: Dijkstra (Numer. Math. 1, 1959) com fila de prioridade, Bellman–Ford com detecção de ciclo negativo, e BFS sem pesos;
  - árvore geradora mínima por Kruskal (Proc. AMS 7, 1956), com união e busca;
  - fluxo máximo por Dinic (1970), com o corte mínimo;
  - centralidades de grau, proximidade e intermediação (U. Brandes, J. Math. Sociol. 25, 2001).
- **Oráculos:** força bruta em grafos pequenos (todos os caminhos, todos os cortes, todas as árvores geradoras), dualidade fluxo máximo = corte mínimo, e redes de teste publicadas com valor conhecido.

## DT-34 — Superfície da linguagem própria (F001–F006) · APROVADA (2026-11-20, decisão do responsável)
Complementa o DT-22 (base 1, blocos por indentação). Respostas às perguntas 27–36 de `PENDENCIAS_DE_INVESTIGACAO.md`:
- **Produto (27.A):** `*` é o produto matricial, e `.*`, `./` e `.^` são as operações elemento a elemento.
- **Literais de matriz (28.C):** `[1 2; 3 4]` e `[[1, 2], [3, 4]]` produzem o mesmo valor.
- **Intervalos (29, P → A):** `a:b` e `a:passo:b`, com o fim incluído.
  - Com base 1, `1:n` percorre exatamente os índices válidos e `A[2:end]` lê como se escreve. A combinação base 0 com fim excluído do Python e a combinação base 1 com fim incluído (Julia, Fortran, notação matemática) são coerentes cada uma; misturar base 1 com fim excluído forçaria `1:n+1`, a pior das opções.
  - O nome `range` **não** terá a semântica do Python: um `range(a, b)` com fim excluído causaria erros silenciosos de um elemento. Haverá `range(a, b; length=n)` e `range(a, b; step=h)`, sempre com o fim incluído e argumento nomeado obrigatório. `range(a, b)` sem nome é erro, com mensagem que sugere `a:b`.
  - `1..10` fica livre para uso futuro.
- **Funções (30.A):** `def f(x, y=2):`, com `return a, b` e `a, b = f(x)`. A função conhece quantas saídas foram pedidas (equivalente a `nargout`, para F002).
- **Comentários (31.A):** `#` até o fim da linha e `#= … =#` em bloco, com aninhamento.
- **Textos (32.A):** aspas simples e duplas equivalentes, mais f-strings `f"x = {x}"`. Como `'` também é a transposta (35), o analisador decide pelo contexto: depois de um valor (identificador, `)`, `]`, `end`, literal, outra `'`) é transposta; nos demais casos abre um texto, a mesma regra usada por linguagens com as duas notações.
- **Indexação e chamada (33.A):** `A[i, j]` indexa e `f(x)` chama.
- **Exibição (34, P → A no console e B nos documentos):**
  - No console (CA de F001), todo comando mostra o seu valor, inclusive atribuições (`x = 5` mostra `x = 5`), e `;` no fim suprime. É o comportamento do REPL de Julia e do original, adequado à exploração numérica. O resultado sem atribuição vai para `ans`.
  - Nos documentos executáveis e nos scripts, só a última expressão da célula é mostrada, e `;` também a suprime (convenção de IPython e Jupyter). Assim um laço numa célula não inunda a saída.
  - Nos dois modos, `display(x)` e `print(x)` mostram explicitamente.
- **Transposta (35.C):** `A'` (conjugada) e `A.'` (simples), e também `A.H` e `A.T`.
- **Último índice (36.A):** `end` dentro de `[]`, como em `A[end]` e `A[end-1, :]`. Índices ≤ 0 são erro, com identificador estável e uma mensagem que lembra que a base é 1 e sugere `end`.

## DT-35 — Publicação: repositório público novo, sem o histórico privado · APROVADA (2026-11-20, decisão do responsável)
- **Contexto:** a cota do CI de repositório privado se esgotou (pergunta 37). O responsável decidiu publicar o projeto (DT-25) num repositório novo, `LucasRamos13/ChimaLabs`; o repositório privado anterior será apagado.
- **Decisão:**
  - o estado atual entra num commit inicial único, com o e-mail `noreply` do GitHub como autor;
  - o material da investigação em sala limpa (inventário de mídia, listas de componentes, tabelas de terceiros, análise técnica e riscos contratuais) **não** é publicado e fica guardado localmente pelo responsável, com o histórico completo;
  - o catálogo, a matriz, os requisitos e a política de licenças foram reescritos sem esse material: o catálogo deixou de citar evidências de componentes e as extensões futuras deixaram de ser listadas por nome;
  - a documentação pública não cita o nome do produto original (DT-19, item 7).
- **Auditoria antes do push:** busca de nomes de terceiros, caminhos locais, e-mail pessoal, segredos e cabeçalhos de licença copyleft em todos os arquivos; `verificar.sh` completo.


## DT-36 — Analisador léxico da linguagem (`chimalab-lang`) · APROVADA (2026-11-25, decisão técnica dentro do DT-22 e do DT-34)
- **Local:** `chimalab-lang::lexico`, sem dependências. Escrito à mão (o `logos` da Parte B das licenças fica dispensado: a sensibilidade a indentação, ao contexto de `'` e ao espaço dentro de colchetes não se expressa bem em autômatos gerados).
- **Palavras:** todas saem como identificadores. As palavras-chave são decididas no parser (decididas no DT-39); o léxico não precisa mudar quando elas forem fixadas.
- **Blocos por indentação** (DT-22): tokens `Indent` e `Dedent` como no Python. Linhas em branco e só de comentário não contam. Tabulação conta até o próximo múltiplo de 4 colunas; uma linha que recua para uma coluna que não abriu bloco é erro `chimalab:lexico:indentacaoInconsistente`. No fim, os blocos abertos são fechados.
- **Dentro de `(` e `{`** a quebra de linha é ignorada (continuação implícita). **Dentro de `[`** ela vira `Newline`, que o parser trata como separador de linhas da matriz (equivale a `;`).
- **Espaço significativo dentro de `[ ]`:** cada token guarda se veio depois de espaço. O parser usa isso para separar elementos: `[1 -2]` tem dois elementos e `[1 - 2]` tem um.
- **`'`** (DT-34, item 32): é transposta se o token anterior termina um valor (identificador, número, texto, `)`, `]`, `}`, `'`, `.'`) e, dentro de colchetes, não há espaço antes; senão, abre um texto.
- **Números:** decimais com `_` entre dígitos, expoente `e`/`E`, `0x`, `0o` e `0b`, e o sufixo imaginário `i` ou `j` colado (a escolha final é a pergunta 43; o léxico aceita os dois). O texto original é guardado, para inteiros acima de 2⁵³.
- **Textos:** aspas simples e duplas equivalentes, com escapes `\n`, `\t`, `\r`, `\0`, `\\`, `\'`, `\"` e `\u{…}`; o prefixo `r` desliga os escapes (caminhos do Windows). **f-strings:** `f"…"` com partes `{expr}` e `{expr:formato}` (o primeiro `:` fora de parênteses e colchetes começa o formato; um intervalo dentro da expressão precisa de parênteses), `{{` e `}}` literais. A expressão fica como texto-fonte com a posição, e o parser a analisa recursivamente.
- **Comentários:** `#` até o fim da linha; `#= … =#` aninhados, podendo atravessar linhas.
- **Posições:** linha e coluna (1-based, em caracteres) e o intervalo em bytes, para as mensagens de erro com posição (F001).
- **Erros** com identificador estável `chimalab:lexico:…`: `textoNaoTerminado`, `comentarioNaoTerminado`, `caractereInvalido`, `indentacaoInconsistente`, `numeroInvalido`, `escapeInvalido`, `fstringInvalida`, `delimitadorDesbalanceado`.

## DT-37 — Expressões da linguagem: AST e precedência (`chimalab-lang::sintaxe`) · APROVADA (2026-11-25, decisão técnica revisável)
- **Escopo deste incremento:** só expressões. Instruções, blocos e operadores lógicos vêm no próximo incremento (DT-39).
- **Precedência**, da menor para a maior:
  1. comparação `==`, `!=`, `<`, `<=`, `>`, `>=` — **não associativa**: `a < b < c` é erro de sintaxe com sugestão. No original, a cadeia compara o resultado lógico da primeira com c, um erro clássico; o Python a lê como conjunção, e preferimos não adivinhar;
  2. intervalo `a:b` e `a:passo:b`;
  3. aditivos `+`, `-` (à esquerda);
  4. multiplicativos `*`, `/`, `\`, `.*`, `./`, `.\` (à esquerda);
  5. sinal `+`, `-` prefixos: `-2^2` é −4, como na notação matemática;
  6. potência `^`, `.^` **à direita**: `2^3^2` = 2^9, como na matemática, no Python e no Julia (o original associa à esquerda); o expoente pode ter sinal (`2^-1`);
  7. pós-fixos: transpostas `'` e `.'`, indexação `A[…]`, chamada `f(…)` e membro `.nome` (inclui `.T` e `.H`).
- **Dentro de `[ ]`:** linhas separadas por `;` ou quebra de linha, elementos por `,` ou espaço. Com espaço, `[1 -2]` tem dois elementos (sinal colado ao número depois de espaço) e `[1 - 2]` um; `[a (1)]` e `[a [1]]` têm dois. A forma aninhada com vírgulas `[[1, 2], [3, 4]]`, em que todos os elementos são literais de matriz, empilha as linhas (DT-34, item 28); com espaços, `[[1 2] [3 4]]` concatena na horizontal.
- **Índices:** `end` dentro de `A[…]` é o último índice da dimensão; `:` sozinho é "todos". Fora de índices, `end` é erro.
- **Chamadas:** argumentos posicionais e nomeados (`range(0, 1; length=5)` e `f(x, y=2)`); posicional depois de nomeado é erro.
- **f-strings:** cada `{expr}` é analisado como expressão, com a posição ajustada para o texto original.
- **Erros** `chimalab:sintaxe:…`: `tokenInesperado`, `fimInesperado`, `comparacaoEncadeada`, `endForaDeIndice`, `argumentoPosicionalDepoisDeNomeado`, e os léxicos repassados.

## DT-38 — Concatenação no runtime (`horzcat`, `vertcat`, `concat`) · APROVADA (2026-11-25, decisão técnica revisável)
- **Local:** `chimalab-runtime::concat`. É a semântica dos literais `[A B; C D]` (DT-34, item 28), independente da sintaxe.
- **Dimensões:** ao longo de `dim` (1-based), todas as outras dimensões iguais, com dimensões finais 1 implícitas; o resultado soma os tamanhos em `dim`. Vazios 0×0 (`[]`) são ignorados; outros vazios (0×3, por exemplo) participam da verificação. Erro `chimalab:runtime:dimensoesIncompativeis` com as duas formas.
- **Classes:** as da aritmética (DT-32): inteiro domina (com arredondamento e saturação), inteiros de classes diferentes dão erro, `single` domina `double`, complexo só com `double`. Operandos todos lógicos ou todos caracteres mantêm a classe.
- **Caractere com outra classe é erro** (`classesIncompativeis`), em vez de converter números em caracteres sem aviso, uma armadilha comum (`['a' 66]` virando "aB"). A conversão explícita fica para as funções de texto. Revisável se o modo de compatibilidade precisar do comportamento antigo.

## DT-39 — Construções da linguagem e primeiro interpretador (respostas 40–48) · APROVADA (2026-11-25, decisão do responsável)
- **Blocos (40.A):** como no Python: `if c:` / `elif c:` / `else:`, `for k in expr:`, `while c:`, `break`, `continue`, `try:` / `except e:`, `match x:` / `case v:`.
- **`for` sobre matriz (41.A):** percorre as colunas (coincide com os elementos num vetor linha).
- **Lógicos (42.C):** as duas formas, sem conflito: `and` ≡ `&&` e `or` ≡ `||` com curto-circuito (escalares), `not` ≡ `!` (elemento a elemento em arrays); `&` e `|` elemento a elemento. `!` só é prefixo; o léxico já separa `!=`.
- **Imaginário (43.A):** sufixo `i` ou `j` colado ao número.
- **Diferente (44.A):** `!=`; `~=` é erro `chimalab:sintaxe:diferenteComTil` com a sugestão (implementado no parser).
- **Continuação de linha (45: B pedido, A aplicado por conflito):** a barra invertida já é a divisão à esquerda; como continuação no fim da linha, `[A \`↵`b]` viraria `[A b]` (concatenação silenciosa no lugar do sistema linear). Fica só a continuação implícita dentro de `( )`, `[ ]` e `{ }`, conforme a instrução do responsável para o caso de conflito.
- **Fim de bloco no console (46.A):** uma linha em branco encerra o bloco (o léxico já detecta a entrada incompleta).
- **Funções anônimas (47.A):** `x -> x^2` e `(x, y) -> x + y`.
- **Primeiro interpretador (48.A):** avaliador que percorre a AST, em `chimalab-runtime`, com `ans`, exibição e `;`; o bytecode do DT-03 vem depois de medição de desempenho, verificado contra o avaliador da árvore (que vira oráculo).

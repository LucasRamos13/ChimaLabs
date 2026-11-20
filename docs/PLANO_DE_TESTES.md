# Plano de Testes e Validação da Paridade

Os casos por funcionalidade (pré-condições, entradas, procedimento, resultado esperado, casos extremos e de erro, critérios) estão em **`TESTES_POR_FUNCIONALIDADE.md`** (TF-xxx ↔ Fxxx), gerado a partir do catálogo.

## 1. Princípios
1. A validação verifica o **atendimento aos requisitos**, não a reprodução do código ou da aparência do original.
2. **Oráculos independentes:** soluções analíticas, conjuntos de referência publicados (NIST StRD para estatística, conjuntos de teste de matrizes, problemas de referência de EDO/DAE, FMI Cross-Check), bibliotecas de referência independentes (SciPy/NumPy, BSD-3) usadas **só como oráculo em testes**, sem embuti-las no produto. **Nunca** usar saídas do produto original.
3. Um teste só passa a existir com critério de aceitação rastreável (CAx). Funcionalidade sem teste não pode passar de IMPLEMENTADA.

## 2. Tipos de teste

| Tipo | Ferramenta proposta | Escopo |
|---|---|---|
| Unitário | `cargo test`, Vitest | Funções puras de math, data, parser, componentes da UI |
| Propriedades | proptest | Invariantes (ifft(fft(x))≈x, ordenação estável, ida-e-volta de formatos) |
| Conformidade de linguagem | executor próprio sobre `tests/conformidade/*.m` + `.esperado` | Semântica da linguagem (F001–F020) |
| Integração | testes de workspace Rust; kernel em processo | Runtime + bibliotecas + E/S |
| Ponta a ponta | Playwright sobre o desktop (modo teste) | IDE, apps, documentos, editor Fluxo |
| Regressão | toda a suíte no CI a cada PR; testes *snapshot* (insta) para saídas textuais e imagens (comparação perceptual) | Todos |
| Compatibilidade | matriz Linux/Windows/macOS × x86-64/arm64; arquivos de amostra de formatos | E/S, instaladores |
| Desempenho | criterion + benchmarks de scripts | Metas dos CA (ex.: F019, F040, F150, F200) |
| Segurança | cargo-fuzz (todos os parsers), cargo-audit, testes de XXE/TLS/token, varredura de rede sem consentimento (RNF-07) | IO, NET, EXT, INS |
| Importação/exportação | corpus de arquivos gerados por ferramentas de terceiros com licença compatível | F110–F120, F069, F107, F216 |
| Persistência e recuperação | encerramento forçado durante gravação, arquivos corrompidos | F097, F106, F110, F216 |
| Acessibilidade | axe-core + roteiro manual | UI (RNF-09) |
| Numérica (precisão) | tolerâncias por função documentadas | F040–F055 |
| Simulação | modelos de referência com solução analítica; comparação entre solvers | F200–F227 |

## 3. Ambientes
- CI: runners Linux/Windows/macOS, mais aarch64 Linux.
- Headless: renderização offscreen (tiny-skia) para testes de gráficos sem display.
- Rede: servidores de teste locais (HTTP, FTP, SMTP, MQTT) em contêineres. Nada de serviço externo real.

## 4. Indicadores de cobertura

### 4.1 Cobertura do catálogo (atual)
| Indicador | Valor |
|---|---|
| Funcionalidades identificadas (núcleo + Fluxo) | 162 |
| Documentadas (18 campos) | 162 |
| Com requisito completo (RF + CA) | 162 |
| Em INVESTIGADA PARCIALMENTE | 12 |
| Em NÃO VERIFICÁVEL | 1 (F124) |
| Dependentes de serviço externo | 8 (F063, F105, F121, F138, F166, F167, F171, F174) |

### 4.2 Cobertura da implementação (atual)
| Indicador | Valor |
|---|---|
| Planejadas | 131 / 162 |
| Em desenvolvimento | 17 |
| Implementadas | 1 |
| Testadas | 0 |
| Validadas | 0 |

> Nenhuma funcionalidade está VALIDADA ainda; o estado de cada uma está na matriz e em `docs/progresso/`.

Os indicadores são recalculados por `ferramentas/gerar_docs.py` em `docs/dados/resumo_catalogo.json`.

## 5. Critérios de saída por estado
- **IMPLEMENTADA:** código mesclado e CA cobertos por testes escritos.
- **TESTADA:** todos os TF-xxx passando em todas as plataformas suportadas.
- **VALIDADA:** revisão funcional contra o requisito (não contra o original), desempenho dentro da meta, documentação de usuário publicada.

## 6. Padrão rigoroso (obrigatório para concluir qualquer parte)
Exigência do responsável (2026-11-17). Resumo em `CLAUDE.md` §7.

| Camada | Ferramenta | Meta |
|---|---|---|
| Critérios de aceitação | testes `caN_*` | 100% dos CA cobertos |
| Oráculo independente | mpmath (desenvolvimento), soluções analíticas | tolerância documentada por função |
| Propriedades | proptest | ≥ 256 casos por propriedade; invariantes de cada função pública |
| Bordas | testes dedicados | lista obrigatória do CLAUDE.md §7.4 |
| Erros | testes de `id()` | 100% das variantes de erro |
| Cobertura | cargo-llvm-cov | ≥ 90% de linhas por módulo concluído |
| Mutação | cargo-mutants | 0 mutantes sobreviventes sem justificativa |
| Regressão | teste por bug | 100% dos bugs corrigidos |
| Desempenho | benchmark / `#[ignore]` | medição registrada para cada CA de desempenho |
| Plataformas | CI | Linux, Windows, macOS |


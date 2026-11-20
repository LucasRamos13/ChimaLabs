# Plano de Desenvolvimento

> **Plano aprovado em 2026-11-17 (DT-15).** Fase 1 em andamento; nenhuma funcionalidade implementada ainda.
> **Prazos:** não há estimativa em calendário. Ela depende do tamanho da equipe e da resposta às questões jurídicas Q2/Q3, e nenhum desses dados está definido. A complexidade relativa vem da contagem de funcionalidades por complexidade (B/M/A/MA) em `docs/dados/funcionalidades.json`. Para referência de escala, sem qualquer promessa: projetos comparáveis de computação numérica levaram anos-pessoa só para atingir compatibilidade parcial de linguagem.

Ordem geral: **2 → 3 → (4 ∥ 5 ∥ 6) → 7 → 8 (contínua) → 9 (contínua) → 10.** Os testes (Fase 8) e a documentação (Fase 9) acompanham todas as fases, e a Fase 8 é também uma fase final de consolidação.

## Portões que bloqueiam o início
- **G0:** aprovar este planejamento, o nome (Q6), a licença do projeto (P-12) e as decisões DT-01..DT-14.
- **G1:** parecer jurídico sobre Q2 (compatibilidade de linguagem). Bloqueia o desenho final de `chimalab-lang`, mas não bloqueia `chimalab-math` e `chimalab-data`.
- **G2:** parecer sobre Q3 (formatos). Bloqueia F110 (.mat), F216 (importadores) e F106 (importação).

---

## Fase 1 — Preparação · **EM ANDAMENTO** (iniciada 2026-11-17)
- Feito: workspace com 24 crates esqueleto, `rust-toolchain.toml`, lints, `deny.toml` (verificado: aprova o workspace e rejeita LGPL), `about.toml`, CI (`.github/workflows/ci.yml`), template de PR com checklist de sala limpa, `gerar_docs.py --verificar`, licenças do projeto, repositório git local.
- Falta: hospedar repositório remoto e ver o CI verde nas 3 plataformas; `docs/linguagem/` (estrutura criada, conteúdo depende de G1).
- **Objetivos:** repositório, CI, política de licenças automatizada, convenções.
- **Entregáveis:** workspace Cargo vazio com crates esqueleto; `deny.toml` com a lista de licenças permitidas; CI multiplataforma (Linux/Windows/macOS); `cargo-about`; templates de PR com checklist de sala limpa; `docs/linguagem/` iniciado.
- **Dependências:** G0. **Riscos:** baixos. **Complexidade:** Baixa.
- **Critérios de conclusão:** CI verde em 3 SOs; uma dependência GPL de teste faz o CI falhar.
- **Funcionalidades:** nenhuma (infraestrutura).

## Fase 2 — Fundação arquitetural
- **Objetivos:** modelo de valores, parser, interpretador mínimo, protocolo kernel↔UI, configuração.
- **Entregáveis:** `chimalab-lang` (gramática documentada), `chimalab-runtime` (arrays N-D, tipos fundamentais, indexação, operadores, workspaces, caminho), REPL de terminal, especificação do protocolo, sistema de configurações (base de F097).
- **Funcionalidades:** F001, F002, F003, F004, F005, F013, F014 (7; complexidades A×6, MA×1).
- **Dependências:** Fase 1, G1 (sintaxe). **Riscos:** semântica de indexação e expansão é o maior risco de compatibilidade. Mitigação: suíte de conformidade escrita antes do código.
- **Critérios de conclusão:** suíte de conformidade das 7 funcionalidades aprovada; REPL utilizável.

## Fase 3 — Funcionalidades centrais
- **Andamento:** iniciada em 2026-11-17 pelo `chimalab-math` (não depende de G1). F040 em desenvolvimento; ver `docs/progresso/F040.md`.
- **Objetivos:** linguagem completa, tipos de alto nível, matemática essencial, gráficos centrais, IDE mínima, núcleo do Fluxo.
- **Funcionalidades (54):** F006–F012, F015–F026, F040–F055, F060–F070, F085–F088, F200–F203.
- **Ordem interna:** linguagem (F006→F012→F016→F007→F008…) → math densa (F040) → dados (F021–F026) → gráficos (F060→F061→…) → desktop + editor + LSP (F085–F088) → Fluxo (F202 motor antes de F200 editor; F201 incremental).
- **Riscos:** volume (54 itens, 14 MA); desempenho do interpretador (F019); renderização multiplataforma.
- **Critérios de conclusão:** todas as funcionalidades da fase em **TESTADA**; benchmarks de referência registrados.

## Fase 4 — Funcionalidades secundárias e ferramentas avançadas
- **Funcionalidades (56):** apps e UI (F075–F079), IDE avançada (F089–F105), documentos vivos (F106–F109), paralelismo (F150–F152), testes e build (F155–F160), Fluxo avançado (F204–F215, F217–F227).
- **Riscos:** editor visual de apps (F076) e de diagramas são grandes projetos de UI; o modo acelerado (F213) depende de compilador C no sistema.
- **Critérios:** idem.

## Fase 5 — Integrações
- **Funcionalidades (31):** E/S (F111–F122, F124), big data (F130–F131), rede (F135–F139), interoperabilidade (F140–F146), pacotes (F165–F168).
- **Dependências:** aprovações D4 (HDF5, NetCDF, libclang); G2 para formatos do original.
- **Riscos:** codecs com patente (F119); serviços externos (F138, F167) exigem infraestrutura própria.

## Fase 6 — Persistência
- **Funcionalidades:** F110 (.mat + formato nativo), F216 (formato Fluxo, autosave), F097 (preferências), F093 (histórico), além de caches e recuperação de sessão.
- **Riscos:** compatibilidade de formatos (G2). **Critérios:** ida-e-volta sem perda; recuperação após encerramento forçado.

## Fase 7 — Segurança
- **Funcionalidades:** F123 (segredos), F174 (conta própria opcional), F175 (telemetria opt-in), endurecimento de F104 (servidor local), F135 (TLS), F115 (XML seguro), F140 (isolamento nativo).
- **Entregáveis:** modelo de ameaças, fuzzing de todos os parsers, revisão de dependências (cargo-audit), política de divulgação de vulnerabilidades.

## Fase 8 — Testes (contínua + consolidação)
Ver `PLANO_DE_TESTES.md`. Consolidação: regressão completa, desempenho, compatibilidade entre plataformas, validação de paridade (todas as funcionalidades em **VALIDADA**).

## Fase 9 — Documentação (contínua + consolidação)
- **Funcionalidades:** F180–F182 (sistema de ajuda, localização pt-BR/en, exemplos próprios).
- **Entregáveis:** referência de linguagem, referência de funções (escrita do zero), guias, especificações de formatos e do protocolo, documentação de blocos Fluxo.

## Fase 10 — Empacotamento e validação
- **Funcionalidades:** F170–F173 (instalador interativo e silencioso, atualizações, desinstalação, decisão "sem ativação").
- **Critérios de conclusão:** instaladores assinados nas 3 plataformas; instalação silenciosa; atualização com reversão; auditoria final de licenças e de proveniência; matriz de paridade sem itens em estados anteriores a **VALIDADA**, exceto os justificados como BLOQUEADA ou NÃO VERIFICÁVEL.

---

## Extensões futuras
Depois da v1 do núcleo + Fluxo, cada área de extensão (processamento de sinais, otimização, estatística e aprendizado de máquina, controle, matemática simbólica, processamento de imagens, computação paralela, máquinas de estado, modelagem física, geração de código) passa por um ciclo próprio: pesquisa em literatura e especificações públicas → catálogo F-xxx → matriz → fases. A prioridade segue as dependências entre as áreas.

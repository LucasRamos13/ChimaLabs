# Arquitetura do ChimaLab

> Arquitetura **independente**, desenhada a partir de `CATALOGO_FUNCIONALIDADES.md`. Ela não deriva da organização interna do original. As decisões, alternativas e trade-offs estão em `DECISOES_TECNICAS.md` (DT-xx).

## 1. Visão geral

```
┌──────────────────────────── ChimaLab Desktop (Tauri + webview do SO) ────────────────────────────┐
│  Frontend TypeScript: painéis, editor (CodeMirror 6), documentos vivos, editor de apps,        │
│  editor de diagramas Fluxo, navegador de ajuda, visualizador de figuras                        │
└───────────────▲───────────────────────────────────────────────▲────────────────────────────────┘
                │ IPC (protocolo ChimaLab: mensagens tipadas,      │ fluxo de quadros/cena gráfica
                │ JSON-RPC + canais binários)                    │
┌───────────────┴───────────────── chimalab-kernel (processo próprio) ─────────────────────────────┐
│ chimalab-lang  →  chimalab-runtime  →  chimalab-jit (opcional)                                       │
│     │              │  ├─ chimalab-data (tabelas, datas, categóricos, strings, dicionários)       │
│     │              │  ├─ chimalab-math (álgebra linear, esparsas, FFT, EDO, ...)                 │
│     │              │  ├─ chimalab-graphics-core (grafo de cena) ──► chimalab-render (wgpu/tiny-skia)│
│     │              │  ├─ chimalab-io / chimalab-net / chimalab-ext                                   │
│     │              │  └─ chimalab-test, chimalab-pkg                                               │
│ chimalab-lsp (análise, completar, formatação, DAP do depurador)                                  │
│ fluxo-model → fluxo-compile → fluxo-engine (+ fluxo-blocks, fluxo-codegen, fluxo-fmi)          │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
  CLI: `chimalab` (REPL de terminal, -batch, servidor headless)   Instalador/atualizador: chimalab-install
```

Princípios:
1. O **kernel fica separado da UI**. Travar o código do usuário nunca trava a interface (RNF-05). Isso também habilita uso headless, remoto e no navegador (F100, F104).
2. Há uma **especificação de linguagem própria** em `docs/linguagem/` (guia rápido, léxico, expressões, instruções, valores, console e funções embutidas, todos com testes de conformidade que leem o próprio documento). A semântica é definida por testes de conformidade escritos pelo projeto.
3. **Bibliotecas puras e testáveis**: `chimalab-math` e `chimalab-data` não dependem do runtime.
4. **Formatos abertos e versionados** (RNF-12).
5. **Extensibilidade** por pacotes (F165/F166) e por uma API nativa estável (F140).

## 2. Módulos (workspace Cargo + pacotes npm)

| Módulo | Responsabilidade | Funcionalidades |
|---|---|---|
| `chimalab-lang` | Léxico, parser, AST, análise semântica, resolução de nomes | F001–F002, F006, F008, F014–F016 |
| `chimalab-runtime` | Modelo de valores (arrays N-D copy-on-write, tipos), interpretador, workspaces, despacho, OOP, exceções, eventos, timers, laço de eventos, interrupção | F001–F020, F150–F152 |
| `chimalab-jit` | Compilação JIT de funções quentes (Cranelift) | F019 |
| `chimalab-data` | datetime, duration, categorical, table/timetable, string, dictionary, missing | F021–F026 |
| `chimalab-math` | Álgebra densa/esparsa, funções especiais, estatística, interpolação, FFT, geometria, quadratura, EDO/DAE/BVP/DDE, otimização, grafos, RNG | F040–F055 |
| `chimalab-graphics-core` | Grafo de objetos gráficos, propriedades, padrões, callbacks | F060–F068, F070 |
| `chimalab-render` | Renderização GPU (wgpu) e CPU (tiny-skia); exportação PNG/PDF/SVG/EPS | F062, F069 |
| `chimalab-ui` | Componentes de app (modelo no kernel, visualização no frontend) | F075–F079 |
| `chimalab-io` | Formatos de arquivo e sistema de arquivos | F110–F124, F130–F131 |
| `chimalab-net` | HTTP, FTP/SFTP, sockets, serial, BLE | F135–F139 |
| `chimalab-ext` | API nativa C (ABI estável), motor embutível, Python, Java, .NET/COM, FFI C/C++ | F140–F146 |
| `chimalab-test` | Framework de testes, mocks, cobertura, ferramenta de build | F155–F160 |
| `chimalab-pkg` | Pacotes, repositórios, empacotamento de apps e extensões | F079, F165–F168 |
| `chimalab-lsp` | Servidor de linguagem + adaptador de depuração | F086–F088, F099 |
| `chimalab-notebook` | Documentos vivos: formato, execução por seção, exportações | F106–F109 |
| `chimalab-desktop` (TS) | Shell da UI, painéis, configurações, VCS, comparações, busca | F085–F105 |
| `chimalab-help` | Ajuda, índice de busca, localização | F180–F182 |
| `chimalab-install` | Instalador, atualizador e desinstalador assinados | F170–F175 |
| `fluxo-model` | Modelo de diagramas, formato de arquivo, bibliotecas, máscaras, variantes, API | F200, F203–F208, F216–F218, F223–F226 |
| `fluxo-compile` | Propagação de tipos/dimensões/tempos, ordenação, laços algébricos | F202–F203 |
| `fluxo-engine` | Execução de simulação, solvers, cruzamento por zero, multitaxa, registro | F202, F210–F212, F214, F219–F220, F222, F227 |
| `fluxo-blocks` | Biblioteca de blocos (cada bloco especificado em `docs/blocos/`) | F201, F209 |
| `fluxo-codegen` | Geração de C para os modos acelerados | F213 |
| `fluxo-fmi` | Importação e exportação de FMU | F221 |
| `fluxo-editor` (TS) | Canvas de diagramas, roteamento, inspetor | F200, F206, F210 (UI) |

## 3. Modelo de dados central (runtime)

- `Value`: enum de tipo + `Array<T>` N-D (forma `SmallVec<usize>`, buffer em `Arc` com copy-on-write, layout column-major). Variantes: numéricos (f64, f32, i8…u64, complexos), lógico, char (UTF-16 para compatibilidade semântica de indexação; decisão em DT-06), string, cell, struct (campos ordenados + array de registros), objeto (valor/handle), handle de função, tipos de `chimalab-data`.
- Arrays esparsos: CSC com `f64`/`c64`/lógico.
- Objetos de referência: `Rc`-like com coleta de ciclos (DT-07).
- Workspaces: pilha de quadros com tabela de símbolos; globais e persistentes separados.

## 4. Comunicação interna
- **Kernel ↔ UI**: protocolo próprio versionado sobre stdio/socket local com token (JSON-RPC 2.0 para controle; canais binários para arrays, imagens e quadros gráficos). Documentar em `docs/protocolo/`.
- **Gráficos**: o kernel mantém o grafo de cena e envia *diffs*. A UI renderiza via WebGPU/Canvas. A exportação headless usa `chimalab-render` no próprio kernel.
- **Eventos**: barramento de eventos no runtime (callbacks de UI, timers, futures), executado no laço principal do kernel entre instruções (pontos seguros).
- **Segundo plano**: pool de threads (rayon) para operações numéricas. Pool de tarefas do usuário com futures (F151).

## 5. Persistência
| Dado | Formato |
|---|---|
| Código | Texto UTF-8 (`.m`, compatibilidade sujeita a Q2; extensão própria `.chima` como alternativa) |
| Dados de sessão | Formato nativo próprio `.chimad` (contêiner HDF5 ou próprio; DT-11) + leitor/escritor `.mat` v5/v7.3 (sujeito a Q3) |
| Documentos vivos | `.chimanb` = ZIP {manifest.json, cells.json, outputs/} |
| Apps | `.chimaapp` = ZIP {layout.json, code.m, resources/} |
| Modelos Fluxo | `.fluxo` = ZIP {model.json, blocks/, data/, preview.png} + formato de texto para diff |
| Projetos | `chimalab-project.toml` + `.chimalab/` |
| Pacotes | `.chimapkg` = ZIP assinado {chimalab-package.toml, src/, docs/} |
| Preferências | TOML por nível (fábrica/instalação/usuário) + sessão em memória |
| Caches | SQLite? **Não**: `redb` (MIT/Apache-2.0) ou arquivos; decisão adiada |

## 6. Segurança
- Parsers com fuzzing contínuo. XML sem DTD externo. Limites de tamanho e profundidade.
- TLS verificado por padrão (rustls). Proxy e certificados configuráveis.
- O servidor local do kernel aceita apenas conexões locais, autenticadas por token aleatório por sessão.
- Extensões nativas podem rodar em processo isolado (RNF-06).
- Segredos ficam no cofre do SO (F123).
- Atualizações e pacotes são assinados com Ed25519, com chave pública embutida e rotação documentada.
- Telemetria opt-in (F175).
- Sem licenciamento ou ativação (F173).

## 7. Empacotamento e atualização
Instaladores nativos por plataforma: MSI/MSIX no Windows, `.dmg` assinado e notarizado no macOS, AppImage/`.deb`/`.rpm`/Flatpak no Linux. Instalação silenciosa com arquivo de respostas TOML (F170). Atualizador com canal estável/beta, verificação de assinatura e reversão (F171). Componentes opcionais (documentação offline, Fluxo, pacotes de idioma) ficam em pacotes `.chimapkg`.

## 8. Testabilidade
- Testes de conformidade da linguagem em arquivos `.m` próprios com saída esperada (`tests/conformidade/`).
- Oráculos numéricos: soluções analíticas, bibliotecas de referência independentes em Python/SciPy e conjuntos publicados (NIST StRD, problemas de referência de EDO). **Nunca** o produto original.
- UI: Playwright sobre o frontend + modo de teste do kernel.
- Simulação: modelos de referência com solução analítica e FMI Cross-Check.

## 9. Cobertura da arquitetura
Todas as 162 funcionalidades do catálogo têm módulo atribuído (coluna "Módulo" da matriz). As extensões futuras (ver `PLANO_DE_DESENVOLVIMENTO.md`) ainda não têm arquitetura definida. Elas serão incorporadas como pacotes sobre `chimalab-runtime` e `fluxo-*`.

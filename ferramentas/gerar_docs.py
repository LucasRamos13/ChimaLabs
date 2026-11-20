#!/usr/bin/env python3
"""Gera documentos derivados do catálogo funcional.

Fonte única de verdade: docs/dados/funcionalidades.json
Gera:
  docs/CATALOGO_FUNCIONALIDADES.md
  docs/MATRIZ_PARIDADE_FUNCIONAL.md
  docs/REQUISITOS.md
  docs/TESTES_POR_FUNCIONALIDADE.md

Uso: python3 -I ferramentas/gerar_docs.py   (a partir da raiz do projeto)
Para mudar status de uma funcionalidade, edite "status_forcado" no JSON e rode novamente.
"""
import json, re, os, sys, collections

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
D = os.path.join(RAIZ, "docs")
dados = json.load(open(os.path.join(D, "dados", "funcionalidades.json"), encoding="utf-8"))
AREAS, JUR, FS = dados["areas"], dados["juridico"], dados["funcionalidades"]

CX = {"B": "Baixa", "M": "Média", "A": "Alta", "MA": "Muito alta"}
ESTADOS = ["NÃO INVESTIGADA", "INVESTIGADA PARCIALMENTE", "DOCUMENTADA", "PLANEJADA", "EM DESENVOLVIMENTO",
           "IMPLEMENTADA", "TESTADA", "VALIDADA", "BLOQUEADA", "NÃO VERIFICÁVEL"]
FASE_AREA = {"LNG": 3, "DAT": 3, "MAT": 3, "GRF": 3, "APP": 4, "IDE": 4, "DOC": 4, "IO": 5, "BIG": 5,
             "NET": 5, "EXT": 5, "PAR": 4, "TST": 4, "PKG": 5, "INS": 10, "LOC": 9, "SIM": 4}
FASE_OVR = {"F001": 2, "F002": 2, "F003": 2, "F004": 2, "F005": 2, "F013": 2, "F014": 2,
            "F085": 3, "F086": 3, "F087": 3, "F088": 3, "F097": 6, "F093": 6, "F110": 6, "F216": 6,
            "F200": 3, "F201": 3, "F202": 3, "F203": 3, "F123": 7, "F175": 7, "F173": 10, "F174": 7}


def status(f):
    if f.get("status_forcado"):
        return f["status_forcado"]
    return "PLANEJADA" if f["aceitacao"] and (f["estrategia"] or AREAS[f["area"]]["deps"]) else "DOCUMENTADA"


def confianca(f):
    return f.get("confianca_forcada") or "Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes"


def estrategia(f):
    a = AREAS[f["area"]]
    base = f["estrategia"] or "Implementação própria a partir deste requisito, usando algoritmos/padrões publicados."
    return f"{base} Módulo: `{a['modulo']}`."


def tipos_teste(f):
    t = ["Unitário", "Integração"]
    txt = " ".join(f["aceitacao"]).lower()
    if f["area"] in ("IDE", "APP", "DOC", "SIM", "GRF", "INS", "PKG"):
        t.append("Ponta a ponta")
    if re.search(r"fps|speedup|< ?\d+ ?(s|ms)|tempo|desempenho|o\(1\)|passagens", txt):
        t.append("Desempenho")
    if f["area"] in ("NET", "EXT", "INS", "PKG") or f["id"] in ("F115", "F123", "F110", "F175", "F104", "F105"):
        t.append("Segurança")
    if f["juridico"] == "fmt" or f["area"] == "IO":
        t.append("Compatibilidade/E-S")
    t.append("Regressão")
    return t


def fase(f):
    return f.get("fase") or FASE_OVR.get(f["id"]) or FASE_AREA[f["area"]]


VERIFICAR = "--verificar" in sys.argv
DIVERGENTES = []


def escrever(caminho, conteudo):
    """Grava o arquivo; em modo --verificar apenas compara com o conteúdo existente."""
    if VERIFICAR:
        atual = open(caminho, encoding="utf-8").read() if os.path.exists(caminho) else None
        if atual != conteudo:
            DIVERGENTES.append(os.path.relpath(caminho, RAIZ))
        return
    open(caminho, "w", encoding="utf-8").write(conteudo)


cab = "<!-- GERADO por ferramentas/gerar_docs.py. Não edite à mão: edite docs/dados/funcionalidades.json. -->\n\n"
# ---------------- CATÁLOGO ----------------
L = [cab, "# Catálogo de Funcionalidades\n\n",
     "Catálogo funcional em **sala limpa**. Comportamentos descrevem necessidades e resultados observáveis, "
     "redigidos de forma independente a partir de conhecimento público da área. Nenhum código, binário ou texto "
     "de documentação de produto proprietário foi usado.\n\n"]
por_area = collections.OrderedDict((k, []) for k in AREAS)
for f in FS:
    por_area[f["area"]].append(f)
L.append("## Índice por área\n\n| Área | Nome | Itens |\n|---|---|---|\n")
for k, v in por_area.items():
    L.append(f"| {k} | {AREAS[k]['nome']} | {len(v)} |\n")
L.append(f"| **Total** | | **{len(FS)}** |\n\n")
for k, v in por_area.items():
    a = AREAS[k]
    L.append(f"## {k} — {a['nome']}\n\n")
    for f in v:
        deps = ", ".join(f["dependencias"]) or "—"
        L.append(f"### {f['id']} — {f['nome']}\n\n")
        rows = [
            ("1. Identificador", f["id"]),
            ("2. Nome conceitual", f["nome"]),
            ("3. Descrição", f["descricao"]),
            ("4. Necessidade atendida", f["necessidade"]),
            ("5. Entradas", f["entradas"]),
            ("6. Saídas", f["saidas"]),
            ("7. Fluxo de utilização", a["fluxo"]),
            ("8. Comportamento esperado", f["comportamento"]),
            ("9. Regras de negócio observáveis", f["regras"]),
            ("10. Dependências", deps),
            ("11. Casos especiais", f["casos_especiais"]),
            ("12. Possíveis erros", f["erros"]),
            ("13. Critérios de aceitação", "<br>".join(f"CA{i+1}: {c}" for i, c in enumerate(f["aceitacao"]))),
            ("14. Serviço externo", "Depende de serviço externo." if f["servico_externo"] else "Não."),
            ("15. Grau de confiança", f"{confianca(f)}."),
            ("16. Complexidade estimada", CX[f["complexidade"]]),
            ("17. Restrições técnicas", f["restricoes_tecnicas"] or "—"),
            ("18. Restrições jurídicas", JUR[f["juridico"]]),
        ]
        L.append("| Campo | Conteúdo |\n|---|---|\n")
        for r in rows:
            L.append(f"| {r[0]} | {str(r[1]).replace('|', '\\|')} |\n")
        L.append(f"\n*Status:* **{status(f)}**\n\n")

escrever(os.path.join(D, "CATALOGO_FUNCIONALIDADES.md"), "".join(L))

# ---------------- MATRIZ ----------------
cont = collections.Counter(status(f) for f in FS)
M = [cab, "# Matriz de Paridade Funcional\n\n",
     "Relaciona cada funcionalidade identificada (ID da análise) ao requisito independente, ao módulo e à estratégia do novo projeto. "
     "**Nenhuma linha pode ser removida.** Mudanças de estado devem ser registradas em `docs/dados/funcionalidades.json`.\n\n",
     "## Estados\n\n" + "\n".join(f"- `{e}`" for e in ESTADOS) + "\n\n",
     "## Resumo\n\n| Estado | Funcionalidades |\n|---|---|\n"]
for e in ESTADOS:
    M.append(f"| {e} | {cont.get(e, 0)} |\n")
M.append(f"| **Total** | **{len(FS)}** |\n\n")
M.append("## Matriz\n\n| ID | Necessidade funcional | Comportamento esperado | Requisito | Módulo | Estratégia | Dependências permitidas | Critérios | Testes | Complex. | Viabilidade | Situação jurídica | Fase | Status |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n")
VIAB = {"B": "Alta", "M": "Alta", "A": "Média", "MA": "Média/Baixa (esforço elevado)"}
for f in FS:
    a = AREAS[f["area"]]
    jur = {"default": "Pendente (compatibilidade de linguagem)", "fmt": "Pendente (formato de terceiros)", "svc": "Serviço externo — substituir", "pat": "Pendente (patente/expressão visual)", "none": "Sem restrição específica"}[f["juridico"]]
    viab = "Bloqueada/indeterminada" if status(f) in ("BLOQUEADA", "NÃO VERIFICÁVEL") else VIAB[f["complexidade"]]
    esc = lambda s: s.replace("|", "\\|")
    M.append(f"| {f['id']} | {esc(f['necessidade'])} | {esc(f['comportamento'][:160])}{'…' if len(f['comportamento'])>160 else ''} | RF-{f['id'][1:]} | `{a['modulo']}` | {esc(estrategia(f)[:140])}{'…' if len(estrategia(f))>140 else ''} | {esc(a['deps'][:90])}… | {len(f['aceitacao'])} CA | TF-{f['id'][1:]} ({', '.join(tipos_teste(f))}) | {CX[f['complexidade']]} | {viab} | {jur} | {fase(f)} | **{status(f)}** |\n")
escrever(os.path.join(D, "MATRIZ_PARIDADE_FUNCIONAL.md"), "".join(M))

# ---------------- REQUISITOS ----------------
R = [cab, "# Requisitos\n\n",
     "Requisitos redigidos de forma independente a partir das necessidades funcionais. Cada RF-xxx corresponde à funcionalidade Fxxx.\n\n",
     "## Requisitos não funcionais (RNF)\n\n",
     "| ID | Requisito | Verificação |\n|---|---|---|\n",
     "| RNF-01 | Plataformas: Linux x86-64 e aarch64, Windows x64, macOS arm64 e x86-64. | CI de matriz por plataforma |\n",
     "| RNF-02 | Independência: nenhum código, binário, texto, ícone ou recurso de produto proprietário no repositório. | Auditoria de proveniência a cada release (ver CLAUDE.md) |\n",
     "| RNF-03 | Licenças: só dependências aprovadas em LICENCAS_E_RESTRICOES.txt; SBOM gerado a cada build. | cargo-deny / license-checker no CI |\n",
     "| RNF-04 | Precisão numérica IEEE 754; desvios documentados por função. | Suítes de referência numérica |\n",
     "| RNF-05 | Responsividade: UI não bloqueia durante execução de código (runtime em processo separado). | Teste E2E com laço infinito + Ctrl+C |\n",
     "| RNF-06 | Robustez: falha em extensão nativa não corrompe a sessão (modo isolado disponível). | Teste de injeção de falha |\n",
     "| RNF-07 | Privacidade: zero tráfego de rede sem ação explícita ou consentimento; telemetria opt-in. | Teste de rede em sandbox |\n",
     "| RNF-08 | Segurança: parsers de arquivos resistentes a entradas malformadas (fuzzing); XML sem entidades externas; TLS verificado por padrão. | cargo-fuzz em todos os parsers |\n",
     "| RNF-09 | Acessibilidade: UI navegável por teclado, compatível com leitores de tela, contraste WCAG 2.2 AA. | Auditoria axe-core + manual |\n",
     "| RNF-10 | Localização: pt-BR e en na v1; textos externalizados. | Teste de chaves ausentes |\n",
     "| RNF-11 | Desempenho: metas por funcionalidade nos critérios de aceitação; benchmarks contínuos sem regressão > 5%. | Benchmarks no CI |\n",
     "| RNF-12 | Formatos próprios abertos, versionados e documentados. | Especificações em docs/formatos/ |\n",
     "| RNF-13 | Testabilidade: cobertura de linhas ≥ 85% no núcleo (lang/runtime/math). | Relatório de cobertura |\n",
     "| RNF-14 | Modo sem interface (headless) para todos os recursos não visuais. | Execução da suíte em CI sem display |\n\n",
     "## Requisitos funcionais (RF)\n\n"]
for f in FS:
    R.append(f"### RF-{f['id'][1:]} — {f['nome']} (origem: {f['id']}, status {status(f)})\n\n")
    R.append(f"O sistema deve oferecer: {f['descricao']}\n\n")
    R.append(f"- **Entradas:** {f['entradas']}\n- **Saídas:** {f['saidas']}\n- **Comportamento:** {f['comportamento']}\n- **Regras:** {f['regras']}\n- **Erros a tratar:** {f['erros']}\n- **Casos especiais:** {f['casos_especiais']}\n- **Critérios de aceitação:**\n")
    for i, c in enumerate(f["aceitacao"]):
        R.append(f"  - CA{i+1}: {c}\n")
    R.append("\n")
escrever(os.path.join(D, "REQUISITOS.md"), "".join(R))

# ---------------- TESTES ----------------
T = [cab, "# Testes por Funcionalidade\n\n",
     "Casos derivados dos critérios de aceitação. Complementa `PLANO_DE_TESTES.md`. Nenhum caso usa saída de produto proprietário como oráculo. "
     "Os oráculos são soluções analíticas, implementações de referência independentes ou especificações públicas.\n\n"]
for f in FS:
    T.append(f"## TF-{f['id'][1:]} — {f['nome']}\n\n")
    T.append(f"- **Tipos:** {', '.join(tipos_teste(f))}\n- **Pré-condições:** runtime iniciado; dados de teste em `tests/dados/{f['id']}/` (a criar).\n"
             f"- **Entradas:** {f['entradas']}\n- **Casos de erro:** {f['erros']}\n- **Casos extremos:** {f['casos_especiais']}\n\n")
    T.append("| Caso | Procedimento / resultado esperado | Critério |\n|---|---|---|\n")
    for i, c in enumerate(f["aceitacao"]):
        T.append(f"| TF-{f['id'][1:]}.{i+1} | {c.replace('|', '\\|')} | CA{i+1} |\n")
    T.append(f"| TF-{f['id'][1:]}.E | Cada erro listado produz identificador de erro estável e mensagem própria | Erros |\n\n")
escrever(os.path.join(D, "TESTES_POR_FUNCIONALIDADE.md"), "".join(T))

# ---------------- resumo para outros documentos ----------------
res = dict(total=len(FS), por_status=dict(cont),
           por_area={k: len(v) for k, v in por_area.items()},
           servico_externo=[f["id"] for f in FS if f["servico_externo"]],
           criterios=sum(len(f["aceitacao"]) for f in FS))
escrever(os.path.join(D, "dados", "resumo_catalogo.json"), json.dumps(res, indent=1, ensure_ascii=False) + "\n")
if VERIFICAR:
    if DIVERGENTES:
        print("Documentos desatualizados (rode python3 -I ferramentas/gerar_docs.py):", *DIVERGENTES, sep="\n  ")
        sys.exit(1)
    print("Documentos gerados estão sincronizados.")
else:
    print(json.dumps(res, ensure_ascii=False))

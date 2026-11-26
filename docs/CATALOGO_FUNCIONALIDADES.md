<!-- GERADO por ferramentas/gerar_docs.py. Não edite à mão: edite docs/dados/funcionalidades.json. -->

# Catálogo de Funcionalidades

Catálogo funcional em **sala limpa**. Comportamentos descrevem necessidades e resultados observáveis, redigidos de forma independente a partir de conhecimento público da área. Nenhum código, binário ou texto de documentação de produto proprietário foi usado.

## Índice por área

| Área | Nome | Itens |
|---|---|---|
| LNG | Linguagem e runtime | 20 |
| DAT | Tipos de dados de alto nível | 6 |
| MAT | Matemática e computação numérica | 16 |
| GRF | Gráficos e visualização | 11 |
| APP | Construção de aplicativos e interfaces | 5 |
| IDE | Ambiente de desenvolvimento | 21 |
| DOC | Documentos executáveis e publicação | 4 |
| IO | Entrada/saída e formatos de arquivo | 15 |
| BIG | Dados grandes e processamento fora da memória | 2 |
| NET | Rede, dispositivos e serviços | 5 |
| EXT | Interoperabilidade com outras linguagens | 7 |
| PAR | Concorrência e desempenho | 3 |
| TST | Testes, qualidade e automação de build | 6 |
| PKG | Pacotes, extensões e distribuição | 4 |
| INS | Instalação, atualização, conta e telemetria | 6 |
| LOC | Ajuda, documentação e localização | 3 |
| SIM | Modelagem e simulação por diagramas de blocos | 28 |
| **Total** | | **162** |

## LNG — Linguagem e runtime

### F001 — Interpretador interativo (janela de comandos)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F001 |
| 2. Nome conceitual | Interpretador interativo (janela de comandos) |
| 3. Descrição | Avaliação interativa de expressões e comandos com exibição automática de resultados. |
| 4. Necessidade atendida | Exploração rápida de dados e cálculos sem compilar. |
| 5. Entradas | Texto de comando; teclas de controle (histórico, interrupção). |
| 6. Saídas | Valores exibidos; variável implícita de resultado; mensagens de erro/aviso. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Cada linha é analisada e executada; ponto-e-vírgula suprime exibição; resultado sem atribuição vai para variável implícita 'ans'. |
| 9. Regras de negócio observáveis | Exibição respeita formato numérico ativo; vírgula separa comandos exibindo; Ctrl+C interrompe execução. |
| 10. Dependências | F002, F018 |
| 11. Casos especiais | Comandos multilinha (blocos if/for abertos); sintaxe de comando ('hold on'); entrada muito longa. |
| 12. Possíveis erros | Erro de sintaxe com posição; identificador indefinido; interrupção pelo usuário. |
| 13. Critérios de aceitação | CA1: Expressão '1+1' exibe 2 e define ans<br>CA2: ';' suprime exibição<br>CA3: Bloco multilinha só executa após 'end'<br>CA4: Ctrl+C interrompe laço infinito em <1s sem perder o workspace |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **EM DESENVOLVIMENTO**

### F002 — Scripts e funções em arquivo

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F002 |
| 2. Nome conceitual | Scripts e funções em arquivo |
| 3. Descrição | Arquivos de código-fonte contendo scripts ou funções (principal, locais, aninhadas, privadas). |
| 4. Necessidade atendida | Reutilização e organização de código. |
| 5. Entradas | Arquivos de código (.m equivalente no formato compatível e extensão própria). |
| 6. Saídas | Execução; valores de retorno múltiplos. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Funções com múltiplas entradas/saídas, nargin/nargout, varargin/varargout, funções locais, aninhadas com compartilhamento de escopo, scripts operando no workspace do chamador. |
| 9. Regras de negócio observáveis | Nome do arquivo define a função principal; funções locais visíveis apenas no arquivo; precedência de resolução (variável > aninhada > local > privada > classe > caminho). |
| 10. Dependências | F001, F019 |
| 11. Casos especiais | Recursão profunda; arquivo modificado em disco durante a sessão (recarregamento); script com função local no final. |
| 12. Possíveis erros | Muitos/poucos argumentos; saída não atribuída; recursão máxima excedida. |
| 13. Critérios de aceitação | CA1: Função com 3 saídas retorna somente as pedidas<br>CA2: Edição do arquivo em disco é refletida na chamada seguinte<br>CA3: Limite de recursão gera erro identificável e configurável |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **EM DESENVOLVIMENTO**

### F003 — Tipos numéricos e lógicos fundamentais

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F003 |
| 2. Nome conceitual | Tipos numéricos e lógicos fundamentais |
| 3. Descrição | Ponto flutuante dupla/simples, inteiros com/sem sinal de 8–64 bits, lógico, caractere e números complexos. |
| 4. Necessidade atendida | Precisão e economia de memória adequadas ao problema. |
| 5. Entradas | Literais, conversões de tipo. |
| 6. Saídas | Arrays tipados. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Aritmética mista com regras de tipo dominante; inteiros saturam e arredondam; complexo automático em operações que exigem (sqrt(-1)). |
| 9. Regras de negócio observáveis | Inteiro combinado com double resulta inteiro; operações entre inteiros de classes diferentes são erro; NaN/Inf seguem IEEE 754. |
| 10. Dependências | — |
| 11. Casos especiais | -0, NaN payload, intmin/intmax, eps, realmin denormais, conversão de NaN para inteiro (0). |
| 12. Possíveis erros | Combinação inválida de classes; overflow tratado por saturação (sem erro). |
| 13. Critérios de aceitação | CA1: int8(127)+1 == 127<br>CA2: uint8(3.5) == 4<br>CA3: class(int16(1)*2.7) == 'int16'<br>CA4: sqrt(-4) == 2i |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F004 — Arrays N-dimensionais e indexação

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F004 |
| 2. Nome conceitual | Arrays N-dimensionais e indexação |
| 3. Descrição | Arrays de qualquer dimensão com indexação por subscrito, linear, lógica, 'end', dois-pontos, exclusão e crescimento automático. |
| 4. Necessidade atendida | Manipulação vetorizada de dados. |
| 5. Entradas | Arrays, índices. |
| 6. Saídas | Subarrays, arrays modificados. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Ordem column-major; atribuição fora dos limites expande com zeros; atribuição de [] remove elementos; dimensões singleton finais ignoradas. |
| 9. Regras de negócio observáveis | Índices começam em 1; índice lógico deve ter tamanho compatível; 'end' resolve para o tamanho da dimensão indexada. |
| 10. Dependências | F003 |
| 11. Casos especiais | Arrays vazios com dimensões (0x3); indexação com mais dimensões que o array; colapso de dimensões finais. |
| 12. Possíveis erros | Índice fora dos limites na leitura; índice não inteiro/negativo; dimensões incompatíveis na atribuição. |
| 13. Critérios de aceitação | CA1: A(end+1)=x cresce o vetor<br>CA2: A(:) retorna coluna<br>CA3: Remover linha com A(2,:)=[]<br>CA4: size(zeros(0,3)) == [0 3] |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F005 — Operadores matriciais, elemento a elemento e expansão implícita

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F005 |
| 2. Nome conceitual | Operadores matriciais, elemento a elemento e expansão implícita |
| 3. Descrição | Operadores aritméticos, relacionais, lógicos (com curto-circuito), matriciais (*, /, \, ^) e elemento a elemento (.*, ./, .^) com expansão implícita de dimensões singleton. |
| 4. Necessidade atendida | Expressão concisa de álgebra linear e cálculo vetorizado. |
| 5. Entradas | Operandos arrays. |
| 6. Saídas | Arrays resultantes. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Expansão implícita compatível; '\' resolve sistema linear escolhendo método por estrutura da matriz; transposta e transposta conjugada. |
| 9. Regras de negócio observáveis | Precedência de operadores documentada; && e \|\| exigem escalares lógicos. |
| 10. Dependências | F004, F040 |
| 11. Casos especiais | Expansão entre 1xN e Mx1; operações com vazio; matriz singular em '\' gera aviso. |
| 12. Possíveis erros | Dimensões incompatíveis; operandos não escalares em &&. |
| 13. Critérios de aceitação | CA1: [1 2 3]+[10;20] produz 2x3<br>CA2: A\b resolve sistema com resíduo < 1e-12 em matriz bem condicionada<br>CA3: Aviso emitido para matriz singular |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F006 — Controle de fluxo

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F006 |
| 2. Nome conceitual | Controle de fluxo |
| 3. Descrição | if/elseif/else, switch/case (inclusive cell de casos e strings), for (sobre colunas), while, break, continue, return, try/catch. |
| 4. Necessidade atendida | Lógica de programa. |
| 5. Entradas | Código. |
| 6. Saídas | Execução condicionada. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | for itera sobre colunas da expressão; switch compara números e textos; try/catch captura exceção em objeto. |
| 9. Regras de negócio observáveis | Condição de if com array é verdadeira somente se todos os elementos forem não-nulos e não-vazia. |
| 10. Dependências | F001 |
| 11. Casos especiais | for sobre matriz vazia; switch com cell de alternativas; modificação da variável do laço no corpo. |
| 12. Possíveis erros | Uso de break fora de laço; catch sem identificador. |
| 13. Critérios de aceitação | CA1: for k=[1 2;3 4] itera 2 vezes com colunas<br>CA2: if [] é falso<br>CA3: switch 'abc' casa com {'x','abc'} |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F007 — Programação orientada a objetos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F007 |
| 2. Nome conceitual | Programação orientada a objetos |
| 3. Descrição | Definição de classes com propriedades, métodos, eventos, enumerações, herança múltipla, classes de valor e de referência, atributos de acesso, validação de propriedades, métodos estáticos, sobrecarga de operadores e indexação, getters/setters, metaclasses. |
| 4. Necessidade atendida | Modelagem de domínio e encapsulamento. |
| 5. Entradas | Arquivos de definição de classe; pastas de classe. |
| 6. Saídas | Objetos, metadados de classe. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Semântica de cópia para classes de valor e referência compartilhada para classes de identidade; destrutores em classes de referência; listeners de eventos e propriedades observáveis. |
| 9. Regras de negócio observáveis | Atributos Access/SetAccess/GetAccess/Hidden/Constant/Dependent/Abstract/Sealed; ordem de resolução de métodos; classes enumeradas. |
| 10. Dependências | F002, F017 |
| 11. Casos especiais | Recarregar definição com instâncias existentes; herança diamante; redefinição de indexação (subsref/subsasgn equivalente). |
| 12. Possíveis erros | Acesso negado; violação de validação; método abstrato não implementado. |
| 13. Critérios de aceitação | CA1: Classe de referência compartilha estado entre cópias<br>CA2: Validação de propriedade rejeita tipo inválido com mensagem identificável<br>CA3: Evento notifica listeners em ordem de registro<br>CA4: Metadados da classe consultáveis |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F008 — Validação de argumentos de função

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F008 |
| 2. Nome conceitual | Validação de argumentos de função |
| 3. Descrição | Bloco declarativo de argumentos com tamanho, classe, funções validadoras, valores padrão, argumentos nome-valor e repetidos. |
| 4. Necessidade atendida | Funções robustas com mensagens de erro claras. |
| 5. Entradas | Declaração de argumentos. |
| 6. Saídas | Argumentos convertidos/validados. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Conversão automática quando possível; erros apontam o argumento; suporte a nome-valor com nomes parciais desabilitados/habilitados conforme regra. |
| 9. Regras de negócio observáveis | Validação ocorre antes do corpo; padrões avaliados em ordem. |
| 10. Dependências | F002 |
| 11. Casos especiais | Argumentos nome-valor derivados de propriedades de classe; validadores personalizados. |
| 12. Possíveis erros | Tipo/tamanho incompatível; nome-valor desconhecido. |
| 13. Critérios de aceitação | CA1: Chamada f(x='a') com validador numérico falha antes do corpo<br>CA2: Valor padrão aplicado quando omitido<br>CA3: Mensagem identifica posição do argumento |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F009 — Arrays de células e estruturas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F009 |
| 2. Nome conceitual | Arrays de células e estruturas |
| 3. Descrição | Containers heterogêneos (cell) e registros com campos nomeados (struct), incluindo arrays de structs, campos dinâmicos, listas separadas por vírgula. |
| 4. Necessidade atendida | Agrupar dados heterogêneos. |
| 5. Entradas | Valores quaisquer. |
| 6. Saídas | Containers. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | c{...} extrai conteúdo, c(...) extrai subcélula; s.(nome) acessa campo dinâmico; s(i).campo em arrays de struct produz lista separada por vírgula. |
| 9. Regras de negócio observáveis | Arrays de struct exigem os mesmos campos; ordem de campos preservada. |
| 10. Dependências | F004 |
| 11. Casos especiais | Struct vazio com campos; cell aninhada profunda; deal com listas. |
| 12. Possíveis erros | Campo inexistente; conteúdo de cell com chaves em contexto inválido. |
| 13. Critérios de aceitação | CA1: [s.a] concatena campos de array de struct<br>CA2: fieldnames preserva ordem<br>CA3: c{2:3} expande em múltiplas saídas |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F010 — Texto: arrays de caracteres e strings

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F010 |
| 2. Nome conceitual | Texto: arrays de caracteres e strings |
| 3. Descrição | Tipo char e tipo string com funções de manipulação (concatenação, busca, substituição, divisão, junção, formatação, comparação, conversão numérica), padrões de texto e expressões regulares. |
| 4. Necessidade atendida | Processamento de texto. |
| 5. Entradas | Textos, padrões. |
| 6. Saídas | Textos, índices, tokens. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Strings são escalares de texto com operador +; funções aceitam char, string e cellstr; regexp com tokens nomeados, lookaround, modos. |
| 9. Regras de negócio observáveis | Unicode completo; comparação sensível a maiúsculas por padrão. |
| 10. Dependências | F003 |
| 11. Casos especiais | String ausente (<missing>); texto vazio vs ausente; expressões regulares com retrocesso exponencial. |
| 12. Possíveis erros | Padrão inválido; conversão numérica inválida resulta NaN. |
| 13. Critérios de aceitação | CA1: "a"+"b" == "ab"<br>CA2: regexp com tokens nomeados retorna struct<br>CA3: str2double('x') é NaN<br>CA4: split/join/replace funcionam com arrays de string |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F011 — Formatação e exibição de valores

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F011 |
| 2. Nome conceitual | Formatação e exibição de valores |
| 3. Descrição | Formatos de exibição (curto, longo, engenharia, racional, hexadecimal, espaçamento), sprintf/fprintf/num2str/disp/display, exibição personalizada de objetos. |
| 4. Necessidade atendida | Leitura legível de resultados. |
| 5. Entradas | Valores, especificadores de formato. |
| 6. Saídas | Texto formatado. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Especificadores estilo C reciclados sobre elementos; exibição de matrizes com fator comum de escala. |
| 9. Regras de negócio observáveis | Formato é estado de sessão e preferência. |
| 10. Dependências | F001 |
| 11. Casos especiais | Inteiros grandes, complexos, NaN/Inf, arrays vazios, exibição truncada de arrays grandes. |
| 12. Possíveis erros | Especificador inválido. |
| 13. Critérios de aceitação | CA1: sprintf('%d,',[1 2 3]) == '1,2,3,'<br>CA2: format long mostra 15 dígitos significativos para double<br>CA3: Exibição personalizável por método da classe |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F012 — Tratamento de erros e avisos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F012 |
| 2. Nome conceitual | Tratamento de erros e avisos |
| 3. Descrição | Lançamento e captura de exceções com identificadores, pilha, causas, correções sugeridas; controle de avisos por identificador; erro/aviso com formatação; limpeza garantida (onCleanup). |
| 4. Necessidade atendida | Programas robustos e diagnosticáveis. |
| 5. Entradas | Identificador, mensagem, estado. |
| 6. Saídas | Exceções; avisos exibidos/suprimidos. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Avisos podem ser ligados/desligados/convertidos por id; último erro/aviso consultável; rethrow preserva pilha. |
| 9. Regras de negócio observáveis | Identificadores 'componente:mensagem'. |
| 10. Dependências | F002 |
| 11. Casos especiais | Erro durante limpeza; avisos repetidos; exceções aninhadas (causes). |
| 12. Possíveis erros | Identificador mal formado. |
| 13. Critérios de aceitação | CA1: warning('off',id) suprime apenas aquele aviso<br>CA2: MException tem stack com arquivo/linha<br>CA3: onCleanup executa ao sair do escopo mesmo em erro |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F013 — Workspaces, escopo e avaliação dinâmica

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F013 |
| 2. Nome conceitual | Workspaces, escopo e avaliação dinâmica |
| 3. Descrição | Workspace base e de função, variáveis globais e persistentes, eval/evalin/assignin/feval, exist/which/clear/who/whos, inputname. |
| 4. Necessidade atendida | Controle de estado e metaprogramação. |
| 5. Entradas | Nomes, expressões em texto. |
| 6. Saídas | Valores, informação de variáveis. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Variáveis persistentes por função; globais compartilhadas por declaração; clear remove variáveis/funções/classes do cache. |
| 9. Regras de negócio observáveis | exist retorna códigos de tipo; which resolve caminho e sobreposição. |
| 10. Dependências | F002 |
| 11. Casos especiais | eval com erro e catch; clear dentro de função; variáveis com nomes de funções (shadowing). |
| 12. Possíveis erros | Nome inválido; workspace inexistente. |
| 13. Critérios de aceitação | CA1: Persistente mantém valor entre chamadas<br>CA2: clear all limpa persistentes<br>CA3: whos lista tamanho, classe e bytes |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **EM DESENVOLVIMENTO**

### F014 — Caminho de busca, pacotes e pastas especiais

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F014 |
| 2. Nome conceitual | Caminho de busca, pacotes e pastas especiais |
| 3. Descrição | Caminho de busca de funções, pastas privadas, pastas de classe, namespaces (pastas de pacote), importação de nomes, pastas de recursos, startup/finish de sessão. |
| 4. Necessidade atendida | Organizar bibliotecas e evitar colisões de nomes. |
| 5. Entradas | Diretórios, declarações import. |
| 6. Saídas | Resolução de nomes. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Ordem de precedência configurável; caminho persistível; namespaces aninhados; import de pacote ou função específica. |
| 9. Regras de negócio observáveis | Pasta atual precede caminho; pastas privadas visíveis só ao pai. |
| 10. Dependências | F002 |
| 11. Casos especiais | Duplicatas de nomes (shadowing com aviso); pasta removida em uso. |
| 12. Possíveis erros | Pasta inexistente; nome ambíguo. |
| 13. Critérios de aceitação | CA1: addpath/rmpath/savepath alteram e persistem caminho<br>CA2: Função em pasta privada não visível externamente<br>CA3: import de namespace resolve nome curto |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F015 — Módulos e namespaces modernos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F015 |
| 2. Nome conceitual | Módulos e namespaces modernos |
| 3. Descrição | Agrupamento de código em módulos com descritor, visibilidade e dependências. |
| 4. Necessidade atendida | Encapsulamento em escala de projeto. |
| 5. Entradas | Pastas de módulo com descritor. |
| 6. Saídas | Escopos de visibilidade. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Inferência: módulo expõe API pública e oculta implementação. |
| 9. Regras de negócio observáveis | A investigar em documentação pública. |
| 10. Dependências | F014 |
| 11. Casos especiais | Compatibilidade com caminho tradicional. |
| 12. Possíveis erros | Dependência circular. |
| 13. Critérios de aceitação | CA1: Função interna do módulo inacessível de fora<br>CA2: Descritor inválido gera erro claro |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Hipótese. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

### F016 — Funções anônimas, handles e funções de ordem superior

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F016 |
| 2. Nome conceitual | Funções anônimas, handles e funções de ordem superior |
| 3. Descrição | Handles de função, funções anônimas com captura de valores, cellfun/arrayfun/structfun/func2str/str2func. |
| 4. Necessidade atendida | Programação funcional e callbacks. |
| 5. Entradas | Expressões, handles. |
| 6. Saídas | Handles, resultados agregados. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Anônimas capturam valores no momento da criação; 'UniformOutput' controla agregação; 'ErrorHandler' trata erros por elemento. |
| 9. Regras de negócio observáveis | Saídas não uniformes exigem UniformOutput=false. |
| 10. Dependências | F002 |
| 11. Casos especiais | Handles para funções aninhadas; comparação de handles. |
| 12. Possíveis erros | Saída não escalar com UniformOutput=true. |
| 13. Critérios de aceitação | CA1: Captura por valor: alterar variável depois não muda o handle<br>CA2: arrayfun com saída não escalar exige opção |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F017 — Eventos, listeners e referências fracas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F017 |
| 2. Nome conceitual | Eventos, listeners e referências fracas |
| 3. Descrição | Mecanismo de publicação/assinatura entre objetos, propriedades observáveis, referências fracas. |
| 4. Necessidade atendida | Arquiteturas reativas. |
| 5. Entradas | Objetos, nomes de evento, callbacks. |
| 6. Saídas | Notificações. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Listener removido ao destruir; ordem estável. |
| 9. Regras de negócio observáveis | Eventos declarados na classe. |
| 10. Dependências | F007 |
| 11. Casos especiais | Listener que destrói a fonte; recursão de eventos. |
| 12. Possíveis erros | Evento inexistente. |
| 13. Critérios de aceitação | CA1: notify dispara todos os listeners<br>CA2: Listener desabilitado não executa |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F018 — Interrupção, pausa e temporizadores

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F018 |
| 2. Nome conceitual | Interrupção, pausa e temporizadores |
| 3. Descrição | Interrupção de execução, pause com/sem tempo, input/keyboard, objetos de temporizador com modos de execução (único, periódico, espaçamento fixo) e fila. |
| 4. Necessidade atendida | Execução agendada e controle interativo. |
| 5. Entradas | Tempo, callbacks. |
| 6. Saídas | Execução diferida. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Temporizadores rodam no laço de eventos do runtime; callbacks não interrompem código em execução no meio de instrução. |
| 9. Regras de negócio observáveis | Período mínimo de 1 ms; modos de execução distintos. |
| 10. Dependências | F001 |
| 11. Casos especiais | Callback mais longo que o período; erro no callback. |
| 12. Possíveis erros | Período inválido. |
| 13. Critérios de aceitação | CA1: Timer periódico executa N vezes com desvio <10ms<br>CA2: Erro no callback chama ErrorFcn |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F019 — Desempenho de execução (compilação JIT)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F019 |
| 2. Nome conceitual | Desempenho de execução (compilação JIT) |
| 3. Descrição | Execução acelerada de código interpretado (compilação just-in-time) transparente ao usuário. |
| 4. Necessidade atendida | Desempenho próximo a código compilado em laços. |
| 5. Entradas | Código. |
| 6. Saídas | Mesma semântica, menor tempo. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Motor de execução com otimização; a meta é de desempenho, sem técnica prescrita. |
| 9. Regras de negócio observáveis | Semântica idêntica ao modo interpretado. |
| 10. Dependências | F001 |
| 11. Casos especiais | Código dinâmico (eval) não compilável — fallback. |
| 12. Possíveis erros | Nenhum visível ao usuário. |
| 13. Critérios de aceitação | CA1: Laço escalar de 1e7 iterações < 5x de C equivalente (meta)<br>CA2: Resultados idênticos entre JIT e interpretador em toda a suíte |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Possível patente/técnica proprietária não verificada: busca de patentes PENDENTE; usar algoritmos publicados. |

*Status:* **PLANEJADA**

### F020 — Operações de conjuntos, ordenação, busca e bits

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F020 |
| 2. Nome conceitual | Operações de conjuntos, ordenação, busca e bits |
| 3. Descrição | sort/sortrows/unique/union/intersect/setdiff/ismember/find/any/all/cumsum/bitand/bitor/bitshift etc. |
| 4. Necessidade atendida | Manipulação de dados básica. |
| 5. Entradas | Arrays. |
| 6. Saídas | Arrays e índices. |
| 7. Fluxo de utilização | Usuário digita/executa código na janela de comandos, script ou função → runtime analisa, avalia e exibe resultado ou erro. |
| 8. Comportamento esperado | Estabilidade de ordenação; ordem de saída 'sorted'/'stable'; NaN tratados como maiores. |
| 9. Regras de negócio observáveis | Saídas de índice opcionais. |
| 10. Dependências | F004 |
| 11. Casos especiais | Tipos mistos; strings; NaN em unique (cada NaN distinto). |
| 12. Possíveis erros | Tipos incompatíveis. |
| 13. Critérios de aceitação | CA1: unique([3 1 NaN NaN]) mantém 2 NaN<br>CA2: sort estável<br>CA3: ismember retorna índices |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

## DAT — Tipos de dados de alto nível

### F021 — Datas e horas (datetime)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F021 |
| 2. Nome conceitual | Datas e horas (datetime) |
| 3. Descrição | Pontos no tempo com fuso horário, formatos de exibição, aritmética, componentes, conversão de/para texto e números. |
| 4. Necessidade atendida | Análise temporal correta. |
| 5. Entradas | Textos, números, componentes. |
| 6. Saídas | Arrays de datetime. |
| 7. Fluxo de utilização | Usuário cria/converte valores do tipo → aplica operações, indexação e funções → exibe, exporta ou persiste. |
| 8. Comportamento esperado | Fusos IANA e horário de verão; formatos baseados em padrões Unicode; NaT para ausente. |
| 9. Regras de negócio observáveis | Comparação entre zonas diferentes converte para UTC. |
| 10. Dependências | F003 |
| 11. Casos especiais | Horário ambíguo/inexistente na transição DST; segundos bissextos (opcional 'UTCLeapSeconds'). |
| 12. Possíveis erros | Formato inválido; zona desconhecida. |
| 13. Critérios de aceitação | CA1: Soma de 1 dia atravessando DST mantém hora local<br>CA2: Leitura de texto com formato explícito<br>CA3: NaT propagado em operações |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F022 — Durações e durações de calendário

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F022 |
| 2. Nome conceitual | Durações e durações de calendário |
| 3. Descrição | Intervalos fixos (duration) e de calendário (meses, anos variáveis). |
| 4. Necessidade atendida | Aritmética de tempo. |
| 5. Entradas | Números, textos. |
| 6. Saídas | Arrays de duração. |
| 7. Fluxo de utilização | Usuário cria/converte valores do tipo → aplica operações, indexação e funções → exibe, exporta ou persiste. |
| 8. Comportamento esperado | Duração de calendário depende da data base; formatação dd:hh:mm:ss. |
| 9. Regras de negócio observáveis | Soma de calendarDuration a datetime respeita fim de mês. |
| 10. Dependências | F021 |
| 11. Casos especiais | 31-jan + 1 mês; durações negativas. |
| 12. Possíveis erros | Formato inválido. |
| 13. Critérios de aceitação | CA1: datetime(2024,1,31)+calmonths(1) == 29-fev-2024<br>CA2: duração exibida no formato escolhido |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F023 — Dados categóricos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F023 |
| 2. Nome conceitual | Dados categóricos |
| 3. Descrição | Arrays de categorias com conjunto de categorias, ordenação opcional, contagem, mesclagem, renomeação. |
| 4. Necessidade atendida | Variáveis qualitativas eficientes. |
| 5. Entradas | Textos/números. |
| 6. Saídas | Arrays categóricos. |
| 7. Fluxo de utilização | Usuário cria/converte valores do tipo → aplica operações, indexação e funções → exibe, exporta ou persiste. |
| 8. Comportamento esperado | Categorias ordinais permitem comparação relacional; <undefined> para ausentes. |
| 9. Regras de negócio observáveis | Categorias protegidas impedem novos valores. |
| 10. Dependências | F009 |
| 11. Casos especiais | Combinação de categóricos com conjuntos diferentes. |
| 12. Possíveis erros | Valor fora das categorias protegidas. |
| 13. Critérios de aceitação | CA1: countcats correto<br>CA2: Comparação ordinal<br>CA3: Ausente exibido como <undefined> |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F024 — Tabelas e tabelas temporais

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F024 |
| 2. Nome conceitual | Tabelas e tabelas temporais |
| 3. Descrição | Dados tabulares heterogêneos com nomes de variáveis/linhas, metadados, junções, agrupamento/agregação, empilhamento, sincronização e retemporização de séries temporais. |
| 4. Necessidade atendida | Análise de dados estruturados. |
| 5. Entradas | Arrays, arquivos. |
| 6. Saídas | Tabelas. |
| 7. Fluxo de utilização | Usuário cria/converte valores do tipo → aplica operações, indexação e funções → exibe, exporta ou persiste. |
| 8. Comportamento esperado | Indexação por nome e posição; grouping (groupsummary, rowfun, varfun); joins (inner/outer); retime/synchronize com métodos de agregação/interpolação. |
| 9. Regras de negócio observáveis | Variáveis de mesma altura; nomes únicos válidos. |
| 10. Dependências | F021, F023 |
| 11. Casos especiais | Variáveis multicoluna; tabelas aninhadas; tempos duplicados/não ordenados. |
| 12. Possíveis erros | Nome duplicado; altura incompatível. |
| 13. Critérios de aceitação | CA1: join interno por chave<br>CA2: retime para dados horários com média<br>CA3: groupsummary por categoria<br>CA4: T.Var acesso por nome |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F025 — Dicionários e mapas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F025 |
| 2. Nome conceitual | Dicionários e mapas |
| 3. Descrição | Containers chave-valor tipados (dictionary) e mapas legados (containers.Map). |
| 4. Necessidade atendida | Busca associativa eficiente. |
| 5. Entradas | Chaves e valores. |
| 6. Saídas | Container. |
| 7. Fluxo de utilização | Usuário cria/converte valores do tipo → aplica operações, indexação e funções → exibe, exporta ou persiste. |
| 8. Comportamento esperado | Chaves tipadas com hash; valores ausentes geram erro ou padrão; Map de referência vs dicionário de valor. |
| 9. Regras de negócio observáveis | Tipo de chave fixo após configuração. |
| 10. Dependências | F009 |
| 11. Casos especiais | Chaves NaN; chaves string vs char. |
| 12. Possíveis erros | Chave inexistente. |
| 13. Critérios de aceitação | CA1: Inserção/busca O(1) amortizado<br>CA2: Semântica de valor do dicionário vs referência do mapa |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F026 — Valores ausentes padronizados

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F026 |
| 2. Nome conceitual | Valores ausentes padronizados |
| 3. Descrição | Valor 'ausente' genérico conversível em NaN/NaT/<missing>/<undefined>; funções ismissing/fillmissing/rmmissing/standardizeMissing. |
| 4. Necessidade atendida | Tratamento uniforme de dados faltantes. |
| 5. Entradas | Arrays/tabelas. |
| 6. Saídas | Arrays sem ausentes ou indicadores. |
| 7. Fluxo de utilização | Usuário cria/converte valores do tipo → aplica operações, indexação e funções → exibe, exporta ou persiste. |
| 8. Comportamento esperado | fillmissing com métodos constante/anterior/próximo/linear/spline/movmean. |
| 9. Regras de negócio observáveis | Cada tipo tem representação própria de ausente. |
| 10. Dependências | F024 |
| 11. Casos especiais | Tabelas com tipos mistos. |
| 12. Possíveis erros | Método inválido. |
| 13. Critérios de aceitação | CA1: fillmissing linear em vetor com NaN<br>CA2: rmmissing remove linhas de tabela com ausentes |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

## MAT — Matemática e computação numérica

### F040 — Álgebra linear densa

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F040 |
| 2. Nome conceitual | Álgebra linear densa |
| 3. Descrição | Solução de sistemas, inversa, determinante, decomposições LU/QR/Cholesky/LDL/Schur/Hessenberg/SVD/autovalores (padrão e generalizado), funções de matriz (expm, logm, sqrtm, funm), normas, condição, posto, pseudo-inversa, espaços nulo/coluna. |
| 4. Necessidade atendida | Base de computação científica. |
| 5. Entradas | Matrizes. |
| 6. Saídas | Fatores, soluções, escalares. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Escolha automática de algoritmo por estrutura (triangular, simétrica, banda); avisos de mal condicionamento. |
| 9. Regras de negócio observáveis | Resultados conforme precisão IEEE; ordem dos autovalores não garantida salvo opção. |
| 10. Dependências | F005 |
| 11. Casos especiais | Matrizes vazias, singulares, complexas, grandes (multithread). |
| 12. Possíveis erros | Matriz não quadrada onde exigido; NaN/Inf. |
| 13. Critérios de aceitação | CA1: Resíduo relativo < 1e-12 em suíte de matrizes de teste<br>CA2: svd de matriz 1000x1000 em tempo comparável a LAPACK otimizado<br>CA3: Aviso de singularidade |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F041 — Matrizes esparsas e solvers iterativos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F041 |
| 2. Nome conceitual | Matrizes esparsas e solvers iterativos |
| 3. Descrição | Armazenamento esparso, operações, ordenações de redução de preenchimento, fatorações esparsas, solvers iterativos (pcg, gmres, bicgstab, minres, lsqr, ...), autovalores/valores singulares parciais, precondicionadores incompletos. |
| 4. Necessidade atendida | Problemas grandes de engenharia. |
| 5. Entradas | Matrizes esparsas. |
| 6. Saídas | Soluções, fatores. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | '\' esparso escolhe solver por estrutura; ordenações AMD/COLAMD; ILU/IC. |
| 9. Regras de negócio observáveis | Esparsidade preservada em operações quando possível. |
| 10. Dependências | F040 |
| 11. Casos especiais | Matriz com zero explícito; muito grande; mal condicionada. |
| 12. Possíveis erros | Não convergência (flag + aviso). |
| 13. Critérios de aceitação | CA1: pcg converge em matriz de Poisson 2D<br>CA2: eigs retorna k maiores autovalores<br>CA3: Memória proporcional a nnz |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | Partes do SuiteSparse são GPL/LGPL — NÃO reutilizar; usar faer sparse (MIT), AMD/COLAMD (BSD-3-Clause), arpack-ng (BSD-3-Clause) ou implementação própria. |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F042 — Funções elementares e especiais

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F042 |
| 2. Nome conceitual | Funções elementares e especiais |
| 3. Descrição | Trigonométricas (rad/graus), exponenciais, logaritmos, arredondamento, módulo, gamma/beta/erf/expint/Bessel/Airy/elípticas/Legendre, primos, fatoração, mdc/mmc, combinatória. |
| 4. Necessidade atendida | Cálculo científico. |
| 5. Entradas | Arrays numéricos. |
| 6. Saídas | Arrays. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Saídas complexas quando domínio exige; precisão próxima a 1 ulp. |
| 9. Regras de negócio observáveis | Funções de grau (sind) exatas em múltiplos de 90. |
| 10. Dependências | F003 |
| 11. Casos especiais | Argumentos extremos; complexos; inteiros grandes. |
| 12. Possíveis erros | Domínio inválido gera NaN/complexo conforme regra. |
| 13. Critérios de aceitação | CA1: sind(180)==0 exato<br>CA2: gamma em teste de referência com erro relativo < 1e-14<br>CA3: besselj ordens fracionárias |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F043 — Estatística descritiva e pré-processamento de dados

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F043 |
| 2. Nome conceitual | Estatística descritiva e pré-processamento de dados |
| 3. Descrição | Média, mediana, moda, desvio, variância, quantis, histogramas/contagens, somas e produtos cumulativos, estatísticas móveis, normalização, reescala, detecção/remoção/preenchimento de outliers, suavização, detecção de mudanças e extremos locais. |
| 4. Necessidade atendida | Exploração e limpeza de dados. |
| 5. Entradas | Arrays/tabelas; dimensão; flags NaN. |
| 6. Saídas | Estatísticas. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Opções 'omitnan'/'includenan', 'all', vecdim; janelas móveis com tratamento de bordas. |
| 9. Regras de negócio observáveis | Dimensão padrão é a primeira não singleton. |
| 10. Dependências | F004 |
| 11. Casos especiais | Entrada vazia (mean de vazio é NaN); tipos inteiros. |
| 12. Possíveis erros | Opção inválida. |
| 13. Critérios de aceitação | CA1: mean(x,'omitnan') ignora NaN<br>CA2: movmean com janela par/ímpar<br>CA3: rmoutliers método mediana |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F044 — Interpolação e ajuste de curvas básico

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F044 |
| 2. Nome conceitual | Interpolação e ajuste de curvas básico |
| 3. Descrição | Interpolação 1-D/2-D/3-D/N-D (linear, nearest, cúbica, spline, pchip, makima), interpolantes em grade e dispersos, splines, extrapolação. |
| 4. Necessidade atendida | Reamostragem e aproximação. |
| 5. Entradas | Pontos, valores, consultas. |
| 6. Saídas | Valores interpolados, objetos interpolantes. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Objetos interpolantes reutilizáveis; métodos de extrapolação configuráveis. |
| 9. Regras de negócio observáveis | Pontos de grade monotônicos. |
| 10. Dependências | F047 |
| 11. Casos especiais | Pontos duplicados; consulta fora do domínio (NaN padrão). |
| 12. Possíveis erros | Grade não monotônica. |
| 13. Critérios de aceitação | CA1: interp1 spline reproduz polinômio cúbico<br>CA2: scattered interpolant com Delaunay |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F045 — Polinômios

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F045 |
| 2. Nome conceitual | Polinômios |
| 3. Descrição | Avaliação, raízes, ajuste por mínimos quadrados, convolução/deconvolução, derivação/integração, frações parciais. |
| 4. Necessidade atendida | Modelagem algébrica. |
| 5. Entradas | Coeficientes. |
| 6. Saídas | Coeficientes, raízes. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Coeficientes em ordem decrescente; polyfit com escalonamento opcional (S, mu). |
| 9. Regras de negócio observáveis | roots via autovalores da companheira. |
| 10. Dependências | F040 |
| 11. Casos especiais | Grau alto mal condicionado (aviso). |
| 12. Possíveis erros | Dados insuficientes. |
| 13. Critérios de aceitação | CA1: roots([1 -3 2]) == [2;1]<br>CA2: polyfit grau n em n+1 pontos interpola |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **IMPLEMENTADA**

### F046 — Transformadas de Fourier e filtragem básica

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F046 |
| 2. Nome conceitual | Transformadas de Fourier e filtragem básica |
| 3. Descrição | FFT/IFFT 1-D/2-D/N-D com qualquer tamanho, deslocamento, filtragem IIR/FIR direta (filter), convolução 1-D/2-D/N-D, correlação básica. |
| 4. Necessidade atendida | Análise espectral e processamento de sinais elementar. |
| 5. Entradas | Arrays; tamanho; dimensão. |
| 6. Saídas | Espectros, sinais filtrados. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Tamanhos arbitrários (não só potências de 2); 'symmetric' em ifft; planejamento de desempenho opcional. |
| 9. Regras de negócio observáveis | Normalização sem fator na FFT direta. |
| 10. Dependências | F040 |
| 11. Casos especiais | Tamanho primo grande; entradas reais vs complexas. |
| 12. Possíveis erros | Dimensão inválida. |
| 13. Critérios de aceitação | CA1: ifft(fft(x)) ≈ x (1e-12)<br>CA2: fft de tamanho primo 10007 correta<br>CA3: filter equivale a equação de diferenças |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | FFTW é GPL — NÃO usar; rustfft (MIT/Apache-2.0). |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F047 — Geometria computacional

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F047 |
| 2. Nome conceitual | Geometria computacional |
| 3. Descrição | Triangulação de Delaunay 2-D/3-D, fecho convexo, diagramas de Voronoi, objetos de triangulação, polígonos (polyshape: booleanas, área, buffer), alpha shapes, ponto-em-polígono. |
| 4. Necessidade atendida | Malhas e análise espacial. |
| 5. Entradas | Pontos, polígonos. |
| 6. Saídas | Triangulações, polígonos. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Consultas de vizinhança e localização; operações booleanas robustas. |
| 9. Regras de negócio observáveis | Pontos colineares/coplanares tratados com aviso. |
| 10. Dependências | F040 |
| 11. Casos especiais | Pontos duplicados; precisão finita. |
| 12. Possíveis erros | Pontos insuficientes. |
| 13. Critérios de aceitação | CA1: delaunay de 1e5 pontos aleatórios satisfaz critério de círculo vazio<br>CA2: polyshape union/intersect com área correta |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | Qhull tem licença própria (revisar); GPC proíbe uso comercial — NÃO usar. Alternativas: spade (MIT/Apache-2.0), i_overlay/geo (MIT/Apache-2.0) ou implementação própria. |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F048 — Integração numérica e quadratura

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F048 |
| 2. Nome conceitual | Integração numérica e quadratura |
| 3. Descrição | Integral adaptativa 1-D/2-D/3-D (incl. limites infinitos e singularidades), regra trapezoidal, cumulativa, Gauss-Kronrod. |
| 4. Necessidade atendida | Cálculo de áreas/volumes. |
| 5. Entradas | Funções/handles, limites, tolerâncias. |
| 6. Saídas | Valores integrais, estimativas de erro. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Tolerâncias absoluta/relativa; avaliação vetorizada da função. |
| 9. Regras de negócio observáveis | Limites infinitos via transformação. |
| 10. Dependências | F016 |
| 11. Casos especiais | Singularidade nos extremos; função não vetorizada. |
| 12. Possíveis erros | Tolerância não alcançada (aviso). |
| 13. Critérios de aceitação | CA1: integral(@(x) exp(-x.^2),-Inf,Inf) ≈ sqrt(pi)<br>CA2: trapz em dados amostrados |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F049 — Equações diferenciais ordinárias e algébrico-diferenciais

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F049 |
| 2. Nome conceitual | Equações diferenciais ordinárias e algébrico-diferenciais |
| 3. Descrição | Solvers explícitos/implícitos de passo variável (Runge-Kutta 4/5 e 2/3, Adams, BDF/NDF, Rosenbrock, trapezoidal, TR-BDF2), problemas implícitos totalmente, DAE de índice 1, eventos, matriz de massa, Jacobiano/esparsidade, saída densa, objeto de problema EDO. |
| 4. Necessidade atendida | Simulação de sistemas dinâmicos. |
| 5. Entradas | Função derivada, intervalo, condição inicial, opções. |
| 6. Saídas | Tempo e estado; eventos; solução avaliável. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Controle de passo por tolerâncias; detecção de eventos com direção e terminal; refino de saída. |
| 9. Regras de negócio observáveis | Interface de opções compatível com conjunto de parâmetros nomeados. |
| 10. Dependências | F040, F016 |
| 11. Casos especiais | Problemas rígidos; descontinuidades; eventos simultâneos. |
| 12. Possíveis erros | Falha de tolerância mínima (aviso + resultado parcial). |
| 13. Critérios de aceitação | CA1: Oscilador de Van der Pol rígido resolvido por BDF<br>CA2: Evento terminal para no instante correto ±tol<br>CA3: Comparação com solução analítica em problemas de referência |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F050 — Problemas de contorno, retardo e EDP 1-D

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F050 |
| 2. Nome conceitual | Problemas de contorno, retardo e EDP 1-D |
| 3. Descrição | Solvers de valor de contorno (colocação), equações com retardo (constante e variável), EDPs parabólicas/elípticas 1-D. |
| 4. Necessidade atendida | Modelos físicos especializados. |
| 5. Entradas | Funções, malha, condições. |
| 6. Saídas | Soluções. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Continuação e refino de malha; históricos de retardo. |
| 9. Regras de negócio observáveis | Malhas monotônicas. |
| 10. Dependências | F049 |
| 11. Casos especiais | Retardos que se anulam; malha inicial ruim. |
| 12. Possíveis erros | Não convergência. |
| 13. Critérios de aceitação | CA1: bvp de referência com erro < tol<br>CA2: dde23 em problema publicado |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F051 — Otimização básica e raízes

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F051 |
| 2. Nome conceitual | Otimização básica e raízes |
| 3. Descrição | Minimização escalar limitada, Nelder-Mead sem restrições, zero de função escalar, mínimos quadrados não negativos, estrutura de opções de otimização, funções de saída/plotagem de iteração. |
| 4. Necessidade atendida | Ajustes e calibrações simples. |
| 5. Entradas | Função objetivo, chute inicial, opções. |
| 6. Saídas | Solução, valor, flag de saída, saída de diagnóstico. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Critérios de parada por tolerância/iterações; exibição iterativa. |
| 9. Regras de negócio observáveis | Flag de saída padronizada. |
| 10. Dependências | F016 |
| 11. Casos especiais | Função descontínua; sem mudança de sinal em fzero. |
| 12. Possíveis erros | Intervalo inválido. |
| 13. Critérios de aceitação | CA1: fminsearch na Rosenbrock converge<br>CA2: fzero encontra raiz com bracket<br>CA3: Funções de plotagem de iteração chamadas a cada passo |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F052 — Grafos e redes

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F052 |
| 2. Nome conceitual | Grafos e redes |
| 3. Descrição | Grafos direcionados/não direcionados com nós/arestas tabulares, caminhos mínimos, BFS/DFS, componentes conexos, árvore geradora, fluxo máximo, centralidade, ordenação topológica, isomorfismo, plotagem com layouts. |
| 4. Necessidade atendida | Análise de redes. |
| 5. Entradas | Listas de arestas, matrizes de adjacência. |
| 6. Saídas | Objetos de grafo, resultados de algoritmos. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Pesos; multigrafos; propriedades de nós/arestas em tabelas. |
| 9. Regras de negócio observáveis | Nós nomeados ou numerados. |
| 10. Dependências | F024 |
| 11. Casos especiais | Grafo vazio; laços; pesos negativos (Bellman-Ford). |
| 12. Possíveis erros | Nó inexistente. |
| 13. Critérios de aceitação | CA1: shortestpath Dijkstra correto<br>CA2: maxflow em rede de teste<br>CA3: plot com layout force |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F053 — Números aleatórios

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F053 |
| 2. Nome conceitual | Números aleatórios |
| 3. Descrição | Geradores configuráveis (Mersenne Twister, SIMD-MT, multiplicative lagged Fibonacci, Philox, Threefry, combined recursive), sementes, estados, fluxos e subfluxos, distribuições uniforme/normal/inteira, permutações. |
| 4. Necessidade atendida | Simulações reprodutíveis. |
| 5. Entradas | Semente, gerador, dimensões. |
| 6. Saídas | Arrays aleatórios; estado. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Reprodutibilidade por semente; fluxos independentes para paralelismo. |
| 9. Regras de negócio observáveis | Estado salvável e restaurável. |
| 10. Dependências | — |
| 11. Casos especiais | Reprodução bit a bit de sequências de outros produtos: NÃO exigida (requisito: algoritmos padrão publicados). |
| 12. Possíveis erros | Gerador desconhecido. |
| 13. Critérios de aceitação | CA1: rng(42) reprodutível na mesma versão<br>CA2: Testes estatísticos (TestU01 SmallCrush) aprovados<br>CA3: Subfluxos não sobrepostos |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F054 — Precisão estendida e aritmética especial

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F054 |
| 2. Nome conceitual | Precisão estendida e aritmética especial |
| 3. Descrição | Funções de precisão e de representação (eps, flintmax, realmax), aritmética inteira (idivide), conversões de base/binário/hexadecimal, aritmética de precisão estendida interna. |
| 4. Necessidade atendida | Controle numérico fino. |
| 5. Entradas | Valores. |
| 6. Saídas | Valores. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Uso interno de bibliotecas de precisão múltipla (sem GMP/MPFR, que são LGPL). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F003 |
| 11. Casos especiais | Valores no limite. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: dec2bin/hex2dec ida-e-volta<br>CA2: flintmax correto |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | GMP/MPFR são LGPL — NÃO usar; usar rug? (LGPL) NÃO; usar dashu (MIT/Apache-2.0) ou implementação própria. |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **EM DESENVOLVIMENTO**

### F055 — Funções de data e tempo legadas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F055 |
| 2. Nome conceitual | Funções de data e tempo legadas |
| 3. Descrição | Funções numéricas de datas (números seriais de data, conversão de texto, calendário, cronômetro tic/toc, clock, cputime). |
| 4. Necessidade atendida | Compatibilidade com código existente. |
| 5. Entradas | Números seriais/textos. |
| 6. Saídas | Números/textos. |
| 7. Fluxo de utilização | Usuário chama a função com arrays de entrada e opções → recebe resultado numérico (e diagnósticos/avisos quando aplicável). |
| 8. Comportamento esperado | Número serial de data com época própria documentada publicamente. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F021 |
| 11. Casos especiais | Anos bissextos; fuso local. |
| 12. Possíveis erros | Formato inválido. |
| 13. Critérios de aceitação | CA1: tic/toc mede intervalo com resolução < 1µs<br>CA2: Conversão serial↔texto |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

## GRF — Gráficos e visualização

### F060 — Sistema de objetos gráficos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F060 |
| 2. Nome conceitual | Sistema de objetos gráficos |
| 3. Descrição | Hierarquia de objetos (raiz, figura, eixos, primitivas, grupos, transformações) com propriedades consultáveis/alteráveis, valores padrão hierárquicos, callbacks, busca de objetos, ciclo de vida. |
| 4. Necessidade atendida | Personalização total de visualizações. |
| 5. Entradas | Chamadas de criação; pares propriedade-valor. |
| 6. Saídas | Handles de objetos. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Notação de ponto e get/set; padrões herdados por nível; handles inválidos após exclusão. |
| 9. Regras de negócio observáveis | Ordem de filhos determina empilhamento. |
| 10. Dependências | F007 |
| 11. Casos especiais | Muitos objetos (1e5); exclusão durante callback. |
| 12. Possíveis erros | Propriedade inexistente/valor inválido. |
| 13. Critérios de aceitação | CA1: set/get e notação de ponto equivalentes<br>CA2: Padrão definido na raiz afeta novas figuras<br>CA3: findobj por propriedade |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F061 — Gráficos 2-D

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F061 |
| 2. Nome conceitual | Gráficos 2-D |
| 3. Descrição | Linhas, dispersão, barras (agrupadas/empilhadas/horizontais), área, escada, haste, barras de erro, histograma (1-D/2-D), pizza/rosca, polares, escalas log, eixos duplos, preenchimentos. |
| 4. Necessidade atendida | Visualização de dados. |
| 5. Entradas | Vetores/matrizes, estilos. |
| 6. Saídas | Figuras. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Ciclo automático de cores e estilos; limites automáticos; 'hold' para sobreposição. |
| 9. Regras de negócio observáveis | Especificadores de linha curtos (cor/marcador/estilo). |
| 10. Dependências | F060 |
| 11. Casos especiais | NaN cria lacunas; datetime/duration/categorical nos eixos. |
| 12. Possíveis erros | Dimensões incompatíveis. |
| 13. Critérios de aceitação | CA1: plot com NaN gera lacuna<br>CA2: eixo datetime formatado<br>CA3: bar empilhado soma correta |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F062 — Gráficos 3-D, superfícies e volumes

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F062 |
| 2. Nome conceitual | Gráficos 3-D, superfícies e volumes |
| 3. Descrição | Linhas 3-D, superfícies/malhas, contornos 2-D/3-D, campos vetoriais, fatias de volume, isossuperfícies, streamlines, patches, iluminação, materiais, câmera, transparência. |
| 4. Necessidade atendida | Visualização científica. |
| 5. Entradas | Grades, volumes. |
| 6. Saídas | Figuras 3-D. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Rotação interativa; projeção ortográfica/perspectiva; colormaps em dados. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060 |
| 11. Casos especiais | Grades irregulares; NaN em superfícies; transparência ordenada. |
| 12. Possíveis erros | Tamanhos incompatíveis. |
| 13. Critérios de aceitação | CA1: surf com colormap e iluminação<br>CA2: isosurface correto em volume de teste<br>CA3: Rotação de câmera interativa a ≥30 fps em 1e6 triângulos (meta) |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F063 — Gráficos especializados de alto nível

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F063 |
| 2. Nome conceitual | Gráficos especializados de alto nível |
| 3. Descrição | Mapas de calor, coordenadas paralelas, gráficos empilhados, enxame, bolhas, caixas, dispersão em matriz, nuvem de palavras básica, gráficos geográficos com mapas base (online). |
| 4. Necessidade atendida | Visualização pronta para análise. |
| 5. Entradas | Tabelas/arrays. |
| 6. Saídas | Gráficos. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Objetos de gráfico com propriedades próprias (não primitivas). |
| 9. Regras de negócio observáveis | Mapas base exigem serviço de tiles. |
| 10. Dependências | F061 |
| 11. Casos especiais | Dados ausentes. |
| 12. Possíveis erros | Variável inexistente. |
| 13. Critérios de aceitação | CA1: heatmap a partir de tabela agrega contagens<br>CA2: boxchart com grupos |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F064 — Anotações, legendas, rótulos e texto matemático

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F064 |
| 2. Nome conceitual | Anotações, legendas, rótulos e texto matemático |
| 3. Descrição | Títulos, rótulos, legendas, barras de cor, texto, setas, formas, interpretadores TeX/LaTeX para fórmulas. |
| 4. Necessidade atendida | Comunicação clara de resultados. |
| 5. Entradas | Textos. |
| 6. Saídas | Objetos de texto. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Interpretador LaTeX com subconjunto matemático; legenda automática. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060 |
| 11. Casos especiais | Unicode; fórmulas longas. |
| 12. Possíveis erros | LaTeX inválido (aviso, texto bruto). |
| 13. Critérios de aceitação | CA1: Legenda lista séries visíveis<br>CA2: LaTeX \frac renderizado |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F065 — Mapas de cores e estilos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F065 |
| 2. Nome conceitual | Mapas de cores e estilos |
| 3. Descrição | Mapas de cores padrão e personalizados (incl. perceptuais), limites de cor, temas claro/escuro para gráficos, ordem de cores. |
| 4. Necessidade atendida | Visualização acessível. |
| 5. Entradas | Matrizes Nx3. |
| 6. Saídas | Estado de cor. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Mapas próprios do projeto; TURBO (Apache-2.0) permitido com atribuição. |
| 9. Regras de negócio observáveis | Nomes de mapas próprios — não copiar paletas proprietárias do original. |
| 10. Dependências | F060 |
| 11. Casos especiais | Mapas com poucos níveis. |
| 12. Possíveis erros | Formato inválido. |
| 13. Critérios de aceitação | CA1: Tema escuro altera cores padrão de figura<br>CA2: colormap personalizado aplicado |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | Paletas padrão do original podem ser elementos distintivos — criar paletas próprias. |
| 18. Restrições jurídicas | Possível patente/técnica proprietária não verificada: busca de patentes PENDENTE; usar algoritmos publicados. |

*Status:* **PLANEJADA**

### F066 — Layouts de múltiplos gráficos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F066 |
| 2. Nome conceitual | Layouts de múltiplos gráficos |
| 3. Descrição | Subdivisão de figura em grades, layouts em mosaico com espaçamento e títulos compartilhados, eixos ligados. |
| 4. Necessidade atendida | Painéis comparativos. |
| 5. Entradas | Dimensões de grade. |
| 6. Saídas | Eixos posicionados. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Redimensionamento responsivo; eixos ligados sincronizam limites. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060 |
| 11. Casos especiais | Mosaicos aninhados; spans. |
| 12. Possíveis erros | Índice fora da grade. |
| 13. Critérios de aceitação | CA1: Layout 2x3 com título compartilhado<br>CA2: linkaxes sincroniza zoom |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F067 — Interação com gráficos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F067 |
| 2. Nome conceitual | Interação com gráficos |
| 3. Descrição | Zoom, pan, rotação, dicas de dados, escovação e ligação de dados, barra de ferramentas de eixos, callbacks de mouse/teclado. |
| 4. Necessidade atendida | Exploração visual. |
| 5. Entradas | Eventos de usuário. |
| 6. Saídas | Estado de visão; seleções. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Dicas personalizáveis; escovação marca dados em variáveis ligadas. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060 |
| 11. Casos especiais | Gráficos com 1e6 pontos. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Zoom por roda do mouse<br>CA2: Dica de dados exibe valores<br>CA3: Escovação propaga para gráficos ligados |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F068 — Animação e captura de quadros

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F068 |
| 2. Nome conceitual | Animação e captura de quadros |
| 3. Descrição | Atualização de gráficos em laço, linhas animadas, captura de quadros, reprodução de filmes, exportação para vídeo/GIF. |
| 4. Necessidade atendida | Visualização dinâmica. |
| 5. Entradas | Atualizações de propriedades. |
| 6. Saídas | Quadros, arquivos. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | drawnow com limitação de taxa ('limitrate'). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060, F088 |
| 11. Casos especiais | Animação com 1e4 quadros. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: drawnow limitrate não excede 20 fps<br>CA2: getframe retorna imagem RGB |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F069 — Exportação e impressão de figuras

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F069 |
| 2. Nome conceitual | Exportação e impressão de figuras |
| 3. Descrição | Salvar/exportar figuras e eixos em PNG, JPEG, TIFF, PDF vetorial, EPS, SVG, com resolução, cor de fundo, recorte; impressão; copiar para área de transferência; formato de figura editável próprio. |
| 4. Necessidade atendida | Relatórios e publicações. |
| 5. Entradas | Figura, formato, opções. |
| 6. Saídas | Arquivos. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Vetorial preserva texto; raster em DPI escolhido. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060 |
| 11. Casos especiais | Figuras com transparência em vetor; fontes ausentes. |
| 12. Possíveis erros | Formato não suportado. |
| 13. Critérios de aceitação | CA1: PDF vetorial contém texto selecionável<br>CA2: PNG 300 dpi com dimensões corretas<br>CA3: Figura salva e recarregada idêntica |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F070 — Exibição de imagens

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F070 |
| 2. Nome conceitual | Exibição de imagens |
| 3. Descrição | Exibição de matrizes como imagens (indexada, intensidade, RGB), escala de cores, eixos de imagem. |
| 4. Necessidade atendida | Visualização de dados de imagem. |
| 5. Entradas | Matrizes. |
| 6. Saídas | Figuras. |
| 7. Fluxo de utilização | Usuário chama função gráfica ou interage com a janela de figura → objetos gráficos criados/atualizados → renderização e opcional exportação. |
| 8. Comportamento esperado | Imagens grandes com reamostragem; proporção de pixel. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060 |
| 11. Casos especiais | Imagens 16 bits; com alfa. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: imagesc escala automática<br>CA2: RGB uint8 exibido sem conversão de cor |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

## APP — Construção de aplicativos e interfaces

### F075 — Componentes de interface programáticos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F075 |
| 2. Nome conceitual | Componentes de interface programáticos |
| 3. Descrição | Janelas de aplicativo com componentes (botões, campos, listas, tabelas, árvores, abas, painéis, layouts em grade, menus, barras de ferramentas, sliders, knobs, medidores, lâmpadas, seletores de data, HTML embutido, eixos), callbacks, estilos. |
| 4. Necessidade atendida | Ferramentas interativas sem programação web. |
| 5. Entradas | Chamadas de criação e propriedades. |
| 6. Saídas | Janelas interativas. |
| 7. Fluxo de utilização | Usuário cria interface por código ou editor visual → define callbacks → executa e interage com o aplicativo. |
| 8. Comportamento esperado | Callbacks com objeto de evento; layout responsivo em grade. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F060 |
| 11. Casos especiais | Muitos componentes; callbacks lentos (interrupção/fila). |
| 12. Possíveis erros | Propriedade inválida. |
| 13. Critérios de aceitação | CA1: Aplicativo com 50 componentes abre < 1 s<br>CA2: Callback de botão recebe evento<br>CA3: Tabela editável sincroniza dados |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F076 — Editor visual de aplicativos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F076 |
| 2. Nome conceitual | Editor visual de aplicativos |
| 3. Descrição | Ambiente de projeto visual com paleta de componentes, canvas, alinhamento, inspetor de propriedades, vista de código com regiões geradas/editáveis, gerenciamento de callbacks, executar/depurar, layouts responsivos. |
| 4. Necessidade atendida | Construção rápida de apps. |
| 5. Entradas | Interações de arrastar/soltar. |
| 6. Saídas | Arquivo de app (formato próprio) com código e layout. |
| 7. Fluxo de utilização | Usuário cria interface por código ou editor visual → define callbacks → executa e interage com o aplicativo. |
| 8. Comportamento esperado | Código gerado e código do usuário separados; renomeação propaga. |
| 9. Regras de negócio observáveis | Formato de arquivo próprio do projeto. |
| 10. Dependências | F075, F101 |
| 11. Casos especiais | Conversão de apps legados. |
| 12. Possíveis erros | Arquivo corrompido. |
| 13. Critérios de aceitação | CA1: Criar app com botão e callback sem escrever layout manualmente<br>CA2: Reabrir preserva código do usuário |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F077 — Interfaces legadas baseadas em figura

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F077 |
| 2. Nome conceitual | Interfaces legadas baseadas em figura |
| 3. Descrição | Controles de interface em figuras tradicionais, editor de layout legado e conversão para o novo modelo. |
| 4. Necessidade atendida | Compatibilidade com aplicações antigas. |
| 5. Entradas | Arquivos de layout legado. |
| 6. Saídas | Figuras com controles. |
| 7. Fluxo de utilização | Usuário cria interface por código ou editor visual → define callbacks → executa e interage com o aplicativo. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F075 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: uicontrol cria botão funcional |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

### F078 — Diálogos padrão

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F078 |
| 2. Nome conceitual | Diálogos padrão |
| 3. Descrição | Seleção de arquivo/pasta, mensagens, perguntas, entrada de texto, lista, progresso, cor, fonte; diálogos dentro de app. |
| 4. Necessidade atendida | Interação simples com usuário. |
| 5. Entradas | Opções. |
| 6. Saídas | Escolhas do usuário. |
| 7. Fluxo de utilização | Usuário cria interface por código ou editor visual → define callbacks → executa e interage com o aplicativo. |
| 8. Comportamento esperado | Modais e não modais; cancelamento retorna valor distinto. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F075 |
| 11. Casos especiais | Cancelamento; multisseleção. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Cancelar retorna 0/vazio<br>CA2: Barra de progresso atualiza e fecha |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F079 — Empacotamento e compartilhamento de apps

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F079 |
| 2. Nome conceitual | Empacotamento e compartilhamento de apps |
| 3. Descrição | Empacotar app com dependências em instalador de app, galeria de apps instalados. |
| 4. Necessidade atendida | Distribuir ferramentas a colegas. |
| 5. Entradas | Projeto de app. |
| 6. Saídas | Pacote instalável. |
| 7. Fluxo de utilização | Usuário cria interface por código ou editor visual → define callbacks → executa e interage com o aplicativo. |
| 8. Comportamento esperado | Análise automática de dependências. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F076, F131 |
| 11. Casos especiais | Dependências de outras extensões. |
| 12. Possíveis erros | Arquivo ausente. |
| 13. Critérios de aceitação | CA1: Empacotar e instalar app em outra máquina |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

## IDE — Ambiente de desenvolvimento

### F085 — Ambiente desktop com painéis

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F085 |
| 2. Nome conceitual | Ambiente desktop com painéis |
| 3. Descrição | Janela principal com janela de comandos, navegador de arquivos, workspace, histórico, editor, figuras ancoradas, barras de ferramentas, layouts salvos, tema claro/escuro, zoom de fonte, múltiplos monitores. |
| 4. Necessidade atendida | Ambiente integrado produtivo. |
| 5. Entradas | Interações de UI. |
| 6. Saídas | Estado de layout persistido. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Painéis ancoráveis/desacopláveis; layouts nomeados. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F001 |
| 11. Casos especiais | Telas HiDPI; tela pequena. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Layout persiste entre sessões<br>CA2: Tema escuro aplicado a todos os painéis<br>CA3: Painel desacoplado em outro monitor |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F086 — Editor de código

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F086 |
| 2. Nome conceitual | Editor de código |
| 3. Descrição | Edição com realce, dobra, múltiplos cursores, busca/substituição com regex, indentação automática, formatação, autocompletar contextual, dicas de assinatura, ir para definição, renomeação de variáveis, seções de código executáveis, execução de seleção. |
| 4. Necessidade atendida | Escrita eficiente de código. |
| 5. Entradas | Texto do usuário. |
| 6. Saídas | Arquivos salvos. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Análise contínua em segundo plano; seções delimitadas por marcador de comentário. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F087 |
| 11. Casos especiais | Arquivos > 10k linhas; codificações. |
| 12. Possíveis erros | Arquivo somente leitura. |
| 13. Critérios de aceitação | CA1: Autocompletar sugere variáveis e funções do caminho<br>CA2: Executar seção avalia apenas a seção<br>CA3: Renomear variável atualiza ocorrências no escopo |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F087 — Análise estática de código

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F087 |
| 2. Nome conceitual | Análise estática de código |
| 3. Descrição | Mensagens de problemas/melhorias em tempo real, correções automáticas, supressão por linha/arquivo, regras configuráveis, relatório por pasta, métricas (complexidade). |
| 4. Necessidade atendida | Qualidade e prevenção de erros. |
| 5. Entradas | Código. |
| 6. Saídas | Diagnósticos. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Identificadores de mensagem estáveis; configuração por arquivo de projeto. |
| 9. Regras de negócio observáveis | Regras próprias (não copiar textos de mensagens do original). |
| 10. Dependências | F086 |
| 11. Casos especiais | Código com erro de sintaxe parcial. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Variável não usada sinalizada<br>CA2: Supressão por comentário respeitada<br>CA3: Relatório de pasta exportável |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F088 — Depurador

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F088 |
| 2. Nome conceitual | Depurador |
| 3. Descrição | Pontos de parada (linha, condicionais, em erro/aviso/NaN), passo a passo, entrar/sair, pilha de chamadas, inspeção e alteração de variáveis, continuar até cursor, depuração de apps e callbacks. |
| 4. Necessidade atendida | Diagnóstico de defeitos. |
| 5. Entradas | Pontos de parada, comandos. |
| 6. Saídas | Estado pausado. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Pontos de parada persistem entre edições quando possível. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F086, F019 |
| 11. Casos especiais | Parada em código JIT; em callback de timer. |
| 12. Possíveis erros | Linha inválida. |
| 13. Critérios de aceitação | CA1: Parada condicional dispara só quando verdadeira<br>CA2: 'parar em erro' pausa no local do erro<br>CA3: Variável alterada no modo depuração afeta execução |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F089 — Profiler de desempenho

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F089 |
| 2. Nome conceitual | Profiler de desempenho |
| 3. Descrição | Medição de tempo por função/linha, contagem de chamadas, árvore de chamadas, gráfico de chama, comparação, relatório HTML, profiler de memória. |
| 4. Necessidade atendida | Otimização de desempenho. |
| 5. Entradas | Execução de código. |
| 6. Saídas | Relatórios. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Overhead controlado; histórico de execuções. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F001 |
| 11. Casos especiais | Código recursivo; multithread. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Relatório lista tempo por linha<br>CA2: Gráfico de chama navegável |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F090 — Navegador de arquivos e pasta atual

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F090 |
| 2. Nome conceitual | Navegador de arquivos e pasta atual |
| 3. Descrição | Navegação, criação, renomeação, exclusão, pré-visualização, filtros, indicação de status de controle de versão, busca de arquivos. |
| 4. Necessidade atendida | Gestão de arquivos do projeto. |
| 5. Entradas | Ações de arquivo. |
| 6. Saídas | Sistema de arquivos atualizado. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Sincroniza com 'cd' do runtime. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | Pastas grandes (1e5 arquivos). |
| 12. Possíveis erros | Permissão negada. |
| 13. Critérios de aceitação | CA1: cd no prompt atualiza painel<br>CA2: Excluir pede confirmação |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F091 — Navegador de workspace e editor de variáveis

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F091 |
| 2. Nome conceitual | Navegador de workspace e editor de variáveis |
| 3. Descrição | Lista de variáveis com tamanho/classe/estatísticas, edição em planilha de arrays/tabelas/structs, gráficos rápidos a partir da seleção, inspeção de objetos, limpeza de dados interativa. |
| 4. Necessidade atendida | Inspeção e edição de dados. |
| 5. Entradas | Variáveis. |
| 6. Saídas | Variáveis modificadas; código gerado. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Edições geram código equivalente reproduzível. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | Variáveis enormes (paginação). |
| 12. Possíveis erros | Valor inválido. |
| 13. Critérios de aceitação | CA1: Editar célula atualiza variável<br>CA2: Seleção gera gráfico<br>CA3: Edições registradas como código |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F092 — Ferramenta de importação de dados

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F092 |
| 2. Nome conceitual | Ferramenta de importação de dados |
| 3. Descrição | Assistente para importar texto/planilha com pré-visualização, detecção de tipos, seleção de intervalo, regras para valores inválidos, geração de script/função de importação. |
| 4. Necessidade atendida | Importação sem código. |
| 5. Entradas | Arquivos. |
| 6. Saídas | Variáveis e código gerado. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Usa mesmas opções que funções de leitura (F112, F113). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F112, F113 |
| 11. Casos especiais | Arquivos grandes; cabeçalhos múltiplos. |
| 12. Possíveis erros | Arquivo ilegível. |
| 13. Critérios de aceitação | CA1: Importar CSV com datas detectadas<br>CA2: Script gerado reproduz importação |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F093 — Histórico de comandos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F093 |
| 2. Nome conceitual | Histórico de comandos |
| 3. Descrição | Registro persistente de comandos com busca, reexecução, criação de script a partir de seleção, agrupamento por sessão. |
| 4. Necessidade atendida | Reprodutibilidade de trabalho interativo. |
| 5. Entradas | Comandos. |
| 6. Saídas | Histórico. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Configurável (tamanho, duplicatas). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F001 |
| 11. Casos especiais | Histórico corrompido. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Seta para cima com prefixo filtra histórico<br>CA2: Criar script a partir da seleção |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F094 — Projetos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F094 |
| 2. Nome conceitual | Projetos |
| 3. Descrição | Projetos com arquivos rastreados, rótulos, atalhos, caminho e scripts de inicialização/encerramento, análise de dependências, verificações de integridade, referências a outros projetos, exportação/arquivamento, integração com controle de versão. |
| 4. Necessidade atendida | Organização de trabalho em equipe. |
| 5. Entradas | Pasta de projeto. |
| 6. Saídas | Metadados de projeto (formato próprio). |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Abrir projeto configura caminho e executa inicialização. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F095, F014 |
| 11. Casos especiais | Projetos aninhados/referenciados; arquivos fora do projeto. |
| 12. Possíveis erros | Metadados inválidos. |
| 13. Critérios de aceitação | CA1: Abrir projeto adiciona pastas ao caminho<br>CA2: Análise de dependências encontra arquivos faltantes<br>CA3: Exportar como arquivo compactado |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F095 — Controle de versão integrado

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F095 |
| 2. Nome conceitual | Controle de versão integrado |
| 3. Descrição | Git (clonar, commit, branch, merge, push/pull, histórico, stash, conflitos) e Subversion; indicadores no navegador de arquivos; diff/merge visual de arquivos de código e binários do projeto. |
| 4. Necessidade atendida | Colaboração e rastreabilidade. |
| 5. Entradas | Repositórios. |
| 6. Saídas | Operações VCS. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Credenciais pelo armazenamento do sistema. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F096 |
| 11. Casos especiais | Conflitos; repositórios grandes; LFS. |
| 12. Possíveis erros | Falha de autenticação. |
| 13. Critérios de aceitação | CA1: Commit/branch/merge via UI<br>CA2: Indicador de modificado no navegador |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | libgit2 é GPL-2.0 com exceção de linking — fora da política; usar gitoxide (MIT/Apache-2.0) ou CLI git do sistema; Subversion via CLI ou libsvn (Apache-2.0). |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F096 — Comparação e mesclagem de arquivos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F096 |
| 2. Nome conceitual | Comparação e mesclagem de arquivos |
| 3. Descrição | Diferença lado a lado de texto, código, pastas, arquivos de dados (variáveis), documentos executáveis e modelos; mesclagem de três vias; relatório exportável. |
| 4. Necessidade atendida | Revisão de alterações. |
| 5. Entradas | Dois/três arquivos. |
| 6. Saídas | Relatório; arquivo mesclado. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Comparação semântica de tipos estruturados. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | Arquivos binários grandes. |
| 12. Possíveis erros | Tipos incompatíveis. |
| 13. Critérios de aceitação | CA1: Diff de texto com realce intra-linha<br>CA2: Comparar dois arquivos de dados lista variáveis diferentes |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F097 — Preferências e configurações

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F097 |
| 2. Nome conceitual | Preferências e configurações |
| 3. Descrição | Sistema hierárquico de configurações (fábrica, instalação, usuário, sessão) com API programática, diálogo de preferências, importação/exportação. |
| 4. Necessidade atendida | Personalização. |
| 5. Entradas | Valores de configuração. |
| 6. Saídas | Configurações persistidas. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Precedência de níveis; validação de tipos. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | Arquivo corrompido (restauração). |
| 12. Possíveis erros | Valor inválido. |
| 13. Critérios de aceitação | CA1: Valor de sessão sobrepõe usuário<br>CA2: Exportar/importar preferências |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F098 — Atalhos de teclado e ações personalizáveis

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F098 |
| 2. Nome conceitual | Atalhos de teclado e ações personalizáveis |
| 3. Descrição | Mapa de atalhos editável, conjuntos predefinidos (incl. estilo de outros editores), favoritos (botões de código), barra de acesso rápido. |
| 4. Necessidade atendida | Produtividade. |
| 5. Entradas | Mapeamentos. |
| 6. Saídas | Ações. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Conflitos detectados. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | — |
| 12. Possíveis erros | Conflito de atalho. |
| 13. Critérios de aceitação | CA1: Atalho personalizado executa ação<br>CA2: Favorito executa trecho de código |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F099 — Visualizador de diagramas de classes e dependências

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F099 |
| 2. Nome conceitual | Visualizador de diagramas de classes e dependências |
| 3. Descrição | Diagramas de classes gerados de código, grafo de dependências de arquivos/projetos. |
| 4. Necessidade atendida | Entendimento de arquitetura. |
| 5. Entradas | Pastas/classes. |
| 6. Saídas | Diagramas. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F094 |
| 11. Casos especiais | Projetos grandes. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Diagrama de classes mostra herança |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F100 — Opções de inicialização e modos sem interface

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F100 |
| 2. Nome conceitual | Opções de inicialização e modos sem interface |
| 3. Descrição | Execução em lote ('-batch'), sem desktop, script de inicialização, variáveis de ambiente, log de sessão, código de saída, execução de script por linha de comando. |
| 4. Necessidade atendida | Automação e CI. |
| 5. Entradas | Argumentos de linha de comando. |
| 6. Saídas | Execução sem UI; código de saída. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Erro não tratado em lote retorna código ≠ 0. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F001 |
| 11. Casos especiais | Sem display (headless) com gráficos (renderização offscreen). |
| 12. Possíveis erros | Argumento inválido. |
| 13. Critérios de aceitação | CA1: chimalab -batch "disp(1)" imprime 1 e sai com 0<br>CA2: Erro sai com código ≠ 0<br>CA3: Gráficos exportáveis sem display |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F101 — Barra de ferramentas contextual (faixa de opções)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F101 |
| 2. Nome conceitual | Barra de ferramentas contextual (faixa de opções) |
| 3. Descrição | Faixa de opções com abas contextuais por ferramenta/editor, galeria de apps, pesquisa de ações. |
| 4. Necessidade atendida | Descoberta de recursos. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Disposição própria (não reproduzir layout distintivo do original). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Aba contextual aparece ao focar editor |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | Expressão visual do original é protegida — criar organização própria. |
| 18. Restrições jurídicas | Possível patente/técnica proprietária não verificada: busca de patentes PENDENTE; usar algoritmos publicados. |

*Status:* **PLANEJADA**

### F102 — Notificações e mensagens do sistema

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F102 |
| 2. Nome conceitual | Notificações e mensagens do sistema |
| 3. Descrição | Central de notificações (atualizações, tarefas concluídas). |
| 4. Necessidade atendida | Informação contextual. |
| 5. Entradas | Eventos. |
| 6. Saídas | Notificações. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Notificação de tarefa em segundo plano concluída |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F103 — Busca global

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F103 |
| 2. Nome conceitual | Busca global |
| 3. Descrição | Busca de arquivos, símbolos, ações e documentação a partir de campo único. |
| 4. Necessidade atendida | Navegação rápida. |
| 5. Entradas | Consulta. |
| 6. Saídas | Resultados. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Índice incremental. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085, F150 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Busca encontra função por nome parcial |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F104 — Ambiente via navegador e acesso remoto

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F104 |
| 2. Nome conceitual | Ambiente via navegador e acesso remoto |
| 3. Descrição | Uso do ambiente pelo navegador conectado a sessão local/remota. |
| 4. Necessidade atendida | Uso remoto/nuvem. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Inferência: componentes 'connector' indicam servidor web interno para UI. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Interface acessível por navegador local com autenticação por token |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Hipótese. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

### F105 — Assistente de IA/sugestões de código

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F105 |
| 2. Nome conceitual | Assistente de IA/sugestões de código |
| 3. Descrição | Sugestões de código baseadas em modelos (componentes 'suggestion_models' presentes). |
| 4. Necessidade atendida | Produtividade. |
| 5. Entradas | Contexto do editor. |
| 6. Saídas | Sugestões. |
| 7. Fluxo de utilização | Usuário opera o ambiente desktop (painéis, editor, ferramentas) → ação refletida no estado do runtime/arquivos. |
| 8. Comportamento esperado | Hipótese: autocompletar com modelo local e/ou serviço. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F086 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Sugestões opcionais e desativáveis; nenhum dado enviado sem consentimento |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Hipótese. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

## DOC — Documentos executáveis e publicação

### F106 — Documentos executáveis (scripts vivos)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F106 |
| 2. Nome conceitual | Documentos executáveis (scripts vivos) |
| 3. Descrição | Documento com texto formatado, equações, imagens, links, código e saídas embutidas (valores, gráficos, tabelas); execução por seção; controles interativos (sliders, listas, botões) ligados a variáveis; tarefas interativas que geram código; modo de leitura/ocultar código. |
| 4. Necessidade atendida | Análise reprodutível e comunicação. |
| 5. Entradas | Documento. |
| 6. Saídas | Documento com saídas; formato de arquivo próprio aberto. |
| 7. Fluxo de utilização | Usuário edita documento com texto, código e resultados → executa seções → exporta/publica. |
| 8. Comportamento esperado | Saídas associadas às linhas; execução incremental; alteração de controle reexecuta seção. |
| 9. Regras de negócio observáveis | Formato de arquivo próprio (ZIP+JSON documentado); importação de formatos abertos (ipynb, Markdown). |
| 10. Dependências | F086, F060 |
| 11. Casos especiais | Documentos grandes; saídas pesadas. |
| 12. Possíveis erros | Arquivo corrompido (recuperação). |
| 13. Critérios de aceitação | CA1: Gráfico aparece inline abaixo do código<br>CA2: Slider reexecuta seção ao mudar<br>CA3: Salvar/reabrir preserva saídas |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Leitura/escrita de formatos de terceiros para interoperabilidade: só no escopo do DT-10, a partir de especificações e implementações independentes e permissivas. |

*Status:* **PLANEJADA**

### F107 — Exportação de documentos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F107 |
| 2. Nome conceitual | Exportação de documentos |
| 3. Descrição | Exportar documento executável/script para PDF, HTML, Word (docx), LaTeX, Markdown, notebook Jupyter. |
| 4. Necessidade atendida | Publicação. |
| 5. Entradas | Documento. |
| 6. Saídas | Arquivos. |
| 7. Fluxo de utilização | Usuário edita documento com texto, código e resultados → executa seções → exporta/publica. |
| 8. Comportamento esperado | Opções de página e inclusão de código. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F106 |
| 11. Casos especiais | Equações e imagens em todos os formatos. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Exportar para PDF com equações<br>CA2: Exportar ipynb abre no Jupyter |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F108 — Publicação de scripts com marcação

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F108 |
| 2. Nome conceitual | Publicação de scripts com marcação |
| 3. Descrição | Gerar relatório a partir de script com comentários de marcação (títulos, listas, código, saídas, imagens) em HTML/PDF/DOC/LaTeX/XML. |
| 4. Necessidade atendida | Relatórios a partir de código. |
| 5. Entradas | Script. |
| 6. Saídas | Relatório. |
| 7. Fluxo de utilização | Usuário edita documento com texto, código e resultados → executa seções → exporta/publica. |
| 8. Comportamento esperado | Estilo via folha de transformação personalizável. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F002 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Publicar script com seções gera HTML com índice |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F109 — Verificação ortográfica

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F109 |
| 2. Nome conceitual | Verificação ortográfica |
| 3. Descrição | Verificação ortográfica em texto de documentos e comentários. |
| 4. Necessidade atendida | Qualidade de texto. |
| 5. Entradas | Texto. |
| 6. Saídas | Sugestões. |
| 7. Fluxo de utilização | Usuário edita documento com texto, código e resultados → executa seções → exporta/publica. |
| 8. Comportamento esperado | Dicionários por idioma. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F106 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Palavra incorreta sublinhada |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | Hunspell é MPL/LGPL/GPL — NÃO usar; alternativa: spellbook (MPL? verificar) ou zspell (MIT) + dicionários com licença compatível. |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

## IO — Entrada/saída e formatos de arquivo

### F110 — Arquivos de dados nativos (MAT)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F110 |
| 2. Nome conceitual | Arquivos de dados nativos (MAT) |
| 3. Descrição | Salvar/carregar variáveis em arquivo binário de dados (versões 4, 5/6/7 compactada, 7.3 baseada em HDF5), carga/salvamento parcial, acesso a partes de variáveis sem carregar tudo, texto ASCII. |
| 4. Necessidade atendida | Persistência e intercâmbio com usuários do original. |
| 5. Entradas | Variáveis/arquivo. |
| 6. Saídas | Arquivo/variáveis. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Compressão; objetos serializados; leitura parcial para arquivos v7.3. |
| 9. Regras de negócio observáveis | Formato v5 tem especificação pública do fabricante; v7.3 é HDF5 com convenções. |
| 10. Dependências | F120 |
| 11. Casos especiais | Arquivos > 2 GB; objetos de classes desconhecidas (carregar como struct + aviso). |
| 12. Possíveis erros | Arquivo corrompido; versão não suportada. |
| 13. Critérios de aceitação | CA1: Ida-e-volta de todos os tipos básicos<br>CA2: Ler arquivo v5/v7/v7.3 de referência gerado por ferramenta de terceiros<br>CA3: Leitura parcial de matriz 10 GB sem carregar inteira |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Leitura/escrita de formatos de terceiros para interoperabilidade: só no escopo do DT-10, a partir de especificações e implementações independentes e permissivas. |

*Status:* **PLANEJADA**

### F111 — E/S de arquivo de baixo nível

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F111 |
| 2. Nome conceitual | E/S de arquivo de baixo nível |
| 3. Descrição | fopen/fclose/fread/fwrite/fprintf/fscanf/fgetl/fgets/fseek/ftell/feof/ferror com codificações e ordem de bytes; mapeamento de arquivo em memória. |
| 4. Necessidade atendida | Formatos binários personalizados. |
| 5. Entradas | Arquivos, formatos. |
| 6. Saídas | Dados. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Precisões com conversão ('int16=>double'); codificação de texto configurável. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F003 |
| 11. Casos especiais | Arquivos > 4 GB; fim de linha misto. |
| 12. Possíveis erros | Arquivo inexistente; permissão. |
| 13. Critérios de aceitação | CA1: fread com ordem big-endian<br>CA2: memmapfile acesso aleatório |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F112 — Texto delimitado e de largura fixa

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F112 |
| 2. Nome conceitual | Texto delimitado e de largura fixa |
| 3. Descrição | Leitura/escrita de tabelas, matrizes, células e linhas de texto/CSV com detecção automática de opções (delimitadores, cabeçalhos, tipos, formatos de data, locale), objetos de opções de importação editáveis, leitura formatada com especificadores. |
| 4. Necessidade atendida | Importação de dados comuns. |
| 5. Entradas | Arquivos texto. |
| 6. Saídas | Tabelas/arrays. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Detecção heurística reproduzível; opções explicitamente configuráveis. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F024 |
| 11. Casos especiais | Aspas, delimitadores múltiplos, linhas malformadas, BOM, codificações. |
| 12. Possíveis erros | Arquivo malformado (regra configurável: erro/omitir/preencher). |
| 13. Critérios de aceitação | CA1: CSV com cabeçalho → tabela com tipos corretos<br>CA2: Opções editadas mudam tipo de coluna<br>CA3: textscan com formato |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F113 — Planilhas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F113 |
| 2. Nome conceitual | Planilhas |
| 3. Descrição | Leitura/escrita de planilhas (xlsx, xlsm, xls, ods) com folhas, intervalos, formatos de data, preservação de formatação ao escrever; informação de folhas. |
| 4. Necessidade atendida | Intercâmbio com usuários de planilhas. |
| 5. Entradas | Arquivos de planilha. |
| 6. Saídas | Tabelas/arrays. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Sem depender de software de planilha instalado. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F024 |
| 11. Casos especiais | Células mescladas; fórmulas (valor calculado); datas 1900/1904. |
| 12. Possíveis erros | Folha inexistente. |
| 13. Critérios de aceitação | CA1: Ler intervalo nomeado<br>CA2: Escrever tabela preservando formatação existente<br>CA3: xls legado lido |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | LibXL é comercial — NÃO usar; calamine (MIT) para leitura; rust_xlsxwriter (MIT/Apache-2.0) para escrita; xls legado: leitor próprio (formato documentado publicamente pela Microsoft). |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F114 — JSON

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F114 |
| 2. Nome conceitual | JSON |
| 3. Descrição | Codificação/decodificação JSON ↔ estruturas, leitura/escrita de arquivos JSON em structs/tabelas com opções (formatação, nomes de campos). |
| 4. Necessidade atendida | Integração web e configuração. |
| 5. Entradas | Texto/arquivos JSON. |
| 6. Saídas | Structs/arrays. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Arrays homogêneos viram arrays numéricos; null → [] ou NaN conforme opção. |
| 9. Regras de negócio observáveis | Nomes inválidos ajustados reversivelmente. |
| 10. Dependências | F009 |
| 11. Casos especiais | Números grandes; Unicode; profundidade. |
| 12. Possíveis erros | JSON inválido com posição. |
| 13. Critérios de aceitação | CA1: Ida-e-volta preserva estrutura<br>CA2: Pretty print opcional |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F115 — XML

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F115 |
| 2. Nome conceitual | XML |
| 3. Descrição | Leitura/escrita de XML em structs, DOM completo (parse, navegação, XPath, transformação XSLT, validação por esquema), escrita. |
| 4. Necessidade atendida | Integração com sistemas corporativos. |
| 5. Entradas | Arquivos XML. |
| 6. Saídas | Structs/DOM. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Namespaces; atributos mapeados. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F009 |
| 11. Casos especiais | XML grande (streaming); entidades externas desativadas por segurança. |
| 12. Possíveis erros | XML malformado. |
| 13. Critérios de aceitação | CA1: XPath retorna nós<br>CA2: XXE bloqueado por padrão |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F116 — HTML e documentos de texto

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F116 |
| 2. Nome conceitual | HTML e documentos de texto |
| 3. Descrição | Leitura de tabelas de páginas HTML e de documentos Word (tabelas/texto) para tabelas. |
| 4. Necessidade atendida | Extração de dados de documentos. |
| 5. Entradas | Arquivos HTML/DOCX. |
| 6. Saídas | Tabelas. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F112 |
| 11. Casos especiais | Tabelas aninhadas. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: readtable de tabela HTML<br>CA2: readtable de tabela em .docx |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F117 — Formatos colunares (Parquet, Arrow/Feather)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F117 |
| 2. Nome conceitual | Formatos colunares (Parquet, Arrow/Feather) |
| 3. Descrição | Leitura/escrita de Parquet com compressão, metadados, leitura seletiva de colunas/linhas; Feather. |
| 4. Necessidade atendida | Interoperabilidade com ecossistemas de dados. |
| 5. Entradas | Arquivos. |
| 6. Saídas | Tabelas. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Mapeamento de tipos documentado. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F024 |
| 11. Casos especiais | Tipos aninhados; datas com fuso. |
| 12. Possíveis erros | Tipo sem mapeamento. |
| 13. Critérios de aceitação | CA1: Ida-e-volta Parquet com datetime<br>CA2: Leitura de colunas selecionadas |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F118 — Imagens

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F118 |
| 2. Nome conceitual | Imagens |
| 3. Descrição | Leitura/escrita/informação de BMP, GIF (animado), JPEG, JPEG 2000, PCX, PNG (alfa, 16 bits), TIFF (multipágina, tiles, metadados), PPM/PGM, ICO/CUR, HDF raster, e formatos científicos de imagem. |
| 4. Necessidade atendida | Processamento de imagens. |
| 5. Entradas | Arquivos. |
| 6. Saídas | Arrays. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Metadados acessíveis; qualidade/compressão configuráveis. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F003 |
| 11. Casos especiais | Imagens enormes; CMYK; paletas. |
| 12. Possíveis erros | Formato desconhecido. |
| 13. Critérios de aceitação | CA1: PNG 16 bits com alfa ida-e-volta<br>CA2: TIFF multipágina lido por índice |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | Kakadu (JPEG 2000) é comercial — NÃO usar; OpenJPEG (BSD-2-Clause). |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F119 — Áudio e vídeo

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F119 |
| 2. Nome conceitual | Áudio e vídeo |
| 3. Descrição | Leitura/escrita de áudio (WAV, FLAC, OGG, MP3 leitura, MP4/M4A), reprodução e gravação de áudio com dispositivos, leitura/escrita de vídeo (AVI, MP4, MJ2, etc.) quadro a quadro, informação de mídia. |
| 4. Necessidade atendida | Multimídia científica. |
| 5. Entradas | Arquivos/dispositivos. |
| 6. Saídas | Arrays de amostras/quadros. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Seleção de dispositivo; taxas de amostragem. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F003 |
| 11. Casos especiais | Codecs indisponíveis na plataforma. |
| 12. Possíveis erros | Codec não suportado. |
| 13. Critérios de aceitação | CA1: audioread WAV/FLAC correto<br>CA2: Gravar e reproduzir com dispositivo padrão<br>CA3: VideoWriter MP4 legível em players comuns |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | libsndfile LGPL e FFmpeg LGPL/GPL — NÃO usar. hound/claxon/lewton (Apache-2.0/MIT), cpal (Apache-2.0), codecs H.264/AAC possuem patentes: usar frameworks do SO (Media Foundation/AVFoundation) ou openh264 (BSD-2-Clause, avaliar patentes). |
| 18. Restrições jurídicas | Possível patente/técnica proprietária não verificada: busca de patentes PENDENTE; usar algoritmos publicados. |

*Status:* **PLANEJADA**

### F120 — Formatos científicos (HDF5, HDF4, NetCDF, CDF, FITS)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F120 |
| 2. Nome conceitual | Formatos científicos (HDF5, HDF4, NetCDF, CDF, FITS) |
| 3. Descrição | Leitura/escrita de alto nível e APIs de baixo nível para HDF5, NetCDF; leitura de HDF4/HDF-EOS, CDF, FITS; informação/estrutura de arquivos. |
| 4. Necessidade atendida | Dados científicos padrão. |
| 5. Entradas | Arquivos. |
| 6. Saídas | Arrays/estruturas. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Atributos, grupos, chunking, compressão. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F003 |
| 11. Casos especiais | Arquivos > memória; tipos compostos. |
| 12. Possíveis erros | Arquivo inválido. |
| 13. Critérios de aceitação | CA1: h5write/h5read ida-e-volta com atributos<br>CA2: NetCDF leitura de variável parcial |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | Bibliotecas de referência têm licenças BSD-like próprias (HDF5, NetCDF, CFITSIO) e NASA OSA (HDF-EOS2) — revisar cada uma; NASA OSA fora da política. |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F121 — Sistema de arquivos e arquivos compactados

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F121 |
| 2. Nome conceitual | Sistema de arquivos e arquivos compactados |
| 3. Descrição | Listar, criar, copiar, mover, excluir, atributos, caminhos; zip/unzip, tar/untar, gzip/gunzip; caminhos remotos (armazenamento em nuvem e HTTP) em funções de E/S. |
| 4. Necessidade atendida | Gestão de arquivos. |
| 5. Entradas | Caminhos. |
| 6. Saídas | Arquivos/listagens. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Curingas; recursão; caminhos remotos por prefixo de esquema. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | Links simbólicos; nomes Unicode; caminhos longos no Windows. |
| 12. Possíveis erros | Permissão; inexistente. |
| 13. Critérios de aceitação | CA1: dir recursivo com **<br>CA2: zip/unzip ida-e-volta<br>CA3: Leitura de CSV em armazenamento de objetos compatível S3 |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F122 — Comandos do sistema e ambiente

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F122 |
| 2. Nome conceitual | Comandos do sistema e ambiente |
| 3. Descrição | Executar comandos do SO com captura de saída e status, variáveis de ambiente, informação do computador, área de transferência. |
| 4. Necessidade atendida | Integração com ferramentas externas. |
| 5. Entradas | Comandos. |
| 6. Saídas | Saídas. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | Comandos interativos; codificação. |
| 12. Possíveis erros | Comando inexistente. |
| 13. Critérios de aceitação | CA1: system retorna status e saída<br>CA2: clipboard copiar/colar texto |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F123 — Gerenciamento de segredos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F123 |
| 2. Nome conceitual | Gerenciamento de segredos |
| 3. Descrição | Armazenamento seguro de segredos (tokens, senhas) no cofre do sistema, com funções de definir/obter/remover e entrada mascarada. |
| 4. Necessidade atendida | Credenciais sem expor em código. |
| 5. Entradas | Nome do segredo, valor. |
| 6. Saídas | Segredos. |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | Cofre de credenciais do SO. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | Cofre indisponível (Linux sem serviço). |
| 12. Possíveis erros | Segredo inexistente. |
| 13. Critérios de aceitação | CA1: Segredo persiste entre sessões e não aparece em texto |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F124 — Funções criptográficas/hash internas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F124 |
| 2. Nome conceitual | Funções criptográficas/hash internas |
| 3. Descrição | Componente criptográfico presente; finalidade exposta ao usuário não determinada. |
| 4. Necessidade atendida | A determinar. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário chama função de leitura/escrita (ou ferramenta de importação) com caminho/opções → dados convertidos de/para estruturas do runtime. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Requisito a definir após investigação em documentação pública |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Desconhecida. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **NÃO VERIFICÁVEL**

## BIG — Dados grandes e processamento fora da memória

### F130 — Datastores

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F130 |
| 2. Nome conceitual | Datastores |
| 3. Descrição | Fontes de dados particionadas para coleções de arquivos (texto tabular, planilha, imagem, arquivo genérico com leitor personalizado, chave-valor, Parquet, conjuntos de arquivos, arrays em memória), combinadas/transformadas, particionamento, embaralhamento, escrita de saída. |
| 4. Necessidade atendida | Processar dados maiores que a memória. |
| 5. Entradas | Pastas/arquivos/opções. |
| 6. Saídas | Blocos de dados. |
| 7. Fluxo de utilização | Usuário define fonte de dados particionada → itera/agrega em blocos sem carregar tudo na memória → coleta resultado. |
| 8. Comportamento esperado | read/hasdata/reset/preview/readall; partições para paralelismo. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F112, F121 |
| 11. Casos especiais | Arquivos heterogêneos; remotos. |
| 12. Possíveis erros | Arquivo ilegível (pular/errar). |
| 13. Critérios de aceitação | CA1: Datastore de 1000 CSV itera em blocos<br>CA2: transform aplica função lazy<br>CA3: partition(n,i) cobre todos os arquivos |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F131 — Arrays altos (avaliação adiada) e MapReduce

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F131 |
| 2. Nome conceitual | Arrays altos (avaliação adiada) e MapReduce |
| 3. Descrição | Arrays 'altos' com operações adiadas e execução otimizada em passagens; framework MapReduce. |
| 4. Necessidade atendida | Análise de big data com sintaxe de arrays. |
| 5. Entradas | Datastore. |
| 6. Saídas | Resultados agregados. |
| 7. Fluxo de utilização | Usuário define fonte de dados particionada → itera/agrega em blocos sem carregar tudo na memória → coleta resultado. |
| 8. Comportamento esperado | Combinação de operações em poucas passagens; gather dispara execução. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F130 |
| 11. Casos especiais | Operações não suportadas em modo adiado (erro explícito). |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: mean de tall array igual ao mean em memória<br>CA2: Número de passagens minimizado |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

## NET — Rede, dispositivos e serviços

### F135 — Acesso web (HTTP/REST)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F135 |
| 2. Nome conceitual | Acesso web (HTTP/REST) |
| 3. Descrição | Funções de alto nível de leitura/escrita/download web com opções (tempo-limite, cabeçalhos, autenticação, certificados); API HTTP de baixo nível com mensagens, cabeçalhos, corpo, cookies, progresso, redirecionamentos, multipart; proxy. |
| 4. Necessidade atendida | Integração com serviços web. |
| 5. Entradas | URLs, opções. |
| 6. Saídas | Dados decodificados conforme tipo de conteúdo. |
| 7. Fluxo de utilização | Usuário configura conexão/endpoint → envia/recebe dados → trata respostas, tempo-limite e erros. |
| 8. Comportamento esperado | Conversão automática JSON/imagem/texto; validação de certificado configurável. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F114 |
| 11. Casos especiais | Respostas grandes (streaming); proxy com autenticação. |
| 12. Possíveis erros | Status HTTP de erro; tempo-limite; TLS inválido. |
| 13. Critérios de aceitação | CA1: webread JSON → struct<br>CA2: Upload multipart<br>CA3: Proxy configurável nas preferências |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F136 — FTP, e-mail e sockets

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F136 |
| 2. Nome conceitual | FTP, e-mail e sockets |
| 3. Descrição | Cliente FTP/SFTP, envio de e-mail SMTP, clientes/servidores TCP, UDP, WebSocket. |
| 4. Necessidade atendida | Comunicação com equipamentos e serviços. |
| 5. Entradas | Endereços, portas, dados. |
| 6. Saídas | Dados recebidos. |
| 7. Fluxo de utilização | Usuário configura conexão/endpoint → envia/recebe dados → trata respostas, tempo-limite e erros. |
| 8. Comportamento esperado | Callbacks por bytes disponíveis/terminador; tempo-limite. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | Conexão interrompida. |
| 12. Possíveis erros | Host inacessível. |
| 13. Critérios de aceitação | CA1: tcpclient eco local<br>CA2: sendmail via servidor de teste<br>CA3: SFTP listar/baixar |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F137 — Portas seriais e Bluetooth

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F137 |
| 2. Nome conceitual | Portas seriais e Bluetooth |
| 3. Descrição | Comunicação por portas seriais (configuração, terminadores, callbacks), descoberta de portas; Bluetooth/BLE. |
| 4. Necessidade atendida | Hardware de laboratório. |
| 5. Entradas | Porta, parâmetros. |
| 6. Saídas | Dados. |
| 7. Fluxo de utilização | Usuário configura conexão/endpoint → envia/recebe dados → trata respostas, tempo-limite e erros. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | Desconexão física. |
| 12. Possíveis erros | Porta ocupada. |
| 13. Critérios de aceitação | CA1: serialport loopback<br>CA2: Listar portas disponíveis |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | RXTX é LGPL — NÃO usar; crate serialport é MPL-2.0 — fora da política; implementação própria sobre termios/Win32. |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F138 — Sensores móveis e IoT

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F138 |
| 2. Nome conceitual | Sensores móveis e IoT |
| 3. Descrição | Aquisição de sensores de dispositivo móvel e envio/leitura de canais IoT em serviços de nuvem. |
| 4. Necessidade atendida | Experimentos com dispositivos. |
| 5. Entradas | Conta/dispositivo. |
| 6. Saídas | Dados de sensores. |
| 7. Fluxo de utilização | Usuário configura conexão/endpoint → envia/recebe dados → trata respostas, tempo-limite e erros. |
| 8. Comportamento esperado | Depende de serviços externos; oferecer integração com serviços abertos. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Requisito equivalente com protocolo aberto (MQTT/HTTP) a definir |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Depende de serviço ou infraestrutura de terceiros: não reproduzir acesso; propor serviço próprio. Avaliação jurídica pendente para qualquer integração. |

*Status:* **INVESTIGADA PARCIALMENTE**

### F139 — Gerenciador de hardware e pacotes de suporte

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F139 |
| 2. Nome conceitual | Gerenciador de hardware e pacotes de suporte |
| 3. Descrição | Descoberta e configuração de hardware e instalação de pacotes de suporte a dispositivos. |
| 4. Necessidade atendida | Integração com hardware. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário configura conexão/endpoint → envia/recebe dados → trata respostas, tempo-limite e erros. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F140 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Requisito a definir |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Hipótese. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

## EXT — Interoperabilidade com outras linguagens

### F140 — Funções compiladas externas (C/C++/Fortran)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F140 |
| 2. Nome conceitual | Funções compiladas externas (C/C++/Fortran) |
| 3. Descrição | API C/C++/Fortran para criar funções compiladas chamáveis pelo runtime, comando de compilação com detecção de compiladores, API de arrays de dados (C e C++ moderna). |
| 4. Necessidade atendida | Desempenho e reaproveitamento de código nativo. |
| 5. Entradas | Código-fonte nativo. |
| 6. Saídas | Bibliotecas carregáveis. |
| 7. Fluxo de utilização | Usuário compila/carrega componente externo ou chama a outra linguagem → dados convertidos na fronteira → resultado retorna ao runtime. |
| 8. Comportamento esperado | API própria estável e documentada; camada de compatibilidade de origem com API publicamente documentada do original é questão jurídica pendente. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F003 |
| 11. Casos especiais | Compiladores ausentes; ABI. |
| 12. Possíveis erros | Falha de compilação/carregamento. |
| 13. Critérios de aceitação | CA1: Compilar e chamar função C que soma arrays<br>CA2: Erro nativo não derruba o processo (isolamento opcional) |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Leitura/escrita de formatos de terceiros para interoperabilidade: só no escopo do DT-10, a partir de especificações e implementações independentes e permissivas. |

*Status:* **PLANEJADA**

### F141 — Motor embutível (chamar o runtime de outras linguagens)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F141 |
| 2. Nome conceitual | Motor embutível (chamar o runtime de outras linguagens) |
| 3. Descrição | APIs para iniciar/conectar ao runtime a partir de C, C++, Java, Python, .NET e Fortran; sessões compartilhadas; avaliação assíncrona. |
| 4. Necessidade atendida | Integração em aplicações existentes. |
| 5. Entradas | Chamadas de API. |
| 6. Saídas | Resultados convertidos. |
| 7. Fluxo de utilização | Usuário compila/carrega componente externo ou chama a outra linguagem → dados convertidos na fronteira → resultado retorna ao runtime. |
| 8. Comportamento esperado | Conversões de tipos documentadas por linguagem. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F001 |
| 11. Casos especiais | Múltiplas sessões; threads. |
| 12. Possíveis erros | Sessão perdida. |
| 13. Critérios de aceitação | CA1: Python inicia runtime, avalia e recebe array NumPy<br>CA2: Conectar a sessão compartilhada |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F142 — API de arquivos de dados para outras linguagens

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F142 |
| 2. Nome conceitual | API de arquivos de dados para outras linguagens |
| 3. Descrição | Biblioteca C/Fortran para ler/escrever arquivos de dados nativos fora do runtime. |
| 4. Necessidade atendida | Interoperabilidade de dados. |
| 5. Entradas | Arquivos. |
| 6. Saídas | Dados. |
| 7. Fluxo de utilização | Usuário compila/carrega componente externo ou chama a outra linguagem → dados convertidos na fronteira → resultado retorna ao runtime. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F110 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Programa C lê arquivo de dados escrito pelo runtime |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F143 — Interoperabilidade com Python

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F143 |
| 2. Nome conceitual | Interoperabilidade com Python |
| 3. Descrição | Chamar bibliotecas Python diretamente, executar código/arquivos Python, conversão automática de tipos (incl. arrays NumPy), seleção de ambiente/interpretador, execução em processo ou fora dele. |
| 4. Necessidade atendida | Ecossistema científico Python. |
| 5. Entradas | Módulos/expressões Python. |
| 6. Saídas | Objetos convertidos. |
| 7. Fluxo de utilização | Usuário compila/carrega componente externo ou chama a outra linguagem → dados convertidos na fronteira → resultado retorna ao runtime. |
| 8. Comportamento esperado | Conversão zero-cópia quando possível; ambiente configurável. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F003 |
| 11. Casos especiais | Versões múltiplas de Python; GIL; travamentos. |
| 12. Possíveis erros | Módulo inexistente. |
| 13. Critérios de aceitação | CA1: py.math.sqrt(4) == 2<br>CA2: ndarray ↔ array numérico sem cópia<br>CA3: Modo fora de processo isola falhas |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F144 — Interoperabilidade com Java

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F144 |
| 2. Nome conceitual | Interoperabilidade com Java |
| 3. Descrição | JVM embutida opcional: criar objetos Java, chamar métodos, conversão de tipos, caminho de classes dinâmico/estático. |
| 4. Necessidade atendida | Bibliotecas Java. |
| 5. Entradas | Classes Java. |
| 6. Saídas | Objetos. |
| 7. Fluxo de utilização | Usuário compila/carrega componente externo ou chama a outra linguagem → dados convertidos na fronteira → resultado retorna ao runtime. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | Runtime Java fornecido pelo usuário (não redistribuir JRE comercial). |
| 10. Dependências | F003 |
| 11. Casos especiais | — |
| 12. Possíveis erros | Classe inexistente. |
| 13. Critérios de aceitação | CA1: Criar java.util.ArrayList e adicionar elemento |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F145 — Interoperabilidade com .NET e COM

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F145 |
| 2. Nome conceitual | Interoperabilidade com .NET e COM |
| 3. Descrição | Carregar assemblies .NET, criar objetos, eventos; automação COM/ActiveX (Windows) incluindo controle de aplicativos de escritório; servidor de automação. |
| 4. Necessidade atendida | Integração Windows/corporativa. |
| 5. Entradas | Assemblies/ProgIDs. |
| 6. Saídas | Objetos. |
| 7. Fluxo de utilização | Usuário compila/carrega componente externo ou chama a outra linguagem → dados convertidos na fronteira → resultado retorna ao runtime. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | Somente Windows para COM. |
| 10. Dependências | — |
| 11. Casos especiais | — |
| 12. Possíveis erros | Assembly inexistente. |
| 13. Critérios de aceitação | CA1: Chamar System.DateTime.Now<br>CA2: Automação COM de planilha (se instalada) |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F146 — Bibliotecas compartilhadas C e interfaces C++ geradas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F146 |
| 2. Nome conceitual | Bibliotecas compartilhadas C e interfaces C++ geradas |
| 3. Descrição | Carregar bibliotecas C via cabeçalho e chamar funções; gerar interface para bibliotecas C++ a partir de cabeçalhos. |
| 4. Necessidade atendida | Reuso de bibliotecas nativas sem escrever wrappers. |
| 5. Entradas | Cabeçalhos, bibliotecas. |
| 6. Saídas | Interfaces. |
| 7. Fluxo de utilização | Usuário compila/carrega componente externo ou chama a outra linguagem → dados convertidos na fronteira → resultado retorna ao runtime. |
| 8. Comportamento esperado | Análise de cabeçalhos C/C++. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F140 |
| 11. Casos especiais | Templates; ponteiros; callbacks. |
| 12. Possíveis erros | Tipo não suportado. |
| 13. Critérios de aceitação | CA1: loadlibrary + calllib em libm<br>CA2: Interface C++ gerada para classe simples |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

## PAR — Concorrência e desempenho

### F150 — Multithreading implícito

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F150 |
| 2. Nome conceitual | Multithreading implícito |
| 3. Descrição | Paralelização automática de operações elementares e álgebra linear em múltiplos núcleos, com controle do número de threads. |
| 4. Necessidade atendida | Desempenho sem esforço. |
| 5. Entradas | Operações. |
| 6. Saídas | Mesmos resultados, mais rápido. |
| 7. Fluxo de utilização | Usuário dispara execução concorrente/assíncrona → tarefas executam → resultados coletados ou cancelados. |
| 8. Comportamento esperado | Limiar de tamanho para paralelizar; maxNumCompThreads. |
| 9. Regras de negócio observáveis | Resultados determinísticos (ou documentar diferenças de arredondamento). |
| 10. Dependências | F005 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Speedup ≥ 3x em 4 núcleos para soma elementar de 1e8<br>CA2: Resultados idênticos com 1 e N threads (onde garantido) |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F151 — Execução em segundo plano e futures

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F151 |
| 2. Nome conceitual | Execução em segundo plano e futures |
| 3. Descrição | Pool de segundo plano para avaliar funções assíncronas, futures com estado, cancelamento, callbacks de conclusão (afterEach/afterAll), filas de dados entre tarefas. |
| 4. Necessidade atendida | Interfaces responsivas e paralelismo leve. |
| 5. Entradas | Funções. |
| 6. Saídas | Futures. |
| 7. Fluxo de utilização | Usuário dispara execução concorrente/assíncrona → tarefas executam → resultados coletados ou cancelados. |
| 8. Comportamento esperado | Execução em threads do runtime; resultados obtidos sob demanda. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F016 |
| 11. Casos especiais | Erro em tarefa; cancelamento. |
| 12. Possíveis erros | Erro propagado ao obter resultado. |
| 13. Critérios de aceitação | CA1: parfeval no pool de fundo não bloqueia UI<br>CA2: cancel interrompe tarefa |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F152 — Comunicação entre processos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F152 |
| 2. Nome conceitual | Comunicação entre processos |
| 3. Descrição | Comunicação entre o runtime e outros processos (canais internos, sessões compartilhadas). |
| 4. Necessidade atendida | Arquitetura multiprocessos. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário dispara execução concorrente/assíncrona → tarefas executam → resultados coletados ou cancelados. |
| 8. Comportamento esperado | Inferência a partir de componentes 'interprocess'. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Protocolo próprio documentado |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Hipótese. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

## TST — Testes, qualidade e automação de build

### F155 — Framework de testes unitários

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F155 |
| 2. Nome conceitual | Framework de testes unitários |
| 3. Descrição | Testes baseados em scripts, funções e classes; asserções/verificações/suposições/fatais; fixtures (configuração/limpeza, compartilhadas); parametrização; tags; seleção; executores com plugins (TAP, JUnit XML, cobertura, diagnóstico); execução paralela; navegador de testes. |
| 4. Necessidade atendida | Qualidade de software. |
| 5. Entradas | Arquivos de teste. |
| 6. Saídas | Resultados de teste; relatórios. |
| 7. Fluxo de utilização | Usuário escreve testes/tarefas → executa o executor → recebe relatório (texto, JUnit, cobertura). |
| 8. Comportamento esperado | Diagnósticos ricos em falhas; resultados tabuláveis. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F007 |
| 11. Casos especiais | Testes que alteram estado global; dependências. |
| 12. Possíveis erros | Teste com erro vs falha distintos. |
| 13. Critérios de aceitação | CA1: Classe de teste parametrizada gera N casos<br>CA2: Relatório JUnit válido<br>CA3: Filtro por tag |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F156 — Mocks e testes de desempenho

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F156 |
| 2. Nome conceitual | Mocks e testes de desempenho |
| 3. Descrição | Criação de objetos simulados com comportamento definido e verificação de interações; framework de medição de desempenho com amostragem estatística. |
| 4. Necessidade atendida | Testes isolados e regressão de desempenho. |
| 5. Entradas | Interfaces/classes. |
| 6. Saídas | Mocks; métricas. |
| 7. Fluxo de utilização | Usuário escreve testes/tarefas → executa o executor → recebe relatório (texto, JUnit, cobertura). |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F155 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Mock registra chamadas<br>CA2: Medição com intervalo de confiança |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F157 — Testes de interface de aplicativos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F157 |
| 2. Nome conceitual | Testes de interface de aplicativos |
| 3. Descrição | Gestos programáticos (pressionar, digitar, escolher, arrastar) em componentes de apps para testes automatizados. |
| 4. Necessidade atendida | Testar apps. |
| 5. Entradas | Apps. |
| 6. Saídas | Resultados. |
| 7. Fluxo de utilização | Usuário escreve testes/tarefas → executa o executor → recebe relatório (texto, JUnit, cobertura). |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F075, F155 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Teste pressiona botão e verifica rótulo |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F158 — Cobertura de código

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F158 |
| 2. Nome conceitual | Cobertura de código |
| 3. Descrição | Cobertura por função/linha/decisão/condição, relatórios HTML e Cobertura XML. |
| 4. Necessidade atendida | Medir qualidade dos testes. |
| 5. Entradas | Código, testes. |
| 6. Saídas | Relatórios. |
| 7. Fluxo de utilização | Usuário escreve testes/tarefas → executa o executor → recebe relatório (texto, JUnit, cobertura). |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F155 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Relatório Cobertura XML válido |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F159 — Ferramenta de build

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F159 |
| 2. Nome conceitual | Ferramenta de build |
| 3. Descrição | Arquivo de build com tarefas, dependências, entradas/saídas para build incremental, tarefas padrão (análise de código, testes, limpeza, empacotamento), execução por linha de comando. |
| 4. Necessidade atendida | Automação de CI. |
| 5. Entradas | Arquivo de build. |
| 6. Saídas | Execução de tarefas. |
| 7. Fluxo de utilização | Usuário escreve testes/tarefas → executa o executor → recebe relatório (texto, JUnit, cobertura). |
| 8. Comportamento esperado | Pula tarefas atualizadas. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F155, F100 |
| 11. Casos especiais | Ciclos de dependência. |
| 12. Possíveis erros | Tarefa inexistente. |
| 13. Critérios de aceitação | CA1: Segunda execução pula tarefas inalteradas<br>CA2: Falha de tarefa retorna código ≠ 0 |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F160 — Automação e diagnóstico de execução

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F160 |
| 2. Nome conceitual | Automação e diagnóstico de execução |
| 3. Descrição | Infraestrutura de relatórios/diagnósticos de automação compartilhada por testes e build. |
| 4. Necessidade atendida | — |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário escreve testes/tarefas → executa o executor → recebe relatório (texto, JUnit, cobertura). |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F159 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Requisito a definir |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Hipótese. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

## PKG — Pacotes, extensões e distribuição

### F165 — Empacotamento de extensões

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F165 |
| 2. Nome conceitual | Empacotamento de extensões |
| 3. Descrição | Empacotar código, apps, documentação e exemplos em arquivo de extensão instalável com metadados, versão, dependências e compatibilidade de plataforma. |
| 4. Necessidade atendida | Distribuir bibliotecas. |
| 5. Entradas | Pasta de projeto. |
| 6. Saídas | Pacote. |
| 7. Fluxo de utilização | Usuário empacota, publica, instala ou remove extensão/pacote → dependências resolvidas → caminho atualizado. |
| 8. Comportamento esperado | Instalação adiciona ao caminho; desinstalação remove. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F014 |
| 11. Casos especiais | Conflitos de versão. |
| 12. Possíveis erros | Metadados inválidos. |
| 13. Critérios de aceitação | CA1: Empacotar, instalar, desinstalar<br>CA2: Dependência ausente reportada |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F166 — Gerenciador de pacotes

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F166 |
| 2. Nome conceitual | Gerenciador de pacotes |
| 3. Descrição | Instalar/atualizar/remover pacotes a partir de repositórios, resolução de dependências por versão semântica, repositórios locais/remotos, lockfile. |
| 4. Necessidade atendida | Ecossistema de bibliotecas. |
| 5. Entradas | Nomes/versões. |
| 6. Saídas | Pacotes instalados. |
| 7. Fluxo de utilização | Usuário empacota, publica, instala ou remove extensão/pacote → dependências resolvidas → caminho atualizado. |
| 8. Comportamento esperado | Resolução determinística. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F165 |
| 11. Casos especiais | Conflitos insolúveis. |
| 12. Possíveis erros | Pacote inexistente. |
| 13. Critérios de aceitação | CA1: Instalar pacote com dependências transitivas<br>CA2: Lockfile reproduz ambiente |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F167 — Explorador de extensões (catálogo online)

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F167 |
| 2. Nome conceitual | Explorador de extensões (catálogo online) |
| 3. Descrição | Navegar, buscar e instalar extensões de catálogo comunitário/oficial. |
| 4. Necessidade atendida | Descoberta de extensões. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário empacota, publica, instala ou remove extensão/pacote → dependências resolvidas → caminho atualizado. |
| 8. Comportamento esperado | Depende de serviço externo — equivalente próprio. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F166 |
| 11. Casos especiais | Offline. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Catálogo próprio navegável e instalável |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Depende de serviço ou infraestrutura de terceiros: não reproduzir acesso; propor serviço próprio. Avaliação jurídica pendente para qualquer integração. |

*Status:* **PLANEJADA**

### F168 — Verificação de compatibilidade de versões

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F168 |
| 2. Nome conceitual | Verificação de compatibilidade de versões |
| 3. Descrição | Análise de código para identificar usos de funcionalidades removidas/alteradas entre versões e sugerir atualizações. |
| 4. Necessidade atendida | Migração de código. |
| 5. Entradas | Código. |
| 6. Saídas | Relatório. |
| 7. Fluxo de utilização | Usuário empacota, publica, instala ou remove extensão/pacote → dependências resolvidas → caminho atualizado. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F087 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Relatório lista funções obsoletas |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

## INS — Instalação, atualização, conta e telemetria

### F170 — Instalador interativo e silencioso

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F170 |
| 2. Nome conceitual | Instalador interativo e silencioso |
| 3. Descrição | Instalação com seleção de produtos/componentes, pasta de destino, atalhos; modo silencioso com arquivo de respostas; instalação a partir de mídia offline; verificação de integridade de pacotes. |
| 4. Necessidade atendida | Implantação em máquinas e laboratórios. |
| 5. Entradas | Seleções/arquivo de respostas. |
| 6. Saídas | Instalação. |
| 7. Fluxo de utilização | Usuário instala/atualiza/remove o produto (interativo ou silencioso) → componentes selecionados gravados/verificados. |
| 8. Comportamento esperado | Pacotes assinados verificados; dependências entre componentes resolvidas. |
| 9. Regras de negócio observáveis | Instalação silenciosa por arquivo de respostas; pacotes assinados. |
| 10. Dependências | — |
| 11. Casos especiais | Espaço insuficiente; instalação parcial (rollback). |
| 12. Possíveis erros | Assinatura inválida. |
| 13. Critérios de aceitação | CA1: Instalação silenciosa com arquivo de respostas<br>CA2: Pacote adulterado rejeitado<br>CA3: Remover componente individual |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Inferência fundamentada. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F171 — Atualizações

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F171 |
| 2. Nome conceitual | Atualizações |
| 3. Descrição | Verificação, download e aplicação de atualizações e correções com reversão. |
| 4. Necessidade atendida | Manter software seguro. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário instala/atualiza/remove o produto (interativo ou silencioso) → componentes selecionados gravados/verificados. |
| 8. Comportamento esperado | Canal de atualização próprio assinado. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F170 |
| 11. Casos especiais | Offline. |
| 12. Possíveis erros | Falha de download. |
| 13. Critérios de aceitação | CA1: Atualização assinada aplicada e revertida |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F172 — Desinstalação

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F172 |
| 2. Nome conceitual | Desinstalação |
| 3. Descrição | Remoção completa ou por componente. |
| 4. Necessidade atendida | Higiene do sistema. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário instala/atualiza/remove o produto (interativo ou silencioso) → componentes selecionados gravados/verificados. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F170 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Desinstalar remove arquivos e atalhos |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F173 — Licenciamento e ativação

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F173 |
| 2. Nome conceitual | Licenciamento e ativação |
| 3. Descrição | Em produtos comerciais: ativação, servidor de licenças em rede, verificação de licença por produto. |
| 4. Necessidade atendida | Em produtos comerciais: controle comercial. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário instala/atualiza/remove o produto (interativo ou silencioso) → componentes selecionados gravados/verificados. |
| 8. Comportamento esperado | Decisão: NÃO reproduzir mecanismo; projeto aberto sem ativação. Investigação NÃO tentou contornar o mecanismo. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Aplicação funciona sem ativação (decisão de projeto) |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Inferência fundamentada. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Depende de serviço ou infraestrutura de terceiros: não reproduzir acesso; propor serviço próprio. Avaliação jurídica pendente para qualquer integração. |

*Status:* **PLANEJADA**

### F174 — Conta de usuário e login

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F174 |
| 2. Nome conceitual | Conta de usuário e login |
| 3. Descrição | Login em conta para serviços online. |
| 4. Necessidade atendida | Acesso a serviços online. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário instala/atualiza/remove o produto (interativo ou silencioso) → componentes selecionados gravados/verificados. |
| 8. Comportamento esperado | Equivalente próprio opcional (contas apenas para serviços online do projeto). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Uso offline completo sem conta |
| 14. Serviço externo | Depende de serviço externo. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Depende de serviço ou infraestrutura de terceiros: não reproduzir acesso; propor serviço próprio. Avaliação jurídica pendente para qualquer integração. |

*Status:* **PLANEJADA**

### F175 — Telemetria de uso e relatório de falhas

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F175 |
| 2. Nome conceitual | Telemetria de uso e relatório de falhas |
| 3. Descrição | Coleta de uso e relatórios de travamento. |
| 4. Necessidade atendida | Melhoria do produto. |
| 5. Entradas | Consentimento. |
| 6. Saídas | Relatórios. |
| 7. Fluxo de utilização | Usuário instala/atualiza/remove o produto (interativo ou silencioso) → componentes selecionados gravados/verificados. |
| 8. Comportamento esperado | Novo projeto: telemetria DESLIGADA por padrão, opt-in explícito; relatórios de falha locais com envio manual. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Nenhum tráfego de rede sem consentimento<br>CA2: Relatório de falha salvo localmente |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

## LOC — Ajuda, documentação e localização

### F180 — Ajuda em linha de comando e navegador de documentação

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F180 |
| 2. Nome conceitual | Ajuda em linha de comando e navegador de documentação |
| 3. Descrição | Ajuda a partir de comentários de código, documentação de referência navegável e pesquisável offline/online, exemplos executáveis, 'ver também', documentação de extensões de terceiros integrada. |
| 4. Necessidade atendida | Aprendizado e consulta. |
| 5. Entradas | Consultas. |
| 6. Saídas | Páginas. |
| 7. Fluxo de utilização | Usuário consulta ajuda/documentação ou troca idioma → conteúdo localizado exibido. |
| 8. Comportamento esperado | Documentação própria escrita do zero (NÃO traduzir/parafrasear documentação do original). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F085 |
| 11. Casos especiais | Documentação offline instalada como componente. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: help f mostra comentário inicial de f<br>CA2: Busca offline retorna resultados em < 300 ms<br>CA3: Exemplo abre e executa |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F181 — Localização da interface

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F181 |
| 2. Nome conceitual | Localização da interface |
| 3. Descrição | Interface e mensagens traduzidas (meta: pt-BR e inglês na v1, outros idiomas depois); formatação dependente de locale. |
| 4. Necessidade atendida | Usuários internacionais. |
| 5. Entradas | Configuração de idioma. |
| 6. Saídas | UI localizada. |
| 7. Fluxo de utilização | Usuário consulta ajuda/documentação ou troca idioma → conteúdo localizado exibido. |
| 8. Comportamento esperado | Novo projeto: pt-BR e en como mínimo; arquitetura para outros idiomas. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | — |
| 11. Casos especiais | Texto mais longo que layout. |
| 12. Possíveis erros | Recurso ausente → fallback inglês. |
| 13. Critérios de aceitação | CA1: Troca de idioma sem reinstalar<br>CA2: Fallback para chave ausente |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F182 — Exemplos e materiais de aprendizado

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F182 |
| 2. Nome conceitual | Exemplos e materiais de aprendizado |
| 3. Descrição | Exemplos executáveis e cursos/tutoriais interativos integrados (componentes 'academy', 'grader'). |
| 4. Necessidade atendida | Aprendizado. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário consulta ajuda/documentação ou troca idioma → conteúdo localizado exibido. |
| 8. Comportamento esperado | Conteúdo próprio. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F180 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Exemplos próprios abrem e executam |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

## SIM — Modelagem e simulação por diagramas de blocos

### F200 — Editor de diagramas de blocos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F200 |
| 2. Nome conceitual | Editor de diagramas de blocos |
| 3. Descrição | Canvas para modelos hierárquicos: inserir blocos, conectar portas com roteamento automático e manual, ramificações, rótulos de sinais, anotações (texto/imagem/equações), áreas, zoom/pan, desfazer/refazer, copiar/colar entre modelos, navegação hierárquica com trilha, alinhamento/distribuição, busca rápida de blocos digitando no canvas. |
| 4. Necessidade atendida | Modelagem visual de sistemas dinâmicos. |
| 5. Entradas | Interações de usuário. |
| 6. Saídas | Modelo editado. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Roteamento evita sobreposição; conexões inválidas destacadas. |
| 9. Regras de negócio observáveis | Aparência visual própria (não reproduzir ícones/estilo do original). |
| 10. Dependências | F202 |
| 11. Casos especiais | Modelos com 1e4 blocos; desfazer extenso. |
| 12. Possíveis erros | Conexão de tipos incompatíveis (diagnóstico na atualização). |
| 13. Critérios de aceitação | CA1: Modelo com 1000 blocos edita a 60 fps<br>CA2: Desfazer/refazer ilimitado na sessão<br>CA3: Copiar subsistema entre modelos preserva parâmetros |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | Expressão visual e ícones de blocos do original são protegidos — biblioteca visual própria. |
| 18. Restrições jurídicas | Possível patente/técnica proprietária não verificada: busca de patentes PENDENTE; usar algoritmos publicados. |

*Status:* **PLANEJADA**

### F201 — Biblioteca de blocos padrão

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F201 |
| 2. Nome conceitual | Biblioteca de blocos padrão |
| 3. Descrição | Fontes (constante, degrau, rampa, seno, pulso, relógio, ruído, de workspace/arquivo, sinal de entrada), sumidouros (scope, display, para workspace/arquivo, terminator, stop), contínuos (integrador com limites/reset, derivada, função de transferência, espaço de estados, zero-polo, atraso de transporte, PID), descontinuidades (saturação, zona morta, relé, quantizador, backlash, rate limiter), discretos (atraso unitário, integrador discreto, filtro, FT discreta, ZOH, memória), operações matemáticas (soma, ganho, produto, funções, trigonometria, mínimo/máximo, matriz, polinômio), lógica e bits, tabelas de busca 1-D/2-D/N-D, roteamento (mux/demux, bus, switch, multiport switch, selector, goto/from, merge), atributos de sinal (conversão de tipo, IC, rate transition, largura), funções definidas pelo usuário, verificação de modelo (asserções), utilitários de modelo, strings, mensagens. |
| 4. Necessidade atendida | Construir modelos sem programar cada bloco. |
| 5. Entradas | Seleção de blocos. |
| 6. Saídas | Blocos com parâmetros. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Cada bloco tem semântica de tempo, tipos e dimensões documentada pelo projeto. |
| 9. Regras de negócio observáveis | Parâmetros avaliados como expressões do runtime. |
| 10. Dependências | F202, F203 |
| 11. Casos especiais | Parâmetros variáveis no tempo (tunáveis); blocos com estados. |
| 12. Possíveis erros | Parâmetro inválido. |
| 13. Critérios de aceitação | CA1: Cada bloco tem teste de referência contra solução analítica/numérica independente<br>CA2: Integrador com reset externo<br>CA3: Lookup N-D com extrapolação configurável |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F202 — Motor de simulação

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F202 |
| 2. Nome conceitual | Motor de simulação |
| 3. Descrição | Compilação do modelo (ordenação de blocos, propagação de tempos de amostragem, tipos, dimensões, complexidade), simulação contínua/discreta/híbrida multitaxa, solvers de passo fixo e variável (explícitos e rígidos), detecção e localização de cruzamentos por zero, laços algébricos (detecção e solução), tarefas única/multitarefa, tempos de amostragem herdados/constantes/disparados. |
| 4. Necessidade atendida | Resultados de simulação corretos e rápidos. |
| 5. Entradas | Modelo, configuração. |
| 6. Saídas | Sinais no tempo, estados. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Ordem de execução determinística; diagnóstico configurável de laços algébricos e transições de taxa. |
| 9. Regras de negócio observáveis | Passo mínimo/máximo; tolerâncias. |
| 10. Dependências | F049 |
| 11. Casos especiais | Sistemas rígidos; descontinuidades; múltiplas taxas. |
| 12. Possíveis erros | Passo mínimo violado; laço algébrico insolúvel. |
| 13. Critérios de aceitação | CA1: Modelo massa-mola-amortecedor coincide com solução analítica (tol)<br>CA2: Cruzamento por zero de bola quicando com erro < 1e-6<br>CA3: Sistema multitaxa com transições corretas |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F203 — Atributos de sinal: tipos, dimensões, unidades e barramentos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F203 |
| 2. Nome conceitual | Atributos de sinal: tipos, dimensões, unidades e barramentos |
| 3. Descrição | Tipos numéricos e de ponto fixo, enumerações, dimensões (escalar, vetor, matriz, N-D, tamanho variável), complexidade, unidades físicas com verificação/conversão, barramentos (virtuais e não virtuais) com objetos de definição e editor, portas de elementos de barramento, exibição de tipos/dimensões nas linhas. |
| 4. Necessidade atendida | Modelos corretos e verificáveis. |
| 5. Entradas | Definições de tipos/barramentos. |
| 6. Saídas | Sinais tipados. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Propagação de atributos para frente e para trás; conflitos reportados. |
| 9. Regras de negócio observáveis | Unidades incompatíveis geram aviso/erro conforme configuração. |
| 10. Dependências | F202 |
| 11. Casos especiais | Barramentos aninhados; arrays de barramentos. |
| 12. Possíveis erros | Incompatibilidade de tipo/dimensão. |
| 13. Critérios de aceitação | CA1: Propagação de tipos em cadeia<br>CA2: Conversão automática de unidades onde permitido<br>CA3: Editor de barramentos cria/edita definições |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F204 — Dados e parâmetros de modelo

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F204 |
| 2. Nome conceitual | Dados e parâmetros de modelo |
| 3. Descrição | Workspaces de modelo/base, dicionários de dados (arquivo próprio), objetos de parâmetro/sinal, parâmetros ajustáveis durante simulação, editor de dados do modelo, explorador de modelos, inspetor de propriedades. |
| 4. Necessidade atendida | Gestão de dados de engenharia. |
| 5. Entradas | Variáveis/objetos. |
| 6. Saídas | Dados resolvidos. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Ordem de resolução de símbolos documentada. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | Símbolos duplicados em fontes diferentes. |
| 12. Possíveis erros | Símbolo não resolvido. |
| 13. Critérios de aceitação | CA1: Parâmetro em dicionário usado por vários modelos<br>CA2: Ajuste de ganho durante simulação reflete imediatamente |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Leitura/escrita de formatos de terceiros para interoperabilidade: só no escopo do DT-10, a partir de especificações e implementações independentes e permissivas. |

*Status:* **PLANEJADA**

### F205 — Subsistemas e execução condicional

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F205 |
| 2. Nome conceitual | Subsistemas e execução condicional |
| 3. Descrição | Subsistemas virtuais e atômicos, habilitados, disparados, chamada de função, ação if/switch-case, iteradores for/while, for-each, subsistemas reutilizáveis/referenciados, reset. |
| 4. Necessidade atendida | Modularidade e lógica condicional. |
| 5. Entradas | Blocos de subsistema. |
| 6. Saídas | Hierarquia. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Semântica de estados ao desabilitar (manter/resetar). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | Subsistemas aninhados profundos. |
| 12. Possíveis erros | Configuração inconsistente. |
| 13. Critérios de aceitação | CA1: Subsistema habilitado mantém/reseta estados conforme opção<br>CA2: For-each processa vetor elemento a elemento |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F206 — Máscaras e blocos personalizados

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F206 |
| 2. Nome conceitual | Máscaras e blocos personalizados |
| 3. Descrição | Editor de máscaras (parâmetros, diálogos, validações, ícones desenhados por comandos/imagens, código de inicialização), máscaras dinâmicas, bibliotecas personalizadas com links e controle de versão de links. |
| 4. Necessidade atendida | Encapsular e reutilizar componentes. |
| 5. Entradas | Definições de máscara. |
| 6. Saídas | Blocos personalizados. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Links de biblioteca propagam alterações; links desativados/restaurados. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F200 |
| 11. Casos especiais | Bibliotecas modificadas com instâncias abertas. |
| 12. Possíveis erros | Parâmetro de máscara inválido. |
| 13. Critérios de aceitação | CA1: Máscara com parâmetro validado<br>CA2: Atualizar bloco de biblioteca reflete nas instâncias ligadas |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F207 — Referência de modelos e modelos protegidos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F207 |
| 2. Nome conceitual | Referência de modelos e modelos protegidos |
| 3. Descrição | Usar modelos como blocos dentro de outros, modos de simulação por referência, arquivos de cache de compilação, modelos protegidos (distribuição com proteção de IP: só simulação/código). |
| 4. Necessidade atendida | Escalabilidade de equipes. |
| 5. Entradas | Modelos. |
| 6. Saídas | Hierarquia de modelos. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Recompilação incremental baseada em verificação de alterações. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202, F216 |
| 11. Casos especiais | Referências circulares; múltiplas instâncias. |
| 12. Possíveis erros | Modelo referenciado ausente. |
| 13. Critérios de aceitação | CA1: Alterar modelo referenciado invalida apenas seu cache<br>CA2: Modelo protegido simula sem expor conteúdo |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F208 — Variantes

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F208 |
| 2. Nome conceitual | Variantes |
| 3. Descrição | Subsistemas/fontes/sumidouros variantes com condições, ativação em tempo de atualização/código/arranque, gerenciador de variantes. |
| 4. Necessidade atendida | Linhas de produto e configurações. |
| 5. Entradas | Condições de variante. |
| 6. Saídas | Modelo ativo. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Apenas variante ativa compilada (conforme modo). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F205 |
| 11. Casos especiais | Variantes aninhadas. |
| 12. Possíveis erros | Nenhuma variante ativa. |
| 13. Critérios de aceitação | CA1: Troca de variante muda resultado da simulação |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F209 — Blocos definidos pelo usuário

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F209 |
| 2. Nome conceitual | Blocos definidos pelo usuário |
| 3. Descrição | Bloco de função em linguagem do runtime, blocos de sistema baseados em objeto, funções S em C e em linguagem do runtime, chamada direta de código C (C caller), ferramenta de integração de código legado. |
| 4. Necessidade atendida | Estender a biblioteca. |
| 5. Entradas | Código do usuário. |
| 6. Saídas | Blocos. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Interface de estados, saídas, derivadas e tempos de amostragem própria e documentada. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202, F140 |
| 11. Casos especiais | Código C com falha (isolamento). |
| 12. Possíveis erros | Erro de compilação. |
| 13. Critérios de aceitação | CA1: Bloco de função compila e simula<br>CA2: Função C integrada via C caller |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Leitura/escrita de formatos de terceiros para interoperabilidade: só no escopo do DT-10, a partir de especificações e implementações independentes e permissivas. |

*Status:* **PLANEJADA**

### F210 — Visualização e registro de sinais

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F210 |
| 2. Nome conceitual | Visualização e registro de sinais |
| 3. Descrição | Scope (múltiplos eixos, cursores, medições, triggers), display, painel de controle interativo (botões, knobs, medidores, lâmpadas ligados a parâmetros/sinais), registro de sinais, inspetor de dados de simulação (comparar execuções, tolerâncias, exportar), formato de dados de saída estruturado. |
| 4. Necessidade atendida | Analisar resultados. |
| 5. Entradas | Sinais. |
| 6. Saídas | Visualizações, conjuntos de dados. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Registro sem blocos adicionais; streaming durante simulação. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202, F061 |
| 11. Casos especiais | Simulações longas (decimação). |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Comparar duas execuções com tolerância relativa<br>CA2: Scope atualiza durante simulação<br>CA3: Knob altera parâmetro em tempo de simulação |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F211 — Entrada de dados e cenários de teste

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F211 |
| 2. Nome conceitual | Entrada de dados e cenários de teste |
| 3. Descrição | Importar sinais de workspace/arquivo para portas raiz, editor de sinais, mapeamento de entradas. |
| 4. Necessidade atendida | Estímulos reprodutíveis. |
| 5. Entradas | Dados de sinal. |
| 6. Saídas | Entradas mapeadas. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Interpolação/extrapolação configuráveis. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | Tempos não monotônicos. |
| 12. Possíveis erros | Dimensão incompatível. |
| 13. Critérios de aceitação | CA1: Mapear dataset para portas por nome<br>CA2: Editor cria sinal de degrau |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F212 — Controle programático de simulação

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F212 |
| 2. Nome conceitual | Controle programático de simulação |
| 3. Descrição | Iniciar/pausar/continuar/parar, simulação por API com objetos de entrada/saída, simulações em lote e paralelas, reinício rápido sem recompilar, passo-a-passo e retrocesso, estados operacionais (salvar/restaurar). |
| 4. Necessidade atendida | Automação, otimização e varreduras. |
| 5. Entradas | Objetos de entrada. |
| 6. Saídas | Objetos de saída. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Saída consolidada com metadados e diagnósticos. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202, F151 |
| 11. Casos especiais | Erro em uma simulação de um lote. |
| 12. Possíveis erros | Parâmetro inexistente. |
| 13. Critérios de aceitação | CA1: Lote de 100 simulações paralelas com resultados indexados<br>CA2: Fast restart evita recompilação<br>CA3: Retroceder passos restaura estado |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F213 — Modos de simulação acelerados

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F213 |
| 2. Nome conceitual | Modos de simulação acelerados |
| 3. Descrição | Modos com geração e compilação de código para acelerar simulação, com e sem interface de depuração. |
| 4. Necessidade atendida | Desempenho em modelos grandes. |
| 5. Entradas | Modelo. |
| 6. Saídas | Executável/biblioteca de simulação. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Mesmos resultados do modo normal (dentro de tolerância). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202, F140 |
| 11. Casos especiais | Blocos não suportados no modo. |
| 12. Possíveis erros | Compilador ausente. |
| 13. Critérios de aceitação | CA1: Speedup ≥ 5x em modelo de referência<br>CA2: Resultados equivalentes ao modo normal |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F214 — Depuração e diagnóstico de modelos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F214 |
| 2. Nome conceitual | Depuração e diagnóstico de modelos |
| 3. Descrição | Depurador de simulação (pontos de parada em sinais/blocos), visualizador de diagnósticos, verificação em tempo de edição, perfilador de solver, consultor de desempenho, exibição de ordem de execução e cores/legenda de tempos de amostragem, consultor de modelo com verificações básicas. |
| 4. Necessidade atendida | Encontrar problemas em modelos. |
| 5. Entradas | Modelo. |
| 6. Saídas | Diagnósticos. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Mensagens com links para blocos. |
| 9. Regras de negócio observáveis | Textos de verificação próprios. |
| 10. Dependências | F202 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Ponto de parada condicional em sinal<br>CA2: Legenda de tempos de amostragem<br>CA3: Diagnóstico clicável destaca bloco |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F215 — Configuração de modelo

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F215 |
| 2. Nome conceitual | Configuração de modelo |
| 3. Descrição | Conjuntos de configuração: solver, importação/exportação de dados, otimização, diagnósticos, implementação de hardware (tamanhos de tipos, endianness), referências de configuração compartilhadas. |
| 4. Necessidade atendida | Controle de execução. |
| 5. Entradas | Parâmetros. |
| 6. Saídas | Configuração. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Configurações referenciáveis entre modelos. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | — |
| 12. Possíveis erros | Valor inválido. |
| 13. Critérios de aceitação | CA1: Configuração referenciada compartilhada entre 2 modelos |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F216 — Formato de arquivo de modelo

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F216 |
| 2. Nome conceitual | Formato de arquivo de modelo |
| 3. Descrição | Formato de arquivo próprio aberto (ZIP + JSON/XML documentado) para modelos e bibliotecas; carregamento parcial; recuperação automática (autosave); exportação para versões anteriores do próprio formato. |
| 4. Necessidade atendida | Persistência, diff e controle de versão. |
| 5. Entradas | Modelos. |
| 6. Saídas | Arquivos. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Formato estável versionado e documentado. |
| 9. Regras de negócio observáveis | Leitura de formatos de modelo de terceiros apenas como importador, sujeita a avaliação jurídica (fora do escopo atual, DT-10). |
| 10. Dependências | F200 |
| 11. Casos especiais | Arquivos grandes; autosave após falha. |
| 12. Possíveis erros | Arquivo corrompido. |
| 13. Critérios de aceitação | CA1: Salvar/abrir ida-e-volta sem perda<br>CA2: Autosave recupera após encerramento forçado<br>CA3: Importador experimental de formato de terceiros (se aprovado) |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Leitura/escrita de formatos de terceiros para interoperabilidade: só no escopo do DT-10, a partir de especificações e implementações independentes e permissivas. |

*Status:* **PLANEJADA**

### F217 — Comparação e mesclagem de modelos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F217 |
| 2. Nome conceitual | Comparação e mesclagem de modelos |
| 3. Descrição | Diff estrutural de modelos (blocos, linhas, parâmetros, configuração, workspaces), mesclagem de três vias, relatório. |
| 4. Necessidade atendida | Revisão de modelos em equipe. |
| 5. Entradas | Dois/três modelos. |
| 6. Saídas | Relatório/merge. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F096, F216 |
| 11. Casos especiais | Modelos grandes. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Diff identifica bloco com parâmetro alterado<br>CA2: Merge 3-vias de alterações não conflitantes |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F218 — API programática de construção de modelos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F218 |
| 2. Nome conceitual | API programática de construção de modelos |
| 3. Descrição | Criar/abrir/salvar modelos, adicionar/remover blocos e linhas, obter/definir parâmetros, buscar sistema, layout automático, callbacks de modelo e bloco (pré-carga, inicialização, início, término, etc.). |
| 4. Necessidade atendida | Automação de modelagem. |
| 5. Entradas | Comandos. |
| 6. Saídas | Modelos. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Callbacks em pontos definidos do ciclo de vida. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F200, F202 |
| 11. Casos especiais | Callback com erro. |
| 12. Possíveis erros | Caminho de bloco inválido. |
| 13. Critérios de aceitação | CA1: Script constrói modelo de 3 blocos e simula<br>CA2: Callback InitFcn executa antes da compilação |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F219 — Funções e mensagens em modelos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F219 |
| 2. Nome conceitual | Funções e mensagens em modelos |
| 3. Descrição | Funções definidas em modelo chamáveis de outros pontos, portas de função, sinais de mensagem com filas, eventos discretos. |
| 4. Necessidade atendida | Arquiteturas orientadas a serviços e eventos. |
| 5. Entradas | Blocos. |
| 6. Saídas | Execução. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Filas com capacidade e política. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | Fila cheia. |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Chamada de função entre subsistemas<br>CA2: Fila de mensagens FIFO com capacidade |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F220 — Linearização e equilíbrio básicos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F220 |
| 2. Nome conceitual | Linearização e equilíbrio básicos |
| 3. Descrição | Extração de modelo linear em ponto de operação, busca de ponto de equilíbrio (trim). |
| 4. Necessidade atendida | Análise de controle. |
| 5. Entradas | Modelo, ponto. |
| 6. Saídas | Matrizes A,B,C,D; estados de equilíbrio. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Perturbação numérica. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | Modelos com descontinuidades. |
| 12. Possíveis erros | Não convergência. |
| 13. Critérios de aceitação | CA1: Linearização de modelo linear reproduz matrizes |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Hipótese. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

### F221 — Co-simulação e FMI

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F221 |
| 2. Nome conceitual | Co-simulação e FMI |
| 3. Descrição | Importar unidades de modelo funcional (FMI 1/2/3, troca de modelo e co-simulação) como blocos; exportar modelos como FMU; serviços de co-simulação. |
| 4. Necessidade atendida | Interoperabilidade com outras ferramentas de modelagem. |
| 5. Entradas | Arquivos FMU. |
| 6. Saídas | Blocos/FMUs. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Padrão FMI aberto (Modelica Association). |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | FMU binária de plataforma diferente. |
| 12. Possíveis erros | FMU inválida. |
| 13. Critérios de aceitação | CA1: Importar FMU de referência (FMI Cross-Check) e simular<br>CA2: Exportar FMU validada por ferramenta de terceiros |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Muito alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Sem restrição jurídica específica identificada além das gerais (marca e independência de implementação). |

*Status:* **PLANEJADA**

### F222 — Particionamento, tarefas e agendamento

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F222 |
| 2. Nome conceitual | Particionamento, tarefas e agendamento |
| 3. Descrição | Editor de agendamento de partições/tarefas, mapeamento de taxas, editor de interfaces. |
| 4. Necessidade atendida | Sistemas multitarefa. |
| 5. Entradas | Modelo. |
| 6. Saídas | Agendamento. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F202 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Ordem de partições editável afeta execução |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**

### F223 — Busca e navegação em modelos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F223 |
| 2. Nome conceitual | Busca e navegação em modelos |
| 3. Descrição | Busca de blocos/parâmetros/sinais com filtros, substituição, navegador de hierarquia, pré-visualização de arquivos de modelo. |
| 4. Necessidade atendida | Modelos grandes navegáveis. |
| 5. Entradas | Consultas. |
| 6. Saídas | Resultados. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F200 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Busca por valor de parâmetro em hierarquia |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F224 — Modelos de partida e página inicial

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F224 |
| 2. Nome conceitual | Modelos de partida e página inicial |
| 3. Descrição | Galeria de modelos-modelo, criação a partir de modelo, página inicial com recentes. |
| 4. Necessidade atendida | Início rápido. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | Conteúdo próprio. |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F200 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Criar modelo a partir de modelo-base próprio |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F225 — Documentação de sistemas em modelos

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F225 |
| 2. Nome conceitual | Documentação de sistemas em modelos |
| 3. Descrição | Notas e documentação anexadas a sistemas no modelo. |
| 4. Necessidade atendida | Rastreabilidade. |
| 5. Entradas | Texto. |
| 6. Saídas | Notas. |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F200 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Nota persiste com o modelo |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Baixa |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F226 — Integração com projetos e controle de versão

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F226 |
| 2. Nome conceitual | Integração com projetos e controle de versão |
| 3. Descrição | Modelos em projetos, análise de impacto de alterações, recarregamento após operações de VCS. |
| 4. Necessidade atendida | Trabalho em equipe. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F094, F216 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Recarregar modelo após checkout |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Média |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **PLANEJADA**

### F227 — Simulação em segundo plano e cache

| Campo | Conteúdo |
|---|---|
| 1. Identificador | F227 |
| 2. Nome conceitual | Simulação em segundo plano e cache |
| 3. Descrição | Simulações em segundo plano sem bloquear o editor; cache de artefatos de compilação. |
| 4. Necessidade atendida | Produtividade. |
| 5. Entradas | — |
| 6. Saídas | — |
| 7. Fluxo de utilização | Usuário constrói/edita modelo de blocos → configura parâmetros e solver → simula → inspeciona/registra sinais. |
| 8. Comportamento esperado | — |
| 9. Regras de negócio observáveis | — |
| 10. Dependências | F212 |
| 11. Casos especiais | — |
| 12. Possíveis erros | — |
| 13. Critérios de aceitação | CA1: Simulação longa não bloqueia edição |
| 14. Serviço externo | Não. |
| 15. Grau de confiança | Requisito independente; o comportamento é fixado pelos critérios de aceitação e pelos testes. |
| 16. Complexidade estimada | Alta |
| 17. Restrições técnicas | — |
| 18. Restrições jurídicas | Comportamento funcional e nomes de funções/sintaxe: compatibilidade de linguagem é questão PENDENTE de avaliação jurídica (ver LICENCAS_E_RESTRICOES.txt §15, Q2). Nenhum texto/código do original pode ser usado. |

*Status:* **INVESTIGADA PARCIALMENTE**


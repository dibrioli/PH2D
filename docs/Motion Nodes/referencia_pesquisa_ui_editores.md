# Mineração UI/UX — TouchDesigner · Nuke · Fusion · Substance Designer · Notch

Fontes primárias fetched: docs.derivative.ca (Network_Editor, OP_Create_Dialog, Parameter_Dialog, Flag), learn.foundry.com (working_nodes, properties_panels), manual Resolve/Fusion 18.6 (mirror steakunderwater), experienceleague.adobe.com (exposing-a-parameter), manual.notch.one (nodegraph 0.9.23 + 2026.1). Itens não confirmados na fonte estão marcados `(convenção conhecida — verificar)`.

---

## 1. TouchDesigner (docs.derivative.ca)

URLs: https://docs.derivative.ca/Network_Editor · https://docs.derivative.ca/OP_Create_Dialog · https://docs.derivative.ca/Parameter_Dialog · https://docs.derivative.ca/Flag

### A) Tabela

| Padrão | Como funciona (mecânico) | Por que é bom | Custo |
|---|---|---|---|
| **Cor por família de operador** | 6 famílias (COMP/TOP/CHOP/SOP/MAT/DAT), cada uma com cor própria; no OP Create, **geradores = tom mais escuro, filtros = tom mais claro** da cor da família | Você lê o TIPO de dado e se o nó cria ou transforma sem ler texto | Baixo |
| **Viewer AO VIVO dentro do nó** | Flag Viewer liga preview do dado dentro do corpo do nó; flag **Viewer Active** (canto inferior-direito) torna o viewer INTERATIVO (orbitar 3D, scrollar tabela DAT, tocar num slider CHOP) dentro do próprio nó; nó redimensionável arrastando a borda com LMB | O nó é o monitor; zero janelas extras para inspecionar | Alto (interativo); Médio (só live) |
| **Flags como pips clicáveis nas bordas** | Flags são "binary states, not parameters" (não disparam cook), desenhadas na **borda esquerda + inferior** do nó: Viewer, Viewer Active, Lock, Bypass, Cooking, Immune (clone), Current, Expose; 3D COMPs: Render/Display/Pickable; CHOPs: Export; SOPs: Compare/Template. Todas scriptáveis (`op('x').lock = True`) | Estado por-nó manipulável com 1 clique no lugar onde se lê; sem menu | Médio |
| **4 modos por parâmetro, cor de fundo por modo** | Clicar no label expande o param → 4 botões quadrados: **cinza=constant, azul=expression, verde=export (CHOP/DAT dirigindo), roxo=bind**. **Os valores dos 4 modos ficam salvos simultaneamente** (troca sem perda); botão de modo não-ativo com dado mostra quadradinho no canto | O artista vê DE ONDE vem cada valor pela cor, e alterna sem destruir nada | Médio/Alto (o storage 4-em-1 é a parte cara e a mais valiosa) |
| **Value ladder no MMB** | MMB num campo numérico abre ladder de scrub; ladder **no label** de param composto (Translate xyz) ajusta todos os componentes juntos; MMB no campo individual ajusta só ele | Scrub grosso/fino sem digitar; edição vetorial coerente | Baixo |
| **Param dialog: 3 encarnações** | `p` = docked no pane; RMB no nó = flutuante; tipo de Pane dedicado; botão "sticky" permite N dialogs abertos; header com **cor da família**; PAGES (tabs) por operador; Alt+hover no label = help popup; multi-seleção de nós idênticos = edição em lote | Params onde você precisa deles; batch-edit de graça | Médio |
| **OP Create dialog** | Abre por duplo-clique no fundo, Tab, ícone +, **MMB/RMB no input/output de um nó** (já conecta), RMB num FIO ("Add Operator" insere no fio); tabs por família; digitar filtra — matches ficam **brancos**; Ctrl = colocar vários; Shift = colocar vários JÁ ENCADEADOS | Criar-conectado e inserir-no-fio são o caminho default, não o expert | Baixo (PH2D já tem busca; faltam os spawn-points contextuais) |
| **Wires animadas = cooking** | "Animated wires indicate data flow" — a animação mostra o que está cozinhando agora | Debug de fluxo passivo, sem abrir profiler | Baixo |
| **MMB info popup** | MMB no nó abre popup de info do operador (tipo, canais/resolução, **cook time**, memória) `(cook time/mem: convenção conhecida — verificar)` | Perf e schema a 1 gesto do nó | Baixo |
| **Navegação hierárquica** | Entrar no COMP: duplo-clique, Enter, `i`, ou zoom-scroll ATRAVÉS do nó; sair: `u` ou zoom-out; **breadcrumb clicável** no topo do pane + botões back/forward; `o` = overview map; Ctrl+MMB = box-zoom; `f` = frame all; Shift+T = modo lista (tabela de nós) | Zoom semântico: profundidade sem modal | Médio |
| **Parameter COMP** | Dialog de params pode ser embutido em painel custom escolhendo páginas/params visíveis | Expor params vira UI de usuário final | Alto |

---

## 2. Nuke (learn.foundry.com)

URLs: https://learn.foundry.com/nuke/content/getting_started/using_interface/working_nodes.html · https://learn.foundry.com/nuke/content/getting_started/using_interface/properties_panels.html · https://learn.foundry.com/nuke/content/reference_guide/other_nodes/postagestamp.html · https://support.foundry.com/hc/en-us/articles/207682435 · https://github.com/artandmath/NukeProfilers

### A) Tabela

| Padrão | Como funciona (mecânico) | Por que é bom | Custo |
|---|---|---|---|
| **Tab-search com repeat** | Tab + digitar = typeahead; setas + Return escolhem; **Tab, Return = repete o último nó criado**; MMB num ícone da Toolbar repete o último item DAQUELE menu | O 2º uso do mesmo nó custa 2 teclas | Baixo |
| **Modificadores na criação** | Nó novo auto-conecta ao selecionado; **Shift na criação = branch** (ramo novo em vez de inline); **Ctrl na criação = substitui** o nó selecionado; colar conecta ao selecionado (deselecionar antes p/ colar solto) | Conectar/ramificar/substituir são o MESMO gesto + 1 tecla | Baixo |
| **Postage stamps com toggle** | Checkbox por nó + **Alt+P**; preferência global (Shift+S) com modo **Static frame** (thumbnail de UM frame fixo, não do frame atual) — mitigação oficial de perf | Preview onde importa, custo controlado por nó | Baixo (PH2D já tem stamps; o delta é o static-frame + toggle global) |
| **Vocabulário de badges no nó** | Retângulo **largo vs fino** = processa todos os canais vs subset; indicador de mask; **disable = hachurado/striped**; **clone = linha laranja** até o pai; badge de animação; badge de expressão; **dots coloridos** = split por view (stereo); indicador de mix < 1; **quadrado amarelo no knob** = override de Link node | Estado denso e glanceable; o nó confessa tudo que o torna "não-puro" | Médio |
| **Inserir arrastando no fio** | Arrastar nó sobre o vão entre dois conectados → **o fio "acende"** → soltar insere; **shake do ponteiro desconecta** o nó (heal automático do fio); Ctrl+D desconecta | Insert/extract sem alvo pequeno; o highlight elimina o erro | Baixo/Médio |
| **Dots (reroute)** | Ctrl+clique no **ponto amarelo** que aparece sobre o fio cria um Dot; Dot também via menu Other | Cotovelos deliberados, organização de B-pipe | Baixo (PH2D já tem) |
| **Properties Bin multi-painel** | Duplo-clique abre painel do nó NO BIN (vários abertos empilhados); **campo numérico limita o nº máximo de painéis abertos**; botão lock força novos a abrirem flutuantes; botão remove-all; Ctrl+duplo-clique = flutuante direto; Alt+clique no X fecha todos; Ctrl+clique no X fecha todos menos este | Comparar/editar N nós ao mesmo tempo — o oposto do inspector modal | Médio/Alto |
| **Knobs com expressão inline** | Digitar `=` no campo inicia expressão; aceita fórmulas (`378/2`); **a posição do cursor no número define a magnitude do incremento** das setas (centenas/dezenas/décimos); MMB = slider virtual; Shift aumenta / Alt diminui sensibilidade | Matemática e scrub sem sair do campo | Baixo/Médio |
| **Seleção estrutural** | Ctrl+drag num nó = seleciona todo o UPSTREAM; Ctrl+Alt+A = árvore conectada; `/` = busca por nome com wildcards | Operar em sub-árvores como unidade | Baixo |
| **Backdrop** | Nó Backdrop cria caixa ATRÁS; mover a caixa move os nós que a sobrepõem; título + cor; `(z-order: backdrops menores desenham por cima — convenção conhecida)` | Agrupamento espacial sem hierarquia | Baixo (PH2D já tem) |
| **Node tab padrão em TODO nó** | Cada nó tem aba Node: label livre (aceita HTML), cor do nó, cor dos controles no Viewer, **cache (sublinhado amarelo)**, disable (D), bookmark, lifetime (range de frames em que o nó existe) | Metadados de organização uniformes, sem nó especial | Médio |
| **Profile node** | Desde 11.1: nó Profile mede métricas da árvore para achar gargalo (cf. NukeProfilers) | Profiling é parte do grafo, não ferramenta externa | Médio |

---

## 3. DaVinci Fusion (Blackmagic)

URLs: https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part1556.htm (cap. "Working in the Node Editor") · part2321.htm (Sticky Note/Underlay) · https://brontosaurusrex.github.io/2020/07/17/Fusion-Notes/ · https://forum.blackmagicdesign.com/viewtopic.php?f=22&t=103746

### A) Tabela

| Padrão | Como funciona (mecânico) | Por que é bom | Custo |
|---|---|---|---|
| **Select Tool dialog** | **Shift+Space** abre busca de nós no cursor; cada ferramenta tem **abreviação oficial de 3 letras** ("Nte"=Sticky Note, "Und"=Underlay) usável na busca; Effects Library e toolbar como rotas paralelas | Abreviações memorizáveis = autoria por muscle memory | Baixo |
| **Inputs coloridos por papel** | Inputs = **triângulos** coloridos por SEMÂNTICA: background laranja/amarelo, foreground verde, **mask azul**; output = quadrado `(cores exatas: convenção conhecida — verificar)`; **Ctrl+T troca BG↔FG** do nó selecionado | O PAPEL do fio (não só o tipo) é legível; swap de inputs vira reflexo | Baixo/Médio |
| **Auto-routing + routers** | Fios roteiam automaticamente contornando nós; opção de fio **direto vs ortogonal**; **Alt+clique em qualquer ponto do fio = cria router (dot)** arrastável | Grafo limpo por default; reroute custa 1 clique | Médio |
| **Extract / re-insert com Shift** | **Shift+drag num nó = EXTRAI do fluxo** (fio se cura sozinho); manter Shift e soltar sobre outro fio = re-insere ali; drop de nó novo sobre fio insere | Refactor de pipeline por drag, sem religar 4 pontas | Médio |
| **Instances com linha verde** | Ctrl+C → RMB no fundo → **Paste Instance**: cópia LINKADA, **linha verde** desenhada entre original e instância; **RMB no label de UM param → Deinstance** solta SÓ aquele param (override local); RMB de novo → Reinstance | Clone com override granular por-param — o meio-termo que clone puro não dá | Alto (o de-instance por param é a joia) |
| **Underlay** | Caixa colorida com título ATRÁS dos nós (categoria Flow); mover move os nós dentro; "highlight rather than hide", não restringe conexões externas | Organização visual sem colapsar (complementar a Group) | Baixo (PH2D já tem backdrop) |
| **Sticky Note** | "Não é um nó" — anotação colapsável/expansível ancorada numa área; SEM opções de cor (reclamação de fórum citada) | Comentário rico sem poluir o grafo | Baixo |
| **Groups** | **Ctrl+G agrupa**, **Ctrl+E expande/colapsa in-place**; RMB → Settings → Save As salva o grupo em disco p/ reuso por drag-drop | Subgrafo colapsável + biblioteca de macros grátis | Médio (PH2D já tem subgrafos; o delta é expand-in-place e save-as-asset) |
| **Node Navigator (minimap)** | Tecla **V** alterna minimapa no canto; `(auto-aparece quando o grafo excede a vista — manual §"Using the Node Navigator")`; **Node View Bookmarks** nomeiam vistas do grafo p/ pular | Navegação espacial em comps de centenas de nós | Médio |
| **Thumbnails opcionais por nó** | Toggle entre tile-picture (render) e ícone, global ou por nó; manual §"Node Thumbnails" | Preview onde ajuda, ícone onde o render não diz nada | Baixo |
| **Modos do nó** | **Ctrl+P = pass-through** (disable); Lock; indicação visual de cache; manual §"Node Modes Including Disable and Lock" | Estado operacional visível e atalhável | Baixo |
| **Versions** | Toolbar do Inspector com slots de versão 1–6 por nó `(color page/Fusion — convenção conhecida)` | A/B de settings dentro do nó | Alto |

---

## 4. Substance 3D Designer (Adobe)

URLs: https://experienceleague.adobe.com/en/docs/substance-3d-designer/using/substance-graphs/manage-parameters/exposing-a-parameter · https://experienceleague.adobe.com/en/docs/substance-3d-designer/using/workspace/properties · https://helpx.adobe.com/substance-3d-designer/substance-compositing-graphs/graph-instances-sub-graphs.html

### A) Tabela

| Padrão | Como funciona (mecânico) | Por que é bom | Custo |
|---|---|---|---|
| **O nó É o thumbnail** | Nó = quadrado grande cujo conteúdo é o **resultado computado** ("the image shown in the instance node is its result"); atualiza ao cozinhar | Legibilidade máxima num domínio visual: o grafo é um contact sheet | Já têm stamps; o delta é stamp-como-corpo (Médio) |
| **Expose por-param com dialog completo** | Dropdown ao lado do param → **"Expose as new graph input"** → dialog: Identifier (sem espaço) vs Label, **Type/Editor define widget** (Float: slider/dial/type-in; Int: +dropdown; Bool: checkbox; Color: picker; Gradiente/Curve/Levels NÃO exponíveis), Default, **Min/Max + Clamp (hard vs soft limit)**, Step, **Group com nesting por `/`**, Description (tooltip), **Visible If (expressão condicionada a outro param)**, User Data | O contrato do subgrafo é autorado param a param, com widget e condição — é o que faz uma biblioteca de assets escalar | Alto (mas parcelável: widget+range primeiro) |
| **Batch expose** | Botão Multi-expose → checklist de params, identifier/grupo por linha, **prefixo/sufixo global** | Exposição em massa sem ceremony por-param | Médio |
| **Função por parâmetro** | Botão **"Edit parameter function"** ao lado do param abre um GRAFO de função (nós matemáticos) dirigindo aquele param | Expressões visuais sem linguagem de texto; consistente com o resto do app | Alto |
| **Params de instância viram portas** | Params expostos aparecem como **portas conectáveis no nó de instância** no grafo pai | Dados e parâmetros unificados no mesmo sistema de fios | Médio |
| **Reordenar por drag** | Alças listradas à esquerda de cada param na lista de Input Parameters; grupos afetam a ordem | O autor controla a apresentação do asset | Baixo |
| **Preview mode (SBSAR)** | Tab Preview simula o asset PUBLICADO (limitações de param estático incluídas); mudanças descartadas a menos que Apply | Testar o contrato como o consumidor o verá | Médio |
| **Node Finder + badges de referência quebrada** | Ferramenta na barra localiza quais nós usam um param; **warning badges** indicam referências quebradas | Higiene de grafo auditável | Médio |
| **Clean Inputs** | Ferramenta remove params expostos órfãos | Anti-apodrecimento do contrato | Baixo |
| **Dot/reroute por duplo-clique no fio** | `(convenção conhecida — verificar: duplo-clique num link cria dot node)`; Frames com título/cor/resize + Comments ancorados a nós | Reroute sem menu | Baixo (já têm) |
| **Busca com Espaço no cursor** | `(convenção conhecida)` Espaço abre o menu de criação com busca na posição do mouse; fileira de atômicos na toolbar | Criação sem viagem de mouse | Baixo (já têm) |

---

## 5. Notch (manual.notch.one)

URLs: https://manual.notch.one/2026.1/en/docs/reference/user-interface/nodegraph/ · https://manual.notch.one/0.9.23/en/docs/user-interface/nodegraph/ · https://manual.notch.one/2026.1/en/docs/learning/using-the-node-system/keeping-projects-tidy/ · https://manual.notch.one/2026.1/en/docs/whats-new/everything/

### A) Tabela

| Padrão | Como funciona (mecânico) | Por que é bom | Custo |
|---|---|---|---|
| **Perf inline SOB o nome do nó** | Com profiling ligado, **métricas de tempo aparecem embaixo do nome de cada nó** no grafo; Profiler window com tempo de cálculo por nó | O custo mora ONDE se edita — otimizar vira parte da autoria | Baixo/Médio (PH2D já cozinha; é render de um número já existente) |
| **Broadcast icon = ativo do grupo** | Nós de câmera/renderer ganham ícone de broadcast indicando qual é O ATIVO quando existem vários candidatos | Resolve o "qual dos 3 está valendo?" sem inspecionar | Baixo |
| **Dois tipos de fio com desenho distinto** | **Hierarquia de transform (pai→filho)** vs **property links** (dado→param) são vínculos distintos; modos de render **Straight vs Bezier** configuráveis | Duas semânticas, duas aparências — nunca se confundem | Médio |
| **Shift = multi-conexão** | Segurar Shift arrastando do output permite fazer **várias conexões em sequência** sem re-pegar a porta | Fan-out rápido de um output | Baixo |
| **Floating Search + Floating Find** | **Ctrl+Space / duplo-clique** = busca de criação NO CURSOR; **Ctrl+F = Find**: busca qualquer nó EXISTENTE e navega até ele | Criação e navegação são a MESMA ergonomia de busca | Baixo (criação já têm; o Find é o roubo) |
| **Insert-no-fio com Ctrl** | Arrastar um nó existente sobre um fio **segurando Ctrl** insere entre os dois | Modificador evita inserções acidentais | Baixo |
| **Minimap com presets** | Minimapa simplificado (nós sem fios/labels); **clique = pan, scroll = zoom, RMB = presets de tamanho/posição ou esconder** | Overview persistente, mas domável | Médio |
| **Node pinning** | Pinar nós filtra/persiste através de vistas (2026.1) | Working set explícito em grafo gigante | Médio |
| **Filtros de vista** | Toggles **Hide Disabled**, **Hide Unconnected**, Hide Thumbnails, Hide Input/Output Connections; seleção retângulo E laço; Center View On Selected | Declutter sob demanda sem tocar no documento | Baixo |
| **Erro no nó** | Display de erro integrado à representação do nó | Falha é local, não um log distante | Já têm (validar paridade) |

---

## B) Os 15 padrões que eu ROUBARIA primeiro (ranqueados por alavancagem ÷ custo, dado o que o PH2D já tem)

1. **4 modos por param com cor de fundo + storage 4-em-1 (TD)** — a melhor ideia de UI de params já shipada: a COR diz a proveniência do valor (constante/expressão/export/bind) e trocar de modo **não destrói** o valor dos outros. O PH2D já tem `motion.expression` e text params; falta o seletor visual por-param. https://docs.derivative.ca/Parameter_Dialog
2. **Flags como pips clicáveis na borda do nó (TD)** — bypass/lock/viewer/render como estado binário desenhado E clicável no nó, explicitamente "não-parâmetro" (não cooka). Unifica os badges que o PH2D já tem (portal ⊙, live/cooked) num sistema de bordas com lei própria. https://docs.derivative.ca/Flag
3. **Perf inline sob o nome do nó (Notch) + MMB info popup (TD)** — o cook time que o `Cook` já mede, pintado no grafo (toggle de profiling). O probe/sparkline do PH2D mostra DADO; isto mostra CUSTO. Barato e diferenciador. https://manual.notch.one/0.9.23/en/docs/user-interface/nodegraph/
4. **Drop-no-fio com highlight + shake-to-disconnect (Nuke)** — inserir arrastando o nó sobre o fio (que acende ao aceitar) e extrair sacudindo (fio se cura). Par perfeito com o knife existente. https://learn.foundry.com/nuke/content/getting_started/using_interface/working_nodes.html
5. **Modificadores na criação: Shift=branch, Ctrl=replace, Tab+Return=repetir último (Nuke)** — três decisões estruturais viram teclas no momento da busca que o PH2D já tem. Custo quase zero, ganho diário.
6. **Deinstance por-param com linha verde (Fusion)** — clone linkado onde CADA param pode ser solto individualmente (override) e re-linkado. Supera clone binário; casa com o modelo de params-no-Graph do PH2D. https://brontosaurusrex.github.io/2020/07/17/Fusion-Notes/
7. **Expose dialog completo por-param (Substance)** — widget escolhível, min/max com clamp hard/soft, step, grupos com `/`, **Visible If**, batch expose com prefixo. É o contrato de subgrafo que faz biblioteca escalar. https://experienceleague.adobe.com/en/docs/substance-3d-designer/using/substance-graphs/manage-parameters/exposing-a-parameter
8. **Value ladder no MMB + incremento pela posição do cursor (TD+Nuke)** — scrub grosso/fino sem digitar; ladder no label edita o vetor inteiro; setas incrementam o dígito onde o cursor está; `=` inicia expressão inline.
9. **Ctrl+F "Find" que navega (Notch)** — a MESMA busca da criação, apontada para nós existentes, com pan/zoom até o hit. Grafos de 67+ nós do PH2D já precisam.
10. **Cores semânticas de INPUT por papel + Ctrl+T swap (Fusion/Nuke A-B)** — o input diz seu PAPEL (fonte vs template vs máscara), não só o tipo; swap de inputs é 1 tecla. O PH2D tem porta-template (`SourceRows`) — está pedindo isso.
11. **Static-frame stamps + toggle global (Nuke)** — a lição de perf da Foundry: thumbnail de frame FIXO por nó + preferência global, porque stamps vivos re-cozinham o script inteiro. Aplicar aos stamps existentes ANTES que virem o gargalo. https://support.foundry.com/hc/en-us/articles/207682435
12. **Properties bin multi-painel com teto (Nuke)** — N painéis de params abertos, campo com o máximo, pin/float. Comparar 2 nós lado a lado é o caso diário que inspector modal não serve.
13. **Viewer ATIVO dentro do nó (TD)** — interagir com o dado (scrubbar tabela, orbitar, tocar curva) dentro do corpo do nó redimensionável. Evolução natural do postage stamp existente; caro, mas é o teto da categoria.
14. **Vocabulário denso de badges (Nuke)** — hachurado=bypass, largura do retângulo=quantos canais processa, linha laranja=clone, dot=split por view, quadrado amarelo=override. Sistematizar os badges do PH2D numa gramática documentada.
15. **Grupo com expand-in-place + Save As asset (Fusion Ctrl+G/Ctrl+E)** — colapsar/expandir subgrafo SEM navegar para dentro, e salvar o grupo como asset re-instanciável. Complementa o subgrafo-navegável existente.

Menções honrosas: geradores-escuros/filtros-claros no menu de criação (TD, custo ~zero); Alt+hover = help do param (TD); minimap com RMB-presets (Notch); Node View Bookmarks (Fusion); wires animadas durante cook (TD); grafo de função por-param (Substance).

## C) Anti-padrões conhecidos (reclamações de artistas)

- **Nuke — postage stamps vivos matam o script**: stamps no frame corrente forçam cook de árvores inteiras que nem estão sendo vistas ("was always calculating this 3D script... just because it had to show the postage stamp"); a mitigação oficial é static frame ou desligar. Lição: preview no nó SEMPRE precisa de política de custo. https://support.foundry.com/hc/en-us/articles/207682435 · https://community.foundry.com/discuss/topic/99337 · https://github.com/artandmath/NukeProfilers
- **Nuke — organização é disciplina manual**: B-pipe vertical, dots, backdrops — tudo convenção que o app não impõe; scripts alheios viram arqueologia (cf. benmcewan.com "compositing efficiently").
- **TouchDesigner — undo e refresh de params**: mudanças (incl. undo) não re-renderizam o parameter dialog até forçar refresh (bug report 2021 ainda citado); undo historicamente não cobre tudo em perform mode. Lição: params são uma VIEW do documento — nunca um cache com vida própria. https://forum.derivative.ca/t/213011 · https://derivative.ca/UserGuide/Undo
- **TouchDesigner — flags pequenas demais**: pips de borda são alvos minúsculos em zoom out `(reclamação recorrente de fórum)`; se roubar o padrão, escale os alvos com o zoom.
- **Fusion — inspector modal**: só o(s) nó(s) selecionado(s) têm params visíveis; comparar dois nós exige pin manual — exatamente o que o Properties Bin do Nuke resolve.
- **Fusion — anotações capadas**: Sticky Note sem opção de cor (thread oficial); Underlay com customização mínima. Anotação de segunda classe envelhece mal. https://forum.blackmagicdesign.com/viewtopic.php?f=22&t=103746
- **Substance — thumbnails sempre-vivos em grafo pesado**: o nó-é-o-resultado obriga recompute visual constante; em grafos grandes o engine pausa/atrasa e o artista trabalha "às cegas" `(reclamação recorrente)`. Mesmo remédio do Nuke: política de custo por nó.
- **Substance — expose é cerimônia**: expor 20 params um a um é lento (por isso o Batch Expose existe desde depois); planeje o batch desde o dia 1.
- **Notch — duas semânticas num grafo só**: hierarquia de transform COMO fios confunde quem vem de compositing (o fio "pai" não carrega dado); o manual precisa de página inteira de "node hierarchies" para explicar. Se o PH2D um dia misturar cena e dado, desenhe-os inconfundíveis. https://manual.notch.one/0.9.23/en/docs/techniques/node-hierarchies/
- **Notch — minimap intrusivo**: default visível em cima do grafo; a própria Notch adicionou RMB→hide/presets. Minimap nasce opt-in ou domável.
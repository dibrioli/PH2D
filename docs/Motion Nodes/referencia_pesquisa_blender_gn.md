Tenho tudo — docs oficiais 4.5 (41 páginas), release notes 4.1–4.5, workshops de dev 2025 e threads de comunidade. Segue o markdown denso.

---

# Blender Geometry Nodes 4.x — pesquisa de referência para PH2D Motion Nodes

Fontes primárias: manual 4.5 LTS (`docs.blender.org/manual/en/4.5/`), release notes 4.1/4.3/4.4/4.5 (`developer.blender.org/docs/release_notes/`), Geometry Nodes Workshops jul+set/2025 (`code.blender.org`), devtalk. URLs por seção abaixo.

---

## A) CATÁLOGO — `nó · categoria · função · status vs PH2D`

Categorias do add menu 4.5: Attribute · Input (Constant/Gizmo/Group/Scene) · Output · Geometry (Read/Sample/Write/Operations) · Curve (Read/Sample/Write/Operations/Primitives/Topology) · Grease Pencil · Instances · Mesh · Point · Volume · Simulation · Texture · Utilities (Color/Field/Math/Matrix/Rotation/Text/Vector) · Zones.

| Nó | Categoria | Função | Status vs PH2D (87 nós) |
|---|---|---|---|
| **Instance on Points** | Instances | Instancia geometria em cada ponto; `Pick Instances` escolhe da lista por índice (wrap negativo/overflow); nested instancing; copia `id` | **TEMOS** (clone/instance_field/scatter) — falta o `Pick Instance` por índice de uma lista de variantes |
| **Distribute Points on Faces** | Point | Scatter em superfície, Random ou **Poisson Disk** (Distance Min + Density Max × Density field); gera **ID estável** (sobrevive a deform/mudança de densidade); saídas Normal + Rotation | **PARCIAL** — temos distribute_*/scatter e id estável por hash; Poisson disk (min-distance) e density-como-field a conferir |
| **Random Value** | Utilities | White noise por elemento: Float/Int/Vector/**Boolean(probability)**; entradas ID + Seed separadas | **TEMOS** (hash(seed,id,lane)) — variante Boolean-com-probabilidade e Min/Max por-tipo são o refinamento |
| **Noise/Voronoi/White Noise Texture** | Texture | Campos procedurais N-D (4.1 fundiu Musgrave no Noise) | **TEMOS** (noise, curl divergence-free, voronoi JFA na GPU) |
| **Map Range** | Utilities▸Math | Remap com 4 interpolações + clamp + Float/Vector | **TEMOS** (map_range) — conferir modos B) |
| **Mix** | Utilities▸Math | Lerp Float/Vector/Color/Rotation; fator uniforme ou não-uniforme (por eixo) | **PARCIAL** (math/expression cobrem; nó dedicado com fator por-eixo falta) |
| **Float Curve** | Utilities▸Math | **Curva como widget dentro do nó** — mapeia float por curva editável, com Factor de influência | **FALTA** (nosso param não-f32 canônico é o text-param; curve widget seria o 2º tipo) |
| **Color Ramp / RGB Curves** | Utilities▸Color | Gradiente/curvas como widget no nó | **FALTA** como widget de nó |
| **Sample Curve** | Curve▸Sample | Amostra posição/tangente/normal + **qualquer field (Value)** a um comprimento/fator do caminho; Curve Index ou All Curves | **FALTA** (é o "path sampler" que anima coisas ao longo de trilha) |
| **Resample Curve** | Curve▸Operations | Re-amostra para espaçamento uniforme: Count/Length/Evaluated; count pode ser **field por spline** | **FALTA** no grafo motion (vector módulo tem análogo de editor) |
| **Trim Curve** | Curve▸Operations | Corta início/fim por Factor/Length | **FALTA** no grafo |
| **Fillet Curve** | Curve▸Operations | Arredonda quinas: Bézier/Poly(Count), Limit Radius anti-overlap | **FALTA** no grafo (vector tem como tool) |
| **Points to Curves** | Point | Agrupa pontos em curvas por **Group ID** + ordena por **Weight** | **FALTA** — é o "trails a partir de partículas" deles |
| **Index / ID** | Input | Índice do elemento; `id` se existir senão índice | **TEMOS** (colunas/id) |
| **Position / Set Position** | Geometry Read/Write | Ler/escrever posição; Set tem **Position + Offset** (avaliados juntos, sobre o estado antigo) e Selection | **TEMOS** (attribute/integrate) |
| **Math / Vector Math / Boolean Math** | Utilities▸Math | Aritmética escalar/vetorial/booleana (4.5: Vector Power/Sign) | **PARCIAL** (math sim; boolean math dedicado + vector math completo a conferir) |
| **Integer Math / Bit Math / Hash Value** | Utilities▸Math | Int dedicado (4.3), bits (4.5), hash de qualquer tipo → int (4.3) | **PARCIAL** (hash temos; int/bit não) |
| **Compare** | Utilities▸Math | Comparações com epsilon, saída bool | **PARCIAL** (expression cobre) |
| **Clamp / Float to Integer** | Utilities▸Math | Clamp; round/floor/ceil/trunc | **PARCIAL** |
| **Switch** | Utilities | 2 entradas, condição bool; **só o ramo usado é computado** | **TEMOS** (switch) — lazy evaluation é o detalhe a copiar |
| **Index Switch** (4.1) | Utilities | N entradas escolhidas por inteiro | **FALTA** |
| **Menu Switch** (4.1) | Utilities | **Enum custom criado pelo usuário** — itens nomeados/reordenáveis viram dropdown, exposto como socket de menu até o group input/modifier | **FALTA** — é o "param enum autorável" |
| **Attribute Statistic** | Attribute | Reduce com Selection: **Mean/Median/Sum/Min/Max/Range/StdDev/Variance**, Float/Vector, por domínio | **PARCIAL** — nosso REDUCE GPU tem Max/Min/Sum; faltam mean/median/stddev/variance/range e Selection |
| **Accumulate Field** | Utilities▸Field | **Scan (prefix sum)** com 3 saídas: Leading (inclusivo), Trailing (exclusivo), Total; **Group Index = bins independentes**; Float/Int/Vector/Transform | **TEMOS** (scan GPU) — Group Index (scan segmentado) e Trailing/Leading como saídas gêmeas são o refinamento |
| **Field Average / Field Min & Max / Field Variance** (4.5) | Utilities▸Field | As estatísticas **sem input de geometria** (usam o contexto), com Group ID | **PARCIAL** (mesma família do reduce) |
| **Evaluate at Index** | Utilities▸Field | Lê o valor de um field **em outro elemento** (gather por índice) sem precisar de geometry socket | **FALTA** — chave para vizinhança/lookup entre instâncias |
| **Evaluate on Domain** | Utilities▸Field | Fixa o domínio de avaliação de um field (controla QUANDO interpola) | **N/A parcial** (temos 1 domínio dominante por stream) |
| **Sample Index / Sample Nearest** | Geometry▸Sample | Gather de outra geometria por índice/proximidade | **FALTA** (inter-stream lookup) |
| **Capture Attribute** | Attribute | **Congela** o resultado de um field num ponto do fluxo como atributo anônimo; multi-item (arrasta sockets); auto-GC quando não usado | **PARCIAL** — nossas colunas transientes são próximas; o gesto explícito "capture aqui, use depois" + GC automático é a lição |
| **Store Named Attribute** | Attribute | Grava field como atributo NOMEADO (com Selection; não-selecionado vira zero se atributo novo) | **TEMOS** (attribute) |
| **Named Attribute (read) / Remove Named Attribute** | Attribute | Ler/remover por nome; overlay de uso de nomes | **PARCIAL** |
| **Sort Elements** (4.1) | Geometry▸Operations | Reordena índices por **Group ID + Sort Weight** (por domínio, com Selection) | **FALTA** |
| **Split to Instances** (4.1) | Instances | Separa geometria em N instâncias por Group ID | **FALTA** |
| **Simulation Zone** | Zones | Par Input/Output com estado entre frames; Delta Time; Skip; cache/bake | **TEMOS** (sim.zone passthrough-condicional; GGPO checkpoints) — diferenças em B) |
| **Repeat Zone** (4.0) | Zones | Par Input/Output; roda o miolo **Iterations** vezes, estado passa entre iterações; Inspection Index | **FALTA** (sim.zone não é loop de N passos dentro de um cook) |
| **For Each Geometry Element Zone** (4.3) | Zones | Executa o miolo **por elemento** (por domínio, com Selection); saídas "Main Geometry" (viram atributos) e "Generated" (geometrias juntadas) | **FALTA** como zona — mas nosso modelo por-coluna JÁ é per-element; a zona deles existe como escape hatch e a própria doc avisa que fields são muito mais rápidos |
| **Bake Node** (4.1) | Geometry▸Operations | Congela sub-árvore em disco/packed; Still ou Animation (range custom); multi-item com "Is Attribute"+domínio | **PARCIAL** (checkpoint ring em memória; bake persistente por-nó falta) |
| **Viewer Node** | Output | Espia geometria+1 field no viewport (overlay, inclusive **como texto**) e no Spreadsheet; Ctrl+Shift+LMB; atalhos 1–9 (4.5); domínio auto | **PARCIAL** (probe/sparkline/postage stamps; falta spreadsheet + overlay por-elemento no viewport) |
| **Warning Node** (4.3) | Utilities | Autor de node group emite Error/Warning/Info custom; setting de **propagação** para cima | **FALTA** |
| **Gizmo: Linear / Dial / Transform** (4.3) | Input▸Gizmo | **Gizmo no viewport dirigido pelo grafo**, bidirecional; propaga por links (double link) até group input/modifier | **FALTA** — detalhe em B)/C); casa com nosso norte de autoria no canvas |
| **Set Geometry Name** (4.3) | Geometry | Nomeia geometrias para debug de hierarquias de instâncias (spreadsheet mostra) | **FALTA** (naming de streams p/ debug) |
| **Import CSV/OBJ/PLY/STL/Text/VDB** (4.5) | Input▸Import | Dados externos como nó; **drop de arquivo no editor cria o import node** | **FALTA** (CSV → tabela de dados dirigindo motion é ideia forte) |
| **Format String / Find in String / Match String** (4.4/4.5) | Utilities▸Text | Strings com sintaxe estilo Python | **FALTA** (baixa prioridade p/ motion) |
| **Active Camera / Camera Info / Scene Time / Self Object** | Input▸Scene | Contexto de cena como nós | **PARCIAL** (playhead existe; camera culling não) |
| **Rotation sockets + Utilities▸Rotation** (4.1+) | Utilities | Tipo socket dedicado a rotação com 11 nós de conversão | **N/A** (2D: ângulo é escalar) |
| **Matrix sockets + Utilities▸Matrix** (4.2+) | Utilities | Tipo matriz 4×4 com combine/separate/multiply/invert/determinant | **PARCIAL** (2D affine implícito) |
| **Accumulate/Blur Attribute, Geometry Proximity, Raycast** | Attribute/Sample | Vizinhança espacial | **PARCIAL** (grade espacial GPU existe p/ boids/collide; como NÓS genéricos de query, falta) |

---

## B) PARÂMETROS dos nós mais relevantes (dados crus dos docs 4.5)

### Fields / sockets — a semântica central
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/fields.html
- **Um field é uma função** ("set of instructions that can transform an arbitrary number of inputs into a single output"), reavaliada por elemento e **por contexto**: o mesmo sub-grafo de field ligado a dois data-flow nodes é avaliado 2× e pode dar resultados diferentes (o 2º vê a geometria já alterada pelo 1º).
- **3 formas de socket**: **Círculo** = valor único obrigatório (nunca aceita field). **Diamante** = é/aceita field (varia por elemento). **Diamante com ponto** = *poderia* ser field mas atualmente é valor único — serve para rastrear onde há valor único E faz o socket-inspection mostrar o valor em vez do nome dos inputs do field.
- **Links de field são tracejados**; conexão ilegal (single→field obrigatório) desenha **linha vermelha sólida**.
- Tipos de nó: *data-flow* (geometry in/out), *function* (diamantes), *input* (Position/ID/Index — só significam algo no contexto de um data-flow node).
- Extrair valor único de field = Sample Index ou Attribute Statistic (não existe cast implícito).
- **Capture Attribute** congela o field num ponto do fluxo (atributo anônimo, GC automático); doc de fields mostra exatamente o caso "salvar posição original antes de mover".
- 5.0 (workshop set/2025): reforma de socket shapes saiu do experimental — a direção é distinguir ainda mais estruturas (single/field/list/grid).

### Simulation Zone
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/simulation/simulation_zone.html
- Par **Simulation Input / Simulation Output**; o que entra no Input é avaliado **uma vez** no início da sim; links externos para dentro da zona são reavaliados por passo; **nenhum link pode sair da zona** — resultado só via Output (isso viabiliza cache e interpolação sub-frame).
- **Delta Time** (input): segundos entre frames (inverso do frame rate, com sub-steps) — "keep the simulation playback consistent when the frame rate changes".
- **Skip** (bool input): repassa o estado de entrada direto ao output ignorando o miolo (pausa/hold da sim). 4.3 **escondeu o checkbox** do Skip no corpo do nó porque usuários clicavam sem querer (fica como socket).
- Itens de estado: lista dinâmica (arrastar link no socket vazio cria item; renomear com Ctrl+LMB no nome; painel "Simulation State" no sidebar). **Atributos anônimos NÃO atravessam a sim a menos que armazenados explicitamente como item de estado** (impossível prever o futuro do grafo).
- Clock: amarrado à animação; só avalia quando o frame muda; **cache automático** (linha amarela na timeline); opt-out de cache p/ economizar RAM; **bake em disco** para render não-sequencial; bake atinge todas as sims dos objetos selecionados. Não funciona no contexto Tool.

### Repeat Zone
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/utilities/repeat_zone.html
- Par **Repeat Input / Repeat Output**; **Iterations** (int) lido antes de começar; itens de estado passam de iteração em iteração; links externos são constantes durante o loop; dentro da zona há input **Iteration** (contador atual).
- **Inspection Index**: escolhe QUAL iteração o Viewer/socket-inspection mostra (debug de loop!).

### For Each Geometry Element Zone (4.3)
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/utilities/for_each_geometry_zone.html
- Propriedade **Domain**; inputs da zona: Geometry, Selection, e dentro: **Index** + **Element** (a geometria de UM elemento — "não computado se não usado", e a doc avisa que é caro).
- Saídas em 2 painéis: **Main Geometry** = cada valor single de dentro vira atributo no geometry principal no índice corrente; **Generation** = geometrias geradas por elemento são **joined** (valores não-geometria viram atributo anônimo na geometria logo acima na lista).
- **Inspection Index** para debug. Aviso oficial de performance: "performance of simple field evaluation is way superior" — a zona é para gerar geometria complexa por elemento (uma árvore por curva), não para math por elemento.

### Bake Node (4.1)
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/geometry/operations/bake.html
- **Bake Mode**: Still (frame atual) | Animation (range da cena ou Custom Range Start/End).
- **Bake Target**: Inherit from Modifier | **Packed** (no .blend, default desde 4.3) | Disk (+Custom Path). Botões: Bake, Pack/Unpack, Delete. Formato não é import/export (sem garantia entre versões).
- **Bake Items**: lista dinâmica; cada item tem Socket Type, **Attribute Domain**, e flag **Is Attribute** (baka um field junto da geometria). Cada input ganha output espelho (passthrough).

### Attribute Statistic (nosso reduce)
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/attribute/attribute_statistic.html
- Inputs: Geometry, **Selection** (bool field filtra o dataset), Attribute (field). Props: Data Type Float|Vector (vector = elementwise), Domain.
- **8 saídas simultâneas**: Mean, Median, Sum, Min, Max, Range (max−min), Standard Deviation, Variance.

### Accumulate Field (nosso scan)
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/utilities/field/accumulate_field.html
- Inputs: Value, **Group Index** ("bin" — acumulações independentes por grupo; valores específicos não importam, só a igualdade). Data Type: Float/Int/Vector/**Transform** (matriz!). Domain.
- Saídas: **Leading** (running total começando no 1º valor = scan inclusivo), **Trailing** (começando em zero = scan exclusivo), **Total** (broadcast do total do grupo). Warning de overflow int documentado.

### Random Value
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/utilities/random_value.html
- Data Type: Float | Integer | Vector | **Boolean**. Min/Max (float/int/vector) ou **Probability** (boolean).
- **ID** (default: atributo `id` se existe, senão índice) + **Seed** separado ("different set of random values, even for two nodes with the same ID"). Truque documentado: ligar um Integer constante no ID ⇒ UM valor random (o diamante-com-ponto na prática).

### Map Range
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/utilities/math/map_range.html
- From Min/Max → To Min/Max; Data Type Float|Vector; **Interpolation**: Linear | **Stepped Linear** (+input Steps = quantização) | **Smooth Step** (Hermite) | **Smoother Step**; **Clamp** toggle.

### Float Curve (curva como widget)
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/utilities/math/float_curve.html
- Inputs: **Factor** (influência 0–1, lerp entre input e curva) + Value. Propriedade: o **Curve widget** padrão do Blender embutido no corpo do nó (pontos arrastáveis, handles, presets, clipping). Output float. É um field node — a curva roda por elemento.
- Limitação histórica (workshop jul/2025): **curve/ramp não podem ser inputs de node group** — a solução em curso é tratá-los como **closures** com subtipo de widget, mostrando o widget colapsado no nó do grupo (expande ao clicar).

### Sample Curve
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/curve/sample/sample_curve.html
- Mode Factor|Length; input **Value** (field arbitrário avaliado na curva e devolvido interpolado no ponto amostrado — gather genérico); **Curve Index** (qual spline; ignorado com **All Curves** = comprimento acumulado global); Data Type do Value.
- Saídas: Value, Position, **Tangent** (normalizado), **Normal**. Interpolação linear entre pontos avaliados.

### Resample / Trim / Fillet (curve ops)
URLs: `.../curve/operations/resample_curve.html`, `trim_curve.html`, `fillet_curve.html`
- Resample: Mode Count | Length | Evaluated; Count/Length aceitam **field por spline**; Selection.
- Trim: Mode Factor | Length; Start/End; Start>End ⇒ ponto único; não suporta cíclicas (warning).
- Fillet: Method Bézier | Poly (+Count); Radius (field, por ponto); **Limit Radius** anti-overlap.

### Instance on Points
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/instances/instance_on_points.html
- Points, Selection, Instance (geometria OU lista de instâncias), **Pick Instances** (bool) + **Instance Index** (default = `id`, senão índice; **wrap** nos dois sentidos), Rotation (socket de rotação), Scale (vetor). `id` copiado para as instâncias. Nested instancing composto com o transform do pai.

### Distribute Points on Faces
URL: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/point/distribute_points_on_faces.html
- Method **Random** | **Poisson Disk** (Distance Min = raio mínimo; Density Max × Density-field; cap documentado: densidade acima do que a distância permite satura). Seed. Selection.
- **Gera `id` estável**: "When the mesh is deformed or the density changes the values will be consistent for each remaining point" — é a espinha do random estável deles (Random Value/Instance on Points consomem).
- Saídas: Points, Normal, Rotation (Euler do normal; eixo Z arbitrário, documentado).

### Sort Elements / Points to Curves (par de reordenação)
URLs: `.../geometry/operations/sort_elements.html`, `.../point/points_to_curves.html`
- Sort: Selection + **Group ID** (ordena dentro de grupos) + **Sort Weight** (chave); domínio Point/Edge/Face/Spline/**Instance**; estável (não-selecionado mantém índice; empate mantém ordem relativa).
- Points to Curves: **Curve Group ID** (mesmo ID = mesma curva) + **Weight** (ordena pontos dentro da curva; empate = ordem original).

### Gizmo nodes (4.3) — Linear / Dial / Transform
URLs: https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/gizmos.html + `.../input/gizmo/{linear,dial,transform}_gizmo.html`
- **Value é um socket especial multi-link invertido**: TUDO que estiver ligado nele é *modificado* quando o gizmo se move (múltiplos valores mudam juntos; Multiply/Divide no caminho muda a taxa de cada um).
- **Dependência bidirecional é o coração**: se a Position do gizmo não depender do valor controlado, o gizmo "salta de volta à origem" ao soltar (documentado como o erro clássico). Setup correto: valor → posição do gizmo E gizmo → valor.
- Linear: Position, **Direction**; props Color (cor do tema), **Draw Style** (estilos: seta/cruz/caixa). Dial: Position, **Up**, **Screen Space** (tamanho constante na tela) + Radius (fator em screen-space, unidades caso contrário). Transform: Position, Rotation; sub-gizmos de translação/rotação/escala **desligáveis** individualmente; respeita orientação Global/Local do viewport.
- Saída **Transform** de todo gizmo node "should be joined into the geometry" — é o que faz o gizmo acompanhar o transform da geometria depois.
- UI: socket com gizmo ganha **ícone**; gizmo aparece com o nó selecionado; clicar no ícone **pina**; **propagação** por links (funciona através de muitos nós de math; "double link" indica sucesso) até Group Input e daí ao **modifier** (gizmo do modifier aparece sempre que o modifier está ativo). Nós builtin ainda não têm gizmos próprios (4.5).

### Menu Switch / Switch / Index Switch
URLs: `.../utilities/menu_switch.html`, `.../utilities/switch.html`
- Switch: bool; **"Only the input that is passed through the node is computed"** (lazy nos dois ramos). Type = todos os tipos de socket.
- Menu Switch: itens user-defined (add/remove/rename/reorder no sidebar); renomear **preserva os links**; menu vira socket → exposto no group input/modifier como dropdown; **conflito documentado**: dois Menu Switch diferentes no mesmo socket = erro (mesmo com itens iguais); o workaround oficial é embrulhar num node group. 4.5: menu switch consegue chavear menus.
- Expanded (group interface): menu pode ser desenhado expandido (radio) em vez de dropdown.

### Viewer / Spreadsheet / Inspection
URLs: `.../output/viewer.html`, https://docs.blender.org/manual/en/4.5/editors/spreadsheet.html, `.../modeling/geometry_nodes/inspection.html`
- Viewer: **Ctrl+Shift+LMB** num nó/socket liga ao viewer e ativa (em espaço vazio desativa); **atalhos numéricos** (Ctrl+1..9 atribui, 1..9 ativa; 4.5); Geometry + **Value** (1 field); Domain **Auto** com fallback documentado (face-corner em mesh, point em curva); overlay no viewport pode mostrar **valores como texto** (4.1) e cor; spreadsheet mostra a coluna "Viewer" do domínio escolhido; **pin** no spreadsheet segura o viewer mesmo inativo; desativa sozinho ao trocar de objeto/sair do grupo.
- Spreadsheet: coluna=atributo, linha=elemento; região Data Set: **Evaluation State = Evaluated | Original | Viewer Node** + **Viewer Path** (mostra a cadeia de grupos até o viewer aninhado); navegação por **geometrias aninhadas** (instâncias; 4.3); domínios com **contagem de elementos**; **Row Filters** no sidebar (coluna + Equal/Greater/Less + Value + Threshold); lock a objeto; tooltips com valores crus; colunas redimensionáveis/reordenáveis.
- Inspection: **tooltip de socket com o valor da última avaliação** (primitivos = valor; geometria = tipos+contagens); só loga se o nó alcança o Group Output; não loga durante render. **Warnings no título do nó** (ícone + hover). **Node timings overlay** (frames somam o conteúdo; Group Output = total; field nodes não têm tempo próprio — o custo aparece no data-flow node que os avalia). **Named Attributes overlay** (quem lê/escreve/remove cada nome). **Geometry Randomization** (Developer Extras): embaralha índices de saída para provar que um setup não depende de ordem instável — teste de contrato de determinismo.

### Group interface (panels etc.) — ver C)

---

## C) UI/UX — o que copiar (e como eles desenham)

### 1. Sockets & links
- Forma comunica avaliação: **círculo=single, diamante=field, diamante-com-ponto=field-que-é-constante**; link de field **tracejado**; erro de tipo = link **vermelho sólido**; multi-input = socket em forma de **pílula** (aceita N links). Conversões implícitas existem (float↔int↔bool↔color/vector) e o link mostra o resultado; campo cinza quando input não afeta o output (groups: "Input values that do not affect the output will be greyed out").
- Mute de NÓ (`M`): links ficam **vermelhos** e o nó vira passthrough com "internal links" inteligentes (4.5 melhorou os do Switch). Mute de LINK (Ctrl+Alt+RMB desenhando um traço). Cut links (Ctrl+RMB desenhando). **Swap de links com Alt**. Ctrl+drag de um output **move** todos os links de saída. Auto-insert: arrastar nó sobre um link o insere (Alt desativa); nós vizinhos se afastam (Auto-Offset, `T`).
- Reroute: dot que também carrega field-ness; mutar os links de entrada de um reroute muta os de saída.
- URL: https://docs.blender.org/manual/en/4.5/interface/controls/nodes/editing.html

### 2. Zones (a "cinta")
- Uma zona é um **par de nós** (Input/Output) e a região entre eles é desenhada como um **fundo/overlay que abraça os dois nós e tudo dentro** (retângulo arredondado atrás dos nós; os nós do par não podem ser separados da relação — deletar um deleta o par). Zonas aninham (sim dentro de repeat etc., com regras).
- Regra dura que simplifica tudo: **nenhum link sai de dentro da zona por fora do nó Output** — é isso que permite cache, skip, e sub-frame interpolation. Links de fora para dentro: viram constantes por iteração/passo.
- **Itens de estado dinâmicos**: arrastar um link no **socket vazio** do par cria um item novo dos dois lados ao mesmo tempo (input e output espelhados); renomear com Ctrl+LMB; painel no sidebar gerencia (add/remove/reorder/type).
- **Inspection Index** (repeat/for-each) escolhe qual iteração o viewer mostra — debug de loop resolvido com UM inteiro.
- 4.5: **link-drag-search cria zonas** (soltar o fio no vazio → busca → escolhe "Simulation Zone" → o par nasce conectado ao fio).

### 3. Node panels & group interface (4.x)
URL: https://docs.blender.org/manual/en/4.5/interface/controls/nodes/groups.html
- O **Group Sockets panel** é uma tree-view única com Inputs, Outputs e **Panels** (seções colapsáveis DENTRO do nó); panels aninham por drag; **Closed by Default** por panel; panels sempre desenham após os sockets soltos.
- **Panel Toggle**: converte um input boolean no **checkbox do header do panel** (o socket some da lista e vira ícone ao lado do nome; "Unlink Panel Toggle" desfaz). ⚠️ Documentado: o toggle **não desabilita/acinzenta os sockets do panel** — o autor tem que ligar um Switch por conta (fonte de knob-morto; nosso app resolveria com a lei do knob-morto).
- Por socket: **Description** (tooltip), **Default**, **Min/Max** ("this is not a minimum or maximum for the data that can pass through" — clamp SÓ de UI, valor maior passa intacto ⚠️), **Vector Dimensions 2/3/4** (4.5), **Expanded** (menu radio), **Default Input** (input implícito quando desconectado: Position/Normal/Index/ID… 4.5 somou Left/Right Handle e propagação de default inputs de nós builtin p/ grupos), **Hide Value**, **Hide in Modifier**, **Structure Type** (Auto | **Single** = recusa fields).
- **Make Group (Ctrl+G) de um nó único preserva a interface inteira** (panels, defaults, nome) — group de N nós gera sockets das conexões cortadas. Insert Into Group move nós para dentro atualizando a interface. Tab entra/sai com **breadcrumbs**. Color Tag por grupo; **Usage: Modifier/Tool** filtra onde o grupo aparece; nome iniciando com `.` esconde dos menus (assets internos).
- Futuro (workshop jul/2025): "panel" vira **"layout"** com tipos panel/**column/row** (mais de uma prop por linha); autoria livre + validação só no uso (warnings), para não travar estados intermediários.

### 4. Link drag-search
- Soltar um link no vazio abre **busca filtrada pelo tipo do socket**, listando `Nó ▸ Socket` compatível (cada entrada já diz EM QUAL socket vai ligar); cria o nó já conectado. 4.5 estendeu para **zonas**. Release notes: https://developer.blender.org/docs/release_notes/4.5/geometry_nodes/

### 5. Add menu / assets
- Add menu por categorias fixas (Attribute, Curve, …, Zones) + **node group assets aparecem no MESMO menu** na posição do seu catálogo de asset (catalog path = menu path — workshop jul/2025 quer separar os dois e permitir múltiplos paths por asset + variáveis `{OBJECT_CONTEXT}`). Assets marcados Tool aparecem no menu do viewport. 5.0: **Essentials** — modifiers oficiais feitos de GN (array/scatter) + grupos utilitários, via **packing** (linked+embutido).
- **Asset versioning** (planejado): versão semântica no nome do arquivo (`materials_v1.0.0.blend`).

### 6. Viewer/spreadsheet/debug (resumo operacional)
- Ctrl+Shift+LMB é o gesto único de "me mostre isto"; spreadsheet é o segundo monitor do grafo (Original/Evaluated/Viewer + filtros); tooltips de socket = valores reais da última avaliação; warnings viram ícone no título e **sobem para o modifier organizados por severidade** (4.3), com **Warning node** para autores e setting de **propagação**; timings por nó no overlay; overlay de atributos nomeados; **Geometry Randomization** como teste de dependência de ordem.
- Workshops 2025: viewer generalizado (multi-input dinâmico, decide visualização pelo dado), **custom viewers** (grupo marcado como viewer), **debug views** herdáveis por grupo, desenho abstrato do grafo com **frames em destaque no zoom-out** / minimap (pedindo mockups da comunidade).

---

## D) ARTISTA — assets, defaults, aprendizado, dores conhecidas

### O que funciona para o artista
- **Fields tornam o comum curto**: scatter → instance → random é 3 nós; o `id` estável do Distribute + `ID/Seed` do Random Value dá variação estável "de graça" sem o artista pensar em RNG.
- **Zonas com Inspection Index + viewer** dão debug de loop/sim sem printf.
- **Gizmos (4.3)**: o autor do asset esconde o grafo e o usuário manipula no viewport — inclusive só com o modifier, sem nunca abrir o node editor. Propagação automática até o modifier é o que fecha o ciclo.
- **Menu Switch** deixa o autor de asset criar enums de verdade na UI do modifier.
- **Panels + toggles + descriptions + min/max + hide-in-modifier**: a interface do grupo É a UI final do artista (o modifier renderiza os panels desde 4.1).

### Dores confirmadas (não copiar)
1. **Fields são a barreira conceitual nº 1** — thread "Too frustrating to learn Geometry Nodes" (blenderartists): tutoriais mostram *o quê* ligar, nunca *por quê*; o manual documenta nó-a-nó sem o modelo mental. O modelo "campo avaliado no contexto de quem consome" faz o mesmo sub-grafo dar valores diferentes em dois lugares — poderoso e desorientante. (Nosso modelo de colunas explícitas por stream é mais literal; manter.)
2. **Verbosidade/baixo nível**: devtalk "Geometry nodes can't be used in studios due to avoidable forced repetition" — setups grandes repetem clichês (capture→process→store); a resposta oficial veio como assets Essentials (5.0) + closures/bundles, anos depois.
3. **Curve/ColorRamp não entram em group interface** — "long-standing and annoying limitation" (workshop jul/2025); só agora resolvendo via closures com widget inline. Lição: se o PH2D fizer Float Curve, projetar o widget para ser **promovível a parâmetro de subgrafo desde o dia 1**.
4. **Min/Max de socket que não clampa o dado** — surpresa silenciosa documentada no próprio manual.
5. **Panel Toggle não desabilita os sockets** — o painel parece desligado mas os knobs seguem vivos; cada autor reimplementa o gate com Switch.
6. **Menu Switch conflita** ao ligar dois menus no mesmo socket (mesmo idênticos); workaround oficial é embrulhar em grupo.
7. **Anonymous attributes não atravessam a Simulation Zone** a menos que virem item de estado — pega os usuários (nota no manual).
8. **For Each é lento por design** e o manual precisa avisar para não usá-lo onde field resolve — um escape hatch que vira armadilha de performance.
9. **Auto-reconnect imprevisível de links** (devtalk UX thread: candidatos hardcoded por socket, "difficult for the user to anticipate"), reroute com direção ambígua, auto-offset intrusivo, **datablocks vazios acumulando** ao criar modifiers — microfricções que somam.
10. **Skip checkbox** da sim zone escondido em 4.3 porque era clicado por acidente — controle destrutivo não mora no corpo do nó.
11. **Dependência de ordem de índices** é bug latente clássico — a resposta deles é a ferramenta **Geometry Randomization** (embaralha de propósito para o autor testar). Paralelo direto ao nosso determinismo por contrato.
12. **Bake**: formato sem garantia entre versões; volumes/materiais chegaram aos poucos (4.1); packed só em 4.3 — bake distribuído em N nós sem história de versão é dor real de pipeline.
13. **Viewer vê 1 field por vez** — reconhecido; viewer generalizado multi-dado está em redesign (workshops 2025).
14. O que os devs mesmos listam como faltante (workshops jul+set/2025): **lists** (experimental), **bundles/closures** (5.0; física declarativa por bundles com XPBD/Jolt/Box2D), **object bundle output** (dados não-geometria saindo do modifier — decidido: bundle DENTRO do geometry set), **custom defaults por node-group**, **modal node tools** (nó "Modal Event" = keymap dirigindo o grafo), **multi-object tools**, **asset versioning**.

### URLs (comunidade/dev)
- https://devtalk.blender.org/t/proposal-addressing-geometry-nodes-ux-issues/23239
- https://devtalk.blender.org/t/geometry-nodes-cant-be-used-in-studios-due-to-avoidable-forced-repetition/21627
- https://blenderartists.org/t/too-fustrating-to-learn-geometry-nodes/1554693
- https://code.blender.org/2025/07/geometry-nodes-workshop-july-2025/ · https://code.blender.org/2025/10/geometry-nodes-workshop-september-2025/
- Release notes: https://developer.blender.org/docs/release_notes/4.1/geometry_nodes/ · https://developer.blender.org/docs/release_notes/4.3/geometry_nodes/ · https://developer.blender.org/docs/release_notes/4.4/geometry_nodes/ · https://developer.blender.org/docs/release_notes/4.5/geometry_nodes/

### Síntese acionável para PH2D (1 parágrafo)
O que mais vale copiar: (1) **forma de socket = semântica de avaliação** com o 3º estado "field-mas-constante" e link tracejado — mapeia 1:1 para coluna-vs-uniform nas nossas streams; (2) **Inspection Index** em qualquer contêiner iterativo; (3) o trio **socket-tooltip-com-valor-real / warnings-no-título-propagáveis / timings-overlay** (nosso probe já é meio caminho); (4) **spreadsheet** como vista de colunas com filtros e estado Original/Evaluated/Viewer; (5) **gizmo nodes bidirecionais com propagação** (Value multi-link invertido + "posição deve depender do valor"); (6) **Menu Switch** como enum autorável; (7) **Accumulate com Group Index** (scan segmentado) e **Attribute Statistic com Selection** completando nosso reduce; (8) **link-drag-search que cria zonas**; (9) drop-de-CSV virando nó de dados. O que evitar está na lista de 14 dores — em especial: escape-hatch lento (for-each), widgets que não promovem a parâmetro, toggles que não gateiam, min/max cosmético e qualquer contrato implícito de ordem de elementos.
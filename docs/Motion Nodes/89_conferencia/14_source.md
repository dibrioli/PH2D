# 14 — SOURCE (`source.object` · `source.shape`)

**Conferência do [plano 89](../89_plano_conferencia_dos_nos.md) §3.** Linha `line/motion-value`, 2026-08-09.
Referência autoritativa: [`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) §A.1 (Shapes) e §Duplicator ·
[`referencia_pesquisa_blender_gn.md`](../referencia_pesquisa_blender_gn.md) l.17/l.60 ·
[`referencia_pesquisa_niagara_stardust.md`](../referencia_pesquisa_niagara_stardust.md) l.41 · AE (layers/shape layers).

**Os dois nós mais JOVENS do catálogo** (`source.object` nasceu 2026-08-02, `source.shape` 2026-08-04) e a
única porta pela qual *o que se desenha* entra num grafo que até então só produzia *onde*.

---

## §1 — O que os params são HOJE (lidos do `MANIFEST`, não do doc)

| nó | f32 params (`MANIFEST.params`) | text param | widget |
|---|---|---|---|
| `source.object` | **NENHUM** (`params: &[]`) | `object` (nome do objeto) | `ParamWidget::Source` (picker) |
| `source.shape` | **9** — `kind` `size` `aspect` `sides` `corner` `star_depth` `cleft` `tooth_depth` `hole` | — | `Enum` (8 rótulos) + 7 sliders + 1 `IntSlider`, com `ParamGate` por-kind |

⚠️ **`source.object` é o nó mais magro do catálogo no eixo f32 — zero.** Isso é *natureza* na metade
(*qual objeto* é uma string, não um número) e **omissão** na outra (as referências dão 3-6 controles ao
mesmo nó — ver §2).

⚠️ **O que cada nó PUBLICA no stream** (a metade que a tabela de params não mostra e que decide os gaps):

- `source.object` → sprite: `(P=origem, size, tint, uv_rect, texture_id)` · vetor: `(P=origem, size, tint=BRANCO, geometry_id)` · Flip: tile assada no frame atual · grupo nomeado: **N instâncias vivas** (`group_externals`).
- `source.shape` → **`(P=origem, geometry_id)` e MAIS NADA.** Sem `size`, sem `tint`, sem `rot` — o doc-comment é explícito: *"uma forma nua é branca na origem"*.

---

## §2 — A tabela (colunas fixas da §3 do plano 89)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `source.object` | 0 f32 + text `object` | **a POSE do objeto não viaja com o template** — `Transform Space: Original \| Relative` (Blender GN *Object Info*, manual 4.5 Input▸Scene); Cavalry Duplicator documenta a escolha oposta (*"transforms do input no nível-pai são **ignorados** (filhos respeitados)"*, cavalry §Duplicator l.126) | **NÃO.** Tentado: `source.object → motion.rotate/motion.scale` — gira/escala TODAS as cópias por um número que o artista **digita**, não SEGUE o objeto; manter os dois em sincronia é a falha de duas-portas. ⚠️ O `Transform` já está na query (`motion_bridge_objects.rs:147`) e é **descartado** fora do canal `position_of` | **omissão** (o dado está em mãos) | P1 | `space = Relative` ⇒ o template de hoje, byte-idêntico |
| `source.object` | idem | **QUAL FRAME do objeto animado** — Cavalry **`Shape Time Offset`** (*"retima a animação de cada cópia — par canônico com Stagger"*, cavalry l.131) · Stardust **Clone** (*"ramifica sistema com **time-shift**+reseed"*, stardust l.41) · AE Time Remap | **NÃO, e o mecanismo é estrutural:** `motion.time_remap` escopa o tempo do **COOK**, mas a tile do Flip é assada **no SHELL, antes do pump, UMA por NOME, no frame atual do app** (`bake_flip_objects`) ⇒ dois `source.object` do mesmo Flip recebem a **MESMA** tile. Nenhum nó a jusante pode pedir outro frame | **omissão** | ~~**P0**~~ ✅ **FECHADO em 2026-08-13** — param `time_offset` (SEGUNDOS, não quadros: cada Flip carrega o próprio `fps`, então um offset em quadros deslocaria dois objetos da mesma cena por tempos diferentes; quantos desenhos ele pula é `fps × offset`, uma consequência e não um segundo controle). A porta é `ph2d_nodegraph::external::appearance_of`, ao lado do `is_reserved` que guarda o namespace que ela cunha — a **quarta pergunta sobre o mesmo nome**, depois de `position_of` e `curve_of`, e a primeira que é sobre QUANDO. ⚠️ **A cache do bake passa a ser chaveada pelo QUADRO RESOLVIDO, não pelo offset**, e é isso que fecha o perigo do param DIRIGIDO POR FIO (doc 58): um `value.lfo` no offset cunharia uma tile por quadro de app com uma chave por-offset; pelo quadro, dois offsets no mesmo desenho **são a mesma tile**, e o despejo por quadro limita o conjunto vivo à contagem de nós. ⚠️ **E a membrana escreve DUAS vezes:** a cópia TRANSPARENTE do canal cru (um sprite não tem animação própria — sem ela o objeto SUMIRIA) e depois a tile do Flip deslocado, que vence | `time_offset = 0` ⇒ o frame atual de hoje, e é o **nome cru**: nenhuma chave, nenhuma tile, nenhum canal a mais |
| `source.object` | idem | **escolher UM de vários objetos por índice/aleatório** — Blender `Instance on Points → **Pick Instances**` (e o próprio repo já o lista como faltante: blender-gn l.17 *"falta o `Pick Instance` por índice de uma lista de variantes"*) · Cavalry Duplicator **`Auto Id`(bool)/`Shape Id`(int)** (l.130) · C4D Cloner iterate/random/blend/sort | **NÃO.** Tentado: `motion.mixer` funde dois `source.object` num stream de 2 linhas, e o `motion.duplicator` faz **produto cartesiano** (`duplicate(shape, points, np)`, `params: &[]`, count = `shapes·points`) ⇒ 2 formas × N pontos = **2N cópias, todas de ambas** — não N escolhendo entre 2. `value.switch` opera sobre VALOR, não escolhe LINHA de outro stream | **omissão** | P1 (é upgrade do `motion.duplicator`, não do source) | `pick = Off` ⇒ o produto cartesiano de hoje |
| `source.object` | idem | **sprite KTX2 é fonte INVISÍVEL** — `SpriteSource::CookedTexture ⇒ None` (`motion_bridge_objects.rs:88`) | n/a (é cobertura, não param) | **cerca declarada** (*"deferred to a later wave; it is skipped, not guessed"*) — ver §4 | P2 | — |
| `source.shape` | 43 kinds | **~~39 formas já existem no repo e são inalcançáveis do grafo~~** | — | **FEITO** — o nó passou a cozinhar por `ph2d_vec_scene::cook`, a porta única que ele era o único chamador a não usar | ✅ | tabela índice→`ShapeKind` no SHELL (o nó não alcança a lib de vetor), os 8 índices ONDE ESTAVAM, os 35 apendados; gate compara contra o construtor CONGELADO ⚠️ e a igualdade **não é bit a bit e nunca foi**: o círculo tinha DUAS derivações do mesmo número (`KAPPA` literal × `(4/3)·tan(α/4)`) que tinham deslizado **1,7e-12**. A wave colapsa as duas portas; o número mede a distância que havia |
| `source.shape` | idem | **sem TRAÇO (stroke)** — width/color/cap/join/dash em toda Shape da Cavalry; AE Shape Layer **Stroke**; Illustrator | **PARCIAL, medido.** O renderer **já desenha traço**: `tessellate_shape_instance` ramifica em `path.fill.is_some() \|\| path.stroke.is_some()` → `path_tess` → `draw_path_with` (`ph2d-vec-render/src/instance.rs:38-51`). O que falta é o shell **pôr** fill/stroke no `VecPath` do primitivo (hoje `..VecPath::default()` = ambos `None`). Escape que FUNCIONA: desenhar o traço na ferramenta Vector, nomear, trazer por `source.object` (é o "vetor-DOCUMENTO" do mesmo arquivo) — **mas perde o procedural** (não dá para animar `sides`). ⚠️ **Colide com a cerca §4.4** (pôr paint no primitivo o tira do caminho tingível do `motion.tint`) | ✅ **FECHADO** (2026-08-12) — `stroke_width` + a cor do traço num **SWATCH** (`ParamWidget::Color`, nunca quatro sliders lineares: a lei que o `motion.tint` escreve ao lado do dele, *"um `0,5` linear lê como cinza claro"*), postos no `VecPath` pelo shell. ⚠️ **A cor do traço é PRÓPRIA e tem de ser** — o preenchimento vem do `tint` da instância, então um traço que a herdasse seria invisível. ⚠️ **E a CHAVE do cache era o risco maior:** a `shape_key` **enumerava os nove campos à mão**, e uma chave que enumera as entradas de um valor é como a próxima é esquecida — o traço não mintaria entrada nova, a forma antiga voltaria do store, e o controle ficaria **inerte depois da primeira vez** (o defeito exacto do *Pattern Offset*, 2026-08-09). Ela passou a ser **DERIVADA de `param::ALL`**, e o gate que a vigia também (ele enumerava mutadores — a mesma doença um nível acima) ~~**P0** (é o controle que separa *forma* de *silhueta*)~~ | `stroke_width = 0` ⇒ `stroke: None` ⇒ o preenchimento-por-`tint` de hoje, byte-idêntico |
| `source.shape` | idem | **sem COR própria (fill)** — idem Cavalry/AE/Illustrator | **SIM** — `source.shape → motion.tint` (o `tint` da instância pinta o primitivo, `instance.rs:75`; o picker OKLCH já existe no `motion.tint`). Custo: 2 nós para uma cor | omissão de ERGONOMIA | P2 | fill herda o `tint` da instância ⇒ o de hoje |
| `source.shape` | idem | **sem ROTAÇÃO própria** (uma estrela apontando para cima) | **SIM** — `source.shape → motion.rotate` escreve `basis`, e o `encode` monta `pose = translate(P)·R(basis)·scale(size)` | ergonomia | P2 | `rotation = 0` |
| `source.shape` | idem | **`fill_rule` não é exposto** — SVG `fill-rule`; Illustrator *Even-Odd*; Cavalry **"Fill Rule"** listado como FALTA (cavalry l.70). Uma estrela de `sides` alto com `star_depth` alto **auto-intersecta**, e nonzero × evenodd são visivelmente diferentes | **NÃO** — nenhum nó escreve `VecPath.fill_rule`, e o renderer já o honra (`ph2d-vec-render/src/lib.rs:91`) | **omissão** de fiação | P1 | `NonZero` (o que `VecPath::default()` dá hoje) |
| `source.shape` | idem | **sweep / start / inner** (pizza, rosquinha, anel parcial) | — | **PARCIAL — a FORMA chegou, o CONTROLO não.** `Pie` e `Segment` estão no catálogo com as proporções canónicas da biblioteca; o que falta é o artista mexer em `sweep`/`start`/`inner`, que é a wave dos knobs por-forma | P1 | os defaults do `cook()` |
| `source.shape` | idem | **sem raio POR CANTO nem *corner smoothing* (squircle)** — Figma/Cavalry *Corner Smoothing*; Cavalry **Super Ellipse**; iOS squircle. `rounded_rect_corners(a,b,radii,smoothing)` + `round_rect_radii(base,[3 offsets])` **existem** e são o `RoundRect` do `cook()`; `source.shape` usa `rounded_rect(a,b,radius)` (raio único, sem smoothing) | **NÃO** | **omissão** de fiação | P1 (cai junto) | `radii = [r;4]`, `smoothing = 0` ⇒ o round-rect uniforme de hoje |
| `source.shape` | idem | **sem TRIM / dash** — Cavalry *Trim Path*; AE **Trim Paths** (o item mais usado de shape layer). `ph2d_vec_scene::trim_path(path, start, end)` **existe** (`marker.rs:395`) | **NÃO** — nenhum nó do grafo alcança `trim_path` | **omissão** | P1 | `start=0, end=1` ⇒ o path inteiro |
| `source.shape` | idem | **`size` é GEOMETRIA, não coluna** — Blender GN separa geometria de instância; Cavalry escala a CÓPIA sem re-cozinhar. Aqui o `size` entra no `shape_key` ⇒ **um slider animado re-interna um `VecPath` por valor visitado** (o próprio doc do `VecPathStore` admite: *"an animated slider re-interns each value"*), e nada a jusante que leia a coluna `size` vê o tamanho da forma | **PARCIAL** — `motion.scale` a jusante escala a instância (barato, e é a rota certa); o que não é exprimível é *animar o `size` do nó sem crescer a store* | omissão de DESENHO (perf + contrato de coluna) | P1 | publicar `size` como coluna com o valor de hoje e construir a geometria em raio 1 ⇒ mesma imagem |

---

## §3 — `ESPÉCIES DE FONTE QUE FALTAM:`

A pergunta que a tabela de params não faz. Hoje o `source.object` resolve **quatro** espécies
(`Sprite` · `VecPathRef` · `FlipObjectRef` · `GroupedChildren`) e o `source.shape` **uma** (primitiva
paramétrica). O que falta, em ordem de quanto aparece na tela:

1. **TEXTO — a maior, e a mais cara.** Cavalry **Text Shape**: *String (+Generators) · Font (variable fonts com eixos!) · Font Size · Char/Word/Line/Paragraph Spacing · Alignment · Text Box + Shrink to Fit · **Text Path** · **Background Shape** · Formatting Inputs `{0}`*, e *"Animação por caractere = Sub-Mesh behaviour"* (cavalry l.171-172; status já registrado **FALTA (motion não tem texto)**, l.19). Stardust: **Emitter Text/Mask** (l.41). AE: Text Layer + Animators. **Exprimível? NÃO** — não há nó de texto, e embora `ph2d-vector-font` exista como foundational, **nada no grafo o alcança**: a membrana só resolve as quatro espécies acima. *Metade do mograph do mundo é texto animado por caractere.* **P0.**
2. **CÂMERA / a vista.** Blender GN **`Active Camera` / `Camera Info`** (Input▸Scene; blender-gn l.60, status **PARCIAL** — *"playhead existe; camera culling não"*). Sem ela, *orientar para a câmera*, *escalar com o zoom*, *distribuir na área visível* e *culling* são todos inexprimíveis. ⚠️ **E o mecanismo de publicar já existe a UMA linha de distância:** `motion_bridge_shapes::publish_cursor` publica o external reservado `$cursor` a partir da `Camera2d` — `$camera` seria o irmão exato, pelo mesmo namespace `$` que o `is_reserved` já protege. **Exprimível? NÃO** (nada publica a câmera). **P1, custo quase zero.**
3. **UMA COLEÇÃO POR CRITÉRIO (tag / seleção / padrão de nome).** Blender **Collection Info** (`Separate Children` / `Reset Children`) · Cavalry **Shape Array** (*"arrays indexáveis de cada tipo"*, l.90, PARCIAL) · e o próprio [doc 86 §9](../86_plano_objetos_engine_render_e_preview.md) já nomeia **`source.selection`/tag** como follow-up. ⚠️ **A metade "coleção" JÁ EXISTE:** um **grupo nomeado** resolve N instâncias vivas (`group_externals`, wave A4). **Exprimível? PARCIAL** — agrupar na Hierarquia é o escape e funciona; o que não é exprimível é uma coleção cujos membros mudam sem re-agrupar à mão. **P2.**
4. **SEQUÊNCIA DE IMAGENS / footage.** Cavalry **Footage / Cel Animation / SVG** (l.40, FALTA) · Stardust **Particle: Texture** (l.41) · Niagara sprite-sheet/sub-UV. O Flip cobre *cel animation* **se o artista a desenhar no app** — e sempre no frame atual, que é exatamente o gap #2 da tabela. Importar uma sequência de PNGs como fonte não existe. **Exprimível? NÃO. P2** (é import, não grafo).
5. **O RESULTADO DE OUTRO GRAFO.** Stardust **Auxiliary** (*"emitir de outro sistema"*, l.41) e **Source** · Cavalry **Component Shape** (*"empacota rig+controles num layer reutilizável"*, l.26, FALTA) · Blender node-group como asset. **Exprimível? PARCIAL** — os subgrafos (doc 57) cobrem a reutilização DENTRO do documento; falta a fonte nomeada reutilizável ENTRE documentos. **P2.**
6. **ÁUDIO** — não é fonte de APARÊNCIA, é de VALOR, e o [doc 63 §2.8](../63_pesquisa_industria_2026_e_plano_estado_da_arte.md) já dá **P0** ao `audio.bands`. Nomeado aqui só para o consolidador não o perder na fronteira entre famílias.

---

## §4 — `CERCAS:`

Nove decisões já registradas que qualquer wave desta família tem de honrar (ou encarar de frente, com medição):

1. ✅ **ENCARADA E MEDIDA, e estava incompleta.** A cerca dizia *"All eight are FILLABLE closed shapes;
   **Arc (wedge) and Spiral are follow-ups** … Order is the wire format for the `kind` index — **append
   only**"*. A segunda metade **vale e foi honrada** (os 8 índices ficaram onde estavam). A primeira foi
   MEDIDA em vez de assumida (`ph2d-vec-scene/tests/which_shapes_close.rs`): das 47 formas do `cook()`,
   **42 fecham** e **5 não** — e a cerca nomeava só duas delas. As cinco são **Spiral · Line · Arc ·
   NoteBracket · Brace**; as outras três estavam na mesma classe sem ninguém ter olhado. ⚠️ E o fato que a
   §7.3 mandava conferir resolveu a favor: **`Pie` FECHA** (`verts=4`), então pizza/rosquinha/anel são
   preenchíveis e entraram na wave barata — só o `arc_open` precisa de traço.

2. ⚠️ **`motion_bridge_objects.rs:99`** — *"o `P` da aparência é a ORIGEM de propósito … um nó que lesse AQUELE `P` miraria a origem para todo objeto, que é como o modo Object do `motion.look_at` shipou quebrado"*. ⇒ **não "conserte" o `P` do template.** A pose viaja no canal separado `position_of(name)`.
3. ⚠️ **`motion_bridge_objects.rs`, grupo** — *"VIVO, não um composite congelado: o grupo NÃO é assado numa tile (o que o congelaria); ele emite N quads, cada um resolvendo a própria tile todo frame."* ⇒ não colapsar o grupo numa tile por performance.
4. ⚠️ **`ph2d-vec-render/src/instance.rs:58-60`** — *"Um vetor-documento vivo NÃO é re-tingido a jusante (as cores são as do desenho); a tile assada era tingível — a **troca NOMEADA** de virar vivo. Fiar `tint` pelo fill/stroke do `draw_path` é o follow-up."* ⇒ **o gap do stroke COLIDE com esta cerca** e tem de resolvê-la (pôr paint no primitivo o tira do caminho do `motion.tint`), não ignorá-la.
5. ⚠️ **`motion_bridge_gpu.rs:76-108` + gate `the_gpu_cook_recusal_placement`** — a recusa do **ADR-0155** hoje é **PARCIAL**: `source.shape` recusa o cook GPU **SEMPRE** (flag de tipo `is_live_vector_source`), e `source.object` recusa só (a) se publicar `geometry_id > 0` (content-aware, por frame — um grafo de objetos puro-sprite **fica** na GPU) ou (b) se o sufixo GPU **mudar a contagem** (a partição de runs de `texture_id` desalinharia). ⇒ **toda forma nova que eu propuser em `source.shape` herda a recusa total.** É o preço, está nomeado, e ele decide se a wave vale.
6. ⚠️ **`ph2d-node-source-object/src/lib.rs:26-29`** — objeto ausente ⇒ external vazio ⇒ stream vazio: *"o nó não emite nada, **não adivinha e não falha**"*. ⇒ não transformar em erro.
7. ⚠️ **`motion_bridge_objects.rs:88`** — `SpriteSource::CookedTexture ⇒ None`, *"deferred to a later wave; it is **skipped, not guessed**"*. ⇒ um sprite KTX2 nomeado é fonte **invisível**, por decisão declarada.
8. ⚠️ **`motion_bridge_shapes::is_reserved`** — o namespace `$` é do **EDITOR**; qualquer fonte nova (`$camera`) entra por ali, e um objeto do artista com esse nome é **recusado na publicação**, porque *"com um namespace plano, o dia em que alguém nomear um sprite `$cursor` o sprite silenciosamente VIRA o mouse"*.
9. ⚠️ **[doc 86 §9.6](../86_plano_objetos_engine_render_e_preview.md)** — *"grupo rotacionado/escalado re-orientando filhos vetor/flip"* é **decisão de arquitetura do Enio**, não conserto mecânico (força *bake canônico + linear ao vivo*, que re-abre o trade de qualidade rotação-assada-vs-viva das waves A2/A3). Mais: *"filho vetor/flip sem nome pulado"* e *"FREEZE por-nó"*.

---

## §5 — `SUPERAR:`

Quatro capacidades que **nenhuma referência tem**, derivadas do que só o nosso substrato torna barato:

**(a) A forma do GRAFO e a forma da FERRAMENTA são a MESMA forma, viva nos dois sentidos.**
Na Cavalry uma *Basic Shape* é um layer e um path desenhado é outro objeto; no AE um shape layer e um
path importado não compartilham motor. Nós temos **uma porta única de geometria paramétrica**
(`ph2d_vec_scene::cook`, cujo doc já declara que o `ShapeTool` e o re-cook da forma viva passam pelos
dois lados dela) e o `source.object` já traz um `VecPath` do documento como **vetor VIVO** (`geometry_id`,
ADR-0154). Fiar `source.shape` na MESMA `cook()` faz o catálogo de **47** formas do editor **ser** o
catálogo do grafo — e, no sentido inverso, um `source.shape` pode **materializar-se como Live Shape
editável** na cena, porque a costura fonte≠cozido do **ADR-0121** já existe. *A forma que você desenha e a
forma que o grafo gera deixam de ser duas espécies.*

**(b) A PILHA DE EFEITOS VETORIAIS (ADR-0132) sobre uma instância do grafo.**
O módulo Vector tem `PathEffect`s vivos (Falloff · Twist · Knot · Bloat · Warp · ZigZag · Trim · Repeat…)
que compõem sobre `VecPath`, e a instância do Motion já carrega um `geometry_id` que **É** um `VecPath`.
Nenhuma referência compõe *deformers de geometria vetorial* com *um instanciador de partículas*: a
Cavalry tem deformers e um Duplicator, mas o Duplicator congela; o Stardust deforma o **ESPAÇO**, não a
geometria. Aqui a composição é **dado sobre dado** — e a ADR-0154 já a nomeia como Fase 2.

**(c) `geometry_id` torna barato o que a indústria evita: N formas DIFERENTES por instância.**
Como a geometria é referência interna content-addressed (`VecPathStore`), um `value.*` pode escolher o
`geometry_id` **por linha** — *"cada partícula é uma forma diferente da mesma família"* — e a store interna
uma vez por descritor distinto. Blender **realiza** instâncias; AE **rasteriza**; a Cavalry tem `Shape Id`
mas sobre uma **lista de layers que o artista montou à mão**. O nosso pode ser **função de um campo**.

**(d) O determinismo cross-OS + o scrub bit-exato tornam a FONTE ANIMADA reprodutível.**
Onde Cavalry/Stardust cacheiam um solver, o `source.shape` é `f(params, playhead)` **puro**: uma forma cujo
`sides` é dirigido por `value.lfo` **rebobina exatamente**. E o `time_offset` que falta ao `source.object`
(o P0 da tabela) tem aqui a **mesma forma fechada** que a §10 do plano 89 descreve para o
`inherit_velocity` do emitter — nas referências ele é um acumulador de estado; aqui o frame de um objeto
animado é uma **função do playhead que já temos**.

---

## §6 — `O DOC 63 ERROU EM:`

- **Cegueira TOTAL à família — e não é "envelheceu", é *nunca viu*.** O [doc 63](../63_pesquisa_industria_2026_e_plano_estado_da_arte.md) é de 2026-07; `source.object` nasceu em **2026-08-02** e `source.shape` em **2026-08-04**. A §2 dele tem oito sub-famílias (2.1 Campo … 2.8 Áudio) e **nenhuma é SOURCE**; a §3.2 tabula 22 nós existentes e **nenhum é `source.*`**. ⇒ *o P0/P1 dele não pode ser lido como cobertura desta família.*
- **Envelheceu no sentido BOM** (marcado FALTA/PARCIAL e hoje existe), em `referencia_pesquisa_cavalry.md` §A.1: *"Basic Shape … **PARCIAL** (temos primitivas no vetor, **não como fonte no grafo motion**)"* → hoje `source.shape` **É** essa fonte (8 das 11 famílias que a Cavalry lista). E *"Editable Shape … **PARCIAL** (vetor tem, **motion não consome**)"* → hoje `source.object` consome um `VecPath` do documento como **vetor VIVO** (ADR-0154), nítido em qualquer zoom.
- **Envelheceu no sentido RUIM** (dado por resolvido e não está): a mesma tabela marca *"**Duplicator** … **TEMOS** (motion.clone + distribute_\*)"* — e (i) hoje existe um `motion.duplicator` de verdade, então a linha está imprecisa por baixo; (ii) ele **não tem `Auto Id`/`Shape Id`** (`params: &[]`, produto cartesiano puro), que é exatamente o `Pick Instances` que o `referencia_pesquisa_blender_gn.md` l.17 lista como **faltante**. ⇒ **as duas linhas de referência do repo discordam entre si sobre o mesmo buraco.**
- **A §2.5 dá P0 ao `motion.clone` v2** (multi-fonte iterate/random/blend + time offset por clone) — mas esse upgrade é hoje do **`motion.duplicator`**, não do `motion.clone`: *o nó que a §2.5 nomeia não é mais o que faz o trabalho.*

---

## §7 — Notas para a verificação do §5 do plano (o passo que é do Enio)

Os três fatos decisivos que valem re-conferir antes de qualquer wave:

1. **`ph2d_vec_scene::ALL_SHAPES` tem 47 formas** e `cook()` se declara a porta única — conferir por
   `grep -c "ShapeKind::" crates/ph2d-vec-scene/src/kind.rs` e ler o doc-comment de `cook`.
2. **O renderer já desenha traço num primitivo** se o `VecPath` carregar `fill`/`stroke`
   (`ph2d-vec-render/src/instance.rs:38-51`) — o gap é do shell, não do device.
3. **`ellipse_sweep` produz contorno fechado** (a premissa da cerca §4.1) — é o único fato que decide
   se pizza/rosquinha/anel entram na wave barata ou pedem a wave do traço.

⚠️ **E a consequência que atravessa tudo:** com a recusa do ADR-0155 como está hoje (§4.5), **todo grafo
que traz um `source.shape` cozinha na CPU** — então uma wave que multiplique as formas por seis multiplica
também o número de documentos fora da aceleração. Esse número é do Enio, não meu.

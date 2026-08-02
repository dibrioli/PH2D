# 86 — Plano: objetos da engine no grafo, o Duplicator, a ponte de render, e o preview em moldura própria

> Origem: pedido do Enio (2026-08-02, com screenshot da referência MiniCavalry V2 + `file:///home/enio/Documentos/Recursos/Nodes/MiniCavalryV2/mini-cavalry-v2.html`):
> 1. o sistema de nós do Motion **não sabe** trazer formas diferentes ou objetos da engine (sprites, vetores, Flips, compostos);
> 2. **não temos um nó como o Duplicator** da referência;
> 3. precisamos do **sistema de conexão (renderização)** com os objetos da engine;
> 4. o **preview deve sair de dentro do quadro** e ir para uma **moldura própria**, com um **botão no header** que muda a posição (acima/abaixo).
>
> Este doc é o PLANO. Não construir sem ordem de smoke por wave.

---

## §0 — Resumo executivo (o que muda, e a recomendação)

São **duas features independentes** que o pedido junta:

- **Feature A — os objetos entram no grafo e o grafo os desenha** (a grande):
  a) um jeito de o grafo **referenciar um objeto da engine** (sprite / vetor / Flip / grupo);
  b) o **Duplicator** — instanciar uma *forma* em cada *ponto* (o `Shape × Points` da referência);
  c) a **ponte de render** — o resultado do grafo desenha **os objetos de verdade**, não pontos anônimos.

- **Feature B — o preview em moldura própria** (pequena, rápida, independente):
  tirar o preview de dentro do card e pô-lo numa moldura acima/abaixo, com um botão de toggle no header.

**A descoberta que decide a arquitetura de A** (dois fatos medidos na pesquisa do código):

1. **O POD de render JÁ carrega "o que desenhar".** `ph2d_render::RenderInstance` (o mesmo que o sprite usa) tem `texture_id` + `atlas_uv` + `size` + `basis` + `tint`. O `lower_to_instances` do Motion hoje **cravа `texture_id: 0`** e lê a coluna `uv_rect` com um default. Ou seja: **um sprite instanciado já é quase de graça** — falta só uma coluna `texture_id` por-elemento e uma fonte que a preencha.
2. **A referência durável "este nó aponta AQUELE objeto" já existe e tem dois consumidores:** `ph2d_ecs::stable_name_id(Name) -> u64` (FNV-1a; usado pelo `wire_id` da timeline e pelo `body_a/body_b` da física). E a única porta app→grafo que existe hoje é o **canal External** (`Cook::set_external(name, Stream)`), consumido pelo `motion.path` para ler uma curva desenhada. **A ponte de objeto usa exatamente esse mecanismo — nada de novo.**

**Recomendação central (a decisão de produto do §9):** todo objeto vira um **quad texturizado** no render — o sprite direto, e o vetor/Flip/grupo **assados numa tile UMA vez** (bake-to-tile, cacheado por content-rev). É como toda game engine instancia sprite, escala para milhares de cópias (o ponto do Duplicator), e mantém **uma resposta só** para *"o que uma instância desenha"*. O caminho analítico por-cópia (N re-desenhos Vello/Flip, crisp em qualquer escala) fica como **modo de qualidade futuro**, não o default.

**Custo de contrato: ZERO.** Nós novos são drop-crates (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8` intactos); a coluna `texture_id` é **convenção de stream** (como `uv_rect` já é), não campo do manifesto; a ponte usa External + text-param, canais que já existem. Sem bump de `PROJECT_SCHEMA` (o grafo viaja como texto e carrega a própria versão). Prova por grep no §4.

---

## §1 — Pesquisa do estado da arte (e o que foi TENTADO e abandonado)

### O que a referência (MiniCavalry V2) faz — o desenho que o Enio apontou
Lido em `src/nodes/{render,shape,duplicator,image,gameplay/instanceSource}.js`:

- **Uma instância carrega O QUE ela desenha.** No JS o stream é um array de objetos heterogêneos, cada um com `Position/Color/Alpha/Scale/Rotation/Index/Count/Seed` **mais o payload de forma** (`type:'circle', size:…` ou `type:'image', src:…`). O nó **Shape** emite UMA instância na origem carregando sua forma.
- **Duplicator = `Shape × Points`.** Dois inputs (`shape`, `points`). Para cada shape × cada point, copia o payload da shape (`{...s}`) e SOMA a posição do point (`sp+pp`), soma rotação, renumera `Index/Count` contínuo. Sem points = pass-through. É literalmente *instanciar a forma em cada ponto*.
- **Render** é o **sink** que lê `Position/Color/Alpha/Scale/Rotation` por instância e desenha — delegando o "como" ao renderer, que faz `switch(inst.type)`.
- **instanceSource ("Objetos do Jogo")** lê as entidades VIVAS do ECS (`entityToInstance`) e as entrega ao render — **mão única, nunca escreve no mundo** (a "membrana" do documento *Dois Mundos*). É exatamente a "conexão com objetos da engine" que o Enio pediu.

### Houdini (o padrão-ouro do modelo, que o nosso já segue)
O nosso `Stream` **é** o modelo de atributos do Houdini (colunas nomeadas e tipadas, SoA — `attr.rs` diz isso na 1ª linha). O Duplicator É o **`Copy to Points`** do Houdini: pega geometria-template + um ponto-set e carimba uma cópia por ponto, herdando atributos de instância (`orient`, `pscale`, `Cd`). A diferença nossa: a *geometria-template* não cabe numa coluna f32 — ela é referenciada, e no render vira um quad (a tile assada). O Houdini resolve o mesmo com **packed primitives** / **instancing** (a geometria é referenciada, não copiada vértice-a-vértice) exatamente por custo — o mesmo motivo do bake-to-tile.

### After Effects
AE não instancia geometria vetorial em massa por design; o análogo é o **CC Repetile / o repeater do shape layer** (repete o *conteúdo do grupo* — que é rasterizado no composite) e os **Particle systems** (partículas texturizadas). A lição: quando AE precisa de "muitas cópias de uma coisa desenhada", a coisa é **rasterizada** e as cópias são quads. Nunca N re-desenhos vetoriais.

### Blender (Grease Pencil + Geometry Nodes)
- **Geometry Nodes `Instance on Points`** é o Copy-to-Points do Blender: instâncias são **referências** (não geometria copiada) até o "Realize Instances" — de novo, referência barata + realize caro só quando preciso.
- **Grease Pencil** (nosso Flip é o clone dele) **não instancia traços em massa**; o repeater vive nos modifiers e opera sobre o *stroke rasterizado no fim*. Confirma o bake-to-tile.

### Rive
Rive é runtime de **objetos animados** (a hierarquia é a cena), não um instanciador de grafo. Não tem Duplicator; instanciar uma "artboard" N vezes é N draws da artboard (analítico) — e Rive é otimizado para **poucas** artboards ricas, não milhares. Isso confirma o trade: analítico-por-cópia é para *poucas cópias ricas*; bake-to-tile é para *muitas cópias*.

### O que foi TENTADO e abandonado (por eles e por nós)
- **Copiar geometria vértice-a-vértice por cópia** (a forma ingênua do Copy-to-Points): abandonado por TODOS (Houdini packed prims, Blender instances-são-referências, AE rasteriza) — explode memória e custo. **Nós não repetimos isso:** o `src` é uma referência (índice/tile), não a geometria.
- **N re-desenhos analíticos por cópia** (o que Rive faz por não instanciar): correto para poucas cópias crisp, **inviável para o Duplicator** (cujo caso de uso são centenas/milhares). Fica como modo de qualidade opt-in, não default.
- **Carregar entidade ECS por `Entity::to_bits()`** dentro do grafo: **proibido e documentado** (`ph2d-ecs/src/name.rs:48`) — bits são id de alocação, o undo respawna com bits novos, e bits dentro de bytes de componente **envenenam o `canonicalize` do undo**. Por isso a referência durável é o `stable_name_id(Name)`, não os bits. A timeline e a física já pagaram essa lição.

**Conclusão da pesquisa:** o desenho certo é o convergente da indústria — **instância carrega uma REFERÊNCIA barata ("o que"), o "o que" vira um quad texturizado no render (assado uma vez para mídia analítica), a referência durável é o nome.** É o que o nosso substrato quase já faz.

---

## §2 — O desenho de A (a porta ÚNICA de cada pergunta)

O grafo hoje produz **ONDE** (um `Stream` de posições `P`, `Domain::Instances`); nunca **O QUE**. A referência produz **o que × onde**. As três perguntas e suas portas únicas:

### Pergunta 1 — "qual objeto é a fonte desta instância?" → o **`stable_name_id` no nó fonte** (text param)
Um **nó fonte** (`source.object`, e irmãos por conveniência — ver §8) guarda o **hash do Name** do objeto (via `Graph::set_text_param`, o canal canônico de param não-f32, que **é serializado no formato textual do grafo**, doc 32). Uma porta só: o nó não conhece ECS, conhece um nome — igual a `motion.path` conhece `"Track"`. Renomear o objeto **desacopla** (o preço documentado, o mesmo da timeline e da física).

### Pergunta 2 — "o que aquele objeto desenha?" → o **membrane `bake_source(name)` no shell** (uma porta, dois consumidores)
Antes de cada cook, para cada nó fonte, o shell resolve `name → Entity vivo` (o mapa `stable_name_id → Entity` que a timeline já reconstrói por sessão, `persist::upkeep`) e produz `(P, size, rot, tint, uv_rect, texture_id)`:

- **Sprite** → lê `Sprite` + `GlobalTransform`: a tile É `(texture_id, atlas_uv, size)` **direto** (nenhum bake; o caminho barato, que já existe no render de sprite).
- **Vetor / Flip / grupo** → **assa numa tile UMA vez** (rasteriza o objeto num offscreen via `ph2d-vec-render` / `ph2d-flip-render` / o composite do grupo, registra em `IndividualTextureStore` ou no atlas), cacheado por **content-rev**; devolve `(texture_id, uv_rect, size)`.
- **Publica via `Cook::set_external(name, Stream)`** — o `rev` do External é o content-hash, então **um objeto que muda re-cozinha o grafo** (o fingerprint já dobra o rev, `external.rs`).

⚠️ **Uma porta só, os dois lados leem as MESMAS colunas.** O membrane PUBLICA `texture_id/uv_rect/size`; o `lower_to_instances` do sink LÊ `texture_id/uv_rect/size`. O preview e o render final leem a mesma coisa ⇒ **não podem divergir** (a doença de duas portas que este repo caça). Bake é do shell porque o `eval` do nó é caixa-preta pura (não pode chamar o renderer) — exatamente o motivo de `motion.path` receber a curva por External em vez de ler o `VecScene`.

### Pergunta 3 — "onde vão as cópias?" → o **`motion.duplicator`** (um nó, dois inputs)
Nó novo, drop-crate. Inputs `shape` (as fontes) e `points` (a distribuição — `motion.grid`, `distribute-*`, `scatter`, qualquer stream de `P`). Emite `shapes × points` instâncias: para cada shape × point, **copia as colunas `texture_id/uv_rect/size/tint` da shape** e **soma `P`/`rot`** do point, renumerando `Index/Count` contínuo — o algoritmo da referência, ao pé da letra. Sem points = pass-through das shapes (Cavalry idem). É **Pure** (drop-crate, sem tocar contrato). ⚠️ **Não é o `motion.clone`** — o clone é um multiplicador polar de UM stream (1 input); o Duplicator cruza DOIS.

### A coluna `texture_id` (a ÚNICA adição de substrato) e por que ela cabe
`Column` é f32 (`Scalar/Vec2/Vec3/Vec4`). Um `texture_id` é um u32 pequeno (< 2²⁴, exato em f32). O `lower_to_instances` passa a **ler a coluna `Scalar("texture_id")` com fallback `0`** (hoje é literal `0`). **Ausente = comportamento de hoje, byte-idêntico** — todo grafo sem fonte de objeto renderiza exatamente como antes. É a mesma classe de `uv_rect` (coluna reservada da convenção com identidade). Se um dia um id passar de 2²⁴ (16 M texturas simultâneas — impossível), o número vira índice numa side-table publicada junto do External; **medir antes de escolher** (não há caso hoje).

### O sink (a "Render") — já existe
O `motion.output` é o sink (pass-through, `Domain::Instances`). O shell auto-seleciona todo `motion.output`. **Nenhum nó Render novo** — a "Render (Renderizar)" da referência É o nosso Output (círculo vermelho terminal). Talvez o display-name mude para "Render" (cosmético, §9). O que muda é o `lower_to_instances` ler `texture_id`.

### O tier — DECIDIDO pelo Enio (2026-08-02): **toggle Bake-to-tile por-nó, tile VIVA por default**
O Enio: *"teremos objetos compostos, com animação, com simulação física… o próprio nó pode ter um Toggle para a opção Bake-to-tile."* A preocupação real **não é crispness, é LIVENESS** — um objeto animado/simulado assado uma vez vira um snapshot congelado, e as cópias parariam de animar. Então **todo objeto vira um quad no render (a resposta única do §2 fica), e o nó fonte ganha um toggle que decide QUANDO a tile é refrescada:**

- **Live (default, toggle OFF)**: a tile é **re-derivada quando o objeto muda** (o content-rev sobe — animação da timeline, física, edição do composto). Custo = re-bake quando-muda + N quads. **Todas as N cópias mostram o frame ATUAL do objeto, em lockstep.** É o que um repeater de grupo animado faz no AE/Cavalry.
- **Bake-to-tile / Freeze (toggle ON)**: assa **uma vez**, congela → N quads, zero re-bake. O modo de perf, para quando o objeto não precisa animar nas cópias.

Os dois são quads no render (uma resposta só); o toggle é *"refresca a tile a cada mudança, ou congela?"*. Um 3º modo — **analítico-por-cópia** (N re-desenhos Vello/Flip, crisp em qualquer escala) — fica como opt-in FUTURO se surgir necessidade de vetor-crisp instanciado (raro; a Cavalry rasteriza).

⚠️ **Distinção que decide o escopo (registrada, não construída):** *duplicar o RENDER de um objeto simulado* (as N cópias seguem a MESMA sim, lockstep — o Duplicator) é **diferente** de *spawnar N corpos de física INDEPENDENTES* (cada cópia com sua própria sim — isso é "instanciar entidades no ECS", domínio da física, ADR-0035 diz que stream ≠ ECS). O Duplicator faz o 1º. O 2º, se o Enio quiser, é uma feature à parte (um sink que SPAWNA entidades a partir do stream), não este plano. Nomear cedo evita a expectativa errada no smoke.

⚠️ **Sub-modo de tempo por-cópia (registrado, wave futura):** um repeater rico permite **offset de tempo por cópia** (cópia 3 mostra o frame N−3 — o "stagger" temporal). Isso exige avaliar o objeto em N tempos diferentes (N renders) e é caro; o MVP é **lockstep** (todas no frame atual). Fica nomeado para não parecer um bug.

---

## §3 — O desenho de B (o preview em moldura própria + toggle)

Pergunta única: **"onde o preview de um nó é desenhado, e como sua posição é escolhida?"**

### O estado atual (medido)
- O preview HOJE é a **faixa de baixo DENTRO do card** (`geom.rs::preview_rect`, `card_h` reserva `PREVIEW_H=52` quando `preview.is_some()`).
- Ele **não é textura** — é um `Vec<[f32;2]>` de posições `P` subamostradas (≤48), desenhado como scatter de pontos (`paint_stamp.rs::draw_preview`). Barato de propósito (custo = nº de cards, não tamanho do stream).
- O header de um card tem **só** o retângulo tingido pela categoria + o título. **Não há × nem quadradinho hoje** — o × e o quadrado roxo do screenshot são da **referência Cavalry**, não do nosso app. O quadrado roxo é o botão de toggle que o Enio quer.
- ⚠️ **Já existe `GraphHitKind::PreviewToggle { node }` (editor-core) e está SEM USO** — é o gancho de vocabulário exato para o botão do header (nenhuma mudança foundational).

### O desenho
1. **Estado de posição por-nó** `PreviewPos {Above, Below}` — **runtime, panel-local** num `BTreeMap<u32, PreviewPos>` no `MotionGraphPanelState` (o análogo direto de `selected`, não-undoable, não-persistido). **Default: Below.** Persistir (sobreviver a save) é o caminho pesado (vira text-param no nó) e é **decisão do Enio** (§9) — recomendo runtime-only no MVP.
2. **Geometria da moldura** — `geom.rs::preview_frame_rect(card, pos, zoom)`: um rect ACIMA ou ABAIXO do body do card (com um vão), **FORA** do card. **`card_h` PARA de reservar `PREVIEW_H`** (some a faixa morta interna). A moldura é um painel arredondado próprio (a caixa preta do screenshot). Abaixo do card é espaço livre (sockets são nas laterais, `PAD_BOTTOM` + canvas vazio) ⇒ **sem colisão com portas/fios** (confirmado).
3. **O botão do header** — um quadradinho no header (o roxo). Desenhado em `paint.rs::draw_card` (região do header), hit-pushado via o **`GraphHitKind::PreviewToggle { node }` já existente**, despachado em `interact.rs::apply_gesture`. Clicar inverte o `PreviewPos` do nó.

⚠️ **A porta única é `preview_frame_rect(card, pos)`** — **paint E hit-test chamam a mesma** (senão o clique na moldura e o desenho dela discordam, o seam bug clássico). O rect do botão idem.

---

## §4 — Contrato congelado (§6) e schema — a prova por grep

**Feature A não toca contrato congelado nem schema:**
- Nós novos (`source.*`, `motion.duplicator`) = drop-crates com `const MANIFEST: NodeManifest {…}` (8 campos) + `impl NodeOp` (2 métodos). Prova: `cargo test -p ph2d-nodegraph --test architecture_contract_surface` continua verde (conta `pub `/`fn ` por texto; a mutação seria um campo novo no manifesto — não há).
- `texture_id` é **coluna de stream**, não campo de `NodeManifest`. Grep: `git grep -n "pub " crates/ph2d-nodegraph/src/node.rs` (bloco do `NodeManifest`) → **8 campos, inalterado**.
- Ponte usa `Cook::set_external` (existe) + `Graph::set_text_param` (existe). Sem canal novo no contrato.
- **`PROJECT_SCHEMA` intocado** — o grafo viaja como TEXTO e carrega a própria versão; o `stable_name_id` do nó fonte é text-param (record `x`, header v2, já serializado). Grep de fechamento: `git grep -n "PROJECT_SCHEMA" shells/desktop/src/project.rs` inalterado; `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` inalterados.

**Feature B não toca nada:**
- `PreviewToggle` já existe em `editor-core` (enum de interação, **não** é contrato congelado — a §6 congela `NodeOp/OpResolver/NodeManifest` e `Tool/…`, não os `GraphHitKind`).
- Estado runtime-only ⇒ nenhum schema, nenhum formato textual (a menos que o Enio peça persistir — aí vira text-param, e ainda assim sem `PROJECT_SCHEMA`).

⚠️ **O que MEXE em foundational (aditivo, isolado):** `ph2d-eval-motion::lower.rs` lê 1 coluna nova (fallback 0) — mudança de UMA linha, byte-idêntica quando ausente, gate de regressão pina isso. O membrane vive no shell (`render_loop/motion_bridge_*`). Nenhum símbolo compartilhado renomeado.

---

## §5 — A UI (as 4 condições independentes, por feature)

### Feature A
- **EXISTE**: o nó fonte tem seção de params (o picker de objeto — "Pick Object", o idioma do picker do Vector/Flip); o Duplicator tem card com 2 inputs.
- **Pintado e registrado**: os nós aparecem no palette (via `register_ui` — categoria + silhueta); o picker de objeto é um botão registrado no painel de params.
- **Clique chega ao barramento**: "Pick Object" arma um canvas-pick (o idiom `vec_path_pick`/`joint_body_pick`); o próximo clique num objeto grava o `stable_name_id` no nó (text-param) via um `GraphIntent`.
- **A SEQUÊNCIA leva a algum lugar**: escolher o objeto → o membrane assa/lê → o Duplicator carimba → o sink desenha o objeto nos pontos (visível no canvas). Sem isso é botão morto.

### Feature B
- **EXISTE**: o botão do header + a moldura externa (geom nova).
- **Pintado e registrado**: `paint.rs` desenha o botão + registra o hit `PreviewToggle`; a moldura é desenhada no rect novo.
- **Clique chega ao barramento**: `interact.rs::apply_gesture` ganha o braço `PreviewToggle` → inverte o flag.
- **A SEQUÊNCIA leva a algum lugar**: inverter o flag move a moldura acima↔abaixo no frame seguinte (visível).

---

## §6 — Os gates (red-first) e a fixture que contém o fenômeno

### Feature A
1. `the_lowering_reads_the_texture_id_column` — um stream com coluna `texture_id=[7,7]` lowera dois `RenderInstance` com `texture_id==7`; **fixture: dois elementos com ids DIFERENTES** (senão o default 0 casa). Mutação: ignorar a coluna → `0,0` → RED. **E o irmão de regressão**: stream SEM `texture_id` é byte-idêntico ao lowering de hoje (o caminho comum não pode mover um bit).
2. `the_duplicator_stamps_each_shape_at_each_point` — 2 shapes × 3 points → 6 instâncias, cada uma com o `texture_id` da SUA shape e o `P` do SEU point; **fixture: 2 shapes com ids distintos** (senão N-shapes colapsa em 1 e passa). Mutação: copiar o `texture_id` do point (não da shape) → RED.
3. `bake_source_is_the_same_columns_the_sink_reads` — a porta única: o que o membrane publica e o que o sink lê são as MESMAS colunas (paridade), num objeto real (sprite: sem bake; vetor: com bake). Mutação: o membrane publicar `uv_rect` e o sink ler outra coluna → divergência → RED.
4. `an_object_rename_re_fingerprints_the_cook` — mudar o Name (⇒ novo `stable_name_id`) desacopla; mudar o CONTEÚDO do objeto (novo content-rev) re-cozinha (o `rev` do External sobe). Fixture: o objeto muda entre dois cooks.
5. Bake por-mídia (vetor/Flip/grupo): a tile assada tem a silhueta do objeto (oráculo de APARÊNCIA — densidade/bbox, não bytes; o bake é GPU). Cada um é `#[ignore]` (precisa de adapter), rodado no fechamento.

### Feature B
6. `the_preview_frame_is_one_rect_paint_and_hit_agree` — `preview_frame_rect` é a porta única; paint e hit lêem a mesma. Mutação: hit computar um rect próprio → clique na moldura erra → RED.
7. `the_header_toggle_flips_the_preview_position` — **seam que CLICA o botão** (não `WidgetEvent` sintético — a checagem de focabilidade); Above→Below→Above. Mutação: dropar o braço `PreviewToggle` no dispatch → clique no-op → posição nunca muda → RED.
8. `the_card_no_longer_reserves_in_card_preview_height` — com o preview externo, `card_h` de um nó com preview == `card_h` sem preview (a faixa morta sumiu). Mutação: manter o termo `PREVIEW_H` → RED.

---

## §7 — As cenas de smoke (com o que MEDIR antes de escrever a mensagem)

Cada wave roda a sonda headless ANTES da mensagem de smoke e escreve o número medido (política do plano de física/Painter). Baselines a medir:

- **A1 (sprite + Duplicator)**: `env PH2D_MOTION_OBJ_SMOKE=1 …` monta um sprite + `motion.grid(N) → motion.duplicator → motion.output`. **Medir**: o custo do cook+lower de N instâncias de tile (deve ser plano no nº de tiles, ~o custo do sprite instanciado hoje — comparar contra o baseline `motion.grid → output` puro). Julgar: um sprite carimbado numa grade, cada cópia com a arte do sprite (não um quad chapado).
- **A2 (vetor, bake-to-tile)**: **medir o bake** (1× por content-rev) vs **N quads**; comparar T2 contra o custo de N re-desenhos Vello (para provar o tier). Julgar: uma estrela vetorial carimbada em 100 pontos, nítida na resolução assada.
- **A3 (Flip)**: idem, medir o bake do walk-pass 1× vs N. Julgar: um desenho de Flip carimbado.
- **A4 (grupo / multi-objeto)**: `source.selection`/grupo → medir o bake do composite do subtree. Julgar: um grupo (sprite+vetor) carimbado como uma peça.
- **B (preview)**: `env PH2D_MOTION_PREVIEW_SMOKE=1` — abrir um grafo com nós que têm preview; clicar o botão do header e ver a moldura pular acima↔abaixo; **medir**: nenhum custo novo (a moldura é o mesmo scatter, só reposicionado).

⚠️ O plano NÃO traz números medidos ainda — as sondas não existem. Cada wave os produz. Aqui está **o que** medir e **por que** (o tier de A depende do número bake-vs-analítico).

---

## §8 — Waves (o ordenamento recomendado)

- **Wave B (rápida, independente, primeiro smoke):** preview → moldura própria + toggle no header. Zero schema, zero contrato, gate 6-8. **Fecha numa sessão** — bom smoke rápido para destravar a percepção.
- **Wave A1 (a fatia vertical de A — o alvo irrefutável) — CONSTRUÍDA (2026-08-02, aguarda smoke):** a coluna `texture_id` (`lower.rs`, fallback 0 = byte-idêntico) + o nó **`source.object`** (crate `ph2d-node-source-object`; genérico, o primitivo do §9.3 — a membrana decide sprite/vetor/flip, o nó não) + o nó **`motion.duplicator`** (crate `ph2d-node-motion-duplicator`; Shape × Points) + o membrane para SPRITE (`shells/desktop/.../motion_bridge_objects.rs`, resolve o tile pelo MESMO `region_uv` do sprite renderer, na fase do motion — o `renderer.atlas()` já está em mãos ali) + o `lower.rs` lê `texture_id`. Prova a arquitetura inteira ponta-a-ponta (sprite → duplicator → render) com o tier barato. **Gates:** `the_lowering_reads_the_texture_id_column` + regressão · 7 no duplicator (stamp/soma-P-rot/Index-Count/passthrough/budget/eval-input-0-1) · 3 no source.object (external/decouple/vazio) · `the_membrane_publishes_exactly_the_columns_the_sink_reads` (gate 3). **Medido (`duplicate()`, doc 28 §5.49 median):** ~4 ns/instância no regime central (256–4096), ~10 ns nos extremos (dominado por alloc) — **plano em N**, sem cliff; um smoke de 16 carimbos custa ~0,0002 ms. **Smoke: `PH2D_MOTION_OBJ_SMOKE=1`** (sprite `Object` carimbado numa grade 4×4). ⚠️ **Escopo A1:** só SPRITE (Atlas/Individual; Cooked KTX2 é pulado — precisa do cooked store do renderer). A fonte emite a APARÊNCIA na ORIGEM (template) — por isso a membrana precisa só de `Sprite`+`Name`, nunca `GlobalTransform`. O picker `ParamWidget::Source` lista curvas E objetos (mesmo canal External) — escolher uma curva num `source.object` dá um polyline (P só), inofensivo; separar os pickers por tipo é polimento futuro.
- **Wave A2 — CONSTRUÍDA (2026-08-02, aguarda smoke):** bake-to-tile de **vetor**. ⚠️ **O nó `source.object` NÃO muda** (é media-agnóstico — o §9.3 provado): só a membrana ganha o ramo de vetor. Uma forma vetorial NOMEADA é **rasterizada UMA vez numa tile** (`motion_object_bake.rs::ObjectBake`) — o idiom do FX raster (`fx_live`) verbatim: `path_screen_bounds → draw_path_isolated → um scratch VelloPass → render_and_readback → acquire_individual`, registrada no `IndividualTextureStore` **sem mutar Sprite nenhum**, e o `texture_id` viaja no stream como o de um sprite. ⚠️ **Câmera FIXA por DPI** (`Affine::scale(256)`, não a viva) ⇒ a tile é **camera-independente** (zoom não re-assa). ⚠️ **Cacheada por CONTEÚDO** (o padrão `FxKey`: path autorado LOCAL + parte LINEAR do xform + dpi, por igualdade) ⇒ cena estática assa UMA vez (steady-state grátis); todo `acquire` casa com um `release` (sem vazar VRAM). ⚠️ **Roda na fase do fx** (`motion_bridge::bake_objects`, ao lado do `fx_live.recook` — os handles GPU já em mãos), e a membrana publica a tile no frame seguinte (1 frame de lag, o precedente do readout). **Tier LIVE por default** (re-assa quando o conteúdo muda; editar a forma re-carimba). **Gates:** `moving_the_shape_does_not_rebake_but_rotating_and_editing_do` (a chave de cache é translation-invariante — um MOVE não re-assa, um ROTATE/EDIT sim). O bake GPU em si é julgado pelo **smoke** (render-and-look, o oráculo de APARÊNCIA do gate 5). **Smoke: `PH2D_MOTION_OBJ_SMOKE=2`** (uma estrela FILLED assada e carimbada numa grade 4×4). ⚠️ **Escopo A2:** a tile é o bbox da forma (o stamp centra); a forma é assada como APARECE (com a rotação/escala da entidade, se houver — o `Transform` linear entra na chave). O toggle FREEZE por-nó fica para depois. Cores: os mesmos bytes crus que o copy GPU→GPU do FX moveria (mesmo comportamento). A membrana assa TODAS as formas nomeadas (popula o picker) — VRAM por tile é o preço (cacheado); assar só as referenciadas é otimização futura.
- **Wave A3 — CONSTRUÍDA (2026-08-02, aguarda smoke):** bake-to-tile de **Flip**. ⚠️ **O nó `source.object` NÃO muda** (media-agnóstico, 3ª vez): só a membrana ganha o ramo de Flip. ⚠️ **A3 NÃO foi "idem A2":** o caminho de render de Flip é **100% GPU→GPU** (rasteriza → resolve → compositor 22-modos → blit) — **sem `path_screen_bounds`, sem readback de produção** (o único, `WalkPass::run`, é harness de paridade sem fills/blend). A2 reusou 3 primitivas prontas do `ph2d-vec-render`; A3 **construiu** as três: um bounds-no-frame (honra a meia-espessura + a pose), um readback offscreen (`copy_texture_to_buffer`+`map_async`), e o **drive do compositor inteiro** (`shells/desktop/src/motion_flip_bake.rs::FlipObjectBake`). ⚠️ **Um objeto Flip é uma PILHA de camadas**, não um desenho ⇒ a tile é o objeto **COMPOSTO no frame atual** (todas as camadas visíveis, blend/opacity por-camada), consistente com A1 (sprite inteiro) e A2 (forma inteira). ⚠️ **Um motor, um estado (a espinha):** o bake usa **scratch renderers próprios** (`FlipRenderer`/`FlipCompose`/`LayerCompositor`, como o scratch `VelloPass` de A2) reusando o MESMO `stage_layer → inject_slice_from_texture → composite` do frame pass, e a **MESMA câmera** (`camera_raw`/`fold_model`, agora `pub(crate)`) ⇒ a tile é byte-a-byte o que o Flip desenhado direto seria (Y e régua de espessura idênticos; uma matriz à mão seria 2ª porta). O trail engine é armado do MESMO `new_engine_armed` (`PH2D_FLIP_NEW_ENGINE=0` não faz a tile discordar da tela). ⚠️ **Câmera FIXA por DPI** (o `BAKE_DPI=256` COMPARTILHADO com A2 — uma tile Flip e uma vetorial de mesmo tamanho têm a mesma resolução; `pub(crate)`, uma porta) ⇒ zoom não re-assa. ⚠️ **Cacheada por CONTEÚDO do frame RESOLVIDO** (hash de: geometria de cada camada visível + pose + blend/opacity/depth + a parte LINEAR do xform do objeto + dpi) ⇒ **um HOLD estático assa UMA vez** (o frame cru NÃO entra no hash); frame que troca o desenho ou move uma pose, edição, ou rotate/scale re-assam; a **translação do objeto é excluída** (a tile é bbox-normalizada — arrastar o objeto nunca re-assa, a regra de A2). **Roda na fase do fx** (`motion_bridge::bake_flip_objects`, ao lado do `bake_objects` — `flip` já destruturado, `flip_entities`/`playhead` são campos disjuntos de `self`); a membrana publica a tile no frame seguinte (1 frame de lag). **Tier LIVE por default.** **Gates:** `moving_the_object_does_not_rebake_but_rotating_and_editing_do` (CPU: move=hit · rotate/edit=miss · **MEDIDO: hold estático sobre 25 frames → 1 chave = assa 1×, não por-frame**) + **gate-5-flip** `a_baked_flip_object_carries_the_composed_two_layer_silhouette` (`#[ignore]`, roda no adapter — oráculo de APARÊNCIA: dirige o bake REAL + readback e afirma cobertura + as DUAS camadas presentes; **MEDIDO na RTX: tile 318×216 px, cobertura 100%, BG-azul 55708 px + FG-laranja 12948 px, bake WARM 2,50 ms** — o 1º bake ~1476 ms é init/compile de GPU, o cold path que o §0 exclui) + o `the_membrane_publishes_exactly_the_columns_the_sink_reads` cobre a publicação (mesma `appearance_tile`). **Smoke: `PH2D_MOTION_OBJ_SMOKE=3`** (objeto Flip 'Object' de 2 camadas composto e carimbado numa grade 4×4). ⚠️ **Zero contrato congelado, zero schema, zero crate/nó novo** (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8` intactos). ⚠️ **LOC:** `motion_flip_bake.rs` nasceu 648>600 ⇒ split dos gates para `motion_flip_bake_tests.rs` (FILHO por `#[path]`); e `motion_bridge.rs` estava **vermelho-latente em 636** (a A2 o empurrou; o gate de `tests/` só corre na varredura impactada — a família documentada) ⇒ os 3 wrappers de bake (`publish_objects`/`bake_objects`/`bake_flip_objects`) migraram para a membrana (`motion_bridge_objects.rs`) re-exportados, motion_bridge 654→595. ⚠️ **Escopo A3:** sem paralaxe multiplano (a tile é template, sem pan de câmera). A membrana assa TODOS os objetos Flip nomeados (popula o picker); assar só os referenciados é otimização futura.
- **Wave A4 — CONSTRUÍDA (2026-08-02, aguarda smoke):** o GRUPO como fonte multi-objeto. ⚠️ **A decisão de PRODUTO (Enio: "VIVO"; e para o escopo, "decida — capacidades plenas, padrão-ouro"):** um `source.object` apontando um GRUPO (`GroupedChildren`) emite os **filhos como N instâncias VIVAS**, cada uma no seu transform relativo ao grupo, cada uma resolvendo seu PRÓPRIO tile a cada frame ⇒ um grupo de mídia MISTA (sprite+vetor+flip), animado, físico, é carimbado com o **layout vivo inteiro** em lockstep (o `instanceSource` do Cavalry). ⚠️ **NÃO é o "composite num tile só" que o §2 esboçou:** compor o subtree num tile CONGELA o grupo (animar exigiria re-bake canvas-inteiro por frame) e exigiria um compositor mixed-media offscreen que não existe — a versão VIVA é a que honra a LIVENESS que o próprio tier (§2) prioriza, e é **mais barata** (reusa os tiles de A1/A2/A3, zero renderer novo). ⚠️ **O nó `source.object` de novo INTACTO** (media-E-COUNT-agnóstico agora): a membrana decide single/grupo, o nó lê o Stream (1 ou N instâncias). **A membrana** (`motion_bridge_objects.rs`): um GRUPO nomeado → `walk_group_transforms` (DFS compondo `Transform::compose` do grupo pra baixo, **a pose do grupo EXCLUÍDA** — o layout é relativo, template) → `resolve_leaf` por filho (**Sprite direto**, orientação livre + rot/scale do filho aplicados; **vetor/flip pelo tile A2/A3 por NOME**) → `group_stream` (N linhas: `P`/`size`/`rot`(graus)/`tint`/`uv_rect`/`texture_id`, as MESMAS colunas do sink). **Recursivo** (grupo de grupos). ⚠️ **Escopo (a decisão do §9.3):** o GRUPO é o multi-objeto de A4 — **seleção-viva e por-tag são source nodes SEPARADOS** de waves futuras (escopos de conveniência; o grupo já entrega "carimbar um objeto composto", então nenhuma capacidade falta). **Gates:** `a_group_lays_its_children_out_relative_to_the_group` (CPU: a pose do grupo é excluída, filho no lugar local, neto composto 5+1=6, **recursão**) + `the_group_stream_lowers_to_one_instance_per_child` (o Stream de N → N `RenderInstance`, two-doors do grupo) + o `the_membrane_publishes_exactly_the_columns` cobre a coluna. **Smoke: `PH2D_MOTION_OBJ_SMOKE=4`** (grupo 'Object' = sprite + estrela vetor + objeto Flip, MÍDIA MISTA, 3 filhos lado a lado, carimbados 4×4 = **48 instâncias vivas**; o Flip anima independente). ⚠️ **Zero contrato congelado, zero schema, zero crate/nó novo.** ⚠️ **v1 limites, NOMEADOS não silenciosos:** filho vetor/flip SEM nome é pulado (sprite não precisa de nome); e o tile do filho vetor/flip carrega a orientação em que foi assado (a pose de MUNDO dele) ⇒ o layout é exato para um grupo axis-aligned, e um GRUPO rotacionado/escalado re-orientando seus filhos vetor/flip é follow-up (o sprite já rotaciona pleno). Selection/tag + grupo-rotacionado + filhos-sem-nome = follow-ups escritos, não gaps.
- **Wave A5 (opcional, dovetail B+A):** o preview mostra o **thumbnail assado real** (uma mini-render) em vez do scatter de pontos — de graça depois que o bake existe.

**Isolamento / linha:** A é cross-cutting (toca `ph2d-eval-motion::lower` + um membrane no shell + o render), então é candidata a **linha própria** (não enxertada na fan-out de nós). B é panel-local e pode ir junto ou antes. Decisão de operação no §9.

---

## §9 — As decisões que são do Enio (recomendação em negrito)

1. ~~Tier de mídia analítica~~ — **DECIDIDO (Enio, 2026-08-02): toggle `Bake to Tile` por-nó, default LIVE** (re-bake por content-rev seguindo animação/física, lockstep) · toggle ON congela para perf. Analítico-por-cópia fica como 3º modo futuro. Ver §2 "O tier". Distinção registrada: lockstep (Duplicator) ≠ N corpos de física independentes (spawn de entidades, feature à parte).
2. **Posição default do preview e persistência:** **Recomendo default Below, runtime-only** (view-preference, como a seleção). Persistir (sobreviver a save) é possível via text-param — só peça.
3. ~~Fonte multi-objeto~~ — **DECIDIDO (Enio, 2026-08-02: "decida — capacidades plenas, padrão-ouro"): o GRUPO é o multi-objeto de A4** (`source.object` num grupo → filhos como N instâncias vivas). Serve "objetos compostos com vários tipos, animações, físicos" — a capacidade que o Enio nomeou. **Seleção-viva** (`source.selection`, lê a seleção AGORA — precisa desenhar QUANDO ela é lida, pois o grafo cozinha por frame) e **por-tag** (precisa de um canal de tag; hoje só há `Name`) são **source nodes separados** de waves futuras: escopos de conveniência, não capacidade que o grupo já não entregue.
4. **Renomear o `motion.output` para "Render"?** — cosmético (a referência chama Render). **Recomendo manter o tipo `motion.output`** e no máximo o display-name "Render". Sua chamada.
5. **É linha própria?** A (cross-cutting: lowering + membrane + render) **recomendo linha dedicada**; B pode ir na mesma ou antes como fatia rápida.
6. ~~Ordem~~ — **DECIDIDO (Enio, 2026-08-02): B (preview) primeiro, depois A1**, depois A2→A4. **Wave B FECHADA (smoke OK).** **A1 FECHADA (smoke OK).** **A2 FECHADA (smoke OK).** **A3 FECHADA (smoke OK).** **Wave A4 CONSTRUÍDA (Enio: "SIga" + "VIVO"; aguarda smoke)** — o GRUPO como fonte multi-objeto, ver §8. **O núcleo de A (os 4 meios + o grupo composto) está construído.** Follow-ups nomeados: A5 (preview = thumbnail assado real) · source.selection/tag · grupo rotacionado + filhos-sem-nome · FREEZE por-nó.

---

## §10 — Fila de bugs pós-smoke

- **B1 — o preview do nó Spawn (`motion.emitter`) PISCA; a moldura não fica estável** (Enio, 2026-08-02, no smoke aprovado da Wave B): a moldura **e** o botão do header aparecem e somem repetidamente no card do emitter.
  **Mecanismo (lido no fonte, não hipótese):** `node.preview` vem de `preview_points` (`shells/desktop/src/render_loop/motion_bridge_readout.rs:179`), que devolve **`None` quando o stream não tem pontos** (`p.is_empty()`) — ou quando o tap de GPU não inclui o nó naquele frame (`(None, None)` em `stamp`). O emitter é **stateless** (o conjunto vivo é função pura do playhead), então em alguns ticks a contagem viva **cruza zero** ⇒ `node.preview` alterna Some↔None frame a frame. A Wave B amarrou a **EXISTÊNCIA** da moldura E do botão a `n.preview.is_some()` (`preview_frame_rect`/`preview_toggle_rect` devolvem `None` sem stamp), então o frame inteiro pisca. Antes da Wave B o mesmo toggle acontecia numa faixa **DENTRO** do card (que crescia/encolhia) — menos gritante, mesmo defeito.
  **Cura recomendada (uma porta):** separar *a moldura EXISTE* (propriedade **estável** do nó — "este nó tem slot de preview") de *o preview TEM pontos agora* (o conteúdo, que pode estar vazio). Frame + botão desenham enquanto o nó é preview-capaz; conteúdo vazio pinta um frame vazio, não some. A presença estável pode ser um predicado por-nó no snapshot (ex.: `has_preview_slot`), decidido pelo tipo/domínio do nó, ao lado de `preview: Option<Vec<[f32;2]>>` (o conteúdo vivo).
  ⚠️ **Decisão de produto pendente:** *frame vazio* × *segurar o último conteúdo não-vazio* (memória de display). **Recomendo o frame estável vazio** — sem memória de display, mesma classe de `selected` (view-state panel-local).

---

## Apêndice — mapa de arquivos (onde cada coisa encosta)

**Feature A**
- Coluna `texture_id` no lowering: `crates/ph2d-eval-motion/src/lower.rs:52-88` (lê `uv_rect` hoje; crava `texture_id: 0`).
- Convenção de colunas + identidades: `crates/ph2d-nodegraph/src/attr.rs` (`SIZE_IDENTITY`, `VALUE_COLUMN`), `column.rs`.
- Nós novos (drop-crates): `crates/ph2d-node-source-*`, `crates/ph2d-node-motion-duplicator` — padrão de `ph2d-node-motion-path` (usa `ctx.external`) + `ph2d-node-motion-clone` (2ª input via `ctx.input(1)`).
- Membrane (shell): `shells/desktop/src/render_loop/motion_bridge_shapes.rs` já publica External para `motion.path` (`set_external(name, Stream)`); irmão novo `motion_bridge_objects.rs` para as fontes de objeto + o bake.
- Referência durável: `ph2d_ecs::stable_name_id` (`crates/ph2d-ecs/src/name.rs:80`); resolução name→Entity: padrão `timeline::persist::upkeep`.
- Objetos: `Sprite`/`SpriteSource` (`ph2d-render/src/sprite/component.rs`), `VecPathRef` (`ph2d-ecs/src/vec_path_ref.rs`) + `ph2d-vec-render`, `FlipObjectRef` (`ph2d-ecs/src/flip_object_ref.rs`) + `ph2d-flip-render`, grupo = `Children` + `GroupedChildren`.
- Sink + draw: `ph2d-node-motion-output`, `lower_to_instances`, `shells/desktop/src/render_loop/present.rs`.

**Feature B**
- Geometria: `crates/ph2d-panel-motion-graph/src/geom.rs:124-144` (`card_h`, `preview_rect` → nova `preview_frame_rect`).
- Draw: `paint.rs:364-476` (`draw_card`), `paint_stamp.rs:21-74` (`draw_preview`).
- Botão do header: modelo em `paint_chrome.rs:131-184`; hit via `GraphHitKind::PreviewToggle { node }` (`editor-core/src/interaction/types.rs:138`); dispatch em `interact.rs:158` (`apply_gesture`).
- Estado por-nó: `MotionGraphPanelState` (`state.rs:313-376`).

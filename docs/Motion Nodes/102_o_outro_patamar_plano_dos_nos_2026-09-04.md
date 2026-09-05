# 102 — O OUTRO PATAMAR: o plano dos nós além de Blender, Cavalry e Houdini

> **Ordem do Enio, 2026-09-04:** *«Precisamos chegar além de tudo que existe. Então se esforce
> em pesquisa e depois em profundo planejamento. No plano detalhes técnicos de modo que os
> agentes não venham a esquecer nada. Outra coisa: vamos redesenhar nossos nós com a bela
> aparência dos nós do mini cavalry ou do blender. decida o que é melhor. Me traga um estudo e
> um plano imbatíveis.»* — e, no meio: *«lembre-se: somos uma game engine. precisamos de
> performance.»*

⚠️ **Método.** Este doc é o **plano**; a pesquisa que o sustenta está nos irmãos —
[doc 100](100_estudo_dos_outputs_2026-09-04.md) (onde vive o que um nó calcula), [doc 101](101_pesquisa_cartoes_ricos_2026-09-04.md)
(o cartão rico e a régua do tablet), [doc 99](99_estudo_do_mini_cavalry_2026-09-02.md) (o sistema
visual) e [doc 63](63_pesquisa_industria_2026_e_plano_estado_da_arte.md) (a pesquisa de indústria
de julho, cuja §4 já desenhou a *anatomia-alvo do nó* e cujas waves W-I/W-J este plano **absorve e
completa**, não repete). Toda afirmação sobre a casa foi lida no código em 2026-09-04; toda
afirmação sobre uma referência tem fonte (§9). ⛔ **Nada aqui é código ainda.**

⚠️ **Como um agente deve LER este doc:** a §0 é o índice de PORTAS (cada pergunta tem UM sítio
que responde); a §2 é a lei de performance que toda wave obedece; a §4 é o plano, wave a wave,
com os ficheiros, os tipos, os registos de formato, os gates (red-first, com a mutação que os
prova) e o smoke. **Antes de tocar numa wave, releia a §0, a §2 e a §7 (recusas).**

---

## §0 — O índice das PORTAS (uma pergunta, um sítio)

| pergunta | a porta (hoje) | ficheiro |
|---|---|---|
| que valor um nó lê para um param? | `EvalCtx::param(name)` = **fio `d` > override > default** (panic em nome não declarado) | `crates/ph2d-nodegraph/src/cook_eval_ctx.rs:191` |
| quem dirige um param? | `Graph::param_sources(node) -> Option<&Sources>` · `drive_param` / `undrive_param` | `crates/ph2d-nodegraph/src/graph.rs:479`, `param_source.rs` |
| como um fio chega a um param na UI? | `GraphIntent::DriveParam{from_node,from_port,to_node,param}` → shell `motion_bridge_intents.rs:99` → `Graph::drive_param` | `crates/ph2d-panel-motion-graph/src/snapshot_intent.rs` |
| onde o cook resolve um param dirigido? | `Cook::cook_node` passo **1b** (`driven_value` = **elemento 0** do `v`) | `crates/ph2d-nodegraph/src/cook.rs:467` |
| o que o formato grava? | `n e p x d t y` acima de `[layout]`, `l` abaixo; header `v1..v5`, **bump só se o registo existir** | `crates/ph2d-nodegraph/src/format.rs` |
| o que o device consegue correr? | `plan()` → `GpuPlan{stages: Vec<GpuStage{node,..}>, boundaries}`; recusa: **porta ≠ 0 ligada**, **param dirigido**, kernel ausente, `pre` fora do loop | `crates/ph2d-gpu-cook/src/plan.rs:295-330` |
| o que um kernel É? | `GpuKernel{wgsl, wgsl_lib, bindings:[ColumnBinding{column,dim,access,identity,port}], params, count_law, variant_by_param, applicable}` + `StreamOp::{Compact,SourceRows,Concat,Project}` registados à parte | `crates/ph2d-nodegraph/src/gpu.rs:179`, `column.rs:14`, `stream_op_meta.rs:33` |
| como o corpo de um kernel lê/escreve? | acessores gerados `read_<col>(i)` / `write_<col>(i, v)` / `params.<name>` (`codegen::accessor_suffix`) | `crates/ph2d-gpu-cook/src/codegen.rs:41` |
| por que uma cena caiu para a CPU? | `fell(motion, reason)` — **9 razões nomeadas**, `PH2D_MOTION_ROUTE_LOG=1` | `shells/desktop/src/render_loop/motion_bridge_gpu.rs:66,216-338` |
| o que o cartão sabe? | `GraphNodeView{id,kind,display_name,category,silhouette,x,y,inputs,outputs,readout,count,hot,is_sink,preview,…}` construído em `motion_bridge_fold.rs` | `crates/ph2d-panel-motion-graph/src/snapshot.rs:129` |
| o que se pode tocar no cartão? | `GraphHitKind::{Background,Node,SocketIn,SocketOut,Wire,Waypoint,Backdrop,BackdropResize,PreviewToggle,InertBadge,SplitDivider}` | `crates/ph2d-panel-motion-graph/src/state.rs:120` |
| como um param é editado? | `MotionParamIntent::{SetParam{node,param,value}, SetTextParam{..}}` → `push_param_intent` → shell → `Graph::set_param` → undo por diff | `crates/ph2d-panel-motion-params/src/snapshot.rs:491` |
| o que uma row de param É? | `ParamRow::{Scalar,Color,Toggle,Enum,Angle,Seed,Text,Curve,Gradient,Palette,Channels,Source,File}` de `ParamUiHint{param,label,min,max,step,widget,unit}` + `ParamGroup` + `ParamGate*` | `crates/ph2d-panel-motion-params/src/snapshot.rs:22`, `ph2d-node-registry/src/ui.rs` |
| quem desenha as rows? | `rows_paint::paint_rows(rows, …, store, hit_index, scene, text_system, theme, modified, section_at)` (2 977 LOC) | `crates/ph2d-panel-motion-params/src/rows_paint*.rs` |
| o que um fio solto no vazio faz? | `interact_drop::resolve_loose_output_drop` → cartão dobrado / corpo do nó / **smart-connect** (menu filtrado por `menu_catalog(snap, from)`) | `crates/ph2d-panel-motion-graph/src/interact_drop.rs`, `snapshot_menu.rs:94` |
| o que uma fórmula sabe fazer? | `Expr::{Const,Attr,Param,Unary,Binary,Call,Select}` · `eval(&Expr,&dyn Bindings)` · `eval_column(expr, stream, params)` · **`to_wgsl(&Expr)` existe e ninguém o chama** | `crates/ph2d-expr/src/{expr,eval,stream,wgsl}.rs`, `ph2d-expr-parse` |
| o que a timeline anima? | `TargetBinding{target: AnimTarget(u64 OPACO), prop: PropKind(enum fechado 0..5), wire_id}`; o `AnimTarget` já diz *«a node param»* como uso previsto | `crates/ph2d-timeline/src/binding.rs`, `prop.rs`, `ph2d-anim/src/clip.rs:20` |
| como uma pré-visualização escreve sem sujar o undo? | `PreviewDrive{driven, settle, substitute_authored, restore_live}` keyed `(Entity, Driver)` | `shells/desktop/src/preview_drive.rs:270` |
| que alça de canvas existe? | `field_gizmo.rs` — **escrito à mão** para `field.box` e `field.radial_sweep` | `shells/desktop/src/field_gizmo.rs:209` |
| onde vive a side-metadata de um nó? | os 24 `NodeRegistry::register_*` (ui · param_ui · units · groups · gates · gpu_kernel · stream_op · …) — **nunca o `NodeManifest`** | `crates/ph2d-node-registry/src/lib.rs:185-640` |
| os tokens visuais do grafo? | `node-cat-*` (7) · `port-*` (8, com `port-value`) · `graph-*` (13) nos 3 temas | `docs/design/tokens.json`, `crates/ph2d-tokens` |
| a spec visual de que o Mini Cavalry copiou? | [plano 01 §M1.E1 + «Card / 7 silhuetas / Sockets / Fios / Activity-fire»](01_plano_modulo_motion_nodes.md) (linhas 280-296) | `docs/Motion Nodes/01_plano_modulo_motion_nodes.md` |

---

## §1 — A DECISÃO DO VISUAL: a nossa spec (que o Mini Cavalry implementou), com o comportamento do Blender

**Decisão: o look do Mini Cavalry — porque ele É a nossa spec.** O `visual-tokens.js` dele abre com
*«Doc PH2D §6»*: as 7 silhuetas por papel, a cor de cabeçalho por categoria, a cardinalidade do
pino e a espessura do fio saíram do [plano 01](01_plano_modulo_motion_nodes.md) e foram
implementadas por ele **antes** de nós. Escolher o Blender seria trocar a nossa identidade por
uma que existe para rato e monitor; escolher o Mini Cavalry é terminar de implementar o que
declarámos. **Do Blender vêm três COMPORTAMENTOS, não o look:** o widget inline que some quando o
pino é ligado, a **forma do socket = estrutura do dado** (5.0), e o LOD por zoom.

### 1.1 O que muda no cartão (spec executável — `geom.rs` / `paint.rs` / tokens)

| peça | hoje | alvo | de onde |
|---|---|---|---|
| largura | `CARD_W = 190` | **220** lógicos (o `220px lógico × zoom` do plano 01; o Mini Cavalry tem 220) — medir a elisão do título nos 134 nomes | plano 01 |
| cabeçalho | 26 px, cor plana `cat_token` | **gradiente vertical** `mix(cat_token, Bg2, 0.55) → Bg2` (plano 01), 8×12 de padding, título 13 px/600, **ícone da categoria** à esquerda, **selo de papel** (já) à direita, o **param de modo** como subtítulo (`Random ▾`) | plano 01 · Mini Cavalry `.node-header` |
| corpo | `Bg2` | `Bg2` + borda 1 px + sombra (token `graph-shadow` — **NOVO token**, medir em 3 temas) | Mini Cavalry `.node` |
| silhueta | 7 papéis, selo no cabeçalho | 7 papéis **no contorno** (`RoundedRectRadii` 4 cantos: rect[8] · cigar[22] · circle[16] · diamond · trapezoidDown[18,18,4,4] · trapezoidUp · tabbed) — o corpo do cartão define hit-rects, então `geom` e `paint` leem a MESMA função | plano 01 |
| socket | ○ escalar / ◇ vector, cor por espécie (3) | **forma = estrutura** (○ valor único · ◇ corrente/campo · **◆· param dirigível**) + cor por espécie; **raio de hit 22 px no toque** (44 pt, HIG) e 9 no rato | Blender 5.0 · Apple HIG |
| rótulos | só com ≥ 2 saídas | idem + **rows de param** (doc 101 D1) com widget inline no cartão QUENTE | Blender |
| readout | 1 linha | mantém; + **custo/rota** (`⚡ 0,12 ms` / `CPU 3,4 ms`) quando o overlay de custo estiver ligado (W6) | Blender *Timings* · doc 63 §4.1 |
| moldura | acima/abaixo, só posicionais | mantém + **política de custo**: seleccionado a cada quadro, os outros a 1/5 (Mini Cavalry) ou *static frame* (Nuke) — **medir em W0** | Nuke · Mini Cavalry |
| fios | cor = domínio | cor = **espécie** (pede `PortType` no `GraphEdgeView`, doc 99 §aberto), espessura Event 1,4 / resto 2,6 × zoom, **tracejado = `pre`**, *activity-fire* (plano 01) | plano 01 |
| chips de proveniência | — | por row: `[·]` constante · `[≈]` expressão · `[◆]` fio · `[◇]` keyframe (doc 63 D7, os 4 modos do TouchDesigner) | doc 63 |

⛔ **O que NÃO se copia:** a densidade do Blender (sockets de 5 px, rows para rato); os 134
`renderUI` à mão do Mini Cavalry (o nosso painel é derivado de `ParamUiHint` e o cartão será o
2.º host do MESMO renderer); as 7 cores por tipo dele (o nosso Motion tem 3 espécies — pintar 7
seria inventar distinções que o catálogo não tem, doc 99 §2).

### 1.2 A régua do tablet, que manda em todo pixel permanente
Alvos **1366×1024 · 1194×834 · 1133×744**; toque ≥ **44 pt**. O cartão **magro** (dirigidos ∪
promovidos) fica ≤ **25 %** da altura do mini para TODO nó do registry (gate); «todos os params
no cartão» está **recusado por medição** (doc 101 §4: 48 % no oscilador, 80 % no `bezier_warp`).

---

## §2 — A ESPINHA: as leis de performance que toda wave obedece

1. **Toda capacidade nomeia o seu caminho no DEVICE antes de nascer.** Hoje **duas** portas do
   grafo derrubam um nó para a CPU sem que o artista saiba: um **param dirigido** (`plan.rs:305`
   — *«a GPU stage has no lane for them (F1.2+)»*) e um **pino ≠ 0 ligado** (`plan.rs:323`). Um
   fio a mais custa **50,9×** ([doc 98](98_auditoria_de_performance_2026-09-01.md): 4,19 M em
   3,85 ms no device contra 195,9 ms na CPU). ⇒ **W1 (as lanes do planeador) vem ANTES de qualquer
   capacidade que crie fios novos** (expressões, pick-whip, keyframes por fio, pinos extra).
2. **O orçamento é por QUADRO e por CENA, medido, não estimado:** 16,67 ms a 60 Hz; o cook do
   device hoje custa **um passe por nó** (a tee: 3 passes; o oscilador: 2). W0 instrumenta
   (CPU por nó, GPU por passe, passes por cena sobre as **109 cenas**) e **W9 funde** (Houdini
   *compiled block*, VFX Graph *context*, Niagara *emitter script*: **uma cadeia por-elemento =
   UM kernel**).
3. **Uma capacidade que hoje derruba para a CPU nasce DESLIGADA por omissão** até a lane existir
   — e o cartão DIZ (W6) quando um gesto do artista tirou a cena do device. *Nada de «ficou mais
   lento» sem nome.*
4. **A UI não paga por nó:** widgets vivos só no cartão quente; LOD por zoom; moldura com
   política de custo; **zero alocação por quadro por cartão frio** (gate: contagem de registos
   no `HitIndex` por quadro = f(quente), nunca f(N)).
5. **Determinismo antes de velocidade:** toda fórmula, kernel fundido e keyframe corre a MESMA
   aritmética `f32` na CPU e no device (HR-5, sem transcendentais); o gate de paridade
   bit-a-bit (`ph2d-gpu-cook/tests/gpu_cpu_parity_*`) cobre cada wave que toca o device.

**Baseline medida (2026-09-01..04):** 109 cenas; **69,7 %** nunca chegam ao device; 4,19 M em
3,85 ms; a tee = o oscilador **ao bit**, 3 passes contra 2; params dirigidos ⇒ CPU; ~29 nós
por-elemento seriais na CPU; `value.lfo` 14,82 ms a 1 M (serial) contra 6,67 do oscilador.

---

## §3 — O MAPA das capacidades: o que é «outro patamar», medido contra quem já tem

| # | capacidade | Blender | Houdini | Cavalry | Mini Cavalry | **PH2D hoje** | fonte |
|---|---|---|---|---|---|---|---|
| 1 | **fórmula em qualquer campo**, citando outros nós | ⛔ (drivers, à parte) | ✅ `ch()`, e o **rename não parte** (guarda o id) | ✅ *Attribute Expressions* (1.3) | ⛔ | motor existe (`ph2d-expr`), só vê o próprio nó; `to_wgsl` sem consumidor | [SideFX](https://www.sidefx.com/docs/houdini/network/rename.html) · [Cavalry 1.3](https://cavalry.studio/docs/tech-info/release-notes/1.3/1-3-0-release-notes/) |
| 2 | **qualquer param é uma SAÍDA** (pick-whip) | ⛔ | via `ch()` | ✅ todo atributo | ⛔ | `d` só para dentro | [Cavalry](https://cavalry.studio/docs/getting-started/key-concepts/connections/) · AE parte no rename ([CreativeCOW](https://creativecow.net/forums/thread/error-when-pick-whipping-properties-across-comps-a/)) |
| 3 | **keyframe em qualquer param**, no cartão | ✅ `I` sobre o socket | ✅ | ✅ | ✅ (por menu) | ⛔ **adiado desde 2026-07-09**; `AnimTarget` opaco já prevê «a node param» | [Blender](https://docs.blender.org/manual/en/latest/animation/keyframes/editing.html) |
| 4 | **soltar fio no vazio → lista compatível** + conversor a 1 clique | ✅ 3.1 | ⛔ | ⛔ | ⛔ | ✅ smart-connect (doc 59/63.3) · ⛔ conversor a 1 clique (a recusa já o NOMEIA) | [Blender 3.1](https://developer.blender.org/docs/release_notes/3.1/nodes_physics/) |
| 5 | **alça no canvas** para params espaciais | ✅ 4.3, **ligada à mão** (Gizmo nodes) | handles por nó | parcial | overlays | 2 nós à mão (`field_gizmo.rs`); **127 params** com unidade espacial e **20 pares x/y** | [Blender 4.3](https://www.blender.org/download/releases/4-3/) |
| 6 | **nó próprio com interface + biblioteca** | ✅ grupos + assets | ✅ HDA (promover = **cópia + expressão**) | ⛔ | grupos | subgrafo = dobra da vista (doc 57), sem interface | [SideFX](https://www.sidefx.com/docs/houdini/assets/asset_ui.html) · [Blender](https://docs.blender.org/manual/en/latest/interface/controls/nodes/groups.html) |
| 7 | **custo e ROTA no cartão** | ✅ *Timings* (3.1), só tempo | Performance Monitor | ⛔ | ⛔ | 9 razões no log; **nada no ecrã**; nenhum relógio por nó | [Blender 3.1](https://projects.blender.org/blender/blender/commit/e4986f92f32) |
| 8 | **cadeia por-elemento = UM kernel** | multi-function lazy (CPU) | VOP→VEX, *compiled block* | ⛔ | ⛔ | **um passe por nó**; 56 kernels são mapas puros (fundíveis), 12 mudam a contagem, 17 têm `StreamOp` | [SideFX VEX](https://www.sidefx.com/docs/houdini/vex/index.html) · [Artivoxa](https://www.artivoxa.com/houdini-invoke-compiled-block-speeding-up-heavy-sop-networks/) · [Unity VFX](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@16.0/manual/Block-CustomHLSL.html) |
| 9 | **cartão rico** (widgets · promoção · painéis · LOD · compacto) | ✅ | ⛔ | ⛔ | ✅ | doc 101 | doc 101 |
| 10 | **spreadsheet · sondas · balão com colunas · escolher instância no canvas** | inspecção + viewer | spreadsheet + Info | ⛔ | ✅ | ⛔ (doc 100 §7) | doc 100 |

⭐ **O «além de tudo» não é uma célula desta tabela: é a LINHA inteira, e três coisas que ninguém
tem:** (a) o fio que chega a um param **fica no device** (W1) — hoje nem o Blender corre o grafo no
device; (b) as alças de canvas **derivadas da tabela de unidades** (W5) — o Blender 4.3 exige
ligá-las à mão; (c) a fusão de kernels **com paridade bit-a-bit provada** contra a CPU (W9) — o
Houdini funde, mas o VEX não tem um oráculo CPU/GPU de bits.

---

## §4 — O PLANO, wave a wave (ordem = performance primeiro)

> Cada wave traz: **objectivo · portas (ficheiro, tipo, assinatura) · formato · cook/planeador ·
> UI · gates red-first (com a MUTAÇÃO que os prova) · smoke · orçamento · custo no tablet ·
> contrato**. Um agente que abre uma wave lê a dela **e a §0/§2/§7**; nada aqui substitui ler o
> ficheiro citado — os números de linha são de 2026-09-04 e envelhecem (a função não).

### W0 — INSTRUMENTOS: o custo e a rota de CADA nó, medidos (pré-requisito de W6 e W9)
**Objectivo.** Saber, por nó e por quadro, *quanto custou* e *onde correu* — hoje o cook não tem
relógio (`grep Instant cook.rs` = 0) e o device não escreve timestamps (`timestamp_writes: None`
em `encode.rs:265`, `grid.rs:240`, `scan.rs:209`).
- **CPU por nó:** `Cook::cook_node` mede `Instant` à volta de `op.eval(&mut ctx)` (bypass = 0) e
  guarda `cost_ns: u64` no `Cached` (ao lado de `revision`); porta de leitura `Cook::cost_of(node)
  -> Option<u64>`, irmã de `peek`. Zero alocação.
- **GPU por passe:** pedir `wgpu::Features::TIMESTAMP_QUERY` **se o adapter o tiver** (onde o
  `Device` é criado — `grep request_device`); um `QuerySet` de `2 × stages` por submit;
  `ComputePassDescriptor::timestamp_writes = Some(..)` nos três sítios; `resolve_query_set` para
  um buffer `QUERY_RESOLVE`, cópia para `MAP_READ`, **leitura no quadro seguinte** (nunca
  `poll(Wait)` no caminho do quadro); `queue.get_timestamp_period()`; resultado em
  `GpuCook::last_stage_ns: Vec<(NodeId, u64)>`. Sem a feature ⇒ `None` (o badge diz *«GPU · sem
  relógio»*, nunca inventa). Fonte: [wgpu timestamp queries](https://wgpu.rs/doc/wgpu_examples/timestamp_queries/index.html).
- **Rota por nó:** `plan()` passa a devolver `refusals: Vec<(NodeId, Refusal)>` (campo novo em
  `GpuPlan`, append) com `enum Refusal { Port1Connected, DrivenParam, NoKernel,
  GeneratorWithoutCountLaw, KernelParamUndeclared, PreOutsideLoop, … }` — **um push por `return
  false`** em `plan.rs:295-330`. É a versão por-nó das 9 razões por-cena do `fell`.
- **A sonda:** `the_109_scenes_cost_passes_and_refusals` (`#[ignore]`, no shell) imprime por cena:
  rota · passes (`dispatching_stages`) · nós na CPU com a `Refusal` · ms CPU · ms GPU. Alimenta o
  doc 98 e é a régua de W1/W9.
- **Gates:** `a_cooked_node_reports_its_cost` (real > 0, bypass = 0 — mutação: apagar o `Instant`)
  · `every_refusal_in_the_planner_names_itself` (textual sobre `plan.rs`: nº de `return false` =
  nº de `Refusal::` — a lei do `every_fall_through_to_the_cpu_names_itself`) ·
  `the_timestamp_readback_never_waits_in_the_frame` (textual: zero `Maintain::Wait` no caminho).
- **Orçamento:** medir o custo do próprio relógio (ns por passe); a sonda corre em `--release`
  com `/proc/loadavg` impresso ao lado (§5.0).
- **Contrato:** nenhum. **Tablet:** nenhum pixel.

### W1 — AS LANES DO PLANEADOR: o fio novo NÃO derruba o nó (pré-requisito de W2/W3/W7/W8)
**Objectivo.** Apagar as duas recusas cegas de `plan.rs` (param dirigido · porta ≠ 0) **sem
mudar o resultado ao bit**.
- **(a) Param dirigido = um UNIFORM copiado no device.** Por estágio, um buffer `drv` (`f32 ×
  nº de params dirigidos daquele nó`, `STORAGE | COPY_DST`): a CPU escreve nele o **fallback**
  (override/default — a lei do `driven_value`: *«an empty driver … FALLING BACK … rather than to
  0.0»*), depois `copy_buffer_to_buffer(src_v, 0, drv, k*4, 4)` para cada fonte com `count > 0`
  (a fonte é um estágio, logo o número **nunca sai do device**). O codegen troca `params.<name>`
  por `drv[k]` no prelúdio (`let p_<name> = drv[k];` + substituição por fronteira de palavra) —
  **gate textual**: zero `params.<name>` sobrevive no WGSL gerado para um nome dirigido. Um
  driver por-elemento lê `[0]` — **o mesmo elemento 0** que `driven_value` lê na CPU. ⛔ Um
  buffer por param estourava o mínimo WebGPU de **8 storage buffers por estágio** (um nó com 24
  params dirigidos): daí UM `drv` por nó.
- **(b) Porta ≠ 0 = o complemento de um `Compact`.** `StreamOp::Compact { port, predicate }` ganha
  `complement: Option<u16>` (append): as linhas que o predicado REMOVE saem na porta `k` como
  segunda corrente. Cobre o `sim.lifetime` (`died`/`pulse` são os mortos); `GpuStage` passa a
  `(node, port)` em `streams: BTreeMap<(NodeId, u16), GpuStream>` e `GpuSource::Stage(NodeId,
  u16)`. A recusa `e.from.1 != 0` fica **só** para nós cujo kernel não declara a porta.
- **Cache de pipelines:** a chave inclui o conjunto de nomes dirigidos (o mecanismo
  `presence_signature` já existe — estende-se).
- **Gates (paridade bit-a-bit via o arnês `gpu_cpu_parity_*`):**
  `a_driven_param_keeps_the_node_on_the_device` (oscilador com `amplitude ← value.lfo` ⇒
  `fully_gpu`; mutação: manter o `return false`) ·
  `the_device_reads_the_same_element_zero_as_the_cpu` (driver de comprimento N) ·
  `an_empty_driver_falls_back_on_the_device_too` · `the_death_event_stays_on_the_device`
  (`sim.lifetime.pulse` ligado ⇒ `fully_gpu`) · `no_params_dot_survives_for_a_driven_name`.
- **Orçamento:** **zero passes** a mais em (a) (cópias de 4 bytes); (b) um `Compact` já paga o
  scan — o complemento é o mesmo scan com o predicado invertido (medir: ≤ 1 passe a mais).
- **Contrato:** `GpuKernel`/`StreamOp` **não são congelados** (side-metadata, `gpu.rs`).

### W2 — FÓRMULAS EM QUALQUER CAMPO + PICK-WHIP (capacidades 1 e 2 — um mecanismo)
**Objectivo.** `amplitude = @n12.rows * 4` num campo; arrastar de um param para outro escreve
essa fórmula; renomear um nó **não parte nada**.
- **Gramática** (`ph2d-expr-parse`, append): `@n<id>.<param>` (o param EFECTIVO do outro nó —
  resolvido pela mesma escada) · `@n<id>.<porta>` (o **elemento 0** do `v` daquela saída — o que
  o `d` lê) · `@self.<param>` · `@t` (playhead, s) · `@frame`. ⛔ `Attr` (coluna por-elemento) é
  **recusado num campo** com a cura: *«a field is one number per node; per element use
  `value.attribute` → `motion.drive`»* — um param é um **uniform** (§2, W1).
- **Identidade = id, nunca o nome** (Houdini guarda *«a hidden pointer to the node's ID»*; o AE
  guarda o nome e parte). O editor aceita um **rótulo** (`Graph::label`) e reescreve para `@n<id>`
  ao confirmar; o display mostra o rótulo quando existe; rótulo ambíguo ⇒ recusa com a lista.
- **Formato:** registo **`f <id> <param> <expr...>`** (texto livre após o 3.º espaço, como `x`),
  acima de `[layout]`, header **`v6` iff existe um `f`**. `Graph`: `param_exprs: BTreeMap<NodeId,
  BTreeMap<String, String>>` + `set_param_expr / clear_param_expr / param_expr`. **Invariante com
  gate: um param tem no máximo UMA fonte** — `set_param_expr` chama `undrive_param`, `drive_param`
  apaga o `f`; a leitura é **uma porta**: `Graph::param_source(node, name) ->
  Option<ParamSource<'_>>` com `ParamSource::Wire(NodeId, u16) | Expr(&str)`.
- **Dependências:** `Graph::expr_refs(node, name) -> Vec<NodeId>` (parse cacheado por hash do
  texto no `Cook`: `BTreeMap<u64, Arc<Expr>>`); `would_cycle` anda pelos `f`; `remove_node` limpa
  os dois lados — uma referência a um nó apagado **mantém o texto**, cai para override/default e
  acende o chip de erro (a lei do `d`: nunca `0.0`).
- **Avaliação (cook, passo 1b′):** cozinhar as referências (revisões em `input_revs`), avaliar
  com `ExprBindings { cook, graph, node, in_key }` (impl de `ph2d_expr::Bindings`: `param` resolve
  `@n.p` pela escada efectiva, `attr` é inalcançável por construção) e **inserir no mesmo
  `driven` que o `d` preenche** — `EvalCtx::param` **não muda** (uma porta).
- **Device:** o escalar avaliado na CPU entra no `drv` de W1 como *fallback* (zero cópias) — um
  campo custa nanossegundos e **não precisa de `to_wgsl`**; `@n.<porta>` usa a lane (a) tal qual.
- **Fingerprint:** `param_source::fingerprint` ganha o hash do texto; as revisões das referências
  já entram por `input_revs`.
- **Determinismo:** a política de `Func` do `ph2d-expr` (o agente lê `expr.rs` e escreve no doc
  do gate quais funções existem); um campo corre na CPU como o `motion.expression` corre hoje.
- **UI:** o campo numérico aceita o prefixo `=` (idioma Excel/Cavalry) →
  `MotionParamIntent::SetParamExpr { node, param, text }` (variante nova, append); chip `[≈]`
  com o texto no balão; erro = a mesma tríade do L-System (a queixa **nasce · chega à row ·
  chega a PIXEL** — 3 gates). No cartão (W4): `GraphHitKind::ParamSocketOut { node, param }` — o
  **pick-whip**: soltar num `ParamSocketIn` escreve `= @n<from>.<param>`; soltar num pino `VALUE`
  oferece **«Number from param»** (insere `value.number` com a fórmula — ⛔ sem nós escondidos).
  Um `f` que referencia outros nós desenha **fios tracejados de VISTA** (derivados dos `refs`,
  não são arestas do documento): `GraphEdgeView` ganha `kind: Wire | Ref` (append).
- **Gates:** `an_expression_round_trips_byte_for_byte` (`v6` só quando existe) ·
  `a_param_has_at_most_one_source` (mutação: saltar o `undrive`) ·
  `an_expression_reference_is_cooked_first` · `an_expression_cycle_is_refused_like_an_edge` ·
  `renaming_a_node_does_not_break_an_expression` · `a_deleted_reference_falls_back_and_says_so` ·
  `a_column_in_a_field_is_refused_with_the_cure` ·
  `the_expression_value_reaches_the_device_as_a_uniform` (paridade) · `the_error_reaches_pixel`.
- **Smoke:** `PH2D_EXPR_FIELD_SMOKE=1` — grid → oscillator com `amplitude = @grid.rows * 4`;
  arrastar `rows` e ver a amplitude seguir; pick-whip de `rows` para `phase_stagger`.
- **Orçamento:** um `f` = um parse cacheado + N `f32` ops por quadro; **zero passes**.
- **Contrato:** `NodeManifest` intocado; `Graph`/`format` não são congelados (v6 pela política).

### W3 — KEYFRAME EM QUALQUER PARAM, NO CARTÃO (capacidade 3)
**Objectivo.** O losango ao lado de cada número; a timeline anima params de nós; o undo e o
ficheiro nunca veem a pré-visualização.
- **Timeline:** `PropKind::Param = 6` (append, `repr` numérico — o bump do `DOC_VERSION` do
  `TimelineDoc` segue a regra dele); `AnimTarget` já é **opaco** e documenta *«a node param»* como
  uso: `motion_param_target(node, param) = AnimTarget::new(fnv64("motion-param\0<id>\0<param>"))`
  no shell (a timeline continua genérica; o `NodeId` é estável no documento).
- **Aplicar por quadro:** o shell lê o valor amostrado de cada binding motion → **`MotionPreview`**
  (irmão de `PreviewDrive`, chave `(NodeId, String)`, guarda o **autorado**) → `graph.set_param`
  antes do cook; `substitute_authored` antes de toda captura (undo/save) e `restore_live` depois —
  a lei do `preview_drive.rs`: *«o documento é o valor AUTORADO»*. Entra na **assinatura** de
  `ProjectState::capture` como o ledger existente (uma função-irmã «com ledger» seria a segunda
  porta pela qual o defeito voltava).
- **Precedência — UMA lei, um gate:** `d`/`f` (exclusivos) **>** keyframe **>** override **>**
  default. Um param dirigido e keyframado mostra o chip `[◆]` e a lane cinzenta — nunca um
  controlo que não faz nada sem o dizer (doc 90).
- **UI:** `ParamRow` ganha `keyed: KeyState { None, Keyed, OnKey }` (derivado do `TimelineDoc`);
  losango → `MotionParamIntent::InsertKey { node, param }` / `RemoveKey`; **auto-record** com a
  lei do Mini Cavalry (Phase 26): *Play desliga o auto-record, Stop restaura, toggle manual
  invalida* — sobre o `MotionTransport` que já existe. Keyframes amostram **em tempo de
  timeline**, nunca no `time` do eval (Phase 25).
- **Unidades:** `PropKind::Param` é sem unidade — o param leva a sua (`ParamUnit`); a divergência
  rad/graus da memória de 2026-07-09 **não se coloca**.
- **Memo:** o ledger escreve overrides ⇒ o `params_fingerprint` invalida o memo sozinho (memória
  de 2026-07-09, achado 4).
- **Gates:** `a_keyframed_param_never_reaches_the_undo_queue` · `…_never_reaches_the_file` ·
  `the_precedence_is_one_law` (tabela de combinações) · `insert_key_from_the_card_lands_in_the_doc`
  · `play_disarms_auto_record_and_stop_restores_it` · `a_bound_motion_param_survives_save_and_load`.
- **Smoke:** `PH2D_MOTION_KEYS_SMOKE=1`. **Orçamento:** amostragem = `O(bindings)`
  (`doc.bindings()` nomeia quem anima). **Contrato:** nenhum; o **W4.T4** (dock da timeline no
  Motion) é pré-requisito de UI e está *«aguardando re-smoke»* (§5 do CLAUDE.md).

### W4 — O CARTÃO 2.0 (doc 101 W1–W6 + doc 63 I1–I8 + a spec da §1)
- **Renderer:** crate-folha **`ph2d-param-rows`** (o modelo `ParamRow…`, `paint_rows`,
  `snapshot_ids`, com `RowHost::{Panel, Card { hot }}`); o painel fica **byte-idêntico** (golden
  de cena do `ph2d-ui-testkit` antes/depois).
- **Formato:** promoção **`q <id> <param>`** (**v7**; ⚠️ `p` já é o override — o doc 101
  escreveu `p` por engano, corrigido); sob `[layout]`: `c <id> <0|1>` (expandido) e `w <id> <px>`
  (largura) — **não-semânticos**, como `l`.
- **Rows no cartão:** `card_rows(n, promoted, expanded)` em `geom.rs` (o hit-test e o pintor
  leem a mesma); conteúdo = dirigidos (`d`∪`f`) ∪ promovidos ∪ (expandido ⇒ todos, em secções
  `ParamGroup`). **Só o cartão quente regista widgets**; os frios pintam glifo+texto — gate:
  `the_card_registers_widgets_only_when_hot` (nº de registos = f(quente), nunca f(N)).
- **Sockets de param:** `GraphHitKind::ParamSocketIn/Out { node, param }` (append); 3.ª forma de
  glifo; soltar um fio de valor no `In` = `GraphIntent::DriveParam` (existe); do `Out` = W2.
- **LOD:** `z₁`/`z₂` medidos em W0 (legibilidade = `TITLE_SIZE × zoom ≥ n px` pelo `TextSystem`);
  abaixo de `z₁` rows nem pintadas nem registadas (gate mede as duas metades).
- **Compacto:** `value.*` com ≤ 1 param e 1 saída = pílula com o glifo.
- **Decoração:** a tabela da §1.1 — largura 220, cabeçalho em gradiente + ícone + modo, silhueta
  no contorno, sombra (token novo), fios por espécie (`GraphEdgeView.ty: PortType`, append),
  chips de proveniência `[·] [≈] [◆] [◇]`.
- **Gestos baratos de doc 63:** *value ladder* (arrasto com modificador em degraus
  0,001…100) · **«Drive by…»** no menu do param (LFO / Noise / Random / Expression — insere e liga
  pela porta da paleta + `DriveParam`) · pips **bypass** (existe, `y`) e **viewer** (liga a moldura
  daquele nó); ⏸️ *solo* fica nomeado, sem mecanismo de sink temporário definido.
- **Gates:** os 8 do doc 101 §8 + `the_panel_paint_is_byte_identical_after_the_extraction` +
  `every_card_of_the_registry_fits_the_tablet_when_lean` (≤ 25 % do mini) +
  `every_widget_file_wires_a11y` (linha nova).
- **Smoke:** `PH2D_CARD_SMOKE=1` (a cena do `PH2D_DRIVEN_ROW_SMOKE=1` — já tem dois params
  dirigidos e um por promover). **Orçamento:** paint de 20 cartões ricos medido em W0; **zero
  alocação por cartão frio por quadro**. **Tablet:** cada sub-wave imprime o custo nos 3 alvos.

### W5 — ALÇAS NO CANVAS, DERIVADAS DA TABELA (capacidade 5)
- **Derivação** (`shells/desktop/src/param_handles.rs`, tabela pura sobre
  `ParamUiHint`+`ParamUnit`): `(<b>_x, <b>_y)` ambos `Length` ⇒ `Handle::Point` (**20 pares
  hoje**) · `radius`/`<b>_radius` `Length` ⇒ `Handle::Ring` · `angle`/`rotation`/`<b>_angle`
  `Angle` ⇒ `Handle::Dial` · `width`+`height` ⇒ `Handle::Box`. **127 params** com unidade espacial.
- **Os dois à mão viram os primeiros consumidores:** gate
  `the_derived_handle_for_field_box_equals_the_hand_written_one` (números iguais ao bit), depois o
  código à mão de `field_gizmo.rs:209` **é apagado** (sem segunda fonte).
- **Sink:** `MotionParamIntent::SetParam` (undo por diff); com auto-record ligado ⇒ chave (W3).
  Só para o nó **seleccionado**; hit ≥ 44 pt no toque; espaço de mundo.
- **Bidireccional por construção** (a alça lê o param que escreve) — o Blender 4.3 exige ligar
  *Gizmo nodes* à mão ([fonte](https://projects.blender.org/blender/blender/pulls/112677)).
- **Gates:** `every_spatial_pair_in_the_registry_grows_a_handle` (censo derivado — um nó novo com
  `center_x/center_y` ganha alça de graça) · `a_handle_drag_is_one_undo_step` ·
  `a_handle_never_paints_for_an_unselected_node`.
- **Smoke:** `PH2D_HANDLES_SMOKE=1`. **Orçamento:** 1 overlay por nó seleccionado. **Tablet:**
  0 px permanentes.

### W6 — CUSTO E ROTA NO CARTÃO (capacidade 7 — a metade visível de W0)
- **Badge** sob o título (overlay ligável, como o *Timings* do Blender): `⚡ 0,12 ms` (device,
  `last_stage_ns`) · `CPU 3,4 ms` (`cost_of`) · `CPU · sem kernel` / `CPU · param dirigido` / …
  (a `Refusal` de W0, **por nó**); o balão diz a razão inteira e **a cura** quando há uma (o
  `converter_from` já deriva conversores; a lane de W1 apaga duas razões).
- **HUD por cena** (canto do canvas, overlay): rota · passes · ms CPU/GPU — a linha do
  `PH2D_MOTION_ROUTE_LOG` no ecrã.
- **Gates:** `every_refusal_has_a_badge_text` (tabela `Refusal → &'static str`, censo
  exaustivo — `match` sem `_`) · `the_badge_never_paints_below_the_lod` ·
  `a_badge_with_no_clock_says_so` (sem `TIMESTAMP_QUERY` ⇒ *«GPU · sem relógio»*).
- **Smoke:** ligar o overlay (`O`? — decidir a tecla no design system) numa cena `=108`.
- **Orçamento:** texto por cartão só com o overlay ligado. **Tablet:** 0 px sem o overlay.

### W7 — BUSCA 2.0: o fio solto que se CURA (capacidade 4)
- **Hoje:** `resolve_loose_output_drop` já abre o menu filtrado por `menu_catalog(snap, from)`
  (doc 59/63.3). **Falta:** (a) quando o tipo NÃO encaixa em nenhum candidato directo, oferecer
  **«via <conversor>»** (o `converter_from` de `motion_bridge_subgraph.rs`) e inserir os DOIS nós
  ligados a um clique; (b) ordenar por **recência/uso** (`~/.ph2d/prefs.txt`, chave
  `node_recent=`); (c) aceitar a origem num **`ParamSocketOut`** (W2: oferece *«Number from
  param»*); (d) *Shift = ramo · Ctrl = substituir · repetir o último* (doc 63 I5, o idioma do Nuke).
- **Gates:** `a_loose_wire_that_needs_a_converter_offers_it_and_inserts_both` (costura real) ·
  `the_recent_list_never_hides_a_compatible_node` (é ordem, não filtro).
- **Orçamento:** 0 ms de quadro (só no gesto). **Tablet:** menu ≥ 44 pt por linha.

### W8 — NÓ PRÓPRIO: subgrafo com INTERFACE + BIBLIOTECA (capacidade 6 — o W-J do doc 63)
- **Interface = promoção + fórmula (a lei do Houdini):** *«When you promote a parameter, Houdini
  creates a copy … and replaces the original … with an expression that references the value of
  the parameter on the asset»*. Aqui: a interface de um subgrafo é uma lista de
  `(nome, ParamUiHint, valor)` gravada no documento do subgrafo (`ph2d-motion-doc`, registo
  próprio — o agente lê o formato desse crate antes de escolher a letra), e cada param interno
  promovido recebe **`f = @g<subgraph_id>.<nome>`** — W2 resolve `@g` como resolve `@n`. **Zero
  mecanismo novo de cook.** O cook continua PLANO (doc 57).
- **Cartão do subgrafo:** as rows da interface, pelo mesmo renderer (W4); portas = o que cruza a
  fronteira (já derivado).
- **Ficheiro:** `.ph2dnode` = o texto do sub-grafo (formato `format.rs`, ids re-mapeados a partir
  de 1) + a interface + nome + versão + categoria; biblioteca em `~/.ph2d/nodes/` e
  `<projecto>/nodes/`; a paleta lista-os numa secção **«Yours»** com a mesma busca.
- **Inserir = COPIAR** com `Graph::insert_raw(id, type_name)` e o re-mapeamento de ids (fase A);
  **ligado** (hash de conteúdo + *«Update from library»*) é a fase B — o modelo Components
  (ADR-0164, mestre/instância por `StableId`) é o precedente da casa, ⛔ não reinventar.
- **Gates:** `a_promoted_inner_param_is_an_expression_to_the_interface` ·
  `a_saved_node_reinstantiates_with_fresh_ids_and_the_same_bits` ·
  `the_library_palette_lists_only_readable_files_and_says_why_not`.
- **Smoke:** «DominoWave» salvo, re-instanciado e dirigido só pela interface (doc 63 W-J).
- **Orçamento:** zero por quadro (é documento). **Contrato:** `NodeManifest` intocado — um nó
  próprio **não é um tipo do registry**; é um subgrafo com nome.

### W9 — FUSÃO DE KERNELS: uma cadeia por-elemento = UM passe (capacidade 8)
- **Censo (2026-09-04):** 70 kernels; **56** são mapas puros (`count_law: None`, sem `StreamOp`);
  12 mudam a contagem; `StreamOp` em 17 (`SourceRows` 7 · `Concat` 4 · `Compact` 4 · `Project` 2).
  A tee custa **3 passes**, o oscilador 2; o `motion.drive` que lê um `value.lfo` de comprimento 1
  já vira uniform em W1.
- **Regra de fusão** (em `plan.rs`, depois de `plan()`): estágios **consecutivos** na ordem
  topológica, cada um `count_law: None` e sem `StreamOp`, sem `pre`, sem `applicable` que varie,
  com **uma** aresta entre eles (o consumidor lê só o produtor) e a mesma contagem ⇒ um
  `FusedStage { members: Vec<GpuStage> }`.
- **Codegen** (`codegen.rs`): concatenar os corpos na ordem; os acessores `read_<col>(i)` /
  `write_<col>(i, v)` viram **variáveis locais** (`var c_<col>: <dim>`) carregadas uma vez do
  buffer de entrada e escritas uma vez no de saída no fim; `params.<name>` de cada membro
  prefixado (`p<k>_<name>`) num único uniform struct; `wgsl_lib` deduplicado por hash; presença e
  broadcast (`presence_signature`, `broadcast_mask`) calculados sobre a **união** das bindings.
  ⛔ Um membro cuja binding é `Consume` ou que lê uma coluna que o anterior **remove** quebra o
  grupo (a fusão é por leitura/escrita de colunas — a `ColumnAccess` é a régua).
- **Limites (medir em W0):** membros por grupo (tempo de compilação WGSL contra passes
  poupados) · `max_storage_buffers_per_shader_stage` (união das bindings) · tamanho do uniform.
- **`to_wgsl` ganha o primeiro consumidor:** o `motion.expression` (hoje `LoweringKind::Cpu`)
  passa a gerar o corpo com `ph2d_expr::to_wgsl(expr)` sobre os acessores — é o *compiled
  block* do Houdini e o *context* do VFX Graph, com o que nenhum dos dois tem: **paridade
  bit-a-bit com a CPU provada por gate**.
- **Gates:** `a_fused_chain_is_bit_identical_to_the_unfused_one` (o arnês de paridade, sobre as
  109 cenas — mutação: trocar a ordem dos membros) · `a_fusion_never_crosses_a_count_change` ·
  `the_tee_runs_in_one_pass` · `fusion_is_off_by_env_for_bisection` (`PH2D_GPU_FUSE=0`).
- **Smoke:** `PH2D_MOTION_ROUTE_LOG=1` numa cena de 10 modificadores: passes antes/depois.
- **Orçamento:** o objectivo é **passes ↓**, ms ↓ medidos pela sonda de W0; a régua honesta é a
  tabela das 109 cenas antes/depois, com `/proc/loadavg` ao lado.

### W10 — SPREADSHEET · SONDAS · INSPECÇÃO (capacidade 10 — doc 100 §7.4, doc 63 I7)
- **Balão do socket com as colunas** (`stream_at` existe; `tip_for_hot` é só do pino quente):
  `Out · a stream · P size rot falloff · 400 rows` — a espécie que DESCREVE (a que ACUSA foi
  recusada, handoff 03/09 §6).
- **Painel `ph2d-panel-motion-spreadsheet`:** instâncias × colunas do nó seleccionado, sobre
  `Cook::peek` (e o `gpu_tap` para a porta 0 no device); filtros; **escolher uma instância no
  canvas** (hit-test nas instâncias compostas) selecciona a linha — Houdini/Blender não ligam o
  clique no viewport à linha.
- **Sondas:** um nó de valor mostra o **sparkline** dos últimos 120 quadros no readout (doc 63
  «probe+sparkline ✓» — conferir se o sparkline existe; o readout existe).
- **Gates:** `the_socket_tip_lists_the_columns_the_stream_carries` ·
  `the_spreadsheet_shows_what_peek_holds` · `picking_an_instance_selects_its_row`.
- **Orçamento:** só com o painel aberto; **tablet:** painel dobrável, custo impresso nos 3 alvos.

---

## §5 — ORDEM, dependências e a definição de PRONTO

```
W0 instrumentos ─┬─► W1 lanes ─┬─► W2 fórmulas+pick-whip ─► W8 nó próprio
                 │             ├─► W3 keyframes (pede W4.T4 dock re-smoke)
                 │             └─► W7 busca 2.0
                 ├─► W6 badge de custo
                 └─► W9 fusão
W4 cartão 2.0 (doc 101) ─► W5 alças ─► W10 spreadsheet
```
- **Paralelizável em Modo L:** `W0+W1+W9` (uma linha de substrato, `ph2d-gpu-cook`/`nodegraph`)
  · `W4+W5+W10` (uma linha de UI, `ph2d-panel-*` + shell) · `W2+W3+W7+W8` (uma linha de
  documento/cook, depois de W1). ⚠️ As três tocam `motion_bridge_*` no shell — corram
  `collision-surface.sh` antes do primeiro grep (CLAUDE §1).
- **Nasce DESLIGADO até a lane existir:** W2 e W3 só ligam por omissão depois de W1 (lei §2.3);
  W9 nasce com `PH2D_GPU_FUSE=1` por omissão **só** depois de a paridade sobre as 109 cenas
  estar verde.
- **PRONTO de uma wave = os 6:** gates red-first verdes **com a mutação registada** · paridade
  CPU/GPU onde toca o device · smoke em passos numerados para o Enio · orçamento medido
  (ms/quadro, passes, alocações por cartão) com `loadavg` · custo no tablet nos 3 alvos · o
  handoff com as *«coisas que uma leitura rápida do diff entende ao contrário»*.

## §6 — CONTRATOS e FORMATO (o que muda, o que NÃO muda)
| superfície | congelada? | o que este plano faz |
|---|---|---|
| `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` | **sim** (§6, ADR-0039) | **nada** — toda metadata nova é `register_*` no registry; todo estado novo é documento |
| `Tool=12` / `PanelEvent=4` | **sim** | **nada** — o cartão e o painel falam por `MotionParamIntent`/`GraphIntent` |
| `Graph` + `format.rs` | não (política de versão: bump **iff** o registo existe) | `f` (**v6**, fórmula) · `q` (**v7**, promoção) · `c`/`w` sob `[layout]` (não-semânticos, sem bump) |
| `GpuKernel` / `StreamOp` / `GpuPlan` | não (side-metadata) | `Compact.complement` · `GpuStage.port` · `GpuPlan.refusals` · `FusedStage` |
| `ph2d-timeline` `PropKind` | enum fechado, numérico | `Param = 6` (append) + bump do `DOC_VERSION` do `TimelineDoc` pela regra dele |
| `ph2d-anim::AnimTarget` | opaco, previsto para «a node param» | `fnv64("motion-param\0<id>\0<param>")` no shell |
| `ProjectState::capture` | assinatura com o ledger | ganha o `MotionPreview` **na assinatura** (nunca uma irmã) |

Prova a correr no fecho de cada wave: `architecture_contract_surface` ·
`architecture_tool_contract_surface` · `every_widget_file_wires_a11y` · os tectos de LOC (700 por
ficheiro — cortar por responsabilidade, nunca por isenção) · `no_magic_numeric` · tofu (zero
`→`/`⇒` em literais de string) · `doc-index.sh --check`.

## §7 — RECUSAS MEDIDAS e anti-padrões (não reconstruir)
- ⛔ pinos `value`/`pulse` no `motion.oscillator` (doc 100 §6) · ⛔ todos os params no cartão por
  omissão (doc 101 §4) · ⛔ `renderUI` por nó (segunda fonte) · ⛔ widgets vivos em todo cartão ·
  ⛔ chips que ACUSAM (`drops`, handoff 03/09) · ⛔ referências por NOME em fórmulas (o AE parte) ·
  ⛔ nós escondidos a implementar fórmulas (um mundo paralelo) · ⛔ um buffer por param dirigido
  (8 storage buffers é o mínimo WebGPU) · ⛔ ler timestamps com `Maintain::Wait` no quadro ·
  ⛔ fundir através de um `count_law`/`StreamOp`/`pre` · ⛔ ligar W2/W3 por omissão antes de W1.
- Os 8 anti-padrões do **doc 63 §4.3** valem inteiros (stamps sem política de custo · min/max
  que não clampa · toggle que não gateia · controlo destrutivo no corpo · prioridade numérica ·
  duas semânticas num fio · contrato implícito de ordem · escape-hatch lento sem aviso).

## §8 — O que é decisão do ENIO (poucas, e nomeadas)
1. Confirmar o **visual**: a nossa spec (Mini Cavalry) + os três comportamentos do Blender (§1).
2. A tecla/gesto do **overlay de custo** (W6) e se ele nasce ligado no smoke.
3. **Solo** (W4): sem mecanismo definido — fica nomeado até haver um sink temporário com lei.
4. Biblioteca de nós **ligada** (fase B de W8) — quando, e se segue o modelo Components.
5. A ordem entre as três linhas paralelas da §5, se houver menos de três janelas.

## §9 — Fontes
[Houdini – rename](https://www.sidefx.com/docs/houdini/network/rename.html) ·
[Houdini – ch()](https://www.sidefx.com/docs/houdini/expressions/ch.html) ·
[Houdini – asset UI](https://www.sidefx.com/docs/houdini/assets/asset_ui.html) ·
[Houdini – VEX](https://www.sidefx.com/docs/houdini/vex/index.html) ·
[Artivoxa – compiled block](https://www.artivoxa.com/houdini-invoke-compiled-block-speeding-up-heavy-sop-networks/) ·
[Cavalry 1.3 – attribute expressions](https://cavalry.studio/docs/tech-info/release-notes/1.3/1-3-0-release-notes/) ·
[Cavalry – connections](https://cavalry.studio/docs/getting-started/key-concepts/connections/) ·
[CreativeCOW – AE pick-whip breaks](https://creativecow.net/forums/thread/error-when-pick-whipping-properties-across-comps-a/) ·
[Blender 3.1 – link drag search & timings](https://developer.blender.org/docs/release_notes/3.1/nodes_physics/) ·
[Blender – timings commit](https://projects.blender.org/blender/blender/commit/e4986f92f32) ·
[Blender 4.3 – gizmos](https://www.blender.org/download/releases/4-3/) ·
[Blender – gizmo PR](https://projects.blender.org/blender/blender/pulls/112677) ·
[Blender – node groups](https://docs.blender.org/manual/en/latest/interface/controls/nodes/groups.html) ·
[Blender – keyframes](https://docs.blender.org/manual/en/latest/animation/keyframes/editing.html) ·
[Blender – socket shapes 5.0](https://code.blender.org/2025/08/new-socket-shapes/) ·
[Unity VFX – custom HLSL](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@16.0/manual/Block-CustomHLSL.html) ·
[TouchDesigner – custom parameters](https://docs.derivative.ca/Custom_Parameters) ·
[wgpu – timestamp queries](https://wgpu.rs/doc/wgpu_examples/timestamp_queries/index.html) ·
[Apple HIG – 44 pt](https://www.lukew.com/ff/entry.asp?1085=)

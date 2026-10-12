# 102 — O OUTRO PATAMAR: o plano dos nós além de Blender, Cavalry e Houdini

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/102_o_outro_patamar_plano_dos_nos_2026-09-04.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Pinos `value`/`pulse` no `motion.oscillator` | Recusado em doc 100 §6 | vivo §7 |
| Todos os params no cartão por omissão | Recusado por medição (doc 101 §4: 48 % no oscilador, 80 % no `bezier_warp`) | vivo §1.1 / §7 |
| `renderUI` por nó; widgets vivos em todo cartão; chips que ACUSAM | Segunda fonte de UI; chips acusadores recusados (handoff 03/09) | vivo §7 |
| Referências por NOME em fórmulas; nós escondidos a implementar fórmulas | O AE parte no rename (guarda-se o id); nós escondidos são um mundo paralelo | vivo §7 |
| Um buffer por param dirigido | 8 storage buffers é o mínimo WebGPU | vivo §7 |
| Ler timestamps com `Maintain::Wait` no quadro | Bloqueia o quadro | vivo §7 |
| Fundir através de `count_law`/`StreamOp`/`pre`; ligar W2/W3 por omissão antes de W1 | Quebram a contagem/o plano; a ordem das lanes é pré-requisito | vivo §7 / arquivo §4 W9 |
| `Attr` (coluna por-elemento) num campo | Recusado com a cura «a field is one number per node» | [arquivo](../archive/motion-nodes-2026-10-12/102_o_outro_patamar_plano_dos_nos_2026-09-04.md) §4 W2 |

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

> ⛔⛔ **EMENDADO EM 2026-09-05 pela [dinâmica dos ciclos](103_dinamica_dos_ciclos.md) — leia as
> três emendas ANTES de usar este plano:**
> 1. **A ordem já não é «W0..W10»: é um CICLO por GRUPO de nós** (doc 103 §1/§5). As waves daqui
>    continuam a ser a verdade TÉCNICA de cada capacidade — elas são chamadas de dentro dos
>    ciclos, não corridas em fila.
> 2. **O cartão hospeda TODOS os params e o painel lateral SAI** (decisão do Enio). A §5 D1
>    daqui — *o cartão magro, e os params entram por promoção* — está **revogada**; o que a
>    substitui é a lei do doc 103 §4 (todos os params, secções dobráveis, LOD, widgets só no
>    cartão quente). O registo `q` (promoção) deixa de ser o portão de entrada de um param no
>    cartão e passa a ser, quando muito, uma FIXAÇÃO na ordem.
> 3. **O preço já não é o que a §4 do [doc 101](101_pesquisa_cartoes_ricos_2026-09-04.md) supôs**:
>    medido em 2026-09-05, uma row **no cartão** custa **2,8 µs** (a do painel, 13,5 — são duas
>    coisas: widget vivo contra dois textos), e o pior nó do catálogo cabe em **9 %** do quadro.
>
> O que NÃO mudou e continua a ser a razão de este doc existir: a **§0 (o índice das portas)**,
> a **§2 (as leis de performance)**, a **§6 (os contratos)** e a **§7 (as recusas)**.

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

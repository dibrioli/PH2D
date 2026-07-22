# Plano — Módulo Motion Nodes da PH2D

**Data:** 2026-07-07 · **Status:** aprovado para execução em linha (Modo L) · **Estudo-base:**
[`00_estudo_estado_da_arte.md`](00_estudo_estado_da_arte.md) · **Referência canônica de UX/semântica:**
MiniCavalryV2 (`/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2`, **read-only**, testado e aprovado).

---

## ⚠️ ESTADO REAL — auditoria de 2026-07-22 (leia ANTES de usar este plano como fila)

Este plano foi **cumprido e superado**. O texto abaixo é o de 2026-07-07 e, em quatro pontos,
**descrevia coisas que hoje são falsas** — deixá-las de pé faz a próxima LLM construir o que já
existe, ou reconstruir o que foi deliberadamente revogado. As correções estão marcadas **inline**
com `⚠️ SUPERSEDED`; este bloco é o índice delas.

| Fase | Estado (verificado no repo, não de memória) |
|---|---|
| **M0** | ✅ **13/13** tasks |
| **M1** | ✅ editor E1–E10 + P1/R1 + nós (alguns por **supersessão documentada**: `value.clamp` vive no `map_range` (doc 13), `value.random` no `instance_field` (doc 12)) |
| **M2** | ✅ neck (`cook_scoped` + `CheckpointRing`) + nós |
| **M3** | ✅ **15/15** nós; editor F3 entregue em forma **diferente e melhor** (doc 46) |
| **M4** | 🟡 rig 6/6 ✔ · subgrafo+breadcrumb ✔ (doc 57) · param→socket ✔ (doc 58) · **FX de PASSE 0/6** |
| **M5** | ✅ **excede o plano** — entregue pela linha `line/gpu-nodes` (ADR-0135..0140) |

**Contrato congelado: NUNCA foi tocado** (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`). Os dois
amendments que o §4 previa **dissolveram**: o M4.N1 (`ParamSpec` tipado) virou o **canal de text
param** (doc 32) e o M4.N3 (`Domain::Rig`) foi decidido *contra* — um esqueleto é stream comum.

**Contagem:** o plano projetou ~135 nós MVP / ~90 crates. Hoje: **90 crates-nó, 88 registradas**.

### ⚠️ O que o censo de cobertura GPU de fato diz (medido 2026-07-22)

`cargo test -p ph2d-host-desktop --bins gpu_coverage_census -- --nocapture` reporta o **boot
document** como `HYBRID/CPU`, fronteira `motion.distribute_poisson`; e a demo 2 como `HYBRID/CPU`,
fronteira `motion.sort`. **Nenhum dos dois é um buraco de performance** — os dois são *boundaries
ESTÁTICOS* (`Effect::Pure`, sem aresta `delayed`, sem param dirigido ⇒ **constantes**, cozidas uma
vez e enviadas uma vez). O `plan.rs` **nomeia o caso por escrito**: o Poisson é *"an inherently
sequential algorithm that will never have a kernel"*, e o boundary estático existe exatamente para
ele não arrastar o laço.

> **A lição, e ela é reutilizável: o censo conta FRONTEIRAS, não CUSTO.** Um `source` puro no topo
> da cadeia é o limite mais barato que existe; um nó **no meio de um stream vivo** é o caro, porque
> arrasta para a CPU todo mundo acima dele — inclusive nós que TÊM kernel.

E o corpus do censo **não contém um único deformer** ⇒ ele não consegue ver o buraco real
([[feedback_a_fixture_only_proves_what_it_contains]]). É esse buraco que a fila abaixo ataca.

### A fila REAL (o que sobrou, em ordem de valor medido)

1. **Deformers na GPU** — ⬛ **O CANAL LANDOU (2026-07-22, esta sessão) e a família de redução
   está FECHADA.** O primitivo de reduce reusável (`ph2d-gpu-cook::reduce`, irmão do `scan`) +
   o **6º canal do resolver** `ReduceSpec` (`ph2d_nodegraph::reduce_meta`, side-metadata no
   registry, `default []`, append-only, **contrato congelado intocado**) destravaram a forma
   `reduce → broadcast → map`. **Quatro nós na GPU, cobrindo os 3 operadores e a pluralidade:**
   `motion.bend` (Max sobre `|x−pivot|`, **bit-exato** — sem produto) · `motion.twist` (Max sobre
   `√(dx²+dy²)`, ε) · `motion.spherize` (**Sum**, e as **DUAS** reduções do centróide) ·
   `motion.four_point_warp` (**QUATRO** reduções = o bbox, e a estreia do **Min**; box bit-exato,
   só a homografia carrega ε). Provado no device (RTX): pior ε **3,8e-5** vs bound **medido 2e-4**;
   gate de paridade + excursão + seam + mutações por operador; WGSL valida em todo o espaço de
   presença (sem device); censo GANHOU deformers (antes não via o buraco). Smokes
   `PH2D_GPU_COOK_DEMO=12/13/14`. **Restam SEM redução, de propósito:** `kaleidoscope` é
   **count-changing** (`StreamOp`, replica em N fatias — outra máquina) e `lattice` é
   gerador/distribuição, não deformer de stream — nenhum é fatia do canal de redução.
2. **Auto-inserção de adapters** (§1.1) — a tabela `CONVERSIONS` e o `can_connect` **nunca foram
   construídos**; hoje o editor RECUSA fio incompatível em vez de oferecer o adapter. Os nós-adapter
   existem; falta a ponte.
3. **`motion.delay`** e **`motion.distribute_path`** (o alimentado pelo `vector.*`) — os 2 nós que
   sobraram da FILA 4. O `distribute_curve` diz no próprio doc-comment que o path cross-module é
   *"separate, later"*.
4. **Wire-insert** (soltar da paleta sobre um fio) — o irmão dele (drop no card = port picker) **já
   existe** (`snapshot_drop.rs`).
5. **FX de PASSE** (`glow`/`bloom`/`blur`/`vignette`/`levels`/`hue_shift`) — compositor HDR,
   **cross-module**: a doc 38 §8 manda **PARAR e reportar ao Enio**. Reuso obrigatório do
   compositor 22-modos do Painter — escrever um segundo bloom é dívida, não feature.
6. **`AttrAccess`** — ⚠️ o tipo **nunca foi construído**; é citado num doc-comment do
   `registry/ui.rs` como se fosse chegar. O `flow.rs` explica por que não usou: os atributos que um
   nó lê/escreve **não estão no `NodeManifest`**, que é CONGELADO. A influência entregue é
   **estrutural** (por arestas) e funciona; a por-atributo **custa um ADR**.
7. **W4.T4 — dock da timeline** no `motion_timeline_slot` (ainda `h=0`). ⚠️ Encosta na linha
   `anim` — **duas linhas no mesmo módulo é proibido**: exige ordem do Enio.

> **Norte:** sistema de motion nodes extremamente poderoso, intuitivo e fácil de usar, mirando o
> ápice de beleza e performance com Rust/WGPU. Port **por semântica**, nunca por representação,
> **com os melhoramentos** (correções das fraquezas do MVP) embutidos por construção.

## Decisões de produto (Enio, 2026-07-06)

1. **Escopo: só motion** (mundo declarativo). Gameplay/blocos→Luau = módulo futuro (ADR-0036).
2. **Split fixo viewport+grafo** com a tool Motion ativa, orientação configurável
   **horizontal** (viewport em cima/grafo embaixo, Cavalry) **ou vertical** (lado a lado,
   TouchDesigner).
3. **Timeline DEFERIDA** (plano próprio). Este módulo só deixa o encaixe: `motion_timeline_slot`
   no layout + hierarquia de resolução de param `socket > keyframe > literal` reservada.
4. **Node Maker descartado**; a **decoração visual do MVP é copiada** (anatomia, cores, silhuetas,
   sockets, fios, activity-fire, overlays).

## Contexto

O substrato já existe e está congelado (ADR-0030..0039): Cook pull memoizado por fingerprint,
`playhead` de 1ª classe, `Effect::Temporal`, feedback `pre` de 1 tick, `Stream` SoA,
`PortType{Domain,Dim,Clock}` estrito, fan-out por drop-crate. Faltam: o fio do tempo no shell,
o vocabulário de nós (~135 no MVP), o editor de grafo (zero UI hoje) e o caminho GPU.
Fraquezas do MVP corrigidas por construção: clone imutável O(N·M) allocs/frame → Stream SoA +
memoização; wall-clock default + `Math.random` → tick fixo + hash-PRNG stateless; forces que não
integram → integrador real; re-avaliação total por frame → Cook fingerprint; scrub que reseta
estado → checkpoints.

---

## 1. Arquitetura do runtime

### 1.1 Tipos: os 7 do MVP → PortType (substrato intocado)

| MVP | PortType | Payload |
|---|---|---|
| shape **≡ point** | `(Instances, Vec2, Frame)` | `Stream` (point = menos colunas; defaults preenchem) — **elimina a conversão mais comum por unificação** |
| value | `(Instances, Scalar, Frame)` | `Stream` count-1 (broadcast) ou count-N |
| color | `(Instances, Vec4, Frame)` | RGBA **linear straight** |
| pulse | `(Instances, Scalar, Event)` | 0/1 true-por-tick (lição Rive: trigger de 1ª classe) |
| gradient | `(Field, Vec4, Static)` | `Opaque(Arc<GradientStops>)` |
| skeleton | `(Vector, Mat3, Frame)` | `Opaque(Arc<Skeleton>)` (precedente ADR-0058-am.1) |

> ⚠️ **SUPERSEDED (parcial, 2026-07-22): os nós-adapter EXISTEM; a AUTO-INSERÇÃO não.** Não há
> `CONVERSIONS` nem `can_connect` no repo — o editor pergunta só `connects_directly` e **recusa**
> o fio incompatível (toast). Item 2 da fila nova.

**Conversões = adapter-nodes auto-inseridos pelo editor.** O substrato continua estrito
(`connects_directly`); o adapter é nó `Pure` comum (diffável, params editáveis — o "fio
tracejado" do MVP vira a renderização compacta do adapter). Crates `ph2d-node-adapt-*`:
`value_to_color`, `luminance`, `threshold` (value→pulse, Schmitt; a travessia Frame→Event
legal), `gate` (pulse→value), `sample_gradient`, `make_point` (par X/Y). Tabela estática
`CONVERSIONS: &[(PortType, PortType, &str)]` no editor;
`can_connect = connects_directly || CONVERSIONS`.

### 1.2 Colunas canônicas do Stream (extensão de convenção; dona: ph2d-eval-motion)

`P`(Vec2)→world_pos · `size`(Vec2) · `rot`→basis · `tint`(Vec4 linear) · `opacity` ·
`falloff` (**multiplicativo**, consumido por modifiers) · `seed` (default `hash(node_seed,i)`) ·
`pivot`→anchor · `vel`(Vec2) · `age`/`life` · `z`→z_order quantizado (`base_z +
(z·65535) as u32`; sem coluna = ordem do stream, sort estável preserva) · `cell`/`uv_rect`→atlas_uv ·
**`inv_mass`** (Scalar, o `w = 1/m` do PBD — `1` = livre (default quando ausente), `0` = pinado;
escrito por `motion.pin_constraint`, **consumido pelos solvers** `motion.integrate`/`spring`/`collide`
— doc 34).
**`index`/`count`/`time` implícitos, não materializados** (o nó `expression` injeta `i,n,t` como
bindings virtuais). Cor: **linear no fio, OKLab dentro dos nós perceptuais**.

### 1.3 Estado temporal: self-loop `pre` carregando estado como CookValue (SEM extensão de contrato)

- Estado preferencialmente **como colunas** (spring: `P`+`vel`; verlet: `P`+`P_prev`) —
  visível, serializável, **loweriza 1:1 para ping-pong GPU**.
- Nó sequencial = template do editor com self-loop `pre` já ligado
  (`state_out --pre--> state_in`). `Effect::Pure` (dt fixo; tick entra no fingerprint via pre).
- **Histórico N-frames** (delay/trail/slitScan): o ring buffer É o valor do self-loop —
  `History { slots: Box<[Arc<Stream>]>, head, len }` (clone = N refcounts, nunca dados).
- **Partículas**: spawn determinístico `floor((tick+1)·rate·dt) − floor(tick·rate·dt)`;
  identidade = `hash(node_seed, spawn_index)`; toda aleatoriedade deriva do hash → re-sim bit-exata.

### 1.4 Tempo, transporte e scrub

> ⚠️ **SUPERSEDED (W4.T7, 2026-07-12): o `MotionTransport` MORREU.** O Motion não tem relógio
> próprio — ele cozinha no tick do **`Playhead`** (`motion_bridge::ticks_owed`: play = TODO tick
> para a frente, porque a sim é sequencial; scrub/jump = UMA chamada, sem replay). Relógio único
> era o pré-requisito real do W4.T4. Um teste que queira rodar a sim constrói um `Playhead`.
> O `Cook::checkpoint`/`restore` + `CheckpointRing` do parágrafo abaixo **estão vivos** (doc 11).

`MotionTransport { tick: u64, playing, accumulator, loop_range }`; playhead = `tick × FIXED_DT`
(1/60, do FixedStep de ph2d-core). Nós de sim usam **dt fixo constante**.
`Cook::checkpoint()/restore()` (APIs aditivas, nota-ADR): ring de checkpoints (a cada 60 ticks,
últimos 32) → scrub para trás = restore + re-sim ≤59 ticks. Melhor que o MVP (que resetava).

### 1.5 timeRemap: escopos de tempo no Cook (única mexida em substrato; interna + aditiva)

`Cook::cook_scoped(..., scopes: &BTreeMap<NodeId, TimeMap>)` — ao descer pelos inputs de um nó
remapeador, empilha `t' = map(t)`; **cache keyed por `(NodeId, ScopeKey)`** (FNV da cadeia de
remaps ativos). Loop → subárvore vem do cache quando `t'` repete (o MVP recomputava); freeze →
congela de graça; diamante cruzando remap → correto. Fora de escopo ScopeKey=0 (custo zero).
Restrição v1: nó sequencial dentro de escopo remapeado é recusado (badge).

### 1.6 Forces corrigidas (bug semântico do MVP: `out = in + offset/frame`)

Força = nó `Pure` que **acumula `accel`** (`accel += f(P,vel)·falloff`); integração = UM nó
sequencial `motion.integrate` (**semi-implícito Euler**). Topologia:
`emitter → [pre state] → force₁…forceₙ → integrate → out`. Curl noise divergence-free
(Bridson 2007): ψ = fBm de value-noise, `v = ∇×ψ` por diferenças centrais — transcendental-free,
bit-idêntico CPU/GPU. Boids: spatial hash grid (célula = raio, 9 vizinhas, O(N·k)).

### 1.7 Algoritmos (referências)

| Peça | Algoritmo |
|---|---|
| Noise | Perlin 2002 gradient noise (LUT 8 direções por hash + fade quíntico); fBm. Base = família `ph2d_noise1` (paridade WGSL provada) |
| PRNG | Hash stateless PCG/splitmix (Jarzynski & Olano 2020, JCGT) — scrub-safe, GPU-safe |
| Falloff | linear · smoothstep · smootherstep · quad; invert = 1−f |
| Easing | ~20 de `simple_easing` (MIT, cópia interna); polinomiais transcendental-free; sine/circ/elastic marcadas não-det (presentation é HR-5-exempt) |
| Spring | Semi-implícito + sub-step adaptativo do MVP (`subDt²·tension < 0.05`) — portar literal |
| IK | 2-bone lei dos cossenos; FABRIK (Aristidou & Lasenby 2011, ≤10 iter) |
| Soft/rope | XPBD (Macklin 2016) + small steps (Macklin 2019) |
| Poisson-disk | Bridson 2007 O(N) · Voronoi: Lloyd · Fibonacci: Vogel |
| Blur/bloom | dual-Kawase (Bjørge 2015); bloom threshold + down/up chain no HDR existente |
| expression | parser VEX-lite no crate do nó → IR `ph2d-expr`; `pow`: desugar k-inteiro (Func::Pow = amendment M4) |

### 1.8 Frame path zero-alloc + documento

- `AppGfx.motion: MotionState { doc: MotionDoc, history, cook (persistente), registry,
  transport, sinks, scopes, instances: Vec<RenderInstance> (reusado), checkpoints }`.
- `MotionDoc` (crate `ph2d-motion-doc`): wrapper de `Graph` + `backdrops` + `base_z`; persiste
  no **formato textual v1** (seção UI-only `[backdrop]` irmã de `[layout]`).
- `motion_bridge.rs` (molde `vector_bridge.rs`; **substitui `motion_smoke.rs`**): transport →
  n ticks fixos (`cook_scoped` + `advance_tick`) → `evaluate_motion_into(..., &mut instances)`.
- **Destino: slice direto na coleta do sprite pass** (sort global + run-batching; 1 submit).
  NÃO spawn em PresentWorld (ADR-0035: stream ≠ ECS).
- Gates dhat: paused = **0 allocs**; playing = orçamento declarado. Stream COW por coluna só
  se profile pedir (measure first).

### 1.9 GPU — o ápice, em 2 estágios

- **(a) agora:** CPU cook + GPU instancing maduro (144B RenderInstance, sort+run-batching,
  HDR Rgba16Float → AgX). Teto prático 50–100k instâncias.
- **(b) M5:** `ph2d-motion-gpu` — **CookPlan** particiona o grafo: runs contíguos de nós `Pure`
  elemento-a-elemento com `LoweringKind::Wgsl` fundem num kernel único (`to_wgsl` concatenado;
  1 dispatch/segmento); nós que mudam count cortam segmento. Self-loop `pre` = ping-pong de
  storage buffers 1:1. Zero readback (buffer de saída layout-compatível com RenderInstance).
  Meta: 1–5M instâncias (fill-rate-bound).
- **FX:** afim por instância → coluna/atributo (hueShift, levels, tint; mirror = dup+flip_uv;
  rgbSplit = 3×+tint; dropShadow = dup atrás); espacial → pass no compositor HDR (glow/bloom,
  blur dual-Kawase, vignette). Documento declara `layer_fx`.

---

## 2. Arquitetura do editor

### 2.1 Split viewport↔grafo (foundational leve)

`CenterSplit::{None, Horizontal{t}, Vertical{t}}` (t clamp 0.25..0.75) + `center_viewport`,
`motion_graph`, `motion_timeline_slot` (h=0 — encaixe da timeline) no `HeroLayout`.
Config em `HeroScreen.view` (orientation default Horizontal + t 0.55, persistidos como view
prefs); chips SplitH/SplitV/Fit no toolbar do grafo. Divisor 6px arrastável. Sprite pass:
Fase A = fundo opaco do grafo cobre sprites; Fase B = `set_viewport/set_scissor_rect` +
`Camera2d::uniform_for_subrect` (~20 LOC, enquadramento correto).

### 2.2 Foundational de dispatch (editor-core; molde BlenderHit + CurvePoint)

`InteractiveState::GraphSurface { parent, kind: GraphHitKind, canvas: Rect }` com
`GraphHitKind::{Background, Node{node}, SocketIn/Out{node,port}, Wire{edge},
Waypoint{edge,index}, Backdrop{id}, BackdropResize{id}, PreviewToggle{node}, SplitDivider}`.
Editor-core **não conhece semântica de grafo**: o dispatch stasha
`GraphGesture{kind, phase: Begin/Update/End/Click/DoubleClick, x, y, button, mods}` no
WidgetStore (`push/drain_graph_gestures`, `set_graph_zoom`, `push_graph_key`); o painel drena
e interpreta. **Risco verificado:** middle-button não chega ao dispatch hoje → plumbing
`PointerButton` no foundational (fallback: pan = drag em Background; box-select = Shift+drag).
Hit de fio: bezier flattenada → 6–10 rects de 25px com o MESMO id (registrados antes dos nós;
cull ao rect visível).

### 2.3 Crates e fluxo

| Peça | Crate | Regime |
|---|---|---|
| Tool | `ph2d-tool-motion` | fan-out |
| Painel do grafo | `ph2d-panel-motion-graph` | fan-out (o grande) |
| Painel de params (Inspector takeover) | `ph2d-panel-motion-params` | fan-out |
| Documento | `ph2d-motion-doc` | fan-out |
| Bridge | `shells/desktop/src/render_loop/motion_bridge.rs` | serial (shell) |
| Dispatch/layout/tokens/ícones | editor-core / ph2d-tokens | **FOUNDATIONAL** |

Painel↔bridge sem downcast fora do bridge (canais estáticos, molde
`set_current_vector_style`): bridge publica `GraphViewSnapshot { nodes, edges (out_type +
value_hash), violations, previews, probe, cook_epoch }`; painel devolve
`GraphIntent::{AddNode, MoveNodes, Connect, Disconnect, InsertOnWire, DeleteSelection,
SetParam, AddBackdrop…, SetPreview, SetProbe, SetSplit}`. Undo: `MotionHistory` (snapshot da
doc, 1 passo por gesto — pre no Begin, push no End, molde RECOLOR_PRE). Estado efêmero
(pan/zoom/seleção/drag/popup) = `Panel::State`, não-undoable.

### 2.4 Decoração (tradução exata do MVP → tokens/Vello)

- **~26 ColorTokens novos** (macro `color_tokens!` + tokens.json ×4 temas; gate
  `no_literal_color`): 7 famílias de categoria (`node-cat-source` #3E7F4F, `-distribute`
  #5A8A6C, `-transform` #4A6FA5, `-focus` #A87C3A, `-fx` #9C4A8B, `-output` #B0463A,
  `-utility` #888 → OKLCH); 7 de porta **por eixo do PortType** (`port-instances` roxo,
  `-vector` azul, `-field` laranja c/ gradiente interno, `-signal` teal, `-control` lima,
  `-event`, `-static`); `graph-bg`, `graph-grid`, `wire-fire-glow`; 8 de backdrop translúcido;
  `attr-write` dourado.
- **Card:** 220px lógico × zoom, body `Bg2` neutro, borda 1px; **só o header tintado**
  (gradiente vertical `mix(cat_token, Bg2, 0.55)→Bg2`); selected = stroke 2px Accent + ring
  AccentSoft; error = badge `IconId::Warning`.
- **7 silhuetas** via `RoundedRectRadii` 4 cantos (helpers `fill/stroke_rounded_rect_corners`):
  rect[8] modifier · cigar[22] merge/grupo · circle[16] terminal · diamond[4]+losango 8px no
  header · trapezoidDown[18,18,4,4] source · trapezoidUp sink · tabbed[10]+triângulo.
- **Sockets:** cor = Domain, **forma = Dim** (Scalar ▬, Vec2 ■, Vec3/4 ◆, Mat ▮, Event ▶);
  stream = anel dashed externo; hover scale 1.35 + glow.
- **Fios:** CubicBez tangente horizontal (`|dx|·0.5+20`), Catmull-Rom por waypoints; cor =
  Domain da saída; espessura Event 1.4 / resto 2.6 (×zoom); **dashed = `edge.delayed`**
  (vello 0.8 processa `dash_pattern` — confirmado); taper via `variable_width_band` (polish).
- **Activity-fire:** `value_hash` do edge mudou entre cooks → 800ms de glow decadente + dash
  marchando (`dash_offset` por frame) + 2 orbs percorrendo o path (`eval_path_at_arclen`:
  tabela de arclen por segmento + `ParamCurve::eval`).
- **Compatibilidade ao vivo** no drag de fio: compatíveis scale+glow; incompatíveis
  `mix(token, Bg2, 0.7)`.
- **~7 IconId novos em ordem alfabética:** Backdrop, FitView, Knife, MotionNodes, Probe,
  SplitHorizontal, SplitVertical (reusa Eye/EyeClosed/Search/Warning).

### 2.5 Painel de params + metadata lateral

Takeover do slot Inspector; visível só com a tool ativa. `NodeManifest.params` só tem
`{name, default: f32}` → **metadata lateral no `ph2d-node-registry`** (não congelado, keyed
por NodeTypeId): `ParamUiHint {param, label, min, max, step, widget:
Slider/IntSlider/Angle/Toggle/Seed, unit}` + `AttrAccess{reads, writes}` (chips + influence) +
`NodeUiManifest {display_name, category, silhouette, custom_section: Option<fn>}` (escape
hatch = o renderUI do MVP). Rows geradas: slider + number chip (`link_slider_number` +
`set_number_range`); sem hint → fallback Slider. Ids dinâmicos: fnv64 runtime
`"motion_param/{node}/{param}"` (precedente hierarchy). Labels EN result-named (ADR-0038 §5,
HR-15). **Promoção param→socket e "olhinho": deferidos** (exigem porta dinâmica no modelo).

### 2.6 Overlays

Probe (P+click no fio → balão + sparkline 60 amostras) · Influence (I → BFS downstream via
AttrAccess; resto alpha 0.35) · Live-preview por nó (Eye → cozinha porta 0 **só quando
cook_epoch muda**, ≤512 instâncias, scatter no flap; máx 4 LRU, zoom>0.6 — ADR-0038 §2) ·
Geometry Spreadsheet **deferido**.

### 2.7 Gestos (mapa 1:1 do MVP)

Drag nó (multi) · click/shift seleção · drag socket→socket conecta (recusa tipada = toast) ·
drag reverso do input ocupado · **drag→vazio = smart-connect** (busca fuzzy só compatíveis,
exatos primeiro, ↑↓+Enter cria nó+fio) · drop no card = port picker · alt+click remove edges ·
drag Background = box-select (Shift aditivo) · middle-drag pan · wheel zoom ancorado
(0.2..2.5) · R-click vazio = add-node · R-click nó = menu · middle-click fio = waypoint
(+branch) · drop da paleta sobre fio = wire-insert (auto-layout +240px) · K+drag knife ·
Delete · F fit · Ctrl+A/D/G · Esc · Ctrl+Z global · drag divisor · chips SplitH/SplitV/Fit.

---

## 3. Roadmap e TASKS por fase

> Modelo funil (como node-waves): **neck serial** → fan-out paralelo (1 crate/agente, briefing
> `docs/IntegracaoMultiAgente/briefing-node-crate.md`). Inner loop = `cargo check -p`; gate
> batched no fim de cada wave; ship 1× por jornada (DIRETRIZ §1.5.4).
> Template de node-crate: `crates/ph2d-node-debug-wave/`.

### M0 — Foundational + fio do tempo (NECK, serial, 2 PRs)

| # | Task | Onde | Notas |
|---|---|---|---|
| M0.T1 | `PointerButton` plumbing (middle) winit_host → input_dispatch → dispatch | editor-core + shell | verificar o caminho ANTES; fallback documentado (pan = drag Background) |
| M0.T2 | `InteractiveState::GraphSurface` + `GraphHitKind` + canal `GraphGesture` (push/drain, set_graph_zoom, push_graph_key) | `interaction/state/mod.rs`, `interaction/types.rs` | molde BlenderHit + CurvePoint |
| M0.T3 | Arms nos dispatch: pointer_down (captura+Begin; R-click antes dos ContextMenuKind), pointer_move (Update mesmo fora do rect), pointer_up (End/Click/DoubleClick), scroll (zoom ancorado, consome antes de panel_scroll), key (Delete/F/A/Esc/K/P/Ctrl+D com focus no grafo) | `interaction/dispatch/*` | ~120 LOC; testes de dispatch |
| M0.T4 | `CenterSplit{None,Horizontal{t},Vertical{t}}` + `center_viewport`/`motion_graph`/`motion_timeline_slot` + `for_viewport_split` | `screens/layout.rs` | timeline_slot h=0 (encaixe deferido); view prefs orientation+t |
| M0.T5 | ~26 ColorTokens (macro + tokens.json ×4 temas) | ph2d-tokens + docs/design/tokens.json | famílias §2.4; gate no_literal_color cobra |
| M0.T6 | ~7 IconId (ordem alfabética!) + SVGs 24×24 | editor-core icons | Backdrop, FitView, Knife, MotionNodes, Probe, SplitHorizontal, SplitVertical |
| M0.T7 | Crate `ph2d-motion-doc`: `MotionDoc{graph, backdrops, base_z}` + seção `[backdrop]` no formato textual + `MotionHistory` (molde vec-edit::History) | crate novo | serialização round-trip test |
| M0.T8 | `MotionTransport` (tick fixo→playhead, play/pause, accumulator do FixedStep) + `MotionState` em `AppGfx` | ph2d-motion-doc + app_state.rs | transport mínimo: play/pause + tick readout no toolbar do grafo |
| M0.T9 | Esqueletos registrados: `ph2d-tool-motion` (pill, IconId::MotionNodes) + `ph2d-panel-motion-graph` + `ph2d-panel-motion-params` (codegen tool-sync + panel-sync) | crates novos | gotcha: FloatingPanel não pinta; visibilidade via panel_visibility |
| M0.T10 | `motion_bridge.rs`: visibility edge-triggered + CenterSplit no hero + drena intents + cook por frame; **deleta `motion_smoke.rs`** | shell render_loop | molde vector_bridge.rs; downcast allowlisted |
| M0.T11 | `evaluate_motion_into`/`lower_to_instances_into` (buffer reusado, multi-sink, base_z, colunas §1.2) + injeção do slice na coleta do sprite pass | ph2d-eval-motion + renderer | teste unit das colunas novas |
| M0.T12 | **Teste dhat** paused-0-alloc do caminho bridge+cook+lower | tests | molde `propagate_no_alloc.rs` |
| M0.T13 (PR-2) | Fase B viewport: `set_viewport/set_scissor_rect` + `Camera2d::uniform_for_subrect` | ph2d-render + shell | ~20 LOC; golden do enquadramento |

**Gate M0:** tool Motion ativa → split H e V funcionam; grafo grid→transform→clone cozinha por
frame via bridge e renderiza no viewport; dhat verde; arch-gates verdes.

### M1 — Editor usável + Wave-1 de nós (fan-out máximo)

**Editor (serial dentro do crate ph2d-panel-motion-graph; dividível em 2 agentes):**

| # | Task |
|---|---|
| M1.E1 | Paint de cards: header tint gradiente por categoria, 7 silhuetas (`fill/stroke_rounded_rect_corners`), título, ícone, badge ⚠ (Graph::validate → violations no snapshot) |
| M1.E2 | Sockets por Dim/Domain (formas §2.4) + labels de porta; io-row |
| M1.E3 | Fios: CubicBez + flatten → hit-segments (mesmo id, cull); dashed nos `delayed` |
| M1.E4 | Pan/zoom ancorado + F (zoomToFit) + grid de fundo (`graph-bg`/`graph-grid`) |
| M1.E5 | Seleção: click/shift, box-select, multi-drag (MoveNodes = 1 undo no End) |
| M1.E6 | Connect: drag socket→socket com validação (`can_connect`), recusa = toast + piscada; Disconnect alt+click |
| M1.E7 | Add-node: R-click com busca fuzzy por categoria (cores da biblioteca ensinam o mapa); Delete de seleção (fios órfãos juntos) |
| M1.E8 | Undo/redo (MotionHistory via bridge, Ctrl+Z global) |
| M1.E9 | Divisor arrastável + chips SplitH/SplitV/Fit no toolbar do grafo |
| M1.E10 | `GraphViewSnapshot`/`GraphIntent` completos no crate do painel (canais estáticos) |
| M1.P1 | Painel de params v1: rows geradas de `ParamUiHint` (slider + chip linkados, `set_number_range`), fallback sem hint, ids dinâmicos fnv64, seção por nó selecionado |
| M1.R1 | Metadata lateral no `ph2d-node-registry`: `ParamUiHint` + `AttrAccess` + `NodeUiManifest` keyed por NodeTypeId (aditivo, não congelado) |

**Nós (~30 crates, 100% paralelos, 1 agente cada; template debug-wave; cada um = manifest +
eval + golden test + ParamUiHint/AttrAccess + i18n):**

| Lote | Crates |
|---|---|
| Geradores/distribuição | `motion-duplicator` · `motion-distribute-grid` · `-circle` · `-random` (hash-PRNG §1.7) |
| Behaviours | `motion-stagger` (rampa×easing×falloff, aditivo) · `motion-oscillator` (+porta pulse no zero-crossing) · `motion-lfo` |
| Falloffs | `falloff-circle` · `falloff-rect` · `falloff-linear` (curvas linear/smoothstep/quad, invert; **multiplicativo** — cada um multiplica no upstream) |
| Modifiers | `motion-move` · `motion-scale` · `motion-rotate` · `motion-tint` (todos respeitam `falloff`) |
| Values | `value-random` · `value-map-range` · `value-clamp` · `value-modulate` |
| Streams | `motion-mixer` (avg/add ≤4) · `motion-combine` (concat) |
| Cor | `color-array` · `color-sample-gradient` · `color-range-to-color` (OKLab interno, linear no fio) |
| Expressão | `motion-expression` (parser VEX-lite → ph2d-expr; `i,n,t` virtuais; erro → badge) |
| Adapters | `adapt-value-to-color` · `adapt-luminance` · `adapt-threshold` · `adapt-gate` · `adapt-sample-gradient` · `adapt-make-point` + tabela CONVERSIONS no editor |

**Gate M1 (smoke com Enio):** o "demo Cavalry" — grid 20×20 + stagger + oscillator + falloff +
colorArray, montado NO editor, 60fps.
`cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop`

### M2 — Tempo e dinâmica (neck pequeno + fan-out)

**Neck (nota-ADR, cook.rs — Coordenador/linha única):**

| # | Task |
|---|---|
| M2.N1 | `Cook::cook_scoped` + cache `(NodeId, ScopeKey)` + `TimeMap` (loop/freeze/reverse/speed) |
| M2.N2 | `Cook::checkpoint()/restore()` + `CheckpointRing` no transport (60 ticks, 32 slots) |
| M2.N3 | Testes: diamante cruzando remap · loop vem de cache (contador de evals) · scrub restore+re-sim · recusa de sequencial em escopo |
| M2.N4 | Nota-ADR documentando as APIs aditivas (caps intocadas) |

**Nós (fan-out):** `motion-spring` (sub-step adaptativo §1.7; self-loop pre; colunas P+vel) ·
`motion-delay` · `motion-trail` (History §1.3) · `motion-sample-hold` · `motion-counter` ·
`pulse-threshold` (Schmitt) · `pulse-on-change` · `pulse-compare` · `pulse-switch` ·
`motion-time-remap` · `motion-integrate` + `force-attractor` · `-vortex` · `-wind` · `-drag` ·
`-curl-noise` (§1.6) · `-buoyancy` · `motion-particle-emitter` (spawn determinístico) ·
`motion-wiggle` · `motion-noise` (Perlin §1.7).

**Editor (F2):** smart-connect popup (busca fuzzy compatíveis + auto-inserção de adapter) ·
drag reverso · compatibilidade ao vivo (glow/dessatura) · chips de atributo no rodapé
(AttrAccess) · backdrops (add/move/resize/rename + `[backdrop]`) · waypoints + branches ·
knife · wire-insert com auto-layout · readouts inline no body · template "nó sequencial"
(self-loop pre pré-ligado na criação).

**Gate M2:** spring com scrub para trás correto (checkpoint) · loop de timeRemap sem recompute
(contador) · partículas determinísticas (re-sim = mesmo hash de colunas) · smoke com Enio.

### M3 — Distribuições avançadas, deformers + polish wow

**Nós (fan-out):** `motion-distribute-fibonacci` (Vogel) · `-poisson` (Bridson) · `-voronoi`
(Lloyd) · `-path` (integra vector.*) · `motion-lattice` · `-bend` · `-twist` ·
`-four-point-warp` · `motion-morph` (vertex/switch/crossfade) · `motion-look-at` ·
`motion-slit-scan` · `motion-boids` (spatial hash §1.6) · `motion-verlet-rope` ·
`motion-soft-body` (XPBD) · `motion-pin-constraint`.

**Editor (F3):** activity-fire completo (value_hash + glow + dash marchando + orbs via
`eval_path_at_arclen`) · probe + sparkline (ring 60) · influence (BFS por AttrAccess) ·
live-preview flaps (throttle cook_epoch, máx 4 LRU) · taper `variable_width_band` ·
gradiente interno em portas Field.

**Gate M3:** cena 10k+ instâncias com forças+boids a 60fps (medir — HR-4); overlays sem
degradar frame; smoke com Enio.

### M4 — Rig + FX (necks: 2 amendments)

**Necks:**

> ⚠️ **SUPERSEDED (2026-07-22): os 3 necks estão RESOLVIDOS, e DOIS deles sem tocar o contrato.**
>
> | # | Desfecho real |
> |---|---|
> | **M4.N1** | **NÃO bumpou o `NodeManifest`.** O **canal de TEXT PARAM** (doc 32 — `Graph::set_text_param` + `EvalCtx::text_param`) deu a `motion.expression` com param string **sem tocar o congelado**: params vivem no `Graph`, não no manifest. **É o padrão canônico para param não-f32.** A promoção param→socket saiu depois pelo `GraphIntent::DriveParam` (doc 58), também aditiva |
> | **M4.N2** | **NÃO feito, e nem precisa hoje:** `Func::Pow` não existe no `ph2d-expr`; a `motion.expression` shipa `sin cos abs sqrt floor fract min max mix noise select`. Nenhum nó pede `pow` — acorda quando um pedir |
> | **M4.N3** | **DECIDIDO CONTRA.** Não há `Domain::Rig`: *um esqueleto é um stream de instâncias comum* — `parent`/`len`/`rot` são colunas ordinárias, então todo nó genérico funciona sobre um rig e `rig.*` não precisou de mudança de contrato (racional completo em `ph2d-node-rig-skeleton`) |

| # | Task |
|---|---|
| M4.N1 | **ADR `ParamSpec` tipado** (`ParamValue{F32,Vec2,Color,Enum,Bool}` — cap-bump ADR-0039); inclui a hierarquia socket>keyframe>literal (preparo da timeline futura) |
| M4.N2 | Amendment `Func::Pow` no ph2d-expr + re-prova de paridade CPU↔WGSL |
| M4.N3 | Decisão do tipo skeleton definitivo (`Domain::Rig` só se `(Vector,Mat3)` apertar) |

**Nós:** `rig-skeleton` · `rig-fk` · `rig-ik-2bone` (lei dos cossenos) · `rig-fabrik` ·
`rig-rubber-hose` · `rig-skin-deformer` · FX por instância `fx-mirror` · `fx-rgb-split` ·
`fx-drop-shadow` · FX passes `fx-glow` · `fx-bloom` · `fx-blur` (dual-Kawase) · `fx-vignette` ·
`fx-levels` · `fx-hue-shift` (no compositor HDR; `layer_fx` no documento).
**Editor:** grupos/subgrafo + breadcrumb · promoção param→socket (agora com porta dinâmica).

### M5 — GPU CookPlan (satélite `ph2d-motion-gpu`; design doc próprio antes de codar)

| # | Task |
|---|---|
| M5.T1 | Design doc: particionamento topológico, formato dos segmentos, bindings de colunas |
| M5.T2 | Kernel fusion dos runs Pure (to_wgsl concatenado; 1 dispatch/segmento; plan cacheado por hash topológico) |
| M5.T3 | Ping-pong dos self-loops `pre` (storage buffers = prev_outputs) |
| M5.T4 | Buffer de saída layout-compatível com os 12 atributos GPU do RenderInstance (zero readback) |
| M5.T5 | Harness de paridade CPU↔GPU por segmento (molde do padrão spatial-GPU do painter) |
| M5.T6 | Bench: meta 1M instâncias; medir onde vira fill-rate-bound |

### Deferidos (registrados, fora deste plano)
Timeline + keyframes UI (plano próprio; encaixe: `motion_timeline_slot` + ParamSpec tipado +
socket>keyframe>literal) · Geometry Spreadsheet · audioReact (precisa input de áudio) ·
dataSource CSV/JSON · exports (Lottie/MP4/WebM) · gameplay/blocos (ADR-0036).

## 4. Amendments de contrato (lista exaustiva — todo o resto é fan-out/convenção)

1. **M2, nota-ADR:** `cook_scoped` + `checkpoint/restore` (internals/aditivo; caps intocadas).
2. **M4, ADR real:** `ParamSpec` tipado (cap-bump NodeManifest).
3. **M4, amendment:** `Func::Pow` (re-prova paridade CPU↔WGSL).
4. **Só se profile pedir:** Stream COW por coluna (nunca antes de M3; measure first).

## 5. Verificação

- Por task: `cargo check -p <crate>`. Por wave: `bash scripts/nextest-impacted.sh` + clippy
  `--all-targets` + auditoria ≥2 lentes sobre o diff acumulado.
- Gates novos: dhat paused-0-alloc · golden test por nó (mesmo graph+seed+tick → mesmas
  colunas) · escopo de tempo (diamante/loop/cache, contadores de eval) · re-sim de partículas
  bit-exata.
- Arch-gates existentes: `architecture_contract_surface` · staleness tool-sync/panel-sync/
  node-sync · `no_literal_color` · icon slug order.
- Smoke visual com Enio no gate de cada wave:
  `cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop`
- Ship 1× por jornada: `./scripts/ship.sh` → push → babysit (DIRETRIZ §1.5.4).

## 6. Arquivos críticos

> ⚠️ **SUPERSEDED (2026-07-22)** em dois pontos desta lista: **`motion_smoke.rs` não existe** (foi
> aposentado — a autoria real o tornou obsoleto) e o **`MotionState` não guarda transporte** (§1.4).
> O bridge é hoje uma família de ~24 arquivos `render_loop/motion_bridge*.rs`, não um só. E o
> `ph2d-motion-gpu` do M5 **não** foi construído com esse nome: quem entrega é **`ph2d-gpu-cook`**
> (+ `ph2d-gpu`), da linha `line/gpu-nodes`.

- `crates/ph2d-nodegraph/src/cook.rs` — cook_scoped/checkpoint (único toque em substrato, M2)
- `crates/ph2d-eval-motion/src/lib.rs` — colunas canônicas + evaluate_motion_into (M0)
- `crates/ph2d-editor-core/src/interaction/{state/mod.rs, dispatch/pointer_*.rs}` — GraphSurface (M0)
- `crates/ph2d-editor-core/src/screens/layout.rs` — CenterSplit (M0)
- `crates/ph2d-tokens/src/color.rs` + `docs/design/tokens.json` — tokens novos (M0)
- `shells/desktop/src/render_loop/motion_bridge.rs` (novo; molde `vector_bridge.rs`; substitui
  `motion_smoke.rs`) + `app_state.rs` (MotionState) — M0
- Crates novos: `ph2d-motion-doc` · `ph2d-tool-motion` · `ph2d-panel-motion-graph` ·
  `ph2d-panel-motion-params` · ~90 `ph2d-node-*` (template `ph2d-node-debug-wave`) ·
  `ph2d-motion-gpu` (M5)
- Referência viva (read-only): `/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2`

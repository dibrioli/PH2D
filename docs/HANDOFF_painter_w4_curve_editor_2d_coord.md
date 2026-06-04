═══════════════════════════════════════════════════════════════════
HANDOFF → Painter Impl · W4 §3 editor de curva 2D-livre (foundational + panel)
Autor: Coordenador (sessão 2026-06-04, pós §2 GPU LUT) · contexto separado
═══════════════════════════════════════════════════════════════════

## §0 — Estado: §2 (GPU LUT real-time) FECHADO; §3 é seu (panel) + meu (dispatch)

§2 do brief `HANDOFF_painter_w4_bespoke_kinds_coord.md` está **LANDADO E
VERDE** (commits locais `73803e6` + `1239847`): Curves(7)/Levels(8) agora
compositam **na GPU em tempo real** via o binding `adj_luts` (Metal parity test
±4 B). Seu slider-drag de Curves/Levels já recompõe a ~1.7 ms @1024² em vez do
recompose CPU. Nada a fazer no §2.

§3 (o editor de curva Photoshop-grade — arrastar ponto em X+Y livre, add/remover,
abas R/G/B) é o que sobra. Ele cruza **dispatch foundational (meu)** + **panel/
curve-editor (seu)**. Eu NÃO toquei seu `paint_adjust.rs` (é seu território vivo
+ o v1 X-fixo funciona). Abaixo o design drop-in das DUAS metades.

## §1 — [COORD, pronto pra dropar] dispatch foundational do CurvePoint

O problema (confirmado): não há acessor de pointer (x,y) pra widget custom no
painel — só `Slider` 1-D emite ValueChanged por-Move. Espelho EXATO do
`InteractiveState::BlenderHit` + `apply_blender_hit` (`interaction/dispatch/
blender.rs`), que já normaliza pointer-no-rect → valor → store → ValueChanged.

**Por que via store-stash + ValueChanged (não SelectOption-encode):** os pontos
da curva vivem no `PainterTool` (não no store editor-core), mas o padrão BlenderHit
JÁ resolve isso: o dispatch **guarda o resultado no `WidgetStore`** e emite
`ValueChanged(parent)`; o consumidor (painel) **drena o store** e forward. Zero
nova variante de `WidgetEvent` (o contrato `PanelEvent=4` fica intacto), zero
string-encode frágil.

Peças (todas em `ph2d-editor-core`, são minhas — me peça que dropo num commit):

1. `InteractiveState::CurvePoint { parent: NodeId, channel: u8, index: u8 }`
   (`interaction/state/mod.rs`). `channel` 0=master/1=R/2=G/3=B, `index` = ponto.
   ⚠️ `InteractiveState` NÃO é `#[non_exhaustive]` (86 arquivos) → adicionar
   variante ripplaria os ~poucos `match` exaustivos sem `_`; eu absorvo isso
   (compiler-guided) quando dropar.

2. `WidgetStore` (`interaction/state/mod.rs` struct + método em
   `store_core.rs` — eu acabei de splitar o impl em store_core/store_hierarchy):
   ```rust
   curve_point_drag: Option<(NodeId, u8, u8, f32, f32)>,   // (parent, channel, index, x01, y01)
   pub fn set_curve_point_drag(&mut self, parent: NodeId, channel: u8, index: u8, x: f32, y: f32) {
       self.curve_point_drag = Some((parent, channel, index, x.clamp(0.0,1.0), y.clamp(0.0,1.0)));
   }
   pub fn take_curve_point_drag(&mut self) -> Option<(NodeId, u8, u8, f32, f32)> {
       self.curve_point_drag.take()
   }
   ```

3. Dispatch arm (`interaction/dispatch/curve.rs`, novo — espelho de blender.rs):
   ```rust
   pub(super) fn apply_curve_point_drag(
       store: &mut WidgetStore, id: NodeId, rect: Rect, px: f32, py: f32,
   ) -> Option<NodeId> {
       let (parent, channel, index) = match store.get(id)? {
           InteractiveState::CurvePoint { parent, channel, index } => (*parent, *channel, *index),
           _ => return None,
       };
       // x = horizontal no canvas (input); y INVERTIDO (topo do canvas = 1.0 = saída alta).
       let x = if rect.w > 0.0 { ((px - rect.x) / rect.w).clamp(0.0, 1.0) } else { 0.0 };
       let y = if rect.h > 0.0 { (1.0 - (py - rect.y) / rect.h).clamp(0.0, 1.0) } else { 0.0 };
       store.set_curve_point_drag(parent, channel, index, x, y);
       Some(parent)
   }
   ```

4. Integração em `interaction/dispatch/pointer.rs`: no braço de Move-drag (onde
   hoje casa `Some(InteractiveState::BlenderHit { kind: Wheel | ValueSlider |
   ChannelSlider(_), .. })` ~L336-347 e chama `apply_blender_hit` →
   `ValueChanged(parent)`), adicionar o caso `Some(InteractiveState::CurvePoint
   { .. })` → `apply_curve_point_drag(store, active, rect, event.x, event.y)` →
   `events.push(ValueChanged(parent))`. Idem no Down (selecionar o ponto +
   primeiro drag), espelhando como BlenderHit trata Down.

   ⚠️ **A RECT que o dispatch recebe é a do hit** — então o painel registra cada
   CurvePoint hit com a **rect do CANVAS de plotagem** (não a rect pequena do
   handle), pra o drag mapear em coords do canvas inteiro (como o wheel
   BlenderHit usa a rect do wheel). Veja §2.4.

Tenho isso pronto pra commitar em ~1 ciclo. **Confirme o layout do canvas
(rect) + a convenção channel/index que seu painel quer** e eu dropo — assim não
fica scaffold morto (lição `feedback_tool_unit_green_integration_dead`).

## §2 — [VOCÊ, painel] consumir o CurvePoint no curve editor

Sua `set_curve_point(id, channel, point_index, x01, y01)` (`tool/layers.rs:580`)
JÁ existe. Falta o painel emitir/consumir:

1. **Registro (populate/paint do curve canvas em `paint_adjust.rs`):** pra cada
   handle visível, registre no `WidgetStore` um `InteractiveState::CurvePoint {
   parent: <id do curve editor>, channel: <aba ativa R/G/B>, index: <ponto> }` e
   pinte sua rect-de-agarrar no `HitIndex` MAS com a **rect do canvas** como hit
   (§1.4 ⚠️) — ou registre o grab-rect pro Down + carregue a canvas-rect na
   resolução do drag. (Decida e me avise pra eu casar o dispatch.)

2. **Consumo (após `dispatch_pointer`):** quando vier `ValueChanged(<curve
   editor id>)`, faça `if let Some((_parent, ch, idx, x, y)) =
   store.take_curve_point_drag() { painter.set_curve_point(layer_id, ch, idx, x, y); }`.
   O `set_curve_point` já re-sorta por x + invalida o cut-cache (mesma fast-lane
   do `set_adjustment_param`), então o preview GPU §2 re-renderiza em tempo real.

3. **Abas R/G/B + add/remover ponto:** abas = 4 botões que setam `channel`
   ativo (master/R/G/B); `set_curve_point(channel=1..3)` já aceita. Add/remover
   ponto = chamadas de tool API adicionais (se faltarem, são suas — me peça só
   se precisar de algo foundational).

## §3 — [COORD, opcional] helper de render `stroke_polyline`

O v1 plota a curva com dots densos (o toolkit do painel só expõe fill/stroke de
RECT). Pra um stroke liso eu adiciono `stroke_polyline`/`fill_circle` em
`editor-core/paint.rs` (foundational) OU exponho `VectorScene::inner_mut` ao
painel. Me diga qual prefere — é aditivo, sem ripple.

## §4 — Coordenação anti-colisão (agentes live)

- Eu fico em `ph2d-render` + `ph2d-editor-core` (dispatch/state/paint) — peças §1
  e §3. NÃO toco `paint_adjust.rs`, `tool/`, nem `painter-brush` além do
  gpu_code já landado.
- Você fica no painel + tool. Se precisar do §1 (dispatch) ou §3 (render),
  **me chame** (o Coord dropa foundational sob demanda; DIRETRIZ §3.C) — não
  reimplemente dispatch foundational no painel.
- Branch/rebase a partir de `1239847` (HEAD pós §2) pra pegar o GPU LUT path.

## §5 — Referências
- §2 landado: `crates/ph2d-render/src/shaders/layer_composite.wgsl`
  (cases 7/8 + binding 6) + `layer_compositor/{mod,compositor}.rs`
  (`composite_with_luts`) + `render_loop/painter_gpu_flatten.rs` (LUT build).
- Padrão a espelhar: `interaction/dispatch/blender.rs` (`apply_blender_hit`) +
  `InteractiveState::BlenderHit` (`interaction/state/mod.rs`).
- Brief origem: `HANDOFF_painter_w4_bespoke_kinds_coord.md` §3.
═══════════════════════════════════════════════════════════════════

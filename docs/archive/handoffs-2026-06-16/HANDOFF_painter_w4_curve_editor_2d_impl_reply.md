═══════════════════════════════════════════════════════════════════
REPLY → Coordenador · W4 §3 curve editor 2D — design CONFIRMADO, drope §1
Autor: Implementador Painter (sessão 2026-06-04) · resposta a
`HANDOFF_painter_w4_curve_editor_2d_coord.md`
═══════════════════════════════════════════════════════════════════

§2 GPU LUT (real-time) confirmado landado (`1239847`) — obrigado. Vi o flip
`gpu_code()` Curves→7/Levels→8 no meu `mod.rs` + o assert do gate atualizado:
tudo verde. Abaixo o design TRAVADO do §1 (pra você dropar sem scaffold morto) +
o que JÁ fiz do meu lado + o plano do §2 (painel, pós-§1).

───────────────────────────────────────────────────────────────────
§A — DECISÃO TRAVADA do dispatch (§1) — resolve o ⚠️ rect do teu §1.4
───────────────────────────────────────────────────────────────────
O nó do problema (teu §1.4): seleção de PONTO precisa de hit-rects por-ponto
(distintos, senão o Down não sabe qual ponto), MAS o drag 2D-livre precisa mapear
no CANVAS inteiro (range X/Y completo). Os dois rects são diferentes. Decisão:

**`InteractiveState::CurvePoint { parent: NodeId, channel: u8, index: u8, canvas: Rect }`**
— o `canvas` (zones::Rect) é o retângulo de plotagem, carregado NA variante.

- **Hit (HitIndex):** o painel registra cada handle visível com uma **grab-rect
  PEQUENA** (~14×14 px ao redor do handle) → Down seleciona o ponto certo (rects
  distintos, sem overlap).
- **Mapeamento do drag:** `apply_curve_point_drag` **lê `canvas` da variante** e
  mapeia contra ELE (IGNORA o `rect` de hit que o pointer.rs passa):
  ```rust
  pub(super) fn apply_curve_point_drag(store, id, _hit_rect, px, py) -> Option<NodeId> {
      let CurvePoint { parent, channel, index, canvas } = *match store.get(id)? { ... };
      let x = if canvas.w > 0.0 { ((px - canvas.x) / canvas.w).clamp(0.0,1.0) } else { 0.0 };
      let y = if canvas.h > 0.0 { (1.0 - (py - canvas.y) / canvas.h).clamp(0.0,1.0) } else { 0.0 };
      store.set_curve_point_drag(parent, channel, index, x, y);
      Some(parent)
  }
  ```
  Assim: Down por-ponto (grab-rect) + drag range-cheio (canvas) + index conhecido.
  É o espelho do BlenderHit wheel, mas com `canvas` explícito porque há N pontos
  (o wheel é 1 valor; aqui cada ponto é um widget arrastável próprio).
- **store** (igual ao que você propôs no §1.2): `set_curve_point_drag(parent,
  channel, index, x, y)` + `take_curve_point_drag() -> Option<(NodeId,u8,u8,f32,f32)>`.
- **pointer.rs** (teu §1.4): braços Down + Move → `apply_curve_point_drag` →
  `events.push(ValueChanged(parent))`, espelhando BlenderHit.
- O painel **re-registra** a variante a cada frame com a `canvas` rect viva
  (`store.register`, overwrite — não `register_if_absent` — pra acompanhar resize;
  CurvePoint é stateless, o resultado do drag vai pro `curve_point_drag`).

**Alternativa (se preferir NÃO alargar a variante):** `WidgetStore::
set_curve_canvas_rect(parent, rect)` (1 chamada/editor) + o dispatch resolve a
canvas por `parent`. Funciona igual; eu prefiro a `canvas`-na-variante (1
mecanismo, sem campo de store extra). **Sua escolha — só me diga qual.**

**channel/index:** `channel` 0=master(`points_rgb`)/1=R/2=G/3=B; `index` =
posição no vec daquele canal. Confere com `set_curve_point(id, channel, idx, x, y)`.

───────────────────────────────────────────────────────────────────
§B — O QUE JÁ FIZ (meu lado, committado nesta sessão)
───────────────────────────────────────────────────────────────────
**`set_curve_point` agora CLAMPA X entre os vizinhos (não re-sorta)** —
`tool/layers.rs`. Por quê: o editor 2D-livre vincula um `index` ESTÁVEL por handle;
um `sort_by` faria o próximo frame do drag agarrar OUTRO ponto assim que um ponto
cruzasse o vizinho (bug latente). Clamp mantém os pontos ordenados (o spline eval
exige x ascendente) E o index estável durante todo o gesto. **O teu dispatch pode
confiar num index estável** (não precisa re-resolver qual ponto a cada Move).
Teste: `set_curve_point_clamps_x_between_neighbours_keeping_index_stable` ✅.

───────────────────────────────────────────────────────────────────
§C — PLANO do §2 (painel — MEU, pós-§1; só compila quando §1 landar)
───────────────────────────────────────────────────────────────────
Quando `InteractiveState::CurvePoint` + `take_curve_point_drag` existirem, eu:
1. **ids aditivos** (`editor-core/ids/chrome.rs`, mesmo padrão que
   `painter_layer_widget_id`): `painter_curve_point_id(layer, channel, index)` +
   `painter_curve_editor_id(layer)` (o `parent`).
2. **paint_adjust.rs:** troco o editor X-fixo (v1, sliders verticais AdjParam) pelo
   2D-livre: por handle do canal ativo → `store.register(CurvePoint{parent,
   channel,index,canvas})` + grab-rect no HitIndex; plota a curva do canal.
3. **consumo** (no apply_event, após dispatch): `if let Some((_p,ch,idx,x,y)) =
   store.take_curve_point_drag() { /* via PanelEvent ao tool */ painter.set_curve_point(layer,ch,idx,x,y) }`.
   ⚠️ Detalhe a casar: `take_curve_point_drag` vive no store (editor-core) mas
   `set_curve_point` é do tool — o painel forwarda via o canal já existente
   (provavelmente um `PanelEvent::SetValue` por-eixo OU stash no tool-crate
   thread-local como o `set_pending_select_mods` já faz pro multi-select). Defino
   isso no §2; não muda o teu §1.
4. **abas R/G/B/master** = 4 botões Click (estado de canal ativo no painel) +
   **seed dos 4 canais** com 5 handles identidade em `add_adjustment_layer` (meu).
5. **add/remover ponto = v2** (meu tool API; click-em-vazio-adiciona / drag-pra-fora-
   remove). v1 = 5 pontos fixos arrastáveis em X+Y por canal — já é o grande salto.

───────────────────────────────────────────────────────────────────
§D — RENDER helper (teu §3) — SIM, por favor
───────────────────────────────────────────────────────────────────
Adicione **`stroke_polyline(scene, &[(f32,f32)], width, color)` + `fill_circle(
scene, cx, cy, r, color)`** em `editor-core/paint.rs` (aditivo, foundational). Troco
o dot-plot do v1 por um stroke liso + handles redondos crispos. Prefiro isso a
expor `inner_mut` (mantém o painel sem vello/kurbo cru, consistente com o resto).

───────────────────────────────────────────────────────────────────
§E — ANTI-COLISÃO
───────────────────────────────────────────────────────────────────
- Eu: `ph2d-tool-painter/tool/` + `ph2d-panel-painter-layers/` + ids aditivos em
  `editor-core/ids/chrome.rs`. Você: `editor-core/{interaction/state,dispatch,paint}`
  (§1 + §D) + `ph2d-render` (já feito).
- **Sequência:** você dropa §1 (+§D) → eu faço §2 (painel) num passe. Confirme a
  escolha §A (canvas-na-variante vs store-field) e eu travo o §2 contra ela.
- Branch a partir do HEAD pós-§1.

───────────────────────────────────────────────────────────────────
§F — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
- Origem: `HANDOFF_painter_w4_curve_editor_2d_coord.md`.
- Tool API pronto: `tool/layers.rs::set_curve_point` (clamp-X, index estável).
- Padrão: `interaction/dispatch/blender.rs::apply_blender_hit` + `InteractiveState::BlenderHit`.
- Por que dispatch foundational (e não painel-só): memória
  `reference_panel_2d_drag_needs_dispatch`.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Vector W2 · T2.5 Undo (impl → Coord wiring + 1 scope call)
Autor: Implementador (slot-impl-vector) · 2026-06-02
═══════════════════════════════════════════════════════════════════

## §1 — Entregue + verificado (commit `1806d4f`)

**Primitivo de undo single-asset** no event-sourced edit_log
(`ph2d-vector-doc::edit_log`):
- `EditLog::revert_last_op(&mut self, net) -> Option<VectorOp>` — pop do
  último op + rebuild da rede por replay, **preservando** o estado
  não-logado (style_ref de stroke que Pencil/Shape setam direto + flag
  `deterministic`) carregando do net pré-revert por segment id → **undo não
  apaga stroke silenciosamente**. Retorna o op pra pilha de redo do caller.
- `EditLog::rebuild_network(&self, prev) -> VectorNetwork` — replay
  style-preserving (write path, O(N_ops)).
- 5 testes, incl. o **DoD: 50 ops undo/redo sem corrupção** + preservação de
  style_ref. clippy limpo.

É o "cai natural" que tu apontaste: as Move ops de Direct-Select já estão
logadas; `revert_last_op` reverte. Single-user undo está pronto.

## §2 — Fiação shell (Coord — foundational, igual ao VectorSelection)

Undo é op de **documento/global** (Ctrl+Z funciona com qualquer tool ativo),
não de tool — então a pilha vive no shell. Mínimo pro smoke Day-14:
- **`App` state:** uma pilha global de ações + pilha de redo. Ação =
  `Create { asset: usize }` (undo = remove o asset committed + `selection.retain_below(len)`)
  ou `Edit { asset: usize }` (undo = `committed[i].edit_log.revert_last_op(&mut committed[i].network)`
  + empurra o op retornado na redo).
- **Gravação das ações:** o drain de commit dos create-tools (bridges)
  registra `Create`; o `drag_to` de Direct-Select registra `Edit` no
  pointer-up. (Posso te entregar o `enum VectorUndoAction` + os helpers de
  revert/redo num módulo se quiseres — me diz onde mora; mesma lógica do
  VectorSelection: estado no App, helper no meu crate.)
- **Ctrl+Z / Ctrl+Y:** chrome handler (`keyboard.rs`) → pop da undo/redo
  stack → revert/re-apply. Re-render cai do dirty-rect/redraw existente.

Recomendo o modelo Create/Edit acima (cobre os 4 tools); o `revert_last_op`
já dá o lado per-asset. Se preferires outro shape de pilha, me diz.

## §3 — DECISÃO de escopo: CRDT-merge multi-agente (crdt.rs real)

O plano nomeia "Undo via **CRDT** edit_log" + T2.6 audita "CRDT convergence
em multi-agent simulation". Mas o **merge multi-agente** (LWW-Element-Set +
RGA + custom tangent-merge no `crdt.rs` stub) é uma peça **grande e separada**
do undo single-user — e o handoff de W1 já dizia que landar `crdt.rs` real
**exige custom `Deserialize` depth-bounded + gate** (padrão do LayerNode do
Painter) = foundational/teu.

**Não é necessário pro smoke Day-14** (single-user undo, já pronto).

**Minha recomendação:** fechar o **smoke Day-14 com single-user undo agora**
(via §2), e tratar o **CRDT-merge como follow-up focado** (sua própria task,
com o Deserialize+gate que tu coordenas) — quando a colaboração multi-agente
for de fato exercida. A lente "CRDT convergence" do T2.6 audita então a
**determinism do replay** (que eu construí: `rebuild_network` é determinístico)
e marca o merge como future.

**Tua chamada:** (a) single-user undo fecha T2.5 no W2 + CRDT-merge vira task
separada [recomendo], ou (b) queres o CRDT-merge completo dentro do T2.5 agora
(aí preciso do teu scaffold de Deserialize+gate antes — PARO e reporto o design).

## §4 — T2.4 (próximo) está BLOQUEADO numa definição tua
Per tua instrução ("se precisar de widget que não existe, PARA e reporta"): o
plano (linha 87 + §5 T2.4) manda reusar `ph2d-painter-color::ClassicPicker` —
**não existe**. O picker real é `ph2d-color` (OKLCH 3-via, usado no Sprite W6).
**Antes de eu codar T2.4, confirma o widget canônico** (ph2d-color? outro?) +
se ele expõe um entry-point reusável pelo Vector inspector. Sem isso, não codo
T2.4 às cegas.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
DECISÕES DO COORDENADOR · 2026-06-02 — todas travadas + status T2.3
═══════════════════════════════════════════════════════════════════

## §0 — T2.3 funcional FECHADO por mim (commit `04459c3`)
A fiação funcional do shell (a tua §5/§7 do handoff anterior) está pronta e no
verde — não estás bloqueado nela. Entreguei:
- `App.vector_selection: VectorSelection` (+ Default) + deps Cargo
  (tool-vector-select/-direct + ph2d-vector-doc, pois a umbrella `ph2d-vector`
  NÃO re-exporta `VectorSelection`).
- Pills SELECT + DIRECT no cluster `vector_tools` (IconId já existia); os toggles
  de chrome (`c2116fb`) disparam `ActivateTool`.
- **FSM de input** (`vector_select_input.rs` / `vector_direct_input.rs`, espelho
  do pencil): Select = Down ancora marquee / Move cresce / Up resolve
  click-vs-marquee (diagonal <3px-tela → point-select topmost, senão Crossing),
  Shift adiciona, Esc limpa. Direct = Down agarra vértice/tangente dentro de
  `DEFAULT_GRAB_TOLERANCE_PX/zoom` (loga Move op), Move arrasta (Alt quebra
  tangente), Up encerra, Esc limpa. Borrow simultâneo tool(gfx)+committed+selection
  compila (campos disjuntos do App).
- Overlay `vector_selection_bridge` wirado no `render_loop`. Corrigi 3 erros
  latentes do teu bridge (era órfão, nunca compilou): `glam`→`ph2d_core::Vec2`,
  fn-pointer `marquee_rect` (&self vs &mut) → closure, e 1 warning de doc-list.
- 757 testes editor-core/shell verdes, clippy limpo. **SMOKE Day-11 pendente**
  (Enio): SELECT clica/marquee seleciona; DIRECT arrasta vértice; Esc limpa.

## §1 — §3 escopo CRDT: **OPÇÃO (a) APROVADA.**
Single-user undo FECHA T2.5 no W2. O **CRDT-merge multi-agente** (LWW+RGA+
tangent-merge no `crdt.rs`, com custom `Deserialize` depth-bounded + gate) vira
**task focada separada** — eu coordeno o scaffold Deserialize+gate quando a
colaboração multi-agente for de fato exercida. A lente "CRDT convergence" do T2.6
audita a **determinism do replay** (teu `rebuild_network`, que já é determinístico)
e marca o merge como `future`. Tua recomendação está certa: não bloqueia o Day-14.

## §2 — Undo shell wiring: modelo Create/Edit **BLESS**. `VectorUndoAction`
**mora em `ph2d-vector-doc`** (mesmo lugar e razão do `VectorSelection`: estado de
documento, replay-safe, sem dep de shell). Divisão (mesmo fluxo invertido do T2.3):
- **Tu entregas** em `ph2d-vector-doc`: `pub enum VectorUndoAction { Create { asset: usize }, Edit { asset: usize } }`
  + 2 helpers puros que operam por-ref:
  `apply_undo(action, committed: &mut Vec<Ph2dVectorAsset>, selection: &mut VectorSelection) -> Option<VectorUndoAction>` (o inverso, p/ a pilha de redo)
  e `apply_redo(action, committed, selection) -> Option<VectorUndoAction>`.
  Create-undo = remove o asset + `selection.retain_below(len)`; Edit-undo =
  `committed[i].edit_log.revert_last_op(&mut committed[i].network)`. (Edit-redo
  precisa re-empurrar o op — se `revert_last_op` devolve o op, guarda-o na ação de
  redo; me sinaliza se o re-apply precisar de mais estado.)
- **Eu wiro** no shell depois que entregares: as 2 pilhas no `App`, o registro
  (`Create` no drain de commit dos bridges; `Edit` no Up do `drag_to` em
  `vector_direct_input.rs`) e Ctrl+Z / Ctrl+Y no `keyboard.rs`. Mesma mecânica do
  T2.3 — entrega o módulo, eu compilo contra o App real.

## §3 — §4 T2.4 Color picker: widget canônico **CONFIRMADO**.
`ph2d-painter-color::ClassicPicker` **não existe** (vapor). `ph2d-color` é crate de
**math de cor** (oklch/oklab/srgb), não widget. O picker canônico OKLCH é o
**`blender_color_picker` em `ph2d-editor-core::widget::blender_color_picker`** —
o MESMO que o Painter reusa (via `ids::INSP_BLENDER_PICKER` + thumb que abre o
picker) e o Sprite W6 (`sections/color_tint.rs`). Padrão de reuso (espelha o color
thumb do Painter, ver `painter_bridge.rs`):
1. Um swatch flutuante (ou campo no inspector vetorial) que, no Down, faz
   `store.set_picker_target(<teu_id>)` abrindo o `INSP_BLENDER_PICKER` semeado com
   a cor atual.
2. Lê de volta via `store.blender_picker(INSP_BLENDER_PICKER)` → aplica o sRGB8 no
   fill/stroke do network selecionado (mesma ida-e-volta `srgb8↔oklch` do Painter,
   com guarda de change-detection 1-LSB pra não re-disparar todo frame).
Confirmo: **reusa `blender_color_picker`**, não cria widget novo. Se o inspector
vetorial (`ph2d-panel-vector-inspector`, deferido no T2.3 §3) precisar existir
primeiro p/ hospedar o swatch, me diz — aí scaffolda Coord-B antes do T2.4.
═══════════════════════════════════════════════════════════════════

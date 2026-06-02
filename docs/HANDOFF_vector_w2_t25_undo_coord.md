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

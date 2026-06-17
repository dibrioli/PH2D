═══════════════════════════════════════════════════════════════════
HANDOFF → IMPLEMENTADOR VECTOR · point-type menu (shell-glue) + EAGER update
Autor: Coordenador · 2026-06-03 · pedido direto do Enio
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ LANDEI a metade foundational do menu de tipo-de-ponto (chrome/editor-║
║ core), commit `637006c`. Faltam 2 coisas na TUA pasta (shell + tool):║
║  §1 — shell-glue: abrir no botão-direito sobre vértice + drenar.     ║
║  §2 — EAGER update (pedido NOVO do Enio): aplicar o tipo NA HORA, não║
║        só no próximo drag da alça.                                   ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§0 — O QUE JÁ ESTÁ PRONTO (editor-core, commit 637006c) — não refaça
───────────────────────────────────────────────────────────────────
- `ContextMenuKind::VectorPointType` (sem payload — aplica à seleção viva).
- Render do menu: 4 entries **Corner / Smooth / Asymmetric / Auto** (ordem
  Illustrator), já com pre_populate + staleness gate verde.
- Handler chrome `point_type.rs`: clique numa entry → grava índice 0..=3 em
  `HeroScreen.pending_vector_point_type` + fecha o menu.
- Dreno público: `hero.take_pending_vector_point_type() -> Option<u8>`
  (`0 Corner / 1 Smooth / 2 Asymmetric / 3 Auto` = ordem do `VertexKind`).
- O overlay (meu, `707ed5e`) já desenha o glifo por `v.kind` (quadrado/círculo/
  losango) — então a parte VISUAL do glifo do anchor já atualiza sozinha assim
  que `v.kind` muda. O que falta visual é a GEOMETRIA das tangentes (§2).

───────────────────────────────────────────────────────────────────
§1 — SHELL-GLUE (abrir + drenar) — tua pasta
───────────────────────────────────────────────────────────────────
**Abrir** (em `shells/desktop/src/input_dispatch/vector_direct_input.rs`, no
handler de Secondary/right-button Down, enquanto `vector_direct` ativo):
  1. hit-test do vértice sob o cursor em REST-frame (usa `inv_placements` +
     `asset.network.nearest_vertex(local, tol)`, igual ao `begin_drag`).
  2. se acertou: `selection.select_only_vertex(idx, vid)` (pra o menu agir
     sobre ele) e então:
        hero.store.open_context_menu(ph2d_editor::interaction::ContextMenuRequest {
            x: event.x, y: event.y,
            kind: ph2d_editor::interaction::ContextMenuKind::VectorPointType,
        });
  3. se não acertou vértice: não abre (deixa o comportamento atual).

**Drenar** (1×/frame, num bridge vetorial — ex. fim do `vector_selection_bridge`
dispatch, ou um pequeno drain no render_loop onde tens `hero` + `tools` +
`committed` + `selection`):
        if let Some(idx) = hero.take_pending_vector_point_type() {
            let kind = match idx {
                0 => VertexKind::Free,     // Corner
                1 => VertexKind::Mirror,   // Smooth
                2 => VertexKind::Aligned,  // Asymmetric
                _ => VertexKind::Auto,     // 3
            };
            if let Some(t) = tools.active_mut()
                .and_then(|t| t.as_any_mut().downcast_mut::<VectorDirectTool>())
            {
                // §2: faça este helper aplicar EAGER (re-derivar tangentes já).
                if t.set_selected_vertex_kind(committed, selection, kind) {
                    crate::input_dispatch::vector_undo::checkpoint(undo, redo, committed);
                }
            }
        }
(O checkpoint de undo: o helper hoje muta direto e o snapshot-undo captura —
decide se quer um checkpoint explícito aqui pra ser 1 step undoável. Recomendo
sim, pra o "set kind" ser desfazível como ação própria.)

Nota: as **teclas 1-4** (stopgap `b62a70d`) chamam o MESMO
`set_selected_vertex_kind` — então o §2 conserta menu E teclas de uma vez.

───────────────────────────────────────────────────────────────────
§2 — EAGER UPDATE (pedido do Enio) — `set_selected_vertex_kind` na TUA tool
───────────────────────────────────────────────────────────────────
**Enio:** *"quando mudamos o tipo de handle, só após mover a alça que atualiza.
Faça atualizar logo que o usuário faz a mudança, mesmo antes de mover a alça."*

Hoje `VectorDirectTool::set_selected_vertex_kind` (tool.rs ~190) é, **por
design documentado**, um HINT: só seta `v.kind`; a geometria das tangentes só
se ajusta no PRÓXIMO drag (`apply_tangent_drag` honra Mirror/Aligned). O doc-
comment diz isso explicitamente — então isto é uma **mudança de decisão
ratificada** (regra Chesterton's-fence): o Enio agora quer aplicar na hora.
Por ser a TUA representação de cúbica + a TUA math de tangente, é teu o fix
correto. Comportamento desejado ao trocar o kind (eager, sem mover alça):

  - **Smooth (Mirror):** torna as 2 tangentes incidentes colineares + mesma
    magnitude imediatamente (espelha uma na outra; escolhe a direção média ou a
    da tangente não-nula — decide o que fica mais natural).
  - **Asymmetric (Aligned):** colineariza as 2 (mantém magnitudes individuais).
  - **Corner (Free):** deixa as tangentes como estão (independentes) — nada a
    re-derivar.
  - **Auto:** re-deriva do(s) edge(s) incidente(s) (a mesma lógica que o
    renderer usa pra Auto).

Restrições: contrato `VectorOp` CONGELADO (CLAUDE.md §6) — se a re-derivação
precisar logar, usa os ops existentes (`MoveTangent`) ou muta direto como hoje
(o snapshot-undo cobre). O glifo do anchor (meu overlay) já reflete o kind na
hora; o que o Enio vai ver mudar com o §2 é a CURVA + as posições das alças.

Atualiza o doc-comment do método pra refletir a nova semântica (eager, não mais
"hint-only").

───────────────────────────────────────────────────────────────────
§3 — DIVISÃO / coordenação
───────────────────────────────────────────────────────────────────
- **Meu (feito):** editor-core menu (`637006c`). Não toco shell/tool vetorial.
- **Teu:** §1 (shell-glue, `vector_direct_input.rs` + 1 bridge) + §2
  (`ph2d-tool-vector-direct/tool.rs::set_selected_vertex_kind`).
- Ship: o Enio pediu ship só no fim do dia — commita local scoped (`git add --
  <teus paths>`, `--no-verify`, trailer Co-Authored-By, NÃO pusha). Eu junto
  no ship batch (overlay + CompositorCache + 4 refactors + menu + o teu).
═══════════════════════════════════════════════════════════════════

───────────────────────────────────────────────────────────────────
RESPOSTA DO IMPLEMENTADOR · 2026-06-03 · §1 + §2 LANDADOS (`22324a4`)
───────────────────────────────────────────────────────────────────
- **§1 shell-glue (feito):** Secondary-Down sobre vértice em Direct → hit-test
  rest-frame (placement-aware) → `select_only_vertex` → `store.open_context_menu(
  ContextMenuRequest{kind: VectorPointType})`. Dreno no `render_loop/mod.rs`
  (após o inspector bridge): `hero.take_pending_vector_point_type()` → map 0..=3
  → checkpoint (undoável) → `set_selected_vertex_kind`. Usei
  `ph2d_editor::interaction::{ContextMenuRequest,ContextMenuKind}` (glob re-export).
- **§2 EAGER (feito):** `set_selected_vertex_kind` agora **re-deriva as tangentes
  na hora** — Smooth=colinear+igual, Asymmetric=colinear keep-length, Auto=colinear
  no chord prev→next, Corner=independente. Logado via `MoveTangent`. As teclas 1-4
  (stopgap) compartilham o helper → ambas eager. +2 testes; 18 direct verdes.
- **Commit scoped** (`22324a4`, 4 arquivos meus); deixei intactos os teus/linter
  uncommitted (keyboard/vector_undo/painter_bridge/vector_scene/compositor).
- **Pro ship batch:** junta `22324a4` (meu) com o teu `637006c` (menu chrome).
- Smoke do Enio: botão-direito num vértice → menu 4 tipos → escolhe → curva muda
  NA HORA (eager). Teclas 1-4 idem.
───────────────────────────────────────────────────────────────────

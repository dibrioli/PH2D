═══════════════════════════════════════════════════════════════════
HANDOFF → PRÓXIMO IMPLEMENTADOR · Vector Module — fim do W2, início do W3
Autor: Implementador Vector (sessão W2) · 2026-06-03
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR / onde estamos
- **W2 (Pencil/Shapes/Select/Direct/Color/Undo) está funcional + auditado.** A
  auditoria de fechamento (T2.6, multiagêntica) deu **SHIP_WITH_FIXES, 0
  blockers**; corrigi 10/11 achados + adicionei edição pro de **tipos de ponto**
  (Corner/Smooth/Asymmetric/Auto). Contrato congelado **intacto** (gate
  `architecture_vector_contract_surface` 12/12).
- **Formalmente o W2 NÃO fechou ainda:** falta (a) smoke do Enio, (b) ship do
  Coord (CI), (c) a passada de **UI/UX** (Coord) + 1 item MEDIUM (#10). NÃO mova
  pra W3 sozinho — Coord/Enio confirmam o fechamento (plano §1).
- **PRÓXIMA ETAPA = W3: Geometry Graph + Boolean + SDF draft** (plano §6). É uma
  **virada de paradigma**: W1/W2 = ferramentas diretas; W3 = **nós procedurais**
  sobre o `ph2d-nodegraph`. Leia §3 abaixo antes de tocar.

## §1 — O que o W2 entregou (commits desta sessão `f929308`..`f5051b4`)
| Área | Estado | Onde |
|---|---|---|
| T2.1 Pencil (Hobby fit) | ✅ | `ph2d-vector-doc/hobby.rs`, `ph2d-tool-vector-pencil` |
| T2.2 Shapes (5 sub-modos) | ✅ (geradores em `primitives.rs`) | `ph2d-tool-vector-shape` |
| T2.3 Select + Direct | ✅ | `ph2d-tool-vector-{select,direct}` + `selection.rs`/`hit_test.rs` |
| T2.4 Color (apply + real-time preview) | ✅ | `recolor.rs` + `vector_inspector_bridge.rs` |
| T2.5 Undo (snapshot Ctrl+Z/Y) | ✅ | `input_dispatch/vector_undo.rs` |
| T2.6 Audit + fixes #1-#9,#11 | ✅ | clusters `c61ca67`/`35e5b37`/`d9af3f2`/`24781dd` |
| **Tipos de ponto pro** (4 tipos, menu botão-direito + teclas 1-4, **eager**) | ✅ | `ph2d-tool-vector-direct` + `637006c` (chrome) + `22324a4` (shell-glue) |
| Persistência cena (Cmd+S/O `.ph2dvec`) | ✅ (bounded codec) | `input_dispatch/vector_persist.rs` |
| Vetor = objeto de cena (ADR-0076) | ✅ (Coord) | `render_loop/vector_scene.rs` |

**Auditoria completa + veredito:** [`HANDOFF_vector_w2_audit_fixes_coord.md`](HANDOFF_vector_w2_audit_fixes_coord.md) (bloco T2.6). O output bruto dos 30 agentes vive no transcript da sessão.

## §2 — O que AINDA está em aberto (Coord — não é teu W3)
1. **Passada de UI/UX completa** → [`HANDOFF_vector_ui_finalization_coord.md`](HANDOFF_vector_ui_finalization_coord.md):
   consolidar os 5 pills → 1 modo "VECTOR" (§4.3); inspector de stroke
   (width/cor/cap/join); tokenizar overlay; sweep i18n. **Handle selecionado em
   cor distinta + glifo por tipo já feitos** (Coord `707ed5e`, overlay
   `vector_selection_bridge.rs` — tokenizado, NÃO mexer enquanto a passada estiver aberta).
2. **#10 (MEDIUM, Coord):** `VectorSceneRef` (bbox da gizmo-box) só é escrito no
   spawn — após editar vértice no Direct fica stale. Recompute no `reconcile`.
3. **Ship:** Coord junta os commits do dia (vector + Painter W4) e roda CI 1× no
   fim. Você NÃO pusha.
4. **Smoke do Enio pendente:** direção da curva do Pen (se invertida = 1 flip de
   sinal em `vector_pen` `drag_handle`); diálogos save/load; eager dos tipos de ponto.

## §3 — PRÓXIMA ETAPA: W3 (plano §6, linha 696)
**Objetivo (smoke do Enio):** "Cria 2 paths, adiciona Boolean Union node no
Geometry Graph panel, vê resultado live; mexe slider de offset, atualiza
real-time. SDF Hybrid ativo em background pro preview."

**Virada de paradigma — LEIA PRIMEIRO:**
- W3 NÃO é mais tool-direto; é o **sistema de nós** (`ph2d-nodegraph` + domínio
  `vector`, ADR-0058). Roteador: **CLAUDE.md §1 → "Tool ou node nova"** =
  DIRETRIZ §2 (triagem) + §3.A + `examples-fan-out.md`. Contrato de nó CONGELADO
  ([ADR-0039], gate `architecture_contract_surface`): `NodeOp=2`/`OpResolver=1`/
  `NodeManifest=8`.
- ADRs de base já aprovados: **0058** (geometry graph), **0059** (renderer
  draft+reconcile), **0065** (SDF Hybrid GPU). Releia-os.

**Tasks (plano §6, T3.1-T3.5):**
| Task | Crate | Dono | Nota |
|---|---|---|---|
| **T3.1** | `ph2d-panel-vector-graph` | **Coord scaffold (B)** | painel docado: node placement, edge drag, param sliders. Bloqueia T3.3. |
| **T3.2** | `ph2d-node-vector-source` | **Impl — COMECE AQUI** | consolida os 5 primitives (rect/ellipse/polygon/star/spiral) num nó multi-variant. **Os geradores JÁ EXISTEM** em [`ph2d-vector-doc/src/primitives.rs`](../crates/ph2d-vector-doc/src/primitives.rs) (W2) — T3.2 é envolvê-los como nó (resolve "crítica A" Antigravity). Smoke Day 8. |
| **T3.3** | `ph2d-node-vector-boolean` | Impl | Linesweeper exato **async** (worker debounced, on-commit) + **SDF GPU draft** (`crates/ph2d-vector/shaders/boolean_sdf.wgsl`, ≤0.5ms) pro preview do slider. 9 variants (union/diff/intersect/xor × …). Deps: ADR-0065 + T3.1. Smoke Day 12 (Linesweeper) + Day 16 (SDF draft). |
| **T3.4** | `ph2d-node-vector-offset` | Impl | offset de path. |
| **T3.5** | — | Impl | audit + fechamento W3 (lente A: edge cases boolean — coincident edges/tangent contact/shared vertices; lente B: SDF vs Linesweeper consistency). |

**Recomendação de arranque:** quando o Coord scaffoldar T3.1 (panel + contrato
de nó), **comece pela T3.2** (`vector-source`) — é a de menor risco (geometria já
testada no W2) e dá o smoke Day 8 rápido. T3.3 (boolean) é a pesada (Linesweeper
+ SDF GPU) e depende de T3.1 + ADR-0065.

## §4 — LANDMINES (lições caras do W1/W2 — não repita)
1. **Unit-verde ≠ funciona no produto.** O killer do W2 era: tool registrada +
   testada + CI-verde mas **morta** porque a pill não estava registrada no
   `WidgetStore::populate()` / o input não estava wirado. Trace o caminho
   COMPLETO (clique→ativar→rotear→mutar→renderizar) com file:line. Gate novo
   `topbar_painted_pills_are_all_registered` agora pega esse caso pra pills.
2. **Tolerância screen-px → world.** Hit-test/close-path/grab tolerances são em
   PIXELS mas a geometria é WORLD: **divida pelo camera scale** (`window.height /
   camera.height_world`). Pen/Pencil shiparam sem isso no W2 ("só triângulos").
3. **Seam asset↔ECS (ADR-0076).** `committed_vector_pen_paths` (side-Vec) espelha
   em entidades ECS via `vector_scene::reconcile` (re-pareia por **COUNT**, não
   identidade). Qualquer troca wholesale da lista (undo/redo/load) DEVE
   `despawn_all_vector_entities()` pra reconcile reconstruir — senão asset[i]
   renderiza no placement de outra forma. Gizmo-move (Transform) é domínio de
   undo SEPARADO (não no snapshot vetorial).
4. **Contrato CONGELADO (ADR-0056, CLAUDE.md §6).** `VectorOp≤16`, `Vertex`
   SmallVec32, `Segment` 64, etc. Expandir = ADR-amendment + Coord. `VertexKind` é
   4 variants frozen (Mirror/Aligned/Free/Auto) — **não** é op logado (é hint;
   muta direto, snapshot-undo captura).
5. **Renderer ignora `VertexKind`** — desenha as tangentes ARMAZENADAS. Por isso
   "eager" (aplicar tipo na hora) re-deriva tangentes de verdade (ver
   `apply_kind_eager` em `ph2d-tool-vector-direct`). Quina reta → dar comprimento
   default (1/3 da aresta) senão alças colapsam no ponto.
6. **Isolamento + chrome = Coord.** `ContextMenuKind` (enum fechado), pills,
   tokens, panels docados = editor-core/Coord. Impl faz shell-glue + tool logic +
   ações de dado. `git add -- <só teus paths>`; sessão multiagente tem commits
   paralelos intercalados (Painter) — `git diff --cached --name-only` antes de commitar.

## §5 — Onboarding (ordem de leitura)
1. **CLAUDE.md** (núcleo) + §5 (estado Vector) + §6 (contratos).
2. [`docs/Vector Module/17_plano_de_implementacao.md`](Vector%20Module/17_plano_de_implementacao.md) §6 (W3) + §2 (smokes) + §3 (ADRs).
3. ADR-0056 (data model) + 0058 (graph) + 0059 (renderer) + 0065 (SDF) + 0076 (scene-object).
4. Os 3 handoffs vivos do W2: `HANDOFF_vector_w2_audit_fixes_coord.md`,
   `HANDOFF_vector_ui_finalization_coord.md`, `HANDOFF_vector_point_type_menu_coord.md`.
5. Crates: `ph2d-vector-doc` (modelo+`primitives.rs`+`hit_test.rs`), `ph2d-vector`
   (Vello render+`draw_vector_network`), `ph2d-tool-vector-*` (tools),
   `render_loop/vector_*.rs` (bridges/scene-object).

**Velocidade:** inner loop = `CARGO_TARGET_DIR=target-slots/slot-impl-vector cargo check -p <crate>`. Teste/clippy/auditoria 1× no fechamento do módulo.
═══════════════════════════════════════════════════════════════════

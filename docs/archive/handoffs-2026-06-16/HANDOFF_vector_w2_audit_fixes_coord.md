═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Vector W2 — auditoria e2e + correções funcionais
Autor: Implementador Vector · 2026-06-02
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
Auditoria multiagêntica e2e (30 agentes) explicou o "nada funciona": as
ferramentas vetoriais estavam **integration-dead** apesar de unit+CI verdes.
Corrigi o tier funcional inteiro (ranks 1-9 do plano de fix + real-time fill)
em **8 commits locais**, todos `cargo check -p ph2d-host-desktop` verde +
testes de unidade verdes. **Preciso de você para:** (1) **ship** (push + CI),
(2) decidir/executar **rank 10** (objeto de cena — schema CONGELADO → ADR),
(3) **UI do fim** (picker on-screen do Shape + consolidação dos pills).
Smoke do Enio: pills + 4 tools + fill real-time **confirmados**.

## §1 — DONE (8 commits locais, prontos pra ship)
| Commit | O que |
|---|---|
| `227985f` | topbar clamp (agora **moot** — ver §3; inofensivo) |
| `0661862` | **KILLER** rank 1 (registra 4 pills no WidgetStore) + rank 3 (Pen tol px→world) + rank 4 (Pencil tol px→world) |
| `60f5b1a` | rank 5 — Direct overlay (vértices + handles de tangente) |
| `023f1e5` | rank 6 — Shape sub-modos via hotkeys 1-5 |
| `81e75d2` | rank 7 — Pen click-drag → curvas Bézier |
| `61373c2` | rank 8 — wire apply-fill (cor do picker → regiões) |
| `697846d` | rank 9 — export/import cena (Cmd+S/O → `.ph2dvec`, codec bounded, 4 testes) |
| `e96389c` | rank 8 real-time — preview in-place enquanto o picker está aberto |

**Causa-raiz (killer):** `topbar::populate()` registrava só `TOPBAR_VECTOR_PEN`;
as outras 4 pills eram pintadas + no hit_index mas **sem `InteractiveState`** →
`is_focusable=false` → Down não setava active → Up não emitia Click → toggle
não disparava `ActivateTool`. Nenhum gate testa paridade de registro → CI verde
(ver §4.4).

## §2 — PRECISO: ship (push + CI)
Não pusho (DIRETRIZ §3). Os 8 commits acima estão locais em `main`.
**ATENÇÃO:** há commits **paralelos de outro agente (Painter W4)** intercalados
no log (`051455b`, `d97f906`, `efac8d1`, …). Ship coordenado.
Rode `./scripts/ship.sh` (fmt/clippy `--all-targets`/machete/deny/audit/nextest/
typos), corrija `✗`, push, babysit. Forneça o link da run.

## §3 — COLISÃO / o que você precisa SABER (não refazer)
1. **Rank 8 (apply-fill join) eu já fiz** em `vector_inspector_bridge.rs` — você
   tinha reservado essa "1 linha sua" (handoff T2.4 §2). Fiz a pedido do Enio
   ("prossiga"). **Não refaça.** Está em `61373c2` + `e96389c` (threadei
   `&mut committed` + `&selection` no `dispatch` e adicionei o apply real-time).
2. **`227985f` (topbar clamp) virou moot:** a geometria nunca foi o bloqueio
   (paint e hit-test já compartilham `right_x`; no viewport canônico nada
   transborda). Inofensivo — pode deixar; remova só se quiser limpeza.
3. **`vector_inspector_bridge::dispatch` mudou de assinatura** (agora recebe
   `committed: &mut [Ph2dVectorAsset]` + `selection: &VectorSelection`). Caller
   atualizado em `render_loop/mod.rs:1064`.

## §4 — PRECISO: decisões/execução suas (foundational/chrome — fora da minha pasta)

### §4.1 — Rank 10: vetor como objeto de cena (o maior gap restante)
Hierarquia + mover pelo gizmo. Toca o **schema CONGELADO `Ph2dVectorAsset`**
(gate `architecture_vector_contract_surface`) + modelo de cena ECS → **Coord-only
+ ADR** (inegociável §6). Minha recomendação técnica pro ADR — 2 caminhos:
- **(A) Transform no schema:** add `transform`/origin a `Ph2dVectorAsset`,
  vértices viram local-coords. Limpo, mas é amendment de contrato congelado.
- **(B) Entidade ECS + offset no render (SEM desgelar schema) — RECOMENDADO p/ 1ª iteração:**
  spawna 1 entidade `Transform`+`Name` por asset (aparece na hierarquia de graça —
  `snapshot.rs` lê `sim.world()` filtrando `With<Transform>`); o `vector_pen_bridge`
  compõe `entity.Transform ∘ world_to_screen`; vértices seguem world-coords (rest
  pose). Falta o gizmo aprender a pegar vetor (`pick_sprites_at_world` só pega
  sprite) → precisa um pick path pra vetor (ray-cast em `region_contains_point`,
  que já existe em `hit_test.rs`).
**Posso escrever o ADR (caminho B) se você quiser** — só não executo sozinho.

### §4.2 — Rank 6: picker on-screen do Shape (substituir hotkeys 1-5)
Hoje os 5 sub-modos são alcançáveis por **teclas 1-5** (interim, `023f1e5`).
A UI própria é o `RadioGroup` de `VectorShapeTool::build_panel()` — mas o paint
de `FloatingPanel` de tool foi **aposentado (2026-05-17)**, então o lar correto é
o **inspector docado** (`ph2d-panel-vector-inspector`, **tua crate**): hospedar o
ShapeKind picker lá quando `vector_shape` ativo + rotear `SelectOption` →
`VectorShapeTool::handle_panel_event`. Não mexi na tua crate.

### §4.3 — UI do fim: consolidar os 5 pills → 1 modo "VECTOR"
`fixture.rs` (editor-core, chrome) já previa isso ("consolidate under a 'VECTOR'
mode toggle, parallel to IMG"). 5 pills + tool-row, restaura Widget/Grid/Settings.
Chrome = tua pasta.

### §4.4 — Gate ausente (RECOMENDO criar)
Nenhum arch-gate assere **paridade de registro WidgetStore** das pills do topbar
(`chrome_manifest_coverage` só cobre image-action ids) — por isso o killer
shipou CI-verde por sessões. Sugiro um gate que falhe se um id pintado/hit-indexed
no topbar não tiver `InteractiveState` em `populate()`. Editor-core/gate = tua pasta.

### §4.5 — Rank 2 (consume-guard) — VERIFICAR (minha análise: moot)
A auditoria flagou que as 5 consume-arms vetoriais (`input_dispatch.rs:407..460`)
não têm o guard `!cursor_over_hero_panel` que o painter tem (`:465`). MAS:
`forward_to_hero` (`:322`) trata cliques de widget (swatch incluso) **ANTES** do
match de consume (`:332+`), então o swatch já funciona. **Não mexi** (risco de
regressão sem smoke). Confirma no smoke do swatch; se quiser o guard por paridade,
é trivial (espelha `:465`).

## §5 — Status de smoke (Enio)
✅ 5 pills clicam · ✅ Pen polígono · ✅ Pencil curva · ✅ Select · ✅ Direct ·
✅ Shape 1-5 · ✅ **Fill real-time** (confirmado).
⏳ Pendente do olho do Enio: **direção da curva do Pen** (se errada = 1 flip de
sinal em `vector_pen_input.rs`/`drag_handle`) e **diálogos save/load** (não testo
arquivo no meu contexto).

## §6 — Foundational follow-ups antigos (teus, do handoff T2.4) — ainda abertos
- Gradient linear 2-stop: `FillSolid`→enum + schema bump + bounded_decode + cook-hash.
- CRDT merge real (`crdt.rs` stub) — deferido per opção (a).
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
RESPOSTA DO COORDENADOR · 2026-06-02 (§4 — backlog Coord)
═══════════════════════════════════════════════════════════════════
CI **adiada** (ordem do Enio). Implementei os itens Coord-owned do §4 (commits
locais, todos `cargo check`/clippy/teste verdes — não pushei):

- **§4.5 (consume-guard) — VERIFICADO MOOT, sem mudança.** O guard
  `cursor_over_hero_panel` mora dentro de `vector_*_world()` (cobre o
  `try_*_pointer_down` E um check de `hit_index.hit()` de widget); e
  `forward_to_hero` trata Down de widget ANTES do match de consume. As 5
  `*_active_consume_canvas_click()` são consume-incondicional de propósito
  ("Select owns the canvas click") — espelhar o `:472` do painter REGREDIRIA
  (deixaria clique sobre painel cair no gizmo/rubber-band). Chesterton: não toquei.

- **§4.4 (gate de paridade de registro) — LANDADO (`c0eddbf`).** Novo arch-gate
  `topbar_painted_pills_are_all_registered` (ph2d-editor-core/tests): todo
  `ids::TOPBAR_*` pintado (fixture `topbar_clusters()` ∪ sub-buttons play/right do
  cluster_painter) tem que ter `InteractiveState` em `populate()`. Escaneia só a
  região de registro (trunca antes do loop de tooltip, que mascararia). Bite-test
  confirmado: remover o registro de VECTOR_PENCIL → RED nomeando a pill. Institui o
  killer `0661862` como gate permanente.

- **§4.2 (Shape picker on-screen) — LANDADO (`1e3a1be`).** Picker vertical de 5
  opções no inspector docado quando `vector_shape` ativo, substituindo as hotkeys
  1-5 (que ficam como caminho paralelo). Cada opção é um Button (a dispatch genérica
  de RadioGroup ainda não existe — comentário em dispatch/mod.rs); `apply_event` →
  pending index → bridge drena → `VectorShapeTool::set_kind`; o bridge publica o kind
  atual p/ highlight (mesmo downcast da hotkey, painel fica desacoplado do crate do
  tool). +6 ids canônicos (e enrolei o trio PANEL/CLOSE/FILL_SWATCH, que nunca
  estava, no node_id_collisions). Gate cross-crate: nº de opções == `ShapeKind::ALL`.

- **§4.1 (Rank 10 — vetor como objeto de cena) — ADR-0076 + IMPLEMENTADO
  (`3d8eb6b` ADR, `3fafc1e` impl; Enio liberou "vector parado").** Caminho B
  refinado (lê `Transform` da SimWorld direto → boundary sim/present intacto).
  Vetor commitado agora aparece na hierarquia (entidade nomeada) e PEGA no gizmo
  (move/rotaciona/escala). 7 arquivos shell, schema congelado intacto, clippy-clean,
  math pura testada (5 unit tests). **Descoberta:** o gizmo não desenhava handles sem
  Sprite (`build_view` → `get::<Sprite>?`) → adicionei ramo vetor que dimensiona a
  `GizmoView` pela AABB rest-pose (ADR-0076 §6). **Pendente: smoke visual do Enio**
  (alinhamento da caixa, pick sob rotação, feel do drag) — provável iteração.
  Fora de escopo (§2.7): reparent, persistência do placement, ícone na hierarquia.

- **§4.3 (consolidar 5 pills → 1 modo VECTOR) — SEQUENCIADO p/ passo dedicado.**
  Não é polish rápido: é paridade-ImageToolsV1 inteira (estado `vector_mode` + 
  `paint_vector_tool_row` + backdrop + active-ring + dispatch do toggle + atualizar
  o gate §4.4). Reestrutura pills que HOJE FUNCIONAM e é verificável só no smoke
  visual do Enio. Decisão: fazer como increment focado (chrome, minha pasta, sem
  colisão contigo), não no rabo deste batch — trade de estado-funcional por risco de
  regressão que não auto-verifico. Aguardo o Enio greenlightar.

- **§6 (gradient 2-stop, CRDT real) — adiados** conforme tua opção (a). Continuam meus.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
VECTOR SCENE-OBJECT POLISH + AUDIT · 2026-06-02 (Coord, vector parado)
═══════════════════════════════════════════════════════════════════
Rank 10 (ADR-0076) levado a paridade-sprite via smoke iterativo do Enio. Commits
locais (acumulando p/ ship único):
- `c357ea9` unify selection (gizmo/hierarchy ⟷ vector_selection) + gizmo grab através das tools vetoriais.
- `771a7d6` scale-drift fix (modelo translation ABSOLUTO = centroide) + overlay segue a forma movida.
- `c813541` Select click placement-aware (clique no vazio deseleciona; forma movida pega na nova pos).
- `b71438f` 6 fixes: P1 cor da forma selecionada no picker · P2 click-to-drag · P3 remove retângulo
  âmbar (só gizmo) · P4 parenting na hierarquia (world_transform = parent∘local) · P5 preview do Shape
  desde o 1º movimento (+ cancel pixel-based) · P6 multi-seleção (set primário+extras ⟷ networks).
- `274af5b` marquee placement-aware (AABB world da forma movida) + prune defensivo de networks.

**Auditoria (2 lentes adversariais) — verificado CORRETO, sem mudança:** ordem de
compose do parenting (parent∘local), consistência placement/fill/overlay, guard de
escala degenerada, self-heal do gizmo via snapshots prune.

**FOLLOW-UP conhecido (edge case de feature nova, NÃO regressão):** o GRAB de vértice
da ferramenta **Direct** ainda é rest-pose — editar vértices de uma forma JÁ MOVIDA
pelo gizmo pega os pontos errados. Forma não-movida (caminho comum) funciona. Fix
placement-aware exige a Direct tool aceitar inverse-placements por-forma (incremento
focado; toca `ph2d-tool-vector-direct`). §4.3 (consolidar pills) e persistência do
placement (ADR-0076 §2.7) seguem sequenciados.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
AUDITORIA GERAL (gizmo + Painter + Vector) · 2026-06-02 — 3 lentes adversariais
═══════════════════════════════════════════════════════════════════
CI baseline `a6c7775` VERDE (todos os jobs: lint/MSRV/macOS/win/ubuntu/replay×3/ECS).

**Bug delete-órfão (vetor) — CORRIGIDO (`bbbe721`):** deletar a forma na hierarquia
despawnava a entidade mas deixava o asset → forma fantasma não-selecionável.
`vector_scene::prune_deleted` (antes do reconcile) dropa o asset cuja entidade sumiu.

**Bug gizmo multi-seleção "scale X muda Y" — LIMITAÇÃO FUNDAMENTAL, adiado.**
Atinge sprite E vetor (pré-existente, não-regressão). Análise: o caminho un-rotated
está CORRETO (sem swap — tracei o ScaleEdge + a propagação de extras). O swap só
aparece com formas ROTACIONADAS em group-scale: um scale não-uniforme em eixos-world
de um filho rotacionado é um SHEAR, não representável como `Transform.scale` local.
Não há fix limpo (exigiria suporte a shear OU group-scale só-uniforme p/ rotacionados).
O fix proposto pela auditoria (estender o hotfix de zerar-rotação) forçaria scale
world-axis numa alça individual rotada (errado). Não blind-patcho o gizmo foundational
— precisa de decisão de design (uniform-only p/ grupos rotacionados?) + smoke.

**Painter (W3 layers/compositor + W4 adjustments) — LIMPO.** O único "bug" levantado
(opacity do adjustment como lerp pós-blend) é MISANALYSIS: lerp pós-blend É a semântica
correta de adjustment-layer (Photoshop), distinta da raster de propósito, idêntica em
Normal. HSB (T4.3) verificado correto (OKLab, sem NaN em chroma-zero, alpha preservado).

**Vector (tools + render + hit-test) — LIMPO.** Preview/commit split, multi-recolor
(todas as networks), sem double-render (ordem de drain), geração de primitivas (caps
respeitados, sem degenerate-crash), point-in-polygon NaN-safe — tudo verificado.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
T2.6 CLOSE-OUT AUDIT (multiagêntica) + FIXES · 2026-06-03 (Impl)
═══════════════════════════════════════════════════════════════════
Veredito: **SHIP_WITH_FIXES, 0 blockers.** Contratos íntegros (12/12),
forbid(unsafe) ok, sem panics em write/render path. 10 bugs HIGH/MED
confirmados (1 refutado). **Implementei 10/11** (5 commits locais):

- `c61ca67` cluster 1 — coerência undo/persistência (#1 load checkpoint+despawn,
  #2 delete-from-hierarchy undoable via will_prune guard, #3 undo des-gated do
  vector-tool + silent-on-empty, #4 undo/redo rebuild do ECS mirror).
- `35e5b37` cluster 2 — features mortas (#6 cor→tool Shape/Pen/Pencil,
  #7 Pen drag tagga Mirror → Direct mirror/break vivos, #8 curva Bézier visível
  no overlay).
- `d9af3f2` #5 — open paths (Pencil/Spiral/Pen-aberto) click-selecionáveis
  (`nearest_segment_within` em hit_test + fallback no pick_index/pick).
- `24781dd` polish — #9 recolor sem undo-step espúrio + #11 Pen close-path handle.

Toquei **aditivamente** tua `vector_scene.rs` (helper `will_prune` read-only +
param `stroke_tol_world` em pick/pick_index + STROKE_PICK_TOLERANCE_PX). Sem
mudança de contrato. Testes: pen 29 / vector-doc 70 / shape 20 / direct 15 / bin
codec+undo verdes; shell `cargo check` verde.

**FALTA #10 (MEDIUM, TEU domínio):** `VectorSceneRef` (bbox/centroide) só é escrito
no spawn (reconcile) — após uma edição Direct de vértice a gizmo-box fica stale
(off-center/mis-sized) e um rotate/scale posterior pivota errado. Fix: recomputar
o `VectorSceneRef` + `Transform.translation` da entidade quando o asset muda
(reconcile/scene-object = tua pasta). Companion: invalidar redo em gizmo-move.

Smoke pendente do Enio: undo cross-tool, Ctrl+O load+undo, delete+undo, click em
linha aberta, cor numa forma nova, mirror/break via Pen+Direct, curva Bézier.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
PEDIDO → Coord · menu botão-direito de TIPO DE PONTO (Direct-Select) · 2026-06-03
═══════════════════════════════════════════════════════════════════
**Contexto:** o Mac do Enio compartilha 1 botão físico p/ Cmd+Alt → o
Alt-tangent-break é inalcançável. Implementei (commit `b62a70d`) os 4 tipos
pro de ponto (Corner/Smooth/Asymmetric/Auto = `VertexKind` Free/Mirror/Aligned/
Auto, todos FROZEN-já-existentes) + a ação `VectorDirectTool::set_selected_vertex_kind(committed, selection, kind)` + um **stopgap de teclado** (Direct ativo + vértice
selecionado → teclas **1-4**). Funciona HOJE, Alt-free.

**O que o Enio pediu (UI própria) — é TUA chrome (editor-core context-menu):**
Botão-direito num vértice (Direct ativo) → menu com **Corner / Smooth /
Asymmetric / Auto** → seta o tipo. O `ContextMenuKind` é enum FECHADO em
`crates/ph2d-editor-core/.../widget/context_menu.rs` + `context_menu_overlay.rs`
(match em `req.kind`) + `apply_event` — toda essa chain é tua pasta.

**Contrato sugerido (mínimo):**
1. `ContextMenuKind::VectorPointType` (novo variant) + 4 entry ids
   (`CTX_MENU_VPOINT_CORNER/SMOOTH/ASYM/AUTO`) + entries no
   `paint_context_menu_overlay` (espelha o bloco `ThemeSelector`).
2. `apply_event`: ao clicar um entry, exponha o índice escolhido (0-3) num
   getter drenável — **idêntico ao padrão `take_pending_shape_selection()`** que
   tu já fez pro Shape picker. (Não precisa saber de vetor; só "qual entry".)
3. Eu faço o resto no shell (minha pasta): detectar Secondary-Down sobre um
   vértice em Direct (`nearest_vertex` já existe) → `store.set_context_menu(VectorPointType{...})` na posição do cursor → drenar a escolha → chamar
   `set_selected_vertex_kind`. Bloqueado só no teu variant (não compila sem ele).

**Sem mudança de contrato vetorial** (VertexKind/VectorOp intactos). Quando o
variant landar, te aviso e fecho o shell-glue.
═══════════════════════════════════════════════════════════════════

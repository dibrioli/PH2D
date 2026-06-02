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

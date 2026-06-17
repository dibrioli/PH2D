═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Vector · W2 (Pencil + Shapes + Select + Color + Undo)
Autor: Coordenador · 2026-06-01 · slot dedicado: `slot-impl-vector`
═══════════════════════════════════════════════════════════════════

## §0 — Comece aqui (3 linhas)

**W1 está FECHADA e auditada** (ver §1). Sua missão é **W2** = suíte de tools
básicos: Pencil (Hobby), Shapes, Select/Direct-Select, Color picker, Undo via
CRDT. Plano canônico das tasks: [`docs/Vector Module/17_plano_de_implementacao.md`](Vector%20Module/17_plano_de_implementacao.md) **§5 (T2.1–T2.6)**. Caminho = **(A) drop-crate fan-out** (ADR-0040): cada
tool é um crate isolado. **Primeira task recomendada: T2.1 Pencil** (smoke Day-4).

NÃO continue T1.x — W1 acabou. NÃO toque a API pública do `ph2d-vector` (10+ crates
dependem). NÃO pushe (Coord faz). Leia §3 (isolamento) ANTES de codar.

---

## §1 — Estado factual de W1 (verificado no git, não na memória)

**FECHADA 2026-06-01** após remediação completa da auditoria multi-lens + T1.8 de
confirmação. Relatório canônico: [`docs/AUDIT_vector_module_W1_results.md`](AUDIT_vector_module_W1_results.md) **§8**
(tabela de 15 findings todos CONFIRMED-FIXED). Smoke do Pen (3 pontos → triângulo
Vello) funciona.

### Crates (todos compilam; gates verdes)
| Crate | LOC src | Estado |
|---|---|---|
| `ph2d-vector-traits` | 369 | T1.1 fechado, 14 tests |
| `ph2d-brush-traits` | 193 | T1.1b fechado |
| `ph2d-vector-doc` | 2567 | T1.2 + data model + bounded_decode/encode; 62+ tests |
| `ph2d-vector` | 712 | T1.3 Vello pipeline (`draw_vector_network`) |
| `ph2d-tool-vector-pen` | 976 | T1.5 Pen tool; 28 tests |

### O que é REAL vs STUB (importante para W2)
- `ph2d-vector-doc/src/cubic_fit.rs` (636 LOC) = **REAL** (Levien single-cubic fit,
  T1.4 fechado + testado com fixtures). **W2 precisa do subdivisor multi-cubic**
  (split de um traço em chords ≤90° antes de chamar `fit_cubic_levien`) — é a peça
  que o T2.1 Pencil/Hobby constrói POR CIMA do cubic_fit existente.
- `ph2d-vector-doc/src/crdt.rs` (41 LOC) = **STUB** (`CrdtReplay{site_id, peer_clocks}`,
  sem `apply/merge/replay`). **T2.5 (Undo) implementa a máquina real.** Quando landar,
  exige custom `Deserialize` depth-bounded + gate (padrão já usado no LayerNode do Painter).
- `ph2d-vector-doc/src/spiro.rs` (16 LOC) = **STUB** (Assist mode; W2+, baixa prioridade).

### Shell (já ligado, NÃO re-fazer)
- `shells/desktop/src/render_loop/vector_pen_bridge.rs` — per-frame: committed paths +
  overlay in-progress. Auto-save REMOVIDO (cena in-memory). HR-3: overlay indexado O(N),
  scratch BezPath reusado. Usa `Camera2d::world_to_screen_affine` (fonte única).
- `shells/desktop/src/input_dispatch/vector_pen_input.rs` — clicks + Esc (`try_vector_pen_escape`:
  cancela path em progresso, ou limpa a cena se ocioso) + toast no Rejected.
- `crates/ph2d-editor-core/src/screens/hero/chrome/vector_pen_toggle.rs` — pill PEN (toggle).
- `shells/desktop/src/render_loop/mod.rs` — **COMPARTILHADO**: drena `ActivateTool`/
  `CancelActiveTool` (com toast destrutivo H7(b)) + chama `vector_pen_bridge::dispatch`.

### §6 RATIFICADO (Enio): cena Vector é **in-memory only** em W1. NÃO re-adicione
auto-save. Persistência real (AssetDb + save/load) = **W2 task** (ver §3 Coord).

---

## §2 — W2 scope + primeira task

Tasks completas em **plano §5**. Resumo + dependências:

| Task | Crate novo | Conteúdo | Dep |
|---|---|---|---|
| **T2.1 Pencil** ⭐ | `ph2d-tool-vector-pencil` | Hobby fitter (min curvature variation, MetaPost — NÃO Schneider). Stroke recording + auto-smooth on commit (≈1 cubic / 10 samples). | cubic_fit ✓ |
| **T2.2 Shapes** | `ph2d-tool-vector-shape` | 5 sub-modes (rect/ellipse/poly/star/spiral), live preview no drag. | T1.2 |
| **T2.3 Select** | `ph2d-tool-vector-select` + `-direct` | Select (marquee+click) network-level; Direct Select vertex/tangent (drag move, alt-drag breaks tangent). | T1.2 |
| **T2.4 Color** | (consome picker) | Solid fill + linear gradient 2-stop em regions. **VER §4 gotcha** (crate do picker). | Painter |
| **T2.5 Undo** | (crdt.rs real) | Ctrl+Z → `EditLog::revert_last_op()` → re-render dirty-rect. 50+ ops sem corrupção. | crdt T1.6 |
| **T2.6 Audit** | — | ≥2 lentes (UX · perf undo 1000 ops · CRDT convergence). | tudo |

**Comece por T2.1 (Pencil)** — destrava o smoke Day-4 (traço freehand smoothed) e usa o
cubic_fit que já existe. T2.2/T2.3 são paralelizáveis depois (mas você é 1 agente: serial).

---

## §3 — ISOLAMENTO (leia antes de codar — evita colisão com Painter impl ativo)

### SUA pasta (pode editar livre):
- Crates novos de tool: `ph2d-tool-vector-pencil/shape/select/direct` (você cria).
- `ph2d-vector-doc/src/crdt.rs` + `spiro.rs` (stubs → real, para T2.5/Assist).
- Os arquivos shell *Vector-específicos*: `vector_*_bridge.rs`, `vector_*_input.rs`,
  `vector_*_toggle.rs` (novos por tool, espelhando os do Pen).

### PARE e reporte ao Coord (NÃO edite você mesmo):
- **`ph2d-vector` API pública** — 10+ crates dependem (incl. `ph2d-tool-painter`, o impl
  do Painter está ATIVO). Mudar a superfície = quebra todo mundo. (ph2d-vello/kurbo só aqui.)
- **`Camera2d`** (`ph2d-render`) — `world_to_screen_affine` é a fonte única; consuma, não duplique.
- **`shells/desktop/src/render_loop/mod.rs`** — shared dispatch. Se um tool novo precisa
  mudar a *assinatura* de um `*_bridge::dispatch` ou adicionar um drain, **reporte ao Coord**
  (eu edito o mod.rs pra não colidir com o Painter impl).
- **Persistência / AssetDb host** (T2.4-ish save/load) — é shell + foundational = **Coord**.
- **Gate `vello_kurbo_only_in_ph2d_vector`** — hoje W2-deferred, NÃO EXISTE (CLAUDE.md §6 honesto).
  Se W2 adicionar `vello`/`kurbo` deps em crate novo, o gate precisa ser implementado = **Coord**.
- **arch-gates congelados** do Vector (`architecture_vector_contract_surface`): VectorOp≤16 etc.
  (CLAUDE.md §6). Mexer = Coord + ADR.

---

## §4 — Gotchas / correções (o plano tem erros menores)

1. **Color picker (T2.4):** o plano diz `ph2d-painter-color::ClassicPicker` — esse crate
   **NÃO existe**. O picker real é `ph2d-color` (OKLCH 3-via, já usado pelo Sprite W6).
   Confirme com o Coord qual widget reusar antes de T2.4 (depende do estado do Painter).
2. **Tool nova = drop-crate (ADR-0040):** crate isolado + manifest + register via
   `ph2d-tool-sync` codegen. **SVG novo exige IconId variant** (ordem alfabética) senão o
   índice `ICON_CMDS_BY_ID` quebra TODOS os ícones (gate `enum_order_matches_svgs`).
   Estenda os 2 testes hand-maintained do registry-init (cluster order + icon slug map).
3. **Downcast no bridge** = exceção documentada ADR-0040 §3 (espelhe o Pen/Painter);
   o central dispatch (`mod.rs`) fica downcast-free (gate
   `architecture_no_downcast_to_concrete_tool_in_shell` — bridge é allowlisted).
4. **UI em inglês** sempre (labels/toasts), mesmo descrevendo em pt-BR. Zero hex, zero
   f32 de UI literal, zero string hardcoded (HR-15 / tokens).
5. **Velocidade:** inner loop = `cargo check -p <crate>` no SEU slot (§5). Teste/clippy/
   auditoria 1× no fim. NÃO `cargo check --workspace`.

---

## §5 — Slot, git, ship

- **Slot dedicado:** `slot-impl-vector` (warm, CoW). No início de cada burst:
  `bash scripts/slot-seed.sh impl-vector` → prefixe TODO cargo com o `CARGO_TARGET_DIR`
  impresso (o env não persiste entre tool calls). Nunca use o `target/` default.
- **RAM:** ≤3 cargos compilando (8 GiB). Hoje: Painter impl + Coord + VOCÊ = 3, no limite.
- **Git:** 20 commits locais à frente do origin (Vector W1 + Painter + Coord), **não-pushados**.
  Você commita LOCAL escopado (`git commit --no-verify -m "msg" -- <seus paths>`), `git status`
  antes de stage, NUNCA `-A`/`git add .`/stash. Se houver `M`/`??` alheio, não comite — reporte.
  **Você NÃO pusha** — Coord faz o ship 1× por jornada.

---

## §6 — Memória que você DEVE ler antes de agir
- [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md) — gaps in-scope fecham na sessão; padrão-ouro vence cronograma.
- [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md) — rotacione lentes adversariais entre rounds (T2.6).
- [`feedback-documented-decision-chesterton-fence`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_documented_decision_chesterton_fence.md) — comentário "intentionally NOT X" = decisão ratificada; não sobrescreva por primeiros-princípios.
- [`feedback-audit-scope-discipline`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_scope_discipline.md) — bug em crate alheio = handoff, não fix.
- [`feedback-app-ui-english-only`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_app_ui_english_only.md) — UI strings em inglês.
- [`feedback-new-tool-icon-needs-iconid`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_new_tool_icon_needs_iconid.md) — SVG novo exige IconId.

---

## §7 — Arquivos de leitura inicial
| Path | Por quê |
|---|---|
| [`docs/Vector Module/17_plano_de_implementacao.md`](Vector%20Module/17_plano_de_implementacao.md) §5 | tasks W2 canônicas |
| [`docs/AUDIT_vector_module_W1_results.md`](AUDIT_vector_module_W1_results.md) §8 | estado fechado + carry-overs W2 |
| `crates/ph2d-vector-doc/src/cubic_fit.rs` | o fitter que T2.1 estende |
| `crates/ph2d-vector-doc/src/crdt.rs` | stub que T2.5 implementa |
| `crates/ph2d-tool-vector-pen/src/tool.rs` | padrão de tool (espelhe pro Pencil) |
| `shells/desktop/src/render_loop/vector_pen_bridge.rs` | padrão de bridge |
| ADR-0040 (tool isolation) · ADR-0056/0059/0062 (Vector contracts/renderer/bridge) | contratos |

═══════════════════════════════════════════════════════════════════
Boa caçada. Comece pelo `git status` + ler este handoff + plano §5 T2.1. Depois crie
`ph2d-tool-vector-pencil` espelhando `ph2d-tool-vector-pen`.
═══════════════════════════════════════════════════════════════════

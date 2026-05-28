# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** ATIVO 2026-05-28 — Sprite Inspector v2 W1. Continuação-audit dos 4 commits anteriores fechou **GO**. **T1.1 (Sprite v3→v4, 20 fields) FECHADO padrão-ouro — commit `4591f7e`** pós 2-lens audit (B ABI/serde + A/E scope/coverage): struct 20 campos + VERSION 4 + construtores + `SpriteVersioned::V4`(disc 0x01) + drift-gate `spritev3_struct_wire_matches_live_sprite_v3` aposentado + 2 testes V4 novos (disc-pin + round-trip) + reconciliação nome helper `default_region_filter_clip` em anatomia/schema/ADR-0070. 85 lib + 23 postcard verdes. **Próxima: T1.2.** Sessão única Coord-A + Implementador. Slot: `impl-sprite`. **RESERVO `crates/ph2d-render/`.**

**⚠️ NOVO pre-existing failure (cross-session, NÃO fixado per audit-scope-discipline):** `cargo clippy -p <qualquer> --all-targets` falha em `crates/ph2d-imageio-svg/src/lib.rs:84` (`field_reassign_with_default`, rust-1.95.0). Surge porque clippy-driver linta workspace path-deps. **BLOQUEIA `ship.sh`/CI clippy.** Owner: imageio-svg / foundational. Fix trivial 1-linha (`usvg::Options { ..Default::default() }`). Reportado ao Enio.

**Notas para próximo agente:**
- W1.T1.6 migrator é **MANDATÓRIO** (não fallback) — ADR-0070-amendment-2 §3 reduziu o hybrid `#[serde(default)]` a single tier (wrapper enum único caminho).
- T1.3.5 expandiu workspace-wide (de 1 site em ph2d-ecs para 23 sites em 5 crates) per R1 Lens B finding — todos os transcendentals que tocam Transform direta ou indiretamente. Arch-gate `libm_exact_version_pin_enforced_in_workspace` mantém pin discipline (5 crates). Cross-OS golden hash `d2a3ca34…cf07f` pinada em `transform_determinism.rs`.

**Pre-existing failures cross-session (NÃO fixadas — `feedback-audit-scope-discipline`; reportar ao owner):**
1. `cargo test -p ph2d-editor-core --test architecture_panel_loc_cap` → `panel-hierarchy/src/paint.rs::paint_hierarchy_body 388 LOC > 200 cap`. Inflou via commits `3fab958` + `4fb822b` da hierarchy session.
2. `cargo check -p ph2d-host-desktop` → `ph2d-tool-painter`: `PanelEvent::Activated` variant missing. WIP Painter session pós-`231d6cc`/`1485471`.

Working tree contamination (não staged por mim; aparecerão se próximo agente fizer `git add -A`):
- `shells/desktop/src/hero_intents/sprite_merge.rs` (M)
- `shells/desktop/src/name_unique.rs` (M)
- `crates/ph2d-brush-traits/tests/_audit_*.rs` (??) + `crates/ph2d-vector-traits/tests/_audit_send_sync.rs` (??)
- Recomendação: próximo commit/push escope paths via `git add -- <path>` específico, não `-A`.

**Pastas tocadas (commits cef1959 + e3ad19f + 5974a84 + fix-up pendente):**
- `crates/ph2d-render/` — T0.12 + T0.13: `SpriteVersioned` wrapper + `SpriteV3` mirror + 5 fixtures v3 + 22 tests + postcard `=1.1.3` pin + smoke_fixture_renderable stub + migrate_sprite_v3_to_v4 stub per spec §10.6.
- `crates/ph2d-ecs/` — T1.3.5: `libm = "=0.2.16", default-features = false` + sweep `Transform::compose` + `GlobalTransform::from_transform` + `cross_os_golden_hash_pinned` (blake3 `d2a3ca34…cf07f`) + `libm_exact_version_pin_enforced_in_workspace`.
- `crates/ph2d-editor-core/` — T1.3.5: libm dep + 6 sweep sites em `gizmo/transform.rs` (`world_delta_to_local`, `compose_snapshot`, `resize_corner`, `resize_edge`, `move_pivot_transform`, `pivot_snap_candidates`, `opposite_anchor_translation`) + test parity em `gizmo/tests.rs`.
- `crates/ph2d-tool-rasterize/` — T1.3.5: libm dep + sweep em `rotate_mitchell_premult`.
- `shells/desktop/` — T1.3.5: libm dep + sweep em `input_dispatch.rs` (move-pivot), `input_dispatch/gizmo_drag.rs` (3 sites: scale/rotate/translate), `render_loop/snapshots.rs` (extract), `sim_populate.rs` (Vogel spiral demo).
- `tools/asset-cooker/Cargo.toml` — T1.3.5 R2: libm pin tightened `"0.2"` → `"=0.2.16", default-features = false` para alinhar com workspace pin discipline.
- `docs/architecture/decisions/0070-amendment-2.md` (NEW) — ratifica T0.13 empirical finding.
- `assets/smoke_fixtures/sprite_inspector_v2/README.md` (NEW) — spec §15.8.2 directory contract.
- `docs/Sprite_projeto/smoke_goldens/README.md` (NEW) — idem.
- `docs/Sprite_projeto/10_schema_versionamento.md` (MODIFIED) — §10.1 + §10.4 SUPERSEDED markers.
- `docs/HANDOFF_sprite_inspector_v2.md` (MODIFIED — untracked carrying R3 edits) — §5 + §2 atualizados com o achado empírico T0.13 + workspace-wide sweep scope.
- `docs/SESSION_ACTIVE.md` (esta entrada).

**Pastas reservadas para próxima continuação (T1.1..T1.14 schema bump):**
- `crates/ph2d-render/` (continua: bump `Sprite` v3→v4 com 14 novos campos + ABI `RenderInstance` 144B + 11 vertex attrs + gates `architecture_sprite_inspector_surface` + `vertex_attr_offsets_match_struct` expandido + `sprite_tint_finite_rejects_nan_and_inf` + `sprite_scene_load_size_cap_enforced`).
- `crates/ph2d-host/src/` SOMENTE para declarar `MemoryBudget { sprite_inspector_v2: SpriteInspectorMemoryBudget }` ao final de W1 — Coord-A only.
- Read-only: `docs/Sprite_projeto/` (spec normativa ratificada), `docs/architecture/decisions/0069..0074-*.md` + `0070-amendment-2.md`, `0025-amendment-1.md`.

**Pastas reservadas (genéricas, quando voltar a contexto foundational):**
- `scripts/` · `.github/workflows/` · contratos congelados (`crates/ph2d-nodegraph/`, `crates/ph2d-editor-core/src/tool.rs`)
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` · arch-gates em geral
- `tools/asset-cooker/` · `crates/ph2d-asset/`

**Pastas NÃO tocar nesta sessão (Coord-A reserva passada — Painter T1.8+ pausado):**
- `crates/ph2d-painter-stroke/` · `crates/ph2d-painter-contracts/` · `crates/ph2d-tool-painter/` · `shells/desktop/src/render_loop/painter_bridge.rs`

> **Nota anti-confusão:** `git status` pode mostrar arquivos `M` em `crates/ph2d-painter-stroke/` / `crates/ph2d-tool-painter/` / outros painter-paths — esses são **resíduo da sessão Painter T1.9 anterior** (commit `231d6cc` — Painter T1.9 wire, 2026-05-28), NÃO desta sessão Coord-A. Verifique `git log --oneline -5` ou `git diff <path>` antes de assumir conflito.

**Contexto pausado (retomar pós-Sprite):**
- Painter W1 T1.8 fechado (commit `231d6cc` — Painter T1.9 wire + 14 audit remediations, 2026-05-28).
- KTX2 Fase 2 W0 fechada e W1.T0 destrancada (ADR-0055-v4 Accepted, audit 9.3/10). 2 commits locais: `971e237` + `db6971c`. Próxima retomada lê [HANDOFF §12](HANDOFF_ktx2_phase2.md) antes de W1.T1.

---

## Coord-B (baldes)

**Status:** INATIVO

**Pastas reservadas (quando ativar):**
- `tools/ph2d-{panel,chrome,widget,node,tool}-sync/` (codegens)
- `crates/ph2d-panel-*` · `crates/ph2d-editor-core/src/screens/hero/chrome/*` · `crates/ph2d-editor-core/src/widget/*`

---

## Convenções

- Atualize sua seção ao **iniciar** sessão e ao **terminar** ou pausar longo.
- Se vai tocar pasta da seção do outro Coord, **PARE e renegocie** — adicione comentário `**!!! CONFLITO: ...**` na entrada dele e espere ack.
- Implementadores: sua pasta exclusiva é declarada no briefing do Coord — não precisa aparecer aqui.
- Quando ambos Coords inativos (como agora), deixe ambas as seções com `**Status:** INATIVO` mas mantenha as Pastas reservadas como referência.

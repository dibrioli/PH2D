# SESSION_ACTIVE — coordenação leve entre Coord-A e Coord-B (DIRETRIZ §1.1.1)

**Propósito:** post-it compartilhado para os 2 Coordenadores saberem o que o outro está fazendo agora — evitar pisar um no outro em arquivos foundational/contrato.

**Não é log histórico.** Limpe sua entrada ao terminar a sessão ou em pausa longa. Entradas antigas vão para `git log`, não pra cá.

**Edição:** apenas Coord-A e Coord-B editam. Implementadores leem (informativo) mas não escrevem.

---

## Coord-A (foundational)

**Status:** ATIVO 2026-05-28 — Sprite Inspector v2 W0 carry-over T0.12 + T0.13 (entregues padrão-ouro pós R1 audits B+C + R2 audits E+A + R3 audits D+meta; +ADR-0070-amendment-2). Sessão única absorve Coord-A + Implementador (Enio 2026-05-28). Slot: `impl-sprite`. **T1.3.5 + T1.x = próxima continuação; ainda NÃO iniciado.** Nota: W1.T1.6 migrator é **MANDATÓRIO** (não fallback) — ADR-0070-amendment-2 §3 reduziu o hybrid `#[serde(default)]` a single tier (wrapper enum único caminho).

**Pastas tocadas nesta sessão (já staged):**
- `crates/ph2d-render/` (T0.12 + T0.13 entregues: `SpriteVersioned` wrapper + `SpriteV3` frozen mirror + 5 fixtures v3 + 25+ tests verde + postcard `=1.1.3` pin + smoke_fixture_renderable stub + migrate_sprite_v3_to_v4 stub per spec §10.6)
- `docs/architecture/decisions/0070-amendment-2.md` (NEW; ADR amendment ratificando achado empírico T0.13)
- `assets/smoke_fixtures/sprite_inspector_v2/README.md` (NEW; spec §15.8.2 directory contract)
- `docs/Sprite_projeto/smoke_goldens/README.md` (NEW; spec §15.8.2 directory contract)
- `docs/SESSION_ACTIVE.md` (esta entrada)

**Pastas reservadas para próxima continuação (W1 strategic-only schema bump):**
- `crates/ph2d-render/` (continua: bump `Sprite` v3→v4 com 14 novos campos + ABI `RenderInstance` 144B + 11 vertex attrs + gates `architecture_sprite_inspector_surface` + `vertex_attr_offsets_match_struct` expandido + `sprite_tint_finite_rejects_nan_and_inf` + `sprite_scene_load_size_cap_enforced`)
- `crates/ph2d-ecs/Cargo.toml` + `crates/ph2d-ecs/src/transform.rs` (T1.3.5: `libm = "0.2"` dep + sweep `f32::sin_cos` → `libm::sincosf` em `Transform::compose`; precede W2.T2.2 skew amendment-1)
- `crates/ph2d-host/src/` SOMENTE para declarar `MemoryBudget { sprite_inspector_v2: SpriteInspectorMemoryBudget }` ao final de W1 — Coord-A only
- Read-only: `docs/Sprite_projeto/` (spec normativa ratificada), `docs/architecture/decisions/0069..0074-*.md` + `0070-amendment-2.md`, `0025-amendment-1.md`

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

# SESSION_ACTIVE — coordenação leve (DIRETRIZ §1.1)

**Propósito:** post-it compartilhado do estado vivo da orquestração. **Modelo atual
(2026-05-28): 1 Coordenador único + 5 Implementadores** (Enio reestruturou após
colisões git entre implementadores paralelos). O Coordenador mantém esta seção; os
5 implementadores **leem antes de cada burst** e não escrevem aqui.

**Não é log histórico.** Entradas concluídas vão para `git log`. Limpe ao encerrar.

**Baseline git (2026-05-31, novo Coord):** `origin/main` = `9491f9f` (**PUSHADO**, CI
run [26721192390](https://github.com/dibrioli/PH2D/actions/runs/26721192390) **verde** ✓ —
fundação dos 16 commits). HEAD local = `e15d122` · **1 commit ahead** (só o doc de handoff
do Coord anterior, docs-only). Os 83 commits do baseline antigo JÁ FORAM shipados. Push é
decisão do Enio, via Coordenador, 1× por jornada após `./scripts/ship.sh` verde.

---

## COORDENADOR (único) — ATIVO 2026-06-01

**Modo de entrega (Enio):** **SEM push/CI até o fim de TODA a implementação** — commits
locais acumulados, ship único no fechamento. Ship só quando o Enio mandar.

### ATUALIZAÇÃO 2026-06-01 — Painter W3 + KTX2 fechados pelo Coord (auditados)

Tudo LOCAL, não-pushado. (A seção histórica abaixo descreve a jornada anterior — superseded.)

- **Painter W3 Block 2 — compositor GPU** (`6ba3ed7`): `ph2d-render::LayerCompositor`,
  22 modos W3C + grupos (2 entry points cs_flat/cs_grouped), cache texture-array +
  dirty-rect, paridade bit-a-bit vs `apply_blend` ≤1 byte. Perf gates honestos
  (dirty-rect interativo <5ms; full-4K bandwidth-bound → escala linear).
- **Persistência v2** (`249735e`, ADR-0046-amд-1): `LayerStackEntry::Node` — layer stack
  sobrevive save/load; migração v1→v2; ponte u64↔u32 (stroke records ficam u32).
- **KTX2 W2.T4 fechado** (`385e7e2`): magenta missing-texture placeholder (addendum do plano).
- **Divergência LayerStack ratificada** (Opção A + cap=999) — handoffs RATIFIED + block2_done.
- **AUDITORIA MULTI-AGÊNTICA** (2026-06-01, 33 agentes, 6 lentes): pegou **1 CRITICAL real**
  (stack-overflow DoS na desserialização recursiva do LayerNode num savefile forjado) +
  6 LOW/MEDIUM. **Todos remediados** (`4368a77` deserialize depth-guard + 3 LOW; `834b840`
  active-flag + deep-nest parity + readback + WGSL). Relatório:
  [`docs/AUDIT_painter_w3_ktx2_session_2026-06-01.md`](AUDIT_painter_w3_ktx2_session_2026-06-01.md).

**Implementador (janela separada):** fez Block 1 in-memory (`5d91c91`/`a375479`, tool owns
LayerStack + composite preview) + painel read-only (`6e17c5a`) consumindo a API/ratificação.

**Pendente do Enio:** dock toggle (C, recomendado) + palavra de **ship** (working tree tem
WIP do implementador → ship fecha quando ele terminar a integração). Follow-ups aceitos-LOW
no relatório (eviction/version tests, region test, etc. — sem caller de produção ainda).

**Ambos os slots paralelos FECHARAM + integrados (gate verde, fmt limpo):**
- **KTX2 W2.T3** ✅ (`29defc6`) — `compressed_pipeline.rs` 1 pipeline compartilhado (todos amostram
  filterable-float), block-alignment via helpers wgpu, F4 Rgba32Float rejeitado. 15 tests + 2 ignore-GPU.
- **Picker wire T2.3** ✅ (`b5ba460`) — thumb flutuante top-right (canônico `ColorSwatch`) → abre
  BlenderPicker seedado → read-back→`apply_ui_edit(SetColorSrgb)`. Anti-loop via último-sRGB-empurrado
  (AtomicU32), não round-trip (±1 LSB re-disparava). Contrato intacto, gates verdes.
- **Coord gate fix** ✅ (`f4d24d7`): o UndoController (`640f1d4`) tripava `no_bare_byte_color`
  (blobs `&[u8]` de textura) — anotado `// COLOR-RAW-OK` (multi-line p/ sobreviver rustfmt) +
  limpei fmt drift que os 3 agentes paralelos deixaram (color/tool/undo/painter_bridge). fmt --check
  workspace = 0. **Lição:** nextest `-p` scoped dos slots escondeu o gate de workspace.

**KTX2 status:** W1 fechado (CI verde) · **W2: T1✅ T1.5✅ T2✅ T3✅ + AUDITADA (3 lentes
adversariais → APPROVE consolidado, 0 CRIT/HIGH/MED; 3 LOW remediados `4b48b07`).** W2.T2
(SpriteSource::CookedTexture) = tier-agnóstico/aditivo, sem bump de versão, mirror chain minimal.
**Próximo = W2.T4 (loader/render)** — acende a CookedTexture na tela (hoje extract a PULA).
Handoff pronto: [`docs/HANDOFF_ktx2_w2_loader.md`](HANDOFF_ktx2_w2_loader.md) (W2.T4 + T5 + T6 +
W1 CI bundle). Bundle CI W1 (T10/T11.5/T12) fica pro fim.

**Painter status:** T2.3 surface (`b5085d9`) + hue-fix + T2.2 undo/redo (`640f1d4`) + picker UI
(`b5ba460`). Undo/redo **smoke-OK pelo Enio**.
**🔻 Painter DELEGADO a JANELA SEPARADA** — handoff `docs/HANDOFF_painter_w2_sidebar_color.md`
(implementador segue: swatch DENTRO do painel + T2.4/T2.6/T2.7). Coord (esta sessão) já fez a
parte foundational:
- ✅ Picker thumb flutuante REMOVIDO (`6125409`) — aterrissou órfão na top bar (Enio). O swatch
  certo vai DENTRO do painel sidebar (canônico `ColorSwatch` da Widget Gallery, §5.2) = task do impl.
- ✅ Click-through CORRIGIDO (`0bcf952`) — painel Painter não deixa mais pintar através dele
  (`cursor_over_hero_panel` lista `PAINTER_SIDEBAR_PANEL` + `painter_pointer_uv` None sobre painel).
Wire do picker (dispatch + bridge, keyed em `PAINTER_COLOR_THUMB`) fica dormente até o impl registrar o hit.

**Dívidas foundational do Coord (meu wire, caminho C) — DESBLOQUEIAM o slot Painter:**
- ✅ **T2.2 undo/redo dispatch** (`808383a`): Cmd+Z context-sensitive (painter ativo → stroke undo;
  senão → image-edit undo existente) + Cmd+Shift+Z redo. Flags transientes em app_state → consumidas
  no `painter_bridge` (site downcast) → `undo_last_stroke/redo_last_stroke` (preview_dirty já setado
  pelo slot → tela reverte). Shell compila. **Smoke-ável.**
- ⏳ **T2.3 picker wire** (PRÓXIMO, dimensionado): mecanismo = `store.set_picker_target(Some(NodeId))`
  abre o `BlenderColorPicker` flutuante único (`INSP_BLENDER_PICKER`); read-back via
  `blender_picker(...).value.rgba`. Precisa: (1) NodeId novo do target painter (ids.rs); (2) color
  thumb na Painter top bar (editor-core chrome) + hit + click→`set_picker_target`+seed da
  `active_color_srgb8()`; (3) read-back no `painter_bridge` (tem store+tool) → `SetColorSrgb` quando
  target==painter E cor mudou. Multi-arquivo editor-core+shell.
- (Bloqueadas até o picker: T2.4 modifier/eyedropper, T2.6 a11y, T2.7 smoke+audit W2.)

**⚠️ AVIF está DONE** (não há slot): Path C decode+encode+HDR já em origin (`6bd4620`+`b1c44d7`);
o handoff de transição estava stale nessa metade.
**CI do KTX2** (`spike-texture-cook.yml`): bundle do Coord, Enio delegou — montar no fim.

---

### (histórico do modelo 5-impl 2026-05-28 — referência de posse, mantida abaixo)

Orquestrando implementadores em módulos físicamente disjuntos. Briefings escritos:
`docs/HANDOFF_{sprite_w1,imageio_avif,ktx2_w1,painter_w2,vector_w1}_impl.md`.

### Mapa de posse (anti-colisão — zero overlap de escrita)

| Impl | Slot | Módulo | Pasta(s) exclusiva(s) | Caminho |
|---|---|---|---|---|
| 1 | `impl-sprite` | Sprite Inspector v2 | `crates/ph2d-render/` (+leitura `ph2d-ecs/`) | (C) foundational, **RESERVA ph2d-render** |
| 2 | `impl-avif` | Image I/O AVIF | `crates/ph2d-imageio-avif/` | (A)/(D) |
| 3 | `impl-ktx2` | KTX2 Fase 2 | `crates/ph2d-asset-ktx2/` · `tools/asset-cooker/` · `crates/ph2d-asset/` | (A)/(D) |
| 4 | `impl-painter` | Painter W2 | `crates/ph2d-tool-painter/` · `crates/ph2d-panel-painter-sidebar/` (+`painter-stroke`/`-brush`) | (A)/(D) |
| 5 | `impl-vector` | Vector W1 | `crates/ph2d-vector-doc/` · `-traits/` · `crates/ph2d-brush-traits/` · `crates/ph2d-tool-vector-pen/` · shells bridges abaixo | (A)/(D) |

**Pontos compartilhados resolvidos:**
- `crates/ph2d-asset/` → escrita só do **Impl-3 (KTX2)**; Impl-2 (AVIF) só **lê** o bridge `loader.rs::decode_via_imageio_registry`.
- `crates/ph2d-render/` → **LIBERADO** (Sprite W1 fechado 2026-05-29). Vector H5/M3
  **FECHADO** (`172eff2`): `Camera2d::world_to_screen_affine` é fonte única; shell consolidado.
  → **Vector não tem mais touchpoint foundational = drop-crate isolado puro.** LOW §3.4 deferido p/ W2.
- **Painter T2.5 shell-wire FECHADO** (`d24bbd3`, Coord): Cmd/Ctrl+Enter → `request_commit()` via
  bridge (downcast-allowed), flag transiente sem downcast no teclado. check+clippy verdes.
- ✅ **FUNDAÇÃO 100% FINALIZADA:** zero touchpoint foundational entre os módulos abertos. Sprite,
  Vector, KTX2, AVIF, Painter agora são **drop-crates isolados** → ≤3 agentes sem colisão por
  construção (ADR-0075). Pendente: smoke do Cmd+Enter (runtime) + ship dos commits locais.
- `shells/desktop/` bridges → cada tool dona do seu: `vector_pen_bridge.rs` + `vector_pen_input.rs` = **Impl-5** (exceção tool-bridge §3.A.4). Plumbing compartilhado (`render_loop/mod.rs`, keybinds, `painter_bridge.rs`) = **Coordenador**.

### Itens que o Coordenador segura (não delegados)

1. ~~**Ship-blocker clippy** `crates/ph2d-imageio-svg/src/lib.rs:84`~~ → **FIXADO** (struct-update syntax; `cargo clippy -p ph2d-imageio-svg --all-targets -- -D warnings` exit 0).
2. **fmt drift workspace** (limpar com `cargo fmt --all` no ship): `crates/ph2d-editor-core/src/interaction/dispatch/{number_input,tick}.rs` (puro fmt, zero lógica), `crates/ph2d-editor-core/tests/number_input_mapped_link.rs`, `tools/asset-cooker/tests/sample_cook_brush_atlas.rs`, `shells/desktop/src/render_loop/mod.rs:626`.
3. **Painter T2.5 keybind/shell wire** (`painter_bridge.rs` + Cmd+Enter) — caminho (C); Impl-4 expõe o método público, Coord faz o wire.
4. **Sequenciamento Vector H5/M3** — liberar `ph2d-render` ao Impl-5 só quando Sprite W1 fechar.
5. **Push** dos 83 commits — `ship.sh` verde primeiro; decisão final do Enio.

### Pre-existing failures cross-session (NÃO fixar nos módulos — `feedback-audit-scope-discipline`)

1. `cargo test -p ph2d-editor-core --test architecture_panel_loc_cap` → `panel-hierarchy/src/paint.rs::paint_hierarchy_body` 388 LOC > 200 cap (hierarchy session, commits `3fab958`+`4fb822b`).
2. `cargo check -p ph2d-host-desktop` → `ph2d-tool-painter` `PanelEvent::Activated` variant missing (Painter session pós-`231d6cc`/`1485471`).
3. **Painter `history_integration_t19.rs` 4 testes vermelhos** — regressão REAL commitada nas crates do Painter (NÃO é WIP de dispatch alheio, como handoff antigo dizia; o teste nem toca dispatch). **Owner = Impl-4 (Task 0 do briefing).**
4. **asset-cooker `prefab_cook_hash_is_locked` (cooker_determinism.rs:72) vermelho** — Sprite v3→v4 (`4591f7e`) mudou o postcard de `simple_sprite.json5` → golden desatualizada. **Re-pin AUTORIZADO ao Impl-3 (KTX2)** como manutenção coordenada (golden vive na crate dele; input v4 congelado). Bloqueia ship.sh/CI até re-pinado.
5. **⚠️ ISPC SIGBUS flaky ~50%** (asset-cooker, não-determinístico, não-código) — afeta ship.sh + CI cook job. Fix planejado = override nextest scoped a `package(ph2d-asset-cooker)` retries, no bundle de CI do Coord. No ship: retry manual no cook.

### Progresso dos módulos
- **KTX2 (Impl-3): W1.T15 APPROVE** (16 lentes, gate batched). Commits `dffd62c`+`acc6157`. Próximo = W1.T8.1 (aguarda OK Enio). CI (T10/T12/T13 + retry-SIGBUS) = **bundle do Coord**, não do impl.

### Restrição de recursos
**RAM 8 GiB ⇒ máx 2-3 `cargo` simultâneos.** NÃO rodar os 5 implementadores compilando ao mesmo tempo (swap thrashing). Escalonar; cada um com `source scripts/slot-env.sh <slot>` ou `CARGO_TARGET_DIR` próprio.

### Disciplina git imposta a todos os 5
`git add -- <paths>` (nunca `-A`/`-a`/`.`) · `git commit --no-verify -m "msg" -- <paths>` · race-guard `git diff --cached --name-only` + `git diff --name-only --diff-filter=U` antes do commit · **`git stash` PROIBIDO** (injetou conflito em arquivo alheio) · sem push (Coord faz ship 1×/jornada).

---

## Coord-B (baldes — legado do modelo 2-Coord)

**Status:** INATIVO (absorvido pelo Coordenador único no modelo atual)

---

## Convenções

- Coordenador atualiza esta seção ao iniciar/terminar/pausar.
- Implementador que precise tocar pasta fora da sua: **PARA e reporta ao Coordenador** — não edita, não renegocia sozinho.
- Quando o Coordenador encerrar a jornada, limpe os itens concluídos; mantenha o mapa de posse como referência viva enquanto os módulos estiverem abertos.

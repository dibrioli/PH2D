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

## COORDENADOR (único) — ATIVO 2026-06-04

**Modo de entrega (Enio):** **SEM push/CI até o fim de TODA a implementação** — commits
locais acumulados, ship único no fechamento. Ship só quando o Enio mandar.

### ATUALIZAÇÃO 2026-06-04 — ADR-0065 SDF Phase 3 CLOSED (Coord, smoke-OK)
- **SDF draft+reconcile wireado no geometry-graph smoke** (`58ee181` + `87aa7ec` +
  `e2156d5`): marching-squares (`ph2d-vector-sdf::marching`) + gate draft-vs-exato
  no `vector_graph_bridge` (5 ops SDF, 4 topológicos → exato) + auto-frame + fix do
  painel de sliders (z-order fallback em editor-core `paint.rs`). Smoke-OK do Enio.
  Detalhe + follow-up GPU (deferido, baixo valor): [`HANDOFF_vector_sdf_phase3_coord.md`](HANDOFF_vector_sdf_phase3_coord.md) §6.
- **GPU SDF no bridge DONE** (`e853b04`, smoke-OK): draft do drag computa na GPU
  (`GpuSdf` cacheado), `min/max`+marching na CPU. `surface.gpu()` threadado no
  call-site. **ADR-0065 100% FECHADO** (Phase 1+2+3 + GPU).
- **W3 (§6 do plano) FECHADO** — T3.1–T3.5 ✓. **T3.5 audit (3 lentes) DONE** (Coord,
  APPROVE, 0 crit/high/med, 1 LOW opcional): edge-cases boolean verde (`edge_cases.rs` +
  gate de reprodutibilidade); SDF↔Linesweeper consistente (paridade GPU↔CPU passou em Metal,
  sub-pixel); perf boolean 200-segs = 0.59ms/op release (settle path). Relatório:
  [`AUDIT_vector_w3_session_2026-06-04.md`](AUDIT_vector_w3_session_2026-06-04.md).
- **PRÓXIMO p/ impl Vector = W4** (12 geometry nodes, fan-out drop-crate A) — deps
  (T3.1+T0.3) satisfeitas, roda **em paralelo** ao T3.5. Handoff escrito:
  [`HANDOFF_vector_w4_geometry_nodes_impl.md`](HANDOFF_vector_w4_geometry_nodes_impl.md).
  SDF (`ph2d-vector-sdf` + bridge) é Coord-owned — impl NÃO toca.
- **⚠ Baseline real:** HEAD local **~26 commits ahead de origin/main** (inclui SDF
  Phases 1-3 + Painter W4 §3 curve editor + vector boolean W3 §3.B). **Nada pushado.**
  Ship é decisão do Enio (1×/jornada após `./scripts/ship.sh` verde). `Cargo.lock`
  re-sincronizado em `58ee181` (pegou edge vector-boolean stale + vector-sdf).
### MAPA DE POSSE — W4 ATIVO (2026-06-04)
- **W4 progresso:** **11/12 geometry nodes ENTREGUES** pelo impl (verdes, ~80 testes;
  `03c28b5..4db8408`). `pattern-along-path` (12º) deferido p/ W8 (binário + painter-brush).
  Coord wireou o **smoke `PH2D_VECTOR_NODE=<slug>`** (`f0ca76d`) p/ ver cada nó na tela →
  aguarda smoke visual do Enio → então T4.13 audit fecha. Detalhe:
  [`HANDOFF_vector_w4_nodes_coord.md`](HANDOFF_vector_w4_nodes_coord.md).
- **Implementador Vector = ATIVO em W4** (fan-out 12 geometry nodes). **Posse exclusiva:**
  crates NOVOS `crates/ph2d-node-vector-{outline-stroke,roughen,twist,bend-path,
  pattern-along-path,scatter,width-profile,hatch,mirror,corner-round,warp,recolor}/` +
  o diff GERADO de `ph2d-node-registry-init` (via `cargo run -p ph2d-node-sync`).
- **Coord (eu) NÃO toca** os crates de nó W4. Coord segura: `ph2d-vector-sdf` + bridge
  `vector_graph_bridge` (SDF), `render_loop/mod.rs` (CONTENDED), foundational, smoke-wiring
  por-nó (plumbo quando o impl pedir), audit T4.13, ship.
- **Sem colisão pendente:** minhas edições de audit em `ph2d-node-vector-boolean/engine.rs`
  já estão COMMITADAS (`16a7120`) — o impl ramifica do HEAD local e as recebe limpas.
- **Implementador Painter = ATIVO** (W4 §3 curve editor). Posse: `ph2d-panel-painter-layers`
  + `ph2d-tool-painter` + ids aditivos. Coord entregou os 3 tokens cromáticos `curve-r/g/b`
  (`756eb8e`, foundational §3.C) que o desbloqueiam — wire de 2-3 linhas é dele.
- **RAM 3/3:** Coord + impl Vector (W4) + impl Painter (curve). **TETO — não abrir 4º.**
  Coord minimiza cargo próprio; sequencio se houver contenção.

### ATUALIZAÇÃO 2026-06-02 — Painter W4 T4.1+T4.2 LANDADOS (Coord)
- **W4 Adjustment Layers ABERTO.** Contrato congelado `ph2d-painter-brush::adjustments`
  (ADR-0045 + `0045-amendment-1`: ids `u64` crus + mask-as-id + inner-authoritative).
- **T4.1** (`051455b`) + **T4.2** (`d97f906`): `LayerKind::Adjustment(AdjustmentLayer)` +
  compositor arm (copy→`apply_adjustment`→blend por opacity×mask) + `CompositorCache`
  skeleton (BTreeMap/HR-5). Hook **`apply_adjustment(kind, params, acc: &mut [[f32;4]])`**
  (linear straight f32, NÃO u8) = no-op stub. Gates 81/81 painter-contracts verdes
  (`brush_no_sub_sub_structs` re-escopado p/ excluir `adjustments.rs`). Serde aditivo →
  sem re-lock persist/cook-hash.
- **Painter impl DESBLOQUEADO** p/ fan-out T4.3 (HSB) → smoke Day-4 → 23 kinds + T4.15/T4.16,
  tudo em `ph2d-painter-brush` (compute puro, sem tocar layers/compositor). Briefing:
  [`HANDOFF_painter_w4_triage_coord.md`](HANDOFF_painter_w4_triage_coord.md).
- **Vector impl** segue ativo (W2). **RAM 3/3** — Coord NÃO toca crates quentes de Vector
  (undo shell-wiring T2.5 espera janela do Vector impl, espelho do protocolo Painter↔Coord).

### ATUALIZAÇÃO 2026-06-02 — Vector W2 §4 (audit-fixes backlog Coord) — 4/5 fechados
Resposta completa em [`HANDOFF_vector_w2_audit_fixes_coord.md`](HANDOFF_vector_w2_audit_fixes_coord.md) (bloco "RESPOSTA DO COORDENADOR"):
- **§4.5** consume-guard: VERIFICADO moot (sem mudança; espelhar painter regrediria).
- **§4.4** gate de paridade de registro das pills do topbar (`c0eddbf`, bite-testado) — institui o killer `0661862`.
- **§4.2** Shape picker on-screen no inspector docado (`1e3a1be`) — substitui hotkeys 1-5 (que ficam paralelas).
- **§4.1** Rank 10 (vetor=objeto de cena): **ADR-0076 + IMPLEMENTADO** (`3d8eb6b` ADR + `3fafc1e` impl; Enio liberou "vector parado").
  Vetor commitado aparece na hierarquia + pega no gizmo (move/rot/escala). 7 arquivos shell, schema congelado intacto, math testada. **Pendente: smoke visual do Enio.**
- **§4.3** consolidar 5 pills→1 modo VECTOR: **sequenciado** (paridade-ImageToolsV1 inteira; reestrutura UI que funciona; aguarda greenlight do Enio).

### ATUALIZAÇÃO 2026-06-01 — Vector W2 ATIVADO (slot dedicado)
- **Vector W1 FECHADA** (auditada; `8ce8c97` closure + `b3b2f00` M8 + `69febf7` T1.8).
  cubic_fit REAL; crdt/spiro stubs → W2. Smoke Pen OK.
- **Vector impl arranca W2** (Pencil/Shapes/Select/Color/Undo) no slot `slot-impl-vector`
  (re-seedado warm). Briefing: [`docs/HANDOFF_vector_w2_impl.md`](HANDOFF_vector_w2_impl.md). Caminho (A) drop-crate.
- **Budget RAM CHEIO:** Painter impl + Vector impl + Coord = 3/3 cargos. **Não abrir 4º agente.**
- **Coord segura (foundational p/ Vector W2):** `ph2d-vector` API pública (10+ deps), `Camera2d`,
  `mod.rs` shared dispatch, gate `vello_kurbo` (W2-deferred, implementar se W2 add deps), AssetDb host.
- **Premultiply (Painter item 3): NÃO mexer** — byte-space é o correto; troquei p/ linear por engano
  e revertei (`3870733`). Ver `feedback-documented-decision-chesterton-fence`.

### ESTADO REAL DOS MÓDULOS (2026-06-01) — fonte única; seções históricas abaixo são superseded
| Módulo | Estado | Tracker |
|---|---|---|
| **Painter** | W0-W2 fechados; **W3 ATIVO** (impl em janela separada): layers panel + compositor GPU 22-modos + persist v2 + dirty-rect FPS-fix + audit-remediation | `HANDOFF_painter_w3_*` |
| **Vector** | **W1 FECHADA** (auditada); **W2 ATIVO** (slot impl-vector) | `HANDOFF_vector_w2_impl.md` |
| **Sprite Inspector v2** | W0-W3 + W6 + W10 completos (§0-§9 + render Visibility/Ordering/Sampling/ClipChildren + widgets + OKLCH + Material&Blend) | `HANDOFF_sprite_inspector_v2.md` |
| **KTX2 Fase 2** | W0+W1+W2 fechados (cooker+pipeline+budget); W3 Painter-integration | `2026-05-texture-compression-waves.md` |
| **imageio AVIF** | W0-W3 fechado (Path C real encode/decode, zero RUSTSEC) | `2026-05-imageio-waves.md` |
| **Nodes** | W1+W2 fechados + contrato CONGELADO; fan-out aberto | `HANDOFF_node_system.md` |

**Agentes ativos (RAM 3/3):** Painter impl (W3) · Vector impl (W2) · Coord. **Não abrir 4º.**

**⚠ Gate VERMELHO restante (1) a resolver no ship:**
1. `shell_files_respect_hr18_loc_cap` → `render_loop/inspector_commits.rs` 616 LOC > 600 (16 over).
   **Owner = Sprite Inspector v2** (módulo COMPLETO, sem owner ativo). Decompor (extrair
   `apply_sprite_field`/`clamp_frame` ou o `mod tests` p/ arquivo-filho) OU `// ph2d-loc-cap:`.
   NÃO é Painter/Vector. Coord resolve no ship-prep quando o Enio mandar (decompor preferido).

**✅ RESOLVIDO pelo Coord (`12853c1`):** `arch_mode_has_reconcile` (era
`set_blend_mode` do Painter) — era pure field-write benigno (espelho do `set_opacity`
não-flagado; compositor lê `blend_mode` fresh, sem cache no LayerStack). Entrou em
`BENIGN_SET_MODE`. Gate verde.

**✅ Vector W2 wiring (Pencil 69b3788 + Shape 26b5143) verificado:** shell completo
compila + clippy-clean após o Painter fechar o T3.5 (`d53d52d`). Smokes Day-4/Day-8 prontos.

**Pre-existing failures (seção histórica abaixo):** status NÃO re-verificado nesta jornada
(`PanelEvent::Activated`, `history_integration_t19` 4 tests) — Impl Painter confirma ao fechar W3.

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
| 5 | `impl-vector` | **Vector W2** (W1 FECHADA 2026-06-01) | crates `ph2d-tool-vector-{pencil,shape,select,direct}` (NOVOS) · `ph2d-vector-doc/src/{crdt,spiro}.rs` · shells `vector_*_bridge/input/toggle` | (A) drop-crate |

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

# HANDOFF — Sprite Inspector v2 · W4+ (novo agente ASSUME daqui)

> **Self-contained.** Lê este doc + os arquivos citados; não precisa do
> transcript. Mandato: padrão-ouro, zero deferral (CLAUDE.md §0.6).
> Estado base: **W0–W3 SHIPADOS + CI VERDE** (2026-05-31, run 26702041382).
> Doc canônico do módulo: [`HANDOFF_sprite_inspector_v2_finalization.md`](HANDOFF_sprite_inspector_v2_finalization.md).

---

## §0.2 — Smoke pendente do W6 (Enio, `./play.command`)
1. Abra o **Widget Gallery** → seção "Inspector v2 (W6)": confira os 6 widgets
   (Rect2Editor em grade 2×2, BitmaskGrid32 4×8, NumericInputWithUnit "deg",
   SegmentedAdaptive 4 modos, VariantEditor recursivo, KeyValueList).
2. Inspector → §8 Visibility: o grid Visibility Layer (32 bits) e o **Enabler Rect**
   (com On-Screen Enabler ligado) devem renderizar idênticos e editáveis; o rect
   agora é 2×2 (X Y / W H) em vez de 4 linhas empilhadas.
3. Color picker (qualquer painel): o toggle de canais agora tem **3 vias
   RGB/HSV/OKLCH**; em OKLCH os 4 sliders viram L/C/H/Alpha (normalizados 0..1) e
   editar L/C/H altera a cor coerentemente.

## §0.3 — Steer do Enio: OnScreenEnabler culling (quando for ligado)
`OnScreenEnabler` segue **data-stub** (componente + UI + serialização; sem
sistema runtime). Enio (2026-05-31): o modelo per-node-rect do Godot
`VisibleOnScreenEnabler2D` **não é moderno** — quando o culling de
processamento for implementado, NÃO replicar isso à risca; buscar abordagem
data-oriented (ativação por chunk / spatial-hash / query de visibilidade em
batch) em vez de um rect por entidade.

## §0 — Onde estamos (contexto mínimo)

- **W0/W1/W2/W3 = COMPLETOS, pushados, CI verde.** As seções §1-4,6,7,8,9
  do Inspector funcionam (Identity, Transform+skew, Render Source, Sprite
  Sheet, Color&Tint, Ordering, Visibility+Clip+Mask, Sampling). Render de
  stencil (ClipChildren + Mask2D) auditado (6 lentes) + gates pixel-exatos.
- **W6 widgets = COMPLETO (local, commit `76d2645`, pré-smoke).** 6 primitivos
  novos em `crates/ph2d-editor-core/src/widget/` — `Rect2Editor` (Row +
  Grid2x2), `BitmaskGrid32`, `VariantEditor` (recursivo, depth≤4),
  `NumericInputWithUnit`, `KeyValueList`, `SegmentedAdaptive` — cada um com
  a11y + testes + seção no Widget Gallery ("Inspector v2 (W6)", seções 10→11).
  §8 Visibility re-cabeada (grid inline→`BitmaskGrid32`; rect rows→`Rect2Editor`
  Grid2x2). **T6.1 OKLCH:** `ChannelMode::Oklch` + toggle RGB/HSV/OKLCH 3-vias
  + sliders L/C/H/A no BlenderColorPicker (`oklch_lcha()` emite tupla; ph2d-color
  fica dev-only). Auditoria 3-lentes + round-2 verde. **Smoke pendente do Enio**
  (vide §0.2). Decisões de layout: Rect2 usa Grid2x2 no Inspector (coluna estreita
  < MIN_W p/ 4-em-linha); VariantEditor/KeyValueList são referência VISUAL no
  gallery (interação real é dos consumidores W4/W5).
- **Falta:** §5 9-Slice · §10 Material&Blend · §11 Animation · §12 Named
  Anchors · W7 polish (i18n/a11y) · W8 cooker.
- **`RenderInstance` ABI = 184 B / 16 campos, FROZEN.** GPU layout 164 B /
  12 attrs (locations 0..15 CHEIOS). Gates: `render_instance_pod_size_v4`,
  `architecture_sprite_inspector_surface`.

### §0.1 — Mapa de dependências (LEIA antes de escolher a wave)

| Item | Dá pra fazer JÁ? | Dependência |
|---|---|---|
| **W6 widgets** (Rect2Editor, VariantEditor, BitmaskGrid32, …) | ✅ standalone | nenhuma; **desbloqueia** W4/W5 |
| **§11 Animation** (SpriteAnimator runtime + seção) | ✅ standalone | sprite-sheet já existe; Timeline editor é **só stub** |
| **§10 BlendMode** (6 modos) | ✅ standalone | pipeline já tem `Option<BlendState>` |
| **§5 9-Slice** (NinePatch) | ✅ standalone | feature de render pura |
| **W5 Named Anchors** (core) | ✅ standalone | só o *import slice Aseprite* espera o cooker |
| **§10 Material + InstanceShaderParams** | ⚠️ stub ou bloqueado | renderer é **fixed-function** (sem material/shader runtime) → shipar data-stub (igual MaskInteraction foi) OU construir o runtime |
| **W8 Asset Cooker** (Aseprite/PSD) | ❌ depende W4/W5 | precisa dos schemas SpriteFrames + NamedAnchor pra importar |
| **W7 i18n/a11y** | ⏳ depois | melhor após as 12 seções existirem |

**Ordem recomendada:** W6 widgets → §11 Animation → §10 BlendMode → §5
9-Slice → W5 Named Anchors → (Material stub) → W7 → W8.

---

## §1 — Padrões REUSÁVEIS (aprendidos em W2/W3 — siga à risca)

### §1.1 — Render-first
O RENDER de uma feature existe e é validado (gate headless) ANTES da UI da
seção. Sempre.

### §1.2 — Optional-component (presença = override)
Toda feature opcional vira um **Component ECS** (não campo do `Sprite`):
presença = override, ausência = default. Editar = `SetComponent` (attach/
update) ou `RemoveComponent` (tag-0 / valor-default DETACHA). Registra em
[`register_ecs_components`](../crates/ph2d-ecs/src/scene/registry.rs) →
**re-locka 3 count gates** (ecs `registry.rs` + render `registry.rs` +
script `registry.rs`; hoje 20/21/21).

### §1.3 — A stack de UM seção do Inspector (espelhe §9 Sampling / §8 Visibility)
O caminho COMPLETO de uma seção nova (cada arquivo já tem §8/§9 working):
1. **ids** [`crates/ph2d-editor-core/src/ids.rs`](../crates/ph2d-editor-core/src/ids.rs): `INSP_<X>_*`.
2. **hero.rs** [`screens/hero.rs`](../crates/ph2d-editor-core/src/screens/hero.rs): `Inspector<X>Info` (snapshot) + `<X>FieldEdit` (enum) + `Inspector<X>Mixed` (BulkSelect). **f32 → sem derive Eq.** Re-export em `screens/mod.rs` + `lib.rs`.
3. **action_bus.rs**: `EditorAction::Inspector<X>Edit { entity_bits, edit }`.
4. **sections/<x>.rs** (painter, espelha `sampling.rs`/`visibility.rs`).
5. **panel** (`ph2d-panel-inspector`): `state.rs` (thread-local + setter), `lib.rs` (re-export setter), `populate.rs` (**register controles como Button** — senão click dropado, ver §1.4), `sync.rs` (seed NumberInputs do snapshot, focus-guard), `event_ordering.rs` (rota Click/ValueChanged → edit; **split em fn própria se passar de 200 LOC** — gate `architecture_panel_loc_cap`), `paint.rs` (chama o painter + `LIVE_SECTION_IDS` se for seção nova).
6. **shell** (`shells/desktop/src/render_loop/`): `inspector_<x>.rs` (`build_<x>_info` + `apply_<x>_edit` via SetComponent/RemoveComponent; reusa `queue_set`/`queue_remove` de `inspector_ordering.rs` — são `pub(super)`), `snapshots.rs` (produz + publica), `mod.rs` (drena o action + **BulkSelect fan-out**), `inspector_commits.rs` (loop de commit).

### §1.4 — Os 2 bugs de smoke que SEMPRE pegam (não repita)
- **is_focusable:** um id hit-registado SEM `InteractiveState` é rejeitado →
  zero Click. **Toda** tab/checkbox/toggle precisa `register_button_ids` em
  `populate.rs`. (Foi o "nenhum botão funciona" do §7/§9.)
- **label overlap:** label numa fração da row do controle sobrepõe os botões.
  Label em row PRÓPRIA curta acima do controle (ver `visibility.rs` helpers).

### §1.5 — ABI discipline (RenderInstance FROZEN)
Novo dado per-instância pro shader:
- **bool GPU** → bit livre de `flip_uv` (amendment-3) — custo ZERO.
- **dado de stencil/agrupamento** → bits livres de `clip_meta` (amendment-7;
  bits 2-7, 18-31 livres) — custo ZERO.
- **metadata CPU** (agrupa draw calls, escolhe pipeline/sampler/blend) → campo
  novo no **tail CPU** + **ADR-0070-amendment-N** + re-lock dos 2 gates de
  size/field-count + TODOS os sites de construção. (Ver amendment-5 `sampling`
  como template.)
- **16 vertex attrs (loc 0..15) estão CHEIOS** — não cabe um 17º. Se precisar
  de dado GPU numa pipeline específica, **repurpose um location não-usado por
  ela** com offset explícito (ver `clip_meta`@`@location(5)` no mark pipeline,
  ADR-0070-amendment-7 §3).

### §1.6 — Stencil infra (pra qualquer feature de silhueta)
[`clip_pass.rs`](../crates/ph2d-render/src/clip_pass.rs) +
[`pipeline.rs`](../crates/ph2d-render/src/pipeline.rs) (4 pipelines: normal/
mark/test/test_outside) já existem. **Atenção contiguidade:** o clip pass
batcha por scan de runs; o sort do renderer usa **clip-anchor** pra manter o
grupo contíguo (audit HIGH fix). Qualquer feature que dependa de runs
contíguos por grupo precisa de um anchor de sort análogo.

### §1.7 — Gate de regressão headless (render)
Toda feature de render tem um gate pixel-exato headless (ver
`clip_children_regression.rs` / `mask_interaction_regression.rs`): GameRt-like
`Rgba8Unorm` offscreen + readback + amostra de N pixels canônicos × modos.
Skip gracioso sem adapter; roda em Mac dev + CI.

---

## §2 — Briefings por-wave (a fazer)

### W6 — Foundational widgets (FAÇA PRIMEIRO; desbloqueia W4/W5)
Spec §15.7. Vivem em [`crates/ph2d-editor-core/src/widget/`](../crates/ph2d-editor-core/src/widget/) + showcase no Widget Gallery (arch gate de cobertura).
- **Rect2Editor** (4 NumberInputs x/y/w/h + handles de drag no canvas). **Extrair
  do §8 OnScreenEnabler** (que hoje usa 4 NumberInputs soltos em
  `sections/visibility.rs` — após extrair, troca lá).
- **VariantEditor** (dropdown kind + sub-widget recursivo; **cap depth ≤4**).
  Needed por InstanceShaderParams (W4) + NamedAnchor user_data (W5).
- **BitmaskGrid32** (4×8 checkbox). **Extrair do §8 visibility.rs** (grid inline
  de 32 checkboxes; troca lá após extrair).
- NumericInputWithUnit (px/m/deg/rad/%), BlenderColorPicker→OKLCH extend,
  KeyValueList.
- Gate: showcase coverage. Sem ECS, sem render — UI pura.

### §11 — Animation (mais demonstrável e independente)
Spec §3.11 + §15.5. **Render/runtime-first:**
1. **`SpriteFrames` asset** (lista de frames = região + duração) no AssetDb.
2. **`SpriteAnimator` component** + **sistema de tick** (sim-side): avança
   `elapsed_ticks: u64` (fixed-point, W0 specou cross-OS) → calcula o frame
   atual → escreve em `Sprite::frame` (o sprite-sheet sub-UV **já existe**, W2).
   Direction (Forward/Reverse/PingPong/PingPongReverse), Loop, HoldMs,
   RepeatDelayMs, SpeedScale, Playing, Autoplay. **Determinismo HR-5:** ticks
   inteiros; se houver float, `libm`.
3. **§11 seção** (espelha §9): Current(dropdown)/Progress(read-only bar)/
   Speed(slider)/Playing+Autoplay(toggles)/Direction+Loop(segmented/toggle)/
   Hold+RepeatDelay(NumberInput) + **botão "Open Timeline" = STUB** (toast
   "coming soon" — o Timeline editor é módulo separado/futuro, NÃO bloqueia).
- Entry: o sim tick roda em [`sim_extract.rs`](../shells/desktop/src/render_loop/sim_extract.rs) (ou crie um sistema ECS dedicado).
- Gate: teste de determinismo (tick→frame) + um headless visual opcional.

### §10 — Material & Blend
Spec §3.10. **Split em DUAS partes:**
- **BlendMode (FAÇA):** 6 modos (Mix/Add/Sub/Mul/Screen/PremultAlpha). Render:
  estende [`pipeline.rs`](../crates/ph2d-render/src/pipeline.rs) (já tem
  `Option<BlendState>` por variante) pra N pipelines de blend; `compute_runs`
  passa a keyar no blend mode; o renderer escolhe a pipeline por-run. O blend
  mode viaja em bits livres de `flip_uv` (zero ABI) OU num campo `blend` CPU-
  tail (amendment-N, igual `sampling`). Seção = segmented 6 modos.
- **Material + InstanceShaderParams (STUB ou bloqueado):** o renderer é
  **fixed-function** (não há material/shader runtime). Ou (a) shipa data-stub
  (componente + UI persistem, sem efeito — igual MaskInteraction antes do
  Mask2D), ou (b) constrói o material/shader runtime primeiro (foundational,
  Coord-only + ADR). **Recomendado W4: BlendMode real + Material stub**,
  documentando que o runtime é wave futura.

### §5 — 9-Slice (NinePatch)
Spec §3.5. Feature de render pura (não foi feita em W2). Component
`NinePatch { borders: [f32;4], tile_mode }`. Render: remapeia o quad em 9
sub-regiões (cantos fixos, bordas/centro esticam ou tilam) — extract emite 9
sub-quads OU o shader faz o remap de UV 9-slice. Seção §5 (DrawMode segmented
+ borders Rect2Editor + tile mode). Gate headless (canto vs centro).

### W5 — Named Anchors / Sockets (§12)
Spec §3.12 + §15.6. `NamedAnchor` (SortedSmallVec de pontos nomeados, W0
specou) + per-frame override + `CameraFollowAnchor`. Editor usa
**Rect2Editor + VariantEditor (W6)**. Handles visuais no canvas (drag).
**Aseprite slice import = W8** (não-bloqueante; o core é standalone).

### W7 — Polish (depois das 12 seções)
i18n Fluent ~155 keys (en-US + pt-BR; gate `sprite_inspector_i18n_keys_present`)
— hoje as labels são strings inglesas hardcoded (padrão de `sampling.rs` etc.),
migra pro Fluent. WCAG 2.2 AA + AccessKit. `LIVE_SECTION_IDS` → 12. Bug bash.

### W8 — Asset Cooker (depende W4/W5)
Aseprite full + Linked Cels dedup-hash + PSD em [`tools/asset-cooker/`](../tools/asset-cooker/)
(hoje só texture cooker). Precisa dos schemas SpriteFrames (W4) + NamedAnchor
(W5). Registry de MCP Destructive Ops (§7.1.2) já specado.

---

## §3 — Protocolo (inegociável)

- **Anti-colisão git:** `git add -- <só paths Sprite>` (NUNCA `-A`/`stash`).
  `git status` antes de stage; foreign no working tree (`.vscode/settings.json`,
  `docs/HANDOFF_imageio_*`, `docs/HANDOFF_ktx2_*`, `docs/Painter_projeto/*`,
  `docs/UI_Fonts/`, `test_strip`) **NÃO entram**. `commit --no-verify` em
  background (hook estoura timeout). Termina msg com
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
- **Velocidade:** inner loop = `cargo check -p <crate>` no slot warm
  (`bash scripts/slot-seed.sh <N>` → prefixe `CARGO_TARGET_DIR=...`). Gate
  batched no fim: `nextest --workspace` (pega registry counts/ABI/LOC/UI-canon
  que `cargo check` ESCONDE).
- **Gates por feature:** registry counts (se add component), ABI (se mexer
  RenderInstance), UI-canon (`no_literal_color`/`hr15`/`no_magic_numeric`/
  `architecture_panel_loc_cap`/`file_loc_caps`), regressão headless. **Strings
  de UI em INGLÊS** (comentário pode ser pt-BR). Docs `.md` são typos-exempt.
- **Auditoria antes do ship:** rode a auditoria multiagêntica adversarial
  (várias lentes + verificação) sobre a feature nova antes de shipar — pegou 1
  HIGH (data-loss) + 1 MEDIUM em W3 que os gates não pegaram.
- **Ship (fim da wave):** `./scripts/ship.sh` (paridade-CI: fmt/clippy
  --all-targets/machete/deny/audit/nextest ci-test/typos) → corrige todo `✗` →
  `git push origin main` → babysit CI (`gh run watch` / poll) até `success`.
  Forneça SEMPRE o link `https://github.com/dibrioli/PH2D/actions/runs/<id>`.
  Os 4 flaky de `ph2d-asset-cooker texture::cook` (ISPC-macOS) são conhecidos —
  passam no retry.
- **Smoke do Enio:** visual, 1× no fim da implementação da feature.

---

## §4 — Specs canônicas por-seção
`docs/Sprite_projeto/`: §3.x = layout de cada seção; §05 ordering; §06
mask/clip; §09 sampling; §15 plano de implementação (W4 §15.5 / W5 §15.6 /
W6 §15.7 / W7 §15.8); §16 catálogo i18n. ADRs `docs/architecture/decisions/`
0069-0074 + amendments (0070-amд-2..7, 0073-amд-1, 0074-amд-1, 0025-amд-1).

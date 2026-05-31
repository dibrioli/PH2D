# HANDOFF — Sprite Inspector v2 · W3 · Phase 6 (ClipChildren + §8 Visibility)

> **Self-contained.** Lê este doc + os arquivos citados; não precisa do
> transcript da sessão. Mandato: padrão-ouro, zero deferral (§0.6 CLAUDE.md).
> Autor: sessão solo Coord+Impl, 2026-05-30. Estado: Fases 1–5 + §7 + §9 +
> UV tiling + smoke-fixes ENTREGUES e gateadas (≈25 commits LOCAIS, nada
> pushado). Esta fase é a **peça GPU mais difícil da W3** — por isso o
> handoff. Decisão do Enio: **"faz agora (stencil GPU às cegas)"**.

---

## 0. Onde estamos (contexto mínimo)

- **Render-first** (handoff W3 §3): o RENDER de uma feature existe ANTES da
  UI da seção. Já feito: §7 Ordering (sort pipeline), §9 Sampling
  (filter/repeat per-node + UV tiling), Phase 4 VisibilityLayer cull.
- **§8 NÃO pode vir antes do ClipChildren**: das linhas do §8 (spec §3.8), a
  única visualmente demonstrável é ClipChildren. VisibilityLayer cull é
  no-op sem UI de camera-mask (não existe); MaskInteraction é **data stub**
  em W3 (spec T3.10 — `Mask2D` é módulo futuro); OnScreenEnabler é não-visual.
  → **Phase 6 lidera com o render do ClipChildren**, depois monta o §8.
- **Componentes já existem** em [`crates/ph2d-ecs/src/masking.rs`](../crates/ph2d-ecs/src/masking.rs):
  `ClipChildren { mode: ClipMode, alpha_cutoff: f32 }` (Disabled/ClipOnly/
  ClipAndDraw, DEFAULT_CUTOFF 0.5, `.clamped()`), `MaskInteraction { mode:
  MaskMode, alpha_cutoff }` (None/VisibleInside/VisibleOutside). Já têm serde
  round-trip + clamp tests. **Falta**: registrar no ComponentRegistry (estão
  fora dos 20 atuais? conferir — ver §4.1) + o RENDER do clip + o §8.

---

## 1. Deliverable A — Render do ClipChildren (STENCIL)

### 1.1 Decisão técnica: stencil, não backbuffer

A spec §6.2 diz "back-buffer pass", mas **stencil é a técnica canônica de
clip por silhueta** e é mais barata (sem RT extra por clip-parent). O
back-buffer precisaria de 1 render-target por clip-parent + pass de
composição + sample da máscara. Stencil precisa só de 1 attachment de
stencil compartilhado + sub-passes. **Recomendação: stencil.** Ratificar via
**ADR-0074-amendment-1** (ou o ADR de mask que estiver Accepted — conferir
`docs/architecture/decisions/0074-*`) registrando "impl = stencil, não
backbuffer; spec §6.2 wording atualizado".

### 1.2 Como o stencil mapeia nos 3 modos (spec §6.2)

Para cada **clip-parent** (entidade com `ClipChildren.mode != Disabled`),
processado na ordem de z_order:

1. **Marcar a silhueta**: desenhar o sprite-pai num pipeline "stencil-mark"
   que faz `discard` onde `alpha <= cutoff` e escreve `stencil = ref` (op
   `Replace`) onde passa. **Não escreve cor.** (color write mask = `EMPTY`.)
2. **Desenhar descendentes** com stencil test `Equal ref`, `mask` de leitura
   = 0xFF, sem escrever stencil. Só pinta onde a silhueta marcou.
3. **ClipAndDraw**: desenhar TAMBÉM a cor do pai (pass normal, stencil off).
   **ClipOnly**: pular o passo 3 (pai é molde invisível).
4. **Reset**: limpar o stencil da região antes do próximo clip-parent — ou
   usar `ref` incremental por grupo (evita clear; cuidado com overflow u8 →
   reciclar a cada 255 grupos, raríssimo). **Recomendado: clear stencil
   load no início + ref incremental; clear só se ref estourar.**

Sprites **sem** clip (a maioria) desenham no pass normal como hoje — zero
regressão. Identity garantida: `clip_group == 0` → caminho atual intacto.

### 1.3 Identificação do subtree (extract-side)

`z_order` é a ordem DFS de `propagate_transforms` → **o subtree de um
clip-parent é um range CONTÍGUO de z_order**. No extract
([`shells/desktop/src/render_loop/sim_extract.rs`](../shells/desktop/src/render_loop/sim_extract.rs)),
que tem a hierarquia + a Entity de cada instance:

- Ao caminhar a hierarquia, manter um stack de clip-ancestors. Para cada
  entidade, se um ancestral tem `ClipChildren.mode != Disabled`, taggar a
  instance com o **clip_group** (= z_order do clip-parent + 1, nunca 0) e
  marcar o clip-parent como mask-source.
- O clip-parent recebe `clip_group` próprio + `clip_role` = mask-source
  (com mode + cutoff). Os descendentes recebem o mesmo `clip_group` +
  `clip_role` = member.
- **Nesting** (clip dentro de clip): W3 = **single-level** (guard: se um
  clip-parent é descendente de outro, log warn + trata só o mais interno,
  OU empilha ref incremental — decisão §6). Marcar no gate.

### 1.4 ABI: RenderInstance amendment-7 (CPU-only tail)

Hoje `RenderInstance` = 14 campos, **176 B** (amendment-6 uv_xform). O clip
grouping é **CPU-side** (agrupa draw calls; NÃO vai pra vertex attr). Adicionar
**2 campos CPU-only tail** (como sampling/z_order já são):

```rust
// CPU-only (após uv_xform): clip-stencil grouping (ADR-0070-amendment-7).
pub clip_group: u32,  // 0 = sem clip; senão (clip_parent.z_order + 1)
pub clip_meta: u32,   // bits0-1 role: 0 member · 1 mask ClipOnly · 2 mask ClipAndDraw
                      // bits8-15: alpha_cutoff quantizado u8 (round(cutoff*255))
```

→ size 176 → **184 B**. Re-lockar **3 gates**:
- `crates/ph2d-render/tests/render_instance_pod_size_v4.rs` (size + field count 14→16).
- `crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs` (destructure inclui clip_group+clip_meta).
- `vertex_attr_offsets_match_struct` (no `sprite.rs` in-module): **não muda**
  (campos GPU contíguos intactos; clip é tail CPU). Confirmar offsets dos 12
  attrs inalterados.
- `IDENTITY` consts dos tests do renderer (`inst()` em renderer.rs linha ~528,
  `compute_runs` tests) ganham `clip_group: 0, clip_meta: 0`.

**Alternativa sem ABI** (avaliar): render() já tem `present.world_mut()` →
poderia re-query `ClipChildren`+`ChildOf` por frame, mas `RenderInstance` não
carrega `Entity`, então o mapeamento instance→entity se perde. Tail CPU é o
caminho consistente com sampling/z_order. **Recomendado: amendment-7.**

### 1.5 Pipeline + attachment (wgpu)

[`crates/ph2d-render/src/pipeline.rs`](../crates/ph2d-render/src/pipeline.rs)
hoje: `depth_stencil: None`. Mudanças:

- Criar textura de stencil (`Stencil8` se suportado, senão
  `Depth24PlusStencil8`) do tamanho do target. Guardar no `SpriteRenderer`,
  recriar no resize (o renderer recebe `window: WindowSize`).
- **3 pipelines** (ou 1 base + variações de `depth_stencil`/`color_write`):
  1. **normal** (atual) — sem stencil (clip_group==0 path).
  2. **stencil-mark** — color write `EMPTY`, stencil `Replace` on pass,
     fragment `discard` se `alpha <= cutoff`. Precisa de um entry-point
     fragment novo no `sprite.wgsl` (`fs_stencil_mark`) OU um uniform/flag
     que faça o `fs_main` só discard+sem cor (mais simples: entry separado).
  3. **stencil-test** — color write normal, stencil `Equal ref` read-only.
- O `cutoff` por grupo: passar via push-constant? PH2D usa push-constants?
  **Conferir** — se não, via um pequeno uniform por grupo (set_bind_group)
  OU embutir o cutoff no instance (já está em clip_meta) e o fragment
  stencil-mark lê do attr. **Recomendado: cutoff no instance attr** (já vai
  no tail; promover clip_meta a GPU attr SÓ no pipeline stencil-mark — ou
  decodificar cutoff de um attr existente). Detalhe a resolver no código.

### 1.6 render() multi-pass (renderer.rs)

O `render()` atual (linha 390) faz 1 pass. Reestruturar:

- Após sort + compute_runs, **detectar clip-groups** (runs com mesmo
  clip_group != 0 contíguos). Construir um `Vec<ClipSpan>` reusável (HR-3):
  `{ group, role, cutoff, mask_run_range, member_run_range }`.
- **Pass 1** (load Clear cor + Clear stencil): desenhar TODOS os runs com
  clip_group==0 (normal) — exatamente o loop atual, filtrado.
- **Por clip-group** (em ordem de z): sub-pass stencil-mark (mask run) →
  sub-pass stencil-test (member runs) → se ClipAndDraw, draw normal do mask.
  Mesma `encoder`, `begin_render_pass` com `depth_stencil_attachment` Some.
  **Atenção**: múltiplos `begin_render_pass` no mesmo encoder com load
  `Load` (preservar cor) após o pass 1 com `Store`. Stencil `Clear(0)` no
  primeiro, `Load` nos seguintes (ou ref incremental).
- Manter alloc-free: `Vec<ClipSpan>` reusado como `runs`.

**Risco LOC**: renderer.rs já é grande (618 linhas). O multi-pass + spans
provavelmente estoura HR-18 (600). **Extrair** o clip-pass para um módulo
`crates/ph2d-render/src/clip_pass.rs` (`fn encode_clip_groups(encoder, ...)`).

### 1.7 Gate de regressão OBRIGATÓRIO (spec §6.3)

Razão (spec §3.8): Godot teve 5 issues sucessivos de regressão de clip
(#79885, #102190, #102224, #91068, #90793). PH2D **exige**
`crates/ph2d-render/tests/clip_children_regression.rs`:

- Render headless num offscreen target (ver `GameRt`/offscreen em
  `crates/ph2d-render/src` — há infra de render-to-texture usada por outros
  testes; procurar `readback`/`GameRt`). Montar fixtures mínimas:
  - **Disabled**: filho fora da silhueta do pai aparece (sem clip).
  - **ClipOnly**: filho recortado pela silhueta; pai NÃO desenha (pixels do
    pai = clear; pixels do filho só dentro da silhueta).
  - **ClipAndDraw**: pai desenha + filho recortado.
- Comparação **4-pixel** por modo (spec): amostrar 4 pixels canônicos
  (dentro-silhueta-filho, fora-silhueta-filho, pai-only, fundo) e asserir
  cor esperada. Não snapshot de imagem inteira — 4 pontos determinísticos.
- Cutoff: 1 fixture com alpha_cutoff variando (0.2 vs 0.8) muda a borda.

---

## 2. Deliverable B — §8 Visibility section (espelha §9)

Só DEPOIS do render do ClipChildren existir. Layout spec §3.8:

| Campo | Widget | Default | Componente |
|---|---|---|---|
| Visible | toggle | true | universal (já pintado acima das seções) |
| Visibility Layer | Bitmask 4×8 (32 checkbox) | bit 0 | `VisibilityLayer` (já existe, cull feito P4) |
| Clip Children mode | Segmented (Disabled/ClipOnly/ClipAndDraw) | Disabled | `ClipChildren` |
| Mask Interaction | Segmented (None/VisibleInside/VisibleOutside) | None | `MaskInteraction` (stub) |
| Alpha Cutoff (se Mask != None) | Slider 0..1 | 0.5 | `MaskInteraction.alpha_cutoff` |
| On-Screen Enabler | toggle + Rect2 editor (collapsible) | false | `OnScreenEnabler` (ver visibility_layer.rs) |

### 2.1 Padrão a copiar — o §9 Sampling é o template EXATO

Cada seção optional-component segue o pipeline (todos os arquivos já têm o
§9 como exemplo working):

1. **ids** [`crates/ph2d-editor-core/src/ids.rs`](../crates/ph2d-editor-core/src/ids.rs):
   `INSP_LIVE_VISIBILITY_SECTION` + `_COLOR`; `INSP_VIS_*` (CLIP[3] segmented,
   MASK[3] segmented, ALPHA_CUTOFF slider, LAYER_BIT[32] checkbox grid,
   ON_SCREEN toggle + RECT_X/Y/W/H). **Bump `LIVE_SECTION_IDS` [_;8]→[_;9].**
2. **hero.rs** structs: `InspectorVisibilityInfo` (clip_mode u8, mask_mode u8,
   alpha_cutoff f32, layer_mask u32, on_screen bool, rect [f32;4], mixed) +
   `VisibilityFieldEdit` (ClipMode(u8), MaskMode(u8), AlphaCutoff(f32),
   LayerBit(u8,bool), OnScreen(bool), Rect([f32;4])). **f32 → sem derive Eq.**
3. **action_bus.rs**: `EditorAction::InspectorVisibilityEdit { entity_bits, edit }`.
4. **sections/visibility.rs** (novo, painter) — espelha `sampling.rs`. O
   **bitmask 4×8** é o widget novo: loop 32 checkboxes em grid (4 col × 8
   linha), cada `hit_index.register` + `read checkbox`. Cuidado LOC (≤600) +
   label-above (não sobrepor — ver bug smoke 2026-05-30 em sampling.rs:99).
5. **panel** state.rs (thread-local current_inspector_visibility), lib.rs
   (re-export), populate.rs (registrar segmenteds/checkboxes como **Button**
   — senão WidgetStore is_focusable rejeita o click, ver bug smoke §9!),
   sync.rs (seed fields do snapshot), **event_visibility.rs** (novo, espelha
   event_ordering.rs — split por LOC), paint.rs (chamar paint_visibility +
   notes_per_section/LIVE_SECTION_IDS → 9).
6. **shell** `shells/desktop/src/render_loop/inspector_visibility.rs` (novo):
   `apply_visibility_edit` (RMW ClipChildren/MaskInteraction/VisibilityLayer/
   OnScreenEnabler via SetComponent/RemoveComponent), `build_visibility_info`
   (lê os componentes). `snapshots.rs` (producer + publish), mod.rs (drain +
   BulkSelect fan-out), inspector_commits.rs (dispatch), init.rs/app_state.rs
   (params se precisar).

### 2.2 ⚠️ Os 2 bugs de smoke a NÃO repetir

- **is_focusable**: um id hit-registado SEM InteractiveState é rejeitado →
  sem Click. **Toda** tab/option/checkbox de segmented precisa ser registada
  como Button em populate (`register_button_ids`). Foi o "nenhum botão
  funciona" do §9.
- **label overlap**: label numa fração da row de controle sobrepõe os botões.
  Dar ao label sua PRÓPRIA row curta (`label_h = label_font + Spacing::Xs`),
  controle abaixo. Ver `sampling.rs` `uv_pair_row` + o comentário linha 97-100.

---

## 3. ADRs a escrever (Phase 8, mas listar aqui pra não esquecer)

- **ADR-0073-amendment-1**: Z-before-YSort no sort pipeline §5.2 (já
  implementado §7, falta o doc).
- **ADR-0070-amendment-5**: sampling CPU-tail (já impl Phase 5).
- **ADR-0070-amendment-6**: uv_xform GPU @location15 (já impl).
- **ADR-0070-amendment-7**: clip_group+clip_meta CPU-tail (esta fase).
- **ADR-0074-amendment-1**: ClipChildren impl = stencil (não backbuffer).

---

## 4. Riscos / gates / anti-colisão

### 4.1 Registro dos componentes
Conferir se `ClipChildren`/`MaskInteraction`/`VisibilityLayer`/
`OnScreenEnabler` estão no `register_ecs_components`
([`crates/ph2d-ecs/src/scene/registry.rs`](../crates/ph2d-ecs/src/scene/registry.rs)).
Hoje são **20** (4 base + 14 W3 sorting + UvTransform + ...). Se faltam, somar +
re-lockar a contagem nos gates `ph2d-script` E `ph2d-render` (a W2/W3 já
pegou: `cargo check` ESCONDE essas contagens — rodar `nextest --workspace`).

### 4.2 Gates a rodar no fechamento
- `nextest --workspace` (não só `-p`): pega registry counts, pod_size,
  surface, LOC caps, no_literal_color, hr15, no_magic_numeric, cook_hash.
- `naga` compile (sprite.wgsl novo entry-point stencil-mark).
- clippy `--all-targets` + fmt.
- O novo `clip_children_regression`.

### 4.3 Anti-colisão git
`git add -- <só meus paths W3>`; **NUNCA** `-A`/`git add .`/`stash`.
`git status` antes de cada stage; se houver `M`/`??` alheio
(`.vscode/settings.json`, `docs/UI_Fonts/`, `test_strip`, outros
`docs/HANDOFF_*`), **não comitar junto**. Commit `--no-verify` em background
(hook estoura timeout 2min). Mensagem termina com
`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
**Não pushar** — reportar commits locais; o Coord faz ship+push.

### 4.4 UI canônica
Zero hex/f32-UI-literal/string hardcoded. Tokens (`Spacing::`, `TypeToken::`,
`ColorToken::`) + markers `// LITERAL-PX-OK:` / `// LITERAL-COLOR-OK:` só
quando justificado. **Strings de UI em INGLÊS** (comentário pode ser pt-BR).

---

## 5. Ordem de execução sugerida

1. **ABI**: RenderInstance amendment-7 (clip_group+clip_meta) + re-lock 3
   gates + `inst()`/IDENTITY consts. `cargo check -p ph2d-render`.
2. **Extract**: clip-group tagging em sim_extract.rs (stack de clip-ancestors,
   single-level). Risco LOC (já tem marker `// ph2d-loc-cap:`).
3. **Pipeline**: stencil texture + 3 pipelines + `fs_stencil_mark` no wgsl.
   naga compile.
4. **render() multi-pass**: extrair `clip_pass.rs`; sub-passes mark/test/draw.
5. **Gate**: `clip_children_regression.rs` headless 4-pixel × 3 modos + cutoff.
   **Smoke do Enio aqui** (ClipChildren visual) ANTES do §8.
6. **§8 section**: toda a stack §2.1 (espelhar §9). Smoke do Enio.
7. Commit em blocos (ABI+extract+pipeline+render = 1; gate = 1; §8 = 1).

## 5-BIS. STATUS 2026-05-30 — Deliverable A FECHADO + COMMITADO

**Commit local `45ab07c`** (não pushado): `feat(sprite): W3 §8 ClipChildren
stencil render (Deliverable A)`. Steps 1–5 da §5 completos e **verificados**:

- **ABI amendment-7**: `RenderInstance` +`clip_group:u32` +`clip_meta:u32`
  (CPU-tail, 176→**184 B**; GPU layout 164 B/12 attrs INTACTO). 3 gates
  re-lockados (pod_size 184, field_count 14→16). Helpers `pack_clip_meta`/
  `clip_role`/`clip_cutoff` + `CLIP_ROLE_*` consts em `sprite.rs`.
- **Extract** (`sim_extract.rs`): `resolve_clip_grouping()` no post-rank
  loop (clip_group = rank do clip-parent +1, single-level/innermost). Os 4
  componentes JÁ estavam no registry (§4.1 era falso alarme — nada a somar).
- **Pipeline** (`pipeline.rs`): 2 variantes stencil (`mark`/`test`) +
  `STENCIL_FORMAT=Stencil8` lazy attachment. **DECISÃO CHAVE**: o limite de
  16 vertex-attrs (loc 0..15) está CHEIO → o `@location(16)` do plano §1.4
  NÃO cabe. Cutoff viaja per-instance via `clip_meta` **repurposed para
  `@location(5)`** (normalmente `tint`, não usado no mark) na mark layout.
  Isso evita o pitfall de uniform-por-grupo (write-ordering no mesmo encoder).
- **Shader** (`sprite.wgsl`): `vs_stencil_mark`/`fs_stencil_mark` +
  `MarkInstanceInput` (loc 2/3/4/6/8/14/15 + 5=clip_meta). discard se
  `texel.a <= cutoff`.
- **render() multi-pass** + **`clip_pass.rs`** (módulo novo, `pub(crate)`):
  pass normal (clip_group==0) → clip pass (1 render-pass, stencil Clear(0),
  ref incremental por span, sem clears inter-grupo). ClipAndDraw desenha a
  cor do mask via test-pipeline (Equal-ref coincide com a silhueta) → só **3
  pipelines, não 4**. DrawRun ganhou `clip_group`+`clip_role` (run key).
- **Gate** `clip_children_regression.rs`: headless 64×64, 4 pixels canônicos
  × 3 modos + cutoff. **PASSANDO em GPU real** (2/2). Pipeline-compile test
  valida as 3 pipelines via naga.

### O que sobra: Deliverable B (§8 UI) — CORREÇÕES ao plano §2 (estava STALE)

⚠️ O plano §2 assumia §8 inexistente. **Já existe** estado parcial:
- `INSP_LIVE_VISIBILITY_SECTION`/`_COLOR` JÁ existem; **`LIVE_SECTION_IDS` JÁ
  tem comprimento 8 com Visibility no índice 1** (NÃO bumpar p/ 9 — a Sampling
  é índice 7; reusar o índice 1 existente). `paint.rs` linha 281-296 já pinta
  a seção 1 chamando `paint_visibility_row` (o toggle Visible).
- `InspectorVisibilityInfo {entity_bits, visible}` JÁ existe (mínimo, dirige o
  toggle Visible acima/dentro da seção). `EditorAction::InspectorVisibilityEdit
  (InspectorVisibilityInfo)` é **tuple-variant** (carrega o info inteiro), NÃO
  `{entity_bits, edit}`. state/sync/snapshots/event/populate JÁ wirados p/ o
  toggle Visible (`INSP_VISIBILITY_CHECK`).
- **Recomendação**: NÃO renomear/quebrar o struct existente. Criar um struct
  IRMÃO p/ o conteúdo da seção (ex.: `InspectorVisibilitySectionInfo`
  {entity_bits, clip_mode u8, mask_mode u8, alpha_cutoff f32, layer_mask u32,
  on_screen bool, rect [f32;4], mixed}) + `VisibilityFieldEdit` enum + um 2º
  action `InspectorVisibilitySectionEdit {entity_bits, edit}` (espelhar §9
  Sampling). Pintar os controles novos DENTRO da seção 1, abaixo do toggle
  Visible (novo `sections/visibility.rs`). Os 2 bugs de smoke (§2.2) seguem
  válidos: registrar segmented/checkbox como **Button** (is_focusable) +
  label em row própria. Componentes (`ClipChildren`/`MaskInteraction`/
  `VisibilityLayer`/`OnScreenEnabler`) já no registry → apply via
  SetComponent/RemoveComponent direto.
- Smoke do Enio (ClipChildren visual) precisa do controle ClipChildren da §8
  p/ setar o modo in-app; até lá, o gate headless É a verificação de A.

## 6. Decisões abertas (perguntar ao Enio se travar)

- **Nesting de clip** (clip dentro de clip): single-level W3 (warn) vs ref
  incremental full. Recomendado: single-level + guard, full em wave futura.
- **OnScreenEnabler**: incluir o Rect2 editor agora ou só o toggle? Spec diz
  collapsible inner — pode ser toggle-only em W3 + Rect2 follow-up.
- **Stencil format**: Stencil8 vs Depth24PlusStencil8 (suporte de adapter).

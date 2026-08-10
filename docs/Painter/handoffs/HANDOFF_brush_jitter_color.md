# HANDOFF — Brush: Randomize Color + Jitter Scale + Jitter Rotate

> ## ✅ STATUS: IMPLEMENTADO (2026-06-23) — commit local, ship pendente
>
> As três features estão prontas, testadas e auditadas e2e. **O que landou:**
> - **Modelo** (`ph2d-painter-brush`): novo módulo `jitter.rs` (RNG splitmix64 consolidado + `per_dab()`
>   + HSV puro, transcendental-free). `BrushSpec` +6 campos; `Dab` +`color`/`rotation`; `dab_at`
>   aplica scale→rotate→H→S→V em ordem FIXA, gated por `allows_jitter()`; `texture::dab_basis` ganhou
>   `extra_rot` (composto só no ramo não-stencil → Stencil ignora de graça).
> - **Tool** (`ph2d-tool-painter`): os 4 paths de stamp injetam `d.color`; o per-pixel + ramped passam
>   `d.rotation` ao `dab_basis`; dispatch desvia os caches quando `has_per_dab_rotation()` (senão a
>   máscara baked ignoraria a rotação → feature morta). Setters + `route_brush_jitter_event` em novo
>   `jitter_settings.rs`; snapshot `BrushSettings` +4 campos. Per-pixel loop extraído p/ `stamp_cache.rs`
>   (LOC cap).
> - **UI** (`ph2d-editor-core` ids + `ph2d-panel-painter-layers`): 6 ids novos + slice
>   `PAINTER_BRUSH_RANDOMIZE_SLIDERS`; populate registra os 5 sliders + enable; paint_brush pinta a
>   seção "Randomize Color" (toggle + H/S/V quando ligado); paint_stroke pinta "Scale" (sempre) +
>   "Rotate" (só com textura); event.rs forwarda via slice-contains + Click.
>
> **Decisões padrão-ouro tomadas** (delegadas pelo §0 do handoff):
> - Os 3 jitters respeitam `allows_jitter()` (DragDot/Anchored opt-out, como o position-jitter) →
>   preserva 2 invariantes de graça: tudo-0 == baseline bit-idêntico, e DragDot/Anchored = zero draw.
> - **Per-dab** (não per-stroke). **Ramp vence Randomize Color** (LUT sobrescreve `spec.color`).
> - Jitter Rotate só compõe fora do Stencil; UI da row "Rotate" só aparece com textura (não-morta).
>
> **Verde:** 130+21+119 lib-tests · e2e `randomize_*` (wiring + pixels variados) · gates
> workspace/panel/widget LOC + tool_contract + **panel_wiring_parity** + behavioral + clippy `--all-targets`.
> Todos os arquivos ≤600. **Próximo:** smoke manual do Enio + ship.
>
> _O texto abaixo é o briefing original da implementação (mantido como referência)._

---

**Para:** o agente que vai implementar.
**De:** sessão anterior (Texture Layer + perf).
**Escopo:** três features per-dab do **brush** (`ph2d-painter-brush` + tool + painel). NÃO é o texture-layer (esse já está pronto).
**Status do repo:** `main` local, verde (tests + clippy + gates). Commit local, **sem push** (o Enio dá ship).

---

## 0. O que implementar (e a referência)

Da imagem do Blender que o Enio anexou (painel **Color → Randomize**):

1. **Randomize Color** — uma subseção com um **toggle de enable** (a checkbox do header) + 3 sliders: **Hue / Saturation / Value**. Cada um é uma quantidade `0..1` de jitter aplicada por dab à cor do pincel no espaço HSV. (Os 2 ícones por linha no Blender são "input de pressão" + "curve editor" — **fora de escopo nesta entrega**; só os 3 valores + o enable.)
2. **Jitter Scale** — **NÃO existe no Blender.** Quantidade `0..1` de jitter no **tamanho** (raio) por dab.
3. **Jitter Rotate** — **NÃO existe no Blender.** Quantidade `0..1` de jitter na **rotação** por dab (só visível com textura — dabs redondos são isotrópicos).

**Semântica recomendada:** **per-dab** (cada dab um pouco diferente → efeito de mídia natural / speckle). É a opção mais útil e visível. (Per-stroke — uma cor/escala aleatória por traço — é alternativa; se quiser, exponha depois. Comece per-dab.)

**Referência canônica = o próprio repo.** O brush já tem o padrão completo de "param per-dab determinístico": o **jitter de posição** (`Brush.jitter` / `jitter_absolute_px`) e o **Random** da textura (`random_angle`). Copie esse padrão. O recorte do Blender está em `reference/blender-texture-paint/` (GPL-2.0 → **clean-room**, nunca portar expressão; só comportamento). **Não** afirme nomes de campos DNA do Blender sem grep no recorte (memory `feedback_no_industrial_claims_without_verification`).

---

## 1. Lei de ferro: determinismo (HR-5)

`ph2d-painter-brush` é `#![forbid(unsafe_code)]` e **transcendental-free** (só `+ - * /`, `floor`, `abs`, `sqrt`; rotação via vetor unitário, nunca ângulo). Há um gate que grepa `sin|cos|tan|atan2|exp|pow` alternados (memory `feedback_determinism_sweep_grep_all_transcendentals`). Portanto:

- **RNG:** use **`Stroke::next_f32()`** (splitmix64, `stroke.rs:543`). NÃO use `rand`, `Math.random`, nem `thread_rng`. O stroke já carrega `rng: u64` semeado por-stroke (`stroke.rs:54`, `Stroke::new(.., seed)` em `stroke.rs:94`).
- **Ordem dos draws importa.** Saque os randoms numa ordem FIXA por dab (ex.: scale → rotate → H → S → V) e **nunca reordene** — senão muda o resultado e quebra os golden/replay-hash. Documente a ordem onde sacar.
- **HSV↔RGB** é puro min/max/div (sem transcendental) → escreva um helper inline pequeno no brush crate (≈15 linhas). NÃO adicione dep `ph2d-color` só por isso (o crate é proposital-mente magro). (Se preferir, `ph2d-color::color_ramp::convert` tem `rgb_to_hsv`/`hsv_to_rgba`, mas são `pub(crate)` lá — exportá-los é mais churn que inline.)
- **Rotação** via `rotate_by_degrees(deg)` (texture.rs) — já é DEG_STEP^deg (matriz de 1°, transcendental-free). Para jitter de rotação, gere um deslocamento de graus aleatório e rode por ele com o mesmo mecanismo.

---

## 2. Onde cada coisa engata (file:line)

### 2a. Modelo do brush — `crates/ph2d-painter-brush/src/spec.rs`
`BrushSpec` (`spec.rs:31`, `#[derive(Clone, Copy)]`). Adicione os campos + defaults (`spec.rs:105`):
```
color_jitter_enabled: bool   // default false
color_jitter_hue: f32        // 0..1, default 0
color_jitter_sat: f32        // 0..1, default 0
color_jitter_val: f32        // 0..1, default 0
jitter_scale: f32            // 0..1, default 0  (PH2D, não-Blender)
jitter_rotate: f32           // 0..1, default 0  (PH2D, não-Blender)
```
Mantenha `Copy` (são `f32`/`bool` → ok). Atualize o teste `defaults_are_sane`.

### 2b. O dab — `crates/ph2d-painter-brush/src/stroke.rs`
- **`Dab` struct (`stroke.rs:31`)** carrega hoje `{center, radius_px, coverage}`. Adicione:
  - `color: [f32; 3]` — a cor já jitterada deste dab (Randomize Color).
  - `rotation: [f32; 2]` — vetor unitário de rotação extra deste dab (Jitter Rotate); identidade = `[1.0, 0.0]`.
- **`dab_at()` (`stroke.rs:501`) é O HOOK central** — tem `self.rng`, `self.spec`, e computa radius/coverage. Aqui você:
  - **Jitter Scale:** `radius *= 1.0 + (self.next_f32()*2.0 - 1.0) * self.spec.jitter_scale;` depois `radius = radius.max(0.5);` (não deixe colapsar). (`next_f32` é `[0,1)`; `*2-1` → `[-1,1)`.)
  - **Jitter Rotate:** compute um vetor unitário de rotação extra. Reuse `rotate_by_degrees` (em `texture.rs`; exponha-o p/ `stroke.rs` se preciso): `let deg = (self.next_f32()*2.0-1.0) * self.spec.jitter_rotate * 180.0; let rotation = rotate_by_degrees(deg.round() as i32 ...);`. Guarde em `Dab.rotation`. Se `jitter_rotate == 0` → `[1.0, 0.0]`.
  - **Randomize Color:** se `color_jitter_enabled`, RGB→HSV de `self.spec.color`, aplique offsets: `h += (next*2-1)*hue; s += (next*2-1)*sat; v += (next*2-1)*val;` (hue wrap `0..1`, s/v clamp `0..1`), HSV→RGB → `Dab.color`. Senão `Dab.color = self.spec.color`.
- **Jitter desligado para alguns métodos:** o jitter de posição respeita `stroke_method.allows_jitter()` (`stroke.rs:517`, DragDot/Anchored desligam). Decida se Scale/Rotate/Color seguem a mesma regra (provável: Color sim per-dab sempre; Scale/Rotate idem ao position-jitter). Documente.
- **Anchored / shapes (`dab_at` é chamado de vários sites):** confira `ellipse.rs`/`polygon.rs`/`curve.rs` — eles emitem dabs também; garanta que pegam a cor/rotação per-dab (ou herdam o default). Rode os testes de `stroke/tests.rs`.

### 2c. Rotação da textura — `crates/ph2d-painter-brush/src/texture.rs`
`dab_basis(s, dab_dir, rng, canvas)` (`texture.rs:401`) constrói o frame `u`/`v` da textura. O Jitter Rotate precisa **compor** `Dab.rotation` no `u` resultante. Duas opções (escolha uma):
- **(A) passar a rotação extra p/ `dab_basis`:** adicione param `extra_rot: [f32;2]` e componha `u` (rotação 2D = multiplicação complexa: `u' = [u.x*r.x - u.y*r.y, u.x*r.y + u.y*r.x]`). Mais limpo (o frame nasce certo).
- **(B) método em `TexDabBasis`:** `fn rotated_by(self, r:[f32;2]) -> Self` (os campos `u`/`v` são privados — método dentro do módulo). O tool chama após `dab_basis`.
Stencil tem frame fixo (`dab_basis` early-returns p/ stencil) — Jitter Rotate **não** se aplica a Stencil (igual Rake/Random). Mantenha.

### 2d. O tool stampa — `crates/ph2d-tool-painter/src/tool/paint/stamp_cache.rs`
Há **3 loops de stamp** que constroem um `BrushSpec` per-dab sobrescrevendo `radius_px` (`stamp_dabs_cached` ~`:78`, `stamp_dabs_canvas_cached` ~`:128`, `stamp_dabs_ramped` ~`:205`). Em **cada um**:
- Sobrescreva também a cor: `let spec = BrushSpec { radius_px: d.radius_px, color: d.color, ..*brush };`. A cor entra no stamp via `spec.color` (o `blit_stamp`/`stamp_dab` lê `spec.color`).
- Passe `d.rotation` para o `dab_basis(...)` (`stamp_cache.rs:209`) — opção A acima.
⚠️ **Nota ramp:** no caminho `stamp_dabs_ramped` a cor vem do **Color Ramp** (per-texel), não de `spec.color`. Decida a interação: Randomize Color + Ramp ligados ao mesmo tempo — provavelmente o Ramp vence (ele já define a cor por-texel) e o Randomize Color só age quando o Ramp está **off**. Documente e teste essa precedência.

---

## 3. UI — checklist COMPLETO (aqui é onde a entrega passada quase morreu)

A seção Texture/Stroke do painel é toda **fixed-id widgets, tool-global**, lidos de um snapshot `BrushSettings`. **Cada controle novo precisa de TODOS os passos abaixo** — pular um = controle morto/silencioso. Uma auditoria multiagente recente pegou exatamente isso (botão "+ Texture" pintado mas sem slot em `populate.rs` → feature inalcançável; memory `feedback_panel_populate_register` + `project_texture_layer_design` LIÇÃO 2).

Para CADA widget novo (3 sliders Color + enable Color + 2 sliders Jitter):

1. **ID fixo** em `crates/ph2d-editor-core/src/ids/chrome/painter.rs` via `hash_node_id("painter_brush.color_jitter_hue")` etc. (siga o padrão `PAINTER_BRUSH_TEXTURE_*`). Slider → e o evento `SetValue`; toggle de enable → `Click`.
2. **populate** em `crates/ph2d-panel-painter-layers/src/populate.rs`:
   - sliders → array `brush_sliders` (registra `InteractiveState::Slider`).
   - enable toggle → array de botões (`InteractiveState::Button`).
   - **Há um teste de regressão** (`populate::tests::action_toolbar_buttons_have_store_slots`) — estenda-o p/ cobrir o enable novo.
3. **Paint** a seção. **Color** é uma seção nova no Brush body (`crates/ph2d-panel-painter-layers/src/paint_brush.rs` — `paint_brush_body`); hoje a cor é só o swatch (`paint_brush::paint_brush_mode`). Crie um `paint_color_section` (mirror de `paint_texture_section`): o swatch + um `paint_toggle_row(... "Randomize Color")` + (quando ligado) 3 `paint_param_row` Hue/Sat/Value. **Jitter Scale/Rotate** vão na **Stroke section** (`crates/ph2d-panel-painter-layers/src/paint_stroke.rs`), ao lado do Jitter de posição existente — 2 `paint_param_row` a mais. Helpers de row: `paint_param_row`, `paint_toggle_row` em `paint_brush.rs`.
4. **Event forward** em `crates/ph2d-panel-painter-layers/src/event.rs` → `try_apply_brush_event`:
   - sliders novos → adicione os ids ao guard `ValueChanged` que emite `PanelEvent::SetValue` (perto de `:519`).
   - enable toggle → ao guard `Click` que emite `PanelEvent::Click` (perto de `:424`).
5. **Tool setters** em `crates/ph2d-tool-painter/src/tool/paint/brush_settings.rs` (a fonte única de clamp; mirror de `set_brush_jitter_norm`/`set_brush_texture_param_norm`): `set_brush_color_jitter(slot, t)` (ou 3 setters), `set_brush_jitter_scale(t)`, `set_brush_jitter_rotate(t)`, `toggle_brush_color_jitter_enabled()`. Cada um escreve em `self.paint.brush.*`.
6. **handle_panel_event** em `crates/ph2d-tool-painter/src/tool/trait_impls.rs` — roteie os ids novos:
   - `PanelEvent::SetValue(id, v)` → o setter (no bloco grande de `else if id == PAINTER_BRUSH_*`).
   - `PanelEvent::Click(id)` (enable) → `toggle_brush_color_jitter_enabled()` (no match de Click).
   - ⚠️ **Não** vão pro `route_texture_layer_event` (esse só rouba `PAINTER_BRUSH_TEXTURE_*`). Seus ids novos são `PAINTER_BRUSH_COLOR_*` / `PAINTER_BRUSH_JITTER_*` → caem no handler do brush normal. Confirme que o guard de texture-layer não os intercepta.
7. **Snapshot** em `brush_settings.rs` → `BrushSettings` (struct `:82`) ganha os campos + o builder `brush_settings()` (`:214`) os preenche de `self.paint.brush`. O painel lê o snapshot p/ posicionar os sliders.

**Não precisa de IconId novo** (sem ícone novo) — a menos que você dê um ícone à seção Color; se der, IconId variant em ordem alfabética senão quebra TODOS os ícones (memory `feedback_new_tool_icon_needs_iconid`).

**UI em inglês** sempre (labels "Randomize Color", "Hue", "Jitter Scale", …), mesmo o Enio descrevendo em pt-BR (memory `feedback_app_ui_english_only`).

---

## 4. Testes (unit-verde ≠ funciona no produto)

- **Brush unit** (`stroke/tests.rs` ou novo): com `jitter_scale>0`, os dabs de um traço têm raios **variados** mas **determinísticos** (mesmo seed → mesmos raios); idem cor (Randomize) e rotação. Com tudo `0`, dabs idênticos ao baseline (regressão).
- **HSV roundtrip:** RGB→HSV→RGB ≈ identidade (dentro de 1/255) p/ uma grade de cores.
- **e2e no tool** (OBRIGATÓRIO — a lição que mais dói aqui): em `crates/ph2d-tool-painter/src/tool/paint/tests.rs`, dirija `handle_panel_event(PanelEvent::SetValue(PAINTER_BRUSH_JITTER_SCALE, ...))` e `Click(PAINTER_BRUSH_COLOR_JITTER_ENABLE)`, depois rode um traço (begin/extend via os métodos de canvas-pointer ou `Stroke` direto) e prove que (a) o setter chegou no `self.paint.brush`, (b) os dabs saíram variados. Isso pega controle-morto (não-registrado / não-roteado), que unit não pega (memories `feedback_tool_unit_green_integration_dead`, `feedback_panel_populate_register`).

---

## 5. Velocidade, gates, disciplina

- **Inner loop:** `export CARGO_TARGET_DIR=...slot...; cargo check -p ph2d-painter-brush` (warm slot: `bash scripts/slot-seed.sh slot-1`). NÃO use `--workspace`.
- **Caps de LOC** (600/arquivo, 200/fn): `spec.rs` (272), `stroke.rs` (595 — **APERTADO**, +campos no Dab + lógica no dab_at pode estourar → considere extrair um helper `jitter.rs` no brush crate), `event.rs`/`paint.rs` (no cap 600 — reaproveite linhas, comprima comentários se preciso), `brush_settings.rs` (590), `trait_impls.rs` (600 — **no cap**, cuidado). **`stack.rs` está CONGELADO em 630 — não toque.** `fmt` re-expande arrays → re-cheque LOC DEPOIS do fmt. `rustfmt --edition 2024` (let-chains).
- **Fechamento (1× no fim):** `cargo test -p <crates tocadas> --lib` + `cargo clippy -p ... --all-targets` + os gates `architecture_panel_loc_cap` / `architecture_workspace_file_loc_cap` / `architecture_widget_loc_cap` / `architecture_tool_contract_surface`. Tudo verde antes de commit.
- **Isolamento:** brush + tool + painel + ids — **tudo na sua pasta** (mesmas crates da Texture Layer). Se precisar de algo fora, PARE e reporte.
- **Git:** `git add -- <só seus paths>`; `git commit --no-verify -m "..." -- <paths>`; **sem push** (Enio shippa). Co-author trailer `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

---

## 6. Resumo dos hook points (cola rápida)

| Feature | Hook principal | Dab field | Stamp |
|---|---|---|---|
| Jitter Scale | `stroke.rs:501 dab_at` (multiplica `radius`) | usa `radius_px` (já existe) | nenhum (raio já per-dab) |
| Jitter Rotate | `stroke.rs:501 dab_at` (gera `rotation` via `rotate_by_degrees`) | **+`rotation:[f32;2]`** | `stamp_cache.rs:209` compõe em `dab_basis` |
| Randomize Color | `stroke.rs:501 dab_at` (RGB→HSV jitter→RGB) | **+`color:[f32;3]`** | os 3 loops setam `color: d.color` no spec per-dab |

RNG = `Stroke::next_f32()` (`stroke.rs:543`), ordem de saque FIXA. Tudo transcendental-free.

Boa implementação. Qualquer ambiguidade de semântica (per-dab vs per-stroke, precedência Ramp×Randomize, Scale/Rotate respeitam `allows_jitter`), **decida no padrão-ouro e reporte a decisão** — não trave perguntando (memory `feedback_decide_dont_ask_gold_standard`).

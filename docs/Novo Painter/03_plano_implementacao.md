# 03 — Plano de implementação do Brush Engine (paridade Procreate)

> **Cada feature do Procreate = uma etapa.** Índice passo-a-passo até o quadro total de features do Brush
> Studio. Paridade primeiro; diferencial depois. Baseado na [referência](02_referencia_parametros_procreate.md)
> e na [base teórica](01_pesquisa_teorica_e_literatura.md).

## Como ler este plano

Cada parâmetro é uma etapa com **3 dimensões de status**, porque o modelo de dados já existe (ADR-0044) mas
a avaliação e a UI não:

- **[M]** Modelo — o campo existe no `Brush`? (`✅`/`⚠️ diverge`/`❌ falta` — da [referência §resumo](02_referencia_parametros_procreate.md#resumo-dos-gaps-de-modelo-o-que-faltadiverge-no-nosso-brush))
- **[E]** Engine — o parâmetro **afeta o render** no dab pipeline? (a verificar em W0.1)
- **[U]** UI — existe controle no painel Brush Studio? (hoje só 5 das 14 seções existem)

Uma etapa só está **fechada** quando M+E+U estão verdes e há teste de paridade headless. `[ ]` = pendente.

---

## W0.1 RESULTADO — Matriz de cobertura `[E]` (auditoria 2026-06-16)

> Varredura de produção (exclui tests/defs/UI) do dab pipeline: `stamp_scheduler/{mod,advance}.rs`
> (geometria: brush+path → `Vec<Stamp>`), os `FLAG_*` em `stamp.rs`, e o **render vivo**.

**Fato 1 — o render VIVO é CPU.** O tool chama `apply_stamps_wash`/`apply_stamps_buildup`/
`apply_stamps_with_options` ([`lifecycle.rs:274-303`](../../crates/ph2d-tool-painter/src/tool/lifecycle.rs#L274))
→ [`cpu_render/mod.rs`](../../crates/ph2d-painter-brush/src/cpu_render/mod.rs). O GPU `StampPipeline` +
`shader/stamp.wgsl` existe e é naga-validado, mas **não é despachado** (revertido a CPU-residente pós-ADR-0096).
**Consequência:** um param "wired" só no `stamp.wgsl` está **dormente** no caminho vivo. Notavelmente
`Stamp.tilt`/`azimuth`/`barrel_roll` são consumidos pelo WGSL mas **não** pelo `cpu_render` → tilt/azimuth/barrel
não têm efeito de render hoje.

**Fato 2 — Mixbox/pigment está VIVO.** `cpu_render` aplica `pigment_mix::prepare_pigment` + `mix_prepared`
gated por `pigment_mode==1` ([`cpu_render/mod.rs:321,437,616,735`](../../crates/ph2d-painter-brush/src/cpu_render/mod.rs#L321)).
A mistura espectral de cor já funciona no caminho wash/blend.

**Fato 3 — Wet Mix está 100% MORTO.** O scheduler faz `s.wet_amount = 0.0; // T-wet-mix W7+`
([`advance.rs:829`](../../crates/ph2d-painter-brush/src/stamp_scheduler/advance.rs#L829)). Nenhum dos 8 sliders
de Wet Mix é lido. O shader/cpu lêem `wet_amount` mas sempre recebem 0.

### Matriz por categoria (✅ vivo · 🟡 parcial/dormente · ❌ morto · `—` não no modelo)

| Categoria | ✅ VIVO (avaliado no caminho CPU) | 🟡 parcial / dormente | ❌ MORTO (campo existe, não avaliado) |
|---|---|---|---|
| **Stroke Path** | spacing · spacing_jitter · jitter_lateral · falloff | — | — (falta `jitter_linear` no modelo) |
| **Stabilization** | streamline_amount · stabilization · motion_filtering_amount | — | streamline_pressure · motion_filtering_expression |
| **Taper** | length start/end · size · opacity (via `taper_factors`/`start_taper`) | taper_pressure_link (bool) | — (faltam tip/classic/split touch) |
| **Shape** | source · count · count_jitter · scatter · rotation_follow · randomized · flip_x · flip_y | — | input_style · **roundness** · pressure_roundness · tilt_roundness · vertical_jitter · horizontal_jitter · filtering |
| **Grain** | source · scale · depth · behavior (Moving flag) | — | movement · zoom · rotation · depth_min · depth_jitter · offset_jitter · blend_mode · brightness · contrast · filtering |
| **Rendering** | rendering_mode (6) · pigment_mode (Mixbox) · flow · accumulate (wash↔buildup) | — | wet_edges · burnt_edges · burnt_edges_mode · luminance_blending · alpha_threshold · stroke_blend_mode_index · edge_intensity |
| **Wet Mix** | — | — | **TODOS** (dilution · load · attack · pull · grade · blur · blur_jitter · wetness_jitter) — `wet_amount=0.0` hardcoded |
| **Color Dynamics** | stamp: hue · saturation · lightness · darkness · secondary | — | stroke_* (5) · pressure_* (4) · tilt_* (4) · barrel_* (4) |
| **Dynamics** | speed_size · speed_opacity · speed_spacing · jitter_size · jitter_opacity | — | — (categoria 100% viva) |
| **Apple Pencil** | pressure_curve · pressure_targets | tilt **input** empacotado mas só consumido no WGSL (dormente) | tilt_curve · tilt_targets · barrel_roll_curve · barrel_targets · hover_* |
| **Properties** | — | max/min size/opacity (clamp tool-side) · smudge_pull (smudge tool) | orient_to_screen |
| **Color (Mixbox)** | `pigment_mix` vivo no cpu_render | — | — |

### Veredicto e re-priorização

O engine vivo já entrega um **MVP de mark respeitável**: carimba ao longo do path (spacing/jitter/falloff),
com taper, colocação de shape (count/scatter/flip/rotation), todas as Dynamics, pressão via curva, os 6
Rendering Modes, **mistura de cor Mixbox**, jitter de cor por-stamp, e grain básico.

**Os grandes buracos (ordem de impacto):**
1. **Wet Mix inteiro** (W7) — `wet_amount=0.0`. É o coração do mixer-brush; é o maior trabalho e o maior valor.
2. **Shape: roundness/elipse + filtering + input_style** (W2) — sem elipse, brushes caligráficos não existem.
3. **Grain detalhado** (W4) — 10 params mortos (movement/zoom/depth_jitter/blend_mode/brightness/contrast…).
4. **Rendering edges** (W3) — wet_edges/burnt_edges/luminance/alpha_threshold (FLAGs nunca setados).
5. **Color dynamics não-stamp** (W8) — stroke/pressure/tilt/barrel (16 params).
6. **Tilt/barrel como resposta de brush** (W9) — input chega mas só o WGSL (dormente) usa; cpu_render ignora.
7. **2 params de Stabilization** (streamline_pressure, motion_filtering_expression).

**Decisão de arquitetura que a auditoria força (para o ADR-0097, W0.0):** o GPU `StampPipeline` está
validado mas morto. Ou (A) **consolidar no CPU** como fonte-da-verdade e tratar o WGSL como paridade futura,
ou (B) **despachar o GPU** (retoma parte do plano [wise-seeking-gem](../../), sem fluido). Recomendação:
**(A) CPU-first** — menos superfície, destrava Wet Mix/grain/edges no caminho vivo já; GPU vira otimização
de perf depois (brush grande/4K), reconciliado por paridade ULP. Decidir no ADR.

> As marcações `[E?]` por-etapa abaixo ficam **superseded por esta matriz**.

## Crates e papéis

| Crate | Papel | Contrato congelado (§6) |
|---|---|---|
| [`ph2d-painter-brush`](../../crates/ph2d-painter-brush/) | modelo `Brush` + dab pipeline + GPU `StampPipeline` | `Brush≤168`, `Stamp=96B align(16)`, `RenderingMode=6` |
| [`ph2d-color`](../../crates/ph2d-color/) | Mixbox/K-M (`pigment_space`), espaços de cor | `ColorProfile=8` |
| [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/) | wiring do tool, input, lifecycle | `Tool=11`, `PainterParams≤12` |
| [`ph2d-panel-brush-studio`](../../crates/ph2d-panel-brush-studio/) | UI do Brush Studio (14 seções) | UI canônica (tokens/i18n, HR-15) |
| [`ph2d-painter-stroke`](../../crates/ph2d-painter-stroke/) | persistência (`.ph2d-painter`, SCHEMA_VERSION=3) | `PaintProject≤12` |

**Regra de ouro:** mexer na superfície de `Brush`/`Stamp`/`RenderingMode` = **Coord-only + ADR-amendment** +
gate `architecture_painter_contract_surface`. Há **1 slot top-level de headroom** no `Brush` (cap ≤ 14).
UI sempre em **inglês** (labels/toasts), via tokens (HR-15).

---

## Wave 0 — Fundação *(Coord-only + ADR; pré-requisito de tudo)*

- [x] **W0.0 — ADR-0097: arquitetura do Brush Engine — ACEITO (CPU-first).** Define o dab pipeline como fonte-da-verdade do mark:
  `evaluate_stroke(brush, path, input) → Vec<Stamp>` (scheduler determinístico CPU, HR-5) → render
  (CPU `apply_stamps_*` + GPU `StampPipeline`). Formaliza os **3 estágios ortogonais**: (1) **geometria**
  (path→dabs: spacing/jitter/taper/shape/dynamics), (2) **cobertura/composição** (build-up vs wash, os 6
  Rendering Modes, grain, linear-light), (3) **cor** (Mixbox/K-M, color dynamics, wet mix). Ratifica
  paridade-Procreate como spec. Supersede o `apply_stamps` ad-hoc. **Coord-only.**
- [ ] **W0.1 — Auditoria de cobertura (a matriz real [E]).** Para cada um dos ~90 parâmetros, grep+leitura do
  caminho `queue_pointer`/`apply_stamps_*`/shaders: classificar avaliado / parcial / morto. Produz a tabela
  `[E]` que este plano deixa "a verificar". **Entregável: matriz de status que prioriza as ondas.**
- [ ] **W0.2 — Consolidar o dab scheduler.** Um único `evaluate_stroke` determinístico (substitui a lógica
  espalhada em `lifecycle.rs`). Resample por comprimento-de-arco; emite `Stamp` (96B). Testes de determinismo
  (HR-5, cross-OS).
- [x] **W0.3 — Estágio de cor (Mixbox) — AUDITADO (2026-06-16, sem risco).** `pigment_mix.rs` é uma impl
  **clean-room espectral** (24 bandas: reconstrução por base Gaussiana → mistura Kubelka–Munk `K/S=(1−R)²/2R`
  → integração de volta a RGB; endpoints exatos via `4·t·(1−t)` + residual round-trip por-cor). **NÃO usa a
  LUT do scrtwpns/mixbox** (a única referência a `scrtwpns` é um comentário/URL; a "LUT" interna é um cache de
  perf 17³ RGB próprio). **Sem risco de licença CC BY-NC** — PH2D é dono da mistura, det-mode-portable (módulo
  `powf` bit-identity, com follow-up Q-fixed-point já documentado). **Veredicto: usar `pigment_mix` como o
  estágio de cor de W7/W8 sem mudança.** A ortogonalidade cobertura(linear)↔matiz(K-M) já é respeitada.
- [ ] **W0.4 — Esqueleto da UI do Brush Studio.** Painel com as **14 seções** colapsáveis (espelha
  `ph2d-panel-inspector`/gallery, HR-15). Hoje existem 5 (`stroke/shape/rendering/color_dynamics/dynamics`);
  scaffold das 9 faltantes (stabilization, taper, grain, wet_mix, pencil, properties, preview, about, materials-stub).

**Verificação W0:** headless GPU (`cargo test --features gpu -- --ignored`, Metal) — naga validate do pipeline
consolidado; determinismo do scheduler; paridade Mixbox CPU↔WGSL.

---

## Wave 1 — Stroke Path + espinha de input *(o esqueleto de qualquer brush)*

> Um brush precisa carimbar ao longo de um path com pressão antes que qualquer outro parâmetro importe.

- [ ] **W1.1 — Spacing** `[M✅][E?][U parcial]` — passo = `spacing × diâmetro`; spacing baixo = linha lisa.
- [ ] **W1.2 — Spacing Jitter** `[M✅][E?][U?]` — variabilidade aleatória do passo.
- [ ] **W1.3 — Jitter Lateral** `[M✅][E?][U?]` — deslocamento perpendicular do carimbo.
- [ ] **W1.4 — Jitter Linear** `[M❌][E❌][U❌]` — **gap de modelo:** adicionar `stroke_path.jitter_linear`
  (`plotJitterLongitudinal`); deslocamento na direção do traço. ⚠️ toca `Brush` cap → ADR-amendment.
- [ ] **W1.5 — Fall Off** `[M✅][E?][U?]` — fade de opacidade ao longo da pincelada.
- [ ] **W1.6 — Apple Pencil: curva de pressão** `[M✅][E?][U❌]` — editor de curva (x=pressão, y=efeito);
  remapeia pressão crua antes de dirigir os targets.
- [ ] **W1.7 — Pressure → Size / Opacity / Flow** `[M⚠️][E?][U❌]` — os 3 targets essenciais. Decidir encoding:
  manter bitmask+curva compacto **ou** expandir p/ amounts individuais (paridade exata). ⚠️ se expandir, toca `Brush`.

**Verificação W1:** parity headless do scheduler (spacing/jitter determinísticos); manual: traço com pressão
real-time, taper de pressão visível.

---

## Wave 2 — Shape *(a silhueta do mark)*

- [ ] **W2.1 — Shape Source** `[M✅][E?][U?]` — máscara importável; Shape Editor.
- [ ] **W2.2 — Input Style** (Touch/Azimuth/Azimuth+Roll) `[M✅][E?][U?]`.
- [ ] **W2.3 — Rotation (vs direção do traço)** `[M⚠️][E?][U?]` — **diverge:** virar `bool`→slider ±100% (`shapeRotation`).
- [ ] **W2.4 — Scatter** `[M✅][E?][U?]` — rotação aleatória por carimbo.
- [ ] **W2.5 — Count** `[M✅][E?][U?]` — até 16 carimbos por ponto.
- [ ] **W2.6 — Count Jitter** `[M✅][E?][U?]`.
- [ ] **W2.7 — Randomised** `[M✅][E?][U?]` — rotação aleatória no início.
- [ ] **W2.8 — Flip X / Flip Y** `[M✅][E?][U?]`.
- [ ] **W2.9 — Roundness: squash (elipse)** `[M✅][E?][U?]`.
- [ ] **W2.10 — Roundness: Angle base** `[M❌][E❌][U❌]` — **gap:** adicionar `shape.shape_angle` (`shapeAngle`). ⚠️ toca `Brush`.
- [ ] **W2.11 — Pressure → Roundness** `[M✅][E?][U?]`.
- [ ] **W2.12 — Tilt → Roundness** `[M✅][E?][U?]`.
- [ ] **W2.13 — Roundness Vertical / Horizontal Jitter** `[M✅][E?][U?]`.
- [ ] **W2.14 — Shape Filtering** (None/Classic/Improved) `[M✅][E?][U?]`.

**Verificação W2:** parity de um carimbo isolado (scatter/count/roundness/flip determinísticos por seed).

---

## Wave 3 — Rendering *(o "feel" — a acumulação de alpha; o mais distintivo)*

- [ ] **W3.1 — Os 6 Rendering Modes** `[M✅ CONGELADO][E?][U existe]` — Light/Uniform/Intense/Heavy Glaze +
  Uniform/Intense Blending. **Garantir o split build-up↔wash** (§1/§6 da teoria): Glaze = cap único por
  pincelada (buffer de cobertura), Blending = build-up contínuo + ativa Wet Mix. Verificar que os 6 modos
  diferem em opacidade baixa (`1−(1−a)^n` vs `a`).
- [ ] **W3.2 — Flow** `[M✅][E?][U?]` — força por-dab (acumula dentro da pincelada nos modos build-up).
- [ ] **W3.3 — Wet Edges** `[M⚠️][E?][U?]` — **diverge:** virar `bool`→slider 0–100% (`wetEdgesAmount`); blur/sangramento de borda. ⚠️ toca `Brush`.
- [ ] **W3.4 — Burnt Edges** `[M⚠️][E?][U?]` — **diverge:** `bool`→slider 0–100% (`burntEdgesAmount`); color-burn de borda. ⚠️ toca `Brush`.
- [ ] **W3.5 — Burnt Edges Mode** `[M✅][E?][U?]` — blend mode do burnt edge.
- [ ] **W3.6 — Blend Mode (pincelada)** `[M✅][E?][U?]` — blend mode da pincelada inteira (enum de 28).
- [ ] **W3.7 — Luminance Blending** `[M✅][E?][U?]` — mistura luminância (gamma correct).
- [ ] **W3.8 — Alpha Threshold** `[M✅][E?][U?]` — binariza o alpha.

**Verificação W3:** parity headless build-up vs wash (a desigualdade `1−(1−a)^n`); manual: brush de opacidade
baixa mostra a diferença entre os 6 modos; wet/burnt edges visíveis ao sobrepor.

---

## Wave 4 — Grain *(textura dentro do shape)*

- [ ] **W4.1 — Grain Source** `[M✅][E?][U❌]` — bitmap cinza + Grain Editor/tiling.
- [ ] **W4.2 — Movement mode: Moving vs Texturized** `[M✅][E?][U❌]` — **o eixo de coords** (local-da-pincelada
  vs canvas). Moving é distintivo do Procreate (§4 da teoria).
- [ ] **W4.3 — Movement (slider, só Moving)** `[M✅][E?][U❌]`.
- [ ] **W4.4 — Scale** `[M✅][E?][U❌]`.
- [ ] **W4.5 — Zoom (Follow Size ↔ Cropped, só Moving)** `[M✅][E?][U❌]`.
- [ ] **W4.6 — Rotation (só Moving)** `[M✅][E?][U❌]`.
- [ ] **W4.7 — Depth** `[M✅][E?][U❌]` — força da textura sobre a cor.
- [ ] **W4.8 — Minimum (Depth Min)** `[M✅][E?][U❌]` — piso de contraste.
- [ ] **W4.9 — Depth Jitter (só Moving)** `[M✅][E?][U❌]`.
- [ ] **W4.10 — Offset Jitter (só Moving)** `[M✅][E?][U❌]` — quebra o tiling.
- [ ] **W4.11 — Blend Mode** `[M✅][E?][U❌]` — Multiply/Subtract/Height etc. (modula o alpha — §4 da teoria).
- [ ] **W4.12 — Brightness / Contrast** `[M✅][E?][U❌]`.
- [ ] **W4.13 — Grain Filtering** (None/Classic/Improved) `[M✅][E?][U❌]`.

**Verificação W4:** parity Texturized (idempotente sob sobreposição) vs Moving (acumula); manual: textura
visível, Moving "rola" com o traço.

---

## Wave 5 — Dynamics + Stabilization

- [ ] **W5.1 — Speed → Size** `[M✅][E?][U existe]` — slider ±100%.
- [ ] **W5.2 — Speed → Opacity** `[M✅][E?][U existe]`.
- [ ] **W5.3 — Speed → Spacing** `[M✅][E?][U?]`.
- [ ] **W5.4 — Jitter → Size** `[M✅][E?][U existe]`.
- [ ] **W5.5 — Jitter → Opacity** `[M✅][E?][U existe]`.
- [ ] **W5.6 — StreamLine Amount** `[M✅][E?][U❌]` — suavização do path (inking). Aplica no scheduler (W0.2).
- [ ] **W5.7 — StreamLine Pressure** `[M✅][E?][U❌]`.
- [ ] **W5.8 — Stabilization Amount** `[M✅][E?][U❌]` — média móvel.
- [ ] **W5.9 — Motion Filtering Amount** `[M✅][E?][U❌]` — FFT.
- [ ] **W5.10 — Motion Filtering Expression** `[M✅][E?][U❌]`.

**Verificação W5:** parity de velocidade (size/opacity vs ‖v‖); StreamLine suaviza um traço trêmulo de forma determinística.

---

## Wave 6 — Taper *(taper de início/fim; gaps de modelo)*

- [ ] **W6.1 — Pressure Taper: comprimento início/fim** `[M✅][E?][U❌]`.
- [ ] **W6.2 — Size / Opacity (taper)** `[M✅][E?][U❌]`.
- [ ] **W6.3 — Pressure (taper)** `[M⚠️][E?][U❌]` — **diverge:** `bool`→slider (`taperPressure`).
- [ ] **W6.4 — Link Tip Sizes** `[M✅][E?][U❌]`.
- [ ] **W6.5 — Tip / Shape** `[M❌][E❌][U❌]` — **gap:** `taper.tip` (`pencilTaperShape`). ⚠️ toca `Brush`.
- [ ] **W6.6 — Touch Taper (split pencil↔touch)** `[M❌][E❌][U❌]` — **gap:** conjunto separado p/ dedo (sem
  Pressure/Tip Animation). Decisão de design: duplicar campos vs flag. ⚠️ toca `Brush`.
- [ ] **W6.7 — Classic** `[M❌][E❌][U❌]` — **gap:** `taper.classic` (`taperVersion`). ⚠️ toca `Brush`.
- [ ] **W6.8 — Tip Animation** `[M❌][U❌]` — preview-only (baixa prioridade).

**Verificação W6:** parity do perfil de taper (size/opacity vs posição-na-pincelada); manual: ponta afina no fim.

---

## Wave 7 — Wet Mix *(mixer-brush — o coração da cor; o maior trabalho)*

> Modelo reservatório pickup-and-deposit (IMPaSTo/DAB, §2 da teoria), com a cor passando por Mixbox.
> Depende de W0.3 (estágio de cor) e dos modos Blending (W3.1).
>
> **✅ NÚCLEO + GATING + UI IMPLEMENTADOS (2026-06-16)** — design em [`04_design_W7_wet_mix.md`](04_design_W7_wet_mix.md).
> `WetState`/`WetMixConfig` em `cpu_render`; reservatório threaded no `apply_stamps_wash`; wiring no tool
> (`begin_stroke` semeia, `queue_pointer` passa, `end_stroke` dropa). **Gating:** `RenderingMode::is_blending()`
> ativa o Wet Mix nos 2 modos Blending (ou toggle explícito) — paridade Procreate. **UI:** seção "Wet Mix" no
> Brush Studio com 5 sliders (Dilution/Charge/Attack/Pull/Wetness Jit) + reset, wirada ponta-a-ponta
> (ids → BrushParam → set_param → snapshot). 4 testes de paridade + suítes brush/tool/panel/contracts verdes +
> shell compila.
>
> **✅ FASE 2 COMPLETA (2026-06-16)** — Grade (contraste de textura, pivota em 1.0), Blur (composição sobre
> backdrop box-blurred, raio ≤3px) e Blur Jitter (raio randomizado por-dab) implementados no engine + 3 sliders
> na UI (ponta-a-ponta) + 3 testes (helpers + wiring). **W7 INTEIRO FECHADO.** Pendente só o teste visual manual.

- [x] **W7.0 — Reservatório por-pincelada.** Estado do brush: carga (`load`) depositada no início, esgota ao
  arrastar, recarrega ao levantar/retocar. Pickup (`r_pickup ∝ canvas`) + deposit (`r_deposit ∝ reservoir`)
  por dab; cor mistura via Mixbox (`C_new` ponderado).
- [x] **W7.1 — Dilution** `[M✅][E?][U❌]` — água/transparência.
- [x] **W7.2 — Charge** `[M✅][E?][U❌]` — carga inicial / esgotamento.
- [x] **W7.3 — Attack** `[M✅][E?][U❌]` — taxa de depósito.
- [x] **W7.4 — Pull** `[M✅][E?][U❌]` — pickup/esfregaço do canvas.
- [x] **W7.5 — Grade** `[M✅][E✅][U✅]` — contraste/chunkiness da textura (pivota em 1.0; 0.5=neutro).
- [x] **W7.6 — Blur** `[M✅][E✅][U✅]` — espalhamento: compõe sobre backdrop box-blurred (raio ≤3px).
- [x] **W7.7 — Blur Jitter** `[M✅][E✅][U✅]` — randomiza o raio do blur por-dab.
- [x] **W7.8 — Wetness Jitter** `[M✅][E?][U❌]`.

**Verificação W7:** parity headless do reservatório (esgotamento determinístico); manual: amarelo sobre azul →
**verde** (Mixbox), não cinza; o traço afina conforme a carga esgota; Pull esfrega cor existente.

---

## Wave 8 — Color Dynamics

- [ ] **W8.1 — Stamp Color Jitter: Hue/Saturation/Lightness/Darkness/Secondary** `[M✅][E?][U existe]` (intra-pincelada).
- [ ] **W8.2 — Stroke Color Jitter: Hue/Saturation/Lightness/Darkness/Secondary** `[M✅][E?][U existe]` (inter-pincelada).
- [ ] **W8.3 — Color Pressure: Hue/Saturation/Brightness/Secondary** `[M✅][E?][U?]`.
- [ ] **W8.4 — Color Tilt: Hue/Saturation/Brightness/Secondary** `[M✅][E?][U?]`.
- [ ] **W8.5 — Color Barrel Roll (Pencil Pro): Hue/Saturation/Brightness/Secondary** `[M✅][E?][U?]`.

**Verificação W8:** parity do jitter por seed (stamp varia dab-a-dab; stroke varia entre pinceladas).

---

## Wave 9 — Apple Pencil (tilt/barrel) + Properties + Preview

- [ ] **W9.1 — Tilt: curva + Angle (threshold)** `[M⚠️][E?][U❌]` — **gap:** `pencil.tilt_angle` (`dynamicsTiltAngle`). ⚠️ toca `Brush`.
- [ ] **W9.2 — Tilt → Opacity / Gradation / Bleed / Size** `[M⚠️][E?][U❌]` — amounts per-target (decisão de encoding, W1.7).
- [ ] **W9.3 — Tilt: Size Compression** `[M❌][E❌][U❌]` — **gap:** `pencil.tilt_size_compression`. ⚠️ toca `Brush`.
- [ ] **W9.4 — Barrel Roll (Pencil Pro) → Size / Opacity / Bleed** `[M⚠️][E?][U❌]`.
- [ ] **W9.5 — Hover (Outline / Estimated Pressure / Fill)** `[M✅][E?][U❌]`.
- [ ] **W9.6 — Properties: Orient to Screen** `[M✅][E?][U❌]`.
- [ ] **W9.7 — Properties: Smudge Pull** `[M✅][E?][U❌]`.
- [ ] **W9.8 — Properties: Max/Min Size, Max/Min Opacity** `[M✅][E?][U❌]` — clamps da sidebar.
- [ ] **W9.9 — Preview (categoria 13)** `[M❌][U❌]` — nova `PreviewParams` (ou dobrar em `properties`):
  use stamp preview, size, pressure min/scale, wet mix, tilt angle. ⚠️ se struct nova, usa o slot de headroom.

**Verificação W9:** manual com Apple Pencil real (único que exige o Enio — tilt/barrel/hover); clamps respeitados.

---

## Wave 10 — UI completa do Brush Studio *(track paralelo às W1-W9)*

> Cada seção da UI cresce junto com a onda do seu parâmetro. Esta wave fecha o que sobrar + o chrome.

- [ ] **W10.1 — Seções faltantes:** stabilization, taper, grain, wet_mix, pencil, properties, preview, about.
- [ ] **W10.2 — Widgets canônicos:** sliders bidirecionais (±100%), curve editor 2D (pressão/tilt — ver
  [reference_panel_2d_drag](../../docs)), graph de Roundness, dropdowns de enum, image-source pickers. Espelhar
  gallery/inspector; tokens + i18n (HR-15); **labels em inglês**.
- [ ] **W10.3 — Live preview:** o stroke/stamp preview no topo do Brush Studio reflete os params em tempo real.
- [ ] **W10.4 — Brush Library:** thumbnails, grupos, default brushes que exercitam cada feature.

**Verificação W10:** smoke da UI; cada controle altera o `Brush` e o preview; zero hex/f32-literal/string hardcoded.

---

## Wave 11 — Persistência + import de `.brush`

- [ ] **W11.1 — Round-trip do `Brush`** no `.ph2d-painter` (já serializa; validar com o set completo de params).
- [ ] **W11.2 — Reset Points** (`about.reset_points`) — checkpoint/restore.
- [ ] **W11.3 — (Opcional, diferencial) Import de `.brush`/`.brushset` do Procreate** — parsear o
  `Brush.archive` (NSKeyedArchiver plist) + Shape.png/Grain.png, mapear as chaves
  ([referência](02_referencia_parametros_procreate.md)) → nosso `Brush`. Destrava milhares de brushes existentes.

**Verificação W11:** round-trip byte-idêntico (gate `painter_persistence_roundtrip`); import de um `.brush` real renderiza igual.

---

## Fora de escopo (deferred)

- **Materials (categoria 12)** — metallic/roughness/height são p/ pintura de modelo **3D**. Fora do Painter 2D.
- **Dual Brush** — segundo brush aninhado (`Sub01`). Avaliar depois da paridade single-brush (é commodity, mapeia a "masked brush").

## O diferencial (depois da paridade — não antes)

Só após ter **tudo o que o Procreate tem**, atacar os melhoramentos, ancorados no que já é nosso forte:
mistura de cor **Mixbox/K-M espectral** de qualidade (ADR-0080/0091) acima do baseline; brushes grandes /
4K real-time via GPU-resident (o substrato de [`plans/wise-seeking-gem`](../../) existe mas **não** reintroduzir
fluido); e qualquer física de deposição realista **como melhoria do mark estático**, não como simulação.

---

## Resumo das etapas que tocam contrato congelado (Coord-only + ADR)

Estas exigem ADR-amendment + gate `architecture_painter_contract_surface` (há 1 slot de headroom no `Brush`):
W1.4 (jitter_linear), W1.7/W9.2 (encoding de pencil targets), W2.3 (rotation slider), W2.10 (shape angle),
W3.3/W3.4 (wet/burnt edges slider), W6.3/W6.5/W6.6/W6.7 (taper), W9.1/W9.3 (tilt angle/compression), W9.9 (PreviewParams).
**Agrupar num único ADR-amendment** ("Brush param surface para paridade Procreate completa") em vez de N amendments.

## Ordem recomendada de execução

**MVP do mark** (W0 → W1 → W3 → W2) — carimba com pressão + os 6 rendering modes + shape. Depois **refino do
mark** (W4 grain, W5 dynamics/streamline, W6 taper). Depois **cor avançada** (W0.3+W7 wet mix, W8 color
dynamics). Depois **input fino + chrome** (W9), **UI** (W10, em paralelo desde W1), **persistência** (W11).
A matriz de W0.1 pode reordenar conforme o que já estiver vivo no engine.

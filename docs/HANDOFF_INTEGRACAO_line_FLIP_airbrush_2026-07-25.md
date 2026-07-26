# HANDOFF DE INTEGRAÇÃO — `line/FLIP` · AIRBRUSH + REAMOSTRAGEM (2026-07-25)

> **Para o AGENTE INTEGRADOR.** Este é o handoff do **tip atual** da `line/FLIP`. A linha traz
> agora **10 commits** sobre o `main` de hoje (ff-only): o multiplano 2.5D + o polish dos sliders +
> o Self Overlap (detalhe em [`HANDOFF_INTEGRACAO_line_FLIP_self_overlap_2026-07-25.md`](HANDOFF_INTEGRACAO_line_FLIP_self_overlap_2026-07-25.md))
> + o **Airbrush** (`0443f98c2`) + a **reamostragem suave do traço** (`a14335e5f`, §7 abaixo).
> **Não integrar nem pushar sem ordem EXPLÍCITA do Enio.**
>
> + a **dinâmica de pressão** (`c42edabf7`, §8 abaixo).
>
> ⚠️ **Contagem/schema:** ~12 commits; confira `git rev-list --count main..line/FLIP`. Schema
> INTOCADO pela reamostragem E pela pressão (a primeira só produz mais pontos no MESMO `FlipStroke`;
> a segunda assa a largura por-ponto e é estado de FERRAMENTA, não dado) ⇒ segue `FLIP_SCHEMA` **12**,
> `PROJECT_SCHEMA` **34**, tripla `(34, 12, 13)`.

## 0. GO — o essencial em 6 linhas

- **8 commits, ff-only** (`git rev-list --count main..line/FLIP` = 8). O tip é `0443f98c2`
  (Airbrush); o commit anterior `f6d6b78d4` é o **registro do smoke do Self Overlap** (Enio aceitou
  a estrela do pincel macio como o `over` correto — nenhuma mudança de código lá).
- **Bump de schema (acumulado nesta linha):** `FLIP_SCHEMA_VERSION` **9→12**, `PROJECT_SCHEMA`
  **31→34**, tripla do pin **`(34, 12, 13)`**. Multiplano fez 10/32; Self Overlap 11/33; Airbrush
  **12/34**. ⚠️ **O 34 se CONTA, não se escolhe** — se outra linha bumpar o `PROJECT_SCHEMA` na
  mesma janela, **renumere** (a soma) e atualize a tripla em `project_tests.rs`
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **Contratos congelados (§6): INTOCADOS.** Flip não tem contrato congelado (freeze é item futuro
  do §8). Nenhum ADR novo (Airbrush é o item §8 do `03_traco_rasterizacao.md`). `GpuStroke` **não**
  mudou de tamanho (o bit entrou no `flags` existente).
- **Smoke:** `env PH2D_FLIP_AIRBRUSH_SMOKE=1 cargo run -p ph2d-host-desktop --release`.
- **Gotchas do `ship.sh` (herdados, não novos):** os gates GPU do Flip são `#[ignore]` e precisam
  de adapter (rodei os 22 na RTX: **22/22**; sem adapter fazem skip gracioso, que não é verde); o
  `ph2d-flip-colorize` **panica em build DEBUG** e passa em `--release`/`ci-test` — rode as duas.
- **Sem MERGE textual esperado** (a linha só toca arquivos do Flip + o wiring de schema do shell;
  conflito em `project.rs`/`project_tests.rs` por outra linha ter bumpado ⇒ resolva pela **SOMA**).

## 1. O que a wave entrega (Airbrush)

Um toggle **Airbrush** por-pincel (seção Brush, modo Draw, abaixo do Self Overlap): o falloff da
borda do traço deixa de ser o `pow`+smoothstep e vira a **transmitância física** da tinta por um
dab esférico (Beer-Lambert): `A(dn) = 1 − exp(−k·√(1−dn²))`, `k = mix(1, 8, hardness)`. É um **domo
largo** de núcleo chato e borda SEMPRE macia — o oposto do pico estreito do `pow`. O slider Hardness
vira a **densidade** da névoa. Casa com o Self Overlap (a acumulação `over` de airbrush é a
multiplicação de transmitâncias, o build-up físico).

**Medido** (a sonda headless = o gate GPU, banda raio 10, hardness 0.5): eixo **222 padrão / 252
airbrush** (o domo segura onde o pico já caiu, 1 px do centro) · meio-raio (dn≈0.5) **0 padrão / 249
airbrush** (O DISCRIMINANTE) · borda (dn≈0.8) **0 padrão / 231 airbrush** (domo largo, rola suave a
zero). Render-and-look confirmou: o padrão é uma espinha fina, o airbrush um domo largo macio.

## 2. A espinha (para revisar com confiança)

- **Porta ÚNICA = o ramo `if (airbrush)` no `hardness_mask` do `flip.wgsl`.** OFF: byte-idêntico (o
  bit é 0, o ramo nem roda). ON: `1 − exp(−k·√(max(1−dn²,0)))`, `k = mix(1,8,hardness)`. O AA de
  borda (`edge`, fwidth) fica intocado — o airbrush troca só o *profile*.
- **`FlipStroke::airbrush` (por-curva, irmão de `self_overlap`)** → `pack.rs::stroke_flags` seta
  `FLAG_AIRBRUSH = 1<<4` no `flags` → shader.
- ⚠️ **A flag chega ao FRAGMENT por um varying flat `flags` novo (loc 14).** Antes o fragment não
  lia `flags` (o Self Overlap é vertex-only, via depth); o airbrush é a 1ª decisão de máscara por
  flag no fragment. É reusável — a próxima flag de máscara testa o mesmo varying.
- ⚠️ **`exp`/`sqrt` no shader = display path, não determinismo** (o `pow` já estava lá; o raster do
  Flip é re-rasterizado por frame, não replayado bit-exato).
- ⚠️ **`k∈[1,8]` é ESTÉTICO** (a densidade da névoa mais forte), não limite de recurso — a borda do
  airbrush é sempre macia, mesmo em `K_MAX = 8`. Ajustável sem medo.
- **Wiring:** `FlipTool::airbrush` + `FlipStyleSnapshot::airbrush` + `flip_draw.rs` (o traço herda)
  + toggle-chip no painel (`paint_tip.rs`, `segmented` de 1 opção, Draw-only) + `event.rs` roteia o
  Click (`is_style_forward_click`) + `populate.rs` registra + id `FLIP_AIRBRUSH`
  (`ids/chrome/flip.rs`).

## 3. Gates (red-first, mutação-provados nesta sessão)

- `ph2d-flip-render/tests/gpu_render.rs::an_airbrush_has_a_flatter_core_than_the_standard_brush`
  (RTX, `#[ignore]`): meio-raio ~249 airbrush vs ~0 padrão; borda macia; o domo segura perto do
  eixo (252 vs 222). **RED provado**: mutando `if (airbrush)` → `if (false)` (a flag ignorada, cai
  no `pow`), o meio-raio do airbrush desaba para o pico → o gate falha; restaurei o shader por
  backup + `touch`.
- `ph2d-flip-render/src/pack.rs::airbrush_sets_the_flag_bit_and_nothing_else` — o bit é o ÚNICO
  delta de `flags` entre OFF e ON (OFF byte-idêntico).
- `ph2d-tool-flip/src/tool_tests.rs::the_airbrush_toggle_reaches_the_tool_and_toggles`.
- `ph2d-panel-flip/tests/seam.rs::the_airbrush_toggle_is_draw_only_and_forwards_to_the_tool` —
  pintado+clicável no Draw · MODAL (some fora do Draw) · Click FORWARDS pela costura real.
- Pin: `project_tests.rs::a_schema_bump_anywhere_must_bump_the_project_schema` → `(34, 12, 13)`.

## 4. LOC / higiene

- Nenhum split necessário nesta wave (`stroke.rs` 510, `tool.rs` 496, `event.rs`/`populate.rs`
  abaixo do teto de 200, `flip_airbrush_smoke.rs` 125). O `gpu_render.rs` (tests/) cresceu ~60
  linhas — a suíte de tests não bate no `workspace_file_loc_cap` (gate verde).
- Uma sonda `probe_airbrush_png` (throwaway) foi usada para o render-and-look e **removida** (não
  commitada); os PNGs `air_std`/`air_dome` ficaram no scratchpad da sessão.

## 5. Passos de integração (quando o Enio mandar)

1. `cd` na worktree do integrador, `git fetch`, confira que `line/FLIP` está 8 à frente do `main`
   e é **ff-only** (`git merge-base --is-ancestor main line/FLIP`).
2. Se `PROJECT_SCHEMA` colidir com outra linha da jornada: **renumere pela SOMA** e atualize a
   tripla `(N, 12, 13)` em `project_tests.rs`.
3. `git merge --ff-only line/FLIP` (ou `foundational-integrate.sh` se houver outras linhas).
4. `./scripts/ship.sh` — corrija todo `✗`. ⚠️ perfil `ci-test`/`--release`; os gates GPU
   `#[ignore]` do Flip precisam de adapter na RTX (`cargo test -p ph2d-flip-render --release
   --test gpu_render -- --ignored` = 22 tests).
5. Smoke do Enio: `env PH2D_FLIP_AIRBRUSH_SMOKE=1 cargo run -p ph2d-host-desktop --release` — a
   cena imprime `[airbrush-smoke] cena montada: 2 tracos grossos ...`. **Se essa linha não
   aparecer, PARE** (árvore/env errada). Depois: o traço da DIREITA (airbrush) é um domo largo
   macio; o da ESQUERDA (padrão) uma espinha fina; no painel Draw, abaixo do Self Overlap, o
   toggle **Airbrush** (com ele ligado, o slider Hardness vira a densidade).

## 6. Aberto (não-bloqueante, decisões de produto)

- **Corner types / joins & caps (SVG parity)** — o próximo item grande do §8 (miter/bevel/round
  joins + butt/round/square caps). Foi avaliado nesta sessão e **deixado para wave dedicada**: mexe
  na cobertura-união (a joia da coroa que custou uma semana de bugs), merece tratamento próprio.
- **Congelar o contrato do `ph2d-flip`** — com multiplano/tween/colorize/edit/tira/self-overlap/
  airbrush assentados, o modelo estabilizou; o gate de superfície (padrão Nodes/Tools/Vector) é
  candidato natural.
- O **eraser/reshape/tween** não leem `airbrush` (é atributo de DEPÓSITO); um traço com o flag
  sobrevive a eles pelo `clone_attrs`, que o carrega.

## 7. Reamostragem suave do traço (`a14335e5f`, T2.8) — o report do "tracejado"

Enio (com screenshot): *"o traço de qualquer modo tem baixo número de vértices e assim fica
tracejado e não arredondado nas curvas. dê mais resolução ao traço."* O RDP + o render ligavam os
pontos por RETAS ⇒ poucos pontos = curvas facetadas; o pipeline só **removia** pontos, nunca
adicionava resolução suave.

- **Fix:** `flip_smooth::resample_smooth(pts, prs, step)` — uma **Catmull-Rom** (Hermite uniforme)
  entre o RDP e o `build_stroke`: interpola pelos pontos (o traço passa EXATO por eles) e densifica
  a cada `step = 0.4 × size_to_world(width)` de arco. Pontos evenly-spaced (nem facetas, nem
  redundantes). **Quinas preservadas** (giro > 60° ⇒ vinco C0, tangente unilateral); pressão
  interpolada linear; cap por-span (`MAX_SUB_PER_SPAN=24`). É a MESMA porta do preview e do bake.
- **Sem schema/contrato** — o `FlipStroke` não muda de forma, só ganha mais pontos.
- **Gates:** 5 unit em `flip_smooth` (densifica+alisa · reta fica reta · quina sobrevive · pressão
  interpola · <3 pontos identidade) + end-to-end `flip_draw_tests::a_sparse_stroke_becomes_a_smooth_curve`
  (C esparso pela porta REAL: giro 45°→<15°; **RED provado** bypassando o `resample_smooth`) + o
  `the_resampled_stroke_tracks_the_drawing_without_redundant_points` (reescrito do
  `the_stroke_drops_redundant_vertices`: acompanha o desenho, sem pontos grudados). Render-and-look
  (PNG): C 7 pts facetado × 49 pts liso.
- **Smoke:** `PH2D_FLIP_RESAMPLE_SMOKE=1` — C esparso (6 pts, facetado) × reamostrado (62 pts, liso)
  lado a lado; e desenhar à mão sai arredondado. ⚠️ **A DENSIDADE (`RESAMPLE_STEP_FRACTION=0.4`) é o
  smoke que decide** (como a tolerância do RDP) — mais/menos arredondado é o olho do Enio.
- **Aberto:** custo por-frame no preview (a reamostragem roda a cada frame do drag, como o RDP; o
  item "Pack INCREMENTAL do traço em curso" do §8 cobre o caso patológico de 4000 pontos).

## 8. Dinâmica de pressão → largura (`c42edabf7`, T2.6)

A pressão da caneta virava largura LINEAR (`pr.clamp(0.05, 1.0)`, "1º corte"), sem controle. Toda
caneta de tablet tem uma dinâmica editável.

- **Fix:** dois controles do artista, pela porta única `ph2d_tool_flip::pressure_width_factor` (pura,
  testada), aplicada no `build_stroke`: `factor = min + (1−min)·pr^γ`, `γ = 2^((response−0.5)·4)`.
  **Min Width** (a largura em pressão zero, o piso; `1.0` ignora a pressão) + **Response** (a curva
  macia⇔dura: `0.5` linear, `<0.5` ease-in, `>0.5` ease-out). No mouse `pr=1` ⇒ largura cheia.
- **Sem schema/contrato** — a largura é assada por-ponto no traço em tempo de desenho; os params são
  estado vivo da ferramenta (como Size/Hardness), NÃO dado guardado. `powf` roda 1×/ponto no build
  (CPU, não é caminho de determinismo).
- **UI:** 2 sliders **Min Width** (%) + **Response** na seção Brush(Draw), o `slider_row` canônico
  (abaixo de Smoothing). Ids `FLIP_PRESSURE_MIN`/`_RESPONSE`(+`_NUM`), com populate + event.rs
  (ValueChanged→SetValue + swallow do `_NUM`) + os setters do tool.
- **Gates:** 3 puros em `params::pressure_tests` (piso/cheio · macia>linear>dura · min=1 ignora ·
  clamps) + tool (`the_pressure_sliders_reach_the_tool`) + seam (`pressure_sliders_drag_reaches_tool`)
  + end-to-end `flip_draw_tests::pressure_tapers_the_stroke_width` (rampa 0→1: ponta=piso, fim=cheio,
  5×; **RED provado** com largura constante). Render-and-look (PNG): 3 tapers (default · piso alto ·
  response dura) confirmam a cunha e os controles.
- **Smoke:** `PH2D_FLIP_PRESSURE_SMOKE=1` — 3 traços com rampa de pressão 0→1 (default · Min 60% ·
  Response dura). ⚠️ **A FAIXA/CURVA (`K=4`, defaults 0.05/0.5) é decisão de smoke** — mais/menos
  taper é o gosto do Enio. Uma CURVA editável cheia (widget) seria o superset se ele quiser controle
  fino (o `ph2d-curve` + `ParamWidget::Curve` já existem no repo).

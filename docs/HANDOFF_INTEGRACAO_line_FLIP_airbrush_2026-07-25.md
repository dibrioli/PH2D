# HANDOFF DE INTEGRAÇÃO — `line/FLIP` · AIRBRUSH (2026-07-25)

> **Para o AGENTE INTEGRADOR.** Este é o handoff do **tip atual** da `line/FLIP`. A linha traz
> agora **8 commits** sobre o `main` de hoje (ff-only): o multiplano 2.5D + o polish dos sliders +
> o Self Overlap (detalhe em [`HANDOFF_INTEGRACAO_line_FLIP_self_overlap_2026-07-25.md`](HANDOFF_INTEGRACAO_line_FLIP_self_overlap_2026-07-25.md))
> e agora o **Airbrush** (`0443f98c2`). **Não integrar nem pushar sem ordem EXPLÍCITA do Enio.**

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

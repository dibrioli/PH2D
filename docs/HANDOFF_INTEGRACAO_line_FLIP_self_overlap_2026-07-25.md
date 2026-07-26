# HANDOFF DE INTEGRAÇÃO — `line/FLIP` · SELF OVERLAP (2026-07-25)

> **Para o AGENTE INTEGRADOR.** Este é o handoff do **tip atual** da `line/FLIP`. A linha traz
> agora **5 commits** sobre o `main` de hoje (ff-only): o multiplano 2.5D + o polish canônico dos
> sliders (os quatro primeiros — detalhe em
> [`HANDOFF_INTEGRACAO_line_FLIP_multiplane_2026-07-25.md`](HANDOFF_INTEGRACAO_line_FLIP_multiplane_2026-07-25.md))
> e agora o **Self Overlap** (`ef391420d`). **Não integrar nem pushar sem ordem EXPLÍCITA do Enio.**

## 0. GO — o essencial em 6 linhas

- **5 commits, ff-only** (`git rev-list --count main..line/FLIP` = 5): `9ac0f2fe7` (multiplano) ·
  `5a63bc08a` (handoff) · `aebe3ae52` (sliders canônicos) · `f97be4689` (handoff) · **`ef391420d`
  (Self Overlap — este handoff)**.
- **Bump de schema (acumulado nesta linha):** `FLIP_SCHEMA_VERSION` **9→11**, `PROJECT_SCHEMA`
  **31→33**, tripla do pin **`(33, 11, 13)`**. O multiplano fez 10/32; o Self Overlap fez **11/33**.
  ⚠️ **O 33 se CONTA, não se escolhe** — se outra linha bumpar o `PROJECT_SCHEMA` na mesma janela,
  **renumere** (a soma, não o que a linha escreveu) e atualize a tripla em `project_tests.rs`
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **Contratos congelados (§6): INTOCADOS.** Flip não tem contrato congelado (ADR-0114 §Contrato).
  Nenhum ADR novo (Self Overlap era o item §8 do `03_traco_rasterizacao.md`). `GpuStroke` **não**
  mudou de tamanho (o bit entrou no `flags` existente).
- **Smoke:** `env PH2D_FLIP_SELF_OVERLAP_SMOKE=1 cargo run -p ph2d-host-desktop --release`.
- **Gotchas do `ship.sh` (herdados, não novos):** os gates GPU do Flip são `#[ignore]` e precisam
  de adapter (rodei os 21 na RTX: **21/21**; sem adapter fazem skip gracioso, que não é verde);
  o `ph2d-flip-colorize` **panica em build DEBUG** e passa em `--release`/`ci-test` — rode as duas.
- **Sem MERGE textual esperado** (a linha só toca arquivos do Flip + o wiring de schema do shell;
  se houver conflito no `project.rs`/`project_tests.rs` por outra linha ter bumpado, resolva pela
  **SOMA** dos schemas, não por lado).

## 1. O que a wave entrega (Self Overlap)

Um toggle **Self Overlap** por-pincel (seção Brush, modo Draw): quando ligado, um traço que
**cruza a si mesmo** (laço, X, rabisco que volta) fica **MAIS ESCURO no cruzamento** — as passagens
compõem (`over`) em vez de ficarem a união chapada. É o `GP_STROKE_OVERLAP` do Grease Pencil
(marcador/nanquim expressivo). Só visível com **opacidade < 1** (a tinta opaca já satura).

**Medido** (sonda headless = o gate GPU, cruzamento em X a opacity 0.5): braço (1 passagem)
**~128 de 255** · cruzamento **OFF ~128** (== braço) · cruzamento **ON ~191** (`0.75` = a
composição de 2 camadas `0.5 + 0.5·(1−0.5)`). A passagem mais NOVA fica por cima no ON.

## 2. A espinha (para revisar com confiança)

- **A nota §8 do `03_traco_rasterizacao.md` dizia "1 bit + 1 linha: depth por-PONTO"** — herdada do
  GP, que usa cobertura own-segment. **Nós temos a UNIÃO global** (o fix da mordida). Conferi no
  shader: o acúmulo vem do **DOUBLE-BLEND**, e a união muda o *mask*, não o *número de fragmentos*
  — o alpha é `opacity·mask`, e com `opacity < 1` dois fragmentos de 0.5 compõem 0.75 **mesmo com
  mask=1**. Logo a nota está **certa para nós**: mantém-se a união, flipa-se só a depth.
- **Porta ÚNICA = `out.clip.z` no `flip.wgsl`.** OFF: `f32(2u*sid+2u)*2e-7` (byte-idêntico — o bit
  é 0, o ramo `if` nem roda). ON: `+ f32(li)/f32(count)*1.9e-7` → depth **por-SEGMENTO**, monótona
  no índice do ponto, **dentro do slot do traço** `[(2sid+2), (2sid+3))` (acima do próprio fill em
  `2sid+1`, abaixo do próximo fill em `2sid+3`). `Depth32Float` resolve o degrau (~5e-11 a 4000
  pontos ≫ ULP ~2,4e-14). As faces sobrepostas passam o GREATER estrito e **BLENDam**.
- **A UNIÃO fica nos dois modos** (mata a mordida e mantém o acúmulo limpo: composição de 2 camadas,
  nunca a mordida).
- **Consequência NOMEADA (o que o smoke decide):** a quina **afiada** (miter_break) também acumula
  — as fitas estendidas se sobrepõem, bleed de marcador. Curva **suave** NÃO (as fitas mitram/
  abutam). Cruzamento **verdadeiro** SIM. Decisão: **acumular em tudo** (semântica GP, opt-in). Se
  o Enio quiser quina limpa, é follow-up (mecanismo diferente — nomeado no gate, não construído).
- **Wiring:** `FlipStroke::self_overlap` (por-curva, irmão de `tip`) → `pack.rs::stroke_flags`
  seta `FLAG_SELF_OVERLAP = 1<<3` no `flags` → shader. Brush: `FlipTool::self_overlap` +
  `FlipStyleSnapshot::self_overlap` + `flip_draw.rs` (o traço herda) + toggle-chip no painel
  (`paint_tip.rs`, um `segmented` de 1 opção sem caption, Draw-only) + `event.rs` roteia o Click
  (toggle) + `populate.rs` registra + id `FLIP_SELF_OVERLAP` (`ids/chrome/flip.rs`).

## 3. Gates (red-first, mutação-provados nesta sessão)

- `ph2d-flip-render/tests/gpu_render.rs::a_stroke_with_self_overlap_accumulates_at_the_crossing`
  (RTX, `#[ignore]`): cross ~191 vs braço ~128; cross > arm; a passagem nova (azul) por cima; a
  quina também acumula. **RED provado**: revertendo a depth para por-traço (`z = z`), `cross=127
  == arm=127` → o gate falha; o gate OFF (`..._without_accumulation`) permanece verde. Restaurei
  o shader por backup + `touch`.
- `ph2d-flip-render/src/pack.rs::self_overlap_sets_the_flag_bit_and_nothing_else` — o bit é o
  ÚNICO delta de `flags` entre OFF e ON (OFF byte-idêntico).
- `ph2d-tool-flip/src/tool_tests.rs::the_self_overlap_toggle_reaches_the_tool_and_toggles` — o
  Click alterna o flag; nasce OFF; snapshot concorda.
- `ph2d-panel-flip/tests/seam.rs::the_self_overlap_toggle_is_draw_only_and_forwards_to_the_tool` —
  pintado+clicável no Draw · MODAL (some fora do Draw) · Click FORWARDS pela costura real.
- Pin: `project_tests.rs::a_schema_bump_anywhere_must_bump_the_project_schema` → `(33, 11, 13)`.

## 4. LOC (refatores de carona, não hacks)

As duas fns do painel estavam **no teto de 200** e as +2 linhas do feature estouraram; `stroke.rs`
/`tool.rs` estavam **no teto de 700**. Por responsabilidade:
- `event.rs`: o predicado do guard gigante virou `fn is_style_forward_click(id)` (pura, testável);
  `apply_event` 202→194.
- `populate.rs`: os botões de estilo (Shape·Tip·Self Overlap) viraram `fn draw_style_buttons`;
  `populate` 202→195.
- `stroke.rs` 707→497 (tests → `stroke_tests.rs`) · `tool.rs` 727→478 (tests → `tool_tests.rs`),
  o padrão `#[path]` que `object.rs`/`object_tests.rs` já usam.
- **Fix de carona (a wave anterior):** `paint_layers.rs` — os `100.0` do `%` do slider canônico
  (multiplano/Phase B) tinham o `// LITERAL-PX-OK` só em 1 dos 3 sítios; os outros 2 (a mesma
  conversão fração→percent, math constant) ganharam a anotação. O `no_magic_numeric` estava
  vermelho-latente por isso e agora é verde.

## 5. Passos de integração (quando o Enio mandar)

1. `cd` na worktree do integrador, `git fetch`, confira que `line/FLIP` está 5 à frente do `main`
   e é **ff-only** (`git merge-base --is-ancestor main line/FLIP`).
2. Se `PROJECT_SCHEMA` colidir com outra linha da jornada: **renumere pela SOMA** e atualize a
   tripla `(N, 11, 13)` em `project_tests.rs`.
3. `git merge --ff-only line/FLIP` (ou `foundational-integrate.sh` se a jornada tiver outras
   linhas na árvore combinada).
4. `./scripts/ship.sh` — corrija todo `✗`. ⚠️ perfil `ci-test`/`--release`; os gates GPU
   `#[ignore]` do Flip precisam de adapter na RTX (rode `cargo test -p ph2d-flip-render
   --release --test gpu_render -- --ignored` = 21 tests).
5. Smoke do Enio: `env PH2D_FLIP_SELF_OVERLAP_SMOKE=1 cargo run -p ph2d-host-desktop --release` —
   a cena imprime `[self-overlap-smoke] cena montada: 2 lacos ...`. **Se essa linha não aparecer,
   PARE** (árvore/env errada). Depois: o laço da DIREITA (ON) tem o nó do cruzamento mais escuro
   que o da ESQUERDA (OFF); no painel Draw, abaixo de Tip, há o toggle **Self Overlap**.

## 6. Aberto (não-bloqueante, decisões de produto)

- **SMOKE APROVADO — Enio "aceitar" (2026-07-25).** O smoke expôs um **brilho de 4 pontas** (star)
  no cruzamento com pincel MACIO; render-and-look (3 casos lado a lado) provou que **não é bug**:
  é a acumulação `over` de duas passagens de falloff **em pico** (o macio) concentrando no ponto
  central — fisicamente correta para tinta translúcida (o que um aerógrafo sobreposto faz). Com
  pincel DURO o mesmo `over` dá um **quadrado mais claro limpo** (o marcador). Ou seja: o Self
  Overlap está correto para o caso de uso dele (marcador/nanquim, borda dura); a estrela é o macio.
  Enio **aceitou como está** (opção 1: modelo correto, cruzamento limpo se faz com pincel duro).
  **Nenhuma mudança de código.** Evidência renderizada: `so_A_x` (duro, quadrado limpo) ×
  `so_D_x_soft`/`so_F_x_soft_off` (macio ON=estrela, OFF=liso) — sonda throwaway, não commitada.
- **Quina afiada limpa** — hoje a quina acumula junto (bleed de marcador, por design, gateado). Se
  o Enio quiser separar "cruzamento verdadeiro" de "quina", é outro mecanismo (a depth por-segmento
  faz as duas blendarem por construção). Nomeado, não construído.
- O **eraser/reshape/tween** não leem `self_overlap` (é atributo de DEPÓSITO); um traço com o flag
  sobrevive a eles pelo `clone_attrs`, que o carrega.

# HANDOFF de integração — `line/motion-value`, CICLO 20 até a W3b (2026-10-11)

> Ciclo 20 «O ACABAMENTO» ([doc 129](../129_ciclo_20_o_acabamento.md): desenho §3, medições §5,
> lições §6, abertos §7). INTEGRADO no `main` por ordem do Enio (11/10: «quando finalizar integre ao
> main») com o ciclo ainda ABERTO: W0–W3b fecharam; a W4 (as fotos de cada passo, o relógio a 4K, o
> tutorial 13) e a W5 (mutação + handoff do ciclo) seguem no
> [prompt da W4](HANDOFF_CONTINUACAO_line_motion_value_2026-10-10_CICLO_20_W4.md). A linha leva os
> ciclos 13–20; o ciclo 19 tem [handoff próprio](HANDOFF_INTEGRACAO_line_motion_value_2026-10-10_CICLO_19.md).

## §1 — O que o ciclo entrega (até a W3b)

- **`fx.lens`** (W0–W2): a lente da câmera — vinheta, aberração cromática, color grade com LUT
  `.cube`, grão — num passe, ordem fixa, uma lei na CPU (`ph2d-lens`) com paridade WGSL; o quadro
  VISÍVEL (não o renderizado). Smoke do dono APROVADO 10/10.
- **`fx.filter`** (W3): os 15 filtros raster do Vector (`FxOp::SPECS`) sobre a CAMADA do Motion,
  desenhada à parte e colada no mundo antes da lente; um nó por degrau, a pilha é a ordem do fio.
- **W3b** (o smoke do dono reprovou: a sombra interna não agia em cada forma; o bevel tremia):
  - `Apply To` = **Each Shape** · **Whole Layer** (decisão do dono): a pilha parte-se no primeiro
    `Whole Layer`; os `Each Shape` correm em CÉLULAS de um átlas (crate nova **`ph2d-fx-cells`**:
    cada instância sozinha, a pilha UMA vez, a volta na ordem de z); um `Each Shape` depois de um
    `Whole Layer` di-lo no cartão.
  - O bevel/campo raster: semente de **Gustavson & Strand 2011** (não treme: 6,8 níveis no pior
    contra 43); deslocamento e luz **sub-pixel**; o borrão em **pirâmide** acima de σ 32 px (honra
    qualquer σ: 1,2 níveis a σ 300); a cor do cartão chega sRGB; os defaults de cada tipo = os do
    Vector (o gesto `switch_kind`); o ruído preso ao mundo ou à forma; nomes únicos («Effect»,
    «Outer Glow», o nó das cópias de sombra passa a «Shadow Copies»).
  - A cena `=135`: dois `Filter` (Bevel em cada disco · Drop Shadow da galáxia inteira).

## §2 — Superfície de colisão (FOUNDATIONAL e partilhado)

- **`ph2d-render`** (o motor dos filtros, partilhado com o Vector):
  - `FxOpGpu.offset_px: [f32; 2]` (era `[i32; 2]`) — quem constrói um `FxOpGpu` à mão escreve floats;
  - `FxStackPass::run_premultiplied` SAIU → `run_layer(gpu, src, dst, w, h, ops, Origem)`;
    `run`/`run_from` (o Vector) iguais;
  - `Origem { Margem, Ponto, Mapa }`, `stack_read`/`op_read` (o que a pilha LÊ), `SIGMA_ALCANCAVEL`,
    `Plan::Blur { niveis }`; `Globals` 304 → **320 B** (`aux`, a pirâmide) — Rust e WGSL;
  - a semente do campo (`fx_stack_field.rs`) é outra: TODO raster-seeded (o Vector inclusive: a
    morfologia) mede a distância pela fronteira de G&S; os 95 gates de placa do `fx_stack` verdes;
  - **`fx_atlas` mudou de crate**: `ph2d-app-vec/src/fx_atlas.rs` → `ph2d-render/src/fx_atlas.rs`
    (o Vector reexporta `ph2d_render::fx_atlas`), com `pack_in_order` ao lado do `pack`.
- **`ph2d-shape-gpu`**: `ShapePass::caixas` (passe de cálculo, grupo `3`) + `caixas.rs`;
  `shape.wgsl` ganha `cs_caixas` (append).
- **`ph2d-fx-op`** / **`ph2d-i18n`** / **`ph2d-panel-vector`**: o nome «Glow» → «Outer Glow» na
  `SPECS` e no mapa nome → chave do painel do Vector (`nomes_do_motor.rs`).
- **`ph2d-node-fx-filter`**: param `scope` (apendado após `kind`), `partition`, `switch_kind`,
  `card_values`; dependência nova `ph2d-color`.
- **`ph2d-app-motion`**: `Engrenagem.formas: Option<&PlacaDeFormas>` (era a camada);
  `PlacaDeFormas::copias_do_quadro`/`passe_irmao`/`afina`; o gesto do cartão chama `switch_kind`.
- **Shell** (`present_chrome.rs`, 3 linhas): passa a placa à `Engrenagem`.
- O tutorial 07: o nó `Drop Shadow` passa a `Shadow Copies` (o PDF por regenerar na W4).

## §3 — Gates e medições

Doc 129 §5.6 (a tabela achado → cura → medido). Relógio: 256 discos 0,55 ms cada forma contra
0,51 camada inteira; 4 096 — 4,64 ms (4 páginas) contra 0,61.

## §4 — Abertos

Doc 129 §7: a sensibilidade da rota da PLACA do `SpriteRenderer` na silhueta (13 texels entre as
duas rotas na camada inteira); 4 096 formas = 4 páginas; U2 (px do projecto); a lei de mistura sem
ficha e a rampa com 2 stops; a camada filtrada por CIMA do mundo; as formas da rota da CPU (Vello)
fora da camada. A W4/W5 no prompt.

## §5 — A integração (11/10, por ordem do Enio)

- **Rebase** dos 69 commits da linha sobre o `main` `09a24afe1` (344 commits à frente desde o
  merge-base `782f90b69`), conflitos pelos ESTÁGIOS do índice; nenhum marcador em nenhum commit
  rebaseado; comparação commit a commit pelo multiconjunto de `+/-` (só divergem as contagens da
  memória recontadas e a adaptação do cartão do Motion Input à Luz 2D).
- **Conflitos:** `project-memory/` (as contagens das famílias RECONTADAS pelo ficheiro de cada uma:
  medição 211, gate 137, gotchas 35, oráculo 27, controlo 25); o ciclo 19 W4 (o cartão do Motion
  Input) contra a Luz 2D nas mesmas listas do Inspector e das `fase_*` da shell — os dois lados, Luz
  primeiro; o `present.rs` (W2/W3 contra a Luz 2D) — a Luz liga-se e só depois
  `plan.banded |= lente || filtrada`.
- **Números que somam, CONTADOS:** `LIVE_SECTIONS` 47 → **48**, `SECTION_COMPONENT` 48 entradas,
  `any_live_section(flags: [bool; 37])` (o `main` e a linha tinham subido 35 → 36 cada um: fundia
  limpo e errado); `PROJECT_SCHEMA` 188 e a tripla `(188, 13, 22)` (o `main` não mexeu no esquema);
  componentes 107 / 108 / 108 (idem); as cenas do Motion sem colisão.
- **`fix(integração)`** (`bd291557d`): a assinatura das 37 secções, a chamada de teste da Luz 2D com o
  parâmetro do Motion Input, `cargo fmt`. ⚠️ Os commits do ciclo 19 W4 até ao penúltimo ficam
  vermelhos nesse ponto (36 contra 37): só o tip compila — um `bisect` por ali salta-os.
- O primário tinha ficheiros por rastrear de OUTRA sessão (uma sonda do Painter e
  `docs/Painter/NewFeaturesWetPaint/`): o `foundational-integrate.sh` recusa por isso; os passos dele
  correram à mão, sem tocar nesses ficheiros, e o `main` avançou por `merge --ff-only`.

# Handoff de integração — `line/Vector`: O GRADIENT MAP de N stops (plano 24 W11)

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `78d770370`
**Estado:** fechada, **pendente de smoke**

⚠️ **Esta é a SÉTIMA wave da mesma abertura de linha,** e as sete **compartilham o bump de schema**
(`PROJECT_SCHEMA` 38). Integre-as juntas — o `main` nunca viu nenhuma:

1. [`…_fx_blend_…`](HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md) — a lei de mistura
2. [`…_turbulence_…`](HANDOFF_INTEGRACAO_line_Vector_turbulence_2026-07-28.md) — a turbulência
3. [`…_morphology_…`](HANDOFF_INTEGRACAO_line_Vector_morphology_2026-07-28.md) — Grow / Shrink
4. [`…_colour_adjust_…`](HANDOFF_INTEGRACAO_line_Vector_colour_adjust_2026-07-28.md) — Color Adjust
5. [`…_duotone_…`](HANDOFF_INTEGRACAO_line_Vector_duotone_2026-07-29.md) — Duotone + Luma to Alpha
6. [`…_atlas_…`](HANDOFF_INTEGRACAO_line_Vector_atlas_2026-07-29.md) — o atlas de raster
7. **esta** — o Gradient Map

---

## O que muda

Um degrau que mapeia a claridade da arte numa rampa de até **8 stops**, com o **trilho** de autoria
no card: barra de preview, um punho arrastável por stop, `+`/`−`, e a swatch do stop em foco pelo
MESMO picker OKLCH das duas pontas do Duotone.

**E ele SUBSUME o Duotone:** dois stops nas pontas são o Duotone **ao byte** — medido no dispositivo,
**0 de 6144 bytes diferem**, em três opacidades. É isso que faz da wave uma generalização em vez de
um 12º tipo que responde à mesma pergunta.

---

## O oráculo é a outra metade do app

**A lei da rampa é a que o app já ship** — paridade com o `gradient_map_lut` do
`ph2d-painter-effects` (a camada de ajuste do Painter, escrita por outra wave, para outro
consumidor): **1 nível de byte** no pior caso, em 3 rampas × 2 modos.

⚠️ **A RÉGUA divergem de propósito, e o número está medido.** O Painter mede claridade em **Rec.601
sobre bytes de display**, esta pilha em **`L` do OKLab** (a régua que o Duotone e o Luma to Alpha já
usam). Em sRGB 128: **0,502** contra **0,600**. A paridade é sobre *"que cor vive em `t`"*, não sobre
*"que `t` este pixel tem"*.

**E o bar do painel sai da MESMA função** — a shell amostra (`fx_live::ramp_preview`), o painel pinta
bytes. Um lerp de conveniência no painel seria a 2ª resposta a *"que cor vive em `t`"*, divergindo em
gama nos meios-tons, e o único lugar onde isso apareceria é uma screenshot.

---

## Os três defeitos, e os dois primeiros foram achados por gates que JÁ existiam

1. ⚠️ **`MAX_FILTER_KINDS` ficou em 14 com `FxOp::KINDS` em 15** — o Gradient Map **não teria botão
   "Add"**: a wave inteira inalcançável. Pego pelo gate de espelho de tetos, que existe para isto.
2. ⚠️ **`mode 1` num tipo pointwise era varrido para o plano do campo de distância** e o degrau saía
   **no-op completo** (byte-idêntico à fonte, com o Linear correto ao lado). A regra perguntava *"tem
   modos, e escolheu o 1?"* — **terceira instância** da mesma enumeração disfarçada de regra. E a
   **quarta** apareceu logo depois: o `BLANK` compartilhado tem `mode: MODE_CONTOUR` (= 1), então um
   Gradient Map novo **nascia em Smooth**.
3. ⚠️ **Uma mutação minha SOBREVIVEU e o defeito era do meu gate** — ver §18.6 do plano: um
   arch-gate sobre o fonte vê FORMA, e dobrar o stop na ponta escura manteve o nome do slot num braço
   inalcançável. A cura foi **mover a decisão** para uma função pura, não reforçar o gate.

---

4. ⚠️ **E o quarto foi o REPORT do Enio, que não estava no trilho** (*"não é possível arrastar os
   pontos de cor"* → *"ainda não posso mover"*): o painel de **camadas do Painter** drenava o stash
   **global** de arrasto para QUALQUER `ValueChanged` e devolvia `Consumed` — então o gesto era
   engolido por um painel que nem estava na tela, com os gates isolados dos dois painéis **verdes**.
   Reproduzido por `PH2D_FX_RAMP_DIAG=1`; cura estrutural + 3 gates novos em **§18.9 do plano 24**.
   ⚠️ **É por isso que esta wave toca três crates fora do vetor** — ver a tabela abaixo.

## Superfície tocada

| | |
|---|---|
| **`PROJECT_SCHEMA`** | **fica em 38** — o `FxOp` ganhou campos, mas ele já viajava dentro da pilha que o 38 cobre; a wave não muda a forma do `ProjectFile` |
| **Contrato congelado** | **intacto** (`architecture_contract_surface` 3 · `_tool_` 4 · `_vector_` 11) |
| **ADR** | nenhum |
| **`Cargo.toml`** | **zero** — nenhuma dep nova, nenhuma crate nova |
| **`ph2d-ecs`** | `FxOp` +`stops`/`stop_pos`/`stop_count` + `GRADIENT_MAP=14` · `KINDS` 14→**15** · `MAX_GRADIENT_STOPS=8` · porta `ramp_for_device` · `mode_selects_the_distance_plan` · `FxKindSpec.takes_ramp` · **`vec_filter_new.rs` NOVO** (LOC) |
| **`ph2d-render`** | `Globals` +rampa (**304 bytes**, pin atualizado) · `ramp_sample` no WGSL · `plan_of` pergunta ao TIPO · **`Globals::for_op`** movido para o dono do uniform (LOC) |
| **`ph2d-editor-core`** | `MAX_FILTER_KINDS` 14→**15** · `MAX_FILTER_STOPS` · 5 ids do trilho |
| **`ph2d-panel-vector`** | snapshot +rampa +`takes_ramp` · `RAMP_PREVIEW_N` · `selected_stop` (vista) · o trilho em `paint_filters` |
| ⚠️ **`ph2d-editor-core`** | **API FOUNDATIONAL REMOVIDA:** `WidgetStore::take_curve_point_drag()` **deixou de existir** → `take_curve_point_drag_if(\|parent\| …)`, que deixa o stash intacto na recusa. É a cura do defeito reportado (§18.9) e o compilador é o gate |
| ⚠️ **`ph2d-panel-painter-layers`** | **crate de OUTRA linha** — o ladrão do gesto: `route_value_changed` passou a perguntar de quem é o arrasto (drenagem extraída para `event/curve_drag.rs`) + os 4 sítios id-gated migrados + gate novo `seam_curve_drag_ownership.rs` |
| ⚠️ **`ph2d-panel-motion-params`** | **crate de OUTRA linha** — 1 sítio migrado (o `if parent != …` dele virou o predicado; o comentário *"put it back is impossible"* morreu com o defeito) |
| **`shells/desktop`** | `ColourSlot` (o `bool` de 2 virou 3) · `apply_picked_colour`/`add_stop`/`remove_stop`/`ramp_preview` · o arrasto por `pending_filter_stop` · **`fx_live_resolve.rs` NOVO** (LOC) · cena `=39` |

---

## Gates

| onde | quantos | o que provam |
|---|---|---|
| `ph2d-render/tests/fx_stack_gradient_map_gpu.rs` (⚠️ `#[ignore]`, precisa de adapter) | 8 | **a subsunção do Duotone ao byte** · paridade com o Painter · ordem de autoria não muda um byte · o degenerado · força por-stop · Smooth achata NO stop interno · cobertura intocada |
| `ph2d-render/src/fx_stack_tests.rs` | 1 | um pointwise com modos próprios nunca cai no plano do campo — **e um falloff sempre cai** (as duas metades) |
| `ph2d-panel-vector/tests/seam_filters.rs` | 3 | o trilho é oferecido só por quem tem rampa (presença E ausência) · punhos alcançáveis e **sem sobreposição** · `+`/`−` chegam ao bus |
| `shells/desktop/src/fx_live_tests.rs` | 4 | a rota por slot · o `+` neutro em Linear **e não-neutro em Smooth** · o piso do `−` · o espelho dos tetos |
| `shells/desktop/tests/the_picker…rs` | 2 | a shell entrega à porta única com o slot (+ controle positivo do scanner) |
| `ph2d-panel-painter-layers/tests/seam_curve_drag_ownership.rs` | 2 | **o arrasto de outro painel sobrevive E não é consumido** + o CONTROLE de que o próprio segue drenado |
| `ph2d-editor-core` (`interaction::state::tests`) | 1 | a recusa do predicado é **não-destrutiva** (é o que torna a recuperação possível) |
| `ph2d-editor-core/tests/architecture_curve_drag_asks_whose_gesture.rs` | 1 | nenhum painel responde *"é meu"* com **tautologia** (`\|_\| true`), com controle positivo de sítios lidos |

**Mutações: 7 tentadas, 6 sangram, 1 não compila** (o tipo a impede — os arrays do snapshot e do
componente têm o tamanho tipado pelas duas constantes).

| # | mutação | resultado |
|---|---|---|
| M1 | a regra antiga do plano (*"tem modos?"*) de volta | RED (o Gradient Map cai em `Field`) |
| M2 | `mode_selects_the_distance_plan` sempre falso | RED (o Contour da família de dentro morre) |
| M3 | o stop dobra na ponta escura | RED (`assert_eq!` na rota pura) |
| M4 | o `+` sorteia a cor em vez de amostrar a rampa | RED (**113 níveis**) |
| M5 | o `−` sem piso | RED |
| M6 | os dois botões do trilho fora do `populate` | RED (*pintados e mortos sob o mouse*) |
| M7 | `MAX_FILTER_STOPS` = 4 | **não compila** |
| M8 | o roteador do Painter volta a drenar tudo (`\|_\| true`) | RED (o seam de posse **e** o arch-gate) |
| M9 | a recusa do predicado deixa de recusar | RED (o gate do primitivo + o seam de posse) |
| M10 | o vetor pergunta pela **linha errada** ao drenar | RED (os **3** gates de arrasto) |
| M11 | o id da rampa de textura do Painter trocado pelo da rampa de Shape | ⚠️ **SOBREVIVE** — a suíte daquele módulo não cobre o forward do arrasto de rampa dele (vão **pré-existente**, de outro dono). Os 5 ids escritos lá foram verificados **lendo a registração** (`parent: ids.edit`) |

---

## Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=39 cargo run -p ph2d-host-desktop --release
```

Oito estrelas em quatro pares, e a cena **imprime o que montou**. O que decide:

1. **O par 1 tem de ser INDISTINGUÍVEL** (Duotone × rampa de dois stops). Se as duas diferirem, a
   subsunção quebrou e a wave perde a razão de existir.
2. **A estrela 4 tem de sair idêntica à 3** — é a MESMA rampa autorada fora de ordem.
3. **No painel** (é o passo que fecha): selecione a estrela 3, abra FILTERS, e no card
   'Gradient Map' — **arraste um punho por cima do vizinho e continue** (o punho sob o dedo não pode
   trocar de stop no meio do gesto) · **clique um punho e depois a swatch Stop** (a cor tem de pousar
   NAQUELE stop) · **`+`** (o desenho não pode mudar) · **`−`** (para em dois).

⚠️ **O passo (a) é o que o report do Enio derrubou duas vezes** — se um punho não se mover, rode com
`PH2D_FX_RAMP_DIAG=1`: `[ramp] linha … · N punho(s) · store armado: true` diz que o punho existe e
está armado, e `[ramp] painel entregou:` diz que o gesto **atravessou o registry de painéis** (era
exactamente essa linha que faltava quando um painel alheio roubava o arrasto).

As outras cenas de FX (`=33` a `=38`) servem de regressão pelo mesmo critério: **igual ao de antes**.
E o **Painter** entra na regressão desta vez, porque a cura tocou a crate dele: as curvas / o Gradient
Map das camadas de ajuste, o editor de falloff, o dial de tilt do Wet Paint e as duas barras de rampa
(Grain e Shape) têm de continuar a arrastar.

---

## Aberto, nomeado

- **O trilho não foi extraído para um widget compartilhado**, e o custo está no plano §18.7: as
  constantes de geometria existem em duas cópias (esta e a do Painter), hoje batendo valor a valor. O
  que É compartilhado é o **gesto** (`InteractiveState::CurvePoint`), que é o precedente do repo.
- **A rampa não é animável** — ela é estado autorado do componente, como as duas cores do Duotone.
  Animar stops é outra pergunta (quem interpola: as posições, as cores, ou a rampa amostrada?).
- **O `+` escolhe o maior vão**; não há gesto de *inserir aqui* (clicar na barra). É a adição óbvia
  seguinte, e ela reusa o mesmo `add_stop` com um offset informado.
- **Nenhum stop é removível pelo punho** (arrastar para fora da barra, o idioma do Photoshop) — o
  `−` é o único caminho. Decisão de produto, não impedimento técnico.

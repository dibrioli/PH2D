# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 + W5b + W6 + W7 + W8 + W9 «Fidelidade ao Excalidraw» → `main` (2026-10-10)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem
> pusha. **Substitui** o [handoff W8](HANDOFF_INTEGRACAO_line_MiroClone_W8_2026-10-10.md) como o último da
> linha. W5…W8 (smoke do dono OK em todas, a W8 com as três correcções pós-smoke também OK em 10/10) não
> foram integradas e a W9 vai por cima, na mesma branch — **integra-se tudo de uma vez**; para as
> anteriores valem os handoffs delas. Briefing: [continuação W9](HANDOFF_CONTINUACAO_W9_2026-10-10.md).
> Estudo (as leis XA/XB/XV com número, as recusas, a tabela do formato, M1–M10):
> [`../07_fidelidade_excalidraw.md`](../07_fidelidade_excalidraw.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `2a925859d` — **o `main` não andou** (`git rev-parse main` = `2a925859d`) ⇒ `--ff-only` se ninguém aterrar antes |
| commits da W9 | `cf569d02c` (núcleo) · `c56682526` (oráculo) · `c863826a8` (cena 11) · `877aae148` (curas do portão e da auditoria) · `2de1cdcff` (mutação) + este handoff; o estudo e a decisão do dono vieram antes (`ac95616c2`, `c5a83477f`, `3b4ec7784`) |
| diff da W9 | `git diff --shortstat 3b4ec7784..HEAD`: 69 ficheiros, +14 147 / −629 (código em `crates/`: 46 ficheiros, +3 436 / −448; o resto são as saídas do oráculo re-corridas, a foto e o ficheiro do dono) |
| smoke | **por fazer pelo dono** (cena 11, §8) |

O que a W9 trouxe ao Quadro, pela ordem do estudo §5: **M1** a seta pára ANTES da forma (`5 + traço da
forma/2`, como o Excalidraw) em TODO o quadro — decisão do dono 10/10 —, com a âncora nova `Orbit` (a lei
XB1: a recta entre os dois focos, no 1.º ponto da faixa); **M3** a seta «redonda» que vem de um
`.excalidraw` é a curva DELE (`Route::Spline`, Catmull-Rom 1/6: com dois pontos, recta) e a curva do nosso
quadro não muda (ordem do dono); **M2** a ponta de seta tem o tamanho do Excalidraw por tipo e ENCOLHE
para metade do último trecho (antes uma seta ≤ 20 ficava sem ponta); **M4/M6** o gesto de ligar prende em
órbita junto ao contorno e o escritor fala as DUAS gramáticas de ligação (`mode`/`fixedPoint` do
excalidraw.com e `focus`/`gap` do 0.18.1); **M5** a nota `stickynote`; **M7** o preenchimento às riscas, a
rugosidade 2 e a semente importados; **M8** o alinhamento (horizontal e vertical) e a letra do texto;
**M9** o triângulo vazado e as seis pontas de cardinalidade (pé-de-galinha); **M10** a caneta de largura
constante. **Cena 11** (`PH2D_BOARD_SMOKE=11`): o `bv.excalidraw` do dono, a foto do Excalidraw, setas
curtas de 10 a 60, desenhar uma seta que pára antes da caixa.

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | porque é aditivo |
|---|---|---|
| `ph2d-board-model` | **formato 9** (`FORMAT_VERSION` 8 → 9): `Style::look` (`Look`: `fill_style`, `cartoonist`, `align`, `valign`, `letter` — cinco campos de 1 byte) no FIM do `Style`; `Org::seed` no FIM do `Org`; `Anchor::Orbit`, `Route::Spline`, `Head::TriangleOpen` + `One/Many/OneOrMany/ExactlyOne/ZeroOrOne/ZeroOrMany` no FIM dos enums (postcard posicional); `ORBIT_GAP`; `Element::set_imported_seed` | `legacy.rs::read_v8` (+ `StyleV8`/`KindV8`/`OrgV8`, e os leitores 4–7 passam por `KindV8`); fixture `fixtures/format_v8.bin` **gravada pelo build `3b4ec7784`** (teste temporário, apagado) — gate `a_format_8_file_still_opens_with_the_default_look`. `serde(default)` no `look` e na `seed`: o `customData` de um `.excalidraw` escrito pela W8 lê-se igual (fixture `ph2d-board-excalidraw/fixtures/w8_escrito.excalidraw`, gate `a_w8_file_still_carries_our_elements`). ⛔ **Tamanho medido**: com a semente dentro do `look` o `Style` ia de 40 a 56 bytes e cada `Element` de 208 a 224 (o clippy `large_enum_variant` apanhou-o no `BoardOp`); como ficou, `Style` 40 e `Element` 208 — **zero bytes a mais por elemento**. ⛔ O `PROJECT_SCHEMA` **não** se toca: o quadro é um blob com versão própria |
| `ph2d-board-geom` | `band_entry` (a marcha por esferas na distância verdadeira ao contorno; a elipse pela elipse de semi-eixos + vão, a dele), `ray_exit_band`; `text_origin` recebe `VAlign` (2 chamadores); `outline_normal` mede a partir do ponto DO CONTORNO | funções novas; a correcção do `outline_normal` só muda o resultado num ponto FORA do contorno (antes nunca pedido) |
| `ph2d-board-route` | `gap(el)`; o centro pára a `g`; `Orbit` (XB1 nas rectas, porta + vão na curva do Miro e no cotovelo); `spline()`; `heads.rs` (novo: tamanhos, encolher, cardinalidade); `curves.rs` (cortado do `lib.rs`); `Routed::legs`; `head_reach` no lugar de `head_scale`/`HEAD_SCALE` (2 chamadores no render); a chave da cache leva o vão e as pontas; `anchor_for` devolve `Orbit` na faixa e a faixa por fora conta o vão | os chamadores todos actualizados no mesmo commit (render, editor-core); nenhum fora do quadro |
| `ph2d-board-layout` | `Face { hand, align }` no lugar do `hand: bool` (`impl From<bool>`: os chamadores que só passam a letra ficam iguais); `TextEdit::align`; `text_height` recebe a `Face` (1 chamador) | aditivo pelo `From<bool>` |
| `ph2d-board-render` | `paint_shape` pela `Face` e pelo `valign`; o rascunho lê o `look` (`base_roughness`, o «cartoonist» 2 só no importado; o preenchimento às riscas só no importado); `hand_reach` (a folga da camada de opacidade) | família do quadro |
| `ph2d-board-edit` | `face(el)`, `hand_lettered` pela `Letter`; as cópias esquecem a semente importada (3 sítios); `wire.rs` aceita `Orbit` | família do quadro |
| `ph2d-board-excalidraw` | leitor (as duas gramáticas em `bind.rs`, o estilo em `style.rs`, `stickynote`, caneta constante, o aspecto); escritor (as duas gramáticas, o aspecto, o `y` pelo `verticalAlign`, o cotovelo pelo ponto do contorno); `head_name`/`head_of` das pontas novas | crate da linha (W8) |
| `ph2d-editor-core` | `board_bar_look.rs`: `Route::Spline` sem dica e com ícone recto (a barra não a oferece) | 2 braços de `match` |
| `ph2d-i18n` | `board_fidelity.rs` (módulo novo) | chaves novas `board.smoke.fidelity.*` |
| `ph2d-app-board` | `smoke_fidelity.rs` (cena 11) + gate; cena 3 prende o «saltar» e o «não» em órbita; dev-deps `ph2d-board-geom`, `serde_json` | família do quadro |
| `Cargo.lock` · `.typos.toml` | as dev-deps acima · `^t_hach$` (o id do rectângulo «hachure» na fixture do oráculo) | — |
| shell | **nada** | — |

**Contratos congelados (§6):** nenhum tocado — `git diff --name-only 3b4ec7784..HEAD | grep -E
'ph2d-nodegraph|ph2d-tool|ph2d-vec|ph2d-vector|ph2d-imageio|schema|project\.rs'` → vazio; o diff não
contém `PROJECT_SCHEMA`, `NodeOp`, `Tool=` nem `VectorOp`. O catálogo `Marker` (`ph2d-vec-scene`) **não**
mudou: as pontas de cardinalidade desenham-se no `ph2d-board-route/src/heads.rs`.

## 3. Números MEDIDOS

- **O oráculo** (Excalidraw 0.18.1 e o `next` 0.18.0-143b5b6 do excalidraw.com, as 220 saídas do estudo,
  nada re-corrido para as leis):
  - **o ficheiro do dono**: as três setas abrem RECTAS, presas em órbita nas seis pontas, a **≤ 0,05 px**
    de onde o excalidraw.com as gravou (antes: um S de 41–83 px e pontas até 45 px DENTRO das formas);
  - **XB1** em todos os casos `B_*` depois de o next re-ligar as setas: **70 pontas presas**, rectângulo e
    elipse a ≤ 0,05 px; o losango a ≤ `1/sen θ` (a assimetria `⌊w/2⌋+1` dele, recusada em W1: numa seta
    rasante a uma aresta o 1 px da aresta estica até **1,68 px** medido — a tolerância de 1 px do estudo
    não o previa) e a NOSSA ponta exactamente a `g` do nosso contorno;
  - **XA2**: as 39 setas `arrow` (L 8…400 × traço 1/2/4) com o fundo `min(25, L/2)·cos 20°` a ≤ 0,05; as
    63 de `P_tipos_comprimento` (barra, círculo, triângulo, losango, 3 de cardinalidade) com o tamanho
    dele; as 18 de cardinalidade de `P_tipos` peça a peça a ≤ 0,1;
  - **XA1**: as pegas do SVG dele (rugosidade 0) ao centésimo, em 2 e 3 pontos e na linha;
  - **o NOSSO ficheiro** passado pelas duas versões (corrida nova, `w9_nosso_casos.json`): nenhuma mexe ao
    abrir; o next re-liga as **5 pontas presas a ≤ 0,05 px** de onde as escrevemos.
- **Tamanhos**: `Style` 40 B e `Element` 208 B, os de antes (§2).
- **Régua das rotas** (`ph2d-board-render/tests/it/measure_route_cost.rs`, `--release`), 100 mil formas e
  10 mil setas, base `3b4ec7784` × W9 **intercaladas** (worktree e `target` à parte para a base, duas
  rodadas cada, o MÍNIMO, `load` 5–12):

  | | base | W9 | |
  |---|---|---|---|
  | frio | 56,7 ms | 58,3 ms | +2,8 % |
  | arrasto (1 forma com seta) | 4,08 ms | 4,21 ms | +3 % |
  | desenho | 12,1 ms | 12,3 ms | +1 % |
  | parado | 0 | 0 | |

  ⇒ a marcha do vão custa ~3 % no frio; **nenhum tecto**.
- **Antes da W9, medido**: uma seta de comprimento ≤ 20 não tinha ponta (o `chord_cut` não achava a
  profundidade fixa de 23,5); agora tem, do tamanho do Excalidraw (cena 11, lição 3).

## 4. Portão batched (1× sobre `2a925859d..HEAD`)

| portão | resultado |
|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`, `PH2D_GPU=1`, `NO_FAIL_FAST=1`) | ✓ **17 558/17 558** (3.ª passagem, depois das curas e da mutação, 128 s) |
| `cargo check --workspace --all-targets` com `CARGO_BUILD_WARNINGS=deny` | ✓ (2.ª passagem; a 3.ª só tocou `legacy*.rs` e um teste — clippy `-D warnings` verde sobre eles) |
| clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs`, `--keep-going`, `-D warnings`) · `ph2d-board-export --features gpu --all-targets` | ✓ 0 avisos (3.ª) · ✓ (2.ª) |
| `cargo fmt --all -- --check` | ✓ |
| `cargo machete` · `cargo deny --all-features check` | ✓ · ✓ (2.ª; as dependências não mudaram depois) |
| `ph2d-editor-core --test it` · `ph2d-host-desktop --test it` (sem filtro) | ✓ 536 (3.ª) · ✓ 791 (2.ª) |
| `censos-da-arvore-combinada.sh` | ✓ 114/114, 12/12 censos correram |
| `typos` | ✓ |
| oráculo Excalidraw re-corrido sobre o escritor final (W8 `w8_ph2d_quadro` + `w9_X_nosso` nas duas versões) | ✓ (24 testes do `.excalidraw`) |
| fotos da cena 11 (`fotografa_cena.sh`, tela virtual, duas) | ✓ |

Nomeados verdes: `workspace_src_files_under_loc_cap` (o tecto de 700 — 2.ª passagem apanhou ainda o
`legacy.rs` a 711 com o `OrgV8`: cortado para `legacy_v8.rs`, 515 + 211), `shell_files_respect_hr18_loc_cap`
(`file_loc_caps`), `the_shell_only_shrinks`, `clamp_calls_in_ui_are_safe_or_justified` (`arch_safe_clamp_only`),
`no_magic_numeric_in_widget_or_screens`, `no_hex_color_literals_in_widget_or_screens`,
`architecture_*contract_surface` — todos dentro do nextest impactado (a shell não mudou).

⚠️ O que a 1.ª passagem apanhou (todos curados em `877aae148`): `cargo fmt` em 13 ficheiros; o clippy
`large_enum_variant` (os 16 bytes do `look` em cada elemento — §2); o tecto de 700 linhas em três ficheiros
(`route/lib.rs` 752, `excalidraw/read.rs` 782, `fidelity_tests.rs` 875 — cortados por responsabilidade:
`curves.rs`, `bind.rs`, `style.rs`, `fidelity_file_tests.rs`); `typos` no id `t_hach` da fixture. Os censos
da 1.ª passagem não correram (a placa estava presa pelo próprio portão) — corridos à parte (abaixo).

## 5. Auditoria de 2 lentes (antes da mutação) — curadas com teste

**Lente 1 (as leis contra o estudo e o oráculo)** e **lente 2 (o ecrã: o que o artista consegue fazer)**,
independentes, só de leitura; curas em `877aae148`:

| achado | lente · peso | cura | o teste que o via vermelho |
|---|---|---|---|
| agarrar a ponta presa com zoom (≳ 230 %) e mexer 1 px SOLTAVA-A: a ponta está a `g` da forma e a faixa de ligação em px fica mais fina que o vão | 2 · alta | a faixa por fora conta o vão (`anchor_for`) | `w9::grabbing_a_bound_end_with_zoom_keeps_it_bound` |
| a curva do Miro entrava torta numa elipse/pílula RODADA presa ao meio do lado: a junção da normal comparava-se com o ponto de fora | 1+2 · média | a junção mede-se no ponto do contorno | `the_normal_off_the_outline_is_the_normal_of_its_nearest_point` |
| a cópia de um elemento importado tremia IGUAL ao original (a semente viajava) | 1+2 · baixa | as cópias esquecem a semente (duplicar, colar, Alt-arrastar, ponto azul) | `w9::a_copy_of_an_imported_element_trembles_its_own_way` |
| a nota com a letra à mão importada media a altura na letra da interface | 2 · baixa-média | `text_height` recebe a `Face` | `w9::a_note_with_the_imported_hand_letter_measures_in_that_letter` |
| o rótulo da seta: a edição alinhava pelo `look`, o desenho centrava | 2 · baixa | o desenho pela mesma `face(el)` | (a porta é a mesma função; sem teste próprio) |
| o escritor punha o texto sempre ao meio com `verticalAlign: top` | 1 · baixa | o `y` pelo `valign` | `the_text_sits_where_its_valign_says_and_an_elbow_writes_a_point_of_the_outline` |
| o cotovelo escrevia um `fixedPoint` fora da caixa (a ponta com o vão): num estranho o vão escalava com a forma | 1 · baixa | grava o ponto do contorno | o mesmo |
| a forma importada «cartoonist» e meio transparente era cortada a direito (a camada só tinha a folga do traço) | 2 · baixa | `hand_reach` (a conta do `line_ops` do porto) | `a_hand_drawn_shape_stays_inside_its_opacity_layer` (597 formas) |
| a cena 3 ensinava pontos fixos a encostar (o gesto já não os faz) | 2 · baixa-média | o «saltar» e o «não» em órbita | `scene_3_teaches_arrows_that_stop_before_their_shapes` |

Decididos e escritos no estudo §3 (não curados, com o porquê): o círculo da ponta fica TANGENTE ao bico (o
dele é centrado — taparia o vão), e o miolo da forma continua `Center` (a face do Miro e o afastamento das
paralelas), agora com o vão. Ficam ABERTOS (§9): o 0.18.1 depois de mexer, o ponto do realce, a letra
importada fixa.

## 6. Prova de mutação

25 mutações sobre as leis e as curas (o agente `mutacao`, os quatro controlos: filtros verdes e não vazios,
âncoras únicas, tudo compila, nenhuma corrida de zero testes; cada ficheiro restaurado com `touch`, árvore
limpa no fim): **24 sangraram**. A sobrevivente — o foco do legado `{focus, gap}` forçado ao centro passava
nos dois casos horizontais (a recta deles já passa pelo centro) — foi tapada com o caso diagonal
`B_legado_focus05_gap10_diag` (`2de1cdcff`) e re-mutada: **sangra (48,5 px)**. Nota: a nº 2 (`ORBIT_GAP` a
0) sangra no `xb1` mas não no teste do roteador, que calcula o esperado pela própria `gap()` — quem segura o
5 é o oráculo, como deve.

## 7. Fixtures, instrumentos e o que vive versionado

- `ph2d-board-model/fixtures/format_v8.bin` (gravado pelo build `3b4ec7784`).
- `ph2d-board-excalidraw/fixtures/w8_escrito.excalidraw` (a entrada do oráculo da W8, escrita pelo escritor
  da W8 antes de a W9 a regravar).
- Oráculo: `entradas/w8_ph2d_quadro.excalidraw` regravada (o escritor mudou) e o esquema re-corrido
  (`saidas/w8_ph2d_quadro.*`); **`entradas/w9_nosso_casos.json`** (à parte do `w9_casos.json`, cujo sha vai
  nas 220 saídas) e `saidas/w9_X_nosso.{0181,next}.{json,svg}`. `w8_sondas.json` e `w8_tipos.meta.json`
  ficam os da W8 (só sorteados/data mudavam). README do oráculo §W9: como se regrava e re-corre; a página
  `www/` instalada era de 07/10 e reconstrói-se pela secção «Página» do `instala.sh`.
- `docs/MiroClone/bv.excalidraw` (o ficheiro do dono, cópia de `entradas/w9_dono_bv.excalidraw`) e
  `docs/MiroClone/bv_excalidraw.png` (o SVG do next pelo `resvg` 0.48.1, Apache-2.0/MIT, 2×) — as lições 1
  e 2 da cena nomeiam-nos e um gate confirma que existem.

## 8. Smoke (o comando exacto)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=11 ./target/smoke/ph2d-host-desktop
```

A cena abre com o desenho do dono em cima à esquerda, as setas curtas por baixo, as caixas «Start here» e
«Drop inside» à direita e as quatro lições: (1) comparar com `docs/MiroClone/bv_excalidraw.png`; (2) abrir
`docs/MiroClone/bv.excalidraw` pelo menu da aba; (3) as pontas que encolhem; (4) desenhar com `A`, e o
`Ctrl` ao largar. Fotografada duas vezes pelo `fotografa_cena.sh` (a 1.ª partia o caminho longo a meio e
punha os números ao lado das setas — curado). As cenas 1–10 continuam; a 3 e a 5 mostram agora o vão.

## 8-bis. Depois do fecho — o 1.º smoke do dono (10/10): «funcionou», e um defeito

*«Ao criar uma nova seta, se aplicar a opção hand-draw, a seta fica deformada»* (duas capturas: a curva
em «U» do Miro em rascunho afastava-se da guia azul). **Não era da W9: era da W4.** O `hand_line`
(`ph2d-board-render/src/sketch.rs`) dava à `curve` do rough.js TRÊS pontos por cúbica, e o Catmull-Rom 1/6
dela por tão poucos pontos corta o canto à saída (a tangente da ponta é a corda para o 1.º ponto) e chega
torto à ponta. **Medido**: a curva em «U» afastava-se **31,8** da rota. Cura (`curve_points`): os três de
antes e MAIS só onde o Catmull-Rom dele se afasta da curva verdadeira mais que `HAND_CURVE_TOL` = 0,5
(o tremor continua o dele; 29 e 25 pontos nas duas setas do teste; **~40 µs** por seta em `--release`, só
quando a seta muda — o rascunho está em cache; a 1.ª versão, com o `nearest` a 1e-9 em todos os
segmentos, custava ~110 µs). Gate `a_hand_drawn_curved_arrow_follows_its_route` (vermelho com a lei
antiga: 31,8), na curva do Miro e na do Excalidraw com pontos; os portões do rascunho contra o oráculo
(`rascunho_setas`) continuam verdes. Fotografada (cena 11 com uma seta em «U» temporária, retirada).

## 9. O que fica ABERTO

- **O 0.18.1 depois de MEXER uma forma** (lente 1, média): ao abrir o nosso ficheiro as duas versões
  mostram as pontas onde as escrevemos (gate), mas se o artista mexe uma forma NO 0.18.1 ele re-liga pela
  lei antiga (mira ao centro com o `focus: 0` que escrevemos; o `inside` não existe lá) e as pontas em
  órbita fora do centro saltam (medido na corrida: 9,5–36,9 px). O excalidraw.com (next) re-liga certo
  (gate, ≤ 0,05). Curar pede MEDIR primeiro a lei do `focus` do 0.18.1 (uma varredura de `focus` × forma ×
  ângulo no oráculo) e escrever o `focus` inverso; os ficheiros do PRÓPRIO excalidraw.com abertos no 0.18.1
  ficam pior (sem `focus`/`gap`). Baixo para o dono (usa o excalidraw.com).
- **O ponto do realce** ao arrastar uma ponta marca o FOCO (no contorno), não onde a ponta vai parar (a `g`
  dali); o oráculo do realce do Excalidraw não foi corrido (lente 2, baixa).
- **A letra importada é FIXA** (`Letter::Hand`/`Sans`): o botão «Rascunho ↔ Final» não a troca e a barra não
  tem como a devolver ao modo — é a do ficheiro dele, mas é pergunta de PRODUTO para o dono (lente 2).
- Da tabela do estudo §4, de onda própria ou baixo (inalterados): a `line` fechada `polygon:true`
  (`ShapeType::Polygon`), o `link` URL, o `labelPosition`, o `customData` de terceiros, imagem/embeddable;
  o `zigzag` lê-se e escreve-se mas desenha-se às riscas (não portado nem medido, estudo §6.4).

## 10. Para o integrador

- Ordem: a linha é uma só (W5…W9 na mesma branch); `main` = merge-base ⇒ `--ff-only`.
- O degrau do formato do QUADRO (8 → 9) é próprio do blob (`board.rs`), não soma com o `PROJECT_SCHEMA`.
- `Cargo.lock`: só as dev-deps do `ph2d-app-board` (`ph2d-board-geom`, `serde_json`) — regenere, não à mão.
- `.typos.toml`: `^t_hach$` (W9), com `^FlateDecode$` (W8) e as da W7.
- CLAUDE.md §5.1 (Quadro) aponta este handoff e `PH2D_BOARD_SMOKE=1..11`.

## 11. A máquina

Limpa: nenhum processo desta linha (`pgrep -af 'ph2d|cargo|rustc'`); quem segura a placa
(`fuser -v /dev/dri/*`) é o smoke de OUTRA linha (`line-ObjectsAndComponents`, pid 1253236) — deixado.
`rm -rf target/*/incremental`: 36 GB recuperados (debug 32 G, smoke 4,2 G); disco 47 %.

## 11-bis. O binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, duas vezes; a 2.ª:

```text
    Finished `smoke` profile [optimized] target(s) in 0.24s
```

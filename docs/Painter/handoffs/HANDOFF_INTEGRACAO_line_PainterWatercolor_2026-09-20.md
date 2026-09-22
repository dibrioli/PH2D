# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, 2026-09-20

> **A costura dura que o retorno sobre o próprio traço deixava, curada.** Report do dono com foto:
> *«ao traçar voltando sobre o próprio traço (sem mouse up), o pincel lava parte da borda escura …
> mas deixa por baixo uma borda plana dura pixelada. Mesmo se aumento ao máximo Rewet e Smudge, não
> consigo me livrar dessa borda.»*
>
> Mecanismo, estado da arte, desenho e recusas: [`docs/Painter/40`](../40_a_costura_dura_do_retorno_sobre_o_proprio_traco.md)
> (a análise, sem código) e [`docs/Painter/41`](../41_a_reserva_e_um_nivel_disputado.md) (a cura).
> **Este documento é para o INTEGRADOR** — superfície de colisão, contadores, e o que uma leitura
> rápida do diff entende ao contrário.

---

## §1 — Coordenadas

| | |
|---|---|
| ramo | `line/PainterWatercolor` |
| HEAD | ver `git log -1` (o §12 acrescentou commits depois de este quadro ser escrito) |
| base (merge-base = ponta do `main`) | `395da6a55` |
| commits | **10** (5 da costura + 5 do item 4, §12) |
| ficheiros | **31** |
| `shells/desktop` | **0 linhas** — a linha não toca na shell |
| crates novas | **nenhuma** |
| pacotes externos novos | **nenhum** (`Cargo.lock` intocado) |

⚠️ **O `main` andou UM commit depois de a linha abrir** (`395da6a55`, que edita `CLAUDE.md`,
`DIRETRIZ.md` e o `HOWTO_partir_uma_familia_da_shell.md`). O rebase está **feito e conferido**:
`git range-diff ORIG_HEAD...HEAD` dá `=` nos cinco commits. ⛔ **Um `git diff main..HEAD` tirado antes
do rebase acusava a linha de APAGAR 72 linhas daqueles dois docs de processo** — é o artefacto de
comparar contra um `main` que andou, e a régua certa é o **MERGE-BASE**.

---

## §2 — Superfície de colisão

**Contadores partilhados: TODOS intactos.** `PROJECT_SCHEMA` · `VEC_SCENE_SCHEMA_VERSION` ·
`FLIP_SCHEMA_VERSION` · `DOC_VERSION` · `FIELD_DOC_VERSION` · os três registos de componentes
(`ph2d-ecs`, `ph2d-render`, `ph2d-script`) — **delta `0` em todos**. Esta linha não grava nada: o mapa
de reserva é estado **de traço vivo**, morre no pen-up e nunca entra no ficheiro.

**Contratos congelados (§6): nenhum encostado.** Toda a mudança vive dentro de
`crates/ph2d-tool-painter/src/tool/paint/`. ⚠️ O contrato de *pintura* do Painter foi **revogado** pelo
ADR-0099; o que sobrevive (`ph2d-painter-effects`) não é tocado, e o `Tool`/`CanvasPaintTool` também
não.

**Ficheiros — onde um merge textual pode doer:**

| ficheiro | natureza |
|---|---|
| `watercolor_reserve.rs` · `watercolor_reserve_tests.rs` | **NOVOS** — não colidem |
| `tests/watercolor_selfseam.rs` | **NOVO** |
| `watercolor_accum.rs` (`+88`) | ⚠️ **o mais disputado**: o depósito e a cadeia do Smudge |
| `watercolor_field/style.rs` (`+22`) | campo novo no `WetStrokeStyle` — ⚠️ struct partilhada |
| `watercolor_render.rs` · `watercolor_render/window.rs` | a montagem do campo e o `pad` da janela |
| `state.rs` · `state_default.rs` · `paint.rs` · `stamp_route.rs` · `stroke_lifecycle.rs` · `watercolor_backdrop.rs` · `watercolor_smudge.rs` · `tests.rs` | linhas soltas (declarar o plano novo, dimensioná-lo, limpá-lo) |
| `docs/Painter/{00_INDEX,40,41}` · `docs/Painter/ferramentas/oraculo_costura/` | docs e o oráculo |

---

## §3 — Foundational tocado

**Nenhum.** Tudo dentro da crate da família. O único ponto que se lê como partilhado é o campo novo
`WetStrokeStyle::reserve_r` — mas a `WetStrokeStyle` é **privada da `ph2d-tool-painter`**
(`pub(in crate::tool::paint)`), e o campo é **apendado**.

---

## §4 — Constantes, campos e variantes novos (com o valor e de onde ele saiu)

| nome | valor | de onde |
|---|---|---|
| `RESERVE_RIM_RAMP` | `0.15` | a casca da beira que a lei antiga já usava — ela **mudou de sítio**, não de número (era aplicada DENTRO do `max`, hoje é o factor `T`) |
| `RESERVE_ROUND_FRAC` | `0.10` | fracção do raio do pincel para o alisamento base — MEDIDA em `measure_the_self_seam` |
| `CLAIM_TAIL_DRY` | `0.38` | a cauda do próprio feather do depósito (a seco) |
| `REWET_DIFFUSE_FRAC` | `0.25` | o Rewet alarga a difusão à escala do PINCEL, não do Bleed — MEDIDA |
| `RESERVE_R_MAX` | `256` | ⚠️ o recurso é a **área da janela do composite**, que cresce `R` para cada lado |
| `WetStrokeStyle::reserve_r` | `u16` | capturado por dono, como o resto da geometria |
| `PaintState::stroke_deplete_prox` | `Vec<u8>` | **plano novo** — `255` no centro do dab, `1` na beira, `0` = a lavagem nunca tocou |
| `PaintState::wet_level_smear_pos` | `Option<[f32;2]>` | a cadeia do Smudge sobre os NÍVEIS |

⚠️ **O `stroke_deplete` MUDOU DE SIGNIFICADO** e isso é o que um merge textual não vê: ele guardava
`v · rampa_da_beira` e passa a guardar o **NÍVEL nu** da reserva. O afilamento da beira derivou-se para
o plano irmão. *Quem fundir uma linha que leia `stroke_deplete` cruamente tem de reconferir o leitor.*

---

## §5 — O que SÓ o `ship.sh` / a árvore combinada apanham

- Os **30 censos de texto** (HR-15) e os tectos de LOC são propriedades da SOMA. Corridos aqui
  **depois do rebase**: `censos-da-arvore-combinada.sh` → **`127/127` verdes**, com o controlo do
  filtro a dizer `12 de 12 censos correram`.
- A linha **não acrescenta literal de UI nenhum** (nenhum painel tocado, nenhum rótulo novo).
- Tectos de LOC: nenhum ficheiro desta linha os estoura. O `watercolor_render.rs` foi **cortado** na
  própria wave (a construção do campo desceu para trás da porta `PainterTool::reserve_fields`) e
  fechou em `697` de `700`.

### §5-bis — O portão de fecho, corrido (todos os números)

| passo | resultado |
|---|---|
| `nextest-impacted.sh` | ✅ **`15 469` testes, `15 469` passaram**, `13 230` saltados |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo machete` | ✅ |
| `check-standalone-optional.sh` | ✅ |
| `check-workflow-packages.sh` | ✅ |
| `censos-da-arvore-combinada.sh` (pós-rebase) | ✅ **`127/127`**, controlo do filtro `12 de 12` |
| `cargo fmt --check -p ph2d-tool-painter` | ✅ |
| as **10** vassouras clean-room vivas | ⚠️ `5` acusam — **todos pré-existentes** (§5-ter) |

⚠️ **As vassouras: `4` de `10` acusam e NENHUM achado é desta linha.** ⛔ Os tokens **não são
citados aqui de propósito** — escrevê-los neste documento acrescentaria uma ocorrência à dívida que
ele descreve, que é a lei *«um instrumento que procura uma agulha não pode CONTÊ-LA»* (a vassoura
guarda-as em base64 pela mesma razão). São dois nomes internos do alvo, num identificador de painel
e num parâmetro de espaçamento, em quatro ficheiros da família do Painter. Duas provas
independentes:
**(a)** nenhum aparece como ADIÇÃO em `git diff <merge-base>..HEAD`; **(b)** os quatro ficheiros já os
carregam **no merge-base**, com as contagens idênticas (`1`, `2`, `2`, `1`). Três dos quatro ficheiros
esta linha nem toca. ⛔ A triagem deles é do **R**, não desta linha — e esta linha **não tem parede**:
o oráculo dela é a `libmypaint` (**ISC**), porta aberta pelo §0.9.

⛔⛔ **E uma LIÇÃO DE INSTRUMENTO que quase entrou neste documento como facto.** A primeira corrida do
controlo do merge-base devolveu **`0` ocorrências nos quatro ficheiros** — o que, lido à letra, diria
que os achados eram NOVOS, o oposto da verdade. A causa foi o shell a comer o `:c` de
`$B:crates/…` (`fatal: Not a valid object name 395da6a55rates/…`) com o erro engolido por um
`2>/dev/null`, deixando o `grep -c` a contar **entrada vazia**. ⚠️ *Um zero de «não medido» e um zero
de «limpo» são o mesmo byte* — a cura foi citar o caminho (`"${B}:…"`) e **conferir primeiro que o
objecto existe**, e só então acreditar no número.

---

## §6 — Catracas baixadas

**Nenhuma.** ⭐ E uma catraca foi **apertada**: `SINGLE_PASS_SLACK = 16` bytes do canal G, com a
medição (`11`) escrita ao lado — a folga de uma passada só entre a leitura bilinear e a de
vizinho-mais-próximo na casca de `15 %` do raio.

---

## §7 — As provas de mutação, e o que elas custaram

**`8` de `8` sangram**, com CONTROLO do arnês nas duas pontas (`18` verdes, `0` vermelhos antes e
depois). A tabela por mutação está no [doc 41 §7](../41_a_reserva_e_um_nivel_disputado.md).

⛔⛔ **A 1.ª ronda deu `5` de `8`, e as três que faltaram eram defeito do AUTOR — cada uma nomeia uma
lei para a próxima wave:**

1. **Uma mutação que PENDURA lê-se como uma que NÃO COMPILA.** A agulha apagava o único `x += 1` do
   ramo seco do varredor de troços ⇒ laço infinito ⇒ nunca houve linha `test result:`, e o arnês leu a
   ausência dela como falha de compilação. *Para quem faz parse da saída, não terminar e não compilar
   são o mesmo silêncio.*
2. **Quatro gates sobre o mesmo objecto podem ter todos o MESMO ponto cego, e a contagem deles lê-se
   como cobertura.** O afilamento da beira não tinha régua: três dos quatro gates do campo amostram
   onde `prox = 255` (ali `T = 1` **por construção**) e o quarto afirma sobre o plano do NÍVEL.
3. **Um gate cujo comentário promete o que a régua dele não alcança é o verde que não afirma nada.**
   O `…incremental_equal_to_full` trazia escrito *«sem o `reserve_reach` … é aqui que o pixel fica
   velho»* e a mutação deixava-o verde. ⚠️ **Mudar o raio da fixtura NÃO curou** (2.ª ronda), e foi
   isso que provou que o gate afirmava outra coisa — a metade partiu-se em **premissa** (unidade) +
   **fiação** (texto), e o comentário foi retractado.

---

## §8 — Premissas que a medição derrubou

1. ⛔ *«a `r = 80` a janela reserva só o Bleed (`7`)»* — **falso**. Ela reserva
   `pad = reach + ceil(warp) + 2 = 22`, e o `pad` é aplicado **duas vezes** (região de saída, depois
   janela de leitura). A conta certa estava no ficheiro; eu escrevi-a de memória.
2. ⛔ *«a `r = 120` a ausência do `reserve_reach` morde»* — **falso**: a margem **é** deficiente
   (`22` contra `R = 30`) e **a imagem ainda concorda a dois níveis**, porque o erro do campo mora a
   `22 px` de qualquer dab daquele quadro e o quadro seguinte reescreve por cima.
3. ⛔ *«o `--release` vale `20×` o dev»* — **falso NESTA crate**: a `ph2d-tool-painter` está na lista
   `[profile.dev.package.*]` `opt-level = 2`, e as duas colunas batem (`seco` `1,73`/`1,75`).
4. ⛔ *«um pico na escada do raio é dívida desta wave»* — **falso**, e o CONTROLO data-o: a `r = 88` e
   `96` a divergência incremental↔cheio lê `94`/`93` **a seco** e `94`/`90` **com a LEI ANTIGA**.

---

## §9 — Sete leituras que o diff inverte

1. **O `stroke_deplete` mudou de SIGNIFICADO** (§4) — quem o ler cru lê outra grandeza.
2. **A `RESERVE_RIM_RAMP = 0.15` não é um número novo**: é o mesmo de antes, movido de dentro do `max`
   para um factor separado. *O diff parece introduzir uma constante e está a mudar o operador.*
3. **O `LEI_ANTIGA` não é código morto** — é o CONTROLO de quatro gates, e apagá-lo cega-os.
4. **O gate de TEXTO (`a_janela_do_composite_reserva_o_raio_do_campo`) não é preguiça**: está medido
   que a propriedade não é alcançável por pixel (§8.2).
5. **O `diag_a_escada_do_raio_da_janela` é `#[ignore]` e vale mais que um gate**: ele carrega a tabela
   que DATA um defeito pré-existente. Apagá-lo perde a medição, não um teste.
6. **A `paper: false` do arnês não é conveniência**: o dente do papel move `~5` bytes/px e afogava a
   régua exactamente onde o contraste é menor — na costura, que é o lado que a barra julga.
7. **A linha não grava nada** — o mapa é de traço vivo. Nenhum degrau de schema é devido.

---

## §10 — O que smokar (o dono) — ✅ **APROVADO em 2026-09-20**

> ✅ **Smoke do dono APROVADO** (2026-09-20): *«Smoke OK.»* — sobre o gesto do report original (descer,
> virar e voltar por cima do próprio traço sem levantar a caneta, com `Charge < 1`).
> ⛔ **Aprovar o smoke NÃO é autorizar a integração** (§0.7): a linha fica fechada e PARADA até ordem
> explícita do dono.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke
```

Não há variável de ambiente: é o caminho normal do **Paint Mode: Watercolor** com **Charge < 1**.
O gesto do report — descer, virar e **voltar por cima do próprio traço sem levantar a caneta**.

Medido (largura efectiva da costura, px — a régua está em `tests/watercolor_selfseam.rs`):

| `r` | antes | depois | com Rewet 1 |
|---|---|---|---|
| 20 | `5,40` | `6,76` | `10,00` |
| 32 | `5,86` | `9,32` | `12,00` |
| 45 | `6,52` | `11,30` | `15,26` |
| 96 | `10,22` | `22,00` | `30,00` |

E o Smudge, que **antes era byte-idêntico a Smudge 0** sobre esta costura: `10,43 → 33,13 px`.

⚠️ A barra sai do **lado aprovado**: o oráculo é a `libmypaint 1.6.1` (**ISC** — porta aberta, sem
clean-room), corrida sem interface sobre um traço em U NOSSO; nela a costura interna tem a **mesma**
largura da borda externa do traço (razão `~1,00` em quatro durezas).

---

## §11 — Aberto, e de quem é

- ⏳ **Item 4 do report** (o pincel esgotado voltar a captar a própria tinta molhada) — **decisão do
  dono**, não implementado. O custo está avaliado no doc 40.
- ⏳ **Promoção pedida à família de flakes de fan-out (CLAUDE.md §5.0)** — a linha pede, o integrador
  escreve: `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` (`ph2d-tool-painter`,
  `tool::paint::tests::measure_input_cost`). Assinatura completa: único ✗ de `1 257` numa corrida da
  suíte da crate · **zero** linhas do diff desta linha naquele caminho · **3 de 3 verde sozinho a
  `load 45,37`**, ou seja MAIS carga do que aquela em que reprovou.
- ⏳ **Defeito PRÉ-EXISTENTE, datado com controlo** (não é desta wave): a `r = 88`/`96` o composite
  incremental diverge do cheio `~94` níveis **a seco e com a lei antiga**. Parente dos dois
  `watercolor_app_params_incremental_*` que seguem `#[ignore]`.
- ⏳ Sob **Tiling**, o arrasto dos níveis levanta toroidal como o da base; não tem gate próprio.

---

## §12 — O ITEM 4, construído por ordem do dono (2026-09-20)

> *«implemente Item 4 como opção extra e não como substituto»*. Knob **`Self Pickup`** (`0..1`), que
> **nasce em `0`** ⇒ o caminho de fábrica é byte-idêntico, com gate a afirmá-lo. Análise prévia:
> [doc 40 §S2-C e §9](../40_a_costura_dura_do_retorno_sobre_o_proprio_traco.md).

### §12.1 — A cerca, e porque ela é a feature

O risco não é a recolha não funcionar, é funcionar **de mais**: com espaçamento de fábrica o dab
`i−1` cobre `~95 %` do disco do dab `i`, logo ler o próprio rasto é **self-feeding literal** e o
Charge deixa de gastar. A cerca é a **IDADE** — plano novo `stroke_arc` com o **arco da PRIMEIRA
cobertura** de cada texel.

⛔ **PRIMEIRA e não última:** um plano de «última cobertura» seria sobrescrito pela cabeça da
própria volta e leria idade `≈ 0` em quase tudo — o travão nunca armaria.

⭐ **O limiar é DERIVADO:** num traço recto o tap mais atrasado fica a `r/2` do centro e foi coberto
quando o centro estava a `1,5 r` ⇒ **`0,75` diâmetros**. `PICKUP_AGE_DIAMETERS = 2,5` deixa `3,3×`
de margem; a perna de ida de um U de raio `32` é cruzada com `~5,8` diâmetros. Daí o par de fixturas
que discrimina sozinho: **recto ⇒ AO BIT igual**, **U ⇒ diverge**.

### §12.2 — Três correcções que a MEDIÇÃO impôs ao desenho prévio

1. ⛔ **A recolha não pode entrar no reservatório de COR.** Em papel virgem com `Charge < 1` o passe
   de cor toma o caminho *«pula»* e o `stroke_color` **nunca é escrito** (doc 40 §8) ⇒ o reservatório
   lia BRANCO e a volta sairia mais **clara** — o oposto do item 4. A reserva viaja em canal próprio.
2. ⛔ **Ela não pode viver no AVANÇO** (passe de cor). Instrumentado, o avanço media `live_pig =
   0,58` na volta e **a tela não mexia**: quem escreve o mapa de pigmento que o composite lê
   (`stroke_deplete`) é o passe de **COBERTURA**, e ele corre **primeiro**. ⭐ E pô-la no replay
   daquele passe torna-a **independente do corte dos lotes de graça**.
3. ⛔⛔ **Escrita com o idioma `fresco ∨ carry` da casa, ela é um DEGRAU.** `base.max(live·gain)` só
   morde quando `live·gain` passa o fresco, e abaixo disso morre ao primeiro dab — porque esse dab
   dilui o nível que o seguinte vai amostrar. **Medido** (nível da reserva na volta): `67 → 69 →
   137` para knob `0 / 0,5 / 1`. Interpolando (`base + gain·(live − base)⁺`): **`67 → 98 → 137`**,
   com os dois extremos intactos.

### §12.3 — Medido

| grandeza | número |
|---|---|
| G na perna de VOLTA, knob `0 → 1` | `230 → 206` no corpo, **`198 → 166`** no flanco |
| a perna de IDA | **intocada** (`193`/`191` nas duas) |
| nível da reserva na volta, knob `0 / 0,5 / 1` | `67 / 98 / 137` |
| custo (razão knob-1 / knob-0, `92 %` CPU ociosa) | **`1,004`** (r = 32) · **`0,998`** (r = 96) |
| memória do plano | `2 B/texel`, **só alocado com o knob ligado** |
| prova de mutação | **`6` de `6` sangram**, controlo do arnês nas duas pontas |

⚠️ **A §9.5 do doc 40 não se materializou:** a reordenação «alto risco de construção» foi
desnecessária — a cerca de idade já dá a independência de lotes.

⛔⛔ **E uma mutação SOBREVIVEU antes de a régua existir**, com uma lição própria: o knob é guardado
em **três** sítios (a alocação do plano, o atalho `gain > 0`, o multiplicador) e cada um sozinho já
entrega o comportamento certo a `0` ⇒ mutar um era **neutralizado** pelos outros e lia-se como
sobrevivência. *Três guardas correctas tornam-se, juntas, uma lei que nenhuma mutação de um sítio
consegue matar.* A saída não foi apagar guardas — as três ganham o lugar (`33,6 MB`, o laço de taps
por dab, e a lei) — foi **medir a lei onde ela é CONTÍNUA** (`the_knob_is_a_dial_not_a_switch`).

### §12.4 — Superfície

`PROJECT_SCHEMA` **0** (o `BrushSpec` não deriva `Serialize`) · contratos congelados **0** ·
`shells/desktop` **0** · pacotes externos **0**. O knob percorre os **sete** sítios que um irmão
(`wet_pull`) ocupa: `BrushSpec` + default · `BrushSettings` + o `snapshot` que a popula · o `reset` ·
o id · o setter + o braço do despacho · a fileira do painel · a chave de i18n.
⚠️ **O `BrushSettings` é um espelho separado do `BrushSpec`** e um censo do irmão por nome não os
separa — foi o clippy que o apanhou.

### §12.5 — O que smokar

Paint Mode **Watercolor**, **Charge** a meio, **Self Pickup** no máximo, e o MESMO gesto do report:
descer, virar e voltar por cima do próprio traço **sem levantar**. A volta tem de sair **mais
carregada** onde cruza a ida, e um traço **recto** tem de ficar exactamente como estava.


---

## §13 — As vassouras, re-corridas sobre a linha INTEIRA (incl. o item 4)

`5` de `10` acusam; **nenhum achado é código novo desta linha**. A atribuição, por duas provas:

* **Nenhuma linha acusada é uma adição de código minha.** As acusadas vivem em declarações e
  comentários que o merge-base já tinha (`spec.rs`, `spec_default.rs`, `brush_fallback.rs`,
  `stroke_lifecycle.rs`, `watercolor_accum.rs`), e **três** delas são texto do `CLAUDE.md` §5
  escrito por *outra* linha.
* **A única adição que casa é a LEITURA de um campo público NOSSO** (o *método de traço* do
  pincel — ⚠️ o nome exacto dele fica FORA desta frase de propósito, senão o documento passa a ser
  mais um acerto da vassoura: é a mesma cura que o parágrafo abaixo descreve), cuja
  **declaração** — essa sim, pré-existente — é que traz a citação do alvo no doc-comment. Usar a
  nossa própria API não é uma citação nova; a dívida é da declaração, e a triagem dela é do **R**.

⛔⛔ **E uma ocorrência era MINHA, neste documento — curada.** A redacção anterior do §5-bis
**citava os dois tokens** para explicar que eram pré-existentes, e as vassouras passaram a acusar a
explicação: os achados caíram de `24 / 8 / 12` para `19 / 4 / 8` em três delas só por deixar de os
escrever. ⚠️ *Um instrumento que procura uma agulha não pode CONTÊ-LA* — é a razão por que a
própria vassoura guarda as entradas em base64, e vale igual para o documento que a comenta.

⚠️ Esta linha **não tem parede**: o oráculo dela é a `libmypaint` (**ISC**, porta aberta pelo §0.9),
e nada aqui foi escrito a partir de fonte restrito.

---

## §14 — O SMOKE do item 4 REPROVOU a fileira, e eram TRÊS defeitos numa edição de quatro linhas

> Report do dono (2026-09-20, com foto): *«label embolada e valor não sai de 0»*.

A LEI do item 4 estava certa — os quatro gates e as seis mutações do §12 medem-na, e nenhum deles
toca no painel. O que reprovou foi a **fileira**, e ela tinha três defeitos independentes:

| # | defeito | o que o dono vê |
|---|---|---|
| 1 | `card_frame(…, 3)` com **4** linhas dentro | a 4.ª linha é desenhada **FORA** da moldura |
| 2 | a linha do `Pull` fazia `let _ = card_row(…)` | `Pull` e `Self Pickup` no **MESMO `y`** — a «label embolada» |
| 3 | o id nunca entrou em `PAINTER_WATERCOLOR_FIELDS` | o `ValueChanged` **morre no painel** ⇒ *«o valor não sai de 0»* |

⭐ **O (2) é a armadilha do idioma:** o `y` de retorno de uma `card_row` é o `y` da linha SEGUINTE, e
deitá-lo fora (`let _ =`) só é honesto na **ÚLTIMA**. O `Pull` era a última quando foi escrito, e
ganhar uma vizinha transformou uma linha correcta num defeito **sem que ela fosse tocada**.

⛔⛔ **O (3) tinha um gate a apontar-lhe e ele estava VERDE por construção:** o
`seam::watercolor_sliders_forward_setvalue` itera o próprio `PAINTER_WATERCOLOR_FIELDS` — *um censo
que varre a lista que esqueceu o membro não pode ver o membro que falta*. ⇒ o instrumento novo é
**DERIVADO DA TELA**: pinta o painel no meio `Watercolor` e no `Digital`, e a população é a
**diferença** — *o que a secção ACRESCENTA ao ecrã*, que ninguém tem de se lembrar de estender.

⚠️ **E o (1) não move um único retângulo de hit**, logo nenhum gate de costura o alcança: o censo
dele lê o FONTE dos dois ficheiros que declaram cartões e compara o `n_rows` com as chamadas
`…_row(` do mesmo corpo. ⚠️ A classificação é **derivada** — um cartão que DELEGA as linhas a um
irmão (`…_rows(`, plural) é detectado pela CHAMADA e saltado, com **piso nos dois lados** para o
classificador não colapsar. ⚠️⚠️ E a 1.ª redacção dele acusou **três cartões correctos**, porque
contava só `card_row(`: *uma `card_row` não é a única espécie de linha de um cartão* (há botões,
caixas e dropdowns, cada um a ocupar uma fileira).

⚠️ **O gesto do gate é a EDIÇÃO do número, nunca um arrasto** — medido nos irmãos que shipam há
meses (`Dilution`, `Pull`): um `drag_at` sobre um chip deste painel produz cinco eventos e autora
**ZERO**, porque um chip é um `NumberInput` e não um `Slider`. *Escolher o gesto errado daria um
gate vermelho sobre produto certo nas três linhas, o que se lê como defeito de lei.*

**Prova de mutação: 5 de 5 sangram**, cada uma no gate que lhe corresponde, com o arnês a **contar
os testes que de facto correram** (`4`) e controlo verde nas duas pontas:

| mutação | sangra em |
|---|---|
| o cartão volta a declarar `3` | o censo dos cartões |
| o `Pull` volta a `let _ =` | a sobreposição **e** a linha própria |
| o id sai de `…_FIELDS` | o censo derivado **e** a edição que chega |
| o `is_param_field` larga a família | os dois acima |
| **CONTROLO:** tudo lido como delegado | o censo (piso de população) |

**Superfície:** `PROJECT_SCHEMA` **0** · contratos **0** · shell **0** · pacotes **0**. Três
ficheiros de produto (`paint_watercolor.rs`, `ids/painter_watercolor.rs`) e um de teste novo
(`tests/it/seam_watercolor_cards.rs`). Portão: `nextest-impacted` **18 067/18 067**, clippy
`-D warnings` a zero.

---

## §15 — O ITEM 4 FOI RETIRADO, por veredito do dono, no dia em que shipou

> *«feature com resultado ruim, não consegue distribuir corretamente a carga da tinta. retire e
> limpe o código de Self Pickup»* (2026-09-20, depois do smoke da fileira curada na §14).

O `Self Pickup` **não existe**: o controlo, a lei, o plano do arco e o campo do traço foram
apagados. ⇒ **as §12 e §14 descrevem código que já não está na árvore**, e ficam aqui como a
medição da recusa.

### §15.1 — O que saiu, e o que a ausência PROTEGE

| camada | o que saiu |
|---|---|
| contrato do pincel | `BrushSpec::wet_self_pickup` + o default · o espelho `BrushSettings` · o `snapshot` |
| painel | a fileira do cartão BRUSH (`4` → `3` linhas) · o `brush_fallback` · a chave de i18n |
| despacho | o id `PAINTER_WATERCOLOR_SELF_PICKUP` · a entrada em `…_FIELDS` (`28` → `27`) · o setter · o braço · a linha do reset |
| motor | `ARC_UNIT_PX` · `PICKUP_AGE_DIAMETERS` · `arc_stamp` · `LivePickup` · `live_reserve` · o termo `gain` da `wet_mix_depletion` |
| estado | `PaintState::stroke_arc` e os **quatro** sítios do ciclo de vida dele |
| gates | `tests/watercolor_pickup.rs` (4 gates + 2 sondas) · o `SeamKnobs::pickup` · o gate da fileira |

⭐ **A `wet_mix_depletion` volta a devolver `Option<Vec<f32>>`**: o segundo elemento do par (o
percurso) tinha **um** consumidor, o carimbo do arco — *quando o único leitor de um valor sai, o
valor sai com ele, senão fica um campo que ninguém lê e que a próxima leitura interpreta*.

⚠️ **E a EMENDA ao `wet_charge` foi RETRACTADA.** Ela tinha tornado condicional a frase *«the pickup
reads the FROZEN pre-stroke base, so it can't self-feed»*; com o controlo fora, a frase é
incondicional outra vez, e o doc di-lo com a data das duas mudanças. *Uma emenda que sobrevive à
feature que a motivou é uma nota que mente ao contrário.*

### §15.2 — O que NÃO saiu, e porquê

Os **três** gates de painel da §14 **ficam** — nenhum deles é sobre o Self Pickup:

* `cada_campo_que_a_aquarela_acrescenta_a_tela_chega_a_ferramenta` (o censo derivado da TELA),
* `nenhuma_linha_dos_cartoes_e_pintada_por_cima_de_outra`,
* `o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta`.

⭐ Eles nasceram de um defeito desta feature e medem uma propriedade do **painel**: *a fileira que
os motivou foi-se, e a armadilha que eles apanham fica* — e o `let _ = card_row(…)` do `Pull`, que
voltou a ser a última linha, leva agora escrito por cima o aviso de que quem lhe puser uma vizinha
tem de o trocar por `ry =` **e** subir o `n_rows`.

### §15.3 — O instrumento que esta wave teve de curar

⛔⛔ **O censo dos cartões NÃO saltava comentários, e a primeira prosa com uma VÍRGULA partiu-o ao
meio:** o scanner de parênteses leu essa vírgula como separador de argumento e o `n_rows` passou a
ler-se *«e nada no desenho3»* ⇒ ele acusou um cartão **CORRECTO** de não declarar um literal.
⚠️ A mesma cegueira inflaria a CONTAGEM (um `…_row(` citado num comentário conta como uma linha
pintada). ⇒ `sem_comentarios`, com reconhecimento de string, corrido **antes** de qualquer
contagem. *Uma régua que lê código tem de decidir o que é código antes de contar seja o que for.*

**Prova de mutação: 5 de 5 sangram** (o cartão a declarar a mais · a `card_row` a não avançar o `y`
⇒ TODAS as linhas sobrepostas · o `is_param_field` a largar a família · **CONTROLO** do
classificador de delegação · e o `sem_comentarios` inerte, que é a prova da cura acima), com o
arnês a contar os testes que de facto correram (`3`). ⚠️ **Uma delas casou a âncora `3×` e foi
ABORTADA** antes de medir seja o que for — *outros cartões também declaram `3`, e uma mutação que
casa N vezes não é a mutação que se quis medir*.

### §15.4 — Portão, e as vassouras

`nextest-impacted` **18 062/18 062** · `CARGO_BUILD_WARNINGS=deny cargo check --workspace
--all-targets` verde · clippy `--workspace --all-targets -D warnings` zero · `cargo machete` limpo ·
standalone-optional e workflow-packages verdes · censos da árvore COMBINADA **127/127** · `fmt`
limpo.

⚠️ **As vassouras foram re-atribuídas FICHEIRO A FICHEIRO contra o merge-base** (o total agregado
não serve: `11` dos `36` ficheiros do diff não existiam lá, e comparar `34` com `40` é comparar
populações diferentes). Resultado: **`24` ficheiros iguais ou melhores**, e três a olhar:

* `watercolor_smudge.rs` `0 → 2` — a **leitura** de um campo público NOSSO, cuja declaração é que
  traz a citação e é **pré-existente** (já registado no §13; a triagem é do **R**).
* `docs/Painter/ferramentas/oraculo_costura/costura.c` `— → 2` — a chamada que entrega um ponto do
  traço ao pincel, **API pública da `libmypaint` (ISC, a porta aberta do §0.9)**: a vassoura casa um
  pedaço genérico do nome dela. Isenção **NOMEADA**; a triagem é do **R**. ⚠️ O nome exacto da
  função fica FORA desta frase pela mesma razão do ponto abaixo.
* o próprio handoff — **era MINHA, TRÊS vezes, e está curada**: o §13 escrevia o nome do campo
  para explicar que ele é pré-existente, esta §15 voltou a escrevê-lo, e a §15.4 chegou a citar o
  nome da função ISC ao explicar que ela é legítima. *Um instrumento que procura uma agulha não
  pode CONTÊ-LA — e a prosa que EXPLICA um achado é o sítio mais provável para ela reaparecer,
  porque explicar dá vontade de nomear.*

⚠️ **E o arnês das vassouras tinha um defeito que lia como achado:** ele alimentava a lista de
caminhos do `git diff` **incluindo os APAGADOS**, e um caminho inexistente faz a vassoura sair
`rc = 2` — as dez acusaram de uma vez. Curado com `--diff-filter=d`.

## §16 — A COR do dono era o ÚLTIMO param por-dono lido DISCRETO (report do dono, 2026-09-20)

> *"trocar a cor do pincel também criou pixelamento da borda"* — foto com um traço AZUL e um VERDE
> lado a lado, a fronteira entre eles serrilhada, e as setas do painel em **Charge 0,326 · Rewet
> 0,272 · Smudge 0,468**.

### §16.1 — A causa, e porque ela sobreviveu dois meses

O **#18** (doc 13, 2026-07-11) suavizou os params por-dono na fronteira de posse: `fill` · `depth` ·
`edge_gain` · `opacity` · `warp`, mais o `wet` que já tinha campo próprio. **A COR ficou de fora** —
o `st.color` continua a sair do mapa de posse por `style_at`, que é **NEAREST**. Enquanto os dois
traços de uma sessão molhada têm a mesma cor o degrau é invisível; trocar a cor imprime-o.

⚠️⚠️ **E o que o torna VISÍVEL é exactamente o que as setas da foto apontam.** Com o mixer armado
(`wet_charge < 1`) **sobre papel virgem** o `sample_surface` não tem o que captar, logo
`prio = pickup × load` é **ZERO**, `accumulate_wet_color` escreve alfa `0` em cada texel, e a cor
passa a vir **INTEIRA** do dono. Com o mixer desligado o depósito escreve uma rampa `source-over`
que **MASCARA** o degrau. *O defeito esteve lá desde o #18; foi preciso um knob para o revelar.*

### §16.2 — A régua, e porque a óbvia mente

Um `max |Δ|` cru por pixel lê a **borda sobre papel** — um degrau legítimo contra o branco — como
sempre pior que a junção (`56`–`67` contra `54`), e teria ilibado o defeito. O que separa um degrau
duro de uma rampa é a **fracção do salto TOTAL de cor que cabe num pixel**: `1,00` = um pixel,
`~0,4` = três a quatro. E a **barra sai da MESMA imagem** — a junção não pode ser mais dura que a
borda de silhueta que a AA já entrega ali ao lado; não é um número escolhido.

| célula (fixtura do report) | antes | depois | barra (borda sobre papel) |
|---|---|---|---|
| junção, mixer **ON**, cor trocada | **0,99** | **0,24** | 0,58 |
| junção, mixer **OFF**, cor trocada | 0,40 | 0,43 | 0,54 |
| junção, cor **IGUAL** (controlo) | — | — | o swing total cai `5,7×` |

A tabela de atribuição que separou as quatro causas possíveis (medida antes de uma linha de cura):

| | junção (salto abs.) | alfa do depósito |
|---|---|---|
| mixer ON + cor trocada | **79** | **0** |
| mixer ON + cor igual | 14 | 0 |
| mixer OFF + cor trocada | 38 | 255 |
| mixer OFF + cor igual | 19 | 255 |

### §16.3 — A cura

A cor entra no campo suavizado por-dono (`StyleField::sample_color`, `blur(c·m)/blur(m)`, o mesmo
`WET_FIELD_BLUR_PX` dos outros cinco), e **a ponta do lerp do depósito parte dela e nunca da
discreta** — é o lado que o depósito não cobre, e era ele que degrauava.

⚠️ **`params_differ` passa a incluir a cor, e sem isso a cura é INALCANÇÁVEL:** trocar SÓ a cor
deixava o predicado `false` ⇒ campo nenhum ⇒ o caminho discreto, que é o defeito. *Um param novo que
não seja acrescentado ali nasce com a cura morta.*

⛔ **RECUSA MEDIDA — um raio PRÓPRIO para a cor foi construído e RETIRADO.** A justificação que eu
lhe tinha escrito (*«a `8` a junção de dois traços de 24 px fica com 16 px de degradê e as duas
cores deixam de ser duas»*) é **FALSA**: varridos `2 · 4 · 8` sobre a fixtura do report, os miolos
dos dois traços saem **byte-idênticos nos três** e a junção mede `0,21 · 0,21 · 0,26`. E um raio
próprio pedia uma **segunda MASSA** (normalizar uma cor borrada a `r` pela máscara de `8` clareia a
orla, onde as duas discordam) — dois campos e uma constante a mais para comprar `0,05`, quando o
`#18` já escolhera aquele raio contra o vão do guarda de não-contacto, que a cor herda de graça.

⚠️ **Mudança declarada no lado que NÃO foi reportado:** com o mixer desligado a junção passa de
`0,40` para `0,43`, porque uma troca de cor passa a construir o campo (antes `params_differ` era
`false` e não havia nenhum). Os outros cinco params são iguais entre os donos, logo suavizá-los ali
é um no-op; o que se move é a ponta do lerp.

### §16.4 — O tecto de LOC, curado por CORTE

O `watercolor_render.rs` passou a `703` (tecto `700`). Cortado **por assunto, nunca por isenção**:
*«que cor tem o pigmento neste pixel»* (dono suavizado → depósito → água: dissolve e backrun) vira o
irmão [`watercolor_render/pigment.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render/pigment.rs),
ao lado do `diag.rs` e do `window.rs` que o mesmo tecto já produzira. O pai fica em **667**, e ali o
laço volta a responder só *quanta densidade há aqui*.

### §16.5 — A prova de mutação, e o sobrevivente que foi ATRIBUÍDO

**4 de 4 sangram**, corridas contra a **suíte INTEIRA da crate** de propósito — para que um
sobrevivente do gate novo seja *atribuído a outro gate* em vez de lido como buraco:

| mutação | veredito |
|---|---|
| M1 a cura desligada (`sample_color` devolve sempre a discreta) | **sangra** no gate novo |
| M2 `params_differ` larga a cor | **sangra** no gate novo |
| M3 `sample_color` lê sempre o plano `0` | **sangra** em **3** (o novo + dois de sessão) |
| M4 a ponta do lerp troca dono↔depósito | **sangra em QUATRO gates do Wet Mix que já existiam** |

⚠️ **O M4 NÃO sangra no gate novo, e a razão é a fixtura:** com o mixer armado sobre papel virgem
`ca8 == 0`, logo o ramo do lerp **nunca corre** ali. A propriedade está gateada — pelos
`watercolor_wet_mix_*`, que é onde ela vive (a brocha a cruzar tinta que existe). *Um sobrevivente
só é um buraco depois de se perguntar quem mais o mede.*

### §16.6 — O gate

`watercolor_color_change_junction_is_soft`, irmão do `watercolor_param_change_junction_is_soft` do
#18, com **três** metades:

1. **a fixtura CONTÉM o fenómeno** — `alfa do depósito == 0`. Se alguém mudar o mixer para escrever
   ali, o gate diz que a fixtura deixou de conter o fenómeno em vez de ficar verde em silêncio;
2. **há uma troca de cor para medir** — o swing da junção colorida é `≥ 3×` o da de cor igual, senão
   a fracção é a razão de dois ruídos;
3. **a cura** — `fracção(junção) ≤ fracção(borda sobre papel)`, na mesma imagem.

### §16.7 — Aberto

⏳ A junção com o mixer **desligado** mede `0,43` contra `0,54` da borda — dentro da barra, e é o
lado que o dono nunca reportou. Se algum dia ele o apontar, a alavanca é a rampa do próprio
depósito (`accumulate_wet_color`, `source-over` com o feather do dab), não este campo.

## §17 — A LEI DA TINTA vira folha; o Digital troca, a Aquarela NÃO (ordem do dono, 2026-09-20)

> *«Em watercolor pigment só funciona se pintar sobre tinta seca. Sobre tinta molhada fica como na
> imagem.»* (foto: azul e amarelo encostados, faixa PÁLIDA no meio) · *«Prefiro: Trocar as duas para
> a lei do Wet Paint (os três meios passam a misturar igual).»*

### §17.1 — O que a medição respondeu ANTES de construir

A pergunta que veio antes (*«podia ser implementado para digital e watercolor?»*) tinha uma resposta
que só a medição dá: **já estava implementado nos dois** — é o slider `Pigment`, `OFF` de fábrica —
e **não é a lei do Wet Paint**: era **RYB** (Gossett & Chen 2004) contra o **Kubelka–Munk** do
fluido. Uma nota do `spec.rs` dizia «Kubelka–Munk» sobre a implementação RYB, e foi corrigida
(`aa331f9e9`) — *uma nota que troca o nome do modelo faz alguém comparar duas tintas diferentes
julgando que são a mesma*.

**Custo, medido** (release, 94–97 % de CPU ociosa, canvas 4096², quadro de 16,7 ms): na aquarela o
`Pigment` custa **abaixo do ruído** (`0,897 → 0,891 ms`); no digital **`2,1×`** o traço (`+0,20` a
`+0,36 ms` para raios de 50–200 px, `1,2 %`–`2,2 %` de um quadro). A lei em si: RYB `18,3 ns`,
K–M `30,3 ns` (`1,65×`). ⚠️ A linha de `raio 400` da varredura leu `0,004 ms` e foi **descartada**:
o passo fixo de 40 px não emite dab para aquele raio — *aquilo é nada pintado, não um caso barato*.

### §17.2 — W1: a lei muda de ENDEREÇO (entregue)

[`ph2d-pigment`](../../../crates/ph2d-pigment/) — folha nova, `libm` pinado e mais nada. O K–M sai
do `ph2d-wet-paint` **sem uma linha de matemática mudada** e aquele motor **delega** (suíte
`88 + 37 + 6` verde, byte-idêntica). ⚠️ *Uma lei escrita em dois sítios ainda não é uma lei.*

### §17.3 — W2: o Digital troca (entregue) · a Aquarela NÃO (medido e revertido)

O `blend_over_pigment` lê a porta. A troca é **observável**: a 50/50, `amarelo + vermelho` lia
`141,84,28` e lê `227,38,28`.

⛔⛔ **Na aquarela a troca foi CONSTRUÍDA, MEDIDA e REVERTIDA, e o que ela quebra não é a cor — é a
FAIXA DINÂMICA.** Ali o parceiro da mistura é a **BASE** (papel quase branco, `K/S ≈ 0`), e um lerp
em `K/S` contra o zero é violentamente não-linear: um pigmento saturado escurece até ao chão. Medido
pelo produto, o `watercolor_soak_deepens_and_widens_the_dissolve_while_parked` passou a ler **o
MESMO pixel** (`228,23,23`) para 2 s de demora e para a passagem rápida — *os dois lados saturam e o
knob deixa de modular*. ⭐ **A pista da 2.ª tentativa está nomeada no sítio:** para *«uma camada de
pigmento SOBRE um fundo»* o operador K–M não é o `mix`, é o **glaze**
(`km_glaze_channel_linear`, que já existe e é energia-limitado).

### §17.4 — O defeito da FOTO: diagnosticado, com duas curas medidas e mortas

⛔⛔ **A causa não é a lei — é COM O QUÊ se mistura.** O termo `Pigment` mistura a lavagem com a
base **CONGELADA**, e um vizinho molhado é *session-mate*: **não está lá**. Ele mistura com o PAPEL.
Medido no meio da sobreposição: `254,252,235` (quase branco) com o botão LIGADO contra `253,245,140`
desligado — *o botão não falha wet-on-wet; ele mistura com a coisa errada*.

**Duas curas construídas e revertidas:**
1. **trocar a LEI no depósito** — irrelevante: no meio o peso do `over` é `w = a/na ≈ 1` e **qualquer**
   lei devolve a cor de cima;
2. **trocar o PESO** para a fracção de massa (`a/(a+da)`) — o depósito vai de `250,230,64` para
   `210,223,65` e pára aí: *~20 dabs amarelos passam pelo mesmo texel e cada um volta a misturar*,
   lavando o azul embora geometricamente.

⭐ **A cura que sobra é estrutural e está desenhada:** a cobertura é **max-blended** porque uma
lavagem é **UMA passagem**, e a cor tem de obedecer à mesma lei — o peso do depósito não é o alfa do
dab, é o **INCREMENTO da cobertura** (`max(0, depois − antes)`), zero quando o mesmo traço repassa e
positivo quando um traço NOVO chega. O incremento só é visível dentro do passe de COBERTURA (o de
cor corre depois, sobre o envelope já fechado) ⇒ **a cura junta os dois passes**, o que de graça
apaga a duplicação do replay de rng que eles hoje mantêm em lock-step. O instrumento fica versionado
em `diag_pigment_molhado_sobre_molhado` (`#[ignore]`, com a tabela das quatro células).

### §17.5 — Premissas mortas e armadilhas pagas

- `pigment_mixes_blue_and_yellow_toward_green` exigia *«o verde ACIMA da média plana»* — propriedade
  do **RYB**. No K–M as absorvências SOMAM e a mistura é mais **ESCURA** (`0,275` contra `0,500`).
  Fica o hallmark das duas leis (o verde DOMINAR) + a metade que **afirma a troca** + o controlo.
- O cabeçalho do `ph2d-painter-brush` dizia *«zero dependências de propósito — nenhuma transferência
  sRGB é precisa aqui»*: a lei da tinta **é** uma.
- ⛔ A 1.ª `mix_unit` passava por **BYTES** e quantizava a saída a `1/255`, apagando a diferença que
  o gate do soak mede — a porta do `K/S` é uma **tabela interpolada** e aceita `0..255` fraccionário.
- ⛔ **Mover um ficheiro troca o REGIME DE LINT que o governa** (§5.0, outra camada): o `transfer.rs`
  vivia sob dois `allow` de **crate inteira** da casa de origem. Um virou `clamp` (mesma resposta,
  `NaN` incluído) e o outro é **isenção nomeada de um método só** — *uma folha nova não herda uma
  licença em branco porque o ficheiro veio de uma casa que a tinha*.

### §17.6 — Aberto, e de quem é

- ⏳ **O controlo do `Pigment` no DIGITAL** — a lei já é a do Wet Paint ali, e o portão
  (`effective_pigment_mix` exige `watercolor && pigment`) mantém-na **inalcançável**: a fileira vive
  no cartão da AQUARELA. Abrir o portão sem pintar a fileira entregaria uma feature que o artista
  não alcança, que é o defeito que esta casa já nomeia. ⚠️ **E há uma razão de PRODUTO para a ordem
  ser esta:** a troca escurece as misturas, e o dono deve julgá-la num meio antes de ela alcançar um
  terceiro.
- ⏳ **O wet-on-wet** (§17.4) — cura desenhada, junta os dois passes do depósito.
- ⏳ **A aquarela com o glaze** (§17.3) — a 2.ª tentativa da troca de lei.

---

## §18 — «LIGUE O DIGITAL»: a cerca do MEIO sai, e a fileira ganha uma porta (ordem do dono, 2026-09-20)

> *«smoke OK. LIgue o digital»* — depois de ele aprovar o smoke da §16/§17.

**O que muda:** `BrushSpec::effective_pigment_mix` deixa de gatear em `watercolor && pigment` e
passa a gatear em `pigment` sozinho. A pergunta que aquele predicado responde é ***«como é que a cor
deste dab encontra a tinta que já lá está?»***, e ela é de **todo meio que componha um dab** pela
porta `blend_over_pigment` — amarrá-la à aguada tornava a lei **inalcançável no Digital**, que é o
meio de omissão do app.

### §18.1 — O ALCANCE foi MEDIDO antes de a cerca sair

⛔⛔ *Alargar uma LEI cria imediatamente duas maneiras de ela e o BOTÃO discordarem, e as curas são
OPOSTAS* — o meio que lê a lei e não vê a fileira tem um knob **INALCANÇÁVEL** (cura: pintá-la), o
que a vê e não a lê tem um knob **MORTO** (cura: escondê-la). *As duas leem-se igual numa tabela.*

A sonda [`diag_pigmento_por_meio`](../../../crates/ph2d-tool-painter/src/tool/paint/diag_pigmento_por_meio.rs)
pinta **azul sobre amarelo** (complementares: a lei aditiva dá cinzento, a subtractiva dá escuro e
saturado) e compara o pixel da sobreposição com o knob a `0` e a `1`:

| meio | sem | com | `|Δ|max` |
|---|---|---|---|
| **Digital** | `197,203,203` | `55,111,202` | **142** |
| **Watercolor** | `178,198,247` | `182,194,243` | **4** |
| **Impasto** | `197,203,203` | `55,111,202` | **142** |
| **WetPaint** | `42,76,203` | `42,76,203` | **0** |

⇒ a porta é **`PaintMedia::offers_pigment_mixing`** — Digital · Watercolor · Impasto.
⛔ **O Wet Paint fica de fora com a medição ao lado, e não por esquecimento:** o depósito dele é do
solver de fluido, que **tem o Kubelka–Munk próprio** (`ph2d_wet_paint::ColorMix::Km`) e o slider de
pigmento por dab dele — oferecer esta fileira ali seria um segundo controlo sobre a mesma pergunta,
e ele seria **inerte**.

⚠️⚠️ **A 1.ª redacção da sonda leu `0` nas QUATRO linhas** e teria concluído que a lei não chega a
lado nenhum: ela corria com cobertura CHEIA, e com `a = 1` o `blend_over_pigment` devolve a cor de
cima *seja qual for a lei*. **A meia força é o que faz a fixtura conter o fenómeno** — *a mistura só
é observável onde há o que misturar*.

### §18.2 — A fileira: uma lei, uma porta, dois hospedeiros

A fileira `Pigment` deixa de ser propriedade de uma secção. O valor que ela mostra **é DERIVADO**
(`pigment ? pigment_mix : 0` — o par *toggle + amount* que a redesign de 2026-07-07 fundiu), e
*duas derivações do mesmo número divergem no dia em que alguém afina uma delas* ⇒
[`paint_pigment.rs`](../../../crates/ph2d-panel-painter-layers/src/paint_pigment.rs), com:

- o cartão **Water** da aquarela (onde o dono a aprendeu, ao lado do Rewet e do Smudge) e
- o cartão **`Mixing`** novo, no lugar da secção do meio, para o **Digital** e o **Impasto**.

⚠️ **A aguada é subtraída explicitamente** em `paint_brush_sections.rs`: sem isso os dois cartões
pintam o MESMO id no mesmo quadro — e um id repetido não é um controlo a mais, é um **hit-rect a
tapar o outro** (o `HitIndex::hit` resolve de trás para a frente).

⚠️ **O id continua a chamar-se `PAINTER_WATERCOLOR_MIX` de propósito.** Ele é `hash_node_id` de uma
STRING: renomeá-lo muda o `NodeId`, que é o que a arrumação guardada do artista e todo
`register`/`route` já usam — *o preço de um nome mais honesto seria uma chave que não casa com
nada*. A herança está escrita no doc dele.

### §18.3 — O PREÇO, e onde ele mora mesmo

**Medido** (`--release`, mediana de 96 traços de 24 eventos, **89 % de CPU ociosa**):
`0,075 ms` sem contra **`0,130 ms`** com ⇒ **`1,73×`** o traço na CPU, e o absoluto fica bem abaixo
de `1 %` de um quadro.

⛔⛔ **Mas esse não é o preço inteiro, e o número grande não está nesta tabela:** o
`stamp_device::eligible` exige `pigment_mix == 0`, logo **ligar o knob DESLIGA o carimbo no
dispositivo** e o traço volta à rota em banda. Não é defeito — é o preço de uma lei que lê o alfa do
destino por texel —, e ele passou a estar escrito no doc daquela cláusula, que até hoje só falava da
aguada.

### §18.4 — Os gates, e por que dois deles vivem na crate ERRADA à primeira vista

**8 mutações, todas a sangrar**, com controlo sobre o próprio filtro (`passed+failed` do
`test result:`, porque `running N tests` conta os `#[ignore]`).

⭐⭐⭐ **M2 e M3 SOBREVIVEM no gate do PAINEL e sangram no da FERRAMENTA — e isso é o desenho.** O
gate do painel compara *a tela* com *a porta*; mexer na porta põe os dois lados a concordar e ele
fica verde sobre um knob inalcançável. ⇒ **a asserção da porta vive nos gates de LEI**
(`assert!(PaintMedia::Digital.offers_pigment_mixing())` depois de medir o barro): *só quem mede o
barro pode amarrar a porta*.

| # | mutação | sangra em |
|---|---|---|
| M1 | a cerca do meio volta ao `effective_pigment_mix` | ferramenta (2 de 3) |
| M2 | a porta passa a oferecer ao Wet Paint (knob MORTO) | ferramenta |
| M3 | a porta deixa de oferecer ao Digital (INALCANÇÁVEL) | ferramenta |
| M4 | a aguada deixa de ser subtraída (id pintado 2×) | painel |
| M5 | o cartão `Mixing` deixa de ser pintado | painel |
| M6 | a fileira mostra o campo CRU (`pigment_mix`) | painel |
| M7 | a cadeia do HR-12 é apagada (volta a exigir 1 salto) | `ph2d-editor-core` |
| M8 | o salto aponta para um ficheiro que a porta não chama | `ph2d-editor-core` |

### §18.5 — O HR-12 apanhou o ficheiro novo, e a cura foi ESTENDER a verificação

O `every_widget_file_wires_a11y` reprovou sobre `paint_pigment.rs`. Ele delega por `card_row`, que é
uma **porta desta crate** — a terceira forma que o gate já reconhece — só que de **DOIS saltos**
(`card_row` → `number_field::chip` → `paint_number_input_with_buffer`), e a verificação das
`PORTAS_DE_CRATE_VERIFICADAS` só sabia **um**.

⛔ **A cura NÃO foi isentar o consumidor** (`PANEL_A11Y_DELEGATE_OK`), que é como uma catraca vira
licença. Foi dar à lista um **4.º campo — o salto seguinte** —, que tem de existir, de ser
mencionado pela porta e de **ACABAR num primitivo**. *Exigir um salto só não era uma regra: era o
formato do primeiro caso* (o `fileira_de_param` do audio-editor, que a criou).

### §18.6 — Três notas que a troca deixou FALSAS

- o doc do campo `BrushSpec::pigment`: *«Only read when `watercolor` is on»*;
- o doc do `stamp_device::eligible`: *«o crossfade **RYB** do pigmento»* (a lei é K–M desde a §17);
- ⚠️ **a fixtura do teste dele**, que pregava `watercolor: true` — *sem essa linha ela lia `0` e o
  caso passava **pelo motivo errado**: ele media a aguada, não o pigmento*.

### §18.7 — Aberto, e de quem é

- ⏳ **O escurecimento chega agora a DOIS meios novos** — é a mesma troca que ele aprovou na aguada,
  e o veredito sobre ela no Digital é **smoke do dono**.
- ⏳ **O `Mixing` no Impasto não foi pedido**, e entra porque a medição diz que aquele meio lê a lei:
  a alternativa era um knob vivo sem botão. **Decisão do dono**, e reverter é uma linha.
- ⏳ Ficam os dois da §17.6: o **wet-on-wet** e a **aquarela com o glaze**.

---

## §19 — «CONFIRA SE FUNCIONA PARA BLUR E SMEAR»: não funciona, e a fileira aparecia lá (2026-09-20)

> Report do dono logo a seguir a aprovar o smoke da §18.

**A resposta directa: não.** E a pergunta expôs uma **classe**, não dois casos.

### §19.1 — A tabela, e o CONTROLO que a 1.ª corrida não tinha

A fixtura tem de ser outra: aqueles gestos **não depositam a cor do pincel**, eles mexem na que já
está ⇒ **duas faixas encostadas** (amarela/azul) com o gesto a correr **ao longo da fronteira**.

| gesto | `|Δ|max` | | gesto | `|Δ|max` |
|---|---|---|---|---|
| **brush** | **100** | | fill | `0` |
| blur | `0` | | knife | `0` |
| smear | `0` | | sculpt | `0` |
| clone | `0` | | mask | `0` |

⚠️⚠️ **A 1.ª corrida leu `0` em TODAS as linhas, o `brush` incluído** — ela corria à força cheia,
onde o composite devolve a cor de cima seja qual for a lei. *Com o controlo a ler zero, os zeros do
Blur e do Smear não ilibavam ninguém: a tabela dizia «nenhum gesto mistura», incluindo o que
mistura.* A meia força é o que a torna uma medição.

### §19.2 — Oito knobs mortos, seis deles anteriores a esta ordem

Medida a TELA, a fileira era pintada em **nove** gestos (brush · blur · smear · clone · sculpt ·
mask · fill · knife · borracha) e **um** deposita a cor de um dab. ⚠️ **Seis já eram mortos antes
desta wave** — a aguada mostrava a fileira no Blur desde que ela existe —, e a troca da §18 alargou
o defeito de **um meio para três**. *A pergunta dele não achou um defeito novo: achou um que a minha
wave multiplicou.*

### §19.3 — E o nono era um DEFEITO: a borracha TINGIA o que apagava

A borracha lia **`|Δ| 143`**. O `BrushBlend::EraseAlpha` devolve o RGB do destino **letra por letra**
e mexe só no alfa — e o crossfade do pigmento reescrevia-o por cima ⇒ *ela apagava o alfa e pintava,
com uma cor que o artista nem vê em modo borracha*.

⛔ **A cura é do BLEND e não do modo** (`BrushBlend::lays_pigment`): quem apaga é a borracha autorada
**ou** o botão direito do Grid Stamp, e as duas chegam ao carimbo pelo mesmo
`brush.blend = EraseAlpha` — *uma cerca no modo teria de ser escrita duas vezes e só uma delas seria
lembrada*.

⚠️ **Pré-existente**, alcançável desde que o `Pigment` existe, e **invisível a tudo**: nenhum gate
desta casa punha uma borracha e um pigmento na mesma fixtura.

### §19.4 — A oferta passa a ter UMA porta, e ela vive onde o facto vive

`BrushSettings::pigment_offered`, derivado pela ferramenta em `PaintMedia::offers_pigment_mixing_in`
= **o MEIO ∧ o GESTO ∧ não-borracha**.

⚠️ Ele é um **campo do snapshot** e não um predicado do painel porque a resposta precisa do
`PaintMode`, que o painel não vê — e re-derivá-lo de uma lista de `is_*` nasceria com **dois
buracos** (não há `is_fill` nem `is_knife`).

⚠️ `PaintMode::deposita_a_cor_do_dab` é `matches!(self, Paint)` com um `_ =>` deliberado: um braço
por variante empataria com o valor de fábrica e a mutação não o conseguiria matar, e assim **um modo
novo nasce do lado conservador**.

### §19.5 — ⭐⭐⭐ E o `Pigment` saiu do cartão Water porque um CENSO o mandou sair

Com a fileira a poder desaparecer, a 1.ª redacção derivou o `n_rows` do cartão *Water* da mesma
condição — e o `o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta` **reprovou em voz alta**:
*«o `n_rows` não é um literal»*.

⛔ **Ele tinha razão, e a cura barata era cegá-lo.** Ensiná-lo a ler um `if` fá-lo-ia um parser — e
ele existe porque um cartão dimensionado para `3` com `4` dentro já shipou uma vez (§14). ⇒ a fileira
mudou-se para **um hospedeiro só**, o cartão `Mixing`, que some INTEIRO.

⭐ *E essa é também a resposta certa de produto*, que eu tinha hesitado em tomar na §18 para não mexer
num painel aprovado: o controlo governa três meios, logo não pertence à secção de um — o mesmo
argumento que já está escrito para o chip do **Paint Mode** ficar acima do que governa.

### §19.6 — Mutação: 5 de 5 nesta wave (13 na ordem inteira)

| # | mutação | sangra em |
|---|---|---|
| M9 | a cerca do blend cai (a borracha volta a tingir) | ferramenta |
| M10 | todo gesto «deposita» (voltam os oito mortos) | ferramenta |
| M11 | nenhum gesto deposita (a fileira some da matriz) | painel |
| M12 | a borracha deixa de ser subtraída da oferta | ferramenta |
| M13 | o cartão Water volta a declarar `3` com `2` dentro | o censo dos cartões |

⭐⭐ **M10 e M12 SOBREVIVEM no gate do painel e sangram no da ferramenta — e isso é o desenho**, o
mesmo de M2/M3: o do painel compara *a tela* com *a porta*, logo mexer na porta põe as duas a
concordar. **Só quem MEDE o barro pode amarrar a porta**, e é por isso que cada gate de lei acaba
numa asserção sobre `pigment_offered`.

### §19.7 — Aberto

- ⏳ O `Pigment` mudou de sítio na **aquarela** também (saiu do cartão *Water* para o `Mixing`, acima
  da secção) — **smoke do dono**, e é a única coisa que ele vê mudar num painel que já aprovou.

---

## §20 — «MEÇA O CUSTO DE MAIS DUAS TOOLS NA CADEIA» (ordem do dono, 2026-09-20)

> *«Em Digital:Composite Brush temos 3 tools juntas Brush+Smear+Blur. Meça custo de colocar mais 2
> tool nesse cadeia: Erase + um segundo Brush. Nos brushes entram campos de cor e tamanho do carimbo
> além do Strength que já tem. Smear e Blur ganham o tamanho do carimbo além do Strength que já tem.
> Analise a viabilidade e o preço em performance.»*

⚠️ **Esta wave NÃO implementou nada.** Ela entrega a MEDIÇÃO e o desenho, que é o que a decisão
exige. O instrumento é [`diag_composite_cinco_camadas.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/diag_composite_cinco_camadas.rs)
— seis sondas `#[ignore]`d, todas pela porta do produto (`on_canvas_pointer`), `--release`, canvas
`1024²`, traço reto de `720 px` em passos de `2 px` (362 eventos), raio `24`.

```
bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
  diag_composite_cinco_camadas -- --ignored --nocapture --test-threads=1
```

### §20.1 — O PREÇO: as duas camadas custam **+19 %**, e a conta não é o que parece

| pilha | traço (ms) | por evento |
|---|---|---|
| pincel sozinho (pilha DESLIGADA) | `5,27` | `0,015` |
| `[Brush]` | `5,17` | `0,014` |
| `[Smear]` | **`67,22`** | `0,186` |
| `[Blur]` | `8,51` | `0,024` |
| `[Brush+Smear]` | `78,84` | `0,218` |
| `[Brush+Brush+Smear]` | `88,94` | `0,246` |
| **`[Brush+Smear+Blur]` ← HOJE** | **`95,36`** | **`0,263`** |
| **PREVISTO `[Brush+Brush+Erase+Smear+Blur]`** | **`113,91`** | **`0,315`** |

⇒ **`×1,19`**. As marginais são **EMPARELHADAS** (ver §20.5): 1.º Brush `10,47` `[10,19..10,67]` ·
2.º Brush `10,18` `[8,49..10,35]` · Blur `16,92` `[16,16..18,70]` · Erase `= Brush × 0,83` (medido:
uma borracha custa `4,65` contra `5,59` do pincel).

⭐⭐ **O `+19 %` é pequeno porque o SMEAR já é 70 % da pilha de hoje** (`67,22` de `95,36`), e ele
**segue a TELA e não o traço**:

| tela | `[Brush]` | `[Smear]` | `[Blur]` | pilha de HOJE |
|---|---|---|---|---|
| `512²` | `2,56` | `19,46` | `4,25` | `32,91` |
| `1024²` | `5,60` | `68,55` | `8,61` | `96,18` |
| `2048²` | `12,97` | `252,55` | `21,50` | **`307,18`** |

⚠️ **E é aqui que está o único risco de relógio real:** a `2048²` a pilha de HOJE já custa
`0,85 ms/evento`, e um rato de `1000 Hz` entrega `~16` eventos por quadro ⇒ **`13,6` dos `16,7 ms`
de um quadro**. Com cinco camadas isso vai a `~16,2` ms — *a pilha passa a comer o quadro inteiro
numa tela grande*. A `1024²` sobra folga (`4,2 → 5,0 ms/quadro`). ⛔ O método de traço de omissão
(`Space`) **não coalesce** eventos por quadro (`coalesces_canvas_motion` lista só Arc/Ellipse/
Polygon/Line/Anchored/DragDot), logo a conta é mesmo por EVENTO.

⭐ **A DOBRA é metade do preço de cada camada nova:** com uma sessão de smear viva, toda camada que
não é smear deposita **duas vezes** (canvas + base congelada, o `lay_into_smear_base` de 09/08) —
medido `×2,03` no Brush e `×1,99` no Blur. As partes sozinhas somam `80,91` e a pilha mede `95,36`
⇒ `+17,9 %`, que **é** a dobra.

### §20.2 — ⛔⛔ A VIABILIDADE: um SEGUNDO Brush é INERTE hoje, e são DUAS causas

| Strength | UM Brush | DOIS | o 2.º acrescentou |
|---|---|---|---|
| `1,0` | `101,99` | `102,00` | **`+0,01`** |
| `0,6` | `61,08` | `61,04` | **`−0,04`** |
| `0,3` | `30,73` | `30,67` | **`−0,05`** |

⭐ **A aritmética separa as duas, e elas pedem curas DIFERENTES:**

- **Em `1,0` o cap está DESARMADO** (`stroke_cover_wanted` é `!accumulate && strength < 1.0`): os
  dois passes depositam, e o segundo é invisível porque pinta **a mesma cor** sobre tinta opaca.
  ⇒ a **cor por camada** que o dono pediu cura exactamente isto.
- **Abaixo de `1,0` quem bloqueia é o CAP, e a cor NÃO o cura.** Pintar `0,6` com `[0,6, 0, 0]`
  sobre branco escurece o vermelho `0,6 × 0,4 × 255 = 61,2`; o medido é **`61,04`**. Dois passes
  dariam `1 − 0,4² = 0,84` ⇒ `~86`. *O número lido é o cap, ao décimo.* ⇒ a cura é um
  `stroke_mask` **por camada** — a mesma coisa que o `lay_into_smear_base` já faz à mão (ele salva
  e repõe o mapa à volta da dobra, e o doc dele diz porquê).

### §20.3 — ⛔⛔ O TAMANHO por camada NÃO é um multiplicador no dab

O `Dab` já carrega `radius_px` **e** `color`, o que faz parecer que as duas features são um campo.
A **cor é** (o stamp lê `d.color`; per-camada = reescrever esse campo numa lista de rascunho, custo
algorítmico **zero**). O **tamanho não é**: o espaçamento entre dabs foi resolvido pelo motor de
traço com o raio do PINCEL (`spacing × diâmetro`, fábrica `0,10`).

⭐ **Medido por equivalência EXACTA** — a lista de um pincel de raio `r` reutilizada por uma camada
de raio `k·r` é, para essa camada, a lista que ela emitiria com `spacing/k`; logo `raio 96 · sp
0,025` **é** «a camada de 96 sobre a lista de um pincel de 24 a `0,10`»:

| op | percurso PRÓPRIO (`sp 0,100`) | lista REUSADA (`sp 0,025`) | razão |
|---|---|---|---|
| Brush | `7,00 ms` | `25,97 ms` | **`×3,71`** |
| Smear | `95,04 ms` | `386,99 ms` | **`×4,07`** |
| Blur | `88,65 ms` | `344,14 ms` | **`×3,88`** |

⇒ **um tamanho por camada honesto é um PERCURSO DE TRAÇO por camada** (`[Stroke; N]` no lugar do
`Option<Stroke>` único do `paint::state`), nunca um raio escalado sobre a lista partilhada. Sem
isso, uma camada `4×` maior paga `4×` a mais do que precisa — e a sobreposição extra é o
endurecimento de borda que o doc 25 §13.10 já mede.

⚠️ **E mesmo com percurso próprio o Blur é o caro:** (raio `12 · 24 · 48 · 96`)

| op | 12 | 24 | 48 | 96 |
|---|---|---|---|---|
| Brush | `2,79` | `5,18` | `6,74` | `6,52` (plano) |
| Smear | `64,28` | `67,59` | `75,89` | `95,12` (`×1,4`) |
| **Blur** | `3,67` | `8,70` | `26,63` | **`87,40`** (`×10`) |

### §20.4 — O que a extensão encosta (superfície MEDIDA)

- ⭐ **Zero schema e zero contrato congelado.** O stack do composite **não é serializado** em lado
  nenhum (`composite_enabled`/`composite_ops`/`composite_strength` aparecem em 12 ficheiros de
  **duas** crates, nenhum deles de persistência): é estado de sessão. `PROJECT_SCHEMA` intocado.
- Arrays `[NodeId; 3]` → `[5]` em `ids/painter_brush_sections.rs` (`…COMPOSITE_STRENGTH`, `…_UP`,
  `…_DOWN`) e o `PAINTER_BRUSH_COMPOSITE_BUTTONS: [NodeId; 7]` → `[11]` (mais os ids novos de cor e
  tamanho por camada), com o registo em `populate.rs` a segui-los — ⚠️ é o gate
  `hit_indexed_ids_are_registered` que o cobra.
- `BrushSettings::composite_ops`/`composite_strength` `[_; 3]` → `[_; 5]` e os dois campos novos.
- O cartão (`paint_composite.rs`): `N_LAYERS` é o único literal `3` do painel.
- ⚠️ **O `composite_active()` exige `!eraser`** — uma camada `Erase` é um `BrushBlend::EraseAlpha`
  na camada, e **não** o modo borracha da ferramenta; a cura do pigmento de ontem
  (`BrushBlend::lays_pigment`) já a impede de tingir o que apaga.
- ⏳ **O PAINEL é o custo que não é CPU:** o cartão hoje mede `6 + 22 + 4 + 3×25 + 6 = 113 px`.
  Cinco camadas numa linha cada dão `163`; com cor **e** tamanho a pedirem uma segunda linha por
  camada dão **`288 px`** (`+175`) dentro da secção Brush, que já é a mais alta do painel.

### §20.5 — ⚠️ DUAS réguas minhas reprovaram antes de uma medir

1. **A ADITIVIDADE era um controlo VAZIO.** A 1.ª redacção somava as marginais
   (`c_S + (c_BS − c_S) + (c_BSBl − c_BS)`), que **telescopa** para `c_BSBl` por construção — ela
   imprimia `+0,0 %` e não afirmava nada. Hoje o controlo é a soma das partes **independentes**
   (`80,91` contra `95,36`), e o resíduo **é** a dobra.
2. **A MARGINAL subtraía dois mínimos de corridas separadas**, e numa leitura deu **`−0,54 ms`**
   para uma camada que custa `~10`: a dispersão do Smear nesta máquina é **`14 ms`** (leu `67,56` e
   `81,60` no mesmo dia). ⇒ *uma diferença de dois números ruidosos não é uma marginal*. Hoje `A` e
   `B` correm **lado a lado na mesma iteração** e o que sai é a **mediana das diferenças
   emparelhadas**, com `p10..p90` ao lado — a deriva entra nos dois lados e cancela-se.

### §20.6 — Recomendação, e o que fica para o dono decidir

**Viável.** O preço de CPU das duas camadas é `+19 %` de uma pilha que a `1024²` usa `5` dos
`16,7 ms` de um quadro. O trabalho não está no relógio — está em três peças:

1. um **`stroke_mask` por camada** (sem ele o 2.º Brush não pinta abaixo de Strength `1,0`);
2. um **percurso de traço por camada** (sem ele o tamanho custa `×4` e endurece a borda);
3. o **espaço do cartão** (`+175 px`), que é decisão de produto.

⏳ **Decisão do dono:** (a) a tela grande — a `2048²` a pilha de cinco põe o quadro no limite, e a
saída medida é o Smear, que é `70 %` dela e cresce com a ÁREA; (b) o cartão a `288 px`.

---

## §21 — «SIM. VAMOS FAZER» — a pilha de CINCO camadas, e o Blur da pilha com núcleo de CAIXA

> Ordem do dono, 2026-09-20, logo a seguir à medição da §20: *«sim. vamos fazer. veja se abaixando
> a qualidade do blur não fica bem mais leve. Mas só no Blur do composite. O Blur como ferramenta
> isolada não deve ser modificado.»*

### §21.1 — O que o artista passa a poder fazer

O *Composite Brush* tinha **três** camadas (Brush · Smear · Blur) com **um** knob cada (Strength).
Passa a ter **cinco**, cada uma com um **selector de operação** (Brush · Smear · Blur · **Erase**),
e cada uma com **tamanho do carimbo** próprio; as que depositam cor ganham também uma **cor**
própria, com botão de voltar à cor do pincel.

As duas posições novas **nascem caladas** (`strength = 0`), logo *quem já usava a ferramenta não vê
um pixel diferente* — há gate a afirmá-lo por igualdade byte a byte contra uma pilha de três.

### §21.2 — Três coisas que a construção obrigou, e nenhuma estava no pedido

**(a) Duas camadas Brush partilhavam o CAP de cobertura e a segunda depositava ZERO.** O
`stroke_mask` é o tecto do *Accumulate OFF* e é **por TRAÇO**; com duas camadas a primeira levava-o
ao tecto e a segunda não tinha onde escrever. Medido antes da cura, com Strength `0,6`: **uma**
camada deixava `61,08` de tinta e **duas** deixavam `61,04` — e `0,6 × 0,4 × 255 = 61,2` é
exactamente o cap, quando dois passes dariam `1 − 0,4² = 0,84` ⇒ `~86`. ⇒ **uma máscara por
POSIÇÃO** (`composite_mask[pos]`, trocada para dentro e para fora por `mem::swap` à volta de cada
camada). O gate mede a **TINTA** e leva o **CONTROLO** da mesma pilha com uma camada só — *um número
sozinho não diz se a segunda camada trabalhou*.

**(b) Reutilizar a lista de dabs num carimbo maior custa ×4, e isso é EXACTO.** O `spacing` é uma
fracção do **DIÂMETRO**; dobrar o raio de uma camada sem tocar na lista dá um espaçamento efectivo
de `0,10 → 0,025`, ou seja **quatro vezes** os dabs pelo mesmo traço. ⇒ a camada com `size > 1`
**sub-amostra por arco** (`composite_arco[pos]`), e a que tem `size == 1` e nenhuma cor autorada
devolve `None` — *a mesma fatia, zero cópias*.

**(c) Os dois limites do tamanho são DERIVADOS, não escolhidos** (§0.0): o **piso** é o `spacing` do
próprio pincel — *abaixo dele o traço conta como esferas* e o defeito começa exactamente ali —, e o
**tecto** é `4,0`, que é onde o relógio do quadro o põe.

### §21.3 — O núcleo de CAIXA, e o que a medição pareada corrigiu

Três caixas por **somas correntes** no lugar da convolução binomial, com as larguras escolhidas por
**casamento de variância** (`σ² = k/2`; a largura ideal `√(1+2k)` realizada como mistura de duas
larguras ímpares vizinhas). Custo `~6` ops/pixel **independente de `k`**, contra `2(2k+1)` taps.

**MEDIDO** (`--release`, canvas `1024²`, traço de 720 px, **pareado**, `load 3,4`–`4,0`):

| raio | binomial | caixa | ganho p50 (p10..p90) | pior byte | média |
|---|---|---|---|---|---|
| 12 | `4,03` | `4,09` | **`×0,98`** (`0,95`..`1,00`) | `1` | `0,001` |
| 24 | `8,84` | `7,04` | `×1,26` (`1,23`..`1,27`) | `3` | `0,017` |
| 48 | `24,21` | `13,31` | `×1,83` (`1,76`..`1,86`) | `1` | `0,001` |
| 96 | `84,23` | `26,28` | **`×3,19`** (`3,09`..`3,26`) | `1` | `0,007` |

⛔⛔ **A leitura anterior dizia `×1,21` no raio 12 e estava ERRADA.** Ela era `min(B)/min(A)` de
corridas **separadas** a `load 15` — *a mesma forma que esta sessão já tinha pago uma vez*, quando
um marginal de `~10 ms` se leu como **`−0,54 ms`**. Pareada, ali **não há ganho nenhum**: a caixa só
se paga a partir do raio `~24`, porque três passagens com avental próprio custam o que um binomial
de lado `9` custa. ⇒ *um ganho lido de dois mínimos de janelas diferentes é um palpite com cara de
medição, e o erro tanto inventa ganho como o esconde.*

A diferença de **imagem** é de `1` a `3` bytes no pior pixel e `0,001`–`0,017` na média: **a
«qualidade mais baixa» que o dono autorizou não é visível.**

### §21.4 — A fronteira do dono é GATEADA, em três metades

`o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele`, e cada metade sozinha mente:

1. **as duas rotas DISCORDAM** — sem isto a pilha podia estar no binomial em silêncio e o ganho
   medido seria de outra coisa;
2. **elas discordam POUCO** (`pior ≤ 8`) — é esta que torna *«baixar a qualidade»* uma afirmação e
   não uma esperança;
3. o `Caixa` é pedido num **ficheiro só** de toda a crate da ferramenta, o do laço da pilha.

⚠️ A 3.ª é **textual de propósito**: a rota isolada entra por uma porta **sem argumento**
(`stamp_dabs_blur`), logo o binomial dela é **por construção** e não há barro onde medir a ausência
— *uma ausência prova-se contando os sítios, não olhando*.

⚠️⚠️ **E a agulha é montada em runtime porque este ficheiro seria ele próprio um acerto** (a lição
do gate auto-referente da escultura). A 1.ª redacção esperava `composite.rs` **e este ficheiro**, e
**reprovou alto**: eu escrevera a expectativa como se ele fosse cúmplice e o código já o punha de
fora.

⭐ E o núcleo é nomeado **uma vez** no braço do Blur (`const NUCLEO`), com **dois** chamadores —
escrito duas vezes, o dia em que um deles mudasse deixava a base do smear a receber um borrão de
outra lei, e *as duas metades do mesmo depósito divergiriam em silêncio*.

### §21.5 — Prova de mutação: 5 de 5, e DUAS sobreviveram primeiro

As duas sobreviventes nomearam gates **meus** a afirmar menos do que prometiam:

⛔ **`a_caixa_tripla_preserva_um_campo_constante` varria só `0..3`.** Trocar `1/lado` por
`1/(lado − ½)` — um ganho uniforme de `20 %` — passava **VERDE**. Este é um borrão em espaço
**premultiplicado** e o último passo un-premultiplica por `255/α`: *um ganho uniforme entra no
numerador e no denominador e divide-se a si próprio*. O único canal que não é dividido por nada é
o **α**. ⇒ **num borrão premultiplicado, um campo constante de RGB não é régua de normalização
nenhuma.**

⛔ **`os_dois_nucleos_borram_a_mesma_quantidade` ficava trivialmente verdadeiro** com o despacho a
ignorar o núcleo pedido: os dois lados passam a ser o **mesmo** binomial e `d = 0`. *Uma régua de
SEMELHANÇA é satisfeita de graça pela IDENTIDADE* ⇒ ela precisa do controlo **«e as duas saídas
DIFEREM»**, que é o que prova que o argumento chegou a ser lido.

⚠️ E uma terceira **abortou**: a agulha `let alvo = k as f32 / 2.0;` casa **duas** vezes, porque o
gate recalcula o alvo com a mesma expressão do produto (ali é uma fórmula do domínio, não uma
constante partilhada — mas a agulha teve de levar contexto).

### §21.6 — O preço das cinco camadas, com o Blur já barato

Re-medido a `load 3,4` com o núcleo novo já no sítio (`raio 24`, traço de 720 px, 362 eventos):

```
  pincel sozinho (pilha DESLIGADA)      5,82 ms    0,016 ms/evento
  [Brush]                               5,30 ms
  [Smear]                              70,82 ms
  [Blur]                                6,77 ms        (isolado ⇒ binomial)
  [Brush+Smear+Blur]  <- HOJE          94,35 ms    0,261 ms/evento
  1.º Brush  11,05 ms [10,67..11,48]   contra 5,30 sozinho ⇒ a DOBRA vale ×2,09
  2.º Brush  10,03 ms [ 6,91..12,69]   <- a camada que o dono pediu
  Blur       13,02 ms [12,81..14,14]   contra 6,77 sozinho ⇒ ×1,92
  PREVISTO [Brush+Brush+Erase+Smear+Blur] = 112,55 ms (0,311 ms/evento) = ×1,19 a pilha de hoje
```

⚠️ **A DOBRA é o mecanismo dominante e não é desta wave:** com uma sessão de Smear viva, toda camada
que não é Smear deposita **DUAS** vezes (canvas + a base congelada do Smear) — é a cura de
2026-08-09, sem a qual um traço perde `33` das `141` colunas. Medida aqui em `×2,09` (Brush) e
`×1,92` (Blur), e é por isso que as partes sozinhas somam `82,89` e a pilha mede `94,35`
(**`+13,8 %`**).

⚠️ E o `Smear` é **`70,82` dos `94,35`**: *a camada que o dono acrescentou custa `10 ms` numa pilha
cujo membro mais caro custa setenta*.

### §21.7 — O que NÃO se mexeu

Zero `PROJECT_SCHEMA`, zero contrato congelado (a pilha **não** é serializada), zero pacote novo, e
**o Blur isolado sai byte a byte igual ao de sempre**. Os ids novos do painel entram **apendados**
aos arrays que já existiam, logo os três primeiros hashes ficam intactos.

### §21.8 — Aberto

- ⏳ **O tecto de `4,0` do tamanho é do relógio a `raio 24`** — num pincel muito maior ele é
  generoso de mais, e a faixa não foi varrida por raio.
- ⏳ **O cartão cresce de `113 px` para `~288 px`** com cinco camadas de duas fileiras; não há
  medição de quanto isso empurra o resto do painel para baixo da dobra.
- ⏳ **O ganho da caixa a `raio 12` é `×0,98`** — dentro do ruído, e o `p90` toca `1,00`. Não há
  cerca por raio a desligar a caixa em baixo, e nenhuma medição diz que valha a pena tê-la.

### §21.9 — O que o PORTÃO cobrou depois de a wave estar escrita

A varredura impactada fecha **`18 216` de `18 216`**. Antes disso ela devolveu **dois** vermelhos, e
nenhum era sobre a lei desta wave: os dois são de arquitectura, e os dois foram curados **por corte
e por porta** — nunca por uma entrada nova numa lista de dívida.

**(a) O tecto de LOC do `blur.rs`** (`822` contra `700`) — o núcleo de caixa entrou num ficheiro que
já estava perto do tecto. ⇒ [`blur_caixa.rs`](../../../crates/ph2d-painter-brush/src/blur_caixa.rs),
que é *as três passagens de caixa por somas correntes e a variância que as escolhe*:
**`822 → 579 + 279`**, e o módulo novo carrega no cabeçalho a tabela medida e a nota de que nasceu
de um CORTE.

⛔⛔ **E o corte pagou DUAS armadilhas que esta casa já tem escritas:**

- **A varredura subia por `///` e cortou DENTRO do item vizinho.** Ela levou o doc-comment do
  `blur_region` — que **fica** com o binomial — e mais o `src_coord` inteiro. *Um corte que sobe por
  doc-comment em vez de por `#[` deixa o vizinho com a prosa de outro e sem a dele, e a metade que
  falta é muda.* Os dois voltaram ao sítio; o `src_coord` é hoje `pub(crate)`.
  ⛔⛔ **E o corte trocou MAIS DOIS docs de sítio, nos DOIS sentidos — e só o CLIPPY os viu:**
  o do `blur_region_caixa` **ficou para trás** em `blur.rs` (logo a função que VIAJOU chegou ao
  ficheiro novo sem prosa nenhuma, e a que fica herdou um parágrafo sobre outra lei), e o do
  `blend_blurred` **viajou** para o `blur_caixa.rs` (onde ficou órfão por cima do `mod tests`, com o
  `blend_blurred` a ficar sem doc em `blur.rs`). ⚠️ **Nenhuma das três suítes o podia ver** — *prosa
  não é compilada contra nada* — e o passe de compilação também não: quem os apanhou foi o
  `empty_line_after_doc_comments` sob `-D warnings`, **e só porque a residência deixou uma linha em
  branco**; sem ela o doc trocado ficava verde para sempre.
  ⚠️⚠️ **E o clippy dá UM erro por ALVO, não por ficheiro** — a 1.ª corrida acusou o `lib` e a 2.ª o
  `lib test`, ou seja *uma corrida verde a seguir a uma cura não prova que não há um terceiro*. ⇒ a
  prova que fecha o assunto é de **CONJUNTO**: as linhas `///` dos dois ficheiros DEPOIS do corte
  contêm as de ANTES (`96` → `104`, com as duas «em falta» a serem as que esta jornada reescreveu
  para o link absoluto), e nenhuma função das duas metades fica sem doc por cima.
- **Um passe de compilação VERDE com a suíte VERMELHA.** O `BLUR_KERNEL_MAX` deixou de estar à vista
  dos testes do módulo novo — a cegueira do `--all-targets` a um `mod tests` que atravessa uma
  fronteira de MÓDULO, irmã pequena da que o HOWTO §2 regista para uma fronteira de *crate*.

**(b) O `INDENT: f32 = 14.0` do painel**, acusado pelo `no_surface_declares_an_indent_constant_of_its_own`
na primeira corrida. ⇒ `ph2d_tokens::list_indent_px()`. ⚠️ **O marcador `LITERAL-PX-OK` que estava ao
lado NÃO isenta**, e é essa a parte que engana: *a queixa daquele gate não é o literal, é a SEGUNDA
resposta* — dois números de recuo divergem no dia em que alguém mexer num deles, e o artista vê duas
fileiras que deviam alinhar e não alinham.

**E a prova de mutação foi RE-CORRIDA sobre a árvore CORTADA** — *mover código parte gates em duas
espécies e só uma avisa*, logo `5 de 5` sobre o ficheiro antigo não afirma nada sobre o novo:

| # | mutação | gate que sangra | população |
|---|---|---|---|
| B1 | o composite pede o núcleo binomial | `o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele` | `1` de `9` |
| B2 | a porta isolada crava o núcleo de caixa | o censo de alcance do mesmo gate | `1` de `9` |
| B3 | a variância alvo `k/2 → k/4` | `a_caixa_tripla_tem_a_variancia_do_binomial` | `2` de `10` |
| B4 | a des-premultiplicação `255/α → 1` | `a_caixa_tripla_preserva_um_campo_constante` | `1` de `10` |
| B5 | o despacho do núcleo | `os_dois_nucleos_borram_a_mesma_quantidade` | `1` de `10` |

⛔⛔ **E o ARNÊS mentiu de três maneiras antes de dizer a verdade, as três já registadas neste repo:**

1. **Ele foi morto a meio (`exit 137`) e deixou o ficheiro MUTADO na árvore**, com o `.bak` ao lado —
   *uma prova de mutação que não sobrevive a ser morta PLANTA um defeito, e o `git status` dela
   lê-se como «trabalho por committar»*. ⇒ `trap … EXIT INT TERM` que repõe **sempre**, e a reposição
   faz `touch` (um `mv` devolve o mtime ANTIGO e o cargo guarda o build da mutação).
2. **A âncora do B3 casou DUAS vezes** — o produto e **o gate, que recalcula a mesma fórmula**. Uma
   mutação que casa `N` vezes não é a que se quis medir, logo o arnês **aborta** o caso em vez de o
   reportar.
3. **Um `\n` dentro de `'…'` do bash são dois caracteres** (foi preciso `$'…'`) e, na 1.ª redacção,
   um caso abortado matava a corrida inteira em vez de contar e seguir.

---

## §22 — A PILHA É DO TRAÇO, e a borracha ganha ESCOPO

> Ordem do dono, 2026-09-20, em três reports e um pedido:
> *«o brush de cima sempre deve ser desenhado por cima do Brush de baixo. Atualmente o brush de
> baixo cobre o brush de cima do carimbo anterior. corrija»* · *«Blur em cima não consegue borrar os
> Brushes. corrija e veja se smear também está bugado»* · *«o Z index não funciona também para
> eraser em cima… tem que rever para todos»* · *«Vamos precisar de uma opção em erase: se a borracha
> atua só no próprio traço do Brush ou se ela apaga também a camada da imagem abaixo.»*

### §22.1 — O mecanismo do defeito

O `stamp_dabs_composite` corria `for pos in (0..N).rev()` sobre a fatia de dabs **daquele lote**. A
ordem DENTRO de um lote estava certa; entre lotes estava invertida — um traço chega em dezenas de
eventos de ponteiro, e o lote `k+1` começa pela camada de BAIXO, que aterra por cima do que a de
CIMA do lote `k` acabou de pintar. Com o espaçamento de fábrica dois dabs consecutivos sobrepõem-se
~90 %, logo **a camada de cima só sobrevive na lasca da frente**.

### §22.2 — A régua UNIVERSAL, e o que ela mediu

*O mesmo traço entregue numa tacada ou em N lotes tem de dar a MESMA imagem* — ela não sabe o que
cada camada faz, só que a ordem é da PILHA e não da taxa do rato. `|Δ| médio` sobre a linha central,
topo sobre um Brush azul, contra a entrega num lote só:

| topo | passo | ANTES | DEPOIS |
|---|---|---|---|
| Brush vermelho | 2 | `41,59` (pior `92`) | **`0,00`** |
| Brush vermelho | 8 | `31,26` (pior `90`) | **`0,00`** |
| Erase / Smear / Blur | 2 e 8 | `0,00` | `0,00` |

⚠️ **As três linhas que já liam `0,00` não eram um motor certo — era a FIXTURA a não conter o
fenómeno** (apagar à força cheia satura; borrar o miolo chapado de um traço é um no-op). Elas ficam
porque a régua tem de reprovar no dia em que alguém as quebrar.

E a cor na linha central, com dois Brushes de Strength `1`:

| passo | ANTES | DEPOIS |
|---|---|---|
| 2 | `R 167,3 · B 87,7` (o fundo aparece) | **`R 255,0 · B 0,0`** |
| 8 | `R 187,9 · B 67,1` | **`R 255,0 · B 0,0`** |
| 280 (um lote) | `R 250,4 · B 4,6` | `R 255,0 · B 0,0` |

### §22.3 — A lei nova

```text
    tela = L₀( L₁( … L_N( pre ) … ) )
```

com **cada `L` aplicada sobre o TRAÇO INTEIRO** e `pre` = a tela no pen-down. A pilha corre **por
CAMADA e não por lote**. É a lei que o `PerLayerStroke` já declara para as camadas de Shape, um
nível acima.

⭐ **E ela NÃO é `O(n²)`:** as operações são locais, logo a tela só muda onde os dabs NOVOS caem.
Recompõe-se essa região a partir do `pre`, replayando **só os lotes cuja caixa a toca** — um número
que não cresce com o traço (vale `~2/spacing`, a sobreposição). Os seis passos estão comentados no
sítio, em [`composite_pilha.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/composite_pilha.rs).

### §22.4 — As quatro coisas que a construção OBRIGOU, e nenhuma estava no pedido

1. ⭐⭐⭐ **A DOBRA MORREU.** Ela existia porque o render do Smear parte de uma base congelada que
   não via o depósito das outras camadas ⇒ toda camada não-smear depositava **DUAS** vezes (medido
   ×2,09 num Brush, ×1,92 num Blur). Hoje a base é **refrescada da tela** no momento em que a vez do
   knife chega na pilha — o que põe lá **exactamente as camadas de BAIXO**, que é a ordem que o
   report pedia, em vez de todas, que era o defeito. −68 linhas e um passe a menos por camada.
2. ⛔⛔ **A base do smear refresca-se na `caixa_nova`, NUNCA na `caixa_grande`** — e isto custou a
   medição mais cara da wave. Dentro da caixa grande as camadas de depósito estão **incompletas**
   (só os lotes da janela foram replayados), logo refrescar ali escreve BRANCO na base onde os dabs
   antigos tinham pintado, e o render devolve a cauda do traço à cor do papel: `|Δ| médio 110,13`,
   `pior 255`. *A região em que o resultado é correcto é exactamente a que sobrevive.*
3. ⚠️ **O RNG é por CAMADA.** Com a recomposição a correr por camada, um fluxo partilhado seria
   consumido noutra ordem a cada lote — *o Grain Random a cintilar por baixo da mão*. Cada lote
   guarda a posição do fluxo de cada camada e o replay repõe-na. ⛔ E o jitter de **COR** não o
   revela: ele é assado na lista de dabs, logo é replay-safe por construção — quem o observa é o
   **grão com mapeamento aleatório**, e é com ele que o gate mede.
4. ⚠️ **O cap de Accumulate de cada camada recomeça na região recomposta.** Ele é um mapa por
   TRAÇO; deixá-lo cheio faz o replay encontrar o tecto atingido e depositar **ZERO** — o traço
   desaparece à medida que a mão anda. Só é observável com `strength < 1`: *um corpus no ponto
   neutro de um mecanismo não testa esse mecanismo*.

### §22.5 — A BORRACHA GANHOU ESCOPO

`Image` (o valor de fábrica, o comportamento de sempre) · `Stroke` — *devolve o pixel ao que ele era
antes deste traço*. O chip vive na fileira B, que **numa borracha está vazia à esquerda** (a amostra
de cor e o botão de volta só existem para quem deposita) ⇒ **zero pixels de altura a mais no cartão**.

⭐ **A álgebra é exacta, não uma aproximação:** no ponto da pilha em que a borracha corre a tela é
`pre ⊕ tinta`, e o que se quer é `pre ⊕ (tinta·(1−c))`. Desenvolvendo o `over`, isso é **idêntico**
a `lerp(tela, pre, c)` — canal a canal, alfa incluído.

⛔ **E a cobertura `c` é MEDIDA, nunca recuperada do alfa:** `c = 1 − α_depois/α_antes` funciona…
menos onde `α_antes = 0`, que é exactamente o pixel que uma borracha de escopo `Image` numa camada
de baixo acabou de esvaziar. Ela corre no **ESCUDO**: um plano opaco onde o MESMO depósito escreve,
e `c = 1 − α`. Uma lei, uma porta, nenhum caso degenerado.

**Medido** (papel verde opaco, uma camada Brush vermelha, a borracha por cima):

| | pixel do miolo |
|---|---|
| sem borracha (controlo) | `(255, 0, 0, 255)` — a tinta |
| escopo `Image` | `α = 0` — **a imagem foi-se com a tinta** |
| escopo `Stroke` | `(0, 200, 0, 255)` — **o verde de antes do gesto** |

### §22.6 — O PREÇO, medido

⛔⛔ **ESTA TABELA FOI RETIRADA EM 2026-09-22 — a tabela válida é a da §22.6-bis, logo abaixo.** Ela
foi medida a `load 29` (o §5 da casa: *nenhuma leitura de relógio desta máquina vale nada acima de
`load ~5`*), e re-tirada com a máquina a **91–95 % ociosa** não reproduz: a `2 Brush` a raio 24 lia
`5,73` e lê **`24,49`**. ⚠️ **Não é o motor que mudou** — a sonda é a MESMA (`diag_preco_da_pilha`,
tocada desde então só pelo `rustfmt`) e as duas rotas foram medidas: a acumulação lê `24,49` e o
replay `111,28`. *Porque é que ela leu `5,73` fica por explicar, e dizer que sei seria inventar.*

⛔⛔ **E a frase que ela carregava está REFUTADA:** *«a recomposição de duas e três camadas de
depósito não é distinguível de uma»* — com a máquina calma, `2 Brush` custa **`4,1×`** uma e
`3 Brush` **`5,9×`**. ⭐ Ela tinha a forma exacta de uma fixtura que não contém o fenómeno: *três
linhas que leem o mesmo é o que se vê quando o trabalho não está a correr*, e aqui foi lida como uma
propriedade do motor.


### §22.6-bis — O PREÇO, re-medido com a máquina CALMA (2026-09-22)

`--release`, canvas `1024²`, traço de 720 px em passos de 2 px (**362 eventos**), mínimo de três
corridas, `load 3,1` com **95 % de CPU ociosa** — e estável (`24,49` · `24,95` · `25,37` em três
corridas). ⚠️ A minha cura do dia foi **ablada** para a atribuição: sem ela lê `24,08`, dentro do
ruído ⇒ ilibada.

| pilha | raio | traço | **ms/evento** | **% de um quadro** |
|---|---|---|---|---|
| 1 Brush (sem recomposição) | 24 | `6,01` | `0,017` | `0,1 %` |
| 2 Brush | 24 | `24,49` | `0,068` | `0,4 %` |
| 3 Brush | 24 | `35,73` | `0,099` | `0,6 %` |
| Blur sobre Brush | 24 | `88,51` | `0,245` | `1,5 %` |
| Smear sobre Brush | 24 | `43,35` | `0,120` | `0,7 %` |
| 1 Brush (sem recomposição) | 96 | `6,79` | `0,019` | `0,1 %` |
| 2 Brush | 96 | `64,26` | `0,178` | `1,1 %` |
| 3 Brush | 96 | `96,32` | `0,266` | `1,6 %` |
| **Blur sobre Brush** | 96 | `342,34` | `0,946` | **`5,7 %`** |
| Smear sobre Brush | 96 | `90,03` | `0,249` | `1,5 %` |

⭐⭐ **A COLUNA QUE DECIDE É A DO EVENTO, e a tabela antiga só tinha a do traço.** O que tem de caber
num quadro de `16,7 ms` é o custo de **um** evento de ponteiro; somar o traço inteiro faz um custo
perfeitamente interactivo (`0,4 %` de um quadro) parecer um congelamento de `24 ms`. ⇒ **o pior caso
da pilha é `5,7 %` de um quadro**, e não há problema de relógio nenhum a resolver aqui. A sonda passou
a imprimir as duas colunas.

⭐ **E a acumulação da §24 está confirmada a fazer o trabalho dela:** no mesmo traço recto, o replay
(`PH2D_COMPOSITE_REPLAY=1`) lê `111,28` contra `24,49` — **`4,5×`**.

⛔ **Com MENOS DE DUAS camadas activas nada disto corre** — não há ordem para arrumar, e nem a
fotografia do `pre` é paga (gate `uma_camada_so_nao_abre_a_recomposicao`, com controlo).

### §22.7 — Prova de mutação: 10 de 12 sangram, 2 NOMEADAS

| | |
|---|---|
| M1 a recomposição nunca corre | SANGRA |
| M2 a janela é só o lote novo | SANGRA |
| M3 sobrevive a caixa grande em vez da nova | SANGRA |
| M4 a base do smear é refrescada na caixa grande | SANGRA |
| **M5 o render do smear não é limitado** | **NOMEADA** |
| M6 o cap da camada não é limpo | SANGRA |
| M7 a borracha ignora o escopo | SANGRA |
| M8 a borracha do traço não devolve o `pre` | SANGRA |
| **M9 a pilha não fecha no pen-DOWN** | **NOMEADA** |
| M11 a recomposição corre de cima para baixo | SANGRA |
| M12 o replay não repõe o fluxo de RNG da camada | SANGRA |
| M13 a pilha não fecha no pen-UP | SANGRA |

* **M5** é um guarda de **RELÓGIO e não de imagem**: com a base refrescada só na `caixa_nova` ela
  fica correcta em toda parte, logo o render da união inteira dá a MESMA imagem. O que ele compra
  está medido: `Smear sobre Brush` de `70,43 → 184,40 ms` (raio 24) e `98,94 → 368,13` (raio 96).
* **M9** é a metade DEFENSIVA — o `close_stroke` fecha a pilha em todo pen-up (M13 sangra), e esta
  cobre o gesto que acaba sem passar por lá.

### §22.8 — Três armadilhas de ARNÊS, e a terceira é nova neste repo

1. **`grep -cF` conta LINHAS.** Numa âncora multi-linha ele lê o número errado e o caso ABORTA (ou,
   pior, muta o sítio errado). A contagem é de OCORRÊNCIAS.
2. **Um filtro que casa ZERO testes imprime `ok`** e lê-se como *«sobreviveu»* — o arnês conta
   `passed + failed` e acusa.
3. ⛔⛔⛔ **DUAS corridas do MESMO arnês na MESMA árvore destroem-na em silêncio.** Eu lancei três
   (duas em segundo plano e uma em primeiro) e elas colidiram no `.bak`: a segunda restaurou o
   ficheiro **já mutado** da primeira, o `mv` da primeira falhou com uma linha de erro no meio da
   tabela, e a árvore ficou com **duas** mutações vivas — com o «controlo» a ler `1 passed;
   6 failed` sobre elas, que se lê como *«o produto está partido»*. ⇒ o arnês ganhou uma **tranca**
   (`flock`), e *um arnês de mutação é um recurso EXCLUSIVO da árvore, como a placa*.

### §22.9 — O que fica ABERTO

* ⏳ **O RELEVO não entra na recomposição.** As três metades do impasto (`heights`/`covers`/`mats`)
  são escritas pelo depósito e **não** são guardadas nem repostas pela janela — no meio Digital,
  onde o composite vive, não há relevo, e o gate que o afirmaria não existe.
* ⏳ **A tabela de relógio foi tirada a `load 29`** e tem de ser re-tirada com a máquina calma.
* ⏳ **O cartão com cinco camadas × três fileiras** continua sem medição de quanto empurra o resto
  do painel para baixo da dobra — item herdado da §21.

---

## §23 — O PORTÃO DE FECHO apanhou TRÊS vermelhos, e nenhum era a wave de hoje

O `nextest-impacted` do fecho (`18 229` testes) reprovou em três. ⚠️ **Dois deles foram
introduzidos pelas waves ANTERIORES desta mesma linha** (a §20/§21, a das cinco camadas e a da
ordem de fábrica do dono) e ficaram vermelhos **em silêncio** porque aquelas waves correram o
portão sobre um FILTRO — *o laço interno corre o que se está a escrever, e o que se parte é
sempre outra coisa*.

### §23.1 — O censo do núcleo de caixa mudou de FICHEIRO, e a lei não

`o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele` lê `left: ["composite_pilha.rs"]` contra
`right: ["composite.rs"]`. A lei é *«o núcleo de caixa só pode ser pedido dentro do LAÇO DA
PILHA, e o censo nomeia onde ele mora»* — e o laço mudou de casa nesta wave, quando a
recomposição passou a ser por CAMADA. ⇒ o nome esperado passa a ser o novo, **com a mudança de
endereço escrita na mensagem**: se o censo voltar a acusar dois sítios, o pedido escapou do laço.

⚠️ *Este é o gate a funcionar.* Um censo que nomeia um ficheiro é a única coisa que impede o
pedido do núcleo barato de se espalhar para o Blur isolado — que é a fronteira que o dono
autorizou («**só** no Blur do composite»).

### §23.2 — Um teste que CRAVA uma posição mede a ORDEM DE FÁBRICA, não o que ele diz medir

`composite_brush_runs_an_isolated_layer_and_reorders` lê `left: Erase, right: Blur`. Ele isolava
a camada Blur por `STRENGTH[0] = 0`, `[1] = 0`, `[2] = 1` — com o comentário
*«default positions: 0 Brush · 1 Smear · 2 Blur»* ao lado — e subia a posição `2` ao topo com dois
cliques escritos à mão.

⛔ **A ordem de fábrica é uma DECISÃO DE PRODUTO**, e o dono reordenou-a na §21
(`Brush · Brush(0) · Erase(0) · Smear · Blur`). A partir daí a posição `2` é a **borracha**, e o
teste passou a reprovar sobre um motor correcto, **a acusar a lei do reordenar**. ⇒ o índice do
Blur é **DERIVADO** da pilha (`.position(|l| l.op == Blur)`), as outras camadas são zeradas por
laço, e a subida é `for i in (1..=blur).rev()`.

⚠️ *A wave que mudou a ordem trouxe um gate a afirmar que ela «não muda um pixel» — e essa
afirmação é sobre o EFEITO, que corre de baixo para cima e só viu camadas de força zero trocarem
de sítio. Ela é verdadeira, e não dizia nada sobre quem endereça a pilha por ÍNDICE.*

### §23.3 ⭐⭐⭐ — Um CONTROLO POSITIVO construído sobre uma população que ENCOLHE quando o código melhora

`every_slider_wears_the_live_hover` (foundational, `ph2d-editor-core`) reprovou com
*«o scanner viu apenas 4 cadeias `Slider::new(...).visual(...)` — ele está partido»*, contra um
piso de `5`.

⛔⛔ **O scanner não estava partido: a população migrou.** A §21 trocou, no cartão do composite,
duas barras feitas à mão pelo `paint_slider_with_chip_layout` — **a porta**, que pergunta ao store
por DENTRO e que o doc-comment daquele mesmo gate já nomeia como *«a alavanca de verdade, que não
aparece nesta varredura»*. A contagem caiu de `6` para `4`, e **o gate reprovou sobre código
melhor do que o que ele defendia**.

⭐ **A cura é mover o CONTROLO para a grandeza que a migração não consome:** o scanner passa a
devolver também **quantos sítios de `Slider::new(` ele ENXERGOU** (convertidos ou não — `19` em
`9` ficheiros, medido), e é esse o piso que prova que a varredura funciona. A metade dos
convertidos fica como piso do caminho **que ainda existe**, com os quatro sítios NOMEADOS e com a
saída escrita: no dia em que todos migrarem, essa metade deixa de descrever alguma coisa e **sai**
— a do `seen` fica.

⚠️⚠️ **A lei, que vale para todo controlo positivo deste repo:** *um piso que conta a população de
um padrão que a melhoria ELIMINA é uma catraca ao contrário — ele obriga o próximo autor a manter
o padrão velho, ou a baixar o número até ele não medir nada.* O controlo tem de medir o
INSTRUMENTO (a varredura vê a árvore?), nunca a dívida que ele conta.

---

## §24 — A AUDITORIA de 2026-09-21, e a pilha que passou a ACUMULAR

**Report do dono, com duas fotos:** *«1) A performance ficou ruim, muito aquém do esperado.
Pinceladas rápidas quase travam a tool. 2) Temos problemas com o carimbo que fica retangular.
Provavelmente o principal culpado é Smear. Estude as otimizações e correções do sistema per-layer
color — talvez descubra lá as soluções.»* Depois da auditoria: *«siga»*.

⭐ **Ele tinha razão nas duas, e o ponteiro dele era a cura da primeira.** A auditoria inteira, com
as tabelas, está em [`docs/Painter/40_auditoria_da_pilha_2026-09-21.md`](../40_auditoria_da_pilha_2026-09-21.md);
aqui fica o que a **integração** precisa de saber.

### §24.1 — As duas metades estavam escondidas pela MESMA coisa

⛔⛔ **As réguas que existiam sentavam no ponto neutro**, e é a lição que atravessa esta wave:

| régua | o ponto neutro dela | o que ela não podia ver |
|---|---|---|
| `diag_preco_da_pilha` | traço **RECTO** | a janela do replay a crescer |
| `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` | canvas **OPACO** + Smear no **TOPO** | o esfregão (no fundo ele corre sobre o `pre`) |

⭐⭐⭐ **O CONTROLO POSITIVO que faltava:** calar cada camada uma de cada vez. Sobre a fixtura antiga
o **Smear movia `0` pixels** e `warp.active` lia `false` — *todo A/B da linha comparava duas
corridas de uma camada que não corre*. Numa tela vazia não há o que esfregar; a fixtura honesta
pinta arte ANTES.

### §24.2 — O que MUDA no produto (para quem funde)

* **A rota de omissão da pilha é a ACUMULAÇÃO** ([`composite_acumulado.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/composite_acumulado.rs), ficheiro novo). O
  replay fica como porta de bissecção: **`PH2D_COMPOSITE_REPLAY=1`**, lido UMA vez para o campo
  `paint.pilha_por_replay` — ⚠️ é um CAMPO e não a env, porque *um gate que lê o ambiente mede a
  máquina*.
* **Brush, Erase e Smear saem `|Δ| = 0`** contra o replay. **O Blur diverge por desenho** (pior
  `114` de 255): uma passagem de raio `P·k` em vez de `P` de raio `k`.
* **Zero contrato, zero schema, zero ADR.** ⚠️ A `ph2d-painter-brush` ganha **duas** entradas
  públicas, as duas **append-only**: `blur_region_por_peso` e `kernel_radius` (que era
  `pub(crate)`).
* Campos novos em `PaintState`: `pilha_por_replay` e `acumulando_no_plano`. `PilhaDoTraco` ganha
  `planos: [Arc<Vec<u8>>; N_CAMADAS]`.
* ⚠️ **`stamp_route.rs` tem uma linha nova no caminho quente** — o trinco de alfa passa a perguntar
  `!self.paint.acumulando_no_plano` primeiro. **É território partilhado**; a razão está escrita no
  sítio.

### §24.3 — O censo do núcleo de caixa mudou de endereço, outra vez

O `o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele` passa a esperar **DOIS** ficheiros
(`composite_acumulado.rs` + `composite_pilha.rs`). ⚠️⚠️ **A LEI não mudou duas vezes; o ENDEREÇO
dela mudou duas vezes** — 20/09 o laço saiu do `composite.rs`, 21/09 a pilha ganhou a segunda rota.
Um TERCEIRO sítio continua a ser o pedido a escapar da pilha.

### §24.4 — DUAS premissas do repo morreram, e estão reescritas com a morte à vista

1. O cabeçalho do [`composite_pilha.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/composite_pilha.rs): *«um número que não cresce com o traço»*.
2. A nota do [`state.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/state.rs) sobre o `limite_do_smear`: *«um guarda de RELÓGIO e não de imagem, e a
   mutação que o apaga SOBREVIVE»*.

As duas foram medidas em traço recto sobre canvas opaco. Sobre a fixtura que contém o fenómeno,
apagar o limite **muda a imagem nos dois sentidos** (`39 → 0` no rápido, `48 → 167` no rabisco).

### §24.5 — O que uma leitura rápida do diff entende ao contrário

1. **O `LoteDaPilha` não foi apagado** — ele alimenta a rota de bissecção, e só ela.
2. **A acumulação NÃO é uma aproximação** em três das quatro operações; ela é a lei aplicada
   directamente, e o replay é que era a implementação.
3. **O `escreve_do_pre` continua a ser chamado** — é ele que faz a composição partir da tela do
   pen-down em vez de compor por cima de si mesma (mutação a sangrar).
4. **O `pad_do_nucleo` do replay continua em `k` e está CERTO** — lá cada dab é uma chamada com o
   avental dela. O `pad_do_borrao` da acumulação é que é `k × P`.
5. **O resíduo rectangular não foi a zero, foi a `12` de 255** — e está atribuído (só com Blur e
   Smear juntos, na base do esfregão).
6. **A sonda `diag_auditoria_a_tela_vazia` foi CORTADA e o achado dela ficou** (a divergência sobre
   alfa 0 é invisível hoje, e um Blur ou Smear posterior lê aquele RGB).

### §24.6 — O que fica ABERTO

* ⏳ **O resíduo do par Blur+Smear** (§5.3 da auditoria), com a cura nomeada.
* ⏳ **O blend não-`Mix` de uma camada Brush** é hoje aplicado **uma vez, na composição** — que é a
  lei de CAMADA de um editor. Com `Mix` (o de fábrica) é exacto; nos outros é divergência **não
  medida**, porque nenhuma fixtura desta casa a exercita.
* ⏳ Os três itens da §22.9 continuam abertos (o relevo fora da recomposição, o cartão de cinco
  camadas contra a dobra, e a tabela de relógio da wave anterior).

---

## §25 — A PILHA MONTA-SE À MÃO (ordem do dono, 2026-09-21)

> *«A seção nasce sem nenhuma camada. Teremos um botão + para criar camadas (as possibilidades
> aparecem no dropdown ao lado do +). O máximo que pode ser criado é: 3 de Brush, 2 de erase, 1 de
> Blur, 1 de Smear. As opções vão sumindo do dropdown à medida que vão sendo usadas. Ao usar todas
> inativa-se o dropdown e o botão +. Cada camada passa a ter um x para ser retirada. Todos os
> sliders devem ter nome inclusive o Strength.»*

### §25.1 — O que MUDA, para quem funde

* **`N_CAMADAS` deixou de ser «quantas há» e passou a ser «quantas cabem»** — ele é hoje um alias de
  **`MAX_CAMADAS`**, a **SOMA das quotas** (`3+2+1+1 = 7`), **derivada**. Quantas existem é o
  `composite_len`.
* **Três campos novos** no `PaintState` (`composite_len`, `composite_add_op`) e três no instantâneo
  (`composite_len`, `composite_add_op`, `composite_add_available`).
* **Os arrays de ids do composite foram de `5` para `7`**, e há **ids novos**: `…_REMOVE[7]`,
  `…_ADD`, `…_ADD_KIND`, `…_ADD_OPTION[4]`.
* ⛔ **O `PAINTER_BRUSH_COMPOSITE_OP` e o `set_composite_layer_op` SAÍRAM.**
* ⚠️ **`stamp_route.rs` não foi tocado nesta wave**; a `event/decode.rs` e a `event/option_route.rs`
  ganham uma linha cada (território partilhado do painel).
* **Zero contrato, zero `PROJECT_SCHEMA`, zero ADR, zero pacote novo.**

### §25.2 — ⭐ O MOTOR ficou intocado, e a linha que o consegue é UMA

A cauda das posições que não existem fica a **`strength = 0`**, que é como o laço da pilha já pulava
uma camada. ⇒ o `composite_acumulado`/`composite_pilha` não sabem que esta wave existiu, e
*«quantas camadas existem»* é uma pergunta do ARTISTA e não do carimbo. É também por isso que os
arneses de motor que escrevem `paint.composite[pos]` directamente continuam válidos.

### §25.3 — A QUOTA é uma porta com TRÊS leitores

`quota_da_operacao` é lida (a) pelas opções que o menu mostra, (b) pelo estado do `+`, e (c) pelo
guarda do `acrescenta_camada`. ⚠️ **O painel não recalcula nada**: a disponibilidade viaja
**derivada** no instantâneo (`composite_add_available`).

⭐⭐ **E a derivação compra uma propriedade de COMPILAÇÃO:** subir uma quota sem estender os arrays
de ids deixa de compilar, com um `E0080` a apontar o array exacto — *medido pela prova de mutação,
que ali não sangra porque não chega a compilar, e isso é mais forte*.

### §25.4 — O chip da operação SAIU, e é uma capacidade que sai

Com as quotas, um chip que cicla livremente torna-as mentira — dois cliques punham dois Blurs. A
operação escolhe-se na **criação** e trocá-la é retirar a camada e criar outra. ⚠️ **Declarado e
reversível**: o ciclo limitado à quota é uma alternativa que não foi construída.

### §25.5 — A altura do cartão, e o item da §22.9 que isto FECHA

| camadas | fileiras | altura |
|---|---|---|
| 0 (como nasce) | 1 | **`63` px** |
| 1 | 4 | `138` |
| 5 | 16 | `438` |
| 7 (cheia) | 22 | `588` |

⭐ Antes desta wave o cartão media **`413` px sempre**, com cinco posições que o artista talvez não
quisesse. A §22.9 nomeava *«o cartão de cinco camadas × três fileiras sem medição de quanto empurra
o resto do painel para baixo da dobra»* — **a ordem do dono dissolveu o item**: ele só paga o que
usa.

### §25.6 — O que uma leitura rápida do diff entende ao contrário

1. **`N_CAMADAS` não encolheu nem cresceu de significado por acaso** — ele passou de uma contagem
   para um TECTO, e o nome honesto é `MAX_CAMADAS`; o alias fica porque o motor dimensiona planos
   por ele.
2. **`composite_len = 0` não desliga a pilha** — quem a desliga é a caixa `Composite Brush`. Com ela
   ligada e zero camadas o cartão mostra a fileira do `+`, que é como se enche.
3. **O menu do `+` é um `Dropdown` e não um `Button`** (o despacho genérico abre/fecha; ele não
   emite `Click`).
4. **A escolha do menu não é um `PanelEvent` novo** — ela viaja pela `SelectOption`, o canal de
   texto do contrato congelado.
5. **O `x` não pede confirmação** — uma camada criada à mão tira-se à mão, e o `+` recria.
6. **A camada nova nasce a `strength = 1.0`**, não a zero: *antes de hoje o zero era como uma
   posição se calava; hoje calar-se é não existir*, e uma camada criada que não faz nada lê-se como
   a ferramenta partida.

### §25.7 — O que fica ABERTO

* ⏳ **O Blur continua a governar o preço de todas as camadas** (§5.1 da auditoria: `+29 ms` por
  camada sem Blur, `+115` com um). Com sete posições isso importa mais, e a alavanca é o avental.
* ⏳ **Sete camadas nunca foram medidas no produto real** — a tabela de `+1 Brush`/`+1 Erase` foi
  tirada com a pilha levada a sete à mão, antes desta wave existir.
* ⏳ O `x` e o `+` não têm atalho de teclado nem desfazer próprio (eles entram no undo do painel,
  que **não existe** — decisão antiga do dono).

### §25.8 — O menu do `+` era um BOTÃO disfarçado, e o dono devolveu-o

> *«Ao lado do + deveria ser um dropdown como eu especifiquei. Com todas as opções que ainda podem
> ser colocadas em camadas.»*

⛔⛔ **Ele estava REGISTADO como `Dropdown` e abria a lista — o que faltava era a AFORDÂNCIA.** Eu
pintei-o com o `paint_button`, logo ele tinha o comportamento certo e a cara errada. *Um controlo
que se comporta como um dropdown e se desenha como um botão é um dropdown que ninguém clica.*

⭐ A cura é o pintor CANÓNICO da casa (`paint_dropdown_chip`, o mesmo do Blend e do Falloff), com a
seta e a moldura do tema, e a **largura que sobra na fileira** — um chip estreito ao lado de um `+`
lê-se como um segundo botão. O pintor partilhado ganhou um estado **INATIVO**, que é um parâmetro e
não um segundo pintor; inativo ele **não regista hit rect**, e é isso que o torna inerte.

⭐⭐ **E o gate que nasceu daí apanhou um defeito de ORDEM que ninguém tinha visto:** a passagem
diferida que pinta a lista drenava **ANTES** de o cartão a encher (o cartão vive no CORPO, que
pinta depois dos outros popovers), logo o menu abria sempre com o rect do quadro **ANTERIOR**. Ela
passou para o fim da pintura. ⚠️ *Uma passagem diferida tem de correr depois de quem a ENCHE, e a
ordem de pintura deste painel não é a ordem do ficheiro.*

## §26 — «O BOOLEAN NÃO FUNCIONA COM O COMPOSITE» + «UNDO/REDO DEIXA RESÍDUOS» — um defeito só (2026-09-21)

Dois reports do dono, no mesmo smoke:

> *«OS Stroke:Method vivos (booleanos) — Ellipse, Polygon, Line, etc — não funcionam corretamente
> com o composite, mudam de aparência e o Boolean não funciona»* (com foto: duas circunferências
> `Add` que se cruzam, as duas desenhadas INTEIRAS)
>
> *«undo/redo não estão perfeitos e podem deixar resíduos para os mesmos strokes methods»*

**São a mesma causa.**

### §26.1 — A lei violada, e ela já estava escrita em dois sítios

Um método de **RE-CARIMBO** — os cinco editores de figura mais Drag Dot / Anchored / Line — não
acrescenta tinta: ele **restaura** o recorte do quadro anterior e re-emite a lista INTEIRA de dabs
sobre a tela limpa. O `StrokeMethod::is_incremental` já o diz por escrito, e nomeia o perigo:

> *«the shape editors and the interactive methods re-emit their whole dab list every preview frame,
> which against a fluid / accumulating target would pile paint while the artist just looks»*

A pilha do Composite **é** um alvo que acumula: desde 21/09 cada camada tem um PLANO onde ela
acumula ao longo do traço. E o `stamp_drag_preview` já tinha **três** canais que um re-carimbo repõe,
lado a lado, cada um com o parágrafo dele:

```rust
self.reset_stroke_height();   // o envelope de relevo
self.restamp_reset_sculpt();  // o plano do sculpt
self.restamp_reset_erase();   // a mordida da borracha
```

**A pilha era o QUARTO, e ninguém lhe perguntou.**

### §26.2 — O que se via, medido pelo caminho do produto

⚠️ Uma sessão de figuras é **UM traço** (o pen-up não fecha nada — a figura fica editável até ao
Apply), logo os planos guardavam *toda geometria por onde a mão passou*.

O ciclo de vida, impresso (`diag_o_ciclo_de_vida_da_pilha`, pilha `Blur`/`Brush`):

| passo | tela | `pilha.pre` | planos |
|---|---|---|---|
| `up 1` (a 1.ª figura carimba) | `1 675` | `262 144` (branco) | `[262144, 262144]` |
| `down 2` (o rascunho DESCASCA) | `0` | `262 144` | **`[262144, 262144]`** ← ✗ |
| `up 2` (o boolean carimba a UNIÃO) | **`2 839`** | `262 144` | `[262144, 262144]` |

*A tela voltou a zero e os planos ficaram cheios* — a composição repõe a 1.ª figura INTEIRA por cima
do contorno da união, e os `448` texels a mais **são** os arcos interiores.

**O arco que o boolean tem de apagar** (o branco é `255`, duas circunferências `Add` que se cruzam):

| pilha | tinta | o arco interior |
|---|---|---|
| sem pilha | `2 391` | `255` ✓ |
| 1 camada | `2 391` | `255` ✓ (com menos de duas camadas activas a pilha nem abre) |
| `Blur`/`Brush` | `2 839` | **`26`** ✗ |
| `Smear`/`Brush` | `672` | **`0`** ✗ |
| `Brush`/`Brush` | `2 839` | **`63`** ✗ |
| `Erase`/`Brush` | `2 839` | **`0`** ✗ |

**O RASTO** (arrastar UMA figura 70 px, a tela tem de mostrar UMA figura):

| pilha | uma figura | depois de arrastar | excesso |
|---|---|---|---|
| sem pilha | `1 496` | `1 496` | `+0 %` |
| `Blur`/`Brush` · `Brush`/`Brush` · `Erase`/`Brush` | `1 496` | `1 822` | **`+22 %`** |
| `Smear`/`Brush` | `481` | `804` | **`+67 %`** |
| `Blur`/`Smear`/`Brush` | `1 110` | `1 584` | **`+43 %`** |

⇒ *o «resíduo» do 2.º report é este rasto, e o «boolean não funciona» do 1.º é o mesmo plano por
descascar visto noutro gesto.*

### §26.3 — A cura: uma porta, um sítio, e a SEGUNDA metade que o Smear exigiu

[`PainterTool::restamp_reset_pilha`](../../../crates/ph2d-tool-painter/src/tool/paint/composite_acumulado.rs),
chamada do `stamp_drag_preview` **ao lado das outras três**:

1. os **planos** de cada camada voltam a vazio;
2. os **lotes** da rota de bissecção (`PH2D_COMPOSITE_REPLAY=1`) também — *uma cura que só tratasse
   a rota de omissão deixaria a bissecção a medir outro programa*;
3. o acumulador de **ARCO** da subamostragem de uma camada MAIOR;
4. ⭐ e o **campo de deslocamento do Smear** (`paint.warp`), que é por TRAÇO como os planos e vive
   **fora** deles — pela porta que o pen-up já usa (`end_smear_session`, que exclui o Deform por
   construção). Sem esta metade o rasto do `Smear` ficava em `+10 %`.

⚠️ **O `pre` FICA, e isso é lei e não economia:** o descascar devolve a tela a exactamente o que ele
guarda. Re-fotografá-la por evento custaria uma cópia do canvas inteiro por movimento do rato
(67 MB a 4096²) para escrever os mesmos bytes.

**Depois:** o arco interior lê `255` nas SEIS pilhas, o rasto lê **`+0 %`** nas seis, e a tinta do
boolean volta a `2 391` — **o mesmo número que sem pilha nenhuma**.

### §26.4 — Três coisas que a medição me corrigiu

⛔ **(a) A 1.ª redacção pôs a cura na porta ERRADA.** Eu pu-la dentro do `peel_drag_preview`, que é
a porta de descascar — e ela tem um chamador que **não é um re-carimbo**: o bracket do
`Style: Solid` no `stamp_dabs`, que descasca a cada LOTE de um traço à mão livre **cumulativo**.
Ali esquecer o acumulado é o oposto do certo. ⚠️ **E a medição corrigiu-me duas vezes:** eu li a
camada `Smear` inerte sob `Solid` como uma regressão minha, e ela é **PRÉ-EXISTENTE** — com a
chamada lá e sem ela a saída é byte-idêntica. A chamada saiu porque é **redundante** (a mutação que
só a apaga sobrevive aos gates) *e* porque a porta é partilhada, não porque estivesse a partir nada.

⛔⛔ **(b) O cap de Accumulate JÁ era limpo, e eu ia escrever a segunda resposta.** A 1.ª redacção
limpava o `composite_mask`, e a mutação que a apagava **sobreviveu**: o
`stamp_cache::prepare_stroke_mask` já o zera para todo método que não é incremental, com esta lei no
doc dele desde que existe — *«os métodos de preenchimento re-carimbam o TRAÇO INTEIRO, logo o cap
tem de começar FRESCO»* — e o `composite_mask[pos]` está TROCADO para dentro do `stroke_mask` quando
ele corre. *Duas respostas à mesma pergunta divergem no dia em que uma mudar de chave*, e a minha
era indexada pelo DESCASCAR contra uma indexada pelo MÉTODO. ⇒ **linha apagada.**

⭐ **(c) O ARCO, esse, era dívida REAL e ninguém mais o repõe.** Com duas camadas `Brush` de
`size = 3`, a figura re-carimbada pinta **`740`** texels contra `5 641` — o `d.arc_len` recomeça do
zero a cada re-carimbo e o acumulador do carimbo anterior recusa a lista inteira. Gate próprio, e
⚠️ **`size = 3` é ingrediente da fixtura**: com `size = 1` a camada não subamostra nada e o
acumulador nunca é escrito (*uma fixtura no ponto neutro do knob não testa o knob*).

### §26.5 — Os gates, e a prova de mutação

[`composite_formas_tests.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/composite_formas_tests.rs),
todos pela porta do produto (pen-down / move / pen-up):

- `o_boolean_apaga_o_arco_interior_com_a_pilha_montada` — ⚠️ com **CONTROLO**: as mesmas duas
  figuras em `Overlay` têm aquele ponto **pintado**; sem ele, um gate que só exigisse branco passaria
  numa fixtura em que as figuras nem se cruzam.
- `arrastar_uma_figura_nao_deixa_rasto` — a `Smear`/`Brush` é o membro que exige a 2.ª metade.
- `uma_camada_maior_sobrevive_ao_recarimbo` — ⚠️ a barra sai de um **vale medido** (`99,5 %` contra
  `13 %`, um vale de `7,6×`) e não de uma folga escolhida; a igualdade exacta não vale porque a
  lista de dabs de uma figura é re-derivada a cada carimbo (`5 627` de `5 658`).
- `a_rota_de_bisseccao_tambem_descasca_a_historia` — o `PH2D_COMPOSITE_REPLAY`, por CAMPO e nunca
  pela variável de ambiente.

**Mutação: 5 sangram, 1 NOMEADA.**

| # | mutação | veredito |
|---|---|---|
| M2 | `stamp_drag_preview` sem o `restamp_reset_pilha()` | SANGRA (3 de 3) |
| M3 | sem o `end_smear_session()` | SANGRA |
| M4 | sem limpar os planos | SANGRA (2 de 3) |
| M6 | sem limpar os `lotes` | SANGRA (a rota de bissecção) |
| M7 | sem repor o `composite_arco` | SANGRA |
| M8 | sem a saída antecipada do `pre` vazio | **NOMEADA** — ela é guarda de CUSTO e não lei (sem ela um pincel comum paga os `clear()` por evento para não mudar um bit) |

⚠️ **E o arnês mentiu primeiro, na forma que este repo já tem escrita:** `grep -cF` conta **LINHAS**,
logo uma agulha multi-linha casou `4` e `3` vezes e o arnês abortou dois casos sobre produto
correcto. Ele passou a contar ocorrências.

### §26.6 — O que fica ABERTO, e não é da pilha

- ⚠️ **A camada `Smear` é inerte sob `Style: Solid`** — medido: a impressão da tela é, ao bit, a de
  não haver pilha nenhuma. **PRÉ-EXISTENTE** (não muda com esta wave, em nenhum dos dois sentidos),
  sem causa investigada.
- ⚠️ **Dois `Ctrl+Z` a meio de uma sessão de duas figuras esvaziam a tela** em vez de deixarem a 1.ª
  — e **o mesmo acontece SEM pilha** (`sem pilha` lê `0` exactamente como as seis pilhas). É do undo
  dos editores de figura, não do composite, e é o que sobra do 2.º report depois desta cura.
- ⚠️ Com uma camada `Smear` na pilha, desenhar uma figura **por cima de arte já aplicada** esborrata
  essa arte. É o que um esfregão faz — nomeado aqui porque a sonda o lê como «perdeu a 1.ª figura».

---

## §27 — «2 CÍRCULOS COM O MESMO PINCEL E UM ESTÁ DIFERENTE DO OUTRO» (2026-09-21)

> Report do dono, a seguir ao da §26: *«veja : 2 círculos com o mesmo pincel e um está diferente do
> outro. Boolean funcionou! Performance ruim»*. São **duas** metades — a aparência e o relógio —, e
> esta secção é a primeira. A segunda é a §28.

### §27.1 — A causa: a corrente do esfregão atravessava a fronteira de uma sub-figura

Uma sessão de figuras é **UM traço** (o pen-up não a fecha — a figura fica editável até ao Apply), e
um lote do `restamp_shapes_preview` é a **CONCATENAÇÃO** das listas de dabs de todas as figuras
vivas: a activa, mais cada parqueada, mais um contorno por região no boolean.

O acumulador do esfregão (`last_smear_pos`) é **por TRAÇO**. Sem uma fronteira, *o último dab de um
círculo levantava tinta para o primeiro dab do círculo seguinte, **através da tela***.

**Medido** na janela que contém só a 1.ª figura (soma do alfa), com a 2.ª **longe** dela:

| pilha | sozinha | com a 2.ª | |
|---|---|---|---|
| `Brush` | `551 889` | `551 889` | IGUAL |
| `Blur/Brush` | `445 418` | `445 418` | IGUAL |
| **`Smear/Brush`** | **`522 145`** | **`448 091`** | **`−14,2 %`** (`n` `2 859 → 2 568`) |
| `Brush/Brush` | `607 839` | `607 839` | IGUAL |
| `Erase/Brush` | `316 829` | `316 829` | IGUAL |
| **`Blur/Smear/Brush`** | **`422 835`** | **`361 950`** | **`−14,4 %`** |

Depois da cura: **as seis IGUAIS**, byte a byte.

### §27.2 — A fronteira é DERIVADA, e não um campo novo

Todo `fill_*_preview` recomeça o `arc_len` em zero (`stroke/ellipse.rs`: *«fresh fill → the Flow
along-coordinate starts at the perimeter's origin»*) ⇒ **um arco que ANDA PARA TRÁS é uma sub-figura
nova**. O `last_smear_pos` passa a ser o par `([f32; 2], f32)` — o centro **e** o arco —, e o par é
deliberado: *dois campos que têm de concordar são um campo que alguém esquece de escrever*.

⛔ **Um limiar sobre o COMPRIMENTO do salto foi recusado por mecanismo:** um traço à mão livre
rápido produz saltos legítimos do mesmo tamanho.

### §27.3 — O gate, e porque ele mede as SEIS pilhas

`a_primeira_figura_nao_muda_quando_nasce_a_segunda` afirma sobre **todas** as pilhas, não só as do
esfregão: *a lei é de todo acumulador por-traço, e hoje só ele atravessa*. O gate reprova no dia em
que outro o faça.

- ⚠️ **A tela da fixtura nasce VAZIA de propósito.** Numa tela branca opaca o esfregão move branco
  para dentro de branco e **a fixtura não contém o fenómeno** — a `cena()` daquele ficheiro, que é
  opaca, não serve, e a régua é o **alfa**.
- **Três controlos**, e cada um fecha uma maneira de o gate passar por vácuo: a 1.ª figura pintou ·
  a 2.ª de facto nasceu · as duas não se tocam (senão «idêntico» seria a expectativa errada).

**Mutação 3 de 3 a sangrar:** a corrente não parte · o arco escrito a zero (que prova que o PAR
viaja junto) · a comparação invertida (contra a suíte inteira: 9 vermelhos).

⚠️ **O arnês mentiu primeiro:** `grep -cF` conta **LINHAS**, logo uma agulha multi-linha abortou
dois casos sobre produto correcto.

---

## §28 — «PERFORMANCE RUIM»: a atribuição, DUAS recusas medidas, e a cura (2026-09-21)

### §28.1 — Onde o custo está

`--release`, `91`–`96 %` de CPU ociosa, por evento de carimbo, arrastando uma figura:

| `2048²` | CARIMBAR | pre | acumular | **COMPOR** | cópias | alvo |
|---|---|---|---|---|---|---|
| sem pilha | `1,29` | — | — | — | — | — |
| `Brush/Brush` | `9,48` | `0,05` | `3,22` | **`5,60`** | `1,55` | `50,6 %` |
| `Blur/Brush` | `51,86` | `0,06` | `3,24` | **`49,31`** | `1,97` | `54,1 %` |
| 7 camadas | `84,74` | `0,04` | `9,44` | **`79,92`** | `1,68` | `54,1 %` |

⇒ **COMPOR é `90`–`99 %` do carimbo, e UMA camada `Blur` custa `8×` uma `Brush`** (`43,7` dos
`49,31 ms`). ⚠️ E o re-carimbo corre **a cada movimento do ponteiro** enquanto se arrasta a figura,
não uma vez por traço: `51,86 ms` são `19` fps.

### §28.2 — Não é o núcleo nem o avental

Varrido o **tamanho da camada** (que escala `k` linearmente e mais nada), `k` anda `14×`:

| tamanho | `0,25` | `0,5` | `1` | `2` | `4` |
|---|---|---|---|---|---|
| `k` | `8` | `16` | `24` | `56` | `112` |
| COMPOR | `42,69` | `40,38` | `40,86` | `46,53` | `57,08` |
| alvo | `51,9 %` | `53,0` | `54,1` | `60,4` | `72,4` |
| **por tela cheia** | **`82,3`** | **`76,2`** | **`75,5`** | **`77,0`** | **`78,8`** |

**Plano a `±4 %` sobre `14×` de `k`** ⇒ o custo é `O(área)` puro (`31,1 ns/px`). A caixa é `O(1)` em
`k` por construção, e o avental (`(bw+2k)(bh+2k)`) não aparece.

### §28.3 — ⛔ RECUSA MEDIDA: a alocação está ILIBADA

A cadeia aloca **sete** `Vec<[f32; 4]>` do tamanho da região — `~250 MB` por composição a `2048²` —
e isso custa **`0,00 ms`**: `vec![[f32; 4]; n]` é `alloc_zeroed`, as páginas chegam preguiçosas e
quem as toca é a passagem. *Contar bytes alocados não é medir o custo de os alocar.*

### §28.4 — ⛔⛔ RECUSA MEDIDA: o cover por BLOCOS, a família inteira

Um anel ocupa `~5 %` da caixa envolvente dele, mas cada bloco paga o **avental**. Medido sobre a
lista de dabs REAL (`1,00×` = a caixa envolvente):

| | `16 px` | `32` | `64` | `128` | `256` |
|---|---|---|---|---|---|
| `1024²` | `6,66×` | `3,16×` | `1,99×` | `1,62×` | `2,39×` |
| `2048²` | `3,63×` | `1,68×` | `1,01×` | **`0,89×`** | `0,99×` |

A `1024²` **todo** tamanho é pior que a caixa; a `2048²` o melhor poupa `11 %`. Com `pad = 25` um
bloco de `16` compõe `66²` para cobrir `16²`. ⇒ *não é um tamanho que está errado, é a forma do
cover* — e uma que siga o anel é redesenho, não afinação.

### §28.5 — A cura, e o tecto dela

O borrão move `~224 B/px` em `31,1 ns/px` = **`6,8 GB/s`**, que é da ordem de UM núcleo. O soquete:

| fios | 1 | 2 | 4 | 8 | 16 |
|---|---|---|---|---|---|
| débito | `6,8` | `12,0` | `20,2` | `27,4` | `30,1 GB/s` |
| ganho | `1,00×` | `1,78×` | `2,99×` | **`4,05×`** | `4,46×` |

⇒ [**ADR-0171**](../../architecture/decisions/0171-o-borrao-de-caixa-da-pilha-parte-se-em-fatias-e-a-largura-da-banda-e-medida.md)
— o núcleo de caixa parte-se em fatias. ⚠️ **O número do ADR SOMA entre linhas: reconte-o.**

**Resultado, pela porta do produto** (`93 %` ocioso, a MESMA corrida de antes e depois):

| | antes | depois | |
|---|---|---|---|
| `2048²` `Blur/Brush` | `51,86` | **`29,09`** | `1,78×` |
| `2048²` bool `Blur/Brush` | `78,48` | **`47,76`** | `1,64×` |
| `2048²` 7 camadas | `84,74` | **`63,98`** | `1,32×` |
| `1024²` `Blur/Brush` | `8,87` | `8,11` | `1,09×` |

**Nenhuma célula piorou.**

⚠️ **O ganho é `2,09×` na região do produto e NÃO os `4,05×` do tecto**, e a diferença está medida:
a horizontal escala `3,6×`, a vertical `2,25×`, mais sete barreiras de junção. *A sonda do soquete
mede `N` borrões INDEPENDENTES; partir UM não é a mesma coisa.*

### §28.6 — ⛔⛔ O número de fatias da VERTICAL não é o número de threads

Em série a vertical percorre cada linha INTEIRA e é um fluxo sequencial; partida em `nb` bandas de
colunas vira `nb` fluxos com passo `w`:

| fatias | 1 | 2 | 3 | **4** | 6 | 8 | 12 | 16 |
|---|---|---|---|---|---|---|---|---|
| largura | `1 484` | `742` | `494` | **`371`** | `247` | `185` | `123` | `92` |
| ganho | `1,00×` | `1,58×` | `2,00×` | **`2,25×`** | `2,12×` | `1,66×` | `1,39×` | `1,14×` |

⇒ fixa-se a **LARGURA** (`384`, o meio do planalto) e a **CONTAGEM sai dela**, com tecto na pool.
⚠️ *Isto não contradiz «não fixar o número de pedaços»: a contagem continua a seguir a máquina pelo
tecto, e o que a medição acrescenta é um **PISO que vem do acesso à memória**, não do escalonador.*

### §28.7 — As cinco coisas que uma leitura rápida do diff entende ao contrário

1. **A paridade das duas rotas não prova a lei da passagem.** As duas partilham o corpo da banda ⇒
   uma mutação DENTRO dele move os dois lados por igual. *Um oráculo de IGUALDADE só afirma sobre o
   que as duas rotas fazem DIFERENTE.* Quem julga os valores é a **convolução directa**.
2. **O gate do impulso não cobre a semente.** A região é centrada no impulso, logo as linhas
   `0..lado` não contêm sinal e uma mutação na semente é invisível ali.
3. **As fixturas da paridade nunca correram o código que parte as bandas** (`59`–`291` px, todas
   abaixo do piso de `384`) ⇒ a contagem é um **argumento** no gate das bandas, não o que a região
   e a máquina decidem.
4. **`uma_banda_nunca_fica_abaixo_do_piso` não afirma um número.** A contagem é limitada pela pool,
   logo `bandas_da_vertical(1484) == 4` seria uma propriedade da MÁQUINA; o que é da lei é *a banda
   que sai tem pelo menos a largura do piso*.
5. **`a_decisao_de_partir_segue_o_joelho_medido` existe porque a rota é byte-idêntica.** Sem ele,
   apagar o paralelo do produto não acorda nada — as outras réguas dizem que as rotas concordam, e
   desligar uma mantém-nas a concordar.

### §28.8 — E o censo do alcance do núcleo de caixa foi CURADO, com dois defeitos MUDOS meus

Ele acusou uma **sonda `#[cfg(test)]`** de ser um terceiro sítio. ⛔ A cura não é filtrar pelo
SUFIXO do nome — este repo já pagou um portão vermelho sobre produto correcto assim
(`line/sculpt3d`, 15/09) —: a população passa a ser **DERIVADA** das declarações `mod`, com a
sub-árvore de um `#[cfg(test)]` de fora **transitivamente**.

Dois defeitos na derivação, os dois silenciosos:

- **`mod x; // comentário` era deitado fora** (eu exigia que a linha ACABASSE em `;`) — `71` dos
  `159` módulos de uma sub-árvore desapareciam. ⭐ *Só o apanhei por ter uma SEGUNDA implementação
  da mesma regra e as duas discordarem* (`135` contra `209`).
- **`#[path]` resolve contra a pasta do ficheiro que DECLARA**, e eu juntava-o à pasta-filha ⇒ a
  sub-árvore inteira sumia. ⭐ Apanhado por uma **mutação que SOBREVIVEU**: ignorar o `#[cfg(test)]`
  não mudava nada, *porque aqueles ficheiros não estavam a ser alcançados de todo*.

⚠️ **E os quatro guardas não bastavam:** eles não distinguem *«excluído por ser teste»* de
*«excluído porque o caminhar não o achou»* ⇒ o quinto nomeia dois ficheiros que só se alcançam por
`#[path]` de PRODUTO (`undo_window.rs`, `curve_overlay.rs`).

### §28.9 — O portão

`nextest-impacted` **18 255/18 255** · censos da árvore COMBINADA **127/127** · clippy zero ·
fmt · machete · standalone · workflow · doc-index · adr-index verdes. **Mutação 12 de 12 a
sangrar.** Dois tectos de LOC curados por **CORTE**, nenhum por isenção:

- `blur_caixa.rs` `1 028 → 407 + 632` — os testes entram por `#[path]` **de dentro** do módulo, para
  que continuem DESCENDENTES dele e nenhum `fn` privado tenha de alargar visibilidade só para
  hospedar um teste. *Mover um ficheiro troca o REGIME DE VISIBILIDADE que o governa, e num `lib`
  `pub(crate)` é API.*
- `diag_composite_e_as_formas.rs` `933 → 604 + 348`, por ASSUNTO: o que o re-carimbo **DESENHA**
  contra o que ele **CUSTA**.

### §28.10 — ⏳ ABERTO, com o mecanismo

- **Metade dos BYTES vale o mesmo que dois fios, sem política de concorrência nenhuma.** Os sete
  intermédios são `[f32; 4]` = `16 B/px` e o borrão é limitado por LARGURA DE BANDA; um intermédio
  de `u16` premultiplicado é `2×`. ⛔ Ele **não** é byte-idêntico, logo pede a mesma barra de
  qualidade que a caixa já tem contra o binomial (pior byte `1`–`3`) — wave própria.
- **`8` rondas de fatias não compram nada** (o soquete satura em `4,46×` a 16 fios contra `4,05×` a
  8): quem quiser o degrau seguinte ataca os BYTES, não os fios.
- Os dois itens da §26.6 continuam abertos e **não são da pilha** (a camada `Smear` inerte sob
  `Style: Solid`; dois `Ctrl+Z` a meio de uma sessão de figuras, que faz o mesmo SEM pilha).

---

## §29 — «ARTEFATOS DE IMAGEM NAS MARGENS RETANGULARES» — DOIS mecanismos, uma pergunta (2026-09-21)

**Sete fotos do dono ao longo do dia**, do fantasma cinzento nas margens ao **rectângulo branco
opaco** a cobrir arte pintada — *e ele aparece numa sprite TRANSPARENTE*, onde o «branco» não podia
vir do papel. ✅ **Smoke do dono aprovado:** *«Ficou tudo curado e sem nenhum retângulo nem
artefato»* e, depois da 2.ª cura, *«Perfeito! Tudo corrigido!»*.

⭐⭐⭐ **Eram DOIS mecanismos sem nada em comum, e os dois respondem mal à MESMA pergunta: «que
região é que este gesto mexe?»** O post-mortem completo — com as **quatro réguas erradas** e a
**atribuição falsa** que eu reportei ao dono — é o **[Bug #24](../BUGS_painter.md)**; aqui fica o
que o INTEGRADOR precisa: a superfície tocada.

### §29.1 — A caixa SALVA é a do PINCEL, a ESCRITA é a da CAMADA

Um método de re-carimbo **restaura** o recorte do quadro anterior e re-emite a figura. A região
guardada era `∪ dab_bbox(centro, radius_px)` — o raio do **PINCEL** —, e a pilha escreve
`caixa_das_camadas`, onde o `camada_dabs` faz `radius_px *= escala` **por camada** (na pilha do dono
a escala é `1,904`). ⇒ o anel entre as duas caixas **nunca era restaurado**: a crescer o quadro
seguinte tapa-o, a encolher ele fica à vista — *que é a dica do dono à letra*.

| ablação (Anchored `40→200→40` contra `40` directo) | texels de rasto |
|---|---|
| nenhuma (o que shipava) | **`290 534`** |
| sem composite · todos os tamanhos a `1,0` · calar a camada de `size 1,904` | `0` |
| calar qualquer outra camada | `290 534` |

O resíduo acabava em **raio `380` = `200 × 1,904`**. Depois da cura: `0` em todas as células.

**Cura:** `region::caixa_do_lote` + `escala_do_carimbo` (`region.rs` `+53`), lida pelo descasque em
`stamp_preview.rs`. Gate `um_recarimbo_que_encolhe_nao_deixa_rasto`, com o interruptor
`region::CAIXA_DO_PINCEL` (**campo `#[cfg(test)]`, nunca env** — *um gate que lê o ambiente mede a
máquina*) como CONTROLO.

### §29.2 — FIXAR (Enter) não fechava a pilha

O pen-up de uma figura **não** fecha o traço; o **Enter** (`commit_open_shape`) assa os pixels e
larga os editores, e o `pre` da pilha continuava a ser a tela de **antes** delas ⇒ a figura seguinte
reconstrói-se dessa base velha e, na região dela, **apaga o que acabou de ser fixado**.

| caso (tela preta transparente, pilha do dono, duas elipses `r = 110`) | arte fixada destruída | calando o Smear |
|---|---|---|
| sem Enter entre as duas | `0` | — |
| **com Enter, SEM a cura** | **`12 530`** (pior `127`, caixa `119×280`) | `12 530` |
| **com Enter, com a cura** | `3 083` | **`0`** |

⭐⭐ **A atribuição é DISJUNTA, e é ela que fecha o assunto:** sem a cura o esfregão **não é a
causa**; com a cura o que sobra é **só** ele.

**Cura:** `composite_reposicoes::commit_reset_pilha` — o **quarto canal** da lista do
`commit_drag_preview`, ao lado do relevo do traço, da sessão do escultor e da borracha. Gate
`fixar_fecha_a_pilha` (⚠️ a pilha mede-se **logo a seguir** ao Enter: a 2.ª figura reabre-a).

### §29.3 — A SUPERFÍCIE DE MERGE (o que o integrador precisa de saber)

| ficheiro | antes → depois | porquê |
|---|---|---|
| `paint/region.rs` | `249 → 302` | as duas portas novas |
| `paint/stamp_preview.rs` | `−/+19` | o descasque lê a caixa do lote; `commit_drag_preview` ganha o 4.º canal |
| `paint/composite_acumulado.rs` | **`676 → 600`** | **CORTE por responsabilidade** (o tecto é `700` e a porta nova cruzava-o) |
| `paint/composite_reposicoes.rs` | **NOVO, `155`** | as DUAS reposições da pilha |
| `paint/diag_o_resto_do_descasque.rs` | **NOVO, `596`** | a sonda + o gate do descasque |
| `paint/diag_o_enter_e_a_pilha.rs` | **NOVO, `343`** | a sonda + o gate do Enter |

⚠️⚠️ **O corte tem uma razão que não é o tecto, e ela é a que interessa a quem funde:** as duas
reposições (`restamp_reset_pilha` · `commit_reset_pilha`) são **OPOSTAS no `pre`** — descascar
MANTÉM-NO, fixar tem de o MATAR —, e escritas longe uma da outra a segunda nasce como cópia da
primeira e leva o `pre` que não devia levar. *Elas vivem lado a lado de propósito; separá-las outra
vez reabre o defeito.*

⚠️ **Zero `PROJECT_SCHEMA`, zero contrato, zero registo, zero pacote novo, zero linha de shell.**

### §29.4 — As armadilhas que esta wave pagou (as de MÉTODO estão no Bug #24)

* ⛔ **`cargo check` sem `--all-targets` foi CEGO ao corte:** mover a porta deixou o
  `COMMIT_SEM_FECHAR` fora de âmbito e o check simples passou **verde**; o `E0425` só apareceu com
  `--all-targets`. A armadilha que o `CLAUDE.md` §2 escreve por extenso.
* ⛔ **O gate do Enter, na 1.ª redacção, media a pilha no FIM do gesto** — a 2.ª figura reabre-a, e
  ele lia-a aberta sobre uma cura correcta. *Uma régua lida depois do gesto seguinte não afirma nada
  sobre o commit.*
* ⚠️ **A numeração colidiu:** a entrada nasceu `#16` e o `#16` já existia (aquarela, 20/07) ⇒ é
  **`#24`**. *Um número que soma numa lista conta-se, nunca se escolhe.*
* ⚠️ **E dois números da mensagem do `bf612ece1` estão ERRADOS** (ela diz `734 → 600` e `912 → 596`):
  os committados são **`676 → 600`** e **`709 → 596`**. Os `734`/`912` eram picos **transitórios** da
  árvore de trabalho, não estados gravados — *um número lido do editor e não do `git show` descreve
  um ficheiro que nunca existiu*. A tabela do §29.3 é a que vale.

⏳ **ABERTO e ATRIBUÍDO:** os `3 083` que sobram são o resíduo da **base congelada do esfregão**
(§5.3 da [auditoria](../40_auditoria_da_pilha_2026-09-21.md)) — ela é refrescada só dentro da região
recomposta enquanto o render lê `p − disp(p)`, que pode cair fora dela. *Outro mecanismo, com a cura
endereçada lá.*

---

## §30 — ⏳ A FILA DA PRÓXIMA JANELA (ordem do dono, 2026-09-21, fim do dia)

⛔⛔ **Isto NÃO é dívida de integração.** Nada aqui bloqueia a fusão desta linha nem descreve
código escrito — são **duas ordens do dono guardadas para amanhã**, e o sítio é este porque o
handoff é o que a próxima janela desta linha lê por protocolo. *O integrador salta esta secção.*

### (1) O slider `Spacing` sobe para debaixo do dropdown `Method`

> *«O slider spacing nos strokes vivos (como freehand, elipse, line, etc) deve ser posicionado logo
> abaixo do dropdown Method.»*

Assunto de **disposição** na secção Stroke ([`paint_stroke.rs`](../../../crates/ph2d-panel-painter-layers/src/paint_stroke.rs),
que é onde o `Method` e o `Spacing` vivem os dois).

⚠️ **Antes da 1.ª linha, MEÇA o que a subida empurra para baixo.** Esta casa já pagou isto duas
vezes este mês: uma fileira que muda de sítio custa altura a **todos** os pincéis, e a catraca da
DOBRA do painel foi o que apanhou o defeito das duas vezes — uma delas transformou um preço de
disposição num diagnóstico de LEI. *A pergunta não é «cabe?», é «o que é que isto atira para
debaixo da dobra?».*

### (2) O composite tem de OBEDECER à lei do re-carimbo: apagar → actualizar → voltar a mostrar

> *«Nos strokes vivos, o ajuste de propriedades no painel deve apagar o traço, atualizar e voltar a
> mostrar. Isso já acontece com todos os ajustes, menos o composite que deve ser atualizado para
> obedecer essa regra.»*

⭐ **Ele nomeou a lei e nomeou o único infractor** — logo o trabalho é **alinhar**, não desenhar
nada novo: a rota do descasque já faz exactamente isso para os outros ajustes, e a pergunta é
*porque é que uma edição na pilha não passa por ela*.

⚠️ **E esta linha acabou de pagar o vizinho deste defeito duas vezes**, o que dá dois avisos
concretos a quem pegar nisto:

* o descasque tem de repor **tudo** o que é por-traço (§26: os planos, os lotes, o acumulador de
  ARCO e o campo do esfregão — foi o **quarto canal** de uma lista que já existia e onde a pilha
  não estava), e o **§29.2** acrescentou o quinto para o Enter. *Se a cura for «mais um canal»,
  conte os canais primeiro.*
* ⛔ e a caixa que ele repõe é a do **LOTE**, nunca a do raio do pincel (§29.1) — *repor a região
  errada é o rectângulo que o dono acabou de ver sete vezes.*

⏳ **Nenhum dos dois foi medido, reproduzido ou planeado** — ficam como ele os disse.

---

## §31 — A AUDITORIA DO WATERCOLOR, E AS DUAS CURAS (2026-09-22)

> Ordem do dono: *«agora que sabemos como resolver vários tipos de artefatos no painter, quero uma
> auditoria do modo watercolor buscando por problemas similares»*, e a seguir *«vamos curar as
> coisas»*. Auditoria inteira (as duas lentes, as hipóteses refutadas, as tabelas):
> [doc 42](../42_auditoria_do_watercolor_2026-09-22.md).

### §31.1 — O RESERVATÓRIO DO MIXER RENASCE COM A COBERTURA (P0)

**Superfície:** `watercolor_accum.rs` (**uma linha** de produto) + gates novos
(`watercolor_recarimbo_tests.rs`) + a sonda (`diag_o_resto_do_watercolor.rs`, `#[ignore]`).
⛔ **Zero schema, zero contrato, zero shell, zero pacote novo, zero contador partilhado.**

⭐ **A lei:** *quem reconstrói o LOTE INTEIRO a cada quadro reconstrói também o RESERVATÓRIO.* O
`clear_wet_coverage` tem **dois** chamadores — o re-carimbo de FIGURA
(`stamp_drag_preview_watercolor`) e o `Anchored`/`Drag Dot`/`Line` do TRAÇO (`stamp_stroke_dabs`) —
e o `wet_mix` (a carga do pincel, que se **esgota** com o depósito) sobrevivia aos dois. ⇒ a mesma
figura, com os mesmos números, saía diferente conforme quantas vezes lhe tinham tocado.

⚠️ **Para quem funde:** a cura é `self.reset_wet_mix();` no fim do `clear_wet_coverage`, colada ao
bloco que já repõe a reserva com a frase *«A reserva recomeça com a cobertura»*. **A porta já
existia** (`reset_wet_mix`, com UM chamador: o `open_stroke`) — *o caminho da figura simplesmente
nunca a invocava*.

**Medido** (elipse parqueada de 160 px, pincel 200 px, `Charge = 0.5`): a deriva cresce
`15 → 22 → 38 → 64` bytes em `n = 2, 3, 5, 10` e **satura em `64/255`** sobre a figura inteira; o
pixel do CENTRO não muda, logo o que deriva é o **gradiente de densidade**. No `Anchored`,
`n = 2 → 61 010` texels. Com `Charge = 1` (o valor de fábrica) é **no-op byte-idêntico**.

⛔⛔ **As TRÊS hipóteses com que a auditoria abriu estão REFUTADAS, e isso importa para quem vier
a seguir:** o descasque do watercolor **é exacto** (`resto = 0` em três regimes), as **máximas da
sessão** não deixam resto (baixar o `edge_spread` de `40` para `7` lê o mesmo número do controlo
`7 → 7`), o **`wet_cum_dirty` É drenado** pelo próprio `clear_wet_coverage`, e o **plano da reserva
está ilibado por ablação**. *A caixa-contra-janela do Bug #24 não se repete aqui.*

**Gates, red-first:** 3 reprovavam e os **2 CONTROLOS** (mixer desarmado) já passavam — é isso que
prova que eles medem o mixer e não uma propriedade trivial do re-carimbo. **Mutação 2 a sangrar +
controlo:** `if false` à volta da chamada faz **as duas leis sangrarem** com a fiação textual verde;
mudar a reposição da porta para o chamador da figura **cura a figura e faz o Anchored sangrar** ⇒ a
escolha da porta é load-bearing.

### §31.2 — O ID ÓRFÃO DO PIGMENT, E O CENSO DA TERCEIRA DIRECÇÃO (P3)

`PAINTER_WATERCOLOR_PIGMENT` era o **único dos 38** ids do módulo que o painel nunca citava — nem no
`CLICKS` nem no `FIELDS`, logo **não registado e não focável** — e mesmo assim com um braço de
despacho vivo. O *back-compat* que o doc dele invocava tinha **zero** consumidores medidos.
⛔ E ele não era só morto: virava a bandeira `pigment` **sem tocar no `pigment_mix`**, enquanto o
`set_brush_pigment_mixing` mantém o par coerente — *a segunda resposta ao mesmo facto, e a morta era
a errada*. Saiu o trio (id · braço · setter); **o campo `brush.pigment` FICA** (vivo, lido pelo
`spec/queries.rs`).

⭐ **O censo novo mede a direcção que nenhum outro deste repo mede.** Os que existem perguntam *«o
que é PINTADO está registado?»* e *«o que é registado é ALCANÇÁVEL?»*; faltava ***«o que é
DESPACHADO chega a ser pintado?»***. ⚠️ A isenção do `PAINTER_SHAPE_*` é **medida** (pintado,
populado e roteado pela secção de FORMA) e há **piso de população nos DOIS lados** da extracção.

### §31.3 — ⚠️ PARA O INTEGRADOR: uma flake de carga NOVA para promover

`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
([`ph2d-tool-painter/src/tool/paint/measure_input_cost.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs))
reprovou **1 de 18 272** no `nextest-impacted` e passa **3 de 3 sozinho a 93–94 % de CPU OCIOSA**. As
três assinaturas: ele é uma **RAZÃO de dois relógios de parede** (`pen-down / cópia do canvas`,
barra `> 0,5`), **zero linhas do diff** naquele ficheiro, e o `tool(SIDE)` dele **nem arma
watercolor** ⇒ a cura desta jornada não corre naquele caminho. **Pedido de promoção à lista do
`CLAUDE.md` §5.0.**

### §31.4 — O portão

`nextest-impacted` **18 271/18 272** (a única reprovada é a flake do §31.3) · suíte da crate
**1 309 passed, 0 failed** · clippy `-D warnings` **zero** · `cargo fmt --check` **limpo**.

---

## §32 — A FILA INTEIRA (ordem do dono, 2026-09-22: *«enfileire tudo e comece com 2»*)

> ⛔ **Isto NÃO é dívida de integração.** Nada aqui bloqueia a fusão; é o backlog vivo desta linha,
> em ordem, com **o que cada item precisa antes da 1.ª linha de código**. A §30 tinha só os dois
> primeiros — eles fecharam hoje e ficam no topo como **✅**, para o histórico da fila ser legível.

### Feito hoje

* ✅ **(2) O composite obedece à lei do re-carimbo** — §31.5. A causa era a `AppearanceSig`, não a
  rota; a cura são três campos e a ausência dos outros três está escrita ao lado.
* ✅ **(1) O `Spacing` sobe para debaixo do `Method`** — §31.6. Reordenamento puro, o guarda viajou,
  e o tecto de LOC obrigou o corte que deixou o ficheiro melhor do que estava.

### A seguir — **pedem uma DECISÃO SUA** (medidos, com o preço ao lado; não há trabalho a fazer até
o veredito)

1. **O `Mixing` no Impasto** (§19) — ele entrou porque a medição diz que aquele meio LÊ a lei, e
   você nunca o pediu. Reverter é uma linha.
2. **A tela grande com a pilha de cinco camadas** (§21.10) — a `2048²` a pilha põe o quadro no
   limite; as saídas estão escritas com o custo de cada uma.
3. **O `Pigment` mudou de sítio também na aquarela** (§19) — é a única coisa que você vê mudar num
   painel que já tinha aprovado.
4. **A cadência da 7.ª camada** (§25) — o `x` e o `+` não têm atalho nem desfazer próprio.

### A seguir — **MEDIÇÕES que faltam** (nenhuma é um defeito conhecido; são números que hoje não
existem, e sem eles não se pode decidir)

5. ✅ **FEITO em 2026-09-22 — §22.6-bis.** A tabela foi re-tirada a `95 %` de CPU ociosa, ganhou a
   coluna de **ms/evento** (a que decide) e a antiga foi **retirada com a refutação ao lado**: a
   frase *«duas e três camadas não se distinguem de uma»* era a assinatura de uma fixtura que não
   continha o fenómeno. ⇒ **não há problema de relógio na pilha**: o pior caso é `5,7 %` de um
   quadro por evento.
6. **Sete camadas nunca foram medidas no produto real** (§25) — a tabela existente é de `+1 Brush` /
   `+1 Erase`, não do topo da quota.
7. **O cartão de cinco camadas × três fileiras** (§22.9) — quanto empurra o resto do painel para
   debaixo da dobra. ⚠️ Hoje o cartão nasce a `63 px` e não a `413`, logo isto encolheu — mas o
   número do topo da quota continua por medir.
8. **O tecto `4,0` do tamanho de camada** (§21.11) é do relógio a `raio 24`; a faixa nunca foi
   varrida por raio.

### A seguir — **RESÍDUOS nomeados, com a cura escrita** (trabalho de facto, por ordem de tamanho do
que o artista vê)

9. **O relevo não entra na recomposição da pilha** (§22.9) — as três metades do impasto ficam de
   fora; no meio Digital, onde a pilha vive, não há relevo, e **o gate que o afirmaria não existe**.
10. **O resíduo do par Blur+Smear** (§27) — `12` de `255` num canto, atribuído à base congelada do
    esfregão, com a cura nomeada.
11. **O blend não-`Mix` de uma camada `Brush`** (§27) é aplicado uma vez na composição.
12. **O Blur governa o preço de TODAS as camadas** (§28) — `+29 ms` por camada sem ele, `+115` com
    um. Com sete posições isto passou a pesar mais.
13. **Metade dos BYTES da pilha** (§28) — os sete intermédios são `[f32; 4]`; um `u16`
    premultiplicado é `2×` e **não** é byte-idêntico ⇒ pede a barra de qualidade que a caixa já tem
    contra o binomial.

### Fora da fila desta linha

* **A flake `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`** (§31.3) — pedido de
  promoção à lista do `CLAUDE.md` §5.0. É acto do **integrador**, não desta linha.

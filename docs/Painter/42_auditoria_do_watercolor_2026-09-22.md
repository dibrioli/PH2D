# Auditoria do modo Watercolor — 2026-09-22

> Ordem do dono: *«agora que sabemos como resolver vários tipos de artefatos no painter, quero uma
> auditoria do modo watercolor buscando por problemas similares»*.
>
> Duas lentes: **CORREÇÃO** (o código faz o que o doc/commit afirma? dois lugares que têm de
> concordar sobre um facto e discordam) e **COSTURA DE UI** (todo widget pintado, populado E
> clicável; o clique chega ao barramento; a sequência leva a algum lado).
>
> ⛔ **Escrito ANTES da cura, e a lista é o que se manteve.** Os dois achados foram curados no
> mesmo dia, por ordem do dono (*«vamos curar as coisas»*) — as curas estão nos §1.5 e §2.5, e a
> parte de cima de cada achado fica **como foi medida**, porque é ela que explica porque a cura é
> essa e não outra.

---

## §1 — ACHADO Nº 1 (P0) · a aguada de uma figura depende de QUANTAS VEZES lhe tocaram

**Severidade: alta.** Visível, reprodutível, e no gesto que o dono já pôs na fila de amanhã
(*«nos strokes vivos, o ajuste de propriedades no painel deve apagar o traço, atualizar e voltar a
mostrar»* — é exactamente esse gesto que o dispara).

### Mecanismo

O caminho de figura do watercolor
([`stamp_drag_preview_watercolor`](../../crates/ph2d-tool-painter/src/tool/paint/stamp_preview.rs))
re-carimba a figura inteira a cada re-carimbo e, para isso, **limpa os acumuladores por-traço**:

```rust
self.clear_wet_coverage();   // zera stroke_coverage, stroke_density, stroke_deplete(+prox), drena wet_cum_dirty
self.clear_wet_color();      // zera stroke_color
self.accumulate_wet_coverage(wet_dabs);
self.accumulate_wet_color(wet_dabs);
self.apply_watercolor(true);
```

⛔ **O `wet_mix` NÃO está nessa lista.** Ele é o estado por-traço do MIXER (a carga do pincel, que
se esgota à medida que a tinta é depositada), e sobrevive de um re-carimbo para o seguinte. ⇒ o
2.º re-carimbo corre o mixer com a carga **já gasta** pelo 1.º, o 3.º com a dos dois, e por aí.

⚠️ **O próprio ficheiro declara a lei que isto viola**, duas linhas acima:

> *«shapes are not a wet session, so rebuild the whole shape's coverage fresh each frame»*

e o código escreve `self.paint.wet_session_base = None;` para o afirmar. *A intenção está escrita; o
`wet_mix` é o campo que ficou de fora dela.* É a **classe B** de 2026-09-21 — um acumulador
por-traço a atravessar uma fronteira que ninguém fecha — e o gémeo exacto do
[Bug #26](BUGS_painter.md) no composite, que precisou dos quatro canais de reposição.

### Como reproduzir

```
bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
    diag_o_resto_do_watercolor -- --ignored --nocapture --test-threads=1
```

A ESCADA (`n` re-carimbos contra UM, `Charge = 0.5`, elipse de 160 px, pincel 200 px):

| `n` | texels que diferem | pior byte |
|---|---|---|
| 1 (controlo) | `0` | `0` |
| 2 | `377 756` | **`15`** |
| 3 | `378 914` | `22` |
| 5 | `379 375` | `38` |
| 10 | `379 507` | **`64`** |
| 30 | `379 507` | `64` (satura) |
| 10, **`Charge = 1`** (controlo) | **`0`** | `0` |

⇒ a deriva **satura em `64/255`**, um quarto da gama, sobre a figura inteira (banda `1.4`–`349` px
do centro). O pixel do CENTRO não muda (`[38, 89, 191]` nos dois): a deriva vive no gradiente de
densidade, que é o que a aguada tem de mais característico.

### Atribuição — as TRÊS hipóteses que eu tinha estão REFUTADAS

Eu abri esta auditoria com três divergências lidas no código entre a caixa que se **SALVA**
(`watercolor_preview_footprint`: `edge_spread·2 + warp + 4`, do pincel ACTUAL) e a janela que se
**ESCREVE** (`watercolor_render/window.rs`: `reach + warp + 2`, das MÁXIMAS DA SESSÃO, unida com o
`wet_cum_dirty` cumulativo). **As três caíram por medição**, e é isso que dá direito ao achado:

1. ⛔ **o descasque é EXACTO.** Carimbar → descascar à mão → comparar com a tela intocada lê
   `resto = 0` nos três regimes (sem mixer · mixer armado · rewet alto). *A geometria da caixa está
   limpa neste caminho.*
2. ⛔ **as máximas da sessão não deixam resto:** baixar o `edge_spread` de `40` para `7` entre dois
   re-carimbos, **com o mixer desarmado**, lê `difere = 0` — e com o mixer armado lê exactamente o
   MESMO número que o controlo `7 → 7`. *O knob é irrelevante.*
3. ⛔ **o `wet_cum_dirty` É drenado:** o `clear_wet_coverage` faz-lhe `take()` e dobra-o no
   `wet_frame_dirty`. E mover a figura `120 px` com o mixer armado lê o mesmo número do controlo
   `0 px`. *O movimento é irrelevante.*
4. ⛔ **o plano da RESERVA está ilibado por ablação:** largar `stroke_deplete` +
   `stroke_deplete_prox` entre os dois re-carimbos (o que força o ramo de alocação do
   `watercolor_accum.rs:367` a re-semear) **não move o número** (`377 756` com e sem).

**A bissecção por campo é que decide**, repondo UM campo entre os dois re-carimbos:

| campo reposto | `difere` |
|---|---|
| nada (controlo) | `377 756` |
| **`wet_mix`** | **`0`** |
| `wet_styles` | `377 756` |
| `stroke_water` | `377 756` |
| `stroke_density` | `377 756` |
| `wet_soak` (+pos, +active) | `377 756` |
| `wet_shape_active = false` | `377 756` |
| `wet_session_canvas` | `377 756` |
| `wet_level_smear_pos` | `377 756` |
| `wet_backdrop` | `378 108` (não é cura — re-deriva o fundo e piora) |

### O gate que faltava

⛔ **Nenhum.** A busca por gates sobre a idempotência do re-carimbo no watercolor devolve zero: os
gates deste caminho medem **o que o carimbo produz** (a aguada bate o oráculo, o descasque não deixa
rasto) e **nunca se ele produz o MESMO ao ser repetido**. O gate que falta tem a forma que o Bug #26
já pagou:

> *re-carimbar `n` vezes tem de dar a MESMA tela que re-carimbar uma vez* — com o **CONTROLO** do
> mixer desarmado ao lado, porque sem ele um `0` lê-se como produto limpo E como fixtura que não
> contém o fenómeno.

⚠️ E a fixtura tem de **PARQUEAR** a figura: a 1.ª redacção desta sonda deixava-a **em voo** e leu
`MEXEU = 0` em todas as células, **o controlo digital incluído** — porque a 1.ª instrução do
`restamp_shapes_preview` é `if self.draft_stamp() { peel; return }`, e `draft_stamp()` é verdadeiro
durante *todo* o gesto até ao `Up`. *Uma fixtura que não larga o ponteiro mede um programa que nunca
pinta.*

### Alcance para o artista

O `Charge` é fileira pintada do cartão *Water* (`0..1`,
[`paint_watercolor.rs:327`](../../crates/ph2d-panel-painter-layers/src/paint_watercolor.rs)), e o
valor de fábrica é **`1.0`** — mixer desligado, caminho byte-idêntico. ⇒ **o defeito só existe com o
mixer armado**, que é a feature do `Charge`. Com `Charge = 1` a escada lê `0` a `n = 10`.

### §1.5 — A CURA (2026-09-22)

⭐ **A porta já existia e tinha UM chamador.** `reset_wet_mix()`
([`watercolor_mixer.rs`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_mixer.rs)) é
invocada pelo `open_stroke`, com o doc *«the mixer reservoir starts fresh (no pickup) each stroke»*.
A cura é **uma linha**: o `clear_wet_coverage` passa a chamá-la.

⚠️ **E a porta é o `clear_wet_coverage` e NÃO o caminho da figura — isso foi MEDIDO, não escolhido.**
O 2.º chamador daquela função é o `Anchored`/`Drag Dot`/`Line` do TRAÇO (`stamp_stroke_dabs`), que
reconstrói o lote inteiro a cada movimento pelo mesmo mecanismo, e a sonda mostra que ele tinha o
**mesmo defeito** (`n = 2` ⇒ `61 010` texels, pior byte `4`; com o mixer desarmado, `0`). ⇒ *o
defeito não era das figuras: era de quem reconstrói o lote.* A lei que fica é a que o próprio corpo
daquela função já escrevia para a reserva — ***o que recomeça com a cobertura recomeça ali***.

⛔ **Com `wet_charge = 1` (fábrica) é no-op byte-idêntico:** o `WetMix::default()` é o estado que o
caminho sem mixer nunca abandona.

**Gates** ([`watercolor_recarimbo_tests.rs`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_recarimbo_tests.rs)),
**red-first — 3 reprovavam e os 2 CONTROLOS já passavam**, que é o que prova que eles medem o mixer
e não uma propriedade trivial do re-carimbo:

| gate | antes | depois |
|---|---|---|
| a figura: `n = 10` == `n = 1` | ✗ `379 507` | ✓ |
| o Anchored: `n = 10` == `n = 1` | ✗ `62 032` | ✓ |
| a fiação: a reposição vive na PORTA | ✗ | ✓ |
| CONTROLO figura, mixer desarmado | ✓ | ✓ |
| CONTROLO Anchored, mixer desarmado | ✓ | ✓ |

**Prova de mutação — 2 a sangrar + controlo limpo**, e a 2.ª é a que justifica o desenho:

- **M2** — a chamada existe no TEXTO e não corre (`if false`): as **duas leis sangram** e a fiação
  textual fica verde ⇒ *as leis medem o barro, não o texto*.
- **M3** — a reposição muda-se da porta para o chamador da FIGURA: a figura **cura**, o **Anchored
  sangra** e a fiação sangra ⇒ *a escolha da porta é load-bearing, e o gate do Anchored não é
  redundante*.

Suíte da crate depois da cura: **1 309 passed, 0 failed**.

---

## §2 — ACHADO Nº 2 (P3) · um braço de clique que gesto nenhum alcança

**Mecanismo.** `PAINTER_WATERCOLOR_PIGMENT` é declarado em
[`ids/painter_watercolor.rs`](../../crates/ph2d-tool-painter/src/ids/painter_watercolor.rs) e é o
**único dos 38** ids daquele módulo que o painel **nunca cita** (medido: 37 de 38 citados). Ele não
está no `PAINTER_WATERCOLOR_FIELDS` nem no `PAINTER_WATERCOLOR_CLICKS` ⇒ **não é registado e não é
focável** — não rouba foco a ninguém. Mas o braço

```rust
// watercolor_settings.rs:51
PanelEvent::Click(id) if *id == crate::ids::PAINTER_WATERCOLOR_PIGMENT => { … }
```

continua vivo, à espera de um clique que **nenhuma superfície pode produzir**: a fileira `Pigment`
saiu do cartão *Water* em 2026-09-20 e o painel conduz aquilo pelo `PAINTER_WATERCOLOR_MIX`.

⚠️ **É a terceira espécie do §5.0 — o ÓRFÃO, não o morto** — e a cura é OPOSTA à de um knob morto:
apagar, nunca ligar. ⭐ **E ele está declarado:** o doc do próprio id diz *«The tool setter
`toggle_brush_pigment` + this id stay for the tool API / back-compat»*, e o doc do id **vizinho**,
doze linhas acima, avisa contra exactamente esta podridão (*«…ever painted — the same rot the Paper
slot's Rake/Random ids became before they were removed»*).

**Como reproduzir:** o censo acima (declarados `38`, citados pelo painel `37`).

**O gate que faltava:** os censos de id deste repo perguntam *«o que é pintado está registado?»* e
*«o que é registado é alcançável?»* — **nenhum pergunta «o que é DESPACHADO chega a ser pintado?»**,
que é a direcção deste achado. ⏳ **Decisão do dono/linha dona:** ou o `back-compat` tem consumidor
(hoje não tem — a varredura do repo inteiro devolve só aquele braço) e fica isento com número, ou os
dois saem juntos.

### §2.5 — A CURA (2026-09-22)

O trio saiu inteiro — o id, o braço de despacho e o setter `toggle_brush_pigment` —, que é a lei que
esta casa já escreveu na retirada do *Self Pickup*: ***quando o único leitor de um valor sai, o valor
sai com ele***. ⚠️ O campo `brush.pigment` **FICA**: ele é lido pelo `spec/queries.rs` e escrito pelo
`set_brush_pigment_mixing`, que é o caminho vivo.

⛔ **E havia uma segunda razão, mais forte que «está morto»:** o `toggle_brush_pigment` virava a
bandeira `pigment` **sem tocar no `pigment_mix`**, enquanto o `set_brush_pigment_mixing` mantém o par
coerente — *duas respostas ao mesmo facto, e a morta era a que estava errada*. Se alguém o acordasse,
acordava com o defeito dentro.

**O gate** (`todo_click_despachado_pela_seccao_e_alcancavel`, em
[`watercolor_settings/tests.rs`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_settings/tests.rs))
mede **a terceira direcção**: todo id que o `route_brush_watercolor_event` despacha por `Click` tem
de estar no `PAINTER_WATERCOLOR_CLICKS`, que é a lista que o `event.rs` do painel varre. ⚠️ A isenção
do `PAINTER_SHAPE_*` é **medida** (aquele id é pintado, populado e roteado pela secção de FORMA), e
há **piso de população nos DOIS lados** da extracção — sem ele uma agulha partida varre zero e fica
trivialmente verde. Mutação: repor um braço sobre um id que só está no `…_FIELDS` **sangra**; partir
a agulha da extracção **sangra**.

---

## §3 — o que a auditoria mediu e NÃO acusou

- **O composite não corre no watercolor**, e não é um defeito: o `stamp_dabs_dispatch` desvia o
  watercolor e devolve **antes** do `stamp_dabs_routed`; o painel já esconde o cartão do composite
  nesse meio desde 2026-07-07, com o mecanismo escrito.
- **O descasque do watercolor é exacto** (§1, hipótese 1) — ao contrário do composite, que precisou
  do Bug #24.
- **A fileira `let _ = wetness_button_row(…)`** do `paint_watercolor.rs:132` é a **ÚLTIMA** do
  cartão (a função devolve `next_y` a seguir) ⇒ o `y` deitado fora é honesto ali, que é a cerca que
  a armadilha de 20/09 deixou escrita.
- **`the_mask_stroke_cost_does_not_follow_the_canvas`** reprovou uma vez durante a auditoria: é
  membro **já NOMEADO** da família de flakes de fan-out do `CLAUDE.md` §5.0 (3/3 verde sozinho a
  `load 2,59`, zero linhas de diff naquele caminho). **Não é achado.**

---

## §4 — a sonda

[`diag_o_resto_do_watercolor.rs`](../../crates/ph2d-tool-painter/src/tool/paint/diag_o_resto_do_watercolor.rs)
(`#[ignore]`, imprime a tabela e não afirma barra nenhuma). Ela tem **cinco** blocos: o descasque ·
o knob baixado · a figura que anda · a ablação do plano da reserva · a bissecção por campo · a
escada. ⚠️ Os controlos positivos (`vivo`, `MEXEU`, `armou`, `deplete`) vêm **à frente** do veredito
de propósito: sem eles a 1.ª redacção leu `0` em tudo e isso lê-se igual a um produto limpo.

---

## §5 — a lição de ARNÊS que esta jornada pagou

⛔⛔ **Um `git checkout -- <ficheiro>` para desfazer uma mutação APAGA trabalho não commitado.** A
prova de mutação do §2.5 mutava um ficheiro de teste que carregava o gate **acabado de escrever e
ainda não commitado**; o `checkout` de restauro devolveu-o ao `HEAD` e levou o gate junto. ⚠️ *O modo
de falha é o mau:* a corrida seguinte fica **verde** — não por o produto estar certo, mas por já não
haver gate nenhum a medi-lo. ⇒ **a restauração de uma mutação usa o `.bak` que o próprio arnês
guardou, nunca o git**, e o que é novo commita-se ANTES de se correr a mutação.

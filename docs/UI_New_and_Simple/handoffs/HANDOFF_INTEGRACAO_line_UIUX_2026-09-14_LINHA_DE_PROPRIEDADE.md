# HANDOFF — `line/UIUX` · 2026-09-14 · **A LINHA DE PROPRIEDADE TEM PORTA**

> Report do dono, com duas fotos: *«Label acima do campo numérico! Muito ruim!»* (secção LEG do
> Platform Player) · *«número na frente da label»* (Sprite Sheet) · *«falta para nós um modelo
> pronto e bem estabelecido com todas as regras para todos os widgets … o inspector está assustador
> de horrível»*.

## 1 — O censo: a coisa que o app mais repete tinha ONZE respostas

**Cinco pintores** de «rótulo + campo numérico» só no Inspector — **dois empilhavam** o rótulo
(`sections/rows::num_row`, `sections/visibility::number_row`, a foto 1) e três punham-no à esquerda.

E a **largura da coluna do rótulo**, medida por régua sobre todas as fontes de UI:

| valor | onde |
|---|---|
| `96` | `inspector/ordering` (×2) · `inspector/anchor_mount_row` |
| `78` | `inspector/sprite_sheet` · `inspector/transform` · `panel-physics` · `panel-wet-tuning` |
| `64` | `editor-core/panel/rows` · `panel-flip` · `panel-padding` · `panel-upscale` |
| `84` | `panel-color-equalization` · `panel-sculpt3d` |
| `76` · `72` · `150` · `176` | `panel-bgremoval` · `panel-equalize-sizes` · `panel-grid-snap` · `panel-timeline` |

⚠️⚠️ **E a primeira contagem — a minha, à mão — dizia SEIS.** Ela procurou nos painéis que a foto
do dono me pôs à frente; a régua varreu as fontes todas e devolveu **onze**. *Um censo escrito à mão
mede os sítios de que já se suspeita.*

## 2 — ⛔ Uma largura FIXA está errada por CONSTRUÇÃO, e a prova não é de gosto

A coluna docada é **arrastável** (`WidgetStore::DOCK_W_MIN`..`720`). Um rótulo de `96 px` numa
coluna aberta a `720` deixa o controlo com `600`; na largura mínima ele come a linha inteira.
⇒ ***os onze literais não são onze gostos: são onze leituras da MESMA coluna à largura de omissão.***

## 3 — A porta: `widget::property_row_columns`

Devolve `PropertyRow { label, control, dot }`.

- **A coluna do rótulo é uma FRACÇÃO da linha.** `LABEL_COL_FRAC = 0,348` é **medido**: `96 / 276`,
  onde `96` é a resposta que 3 dos 6 sítios do Inspector escreviam e `276` é a largura real de uma
  linha dele (`304` do token `inspector-w`, menos `2×8` de recuo do painel e `2×6` do cartão).
- **O tecto tem RECURSO:** o controlo nunca fica abaixo de `ICON_BTN_SIZE_PX + Spacing::Lg` — a
  coluna do *stepper* de um `NumberInput` mais um dígito (§0.0: *um limite legítimo diz de que
  recurso ele é*).
- **Ela COMPÕE com a [`form_row_columns`]**, não a duplica: o ponto da coluna de animação continua
  a sair de lá. *Duas portas a responder ao mesmo `x` é a forma que esta casa acabou de pagar onze
  vezes.*
- ⏳ **O vão horizontal `rótulo → controlo` é `Spacing::Md` (8) porque é o que a casa já responde**
  em 3 dos sítios — e **não tem derivação**. Fica NOMEADO como dívida em vez de fingir-se lei.

## 4 — Convertido: o Inspector inteiro

`sections/rows::num_row` · `sections/rows::seg_row` · `sections/visibility::number_row` ·
`sections/ordering::number_row` (×2) · `sections/sprite_sheet` · `sections/transform` ·
`sections/anchor_mount_row`. **Zero** literais de coluna ficaram no Inspector.

⭐ **A altura de uma row caiu de `rótulo + caixa + respiro` para uma LINHA** (~16 px por row), e a
`num_row_h`/`card_h` encolheram com ela — as duas têm de ser a mesma resposta, senão a moldura do
cartão nasce por cima das caixas.

⚠️ **A `seg_row` foi convertida no MESMO commit, e não por zelo:** ela partilha as secções com a
`num_row`, e deixar uma à esquerda e a outra por cima **é** a queixa do dono uma linha mais abaixo.
O grupo segmentado **reflui** dentro da coluna dele (três opções compridas quebram em duas
fileiras); o que mantém a secção legível é a coluna da esquerda estar sempre no mesmo `x`.

⚠️ **O `transform` teve só o NÚMERO trocado.** A lei adaptativa dele (inline × empilhado, decidida
por o que dois chips precisam) tem feedback do dono de 2026-05-24 por trás e fica intacta — o que
muda é ela passar a calcular-se com a coluna que a secção de facto desenha.

## 5 — ⭐⭐ A prova de que o painel encolheu não foi uma medição minha

O `seam_player::every_player_control_carries_a_hover_hint` reprovou com **«isenção STALE»**: a
entrada isentava *«o scrollbar do Inspector (ele nasce porque a §14 estica o painel)»*, e com as
rows mais curtas **o scrollbar deixou de nascer**. ⇒ *a metade de obsolescência de uma catraca
mediu o efeito da conversão antes de eu o medir.*

E o `every_form_row_reserves_the_animation_column` reprovou por ser uma régua **textual** que
procurava o nome da porta antiga: ensinada às **duas** (a nova compõe com a velha e devolve o mesmo
`dot`).

## 6 — O gate: `the_label_column_is_one_answer`

Duas metades — *ninguém escolhe a largura* + *a tolerância ainda descreve alguma coisa*. A catraca
nasce com **12** entradas (os painéis fora do Inspector) e ⛔ **só encolhe**.

⚠️ **A porta é isenta de si própria** — ela é onde a resposta VIVE, não um sítio de pintura.
⚠️ **O da timeline (`176`) responde a OUTRA pergunta** (a coluna de nome de uma FAIXA) e está na
lista por a régua ser textual; quem o converter decide primeiro se a pergunta é a mesma.

## 7 — ⏳ ABERTO

- **Os 12 painéis da catraca** — é a continuação directa desta wave, e agora é mecânica: a porta
  existe e o gate nomeia os sítios.
- **O vão horizontal** (`Spacing::Md`) sem derivação.
- **O rótulo com UNIDADE no texto** (`Float Height (m)`, `Cling Distance (m)`): numa coluna de
  ~96 px eles elidem. A saída não é alargar a coluna — é a unidade sair do rótulo e ir para o campo,
  que é o que as ferramentas de referência fazem. ⛔ Mexe em strings de i18n e é wave própria.
- **A página escrita** do sistema, derivada destas portas — o passo 4 do plano que o dono aprovou.
  ⚠️ Ela vem **depois** das portas, de propósito: um documento escrito primeiro envelhece contra o
  código, que é exactamente o que a `docs/design/` fez (ela não diz **nada** sobre a linha de
  propriedade, a coisa que o app mais repete).

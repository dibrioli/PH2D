# HANDOFF — `line/UIUX` · 2026-09-14 · **O PASSO VERTICAL DE UMA PILHA PERGUNTA A PORTA**

> Wave escolhida pelo dono (*«qual próxima fase de implementação?»* → **os vãos dos painéis**),
> a seguir à varredura completa da paleta ([handoff do Inspector](HANDOFF_INTEGRACAO_line_UIUX_2026-09-13_INSPECTOR.md) §14).

## 1 — O censo: 53 sítios, TRÊS respostas, e a lei diz uma

`53` sítios avançavam o cursor vertical somando um degrau da escada à mão — `Spacing::Sm` (6 px) em
**26**, `Spacing::Xs` (4) em **25**, `Spacing::Md` (8) em **2**. A lei desta casa é **3**
(`ph2d_tokens::control_gap_px`), e ela é do próprio dono (2026-09-07): *«entre grupos de botões
temos um espaçamento, entre sliders outro espaçamento. Para ambos vamos colocar o padrão de
espaçamento de 3 px»*.

**27 dos 53 estavam no Inspector** — o painel que ele tem aberto o dia inteiro.

| painel | sítios | altura que ele deixa de gastar |
|---|---|---|
| `ph2d-panel-inspector` | 27 | **61 px** |
| `ph2d-panel-audio-mixer` | 6 | 18 px |
| `ph2d-panel-audio-editor` | 7 | 15 px |
| `ph2d-panel-painter-layers` | 2 | 8 px |
| `ph2d-panel-motion-params` | 5 | 5 px |
| `ph2d-panel-flip` · `ph2d-editor-core` · `ph2d-panel-vector` | 6 | 6 px |
| **total** | **53** | **113 px** |

## 2 — ⛔ Porque as réguas que já existiam não os viam — TRÊS cegueiras diferentes

Esta casa já tinha **seis** gates de ritmo, e a suíte estava **verde** com os 53 lá dentro.

| régua | o que ela procura | porque falhou |
|---|---|---|
| `the_row_pitch_is_never_written_at_the_painting_site` | `ROW_H_PX + Spacing::` | exige que a altura somada seja a da LINHA; estes somam `font`, `cb_h`, `BTN_H`, `CHECK_H`, `BAR_H`, `STRIP_H`, `MUTE_H`… |
| `the_tail_of_a_block_is_one_answer` | `y + … + Spacing::` **sem `;`** | estes são INSTRUÇÕES (`y += …;`), não a cauda que sai da função |
| `every_stack_of_rows_asks_the_rhythm` | um FICHEIRO que empilha chama alguma porta | é por **ficheiro**: quem chama a porta **uma** vez sai do censo com dez sítios ainda a escrever o degrau à mão |

⇒ ***três censos sobre a mesma grandeza, e a interseção deles tinha 53 sítios lá dentro.*** É a
sexta vez que esta jornada paga a forma *«um censo que conhece uma forma da pergunta é cego às
outras»* — e a cura é sempre a mesma: **a régua passa a ser a PERGUNTA** (*este cursor vertical
avança um vão?*), nunca a sintaxe de um caso.

## 3 — ⭐ A resposta não foi escolhida: já estava escrita na secção mais NOVA

O [`sections/actions.rs`](../../../crates/ph2d-panel-inspector/src/sections/actions.rs) (a *Signal
Actions*, 10/09) escreve `cur_y += font + ph2d_tokens::control_gap_px();` em **três** sítios. ⇒ a
pergunta *«o que fica depois de uma linha de texto numa secção?»* **já tinha dono**, e os 53 são os
sítios escritos antes de a porta existir. *Quando o código novo e o velho discordam, o novo é a lei
e o velho é a dívida — o censo é o que os torna comparáveis.*

⚠️ **Duas formas, UMA pergunta.** Metade dos sítios vinha depois de uma linha de TEXTO
(`cur_y += font + …`) e metade depois de um CONTROLO (`cur_y += BTN_H + …`). Parti-os para decidir
se precisavam de portas diferentes, e o doc do próprio `control_gap_px` responde por escrito: *«o
nome é `control_gap` e não `row_gap` de propósito… um nome que só cobre metade dos leitores convida
o segundo número»*. ⇒ **uma porta, para as duas.**

## 4 — O gate novo: `the_vertical_step_of_a_stack_asks_the_door`

Terceira metade do [`the_gap_between_two_rows_is_one_answer`](../../../crates/ph2d-editor-core/tests/it/the_gap_between_two_rows_is_one_answer.rs).
Régua: `y += <altura> + Spacing::<degrau>.px();` em qualquer fonte de UI.

⚠️ **A asserção NÃO é «chama o `control_gap_px`»: é «chama UMA das três».** Qual delas é a resposta
do sítio — uma lista pede o `list_row_gap_px` (1), uma fronteira de cartão o `section_gap_px` (8) —,
e é isso que impede o gate de empurrar o número errado para uma superfície que responde a outra
pergunta.

⛔ **Duas ausências deliberadas, e cada uma tem um caso que a paga:**

1. **`y += Spacing::Md.px();` SOZINHO não é acusado** — ali não há altura a somar, logo ele é um
   **RECUO** (o ar dentro de uma caixa), outra grandeza, que ainda não tem porta. *Alargar a régua
   até lá fabricaria dívida sobre uma pergunta que ninguém fez.*
2. **Um degrau MULTIPLICADO não é acusado** — o 1.º falso positivo da régua foi o separador do menu
   de contexto: `y += 1.0 + Spacing::Xs.px() * 2.0;` é *um fio de 1 px com recuo em cima e em
   baixo*, ou seja a **ALTURA** do separador, não o vão que vem depois de alguma coisa.

⭐ **E o gate apanhou um 54.º sítio que a conversão mecânica não viu:**
`rows_paint_kinds.rs:438` — `y += seg_rows * ROW_H_PX + (seg_rows - 1.0) * gap + Spacing::Xs.px()`
— a altura de uma grelha calculada **à mão**, que por isso escapa também ao `grid_height(` da
segunda metade do gate da cauda. *A conversão era por LINHA e aquela linha estava quebrada em
várias antes do `fmt`; o gate lê o ficheiro já formatado e por isso viu o que ela não viu.*

**Prova de mutação:** repor `Spacing::Sm.px()` em `sections/timers.rs:309` ⇒ RED, nomeando ficheiro
e linha.

## 5 — Estado

- **54 sítios** convertidos (53 mecânicos + 1 apanhado pelo gate), **113 px** de altura recuperados.
- `clippy --workspace --all-targets -D warnings`: **exit 0** (um `use … Spacing` ficou morto no
  `painter-layers` e foi removido).
- ⚠️ **Nenhum gate de geometria se moveu**: as alturas que os testes medem são de LINHA e de CARTÃO,
  e o que mudou foi o vão ENTRE elementos.

## 6 — ⏳ ABERTO

- **O RECUO não tem porta.** `y += Spacing::X.px();` sozinho aparece em vários painéis e responde a
  *«quanto ar dentro desta caixa»* — uma grandeza real, sem nome e sem derivação. É a candidata
  natural à wave seguinte, e ⛔ **não se adivinha o número**: ele sai do modelo, como os outros três.
- **Os 74 sítios HORIZONTAIS** (`x + Spacing::…`) não foram tocados: o vão horizontal entre um
  controlo e a borda, e entre dois controlos lado a lado, ainda não foi censado por pergunta.
- **356 constantes locais** (`let pad = Spacing::Md.px();`) — cada uma é uma resposta privada de um
  ficheiro. Elas não são dívida por si: viram dívida quando duas delas respondem à mesma pergunta
  com números diferentes, e isso **ainda não foi medido**.

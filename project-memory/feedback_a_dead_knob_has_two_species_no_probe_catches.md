---
name: feedback_a_dead_knob_has_two_species_no_probe_catches
description: Seguir um controlo até «quem o lê» não basta — o defeito vive no terceiro passo, e há duas espécies que sobrevivem a todo gate de registo: o dreno de um braço só, e o consumidor que projecta o valor fora
metadata:
  type: feedback
---

Para saber se um controlo está morto, seguir **o fio inteiro**: *o painel escreve onde · **quem
lê** · **o leitor DECIDE, ou entrega a alguém que descarta?*** ⚠️ **O terceiro passo é o que um
`grep` não vê**, e é onde os defeitos vivem.

**Medido (2026-08-30): 34 controlos MORTOS sobre ~504 seguidos até ao efeito.** Duas espécies que
nenhuma sonda deste repo apanhava:

⭐ **O dreno de UM BRAÇO SÓ.** Não é um clique sem handler — é um handler cujo `if let` **não cobre
a variante**. Um painel inteiro perdeu **seis famílias de widget** de uma vez (tabs, segmented,
radio, dropdown, text, number), e a acusação **sobrevive a todo gate de registo**: os ids estão
registados, os cliques chegam, e morrem no fim do quadro.

⭐ **O consumidor que PROJECTA o valor fora.** O fio está completo, o valor chega ao solver — e a
matemática descarta-o (mínimos quadrados sobre um campo não-integrável: pedir `400 %` move a saída
`7 %`). *Nenhuma sonda de «quem lê este campo?» o vê: ele **é** lido.*

⚠️ **E a forma mais comum: a lente do PAINEL é mais larga que a do CONSUMIDOR.** O painel pergunta
*«há uma moldura?»* / *«há um corpo?»* / *«há um eixo?»* onde o consumidor pergunta *«qual
**direcção**?»* / *«a fonte tem `fvar`?»* / *«quantos slots **registei**?»*. Cinco dos 34 são isso —
e em **três** a regra certa já estava escrita no mesmo ficheiro, para o controlo vizinho.
⚠️ Irmã disto: **o gate que PINTA é um `OR` e o que CONSOME é um `AND`** ⇒ existe sempre uma janela
morta, e ela costuma ser **o primeiro estado que o artista alcança**.

⛔ **Nenhum instrumento deste repo pergunta se o VALOR chega a um consumidor.** O
`architecture_panel_wiring_parity` mede *focalizabilidade* (pintado sem estado interactivo ⇒ morto
sob o dedo); os `seam_*` provam que o clique **chega à ferramenta**, nunca que a escrita dela chega
a um **efeito**. Cinco dos defeitos são exactamente esse buraco.

⭐ **E o achado positivo diz o que construir:** o único painel **42/42 limpo** é o **gerado por
tabela** — *um painel derivado de uma tabela não tem onde esconder um knob morto.*

**Why:** um controlo que não move nada é indistinguível, na tela, de um que move — até o artista
tentar. E as duas espécies acima passam por todo gate que este repo tem.

**How to apply:** ao caçar, não pare em «alguém lê». Abra o consumidor final — se for uma
dependência, **leia o fonte dela** — e pergunte se o valor **decide** alguma coisa. Ao construir um
painel, prefira **tabela**. Ver
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] e
[[feedback_a_smoke_scene_that_teaches_the_opposite_is_worse_than_no_scene]].

## ⭐ Adenda 2026-09-01 — a TERCEIRA leitura: o consumidor GENÉRICO

O `the_painted_control_reaches_a_consumer` acusou a caixa arrastável do Widget Lab de não ter
consumidor. Ela **tem**: é um `InteractiveState::Slider` registado, e quem a move é o despacho de
ponteiro **genérico** (`interaction/dispatch/pointer_*`), que move *qualquer* slider **sem nunca
nomear um id**. Provado por medição: premir a 75 % da largura põe o valor em `0,75` e o estado em
`Dragging`, partindo do `populate` do próprio painel.

⇒ **um consumidor genérico lê-se exactamente como consumidor nenhum.** A régua só reconhece
términos POSITIVOS (`id == X`, braço de `match`, chave de tabela), e um despacho por *tipo de
estado* não tem nenhum — de propósito, porque é isso que faz o gesto ser o do produto.

**Como distinguir das outras duas:** pergunte *como é que este controlo é servido?*
- por um braço que **nomeia** o id → a régua vê; se não houver braço, é **morto**.
- por um despacho que serve o **TIPO** (`Slider`, `Button`, `TextInput`) → a régua **nunca** o vai
  ver, e a cura é uma **prova medida** na catraca, não um braço inventado só para a calar.
- ⛔ Inventar um braço `id == X` que não faz nada só para satisfazer a régua **cria** o knob morto
  que ela procura.

Irmã da entrada `HIER_SEARCH` da mesma catraca, que é invisível por outro motivo (o efeito não sai
do pintor). *São dois pontos cegos diferentes com o mesmo sintoma.*

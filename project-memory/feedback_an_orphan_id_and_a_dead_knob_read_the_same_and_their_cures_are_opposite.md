---
name: feedback-an-orphan-id-and-a-dead-knob-read-the-same-and-their-cures-are-opposite
description: Um id declarado que ninguém pinta é LIXO (cura = apagar); um pintado sem consumidor é knob MORTO (cura = ligar) — e uma varredura de fonte vê os dois iguais
metadata:
  type: feedback
---

A caça a controlos mortos de 2026-08-30 sinalizou dez ids. Dois deles não eram controlos:
`WET_TUNING_SCROLL` e `INSP_PLAYER_ADD` — **declarados, nunca pintados, nunca registados**. Um era
resíduo de uma barra de rolagem que usa outro id; o outro, o registo de um botão que saiu do
produto numa wave anterior.

- **ÓRFÃO** — o `const` existe e nada o pinta nem regista. Cura: **apagar**.
- **MORTO** — é pintado, registado, clicável, e o valor não chega a consumidor nenhum. Cura:
  **ligar o braço**.

**Why:** uma sonda cuja população é *"todo `pub const X: NodeId`"* não distingue as duas — ela
pergunta *«quem consome este id?»* e recebe "ninguém" nos dois casos. Tratar um órfão como knob
morto leva alguém a **construir um consumidor para um widget que não existe**; tratar um morto
como órfão apaga o id e deixa o widget pintado a apontar para o nada.

**How to apply:** antes de acusar um id, faça a pergunta que separa as espécies — *isto chega a
ser PINTADO ou REGISTADO?* Cruze a população de `ids::*` com as ocorrências em `paint*` /
`hit_index.register`. O que não aparece em nenhum é órfão, e é uma classe própria. É o terceiro
passo de [[feedback_a_dead_knob_has_two_species_no_probe_catches]], aplicado um nível acima: ali a
pergunta é *quem DECIDE com o valor*, aqui é *existe superfície de todo*.

⚠️ E há uma terceira leitura que se confunde com as duas: um `HitIndex::register` cujo efeito é
**bloquear** (o fundo de uma janela flutuante) tem término por **AUSÊNCIA** — o canvas só recebe o
clique quando o índice responde `None`. Nenhuma varredura de términos positivos o vê, e ensiná-la a
aceitar o padrão branquearia os cabeçalhos de secção genuinamente mortos, que têm a mesma forma.

## ⛔⛔ Adenda 2026-09-01 — a espécie **SEM GEOMETRIA**, e a sonda que faltava

Report do Enio sobre um painel novo: *«não tem scroll nem modo de estreitar para testar»*. As três
alças da janela — mover, redimensionar à direita, à esquerda — estavam **registadas no `populate`
desde o primeiro dia**, com o `parent` certo e o `BlenderHitKind` certo. A janela não se mexia.

**`store.register(id, …)` diz o que o id É. `hit_index.register(id, rect)` diz ONDE ele está.**
Sem o segundo, o id existe, tem estado, aparece em todo censo — e **nenhum pixel lhe pertence**.

⇒ **quatro** coisas leem-se igual e têm cura diferente:

| espécie | o que falta | cura |
|---|---|---|
| **órfão** | ninguém o pinta nem regista | **apagar** o const |
| **morto** | ninguém lê o valor | **ligar** o braço do consumidor |
| **sem geometria** | ninguém lhe dá um rect | **pintar**: `hit_index.register(id, rect)` |
| *(falso positivo)* | o consumidor é **genérico** | prova medida na catraca — ver [[feedback_a_dead_knob_has_two_species_no_probe_catches]] |

⚠️ **A terceira escapa às duas sondas existentes, e por razões simétricas:**
`hit_indexed_ids_are_registered` pergunta *«este id PINTADO está no store?»* — a alça está;
`the_painted_control_reaches_a_consumer` pergunta *«alguém LÊ?»* — o despacho `BlenderHit` lê.
A pergunta em falta é a terceira: ***algum sítio dá um rectângulo a este id?***

**How to apply:** ao acrescentar um painel, o gate barato é textual e mora na crate dele —
*todo `ids::X` do `populate.rs` aparece num pintor* (`ph2d-panel-widget-lab/tests/geometry.rs`).
⚠️ E o irmão: **um corpo que `push_clip` tem de chamar `paint_scrollbar`** — um painel que recorta
e não rola é a pior das três formas (sem recorte desenha por cima e vê-se; com recorte e rolagem
funciona; **com recorte e sem rolagem esconde os controlos e não diz nada**). ⛔ Essa nota já estava
escrita no `MODEL3D_SCROLLBAR_ID` e **não impediu nada** — *uma nota num irmão não é um gate.*

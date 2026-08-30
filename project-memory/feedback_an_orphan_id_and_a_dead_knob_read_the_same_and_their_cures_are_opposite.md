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

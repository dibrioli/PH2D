---
name: feedback_a_reserved_band_is_not_a_painted_band_and_geometry_gates_go_green_over_a_blank_screen
description: Gates that measure card_h / row rects measure the RESERVED band, and a band stays reserved when nobody paints it — six were green over the blank screen Enio photographed; the oracle has to be the Vello scene (glyphs + path segments) at EVERY zoom
metadata:
  type: feedback
---

**Uma faixa RESERVADA não é uma faixa PINTADA.** Em 2026-09-05 shipei os params dentro dos
cartões do Motion com **seis** gates verdes — todos sobre `card_h`, `param_row_rect`,
`readout_top` — e o Enio fotografou o resultado: *«tudo em branco»*. Os gates mediam a
GEOMETRIA que a row ocupa, e a row ocupa-a na mesma quando ninguém a desenha.

**A causa** era um LOD que saltava a row inteira abaixo de um limiar de legibilidade: o grafo
de smoke tem cartões altos (o `source.lsystem` põe **30** rows), o auto-fit da primeira pintura
afasta até ~0,2, e a faixa ficava paga e vazia. *A cura foi separar as duas coisas — a BARRA
pinta-se sempre (dois rectângulos), só o TEXTO segue o zoom, que é o que o Blender faz.*

**Why:** é o achado §4.2 da auditoria do `source.lsystem` a repetir-se **uma wave depois, no
mesmo módulo** (ali o gate da queixa media a linha reservada, e apagar a pintura deixava-o
verde). Repetiu porque a tentação é a mesma: a geometria é fácil de asserir e a pintura não.

**How to apply:** quando uma wave desenha algo NOVO num sítio, o gate de aceitação é de
**PIXEL**, com a cena do Vello por oráculo — `scene.inner().encoding()`: `n_path_segments`
para as formas, `resources.glyphs.len()` para o texto (⚠️ um glifo **não** entra na contagem de
caminhos). O arnês é `MockPanelHost::paint_and_count_geometry[_with_layout]`.
⚠️ **Três armadilhas medidas no mesmo dia:**
1. um painel do **split** (o grafo do Motion, a timeline) recebe rect de área ZERO com
   `HeroLayout::for_viewport` ⇒ o gate fica **vácuo**: use a variante `_with_layout` e ponha
   `layout.motion_graph = viewport`;
2. o painel **ENQUADRA** na primeira pintura (`if !state.fitted`) e deita fora o zoom que o
   teste pediu — arme `state.fitted = true`, senão mede-se o auto-fit;
3. meça em **mais de um zoom**: um gate só a `zoom = 1` teria passado sobre o ecrã em branco.

Relacionado: [[feedback_i_write_the_right_guard_and_do_not_gate_it]] ·
[[reference_topic_gate_discipline]] · [[feedback_alive_reachable_and_in_the_wrong_place_are_three_questions]].

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

⛔⛔ **E há uma ESPÉCIE FORA DA APP, medida em 2026-09-15: o gerador de PDF que sai `✓` sobre uma
página partida.** O tutorial do componente foi gerado por `scripts/tutorial-pdf.sh` (HTML → Chrome
headless → PDF); o script confere que o ficheiro não saiu vazio, imprimiu `✓ … (153 KiB)` — e a
página 1 tinha **o título e mais nada**. Duas causas, as duas **mudas num browser aberto** e
visíveis só no PDF:

1. um `<div class="note">` fechado com `</p>`. O browser aninha **todo o resto do documento** dentro
   dele, e como a regra dele é `break-inside: avoid`, a impressão empurra o documento inteiro para a
   página seguinte. ⚠️ *Um `div` por fechar não dá erro em lado nenhum* — o balanço
   `<div>`/`</div>` (40 contra 39) foi o que o nomeou, depois de **sete** experiências a acusar
   inocentes (o `display:grid`, o `break-inside`, o SVG, o conteúdo do bloco…).
2. um `<path>` de SVG **sem `fill="none"`**: o `fill` inicial é preto, logo um caminho ABERTO em L
   desenha uma cunha sólida onde devia estar uma seta tracejada. As setas rectas não o mostram —
   uma linha tem área zero.

⇒ *o `✓` do gerador mede que um ficheiro nasceu, não que a página se lê.* **Renderize as páginas e
OLHE** (`pdftoppm -r 80 -png`), e use `pdftotext -f N -l N` para dizer, sem dúvida, o que caiu em
cada página — foi ele que provou que o corte era entre dois elementos vizinhos e matou seis
hipóteses de uma vez.

⭐ **E o texto de um tutorial ganha o gate que lhe faltava:** tudo o que ele marca como rótulo de
tela (`<code class="ui">`) tem de existir num pintor — com **piso de população** e a **metade
justa** (`ph2d-panel-inspector/tests/it/o_tutorial_nomeia_rotulos_que_existem.rs`). Ver
[[feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list]], 3.ª espécie.

Relacionado: [[feedback_i_write_the_right_guard_and_do_not_gate_it]] ·
[[reference_topic_gate_discipline]] · [[feedback_alive_reachable_and_in_the_wrong_place_are_three_questions]].

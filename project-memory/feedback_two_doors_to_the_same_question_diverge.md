---
name: feedback-two-doors-to-the-same-question-diverge
description: O botão e o atalho têm de chamar a MESMA função — e o gate anti-botão-morto percorre a lista que a UI PINTA, não uma lista escrita à mão
metadata:
  type: feedback
---

O Enio disse quatro palavras: *"undo/redo no sistema"*. O Ctrl+Z funcionava — eu tinha acabado
de provar isso no app. Os **botões Undo/Redo da barra** é que não:

- o **Undo** despachava `EditorAction::UndoImageEdit` — o desfazer de **imagem**, single-level
  (Trim / Make Square / Bg Removal). Mover uma forma, desenhar, apagar, e clicar em Undo **não
  fazia nada**;
- o **Redo** não despachava **coisa alguma**. Pintado, clicável, órfão — e com um gate ao lado
  afirmando que ele estava *"no store"*.

**Why:** o botão e o atalho respondem à MESMA pergunta ("desfaça"), e tinham **dois caminhos**.
Dois caminhos para a mesma pergunta divergem — é sempre quando, não se. O do teclado ganhou o
roteamento novo (Áudio → Painter → global → image-edit) e o do botão ficou parado no que era
verdade quando ele foi escrito. É a mesma família de
[[feedback_derived_coordinate_seed_must_match_sample]]: a coordenada derivada tem de sair da
MESMA função que a lê.

**How to apply:**
1. **Uma porta.** O botão levanta uma ação que o shell roteia para **a função que o atalho
   chama** (`EditorAction::UndoStep{redo}` → `App::undo_or_redo`). Se o botão tem lógica
   própria, ele vai divergir.
2. **Registrado ≠ despachado.** O gate que existia pedia que o id estivesse no `WidgetStore`
   — e estava. Ele era pintado, clicável e inerte. *(Irmão de
   [[feedback_painted_is_not_populated_paint_gate]] e [[feedback_panel_populate_register]]: a
   cadeia é `register → paint → hit → DISPATCH`, e cada elo precisa do seu gate.)*
3. **O gate anti-botão-morto percorre a lista que a UI PINTA.** Extraia a construção das
   entradas (`left_rail::rail_entries`) do `paint` e faça o gate iterar sobre ela, exigindo que
   `chrome::dispatch_all` consuma cada chip. Uma lista escrita à mão dentro do gate **drifta da
   tela** — seria repetir o erro um nível acima.

**Corolário (o que me impedia de gatear o Ctrl+Z):** o `winit::KeyEvent` tem campo privado e
**não pode ser construído** fora do winit. Enquanto a política morava dentro do
`on_keyboard_input(event: WinitKeyEvent)`, **nenhum teste conseguia apertar uma tecla**, e o
roteamento do Ctrl+Z era a única parte do input que gate nenhum alcançava. Um tipo de terceiro
que não dá para construir é uma **fronteira**: desembrulhe-o na hora e ponha a política do
outro lado.

Relacionadas: [[feedback_disabled_button_still_dispatches]] ·
[[feedback_context_menu_closes_on_down_repaint]] · [[feedback_tool_unit_green_integration_dead]]

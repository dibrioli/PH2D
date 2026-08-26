---
name: feedback-whoever-freezes-the-loop-declares-it-and-a-computation-freezes-too
description: A mensagem escrita depois de um trabalho pesado morre pelo mesmo mecanismo do diálogo modal — o relógio do chrome não distingue as duas paradas.
metadata:
  type: feedback
---

O `render_loop` envelhece os toasts (e a UI viva) com `wall_dt` **menos o congelamento
declarado** (`crate::modal::chrome_dt`). Quem congela o loop e **não declara** faz o quadro
seguinte cobrar a parada inteira à mensagem que ele acabou de escrever — que morre antes de
ser pintada uma segunda vez.

Em 2026-08-22 o culpado era o **diálogo nativo** e a cura foi a porta `modal::save_file` /
`pick_file`. Em 2026-08-25 o Enio reportou o **mesmo sintoma** (*"a mensagem não aparece"*) com
outra causa: a exportação do módulo 3D passou a correr uma cadeia de retopologia que levava
**8 min 15 s** numa peça de um milhão de faces. O diálogo passava pela porta; a **conta** não.

**Why:** *o relógio do chrome não distingue «parado por um diálogo» de «parado por uma conta»:
para a tela, os dois são o mesmo nada.* A porta nunca foi sobre `rfd` — a lei dela sempre foi
«quem congela o loop declara quanto», e o `rfd` era só o primeiro que a violava.

**How to apply:** todo trabalho síncrono que possa passar de um quadro corre dentro de
`crate::modal::stalling(|| …)` — cozer malha, serializar, gravar arquivo, assar textura. E o
gate corre o **produto** (chamar a função e conferir que `take_stall() > 0`), nunca um censo de
texto à procura do nome da porta: um censo passa verde sobre uma chamada morta ao lado do
caminho real. Ver [[feedback-a-rule-only-exists-if-it-is-on-the-path-of-who-executes-it]] e
[[feedback-a-tool-is-adopted-only-when-a-written-step-names-it]].

⛔ Medido 2026-08-22: há **25** chamadas de `rfd::FileDialog` em **12** arquivos do shell e só as
`field3d_*` passam pela porta — as outras 23 continuam a perder a mensagem que escrevem a seguir.

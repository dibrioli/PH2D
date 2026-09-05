---
name: feedback_the_face_of_an_app_is_its_token_table_not_its_widgets
description: "Redesenhar 44 widgets com a MESMA pele deixa o app «com a mesma cara» — a cara está em meia dúzia de números (raio, rampa de fundo, acento, moldura, sombra) que todos os pintores lêem"
metadata:
  type: feedback
---

Enio, 2026-09-04: *«o que eu pedi do início foi um redesenho completo da UI […] muito mais
parecida com Blender/Godot […] cada widget, até as cores, absolutamente tudo. Mas o que foi feito
deixou o app com praticamente a mesma cara.»*

**O que tinha acontecido:** dois dias (02–03/09) a redesenhar *pintores* — a caixa única, a marca à
direita, a coluna de animação — com o `tokens.json` intacto: `panel-radius: 16`, fundo **tingido**
de magenta, **quatro** acentos saturados, `stroke_rounded_rect` em 271 sítios, três sombras. Cada
widget ficou melhor e o conjunto ficou igual, porque o que o olho lê como «a cara» são **os números
que 1 629 sítios `ColorToken::` e 294 `Radius::` partilham**, não a forma de um slider.

**Why:** a cara de uma UI é uma propriedade **global** (rampa de fundo neutra ou tingida · um
acento ou vários · raio 4 ou 16 · moldura 0 ou 1 px · sombra sim ou não). Um pintor só consegue
ser tão plano quanto a tabela que ele lê lhe permite. Os quatro modelos planos que a pesquisa mediu
(Godot Modern, Graphite, egui, o MVP do próprio Enio) coincidem nessas cinco coisas e divergem em
tudo o resto — é por isso que se parecem entre si e não connosco.

**How to apply:**
- ⭐⭐ **Um pedido de «redesenho completo» começa pela TABELA, nunca por um widget.** Troque os
  cinco números primeiro e olhe; só depois vale a pena tocar num pintor.
- ⭐ **Um tema deve nascer de poucas entradas e derivar o resto** (Godot: base · acento · contraste
  · raio · espaçamento → 108 tipos de controlo). 83 slots escritos à mão são 83 sítios para a
  coerência se perder — e o sintoma é exactamente «cada parte está bem e o todo não».
- ⛔ **O «modelo a seguir» tem de ter CÓDIGO e licença que se porte** — a pesquisa que o dono pediu
  foi essa (`docs/UI_New_and_Simple/pesquisa/08`), e a resposta (Godot 4.6 «Modern», MIT) já
  estava vendorizada na árvore sem que ninguém a tivesse lido como modelo.
- ⚠️ Ao medir «quanto mudou», meça a **pele** (tokens, raios, molduras, sombras), não a contagem
  de widgets tocados.

Relacionado: [[feedback_a_line_that_replaces_a_ui_surface_orphans_what_another_line_added_to_the_old_one]] ·
[[feedback_the_design_being_asked_for_may_already_be_law_in_another_half_of_the_app]] ·
[[reference_topic_ui_seam_discipline]]

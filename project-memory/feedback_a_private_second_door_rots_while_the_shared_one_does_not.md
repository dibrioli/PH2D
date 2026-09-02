---
name: feedback-a-private-second-door-rots-while-the-shared-one-does-not
description: "Quando um módulo tem a SUA porta para uma pergunta que o app já responde, é a privada que apodrece — cure apagando-a, não completando-a"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T02:04:28.190Z
---

Dois sítios a responder à mesma pergunta não envelhecem por igual: **a que poucos leem é a que
apodrece**, e o defeito aparece longe de quem a escreveu.

**Medido na `line/UIUX`, 2026-08-30.** Enio: *«quando coloco Model, não consigo mais clicar nos
menus superiores nem nas abas. É como se tudo fosse canvas.»*

| porta para *«isto é moldura ou desenho?»* | como responde | quem perguntava |
|---|---|---|
| `chrome_hit::pointer_over_chrome` | o **índice de acerto** — o que o chrome pintou naquele quadro | todo o resto do app |
| `forwarding::cursor_over_hero_chrome` | uma **lista de 4 ids escrita à mão** | só os dois módulos 3D |

Ao trocar a barra de pills por uma barra de menus + abas, a lista ficou com **três entradas
mortas** e **duas superfícies novas descobertas**. A porta partilhada não precisou de uma linha:
ela pergunta a um índice que já sabia.

⭐ **A cura é APAGAR a segunda porta**, não completá-la — completá-la funciona no dia em que se
escreve e volta a apodrecer na wave seguinte. Os quatro pontos de chamada passaram à porta única, e
isso curou de borla um *«irmão por curar»* que um doc-comment nomeava havia meses. *Uma nota não
cura; uma porta cura.*

**Why:** a lista privada é lida por dois ficheiros, e nenhum deles é tocado pela wave que muda o
chrome. O índice partilhado é escrito por quem pinta — logo acompanha por construção.

**How to apply:** ao ver um módulo com a sua própria versão de uma pergunta transversal (*isto é
UI? isto é selecionável? isto é visível?*), **conte os chamadores das duas**. Se a privada tem
poucos e a partilhada tem o resto do app, a privada é dívida, não isolamento — e o gate que a
guarda também acredita nela.

Relacionadas: [[feedback_a_hand_written_list_beside_a_predicate_is_two_answers]] ·
[[feedback_paint_and_dispatch_must_read_the_same_source]] ·
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]] ·
[[feedback_a_source_parsing_gate_must_know_every_shape_of_what_it_parses]]

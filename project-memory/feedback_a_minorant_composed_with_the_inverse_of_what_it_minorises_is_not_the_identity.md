---
name: feedback_a_minorant_composed_with_the_inverse_of_what_it_minorises_is_not_the_identity
description: Encolher por `+r` e voltar por `−r` devolve o campo ORIGINAL, e não o arredondado — a nota que prometia uma abertura morfológica viveu waves em código shipado
metadata:
  type: feedback
---

Numa receita SDF de *encolher-e-deslocar* (`f_round = dist(x, S⊖r) − r`), a tentação é escrever
`dist(x, S⊖r) ≈ f(x) + r`. ⛔ **`f + r` é apenas um MINORANTE** da distância ao conjunto erodido, e a
igualdade falha **exactamente numa quina convexa** (ali a distância real ao erodido é `√2·r` e o
minorante dá `r`). ⇒ `(f + r) − r = f`: o campo volta a ser o **original**, com a quina viva.

**Medido** (2026-09-06, `sd_extrude` de um quadrado de meia-largura `0,2`, `round = 0,05`):

| ponto | `round = 0` | `round = 0,05` |
|---|---:|---:|
| quina **vertical** | `−0,00000` | **`−0,00000`** |
| **aro** (parede × tampa) | `−0,00000` | **`+0,02071`** |

`0,02071 = √2·r − r`. O aro arredonda porque ali as duas coordenadas **são ortogonais** e o
`√(a² + b²)` é a distância a sério; a quina do contorno não, porque `flat + r` já a subestimou.

**Why:** o doc do `sd_extrude` prometia, com todas as letras, uma **abertura morfológica** (*«um
pescoço mais fino que `2·round` desaparece»*) — e o pescoço **fica** (leu `−0,035`, o campo sem
filete nenhum). A frase viveu waves em código shipado, e o doc da **própria variante** dizia o
contrário e certo (*«as arestas verticais são o que o perfil desenhou»*), duas camadas acima. Eu
escrevi um gate para a frase errada, e foi o gate que a derrubou.

⭐⭐⭐ **E a MESMA lei pelo outro lado (2026-09-07, W134, o nó de toro): um minorante frouxo não deixa
a peça conservadora — ele ENGORDA-A.** O zero de `m·c − raio` está em `m = raio/c`, logo apertar o
divisor `c` para tornar o campo 1-Lipschitz **desloca a superfície para fora**: com `c` a `0,60` a
corda do nó engolia o toro inteiro, e os gates de campo (gradiente, minorante) estavam todos verdes.
⇒ **a folga que a marcha pede tem de multiplicar o campo INTEIRO** (`λ·(m − raio)` tem o mesmo
conjunto-zero) e **nunca o divisor de dentro**. *Um campo pode estar certo como minorante e errado
como forma, e nenhuma régua de marcha vê a diferença.*

**How to apply:** ao herdar uma receita de arredondamento, **meça de que ARESTA o filete é** antes de
prometer o que ele faz — não é obrigatoriamente de todas. E quando duas notas do mesmo mecanismo
discordam, a que está encostada ao tipo costuma ser a certa; a que está no algoritmo é a que
envelheceu. Escreva uma cena de smoke só depois disso: a `=27` ia demonstrar a peça a partir-se em
duas, que é ensinar o contrário do que acontece.

Vizinhas: [[reference_topic_implicit_field_laws]] · [[feedback_stale_comment_and_dead_code_lie]] ·
[[feedback_a_measured_refusal_can_name_a_property_the_consumer_never_needed]]

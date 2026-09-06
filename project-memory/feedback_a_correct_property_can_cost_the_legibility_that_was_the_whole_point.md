---
name: a-correct-property-can-cost-the-legibility-that-was-the-point
description: Partilhar um nó para garantir que duas comparações usam a mesma entrada torna o grafo ilegível; a igualdade por CONSTRUÇÃO (a mesma função) é tão forte e não custa um fio a atravessar a tela.
metadata:
  type: feedback
---

Ao montar um ARTEFACTO PARA SER LIDO — uma cena de demonstração, um grafo de exemplo, um
diagrama — há sempre a tentação de **partilhar** o pedaço comum: uma forma que três casos
carimbam, uma fonte que quatro ramos consomem. A razão é boa e verdadeira: *se as entradas
diferissem, quem olha atribuiria a diferença às entradas e não ao knob que se quer ensinar.*

⛔ **E o preço é o grafo.** Medido em 2026-09-05/06 (cena `=110`, treze casos do
`motion.duplicator`): com as formas partilhadas, um nó alimentava quatro carimbos em quatro
alturas e os fios atravessavam a tela. O report do dono chegou no dia seguinte, e é a régua:
*«tem tantos nós interligados que não pude entender. Crie uma cadeia de nós por output.»*

⭐ **A propriedade não tinha de morrer — tinha de mudar de dono.** Os três casos usam as mesmas
formas porque saem da **mesma função construtora**, não porque partilhem um nó. *Uma igualdade
por CONSTRUÇÃO é tão forte quanto uma por referência, e não custa um fio a atravessar a tela.*
Preço real: `98` nós em `4` ilhas passaram a `140` em `13` — **mais** nós, e legível.

⚠️ **A régua da legibilidade é a TOPOLOGIA, não a contagem de nós.** O que torna um grafo
ilegível não é ele ser grande, é uma aresta sair de um caso e chegar a outro. ⇒ o gate conta as
**componentes ligadas** e exige uma por saída (`each_output_is_its_own_closed_chain`), e o que
garantia a igualdade passa a medi-la **na SAÍDA** — que é onde a afirmação vive.

⇒ **A pergunta a fazer antes de partilhar:** *o que a partilha compra pode ser comprado por uma
FUNÇÃO?* Se sim, partilhar só compra o custo.

Relacionado: [[feedback_a_scene_that_teaches_the_opposite_is_worse_than_an_absent_one]] ·
[[feedback_the_example_the_user_points_at_may_be_the_exception_of_its_family]] ·
[[feedback_a_replacement_surface_must_copy_what_the_old_one_RESOLVES_not_only_what_it_shows]]

---
name: feedback_a_smoke_for_the_owner_explains_what_each_thing_on_screen_is
description: Smoke ao dono explica o CONCEITO e o que cada aparência na tela significa — ele nunca usou um editor desse tipo; clique-a-clique sem isso não basta, e cada passo é conduzido pelo gesto real e fotografado antes de enviar
metadata:
  type: feedback
---

Um smoke em passos numerados (§0.8) ainda falha se só disser *onde clicar*. O dono aprende a
ferramenta no smoke e **nunca usou um app desse tipo**: ele precisa saber o que está a olhar.

Relato de 2026-09-12, sobre o editor de regras do tilemap: *«O Smoke não é claro o suficiente
para quem nunca usou um app desse tipo»* — junto com *«a célula selecionada não fica marcada; ao
apertar a primeira vez ela não fica em branco»*.

**Why:** o smoke dizia *«clique o quadrado do centro até ficar com a cor de parede»* e *«clique
o quadrado duas casas abaixo, para ficar vazio»*. Nada explicava o que a grade 5×5 representa, e
o passo do centro estava **errado**: uma regra nova já nasce com o centro no terreno do pincel,
então o clique tirava-o da parede. Eu escrevi o passo lendo o código de outro caminho, sem
conduzir o gesto nem fotografar.

**How to apply:**
- Antes dos passos, 2–3 frases com o conceito em palavras de quem nunca viu (o que é uma célula,
  um terreno, um tile, uma regra).
- Para cada coisa que um passo nomeia, dizer o que ela É e o que cada aparência significa (ex.:
  *o quadrado do meio é a célula que recebe o desenho; escuro = não importa; branco = tem de estar
  vazio; cor = aquele terreno; X = fora do mapa*).
- Dizer o que olhar para confirmar (a cor que acende no mapa, o número que tem de bater).
- Conduzir cada passo pelo GESTO real numa cena e fotografar o estado inicial e o final antes de
  enviar — um passo escrito de memória afirma um estado inicial que pode não existir.

**Confirmado 2026-09-16 (TOP-20 #16):** a foto da cena, tirada ANTES de enviar, apanhou **três**
defeitos com a suíte inteira verde — botões `Reset`/`Remove` lidos `…` (largura escolhida em vez de
medida do rótulo), dois pontos numa linha de caixa, e a cena a sair do ecrã do dono. Como fotografar
sem tocar no ecrã real: [[feedback_spectacle_in_a_virtual_kwin_photographs_the_owners_real_screen]].

Irmã de [[feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list]].

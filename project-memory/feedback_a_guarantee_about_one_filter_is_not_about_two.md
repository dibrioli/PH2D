---
name: feedback-a-guarantee-about-one-filter-is-not-about-two
description: "Uma nota que garante «isto nunca fica vazio» fala de UM estágio; o filtro seguinte esvazia-o e o degenerado dispara em silêncio — medido 18× no torno"
metadata:
  type: feedback
---

⛔⛔ **Uma garantia escrita sobre UM estágio lê-se como uma garantia sobre a COMPOSIÇÃO — e o
estágio seguinte pode esvaziá-la, com o degenerado a disparar em silêncio.**

Medido em 2026-09-23 (`ph2d-field-eval`, o torno da cena `5`). O
`profile::sd_profile_in_region` pedia ao índice as arestas que a distância precisa naquela região
(`ProfileIndex::distance_edges`) e **depois** tirava, com um `continue`, as que assentam no eixo de
revolução. Ao lado do degenerado estava escrito:

> *«Um perfil cujo corte não deixou aresta nenhuma é **impossível** — a regra do corte guarda
> sempre pelo menos a aresta que realiza o `dmax`. Recair na conta completa é o degenerado seguro,
> e não um caso que se espera.»*

⭐ **A frase é VERDADEIRA sobre o corte e FALSA sobre os dois filtros juntos.** O corte podia
devolver exactamente a costura do eixo, o `continue` tirava-a, e a região recaía na árvore
**INTEIRA** — imagem certa, custo `18×`. Medido: `1` das `24` arestas do vaso assenta no eixo, e
numa grelha `32×32` em `(u, v)` **`5` de `1 024`** células pagavam `931` linhas em vez de `~50`,
todas elas **sobre o eixo à altura da costura** (dentro do sólido, por onde a marcha passa).

**Why:** um degenerado com a nota *«não se espera»* ao lado não é medido por ninguém — ele é o
ramo que se lê como morto. E as duas metades estavam em **crates/módulos diferentes**: quem
escreveu a garantia (o índice) não sabia do filtro que o consumidor aplicaria a seguir.

**How to apply:**
1. Quando um ramo defensivo tiver uma nota a dizer que ele é inalcançável, **conte quantas vezes
   ele dispara** antes de acreditar nela. Um contador ou um censo sobre uma grelha resolve-o em
   minutos, e não depende de máquina calma (é uma CONTAGEM).
2. A cura não é «um filtro a mais»: é a pergunta passar a ter **uma porta** e o estágio que
   garante a não-vacuidade correr sobre a **população que sobra** — aqui, `ProfileIndex::no_eixo`
   com dois leitores, e o `dmax` calculado já sem a costura. ⚠️ Uma mutação mostrou que **as duas
   metades** contam: com o `dmax` a sair da população inteira, nada sobrevive e o degenerado volta.
3. ⚠️ **Não sobrevenda a cura:** meça que CAMINHO a paga. Aqui o dispositivo nem usa aquele
   compilador de regiões (a fita dele é da peça inteira), logo a cura vale para a CPU e para a
   extracção, **não** para o quadro de omissão.

Ver [[reference_topic_code_gotchas]] · [[reference_topic_gate_discipline]] ·
[[feedback_a_promise_of_sameness_beside_a_copy_is_the_shape_that_diverges]].

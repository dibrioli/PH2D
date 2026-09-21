---
name: feedback_a_fulfilled_plan_that_does_not_say_it_was_superseded
description: Um plano CUMPRIDO que não declara ter deixado de ser a fila responde com confiança ao «qual é a próxima etapa?» — e a resposta é a wave errada
metadata:
  type: feedback
---

**Um plano velho que se declara velho é inofensivo; um plano CUMPRIDO que não diz que deixou de ser
a fila responde com confiança — e responde errado.**

Medido em 2026-09-21 (`docs/Render3d/`): o dono perguntou *«qual a próxima etapa do plano do
render?»*. Eu li o `03_o_plano.md` — auditado **contra o código** dois dias antes, com o estado de
cada wave correcto e uma secção *«A ordem, num parágrafo»* — e respondi `W9`. ⛔ **A fila tinha
mudado no dia seguinte a essa auditoria** (ordem do dono de 20/09: *«faça tudo que for necessário
para superar»*), e vivia noutras duas páginas (`14` §6 + `15` §5). Quem me corrigiu foi o dono:
*«será que está lendo o plano correto?»*.

**Why:** as três defesas desta casa não apanham isto. O plano **não estava desactualizado** (o
estado das waves estava certo), logo *«audite a lista contra o código»* passa; o índice do módulo
**listava** as páginas novas, mas nenhuma linha dizia **qual delas é a fila**; e uma fila cumprida
lê-se como *«o que sobra é o próximo»* — a `W9` era o único item aberto ali, e era uma resposta
**coerente, verificável e errada**. *A informação que faltava não era o estado de nada: era a
ORDEM em que duas páginas se substituíram.*

**How to apply:** ao responder *«qual é o próximo passo?»* de um módulo, a primeira pergunta é
**«qual é a fila EM VIGOR?»** e não «o que falta neste plano?» — a fila é a última **ordem do
dono**, e ela é datada. E ao fechar uma fila, o mesmo commit escreve, **no plano que deixa de valer
e no índice do módulo**, a frase que diz para onde ela foi e para onde foi cada item que ficou
aberto lá dentro (a `W9` foi absorvida por uma obra da fila nova; a `W6` ficou **fora** dela, e
dizê-lo é o que impede que alguém a pegue por ser o que sobra). Ver [[the-ruler-is-the-merge-base-not-a-moving-main]]
para a mesma forma noutra grandeza: *a régua certa é a que o tempo já moveu, não a que está escrita*.

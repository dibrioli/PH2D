---
name: feedback_a_gesture_that_returns_must_give_back_the_pose
description: Nenhum portão perguntava se um gesto é REVERSÍVEL — voltar ao ponto de partida deixava o corpo 155 cm fora do lugar, com 88 testes verdes
metadata:
  type: feedback
---

Toda ferramenta de arrastar precisa de um portão que pergunte **«e se eu voltar?»**: levar o ponto de volta ao lugar de onde saiu tem de devolver o estado de onde se saiu.

Medido no testbed do Cascadeur (14/09). Arrastar a mão direita num círculo de 25 cm no sentido **anti-horário** e soltá-la exatamente no ponto de partida: a mão voltava a **0,0 cm** do lugar e o **corpo ficava destruído** — cabeça **155 cm** fora do lugar, coxas **138°** viradas, canelas 77°. Havia **88 verificações automáticas verdes**, entre elas uma que media a suavidade do arrasto e outra que media o pé preso.

**Why:** todos os portões mediam o RESULTADO de um gesto — o ponto chegou? o pé ficou? a junta virou demais? Nenhum media a **relação entre dois estados do mesmo gesto**. Um solver de alcance é dependente do caminho por construção (a solução que ele acha depende de por onde o alvo passou), e essa dependência só aparece quando o caminho **fecha**. E o defeito é mudo: quem arrasta vê a mão obedecer.

**How to apply:** para qualquer gesto contínuo (arrastar, esculpir, pintar, girar a câmera), acrescente o portão do **caminho fechado** — e com um CORPUS, não um caso: aqui o pior caso é **caótico** (a mesma configuração lia 26 cm num conjunto de 6 círculos e 4 cm noutro), então o que se mede é a **média** e a **contagem** sobre 16 círculos (4 controles × 2 raios × 2 sentidos). Antes: média 12,4 cm, 14 de 16 acima de 5 cm. Depois: 2,1 cm e **zero**. Ver [[reference_topic_measurement_discipline]], [[reference_topic_gate_discipline]] e [[project_teste_cascadeur_2d_bones_testbed]].

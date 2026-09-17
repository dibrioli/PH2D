---
name: feedback_transferring_motion_to_another_body_preserves_the_contact
description: Ao levar um movimento capturado para outro corpo, preserve a altura do PÉ, não a do quadril — senão o boneco flutua e não há contato nenhum para a física medir
metadata:
  type: feedback
---

Uma captura de movimento traz **ângulos**; o corpo continua a ser o seu. Se as proporções diferirem, pôr o quadril na altura do original deixa o seu boneco **flutuando**.

Medido no testbed do Cascadeur (14/09), levando saltos da base da CMU para o nosso rig: as pernas do sujeito são ~25% mais longas (canela 54 cm contra 40), e com o quadril à altura dele **o nosso pé ficava 24 cm acima do chão em todos os 81 quadros**. Resultado: `detectarContatos` achou **0 apoios**, a física não encontrou voo nenhum, e a régua imprimiu *«a nossa física não achou voo nesta animação»* — que se lê como «o clipe não serve» quando o defeito era a transferência.

**Why:** o que define um salto é a relação do PÉ com o CHÃO — quando o pé dele toca, o seu tem de tocar; quando está 50 cm no ar, o seu está 50 cm no ar. A altura do quadril é consequência da perna, não causa. Toda física de personagem é feita de contatos, e uma transferência que não os preserva destrói exatamente a coisa que se ia medir.

**How to apply:** ao importar, ajuste a raiz quadro a quadro para que a **folga do pé** seja a da fonte (e nunca negativa: numa captura real o pé entra alguns micrómetros no chão, e copiar isso põe o seu debaixo dele). Diga quanto ajustou — aqui o quadril desceu até 31 cm. E note que **uma pose humana real não cabe nos limites das juntas de um rig**: nos mesmos clipes o cotovelo passava 33,7° do nosso limite em 56 de 81 quadros e o joelho 19,6°; apare na importação e **imprima quanto aparou**, senão a chave gravada e a chave mostrada divergem (um autoteste da interpolação apanhou isso). Ver [[project_teste_cascadeur_2d_bones_testbed]] e [[reference_topic_fixture_discipline]].

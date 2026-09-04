---
name: feedback-a-dependency-asserted-without-dismantling-it-is-a-deferred-feature
description: "«X precisa de Y» escrito numa nota de adiamento é uma AFIRMAÇÃO, não um facto — três vezes na mesma linha ela era falsa, e a feature era barata"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-03T01:27:28.099Z
---

Uma nota de item aberto que diz *«isto fica para depois porque **precisa** de Y»* carrega uma
afirmação técnica que quase nunca foi verificada — e ela envelhece a fingir que é arquitectura.
⚠️ *Uma dependência afirmada sem a desmontar é uma feature adiada com cara de arquitectura.*

Medido três vezes na `line/3DModeling`, e as três vezes a dependência era falsa:

| a nota dizia | o que a medição achou |
|---|---|
| *«as divisórias arrastáveis dependem do cabeçalho»* | não dependiam — a nota estava simplesmente errada |
| *«restaurar a divisão obrigaria a restaurar as quatro câmeras»* | **três** delas são DERIVADAS (nascem da orientação que o nome promete) |
| *«o cabeçalho clicável pede a faixa reservada»* | uma faixa é para uma **barra**; um **menu** precisa só de um alvo de clique, e o rótulo já tinha posição e tamanho |

A terceira custou meio dia e fechou o último item aberto de um canvas que estava *«quase pronto»*
havia semanas.

**How to apply:**
- ⭐⭐ **Ao pegar num item adiado, ataque primeiro a frase «porque precisa de»**, e não o item. Vinte
  minutos a desmontá-la decidem se a wave é de meio dia ou de uma semana.
- ⭐ A forma de a desmontar é perguntar **o que exactamente** o Y compra e **quem** o pediria: uma
  faixa reservada compra *espaço permanente*; um menu quer *um rectângulo clicável*. Palavras
  diferentes para necessidades diferentes que a nota tinha colapsado numa.
- ⚠️ É a irmã de [[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]]:
  ali a recusa foi **medida** e responde a outra pergunta; aqui ela **nunca foi medida** e responde
  a nenhuma.

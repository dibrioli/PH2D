---
name: feedback_two_requirements_can_be_contradictory_over_time
description: "«só nas pontas» e «não pode encolher» parecem independentes e não são: uma ponta VIRA interior quando a coisa cresce — procure a contradição no TEMPO antes de desenhar a lei"
metadata:
  type: feedback
---

**L-System, 2026-08-30, dois smokes do Enio no mesmo dia.**

1. *"elas não nascem e crescem na ponta dos galhos, elas aparecem em cada segmento"*
2. *"a cada segmento a folha cresce e diminui. bem bizarro"*

A minha lei entre os dois foi um **cruza-fade**: peso `f` à geração mais nova, `1 − f` à
anterior. Ela satisfaz (1) **numa planta parada** — sobram exactamente as pontas, cheias — e é
elegante: soma `1`, contínua na virada, sai de graça das colunas que já existiam.

⛔ E é insustentável em movimento, porque **uma ponta vira interior quando a planta cresce**.
Para manter «só as pontas» ela tem de apagar a folha que deixou de ser ponta ⇒ cada folha vira
um **pulso**: nasce, cresce, encolhe, some. O veredito de uma palavra: *«bizarro»*.

**Why:** os dois pedidos falam de instantes diferentes — (1) descreve o estado PARADO, (2)
descreve a TRAJECTÓRIA. Uma lei que só é verificada no estado parado parece satisfazer os dois.
*A contradição não está nos requisitos: está em exigir que uma propriedade do estado se
mantenha enquanto o estado muda de dono.*

**How to apply:** antes de escrever uma lei que preserva uma propriedade *posicional* («só na
ponta», «só o de cima», «só o mais recente»), pergunte **o que acontece ao membro que perde a
posição**. Se a resposta é *«desaparece»*, você desenhou um pulso — e o utilizador vai chamar-lhe
bizarro. As saídas honestas são duas: a propriedade passa a ser **monótona** (a idade: sobe e
nunca desce, e quem já foi ponta fica), ou **quem escolhe é o autor** e não a lei. Aqui foram as
duas: a lei é a idade, e ONDE a marca vive é a gramática do artista.

⚠️ E meça a fixtura do estado parado **e** uma varredura ao longo do crescimento: o gate que eu
tinha media `g = 4,0`, `4,5` e `5,0` e passava — o defeito vivia **entre** as amostras que eu
escolhi, no sinal da derivada. Relacionado:
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_a_correct_mechanism_can_prescribe_the_wrong_cure]] ·
[[feedback_ergonomics_verdict_is_a_design_bug]]

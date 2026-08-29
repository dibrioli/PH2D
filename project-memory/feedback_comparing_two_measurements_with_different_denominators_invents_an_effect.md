---
name: comparing-two-measurements-with-different-denominators-invents-an-effect
description: "Citar \"antes 69%, depois 138%\" com duas réguas diferentes fabrica um efeito que não existe — o A/B tem de correr a MESMA régua nas duas pontas"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-29T15:37:46.856Z
---

Um "antes → depois" só é uma medição se **as duas pontas saírem do mesmo instrumento**.

Caso real (L-System, 2026-08-29). Escrevi num doc-comment que ligar o `Grow Angle`
**piorava** as gramáticas de refinamento, *"Koch 69 % → 138 %"*. Os dois números existiam, os
dois estavam corretos — e vinham de **réguas diferentes**: o `69 %` de uma normalizada pela
subida total com 24 amostras, o `138 %` de outra normalizada pela média com 40. Com a MESMA
régua, Bush dá **138 % ligado e 138 % desligado**: o efeito que eu afirmei **não existe**.

Quem o apanhou foi o gate que eu escrevi para *defender* a afirmação
(`turning_on_the_angle_growth_makes_the_refinement_grammars_worse_not_better`) — ele reprovou
com `138% contra 138%`. Sem esse gate, a nota teria shipado e mandado a próxima pessoa
reconstruir uma recusa inventada.

**Why:** entre duas medições de uma sessão longa, o instrumento muda por baixo (o número de
amostras, o denominador, a fixtura, o default de um param). A memória guarda o NÚMERO e esquece
a régua, e dois números da mesma família parecem comparáveis quando não são.

**How to apply:** um A/B corre-se **na mesma invocação**, com a mesma função de medida e os
mesmos parâmetros de varredura — nunca citando um número de um scroll anterior. Se um número
antigo for a única referência disponível, **re-meça a linha de base** antes de a citar. E toda
afirmação de "X piora/melhora Y" ganha um gate que a corre nas duas pontas: relacionado a
[[feedback_subtracting_two_clocks_from_separate_runs_gives_the_sum_of_the_noises]] e
[[feedback_a_quality_bar_copied_from_another_doc_loses_the_density_it_was_measured_at]].

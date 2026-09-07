---
name: feedback_when_the_composition_already_expresses_the_geometry_ask_who_owns_the_numbers
description: «A composição já exprime isto?» tem duas metades — a geometria e a AUTORIA; a segunda é a que decide se os números podem ter linha no painel
metadata:
  type: feedback
---

O `CLAUDE.md` §5.0 manda medir se a composição já exprime um item antes de o construir, e isso já
tirou várias formas de filas sem uma linha escrita. ⚠️ **Mas a pergunta tem duas metades**, e medir só
a primeira devolve a resposta errada nos dois sentidos.

**Medido** (2026-09-06, o polígono de `N` vértices): a **geometria** já estava expressa **inteira** —
a distância a um polígono simples é literalmente o que o `sd_extrude` calcula, e o gate mede a
diferença entre as duas em `0,0` (não «pequena»). Pela primeira metade, a forma não devia existir.

O que faltava era a **AUTORIA**: os pontos de um `Extrude` são do **editor vetorial**, e o vínculo
re-coze o perfil a cada quadro ⇒ uma linha de painel sobre um daquele pontos seria escrita e
**apagada no quadro seguinte** — um controlo morto, que é o defeito que esta casa caça por escrito.

⇒ **primitiva própria que PARTILHA o campo**: mesma superfície, mesma função, dono diferente. Ela
herda de graça a especialização por ladrilho e o filete, e um gate impede a segunda cópia da lei.

**Why:** «o campo já existe» e «o artista pode editar isto aqui» são perguntas ortogonais, e só a
segunda decide a forma do painel. A nota do doc que planeava a wave dizia *«os vértices arbitrários
são o que o desenho já é — o ganho é só o custo»*: meia verdade, e a metade errada era a que decidia
se valia a pena.

**How to apply:** quando a medição disser *«a composição já exprime isto»*, faça a segunda pergunta
antes de fechar o item: **quem é o DONO dos números, e ele reescreve-os por quadro?** Se reescreve, a
composição exprime a forma e **não** exprime a edição — e são coisas diferentes para quem usa.

Vizinhas: [[feedback_the_design_being_asked_for_may_already_be_law_in_another_half_of_the_app]] ·
[[reference_topic_control_design_hazards]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]]

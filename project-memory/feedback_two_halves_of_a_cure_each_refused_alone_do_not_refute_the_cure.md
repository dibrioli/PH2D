---
name: feedback-two-halves-of-a-cure-each-refused-alone-do-not-refute-the-cure
description: "Duas fases, cada uma medida SOZINHA e recusada; juntas atingem o alvo. Meça o PAR antes de aceitar duas recusas como fecho"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-09-03T16:58:02.831Z
---

Na `line/quadextract` (2026-08-30) a densidade da ponta tinha **duas** recusas medidas e
escritas: graduar a fase zero (`PH2D_ISO_ADAPT=1`) *«cura a agulha e parte a cadeia»*, e
graduar o mapa (`Follow Curvature`) *«pede-se 400 % e a saída move-se 7 %»*. Li as duas e
conclui que o caminho estava fechado.

⭐ **Corri a matriz `2×2` e a célula que ninguém tinha corrido — as DUAS ligadas — foi a única
a atingir o alvo**: razão ponta/corpo `0,536` contra o alvo `0,59`, com as cascas radiais a
afinar `−54 %` (a referência aprovada faz `−52 %`). Cada metade sozinha dá `1,075` e `1,502`.

⚠️ **E os dois docs das recusas já diziam a razão**, cada um na sua última linha: *«a cadeia
inteira tem de ser consciente do sizing — não uma cerca só nesta fase»*. As recusas não eram
sobre a cura; eram sobre **meia** cura.

**Why:** uma recusa medida responde à pergunta que foi feita. *«A fase X sozinha serve?»* e
*«X e Y juntas servem?»* são perguntas diferentes, e o custo de as confundir é uma família
inteira de soluções dada por fechada. Aqui isso durou dois dias e apareceu num §5 como
«medido e não adoptado» duas vezes.

⭐⭐⭐ **CONFIRMADA numa segunda linha, 2026-09-03 (`line/3DModeling`, o chanfro honesto).** A nota
da recusa anterior dizia, literalmente, *«a dívida é uma só e tem duas metades que se movem juntas
⇒ curar isto é wave com espec»*. A célula `(1,1)` fechou **na mesma sessão** — e eram **quatro**
peças, não duas: recuo · normalização · filete por pares · **o limite da faceta**, esta última
ausente da matriz `2×2` original, o que fazia duas células medirem `17,5` e `20,1` onde a lei
completa mede `4,1` e `0,8`. ⚠️ *Uma matriz `2×2` só refuta o que ela consegue exprimir; a peça em
falta parece ruído nas células que a precisavam.*

**How to apply:**
- Quando duas recusas apontam **uma para a outra** (*«a cura verdadeira trata as duas ao mesmo
  tempo»*), isso não é uma nota de rodapé: é o **desenho experimental** que falta. Corra a
  matriz completa, não as diagonais.
- A célula `(1,1)` custa uma corrida. Duas recusas custam uma família.
- ⭐ E trate *«curar isto é uma wave com espec»* como uma **estimativa**, nunca um facto — irmã de
  [[feedback_a_dependency_asserted_without_dismantling_it_is_a_deferred_feature]]. Duas vezes em
  três dias a obra prevista como grande coube numa sessão.
- ⚠️ Se uma célula da matriz medir **muito** pior do que a vizinha, suspeite de uma peça em falta
  antes de a dar por refutada: ali a assinatura era *piorar com o filete*, e a peça que faltava era
  precisamente o limite em que o filete deixa de caber.
- ⚠️ Escreva a matriz **inteira** na tabela, incluindo a célula que não foi corrida, para que a
  ausência seja visível em vez de parecer respondida.

Relacionadas: [[feedback_a_phase_measured_alone_can_improve_and_make_the_pipeline_worse]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_a_constraint_imposed_in_one_phase_and_not_the_next_is_a_starting_point]] ·
[[feedback_a_sweep_whose_cells_all_agree_has_not_chosen_anything]]

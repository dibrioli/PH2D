---
name: feedback-a-suppression-log-that-names-the-wrong-simultaneous-cause-costs-a-journey
description: Quando dois motivos podem ser verdade no mesmo quadro, a ORDEM do `if` escolhe o que o log diz — e um FACTO tem de vir antes de uma AUSÊNCIA.
metadata:
  type: feedback
---

O `post_frame_undo` do PH2D suprime um passo por cinco motivos e imprime o nome do primeiro que
casa. A ordem tinha `!had_input` («sem entrada neste quadro») à frente de `gesture_in_progress`
(«arrasto do gizmo em curso») — e um arrasto em que o ponteiro não se mexeu naquele quadro é **os
dois ao mesmo tempo**. O log que o dono colou vinha cheio de *«sem entrada neste quadro»* sobre
quadros que eram, de facto, um arrasto; passei uma jornada a caçar uma deriva do documento que não
existe.

**Why:** um diagnóstico não descreve o estado, descreve **o primeiro ramo que casou**. Com causas
simultâneas isso é uma escolha de desenho, não um facto — e nomear a errada é pior do que não
nomear nenhuma, porque a errada é acreditada.

**How to apply:** numa cadeia de motivos, os que são um **FACTO do sistema** (um botão em baixo, um
gesto em curso, um worker a recalcular) vêm **antes** dos que são uma **AUSÊNCIA** (não houve
eventos). E, quando um motivo pode coexistir com outro, imprima **todos** os que casam em vez do
primeiro. Irmão: [[feedback_a_decision_log_that_omits_one_key_explains_every_choice_but_the_one_that_matters]].

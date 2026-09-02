---
name: feedback-a-change-detector-on-a-gesture-path-misses-the-gestures-that-return-early
description: "Detector de «isto mudou, grave» posto no caminho de um gesto só vê os gestos que passam por ele — ponha-o no QUADRO, onde o estado assenta"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T15:52:51.888Z
---

Persistência por **detecção** (o ficheiro é uma projecção do dono vivo; ninguém emite intent) é o
padrão certo — e o sítio onde o detector corre decide se ele funciona.

**Medido na `line/UIUX`, 2026-08-30.** A arrumação dos painéis era detectada no hook de ponteiro
(`forward_to_hero`), ao lado de dois inquilinos que já lá viviam. **Os dois gestos que arrumam o app
não passavam por lá:**

| gesto | por que escapava |
|---|---|
| arrastar a **borda** de uma coluna | o handler faz `return` no Move **e** no Up, antes do hook |
| largar uma **aba** noutro encaixe | é resolvido **dentro** do `paint`, depois do hook |

⇒ a largura da coluna nunca era gravada. Sintoma reportado: *«não funcionou. Voltou ao zero.»*

⭐ **A cura é o QUADRO**, depois de pintar: *um detector no caminho de um gesto só vê os gestos que
passam por ele; o quadro vê todos, porque é onde o estado assenta.* O custo é uma projecção por
quadro (consultas a um mapa que está **vazio** enquanto ninguém mexeu, mais um FNV).

⚠️ **E os inquilinos antigos ficam onde estão, de propósito** — eles são cliques que atravessam o
hero, e arrastá-los consigo seria mudar o que funcionava. O gate leva o controlo que o impede.

**Why:** o hook parece o sítio natural porque os primeiros factos persistidos eram todos
pointer-driven. A propriedade que importa não é *«o artista mexeu»* — é *«o estado assentou»*, e
essas duas coincidem só para os gestos que não têm caminho próprio.

**How to apply:** ao acrescentar um facto à persistência por detecção, **liste os gestos que o
mudam** e siga cada um até ao detector. Um `return` antes dele, ou uma resolução no `paint`, é a
resposta. E o gate afirma o **mecanismo** (*está no quadro* · *não está no hook*), nunca o endereço.

Relacionadas: [[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]] ·
[[feedback_a_private_second_door_rots_while_the_shared_one_does_not]] ·
[[feedback_a_surviving_mutation_can_mean_the_code_is_wrong_not_the_gate_missing]]

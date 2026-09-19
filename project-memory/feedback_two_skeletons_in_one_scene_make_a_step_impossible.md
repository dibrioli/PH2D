---
name: feedback-two-skeletons-in-one-scene-make-a-step-impossible
description: Um passo de smoke que nomeia um osso tem de dizer que PEÇA ele governa — numa cena com dois esqueletos o nome sozinho é ambíguo e o dono segue-o à letra
metadata:
  type: feedback
---

Na cena `PH2D_VEC_BONE_SMOKE=1` há **dois** esqueletos de três ossos: a barra vectorial
(`Bone 1..3`) e o braço PINTADO (`Bone 13..15`). Eu mandei o dono escolher `Bone 14` e pintar
na barra laranja — e ele respondeu com a medição: *«Bone 14 está ligado à imagem e não ao
vetor.»* Seguir o passo à letra **não podia** funcionar.

**Why:** os nomes são `Bone {índice da entidade}`, logo não dizem a que peça o osso pertence e
**renumeram-se** quando alguém acrescenta uma peça à cena. Um passo que nomeia um osso é uma
afirmação sobre duas coisas — o nome *e* o sujeito — e só a primeira é visível na Hierarquia.

**How to apply:** o roteiro imprime, **derivado do mundo**, quem governa cada peça («a BARRA
LARANJA obedece a Bone 1, Bone 2, Bone 3») e diz em voz alta que os esqueletos são separados.
⚠️ E a lista sai de um passeio pela CADEIA: a `esqueletos::ossos_desde` ordena por `to_bits`,
que no bevy é a criação **invertida**, e a frase saía «Bone 3, Bone 2, Bone 1». Relacionado:
[[feedback-a-smoke-step-that-names-a-panel-row-must-prove-the-row-is-in-the-list]] ·
[[feedback-a-smoke-for-the-owner-explains-what-each-thing-on-screen-is]]

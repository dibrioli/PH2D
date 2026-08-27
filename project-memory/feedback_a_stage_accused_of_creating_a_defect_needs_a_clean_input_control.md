---
name: feedback-a-stage-accused-of-creating-a-defect-needs-a-clean-input-control
description: "Escrevi «o remalhe cria nao-manifold sozinho» comparando dois numeros da MESMA peca partida — em onze pecas limpas ele cria zero; ele PROPAGA"
metadata:
  node_type: memory
  type: feedback
---

Medi `4 ⇒ 0` arestas nao-manifold na porta do remalhe e **`2` outra vez** depois do
laco, e escrevi no doc: *«o remalhe cria nao-manifold sozinho»*. Essa frase decidiu
**onde a reparacao vivia** (no fim do passe em vez da entrada) e ficou la' um dia.

Corri o controlo que faltava — o mesmo passe sobre as **onze pecas limpas** do
corpus. Ele cria **zero** em todas (`0 ⇒ 0`). O `4 ⇒ 2` era de UMA peca, que **entra**
com `4`: o remalhe **propaga** o defeito, nao o cria.

**Why:** eu tinha comparado dois numeros da **mesma peca partida** e chamado a isso
uma medicao. Um passe que degrada um defeito existente e um passe que inventa um
defeito produzem exactamente a mesma leitura nessa peca — e so' uma **entrada limpa**
os separa. *Um controlo sobre a FEATURE (ligar/desligar) nao substitui um controlo
sobre a ENTRADA.*

**How to apply:** antes de afirmar que uma fase **cria** um defeito, corra-a sobre
entradas que **provadamente nao o teem** — e de preferencia sobre todas as que ha'.
A afirmacao «a fase X cria Y» exige a coluna «X sobre entrada sem Y». Sem ela, a
unica coisa medida e' «X nao cura Y», que e' outra frase e escolhe outra
arquitectura. Irma^ de
[[feedback-a-new-features-gate-can-expose-a-pre-existing-bug-check-the-control-first]].

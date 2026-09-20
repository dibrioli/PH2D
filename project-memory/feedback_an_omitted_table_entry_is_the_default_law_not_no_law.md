---
name: an-omitted-table-entry-is-the-default-law-not-no-law
description: "Um osso ausente da tabela de limites/rigidez de um gerador não fica «sem lei»: fica com a lei de OMISSÃO do motor (±180°, rigidez de perna) — e o portão de continência não o vê porque salta quem não tem limite (o dorso do Dracorex, auditoria de 19/09)"
metadata:
  type: feedback
---

**O facto (auditoria do dinossauro, 2026-09-19, `auditoria/ACHADOS.md` B45):** o `gerar_rig_dracorex.py`
tem uma tabela de 15 limites medidos na caminhada do Cascadeur e uma de rigidez; o osso `dorso`
(a dobradiça entre a bacia e o tronco inteiro) não está em nenhuma das duas. O `construirRig` dá a
quem falta `lim = [−180, 180]` e `rigidez = 0,05` — o valor de uma PERNA. Na caminhada deles o dorso
anda **1,2°**; numa varredura de 432 arrastos o tronco chegou a **118°**, com **90°** no dorso. Era o
«tronco a 47°, medido e não curado, não é a cauda» da B44 — a B44 varreu a rigidez da CAUDA e nunca
perguntou que osso rodava.

**Why:** uma tabela escrita à mão por NOME cobre o que o autor lembrou; o que ele esqueceu não fica
neutro — recebe o valor de omissão, e esse valor foi escolhido para OUTRA população (a perna humana).
E o portão *«todo limite CONTÉM o medido»* percorre os ossos e faz `continue` em `!a.lim`: o osso sem
limite é exactamente o que ele não consegue ver, e o piso de população (`>= 15`) é cumprido pelos
outros. *O ausente é invisível a um portão cuja população são os presentes.*

**How to apply:**
- ao gerar um rig novo, o primeiro portão é de COBERTURA: *todo osso móvel (não-fixo, não-raiz) tem
  limite ESCRITO* — e só depois o de continência. O controlo é o rig antigo (a Cascy passa com zero
  ossos sem limite).
- quando um número declarado «não curado» não tem osso nomeado, DECOMPONHA-O por osso antes de varrer
  parâmetros (aqui: tronco = raiz + dorso + tórax; o dorso levava 89,9 dos 118,2°). Varrer a rigidez
  de um osso que não é o culpado dá «não é ele» para qualquer valor — que é o que a B44 leu.
- nunca deixe o motor escolher o valor por omissão de um osso NOVO em silêncio: ou o gerador escreve
  o número, ou o `construirRig` recusa em voz alta um osso móvel sem `lim`.

Família: [[reference_topic_gate_discipline]] (censo que presume o destino · piso de população) ·
[[feedback_a_gate_can_record_a_loading_defect_as_a_law]] · [[feedback_when_the_subject_changes_every_ruler_and_note_about_the_old_one_must_be_rechecked]] ·
[[project_teste_cascadeur_2d_bones_testbed]].

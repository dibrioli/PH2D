---
name: feedback-a-gate-that-fails-on-its-precondition-says-the-product-left-the-regime
description: Um gate que reprova na PRECONDIÇÃO («a fixtura tem de produzir o defeito») não diz que a lei está errada — diz que o produto deixou de alcançar aquele regime.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-09T21:14:11.844Z
---

Quando uma mudança faz **três gates reprovarem na linha da precondição** — o
`assert!` que diz *«a fixtura tem de produzir o defeito / o colapso / esvaziar»*,
e não a asserção que mede a lei —, o instrumento não está a acusar a lei nova:
está a dizer que **o produto deixou de chegar onde chegava**.

**Why:** medido em 2026-09-09 no filtro de tecido. Ao parametrizar o filtro pelo
arrasto em vez do relógio, usei como passo do produto o `QUANTUM_DE_ARRASTO` que
saía do **cabeçalho das fixtures do alvo** (`avanco_por_passo_px 90` ⇒ `0,09`).
Ele é a unidade em que a lei do alvo foi **calibrada**, não uma decisão sobre
quanto o dedo tem de andar para a simulação avançar. Com ele o filtro ficou `9×`
mais fraco (um arrasto de ecrã inteiro dava `11` passos contra os `~120` de
antes), e as asserções de lei continuaram todas verdes — só as **precondições**
caíram. ⇒ *um número do lado aprovado responde à pergunta dele, não à nossa.*

**How to apply:** ao ver uma precondição vermelha, não afrouxe a fixtura nem a
barra: pergunte **que número do produto encolheu**. E quando uma constante do
lado aprovado for reutilizada, escreva ao lado dela **de que pergunta ela é** —
se a resposta for «da calibração do alvo», o produto precisa da sua própria,
com a calibração escrita (aqui: `PASSO_DE_ARRASTO = 0,01`, derivado de preservar
o ritmo que o dono já aprovara). Duas constantes que medem a mesma grandeza
física podem ter **donos diferentes**.

Relacionado: [[feedback-a-bar-calibrated-without-the-approved-side-measures-our-own-defects]] ·
[[feedback-a-measured-refusal-answers-one-question-recheck-it-when-yours-is-another]] ·
[[feedback-the-example-the-user-points-at-may-be-the-exception-of-its-family]]

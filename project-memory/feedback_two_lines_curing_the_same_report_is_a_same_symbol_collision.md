---
name: feedback-two-lines-curing-the-same-report-is-a-same-symbol-collision
description: "Quando o Enio reporta o mesmo defeito a duas linhas no mesmo dia, as duas curam-no e a integração recebe a MESMA lei escrita duas vezes (dois nomes, mesma fórmula). O critério mecânico para fundir — uma lei sobrevive e tem de passar as suítes das DUAS — só vale se as fórmulas forem iguais; se diferem, é decisão de produto do Enio."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-08-23T00:53:12.185Z
---

**Caso (2026-08-22):** a emenda do tracejado num contorno fechado foi reportada pelo Enio às
linhas `motion-value` e `Vector` no mesmo dia. Cada uma curou-a no `ph2d-vec-scene`:
`dash_fit::{fit, longest_contour, dash_lengths_for}` de um lado, `StrokeSpec::dash_lengths_fitted`
+ `stroke_plan::dash_for` do outro — e as duas mudaram `kurbo_stroke` para a MESMA assinatura nova.
O `lib.rs` fundiu LIMPO (arquivos diferentes); só `expand.rs`/`vec-render` conflitaram. Sem olhar,
o `main` teria shipado **duas leis** para a mesma pergunta ([[feedback-two-engines-one-state-is-worse-than-a-slow-engine]]).

**Why:** o mesmo report → a mesma pesquisa (Illustrator/Figma: «não se muda o número de traços,
muda-se o período») → a mesma fórmula; o que difere é ONDE se mede (cache na tesselação vs por
peça) e o nome. É colisão de mesmo símbolo (DIRETRIZ §1.5.5) ainda que nenhum símbolo colida
textualmente.

**How to apply:**
- Ao ver dois hunks que curam «o mesmo report do Enio» em lados diferentes, pare de fundir texto e
  **compare as fórmulas** (e as suítes: cada lado trouxe 5-9 gates que dizem a lei em prosa).
- Fórmulas iguais ⇒ uma lei + as portas que só diferem em *quem já pagou* (cozido vs fonte, cache
  vs medição); a prova é a sobrevivente passar **as duas suítes** sem mudar asserção nenhuma, e o
  handoff da linha integrada ganha a nota (quem decidiu, por que critério, o que saiu).
- Fórmulas diferentes ⇒ não é mais a integração que decide: reporte ao Enio com a tabela lado a lado.
- O sinal de alerta é barato: `git log --format=%s` das duas linhas no mesmo dia com a mesma
  palavra do report (aqui, «tracejado»/«emenda»).

---
name: feedback-a-sampled-fixture-proves-what-it-sampled-gate-the-property-where-it-is-defined
description: Três mutações que cortavam a mais sobreviveram a 7.200 amostras — pare de caçar fixtura e afirme a desigualdade onde ela é definida
metadata:
  type: feedback
---

O corte por casco convexo do módulo 3D tinha um gate de amostragem: 300 regiões × 24 pontos, «a
aresta vencedora de cada ponto tem de sobreviver ao corte». Três mutações que **deitavam fora
arestas a mais** (medido: a média caiu de `39,7` para `37,8`) passaram — **nenhuma** amostra perdeu
a sua vencedora. Trocar a fixtura por um contorno espinhoso não chegou.

**Why:** a prova do corte assenta em **duas desigualdades**, e um gate de amostragem só as toca por
acidente. Num contorno redondo todas as arestas estão à mesma distância do interior, então um corte
agressivo raramente deita fora *a vencedora* — o defeito existe e a amostra não o encontra.

**How to apply:** quando o código tiver uma **prova** por trás (um majorante, um minorante, uma
convexidade), gateie **a desigualdade**, não o efeito dela: expor a função interna por um
`#[doc(hidden)] pub fn probe_*` e afirmar `bound ≤ verdade` sobre uma grelha densa.
⚠️ E o **controle** dessa gate mede outra coisa: pedir que a *maioria* dos pontos fique perto da
barra reprova sobre produto correto quando a folga é **da lei** (aqui `82%`, porque o majorante tem
de valer para o *pior* ponto). Peça que **alguns** cheguem lá.
Irmã de [[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]].

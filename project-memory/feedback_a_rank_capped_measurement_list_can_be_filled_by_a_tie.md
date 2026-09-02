---
name: feedback-a-rank-capped-measurement-list-can-be-filled-by-a-tie
description: "Um `take(N)` por posto é uma decisão sobre QUEM não é medido — e um empate pode preenchê-lo inteiro"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T19:04:29.808Z
---

Uma régua que ordena candidatos e corta em `N` (`sort` + `truncate(12)`) não está a
limitar custo: está a decidir **quem não é medido**. Se a grandeza que ordena admitir
**empates**, um empate de `N` elementos enche a lista e o que interessa fica de fora — e a
saída lê-se como *«medi tudo e está perfeito»*.

Medido 2026-08-31 (`line/quadextract`): a lei de «o que é uma ponta» ordena ápices por raio
e corta em `MAX_TIPS = 12`. Numa fixtura com um anel de base de **12** vértices de raio
igual (a lei aceita empate: `r[j] <= r[i]`), os doze ficaram à frente e **o espinho era o
13.º** — a medição devolvia `12` pontas, todas a zero, e o gate dizia que a peça estava
impecável. Com `8` no anel o gate passou a medir o que promete.

**Why:** é a família do balde vazio ([[feedback-a-bucket-nobody-fills-reads-as-perfect]]) —
«não medido» e «perfeito» são o mesmo byte —, mas a causa é outra: aqui o balde existe e
foi **ocupado por empatados**.

**How to apply:** ao pôr um `take(N)`/`truncate(N)` numa régua, escreva ao lado *o que
acontece se `N` elementos empatarem à frente*, e faça a saída dizer **quantos** foram
medidos (nunca só a média). No gate, escolha a fixtura de modo que a população de interesse
não possa ser empurrada para fora — e afirme a contagem, não só o valor.

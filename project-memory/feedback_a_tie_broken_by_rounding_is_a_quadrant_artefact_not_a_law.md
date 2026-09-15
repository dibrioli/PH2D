---
name: feedback-a-tie-broken-by-rounding-is-a-quadrant-artefact-not-a-law
description: Um empate resolvido por `round` não é uma decisão — é um artefacto do quadrante, e ele passa em todo gate até o dono o encontrar.
metadata:
  type: feedback
---

Medido 2026-09-15 (`line/components`, TOP-20 #13, ordem do dono depois do smoke aprovado: *«no 4 dir,
mesmo com duas setas pressionadas, a última a ser pressionada sempre é dominante»*).

Com duas setas em baixo a intenção é **exactamente diagonal**, e o quantizador resolvia-a por
`roundf(ang / 90°)` — que parte empates **para longe do zero**. Resultado, por quadrante:
`→+↑` dava **cima**, `→+↓` dava **baixo**, `←+↑` e `←+↓` davam ambos **esquerda**. *Duas verticais,
duas horizontais, e nenhuma escolhida por ninguém.*

**Why:** um `round` sobre um valor exactamente a meio caminho **sempre** devolve alguma coisa, e essa
coisa lê-se como uma decisão. Nenhum gate a apanha, porque nenhum gate sabe que ali havia uma
pergunta: a saída é determinística, é contínua com a vizinhança, e o único sítio onde o defeito
aparece é o **dedo do artista**. ⚠️ E a forma esconde-se melhor quando o artefacto acerta em parte do
corpus — aqui metade dos quadrantes dava a resposta «plausível».

**How to apply:** sempre que uma lei tiver um **caso de empate exacto** (um `round` a meio, um
`>=` contra um `>`, um `sort` instável, dois candidatos com o mesmo custo), pergunte *quem escolheu
isto?* — e se a resposta for «o arredondamento», **declare** a escolha com um nome e um gate, nos
QUATRO casos simétricos e não em dois. ⚠️ Uma cura que devolva o resultado por outra aritmética cria
**duas respostas para o mesmo rumo** (aqui: `libm::cosf(π/2)` é `−4,4e-8`, não zero) — a lei nova
reescreve a **entrada** do cálculo que já existe, nunca a saída. Relacionado:
[[reference_topic_measurement_discipline]] · [[reference_topic_control_design_hazards]] ·
[[feedback_an_inequality_accepts_a_whole_interval_only_an_oracle_accepts_an_answer]]

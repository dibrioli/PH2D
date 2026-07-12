---
name: feedback_loose_oracle_hides_systematic_bias
description: "Teste com tolerância da ordem do próprio efeito não distingue \"funciona\" de \"funciona mas está sistematicamente errado\" — meça na unidade que o usuário percebe e fixe o valor exato"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: abd9ab40-c406-476a-84e5-a884bdcf3d74
---

Um oráculo cuja **folga é da ordem de grandeza do efeito** passa verde com um **viés
sistemático** dentro. O pitch shifter do áudio passou o próprio teste por 3 jornadas
estando **54 cents baixo** numa oitava: o teste media taxa de cruzamento por zero e
aceitava `up > dry * 1.6` para uma oitava (que deveria dar exatamente 2.0) — a realidade
de 1.94× passava folgada. Só apareceu quando o Harmonizer (que toca **acordes**) tornou o
erro audível; numa voz de monstro ninguém percebia.

**Why:** verificar "mudou na direção certa" é mais fácil de escrever do que "mudou a
quantidade certa", e a tolerância generosa parece prudência (evita flake). Mas ela é
exatamente o espaço onde um erro de escala/viés se esconde — e o proxy escolhido
(cruzamentos por zero) era insensível justamente ao que o usuário ouve (afinação). O bug
sobreviveu a smoke, CI e três auditorias porque o teste **provava a propriedade errada,
com folga**. Ver [[feedback_oracle_must_model_appearance_not_implementation]] e
[[feedback_no_industrial_claims_without_verification]].

**How to apply:** para qualquer quantidade derivada (frequência, tempo, ganho, posição),
asserte o **valor exato esperado** na **unidade que o usuário percebe** (cents, ms, dB,
px) com tolerância **muito menor** que o efeito — não uma faixa proporcional a ele. Se a
tolerância precisa ser folgada pra não flakar, o problema é o oráculo (proxy errado), não
o alvo: troque o proxy. E quando um teste passa com folga suspeita, **meça o valor real e
compare com o previsto pela teoria** — os 4 pontos medidos batendo com a fórmula do erro
foi o que provou a causa-raiz em vez de chutar. Corolário: **ferramenta condicional
precisa de fixture que satisfaça a condição** — um de-clicker só deixa marca em áudio com
clique, então a probe compartilhada tem que conter dano, senão o único jeito de o teste
passar é o efeito borrar áudio íntegro.

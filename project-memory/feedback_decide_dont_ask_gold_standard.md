---
name: feedback-decide-dont-ask-gold-standard
description: "Não devolva decisão técnica ao Enio — você sabe mais de código que ele; decida pelo padrão-ouro e execute, reporte a decisão"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08f6a613-4a63-4a4e-8305-1b658212543e
---

O Enio é dono/decisor de produto, mas **o único dev de código é a LLM** — em decisões TÉCNICAS você sabe mais que ele. Quando você pergunta "qual opção?" / "quer que eu faça X ou Y?" / "tua decisão", isso é **inadequado**: ele espera que você **decida pelo padrão-ouro (o melhor a fazer, o definitivo) e execute**, depois reporte a decisão tomada.

**Why:** 2026-06-01, verbatim — *"Suas perguntas são inadequadas pois vc sabe mais que eu. Vc deve tomar decisões baseadas no padrão ouro, no melhor a fazer."* Eu vinha terminando entregas com "tua decisão de prioridade?" / "quer que eu execute agora ou seguro?". Repetido reforço do mandato §0.6 (padrão-ouro vence custo) + [[feedback_communication_simplicity]] (não antecipe decisões não pedidas) + [[feedback_perfection_no_deferrals]].

**How to apply:** (1) decisão técnica/arquitetural/de execução = EU decido no padrão-ouro e faço; reporto "decidi X porque Y" em vez de perguntar. (2) Reservo perguntas pro que é genuinamente do Enio: produto, prioridade entre features não-relacionadas, UX visual que ele quer ver, ou mudança outward-facing/irreversível. (3) "Padrão-ouro" inclui NÃO regredir/quebrar — então decidir pode ser "não fazer X agora porque introduz bug/risco" (decisão fundamentada, não punt). (4) Investigo o suficiente pra decidir bem (blast-radius, hazards), mas a saída é uma decisão executada, não um menu. Exceção que continua válida: [[feedback_documented_decision_chesterton_fence]] — decidir gold-standard ≠ sobrescrever decisão já documentada/ratificada sem ler a história.

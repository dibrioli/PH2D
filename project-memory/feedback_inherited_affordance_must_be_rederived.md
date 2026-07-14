---
name: feedback-inherited-affordance-must-be-rederived
description: "Copiar uma affordance de UI da feature vizinha \"por analogia\" produz bug que passa em TODO gate — re-derive do que a coisa nova É"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 362c6c4f-9b8e-4ef4-b261-2d5564753f1a
---

O Sculpt herdou o **"Adjust Last Stroke"** do impasto: os knobs do card (Radius, Smooth↔Sharpen)
re-renderizavam o traço já feito. Passou em 15 gates, incluindo um gate dedicado, bem escrito, com o
vermelho provado por mutação. O Enio derrubou no 1º smoke, em uma frase: pegar **Sharpen** — pra afiar em
*outro lugar* — convertia o Smooth que ele acabara de fazer no **oposto** dele.

**Why:** a analogia não valia, e eu não a re-derivei — herdei.

- Tinta é uma **substância**. Depth/Body são propriedades *da tinta que aquele traço depositou*, então
  "continue afinando" é uma oferta coerente.
- Um traço de sculpt é uma **operação**. Não deixa pra trás nada que tenha propriedades — só o relevo, como
  está agora. Não existe "o smoothing" parado ali pra ser re-parametrizado. Operações se **desfazem**.

**How to apply:** ao reusar uma affordance da feature vizinha, re-derive do que a coisa **NOVA é**; não
herde do que a velha faz. Dois testes afiados, nesta ordem:

1. **O knob descreve algo que ainda EXISTE?** "Depth" descreve a tinta que está lá. "Raio do smooth"
   descreve um evento que já acabou. Se o referente do knob não existe mais, o knob está reescrevendo
   história.
2. **O knob escolhe QUAL FERRAMENTA?** Parâmetro retroativo é discutível; **verbo retroativo nunca é
   ajuste — é destruição**. Selecionar uma ferramenta jamais pode alterar trabalho já feito.

E a meta-lição, que é a mais cara: **um gate verde, mutation-proven, pode pinar um bug de DESIGN.** Gates
provam que o código faz o que você **disse**; nenhum gate te diz que o que você disse está errado. O smoke
do Enio é o único oráculo pra isso — não é etapa opcional, é a única lente que enxerga essa classe.

Relacionadas: [[feedback_ergonomics_verdict_is_a_design_bug]] (o veredito dele sobre ergonomia é um bug de
design, não de calibragem) · [[feedback_convention_vs_inertia]] (checar se "convenção" tem razão ou é
inércia) · [[feedback_smoke_at_end]].

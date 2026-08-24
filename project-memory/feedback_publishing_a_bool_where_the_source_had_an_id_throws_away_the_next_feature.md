---
name: feedback-publishing-a-bool-where-the-source-had-an-id-throws-away-the-next-feature
description: Um shell que publica «há?» onde a fonte tinha «qual?» apaga a identidade, e a metade apagada é a que a feature seguinte pede
metadata:
  type: feedback
---

O shell do PH2D publicava `has_profile: bool` — *"há um contorno fechado escolhido?"* — e bastava
para os botões `+ Extrude`/`+ Revolve` aparecerem. O gesto seguinte (**religar** uma forma a outro
desenho) precisa do **id**, e ele **não é redescobrível do lado de lá**: quem drena as intenções
recebe o mundo ECS, nunca a cena vetorial.

**Why:** a travessia que responde «há?» já teve «qual?» nas mãos e deitou-o fora. O custo de o
guardar é um `Option<u64>` em vez de um `bool`; o custo de o deitar fora é uma ponte nova quando a
próxima feature o pedir.

**How to apply:** quando um shell publicar um predicado derivado de uma **identidade**, publique a
identidade e derive o predicado (`pick.is_some()`). Mesma família de
[[feedback-a-parameter-that-changes-nothing-is-discarded-downstream]] pelo avesso: ali o valor
morria a jusante, aqui a identidade morre na ponte.

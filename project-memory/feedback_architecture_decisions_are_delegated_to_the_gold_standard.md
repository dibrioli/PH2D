---
name: feedback-architecture-decisions-are-delegated-to-the-gold-standard
description: Enio (12/09) delegou as decisões TÉCNICAS/de arquitectura — «eu não decidirei nada, vc sabe mais que eu; qual o padrão ouro? vamos até o estado da arte» — decida pelo padrão-ouro, execute e reporte; não devolva escolhas técnicas como perguntas
metadata:
  type: feedback
---

Quando uma auditoria ou um plano chega a um ponto de **desenho técnico** (camadas, onde mora um tipo,
lints, gates, registos, partir crates), **não o devolva ao Enio como decisão**: escolha o padrão-ouro,
diga numa linha qual foi e porquê, e execute. Ele respondeu à auditoria de arquitectura de 12/09, que
lhe listava «quatro decisões que são tuas», com: *«eu não decidirei nada. Vc sabe mais que eu! Qual o
padrão ouro? Vamos até o estado da arte. Mas quero tudo no melhor estado antes de enviar»*.

**Why:** ele é o dono do PRODUTO, não o engenheiro (CLAUDE.md §0.8); pedir-lhe que arbitre a direcção
de uma dependência é pedir o que ele não tem como avaliar, e atrasa a obra.

**How to apply:**
- Técnico/arquitectura/processo ⇒ decido eu, pelo padrão-ouro ([[feedback-perfection-no-deferrals]]).
- ⚠️ **Continua dele:** o que muda o que se VÊ ou SENTE no app (vai para o smoke, §0.8), e o **envio**
  (push) — só por ordem explícita (§0.7). *«Antes de enviar quero tudo pronto»* não é ordem de enviar.
- Uma decisão minha que contrarie uma recusa medida ou uma ordem de produto anterior dele **não** é
  coberta por esta delegação: essas continuam de pé.

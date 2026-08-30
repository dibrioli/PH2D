---
name: a-stage-ends-in-a-smoke-and-complexity-is-audited-before-it
description: Enio 2026-08-30 — pare de entregar em micro-passos; cada ETAPA termina num smoke, e uma etapa com qualquer complexidade é AUDITADA antes de o smoke ser sugerido.
metadata:
  type: feedback
---

**Enio, 2026-08-30, ao reabrir uma linha** (a instrução chegou a mais de uma no mesmo dia): *«Dessa vez não deve ser em micro passos. A
partir de agora a implementação deve ser de tal forma que cada etapa deve ao fim ter um smoke. Se a
etapa tiver qualquer complexidade, deve ser auditada antes de sugerir o smoke.»*

A unidade de entrega deixa de ser a fatia técnica e passa a ser a **etapa que ele consegue TOCAR**.

**Why (MEDIDO, e são duas medições de dias diferentes):** na jornada de **29/08** houve **quatro
rondas de report do mesmo módulo, e em três delas a causa era um defeito que uma auditoria teria
apanhado antes**; na de **27/08** fecharam-se sete fatias com sete «siga», e **três** voltaram como
report. *Cada ronda que ele gasta a descobrir o que uma auditoria já saberia é tempo dele pago por
economia minha.* O custo de uma entrega não é o meu trabalho — é o **turno dele**. Uma fatia que fecha sem
smoke gasta a atenção dele para dizer *«ok, siga»* sobre uma coisa que ele não viu; e um smoke
sugerido sobre trabalho não auditado gasta-a duas vezes, porque o defeito volta como report. A
jornada de 27/08 fechou **sete** fatias com sete «siga» — e três delas voltaram como report do Enio
([[feedback_communication_simplicity]], [[user_role]]: ele é o dono que TESTA, e o smoke é onde ele
aprende a ferramenta).

**How to apply:**
1. **Dimensione a etapa pelo GESTO**, não pelo diff: ela acaba quando existe uma frase do tipo
   *«faça X e veja Y acontecer»*. Se não existe essa frase, a etapa não acabou — continue.
2. ⛔ **Não peça «siga» a meio.** Vários commits internos continuam certos; o que não pode é
   devolver-lhe o turno sem um smoke na mão.
3. ⚠️ **O GATILHO da auditoria é concreto, e é generoso de propósito:** a etapa tocou **mais de um
   subsistema**, mudou um **default**, ou tem um **número escolhido**. Qualquer um dos três ⇒ audite.
4. ⚠️ **Complexidade ⇒ auditoria ANTES do smoke** (`/pd-auditoria`, ≥2 lentes: correcção ·
   costura de UI). O gatilho é *«qualquer complexidade»*, e ele é generoso de propósito — na
   dúvida, audite. O que a auditoria procura é o que o smoke dele encontraria: dois sítios que
   devem concordar sobre um facto e discordam · um controlo pintado e morto sob o dedo · uma
   sequência que não leva a lado nenhum.
5. **A auditoria LISTA antes de consertar** (a lei do `/pd-auditoria`), e o que ela achar entra na
   mesma etapa — não vira um «siga» extra.
6. O smoke continua no formato do [[feedback_run_command_include_cd]] e do §0.8: passos numerados,
   comando completo com `cd`, o nome que aparece **na tela**, e como saber que deu errado.

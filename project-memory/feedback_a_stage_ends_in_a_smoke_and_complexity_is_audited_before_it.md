---
name: a-stage-ends-in-a-smoke-and-complexity-is-audited-before-it
description: Enio 2026-08-30 — pare de entregar em micro-passos; cada ETAPA termina num smoke, e uma etapa com qualquer complexidade é AUDITADA antes de o smoke ser sugerido.
metadata:
  type: feedback
---

**Enio, 2026-08-30, ao reabrir a `line/components`:** *«Dessa vez não deve ser em micro passos. A
partir de agora a implementação deve ser de tal forma que cada etapa deve ao fim ter um smoke. Se a
etapa tiver qualquer complexidade, deve ser auditada antes de sugerir o smoke.»*

A unidade de entrega deixa de ser a fatia técnica e passa a ser a **etapa que ele consegue TOCAR**.

**Why:** o custo de uma entrega não é o meu trabalho — é o **turno dele**. Uma fatia que fecha sem
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
3. ⚠️ **Complexidade ⇒ auditoria ANTES do smoke** (`/pd-auditoria`, ≥2 lentes: correcção ·
   costura de UI). O gatilho é *«qualquer complexidade»*, e ele é generoso de propósito — na
   dúvida, audite. O que a auditoria procura é o que o smoke dele encontraria: dois sítios que
   devem concordar sobre um facto e discordam · um controlo pintado e morto sob o dedo · uma
   sequência que não leva a lado nenhum.
4. **A auditoria LISTA antes de consertar** (a lei do `/pd-auditoria`), e o que ela achar entra na
   mesma etapa — não vira um «siga» extra.
5. O smoke continua no formato do [[feedback_run_command_include_cd]] e do §0.8: passos numerados,
   comando completo com `cd`, o nome que aparece **na tela**, e como saber que deu errado.

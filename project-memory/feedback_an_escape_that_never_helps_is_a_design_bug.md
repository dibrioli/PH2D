---
name: feedback_an_escape_that_never_helps_is_a_design_bug
description: Um botão de escape/override que produz lixo em 100% dos casos não é um escape — é um enfeite que engana; meça em vários casos antes de calibrá-lo.
metadata:
  type: feedback
---

Quando um controle de escape (override manual, "match", toggle) "tem resultados estranhos", **meça-o
em vários casos antes de tentar calibrá-lo** — pode ser que ele nunca devesse existir. Um escape que
produz lixo em 100% dos casos não é um escape: é um enfeite que engana o usuário.

**Caso medido (Blend do Vector, `ph2d-vec-blend`, 2026-07-14):** o **"Reverse Match"** invertia o
sentido de percurso da 2ª forma, o que inverte o **winding**, e interpolar entre windings opostos
**colapsa a forma no meio** (a área cruza zero). Todo o catálogo nasce com o mesmo winding, então o
botão colapsava em **3 de 3** pares testados (`min|área|` ~0,02-0,00). E era **redundante**: o motor já
escolhe o sentido de menor custo automaticamente (inclusive quando a 2ª forma tem winding oposto de
verdade). Nenhuma ferramenta profissional (Illustrator, GSAP) tem um "reverse" que inverte winding —
elas detectam a direção sozinhas. **Removido**, não calibrado.

**Por que:** é a regra [[feedback_ergonomics_verdict_is_a_design_bug]] ("difícil de ajustar = bug de
design; pare de calibrar, questione o modelo") levada ao limite — às vezes o modelo certo é *não ter o
controle*. O sinal foi a **MEDIÇÃO** em vários casos (o padrão só apareceu com 3 pares), não a leitura
do código. E o corolário oposto vale para o Rotate, que FICOU: o quantum dele vinha das âncoras
arbitrárias da forma lisa (colapsava no 2º toque) e passou a vir das **quinas** (passos finos) — um
escape útil merece um quantum que reflita a estrutura que o usuário percebe.

**How to apply:** antes de mexer num override que "às vezes dá errado", monte 3-5 casos
representativos e meça a saída. Se ele falha em todos (ou se o caminho "bom" já é escolhido
automaticamente noutro lugar), a resposta é removê-lo — e ao remover, varra os comentários/docs/i18n
órfãos ([[feedback_stale_comment_and_dead_code_lie]]), que a remoção de código deixa para trás.

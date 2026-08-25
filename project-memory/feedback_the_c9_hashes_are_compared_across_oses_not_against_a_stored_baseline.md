---
name: feedback-the-c9-hashes-are-compared-across-oses-not-against-a-stored-baseline
description: Os hashes C9 (replay/física/bridge) não têm baseline gravado — o CI compara os 3 SOs entre si; mudar o VALOR é inofensivo, o que reprova é os 3 discordarem
metadata:
  type: reference
---

Os três gates de determinismo do `spike.yml` — `c9_replay` (ECS) · `ph2d_physics_c9` (rapier) ·
`physics_ecs_c9` (a ponte, ADR-0131 W1) — funcionam assim: cada SO **corre o binário, imprime o
hash e sobe-o como artefacto**; o job `determinism-compare` faz `sort -u | wc -l` e reprova se
der `!= 1`.

⇒ **Não existe baseline gravado em lado nenhum.** Verificado por `grep` em `crates/`, `shells/`
e no workflow: nenhum `assert`, `const` ou ficheiro pina o valor — o binário só faz
`println!("physics-ecs-c9 hash: {hex}")`.

**Why:** uma mudança de formato que altere o valor do hash — como o `WorldSnapshot` v1→v2 da
`line/components` — **não reprova o CI**. O handoff dela avisava, correctamente, que o hash
mudava de valor, e concluía que era *«o item mais provável de partir o CI»*. O facto estava
certo; a consequência não. *Um aviso pode estar certo sobre o mecanismo e errado sobre o que
ele causa* — e um bloqueador fantasma custa a mesma atenção que um real.

**How to apply:** ao integrar algo que mexa no snapshot, na ordem de iteração ou na fronteira
metros↔rapier, **não perca tempo a «re-capturar» o hash** — não há o que recapturar. O risco
real é outro e só o CI o mede: que a mudança tenha introduzido **divergência ENTRE SOs**
(iteração não-determinística, ordem de `HashMap`, formatação de float). Isso aparece como
`FAIL: … hashes differ across OSes` no `determinism-compare`, e aí o `BTreeMap`-nunca-`HashMap`
do módulo é o primeiro sítio a olhar.

Irmãs: [[reference_topic_process_cadence]] ·
[[feedback_a_deferral_notes_bar_may_exceed_the_projects_policy]] (a nota de um handoff é uma
AFIRMAÇÃO — confira-a antes de a herdar).

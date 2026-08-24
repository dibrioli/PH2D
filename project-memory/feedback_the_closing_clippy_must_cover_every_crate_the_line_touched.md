---
name: feedback_the_closing_clippy_must_cover_every_crate_the_line_touched
description: O clippy do gate de fechamento tem de cobrir TODA crate que a linha tocou — rodá-lo só na shell deixa um latente que o integrador paga
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-23T02:29:23.074Z
---

No fechamento da `line/motion-value` (2026-08-22) corri
`cargo clippy -p ph2d-host-desktop -p ph2d-node-registry-init --all-targets` e chamei-lhe verde.
A linha tinha editado **~40 crates `ph2d-node-*`**. O `ship.sh` do integrador encontrou um
`needless_range_loop` em `ph2d-node-pulse-beat` — e ele ficou escrito no commit de drenagem:
*"a linha só corria clippy no shell"*.

**Why:** o gate de fechamento existe para que o integrador não descubra nada. Um clippy com `-p`
escolhido a dedo mede *as crates de que me lembrei*, não *as que toquei* — e a lista de que me
lembro é sempre a das duas em que estive a depurar por último. O custo não é o lint: é o
integrador a parar uma fusão de cinco linhas para consertar código meu, com a minha worktree já
fechada.

**How to apply:** derive o alvo do DIFF, nunca da memória. No fecho:

```bash
B=$(git merge-base main HEAD)
git diff --name-only $B..HEAD | grep -oE '^(crates|shells)/[^/]+' | sort -u \
  | sed 's|.*/||' | sed 's/^/-p /' | xargs cargo clippy --all-targets
```

⚠️ **A mesma lei vale para o `typos`**, que na mesma integração deu 16 falsos positivos das cinco
linhas — *nenhuma* corria o scan project-wide. Ver
[[reference_topic_process_cadence]] e [[project_integrator_ship_catches_latents_budget_iterations]].

*O gate de fechamento mede o que a linha TOCOU; se o alvo dele é escrito à mão, ele mede a minha
memória.*

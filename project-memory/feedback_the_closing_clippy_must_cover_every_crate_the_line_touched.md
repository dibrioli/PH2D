---
name: feedback_the_closing_clippy_must_cover_every_crate_the_line_touched
description: O clippy do gate de fechamento tem de cobrir TODA crate que a linha tocou — rodá-lo só na shell deixa um latente que o integrador paga
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-26T18:55:38.085Z
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

⚠️⚠️ **E o alvo certo ainda não basta: tem de ser o COMANDO do ship, com `-D warnings`.**
Medido na `line/components` (2026-08-26): correr `cargo clippy -p <as crates do diff> --all-targets`
imprimiu 5 lints — e eu li «verde» porque o exit code foi **0**. O `ship.sh` corre
`cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`, onde os
mesmos 5 são **erros**: 3 em ficheiros que esta linha CRIOU (`incremental.rs`, `component_seed.rs`,
`project_migrate_sprite.rs`) e 2 em ficheiros que ela modificou. Três fatias tinham fechado assim.
*Um lint sem `-D warnings` não reprova nada, e um gate que não reprova não é um gate.*
⇒ no fecho, copie a linha do `ship.sh` em vez de compor uma.

⛔⛔ **TERCEIRA ocorrência, na MESMA linha, 2026-09-09** — e desta vez a regra já estava escrita
aqui. Os ciclos 3, 4 e metade do 5 da `line/motion-value` fecharam com `cargo test --bins` e
`cargo clippy -p <a crate em que eu estava>`; ao correr enfim a linha do `ship.sh`
(`--workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`) apareceram **sete**
reds acumulados, **todos em código desta linha**: um `&&` cuja metade direita não tinha efeito
(um controlo de gate que assim nunca teria falhado), duas closures redundantes, um `if`
colapsável, um empréstimo desnecessário, um `map` da identidade, uma linha em branco entre um doc
e o seu item — e a `kernel_module` a **oito** parâmetros sobre um teto de sete, empurrada até lá
pelo `shared` do ciclo 3.

⚠️ **Dois deles não eram cosmética.** O `&&` (`4,5 < bar && 3,5/√2 < bar`) colapsava **dois
controlos de grandezas diferentes** num só: a segunda metade é logicamente implicada pela
primeira, então o dia em que o losango deixasse de reprovar o gate não se movia — *o clippy estava
a nomear um gate meio morto, não um estilo*. E o teto de argumentos cobrou o corte que a casa
manda (`ExtraBuffers`, um tipo para os três fornecedores de buffer que já viajavam sempre juntos),
que de caminho **reforçou** uma garantia que o doc daquela função já reivindicava por escrito.

⇒ *o `-D warnings` do fecho não é higiene: ele encontra gates que não julgam nada.*

*O gate de fechamento mede o que a linha TOCOU; se o alvo dele é escrito à mão, ele mede a minha
memória.*

## ⚠️⚠️ A OUTRA FACE — cobrir as crates certas ainda dá o número ERRADO (2026-09-12)

Na integração da W2 Fase D o integrador contou os avisos que travavam o `ship.sh` crate a crate e deu
ao dono **dois números errados seguidos** (*«sobra 1»*, depois *«são 25»*). Duas causas, opostas:
- `cargo clippy -p <crate>` **desliga as features que a shell liga** ⇒ **inventa** avisos: 7 na
  `ph2d-app-vec`, todos parâmetros usados só dentro de `#[cfg(feature = "panel-vector")]`;
- e **não vê** as crates que não nomeia ⇒ **escondeu 19** na `ph2d-app-motion`.
**Why:** o clippy por crate mede uma configuração de features que nenhuma build real produz — em
workspace a unificação de features liga-as, e é essa a que o CI corre.
**How to apply:** para CONTAR avisos (e para dizer a alguém quantos faltam), só vale a linha do
`ship.sh`: `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`.
O `-p` derivado do diff, acima, serve para o inner loop — não para um veredito. (HOWTO §2.17.)

## ⛔ E O MACHETE TAMBÉM É DO FECHO — mudar código de casa deixa a linha do `Cargo.toml` (2026-09-12)

As mudanças de casa da W2 deixaram **79 dependências declaradas e não usadas** (62 na shell, 17 nas
famílias), e **nenhuma** das linhas correu `cargo machete` ao fechar — quem as achou foi o `ship.sh` do
integrador. A DIRETRIZ §1.5.9 já pedia *«deps novas p/ machete»* no handoff; o comando que as linhas
de facto EXECUTAM ao fechar (`/pd-linha-fechar`) só corria testes e clippy. *Ponteiro não é adoção* —
a lei do CLAUDE.md §2, outra vez.
**How to apply:** o `/pd-linha-fechar` chama agora o `cargo machete` pelo nome. Ao triar, verifique
antes de apagar: (1) a linha liga `features =` numa biblioteca partilhada? (apagar desliga-a em
silêncio para o programa inteiro); (2) é só alvo de `dep:x` numa feature? (sai o elemento, a feature
fica); (3) o comentário por cima vai junto — num `Cargo.toml` ele pertence à declaração de baixo e,
deixado, passa a parecer explicar a seguinte. HOWTO §2.18.

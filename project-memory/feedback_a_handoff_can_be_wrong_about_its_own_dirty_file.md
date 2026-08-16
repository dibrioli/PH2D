---
name: feedback-a-handoff-can-be-wrong-about-its-own-dirty-file
description: Uma instrução de "NÃO toque neste arquivo" num handoff é uma AFIRMAÇÃO — meça-a antes de a honrar
metadata:
  type: feedback
---

Um handoff de integração pode errar sobre o **próprio** estado da linha, e a
instrução mais perigosa é a que manda **não** mexer em algo: ela não produz
falha, produz silêncio.

**Caso medido (2026-08-16, integração da `line/motion-value`):** a §6.1 do
handoff dizia que `crates/ph2d-node-pulse-signal/src/tests.rs` estava sujo na
worktree com *"um diff de formatação pura de OUTRO dono, deixado
deliberadamente fora dos 162 commits"*, e mandava **não o stagear**. Medido, o
veredito **inverte** nos três pontos:

- aquela crate é uma das **sete crates novas da própria linha** — não é de
  outro dono, é dela;
- a versão **COMMITADA reprova** em `rustfmt --check` e a **suja PASSA**;
- e o `rustfmt` reproduz o arquivo sujo **byte a byte**.

Não era estilo alheio: era **a correção que faltava ao tip**, e honrar a
instrução teria deixado o `main` fmt-vermelho — com o `fmt --check` sendo o
PRIMEIRO gate do `ship.sh`.

**Why:** um handoff é escrito pela linha sobre si mesma, no fim de uma jornada
longa, e as afirmações sobre *proveniência* ("isto é de outro dono") são
exatamente as que ninguém re-mede. Uma instrução de não-tocar transfere a
decisão sem transferir a evidência.

**How to apply:** toda cláusula de handoff da forma *"não mexa em X"* / *"X é de
outro dono"* / *"X foi deixado de fora de propósito"* é uma **afirmação
verificável**, e o integrador a verifica antes de a honrar — quem criou o
arquivo (`git log --diff-filter=A`), e o que a ferramenta relevante diz sobre a
versão commitada **contra** a suja. O custo é um comando; o custo de acreditar é
um vermelho que só o ship vê.

Irmão de [[feedback_a_deferral_notes_bar_may_exceed_the_projects_policy]] (a
nota de diferido que não é spec) e de
[[feedback_a_silenced_instrument_reads_as_a_result]].

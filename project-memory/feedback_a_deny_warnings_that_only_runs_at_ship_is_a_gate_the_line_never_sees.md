---
name: feedback-a-deny-warnings-that-only-runs-at-ship-is-a-gate-the-line-never-sees
description: Warnings de clippy acumulam-se em waves porque o `-D warnings` só vive no ship.sh, e o portão de fecho de uma linha corre clippy sem ele
metadata:
  type: feedback
---

O portão de fecho da wave anterior desta linha deu verde, e a wave seguinte encontrou **quatro**
avisos de clippy dela: um `..Default::default()` inerte, dois imports mortos, um `if/return None` que
é um `?`, e uma função que chegou a oito argumentos. O `ship.sh` corre
`cargo clippy --workspace --all-targets -- -D warnings`; um `cargo clippy` corrido à mão pela linha
**não** tem o `-D`, e um aviso lê-se como ruído de build.

**Why:** a linha fecha com «verde», o integrador herda vermelhos, e quem os paga é a jornada seguinte
— exactamente o mesmo mecanismo de *«um portão que só corre as crates EDITADAS é cego a todo
espelho»*, uma volta acima.

**How to apply:** no portão de fecho de uma linha, corra clippy **com o `-D warnings` e com
`--workspace --all-targets`**, que é a forma exacta que o ship vai usar. É uma corrida, e ela
distingue «não há avisos» de «há avisos que eu não estou a ler». Ver
[[reference-topic-ship-ci-integration-lessons]] e [[feedback-a-bins-run-never-reaches-the-gates-that-live-in-tests]].

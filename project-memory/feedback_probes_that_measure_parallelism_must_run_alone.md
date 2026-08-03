---
name: feedback-probes-that-measure-parallelism-must-run-alone
description: "Sondas de medição rodando concorrentes disputam o MESMO pool de threads — cada uma passa a medir o agendador das outras, e o número sai inflado sem nada parecer errado"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 39ec3808-26ec-4cf4-b80e-b2291882bc64
  modified: 2026-08-02T14:02:14.934Z
---

Um filtro de teste que casa com **várias** sondas de medição as roda em paralelo (o default do
`cargo test`), e elas **disputam o mesmo pool global do rayon**. Cada uma deixa de medir o próprio
kernel e passa a medir a contenção com as irmãs — números inflados, sem erro, sem flake, sem nada
parecer errado. **`--test-threads=1` não é higiene: é parte da FIXTURE** sempre que o que se mede é
paralelismo ou wall-clock.

**Caso medido (PH2D, o pen-up do impasto, 2026-08-02):** rodar `measure_stroke_extent::` (três sondas)
num filtro só deu pen-up **496 ms** e commit **310**; isoladas com `--test-threads=1`, **424** e **276**.
Eu ia publicar a primeira tabela.

**Why:** o pool do rayon é global ao processo, então duas sondas paralelas não são dois experimentos —
são um experimento com uma variável escondida. É o irmão exato de *"nenhum smoke desta máquina significa
nada com o load acima de ~5"*: ali a carga vem de fora, aqui a sonda a cria sozinha.

**How to apply:**
1. Toda sonda de tempo roda com **`--test-threads=1`**, e o comando na doc dela já traz o flag.
2. **O detector é o CONTROLE INTERNO** — uma célula da tabela cujo valor é conhecido de corridas
   anteriores (aqui, o traço curto de 200 px). Se ele se moveu, a corrida inteira vai fora, não só a
   linha suspeita (ver [[reference_topic_repro_discipline]]).
3. Vale também para `cargo test` de crates diferentes encostados no mesmo lote: rode a suíte, **depois**
   a medição.

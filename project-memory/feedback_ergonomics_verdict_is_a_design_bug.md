---
name: feedback-ergonomics-verdict-is-a-design-bug
description: "'Ficou mais difícil de ajustar' é veredito de DESIGN, não pedido de calibragem — pare de corrigir sintomas e questione o modelo"
metadata:
  type: feedback
---

Quando o Enio olha uma feature e diz **"não sei se melhorou ou piorou; ficou mais difícil de
ajustar"**, ele NÃO está pedindo para você mexer nos números. Ele está dizendo que a feature está
**tecnicamente correta e ergonomicamente errada** — e nenhuma calibragem conserta isso.

**Why:** no Impasto (2026-07-12) eu passei cinco rodadas corrigindo sintomas, cada foto dele revelando
o próximo defeito real. Todos os fixes eram legítimos e todos tinham gate. Mas a superfície de knobs
cresceu ACOPLADA (Depth e Amount escalam a mesma percepção; Smoothing e a maciez do falloff borram a
mesma coisa) e o modelo de depósito — que eu **inventei sem referência** — herdava o perfil MACIO da
cor, então o relevo saía um domo em vez de um corpo com borda. Nenhum knob conserta um perfil errado.

**How to apply:** ao ouvir esse veredito, PARE de implementar. (1) Liste os knobs expostos E as
constantes que você calibrou na mão — se duas mexem na mesma percepção, o modelo está acoplado.
(2) Pergunte se você INVENTOU o modelo ou o portou de uma referência; se inventou, pesquise como o
estado-da-arte faz. (3) Proponha o MENOR conjunto ortogonal de controles, e mate o resto. Só então
volte a codar.

Corolário: [[feedback_measure_perf_symptom_scale]] vale para ergonomia também — meça a coisa certa. Duas
vezes nessa linha eu medi errado e afirmei com confiança (autocorrelação num lag onde todo campo suave
correlaciona; periodicidade na linha central, onde o falloff é platô e não escalona). **Quando o Enio vê
com os olhos o que seu número não vê, o número está medindo a coisa errada.**

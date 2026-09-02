---
name: gates-that-all-read-size-leave-a-pose-feature-uncovered
description: "Todos os gates de um módulo podem medir a mesma grandeza e deixar a feature sem cobertura nenhuma — apagá-la deixou 85 testes e o clippy verdes"
metadata:
  type: feedback
---

**Medido por auditoria adversarial na `line/motion-value`, 2026-08-30.**

O `source.lsystem` tem uma feature chamada *Grow Angle*: a geração nova **abre as dobras** em
vez de saltar para o ângulo cheio. Uma mutação de uma linha
(`set.angle_frac` → `set.angle_frac.max(1.0)`) apaga-a por completo — e deixava **85 testes e o
clippy `-D warnings` verdes**, sem sequer um aviso de campo não lido.

**O mecanismo:** *todos* os gates liam a figura por um TAMANHO, e a feature é sobre a **POSE**.
A largura média é quase cega ao dobrar — medido, a âncora do Bush com as dobras fechadas dá
`0,333333` e com elas abertas `0,333289`: **`0,013 %`**. E o gate que existia com o nome do
interruptor (*«senão o interruptor é um knob morto»*) era satisfeito pelo **efeito colateral**
da normalização do COMPRIMENTO, nunca pelo ângulo. É a espécie do `CLAUDE.md §5.0`: *o
consumidor que projecta o valor fora.*

A cura é um gate cuja régua é a pose (a maior viragem que a figura contém), com as três
metades: ligado ela **abre** monotonamente · a excursão é **grande** (senão passava com um
milésimo) · desligado ela é a cheia **byte a byte**.

**Why:** contar gates não mede cobertura. Uma suíte inteira pode ser uma só pergunta feita de
`N` maneiras.

**How to apply:** ao fechar uma wave, liste as GRANDEZAS que os gates lêem, não os gates. Se a
feature produz uma grandeza que nenhum gate lê, ela não tem cobertura — mesmo com 85 verdes.
E prove-o por mutação: apague a feature no CONSUMIDOR (não na assinatura, que o compilador
apanha) e veja quantos morrem.
Relacionado: [[feedback_a_dead_knob_has_two_species_no_probe_catches]] ·
[[reference_topic_mutation_proofs]] · [[reference_topic_gate_discipline]].

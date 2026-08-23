---
name: feedback_a_bucket_nobody_fills_reads_as_perfect
description: Uma régua que agrega (mediana/média) devolve ZERO quando o balde está vazio, e zero lê-se como «perfeito» — ponha a CONTAGEM ao lado, e nunca deixe um `continue` saltar a escrituração do laço
metadata:
  type: feedback
---

Uma régua que **agrega** — mediana, média, percentil — devolve `0` sobre um balde
**vazio**, e `0` é indistinguível de *«medi e está perfeito»*. **Imprima a contagem ao
lado do valor, sempre.**

⛔ **E a forma nº 1 de esvaziar um balde sem dar por isso é um `continue` a meio de um
laço cuja escrituração vive no fim.** Quem escreveu a contabilidade tinha um caminho em
mente; a ramificação seguinte salta-a, e nada avisa. *Use `else` — ele não tem como
armar a armadilha.*

**Why:** medido no `ph2d-quadfill` (2026-08-23). Dois campos separavam o enviesamento
por valência do patch; o caminho do patch de 4 lados saía do laço por `continue` antes
da escrituração ⇒ (a) o balde do domínio dos rectângulos ficava **sempre vazio** e
imprimia `0,0°`, (b) o vector de etiquetas não era estendido e as faces do rectângulo
eram rotuladas *leque* pelo primeiro leque a seguir. Sobre o `0,0°` escreveu-se uma
conclusão inteira — *«a grade do rectângulo nasce perfeita no domínio e chega torta à
superfície ⇒ são dois defeitos em duas fases»* — e um dia de hipóteses saiu dela. Os
números reais eram `1,0°` e `16°`, não `0,0°` e `12°`. ⚠️ A régua irmã até **trazia o
aviso no doc** («0 não é perfeito, é não medido») — para a coluna dela. *Um aviso escrito
ao lado de uma régua não protege a régua do lado.*

**How to apply:**
1. Toda mediana/média num relatório leva a **contagem da amostra** no mesmo struct e na
   mesma linha de log. Um inteiro a zero não se disfarça de bom resultado.
2. Uma função que recebe uma etiquetagem por índice **recusa alto** (`NaN`, `None`,
   `Err`) quando ela não cobre a colecção — nunca `unwrap_or(default)`, que atira os
   não-etiquetados para uma das colunas em silêncio.
3. Contabilidade no fim de um laço ⇒ **`else`, nunca `continue`**. E uma cerca
   (`debug_assert` de cobertura) por cima.
4. Antes de construir sobre um número surpreendente, **prove que ele foi medido** — o
   par numerador/denominador é o teste (`slid: 5/5`, `flattened: 19/19`).

Irmãs: [[feedback_an_unlabelled_probe_column_gets_read_backwards]] ·
[[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]] ·
[[feedback_a_new_features_gate_can_expose_a_pre_existing_bug_check_the_control_first]] ·
[[reference_topic_gate_discipline]]

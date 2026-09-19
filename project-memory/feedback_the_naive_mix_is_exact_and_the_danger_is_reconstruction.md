---
name: feedback-the-naive-mix-is-exact-and-the-danger-is-reconstruction
description: `(1−w)+w` é `1,0` exacto em f32; o perigo é reconstruir b por `a + (b − a)` com a ≠ b
metadata:
  type: feedback
---

**Ao escrever uma lei cuja identidade tem de ser exacta em `f32`, a forma perigosa
NÃO é `a·(1−w) + b·w`.** Ela é exacta no ponto neutro. O perigo é **reconstruir um
valor a partir de uma diferença**: `a + (b − a)` não devolve `b` quando `a ≠ b`.

**Caso medido (2026-09-19, `line/3DModeling`, a `W8` do render).** Escrevi no
cabeçalho de uma crate nova que a forma ingénua *«não serve, porque `(1−w) + w` não é
`1` para todo `w` em f32»*, e escrevi as três tintas na forma robusta por causa disso.
**Duas mutações que instalavam a forma ingénua SOBREVIVERAM** a 15 gates.

A varredura que se seguiu refutou a premissa:

```text
(1−w) + w  ==  1.0   em 2 044 824 amostras de f32 em [0,1]  e  200 000 acima
```

⇒ e tinha de dar: o erro daquela soma é no máximo **meia ULP** de `1`, e o desempate
em *round-half-to-even* escolhe `1,0`, cuja mantissa é par.

⭐ **A mutação que SANGROU é quem nomeia o perigo verdadeiro.** Ela estava na
saturação (`rgb·s + luma·(1−s)` trocado por `luma + (rgb − luma)·s`): ali os dois
extremos são **diferentes**, o parêntesis não é zero, e `s = 1` deixa de devolver a
entrada por cancelamento. Nas tintas os extremos são iguais no ponto de fábrica (as
duas brancas) e o parêntesis é **exactamente zero** — por isso as duas redacções são
igualmente exactas *ali*.

**Why:** a regra que eu tinha na cabeça (*«mix é impreciso»*) é aproximadamente
verdadeira e **precisamente errada**, e o que ela esconde é a distinção que interessa:
não é a forma do peso, é se o valor que se quer de volta é **multiplicado** ou
**reconstruído**.

**How to apply:** escreva a lei de modo a **multiplicar o valor que se quer de volta**
(`x·s + …`), nunca a reconstruí-lo por diferença. E quando um cabeçalho afirmar uma
propriedade de `f32`, **varra-a**: uma afirmação sobre aritmética é barata de medir
(2 milhões de amostras correm num teste) e cara de acreditar. ⚠️ Um gate de
identidade sobre a SAÍDA **não** prova a forma das expressões — foi por isso que ele
ficou verde sobre as duas redacções.

Irmão de [[reference_topic_measurement_discipline]] e de
[[reference_topic_mutation_proofs]].

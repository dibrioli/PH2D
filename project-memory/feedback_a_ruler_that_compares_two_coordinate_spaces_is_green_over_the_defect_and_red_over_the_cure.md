---
name: a-ruler-that-compares-two-coordinate-spaces-is-green-over-the-defect-and-red-over-the-cure
description: Um gate que compara geometria LOCAL com pontos de MUNDO pode passar por causa do defeito e falhar por causa da cura — e as duas leituras parecem igualmente convincentes
metadata:
  type: feedback
---

Caça ao bug #28 do vector (o Build duplicava a forma). A fixtura dos gates guarda a geometria em
**LOCAL** com a pose num `Xform` — que é como a Shape tool a deixa, e o próprio cabeçalho do
ficheiro celebra isso como *"a fixture é a do produto"*. As constantes de teste (`IN_PENT`,
`IN_STAR`) são pontos de **MUNDO**. O helper comparava as duas cruas:

```rust
let hits = |p| sc.paths().iter().filter(|q| contains_point(q, p)).count();
```

Consequências, todas simultâneas:

1. Ele **nunca via** o pentágono (local `x ∈ [−1,25; 1,25]` contra o ponto `−1,75`).
2. O que fazia `hits(IN_PENT) > 0` passar era **a sobra do rectângulo** — que nasce em coordenadas
   de mundo e cobria a área do pentágono. Ou seja: *o gate estava a PINAR o defeito*, e a cura
   correcta punha-o vermelho.
3. A 1.ª redacção do gate NOVO tinha a mesma doença ao contrário: acusou uma sobreposição de `3,19`
   entre duas formas que não se tocam, porque comparava uma nova (mundo) com uma sobrevivente
   (local).

**Why:** quando um sistema guarda geometria num espaço e nomeia pontos noutro, uma comparação crua
não dá erro — dá um número. Ele é plausível nos dois sentidos, e nenhuma das duas falhas se parece
com "a régua está errada": a primeira parece o produto correcto, a segunda parece um defeito novo.

**How to apply:** num teste de geometria, **escreva a conversão explicitamente uma vez** e passe
tudo por ela (aqui: `bake_xform(clone, xform_of(xf, id))`), inclusive as formas que o gesto acabou
de criar. E quando um gate existente ficar vermelho por causa de uma cura que você acredita estar
certa, **pergunte primeiro em que espaço ele mede** — antes de concluir que a cura está errada.

Relacionado: [[feedback_the_example_the_user_points_at_may_be_the_exception_of_its_family]] ·
[[reference_topic_measurement_discipline]] · [[reference_topic_fixture_discipline]]

---
name: feedback_two_copies_of_a_dependency_can_be_the_mechanism_not_the_residue
description: O cargo unifica features POR VERSÃO — duas cópias de uma dep podem ser a única forma de duas metades do repo terem políticas diferentes, e unificá-las impõe a política mais restritiva a todos
metadata:
  type: feedback
---

⭐⭐ **Duas cópias de uma dependência nem sempre são resíduo a arrumar.** O cargo unifica
features **por versão**, então duas versões são a única forma de duas metades do repo
terem **políticas diferentes** sobre a mesma biblioteca.

**Medido (2026-08-29).** A árvore tem `glam` **0.30.10** (as 8 crates de desenho) e
`glam` **0.33.6** (a física, via `glamx`). A cadeia
`rapier/enhanced-determinism → parry/enhanced-determinism → glamx/scalar-math →
glam/scalar-math` liga **SIMD off** na cópia da física, porque o determinismo entre
sistemas (HR-5) o exige. Unificar numa versão só imporia `scalar-math` a
`ph2d-core`, `-mesh-render`, `-vector`, `-anim`, `-vec-edit`, `-vector-font`,
`-vector-doc` e `-vector-traits`. ⇒ *as duas cópias são o que deixa a física ser
determinística e o renderizador ser rápido ao mesmo tempo.*

⭐ **E a recusa não precisou de um número de desempenho.** O outro lado da balança
CONTOU-SE: `Affine3`/`Vec3A` **0 usos**, `ISizeVec*` **0**, a correcção de
`escalar / matriz` **0**, e o `Vec2::angle_between` removido são **6 sítios a
reescrever** — ou seja *custo*. **Um ganho de zero perde para qualquer custo.**

⚠️ **Uma hipótese minha caiu por medição:** escrevi primeiro que o perigo era o `Vec3A`
encolher de 16 para 12 bytes e desalinhar buffers de GPU. Há **zero** usos de `Vec3A` no
repo. *Uma hipótese plausível sobre um tipo que ninguém usa mede zero.*

**Why:** a leitura por omissão de «duas versões da mesma crate» é *«limpar isto»*. Aqui,
limpar custaria o SIMD do renderizador inteiro por benefício medido nulo.

**How to apply:** antes de unificar versões, leia o **grafo de features** (não o
changelog) e pergunte qual política cada cópia carrega. Ver
[[feedback_the_newest_possible_is_not_the_newest_count_the_ceilings_first]].

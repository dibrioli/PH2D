---
name: feedback_a_source_parsing_gate_must_know_every_shape_of_what_it_parses
description: "Gate que lê o fonte assume as formas que existiam quando foi escrito; a forma NOVA fá-lo acusar o errado E ficar cego ao certo"
metadata:
  type: feedback
---

Medido 2026-08-30: um gate derivava a lista de variantes de um `enum` lendo o fonte, e
conhecia **duas** formas — unitária (`Nome,`) e tupla (`Nome(T)`). Ao entrar a primeira
variante com **campos nomeados** (`Nome { a: usize, b: usize }`) ele colheu `a: usize` e
`b: usize` como se fossem variantes e **perdeu a variante real**.

⛔ **O dano é duplo e a metade silenciosa é a pior:** ele reprovou a exigir uma propriedade
para um CAMPO (ruidoso, fácil de diagnosticar) **e** deixou de exigir a propriedade para a
variante nova (silencioso, e é o que o gate existia para garantir).

**Why:** um parser escrito por enumeração de formas conhecidas é um proxy da gramática, e a
gramática tem mais formas do que o autor tinha à frente. *A lista que ele devolve deixa de ser
a propriedade e passa a ser um artefacto do dia em que foi escrito.*

**How to apply:** ⛔ **não dobre o produto para caber no instrumento** — trocar a variante por
uma tupla faria o gate passar e é a decisão errada (dois `usize` na mesma ordem são o par que
se troca). ⇒ torne a regra **estrutural** (aqui: uma variante é uma linha no nível ZERO de
chavetas do corpo, começada por maiúscula) e ponha uma **fixtura de cada forma** no próprio
gate, para que a regressão reprove pelo nome da forma perdida em vez de por um sintoma
estranho. ⚠️ Ao contar delimitadores para achar o nível, ignore **comentários** — um `{` num
doc-comment desloca a profundidade e esconde tudo o que vem depois. Relacionado:
[[feedback_an_exhaustive_match_does_not_guard_the_list_a_loop_iterates]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate]]


## 2.ª ocorrência: o `#[cfg(test)]` também se põe sobre uma `fn`

**`render_loop`, 2026-08-30.** Um censo listava as *membranas* do shell varrendo os `.rs` do
directório, e um arnês de teste novo (nascido de um corte de LOC) passou a contar como
membrana. Ensinei o censo a excluir o que só existe sob `cfg(test)`, lendo o `mod.rs` — e a
varredura procurava o **primeiro `;` depois do atributo**.

⛔ Havia um `#[cfg(test)] pub(crate) fn …` cujo corpo não tinha `;` nenhum. O primeiro `;` a
seguir era o do `mod` SEGUINTE ⇒ o censo excluiu `motion_audio_gen.rs`, uma membrana a sério,
**calando-a**. O gate ficou verde-por-baixo (5 em vez de 6) e só o controlo de contagem o
apanhou.

⭐ **A cura é exigir a FORMA, não procurar um delimitador:** salta-se atributo por atributo
(guardando o `#[path = "…"]`), depois a visibilidade, e o que sobra **tem de começar por
`mod `** — senão não é um módulo e o atributo não diz nada sobre ficheiros.

⚠️ **E o que salvou isto foi o censo ter as duas direcções**: excluir a menos dá `7 ≠ 6`,
excluir a mais dá `5 ≠ 6`. *Uma lista derivada precisa de um controlo que reprove nos dois
sentidos* — um que só verificasse «não sobra nada» teria aceitado a membrana calada.


## 3.ª ocorrência: uma ATRIBUIÇÃO proibida casa dentro de uma COMPARAÇÃO

**`line/components`, 2026-08-31.** Um gate de árvore proíbe o painel de escrever o vínculo de
instância à mão, e a lista de padrões proibidos era literal:

```rust
for forbidden in ["InstanceOf {", "InstanceOf::new(", ".master ="] { assert!(!body.contains(forbidden)) }
```

⛔ Uma guarda nova no despachante — `if info.root_bits == 0 || choice.master == 0` — **reprovou-o**.
`.master =` é prefixo de `.master ==`, e o `contains` não sabe a diferença. A mensagem acusava o
autor de *«escrever o vínculo por outra via, saltando o re-key das excepções»*, sobre um `if` que
não escreve nada.

⭐ **A cura é o detector saber a forma inteira:** depois do campo, salta espaços e exige um `=`
**não seguido de `=`**. Três linhas, e ele veio com **controlo próprio**
(`the_assignment_detector_tells_an_assignment_from_a_comparison`) — sem ele, um detector que
devolvesse sempre `false` deixaria o gate verde para sempre a dizer exactamente o que diz hoje.

⚠️ **Por que isto é pior do que parece:** o gate não ficou cego, ficou **falso acusador** — e o
custo é o mesmo do balde vazio, um nível acima. *Um portão que reprova quem não fez nada é um
portão que o autor seguinte aprende a contornar com um `allow`, e a partir daí a lei real
(a atribuição) passa também.* Relacionado:
[[feedback_a_textual_gate_must_strip_comments_or_documenting_the_cure_fails_it]] (o irmão: ali o
gate lia o que estava num COMENTÁRIO; aqui lê o que está num OPERADOR).

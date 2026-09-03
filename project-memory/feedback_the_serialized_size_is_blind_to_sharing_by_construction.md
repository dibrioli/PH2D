---
name: feedback_the_serialized_size_is_blind_to_sharing_by_construction
description: Medi a residência de uma pilha somando bytes SERIALIZADOS — e o doc do tipo dizia, na mesma frase, que a partilha não viaja no fio. A régua era garantidamente cega ao que eu queria ver.
metadata:
  type: feedback
---

Fui medir quanto a pilha de undo ocupa em memória (2026-09-02). Escrevi:
`postcard::to_allocvec(&snapshot).len() × UNDO_CAP` ⇒ **189 MB** a 10 k entidades. Número grande,
plausível, e ia entrar num relatório como achado.

Está errado. O `WorldSnapshot` guarda `Arc<Row>` **por linha**, e a pilha **partilha a linha de quem
não mudou entre passos** — a residência real, já medida noutra fase, é **~12,5 MB**. E o
doc-comment do campo dizia, **na frase seguinte à que eu tinha lido**: *«a partilha NÃO viaja no
fio — a serde escreve um `Arc<T>` como o próprio `T`»*.

⇒ *o tamanho serializado é, por construção, o único número **cego** à partilha.* Escolhi a régua
que não podia ver o que eu queria medir.

**Why:** uma régua errada não devolve um erro — devolve **um número**, e um número tem autoridade.
Aqui ela era ainda pior que aleatória: era sistematicamente enviesada para cima, no sentido que
confirmava a hipótese que eu tinha ido testar (*«a pilha é pesada»*). O único sinal de alarme
disponível era **ler o tipo até ao fim** antes de escolher como o medir.

**How to apply:** antes de medir o TAMANHO de uma estrutura, pergunte **o que ela partilha** — `Arc`,
`Cow`, `Rc`, um índice para uma arena. Se partilha, o serializado mede o custo de a **escrever**,
nunca o de a **guardar**; a residência mede-se contando alocações únicas (ou lendo o número de quem
já a mediu). ⚠️ E leia o doc-comment do campo **inteiro**: a frase que refuta a sua régua tende a
estar na linha a seguir à que explica porque a estrutura é assim.

Irmão de [[reference_topic_measurement_discipline]] e de
[[feedback_a_ruler_placed_after_the_tidying_step_measures_the_tidying]] — a mesma família: a régua
mede uma coisa e o relatório afirma outra.

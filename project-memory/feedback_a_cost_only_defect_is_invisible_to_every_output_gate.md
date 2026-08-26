---
name: feedback_a_cost_only_defect_is_invisible_to_every_output_gate
description: Quando a cura não muda um bit da saída, nenhum gate de paridade pode vê-la — o instrumento tem de ser um CONTADOR no produto, e ele só se lê onde mais ninguém escreve nele.
metadata:
  type: feedback
---

O traçado do módulo 3D compilava **quatro** fitas por região especializada e avaliava **uma**: uma
fita de gradiente que só a exportação consome, e um `fork` que recompilava o par que acabara de ser
construído. `132` regiões por quadro ⇒ **293** fitas, das quais 132 úteis. ⛔ **Gate nenhum podia
ver isto:** a imagem é byte-idêntica nos dois casos — o defeito era só relógio.

**Why:** um gate de paridade compara **saídas**. Um defeito que não muda a saída existe fora do
alcance dele, e sobrevive a suítes verdes por tempo indefinido (este sobreviveu a `1,65×`–`1,92×` do
quadro). *O que não tem instrumento não tem gate, e o que não tem gate volta.*

**How to apply:**
- Ponha um **contador atómico no produto** (`#[doc(hidden)] pub static`) no sítio exacto onde o
  recurso é consumido — uma fita compilada, uma árvore especializada, uma alocação. O incremento é
  ruído ao lado do que ele conta.
- ⚠️ **Leia-o onde mais ninguém escreve nele.** O contador é do **processo** e o `cargo test` corre
  a suíte em paralelo: um gate que exige `== 0` e um irmão que constrói **um** não cabem no mesmo
  binário. A separação é `tests/<lei>.rs`, um binário por lei — um cadeado não chega, porque teria
  de ser tomado por todos os testes que ainda não existem.
- ⚠️ E **não adivinhe a contagem**: uma sonda deste mesmo módulo assumiu «60 especializações» (o
  número de ladrilhos) e concluiu um preço 4× errado, porque o produto compila **preguiçosamente**.
  *Uma sonda que assume a contagem mede a própria suposição.*

Vizinha de [[feedback_counting_the_work_done_is_not_counting_the_work_delivered]] e de
[[feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing]].

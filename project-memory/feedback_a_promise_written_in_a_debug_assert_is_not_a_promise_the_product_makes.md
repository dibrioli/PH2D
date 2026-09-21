---
name: feedback_a_promise_written_in_a_debug_assert_is_not_a_promise_the_product_makes
description: Uma pré-condição defendida por `debug_assert` não existe no perfil que o dono corre — ele recebe o índice fora de alcance, e a régua nunca chega a falar
metadata:
  type: feedback
---

Uma porta cuja pré-condição é defendida por **`debug_assert`** não tem defesa nenhuma no produto: o
perfil `smoke` herda `release`, logo a linha **não é compilada**. O que a documentação promete
(*«têm de ser as MESMAS faces, senão o payload descreve outra malha»*) e o que o artista recebe
(`index out of bounds`) são coisas diferentes, e entre as duas não há régua.

**Medido** (2026-09-21, tinta fina): report do dono —
`topo.rs:239 index out of bounds: the len is 196608 but the index is 196608`, quadro `10216`. A
mesma reprodução em `dev` dispara o `debug_assert_eq!` com a mensagem certa (*«o payload recebeu
outra face»*); em `release` dá o pânico do dono, **na mesma linha e na mesma coluna**. ⇒ a única
régua daquela pré-condição só existia no perfil em que o defeito nunca aparece.

⭐ **A cura não é um `assert!`**: é a porta passar a devolver um **VEREDITO** (`-> bool`, com
`#[must_use]`) e a recusar — deixando a saída **vazia**, nunca meio escrita. Um `assert!` troca um
pânico por outro; um veredito deixa quem chama **desarmar** e mostrar a coisa de baixa resolução,
que é o que ia acontecer no quadro seguinte de qualquer maneira.

⚠️ **E a metade CURTA é a que nunca estoura e é a pior:** com MENOS entradas do que a estrutura
descreve o laço acaba sozinho, nada sai de alcance, e o registo fica truncado — bytes válidos no
sítio errado, em silêncio. ⇒ a régua é a **IGUALDADE**, nunca um tecto.

**Why:** um `debug_assert` documenta uma pré-condição para quem LÊ o código e não para quem CORRE o
produto; ele faz o autor sentir que a cerca existe. O modo de falha é o pior possível — a cerca
parece lá, o `cargo test` em `dev` confirma-a, e o único que a experimenta é o dono.

**How to apply:** `grep` por `debug_assert` em toda porta que recebe DUAS coisas que têm de
concordar (uma estrutura e os dados de que ela nasceu) — cada um é um pânico à espera do perfil
certo. Substitua por veredito + recusa, e ponha o gate no **valor de retorno**, não na ausência de
pânico. E quando a mesma pergunta tem um leitor noutra crate, a lei muda-se para a crate que POSSUI
o dado: [[feedback_the_door_with_the_right_law_had_no_caller_and_the_consumer_used_a_third]].

Vizinhas: [[feedback_a_fence_that_never_bites_hides_the_one_that_does]] ·
[[feedback_a_doc_comment_naming_a_cfg_expires_grep_the_attribute]] ·
[[feedback_a_calm_ruler_that_does_not_name_the_build_profile_reads_as_a_verdict]]

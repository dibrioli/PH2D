---
name: feedback_matching_by_a_loose_name_breaks_the_tree_the_key_is_a_path
description: "Emparelhar duas árvores por NOME solto produz mapas que partem a estrutura — e um reconciliador que não muda pais deixa o defeito estável e mudo"
metadata:
  type: feedback
---

Quando é preciso emparelhar as peças de duas árvores que não têm elo entre si (substituir um
componente por outro, importar sobre uma hierarquia existente), o atalho óbvio é `nome → nome`. Ele
está errado, e o modo de falha é silencioso.

Medido em 2026-09-05 (`line/components`, F5). A peça `Wheel` que numa receita pende da raiz e noutra
pende da `Cabin` **emparelha por nome** — e a cópia fica com a roda debaixo da raiz enquanto a
receita diz que ela vive debaixo da cabina. ⚠️ O passe estrutural desta casa **materializa e apaga,
mas não muda peças de pai**: ⇒ o estado resultante é **estável** (nenhum passe o corrige, nenhum o
acusa) e **mudo**.

⇒ a chave dos dois modos passou a ser o **caminho desde a raiz**, e a única diferença entre eles é o
degrau:

| modo | um degrau é | sobrevive a | perde-se com |
|---|---|---|---|
| por nome | o `Name` da peça | reordenar os irmãos | renomear |
| por posição | o índice entre os irmãos | renomear | reordenar |

⭐ **E um caminho só emparelha se o PAI dele emparelhar.** Com dois irmãos `Arm`, o caminho `Arm` é
ambíguo e cai — e sem esta lei o `Arm/Hand` de um deles, que é **único**, emparelharia sobre um pai
que não existe daquele lado. Um percurso só chega: num `BTreeMap` de caminhos, um filho ordena
sempre depois de todos os prefixos dele, então verificar o pai imediato verifica a cadeia inteira.

**Why:** um mapa de emparelhamento não é uma tabela de correspondências — é uma afirmação de que as
duas árvores podem ser sobrepostas. Sem a estrutura, o mapa promete algo que o consumidor a jusante
não consegue realizar, e quem paga é uma cena que fica errada sem erro.

**How to apply:** a fixtura que mede dois modos de emparelhamento tem de os fazer **discordar** — ali
foi pôr a ordem dos irmãos ao contrário entre as duas receitas (`[Body, Wheel]` contra
`[Wheel, Body]`). ⚠️ *Duas leis que concordam na fixtura são uma lei só para quem mede*: com as peças
na mesma ordem, um gate do modo «por nome» fica verde sobre uma implementação que só sabe contar
índices. Ver [[reference_topic_fixture_discipline]] e
[[feedback_a_key_whose_owner_never_dies_has_no_gravedigger]] (a outra metade da mesma wave).

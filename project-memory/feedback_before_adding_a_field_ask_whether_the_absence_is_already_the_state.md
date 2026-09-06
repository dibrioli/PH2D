---
name: feedback_before_adding_a_field_ask_whether_the_absence_is_already_the_state
description: "Duas features espelho, uma precisou de um campo persistido e a outra de NADA — a régua é se a ausência já é load-bearing em alguém"
metadata:
  type: feedback
---

Duas features podem ser espelhos exactos uma da outra e ter **preços de modelo opostos**. A régua
não é a simetria: é perguntar se a coisa que se ia guardar **já está escrita na estrutura, e já é
lida por alguém**.

Medido em 2026-09-06 (`line/components`), no par *Removed / Added GameObject*:

| feature | o que ia ser guardado | veredito |
|---|---|---|
| a cópia **RECUSA** uma peça da receita | `removed: BTreeSet<StableId>` | **necessário** — a peça continua viva na receita e ausente na cópia, e uma ausência não distingue *«recusei»* de *«ainda não materializei»*. Degrau de `PROJECT_SCHEMA` |
| a cópia **GANHA** uma peça | nada | ⭐ **a verdade já estava escrita**: uma entidade sem elo dentro de uma cópia é autoria do artista, e essa ausência já era **load-bearing** em dois sítios (o passe não lhe toca; o apagar deixa-a morrer). Lista derivada, `PROJECT_SCHEMA` intocado |

⭐ O critério é o da refutação que a mesma linha já tinha pago: *guardar um valor cria duas fontes
para o mesmo facto, e isso só é aceitável quando não há primeira*. Aqui ele corre ao contrário —
**procure a primeira fonte antes de escrever a segunda.**

⚠️ **O sinal de que a primeira fonte existe é ela já ter CONSUMIDORES.** Não basta que a informação
seja *derivável*: se ninguém a lê, a derivação é código novo tão frágil como um campo. O que tornou
a segunda feature barata foi haver já duas leis escritas sobre aquela ausência — a lista nova é a
**terceira leitora** da mesma verdade, não a primeira.

**Why:** um campo persistido custa um degrau de schema, uma migração, três sítios a manter, e é
irreversível na prática. Um derivado custa uma função. A diferença entre os dois casos acima é
invisível na descrição do produto (*«mostrar o que esta cópia tem de diferente»*) e decisiva na
implementação.

**How to apply:** antes de acrescentar um campo, `grep` o predicado que o descreveria e veja **quem
já o pergunta**. Se houver leitores, a lista é derivada. Ver
[[feedback_the_representation_can_delete_the_special_case]] e
[[feedback_the_missing_piece_may_already_be_built_measure_its_structure_first]].

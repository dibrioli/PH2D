---
name: feedback_a_note_that_names_the_right_granularity_for_one_side_reads_as_done
description: Uma fórmula com dois lados cuja nota declara a granularidade certa só para UM deles lê-se como cumprida — e o outro lado fabrica o defeito
metadata:
  type: feedback
---

⛔⛔ **Uma nota que declara a granularidade CERTA para um dos lados de uma fórmula lê-se como
cumprida — e o outro lado continua a produzir o defeito.**

Medido 2026-09-15 (`line/UIUX`, report do dono com foto: *«a caixa recua quando na verdade o nome
deveria criar as colunas»*). A largura da coluna do nome de uma linha de propriedade sai de **duas**
grandezas:

| grandeza | granularidade | onde estava escrito |
|---|---|---|
| o que o CONTROLO precisa | **secção** ✅ | no doc da porta, com a razão: *«se cada linha cedesse pelo que ELA precisa, a coluna saía esfarrapada»* |
| o que o NOME precisa | **linha** ⛔ | em lado nenhum — o pintor media-o a cada chamada |

E o nome entra na conta **duas** vezes (o empréstimo e o piso da cedência). Resultado: a linha com o
nome mais comprido empurrava a caixa **dela** e mais nenhuma (`x = 100,2` contra `98,0` das irmãs a
`220` de painel), e noutra secção o desalinhamento chegava a **`48 px`** sem ninguém reportar.

**Why:** ao rever o doc da porta eu li a frase sobre a SECÇÃO e concluí que a lei estava cumprida —
ela estava, para metade da fórmula. *Uma nota correcta sobre um dos lados é o disfarce mais eficaz
do outro lado.*

**How to apply:** quando uma fórmula combina N grandezas que vêm de fora, **liste-as e escreva a
granularidade de CADA uma** antes de acreditar na nota. E se elas têm de ter a mesma, não as passe
como N argumentos: **junte-as num TIPO** (aqui, `property_row::Seccao { campos, nome_w }`) — assim
declarar uma sem a outra deixa de compilar. Ver [[reference_topic_measurement_discipline]] e
[[reference_topic_gate_discipline]].

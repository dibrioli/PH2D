---
name: feedback_an_impacted_test_selector_that_maps_paths_by_prefix_is_blind_outside_it
description: O nextest-impacted.sh já saiu VERDE sem medir DUAS vezes — pelo prefixo `crates/` (curado 22/08) e por `git diff A...` ser cego ao trabalho NÃO COMMITADO (curado 23/08)
metadata:
  type: feedback
---

`scripts/nextest-impacted.sh` monta o conjunto impactado com
`sed -n 's#^crates/\([^/]*\)/.*#\1#p'`. Um diff inteiramente fora de `crates/`
produz `CHANGED` **vazio**, o script cai no ramo *"no crate changes"* e roda
**só o golden de determinismo** — 4 testes, 1306 binários pulados, exit 0.

Medido em 2026-08-19 (`line/sculpt3d`, 5 arquivos em `shells/desktop/src/`).

**Why:** `shells/desktop` guarda o shell inteiro do sculpt3d, o undo, a
persistência e o `input_dispatch` — dezenas de milhares de LOC. O gate de
fechamento da DIRETRIZ §1.5.9 chama este script pelo nome, então **todo
fechamento cujo diff seja só de shell correu com essa cobertura**. ⚠️ E o
comentário do próprio script promete o contrário (*"the filterset errors out —
that surfaces the dir→package mismatch rather than silently under-testing"*):
isso vale para um NOME errado dentro de `crates/`; um caminho FORA dele não gera
nome nenhum, e o silêncio é total. É a forma mais cara de um gate verde — ver
[[reference_topic_gate_discipline]].

**How to apply:** ao fechar uma linha cujo diff toque `shells/`, `tools/` ou
`tests/`, **não confie no script**: rode a superfície da crate à mão
(`cargo test -p <pacote> --release --bins` + `--include-ignored` para os gates de
GPU) e registre a contagem no handoff. A cura de raiz é derivar o pacote do
`cargo metadata` (o `manifest_path` como prefixo mais longo) em vez do caminho —
⚠️ mas ela muda o que **toda** linha roda ao fechar, então é decisão do Enio, não
um detalhe que viaje dentro de outro conserto ([[feedback_ship_only_enio_end_of_all_lines]]).

---

## A SEGUNDA cegueira do mesmo script: o trabalho NÃO COMMITADO (2026-08-23)

O conjunto impactado vinha de `git diff --name-only "${BASE}"...`, e **`A...`
compara COMMITS**: um arquivo editado e ainda por commitar não aparece.

Medido no fecho da booleana×States (`line/Vector`): quatro arquivos da
`ph2d-ui-state` estavam por commitar, o script imprimiu
`changed: ph2d-host-desktop ph2d-panel-vector`, e correu **10 418 testes sem
correr UM único da `ph2d-ui-state`** — incluindo os cinco gates que a wave tinha
acabado de escrever. Exit 0, sem uma linha a dizê-lo.

⚠️ **`rdeps(X)` não salva:** ele traz `X` e quem DEPENDE de `X`. A `ph2d-ui-state`
é *dependência* da `ph2d-host-desktop`, não dependente — os testes dela ficam
fora por construção.

**Why:** as duas cegueiras são a mesma doença — *o gate sai verde por não medir* —
e ela é a mais cara que existe, porque quem a lê conclui o oposto do que aconteceu.
A primeira foi achada a escrever um handoff; esta, a conferir por que a lista
impressa não tinha uma crate que o `git status` mostrava.

**How to apply:** **leia a linha `[nextest-impacted] changed:` e confira-a contra
o seu `git status`** — se faltar uma crate que você editou, o resultado não vale.
Curado no mesmo dia: o conjunto passou a ser a **união** do diff commitado com o
`git status --porcelain`, e o script ganhou `NO_FAIL_FAST=1` (sem ele o `nextest`
cancela na primeira falha e deixa 7 mil testes por correr — a regra estava no
CLAUDE.md §5.0 e **não estava no caminho de quem a executa**, que é o script:
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).

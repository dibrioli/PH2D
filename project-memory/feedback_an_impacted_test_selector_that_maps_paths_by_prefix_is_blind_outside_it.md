---
name: feedback_an_impacted_test_selector_that_maps_paths_by_prefix_is_blind_outside_it
description: O nextest-impacted.sh deriva o pacote do prefixo `crates/` — um diff só em `shells/`, `tools/` ou `tests/` roda 4 testes e sai VERDE
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

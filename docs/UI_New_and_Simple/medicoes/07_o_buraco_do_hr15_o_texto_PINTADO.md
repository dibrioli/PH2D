# 07 — O buraco do HR-15: o texto **PINTADO**

> Medido em 2026-09-09 pela `line/UIUX`, a partir de um achado lateral da wave das faces vazias.

## §1 — O que o gate vê, e o que ele não vê

O `CLAUDE.md` §0.3 diz **«zero string hardcoded — tudo via tokens / i18n (HR-15)»**, e o gate que
o defende é [`hr15_no_hardcoded_ui_strings.rs`](../../../crates/ph2d-editor-core/tests/hr15_no_hardcoded_ui_strings.rs).
O doc dele nomeia, de si próprio, exactamente **dois** padrões:

| padrão | o que é |
|---|---|
| `.label("…")` | a etiqueta de acessibilidade |
| `.placeholder("…")` | o texto de sugestão de um campo |

⛔⛔ **A maior categoria de texto que o artista lê não é nenhuma das duas: é o texto que um pintor
DESENHA** — `paint_text(.., "Select an entity in the Hierarchy…", ..)`. Ele não passa por
`.label` nem por `.placeholder`, logo o gate **nunca o viu**.

*Um gate que enumera dois padrões afirma sobre esses dois, e a prosa à volta dele afirma sobre a
lei inteira.* É a mesma forma do censo de recuo que esta linha já pagou quatro vezes — só que aqui
o que falta não é uma FORMA do que se lê, é uma **população**.

## §2 — O tamanho, contado

Varredura sobre `crates/ph2d-editor-core/src` + todos os `crates/ph2d-panel-*/src`, contando
chamadas `paint_text*(…)` cujo argumento de texto é um **literal**, com os comentários e o bloco
`#[cfg(test)]` fora:

| crate | literais pintados |
|---|---|
| `ph2d-panel-inspector` | **36** |
| `ph2d-editor-core` | 17 |
| `ph2d-panel-painter-layers` | 15 |
| `ph2d-panel-vector` | 8 |
| `ph2d-panel-motion-params` | 7 |
| `ph2d-panel-audio-editor` | 6 |
| `ph2d-panel-flip` | 5 |
| `ph2d-panel-timeline` | 4 |
| `ph2d-panel-color-equalization` | 3 |
| `ph2d-panel-tokens` | 3 |
| `ph2d-panel-widget-lab` | 2 |
| `ph2d-panel-grid-snap` | 1 |
| `ph2d-panel-asset-browser` | 1 |
| **total** | **108** em 13 crates |

⚠️ **A premissa que o gate declara sobre si mesmo já não é verdade.** Ele diz: *«Until the Fluent
runtime (`ph2d-i18n`) is wired and `t!(...)` exists, this test enforces a frozen baseline»* — e o
`ph2d_i18n::tr` **existe e é usado** (o painel de escultura, o modelador 3D e o vetorial têm tabela
própria em `crates/ph2d-i18n/src/`). *O gate está à espera de uma coisa que chegou.*

## §3 — Por que esta linha MEDIU e não CUROU

⛔ **Não é preguiça, é o custo de merge.** Curar os 108 toca **13 crates**, e onze delas são de
outras linhas — cinco delas vivas em 2026-09-09. A memória do repo já regista o preço desta forma
exacta (*«apagar ~2 961 LOC do `VecInstance` em 24 ficheiros com a `line/Vector` VIVA é catástrofe
de merge»*), e uma migração mecânica larga é o caso pior: diff enorme, zero conflito semântico,
conflito textual em todo o lado.

⛔ **E a catraca também não se arma sozinha.** Um censo com 108 entradas por crate faria a próxima
linha que pinte um rótulo ficar **vermelha na crate dela por causa de um gate desta**. O
`CLAUDE.md` §0.2 pede que o foundational novo seja *projectado para isolamento*; um censo global
com dívida alheia dentro é o contrário disso.

## §4 — O que fica recomendado, em ordem

1. **Alargar o gate para VER o texto pintado** (a forma nova: `paint_text*` com literal), com a
   baseline por crate congelada nos números da §2 — de preferência **na janela de integração**, que
   é quando as 13 crates estão na mesma árvore e ninguém está a meio de uma wave.
2. **Curar por CRATE, pela linha dona de cada uma** — a `ph2d-panel-inspector` sozinha é um terço
   do total.
3. ⚠️ **Corrigir a prosa do gate** no mesmo commit: ela promete a lei inteira e entrega dois
   padrões, e é essa frase que faz a próxima pessoa acreditar que o HR-15 está fechado.

## ⛔ Recusas MEDIDAS

| o que | por que não |
|---|---|
| migrar os 108 nesta janela | 13 crates, 11 de outras linhas, 5 vivas — catástrofe de merge por diff mecânico |
| armar a catraca global agora | reprovaria outras linhas na crate delas por um gate desta; §0.2 pede isolamento |

# 14 — A última fronteira: o catálogo do vector

> **Medido em 2026-09-17, `line/UIUX`.** A 9.ª fatia do HR-15. A nota de pendências dizia
> *«`ph2d-tool-vector` — 7 braços de rótulo»*. A régua registada conta **163**.

## §1 — ⛔ A nota estava certa sobre uma FUNÇÃO e errada sobre a crate

Os `7` vinham do censo de PORTA (`scripts/censo-texto-pintado.py`), que só vê texto a chegar a um
pintor **pelo nome dele** — e ali ele via os sete braços de `ShapeGroup::label()`. ⚠️ Mas esta crate
não pinta: ela **publica uma tabela de dados**, e o painel é que pinta. É a mesma fronteira dos
MOTORES que as seis primeiras fatias fecharam nos nós, um módulo depois.

| ficheiro | literais |
|---|---:|
| `shapes.rs` | 134 |
| `tool.rs` | 12 |
| `connector.rs` | 8 |
| `params.rs` | 5 |
| `frames.rs` | 4 |

## §2 — ⚠️ A crate é de OUTRA LINHA VIVA, e isso escolheu a cura

`git worktree list` diz que a `line/Vector` está aberta. ⇒ `ph2d-tool-vector` é o módulo dela
(`CLAUDE.md` §0.2), e esta fatia **não escreve uma linha lá dentro**.

⭐ **E a cura certa já existia:** a wave de 2026-09-16 construiu o
[`nomes_do_motor`](../../../crates/ph2d-panel-vector/src/nomes_do_motor.rs) — *um `match` por
família* que leva o rótulo inglês do motor a uma chave, com o gate
`every_name_the_engine_publishes_has_a_key` a varrer as tabelas do motor e a exigir uma chave para
cada rótulo. Esta fatia **estende** esse mecanismo com cinco famílias novas; não inventa nada.

⇒ o rótulo do motor é um **identificador** (o braço de um `match`, que a régua lexical isenta) e o
que se pinta é a chave.

## §3 — ⛔ E a medição achou a colisão que a lei daquele ficheiro só citava

O doc do `nomes_do_motor` explicava *«um `match` POR FAMÍLIA, porque o mesmo rótulo diz coisas
diferentes»* com um exemplo. Medido sobre as famílias novas, ele tem razão **três** vezes:

| rótulo | família A | família B |
|---|---|---|
| `Round` | a FAMÍLIA do catálogo | a FORMA (rectângulo arredondado) |
| `Corner` | um campo de FORMA (a seta dobrada) | um campo de CONECTOR |
| `Curve` | um campo de FORMA (o escudo) | um campo de CONECTOR |

*Uma tabela só daria a uma delas a palavra da outra.*

## §4 — ⛔⛔ Eu ia construir a SEGUNDA resposta para uma pergunta já respondida

A família das **famílias do catálogo** (`Basic`/`Round`/`Arrows`/…) já tinha ponte desde antes: o
[`state::group_i18n_key`](../../../crates/ph2d-panel-vector/src/state.rs), chaveado pela **VARIANTE
do enum** e não pela palavra inglesa — que é a forma **mais forte** das duas. Eu contei aqueles sete
rótulos no censo lexical da crate do motor e escrevi-lhes chaves antes de perguntar se alguém já os
traduzia.

⇒ `CLAUDE.md` §5.0: *antes de construir um item de lista aberta, MEÇA se a composição já o exprime.*
A família saiu (7 chaves a menos), e o que fica escrito ao lado do código é a razão.

## §5 — ⭐⭐ O GATE achou cinco rótulos que a minha extracção não via

Eu derivei a população lendo o fonte com uma varredura de `const … : FieldDesc` e dos helpers. O
gate, que percorre a tabela **VIVA** (`shapes::SHAPES` → `d.fields` → `f.label`), reprovou com cinco
nomes que a varredura perdeu — `Bubbles`, `Smooth`, `Spikes`, `Teeth`, `Wedge`: são `FieldDesc`
escritos **em linha** dentro da forma.

⇒ *a tabela viva é o oráculo; um extractor de fonte é uma aproximação dela.* É a mesma lei que o §0.9
escreve sobre apps de referência, aplicada ao nosso próprio código.

## §6 — ⚠️ Duas listas ficam escritas à mão, e trazem um controlo DERIVADO

As formas publicam `SHAPES`; o conector e as pontas **não publicam um `ALL_FIELDS`** — e
acrescentar-lhes um seria editar o módulo de outra linha. ⇒ o gate enumera aqueles `FieldDesc` à mão
**e conta os `pub const …: FieldDesc` no fonte do motor**, exigindo que a lista os cubra.

*Uma lista escrita à mão sem um controlo que a confronte com a fonte é uma lista que envelhece em
silêncio.*

## §7 — ⛔ E dez palavras NÃO são texto de interface

A `PALETTE` do `tool.rs` (`White`, `Black`, `Gray`, …) tem **zero consumidores fora da crate**: o doc
dela diz-se *«retained as the seed source for the tool's defaults (and a stable named-colour
reference)»*, e o caminho vivo da cor é o seletor OKLCH. ⇒ elas são uma **referência de cor com
nome**, não um rótulo — e migrá-las seria dar tradução a uma coisa que ninguém lê.

## §8 — Os números

| | |
|---|---:|
| chaves novas na tabela do motor | **122** |
| famílias novas na ponte | **5** (`forma` · `campo` · `conector` · `moldura` · `ponta`) + o `marcador` |
| famílias que NÃO entraram por já existirem | 1 (`grupo`) |
| sítios de pintura ligados | 12 |
| linhas escritas na crate da outra linha | **0** |

**Prova de mutação: 3 de 3 sangram**, com controlo negativo — uma FORMA do motor a perder a chave
(o painel pintaria o nome cru) · a PORTA do catálogo a deixar de traduzir (⚠️ em inglês a tabela diz
o mesmo que o motor, logo **nenhum pixel muda** e só o gate das portas o vê) · e a lista escrita à
mão a deixar de cobrir os `FieldDesc` do motor.

## §9 — ⏳ O que fica

| alvo | nota |
|---|---|
| `ph2d-vec-scene` | 86 literais pela régua registada; esta fatia levou só as **8 pontas** que este painel pinta — o resto é população de outra crate do motor |
| o `keys_used` conta USOS em ficheiros de TESTE | ver `13_…` §8 |
| o MENU e a ABA têm chaves diferentes para a mesma palavra | ver `12_…` §6.1 |

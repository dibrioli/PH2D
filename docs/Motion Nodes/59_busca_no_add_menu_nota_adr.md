# 59 — A busca no add-menu (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-13, commit `cc42f47c`).
> Escrito **depois** do código (o código já citava "doc 59" e o arquivo não existia — referência
> pendurada, fechada aqui).

## 1. O smoke que forçou a feature

Enio, 2026-07-13: ***"não encontrei value.lfo"***.

A biblioteca tinha **86 tipos de nó numa lista plana e rolável**. O artista sabia **exatamente** o
que queria e ainda assim não achou. Uma lista que você precisa **varrer** para de funcionar em algum
lugar perto de quarenta entradas — e todo editor de nós que se leva a sério responde isso da mesma
forma: você abre e **digita** (Shift+A do Blender, a palette do Unreal, o tab menu do Houdini).

**Metade do bug era minha:** eu disse a ele pra procurar `value.lfo` — o **nome canônico**, uma
string que o menu **nunca mostrou**. O menu só exibia o rótulo `LFO`.

## 2. A decisão

O popup abre com um **campo que já tem o teclado**: o gesto é *aperte `A` e digite*.

**A busca casa nos DOIS nomes** — o rótulo que o artista lê (`"LFO"`, `"Map Range"`) e o nome
canônico que o código (e a doc, e eu) fala (`"value.lfo"`, `"value.map_range"`). Achar um nó por
qualquer um dos dois **é achar o nó**.

E isso tem um efeito de segunda ordem que vale mais que a busca em si: **o DOMÍNIO vira query.**
Digite `value` e sai a lista de todos os `value.*`; `force`, todas as forças; `pulse`, todos os
gatilhos. A lista de 86 entradas tinha uma **estrutura** que o menu plano nunca expôs.

**Fuzzy = SUBSEQUÊNCIA, não substring:** `mr` acha "Map Range", `atr` acha "Attractor".

**E é RANQUEADA** — que é a diferença inteira entre uma busca que funciona e uma lista que por acaso
contém a resposta. Filtrar sem ranquear devolve ao artista a mesma lista que ele já tinha, só que
menor. Os pesos são os que todo fuzzy finder converge, e estão ordenados por quão fortemente um
humano os lê como **intenção**:

| sinal | exemplo | por quê |
|---|---|---|
| **prefixo** | `lf` → **LF**O | você começou a digitar o nome dele |
| **fronteira de palavra** | `mr` → **M**ap **R**ange | você digitou as iniciais |
| **corrida contígua** | `ange` → M**a**p R**ange** | você digitou um pedaço dele |
| **penalidade de lixo pulado** | — | um nome curto que casa apertado ganha de um longo que casa frouxo |

O nome canônico casa **com desconto**: quando os dois batem, o artista quase sempre queria o rótulo
que ele **vê**.

**Enter** escolhe o primeiro (a linha que ele já está olhando); **Esc** fecha o popup.

## 3. As duas disciplinas que impedem o bug clássico de menu

1. **`menu_rows`/`menu_matches` são a ÚNICA fonte das linhas** — pintura, hit-test **e** geometria
   saem dali. Um segundo enumerador é exatamente como uma linha passa a significar uma coisa na tela
   e outra debaixo do cursor. (Este popup **já teve** esse bug: o hit lia o catálogo cru enquanto a
   pintura lia a lista filtrada.)
2. **A STORE é dona do buffer** (texto, caret, seleção); `Menu::query` é só uma **leitura** dela,
   espelhada no topo de cada `process`. Duas cópias de uma string, editadas dos dois lados, é o jeito
   clássico de um campo de texto começar a mentir sobre o que contém.

## 4. Dívida morta de brinde

O `cancel_on_escape` era uma **lista de ids hardcoded dentro do `dispatch_key`**
(`id == HIER_RENAME_INPUT || id == TIMELINE_MARKER_RENAME_INPUT`). Virou **flag por widget**
(`mark_cancel_on_escape`), e os dois ids existentes agora **se marcam**. Sem isso, todo campo novo
que quisesse "Esc cancela" teria que ser adicionado àquela expressão — e o terceiro que esquecesse
perderia o Esc **em silêncio**.

## 5. O que veio DEPOIS, e é a lição maior

Esta fatia é a que expôs o bug que o menu tinha **desde que existe**: **o clique não inseria nada**.

A causa: o dispatcher classifica um press-release **com qualquer movimento** como `End` (arrasto),
não como `Click` — e **mão humana sempre desliza um pixel**. O braço do `End` dispensava o menu sem
resolvê-lo.

**Nenhum gate pegou porque todos os testes desta crate mandavam Down e Up na MESMA coordenada** — a
única coisa que uma mão de verdade nunca faz. Setenta e cinco testes verdes numa feature inusável.

Fix: enquanto há menu aberto, **o ponteiro pertence ao MENU** — onde o botão **sobe** é o que vale.
Registrado em [[feedback_a_click_is_a_press_that_drifted]]; todo gate de clique deste painel agora
**desliza 1px** entre o press e o release.

## 6. Superfície

- **Painel:** `menu_search.rs` (novo) · `snapshot_menu.rs` (as linhas) · `paint_menu.rs` ·
  `hits::menu_search_id`.
- **Foundational:** `cancel_on_escape` virou flag (`ph2d-editor-core`: `state/{mod,store_core,
  store_hierarchy}.rs` + `screens/hero.rs` + `ph2d-panel-timeline/src/marker_rename.rs`).
- **Allowlist:** `hr12_widgets_a11y` ganhou `paint_menu.rs` (o menu registra hit **panel-side**, via
  o escudo de `Background` — desenho que precede este split).
- **Contrato congelado:** intocado.
- **Depois (doc 62):** o campo de busca virou **exclusivo da biblioteca** — ele estava sendo pintado
  em **todo** popup, inclusive nos que não têm o que filtrar, **e tomava o teclado**.

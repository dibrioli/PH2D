# 10 — O clássico não é um ecrã anterior: é uma pele de widget

> Medido em 2026-09-10 pela `line/UIUX`, contra o código e contra o `git log`.
>
> ⚠️ **A pergunta que abriu isto:** o `CLAUDE.md` §5 lista, entre os smokes deste módulo,
> *«`PH2D_UI_NEW=0` (o clássico tem de ficar **byte a byte**)»* — e as waves das abas (o ícone, a
> aba sozinha, as setas de transbordo) aplicam-se às **duas** aparências. Era regressão?

## §1 — O veredito

**Não é regressão: a cláusula do §5 está errada, e já estava errada no dia em que foi escrita.**

| facto | data | commit |
|---|---|---|
| o sistema de **abas de encaixe** nasce, sem consultar a aparência em sítio nenhum | **2026-08-30** | `c7c5c653a` |
| o redesenho passa a ser o caminho de omissão | 2026-09-05 | `34a72fbba` |
| a cláusula *«o clássico tem de ficar byte a byte»* entra no roteador | **2026-09-07** | `5fe3c51cc` |

⇒ **oito dias** separam o facto da promessa. *Uma promessa escrita depois do facto que a desmente
não é uma regressão de quem veio a seguir* — e as três waves das abas (w44 ícone · w46 aba sozinha ·
w48 transbordo) só engordaram um sistema que já corria nas duas aparências desde que existe.

## §2 — O que o interruptor DE FACTO devolve

Censo derivado do fonte, separando **consumir** (`ui_is_redesign()` / `ui_look()`) de **publicar**
(`set_ui_look` / `ui_look_from_env`) — **11 ficheiros mencionam a aparência, 9 consomem-na**:

| consumidor | o que o `PH2D_UI_NEW=0` lhe muda |
|---|---|
| `widget/checkbox/mark.rs` | a marca da caixa |
| `widget/toggle.rs` | o interruptor |
| `widget/slider_with_chip/mod.rs` (4 sítios) | o slider e o chip |
| `widget/showcase/slider.rs` | a amostra do slider |
| `widget/property_box/mod.rs` + `paint.rs` | a caixa de propriedade e o decorador |
| `screens/hero/menu_rows.rs` | a **família de temas** que a barra do topo oferece |
| `ph2d-panel-widget-lab/src/paint.rs` | a bancada, que se **força** ao redesenho |
| `ph2d_tokens::Theme::default_for` | o tema de **arranque** (`Classic → Forge`, `Redesign → Dark`) |

⛔ **E o que ele NÃO toca:** o layout, as colunas, o encaixe, a faixa de abas, o transbordo, a
moldura, o vão, a quina, o recuo, o cartão, o chão da janela, o `panel_chrome` — **nada** em
`screens/` que decida GEOMETRIA lê a aparência.

⇒ o interruptor responde *«a **caixa** já era assim?»*. Ele nunca respondeu *«o **ecrã** já era
assim?»* — e o gate irmão
[`the_two_looks_are_one_switch_apart`](../../../crates/ph2d-editor-core/tests/it/the_two_looks_are_one_switch_apart.rs)
sempre soube disso: o doc dele diz, com todas as letras, que mede **três pintores de widget**.

## §3 — ⛔ Por que a cura NÃO é gatear as abas pela aparência

| saída | preço medido |
|---|---|
| pôr a faixa de abas atrás do interruptor | **dois modelos de áreas vivos ao mesmo tempo**: cada gate de coluna, faixa e transbordo passa a ter duas respostas certas, para sempre |
| reverter as três waves no clássico | compra uma bissecção que a ferramenta **nunca ofereceu** (as abas nasceram nas duas) |
| **corrigir a cláusula** | uma linha no §5, e um gate que impede que ela volte por acidente |

⚠️ E a ordem do dono de 2026-09-09 (*«mesmo se houver apenas 1 painel, a aba aparece sozinha, mas
aparece. Isso deve valer para Hierarchy também»*) **não traz aparência nenhuma dentro dela**.

## §4 — A lei fica executável

[`the_look_is_a_widget_skin_never_an_area_model`](../../../crates/ph2d-editor-core/tests/it/the_look_is_a_widget_skin_never_an_area_model.rs)
— um censo derivado com as **três** metades que este repo já pagou:

1. **A lei** — nenhum ficheiro de `screens/` consome a aparência, salvo a excepção nomeada.
2. **O controlo de vacuidade** — a pele tem de ser ENCONTRADA (`≥ 5` pintores de widget), senão uma
   varredura partida devolve zero acusados e lê-se como aprovada.
3. **A metade justa** — a excepção tem de continuar a descrever um leitor real, senão a lista de
   dívida vira licença (§5.0).

⚠️ **O discriminador é a chamada, não o texto:** o `screens/hero/paint.rs` menciona `UiLook` três
vezes e é o **publicador** (`set_ui_look(ui_look_from_env())`) — um censo que casasse a palavra
acusaria exactamente quem tem de existir.

⭐ Provado por mutação nos dois sentidos: pôr um `ui_is_redesign()` no `slot_tabs.rs` deixa o gate
vermelho pelo nome do ficheiro; apontar a excepção a um ficheiro que não consome derruba a metade
justa.

## §5 — A correcção pedida ao §5 do roteador

A linha de smokes do módulo diz hoje:

> `PH2D_UI_NEW=0` (o clássico tem de ficar byte a byte)

e devia dizer:

> ⚠️ `PH2D_UI_NEW=0` **não é o ecrã de antes do redesenho** — ele devolve **seis pintores de
> widget**, a família de temas da barra do topo e o tema de arranque. A estrutura (colunas,
> encaixes, faixa de abas, transbordo) é a **mesma** nas duas desde 2026-08-30, e há gate a
> mantê-la assim.

⚠️ **Esta linha não se edita aqui** — o §5 só se edita na integração (DIRETRIZ §1.5.9). A correcção
viaja no handoff.

## ⛔ Recusas MEDIDAS

| o que | por que não |
|---|---|
| gatear a faixa/o ícone/o transbordo das abas pela aparência | dois modelos de áreas vivos para sempre, por uma bissecção que o interruptor nunca ofereceu (§3) |
| um gate que arme as duas aparências no MESMO processo e compare o quadro | **vácuo**: o `paint_hero_screen` chama `set_ui_look(ui_look_from_env())` **a cada quadro**, e o `ui_look_from_env` é um `OnceLock` — o produto reescreve o que a sonda armou, e as duas leituras saem iguais. *Uma sonda que arma por API o que o produto rearma por ambiente mede o mesmo programa duas vezes.* |
| ler «byte a byte» como promessa viva | ela entra no roteador oito dias **depois** do facto que a desmente (§1) |

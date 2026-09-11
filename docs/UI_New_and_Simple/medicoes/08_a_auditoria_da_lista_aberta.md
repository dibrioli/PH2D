# 08 — A auditoria da lista ABERTA do módulo

> Medida em 2026-09-09 pela `line/UIUX`, contra o **código**, item a item da lista *«Aberto:»* que
> o `CLAUDE.md` §5 traz para o módulo de UI/UX.
>
> ⚠️ **A razão de existir deste doc está escrita no próprio §5:** *«o §5 só se edita na integração,
> então ele acumula trabalho já pago — audite a lista antes de pegar um item dela, e confira o
> CÓDIGO antes de acreditar numa ausência.»* Das **cinco** entradas, **uma** continua aberta.

## §1 — O placar

| item do §5 | veredito medido |
|---|---|
| o **DESENHO das abas de painel** (largura por conteúdo · quina só em cima · fundo da inactiva · ícone · transbordo) | **4 de 5 FECHADOS**; sobra o transbordo (§2) |
| a **incoerência** entre as abas de ENCAIXE e as de LAYOUT | **NÃO é incoerência** — há uma lei, e ela já corre em dois sítios (§3) |
| o **alvo de toque** de uma aba tem 22 px num app cujo alvo é TABLET | o `22` é o veredito do dono para **toda linha do app** (§4) |
| **esvaziar os painéis** (*«1 de 25 censuados»*) | **3 de 26** têm face vazia; o único mudo está **desligado** (§5) |
| as superfícies das outras linhas fora das portas do RITMO | **FECHADO** na w47 — a catraca está a zero (§6) |

## §2 — As abas: 4 de 5

| metade | onde | estado |
|---|---|---|
| largura por conteúdo | `slot_tabs_face::natural_w` | ✅ w34/w35 |
| quina só nos dois cantos de cima | `slot_tabs::tab_radii` → `(r, r, 0, 0)` | ✅ |
| fundo da aba inactiva | ⚠️ **decidido, não esquecido**: a inactiva não pinta fundo (porte do `theme_modern.cpp`), e o que a torna legível ao encolher é a **divisória** | ✅ w38 |
| **ícone** | `Panel::ICON` + `slot_tabs_face` | ✅ w44 |
| afordância de **transbordo** | ⏳ **ABERTA** | — |

⚠️ **O transbordo tem o alcance medido, e ele é menor do que parece.** Com o piso de uma aba a ser
um quadrado de `ROW_H_PX`, a coluna da direita de fábrica (296 px úteis) pinta **13** abas; é
preciso um encaixe com **14+** ocupantes para esconder alguma. ⭐ Mas a coluna **estreita-se**: no
degrau do fecho (198 px, 190 úteis) cabem **8** — logo `9+` painéis do mesmo lado numa coluna
apertada já escondem abas.

⛔ **E não é um beco sem saída:** o painel escondido continua alcançável pelo menu *Window*, e
levantá-lo desliza a janela até ele (`tab_layout` centra-se no escolhido). ⇒ é **conveniência**,
não recuperação — e é por isso que ficou para o fim.

## §3 — As duas famílias de aba não discordam: elas respondem a perguntas diferentes

| superfície | acesa | apagada |
|---|---|---|
| aba de **LAYOUT** (`layout_tabs`) | `AccentSoft` + `Accent` | `BgElev` + `Text2` |
| chip de **MÓDULO** na barra do topo (`topbar/cluster_painter`) | `AccentSoft` + `Accent` | — |
| aba de **ENCAIXE** (`slot_tabs`) | o tom do **painel** (soldada ao corpo) + `Text1` | **sem fundo** + `Text2` |

⇒ a lei que o código já corre em **dois** sítios:

> ⭐ **O acento marca um MODO que está LIGADO; uma aba de encaixe escolhe qual CONTEÚDO está à
> frente.**

Uma aba de layout troca o ecrã inteiro — é um modo, e veste-se como os chips de módulo. Uma aba de
encaixe diz qual painel está por cima — e solda-se ao corpo dele, como o `TabContainer` do modelo.
*O §5 dizia «nada no repo escolhe qual está errada»; o que faltava não era uma escolha, era ler as
três superfícies juntas.*

## §4 — O alvo de toque: `22` é veredito do dono, e não da aba

A faixa de abas tem a altura de uma linha (`ROW_H_PX = 22`). ⚠️ **A aba não é um caso especial:**
o gate [`the_app_default_slider_style_is_the_one_the_owner_chose`](../../../crates/ph2d-editor-core/tests/it/the_app_default_slider_style_is_the_one_the_owner_chose.rs)
regista, com a citação, que **o dono escolheu a linha de 22** — e a escada de densidade existe e
tem os outros dois degraus prontos (`cozy` 26 · `comfortable` 32, em `tokens.json`).

⇒ o item, escrito honestamente, não é *«a aba é pequena»*: é *«o app inteiro corre em `compact`
num alvo de tablet»*, e isso é **uma decisão de produto que já foi tomada**, com o mecanismo para a
inverter num sítio só no dia em que o dono quiser.

## §5 — As faces vazias: 3 de 26, e o mudo está desligado

Medido pintando **cada painel de coluna sozinho** sobre uma cena vazia e contando os controlos
registados no `HitIndex` **dentro** do rect dele (o próprio painel e a aba fora):

| painel | controlos | diz porquê está vazio? |
|---|---|---|
| `motion_params` | **0** | ⛔ não — mas o painel **está desligado** por omissão desde 07/09 (`PH2D_MOTION_PANEL=1` traz-no de volta) |
| `model3d` | 1 | ✅ `tr("panel.model3d.empty")` |
| `sculpt3d` | 1 | ✅ desde a w45 |
| `inspector` | 2 | ✅ *«Select an entity in the Hierarchy…»* |
| `hierarchy` | 7 | tem busca + criar — não está vazio |
| os outros 12 | 10..74 | idem |

⇒ o *«1 de 25»* do §5 descrevia o estado de antes das faces vazias. ⚠️ O que a lei da w45 **de
facto** deixou por fazer não é uma lista de painéis: é o `motion_params`, e ele pertence a outra
linha e está desligado.

## §6 — O ritmo: catraca a zero

A w47 fechou-a: a lista de dívida (`MUTE_OK`) está **vazia**, e as duas entradas que sobram estão
noutra lista, com o mecanismo (`NOT_THIS_QUESTION` — a grelha densa do dope-sheet).
⏳ Fica **um** alvo que a régua não vê por construção: o **cartão** do
`ph2d-panel-motion-graph/src/paint_card.rs`, que não menciona `ROW_H_PX` — ele pede outra régua, e
é superfície de uma linha **viva**.

## ⛔ Recusas MEDIDAS

| o que | por que não |
|---|---|
| unificar as duas famílias de aba | elas respondem a perguntas diferentes, e a lei já corre em dois sítios (§3) |
| subir a altura da aba pelo alvo de toque | `22` é o veredito do dono para toda linha do app; o eixo certo é a densidade, num sítio só (§4) |
| uma wave de faces vazias | 3 de 26 já a têm e o único mudo está desligado e é de outra linha (§5) |
| curar o cartão do Motion | superfície de uma linha viva, e a régua actual não a alcança (§6) |

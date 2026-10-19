# 32 — Plano F21: o «misto» completo — a cor escolhida chega a TODAS, e a pilha de camadas também sabe «misto»

> ABERTO 3, 4, 5 e 7 do [handoff F20](../handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-18_F20_MISTO_E_ESPESSURA.md).
> Pedido do dono (19/10): *«escolha e siga. se possível implemente várias no mesmo ciclo»*. Os quatro são da
> MESMA família (a seleção múltipla e o painel) e partilham as portas do F20 (`StyleFields`, `fill_merge`,
> `mixed_of`, `paint_stack::published`), então são um bloco (CLAUDE.md §0.10). O 6 (`token_world` à mão)
> fica de fora: devolve `None` com régua ≤ 0 de propósito, e o `DocPx` limita — outra lei, não esta família.

## §1 — O mecanismo (medido no código, 19/10)

| facto | onde |
|---|---|
| O seletor de cor é lido por SONDAGEM a cada quadro; `set_fill_rgba` só arma um campo quando o valor DIFERE do da ferramenta (que é o do primário) | `ph2d-app-vec/src/vector_bridge.rs:219-237`, `ph2d-tool-vector/src/tool.rs:82-91,545-553` |
| ⛔ ⇒ **escolher exatamente a cor do primário com o Fill «misto» não faz nada**: o artista clica, nada arma, as outras formas ficam com a cor delas | idem |
| O seletor **não tem sinal de «foi tocado»**: `blender_picker` devolve 4 campos de ESTADO. Mas o despacho JÁ emite `WidgetEvent::ValueChanged(INSP_BLENDER_PICKER)` em cada toque (`apply_blender_hit` no Down, o arrasto no Move) — e ninguém o consome: está na lista de «esperado sem dono» | `ph2d-editor-core/src/interaction/state/blender_ops.rs:104-117`, `dispatch/pointer_down.rs:618-621`, `dispatch/mod.rs:509-516` |
| ⛔ **Uma forma sem preenchimento nunca ganha cor pela escolha**: «sem preenchimento» = alfa 0; o seletor abre com a alfa do primário; mexer só no matiz arma `FILL_RGB` e `fill_merge` guarda a alfa 0 da forma → `apply_fill_colour` lê alfa 0 como «apagar». Vale para a seleção múltipla (a forma vazia fica vazia ao lado da cheia que mudou) **e para uma forma só** (abre com alfa 0, o matiz muda, nada aparece) | `vector_bridge_fields.rs:115-123`, `vector_bridge_style.rs:191-210,227-237` |
| ⛔ **A pilha de camadas lê `sel.first()`**, não o primário da caneta (`pen.selected()`): selecionar a azul e depois a vermelha (primário = vermelha) mostra as camadas da azul ao lado da Width da vermelha | `ph2d-app-vec/src/paint_stack.rs:150-172`, chamada em `vector_bridge_publish.rs:119` |
| ⛔ **A pilha não sabe «misto»**: opacidade e mistura do objeto já sabem (F20); as linhas de camada mostram as do primário, e um clique na linha `i` escreve na `i` de CADA forma — que pode ser outra espécie (contorno numa, preenchimento noutra) | `paint_stack.rs:1-15,150-172`, `ph2d-panel-vector/src/paint.rs:111`, `state_appearance.rs:22-73` |
| `vector_mode.rs` em 697/700 LOC; o roteador das famílias de estágio (140..=253) é um bloco contíguo `553-605` de braços idênticos | `ph2d-app-vec/src/vector_mode.rs:553-605`, gate `architecture_workspace_file_loc_cap.rs:39` |
| O teste das faixas de estágio NÃO inclui o `misto` (225..=229) — só os `const assert` o protegem | `vector_mode_scenes_tests.rs:147` |

## §2 — Estado da arte (e o que foi tentado e largado)

| produto | lei | tentado e largado |
|---|---|---|
| **Illustrator** | amostra «?» quando a seleção difere; clicar numa amostra **aplica-a a todos**, inclusive aos que estavam em *None* — o clique é o gesto, não a diferença de valor. O painel Aparência mostra «Aparências mistas» quando as pilhas diferem, e *Add New Fill/Stroke* continua a valer para todos | o bug de trocar a cor do traço apagar o preenchimento ([fórum Adobe](https://community.adobe.com/t5/illustrator-discussions/changing-same-stroke-color-of-multiple-objects-deletes-the-fill-color/td-p/12131032)) — a escrita que vaza de campo (curada no F20) |
| **Figma** | *Mixed* por propriedade; com preenchimentos diferentes o painel não lista camadas — oferece substituir todas por uma; escolher uma cor aplica a todas ([cores numa seleção mista](https://help.figma.com/hc/en-us/articles/360042553434)) | — |
| **Inkscape** | Fill & Stroke com vários objetos: escolher «cor lisa» num objeto sem preenchimento dá-lhe a cor **opaca** | — |
| **Este app** | Inspector: BulkSelect (número em branco); F20: amostra «—», número em branco, escrita por campo | — |

**As leis que saem disto:**
1. **Escolher é o gesto** — tocar no seletor escreve o que o seletor mostra, mesmo que seja a cor do primário.
   Só os canais que o toque MUDOU (ou, num toque sem mudança, o RGB que está à vista) são escritos: arrastar só
   a alfa continua a não pisar as cores das outras.
2. **Uma forma sem preenchimento que recebe uma cor fica VISÍVEL**: herda a alfa da escolha, e se a escolha tem
   alfa 0 (o seletor abriu numa forma vazia), opaca. As que já tinham preenchimento guardam a alfa delas.
3. **A pilha lê o primário e diz «misto»**: se as formas têm a mesma estrutura de camadas (mesmo número, mesma
   espécie por índice), as linhas aparecem e cada campo que difere pinta-se em «misto»; se a estrutura difere,
   uma linha só, «Camadas mistas», e os botões *+ Fill / + Stroke* continuam a acrescentar a todas (a escrita
   por índice numa pilha diferente era o vazamento — a linha 2 de uma forma não é a linha 2 da outra).

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta única | escreve · lê |
|---|---|---|
| **O artista tocou no seletor neste quadro?** | `WidgetStore::picker_edits() -> u64`, um contador incrementado num sítio só — o fim do `dispatch` quando a lista de eventos contém `ValueChanged(INSP_BLENDER_PICKER)`. Cada ponte guarda o SEU cursor (a lei do runtime: cada consumidor lê com o próprio) | editor-core (escreve) · `vector_bridge` passo 2 (lê) |
| **Que campos uma escolha arma?** | `VectorTool::pick_fill_rgba(rgba, tocado)` / `pick_stroke_rgba` — `rgba_fields` como hoje; tocado e nada mudou ⇒ o RGB. `set_*_rgba` ficam para o conta-gotas e a fábrica (o mesmo corpo, `tocado = true`: o conta-gotas é um gesto) | tool · ponte, passo 4 |
| **Que alfa recebe uma forma vazia?** | `fill_merge` (a mesma porta do F20): RGB armado sem alfa sobre uma cor de alfa 0 ⇒ alfa da escolha, ou opaca se ela for 0. A ferramenta segue a mesma lei para si (`promote_empty_fill`), e a ponte devolve a alfa ao seletor no mesmo quadro (senão a sondagem seguinte leria a alfa 0 do seletor como «apagar») | ponte, passo 4 · tool |
| **De quem é a pilha mostrada?** | `paint_stack::published(scene, primary, sel)`: primário = `pen.selected()` se estiver na seleção, senão o primeiro | `vector_bridge_publish.rs` |
| **A pilha difere?** | `stack_mixed(scene, sel) -> StackMixed { differ: bool, rows: Vec<RowMixed> }` em `paint_stack.rs`, ao lado do `published` | publicado em `Appearance.layers_differ` + `PaintRow.mixed` |
| **Como o painel pinta uma pilha mista?** | `paint.rs` (o laço das linhas): `layers_differ` ⇒ uma linha «Camadas mistas» (chave i18n nova) e nenhuma linha por índice; senão cada linha pinta a amostra «misto» e os números em branco pelos MESMOS pintores do F20 | `ph2d-panel-vector` |

Recusas deste desenho:
- ⛔ armar `FILL_RGB` sempre que o seletor está ABERTO (sem o sinal de toque): o seletor é sondado a cada quadro
  — abrir o seletor sobre um Fill misto pintaria todas com a cor do primário sem o artista tocar.
- ⛔ armar tudo (RGB + alfa) num toque: arrastar só a alfa pisaria as cores das outras — a família do F20.
- ⛔ dar opacidade 100 % às formas vazias sempre: se o artista escolheu uma cor a 50 %, a forma vazia nasce a 50 %.
- ⛔ esconder a pilha toda em «misto» quando só os valores diferem: o Illustrator mostra as linhas quando a
  estrutura é a mesma; esconder tiraria do artista a edição da camada 2 de cinco botões iguais.
- ⛔ criar camadas ao escrever num índice que a forma não tem (lei antiga do `paint_stack.rs`, fica).

## §4 — Contrato congelado / formato

- `ph2d-editor-core/src/tool.rs`, `architecture_tool_contract_surface.rs`, `ph2d-vector-doc/src/primitives.rs`:
  não tocados (conferir por `git diff --stat` no fecho; o F20 deu vazio). O contador é do `WidgetStore`, não do
  `PanelEvent` (contrato `PanelEvent=4` intocado).
- `PROJECT_SCHEMA` intocado: nenhum campo gravado novo; `Appearance`/`PaintRow` são estado de VISTA.

## §5 — A UI: as quatro condições

Nenhum controlo novo clicável. A linha «Camadas mistas» é TEXTO (não regista id). Muda o estado pintado e a
SEQUÊNCIA:

| sequência | hoje | depois |
|---|---|---|
| vermelha (cheia) + vazia selecionadas → escolher azul no Fill | a vermelha fica azul, a vazia continua vazia | as duas azuis |
| vermelha + azul, primário vermelha → escolher exatamente o vermelho | nada | as duas vermelhas |
| uma forma vazia → abrir o Fill e mexer no matiz | nada aparece | a forma ganha a cor, opaca; o Opacity diz 100 |
| vermelha + azul → arrastar só a alfa no seletor | as alfas mudam, as cores ficam | igual (controlo) |
| selecionar azul, depois vermelha | pilha da azul | pilha da vermelha |
| pilhas com estrutura diferente | as linhas do primeiro | «Camadas mistas» + *+ Fill/+ Stroke* |
| mesma estrutura, cores diferentes na camada 1 | a cor do primeiro | amostra «—» na camada 1 |

## §6 — Gates, vermelhos primeiro

Fixture que contém o fenómeno: vermelha cheia `[220,40,40,255]` com DUAS camadas extra (contorno + preenchimento)
e uma forma FECHADA sem preenchimento com UMA camada extra (contorno de outra cor), as duas selecionadas,
primário a vermelha (selecionada por último). O despacho é o `dispatch` real e o seletor é escrito pela porta
do store, com o toque registado pela porta nova.

1. `uma_cor_escolhida_preenche_quem_nao_tinha` — **vermelho hoje**.
2. `escolher_a_cor_do_primario_escreve_em_todas` — vermelha + azul, toque com o vermelho exato. **Vermelho hoje.**
3. `uma_forma_vazia_ganha_cor_ao_mexer_no_matiz` — seleção de uma; a ferramenta e o seletor ficam com alfa 255
   no mesmo quadro, e o quadro seguinte (o seletor sondado de novo) NÃO apaga. **Vermelho hoje.**
4. Controlos: `arrastar_so_a_alfa_nao_escreve_o_rgb`; `o_seletor_aberto_sem_toque_nao_escreve`; o conta-gotas
   continua a escrever; o balde continua a não restilar a seleção; o F20 inteiro (`vector_bridge_misto_tests`).
5. `a_pilha_le_o_primario` — seleção `[azul, vermelha]`. **Vermelho hoje.**
6. `a_pilha_diz_misto_quando_as_camadas_diferem` (estrutura) e `cada_campo_da_camada_diz_misto` (valores). **Vermelho hoje.**
7. Painel: `as_camadas_mistas_pintam_uma_linha` (pintura pela porta real; a linha «Camadas mistas» aparece, os
   ids das linhas por índice não são hit-indexados).
8. Store: `o_toque_no_seletor_conta_uma_vez_por_quadro` (Down na roda conta; um quadro sem toque não conta).
9. Cenas: `as_faixas_de_estagios_nao_se_cruzam` passa a incluir `misto` e a nova família.
10. Censos: `the_shell_only_shrinks`, tectos de LOC (o `vector_mode.rs` desce com o roteador movido),
    `architecture_panel_wiring_parity`.
11. Mutação: o toque não arma o RGB (sangra 2) · o contador nunca sobe (sangra 2, 8) · `fill_merge` guarda a
    alfa 0 (sangra 1, 3) · a ferramenta não promove (sangra 3) · `published` volta a `first()` (sangra 5) ·
    `stack_mixed` sempre vazio (sangra 6, 7) · o toque arma tudo (sangra o controlo da alfa).

## §7 — Smoke (cenas novas `33` e `34`; a próxima livre conta-se em `vector_mode_scenes.rs`)

Estágios livres medidos: `235..=239` (230..=234 é do `controle`, 240 do `tracejado`).

- `PH2D_OBJECT_MODE_SMOKE=33`: vermelha cheia (com camadas extra) + uma forma vazia (com outra pilha), as duas
  selecionadas ⇒ Fill «—», a pilha diz «Camadas mistas».
- `=34`: a 33 e o toque no seletor do Fill com o vermelho exato do primário ⇒ as duas vermelhas (a vazia ganhou
  preenchimento opaco), o Fill deixa de ser «—».
- Medido ANTES pelos testes 1, 2, 5 e 6 (vermelhos sobre o código de hoje).

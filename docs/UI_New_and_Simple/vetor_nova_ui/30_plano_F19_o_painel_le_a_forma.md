# 30 — Plano F19: o painel LÊ a forma (cor, traço e opacidade depois de uma escrita alheia)

> ABERTO 2 do [handoff F18](../handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-17_F18_O_DESENHO_E_O_CORPO.md):
> *«a amostra Fill mostra a cor da FERRAMENTA e só relê da forma na troca de seleção»*. Pedido do dono (18/10):
> juntar a família inteira — todo controlo do painel do vetor que fica com o valor da ferramenta depois de
> uma mudança que não veio do painel (Show, Preview, sinal, desfazer).
>
> Foto do defeito, **já aprovada no smoke F18**: [cena 28](fotos/F18_cena28_hover.png). Medido nela:
> botão no canvas `[240,140,40]` (laranja do Hover) · Fill do painel `[40,120,240]` · FILL da barra de
> controlo `[40,120,240]`. As duas amostras mentem sobre a mesma forma.

## §1 — O mecanismo (medido no código, 18/10)

| facto | onde |
|---|---|
| A semente do estilo corre só quando o **id** do primário muda: `STYLE_TARGET == target ⇒ return` | `ph2d-app-vec/src/vector_bridge_style.rs:376` |
| O painel pinta a **tool**: as amostras Fill/Stroke por `set_widget_color(tool.fill/stroke)` a cada quadro; Opacity idem; Width do store, semeado só na seleção; Cap/Join/Align, os 6 campos do tracejado e as pontas do `snap` = `tool.ui_snapshot()` | `vector_bridge_publish.rs:65-91`, `vector_bridge_style.rs:395`, `ph2d-panel-vector/src/paint_dashes.rs:52`, `paint_markers.rs:80` |
| A barra de controlo (F16: FILL, LINE, W, OPACITY) lê as MESMAS fontes | `ph2d-panel-vector/src/bars_control.rs:112-131` |
| Escritas na forma que não passam pelo painel, nenhuma notifica: **Show** e **sinal** (`ui_state_bridge.rs:157` → `ui_state_edit::install` `:330`), **Preview** entrar/sair (`ui_preview.rs:133,155`), **desfazer/refazer** (`undo.rs:190-220`, a `VecScene` inteira trocada), morph vivo, balde, contorno, FX, sincronia de instância | levantamento completo no histórico da sessão; `VecScene` não tem revisão (`morph_live.rs:53` recusa-a de propósito) |
| ⛔ **Não é só display — APAGA.** Depois de um Show, tocar QUALQUER controlo de estilo (ex.: Width) arma `apply_to_selected`; o `differs` do passo 4 compara a forma com a tool (velha) e `apply_fill_colour` reescreve o azul por cima do laranja do Hover | `vector_bridge.rs:327-405` |
| ⛔ **O seletor de cor aberto desfaz escritas alheias.** Aberto sobre Fill/Stroke, ele chama `set_*_rgba(picked)` a CADA quadro, o que arma `apply_to_selected`; um desfazer, um Show ou o hover do Preview é revertido no quadro seguinte (também na alça de gradiente: `set_selected_grad_color`) | `vector_bridge.rs:212-226`, `ph2d-tool-vector/src/tool.rs:515-541` |
| A cura já existe, **duas vezes**, para vizinhos: os parâmetros da forma comparam com a última semente (`por_fora`) e re-semeiam; a cor da luz no Inspector re-semeia do documento fora do seletor (*«é assim que um Ctrl+Z se vê»*) | `shape_params.rs:284-291`, `ph2d-panel-inspector/src/sync_luz.rs:15-41` |

**A família, por extenso:** (L1) Fill + Opacity do fill · (L2) Stroke + Opacity do traço · (L3) Width
(slider + caixa) · (L4) Cap · Join · Align · tracejado (3 pares) · pontas (início/fim, tamanho,
arredondamento) · (L5) a escrita velha por cima da forma ao tocar um controlo · (L6) o seletor aberto que
reverte escritas alheias · (L7) a barra de controlo, que herda L1–L3 das mesmas fontes.

Fora da família (escrito para não se perder): a seleção múltipla mostra o primário em vez de «misto» (o
*«?»* do Illustrator); a opacidade conduzida pela timeline e a cor de token são da VISTA (`VecDrivenStyle`,
`vec_view.bound`) e não do `VecPath` — o token já tem rachura na amostra.

## §2 — Estado da arte (e o que foi tentado e largado)

| produto | lei | tentado e largado |
|---|---|---|
| **Illustrator** | as amostras Fill/Stroke e o painel Appearance SÃO a seleção; sem seleção, mostram o estilo do próximo objecto; seleção com valores diferentes mostra «?» | — |
| **Figma** | o painel Design é derivado da seleção em cada desenho; em modo de protótipo o painel de edição sai | — |
| **Blender** | o painel é uma VISTA dos dados (desenhado de novo a partir do RNA); campo animado ganha cor (amarelo = chave aqui, verde = interpolado, **laranja = editado e vai ser descartado** na troca de quadro) | **notificador por escritor**: o C chama `WM_event_add_notifier`, mas script Python que muda dados não redesenha o painel até o rato mexer — a resposta oficial é `area.tag_redraw()` à mão ([bf-python 2012](https://lists.blender.org/pipermail/bf-python/2012-September/005969.html)). Cada escritor novo é um esquecimento novo |
| **After Effects / Rive** | o inspector mostra o valor no instante corrente (o animado), não o último digitado | — |

**A lei que sai disto:** o painel mostra a FORMA, venha a mudança de onde vier (Illustrator/Figma/Blender);
a deteção é por **comparação de conteúdo num sítio só**, nunca por aviso de cada escritor (a lição do
notificador do Blender: 10 escritores hoje, o 11.º esquece). E o que se edita e vai ser descartado avisa
ou não se deixa editar (o laranja do Blender, o modo protótipo do Figma) — ver §5.1.

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta única | escreve · lê |
|---|---|---|
| **O que o painel viu da forma?** | `StyleSeen { id, fill: Option<Paint>, stroke: Option<StrokeSpec> }` no lugar do `STYLE_TARGET` (só id) — dados do DOCUMENTO, em mundo (o zoom não a perturba) | `vector_bridge_style` (privado) |
| **A forma mudou por mão alheia?** | `seed_style_from_selection`: re-semeia se o id mudou **OU** `(fill, stroke)` do primário ≠ `StyleSeen` | a ponte, no mesmo sítio de hoje (antes de qualquer leitura do Style) |
| **O que o próprio painel escreveu conta como alheio?** | não: `note_style_seen(pen, scene)` grava o `StyleSeen` **depois** do passo 4 (a escrita do painel) — a régua da próxima comparação é o que o painel deixou na forma | um chamador, no fim do passo 4 |
| **E o seletor aberto?** | a mesma re-semeadura, quando o `picker_target()` é Fill/Stroke, escreve a cor nova no seletor (`set_blender_value`, a porta que o clique da amostra já usa — `ui_panel_spec.rs:90`); na alça de gradiente, `selected_grad_color` | `seed_style_from_selection` |
| **Width, Opacity, tracejado, pontas, Cap/Join/Align, barra de controlo** | nenhuma porta nova: já pintam da tool/store, e a tool adota pelo `adopt_fill`/`adopt_stroke` + `reseed_style_sliders` de sempre | — |
| **Arrasto em curso** | as guardas que já existem (`SliderState::Dragging`; campo do tracejado com foco) continuam a mandar | — |

Recusas deste desenho:
- ⛔ notificar o painel a partir de cada escritor (Show, Preview, undo…): é o notificador do Blender; 10
  sítios hoje e o 11.º esquece.
- ⛔ contador de revisão na `VecScene`: recusado de propósito no `morph_live.rs:53`; a comparação de
  conteúdo custa uma comparação de `Paint`+`StrokeSpec` por quadro.
- ⛔ re-semear a cada quadro sem `StyleSeen`: brigaria com o seletor a meio do arrasto (o quadro da
  escolha lê a forma ainda velha).
- ⛔ o painel ler a forma diretamente e deixar de pintar a tool: a tool continua a ser *o estilo do
  próximo desenho* (Illustrator/Figma «último usado», `vector_bridge.rs:279-296`); muda quem a semeia.

## §4 — Contrato congelado / formato

- `grep -cE 'seed_style|StyleSeen|adopt_fill|adopt_stroke|set_blender_value|VectorStyleSnapshot|picker_target|STYLE_TARGET'`
  em `ph2d-editor-core/src/tool.rs`, `architecture_tool_contract_surface.rs`,
  `architecture_vector_contract_surface.rs` e `ph2d-vector-doc/src/primitives.rs` → **0** nos quatro (18/10).
- `PROJECT_SCHEMA` (`191`) intocado: nada é gravado; muda quem lê, não o documento.

## §5 — A UI: as quatro condições

Nenhum controlo novo. Todos existem, são pintados e registados, e o clique já chega ao barramento. O que
morre é a SEQUÊNCIA:

| sequência | hoje | depois |
|---|---|---|
| Show Hover → ler a cor no painel | mostra o azul | mostra o laranja (e a barra de controlo também) |
| Show Hover → mudar a Width → **Update** do Hover | o Update grava o Hover **com o azul** (L5) | grava o laranja com a Width nova — é assim que se edita um estado (Figma/Rive) |
| Ctrl+Z de uma cor → ler o painel | mostra a cor desfeita | mostra a cor de volta |
| seletor aberto → Ctrl+Z / Show | o seletor reverte no quadro seguinte (L6) | o seletor passa a mostrar a cor nova |
| Preview, rato sobre o botão | painel azul | painel laranja enquanto está em cima, azul ao sair |

### §5.1 — Editar durante o Preview: **decisão do dono**

Hoje o Preview barra os gestos do canvas mas deixa o painel clicável, e ao sair **restaura o mundo** (`ui_preview.rs:14-22`):
uma cor mudada no painel durante o Preview perde-se em silêncio (o «laranja» do Blender, sem o aviso).
- **(A) recomendado:** durante o Preview o painel mostra os valores vivos mas não edita (como o modo de
  protótipo do Figma); o botão *Preview* continua clicável para sair.
- **(B)** deixar como está (edita e perde ao sair).

## §6 — Gates, vermelhos primeiro

Fixture que contém o fenómeno: a forma da cena 25 (RoundRect `180×70`, fill `[40,120,240]`, traço branco
`3 px`) selecionada, e uma pose de Hover que muda **tudo da família**: fill `[240,140,40]`, traço
`[250,210,60]`, largura `6`, Round/Round, tracejado `2/1`, ponta final. A escrita alheia é pela porta REAL
(`ui_state_edit::install`, a que o Show, o sinal e o Preview usam) e, para o desfazer, pela troca da
`VecScene` inteira (o que `undo::restore` faz).

1. `o_painel_reve_a_forma_depois_de_uma_escrita_alheia` (app-vec, `dispatch` real): depois do `install`, o
   snapshot e o store mostram os 7 campos da pose. **Vermelho hoje.**
2. `tocar_um_controlo_depois_do_show_nao_apaga_o_hover`: `install` → `SetValue(VECTOR_WIDTH)` → o fill
   continua laranja e a largura muda. **Vermelho hoje** (L5).
3. `o_seletor_aberto_nao_desfaz_uma_escrita_alheia` (sólido E alça de gradiente): seletor aberto, `install`,
   3 quadros ⇒ a forma continua laranja e o seletor mostra laranja. **Vermelho hoje** (L6).
4. `o_desfazer_se_ve_no_painel`: troca da `VecScene` por um clone anterior, mesmo id selecionado ⇒ o
   painel volta. **Vermelho hoje.**
5. Controlo `o_painel_nao_briga_com_a_propria_escrita`: escolhas sucessivas no seletor aberto + slider em
   arrasto ⇒ a forma segue cada escolha e o slider não é tocado. Verde antes e depois.
6. Controlo `o_zoom_nao_re_semeia`: muda o zoom sem escrita alheia ⇒ nada re-semeia.
7. Na shell, pela sonda: cena 29 (Show) e 30 (Show + Width) — o pixel da amostra Fill do painel e o da
   barra = o pixel do botão no canvas.
8. Se (A) no §5.1: `durante_o_preview_o_painel_nao_edita` (evento de painel com o Preview ligado não muda a
   forma; ao sair o mundo volta ao bit).
9. Censos que já mordem: `the_shell_only_shrinks`, tectos de LOC, `architecture_panel_wiring_parity`.
10. Mutação: comparar só o id (sangra 1, 2, 4); não re-semear o seletor (sangra 3); nunca gravar o
    `StyleSeen` (re-semeia todo quadro — sangra 5); gravá-lo ANTES do passo 4.

## §7 — Smoke (cenas novas `29` e `30`; a próxima livre, contada em `vector_mode_scenes.rs:33`)

- `PH2D_OBJECT_MODE_SMOKE=29`: a 27, com um Hover que muda fill, traço, largura, tracejado e ponta; o
  Show do Hover ⇒ o canvas e o painel (Fill, Stroke, Width, Dash, Cap/Join, pontas) e a barra de controlo
  mostram o mesmo.
- `=30`: a 29 e depois a Width mudada pelo painel ⇒ o botão continua laranja e amarelo, mais grosso.
- Passos manuais do dono: Ctrl+Z / Ctrl+Shift+Z com a forma selecionada; o seletor aberto + Show; o
  Preview com o rato real.
- Medido ANTES (cena 28, foto aprovada): canvas `[240,140,40]` × painel `[40,120,240]` × barra
  `[40,120,240]`. A medir DEPOIS na sonda: os mesmos três pixels nas cenas 29 e 30, mais o Stroke e a
  Width.

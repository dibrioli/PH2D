# 29 — Plano F18: o DESENHO é o corpo do widget (a cor do artista e a do Hover no canvas)

> Item ABERTO 4 do [handoff de integração F16–F17](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-17_F16_F17.md):
> *«a pele Button desenha o botão do tema por cima da cor autorada — ver a cor do Hover no canvas é outra onda»*.
> Esta é a onda. Foto do defeito: [F17 cena 25](fotos/F17_cena25_aba_drawing.png) — Fill azul `[40,120,240]`,
> traço branco, raio `12,143` no painel; no canvas, o retângulo escuro do tema com «Path 0 (1)».

## §1 — O mecanismo (medido no código, 17/10)

| facto | onde |
|---|---|
| Uma forma vestida é **substituída** pela pele: `fx` → imagem; `skins` → fragmento; só no `else` o vetor desenha | `ph2d-vec-render/src/dispatch.rs:165-170` |
| A pele de `Button` recebe só rótulo, retângulo e tema; o `fill` só entra em `SkinParam.rgba` para `takes_colour()` (só a `ColorSwatch`) | `ph2d-app-vec/src/widget_live.rs:186-190`, `skin/kind.rs:137` |
| A cor do corpo vem do `bg_token` (Bg2 · BgElev · AccentSoft) | `widget/button_surface/cor.rs:39-56` |
| A lei escrita em 08/2026: *«A APARÊNCIA vem dos TOKENS, não do desenho — e isso é decisão»*, com a fronteira «o ESTADO é `Normal` … idle/hover/press é a W7» | `widget/skin.rs:17-43`, `widget_live.rs:25-30` |
| A W7 CHEGOU: estados autorados (`ObjectPose.fill/stroke/opacity/…`), máquina viva e o **modo de Preview** em que o desenho responde ao rato e a pose ESCREVE no mundo | `ph2d-ui-state/src/pose.rs:36-61`, `render_loop/ui_preview.rs:1-45` |
| ⇒ o Hover gravado muda o `fill` da forma no mundo, e a pele deita-o fora: **a máquina funciona e é invisível** | — |
| O plano-mãe já previa o degrau: *«a geometria autorada de um botão vira os parâmetros de pintura daquele widget — cantos, sombra, gradiente, estados»* | `docs/Vector Module/Estudos/PLANO_UI_UX_padrao_figma.md` §2, degrau 2 |
| 17 pintores, cada um pinta o próprio corpo (sem helper central); o tema nasce em DUAS portas: `ColorToken::resolve` (com a camada de override thread-local) e `Chrome::of`/`Widgets::of` (derivados dos `Roles` no tema moderno, sem passar pelo `resolve`) | `ph2d-tokens/src/color.rs:599`, `overrides.rs:165`, `visuals.rs:245` |
| Há gravação de cena (`VectorScene::record` → `DrawOp` com pincel e caminho) — a régua estrutural deste plano | `ph2d-vector/src/record.rs:19,126` |
| Há `contrast_ratio` WCAG e a tabela de pares; não há «escolher a tinta legível sobre um fundo» | `ph2d-tokens/src/color.rs:97`, `contrast.rs` |

## §2 — Estado da arte (e o que foi tentado e largado)

| produto | lei | tentado e largado |
|---|---|---|
| **Figma** | o componente É o desenho; estados = variantes desenhadas (*interactive components*, 2021); sem variante de Hover ⇒ nada muda | protótipos com uma cópia do ecrã por estado — largados pelas variantes (explosão de frames) |
| **Rive** | o desenho é o botão; a *state machine* anima propriedades do próprio desenho (Hover/Pressed por *listeners*) | — |
| **Flash/Animate** | o símbolo Botão tem quadros *Up/Over/Down/Hit* desenhados pelo artista; quadro vazio herda o *Up* | — |
| **Unity uGUI** | `Button.transition`: *Color Tint* MULTIPLICA a cor do gráfico | o tint multiplicativo obriga a desenhar em branco; a própria Unity acrescentou *Sprite Swap* e *Animation* para devolver a aparência ao artista |
| **Godot 4** | o tema dá o corpo (`StyleBox`) e o nó aceita *theme overrides* LOCAIS, que valem só para ele e os filhos | — |
| **CSS** | propriedades personalizadas com escopo no elemento; `contrast-color()` (CSS Color 5) escolhe a tinta legível sobre um fundo arbitrário | — |
| **Material 3** | *on-colour* derivado do contentor; o retorno de hover/press é uma *state layer* (8 % / 10 %) sobre QUALQUER cor de contentor | o M2 tingia por cor fixa, o que falhava em contentores de marca |

**A lei que sai disto:** o corpo é o DESENHO (Figma/Rive/Flash); o widget só põe o CONTEÚDO (rótulo, marca,
valor, cursor); o conteúdo escolhe a tinta legível SOBRE o desenho (CSS `contrast-color`, M3 *on-colour*); e o
estado que o artista gravou manda: **sem Hover gravado, nada muda** (Figma, Flash). O *tint* da Unity é a
recusa explícita: multiplicar estraga a cor que o artista escolheu.

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta única | escreve · lê |
|---|---|---|
| **Este tipo veste o desenho?** | `WidgetKind::wears_the_drawing() -> bool`, `match` EXAUSTIVO (o compilador cobra o 21.º tipo). `false` só em `IconButton` (o desenho já é o GLIFO) e `ColorSwatch` (o desenho já é a COR) | `widget_live::build` pergunta · nenhum outro |
| **Que corpo há debaixo?** | `SkinBody::{Theme, Drawn { under: Option<Color> }}`, novo argumento de `paint_widget_skin_with`. `under` = `Paint::primary_color` do fill VIVO (a porta que a swatch já usa); `None` = sem fill | canvas passa `Drawn`; o painel gerado passa `Theme` (§3.1) |
| **Como 17 pintores param de pintar o corpo sem 17 assinaturas novas?** | um **escopo local do tema** em `ph2d-tokens` (`scope::Drawn` com guarda RAII, pilha thread-local): enquanto vale, o tema responde pelo papel de cada token. É o *theme override* local do Godot e a propriedade com escopo do CSS — uma camada ACIMA da camada autorada, na MESMA porta | `paint_widget_skin_with` abre/fecha · `ColorToken::resolve` **e** `Chrome::of`/`Widgets::of` consultam (as duas portas onde uma cor de tema nasce) |
| **Qual é o papel de cada token?** | `ColorToken::role() -> Role::{Surface, Line, Ink, Graphic}`, `match` exaustivo. `Surface` (Bg*, BgElev, PanelBg, RailBg, WindowGround, BgScrim, `*Soft`, CodeBg) e `Line` (Border, BorderStrong) ⇒ **transparente**; `Ink` (Text1/2/3, TextDisabled, AccentFg, Code*) ⇒ a tinta legível; `Graphic` (Accent*, Danger, BorderEmph…) ⇒ fica se `≥ 3:1` contra `under`, senão a tinta legível | o escopo |
| **Que tinta se lê sobre o desenho?** | `ph2d_tokens::readable_on(under, theme) -> Color`: entre os DOIS polos do tema (`Text1` e `Bg0`), o de maior contraste WCAG — o `contrast-color()` dentro da paleta, zero hex. `Text3`/`TextDisabled` herdam a hierarquia como alfa sobre essa tinta | o escopo · o gate 3 |
| **Quem desenha o corpo?** | o `dispatch`, pelo caminho vetorial de sempre (fill, traço, padrão, pincel, efeitos da forma), e DEPOIS o fragmento da pele por cima, se `WidgetSkin.over_drawing` | `widget_live` marca · `draw_one` lê |
| **E o Hover?** | nenhuma porta nova: o Preview escreve a pose no mundo (já faz), o `dispatch` desenha o fill novo, o `build` relê o `under` VIVO e o rótulo reescolhe a tinta | — |

### §3.1 — O painel gerado (`ph2d-panel-authored`): **o dono escolheu (A), 17/10**

O mesmo desenho alimenta o painel que o app gera para si próprio (W8b). Duas respostas:
- **(A) o painel também veste a cor do desenho** (recomendado: *«a versão final da UI eu mesmo desenharei»*). O
  gerador grava a cor do corpo na linha (`body: Option<[u8;4]>`, irmão do `rgba` da swatch, ⛔ nunca o MESMO campo:
  uma chave com dois sentidos), e a pele pinta `SkinBody::Flat(cor)` com a geometria do tema e a *state layer* M3
  (8 % hover, 10 % press) para o retorno do rato. O golden do gerado muda.
- **(B) o painel segue o tema do app**; só o canvas e o jogo vestem o desenho. A divergência fica escrita no
  `widget_live` e com gate.

Recusas deste desenho:
- ⛔ um campo de corpo por pintor (17 assinaturas): é a segunda porta da aparência que a nota de 08/2026 já recusou.
- ⛔ o *tint* multiplicativo (Unity): estraga a cor escolhida.
- ⛔ hover automático quando o artista não gravou Hover: é um 2.º produtor por cima da chave do artista (memória
  *«a chave que o artista fez manda mais que a correção automática»*).
- ⛔ adivinhar o corpo pela GEOMETRIA da operação gravada (o primeiro `fill` que cobre o retângulo): heurística
  que um pintor futuro quebra calado.

## §4 — Contrato congelado / formato

- `ColorToken`, `Chrome`, `WidgetKind`, `WidgetSkins` e `dispatch` não são superfície congelada: grep em
  `architecture_tool_contract_surface` (`ph2d-editor-core/src/tool.rs`, `tests/it/architecture_tool_contract_surface.rs`)
  e em `architecture_vector_contract_surface` (`ph2d-vector-doc/src/primitives.rs`) devolve **0** ocorrências de
  `WidgetSkin|paint_widget_skin|ColorToken|overrides|WidgetKind|draw_one` (17/10).
- `PROJECT_SCHEMA` (`191`) intocado: nada novo é gravado no projeto; muda a LEI de desenho, não o documento.
  Com (A), muda só o formato do código GERADO (golden), não o projeto.

## §5 — A UI: as quatro condições

Não há controlo novo; a sequência que hoje morre a meio é a do Preview.

| peça | existe | pintada + registada | o gesto chega | a SEQUÊNCIA leva a algum lugar |
|---|---|---|---|---|
| pele vestida (Widget Skin) | sim | o canvas pinta o desenho + o conteúdo | — | o artista vê a cor dele com o rótulo legível |
| Hover gravado (States) | sim | idem | o rato no modo Preview → `ui_preview` → pose no mundo | ⛔ HOJE morre na pele; depois: fill laranja + rótulo reescolhido |

## §6 — Gates, vermelhos primeiro

Fixture que contém o fenómeno: o **documento da cena 25** (RoundRect `180×70`, fill `[40,120,240]`, traço branco,
raio 12, estados Default + Hover `[240,140,40]`) e, para o censo, corpos `{azul do dono, laranja do Hover, quase
branco, quase preto, o próprio Accent do tema}` × os 5 temas × os 20 tipos, moldura `180×70` (não quadrada: a caixa
do checkbox não pode cobrir a moldura inteira).

1. `o_corpo_da_pele_vestida_e_o_desenho` (app-vec, `build` + `dispatch` REAIS, gravados): o fill azul da forma é
   desenhado, e o fragmento da pele vem DEPOIS. **Vermelho hoje** (o `dispatch` substitui).
2. `nenhuma_superficie_do_tema_cobre_o_desenho` (editor-core, censo 20 × 5 × 5): sob `Drawn`, nenhuma operação opaca
   cobre a moldura inteira (±1 px). **Controlo:** sob `Theme`, os tipos com corpo têm ≥ 1 — prova que a régua vê
   corpos. Apanha um corpo que escape pela porta `Chrome`.
3. `o_conteudo_le_se_sobre_o_desenho` (o mesmo censo): todo `Glyphs` `≥ 4,5:1` contra `under`; todo `Graphic`
   opaco `≥ 3:1`. Vermelho com a escolha por `Text1` fixo sobre o quase-branco.
4. `cada_pele_vestida_guarda_o_seu_conteudo` (o mesmo censo): a lista do que sobrevive por tipo, por extenso (rótulo
   do Button, polegar do Toggle, valor do Slider/Progress, marca do Checkbox ligado, galo do Dropdown…; o
   `Divider` declara ZERO — o desenho É a linha). Apanha um polegar pintado com token de superfície.
5. `a_pele_do_tema_fica_ao_bit`: `Theme` é byte-idêntico (o `the_skin_paints_exactly_what_the_native_painter_paints`
   continua verde, sem edição).
6. `o_escopo_nao_deixa_residuo`: depois da pele (e num `panic` dentro dela) o `resolve` volta à resposta de antes;
   escopos aninhados; a camada autorada continua por baixo.
7. `o_hover_gravado_aparece_no_canvas` (shell, Preview + rato REAL sobre o botão): a operação do fill passa a laranja e
   a tinta do rótulo é a de `readable_on(laranja)`. **Vermelho hoje.**
8. `o_icone_e_a_amostra_nao_desenham_duas_vezes`: `IconButton`/`ColorSwatch` continuam a substituir (um glifo, não dois).
9. Censos que já mordem: `the_shell_only_shrinks`, `file_loc_caps`, i18n, paridade de painéis.
10. Mutação: o `role()` de `Bg2` trocado para `Ink`, o `over_drawing` ignorado no `draw_one`, o `readable_on`
    a devolver sempre `Text1` — cada uma tem de sangrar um gate acima.

## §7 — Smoke (cenas novas `27` e `28`; a próxima livre, contada em `vector_mode_scenes.rs:32`)

- `PH2D_OBJECT_MODE_SMOKE=27` (`vector_mode_vestidos.rs`, estágios `214..=219`): a 25 inteira e, por baixo, uma
  fila de formas vestidas pelo chip do tipo, cada uma com a sua cor (Toggle verde, Slider roxo, Checkbox claro,
  campo de texto escuro). Tudo na cor do artista, o texto legível em cada uma. O rótulo é o NOME do objecto
  («Path 0 (1)») — renomeá-lo na Hierarquia é um passo do smoke, não da cena (a cena não escreve `Name`).
- `=28`: a 27 e o *Show* do Hover ⇒ o botão fica laranja e o rótulo reescolhe a tinta. O Preview com o rato REAL
  fica como passo manual do dono (o roteiro de foto não move o rato).
- Medir na sonda antes da mensagem: o píxel do centro do botão nas duas cenas (azul / laranja), o contraste do rótulo
  nas duas, e o tempo do quadro com e sem as peles (a pele passa a desenhar o vetor E o conteúdo).

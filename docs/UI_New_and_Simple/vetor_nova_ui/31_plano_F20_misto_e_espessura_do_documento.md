# 31 — Plano F20: a seleção múltipla diz «misto» e a espessura fala na unidade do DOCUMENTO

> ABERTO 4 e 3 do [handoff F19](../handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-18_F19_O_PAINEL_LE_A_FORMA.md).
> Pedido do dono (18/10): *«pode seguir a ordem; tente pelo menos duas nesta corrida»*. As duas tarefas
> partilham a MESMA porta (o passo 4 de `vector_bridge::dispatch` e a semente do estilo), então são um bloco
> só (CLAUDE.md §0.10).
>
> ✅ **Decisões do dono (18/10), as duas recomendadas:** (1) a seleção múltipla mostra «misto» e escreve por
> campo — isto **revoga** a recusa do `appearance.rs:14` (o doc-comment muda com o código); (2) a espessura
> fala na unidade do documento, fixa a qualquer zoom, e o mesmo vale para os raios e cantos.

## §1 — O mecanismo (medido no código, 18/10)

| facto | onde |
|---|---|
| O painel semeia SÓ do primário (`pen.selected()`); nada calcula «misto» | `ph2d-app-vec/src/vector_bridge_style.rs:353-361,411-453` |
| ⛔ **Escrever em várias formas grava o estilo INTEIRO do primário em todas.** Qualquer controlo arma um único `apply_to_selected`; o passo 4 monta um `StrokeStyle` com TODOS os campos da tool (cor, cap, join, align, tracejado, pontas) e `restyle_selected_strokes` → `StrokeStyle::onto` grava-os em cada forma; e `apply_fill_colour(rgba(fill))` corre em todas as selecionadas. Mexer só na Width de uma forma vermelha e uma azul (primário vermelho) pinta as duas de vermelho | `vector_bridge.rs:329-410`, `vector_bridge_style.rs:250-270`, `ph2d-vec-scene/src/stroke_style.rs:139` |
| A tool só sabe «algo mudou» (`apply_to_selected`) e, à parte, «a largura mudou» (`width_authored`): a segunda já é a cura certa para UM campo — falta para os outros 11 | `ph2d-tool-vector/src/tool.rs:190-215,425-541`, `tool_panel_event.rs:22-60` |
| ⛔ **A lei antiga está registada e recusa o «misto»** — *«MOSTRA o primário, ESCREVE em todos»*: o terceiro estado «obriga um slider sem valor» e «o artista fica sem saber o que vai acontecer ao arrastar» | `ph2d-app-vec/src/appearance.rs:8-16` |
| …mas o **Inspector do MESMO app já mostra «misto»** (BulkSelect T2.0): `sp.mixed.opacity` deixa o slider no primário e apaga só o número — *duas leis no mesmo programa* | `ph2d-panel-inspector/src/sync_sprite_value.rs:44-60`, `sync.rs:247` |
| Os widgets do terceiro estado JÁ existem: amostra «misto» (`paint_swatch_or_mixed`), fila de escolha sem segmento aceso = «misto» (`paint_choice_row`), `CheckboxValue::Indeterminate`, número em branco que não pisa os valores ao perder o foco (`blank_number_input`) | `ph2d-editor-core/src/widget/color_swatch.rs:329`, `property_row/escolha.rs:109`, `widget/checkbox/mod.rs:28`, `interaction/state/store_hierarchy.rs:443-461` |
| **Width**: o documento já guarda a largura em MUNDO (`StrokeSpec.width`, metros); só a fronteira da tool converte pelo **zoom** (`px_to_world`): leitura `vector_bridge.rs:238-245` + `vector_bridge_style.rs:446`; escrita `vector_bridge.rs:344`; forma nova `ph2d-vec-edit/src/shape.rs:258`, `pencil.rs:322,356`, `pencil_edit.rs:140`, `lib.rs:313`, `ph2d-app-vec/src/stroke_present.rs:58` | relatório de exploração, 18/10 |
| Consequência medida (foto F19, cena 28): um traço de 3 desenhado a 100 % lê **2,952** a 98 %. E o MESMO «3» desenhado a 400 % e a 50 % dá duas espessuras diferentes no documento (12× entre si) | `fotos/F19_cena28_hover_painel.png` |
| A **mesma lei de tela** está nos raios e cantos das formas (`FieldUnit::Px` → `shapes_units.rs:34-58`, `ui * px_to_world`) | `ph2d-tool-vector/src/shapes_units.rs` |
| A porta de unidade do projeto JÁ existe e é a do Transform, régua e guias: `LengthDisplay { unit, pixels_per_meter }` com `value(world)` / `to_world(display)` e gate `to_world(value(w)) == w` | `ph2d-editor-core/src/length.rs:35-85` |
| Tracejado e pontas são MÚLTIPLOS da largura (`dash_lengths = d × width`) — herdam a lei sem tocar | `ph2d-vec-scene/src/stroke_style.rs:435-445`, `params.rs:13-22,151-168` |

## §2 — Estado da arte (e o que foi tentado e largado)

| produto | lei | tentado e largado |
|---|---|---|
| **Illustrator** | campo em branco / amostra «?» quando a seleção difere; editar UM campo muda só esse campo em todos | um bug documentado em que **trocar a cor do traço de vários objetos APAGAVA o preenchimento** — exatamente a nossa família (escrita que vaza para outro campo), tratado como defeito ([fórum Adobe](https://community.adobe.com/t5/illustrator-discussions/changing-same-stroke-color-of-multiple-objects-deletes-the-fill-color/td-p/12131032)). Espessura em pt do documento, nunca do zoom |
| **Figma** | «Mixed» por propriedade; a API devolve `figma.mixed` por campo ([update 48](https://developers.figma.com/docs/plugins/updates/2022/06/08/version-1-update-48/)); px do documento | `strokeWeight` único que devolvia valor velho com pesos individuais → passou a `mixed` |
| **Blender Grease Pencil** | espessura em unidades do MUNDO | **o modo «Screen Space» (espessura em px de tela) foi REMOVIDO no 4.3** depois de anos de queixas de escala com o zoom; quem quer o efeito usa um modificador ([migração 4.3](https://developer.blender.org/docs/release_notes/4.3/grease_pencil_migration/), [#85204](https://projects.blender.org/blender/blender/issues/85204): «a UI não indicava a conversão») |
| **Inkscape / Affinity** | largura em unidade do documento; seleção diferente = campo vazio ou média marcada | — |
| **Este app (Inspector)** | BulkSelect: slider no primário, número em branco | — |

**As leis que saem disto:**
1. **Escrever é POR CAMPO**: o gesto escreve só o campo que o artista mexeu, em cada forma selecionada.
   É isto que resolve a objeção registada no `appearance.rs` — com escrita por campo, arrastar a Width de
   uma seleção mista faz exatamente uma coisa: dá a TODAS essa espessura, e nada mais muda.
2. **Ler é por campo também**: o campo onde as selecionadas concordam mostra o valor; onde diferem, mostra
   «misto» (amostra «misto», número em branco, nenhum segmento aceso). O slider continua no primário — o
   arrasto parte de um sítio previsível (a lei do Inspector, um programa com uma lei só).
3. **A espessura é uma medida do DOCUMENTO**: o número é o mesmo a qualquer zoom e desenhar com «3» dá a
   mesma linha a 50 % e a 400 % (Illustrator, Figma, Inkscape; o Blender largou o contrário).

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta única | escreve · lê |
|---|---|---|
| **Que campos o gesto mudou?** | `Autorado` (máscara de bits) na tool, no lugar de `apply_to_selected` + `width_authored`: `FILL_RGB · FILL_A · STROKE_RGB · STROKE_A · WIDTH · CAP · JOIN · ALIGN · DASH · MARKER_START · MARKER_END · MARKER_SCALE · MARKER_ROUND`. Cada setter arma o SEU bit; cor arma só quando o valor muda (o seletor relê a cada quadro). `take_authored()` drena | tool (setters, `apply_panel_event`) · ponte, passo 4 |
| **Como se grava um campo numa forma?** | `StrokeStyle::onto_fields(old, mask, width)` em `ph2d-vec-scene` — o `onto` atual vira o caso «todos os campos» (forma nova). Para o preenchimento, a cor nova compõe-se com a da forma: `fill_merge(paint, rgba, mask)` lê a cor atual pela MESMA função que a semente usa (`seed_fill_from_paint`) e entrega a `apply_fill_colour` (a lei da espécie de tinta continua numa porta) | ponte, passo 4 |
| **As selecionadas concordam?** | `Misto` (máscara dos mesmos bits) = `mixed_of(pen, scene)` em `vector_bridge_style`, varre `selected_paths()` uma vez por quadro | publicado ao painel com o snapshot (`VectorStyleSnapshot.mixed`) |
| **Como o painel mostra «misto»?** | cada pintor pergunta ao snapshot: amostras Fill/Stroke (painel + barra de controlo) por `paint_swatch_or_mixed`; Cap/Join/Align/pontas sem segmento aceso; Width, opacidades e tracejado por `blank_number_input` (slider fica no primário) | `ph2d-panel-vector` |
| **Em que unidade a Width fala?** | `LengthDisplay` (a porta do Transform): número = `value(world)`; digitado = `to_world(n)`. A tool guarda a largura em **px do documento** (= `LengthDisplay` em Pixels), e as conversões px↔mundo de largura e de raios trocam o `px_to_world` (zoom) por `1 / pixels_per_meter` numa função só: `ph2d_app_vec::doc_px::to_world` / `of_world` | ponte + `ph2d-vec-edit` (as 6 chamadas) + `shapes_units` |

Recusas deste desenho:
- ⛔ manter «mostra o primário»: é a porta pela qual a escrita do estilo inteiro pareceu inocente; o
  Inspector já mostra «misto», e o artista vê duas leis.
- ⛔ «misto» sem escrita por campo: o número em branco diria a verdade e o arrasto continuaria a pintar
  tudo com a cor do primário.
- ⛔ largura em px de tela «como opção»: é o Screen Space que o Blender removeu; o efeito de linha fina
  constante no ecrã é dos realces (`hover_outline.rs`, `Affine::IDENTITY`), que ficam como estão.
- ⛔ mexer na tolerância do lápis (Fidelity) e no raio de clique: são da MÃO e do ecrã, devem continuar
  em px de tela (Illustrator também mede a Fidelity no ecrã).

## §4 — Contrato congelado / formato

- `grep` de `StrokeStyle|onto|Autorado|apply_to_selected|width_authored|px_to_world|LengthDisplay` em
  `ph2d-editor-core/src/tool.rs`, `architecture_tool_contract_surface.rs`,
  `architecture_vector_contract_surface.rs`, `ph2d-vector-doc/src/primitives.rs`: a conferir no 1.º commit
  (o F19 deu 0 para a família vizinha). `ph2d-vec-scene` é do motor novo, contrato **não** congelado.
- `PROJECT_SCHEMA` intocado: o documento já guarda mundo; só muda o número que o painel imprime e a
  largura de formas NOVAS desenhadas fora de 100 %.

## §5 — A UI: as quatro condições

Nenhum controlo novo: todos existem, são pintados e registados, e o clique chega. Muda o ESTADO pintado
(o terceiro, «misto») e a SEQUÊNCIA:

| sequência | hoje | depois |
|---|---|---|
| vermelha + azul selecionadas → ler Fill | vermelho (o primário) | amostra «misto» no painel e na barra |
| → mexer na Width | as duas ficam vermelhas e com o traço do primário | as duas ganham a Width; a azul continua azul |
| → escolher uma cor no seletor do Fill | as duas recebem a cor (certo) e o traço do primário (errado) | só o Fill muda |
| Width a 98 % de zoom | «2,952» | «3» a qualquer zoom |
| desenhar com Width 3 a 400 % | traço de 0,75 px do documento | traço de 3 px do documento (12 px na tela a 400 %) |
| Corner round 10 a 200 % | raio de 5 px do documento | raio de 10 |

## §6 — Gates, vermelhos primeiro

Fixture que contém o fenómeno: duas formas na mesma cena com TODOS os campos diferentes (vermelha
`[220,40,40]` com traço preto 2 Round/Round tracejado; azul `[40,120,240]` com traço branco 6 Butt/Miter,
pontas), as duas selecionadas, primário a vermelha. O despacho é o `dispatch` real.

1. `mexer_na_width_de_duas_formas_so_muda_a_width` — **vermelho hoje** (a azul fica vermelha).
2. `cada_controlo_escreve_so_o_seu_campo` — tabela: um evento por controlo (Fill, Stroke, opacidades, Cap,
   Join, Align, tracejado, pontas, Width) × os outros 12 campos das duas formas intocados. **Vermelho hoje.**
3. `o_painel_diz_misto_onde_as_formas_diferem` — `snapshot.mixed` tem os bits que diferem, nenhum dos que
   concordam; com uma forma só, máscara vazia.
4. `o_numero_misto_nao_pisa_ao_perder_o_foco` — focar e sair da Width em branco não escreve nada.
5. `a_width_e_a_mesma_a_qualquer_zoom` — mesmo traço lido a 50 %, 98 %, 400 %: o número é igual
   (`LengthDisplay::value`). **Vermelho hoje** (2,952 a 98 %).
6. `desenhar_com_3_da_a_mesma_linha_a_qualquer_zoom` — forma nova a 50 % e 400 %: `StrokeSpec.width`
   igual. **Vermelho hoje.** Idem para o raio de canto.
7. Controlos: o F19 inteiro (`vector_bridge_alheia_tests`), `o_zoom_nao_re_semeia` passa a afirmar «3»;
   `to_world(value(w)) == w`.
8. Na shell, pela sonda: cena 31 (pixel da amostra «misto» no painel e na barra) e cena 32 (as duas
   formas depois da Width: cada uma com a SUA cor).
9. Censos: `the_shell_only_shrinks`, tectos de LOC, `architecture_panel_wiring_parity`.
10. Mutação: `onto_fields` grava todos (sangra 1, 2) · um setter arma o bit errado (sangra 2) · `mixed_of`
    só olha o primário (sangra 3) · `doc_px` volta a `px_to_world` (sangra 5, 6).

## §7 — Smoke (cenas novas `31` e `32`; a próxima livre conta-se em `vector_mode_scenes.rs`)

- `PH2D_OBJECT_MODE_SMOKE=31`: duas formas (vermelha e azul, traços diferentes) selecionadas ⇒ Fill e
  Stroke em «misto» no painel e na barra; Width, Cap e Join sem valor aceso; zoom diferente de 100 % e a
  Width de uma forma só lê o número inteiro.
- `=32`: a 31 e a Width mudada pelo painel ⇒ as duas mais grossas, cada uma com a SUA cor.
- Medido ANTES: Width «2,952» a 98 % (foto F19 28). A medir na sonda: pixel das amostras «misto» e das duas
  formas depois da Width.

## §8 — Feito (18/10)

Commits `d2d8c2d7d7` (feature) e `2027a500b6` (censos). Handoff:
[F20](../handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-18_F20_MISTO_E_ESPESSURA.md). Desvios do plano:

- **Estágios das cenas 31/32 = `225..=229`** (o commit diz `225..=228`, errado); a 32 usa o 229.
- **Defeito extra curado:** `blank_number_input` nunca pintou em branco (todo pintor mostra `format_number(value)` sem foco) — `WidgetStore.mixed_numbers` + parâmetro `mixed` nos pintores de NumberInput/chip; vale também para o BulkSelect do Inspector. Mexeu ~30 ficheiros de painéis e partiu `widget/skin.rs` em `skin/number.rs`.
- **Teste 4 do §6** (`o_numero_misto_nao_pisa_ao_perder_o_foco`) saiu como `o_misto_entra_vazio_e_digitar_o_numero_do_primario_escreve` em `dispatch/tests/blur.rs`; digitar o número do primário num campo misto passou a escrever em todas.
- **`PenStyle.stroke_w` em MUNDO** (não `doc_px` na ferramenta só): 8 produtores perderam a conta `px_to_world`; Px de tela ficou nos raios de clique, fidelidade do lápis, imã e realces.
- **Vermelho primeiro não observado** (testes escritos após a cura); trocado por mutação 10/10 (lista no handoff §4).
- Fora de escopo, aberto: pilha de camadas sem «misto»; alfa de Fill por campo; cor igual à do primário no seletor misto é no-op.

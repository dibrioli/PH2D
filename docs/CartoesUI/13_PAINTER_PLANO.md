# 13 — Painel Painter em cartões: censo, fatias, decisão sobre `card.rs`

Medido em 04/11 (`paint_brush_body`, `paint.rs`; censo por sonda: `Bancada` de `censo_dos_controlos`, todas as secções abertas,
pincel de fábrica). «Ctrl» = alvos registados por secção no quadro, sem o cabeçalho; muda com o estado (Symmetry ligada, etc.).
Duas vistas no mesmo painel: **Brush** e **Layers**. 4 modos = Paint Mode do Pincel. D Digital · W Watercolor · I Impasto · Wet.

| Secção (id) | D | W | I | Wet | Especial |
|---|---|---|---|---|---|
| Topo e básicos (bloco sem título) | 20 | 10 | 16 | 15 | Paint Mode, navegador de presets (`paint_wetpaint_presets*`, 760 linhas), Sync, Blend, Color, Size, Strength, Flow, Line, Composite, Clone, Inpaint, Accumulate |
| Mask (só ferramenta Mask) | · | · | · | · | 25 ctrl; 3 caixas `card_frame`, cabeçalho próprio |
| Watercolor | – | 22 | – | – | 4 `card_frame`, 13 `card_row`, prévia do Flow |
| Impasto | – | – | 37 | – | 5 `card_frame`, lista TOOL, rig de luz |
| Wet Paint | – | – | – | 41 | Tilt e visor (E10: não redesenhar), Paint type, Óleo Real, Tuning |
| Mixing (bloco, uma linha) | ✓ | ✓ | ✓ | Pigmento | `pigment_offered`, `MisturaDoPigmento` |
| Randomize | 16 | 16 | 16 | 16 | tabela da marca (L11), cor secundária |
| Shape | 18 | 3 | 15 | 26 | curva Falloff, prévia do dab, camadas de forma, pontas, Taper (gizmo) |
| Shape Tone | 3 | 3 | 3 | 3 | rampa `paint_ramp_widget` (559 linhas) |
| Paper | 6 | 6 | 6 | 9 | parâmetros de padrão, relevo, prensa |
| Grain | 4 | 4 | 4 | 4 | Color Ramp aninhada (`Lugar::Dentro`) |
| Stroke | 43 | 42 | 42 | 42 | Spacing + Continuous (E10: não desfazer), Grid, Jitter, apply, op |
| Pen | 25 | 18 | 25 | 22 | curvas de pressão, popovers |
| Symmetry | 3 | 3 | 3 | 3 | nasce com 3; ligada, 5 a 7 |
| Tiling | 5 | 5 | 5 | 5 | 3 caixas, Reset |

Exclusivos por ferramenta: Inpaint (14 ctrl, sem secções), Selection (29), Deform (`paint_deform`, 529 linhas), Sculpt (13 no topo).
Blur, Smear, Eraser e Clone: mesmas secções, sem Randomize. Vista Layers: 2 barras (6 ícones, 4 Mask/Clip/Lock/Ref), linhas em árvore
com arrasto (`set_panel_row_ids`), `paint_rows*`, `paint_adjust/` (curva, degradê, Levels), `paint_mask*`, Texture, «Apply».

**Ajuda ilustrada:** `grep` sobre a crate dá zero: ainda não existe no painel. Nenhuma fatia a inventa; só tem de não a impedir.
**Miniaturas de pincéis:** moram no navegador de presets, no topo, e na biblioteca (`paint_wetpaint_presets_biblioteca.rs`).

## `card.rs` face à API

`card.rs` (184 linhas) não é um cartão: é a **caixa com título DENTRO de uma secção** (Wash, Brush, Water, Lighting, Mixing, …): 14
`card_frame` e 43 `card_row`, altura em `n_rows` fixo. `card_row` passa pela coluna da secção (`seccoes.rs`); `Widget::Field` e o
`CardCtx` da API têm rótulo em cima e medem a altura ao pintar. A API não tem esse nível, e a lei é «nunca um 2.º título» (05).

**Decisão: FUNDIR, sem retirar agora.** Cada fatia troca as caixas da secção dela por um grupo dentro do cartão (`linha de grupo`
da API) e as `card_row` por `Field`. `card.rs` e `seccoes.rs` ficam vivos só para `PH2D_CARDS=0` e para o corpo hospedado, e saem
com o painel antigo. **Porquê não manter dois sistemas:** duas caixas, duas colunas e duas alturas é o defeito que `seccoes.rs` matou.
**Porquê não fundir na 1.ª fatia:** `n_rows` fixo com linhas de altura medida sobrepõe; a conversão só serve secção a secção.

## Como o painel fica coerente a meio

**Todo o painel vira lista na fatia 1**, como o Vector (12). O corpo antigo pede `PaintCtx` (`store_mut` 82×, `bus_mut` 36×) e o
corpo do cartão só tem `&WidgetStore`, corre mais de uma vez e pode voar ou estar solto. Por isso o hospedeiro é outro:

- **Captura:** antes da lista, cada secção corre UMA vez com o `PaintCtx` real, com cena e `HitIndex` próprios, na posição do quadro
  anterior. Devolve fundo, título absorvido e Reset. O corpo do cartão só **repete**: cena transladada e alvos transladados.
- Vale em voo, solto e rolagem; a altura é a do próprio quadro; popovers, gizmos e rects guardados não mudam de código.
- O corpo capturado NÃO usa as linhas Figma (`cartao::com`), porque quebrariam as caixas de `n_rows`; usa só a tinta (`card_ink::com`).
- **Reset** de cada secção passa ao cabeçalho do cartão, mesmo id. **Dock Brush/Layers:** faixa presa (`ListTop`), com a prévia fixada.
- **Fixas:** Mask e a secção do meio passam a arrastar-se, como todo cartão. A razão antiga era o chip acima do governado, e o chip
  mudou para o cartão de topo a 22/10. «Reset Positions» devolve. Decidido sem o dono; fica em `10_TODO` como pergunta.

## Fatias (≈ 35 min, independentes depois da H1; ordem por valor)

| Fatia | Conteúdo | Risco |
|---|---|---|
| **H1** | Lista, faixa, captura, `PH2D_CARDS=0`, Layers hospedada, popovers. Nativos: Mixing, Symmetry, Tiling | alto: fundação |
| H2 ✅ | Topo e básicos: Paint Mode, Preset (sem tocar nos volumes), Sync, Blend, Color, Size, Strength, Flow, Accumulate (`cards_topo.rs`). Peças ainda capturadas: prévia, volumes do Wet (→ H3), Clone, Inpaint, Line, Composite (→ H8) | médio; miniaturas |
| H3 ✅ | Wet Paint e volumes em linhas de cartão (`cards_wet.rs`; lei E10: Tilt e visor iguais, avançado preservado, Body e sombra ficam; Dilution ≠ Volume). Tabela `Numero` única para as duas caras. Só o visor do Tilt capturado | alto: 41 + Óleo |
| H4 ✅ | Watercolor + Impasto em linhas (`cards_watercolor.rs`, `cards_impasto.rs`), rig por lâmpada. Capturados: prévia do Flow, knobs do Sculpt (→ H8) | médio |
| H5 ✅ | Shape + Shape Color em linhas (`cards_shape.rs`, `cards_shape_tom.rs`). Capturados: curva do Falloff, visor do Taper, gizmo Flatten, prévia, linhas por camada, editor da rampa (nunca redesenhados aqui) | alto: curva e rampa |
| H6 ✅ | Paper + Grain + Randomize em linhas (`cards_papel.rs`, `cards_grao.rs`, `cards_aleatorio.rs`). Capturados: prévias, editor da rampa, títulos Grain Behavior e Grain Colors | médio; prensa do papel é controlo de pincel |
| H7 ✅ | Stroke + Pen em linhas (`cards_traco.rs`, `cards_caneta.rs`; Spacing e Continuous juntos). Capturado: só o título do Jitter | médio: 43 + 25 |
| H8 ✅ | Composite, Line, Clone, Inpaint, Mask, Deform, Selection, Sculpt em linhas (`cards_composto.rs`, `cards_ferramenta.rs`, `cards_exclusivos.rs`). Mixing, Symmetry e Tiling deixam de correr o corpo antigo. Capturada: a prévia do Flow | médio |
| H9 parcial | Layers nativa (`cards_camadas.rs`, `cards_camadas_linha.rs`): verbos, modificadores, linha, mistura, opacidade, relevo, máscara, voo. **Falta:** ver abaixo | alto |

**O que resta capturado (o `Captura::da_peca`, nada a nível de secção):** prévia dos presets; prévias do padrão (Paper, Grain, Shape, Flow); curva do Falloff; visor do Taper; gizmo Flatten; prévia da forma; linhas por camada da Shape; editor da rampa (Shape Color, Grain, Texture); títulos que dobram (Grain Behavior, Grain Colors, Jitter); visor do Tilt; parâmetros de ajuste (curva, degradê, Levels, barras); editor da camada Texture; prévia fixada na faixa. `card.rs` e `seccoes.rs` ficam vivos para `PH2D_CARDS=0` e para estas peças. A prévia só pede `scene` e `theme`: é a próxima a sair.

Leis que nenhuma fatia desfaz: só pele, **nenhum motor, preset ou pincel muda** (por nome, `a_largura_de_fabrica_e_a_aprovada` e
`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`) · E10 · Paper press é controlo do pincel · Paint type Water/Oil · Volume ≠
Dilution · rótulo em campo não óbvio · Auto Key: `.key` nos ids de `paint_decorator_campo`, o que é ajuste não leva.

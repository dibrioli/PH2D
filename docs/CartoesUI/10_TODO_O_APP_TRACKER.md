# 10 — Nova UI em todo o app: tracker da linha (reaberta em 04/11/2026)

Leitor: a LLM que continua a linha `line/NovaUI` (worktree `Worktrees/line-NovaUI`, base `main` @ `c12143dcdf`).
Ordem do dono (04/11): «reabra a linha NovaUI, continue a nova UI em todo o app, painel por painel, padrão-ouro
absoluto; sem perguntas, sem permissões, sem propor smoke; quando eu voltar dou o sinal para pausa e smoke».
Sem push, sem integração (CLAUDE.md §0.7). Lei visual: [05](05_ESPECIFICACAO_UI_COMPLETA.md) + [`maquete/`](maquete/).

## Censo de partida (medido no código, 04/11)

Em cartões: `inspector`, `skeleton`, `widget_gallery`. API reutilizável em `ph2d-editor-core/src/panel/cards*.rs`;
o modelo mais curto é `ph2d-panel-skeleton/src/cards.rs` + `paint.rs:98-108`. Restam 26 painéis registados.

## Como se trabalha aqui

- Uma árvore, um `target`; agentes em paralelo só em crates disjuntas. Commit por caminho (`git add -- <paths>`,
  `git commit --no-verify -m … -- <paths>`), sem `-A`, sem stash.
- `ph2d-editor-core` só em passos pequenos que compilam: o WIP dele para os outros agentes.
- `PH2D_CARDS=0` continua a dar o painel de antes, em todo painel convertido.
- Decisão de produto sem o dono: segue a recomendação registada no plano, e fica nesta lista para o smoke.

## Blocos, por ordem

Estado: ⬜ por fazer · 🟨 em curso · ✅ feito (foto vista, testes por nome verdes) · o smoke do dono é à parte.

| # | Bloco | Conteúdo | Estado |
|---|---|---|---|
| A | Dívidas do Inspector | Camera card ([05 §8](05_ESPECIFICACAO_UI_COMPLETA.md)), 2.ª metade do T29 (§9), popover no tom do cartão, caixas/listas do preso | ✅ |
| B | Piloto + receita | Physics em cartões; [11_RECEITA](11_RECEITA_PAINEL_EM_CARTOES.md) nasce daqui | ✅ |
| C | Alfinete | [Plano 09](09_alfinete_nos_cartoes_PLANO.md): Material, Ordering, Object → Appearance → Fill + Stroke | ⬜ |
| D | Painéis pequenos | Tokens, Grid Settings, Wet Paint, Macros, Tags, Padding, Upscale, Equalize Sizes, Background Removal, Color Equalization, Audio Mixer, Assets, Authored, Widget Lab | ⬜ |
| E | Ferramentas | Flip, Frames, Pixel Tools | ⬜ |
| F | Hierarchy | A árvore dentro da lista flutuante | ⬜ |
| G | Vector | 41 k linhas; respeita F43/F44 (layout Figma já decidido). [Plano e fatias](12_VECTOR_PLANO.md) | 🟨 G1 de 8 |
| H | Painter | Respeita E10 (decisão do dono) | ⬜ |
| I | Desenho próprio | Timeline, Motion Graph, Audio Editor, Pixel Timeline, Pixel Preview, Code; o que neles é chrome, não o canvas | ⬜ |
| J | Resto de 05 §9 | Mola do Kanban (família 3), solto sob a barra de modos, corda sobre soltos de outro painel | ⬜ |

## Decisões tomadas sem o dono (para ele confirmar no smoke)

- Physics, as barras somem nos cartões: o número muda arrastando o ícone (Figma), a barra só existe com `PH2D_CARDS=0`.
- Physics, o cartão Debug nasce dobrado com o resumo «Bodies: n»: paga a altura de abertura, que em cartões subia 53 px.
- Lista de cartões com «×»: fica na ponta e o «…» encosta-se a ele; só nos painéis que se fecham por ele (Physics). O Bones segue sem «×».
- Facto só de leitura (Scale, Bodies) mostra o valor sem caixa: a caixa promete edição.
- Physics, glifo dos campos = a letra (regra T56), não ícone desenhado. Interaction e Joints sem marca de animação (ajustes de sessão, D17).
- Camera/par de caixas (bloco A): a marca de animação da caixa da esquerda fica colada ao nome dela; no fim da metade ela encostava na caixa da direita e lia-se como sendo dela. A marca da direita segue no fim da linha.
- Camera, Game Format sem marca de animação no cartão: é definição do projecto, não se anima (D17). Frame, Outside, Editor View e Preview ficaram com a marca de hoje, a rever no smoke.
- Subsecção dentro de um cartão (Cull Mask, Visibility Layer) tem a cara do grupo (Camera Follow), nunca um 2.º título; a fronteira de secção lá dentro não pinta chapa.
- Um booleano de script no cartão é a linha de marcar com o nome à direita; o «Reset» desce para baixo dela.
- Lista suspensa aberta de um chip de cartão: a cara da lista T16 (sombra da lista, sem moldura, nome à esquerda, realce no tom do hover), tema próprio incluído.

- D3, ferramentas de imagem: `Reset`, `Cancel` e `Apply` numa faixa presa abaixo da lista, fora dos cartões: não se dobram, não se reordenam, não saem para o canvas. `Reset` em linha própria, para não ficar ao lado do `Apply`.
- D3: os liga/desliga pintados como botão azul viraram caixas de marcar, mesmo os que armam um gesto no canvas (Pick colors, Add area, Protection brush); segue a ordem do dono na Física (um liga/desliga é um valor).
- D3, nomes novos: Padding «Recenter pivot» (era «Pivot: Recenter/Keep» a trocar no botão), Equalize «Size from» e cartões Target/Result, Background Removal «Background Colors», «Protection brush», Color Equalization cartões Histogram · Auto · Tone · Color · Sharpen · Local Contrast · Color Lookup · Stylize.
- D3, Color Equalization: Color Lookup e Stylize nascem dobrados (13 números e 4 listas não cabem); Background Removal: Detect subject vem antes de Pick colors, porque decide o que essa linha é.
- D3: sem barras e sem o sinal «+» nos números (Temperature mostra `0.25`, não `+0.25`), como no resto dos cartões. Nenhum destes números leva marca de animação (ajuste de ferramenta, D17).

- **D1, Wet Paint:** janela flutuante mantida (título move, cantos redimensionam); a altura do canto é só tecto, porque a lista abraça o conteúdo. Rótulo que não cabe em duas linhas de meia coluna ganha a linha toda.
- **D1, Design Tokens:** Mode e App Colors em cima, famílias e Scale dobradas ao abrir (paga a catraca). «Autorada» deixou de pintar o nome em acento: quem diz é o ↺ da linha. Quem segue outro na Scale diz «Follows x» por baixo.
- **D1, Macros:** cartões Record, Steps, Step (só com etapa escolhida) e Library. Stop e Delete não são vermelhos: `Widget::Button` só tem normal e primário.
- **D1, Grid Settings:** nove tipos em 3 × 3 de escolha, e não lista suspensa: cada um muda o que está abaixo, um clique, como antes. O *Snap* mora no cartão Target. A opacidade fica na barra de antes (ponte `legacy`): não tem caixa nem id de campo.
- **D2, Tags:** a árvore continua lista no cartão «All tags»; o «+ New» virou «+» no cabeçalho (dica «New tag», e a frase do vazio diz «Press +»); os verbos moram em «Selected tag», com «Delete (n tags, m objects)» em vermelho. A linha arrastada voa como o cartão do Kanban, coisa que o painel antigo não tinha (só largava).
- **D2, Assets:** continua a flutuar no mesmo sítio, com as mesmas alças; três cartões (Find, Catalogs, Library). A coluna de catálogos virou cartão empilhado e nasce dobrado, com o catálogo escolhido no resumo: a grade ganha a largura toda. A família de 7 opções virou lista suspensa (receita: mais de 5 ⇒ lista), um clique a mais. As colunas da grade esticam até à largura do cartão: o tamanho pedido decide quantas cabem. A barra do tamanho virou campo «Thumbnail size (px)». A grade corta só pela janela, não pelo rect do painel, para um cartão solto no canvas continuar a pintar.
- **D2, Audio Mixer:** a mesa fica em faixas verticais; cada faixa com fader, medidor, ganho, M/S, e por baixo as filas Pan, Tone e Low cut com rótulo (eram barras sem nome). As barras do rack viraram campos de 0 a 1, a fracção que já se lia. Reverb, Delay, Comp e Ducking nascem dobrados, com «On/Off» no resumo; a caixa de marcar lá dentro chama-se «Enabled». Os M/S acesos usam as cores de sempre; o medidor mantém as cores do sinal.
- **D2, Authored UI:** só o cromo. Cada `SectionHeader` do artista é um cartão; as linhas seguem a pele dele. O programa não escreve palavra no painel: as linhas antes do 1.º cabeçalho vão num cartão com o nome da moldura, e o título aparece duas vezes.
- **D2, Widget Lab:** só o cromo: uma lista com um cartão «Bench». O estudo mantém as cores dele porque mede as variantes antigas; passá-lo à tinta do cartão apagava o que ele compara.
- C1, cartão Object solto: tem cabeçalho sem título (o nome do objecto, o alfinete e o ×); na lista segue sem cabeçalho. Sem isto o alfinete caía em cima do olho.
- C1, Material preso: escolher `Mix` tira o componente (lei de sempre, lido em `BlendFieldEdit`, não corrido); o cartão do preso passa a «Not available», como o da lista some. A rever no smoke.
- C1, Object sem linha na Hierarquia: sem cadeado, como na lista.

- **G, Vector: PERGUNTA para o smoke.** Fill e Stroke (e Appearance) aparecem no Inspector e no painel Vector, com os mesmos ids. Ficaram nos dois: F44/F45 são o aprovado, e só o Vector tem tipo, degradê, tracejado e pontas ([12](12_VECTOR_PLANO.md)). Quer manter nos dois?
- **G, Vector: PERGUNTA para o smoke.** Advanced é um cartão com os cinco membros dentro, como subsecções. Figma não tem Advanced e a forma nova (Path Effects em lista) ainda não está decidida.
- **G1:** as abas Drawing e Interface moram no cabeçalho da lista, no lugar do título, como F17 pedia: «VECTOR» não se escreve. Snap em 2 × 2 (Shapes, Path, Cross, Guides), sem marca de animação (ajuste de ferramenta). Rotação numa linha só, porque o Vector não tem os verbos que o Inspector põe ao lado. A secção por converter fica com o corpo de hoje dentro do cartão: até G8 há cartões com caras diferentes por dentro, nunca secção fora de cartão.

## Registo por bloco

(cada bloco fechado acrescenta aqui: commits, fotos, testes, o que ficou aberto)

**Bloco A (04/11).** Mecanismo do Camera (a hipótese de 05 §8 estava errada): `close_section` dentro do corpo fechava uma chapa
`Bg1` do topo do cabeçalho até à fronteira do Cull Mask, pintada DEPOIS do título, que cobria título e caixas (o mesmo em
Anchors, com ficha aberta). Cura em `section_cards::close_with`. Os outros dois: `cabe` do par não contava a coluna de animação
(Look Through elidido) e a marca ficava a meio da linha. T29, recontagem com critério: `Checkbox::new` nas secções = 6 sítios, 4
passam por `rows::caixa_row`, 2 (Anchors, script) têm ramo do cartão antes; `property_label_row`/`paint_label_row` = 11 + 14
chamadores directos que a conta antiga não via: a porta sabe o cartão e todos fecham por `PropertyRow::fim` (fora do cartão, a
mesma conta dá `y + passo`); `paint_text` directos = 81 (72 + 9 por variável local), todos por `resolve`, que nas secções passou a
ser `card_ink::tinta` (censo `a_section_paints_through_the_card_ink`). `number_input`, `icon_button`, `slider_with_chip`,
`bitmask_grid32`, listras e realces de lista pedem a tinta; `paint_text_row` é a do cartão. Popover: o chip aberto lembra a tinta
(`lembra_popover`) e `paint_list` pinta dentro dela. Preso: as caixas já editavam o objecto certo; as LISTAS não: os popovers
pintavam com o retrato da seleção e a opção fora do rect do solto caía na seleção. Cura: `toma_sujeito_da_lista` e a guarda em
`note_card_press`. Cenas novas `PH2D_VEC_CARTOES_SMOKE=8` (Camera aberto) e `9` (lista aberta). Commits `76b2cae0ba`, `ebad56ecd1`,
`847fb583ee`, `32f3ee4b91`, `ac821bee14`. Fotos: `maquete/a_camera_antes_depois.png`, `a_lista_do_formato_no_cartao.png`; vistas e
sem arquivo: cena 6 (Player igual ao aprovado) e `PH2D_CARDS=0` na cena 8 (painel de antes). Testes por nome:
`tests/it/o_bloco_a_camera_e_presas.rs` (5), `a_section_boundary_inside_a_card_body_paints_no_plate`,
`a_pair_of_checks_never_cuts_a_name_at_any_width`, `a_chip_list_in_a_card_takes_the_card_ink`; verdes também
`every_form_row_reserves`, `nenhuma_linha_pinta_o_nome`, `no_row_paints_its_name_above`, `os_corpos_dos_componentes`, `o_bloco9`,
`o_alfinete_do_inspector`, `every_icon_in_a_card`. Mutação: 12/12 vermelhas com a cura desligada. O T56 passou a rolar: o par que
deixou de caber empilha, o cartão cresceu e o Damping saía da janela de 1800 px do teste.
ABERTO: as secções só de sprite (slice, sampling, visibility, material, render source) foram convertidas, mas hoje não se pintam
em cartões (com sprite o Inspector é o de antes): sem foto. Só Camera, Player e a lista foram fotografados; Anchors, Script,
Joint, Wheel, Constraints e o solto preso têm teste ou nada, não foto, e o tema próprio na lista só o teste viu. `paint_color_row`
(só `color_tint`, de sprite) e a amostra de cor do script seguem na forma de hoje. O corpo de letra de `paint_text`/avisos segue o
`Sm`: só a cor mudou. Dobra partilhada (05 §9) fora deste bloco. Sem clippy (fecho da linha) nem suíte.

**Bloco D2 (04/11).** Tags, Assets, Audio Mixer, Authored UI, Widget Lab em cartões: `src/cards.rs` e `tests/it/os_cartoes.rs` em cada
crate; `PH2D_CARDS=0` dá o corpo de antes (testes antigos forçam-no; foto vista de Authored e Lab). Commits `8907f74fef`, `b928ba8308`,
`8585a5f09a`, `2628597fad` e o deste registo. API nova em `panel/cards_linhas_extra.rs`: `ListRow` + `list_row` + `list_row_flight`, `CardBar` +
`bar`, `meter`, `tone_button`, `danger_button`, `problem`, `FloatHandles` + `paint_floating_list`; métricas `MIXER_*`, `BAR_*`. Testes
por gesto real: Tags 12, Assets 13, Mixer 8 + 1 de largura, Authored 5, Lab 4. Contraprovas vistas vermelhas: sem `register_rows`, sem
`set_asset_cells`, sem `link_slider_number_mapped`, «×» errado no Lab; o `adopt` do Authored SOBREVIVEU (o `populate` conhece as rows de fábrica) e deu
`a_row_authored_after_the_populate_is_alive`. Catracas: `audio_mixer` abertura `1237 → 665`; `asset_browser` saiu de
`PAINEIS_COM_ID_DERIVADO`; `um_seletor_de_cor_…` aprendeu o corpo do cartão; catraca de comandos verde sem mexer. A foto apanhou: «Master» cortado (a coluna de
rótulos à esquerda comia 50 px, e `paint_text_centered` reserva respiro), polegar a invadir a faixa ao lado. Fotos: `maquete/d_tags.png`,
`d_assets.png`, `d_assets_catalogos.png`, `d_audio_mixer.png`, `d_audio_mixer_aberto.png`, `d_authored.png`, `d_widget_lab.png` (estados por patch
temporário e HOME semeado, receita). **Aberto:** o arrasto real de miniatura para o canvas é da shell, provado só até `is_asset_cell`; medidor com sinal e
clip não fotografados (foto sem áudio); voo da linha, campo de renomear, lista suspensa aberta e faixas de relação/seletor provados por teste, sem foto (a tela virtual não aceita gesto);
`PH2D_TAGS_SMOKE` levanta o Inspector de propósito, e o painel Tags fica na aba de trás. Vermelhos alheios por nome no fim: `no_magic_numeric` (bgremoval, color-equalization, upscale), censo de canais
(`rows_par`, `INSP_VIS_*`), `a_marca_tem_a_altura_da_linha` (isenção `grid_snap`). Sem clippy nem suíte (fecho da linha).

**Bloco D3 (04/11).** Padding, Upscale, Equalize Sizes, Background Removal e Color Equalization em cartões, commit `b787452494`:
`src/cards.rs` e `tests/it/os_cartoes.rs` em cada crate; `populate`, `event` e `panel_seam!` intocados, salvo a caixa de lista do
Color Equalization (`Button` só em cartões). `PanelEvent` e `Tool` não mudaram. Fundação: `panel/cards_ferramenta.rs` (faixa dos
verbos, `prepara_numeros`, `campo`, `caixa`, `caixas_aos_pares`, `opcoes`, `cartao`), `ph2d_ui_testkit::mesa::Mesa<P>`. Abrir para a
foto: `PH2D_IMAGE_TOOL_SMOKE=<id>`, no `image_edit_mode::mirror`; a shell não cresceu. Provas: inventário, scrub com Esc, escrita, cada
caixa, opção e verbo, listas suspensas, dobra, faixa, corpo antigo; crate inteira verde nas cinco. 9 contraprovas vistas vermelhas (faixa
fora do rect, lista suspensa por baixo da faixa, sem semente, `Dropdown` em cartões, sem `declare`, sem a lei `show`, pedido no lugar da
entrega do EPX, amostra sem clique, Min px sempre escondido). Medido: a 1.ª versão pintava a faixa por cima da opção aberta. Catracas:
nenhuma mexida; estes painéis não estão nas tabelas de `quantas_entradas_tem_cada_painel`. Fotos: `maquete/d_padding.png`,
`d_upscale.png`, `d_equalize_sizes.png`, `d_bgremoval.png`, `d_color_equalization.png`, as três últimas com estados por patch
temporário. **Aberto:** a lista suspensa do cartão não rola (LUT = 16 linhas, 448 px: cabe a 1040; numa janela muito baixa não, e o
corpo antigo rolava); arrastar o número do EPX não foi visto em foto, só a semente em repouso; histograma cheio não fotografado (a
foto corre sem imagem); gesto com imagem real na mão fica para o smoke. Vermelhos alheios vistos a meio: `audio_mixer` e `asset_browser`
nas catracas de `quantas_entradas`, `rows_par` e `INSP_VIS_*` no censo de canais, `insp_camera_format_pick` em `node_id_collisions`.

**Bloco B (04/11).** Physics em cartões: `ph2d-panel-physics/src/cards.rs`, 9 cartões, mesmos ids, `populate.rs` e `event.rs`
intocados; `PH2D_CARDS=0` dá o corpo de antes (foto vista, `tests/seam.rs` 27/27 nesse caminho). Fundação: `Widget::Readout`;
`CardCtx::hint`, `check`, `toggle_triangle` (`panel/cards_linhas_extra.rs`); «×» opcional da lista
(`mark_card_list_close`); `CardCtx::choice` declara o grupo no censo; métricas `GRID_CELL`/`GRID_GAP`. Cura de família:
o Esc a meio do scrub agora avisa a barra ligada (`dispatch/card_field.rs`). Testes: `tests/it/os_cartoes.rs` (15, gesto real:
inventário de ids, scrub e Esc de cada número, escrever, cada opção/caixa/verbo/célula, arrasto na matriz, dobra, reordenar,
soltar e usar no canvas, rolar, menu do cabeçalho, marca de animação, tooltips, corpo antigo); `linhas_extra` (3). Contraprovas
vistas vermelhas: tirar o espelho do Esc, tirar o fecho da lista, tirar o grupo da escolha. Catracas medidas: comandos
`2 → 3` (o «…»), abertura `1317 → 1236`; `a_marca_tem_a_altura_da_linha` aprendeu a altura do cartão. Fotos:
`maquete/b_physics_abertura.png`, `b_physics_todos_abertos.png` (estados por patch temporário: Pull, IK, células desligadas,
verbo da fita), `b_physics_debug_dobrado_hud_por_cima.png`. Commits `2baa0f6bff`, `c4454ae705`, `03b4f5b3be`, `0eb72eac36` e o
da receita. **Aberto:** o HUD «EDIT · fps» passa por cima da lista numa janela de 1200 px (visto no Physics, a 1930 não);
o alfinete não se oferece no Physics (o mundo não tem sujeito); clippy fica para o fecho da linha.

### Bloco D1 (04/11)

Wet Paint, Design Tokens, Macros e Grid Settings em cartões, cada um com `src/cards.rs` + `tests/it/os_cartoes.rs` (gesto real) e os mesmos ids. API nova: `Widget::Mark` (sinal colorido sem clique), `Widget::ButtonOff` (verbo apagado sem rect). Catracas medidas: comandos `wet_tuning 13 → 14`, `tokens 4 → 5` (só o «…»); abertura `tokens 2302 → 2012` (`2871` com a Scale aberta). Fotos: `maquete/d_wet_paint.png`, `d_tokens.png`, `d_macros.png`, `d_grid.png`; cenas com patch temporário restaurado (Tuning aberto; etapas semeadas). **Aberto:** o Wet Paint e o Grid não têm cena de smoke que abra o painel.

### Bloco C1 (04/11)

Guarda do espalhamento + **Material, Ordering, Object, Appearance** presos, todos os controlos ([plano 09](09_alfinete_nos_cartoes_PLANO.md), com as três perguntas medidas e os achados). Foto vista: `maquete/c_alfinete_presos.png` (cena `PH2D_VEC_CARTOES_SMOKE=10`: quatro soltos presos a A, 30 %/Multiply/Add/Z 5, com B escolhido a 100 %/Normal).

- **Guarda:** `WidgetStore::card_fan_out`, pedida em `fase_bus_drain`; `card_edit_subject` diz de quem são as edições do quadro e `end_card_edit_frame` fecha-o (em `fase_transform_ops`).
- **Provas por gesto** (`ph2d-panel-registry-init`, `o_alfinete_dos_cartoes_c1`; B, B+C, A+B): `every_material_mode_…`, `the_z_typed_…`, `name_eye_and_lock_…`, `opacity_blend_and_eye_…`, `rotate_and_flip_…`, `an_unpinned_card_fans_out_and_a_deleted_object_lets_go`. Do dreno ao mundo e ao passo de undo: `a_pinned_card_edit_lands_on_its_object_only_and_the_undo_step_restores_it` (shell). Ícone do olho: `the_eye_of_the_card_…` (`ph2d-panel-inspector`).
- **Na app real, com o ponteiro real** (sonda temporária, não commitada, log no plano 09): Material e Blend no preso com B e com B+A: A muda, B não, um passo, Ctrl+Z repõe A; o alfinete sobrevive ao respawn do undo.
- **Contraprovas vistas vermelhas:** tirar a guarda (4 testes), o sujeito do commit (2), a semente da opacidade (1), a aparência do retrato (1), `dropdown::cobre` (1), `card_event_subject` (1).
- **Objecto apagado:** `ph2d_card_rope::chave::resolve` é genérica por solto; o teste cobre o efeito (`rebind_loose_pin(None)`: o gesto volta à seleção). A chave durável fica e o alfinete volta com o undo.
- **Só na foto, sem teste:** o realce do modo no Material (`Add`, o de A) e o `Multiply` do Blend.
- **Não provado, fora do C1:** a marca de animação (losango) de um campo preso e o `Reset` do Transform preso. Mix no Material preso a ver no smoke.

**Para o agente de Fill e Stroke:**

1. O `sel` de `fase_transform_ops` já é `alvos`: verbo, largura, deslocamento, opacidade e mistura da camada, dilate, join e `apply_picker` já vão ao preso quando há sujeito. Falta tudo o que passa por outro caminho: `set_kind`, `stroke_paint::apply`, as tintas de base.
2. **Publicar:** `paint_stack::aparencia_do_preso` corre em `com_cada_preso`, em `fase_snapshots_publish`, e a aparência já está no `Retrato`. Falta no retrato: `BASE_PAINTS`, `BASE_FILL_RAMP`, `current_vector_style`, `STYLE`, o pattern. O custo de um `Vec` no retrato é desprezável (medido, plano 09).
3. **Semente:** em `cards_presos::ensaia` falta correr `semeia_as_linhas` (hoje só `semeia_a_opacidade`); sem isso o hex e a opacidade da linha mostram os da seleção.
4. **Globais por objecto, ainda por resolver:** `OPEN`/`open_layer_index`, `LIGADOS` e `estilos::drena`, o seletor de cor (`HEX_PENDENTE`, `apply_picker`). A lista `estilos` tem o seu próprio rect: `note_card_press` só guarda a de `cards::dropdown` e a do store; a dos estilos tem de entrar na mesma guarda.
5. **Só depois:** acrescentar `CARD_FILL`/`CARD_STROKE` a `DO_OBJECTO`, no mesmo commit da escrita. A cena 10 estende-se em `os_quatro` (`smoke_cartoes_inspector.rs`), e o gesto em `o_alfinete_dos_cartoes_c1::preso`.
6. Decisão de produto aberta no plano 09: Fill preso com o seletor aberto, escolher outra forma. Recomendação registada: continua em A.

**Bloco G, fatia 1 (04/11).** Plano em [12](12_VECTOR_PLANO.md) (30 entradas no corpo, não ~41). Todo o painel passou a cartões de uma vez, com `PH2D_CARDS=0` a dar o de antes (`crates/ph2d-panel-vector/src/cards.rs`). Corpos nativos: Transform, Snap, Constraints, Frame, Clip. O resto: hospedeiro com o cabeçalho absorvido, como os componentes do Inspector. **A sonda** (`sonda_da_seccao`): o pintor antigo corre só até ao cabeçalho, que responde «dobrada» e devolve título e «+»; assim a condição de cada secção continua a ser a única resposta, sem pintar duas vezes nem repetir predicados. O «+» de F43/F44 passou ao cabeçalho do cartão com o mesmo id; em `rows.rs`, um cabeçalho absorvido deixou de registar a pega e a acção por cima do título. Advanced corre pelo `corre()` de sempre, dentro de um cartão: ordem, tema e notas dos membros seguem iguais.

Defeito apanhado: `seed_number_fields` re-semeava o Transform sem ver o scrub (que nos cartões não dá foco): a rotação relativa zerava e somava a mais. Curado para X, Y, W, H e R, com teste.

Catracas: altura de abertura `219 → 250` (as mesmas três secções, Snap e Advanced dobradas, provado por `the_panel_opens_with_the_same_three_sections`; os 31 px são moldura e não há o que dobrar sem esconder Fill & Stroke). Piso de `onde_comeca_o_valor` `11 → 9` (cartão tem rótulo em cima, a população encolhe; `PISO_DE_LINHAS` não mudou). Saiu a excepção «vector · vector.tab.drawing» de `nenhum_nome_por_cima_do_controlo`, que o gate mandou apagar.

Testes: `g1_os_cartoes`, 17, por gesto real; o clique em cada id de cada corpo nativo produz no barramento o MESMO que `PH2D_CARDS=0`. Crate inteira uma vez: 53 + 339 verdes. Contraprovas: 12 de 12 sangram (a 12.ª, «diferidos no slot», sobreviveu à 1.ª bateria e pediu o teste `the_left_panel_keeps_its_distance_from_the_list`). Não contraprovada: a guarda em `rows.rs`, porque mutar `editor-core` parava os outros agentes. Fotos vistas: `maquete/g_vector_{0_antes_cards_0,1_drawing,2_interface,3_painel_a_esquerda}.png`; nas 1 a 3 um patch temporário abriu os estados que a cena não semeia (a moldura, a âncora, a linha do Fill), já retirado.

Não visto no app, só no mock: soltar um cartão no canvas, arrastar para reordenar, Esc a meio do scrub. Sem foto: a lista do «+» aberta. Corpos antigos nos cartões têm a cara de antes, e os do Advanced deixam a metade esquerda vazia (G6). Vermelhos vistos e alheios a esta fatia, todos em WIP ou blocos de outros agentes: `a_marca_tem_a_altura_da_linha` (`pixel_tools`), `um_seletor_de_cor_sozinho…` (`flip`), `todo_campo_pintado_com_losango…` (`ph2d-panel-inspector/src/sections/rows_par.rs`), `no_magic_numeric` (bgremoval, color-equalization, upscale). Próxima: G2, Appearance e Fill & Stroke.

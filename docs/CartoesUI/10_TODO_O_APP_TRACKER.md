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
| C | Alfinete | [Plano 09](09_alfinete_nos_cartoes_PLANO.md): Material, Ordering, Object → Appearance → Fill + Stroke | ✅ (sobras na próxima limpeza) |
| D | Painéis pequenos | Tokens, Grid Settings, Wet Paint, Macros, Tags, Padding, Upscale, Equalize Sizes, Background Removal, Color Equalization, Audio Mixer, Assets, Authored, Widget Lab | ✅ (contraprovas do D1 na rodada R) |
| E | Ferramentas | Flip, Frames, Pixel Tools | ⬜ |
| F | Hierarchy | A árvore dentro da lista flutuante | ⬜ |
| G | Vector | 41 k linhas; respeita F43/F44 (layout Figma já decidido). [Plano e fatias](12_VECTOR_PLANO.md) | 🟨 G1 de 8 |
| H | Painter | Respeita E10 (decisão do dono) | ⬜ |
| I | Desenho próprio | Timeline, Motion Graph, Audio Editor, Pixel Timeline, Pixel Preview, Code; o que neles é chrome, não o canvas | ⬜ |
| J | Resto de 05 §9 | Mola do Kanban (família 3), solto sob a barra de modos, corda sobre soltos de outro painel | ⬜ |

## Decisões tomadas sem o dono (para ele confirmar no smoke)

- **E, Flip:** cada modo só tem os seus cartões. O que era linha do modo mora no cartão Tool (Shape, Erase, Sculpt Brush, Reset shifts, Select); as ferramentas continuam em filas de três, porque «Colorize» não cabe num quinto da coluna. Self Overlap e Airbrush viraram caixas de marcar; a linha de camada é olho, cadeado, nome, setas, com a mistura e os números por baixo.
- **E, Pixel:** cartões pela ordem de antes, salvo os pincéis guardados (saem das opções para «Saved Brushes») e a simetria (cartão próprio, depois do seletor). Select, Edit e Sprite nascem dobrados: são listas longas de verbos e a paleta fica mais acima. Os ajustes de ferramenta perderam o ponto de animação, que prometia uma animação que não existe (D17). As amostras de frente e fundo no cartão passam a abrir o seletor, como as da coluna; antes eram só de leitura. Sem redesenho da paleta, do seletor nem dos pincéis no computador.
- **E, Frames:** só o cromo: a tira mede a largura da banda e a barra leva 25 controlos. Um cartão só, sem cabeçalho, porque dobrar ou reordenar um cartão único não diz nada. Medido a `1930 × 1040`, altura da célula: `98` no corpo de antes, `48` com cabeçalho de cartão, `78` sem; a costura do topo da banda devolve o resto. A barra e as células não mudaram de cara.
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
- C2, Fill preso a A com o seletor de cor aberto, escolher outra forma: o seletor CONTINUA em A, porque é para isso que existe o alfinete. Por isso o clique fora não o fecha: fecha pelo × ou abrindo outra amostra (o da seleção segue a fechar).
- C2, o cartão preso só edita as tintas de base e a opacidade, olho e «–» das camadas, como o da lista. Tipo, degradê, padrão, tracejado, setas, reordenar e abrir camada continuam no painel Vector, sobre a seleção. O Weight de uma camada só aparece num traço que é só camada, a lei de sempre.
- C2, o cartão Fill solto e preso sai da lista («Fill · on canvas», visto na foto): a seleção volta a ter Fill na lista só com a cópia T54.
- F, Hierarchy: a lista flutua e o canvas passa por trás dela, como no Inspector. A régua vertical fica na borda da janela e a lista começa 5 px depois dela (espelho da D31); as barras do módulo saem de baixo da lista.
- F, Hierarchy: busca e filtro presos debaixo do título, fora da rolagem (Figma, Godot): numa cena grande rolavam para longe. O «+» mora no cabeçalho da lista pelo mesmo motivo.
- F, Hierarchy: um cartão só, cujo título é a contagem («18 entities · 128 components»). Linhas de 22 px, listra alternada mantida (pedido do dono, 06/09), fundo e clique na largura inteira da linha, recuo só no conteúdo.
- F, Hierarchy: arrastar linha = voo inclinado + a marca de sempre (barra antes/depois, contorno «dentro»). Sem vão a abrir: numa árvore a queda «dentro» não tem vão e a marca é a linha destacada que o dono pediu.
- F, Hierarchy: selecção vinda de fora rola até à linha (não existia, nem no painel antigo); não abre ramos recolhidos: uma linha escondida fica escondida. Auto-scroll com a mão na borda (também novo).
- F, Nodes: o grafo e o timeline dele param nas colunas, dos dois lados: uma lista alta tapava-lhes as beiradas. O canvas acaba onde a cena acaba.

- **G, Vector: PERGUNTA para o smoke.** Fill e Stroke (e Appearance) aparecem no Inspector e no painel Vector, com os mesmos ids. Ficaram nos dois: F44/F45 são o aprovado, e só o Vector tem tipo, degradê, tracejado e pontas ([12](12_VECTOR_PLANO.md)). Quer manter nos dois?
- **G, Vector: PERGUNTA para o smoke.** Advanced é um cartão com os cinco membros dentro, como subsecções. Figma não tem Advanced e a forma nova (Path Effects em lista) ainda não está decidida.
- **G1:** as abas Drawing e Interface moram no cabeçalho da lista, no lugar do título, como F17 pedia: «VECTOR» não se escreve. Snap em 2 × 2 (Shapes, Path, Cross, Guides), sem marca de animação (ajuste de ferramenta). Rotação numa linha só, porque o Vector não tem os verbos que o Inspector põe ao lado. A secção por converter fica com o corpo de hoje dentro do cartão: até G8 há cartões com caras diferentes por dentro, nunca secção fora de cartão.

## Registo por bloco

(cada bloco fechado acrescenta aqui: commits, fotos, testes, o que ficou aberto)

**Censo A2 (04/11, antes de curar).** O ramo é `cards::active()`: `LIGADO && current_inspector_sprite().is_none()`, comentário «a sprite fica no de hoje». A sprite publica-se para toda entidade com `Sprite` + `Transform` (`snapshots_inspector_sprite.rs:147`), logo o tipo decide só por essa presença. O `cards()` só conhece Object, Transform, Appearance, Fill, Stroke, Ordering, Material; o resto vem de `cards_componentes::plano` (física + opcionais).

| Selecção | Marcador | Pinta hoje | O que falta |
|---|---|---|---|
| Imagem/sprite | `Sprite` | **antigo** | tudo: Image, Use, Animation sob ela, Texture & Atlas, Color & Tint, 9-Slice, Sampling, Visibility |
| Documento do Painter | `PaintedDoc` + `Sprite` (`project_painter.rs:66`) | **antigo** | o mesmo |
| Forma vetorial, texto | `VecPathRef` | cartões | linhas abaixo |
| Flip, Motion, Pixel, esqueleto/osso | sem `Sprite` | cartões | linhas abaixo; os giros |
| Câmera, luz, som, macro, Empty, grupo | Transform + componente | cartões | linhas abaixo; os giros |
| Multi-selecção mista | a primária decide | o da primária | a frase «a editar 1 de N» |
| Nada selecionado | nenhum | cartões, lista **vazia** | a frase «Select an entity…» |

Faltas em TODO tipo em cartões, medidas por leitura de `paint_inspector` × `cards()`: **Ordering** só mostra Z index, faltam Z relative, Show behind, Order in layer, Sorting layer, Y Sort e pontos, Sorting group, Sort at root, Top level. **Sampling**, **9-Slice** e os controlos de **Visibility** (publicados para quem tem o componente) não têm cartão. Os cartões de **cópia** e **propriedades** e a frase da selecção (`paint_top_cards`) não se pintam. Os **giros** (rodar, inverter) no Transform de quem não é vetor: `vec_flip_for_id` só serve a forma, controlo morto.

**Bloco A2 (04/11).** Todo tipo de objecto em cartões, commits `d8d206fe1e`, `83de088e3a` e o deste registo. Sem `PH2D_CARDS` por tipo. Ordem: Object, cópia, propriedades, Image (+ Use, sem título, não dobra, como a linha fixa de antes), Animation só no uso Animated, Visibility, Transform, Appearance/Fill/Stroke, Texture & Atlas, Color & Tint, 9-Slice, Ordering, Sampling, Material, componentes. Alfinete: Render, Color, Slice, Sampling e os cartões novos nascem sem ele (escrevem pela seleção, plano 09). A família da queixa do dono: nenhum nome à esquerda, barra, número com setas, amostra sem código ou botão-faixa num cartão. API nova: `corpo::{bar_field, fields_in, swatch_in, swatch_grid, LABEL_PX}` (`panel/cards_corpo_campos.rs`), `CardRows::swatch`, `paint_button` e `caixa_do_botao` sabem o cartão, `widget::censo_cartao`. Decisão sem o dono: a Animation é o cartão logo abaixo da Image, não uma subsecção dentro dela. Portão: `nenhum_layout_antigo_no_cartao` (pintura gravada da sprite e do Script, varredura de `sections/`), `PENDENTES` vazio. Testes: `o_inspector_em_cartoes_por_tipo` (inventário id a id contra `PH2D_CARDS=0`: só as 2 barras ficam de fora; clique id a id pede o mesmo; escrita em Opacity e Emissive). Contraprovas: 13/13 vermelhas. Fotos: `maquete/a2_imagem_animada.png`, `a2_documento_do_painter.png`, `a2_imagem_9_slice.png`, `a2_script.png`, `a2_camera_ordering.png`, `a2_nada_selecionado.png`, `a2_forma_vetorial.png`. Medido: `os_corpos_dos_componentes` subiu a janela de 1800 para 2400 px, botões na altura de campo empurraram Init Velocity para fora. **Aberto:** o gesto só provou a cena da sprite e do Script: os outros componentes têm a varredura de fonte, não pintura gravada. Seleção mista, cópia e propriedades têm teste, sem foto; luz, som, macro e Flip selecionados não fotografados (as cenas abrem sem seleção). Marca de `Flip H` perto do nome, como antes. `onde_comeca_o_valor` vermelho em `audio_editor`, ficheiro `cards_fileira.rs` em WIP de outro agente (I2), não desta fatia.

**Bloco E (04/11).** Flip, Pixel e Frames em cartões, commits `21772a0937`, `f28bef813e`, `9994d86ed2`, `43449f82fa` e o de Frames. Testes por gesto: `ph2d-panel-flip` 52, `ph2d-panel-pixel-tools` 74, `ph2d-panel-flip-frames` 21, verdes; os gates antigos correm no corpo de `PH2D_CARDS=0` (`host_de_antes`). API nova: `CardCtx::bare` público, `Rotulo::vista_de`, `corpo::choices` sem ids e sem nome; `pixel_layer_row` pinta com `card_ink::tinta`, igual fora de um cartão. Fotos: `maquete/e_flip.png`, `e_frames.png`, `e_pixel.png`, `e_pixel_menus.png`. Nenhuma catraca mexida: Flip, Frames e Pixel não têm linha nas tabelas de `quantas_entradas_tem_cada_painel`. A caixa da cor sem campo de texto passa a abrir o seletor inteira (`Widget::Swatch`): era um alvo de 26 px numa caixa da largura da fileira, e `um_seletor_de_cor_sozinho…` acusou-a no Flip. `a_marca_tem_a_altura_da_linha` mede a lista assente: o 1.º quadro cortava uma marca do Pixel a 24,5 px. Fotos dos estados: `e_flip_fill.png`, `e_flip_colorize_edit_sculpt.png`. Aberto: a mão ainda não arrastou uma camada do Flip (só setas, como antes); o estado do seletor em Normal Map e o texto do Pixel só têm teste, sem foto.

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

### Bloco C2 (04/11)

**Fill e Stroke presos.** Commits `ffd18494e9`, `a51c331530`. Crate nova `ph2d-app-vec-preso` (a `ph2d-app-vec` ficava 309 linhas acima do tecto; ficou em 93 623 de 93 637). Foto vista: `maquete/c_alfinete_fill_stroke.png` (cena `PH2D_VEC_CARTOES_SMOKE=11`: Fill e Stroke presos a Shape A e a Shape B, com B escolhido; cada cartão mostra as cores, o Weight, o Cap e o Join do seu).

Por gesto real, com as portas reais do vetor (`o_alfinete_fill_stroke_c2`, 8 testes; prende-se a A; seleção B, B+C, A+B; A muda, B e C ficam byte a byte, a tool fica a da seleção):

| Controlo | Prova |
|---|---|
| Cor (base Fill, base Stroke, camada) | `the_colour_picker_opened_…`: semente de A, escreve em A, continua em A com outra forma escolhida |
| Hex (as três linhas) | `the_hex_typed_…`: fica em A, o seletor que ele abriu fecha ao chegar a cor |
| Opacidade da base e da camada, olho, «–», «+» | `the_base_fill_row_…`, `the_layer_rows_…` |
| Largura, Cap, Join, Position pela lista, olho, «–» do Stroke | `every_control_of_the_pinned_stroke_card_…` |
| Largura de camada | `the_layer_rows_…`, num traço só de camada |
| Estilos ⠿: abrir, aplicar, soltar | `the_colour_styles_…`: só A liga, a vista da seleção não herda |
| Editar estilo («Edit style», Rename, Duplicate, Delete) | **sem prova nova**: são do estilo, não do objecto; aplicam-se a todas as formas ligadas |
| Criar estilo («+») | **sem prova**: o pedido usa as mesmas formas do aplicar (lido, não corrido) |
| Camada aberta independente | `the_layer_rows_…`, `every_control_…`: a aberta da seleção não muda e o Cap vai à base de A |
| Dois soltos em objectos diferentes | `two_pinned_fill_cards_…`: número, seletor e ⠿ de cada um |
| Degradê | `a_gradient_on_the_pinned_…`: a escolha não o esmaga |
| Tipo de tinta, degradê como editor, pattern, tracejado, reordenar, abrir camada | **não existem no cartão** (lido em `cards_pilha.rs`); o retrato não leva o pattern |

Desfazer: `a_pinned_fill_and_stroke_edit_…` (shell): verbos, olho e cor pelas portas, sobre `ProjectState`; repõe A, B intacto. **Não corrido na app real com o ponteiro real**, ao contrário do C1.

Contraprovas vistas vermelhas: `alvo_do_gesto` fora (4), `seletor_na_base` fora (1), `aponta` a escrever na aberta (1), `devolve_o_estilo` fora (2), `semeia_as_linhas` fora do ensaio (1), `apply_no_alvo` sem o alvo (3), a vista da lista de estilos a seguir o gesto (1). Do C1: semente da opacidade no `prepare` fora, `opacity_blend_and_eye_…` vermelho; teste da shell de C1 verde de novo.

Pedidos do C1: `Reset` do Transform preso é de A (`the_reset_of_the_pinned_transform_card_…`). `Mix` num Material preso tira o componente e o bloco deixa de se publicar (`mix_on_a_pinned_material_…`, shell); o cartão deixa de oferecer os modos e o alfinete fica (`a_material_card_pinned_…`). **O texto «Not available» no solto não foi testado.**

Catracas: `the_shell_only_shrinks` e `…_every_app_crate_too` verdes (tecto novo `ph2d-app-vec-preso` 395).

**Aberto:**
- ⛔ **O losango de um campo preso é o da seleção**, e clicá-lo grava a chave na seleção (lido em `fase_autokey.rs`, não corrido; plano 09). Vale para todos os presos, desde o Transform.
- Custo por quadro não medido com as tintas no retrato (plano 09 mediu só a aparência).
- `press_subject` fica até à pressão seguinte: um atalho de teclado que autorasse estilo depois de um clique no preso iria ao preso. Nenhum achado; só leitura.
- Um clique rápido duplo no olho vira `DoubleClick`, e o segundo não alterna. Vale para o painel de antes também (visto no teste).
- Tinta de base ligada a estilo com o seletor do preso: `solta_bases_mexidas` é a lei de sempre; sem teste no preso.
- Sem clippy nem suíte (fecho da linha).

### Bloco C3 (04/11)

**O losango de um campo preso é do objecto dele, e o Auto Key grava-o.** Desenho, armadilhas, seletor e custo no [plano 09](09_alfinete_nos_cartoes_PLANO.md). Commit `22f2dee1a1`. `ph2d-app-vec-preso` passou a `shells/presos` (`ph2d-host-presos`): só a composição vê várias famílias.

Por gesto real (`o_alfinete_fill_stroke_c2::losango_c3`, 9 testes; prende-se a A, B escolhido; A+B e B+C em `a_diamond_of_a_pinned_card_keys_only_…`):

| Cartão preso | Losangos | Prova |
|---|---|---|
| Transform, Appearance, Fill, Stroke | 4, 2, 2, 6: todos gravam em A, mostram a chave de A, a vista da seleção não muda, desfazer tira | `every_diamond_of_every_pinned_card_…` |
| Object, Ordering, Material | 0 solto = 0 preso | `pinning_a_card_neither_adds_nor_removes_diamonds` |
| Player (componente) | 12 | `a_diamond_of_a_pinned_component_card_…` |
| Osso no Bones | Length: mapa do osso preso, chave nele | `a_diamond_of_a_pinned_bone_card_…` |
| Lista, com um preso aberto | da seleção | `a_diamond_of_the_docked_card_…` |
| Auto Key | A em `populacao`, chave só em A, um desfazer | `an_edit_on_a_pinned_card_…` (shell) |
| Seletor | sem alfinete fecha com clique fora (lista e solto); preso fica em A com o canvas e outra forma | `an_unpinned_picker_closes_…` |
| `X` depois de pressão num preso | seletor da seleção | `the_x_shortcut_…` |
| «+», Rename, Duplicate, Delete do estilo | «+»: cor de A, liga só A; acções não mexem em B e C | `the_new_style_of_…` |
| «Not available» do Mix preso | **sem prova por texto**: o teste cobre o efeito (o cartão deixa de oferecer os modos, o alfinete fica) | `a_material_card_pinned_…` |

**Na app real:** cena `PH2D_VEC_CARTOES_SMOKE=12` (clique pelo despacho real). Sonda temporária, retirada: `sujeito = A`, `AddKey` só com `entity = A`, 6 canais do Stroke. Foto vista: `maquete/c_losango_preso.png`: Stroke de Shape A com o losango cheio em Color, a cópia de Shape B com o ponto.

**Contraprovas vistas vermelhas (2 lotes, tudo restaurado):** `c.sujeito = None`, guarda do seletor fora, `X` com `open_picker_on`, `com_marcas_de` fora do Inspector, `Novo` nas formas da seleção: 9 testes. `alvo_do_clique` sem o sujeito, seletor que nunca fecha, `com_marcas_de` fora do Bones: 5 testes. Pinos fora da `populacao`: 1 (shell). Verdes depois, na árvore final: `o_alfinete` 34 (o do osso incluído), `preso` da shell 5; `the_shell_only_shrinks`, leis das camadas, `staleness`.

**Aberto:**
- Numa pilha com camada, só a 1.ª linha do Fill tem rótulo e losango; a da base fica sem marca visível (igual solto e preso; não mudou). A sonda mostrou a chave da base gravada em A sem nada a acender.
- «Not available» do Mix preso: o texto não foi conferido por teste nem foto.
- Mapa do Player com fixtura de bits falsos: o clique passa pela porta real, o `lidos` é do teste.
- ⚠️ A shell cresceu 19 linhas (+89 −70, 57 de teste), dentro do tecto: falta mover ou cortar.
- Sem clippy nem suíte (fecho da linha). O custo mediu 5 cartões em 2 objectos.

**Bloco G, fatia 1 (04/11).** Plano em [12](12_VECTOR_PLANO.md) (30 entradas no corpo, não ~41). Todo o painel passou a cartões de uma vez, com `PH2D_CARDS=0` a dar o de antes (`crates/ph2d-panel-vector/src/cards.rs`). Corpos nativos: Transform, Snap, Constraints, Frame, Clip. O resto: hospedeiro com o cabeçalho absorvido, como os componentes do Inspector. **A sonda** (`sonda_da_seccao`): o pintor antigo corre só até ao cabeçalho, que responde «dobrada» e devolve título e «+»; assim a condição de cada secção continua a ser a única resposta, sem pintar duas vezes nem repetir predicados. O «+» de F43/F44 passou ao cabeçalho do cartão com o mesmo id; em `rows.rs`, um cabeçalho absorvido deixou de registar a pega e a acção por cima do título. Advanced corre pelo `corre()` de sempre, dentro de um cartão: ordem, tema e notas dos membros seguem iguais.

Defeito apanhado: `seed_number_fields` re-semeava o Transform sem ver o scrub (que nos cartões não dá foco): a rotação relativa zerava e somava a mais. Curado para X, Y, W, H e R, com teste.

Catracas: altura de abertura `219 → 250` (as mesmas três secções, Snap e Advanced dobradas, provado por `the_panel_opens_with_the_same_three_sections`; os 31 px são moldura e não há o que dobrar sem esconder Fill & Stroke). Piso de `onde_comeca_o_valor` `11 → 9` (cartão tem rótulo em cima, a população encolhe; `PISO_DE_LINHAS` não mudou). Saiu a excepção «vector · vector.tab.drawing» de `nenhum_nome_por_cima_do_controlo`, que o gate mandou apagar.

Testes: `g1_os_cartoes`, 17, por gesto real; o clique em cada id de cada corpo nativo produz no barramento o MESMO que `PH2D_CARDS=0`. Crate inteira uma vez: 53 + 339 verdes. Contraprovas: 12 de 12 sangram (a 12.ª, «diferidos no slot», sobreviveu à 1.ª bateria e pediu o teste `the_left_panel_keeps_its_distance_from_the_list`). Não contraprovada: a guarda em `rows.rs`, porque mutar `editor-core` parava os outros agentes. Fotos vistas: `maquete/g_vector_{0_antes_cards_0,1_drawing,2_interface,3_painel_a_esquerda}.png`; nas 1 a 3 um patch temporário abriu os estados que a cena não semeia (a moldura, a âncora, a linha do Fill), já retirado.

Não visto no app, só no mock: soltar um cartão no canvas, arrastar para reordenar, Esc a meio do scrub. Sem foto: a lista do «+» aberta. Corpos antigos nos cartões têm a cara de antes, e os do Advanced deixam a metade esquerda vazia (G6). Vermelhos vistos e alheios a esta fatia, todos em WIP ou blocos de outros agentes: `a_marca_tem_a_altura_da_linha` (`pixel_tools`), `um_seletor_de_cor_sozinho…` (`flip`), `todo_campo_pintado_com_losango…` (`ph2d-panel-inspector/src/sections/rows_par.rs`), `no_magic_numeric` (bgremoval, color-equalization, upscale). Próxima: G2, Appearance e Fill & Stroke.

### Bloco F (04/11)

Hierarchy em cartões: `ph2d-panel-hierarchy/src/cards.rs` + `row_ink.rs` + `tests/it/os_cartoes.rs`. Mesmos ids, `populate` e `event`; `PH2D_CARDS=0` pinta o painel de antes (foto conferida; testes antigos presos a ele por `force_cards_on_this_thread(Some(false))`).

**Desempenho** (`custo_por_quadro_com_cinco_mil_linhas`, `--profile smoke`, as duas pinturas intercaladas no mesmo processo, mínimo de 30 cada, load ~3,7): cartões **860 e 868 µs** contra **117 812 e 120 456 µs** do painel de antes, que pintava e registava as 5 000 linhas e copiava o mapa todo a cada quadro. A cura: `faixa` recorta à janela, e o mapa vivo passou a `Rc` (`live_entries_shared`). Só a pintura do painel, em `MockPanelHost`: o `sync_from_hierarchy` da shell não entra na conta e é igual nos dois.

**O canvas e a lista:** o canvas vai à borda da janela e a régua vertical fica lá; a lista começa 5 px depois dela. `clear_of_left_list` tira as barras do módulo e o quadro de baixo da lista (foto Vector). Nos Nodes, grafo e timeline param nas colunas, dos dois lados, e o canvas acaba onde a cena acaba (`a_floating_list_on_the_left_…`, foto). Clique dentro da lista é dela, abaixo e ao lado é do canvas (`panel_at`, teste). Largar uma linha abaixo de uma lista curta é queda no canvas, não «fim da raiz»: esse alvo são os ~18 px do fundo da lista.

**Defeito antigo curado:** o galo da Cena e dos Clips estava morto sob o dedo, nos dois modos. Ids sintéticos com 58 bits contra a guarda `COMPANION_ROW_ID_MAX` de `2^32` no despacho: `ROW_ID_MASK` desceu a 32 bits (o clique no corpo do grupo funcionava e escondia-o). Novos também: selecção de fora rola até à linha; auto-scroll com a mão na borda. Nenhum dos dois existia no painel antigo.

**Provas:** 21 testes de gesto em `os_cartoes.rs` + 1 de medição. Mutações: sem voo nem marca, sem trazer a selecção, sem auto-scroll e linha com altura de campo deram vermelho; pintar tudo não deu vermelho, deu `exit 143` (o prazo de 120 s em debug). `a_docked_panel_never_reaches` passou a contar `ancorados + listas`: a anti-vacuidade caía só porque os painéis viraram cartões.

**API nova:** `paint_card_list_top` + `ListTop` (faixa presa, «+» sem pé), `scroll_at_edge`, `CardCtx::text_box`, `HeroLayout::clear_of_left_list`, `dicas::icone` pública, ícone em `list_row_flight`, `metrics::{LIST_TOP_GAP, TREE_ROW_BLEED}`. Catraca: nada subiu; `row.rs` ficou em 577 linhas (tecto 600) com a tinta em `row_ink.rs`; a shell não foi tocada.

**Fotos:** `maquete/f_hierarchy_abertura.png`, `_muitos_com_componentes.png` (19 objectos, cena do jogo), `_ao_lado_do_inspector.png`, `_modo_vector.png`, `_modo_nodes.png`, `_voo_marca_filtro.png` (estados por patch temporário; a caixa de renomear aparece vazia porque o patch não semeia o texto). Flip e Draw olhados, sem foto guardada; Animate não olhado.

**Aberto:** o voo da linha recorta-se na beirada do cartão (igual ao Tags). A lista suspensa do filtro continua no tom do Inspector (ABERTO de 05 §9). Selecção dentro de ramo recolhido não abre o ramo. Multisselecção a voar mostra só a linha agarrada. Vermelhos alheios vistos nos mesmos portões: `um_seletor_de_cor_…` (Flip), `panel_files_under_loc_cap` (Audio Mixer), `the_shell_only_shrinks` (`ph2d-app-vec`, 93 883 contra 93 637). Os gestos que dependem da shell (Ctrl+Z, drop de asset, Delete) não foram corridos na janela: o teste vai até ao rect e ao pedido no barramento.

### Rodada R (limpeza, 04/11)

Portões por nome, medidos no fim, com o WIP dos outros na árvore. **Curados:** `no_magic_numeric` (passos de seta de D3 sem anotação; 12/12 corridas iguais: não há instabilidade no portão, ele lê a árvore de trabalho e via ficheiros de outro agente a mudar); censo de canais e `INSP_VIS_*` (`caixa_row`/`seg_row_misto` entraram em `LINHAS`, `linha_de_marcar` classificada); `no_slug_is_declared_in_two_places` (`INSP_CAMERA_FORMAT_PICK` na fundação); isenção órfã de `grid_snap` em `a_marca` (fixtura no teste do par); `the_foundation_modules_form_a_dag` (a lista suspensa e a lista de estilos de C2 publicam a capa, `note_card_press` deixou de ler `panel`); tecto de `cards_estilos.rs` (736 → 662, peças em `cards_estilos_pintura.rs`). Commits `899b26cd13`, `048aed247c`, `2a40ace5c9`.
**Vermelhos que ficam, de outros:** `the_shell_only_shrinks`: `ph2d-app-vec` 93 890 linhas no HEAD contra o tecto de 93 637, código do alfinete C2 (`preso.rs`), cura por mover; `toda_marca_booleana_tem_a_altura` em `pixel_tools` (24,5 px) e `um_seletor_de_cor_sozinho` em `flip` (w=26), do bloco E.
**Defeitos curados, com teste por gesto e mutantes vistos vermelhos:** lista suspensa do cartão rola e cabe em qualquer janela, para todo painel (`the_long_list_*`, Color Equalization, 6 mutantes; `Mesa::assenta_em`); HUD cede aos painéis que flutuam na faixa dele (`the_hud_gives_way_*`, 3); solto abaixo da barra de modos, com o alfinete a receber o clique (`a_loose_card_at_the_top_*`, 1); corda pintada depois da caminhada, por cima dos soltos de todos os painéis (`a_rope_paints_after_*`, 1). Cena `PH2D_PANEL_SMOKE=wet_tuning|grid_snap` na `panel_walk`, sem crescer a shell.
**Contraprovas D1:** 9 linhas distintas vistas vermelhas (Wet 3, Tokens 3, Macros 2, Grid 1), 2 delas só depois de um assert novo. **Sobreviventes, buracos abertos:** Wet `default` do duplo-clique na pega; Macros `mark_card_field`; Grid `set_number_value` (prova incompleta); Grid sincronia de `snap_enabled`/`show_overlay`.
**Aberto:** sem foto de nenhum destes defeitos nem das duas cenas novas: `fotografa_cena.sh` recusou a árvore, `ph2d-panel-vector` não compilava pelo WIP do bloco G, e a abertura real do Wet e do Grid pela variável só foi lida no código. Nome do HUD apertado: não vi se o texto cabe na pílula estreita. Casos de 05 §9 «Outros antigos»: não tentados.

**Bloco G, fatia 2 (04/11).** Appearance, Fill & Stroke e os três painéis à esquerda em cartão; F44/F45 mantidos (linha `[cor]◇ [Weight]◇ [%]◇ [⚙] [👁]◇ [−]`, «+» no título, painel à esquerda, arrasto pelo ⚙). Corpos nativos em `cards_tinta.rs` e `cards_traco.rs`; moldura em `cards_lado.rs` (tinta do cartão Fill & Stroke, sombras do solto). As linhas reusam o corpo: os primitivos perguntam `card_ink::card_body()`. Mesmos ids e eventos, `populate` e `event` sem mudança, contratos §6 intactos. API: nenhuma nova; `CardCtx::choice` passou a `paint_icon_toned` (Position de duas tintas, outros ícones iguais).

Decisões: Appearance numa linha, como o Inspector. Rótulo «Type» na escolha do Fill (campo sem rótulo não se explica). Offset do F44 passou a dois nomes com unidade: «Shift (px)» e «Grow (px)» (`Offset` a secas confundia). Cap, Join e a quina com os ícones `Card*` do Inspector. Lista do «+» cresce para dentro do cartão: colada ao chip, saía da lista até à borda da janela (foto).

Defeitos que o teste ou a foto apanharam: «Above Effects» sem marca de animação (`Rotulo::vista`); Position só com anéis finos (`paint_icon` numa tinta); amostra translúcida sem xadrez; sementes do tracejado, das pontas e da camada sem ver o scrub.

Testes: `g2_os_cartoes`, 19, por gesto real; o clique em cada id produz no barramento o MESMO que `PH2D_CARDS=0`. Vizinhos por nome (`g1_os`, `f44`, `f45`) verdes. Contraprovas: 19 de 19 sangram. `g1::the_left_panel_keeps_its_distance` passou a medir a moldura, não o X. Catraca: altura de abertura `250 → 260`: as mesmas duas linhas, à altura `FIELD_H` de todo campo de cartão; dobrar Fill & Stroke esconderia a razão do painel. Portões por nome: `no_magic_numeric`, `the_shell_only_shrinks`, `todo_campo_pintado_com_losango` verdes; `quantas_entradas…` sem `vector`. Vermelhos alheios vistos em `registry-init`: `painter_layers` (altura, seletor, exceção de `nenhum_nome`) e `timeline` (`a_marca`, `onde_comeca`).

Fotos vistas: `maquete/g2_vector_{0_antes_cards_0,1_lista,2_camada_fill,3_camada_traco,4_stroke_settings,5_blend_aberto,7_ponta_aberta}.png` (cenas 45, 46, 47, 49); 5 e 7 com patch temporário já retirado. A 2, a 3 e a 1 são anteriores ao xadrez e às duas tintas; a 4 já os tem.

Fica: a moldura e o corpo de `fx_side` (G3); o popover do traço da barra ② continua na cara antiga, e nele a pega de 26 px dos campos agora marcados (tracejado, pontas) faz scrub em vez de focar. Sem foto depois da cura: a lista do «+» (prova só no teste). Não visto no app, só no mock: arrasto pelo ⚙, scrub e Esc. Linha com o Weight em duas filas: só no mock. Não correram: `hr12_widgets_a11y`, `architecture_widget_showcase_coverage`, `architecture_panel_loc_cap` (ficheiros medidos abaixo de 600 linhas à mão).

**Bloco G, fatia 3 (04/11).** Effects, Filters, a rampa e o painel à esquerda deles em cartão; F43 mantido (o «+» no título, a lista, as propriedades à esquerda, ↑ ↓ ✕ no cabeçalho dele). Corpos nativos em `cards_fx.rs`; o que cada linha oferece continua a ser resposta da tabela do motor. Cada campo é o chip ligado à barra de sempre: `populate`, `event` e contratos §6 sem mudança. O «+» ancora-se no passe diferido (`fx_side::ancora_no_cartao`), não no corpo: cartão dobrado não corre o corpo e o «+» continua vivo. Lista cresce para dentro do cartão, como a de Fill & Stroke. **O painel do traço da barra ②** passou a cartão: `cards_lado::flutua` é a moldura comum e `lado` é um caso dela. Era a cura pedida: a pega de 26 px está pintada, faz scrub de verdade, e o clique no valor foca. Com a aba do painel atrás, o ramo escondido de `paint` semeia os campos (o caminho só existe no hero; o mock força o painel visível). **Mesmo defeito, outra porta:** o chip W e o da opacidade da fila ② têm o id de um campo marcado e não têm pega; curado em `dispatch_down` (`store.tool_row_slot(id) != Some(rect)`).

Decisões tomadas sem o dono: efeito com números dois a dois e o «Nome: On» de antes vira caixa de marcar com o nome (id continua botão, emite `Click`). Filtro: Radius, Size, Amount e Offset com a unidade do projecto no rótulo, Hue com «(°)»; X e Y sempre na mesma linha. O `+0.06` do Amount passa a `0.06`: sem barra não há meio de curso a distinguir. Opacity do degrau continua `0..1`, como antes. Cor: amostra + código, a caixa inteira abre o seletor. Rampa: `+` e `−` ao lado da cor do stop, em vez de fileira própria. Opacity e Blend do degrau numa linha, como no Appearance. Glifo do tipo no título do painel (F43 tinha-o).

Testes: `g3_os_cartoes`, 17, por gesto real; o clique em cada id e o número escrito produzem no barramento o MESMO que `PH2D_CARDS=0`. Vizinhos por nome (`g1_os`, `g2_os`, `f43`, `f44`, `f45`, `seam_filters`, `the_bars_carry`) verdes: 114, com os 17, no fim. Contraprovas: 23 mutações, 22 sangram (4 só depois de testes novos: passo e fábrica de cada campo, ordem da linha, semente com a aba atrás pelo hero). **Sobrevive:** `register_ramp_handles` em `fx_side`, com `seed_and_publish` a registar o mesmo um quadro depois; o teste arrasta aos 60 quadros, não mede esse quadro. Catraca: nada subiu; `cards_fx.rs` tem 573 linhas (tecto 600). `linha` entrou em `POR_FUNCAO` do censo de canais. Portões por nome verdes: `the_shell_only_shrinks`, `hr12_widgets_a11y`, `architecture_panel_loc_cap`, `todo_campo_pintado_com_losango`, `quantas_entradas…`, `a_marca…`, `toda_dica…`. **Vermelhos vistos e alheios:** `no_magic_numeric` (`context_menu_overlay_tests_cromo.rs`), `nenhuma_escolha_do_app_e_montada_a_mao` (exceção de `painter_layers`), `onde_comeca_o_valor` (8 contra o piso 9: `vector` continua contado, com 1 coluna; não medi de quem é a queda).

Fotos vistas: `maquete/g3_{1_efeito_aberto,2_filtro_sombra,3_rampa,4_lista_das_leis,5_lista_do_mais,6_traco_da_barra}.png`. Cenas 43 e 44, e patch temporário já retirado para os outros degraus e estados. Só a 2 e a 4 são posteriores à cura do par de offset, que só mexe na linha do Drop Shadow. Turbulence e Color Adjust olhados, sem foto guardada.

Fica: G4. Não visto no app, só no mock: scrub, Esc, arrasto do punho, clique do chip da fila. Sem foto: a lista do «+» dos Filters, Grow/Shrink. Por medir: o «+» de Fill & Stroke num cartão dobrado (`anota_o_menu` corre no corpo, como o de G3 corria antes desta cura). Popover da fila ② com a pega: o chip do W continua com a cara antiga, de propósito.

**Bloco H, fatia 1 (04/11).** Painel Painter em cartões, plano em [13](13_PAINTER_PLANO.md): 9 fatias, `card.rs` funde-se fatia a fatia e sai com o painel antigo. Toda a lista de uma vez, `PH2D_CARDS=0` dá o de antes (foto conferida). Brush/Layers numa faixa presa, com a prévia fixada; Reset no cabeçalho do cartão, mesmo id. Nativos: Mixing, Symmetry, Tiling. Resto: **captura** (`cards_captura.rs`): o pintor antigo corre uma vez com o `PaintCtx`, cena e alvos próprios; o corpo só repete, transladado. Vista Layers: um cartão capturado. Nenhum motor, preset ou pincel tocado.

Decidido sem o dono: Mask e a secção do meio passam a arrastar-se como todo cartão (a razão antiga era o chip acima do governado, que subiu ao topo a 22/10). **Pergunta para o smoke:** quer essas duas presas no lugar? Corpo novo «Basics» é nome meu.

Testes: `h1_os_cartoes`, 22 + 1 de medição não corrido: inventário em 13 estados (cada id do painel de antes continua pintado, salvo a barra de Segments e a rolagem antiga), mesma acção no barramento em 18 + 6 cliques, scrub e Esc, escrever, popover, dobra, reordenar, voo sem clique, solto vivo, rolagem, menu, dicas. Crate inteira uma vez antes do último ajuste: 62 + 276 verdes. `PH2D_COMPOSITE_PAINEL=digital|water|impasto|wet|layers` põe a aba à frente na cena do Composite. Catraca: altura de abertura `1 618 → 1 629` (moldura, mesmas secções, teste `the_panel_opens_with_the_same_sections`). `quantas_entradas`: a caixa da cor num corpo capturado conta como coluna da secção (acaba na borda e tem pelo menos a largura de um número). Fotos vistas: `maquete/h_painter_{0_antes_cards_0,digital,impasto,wet,layers}.png`; `_watercolor` guardada e não olhada.

Aberto: na linha 1 do Composite o nome «Blur» e «Brush» encostam-se no cartão mais estreito (no de antes já cortava, «Bru»), vai com H8. Sem foto nem gesto na janela: prévia fixada na faixa, voo, solto e popover (só mock). Corpos capturados têm a cara antiga. Por correr: `a_largura_de_fabrica_e_a_aprovada` e `a_miniatura_…` (código de pincel não mudou). `Cargo.lock` por commitar: outro agente tem-no em obra; falta-lhe `ph2d-card-stack` e `bumpalo` desta crate. Vermelho alheio: `a_marca_tem_a_altura_da_linha` (timeline, bloco I). Custo por quadro não medido.

Contraprovas da fatia H1: **zero corridas**, por falta de prazo. A primeira coisa da H2: tirar dos corpos a repetição dos alvos, o `dy` deles, a guarda do scrub e a publicação das linhas, e ver vermelho.

### Bloco I1 (04/11)

Cromo da Timeline e do Motion Graph, plano em [14](14_CHROME_TIMELINE_MOTION.md). Mesmos ids e eventos, `PH2D_CARDS=0` dá o de antes. API nova: `panel/cards_banda.rs` (moldura sem cartão, pílula, campo, caixa, segmentos e ícones em fila); `card_field` lê `∞`. Decisões: sem cartão por dentro (o `paint_card_list` do Frames custa 80 px); rótulo do campo ao lado (em cima custa 23 px). A área de trabalho não mudou.

**Altura das faixas (mock):** janela 1930: 116 → 126 px; 1600: 91 → 92; 1100: 0 → 0. Grafo: 0 px perdidos.

**Curas fora do pedido:** o HUD cobria as filas da Timeline nos Nodes (`reserve_bottom_strip` saía cedo); a pílula do grafo sobe acima do HUD, que tapava 5 dos 9 chips nos dois modos. **Catracas:** `a_marca_tem_a_altura_da_linha` passou a reconhecer a moldura sem cartão pelo «…» (medido 28 contra 22 px). Nada subiu. `onde_comeca_o_valor` acusou a coluna dos números: curado com coluna comum.

**Provas:** Timeline 15 testes de gesto (efeito igual ao de `PH2D_CARDS=0` em 26 controlos), Motion 5 testes novos. Contraprovas: 14 mutantes, todos vermelhos, mas **a corrida NÃO é limpa**: entre 13:27 e 13:29 outro agente correu o mesmo `mut.py` e mutou os mesmos ficheiros. «Sem fechar» sangrou em testes sem relação com ele. Repetir antes de contar; não houve prazo. Os mesmos testes sem mutante: 6 corridas seguidas verdes depois do incidente.

**Fotos:** `i_timeline_keys_com_chaves`, `i_timeline_arrange`, `i_timeline_clips_lista_aberta`, `i_motion_barra_e_timeline`, `i_motion_nos_abertos_zoom3x` (nós abertos por patch temporário, ampliada 3×). As três primeiras são de antes da coluna comum dos números; a `i_motion_barra` já a mostra. **Não fotografado:** Graph Editor com curvas expandidas, `∞` na Dur (só teste do texto), migalhas dentro de um grupo (só clique). Pixel: painel próprio, fora deste bloco.

**Aberto:** o «…» só oferece resets, sem efeito num painel sem cartões (controlo morto, por decidir) · régua, barra de rolagem e linhas de faixa na tinta antiga · popover no tom do Inspector. Vermelho alheio: `nenhuma_escolha_do_app_e_montada_a_mao` (exceção `painter_sidebar.toggle_dock`, Painter).


### Rodada R2 (limpeza, 04/11)

**Fotos olhadas** (`maquete/r2_*.png`; patch temporário no estado que a cena não semeia, já retirado). Abrem de verdade: `PH2D_PANEL_SMOKE=wet_tuning|grid_snap`. Confirmadas: solto no topo com o alfinete fora da barra; corda por cima do solto Target do Grid; voo da linha sem corte (`r2_voo_fora_do_recorte`). **O alfinete e o voo vistos parados: o clique continua provado só em teste.**
**A foto desmentiu o verde do R, três vezes, curado:** HUD a ceder com `59 fps ·…` em todas as leituras (agora tira leituras inteiras, diagnóstico primeiro, `the_hud_drops_whole_readings_*`); lista suspensa que cabia mas lia «Cinem…» (largura do nome mais longo, dentro da janela, `the_long_list_shows_whole_names_*`, fotos antes e depois); ponto de alargar da lista por cima de um solto, com a costura a roubar-lhe o clique (`loose_card_covers`, `a_loose_card_over_the_edge_*`; **a guarda da shell em `dock_resize.rs` não tem teste, só leitura**).
**Passeio B** (d_*, e_*, f_ao_lado, b_physics_todos): «Colour» → «Color» nos 40 valores da tabela; «Min size (px)»; «Fbk» → «Feedback»; «Add.» → «Additive» (**sem foto de depois**).
**Curados, com teste por gesto:** contadores (`{n?um|outros}` em `tr_with`; 58 frases + 44 `(s)` em 27 ficheiros; catraca `no_sentence_counts_*`); tom do filtro All; voo fora do recorte, também no Tags; multisselecção no voo, «+n more» acima de 5; seleção em ramo recolhido abre o ramo uma vez; largar abaixo de lista curta, na coluna dela, é fim da raiz; Mixer partido (`cards.rs` 655 → 349 + `cards_mesa.rs`); arrastar camada do Flip, com vão, marca e voo, as setas ficam (família `FlipLayer`, `SetValue` na Row, `move_layer_to`, um passo de desfazer, **sem foto**: a cena só tem uma camada). Decisão minha: fim da raiz vale na coluna da lista, abaixo dela; ao lado continua a ser o canvas.
**Contraprovas vistas vermelhas (12):** ramo, voo só da agarrada, «+n», tom, Flip ids/queda/vão/app, Wet `default`, Macros `mark_card_field`, Grid sincronia, Grid `set_number_value`. O Grid sobreviveu à 1.ª bateria: os campos em metros têm outra porta (`sync_meter_inputs_*`); sangrou com Subdivisions. Não mutadas: as de `editor-core` e `i18n` (voo no recorte, guarda de queda, HUD, costura, plural, largura da lista), para não parar os outros agentes; os testes delas foram só vistos verdes.
**Portões, por nome:** verdes `panel_files_under_loc_cap`, `architecture_widget_showcase_coverage`, `nenhum_nome_por_cima_do_controlo`, `onde_comeca_o_valor`; `ph2d-i18n` inteiro; `cartoes|swatch` de Hierarchy, Flip, Tags, Mixer, Galeria (48). Foto `PH2D_GALLERY_SMOKE=1`: abre, sem nada torto, mas não mostra uma amostra sem campo. **Vermelho alheio:** `the_shell_only_shrinks`, `ph2d-app-vec` 93 656 contra 93 637, com WIP de outro agente na árvore (`vector_mode_listas.rs`, +33); a shell passou.
**Censo do cromo:** [15](15_CHROME_CENSO.md), 9 fatias; nenhuma peça usa `CardInk`; o ecrã inicial não existe; Save/Export são do sistema.
**Aberto:** Grid «Opacity» ainda é o slider antigo dentro do cartão (`c.legacy`), único caso visto; Frames «Ghost [1][1]» sem rótulo; Pixel mostra `#000000`, os outros `F0F0F5` (o teste de Pixel exige o `#`, ficou); `bars_control.rs` do Vector pode dizer «1 objects» no recurso (crate de outro agente); abas de encaixe cortam o nome a 1000 px («Inspec…»); a lista suspensa aberta tapa o título do painel na janela estreita; «Used by 1 layer» sem foto. Não olhadas: as `f_*` além da `ao_lado`, `b_physics_abertura` e `_debug`, `d_grid`, `d_wet_paint` (vi as novas `r2_*`).

**Bloco H, fatia 2 (04/11).** Topo e Basics em linhas de cartão (`cards_topo.rs`), mesmos ids e eventos: Paint Mode, Preset, Favorite, Save, biblioteca, On switch, Sync, Blend, On empty, Color, Size, Strength, Flow, Accumulate, Wash, fio do Smear. As listas abrem o popover de sempre (o do Preset arrasta linhas). Ainda capturados, como peças: prévia, volumes do Wet (H3), Clone, Inpaint, Line, Composite (H8). Nenhum motor, preset ou pincel tocado. Fotos vistas: `maquete/h2_painter_{digital,wet,impasto_preset_ativo}.png` (a última com patch restaurado). `h_painter_watercolor.png` olhada: legível; a linha sob Flow Angle sai cortada (caixa de `n_rows`, vai com H4).

Dívida da H1. Testes de pincel verdes (antes da H2, que não toca o motor). Custo por quadro, Wet, perfil smoke, intercalado, mínimo de 30: `3 955 µs` cartões × `3 913 µs` antes, `+42 µs`, sem cura. `Cargo.lock` já tem as entradas, sem diff. Contraprovas da H1: `23` mutantes, `7` vermelhos, `14` sobreviveram, `2` sem compilar (erro alheio). Com a lista parada o deslocamento é zero, por isso `h1_contraprovas.rs`. 2.ª rodada, 16 mutantes: `14` vermelhos; sobrevivem `h2_coluna_do_solto` (a largura do solto é igual à da lista no mock: a guarda fica sem prova) e `h2_sem_camada` (teste reforçado depois, mutante não recorrido). Não recorreram, por prazo: `h2_cor_sem_botao`, `coluna_nao_lembrada`, `reset_sem_dica`, `sem_previa_fixada`, `faixa_sem_altura_da_previa`, `sem_fantasma`, `alturas_nao_publicadas`, `segmentos_sem_guarda`; os testes novos deles estão verdes. Defeitos reais achados: num cartão solto a lista de um corpo capturado abria na coluna (cura em `dropdown_popover`: vale o rect de agora) e um solto mudava a coluna dos outros. Saíram da captura o gémeo sem uso e a guarda do 2.º passe.

Catraca: altura de abertura `1 629 → 1 748` (rótulo em cima, mesmas secções). Sai de `FORA` a exceção das abas Brush/Layers, obsoleta desde a H1. Decidido sem o dono: Size e Strength dividem a linha, e Flow também quando cabe aos pares; o Save e a biblioteca em botões aos pares; «On empty» continua número 0/1, para não mudar o evento. Vermelhos alheios: `no_magic_numeric` (`context_menu_overlay_tests_cromo.rs`, `cromo_smoke.rs`) e `onde_comeca_o_valor` (8 contra piso 9; o Painter mede 1 coluna, a `timeline` mede 2). Sem foto nem gesto na janela: seletor de cor, popovers, solto, voo (só mock).

**Bloco M1, M2 e M8 parcial (04/11).** Porta `widget/lista_flutuante.rs`: menu, lista de cenas, transbordo, dica, título de menu aberto e as caixas internas na superfície da lista suspensa do cartão, sem mover alvo; `PH2D_CARDS=0` dá o de antes. Cena de foto `PH2D_CROMO_SMOKE=1..9`. Estado, aberto, contraprovas, decisões sem o dono e correções ao censo: [15](15_CHROME_CENSO.md). M3 a M7 esperam a referência do dono; M9 não foi olhada.

### Bloco I2 (04/11)

Sobras da Timeline e do Motion, Audio Editor, Pixel Timeline e Preview, barra do editor de scripts, sobras do R2; tabela em [14](14_CHROME_TIMELINE_MOTION.md). Mesmos ids, `PH2D_CARDS=0` dá o de antes. A shell encolheu (moldura da janela da onda foi para `ph2d-panel-audio-editor/src/cards_moldura.rs`).

**Decisões tomadas sem o dono:** o «…» da moldura sem cartão **saiu** (só repunha cartões; Timeline e Pixel Timeline; as listas com cartões conservam-no). Pixel: hex sem `#`, o formato dos outros; o teste exigia só o formato antigo, foi ajustado. Ghost: rótulos «Before» e «After». Audio: resumo da secção aberta repetido em leitura no topo do corpo, porque o cartão só o mostra dobrado. Janela da onda desceu para baixo da régua do canvas (o título estava cortado por ela) e as caixas de texto da régua e dos marcadores alargaram (liam `0…`, `…`): a onda em si não mudou.

**Contraprovas da I1, limpas:** 14 de 14 vermelhas (10 Timeline, 4 Motion; «tira uma dica» não compilou e foi trocada por «sem as dicas»). **Uma sobreviveu e pediu teste:** `∞` da Dur só tinha prova do texto (`a_dur_le_infinito_…`, depois vermelha). 3 mutantes do Motion passaram por filtro errado, sem teste nenhum; repetidos com `cartoes::tests`, vermelhos. **I2:** 16 de 16 vermelhas; «Audio tira Delivery» sobreviveu e pediu `cada_cartao_pinta_o_seu_corpo` (a lista alta passa da janela e o inventário não via).

**Defeito que só o teste apanhou:** `SEM_RECORTE` com `f32::MIN + f32::MAX` = 0: o Audio pintava lindo e nenhum alvo registava.

**Testes:** novos `os_cartoes` em Audio (9), Pixel Timeline (6), Preview (3), Opacity no Grid, `rotulo_tests`, `cromo_tests`, `folha_tinta`. Medidas: Pixel 6 → 6 linhas, 1.ª a 103 → 104 px; Preview título 42 → 42 px; Timeline 126 px.

**Fotos vistas:** `maquete/i2_{timeline_keys,timeline_graph_expandido,timeline_lista_aberta,timeline_dur_infinito,audio_editor,pixel_timeline,pixel_preview,editor_scripts}.png`; estados por patch temporário, retirado. ⚠️ `i2_pixel_preview` é de antes de centrar os verbos e baixar o título 2 px; `i2_audio_editor` de antes da cura do `SEM_RECORTE` (só alvos). **Sem foto:** Opacity do Grid, Ghost com rótulos, «1 object», barra de busca aberta, «+ Track» aberto (foto tirada, não olhada), migalhas dentro de grupo (só clique; a Timeline na cena de nesting não abriu).

**Portões, por nome:** `hr12_widgets_a11y`, `the_shell_only_shrinks`, `architecture_panel_loc_cap`, `todo_campo_pintado_com_losango` verdes. `no_magic_numeric` vermelho em dois sítios: o meu (`cards_fileira.rs`), anotado e **não recorrido**; o outro `widget/slider_with_chip/number_chip.rs:275`, WIP alheio. **Não correram, por prazo:** `quantas_entradas_tem_cada_painel`, `nenhum_nome_por_cima_do_controlo`, `onde_comeca_o_valor`, `a_marca_tem_a_altura_da_linha`, `toda_dica_dos_cartoes`. Estes vão acusar o que mudou (Audio virou lista; «…» saiu da moldura): a primeira coisa a correr. `nome_tests` de `bars_control` escrito e não corrido.

**Fica:** números das propriedades da Pixel Timeline com as setas antigas (`paint_field_row` não pinta no cartão). Corpo do Audio: mesmos pintores, recolor; sem scrub Figma, de propósito (valor é string do motor). Editor de scripts: raio dos botões da barra continua o do tema. Sem clippy nem suíte.

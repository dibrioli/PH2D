# Sprite

Estudos lidos: crítica «Sprite» · estudo «Sprite» e «Transform» · 12 ideias. Código conferido em 05/11/2026, sobre `c61f228b05`.

## Como está (conferido)

- `Add ▸ Sprite`: quadrado branco de 1 × 1 m, sem pedir imagem (`crates/ph2d-app-game/src/object_add.rs:158-161`).
- `Add ▸ Image…`: abre Nova Imagem e cria «Canvas», o mesmo componente `Sprite` (`crates/ph2d-app-painter/src/paint_mode.rs:113-145`).
- Painel em Repeated (foto real): 5 cartões abertos ou dobrados; Texture & Atlas com Strategy (3), Storage «Atlas · key 4294967291», Source, Region, Format, Emissive, Centered, Offset, Flip H/V, Reimport.
- Image: ranhura de uma linha com ícone de 20 px; clicar abre o seletor; não existe botão Replace.
- Trocar a imagem mantém a escala: o tamanho no mundo muda sem aviso (`crates/ph2d-app-sprite-anim/src/image_choice.rs:72-80`).
- Pivot: só a ferramenta do canvas, que escreve `Transform.translation` + `Sprite.anchor` compensando (`crates/ph2d-editor-core/src/gizmo/transform.rs:373`).
- Espelhar: os 4 botões vivem juntos no Look, `Flip image` (H, V) e `Mirror object` (H, V); na Sprite o Transform perdeu os dois espelhos, os outros objetos não mudam (decisão 14).
- Nenhuma ponte Sprite → Painter que volte ao asset. Precedente de volta: `shells/desktop/src/hero_intents/image_edit/painter.rs`, `texture_edit.rs:279`.
- Repeated: Width/Height são **cópias da imagem** (`InspectorDraw::Tiled { copies }`), sem unidade no rótulo. A chave `image.width_m` engana: o valor não é metro.

## O que as 9 ensinam

Godot e GameMaker: imagem grande no campo. GDevelop: um só convite no vazio. Unity: ordem de desenho à vista e seletor que troca os campos.
GameMaker e Unreal: pivô de 9 pontos. GDevelop, Construct e GameMaker: editar a imagem a um clique. Evitar: asset separado obrigatório (Unreal), modal (GDevelop).

## Decisões do dono

**Vigentes:** imagem é asset por referência (26/10) · seletor Use abaixo de Image (26/10) · Strategy fica no objeto (26/10) · largar na tela sempre cria objeto (26/10) · Still mostra a imagem inteira (27/10) · todo objeto tem Z-index (01/11).

**Desta revisão (05/11):**

1. **Versão B da crítica, sem nada da C.** Saem as miniaturas no seletor Use e o pivô arrastado na miniatura.
2. Folha em Still: mantém 27/10 + aviso com *Make it Animated* e *Use one Region*; em *Use one Region*, clicar no quadro cria a Region (delegado ao agente).
3. `Add ▸ Sprite` abre a escolha da imagem; `Image…` passa a «Paint a new image…».
4. ~~**Edit in Pixel completo nesta rodada.**~~ Revogada pelo dono em 06/11 (ver abaixo).
5. **Edit in Painter**, pedido do dono: mesma ida e volta.
   **Aprovado pelo dono em 06/11:** quando um objecto com camadas do Painter recebe píxeis por fora (a pintura de outro objecto que usa a mesma imagem), a imagem nova vence: as camadas dele saem, com o aviso «Layers flattened», e o Undo devolve-as. No objecto em que o próprio Painter pintou, as camadas continuam.
6. 9-Slice segue componente; não vira quinto uso (delegado ao agente).

**Do smoke do dono (05/11, depois das fotos):**

7. O layout da crítica: miniatura, nome, tamanho e botões numa superfície própria dentro do cartão Image.
8. **Sai o botão Replace:** clicar na miniatura é trocar a imagem. Em hover a borda acende e um ícone de troca entra a girar no canto.
9. Sai a linha «Size (px)»; em Repeated o tamanho vai entre parênteses na linha da miniatura: `256 × 128 px (300 × 50)`.
10. A tira do Repeated: cópias coladas na largura do corpo, sem xadrez entre elas.
11. O Transform dobrado da imagem tem alfinete, como todo Transform solto.
12. Flip e Pivot têm bolinha de chave. O Flip grava os dois eixos; a chave do Pivot grava também a posição, ou tocar a animação moveria o desenho.

**Do dono, 05/11, noite:**

13. **O Pivot não separa desenho e colisão:** num objecto com colisão, mudar o pivô deixa o desenho E a colisão parados no mundo. Vale para o cartão e para a ferramenta do canvas (`ph2d_sprite_edit::pivot::o_resto_fica`: o `Collider.offset` compensa, os filhos ficam). A chave do Pivot leva também o deslocamento do collider, de quem o tem. As sondas do Player nascem do centro do collider (`origem_e_centro`); com o pivô no pé, medir da origem desviava 1,18 m. Teste: `com_o_pivo_no_pe_o_boneco_pisa_e_salta_igual`.
14. **Espelhar a Sprite: não dão o mesmo** (`sprite_espelhar.rs`): o Transform leva os filhos e a colisão e o Flip deixa-os; com o pivô de lado, o Transform vira à volta do pivô e o Flip no mesmo sítio. **Dono, 06/11, depois do oráculo: os 4 botões juntos no Look**, linhas `Flip image` (H, V) e `Mirror object` (H, V); na Sprite o Transform perde os dois espelhos, os outros objetos não mudam. Oráculo ([`espelhar_oraculo.jsonl`](ferramentas/espelhar_oraculo.jsonl), 72 casos, Godot e Unity): escala −1 é igual nas duas (a imagem vira à volta do pivô, filho e colisão vão junto); o Flip nunca move filho nem colisão, e com pivô fora do centro o Godot vira no lugar e o Unity à volta do pivô. O nosso Flip = o do Godot, medido e mantido (o Player usa `flip_x`, `personagem.rs:192`).
15. **Censo (dono, 06/11):** 13 campos à vista e 31 no total, aceito (a meta era 10 e 24; o 13.º e o 31.º são o Keep in place, pedido pelo dono). Numa sprite com filhos aparece também `Self Tint`, à vista.
16. **A chave do Pivot leva os filhos diretos** (`ph2d_sprite_edit::pivot_chave`: `filhos_do_pivo`, população do Auto Key via `autokey_amostra.rs`; `clique_leva_os_filhos`, chamado em `shells/presos/src/losango.rs`). Medido na cena `sprite_pivot_chave`: a moeda presa fica em (8.5, 2.8) em t=1, t=2 e t=1,5; antes da cura o filho derivava até 1,5 m.

**Do dono, smoke de 06/11:**

17. **Keep in place no Look** (`ImageUseEdit::MirrorInPlace`, `pivot::espelha_no_lugar`): o Mirror espelha filhos e colisão, a imagem fica onde está e a posição compensa. Preferência de sessão, desligada por omissão.
18. **Grade do Pivot sem o nome** (dono: desnecessário). O botão … abre X e Y; os nomes ficam na dica de cada ponto.
19. **Ctrl+Z mantém selecionado o objeto comum que sobreviveu** (`plain_selection_ids`, dono 06/11). As famílias de `keeps_its_selection` seguem a marca do passo.

Outros acertos da mesma noite: `Reimport at scale` e o nome do ponto do Pivot (desce para debaixo da grade quando o maior dos dez nomes não cabe ao lado) não se cortam no piso real do painel, `DOCK_W_MIN` = 210 px (`nada_do_cartao_da_sprite_e_cortado_no_minimo_do_painel`). A imagem editada herda o nome da de onde veio e toma-lhe o lugar enquanto só ela estiver em uso (`imagem::herda_o_nome`); ⚠️ só nesta sessão, porque o `.ph2dproj` não grava nomes de imagem: ao reabrir, volta o nome do 1.º objecto.

Miniatura quadrada de 72 px com recorte pelo raio, moldura de altura fixa e um só botão. **Dono, 06/11:** em qualquer largura a miniatura mantém o tamanho, e o nome, o tamanho e o botão ficam ao lado dela; o tamanho nunca sobe ao título. O `1024 × 1024 px (300 × 50)` do Repeated vem inteiro onde cabe; no estreito, sem os parênteses. Nada é cortado.

**Dono 06/11: dois objetos; cada um só com o botão da sua ferramenta; revoga a 4.** Pixel e Painter são ferramentas de artes diferentes: ficam Image (Painter) e Pixel Sprite, com um ícone cada (a tela e o pixel). O cartão da imagem tem só `[enviar] Painter`. Nenhum modo novo: cada objecto mantém o seu Draw, e o `Tab` não muda.

**Dono 06/11: o Pixel Sprite tem cartão igual ao da imagem.** *«O obj pixel tem apenas o botão de editar no pixel»*, e perguntado: *«Criar um cartão com o botão Pixel: fica igual à imagem»*. Selecionado, ele mostra o cartão `Image` com a mesma moldura: miniatura de 72 px (o quadro corrente, achatado nas camadas visíveis, sobre o mesmo xadrez, nítida), o nome do objecto, `W × H px` e um só botão `[enviar] Pixel`, que entra no Draw daquele objecto. A miniatura dele não troca nada (nem clique, nem ícone em hover), e não há Use nem Look, que ele não tem. O Transform fica como estava. Tem alfinete: preso, mostra o seu objecto, e o botão selecciona-o e entra no Draw dele. A família publica o facto (`ph2d_app_pixel::cartao`, no store, para a seleção e para cada preso); a miniatura só se refaz quando o desenho muda, e em Draw espera até sair. No documento gigante guarda até 512 px de lado. Cena `PH2D_PIXEL_SMOKE=21`. Testes: `o_cartao_do_pixel_sprite` (Inspector), `cartao` (`shells/pixel-scenes`), `o_cartao_do_pixel_sprite_preso_e_o_dele_e_pede_o_draw_dele`.

**Dono 06/11: a imagem abre no Painter sem a repetição.** No Draw, o objecto mostra UMA cópia da imagem inteira, no centro do bloco; o traço cai onde o cursor está. Ao sair, volta o Repeated com as mesmas cópias e os píxeis novos em todas. É só uma vista (`ph2d_ecs::ImagemAberta`, publicada por `ph2d_sprite_screen::imagem_aberta`): nada do objecto muda, o save grava o Repeated e a outra plataforma continua repetida. A Region abre igual (a imagem inteira à escala da área; antes ficava esmagada no quad da área), e o Animated já abria a folha inteira. **Vista do Painter (dono):** o rectângulo verde fica da largura do objecto · a cópia abre centrada · a Region salta.

**Dono 06/11: as camadas saem da Hierarchy.** *«vamos tirar os layers da hierarchy. Eles vivem apenas no painter/pixel»*: a imagem deixa de ter a sub-linha `Layers`; as camadas ficam nos painéis do Painter e do Pixel. As outras famílias mantêm o conteúdo delas.

## O desenho aprovado

Por estado do painel: **vazio** = cartão Image inteiro é «Drop an image or click to choose» + recentes + «Paint a new image…» · **em uso** = cartões abaixo · **Repeated** = os mesmos, mudam só os campos do uso.

| Cartão | Conteúdo |
|---|---|
| Image | superfície com a miniatura (clicar troca), nome, tamanho em px, Edit in Painter; Use; campos do uso |
| Look | Tint, Opacity, Flip em ícones, Pivot de 9 pontos (+ Custom X/Y); Self Tint só com filhos |
| Transform (dobrado) | resumo `850, 200 · 0° · 1 × 1` |
| Ordering (dobrado) | Z Index e Y-Sort; as outras 6 por «+ Order option…», e ficam à vista se em uso |
| Advanced (dobrado) | Strategy, Format, Emissive (glow), Silhouette, Tint each corner, Reimport; resumo `Atlas · RGBA8` |

**Sai:** Storage com número · caixa Region · botão Hand-packed (vira texto no resumo se for o caso) · Show Sheet · nota «shader runtime pending».

**Do agente, além dos estudos:** Pivot de 9 pontos conta sobre o que está **visível** (a caixa opaca do desenho), não a borda da imagem · trocar a imagem mostra `256 × 128 → 64 × 64 · Keep old size`.

## Aceitação

- **Poder:** pivô no painel, em 9 pontos sobre o desenho visível e Custom · editar a imagem no Painter a partir do cartão, voltando sozinha para **todo objeto que usa aquele asset**, com desfazer · um quadro da folha a um clique.
- **Facilidade:** em uso, 13 campos à vista e 31 no total (meta original 10 e 24; número real aceito pelo dono em 06/11, decisão 15; `o_censo_dos_campos_da_sprite_em_uso`). Até ver a imagem pelo Add: 3 gestos (`so_a_sprite_nasce_a_pedir_a_imagem`). Virar e tingir com cartões fechados: 3 gestos (`virar_e_tingir_com_os_cartoes_fechados_sao_tres_gestos`).
- **Intuitividade:** nenhum número interno na tela · o salto de tamanho da folha e do Replace são explicados na hora · unidade no rótulo de todo campo.

## ⛔ Recusas

| O quê | Por quê |
|---|---|
| Miniaturas no seletor Use, pivô na miniatura, cores da imagem | versão C, recusada pelo dono em 05/11 |
| «Frame · click to pick» em Still | contraria 27/10; o gesto vive em *Use one Region* |
| 9-Slice como uso «Panel» | quatro usos do dono; rever no item 9-Slice |
| Colisão a partir da imagem neste item | pertence a Physics Body |
| Botão Replace | supérfluo: a miniatura é o alvo (dono, 05/11) |
| Flip no estilo Unity (vira à volta do pivô) | oráculo corrido (06/11, 72 casos): o nosso Flip = o do Godot, vira no lugar; quem quer à volta do pivô usa `Mirror object` |
| Tira de prévia do Repeated | recusada pelo dono em 05/11; a miniatura basta |
| Edit in Pixel num objecto imagem | dono 06/11: Pixel e Painter servem artes diferentes; a ida e volta saiu do código (`no_pixel`, `ph2d_app_pixel::imagem`) |
| Um só objecto com modos Draw in Pixel e Draw in Painter | dono 06/11: as ferramentas servem artes diferentes; ficam dois objectos, cada um com o seu Draw |

## Provas

Pivot: `a_caixa_opaca_ignora_a_sobra_transparente`, `mudar_o_pivo_nao_move_o_desenho`, `mudar_o_pivo_nao_move_os_filhos`, `mudar_o_pivo_nao_move_a_colisao`, `arrastar_o_pivo_no_canvas_nao_move_a_colisao_nem_os_filhos` (`ph2d-sprite-edit`, `pivot_tests.rs`) · `cada_um_dos_nove_pontos_pede_o_seu_pivo`, `o_flip_e_o_pivot_pintam_a_bolinha_de_chave`, `editar_um_eixo_do_pivot_nao_mexe_no_outro` (`ph2d-panel-inspector/tests/it/o_cartao_da_sprite.rs`) · `com_o_pivo_no_pe_o_boneco_pisa_e_salta_igual` (`shells/game-scenes`).

Cartão: `a_miniatura_abre_o_seletor`, `a_miniatura_e_o_tamanho_nao_mudam_com_a_largura`, `nada_do_cartao_da_sprite_e_cortado_no_minimo_do_painel` (mesmo ficheiro) · Pixel Sprite: `o_pixel_sprite_selecionado_mostra_o_cartao_so_com_o_pixel`, `nada_do_cartao_do_pixel_e_cortado_no_minimo_do_painel` (`o_cartao_do_pixel_sprite.rs`).

Onda 3 (06/11): `so_a_sprite_nasce_a_pedir_a_imagem` (E3 confirmado no ecrã) · `o_aviso_de_folha_da_as_duas_saidas` (clique) e `da_folha_parada_use_one_region_abre_o_editor_e_o_quadro_vira_a_area` (editor); `card.insp.sheet_use_region` é só a chave do rótulo · foto `sprite_transform_no_draw` (Auto Key grava Position, Ctrl+Z no Draw desfaz os dois) · `a_predefinicao_sai_do_custom_e_o_lado_para_no_dispositivo` · `o_mirror_do_look_espelha_o_objeto_e_saiu_do_transform` · `com_a_imagem_virada_o_ponto_segue_o_lado_que_se_ve` (a grade do Pivot segue o lado que se vê; em objeto com Mirror continua no referencial do objeto) · `enquanto_assenta_a_cola_leva_o_cartao_ao_lugar_deste_quadro` · os 4 de `pivot_chave_tests.rs` + cena `sprite_pivot_chave`. Cena nova `PH2D_GAME_SMOKE_FOTO=sprite_moeda` (prende a Coin 4 à Platform High). Ver [onda 3](../handoffs/HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_ONDA3.md).

**Sem prova ainda:** o smoke do dono desta onda; a Sprite não está fechada.

Imagem sem a repetição no Painter: `o_painter_abre_o_repetido_numa_copia_e_ao_sair_volta_o_mesmo` (`ph2d-app-painter`), `o_traco_cai_onde_o_cursor_esta_na_copia` e `a_region_abre_a_imagem_inteira_a_mesma_escala` (`ph2d-sprite-screen`) · censo `every_painter_map_reads_the_sprite_as_it_is_shown` · fotos `sprite_no_painter`, `sprite_no_painter_traco`, `sprite_painter_volta`.

Camadas fora da Hierarchy: `a_imagem_com_camadas_nao_tem_linhas_na_hierarchy` (Painter), `o_pixel_sprite_nao_tem_linhas_de_camadas` (Pixel), `o_flip_mantem_as_camadas_na_hierarchy` (controlo) · foto `sprite_uso`.

## Etapas

| # | O quê | Estado |
|---|---|---|
| E1 | Cartões: Image, Look, Advanced, Ordering, Transform dobrado, cortes | feito |
| E2 | Pivot de 9 pontos sobre o desenho visível | feito (`a_caixa_opaca_ignora_a_sobra_transparente`) |
| E3 | Vazio, `Add ▸ Sprite` que pede imagem, «Paint a new image…» | feito: Add ▸ Sprite nasce quadrado branco selecionado, abre Assets com «Choose the image for "Sprite"», convite no cartão e «Paint a new image…» (3 gestos; `so_a_sprite_nasce_a_pedir_a_imagem`) |
| E4 | Aviso de folha, *Use one Region*, Replace com recibo | feito, com prova (Replace saiu: decisão 8) |
| E5 | Edit in Painter: ida, volta, todos os objetos, desfazer | feito só no Painter; Pixel revogado (06/11) |

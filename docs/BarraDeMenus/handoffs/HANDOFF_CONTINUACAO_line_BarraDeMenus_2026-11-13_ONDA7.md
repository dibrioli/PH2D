# HANDOFF de continuação — line/BarraDeMenus, onda 7 (13/11): polimento medido da faixa, da fila e da busca

Antes: [onda 6](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-13_ONDA6.md). Pedido do dono no smoke da onda 6: *«não é
bela o suficiente, não bem alinhada, barras um pouco mais largas, ícones melhores e mais nítidos; estado da arte»*.

## Estado da arte, MEDIDO (não lido)
VS Code 1.145, Godot 4.7 e Blender 5.2 corridos numa Xvfb própria (`ferramentas/oraculos_onda7.sh`) e medidos com a
régua de pixels `ferramentas/regua_faixa.py` (bandas por cor de fundo; por item: caixa, desvio do centro `dc`, pico de
contraste). Tabela e fotos: [`fotos/oraculos/LEIA-ME.md`](../fotos/oraculos/LEIA-ME.md).

## Causas encontradas e curas (um bloco)
1. **Ícone lavado:** `paint_icon` escala o traço com a viewbox e recua 10 %: a 13 px o glifo media 10 px com traço de
   0,65 px (pico 0,16–0,40 contra 0,50 do texto). Cura: `paint_icon_nitido` (`paint_icons.rs`) — viewbox enche a caixa,
   traço `chrome.bar-icon-stroke` = 1,5 px de ECRÃ, origem no MEIO do pixel (`ui_scale::meio_pixel`; a 16 px as linhas
   da grade Lucide caem no centro de um pixel — o «−» foi de 0,28 a 0,50). Transporte (Play/Pause/Stop) cheio.
   Usado na faixa, nas abas de documento, na fila fina, nas abas das colunas e na lupa da busca. Portão
   `o_glifo_das_barras_tem_o_traco_do_token_em_qualquer_lado`.
2. **Quatro alturas na faixa** (campo/Menu/Play 22, aba 24, pílula do jogo 28, separador 22): `menu_faixa::caixa` =
   `chrome.menu-bar-ctl-h` (26) para todo controlo; `menu_faixa::segmento` (caixa − `SEG_PAD`, 22) para o que vive dentro
   de uma pílula (aba escolhida, Play, Pause, Stop, moldura do «Controls»); `Barra::pilula_em`/`segmento_em` pintam a
   caixa exacta. Separador = 16 px ao meio. Portão `todo_controlo_da_faixa_tem_uma_altura_e_um_centro`.
3. **Texto ~1 px abaixo do meio** (`y + (h − font)/2`) na marca, na plaquinha, no convite do campo e em 6 textos de
   modais vizinhos (fill, onion, «X» do cabeçalho da paleta): passam à porta da casa `folga_do_texto`.
4. **Abas das colunas** («Hierarchy»/«Inspector») 2 px acima: `TAB_BAR_H` = `TOOL_ROW_H_PX` (a banda é a da fila), o
   segmento com o lado do chip (24), a primeira aba na borda da marca (x = `Md`).
5. **Busca tapava meio campo:** `scroll::list_top` era `ROW_H_PX` (22, de quando a faixa media uma linha) → faixa + `Xs`.
   Linhas e campo de 32 (`lista::item_h`, `look` da lista), respiro `Md`.
6. **Alturas** (tokens): faixa 30 → 36, fila 26 → 32, glifo 13 → 16, chip 24; vãos glifo→glifo 12 (VS Code), entre grupos 20.

## Medido depois (foto `fotos/onda7_v2_topo.png`, 1930×1040)
Faixa: `dc` 0…+1,0 em todos os 14 itens (antes ±0,5 a 30 px, mas com caixas de alturas diferentes); fila: 0…+1,5 (o
+1,5 é o descendente de «Object Mode»; antes −2…+1,5 com as abas). Pico do ícone = 0,502 = texto em todos, salvo o
«Controls» desligado de propósito na cena (`TextDisabled`, 0,157).

## Portões mexidos
- `the_chrome_never_eats_more_of_a_tablet_than_this`: piso 51,5·50,8·50,3 → **50,9·50,0·49,4** (medido 51,0·50,1·49,5;
  antes da onda 6 era 50,5·49,3·48,6). Descida assinada pelo pedido do dono.
- `na_tela_cheia_a_moldura_e_o_separador_sao_os_da_barra_do_editor`: tolerância 1e-6 px (o contorno arredondado em Bézier
  dá 2e-15; erro de geometria é de pixel).
- `fila_fina_tests` segue os tokens novos sem mudança. Verdes por nome: os três de contraste, `nenhum_rotulo…`,
  `menu_faixa`, `transport_bar`, `window_buttons`, `slot_tabs`, `tool_bar`, `command_palette`, `design_token_sync`.

## Fotos (antes do smoke)
`onda7_lado_a_lado_inteiro.png` (maquete C · antes · depois · VS Code) · `onda7_ampliado_3x_{esquerda,direita}.png` ·
`onda7_busca_antes_depois.png` · `onda7_v2_x4_*`. Script: `ferramentas/fotos_onda7.sh <prefixo>`.

## Aberto
- Pause/Stop seguem à vista em repouso (a maquete C só os mostra a jogar) — decisão de produto, não tomada.
- `PH2D_CARDS=0` fora das fotos desta onda.
- Clippy/fmt das crates tocadas e mutação dos dois portões novos: no fecho da linha.

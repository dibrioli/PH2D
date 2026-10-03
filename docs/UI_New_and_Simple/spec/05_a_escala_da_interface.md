# 05 — A escala da interface inteira

> **Decisão do dono, 2026-10-02:** *«Sim, fazer agora»* — uma opção em *Edit ▸ Preferences…* que
> aumenta a interface INTEIRA (linhas, botões, ícones, texto, painéis), além do tamanho de texto
> (`UiTextSize`, que pára no `Large` porque as linhas têm `ROW_H_PX = 22`).
> Leitor: a janela que implementa esta onda. Estado da linha: o handoff de continuação em
> [`../handoffs/`](../handoffs/README.md).

## 1. O que os oráculos dizem (CORRIDOS em 02/10, não lidos)

| oráculo | controlo | faixa | como se corre |
|---|---|---|---|
| Blender 5.x | *Resolution Scale* (`PreferencesView.ui_scale`), float | `0,5 – 6,0` (deslizante `0,5 – 3,0`), fábrica `1,0` | `blender -b --factory-startup --python-expr "…bl_rna.properties['ui_scale']…"` |
| Godot 4.7.2 | `interface/editor/display_scale`, enumerado | `Auto (%d%%) · 75 · 100 · 125 · 150 · 175 · 200 · Custom` | tabela de opções no binário (`strings $(which godot)`) |

⇒ **Degraus como o Godot** (o artista escolhe, não arrasta): `100 · 125 · 150 · 175 · 200 %`.

**HiDPI (corrido em 02/10, [`oraculos/hidpi_2026-10-02.md`](oraculos/hidpi_2026-10-02.md)):** o
Blender faz `efectivo = sistema × utilizador` e a arte fica física (1 px da imagem = 1 px do ecrã a
1:1); o `Auto` da Godot é o factor do sistema, e um valor manual SUBSTITUI-O. ⇒ PH2D: `s =
scale_factor × UiScale` (o nosso `100 %` é o `Auto` da Godot; os degraus são relativos ao ecrã,
como no Blender) e a arte continua física, como no Blender.
⚠️ O tecto não é escolhido: **a interface tem de caber na janela**. Medido em 02/10 (§3b): a
`200 %` numa janela de `1366` px (`683` lógicos) o layout ainda degrada com graça, e por isso
nenhum degrau é desactivado.

## 2. O mapa medido (02/10, conferido no código)

- **Hoje tudo é pixel FÍSICO e não há factor nenhum:**
  - o viewport do quadro é `surface.size()` cru: `shells/desktop/src/render_loop/fase_frame_open.rs:128`;
  - o ponteiro entra como `PhysicalPosition` e é copiado sem conversão: `shells/desktop/src/input_dispatch.rs:145-150`;
  - o `scale_factor()` do winit é lido (`winit_host.rs:33`) e **não usado**.
- **Passes de GPU separados, com rects em pixel físico** (os que a escala tem de converter):
  - o chrome em vello (`present.rs:336`, `render_with_streams`);
  - as bandas e os efeitos (`present_bands.rs:114`, `present_fx.rs:53/121`, `present_frost.rs:125`);
  - a cena do jogo, tonemap e composição (`present_chrome.rs:189`);
  - os viewports 3D (`fase_field3d_smoke_draw.rs:41-44`, `ph2d_viewport3d::layout::area`);
  - o *split* do Motion (`present.rs:81`, `scene_viewport`).
- **Preferências:** `~/.ph2d/prefs.txt` → `HeroScreen.text_style` → `ph2d_text::set_active_text_style`
  (`screens/hero/paint.rs:152`). Linhas do menu: `screens/hero/text_style_rows.rs`. Despacho:
  `chrome/settings_font.rs`. A escala é um quarto eixo IRMÃO destes três.

## 3. O desenho: pixel LÓGICO + uma porta de escala

⭐ **A interface passa a viver em pixels LÓGICOS.** Um factor `s` (a preferência; mais tarde
`× scale_factor` do winit, o HiDPI de graça) converte nas DUAS fronteiras, e só nelas:

1. **Saída.** O hero recebe `viewport = físico / s`. A cena do chrome é desenhada sob
   `Affine::scale(s)` (um `append` com transform, no sítio onde a cena é entregue ao renderer). Cada
   pass que recebe um rect do layout (bandas, 3D, Motion, jogo) converte-o por **uma** função
   (`lógico → físico`), e nunca à mão.
2. **Entrada.** O ponteiro (e os deltas da roda em pixel) divide-se por `s` **no** `on_cursor_moved`
   (e na roda), antes de qualquer consumidor. Nenhum consumidor converte.

⛔ **Recusado: multiplicar cada medida por `s`** (o `EDSCALE` do Godot). São `~1 864` literais
`LITERAL-PX-OK` em `~450` ficheiros: seria uma lei por sítio, e o primeiro esquecido é um controlo
que não escala.

⚠️ **As três perguntas a MEDIR antes de fechar a onda** (não adivinhar):
- **Nitidez do texto.** O *hint* e o `snap_x` alinham à grelha do layout; sob `Affine::scale(1,25)`
  essa grelha deixa de ser a do ecrã. Fotografar `100/125/150` e decidir pela foto: desligar o hint
  fora de `s` inteiro, ou levar `s` ao `TextSystem`.
- **A arte.** Sob a mesma transformação a arte do canvas também cresce `s×`. Medir o que o
  *zoom 100 %* passa a significar e decidir com o dono se a arte acompanha (o Blender acompanha a
  escala nas vistas 2D) ou se o canvas desfaz `s`.
- **Os passes físicos.** Cada um dos rects do §2 com um gate: o rect pintado a `s = 1,5` é
  `1,5×` o lógico.

## 3b. O que se mediu ao construir (02/10)

- ⛔ **Recusa MEDIDA: escalar os TOKENS** (o `EDSCALE` da Godot). Há `1 647` literais
  `LITERAL-PX-OK` (269 no `ph2d-editor-core`, 1 378 nos painéis) e `~80` constantes de compilação
  (`ROW_H_PX` em 73, `MENU_BAR_H`, `TAB_BAR_H`, `DOCK_W_*`, `MIN_W_PX`). Uma escala nos tokens
  deixaria essas medidas fixas: as linhas cresceriam e as colunas não.
- **A arte NÃO acompanha a escala.** Os passes do mundo (sprites, câmera, 3D) ficam físicos.
  A grade, as réguas e os gizmos que o chrome pinta são projectados com a janela LÓGICA. A
  projecção é linear na janela, por isso caem no píxel físico da arte. As alças crescem com a
  interface, e as réguas marcam o mesmo número de unidades por píxel a 100 % e a 200 %.
- **O chrome pinta-se em espaço lógico; o resto da cena fica físico.** Várias fases da shell
  pintam na mesma cena em coordenadas do mundo: a selecção, a física, as guias do vetor e o anel
  do pincel. Por isso a escala não se aplica à cena inteira. O chrome pinta numa cena lógica, que
  é colada sob `Affine::scale(s)` no seu ponto da ordem de pintura (`ui_scale::pintar_no_chrome`).
  Os toasts e a forma de onda do áudio são chrome e entram na mesma porta.
- **Nenhum degrau é desactivado.** Medido a 200 % numa janela de 1366×768 (683 lógicos): as
  colunas encolhem, a fila manda o resto para o `⋯` e o canvas fica estreito, mas usável.
  ⚠️ **Defeito anterior que a medição expôs:** a barra de estatísticas estreita quebra itens em
  duas linhas (`0` / `ent`) e encosta outros (`EDIT60 fps`). Acontece em qualquer janela dessa
  largura lógica, mesmo a 100 %.

## 4. Gates (red-first)

- **Fábrica ao bit:** a `s = 1` a cena e o índice de hit são IDÊNTICOS aos de hoje (nenhum
  píxel se move no caminho de omissão).
- **O clique cai onde se vê:** a `s ∈ {1,25; 1,5; 2}`, o centro de cada chip registado (em
  lógico), convertido para físico e devolvido pela entrada, acerta o MESMO id.
- **Nenhum pass em pixel cru:** censo — todo rect que vai a um pass passa pela porta `lógico → físico`.
- **A preferência viaja:** `prefs.txt` grava e lê `ui_scale`; valor inválido ⇒ fábrica.
- **Fotos** (`fotografa_cena.sh`, HOME isolado com `ui_scale=150`): barra, Inspector, um 3D.

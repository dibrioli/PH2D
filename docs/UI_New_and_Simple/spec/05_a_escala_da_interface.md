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
⚠️ O tecto não é escolhido: **a interface tem de caber na janela**. A `200 %` uma janela de
`1366` px tem `683` px lógicos e as duas colunas abertas não cabem. ⇒ medir o mínimo lógico do
layout e **desactivar** (não esconder) os degraus que a janela actual não leva, com a razão escrita
na linha do menu.

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

## 4. Gates (red-first)

- **Fábrica ao bit:** a `s = 1` a cena e o índice de hit são IDÊNTICOS aos de hoje (nenhum
  píxel se move no caminho de omissão).
- **O clique cai onde se vê:** a `s ∈ {1,25; 1,5; 2}`, o centro de cada chip registado (em
  lógico), convertido para físico e devolvido pela entrada, acerta o MESMO id.
- **Nenhum pass em pixel cru:** censo — todo rect que vai a um pass passa pela porta `lógico → físico`.
- **A preferência viaja:** `prefs.txt` grava e lê `ui_scale`; valor inválido ⇒ fábrica.
- **Fotos** (`fotografa_cena.sh`, HOME isolado com `ui_scale=150`): barra, Inspector, um 3D.

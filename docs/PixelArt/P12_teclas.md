# P12a — As teclas do Pixel como dados (08/11)

A tabela vive em `crates/ph2d-pixel-keys` (`FACTORY`, `RESERVED_GLOBAL`, `SAME_MEANING`, `ASEPRITE`,
`OWNER_EXCEPTIONS`). Fonte: [`P12_oraculos.md`](P12_oraculos.md) (tabela de fábrica + Rodada 2) e as
excepções do dono em [`P12_plano.md`](P12_plano.md) §4.2 e §9. **Ligada** (08/11): `input_teclas::key_with`
faz `PixelApp::keys.resolve(chord, contexto)` e despacha a `PixelAction` (`request()` → `pede`, `tool()` →
`set_tool`); ficam à mão a cascata do `Esc`, o `Enter`/`Esc` a meio, o desfazer e a caixa do Text. O
delta do utilizador lê-se de `~/.ph2d/pixel-keys.txt` no arranque (`with_armed_smoke`). Provas por nome:
`teclas::no_app_wide_key_is_taken_by_the_pixel` e `teclas::every_factory_key_in_normal_does_its_action`
(`shells/pixel-scenes`).

⚠️ Ainda sem porta: `Ctrl+'` Show Grid (a grade do app não tem pedido do Pixel: a tecla passa ao app,
que não a usa) e `Ctrl+Alt+Shift+K` (a janela de atalhos é a P12 seguinte). `F2` pede ao painel
Timeline que abra o campo do nome da camada activa (só com a Timeline à vista). O laço poligonal
aberto conta como gesto a meio (contexto `Stroking`).

## 1. O que muda com a ligação

| Tecla | Antes | Depois | Porquê |
|---|---|---|---|
| `F2` | Set Loop Section | Rename Layer | §9.1: `F2` renomeia no app inteiro; o laço fica sem tecla (botão + paleta) |
| `F9` | Convolution Matrix (no código; nunca chegava: o F9 do chrome legado corre antes do Pixel, `keyboard.rs:115`) | sem tecla no Pixel | é do app (§9.7) |
| `Ctrl+Space` | engolida pelo `input::space` do Pixel (o zen não liga na aba) | devolvida ao app (zen) | §9.2 + §9.7; cura na ligação |
| `Space+H` / `Space+X` | Hand / Swap Colors (o Space era ignorado) | Brush: Flip Horizontally | Aseprite |
| `Space+V` / `Space+Y` | Move/Transform / nada | Brush: Flip Vertically | Aseprite |
| `Space+R` | Blur/Jumble | Brush: Rotate 90° | Aseprite |
| `Space+N` | nada | New Tilemap Layer | Aseprite |
| `Ctrl+R` | nada | Redo (3.ª tecla) | Aseprite |
| `Shift+Del` · `Ctrl+Ins` · `Shift+Ins` | nada | Cut · Copy · Paste (2.ª tecla) | Aseprite |
| `Shift+R` | nada | Replace Color | Aseprite (o pedido já existia) |
| `F10` | nada | Color Curve (2.ª tecla) | Aseprite |
| `Ctrl+Shift+V` | nada | New Layer from Clipboard | Aseprite |
| `Shift+1..4` | nada | Layer Opacity 25/50/75/100 % (64/128/192/255) | Aseprite |
| `Alt+Shift+D` | nada | Duplicate Linked Cels (2.ª tecla) | Aseprite |
| `Ctrl+'` | nada | Show Grid (a grade do app, como o `G`) | §4.2: 2.ª tecla para quem vem do Aseprite |
| `Ctrl+Alt+Shift+K` | nada | Keyboard Shortcuts (a janela é da P12a) | Aseprite |

Tudo o resto fica como hoje (ferramentas, `P`/`Shift+P` balde/gradient, `Shift+C` Slice, `F4`
timeline, `Ctrl+Shift+'` Pixel Grid, Alt+menus, Esc/Enter por contexto, `Space+Tab`/`Space+1..3`).

## 2. Contextos

`Normal` · `Selection` (máscara, intervalo, modo do `Ctrl+B`: `Esc` = Clear Selection, com a cascata
de hoje) · `Transformation` (células levantadas: `Enter` aplica, `Esc` cancela, `Ctrl+Z`/`Ctrl+Shift+Z`
largam) · `Stroking` (gesto a meio, laço poligonal aberto: `Enter` acaba, `Esc` cancela) · `Slices`
(Slice Tool + slices seleccionadas: `Del` apaga, `Esc` larga). Um contexto que não tem a tecla cai
no `Normal`; a gesto a meio só caem as acções que hoje valem a meio (`PixelAction::while_stroking`).

## 3. Do Aseprite e sem tecla (e porquê)

| Aseprite | Porquê |
|---|---|
| `G` / `Shift+G` balde/gradient | `G` é a grade do app (dono 07/11): `P` / `Shift+P` |
| `Shift+P` / `F2` Layer Properties, `P` Frame Properties | o `P` é do balde; `F2` = renomear (§9.1) |
| `F2` Set Loop Section (Frames Selection) | `F2` é renomear |
| `F9` Convolution Matrix | `F9` é do app |
| `Tab` timeline | `Tab` é o modo (dono 06/11): `F4` |
| `F4` Edit Palette Mode / `A` paleta | `F4` é a timeline; sem pedido de painel para a paleta |
| `Ctrl+K` Preferences · `Ctrl+Space` Run Command · `Ctrl+P` Sprite Properties · `Ctrl+0` Fit · `Ctrl+N/O/S`, `Ctrl+Shift+S` · `Ctrl+Tab` · `F5` | teclas do app (§2 abaixo) |
| Zoom `1..6`, `` ` ``, `Ctrl+=`/`Ctrl+-`/`Ctrl+Shift+=` · pincel `-`/`+` (e do teclado numérico) · `[`/`]`/`9`/`0` índice da cor | sem `PixelRequest` simples (zoom é da vista do app; o tamanho do pincel vai nas `Options` inteiras; o índice precisa do estado) — ficam para a ligação |
| setas na selecção, `↑`/`↓` camada, `,`/`.` dentro da tag, `Alt+G`, `Alt+D`, `Shift+E`, `Shift+Space+N`, `Space+D`, `Shift+S`, `Ctrl+H`, `F6`, `F8`, `F11`, `Shift+W`, `Shift+Z`, `Ctrl+Alt+N`, `Ctrl+F`, `Z` (Zoom tool), `Ctrl+W/Q`, `Alt+F/E/…` | o Pixel não tem o comando (ou o pedido precisa de estado) |

## 4. Teclas do app

**Globais** (`RESERVED_GLOBAL`: a janela recusa-as e diz de quem são): `Ctrl+K` paleta
(`input_handlers.rs:137`) · `Ctrl+Space` zen (`:130`) · `Tab`/`Shift+Tab`/`Ctrl+Tab` modo (`:124`) ·
`Ctrl+S`/`Ctrl+Shift+S` (`input_dispatch/keyboard_files.rs:47`) · `Ctrl+O` (`:57`) · `Ctrl+N`
(`handlers_teclas_editor.rs:233`) · `G`/`Shift+G` grade (`:69`) · `F5`/`Shift+F5` jogo
(`posse_do_jogo.rs:46`) · `F9` chrome legado (`keyboard.rs:115`) · `Ctrl+P` prender ao esqueleto
(`handlers_teclas_editor.rs:188`) · `Ctrl+0` (`:40`) · `Ctrl+Enter` (`:226`) · `Shift+A` menu Add (`:240`).

**Mesmo sentido** (`SAME_MEANING`, só em `Normal`): `F2` renomear · `Ctrl+Z` desfazer ·
`Ctrl+Y`/`Ctrl+Shift+Z` refazer · `Ctrl+C`/`X`/`V` · `Ctrl+A` · `Del` — o Pixel encaminha-as ao sprite.

**Sombreadas na aba do Pixel** (letras de contexto do editor de cena; o Pixel tem o teclado antes,
como o Painter e o Vector): `M` tema (`input_handlers.rs:148`) · `B` colliders (`:160`) · `W` física
(`:172`) · `T` tokens (`:194`) · `L` timeline (`handlers_teclas_editor.rs:134`) · `K` keyframe
(`:152`) · `F`/`Home` enquadrar (`:47`) · `Space` tocar (`:86`) · setas (Flip, `:110`).

## 5. O ficheiro do utilizador

`~/.ph2d/pixel-keys.txt`, só o que difere da fábrica (como o delta do Aseprite, O2/O3), uma linha por
acção: `edit.transform = Ctrl+T, Shift+T` · `edit.fill = -` (sem tecla) · `frame.loop_section = Alt+L
@Selection`. Id desconhecido salta-se e reporta-se; tecla do app salta-se e reporta-se; a tecla que a
linha dá sai de quem a tinha (a linha é a palavra do utilizador).

# P12a — As teclas do Pixel como dados (08/11)

A tabela vive em `crates/ph2d-pixel-keys` (`FACTORY`, `RESERVED_GLOBAL`, `SAME_MEANING`, `ASEPRITE`,
`OWNER_EXCEPTIONS`). Fonte: [`P12_oraculos.md`](P12_oraculos.md) (tabela de fábrica + Rodada 2) e as
excepções do dono em [`P12_plano.md`](P12_plano.md) §4.2 e §9. **Ligada** (08/11): `input_teclas::key_with`
faz `PixelApp::keys.resolve(chord, contexto)` e despacha a `PixelAction` (`request()` → `pede`, `tool()` →
`set_tool`); ficam à mão a cascata do `Esc`, o `Enter`/`Esc` a meio, o desfazer e a caixa do Text. O
delta do utilizador lê-se de `~/.ph2d/pixel-keys.txt` no arranque (`with_armed_smoke`). Provas por nome:
`teclas::no_app_wide_key_is_taken_by_the_pixel` e `teclas::every_factory_key_in_normal_does_its_action`
(`shells/pixel-scenes`).

**P12a, fecho das teclas (08/11).** `Ctrl+'` Show Grid liga a grade de píxeis do canvas (`app.grid`, a
mesma do `Ctrl+Shift+'`). `F2` com a Timeline FECHADA abre-a e o campo do nome da camada activa abre
nela (`PixelApp::mostra_timeline` → `panel.rs`): o artista vê o campo onde sempre o viu. Novas, todas em
`Normal` (prova por nome, porta real `compose::keyboard`: `teclas::as_teclas_novas_*`,
`teclas::f2_opens_the_timeline_to_rename_the_layer`):

| Tecla | Acção | Nota |
|---|---|---|
| `1`..`6` | Zoom 100…3200 % | o degrau da escada inteira (`ph2d_pixel_render::zoom`) à volta do meio da vista |
| `` ` `` | Zoom 50 % | ⚠️ cai no 100 %: o Draw é zoom INTEIRO (uma célula = N píxeis de ecrã, `LEVELS[0] = 1`) |
| `Ctrl+=` · `Ctrl+Shift+=` / `Ctrl+-` | Zoom In / Out | um degrau |
| `-` · `NumpadMinus` / `Shift+=` (`+`) · `NumpadPlus` | Brush Smaller / Bigger | ±1, `1..=PIXEL_MAX_BRUSH` |
| `[` · `9` / `]` · `0` | FG: cor anterior / seguinte da paleta | dá a volta; parte da última clicada ou da posição do FG |
| `↑` / `↓` | Camada de cima / de baixo | pára nas pontas |
| `,` / `.` | Quadro anterior / seguinte DENTRO da tag | a tag mais interior do quadro (`play::tag_at`); sem tag, a linha inteira; dá a volta |

O zoom é um `KeyAnswer::Zoom` que a porta (`compose`) aplica, porque a câmera é dela.

⚠️ **Não feito:** `Ctrl+Alt+Shift+K` SEM sprite em Draw. Não há «aba do Pixel» no roteamento: o Pixel é
um modo do objecto, e a `View` só sabe o objecto em Draw; abrir a janela fora do Draw tomaria a tecla
em toda a cena. Fora do Draw abre-se pelo menu Edit ▸ Keyboard Shortcuts.

**Import/Export** da janela: o diálogo de ficheiro do app (`ph2d_app_host::modal::save_file`/`pick_file`,
a porta cronometrada, a mesma do Save Palette). A janela pede (`canal::Troca`), a família abre o
diálogo em `compose::frame_of` (`input_teclas::troca_de_teclas`) e devolve o caminho
(`entrega_troca`); o Import SOMA (Aseprite R6, `Keymap::import_text`). O ficheiro e a escrita atómica
vivem em `ph2d_pixel_keys::user_file` (era o `shim.rs` da janela).

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
| setas na selecção (mover o conteúdo), `Alt+G`, `Alt+D`, `Shift+E`, `Shift+Space+N`, `Space+D`, `Shift+S`, `Ctrl+H`, `F6`, `F8`, `F11`, `Shift+W`, `Shift+Z`, `Ctrl+Alt+N`, `Ctrl+F`, `Z` (Zoom tool), `Ctrl+W/Q`, `Alt+F/E/…` | o Pixel não tem o comando (ou o pedido precisa de estado) |

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

**A lei é um CENSO** (U3, `crates/ph2d-pixel-keys/tests/the_app_keys_census.rs`): as teclas do app
lêem-se dos braços `KeyCode::…` da cadeia da shell que corre na aba do Pixel (`keyboard.rs`,
`posse_do_jogo.rs`, `keyboard_files.rs`, `keyboard_timeline.rs`, `keyboard_hierarchy.rs`,
`input_handlers.rs`, `handlers_teclas_editor.rs`), com os modificadores que o braço e os `if` à
volta exigem e as `LETRAS_SOLTAS` de `letra_solta.rs`. Toda tecla de fábrica que casa uma delas
está em `RESERVED_GLOBAL`, `SAME_MEANING`, `SHADOWED` ou `OWNER_EXCEPTIONS`; toda linha de
`RESERVED_GLOBAL`/`SHADOWED` tem de ainda casar uma tecla do app. Fora do censo: `keyboard_cadeia.rs`
(vetor e Motion, só com a tool deles viva) e o `keymap.rs` do grafo (só com o cursor no grafo).

**`SHADOWED`** (a tabela, com onde e porquê) junta às letras de cena acima o que o censo achou sem
lista: `Ctrl+D`/`Ctrl+Shift+D` (duplicar a linha da Hierarquia, `keyboard_hierarchy.rs:141`, só com
o cursor sobre ela; duplicar chaves do dope-sheet, `keyboard_timeline.rs:136`) — no canvas do Pixel
é Deselect/Reselect, como no Aseprite · `,`/`.` (`handlers_teclas_editor.rs:91`): o MESMO verbo
(um quadro), no relógio do sprite, dentro da tag · `Ctrl+R`/`Ctrl+E` (inverter/distribuir chaves,
`keyboard_timeline.rs:140`/`:145`) · `Backspace`/`Shift+Del` (apagar linha/chaves, `:140`/`:78`) ·
`Ctrl+Shift+C/X/V` (o editor de áudio e o dope-sheet ignoram o Shift, `handlers_teclas_editor.rs:172`,
`keyboard_timeline.rs:129`) · `Ctrl+Alt+Shift+K` (o braço do `Ctrl+K` ignora Shift/Alt; o `Ctrl+K`
continua reservado) · `Space+Tab` (o braço do `Tab` ignora o Espaço preso; o `Tab` continua
reservado). Nenhuma é acção GLOBAL do app: todas são de painel, de selecção ou letra de cena, por
isso nenhuma tecla do Pixel mudou.

⭐ **O painel sob o ponteiro fica com as teclas dele** (dono: «nenhuma tecla do Pixel rouba uma
tecla do app»; F2 do fecho P12). O Pixel tem o teclado antes da cadeia da shell, mas
`pixel_owns_the_keyboard` (`input_dispatch/despacho_pixel.rs`) cede primeiro as teclas que o painel
sob o ponteiro reclama — `ph2d_pixel_keys::PanelUnderPointer::keeps`, com as mesmas perguntas de
área da shell (`cursor_over_hierarchy`, `cursor_over_timeline`): sobre a **Hierarquia**
`Delete`/`Backspace` e `Ctrl+D` (com ou sem Shift); sobre o **dope-sheet** da timeline da cena
`Delete`/`Backspace`, `M` e `Ctrl+C/X/V/D/R/E`. Fora deles (o canvas, a timeline do sprite) as
teclas continuam do Pixel: `Ctrl+D` é o Deselect. Prova:
`every_shadowed_panel_chord_goes_to_the_panel_under_the_pointer` (toda linha de `SHADOWED` cujo dono
é a Hierarquia ou o dope-sheet cede ao painel e só a ele).

## 5. O ficheiro do utilizador

`~/.ph2d/pixel-keys.txt`, só o que difere da fábrica (como o delta do Aseprite, O2/O3), uma linha por
acção: `edit.transform = Ctrl+T, Shift+T` · `edit.fill = -` (sem tecla) · `frame.loop_section = Alt+L
@Selection`. Id desconhecido salta-se e reporta-se; tecla do app salta-se e reporta-se; a tecla que a
linha dá sai de quem a tinha (a linha é a palavra do utilizador).

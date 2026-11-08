# P12 — Ergonomia no Aseprite Trial (só medido)

08/11. Aseprite Trial 1.3.18.6, Xvfb `:77` 1600×1000, HOME **vazio** (nada copiado: o que se vê é o de fábrica; `screen_scale 2` já é o de fábrica), ponteiro e teclado reais por `xdotool` (teclado só depois de `windowfocus --sync`), fotos por `import -window root`. Caixa preta: observa-se, nada se lê do fonte. Salvo dito, cada caso foi visto **1 vez**. Fotos em [`aseprite/`](aseprite/) com prefixo `P12_`.

**Lições do instrumento.** `Ctrl+Space` **abriu** o Run Command desta vez (na P11 não abria: lá o foco não estava garantido). No diálogo de captura de tecla, `Return` **é capturado como a tecla** (vira «Enter / Play Animation»): confirmar sempre com clique no `OK`. Em zsh, `set -- $p` não parte palavras: o laço de fotos corre em `bash -c`.

## O1–O8. Atalhos (Edit ▸ Keyboard Shortcuts, `Ctrl+Alt+Shift+K`)

| ID | Pergunta | Visto |
|---|---|---|
| O1 | Diálogo | Busca no topo · lista de secções `Menus` · `Commands` · `Tools` · `Action Modifiers` · `Mouse Wheel` · `Drag Value` · botões `Import` `Export` `Reset` · colunas `Action` · `Key` · `Context` ([foto](aseprite/P12_a_atalhos_dialogo.png)). `Menus` é a árvore dos menus (File ▸ New… `Ctrl+N` …) |
| O2 | `Export` no Trial | **Grava** (o bloqueio do Trial é só de sprites). Mas grava **só o DELTA do utilizador**: de fábrica sai um XML vazio, 161 bytes ([`P12_default.aseprite-keys`](aseprite/P12_default.aseprite-keys)). ⇒ a lista de fábrica NÃO sai por aqui; a tabela abaixo veio da tabela de teclas instalada (ver ⛔) e foi conferida por amostra no diálogo (File, Grid, Undo, Mouse Wheel: iguais) |
| O3 | Formato do delta | `<keyboard version="1">` com `<commands>` `<tools>` `<quicktools>` `<actions>` `<wheel>` `<drag>`; cada tecla `<key command="…" shortcut="…"/>`; tecla **tirada** a outro = `removed="true"` ([`P12_delta_conflito.aseprite-keys`](aseprite/P12_delta_conflito.aseprite-keys)) |
| O4 | Busca | Substring sobre o nome **em todas as secções ao mesmo tempo**, agrupada por secção (`grid` → 5 em Menus, 6 em Commands, 1 em Action Modifiers: `Translating Selection: Snap To Grid` `Alt`) ([foto](aseprite/P12_a_busca_e_tecla_ocupada.png)) |
| O5 | Editar | Pairar na linha mostra `Add` + um botão por tecla + `×` por tecla. `Add` abre `Keyboard Shortcut`: caixa `Key:` que captura · `Clear` · caixas `Ctrl` `Alt` `Shift` `Space` `Super` · **`Assigned to:`** · `OK` `Cancel` |
| O6 | Tecla já usada | `Shift+S` (de `Snap to Grid`) em `Grid Settings`: a caixa diz **`Assigned to: Snap to Grid`** antes do OK — é o único aviso. `OK` **rouba sem perguntar**: Snap to Grid fica sem tecla (delta `removed="true"`) ([foto](aseprite/P12_a_busca_e_tecla_ocupada.png)). A linha do menu e a do comando mudam juntas (são o mesmo comando) |
| O7 | Várias teclas | Sim: `Show Grid` ficou `Ctrl+'` **e** `Ctrl+G`, uma por linha ([foto](aseprite/P12_a_duas_teclas_e_reset.png)). Teclas com contexto: a mesma tecla em contextos diferentes (`Normal`, `Selection`, `Transformation`, `FramesSelection`) não conflitua (`F2` = Layer Properties em Normal e Set Loop Section em FramesSelection) |
| O8 | Repor | `Reset` repõe **tudo**, com aviso *«Do you want to restore all keyboard shortcuts to their original default settings?»* `Yes`/`No`. Repor UMA: não há botão; só `×` na tecla (tira) |
| O9 | Roda e arrasto | `Mouse Wheel`: ☑ `Zoom with scroll wheel`, ☐ `Invert the mouse wheel direction to increase the brush size`, abas `Default`/`Custom`; 26 acções (Brush Angle/Size, Cel/Ink/Layer Opacity, Frame, Ink Type, Layer, Tool, cor HSL/HSV/alpha, Scroll, Tile, Zoom). De fábrica: `Ctrl` = tamanho do pincel · `Ctrl+Shift` = quadro · `Alt` = entrada da paleta FG · `Alt+Shift` = BG · `Shift` = scroll horizontal · sem modificador = Zoom ([foto](aseprite/P12_a_roda_e_arrasto.png)). `Drag Value`: as mesmas acções, de fábrica só `Ctrl+Alt`+arrasto = tamanho do pincel, com `Angle` (8 setas) e `Distance` 1 ([foto](aseprite/P12_a_drag_value.png)) |

### Tabela de fábrica (Linux)

**Ferramentas** (a mesma tecla em 2 = alterna dentro do grupo): `M` Rect Marquee · `Shift+M` Ellipse Marquee · `Q` Lasso · `Shift+Q` Polygonal Lasso · `W` Magic Wand · `B` Pencil · `Shift+B` Spray · `E` Eraser · `I` Eyedropper · `H` Hand · `V` Move · `Shift+C` Slice · `Z` Zoom · `G` Paint Bucket · `Shift+G` Gradient · `L` Line · `Shift+L` Curve · `U` Rectangle/Filled · `Shift+U` Ellipse/Filled · `D` Contour · `Shift+D` Polygon · `R` Blur/Jumble · `T` Text.
**Ferramenta momentânea** (segurar): `Alt` Eyedropper · `Ctrl` Move · `Space` Hand.

**Modificadores** (contexto): `Ctrl` copiar selecção · `Alt` snap à grelha · `Shift` travar eixo · `Ctrl` controlo fino (mover/escalar selecção) · `Shift` snap de ângulo · `Shift` manter proporção · `Alt` escalar do centro · `Shift` somar selecção · `Alt+Shift` subtrair · `Ctrl+Shift` intersectar · `Ctrl` auto-seleccionar camada · `Shift` recta desde o último ponto · `Ctrl` snap de ângulo desde o último ponto · `Shift` forma quadrada · `Ctrl` desenhar do centro · `C` raio do canto · `Space` mover origem · `Alt` rodar forma.

| Família | Teclas |
|---|---|
| Ficheiro | `Ctrl+N` New · `Ctrl+O` Open · `Ctrl+Shift+T` Reopen Closed · `Ctrl+S` Save · `Ctrl+Shift+S` Save As · `Ctrl+Alt+Shift+S` Export As · `Ctrl+E` Export Sprite Sheet · `Ctrl+Shift+X` Repeat Last Export · `Ctrl+I` Import Sprite Sheet · `Ctrl+W` Close · `Ctrl+Shift+W` Close All · `Ctrl+Q` Exit ✎R1 (sem `Alt+F4` no diálogo) |
| Editar | `Ctrl+Z` Undo (2 linhas: sem contexto e `Transformation`) ✎R1 · `Ctrl+Y`/`Ctrl+Shift+Z`/`Ctrl+R` Redo · `Ctrl+X`/`Shift+Del` Cut · `Ctrl+C`/`Ctrl+Ins` Copy · `Ctrl+Shift+C` Copy Merged · `Ctrl+V`/`Shift+Ins` Paste · `Del`/`Backspace` Clear · `F` Fill · `S` Stroke · `Shift+H`/`Shift+V` Flip · `Ctrl+T` Transform · `Ctrl+B` New Brush · `Ctrl+Alt+N` Sprite from Selection · `Shift+R` Replace Color · `Shift+O` Outline · `Ctrl+M`/`F10` Color Curve · `Ctrl+U` Hue/Saturation · `F9` Convolution · `Ctrl+K` Preferences · `Ctrl+Alt+Shift+K` Keyboard Shortcuts · `Ctrl+Space` Run Command · `Ctrl+F` Advanced Mode ✎R1 · `Esc` Cancel Current Operation ✎R1 |
| Selecção | `Ctrl+A` All · `Ctrl+D` (e `Esc` em Transformation) Deselect · `Ctrl+Shift+D` Reselect · `Ctrl+Shift+I` Invert · setas move conteúdo 1 px (`Shift` = 1 tile) · `Alt`+setas move só a borda (`Alt+Shift` = 1 tile) · `Enter`/`Enter Pad` aplica transformação (`Transformation`) ✎R1 |
| Vista | `Ctrl+'` Grid · `Ctrl+Shift+'` Pixel Grid · `Shift+S` Snap to Grid · `Ctrl+H` Extras · `F3` Onion Skin · `F6` miniaturas da timeline · `F7` Preview · `Shift+F7` opacidade das outras camadas **no Preview** ✎R1 · `F8` Full Preview · `F11` Fullscreen · `Tab` Timeline · `Shift+W` alterna layout · `Ctrl+0` Fit · `Shift+Z` Centrar · `Space`+setas scroll 1 px · `Shift+Space`+setas 1 tile ✎R1 |
| Zoom | `1` 100 % · `2` 200 · `3` 400 · `4` 800 · `5` 1600 · `6` 3200 · `` ` ``/`~` 50 · `Ctrl++`/`Ctrl+=`/`Ctrl+Shift+=` in ✎R1 · `Ctrl+-` out |
| Camada | `Shift+N` New · `Shift+Space+N` New abaixo · `Space+N` New Tilemap (pergunta) · `Alt+Shift+N` Group · `Ctrl+J` via Copy · `Ctrl+Shift+J` via Cut · `Ctrl+Shift+V` from Clipboard · `Shift+P`/`F2` Properties · `Shift+X` visibilidade · `Shift+E` abrir grupo · `Shift+1..4` opacidade 64/128/192/255 · `↑`/`↓` camada seguinte/anterior |
| Quadro | `Alt+N` New · `Alt+B` Empty · `Alt+D` cópias · `Alt+M`/`Alt+Shift+D` linked · `Alt+C` Remove · `Alt+I` Reverse · `P` Frame Properties · `Home`/`End` primeiro/último · `←`/`→` anterior/seguinte · `,`/`.` dentro da tag · `Alt+G` Go to Frame · `Enter`/`Enter Pad` Play ✎R1 · `Shift+Enter` Play Preview · `F2` loop (em selecção de quadros) |
| Pincel/cor | `-`/`- Pad` e `+`/`=`/`+ Pad` tamanho ✎R1 · `Alt+1..9` slots · `Space+H`/`X` flip x · `Space+V`/`Y` flip y · `Space+D` diagonal · `Space+R` 90° · `[`/`9` e `]`/`0` índice FG −/+ · `X` troca FG/BG · `A` Edit Palette Mode / `F4` Edit Palette Mode **e** popup FG ✎R1 |
| Sprite/tiles | `C` Canvas Size · `Ctrl+Alt+I` Sprite Size · `Ctrl+P` Sprite Properties · `Space+Tab` modo tiles · `Space+1/2/3` Manual/Auto/Stack |
| Menus | `Alt+F/E/S/L/R/T/V/H` abre File/Edit/Sprite/Layer/Frame/Select/View/Help |
| Abas | `Ctrl+Tab`/`Ctrl+PgDn` seguinte · `Ctrl+Shift+Tab`/`Ctrl+PgUp` anterior · `F5` Refresh · `F12` screenshot |

245 entradas de comando (muitas sem tecla), 26 de ferramenta, 3 momentâneas, 21 modificadores, 1 arrasto.

## O10. Run Command (`Ctrl+Space`)

| Pergunta | Visto |
|---|---|
| O que abre | Janelinha `Run Command` no topo do centro, caixa *«Search for commands, ? for help.»* ([foto](aseprite/P12_b_run_command.png)) |
| `?` | Título vira `Run Command: Help`: *«Type to search through Aseprite's functionality. You can also start with "=" to do math expressions»* |
| `=` | Calculadora: `=2*8+1` → `= 17`, título `Run Command: Math Expressions` ([foto](aseprite/P12_b_run_command_math_undo.png)) |
| Casamento | **Difuso, por subsequência**: `flp` dá New File, Close File, Change Brush: Flip…; `rid` dá Show Grid primeiro (meio da palavra). Não é prefixo |
| Ordem | Melhor casamento primeiro (`show g` → Show Grid, Go to Next Tab, Show Pixel Grid…); não lida a regra |
| Tecla | À direita de cada linha (`Ctrl+'`). Duas teclas aparecem juntas: `Undo` `Ctrl+Z, Ctrl+Z` (as duas de contexto); `Redo` só `Ctrl+Y` (das 3) |
| Indisponíveis | Aparecem **cinzentos** (sem sprite: Trim Sprite, Undo, Go to Frame) |
| Limite | ~10 linhas + `… (19 more results)` |
| Ferramentas | **Não**: `pencil` não dá nada. Só comandos (com parâmetros: «Change Brush: Flip Horizontally», «Set Color Selector: …», «Show Menu: File») |

## O11–O13. Undo History (Edit ▸ Undo History)

| ID | Pergunta | Visto |
|---|---|---|
| O11 | Entradas | Janela flutuante `Undo History (5.4K)` — **o título mostra a memória usada**. Linhas: `Initial State` · `Pencil Tool` · `Line Tool` · `Rectangle Tool` · `Paint Bucket Tool` (nome da ferramenta, não do gesto); a actual realçada ([foto](aseprite/P12_c_undo_history.png)) |
| O12 | Clicar numa antiga e agir | Clique em `Line Tool` = desfaz até lá (as de baixo ficam visíveis). Um traço novo **corta**: fica `Initial · Pencil · Line · Pencil Tool`, título `1.1K`. Com o de fábrica (☐ non-linear) não há ramo |
| O13 | Por sprite | Sim: ao abrir outro sprite a janela mostra a história dele (`Undo History (0 bytes)`, só `Initial State`) |
| O14 | Preferences ▸ Undo | ☐ `Undo Limit:` `∞` `MB` · ☑ `Go to modified frame/layer` · ☐ `Allow non-linear history` · ☑ `Show Undo Tooltip` ([foto](aseprite/P12_f_prefs_files_undo.png)). Non-linear ligado: não medido |

## O15–O16. Recuperação (Preferences ▸ Files ▸ Recover Files)

| ID | Pergunta | Visto |
|---|---|---|
| O15 | Opções de fábrica | ☑ `Automatically save recovery data every` **2 Minutes** (lista: 10 s, 30 s, 1, 2, 5, 10, 15, 30 min) · ☑ `Keep edited sprite data for` **1 Week** · ☑ `Keep closed sprite on memory for` **15 Minutes** ([foto](aseprite/P12_f_escopo_grid_e_intervalo.png)). No `.ini`: `data_recovery = true`, `data_recovery_period = 0.166700` (minutos) |
| O16 | Matar e relançar | Período 10 s, traço, 25–30 s de espera, `kill -9` ao grupo, relançado (2 vezes, a 2.ª com o período já no arranque): **nenhum ficheiro de sessão** em `~/.config/aseprite/` (só `aseprite.ini`, `palettes/`, `Aseprite.log` vazio) nem em `/tmp`; o arranque é o Home normal, sem «Recover» ([foto](aseprite/P12_d_relancamento_sem_recuperacao.png)). Leitura: o Trial não grava recuperação (coerente com «não grava sprites»). Sem prova de onde o pago grava |

## O17. User Data

| Onde | Visto |
|---|---|
| Layer Properties (`Shift+P`) | `Name` + botão de dados à direita · `Mode` · `Opacity`. O botão acrescenta `Color:` (de fábrica `#00000000`, transparente, com alfa) e `User Data:` (caixa de texto multilinha) ([foto](aseprite/P12_e_user_data_layer.png)). A janela não é modal (ficou aberta por cima de `Ctrl+P`) |
| Sprite Properties (`Ctrl+P`) | O mesmo botão ao lado de `File name`. Mais: `Type` · `Size` (`32x32 (80 bytes)`) · `Frames` · Advanced: `Transparent Color` · `Pixel Aspect Ratio` · `Color Profile` sRGB `Assign`/`Convert` · ☐ `Create UUID for layers` ([foto](aseprite/P12_e_sprite_properties.png)) |
| Propriedades tipadas | **Não há UI**: só cor + texto. (Cel/Tag: o mesmo botão, P11 T3 viu-o na Slice; não reaberto aqui) |

## O18–O21. Preferências do módulo

| ID | Página | De fábrica |
|---|---|---|
| O18 | Grid | Escopo no topo: **`Grid for New Documents`** / **`Grid for the Active Document`** (abre neste). ☐ `Visible Grid` · `X` 0 `Y` 0 · `Width` 16 `Height` 16 · `Color` `#0000FF` · `Opacity` 160 ☑ `Auto` · ☐ `Visible Pixel Grid` · `Color` `#C8C8C8` · `Opacity` 160 ☑ `Auto` · `Reset` ([foto](aseprite/P12_f_prefs_grid_background.png)) |
| O19 | Background | Mesmo escopo (`…for the Active Document`). `Checkered Background`: `Size` **16x16** ☑ `Apply Zoom` · `Colors` `#808080` e `#C0C0C0` · `Reset`. Com `Apply Zoom` a 800 % o xadrez de 16 px do sprite fica em 4 quadrados enormes num 32×32 |
| O20 | Timeline | ☑ `Show timeline automatically` · ☐ `Rewind on Stop` · `Default First Frame` 1 · Range: ☐ `Keep selection` · ☑ `Select on Click` · ☑ `Select on Shift + Click` · ☑ `Select on Drag` · ☑ `Drag & drop from edges` · `Reset` ([foto](aseprite/P12_f_prefs_timeline_editor.png)). ⚠️ A timeline **não** abre num sprite novo de fábrica: só com `Tab` |
| O21 | Onion skin | **Não está nas Preferences**: botão direito no botão de onion da timeline abre a configuração: `Position` (3 lugares) · `First Frame` · ☐ `Thumbnails` · ◉ `Merge Frames` / ○ `Red/Blue Tint` · `Opacity` **27 %** · `Opacity Step` **11 %** · ☑ `Loop through tag frames` · ☐ `Current layer only` · ◉ `Behind sprite` / ○ `In front of sprite` · `Reset` · `Set as Defaults` ([foto](aseprite/P12_f_onion_skin.png)). Quantos antes/depois: não está no painel (no Aseprite é o arrasto da faixa no cabeçalho; não medido aqui) |
| O22 | Editor | ☑ Zoom with scroll wheel · ☐ Zoom from center (roda/teclas) · ☑ scroll-bars · ☑ auto-scroll nas bordas · ☐ auto-fit ao abrir · ☑ recta imediata no Pencil · ☐ descartar pincel custom no conta-gotas · `Downsampling` Bilinear mipmapping · `Right-click` Paint with background color |
| — | Lista | General · Files · Color · Alerts · Editor · Selection · Timeline · Cursors · Background · Grid · Guides & Slices · Undo · Theme · Extensions · Aseprite Format · Experimental · Reset; busca no topo |

De fábrica, um sprite novo abre com a janela `Preview` (F7) já aberta e sem timeline.

## ⛔ O que o Trial não deixou medir

- **Lista completa de atalhos pelo `Export`**: o ficheiro sai, mas só com o delta (vazio de fábrica). A tabela de fábrica acima foi lida da tabela de teclas que vem **instalada** com o binário (`usr/share/aseprite/data/gui.xml`, secção `<keyboard>`: dados, não código) e conferida por amostra no diálogo. Licença: EULA proprietária; ⇒ só a tabela de factos (tecla→acção) entra aqui, o ficheiro não foi copiado para o repo.
- **Recuperação de dados**: nenhum ficheiro de sessão em 2 tentativas; o fluxo «Recover Lost Sprites» do pago não é observável no Trial. Onde e em que formato o pago grava: não medido.
- `Allow non-linear history` ligado (ramos), `Undo Limit` a funcionar, `Go to modified frame/layer` desligado: não medidos (tempo).
- Quantos quadros de onion antes/depois e o seu gesto; `Red/Blue Tint` visto só como opção.
- Escopo `Grid for New Documents` vs activo: só a lista; propagação não medida. Background: idem.
- Cel/Tag Properties ▸ User Data: não reabertos (a Slice, P11, tem o mesmo botão).
- `Import` de `.aseprite-keys`; repor uma tecla só; tecla com `Space`/`Super` como modificador.

## Rodada 2 (08/11)

Mesmo instrumento (Xvfb `:77` 1600×1000, HOME novo no scratchpad, `xdotool` com `windowfocus --sync`, `import -window root`); sprite RGB 256×256. Janela principal posta a 1600×1000 e o diálogo de atalhos **redimensionado** pelo canto (aceita) para ~33 linhas por página: a lista inteira de `Commands` foi lida em 12 páginas de 20 cliques de roda, `Tools` e `Action Modifiers` em 2 cada. Cada caso visto **1 vez**. Células corrigidas na tabela de fábrica acima levam **✎R1**.

| ID | Pergunta | Visto |
|---|---|---|
| R1 | Tabela de fábrica, linha a linha no diálogo | **Ferramentas** ✔ (23 + momentâneas `Alt`/`Ctrl`/`Space`) · **Modificadores** ✔ (21, todos com o mesmo contexto) · Ficheiro ✔ salvo `Exit`: só `Ctrl+Q` (**sem `Alt+F4`**) · Editar ✔ + 2 em falta: `Ctrl+F` Advanced Mode, `Esc` Cancel Current Operation · Selecção ✔ (setas/`Shift`+setas e `Alt`/`Alt+Shift`+setas, contexto `Selection`) · Vista ✔ salvo `Shift+F7` = *Toggle Other Layers Opacity **in Preview***, e falta `Shift+Space`+setas = scroll 1 tile · Zoom ✔ + `Ctrl+Shift+=` (3.ª tecla de Zoom In) · Camada ✔ · Quadro ✔ + `Enter Pad` em Play · Pincel/cor ✔ + `- Pad`/`+ Pad`; `F4` = *Switch Edit Palette Mode and Foreground Color Popup* (não o mesmo que `A`) · Sprite/tiles ✔ · Menus `Alt+letra` ✔ · Abas ✔ (`F12` = *Take & Add Screenshot to Steam (sRGB)*) ([foto](aseprite/P12_r2_r1_undo_zoom.png), [foto](aseprite/P12_r2_r1_contexto_selection.png)) |
| R1 | Coluna Context | Vazia na maioria (= global). Com nome: `Normal` em `←`/`→` (quadro), `↑`/`↓` (camada), `F2` Layer Properties · `Transformation` em `Enter`/`Enter Pad` Apply, `Esc` Deselect, e numa 2.ª linha `Ctrl+Z` Undo · `Selection` nas 16 de mover conteúdo/borda · **`Frames Selection`** (assim, com espaço) em `F2` Set Loop Section · modificadores: `Freehand Tool`, `Move Tool`, `Rotating Selection`, `Scaling Selection`, `Selection`, `Shape Tool`, `Translating Selection`. Há linhas-fantasma sem tecla duplicando o nome (`Go to Next Frame`, `Set Loop Section`, `Selection: Add Selection`…) |
| R2 | Non-linear ligado: desfazer a uma antiga e agir | 4 traços de Pencil; clique no 1.º `Pencil Tool` (canvas só com 1 traço); uma `Line Tool` nova **não corta**: a lista fica `Initial · Pencil ×4 · Line Tool` — **uma lista plana cronológica**, sem recuo nem marca de ramo; título `2.8K → 13.3K` ([foto](aseprite/P12_r2_r2_nonlinear_corte.png)) |
| R2 | Voltar ao ramo cortado | Clique no 4.º `Pencil` = canvas com os 4 traços **sem** a linha; `Ctrl+Z` dali = 3 traços ([foto](aseprite/P12_r2_r2_nonlinear_ramo_volta.png)). Clique em `Line Tool` e `Ctrl+Z` = canvas com **os 4 traços** (o vizinho de cima na lista), não o pai real (1 traço) ([foto](aseprite/P12_r2_r2_nonlinear_ctrlz.png)). ⇒ o undo anda pela **lista**, não pela árvore |
| R3 | Undo Limit | ☑ `Undo Limit` **1 MB** (`.ini`: `size_limit = 1`), 12 baldes de tinta ao sprite inteiro (branco/preto alternados): a história fica **só `Initial State` + o último `Paint Bucket Tool`** — as 4 Pencil, a Line e os 11 baldes antigos **somem**, título `1.5M` (passa do limite: a entrada mais nova fica sempre). A **8 MB**, mais 12 baldes: todos ficam (lista com barra de rolagem, ~13 baldes), título `4.3M` ([foto](aseprite/P12_r2_r3_limite_1mb_8mb.png)). Saem as **mais velhas**, sem aviso. ⚠️ Não explicado: 1 entrada a 1.5M contra ~0.33M/balde a 8 MB (talvez o ramo non-linear, ligado, conte) |
| R4 | Onion: quantos e o gesto | `F3` liga; com 8 quadros, no quadro 4 a faixa é **`[3 … 5]`: 1 antes, 1 depois** de fábrica, desenhada como **colchetes nos cabeçalhos dos quadros** da timeline. **Arrastar** o colchete esquerdo até o quadro 1 → 3 antes; o direito até o 8 → 4 depois. A faixa é **relativa** ao quadro actual: `←` para o quadro 3 dá `[1 … 7]` (antes cortado no 1.º quadro). Nada no `.ini` (fica no documento) ([foto](aseprite/P12_r2_r4_onion_faixa.png): fábrica · esq. arrastado · dir. arrastado · quadro 3) |
| R5 | Cel Properties | `Opacity` (barra 100 %) · `Z-Index` (número + setas) · botão de dados → `Color` (`#000000` com alfa 0, xadrez) · `User Data` (texto) |
| R5 | Tag Properties (`New Tag`) | `Name` (`Tag`) + botão de dados · `From` · `To` · `Animation Direction` (Forward…) · ☐ `Repeat` `∞` · dados: `Color` (`#000000` **opaco**) · `User Data` (texto) ([foto](aseprite/P12_r2_r5_cel_tag_user_data.png)). Igual à Layer/Sprite: só cor + texto, sem propriedades tipadas |
| R6 | Import de delta | Ficheiro escrito à mão [`P12_r6_import.aseprite-keys`](aseprite/P12_r6_import.aseprite-keys) (`ShowGrid` += `Ctrl+Shift+G`; `SnapToGrid` `Shift+S` `removed="true"`). `Import` abre o seletor próprio do Aseprite, **sem confirmação nem relatório**. Resultado: `Show Grid` = `Ctrl+'` **e** `Ctrl+Shift+G` (soma, não substitui), `Snap to Grid` **sem tecla**, nas linhas de Menus e Commands ([foto](aseprite/P12_r2_r6_import_grid.png)). **`Cancel` do diálogo desfaz o import** (reaberto: tudo de fábrica): só vale com `OK` |

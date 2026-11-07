# P11 — A ferramenta Slice no Aseprite Trial (só medido)

07/11. Aseprite Trial 1.3.18.6, Xvfb `:84` 1600×1000, `screen_scale 2`, HOME limpo, ponteiro e teclado reais, sobre `.ase` nossos (`ferramentas/trial_slices.py`, `trial_slices_tool_fixturas.py`). Leitura: diálogo Slice Properties e fotos. Fixtura: `crates/ph2d-pixel-io/tests/fixtures/oracle_slices_ferramenta.json`. Condutores: `ferramentas/trial_slices_tool.py`, `trial_slices_tool_t7.py`. Salvo dito, cada caso foi visto **1 vez**. Coordenadas em pixels do sprite; quadros de 0; centro e pivot relativos aos bounds.

**Lições do instrumento.** `--appimage-extract-and-run` partilha `/tmp` e deu «Text file busy» com outro agente a arrancar: usar cópia própria (`--appimage-extract`). O arrasto do Slice com assentamento de 0,08 s antes do largar deu largura a variar 21–24 para o mesmo destino; com 0,25–0,3 s deu 22 em 6/6. Duas tomadas de `xdotool` em paralelo sobre o mesmo ecrã contaminam: os casos T6 «Reverse» e a 1.ª foto dos diálogos foram refeitos.

## T1. Criar

| Pergunta | Visto |
|---|---|
| Lugar | Botão próprio da barra de ferramentas (6.º, entre Zoom e Paint Bucket). Tooltip `Slice Tool` / `Shortcut: Shift+C`. Grupo escondido não medido |
| Arrasto em área vazia | Cria. A nova NÃO fica seleccionada; a caixa da barra mostra o nome dela e os botões ficam apagados |
| Nome | `Slice 1`, `Slice 2`… Depois de apagar a 2: lista 1,3,4 e a seguinte é `Slice 5`; não reaproveita. `max+1` × contador monótono: não distinguido |
| Cor | `#0000FF` nas criadas (lida em 1; todas azuis na tela); não cicla. Vem de Preferences ▸ Guides & Slices ▸ Default Color |
| Direita→esquerda | Normaliza (x e w positivos) |
| Passa do canvas | Recorta: de (50,40) a (70,70) dá `50,40,14,24`; de (-5,44) a (5,50) dá `0,44,5,6` |
| Clique sem arrastar | Em vazio não cria; tira a selecção |
| Encaixe | Inteiro. Pixel 8 → pixel 30, ponteiro no meio do pixel: `8,8,22,22`, 6/6. Pixel 8 → 32: `24`. A borda final é a do pixel onde o ponteiro está, sem contar esse pixel, nesse ponto do pixel; regra por sub-pixel não fechada |
| Tamanho mínimo | Por arrasto de canto: `1×1` (T2). Criação com arrasto de 0: não cria |
| Arrasto começado em cima de slice | Não cria. Em slice não seleccionada: **move-a** e selecciona (`8,8`→`12,12`) |
| Barra de contexto | `All` · `None` · caixa de nomes com lista (`Search` quando vazia, ordem alfabética) · `Transform` (tooltip `Transform pixels along slice modification`) · botão Propriedades · botão `X` Apagar. Os dois últimos apagados sem selecção |

Escolher um nome na lista só preenche a caixa: não selecciona, nem com `Return` (1 vez).

## T2. Seleccionar e editar

| Gesto | Visto |
|---|---|
| Clique | Selecciona e substitui |
| `Shift`+clique | Fica só a segunda; não soma (1 vez) |
| `Ctrl`+clique em outra | A selecção não muda |
| `Shift`+arrasto | Marquise: selecciona as que toca; a slice que só tocava com a coluna da borda entrou |
| `All`/`None` | Todas / nenhuma |
| Clique em vazio | Limpa |
| Arrasto no interior | Move; `Ctrl`+arrasto também move |
| Alças | Bordas e cantos da slice seleccionada redimensionam (8 pontos: 4 bordas, 4 cantos; lados e cantos provados, lado de cima/baixo do contorno não). Sem quadradinhos desenhados |
| Passar da oposta | Trava em `1×1`: canto inf-dir para além do sup-esq dá `4,4,1,1` |
| Setas | Mudam de quadro; a slice não anda |
| `Esc` | Limpa a selecção de slices |
| `Delete` | Foco no canvas: apaga a slice inteira. ⚠️ Depois de clicar no cabeçalho da linha do tempo, `Delete` com `All`: a slice ficou e o índice 3 passou a mostrar a arte do quadro 4. Leio como quadro apagado; a contagem de quadros não foi fotografada (1 vez, inferência) |
| `Ctrl+A` | Selecção de PIXELS do canvas inteiro; a de slices não muda |
| `Ctrl+C`/`Ctrl+V` | Sem selecção de pixels: não cria slice |
| Botão direito em slice seleccionada | `Slice Properties...` · `Duplicate Slice` · `Delete Slice` |
| Botão direito em não seleccionada | Sem menu (1 vez) |
| `Duplicate Slice` | `Slice 2 Copy`, deslocada ~+2,+2, seleccionada; a original perde a selecção |
| Duplo-clique | Abre Slice Properties |

**Aspecto.** Não seleccionada: contorno fino na cor da slice. Seleccionada: contorno duplo claro/escuro, a cor some do contorno. Passar por cima: nada muda na foto. Sem rótulo de nome no canvas em nenhum estado (a caixa da barra é o único sítio do nome).

**Menus.** View ▸ Show: `Layer Edges` · `Selection Edges` ✓ · `Grid` · `Auto Guides` ✓ · `Slices` ✓ · `Pixel Grid` · `Tile Numbers` ✓ · `Brush Preview` ✓. Preferences ▸ Guides & Slices: `Layer Edges Color` #0000FF · `Auto Guides Color` #0000FF · `Default Color` #0000FF · ☐ `Prefer slice keyframes (obsolete behavior)` ([foto](handoffs/P11_t2_preferencias_guides_slices.png)). [Menu do direito](handoffs/P11_t2_menu_direito.png).

## T3. Slice Properties

Abre por duplo-clique, botão da barra ou menu do direito. Campos, por ordem: `Slice Name:` · botão de dados do utilizador · `Bounds:` `X` `Y` `Width` `Height` · ☐ `9-Slices` (4 campos) · ☐ `Pivot` (2 campos) · `OK` `Cancel`. O botão abre `Color:` e `User Data:` entre o nome e os bounds ([foto](handoffs/P11_t3_dialogo_com_user_data.png)). Tooltips nos campos: nenhum em X, `9-Slices`; o do botão de dados não foi lido.

| Ao ligar, slice | 9-Slices | Pivot |
|---|---|---|
| 10×10 | `1,1,8,8` | `0,0` |
| 24×24 | `1,1,22,22` | `0,0` |
| 15×9 | `1,1,13,7` | `0,0` |

Lei vista: margem de 1 em todos os lados. Pivot nasce no canto.

| Entrada | Resultado |
|---|---|
| Centro `100,100,100,100` em 24×24 | Aceite sem corrigir ao reabrir |
| Width `0` | Vira `1` |
| Pivot negativo pelo teclado | Não medido (`-5` no campo falhou no condutor). `-3` por fixtura é aceite e sobrevive: T7 |

O diálogo edita a tecla do quadro actual (T5). O nome e a cor valem para a slice inteira: não medido por quadro.

## T4. 9-slice e pivot na tela

**O centro do 9-slice ARRASTA-SE com a ferramenta; o plano está errado nesse ponto. O pivot NÃO se arrasta: a metade do plano sobre o pivot confirma-se.**

Slice `nine` `4,4,24,24`, centro `8,8,8,8`, seleccionada ([foto](handoffs/P11_t4_nove_e_pivot_na_tela.png)):

| Arrasto | bounds | centro |
|---|---|---|
| Linha esquerda 12→16 | igual | `12,8,4,8` |
| Linha esquerda 12→24, além da direita | igual | `15,8,1,8`: trava com largura 1 |
| Linha de cima 12→14 | igual | `8,10,8,6` |
| Linha direita 20→18 | igual | `8,8,6,8` |
| Linha de baixo 20→22 | igual | `8,8,8,10` |
| Canto (12,12)→(14,14) | igual | `10,10,6,6` |
| Canto (20,20)→(22,22) | igual | `8,8,10,10` |
| Interior do centro (16,16)→(18,18) | `6,6,24,24` | igual: move |
| Banda (8,8)→(10,10) | `6,6,24,24` | igual: move |

Redimensionar os bounds não escala: o `x,y` do centro fica, e a largura do centro leva a diferença.

| Bounds | Resultado | centro |
|---|---|---|
| Esquerda -2 | `2,4,26,24` | `8,8,10,8` |
| Esquerda +2 | `6,4,22,24` | `8,8,6,8` |
| Esquerda até w=17 | `11,4,17,24` | `8,8,1,8` |
| Esquerda até w=16 | `12,4,16,24` | **9-Slices desliga** |
| Direita -2 | `4,4,22,24` | `8,8,6,8` |
| Direita até w=17 | `4,4,17,24` | `8,8,1,8` |
| Direita até w=16 / 11 | `4,4,16,24` / `4,4,11,24` | **9-Slices desliga** |
| Canto sup-esq -2,-2 | `2,2,26,26` | `8,8,10,10` |
| Canto inf-dir +4,+4 | `4,4,28,28` | `8,8,12,12` |

Cada um destes volta com 1 `Ctrl+Z`, centro incluído. Efeito de redimensionar no pivot: não medido.

**Pivot.** Quadradinho de ~1 pixel na cor da slice, seleccionada ou não, desenhado no pixel `bounds+pivot`. Slice `pivot` `36,4,24,24`, pivot `12,12`, agarrado no pixel (48,16) e arrastado +4,+4:

| Modificador | Resultado |
|---|---|
| nenhum (2 vezes, em (48,16) e (49,17)) | Move a slice: `40,8`, pivot `12,12` |
| `Ctrl` | Move a slice: `40,8`, pivot `12,12` |
| `Shift` | Slice e pivot iguais (é a marquise) |
| `Alt` | Slice e pivot iguais |

Com pivot `12,24`, no pixel (48,28), fora dos bounds: slice e pivot iguais. Um pixel acima, (48,27), agarra a borda: altura 17.

Cursor: a foto por `import` não mostra o cursor sobre as linhas; não medido.

## T5. Teclas por quadro

`keys.ase`, `moves`, teclas em 0, 2, 4. A tecla **dura até à seguinte**: 0–1 `8,8`, 2–3 `24,8`, 4–5 `24,24,32,16`.

| Acção | Visto |
|---|---|
| Mover no quadro 3 (+6,+4) | Cria tecla em 3: `30,12`. 0, 1, 2, 4, 5 intactos |
| `Ctrl+Z` 1× | Valores voltam. ⚠️ A tecla em 3 **fica**: depois disso, X=1 no quadro 2 não chega ao 3 |
| Diálogo X=1 no quadro 2, tecla própria | 2 e 3 ficam `1,8`; 4 intacto |
| Diálogo com `OK` sem mudar no quadro 3 | **Não cria tecla**: X=1 no quadro 2 chega ao 3 |
| Slice nova no quadro 2 | Aparece também nos quadros 0, 1, 3, 5 no mesmo sítio. Quadro da tecla: não distinguido |
| Width 0 no quadro 2 | Vira 1 |
| Apagar slice noutro quadro | Só provado o foco no canvas no quadro 0 (T1). No quadro 3 foi o quadro que saiu (T2). «Só desse quadro em diante»: não medido |
| Apagar uma tecla | Nenhuma entrada de menu nem de diálogo vista para isso |
| Undo | 1 por arrasto; 1 por `OK` do diálogo com mudança |
| Linha do tempo | Nenhuma marca de tecla nem linha de slices |
| Reproduzir | As slices acompanham o quadro (3 fotos) |
| Selecção ao mudar de quadro | Sobrevive, por clique, setas e reprodução |

## T6. Operações de quadro

⛔ **As teclas ficam no mesmo ÍNDICE de quadro; nenhuma operação medida as desloca, apaga ou funde; a arte anda e a slice não.** Sessão própria por caso, leitura em todos os quadros.

| Operação | Quadros | Por índice |
|---|---|---|
| Original | 6 | 0–1 `8,8` · 2–3 `24,8` · 4–5 `24,24,32,16` |
| `New Frame` `Alt+N` no 1 | 7 | 0–1 `8,8` · 2–3 `24,8` · 4–6 `24,24,32,16` |
| `New Empty Frame` `Alt+B` no 1 | 7 | igual ao anterior |
| `Delete Frame` `Alt+C` no 2, com tecla | 5 | 0–1 `8,8` · 2–3 `24,8` · 4 `24,24,32,16` |
| `Delete Frame` no 1, sem tecla | 5 | igual |
| `Delete Frame` no 0, 1.ª tecla | 5 | igual |
| `Duplicate Linked Cel(s)` `Alt+M` no 1 | 6 | igual ao original; não cria quadro |
| Mover o 1 para o índice 3 (arrastando pela borda do cabeçalho) | 6 | igual ao original; a arte do 1 foi para o índice 3 (foto) |
| `Reverse Frames` `Alt+I`, intervalo 1–3 | 6 | igual ao original; arte não conferida |

Arrastar o cabeçalho por dentro faz selecção de intervalo, não move. Quadro a ficar sem tecla por ter menos quadros que a última tecla: não medido. Undo dessas operações: não medido.

## T7. Geometria

`full.ase`: `nine` `4,4,24,24` c`8,8,8,8` · `pivot` `36,4,24,24` p`12,24` · `both`₀ `4,36,24,20` c`4,4,16,12` p`0,0` · `both`₃ `36,36,16,20` c`2,6,12,8` p`8,10`.

| Operação | nine | pivot | both₀ | both₃ |
|---|---|---|---|---|
| Sprite Size 200 % | `8,8,48,48` c`16,16,16,16` | `72,8,48,48` p`24,48` | `8,72,48,40` c`8,8,32,24` p`0,0` | `72,72,32,40` c`4,12,24,16` p`16,20` |
| Sprite Size 50 % | `2,2,12,12` c`4,4,4,4` | `18,2,12,12` p`6,12` | `2,18,12,10` c`2,2,8,6` p`0,0` | `18,18,8,10` c`1,3,6,4` p`4,5` |
| Canvas Left+Top +10 | `14,14,24,24` c igual | `46,14,24,24` p igual | `14,46,…` c, p iguais | `46,46,…` c, p iguais |
| Canvas Left+Top -10 | `-6,-6,24,24` c igual | `26,-6,24,24` p igual | `-6,26,…` c, p iguais | `26,26,…` c, p iguais |
| Crop (0,0)–(40,32), canvas 40×32 | igual | igual; `36+24` passa do canvas | na lista, toda fora | não lido |
| Flip Canvas Horizontal | **igual** | **igual** | não lido por posição: vazio no ponto espelhado | idem |
| Flip Canvas Vertical | não lido | não lido: vazio no ponto espelhado | **igual** | não lido |
| Rotate Canvas 90 CW | **igual** | **igual** | não lido: vazio no ponto girado | idem |

**Sprite Size:** bounds, centro e pivot escalam. **Arredondamento** em `odd.ase`, 50 %:

| Slice | Entrada | Saída |
|---|---|---|
| `odd` | `5,7,15,9` c`3,1,9,7` p`7,3` | `2,3,8,5` c`1,0,5,4` p`3,1` |
| `odd2` | `33,11,13,21` c`1,5,11,11` p`-3,25` | `16,5,7,11` c`0,2,6,6` p`-1,12` |

Posição e pivot truncam para zero (`-3`→`-1`). O tamanho sobe. Os casos cabem igualmente em «tamanho arredonda para cima» e em «cada borda trunca» (`5+15=20`→`10-2=8`); para os separar falta um caso `x` par com `w` ímpar (`4,15`): 8 × 7. Não medido.

**Canvas Size:** bounds transladam, centro e pivot não mudam, slice com X/Y negativos sobrevive. **Trim** (`trim.ase`, margem 8, canvas 48×48): `cross` `4,4,24,24` c`8,8,8,8` p`12,12` → `-4,-4,24,24`, c e p iguais; `outside` continua na lista. **Crop:** nada apaga slices; recorte com origem ≠ 0 não medido. **Flip e Rotate Canvas não tocam nas slices** (a arte vira; a slice fica). **Undo** 1× restaura `nine` após 200 %, Crop e R90; lista completa depois de Crop.

## T8. Com o desenho

| Pergunta | Visto |
|---|---|
| `Transform` ligado | Mover a slice leva os pixels e deixa buraco transparente ([foto](handoffs/P11_t8_transform_move_pixels.png)). Por omissão desligado |
| «Select slice» / slice → selecção | Não há na barra de contexto, no menu do direito, nem nos menus `Select`, `Sprite`, `Frame`. `Edit` lido até `Invert` |
| Filtro ou preenchimento limitado à slice | Sem entrada vista; não experimentado |
| Slices com outra ferramenta | Com a Eyedropper activa e o diálogo Canvas Size aberto, os contornos estão na tela (fotos, 1 vez). Clicáveis: não medido |

## Nomes de UI exactos

| Sítio | Nome |
|---|---|
| Ferramenta | `Slice Tool`, `Shift+C` |
| Barra | `All`, `None`, `Search`, `Transform`, `Transform pixels along slice modification` |
| Menu do direito | `Slice Properties...`, `Duplicate Slice`, `Delete Slice` |
| Diálogo | `Slice Properties`, `Slice Name:`, `Color:`, `User Data:`, `Bounds:`, `X`, `Y`, `Width`, `Height`, `9-Slices`, `Pivot`, `OK`, `Cancel` |
| View ▸ Show | `Slices` |
| Preferences | `Guides & Slices`, `Default Color:`, `Prefer slice keyframes (obsolete behavior)` |
| Sprite | `Sprite Size...`, `Canvas Size...`, `Rotate Canvas` ▸ `180`, `90 CW`, `90 CCW`, `Flip Canvas Horizontal`, `Flip Canvas Vertical`, `Crop`, `Trim` |
| Frame | `New Frame`, `New Empty Frame`, `Duplicate Cel(s)`, `Duplicate Linked Cel(s)`, `Delete Frame`, `Reverse Frames` |
| Canvas Size | `Left:`, `Right:`, `Top:`, `Bottom:`, `Trim content outside the canvas` |

## Não medido

- Cursor sobre as alças: a foto não o mostra.
- Multi-selecção: ligar `9-Slices`/`Pivot` E digitar os números juntos (caso escrito no condutor, `G3c`, não correu); qual a cor que o diálogo mostra (só visto `both`); lei da borda com várias; Height e Y; `Width` pelo diálogo com UMA slice: o centro acompanha?
- Mover em quadro herdado com tecla só no 0: o 0 também muda (G4); regra por detrás não fechada; o mesmo em `keys.ase`, quadro 1: não medido. Tecla criada depois de undo em redimensionar.
- `?` do diálogo; alcance dos campos acima de `99999`.
- Duas regras de encaixe: sub-pixel fino da borda final; `Shift`+clique repetido.
- Bounds por cima/baixo com pivot além do caso `Cima +16`; canto inf-dir com pivot.
- Filtro de escala do `Transform`; `Transform` com célula ligada, camada bloqueada, camada de tiles.
- Flip de selecção (G7, não correu); Rotate num canvas não quadrado; Canvas Size com `Trim content`.
- Undo de `New Frame` e de `Reverse Frames` (G8, não correram); Reverse sem conferir a arte.
- `Paste Special`; Copy de slice; paleta de comandos (`Ctrl+Space` não abriu pelo xdotool: as letras foram atalhos).
- Clique em slice com o Pencil ou outra ferramenta activa (G9, não correu). Com `Slices` desligado: duplo-clique e arrasto sobre uma slice existente (não correram). `Prefer slice keyframes` ligado.
- Reabertura ou gravação: o Trial não grava.

## Rodada 2 (G1–G9)

07/11, mesma versão e método; condutor `ferramentas/trial_slices_tool_g.py`; fixturas novas no próprio condutor (`odd3`, `pv`, `tr`). Fixtura de números: `oracle_slices_ferramenta.json`, chave `rodada2`. Salvo dito, cada caso foi visto **1 vez**, sessão própria.

### G1. Sprite Size: arredondamento

⛔ **Cada borda trunca, não «o tamanho sobe»:** `x'=trunc(x·s)`, `w'=trunc((x+w)·s)-x'`; o mesmo em y. **O centro faz o mesmo no seu próprio referencial** (relativo aos bounds, sem somar o `x` da slice). Pivot: `trunc(p·s)`, para zero. Os 12 casos abaixo e os 2 da T7 cabem na fórmula (conferido por script). 33 %: `s=0,33` e `s=21/64` (canvas 21×21) dão o mesmo em todos; não distinguidos.

| Entrada | 50 % | 150 % | 33 % |
|---|---|---|---|
| `4,6,15,9` c`3,1,9,7` p`7,3` | `2,3,7,4` c`1,0,5,4` p`3,1` | `6,9,22,13` c`4,1,14,11` p`10,4` | `1,1,5,3` c`0,0,3,2` p`2,0` |
| `35,7,16,11` c`2,3,11,5` p`5,-3` | `17,3,8,6` c`1,1,5,3` p`2,-1` | `52,10,24,17` c`3,4,16,8` p`7,-4` | `11,2,5,3` c`0,0,4,2` p`1,0` |
| `7,33,17,20` c`5,2,7,13` p`9,19` | `3,16,9,10` c`2,1,4,6` p`4,9` | `10,49,26,30` c`7,3,11,19` p`13,28` | `2,10,5,7` c`1,0,2,4` p`2,6` |
| `40,34,19,13` c`5,3,9,7` p`-5,11` | `20,17,9,6` c`2,1,5,4` p`-2,5` | `60,51,28,19` c`7,4,14,11` p`-7,16` | `13,11,6,4` c`1,0,3,3` p`-1,3` |

### G2. Apagar em quadro > 0

⛔ **A slice inteira sai, de todos os quadros, pelos 3 caminhos.** `keys.ase`, foco no canvas (clique na slice antes):

| Caminho | Quadro | Quadros depois | Slice | `Ctrl+Z` 1× |
|---|---|---|---|---|
| Botão `X` | 3 (tecla herdada) | 6 | Sem contorno nem diálogo em 0–5 | As 3 teclas voltam, leitura por quadro igual à original |
| `Delete` | 3 | 6 | idem; lista vazia | idem |
| Direito ▸ `Delete Slice` | 3 | 6 | idem; lista vazia | idem |
| `Delete` | 2 (tecla própria) | 6 | idem; lista vazia | idem |

A foto da lista depois do `X` saiu sem a barra (instrumento); a ausência está nas 6 leituras e nas fotos.

### G3. Multi-selecção (`All`, 3 slices)

Diálogo: todos os campos mostram `*` em cinzento, incluindo nome; `9-Slices` e `Pivot` desligados. A cor mostra UMA: `#FF8000` A128, a de `both` (qual é a regra: não distinguido). Barra: caixa com `both`.

| Mudança + `OK` | Resultado |
|---|---|
| Nada | Nada muda |
| Nome `zzz` | As 3 ficam `zzz`, incluindo a 2.ª tecla de `both` (3 slices com o mesmo nome) |
| Cor `#FF00FF` | As 3 ficam magenta (foto) |
| X `20` | As 3: X=20; Y, W, H, centro e pivot de cada uma ficam |
| Width `30` | As 3: W=30; X, Y, H ficam; **centro não muda** (`nine` fica c`8,8,8,8`) |
| Ligar `9-Slices`, sem número | **Nada muda**: `pivot` continua sem centro; `nine` e `both` iguais |
| Ligar `Pivot`, sem número | **Nada muda**: `nine` continua sem pivot |
| Centro X `3`, caixa desligada | ⚠️ `nine` c`0,8,8,8`, `both` c`0,4,16,12`: **ficou 0, não 3**; `pivot` continua sem centro |
| Pivot X `3`, caixa desligada | ⚠️ `pivot` p`-21474836…,24`, `both` p`-21474836…,0` (cortado pela largura do campo); `nine` continua sem pivot. Defeito visível do Trial: não copiar |

| Gesto com `All` | Resultado | `Ctrl+Z` 1× |
|---|---|---|
| Arrasto (+4,+4) a partir de `nine` | As 3 andam +4,+4; centro e pivot iguais | As 3 voltam |
| Borda direita de `nine`, -4 | Muda as 3, e **não por delta igual**: `nine` `4,4,23,24` c`8,8,7,8` · `pivot` `34,4,23,24` p`12,24` · `both` `4,36,23,20` c`4,4,15,12` p`0,0`. Regra não fechada | As 3 voltam |
| `Delete` | As 3 saem; lista vazia; barra de estado `3 slice(s) removed` | As 3 voltam, 1× |
| Botão `X` | As 3 saem | As 3 voltam, 1× |

### G4. Redimensionar bounds com pivot

⛔ **O valor do pivot nunca muda; não é recortado nem deslocado.** Daí, puxar pela esquerda ou por cima **anda com o pivot no mundo** (`8+12=20` → `12+12=24`). `pv.ase`, quadro 0, `8,8,24,24` p`12,12`:

| Gesto | Resultado | p | `Ctrl+Z` 1× |
|---|---|---|---|
| Direita até w=8 (pivot fica fora) | `8,8,8,24` | `12,12` | Volta |
| Esquerda +4 | `12,8,20,24` | `12,12` | Volta |
| Esquerda +16 | `24,8,8,24` | `12,12` | Volta |
| Cima +16 | `8,24,24,8` | `12,12` | Volta |
| Canto sup-esq +4,+4 | `12,12,20,20` | `12,12` | Volta |

**Tecla herdada** (teclas 0, 2, 4; quadro 3, direita -4): 3 fica `30,30,20,24` p`6,18`; 2 `30,30,24,24`, 4 e 5 `8,30,24,24`, 0 `8,8,24,24` intactos. Cria tecla em 3. ⚠️ Tecla **só no quadro 0**, mover no quadro 1 (`tr.ase`): **o quadro 0 também muda** (0, 1, 2 todos `20,20`), com e sem `Transform` (leitura em cada quadro, 1 vez cada). Contradiz «cria tecla» só nesse caso; regra (só tecla? quadro 0?) não fechada.

### G5. `Transform` ligado

`tr.ase`: camadas `base` (cheia) e `top` (2 blocos); `top` sem cel no quadro 2; slice `12,12,32,24`.

⛔ **Só a cel da camada ACTIVA no quadro ACTUAL muda; 1 `Ctrl+Z` desfaz slice e pixels. Redimensionar ESCALA, não recorta.**

| Gesto | Pixels (fotos dos 3 quadros) | Slice |
|---|---|---|
| Mover +8,+8, `top` activa, quadro 0 | Blocos do `top` no 0 andam +8,+8; `base` e quadro 1 intactos; nada no 2 | `20,20,32,24` |
| Mover +8,+8, `base` activa, quadro 0 | `base` do 0 anda e deixa um L transparente; `top` e quadro 1 intactos | `20,20,32,24` |
| Mover +8,+8, `top`, quadro 1 | Blocos do 1 andam; os do 0 ficam | `20,20,32,24` em 0, 1, 2 (G4) |
| Canto inf-dir +8,+8, `top` | Blocos do 0 maiores; `base` e quadro 1 intactos | `12,12,40,32` |
| Canto inf-dir -8,-8, `top` | Blocos do 0 menores, dentro da caixa; `base` e quadro 1 intactos | `12,12,24,16` |

Fotos a escala de 4×; o filtro de escala não foi lido. [Mover a base](handoffs/P11_g5_mover_base.png) · [escalar](handoffs/P11_g5_escala.png).

### G6. Entradas no diálogo

| Entrada | No campo | Depois de `OK` |
|---|---|---|
| Pivot X `-5` | `-5` | `-5` |
| X `-10` | `-10` | `-10` |
| Width `99999` | `99999` | `99999` |
| Height `-7` | `-7` | `1` |
| Width `abc` | `abc`: o campo aceita letras | `1` |
| Width `12abc` | `12abc` | `12` |
| X `1.5` | `1.5` | `1` |
| X vazio | vazio | `0` |

**Nome, cor e dados do utilizador valem para a slice inteira:** mudados no quadro 0, `renamed`, `#FF00FF`, `hello` lidos nos quadros 0–5; bounds e pivot de cada tecla iguais. 1 `Ctrl+Z` desfaz os 3 (lido no quadro 3). A cor abre um popup: paleta, `Index`·`RGB`·`HSV`·`HSL`·`Gray`·`Mask`, `#` hex, `R G B A`. `Esc` no popup fecha também o diálogo.

### G7. Geometria (leituras no diálogo)

| Operação | Arte (foto) | Slices |
|---|---|---|
| Rotate Canvas 180, `full.ase` | Girada | **Iguais**: `nine`, `pivot`, `both`₀, `both`₃ no sítio original, centro e pivot iguais |
| Rotate Canvas 90 CCW | Girada | **Iguais**, as mesmas 4 leituras |
| `Shift+H` (Edit ▸ Flip Horizontal) sem selecção | A barra vermelha passa ao canto oposto | **Iguais**, as mesmas 4 leituras |
| Crop com selecção de (8,8), canvas fica 40×32 | Recortada | **Transladam -8,-8**: `nine` `-4,-4,24,24` c igual · `pivot` `28,-4,24,24` p`12,24` · `both`₀ `-4,28,24,20` c, p iguais. A selecção de pixels continua desenhada depois |

### G8. Undo de operações de quadro

| Operação | Antes do undo | Depois |
|---|---|---|
| `Delete Frame` no 5 e no 4 (ficam 4 quadros, a tecla do 4 fora) + 2× `Ctrl+Z` | Fotos dos 4 quadros | 6 quadros; 0–1 `8,8` · 2–3 `24,8` · 4–5 `24,24,32,16`: a tecla do 4 sobreviveu |
| Mover o 1 para a frente, arrastando pela borda do cabeçalho + `Ctrl+Z` | A arte anda; slices no mesmo índice (fotos) | Arte na ordem original; as mesmas leituras |

Destino exacto do mover: lido a olho na foto reduzida (índice 4), não confirmado.

### G9. Parcial: o prazo cortou

| Caso | Visto |
|---|---|
| `View ▸ Show ▸ Slices` desligado | Os contornos somem (foto) |
| Arrastar em vazio com a Slice Tool, Slices desligado | A caixa da barra passa a `Slice 1`, sem contorno: a ferramenta continua a criar (foto, 1 vez) |

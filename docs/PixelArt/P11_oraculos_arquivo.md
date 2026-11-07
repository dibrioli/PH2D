# P11 Slices: ficheiro, linha de comando e exportação (oráculos, 07/11)

Só medido; o que vem de documentação diz «documentado, não corrido». Aseprite Trial 1.3.18.6 como caixa preta: nenhum código lido, nada copiado. A ferramenta Slice em si é da outra medição.

Entradas: `.ase` nossos (`ferramentas/trial_slices.py`, `ferramentas/trial_slices_arquivo.py gerar`). Fixture: `crates/ph2d-pixel-io/tests/fixtures/oracle_slices_arquivo.json`.

```
S=<scratch>; A=$S/ase; F=docs/PixelArt/ferramentas
python3 -I $F/trial_slices.py gerar $A && python3 -I $F/trial_slices_arquivo.py gerar $A
python3 -I $F/trial_slices_arquivo.py correr $A $S/b                        # 88 casos, ~10 s; grava batch.json
python3 -I $F/trial_slices_arquivo_parse.py <reais...> > $S/f2.json         # F2
python3 -I $F/trial_slices_arquivo_fotos.py $A $S/fotos                     # F2-F4, Xvfb :83, ~4 min
python3 -I $F/trial_slices_arquivo_fixture.py $S/b/batch.json $S/b2/batch.json $S/f2.json $A <fixture>   # confere as leis por assert
```

⛔ **O ponto principal:** o Trial recusa imagem, mas **escreve o JSON de `--data`**. Layout, `meta.slices` e nomes de item medem-se de verdade. O corte em píxeis (`--save-as`, `--split-slices`, `--slice`) não se mede. Rodei só as opções do `--help` e o botão Export; `--script` (Lua) não foi corrido nem tentei contornar a recusa.

## F1: que oráculos têm slices

| Oráculo (licença por artefacto) | Tem slices? | Medido |
|---|---|---|
| Aseprite Trial 1.3.18.6 (EULA, só corrido) | Sim | Tabela abaixo |
| LibreSprite 1.3-1.1, `pacman`: GPL-2.0-only (só corrido) | **Não** | Lê o `.ase` e avisa `Warning: Unsupported chunk type 8226 (skipping)` por chunk 0x2022; `--help` sem «slice» (0 linhas); abrir e regravar: 5 fixturas + 1 real, **0 slices na saída** (parse), exit 0; imprime também `LibreSprite can't load file` e grava na mesma. JSON sem `slices` |
| Pixelorama (`strings`, não corrido) | **Não achado** | `slice` no `.pck`: shader cleanEdge e `Smart Slice` (importar folha); nada de chunk 0x2022 nem painel. Abre `*.aseprite`, mas o que faz às slices não foi corrido |
| Godot 4.7.2 (MIT) | Fora | `godot --help`: 0 linhas com «aseprite»; saltado, como combinado |

Trial em batch (exit 0 em **todos** os 88 casos; a recusa vai para o **stdout**, não muda o exit):

```
Save operation is not supported in trial version.
Go to http://www.aseprite.org/download/ and get the full-version.
```

| Comando (`-b`) | Ficheiros | Resultado |
|---|---|---|
| `--list-slices keys.ase` | nenhum | stdout: `moves` (um nome por linha) |
| `keys.ase --list-slices` (depois do ficheiro, sem `--data`) | nenhum | stdout **vazio** |
| `--list-slices keys.ase --data d.json --sheet s.png` | `d.json` | `meta.slices` presente. Recusa a `s.png` |
| `keys.ase --list-slices --data d.json --sheet s.png` | `d.json` | `meta.slices` presente: com `--data` a ordem não importa |
| `keys.ase --data d.json --sheet s.png` | `d.json` | **Sem** `slices`: é `--list-slices` que o liga |
| `--split-slices keys.ase --data d.json --sheet s.png` | `d.json` | Folha **igual** à sem o flag: 6 células de 64×64 |
| `--slice moves keys.ase --data d.json --sheet s.png` | `d.json` | Folha **igual**: 6 células de 64×64, 384×64. Sem `slices`; `--slice nope` não dá erro |
| `--split-slices keys.ase --save-as 'x_{slice}.png'` (com e sem `-v`) | nenhum | Recusa; `-v` não imprime nomes |
| `--slice moves keys.ase --save-as x.png` | nenhum | Recusa |
| `--split-slices keys.ase --data d.json --save-as 'x_{slice}.png'` | `d.json` | 6 frames de 64×64: o JSON não descreve os ficheiros partidos |

Duas corridas completas de batch: iguais em tudo menos o tempo, com a checagem do assert (`--split-slices` e `--slice` não mudam `frames`).

## F2: ficheiros reais (parse nosso, 3 ficheiros, 54 slices)

| | `file-tests-props` | `appElements` | `icons` |
|---|---|---|---|
| Quadros, canvas, bits | 6, 64×64, 8 | 1, 512×340, 32 | 1, 512×230, 32 |
| Slices | 2 | 4 | 48 |
| Chunk mora no quadro | 0 | 0 | 0 |
| Chaves por slice, quadro | 1, 0 | 1, 0 | 1, 0 |
| Flags | 0: 1 · 1: 1 | 0: 4 | 0: 48 |
| User data logo a seguir | 2/2 | 4/4 | 48/48 |
| Com cor | 2/2 (`#0000ffff`) | 4/4 (`#e900ffff`) | 48/48 (`#0000ffff`) |
| Texto | 0 | 0 | 0 |
| Propriedades | 1 (flags 6) | 0 | 0 |
| Nomes | `Slice N`: 2 | próprios: 4 | próprios: 48, sem repetir |
| Ordem no quadro | camadas → slice+user… → cels | igual | igual |

- **Chave única no quadro 0 em todos:** 0 de 54 com chave móvel, escondida ou pivot. A amostra não exercita chaves; isso vem só das nossas fixturas.
- **Cor sempre presente nos reais; o reader não pode contar com ela** (ver F3, sem cor).
- Tags e tileset, quando há, ficam antes das camadas. O slice nunca é a primeira coisa no quadro.
- **Trial lê o mesmo:** `--list-slices --data` igual ao parse nos 3 ficheiros (assert em bounds e center, por slice). Propriedades do diálogo em `file-tests-props`: Slice 1 = 32,32,16,16; Slice 2 = 16,0,16,16 com 9-Slices 4,4,8,8 (`handoffs/P11_f2_real_propriedades.png`).
- Propriedades no JSON real: `"properties": {"a": "[unsupported type]", "b": "[unsupported type]", "ext": {"a": "[unsupported type]"}}`, entre `color` e `keys`.

## F3: o Trial a ler as nossas fixturas

Fotos a 8 px de tela por pixel, cada quadro. JSON = `--list-slices --data`. Foto: `handoffs/P11_f3_quadros.png`. Duas corridas de fotos: 0 píxeis diferentes no editor em `c_late`, `c_hidden`, `c_tarde`, `c_weird`, `c_zeros`, `full` (6 quadros), `c_zero2` (Q0, Q3) e em 7 diálogos e listas comparados. `keys` em cor e o `late` etc. sem cor: o conteúdo foi lido das fotos, sem comparar as duas corridas.

| Fixtura | Tela e diálogo | JSON |
|---|---|---|
| `keys` | Q0–1: 8,8,16,16 · Q2–3: 24,8,16,16 · Q4–5: 24,24,32,16. **A chave vale até à seguinte** | Igual |
| `late` (1.ª chave no Q2) | **Desenhada também em Q0–1**, com a caixa da 1.ª chave; o diálogo em Q0 mostra 8,8,16,16 | `frame: 2`, não muda |
| `hidden` (w=h=0 em Q2) | Q0–1: 8,8,16,16 · **Q2–3 nada** · Q4–5: 32,32,16,16 | Chave `0,0,0,0` fica |
| `tarde` (chunk no quadro 3) | **Lido.** Q0–2: 8,8 · Q3–5: 32,8 | Chaves 0 e 3 |
| `weird.out` (−8,−8,24,24) | Abre, sem erro. Diálogo −8,−8,24,24; contorno **cortado no canvas** | Igual, sem clamp |
| `weird.past` (56,56,24,24) | Contorno cortado no canvas | Igual |
| `weird.twin` ×2 | Os dois existem e aparecem nas listas | Os dois |
| `weird.badcenter` (center 12,12,20,20 em caixa 16×16) | Diálogo mostra os valores; só desenha as linhas dentro da caixa | Igual, sem clamp |
| `weird.badpivot` (−5,99) | Diálogo mostra; marca fora do canvas, não se vê | Igual, sem clamp |
| `weird.farframe` (chave no quadro 99 de 6) | **Desenhada nos 6 quadros**, mesma lei do `late` | `frame: 99` fica |
| `weird.unsorted` (chaves 3, 1 no ficheiro) | Q0–2: chave do 1 · Q3–5: chave do 3 | **Ordenada:** 1, 3 |
| `c_zero2.w0_only`, `h0_only` | **Só w=0 ou só h=0 já esconde**: nenhuma linha | Fica como está |
| `c_zero2.w0_later` | Q0–2 visível, Q3 em diante nada | Fica |
| `c_zero2.one_px` | Quadradinho de 1 px | Fica |
| `c_zero2.center_zero` (center w=h=0, flag 1) | Sem linhas interiores | **Sai sem `center`** |
| `realorder.pivonly`, `cen0` (pivot 0,0; center 0,0,16,16) | Aberto | **Ficam** |
| `zeros.no_keys` (0 chaves) | Abre sem erro | `{ "name": "no_keys" }`, sem `keys` |
| `full` | Diálogo por chave: `both` em Q0 = 4,36,24,20 / 4,4,16,12 / 0,0; em Q3 = 36,36,16,20 / 2,6,12,8 / 8,10 | Igual |

Diálogos em `handoffs/P11_f3_propriedades.png`. `weird` não deu erro, aviso nem clamp.

**Sem cor não se desenha.** Slice sem user data (ou só com texto) **não tem contorno**: `late`, `hidden`, `tarde`, `weird`, `zeros` e `ud.text_only/no_ud` ficam invisíveis no canvas, e o JSON sai sem `color`. `--list-slices` continua a nomeá-las. Foi preciso gerar cópias `c_*` com cor para medir o resto.

**Escondida não some das listas.** `c_hidden` em Q2: a barra lista `blinks`, e `All` seleciona-a (o nome vai para a caixa), sem nada no canvas. `c_late` em Q0: `All` seleciona e desenha a caixa.

**Como o Trial desenha** (`handoffs/P11_f3_desenho.png`):
- Bounds: contorno opaco na cor da slice.
- 9-Slices: quatro linhas interiores, mais claras que o contorno.
- Pivot: quadradinho de 1 pixel em (bounds + pivot). `pivot` de 12,24 cai em (48,28), logo abaixo da caixa; `both` 0,0 cai no canto.
- Cor com alfa (`#ff800080`): contorno e linhas mais pálidos.

**Ordem dos chunks.** Nossa: cel → slice → user (fixturas base); real: camadas → slice → user → cel (`realorder`). As duas são lidas, com a cor. User data logo depois da slice: aceite, com cor e/ou texto.

**Ordem das listas.**

| Onde | Ordem |
|---|---|
| `meta.slices` e stdout de `--list-slices` | **Inversa da do ficheiro** (assert em 18 fixturas + 3 reais) |
| Lista da barra da ferramenta e `Area:` do Export File | **Por nome** (`badcenter … unsorted`; `twin` duas vezes). Maiúsculas não medidas |

## F4: diálogos de exportação

**Export Sprite Sheet (Ctrl+E). ⛔ Não há «Split Slices», nem seletor de slice, nem corte: o único item é `Meta: Slices`.** Fotos `handoffs/P11_f4_abas.png`, `P11_f4_listas.png`, `P11_f4_saida.png`.

| Aba | Conteúdo |
|---|---|
| Layout | Sheet Type (By Rows), Constraints (None), Merge Duplicates, Ignore Empty, Size=x² |
| Sprite | Source: Sprite · Sprite Grid · Tilesets; Layers (Visible layers, Selected layers, Layer: *nome*) + Split Layers; Frames (All frames, Selected frames, Tag: *nome*) + Split Tags |
| Borders | Border Padding, Spacing, Inner Padding, Trim Sprite, Trim Cels, Extrude |
| Output | Output File (off), JSON Data (off). Com JSON Data: Hash, **Meta: ☑ Layers ☑ Tags ☑ Slices**, Item Filename `{title} {frame}.{extension}`, Item Tagname `{tag}` |

Estados: `Meta:` só existe com JSON Data ligado; os três começam ligados (HOME isolado). Export fica cinzento com Output File e JSON Data desligados.

| Ação | Resultado |
|---|---|
| JSON Data + Export | **Escreve `<nome>.json`** ao lado do `.ase`; barra: `Sprite Sheet Generated` |
| Meta: Slices ligado | `meta`: …, `frameTags`, `layers`, `slices` |
| Meta: Slices desligado | Mesma coisa **sem a chave** `slices` |
| Output File + Export | Barra: `Sprite Sheet Generated`, **nenhum ficheiro**, sem aviso |

**Preview com chaves que mexem:** folha inteira, 384×64 em `keys`; o preview não corta por slice. Nada a ver com Split Slices, que não existe aqui.

**Export File (Ctrl+Alt+Shift+S).** Sem caixa de partir; **o corte é `Area:`**, ao lado de Resize. `Canvas · Selection · Slice: <nome>…` por ordem de nome, com gémeos repetidos. Default Canvas. Foto `handoffs/P11_f4_area.png`. Com `x.png`, Slice: both e 6 quadros: aviso `Do you want to export the animation in 6 files? x1.png, x2.png...`; Agree → janela Console com a recusa. Foto `handoffs/P11_f4_recusa.png`. Sem ficheiro. Aba passa a marcar modificado depois disso (visto 1×, sem medir o porquê).

**CLI, combinações com `--split-slices`** (`keys`/`tags`/`transp`, células e tamanho do JSON; o corte não se vê porque a folha é sempre a mesma com ou sem o flag):

| Flags | Células | Nomes |
|---|---|---|
| nenhum (hash) | 6 | `keys 0.ase` … `keys 5.ase` |
| quadro único | 1 | `ud.ase` |
| `--split-tags` | 6, 192×128 | `tags #idle 0.ase` … `tags #walk 2.ase` |
| `--split-layers` | 12, 384×128 | `tags (bottom) 0.ase` … `tags (top) 5.ase` |
| ambos | 12, 192×256 | `tags (bottom) #idle 0.ase` … |
| `--tag idle` | 3 | `tags #idle 0.ase` … |
| `--frame-range 1,3` | 3 | `keys 0.ase` … `keys 2.ase` |
| `--split-grid` | 96 | 16×16, `keys 0.ase` repetido por célula |
| `--trim` | 3, 13×8 | Células 8×8, 4×4; quadro vazio 1×1 |
| `--trim-sprite` | 3, 108×36 | Células 36×36 |
| `--ignore-empty` | 2 | |

`{slice}` em `--filename-format` com `--sheet`: **vazio**, com ou sem `--split-slices` (`tags||0|idle||ase`).

**`meta.slices` não acompanha a folha.** `--frame-range`, `--tag`, `--split-tags`, `--split-layers`, `--trim`, `--sheet-pack`: as chaves ficam em quadros absolutos, sem filtrar nem renumerar, e as bounds em coordenadas do sprite. Em `--frame-range 1,3` as células são 0–2 e as chaves 0, 2, 4. Dois ficheiros: `moves` e `late` juntos, sem dizer de qual. `--scale 2` antes do ficheiro: não mudou nada (`scale: "1"`); depois do ficheiro não foi medido.

## F5: JSON e nomes

**JSON medido** (Trial, `keys`, `full`, `ud`, `realorder`):

```json
"slices": [
 { "name": "both", "color": "#ff800080", "keys": [{ "frame": 0, "bounds": {"x": 4, "y": 36, "w": 24, "h": 20 }, "center": {"x": 4, "y": 4, "w": 16, "h": 12 }, "pivot": {"x": 0, "y": 0 } }, … ] },
 { "name": "color_text", "color": "#0a141e28", "data": "hello", "keys": […] },
 { "name": "quo\"te\\", "data": "a\"b\\c", "keys": […] },
 { "name": "açã ο/x", "color": "#010203ff", "keys": […] },
 { "name": "", "color": "#040506ff", "keys": […] }
]
```

| Campo | Lei medida |
|---|---|
| Ordem das chaves | `name`, `color`, `data`, `keys`; com propriedades, `properties` depois de `color` (com `data` junto: não medido) |
| `color` | `#rrggbbaa`, minúsculas. Sem cor: a chave não sai |
| `data` | O texto do user data. Sem texto: a chave não sai. Escapado como JSON; UTF-8 cru |
| `keys[]` | `frame`, `bounds{x,y,w,h}`, `center{x,y,w,h}`, `pivot{x,y}`, ordenadas por `frame` |
| `center` | Só com a flag **e** w, h ≠ 0 (`center_zero`). Pontos negativos e 0 mantêm-se |
| `pivot` | Só com a flag. `0,0` sai |
| `slices` vazio | `"slices": [\n  ]`: a chave existe com `--list-slices` |
| Com `--list-slices`, `--list-tags`, `--list-layers` | `meta`: `version`, `image`, `format`, `size`, `scale`, `frameTags`, `layers`, `slices` |

**Documentado, não corrido:**

- `aseprite.org/docs/cli`, `--list-slices`: *«When used with --data, slices will be available in the JSON output in the meta attribute.»* (o medido confirma). Exemplo publicado, mesma ordem: `{ "name": "cursor", "color": "#0000ffff", "keys": [{ "frame": 0, "bounds": {"x": 80, "y": 0, "w": 16, "h": 16 }, "center": {"x": 2, "y": 2, "w": 12, "h": 12 }, "pivot": {"x": 8, "y": 8 } }] }`.
- `--split-slices`: *«Since v1.2-beta8, splits next document slices into different files. It affects the --save-as option. Same as doing: aseprite.exe -b sheet.ase --save-as part-{slice}.png»*. Nomes de `--split-slices` sem `{slice}` no `--save-as`: não documentados nem medidos.
- `--slice`: *«Exports only the area specified by the given slice. It affects --save-as option.»*
- `--filename-format`: lista `{fullname} {path} {name} {title} {extension} {layer} {tag} {innertag} {outertag} {frame} {tagframe} {duration}`; **`{slice}` não está nessa lista**.
- `aseprite.org/docs/slices`: exemplo `Button-patch` (name, color, keys, frame, bounds, center); Slice Properties por duplo clique; atalho Shift+C; Delete apaga.
- `aseprite.org/docs/sprite-sheet`: não fala de slices.
- `docs/ase-file-specs.md` (spec pública), chunk 0x2022: *«Frame number (this slice is valid from this frame to the end of the animation)»*; *«Slice width (can be 0 if this slice hidden in the animation from the given frame)»*; flags `1 = It's a 9-patches slice`, `2 = Has pivot information`; center *«relative to slice bounds»*; pivot *«relative to the slice origin»*. A spec fala só da **largura** a 0; medido: altura a 0 também esconde. O que acontece antes da 1.ª chave a spec não diz; medido em F3.
- User data 0x2020: *«Specifies the user data (color/text/properties) to be associated with the last read chunk/object.»* Slices não são caso especial na lista; a regra do «último lido» é a medida.

**Amostra real pública com pivot e `data`** ([fórum, versão no JSON `1.3.17.2-x64`](https://community.aseprite.org/t/what-exactly-does-the-data-in-exported-json-represent/28435)), documentado, não corrido; mesma ordem e chaves repetidas por nome com `data` diferente:

```json
{ "name": "wizard_up", "color": "#0000ffff", "data": "#attack", "keys": [{ "frame": 0, "bounds": { "x": 12, "y": 32, "w": 8, "h": 8 }, "pivot": { "x": 3, "y": 3 } }] }
```

**Issues públicas, documentado, não corrido:**
- [aseprite#2469](https://github.com/aseprite/aseprite/issues/2469) (aberta): *«--slice argument ignored with --sheet option in CLI»*. O batch reproduz: folha igual.
- [aseprite#2283](https://github.com/aseprite/aseprite/issues/2283): slice fantasma de bounds 0,0,0,0 faz `--split-slices` *«export no output at all»*. O nosso batch não o pode confirmar: `--save-as` recusa. Quem responde diz que *«slices do not work in animations»* é problema conhecido; sem data nem versão confirmadas aqui.

## Não medido

- **Píxeis de qualquer exportação:** ficheiros de `--save-as` + `--split-slices`/`--slice`/`{slice}`; Export File com Area: Slice. Logo: se cada quadro é cortado na chave desse quadro; se quadros antes da 1.ª chave ou escondidos saem; clamp de fora do canvas; nomes reais dos ficheiros partidos.
- `--script` (Lua): não corrido, fora do autorizado.
- Slice em ficheiro indexado ou cinza; `0x2020` não colado à slice; `data` + `properties` juntos; center com só w=0 ou só h=0.
- Slices sem cor nas listas da barra e de `Area:`; ordem de listas com maiúsculas; botão de user data do diálogo.
- Propriedades de `Slice 1`: valores vistos, botão não aberto.
- `--scale` depois do ficheiro; `--sheet-type columns`, `--sheet-width`, com JSON (`--sheet-pack` e `--tag` só em `meta`).
- LibreSprite pela interface; Pixelorama corrido; importador de Aseprite em Godot (plugin de terceiros).
- Ficheiro real com chaves múltiplas, pivot ou chave escondida: nenhum dos 3 tem.

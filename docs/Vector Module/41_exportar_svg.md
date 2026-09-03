# Plano 41 — **EXPORTAR SVG** (o desenho sai do app)

> Pedido do Enio (2026-09-02): *"ainda temos inconsistências mas antes de tentar resolver
> precisamos de um meio de exportar o path para que vc possa analisar melhor. Veja se já há
> exportação no app"*.

## §1 — O censo: o que o app exportava, e o que não

Havia **onze** acções de exportação alcançáveis, e **nenhuma** levava uma curva:

| o quê | escreve |
|---|---|
| `Save` / `Save As…` | `.ph2dproj` (postcard binário) |
| `Export Image…` (Hierarquia) | 16 formatos raster |
| `Export Sheet` (Hierarquia) | `.png` + `.json` de atlas |
| model3d `Export` (Draft/Fine/Max) · Sculpt `Ctrl+Shift+E` | OBJ / PLY / STL |
| `Export DTCG…` | `.tokens.json` |
| Palette Export | `.gpl` / `.hex` / `.ase` / `.aco` |
| Áudio (`Export`, `Export Pieces`, `Export Set`, presets, Batch LUFS) | WAV / Ogg / Opus / `.txt` |

⛔ **PDF, EPS, AI: zero.** ⛔ **JSON de caminhos: zero** — o `ph2d-vec-scene` só serializa por
postcard. ⛔ O único despejo de geometria era o `PH2D_FX_DUMP`, que escreve **segmentos achatados**
(`seg x0 y0 x1 y1`), não um caminho.

⚠️ **E havia uma promessa por cumprir:** `ExportFormat::Svg` já era oferecido no diálogo do *Export
Image…* e o exportador era um **stub que sempre erra** (*"deferred to W3+"*). ⭐ Ele **fica** — a
lista inclui de propósito formatos que recusam, *"porque a recusa vem do exportador, com a razão
dele"* (a cerca de Chesterton está escrita no `image_export.rs`) — mas a razão passou a ser a
verdadeira: **uma imagem não tem curva nenhuma para escrever**, e a mensagem agora **nomeia** o
*File > Export SVG…*.

## §2 — A lei

> **O que sai é o que se VÊ: a geometria COZIDA, no MUNDO.**

- **Cozida** — a pilha de Live Path Effects e o raio de quina já correram. Um ficheiro exportado de
  uma forma com efeitos abre igual ao que está no ecrã.
- **No mundo** — a pose de cada objecto é **assada na geometria**. ⛔ Não há `transform` por
  elemento, que discordaria da régua do editor e do que qualquer sonda mede.
- ⚠️⚠️ **O `d` sai da MESMA porta que o renderer usa** (`ph2d_vec_render::build_contours`, que
  deixou de ser `pub(crate)` para isso). Uma segunda travessia dos contornos daria um ficheiro que
  discorda do ecrã em curvas que nenhum olho apanha.
- ⭐⭐ **DOIS elementos por forma, e é a lei do renderer**: o preenchimento leva só os contornos
  **FECHADOS** (`build_fill_bezpath`), o traço leva **todos**. Um elemento só fecharia
  implicitamente cada contorno aberto e abriria regiões que o app não pinta — e uma **rede soldada
  é feita exactamente de contornos abertos**.

### §2.1 — As marcas que só quem analisa lê

Cada `<path>` leva `data-ph2d-id` (o id do caminho no documento) e, quando é área de balde,
`data-ph2d-fill="1"`. ⚠️ É o que deixa separar a **LINHA** da **TINTA** sem adivinhar pela cor — e a
pergunta é a MESMA que o balde faz (`VecViewState::is_derived`, populado do `VecBucketFill`).

### §2.2 — ⛔ O que ele não carrega, e DIZ

*Um exportador que ignora em silêncio é pior do que um que recusa* (a lei do importador `.ase`). Um
**padrão de textura**, um **pincel de contorno** e um **gradiente multi-ponto** não têm equivalente
em SVG 1.1: saem com a **mesma cor de recurso que o renderer usa** quando o ladrilho não resolve
(`Paint::primary_color` / `PatternFill::fallback`) — ⛔ nunca uma cor inventada no exportador —, e o
**cabeçalho do ficheiro nomeia** cada forma em que isso aconteceu.

Gradientes **linear** e **radial** viajam como gradientes de verdade (`<linearGradient>` /
`<radialGradient>` em `userSpaceOnUse`), porque as coordenadas deles vivem no mesmo espaço da
geometria e são assadas com ela.

## §3 — ⛔⛔ O gate que estava VERDE sobre um menu que tinha crescido

Os dois gates do menu Ficheiro — `each_file_menu_item_raises_its_own_flag` e
`every_file_menu_flag_is_drained` — são **listas literais de três**. O item novo passou por eles
**sem os acordar**: eles ficaram verdes afirmando sobre três itens de um menu com quatro.

⇒ além de acrescentar a quarta entrada em cada, entrou o
`no_file_menu_row_is_left_out_of_the_census`, que **lê as linhas do menu** (a fonte que o artista
vê) e exige que cada uma levante alguma bandeira. *Assim o próximo item reprova em vez de passar
despercebido, mesmo que ninguém se lembre de editar a lista literal.*

## §4 — ⏳ Aberto

- ⏳ **Importar SVG** continua a devolver um documento vazio (`ph2d-imageio-svg` faz o parse com
  `usvg` e deita fora o resultado). A exportação não o cura.
- ⏳ **Marcadores** (setas) e **efeitos que não estão no cozido** não têm elemento próprio: o que
  sai é a geometria da forma.
- ⏳ **Uma forma dentro de um grupo com `layout_pose`** sai na pose que o `vec_transform` publica —
  o mesmo que o ecrã mostra; ⛔ não foi medida contra auto layout.

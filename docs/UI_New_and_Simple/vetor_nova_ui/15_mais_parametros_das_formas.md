# 15 — Mais parâmetros, o cartão lateral e as alças no modo Forma (F10, 11/10)

> Pedido do dono (11/10, depois da F9): *«Não podemos reduzir parâmetros de configuração das formas.
> Pelo contrário. Precisamos de mais parâmetros. Se algum parâmetro não pode ser colocado no gizmo
> então precisamos criar um modal que aparece na lateral.»* · *«Ao criar uma forma, os controles devem
> aparecer imediatamente mesmo fora do modo select, ainda no modo shape. No modo select os controles
> não devem aparecer.»* · *«No modo shape ao clicar nos controles ou gizmo esses têm a preferência em
> relação a criar nova forma.»* Prompt: [`14_prompt_F10_…`](14_prompt_F10_mais_parametros_e_alcas_no_modo_forma.md).
> A F9 (a lei das alças, os oráculos dela): [`13_alcas_das_formas.md`](13_alcas_das_formas.md).

Código: a geometria nova em `ph2d_vec_scene::{polystar, corner_style}` (+ `gear_ex`, `trapezoid_asym`,
`diamond_mid`); os campos em `ph2d_tool_vector::{shapes, shapes_fields}`; as alças novas em
`shape_handles{,_table}` (`Spot::EdgeMid`, `Law::Polar`, `inset`); o cartão em
`ph2d_editor_core::screens::hero::tool_bars_card` + `ph2d_panel_vector::bars_row::card_chips`; as
portas do modo Forma em `ph2d_app_vec::{overlay::object_box, shape_handles_live::Gate}`.

## §1 — A tabela forma × parâmetro

`✓` temos · `+` entrou na F10 · `—` o app não tem · `?` o manual não confirma. Affinity pelo MANUAL
(app novo `affinity.studio/help/tools-tools-<forma>` e Designer 2 `affinity.help/designer2/…/Tools/`);
Illustrator pelo helpx (Live Shapes / Live Corners); Inkscape CORRIDO (§2) e o manual.

| forma | parâmetro | nós | Affinity | Illustrator | Inkscape |
|---|---|---|---|---|---|
| Rect arredondado | raio (mestre) + desvio por canto | ✓ | Corner % + *Single radius* | Corners / Radius (por canto) | Rx/Ry |
| | suavização (squircle) | ✓ | — | — | — |
| | **tipo do canto** | **+** | Rounded/Straight/Concave/Cutout | Round/Inverted/Chamfer | — |
| | *Absolute sizes* (raio fixo ao escalar) | — | ✓ | *Scale Corners* | — |
| Polígono | lados · raio do canto | ✓ | Sides | Sides · corner radius | Corners |
| | **tipo do canto** | **+** | — | Live Corners | — |
| | **envergadura** (*Curve*) | **+** | Curve % (± côncavo) | — | — |
| | **liso** (*Smooth points* / *Rounded*) | **+** | Smooth points (bool) | — | Rounded |
| Estrela | pontas · interior · raio da ponta · raio do vale | ✓ | Points · Inner radius · Outer/Inner circle | Side count · Ratio · corner | Corners · Spoke ratio |
| | **tipo do canto** | **+** | — | Corner types | — |
| | **envergadura** | **+** | Curved edges | — | — |
| | **liso** | **+** | — | — | Rounded |
| | **torção** | **+** | — | — | `arg2` (a alça do vale) |
| Elipse / pizza / arco | abertura · início · furo | ✓ | Start/End/Total · Hole | Pie start/end | Start/End · arc type |
| | *Invert* / *Close pie* | — | botões | *Invert Pie* | *Make whole* |
| Espiral | voltas | ✓ | Turns | Segments | Turns |
| | **divergência** (*Expansion*) | **+** | Decay % | Decay | Divergence |
| | **raio interno** | **+** | Inner radius | — | Inner radius |
| | sentido (horário) | — | ✓ | Style | — |
| Engrenagem | dentes · profundidade · furo | ✓ | Teeth · Inner radius · Hole | — | — |
| | **largura do dente** | **+** | Tooth size | — | — |
| | **envergadura dos flancos** | **+** | Curvature | — | — |
| | entalhe (*Notch size*) | — | ✓ | — | — |
| Trapézio | recuo | ✓ | Left/Right point | — | — |
| | **lado direito solto** (Alt) | **+** | ✓ (dois campos) | — | — |
| Losango | — | — | Mid point | — | — |
| | **meio** (a pipa) | **+** | ✓ | — | — |
| Seta | haste · cabeça (comp./larg.) | ✓ | Thickness · Length · Size · Ends · Style | — | — |
| Balão rect./oval | raio · bico 2D · base · faixa | ✓ | Radius · Tail position/end/height/width | — | — |
| Nuvem | bolhas · fofo · vale · irregular | ✓ | Bubbles · Inner radius | — | — |
| Coração | entalhe | ✓ | Spread | — | — |

## §2 — Os oráculos

### O Inkscape 1.4.4 — CORRIDO (`oraculo_parametros/`)

Triagem por artefacto (`pacman -Qi inkscape`): GPL/LGPL ⇒ caixa preta, só a saída. Sem rato desta
vez: `corre_inkscape_params.py` escreve as formas NOSSAS num SVG com os atributos do parâmetro e o
Inkscape cozinha-as pela linha de comando (`object-to-path` + `export-plain-svg`) — 31 formas, 4
fixtures com cabeçalho. Os gates (`shape_handles_f10_tests.rs`) passam pelo catálogo inteiro
(`ph2d_vec_scene::cook`).

| lei medida | número |
|---|---|
| o *Rounded* = vértice LISO, alças de `rounded × a corda` da aresta | 12 estrelas/polígonos (−0,25…1) a Hausdorff `< 1e-6` |
| a direcção da alça = vizinho ANTERIOR → SEGUINTE (não a perpendicular ao raio) | a estrela torcida 15°: a alça da ponta a `15,015°` (a perpendicular daria 0°); sem torção coincidem por simetria |
| o resíduo da torção com liso | `1,8e-5` da forma (`0,002` px num raio de 100): a alça dele aponta `0,015°` ao lado — tolerância `5e-5` |
| a torção = o `arg2` fora do meio | 2 estrelas a `< 1e-6` |
| a espiral: `r = t^divergência`, `θ = voltas·2π·t`, `t ∈ [t0, 1]` | as ÂNCORAS dele caem na lei a `≤ 9e-7` (12 espirais) |
| o ajuste dele (4 cúbicas por volta) | afasta-se até `0,006` da lei — a régua é a LEI, não as cúbicas dele |
| a nossa espiral divergente (16 por volta; abaixo de 1 pelo RAIO) | `< 1e-3` da lei (a `0,5` sem a reparametrização: `0,0135`) |
| o vale torce com a mão (o arrasto livre do vale da F9) | 12 arrastos: a torção = `Δarg2` a `< 0,3°`; Ctrl = só o raio (`Δ = 0`) |

### O Affinity — o MANUAL (o produto de referência do dono)

Sem build para Linux. Lidas as páginas de cada ferramenta de forma (o app novo e o Designer 2 — o
texto é o mesmo): a tabela do §1. O que daqui se adoptou: os quatro tipos de canto e os nomes deles
(*Rounded · Straight · Concave · Cutout*); *Curve* / *Curved edges* (a envergadura); *Decay* + *Inner
radius* da espiral; *Tooth size* + *Curvature* do Cog; *Left/Right point* do trapézio; *Mid point* do
losango.

## §3 — O que entrou (todos APENSOS e NEUTROS em zero)

Um save anterior tem zeros nos campos novos e cozinha byte a byte o de antes — gate
`the_f10_fields_are_neutral_at_zero_and_each_one_moves_the_shape` (e cada campo novo MEXE a forma:
nenhum controlo morto). Nenhuma forma passa de 8 campos: o tecto `MAX_SHAPE_FIELDS` /
`MAX_SHAPE_VALUES` (o documento GUARDA `[f64; 8]`) não mudou.

| forma | campo novo (índice) | faixa | gesto |
|---|---|---|---|
| Rect arredondado | Corner (5) | Rounded · Straight · Concave · Cutout | **cartão** |
| Polígono | Corner (2) · Curve (3) · Smooth (4) | escolha · −1…1 · −1…1 | cartão · **meio da 2.ª aresta** · cartão |
| Estrela | Corner (4) · Curve (5) · Smooth (6) · Twist (7) | escolha · −1…1 · −1…1 · ±90° | cartão · **meio da 3.ª aresta** · cartão · **o vale (polar)** |
| Espiral | Expansion (1) · Inner (2) | −0,9…9 · 0…0,9 | cartão · **a ponta de dentro** |
| Engrenagem | Tooth (3) · Curvature (4) | −0,35…0,4 · −0,5…0,5 | **o fim do topo do 1.º dente** · cartão |
| Trapézio (e o invertido) | Right (1) | −0,49…0,49 | **Alt no canto direito** (Alt no esquerdo devolve a diferença) |
| Losango | Mid (0) | −0,45…0,45 | **o vértice da direita, desenhado para dentro** |

- ⭐ **O tipo do canto** (`CornerStyle`): o corte é o MESMO nos quatro (o recuo sai do raio pela conta de
  sempre) — trocar o tipo só troca a ligação: o arco, a recta, o arco CENTRADO na quina, o entalhe
  (`p_in + p_out − quina`). Sobre arestas curvas passa pelo motor das quinas vivas (`corner_live`).
- ⭐ **O vale da estrela é POLAR** (`Law::Polar`): o raio pela métrica do F9 e a torção pelo ângulo
  desenrolado — o arrasto livre do Inkscape; **Ctrl = só o raio** (o Ctrl dele).
- **A envergadura** tem alça no MEIO de uma aresta (`Spot::EdgeMid`, `t = ½` da cúbica — o meio da
  corda não está no contorno). A 1.ª aresta é a do arredondamento (satura no meio dela): a alça vai à
  2.ª (polígono) / 3.ª (estrela).
- **A alça do canto sobre uma aresta CURVA**: o `Spot::Rounding` procura o corte na cúbica e não na
  corda (gate `the_corner_handle_still_rides_a_curved_edge`).
- **O meio do losango** vive SEMPRE sobre a aresta da caixa (onde mora a alça de escala): desenha-se
  `PARK_PX` para dentro (`inset`), a pega mede-se ao ponto verdadeiro.

## §4 — O cartão lateral

Os campos que NENHUMA alça move (`shape_handles::fields_without_handle` — a conta, nunca uma lista)
num cartão flutuante JUNTO da forma, enquanto as alças estão à vista (modo Forma + forma viva) — some
com elas. Os MESMOS ids da fila ② (`vector_shape_field_id/_choice_id`): um comando, um id (o
`HitIndex` aceita o id em dois rects; o gate de ids duplicados declara o cartão).

- **A fila ② continua com TODOS os números** (decisão: o dono não mandou tirá-los).
- **A posição** (proposta — pergunta ao dono): à DIREITA da forma, alinhado ao topo dela, com folga
  `Spacing::Sm`; espelha à esquerda quando não cabe; desvia da ④, da coluna ① e do flyout; nunca cobre a
  forma (`placed_card`, 7 gates de geometria).
- Hoje no cartão: o Corner e o Smooth (polígono/estrela), o Corner e a suavização (rect), a Expansion
  (espiral), a Curvature (engrenagem) e os da F9 (bolhas, jitter, Viewed…).

## §5 — As alças no modo Forma (a inversão da F9)

- O plano do overlay: `shape_handles` no **Shape**, nunca no Select. O alvo das alças é a MESMA porta
  dos campos (`shape_field_target` com o latch): armado no catálogo ⇒ sem alças; a forma acabada de
  desenhar (seleccionada, latch apagado) ⇒ alças NO MESMO QUADRO.
- A caixa (gizmo) passa a existir no modo Forma **só com as alças** (`ObjectBox::HandlesOnly`: sem o
  miolo nem o pivô) — as alças dela tiram o `on_canvas`, o miolo fica canvas: desenhar uma forma por
  cima/dentro de outra continua (o Affinity). Revê o ADR-0112 («uma caixa sobre um canvas de autoria é
  um ladrão de cliques») por ordem do dono: ela rouba exactamente o clique que ele quer que ela ganhe.
- A prioridade do press: alça da forma (mesmo sobre uma da caixa) › alça da caixa › desenhar.
- **Os nós da forma somem com as alças à vista** (`hides_the_nodes`): a 1.ª foto da cena 16 mostrou
  as âncoras laranja e as alças bézier azuis a tapar as verdes — o idioma do envelope.
- A dica ⑤: `panel.vector.hint.shape_handles` (…«Empty canvas: draw another»).

## §6 — ⛔ Recusas MEDIDAS

| proposta | porque não |
|---|---|
| o *Randomized* do Inkscape | o acaso dele não se reproduz (uma corrida = uma semente dele), e a composição já o exprime: os efeitos `fx_sketch`/`fx_zigzag` desordenam QUALQUER forma, com semente nossa |
| o tipo do canto POR CANTO (o *Single radius* desligado do Affinity) | +3 campos no rect (6 → 9) passa o tecto de 8, que é do DOCUMENTO (`VecShape::Param.values: [f64; 8]` por serde — subir exige migrar saves e o `PROJECT_SCHEMA`); um tipo para os quatro cobre o Illustrator e o Affinity com *Single radius* |
| o tipo de arco do Inkscape (fatia · arco · corda) como escolha | já são TRÊS formas (`Pie`, `Arc`, `Segment`) — e a forma aberta/fechada é do `kind` (`is_closed`), não de um valor |
| o sentido da espiral (horário) | é o espelho horizontal da forma — a composição já o exprime |
| a perpendicular ao raio como direcção do liso (a 1.ª redacção) | o Inkscape torcido mediu `15,015°`; a perpendicular dava 0° (`0,025` da forma) |
| comparar a espiral com as CÚBICAS do Inkscape | ele ajusta 4 por volta e afasta-se até `0,006` da lei; as âncoras dele são a lei a `1e-6` |
| a espiral divergente parametrizada por `t` | com divergência < 1 a derivada é infinita no centro: `0,0135` da lei; pelo raio, `< 1e-3` |
| *Invert* / *Close pie* | são BOTÕES (acções), não parâmetros — um botão novo numa barra é decisão do dono (§7) |
| *Absolute sizes* / *Scale Corners* | não é geometria: é a política de escala do raio quando a caixa muda — fica para medir com o redimensionar da caixa |
| *Notch size* do Cog | o nosso dente é trapézio de flancos rectos (o `gear6` do ECMA): o entalhe já é a profundidade + a largura do dente |

## §7 — Perguntas ao dono (no relatório)

- O cartão fica à direita da forma (espelha à esquerda na borda)? Ou noutro lado?
- Quer os botões *Inverter* e *Fechar* da pizza (o Affinity e o Illustrator têm)?
- Quer o tipo de canto diferente em cada canto? (Pede subir o limite de 8 parâmetros por forma, que é
  do ficheiro do projecto.)

## §8 — As cenas

- `PH2D_OBJECT_MODE_SMOKE=15` (refeita): as três formas, o botão **Forma** (não a Seleção), o canto, o
  vale e o fim da pizza — foto [`f10_02`](fotos/f10_02_cena15_as_alcas_no_modo_forma.png).
- `PH2D_OBJECT_MODE_SMOKE=16` (nova, `vector_mode_card`): uma estrela; as pontas arredondam, o vale
  TORCE 12°, a aresta ENVERGA, e o chip *Corner* do cartão vai a *Concave* — foto
  [`f10_01`](fotos/f10_01_cena16_a_estrela_torcida_curva_e_o_canto_concavo_pelo_cartao.png).

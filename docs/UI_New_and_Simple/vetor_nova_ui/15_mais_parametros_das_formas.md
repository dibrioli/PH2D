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

## §1b — A estrela do dono: o Affinity e o Illustrator PELAS FOTOS (a 2.ª redacção)

> O dono, depois da 1.ª redacção: *«a estrela ficou horrível. nem tem como criar uma estrela com
> isso. vá descobrir como fazem Affinity e Illustrator. veja por fotos como eles posicionam o gizmo e
> seus pontos»*. A 1.ª redacção copiou o Inkscape (o vale torcia com a mão) e espalhou cinco pontos
> pela estrela (dois quase sobrepostos no vale); a estrela nascia gorda (0,5).

Fontes: a figura da estrela de omissão do manual do Affinity (Designer 2 e o app novo — o mesmo
ficheiro, md5 igual), a captura do fórum Serif (12 pontas, barra visível), os quadros do vídeo «Star
Tool Ninja»; os GIFs do Tuts+ do Illustrator (o widget de pontas, o arrasto de R1/R2). Imagens
guardadas fora do repo; a leitura está aqui.

| | Affinity | Illustrator | nós (2.ª redacção) |
|---|---|---|---|
| os pontos | 3 vermelhos, JUNTO da ponta de cima: a ponta e os dois vales dela | R1 numa ponta, R2 num vale, ⊙ de canto, ⇕ das pontas | 4 junto da ponta de cima + o ⇕ |
| a ponta | arredonda (*Outer circle*): o ponto desce pelo eixo da ponta | R1 (raio exterior), radial | arredonda; o ponto é o cruzamento do contorno com o EIXO da ponta (`Spot::Apex`), desenhado `PARK_PX` para dentro |
| o vale da direita | o raio interior | R2 (o raio interior), radial, círculo tracejado | o raio interior, SÓ radial |
| o vale da esquerda | arredonda os vales (*Inner circle*) | ⊙ | arredonda os vales, pelo eixo do vale |
| *Curved edges* | dois pontos a meio dos lados da ponta | — | a envergadura no meio do lado esquerdo da ponta |
| as pontas | só a barra (*Points*) | ⇕ na borda direita da caixa, «Points: N» | ⇕ na borda direita da caixa VISÍVEL, 1/24 da altura por degrau (`Law::Steps`) |
| torção | não existe | não existe | só no cartão |
| omissão | 5 pontas, raio interior ≈ 38 % (os ombros alinhados) | 5, regular | 5, `cos 72°/cos 36° = 0,382` (era 0,5) |
| a caixa | aparece junto (8 alças + rotação); o ponto da ponta cai em cima da alça do meio-topo | some durante o arrasto de um raio | junto; o ponto da ponta desenha-se para dentro do topo (a caixa abraça o contorno cozido e desce com a ponta arredondada — estacionar a partir do vértice vivo deixava-o por baixo da alça azul: a foto) |

O polígono ganhou o mesmo idioma: o canto no vértice de cima (pelo eixo dele), a envergadura no meio
do lado esquerdo, os lados no ⇕.

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
| o vale torce com a mão (o arrasto livre do vale da F9) | 12 arrastos: a torção = `Δarg2` a `< 0,3°` — **medido e RECUSADO** (§1b, §6): o produto do dono não torce |

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
| Polígono | Corner (2) · Curve (3) · Smooth (4) | escolha · −1…1 · −1…1 | cartão · **meio do lado esquerdo do vértice de cima** · cartão |
| Estrela | Corner (4) · Curve (5) · Smooth (6) · Twist (7) | escolha · −1…1 · −1…1 · ±90° | cartão · **meio do lado esquerdo da ponta de cima** · cartão · cartão |
| Espiral | Expansion (1) · Inner (2) | −0,9…9 · 0…0,9 | cartão · **a ponta de dentro** |
| Engrenagem | Tooth (3) · Curvature (4) | −0,35…0,4 · −0,5…0,5 | **o fim do topo do 1.º dente** · cartão |
| Trapézio (e o invertido) | Right (1) | −0,49…0,49 | **Alt no canto direito** (Alt no esquerdo devolve a diferença) |
| Losango | Mid (0) | −0,45…0,45 | **o vértice da direita, desenhado para dentro** |

- ⭐ **O tipo do canto** (`CornerStyle`): o corte é o MESMO nos quatro (o recuo sai do raio pela conta de
  sempre) — trocar o tipo só troca a ligação: o arco, a recta, o arco CENTRADO na quina, o entalhe
  (`p_in + p_out − quina`). Sobre arestas curvas passa pelo motor das quinas vivas (`corner_live`).
- ⭐ **A estrela e o polígono do Affinity** (§1b): os pontos junto da ponta/vértice de cima; o raio
  interior SÓ radial; o ⇕ das pontas/lados; a estrela regular de omissão.
- **A envergadura** tem alça no MEIO de uma aresta (`Spot::EdgeMid`, `t = ½` da cúbica — o meio da
  corda não está no contorno).
- ⭐ **Arredondar nunca engorda**: com a envergadura E o raio do canto, a ligação lisa entre duas
  arestas curvas passava `0,067` ACIMA do vértice; agora arredonda-se o contorno recto (arcos
  exactos) e enverga-se só o troço entre cantos, com as alças do arco rodadas para a tangente (G1) —
  `the_rounding_never_grows_past_the_sharp_corner`.
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
| ⭐ **o vale que TORCE com a mão** (a `Law::Polar`, o `arg2` do Inkscape — a 1.ª redacção) | medido (12 arrastos a `< 0,3°`) e recusado: nem o Affinity nem o Illustrator rodam a estrela pelo raio interior (as fotos); o desvio da mão torcia a estrela — o dono: *«ficou horrível»*. A torção fica no cartão |
| a contagem de pontas pelo ÂNGULO de uma ponta (o *side widget* lido como ângulo, a F9) | obscuro (agarrar uma ponta e rodar); o Illustrator usa um ⇕ na borda da caixa, vertical — é o que ficou |
| estacionar o ponto da ponta a partir do vértice VIVO | a caixa abraça o contorno cozido: com a ponta arredondada o topo dela (a alça do meio-topo) desce até ao ponto — a alça verde ficava por baixo da azul (a foto da cena 16) ⇒ desenhado sempre `PARK_PX` para dentro do topo |
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
- `PH2D_OBJECT_MODE_SMOKE=16` (nova, `vector_mode_card`): a estrela regular; as pontas arredondam (o
  ponto desce pelo eixo), os vales arredondam, o ⇕ sobe dois degraus — 7 pontas; o cartão ao lado —
  foto [`f10_01`](fotos/f10_01_cena16_a_estrela_do_affinity_pontas_macias_e_sete_pontas_pelo_widget.png).
  A estrela acabada de desenhar, com os pontos junto da ponta: [`f10_03`](fotos/f10_03_a_estrela_nova_regular_e_os_pontos_junto_da_ponta.png).

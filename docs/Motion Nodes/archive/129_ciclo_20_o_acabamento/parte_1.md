# ARQUIVO — 129_ciclo_20_o_acabamento.md (história, 253 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`129_ciclo_20_o_acabamento.md`](../../129_ciclo_20_o_acabamento.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-234,488-547` do original.
>
> ⚠️ **A única alteração ao corpo:** 2 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §5.1 — Os oráculos (W0, 10/10)

`bash "docs/Motion Nodes/ferramentas/lens_oracle/corre.sh"` corre o ImageMagick e o ffmpeg sobre
a cena NOSSA (`entrada.py`, 192×128, gradiente + discos + barra branca) e a LUT NOSSA (`cubo.py`,
9³, mistura entre canais) e grava 11 fixtures com cabeçalho em `crates/ph2d-lens/tests/fixtures/`.
Os gates (`ph2d-lens/tests/oraculos.rs`) — cada um com um CONTROLO que tem de reprovar:

| lei | oráculo | erro medido | controlo (reprova) |
|---|---|---|---|
| vinheta = superelipse **convolvida** com a gaussiana (integral 1D por pixel, 16 nós) | `magick -vignette 0xσ+ix+iy`, 4 configurações | rms `0,008`–`0,020`, máx `0,015`–`0,063` | `Φ(distância/σ)`: máx `0,23` na borda larga |
| aberração = canal escalado do centro (radial) / deslocado (linear), bilinear, borda replicada | `magick -separate … -distort SRT` por canal | **`8,5e-6`** (o bit de 16) | o sentido trocado: `> 0,05` |
| LUT `.cube` tetraédrica · trilinear | `ffmpeg lut3d interp=…` | **`1,6e-5`** nas duas | trocadas: `> 1e-3` (medido `0,0036`) |
| blur gaussiano em luz linear (para a W3) | `magick -colorspace RGB -gaussian-blur 0x3` | gravado, por gatear (W3) | — |

O que os oráculos ensinaram (e o que NÃO era a lei):
- a vinheta do ImageMagick é uma elipse RASTERIZADA e depois borrada; o resto do erro (pior com
  `σ = 4`) é a rasterização dele, não a lei. O centro do `-vignette` é o pixel de ÍNDICE `(W/2,
  H/2)`; o do `-distort` é contínuo (`x + 0,5`, a convenção da placa) — dois programas, duas
  convenções, e o gate traduz na fronteira;
- ⚠️ **o ffmpeg a ler um PNG desvia `0,0046`** (a conversão de formato dele), e acusaria a lei por
  um erro do leitor: as entradas viajam em BRUTO de 16 bits (`rgb48be`);
- sem oráculo instalado (gateados por propriedade, `tests/leis.rs`): o grão (aleatório: média `0`,
  variância `1`, o MESMO por tique, mexe com o tique), o balanço de branco (colorimetria publicada:
  neutro = identidade EXACTA, quente puxa para o vermelho, magenta tira verde), a exposição e o
  contraste (o cinzento médio fica).

### §5.2 — A placa contra a CPU (W1, 10/10)

`LensPass` (`ph2d-render/src/lens.rs` + `shaders/lens_lib.wgsl`/`lens.wgsl`) contra
`ph2d_lens::aplica_cpu`, num sub-rect `96×64` em `(20, 9)` de um alvo `Bgra8Unorm` `128×88` (o
acumulador do mundo), cena do oráculo; `tests/it/lens_gpu.rs` (`#[ignore]`, `PH2D_GPU=1`):

| caso | desvio máximo (níveis de 8 bits) | canais que diferem (de 18 432) |
|---|---:|---:|
| aberração radial · linear | 1 · 1 | 172 · 207 |
| vinheta escura · clara quase retângulo | 1 · 1 | 516 · 502 |
| grade sem LUT · só a LUT (9³, `mix 0,7`) | 1 · 1 | 187 · 450 |
| grão mono · colorido | 1 · 1 | 447 · 496 |
| os quatro juntos, com LUT | 1 | 529 |

Fora do sub-rect: zero bytes mudam (todo caso). Lente neutra: `aplica` devolve `false` e o alvo é
byte-idêntico, mesmo com LUT carregada e `lut_mix = 0`. O hash do grão (`pcg3d`) e a gaussiana de
célula: **o mesmo bit** em 256 células (sonda de compute sobre a mesma biblioteca WGSL).

Desenho que a W1 decidiu (e porquê):
- a fonte lê-se em bytes CRUS (`textureLoad`) e **des-pré-multiplica antes de descodificar** — o
  acumulador guarda sRGB pré-multiplicado, e a vista sRGB do hardware descodificaria o valor ainda
  multiplicado (errado com `α < 1`); a descodificação à mão é a fórmula da `ph2d-color`;
- o que é por QUADRO (balanço, geometria da vinheta, deslocamento/escalas da aberração, semente,
  `2^exposure`, `2^contraste`) sai das funções da `ph2d-lens` e viaja como uniform — o deslocamento
  linear ganhou a porta `aberracao::deslocamento` para não ser reconstruído por subtração;
- a LUT é uma textura 3D `Rgba32Float` (os valores exactos da CPU), lida por `textureLoad`;
- o uniform tem o mesmo tamanho e a mesma ordem de campos nos dois lados (gate pelo `naga`).

### §5.3 — O nó e a costura (W2, 10/10)

- **O nó** `ph2d-node-fx-lens`: passthrough com kernel `PASSTHROUGH` no dispositivo, 22 params + o
  texto `lut` (`ParamWidget::File { kind: FileKind::Lut }`, espécie nova, filtro `.cube`), quatro
  secções na ordem fixa (só a `Vignette` nasce aberta, e é a única ligada: `0,4`), o ângulo da
  aberração só no modo Linear, o `LUT Mix` só com LUT. Um passe de tela é UM: `fx.lens` entrou em
  `SINGLETON_SCREEN_PASSES`.
- **O estado** `ph2d-app-motion/src/motion_lens.rs` (`MotionState::lente`): `prepara` no
  `coze_o_quadro` (o grafo, o TIQUE de `motion_tick` — o grão anda com o relógio do Motion — e o
  `.cube`, relido só quando o caminho ou a data do ficheiro mudam); `liga` no plano do quadro
  (`plan.banded |=`); `aplica` depois da colagem da placa e antes do vidro.
- **A recusa da LUT diz-se no cartão**: a linha de leitura do `Lens` troca o readout pela frase
  (`motion_bridge_readout.rs`), uma frase por variante do `ErroCubo` + ficheiro ilegível.
- ⛔ **Medido na foto: o `scene_viewport` NÃO é a cena que se vê.** O mundo é renderizado por
  baixo dos painéis docados (`[0, 0, 1600, 534]` numa janela `1600×972`), e uma vinheta centrada
  ali escurecia cantos ESCONDIDOS — os cantos visíveis não mudavam (foto 1). Cura: o quadro da
  lente é `HeroScreen::cena_visivel` = o conteúdo depois das réguas (já sem painéis nem timeline),
  em pixels físicos, cortado pelo split (`[332, 96, 960, 438]`). Depois: cantos visíveis `83 → 69`
  (≈ −17 %) com o centro `86 → 85`.
- A cena `=135` (`motion_state_acabamento_demo.rs`): a galáxia simulada da `=132` com o degradê
  OKLCH, `Tint → Lens → Output`; exemplos `docs/Motion Nodes/tutoriais/luts/quente.cube` (entra) e
  `lut_1d.cube` (o cartão recusa).
- Achado de passagem: o ângulo do `fx.glow` está gateado ao contrário — [BUGS #19](../../BUGS_motion_nodes.md).

### §5.4 — Os filtros da camada (W3, 10/10)

- **O nó** `ph2d-node-fx-filter`: passthrough, kernel `PASSTHROUGH`, **NÃO singleton** — a pilha são
  os `fx.filter` no fio principal (porta 0) de cada `Output`, de montante para jusante
  (`from_graph`); um `fx.filter` fora do fio diz-lo no cartão (`fora_do_fio` → `MotionFiltro::queixa`,
  «Not on the wire to an Output - filters nothing»). 23 params: o tipo (Enum, as 15 chaves
  `node.opts.node_fx_filter.kind_labels.*` — o gate confere cada palavra contra a `SPECS`) e os
  knobs, cada um com um `ParamGate` sobre o tipo DERIVADO em `const fn` da `SPECS` — a lista
  COMPACTA, com o comprimento contado por genérico `const` (o sentinela `-1` do doc 94 §1 reprovava
  o censo `no_gate_hides_nothing`, que mede o comprimento: uma lista de 15 com sentinelas lê-se como
  «não esconde nada»). Os três params de modo (queda · ruído · rampa) servem os tipos com a MESMA lista
  de modos (gate: todo tipo com modos tem exactamente um). Fora da ficha, declarado: a lei de
  mistura (Normal) e a rampa do Gradient Map com 2 stops (as duas cores).
- **A régua de comprimento é MEDIDA:** raio, deslocamento, tamanho e crescimento partilham
  `COMPRIMENTO_MAX = 0,6` mundo — o raio em que o passe SATURA (`MAX_HALF = 96` ⇒ `σ ≤ 32` px) na
  câmera do produto (`height_world = 10` sobre os `534` px da cena: `53,4` px/unidade). Gate
  `the_length_ruler_is_where_the_pass_saturates` refaz a conta pela projecção da câmera.
- **Mundo → pixel tem UMA porta:** `FxOpGpu::resolve(op, [a, b, c, d])` (`ph2d-render`), chamada pelo
  Vector (`fx_live_resolve`, a conta saiu de lá) e pelo Motion (a parte linear de
  `Camera2d::world_to_screen_affine` no sub-rect da cena, sem a matriz de modelo — Y para baixo).
- **A camada** (`ph2d-app-motion/src/motion_filter.rs`): com pilha, o Motion SAI do passe fundido
  (`MotionState::sprites_da_camada`, a mesma pergunta para os dois) e o quadro vai em faixas; no
  lugar da colagem da placa: as sprites por `render_instances_only` num alvo linear `Rgba16Float` →
  `BandBlit` `Sprites` numa camada `Rgba16Float` (espaço do desenhista, pré-multiplicado) → a placa
  das formas por cima (`Formas`) → `FxStackPass::run_premultiplied` → `BandBlit` `Vector` sobre o
  `WorldRt`, antes da lente. Sem `fx.filter` no fio, o quadro é o de sempre (os dois ramos novos só
  correm com `MotionFiltro::liga = true`).
- **O ingest pré-multiplicado:** o `_pad3` do `Globals` virou `src_premul` (o Vector escreve `0` e
  fica byte-idêntico); com `1` o ingest desfaz `rgb·α` antes do `srgb_to_linear3`.
- **Medido na placa** (`tests/it/fx_stack_premul_gpu.rs`, `PH2D_GPU=1`): a mesma imagem reta ou
  pré-multiplicada sai igual — pilha vazia **0** níveis, blur · sombra · ajuste de cor **≤ 1** —, e
  o controlo (a camada lida como reta) desvia **> 20**. O borrão σ = 3 da camada contra o ImageMagick
  (`blur_linear_s3.txt`), no miolo (a pilha tem borda TRANSPARENTE, o oráculo REPLICADA; margem 10 px
  > `3σ`): **1 161** pontos, pior **1,19** níveis; o controlo σ = 2,5 desvia **16,65**.
- **Na foto** (`fotografa_cena.sh`, `=135`, 1600×972): sem filtro a cena de sempre; com o `Filter`
  em `Drop Shadow` cada disco ganha a sombra para baixo e à direita, com a vinheta da lente por cima.
- ⚠️ **A camada filtrada vai POR CIMA do mundo** (a camada do After Effects): fora do passe fundido o
  Motion deixa de se intercalar por z com as sprites da cena. Onion, partículas e barras ficam lá.
- ⚠️ **Sem tonemap:** ele está em bypass (`clamp`); o gate `the_layer_skips_a_tonemap_that_is_a_bypass`
  reprova se o AgX ligar — a camada terá de o seguir.

### §5.5 — Auditoria do `fx.filter` (10/10, depois do smoke do dono)

O dono: *«a sombra interna não se aplica em cada forma individual; o bevel tem qualidade ruim — as
linhas parecem um ruído animado»*. Duas lentes (correcção · costura de UI) e uma SONDA na placa
(`ph2d-render/tests/it/fx_stack_camada_sonda.rs`, `PH2D_GPU=1 … --ignored --nocapture`): discos de
raio 7 px, sombra σ 5,3 px deslocada (6, 6), bevel de profundidade 5,3 px.

| # | achado | mecanismo | medido / reproduzir | o gate que faltava · e o verde que o escondia |
|---|---|---|---|---|
| A1 | **Sombra interna, brilho interno, bevel, contorno, feather e grow veem a UNIÃO das formas** | a pilha corre UMA vez sobre a camada inteira (`motion_filter.rs`), e esses tipos leem a vizinhança (alfa invertido ou campo de distância); o Vector corre POR FORMA, numa célula | miolo do disco central (255 = sem sombra), modo Proximity: **sozinho 128 · cercado com vão 2 px 200 · a tocar 228**; modo Contour: sozinho 30 · a tocar 79 | um gate «a forma cercada recebe o MESMO efeito que sozinha» · verde: `a_premultiplied_layer_filters_like_the_straight_image` compara dois caminhos da MESMA camada (razão entre dois doentes) e o oráculo do borrão é uma cena sem formas vizinhas |
| A2 | **O default do nó é OUTRO algoritmo que o do Vector** na sombra/brilho interno e no Glow | o nó nasce com `falloff = 0` (Proximity = borrão do alfa invertido) para todo tipo; o `FxOp::new` do Vector herda `MODE_CONTOUR` do `BLANK` (campo de distância) | o mesmo disco sozinho: Proximity 128 contra Contour 30 | gate «um tipo escolhido no nó = o `FxOp::new(tipo)` do Vector» · verde: `a_dropped_filter_is_the_manifest_defaults` confere o nó contra o PRÓPRIO manifesto |
| A3 | **O bevel do Motion vai sempre pelo campo SEMEADO PELO RASTER, e ele treme sob movimento sub-pixel** | sem geometria (`geom = &[]`) o campo é JFA sobre a cobertura limiarizada; o normal do relevo é o gradiente desse campo, quantizado ao texel — a galáxia anda frações de pixel por quadro | o mesmo ponto do disco, deslocado ¼–¾ px: **raster muda 16–43 níveis · exacto 3,5–10,7** | gate de estabilidade sub-pixel do caminho raster · verde: `fx_stack_bevel_gpu` mede a constância só com `oblique_segments` (o caminho EXACTO); `both_paths_actually_light_the_rim` só afirma presença |
| A4 | **As cores do nó entram na pilha no espaço ERRADO** | o cartão do Motion guarda a cor em RGBA LINEAR (`motion_bridge_color.rs:81/290`, `srgb8_to_linear_rgba`); a `FxOp.color` é sRGB RETO e o shader lineariza-a (`tint_lin`, `fx_stack_shader.rs:80`) ⇒ linearizada DUAS vezes: toda cor intermédia sai escura e saturada | escolher um cinzento médio (sRGB 128) para a sombra: chega à pilha como sRGB 55 | gate «a cor escolhida no cartão chega à pilha como o artista a vê» · verde: `the_ramp_is_the_two_colours` usa preto e branco, que a transferência não move (fixtura sem o fenómeno) |
| A5 | **Defaults por tipo ausentes**: Glow, Inner Glow e Color Overlay nascem PRETOS, Duotone preto→branco; a luz do bevel nasce espelhada (`0,12, −0,12` contra o `−0,1, 0,1` do Vector) | o manifesto tem UM conjunto de defaults; o `FxOp::new(tipo)` tem um por tipo | escolher Glow: um halo preto; Color Overlay: tudo preto. O roteiro da `=135` já ensinava o contorno («Glow com Color clara») | o mesmo gate do A2 |
| A6 | **A turbulência ancora o ruído na JANELA** | `org` = a margem da pilha, que no Vector é a caixa da forma e no Motion é a janela inteira | as formas andam e o padrão fica parado no ecrã (efeito «vidro de chuveiro»); um pan da câmera idem | gate «o padrão anda com a forma» |
| A7 | **A luz do bevel e o deslocamento arredondam a pixel INTEIRO** | `FxOpGpu::resolve` arredonda `offset_px`; com `|offset| < 0,5` px a luz cai no fallback `(0, −1)` | zoom-out: a direcção da luz salta | os fixtures passam `offset_px` inteiro e contornam o `resolve` |
| A8 | **A régua `COMPRIMENTO_MAX` é verde por construção** | o gate alimenta os `534` px como ENTRADA; a saturação depende da altura da cena (`k = viewport_h / height_world`) | numa janela de 4K a mesma régua satura em ~0,27 | um gate que meça a janela do produto, ou a régua por px de ecrã |
| U1 | **Três «Filter» e dois «Glow»** | o nó chama-se Filter, a linha do tipo também, e há o componente `vec_filter` «Filter»; a opção Glow tem o nome do nó `fx.glow` | buscar «glow» no menu de adicionar | gate de rótulos únicos por superfície |
| U2 | **O cartão mostra a régua de comprimento em px do PROJECTO** | `display_face` multiplica pelo `pixels_per_meter` do projecto (`motion_bridge_params_wire.rs:179`), não pela câmera | o máximo «0,6» aparece como px que não são os do ecrã | — (menor) |

Costura OK (lente 2): o seletor de 15 tipos alcança-se por clique (as bordas andam, o centro abre a
lista); os gates escondem as linhas no cartão e no painel pela MESMA função; o aviso «fora do fio»
chega ao cartão mesmo sem cozimento; o `hue` em graus com sufixo «deg» e sem dar a volta.

### §5.6 — Cada forma e o resto da auditoria (W3b, 10–11/10)

| achado | a cura | medido (gate) |
|---|---|---|
| A1 a sombra interna/bevel/contorno veem a UNIÃO | `scope` no nó; os `Each Shape` correm nas CÉLULAS de um átlas (`ph2d-fx-cells`): cada instância sozinha, a pilha UMA vez sobre o átlas, a volta na ordem de z | disco cercado × sozinho: **0–1 nível** (Proximity e Contour, a tocar e com vão de 2 px); a camada inteira, o controlo: **28–146** (`a_surrounded_shape_gets_the_same_effect_as_alone`) |
| — a vala entre células | a margem é o máximo do que a pilha ESPALHA (`stack_reach`) e do que ela LÊ (`stack_read`, porta nova) | só com o `stack_reach` a sombra interna (espalha 0, lê ~23 px) desviava **71** níveis |
| A2/A5 os defaults de outro algoritmo | `op_of` parte do `FxOp::new(tipo)`; `switch_kind` (o gesto do cartão, o mesmo passo de undo) escreve os defaults do tipo novo nos knobs não tocados | todo tipo = o `FxOp::new` do Vector; controlo: só o `kind` → Glow preto (`a_kind_chosen_on_the_card_is_the_vectors_new`) |
| A3 o bevel do raster treme | semente de Gustavson & Strand (Sobel √2 + `edgedf`) em TODO texel de cobertura parcial, a normal da aresta na semente e a distância à TANGENTE perto do pé | ¼–¾ px: raster **6,8** níveis no pior (era 43) contra o exacto **10,7**; contorno à mesma distância **5** (fixtura de cobertura por ÁREA; a antiga era a rampa da semente antiga) |
| A4 a cor linearizada duas vezes | `read_color`/`card_color`: a fronteira linear ↔ sRGB no nó | sRGB 128 chega como 128 (controlo: a fixtura move-se > 0,2) |
| A6 o ruído preso à janela | `Origem` explícita: o mundo (camada inteira) ou o mapa por célula | um pan de 1 unidade leva a origem `k` px (`the_layer_noise_is_anchored_to_the_world`) |
| A7 a luz/deslocamento a pixel inteiro | `offset_px` em `f32`: a gaussiana deslocada `gauss(k + frac)`, o campo interpolado | ¼ px move a sombra (arredondado: 3 de 4 passos parados); luz `(0,3; 0,2)` = `(3; 2)` |
| A8 a régua verde por construção | o borrão desce na PIRÂMIDE acima de σ 32 px (médias 2×2 com uma célula de folga; o σ de baixo desconta a caixa e a tenda); slider 0,6, digitado até 10 | retângulo borrado × `Π Φ` a σ 30/100/300: **0,9 / 1,0 / 1,2** níveis (a lei antiga: 4 / 64 / 128); o tecto digitado a 8K no zoom máximo cabe no `SIGMA_ALCANCAVEL` |
| U1 nomes repetidos | a linha do tipo é «Effect»; «Glow» → «Outer Glow» (também no Vector); o nó `fx.drop_shadow` (cópias) → «Shadow Copies» | `the_filter_labels_are_unique_where_they_meet` |

- **O relógio** (`relogio_cada_forma_contra_camada_inteira`, perfil `smoke`, a sombra interna,
  camada 1600×972, mínimo de 12 blocos intercalados, `load 4,3`): **256 discos — cada forma
  0,55 ms (1 página) · camada inteira 0,51 ms; 4 096 — 4,64 ms (4 páginas) · 0,61 ms.** Inclui a
  leitura das caixas (à espera da própria submissão).
- **As duas rotas das sprites:** a da CPU sai IGUAL à camada inteira (0 texels); a da placa fica
  ≤ 1 nível fora da silhueta e muda 17 de 8 716 texels NA silhueta (0,2 %) — o desenhador já difere
  entre as suas duas rotas em 13 texels na camada inteira. Sem filtro, as duas rotas são idênticas
  entre «cada forma» e «camada inteira». Fixtura AMPLIADA: reduzida, a escolha do nível de mipmap
  (o disco de 256 px a ~22 px, ×11,6 ≈ 2^3,5) troca de nível por uma derivada ínfima.
- **A relocação** é em MUNDO (`L⁻¹·T`, `T` inteiro): a fração sub-pixel não muda e a volta é cópia
  ao texel (gate `the_device_relocation_is_the_cpu_sum`).

### §5.7 — As fotos, o relógio e a mistura (W4, 11/10)

- **As fotos** (`fotografa_cena.sh`, 1600×972, `PH2D_ACABAMENTO_PASSO=<passo>`: cada passo do roteiro
  escrito na cena pelo MESMO gesto do cartão; gate `every_photo_variant_is_the_step_it_names`), em
  [`handoffs/fotos_W4/`](../../handoffs/fotos_W4): os passos (0)–(8) e a `mistura`. ⚠️ A 1.ª rodada
  mostrou que o passo (1) **não se via**: o disco de raio `0,15` deixava um vão de `0,02` na grelha
  de `0,32`, e as fotos de `Each Shape` e `Whole Layer` saíam iguais — a cena ensinava o que não
  acontece. O raio passou a `0,2` (gate `the_discs_touch_so_whole_layer_differs_from_each_shape`).
  O cartão do passo (4) não se fotografa (os cartões estão fechados na foto): o gate
  `an_each_shape_after_a_whole_layer_says_so_on_its_card` prova-o.
- **O relógio** (`relogio_cada_forma_contra_camada_inteira`, `--release`, 12 blocos intercalados, o
  mínimo, `load 17`; a sombra interna):

  | | 1600×972 · 256 | 1600×972 · 4 096 | 4K · 256 | 4K · 4 096 |
  |---|---:|---:|---:|---:|
  | cada forma (átlas) | 0,60 ms | 4,63 ms | 0,77 ms | 4,89 ms |
  | camada inteira, janela inteira | 0,52 | 0,63 | **2,01** | **2,16** |
  | camada inteira, só a cena (55 % × 55 %) | 0,28 | 0,37 | **0,77** | **0,85** |
  | a lente, 4 secções, janela inteira | 0,37 | 0,37 | 1,75 | 1,75 |

  ⇒ a pilha da camada passou a correr SÓ no retângulo da cena alargado pelo que ela lê e espalha
  (`ph2d_render::retangulo_da_camada`, `run_layer_rect`, `blit_rect`); a lente já corria só na cena.
  Gate `fx_stack_camada_rect_gpu` (`PH2D_GPU=1`, `--ignored`), camada 1100×760, discos também FORA da
  cena: dentro dela **0 níveis** (sombra de fora para dentro; sombra interna + borrão σ 45 na
  pirâmide, a origem na grelha de 2) e **1** (turbulência no mundo); o controlo sem margem desvia
  **141 · 87 · 197**.
- **A rota da CPU com filtro ACONTECE** (o aberto do W3): a saída em `Add` manda tudo ao Vello e o
  filtro não alcança nada (§3.5). O cartão de cada `Filter` di-lo desde a W4; a cura é a W4b.

### §5.8 — A mistura atravessa a camada (W4b, 11/10)

- **A pousa é mistura de HARDWARE, não um passe com cópia do mundo** (o desenho do §3.5 pedia a
  cópia): `Soma` = `One/One`, `Tela` = `One/OneMinusSrc` — exactos com qualquer alfa —, e o
  `Multiplica` em DOIS passes no mesmo passe de desenho (`Dst/OneMinusSrcAlpha` com o alfa intacto,
  depois `OneMinusDstAlpha/One`): `s·b + b·(1−αs) + s·(1−αb)`, exacto. `ph2d_render::Pousa` +
  `BandBlit::blit_pousa`. Gates `band_blit_pousa_gpu`: contra a lei ≤ 1 nível (fundos meio
  transparentes incluídos; o controlo separa os quatro modos e o `Multiplica` de um passe só) e
  contra o Vello **0 níveis** nas 18 células opacas.
- **A porta do produto pinta o que o quadro sem filtro pinta, AO BYTE**: a tabela do doc 118 pela
  cena vectorial na camada, `filtra_a_camada` com pilha neutra e a pousa — 9 células (3 modos × 3
  alcances), diferença **0** contra o quadro de hoje (gate
  `the_output_blend_crosses_the_filtered_layer_as_it_paints_without_it`).
- **O `Scene` não agrupa com a camada ligada** (`MisturaDoSink::pousa_na_camada`, a mesma regra na
  bomba e na rota da placa): as formas vão à placa e às células; `Everything`/`Copies`, o `Blend`
  por linha e a geometria recusada vão ao **Vello dentro da camada** (`cena_da_camada`, sem o
  mundo por baixo), onde os `Each Shape` correm sobre o CONJUNTO (`FxStackPass::run_straight`) e o
  cartão di-lo (`each_shape_sees_the_vector_path_as_one`).
- **O preço do Vello na camada** (`o_preco_do_vello_na_camada`, `--release`, 12 blocos
  intercalados, o mínimo, `load 12–23`; pilha neutra):

  | | 1600×972 · 256 | 1600×972 · 4 096 | 4K · 256 | 4K · 4 096 |
  |---|---:|---:|---:|---:|
  | a camada sem formas | 0,10 ms | 0,10 | 0,29 | 0,28 |
  | com o Vello dentro dela | 0,49 | 1,85 | 1,54 | 2,25 |

  ⚠️ Não é custo NOVO por inteiro: esse Vello corria antes na cena do quadro (agora sai dela).
- **Duas saídas com modos diferentes partilham a camada** (item 4, decidido): medido, NENHUMA cena
  tem duas saídas com filtro, e uma camada por saída partiria a ordem por linha ENTRE saídas (doc
  119 W4) e pediria faixas nas sprites, nas células e na placa. ⇒ a camada pousa com o modo da
  primeira, e o cartão de cada filtro di-lo (`outputs_land_differently`).
- **A foto** (`handoffs/fotos_W4b/`, passo (9) = `Output` em `Add`): os discos clareiam o fundo e o
  relevo fica; a sombra escura some na soma (a lei). A da W4 (`mistura`) saía chapada.

### §5.9 — As provas de mutação (W5, 11/10)

| lote | linhas | sangravam à 1.ª | o que se fez | depois |
|---|---|---|---|---|
| W4b | 21 | 15 | as 6 sobreviventes (P9, P11, P12, P14, P18, P19) viraram 6 gates: `com_a_camada_o_scene_desenha_sem_modo_e_o_everything_agrupa`, `the_device_route_asks_the_same_landing_rule`, a tabela de ponta a ponta nos dois ramos do Vello, `the_frame_step_isolates_the_pump_when_a_filter_is_on_the_wire`, `as_formas_do_motion_vao_a_camada_filtrada` | **21/21** |
| W0–W4 | 41 | 36 de 40 (Q23 sem arnês) | Q23 (shell) → gate de texto `com_filtros_o_motion_sai_do_passe_fundido`; Q10, Q17, Q25, Q28 → `the_grain_follows_the_frame_tick`, `the_lens_frame_is_the_scene_not_the_window` (a escolha passou a `motion_lens::quadro_da_cena`, partilhada com a camada filtrada), a partição sem `Whole Layer`, `every_kind_keeps_its_default_colours_through_the_card` — cada um visto VERMELHO sob a sua mutação; Q29–Q31, Q34 já sangravam (ver abaixo) e ganharam gates directos | **41/41** (Q7 só pela placa) |

- **Q29–Q31, Q34** (o passe do campo: semente G&S, ramo do pé da aresta, distância à tangente,
  `org_map`) **não sobreviviam**: a 1.ª corrida só filtrou os 5 gates da tabela; sob a suíte `fx_stack`
  inteira cada uma já derrubava gates antigos (Q29–Q31: `the_raster_bevel_holds_still_under_subpixel_motion`
  e outros; Q34: `the_noise_is_pinned_to_the_shape_not_to_the_scratch`). ⚠️ Lição: o filtro de uma
  prova de mutação é a TABELA, e uma tabela estreita lê «sobrevive» onde a suíte sangra. Ficaram dois
  gates DIRECTOS: `the_raster_field_is_the_true_distance_to_a_disc_at_every_angle` (disco fora da
  grelha, 1 027 texels da rampa: real 0,047 px · Q29 0,654 · Q30 0,297 · Q31 0,297; barra 0,1) e
  `the_map_is_the_point_it_holds_texel_by_texel` (o `Origem::Mapa` = o `Ponto` de cada metade, ao
  byte; os antigos só cobriam o lado `Ponto`).
- **Q7** (a ordem fixa da lente) só é apanhada pelo gate de placa `the_four_together_match_the_cpu`.
- Auditoria (duas lentes): «camada codificada e não desenhada» = falso positivo
  (`motion_active` de `present_world_sprites` recebe `motion_draws`); «virar o isolamento não suja o
  pump» = real, gate `turning_the_layer_on_in_the_same_tick_relowers`.


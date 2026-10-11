# 129 — CICLO 20: O ACABAMENTO (a lente da câmera e os filtros da camada)

> Quarto ciclo da obra 17 → 23 ([doc 126 §0](126_ciclo_17_o_ritmo.md), linha 20), aberto depois do
> smoke do ciclo 19 e do BUGS #18 aprovados (10/10). Base: `line/motion-value` sobre o `main`
> `2a925859d` (ciclos 13–20 por integrar). Prompt:
> [handoff de continuação](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-10_CICLO_20.md).

## §1 — O grupo e a premissa do tutorial

O item do doc 126 lista oito efeitos (blur · glow · sombra de pixel · warp · grade/LUT · vinheta ·
grão · aberração). A auditoria (§2) mostra que eles são **duas famílias com leis diferentes**, e o
grupo do ciclo é as duas:

| família | o que é | efeitos | quem decide a ORDEM |
|---|---|---|---|
| **A LENTE** (`fx.lens`, novo) | a imagem da CENA inteira, como a câmera a vê | aberração cromática · vinheta · grade (exposição, balanço de branco, contraste, saturação, LUT `.cube`) · grão | a **física**: lente → filme (fixa, §3.2) |
| **OS FILTROS DA CAMADA** (`fx.filter`, novo — o `fx.adjust` do [doc 94](94_plano_fx_adjust.md)) | a imagem da camada do Motion | blur · sombra de pixel · warp (turbulência) · ajuste de cor · mapa de gradiente · duotone… (os 15 da `FxOp::SPECS`) | o **artista**: a ordem dos nós no fio |
| o brilho | já existe: `fx.glow` (bloom HDR, doc 67) | glow | — |

**Tutorial 13 «O acabamento»:** uma cena viva (formas numa simulação com campos) que começa crua e,
nó a nó, ganha a cara de um plano de cinema — a lente torce as cores nas bordas, a vinheta fecha o
olhar no centro, o grade esquenta a luz e carrega uma LUT, o grão dá a película; depois os filtros
da camada borram o fundo e projetam a sombra. O artista liga e desliga cada nó e vê a diferença.

## §2 — Auditoria (o código e o estado da arte)

**O código** (exploradores, 10/10 — conferido no fonte antes de acreditar em cada ausência):

| efeito | existe? | onde | o que falta |
|---|---|---|---|
| glow | ✅ pixel, HDR | `fx.glow` → `MotionFx::bloom_over` (`present_fx.rs`) | nada |
| blur · sombra de pixel · warp · ajuste de cor | ✅ na placa, **inalcançável do grafo** | `ph2d_fx_op::FxOp` (15 tipos) corrida por `FxStackPass` (`ph2d-render/src/fx_stack.rs`), só o Vector a chama (`ph2d-app-vec/src/fx_live.rs`) | a PONTE (doc 94 §0, medido 30/08: o nó e o passe de composição) |
| `fx.drop_shadow` · `fx.rgb_split` | geometria | cópias por instância (`ph2d-node-fx-drop-shadow/src/lib.rs:2-41`, `ph2d-node-fx-rgb-split/src/lib.rs:12-35`) | — não são pixel e ficam como estão |
| aberração cromática | só no Painter | `AdjustmentKind::ChromaticAberration` (compositor de DOCUMENTO) | inalcançável fora de camadas de um `.ph2d` (doc 94 §0 linha 2) |
| grão | só no Painter | `AdjustmentKind::Noise` (idem) | idem |
| LUT | ⛔ **não há LUT de ficheiro** | o `ColorLookupLut` do Painter são 8 looks escritos à mão (`painter-effects/src/adjustments/lut.rs`); nenhum leitor de `.cube` no workspace (`grep LUT_3D_SIZE`) | o leitor e a amostragem 3D |
| vinheta | ⛔ nenhuma | `grep vignette` = 0 no workspace | tudo |

**Três medições que decidem o desenho:**
1. ⛔ **Apender tipos à `FxOp` custa o ficheiro e o Vector:** a `FxOp` é postcard posicional sem
   `serde(default)` (`ph2d-ecs/Cargo.toml:83-88`), viaja no `VecFilter` e no `ObjectPose.filters`;
   o painel do Vector corta em `MAX_FILTER_KINDS = 15` literal (`ph2d-panel-vector/src/ids/vector_filters.rs:35`)
   e um tipo desconhecido cai no ramo do Glow do `cs_op_point`. ⇒ os efeitos de LENTE **não entram
   na `FxOp`** — são da câmera, não de uma forma.
2. ⚠️ **Antes do tonemap a cena não está inteira:** o Motion entra no `game_rt` misturado por z com
   os sprites (`present.rs:258-342`), e as formas vectoriais da rota da CPU são desenhadas DEPOIS,
   pelo Vello, em LDR. A cena inteira sem a interface só existe no **acumulador do mundo**
   (`WorldRt`, `Bgra8Unorm` + vista sRGB, `world_rt.rs:38-40`) — o modo em faixas, que a placa das
   formas já liga (`present.rs:299`). ⇒ a lente corre **no acumulador**, depois da última escrita do
   mundo (a colagem da placa, `present_chrome.rs`) e antes do vidro e do chrome.
3. A camada do Motion **não é uma lista só**: o halo do glow difere do passe fundido nas formas da
   rota da CPU (tiles assados) e não leva onion/partículas (`motion_glow_layer.rs:92-165`). ⇒ os
   filtros da CAMADA pedem isolar a camada de verdade — é a wave mais cara (§4).

**O estado da arte:**

| quem | o que faz | o que levamos |
|---|---|---|
| Unity URP — *Volume* (Bloom · Chromatic Aberration · Vignette · Color Adjustments · White Balance · Color Lookup · Film Grain) | a pilha de PÓS da câmera tem **ordem fixa** (a do *uber post*), não escolhida pelo artista | a ordem fixa da lente; o conjunto pro de params de cada um |
| Unreal — *Post Process Volume* (Chromatic Aberration · Vignette Intensity · Grain · Color Grading + LUT) | idem, com LUT por textura | a LUT como 3D, misturável por `mix` |
| DaVinci Resolve / Adobe Lumetri (vinheta: Amount · Midpoint · Roundness · Feather) · padrão *Adobe Cube LUT 1.0* | a vinheta do COLORISTA; o `.cube` é o formato de troca da indústria | os quatro params da vinheta; o `.cube` lido pela spec pública |
| After Effects / Cavalry — efeitos por camada, ordenados | a pilha da CAMADA é do artista | a ordem do fio nos filtros da camada |
| Godot 4 (MIT, `4.7.2`) — `Environment` (adjustments, color correction) | grade por textura 3D | — oráculo de reserva |

**Os oráculos (§0.9 — triagem por ARTEFACTO, `LC_ALL=C pacman -Qi`, 10/10):** `imagemagick
7.1.2.32` — licença *ImageMagick* (permissiva): `-gaussian-blur`, `-vignette`, `-distort SRT`,
`+noise` · `ffmpeg 9.0.2` — **GPL-3.0-only: corre-se, nunca se lê** (`gblur`,
`vignette`, `lut3d`, `rgbashift`, `noise`) · `godot 4.7.2` — MIT. Correm em W0 sobre entradas
NOSSAS e gravam fixtures com cabeçalho (§5.1).

## §3 — O desenho

### §3.1 — Duas portas, porque são duas perguntas

`fx.lens` e `fx.filter` nascem com a forma do `fx.glow` (doc 94 §0): **nó passthrough**
(`out == in`, `Effect::Pure`) que DECLARA, e um passe que a shell corre. Sem o nó no grafo o quadro
é **byte-idêntico** (gate). ⛔ Nenhum toca `NodeOp`/`OpResolver`/`NodeManifest` — são nós novos.

### §3.2 — A lente: ordem fixa, um passe, uma lei na CPU

`fx.lens` tem quatro secções dobráveis (`ParamGroup`), cada uma com `amount` que a desliga em zero:

1. **Aberração** — `amount` (fração da meia-diagonal) · `mode` Radial | Linear · `angle` (Linear).
   Lei: cada canal amostrado na posição escalada a partir do centro (R por `1+k`, B por `1−k`, G
   fixo); Linear = deslocamento por canal.
2. **Vinheta** — `amount` (−1..1: negativo clareia) · `midpoint` · `roundness` · `feather` ·
   `center` · `color`. Lei: borda gaussiana (`Φ`) sobre a distância elíptica normalizada.
3. **Grade** — `exposure` (stops) · `temperature` · `tint` · `contrast` · `saturation` (OKLCH, a
   lei do ciclo 18) · `lut` (o NOME de um `.cube`) · `lut_mix`.
4. **Grão** — `amount` · `size` · `response` (peso nos meios-tons) · `colored` · `animated`
   (semente = f(tique): o scrub devolve o MESMO grão — lição do ciclo 19).

**Ordem fixa** aberração → vinheta → grade → grão (a do *uber post*: lente, depois o filme). Um
passe de tela sobre uma cópia do acumulador, em luz LINEAR (descodifica a vista sRGB, codifica na
escrita). A cena da Motion no split usa o MESMO sub-rect (`scene_viewport`): o centro da vinheta é o
centro da CENA, não da janela.

**Uma lei, duas línguas:** a crate-folha `ph2d-lens` (sem GPU) tem cada lei em Rust; o WGSL é
gateado contra ela em pixels (DIRETIVA §4). O `.cube` é lido por ela (spec *Adobe Cube LUT 1.0*),
amostrado tetraédrico nas duas línguas.

### §3.3 — Os filtros da camada (a ponte do doc 94)

`fx.filter`: um nó por degrau, a ficha DERIVADA da `FxOp::SPECS` (doc 94 §1 — nunca a lista à
mão); a ordem dos `fx.filter` no fio é a ordem da pilha. A camada do Motion desenhada ISOLADA num
alvo próprio (sprites + formas pela mesma lista do quadro), `FxStackPass` com ingestão/resolução
LINEAR premultiplicada (o alvo é `Rgba16Float`, não sRGB 8 bits), e o passe de composição «esta
textura sobre aquele alvo» — a metade B que o doc 94 §2 mediu como ausente.

### §3.4 — W3b: CADA FORMA ou a CAMADA INTEIRA (decisão do dono, 10/10, depois da auditoria §5.5)

*«Ambas as opções devem estar disponíveis no nó.»* ⇒ o `fx.filter` ganha o param **`scope`**
(Enum: **Each Shape** · **Whole Layer**; nasce **Each Shape**, o que o dono esperava).

- **A pilha parte-se em DOIS estágios pela ordem do fio:** os degraus `Each Shape` do início do fio
  correm por forma; o primeiro `Whole Layer` junta as formas e tudo o que vem depois corre na camada.
  Um `Each Shape` DEPOIS de um `Whole Layer` já não vê as formas: corre na camada e o cartão di-lo
  (a mesma porta da queixa «fora do fio»). Gate da partição.
- **Cada forma = um ÁTLAS de células com valas.** Cada instância do Motion desenha-se na SUA célula
  (o `RenderInstance.world_pos` e o `ShapeInstance.pos` são translação pura — relocar é somar;
  a fração da posição no ecrã fica DENTRO da célula, logo a volta é exacta ao pixel); as células
  separam-se por valas ≥ `stack_reach` + 1, e então UMA corrida da pilha sobre o átlas = uma corrida
  por célula (nenhum degrau alcança além do `reach`). A volta: cada célula como sprite no lugar da
  instância, na ordem de z do stream — a sombra do disco de cima cai sobre o de baixo.
  ⚠️ A medir/decidir na wave: o tamanho da célula na rota do DISPOSITIVO (as caixas das instâncias
  não estão na CPU: passe de compute de limites com `atomicMax` + o tecto do átlas MEDIDO no
  dispositivo); a relocação das sprites e das formas no buffer do dispositivo (cópia + translação por
  compute); o custo contra a `Whole Layer` (MEDIR_VELOCIDADE.md) a 256 e a 4 096 instâncias.
- **O bevel e o campo semeado pelo raster (A3) — nas DUAS escalas:** a semente sub-pixel a partir
  da cobertura (a transformada de distância anti-aliased de Gustavson & Strand 2011, publicada) em
  vez do limiar; gate de estabilidade sob deslocamento de ¼ px (a sonda §5.5: raster 16–43 níveis
  contra o exacto 3,5–10,7 — o alvo é o exacto).
- **O resto da auditoria no MESMO bloco** (§0.10): A4 a cor do cartão (linear) convertida para a
  sRGB reta da `FxOp` na fronteira do nó; A2/A5 os defaults de cada tipo = o `FxOp::new(tipo)`
  (o gate compara o nó com a lei do Vector, nunca consigo próprio) — ⚠️ um default por tipo num
  manifesto único pede desenho: ou o nó escreve os defaults do tipo ao trocar de tipo (gesto do
  cartão, com undo), ou o tipo vira o default e os knobs «não tocados» seguem o `FxOp::new`; meça
  qual o cartão mostra sem mentir; A6 a origem do ruído na forma (Each Shape: a célula; Whole Layer:
  o mundo, pela câmera); A7 o deslocamento sub-pixel da luz; A8 a régua na janela do produto; U1 os
  nomes únicos.

#### §3.4.1 — O desenho decidido (W3b, 10/10, depois de ler o código)

**Estado da arte.** Illustrator: um efeito aplicado a cada objecto corre POR objecto; aplicado ao
grupo, corre na união — as duas leituras existem e o painel Appearance diz qual. Photoshop/After
Effects: estilos de camada correm na CAMADA (a união); «por forma» exige uma camada por forma.
Figma idem (efeito no grupo × em cada filho). ⇒ o `scope` é o idioma do Illustrator.

**Portas únicas.**
- *Que degraus correm por forma?* `ph2d_node_fx_filter::partition(&[NoDoFiltro])` — o prefixo
  `Each Shape` do fio; o resto é da camada; um `Each Shape` depois de um `Whole Layer` queixa-se pela
  porta do `fora_do_fio` (`MotionFiltro::queixa`).
- *Onde está cada instância no ecrã?* UM passe de compute de LIMITES (WGSL) sobre os buffers das
  instâncias (sprites, 47 palavras; formas, 16) — as DUAS rotas passam por ele (a da CPU sobe as
  instâncias para um buffer); a caixa volta à CPU por uma leitura pequena (`n × 16 B`, à espera do
  índice de submissão do próprio passe). A lei espelhada em Rust só nos gates de paridade.
- *Que célula é de quem?* `fx_atlas` (sai do `ph2d-app-vec` para o `ph2d-render`, o Vector reexporta),
  com um empacotamento EM ORDEM: páginas contíguas na ordem de z (sprites, depois formas — a ordem
  da camada de hoje), célula = caixa + margens do `stack_reach` + 1. Uma instância que não cabe numa
  página corre SOZINHA numa página do tamanho da janela (translação zero — igual a «sozinha» por
  construção). Instâncias fora da janela ⊕ alcance não entram.
- *Como chega lá?* translação INTEIRA `T_i` em pixels; a relocação no mundo é `L⁻¹·T_i` (a mesma
  parte linear da câmera no ecrã e no átlas), por compute (gather + soma) para as duas famílias; a
  fração sub-pixel não muda ⇒ a volta é cópia exacta ao texel.
- *A volta:* um passe de composição instanciado, célula a célula na ordem, `over` pré-multiplicado
  no espaço do desenhista, para a camada; depois a pilha `Whole Layer` (vazia = só a conversão).
- *A6:* a pilha recebe a origem do ruído explícita (`Origem`): `Whole Layer` = o pixel da origem do
  mundo; `Each Shape` = um mapa por célula (o pixel da âncora da instância no átlas); o Vector fica
  na margem (a caixa da forma — a translação dele não entra na chave do memo).
- *A7:* `offset_px` em `f32`: o borrão amostra a gaussiana deslocada (`gauss(k + frac)`), o finalize
  interpola o campo, a luz do bevel é o vector sem arredondar.
- *A3:* semente de Gustavson & Strand (2011): TODO texel de cobertura parcial semeia, com o gradiente
  de Sobel ponderado (√2) e a distância `edgedf(g, a)` da aresta recta no quadrado do pixel.
- *A8:* a régua deixa de ser onde o passe SATURA — o borrão ganha a pirâmide (σ acima de `MAX_HALF/3`
  corre numa resolução dividida por 2ᵐ e volta por bilinear), o tecto passa a ser o do hardware, e o
  gate mede contra o ImageMagick a um σ grande.
- *A2/A5:* trocar o tipo no cartão (o gesto, no MESMO passo de undo — precedente
  `apply_channel_presets`) reescreve cada knob que ainda vale o default do tipo ANTERIOR com o
  `FxOp::new(novo)`; o que o artista mexeu fica. O manifesto nasce com o `FxOp::new(tipo inicial)`.
- *A4:* `op_of` converte a cor linear do cartão para a sRGB reta da `FxOp` (`linear_to_srgb_unit`).
- *U1:* a linha do tipo chama-se «Effect»; a opção «Glow» passa a «Outer Glow» (o nome do
  Photoshop, par do «Inner Glow»); gate de rótulos únicos por superfície.
- U2 (px do projecto) fica nomeado: a unidade é do projecto inteiro, não deste nó.

## §4 — Waves

| wave | o quê | estado |
|---|---|---|
| W0 | os oráculos corridos (fixtures) · a crate `ph2d-lens` com as leis na CPU e os gates contra as fixtures | ✅ 14 gates verdes |
| W1 | o passe da lente em `ph2d-render` · paridade WGSL × `ph2d-lens` | ✅ §5.2 |
| W2 | o nó `fx.lens` · o cartão · i18n · a costura na shell (o acumulador; o modo em faixas ligado pelo nó) · o `.cube` por nome | ✅ §5.3 (+ a cena `=135` com a lente; o filtro entra na W3) |
| W3 | `fx.filter`: a camada isolada · a ingestão linear da pilha · o passe de composição · o nó derivado | ✅ §5.4 (+ a `=135` com uma sombra projectada) |
| W3b | `scope` (Each Shape · Whole Layer) · o átlas por forma · a semente sub-pixel do campo · o resto da auditoria §5.5 | ✅ §5.6 · smoke do dono APROVADO 11/10 |
| W4 | a cena `=135` (formas + simulação com campos) fotografada · o relógio do grupo · o tutorial 13 | — |
| W5 | provas de mutação · o handoff | — |

## §5 — Medições

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
- Achado de passagem: o ângulo do `fx.glow` está gateado ao contrário — [BUGS #19](BUGS_motion_nodes.md).

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

## §6 — Lições

- **O oráculo ensina a PERGUNTA e o programa dele tem convenções próprias** — dois comandos do mesmo
  ImageMagick põem o centro em sítios diferentes (índice × contínuo), e o leitor de PNG do ffmpeg
  desvia `0,0046`. Antes de gravar uma fixture, prove o oráculo num caso de resposta conhecida (os
  cantos da LUT têm de sair exactos).
- **O retângulo onde o mundo se RENDERIZA não é o que o artista VÊ** — os painéis docados pintam
  por cima do mundo. Um efeito de câmera centrado no primeiro passou nos gates de GPU (que não
  têm painéis) e só a FOTO mostrou que os cantos visíveis não mudavam.
- **«O mesmo bit nas duas línguas» exige UM arredondamento só**: a gaussiana do grão somava quatro
  uniformes em `f32` e a placa errou 1 ulp na célula `[-6, -6]` (o compilador da placa pode fundir
  ou reordenar). Cura: uniformes de 23 bits somadas em INTEIRO (a soma centrada cabe exacta num
  `f32`), e a única operação de vírgula flutuante é a multiplicação final.
- **Uma aproximação que acerta no caso fácil reprova no difícil**: `Φ(d/σ)` errava `0,03` com a
  borda fina e `0,23` com a larga. Varra as configurações ANTES de escolher a lei, nunca uma só.

## §7 — Abertos nomeados

- (W3b) A rota da PLACA do desenhador de sprites tem uma sensibilidade na silhueta que a da CPU não
  tem (13 texels entre as duas rotas na camada inteira, 17 entre células e camada) — do
  `SpriteRenderer`, não das células; medir quando uma cena de sprites com filtro aparecer.
- (W3b) 4 096 formas em cada forma = 4 páginas de 2048 (o `LADO_MAX`, recurso: memória) e 4,6 ms;
  as páginas reaproveitam o átlas em série. Uma página de 4096 custaria ~1,2 GB.
- (W3b) U2 (o cartão mostra px do PROJECTO): a unidade é do projecto inteiro, não deste nó.

- (W3) Formas da rota da CPU (o Vello, `formas_pedem_o_vello`) não entram na camada filtrada — medir
  quando acontece numa cena com filtro (W4).
- (W3) O passe da camada corre na janela INTEIRA (o tamanho do `WorldRt`), não no sub-rect da cena:
  o relógio a 4K é da W4.
- (W3) A lei de mistura do `fx.filter` sem ficha (os rótulos são da shell) e a rampa com 2 stops.

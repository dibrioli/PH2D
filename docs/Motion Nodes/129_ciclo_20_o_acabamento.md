# 129 — CICLO 20: O ACABAMENTO (a lente da câmera e os filtros da camada)

> **Índice** — partido em 2026-10-11 (`doc-split`, sha256). Este é o ciclo ATUAL: ficam vivos o desenho,
> o §5.10, as lições (§6) e os abertos (§7). As medições §5.1–§5.9 estão em
> `archive/129_ciclo_20_o_acabamento/`.

| § / Bug | Onde |
|---|---|
| §1–§4 (grupo, auditoria, desenho §3.1–§3.5, waves) | **neste doc** (vivo) |
| §5.1–§5.9 (medições W0–W5) | [`parte_1`](archive/129_ciclo_20_o_acabamento/parte_1.md) |
| §5.10, §6, §7 | **neste doc** (vivo) |

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

### §3.5 — W4b: a MISTURA do `Output` e a camada filtrada (desenho 11/10 — FEITO, §5.8)

**Medido (W4, §5.7):** com o `Output` em `Blend` = `Add`/`Multiply`/`Screen` o sink mistura EM GRUPO
(doc 118): o cozimento da placa recusa (`RECUSA_MISTURA_EM_GRUPO`), a CPU baixa o sink inteiro para
cópias vectoriais (`lower_group_onto`) e o quadro vai ao Vello — a camada filtrada só recebe a placa,
logo **o filtro não filtra nada**. A foto da `=135` com `PH2D_ACABAMENTO_PASSO=mistura`: discos
chapados, sem relevo nem sombra. Da MESMA família (§0.10): um `Blend` por LINHA (a coluna `blend`,
`FormaComMistura`) e uma geometria que a placa recusa (`Recusada`/`Ausente`) — qualquer UMA manda o
quadro inteiro ao Vello. Na W4 o cartão de cada `Filter` dizia-o (`formas_fora_da_camada`); desde a W4b as
formas estão na camada e só o `Each Shape` avisa (`formas_no_vello`, gate
`shapes_drawn_by_the_vector_path_are_named_on_every_each_shape_card`).

**A lei (estado da arte):** After Effects / Photoshop — os efeitos de uma camada correm na camada
ISOLADA e o modo de mistura da camada aplica-se quando ela POUSA no que está por baixo. As três
arrumações do doc 118 (`Everything` `f(f(B,S₁),S₂)` · `Copies` `f(S₁,S₂) over B` · `Scene`
`f(B, S₁ over S₂)`, em sRGB codificado, W3C) passam, com um `Filter` no fio, a:
- `Scene`: as cópias em `Normal` na camada → a pilha → a camada pousa com `f` (a fórmula de hoje,
  com o filtro no meio — exacta);
- `Copies`: as cópias misturam-se ENTRE SI com `f` dentro da camada → a pilha → `over`;
- `Everything`: a camada isola-se (os efeitos pedem-no) ⇒ `f` entre as cópias e `f` na pousa
  (`f(B, f(S₁,S₂))` — igual a `f(f(B,S₁),S₂)` para Add/Multiply/Screen de alfa cheio, que são
  associativos; com alfa parcial é a lei do Photoshop, não a do Vello: diga-o no tutorial).

**O desenho:** (1) a POUSA com modo: um passe de composição que lê a camada filtrada e uma CÓPIA do
mundo e escreve `f` no espaço do desenhista (a lei de `ph2d-blend-mode`, a mesma do
`layer_composite.wgsl`), no retângulo da cena; oráculo = o `a_mistura_em_grupo_pinta_a_tabela_do_doc_118`
(o `Scene` sem filtro tem de sair AO BYTE igual ao de hoje com a pilha vazia). (2) com filtro, o sink
não agrupa (`lower_group_onto` só sem filtro; a placa aceita-o; a recusa do cozimento cai) e a
`MisturaDoSink` vai à pousa; (3) `Copies`/`Everything` e os `Blend` por linha: o Vello DENTRO da
camada (a cena vectorial do Motion, sem o mundo por baixo, num alvo `Rgba8Unorm` colado com
`BandSource::Vector`) — e com eles o `Each Shape` corre sobre o conjunto (o Vello não tem buffer de
instâncias para as células: o cartão di-lo); (4) as geometrias recusadas entram pelo mesmo Vello.
⚠️ Dois sinks com modos diferentes partilham UMA camada (a pilha é de todo o Motion): a pousa só
tem um `f` — meça e decida (camada por sink, ou a queixa).

## §4 — Waves

| wave | o quê | estado |
|---|---|---|
| W0 | os oráculos corridos (fixtures) · a crate `ph2d-lens` com as leis na CPU e os gates contra as fixtures | ✅ 14 gates verdes |
| W1 | o passe da lente em `ph2d-render` · paridade WGSL × `ph2d-lens` | ✅ §5.2 |
| W2 | o nó `fx.lens` · o cartão · i18n · a costura na shell (o acumulador; o modo em faixas ligado pelo nó) · o `.cube` por nome | ✅ §5.3 (+ a cena `=135` com a lente; o filtro entra na W3) |
| W3 | `fx.filter`: a camada isolada · a ingestão linear da pilha · o passe de composição · o nó derivado | ✅ §5.4 (+ a `=135` com uma sombra projectada) |
| W3b | `scope` (Each Shape · Whole Layer) · o átlas por forma · a semente sub-pixel do campo · o resto da auditoria §5.5 | ✅ §5.6 · smoke do dono APROVADO 11/10 |
| W4 | a cena `=135` (formas + simulação com campos) fotografada · o relógio do grupo · o tutorial 13 | ✅ §5.7 (+ a pilha só na área da cena; a mistura medida e dita no cartão) |
| W4b | a mistura do `Output` com a camada filtrada (§3.5) | ✅ §5.8 (+ o passo (9) do roteiro e a secção 10 do tutorial) |
| W5 | provas de mutação · o handoff | ✅ (handoff de integração) · §5.9 |

## §5 — Medições

### §5.10 — O `Collide` do Output e a cena que saltava (report do dono, 11/10)

*«Por que temos collide em output, e por que muda a imagem se ON mesmo sem colliders?»*

- **Medido na `=135`** (sonda de 180 quadros, posições de cada disco): o passe de separação não
  mexe em NADA sem colisores (ligado desde o quadro 1 = desligado, ao bit), e as duas rotas pintam
  igual ao bit por quadro. O que mudava a imagem era a **troca de rota**: a cerca manda o sink à
  CPU, e a rota HÍBRIDA só marcha na CPU o cone das fronteiras (`ph2d-eval-motion/src/marcha.rs`) —
  ao passar a cozer o grafo inteiro a bomba avançava os laços do dispositivo A PARTIR DA SEMENTE:
  **salto de `0,86`** de uma vez (um quadro anda `0,011`) e `~1` de desfasagem para sempre. A mesma
  família: toda queda da placa a meio (p. ex. `Blend = Add` com `Everything`).
- **Cura (`e30104881`)**: `MotionCookPump::so_o_cone` — depois de uma marcha só do cone, o 1.º
  cozimento inteiro limpa o anel e re-simula da semente até ao tique. Gate
  `the_scene_does_not_jump_when_the_route_leaves_the_device` (placa→CPU no quadro 90 = CPU de ponta a
  ponta, `0,000000`), VERMELHO sem a cura. **Preço**, uma vez por troca (`--release`, `load 56`):
  `0,115 ms` por tique — `0,07 s` a 10 s de animação, `0,4 s` a 1 min, `2 s` a 5 min. ⚠️ Sem engasgo
  pediria trazer o estado do dispositivo à CPU, que não existe (aberto).
- **O `Collide` passa a AUTOMÁTICO** (decisão do dono: *«automático é melhor»*; doc 115 §33): sai
  do `Output`; o `source.object` ganha o seu `Collide` (default desligado, o espelho do da forma) e
  a membrana deixa de declarar; o passe corre sempre (sem colisor é a identidade ao bit); a cerca
  da placa é UMA porta (`peca_declara_colisor`: algum `Collide` resolvido ligado); um projecto antigo
  com o `Collide` do Output ligado abre com os objectos a montante ligados
  (`migra_o_collide_da_saida`, no `MotionState::load_text`). Foto na `=122`: com o `Collide` das
  formas ligado (e o Output intocado) a fileira de cima separa-se e a de baixo, que um campo
  declara «não colide», fica metida.

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

- ~~(W4) A MISTURA do `Output` fora da camada filtrada~~ — curada na W4b (§5.8).
- (W4b) Os filtros de TODOS os fios aplicam-se à camada do Motion inteiro (o `from_graph` concatena
  o fio de cada `Output`): um filtro no fio de uma saída filtra também a outra. Medido: nenhuma cena
  o tem; a cura (uma camada por fio) é a mesma que o item 4 recusou por agora.
- (W4b) `Multiply` dentro da camada com placa (hardware) pediria `DUAL_SOURCE_BLENDING`; hoje o
  `Everything`/`Copies` vai ao Vello (+0,4–1,9 ms, §5.8).
- (W4) O retângulo da camada é o `scene_viewport` (a área onde o mundo se renderiza); a área que o
  artista VÊ (`cena_visivel`, a da lente) é menor quando há painéis por cima — mais barato, por medir.
- (W3) A lei de mistura do `fx.filter` sem ficha (os rótulos são da shell) e a rampa com 2 stops.

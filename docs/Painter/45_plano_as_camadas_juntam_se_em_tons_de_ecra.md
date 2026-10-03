# 45 — Plano: as camadas do Painter juntam-se EM TONS DE ECRÃ (sRGB codificado), não em luz

> **Ordem:** o dono, 03/10 (opção 1). Este doc é o PLANO — nenhuma linha de produto escrita. A
> execução é uma onda própria, numa janela nova (`CLAUDE.md §2`). Nasce da linha `line/sculpt3d`
> (etapa 4, camadas na peça), mas a lei é do Painter INTEIRO (2D e peça).

## 1. A origem — o report e a medição

O dono (03/10, smoke da W1b): *«o traço feito na camada 2 tem qualidade menor que o traço feito na
camada 1»* — a borda de um traço preto numa camada nova sai serrilhada; o mesmo traço na camada de
baixo sai liso. Sonda `camadas_painel::diag_o_traco_numa_camada_e_o_da_base` (o MESMO traço do Painter
na base e numa camada nova por cima, cena `52`, `8x`):

| | pior | mediana | > 16/255 |
|---|---|---|---|
| camada nova contra a previsão «a mesma cobertura misturada EM LUZ» | `0,002` | `0,0009` | `0` de `264` |
| camada nova contra a base | **`0,286`** (73 degraus) | `0,098` | `154` de `264` |

⇒ **não há defeito na pilha**: a camada faz exactamente o que a lei manda. A lei é que é DUAS:

- o PINCEL mistura no espaço CODIFICADO da camada (`ph2d-painter-brush/src/blend.rs:1-12`: *«Blender's
  8-bit texture paint blends in the image's native (encoded, straight-alpha) space»*; `dab/bands.rs`
  `stamp_band`/`encode` — `byte/255`, mistura, `round`);
- o COMPOSITOR mistura em LUZ (`ph2d-tool-painter/src/compositor/mod.rs` `SRGB_DECODE_LUT`/`decode`,
  `compose.rs` `apply_blend` + `encode_byte`; o gémeo `ph2d-render/src/shaders/layer_composite.wgsl`).

Um traço suave numa camada por cima vira cobertura (alfa), e a cobertura junta-se em luz: a
meia-sombra clareia (`0,23 → 0,52` com `w = 0,77`), a transição aperta-se em poucas amostras e, na
peça, a grelha delas vê-se. No 2D é o mesmo desvio, a píxel fino. ⚠️ O doc 01 §7 prescrevia o
contrário (*«o dab deve fazer o mesmo: decode sRGB8→linear…»*) e o pincel nunca o fez.

## 2. A decisão (do dono, 03/10)

**Opção 1:** as camadas juntam-se no espaço CODIFICADO (sRGB), como o Photoshop, o Procreate, o Krita
em 8 bits e o GIMP «perceptual» — um traço numa camada nova fica igual ao mesmo traço na camada de
baixo. (A opção 2, recusada: tudo em luz, pincel incluído — os dois traços iguais, os dois com a borda
dura.) Documentos antigos ABREM (o ficheiro guarda as camadas, nunca o composto) e mudam de aparência
onde há translucidez ou modos — é a funcionalidade, não uma migração.

## 3. A lei nova (a escrever num ADR — o próximo número LIVRE, que se conta na integração — 1.º passo da onda)

`decode(b) = b / 255` · mistura W3C (os 22 modos) sobre esses valores · `encode(v) = round(v · 255)`.
Máscara, recorte, opacidade e grupos são COBERTURA e não mudam. O `lum` dos modos não separáveis
(`0,30 · 0,59 · 0,11`) **fica**: é o do W3C e do Photoshop, aplicado precisamente a valores
codificados (⛔ o levantamento de 03/10 propôs trocá-lo por `0,2126/0,7152/0,0722` — recusado: isso
é luma LINEAR Rec.709, o oposto da decisão).

## 4. O alcance (levantamento de 03/10 — confira no código antes de confiar)

| peça | o que muda |
|---|---|
| `ph2d-tool-painter/src/compositor` (a REFERÊNCIA) | `decode`/`encode_byte`/as três LUTs saem; `composite_below` semeia em codificado; os 2 gates das LUTs saem, os ~24 de composição re-pinam |
| `ph2d-render` `layer_composite.wgsl` + `layer_compositor` (o gémeo) | a ligação do LUT de decode sai; o encode final vira `round`; ~37 gates com placa re-pinam; a paridade CPU↔GPU continua (pode ficar AO BIT: sem `pow` no caminho) |
| ajustes (`ph2d-painter-effects/src/adjustments/compute`) | os que já são de ecrã (Curves, Levels, Posterize, Threshold, Invert, ColorBalance, SelectiveColor, ChannelMixer) deixam de converter; os definidos em luz/OKLab (HSB, Exposure, Vibrance, …) convertem NA FRONTEIRA deles — cada um MEDIDO: um ajuste neutro tem de ser no-op ao bit nos dois espaços |
| efeitos de vizinhança (Gaussian, Sharpen, Bloom, Shadows/Highlights, motion, chroma) | o espaço do kernel é uma decisão por efeito, pelo ORÁCULO (§5) — o Photoshop borra no espaço do documento |
| outros consumidores do mesmo compositor/lei | pré-visualização e Apply do Painter 2D, a peça 3D (`composto_na_placa`, `tinta_achata` — o achatar continua igual), bake de sprites, `ph2d-flip-render` (`composite_blend`), a pilha de FX raster do Vector (`BLEND_MODES_WGSL`), as camadas Wet Paint / Watercolor / Composite (`docs/Painter` 30–43: confirmar que nenhuma lei física deles assumia luz no compositor) |
| documentos | nenhuma migração: o `.ph2dproj` guarda camadas, não compostos (`tool/persist.rs` `PaintedDocument`) |

## 5. O oráculo (DIRETIVA §1: corre-se, não se lê)

**GIMP 3.2.6** (`pacman -Qi gimp`: GPL-3.0 ⇒ só se CORRE, caixa-preta, nunca o fonte) — cada camada tem
*Blend space* e *Composite space* «Perceptual» vs «Linear»: é EXACTAMENTE a nossa pergunta. Por script
(`gimp -i --batch-interpreter=python-fu-eval`), sobre entradas NOSSAS: base + camada com rampa de alfa
× cada modo × opacidades, em «Perceptual» (o alvo) e em «Linear» (o CONTROLO: tem de reproduzir o
compositor de HOJE ±1 — se não reproduz, a régua não vale). Saída gravada como fixtura com cabeçalho
(versão, comando, data); cada corrida vira gate. Krita 6 (GPL, `kritarunner`) como 2.ª opinião onde o
GIMP divergir do W3C (o Soft Light tem duas fórmulas no mundo).

## 6. As ondas, os gates e a desistência

| onda | entrega | gate red-first | desiste-se desta forma se… |
|---|---|---|---|
| **P0** | o ADR; o oráculo a correr; o CONTROLO «Linear» reproduz o compositor de hoje ±1 | as fixturas com cabeçalho | o controlo não reproduzir hoje (a régua mente) |
| **P1** | Normal + opacidade + máscara + recorte + grupos em codificado, CPU e placa | (a) o oráculo «Perceptual» ±1; (b) **o seam do report: o traço numa camada nova = o mesmo traço na base** (a sonda vira gate, ±1); (c) CPU↔placa | o (b) não fechar a ±1 — então o desvio não é só o espaço |
| **P2** | os outros 21 modos | o oráculo, modo a modo (diferenças W3C×GIMP documentadas, não «aceites») | — |
| **P3** | os ajustes, fronteira por fronteira | neutro = no-op ao bit · CPU↔placa · o oráculo onde o GIMP tem o mesmo ajuste | — |
| **P4** | os efeitos de vizinhança e os outros consumidores (Flip, FX do Vector, sprites, Wet/Composite); re-pinar; doc 01 §7 corrigido | o oráculo por efeito; os gates de cada consumidor | — |

Mutação: um arnês por porta (decode/encode, a mistura, cada fronteira de ajuste), com pré-voo.

## 7. Riscos e colisões

- **`line/PainterWatercolor`** está aberta (sem commits no compositor a 03/10): o integrador funde por
  cima; a lei é UMA, e os gates dela que fixarem compostos re-pinam no mesmo passo.
- **A aparência de documentos 2D existentes muda** onde há translucidez entre camadas ou modos — o
  smoke do dono tem de o MOSTRAR (um documento antigo aberto antes e depois), não esconder.
- O custo baixa (sai o `pow`/LUT de decode por píxel); medir no fecho, não prometer.

## 8. Execução (P0–P2, 03/10) — o que cada onda mediu

**P0 — a régua** ([ADR-0177](../architecture/decisions/0177-as-camadas-do-painter-juntam-se-em-tons-de-ecra.md),
[oráculo](ferramentas/oraculo_camadas_gimp/README.md)). GIMP 3.2.6 por `python-fu-eval`: 84 corridas.
O CONTROLO passa — o «linear» do GIMP é o compositor de então a `0`/`≤1` nos 10 modos de fórmula
partilhada (Normal, Multiply, Darken, Lighten, Screen, Overlay, HardLight, VividLight, Difference,
Exclusion) — e a W3C em tons de ecrã fecha o «perceptual» a `≤1` nesses 10. Hoje × «perceptual»:
`73` degraus no Normal, o número do report.

**P1 — a lei, CPU e placa.** `decode = b/255`, `encode = round`; os ajustes e os efeitos de
vizinhança, definidos em luz, recebem `luz(acc)` e devolvem `ecrã(resultado)` (a fronteira; a mistura
de volta corre em ecrã) — na CPU no braço do ajuste, na placa no `apply_adjustment_op` e nos 1.ºs
leitores/combines do grafo.

| gate | antes | depois |
|---|---|---|
| (a) compositor × GIMP «perceptual», 10 modos | `73` | `≤1` |
| (b) peça 3D, cena 52: traço numa camada nova × o traço na base | `0,286` (73 degraus) | **`0,000`** |
| (b) gémeo 2D (`o_traco_numa_camada_nova_e_o_traco_na_base`) | `42` | um dab `≤1`; um traço `≤2`, raro |
| (c) CPU↔placa, pilha rica `8x` | `23` de `188 424` bytes a 1 | `11`–`12`; pior `31` de `141 318` canais; nunca 2 |
| ajuste NEUTRO (18 tipos com neutro) | — | `0` bytes, base opaca e translúcida |

⚠️ **Premissas do plano que a medição derrubou:**

1. **«o traço numa camada nova = o traço na base, ±1»** só vale por DAB. Num traço os dabs
   sobrepõem-se e cada um arredonda a 8 bits — a base guarda a COR, a camada guarda o ALFA, e os
   erros propagam-se diferente. O modelo só das duas recursões arredondadas (sem lei de composição
   nenhuma) reproduz o produto: 1 dab `≤1` sempre; 2+ dabs, 2s em `0,006–0,03 %`, nunca 3. É a
   quantização de qualquer pintor a 8 bits, não o espaço (o critério de desistência não se aplica:
   o espaço fechou, e na peça 3D fecha ao bit).
2. **«a ligação do LUT de decode sai»** — FICA, com o conteúdo novo (`b/255` calculado na CPU): o
   WGSL não promete que a conversão `rgba8unorm → f32` seja a divisão correctamente arredondada em
   todo driver; a tabela promete.
3. **«sem `pow` no caminho a paridade pode fechar ao bit»** — não fecha: a divisão do `over` (`2,5
   ULP` no WGSL) continua. Desce para metade; o `FRACCAO_A_UM_DEGRAU` da W1b fica.
4. **A aquarela assumia LUZ no compositor** (era item da P4, chegou na P1 por um gate vermelho,
   `watercolor_ground_is_the_real_backdrop`, 73 degraus): o «base sobre o chão», a franja de AA e o
   des-premultiplicar `L = (aparência − chão·(1−a))/a` imitam o compositor e passaram a tons de ecrã;
   a óptica Beer–Lambert continua em luz. Com base opaca o caminho velho `l2s_byte(s2l[b])` nem era a
   identidade (descia 1 em 7 bytes escuros): pino `watercolor_aa` movido (405 bytes, todos por 1).

**P2 — os modos, dois oráculos.** O Krita 6.0.4 a 8 bits (codificado; `kritarunner` sem interface)
é o compositor novo nos **22** modos — os HSL do W3C, Soft Light SVG, Behind e Erase=Clear incluídos
— a `≤4` degraus em `≤93` canais de `4 896` (o arredondamento inteiro dele). As divergências do GIMP
são fórmulas nomeadas, com desvio `0`: `B` SEM corte a `[0, 1]` em vírgula flutuante (Add,
ColorBurn, ColorDodge, LinearBurn, LinearLight) e o Soft Light Pegtop; os «HSL» dele são HSV/HSL.
**Nenhuma fórmula nossa muda** (W3C = Photoshop = Krita). Gates: `oraculo_gimp_tests`.

**Mutação:** [`muta_as_camadas_em_ecra.sh`](ferramentas/muta_as_camadas_em_ecra.sh), 17 mutações
(decode, encode, as duas fronteiras, a aquarela; na placa a tabela, o encode final, a fronteira por
píxel e a do grafo).

**P3 — cada ajuste na SUA fronteira (03/10).** O contrato de `ph2d_painter_effects::adjustments`
(`apply_adjustment`, `apply_adjustment_windowed` e cada kernel público) é o acumulador CODIFICADO;
a fronteira saiu do compositor e do `apply_adjustment_op` da placa para dentro de cada tipo
(conferido tipo a tipo no código):

| espaço | tipos | como |
|---|---|---|
| ecrã (sem conversão) | Curves, Levels, Posterize, Threshold, Invert, Color Balance, Selective Color, Channel Mixer, Color Lookup, Black & White, Noise, Halftone, **Shadows/Highlights** | os valores como estão; o S/H era «de luz» na lista do plano e é de ecrã de ponta a ponta (luma, retoque e o blur escalar da luma) |
| luz / OKLab | HSB, Vibrance, Photo Filter (por píxel) · Exposure, Brightness/Contrast (função 1-D com a fronteira dobrada numa tabela: zero transcendentais por píxel) · o tinte do B&W · o brilho do Bloom | `shared::{em_luz, em_tons_de_ecra, build_lut_em_luz}` |
| misto | Gradient Map | a luma lê o codificado; o gradiente interpola em luz (a tabela), codificada por entrada |
| kernel de vizinhança | Gaussian, Sharpen, Motion, Chroma, Bloom | UMA porta, `premultiply`/`unpremultiply`, passa a luz e volta — é ali que a P4 decide |

Medido (sonda `diag_o_composto_de_cada_ajuste_nao_neutro`, 24 tipos × base opaca/translúcida, cada
ajuste a mover ~9 000 de 12 288 bytes): **47 de 48 compostos iguais ao byte**; o Invert sobre base
opaca muda 12 bytes por 1 — o novo é o `1 − x` exacto (`invert_is_the_exact_display_negative_at_every_byte`).

**O oráculo dos ajustes** ([`oraculo_ajustes.py`](ferramentas/oraculo_camadas_gimp/oraculo_ajustes.py),
8 corridas): a grelha dos modos composta em «perceptual», o ajuste do GIMP sobre o visível. Curves e
Levels expõem `trc` (via `Gimp.DrawableFilter`), o Invert `linear` — três controlos reais.

| ajuste | nosso × GIMP «perceptual» | controlo: GIMP «linear» × o ajuste em luz | nosso × «linear» |
|---|---|---|---|
| Invert | `0` | `0` | `120` |
| Curves (recta `0,1037 → 0,9113`) | `1` (3 de 4 896 canais) | `1` | `65` |
| Levels | `2` (a tabela de 256 junto do ponto preto; a função exacta dá `0`) | `0` | `60` |
| Posterize 4 | `0` (cor) | — | — |
| Threshold 128 | `0` (cinzentos) | — | — |

Divergências NOMEADAS do GIMP (gateadas para avisar se mudarem): o Threshold dele deixa o branco puro
FORA do intervalo `[low, high]` (`high` não passa de 1); o Posterize dele quantiza também o ALFA.
⚠️ Armadilha medida: uma curva `0,1 + 0,8·x` cai em empates de meio degrau (`25,5 + 0,8·k`) e o byte
decide-se pelo ruído de vírgula flutuante de cada programa (87 canais a 1) — pontas sem empate.

⚠️ **Dois defeitos que o oráculo expôs (CPU e placa):**
1. **A mistura de volta do ajuste era um `over` com o alfa da base** — um ajuste a 100 % sobre um
   píxel translúcido aplicava-se só em parte (`a = 0,5` ⇒ ⅔): 79 degraus do GIMP. Hoje a cor ajustada
   mistura-se com a da base como se as duas fossem opacas e a cobertura fica (o combine do S/H na placa
   já o fazia no Normal). Muda a aparência dos ajustes sobre zonas translúcidas — a funcionalidade.
2. **O Threshold cortava em `t/255`**: um cinzento EXACTO no limiar saía preto (os pesos Rec.601 somam
   um nadinha abaixo de 1 em `f32`). Hoje é a regra de 8 bits do Photoshop, o byte da luma `≥ t`.

Mutação: [`muta_as_camadas_em_ecra.sh`](ferramentas/muta_as_camadas_em_ecra.sh) reescrito para as
fronteiras da P3 — 42 mutações (cada tipo na CPU e na placa, a porta dos blurs, a mistura de volta,
o corte do Threshold): **42/42**. A 1.ª corrida deu 38: três sobreviveram a gates RELACIONAIS
(«cinzento fica cinzento», «o contraste espalha») ou a um ponto que não separa as regras — gates
novos `vibrance_desaturates_to_the_oklab_lightness_of_the_light`,
`contrast_pivots_on_the_mid_gray_of_the_light` e o `127,75` no
`gpu_o_threshold_e_o_byte_da_luma_contra_o_limiar` (na placa a luma do `128` exacto não cai abaixo
do corte: só um cinzento entre `127,5` e `128` distingue); a 4.ª não compilava (o arnês). `MUTA_FILTRO`
re-corre só algumas.

**Fica para a P4 (janela nova):** o espaço do kernel de cada efeito de vizinhança pelo oráculo (hoje:
luz, pela porta dos blurs), e os outros consumidores: `ph2d-flip-render` (`composite_blend`), o FX
raster do Vector (`BLEND_MODES_WGSL`), o bake de sprites, Wet Paint / Composite (a aquarela já está),
o doc 01 §7 e a tabela ⛔ Recusas MEDIDAS. A tabela de 256 do Levels (2 degraus junto do ponto preto)
fica nomeada.

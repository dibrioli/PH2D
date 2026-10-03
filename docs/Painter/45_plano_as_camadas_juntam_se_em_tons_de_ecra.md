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

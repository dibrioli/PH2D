# ADR-0177 — As camadas do Painter juntam-se em tons de ecrã (sRGB codificado), não em luz

- **Status:** Accepted
- **Data:** 2026-10-03
- **Linha:** `line/sculpt3d` (a lei é do Painter INTEIRO — 2D e a peça 3D)
- **Nota (2026-10-05):** a parte «peça 3D» deste texto é **histórica** — o 3D saiu do PH2D ([ADR-0179](0179-o-3d-sai-do-ph2d.md)); a lei continua válida no Painter 2D.
- **Decisão do dono:** 2026-10-03, «opção 1»
- **Plano e ondas:** [doc Painter 45](../../Painter/45_plano_as_camadas_juntam_se_em_tons_de_ecra.md)
- **Oráculo:** [`docs/Painter/ferramentas/oraculo_camadas_gimp/`](../../Painter/ferramentas/oraculo_camadas_gimp/README.md)
- **Corrige:** o [doc Painter 01 §7](../../Painter/01_arquitetura_e_decisoes.md), que prescrevia luz e o pincel nunca seguiu

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui. A 03/10 o `main` acabava no `0175` e a `line/3DModeling` já tinha um
> `0176` por integrar.

## Contexto

Report do dono no smoke da W1b (03/10): *«o traço feito na camada 2 tem qualidade menor que o traço
feito na camada 1»*. Medido pela sonda `camadas_painel::diag_o_traco_numa_camada_e_o_da_base`
(`ph2d-app-sculpt3d`): a camada nova é EXACTAMENTE a mesma cobertura misturada em luz (pior `0,002`),
e difere do mesmo traço feito na base até `0,286` (**73 degraus** de sRGB8). Não há defeito na pilha
— há DUAS leis:

| quem | onde mistura | onde está |
|---|---|---|
| o pincel | no espaço CODIFICADO da camada (`byte/255`, mistura, `round`) | `ph2d-painter-brush/src/blend.rs` · `dab/bands.rs` |
| o compositor de camadas | em LUZ (decode sRGB → linear, W3C, encode) | `ph2d-tool-painter/src/compositor` · o gémeo `ph2d-render` `layer_composite.wgsl` |

Um traço suave numa camada por cima vira cobertura (alfa), e a cobertura junta-se em luz: a
meia-sombra clareia, a transição aperta-se em poucas amostras, e na peça a grelha delas vê-se.

## A régua (P0) — o GIMP corrido, não lido

GIMP 3.2.6 (`pacman -Qi gimp` ⇒ GPL-3.0-or-later ⇒ só se CORRE, caixa-preta) tem por camada
*blend space* e *composite space* «Perceptual» × «Linear» — exactamente a pergunta. Corrido sem
interface (`python-fu-eval`) sobre entradas NOSSAS: base (opaca e a `140/255`) + UMA camada com
rampa de alfa (17 degraus) × 6 × 6 cores × 21 modos × 2 opacidades × 2 espaços = 84 corridas,
fixtura `gimp_3.2.6.bin` com cabeçalho. Gate `compositor::oraculo_gimp_tests`; sonda
`diag_o_compositor_contra_o_gimp` (pior desvio em degraus; entre parênteses, canais a mais de 1, de
`4 896`):

| modo (os 10 de fórmula partilhada) | compositor de hoje × GIMP «Linear» | W3C em tons de ecrã × GIMP «Perceptual» | hoje × «Perceptual» |
|---|---|---|---|
| Normal | `0` | `0` | `73` (`2 475`) |
| Multiply · Darken · Lighten | `0` | `0` | `73` |
| Screen · Overlay · HardLight | `≤ 1` | `≤ 1` | `73`–`105` |
| VividLight · Difference · Exclusion | `0` | `0` | `76`–`209` |

⇒ **o controlo passa** (o GIMP «Linear» reproduz o compositor de hoje, logo a régua mede o que diz)
e **a lei nova fecha a ±1** contra o «Perceptual» em todo modo de fórmula partilhada. A régua de duas
camadas do teste reproduz o compositor de produção nas 84 corridas (as colunas «compositor» e «em
luz» da sonda são iguais).

Os outros 11 divergem do GIMP **nos dois espaços** — é fórmula, não espaço, e é a P2: o GIMP em
vírgula flutuante não parece cortar `B(Cb, Cs)` a `[0, 1]` (Add, ColorBurn, ColorDodge, LinearBurn,
LinearLight — hipótese a medir), o Soft Light dele é outra fórmula, os seus «Hue/Saturation/Color/
Luminance» são HSV/HSL/luminância e não o `SetLum`/`SetSat` do W3C, e o `ERASE` não é o nosso
`Clear`. O `BEHIND` é modo só de pincel no GIMP (a camada volta a Normal): sem oráculo.

## Decisão

As camadas juntam-se no espaço CODIFICADO, como o Photoshop, o Procreate, o Krita em 8 bits e o GIMP
«Perceptual»:

```text
decode(b) = b / 255        mistura = W3C Compositing L1 (os 22 modos), straight, sobre esses valores
encode(v) = round(clamp(v, 0, 1) · 255)
```

- Máscara, recorte, opacidade e grupos são COBERTURA e não mudam (já eram `byte/255`).
- O `lum` dos modos não separáveis **fica** `0,30 · 0,59 · 0,11` — o do W3C e do Photoshop, aplicado
  precisamente a valores codificados.
- O pincel NÃO muda (ele já mistura em codificado): um traço numa camada nova fica igual ao mesmo
  traço na camada de baixo.
- Os ajustes definidos em luz ou OKLab (HSB, Exposure, Vibrance, …) convertem NA FRONTEIRA deles; os
  que já são de ecrã (Curves, Levels, Posterize, Threshold, Invert, ColorBalance, SelectiveColor,
  ChannelMixer) deixam de converter. Cada um medido: ajuste neutro = no-op ao bit.
- O espaço do kernel de cada efeito de vizinhança decide-se pelo oráculo, efeito a efeito (P4:
  tons de ecrã pré-multiplicados; o Bloom, óptico, em luz).
- A lei é do PAINTER: o compositor partilhado (`ph2d-render` `LayerCompositor`) recebe o espaço do
  produto que o chama (`CompositeSpace`) — o Flip junta as camadas em luz, como os traços dele.

## Alternativas rejeitadas

| alternativa | porquê não |
|---|---|
| **Opção 2: tudo em luz, o pincel incluído** | os dois traços ficariam iguais, os dois com a borda dura que o dono reportou; e nenhum app de pintura de referência a 8 bits faz assim (o GIMP «Linear» é a excepção configurável) |
| Deixar as duas leis | é o defeito do report: a camada nova não é a camada de baixo |
| Trocar o `lum` por `0,2126 · 0,7152 · 0,0722` (proposta de um levantamento automático, 03/10) | é luma LINEAR Rec.709 — o oposto desta decisão; o W3C e o Photoshop aplicam `0,30/0,59/0,11` a valores codificados |

## Consequências

- **Documentos antigos ABREM** (o `.ph2dproj` guarda camadas, nunca o composto) e mudam de aparência
  onde há translucidez entre camadas ou modos. É a funcionalidade, não uma migração; o smoke do dono
  MOSTRA a mudança.
- Saem a `SRGB_DECODE_LUT` e as duas tabelas de encode do compositor de referência, e com elas os 2
  gates delas; os gates de composição que fixam compostos re-pinam com o número ao lado.
- No gémeo da placa sai a ligação do LUT de decode e o encode final vira `round`; sem `pow` no
  caminho a paridade CPU↔placa pode ficar ao bit — mede-se, não se promete.
- Consumidores da mesma lei: pré-visualização e Apply do Painter 2D, a peça 3D (`composto_na_placa`;
  o achatar sobre o fundo não muda), o bake de sprites, `ph2d-flip-render` (`composite_blend`), o FX
  raster do Vector (`BLEND_MODES_WGSL`), as camadas Wet Paint / Watercolor / Composite.

## Execução (P0–P3, 03/10)

Medido e gateado — detalhe e premissas derrubadas no [doc 45 §8](../../Painter/45_plano_as_camadas_juntam_se_em_tons_de_ecra.md):
o oráculo «perceptual» fecha a `≤1` (era `73`); o traço numa camada nova da peça 3D é o da base **ao
bit** (era `0,286`); no 2D um dab `≤1` e um traço `≤2` raro (quantização de 8 bits por dab, não o
espaço); CPU↔placa a um degrau em `0,006 %` dos bytes (a divisão do WGSL — não fecha ao bit); os 18
ajustes com ponto neutro são no-op ao bit; o Krita a 8 bits é o compositor novo nos 22 modos, e as
divergências do GIMP são fórmulas nomeadas (`B` sem corte, Soft Light Pegtop).

**P3 (03/10):** cada ajuste recebe o acumulador CODIFICADO e converte na sua fronteira (tabela por
tipo no doc 45 §8); os blurs atravessam uma porta só (`premultiply`/`unpremultiply`). O composto
fica igual ao byte em 47 de 48 sondas (o Invert passa a ser o `1 − x` exacto). Contra o GIMP
«perceptual», Invert/Curves/Levels/Posterize/Threshold a `0`/`1`/`2`/`0`/`0`, com o controlo «linear»
a ser o ajuste em luz (a `60`–`120` degraus do nosso). O oráculo expôs dois defeitos, corrigidos na
CPU e na placa: a mistura de volta aplicava só parte de um ajuste a 100 % sobre um píxel translúcido,
e o Threshold mandava para o preto o cinzento exacto no limiar.

**P4 (03/10):** os efeitos de vizinhança, pelo oráculo
([`corre_vizinhanca.sh`](../../Painter/ferramentas/oraculo_camadas_gimp/corre_vizinhanca.sh)): o Krita
a 8 bits borra em tons de ecrã pré-multiplicados — o Gaussian dele de raio 9 é o nosso
`gaussian_weights(9)` **ao byte** (0 de 10 800 canais), o Motion a ≤1 —; o GIMP borra em luz (o
controlo: o nosso núcleo em luz fecha-o a ≤1 no miolo, o produto fica a ≥60). Gaussian, Sharpen, Motion
e Chroma passam a tons de ecrã (CPU e placa); o Bloom, óptico, fica em luz na porta dele. Os
consumidores conferidos no código: o Painter 2D e a peça herdam; o Wet Paint, o Composite, o Impasto e o
bake de sprites não compõem o de baixo. **O Flip foi partido pela P1 em silêncio** (o rasterizador junta
os traços de uma camada em luz; as camadas passaram a tons de ecrã — `0,216` onde a mesma camada dá
`0,5`): o espaço passou a ser do PRODUTO que chama (`ph2d_render::CompositeSpace`; o Flip em luz pela
porta `ph2d_flip_render::compositor_do_flip`). O FX do Vector fica em luz (abaixo). Detalhe:
[doc 45 §8 (P4)](../../Painter/45_plano_as_camadas_juntam_se_em_tons_de_ecra.md).

| ⛔ Recusa MEDIDA | porquê |
|---|---|
| trocar o `lum` por Rec.709 linear | o oposto desta decisão (acima) |
| copiar o `B` sem corte do GIMP | é a vírgula flutuante dele; o W3C, o Photoshop e o Krita cortam (gate `as_divergencias_do_gimp_sao_as_formulas_nomeadas`) |
| o Soft Light Pegtop do GIMP | o W3C/SVG é o do Krita `soft_light_svg` e o nosso |
| ler o texel pela conversão `unorm` da textura | o WGSL não a promete correctamente arredondada; a tabela `b/255` sim |
| misturar a cor ajustada de volta por `over` com o alfa da base | um ajuste a 100 % aplicava-se só em parte num píxel translúcido: `79` degraus do GIMP; uma camada de ajuste muda a cor, nunca a cobertura (gates `um_ajuste_de_ecra_e_o_do_gimp_perceptual`, `gpu_um_ajuste_cheio_sobre_um_pixel_translucido_aplica_se_inteiro`) |
| cortar o Threshold em `t/255` | o cinzento exacto no limiar saía preto; a regra é o byte da luma `≥ t` (Photoshop; = o `low = 0,5` do GIMP) |
| o Posterize do GIMP que quantiza o alfa · o Threshold dele que deixa o branco de fora | convenções dele, não espaço; um ajuste não toca a cobertura e o branco passa o limiar (gateadas como divergências nomeadas) |
| borrar em luz (o GIMP, e o nosso até à P3) | o Krita a 8 bits — e o Photoshop/Procreate que o dono nomeou — borram no espaço do documento; o Krita é o nosso ao byte (gate `the_gaussian_is_kritas_8_bit_blur_to_the_byte`); em luz a meia-sombra de um degrau preto↔branco clareia 73 degraus |
| o Bloom em tons de ecrã com os outros desfoques | é óptico (luz somada) — a mesma família do Exposure; o brilho já era de luz (gate `the_bloom_stays_a_glow_of_light`, sobre cinzento: 0 e 1 são pontos fixos da curva) |
| a lei do Painter no FX raster do Vector | o mundo vetorial compõe em luz de ponta a ponta (o Vello: âmbar a meia cobertura `(173,128,41)` = `encode(0,5·linear)`, em tons de ecrã `(118,88,30)`) — a lei aqui abriria lá a costura do report |
| o Flip a herdar o compositor em tons de ecrã | o rasterizador dele junta os traços de uma camada em linear 16F: a camada de cima tem de se juntar como um traço na mesma (gate `a_camada_de_cima_junta_se_como_um_traco_na_mesma_camada`; `0,216` contra `0,5`) |
| um compositor em luz que aceite ajustes | os ajustes são definidos contra o acumulador codificado (P3) — seriam errados em silêncio; recusa-se (`LayerCompositeError::AdjustmentInLightSpace`) |

## Critério de desistência

Se o seam do report (o traço numa camada nova = o mesmo traço na base, ±1) não fechar na P1, o
desvio não é só o espaço — PARA-se, mede-se e reporta-se ao dono antes de uma 2.ª tentativa. (O da
P0, o controlo «Linear», passou: tabela acima.)

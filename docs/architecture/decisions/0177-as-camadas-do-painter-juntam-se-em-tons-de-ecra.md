# ADR-0177 — As camadas do Painter juntam-se em tons de ecrã (sRGB codificado), não em luz

- **Status:** Accepted
- **Data:** 2026-10-03
- **Linha:** `line/sculpt3d` (a lei é do Painter INTEIRO — 2D e a peça 3D)
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
- O espaço do kernel de cada efeito de vizinhança decide-se pelo oráculo, efeito a efeito.

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

## Critério de desistência

Se o seam do report (o traço numa camada nova = o mesmo traço na base, ±1) não fechar na P1, o
desvio não é só o espaço — PARA-se, mede-se e reporta-se ao dono antes de uma 2.ª tentativa. (O da
P0, o controlo «Linear», passou: tabela acima.)

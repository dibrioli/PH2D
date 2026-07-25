# Doc 68 — `value.curve`: o shaper do domínio de VALOR (nota-ADR)

**Data:** 2026-07-25 · **Linha:** `line/motion-value` (reaberta pós-integração) · **Modo:** L

## O que é

`value.curve` — passa um valor por uma **curva desenhada** (transfer arbitrário), o gêmeo
de forma-livre do `value.map_range` (que é o remap LINEAR). Um nó de VALOR unário
(`(Instances, Scalar, Frame)` no `v`), `Pure`, HR-5.

## Pesquisa (regra-ouro — porto por SEMÂNTICA, não por código)

O padrão-ouro da indústria para "moldar um valor por uma curva editável" converge:

| App | Nó / recurso |
|---|---|
| **Blender** | **Float Curve** (`ShaderNodeFloatCurve`) — valor → curva editável → valor |
| **Houdini** | o parâmetro **ramp** (`chramp`) — a rampa como transfer |
| **Cinema 4D** | o **Spline** mapper / o Range Mapper com spline |
| **Cavalry** | a **Value Graph** de qualquer atributo |
| **After Effects** | o graph editor de expressão; a Curves |

A semântica comum: normaliza a entrada para `[0,1]`, passa pela curva, escala para a saída.

## As decisões

1. **Reusa o A1 inteiro, não reinventa.** A curva é o `ph2d-curve` (crate leaf), autorada
   pelo `ParamWidget::Curve` (o editor arrastável), e o **mesmo transfer** que o contour
   **Curve** do `field.remap` roda — só que na coluna `v` em vez da máscara `falloff`. O
   `shape_one` é o `contour(4, …)` do field.remap portado para o domínio de valor.

2. **O superset de faixa (produto final, não MVP).** `in_lo`/`in_hi`/`out_lo`/`out_hi` —
   o `fit()` do Houdini **mais** a curva. `t = clamp((v − in_lo)/(in_hi − in_lo), 0, 1)`,
   `s = curve.eval(t)`, `out = out_lo + s·(out_hi − out_lo)`.

3. **Curva não-desenhada = IDENTIDADE.** Um nó recém-largado é *exatamente* o
   `value.map_range` com clamp ligado — uma reta que você depois **entorta**. É o que o
   torna seguro largar no grafo antes de desenhar (`curve = None` → `eval(t) = t`).

4. **100% GPU-resident, sem fallback.** O canal de **LUT** do A1-gpu (o 6º side-metadata do
   `KernelResolver`, `luts`) baka a curva num buffer que a WGSL amostra (`vc_curve_sample`).
   Sem gate `applicable` — o sequenciador nunca cai pra CPU (o norte "maximize GPU"). O
   `NodeManifest` fica intacto (a curva é text param, a LUT é side-metadata — §6 conferido).

## Alternativas rejeitadas

- **Um param `clamp` separado** (como o `value.map_range` tem, para o `efit`/extrapolação):
  **descartado** — o domínio da curva **é** `[0,1]`, então `t` é sempre clampado antes do
  `eval` por construção. Um "clamp off" que extrapolasse leria a curva fora do domínio dela.
  O output pode sair de `[out_lo, out_hi]` só se o artista arrastar um ponto da curva acima de
  1 ou abaixo de 0 — o que é a **feature** (overshoot autorado), não um modo.
- **Um fallback pra CPU** (como o `field.remap` teve no A1-core antes do A1-gpu): **desnecessário**
  — o canal de LUT já existe no main, então o nó nasce device-resident.
- **Reamostrar a curva por instância na WGSL** (sem LUT): seria `eval` transcendental-lite por
  texel; a LUT (256 nós + lerp) troca isso por 1 amostra e um upload de 1 KiB/frame.

## O preço (medido no gêmeo `field.remap`)

- A LUT é reconstruída por frame (parse + 256 `eval`, sub-µs) para todo `value.curve`. Um cache
  por-string é otimização futura, não necessária (mesma decisão do `field.remap`).
- Paridade CPU↔GPU dentro de ε ~4e-3 no pior caso (o canto de um pico de tent entre 2 amostras).

## Demo

`PH2D_VALUE_CURVE_SMOKE=1` — a curva vira o **perfil espacial** de uma fileira:
`grid → move → drive(Y)` com o valor `instance_field(Ramp) → value.curve`. De cima um TENT
(`0→1→0`) → a fileira **arqueia**; de baixo a MESMA `value.curve` sem curva → uma **rampa**
reta. Selecione o `value.curve` de cima → o editor arrastável aparece no painel; arraste e o
arco muda ao vivo.

# Doc 85 — `motion.color_ramp` Custom: o editor de GRADIENTE (nota-ADR)

**Data:** 2026-07-29 · **Linha:** `line/motion-value` (reaberta pós-integração) · **Modo:** L

## O que é

O `motion.color_ramp` Custom deixa de ser **seis sliders crus `0..1`** (`a_r`…`b_b`, dois stops
fixos) e vira um **gradiente MULTI-STOP** autorado por um editor direto no painel — a barra, os
stops arrastáveis, um swatch OKLCH por stop, `+`/`−` e o interp. É o **item I9 do plano 63**
(§7 W-I, ordem do Enio 2026-07-25) e o **irmão de COR do editor de curva** (doc 68, A1): a
mesma arquitetura *text param → editor arrastável → LUT de GPU*, na coluna `tint` em vez da
máscara.

## Pesquisa (regra-ouro — porto por SEMÂNTICA)

O padrão-ouro para "editar um gradiente multi-stop" converge:

| App | Recurso |
|---|---|
| **Blender** | Color Ramp — barra, stops arrastáveis, swatch por stop, dropdown de interpolação |
| **Houdini** | o parâmetro **ramp** (cor) |
| **After Effects** | Gradient Ramp / o editor de gradiente |
| **Cavalry** | o Gradient attribute |

A semântica comum: uma lista de stops `(posição, cor)` + um modo de interpolação global; o
artista arrasta os stops e escolhe as cores num picker.

## As decisões

1. **O gradiente É um `ph2d_color::ColorRamp` serializado num TEXT param** (doc 32) — nunca
   `ParamSpec` (uma lista de comprimento variável não é um conjunto fixo de `f32`). Formato
   compacto `g1 <interp_u8> <pos>:<r>,<g>,<b> …` (`color_ramp_text.rs`), espelho do `c1 …` da
   curva. O interp GLOBAL viaja NA string porque **o fill da LUT da GPU só vê a string** — o
   interp tem de viajar com os stops ou o bake do device não casaria com o `eval` da CPU.

2. **`ParamWidget::Gradient`** (side-metadata do registry, **contrato congelado intacto** —
   `NodeOp=2`/`OpResolver=1`/`NodeManifest=8`), o irmão do `::Curve`. O painel desenha a barra
   + marcadores de posição (o MESMO `InteractiveState::CurvePoint` x-drag da curva, y ignorado)
   + um swatch OKLCH por stop (`register_picker_swatch`).

3. **A COR de um stop vai ao DEVICE por 3 LUTs escalares** (r/g/b), via o canal `luts` que a
   curva já criou. Uma LUT de cor **é** três LUTs escalares ⇒ **zero mudança foundational** de
   GPU (o `LutSpec` continua escalar). A branch Custom da WGSL amostra as 3; os presets ficam
   inline (stops constantes). Alpha implícito 1.0 (stops opacos — todo preset já é).

4. **Curva não-desenhada = default preto→branco.** Um `color_ramp` Custom recém-largado com
   string vazia/malformada abre no `ColorRamp::default()` — seguro largar antes de editar.

5. **A POSIÇÃO/add/remove/interp saem por `SetTextParam`** (como a curva, painel-side); a **COR
   sai pela pinça OKLCH do bridge** (como a `ColorRow`, re-serializando a string). Uma edição,
   dois canais — cada um pelo mecanismo que já existe.

## Alternativas rejeitadas

- **Estender o `LutSpec` para carregar vec4** — desnecessário: 3 LUTs escalares dão a mesma
  coisa reusando o canal existente, sem tocar o substrato de GPU. *A representação apaga o caso
  especial.*
- **Manter os 6 sliders + só adicionar swatch** — o pedido é MULTI-stop; 2 stops fixos seriam o
  MVP que o D13 proíbe.
- **Per-stop alpha** — diferido: os stops do `color_ramp` são opacos (a cor da instância). Um
  alpha por stop é um campo append-only futuro no formato, não uma reinterpretação da string.
- **Um fallback pra CPU** — desnecessário; o canal de LUT já existe no main, o nó nasce
  device-resident. Presets exatos (`<1e-5`), Custom dentro do ε da LUT (`6e-3`, a convenção do
  `field.remap` Curve).

## O preço (medido)

- A LUT (3× 256 amostras) é reconstruída por frame para todo `color_ramp` Custom (parse + bake,
  sub-µs) — a mesma decisão do `value.curve`; um cache por-string é otimização futura.
- Paridade CPU×GPU na RTX: presets 0..3 exatos, Custom dentro de `6e-3` (o corte do canto do
  stop por ~um passo de amostra da LUT, o trade documentado).

## Onde vive

- **`ph2d-color`**: `color_ramp_text.rs` (`serialize_gradient`/`parse_gradient`).
- **`ph2d-node-registry`**: `ParamWidget::Gradient`.
- **`ph2d-node-motion-color-ramp`**: Custom lê o text param; 3 `LutSpec` (`cr_grad_r/g/b`); a
  WGSL Custom amostra as LUTs.
- **`ph2d-panel-motion-params`**: `gradient_row.rs` (o editor) + `shaper_dispatch.rs` (o
  despacho drag/click da curva E do gradiente).
- **shell**: `motion_bridge_params.rs` monta a `GradientRow`; `motion_bridge_color.rs` faz a
  pinça (`picker_session`/`apply_picker_readback`/`gradient_picker_stop`/`apply_gradient_stop_pick`).

## Smoke

`env PH2D_GRADIENT_SMOKE=1 cargo run -p ph2d-host-desktop --release` — uma fileira de 24 pontos
num sweep vermelho→verde→azul, o `Color Ramp` selecionado, o editor de gradiente no painel.
Arraste um marcador (re-colore ao vivo), clique um swatch (o picker OKLCH abre), `+`/`−`
adiciona/remove. Roda igual com `PH2D_GPU_COOK=1` (default) e `=0`.

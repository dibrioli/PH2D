# Aquarela PH2D — Umidade visível, diluição física e secagem viva

> Como foram construídos os três efeitos que fazem a aquarela do PH2D parecer *água de verdade
> sobre papel*: o **preview de umidade** (véu escuro + menisco brilhante), o **sistema de
> diluição** (pigmento dissolvido em água, não "opacidade"), e a **secagem viva** (a tinta seca
> mais claro, escurece na borda e granula no dente do papel). Documento de engenharia + estética:
> cada efeito tem a física, a fórmula exata, a decisão de arquitetura e os números medidos.
>
> Código-fonte canônico: [`crates/ph2d-painter-brush/src/diffusion.rs`](../../crates/ph2d-painter-brush/src/diffusion.rs)
> (a verdade CPU), [`crates/ph2d-painter-fluid/src/shader/composite.wgsl`](../../crates/ph2d-painter-fluid/src/shader/composite.wgsl)
> (o espelho GPU), ADRs 0078–0084. Estado: **W15 fechado**, tudo validado em Metal com paridade
> GPU↔CPU de 0 LSB a ~1e-6.

---

## 1. O princípio que governa tudo: o pixel não é cor — é *matéria*

A decisão fundadora (ADR-0080) é que o campo molhado não armazena "cor com alpha", e sim
**massa de pigmento** em 32 canais por célula: 24 bandas espectrais de absorção/espalhamento
Kubelka–Munk acumuladas *ponderadas por massa* (`ks_acc = Σ massa·K/S`), 3 canais de erro de
round-trip, a massa total, e o acumulador de staining. Tudo no campo é **extensivo** — transporta
linearmente sob difusão, advecção e capilaridade — então três comportamentos que em outros apps
são *hacks* aqui **emergem de graça**:

1. **Mistura subtrativa real**: dois pigmentos na mesma célula somam seus K/S ponderados por
   massa; a redução por pixel (`ks_mix = ks_acc/massa` → reflectância → RGB) dá azul+amarelo→
   **verde**, nunca o cinza da média RGB.
2. **Diluição é divisão, não transparência**: água adicionada não "abaixa o alpha" — ela
   *espalha a mesma massa por mais área*. A cobertura visual vem de
   `alpha = 1 − exp(−(massa/color_sum)·K)`: menos massa por célula ⇒ wash mais claro e mais
   translúcido, exatamente como pigmento disperso em mais água.
3. **Tudo é conservativo**: nenhum efeito "pinta por cima"; todos movem massa de um lugar/camada
   pra outro. É o que permite empilhar mecânicas (lift, deposição, capilaridade, branching) sem
   que uma corrompa a outra.

---

## 2. O sistema de diluição — pigmento dissolvido em água

### 2.1 As três alavancas (todas fisicamente distintas)

| Alavanca | O que faz fisicamente | Onde |
|---|---|---|
| **Opacity** do pincel | Escala a **massa de pigmento por dab**, água fixa — um pincel com menos tinta carregada | `dep = WET_PIGMENT_DEPOSIT · opacity · brush_opacity` ([lifecycle.rs](../../crates/ph2d-tool-painter/src/tool/lifecycle.rs)) |
| **Water** (controle 19) | Escala o pigmento por `1 − water`, água **integral** — de tinta plena (0) a **água pura** (1), contínuo (0.5 = pincel úmido de meia carga) | mesmo chokepoint, `· pigment_load` |
| **A própria física** | A difusão/advecção espalham a massa; a evaporação concentra; capilaridade leva um fio de pigmento na franja | `DiffusionGrid::step` |

A peça-chave é que **água pura é um caso degenerado natural do mesmo pipeline**: um dab com
`massa = 0` e `água > 0` atravessa o splat CPU, o `DabGpu` e o `cs_splat` sem nenhum caminho
especial — molha o papel, reabre o gate, dissolve e re-mobiliza o que encontra. O pincel de água
não foi "uma feature nova no engine"; foi *descobrir que o engine já a continha* e expor o
controle (commit `b6afaf8d`).

### 2.2 Por que parece profissional

- **Pré-molhar funciona** (wet-on-wet real): o campo molhado persiste entre traços (reuso de
  campo ainda-úmido), então água primeiro + tinta depois = bloom dentro da área molhada.
- **Lift limpo**: Water=1 + Lift>0 = levantar tinta seca **sem depositar cor** — antes o lift
  vinha sempre contaminado pela cor do pincel.
- **Value-opacity calibrada** (ADR-0079, re-tune 2026-06-09): o divisor
  `color_sum = 0.55 + 0.45·value` faz pigmento profundo cobrir mais rápido (1.8×) sem o
  "penhasco" que binarizava bordas escuras (o floor original 0.3 dava 3.3× e serrilhava — ver §5).

---

## 3. O preview de umidade — véu escuro + menisco brilhante

### 3.1 A observação

Papel molhado de verdade fica **discretamente mais escuro** (a água preenche o dente do papel e
reduz o espalhamento difuso) e tem um **brilho fino na fronteira** da mancha — o menisco, onde a
película d'água curva e reflete especularmente. São os dois sinais que um aquarelista usa o tempo
todo pra saber *onde ainda dá pra trabalhar*.

### 3.2 A fórmula (3 linhas, em luz linear)

```wgsl
wet  = smoothstep(0.05, 0.45, water)        // 0 = seco, 1 = encharcado (amostra bilinear do campo)
band = 4.0 * wet * (1.0 - wet)              // parábola: pico EXATAMENTE na fronteira (wet = 0.5)
rgb  = clamp(rgb * (1.0 - 0.07 * wet) + 0.05 * band, 0, 1)
```

- O **escurecimento** (−7% no máximo) é proporcional à umidade — uma poça lê mais escura que uma
  névoa, gradiente contínuo.
- O **menisco** usa a identidade `4t(1−t)`: zero no seco, zero no encharcado, máximo na transição
  — ou seja, a banda brilhante *desenha sozinha o contorno da mancha*, sem detecção de borda,
  sem passe extra, sem derivadas. Um termo algébrico no mesmo pixel.
- Ambos sobre **luz linear** (decodifica sRGB → aplica → re-codifica), então o véu se comporta
  perceptualmente igual sobre qualquer cor de fundo.

### 3.3 A decisão de arquitetura que faz tudo funcionar: **view-only**

O sheen é aplicado **somente** nos kernels de textura de preview (`cs_premul_tex` /
`cs_straight_tex`) — nunca no `out_buf`, o composite canônico que é lido de volta e *baked* no
canvas. Display e documento são caminhos separados:

```
                       ┌─► cs_premul_tex / cs_straight_tex (+ SHEEN) ─► tela (PreviewOverride)
cs_composite ─► out_buf┤
                       └─► readback ─► canvas_rgba (SEM sheen) ─► commit/undo/export
```

Consequências em cascata, todas desejáveis:

1. **O sheen nunca contamina o documento** — com Keep Wet ligado por uma hora, o canvas continua
   limpo.
2. **"Seca mais claro" de graça**: o artista vê o véu úmido escuro recuar conforme a água
   evapora, e a pintura final é a versão clara — o comportamento icônico da aquarela real,
   produzido por *subtração de um efeito de view*, não por simulação extra.
3. **Toggle byte-idêntico**: `Show Wet` off ⇒ flag 0 ⇒ saída byte-exata (gate
   `wet_sheen_off_is_byte_identical`, com buffer de água VIVO ligado).
4. **Visível entre traços**: durante a secagem (e sob Keep Wet) o drive publica a textura com
   sheen *além* do readback de bake — você vê a mancha molhada esperando por você entre um traço
   e outro, sem nenhuma mudança na semântica de persistência.

### 3.4 Keep Wet

Um toggle que zera a evaporação nos dois chokepoints (lane GPU via `fluid_diffusion_params`;
fallback CPU no tick) — o campo nunca cruza o piso de secagem, o dry-check nunca dropa o campo, e
a mancha fica trabalhável indefinidamente. O flip re-sobe os params do solver **na hora** (não no
próximo traço). Detalhe físico honesto: com evaporação zero a capilaridade continua espalhando a
água lentamente — papel encharcado faz isso mesmo.

---

## 4. A secagem viva — por que ela conta uma história

A secagem no PH2D não é um fade: é uma **sequência de eventos físicos**, cada um com assinatura
visual própria, todos emergentes do mesmo loop:

1. **A água recua** (`evaporation` por substep) → o véu do sheen clareia de fora pra dentro
   (as bordas secam primeiro — menos água acumulada).
2. **O gate fecha** (`smoothstep(w_lo, w_hi, water)`) → a difusão para de mover pigmento na
   região que secou: o desenho "congela" progressivamente.
3. **Edge darkening**: a taxa de deposição cresce onde está secando
   (`rate = (deposition + deposition_dry·dry)·gran`) → o pigmento ainda móvel é arrastado pelo
   fluxo de saída e **congela na borda que recua** — a linha escura característica do wash.
4. **Granulação**: a deposição é enviesada pros vales do dente do papel
   (`gran = 1 + granulation·(1−paper)`) — pigmentos sedimentares (Ultramarine…) texturizam, os
   staining (Phthalo…) mancham liso, conforme a paleta de 18 pigmentos reais (ADR-0081).
5. **Backruns/blooms**: água nova tocando wash semi-seco empurra pigmento pra fora pela camada de
   velocidade shallow-water + projeção de pressão → o anel "couve-flor" (emergente, não desenhado).
6. **O sheen some por último** onde a água durou mais — e o que resta é a pintura *mais clara*
   que o que se via molhado (§3.3.2).

---

## 5. O método de engenharia (por que isso ficou *certo*, não só bonito)

Os efeitos acima são frágeis se construídos no olho. Três disciplinas seguraram a qualidade:

### 5.1 CPU é a verdade; GPU é um espelho provado
Cada mecânica nasce como código CPU determinístico (HR-5) em `diffusion.rs`/`wet_composite.rs`,
e o WGSL é validado contra ela **numericamente em Metal**: paridade do campo a ~1e-6 (FMA), do
composite a ≤1 LSB, e do premultiply/sheen a **0 LSB byte-exato** (a aritmética inteira
`(c·a+127)/255` foi espelhada bit a bit no shader). 32 gates `--ignored` rodam no Mac de
desenvolvimento; a CI roda as 4.100 provas restantes em 3 plataformas.

### 5.2 Medir o sintoma antes de tocar na causa
O exemplo canônico foi a borda serrilhada de pigmento escuro: antes de "consertar", uma sonda no
espelho CPU **descartou com números** ringing da bicúbica (flips = 0), supersampling (ss 2→4:
salto 75→73) e feather de cobertura — e isolou o culpado real: o floor 0.3 da value-opacity
binarizava `lum(massa)` pro escuro (fração de pixels intermediários no rim: 0.04 vs 0.19 do
claro). O fix foi **um número** (floor 0.55, medido: 0.04→0.14) — e não uma camada de pós-processo
que mascararia o sintoma.

### 5.3 Não-destrutivo por contrato
Toda mecânica nova entra com default = identidade e um gate que o prova bit a bit
(`branching_off_is_bit_identical`, `wet_sheen_off_is_byte_identical`, `lift = 0` byte-idêntico no
compositor…). O look que o artista validou ontem é **invariante de regressão**, não uma esperança.

### 5.4 E a performance que torna tudo *vivo*
Nada disso seria "extraordinário" a 12 FPS. O W15 fechou com o hot loop inteiro residente na GPU
(E1–E5): splat por lista de dabs, água/evaporação/dry-check residentes, wet-bbox por redução GPU,
e o preview indo **direto do composite pra textura do sprite** — zero round-trip de CPU
mid-stroke (imposto de readback removido: ~1 ms/frame em banda típica, 10 ms num full-wash 4K,
mais o premultiply CPU O(canvas) + re-upload que sumiram). É o orçamento que paga o sheen, a
franja capilar e os 22 modos de blend do compositor de camadas no mesmo frame.

---

## 6. Mapa rápido (pra quem for estender)

| Efeito | Verdade CPU | Espelho GPU | Gate |
|---|---|---|---|
| Diluição / dab | `lifecycle.rs` (emissão, `pigment_load`) + `splat` | `DabGpu` + `cs_splat` | `water_brush_emits_waterful_pigmentless_dabs` |
| Sheen | fórmula §3.2 (referência no teste) | `cs_premul_tex`/`cs_straight_tex` + binding `water` | `wet_sheen_matches_cpu_reference` (0 LSB) |
| Secagem/edge/granulação | `transfer_pigment` | `cs_transfer` | `gpu_transfer` parity |
| Lift paper-reveal | `lift_pigment`/`lift_from_backdrop` | `cs_lift` + lerp pro papel no composite | `gpu_cpu_parity_backdrop_lift` |
| Franja ramificada | crest-gate em `capillary_flow` | `capillary.wgsl` | `branching_off_is_bit_identical` |
| Keep Wet | `fluid_diffusion_params` (evap = 0) | `set_from_diffusion` re-upload | pills round-trip (panel) |

**Follow-ups registrados**: LBM/MoXi dendrítico completo (ADR-0082 §2.3), tiling esparso 4K
(ADR-0083 §4), dirty-rect no recomposite multi-camada, pinar o slice ativo contra LRU.

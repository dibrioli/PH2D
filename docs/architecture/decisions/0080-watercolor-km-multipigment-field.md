# ADR-0080 — Watercolor: Kubelka–Munk multi-pigment wet-on-wet field

**Status:** Accepted (2026-06-09) — pedido pelo Enio (a "mágica da aquarela que falta": azul + amarelo molhados sangram num verde vibrante, mistura SUBTRATIVA real, não cinza lamacento). Loop autônomo P0–P6 (`docs/HANDOFF_painter_km_multipigment.md`); #1 da pesquisa (`docs/Painter_projeto/pesquisa_aquarela_estado_da_arte.md`).
**Decisor(es):** Enio (dono/decisor) + Claude.
**Estende:** [ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md) (motor S0–S5c — difusão gateada + deposição/edge-darkening + shallow-water/backruns + capilaridade + BFECC/MacCormack), [ADR-0079](0079-watercolor-params-per-brush-exposure.md) (`WatercolorParams` per-brush; value-opacity), [ADR-0049](0049-fluid-brushes.md) (`ph2d-painter-fluid` GPU resident), [ADR-0051](0051-pigment-mixing.md) (Mixbox/PigmentMode).
**Tags:** painter, fluid-sim, watercolor, kubelka-munk, pigment-mixing, contract-surface, gpu-parity

---

## 1. Contexto — o gap (medido no código)

O motor de aquarela (ADR-0078, S0–S5c, validado pelo Enio) carrega, hoje, no campo molhado:

- **`DiffusionGrid::pigment: Vec<[f32;3]>`** (e `deposited`, e os mirrors GPU `pig_a`/`pig_b`/
  `deposited`/`total`) = uma **cobertura CINZA cor-independente** (`[dep/3; 3]` por dab, soma = `dep`;
  ADR-0079). A **cor** entra só no composite, via `pcol` — **UMA cor por traço**
  (`wet_composite.rs::prepare_wet_composite_from_stroke`).

Consequência: **as cores não se misturam NO CAMPO**. Dois traços de cores diferentes que sangram
juntos não fazem um terceiro — cada traço aplica seu próprio `pcol` uniforme. E o campo é **fresco a
cada traço** (`lifecycle.rs` — "a fresh field per stroke") → não há wet-on-wet *entre* traços.

O K–M óptico **já existe** e é validado: `pigment_mix.rs` — mas **não é** o K–M de constante única
por-canal RGB. É um modelo **ESPECTRAL de 24 bandas** (`NB = SPECTRAL_BANDS = 24`): reconstrói um
espectro de refletância a partir da cor (base de 7 curvas White+CMY+RGB), mistura por **Kubelka–Munk
por banda** (`K/S = (1−R)²/2R`, blend linear por concentração, inverte `R = 1 + K/S − √(K/S²+2·K/S)`)
e re-integra para RGB. `prepare_pigment(color) → PreparedPigment { color, ks:[f32;24], err:[f32;3] }`
(o `err` re-ancora o round-trip da base leaky → endpoints exatos); `mix_prepared_exact(brush, a, t)`
faz o glaze sobre o backdrop. É *este* modelo espectral que produz azul+amarelo→verde
(`blue_plus_yellow_is_green`), e não a mistura cinza/teal que uma média de refletância dá.

## 2. Decisão

### 2.1 Representação do campo — acumulação de K/S espectral ponderada por massa

Por célula, em vez de `[f32;3]` (cinza), o campo carrega **a forma ACUMULADA de um
`PreparedPigment`**:

| acumulador | def. | tamanho |
|---|---|---|
| `ks_acc[NB]` | `Σ_i mass_i · ks_i` (K/S por banda, ponderado por massa) | 24 floats |
| `err_acc[3]` | `Σ_i mass_i · err_i` (re-âncora do round-trip, ponderada) | 3 floats |
| `mass` | `Σ_i mass_i` (a cobertura — substitui o `dens` cinza) | 1 float |

= **28 floats/célula** (flowing) + 28 (deposited). Num grid low-res (256² ou canvas/4) isso é
~7 MB/campo — irrelevante; o grid NÃO é canvas-res (o composite faz upsample).

**Por que a mistura é automática + correta:** `ks_acc`, `err_acc` e `mass` são **extensivos e
lineares** → transportam (`diffuse`/`advect`/`advect_maccormack`/`transfer_pigment`/`capillary_flow`)
**exatamente como o `[f32;3]` de hoje, só com 28 canais em vez de 3** (mesmos stencils conservativos,
mesma ordem de operações). Quando dois pigmentos se encontram numa célula, seus `ks_acc`/`err_acc`/
`mass` somam → a redução por-célula `(K/S)_mix = ks_acc/mass` é a **média de K/S ponderada por massa**
= a lei de mistura K–M de N-pigmentos. **Transporte linear de K/S ⇒ mistura subtrativa** — o ponto
inteiro.

**Redução por-célula (campo → cor):** `prepared_from_field(ks_acc, err_acc, mass) → PreparedPigment`
(novo em `pigment_mix.rs`): `ks = ks_acc/mass`, `refl = ks_to_refl(ks)` por banda,
`color = reflectance_to_rgb(refl) + err_acc/mass` (clamp ≥0). É **idêntico a
`prepare_pigment(mixed_color)`** — reusa a base espectral e os maps K–M existentes (nada de K–M novo).

### 2.2 Por que NÃO o default do handoff (K/S de constante única por-canal RGB)

O §2.1 do handoff recomendava `Kc[3]` (K/S por-canal RGB) + `mass` = 4 floats. **Rejeitado, com razão
forte (o §2.1 permite o desvio):** K/S por-canal RGB dá **lama, não verde**. Azul `[0,0,1]` → K/S
`[∞,∞,0]`; amarelo `[1,1,0]` → K/S `[0,0,∞]`; média → `[∞,∞,∞]` → `R≈[0,0,0]` (preto/cinza). Primárias
RGB não se sobrepõem espectralmente, então o K–M por-canal não pode gerar a banda verde a partir de
azul+amarelo — é exatamente o artefato que o modelo espectral de `pigment_mix.rs` foi construído para
evitar (ver o doc-comment do módulo: *"the muddy grey a linear/OKLab lerp gives"*). O **padrão-ouro**
([feedback-decide-dont-ask], [feedback-perfection-no-deferrals]) é reusar o modelo espectral validado;
a representação mínima que faz isso E mistura de verdade é o **K/S por-banda** (24), não 3.

### 2.3 O single-pigmento reproduz o look validado (o maior risco, §2.3 do handoff)

Para **UM** pigmento de cor `c` e massa `m`: `ks_acc = m·ks_c`, `err_acc = m·err_c`, `mass = m` →
`prepared_from_field` devolve `{ color: c, ks: ks_c, err: err_c }` = **exatamente
`prepare_pigment(c)`** (o `err` re-ancora o round-trip da base leaky, então a cor volta a `c`, não ao
round-trip `rt_c`). O composite então roda **o mesmo `mix_prepared_exact` + glaze straight-alpha de
hoje**, com `alpha = f(mass)` no lugar de `f(dens)` (`mass ≡ dens`) e a **value-opacity ADR-0079
preservada** (`color_sum = 0.3 + 0.7·value(color_mix)`). ⇒ traço de uma cor = **byte-idêntico** ao
caminho pré-ADR-0080. Gate de paridade single-color guarda isso (P2).

**Equivalência provada (P0 gate):** a acumulação a massa igual é **exatamente
`mix_prepared_exact(brush, a, 0.5)`** (`field_mix_equals_mix_prepared_at_5050`) — o campo não é um
modelo novo, é a generalização N-pigmento do K–M já ratificado. (A mistura *do campo* é o blend puro
de K/S; o `4·t·(1−t)` lerp-com-linear do `mix_prepared` é só do estágio de **glaze sobre o backdrop**,
que continua no composite.)

### 2.4 Composite — upsample barato, paridade preservada

Para limitar o custo por-pixel ao de hoje: reduz-se cada **célula do grid** a `color_mix[3]` (via
`prepared_from_field(...).color()`) + `mass`; faz-se o **bicubic Catmull-Rom upsample de `color_mix` +
`mass`** (4 canais, como hoje — `mass` no papel de `dens`); por pixel reconstrói-se a
`PreparedPigment` (`prepare_pigment(color_mix_up)`, o mesmo reconstruct espectral que o backdrop já
paga) e roda o glaze. Single-pigmento: `color_mix` é uniforme = `c` → bicubic devolve `c` →
`prepare_pigment(c)` = o brush de hoje; `mass` bicubic = `dens` bicubic ⇒ idêntico. (Upsample em
espaço-cor RGB vs K/S difere só no sub-cell de campos JÁ misturados no grid — irrelevante; a mistura
acontece no transporte, em res de grid.) Custo extra: um `to_reflectance` por pixel (o lado brush
agora varia no espaço) — simétrico ao backdrop; perf-followup se medido (o smoke é low-res).

### 2.5 GPU mirror (P3) — bit-a-bit

`spectral_basis()` já é `pub` e documentado *"Exposed so a GPU port uploads it ONCE and runs
to_reflectance/reflectance_to_rgb inline"*. Os buffers `pig_a`/`pig_b`/`deposited`/`total` passam a
carregar os 28 canais (ex.: 7 texturas/buffers RGBA32F); **todo passo é separável e idêntico por
canal** (diffuse/advect/MacCormack-clamp/transfer/combine/capillary operam por-canal com a mesma
aritmética — o clamp do MacCormack é por-canal, `transfer`/`combine` aplicam a mesma fração escalar),
então é o mesmo shader de hoje rodado sobre 28 canais. Espelhar a ordem de ops da CPU **idêntica**
(os passos atuais batem **0 ULP** em Metal — manter). Gate: `tests/{gpu_parity,composite_parity}.rs`
estende para os 28 canais (`worst |Δ| < 2e-2`, idealmente 0 ULP) + naga valida os shaders.

### 2.6 Cross-stroke wet-on-wet (P4)

O campo **persiste enquanto molhado** (não mais fresco-por-traço): um traço novo deposita
`(ks, err, mass)` no campo ainda úmido → mistura com o pigmento do traço anterior via a mesma
acumulação. Toca `lifecycle.rs` (`begin_stroke`/epoch) + `painter_fluid_bridge.rs` (o reset por
epoch); mantém o **dry-drop** (só dropa o campo quando REALMENTE seco) e o envelope all-time-wet. Não-
regressão: traços separados (não-sobrepostos) ficam inalterados.

### 2.7 Seleção de pigmento (P5) — color→K/S direto, sem novo param

v1: a **cor do traço** (color picker existente) → `prepare_pigment(color)` → K/S. A mistura é
automática (campo). **Nenhum campo novo em `WatercolorParams`** (cap ≤18 intacto, 17/18) e o
cross-stroke é always-on (sem toggle) — o look single/non-overlap é preservado por construção (§2.3).
Paletas de pigmentos reais com K/S tunado (staining/granulação por-pigmento) ficam como extensão
futura (≥2 consumidores), não bloqueiam o "azul+amarelo=verde".

## 3. Impacto em contratos congelados

- **`pigment_mix.rs` (API):** +`prepared_from_field` / +`ks_field_color` (reduções do campo). Reusam
  os maps K–M privados; `prepare_pigment`/`mix_prepared_exact` intactos. Sem mudança de ABI serializada.
- **`WatercolorParams ≤ 18`** (ADR-0079-amendment-1): **intacto** (17/18; v1 não adiciona controle).
- **`Brush`/`RenderingParams`/`Stamp`/`PainterUiEdit`:** intactos.
- **`SPECTRAL_BANDS = 24`:** pinado (campo e GPU dependem dele = `NB`); já `pub const`.
- **HR-5 (determinismo):** acumulação + transporte são aritmética pura; o single-pigmento é
  byte-idêntico; gates GPU 0-ULP mantidos.

## 4. Consequências

Azul + amarelo molhados se misturam num **verde subtrativo real**, no campo e entre traços
(wet-on-wet), reusando o motor K–M espectral validado — sem assets, determinístico, cross-OS. O
single-pigmento reproduz o look ratificado (value-opacity, edge-darkening, capilaridade, sharpness)
byte-a-byte. Custo: 28 canais no campo (CPU+GPU) — trivial em memória low-res, mecânico em shader.

**Trade-off vs maior fidelidade (avaliado, não adotado em v1):**
- **K–M de 2 constantes** (`K` e `S` separados, 6+ floats): mais fiel para pigmentos opacos/staining,
  mas o modelo de `pigment_mix.rs` é de constante única (`K/S`) e está validado — mudá-lo seria um
  re-tune do look inteiro, fora de escopo.
- **Mixbox** (Sochorová & Jamriška 2021, LUT): perceptualmente o mais preciso, mas o LUT publicado é
  **non-commercial-free / commercial-paid** — `pigment_mix.rs` já é um clean-room espectral
  justamente para evitá-lo (ADR-0051). Não reintroduzir a dependência.

Default v1 = **K/S espectral por-banda acumulado** — auto-contido, reusa o validado, mistura de
verdade. ([feedback-no-industrial-claims]: as afirmações acima são checadas no código —
`pigment_mix.rs` NB=24, `field_mix_*` gates.)

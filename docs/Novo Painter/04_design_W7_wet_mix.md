# 04 — Design W7: Wet Mix (mixer-brush)

> Design-first da onda W7 ([plano §W7](03_plano_implementacao.md)). Modelo reservatório pickup-and-deposit
> (IMPaSTo/DAB, [teoria §2](01_pesquisa_teorica_e_literatura.md#2-wet-mixing--mixer-brush-modelo-reservatório-pickup-and-deposit)),
> reusando ao máximo o renderer wash vivo. ADR-0097 (CPU-first).

## Insight central

O `apply_one_stamp_wash` vivo ([`cpu_render/mod.rs:564`](../../crates/ph2d-painter-brush/src/cpu_render/mod.rs#L564))
já faz **quase tudo** que o mixer-brush precisa: build-up de cobertura monotônica por-pincelada (`coverage`),
cor depositada acumulada (`wash_color`), e composição da cor depositada **com o backdrop via Mixbox K-M**
(`mix_prepared(brush_color, backdrop, t)`). O que ele **não** faz: a cor depositada é **constante** (a cor
do brush) e o depósito **não esgota**.

**Wet Mix = trocar duas coisas, sem tocar o resto do composite:**
1. a **cor depositada** deixa de ser a cor fixa do brush e passa a ser um **reservatório que evolui** dab-a-dab
   (pega cor do canvas → smear);
2. a **taxa de depósito** passa a ser **escalada por carga/attack/dilution e a esgotar** (charge → trilha que some).

Tudo o mais (coverage build-up, z-order, mistura K-M com backdrop, paper tooth, grain, taper, falloff) **fica idêntico**.

## Onde o estado mora

O reservatório é uma propriedade do **brush enquanto viaja** — não cabe nos buffers por-pixel. Mora no **tool**
(runtime, não serializado), ao lado de `wash_coverage`/`wash_color`, alocado no `begin_stroke` e re-semeado a
cada pen-down (recarga = "re-molhar o pincel"):

```rust
// ph2d-painter-brush (novo, exposto p/ o tool segurar o estado)
pub struct WetState {
    pub color: [f32; 3], // pigmento do reservatório (linear sRGB); seed = cor do brush no begin_stroke
    pub load:  f32,      // [0,1] tinta restante; seed = charge
}
```

`WetMixConfig` (lido de `brush.wet_mix`, passado por valor — NÃO pelo `Stamp` de 96B congelado):

```rust
pub struct WetMixConfig {
    pub dilution: f32, pub charge: f32, pub attack: f32, pub pull: f32,
    pub grade: f32, pub blur: f32, pub blur_jitter: f32, pub wetness_jitter: f32,
}
```

**Assinatura nova** (API Rust interna, não é ABI congelado):
```rust
pub fn apply_stamps_wash(
    canvas, backdrop, coverage, wash_color, w, h, stamps,
    opacity_cap, pigment, alpha_lock, paper_grain,
    wet: Option<(&mut WetState, WetMixConfig)>,   // ← novo; None = comportamento atual byte-idêntico
)
```
`None` ⇒ caminho atual **byte-for-byte** (sem regressão no brush default). `Some` ⇒ mixer-brush.

## Gating

Wet Mix ativa quando `brush.wet_mix.wet_mix_enabled` (o tool liga isso quando `rendering_mode ∈
{UniformBlending, IntenseBlending}` — paridade Procreate: Wet Mix aparece nos 2 modos Blending). Fora disso,
`wet = None`.

## Algoritmo por-dab (substitui só o cálculo de `brush_color` + `rate`)

No `apply_one_stamp_wash`, por pixel do footprint (após `shape_alpha`/`rate` base já computados):

```
# 1. PICKUP (Pull) — o pincel pega cor do canvas e carrega adiante (o smear)
picked   = decode_linear(backdrop[pix])           # backdrop pré-stroke = estável
k_pickup = pull * shape_alpha                      # contato modula o pickup
reservoir.color = lerp(reservoir.color, picked, k_pickup)

# 2. COR DEPOSITADA = reservatório (não mais a cor fixa do brush)
dab_color = reservoir.color                        # entra onde antes entrava rgb_clamped

# 3. WETNESS JITTER — randomiza a diluição por-dab (seed determinístico do stamp)
dilution_eff = clamp(dilution * (1 + wetness_jitter * rand_signed(seed)), 0, 1)

# 4. TAXA DE DEPÓSITO — escala por attack·load, afina com dilution, e ESGOTA
deposit = rate * attack * reservoir.load * (1 - dilution_eff)

# 5. CHARGE / depleção — a carga cai conforme deposita → trilha some
reservoir.load = max(reservoir.load - deposit * DEPLETE_K, 0)
```

Daí em diante, **idêntico ao código atual** com `rate := deposit` e `rgb_clamped := dab_color`:
`coverage[pix] += deposit·(1−cov)`; `wash_color = lerp(wash_color, dab_color, deposit)`; composição K-M com
o backdrop por `t = eff/out_a`. **Reúso total do composite.**

### Mapeamento Procreate → equação (rastreável à [referência §7](02_referencia_parametros_procreate.md#7-wet-mix-núcleo-do-mixer-brush))

| Slider | Papel na equação |
|---|---|
| **Charge** | `reservoir.load` inicial (seed no begin_stroke); maior = trilha mais longa antes de secar |
| **Dilution** | `(1 − dilution_eff)` no depósito → mais água = mais transparente |
| **Attack** | multiplicador do depósito → quão "grudenta"/sólida a tinta carregada |
| **Pull** | `k_pickup` → quanto o reservatório puxa/esfrega cor do canvas (o smear) |
| **Wetness Jitter** | randomiza `dilution_eff` por-dab |
| **Grade** | chunkiness: escala o contraste do `paper_tooth`/grain (interage com W4) — *fase 2 de W7* |
| **Blur / Blur Jitter** | blur espacial do depósito (precisa de vizinhança) — *fase 2 de W7* |

## Faseamento de W7

- **W7.0+W7.1-W7.5 (núcleo):** `WetState` + pickup/deposit + Charge/Dilution/Attack/Pull + Wetness Jitter.
  É o mixer-brush funcional. Sem mudança de ABI (usa args de fn + estado no tool).
- **W7.6-W7.7 (Grade/Blur):** chunkiness + blur espacial. Blur exige um pass de vizinhança no footprint —
  mais caro; isolado numa fase 2 pra não travar o núcleo.

## Determinismo (HR-5)

`rand_signed(seed)` deriva do `seq`/contador do stroke (mesmo esquema dos jitters de cor existentes em
`advance.rs`), nunca de `Math.random`. O reservatório evolui deterministicamente (mesma ordem de dabs →
mesmo resultado). Teste de paridade cross-OS.

## Verificação

- **Headless (`cargo test -p ph2d-painter-brush`):**
  - `wet_none_is_byte_identical` — `wet=None` reproduz o wash atual byte-for-byte (anti-regressão do brush default).
  - `charge_depletes` — com `pull=0, charge<1`, a cobertura de uma linha reta **cai monotonicamente** ao longo da trilha.
  - `pull_smears` — arrastar de um blob vermelho sobre branco carrega vermelho além do blob (reservatório pega).
  - `dilution_transparency` — maior `dilution` ⇒ menor cobertura efetiva no mesmo traço.
  - `yellow_over_blue_is_green` — depósito K-M sobre backdrop azul dá verde (já passa hoje; garantir que Wet Mix não regrede).
- **Manual (`play.command`):** pincel Blending com charge baixo afina; pull alto esfrega; dilution alto aguado.

## Riscos

1. **Pickup do backdrop vs live canvas:** lemos o **backdrop pré-stroke** (estável). Smear *dentro* do mesmo
   traço emerge da evolução do reservatório (carrega o que pegou), sem ler o canvas vivo → mantém a
   estabilidade de overlap do wash. Se faltar realismo de "molhar o próprio traço", é refinamento de fase 2.
2. **`begin_stroke` deve semear `WetState`** (load=charge, color=cor do brush) e o `end_stroke` descartá-lo —
   espelha o ciclo de `wash_coverage`. Esquecer = trilha não recarrega entre traços.
3. **`DEPLETE_K`** é o único parâmetro de tuning livre; calibrar pra que charge=100% ≈ trilha "infinita" e
   charge baixo seque em ~1 diâmetro. Documentar o valor.

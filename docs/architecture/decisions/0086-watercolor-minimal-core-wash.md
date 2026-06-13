# ADR-0086 — Núcleo mínimo de aquarela (`ph2d-painter-wash`): difusão-gateada + edge-darkening, GPU, RGB linear

**Status:** PROPOSTO (aguarda ratificação do Enio antes de codar) · **Data:** 2026-06-13
**Supersede (em escopo):** a complexidade empilhada de ADR-0078..0085 (ver §7). O sistema
v2 fica **intacto** em `ph2d-painter-fluid` (backup tag `watercolor-v2-backup-2026-06-12`)
para comparação lado-a-lado; este ADR cria uma crate **nova e isolada**.

> **Decisão:** reconstruir a aquarela a partir do **núcleo canônico mínimo do Curtis 1997**
> — água + difusão-gateada de pigmento + FlowOutward (edge-darkening) + evaporação +
> composição subtrativa — numa crate limpa `ph2d-painter-wash`, **sem** shallow-water,
> **sem** wick capilar autônomo, **sem** lift/branched/MacCormack, e com pigmento **RGB
> linear (1 vec4)** no lugar do K–M espectral de 24 bandas. Fundamento: análise
> [`simplificacao_ultra_brutal_watercolor.md`](../../Painter_projeto/simplificacao_ultra_brutal_watercolor.md).

---

## 1. Por quê (problema)

O motor v2 (`ph2d-painter-fluid`) é instável, inconsistente e de difícil manutenção: ~7
subsistemas acoplados (shallow-water, wick capilar, lift, branched, MacCormack, K–M 24-band,
gel-timer), 28 params de UBO, 21 controles, ~20 passes, ~15 buffers. A instabilidade vem do
**acoplamento** — em particular do **wick capilar autônomo**, que espalha água sozinho e
exigiu uma saga inteira de mecanismos de "limite" (Surface Tension → absorção → gel), nenhum
estável. A nossa própria base ADR-0049 (difusão-advecção gateada, **sem** velocidade nem
wick) já fazia aquarela; tudo após isso foi empilhado.

## 2. Princípios do núcleo mínimo

1. **Estável por construção.** Difusão explícita com `D ≤ 0.25` (CFL do calor 2D) e advecção
   upwind com `|v|·dt ≤ 0.5` (CFL) → incondicionalmente estáveis. Sem projeção de pressão.
2. **Bounded por construção.** A água só é posta pelo pincel e só **diminui** (evaporação).
   **Não há espalhamento autônomo de água** → não há "poça que cresce" → **nada a limitar**
   (toda a saga Surface-Tension/absorção/gel deixa de existir).
3. **Conservativo + determinístico.** Transporte em forma de *gather* com faces simétricas
   (massa conservada); ping-pong de buffers (sem corrida).
4. **GPU-first, mínimo.** 2 campos, ~3 kernels, 1 UBO pequeno, region-scoped p/ 4K.
5. **RGB linear.** Pigmento = absorbância por canal (Beer–Lambert). K–M é add-on futuro.

## 3. Estado (campos por célula)

| Campo | Tipo | Bytes | Papel |
|---|---|---|---|
| `water` | `f32` | 4 | molhabilidade; pincel adiciona, evaporação remove |
| `pigment` | `vec4<f32>` | 16 | `(a_r, a_g, a_b, mass)` — absorbância óptica por canal (Beer–Lambert) + massa p/ cobertura/gate |
| `paper` | `f32` (estático) | 4 | altura do grão (permeabilidade + granulação) |

**20 B/célula dinâmicos** (vs ~128 B + 6 buffers de pigmento hoje). Absorbância transporta
**linearmente** (mistura por soma → Beer–Lambert); mistura subtrativa vibrante (K–M) é o
add-on §8.

## 4. Buffers GPU (solver `WashSolver`)

`water_a`, `water_b` (`f32`), `pig_a`, `pig_b` (`vec4`) — ping-pong; `paper` (`f32`,
estático); `dabs_buf` (lista de dabs); `params` (UBO); `stats` (opcional, wet-bbox p/
region-scope). **4 buffers de campo dinâmicos** (vs ~15).

## 5. Passes (3 kernels)

### 5.1 `cs_splat` (entrada do pincel — 1×/frame)
Para cada dab (centro, raio, `water_add`, `pigment_add=(a_r,a_g,a_b,m)`), falloff
`f = 1 − smoothstep(0,1,dist/r)`:
```
water[i]   = min(water[i] + water_add·f, 1.0)
pigment[i] = pigment[i] + pigment_add·f          // absorbância acumula
```

### 5.2 `cs_step` (a FÍSICA — único kernel, por substep, region-scoped, gather)
`(water_a, pig_a) → (water_b, pig_b)`, swap. Pseudo-WGSL:
```wgsl
let wc = water_a[i];
let perm = mix(P.perm_valley, P.perm_crest, paper[i]);
let gc   = smoothstep(P.w_lo, P.w_hi, wc) * perm;          // wet-gate do centro
// 4 vizinhos (clamp na borda): wN, pN e seus gates gN

// (1) DIFUSÃO (bloom) — Laplaciano 5-pt conservativo, gate de FACE = min(gc,gN) (simétrico)
var lap = vec4(0.0);
for face in {L,R,U,D}: lap += min(gc, gN) * (pN - pc);
var p_new = pc + P.diffusivity * lap;                      // D ≤ 0.25 ⇒ estável

// (2) FLOWOUTWARD (edge-darkening) — advecção upwind do pigmento p/ a célula mais SECA
let dwx = 0.5*(wR - wL);  let dwy = 0.5*(wD - wU);
let v   = vec2(-P.flow_outward*dwx, -P.flow_outward*dwy);  // wet→dry (CFL: clamp |v|≤0.5)
p_new  += upwind_donor(pc, pN, v, gc);                     // forma donor-cell conservativa

pig_b[i]   = max(p_new, vec4(0.0));
// (3) EVAPORAÇÃO (per-célula)
let w = max(wc - P.evaporation, 0.0);
water_b[i] = select(0.0, w, w >= 1e-4);
```
**Deposição é IMPLÍCITA:** quando `water` cai abaixo de `w_lo`, `gc→0` → o pigmento para de
difundir/advectar = **assentou**. O FlowOutward concentra pigmento na fronteira que recua;
ao secar ali, ele congela → **edge-darkening emerge** sem buffer/pass de deposição separado.
*(Granulação opcional v1.1: modular o congelamento por `paper` — mais nos vales.)*

### 5.3 `cs_composite` (render subtrativo)
Amostra bilinear/bicúbica do campo de pigmento na resolução do canvas:
```wgsl
let absorb = max(pig.rgb, 0.0);
let T      = exp(-absorb);              // transmitância Beer–Lambert por canal
out_rgb    = backdrop_rgb * T;          // glaze subtrativo multiplicativo
let alpha  = 1.0 - exp(-K_COV * pig.w); // cobertura a partir da massa
```
(+ `cs_premul_tex` p/ a textura de preview, como hoje.) **K–M de 1 pigmento** é trivial de
trocar aqui depois sem mexer no transporte.

## 6. Parâmetros (UBO) e controles

**UBO (~8 f32 + 6 u32):** `width,height, region_ox/oy/w/h, diffusivity, flow_outward,
evaporation, w_lo, w_hi, perm_valley, perm_crest, granulation`. (Sem velocity/pressure/
viscosity/drag/capillary/lift/sharpness/bleed_limit/mobility.)

**Controles de artista (~5):** `Diffusivity` (bloom) · `Bleed` (FlowOutward/edge) ·
`Evaporation` (secagem) · `Pigment Load` (força do dab) · `Granulation` (grão).
*(De 21 → 5. wet-gate e perm ficam fixos ou 1 slider avançado.)*

## 7. O que NÃO entra (vs v2) — e fica disponível como add-on isolado depois

Removidos do núcleo: camada shallow-water (velocidade/pressão/viscosidade/drag/backrun, 6
Jacobi/substep) · wick capilar autônomo + co-advecção de pigmento · branched capillary ·
lift/backdrop · MacCormack/BFECC · K–M espectral 24-band · gel-timer/Surface-Tension/
absorção · `deposited`/`total`/`combine` separados (deposição vira implícita) · ~16 controles
· ~11 buffers · ~5 constantes mágicas (GEL_RATE, CAPILLARY_MIN_SAT, LIFT_BLEED_KEEP,
RELAX_ITERS, mobility).

## 8. Add-backs futuros (1 por vez, opt-in, sobre o núcleo estável)

1. **Cor subtrativa K–M / Mixbox** (LUT 3-4 pigmentos) — só no `cs_composite` + no dab; o
   transporte não muda. Maior salto visual.
2. **Franja capilar water-only (Curtis-faithful)** — difusão de ÁGUA limitada pela
   evaporação (sem co-advecção de pigmento = a versão estável).
3. **Backruns/cauliflower** explícitos — modelados direto, sem a camada shallow-water.

## 9. Plano de validação (headless, GPU Metal)

Teste `wash_invariants` (espelha o mínimo do `physical_invariants`):
- **massa de pigmento conservada** sob `cs_step` (sem evaporação).
- **água bounded** (trivial: nunca cresce; só decai).
- **bloom**: difusão espalha pigmento em região molhada (slider tem efeito).
- **edge-darkening**: ao secar, a massa concentra na fronteira (rim mais escuro que o miolo).
- **estabilidade**: 1000 substeps em `D=0.24, λ=max` não diverge (sem NaN/runaway).
- **composição subtrativa**: 2 glazes empilhados escurecem multiplicativamente.

## 10. Isolamento (multi-agente / contrato)

Crate nova `ph2d-painter-wash` — **drop-in, zero edição em arquivo central** (membro via
glob `crates/*`). Não toca `ph2d-painter-fluid`, contratos congelados, nem o tool até a
fase de integração (decisão separada). Sem novo gate de contrato nesta fase (a superfície é
interna à crate nova).

---

**Próximo passo após ratificação:** scaffold da crate `ph2d-painter-wash` (Cargo.toml +
`WashSolver` + 3 shaders + `wash_invariants`), commit local, smoke do Enio. Integração no
tool (segunda aquarela / flag) é ADR/decisão separada.

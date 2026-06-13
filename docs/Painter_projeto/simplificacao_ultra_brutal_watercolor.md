# Simplificação ultra-brutal do watercolor — alvos de corte para o núcleo mínimo

**Data:** 2026-06-13 · **Base:** Curtis et al. 1997 (canon) + linhagem real-time GPU
(Van Laerhoven 2005, Scott 2004), cruzada com o inventário exaustivo da nossa
implementação no commit `9fa573bf` (backup: tag `watercolor-v2-backup-2026-06-12`).

> **Tese:** o sistema ficou instável/inconsistente porque empilhamos ~7 subsistemas
> acoplados (cada um com buffers, params e laços de realimentação próprios) sobre um
> núcleo que já era suficiente. A simplificação não é "tirar features ruins" — é
> **voltar ao núcleo canônico mínimo** (que a nossa própria base ADR-0049 já era) e
> remover tudo que foi empilhado depois, mantendo só o que entrega a assinatura visual
> da aquarela: **bloom + edge-darkening + deposição granular + cor subtrativa**.

---

## 1. O núcleo mínimo, segundo a fonte original

Curtis 1997 tem 3 camadas (shallow-water + deposição + capilar) + render Kubelka–Munk.
Mas o próprio paper e a linhagem real-time deixam claro o que é **irredutível** vs.
**refinamento para efeito específico**:

| Mecanismo | Papel | Irredutível? |
|---|---|---|
| **Campo de água** `w` (molha, seca) | define onde a tinta se move + a borda que recua | **SIM** — é o substrato |
| **Difusão de pigmento gateada pela água** | o *bloom* (a tinta se espalha no molhado) | **SIM** — é a aquarela |
| **FlowOutward `−λ∇w`** (pigmento empurrado à borda que seca) | o *edge-darkening* (a assinatura nº1 da aquarela) | **SIM** — define o "olhar de aquarela" |
| **Evaporação** (gate seca → congela) | o traço assenta; termina o bloom | **SIM** |
| **Deposição + granulação** | a marca fica no papel; textura no grão | quase — pode ser implícita |
| **Composição subtrativa** (cor sobre fundo) | glaze translúcido | **SIM**, mas a forma mínima basta |
| Shallow-water (velocidade+pressão+viscosidade+drag) | fluxo *direcional* + backruns | **NÃO** — refinamento |
| Capilaridade (wick autônomo de água) | franja *além* do traço | **NÃO** — refinamento (e a nossa fonte nº1 de instabilidade) |
| K–M espectral multi-pigmento (24 bandas) | azul+amarelo = verde vibrante | **NÃO** — refinamento de cor |
| Lift / backdrop, branched capillary, MacCormack, gel-timer | efeitos pontuais | **NÃO** |

**Conclusão da fonte:** o mínimo funcional = **água + difusão-gateada + FlowOutward +
evaporação + deposição + composição subtrativa**. Exatamente a nossa base ADR-0049,
*antes* de empilharmos ADR-0078..0085. Tudo após isso é candidato a corte.

---

## 2. Inventário do que temos (resumo do que foi medido)

- **GpuParams: 28 campos** (112 B UBO). Núcleo (sempre lido): width/height/region(6)/
  diffusivity/evaporation/downhill/flow_outward/w_lo/w_hi/perm_valley/perm_crest. **Os
  outros 11+ são lidos por exatamente UMA passagem gateada cada** (add-on).
- **21 controles de artista** — 8 já vêm **desligados no preset** (Downhill, Sharpness,
  Lift, Branching, Water=0, etc.).
- **~20 passagens compute** em 8 shaders; o passo residente dispara, com tudo ligado,
  o bloco shallow-water (8+ dispatches, incl. **6 Jacobi por substep**) + capilar (2) +
  difusão/advecção/transfer/evaporate/combine.
- **Buffers de campo cheio:** 6 buffers de pigmento a **128 B/célula** (pig_a, pig_b,
  pig_c, deposited, total, lift_source) + água/water_b/gel/paper/lifted_frac (f32) +
  vel_a/b (vec2) + pressure_a/b/divergence (f32). Vários marcados `#[allow(dead_code)]`
  (scratch). **Pegada de memória enorme a 4K.**
- **Pigmento: PIG_CH=32 (8 vec4/célula), 24 bandas K–M** + err(3) + mass + stain + pad.
  Era **1 vec4** antes do ADR-0080 → cresceu **8×**.

---

## 3. ALVOS DE CORTE — a lista da simplificação ultra-brutal

### C1 — REMOVER SUBSISTEMAS INTEIROS (o grosso da instabilidade e do custo)

| # | Subsistema | O que sai | Por que (instabilidade/custo) |
|---|---|---|---|
| **C1.1** | **Camada shallow-water (velocidade)** | passes `cs_add_forces`, `cs_divergence`, `cs_clear_pressure`, `cs_jacobi`×6, `cs_project`, `cs_advect_velocity`; buffers `vel_a/b`, `pressure_a/b`, `sw_divergence`; params `velocity`, `viscosity`, `drag`, `pressure`; controles **Flow Velocity, Viscosity, Drag, Backrun** | **8+ dispatches/substep** (incl. 6 Jacobi) = o pico de custo. Projeção de pressão é a parte mais frágil/cara. A base sem ela já fazia aquarela (ADR-0049). FlowOutward sobrevive via `−λ∇w` na advecção estática. |
| **C1.2** | **Wick capilar autônomo** (água que se espalha sozinha) | passes `cs_capillary`, `cs_copy_fields`; buffers `water_b`, `gel`; params `capillary`, `capillary_mobility`, `capillary_branching`, `bleed_limit`; controles **Capillary, Branching, Bleed Limit** | **A fonte de TODA a instabilidade desta sessão** (creep da poça, borda pixelada, a saga Surface-Tension→absorção→gel). Sem wick autônomo, a poça = footprint do pincel; **não há nada para "limitar" → o problema some por construção.** |
| **C1.3** | **MacCormack/BFECC sharpness** | passes `cs_advect_velocity_rev`, `cs_advect_correct`; buffer `pig_c`; param `sharpness`; controle **Sharpness** | Já vem **desligado** ("triplica o pass mais caro"). Add-on de nitidez. |
| **C1.4** | **Lift / backdrop (re-wetting)** | pass `cs_lift`; buffers `lift_source`, `lifted_frac`; param `lift`; controle **Lift**; o caminho paper-reveal no compositor | Já vem **desligado**. Re-molhar tinta seca é efeito de luxo; remove um campo inteiro + lógica do compositor. |
| **C1.5** | **Branched capillary (franja fibra-a-fibra)** | já coberto em C1.2 (morre com o wick) | Opt-in, default 0. |
| **C1.6** | **Co-advecção de pigmento na capilaridade** (nossa extensão *novel*) | morre com C1.2 | A pesquisa marcou como **não-canônica** (Curtis é water-only) e fonte de inconsistência. |

### C2 — COLAPSAR (reduzir a representação ao mínimo)

| # | Hoje | Vira | Tradeoff |
|---|---|---|---|
| **C2.1** | **Pigmento K–M 24-bandas (PIG_CH=32, 8 vec4, 128 B/cél, 6 buffers)** | **1 vec4 = RGB linear + massa (16 B/cél)** | **Perde** a mistura subtrativa vibrante (azul+amarelo=verde) e o glaze espectral. Ganha **8× menos memória/banda** + simplicidade total do transporte. *Maior decisão do Enio* — ver §5. |
| **C2.2** | Canais `stain`, `err[3]`, pad (do PIG_CH=32) | somem com C2.1 | staining/re-anchor eram do K–M; sem ele, desnecessários. |
| **C2.3** | Buffers `deposited` + `total` separados + pass `cs_combine` | deposição **implícita** (pigmento em célula seca não difunde) OU 1 buffer `deposited` simples sem `cs_combine` | Remove 1-2 buffers de 128B/cél + 1 pass. |

### C3 — LIXO MORTO / REDUNDANTE (corte sem custo, faça já)

| # | Item | Evidência |
|---|---|---|
| **C3.1** | `cs_deposit` em `fluid.wgsl` — legado, **não dispachado** | o caminho residente usa `cs_splat` |
| **C3.2** | `_pad_lift2` (offset 108 do UBO) — padding lido por ninguém | dead |
| **C3.3** | `capillary_mobility` — constante 0.35 escondida ocupando slot de UBO | não é controle; vira literal (ou some com C1.2) |
| **C3.4** | Docstrings dizendo "20 controles" — o array tem 21 | stale |
| **C3.5** | Buffers scratch `water_b`/`pig_b`/`pig_c`/`sw_divergence` (`#[allow(dead_code)]`) | caem sozinhos quando C1.1/C1.2/C2 saem |

### C4 — PARÂMETROS IMPRECISOS / MÁGICOS / REDUNDANTES

| # | Param / constante | Problema | Ação |
|---|---|---|---|
| **C4.1** | Constantes do Curtis que **não verificaram** (μ=0.1, ν=0.01, β) | citadas em mirrors, não no primário | irrelevante após C1.1 (somem com a camada) |
| **C4.2** | `GEL_RATE=0.004`, `CAPILLARY_MIN_SATURATION=0.005`, `LIFT_BLEED_KEEP=0.25`, `RELAX_ITERS=6`, `CAPILLARY_PIGMENT_MOBILITY=0.35` | **5 constantes mágicas** sem fundamento medido | todas somem com C1.1/C1.2/C1.4 |
| **C4.3** | Trio de deposição `deposition` / `deposition_dry` / `granulation` | 3 params acoplados para 1 efeito | colapsar para **1-2** (intensidade + grão) |
| **C4.4** | `perm_valley` / `perm_crest` (2 params de permeabilidade do papel) | controle fino raramente tocado | **1** param (ou fixo) |
| **C4.5** | `w_lo` / `w_hi` (banda do wet-gate) | 2 sliders para 1 limiar suave | **1** (centro) ou fixo |
| **C4.6** | `Downhill` (default 0, forçado off) | inércia; não usado | **remover controle** |

### C5 — CONTROLES: de 21 → ~5-6

**Manter (núcleo):** `Diffusivity` (bloom), `Bleed` (FlowOutward/edge-darkening),
`Evaporation` (secagem), `Deposition` (+ edge-darkening), `Granulation` (grão).
Opcional: 1 do wet-gate.

**Cortar (16):** Downhill, Flow Velocity, Viscosity, Drag, Backrun (C1.1) · Capillary,
Branching, Bleed Limit (C1.2) · Sharpness (C1.3) · Lift (C1.4) · Wet Gate Lo/Hi → 1
(C4.5) · Perm Valley/Crest → 1 (C4.4) · Water (manter? é tool-side, barato — decisão).

---

## 4. O núcleo mínimo resultante (alvo)

- **2 campos:** `water` (f32) + `pigment` (1 vec4: RGB linear + massa) = **20 B/cél**
  (vs ~128 B+ e 6 buffers hoje). Sem velocity/pressure/capillary/lift/gel/scratch.
- **~4 passes/frame:**
  1. `cs_splat` — pincel adiciona água + pigmento.
  2. `cs_step` — difusão gateada (bloom) **+** FlowOutward `−λ∇w` (edge-darkening) **+**
     evaporação, num único kernel (gather 5-pt, conservativo). *(deposição implícita: em
     célula seca o gate fecha e o pigmento para.)*
  3. `cs_composite` — cor subtrativa (Beer–Lambert ou K–M de 1 pigmento) sobre o backdrop.
  4. (`cs_reduce` opcional p/ wet-bbox/region-scope — perf, não física.)
- **~5 params no UBO** (+ region). **~5-6 controles.**
- **Estabilidade por construção:** sem wick autônomo não há creep → **nada para limitar**
  (toda a saga Surface-Tension/absorção/gel evapora). Sem projeção de pressão não há o
  solver frágil. Sem 6 buffers acoplados não há laços de realimentação ocultos.
- **Velocidade:** ~4 dispatches vs ~20+; 20 B/cél vs 128+. **Folga enorme a 4K.**

---

## 5. A ÚNICA decisão de peso para o Enio

**C2.1 (pigmento K–M 24-bandas → 1 RGB):** é o maior ganho de simplicidade/velocidade,
mas é o único corte que **muda o look** — perde a mistura subtrativa vibrante (verde de
azul+amarelo) e o glaze espectral profundo. Caminhos:

- **(a) Cortar agora** (RGB linear): núcleo mais simples/rápido possível; cor "boa o
  bastante"; **re-adicionar K–M depois** como camada isolada *se* o look pedir (e aí via
  Mixbox/LUT, não 24 bandas à mão).
- **(b) Manter K–M mas reduzir** de 24 bandas → ~8, ou trocar pelo LUT do Mixbox (3-4
  pigmentos reais), mantendo o look subtrativo a ~½ do custo.

Recomendo **(a)** para a primeira versão do núcleo estável — provar simplicidade+
estabilidade primeiro, re-introduzir cor subtrativa como add-on opcional depois.

---

## 6. Add-backs futuros (depois do núcleo estável, 1 de cada vez, isolados)

Ordem por valor/esforço, cada um como camada opt-in sobre o núcleo:
1. **K–M / Mixbox** (cor subtrativa) — maior salto visual.
2. **Franja capilar water-only** (Curtis-faithful, limitada pela evaporação) — a franja
   suave *além* do traço, **sem** co-advecção de pigmento (a versão estável).
3. **Backruns / cauliflower** explícitos (efeito assinatura) — sem a camada shallow-water
   inteira; modelado direto.
4. BFECC, lift, branched — só se o look específico pedir.

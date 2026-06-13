# Wash (aquarela núcleo mínimo) — Solução de Erros / Postmortem

> Catálogo sério dos bugs do modo **Wash** (`ph2d-painter-wash`, ADR-0086/0087) e como foram
> resolvidos. Vários custaram MUITAS tentativas porque o **sintoma enganava sobre a causa**. Esta
> doc existe pra que ninguém (humano ou LLM) refaça a mesma caça. Leia o §0 antes de tocar em
> qualquer artefato visual de aquarela.
>
> Código: solver `crates/ph2d-painter-wash/src/solver.rs` + shaders `src/shader/{splat,wash,composite}.wgsl`;
> bridge `shells/desktop/src/render_loop/painter_wash_bridge.rs`. Gates: `tests/wash_invariants.rs`.
> Tracker de status: [`../HANDOFF_wash.md`](../HANDOFF_wash.md).

---

## §0 — As 6 lições que custaram caro (leia isto)

1. **Borda "pixelada" tem ≥3 causas DIFERENTES — não trate como uma só.** Foram, em ordem: violação
   de CFL (xadrez), frente molhada estática (rim duro), contorno do cap de saturação (degrau
   núcleo↔halo) e quantização 1:1 do campo no zoom (escada do contorno). Cada uma exige um fix
   distinto, em camada distinta (kernel de física vs. composite vs. bridge). Confundir uma com a
   outra = rounds perdidos. **Antes de mexer, classifique QUAL borda.**

2. **O cap de saturação tornou VISÍVEIS bugs que já existiam.** Antes do cap, áreas densas saturavam
   pro preto/escuro uniforme e MASCARAVAM xadrez, mosqueado e degraus. Ao consertar o preto (B1), os
   artefatos do campo "apareceram". Lição: **um fix de display pode desmascarar bugs de campo
   pré-existentes** — não assuma que é regressão do seu fix.

3. **Provas de estabilidade isoladas mentem quando os termos somam.** O kernel garantia `D≤0.25` e
   `v≤0.25` SEPARADAMENTE; juntos, o outflux por substep passava de 1.0 → célula negativa →
   `max(·,0)` = buraco branco = xadrez. **Some os orçamentos de CFL de TODOS os termos do gather.**

4. **Em keep-wet / Evaporation 0 nada congela — todo transporte roda PRA SEMPRE.** A estabilidade
   "por construção" do núcleo mínimo pressupunha que a secagem congela o campo. Sem secagem, qualquer
   anisotropia (região móvel, permeabilidade por-pixel, advecção) acumula sem limite. **Teste SEMPRE
   o caso evap=0 — é o pior caso, não um canto.**

5. **Físico ≠ o que o artista quer, mas a intuição do artista aponta a causa.** "Escurecer o
   gradiente externo proporcionalmente" (Enio) era a descrição exata do bug do cap duro. Quando o
   dono descreve a solução em termos perceptuais, **traduza pra causa técnica em vez de descartar.**

6. **Campo 1:1 com o canvas = sem anti-alias grátis.** `inv=1` ⇒ o composite amostra *nearest*.
   Qualquer borda nítida no campo vira escada no zoom. Suavizar VALOR (saturação) não conserta
   CONTORNO; é preciso reamostrar (blur) no composite.

---

## §1 — Catálogo de bugs (sintoma → causa → fix → gate)

### B1 — pintar repetido no mesmo lugar → PRETO
- **Sintoma:** sobrepor traços no mesmo ponto escurecia sem limite até o preto.
- **Causa:** pigmento = absorbância Beer–Lambert (`a=−ln(c)·mass`) SOMADA por dab sem teto →
  `exp(−a)→0`.
- **Fix:** saturação de papel no composite — capar a massa efetiva no `MASS_MAX` (hue `absorb/mass`
  preservado) → converge ao masstone `c`, nunca mais escuro. `composite.wgsl`.
- **Gate:** `inv_overlap_saturates_to_pigment_not_black`. **Commit:** `c072c390`.

### B2 — xadrez/dither + centro oco em keep-wet/evap-0
- **Sintoma:** interior dos traços densos ditherava (buracos brancos entre células); centro esvaziava
  deixando anel.
- **Causa (2):** (a) **CFL combinada** — difusão+advecção somavam outflux >1 → célula negativa →
  `max(·,0)` = buraco. (b) **FlowOutward eterno** — edge-darkening é fenômeno de SECAGEM; dirigido só
  pelo gradiente de água, em keep-wet bombeava o interior pra borda pra sempre.
- **Fix:** (a) orçamento de CFL ÚNICO (`D_MAX=0.20`, `V_MAX=0.03`, `4·0.23=0.92<1`) → positivo por
  construção. (b) `flow_outward *= ((evap−0.004)/(0.012−0.004))²` no bridge → ~0 em keep-wet/evap-0.
- **Gate:** `inv_no_checkerboard_under_extreme_flow`. **Commit:** `cd33ffc6`.

### B3 — marcas retangulares + borda pixelada em Evaporation 0
- **Sintoma:** retângulos tonais embossados pela área pintada; borda em escada.
- **Causa (2):** (a) **costura de região** — o `cs_step` rodava só na janela móvel (30 frames); em
  evap-0 o footprint inteiro difunde, então células com contagens de step diferentes formavam degraus
  nos retângulos. (b) **frente molhada estática** — sem recessão de água o gate trava em borda dura
  de 1 célula (lição do v2).
- **Fix:** (a) região = **envelope molhado monotônico** (step+composite+copy) → toda célula evolui
  igual. (b) **recessão de água viesada à borda** (`EDGE_EVAP_FLOOR=0.01·(1−w)`) → só a borda fina
  recua e suaviza ao cruzar a banda do gate; interior intacto (keep-wet preservado).
- **Commit:** `fe0aa2e9`.

### B4 — mancha dura ao re-depositar sobre traço seco (wet-on-dry)
- **Sintoma:** depositar mais tinta encostando num traço seco → fronteira dura interna.
- **Causa:** a recessão (B3) congela a borda; pigmento novo (molhado) não funde no velho (congelado)
  — gate fechado onde não há água.
- **Fix:** **halo de água** no splat (`WATER_HALO=1.5`) — água molha disco mais largo/suave que o
  pigmento → re-molha a borda seca em que encosta → pigmento velho re-mobiliza e funde. Limitado ao
  raio do dab (sem espalhamento autônomo).
- **Commit:** `3cc143cd`.

### B5 — acúmulo de pigmento "marca os pixels" (mosqueado)
- **Sintoma:** áreas densas com mosqueado por-pixel.
- **Causa:** `gate()` modulava transporte por `mix(perm_valley, perm_crest, paper)`, e `paper` é
  ruído POR-PIXEL → gravava o grão no pigmento (exposto pelo cap perto do teto).
- **Fix:** removida a permeabilidade do papel do gate → transporte uniforme → mancha chapada.
  Granulação volta depois como feature v1.1 com campo de baixa frequência.
- **Commit:** `b20e1b36`.

### B5b — degrau núcleo↔halo (a "borda dura" persistente)
- **Sintoma:** miolo chapado + halo em gradiente, com uma LINHA de contorno dura entre os dois.
- **Causa:** o cap DURO `min(mass, MASS_MAX)` cria dois regimes (chapado vs. proporcional); a
  iso-linha `mass=MASS_MAX`, quantizada por pixel, é a escada.
- **Fix:** **saturação suave** — `eff = MASS_MAX·(1−exp(−mass/MASS_MAX))` → proporcional pra glaze
  fino, assintótico pro masstone, SEM descontinuidade de valor ou inclinação.
- **Commit:** `452c477b`.

### B6 — borda seca em escada (staircase) no zoom
- **Sintoma:** borda do miolo em escada ao dar zoom; o gradiente "seco" não suavizava.
- **Causa:** campo 1:1 com canvas; borda seca cai de cheio→0 em ~1 célula; composite amostra
  *nearest* (`inv=1`). Saturação suave (B5b) conserta valor, não o CONTORNO quantizado.
- **Fix:** **anti-alias no composite** — reamostra o campo com gaussiano (raio 2, σ≈1.2). Interior
  uniforme não muda; só bordas/gradientes suavizam (molhado E seco). `composite.wgsl`.
- **Gate:** INV-5 (passou a usar blocos; células isoladas eram diluídas pelo blur). **Commit:**
  `97ea380c`.

---

## §2 — Mapa: que camada controla qual artefato

| Artefato | Camada | Símbolo / knob |
|---|---|---|
| Overlap → preto / saturação | composite | `MASS_MAX`, `eff = MASS_MAX·(1−exp(−mass/MASS_MAX))` |
| Degrau núcleo↔halo | composite | saturação suave (acima) — NÃO usar `min()` duro |
| Escada de borda no zoom | composite | `BLUR_RADIUS`, `BLUR_SIGMA` (gaussiano em `sample_pig`) |
| Xadrez/dither | kernel `wash.wgsl` | `D_MAX`+`V_MAX` (orçamento CFL único) |
| Centro oco / over-bleed | bridge `wash_params_from` | `flow_outward` acoplado à secagem |
| Borda dura (frente estática) | kernel `wash.wgsl` | `EDGE_EVAP_FLOOR·(1−w)` |
| Wet-on-dry não funde | kernel `splat.wgsl` | `WATER_HALO` (água > pigmento) |
| Mosqueado por-pixel | kernel `wash.wgsl` | perm do papel REMOVIDA do `gate()` |
| Marcas retangulares | bridge | região = envelope monotônico (não janela móvel) |

**Invariante de física (não quebrar):** o `cs_step` é um gather conservativo (massa conservada). Os
fixes de B1/B5b/B6 são todos DISPLAY-side (composite) — não tocam a física. Os de B2/B3/B5 mexem no
kernel mas preservam conservação (verificada por `inv_mass_conserved_under_diffusion`).

---

## §3 — Checklist diagnóstico pra um novo artefato visual de aquarela

1. **Reproduz em Evaporation 0 / Keep Wet?** Se sim, suspeite de transporte que não relaxa (região
   móvel, advecção eterna, perm anisotrópica). Esse é o pior caso — comece por ele.
2. **É VALOR ou CONTORNO?** Valor errado (cor, escuro demais) → composite/saturação. Contorno
   serrilhado/quantizado → anti-alias do composite OU campo nítido demais.
3. **Aparece só em área densa?** Suspeite do cap expondo variação de massa (paper-perm, ruído).
4. **É xadrez regular (alterna célula sim/não)?** É CFL/positividade — some os orçamentos dos termos
   do gather; cheque `max(p_new,0)` cortando negativos.
5. **É retangular/alinhado à grade de tiles?** É costura de região (janela móvel vs. envelope) ou
   limite de workgroup — torne a região consistente entre frames.
6. **Mude UM knob por vez** e rode `tests/wash_invariants.rs` (`--features gpu -- --ignored`). Um gate
   verde com sintoma vivo = o gate não cobre o caso → adicione um gate ANTES de seguir.
7. **Construa o commit "claimed-green" e VEJA** — vários desses só foram resolvidos com screenshot do
   Enio; bench/teste verde ≠ vivo correto (ver [`feedback_tool_unit_green_integration_dead`]).

---

## §4 — Constantes atuais (e por quê)

| Const | Valor | Arquivo | Razão |
|---|---|---|---|
| `MASS_MAX` | 1.0 | composite.wgsl | capacidade de pigmento do papel; teto = masstone `c` |
| `BLUR_RADIUS`/`BLUR_SIGMA` | 2 / 1.2 | composite.wgsl | anti-alias da borda no zoom; aumentar = mais macio |
| `D_MAX` | 0.20 | wash.wgsl | orçamento de difusão (folga pra advecção) |
| `V_MAX` | 0.03 | wash.wgsl | orçamento de advecção; `4·(D_MAX+V_MAX)=0.92<1` |
| `EDGE_EVAP_FLOOR` | 0.01 | wash.wgsl | recessão de borda; suaviza rim mesmo em evap 0 |
| `WATER_HALO` | 1.5 | splat.wgsl | raio de água ÷ raio de pigmento; re-molha p/ fundir |
| `KEEP_WET_EVAP` | 0.004 | tool/lifecycle.rs | evap mínima em keep-wet (limiar do acople flow) |

Calibração visual é esperada — esses são pontos de partida validados por olho, não constantes
físicas (exceto a forma de Beer–Lambert e o orçamento de CFL, que são matemática dura).

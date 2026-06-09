# HANDOFF — Painter Watercolor: **Kubelka–Munk multi-pigment wet-on-wet** (do início ao fim)

> **Para uma LLM nova, em LOOP AUTÔNOMO.** Este é o **#1 da pesquisa** (`docs/Painter_projeto/pesquisa_aquarela_estado_da_arte.md`)
> e a Proposta 2 do `docs/Painter_projeto/avaliacao_e_melhorias.md`: a **mágica da aquarela que
> falta** — pintar **azul**, pintar **amarelo** molhado por cima, e ver sangrar num **verde
> vibrante** (mistura SUBTRATIVA real), não num cinza lamacento. Hoje o campo molhado carrega
> **cobertura cinza + UMA cor por traço** → cores NÃO se misturam. Você vai consertar isso.
>
> **Pré-requisito 100% pronto e validado pelo Enio:** o motor de aquarela S0–S5c (difusão gateada
> + deposição/edge-darkening + shallow-water/backruns + capilaridade transparente + BFECC/
> MacCormack). Não refaça nada disso. Norte: [ADR-0078](architecture/decisions/0078-watercolor-gold-standard-resident-tiled-shallow-water.md)
> + [ADR-0079](architecture/decisions/0079-watercolor-params-per-brush-exposure.md). 47 commits
> locais, NÃO pushados.

---

## §0 — PROTOCOLO DE LOOP AUTÔNOMO (leia primeiro, é a instrução-mestre)

**O Enio quer que você implemente isto do início ao fim, FASE A FASE, em loop
implementação → auditoria, SEM PARAR.** Concretamente:

1. **Trabalhe as 7 fases (P0–P6 do §3) em sequência, numa execução contínua.** Use uma
   `TodoWrite` com as 7 fases pra rastrear. Termine uma, comece a próxima imediatamente.
2. **Ao fim de CADA fase: AUDITE antes de seguir.** Rode os gates da fase (§3) + uma passada de
   revisão de corretude contra as invariantes (paridade bit-exata GPU↔CPU, conservação, o look
   single-color preservado). Para auditar, ou rode os gates + uma auto-revisão focada, OU spawne
   1 sub-agente de auditoria adversarial (lente: "este passo diverge da referência / quebra
   conservação / regride o look validado?"). Corrija TODO achado **na mesma fase** antes de
   avançar.
3. **Commit local por fase** (`git commit --no-verify -m "..." -- <suas paths>`), acumulando.
   **NÃO pushe** (decisão do Enio).
4. **NÃO PARE pra perguntar nem pra esperar validação visual.** Você tem o plano completo aqui.
   Não use `AskUserQuestion`. **O smoke (validação visual do Enio) fica pra AMANHÃ** — não tente
   rodar o app nem pedir confirmação; só deixe o handoff de smoke pronto (P6). Decida no
   padrão-ouro e siga ([feedback-decide-dont-ask](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_decide_dont_ask_gold_standard.md)).
5. **Só PARE se:** (a) todas as 7 fases estiverem implementadas + auditadas + commitadas, OU
   (b) você bater num bloqueador REAL fora da sua pasta que exija coordenação (regra de isolamento
   §0.5.2 — aí reporte e pare). Caso contrário, **continue até o fim**.
6. Se a janela de contexto encher no meio, o resumo te devolve o estado — continue de onde parou
   (a `TodoWrite` + os commits por fase são o seu marcador).

---

## §0.5 — Inegociáveis (memorize)

1. **Paridade é lei (HR-5).** A referência CPU (`ph2d_painter_brush`) é o ground-truth +
   det-fallback. **TODO passo GPU novo tem gate de paridade** vs a CPU
   (`crates/ph2d-painter-fluid/tests/{gpu_parity,composite_parity}.rs`, Metal `--ignored`). Mude
   a CPU e o espelho WGSL **identicamente**; mesma ordem de operações (cuidado com FMA — os passos
   existentes batem **0 ULP** em Metal, mantenha assim).
2. **Isolamento.** Edite só as crates do painter/fluid (lista no §4). O **contrato da
   representação do pigmento** que você vai mudar é foundational → isso EXIGE um **ADR (ADR-0080,
   P0)**; está in-scope porque o Enio pediu esta feature. Qualquer OUTRA necessidade fora dessas
   pastas: PARE e reporte.
3. **Inner loop = `cargo check -p <crate>`** no slot warm. Gates GPU + nextest 1× no fechamento
   (P6), não por task. RAM 8 GiB → ≤3 cargos simultâneos. Slot: prefixe cada cargo com
   `CARGO_TARGET_DIR="$PWD/target-slots/slot-brushoverhaul"` (warm; é o que o `./play.command` usa).
4. **Você NÃO pusha.** Commits locais `--no-verify`. `./scripts/ship.sh` + push = decisão do Enio.
5. **NÃO regrida o look validado (S0–S5c).** Um traço de UMA cor deve continuar com a aparência
   que o Enio ratificou (value-opacity ADR-0079, edge-darkening, capilaridade transparente,
   sharpness). O multi-pigmento ADICIONA mistura; o caminho single-pigmento deve reproduzir o look
   atual (ver §2.3 — é o maior risco). [Cerca de Chesterton](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_documented_decision_chesterton_fence.md):
   comentário "intentionally X" = decisão ratificada, não sobrescreva.
6. **Git anti-colisão.** `git add -- <suas paths>` (NUNCA `-A`/`.`); `git status` antes. **Há WIP
   ALHEIO no working tree** — `shells/desktop/src/input_dispatch/gizmo_drag.rs`,
   `shells/desktop/src/input_dispatch.rs`, `rot.log` (diagnóstico de gizmo do Enio). **NÃO toque,
   NÃO commite.**
7. **UI em inglês** (HR-15) mesmo o Enio descrevendo em pt-BR; labels/toasts inglês. Zero hex, zero
   `f32` de UI literal, zero string hardcoded — tokens/i18n.

---

## §1 — O alvo + o gap exato (medido no código)

**Objetivo:** mistura **subtrativa Kubelka–Munk** de pigmentos molhados — cores que sangram juntas
no campo molhado se misturam como TINTA REAL (azul+amarelo=verde), e o campo **persiste entre
traços** (wet-on-wet cross-stroke).

**O gap (verificado):**
- O campo molhado (`DiffusionGrid::pigment: Vec<[f32;3]>`, `crates/ph2d-painter-brush/src/diffusion.rs`)
  carrega hoje **cobertura CINZA** (`[dep/3; 3]`, dep = `WET_PIGMENT_DEPOSIT·opacity`) — cor-
  independente (ADR-0079). A **cor** vem do `pcol` (uniform, **UMA por traço**) no composite
  (`crates/ph2d-painter-brush/src/wet_composite.rs`). Resultado: **as cores não se misturam no
  campo** — dois traços de cores diferentes não fazem um terceiro; cada traço usa seu próprio `pcol`.
- O campo é **fresco a cada traço** (`lifecycle.rs:166` — *"a fresh field per stroke (v1 —
  cross-stroke wet-on-wet is a W15.3 refinement)"*). Então não há wet-on-wet entre traços.
- ✅ **O K–M óptico JÁ EXISTE — módulo dedicado `crates/ph2d-painter-brush/src/pigment_mix.rs`:**
  `prepare_pigment(color) -> PreparedPigment` deriva os coeficientes K/S de uma cor (a inversa
  cor→K/S já está lá), e `mix_prepared_exact(&prepared, backdrop, t) -> [f32;3]` faz o glaze K–M
  sobre o backdrop (usado por `wet_composite.rs`). **Reuse `pigment_mix.rs` — não reimplemente K–M;
  grep antes.** O que falta é o CAMPO carregar pigmento MISTURÁVEL (não cinza + 1 cor) + a
  persistência cross-stroke.

**Então o trabalho é, em essência:** trocar a representação do pigmento no campo (CPU+GPU) de
"cobertura cinza + 1 cor/traço" para "pigmento K–M misturável por-célula", reusando a matemática
K–M óptica que já existe no composite, + ligar a persistência cross-stroke.

---

## §2 — Arquitetura recomendada (você refina + congela no ADR-0080, P0)

### §2.1 — Representação do campo: K–M de constante única (mass-weighted K/S)

A escolha-chave do P0. **Recomendado** (auto-contido, sem assets, determinístico):

- Por célula, guarde **`Kc = Σ_i mass_i·(K/S)_i` (3 floats, por canal RGB)** + **`mass = Σ_i mass_i`
  (1 float)** → **4 floats/célula** (vs 3 hoje). `(K/S)_i` = coeficiente do pigmento i (de
  `prepare_pigment(cor_i)`). 
- **A mistura é AUTOMÁTICA e LINEAR:** `Kc` e `mass` transportam (diffuse/advect/capillary)
  exatamente como o pigmento hoje (conservativos, lineares) → quando dois pigmentos se encontram,
  `Kc` e `mass` somam → `(K/S)_mix = Kc/mass` (média ponderada por massa dos K/S) → a cor misturada
  emerge da fórmula de reflectância K–M. **É o ponto inteiro:** transporte linear de K/S = mistura
  subtrativa.
- **Composite:** `(K/S)_mix = Kc/mass` por canal → `R = reflectance(K/S)` (a cor molhada misturada);
  `alpha` vem de `mass` (a cobertura — substitui o `dens` de hoje). Reuse `mix_prepared_exact` pro
  glaze sobre o backdrop, mas com o `R` por-célula no lugar do `pcol` único.
- **Reflectância K–M de constante única:** `R = 1 + (K/S) − √((K/S)² + 2·(K/S))`; inversa (cor→K/S):
  `K/S = (1−R)²/(2R)`. (Confira em `prepare_pigment` — provavelmente já faz a inversa.)

**Alternativa de maior fidelidade (avalie no P0, NÃO default):** K–M de DUAS constantes (`Kc[3]` +
`Sc[3]` = 6 floats, K e S separados) ou **Mixbox** (Sochorová & Jamriška 2021, código MIT
`github.com/scrtwpns/mixbox`: RGB→4 coeffs latentes via LUT→mix linear→RGB). Mixbox é perceptual-
mente mais preciso mas precisa do LUT (~asset binário) — só adote se a integração for limpa e o
Enio aceitar a dependência. **Default: constante única, auto-contido.** Documente a escolha + o
trade-off no ADR-0080 ([no-industrial-claims](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md):
nada de claim sem grep/teste).

### §2.2 — Camadas que viram K/S

Tudo que hoje é pigmento `[f32;3]` cinza vira o par `(Kc, mass)`:
- **Flowing** (`DiffusionGrid::pigment`, GPU `pig_a`/`pig_b`).
- **Deposited** (`DiffusionGrid::deposited`, GPU `deposited`) — pigmento congelado também é K/S.
- **Total** (GPU `cs_combine`: `total = flowing + deposited`) — soma de K/S.
- O **dab** (`lifecycle.rs`) deposita `(Kc, mass)` da cor do dab (via `prepare_pigment`), não `[dep/3;3]`.

### §2.3 — ⚠️ O MAIOR RISCO: reconciliar com o look single-color validado

O modelo atual (cinza + `pcol` + `color_sum = 0.3+0.7·value` + `alpha = 1−exp(−dens/color_sum·k)`)
foi tunado e validado (ADR-0079: value-opacity, preto/escuro pintam, dark colors cobrem). **O novo
modelo K/S DEVE reproduzir esse look pra um único pigmento** (senão regride tudo). Mapeie:
`mass` ≡ o `dens`/cobertura de hoje (mesma deposição + value-opacity), e `(K/S)` do único pigmento
= `prepare_pigment(cor)` → `R = cor`. Valide CEDO (P1/P2) que single-pigmento = look atual (gate de
paridade do composite ≤ tolerância). **Não avance pro multi sem isso verde.** Veja
[unit-verde≠funciona-no-produto](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_tool_unit_green_integration_dead.md).

### §2.4 — Cross-stroke wet-on-wet (P4)

Hoje o campo é fresco por traço (`lifecycle.rs:166`) + o bridge dropa o field quando seca
(`fluid_dry_check_and_drop_gpu`). Pra wet-on-wet entre traços: o field **persiste enquanto molhado**;
um traço novo deposita `(Kc, mass)` no field ainda úmido → mistura com o pigmento do traço anterior.
Toca o epoch handling (`lifecycle.rs` `begin_stroke`/`fluid_stroke_epoch`) + o bridge
(`painter_fluid_bridge.rs` — o reset por epoch). Mantenha o envelope (§2.2 do
`HANDOFF_painter_capillary_edge.md`) e o dry-drop (só dropa quando REALMENTE seco).

---

## §3 — AS 7 FASES (implemente → audite → commite cada uma)

> Cada fase: **deliverable + GATES**. Audite (gates + revisão de corretude) antes de seguir. Commit
> local por fase. Inner loop = `cargo check -p`.

### **P0 — Design + ADR-0080** (sem código de produção ainda)
- Leia: `pesquisa_aquarela_estado_da_arte.md` (§2/§3 K–M), `diffusion.rs` (pigment/diffuse/advect/
  transfer), **`pigment_mix.rs`** (`prepare_pigment`, `PreparedPigment`, `mix_prepared_exact` — a
  matemática K–M), `wet_composite.rs` (`composite_wet_field_cpu`). Confirme o que `prepare_pigment`
  já computa (a inversa cor→K/S) — grep antes de qualquer K–M novo.
- Decida a representação (§2.1): **constante única `(Kc[3], mass)`** salvo razão forte pro contrário.
- Escreva **`docs/architecture/decisions/0080-watercolor-km-multipigment-field.md`** congelando: a
  representação do campo (Kc+mass por célula), a função pigmento→K/S, a fórmula de reflectância, e
  como o single-pigmento reproduz o look (§2.3). Registre o trade-off vs Mixbox.
- **GATE P0:** ADR escrito + um **teste unitário da matemática** em `wet_composite.rs` ou um módulo
  novo: `azul (K/S) + amarelo (K/S) → R verde` (componente G do `R` mistura > a dos puros; não vira
  cinza/lamacento). Determinístico. Commit.

### **P1 — CPU: campo K/S misturável** (`ph2d-painter-brush`)
- Estenda `DiffusionGrid`: pigment/deposited viram `(Kc, mass)` (ou um struct `Pigment { ks:[f32;3],
  mass:f32 }`). `splat` deposita `(Kc, mass)` de uma cor. `diffuse`/`advect`/`advect_maccormack`/
  `transfer_pigment`/`capillary_flow` transportam Kc E mass (conservativos — espelhe a estrutura
  atual, só com 4 canais). Acessor `reflectance()` → cor por célula via K–M.
- **GATES P1:** (a) **mistura** — blob azul + blob amarelo sobrepostos molhados → reflectância verde
  na zona de overlap; (b) **conservação** — Kc e mass conservados sob diffuse/advect (como hoje);
  (c) **determinismo** (HR-5); (d) **single-pigmento reproduz** — um traço de 1 cor → reflectância =
  a cor, e a massa = a cobertura de antes (§2.3). Reuse os padrões dos testes existentes em
  `diffusion.rs`. `cargo test -p ph2d-painter-brush --lib`. Commit.

### **P2 — CPU: composite por-célula** (`ph2d-painter-brush::wet_composite`)
- O composite lê `(Kc, mass)` por célula: `alpha = f(mass)` (substitui `dens`; preserve a value-
  opacity), `R = reflectance(Kc/mass)` por célula (substitui `pcol` único), glaze K–M sobre o
  backdrop via `mix_prepared_exact` com `R` por-célula. `prepare_wet_composite_from_stroke` perde o
  `pcol` único (ou vira fallback).
- **GATES P2:** (a) **paridade single-color** — um traço de 1 cor composita ≈ idêntico ao composite
  atual (≤ tolerância; o gate guarda o §2.3); (b) **mistura composita** — overlap azul/amarelo
  composita verde. `cargo test -p ph2d-painter-brush --lib wet_composite`. Commit.

### **P3 — GPU mirror** (`ph2d-painter-fluid`) — o bloco grande
- Os buffers GPU carregam `(Kc, mass)`: `pig_a`/`pig_b`/`deposited`/`total` viram 4-canais (ou 2
  buffers). diffuse/advect(+MacCormack rev/correct)/transfer/combine/splat/capillary operam em Kc+
  mass. O composite GPU (`composite.wgsl` + `composite.rs`) lê `(Kc, mass)` → reflectância.
- Espelhe a matemática CPU **bit-a-bit** (mesma ordem de ops; os passos atuais batem 0 ULP).
- **GATES P3:** estenda `tests/gpu_parity.rs` + `composite_parity.rs` — cada passo K/S bate a CPU
  (`worst |Δ| < 2e-2`, idealmente 0 ULP) + naga valida os shaders (`tests/contract_surface.rs`) +
  os 13 gates atuais seguem verdes. `cargo test -p ph2d-painter-fluid --features fluid --test
  gpu_parity --test composite_parity -- --ignored`. Rode no Metal (você está em Mac). Commit.

### **P4 — Cross-stroke wet-on-wet** (`ph2d-tool-painter` + bridge)
- O field persiste enquanto molhado (§2.4); um traço novo deposita no field úmido → mistura com o
  anterior. Ajuste o epoch/reset (`lifecycle.rs` `begin_stroke`, `painter_fluid_bridge.rs`). Mantenha
  o dry-drop (só seco) e o envelope.
- **GATE P4:** headless/unit — um traço amarelo num field com pigmento azul úmido → zona de mistura
  verde (sem GPU se der; senão um gate no fluid). Não-regressão: traços separados (não-sobrepostos)
  inalterados. Commit.

### **P5 — Seleção de pigmento + controles** (`ph2d-painter-brush` + panel/tool)
- Como o brush escolhe o pigmento: a cor do traço → K/S direto (mínimo viável), OU uma paleta de
  pigmentos reais (cada um com K/S tunado — staining, granulação por-pigmento). Decida o escopo
  mínimo que entrega "azul+amarelo=verde" + o look preservado. Se adicionar controle per-brush:
  cabe no `WatercolorParams` (cap ≤18, **1 folga** — ADR-0079-amendment-1; 17/18 usados hoje) OU
  emenda do cap. Atualize o gate `architecture_painter_contract_surface`.
- **GATE P5:** round-trip panel↔tool (`cargo test -p ph2d-panel-brush-studio`) + contract gates
  (`cargo test -p ph2d-painter-contracts`). Commit.

### **P6 — Auditoria final + handoff de smoke**
- **Auditoria ≥2 lentes** sobre o diff acumulado (correção K–M, paridade GPU↔CPU, conservação,
  look single-color preservado, perf). Pode spawnar sub-agentes adversariais. Corrija tudo.
- Gate batched: `scripts/nextest-impacted.sh` (ou nextest dos crates tocados) + clippy
  `--all-targets` + os gates GPU 1× + naga/ABI. Tudo verde.
- **Escreva `docs/HANDOFF_painter_km_smoke.md`** pro Enio validar AMANHÃ: exatamente o que olhar
  (azul+amarelo molhados = verde vibrante; single-color = look idêntico; cross-stroke = mistura;
  preto/escuro ainda pintam; capilaridade/sharpness intactos) + os comandos. **NÃO rode o app.**
- Commit final. **FIM do loop.**

---

## §4 — Arquivos-chave (suas paths)

- **Campo CPU (ref + física):** `crates/ph2d-painter-brush/src/diffusion.rs` (`DiffusionGrid`:
  `pigment`, `deposited`, `splat`, `diffuse`, `advect`, `advect_maccormack`, `transfer_pigment`,
  `capillary_flow`, `step`).
- **K–M óptico (REUSE — não reimplemente):** `crates/ph2d-painter-brush/src/pigment_mix.rs`
  (`prepare_pigment` = cor→K/S, `PreparedPigment`, `mix_prepared`/`mix_prepared_exact` = mistura/
  glaze K–M). **Esta é a fonte da matemática K–M.**
- **Composite:** `crates/ph2d-painter-brush/src/wet_composite.rs` (`composite_wet_field_cpu`,
  `WetCompositeBrush`, `prepare_wet_composite_from_stroke` — consome `pigment_mix.rs`).
- **Dab deposit:** `crates/ph2d-tool-painter/src/tool/lifecycle.rs` (`WET_PIGMENT_DEPOSIT`,
  `WET_WATER_DEPOSIT`, o `[dep/3;3]` cinza nos 2 sites ~813/845, `begin_stroke`, epoch).
- **Solver GPU:** `crates/ph2d-painter-fluid/src/solver.rs` (`FluidSolver`: `pig_a`/`pig_b`/
  `deposited`/`total`, todos os `cs_*` passes, `GpuParams` 96B, `set_from_diffusion`,
  `step_resident_splat`) + `src/shader/{fluid,splat,transfer,combine,shallow,capillary}.wgsl`.
- **Composite GPU:** `crates/ph2d-painter-fluid/src/composite.rs` + `src/shader/composite.wgsl`.
- **Bridge (cross-stroke/epoch):** `shells/desktop/src/render_loop/painter_fluid_bridge.rs`.
- **Brush params/UI:** `crates/ph2d-painter-brush/src/watercolor.rs` (`WatercolorParams`) +
  `crates/ph2d-panel-brush-studio/` + `crates/ph2d-tool-painter/src/params.rs`.
- **Contratos:** `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`.
- **Gates GPU:** `crates/ph2d-painter-fluid/tests/{gpu_parity,composite_parity,contract_surface}.rs`.

---

## §5 — Referência K–M + comandos

**Matemática (constante única):** reflectância `R = 1 + KS − √(KS² + 2·KS)` com `KS = K/S`;
inversa `KS = (1−R)²/(2R)`. Mistura: `KS_mix = (Σ mass_i·KS_i)/(Σ mass_i)` (média ponderada por
massa). Por canal RGB. Fontes (na pesquisa): Curtis et al. 1997 "Rendering the Pigmented Layers";
Sochorová & Jamriška 2021 "Practical Pigment Mixing" (Mixbox). **Reuse `pigment_mix.rs::prepare_pigment`**
— ele já faz a inversa cor→K/S; grep antes de reimplementar.

```bash
SLOT='CARGO_TARGET_DIR="$PWD/target-slots/slot-brushoverhaul"'   # prefixe cada cargo
# Inner loop:
cargo check -p ph2d-painter-brush          # ou -p ph2d-painter-fluid --features fluid ; -p ph2d-host-desktop --features fluid
# CPU (P1/P2):
cargo test -p ph2d-painter-brush --lib
# GPU parity (P3, Metal, --ignored, no fechamento):
cargo test -p ph2d-painter-fluid --features fluid --test gpu_parity --test composite_parity -- --ignored --nocapture
# naga/ABI:
cargo test -p ph2d-painter-fluid --features fluid --test contract_surface
# Panel/contracts (P5):
cargo test -p ph2d-panel-brush-studio ; cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
```

**Git por fase:** `git add -- <suas paths>` então `git commit --no-verify -m "feat(painter): K–M PX — ..." -- <suas paths>`.
Termine mensagens de commit com `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
**Não toque** `gizmo_drag.rs`/`input_dispatch.rs`/`rot.log` (WIP alheio).

---

## §6 — Estado de entrada (não refazer)

Pré-requisito 100% pronto, validado pelo Enio, 47 commits locais (não-pushados): difusão gateada +
deposição/edge-darkening + shallow-water/backruns (Flow Velocity + Backrun esvaziam o centro =
"aguado") + capilaridade transparente (filtragem cromatográfica) + diluição global via Opacidade +
BFECC/MacCormack (Sharpness 0–2.5, GPU bit-exato). O K–M óptico do GLAZE já existe (`mix_prepared_exact`).
**O que falta é só o que este handoff descreve: o CAMPO multi-pigmento + cross-stroke.**

— deixado por Claude (sessão 2026-06-09): motor de aquarela S0–S5c completo + validado; próximo é o
**K–M multi-pigmento wet-on-wet** (#1 da pesquisa). Loop autônomo P0–P6, smoke do Enio amanhã.

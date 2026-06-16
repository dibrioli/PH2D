# ADR-0092 — Wash: borda de aquarela (edge-darkening rim + franja capilar)

- **Status:** ACEITO (Enio, 2026-06-14); implementado, **pendente smoke visual**.
- **Contexto:** [ADR-0086 §8](0086-watercolor-minimal-core-wash.md) planejou a franja capilar como add-back #2 (o #1, cor K–M/Mixbox, fechou no [ADR-0091](0091-wash-mixbox-residual-faithful-pigment-color.md)). [ADR-0085 §0](0085-watercolor-v2-gpu-first-realtime.md) manda "borda de deposição realista". Smoke do Enio mostrou que faltava a **borda escura** (edge-darkening) — *o* efeito nº1 da aquarela. Pesquisa do algoritmo (Curtis et al., SIGGRAPH 1997) + auditoria do nosso app revelaram a divergência exata (§2).
- **Estende:** o núcleo do [ADR-0086](0086-watercolor-minimal-core-wash.md). **Opt-in** (coeficientes default 0). Modelo de UI = **Rebelle** (controles "Edge Darkening" + difusão separados, [manual](https://escapemotions.com/products/rebelle/manual/8/interface/panel-visual-settings/)).

## 1. Problema — a água não se move, e a borda não escurece

No núcleo v1 a água **só decai**: a região molhada é o footprint e só encolhe — sem sangramento e **sem o rim escuro**. A aquarela real tem DUAS assinaturas de borda, de mecanismos opostos:
- **Edge-darkening (rim):** ao **secar**, a água evapora mais rápido na borda fina; o interior repõe via capilaridade, **carregando pigmento que encalha e ESCURECE a borda** (Curtis 1997, *FlowOutward*).
- **Franja capilar (bleed):** wet-on-wet, a água avança no papel e a cor **esmaece** até transparente.

## 2. Decisão — dois efeitos, um gate (min-gate)

### (a) Edge-darkening RIM — slider "Bleed" (`flow_outward`)
O **FlowOutward** de Curtis (§4.3): remove água extra na **banda de fronteira** molhado/seco (proxy: o gradiente da máscara molhada `mgrad = Σ|gate(wc)−gate(wn)|`, ≈ a distância-à-borda do blur Gaussiano de Curtis). Isso mantém o rim **mais seco** que o centro, SUSTENTANDO a advecção do pigmento para a borda. O pigmento **empilha JUSTO POR DENTRO** da interface molhado/seco e **não atravessa para o papel seco** — porque o flux usa `gf = min(gc,gn)` (≈0 num vizinho seco). Ao secar, o gate fecha e **congela o rim escuro**.

### (b) Franja capilar BLEED — slider "Capillary" (`wick`)
Água livre (acima de `W_ABSORB`) difunde no papel seco; o pigmento **co-advecta** para as células que o wick acabou de molhar (lá `gate(n) > 0`, então o `min-gate` deixa passar) — esmaecendo atrás da frente d'água = a franja-pena.

> **A descoberta (auditoria):** eu tinha trocado o gate da advecção para *donor-gate* (a célula mais molhada empurra), o que **EJETAVA pigmento para o papel seco** (vira bleed) em vez de empilhar no rim — **quebrava o edge-darkening**. O **min-gate** serve os DOIS: bloqueia a travessia para seco (rim) E deixa o pigmento seguir o wick para a franja molhada (bleed). Foi o erro que custou os smokes "não funciona".

### 2.1 Conservativo + positivo
Difusão + advecção (FlowOutward + drift capilar) partilham UM orçamento CFL: `4·(D_MAX + V_MAX) = 4·(0.12 + 0.12) = 0.96 < 1` ⇒ sem negativos ⇒ sem checkerboard (INV-8). Tudo anti-simétrico ⇒ **massa de pigmento conservada** (INV-11/12: `109→109`). O FlowOutward e a evaporação só REMOVEM água ⇒ água não-crescente.

### 2.2 Limites
- **Wick (bleed):** bounded por construção — só água livre move ⇒ raio finito mesmo a evap 0 (`Σágua/W_ABSORB`, INV-11 `far=0`). Sempre ligado (fenômeno de MOLHAR).
- **FlowOutward (rim):** rampado por `dry_drive` (0 no keep-wet 0.004 — senão a frente não-secante bombearia o centro oco; cheio a 0.008, MODERADO, onde o rim de fato se forma — não a 0.012 onde o wash seca antes). É um fenômeno de SECAR.

## 3. Implementação (dentro de `ph2d-painter-wash` + ponte + painel wash)

- **`solver.rs`:** `WashParams.wick` no pad livre (UBO 64 B, zero ABI).
- **`shader/wash.wgsl`:** `face()` = difusão `min(gc,gn)` + advecção `clamp(flow_outward·∇w + wick·∇freew, ±V_MAX)` com flux `gf·v·up` (**min-gate** — o coração do rim/bleed); FlowOutward `EDGE_RATE·flow_outward·gc·mgrad` removendo água da banda de borda; wick da água; `D_MAX 0.12 / V_MAX 0.12` (orçamento p/ a velocidade de migração que faz o rim).
- **`painter_wash_bridge.rs`:** `wick: dp.capillary` (slider vivo, §abaixo); `flow_outward: dp.flow_outward · dry_drive` (ramp linear cheio a 0.008).
- **`ph2d-panel-brush-studio` (sections.rs):** o slider **"Capillary"** (`WatercolorParams::CONTROLS[15]`) entrou na seção Wash (reusa o edit path; cap de 21 controles cheio, sem campo novo). O rim usa o slider **"Bleed"** (`flow_outward`, já presente).

## 4. Invariantes / validação (headless, Metal — 12/12)

- **INV-12 (rim, NOVO):** de um wash realista (interior chapado + falloff de borda), o FlowOutward **auto-forma um rim** — banda externa **1.46×** o centro (`1.32` vs `0.90`), e `r > borda = 0` (o min-gate segura o pigmento DENTRO ⇒ borda externa nítida). Sem edge-darkening (flow0) o perfil fica chapado (1.00).
- **INV-11 (bleed):** wick leva pigmento à franja (`ring 0→19.4`), bounded (`far=0`), massa conservada, sem centro oco.
- **INV-1..10 inalterados** (coeficientes default 0).

## 5. Consequências

- **+** As DUAS bordas de aquarela: rim escuro (edge-darkening) **e** franja macia (capilar), **separáveis** (Bleed vs Capillary, modelo Rebelle), em gates executáveis (INV-11/12).
- **+** Diagnóstico ancorado em pesquisa (Curtis FlowOutward) + auditoria (o bug do donor-gate). Seguro: conservativo, positivo, bounded.
- **−** `mgrad` é a aproximação local (1-célula) do blur Gaussiano de Curtis — o rim puxa do *outer band* (migração limitada pelo tempo de secagem), não do wash inteiro. Forte o bastante (1.46×); se um rim mais largo for pedido, o follow-up é o blur Gaussiano da máscara (custa 1 buffer — esbarra no limite de 8 storage/stage, exige repack).
- **−** `D_MAX` caiu p/ 0.12 (bloom máximo menor) para financiar a migração do rim.
- **Follow-ups:** expor um slider **"Edge Darkening"** dedicado renomeando o "Bleed" no wash; granulação v1.1 (borda dendrítica via campo de papel).

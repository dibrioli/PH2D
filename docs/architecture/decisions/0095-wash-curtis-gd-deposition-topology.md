# ADR-0095 — Wash: topologia Curtis `g`/`d` (suspenso/depositado) com TransferPigment

- **Status:** ACEITO (Enio, 2026-06-15).
- **Data:** 2026-06-15.
- **Plano:** [`docs/Painter_projeto/16_plano_migracao_curtis_gd.md`](../../Painter_projeto/16_plano_migracao_curtis_gd.md).
- **Fonte canônica de parâmetros:** [`wash_parametros_canonicos.md`](../../Painter_projeto/wash_parametros_canonicos.md)
  (pesquisa 2026-06-15: Curtis 1997, Stam, Kubelka–Munk/Mixbox, MoXi, consenso de produtos).
- **Auditoria que motiva:** varredura multiagêntica 2026-06-15 — **34 achados confirmados, 0 refutados**.
- **Supersede (parcial):** a **deposição IMPLÍCITA** da [ADR-0094](0094-wash-gpu-resident-simplified-core.md)
  (wet-gate congela pigmento) e os ADR-0092 (capillary fringe ad-hoc). **MANTÉM:** a cor Mixbox residual
  ([ADR-0091](0091-wash-mixbox-residual-faithful-pigment-color.md)) e a topologia GPU-residente/single-submit
  ([ADR-0093](0093-gpu-resident-painter-canvas.md)/0094).

## 1. Contexto — por que mudar

As três tentativas de wash/fluid divergiram entre si e nenhuma é reprodutível porque **nenhuma implementou o
modelo canônico**: todas fizeram reduções ad-hoc com constantes inventadas. A auditoria provou que o
erro-raiz é estrutural (achado `no-suspended-deposited-separation`, **CRÍTICO**): o solver tem um **campo de
cor único** (`pig`/`dye`/`res`) advectado pelo mesmo operador, e "deposição" é apenas o wet-gate congelando o
pigmento onde ele está — `grep TransferPigment|deposit|g_k|d_k` retorna **zero**.

Consequências provadas: diluição e "seca-escurece" são fisicamente impossíveis (viraram hacks —
`pigment_load`, `EDGE_EVAP_FLOOR`); staining (ω) e granulação (γ) por pigmento não existem; o edge-darkening
é um proxy ad-hoc; e ~12 constantes não têm equivalente publicado.

## 2. Decisão

Migrar o solver Wash para a **topologia Curtis 1997 completa**, com constantes **NOMEADAS e PUBLICADAS**:

1. **Separar pigmento por estado**, por pigmento `k`:
   - `g_k` — **suspenso** (na camada de água / shallow-water).
   - `d_k` — **depositado** (fixado no papel).
   - Espessura óptica enviada à cor = `x_k = g_k + d_k`.
2. **TransferPigment(ρ, ω, γ)** — passo de adsorção/dessorção entre `g` e `d` (Curtis §4.5):
   ```
   Δdown = g · (1 − h·γ) · ρ            # adsorção  (suspenso → depositado)
   Δup   = d · (1 + (h−1)·γ) · ρ / ω    # dessorção (depositado → suspenso)
   clamp d≤cap, g≤cap ;  d += Δdown−Δup ;  g += Δup−Δdown
   ```
3. **Shallow-water de SUPERFÍCIE LIVRE** (decisão Enio "completo" 2026-06-15; refinado na impl C2) —
   campo de velocidade `u,v` = `−∇(altura de água) − slope·∇h`, arrasto `κ=0.01`, CFL `|v|<V_MAX`;
   `MoveWater`+`MovePigment` = advecção donor-cell conservativa de água+`g`; `FlowOutward` Eq.3
   (`η∈[0.01,0.05]`) p/ edge-darkening. **SEM projeção de pressão/RelaxDivergence:** aquarela é um filme
   fino de superfície livre (compressível — a água espalha E a altura cai); a projeção incompressível
   CANCELA justamente o espalhamento radial que produz a diluição (provado pelo gate `flow_spreads`).
   É o modelo lubrication/porous-medium, mais correto p/ filme fino que o shallow-water inercial. A
   difusão do `cs_step` suaviza o miolo (anti-pinning da advecção simétrica).
4. **Camada capilar / backruns** (Curtis §4.6) — saturação `s`, capacidade `c`, difusão ≈0.25, expansão de máscara.
5. **Pigmentos reais** — ρ/ω/γ (e cor) da tabela de 11 pigmentos Curtis §1.7.

## 3. Princípios (inegociáveis)

- **GPU-first, tempo-real-only, ZERO fallback CPU** (herda ADR-0094 §2).
- **Diluição = CONCENTRAÇÃO de pigmento do pincel** (decisão Enio 2026-06-15, após **investigação
  empírica**). A investigação (`tests/wash_investigation.rs`, medindo o caminho ativo) PROVOU que: (a) a
  cobertura NUNCA dependeu de água em nenhuma versão (o composite do backup nem binda água — `cobertura =
  1−exp(−massa/0.6)`); (b) a diluição-por-espalhamento do C2 NÃO funciona na prática (a água satura em 1.0
  no splat ⇒ interior chapado ⇒ ∇água=0 ⇒ escoamento inerte no miolo; + re-injeção contínua acumula `g`).
  Logo a diluição vem da **concentração do pincel**: `conc = 1/(1+K·water_add)` escala o `g` SUSPENSO
  injetado (água↑ ⇒ menos pigmento/área ⇒ transparente). É entrada FÍSICA (par water:load do Rebelle /
  Dilution do Procreate) que flui pela física, NÃO o `pigment_load` antigo (aquele multiplicava a
  COBERTURA num modelo sem g/d). **Provado:** água 1.0 ⇒ FORÇA 27 (transparente) vs água 0.1 ⇒ 107.
- **Mistura:** já funcionava (o composite soma `g+d` antes da razão ⇒ azul+amarelo→verde; provado:
  sobreposição = `(113,204,126)`). A deposição (travar a 1ª cor) é o que a SEGURA no lugar p/ misturar —
  desacoplar a deposição do molhado a PIOROU (a 1ª cor dispersa antes da 2ª chegar), então NÃO se faz.
- **Constantes nomeadas, não inventadas.** Cada constante mapeia a um valor publicado (Curtis §1, MoXi §3) ou
  é explicitamente marcada art-directable (as não-publicadas: capilar α/ε/δ/σ §1.6). SAEM: `D_MAX`, `V_MAX`,
  `FIELD_CAP`, `WATER_HALO`, `dry_drive`, `EDGE_EVAP_FLOOR`, `COVER_K` (ou re-fundamentado), `MIN_PIGMENT_LOAD`.
- **Cor permanece Mixbox residual** (ADR-0091, auditado correto). A ausência de scattering `S` **não é bug**:
  granulação/staining entram pela DEPOSIÇÃO (TransferPigment sobre `d`), não pela cor.

## 4. Estado do solver (campos)

| Campo | Formato | Papel | Status |
|---|---|---|---|
| `water` | `r32f` | altura/volume de água | mantém |
| `vel` | `rg32f` | velocidade `u,v` (staggered) | **novo (C2)** |
| `pressure` | `r32f` | pressão `p` | **novo (C2)** |
| `g` (pig/dye/res) | `rgba32f` ×3 | pigmento SUSPENSO (renomeia os atuais) | re-papel |
| `d` (pig/dye/res) | `rgba32f` ×3 | pigmento DEPOSITADO | **novo (C1)** |
| `sat` | `r32f` | saturação capilar `s` | **novo (C3)** |
| `paper` | `r32f` | altura/tooth `h` (estático) | ativa (hoje morto) |

## 5. Contratos / ABI afetados (rebump aqui, não pingado por fase)

- `Dab` (64B) e `WashParams` (64B) — `abi_sizes_are_frozen` em `solver.rs`. Mudam ao adicionar `d`/ρ/ω/γ/vel.
  **Rebump autorizado por este ADR**; novos tamanhos travados nos testes ao fim de cada fase que os toca.
- `architecture_painter_contract_surface` (PainterParams field cap = 12): os 3 sliders que faltam
  (Load/Dilution/Mixing) podem estourar o cap → avaliar manter tudo no sub-struct `WashUiParams` (1 campo).
- Downcast allowlist já cobre `painter_wash_gpu.rs`.

## 6. Consequências

**Positivas:** diluição/seca-escurece/edge-darkening/granulação/staining/backruns passam a ser FÍSICOS e
emergentes; sistema reprodutível e auditável contra um modelo publicado; fim do trial-and-error de constantes.
Conserta de quebra os 2 bugs vivos (undo morto, switch-target) no C5 sobre o campo já settled.

**Custos/riscos:** mais campos GPU (g+d+vel+pressure) → wet-bbox (C5) vira o orçamento real p/ 4K (MoXi fez
512²@44fps em HW 2005, então canvas-res é viável); ABI churn concentrado aqui; risco de instabilidade do
shallow-water mitigado por Δt adaptativo + fallback Stam. Regressão B1–B9 vigiada por gate a cada fase.

## 7. Alternativas rejeitadas

- **Manter a deposição implícita + mais band-aids** — é o que gerou 3 versões irreprodutíveis. Rejeitado (a auditoria é o veredito).
- **Núcleo g/d sem shallow-water** (só C1) — resolveria diluição/seca-escurece, mas sem fluxo dirigido/backruns
  fiéis. Rejeitado pelo Enio (2026-06-15: "shallow-water completo").
- **Trocar a cor Mixbox por K–M glaze com scattering** — a cor residual está auditada como correta; granulação
  vem da deposição. Rejeitado (não mexer no que está certo).

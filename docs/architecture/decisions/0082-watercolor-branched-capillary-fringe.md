# ADR-0082 — Watercolor: branched (fiber-channeled) capillary fringe — opt-in

**Status:** Accepted (2026-06-09) — pedido pelo Enio (#2 da fila): "MoXi/LBM na franja capilar
(franja ramificada fibra-a-fibra)", com a instrução EXPLÍCITA: *"cuidado para não destruir o que já
temos; coloque novos parâmetros para introduzir o efeito quando o usuário assim desejar, e não
sobreescreva o que já temos."*
**Decisor(es):** Enio (dono/decisor) + Claude.
**Estende:** [ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md) (S5 capilaridade), [ADR-0080](0080-watercolor-km-multipigment-field.md)/[ADR-0081](0081-watercolor-real-pigment-palette.md) (campo 32-ch).
**Tags:** painter, watercolor, capillary, fiber, non-destructive, contract-surface, gpu-parity

---

## 1. Contexto

A franja capilar de hoje ([ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md)
S5, `capillary_flow`) é uma **difusão de água isotrópica ponderada por permeabilidade** — wicka
para fora num anel macio + uniforme. Aquarela real numa folha rugosa wicka **fibra-a-fibra**: a água
segue os canais do papel e a franja fica **ramificada/dendrítica** (lobos + dedos), não um anel liso.
A pesquisa (#3, `pesquisa_aquarela_estado_da_arte.md`) aponta MoXi/LBM (Chu & Tai 2005) como o
método canônico — mas é um **substrato LBM novo** (alto esforço/risco, reescreveria a capilaridade).

## 2. Decisão — fiber-channeled, NÃO um substrato LBM novo; OPT-IN, NÃO-DESTRUTIVO

### 2.1 Um novo parâmetro, default 0 (não sobreescreve nada)
`WatercolorParams.capillary_branching` ∈ [0,1], **default 0**. Em 0 a capilaridade é **bit-idêntica**
à de hoje (o gate `capillary_*` existente continua passando). O artista sobe o "Branching" para
introduzir a ramificação — exatamente a instrução do Enio.

### 2.2 Modulação da condutância pela fibra do papel (estável, conservativa, parity-exata)
Cada face da difusão capilar tem condutância `cond = 0.5·(perm_c+perm_n)`. Multiplicamos por um
**fator de fibra** derivado do papel na face:
```
fiber = 0.5·(paper[c] + paper[n])          // ∈ [0,1], média de face do tooth (já é o que perm usa)
fiber_factor = 1 − capillary_branching·(1 − fiber)   // ∈ [1−branching, 1]
cond_face = cond · fiber_factor
```
- **Por que SÓ supressão (≤1, nunca boost):** a estabilidade de `capillary_flow` depende de
  `capillary·cond ≤ 0.24` (perm ≤ 1 ⇒ a água é média convexa ⇒ fica em [0,1] sem clamp). Um boost
  (cond > original) quebraria isso. Suprimir (fiber_factor ≤ 1) preserva a média convexa →
  **estável + conservativo** (a mesma prova de S5). `branching = 0 ⇒ fiber_factor = 1 ⇒ idêntico`.
- **Efeito:** com branching > 0 a água wicka MENOS nas faces de baixo-papel (vales) e ~full nas de
  alto-papel → a frente avança irregular, **lobada/ramificada** seguindo o tooth, em vez do anel
  liso. O pigmento co-advecta pela MESMA `cond_face`, então a cor segue os dedos.
- **Por que o papel (não um ruído novo):** o campo de papel JÁ está no CPU **e** na GPU (uploadado),
  é coerente (simplex em `HEIGHT_FREQ`), e é fisicamente a fibra. Logo **paridade GPU↔CPU é trivial**
  (ambos leem o mesmo `paper`), zero buffer/ruído-WGSL novo. (A frequência do tooth dá lobos de
  ~poucas células; dendritos finos verdadeiros = o LBM completo, follow-up.)

### 2.3 Honestidade (não é LBM)
Isto **aproxima** a ramificação MoXi/LBM via condutância fiber-canalizada — NÃO é um solver
Lattice-Boltzmann. O LBM real (dendritos finos fibra-a-fibra) fica como opção futura de maior
fidelidade (substrato novo). [feedback-no-industrial-claims]: sem alegar LBM; o efeito é a
modulação de condutância acima, verificável no código + gates.

## 3. Impacto em contratos
- **`WatercolorParams ≤ 18` → `≤ 20`** (ADR-0082): `+capillary_branching` (índice 18 em `CONTROLS`,
  "Branching", 0..1). 19/20 usados. Gate `architecture_painter_contract_surface` substring → ≤ 20.
- **`DiffusionParams`** `+capillary_branching` (default 0). **`GpuParams`** carrega-o num slot livre
  (offset 25, reusando um pad pós-`lift`); só `capillary.wgsl` o lê.
- **HR-5:** aritmética pura + `paper` determinístico ⇒ parity-exata; `branching=0` bit-idêntico.

## 4. Consequências
A franja ganha textura ramificada controlável (0 = liso de hoje, preservado). Próximo da fila: **#3
4K full-res residency** (ADR-0083). MoXi/LBM completo (dendritos finos) = follow-up de maior
fidelidade se o Enio quiser depois.

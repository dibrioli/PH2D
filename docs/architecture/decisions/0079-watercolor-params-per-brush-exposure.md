# ADR-0079 — Watercolor Params: per-brush exposure (full artist control)

**Status:** Accepted (2026-06-08) — ratificado pelo Enio ("Quero todos os controles expostos ao usuário para total controle do usuário, em subseção específica da aquarela").
**Decisor(es):** Enio (dono/decisor) + Claude.
**Estende:** [ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md) (o estágio S4 "multi-pigmento + emenda do `FluidParams`"), [ADR-0049](0049-fluid-brushes.md) (`FluidParams`/`FluidSim`), [ADR-0044](0044-brush-engine.md) (`Brush`/`RenderingParams`).
**Tags:** painter, fluid-sim, watercolor, brush-studio, ui, contract-surface, per-brush

---

## 1. Contexto

Pós-ADR-0078 (S0–S3d), o motor de aquarela tem **15 parâmetros de tuning** (8 da difusão-advecção base + 3 de deposição + 4 de shallow-water). Hoje todos são **globais**: os 8 base vêm de `FluidParams::default()` e os 7 da assinatura (deposição + fluxo) de **consts `WATERCOLOR_*`** no solver. Todo pincel de aquarela compartilha um único look e o artista não controla nada além do opt-in `fluid_enabled`.

O Enio quer **controle total do usuário** — todos os 15 controles expostos como sliders per-brush, numa subseção "Watercolor" do Brush Studio.

## 2. Decisão

### 2.1 Modelo de dados — `WatercolorParams` (novo DTO de brush, capado)
Um sub-struct **`WatercolorParams`** em `ph2d-painter-brush` (serializável, `#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]`), com os **15 campos** de tuning (espelho dos campos tunáveis de `DiffusionParams`). Armazenado em `RenderingParams.watercolor` (usa 1 dos 2 slots de headroom — cap `RenderingParams ≤ 14` intacto, agora 13). `#[serde(default)]` mantém brush-files antigos carregando.

**Por que um DTO dedicado e não serializar o `DiffusionParams` interno:** `DiffusionParams` é o tipo de tuning **interno do solver** — adicionamos campos a ele livremente durante o desenvolvimento (S3d acabou de somar 4). Serializá-lo direto acoplaria o **formato de brush-file** a cada mexida no solver. `WatercolorParams` é a **superfície de contrato** estável; `WatercolorParams::to_diffusion()` faz o mapeamento 1:1. `FluidParams` (ADR-0049) permanece intacto (8 campos, usado pelos testes de paridade + ref CPU); o caminho live passa a usar `WatercolorParams → DiffusionParams`.

### 2.2 Default = o preset validado (não dormant)
`WatercolorParams::default()` = o **preset de aquarela tunado** (deposição + shallow-water LIGADOS, com os valores dos consts `WATERCOLOR_*` atuais), NÃO `DiffusionParams::default()` (que é dormant/all-off, para preservar os gates do solver). Assim, ao ligar `fluid_enabled`, o pincel já nasce com o look validado pelo Enio, e os sliders partem daí.

### 2.3 Plumbing — per-brush, não consts
O bridge (`painter_fluid_bridge.rs`) deixa de usar `FluidParams::default()` + consts e passa a ler `painter` → `brush.rendering.watercolor.to_diffusion()`, via um único `FluidSolver::set_from_diffusion(queue, &DiffusionParams)` (que escreve os 15 no `GpuParams` cacheado — consolida os antigos `set_params`/`set_deposition`/`set_shallow_water`, que ficam para os testes). Os consts `WATERCOLOR_*` ficam como os **valores-default documentados** (referência do preset).

### 2.4 UI — subseção "Watercolor" no Brush Studio
Uma seção colapsável "Watercolor" em `ph2d-panel-brush-studio` com **15 sliders** (1 por controle), agrupados visualmente em **Diffusion** (8) / **Deposition** (3) / **Flow** (4). Cada slider é 0..1 normalizado mapeado para o range físico do parâmetro via uma **tabela de ranges** (a fonte única: `WatercolorControl` descriptor — id, label, min, max, get/set). Segue o padrão canônico do painel (ids → sections → populate → snapshot → `BrushParam` → `apply_ui_edit`). Sliders 1D são panel-only (sem dispatch em editor-core). Labels = strings literais em inglês (convenção atual do painel; i18n deferida, HR-15 exemption documentada).

## 3. Impacto em contratos congelados

- **`RenderingParams ≤ 14`** (ADR-0044): 12 → 13 campos (1 headroom usado). Gate `architecture_painter_contract_surface` continua passando (`assert_capped` é `≤`).
- **`WatercolorParams ≤ 16`** (NOVO): cap textual adicionado ao gate (15 campos usados, 1 headroom). Superfície serializada nova, capada por higiene.
- **`Brush ≤ 14`** (top-level): intacto (`RenderingParams` já é campo).
- **`FluidParams ≤ 12` / `FluidSim ≤ 12`** (ADR-0049): **intactos** (não tocados).
- **HR-14 (versioning):** `#[serde(default)]` no campo `watercolor` cobre brush-files pré-feature.

## 4. Consequências

O artista ganha controle total e per-brush sobre a física da aquarela (difusão, deposição, fluxo) — cada pincel pode ter um comportamento de aquarela distinto, salvo/replay via o brush-file existente. O preset default preserva o look validado. O acoplamento brush-file ↔ solver fica isolado pelo `WatercolorParams`/`to_diffusion()`. Caminho CPU-fallback (`wet_field`) honra os mesmos params quando ligado (mesma fonte). Próximo natural (ADR-0078 S4 restante): multi-pigmento K–M.

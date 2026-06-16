# Novo Painter — Brush Engine com paridade Procreate

> Pasta-mãe do **norte atual** do Painter após a remoção da simulação de aquarela/fluido
> ([ADR-0096](../architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md), Enio 2026-06-14).
> Decisão do Enio (2026-06-15): desenvolver de forma plena, **primeiro**, o **Brush Tool / Brush Engine**,
> usando **cada parâmetro do Procreate como a fonte definitiva** da nossa implementação.
> Estratégia: **paridade total com o Procreate primeiro; o diferencial vem depois, nos melhoramentos.**

## Por que Procreate, e por que "stamp-based" (não fluido)

O Procreate **não é simulação de fluido**. É um **brush engine baseado em dabs/stamps**: uma pincelada
é um caminho ao longo do qual uma imagem de **Shape** é carimbada repetidamente a um **espaçamento**,
modulada por uma textura de **Grain**, com mistura de cor opcional (**Wet Mix** = mixer-brush) e modos
de acumulação de alpha (**Rendering**: a família "glaze/blending"). É o modelo mais **pragmático,
previsível, reproduzível e rico em literatura** entre os apps de pintura — por isso é o nosso alvo.

A simulação de aquarela (shallow-water Curtis/MoXi) que tentamos era **submit/copy-bound** e nunca
estabilizou; foi removida e arquivada (`backups/wash_2026-06-14`). O pivot é mixer-brush + mistura de
pigmento Kubelka–Munk/Mixbox — exatamente o que o Procreate faz.

## Achado decisivo: nosso modelo de dados **já espelha o Brush Studio**

O `Brush` em [`crates/ph2d-painter-brush/src/brush.rs`](../../crates/ph2d-painter-brush/src/brush.rs)
(ADR-0044) **já tem as 14 categorias** do Brush Studio como sub-structs:
`stroke_path · stabilization · taper · shape · grain · rendering · wet_mix · color_dynamics ·
dynamics · pencil · properties · about`. E [`ph2d-color`](../../crates/ph2d-color/src/pigment_space.rs)
já tem **Mixbox/Kubelka–Munk** (ADR-0080/0091). O `RenderingMode` já tem **exatamente os 6 modos**
do Procreate (Light/Uniform/Intense/Heavy Glaze + Uniform/Intense Blending).

**Logo isto NÃO é greenfield.** É um plano de **completar a paridade** sobre um modelo já correto:
o gap real é (1) **avaliar cada parâmetro no dab pipeline** (muitos campos existem mas não afetam o
render), (2) **construir a UI do Brush Studio** (só 5 das 14 seções existem hoje), e (3) fechar uns
**poucos campos faltando** no modelo (ex.: `jitter_linear`, `taper tip/shape`, `shape angle`, per-target
do Apple Pencil, categoria `Preview`).

## Os documentos desta pasta (ordem de leitura)

1. **[`01_pesquisa_teorica_e_literatura.md`](01_pesquisa_teorica_e_literatura.md)** — a base teórica e a
   literatura: modelo de dab/stamp, build-up vs wash, mixer-brush (DAB/IMPaSTo), mistura de cor
   (Kubelka–Munk/Mixbox), grain, dinâmica de pressão/tilt, a matemática dos modos "glaze". **Reading
   list rankeada** + 2 alertas (licença do Mixbox CC BY-NC; ortogonalidade cobertura↔mistura).
2. **[`02_referencia_parametros_procreate.md`](02_referencia_parametros_procreate.md)** — a **referência
   definitiva**: os 14 painéis do Brush Studio e **cada controle**, com o nome exato, o que faz, tipo/range,
   a **chave do `Brush.archive` plist** (ground-truth) e **o campo correspondente no nosso `Brush`**. É o
   espelho contra o qual implementamos.
3. **[`03_plano_implementacao.md`](03_plano_implementacao.md)** — o **plano passo a passo**: cada feature
   do Procreate = uma etapa. Ondas por dependência, com status atual por parâmetro
   (modelo / engine / UI), itens fundacionais marcados Coord-only + ADR, e os contratos congelados a
   respeitar.

## Princípios (herdados do CLAUDE.md)

- **Padrão-ouro sem adiamento** (§0.6): a melhor opção técnica vence cronograma; gaps in-scope fecham na sessão.
- **Isolamento** (§0.2): cada implementador edita só a sua pasta; foundational/contrato = Coord-only + ADR.
- **Contratos congelados** (§6): `Brush≤168`/`Stamp=96B align(16)`/`RenderingMode=6`/`PainterParams≤12` —
  mexer na superfície = ADR + gate `architecture_painter_contract_surface`.
- **Paridade primeiro, diferencial depois.** Nada de "melhorar" antes de ter o que o Procreate tem.

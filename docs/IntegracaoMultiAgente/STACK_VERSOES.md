# Versões do stack — o que TODA linha nova usa (2026-10-30)

> Uma página. É o que um agente novo precisa saber antes da primeira linha de código.
> Detalhe técnico → [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) (HR-1..HR-18).
> Decisão da subida → [ADR-0168](../architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md).
> ⚠️ **Esta tabela é GATEADA** contra o `Cargo.lock` por
> [`architecture_stack_versions_doc_matches_the_lockfile`](../../crates/ph2d-editor-core/tests/it/architecture_stack_versions_doc_matches_the_lockfile.rs).
> Ela não pode envelhecer em silêncio: quem subir uma dependência **tem** de a editar, ou o portão fica vermelho.

## A tabela

| | versão | onde é a fonte |
|---|---|---|
| **Rust (toolchain)** | **1.99** | `rust-toolchain.toml` — e é **também o MSRV** (`rust-version` no `Cargo.toml`) |
| **edition** | **2024** | `Cargo.toml` `[workspace.package]` |
| **wgpu** / naga | **30.0.1** | sem tecto desde 30/10 (o `vello` 0.11 pede `^30`) |
| **vello** | **0.11.0** | |
| **parley** / fontique | **0.12.0** | o `fontique` segura o `read-fonts` |
| **skrifa** / read-fonts | **0.44.0** / 0.41.0 | tecto: `vello`, `usvg` e `fontique` pedem `^0.44` / `^0.41` |
| **kurbo** / peniko | **0.13.1** / 0.6.1 | |
| **bevy_ecs** | **0.20.1** | |
| **rapier2d** / parry2d | **0.36.1** / 0.31.1 | |
| **glam** | **0.30.10** *(+ 0.31 / 0.32 / 0.33)* | ⚠️ quatro cópias **de propósito** — ver abaixo |
| **glamx** | 0.3.1 | a matemática do `rapier` 0.35 |
| **taffy** / winit | 0.15.1 / 0.30.13 | |
| **accesskit** / rfd / mlua / cpal | 0.25.2 / 0.17.2 / 0.12.2 / 0.18.2 | |

> Rust 1.99 (o canal resolve 1.99.1; 29/10): rebuild da `ph2d-app-motion` 7,4 → 7,0 s (ABBA, mínimo).

## As três regras que um agente novo erra

1. ⛔ **Nunca escreva uma versão de memória, e nunca responda «dá para atualizar X?» sem correr
   `bash scripts/stack-audit.sh --tetos`.** *«O mais recente possível» ≠ «o mais recente»:* em 30/10 **15**
   crates são seguradas por outra. Forçar não dá erro de resolução — dá **duas cópias**, e um
   `Device`/`NodeId` de uma não serve à outra.
2. ⛔ **As quatro cópias de `glam` são o MECANISMO, não resíduo.** Unificá-las desligaria o SIMD de
   8 crates de desenho, porque a física corre `scalar-math` (HR-5, determinismo). É uma **recusa
   medida** — [registo §15](../Atualizar%20Stack/04_registro.md). Não «arrume» isto.
3. ⛔ **O que não subiu ficou por MEDIÇÃO** (30/10): `skrifa`/`read-fonts` (tecto do `vello`/`usvg`/
   `fontique`), `ndarray` 0.15 (`deep_filter`), `pollster` 0.4 (`rfd`), `sha2`/`hkdf`/`rand_core` dentro
   do `spake2`. Registo e recusas: [`docs/Atualizar Stack/04_registro.md`](../Atualizar%20Stack/04_registro.md) §26.

## O que a subida de 30/10 mudou no CÓDIGO (e morde quem não souber)

- **`wgpu` 30** — `get_mapped_range*` devolve `Result`: trate no idioma da função (`?`/`ok()?`; um
  anel que não pode entrar em pânico perde a amostra e DESMAPEIA). `Queue::present(tex)`, não
  `tex.present()`. `VertexState.buffers` é `&[Option<…>]`. Inteiros ENTRE estágios de shader exigem
  `@interpolate(flat)` — falha só ao criar o pipeline, não no `cargo check`.
- **`parley` 0.12** — coordenadas de variação: use `ph2d_text::run_coord_bits` (a porta única para o
  `i16` do `vello`); `Run::font_metrics`; `LineMetrics.hanging_advance` no lugar do `trailing_whitespace`.
  Espaçamento entre letras passa a ser por grafema e a linha mista segue o modelo CSS.
- **`rapier2d` 0.36** — `step`/`remove` pedem um `SoftBodySet` (o nosso fica vazio, e entra no
  checkpoint); o contacto rígido está atrás de `ctx.rigid_mut()`.
- **`ctt` 0.6** — o espaço de cor vive no VkFormat: cor → `*_SRGB`, dados → `*_UNORM`
  (`stored_color_space`).

## O que a subida de 29–30/08 mudou no CÓDIGO

- **`rapier2d` 0.35** — a matemática deixou de ser `nalgebra` e passou a `glam`/`glamx`. ⛔
  `Point` / `Isometry` / `Translation` **não existem**; o vocabulário vive em
  [`rmath.rs`](../../crates/ph2d-physics/src/rmath.rs).
- **`rapier2d` 0.35, o sono** — o critério deixou de comparar **velocidade** e passou a comparar
  **deriva de pose por passo** (inclui as correcções de posição do solver). ⭐ *Um número de
  afinação preservado através de uma reescrita de motor NÃO é conservador* — foi assim que uma
  caixa parou antes de assentar no chão.
- **`vello` 0.10** — atlas de imagem **persistente**. Quem recozinha pixels tem de chamar
  `mark_texture_dirty` / `mark_override_image_dirty`, senão a imagem congela; e quando não cabe,
  ela **não é desenhada, em silêncio**.
- **`vello` 0.10, `ImageQuality::High`** — era bilinear disfarçado na 0.8, hoje é bicúbico Mitchell
  de verdade. A paridade *pré-visualização ↔ sprite assada* mudou de significado.
- **`parley` 0.11** — o **sinal** do `y_offset` do glifo inverteu (foi a **cura** de um defeito
  nosso), e as larguras de avanço acima de `wght 400` encolheram até **−0,50 px**.

## Como se corre isto

```
cd /home/enio/Documentos/Projetos/PH2D
bash scripts/hw-profile.sh                     # o tier decide o MODO (L ou C) e a concorrência
bash scripts/cargo-check-narrow.sh <crate>     # inner loop — só isto
bash scripts/cargo-test-narrow.sh <crate> <nome>   # corrida dirigida de teste (sem nome: só com PH2D_SUITE="motivo: …")
bash scripts/stack-audit.sh --tetos            # ANTES de responder sobre versões
```

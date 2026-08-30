# Versões do stack — o que TODA linha nova usa (2026-08-30)

> Uma página. É o que um agente novo precisa saber antes da primeira linha de código.
> Detalhe técnico → [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) (HR-1..HR-18).
> Decisão da subida → [ADR-0168](../architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md).
> ⚠️ **Esta tabela é GATEADA** contra o `Cargo.lock` por
> [`architecture_stack_versions_doc_matches_the_lockfile`](../../crates/ph2d-editor-core/tests/architecture_stack_versions_doc_matches_the_lockfile.rs).
> Ela não pode envelhecer em silêncio: quem subir uma dependência **tem** de a editar, ou o portão fica vermelho.

## A tabela

| | versão | onde é a fonte |
|---|---|---|
| **Rust (toolchain)** | **1.98** | `rust-toolchain.toml` — e é **também o MSRV** (`rust-version` no `Cargo.toml`) |
| **edition** | **2024** | `Cargo.toml` `[workspace.package]` |
| **wgpu** / naga | **29.0.4** | teto: o `vello` pede `^29.0.3` |
| **vello** | **0.10.0** | |
| **parley** / fontique | **0.11.1** | segura o `skrifa` e o `accesskit` |
| **skrifa** / read-fonts | **0.44.0** / 0.41.0 | |
| **kurbo** / peniko | **0.13.1** / 0.6.1 | |
| **bevy_ecs** | **0.19.1** | |
| **rapier2d** / parry2d | **0.35.3** / 0.30.2 | |
| **glam** | **0.30.10** *(+ 0.31 / 0.32 / 0.33)* | ⚠️ quatro cópias **de propósito** — ver abaixo |
| **glamx** | 0.3.0 | a matemática do `rapier` 0.35 |
| **taffy** / winit | 0.14.0 / 0.30.13 | |
| **accesskit** / rfd / mlua / cpal | 0.24.1 / 0.17.2 / 0.12.1 / 0.18.2 | |

## As três regras que um agente novo erra

1. ⛔ **Nunca escreva uma versão de memória, e nunca responda «dá para atualizar X?» sem correr
   `bash scripts/stack-audit.sh --tetos`.** *«O mais recente possível» ≠ «o mais recente»:* hoje **8**
   crates são seguradas por outra. Forçar não dá erro de resolução — dá **duas cópias**, e um
   `Device`/`NodeId` de uma não serve à outra.
2. ⛔ **As quatro cópias de `glam` são o MECANISMO, não resíduo.** Unificá-las desligaria o SIMD de
   8 crates de desenho, porque a física corre `scalar-math` (HR-5, determinismo). É uma **recusa
   medida** — [registo §15](../Atualizar%20Stack/04_registro.md). Não «arrume» isto.
3. ⛔ **O que não subiu ficou por MEDIÇÃO.** O `wgpu` 30 é inalcançável enquanto o `vello` pedir
   `^29.0.3`. Plano vivo: [`docs/Atualizar Stack/`](../Atualizar%20Stack/).

## O que a subida de 29–30/08 mudou no CÓDIGO (e morde quem não souber)

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
bash scripts/cargo-test-narrow.sh <crate>      # corrida dirigida de teste
bash scripts/stack-audit.sh --tetos            # ANTES de responder sobre versões
```

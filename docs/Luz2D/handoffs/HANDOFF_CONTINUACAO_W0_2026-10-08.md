# Handoff de continuação — Luz 2D, W0 → W1 (2026-10-08)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · base `2a925859d` (main de 07/10).
- **Commits da linha:** `03c40b7c5` (pesquisa) · `b40502072` (aparelhos do dono) · `505041930` (plano + guia de stack).
- **Estado:** W0 FECHADA — só docs. Nenhum código, nenhum foundational de código tocado. Toque fora de
  `docs/Luz2D/`: `SKILL_Stack_PH2D_Definitiva.md` (3 trechos: `ph2d-light` é o rig do impasto, não RC;
  o «ADR-0009» nunca existiu). Gates `architecture_docs_*` do `ph2d-editor-core`: 6/6 verdes.

## Leia, nesta ordem
1. [`../01_plano.md`](../01_plano.md) — o desenho inteiro (portas, componentes, quadro, gates, ondas).
2. Só se precisar do porquê de uma técnica: o relatório em `../pesquisa/reports/`.

## Próximo passo — W1, o gabarito
- Criar `crates/ph2d-light2d-ecs` (componentes `Luz` / `BloqueiaLuz` / `NaoRecebeLuz` + porta
  `register_light2d_components` + `coleta::cena_de_luz`) e `crates/ph2d-light2d-ref` (path tracer 2D
  CPU, `f64`, determinístico).
- **Red-first:** G1 (fluência de disco no vazio = `2·L·asin(a/r)`), G2 (penumbra por semi-plano NÃO
  alinhado aos eixos), G10 (duas corridas bit-idênticas), G12 lado Rust (`queda`).
- Testes de integração em `tests/it/` (um binário por crate).

## Premissas medidas nesta janela (não re-derive)
- Componente novo **não** pede degrau de `PROJECT_SCHEMA` (blob chaveado por `blake3(nome)` —
  `crates/ph2d-ecs/src/scene/registry.rs`, comentário do `VecMorphMachine`). O schema está em 187.
- O vetor do documento é desenhado pelo Vello DEPOIS do tonemap, em `Rgba8Unorm`
  (`shells/desktop/src/render_loop/present_chrome.rs:54`); por isso o caminho de receber do vetor
  é no compositor (plano §2.4, gate G7).
- Menu Add: as famílias contribuem `ENTRIES` em `shells/desktop/src/render_loop/fase_object_add.rs:14-17`.
- Não há `#include` em WGSL; shaders por `include_str!`. Features do device em `crates/ph2d-gpu/src/context.rs:79`
  (tem `TIMESTAMP_QUERY`; não tem `SHADER_F16`).
- Tecto da shell: `TETO_LOC = 189_041` (`architecture_the_shell_only_shrinks.rs:112`).
- ⚠️ Aberto para a W4: como um objeto SEM sprite (Câmera) é escolhido no canvas —
  `shells/desktop/src/input_dispatch/despacho_clique_pick.rs`.

## Do dono
- Celular adiado («ainda é cedo»); aparelhos: iPhone 13, iPhone 16, POCO X5.
- Godot **não** é referência — é o piso a vencer.

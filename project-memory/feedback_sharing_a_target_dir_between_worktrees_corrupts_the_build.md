---
name: sharing-a-target-dir-between-worktrees-corrupts-the-build
description: Apontar CARGO_TARGET_DIR de uma worktree para o target de outra troca os .rlib entre as duas — e o erro que sai acusa o CÓDIGO, não o artefacto
metadata:
  type: feedback
---

Medir o `main` numa worktree temporária **reutilizando o `CARGO_TARGET_DIR` da linha** parece a
poupança óbvia (as dependências externas já estão compiladas). ⛔ Ela **corrompe as duas árvores**:
os `.rlib` das crates do repo colidem no mesmo `deps/`, e a árvore seguinte a compilar liga-se aos
artefactos da outra.

**Medido (PH2D, 2026-09-20).** Depois de medir o `main` com o target da linha, a linha deixou de
compilar com erros que acusavam o **meu** código:

```
error[E0599]: no method named `at_curvature` found for reference `&Surface`
error[E0599]: no variant named `Style` found for enum `Param`
```

⚠️ **Nenhum desses erros era verdade** — o código estava intacto (`git status` limpo). O que estava
errado era o `ph2d-material`/`ph2d-field` **do main** a servir de dependência à linha. E um
`cargo clean -p <crate>` das acusadas **não chegou**: a contaminação alcança a árvore de
dependências inteira, e a cura foi `cargo clean --release` (4,3 GB, rebuild completa).

⛔⛔ **E o preço não é só o relógio: é uma MEDIÇÃO FALSA.** Antes de perceber isto, eu medi um gate
de performance na linha contra o main e li **`6,5×` de regressão**; com o target limpo a mesma
medição deu **`1,4×`** na mesma cena. *Reportei ao dono um alarme que era meu.*

**Why:** o cargo distingue pacotes pelo manifesto, mas o nome do ficheiro de saída não carrega a
worktree — duas árvores do mesmo repo produzem `libph2d_material-<hash>.rlib` que colidem quando o
`hash` coincide, e o resultado é silencioso até alguém usar um símbolo que só existe de um lado.

**How to apply:**
- Uma worktree de medição leva **target PRÓPRIO**. Deixe o cargo recompilar; é mais barato do que a
  rebuild completa que a contaminação obriga.
- ⚠️ E **toda medição de relógio feita antes de perceber a contaminação fica em quarentena** — não
  a reporte, repita-a.
- Se precisar mesmo de partilhar, partilhe só o registo de dependências externas (`sccache`), nunca
  o `target/`.

Irmãs: [[an-operation-count-is-not-a-profile-and-the-build-profile-decides-the-number]] ·
[[a-touch-does-not-measure-an-edit-and-timings-inflate-under-contention]]

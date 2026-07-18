# Handoff de INTEGRAÇÃO — `line/anim-fixes` (DIRETRIZ §1.5.9)

> Duas correções pequenas e independentes, **smokadas e aprovadas pelo Enio**. A linha
> não integra nem faz ship — este documento vai ao **agente integrador**, por ordem
> explícita dele (CLAUDE §0.7).

## 1. Identidade

| | |
|---|---|
| Branch | `line/anim-fixes` (renomeada de `line/nesting`, que continha o conteúdo errado — o nome está **livre** para a linha de nesting) |
| Worktree | `Worktrees/line-anim/` |
| HEAD | `9a67beb2` |
| Base do fork | `389676f9` (main já integrado, pós-Painter/FLIP) |
| Commits | **2** |
| Contratos congelados | **NENHUM** |
| Deps novas | **NENHUMA** |
| `DOC_VERSION` / `PROJECT_SCHEMA` | **NÃO tocados** (segue 7 / 18) |

## 2. Arquivos — todos dentro do módulo

```
crates/ph2d-timeline/src/stack.rs              (hold_at)
crates/ph2d-timeline/src/stack_eval.rs         (1 chamada)
crates/ph2d-timeline/tests/loop_wrap.rs        NOVO
crates/ph2d-panel-timeline/src/populate.rs     (1 linha)
crates/ph2d-panel-timeline/tests/close_button_seam.rs  NOVO
```

**Zero foundational, zero shell, zero `render_loop`.** É o oposto da linha anterior — nada
aqui disputa arquivo com outra linha.

## 3. Símbolos que podem COLIDIR

Um só, e é **quebra de compilação, não silêncio**:

- **`ClipLane::hold_at` mudou de assinatura** — `hold_at(t)` → `hold_at(t, loop_range)`, e o
  retorno passou de `(&ClipStrip, f64)` para `(&ClipStrip, f64 /* t_clip */, f64 /* w */)`.
  Chamador único no repo (`stack_eval.rs:171`), já atualizado. Outra linha que tenha passado a
  chamá-la falha no `cargo check` — que é o resultado desejado.

Nenhum `NodeId` novo, nenhum token, nenhuma chave i18n, nenhum campo apendado.

## 4. Contratos congelados

**Nenhum.** Os 3 gates verdes no HEAD.

## 5. O que só o `ship.sh` pega

Pouco, desta vez: sem deps novas (machete/deny/audit inertes), sem arquivo perto do cap de LOC,
sem literal de UI novo. Rode mesmo assim `no_tofu_glyphs` e os caps **na árvore combinada** — os
gates de LOC moram na `ph2d-editor-core` e no shell e **não rodam** com `cargo test -p ph2d-timeline`.

## 6. Ordem, dependências e smoke

Os 2 commits são **independentes entre si** e podem ser cherry-picked em qualquer ordem.

| Commit | O quê |
|---|---|
| `48a47e98` | Sob um loop a faixa é CÍCLICA — o fade-in do topo cruza a partir do que o fim do loop deixa asserido, em vez de saltar para a pose de repouso. `hold_at` deixou de ser forward-only **dentro do intervalo do loop**; fora dele nada muda |
| `9a67beb2` | O X da timeline fecha a timeline — `TIMELINE_CLOSE` passou a ser registrado no `WidgetStore` (o handler já existia; faltava o evento) |

**Smoke: FEITO e aprovado pelo Enio (2026-07-16).** Nada pendente.

⚠️ **Meia-feature deliberada, nomeada** (`48a47e98`): o wrap foi construído só no lado da
**entrada**. Se a ÚLTIMA strip tiver um *fade-out*, ele ainda desbota rumo à pose de repouso em
vez da pose da primeira strip. O caso do relato não precisava, e não construí especulativamente —
mas está aqui para não virar "bug misterioso" daqui a três meses.

---
name: feedback-numbers-that-sum-across-lines-count-dont-pick
description: Contagem/cap/schema que várias linhas incrementam SOMA na integração — o número certo não existe em nenhum dos dois lados do conflito
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6316633f-521c-4b1d-a255-7662e2fda363
---

Na integração das 6 linhas (2026-07-12), três classes de número foram para o conflito, e em
**todas** o valor correto **não aparecia em lado nenhum**:

1. **Contagem do `ComponentRegistry`** (3 arquivos-espelho: `ph2d-ecs/src/scene/registry.rs`,
   `ph2d-render`, `ph2d-script`). Base 26. Painter registrou `PaintedDoc` → **27**. Vector
   registrou `VecConnector` → **27** (o mesmo número, por outro motivo!). Combinado: **28**.
   Depois o `VecLabel` levou a 29/30/30. Um "aceite um dos lados" deixa dois merges verdes e o
   workspace vermelho — e o `nextest-impacted` **não roda esses gates**.
2. **Cap de LOC (HR-18).** `input_handlers.rs`: Motion apendou as teclas do grafo, Áudio o
   Ctrl+X/C/V. Cada linha cabia sozinha (< 600); a **soma** deu 624. Só a árvore combinada vê.
3. **`PROJECT_SCHEMA`.** Quatro donos, um contador: Painter (v3/v4), Motion (v5), Flip (v6/v7).
   Postcard é **posicional** — sem o bump, um arquivo velho passa na checagem de versão e é lido
   com o layout novo: sai **geometria embaralhada**, não um erro.

**Why:** cada linha mede o delta **contra a base**, não contra as irmãs. O conflito apresenta
"27 vs 27" e parece empate — é soma disfarçada de escolha.

**How to apply:**
- **Conte, não escolha.** O número certo é derivável da árvore fundida: `grep -c '\.register::<'`
  na lista já mesclada, `wc -l` no arquivo, a lista de quebras de layout. **Prove com o teste**
  antes de seguir (`cargo test -p ph2d-ecs -p ph2d-render -p ph2d-script --lib`).
- Cap de LOC estourado = **split por responsabilidade**, nunca allowlist
  ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]) — e rode `fmt` ANTES de medir.
- No fechamento, `nextest --workspace` (NÃO o impacted): os gates de contagem moram em crates que
  o impacted-set não alcança.

## Emenda (2026-08-04): nem a aritmética do PRÓPRIO handoff se toma pronta

A `line/physics` fechou com um §2 dedicado ao número, e ele estava errado por um degrau:
*"a linha escreve 52, o `main` diz 53 ⇒ o valor certo é 54"*. Mas a escada dela tem **DOIS**
degraus (`PhysicsJoint.custom` do W-JointCustom · `corner_reach`+`lift_momentum` do W10), então
o par contado é **54 e 55**. O handoff fez a subtração certa sobre uma contagem errada dos
próprios commits.

**A regra fica mais estreita:** o integrador conta os degraus **do diff** (`git show :3:` do
`project.rs` contra o `:1:`, ou o doc-comment da escada), nunca o número que o handoff anuncia.
Um handoff que se dá ao trabalho de nomear o número é justamente o que induz a confiança —
e ele erra pelo mesmo motivo que qualquer linha erra: **ninguém dentro de uma worktree vê a
soma.**

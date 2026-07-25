# SESSION_ACTIVE — coordenação leve (DIRETRIZ §1.1)

**Propósito:** post-it compartilhado da posse VIVA da orquestração multi-agente —
quem está escrevendo o quê AGORA, para evitar colisão de git entre agentes
paralelos. **Modelo (DIRETRIZ §6.8):** 1 Coordenador (absorve PR/CI/ship) +
N Implementadores; os Implementadores **leem antes de cada burst** e não escrevem aqui.

> ⚠️ **O MODO é função do hardware, não uma escolha** (CLAUDE.md §0.5 / [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)):
> `constrained` = **Modo C** (shared tree + Coordenador, o parágrafo acima) · `workstation` =
> **Modo L** (uma worktree por linha, sem Coordenador; integração e ship por um **agente
> integrador dedicado**, só por ordem explícita do Enio). Rode `bash scripts/hw-profile.sh`
> antes de assumir qual dos dois vale nesta máquina.

**Não é log histórico nem fonte de estado.**
- Estado por-módulo (waves/tasks) → **CLAUDE.md §5**.
- Contratos congelados → **CLAUDE.md §6**.
- Histórico (o que já fechou) → **git log** + `docs/HANDOFF_*` / `docs/archive/`.
- Entradas concluídas saem daqui para o git log. **Limpe ao encerrar a sessão.**

---

## Integração — NADA pendente (2026-07-25)

**Nenhuma integração em curso.** A jornada de 2026-07-25 integrou **6 linhas**
(`Painter` · `Vector` · `motion-nodes` · `anim` · `physics` · `FLIP`), shipou e o CI fechou
**verde nos 3 OSes** (incl. o C9 de determinismo). `main` = `33c21c46c`.

> O bloco que vivia aqui avisava de uma "integração pendente" do **cutover Vector de 2026-07-06**
> — que fechou há três semanas, com a `line/audio` integrada em 14/07 e a `line/imageio` já sem
> worktree. Um aviso vencido neste arquivo é pior que arquivo vazio: ele é o **primeiro** que um
> agente novo lê, e mandava rebasear sobre um `main` que não existe mais.

### ⚠️ Ao REABRIR uma linha: rebase primeiro

**Toda worktree está ATRÁS do `main`** (de 2 a ~1150 commits, conforme há quanto tempo a linha
parou) — nenhuma continua de onde parou sem rebase. Rota "linha reaberta" do
[`MODELO_ABERTURA_LINHA`](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md):

```
cd Worktrees/<linha> && pwd && git branch --show-current   # ANTES de ler ou editar qualquer arquivo
git rebase main
```

O `cd` + `pwd` não é zelo: a janela abre na raiz (= `main`) e **o mesmo path relativo existe nas
duas árvores** — editar a errada compila e commita sem erro
([`MODELO_TROCA_DE_AGENTE_NA_LINHA`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).

**UMA linha carrega trabalho que NUNCA entrou no `main`** (confira com
`git -C Worktrees/line-motion-value rev-list --count main..HEAD` — o número é a fonte, não esta linha):

**`line/motion-value` — 17 commits (2026-07-14/15), ~1158 atrás do `main`.** Existe **só** nela:
as crates-nó `ph2d-node-fx-glow` (FX de passe: RT HDR, mip bloom COD/Jimenez, tint OKLCH),
`ph2d-node-motion-delay` e `ph2d-node-motion-path` · o foundational **`ph2d-nodegraph::external`**
(o canal por onde o que o APP possui entra no grafo) · o **W4.T4** (timeline docada no Motion) ·
e os docs de decisão 63/64/66/67. Tag: **`wip/motion-value-2026-07-15`**.

> ⚠️ **NÃO rebasear os 17 commits.** O contrato congelado não mudou desde 14/07
> (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`), então as 3 crates-nó entram como **drop-crates**
> — não existem no `main`, conflito zero. O que **não** é mecânico é o `external.rs`: ele vive
> dentro do `cook.rs`, que as linhas de GPU **reconstruíram** desde então (grid · reduces · luts ·
> o caminho GPU-resident). A rota é reabrir como linha usando os commits antigos de **referência,
> não de base** — e o doc 66 já declarou FALSA uma premissa de FX de passe uma vez, então replayar
> um desenho de julho sobre outro substrato é como se planta a segunda.

**`line/cook-parallel` foi DESCARTADA (2026-07-25)** — estava subsumida: o rayon no cook, o
`cook_determinism.rs` e o ADR de kernel-side-metadata já estão no `main`, levados adiante pela
`line/gpu-nodes` (21/07), que ficou **à frente** (8 casos de teste contra os 2 dela). Worktree e
branch removidas; o histórico vive na tag **`archive/cook-parallel-2026-07-15`**.

> Os `target/` de todas as worktrees foram limpos no fim-de-dia de 2026-07-25 (389 GB): o
> primeiro build de cada linha é **frio**, servido pelo `~/.cache/sccache` (~46 GB, quente —
> **nunca apague**, DIRETIVA_FIM_DE_DIA §3).

## Estado da orquestração

**Sem sessão multi-agente ativa.** Nenhum slot de implementador aberto; sem posse
reservada. Próximo trabalho parte de CLAUDE.md §5 (planos ativos) + §1 (roteador por tarefa).

> Ao abrir uma sessão: registre aqui um **MAPA DE POSSE** (agente → pasta(s) que vai
> escrever, zero overlap) antes do primeiro burst, e o limite de RAM (≤3 cargos
> simultâneos). Ao fechar: mova o que concluiu para o git log e volte este arquivo
> ao estado idle acima.

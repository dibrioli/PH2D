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

## Integração — jornada de 2026-07-27 FECHADA, nada pendente

**Três linhas integraram nesta jornada:** `line/anim` (as joias da coroa — retiming,
extrapolação, expressões, sinais) · `line/physics` (os joints ganharam mãos, e a cena
ganhou uma) · **`line/FLIP`** (multiplano 2.5D · Self Overlap · Airbrush · reamostragem
suave · dinâmica de pressão). `main` local **VERDE** (`ship.sh` com paridade EXATA do CI).
⚠️ **NÃO foi pushado** — o push é 1× por jornada e é ordem explícita do Enio (§0.7).

**Números no `main` de hoje** (a fonte é `project.rs`/`project_tests.rs`, não esta linha):
`PROJECT_SCHEMA` **37** · `FLIP_SCHEMA` **12** · `VEC_SCENE` **13** · `DOC_VERSION` **15** ·
ADR max **0144** · gizmo id max **968** (próximo livre 969) · registro `ph2d-ecs` **21** ·
`physics-ecs-c9` **`c9d4baee…`, 87 corpos**.

> ⚠️ **A colisão de `PROJECT_SCHEMA` aconteceu DUAS vezes na mesma semana, entre as MESMAS
> duas linhas.** Em 25/07 a `line/FLIP` e a `line/physics` escreveram ambas `30`; em 27/07
> escreveram ambas `32`/`33`/`34`. Nas duas o valor certo (**31**, e agora **37**) não estava
> em nenhum dos dois lados — ele se **CONTA** a partir do `main` do dia
> ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). Quem fechar uma linha que
> bumpa schema: escreva o número, mas escreva TAMBÉM que ele se conta.

> ⚠️ **E a `line/FLIP` fechou com dois gates VERMELHOS no próprio tip** (`paint_sections.rs`
> 621 > 600 · `populate` 216 > 200) — medidos idênticos antes e depois do rebase, então não
> era conflito de integração. O `architecture_panel_loc_cap` mora na `ph2d-editor-core`, e um
> fechamento por `cargo test -p ph2d-panel-flip` **não o alcança**. É a **terceira** vez nesta
> família (o `file_loc_caps` da shell na `line/physics`; os dois arch-gates de shell na
> `line/Vector`). **Quem fecha uma linha roda o gate de LOC do dono do teto, não só o `-p` da
> própria crate.**

### ⚠️ Ao REABRIR uma linha: rebase primeiro

**Toda worktree está ATRÁS do `main`** — nenhuma continua de onde parou sem rebase. Rota
"linha reaberta" do [`MODELO_ABERTURA_LINHA`](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md):

```
cd Worktrees/<linha> && pwd && git branch --show-current   # ANTES de ler ou editar qualquer arquivo
git rebase main
```

O `cd` + `pwd` não é zelo: a janela abre na raiz (= `main`) e **o mesmo path relativo existe nas
duas árvores** — editar a errada compila e commita sem erro
([`MODELO_TROCA_DE_AGENTE_NA_LINHA`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).

**UMA linha carrega trabalho que NUNCA entrou no `main`** (confira com
`git -C Worktrees/line-motion-value rev-list --count main..HEAD` — o número é a fonte, não esta linha):

**`line/motion-value` — 17 commits (2026-07-14/15).** Existe **só** nela: as crates-nó
`ph2d-node-fx-glow` (FX de passe: RT HDR, mip bloom COD/Jimenez, tint OKLCH),
`ph2d-node-motion-delay` e `ph2d-node-motion-path` · o foundational **`ph2d-nodegraph::external`**
(o canal por onde o que o APP possui entra no grafo) · o **W4.T4** (timeline docada no Motion) ·
e os docs de decisão 63/64/66/67. Tag: **`wip/motion-value-2026-07-15`**.

> ⚠️ **Quem reabrir começa por
> [`HANDOFF_REABERTURA_line_motion_value_2026-07-25.md`](HANDOFF_REABERTURA_line_motion_value_2026-07-25.md)**
> — ele traz a distância até o `main` **medida**, o contrato congelado **inalterado** desde
> 14/07 (as 3 crates-nó entram como drop-crates, conflito impossível), e a armadilha que
> nasceu em 25/07: **existem DOIS `motion_path_smoke.rs`** — o Motion Path da TIMELINE
> (ADR-0141, no `main`) e o nó `motion.path` (aqui). Mesmo nome, features sem relação; a
> resolução é renomear, nunca fundir.

**`line/cook-parallel` foi DESCARTADA (2026-07-25)** — estava subsumida pela `line/gpu-nodes`.
Worktree e branch removidas; o histórico vive na tag **`archive/cook-parallel-2026-07-15`**.

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

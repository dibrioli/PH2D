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

## Integração — jornada de 2026-07-30 FECHADA (3 linhas), **NÃO pushada**

**Três linhas integraram**, na ordem MEDIDA (a sobreposição de arquivos era ~nula; quem
decidiu foi o número de ADR): **`line/Painter`** (74 commits — o journal de undo por tile +
a performance do Wet Paint) → **`line/Vector`** (58 — a pilha de FX raster + o lápis, a
largura viva e o alcance do nó) → **`line/motion-value`** (43 — o editor de nós ganhou mãos:
adapter, splice, snap, gradiente, bypass). `main` = **`5ef05a0ae`**.

**Verde na árvore COMBINADA:** impactados **8506/8506** (`ci-test`) · **workspace inteiro em
DEBUG** · `clippy --workspace --all-targets` **0** · `fmt --check --all` limpo · `machete`
limpo · os arch-gates por NOME (LOC de workspace/painel/shell, `node_id_collisions`,
`panel_wiring_parity`, `hr12_widgets_a11y`, `adr_numbers_are_unique`, `arch_safe_clamp_only`,
HR-15 magic/color). ⚠️ **NÃO foi pushado, e o `ship.sh` NÃO foi rodado** — os dois são ordem
explícita do Enio (§0.7).

**Números no `main` de hoje** (a fonte é o código, não esta linha):
`PROJECT_SCHEMA` **38** · `FLIP_SCHEMA` **12** · `VEC_SCENE` **13** · `DOC_VERSION` **15** ·
formato textual do nodegraph **v5** · ADR max **0148** · registro `ph2d-ecs` **39**
(espelhos `ph2d-render`/`ph2d-script` **40**) · crate nova **`ph2d-stroke-width`**.

> ⚠️ **A disputa desta jornada foi de ADR, não de schema:** `line/Painter` reivindicou
> **0145/0146/0147** e `line/Vector` reivindicou **0145** — mais a `line/sculpt3d`, que ainda
> não integrou e também escreveu 0145. A Painter chegou primeiro e ficou com os três; o da
> Vector virou **0148** (5ª renumeração no repo). ⚠️ **E o rewrite tem armadilha medida:** o
> `Cargo.lock` contém `0145` **dentro de um checksum**, então o token é `ADR-0145` + o stem do
> arquivo, escopado aos arquivos que a LINHA mudou — nunca o número nu sobre a árvore.

> ⚠️ **SEIS latentes foram drenados na integração, e NENHUM era conflito de merge** — todos
> eram vermelhos que já estavam no tip de uma linha e que só a árvore combinada mostra:
> **(1)** um conflito SEMÂNTICO de mesmo-símbolo (a Vector trocou `take_curve_point_drag()`
> por `..._if(pred)` e a motion-value acrescentou um chamador com a API velha — cada lado
> compilava sozinho; e o `drain_drag` dela já perguntava de quem era o gesto, mas **depois de
> drenar**, que é o roubo que aquele fix descreve) · **(2)** dois gates de **HR-15** (o sufixo
> `_tests.rs` não é cosmético: os gates isentam por ele; e o marcador `LITERAL-COLOR-OK` casa
> **na linha da chamada**, então numa chamada multi-linha ele estava em lugar nenhum) ·
> **(3)** três demos que semeavam um nó por **param morto**, fazendo o `validate` recusar o
> grafo inteiro · **(4)** dívida de `fmt` (a linha conferiu só `-p ph2d-host-desktop`) ·
> **(5)** dois arquivos por cima do teto de LOC — ⚠️ **causados PELO fmt**, que re-expande
> (os dois gates passavam antes dele) · **(6)** cinco avisos de clippy, com o CI em
> `-D warnings`. **A regra que atravessa os seis: um fechamento por `cargo test -p <crate>`
> não alcança o gate cujo dono é outra crate** — é a 4ª jornada seguida com esta família.

> ⚠️ **DUAS gates de RELÓGIO flakaram, e uma foi CORRIGIDA:** o
> `the_cost_of_depth_is_linear_not_explosive` (timeline) é a flake pré-existente que a §5 já
> nomeia — passa isolado, re-rode antes de suspeitar do merge. Já o
> `the_worker_reports_what_a_step_costs` (a água) falhou **duas vezes no mesmo dia** (sob a
> suíte inteira em paralelo, e em DEBUG, ~16× mais lento) porque **esperava uma DURAÇÃO
> fixa**; agora espera uma **CONDIÇÃO**, e a espera **dirige o produto** (o balde `away` só
> existe quando o motor VIAJA — um laço que só dorme nunca o preenche). Mutação conferida.

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

**QUATRO linhas seguem EM VOO** — elas não entraram nesta rodada e **não** estão fechadas
(confira o número, que é a fonte: `for b in anim physics FLIP sculpt3d; do git rev-list
--count main..line/$b; done`, medido em 2026-07-30 como **79 · 69 · 55 · 4**):
`line/anim` · `line/physics` · `line/FLIP` · `line/sculpt3d` (esta última só docs, o pivô
para Rust/wgpu com o SculptGL como referência).

> ⚠️ **A `line/sculpt3d` escreveu `ADR-0145`** e esse número **já foi para a `line/Painter`**.
> Quando ela fechar, o dela se **CONTA** a partir do `main` do dia (hoje o máximo é **0148**).

> ⚠️ **A `line/motion-value` INTEGROU** (2026-07-30) — a nota anterior, que dizia que ela
> carregava 17 commits que nunca entraram no `main`, está **vencida**. O trabalho de
> 14/07-15/07 (as crates-nó `fx-glow`/`motion-delay`/`motion-path`, o `ph2d-nodegraph::external`,
> o W4.T4) e a jornada de 30/07 estão no `main`. ⚠️ **A armadilha dos DOIS `motion_path_smoke.rs`
> foi resolvida na própria linha** — não a re-litigue.

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

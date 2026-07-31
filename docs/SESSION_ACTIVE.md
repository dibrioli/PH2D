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

## Integração — jornada de 2026-07-30 FECHADA (6 linhas), **NÃO pushada**

**Seis linhas integraram**, na ordem MEDIDA: **`line/Painter`** (74 commits — o journal de
undo por tile + a performance do Wet Paint) → **`line/Vector`** (58 — a pilha de FX raster +
o lápis, a largura viva e o alcance do nó) → **`line/motion-value`** (43 — o editor de nós
ganhou mãos) → **`line/physics`** (70 — a POLIA W0..W6 + a Weston + o pino de mundo) →
**`line/sculpt3d`** (7 — a W1 do módulo 3D, a malha) → **`line/FLIP`** (58 — o motor novo de
traço, que passa a ser PERCORRIDO em vez de rasterizado). `main` = **`d1204c500`**.

> **Como a ordem foi medida nas duas últimas** (a sobreposição se MEDE, não se escolhe): as
> duas tocavam os MESMOS 3 arquivos (`Cargo.lock` · `main.rs` · `render_loop/mod.rs`), as duas
> **só acrescentando** um `mod` e uma chamada de smoke — sobreposição **simétrica e trivial**.
> O desempate foi RISCO: sculpt3d são 7 commits de arquivos quase todos NOVOS, a FLIP são 58
> replayados sobre 186 commits de `main`. A barata primeiro dá um `main` verde intermediário
> que serve de CONTROLE para a cara.

**Verde na árvore COMBINADA** (rodado por linha, depois do ff-merge): impactados
**6674 · 1704 · 3692** (`ci-test`) · **workspace inteiro em DEBUG** · `clippy --workspace
--all-targets` **0** · `fmt --check --all` limpo · `machete` limpo · os arch-gates por NOME
(LOC de workspace/painel/shell, `node_id_collisions`, `panel_wiring_parity`,
`adr_numbers_are_unique`, `arch_safe_clamp_only`, HR-15 magic/color) · e os **gates de GPU
`#[ignore]` na RTX: FLIP 114/114 · 3D 4/4** (sem adapter fazem *skip gracioso*, **que não é
verde**). ⚠️ **NÃO foi pushado, e o `ship.sh` NÃO foi rodado** — os dois são ordem explícita
do Enio (§0.7).

**Números no `main` de hoje** (a fonte é o código, não esta linha):
`PROJECT_SCHEMA` **46** · `FLIP_SCHEMA` **12** · `VEC_SCENE` **13** · `DOC_VERSION` **15** ·
formato textual do nodegraph **v5** · **ADR max 0150** · registro `ph2d-ecs` **39**
(espelhos `ph2d-render`/`ph2d-script` **40**) · registro `ph2d-physics-ecs` **24** · gizmo ids
**próximo livre 972** · `physics_ecs_c9` **`7cb7728d…`, 96 corpos** · crates novas
**`ph2d-stroke-width`** + **`ph2d-mesh`/`ph2d-mesh-render`/`ph2d-sculpt3d`**.

> ⚠️ **A disputa desta jornada foi de ADR, e ela CONTINUOU depois de eu declará-la fechada:**
> `line/Painter` levou **0145/0146/0147**, `line/Vector` renumerou para **0148** (5ª vez),
> `line/physics` para **0149** (6ª) e `line/sculpt3d` para **0150** (7ª) — as quatro tinham
> escrito **0145**. Como os NOMES de arquivo diferem, o git **nunca conflita**: quem chega ao
> `main` primeiro fica com o número, e o gate `architecture_adr_numbers_are_unique` é quem
> acusa. ⚠️ **E o rewrite tem armadilha MEDIDA:** o `Cargo.lock` contém `0145` **dentro de um
> checksum**, e no dia da sculpt3d havia **10+ arquivos ALHEIOS** citando `ADR-0145` (o do
> wet-paint) — o token é `ADR-0145` + o stem do arquivo, **escopado aos arquivos que a LINHA
> mudou**, nunca o número nu sobre a árvore.

> ⚠️ **O `PROJECT_SCHEMA` também subiu depois:** a `line/physics` escreveu 45 e o valor certo
> era **46** — a `line/Vector` tinha acabado de levar o 38 e a escada SOMA. O handoff dela
> dizia *"no `main` de hoje é 38"* e **envelheceu no MESMO dia**. *Um número que se CONTA
> envelhece entre escrever o handoff e integrá-lo.*

> ⚠️ **A falha EMERGENTE desta rodada foi de `fmt`, e nenhuma linha a tinha:** a sculpt3d
> fechou com `cargo fmt` verde na própria árvore, e o rebase pôs o `mod sculpt3d;` no slot
> alfabético que era certo **no fork** e errado no `main` de hoje (que ganhou dezenas de
> `mod flip_*`), além de mudar a quebra de linha do `present.rs` pela indentação ao redor.
> Corrigido com `rustfmt` nos 2 arquivos — **nunca `cargo fmt -p`**, que reformata WIP alheio.

> ⚠️ **E um `✗` do gate era do AMBIENTE, não do código:** o `target/` da árvore PRIMÁRIA é um
> symlink para `/dev/shm/ph2d-target` e a tmpfs evaporou, então todo `cargo` rodado da raiz
> morria em `failed to create directory`. Os gates das linhas rodam das worktrees (target
> próprio) e por isso passaram. Com `main == line/FLIP` byte a byte, o gate de fechamento foi
> rodado da worktree quente. **Recrie o diretório antes do `ship.sh`.**

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

**UMA linha segue EM VOO** — ela não entrou nesta rodada e **não** está fechada (confira o
número, que é a fonte: `git rev-list --count main..line/anim`, medido em 2026-07-30 como
**82**): **`line/anim`**. Todas as outras worktrees estão em zero commits contra o `main`.

> ⚠️ **As notas anteriores sobre `line/physics`, `line/FLIP`, `line/sculpt3d` e
> `line/motion-value` "em voo" estão VENCIDAS** — as quatro integraram. ⚠️ E a que dizia que
> a `motion-value` carregava 17 commits órfãos também: o trabalho de 14/07-15/07 (as crates-nó
> `fx-glow`/`motion-delay`/`motion-path`, o `ph2d-nodegraph::external`, o W4.T4) está no `main`.
> **A armadilha dos DOIS `motion_path_smoke.rs` foi resolvida na própria linha** — não a
> re-litigue.

> ⚠️ **TODOS os smokes desta jornada seguem PENDENTES** (as três primeiras linhas já estavam
> assim; physics/sculpt3d/FLIP foram smokadas pelos donos ANTES da integração, e **integrar
> não é aprovar**). Os comandos por módulo estão na §5 do CLAUDE.md.

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

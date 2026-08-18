# CLAUDE.md — núcleo operacional do PH2D (LEIA INTEIRO — é curto de propósito)

> Toda LLM recebe este arquivo automaticamente. Ele é o **roteador**: os inegociáveis +
> para onde ir por tarefa. Detalhe técnico → [`SKILL_Stack_PH2D_Definitiva.md`](SKILL_Stack_PH2D_Definitiva.md)
> (HR-1..HR-18, stack). Processo → [`DIRETRIZ.md`](docs/IntegracaoMultiAgente/DIRETRIZ.md).
> Não leia esses dois inteiros — use o roteador §1.

## §0 — Inegociáveis (memorize os 8)

0. **O alvo é o EXTRAORDINÁRIO, e o teto é o do HARDWARE — nunca o do caminho lento.** Esta engine não busca "bom o suficiente": ela busca o que a máquina de fato faz. Antes de escrever qualquer limite (cap, teto, ceiling, `MAX_*`, faixa de slider, "por ora", "razoável"), **MEÇA** — e depois escreva o número que a MEDIÇÃO deu, com a tabela ao lado dele.
   - ⚠️ **Nunca deixe o fallback definir o produto.** Caso real (GPU/M5, 2026-07-17): a sim de partículas rodava **4,19 M partículas em 3,6 ms na GPU** (22% de um frame de 60 fps) e o teto foi posto em **16.384** — 256× abaixo — porque a *CPU* seria lenta a 262k. O caminho mais lento definiu o teto do mais rápido, no módulo cuja razão de existir é o mais rápido. O caminho de referência (CPU) só precisa **computar a mesma resposta**; quem manda no teto é o dispositivo.
   - ⚠️ **"Fora de escopo porque é inalcançável" é uma afirmação sobre um número que outra pessoa pode mudar.** O mesmo caso: *"id é f32, teto 2²⁴ ≈ 4,8 dias a rate 40 — fora de escopo"* era **verdade** enquanto o slider parava em 200 (23 horas), e virou **4 segundos** quando o slider subiu. Quem move o número que tornava algo inalcançável **tem de reconferir a nota**.
   - Um limite legítimo diz **de que recurso ele é** (memória, largura de banda, precisão de representação) e traz a medição. Um limite que só diz "por segurança" é um palpite esperando um smoke.
   - Isto **não** é licença pra otimização prematura ([memória](project-memory/project_m5_perf_validated.md)): é a exigência de **medir antes de limitar**, e de não confundir conforto de implementação com limite físico.


1. **Norte arquitetural ([ADR-0075](docs/architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)):** monorepo Rust único; desacoplar por **ECS** (components + events/resources, systems não se chamam), **NÃO** por plugin em runtime nem WASM. Feature nova = **drop-crate** (A). Plugin runtime foi pesquisado e **rejeitado** (sem ABI estável; nem resolve o coupling de schema).
2. **Isolamento:** edite a pasta do seu módulo. **Modo C** (shared tree): precisou de algo fora (foundational/shell/contrato/outra crate)? **PARE e reporte ao Coordenador** — nunca renegocie direto com outro agente. **Modo L** (worktree, [ADR-0107](docs/architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)): **foundational você PODE e DEVE tocar** (com cuidado) sob o protocolo testado — a integração roda `scripts/foundational-integrate.sh` (gate da árvore combinada) + Mergiraf funde o resíduo textual. **Ao CRIAR foundational novo, projete-o para isolamento** (módulo irmão / ponto de extensão append-only — a foundation é isolada de propósito, pra várias linhas estenderem sem colidir). **PARE e reporte ao Enio** só em 2 casos: **contrato congelado** (§6, exige ADR) ou **rebase conflitando fora dos seus arquivos** (colisão de mesmo-símbolo, DIRETRIZ §1.5.5). Você **fecha a linha, escreve o handoff de integração (DIRETRIZ §1.5.9) e PARA** — NÃO integra nem faz ship sozinho (§0.7).
3. **UI canônica:** zero hex, zero `f32` literal de UI, zero string hardcoded — tudo via tokens / i18n (HR-15).
4. **Git anti-colisão (Modo C — shared tree):** `git add -- <seus paths>` (NUNCA `-A`/`-a`/`git add .`/`git stash`); `git commit --no-verify -m "msg" -- <paths>`; `git status` antes de stage; se houver `M`/`??` alheio, não comite — reporte. **Modo L:** cada linha tem worktree+índice próprios — colisão de commit não existe; valem só os conflitos de merge + proibições da DIRETRIZ §1.5.5–1.5.6.
5. **Velocidade (§2):** inner loop = **SÓ `cargo check -p`**; teste/clippy/auditoria **1× no fechamento do módulo**, nunca por task. Concorrência **é função do hardware** — `bash scripts/hw-profile.sh` (≤3 agentes só no tier `constrained`/Mac 8 GiB; `workstation` voa). O tier também define o **MODO de operação** ([ADR-0106](docs/architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)): `workstation` = **Modo L** (linhas paralelas por worktree, DIRETRIZ §1.5) · `constrained` = **Modo C** (shared tree + Coordenador). Detalhe: §2 + DIRETRIZ §6.0.
6. **Padrão-ouro sem custo:** a melhor opção técnica vence custo de build/cronograma ([feedback-perfection-no-deferrals](project-memory/feedback_perfection_no_deferrals.md)); gaps in-scope fecham na sessão atual.
7. **Push é 1× por jornada — e NUNCA é seu por conta própria.** Modo C: você NÃO pusha — reporta commit local; o **Coordenador** faz ship + push + babysit CI (§3). **Modo L: integração E ship só por ordem EXPLÍCITA do Enio** (nunca autônomos), via um **agente integrador dedicado** munido do handoff de cada linha (DIRETRIZ §1.5.3–1.5.4). A linha NÃO integra nem pusha sozinha — fecha, entrega o handoff (§1.5.9) e espera. Integrar/pushar sem ordem = **violação do protocolo**.

## §1 — Roteador leia-por-tarefa (leia SÓ o que sua tarefa exige)

> **A CADA passo de QUALQUER implementação, leia primeiro [DIRETIVA_IMPLEMENTACAO.md](docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).**
> É o antídoto das 4 causas da semana perdida no Painter (costura não-testada · "audit"=compilar ·
> isolamento órfão · alvo irrefutável). Regra-mãe: **verde-de-compilação é velocidade; no audit vale ZERO.**

| Sua tarefa | Leia ISTO (e só isto) |
|---|---|
| **Tool ou node nova** | DIRETRIZ §2 (triagem) + §3.A + [examples-fan-out.md](docs/IntegracaoMultiAgente/examples-fan-out.md) |
| **Painel / widget / chrome** | DIRETRIZ §3.B |
| **Modificar feature existente** | DIRETRIZ §3.D |
| **Foundational (Modo L)** | [ADR-0107](docs/architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md) — editável pela sua linha; integra por `scripts/foundational-integrate.sh` (DIRETRIZ §1.5.3). Só contrato congelado (§6) e mesmo-símbolo escapam |
| **Foundational (Modo C) / contrato congelado** | DIRETRIZ §3.C + §4 (**Coord-only + ADR**) |
| **Trabalhar em linha paralela (Modo L / workstation)** | DIRETRIZ §1.5 (worktrees, integração `--ff-only` + gate testado, briefing §1.5.8) |
| **Rodar uma jornada Modo L (você, operador)** | [GUIA_JORNADA_MODO_L.md](docs/IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) — abrir linhas, quando intervir, quem faz o ship (sem coordenador) |
| **Você ASSUMIU uma linha que já existe** (troca de janela / retomada pós-integração) | [MODELO_TROCA_DE_AGENTE_NA_LINHA.md](docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md) — **`cd` + `pwd` + `git branch --show-current` ANTES de ler qualquer arquivo.** A janela abre na raiz (=`main`) e o mesmo path relativo existe nas 2 árvores: editar a errada compila e commita **sem erro** |
| **Build lento / quero voar** | DIRETRIZ §6 (stack de velocidade) — §2 abaixo é o resumo |
| **Dúvida de stack / Hard Rule** | SKILL_Stack §HR-1..18 (cite por ID) |
| **Física / corpo rígido / colisão** | [ADR-0131](docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) (o *porquê*) + tracker [`docs/Physics/handoffs/HANDOFF_line_physics.md`](docs/Physics/handoffs/HANDOFF_line_physics.md) (estado) + [`00_plano_waves.md`](docs/Physics/00_plano_waves.md) (waves) + [`BUGS_physics.md`](docs/Physics/BUGS_physics.md) (bugs cuja causa enganava) |
| **Fim de dia · o disco encheu · "por que o target é tão grande?"** | [DIRETIVA_FIM_DE_DIA.md](docs/IntegracaoMultiAgente/DIRETIVA_FIM_DE_DIA.md) — os 3 portões antes de apagar, e a **§2-bis** com a decomposição MEDIDA do target (54% é `incremental/`) e as 3 regras que atacam o pico |
| **Quem é o Enio / estado do projeto** | [project-memory/MEMORY.md](project-memory/MEMORY.md) |
| **Quem possui o quê agora** | [SESSION_ACTIVE.md](docs/SESSION_ACTIVE.md) |

## §2 — Velocidade ("agents flying"), resumo (detalhe + configs: DIRETRIZ §6)

- **PRIMEIRO: `bash scripts/hw-profile.sh`** — a estratégia é função do hardware, não fixa. Os bullets abaixo são o baseline `constrained` (Mac 8 GiB); tier `workstation` (desktop 128 GB) sobrescreve (RA full, muitos cargos, slots opcionais, tmpfs, sccache). Tabela: DIRETRIZ §6.0. Racional: [ADR-0104](docs/architecture/decisions/0104-hardware-tiered-speed-strategy.md).
- **UM TURNO, N CHAMADAS.** Chamadas independentes — ler 3 arquivos · 2 greps · medir 2 números · rodar 2 sondas — vão na **MESMA** mensagem. ⚠️ **Medido 2026-08-18: 279.566 turnos com 1,00 chamada cada** (oito em 279 mil usaram duas), sobre uma mediana de **991 turnos por sessão**, cada um um round trip completo. É a **maior alavanca de relógio deste repo** e não muda processo nenhum. Só serialize o que **depende** do resultado anterior.
- **EDITE pela ferramenta `Edit`, não por `python3`/`sed`.** ⚠️ Medido: **52% das edições iam por script**, ~93 k tokens de geração a mais por sessão — e o modo de falha é o caro: um `str.replace()` que não casa é **no-op SILENCIOSO e o script imprime sucesso**, enquanto o `Edit` **falha alto** quando `old_string` não casa. Script só onde ele é a forma CERTA (mutação: backup → mutar → testar → restaurar · renomeação em N arquivos · edição derivada de medição), e aí **sempre com `assert` de contagem** ([`project-memory`](project-memory/feedback_python_replace_silent_noop_after_fmt.md)).
- **Inner loop = `cargo check -p <crate>`** (ou `scripts/cargo-check-narrow.sh <crate>` p/ cortar tokens de erro). Nada de test/clippy/auditor por task. ⚠️ **Gate red-first e prova de mutação NÃO são o inner loop** — eles rodam `test -p` de propósito, uma vez cada, e é assim que tem de ser; o que não pode é `test -p` responder ***"minha edição entrou?"***, que é pergunta de `check`. Medido: `test -p` custa **2× a 20×** o `check -p` (0,4 / 17,7 / 38,8 s contra 0,2 / 6,2 / 2,0 em três crates), e a razão real era **4,3:1 na direção errada**.
- **A sonda destas três leis é `bash scripts/agent-loop-profile.sh`** — ela lê os transcripts e imprime paralelismo · turnos/sessão · `test:check` · % de edições pela ferramenta, cada um com o baseline de 2026-08-18 ao lado. *Uma regra sem instrumento é uma nota que envelhece.*
- **Slot warm por CoW** (só `constrained`): `bash scripts/slot-seed.sh <slot>` → prefixe cada cargo com o `CARGO_TARGET_DIR` impresso. No `workstation` os slots são opcionais (`target/` único basta).
- **Diagnóstico via LSP (maior alavanca):** `constrained` = `cargo-check-narrow.sh` on-demand (RA é RAM-blocked); `workstation` = **rust-analyzer full como oráculo**, não leia saída crua do cargo.
- **Gate batched no fim do módulo:** `scripts/nextest-impacted.sh` + clippy `--all-targets` + auditoria ≥2 lentes, **1× sobre o diff acumulado**. ⚠️ **Prefixe o nextest com `CARGO_INCREMENTAL=0`** — o perfil `ci-test` só roda em BATCH (uma ou duas vezes por jornada, sobre a workspace inteira), então compilação incremental não colhe nada ali e paga **11 GB** (medido 2026-08-16). O `cargo check -p` do inner loop fica em paz, de propósito. E ao FECHAR a linha, reclame o resto: `rm -rf target/*/incremental` (DIRETRIZ §1.5.9 item 7).
- **Cargos simultâneos:** `constrained` ≤3 (RAM 8 GiB); `workstation` ~cores/6 (build) / ~cores/3 (check) — vide hw-profile.
- **NÃO use:** Cranelift (ruim p/ check-loop + gaps macOS). Linker = `mold` no Linux (**nunca no `.cargo/config.toml` do repo** — global), `lld/ld-prime` no macOS (mold é ELF-only).

## §3 — CI / ship (Modo C: Coordenador absorve PRCI · Modo L: quem fecha a última integração da jornada — DIRETRIZ §1.5.4)

**Implementador:** não faz `git push`, não monitora CI — reporta commit local pronto.
**Coordenador:** push **1× por jornada** (run de CI ~30min: matrix linux+macOS+windows + replay-hash + bench). Protocolo [DIRETRIZ §8](docs/IntegracaoMultiAgente/DIRETRIZ.md):
1. `./scripts/ship.sh` (paridade EXATA com lint+test do CI — fmt, clippy `--all-targets`+features, machete, deny, audit, nextest `--cargo-profile ci-test`, typos). Corrija TODO `✗`, **não pusha antes de verde**.
2. `git push origin main` → babysit (polling 15min, `gh run watch`) até `success`; em vermelho, fix + re-push (escalona após 3 falhas do mesmo job).
3. Forneça SEMPRE o link: `https://github.com/dibrioli/PH2D/actions/runs/<id>` (`gh run list --workflow=spike.yml --limit=1`).

**Fast mode (dia):** `git commit --no-verify` (instantâneo), `cargo check -p` quando quiser, **zero push/CI**. Ship só no fim quando o Enio mandar ("commit"/"push"/"ship"/"fim do dia").

## §4 — Memória persistente

A memória agora é **versionada no repo** em [`project-memory/`](project-memory/) (índice: [`project-memory/MEMORY.md`](project-memory/MEMORY.md)) — feedback acumulado, perfil do Enio, estado, paths canônicos. **LLM nova lê o índice antes de agir.** O Claude Code lê/escreve a memória via **symlink** de `~/.claude/projects/<key>/memory` → `project-memory/` (bootstrap por-máquina em [`docs/DevOps/MULTI_MACHINE_SETUP.md`](docs/DevOps/MULTI_MACHINE_SETUP.md) §4). **Multi-máquina (Mac testes · Linux dev · Windows build):** GitHub é a fonte única, clone local por máquina — runbook completo em [`docs/DevOps/MULTI_MACHINE_SETUP.md`](docs/DevOps/MULTI_MACHINE_SETUP.md).

## §5 — Estado dos módulos (PONTEIRO, não história)

> **Esta seção é um roteador.** Por módulo: o que ele **é**, o que está **ABERTO**, como **smokar**, onde **ler**.
> O *mecanismo* de cada wave vive no **handoff** dela; o índice cronológico é o `handoffs/README.md` do módulo;
> a *história* até 2026-08-18 está **verbatim** em [`docs/archive/estado-2026-08-18/`](docs/archive/estado-2026-08-18/).
>
> ⚠️ **Fechar uma linha escreve no HANDOFF e edita UMA linha aqui.** Acrescentar a narrativa da jornada a esta
> seção é o que levou o `CLAUDE.md` de **1,7 KB** (2026-05-08) a **917 KB** (2026-08-18) em 326 commits — e este
> arquivo é injetado **por inteiro** em todo agente, todo subagente e toda worktree, antes da primeira palavra do
> Enio. Medido em 2026-08-18: **466 k tokens de contexto inicial, ~47% da janela de 1 M**, e a compactação **não
> alcança isto** (ele é re-injetado inteiro em toda janela nova). **Cada KB aqui é pago por todos, sempre.**

### §5.0 — Leis que atravessam os módulos (ficam aqui de propósito)

- **Número que soma entre linhas se CONTA, nunca se escolhe** — `PROJECT_SCHEMA`, registros do `ph2d-ecs`, ids de
  scrollbar, números de ADR. O valor certo raramente está em qualquer um dos lados de um conflito, e ⚠️ **a colisão
  passa MUDA quando duas linhas escrevem o MESMO literal**: o git não sabe o que o número significa.
- **A fonte de cada número é o código, não esta seção** — `PROJECT_SCHEMA` em
  [`project_schema.rs`](shells/desktop/src/project_schema.rs) (⚠️ com **a escada ao lado** e **a tripla** em
  [`project_schema_tests.rs`](shells/desktop/src/project_schema_tests.rs) — **três** sítios, nunca um; e a
  `line/physics` **partiu** aquele arquivo em 15/08, então um degrau escrito no `project.rs` funde **limpo** e evapora)
  · `VEC_SCENE_SCHEMA_VERSION` / `FLIP_SCHEMA_VERSION` / `DOC_VERSION` nas crates que os declaram.
  ⛔ **Não copie esses valores para cá** — esta seção já os teve errados cinco vezes.
- **O número da próxima cena de smoke se CONTA lendo o roteador**, nunca uma nota (a nota já envelheceu em 11 cenas
  de uma vez): Motion [`motion_state_demo_router.rs`](shells/desktop/src/motion_state_demo_router.rs) · Física
  [`physics_smoke.rs`](shells/desktop/src/physics_smoke.rs) · os gates `no_two_*_scenes_claim_the_same_level`.
- **Antes de construir um item de lista aberta, MEÇA se a composição já o exprime.** Seis células da conferência do
  Motion envelheceram antes de alguém voltar a elas: *o que se perde ao não reconferir não é tempo, é construir o que já existe.*
- **⛔ O que foi MEDIDO E REJEITADO não se reconstrói** — está no handoff da wave que o mediu e na história arquivada.
- **Integrar não é aprovar.** Smoke é do Enio; integrar e shipar só por ordem explícita dele (§0.7).
- ⚠️ **Nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`** (medido: o mesmo binário deu 11,36 e
  5,50 ms para o mesmo passe). Gates de razão reprovam sob carga sem que uma linha de código tenha mudado.
- ⚠️ **Gates de GPU são `#[ignore]`** e precisam de adapter — *skip gracioso não é verde*; e o `nextest` **cancela na
  primeira falha**: use `--no-fail-fast`, senão suítes inteiras nunca chegam a correr.

### §5.1 — Módulos

- **Motion Nodes** — dinâmica declarativa sobre `ph2d-nodegraph` ([ADR-0030..0039](docs/architecture/decisions/) +
  [SKILL §11.13](SKILL_Stack_PH2D_Definitiva.md)); avaliador `ph2d-eval-motion`, painéis `motion-graph`/`motion-params`.
  Semântica Houdini: as `force.*` são **Pure** e acumulam em `accel`; **UM** integrador aplica. Cook **GPU-resident por
  default** (`PH2D_GPU_COOK=0` volta à CPU, útil para bissecar). Editor completo (palette `A`, splice, bypass, clipboard,
  grupos, adapters). O catálogo é conferido nó a nó pelo [plano 89](docs/Motion%20Nodes/89_plano_conferencia_dos_nos.md)
  + as 17 folhas em [`89_conferencia/`](docs/Motion%20Nodes/89_conferencia/); o placar é **derivado** por
  [`placar_conferencia.py`](docs/Motion%20Nodes/ferramentas/placar_conferencia.py), nunca escrito à mão.
  ⚠️ **Todo canal novo é side-metadata no REGISTRY, nunca o contrato** (`NodeOp`/`OpResolver`/`NodeManifest` — §6).
  **Aberto:** os P1 restantes da folha 03 (simulação) · o gate `#[ignore]`
  `the_ceiling_is_honoured_on_every_tick_including_the_turn` (cena `=53`) — ⛔ **não afrouxe a barra** · a composição
  sub-passos × `damping` da `motion.verlet_rope`, **medida e não curada de propósito** · ⛔ a faixa de barras do
  `value.pattern` foi **revertida por veredito de produto sem mecanismo nomeado**: uma 2ª tentativa começa perguntando
  *o que ficou pior*, não reconstruindo (a árvore sobrevive em `ae35416bd`).
  **Smokes:** `PH2D_GPU_COOK_DEMO=<n>` · `PH2D_SPLICE_SMOKE` · `PH2D_ADAPTER_SMOKE` · `PH2D_ATTR_SMOKE` ·
  `PH2D_PICKER_SMOKE` · `PH2D_GRADIENT_SMOKE` · `PH2D_AUTOFIX_SMOKE=1..7` · `PH2D_SHAPE_SMOKE` · `PH2D_LENS_SMOKE` ·
  `PH2D_MOTION_OBJ_SMOKE`.
  **Ler:** [`docs/Motion Nodes/`](docs/Motion%20Nodes/) · [handoffs](docs/Motion%20Nodes/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/motion-nodes.md)

- **Timeline** — dope-sheet + graph editor + transporte sobre `ph2d-core::Playhead` (`ph2d-panel-timeline` +
  `ph2d-timeline` + `ph2d-anim`): curvas com handles bézier e weighted tangents, roving keys, time remap, record,
  clips + **composição** ([ADR-0115](docs/architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md)),
  **nesting** ([ADR-0133](docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md)),
  duração explícita, motion path ([ADR-0141](docs/architecture/decisions/)), onion ([ADR-0142](docs/architecture/decisions/)),
  retiming, extrapolação, **sinais** ([ADR-0143](docs/architecture/decisions/)) e expressões
  ([ADR-0144](docs/architecture/decisions/) / [0151](docs/architecture/decisions/) / [0152](docs/architecture/decisions/)).
  ⚠️ O `TimelineDoc` viaja como **blob dentro do `ProjectFile` e carrega a própria versão** — é por isso que ele evolui
  sem mover o `PROJECT_SCHEMA`. ⚠️ A **AUTORIA** de expressões foi **retirada** (o motor ficou; registro em
  [doc 14](docs/Timeline/14_a_autoria_de_expressoes_foi_retirada.md)) — remover a feature **não** removeu o schema.
  **Aberto:** a expressão **PURA** (sem keys) extrapola a strip — ligar exige vínculo autorado (produto + provável
  `DOC_VERSION`) · **W4.T4** (o dock da timeline dentro do Motion) aguarda re-smoke: duas linhas discordaram, o código
  do `main` shipou com cap e gates, e a nota da rejeição saiu do `layout.rs` · o catálogo de receitas morreu, a pesquisa não.
  **Smokes:** `PH2D_NEST_SMOKE=1..3` · `PH2D_MOTION_PATH_SMOKE` · `PH2D_ONION_SMOKE` · `PH2D_TIMESCALE_SMOKE` ·
  `PH2D_STAGGER_SMOKE` (⚠️ **Ctrl**+drag, o KDE rouba o Alt) · `PH2D_BUFFER_SMOKE` · `PH2D_EXTRAP_SMOKE` ·
  `PH2D_SIGNAL_SMOKE` · `PH2D_EXPR_BLEND_SMOKE` · `PH2D_MORPH_FADE_SMOKE`. ⚠️ `PH2D_EXPR_SMOKE` **morreu** com o card.
  ⚠️ **Flake conhecida e PRÉ-EXISTENTE:** `the_cost_of_depth_is_linear_not_explosive` é gate de RAZÃO sensível a carga —
  re-rode sozinho antes de suspeitar de um merge.
  **Ler:** [`docs/Timeline/`](docs/Timeline/) · [`BUGS_timeline.md`](docs/Timeline/BUGS_timeline.md) ·
  [handoffs](docs/Timeline/handoffs/README.md) · [história](docs/archive/estado-2026-08-18/timeline.md)

- **Áudio** — rack com **42 efeitos + 23 presets** e cadeia editável (`ph2d-audio-edit` + painéis
  `audio-editor`/`audio-mixer`), espectral ([ADR-0122](docs/architecture/decisions/)), export Ogg/Opus
  ([ADR-0113](docs/architecture/decisions/) / [0116](docs/architecture/decisions/)), streaming de vozes
  ([ADR-0118](docs/architecture/decisions/)) e **AI denoise nativo** via `tract`
  ([ADR-0123](docs/architecture/decisions/), feature **`audio-ml` OFF por default**).
  ⚠️ **Invariante da rack:** todo efeito é **no-op byte-idêntico no ponto neutro** e o painel **se auto-popula** da
  tabela `KINDS` — efeito novo = variant + braços + row, **zero mudança de painel**.
  ⚠️ **Fronteiras duras, com gate:** nenhum **codec** e nenhum **runtime de ML** alcança o mixer RT.
  ⚠️ **HR-13 emendado** ([ADR-0117](docs/architecture/decisions/)): *quem declara budget possui um gate que **MEDE*** (dhat).
  ⚠️ `fx.rs` está **no teto de LOC** — o 43º efeito tem de orçar o split.
  **Aberto:** o backlog do módulo vive em [`docs/Audio/03_o_que_falta.md`](docs/Audio/03_o_que_falta.md), **com o
  gatilho que acorda cada item** — é lá que se olha, não aqui. Cercas de Chesterton conhecidas: seek/scrub num stream ·
  pitch ao vivo num stream · toggle "Streamed" no Delivery (os três esperam um consumidor real).
  **Smokes:** `PH2D_AUDIO_DELIVERY_SMOKE` · `PH2D_AUDIO_ML_SMOKE` (+ `--features audio-ml`, e ⚠️ **`--release`**: o modelo
  é 16× mais lento em debug) · `PH2D_AUDIO_ML_SMOKE_SECS=180` (sem isso a barra de progresso passa voando).
  **Ler:** [`docs/Audio/`](docs/Audio/) · [handoffs](docs/Audio/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/audio.md)

- **Painter** — host de Layers + Efeitos ([ADR-0099](docs/architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md))
  **mais** um motor de pintura clean-room do Blender Texture Paint (ref vendorizada, GPL ⇒ só comportamento). Quatro
  **meios** num dropdown (`Digital` é o default) — Digital · Watercolor · **Impasto** (relevo com material por-pixel e
  luz na GPU) · **Wet Paint** (sim de fluido, [ADR-0134](docs/architecture/decisions/0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md),
  solver independente de ordem [ADR-0147](docs/architecture/decisions/), row-parallel [ADR-0145](docs/architecture/decisions/)).
  Mais **sculpt do relevo** (8 verbos), **liquify** ([ADR-0157](docs/architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md)),
  **substrato/papel**, **taper**, **grid stamp**, seleção com caneta, e o carimbo no device (`ph2d-paint-gpu`).
  ⚠️ **A lei que este módulo pagou seis vezes:** *o traço é fato do **CAMINHO**, nunca de quão fino o motor amostrou o
  caminho* — um produto por-dab depende do Spacing e do polling; a forma certa é envelope ou integral de arco.
  ⚠️ **`DEPTH_UNIT_PX = 16.0`**: toda grandeza **geométrica** sobre `h` cruza essa conversão na entrada.
  **Aberto:** ⛔ a composição da cobertura da aquarela muda **toda** cruz/laço/hachura já pintada — **produto** ·
  ⛔ pré-agrupar por banda as arestas do `fill_coverage` (o pré-filtro ingênuo é **mais caro que o passe inteiro**) ·
  ⛔ o caráter do Speed sem rampa **reabre um look que o Enio já recusou** · o **endurecimento da borda da máscara**
  (as duas leis de acúmulo já foram tentadas, cada uma com artefato — a próxima hipótese tem de estar noutro lugar) ·
  a cauda do taper no impasto (o próximo passo **não é código**) · `undo` de knobs de painel **não existe em nenhuma
  ferramenta do app** — decisão do Enio · dois `watercolor_app_params_incremental_*` seguem `#[ignore]` com **diagnóstico
  novo** (é raio de invalidação, e ⛔ `pad += 2·raio` **não** é a cura: vira canvas inteiro por quadro) · dois gates de
  razão de `plane_copy`/`undo_delta` vermelhos porque **a premissa da calibração dissolveu** (o serial deixou de ser
  fault-bound) — pede varredura por tamanho com alocação **fria**, ⛔ não baixar a barra.
  **Smokes:** `PH2D_IMPASTO_SMOKE=1|2` · `PH2D_WETPAINT_SMOKE` (+ `PH2D_FLUID_PROFILE=1`) · `PH2D_MASK_SMOKE` ·
  `PH2D_TAPER_SMOKE` · `PH2D_LINE_SMOKE` · `PH2D_SUBSTRATE_SMOKE`. Diagnóstico: `PH2D_PAINT_PERF=1` ·
  `PH2D_PREVIEW_DIAG` · `PH2D_PREVIEW_DUMP=<dir>`.
  ⚠️ **Rode a suíte do Painter em DEBUG também** (precedente registrado), e os `--ignored` com **`--test-threads=1`**
  e a máquina calma.
  **Ler:** [`docs/Painter/`](docs/Painter/) · [`BUGS_painter.md`](docs/Painter/BUGS_painter.md) ·
  [`28_otimizacoes_o_que_funcionou.md`](docs/Painter/28_otimizacoes_o_que_funcionou.md) (o log de perf, com o que foi
  **rejeitado por medição**) · [handoffs](docs/Painter/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/painter.md)

- **Vector** — motor GPU-first, editor-first, **referenciado** no runtime MIT do Rive (reimplemento nativo kurbo/Vello,
  *não* vendoriza rive-rs; [ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)).
  Todo path é **entidade ECS** com pose no `Transform` ([ADR-0110](docs/architecture/decisions/) /
  [0111](docs/architecture/decisions/)); 13 modos (Select/Node/Pen/Build/Width/Tesoura/…,
  [ADR-0112](docs/architecture/decisions/)); **Live Corners** ([ADR-0121](docs/architecture/decisions/)) e a costura
  **fonte ≠ cozido** que destravou os **Live Path Effects** ([ADR-0132](docs/architecture/decisions/)); blend
  ([ADR-0128](docs/architecture/decisions/)); largura viva ([ADR-0148](docs/architecture/decisions/)); **auto layout**
  via `taffy` atrás de uma crate-folha ([ADR-0153](docs/architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md)).
  Mais guias/régua, simetria como modo, booleana viva, moldura, **tokens no documento**, **estados de UI + Smart
  Animate**, e a **árvore autorada como painel vivo** (o app escreve o código do painel).
  ⚠️ **A lei do ADR-0153:** *o passe publica **onde** as coisas ficam; ele não escreve **onde** elas estão* — nada no auto
  layout toca `Transform`, senão cada quadro de um resize vira um passo de undo.
  ⚠️ **Regra-mãe do pen:** *o que se vê/aponta/encaixa é MUNDO; o que o documento guarda é LOCAL.*
  **Aberto:** ⏸️ o `n`/folga do *tether* e o `DRAG_RATE_X = 50` são números de **FEEL sem medição atrás** (a lei irmã diz
  `rate = step`, **50× menos**, em **141 campos**) — do Enio, com o número na mão · ⏸️ abrir/fechar painel **nunca** foi
  animado (ausência, não regressão; e **não** é o gêmeo da dobra) · a cascata da **F5**, o menu radial (**E4**), o realce
  de proveniência (**C2**), som de UI (**D1**, ⛔ nunca ligado por omissão) e partículas (**D2**) · o **hit-test só
  recebe o produtor de OFFSET** — os outros seis produtores de `LiveGeometry` não chegam ao pick (a cura é o pick ler o
  mapa **fundido**; wave própria) · ⚠️ uma superfície `Plain` **nova** que leia `hover_live` sem estar no mapa nasce muda,
  e ⛔ **não alargue o censo a todo `Plain`** (revive a cerca do estudo §6.2, e há gate).
  **Smokes:** `PH2D_BUILD_SMOKE=<n>` (⚠️ várias cenas **imprimem o que montaram** — *se a linha não aparecer, PARE*) ·
  `PH2D_UI_MOTION_SMOKE=1..3`. Diagnóstico: `PH2D_BUILD_LOG=1`.
  ⚠️ **Preferência de utilizador fora do repo:** `~/.ph2d/prefs.txt` (`motion_character`, `reduced_motion`) — um
  `reduced_motion=1` esquecido **reprova smokes sobre produto correto**.
  **Ler:** [`docs/Vector Module/`](docs/Vector%20Module/) · [`BUGS_vector.md`](docs/Vector%20Module/BUGS_vector.md) ·
  [handoffs](docs/Vector%20Module/handoffs/README.md) · [história](docs/archive/estado-2026-08-18/vector.md)

- **Física** — *runtime-truth* + bake opcional sobre o `rapier2d` que já existia; a linha escreve **integração e
  autoria, não solver** ([ADR-0131](docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)).
  Crate-ponte `ph2d-physics-ecs` (components de **CONFIG**, nunca estado vivo de solver — o undo ordena por bytes),
  ring de checkpoints GGPO, painel global de mundo (tecla `W`), camadas de colisão, **joints como ENTIDADES**
  (7 tipos + polia/talha/tambor + pino de mundo, [ADR-0149](docs/architecture/decisions/0149-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md)),
  a família das **zonas** (força/arrasto/empuxo/torque/falloff/frame/espelho), contatos e sinais, e o
  **player de plataforma** (`ph2d-platformer`, lei pura) em três modos.
  ⚠️ **`BTreeMap`, nunca `HashMap`** — é a espinha do determinismo (lint estrutural), e o hash `physics_ecs_c9` roda na
  matriz 3-OS provando **o nosso** código, não o wrapper.
  ⚠️ **Sem porta de escala:** o `Transform` já é metros; a única px→m é `ProjectSettings.pixels_per_meter`, do projeto.
  **Aberto:** o campo de **atração** não alcança um player de pose própria (é **sustentado**, pede canal por-tique) ·
  `bXYOverride` do Unreal, quando houver quem peça · a trava de beirada não tem gesto de canvas · **quatro ❌** na
  [auditoria 09](docs/Physics/09_auditoria_engines.md) e ⚠️ **nenhum é trabalho pendente** (dois foram **recusados por
  medição**, um é arquitetura com *não agora* escrito, um está fora da fila) — *um ❌ «recusado com motivo» e um ❌
  «ninguém fez» leem igual numa tabela* · o buraco real contra o referencial é *obstacle actions: climbing* (plano 08 §4.8).
  **Smokes:** `PH2D_PHYSICS_SMOKE=<n>` (⚠️ **`=84` não existe, de propósito**).
  **Ler:** [`docs/Physics/`](docs/Physics/) · tracker [`HANDOFF_line_physics.md`](docs/Physics/handoffs/HANDOFF_line_physics.md) ·
  [`00_plano_waves.md`](docs/Physics/00_plano_waves.md) · [`BUGS_physics.md`](docs/Physics/BUGS_physics.md) ·
  [handoffs](docs/Physics/handoffs/README.md) · [história](docs/archive/estado-2026-08-18/physics.md)

- **3D / Sculpt** — módulo **drop-crate puro** ([ADR-0150](docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)):
  `ph2d-mesh` (malha residente + octree) · `ph2d-mesh-render` (passe wgpu + matcaps) · `ph2d-sculpt3d` (kernels) ·
  `ph2d-sdf` (remesh por Surface Nets). Referência **SculptGL (MIT)** para o `s-mode` e **Blender** para o `b-mode`, com
  os kernels **portados e medidos contra o oráculo a um ULP de `f32`** (o Node roda o próprio SculptGL). **23 verbos**,
  multires, extract/transform, alpha por imagem, oclusão de forma, e a razão de existir: **a malha DOA a normal** e a
  tinta 2D chapada sai acesa pela forma (`ph2d-light` é o dono do rig de luz, e por isso **não é removível**).
  ⚠️ **A navegação orbital mora no SHELL, nunca numa `Tool`** — é isso que mantém `Tool=12` (§6) fora do caminho.
  ⚠️ Sem a env var o `AppGfx.sculpt3d` é `None` e **o frame 2D é byte-idêntico**.
  **Aberto:** ⛔ os defaults do `b-mode` e o *Draw Sharp* são **decisão de produto** (os do Blender moram num `.blend`
  **binário**) · ⛔ a lei do zoom (adotar a do SculptGL **diverge da referência**; o número está numa cerca executável) ·
  ⛔ as três divergências **declaradas** da referência, cada uma com gate defendendo a nossa posição · a outra metade da
  **W4**, **W7** (plano MLS), **W9** (Mesh Filter — *o mais barato: não há kernel novo*), **W10-W12**, e o **marching cubes**.
  **Smokes:** `PH2D_SCULPT3D_SMOKE=<n>`. ⚠️ **Rode uma vez SEM a env var** — é a metade que prova a inércia.
  **Ler:** [`docs/3D/`](docs/3D/) · [handoffs](docs/3D/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/sculpt3d.md)

- **Flip** — animação 2D no idioma do Grease Pencil: tira de quadros, onion, tween v2 (correspondência por atribuição
  ótima + espiral logarítmica), **colorize LazyBrush**, multiplano 2.5D, airbrush, pressão, e o
  **motor novo de traço**, em que o traço deixa de ser rasterizado e passa a ser **PERCORRIDO**
  (`τ = ∫ f(dn) ds / pitch`, `α = 1 − exp(−τ)`) — a lei aditiva da tinta, que é o **limite** dos buffers de dab da
  indústria. Dois motores, **uma lei**: referência em CPU e o compute que shipa, unidos por gate de paridade cuja barra é
  **derivada do formato** (`rgba16float` ⇒ `2⁻¹¹`).
  ⚠️ `PH2D_FLIP_NEW_ENGINE=0` volta ao rasterizador antigo (vivo e testado, útil para bissecar).
  **Aberto:** cache em **tiles de MUNDO** (sobreviver ao pan) · o **resíduo de quina** que a lei de área expôs
  (**13 px de 1115** — *não é regressão*) · cache **incremental** do ajuste · e **três itens que são decisão do Enio, já
  devolvidos com os números**: o resíduo de quina, **joins & caps** (⛔ a premissa de correção foi **refutada**; sobra
  pergunta de produto) e a **terceira lei** (o `Soft` do Krita — funciona exato, muda a borda em **+69%**).
  **Smokes:** `PH2D_FLIP_HARDNESS_SMOKE` (o mestre) · `PH2D_FLIP_COLORIZE_SMOKE` · `PH2D_FLIP_STRIP_SMOKE` ·
  `PH2D_FLIP_TIP_SMOKE` · `PH2D_FLIP_TWEEN_SMOKE` / `_PAIRS_` / `_PHASE_` / `_TORSION_` · `PH2D_FLIP_MULTIPLANE_SMOKE` ·
  `PH2D_FLIP_SELF_OVERLAP_SMOKE` · `PH2D_FLIP_AIRBRUSH_SMOKE` · `PH2D_FLIP_RESAMPLE_SMOKE` · `PH2D_FLIP_PRESSURE_SMOKE`.
  Diagnóstico: `PH2D_FLIP_STATS=1` · `PH2D_WALK_DUMP=<dir>`.
  ⚠️ **Rode a suíte em DEBUG e RELEASE** — um gate desta linha reprovou **só em debug** (um bar de relógio mede o perfil do build).
  **Ler:** [`docs/Flip/`](docs/Flip/) · [`BUGS_flip.md`](docs/Flip/BUGS_flip.md) ·
  [`12_novo_motor_pesquisa.md`](docs/Flip/12_novo_motor_pesquisa.md) · [handoffs](docs/Flip/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/flip.md)

- **Runtime — a saída de sinais (R0)** — crate-folha `ph2d-runtime` (`Signal`/`SignalOrigin`/`SignalOutbox`/`SignalReader`),
  onde a **timeline** ([ADR-0143](docs/architecture/decisions/)) e a **física** (`SignalOnHit`) se encontram: os produtores
  **publicam** e cada consumidor lê com o próprio cursor (ADR-0075 — o produtor não chama ninguém).
  ⚠️ **A ordem no quadro é load-bearing e tem gate:** o quadro vira **antes** do primeiro produtor, o dreno roda **depois**
  dos dois — fora dessa janela o sinal chega um quadro atrasado (invisível num toast, visível quando o consumidor for **som**).
  ⚠️ **MEDIDO:** 8 consumidores custam **1,00×** o de 2 — o custo mora no produtor. *O trabalho do R3 é a tabela
  **nome → ação**, que é conteúdo autorado e precisa de UI, não o fan-out.*
  **Aberto:** ⚠️ **adjacência de NOME com uma linha viva** — o plano de UI/UX aponta `ph2d-runtime` para o *runtime de UI*.
  Hoje não há conflito; a decisão é do Enio (a linha recomenda **crate irmã**, senão o gate `the_event_core_is_a_leaf` é
  deliberadamente revogado) · o envelope por seções (**F1.W0**) **não existe no `main`** — é recuperável de `37ff53467`,
  mas **o desenho volta e o diff não** · **R1** (`shells/game`) segue adiado por decisão do Enio.
  **Smokes:** `PH2D_SIGNAL_SMOKE=1|2` (+ `PH2D_SIGNAL_LOG=1`).
  **Ler:** [`docs/Runtime/`](docs/Runtime/) · [handoffs](docs/Runtime/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/runtime.md)

- **Editor / shell — undo, persistência, inspector** — **uma** fila de undo, snapshot-based, registrada por **DIFF num só
  ponto** (`App::post_frame_undo`), cuja unidade é `ProjectState = {WorldSnapshot + VecScene}` — a MESMA captura que a
  persistência usa (o save só anexa os pixels). Ctrl+S/Ctrl+O salvam o projeto inteiro num postcard versionado.
  ⚠️ **`canonicalize()` ordena por CONTEÚDO, nunca por `Entity::to_bits()`** (id de alocação) — foi isso que fazia todo
  frame virar um passo espúrio; e **toda raiz ganha `RootOrder` explícito**: *não se escolhe um desempate melhor, não se
  tem empate*.
  ⚠️ **Referência durável entre objetos é o NOME** (`stable_name_id`, hash do `Name`), nunca os bits — o undo respawna
  tudo com bits novos, e bits **dentro dos bytes de um componente** envenenam o próprio undo.
  ⚠️ O undo de **PAINÉIS** é sistema separado e **não existe** (decisão do Enio).
  **Aberto:** UI real de Save/Save As/Open (o `io_menu` é stub — hoje é path fixo, sem diálogo) · persistir
  `SpriteSource::Individual` e `CookedTexture` · limpar o `vec_history` morto (subsumido pela captura).
  **Ler:** [`project.rs`](shells/desktop/src/project.rs) · [`undo.rs`](shells/desktop/src/undo.rs) ·
  [história](docs/archive/estado-2026-08-18/editor-shell.md)

- **Retirados (histórico — não reconstrua sem ler o porquê):** a simulação de **aquarela/fluid/wash**
  ([ADR-0096](docs/architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md), supersede ADR-0085..0095) ·
  o **brush engine** original ([ADR-0099](docs/architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md),
  supersede ADR-0043..0053/0097/0098) · o **sistema vetorial antigo** (30 crates,
  [ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)).
  ⚠️ A pintura e o vetor **voltaram** com motores novos (acima); `docs/Novo Painter/` e os handoffs W1/W2 do vetor são
  **históricos**. [história](docs/archive/estado-2026-08-18/watercolor-removido.md)

- **Planos de nós (fan-out):** [`2026-05-node-waves.md`](docs/plans/2026-05-node-waves.md) (W1+W2 fechados, contrato
  **CONGELADO** — §6; fan-out aberto) · [`2026-05-wave-11-carry-overs.md`](docs/plans/2026-05-wave-11-carry-overs.md)
  ([ADR-0042](docs/architecture/decisions/0042-wave-10-closure.md)) ·
  [história](docs/archive/estado-2026-08-18/planos-de-nos.md).
  Fechados sem pendência: **Sprite Inspector v2** ([ADR-0069..0074](docs/architecture/decisions/)) · **KTX2 Fase 2**
  ([ADR-0055](docs/architecture/decisions/), W3 = integração com o Painter) · **imageio AVIF**
  ([ADR-0054](docs/architecture/decisions/)).

## §6 — Contratos congelados (mexer = Coord-only + ADR; DIRETRIZ §4)

- **Nodes** ([ADR-0039](docs/architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)): `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — gate `architecture_contract_surface`.
- **Tools** ([ADR-0040](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md)+[0041](docs/architecture/decisions/0041-rasteredit-rename-and-deactivate.md)): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` — gate `architecture_tool_contract_surface`. (`Tool` 10→11 em [ADR-0040-amendment-2](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md): `on_tick` heartbeat p/ aquarela live, ADR-0049/0077-D11. `Tool` 11→12 + sub-trait `CanvasPaintTool` em ADR-0040-amendment-3: `as_canvas_paint_mut`/`on_canvas_pointer` p/ entrega de ponteiro de canvas ao novo Painter, `docs/Painter/`.)
- ~~**Painter (pintura)** (ADR-0043..0053)~~ — **REVOGADO** por [ADR-0099](docs/architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md): os ABIs de pintura (`PainterUiEdit`/`Brush`/`Stamp=96B`/`RenderingMode=6`/`PointerSource`/`DeviceTier`…) e o gate `architecture_painter_contract_surface` (crate `ph2d-painter-contracts`) foram **removidos** junto com a pintura. A superfície de **efeitos** que sobrevive (`AdjustmentKind≤32`/`AdjustmentParams`/`BlendMode`+`MAX_BLEND_MODES`/`apply_blend`) vive agora em **`ph2d-painter-effects`** (não-gateada; re-capear é follow-up). `ColorProfile` segue em `ph2d-color`.
- ~~**Watercolor (física)** (ADR-0049/0078-0084)~~ — **REVOGADO** por [ADR-0096](docs/architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md): a sim de aquarela e seus gates (`gpu_parity`/`composite_parity`) foram removidos junto com a crate `ph2d-painter-wash`. O modelo K–M espectral é histórico (backup); o pivot para mixer-brush usa Kubelka–Munk/Mixbox no blend do pigmento, não shallow-water. Nada congelado aqui.
- **Vector (data-model foundational)** ([ADR-0056..0068](docs/architecture/decisions/)): `VectorOp≤16`/`Vertex`SmallVec32/`Segment`64/`Region.segments`16/`AnimValue` enum/`sample(t:f64)`/`MAX_SPIRAL_TURNS=64`/`MAX_POLYGON_SIDES=128`/`MAX_VERTICES_PER_LLM_GEN=1000` — gate `architecture_vector_contract_surface` (escaneia só `ph2d-vector-doc`+`-traits`). **PERMANECE congelado** e o gate FICA — mesmo após [ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md) ter **retirado** as tools/nodes/panels de edição vetorial (o cutover mexe só em crates satélite, não na superfície do doc). O **motor novo** (`ph2d-vec-*`, §5) tem contrato **próprio, ainda NÃO congelado** (re-congelar é follow-up). Gate `vello_kurbo_only_in_ph2d_vector` nunca existiu (era W2-deferred).

## §7 — Design system

[`docs/design/PROMPT_CLAUDE_DESIGN.md`](docs/design/PROMPT_CLAUDE_DESIGN.md) (brief: tokens.json + mockups + icons + specs) alimenta os widgets em Vello sobre [`ph2d-editor`](crates/ph2d-editor/) (ADR-0023). Mockup de referência: [`docs/design/component-library.html`](docs/design/component-library.html).

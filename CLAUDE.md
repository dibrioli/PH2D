# CLAUDE.md — núcleo operacional do PH2D (LEIA INTEIRO — é curto de propósito)

> Toda LLM recebe este arquivo automaticamente. Ele é o **roteador**: os inegociáveis +
> para onde ir por tarefa. Detalhe técnico → [`SKILL_Stack_PH2D_Definitiva.md`](SKILL_Stack_PH2D_Definitiva.md)
> (HR-1..HR-18, stack). Processo → [`DIRETRIZ.md`](docs/IntegracaoMultiAgente/DIRETRIZ.md).
> Não leia esses dois inteiros — use o roteador §1.

## §0 — Inegociáveis (memorize os 9)

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
8. **O Enio é o DONO do produto, não um engenheiro acompanhando o desenvolvimento.** Ele decide e ele testa — mas **não conhece as ferramentas que estamos construindo**: o smoke é onde ele as **APRENDE**. Toda resposta a ele (Enio, 2026-08-18):
   - **Curta. Só o essencial.** Se ele quiser o detalhe técnico, ele pede.
   - **Sem jargão.** Nada de gate, schema, crate, ADR, contagem de mutação, nome de arquivo ou de função — a menos que ele pergunte. Diga **o que ele consegue FAZER agora**, não o que foi construído.
   - **Smoke em PASSOS NUMERADOS, escritos para quem nunca viu aquilo:** (1) o comando **completo, com o `cd`**, copiável de uma vez · (2) onde clicar / o que pegar, com o nome que aparece **na tela** · (3) o que tem de acontecer · (4) **como saber que deu errado**. Nunca só o nome da variável de ambiente.
   - ⚠️ **Isto vale para a resposta AO ENIO.** Handoff, ADR, doc-comment, gate e mensagem de commit continuam **técnicos e densos** — o leitor deles é a próxima LLM, não ele. Baixar o rigor deles não é o pedido.

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
| **ABRIR uma linha nova** | [MODELO_ABERTURA_LINHA.md](docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) — o bloco colável da 1ª mensagem (`/pd-linha-abrir`) |
| **FECHAR a sua linha** | DIRETRIZ §1.5.9 — gate batched 1× · handoff · `rm -rf target/*/incremental` · **UMA LINHA** no §5 (`/pd-linha-fechar`) |
| **Você é o agente INTEGRADOR** (só por ordem do Enio) | ⚠️ **`collision-surface.sh` em cada worktree ANTES do primeiro grep** — ele responde de uma vez a lista que a integração redescobre ~1.000 vezes. ⚠️ **Invoque o caminho ABSOLUTO do primário** (`bash /…/PH2D/scripts/collision-surface.sh`): uma worktree forkada antes do script **não o tem**, e ele mede a árvore de onde foi CHAMADO — *um script novo só existe nas árvores que nasceram depois dele*. Depois DIRETRIZ §1.5.3 + `scripts/foundational-integrate.sh` (`/pd-integracao`) |
| **Rodar uma jornada Modo L (você, operador)** | [GUIA_JORNADA_MODO_L.md](docs/IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) — abrir linhas, quando intervir, quem faz o ship (sem coordenador) |
| **Você ASSUMIU uma linha que já existe** (troca de janela / retomada pós-integração) | [MODELO_TROCA_DE_AGENTE_NA_LINHA.md](docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md) — **`cd` + `pwd` + `git branch --show-current` ANTES de ler qualquer arquivo.** A janela abre na raiz (=`main`) e o mesmo path relativo existe nas 2 árvores: editar a errada compila e commita **sem erro** |
| **Build lento / quero voar** | DIRETRIZ §6 (stack de velocidade) — §2 abaixo é o resumo |
| **Dúvida de stack / Hard Rule** | SKILL_Stack §HR-1..18 (cite por ID) |
| **«Que versão de Rust/wgpu/vello eu uso?»** | [`STACK_VERSOES.md`](docs/IntegracaoMultiAgente/STACK_VERSOES.md) — **uma página, gateada contra o `Cargo.lock`** (o gate `architecture_stack_versions_doc_matches_the_lockfile` a mantém honesta). Rust **1.98**/edition 2024 · `wgpu` **29** · `vello` **0.10** · `parley` **0.11** · `rapier2d` **0.35** · `bevy_ecs` **0.19** |
| **Subir dependência · «dá para atualizar X?»** | ⚠️ **`bash scripts/stack-audit.sh --tetos` ANTES de responder** — «o mais recente possível» ≠ «o mais recente»: hoje **8** crates são seguradas por outra (o `vello` prende o `wgpu` em 29, o `parley` prende o `skrifa` e o `accesskit`, o `rfd` prende o `pollster`…), e forçar não dá erro de resolução — dá **duas cópias**, e um `Device`/`NodeId` de uma não serve à outra. Plano vivo: [`docs/Atualizar Stack/`](docs/Atualizar%20Stack/) |
| **Física / corpo rígido / colisão** | [ADR-0131](docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) (o *porquê*) + tracker [`docs/Physics/handoffs/HANDOFF_line_physics.md`](docs/Physics/handoffs/HANDOFF_line_physics.md) (estado) + [`00_plano_waves.md`](docs/Physics/00_plano_waves.md) (waves) + [`BUGS_physics.md`](docs/Physics/BUGS_physics.md) (bugs cuja causa enganava) |
| **Fim de dia · o disco encheu · "por que o target é tão grande?"** | [DIRETIVA_FIM_DE_DIA.md](docs/IntegracaoMultiAgente/DIRETIVA_FIM_DE_DIA.md) — os 3 portões antes de apagar, e a **§2-bis** com a decomposição MEDIDA do target (54% é `incremental/`) e as 3 regras que atacam o pico. ⚠️ **Primeiro `bash scripts/btrfs-health.sh`**: «disco cheio» com 500 GB livres é a **metadata do btrfs** sem espaço para crescer — `df` não a vê, apagar target não cura, e a cura (balance) é root → [runbook](docs/DevOps/BTRFS_METADATA_E_SWAP.md) |
| **Quem é o Enio / estado do projeto** | [project-memory/MEMORY.md](project-memory/MEMORY.md) |
| **Quem possui o quê agora** | **Modo L: `git worktree list`** — o registro de posse é a própria árvore, e responde na hora. ⚠️ [SESSION_ACTIVE.md](docs/SESSION_ACTIVE.md) é do **Modo C** e o seu único escritor autorizado (o Coordenador) **não existe neste tier**: em 2026-08-18 ele estava parado desde 04/08 com 5 worktrees vivas |
| **Achar um ADR pelo número** | [`decisions/README.md`](docs/architecture/decisions/README.md) — índice **derivado** (`bash scripts/adr-index.sh`), com o `Status:` e quem alega supersedê-lo |
| **Reimplementar código RESTRITO (GPL/proprietário)** | [SKILL_Cleanroom_Reimplementacao.md](docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md) (`/pd-cleanroom`) — triagem de licença PRIMEIRO (a porta permissiva quase sempre existe); **uma feature = UMA linha, UMA janela** (E/R são subagentes; a retomada assume a MESMA linha); ⛔ quem escreve o produto nunca viu o fonte do alvo |

> **Os 15 comandos de fluxo vivem em [`.claude/commands/`](.claude/commands/) e são versionados** —
> `/pd-feature` · `/pd-linha-abrir` · `/pd-linha-assumir` · `/pd-linha-fechar` · `/pd-integracao` ·
> `/pd-ship` · `/pd-auditoria` · `/pd-mutacao` · `/pd-gate-fechamento` · `/pd-bug-causa` · `/pd-perf` ·
> `/pd-adr` · `/pd-smoke-report` · `/pd-livre` · `/pd-cleanroom`. Cada um é o protocolo desta tabela já destilado.
> ⚠️ **Medido 2026-08-18: existiam há semanas em `~/.claude/commands/` (fora do repo), nenhum doc os
> apontava, e foram usados ZERO vezes em 101 sessões** — enquanto a compactação disparou 1.777 vezes.
> *Uma ferramenta fora do repo não existe nas outras máquinas, e uma que o roteador não aponta não existe em nenhuma.*

## §2 — Velocidade ("agents flying"), resumo (detalhe + configs: DIRETRIZ §6)

- **PRIMEIRO: `bash scripts/hw-profile.sh`** — a estratégia é função do hardware, não fixa. Os bullets abaixo são o baseline `constrained` (Mac 8 GiB); tier `workstation` (desktop 128 GB) sobrescreve (RA full, muitos cargos, slots opcionais, tmpfs, sccache). Tabela: DIRETRIZ §6.0. Racional: [ADR-0104](docs/architecture/decisions/0104-hardware-tiered-speed-strategy.md).
- **UM TURNO, N CHAMADAS.** Chamadas independentes — ler 3 arquivos · 2 greps · medir 2 números · rodar 2 sondas — vão na **MESMA** mensagem. ⚠️ **Medido 2026-08-18: 279.566 turnos com 1,00 chamada cada** (oito em 279 mil usaram duas), sobre uma mediana de **991 turnos por sessão**, cada um um round trip completo. É a **maior alavanca de relógio deste repo** e não muda processo nenhum. Só serialize o que **depende** do resultado anterior.
- **EDITE pela ferramenta `Edit`, não por `python3`/`sed`.** ⚠️ Medido: **52% das edições iam por script**, ~93 k tokens de geração a mais por sessão — e o modo de falha é o caro: um `str.replace()` que não casa é **no-op SILENCIOSO e o script imprime sucesso**, enquanto o `Edit` **falha alto** quando `old_string` não casa. Script só onde ele é a forma CERTA (mutação: backup → mutar → testar → restaurar · renomeação em N arquivos · edição derivada de medição), e aí **sempre com `assert` de contagem** ([`project-memory`](project-memory/feedback_python_replace_silent_noop_after_fmt.md)).
- **Inner loop = `bash scripts/cargo-check-narrow.sh <crate>`.** Nada de test/clippy/auditor por task. ⚠️ **Gate red-first e prova de mutação NÃO são o inner loop** — eles rodam teste de propósito, uma vez cada; o que não pode é o teste responder ***"minha edição entrou?"***, que é pergunta de `check`. Razão medida: **4,3:1 na direção errada**.
- **Corrida dirigida de teste = `bash scripts/cargo-test-narrow.sh <crate>`** (red-first, mutação, "este teste passou?"). Ele roda `check --all-targets` **na frente** e sai cedo se não compilar, e devolve **exit code distinto**: `0` verde · `1` teste vermelho · `2` não compilou.
  ⚠️ **Medido 2026-08-18: `cargo test` é 80,5% de TODO o relógio de shell do repo** (59.215 corridas, 340,6 h) — e **4.414 delas (44 por sessão, 6,3 h) nunca chegaram a rodar um teste**, porque o crate não compilava. **98,9%** das corridas carregavam filtro escrito à mão (**177.063 usos, 19.760 formas distintas** — cada uma reescrita ~9×), e o filtro é onde a resposta se perdia: **797 corridas devolveram literalmente NADA**. ⛔ Um `| head` também **destrói o exit code** — é por isso que os dois scripts preservam o do cargo.
- **Comando pesado vai por `bash scripts/ph2d-run.sh <cmd>`** (scope próprio, teto de RAM, **sem swap**). Não é conforto: um teste alocou **90,2 GB** e o `OOMPolicy=stop` derrubou a janela inteira do agente — e o earlyoom não podia salvar, porque havia 27,6 GB de RAM livres e o que acabara era o **swap** ([memória](project-memory/project_vscode_dies_by_oompolicy_not_by_choice.md)).
- **A máquina tem um VIGIA, e ele não se corre — ele corre.** `scripts/sanidade.sh --instalar` arma um timer de 15 min que **só fala quando há problema** (notificação no ecrã, com a AÇÃO); o `ship.sh` já o imprime. Mede o que de facto matou esta máquina 4× — o esforço do `kcompactd`, as páginas fincadas, a fração do livre em pedaço grande, o swap só-zram. ⚠️ **A régua da contiguidade esteve errada DUAS vezes** (contar blocos de 2 MB lê `4` numa máquina com 86 GB livres; somar GB absolutos acusa uma máquina sã cuja RAM está em cache) — leia o cabeçalho antes de lhe tocar.
- ⚠️ **Ferramenta que nenhum passo escrito chama pelo NOME morre.** Medido: `cargo-check-narrow.sh` está nesta seção há semanas, faz exatamente a coisa certa — e foi invocado **5 vezes em 101 sessões**, contra **13.791** `cargo check` digitados à mão. `git-stage-guard.sh` tem 5 docs a apontá-lo e **zero** invocações, contra 8.439 `git status` à mão. Os quatro scripts realmente vivos (`nextest-impacted`, `ship`, `foundational-integrate`, `hw-profile`) têm em comum **uma coisa só**: um passo obrigatório de protocolo os invoca pelo nome. *Ponteiro não é adoção.*
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
- **⛔ O que foi MEDIDO E REJEITADO não se reconstrói** — e desde 2026-08-18 ele tem **endereço**:
  cada doc cortado leva no fim uma tabela **`⛔ Recusas MEDIDAS`**, derivada do arquivo, uma linha
  por recusa com o link para a linha exata. São **126** hoje. ⚠️ **Consulte-a ANTES de propor
  qualquer otimização ou mudança de desenho no módulo** — uma recusa medida diz *o que foi tentado,
  medido e rejeitado, com o mecanismo*, e é a única coisa que impede refazer trabalho já pago.
  *Arquivar sem indexar as recusas seria apagá-las* (o log de perf do Painter guardava **47**, e o
  §5 citava cinco).
- ⚠️ **Cortar um doc é uma operação com PROVA, nunca à mão:** `python3 scripts/doc-split.py <doc>
  --keep <faixas> --archive <destino>`. Ela recusa faixas sobrepostas ou fora de alcance e **aborta
  se as duas metades não remontarem o original byte-a-byte (sha256)**. A história vai **verbatim**;
  o doc vivo fica sendo um roteador.
- **Índice de diretório se GERA, não se escreve** — `bash scripts/doc-index.sh` (14 diretórios,
  `--check` no `ship.sh`). Medido: `docs/Motion Nodes/` tinha **99 arquivos e zero índice**, e **45%**
  dos markdowns eram inalcançáveis a partir deste roteador. ⛔ `docs/Pixel Art/` e `docs/Tilling/`
  ficam de fora **por decisão do Enio** (estão no `.gitignore`: MVPs paralelos ainda sem associação
  com o PH2D) — o §5 não as mencionar **não é buraco, é o produto de uma decisão**.
- ⚠️ **O stack subiu em 2026-08-29** (`wgpu` 29 · `vello` 0.10 · `parley` 0.11 · `bevy_ecs` 0.19 ·
  `rapier2d` **0.35**, cuja matemática deixou de ser `nalgebra` e passou a `glam`/`glamx` — o vocabulário
  vive em [`rmath.rs`](crates/ph2d-physics/src/rmath.rs) e ⛔ `Point`/`Isometry`/`Translation` **não
  existem**). ⛔ **O que NÃO subiu ficou por MEDIÇÃO, não por preguiça** — o `wgpu` 30 é inalcançável
  enquanto o `vello` pedir `^29.0.3`, e unificar o `glam` desligaria o SIMD de 8 crates de desenho:
  [ADR-0168](docs/architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md)
  + [registo](docs/Atualizar%20Stack/04_registro.md). ⚠️ **Antes de responder «dá para atualizar X?»,
  corra `bash scripts/stack-audit.sh --tetos`** — o `ship.sh` já o imprime antes do veredito de push.
- ⛔⛔ **UM CONTROLO MORTO tem DUAS espécies que nenhuma sonda deste repo apanha** (caça de
  2026-08-30: **34 mortos** sobre **~504 controlos** seguidos até ao efeito, registo em
  [§19](docs/Atualizar%20Stack/04_registro.md)). ⚠️ **O passo que um `grep` não vê é o terceiro:**
  *o painel escreve onde · quem lê · **o leitor DECIDE, ou entrega a alguém que descarta?***
  - **O dreno de UM BRAÇO SÓ** — não é um clique sem handler; é um handler cujo `if let` não cobre
    a variante. Seis famílias de widget morrem de uma vez, e a acusação **sobrevive a todo gate de
    registo**.
  - **O consumidor que PROJECTA o valor fora** — o fio está completo, o valor chega ao solver, e a
    matemática descarta-o. *Nenhuma sonda de «quem lê este campo?» o vê: ele **é** lido.*
  ⚠️ **E cinco dos mortos têm a mesma forma: a lente do PAINEL é mais larga que a do CONSUMIDOR** —
  o painel pergunta *«há uma moldura?»* onde o consumidor pergunta *«qual **direcção**?»*. Em três
  deles a regra certa **já estava escrita no mesmo ficheiro**, para o controlo vizinho.
  ⛔ **Nenhum instrumento do repo pergunta se o VALOR chega a um consumidor**: o
  `architecture_panel_wiring_parity` mede *focalizabilidade*, e os `seam_*` provam que o clique
  **chega à ferramenta**, nunca que a escrita dela chega a um efeito.
  ⭐ O único painel **42/42 limpo** é o gerado por **tabela** — *um painel derivado de uma tabela
  não tem onde esconder um knob morto.*
  ⚠️ **E há uma TERCEIRA coisa que se lê igual e cuja cura é OPOSTA: o id ÓRFÃO.** Um `const`
  declarado que ninguém pinta nem regista é **lixo** (cura: apagar); um pintado e registado cujo
  valor não chega a consumidor é **morto** (cura: ligar o braço). A sonda vê os dois iguais, e
  tratar um órfão como morto leva alguém a construir consumidor para um widget que não existe.
  ⇒ pergunte primeiro *isto chega a ser PINTADO?* (2 dos 10 acusados em 30/08 eram órfãos).
  ⛔ E um `HitIndex::register` cujo efeito é **BLOQUEAR** (o fundo de uma janela flutuante) tem
  término por **AUSÊNCIA** — nenhuma varredura de términos positivos o vê, e ensiná-la a aceitar o
  padrão branquearia os cabeçalhos de secção genuinamente mortos, que têm a mesma forma.
- ⛔⛔ **Uma catraca sem censo de obsolescência não desce: ela vira LICENÇA.** Toda lista de dívida
  tolerada deste repo declara-se «só encolhe», e nenhuma encolhe sozinha. Medido 30/08: a lista de
  folgas de LOC por **função** tinha o censo; a de **ficheiros** não tinha, e ao escrevê-lo ele
  acusou **três** entradas obsoletas na primeira corrida — uma delas congelada em `660` havia três
  meses sobre um ficheiro de **536** linhas. ⇒ ao criar uma tolerância, escreva no mesmo commit o
  teste que pergunta *o alvo ainda existe? ainda estoura? a folga ainda o descreve?* — com a
  metade justa, senão uma varredura partida devolve zero obsoletas e lê-se como aprovado.
- ⛔⛔ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente** — a
  ausente não é acreditada. Medido em 2026-08-30: a `=15` prometia que a bola sem CCD atravessa a
  parede, e as duas paravam **no mesmo sítio** desde a `rapier` 0.35 (que varre contra cenário
  **fixo** de graça). ⚠️ **O doc da biblioteca já estava corrigido; a CENA é que não foi** — quando
  um comportamento muda, o smoke que o demonstra é o **último** sítio a ser lembrado e o **primeiro**
  que o Enio lê (§0.8).
- **Integrar não é aprovar.** Smoke é do Enio; integrar e shipar só por ordem explícita dele (§0.7).
- ⚠️ **O TRACKER também é roteador — e mandar a narrativa para ele só REALOCOU a doença.** A regra
  «uma linha no §5» funcionou para o `CLAUDE.md` e criou o `HANDOFF_line_physics.md` a **710 KB**,
  77% do que o §5 chegou a ser. Medido 2026-08-18: **4,12 MB de história em 206 docs vivos** (42%),
  e o joelho está entre **80 e 110 KB** — a `DIRETRIZ.md` (86 KB) é o doc mais lido do repo e ainda
  cabe num `Read`; acima disso o `Read` **desaparece** e o acesso vira raspagem por shell (o tracker
  de física teve **1 `Read` para 407 comandos**, e **89% dele nunca entrou em contexto nenhum**).
  ⛔ Uma regra enterrada na linha 8.000 não é «difícil de achar»: ela **não é lida por ninguém**
  (667 marcadores `⚠️/⛔` lá dentro, 558 além da linha 2.000). O tracker recebe **uma linha por wave
  com link** para o handoff datado — que **já existe** em `docs/<Módulo>/handoffs/`, com índice
  cronológico. A história vai **verbatim** para `docs/archive/`, no formato de
  [`estado-2026-08-18/README.md`](docs/archive/estado-2026-08-18/README.md).
- ⚠️ **O que fica vivo tem de ser ENDEREÇÁVEL.** Medido: o agente não lê estes docs, ele os **navega** —
  o padrão de busca nº 1 em todos eles é reconstruir o sumário (`'^## '`), depois saltar para um
  endereço (`HR-5`, `Bug #17`, `W6.2`, `§1.5.9`) e ler ~70 linhas. É por isso que o `SKILL_Stack`
  (0% de história, consultado por `HR-N`) funciona e um diário numerado fora de ordem não.
- ⚠️ **As listas de `Smokes:` abaixo são NOMES de variável, não o comando.** Ao passar um smoke ao Enio, escreva-o **inteiro e copiável de uma vez**, com o caminho absoluto da árvore em que você trabalha (§0.8):
  ```
  cd /home/enio/Documentos/Projetos/PH2D && env PH2D_<NOME>=<n> cargo run -p ph2d-host-desktop --release
  ```
  ⚠️ **Modo L: o caminho é o da SUA worktree** (`.../PH2D/Worktrees/line-<módulo>`), não o do primário — o Enio roda de outro diretório, e sem o `cd` o comando falha ou testa a árvore errada.
- ⚠️ **Nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`** (medido: o mesmo binário deu 11,36 e
  5,50 ms para o mesmo passe). Gates de razão reprovam sob carga sem que uma linha de código tenha mudado.
  ⚠️⚠️ **FLAKES DE RECURSO SOB FAN-OUT — é uma FAMÍLIA, não uma lista: pare de as contar uma a uma.**
  A forma que se lê não é o nome, é o MECANISMO: um gate que mede um RECURSO partilhado — razão de
  dois relógios · contagem de alocações · o que vier — reprova sob 10–18 mil testes em paralelo e
  passa sozinho na máquina calma. O sinal de que é carga: o mesmo teste verde isolado (3–5 de 3–5),
  o diff sem uma linha no módulo dele — e num grupo, o **CONJUNTO de reprovadas MUDA entre corridas
  do mesmo binário** (um defeito de lógica reprova o mesmo caso sempre). ⇒ *re-rode sozinho ANTES de
  olhar para o seu commit*, e re-corra com `--no-fail-fast`: o nextest cancela no 1º ✗ e **esconde o
  resto da suíte** (uma corrida parou em 11.240 com 1.007 por correr).
  **Membros confirmados (2026-08-16..23):** `a_round_live_offset_costs_like_the_other_joins`
  ([`ph2d-vec-boolean`](crates/ph2d-vec-boolean/tests/offset_live_cost.rs) — o caso canónico: único
  ✗ de 15.323, no pico do fan-out, commit sem uma linha de produção) ·
  `the_cost_of_depth_is_linear_not_explosive` (Timeline) ·
  `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` ·
  `the_mask_stroke_cost_does_not_follow_the_canvas` ·
  `the_brush_snapshot_costs_the_same_on_a_canvas_sixteen_times_bigger` (as três em
  [`ph2d-tool-painter`](crates/ph2d-tool-painter/) — uma delas com doc a dizer-se *"imune à deriva
  da máquina"* por medir uma RAZÃO, que é precisamente o que o fan-out quebra) ·
  `only_the_lower_row_breathes_and_it_moves_with_the_playhead` (demos de áudio, «max delta 0») ·
  `an_abandoned_march_returns_nothing_and_returns_fast`
  ([`ph2d-field-render`](crates/ph2d-field-render/src/tests.rs) — mede um relógio de desistência) ·
  `emitter_sim_ceiling_probe` ([`ph2d-gpu-cook`](crates/ph2d-gpu-cook/tests/gpu_cpu_parity_sim.rs) —
  ⚠️ **`#[ignore]`, logo o CI nunca o correu**; os dois medidos em 2026-08-29, na subida do stack) ·
  a família `flip_smooth::resample_measurement::precisao::orcamento` — **3 testes** em
  [`flip_fit_budget_tests.rs`](shells/desktop/src/flip_fit_budget_tests.rs), medida 22/08 pela
  `line/3DModeling` e confirmada 23/08 pela `line/sculpt3d`, com a falha a MUDAR de teste entre
  corridas · `the_cost_of_sampling_a_path_is_flat_in_its_anchors` (Timeline) ·
  `the_region_refresh_is_bound_by_the_footprint_not_by_the_mesh`
  ([`ph2d-mesh`](crates/ph2d-mesh/tests/measure_normals.rs) — ⚠️ o doc-comment declara-se imune,
  *«o gate é a FORMA, não o relógio»*, e a forma é medida DIVIDINDO dois relógios: *um gate que se
  diz independente do relógio ainda o é, se o numerador e o denominador forem tempos*) ·
  `measure_brush_kernel` ([`ph2d-sculpt3d`](crates/ph2d-sculpt3d/tests/measure_brush_kernel.rs) —
  cara, 34 s sozinha, no pico do fan-out por construção) ·
  `packing_a_dense_scribble_is_bounded` ([`ph2d-flip-render`](crates/ph2d-flip-render/tests/pack_perf.rs)
  — medido 2026-09-01 numa corrida de **20 309** testes, verde **3 de 3** sozinho e com **zero
  linhas** do diff naquela crate) ·
  `measure_normals_parallel_speedup` ([`ph2d-mesh`](crates/ph2d-mesh/tests/measure_normals.rs) —
  o **segundo** membro deste ficheiro; mede a razão paralelo/série sobre uma esfera de 5 M
  triângulos)
  ⚠️⚠️ **E o «sozinho» da assinatura quer dizer *com a CARGA MEDIDA*, não *sem filtro*** — em
  2026-09-02 eu li `measure_normals_parallel_speedup` como **`3 de 3 VERMELHO` sozinho** e quase
  o arquivei como defeito real; a máquina estava a **load 82** (a suíte de 20 316 ainda a
  esvaziar). Com `load 3,2`: **3 de 3 verde**. *Imprima o `/proc/loadavg` AO LADO de cada
  corrida de confirmação, senão a régua que desmente a flake é a própria flake.* · e ⚠️ **as duas de ALOCAÇÃO**, espécie
  própria: `apply_from_doc_is_zero_alloc_steady_state` (ph2d-timeline) e
  `the_trusted_len_collect_allocates_once` (ph2d-audio-edit) — um contador de alocações parece
  imune a carga e não é: sob fan-out o alocador global reutiliza arenas de outra maneira ·
  `the_shape_match_is_linear_in_the_mesh`
  ([`ph2d-node-motion-soft-body`](crates/ph2d-node-motion-soft-body/) — confirmado 2026-09-01
  pelas TRÊS assinaturas: gate de razão · **zero linhas de diff** da linha acusada naquela crate ·
  5 de 5 verde sozinha · e o **conjunto de reprovadas MUDOU** entre duas corridas da mesma árvore,
  onde a corrida anterior tinha acusado outro teste, esse **real**).
  *Todo gate que compara duas medianas de um RECURSO é candidato, e a lista nunca estará completa.*
- ⚠️ **Gates de GPU são `#[ignore]`** e precisam de adapter — *skip gracioso não é verde*; e o `nextest` **cancela na
  primeira falha**: use `--no-fail-fast`, senão suítes inteiras nunca chegam a correr.

### §5.1 — Módulos

- **Motion Nodes** — dinâmica declarativa sobre `ph2d-nodegraph` ([ADR-0030..0039](docs/architecture/decisions/) +
  [SKILL §11.13](SKILL_Stack_PH2D_Definitiva.md)); avaliador `ph2d-eval-motion`, painéis `motion-graph`/`motion-params`.
  Semântica Houdini: as `force.*` são **Pure** e acumulam em `accel`; **UM** integrador aplica. Cook **GPU-resident por
  default** (`PH2D_GPU_COOK=0` volta à CPU, útil para bissecar). Editor completo (palette `A`, splice, bypass, clipboard,
  grupos, adapters). O catálogo é conferido nó a nó pelo [plano 89](docs/Motion%20Nodes/89_plano_conferencia_dos_nos.md)
  + as 17 folhas em [`89_conferencia/`](docs/Motion%20Nodes/89_conferencia/); o placar é **derivado** por
  [`placar_conferencia.py`](docs/Motion%20Nodes/ferramentas/placar_conferencia.py), nunca escrito à mão
  — ⚠️ *derivado, não auto-escrito*: a ferramenta **IMPRIME e sai vermelha**, `--write` **não existe**,
  e quem reconcilia a linha `Contagem` de cada folha é **quem roda**.
  ⚠️ **Todo canal novo é side-metadata no REGISTRY, nunca o contrato** (`NodeOp`/`OpResolver`/`NodeManifest` — §6).
  ⚠️ **`oscillator`/`noise`/`wiggle` têm uma porta `time` opcional** — desligada ⇒ `ctx.playhead()` byte-idêntico,
  ligada ⇒ **um relógio por ELEMENTO** (mecanismo: [handoff do FECHO](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_FECHO_2026-08-18.md)).
  ⚠️ **O `motion.noise` tem o ESPAÇO do campo** (`rotation` + `uniform`/`scale_y`) — e *escala maior num eixo
  = feição MENOR nele*; o **offset** e o **scale uniforme** NÃO são params, saem da composição e do próprio `scale`.
  ⚠️ **`substeps` é o RELÓGIO DO GRAFO, e a palavra tem dono:** só `sim.zone` e `motion.integrate` a declaram (censo
  `only_the_declared_clock_owners_offer_the_substeps_param`, teto **64** nos dois porque o ritmo é partilhado). Um
  sub-passo LOCAL de um solver folha usa outra chave — a `motion.verlet_rope` usa `solver_substeps` (rótulo "Substeps"),
  e enquanto usava a mesma o app corria as duas leis e a corda caía **4,8× menos** que os gates dela medem.
  **Aberto:** ⭐⭐⭐ **DOIS P2, ZERO P1 e ZERO P0** na conferência (26/08 — de 33; cinco folhas fecharam INTEIRAS nessa jornada: **06** animadores · **08** stream/utilidade · **13** sim stack · **15** value · **17** zero-param). Os dois que sobram são **um na folha 11** (fx raster) e **um na 15** (value), os dois com o preço medido e o desenho escrito; a **16** (rig) tem 18 células em ⏸️, que é outra coisa. ⚠️ **O placar é DERIVADO** por [`placar_conferencia.py`](docs/Motion%20Nodes/ferramentas/placar_conferencia.py) e **envelhece a cada wave: rode-o antes de citar o número** — esta linha já disse `68` com a conferência em `33`, e este `2` saiu da ferramenta em 2026-08-26, não da memória de ninguém. Cenas `=90..` até ao `MAX_DEMO_LEVEL` — ⛔ **conte-o no [roteador](shells/desktop/src/motion_state_demo_router.rs), nunca aqui** (esta nota já esteve parada em 97 com cenas até 102, e as `=98`..`=102` nunca tinham sido diagnosticadas): o gate `no_two_smoke_scenes_claim_the_same_level` mede o **piso**, não o teto, então uma cena **acima** do teto é simplesmente muda e duas com o **mesmo** número passam sem o acordar · ⏳ **[Bug #7](docs/Motion%20Nodes/BUGS_motion_nodes.md) ABERTO** (report do Enio, adiado por ele): na cena `=95` a fileira de mar de **4 ondas não mostra cristas diferentes** — a BOIA é um **passa-baixo** e apaga as camadas finas (excursão vertical `0,228` contra `0,377` da de 1 onda). A alavanca medida é o **calado**, ⛔ **não** a densidade, que reabre a armadilha do Bug #6 · ⚠️ **os KNOBS MORTOS foram caçados e os 19 curados** — [doc 90](docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md) tem a tabela verificada, os 8 pontos cegos da sonda e o que ficou de fora (a porta por-elemento lida no elemento 0, o `falloff` que o `motion.kaleidoscope` ignora, os knobs vivos e inalcançáveis pela UI); ⛔ **consulte-o antes de acusar um param de morto** · ⚠️ **os TETOS foram medidos e o bloco Z fechou 7 células** — [doc 91](docs/Motion%20Nodes/91_os_tetos_que_ninguem_mediu.md): 25 params passam a ter teto digitável DERIVADO do `step` do slider (o `f32` é o recurso), o `sim.spawn::rate` sobe de 60 para **15 360/s** (a lei, medida pela porta do produto), e o `MAX_DT` dos DOIS integradores desce de `0,1`/`0,05` para **`0,03`** — a `0,1` o laço fechado atirava uma grelha a **127×** o raio em que nascia. ⛔ O `motion.spring` fica fora (ele deriva 3 tetos do dele); ⏳ `motion.boids` e `motion.wave` seguem por medir · ⚠️ **a folha 11 (fx raster) FECHOU menos uma célula**: o `fx.drop_shadow` ganhou o MODO da sombra (a coluna por-linha já existia — faltava um param), o `fx.rgb_split` ganhou o EIXO e o RAIO LIMPO (⚠️ a rota por `falloff` modula o ALFA, não o deslocamento), e o `fx.glow` ganhou `Operation` (`Screen`, que é um par de fatores de mistura; ⛔ o `Multiply` do AE **escurece** e o halo compõe-se sem z, com gate a proibi-lo pelo nome), `Glow Based On` (`Alpha` faz uma silhueta PRETA acender pela cobertura) e a **rampa de cor** — ⚠️ cuja régua se corrigiu **duas** vezes: uma grelha uniforme não representa a esquina de uma parada, e num degrau o que encolhe com a densidade é a LARGURA da banda errada, não a altura. Cena **`=84`**. ⚠️ **E a folha 06 (animadores) fechou mais QUATRO células e meia, com uma pergunta só — *o animador não sabia a FORMA que o artista desenha, nem escrever os DOIS eixos*:** a onda `Custom` do `motion.oscillator` e a ease `Custom` do `motion.stagger` (uma lei — a forma vive num **text param**, e vai ao device por **LUT de 512**, ⛔ nunca por `applicable: false` a derrubar o nó para a CPU; custo medido **0,018% de um quadro**), o canal **`Position XY`** do `motion.wiggle` (FECHA a célula) e do `motion.noise` (metade — falta o *Use Layer as Seed*), e o **`align = Normal`** do `motion.path`. ⚠️ **A `natural_range` tem de responder por toda forma nova** (a `Custom` é unipolar e a conta bipolar entregaria metade da faixa com o piso ao centro — a armadilha do `Spike` reaberta), e ⚠️ **a régua dos DOIS eixos é a CORRELAÇÃO, com a barra longe de `1` e não colada em `0`**: o resíduo é ruído de amostragem (cai de `0,120` para `0,009` só ao afinar o campo), então uma barra apertada mediria o tamanho da grelha. Cena **`=85`** + **`PH2D_MOTION_NODE_PATH_SMOKE=3`**. ⭐ **E o primeiro dos QUATRO P1 fechou, com um NÓ NOVO: o `motion.bezier_warp`** (folha 04) — a fronteira CURVA que o *Bezier Warp* do AE tem e o Corner Pin não pode ter. 4 cantos + 8 tangentes, interior por **patch de Coons**; o default é a identidade **ao bit** (as tangentes nascem nos terços, onde a cúbica degenera no segmento *por identidade polinomial*). ⚠️ **Ele NÃO é um param do `motion.four_point_warp`, e há gate a prová-lo:** com arestas rectas o Coons é o mapa **bilinear**, que concorda com a homografia nos quatro cantos e **arqueia** as rectas interiores — uma projectividade preserva rectas por definição. ⚠️ **O teto de linhas do painel foi de 20 para 24** (a 3ª vez por medição, e a folga foi RETIRADA: cada slot multiplica com o `MAX_ENUM_OPTIONS`), e o nó desenha **1083 px num dock de 880** — o gate do dock passa a NOMEAR a excepção com a altura, ⛔ e um segundo nome ali significa que a resposta virou **secções recolhíveis** no painel, que hoje não existem. ⚠️ **E o smoke dela devolveu um defeito de alcance MUITO maior, já curado ([Bug #4](docs/Motion%20Nodes/BUGS_motion_nodes.md)): o `Multiply` do renderer não desobedecia à alfa — ele a INVERTIA** (α=0 pintava **preto**, subir a alfa **clareava**), porque uma fonte **pré-multiplicada** codifica *"não contribui"* como **zero**, que é o neutro de todo modo **menos** o `Multiply`, cujo neutro é `1`. Vale para **toda sprite do app** com `BlendMode::Multiply` em alfa parcial — ⚠️ um golden de outra linha que a contenha muda de valor, e a mudança **é a cura** (em α=1 nada se move, e era **só** α=1 que o gate media). ⏳ Fica a *dirt texture*, com o preço agora medido (a textura de uma sprite é uma de TRÊS coisas, e só uma é um rectângulo no atlas partilhado) · ⭐ **E o SEGUNDO P1 fechou — a folha 03 (simulação) está a zero: o `motion.soft_body` deixou de ser obrigatoriamente um retângulo.** Porta **`shape`** (a ÚLTIMA do manifesto, para `anchor_x`/`anchor_y`/`state` ficarem nos índices 0/1/2); ligada, a nuvem que chega é a forma de repouso; vazia, a malha autorada. ⚠️ **A wave não é «uma porta»: é apagar `rows`/`cols` de TRÊS respostas que nunca foram sobre a grelha** — *quem é pino* (era `i < cols`, hoje a **aresta de cima do repouso**), *qual é o contorno* que a pressão defende (era o passeio do anel, hoje o **casco**) e *como o corpo se divide em regiões* (eram bandas de índice, hoje de **coordenada**). ⚠️ **A malha autorada dá os MESMOS BITS**, e não por promessa: ela é o seu próprio fornecedor das três, e cada uma devolve a **sequência de índices** que o código percorria à mão — o anel e as regiões alimentam somas em `f32`. ⭐ **O casco com os COLINEARES MANTIDOS reproduz o anel da grelha índice a índice** (um casco estrito daria 4 cantos, mesma área em aritmética exacta e **outra** em `f32`); ⚠️ não ao bit — quem entra pela porta tem de ser **re-centrado**, e o centroide somado de uma malha já centrada não é zero (`−1,19e-7`) ⇒ **2 ULP**. ⭐⭐ **E o smoke achou um defeito de PRODUTO:** com o pino a valer *o `y` máximo a menos de um epsilon*, um DISCO fica preso pelo **ponto mais alto, um só**, e balança como pêndulo (envergadura +74% em 2 s) — a lei que fica é **meia FILEIRA**, derivada, que numa malha reduz a `0..cols`. ⛔ **Fronteira NOMEADA:** um repouso **côncavo** tem o *envelope* defendido em vez da área (mais fraco, nunca invertido). + as três secções do painel (**Mesh / Physics / Pin**). Cena **`=87`** · ⭐ **E o TERCEIRO P1 fechou — o `motion.trail` deixa de LEMBRAR e passa a RE-COZINHAR** ([ADR-0163](docs/architecture/decisions/0163-a-node-may-cook-its-own-input-at-n-instants-a-time-fan.md)). A célula estava certa sobre o nó e o **substrato** é que mudou: um ring contém o passado *porque passado é o que um ring é*, e o que faltava era um nó poder cozinhar a **PRÓPRIA entrada em N instantes** — hoje `TimeFans` no `ph2d-nodegraph`, **ponto de extensão append-only** (nenhuma assinatura mexida; ⛔ um argumento novo no `advance_or_scrub_scoped` custaria **29 sítios de chamada**). O nó ganha **`Source`** (`Remembered` = o ring, o default, **byte-idêntico** · `Resampled`) e **`Forward Steps`**. ⚠️ **A lei das gerações vive numa função só** (`echo_offsets`), com dois leitores — o construtor dos mapas e o `eval`, que dela tira a IDADE; a escada escrita duas vezes poria o desenho num instante e a cor noutro. ⭐ **Destrava as outras três do `SUPERAR:` S1 de uma vez**: `length` sem tecto de memória, espaçamento não-uniforme, e o **scrub exacto sem `CheckpointRing`**. ⭐⭐ **A cauda re-cozida é a mais CERTA das duas:** o ring promove a cabeça a fantasma **periodicamente**, logo carrega até `spacing−1` tiques de erro de fase o tempo todo. ⚠️ **É um MODO, nunca uma substituição** — uma simulação não é função de `t`, e um leque sobre uma sub-árvore com `pre` é **RECUSADO**. ⚠️ **E uma afirmação minha ENCOLHEU por mutação:** o `push_scope` não compra «fatias no mesmo instante partilham a faixa» (seis gates ficaram verdes sem ele) — compra o instante repetido **fora de ordem**. Cena **`=88`** · ⭐ **As folhas 09 (cor) e 10 (field) FECHARAM**, e as três células eram a mesma forma — *um número onde a pergunta tem dois lados*: o `soft_angular` do `field.radial_sweep` (um **multiplicador**, porque a **cerca declarada** do nó — *«adimensional, as duas bordas vivem em unidades diferentes»* — escolheu a forma da cura), o `clamp` do `field.remap` como **enum de 4 estados no param que já existia** (⛔ um `clamp_max` apendado mudaria o sentido de `Clamp = 0` em toda cena salva, e um gate que já existia disse-o), e a **interpolação por STOP** do `motion.color_ramp` (geração **`g4`** do formato, com `STOP_INTERP_GLOBAL = 255` — ⛔ **não** `RampInterp::COUNT`, que anda por cima do primeiro modo novo). ⚠️ **E o portão de fecho da workspace apanhou QUATRO vermelhos, um deles vermelho havia um bloco inteiro**: a rolagem do painel deixou de ser inerte quando o teto de linhas subiu, e a cura é ***uma banda, dois consumidores*** — o `HitIndex::push_clip` já existia e este painel não o chamava. ⚠️ **Três dos quatro caíram POR CAUSA da cura:** eles mediam o fundo do último hit-rect, que com a blindagem **satura na janela** — o retrato nomeado do `motion.bezier_warp` foi de `1083` para **969** sem uma linha de produto se mexer (114 px é a faixa do título). *Blindar o hit-index muda o que toda sonda mede.* · ⭐⭐⭐ **E o ÚLTIMO P1 FECHOU — a conferência não tem mais nenhum.** O `motion.emitter` ganha **`Emitter Motion`** (`Carry` = o penacho de sempre **ao bit**, e ⚠️ **não é um bug com nome bonito** — um efeito ANEXADO quer isto · `Leave` = a partícula fica onde nasceu · `Inherit` = e leva a velocidade da fonte) + **`Inherit Strength`** gateado ao modo que o lê. ⚠️ **A recusa DISSOLVEU porque o substrato mudou, e quem o mudou foi esta linha três blocos antes** (§0.0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*) — a terceira saída, **re-cozinhar** a origem, não existia quando a nota foi escrita. ⭐⭐ **E a medição achou a metade que vinha ANTES da célula:** o `P` do emissor é a posição de NASCIMENTO e era a origem de AGORA para toda partícula ⇒ **arrastar o emissor arrastava o penacho inteiro**. ⚠️ **A resolução da história é uma TAXA (240 Hz) e não uma contagem** — uma contagem repartida pela vida pioraria ao alongá-la —, com tecto **MEDIDO em 1024** (uma fatia custa 300-490 ns ⇒ 2,6% de um quadro; 2048 seria 5,5%, que é onde *fácil de usar* deixa de tolerar um knob opcional). ⛔ Os modos novos são **CPU-only**, com o bloqueador nomeado. ⚠️ **E o leque tinha um defeito que só uma FONTE revelava**: ele contava as fatias da PORTA, então um nó sem portas lia **zero** com o leque cheio (529 amostras ignoradas em silêncio) — e o gate não o via porque contava o trabalho FEITO em vez do RECEBIDO. Cena **`=89`** · ⭐⭐ **O `source.lsystem` EXISTE** (nó novo, ABOP completo: paramétrico · estocástico · sensível a contexto · tropismo · gerações fraccionárias) com **modo GUIADO por omissão** (a gramática é DERIVADA de sliders, e o `Mode` assa-a no texto ao converter), oito moldes que carregam o **próprio enquadramento** (ângulo/gerações/passo/espessura, os quatro **CONTADOS** — entre dois itens do mesmo selector o `maior/step` ia de `2,7` a `2 581,8`, **963×**) e o param **`Growth`** (0..1, `1` = no-op **ao bit**) que faz o arrasto crescer por igual, resolvendo `r^g = r + t·(r^G − r)`. ⚠️ **A âncora NÃO se conta da gramática** — duas regras com 5 módulos cada crescem `3,00×` as duas: *a razão é geométrica e mede-se*. ⚠️ **O parser falhava ABERTO** em dois dos três sub-campos de uma regra (uma condição que não compila **evaporava**: `n <= 6` dava `16 384` módulos contra `32` do `n < 6`, e um peso ilegível virava o neutro e ia DESENHAR) ⇒ uma política só, a do predecessor. ⚠️ **E o `motion.sub_uv` deixou de ser `Effect::Pure`** — ele lia `ctx.playhead()` e declarava-se sem tempo, logo estava **congelado desde que existe**; hoje é `Temporal` e **recozinha por quadro** (⛔ não é aditivo: o custo sob cena cheia não foi medido). Cenas **`=107`** (sujidade na lente) e **`=108`** (L-System) + **`PH2D_MOTION_OBJ_SMOKE=11`** (o ritmo) · ⭐⭐ **E o painel do L-System deixou de MENTIR, em duas metades (31/08).** **(a) Nenhum molde mostra um knob que a gramática dele não sabe LER** — o painel passa de **28 para 20** controlos (`Custom`, 23). ⚠️ **A régua é o PRODUTO** (quantas saídas distintas AO BIT o nó emite ao varrer o param pela faixa do próprio hint), e não um scanner de símbolos: o gate que existia lia `!` e `"` no texto e respondia por **2 knobs de 29**. Os três novos não têm símbolo — o `Step Scale` morre porque numa gramática **paramétrica** o comprimento viaja no módulo e o `Setup::step` nunca é lido, e o `Grow Length`/`Grow Angle` são **complementares por construção** (cada braço do `grows_by_refining()` lê **um** só, e o painel mostrava sempre os dois). ⚠️⚠️ **UMA leitura só FABRICA a lista de dívida: 12 acusados, 9 reais** — um param cujo sujeito outro cria mede-se morto no default, então cada célula é medida **duas** vezes (enquadramento + vizinhos acesos + geração **fraccionária**); dos 3 que caíram, esconder qualquer um apagava um controlo vivo. ⛔ O `seed` fica **isento com número** (vive no `Wild`, e os outros dois despertadores dele são **da shell**, que esta régua não vê). ⚠️ O **`Custom` nunca esconde nada**, e é LEI com gate: ali a gramática é a que o artista escreveu. **(b) Uma regra MALFORMADA diz porquê e diz a CURA** — `parse_rules_reporting` devolve as regras **e** as queixas do MESMO percurso (cada `return Err` onde estava um `continue`), porque um contador ao lado do parser seria o 2.º leitor do mesmo texto, defeito que este nó já pagou. ⛔ **Não há `ParamRow::Note`**: uma variante nova custava **104 sítios em 19 ficheiros** (incluindo o painel de outro módulo) e a `TextRow.problem` custou **6**. ⚠️ **Três gates porque são três defeitos**: a queixa nascer certa · chegar à row · chegar a **PIXEL** — sem o 3.º, um `match` no braço errado deixava os outros dois verdes e o artista sem aviso. ⚠️⚠️ **E uma recusa MINHA tinha a premissa errada no dia em que a escrevi** (95 §14.3): «esconder o `Tropism Angle` é inexprimível» — o `ParamGateAbove` shipou **nove dias antes**. *Antes de dizer que uma pergunta é inexprimível, procure o irmão do mecanismo que está a usar.* · ⛔ **ABERTOS da linha:** o `Grow Angle` de Bush/Weed (a lei existe e está medida — falta o veredito do dono; ⚠️ ele agora só é **pintado** em Bush/Weed/Koch/Dragon, que é exactamente a população da pergunta), ⚠️ **a Data Source NÃO está aberta — esta linha construiu-a** (`627f8b1aa`: as crates `ph2d-node-source-table`, `ph2d-node-value-table` e `ph2d-table`, com cena de demo e o campo de ficheiro ligado), e a nota dizia *«nunca começada»* com a API escrita pelo próprio autor dela, no mesmo ramo — *uma ausência afirmada sem olhar a API é um palpite com cara de medição* — ⭐ **a LEGENDA DO ALFABETO fechou (31/08): ela é um BALÃO ao passar o rato** sobre o *Axiom* ou as *Rules* — as quatro superfícies foram medidas e só ela serve (a coluna da row tem **~35 caracteres** e a legenda tem **~100**; nove linhas próprias estouravam o teto de linhas, que está em `31` de `33`). ⚠️⚠️ **E a facilidade JÁ EXISTIA:** a 1.ª medição desta wave concluiu *«a dica só é pintada na barra do topo»* a partir de **onde a função vive** em vez de **onde ela corre** — o `paint_hover_tooltip` corre depois de TODOS os painéis e o painel do grafo já o usava; *uma ausência afirmada pelo endereço do código é um palpite com cara de medição*. O texto sai do **nó** (`alphabet::ALPHABET`, com gate nos dois sentidos: tudo o que está na tabela é interpretado, e nada de fora dela é), e quem o coloca é a shell · ⭐⭐⭐ **E os «pequenos pulos» do crescimento têm causa e cura (31/08 — [doc 97](docs/Motion%20Nodes/97_os_pequenos_pulos_e_a_lei_do_recem_nascido.md)): a LEI DO RECÉM-NASCIDO.** ⛔⛔ **Nenhuma régua desta linha o via, e a cegueira é estrutural:** o `probe_flicker`/`probe_drift` medem um escalar de **TAMANHO**, que é exactamente o que o `build` **normaliza** ⇒ *a régua partilhava a lei do produto, e um espelho não acusa*; e a **imagem rasterizada** é cega à **sobreposição** (5 segmentos colineares sobre o caminho do pai tocam as mesmas células). A grandeza livre é a **TINTA** — a soma dos comprimentos desenhados. ⚠️ **O Houdini tem o MESMO defeito** (a doc dele: *«scales the geometry generated by the last substitution»* — a geração inteira pelo mesmo número), e a resposta é o **ABOP §6.2**: a eq. (6.3) manda o comprimento do pai **repartir-se** pelos filhos do eixo, e a **(6.11)** manda um ramo lateral acabado de formar nascer com comprimento **ZERO**. ⇒ uma **terceira fracção** (`Setup::newborn`), com **duas** metades porque a cadeia só responde a uma: *sou lateral?* sai da pilha do `[`/`]`, e *o meu produtor desenhava?* tem de viajar no módulo — um `F` nascido de um `F` e um nascido de um `X` leem-se **iguais** na cadeia. ⭐⭐ Medido pelo discriminador **salto/movimento** (afinar o passo `8×`): o `Bush` passa de `0,966×` — descontinuidade pura, **67 %** da tinta a aparecer num intervalo de `1e-3` — para **`0,125×`**, o valor **teórico** de movimento, e o salto absoluto cai **248×**. ⚠️ **Seis dos oito moldes saem BYTE-IDÊNTICOS**, e a partição é o mecanismo: ⛔ **não** é «ter colchetes» (os quatro que refinam têm), é o colchete **DESENHAR** — o `[J]` pousa uma marca. ⚠️ **Dois dos quatro gates nasceram de mutações SOBREVIVENTES**: a inércia dos outros três braços estava escrita em COMENTÁRIOS, e pôr `lat = 0` no braço do `Grow Angle` desligado **apaga os ramos laterais** com a suíte inteira verde. ⚠️⚠️ E a 1.ª redacção deles varria os oito moldes: **os dois reprovaram, cada um sobre a família do outro** — a família sai do **produto** (`probe_grows_by_refining`), nunca de uma lista escrita à mão · ⭐⭐⭐ **E o report SEGUINTE do mesmo dia — *"está mais suave mas não é perfeitamente linear"* — era OUTRA lei, com DUAS causas** (doc 97 §10): ⚠️ *suave é a ausência de degraus; linear é a derivada ser CONSTANTE*, e a segunda régua nunca tinha sido corrida. Medida, ela parte o corpus pela **posição** do desvio: quem **refina** acerta nas gerações inteiras (`−0,3 %`) e erra **dentro** delas (`+9,5 %`) — o remap resolvia `g` supondo `r^g` e o `build` entrega a **CORDA** entre `r^k` e `r^{k+1}`, que a `r = 3` está `+15,5 %` acima a meio; quem cresce pela **ponta** erra nas próprias inteiras (`+7,2 %` a `+21,3 %`) porque **ninguém os linearizava**. ⚠️⚠️ **E a nota que isentava a segunda família mediu a régua ERRADA** (*«o exponencial piora-os — o Tree foi de `0,5×` para `0,8×` de ondulação»*): ondulação é **suavidade**. *Uma recusa medida responde UMA pergunta, e esta respondeu à errada durante dois dias.* ⇒ **uma lei só: medir a ESCADA de tamanhos (`growth::size_ladder`) e invertê-la** — ⛔ nenhum modelo, e a densidade sai da família (`1` degrau por geração para quem refina, porque a normalização já força a corda a `±0,02 %`; **`3`** para quem cresce pela ponta). Desvio da recta: **`+21,33 % → +0,13 %`** (Wild), `+9,76 % → −0,01 %` (Koch), pior do corpus **`1,35 %`**. ⭐ **E é mais BARATO** que a lei que substituiu (Bush `1,124 → 0,085 ms`; ela derivava sempre até `g = 6`, a escada deriva até ao topo do arrasto) — o Dragon é a excepção (`0,226 → 0,628 ms`, `3,8 %` de um quadro, e só com o slider abaixo de `1`). ⚠️ **TRÊS defeitos que só a construção revelou:** a escada derivava com a semente **fixa** (numa gramática estocástica isso mede **outra planta** — Wild `−7,69 % → −0,29 %`) · o **`Sprig` tem um PATAMAR** que é da FIGURA (a ponta mais alta é um galho **lateral**, e o rebento tem de o ultrapassar: o `y` máximo fica preso `¼` de geração enquanto a tinta cresce; ⛔ colapsá-lo faria `20 %` da tinta aparecer de uma vez) · e o **`Step Scale` fora do neutro** quebrava a lei, **pré-existente** (`0,3737 → 0,0244`). ⚠️⚠️ **Duas mutações SOBREVIVERAM porque o corpus inteiro está no ponto NEUTRO daqueles params** — *um corpus no neutro de um knob não testa esse knob* —, e as duas fixturas que as mataram são construídas pelo **mecanismo** (uma planta com galhos de `0,9·s` contra um rebento de `0,8·s` nunca os ultrapassa) · ⛔⛔ **A AUDITORIA DE SEIS LENTES está em [doc 96](docs/Motion%20Nodes/96_auditoria_do_lsystem_2026-08-31.md), e a §8 dela é o CENSO — leia-o antes de pegar um item.** Dos 24 achados, **21 estão fechados** (14 na jornada de 31/08, mais sete em 01/09) e ⛔ **ficam DOIS abertos e UM recusado**: ⏳ arrastar o `Generations` numa planta grande passa o quadro (`17,88` contra `16,67 ms` a 78 124 ramos, e uma geração **fraccionária deriva a SEGUINTE**, `5×`) · ⛔ o **memo das fitas** (§2.4) é **recusa medida**: `0 de 237` quadros evitam uma reconstrução, e a cura óbvia — um período de graça na varredura — **reabre o `wgpu OOM` do quadro 19706** (o que mata é uma textura de GPU por `geometry_id`, não a tabela); *o defeito não é a varredura, é a CHAVE mudar a 60 Hz*. ⛔ **DUAS afirmações da auditoria foram REFUTADAS pela cura:** a §2.6 dissolveu sozinha (a `measure_ratio` saiu do caminho do produto quando a escada a substituiu) e a §3.5 tinha a **premissa de custo errada** — os *«>120 s»* não reproduzem (o laço quebra ao saturar: `g = 60 000` custa `3,93 ms` contra `4,86` a `32`), e o defeito real é a gramática que **nunca satura** (`437 ms` a `g = 10 000`, contra `13,91 µs` a `32`). *A cura prescrita estava certa; a razão escrita ao lado dela não.* ⭐⭐ **E TRÊS mutações sobreviventes nomearam costuras que nenhum gate tinha:** o `art` da folha deixou de sincronizar sem ninguém ver (⇒ um guarda `Synced`, e **esquecer a sincronização é erro de compilação**), o atlas deixou de contar as próprias escritas com a suíte do shell inteira verde (⇒ gate em `ph2d-render`), e o gate que prometia medir *«a queixa chega a PIXEL»* media a **linha reservada** — apagar a pintura deixava-o verde (⇒ o arnês ganhou um observador de GLIFOS; ⚠️ e a 1.ª régua nova contava `n_path_segments` e leu **`42` contra `42`**, porque o Vello encaminha texto por `draw_glyphs` e *nenhum glifo entra na contagem de caminhos*). ⚠️ **O fecho de 2026-09-01 está no [handoff](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-01.md)** — o §9 tem **sete** coisas que uma leitura rápida do diff entende ao contrário, o §10 as **cinco** premissas refutadas, e o §11 os **quatro** vermelhos que o portão do fecho apanhou (dois tectos de LOC, um tofu e um censo de elisão), todos curados por corte, nunca por isenção · ⚠️ **o `blend` é uma COLUNA por LINHA**
  (o *Echo Operator* do `motion.trail` e o *Flash Operator* do `motion.strobe`), com a escada
  `0 = o modo do sink` · `m+1 = o modo m` — ⛔ guardar o modo CRU faria a identidade de junção
  rebaixar linha alheia em silêncio; e **as DUAS rotas de lowering** a leem (a do device assava o
  tag como constante) · ✅ **as folhas 02, 05, 06, 08, 09, 10, 11, 14 e 17 FECHARAM** (0 P1 —
  a **14** com o **Trim** (`fx_trim`, na pilha de efeitos do `VecPath`), o **tracejado** e o `size` como
  **coluna** (geometria em raio 1); ⚠️ **um param de forma conduzido por FIO fazia a forma DESAPARECER** —
  a chave de conteúdo é pré-cook e o valor conduzido é do cook (`motion_externals::driven_params`) —,
  o traço **apagava o preenchimento**, e ⚠️ **um param de forma animado matava o app em ~5 min**
  (`wgpu OOM` no quadro 19706): o assador de tiles guardava **uma textura de GPU por `geometry_id`**
  e nunca a libertava — hoje o store e os tiles são **varridos por quadro**, e o assador só corre
  se houver `fx.glow` a consumir —
  a 05 numa wave só: `space`/`use_falloff_y`+`mask_channel`/`flip_rot`/`reindex`/`carry_rotation`, e
  ⚠️ **`falloff_y` é uma coluna NOVA** que só o `motion.falloff(Mask Channel)` escreve e só o
  `motion.scale(Separate Y Mask)` lê; a 09 com o `blend` do `motion.tint`, o `Offset`-como-CAMPO e o
  **kernel de GPU** do `motion.color_array` — ⚠️ a paleta viaja pela LUT com a CONTAGEM no slot 0 e o
  corpo **indexa** o buffer, ⛔ **nunca** o `_sample(t)`, que interpola entre duas cores que não têm
  nada entre si; as 08 e 10 com o `reindex` do `motion.cull`, o **`field.shape`** — nó novo, a
  GEOMETRIA como campo, CPU-only porque o canal de porta-template no device só existe emparelhado
  com um `StreamOp::SourceRows` —, o `key = Attribute` do `field.index_range` e o `curve_offset` do
  `field.remap`) ·
  o gate `#[ignore]`
  `the_ceiling_is_honoured_on_every_tick_including_the_turn` (cena `=53`) — ⛔ **não afrouxe a barra** · a composição
  sub-passos × `damping` da `motion.verlet_rope`, **medida e não curada de propósito** · ⛔ a faixa de barras do
  `value.pattern` foi **revertida por veredito de produto sem mecanismo nomeado**: uma 2ª tentativa começa perguntando
  *o que ficou pior*, não reconstruindo (a árvore sobrevive em `ae35416bd`).
  **Smokes:** `PH2D_GPU_COOK_DEMO=<n>` · `PH2D_SPLICE_SMOKE` · `PH2D_ADAPTER_SMOKE` · `PH2D_ATTR_SMOKE` ·
  `PH2D_PICKER_SMOKE` · `PH2D_GRADIENT_SMOKE` · `PH2D_AUTOFIX_SMOKE=1..8` · `PH2D_SHAPE_SMOKE` · `PH2D_LENS_SMOKE` ·
  `PH2D_MOTION_OBJ_SMOKE=<n>` (⚠️ o **`=9`** é o do sink — filtro, sub-UV e mídia mista; ⛔ **não** é um nível do
  `GPU_COOK_DEMO`, e a razão é medida: aqueles demos amostram um ladrilho **branco chapado**, sobre o qual os três são
  invisíveis) · `PH2D_MOTION_NODE_PATH_SMOKE=1|2|3|4` (⚠️ o `=2` é um **modo** de uma env que já existia,
  não um nível novo do roteador de `GPU_COOK_DEMO`: um nó que anda numa forma **desenhada** precisa do documento
  vetorial, que só aquele smoke encena).
  **Ler:** ⚠️ **[handoff de INTEGRAÇÃO de 03/09](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-03.md)** — o §6 tem **sete** coisas que uma leitura rápida do diff entende ao contrário (entre elas que a nota `drops` foi **revertida** e que `primary_input = 1` deixa a cena **vazia** de propósito), o §7 as **sete** premissas minhas que a medição derrubou, e o §8 os **nove** itens abertos · ⭐⭐⭐ **[estudo do Mini Cavalry, 02/09](docs/Motion%20Nodes/99_estudo_do_mini_cavalry_2026-09-02.md)** — o MVP de referência do Enio, medido dos dois lados. ⚠️ **O `visual-tokens.js` dele abre com *«Doc PH2D §6»*: o sistema visual dele é uma spec NOSSA que ele implementou e nós declarámos pela metade.** Empate no catálogo (**134 nós cada**), e ele faz isso em **22 584 LOC** contra 136 093 de UMA crate nossa. Os dois canais grátis: a **silhueta** (7 valores, 132 nós a declaram, o pintor **nunca a lê**) e a **cor do pino** (100% `Instances` ⇒ não distingue nada, contra os **7 tipos com cor E forma** dele) · ⛔⛔⛔ **[auditoria de PERFORMANCE do módulo, 01/09](docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)** — o device faz **4,19 M objectos em 3,85 ms** contra **195,9 ms da CPU** (`50,9×`), e **69,7% das 109 cenas do produto nunca lá chegam**: `67%` por uma escada que não nomeia recurso nenhum (*«F2+ territory»*), e **23 delas estão a um passo de composição** do caminho rápido. ⇒ *leia-a antes de propor qualquer optimização de nó — o tecto do módulo não está num kernel* · [índice do módulo](docs/Motion%20Nodes/README.md) · ⚠️ [`BUGS_motion_nodes.md`](docs/Motion%20Nodes/BUGS_motion_nodes.md)
  (**o único `BUGS_*` que esta seção não listava**, e foi lido 22×) · [as 17 folhas](docs/Motion%20Nodes/89_conferencia/README.md)
  — a **folha 03** é [`03_simulacao.md`](docs/Motion%20Nodes/89_conferencia/03_simulacao.md) · ⚠️ **o PLANO da próxima janela** está no [handoff de continuação](docs/Motion%20Nodes/handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-08-19.md) (os grupos seguintes, e as dez leis que a linha pagou) · [handoff de 29/08](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-08-29.md) (⚠️ o §7.4 tem **sete recusas medidas** do L-System — entre elas *inventar uma sintaxe amigável* para a gramática, que a tornaria incompatível com todo tutorial — e o §7.5 as **oito leis** que a linha pagou) · [handoffs](docs/Motion%20Nodes/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/motion-nodes.md)

- **Timeline** — dope-sheet + graph editor + transporte sobre `ph2d-core::Playhead` (`ph2d-panel-timeline` +
  `ph2d-timeline` + `ph2d-anim`): curvas com handles bézier e weighted tangents, roving keys, time remap, record,
  clips + **composição** ([ADR-0115](docs/architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md)),
  **nesting** ([ADR-0133](docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md)),
  duração explícita, motion path ([ADR-0141](docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md)), onion ([ADR-0142](docs/architecture/decisions/0142-timeline-onion-ghost-poses-non-destructive-pose-at.md)),
  retiming, extrapolação, **sinais** ([ADR-0143](docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md)) e expressões
  ([ADR-0144](docs/architecture/decisions/0144-timeline-expressions-frozen-ir-separate-post-composition-pass.md) / [0151](docs/architecture/decisions/0151-timeline-expressions-are-per-clip-so-a-strip-windows-them.md) / [0152](docs/architecture/decisions/0152-timeline-expressions-are-a-first-class-lane-source-that-fades.md)).
  ⚠️ O `TimelineDoc` viaja como **blob dentro do `ProjectFile` e carrega a própria versão** — é por isso que ele evolui
  sem mover o `PROJECT_SCHEMA`. ⚠️ A **AUTORIA** de expressões foi **retirada** (o motor ficou; registro em
  [doc 14](docs/Timeline/14_a_autoria_de_expressoes_foi_retirada.md)) — remover a feature **não** removeu o schema.
  **Aberto:** a expressão **PURA** (sem keys) extrapola a strip — ligar exige vínculo autorado (produto + provável
  `DOC_VERSION`) · **W4.T4** (o dock da timeline dentro do Motion) aguarda re-smoke: duas linhas discordaram, o código
  do `main` shipou com cap e gates, e a nota da rejeição saiu do `layout.rs` · o catálogo de receitas morreu, a pesquisa não.
  **Smokes:** `PH2D_NEST_SMOKE=1..3` · `PH2D_PATH_SMOKE=1|2` · `PH2D_ONION_SMOKE` · `PH2D_TIMESCALE_SMOKE` ·
  `PH2D_STAGGER_SMOKE` (⚠️ **Ctrl**+drag, o KDE rouba o Alt) · `PH2D_BUFFER_SMOKE` · `PH2D_EXTRAP_SMOKE` ·
  `PH2D_SIGNAL_SMOKE` · `PH2D_EXPR_BLEND_SMOKE` · `PH2D_MORPH_FADE_SMOKE`. ⚠️ `PH2D_EXPR_SMOKE` **morreu** com o card.
  ⚠️ **Flake conhecida e PRÉ-EXISTENTE:** `the_cost_of_depth_is_linear_not_explosive` é gate de RAZÃO sensível a carga —
  re-rode sozinho antes de suspeitar de um merge.
  **Ler:** [`docs/Timeline/`](docs/Timeline/) · [`BUGS_timeline.md`](docs/Timeline/BUGS_timeline.md) ·
  [handoffs](docs/Timeline/handoffs/README.md) · [história](docs/archive/estado-2026-08-18/timeline.md)

- **Áudio** — rack com **42 efeitos + 23 presets** e cadeia editável (`ph2d-audio-edit` + painéis
  `audio-editor`/`audio-mixer`), espectral ([ADR-0122](docs/architecture/decisions/0122-audio-spectral-fft-via-realfft.md)), export Ogg/Opus
  ([ADR-0113](docs/architecture/decisions/0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md) / [0116](docs/architecture/decisions/0116-audio-export-opus-isolated-unsafe-crate.md)), streaming de vozes
  ([ADR-0118](docs/architecture/decisions/0118-audio-streaming-voices-residency.md)) e **AI denoise nativo** via `tract`
  ([ADR-0123](docs/architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md), feature **`audio-ml` OFF por default**).
  ⚠️ **Invariante da rack:** todo efeito é **no-op byte-idêntico no ponto neutro** e o painel **se auto-popula** da
  tabela `KINDS` — efeito novo = variant + braços + row, **zero mudança de painel**.
  ⚠️ **Fronteiras duras, com gate:** nenhum **codec** e nenhum **runtime de ML** alcança o mixer RT.
  ⚠️ **HR-13 emendado** ([ADR-0117](docs/architecture/decisions/0117-audio-editor-memory-is-measured-not-declared.md)): *quem declara budget possui um gate que **MEDE*** (dhat).
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
  solver independente de ordem [ADR-0147](docs/architecture/decisions/0147-wet-paint-order-invariant-solver.md), row-parallel [ADR-0145](docs/architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md)).
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
  Todo path é **entidade ECS** com pose no `Transform` ([ADR-0110](docs/architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md) /
  [0111](docs/architecture/decisions/0111-vector-shapes-have-transforms-and-use-the-sprite-gizmo.md)); **14** modos — o `DrawMode` em
  [`params_mode.rs`](crates/ph2d-tool-vector/src/params_mode.rs) é a fonte, e ⚠️ **não há
  «Tesoura»**: ela e a Faca viraram **`Cut`**, e o **`Frame`** entrou depois sem ninguém recontar
  ([ADR-0112](docs/architecture/decisions/0112-vector-select-node-pen-are-three-tools.md)); **Live Corners** ([ADR-0121](docs/architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md)) e a costura
  **fonte ≠ cozido** que destravou os **Live Path Effects** ([ADR-0132](docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)); blend
  ([ADR-0128](docs/architecture/decisions/0128-vector-blend-object-live-virtual-steps-editable-spine.md)); largura viva ([ADR-0148](docs/architecture/decisions/0148-vector-live-width-profile-is-an-ecs-component-and-one-baker-serves-preview-and-apply.md)); **auto layout**
  via `taffy` atrás de uma crate-folha ([ADR-0153](docs/architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md)).
  Mais guias/régua, simetria como modo, **booleana viva com UM VERBO POR FORMA** (a receita lê-se na
  hierarquia — [27](docs/Vector%20Module/27_um_verbo_por_forma.md); ✅ **os quatro chips eram DOIS
  defeitos** e o report veio duas vezes: a fileira nunca era pintada (a regra pedia *"exactamente
  uma forma"* e tocar um filho seleciona o GRUPO — o sujeito é o **primário**, e a fileira agora
  NOMEIA a forma), e os chips estavam **mortos sob o ponteiro** (faltavam no `populate_ops`).
  ⚠️ **Um controlo nunca pintado e um morto sob o dedo dão o MESMO report** — e só o gesto REAL
  (`seam_bool.rs`) mede a segunda costura; `Click` sintético passa com o chip morto
  ([27 §8](docs/Vector%20Module/27_um_verbo_por_forma.md))), moldura, **tokens no documento**, **estados de UI + Smart
  Animate**, e a **árvore autorada como painel vivo** (o app escreve o código do painel).
  ⚠️ **A lei do ADR-0153:** *o passe publica **onde** as coisas ficam; ele não escreve **onde** elas estão* — nada no auto
  layout toca `Transform`, senão cada quadro de um resize vira um passo de undo.
  ⚠️ **Regra-mãe do pen:** *o que se vê/aponta/encaixa é MUNDO; o que o documento guarda é LOCAL.*
  **Aberto:** ⭐⭐ **A FILA, em ORDEM** (Enio 24/08 — índice em [doc 29](docs/Vector%20Module/29_fila_morph_state_machine_e_texture_pattern.md)):
  ✅ **(1) O INPUT MAP FECHOU** ([plano 30](docs/Vector%20Module/30_plano_input_map.md) W1–W7,
  [handoff](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_input_map_2026-08-24.md)) — acções **nomeadas** à la
  Godot, janela flutuante em *Settings > Input Map…*, *press-to-bind* de tecla **e** de comando, e o mapa viaja no `.ph2dproj`.
  ⭐ **A falha documentada do Godot está corrigida:** a `deadzone` dele tem **dois papéis** (ponto de disparo *e* offset de
  normalização — proposta #3709); aqui são **dois números** (`dead_zone` · `press_point`), os dois lêem o valor **CRU**, e
  `press_point >= dead_zone` é **coagido na porta**. ⛔ **LEI Nº 1 honrada:** a `InputTape` grava a **acção resolvida**, nunca a
  tecla — remapear não reescreve o passado nem parte o `physics_ecs_c9`. ⚠️ **Faltam os CONTEXTOS com prioridade** (o que o Unreal
  tem e o Godot não): **bloqueado** — só têm sentido com um modo de jogo, e o `shells/game`/R1 está adiado pelo Enio; a cura de
  hoje é uma **lista negra à mão** no [`player_input.rs`](shells/desktop/src/player_input.rs). ⏳ Falta também o *override*
  por-jogador em `~/.ph2d/` ·
  ✅ **(2) A MÁQUINA DE ESTADOS DO MORPH FECHOU** ([plano 32](docs/Vector%20Module/32_plano_maquina_de_estados_do_morph.md) W1–W11j,
  [handoff](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_morph_states_2026-08-26.md)) — um botão faz o conjunto,
  **as setas são virtuais** (o grafo é completo por construção e ninguém o desenha), **uma tecla por FORMA** (a lista é `n`, não
  `n(n-1)`), um **modo** de pré-visualização que toma o teclado, e o conjunto **anima dentro do sistema de States** que já existia.
  ⚠️ A lista de estados **são os FILHOS**, e daí saem de graça o arrastar-para-dentro, a ocultação e a reparação da animação quando
  uma forma sai. ⚠️ **Interromper um morfo a meio faz a forma SALTAR** — nomeado e **não curado**: uma pose carrega **uma** forma e
  o par vivo `(A, B, t)` não cabe nela; curá-lo é **modelo novo**, decisão do Enio. ⛔ *"funcional no runtime do jogo"* segue
  **bloqueado no `shells/game`/R1**, adiado por ele — o **mesmo** bloqueio dos contextos do Input Map, e **não** um preço desta
  feature (a lei já vive numa crate-folha e corre no modo de pré-visualização hoje). ⛔ As **setas desenhadas no canvas** e o
  arrasto forma→forma foram **RETIRADOS por decisão do Enio** (25/08) — o código existiu (W3a/W3b) e foi apagado; não reconstruir
  sem ler o [§5 do plano 32](docs/Vector%20Module/32_plano_maquina_de_estados_do_morph.md). ⚠️ E o `PROJECT_SCHEMA` subiu **sem
  degrau de migração, e está certo** (não há projetos gravados — decisão do Enio, 26/08; o bump fica porque o postcard é
  posicional, então **sem** ele um ficheiro velho seria lido errado **em silêncio** e com ele o load **recusa em voz alta**).
  Cena **`=75`** · diagnóstico `PH2D_MORPH_LOG=1` ·
  ✅ **(3) O TEXTURE PATTERN FECHOU, e virou DOIS modelos** (planos
  [33](docs/Vector%20Module/33_plano_texture_pattern.md) · [35](docs/Vector%20Module/35_plano_padrao_no_traco.md) ·
  [36](docs/Vector%20Module/36_plano_pincel_de_contorno.md), [handoff](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_pattern_brush_2026-08-29.md)):
  o `Paint` ganhou a 5.ª variante e o `StrokeSpec` trocou uma COR por uma TINTA (`StrokePaint`).
  ⭐⭐ **São DOIS modelos, e todo aplicativo sério entrega os dois** — *"a coisa precisa funcionar sem
  limitações"* (Enio, 28/08): **`Pattern`** é a TINTA que o contorno revela (normativo em SVG 2, e
  por isso um tracejado são **buracos** no papel de parede — ⛔ não é defeito), e **`Brush`** é a ARTE
  que PERCORRE a linha (o *Pattern Brush* do Illustrator), que escala com a largura e **reinicia em
  cada traço**. ⚠️ A arte de um pincel é uma **FORMA do documento** (gesto de duas mãos, ⛔ sem
  diálogo de ficheiro), e o motor já estava pago desde o plano 23 — faltava **endereçá-lo como
  propriedade do traço**. ⚠️ Tetos MEDIDOS: `MAX_DASHES = 4096` (o joelho está entre 4 103 traços a
  6,32 ms e 8 205 a 12,08, contra o *kill* de 8). ✅ **A W5 (as QUINAS) FECHOU** — *o avanço encaixa na
  **PEÇA***, e uma peça é delimitada por vão de tracejado **ou por quina**; ⛔ os 5 ladrilhos de quina
  autorados do Illustrator ficam FORA por MEDIÇÃO (os fóruns dele dizem que é impossível fazê-los
  casar à mão). ⚠️ **A metade A é FOUNDATIONAL** (`ph2d-arclen`: velocidade zero deixou de ser lida
  como direção ausente) e muda o desenho do **Zig Zag** e do **texto em caminho** — a segunda tinha
  gate a DEFENDER o comportamento antigo, e ele foi invertido com a tabela do custo dentro.
  ⭐⭐ **GRUPOS** existem na Hierarquia, e um grupo pode ser a arte de uma estampa **e** de um pincel.
  ⭐⭐ **O painel mostra o que serve à FERRAMENTA na mão**
  ([`section_scope.rs`](crates/ph2d-panel-vector/src/section_scope.rs) — **1 de 39** seções
  consultava o modo), e os campos de forma são da forma VIVA, ou da ARMADA, ou de ninguém.
  ⭐⭐⭐ **APARAR · SOLDAR · BALDE · EXPORTAR SVG** (planos 38–41): o **Trim** apara até à fronteira
  seguinte (o motor já vivia dentro do `fx_knot`); **soldar** parte linhas cruzadas em arcos que
  partilham o nó e escreve **UM caminho composto** no lugar do participante mais ao fundo (⚠️ com
  **tampa REDONDA** — cada arco é um sub-caminho e a kurbo põe *tampa*, nunca *junta*, na ponta de
  cada um, o que abria uma cunha por nó num traço largo); o **balde** dá a região que o clique aponta
  com a fronteira em **arcos de verdade** (`ph2d-vec-fill`, crate nova); e **`File > Export SVG…`** é
  o primeiro exportador do app que leva uma CURVA — os outros onze levam pixels, malhas ou som.
  ⚠️⚠️ **A tinta agarra-se às LINHAS, não a uma coordenada**: no clique gravam-se as **âncoras** da
  face (que pedaço de que contorno, em que fracção, de que lado) e cada quadro resolve-se do
  documento sozinho — *o mesmo desenho dá sempre as mesmas cores*. Partir, fundir e crescer caem de
  graça; ⛔ o modelo anterior comparava com o **quadro anterior** e derivava sem volta.
  ⏳ **ABERTO:** sete preenchimentos para **seis** faces nas fixturas do report (o sétimo com o miolo
  fora de toda a face) — ⛔ a cura *"esconder"* foi construída e **revertida por ordem do Enio**, e a
  saída que sobra é **avisar** · a lâmina (`cut_open`) recusa um composto, e quem faz o trabalho é o
  Trim · a caneta só continua a partir do **arco 0** de uma rede.
  ⚠️ **`PROJECT_SCHEMA` 103 → 108** e **`VEC_SCENE_SCHEMA` 17 → 18**, e os dois **contam-se** contra
  o main do dia: a `line/3DModeling` e a `line/components` escreveram **105 as duas**.
  [Handoff de 31/08](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_2026-08-31.md) ·
  [handoff de 02/09](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_2026-09-02.md).
  Cenas **`=76`** (a estampa) · **`=77`** (o pincel) · **`=78`** (as quinas) · **`=80`** (aparar) ·
  **`=81`** (soldar) · **`=82`** (o balde) ·
  ✅ **o `n`/folga do *tether* e o `DRAG_RATE_X = 50` NUNCA foram «feel sem medição» — a NOTA é que
  envelheceu** (conferido 24/08, mecanismo no [estudo §6.6](docs/Vector%20Module/Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md)):
  o `50` é o atalho de **último recurso** de uma caixa **sem intervalo nenhum**, e acima dele está a `ScrubLaw` · ⏸️ abrir/fechar painel **nunca** foi
  animado (ausência, não regressão; e **não** é o gêmeo da dobra) ·
  ✅ **a cascata (F5), o menu radial (E4) e o realce de proveniência (C2) FECHARAM** — o radial é
  **`P` segurado** (o gesto foi MEDIDO: o botão do meio é o pan, a caneta não entrega botão, e só
  9 letras estão livres sem modificador), e o realce vale para **todo objecto em todo modo**.
  ✅ **O som de UI (D1) FECHOU, e nasce DESLIGADO** (`~/.ph2d/prefs.txt`, `ui_sound=0`): quatro
  vozes sintetizadas, e a lei é *um som CONFIRMA o que a mão fez, nunca ANUNCIA o que o app
  decidiu* — o **hover é mudo**, e os sítios que armam são uma lista explícita com gate.
  ⛔ **Do estudo sobra só a D2 (partículas)** — e ⚠️ **ela NÃO é tamanho `G`**: medido 24/08, o motor que o estudo diz que
  *"já temos"* é o simulador **do documento** (grafo de nós + cook por quadro) e **o chrome não tem canal de partículas
  nenhum**; ligar um ao outro por uma faísca de encaixe é arquitectura errada — o que a D2 pede é um **burst local** no
  relógio de UI que o F0/F2 deixaram. ⚠️ E o estudo que os lista **mentiu sobre NOVE das próprias linhas**: meça um item
  daquela tabela antes de o pegar ([§6.6](docs/Vector%20Module/Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md)) ·
  ✅ **o hit-test já lê o mapa fundido — este item FECHOU** e a nota envelheceu aqui por semanas:
  `App::vec_live_drawn` é a `LiveGeometry` fundida e **6 sítios de pick** a consomem
  ([`input_dispatch.rs`](shells/desktop/src/input_dispatch.rs)). *Confira o código antes de abrir a
  wave que esta linha pedia* · ⏸️ **o tablet está MAL PRECIFICADO** («custa uma função»): o `winit`
  escreve `force: None` nos três backends e o Wayland não tem `zwp_tablet_v2` nenhum
  ([`vec_pencil_input.rs`](shells/desktop/src/vec_pencil_input.rs)) — o preço é outro · ⚠️ uma
  superfície `Plain` **nova** que leia `hover_live` sem estar no mapa nasce muda,
  e ⛔ **não alargue o censo a todo `Plain`** (revive a cerca do estudo §6.2, e há gate).
  **Smokes:** `PH2D_BUILD_SMOKE=<n>` (⚠️ várias cenas **imprimem o que montaram** — *se a linha não aparecer, PARE*) ·
  `PH2D_UI_MOTION_SMOKE=1..3`. Diagnóstico: `PH2D_BUILD_LOG=1`.
  ⚠️ **Preferência de utilizador fora do repo:** `~/.ph2d/prefs.txt` (`motion_character`, `reduced_motion`) — um
  `reduced_motion=1` esquecido **reprova smokes sobre produto correto**.
  ✅ **O traço já não vira CANETA ELÍPTICA sob Scale não-uniforme** (bug #27): no Vello o transform
  de um `stroke` multiplica a **caneta**, e com `sx ≠ sy` ela virava elipse. Decisão do Enio —
  *"quando engrossa, engrossa por igual nos dois eixos"* — ⇒ o fator é **`√|det|`** (a média
  geométrica: para escala uniforme é a própria escala, e é invariante à rotação).
  ⚠️ **A mesma lei já estava escrita DUAS vezes** nessa crate (o marquee, o hover outline) e mordeu
  na terceira: *uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é*
  ([`stroke_uniform.rs`](crates/ph2d-vec-render/src/stroke_uniform.rs)). ⚠️ O **caminho rápido é
  intocado**: afim conforme desenha byte a byte, e só o caso partido paga o clone por instância.
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
  **Smokes:** `PH2D_PHYSICS_SMOKE=<n>` (⚠️ **`=84` não existe, de propósito**; ⚠️ **a `=15` tem as
  paredes CINEMÁTICAS desde 30/08 e isso é load-bearing** — com paredes estáticas as duas bolas
  param no mesmo sítio desde a `rapier` 0.35, e a cena passa a ensinar o contrário do que diz).
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
  ⚠️ **A W7 (plano MLS) e a W9 (Mesh Filter) FECHARAM** — o filtro tem as **9** leis (Smooth · Relax · SurfaceSmooth ·
  Inflate · Scale · Sphere · Random · Enhance Details · Sharpen), e a contagem é do `FilterKind::ALL`, **que é a fonte**.
  *O picker é o que as torna alcançáveis*: enquanto a lei era derivada do verbo em mãos, três delas eram inexprimíveis
  por gesto nenhum — o verbo passa a **SEMEAR e nunca a mandar**. ⚠️ O `Sharpen` é o único filtro com **PRÉ-PASSE**, e a
  lei da referência **depende da taxa de polling** (ela não repõe a pose entre eventos) — a nossa entrega a força em
  sub-passos **determinísticos**.
  **Aberto:** ⛔ os defaults do `b-mode` e o *Draw Sharp* são **decisão de produto** (os do Blender moram num `.blend`
  **binário**) · ⛔ a lei do zoom (adotar a do SculptGL **diverge da referência**; o número está numa cerca executável) ·
  ⛔ as três divergências **declaradas** da referência, cada uma com gate defendendo a nossa posição · a outra metade da
  **W4**, **W10-W12**, e o **marching cubes** · ⛔ **duas perguntas do Enio, já devolvidas com a tabela**: se a lei da
  referência é o afiador que se quer (ela alisa detalhe fino e mal toca feição grande) e onde fica o teto do `Sharpen`
  (subir compra excursão real e paga **17,17 ms** por evento de ponteiro contra um quadro de 16,7) · ✅ **o undo do
  filtro FECHOU** (o registo pergunta um FATO, não o verbo em mãos — [handoff](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_UNDO_2026-08-19.md);
  e a rota do PONTEIRO **não é alcançável de um teste**, o `AppGfx` segura uma surface de janela real) · ✅ **o ALPHA
  vale para o FILTRO** (é mais um peso por-vértice, como a máscara — nos 9, o Sharpen incluído; ⚠️ o alpha é um campo
  **infinito**, então num gesto de malha inteira um carimbo **ladrilha**) · ⚠️ **VERMELHO PRÉ-EXISTENTE no `main`:**
  `the_two_lights_agree_where_the_form_turns_away` mede **0,3370** contra a barra de **0,01** — é `#[ignore]`, então o
  **CI nunca o rodou** — ✅ **CURADO, e o produto nunca esteve errado:** a sonda tirava a vista de `Shade::default()`, e
  o **`DEFAULT_MATCAP` virou `Some(0)`** por decisão de produto em 09/08 ⇒ ela passou a medir *matcap contra rig*, que o
  doc dela já chamava de *"outra luz inteira"*. A vista agora é escrita **por nome, os 7 campos**, então um termo novo é
  **erro de compilação** ali. Voltou a **0,0020** — o número exato do autor · **quad remesh — PIVOTOU**
  ([ADR-0162](docs/architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md),
  plano vivo [`docs/3D/quad-remesh/PLAN.md`](docs/3D/quad-remesh/PLAN.md)). O que existe hoje é o **porte fiel** do
  Instant Meshes (BSD) em `ph2d-quadflow`, alcançável pelo botão **`Quad Retopology`** (smoke **`=35`**) — e ele
  **NÃO é o padrão-ouro**: medido lado a lado com o oráculo `quadwild-bimdf` sobre um corpus de 10 malhas
  (`ph2d-quadbench/`), ele dá **65–83% de quads e 21–49% de vértices irregulares** contra **100% e 0,2%**. Uma grade
  numa esfera admite **oito** irregulares. ⚠️ **É a CLASSE do algoritmo, não afinação** — a família local nunca negocia
  globalmente. O caminho é a família global (campo cruzado + patches + quantização inteira), **clean-room dos papers**,
  com o binário GPL só como oráculo **fora da árvore**. ⚠️ **Três premissas do briefing do pivô foram REFUTADAS pela
  preparação** e mudam o preço: **não há pressão de caneta** (há gate a afirmá-lo), o Sculpt **não tem DAG** (é snapshot
  + undo), e a `Mesh` é **f32**. ⚠️ E o que o porte custou está registrado: **três vezes a RÉGUA se corrigiu antes do
  algoritmo**, e a fração de 96,4% que este §5 anunciava era **fabricada** (vinha de emparelhar triângulos e fechar
  n-gons com um nó no meio — operações que criam quads que a referência nunca emite).
  ⭐ **A CADEIA NOVA É O BOTÃO** (`Quad Retopology`, modo `Global` por omissão; `PH2D_RETOPO_LEGACY=1` volta ao
  local só para bissecar) — 100% de quads e irregulares de **39,7% para 0,5%** (o oráculo fica em 0,2%).
  Cinco crates: `ph2d-remesh-iso` (F1) · `ph2d-crossfield` (F2) · `ph2d-trace` (F3) · `ph2d-quantize` (F4, Bi-MDF
  com ótimo demonstrado) · `ph2d-quadfill` (F5). ⛔⛔ **VERMELHO PRÉ-EXISTENTE, medido 22/08: o F3 PERDE ASAS** —
  um toro saía com `χ = 2` **passando em TODAS as outras réguas** (100% quads, zero bordo, zero não-manifold).
  ✅ **CURADO em 22/08, e a cura era olhar:** a ponte que abre o patch-anel em disco **já estava traçada** — o
  `boundary_loops` é que exigia que a face do outro lado fosse de OUTRO patch, e ignorava uma parede interior.
  Honrá-la (`decompose_with(cut_open)`), sob a guarda de que a melhoria seja **estrita**, dá 3 096 quads e `χ = 0`
  com **zero** dissoluções. Duas cercas ficam: `LayoutError::GenusLost` (`V−E+F` do complexo contra o `χ` da peça)
  e `the_cleanup_never_worsens_the_topology`. ⭐⭐⭐ **O `ALIGN_WEIGHT` SHIPA a `0,03` desde 22/08**, e o número
  veio do **campo do oráculo** (`*.rosy`), não de varredura no fim da cadeia: a orelha vai de uma aresta de
  **57% da peça para 5,5%** e de **2 204 dobras para 171**; o relevo vai de 24,2° para **13,7°**. ⚠️ Preço: o
  toro 32×16 recusa (o traçado dá fronteira malformada com campo alinhado — gate `#[ignore]`
  `the_tracer_survives_the_aligned_field`), e **a rede da porta apanha-o** (cai para o só-suavidade, e o log diz).
  ⭐ **E a grade vinha 3× mais grossa do que se pedia:** a varredura que escolheu o custo do arco media dobras e
  pior-arco, e **nenhuma das duas vê uma grade uniformemente grossa** — o F4 devolvia **0,39×–0,98%** do que o F3
  pedia. Com a coluna nova (`Σquant/Σalvo`) a lei passou a ser o **`ScaleFactor`** da referência, e o toro 48×24
  foi de **3 096 para 6 221 quads**. ⚠️ A recusa que barrava essa lei (36,3% de dobras) **dissolveu** quando a
  parametrização por patch curou o grão — hoje ela dá 0,0%. ⛔ Fica aberta a **aresta máxima** em 12–24× o alvo.
  ⛔⛔⛔ **VERMELHO do produto, e é a QUARTA foto («péssimo», 22/08):** os nossos quads não são quadrados.
  ⚠️ **Todas as réguas geométricas desta linha mediam um EXTREMO GLOBAL** (`edge_max`, `edge_median`) e nenhuma
  olhava **um quad de cada vez** — um quad de `0,02 × 0,30` não move nenhuma das duas. A régua nova é
  `ph2d_quadfill::QuadShape` (aspecto · **enviesamento** · área), e a barra saiu do **oráculo medido com o mesmo
  código**: orelha `1,08 / 6° / ZERO` faces com canto pior que 60°, contra os nossos **`1,98 / 27° / 9 159`**.
  ⚠️ **A `sculpt_eared` não estava no corpus da bancada** — nove peças de que ninguém se queixou; foi
  acrescentada, e a orelha é a peça **mais limpa** do corpus dele. ⛔ **Três hipóteses e uma cura foram medidas e
  REFUTADAS** (o alisador dele · a forma dos nossos patches · «não seguimos o campo» · a relaxação por ajuste de
  quadrado, `SQUARE_ROUNDS = 0`): 16 rondas levam o aspecto máximo de 122,7 a 30,3 e o enviesamento mediano de
  27° para **26°**, pagando **3,4× as dobras**. ⭐⭐⭐ *Se mover vértices 16× não move a mediana, o defeito está na
  CONECTIVIDADE* — e a sonda `sculpt3d_field_follow` nomeia qual: medindo **as duas famílias** de linhas de grade
  contra o campo, a nossa 2.ª família não fica ortogonal à 1.ª (gancho `9,9° → 19,2°`; o oráculo `5,1° → 7,6°`).
  ⇒ **o interior de um patch tem de nascer de parametrização ALINHADA AO CAMPO**, não de interpolação da
  fronteira — e o `fill_with` nem **recebe** o campo. Gate vermelho com esse endereço:
  `the_quads_are_as_square_as_the_oracles`. ⚠️ **A régua mudou-se para o caminho do produto no mesmo dia**
  (`FillReport::shape` → `QuadRemeshReport::shape` → a linha do log diz o enviesamento), com dois gates verdes
  provados por mutação. ⛔ E a aresta de **56% da peça** (`the_ear_does_not_ship_an_edge_across_the_piece`)
  passava em TODAS as réguas. ⭐ A causa é **um** patch de perímetro **520% da diagonal** (o 2.º maior em 3 fixturas é 230%): as
  lascas dele forçam raios de leque `[1,39,1,1,39,1]`, e um raio `1` faz o sector ter **uma célula de fundo**.
  ⚠️ **Cinco saídas já foram MEDIDAS e fechadas** (mais segmentos na lasca · `dissolve` · desligar a ponte ·
  recuo do `uv` · faixa em vez de leque): ⇒ o problema **não é como o patch é preenchido, é o patch** — o traçado
  tem de o CORTAR, que é o mesmo trabalho da asa que a ponte só adiou.
  ⭐⭐⭐ **A JORNADA DE 23/08 FECHOU A CAÇA AO ENVIESAMENTO POR ELIMINAÇÃO MEDIDA** — campo (F2)
  **ilibado com número** (8 singularidades nossas contra 8 dele, o mínimo de Poincaré–Hopf; ~76% dos
  nossos cantos são INVENTADOS pelo F3) · **quatro achatamentos medidos, família fechada** (o conforme dá
  o PIOR: «mais conforme ⇒ mais quadrado» é **FALSO** — LSCM leva o erro conforme a `1,01` e o
  enviesamento a `28°`) · forma do domínio fechada · **menos patches é a ordem errada** (a poda empata a
  orelha com o oráculo e colapsa a geometria `18°→38°`; o oráculo usa MAIS leques e sai mais quadrado —
  o preenchimento tem de aguentar um patch grande ANTES de o traçado emitir poucos) · subdivisão local
  fechada · ponto fixo fechado (contrai por exactamente ½ por ronda e não endireita nada — *convergir e
  acertar são coisas diferentes*). ⇒ **A distorção nasce entre o DOMÍNIO e a SUPERFÍCIE**: mesmo com F3,
  marcação e domínio perfeitos, o preenchimento por patch fica em `15°` (o oráculo faz `6°`) — a mesma
  diferença de classe, local contra global, que motivou o pivô, um nível abaixo. **A obra seguinte é a
  EXTRACÇÃO, e em 2026-08-24 ela deixou de ser aposta: MEDIDA.** ⭐⭐⭐ Uma cadeia por extracção, com o **nosso**
  campo, dá **`5,1°`–`5,5°`** de enviesamento mediano e **`100%` de quads** — a classe do oráculo de
  produção (`4,8°`–`7,1°`), que ela **ultrapassa** numa das peças; contra os **`27°`** e **`9 159`** faces
  péssimas do nosso preenchimento por patch de hoje. ⭐⭐ **E o nosso CAMPO bate o do oráculo na malha DELE**
  (`5,1°` · `0` péssimas contra `7,4°` · `9`) ⇒ o F2 está ilibado por **resultado**.
  ⛔⛔ **FASE ZERO obrigatória, medida:** remalhar isotropicamente (`ph2d-remesh-iso`, F1) **antes** da
  cadeia. Sem ela a MESMA cadeia dá `10–12°` — *o dobro, sem uma linha de algoritmo mudar* (o corpus está
  guardado em quads, e triangular por leque injecta viés diagonal: aspecto p99 de **`23`** contra o `1,58`
  do nosso F1, que está **à altura do remalhador do oráculo**). ⛔ **Duas hipóteses foram REFUTADAS por
  medição** antes de se achar essa: o **curl** do nosso campo (ele é *mais* integrável que o de referência)
  e a **densidade** da grade (`0,7°` de `6,3°`). Decisão em
  [ADR-0167](docs/architecture/decisions/0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md):
  **clean-room dos papers; a biblioteca MPL-2.0 fica FORA, como oráculo** (⛔ não se porta — obrigaria a
  publicar arquivos no subsistema mais valioso, e a extração dela não termina na nossa escala). Espec
  funcional pronta e com 11 gates: [`SPEC_extracao_de_malha_quad.md`](docs/3D/cleanroom/SPEC_extracao_de_malha_quad.md)
  (⏳ falta a auditoria R-pré, que é condição de abrir a janela que implementa). ⭐ **A obra parte em duas e
  a extração já pode começar sozinha:** [`fixtures/`](docs/3D/cleanroom/fixtures/README.md) traz mapas de
  grade inteira de referência sobre a **nossa** malha e o **nosso** campo, verificados a `3,55e-15`, com
  costuras a sério (247 e 138) — ⛔ *uma peça sem costura aprovaria uma extração que ignorasse transições*.
  O que a `ph2d-gridmap` ainda deve é o arredondamento **uma-a-uma com re-solve** (resíduo `0,29` de célula).
  ⚠️ **A triagem de licença que abriu tudo isto** — e que achou ~460 notas do repo INTEIRO a citar fonte
  interno de alvo restrito — está em [`TRIAGEM`](docs/3D/cleanroom/TRIAGEM_quad_remesh.md) e
  [`ACHADO`](docs/3D/cleanroom/ACHADO_proveniencia_por_nome_interno.md).
  ⚠️ **Três correções ao que este §5 afirmava:** a leitura `29°/44°` da holonomia saía de uma grandeza
  **limitada a 45° por construção** que nunca testava o fecho de ciclo (a régua a sério dá **0** patches
  incombáveis na esfera lisa — a acusação ao F3 caiu) · «singularidade SEM CANTO = DENTRO de um patch»
  está **refutado** (a oitava está sobre um ARCO — defeito de traçado mais fraco, outra cura) · as duas
  réguas de valência mediam a população ERRADA (o caminho do rectângulo saía por `continue` antes da
  escrituração ⇒ balde sempre vazio, mediana `0,0` lida como «perfeito» — *um zero de «não medido» e um
  de «perfeito» são o mesmo byte*; a cura é `else` + contagens ao lado das medianas).
  ⚠️ **Nada disto muda o produto:** tudo o que a jornada construiu está **desligado com a tabela da
  rejeição ao lado** (`Interior::FromBoundary` · `LSCM_MAP`/`REGRADUATE`/`RECTANGLE_MAP`/
  `PROPORTIONAL_DOMAIN` = `false` · `SQUARE_ROUNDS = 0` · `prune::PRUNE_STEMS = false` · `ph2d-gridmap`
  sem consumidor no produto) — **a saída do botão `Quad Retopology` é byte-idêntica à de antes**.
  O mecanismo de cada passo, as tabelas e as recusas medidas: [`PLAN.md`](docs/3D/quad-remesh/PLAN.md)
  §4-tricies..§4-septemetquinquagies + o [handoff de 23/08](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_QUADREMESH_2026-08-23.md).
  ⭐⭐ **O ORÁCULO GRAVA AS FASES INTERMÉDIAS** (achado 22/08, `PLAN.md` §4-duotricies): em
  `ph2d-quadbench/ref/<peça>/` estão o **campo** dele (`*_rem.rosy`, uma direção por face) e a **decomposição**
  dele (`*_rem_p0.patch`, o dono de cada face) — as duas fases cujo código é GPL. ⇒ Comparar fase a fase **na
  malha dele** é legal (ler saída ≠ obra derivada), e é mais forte que ler código. ⛔ A bancada só comparava o
  resultado FINAL. ⛔ Outros buracos
  medidos: o F1 devolve o cubo **não-manifold** · a esfera **embaralhada** não fecha · **sem feature lines**.
  Tabelas por fase: `PLAN.md` §4-bis..§4-undetricies.
  ⭐⭐⭐ **E a EXTRACÇÃO existe** (2026-08-24, clean-room dos papers sob [ADR-0167](docs/architecture/decisions/0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md); espec, atestados e fixtures em [`docs/3D/cleanroom/`](docs/3D/cleanroom/)): `ph2d-quadextract` extrai a malha das **isolinhas inteiras** de um mapa de grade inteira, e `ph2d_gridmap::round` (G5) faz o **arredondamento misto-inteiro** que ela exige. ⭐ **Sem biblioteca de precisão múltipla e sem epsilon** — a truncagem numa grade *global* põe o domínio saneado em `i64` e a orientação vira um determinante `i128` exacto. Nos dois mapas de referência: **100 % de quads e `χ` preservado** (toro de género 1 e casca fechada), e uma peça com **bordo** resolvida **sem oráculo**. ⭐⭐ **É o caminho de OMISSÃO desde 2026-08-25, por ordem do Enio** (*«pode ligar o motor novo; o antigo não apresenta resultados úteis»*) — `PH2D_RETOPO_EXTRACT=0` volta ao de sempre. ⚠️ A lei «tudo o que é novo shipa desligado» valeu enquanto ele não fechava a casca; a obra A fechou-a. Quando ele shipou desligado, a medição era: na cadeia da casa a **forma** bate a barra do oráculo (aspecto `1,10`, enviesamento **`6,8°`** contra `4,8–7,1°`) mas a topologia não fecha (`χ = −5`) — ⚠️ **e a causa está medida a MONTANTE**: o solver contínuo do G3 **PESA** a costura (`SEAM_WEIGHT`) em vez de a eliminar, e a espec já nomeia a cura (§5.1: as costuras entram por **eliminação de variável**, não por penalização). ⛔⛔ **A 1ª redacção desta linha atribuía a causa às DOBRAS do mapa, e o R-pós refutou-a com a peça que importa:** na esfera fina o mapa tem **`0,0 %` de dobras** e as translações são **exactamente** inteiras, e a extracção ainda deixa `32` arestas de bordo. ⭐⭐ *Duas grandezas estavam a ser lidas como uma*: o G5 torna a costura **INTEIRA** (`shift_frac_max = 0`, e há gate) e **não** a torna **FECHADA** (`seam_max` vai de `0,23` a **`1,00`** — uma célula inteira de rasgo, e **não existe gate sobre ela**). ⚠️ O rasgo é **invariante** às duas constantes da escada (`1,0369`–`1,0897` nas 9 combinações, com o degrau barato a variar de `1,4 %` a `100 %`) ⇒ **não é afinação.** Mecanismo, a tabela e as 5 divergências deliberadas: [handoff de 24/08 §8-bis](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md). ⭐⭐⭐ **O smoke do Enio (24/08) VALIDOU a direcção** — *«o melhor resultado conseguido até agora»*, *«obedece razoavelmente o relevo»* — e nas **peças dele** a forma está **dentro da barra do oráculo** (enrugada `1,15` · `5,7°` · `4` faces péssimas), com os buracos a serem **19 de 2 041 células** (`0,9 %`). As duas queixas dele (buracos · vincos) são o **MESMO mecanismo em falta**, e a obra seguinte está **especificada**: [`SPEC_restricoes_por_eliminacao.md`](docs/3D/cleanroom/SPEC_restricoes_por_eliminacao.md) — *uma restrição linear entra ELIMINANDO uma variável, nunca como termo de energia*; a costura é uma, a aresta de feição é outra. ⛔ **A costura primeiro** (a feição é a mesma maquinaria e herdaria o rasgo). ⭐⭐⭐ **E a OBRA A FECHOU (24/08, `line/seamelim`): a casca fecha.** A costura deixou de ser penalizada — `94 %` das ligações eliminam uma variável e as que fecham ciclo entram todas **num sistema linear só** (⛔ em dois subsistemas o par realimenta-se e diverge: esfera a `NaN`, toro a `6,4e17`, e amortecer **não** cura). Medido A/B em 5 peças fechadas do corpus: resíduo de costura `1,00` → **`0,000`**, arestas de bordo `30`–`78` → **`0`**, `χ` `−4`..`−13` → **`+2`**, e **3–4× mais rápido**. ⚠️ Fica UMA regressão nomeada — as faces com canto `>60°` sobem (`4`→`10` na enrugada), e a cura publicada é o *local stiffening* (§5.4 do mesmo *paper*), **fora daquela wave de propósito**. Shipa dentro de `PH2D_RETOPO_EXTRACT` (que **desde 25/08 é o caminho de OMISSÃO** — esta cláusula dizia «já nasce desligado» e contradizia, na mesma linha, a frase que a corrigiu); `PH2D_GRIDMAP_WELD=0` bissecta. Mecanismo, as 5 recusas medidas e a pergunta devolvida sobre a barra do gate nº1 (`f64` da referência contra `f32` nosso): [handoff de 24/08](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md) + [auditoria](docs/3D/handoffs/AUDITORIA_line_seamelim_2026-08-24.md). ⭐⭐⭐ **E o ESPIRAL das linhas de grade tem causa medida (26/08): as separatrizes do F3 NÃO são linhas de grade do mapa** — `0`–`5 %` dos arcos concordam com o F4 e ~**todos** atravessam **~1 célula** nas 7 peças; ⚠️ o desvio já está **todo no G3 contínuo** (o arredondamento não o cria, e em 2 peças até o melhora), e ⛔ **dez hipóteses caíram por medição** (deriva de ciclo · reticulado das holonomias · cones a meia célula · pregar os cantos · culpar o G5 · a restrição como 2.ª camada — esta com `100 %` de recusas por os cantos **serem** as incógnitas livres da costura). ⇒ a cura entra **DENTRO** do `ClosureSystem`, plano em [`PLANO_arcos_no_sistema_dos_fechos.md`](docs/3D/quad-remesh/PLANO_arcos_no_sistema_dos_fechos.md); `PH2D_GRIDMAP_ARCLINE` nasce desligada. [Handoff](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-26.md) + [fecho](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-26_FECHO.md).
  ⛔⛔⛔ **E a BARRA DO ORÁCULO estava a ser lida a 1/9 da densidade dele (28/08):** a nossa medição corria com `370`–`576` quads e a saída dele tem `3 352`–`4 696`, e mais fina é mais fácil. À densidade dele a mesma cadeia **sem uma linha mudada** dá `3,8°`–`6,5°` — **dentro da barra desde 25/08**. ⇒ *a semana das amarras dos arcos perseguiu um buraco da RÉGUA* ([ACHADO](docs/3D/quad-remesh/ACHADO_o_acabamento_e_a_regua_da_densidade.md)). ⚠️ **Toda comparação com o oráculo passa a nomear a contagem de faces dos DOIS lados**, e o `piece_report` imprime-a (`PH2D_REF=<peça>.obj` acrescenta relevo e fidelidade contra a escultura). ⭐⭐⭐ **O que faltava era o ACABAMENTO, e ele é agora UMA PORTA** (`ph2d_quadfill::finish_extracted`, chamada pelo botão **e** pela `ph2d-quadchain`, que entregava a malha crua): Laplaciano como **ronda zero**, depois **ajuste de quadrado ALINHADO AO RELEVO** — o tamanho vem dos quatro pontos e a orientação roda para a direcção principal com peso = **a anisotropia crua** (numa esfera ela é `0` e a lei degenera **ao bit** no quadrado puro) — e a saída é a **MELHOR ronda**, aceite contra a ronda zero nas três colunas da barra. Medido na densidade do botão: aspecto `1,19→1,10`, enviesamento `7,8°→4,5°` (orelha `10,4°→3,8°`), zero faces péssimas novas; e à densidade do oráculo **batemos a saída alisada dele em todas as colunas de forma** em 3 das 4 peças. ⚠️ **Quatro correcções que só a PORTA impôs** (o limiar calibrado no programa errado · a catraca de Pareto · a paciência a medir «rondas desde a melhor» em vez de «enquanto nada foi aceite», o que quase apagou o maior ganho · e a suavização do campo de direções, **construída e não adoptada**) e a queda para a lei **cega** quando a alinhada não se mexe (1 das 8 células): [handoff de 28/08](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-28.md). ⛔ **E o «o oráculo refina por curvatura» do `ph2d-remesh-iso` está REFUTADO** — medida a malha dele por bandas de curvatura, o expoente é `−0,03`/`−0,01`/`+0,01` sobre uma faixa de `8×`: ele é **uniforme**, e densidade adaptativa (o *Adaptive Size* do ZBrush) é feature de produto, não o que nos separa dele.
  ⛔⛔⛔ **E a «PIORA SEVERA» de 28/08 (foto do Enio) NÃO foi a volta da performance** — as três mudanças dela foram revertidas uma a uma pela porta e as **cinco** configurações dão a MESMA malha. ⭐⭐⭐ **O defeito é que o botão não era IDEMPOTENTE:** o alvo saía do piso `0,75 × aresta_média(malha_da_cena)`, e depois de uma retopologia **a malha da cena É a saída** ⇒ o mesmo ponto do slider pedia quads cada vez maiores (medido, `Detail` parado em `0,50`: `19 786 → 1 747 → 520 → 281` quads, **−98,6 %** — a `281` uma ponta tem duas faces, que é literalmente *«pontas com baixa resolução»*). ⇒ a faixa passa a ser **CONTADA e ancorada na ÁREA** (`edge_for_detail_by_count`, `MIN_QUADS`..**`MAX_QUADS = 25 000`**), que é o que as três referências fazem (ZRemesher *Target Polygons Count* · QuadriFlow *Number of Faces* · Instant Meshes); depois da cura os mesmos três apertos dão `1 377 → 1 413 → 1 494` com a forma a **melhorar** (`2,8°`, `0` faces péssimas). ⚠️ A deriva que fica é a **ÁREA a crescer** com o alisamento, e a barra do gate é `|√(área₁/área₀) − 1|` — ⛔ não um número escolhido. ⚠️ **O `MAX_QUADS` tem DOIS recursos** (relógio `35 s` a `24 190`; e a **topologia**, que a `line/3DModeling` mediu a rebentar acima disso). ⭐⭐ **E «furo» contava METADE:** a chave da frente de `worse` via só o **bordo**, e o ficheiro que ele exportou tinha `19 786` quads impecáveis com **`2` arestas não-manifold** num ponto só — hoje é `open_edges = bordo + não-manifold`, que decide **e** arma a 3.ª tentativa. ⏳ **Aberto e nomeado:** o `Follow Curvature` **não tem consumidor** no motor de omissão (densidade uniforme sobre a superfície), e o motor **`Fast`** do menu devolve na peça dele `437` quads + **`150` não-quads** contra `1 494` e `100 %` — *a um clique do botão, com o nome que um artista alcança depois de ouvir que o bom é lento*. [Handoff §8-ter](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-28.md). ⭐⭐⭐ **E o 2.º report do mesmo dia («faces completamente soltas, buracos; as pontas finas perdem detalhe») partiu em DUAS respostas.** ⭐ **A face solta era uma ALMOFADA — o mesmo quadrado emitido duas vezes, um virado ao contrário** (`[68,69,70,71]` e `[71,70,69,68]`, a flutuar sobre uma ponta, arestas `3×` a mediana), e ⚠️ **nenhuma régua desta linha a via**: `χ` conta os dois lados de uma almofada e dá `2`, o bordo é zero, o não-manifold é zero, e a contagem de quads *sobe* — o que a apanha é **contar os componentes ligados** (`2`, de `23 628` e de `2`). A causa é uma **dobra do mapa**, e a extracção passa a descartar **os dois** lados (`mirrored_cells`, com o log a dizer *«N almofada(s) descartada(s)»*; `PH2D_EXTRACT_MIRROR=0` bissecta). ⚠️ **Preventiva na peça dele** — a partir do ficheiro que ele mandou a cadeia não reproduz a dobra. ⛔⛔⛔ **A densidade nas pontas foi CONSTRUÍDA, MEDIDA e NÃO ADOPTADA:** ele tem razão e há número (o expoente de `aresta ∼ curvatura^n` na saída dele é **`−0,003`** sobre uma faixa de `9,4×` — a grade é rigorosamente uniforme, e *nenhuma régua desta linha media isso*, todas olhavam a aresta **global**). ⭐⭐ O **substrato** fica: o passo do mapa deixou de ser um número e passou a ser um campo (`ph2d_gridmap::Step`), consumido no único sítio onde o passo entra no sistema. ⚠️ **E o §0.0 outra vez:** a 1.ª medição deu saída **byte-idêntica** nas três posições do knob porque o piso de `ScaleField::adaptive_with` é a cerca do motor **local** (`0,75 × aresta_média`) e, emprestada aqui, ela **apaga** a adaptação (campo `0,06728..0,06728`) — daí `adaptive_graded`. ⛔ **Com o campo a chegar de verdade (`4×`), a saída move-se `7 %`** (expoente `+0,047 → +0,014`) e paga `15 %` da contagem e o dobro das faces `>60°`: *o G3 resolve um mapa escalar cujo gradiente alvo com `h` variável **deixa de ser integrável**, e a projecção de mínimos quadrados fica com a parte integrável — a adaptação é projectada fora.* ⭐ A cura publicada é o factor de escala **conforme por construção** (`Δ log h` contra a curvatura de Gauss, `h = h₀·e^{−s}`), que é wave com espec própria. ⇒ o `Follow Curvature` **nasce em `0`** e o caminho de omissão é byte-idêntico. [Handoff §8-quater](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-28.md). ⭐⭐⭐ **E o report de 29/08 («o remesh amputou pontas», 6 fotos) fechou o CICLO de que os anteriores eram sintomas.** Correndo o botão sobre o `.obj` dele: a entrada é `χ = 2` limpa, **a fase zero devolve `χ = 6` com aresta não-manifold**, e a jusante vem `panicked at ph2d-gridmap/src/assembly.rs:193 — index out of bounds` ⇒ ⭐⭐ **é o estouro que este §5 diz estar «SEM ENDEREÇO desde 26/08»** (procurava-se em `solve.rs:336`, que já não existe). ⭐⭐⭐ **A causa é uma MORDIDA que se REALIMENTA:** a saída dele traz **`19` vértices de valência `2`**, todos em pontas finas, e **os `19` são doublets clássicos** (um vértice preso entre duas faces que partilham três cantos) — a extracção emite-os, o artista carrega outra vez, e a fase zero, que só sabe remalhar superfície, **rasga a topologia**. ⚠️ **As fixturas sintéticas NÃO reproduzem** (bola de espinhos varrida de `σ = 0,30` a `0,05`, todas `χ = 2`): *não é a espessura sozinha, é a espessura MAIS a mordida que já lá estava.* ⇒ três curas: a extracção **não emite** (`dissolve_doublets`), o botão **repara** o que a peça já traz (`ph2d_quadextract::repair_doublets` — ⛔ sem isto toda peça já gravada partiria o botão para sempre), e uma tentativa que **estoura perde** em vez de derrubar tudo (`catch_unwind`, a rede que a `ph2d-quadchain` já tinha e este caminho não). ⭐ A fusão é **exacta** (`V−1`, `E−2`, `F−1`, `χ` invariante) e a **ordem** sai do percurso da fronteira. Medido na peça dele: sem estouro, bordo `16 → 4` e `16 → 8`. ⚠️ **E dois gates da crate reprovaram primeiro com `χ = 14` contra `2`** — *doze órfãos, doze unidades*: a superfície estava certa e o **arquivo** não. ⏳ **ABERTO e nomeado:** os espinhos ainda rasgam (`4`–`8` arestas de bordo), e a cura de fundo é a **fase zero preservar a topologia que recebe** — wave em `ph2d-remesh-iso`, com esta reprodução como gate de partida. ⛔ E a hipótese «o F1 tem de seguir o alvo» foi **REFUTADA** (`PH2D_F1_TARGET=1`: `χ = 1`, `4` bordo, `123` dobras contra `χ = 2`, `0`, `21`). [Handoff §8-quinquies](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-28.md).
  ⏳⏳ **ABERTO — o report de 29/08 («buracos nas pontas, faces emboladas nas pontas», 2 fotos, 3 setas) é o que SOBRA depois da mordida, da almofada e da idempotência, e ⛔ NENHUMA RÉGUA O VÊ:** o `QuadShape` mede aspecto e enviesamento **medianos**, e três quads emaranhados na ponta de um espinho não movem uma mediana de milhares — *a próxima janela constrói a régua LOCAL antes de tocar em código*, que é a mesma lição que o `edge_max` global (cego ao quad de `0,02 × 0,30`) e o `χ` (cego à almofada) já cobraram. ⚠️ **Hipótese com endereço, não cura:** as duas queixas podem ser o mecanismo do **factor de escala conforme** — na ponta de uma agulha o F1 não consegue ser isotrópico e fino ao mesmo tempo sem a cerca que foi **medida e recusada** (§8-octies). ⛔⛔ **E QUATRO vermelhos de ÁRVORE ficaram invisíveis a TRÊS portões desta linha** (dois tectos de LOC, o `fmt` da árvore com 40 pontos, e o gate do shell que vive em `--test` e não em `--bins` — `cargo test --bins` corre 3 834 testes da crate certa e **não toca** em `shells/desktop/tests/`): curados por **cinco cortes por responsabilidade**, e a memória subiu de QUATRO para CINCO ocorrências em três linhas. [Handoff de 29/08](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-29.md).
  ⭐⭐⭐ **A RÉGUA LOCAL EXISTE (31/08), e a primeira coisa que ela acusou foi OUTRA RÉGUA** ([plano, Parte IV](docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md)): o `ALCANCE` tirava o centroide da **média dos vértices**, que é uma propriedade da *amostragem* — uma retopologia redistribui vértices por construção, então na escultura do dono ele derivava `0,2129` e lia **`−6,5 %`** onde a verdade era `−0,1 %`; ⛔ **e estava no caminho do produto** (a chave de amputação do selector), com o sinal ao contrário: quem **corta** a ponta perde vértices longe do corpo, o centroide afasta-se e o alcance medido **sobe**. Curado em `ph2d_quadfill::reach` (centroide de **área**: deriva `0,0037`, lê `+0,0 %`), com o controlo dentro do gate. ⚠️ **O relatório imprimia as DUAS réguas a discordar havia semanas** — *quando uma página imprime duas medidas da mesma grandeza e elas discordam, isso É o achado*. ⭐⭐ **E a régua nova é `ph2d_quadfill::tip_deviation`** — a distância da escultura à superfície da saída junto de cada ápice, **em unidades do quad pedido** (adimensional ⇒ densidades comparáveis): pontas sãs `p50 0,08`–`0,30` e **máximo `0,45`**, a partida `p50 1,15` · `p90 2,02`, com a barra em **`1` quad** que é o **chão da discretização** e não um número escolhido. ⛔ **É ponto→FACE**: com ponto→vértice a população sã lê `0,28`–`0,35` e *uma régua cujo valor «são» é feito do artefacto dela própria não tem onde pôr uma barra*. ⭐ **A varredura de sete densidades ilibou a fase zero em TODAS** (`0/4`, pior `−0,5 %`): a amputação que sobra é **100 % a jusante do F1**, é **sempre a mesma ponta**, e o corte vale **uma célula** (`0,91`·`0,93`·`0,69`·`0,36` quads de `Detail 0,50` a `0,70`) ⇒ **`Detail ≥ 0,70` dá `0` de `4` na peça dele**. ⛔⛔ **E DUAS curas foram construídas, medidas e REFUTADAS** — puxar o vértice mais avançado (aspecto `12,11`, enviesamento `85°`) e o *shrinkwrap* da região da ponta (malha destruída, aspecto `1,9·10⁸`, e a régua **mal se move**: `p90 2,03 → 1,55`) ⇒ ⭐⭐⭐ *mover vértices `76×` não cura, logo o que falta não são POSIÇÕES — são CÉLULAS*, e a cura de fundo é o **factor de escala conforme**, que é wave com espec própria. ⭐ **O selector trocou de régua e a troca muda uma escolha medida** (`Detail 0,40`, único sítio onde a chave chega a falar): o alcance escolhia a candidata com **`2`** pontas acima da barra contra a de **`1`**; depois da troca a pior ponta vai de `−19,6 %` para **`−9,9 %`**, pagando `7,48° → 9,33°` de enviesamento mediano — *a troca que a ordem desta função já declarava*.
  ⭐⭐⭐ **E A ORDEM DO DONO — *«o remesh deve funcionar perfeitamente em qualquer lugar»* — teve resposta parcial no mesmo dia** ([plano, Partes V–VII](docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md)): a MESMA escultura, só **deslocada na cena**, dava `0` de `4` pontas cortadas na origem e `2` de `4` em `x = 2` — que é **onde o importador a põe** (`IMPORT_SPAN` ancora fora da origem), logo era o caminho normal do artista que caía no lado mau. ⭐ **Duas causas curadas e gateadas:** a `SizingGrid` indexava por coordenada de **MUNDO** (dispersão `4,9 % → 0,0 %` na esfera; o controlo é o caminho sem graduação, que **já era** invariante) e a régua do `reach` tirava o centroide da **média dos vértices**, que é a AMOSTRAGEM. ⛔⛔ **E QUATRO curas de estabilidade foram construídas, MEDIDAS e REVERTIDAS** — canonicalizar a pose (destrói: `−77 %`, `−105 %`) · o campo **contínuo** (dispersão `4,4 → 1,4 %` e a ponta de `0/4` para `1/4`, com uma célula a `−40,8 %`) · construir o campo **uma vez da referência** (quebra a realimentação de facto — a `SizingGrid` era refeita **a cada ronda sobre a malha que o laço modifica** — e o produto piorou para `−48,6 %`) · **tirar outra carta** quando a ponta sai comida (ganha numa posição, perde noutra a `−46,6 %`). ⭐⭐⭐ *A leitura das quatro é uma só: **a nitidez da ponta vive exactamente do que se estava a suavizar** — o `min` duro das 27 células não é um acidente, é ele que alimenta a agulha.* ⇒ a cura que fica é a **OPOSTA**: dar **FOLGA** à ponta (`ADAPT_RATIO` **`16 → 64`**), medida em **8 células** — melhor ou igual em `7`, **quatro passam a ZERO pontas cortadas** (a de `x = 2` incluída) e o relógio não sobe, porque a renormalização faz a folga **mover** os quads em vez de os criar. ⚠️ **Não é invariância — é a ponta a deixar de depender do sorteio**; a `x = 16` as duas configurações continuam más e isso prova que a folga **mascara**. ⛔ E a régua nova teve o **ponto cego dela** curado no mesmo dia: uma ponta comida **por inteiro** não tem superfície junto do ápice, era **saltada**, e lia-se `0 de 3 acima da barra` sobre um espinho amputado em `−46,6 %`. ⛔⛔⛔ **E A FOLGA FOI REVERTIDA NO MESMO DIA POR VEREDITO DO DONO** (*«piorou; antes amputava uma ponta, agora amputou 2, e piorou até com Detail 1»*) — **a fixtura das 8 células estava errada desde o início.** Eu derivei a transformação do ficheiro **EXPORTADO** (que traz a pose assada) em vez de **ler** o [`sculpt3d_import::place`]: ele **RECENTRA a malha** e põe escala e posição numa `Pose` que só **desenha e exporta** ⇒ ⭐ **o botão vê SEMPRE a peça centrada e na escala original**, e as oito células mediram peças que ele nunca vê. Com a fixtura certa o report reproduz à letra (`Detail 1,00`: `0/4` contra `1/4`; `0,75`: `1/4` contra `2/4`). ⇒ ⭐⭐ **o fantasma da POSIÇÃO dissolve-se** — a sensibilidade é real como propriedade do código e o artista **nunca a atinge** —, as duas curas e as quatro recusas ficam por outros motivos, e a pergunta volta a ser a original e mais simples: **a `Detail 1` a peça sai limpa e a `0,75` perde UMA ponta a `−4,1 %`**. ⛔ **Toda medição futura corre sobre a peça RECENTRADA** — uma sonda que alimente o ficheiro cru mede outro programa, e isto mordeu **quatro** vezes em dois dias.
  ⭐⭐⭐ **E EM 30/08 A CURA APARECEU — ela já era PRODUZIDA e o desempate deitava-a fora.** O registo por candidata (construído nessa janela, porque *um knob descartado e um knob fraco liam-se exactamente igual*) mostra que a 3.ª tentativa do botão — a de **linhas de feição** — empata com a escolhida em furos, peças, gravatas e faces `>60°` e entrega a ponta **`1,8×` mais fina** (`ENTREGA 0,851` contra `1,502`, alvo `0,59` derivado da retopologia que o Enio aprovou). Ela perdia **só** no enviesamento mediano, a única das grandezas em jogo que ele nunca nomeou. ⇒ o [`worse`](shells/desktop/src/sculpt3d_retopo_rulers.rs) ganha a **chave da ponta**, entre `>60°` e o enviesamento (`PH2D_RETOPO_TIPKEY=0` bissecta). ⚠️ **Ela só decide quando TODAS as chaves de defeito empatam** — não compra uma ponta fina com um furo, e ⛔ na `sculpt_t003` (candidata a `0,881` com `8` bordo contra `4`) **não dispara**, que é o desenho. ⚠️ **A troca está medida e não é livre:** enviesamento p50 `2,9° → 5,6°` (a barra do oráculo é `4,8–7,1°`) e uma face a `165°` de torção — contra as `46` faces torcidas que a mesma cadeia já shipa noutra peça. ⛔⛔ **E o PAR `PH2D_ISO_ADAPT=1 PH2D_ADAPT=1` atinge o alvo (`0,536`) e continua fora**: as duas metades tinham sido medidas **sozinhas** e recusadas, ninguém correra a célula `(1,1)`, e ela rasga a topologia (`χ = −5`, `36` bordo). [Handoff de 30/08](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-30.md) + [plano](docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md).
  ⭐⭐⭐ **E EM 31/08 A AMPUTAÇÃO («vamos corrigir as pontas») TEM ENDEREÇO: a razão `ALVO/F1 = 0,34×`, que o relatório já imprimia.** A fase zero entrega uma malha de trabalho **três vezes mais grossa** que o quad que o slider pede, e corta `3` das `4` pontas **antes** de a cadeia começar. ⚠️ **As duas metades do botão são ancoradas em coisas diferentes** — o F1 em `ALPHA × diagonal da caixa`, o quad em `área/contagem` — e isso é **auto-derrotante**: *um espinho longo infla a diagonal, logo uma peça com espinhos recebe uma malha de trabalho mais grossa POR TER espinhos.* ⭐⭐ A cura é a `SizingGrid` deixar de **inflar** e passar a **redistribuir** (`√(N_previsto/N_pedido)`, medido pela própria grelha): o orçamento passa de `8,3×` para **`+7 %`..`+15 %`** e a avaria de topologia que a mantinha desligada **desaparece**. Medido em 5 peças pela régua **por ponta** (⛔ o ALCANCE é um extremo global e esconde uma ponta cortada atrás de outra): `_base_sculpt` pior corte `−41,2 % → −8,4 %` e alcance `−41,8 % → −11,1 %`; `sculpt_antes` `3/6 → 1/6` cortadas com bordo `4 → 0`; a agulha sintética mais fina passa a **fechar** (`χ 1 → 2`, bordo `4 → 0`). ⇒ a porta nasce **LIGADA** (`PH2D_ISO_ADAPT=0` desliga). ⛔⛔ **E o portão apanhou-a a escapar para o motor LEGADO:** a 1.ª versão lia a env **dentro** do remalhador, logo alcançava todos os chamadores e o `the_ear_does_not_ship_an_edge_across_the_piece` reprovou — *o doc do próprio `remesh_with` já escrevia a lei violada («uma bandeira global é uma corrida escrita à mão»)*; hoje é **porta separada** e quem grada é quem chama. ⛔ Uma **banda simétrica** foi construída, medida e **REVERTIDA** (a mutação que a apagava sobreviveu aos dois gates: com a renormalização por cima o tecto deixa de ser observável). ⏳ **ABERTO:** a amputação que sobra nasce **a jusante do F1** (com a fase zero perfeita a saída ainda corta `2` de `4`), e parte dela é **resolução** — a agulha tem raio local `0,037` e o quad pedido mede `0,0399`.
  ⭐⭐⭐ **E o report de 31/08 («uma apenas foi amputada — a menos densa em faces») era um DIAGNÓSTICO: o teto da graduação era EMPRESTADO.** O `ADAPT_RATIO = 4` vinha, com o doc a dizê-lo, da cerca da **grade de quads** (*«duas células cujas escalas diferem por mais do que isto deixam de ter aresta comum»*) — e o consumidor aqui é um **remalhador de triângulos**, que não a tem. ⇒ §0.0: *um limite legítimo diz de que recurso ele é, e este dizia de um recurso de outro subsistema.* ⭐⭐ A aritmética fecha com a observação dele: com o piso em `alvo/4 ≈ 0,026`, **uma agulha mais fina SATURA** e recebe exactamente a mesma grelha que uma mais grossa — medido, `8,5 %` a `14,3 %` dos vértices (coluna nova `NO PISO`, `PH2D_ISO_LOG=1`). Com **`16`** a saturação some (`0,2 %`) e as pontas cortadas ficam **melhores ou iguais nas QUATRO peças** (`_base_sculpt` `1/4 → 0/4`, pior `−5,9 % → −0,4 %`; a agulha sintética pior `−36,4 % → −11,2 %`), com topologia **idêntica** nas oito células. ⛔ **Subi-lo só é barato porque a renormalização já lá está.** ⚠️⚠️ **E a célula `8` devolveu o SEGUNDO defeito:** com a fase zero **perfeita**, a saída cortou a ponta mais longa em **`−43 %`** — a corrida trocou de vencedora e **o `worse` não tinha chave de amputação**. ⇒ ela existe agora, entre as gravatas e o `>60°`, **sem referência** (as duas candidatas vêm da mesma entrada) e com a banda **medida e do repo** (`TIP_CUT_PCT = −2 %`); ⛔ nunca à frente dos furos. ⚠️ **O que ela NÃO faz:** maximiza o ALCANCE, logo protege a ponta **mais longa** e não as outras — *um extremo global não conta quantas*, e a chave por-ponta pede a malha de entrada dentro do `worse` (⏳ nomeado; a régua é a `ph2d_quadfill::tip_survival`).
  ⭐⭐⭐ **E EM 02/09 A RÉGUA PASSOU A CONCORDAR COM O OLHO DO DONO — calibrada pela PRIMEIRA vez no lado que ele APROVOU** ([handoff](docs/3D/handoffs/HANDOFF_line_quadextract_A_REGUA_QUE_CONCORDA_COM_O_OLHO_2026-09-02.md) · [plano, Parte XII](docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md)): a jornada de 01/09 fora reprovada quatro vezes (*«absolutamente nenhuma melhoria»*) sobre réguas que nunca tinham sido corridas na retopologia que ele aprovou (`Sculpt_Blender.obj` — ⛔ que é de `sculpt_antes.obj`, **não** de `_base_sculpt.obj`). Corridas, dois defeitos de CALIBRAÇÃO: o **piso do ápice** (`0,55` do raio, corte em `12`) escondia as pontas da foto (`3138` a `0,47`, `10230` a `0,51` — *a régua do produto media 4 pontas e a da foto não era nenhuma delas*), e a **barra da grade** (`1,5`) saía do vazio entre as **nossas** pontas boas e más (a aprovada entrega `≤ 0,79`; hoje `1,0`). ⭐ O ápice passa a medir-se **sozinho** (`TIP_GAP_MAX = 0,5`, meia célula — a `p50` da vizinhança lia `0,84` numa agulha com o bico a `1,11` da superfície). ⚠️ **Baixar o piso sem um filtro de FORMA acusaria a malha aprovada** — as bossas lêem grade `1,0`–`1,47` **nela** — e o filtro é o cone na **pior faixa de `2 h` entre `3` e `9 h`** (`CONE_MAX = 1,0`): um botão de cinco células que o QRemeshify deixa grosso com aprovação dele (`7328`) salta para o corpo a `5 h`, e uma faixa só junto do topo lia-o `0,95`. ⭐ Portão sobre as **fixturas dos DOIS lados** (`ph2d-quadfill/tests/pontas_do_dono.rs`, 5 `.obj.gz` com proveniência): GREEN na aprovada, RED nas duas reprovadas, margens exigidas. ⚠️ **A saída do botão NÃO muda — o veredito muda:** a `Detail 1,00` na peça dele a mesma malha lê RED no `3138` (grade `1,36`) e nenhuma das nove candidatas cura sem amputar outra; em `sculpt_antes` **todas as nove** comem o espinho principal. ⇒ *o selector não tem onde escolher; a obra seguinte é de substrato.* ⭐⭐ **E o MECANISMO está medido (plano §101–§103): a grade termina onde as singularidades param** — na malha aprovada todo espinho fecha com um pólo `+1` (quatro valência-`3` a `≤ 2 h`), no nosso o `3138` tem três `+¼` a `≤ 1,9 h` **no campo** e a saída deixa a terceira a `6,1 h`, e a grade do bico é monótona nessa profundidade. Reforçar o alinhamento **na calota** (`PH2D_TIP_ALIGN=5`, instrumento) deixa as cinco réguas verdes no campo **e a extracção não fecha a calota** (um laço de `14` arestas em `0,6 h` no bico da agulha, costuras a zero): a calota precisa de `≥ 2` células resolvidas e a fase zero entrega `1,3–2,3 ×` o alvo nos bicos ⇒ a wave é **uma calota por espinho na fase zero**, local (⛔ `PH2D_F1_TARGET=1` já foi refutado). `PH2D_SING_DUMP`/`PH2D_CANDIDATE_DUMP` gravam o campo e cada candidata. ⛔⛔⛔ **E o report de 03/09 (foto) expôs que a FIXTURA da linha era OUTRA REALIZAÇÃO** (plano §104): a peça recentrada em Python dava `21 747` quads com a ponta maior fina; recentrada pela porta do importador (`PH2D_RECENTER=1`, novo na sonda) dá `20 658` — **a malha dele ao bit**, com a ponta maior cortada `7 h` (a régua lê RED onde ele aponta). *Toda medição corre com `PH2D_RECENTER=1` sobre o ficheiro CRU.* ⛔ E o destino da ponta maior é **sorteado nos últimos bits** — cinco realizações da mesma escultura nos mesmos knobs, cinco vereditos —, e o «segundo sorteio» como rede foi medido e recusado (na `sculpt_antes` o espinho principal cai nas `18` candidatas de dois sorteios). ⛔ Recusas medidas (`1,5` · piso `0,55` · cone sem `h` · razão de área · faixa `2,5–4,5 h` · unidade = mediana no produto · alinhamento como discriminador · Dijkstra por pilha, `71 s → 0,5 s` · o reforço da calota como cura · `k ≥ 10` · o segundo sorteio): plano §99, §102 e §104.
  ⭐⭐⭐ **E A PONTA DEIXOU DE SER AMPUTADA na realização do PRÓPRIO dono (03/09):** `0` de `5` pontas cortadas contra `1` de `5`, e a grade no bico de **`3,51` para `0,79`** (barra `1,0`), com `χ = 2`, zero bordo e zero não-manifold. São **DUAS** metades e nenhuma basta sozinha: a fase zero ganha uma **calota resolvida** por espinho afiado ([`ph2d_remesh_iso::Cap`](crates/ph2d-remesh-iso/src/sizing.rs) — ela entregava o bico a `2,22 ×` o passo da grade, e o pólo `+1` precisa de `≥ 2` células), o que faz a cadeia **produzir** a 1.ª candidata verde nas duas réguas de ponta desta peça; e o acabamento passa a **desfazer gravatas** ([`untangle_bowties`](crates/ph2d-quadfill/src/untangle.rs)), porque essa candidata perdia por **UMA** face dobrada — a `5,7` células do bico, no flanco — na **3.ª** chave do selector, que vem antes da amputação. ⛔ **Reordenar as chaves está fora** (a ordem foi medida em 30/08 sobre um report do dono): a lei é *produzir a candidata que tem as duas coisas*. ⚠️ **O log da decisão imprimia `n−1` das `n` chaves** — `bordo` onde o selector lê `bordo + não-manifold`, e nunca as ilhas nem as gravatas; foram precisas **três** corridas para achar o que uma coluna diria à primeira. ⛔ **Afinar MAIS é pior e está medido** (`0,75` e `0,5` põem a fase zero verde e a jusante devolve `7`–`48` arestas de bordo). ⏳ **ABERTO:** a agulha da `sculpt_antes` melhora e **não** fecha (`1/4`, gap `3,00 → 2,57`), a 2.ª realização ainda tem `2` de `5` acima da barra da grade, e o critério do §104 (mesmo veredito nas **cinco** realizações) tem **duas** medidas. ⭐⭐ **E o 2.º report do mesmo dia (uma fenda no flanco de uma ponta) fechou a família:** a malha estava **perfeita** na topologia (`χ = 2`, zero bordo, zero não-manifold) e a fenda eram **cinco dobras de 180° no mesmo ponto** — a **outra espécie** da face do avesso, que o selector nunca contou. ⭐ **A régua que separa é o TAMANHO DO GRUPO** (a retopologia que o dono aprovou tem `3` dobras isoladas — vincos reais; a dele tinha `5` juntas), e ⛔ nem a contagem de faces minúsculas nem o salto de tamanho separam (a aprovada é **pior** nos dois). ⛔⛔ **E a relaxação não cura uma dobra: troca-lhe a espécie** — a gravata sai e a mesma face fica a apontar contra a vizinhança. ⇒ a cura é o **critério**: a chave conta as duas espécies com uma **folga de `20`** calibrada no vazio entre os dois lados que **ele** julgou (`125` = *«destruiu a malha»* · `6`–`8` = *«melhor resultado até agora»*), e as pontas deixam de ser pagas por meia dúzia de faces. Resultado: **`0/5` amputadas, gap `0,18`, grade `0,74`** — melhor que a build que ele aprovou. ⏳ **A fenda FICA** (é um nó da grade; a cura é a montante) e a folga tem **dois** pontos de calibração, não uma varredura. `PH2D_TIP_CAP=0` bissecta. [Handoff de 03/09](docs/3D/handoffs/HANDOFF_line_quadextract_A_CALOTA_E_A_GRAVATA_2026-09-03.md) + [plano §105](docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md).
  **Smokes:** `PH2D_SCULPT3D_SMOKE=<n>` (a W9 é a cena **`=34`**). ⚠️ **Rode uma vez SEM a env var** — é a metade que
  prova a inércia.
  **Ler:** [porta do cofre](docs/3D/README.md) · [00-INDEX](docs/3D/00-INDEX.md) · [handoffs](docs/3D/handoffs/README.md) ·
  [história](docs/archive/estado-2026-08-18/sculpt3d.md)

- **3D Modeling (campo implícito)** — modelador **SDF** editável para sempre
  ([ADR-0161](docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)):
  `ph2d-field` (documento + primitivas + modificadores) · `-field-eval` (avaliador híbrido, bordo
  da peça) · `-field-ecs` (a árvore de modelagem **é** a hierarquia da cena) · `-field-mesh`
  (Surface Nets) · `-field-render` (traçado) · `ph2d-panel-model3d`. Abre pelo pill **MODEL**.
  ⚠️ **A hierarquia da cena É o documento** — o `FieldDoc` é **cozido** dela a cada quadro, e é por
  isso que o undo, o olho, o cadeado e o reparentar da casa valem aqui sem código próprio.
  ⚠️ **Só uma OPERAÇÃO pode ter filhos**, e a lei impõe-se na **derivação** (`promote_leaf_hosts`),
  nunca em cada gesto. ⚠️ **O painel oferece EXATAMENTE o que o gesto faz** (W34) — a lei está em
  [`field3d_reach_tests.rs`](shells/desktop/src/field3d_reach_tests.rs), e ela apanha os dois
  lados (botão mudo · gesto inalcançável).
  ⚠️ **A peça ATRAVESSA o arquivo** (W35) — ela é uma árvore de entidades e o `ProjectState` é o
  mundo inteiro, então o `PROJECT_SCHEMA` **não se mexe**; a nota que dizia o contrário era velha.
  ⚠️ **Tomar o canvas LIBERTA quem o tinha** (W40+W42): pegar noutra ferramenta fecha o MODEL **e
  desarma o módulo** — e a **vista** (câmera · prato parado · verbo e referencial do gizmo ·
  isolamento) sobrevive ao fecho (W43), enquanto o cache do quadro não. ⛔ Um campo novo no `Smoke`
  é **erro de compilação** em `field3d_view::View::of` até alguém dizer se é vista ou cache.
  ⭐ **A CÂMERA é alcançável** (W47–W52): seis **vistas nomeadas** (`Numpad1/3/7` + `Ctrl` para a
  oposta) com botões que dizem o atalho, o **gizmo de navegação** por bolas de eixo — pesquisado a
  pedido do Enio, ⛔ o *ViewCube* da Autodesk está **patenteado** (US 7 782 319, expira 2029) e o
  próprio paper deles mediu que o ganho está no **arrasto**, não no clique —, que se **desloca para
  fugir à moldura** (`panel_ops::panel_rects`, a fuga mais barata), e a **viagem** animada entre
  vistas. ⚠️ **`Role::Viewpoint` SOBREVIVE ao `reduced_motion`**, sozinho entre todos os papéis, por
  decisão do Enio com a alternativa na mão: *aqui o CORTE é pior do que o movimento*.
  ⭐⭐ **O DESENHO VIRA PEÇA, e continua a ser a FONTE** (W53–W55): `+ Extrude` / `+ Revolve` cozem o
  contorno escolhido no editor vetorial (o fluxo do MoI, com a caneta que a casa já tem) — o motor
  existia **desde a W3** e nenhum botão o alcançava. O `FieldProfileSource { path, level }` mantém o
  vínculo **vivo**: editar a curva remodela a peça, e a linha **Resolution** (1..16) afina a
  conversão. ⚠️ **Sem cache, de propósito** — recozer custa 7 µs e comparar 0,2 µs contra um quadro
  de 16,7 ms, e um resumo guardado seria estado derivado a envenenar o undo.
  ⚠️ **A régua da suavidade é a NORMAL, não a silhueta** (W54): a polilinha erra **0,079 %** da peça
  (invisível) e a normal salta **6,43°** — é isso que a luz mostra. A tolerância é `1e-4` pelo joelho
  medido, e ⛔ a tabela de 2026-08-19 estava **desmentida por 2,4×**.
  ⭐⭐ **O traçado é 2,5× mais rápido e o vínculo é ALCANÇÁVEL** (W56e–W58d): a marcha especializa a
  árvore por **ladrilho × fatia de profundidade** (`167 → 66 ms` a 168 arestas) e o passo dela sai do
  **documento** — auditado construtor a construtor, **só o arredondamento exacto infla** (`√2`), e o
  `Taper` **desce** a `0,844`. O vínculo desenho→peça vê-se na Hierarquia (selo `LNK`) e tem gesto
  (`Unlink` / `Link Drawing`), e a **selecção múltipla nasce no canvas** (`Shift`+clique alterna ·
  `Shift`+arrasto **soma**, apanhando também **o que está tapado** — as formas nascem empilhadas no
  alvo da câmera). ⚠️ **Um desenho com contorno interior já virava peça com FURO** — a composição do
  `VecPath` exprimia-o desde a v6 do formato, e o que faltava era o gate.
  ⭐⭐⭐ **A jornada de 24-26/08 (W59–W80)** — a **exportação caiu de 8 min 17 s para 6,4 s** (77×, arquivo
  idêntico) e **saiu da thread que desenha**, porque declarar o congelamento curava a mensagem e não
  o congelamento (a 12 s o KDE dá a janela por morta e oferece forçar o encerramento); a Hierarquia diz
  qual linha está **isolada** (selo `ISO`, que ganha do `LNK` por ser estado da VISTA) e a exportação
  diz **onde** a peça está; o `Mirror` passa a ter **três eixos**; duas formas escolhidas viram **duas
  peças**, cada uma ligada ao seu desenho; e uma escultura que perdeu o ficheiro tem **`Relink
  Sculpture…`**. ⭐⭐ **O custo de MOVIMENTO virou CONSTANTE (~53 ms em qualquer nível)** — o contorno
  também engrossa enquanto a mão mexe —, e por isso o teto de `Resolution` fica em **64**, medido com o
  relógio certo (o quadro **assente**, não o de movimento). ⚠️ **E o passo da marcha estava ERRADO**:
  arredondamentos exactos **encadeados** compõem o factor, e a cena 1 do smoke marchava acima do seguro
  desde que existe. Mecanismo, tabelas e provas de mutação: [doc 06 §69–§81](docs/3DModeling/06_resultados_cena_e_gizmo.md)
  + [handoff de 26/08](docs/3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md).
  ⭐⭐ **O catálogo de formas FECHOU** (W100–W103): **16 entradas** numa **paleta com busca** (`A` ou
  *+ Add shape…*), agrupadas por família — a fileira de chips cortava em `MAX_MODES = 8` e já tinha 8.
  O `Primitive` tem **14** famílias, e cada linha do catálogo carrega o **próprio construtor**
  (⛔ as quatro constantes `SHAPES.len() − N` morreram: acrescentar no fim fazia o botão *Extrude*
  abrir o diálogo de escultura, **sem erro nenhum**). ⭐⭐⭐ **E o filete alcança TODA aresta de toda
  forma** (W104): `0,0 %` da superfície sobre um vinco com o filete a metade do limite, nas dez formas
  que o têm — medido por uma sonda que **acha** as arestas pela variação da normal, e não por uma lista
  escrita à mão. ⚠️ Antes disso o `round` do **cone** e do **prisma** era **inerte** (`+0,0 %` de
  volume, campo bit a bit igual) e o da **cunha** fazia a peça **crescer 41 %**; o **arco de toro** não
  tinha controle de filete nenhum. `FIELD_DOC_VERSION` **4 → 10**. Cena **`=11`**.
  **Aberto:** ⏳ **O filete só é um ARCO a 90°** — o operador recua o vértice `(1 − 1/√2)·r/sin α` e um
  arco verdadeiro recua `r·(1/sin α − 1)`; numa ponta de estrela (19°) isso é **`2,29×` menos** filete
  do que o número diz. Hoje compensa-se **só nas quinas AGUDAS** (`max(1, factor)`), e as duas curas
  gerais estão **medidas e rejeitadas** (doc 06 §102.5 e §104.3) · ⏳ o teto de `round` da **estrela** é
  `12,3 %` do bordo, contra `43–60 %` de todas as outras formas — ela é a única em que a mistura é uma
  faixa estreita a atravessar uma face grande ·
  ⛔ **A BASE FICA:** o quadro de movimento custa `26,7 ms` contra um orçamento de `16,7`
  (era `69` antes de 26/08) — a marcha é `80 %` dele, com `8,7` amostras por pixel, e o custo é **por
  aresta tocada**; ⛔ a **sobre-relaxação** está fora (a contagem de passos já é mínima) e atacar a
  **montagem** tem tecto **medido de `20 %`** · ⏸️ baixar as arestas do contorno a mexer
  (`PREVIEW_MAX_EDGES`) tem o preço medido e **muda a FORMA** — decisão de quem vê · ⏸️ o 2.º degrau
  do assentar custa `504 ms` numa peça densa (a escada tirou-o do caminho; o número fica) · ⏸️ um laço
  que **SUBTRAI** — mecanismo medido (os **três** modificadores são um vocabulário só) e as 4 saídas
  com preço, **decisão do Enio** · ⏸️ a barra **demonstrável** da interpolação trilinear é `√3` e
  ship-se o `√2` medido (dívida nomeada) ·
  ✅ **o panic do `ph2d-gridmap` TEM ENDEREÇO e deixou de matar a tentativa** (30/08): era
  `map.uv[p][l]` na `solve.rs` — a nota procurava a **linha** `336` e a função tinha descido para
  `358/359`, *um número de linha obsoleto não desmente o ficheiro* — mais o irmão `partners[p][l]`
  na `assembly.rs`; os dois **contam e saltam** agora (`SolveReport::mismatched_locals`). ⛔ Ele
  matava **2 das 3** candidatas do botão na escultura mais recente do Enio, e a rede devolvia-as
  como *«a malha é grossa demais»* (hoje `RemeshRefusal::Panicked`). ⏳ A causa a montante fica
  ABERTA — `CutReport::side_patch_flips` dá `0` numa peça e **`2`** na dele
  ([handoff §8-quateretvicies](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-30.md)).
  ⛔⛔ **RECUSAS MEDIDAS — não as reconstrua** (mecanismo no doc 06 §65, §69 e §70): **os níveis de
  exportação NÃO podem mandar na densidade dos quads** — a escada foi implementada inteira e o `Max`
  custou **27 min 29 s** para sair com `316` arestas de bordo e `6` não-manifold; *o limite da cadeia
  não é o tempo, é a TOPOLOGIA da extracção*, e **REVERTEU** (a densidade fina tem de FECHAR primeiro —
  achado da `line/quadextract`) · especializar a **2.ª passagem** do traçado por ladrilho é neutro a pior ·
  o vínculo à escultura **viva** custa `229–389 ms` a 128³ · a grade **fina** para a cadeia de quads é
  **107×** o preço para a mesma resposta, e piora a fidelidade · e a especialização por ladrilho está
  **ilibada** (sem ela o traçado vai de `58` para `565 ms`).
  ⚠️ **SEIS notas deste módulo estavam desactualizadas contra o código** (auditadas em 25 e 26/08): o
  traçado «2,4×» · o teto de `Resolution` · o paralelogramo (feito na W59) · o sítio da peça · o
  `Mirror` «não demonstrável» (**demonstra-se**, e já tem três eixos) · o gradiente da escultura («não
  medido» com um gate a medi-lo). ⛔ E **dois gates prometiam «erro de compilação» sobre listas
  escritas à mão** — os dois estão derivados agora. *O §5 só se edita na integração, então ele acumula
  trabalho já pago — audite a lista antes de pegar um item dela, e confira o CÓDIGO antes de acreditar
  numa ausência.*
  **Smokes:** pill **MODEL** · `PH2D_FIELD_SMOKE=<n>` (o roteador é
  [`field3d_smoke_scenes.rs`](shells/desktop/src/field3d_smoke_scenes.rs)).
  ⚠️ **Preferência fora do repo:** `~/.ph2d/prefs.txt` — um `reduced_motion=1` esquecido reprova
  smokes sobre produto correto **em todo o resto do app**, e a viagem entre vistas é a excepção.
  **Ler:** [`docs/3DModeling/`](docs/3DModeling/) ·
  [`06_resultados_cena_e_gizmo.md`](docs/3DModeling/06_resultados_cena_e_gizmo.md) §1–§104 (uma seção
  por wave, com a tabela medida e as provas de mutação; o **§13.0** é a lista viva do que está aberto,
  **auditada contra o código** em 26/08) ·
  [`07_fillet_e_chanfro_por_aresta.md`](docs/3DModeling/07_fillet_e_chanfro_por_aresta.md) ·
  [`08_formas_por_formula.md`](docs/3DModeling/08_formas_por_formula.md) ·
  [handoff de 29/08](docs/3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-08-29.md)
  (⚠️ o §9 tem **quatro** coisas que uma leitura rápida do diff entende ao contrário — entre elas que o
  `round` do cone e do prisma era **inerte**, não «fraco» — e o §10 as duas premissas que a
  implementação refutou) · [handoffs](docs/3DModeling/handoffs/README.md)
- **Flip** — animação 2D no idioma do Grease Pencil: tira de quadros, onion, tween v2 (correspondência por atribuição
  ótima + espiral logarítmica), **colorize LazyBrush**, multiplano 2.5D, airbrush, pressão, e o
  **motor novo de traço**, em que o traço deixa de ser rasterizado e passa a ser **PERCORRIDO**
  (`τ = ∫ f(dn) ds / pitch`, `α = 1 − exp(−τ)`) — a lei aditiva da tinta, que é o **limite** dos buffers de dab da
  indústria. Dois motores, **uma lei**: referência em CPU e o compute que shipa, unidos por gate de paridade cuja barra é
  **derivada do formato** (`rgba16float` ⇒ `2⁻¹¹`).
  ⚠️ `PH2D_FLIP_NEW_ENGINE=0` volta ao rasterizador antigo (vivo e testado, útil para bissecar).
  **Aberto:** ⏳ **W-Saída — o Flip sai do Flip** (Enio, 2026-08-23, **fim da fila**): assar um
  quadro em pixels destrava **três** features que já existem do outro lado — os 16 exportadores de
  imagem, o `Pack into Sheet`, e a camada do Painter. ⚠️ **É UM buraco, não três:** a entidade de um
  objeto Flip não tem `Sprite` nem pixels, e as três portas só sabem o que é um pixel; o primitivo
  de leitura **já existe** no `walk_gpu` (o harness de paridade usa-o). ⭐ E o T2 fecha o círculo
  com a §11 do Sprite — uma tira empacotada **é** o pool que uma `AnimationTag` percorre. Plano:
  [`01_plano_waves.md` §W-Saída](docs/Flip/01_plano_waves.md) ·
  cache em **tiles de MUNDO** (sobreviver ao pan) · o **resíduo de quina** que a lei de área expôs
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
  onde a **timeline** ([ADR-0143](docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md)) e a **física** (`SignalOnHit`) se encontram: os produtores
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
  ✅ **O UNDO SEPARA PREVIEW DE DOCUMENTO** (Enio, 2026-08-23: *«corrigir o CtrlZ para ambas»* —
  feito no mesmo dia, [`preview_drive.rs`](shells/desktop/src/preview_drive.rs)): *o documento é o
  valor **AUTORADO**; o que um motor escreve agora é pré-visualização — vê-se, não se guarda nem se
  desfaz.* O motor continua a escrever no mundo (um só sink); a **captura** é que repõe o autorado
  durante a fotografia. ⚠️ **O ledger entra na ASSINATURA da `ProjectState::capture`** — uma
  função-irmã «com ledger» seria a segunda porta pela qual o defeito voltava. ⚠️ **A granularidade
  é o CAMPO**: repor o `SpriteAnimator` inteiro engoliria uma mexida na velocidade a meio da
  reprodução. ⚠️ **O passo nascia por CLIQUE, não por quadro** (mover o cursor não conta como
  input) — e é por isso que tirar só o relógio do componente registado **não** curava. ⭐ A `settle`
  faz a corrida virar **um** passo (*«desfaz a corrida»*), e a lei da **outra mão** impede que uma
  edição feita a meio dela fique por baixo do memo. Vale para o **save** pela mesma porta.
  ✅ **E O TERCEIRO MEMBRO — a timeline — CURADO no mesmo dia**
  ([`timeline_preview.rs`](shells/desktop/src/timeline_preview.rs)), pelo mesmo ledger. ⚠️ **A nota
  que o deixava de fora estava ERRADA no ponto que decidia o preço:** ela dizia que o censo era
  `O(mundo)`; o `TimelineDoc` **nomeia** quem ele anima (`doc.bindings()`), então é `O(bindings)`.
  *Uma ausência afirmada sem olhar a API é um palpite com cara de medição* — a segunda no mesmo dia
  (a outra: «este app não tem diálogo de ficheiro»). ⭐ **E os QUATRO componentes que a timeline
  escreve entram**: o `Sprite` ficou de fora na 1.ª versão por uma COLISÃO de granularidade (a §11
  conduz aquele componente por CAMPO), e a cura foi **olhar o que a curva de facto escreve** —
  `tint[3]` é um número, não o `Sprite`. *Quando duas granularidades colidem, a pergunta é qual
  delas é grosseira demais para o que o motor faz*
  ([auditoria 21 §4](docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)) ·
  **Aberto:**
  ✅ **O FICHEIRO DO PROJETO TEM NOME** (2026-08-23, [`project_io.rs`](shells/desktop/src/project_io.rs)):
  `Save` · `Save As…` · `Open Project…` com diálogo, e os três itens do menu deixaram de ser
  **mudos** (eles consumiam o clique e não faziam nada — *pior que um botão ausente: o artista
  conclui que gravou*). A sessão passa a ter um ficheiro (`App::project_path`), a env só o
  **semeia**, e a barra de título diz qual é. ⚠️ **Abrir pergunta SEMPRE** — *o gesto que destrói o
  trabalho não gravado pergunta; o que grava é que pode ser silencioso.* ⚠️ A extensão é
  **`.ph2dproj`** e **não** `.ph2d`, que já é uma **imagem** neste app (há gate a ligar as duas
  listas). ⚠️ O teclado e o menu chamam as **mesmas** funções, e o `project_save()`/`project_load()`
  sem caminho **morreram** — uma decisão de *onde* escondida dentro de quem executa não é alcançável
  nem por um gate nem por um diálogo ·
  ✅ **`SpriteSource::Individual` PERSISTE — esta nota envelheceu** e mandava reconstruir trabalho
  pago: [`project_sprite_pixels.rs`](shells/desktop/src/project_sprite_pixels.rs) fecha as **oito**
  ferramentas de imagem de uma vez pelo funil `commit_edited_texture`, com a identidade a ser o
  CONTEÚDO (`AssetId` blake3) e precedência por ORDEM sobre o Painter/bake. ⚠️ O
  `CookedTexture` fica de fora **por gate explícito** (`should_collect`) — a pergunta aberta é se
  alguém o re-deriva no load, não se ele devia ser embutido · limpar o `vec_history` morto
  (subsumido pela captura).
  **Ler:** [`project.rs`](shells/desktop/src/project.rs) · [`undo.rs`](shells/desktop/src/undo.rs) ·
  [história](docs/archive/estado-2026-08-18/editor-shell.md)

- **Componentes / instâncias** — identidade de objeto (`StableId`), ordem de irmãos como DADO (`SiblingOrder`),
  snapshot v2 e a **PRIMEIRA migração de `PROJECT_SCHEMA` do repo** ([ADR-0164](docs/architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) ·
  [0165](docs/architecture/decisions/0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md) ·
  [0166](docs/architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)),
  mais a crate-folha `ph2d-component-desc` (descritor de componente, catálogo de 108 tipos) de que o Inspector deriva rótulos.
  ⚠️ **`canonicalize` MORREU e a propriedade dele não se perdeu** — mudou de dono (`world_to_snapshot` ordena por `StableId`,
  que sobrevive ao respawn por construção; 18,7 ms para 0,088 ms a 10 k entidades, medido). ⛔ Não o reintroduza.
  ⚠️ **O `StableId` NÃO é componente registado, e a ausência é a decisão** — registá-lo poria a identidade também num
  `ComponentBlob`, e a cópia profunda da F4 daria à cópia a identidade do ORIGINAL.
  ⭐⭐⭐ **AS INSTÂNCIAS EXISTEM** (26/08, F1.6 + F4.1–F4.5 + F4.6a/b —
  [handoff](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_F4_2026-08-26.md)): *Make Component* esconde a
  receita e deixa uma cópia no lugar; *Instantiate* põe outra; **editar a receita muda todas as cópias no mesmo quadro**;
  editar UMA cópia vira **excepção** (e *Apply to Master* promove-a, *Detach* solta, *Revert* devolve **mantendo a
  posição**). Duplicar passou a levar a **subárvore inteira** com identidade nova — e a junta da cópia prende **os corpos
  dela**. ⚠️ **A cópia profunda SALTA TRÊS dos quatro `owned_document`** (`PaintedDoc` · `BakedForm` · `FlipObjectRef`,
  declarados em `DROPPED`): copiar o id de um documento possuído 1:1 poria duas entidades a escrever nele, e duplicar
  uma sprite pintada devolvia um sósia que apaga a tinta do original — *a cópia rasa acertava nisto por acidente*.
  ⚠️ **O `VecPathRef` SAIU dessa lista na F4.6a** — saltá-lo não deixava a peça «sem o vínculo», deixava-a **sem
  geometria nenhuma**; hoje o documento é **clonado** com o par no mapa `path ⟺ entidade`, e o gate é um censo de DOIS
  lados: um bridge novo que não venha à lista **não compila**. ⚠️ **`MasterPiece` é DERIVADO, nunca gravado**
  (só o `MasterRoot` viaja), e o passe tem **duas** metades obrigatórias: marcar sem desmarcar deixa uma peça arrastada
  para fora do mestre **invisível ao solver, em silêncio**. ⚠️ **`deep_copy_subtree` não instancia** — a porta do produto
  é o `instantiate.rs`, com gate a mantê-la com **um** chamador. ⭐ E um **objeto vazio ou um grupo** finalmente se pega
  no canvas (um anel que é o corpo dele, não uma marca de selecção).
  **Aberto:** ⭐⭐ **as INSTÂNCIAS têm VARIANTES** (27/08, F5 critério 2): *Make Component* sobre uma cópia faz uma
  **variante** que segue a base, e a troca base↔variante **preserva as excepções** por re-key lido dos **próprios elos**
  — ⛔ sem nomes, sem caminhos, sem heurística, que é o que a separa do `ByName`/`ByHierarchy` do Unity · ⭐ o **cartão
  no topo do Inspector** diz o que a cópia **É** (*Instance* / *Variant of*), o que ela possui e os órfãos — ⚠️ a nota
  de que *«nada na tela MOSTRA que campo está overridado»* **fechou aqui** · ⚠️ **a F1 FECHOU nas DUAS metades** (a
  física em 24/08, a timeline em 25/08): renomear um objeto animado **não** desliga o binding, com gate
  (`renaming_an_animated_object_does_not_unbind_it` + `a_stranger_with_the_old_name_does_not_capture_the_animation`) —
  *a frase pôde envelhecer três dias porque nenhum gate a contradizia* ·
  ⏳ **F5 critério 4** (*Apply to inner master* apagar o override nos níveis intermediários) e a troca para um mestre
  **NÃO aparentado** (3 modos + relatório, ⛔ nunca automática) · ⏳ **a F4.6c DESBLOQUEOU e passou a CONTER uma fatia:**
  portar os **eixos de propriedade** (`Size=Small, State=Idle`) do `vec_variants.rs` para o cartão geral **antes** de
  apagar os 24 ficheiros do `VecInstance` — *um porte que apaga uma feature não é um porte* · ⏳ **F6–F8** ·
  ⚠️ **O `physics_ecs_c9` NÃO tem baseline a re-capturar e NÃO corre na varredura impactada** — o `spike.yml` compara os
  **três OS entre si**, então o risco real é eles **discordarem**, e só o CI o mede; a **F4.7 FECHOU** e acrescentou-lhe
  a lane de mestre+instância, que localmente só se prova pelo que dá (corre e é estável, mesmo hash em 2 de 2) ·
  ⛔ **a pose de repouso de uma peça DINÂMICA não propaga, e é DECLARADO** (o dono do `Transform` de um corpo dinâmico é
  o solver sempre): mover o braço da receita não move o das instâncias, nem depois de um Reset — gate com o nome inteiro ·
  ⚠️ **`hit_indexed_ids_are_registered` era CEGO aos chips guiados por TABELA** (só lê `.register(ids::LITERAL, …)`, e a
  mutação que apagava o `populate` deles **SOBREVIVEU**): o gate irmão `table_driven_chips_are_registered_too` fecha-o,
  com **catraca de 9 tabelas por registar em 4 painéis de OUTRAS linhas** — ⛔ **ela só ENCOLHE**, e quem registar um
  desses chips tem de apagar a linha correspondente (o gate tem a metade *«já não descreve nada»*) ·
  F2-F8 do [plano vivo](docs/Components/05_plano_de_implementacao.md).
  **Smokes:** abrir um `.ph2dproj` gravado ANTES de 24/08 (tem de dizer *"Project migrated from format 95 to N"*, com
  **N = o `PROJECT_SCHEMA` de hoje** — ⛔ não o copie para cá, leia-o em
  [`project_schema.rs`](shells/desktop/src/project_schema.rs), porque esta linha já o teve errado —
  ⚠️ **um v97 ou v98 é RECUSADO**, e é a decisão da `line/Vector` levada até ao fim: sem `ProjectFileV97` congelado não
  há forma honesta de ler aqueles bytes) ·
  reordenar irmãos na Hierarquia + Ctrl+Z · renomear um corpo com junta (`PH2D_PHYSICS_SMOKE=6` ou `=67`) · copiar um
  ragdoll e dar Play.
  **Ler:** [`docs/Components/`](docs/Components/) ·
  [handoff de 24/08](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_F0_F1parcial_2026-08-24.md) (⚠️ o §9 lista
  **cinco** coisas que uma leitura rápida do diff entende ao contrário, e o §10 as **três** premissas do plano que a
  implementação refutou) · [handoff de 26/08](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_F4_2026-08-26.md)
  (⚠️ o §3 tem **doze** dessas, e o §4 as premissas que a F4 refutou — entre elas a lei que **nenhum documento tinha**:
  *o que não PROPAGA não se REMAPEIA*, achada por uma **mutação que SOBREVIVEU** porque nenhum gate corria o passe duas
  vezes antes de medir) · [handoff de 27/08](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_F5_2026-08-27.md)
  (⚠️ o §7 traz as **quatro** correcções que esta seção precisou, e o §8.2 as **quatro fixturas que mordiam ANTES de
  medirem o produto**)

- **Image Tools — os utilitários de bitmap** (⚠️ **~30 k LOC que esta seção nunca mencionou**, achado
  da auditoria de 2026-08-18): `ph2d-tool-color-equalization` (10.291) · `ph2d-tool-bgremoval` (8.377) ·
  `-upscale` (1.788) · `-equalize-sizes` · `-rasterize` · `-padding` · `-make-square` · `-real-size` ·
  `-trim-transparency`, mais os painéis irmãos. Cada uma é **drop-crate** sob o contrato `Tool=12` (§6),
  e três implementam `RasterEditTool` — ⚠️ **quatro**, contando o Painter: quem escrever a 5ª herda essa conta.
  Vizinhos sem entrada própria: `ph2d-inpaint` (2.140) · `ph2d-grid` + `ph2d-panel-grid-snap` (7.012) ·
  `ph2d-tokens` (5.141, o design system do §7) · `docs/Deform/` · `docs/Pixel Art/`.
  **Ler:** [`Image Tools Bugs`](docs/Image%20Tools%20Bugs/README.md) · **Inpaint** = PatchMatch multiescala CPU+GPU
  ([ADR-0102](docs/architecture/decisions/0102-inpaint-multiscale-patchmatch-cpu-gpu.md), [plano](docs/Inpaint/01_pesquisa_design_plano.md)) ·
  **Deform** = transformação/deformação do Painter, com [tracker único](docs/Deform/00_README.md) e [índice](docs/Deform/README.md)
- ⚠️ **As duas maiores crates do repo não eram nomeadas em lugar nenhum deste arquivo:**
  [`ph2d-tool-painter`](crates/ph2d-tool-painter/) (**136.093 LOC** — é onde o módulo Painter de facto
  vive; o §5 nomeava só `ph2d-paint-gpu`) e [`ph2d-editor-core`](crates/ph2d-editor-core/) (**84.015** —
  widgets, ids, interaction, e **53** gates de arquitetura em `tests/`). *Um módulo que o roteador não
  nomeia é procurado por `grep`, não alcançado por link.*
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
  Fechados sem pendência: **KTX2 Fase 2**
  ([ADR-0055](docs/architecture/decisions/0055-cooked-texture-compression-pipeline.md), W3 = integração com o Painter) · **imageio AVIF**
  ([ADR-0054](docs/architecture/decisions/0054-imageio-pipeline.md)).

- **Sprite Inspector** ([ADR-0069..0074](docs/architecture/decisions/)) — ⚠️ **esta linha dizia
  «fechado sem pendência» e a spec pede 12 seções: existiam 9.** ✅ **As três que faltavam nasceram
  em 2026-08-22/23** (`line/Sprite`): 9-Slice, Sockets/Âncoras (com o gizmo de canvas) e Animation.
  ⚠️ *A informação existia desde 2026-05-31 num handoff **arquivado**, e o roteador dizia o
  contrário — o roteador é o que se lê.*
  **Aberto:** ✅ **a §11 Animation NASCEU (2026-08-23) — as DOZE seções existem.**
  ⛔ **O `SpriteFrames` da spec §8.3 NÃO foi construído, e é uma recusa medida:** o pool de frames
  já existe (a **grelha** `hframes × vframes`, cujo índice `Sprite::frame` é o **único sink vivo** —
  o `SpriteSheetRef` é proveniência de autoria, não índice). Uma animação é um **intervalo nomeado
  sobre as células que a sprite já tem** — o modelo do Aseprite aplicado ao pool que existe.
  ⚠️ **O tique corre no PASSO FIXO** (`SimComponent`, o replay tem de o reproduzir) e a lei pura
  **nunca vê um float**; escreve `Sprite::frame` **só quando ele muda** (o undo regista por diff).
  ⚠️ **Tocar uma vez pára no ÚLTIMO frame** · ping-pong não repete as pontas · `repeat_delay` só
  conta se vier outro ciclo · velocidade negativa toca ao contrário, não faz o tempo recuar.
  ⚠️ **`ANIM_TAGS_MAX` é 64 e não os 256 da spec** — o motivo é o dela («típico < 50»): *um modelo
  que aceita o que o painel não mostra produz estado inalcançável*.
  ⚠️ **O TRANSPORTE foi auditado em 23/08 e tinha quatro defeitos numa família só**
  ([doc 21](docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md), e a wave está aplicada):
  *«pausado» e «terminado» leem-se igual no `playing == false` e não são a mesma coisa* — a
  reprodução que se **ESGOTOU** volta ao princípio quando alguém lhe toca (a caixa **ou** a lista),
  e uma pausa explícita não é tocada; **rebobinar move a IMAGEM**, não só contadores; e a caixa
  «Playing» **pergunta à cena**, nunca ao `WidgetStore` (era dupla fonte de verdade, e o motor
  escreve aquele campo sozinho). ⚠️ **A barra de frames ARRASTA** (pedido do Enio) — ela era
  desenho, hoje é um `Slider` registado que mede **posição** e não progresso, e **agarrá-la pausa**
  (o dedo e o tique escreviam o mesmo campo). ⭐ A régua vivia em **três cópias** e uma mutação
  sobreviveu a mudar só a do pintor: hoje é `scrub_position` ↔ `scrub_cell`, uma lei em dois
  sentidos com gate de ida-e-volta. ⛔ A §11 tinha 33 gates e **nenhum que carregasse num pixel** —
  hoje tem `seam_anim.rs`. ⚠️ **MEDIDO e não curado:** com a animação a tocar, um quadro com input
  regista um passo de undo (o relógio vive num `SimComponent` registado) — **família
  pré-existente**, a física faz o mesmo com o `Transform`; as três saídas estão na auditoria §4 e a
  escolha é do Enio.
  ⭐ **PINTAR UMA FOLHA DESDOBRA-A** (Enio, 2026-08-23, com foto): sob pré-visualização de
  ferramenta o extract troca a UV pelo rect INTEIRO da textura transitória, mas o quad continuava a
  ser o de UMA célula — a tira saía **esmagada 8:1**. ⚠️ **E o caminho do PONTEIRO fazia a mesma
  conta** (`sprite_image_to_screen_affine`), o que os deixava consistentes um com o outro e errados
  com o artista; por isso os dois chamam a MESMA função (`sim_extract_sheet::unfolded_quad`).
  ⚠️ **O desdobrado centra-se no PIVÔ e ignora o frame vivo** — ancorá-lo na célula viva (a 1.ª
  versão) fá-la-ia **deslizar debaixo do pincel**, porque o tique continua a andar. ⛔ A
  pré-visualização da grelha faz o **contrário** e também está certa: ali a célula viva **é** o quad
  real. ⭐ E como a folha aberta mostra tudo, o `frame` deixa de ter efeito visível — daí a
  **célula extra acima dela, a tocar a animação enquanto se pinta**, ⚠️ **mesmo com o transporte
  pausado**: ela corre sobre uma **cópia** do animador (a lei pura também desiste com
  `playing == false`), e do que a cópia produz volta só o relógio — o `playing` do documento fica
  intacto. ⚠️ **As LINHAS da grelha seguem o MODO** (`lattice(.., unfolded)`): a folha pintada
  centra-se no pivô e a pré-visualizada dispõe-se à volta da célula viva, e as duas disposições
  **nunca** coincidem (o desvio é `(lcol + ½ − hf/2)·cw`) — foi o 2.º report com foto.
  ⚠️ **E a CAIXA DO GIZMO envolve a folha aberta** (`sheet_grid_overlay::gizmo_box`) — ela ficava do
  tamanho de UMA célula no meio de oito. ⛔ A escolha vive numa função com gate, e **não no fio**: em
  `snapshots::build_view` ela não é alcançável de um teste, e a mutação que a desligava compilava e
  passava a suíte inteira.
  ⭐ **A GRELHA VÊ-SE** (Enio, 2026-08-23): a caixa **«Show sheet on canvas»** (§4 Sprite Sheet, só
  aparece com grelha) abre a folha no canvas — as outras células esmaecidas no lugar delas, com as
  linhas dos cortes e a viva contornada. ⚠️ **Fantasmas de PRESENTE, nunca documento** (o molde é o
  fan-out do 9-slice) e o interruptor é **vista**: vive só no `WidgetStore`, sem barramento, sem
  undo, sem save. ⛔ Um clique numa célula **não** escolhe o frame — pede hit-test de canvas a
  competir com a seleção; a barra de frames e o campo já o fazem.
  ✅ **IMPORTAR ASEPRITE (`.ase`) — FEITO em 2026-08-23** (crate-folha
  [`ph2d-aseprite`](crates/ph2d-aseprite/) + [`ase_import.rs`](shells/desktop/src/ase_import.rs)):
  largar o ficheiro nativo dá **UMA** sprite com grelha + a biblioteca de animações dele.
  ⚠️ **Clean-room da spec pública** (o Aseprite é GPLv2; a especificação do formato é documentação).
  ⚠️ **O corte entre as duas portas é o que cada uma SABE**: o par `.png`+`.json` traz rectângulos
  com nome ⇒ N sprites soltas; o `.ase` traz a **autoria** ⇒ uma sprite com grelha, que é o modelo
  da §11. ⚠️ **A ordem dos quadros é o CONTRATO** — uma tag indexa **células**, então a folha é
  empacotada em linha; por colunas dá uma folha bonita e todas as animações trocadas.
  ⚠️ **UMA TIRA sempre que couber** (o `hframes` do inspector fica legível); o teto é
  `MAX_SHEET_EDGE_PX = 8192`, que é **memória de GPU** (`max_texture_dimension_2d`).
  ✅ **E a recusa da duração por-FRAME reabriu e FECHOU** (spec §8.12: *«não há quem a produza»* —
  há, é este importador, e nos ficheiros reais elas **variam**): `AnimationTag::per_frame_ms`, vazio
  = uniforme. ⭐ A lei pura não precisou de refactoração — o `step_ticks` já perguntava **por
  frame**, era só a resposta que era uniforme. ⚠️ Curto ou `0` caem no `frame_ms`, então **não há
  estado inválido** quando o intervalo muda. ⭐ **E o Inspector EDITA-O** (Enio pediu: *«se não tiver um
  parâmetro de duração para cada quadro, crie»*) — o campo mora **colado à barra de frames**, que já
  é o selector de célula: *um painel que pergunta duas vezes «qual quadro?» é um painel em que os
  dois podem discordar*. ⚠️ `0` = herda · declarar uma célula **não** escreve as outras · limpar
  **encolhe** o vetor (senão o aviso de ritmo próprio mente para sempre).
  ⚠️ O que o ficheiro traz e não honramos sai numa **nota que nomeia a camada** (tilemaps, z-index
  de cel, modo de mistura de grupo) — *um importador que ignora em silêncio é pior que um que
  recusa*. ⭐ **O smoke ESCREVE o `.ase`** (`PH2D_ASE_SMOKE=1`), então testá-lo não precisa do
  Aseprite instalado — e há gate a correr o escritor do smoke pelo leitor real.
  ⭐ **E 18 ficheiros escritos pelo Aseprite REAL lêem-se, 0 recusados** (as 12 fixturas de teste do
  repositório oficial + 2 exemplos MIT + 4 personagens): o instrumento é
  `cargo run -p ph2d-aseprite --example ase_info -- <ficheiro|pasta>`, que corre o **mesmo** parse
  do produto. ⚠️ Dois achados que só ficheiros reais dão: a duração **varia por quadro** em
  ficheiros comuns, e personagens reais chegam **sem tags** — o que torna a regra «sem tags recebe
  uma» o caminho normal, não a excepção.
  ⚠️ **E o `.ase` não aparecia no diálogo «Import…»** (Enio, no mesmo dia) — o defeito **não era o
  `.ase`**: o drop roteava por um predicado (11 extensões) e o diálogo oferecia uma lista **escrita
  à mão** com 4, então o `.gif`/`.psd`/`.ora` estavam invisíveis lá **há meses**. ⇒
  [`import_router.rs`](shells/desktop/src/import_router.rs): a **lista** é a fonte
  (`ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS` + `ase_import::ASE_EXTENSIONS`), o predicado é derivado
  dela, e **as duas portas chamam a mesma função**. *Uma lista escrita à mão ao lado de um
  predicado é duas respostas à mesma pergunta, e a que o artista vê é a que envelhece.*
  ✅ **OS SINAIS (§8.10) EXISTEM** (2026-08-23) — e saem pelo **outbox** do `ph2d-runtime`, não
  pelo ActionBus que a spec desenha: ela é anterior ao ADR-0143, e um sinal no bus faria o motor de
  animação **chamar** o editor. ⚠️ **Dois nomes AUTORADOS na tag** (`signal_on_finish` /
  `signal_on_loop`), vazio = **calada** — é a lei dos contatos da física: acabar e dar a volta
  distinguem-se por serem nomes diferentes, não por um campo de fase. ⚠️ Um tique atrasado colapsa
  num sinal só, **com a contagem dentro** (`SignalOrigin::Animation::cycles`) — dez passos que
  ninguém deu é ruído. ⚠️ **A pré-visualização é MUDA**: pegar no pincel não pode tocar um som.
  ⛔ Dos quatro eventos da spec, dois ficam fora **com motivo medido**: o `FrameChanged` por
  FREQUÊNCIA (~12×/s por sprite, e quem o consome já lê o `Sprite::frame`) e o `AnimationChanged`
  por NATUREZA (é um clique no Inspector, não um facto da cena). Detalhe:
  [spec 08, secção final](docs/Sprite_projeto/08_animation_inline.md) ·
  ✅ **uma âncora já MOVE coisas** (2026-08-22): `ph2d_ecs::AnchorMount` faz dela um
  **QUADRO na hierarquia** ([ADR-0072-amendment-1](docs/architecture/decisions/0072-amendment-1.md)),
  autorado pela linha «Rides Parent Anchor» da §12 e demonstrado em `PH2D_MOUNT_SMOKE`.
  **Escolher uma âncora POUSA o objeto nela** (só a posição — o ângulo é do filho; e **nunca** ao
  escolher «—», porque desmontar é largar), com «Reset to Anchor» a refazê-lo; a âncora montada
  fica **visível ao mexer no filho, mesmo com a §12 fechada**; e o dono tem duas caixas —
  **«Always show anchors»** (viva) e **«Show anchors at runtime»** (⛔ grava e **não tem quem a
  leia**: não há modo de jogo, o `shells/game`/R1 está adiado).
  ⚠️ **A precedência do overlay é `Editing` > `AlwaysVisible` sobre a MESMA entidade** — o modo de
  edição é *superset* (as mesmas âncoras, mais o realce e as alças). ⛔ Ao contrário, a caixa
  **rouba o destaque à selecionada**, e o gate afirmava-o: *um gate verde pode pinar um defeito de
  produto*, e os três desta linha foram apanhados por smoke.
  ⚠️ **A lei entra nas DUAS travessias de mundo pela MESMA função** (`mount_state`) —
  `propagate_transforms` e `world_transform`: só numa, a espada **desenha** na mão e todo gesto
  agarra-a na origem do pai. ⛔ Das três superfícies do ADR-0072 §2.6 só a **Rust** tinha onde
  existir; **Luau e MCP estão BLOQUEADOS por outro subsistema** (o `ScriptHost` do desktop corre um
  script placeholder e **nunca** recebe `provide_read`; o `McpHost` é um `MemoryHost` de JSON, com
  «backends reais em S2/S3» escrito nele) — o gatilho de cada uma está na
  [spec §7.8-bis](docs/Sprite_projeto/07_named_anchors.md), e construí-las hoje repetiria, um nível
  acima, o defeito que esta wave curou · o `AnchorData::user_data` não tem UI, com o `variant_editor`
  órfão a apontar-lhe · os 4 goldens seguem `unimplemented!()` (falta o arnês headless).
  ✅ A UI de Save/Open **existe** desde 2026-08-23 (ver *Editor / shell*).
  **Smokes:** `PH2D_SLICE_SMOKE=1..3` · `PH2D_SOCKET_SMOKE` · `PH2D_MOUNT_SMOKE` · `PH2D_ANIM_SMOKE` ·
  `PH2D_ASE_SMOKE` ·
  `PH2D_SHEET_SMOKE` · `PH2D_EMISSIVE_SMOKE` · `PH2D_DITHER_SMOKE`.
  **Ler:** ⚠️ [auditoria de 7 lentes](docs/Sprite_projeto/20_auditoria_do_inspector_2026-08-21.md)
  (o que estava morto/incompleto, **com o que já foi curado marcado**) ·
  ⚠️ [auditoria da §11 Animation](docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)
  (aplicada; 11 achados, 11 mutações, e **5 recusas medidas** — leia-as antes de propor um alcance
  de campo ou de mexer no que a lista faz ao clique) · [spec](docs/Sprite_projeto/README.md) ·
  [handoffs](docs/Sprite_projeto/handoffs/README.md) (⚠️ índice **à mão**: esta pasta não entra no
  `doc-index.sh` porque o `README.md` acima dela é a spec — e até 2026-08-23 os handoffs eram
  **órfãos**, citados por nada)

## §6 — Contratos congelados (mexer = Coord-only + ADR; DIRETRIZ §4)

- **Nodes** ([ADR-0039](docs/architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)): `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — gate `architecture_contract_surface`.
- **Tools** ([ADR-0040](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md)+[0041](docs/architecture/decisions/0041-rasteredit-rename-and-deactivate.md)): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` — gate `architecture_tool_contract_surface`. (`Tool` 10→11 em [ADR-0040-amendment-2](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md): `on_tick` heartbeat p/ aquarela live, ADR-0049/0077-D11. `Tool` 11→12 + sub-trait `CanvasPaintTool` em ADR-0040-amendment-3: `as_canvas_paint_mut`/`on_canvas_pointer` p/ entrega de ponteiro de canvas ao novo Painter, `docs/Painter/`.)
- ~~**Painter (pintura)** (ADR-0043..0053)~~ — **REVOGADO** por [ADR-0099](docs/architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md): os ABIs de pintura (`PainterUiEdit`/`Brush`/`Stamp=96B`/`RenderingMode=6`/`PointerSource`/`DeviceTier`…) e o gate `architecture_painter_contract_surface` (crate `ph2d-painter-contracts`) foram **removidos** junto com a pintura. A superfície de **efeitos** que sobrevive (`AdjustmentKind≤32`/`AdjustmentParams`/`BlendMode`+`MAX_BLEND_MODES`/`apply_blend`) vive agora em **`ph2d-painter-effects`** (não-gateada; re-capear é follow-up). `ColorProfile` vive em **`ph2d-imageio`** ([`color.rs`](crates/ph2d-imageio/src/color.rs), com gate `architecture_imageio_contract_surface`) — ⚠️ **não** em `ph2d-color`, que nunca o teve.
- ~~**Watercolor (física)** (ADR-0049/0078-0084)~~ — **REVOGADO** por [ADR-0096](docs/architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md): a sim de aquarela e seus gates (`gpu_parity`/`composite_parity`) foram removidos junto com a crate `ph2d-painter-wash`. O modelo K–M espectral é histórico (backup); o pivot para mixer-brush usa Kubelka–Munk/Mixbox no blend do pigmento, não shallow-water. Nada congelado aqui.
- **Vector (data-model foundational)** ([ADR-0056..0068](docs/architecture/decisions/)): `VectorOp≤16`/`Vertex`SmallVec32/`Segment`64/`Region.segments`16/`AnimValue` enum/`sample(t:f64)`/`MAX_SPIRAL_TURNS=64`/`MAX_POLYGON_SIDES=128`/`MAX_VERTICES_PER_LLM_GEN=1000` — gate `architecture_vector_contract_surface` (escaneia só `ph2d-vector-doc`+`-traits`). **PERMANECE congelado** e o gate FICA — mesmo após [ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md) ter **retirado** as tools/nodes/panels de edição vetorial (o cutover mexe só em crates satélite, não na superfície do doc). O **motor novo** (`ph2d-vec-*`, §5) tem contrato **próprio, ainda NÃO congelado** (re-congelar é follow-up). Gate `vello_kurbo_only_in_ph2d_vector` nunca existiu (era W2-deferred).

## §7 — Design system

[`docs/design/PROMPT_CLAUDE_DESIGN.md`](docs/design/PROMPT_CLAUDE_DESIGN.md) (brief: tokens.json + mockups + icons + specs) alimenta os widgets em Vello sobre [`ph2d-editor`](crates/ph2d-editor/) (ADR-0023). Mockup de referência: [`docs/design/component-library.html`](docs/design/component-library.html).

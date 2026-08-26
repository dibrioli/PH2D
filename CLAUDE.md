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
  a família `flip_smooth::resample_measurement::precisao::orcamento` — **3 testes** em
  [`flip_fit_budget_tests.rs`](shells/desktop/src/flip_fit_budget_tests.rs), medida 22/08 pela
  `line/3DModeling` e confirmada 23/08 pela `line/sculpt3d`, com a falha a MUDAR de teste entre
  corridas · `the_cost_of_sampling_a_path_is_flat_in_its_anchors` (Timeline) ·
  `the_region_refresh_is_bound_by_the_footprint_not_by_the_mesh`
  ([`ph2d-mesh`](crates/ph2d-mesh/tests/measure_normals.rs) — ⚠️ o doc-comment declara-se imune,
  *«o gate é a FORMA, não o relógio»*, e a forma é medida DIVIDINDO dois relógios: *um gate que se
  diz independente do relógio ainda o é, se o numerador e o denominador forem tempos*) ·
  `measure_brush_kernel` ([`ph2d-sculpt3d`](crates/ph2d-sculpt3d/tests/measure_brush_kernel.rs) —
  cara, 34 s sozinha, no pico do fan-out por construção) · e ⚠️ **as duas de ALOCAÇÃO**, espécie
  própria: `apply_from_doc_is_zero_alloc_steady_state` (ph2d-timeline) e
  `the_trusted_len_collect_allocates_once` (ph2d-audio-edit) — um contador de alocações parece
  imune a carga e não é: sob fan-out o alocador global reutiliza arenas de outra maneira.
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
  **Aberto:** ⭐ **33 P2, ZERO P1 e ZERO P0** na conferência — ⚠️ o placar é **DERIVADO** por [`placar_conferencia.py`](docs/Motion%20Nodes/ferramentas/placar_conferencia.py) e **envelhece a cada wave**: rode-o antes de citar o número (esta linha já disse `68` com a conferência em `33`, porque a §5 se edita na INTEGRAÇÃO e uma integração que a salte deixa o roteador a mentir por 2×). **Onze das dezassete folhas sem P2**; sobram **15** (9), **06** (7), **17** (6), **08** (5), **13** (5) e **11** (1) — ~20 dos P2 são *obras*, não knobs. Cenas `=90..=95` · ⏳ **[Bug #7](docs/Motion%20Nodes/BUGS_motion_nodes.md) ABERTO** (report do Enio, adiado por ele): na cena `=95` a fileira de mar de **4 ondas não mostra cristas diferentes** — a BOIA é um **passa-baixo** e apaga as camadas finas (excursão vertical `0,228` contra `0,377` da de 1 onda). A alavanca medida é o **calado**, ⛔ **não** a densidade, que reabre a armadilha do Bug #6 · ⚠️ **os KNOBS MORTOS foram caçados e os 19 curados** — [doc 90](docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md) tem a tabela verificada, os 8 pontos cegos da sonda e o que ficou de fora (a porta por-elemento lida no elemento 0, o `falloff` que o `motion.kaleidoscope` ignora, os knobs vivos e inalcançáveis pela UI); ⛔ **consulte-o antes de acusar um param de morto** · ⚠️ **os TETOS foram medidos e o bloco Z fechou 7 células** — [doc 91](docs/Motion%20Nodes/91_os_tetos_que_ninguem_mediu.md): 25 params passam a ter teto digitável DERIVADO do `step` do slider (o `f32` é o recurso), o `sim.spawn::rate` sobe de 60 para **15 360/s** (a lei, medida pela porta do produto), e o `MAX_DT` dos DOIS integradores desce de `0,1`/`0,05` para **`0,03`** — a `0,1` o laço fechado atirava uma grelha a **127×** o raio em que nascia. ⛔ O `motion.spring` fica fora (ele deriva 3 tetos do dele); ⏳ `motion.boids` e `motion.wave` seguem por medir · ⚠️ **a folha 11 (fx raster) FECHOU menos uma célula**: o `fx.drop_shadow` ganhou o MODO da sombra (a coluna por-linha já existia — faltava um param), o `fx.rgb_split` ganhou o EIXO e o RAIO LIMPO (⚠️ a rota por `falloff` modula o ALFA, não o deslocamento), e o `fx.glow` ganhou `Operation` (`Screen`, que é um par de fatores de mistura; ⛔ o `Multiply` do AE **escurece** e o halo compõe-se sem z, com gate a proibi-lo pelo nome), `Glow Based On` (`Alpha` faz uma silhueta PRETA acender pela cobertura) e a **rampa de cor** — ⚠️ cuja régua se corrigiu **duas** vezes: uma grelha uniforme não representa a esquina de uma parada, e num degrau o que encolhe com a densidade é a LARGURA da banda errada, não a altura. Cena **`=84`**. ⚠️ **E a folha 06 (animadores) fechou mais QUATRO células e meia, com uma pergunta só — *o animador não sabia a FORMA que o artista desenha, nem escrever os DOIS eixos*:** a onda `Custom` do `motion.oscillator` e a ease `Custom` do `motion.stagger` (uma lei — a forma vive num **text param**, e vai ao device por **LUT de 512**, ⛔ nunca por `applicable: false` a derrubar o nó para a CPU; custo medido **0,018% de um quadro**), o canal **`Position XY`** do `motion.wiggle` (FECHA a célula) e do `motion.noise` (metade — falta o *Use Layer as Seed*), e o **`align = Normal`** do `motion.path`. ⚠️ **A `natural_range` tem de responder por toda forma nova** (a `Custom` é unipolar e a conta bipolar entregaria metade da faixa com o piso ao centro — a armadilha do `Spike` reaberta), e ⚠️ **a régua dos DOIS eixos é a CORRELAÇÃO, com a barra longe de `1` e não colada em `0`**: o resíduo é ruído de amostragem (cai de `0,120` para `0,009` só ao afinar o campo), então uma barra apertada mediria o tamanho da grelha. Cena **`=85`** + **`PH2D_MOTION_NODE_PATH_SMOKE=3`**. ⭐ **E o primeiro dos QUATRO P1 fechou, com um NÓ NOVO: o `motion.bezier_warp`** (folha 04) — a fronteira CURVA que o *Bezier Warp* do AE tem e o Corner Pin não pode ter. 4 cantos + 8 tangentes, interior por **patch de Coons**; o default é a identidade **ao bit** (as tangentes nascem nos terços, onde a cúbica degenera no segmento *por identidade polinomial*). ⚠️ **Ele NÃO é um param do `motion.four_point_warp`, e há gate a prová-lo:** com arestas rectas o Coons é o mapa **bilinear**, que concorda com a homografia nos quatro cantos e **arqueia** as rectas interiores — uma projectividade preserva rectas por definição. ⚠️ **O teto de linhas do painel foi de 20 para 24** (a 3ª vez por medição, e a folga foi RETIRADA: cada slot multiplica com o `MAX_ENUM_OPTIONS`), e o nó desenha **1083 px num dock de 880** — o gate do dock passa a NOMEAR a excepção com a altura, ⛔ e um segundo nome ali significa que a resposta virou **secções recolhíveis** no painel, que hoje não existem. ⚠️ **E o smoke dela devolveu um defeito de alcance MUITO maior, já curado ([Bug #4](docs/Motion%20Nodes/BUGS_motion_nodes.md)): o `Multiply` do renderer não desobedecia à alfa — ele a INVERTIA** (α=0 pintava **preto**, subir a alfa **clareava**), porque uma fonte **pré-multiplicada** codifica *"não contribui"* como **zero**, que é o neutro de todo modo **menos** o `Multiply`, cujo neutro é `1`. Vale para **toda sprite do app** com `BlendMode::Multiply` em alfa parcial — ⚠️ um golden de outra linha que a contenha muda de valor, e a mudança **é a cura** (em α=1 nada se move, e era **só** α=1 que o gate media). ⏳ Fica a *dirt texture*, com o preço agora medido (a textura de uma sprite é uma de TRÊS coisas, e só uma é um rectângulo no atlas partilhado) · ⭐ **E o SEGUNDO P1 fechou — a folha 03 (simulação) está a zero: o `motion.soft_body` deixou de ser obrigatoriamente um retângulo.** Porta **`shape`** (a ÚLTIMA do manifesto, para `anchor_x`/`anchor_y`/`state` ficarem nos índices 0/1/2); ligada, a nuvem que chega é a forma de repouso; vazia, a malha autorada. ⚠️ **A wave não é «uma porta»: é apagar `rows`/`cols` de TRÊS respostas que nunca foram sobre a grelha** — *quem é pino* (era `i < cols`, hoje a **aresta de cima do repouso**), *qual é o contorno* que a pressão defende (era o passeio do anel, hoje o **casco**) e *como o corpo se divide em regiões* (eram bandas de índice, hoje de **coordenada**). ⚠️ **A malha autorada dá os MESMOS BITS**, e não por promessa: ela é o seu próprio fornecedor das três, e cada uma devolve a **sequência de índices** que o código percorria à mão — o anel e as regiões alimentam somas em `f32`. ⭐ **O casco com os COLINEARES MANTIDOS reproduz o anel da grelha índice a índice** (um casco estrito daria 4 cantos, mesma área em aritmética exacta e **outra** em `f32`); ⚠️ não ao bit — quem entra pela porta tem de ser **re-centrado**, e o centroide somado de uma malha já centrada não é zero (`−1,19e-7`) ⇒ **2 ULP**. ⭐⭐ **E o smoke achou um defeito de PRODUTO:** com o pino a valer *o `y` máximo a menos de um epsilon*, um DISCO fica preso pelo **ponto mais alto, um só**, e balança como pêndulo (envergadura +74% em 2 s) — a lei que fica é **meia FILEIRA**, derivada, que numa malha reduz a `0..cols`. ⛔ **Fronteira NOMEADA:** um repouso **côncavo** tem o *envelope* defendido em vez da área (mais fraco, nunca invertido). + as três secções do painel (**Mesh / Physics / Pin**). Cena **`=87`** · ⭐ **E o TERCEIRO P1 fechou — o `motion.trail` deixa de LEMBRAR e passa a RE-COZINHAR** ([ADR-0163](docs/architecture/decisions/0163-a-node-may-cook-its-own-input-at-n-instants-a-time-fan.md)). A célula estava certa sobre o nó e o **substrato** é que mudou: um ring contém o passado *porque passado é o que um ring é*, e o que faltava era um nó poder cozinhar a **PRÓPRIA entrada em N instantes** — hoje `TimeFans` no `ph2d-nodegraph`, **ponto de extensão append-only** (nenhuma assinatura mexida; ⛔ um argumento novo no `advance_or_scrub_scoped` custaria **29 sítios de chamada**). O nó ganha **`Source`** (`Remembered` = o ring, o default, **byte-idêntico** · `Resampled`) e **`Forward Steps`**. ⚠️ **A lei das gerações vive numa função só** (`echo_offsets`), com dois leitores — o construtor dos mapas e o `eval`, que dela tira a IDADE; a escada escrita duas vezes poria o desenho num instante e a cor noutro. ⭐ **Destrava as outras três do `SUPERAR:` S1 de uma vez**: `length` sem tecto de memória, espaçamento não-uniforme, e o **scrub exacto sem `CheckpointRing`**. ⭐⭐ **A cauda re-cozida é a mais CERTA das duas:** o ring promove a cabeça a fantasma **periodicamente**, logo carrega até `spacing−1` tiques de erro de fase o tempo todo. ⚠️ **É um MODO, nunca uma substituição** — uma simulação não é função de `t`, e um leque sobre uma sub-árvore com `pre` é **RECUSADO**. ⚠️ **E uma afirmação minha ENCOLHEU por mutação:** o `push_scope` não compra «fatias no mesmo instante partilham a faixa» (seis gates ficaram verdes sem ele) — compra o instante repetido **fora de ordem**. Cena **`=88`** · ⭐ **As folhas 09 (cor) e 10 (field) FECHARAM**, e as três células eram a mesma forma — *um número onde a pergunta tem dois lados*: o `soft_angular` do `field.radial_sweep` (um **multiplicador**, porque a **cerca declarada** do nó — *«adimensional, as duas bordas vivem em unidades diferentes»* — escolheu a forma da cura), o `clamp` do `field.remap` como **enum de 4 estados no param que já existia** (⛔ um `clamp_max` apendado mudaria o sentido de `Clamp = 0` em toda cena salva, e um gate que já existia disse-o), e a **interpolação por STOP** do `motion.color_ramp` (geração **`g4`** do formato, com `STOP_INTERP_GLOBAL = 255` — ⛔ **não** `RampInterp::COUNT`, que anda por cima do primeiro modo novo). ⚠️ **E o portão de fecho da workspace apanhou QUATRO vermelhos, um deles vermelho havia um bloco inteiro**: a rolagem do painel deixou de ser inerte quando o teto de linhas subiu, e a cura é ***uma banda, dois consumidores*** — o `HitIndex::push_clip` já existia e este painel não o chamava. ⚠️ **Três dos quatro caíram POR CAUSA da cura:** eles mediam o fundo do último hit-rect, que com a blindagem **satura na janela** — o retrato nomeado do `motion.bezier_warp` foi de `1083` para **969** sem uma linha de produto se mexer (114 px é a faixa do título). *Blindar o hit-index muda o que toda sonda mede.* · ⭐⭐⭐ **E o ÚLTIMO P1 FECHOU — a conferência não tem mais nenhum.** O `motion.emitter` ganha **`Emitter Motion`** (`Carry` = o penacho de sempre **ao bit**, e ⚠️ **não é um bug com nome bonito** — um efeito ANEXADO quer isto · `Leave` = a partícula fica onde nasceu · `Inherit` = e leva a velocidade da fonte) + **`Inherit Strength`** gateado ao modo que o lê. ⚠️ **A recusa DISSOLVEU porque o substrato mudou, e quem o mudou foi esta linha três blocos antes** (§0.0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*) — a terceira saída, **re-cozinhar** a origem, não existia quando a nota foi escrita. ⭐⭐ **E a medição achou a metade que vinha ANTES da célula:** o `P` do emissor é a posição de NASCIMENTO e era a origem de AGORA para toda partícula ⇒ **arrastar o emissor arrastava o penacho inteiro**. ⚠️ **A resolução da história é uma TAXA (240 Hz) e não uma contagem** — uma contagem repartida pela vida pioraria ao alongá-la —, com tecto **MEDIDO em 1024** (uma fatia custa 300-490 ns ⇒ 2,6% de um quadro; 2048 seria 5,5%, que é onde *fácil de usar* deixa de tolerar um knob opcional). ⛔ Os modos novos são **CPU-only**, com o bloqueador nomeado. ⚠️ **E o leque tinha um defeito que só uma FONTE revelava**: ele contava as fatias da PORTA, então um nó sem portas lia **zero** com o leque cheio (529 amostras ignoradas em silêncio) — e o gate não o via porque contava o trabalho FEITO em vez do RECEBIDO. Cena **`=89`** · ⚠️ **o `blend` é uma COLUNA por LINHA**
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
  `PH2D_MOTION_OBJ_SMOKE` · `PH2D_MOTION_NODE_PATH_SMOKE=1|2` (⚠️ o `=2` é um **modo** de uma env que já existia,
  não um nível novo do roteador de `GPU_COOK_DEMO`: um nó que anda numa forma **desenhada** precisa do documento
  vetorial, que só aquele smoke encena).
  **Ler:** [índice do módulo](docs/Motion%20Nodes/README.md) · ⚠️ [`BUGS_motion_nodes.md`](docs/Motion%20Nodes/BUGS_motion_nodes.md)
  (**o único `BUGS_*` que esta seção não listava**, e foi lido 22×) · [as 17 folhas](docs/Motion%20Nodes/89_conferencia/README.md)
  — a **folha 03** é [`03_simulacao.md`](docs/Motion%20Nodes/89_conferencia/03_simulacao.md) · ⚠️ **o PLANO da próxima janela** está no [handoff de continuação](docs/Motion%20Nodes/handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-08-19.md) (os grupos seguintes, e as dez leis que a linha pagou) · [handoffs](docs/Motion%20Nodes/handoffs/README.md) ·
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
  **(2)** a *state machine* do **Morph** **no canvas 2D** (setas forma→forma, condições nas setas), viva no **runtime do jogo**
  ([pesquisa 31](docs/Vector%20Module/31_pesquisa_maquinas_de_estado.md) — base **Rive**, com duas correções: *só as transições do
  estado CORRENTE* (State Tree) e *todo input sabe quem o lê* (a cura do medo do Animator)). ⚠️ O [`VecMorph`](crates/ph2d-ecs/src/vec_morph.rs)
  **já** é não-destrutivo e re-cozido por quadro, mas é entre **DUAS** formas; e *"no runtime"* obriga a decidir se a lei desce a uma
  **crate-folha** ou se o **R1 sai do gelo** — é o item que muda o preço ·
  **(3)** **Texture pattern** no preenchimento: o `Paint` tem **4** variantes e **nenhuma de imagem**
  ([`paint.rs`](crates/ph2d-vec-scene/src/paint.rs)) — ⚠️ a lei do módulo é *preenchimento em **world-space**, que transforma
  com o path*, e ⛔ leia o [plano 23](docs/Vector%20Module/23_plano_pattern_along_path.md) antes de desenhar ·
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
  ⭐⭐⭐ **E a EXTRACÇÃO existe** (2026-08-24, clean-room dos papers sob [ADR-0167](docs/architecture/decisions/0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md); espec, atestados e fixtures em [`docs/3D/cleanroom/`](docs/3D/cleanroom/)): `ph2d-quadextract` extrai a malha das **isolinhas inteiras** de um mapa de grade inteira, e `ph2d_gridmap::round` (G5) faz o **arredondamento misto-inteiro** que ela exige. ⭐ **Sem biblioteca de precisão múltipla e sem epsilon** — a truncagem numa grade *global* põe o domínio saneado em `i64` e a orientação vira um determinante `i128` exacto. Nos dois mapas de referência: **100 % de quads e `χ` preservado** (toro de género 1 e casca fechada), e uma peça com **bordo** resolvida **sem oráculo**. ⛔ **Shipa DESLIGADO** (`PH2D_RETOPO_EXTRACT=1`), e por medição: na cadeia da casa a **forma** bate a barra do oráculo (aspecto `1,10`, enviesamento **`6,8°`** contra `4,8–7,1°`) mas a topologia não fecha (`χ = −5`) — ⚠️ **e a causa está medida a MONTANTE**: o solver contínuo do G3 **PESA** a costura (`SEAM_WEIGHT`) em vez de a eliminar, e a espec já nomeia a cura (§5.1: as costuras entram por **eliminação de variável**, não por penalização). ⛔⛔ **A 1ª redacção desta linha atribuía a causa às DOBRAS do mapa, e o R-pós refutou-a com a peça que importa:** na esfera fina o mapa tem **`0,0 %` de dobras** e as translações são **exactamente** inteiras, e a extracção ainda deixa `32` arestas de bordo. ⭐⭐ *Duas grandezas estavam a ser lidas como uma*: o G5 torna a costura **INTEIRA** (`shift_frac_max = 0`, e há gate) e **não** a torna **FECHADA** (`seam_max` vai de `0,23` a **`1,00`** — uma célula inteira de rasgo, e **não existe gate sobre ela**). ⚠️ O rasgo é **invariante** às duas constantes da escada (`1,0369`–`1,0897` nas 9 combinações, com o degrau barato a variar de `1,4 %` a `100 %`) ⇒ **não é afinação.** Mecanismo, a tabela e as 5 divergências deliberadas: [handoff de 24/08 §8-bis](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md). ⭐⭐⭐ **O smoke do Enio (24/08) VALIDOU a direcção** — *«o melhor resultado conseguido até agora»*, *«obedece razoavelmente o relevo»* — e nas **peças dele** a forma está **dentro da barra do oráculo** (enrugada `1,15` · `5,7°` · `4` faces péssimas), com os buracos a serem **19 de 2 041 células** (`0,9 %`). As duas queixas dele (buracos · vincos) são o **MESMO mecanismo em falta**, e a obra seguinte está **especificada**: [`SPEC_restricoes_por_eliminacao.md`](docs/3D/cleanroom/SPEC_restricoes_por_eliminacao.md) — *uma restrição linear entra ELIMINANDO uma variável, nunca como termo de energia*; a costura é uma, a aresta de feição é outra. ⛔ **A costura primeiro** (a feição é a mesma maquinaria e herdaria o rasgo). ⭐⭐⭐ **E a OBRA A FECHOU (24/08, `line/seamelim`): a casca fecha.** A costura deixou de ser penalizada — `94 %` das ligações eliminam uma variável e as que fecham ciclo entram todas **num sistema linear só** (⛔ em dois subsistemas o par realimenta-se e diverge: esfera a `NaN`, toro a `6,4e17`, e amortecer **não** cura). Medido A/B em 5 peças fechadas do corpus: resíduo de costura `1,00` → **`0,000`**, arestas de bordo `30`–`78` → **`0`**, `χ` `−4`..`−13` → **`+2`**, e **3–4× mais rápido**. ⚠️ Fica UMA regressão nomeada — as faces com canto `>60°` sobem (`4`→`10` na enrugada), e a cura publicada é o *local stiffening* (§5.4 do mesmo *paper*), **fora daquela wave de propósito**. Shipa dentro de `PH2D_RETOPO_EXTRACT` (que já nasce desligado); `PH2D_GRIDMAP_WELD=0` bissecta. Mecanismo, as 5 recusas medidas e a pergunta devolvida sobre a barra do gate nº1 (`f64` da referência contra `f32` nosso): [handoff de 24/08](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md) + [auditoria](docs/3D/handoffs/AUDITORIA_line_seamelim_2026-08-24.md).
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
  ⭐⭐⭐ **A EXPORTAÇÃO CAIU DE 8 min 17 s PARA 6,4 s — 77× — E A MENSAGEM VOLTOU** (W62, dois
  reports do Enio em 24-25/08; o arquivo que sai é **o mesmo**, e os três níveis saem hoje
  **idênticos até à última casa**: `1,0794725` de aspecto, `6,417694°`, 2 539 quads). ⭐ *O alvo da cadeia de quads sai da CAIXA, nunca da
  densidade* (`target_edge = alpha · diagonal`) — então a grade fina era mastigada pela fase zero e
  **deitada fora depois de paga**: `1 120 158` faces custavam **`495 244 ms`** (97 % só no F1) para a
  **MESMA** resposta (`6,4°`, 2 436 quads) que a grade do `Draft` dá em **`4 613 ms`**. ⛔ E não é só
  preço: nas profundidades 7-8 a fidelidade medida **no campo** *piora* (`0,043 %` → `0,087 %` →
  `11,3 %` da diagonal) e a esfera é **destruída** (`55,5°`). ⇒ a cadeia come a grade do `Draft`
  (`meshes_for`) e a malha do NÍVEL fica se o veto recusar (`quads_or_keep_from`), e a fase zero
  passou a honrar o alvo que lhe dão (`phase_zero` — ela ignorava-o, e os números coincidiam **por
  acidente** com o único chamador de então). ⚠️ **E *"a mensagem não aparece"* era o mecanismo de
  22/08 com outra causa:** o relógio do chrome desconta o congelamento **declarado**, e quem
  congelava era uma **conta**, não um diálogo. ⛔⛔ **E o report SEGUINTE do mesmo dia mostrou que
  declarar cura a MENSAGEM e não cura o congelamento** (*«o linux fica cinza»*): a 12 s o loop não
  responde ao *ping* do compositor, o KDE dá a janela por morta e oferece **forçar o encerramento** —
  e o gesto natural a seguir leva o trabalho não gravado. ⇒ ⭐⭐⭐ **a exportação SAI da thread que
  desenha** ([`field3d_export_job.rs`](shells/desktop/src/field3d_export_job.rs)): bancada com uma de
  cada vez, recusa do segundo **em alto**, resposta drenada uma vez por quadro ao lado do pedido, e um
  sentinela que liberta a bancada no `Drop` (senão o 1.º estouro trancava o botão até ao fim da
  sessão). ⚠️ **O `Send + 'static` é o gate que o COMPILADOR escreve.** ⚠️ E a declaração foi
  **retirada** de `cook`/`bytes_of`: do lado de lá o `note_stall` escreve num `thread_local` que
  ninguém lê — *um no-op silencioso*. A porta do `modal` fica sendo o que sempre foi: a resposta certa
  para o **diálogo**. ⚠️ Doze segundos de silêncio com o app vivo leem-se como «o botão não fez
  nada» ⇒ há aviso de início, e ele **não promete prazo**. ⭐ **E tirar o trabalho da thread ABRIU uma
  porta que o congelamento fechava:** o artista voltou a poder fechar o app a meio, e um `write`
  interrompido deixa **meio arquivo com o nome certo** — daí a gravação por temporário + `rename`,
  com o temporário **na pasta do destino** (o `rename` só é atómico dentro do mesmo sistema de
  arquivos). *Uma cura pode abrir a porta que outra fechava.*
  **Aberto:** ⏳ **decisão do Enio, já com os números:** o nível de exportação **não alcança** a
  densidade da cadeia (ela dá ~2 500 quads em qualquer nível) — manter a razão `célula/alvo` preserva
  a qualidade (`6,2°`) e custa **`48 894 ms` por 3,7× os quads**, e mais um degrau seria minutos ·
  ⏸️ **o custo que sobra é 71 % do `ph2d-gridmap`** (`3 322` dos `4 677 ms` da cadeia estão no G3/G5,
  medido em `max`) — crate de outra linha ·
  ⛔ **segundo reprodutor do panic do `ph2d-gridmap`** (`solve.rs:336`, *"len is 74, index 130"*):
  um alvo **grosso** sobre uma `uv_sphere(48,32)` — a `line/quadextract` é a dona · ⏸️ o traçado ficou **~2,4× mais caro** desde a W3 e ninguém o reconferiu (suspeito
  nomeado: o anti-serrilhado adaptativo) · o teto de `Resolution` (16) foi derivado com o custo
  **antigo** e a tabela dele foi medida a `load ≈ 4,7` · ⏸️ ladrilhar em `(u, v)` contra o
  **paralelogramo** em vez da AABB (o único eixo que não multiplica a montagem de JIT) · a
  composição de dois `Exact` encadeados e o gradiente de uma **escultura** ficam no passo curto sem
  ninguém os ter medido · ⏸️ um laço que **SUBTRAI** (pede decisão: aqui `Shift` e `Ctrl` são a
  mesma tecla) · vários `VecPath` **separados** numa peça só · religar uma escultura que mudou de
  sítio (pede UI) · ⛔ o vínculo à escultura **viva** do módulo 3D
  foi **medido e recusado** (voxelizar custa 229–389 ms a 128³ contra um quadro de 16,7) — a
  escultura entra da cena **sem disco** e não se atualiza sozinha · ⏸️ o `Mirror` não se consegue
  demonstrar (adiado pelo Enio) · a exportação não diz **onde** a peça está (o tamanho já diz, W36) ·
  nada mostra na Hierarquia que há um **isolamento** em curso.
  **Smokes:** pill **MODEL** · `PH2D_FIELD_SMOKE=<n>` (o roteador é
  [`field3d_smoke_scenes.rs`](shells/desktop/src/field3d_smoke_scenes.rs)).
  ⚠️ **Preferência fora do repo:** `~/.ph2d/prefs.txt` — um `reduced_motion=1` esquecido reprova
  smokes sobre produto correto **em todo o resto do app**, e a viagem entre vistas é a excepção.
  **Ler:** [`docs/3DModeling/`](docs/3DModeling/) ·
  [`06_resultados_cena_e_gizmo.md`](docs/3DModeling/06_resultados_cena_e_gizmo.md) §1–§56 (uma seção
  por wave, com a tabela medida e as provas de mutação; o **§13** é a lista viva do que está aberto) ·
  [handoffs](docs/3DModeling/handoffs/README.md)
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
  **Aberto:** ⚠️⚠️ **a F1 está PELA METADE, e foi integrada assim por decisão do Enio (24/08):** a física já aponta por
  identidade (renomear um corpo **não** solta mais a junta), mas a **timeline ainda não** — renomear um objeto animado
  continua a desligar o binding, e nada na tela explica a diferença. Falta a outra metade do passo 5 (`stable_name_id` da
  timeline) e o corte da Sprite (F1.6) · ⛔ **o `physics_ecs_c9` está POR RE-CAPTURAR** — o `deterministic_hash` muda de
  valor com o snapshot v2, e é o item mais provável de reprovar a matriz 3-OS no próximo ship · F2-F8 do
  [plano vivo](docs/Components/05_plano_de_implementacao.md).
  **Smokes:** abrir um `.ph2dproj` gravado ANTES de 24/08 (tem de dizer *"Project migrated from format 95 to 97"*) ·
  reordenar irmãos na Hierarquia + Ctrl+Z · renomear um corpo com junta (`PH2D_PHYSICS_SMOKE=6` ou `=67`) · copiar um
  ragdoll e dar Play.
  **Ler:** [`docs/Components/`](docs/Components/) ·
  [handoffs](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_F0_F1parcial_2026-08-24.md) (⚠️ o §9 lista
  **cinco** coisas que uma leitura rápida do diff entende ao contrário, e o §10 as **três** premissas do plano que a
  implementação refutou)

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

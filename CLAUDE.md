# CLAUDE.md — núcleo operacional do PH2D (LEIA INTEIRO — é curto de propósito)

> Toda LLM recebe este arquivo automaticamente — cada agente, cada subagente, cada worktree, **a cada
> passo**. Ele é o **roteador**: os inegociáveis + para onde ir por tarefa. Detalhe técnico →
> [`SKILL_Stack_PH2D_Definitiva.md`](SKILL_Stack_PH2D_Definitiva.md) (HR-1..HR-18). Processo →
> [`DIRETRIZ.md`](docs/IntegracaoMultiAgente/DIRETRIZ.md). Não leia esses dois inteiros — use o §1.
>
> ⛔⛔ **Tecto: 40 KB, com gate** (`architecture_claude_md_cabe_no_orcamento`). Medido 2026-10-02: este
> arquivo chegou a **710 KB** (era 58 KB em 19/08), punha **~330 mil tokens** no início de toda janela
> e era **~45 % da conta** do Claude — 82 % do custo é RELER o contexto, e ele era relido a cada passo.
> Com mais de 200 mil tokens nenhum subagente de modelo menor conseguia sequer começar. A história
> que aqui esteve vive **verbatim** em [`docs/archive/estado-2026-10-02/`](docs/archive/estado-2026-10-02/README.md).

## §0 — Inegociáveis (memorize os 10)

0. **O alvo é o EXTRAORDINÁRIO e o teto é o do HARDWARE, nunca o do caminho lento.** Antes de escrever
   qualquer limite (`MAX_*`, faixa de slider, «por ora»), **MEÇA** e escreva o número da medição com a
   tabela ao lado. Um limite legítimo diz **de que recurso** é. O fallback (CPU) só tem de dar a mesma
   resposta — quem manda no teto é o dispositivo. Quem move um número que tornava algo «inalcançável»
   **reconfere a nota**. Não é licença para otimização prematura: é medir antes de limitar.
1. **Norte ([ADR-0075](docs/architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)):**
   monorepo Rust único; desacoplar por **ECS** (components + events/resources; systems não se chamam),
   **NÃO** por plugin runtime nem WASM. Feature nova = **drop-crate**.
2. **Isolamento.** **Modo C** (shared tree): precisou de algo fora da sua pasta → **PARE e reporte ao
   Coordenador**. **Modo L** (worktree, [ADR-0107](docs/architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)):
   foundational você PODE tocar (append-only, projetado para isolamento); **PARE e reporte ao Enio** só
   em **contrato congelado** (§6) ou **rebase a conflitar fora dos seus arquivos**. A linha **fecha,
   escreve o handoff e PARA** — não integra nem faz ship sozinha.
3. **UI canônica:** zero hex, zero `f32` literal de UI, zero string hardcoded — tokens / i18n (HR-15).
4. **Git anti-colisão (Modo C):** `git add -- <seus paths>` (NUNCA `-A`/`.`/`stash`);
   `git commit --no-verify -m "…" -- <paths>`; `M`/`??` alheio ⇒ não comite, reporte.
5. **Velocidade e custo (§2):** inner loop = `check`; teste/clippy/auditoria **1× no fecho**.
   Concorrência é função do hardware (`bash scripts/hw-profile.sh`: `workstation` = Modo L,
   `constrained` = Modo C, [ADR-0106](docs/architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)).
6. **Padrão-ouro sem custo:** a melhor opção técnica vence custo de build/cronograma; gaps in-scope
   fecham na sessão atual ([feedback](project-memory/feedback_perfection_no_deferrals.md)).
7. **Push é 1× por jornada e NUNCA seu por conta própria.** Integração e ship **só por ordem EXPLÍCITA
   do Enio**, via um agente integrador com o handoff de cada linha (DIRETRIZ §1.5.3–1.5.4).
8. **O Enio é o DONO do produto, não um engenheiro.** Ele decide e testa; o smoke é onde ele APRENDE a
   ferramenta. Toda resposta a ele: **curta, sem jargão** (nada de gate, crate, ADR, nome de função
   salvo se ele pedir) — diga o que ele consegue FAZER agora. **Smoke em PASSOS NUMERADOS:** (1) o
   comando completo com o `cd` · (2) onde clicar, com o nome que aparece na tela · (3) o que tem de
   acontecer · (4) como saber que deu errado. ⛔ **No Motion, a cena do smoke usa FORMAS (`shapes`) e
   SIMULAÇÃO com campos** — fotografe-a antes de mandar (`docs/Components/ferramentas/fotografa_cena.sh`).
   ⚠️ Handoff, ADR, doc-comment e commit continuam técnicos — o leitor deles é a próxima LLM.
9. **Outro app JÁ FAZ isto? É um ORÁCULO que se CORRE, nunca um fonte que se lê.** Corra-o por script
   sobre entradas NOSSAS, grave a saída como fixture com cabeçalho, cada corrida vira gate, e compare
   **por passo**. **Passo 1 é a triagem de licença por ARTEFACTO instalado** (`pacman -Qi` em cada
   biblioteca) — há portas permissivas abertas (Godot MIT · OpenToonz BSD-3 · `libmypaint` ISC).
   Método, arsenal e recusas: [`docs/_ComoInvestigarApps/`](docs/_ComoInvestigarApps/README.md).

## §1 — Roteador leia-por-tarefa (leia SÓ o que sua tarefa exige)

> **A cada passo de QUALQUER implementação: [DIRETIVA_IMPLEMENTACAO.md](docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).**
> Verde-de-compilação é velocidade; no audit vale ZERO.

| Sua tarefa | Leia ISTO |
|---|---|
| **Tool ou node nova** | DIRETRIZ §2 + §3.A + [examples-fan-out.md](docs/IntegracaoMultiAgente/examples-fan-out.md) |
| **Painel / widget / chrome** | DIRETRIZ §3.B |
| **Modificar feature existente** | DIRETRIZ §3.D |
| **Foundational** | Modo L: [ADR-0107](docs/architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md) · Modo C / contrato congelado: DIRETRIZ §3.C + §4 |
| **Abrir / assumir / fechar uma linha** | [MODELO_ABERTURA_LINHA.md](docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) (`/pd-linha-abrir`) · [MODELO_TROCA_DE_AGENTE_NA_LINHA.md](docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md) (`cd` + `pwd` + `git branch --show-current` ANTES de ler) · DIRETRIZ §1.5.9 (`/pd-linha-fechar`) |
| **Integrar (só por ordem do Enio)** | `/pd-integracao` + DIRETRIZ §1.5.3 — `collision-surface.sh` pelo caminho ABSOLUTO em cada worktree; a coluna `base:` é o merge-base, leia o `main` no ficheiro; o degrau do schema reconta-se por `python3 scripts/schema-recount.py` |
| **Rodar uma jornada Modo L** | [GUIA_JORNADA_MODO_L.md](docs/IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) |
| **Mover código da shell para uma crate** | [HOWTO_partir_uma_familia_da_shell.md](docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md) — leia a §2 antes do primeiro ficheiro |
| **Stack / Hard Rule** | SKILL_Stack §HR-1..18 · versões: [STACK_VERSOES.md](docs/IntegracaoMultiAgente/STACK_VERSOES.md) · «dá para atualizar X?» → `bash scripts/stack-audit.sh --tetos` primeiro |
| **Física** | [ADR-0131](docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) · [tracker](docs/Physics/handoffs/HANDOFF_line_physics.md) · [BUGS](docs/Physics/BUGS_physics.md) |
| **Máquina lenta · comando pesado** | [TETOS_DE_RECURSO_POR_LINHA.md](docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md) |
| **Teste que reprova sob carga** | [FLAKES_DE_CARGA.md](docs/DevOps/FLAKES_DE_CARGA.md) |
| **Fim de dia · disco cheio** | [DIRETIVA_FIM_DE_DIA.md](docs/IntegracaoMultiAgente/DIRETIVA_FIM_DE_DIA.md) — primeiro `bash scripts/btrfs-health.sh` |
| **Que agente usar** | §2 (agentes) · [`.claude/agents/`](.claude/agents/) |
| **Quem é o Enio / memória** | [project-memory/MEMORY.md](project-memory/MEMORY.md) |
| **Quem possui o quê agora** | Modo L: `git worktree list` |
| **Achar um ADR** | [decisions/README.md](docs/architecture/decisions/README.md) |
| **Outro app já faz isto** | [`docs/_ComoInvestigarApps/`](docs/_ComoInvestigarApps/README.md) |
| **Código RESTRITO (GPL/proprietário)** | [SKILL_Cleanroom_Reimplementacao.md](docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md) (`/pd-cleanroom`) |

> Os 15 comandos de fluxo vivem em [`.claude/commands/`](.claude/commands/) (`/pd-*`) — cada um é o
> protocolo de uma linha desta tabela já destilado. Ferramenta que nenhum passo obrigatório chama pelo
> NOME não é usada (medido): por isso os comandos as nomeiam.

## §2 — Velocidade e custo

**O custo é RELER.** Cada passo (cada chamada de ferramenta) relê o contexto inteiro: 82 % do custo
medido em set/2026, com **606 mil tokens por passo** em média. O custo de uma sessão ≈ *passos ×
contexto*. As alavancas, por ordem:

- **Contexto pequeno.** Este arquivo ≤ 40 KB (gate). **Uma janela nova por onda de trabalho** — uma
  sessão de milhares de passos paga a mesma releitura milhares de vezes e compacta (medido em 02/10:
  mediana de 1 691 respostas por sessão, 2 014 compactações num mês). Ao mudar de assunto, abra janela
  nova. ⛔ Não ponha `model:` num comando `/pd-*`: trocar o modelo da janela reescreve o cache do
  contexto inteiro, na ida e na volta — delegar a um agente é o que fica barato.
- **Delegue pelo tipo de trabalho** ([`.claude/agents/`](.claude/agents/), cada um com o seu modelo):
  `explorador` (procurar/ler em muitos arquivos — devolve só a conclusão) · `verificador` (compilar,
  testes, portões — devolve só as falhas) · `integrador` (rebase, conflitos, recontagens) ·
  `documentador` (handoff, linha do §5, memória) · `mutacao` (provas de mutação). A janela principal
  decide e desenha; o mecânico vai para quem é mais barato. Delegue também para trabalhar em paralelo.
- **UM PASSO, N CHAMADAS.** Chamadas independentes (ler 3 arquivos · 2 greps · 2 medições) vão na
  **mesma** mensagem. Medido em 02/10: só 5 % dos passos fazem mais de uma coisa.
- **EDITE pela ferramenta `Edit`**, não por `python3`/`sed` — um `replace()` que não casa é no-op
  SILENCIOSO; o `Edit` falha alto. Script só onde é a forma certa (mutação, renomear em N arquivos),
  sempre com `assert` de contagem.
- **Inner loop = `bash scripts/cargo-check-narrow.sh <crate>`.** O teste não responde «a minha edição
  entrou?». **Corrida dirigida = `bash scripts/cargo-test-narrow.sh <crate>`** (corre `check` na frente;
  exit `0` verde · `1` vermelho · `2` não compilou). Medido em 02/10: `cargo test` ainda corre **~3×**
  mais que `check` (alvo ≤ 1), e é a maior parte do relógio de ferramentas.
- ⛔ **Comando pesado vai por `bash scripts/ph2d-run.sh <cmd>`** (fatia da LINHA: CPU ≤ 50 %, RAM ≤ 24G,
  prazo 30 min; `PH2D_GPU=1` = exclusão na placa). O guarda recusa vigia de fundo sem prazo.
  Tectos e porquê: [TETOS_DE_RECURSO_POR_LINHA.md](docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md).
- **Gate batched no fim do módulo:** `scripts/nextest-impacted.sh` + clippy `--all-targets` + auditoria,
  **1× sobre o diff acumulado**. Ao fechar a linha: `bash scripts/agent-loop-profile.sh` (as réguas
  deste §2, com o alvo de cada uma) e `rm -rf target/*/incremental`.
- **Comentário de código curto.** O porquê histórico (o que foi medido, o que foi recusado) vive no doc
  ou no handoff, com link; o código diz a lei. Medido: 29 % das linhas `.rs` são comentário, e cada
  leitura de código paga por elas.
- ⛔ **Código de família vive em `crates/ph2d-app-<família>`; a shell é COMPOSIÇÃO.** A catraca
  `the_shell_only_shrinks` reprova o crescimento — a cura é MOVER, nunca subir o número
  ([HOWTO](docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md)).
- Smoke = `--profile smoke` (3 s); `--release` só para smoke de PERFORMANCE. Linker `mold` no Linux
  (global, nunca no repo); nada de Cranelift. Detalhe: DIRETRIZ §6 e §6.7.

## §3 — CI / ship

Implementador não faz `git push`. Quem faz ship (por ordem do Enio): `./scripts/ship.sh` (paridade com
o CI) → corrija todo `✗` → `git push origin main` → acompanhe o CI até `success` → dê o link
(`https://github.com/dibrioli/PH2D/actions/runs/<id>`). Protocolo: DIRETRIZ §8 (`/pd-ship`).

## §4 — Memória persistente

Versionada em [`project-memory/`](project-memory/) (índice [`MEMORY.md`](project-memory/MEMORY.md), com
tecto de bytes e gate); o Claude Code chega lá por symlink. LLM nova lê o índice antes de agir.
Multi-máquina: [MULTI_MACHINE_SETUP.md](docs/DevOps/MULTI_MACHINE_SETUP.md).

## §5 — Estado dos módulos (ROTEADOR)

> **Regra (com gate):** cada módulo tem **UMA entrada de no máximo 700 bytes**: o que é, o smoke
> principal, onde ler, e o **último handoff de integração** — é lá que está o que fica ABERTO. Fechar
> uma linha **troca o link desse handoff** e, se o módulo mudou de natureza, a frase do que ele é.
> ⛔ Nada de narrativa aqui: ela vai para o handoff. A história até 2026-10-02 está verbatim em
> [`estado-2026-10-02/`](docs/archive/estado-2026-10-02/README.md) (e a anterior em
> [`estado-2026-08-18/`](docs/archive/estado-2026-08-18/README.md)).

### §5.0 — Leis que atravessam os módulos

- **Número que soma entre linhas se CONTA, nunca se escolhe** (`PROJECT_SCHEMA` — escada e tripla em
  ficheiros irmãos, três sítios —, registos de componentes, ADR, cenas de smoke). A fonte é o código.
- **O que foi MEDIDO e REJEITADO não se reconstrói:** cada doc cortado leva a tabela `⛔ Recusas
  MEDIDAS`; consulte-a antes de propor mudança de desenho.
- **Antes de construir um item de lista aberta, MEÇA se a composição já o exprime**, e confira o
  CÓDIGO antes de acreditar numa ausência.
- **Doc grande se corta com prova:** `python3 scripts/doc-split.py` (sha256). Índice de pasta se gera:
  `bash scripts/doc-index.sh`.
- **Catraca sem censo de obsolescência vira licença**; tecto de LOC **soma entre linhas** (cura por
  corte, nunca isenção); **isenção viaja com o código**.
- **Mover código parte gates em espécies mudas** (censo por prefixo que passa a varrer zero, `#[cfg]`
  órfão, visibilidade que muda de regime): [HOWTO](docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md) §2.
- **Controlo morto:** pergunte *o painel escreve onde · quem lê · o leitor DECIDE?* — e distinga do id
  ÓRFÃO, cuja cura é oposta. Painel derivado de tabela não esconde knob morto.
- **Cena de smoke que ensina o contrário do que acontece é pior que cena ausente.** O número da próxima
  cena conta-se no roteador de cada família.
- **Relógio acima de `load ~5` não vale nada; flakes de carga são uma FAMÍLIA:**
  [FLAKES_DE_CARGA.md](docs/DevOps/FLAKES_DE_CARGA.md) — confirme sozinho com o `loadavg` ao lado
  antes de culpar o commit, e promover um membro são duas escritas (tabela + faixa do nextest).
- **Sweep sobre ficheiro comprimido** só prova algo se descomprimir; a passagem que desfaz quebras de
  linha é uma função com dois chamadores.
- **Integrar não é aprovar:** smoke é do Enio; integrar e shipar só por ordem dele.

### §5.1 — Módulos

- **Motion Nodes** — dinâmica declarativa sobre `ph2d-nodegraph` (crate `ph2d-app-motion`), cook na GPU por omissão (`PH2D_GPU_COOK=0` volta à CPU). Protocolo: [ciclos](docs/Motion%20Nodes/103_dinamica_dos_ciclos.md), smoke = tutorial em PDF. Smoke `PH2D_GPU_COOK_DEMO=<n>`. Último: [handoff 01/10](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-01.md) · [docs](docs/Motion%20Nodes/README.md) · [BUGS](docs/Motion%20Nodes/BUGS_motion_nodes.md) · [história](docs/archive/estado-2026-10-02/motion-nodes.md)
- **Timeline** — dope-sheet, graph editor, clips, nesting, sinais e expressões (`ph2d-timeline`); o `TimelineDoc` viaja como blob com versão própria. Smoke `PH2D_NEST_SMOKE=1..3`. [docs](docs/Timeline/README.md) · [BUGS](docs/Timeline/BUGS_timeline.md) · [handoffs](docs/Timeline/handoffs/README.md) · [história](docs/archive/estado-2026-10-02/timeline.md)
- **Áudio** — rack de efeitos (no-op byte-idêntico no ponto neutro), export Ogg/Opus, denoise ML (`audio-ml` OFF). Aberto: [`03_o_que_falta.md`](docs/Audio/03_o_que_falta.md). Smoke `PH2D_AUDIO_DELIVERY_SMOKE`. [docs](docs/Audio/README.md) · [handoffs](docs/Audio/handoffs/README.md) · [história](docs/archive/estado-2026-10-02/audio.md)
- **Painter** — Layers + Efeitos e o motor de pintura clean-room (Digital · Watercolor · Impasto · Wet Paint · Composite), crate `ph2d-app-painter`. Smoke `PH2D_COMPOSITE_SMOKE=1`. Último: [handoff 04/10](docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-04.md) · [docs](docs/Painter/00_INDEX.md) · [BUGS](docs/Painter/BUGS_painter.md) · [perf](docs/Painter/28_otimizacoes_o_que_funcionou.md) · [história](docs/archive/estado-2026-10-02/painter.md)
- **Vector + Esqueleto** — motor vetorial nativo kurbo/Vello referenciado no Rive ([ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)), crates `ph2d-app-vec`/`ph2d-app-skeleton`. Smokes `PH2D_BUILD_SMOKE=<n>` · `PH2D_VEC_BONE_SMOKE=<n>`. Último: [handoff 04/10](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A_FRENTE_E_OS_EFEITOS_2026-10-04.md) · [docs](docs/Vector%20Module/README.md) · [BUGS](docs/Vector%20Module/BUGS_vector.md) · [história](docs/archive/estado-2026-10-02/vector.md)
- **Física** — runtime-truth sobre `rapier2d`, ponte `ph2d-physics-ecs` (`BTreeMap`, nunca `HashMap`), crate `ph2d-app-physics`. Smoke `PH2D_PHYSICS_SMOKE=<n>`. Último: [handoff Fase C](docs/Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_C_2026-09-12.md) · [docs](docs/Physics/README.md) · [BUGS](docs/Physics/BUGS_physics.md) · [história](docs/archive/estado-2026-10-02/physics.md)
- **3D / Sculpt** — malha que doa a normal ([ADR-0150](docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)); pincéis, Quad Retopology, tinta fina e o Painter na peça, crate `ph2d-app-sculpt3d`. Smoke `PH2D_SCULPT3D_SMOKE=<n>`. Último: [handoff 04/10](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_A_PAREDE_E_O_RELEVO_BORRADO_2026-10-04.md) · [docs](docs/3D/README.md) · [BUGS](docs/3D/BUGS_sculpt3d.md) · [retopologia](docs/3D/quad-retopology/README.md) · [história](docs/archive/estado-2026-10-02/sculpt3d.md)
- **3D Modeling** — modelador SDF editável para sempre e o modo Render ([ADR-0161](docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)), crate `ph2d-app-field3d`. Smoke pill MODEL · `PH2D_FIELD_SMOKE=<n>`. Último: [handoff 01/10](docs/3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_O_RELOGIO_DO_RENDER_2026-10-01.md) · [docs](docs/3DModeling/README.md) · [render](docs/Render3d/README.md) · [BUGS](docs/3DModeling/BUGS_3dmodeling.md) · [história](docs/archive/estado-2026-10-02/3dmodeling.md)
- **Flip** — animação 2D no idioma do Grease Pencil; o traço é PERCORRIDO, não rasterizado (`PH2D_FLIP_NEW_ENGINE=0` volta ao antigo). Smoke `PH2D_FLIP_HARDNESS_SMOKE`. [docs](docs/Flip/README.md) · [BUGS](docs/Flip/BUGS_flip.md) · [handoffs](docs/Flip/handoffs/README.md) · [história](docs/archive/estado-2026-10-02/flip.md)
- **Runtime (sinais)** — `ph2d-runtime`: produtores publicam, cada consumidor lê com o próprio cursor; a ordem no quadro tem gate. Smoke `PH2D_SIGNAL_SMOKE=1|2`. [docs](docs/Runtime/README.md) · [handoffs](docs/Runtime/handoffs/README.md) · [história](docs/archive/estado-2026-10-02/runtime.md)
- **UI/UX** — redesenho plano (`PH2D_UI_NEW=0` volta ao clássico); rolagem única, cartões, fonte/peso/tamanho; texto por `ph2d-i18n` com censos. Último: [handoff 01/10](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-01_A_LINHA.md) · [docs](docs/UI_New_and_Simple/README.md) · [história](docs/archive/estado-2026-10-02/uiux.md)
- **Editor / shell** — uma fila de undo por DIFF (`ProjectState`), projeto `.ph2dproj`, preview ≠ documento (`ph2d-preview-drive`); referência durável entre objetos é o `StableId`/nome, nunca bits. [project.rs](shells/desktop/src/project.rs) · [undo.rs](shells/desktop/src/undo.rs) · [história](docs/archive/estado-2026-10-02/editor-shell.md)
- **Componentes** — instâncias e variantes, Inspector por componente, e os componentes de jogo (vida, navegação, câmera, HUD…), crate `ph2d-app-components`. Smokes `PH2D_NAV_SMOKE=1..4` · `PH2D_VIDA_SMOKE=1..4`. Tutoriais em PDF (`docs/Components/tutoriais/`, 01–03). Último: [handoff 03/10 (W10)](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_A_MONTAGEM_POR_BLOCOS_2026-10-03.md) · [docs](docs/Components/) · [história](docs/archive/estado-2026-10-02/components.md)
- **Sprite Inspector** — as doze secções, animação por intervalos sobre a grelha, import `.ase`. Smoke `PH2D_ANIM_SMOKE`. [spec](docs/Sprite_projeto/README.md) · [handoffs](docs/Sprite_projeto/handoffs/README.md) · [história](docs/archive/estado-2026-10-02/sprite-inspector.md)
- **Image Tools** — utilitários de bitmap (`ph2d-tool-*`) sob o contrato `Tool=12`; Inpaint, Deform. [bugs](docs/Image%20Tools%20Bugs/README.md) · [história](docs/archive/estado-2026-10-02/image-tools.md)
- **Retirados** — aquarela/fluid antiga, brush engine original e o vetor antigo: não reconstrua sem ler o porquê. [história](docs/archive/estado-2026-10-02/retirados.md) · [planos de nós](docs/archive/estado-2026-10-02/planos-de-nos.md)

## §6 — Contratos congelados (mexer = Coord-only + ADR; DIRETRIZ §4)

- **Nodes** ([ADR-0039](docs/architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)): `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — gate `architecture_contract_surface`.
- **Tools** ([ADR-0040](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md)+[0041](docs/architecture/decisions/0041-rasteredit-rename-and-deactivate.md)): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` — gate `architecture_tool_contract_surface`.
- ~~**Painter (pintura)** (ADR-0043..0053)~~ — **REVOGADO** por [ADR-0099](docs/architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md) (o gate `architecture_painter_contract_surface` saiu com a crate). Efeitos sobrevivem em `ph2d-painter-effects` (não-gateada); `ColorProfile` em [`ph2d-imageio`](crates/ph2d-imageio/src/color.rs) (gate `architecture_imageio_contract_surface`).
- ~~**Watercolor (física)** (ADR-0049/0078-0084)~~ — **REVOGADO** por [ADR-0096](docs/architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md). Nada congelado.
- **Vector (data-model foundational)** ([ADR-0056..0068](docs/architecture/decisions/)): `VectorOp≤16`/`Vertex`SmallVec32/`Segment`64/`Region.segments`16/`AnimValue`/`sample(t:f64)`/`MAX_SPIRAL_TURNS=64`/`MAX_POLYGON_SIDES=128`/`MAX_VERTICES_PER_LLM_GEN=1000` — gate `architecture_vector_contract_surface`. Continua congelado depois do [ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md); o motor novo `ph2d-vec-*` tem contrato próprio, ainda não congelado.

## §7 — Design system

[`PROMPT_CLAUDE_DESIGN.md`](docs/design/PROMPT_CLAUDE_DESIGN.md) alimenta os widgets em Vello sobre [`ph2d-editor-core`](crates/ph2d-editor-core/) (ADR-0023). Referência: [`component-library.html`](docs/design/component-library.html).

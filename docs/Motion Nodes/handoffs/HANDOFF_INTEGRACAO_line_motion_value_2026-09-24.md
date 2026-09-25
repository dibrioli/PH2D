# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-09-24

> **Para o agente INTEGRADOR** (só por ordem do Enio — `CLAUDE.md` §0.7). A linha fechou, fez o
> `git rebase main`, fez a auditoria final (§3.1) e passou o portão de fecho na árvore combinada (§7). **Não integra nem pusha
> sozinha.** Este documento SUPERSEDE, como documento de integração, o de
> [2026-09-20](HANDOFF_INTEGRACAO_line_motion_value_2026-09-20.md): aquele foi integrado (commits
> `integ(motion-value)` no `main`), e o que segue são os **85 commits** feitos DEPOIS dele (o último é o da auditoria, `9efb0bb76`, mais o do fecho deste documento).

## §0 — IDENTIDADE

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| ramo | `line/motion-value` |
| base (pós-rebase) | `main` @ `20a630f1b` — **0** commits do `main` por trazer |
| commits da linha | **85** + o do fecho (2026-09-20 → 2026-09-25) · `297` ficheiros (+27 463 / −1 823) |
| integração | `--ff-only` possível no momento do fecho (a linha está POR CIMA do `main`) |

## §1 — O QUE A LINHA ENTREGA

Cinco assuntos, todos no módulo Motion. O mecanismo de cada um vive no doc dele; aqui fica o que
o integrador precisa de saber.

| assunto | doc | o que o dono consegue fazer | smoke do dono |
|---|---|---|---|
| **Ciclo 9 fecha** (rig & corpos moles: a corda veste o osso, a espessura é da cadeia) | [114](../114_ciclo_9_rig_e_corpos_moles.md) + [tutorial 09](../tutoriais/09_coisas_que_se_seguram.pdf) | os seis `rig.*` e os quatro corpos moles com a interface completa | ✅ aprovado |
| **Ciclo 10 — o carimbo no dispositivo** (`motion.clone` e `motion.duplicator` com kernel; forma encodada UMA vez e carimbada N; recorte por câmara; LOD da forma por tile) | [116](../116_ciclo_10_o_carimbo_no_dispositivo.md) | campos de dezenas de milhares de formas a 60 fps; cena `=126` (galáxia de estrelas com simulação) | ✅ aprovado (raw `132`/`128`, galáxia estável) |
| **A mistura em grupo** (Add/Multiply/Screen de um sink inteiro, na cena vectorial; modo de uma linha; o Motion desenha por cima do mundo pelas DUAS rotas) | [118](../118_a_mistura_em_grupo.md) | `Blend With` no cartão; cenas `=13`/`=14`/`=15` | ✅ aprovado |
| **Ciclo 11 — a placa com várias saídas** (plano da UNIÃO de N sinks, `cook_many` num buffer, o `Filter` de uma saída cozida na placa chega ao pixel) | [119](../119_ciclo_11_a_placa_com_varias_saidas.md) | grafos com várias saídas ficam na placa; censo de rota: **95 de 126** cenas na placa | ✅ aprovado (cena `=16`, o enxame) |
| **Ciclo 12 — os tectos confortáveis** (a escada medida nas duas placas; tecto de objectos por nó `16 384 → 32 768` por ordem do dono; o carimbo das IMAGENS vai à placa e a simulação deixa de ser arrastada para a CPU; a leitura dos cartões deixa de esperar pela placa) | [120](../120_ciclo_12_os_tectos_confortaveis.md) | o custo das imagens deixa de crescer com o número: a `32 768` no proxy de telemóvel, Motion `7,30 → 0,72 ms`; na máquina do dono `0,56 ms` | ✅ aprovado (as duas rondas) |

⚠️ **Os ciclos 1 a 12 da fila do [doc 103](../103_dinamica_dos_ciclos.md) estão TODOS fechados.**
O que vem a seguir é decisão do dono.

## §2 — SUPERFÍCIE DE COLISÃO, medida (`collision-surface.sh`, pós-rebase)

```
SCHEMAS             PROJECT_SCHEMA 160 (base 160) · tripla (160, 13, 22) igual
                    VEC_SCENE 22 · FLIP 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23 — todos iguais
REGISTRO            ph2d-ecs 103 · ph2d-render 104 · ph2d-script 104 — todos iguais
CONTRATO (§6)       crates/ph2d-nodegraph/src/node.rs 91 linhas mudadas (ver §2.2 — ADITIVO)
                    crates/ph2d-editor-core/src/tool.rs intocado
ADR                 nenhum criado
Cargo.lock          nenhum pacote externo novo
MARCADORES          nenhum
TETOS DE LOC        nenhum ficheiro da linha acima do tecto (motion_bridge_gpu.rs 717/700 → 658, CORTE, §3)
```

⇒ **Zero contador partilhado se move.** Não há degrau de `PROJECT_SCHEMA` a recontar.

### §2.1 FOUNDATIONAL tocado — o que é aditivo e o que NÃO é

**Aditivo (nenhum chamador existente muda):**
- `ph2d-nodegraph`: `cook_advance_within.rs` (novo: `cone_a_montante` + `Cook::advance_tick_fanned_within`);
  `cook.rs` (`advance_tick_inner` ganha `so: Option<&BTreeSet<NodeId>>`, os dois chamadores antigos
  passam `None` ⇒ byte-idênticos); `node.rs` **só constantes novas** (`MAX_INSTANCIAS_POR_NO = 32 768`,
  `LADO_MAX_DE_GRELHA = 181`, `CELULAS_DE_UMA_GRELHA_CHEIA`, com `const assert`s) — ⛔ **nenhum trait do
  contrato congelado mudou** (`NodeOp`/`OpResolver`/`NodeManifest`; o gate
  `architecture_contract_surface` corre no portão, §7).
- `ph2d-gpu-cook`: `tap_voo.rs` (novo: `tap_sem_espera`/`descarta_tap_em_voo`/`tap_em_voo`), `cook_many`,
  `plan_driven_many`, `ordem_reproduzivel`, `GpuPlan::lineage_boundary`; `tap.rs` partido em
  `encomenda_tap`/`le_tap` (o `tap` síncrono público fica com a mesma assinatura).
- `ph2d-render`: `render_to_intermediate_over_world`, `draw_shared_instances_em_camadas`,
  `Z_ORDER_OVER_THE_WORLD`, `MediumKind::IMAGE_ON_VECTOR` (entra no `ALL`).
- `ph2d-vector`: `resolve_cost`, `scene_prepared` (`SceneResolver`) — módulos novos.
- `ph2d-node-registry`: `register_live_vector_source_when` + `LiveVectorWhen` (side-metadata).

**⚠️ NÃO aditivo — parte código de OUTRA linha que o use:**

| mudança | quem usa hoje (medido com `git grep` nesta árvore) |
|---|---|
| `ColumnAccess` ganha `SourceReadWriteExisting` e `SourceWriteExisting` | só crates do Motion; um `match` exaustivo novo noutra linha deixa de compilar |
| `ColumnAccess::is_source_read` **renomeado** para `is_source_mapped` | **zero** chamadores fora da própria crate |
| `GpuCookError` ganha `SinkStyleMismatch`, `OrdemEntreSaidas`, `Kernel`, `Map` | só crates do Motion |
| `ph2d_nodegraph::layout::plan_edges` passa de `(&[K], &[(K,K,bool)])` a `(&[Item<K>], &[Wire<K>])` | `ph2d-motion-doc` (já migrado) + os testes da crate |

⇒ se outra linha da rodada trouxer um chamador destes, **a falha é de compilação e alta** — a cura é
do lado dela, com a assinatura nova.

### §2.2 A shell

`shells/desktop` tocada em **10** ficheiros (+406/−80). ⚠️ Confira contra as outras linhas a
**ordem das fases do quadro** se alguma mexer no desenho do Motion por cima do mundo (doc 118 §10).

## §3 — O que o FECHO curou (e porquê)

| achado | onde | cura |
|---|---|---|
| tecto de LOC `717/700` | `crates/ph2d-app-motion/src/motion_bridge_gpu.rs` | **corte por responsabilidade**: as quatro cercas de FONTE (vector vivo · objecto) saem para o irmão `motion_bridge_gpu_objecto.rs`, no molde dos irmãos `colisor`/`forma` que já existiam; `pub(super) use` preserva todos os caminhos |
| a doc do `cook_gpu` estava colada ao `type FioDeParam` | o mesmo ficheiro | devolvida à função (a forma do *«corte que sobe por `///`»* que a memória da casa regista) |

### §3.1 A AUDITORIA FINAL (quatro lentes em paralelo, 2026-09-24) — o que achou e a cura

Cada cura tem gate, e cada gate **sangra** sob a mutação que devolve o defeito (§7, `9 de 9`
mais a da medida). Nenhuma mexe em contador partilhado.

| achado | mecanismo | cura |
|---|---|---|
| **o cone da marcha perdia os FIOS de param** | `cone_a_montante` seguia `edges()` e `pre`, e não `graph.param_sources(n)` — um nó conduzido por fio fora do cone ficava parado a meio de uma corrida híbrida | o cone segue as duas relações; gate `o_cone_segue_os_fios_que_conduzem_um_param` |
| **a TOMADA do gizmo lia um laço fora do cone** | a marcha só pedia as fronteiras; uma tomada armada sobre um nó com `pre` fora delas lia o laço de há N quadros | porta única `MotionCookPump::avanca_o_pre` (`ph2d-eval-motion/src/marcha.rs`), com `fronteiras ∪ tomadas`; gate `uma_tomada_armada_poe_o_laco_que_ela_le_no_cone` |
| **o SCRUB marchava por outra porta** | o `scrub.rs` chamava o avanço da rota inteira enquanto a reprodução marchava o cone — duas respostas à mesma pergunta | o scrub passa pela MESMA porta; gate `o_scrub_de_fronteiras_marcha_o_mesmo_cone` |
| **o kernel do `motion.duplicator` lia fora do buffer com ZERO pontos** | com o orçamento esgotado (`dp_np = 0`) o `read_points_P(dp_p)` indexava um buffer vazio | `dp_pp = (0,0)` quando `dp_np == 0`; gate de GPU `sem_orcamento_para_pontos_o_carimbo_devolve_a_forma` (Δpos `0`) |
| **a leitura sem espera guardava números de ANTES do rebobinar** | sem nada a amostrar o `tap_sem_espera` devolvia o `ultimo` velho, onde o `tap` síncrono devolve nada | `ultimo = None` nesse caminho; gate de GPU `sem_nada_a_amostrar_a_leitura_antiga_e_esquecida` |
| **uma leitura REPETIDA parava os fios e duplicava a sonda** | quando a placa ainda não acabou, a chamada devolve a leitura anterior; os fios que «marcham» quando o valor MUDA liam-na como parada, e a sonda juntava a mesma amostra duas vezes | `GpuCook::tap_fresca()` + `MotionState::tap_fresco`/`flow_quente`; gates `uma_leitura_repetida_nao_para_os_fios` e `uma_leitura_repetida_nao_entra_no_anel_da_sonda` |
| **abrir um ficheiro herdava três estados da cena anterior** | o `install` não limpava a leitura em voo (ids do documento velho), o `tap_fresco` nem o `cpu_pedida` (pedido da CENA de demo, que um ficheiro de artista não tem) | limpos no `install`; o gate `loading_forgets_every_id_that_named_the_previous_document` passa a afirmá-los |
| **a arrumação media os cartões uma fileira mais BAIXOS** | o `medir` usa o retrato SEM readout (ele é carimbado depois de cozinhar) e quase todo cartão cozido mostra um número ⇒ `ROW_H` a menos por cartão; o gate irmão não o via porque arrumava e media com a MESMA medida curta | a fileira é sempre reservada (o lado conservador que o cartão de GRUPO já escolhe); gate `a_medida_reserva_a_fileira_do_numero` (sob a mutação: `472` contra `494`) |
| **o gate do tecto do L-System comparava a const CONSIGO MESMA** | `MEASURED_LSYSTEM_CEILING = MAX_INSTANCIAS_POR_NO - 1` é a MESMA expressão do `MAX_MODULES` | literal `32_767` (o doc ao lado já proibia o contrário) |
| **o gate do roteiro da `=126` lia o ficheiro INTEIRO** | `Grid`/`Duplicator` aparecem no código que MONTA a cena ⇒ verde com o roteiro calado | lê só o corpo do `announce()`, como o irmão |
| **o gate do relator de erros da placa lia os COMENTÁRIOS** | o doc cita `on_uncaptured_error` pelo nome ⇒ a 2.ª metade verde com o registo apagado | filtra as linhas `//` (⚠️ um bloco `/* */` continua a passar — nomeado) |
| **nomes de gate e de crate mortos nos docs** | doc 120 citava `a_escada_mede_a_rota_hibrida_com_as_duas_formas` e dizia que a simulação corre na CPU; doc 103 citava `params_are_drawn` e a crate apagada `ph2d-panel-motion-params`; o rustdoc do `paint_card_params.rs` também | corrigidos, com a morte da premissa à vista |
| **as cenas `=16`/`=17` faltavam na lista do roteador de `PH2D_MOTION_OBJ_SMOKE`** | a mesma forma do `12` que a auditoria de 31/08 já tinha curado | acrescentadas |

## §4 — ⚠️ Coisas que uma leitura rápida do diff entende ao contrário

1. **O carimbo das ESTRELAS continua na CPU, e isso é decisão do dono** (*«manter a nitidez»*, a W2
   do doc 116 retirada). Só as IMAGENS do átlas vão à placa.
2. **`advance_tick_fanned_within` NÃO é o `cook_substep::upstream_cone`:** este segue também as arestas
   de `pre` (uma fronteira que lê o tique anterior de uma fonte precisa que ela avance).
3. **O gizmo das posições deixou de pedir TODOS os sinks** — pede só os que a placa não desenhou (o
   registo `GpuCook::shape()` + o mesmo `veredito_do_dispositivo`). O gizmo do PIVÔ passou a pedir o
   sink ele próprio; um terceiro leitor de sinks por tomada tem de o pedir também.
4. **Os números dos cartões do grafo ficam DOIS quadros atrás no total** (já ficavam um: a leitura
   corre antes do cozimento; agora é encomendada num quadro e recolhida no seguinte). Um quadro que
   não é da placa DESCARTA a leitura, e uma leitura repetida (a placa ainda não acabou) não conta
   como valor novo — nem para os fios nem para a sonda.
5. **O tecto `MAX_INSTANCIAS_POR_NO` subiu por ordem do dono (`32 768`)** e o lado da grelha deixou de
   ser exacto (`181² = 32 761`).
6. **Dois testes `#[ignore]` vermelhos são PRÉ-EXISTENTES e não desta rodada de mudanças** (§6).

## §5 — ⛔ Premissas minhas que a medição derrubou (nesta jornada)

- *«Com o carimbo na placa o Motion desce»* — não desceu até se acharem **duas contas escondidas na
  CPU** (a marcha do prefixo a simular o laço que a placa já tinha, `4,0 ms`; a tomada do gizmo a
  recozinhar a cadeia inteira, `2,4 ms`). Doc 120 §8.5.
- *«A leitura dos cartões custa `+0,075 ms`»* — medido num arnês sem janela; no app o `poll` síncrono
  esperava pelo quadro inteiro (`0,68 ms`). Doc 120 §8.6.
- *«Na RTX o custo não mexe»* — eu medi `~1,1 ms` com outras linhas a load `4,5`–`5,8`; a máquina do
  dono, calma, lê **`0,56`**.

## §6 — ⏳ O QUE FICA ABERTO (e de quem é)

| item | dono |
|---|---|
| as ESTRELAS na placa sem perder nitidez (a pergunta da ponte sobre o vector vivo) | **decisão do Enio** |
| na NVIDIA a ENCOMENDA da leitura dos cartões custa `~0,35 ms` (a submissão em si); cura: encomendar no mesmo submit do cozimento | linha Motion, se o dono quiser |
| *«um pedido em voo de cada vez»* na leitura sem espera não tem régua determinística (só com placa mais lenta que o quadro) | nomeado |
| `write_the_rig_figures` (`#[ignore]`): a cena `=120` tem `19` pontos na corda contra os `20` que o gerador afirma | linha Motion |
| `measure_the_source_group` (`#[ignore]`): anterior à lei *«só com forma»*; passa com `PH2D_MOTION_SO_COM_FORMA=0` | linha Motion |
| o gate do relator de erros da placa filtra comentários de linha (`//`); um bloco `/* */` com o nome do registo continuaria a passar | nomeado |
| a medição ACIMA do tecto com a cura (compilação local) não foi refeita — a tabela do doc 120 §7.1 ficou conservadora para imagens | linha Motion |

## §7 — A PROVA DE FECHO (corrida nesta árvore, pós-rebase)

Corrida DEPOIS da auditoria, sobre o `HEAD` `9efb0bb76` (que está POR CIMA do `main` `20a630f1b`,
`0` commits por trazer). ⚠️ A máquina esteve a `load 50`–`115` a corrida inteira (outras linhas).

| portão | resultado |
|---|---|
| `cargo fmt --all -- --check` | ✅ (a 1.ª corrida apanhou um `assert!` meu por formatar — curado) |
| `cargo check --workspace --config 'build.warnings="deny"'` (o passe do CI, sem `--all-targets`) | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ (a 1.ª corrida apanhou um `mut` a mais num teste meu — curado) |
| `nextest-impacted.sh` (`BASE` = merge-base, `ci-test`) | **20 201 / 20 202** — o único ✗ é `the_cost_of_sampling_a_path_is_flat_in_its_anchors` (`ph2d-timeline`), **membro NOMEADO** da família de flakes de fan-out do §5.0, **3 de 3 verde sozinho a `load 53`–`71`**, e a `ph2d-timeline` não tem uma linha do diff |
| `censos-da-arvore-combinada.sh` (HR-15 + tectos, sobre o `main`) | ✅ **127 / 127**, controlo do filtro `12 de 12` |
| `doc-index.sh --check` | ✅ |
| machete · crates opcionais sozinhas · pacotes citados por workflow | ✅ na 1.ª corrida (`20 196 / 20 196` no impacted); a auditoria **não mexeu** em `Cargo.toml` nem no `Cargo.lock` |
| GPU com adaptador (RTX, `#[ignore]`) | ✅ `tap_sem_espera` (3) · paridade do `motion.duplicator` (incl. o gate novo do orçamento a zero, Δpos `0`) · `ph2d-app-motion` tomada/gizmo/readout (11) |
| `grep -rn LOCAL-MEDICAO crates shells` | `0` |

**Provas de mutação das curas da auditoria — 12 de 12 sangram** (arnês com controlo de filtro:
`run = 0` lê-se `FILTRO VAZIO`, nunca «sobreviveu»):

| mutação (devolve o defeito) | gate que sangra |
|---|---|
| `MAX_MODULES = MAX_INSTANCIAS_POR_NO` (fora por um) | `instance_ceiling_agrees` |
| o roteiro da `=126` deixa de nomear o `Grid` | `o_roteiro_nomeia_o_que_o_dono_vai_ver` |
| o relator deixa de registar o `on_uncaptured_error` | `quem_cria_o_dispositivo_liga_a_voz_dele` |
| o cone ignora os fios de param | `o_cone_segue_os_fios_que_conduzem_um_param` |
| a marcha ignora as tomadas | `uma_tomada_armada_poe_o_laco_que_ela_le_no_cone` |
| o scrub avança a rota inteira | `o_scrub_de_fronteiras_marcha_o_mesmo_cone` |
| a leitura repetida conta como nova (fios) | `uma_leitura_repetida_nao_para_os_fios` |
| a leitura repetida entra na sonda | `uma_leitura_repetida_nao_entra_no_anel_da_sonda` |
| o `install` mantém o `cpu_pedida` | `loading_forgets_every_id_that_named_the_previous_document` |
| o `medir` não reserva a fileira do número | `a_medida_reserva_a_fileira_do_numero` (`472` contra `494`) — ⚠️ e o irmão `a_arrumacao_nao_sobrepoe_dois_cartoes` fica VERDE sob ela, que é a razão de o gate novo existir |
| o kernel lê os pontos com `dp_np = 0` (GPU) | `sem_orcamento_para_pontos_o_carimbo_devolve_a_forma` |
| a leitura antiga fica depois do vazio (GPU) | `sem_nada_a_amostrar_a_leitura_antiga_e_esquecida` — ⚠️ **SOBREVIVEU à 1.ª redacção**: com nada a ler as duas versões devolvem vazio, e a leitura velha só reaparece na chamada SEGUINTE; o gate ganhou esse passo e sangra |

## §8 — OS SMOKES (o comando inteiro, copiável)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_MOTION_OBJ_SMOKE=17 PH2D_TECTO_N=32768 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=124 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=126 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_MOTION_OBJ_SMOKE=16 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_MOTION_OBJ_SMOKE=14 cargo run -p ph2d-host-desktop --profile smoke
```

Os dois binários ficam COMPILADOS nesta árvore (2.ª corrida, depois de apagar os `incremental`:
`76 GB` reclamados):

```
Finished `smoke` profile [optimized] target(s) in 0.53s
Finished `release` profile [optimized] target(s) in 0.33s
```

(`=13` a mistura na placa · `=14` a mistura em grupo · `=15` o modo de uma linha · `=16` o enxame ·
`=17` a escada dos tectos, com `PH2D_TECTO_N` e `PH2D_TECTO_FORMA=1` para as estrelas · `=124` as
marcas das posições · `=126` a galáxia de estrelas.)

## §9 — A UMA LINHA proposta para o `CLAUDE.md` §5 (o integrador aplica)

> ✅ **E A LINHA FECHOU EM 24/09 com os ciclos 10, 11 e 12 e a mistura em grupo, os quatro com smoke do dono aprovado** ([handoff](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-24.md) — ⚠️ o §2.1 lista as quatro mudanças NÃO aditivas de API na base (`ColumnAccess` · `GpuCookError` · `plan_edges` · o `is_source_mapped`)): o carimbo das IMAGENS vai à placa e a `32 768` o Motion no proxy de telemóvel vai de `7,30` para **`0,72 ms`**; ⏳ as estrelas ficam na CPU por decisão do dono (nitidez).

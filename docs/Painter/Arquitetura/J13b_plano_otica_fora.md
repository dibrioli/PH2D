# J13b — PLANO: a óptica do óleo sai do fio da tela (02/11)

Plano do agente de arquitetura sobre o HEAD `7718d06194`; diagnóstico em [J13](J13_fps_do_real_oil.md). Estado dos passos: ver o handoff da janela.

Verificado com `pwd` em `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, ramo `line/PainterWatercolor`, HEAD `7718d06194`.

## 0. A escolha: (b), uma thread de composição sobre um espelho das grades

**Por que não (a).** No (a) o worker faria a óptica (C ≈ 50 ms) entre dois passos. Isso falha em três pontos:
- O passo seguinte atrasa 50 ms, o dobro do orçamento de 25 ms. O `MAX_BEHIND_S` (`offthread.rs:60`, 2 passos = 50 ms) perde tempo de água em cada pico (67–83 ms).
- O `bring_home` do pen-down (`offthread.rs:338`, `recv()` nu) passa a esperar até 50 ms em vez de um estágio. É um engasgo ao começar cada traço.
- O teto de imagens é baixo: no máximo (1000 − 40×5)/50 ≈ 16 imagens/s, e cai com a poça.

**O que é o (b).**
1. O worker continua dono do motor.
2. Num fim de passo, quando há folga e um pedido do fio principal, ele copia para um **espelho** só as linhas que mudaram desde a última cópia.
3. Uma terceira thread (o «compositor») faz a metade de cima sobre o espelho, enquanto o solver continua a correr.
4. O fio principal só aplica as caixas de pigmento, sombra e grão e faz o laço de píxel (< 1 ms).
5. O motor deixa de voltar a casa para mostrar água.

**O comentário «MEDIDO E REJEITADO» (`composite.rs:343-365`) já não vale nestes números.** O argumento dele era que as duas metades saturavam os 32 núcleos (sim 10 ms + composite 6 ms a 4096²). Hoje é diferente:
- O solver ocupa 5 ms de cada 25 (20 % do tempo).
- A óptica quase não escala: 55–60 ms com 16 núcleos, 46–49 ms com 32 (J13 §4). Há núcleos livres para sobrepor.

Isto é uma inferência, não uma medição. O passo 7 mede-a.

**O ritmo do dono («relevo a 24×/s, luz a 12×/s»)** mistura dois estados na mesma imagem e não cabe na exigência de «a conta de hoje ao byte por composição». O (b) dá um ritmo único, previsto entre 17 e 20 imagens/s com C ≈ 50 ms, acima dos 12 que ele pediu. Se o hardware der menos, a imagem abranda e a água não.

## 1. O que a metade de cima lê do `PainterTool`, e como chega ao compositor

Estas entradas **não** devem ir por `wet_facts`/`reconcile_facts`. Esse mecanismo escreve no `Sim` do motor; estes valores são da óptica, não da física. Vão num `FactosDaOtica` (`Copy`) que viaja **dentro de cada pedido de imagem**: cada imagem leva os factos com que foi feita e nunca há factos velhos.

| entrada | onde | o que a metade de cima precisa |
|---|---|---|
| `paint.brush.paper.is_active()` (`visual.paper`) | `composite.rs:42` | o bool |
| `wetpaint.smooth_edges` → `raio`, `suave` | `composite.rs:47-48` | `raio` (vai para `tinta_sem_oleo` e para `o.suave_px`) |
| `wetpaint.oleo_adiado` | `composite_oleo.rs:152` | o bool. O teste `grid.oleo_real` lê o motor e passa a correr no compositor sobre o espelho |
| `wetpaint.oleo_real_valores` | `composite_oleo.rs:169` | `OleoReal` (é `Copy`) |
| `self.papel` → `o.papel` | `composite_oleo.rs:171-173` | já convertido para linear |
| `paint.substrate_depth` → `o.dente_papel` | `composite_oleo.rs:182-185` | o valor final |
| o grão (`GraoNoFilme::de(paper, substrate_depth, substrate_rough, paper_image, impasto_rig)`) | `composite.rs:69-80` | **só `is_some()`**: `grao_das_celulas` usa `Some(_)` (`composite_oleo.rs:74`). O `GraoNoFilme` com empréstimo fica no fio principal (o `sombreia` está em `composite.rs:499`) |
| `wetpaint.oleo_na_tela` (escrito) | `composite_oleo.rs:166` | **volta** no resultado |
| `wetpaint.composta = MarcaDaAgua` (escrito) | `composite.rs:86` | volta no resultado (macros, W14) |
| interruptores de teste em `thread_local!` | `composite.rs:194`, `:204`; `composite_oleo.rs:26`, `:161`, `:177`; `grao_no_filme.rs:62` | capturados no pedido, num campo `#[cfg(test)]`. **Na outra thread não se veem** |

Já estão abaixo da fronteira e ficam no fio principal: Selection, protecção e alpha-lock (`wet_splat_gates`, `composite.rs:35`, usados de `:371` em diante), o vidro (`:378`), a marca e a cobertura (`:58-65`, `:151`, `:655-663`), as linhas da gravidade do Smooth Edges (`:51-54`) e o `exata`.

Do lado do motor, o compositor lê sobre o espelho:
- as camadas (grid, opacity, visible), em `:108`, `:158`;
- `alcance_na` (`:114`);
- `sim.gather_params(&tuning)` (`:181`). O `Params` é copiado pelo worker no instantâneo;
- `revisao` (`:198`);
- `tinta_sem_oleo`, que lê `sett`, `susp` e `layers[0].grid.s` (`composite_oleo.rs:108-114`).

O véu lê `film`, `wet` e `s` (`veu.rs`).

## 2. Buffers e protocolo

**Onde vivem os dados:**
- **Fio principal (a sessão):** `sess.pigment`, `sombra`, `grao` continuam como única verdade do laço de píxel (`session.rs:112-118`). Agrupam-se numa `PlanosDaOtica { pigment, sombra, grao, campo_cache }`.
  - O fio principal escreve-os pela composição local (traço, undo, varredura) ou ao aplicar um resultado.
  - O Smooth Edges e `sombra_sem_furo` leem para lá da caixa (`:447`, `:507`). Por isso o resultado copia-se **para dentro** destes planos, em vez de os trocar.
- **Compositor:** tem planos de trabalho próprios e **nenhum `CacheCampo`**. A chave da cache inclui `revisao`, que anda a cada passo, logo nunca acerta (J13 §2). A cache é exacta, por isso tirá-la não muda nenhum byte. Isto poupa 28 B por célula.
- **Espelho (`Box<Espelho>`):** uma `Grid` por camada, só com os planos da óptica (`sett`, `susp`, `*_rgb`, `altura`, `espessura`, `relevo`, `crista`, `nivelado`, `vela`, `film`, `wet`, `paper`, `material`, `aparencia`, `oleo`, `tinta`, `pigmento`). Ficam de fora `vel`, `flow`, `chao`, `oleo_antes*`, `sett_do_traco` e o undo.
  - Viaja por canal, como o motor: worker → compositor → worker. Sem mutex.
  - O worker só escreve nele quando o tem na mão, ou seja, quando o compositor está parado.

**Acumulador novo no motor.** `Engine::espelho_sujo` junta tudo o que passa por `merge_dirty` (`ph2d-wet-paint/src/painter.rs:615`; é a porta que muda as grades e mexe na `revisao`) e por `mark_dirty_full` (`:660`). **Não** junta `merge_dirty_otica`, que só mexe na óptica e não muda grades. É independente do `take_dirty`, que o fio principal também consome (`composite.rs:93`).

**Protocolo:**
1. **Tique** (`ciclo.rs:173-229`):
   - Se `fresh` e não há pedido no ar, publica `PedidoDeImagem { factos, epoca, divida }` numa caixa de correio do `SimWorker`. É um `Mutex<Option<_>>` curto, escrito só pelo fio principal.
   - Faz `try_recv` do canal de resultados. Se há resultado e ele é válido, copia as caixas para `PlanosDaOtica`, corre o laço de píxel (a metade de baixo extraída de `composite.rs:368-663`) e grava `seen_steps`, `oleo_na_tela` e `composta`.
   - **Não** chama `wetpaint_composite` para a água. Os ramos `facts_moved` e `adiados` continuam como hoje.
2. **Worker** (`offthread.rs:300-310`, depois de `closed`, e também no ramo ocioso `:288` se houver pedido e o motor estiver sujo):
   - Só avança se tiver o espelho, se houver pedido e se `acc + copia_prevista ≤ STEP_S`.
   - Junta `divida` ao sujo. Faz `take_dirty`, cresce a caixa por `alcance_na` e copia para o espelho as linhas de `take_espelho_sujo()` (`par_chunks`; com `Full` ou com nº de camadas diferente, reconstrói tudo).
   - Envia ao compositor `(espelho, caixa, params, opacity/visible, revisao, MarcaDaAgua, passos, pedido)`.
   - Com `Dirty::Clean`, responde «nada».
3. **Compositor:** corre a metade de cima pura dentro de `rayon::join`, pela mesma razão do `no_pool` (`por_quadro.rs:51`). Envia `Imagem { caixa_cel, caixa_px, pigmento/sombra/grão da caixa, sem_oleo, oleo_na_tela, veu: Option<film,wet da caixa>, epoca, passos, marca }` ao fio principal e devolve o espelho ao worker.
4. **Validade:** a sessão guarda `uniao_local`, a união das caixas compostas localmente desde o último pedido.
   - Se a imagem chega e a sua caixa toca `uniao_local`, é descartada e a caixa vai para `divida`, que entra no pedido seguinte. Nada se perde.
   - Cada píxel mostra sempre a última composição que o cobriu, que já é a lei de hoje (`por_quadro.rs:7-8`).

**Casos:**
- **Traço em curso:** fica no fio principal (2,5–4 ms). Com o traço aberto o motor nem vai ao worker (`hand_off_sim`, `offthread.rs:486-488`).
  - Risco já existente: a primeira composição de um traço herda o sujo da água (`take_dirty`). Hoje é igual; mede-se na fase «traço do Dabber».
- **Undo/redo (`undo_vivo.rs:89` faz `mark_dirty_full`):** `bring_home`, o mesmo `drain_step`, composição local. A imagem no ar é descartada por intersecção.
  - `PassoEmVoo` (`grid/snapshot.rs:280`) é interno ao motor. O espelho só copia em fronteira de **passo fechado** (`!step_pending`), nunca a meio de um passo.
  - Hoje o tique pode compor um estado a meio do passo. O novo compõe só passos fechados, o que continua a ser «a conta para o estado que lê».
- **Fim de sessão e troca de camada:** `wetpaint_end_session` (`authored_actions.rs:196`) e `wetpaint_guard` (`ciclo.rs:240`) fazem `session = None`. O `Drop` do `SimWorker` fecha também o compositor, pelo mesmo padrão `want` + canal (`offthread.rs:196-200`), e a imagem no ar morre. O caminho do véu no fim (`:198-203`) é local, como hoje.
- **Véu `show_wet`:** o resultado leva `film` e `wet` da caixa. `pinta_o_veu` (`veu.rs:15`) passa a receber fatias em vez de `&Grid`.
- **Roteiro de macros, worker ritmado e relógio fixo:** ficam no caminho de hoje. O compositor só existe no `worker_loop` ao relógio, nunca em `worker_ritmado` (`offthread.rs:214`), `EngineSlot::Roteiro` ou `wet_step_sync`. O determinismo dos portões não muda.
- **Varredura do painel (`oleo_no_quadro.rs:84`):** fica como está (8 ms por quadro). Pode ir pelo compositor mais tarde (um pedido com `merge_dirty_otica`).

## 3. O ritmo

- **Cadência:** uma imagem no ar de cada vez, com contrapressão. Um novo pedido sai no tique em que a anterior é aplicada. O ritmo é 1/(C + cópia + ≤1 quadro), ou seja, o que o hardware deixar, sem constante.
- **A regra que protege o passo:** a cópia só se faz se `acc + copia_ms_ema ≤ STEP_S`; senão espera pelo fim do passo seguinte. Assim nenhum passo atrasa por causa da imagem.
- **Medir (`wet_diag`, `PH2D_FLUID_PROFILE`):** `copia ms` p50/max, `otica ms` no compositor, `imagens/s`, `passo ms` com o compositor ocupado e parado, e `passos/s`.
- **Decisão por tabela:**
  - Se o máximo de `passo ms` com o compositor ocupado passar de 20 ms, o compositor ganha um `ThreadPool` próprio com núcleos − k (k medido).
  - Só se isso não chegar se põe um tecto de imagens/s, com a tabela ao lado (§0.0).
- **Variantes no mesmo processo:** `PH2D_WET_OTICA_FORA=0|1` lido em execução, nunca `const`, intercalado (MEDIR_VELOCIDADE).

## 4. Portões e a prova ao byte

**Continuam válidos sem mudança, porque usam `set_wet_relogio_fixo`/`wet_step_sync` (91 ficheiros), que fica no caminho de hoje:**
- `por_quadro_tests.rs:11-81`
- `oleo_cache_tests.rs`
- `queda_na_ladeira_tests.rs`
- `desempenho_oleo_tests.rs`
- `fps_j13_tests.rs`: os seus ganchos `campo_bit::gancho` (`composite.rs:246`) e `gancho` (`:654`) ficam na função partilhada e só correm no fio do teste.

**`offthread_tests.rs`:**
- `the_frame_does_not_run_a_sim_stage` (`:165`) continua: o tique não menciona `step_stage`.
- `the_tick_leaves_the_engine_with_the_worker` (`:192`) fica mais forte.
- `the_tick_asks_for_the_engine_with_a_bounded_wait` (`:353`) é estrutural sobre `try_bring_home` e mantém-se.
- `adiar_o_oleo_nao_traz_o_motor_para_casa` (`:651`) mantém-se.
- `the_water_runs_at_a_usable_rate…` (`:276`) deve melhorar.
- Reler `:431` (acção de tela com o worker a segurar o motor) e `:564` (pausa da autoria): o compositor tem de respeitar `wet_authoring_hold`, ou seja, não aplicar imagem durante o hold.

**Portões novos (ficheiro `otica_fora_tests.rs`):**
1. **`a_otica_pelo_espelho_e_a_tela_de_casa_ao_byte`:** a cena do J13 com relógio fixo, 96 passos, Real Oil, Smooth Edges e véu ligados e desligados, e troca de azimute.
   - Tool A compõe como hoje.
   - Tool B usa uma porta `#[cfg(test)]` síncrona: espelho, depois `compoe_a_otica`, depois aplicar e laço de píxel, tudo no fio do teste.
   - Compara `canvas_rgba` a cada passo.
   - Mutações que têm de ficar vermelhas: não copiar `altura` para o espelho; tirar a união `espelho_sujo` de `merge_dirty`; não juntar `divida`.
2. **Com a thread real:** deixar a água correr até adormecer (`sim_should_run` falso). Depois `mark_dirty_full` e comparar a imagem pelo compositor com a composição local ao byte.
3. **`o_fio_da_tela_nao_paga_a_otica`:** os contadores `composite_contagem` são `thread_local` no fio do teste e têm de ficar em 0 chamadas de óptica durante a fase «parado» com o worker. O controlo positivo é o caminho do relógio fixo a contar mais de 0.
4. **Na crate do motor:** depois de passos aleatórios, os planos do espelho são iguais aos da grade ao bit.

## 5. Ordem (cada passo ≤ 45 min; P0, P1 e P2 são disjuntos e paralelos)

| passo | ficheiros | o quê |
|---|---|---|
| **P0** primeiro partir | `composite.rs` (680) → `composite_otica.rs` (nova, metade de cima pura, `fn compoe_a_otica(&EntradaDaOtica, &[RenderLayer], &mut PlanosDaOtica) -> SaidaDaOtica`) + `composite.rs` (orquestração e píxel); `session.rs` (`PlanosDaOtica`, `Clone` em `:313`) | refactor ao byte; corre por nome `por_quadro_tests`, `oleo_cache_tests`, `showwet_tests`, `smooth_edges`, `fps_j13` |
| **P1** | `offthread.rs` (654) → `offthread_laco.rs` (`worker_loop` + `SimWorker::spawn`) | mudança pura de sítio; `leak_tests` e `offthread_tests` por nome |
| **P2** | `ph2d-wet-paint/src/painter.rs` (`espelho_sujo`, união em `:615`/`:660`) + nova `painter/espelho.rs` (cópia de linhas dos planos da óptica) | com o teste do espelho ao bit |
| **P3** (depois de P0) | nova `otica_factos.rs`; `composite_oleo.rs:148` passa a construir factos | interruptores capturados; ao byte |
| **P4** (depois de P2 e P3) | nova `otica_fora_tests.rs` + porta síncrona `#[cfg(test)]` | portão 1 e as mutações. **A exactidão fica provada antes das threads** |
| **P5** (depois de P1 e P2) | nova `offthread_otica.rs` (thread, canais, caixa de correio, `Drop`); gancho em `offthread_laco.rs` | regra da folga da cópia |
| **P6** (depois de P4 e P5) | `ciclo.rs` (tique), `veu.rs` (fatias), `authored_actions.rs` (fim) | pedido, aplicação, `uniao_local`/`divida`; portões 2 e 3; reler `offthread_tests :431/:564` |
| **P7** | `wet_diag.rs`, `wetpaint_fps.rs` | instrumentos, a flag de execução, a medição no app e a tabela no `J13_fps_do_real_oil.md` |

Nenhum ficheiro tocado passa de 700 linhas: `ciclo.rs` tem 248, `offthread.rs` desce para ~420 em P1 e `composite.rs` para ~400 em P0.

## 6. Riscos e o que medir no app

**Riscos:**
- **Memória do espelho:** cerca de 150–200 B por célula e por camada (a minha estimativa), por cima do motor. Medir `wet_bytes_retidos` a 1024² e 4096², e o `copia ms` com `Dirty::Full` a 4096². Se a 4096² passar o orçamento, a cópia por faixas fica limitada a passos fechados.
- **Escrita na grade que não passa por `merge_dirty`:** deixaria o espelho velho. Também já seria um defeito da composição de hoje. O portão 1 e a mutação do `espelho_sujo` apanham-no.
- **Contenção no rayon:** `passo ms` com o compositor ocupado. A cura é o pool próprio.
- **Interruptores `thread_local!`:** se algum ficar de fora do pedido, um teste ligado ao «caminho antigo» passa sem medir o novo.
- **Primeira composição do traço:** herda o sujo da água (já é assim hoje). Medir o pior quadro na rodada «traço do Dabber» depois de «parado».

**No app (`PH2D_WETPAINT_SMOKE=3 PH2D_FPS_OLEO=1 PH2D_FLUID_PROFILE=1`, com `PH2D_WET_OTICA_FORA=0|1` intercalado):**
- o período p50 e o máximo nas fases paradas. Alvo ~16–17 ms, contra 48–79 hoje;
- `passos/s` igual ou acima de 39 (hoje 15–27) e `away` baixo;
- `imagens/s`, `otica ms`, `copia ms`, `passo ms` com o compositor ocupado;
- as seis rodadas, sem resumir pelo mínimo (J13 §5).

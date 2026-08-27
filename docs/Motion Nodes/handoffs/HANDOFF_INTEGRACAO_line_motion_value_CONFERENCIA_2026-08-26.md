# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (2026-08-26)

> **A conferência dos nós (doc 89) foi de 22 P2 para 2.** Cinco folhas fecharam INTEIRAS nesta
> jornada (**06 animadores · 08 stream/utilidade · 13 sim stack · 15 value · 17 zero-param**),
> e as duas que ficam estão abertas **com o preço medido e o desenho escrito**.
>
> ⚠️ **A linha NÃO integra e NÃO pusha** (§0.7). Isto é o handoff; a ordem é do Enio.

## §1 — Identidade (§1.5.9 item 1)

| | |
|---|---|
| **Branch** | `line/motion-value` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| **Base (merge-base)** | `0f5ce8040c07742dc1bf7a5a2c5a7e8c2f41b6cb` — *"docs(roteador): o §5 do Motion Nodes dizia 68 P2…"* |
| **Commits** | **24** |
| **Diff DESTA linha** | **217 arquivos · +20 115 · −2 658** |
| **Quanto o `main` andou desde a base** | **158 commits · 419 arquivos** |

⚠️ **Não meça esta linha com `git diff main..HEAD`** — aquilo compara ÁRVORES e inclui tudo o
que o `main` ganhou desde o fork (dá *621 arquivos, −51 707*, e quase nada disso é meu). A
medida certa é a partir da `merge-base`, e é a da tabela.

## §2 — Foundational / compartilhado tocado (§1.5.9 item 2)

⭐ **Nenhum contrato congelado do §6 foi tocado** — os gates `architecture_contract_surface` e
`architecture_tool_contract_surface` passam. O que se acrescentou foram **params e portas de
nós individuais**, que não são o contrato (o §6 congela a contagem de campos do
`NodeManifest`, não o comprimento das listas de um nó — a folha 15 regista esse raciocínio).

Compartilhado de facto tocado:

- **`shells/desktop/src/render_loop/motion_bridge.rs`** — o **TERCEIRO produtor de leque de
  tempo** (`ph2d_node_motion_clone::fan::time_fans`), ao lado do `trail` e do `emitter`.
- **`shells/desktop/Cargo.toml` + `Cargo.lock`** — a dependência nova do shell no
  `ph2d-node-motion-clone`.
- **`crates/ph2d-render/`** — o passe do Flip passou a receber o sub-rect da cena (Bug #10).
- **`shells/desktop/src/motion_state_demo_router.rs`** — 11 cenas novas (`=96`..`=106`), e
  ⚠️ o `MAX_DEMO_LEVEL` subiu de **97 para 106** (ele estava parado em 97 com cenas até 102 —
  as `=98`..`=102` **nunca tinham sido diagnosticadas**).

## §3 — Símbolos e arquivos que podem COLIDIR (§1.5.9 item 3)

⚠️ **Rode `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` na worktree
ANTES do primeiro grep** (caminho ABSOLUTO do primário — uma worktree forkada antes do script
não o tem).

**Os 15 arquivos que as duas árvores tocaram desde a base:**

```
Cargo.lock
crates/ph2d-render/src/lib.rs
crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs
docs/architecture/decisions/0070-amendment-9.md
docs/architecture/decisions/README.md
project-memory/MEMORY.md
project-memory/feedback_bash_cwd_resets_and_slips_to_the_primary.md
shells/desktop/Cargo.toml
shells/desktop/src/input_dispatch.rs
shells/desktop/src/main.rs
shells/desktop/src/render_loop/mod.rs
shells/desktop/src/render_loop/motion_bridge_library.rs
shells/desktop/src/render_loop/sim_extract.rs
shells/desktop/src/render_loop/sim_extract_sheet_tests.rs
shells/desktop/src/render_loop/sim_extract_slice.rs
```

⚠️ **NÚMEROS QUE SOMAM ENTRE LINHAS — contam-se, não se escolhem** (§5.0):

- **`MAX_DEMO_LEVEL` e os braços do roteador de cenas.** Se outra linha acrescentou cenas, o
  número certo é o **máximo dos dois** e os braços são a **união**. O gate
  `no_two_smoke_scenes_claim_the_same_level` apanha a colisão de nível; ⛔ **ele não apanha o
  `MAX_DEMO_LEVEL` baixo demais** (ele mede o piso), e uma cena acima do teto simplesmente
  nunca é diagnosticada — foi o que aconteceu com as `=98`..`=102`.
- **`project-memory/MEMORY.md`** — o índice é append-only por linha; funde as duas listas.

## §4 — Contratos congelados (§1.5.9 item 4)

**Nenhum.** Nada aqui exige ADR.

## §5 — O que só o `ship.sh` pega (§1.5.9 item 5)

- ⚠️ **`typos`** apanhou três vezes nesta jornada — todas em prosa portuguesa (`transforme`,
  `REFLECTE`, `vectores`). ⛔ **Um `typos | tail` mente**: o pipe destrói o exit code.
- ⚠️ **`cargo fmt --all` re-expande** e pode empurrar um arquivo por cima do teto de LOC —
  rode os gates de LOC **depois** do fmt, nunca antes.
- ⚠️ **O gate do teto do shell chama-se `file_loc_caps` e é um teste de INTEGRAÇÃO**
  (`shells/desktop/tests/`). Um `cargo test -p ph2d-host-desktop -- file_loc_caps` corre o alvo
  **BIN** e devolve *"0 tests"* em VERDE — foi assim que ele passou despercebido meio dia.
  O comando certo é `cargo test -p ph2d-host-desktop --test file_loc_caps`.

## §6 — Ordem, e o que SMOKAR (§1.5.9 item 6)

Nada aqui depende de outra linha. As cenas, todas com o caminho da worktree:

```
env PH2D_GPU_COOK_DEMO=<n> cargo run -p ph2d-host-desktop --release
```

| cena | o que ela mostra | Play? |
|---|---|---|
| `=99` | a quicada que varia por peça | sim |
| `=100` | o estado angular (as peças giram, e o giro pára) | sim |
| `=101` | a caixa sólida — um obstáculo com quinas | sim |
| `=102` | a parede que absorve · o seed por peça | sim |
| `=103` | o relógio da simulação (`Forever`/`Once`/`Loop`) | sim |
| `=104` | o eixo do elemento · a máscara como valor (`Remap`) | **não** |
| `=105` | dois berços de onda (os produtores) | sim |
| `=106` | as cópias atrasadas no tempo | sim |

E a do `motion.path`, que precisa do documento vetorial:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_MOTION_NODE_PATH_SMOKE=4 cargo run -p ph2d-host-desktop --release
```

⚠️ **Todas já foram smokadas pelo Enio e aprovadas**, cena a cena, ao longo da jornada.

## §7 — O gate de fechamento, rodado

```
FMT                                  ✓ 0
architecture_workspace_file_loc_cap  ✓ 2 passed
file_loc_caps (shell, --test)        ✓ 2 passed
architecture_runtime_loc_cap         ✓ 1 passed
typos                                ✓ exit 0
clippy --all-targets (crates do diff) ✓ 0 avisos
nextest --workspace --cargo-profile ci-test --no-fail-fast
                                     ✓ 19 063 / 19 063, 0 falhas
```

⚠️⚠️ **DOIS achados de INFRAESTRUTURA que valem mais que qualquer feature daqui:**

1. **Uma corrida de fecho pode TRAVAR, e trava calada.** Uma corrida ficou **56 minutos** sem
   um byte de saída e parecia estar a compilar; não estava — o teste
   `project::tests::field::a_loaded_project_asks_to_open_the_panel_even_with_the_module_disarmed`
   estava **suspenso, com zero CPU**. Sozinho ele passa em **0,01 s**. O binário dele **abre o
   dispositivo de áudio de verdade**, e sob fan-out isso é um RECURSO PARTILHADO por onde se
   bloqueia. ⚠️ **Isto é pior que a família de flakes do §5.0**: um flake fica vermelho e
   avisa; um *hang* fica indistinguível de *"ainda a compilar"*. ⇒ **Diagnóstico:**
   `pgrep -P $(pgrep -f 'nextest run')` mostra o teste vivo e o `ps -o etimes=` diz há quanto
   tempo. ⛔ O `--slow-timeout` **não é aceite na linha de comando** nesta versão do nextest
   (é config de perfil) — pô-lo no `.config/nextest.toml` mudaria o CI de **todas** as linhas,
   e por isso **não o fiz**: é decisão do Enio, e está aqui como recomendação.
2. **Flakes de carga confirmados** (a família do §5.0, re-corridos 5/5 verdes sozinhos):
   `the_mask_stroke_cost_does_not_follow_the_canvas` ·
   `only_the_lower_row_breathes_and_it_moves_with_the_playhead` ·
   `flip_smooth::…::orcamento::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` ·
   ⭐ e um **membro NOVO** da mesma família, irmão dos três já listados:
   `flip_smooth::resample_measurement::precisao::**cache**::the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke`.
   ⚠️ O conjunto de reprovadas **mudou entre corridas do mesmo binário**, que é a assinatura.

## §8 — Aberto, com o preço ao lado (§1.5.9)

**A conferência está em 2 P2, zero P0/P1.** As duas ficam **por decisão, não por esquecimento**:

- ⏳ **`value.switch` — avaliação preguiçosa** (folha 15). **MEDIDO**
  (`measure_switch_laziness`): 4 ramos caros custam **10,805 ms** contra **2,769** de um só
  (**3,90×**) — mas o MESMO ramo nas quatro portas custa **2,851** (**1,03×**), ou seja **o memo
  já resolve o ramo partilhado** e metade do que a célula pedia já está entregue.
  ⚠️ **Duas pré-condições fazem disto uma wave do COOK:** o `select` pode ser um campo **por
  elemento** (feature documentada), e ⛔ **um ramo com ESTADO não pode ser saltado** (uma
  sub-árvore com `pre` congela, e o artista que voltasse a ela encontrava a sim parada no
  passado). **O desenho está escrito na célula** (`LazySelect` lateral no registry, ⛔ nunca um
  método novo no `NodeOp`, que é §6). **A recusa é de PROCESSO**: mexer no escalonador do
  `ph2d-nodegraph` — a crate mais partilhada do repo — merece a própria linha e o próprio
  portão, não a cauda de uma sessão.
- ⏳ **`fx.glow` — dirt texture** (folha 11). O preço já estava medido na folha: são **três
  fontes de textura** (`Atlas` · `Individual` · `CookedTexture`) até um passe de TELA, e cobrir
  só uma daria uma feature que funciona com umas imagens e **falha em silêncio** com outras.

Fora da conferência, herdado e ainda aberto:

- ⏳ **[Bug #7](../BUGS_motion_nodes.md)** — a fileira de mar da `=95`; adiado pelo Enio.
- ⏳ **`motion.boids` e `motion.wave`** seguem sem os tectos medidos (doc 91).

## §9 — Onde está o MECANISMO

Cada célula fechada tem o mecanismo **na própria folha** (`docs/Motion Nodes/89_conferencia/`),
e as mensagens de commit levam o raciocínio longo. O placar é **DERIVADO** —
`python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` — e ⚠️ **rode-o antes de citar
qualquer número**: ele imprime e sai, e quem reconcilia a linha `Contagem` de cada folha é
quem roda.

⭐ **As cinco leis que esta jornada pagou, e que valem para além dela:**

1. ***Curar o mecanismo não cura a aritmética.*** A aresta do ciclo da `sim.zone` perguntava
   *"em que fase eu estava em `t − dt`?"* e re-semeava a sim no 2.º tique de cada janela para
   sempre (`1,0 + dt − dt = 0,999999999999999`). Trocar a pergunta pelo mecanismo certo **não
   curou** — `since` é ele próprio uma subtracção e vem `6e-17` abaixo de `dt`.
2. ***Uma afirmação que mutação nenhuma mata é uma afirmação sobre nada.*** O `eval` da zona
   dizia *"com os defaults a maquinaria não corre"* e nenhuma mutação o matava. O que o
   curto-circuito compra é a zona **não perguntar as horas** — visível só num relógio negativo.
3. ***Uma varredura cujas células concordam todas não escolheu nada.*** A da esponja do
   `motion.wave` deu o mesmo valor nas **trinta** combinações; a grandeza que distingue era o
   **eco**, não a energia. E um óptimo na **borda** da grade significa grade curta demais.
4. ***Uma lei escrita N vezes ainda não é uma lei — só uma PORTA é.*** A do `motion.drive`
   estava em **oito** sítios (seis pares `apply`+`blend`, a mistura em duas closures iguais, e
   duas escritas noutra forma que só o compilador achou).
5. ***Encadear é conjunção, não união.*** Vale para colisores (folha 13) **e** para campos: na
   cena `=105` duas fontes encadeadas deram `falloff = 0,0000` em toda parte, com a porta
   ligada e o ganho posto — os dois tanques saíam byte-idênticos sobre plumbing correcto.

## §10 — Fecho da worktree (§1.5.9 item 7)

- Árvore **limpa**, 24 commits, nada por comitar.
- `rm -rf target/*/incremental` **por rodar** — deixo-o para quando a linha de facto parar, já
  que uma integração vai querer compilar aqui.
- ⚠️ **O `CLAUDE.md` §5 NÃO foi tocado** — ele edita-se **na INTEGRAÇÃO, no primário**
  (DIRETRIZ §1.5.6). A linha que o integrador tem de escrever lá:
  *"⭐ **2 P2, ZERO P1 e ZERO P0** na conferência (eram 22) — cinco folhas fecharam inteiras
  (06 · 08 · 13 · 15 · 17); as duas que ficam têm o preço medido e o desenho escrito. Cenas
  `=96..=106`."*

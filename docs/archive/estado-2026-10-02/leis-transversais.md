# ARQUIVO — CLAUDE.md (história, 305 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-141,447-461` do original.
>
> ⚠️ **A única alteração ao corpo:** 29 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Número que soma entre linhas se CONTA, nunca se escolhe** — `PROJECT_SCHEMA`, registros do `ph2d-ecs`, ids de
  scrollbar, números de ADR. O valor certo raramente está em qualquer um dos lados de um conflito, e ⚠️ **a colisão
  passa MUDA quando duas linhas escrevem o MESMO literal**: o git não sabe o que o número significa.
- **A fonte de cada número é o código, não esta seção** — `PROJECT_SCHEMA` em
  [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) (⚠️ com **a escada ao lado** e **a tripla** em
  [`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs) — **três** sítios, nunca um; e a
  `line/physics` **partiu** aquele arquivo em 15/08, então um degrau escrito no `project.rs` funde **limpo** e evapora)
  · `VEC_SCENE_SCHEMA_VERSION` / `FLIP_SCHEMA_VERSION` / `DOC_VERSION` nas crates que os declaram.
  ⛔ **Não copie esses valores para cá** — esta seção já os teve errados cinco vezes.
- **O número da próxima cena de smoke se CONTA lendo o roteador**, nunca uma nota (a nota já envelheceu em 11 cenas
  de uma vez): Motion [`motion_state_demo_router.rs`](../../../crates/ph2d-app-motion/src/motion_state_demo_router.rs) · Física
  [`smoke.rs`](../../../crates/ph2d-app-physics/src/smoke.rs) (`CENAS`) · os gates `no_two_*_scenes_claim_the_same_level`.
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
- **Índice de diretório se GERA, não se escreve** — `bash scripts/doc-index.sh` (`--check`
  no `ship.sh`). Medido: `docs/Motion Nodes/` tinha **99 arquivos e zero índice**, e **45%**
  dos markdowns eram inalcançáveis a partir deste roteador. ⛔ `docs/Pixel Art/` e `docs/Tilling/`
  ficam de fora **por decisão do Enio** (estão no `.gitignore`: MVPs paralelos ainda sem associação
  com o PH2D) — o §5 não as mencionar **não é buraco, é o produto de uma decisão**.
- ⚠️ **O stack subiu em 2026-08-29** (`wgpu` 29 · `vello` 0.10 · `parley` 0.11 · `bevy_ecs` 0.19 ·
  `rapier2d` **0.35**, cuja matemática deixou de ser `nalgebra` e passou a `glam`/`glamx` — o vocabulário
  vive em [`rmath.rs`](../../../crates/ph2d-physics/src/rmath.rs) e ⛔ `Point`/`Isometry`/`Translation` **não
  existem**). ⛔ **O que NÃO subiu ficou por MEDIÇÃO, não por preguiça** — o `wgpu` 30 é inalcançável
  enquanto o `vello` pedir `^29.0.3`, e unificar o `glam` desligaria o SIMD de 8 crates de desenho:
  [ADR-0168](../../architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md)
  + [registo](../../Atualizar%20Stack/04_registro.md). ⚠️ **Antes de responder «dá para atualizar X?»,
  corra `bash scripts/stack-audit.sh --tetos`** — o `ship.sh` já o imprime antes do veredito de push.
- ⛔⛔ **UM CONTROLO MORTO tem DUAS espécies que nenhuma sonda deste repo apanha** (caça de
  2026-08-30: **34 mortos** sobre **~504 controlos** seguidos até ao efeito, registo em
  [§19](../../Atualizar%20Stack/04_registro.md)). ⚠️ **O passo que um `grep` não vê é o terceiro:**
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
- ⛔⛔ **Um SWEEP sobre um ficheiro COMPRIMIDO não prova nada sobre o conteúdo dele** (medido
  14/09): o `cleanroom-sweep.sh` lê binários por `strings`, e um `.gz` é **opaco** a isso — um
  canário plantado dentro de um passava despercebido. ⇒ a cegueira valia para **todos** os corpora
  de oráculo deste repo, que são guardados comprimidos (o tecido, a pose, os pincéis desbloqueados,
  o dyntopo). O instrumento **descomprime em memória** desde então, e reconhece o formato pelos
  **dois bytes mágicos** e nunca pela extensão. ⭐ *A cura só vale porque tem controlo positivo: com
  um canário plantado ele acusa e sai `exit = 1`* — e sobre as seis vassouras vivas as fixturas
  estavam limpas, só que até aí o instrumento não o conseguia provar.
  ⛔⛔ **E em 16/09 essa cura foi medida a valer METADE:** o ramo do `.gz` fazia só o `grep` **linha
  a linha** e não a passagem que desfaz as **quebras de linha e a ênfase**, que o ramo de texto já
  tinha ⇒ a **mesma** frase era acusada quando cabia numa linha do cabeçalho e passava **LIMPA**
  partida em duas dentro do comprimido. ⭐ *Uma cura que abre um caminho NOVO herda as cegueiras que
  o caminho antigo já tinha pago* — e a forma de o impedir é a passagem ser uma **função com dois
  chamadores** (`achados_desdobrados`), porque um ramo que se esqueça dela fica visível por
  **AUSÊNCIA de chamada**, enquanto duas cópias divergem outra vez em silêncio. Controlo dos dois
  lados: o instrumento de ontem lê `exit 0` sobre o canário dobrado dentro de um `.gz` e o de hoje
  lê `exit 1`, sem falso positivo no corpus sem canário e com as **oito** vassouras vivas verdes.
- ⛔⛔ **Uma catraca sem censo de obsolescência não desce: ela vira LICENÇA.** Toda lista de dívida
  tolerada deste repo declara-se «só encolhe», e nenhuma encolhe sozinha. Medido 30/08: a lista de
  folgas de LOC por **função** tinha o censo; a de **ficheiros** não tinha, e ao escrevê-lo ele
  acusou **três** entradas obsoletas na primeira corrida — uma delas congelada em `660` havia três
  meses sobre um ficheiro de **536** linhas. ⇒ ao criar uma tolerância, escreva no mesmo commit o
  teste que pergunta *o alvo ainda existe? ainda estoura? a folga ainda o descreve?* — com a
  metade justa, senão uma varredura partida devolve zero obsoletas e lê-se como aprovado.
- ⛔⛔ **Um TETO DE LOC pode ficar vermelho por ACUMULAÇÃO, e nenhum portão de fecho o vê.** Medido
  na integração de 2026-09-10: o `screens/hero.rs` chegou a `709` contra o teto de `700` somando
  **três** linhas que fecharam no mesmo dia — e nenhuma delas o estoura sozinha, logo as três
  fecharam verdes de boa-fé. *Um teto por-ficheiro é a única grandeza deste repo que SOMA entre
  linhas sem ninguém a contar* (os schemas somam e há sonda; este soma e não há). ⇒ a cura é do
  **integrador** e é **corte por responsabilidade**, nunca uma entrada nova no `FILE_OVERAGE_OK`.
  ⚠️ E um ficheiro com marcador textual de isenção (`// ph2d-loc-cap:`) é imune — o que engana é
  que os dois se leem igual numa tabela de risco.
  ⚠️⚠️ **E há um ficheiro cuja ACUMULAÇÃO é estrutural: a ESCADA do `PROJECT_SCHEMA`.** Ela ganha
  *um parágrafo por wave e nunca perde um*, logo o tecto dela não mede autor nenhum — mede o
  TAMANHO DA RODADA. Na de 2026-09-17 seis linhas puseram **dezasseis** degraus (`128` → `144`) e o
  `project_schema.rs` foi a `651`; a cura é a **quarta** faixa arquivada
  ([`project_schema_history_v112`](../../../shells/desktop/src/project_schema_history_v112.rs), `v112`..`v128`),
  a seguir às de `v82`, `v83` e `v99`. ⭐ **A fronteira de uma faixa é o `main` de que a rodada
  nasceu**, não um número confortável: abaixo dele estão rodadas fechadas, acima está o que alguém
  a contar o próximo degrau precisa de ver — e a **constante fica colada à ponta**, que é a lei do
  degrau `v69`. ⚠️ Antes de cortar, `grep 'mod project_schema_history'` no `main.rs`: **o padrão já
  existe com três irmãos**, e a 1.ª tentativa daquela integração escreveu por cima do primeiro
  deles (490 linhas de história) por não o ter olhado. ⚠️ **Hoje são SEIS faixas** — a sexta,
  `v144`..`v160`, na rodada 03 de 25/09 (`160` → `176`, dezasseis degraus outra vez: o tamanho da
  rodada, não de um autor; [registo](../integracao-jornadas/HANDOFF_INTEGRACAO_rodada03_2026-09-25.md)).
- ⛔⛔ **UMA ISENÇÃO DE CENSO É PROPRIEDADE DO CÓDIGO, NÃO DO SÍTIO ONDE O CÓDIGO ESTÁ — ela VIAJA.**
  Medido três vezes na integração de 2026-09-17: um ficheiro sai da shell para a crate da família e
  a linha do `FORA` fica **órfã** de um lado enquanto os literais ficam **sem abrigo** do outro.
  ⭐ **As duas metades a acusar na MESMA corrida são o que prova que é uma MUDANÇA DE ENDEREÇO e não
  texto novo** — cada uma sozinha mente: a do órfão lê-se como *«alguém apagou isto»* e a do literal
  como *«alguém escreveu texto novo»*, e **as duas curas seriam erradas**.
  ⛔⛔ **E há um caso em que a cura NÃO é mover a isenção, é mover o FICHEIRO:** a régua
  (`language_literals_uncached`) **salta o que está declarado sob `#[cfg(test)]`**, logo um ficheiro
  de teste que o `git mv` deixou para trás **deixa de ser saltado** e passa a ler-se como PRODUTO.
  Ele aparece como *«literais novos numa fixtura»* e é um **ÓRFÃO** — escrever-lhe uma isenção
  calaria o instrumento para sempre. ⇒ *antes de isentar um literal que apareceu num `_tests.rs`,
  pergunte quem o declara.*
- ⛔⛔ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente** — a
  ausente não é acreditada. Medido em 2026-08-30: a `=15` prometia que a bola sem CCD atravessa a
  parede, e as duas paravam **no mesmo sítio** desde a `rapier` 0.35 (que varre contra cenário
  **fixo** de graça). ⚠️ **O doc da biblioteca já estava corrigido; a CENA é que não foi** — quando
  um comportamento muda, o smoke que o demonstra é o **último** sítio a ser lembrado e o **primeiro**
  que o Enio lê (§0.8).
- ⛔⛔ **MOVER CÓDIGO PARTE GATES EM DUAS ESPÉCIES, e só UMA avisa** (medido na W2/L0, 11/09:
  **15** correcções ao mover 90 ficheiros). ⚠️ *A que falha alto é a barata; a que fica VERDE é a
  que se leva para o main.*
  - **Falha alto** (11): `include_str!` com caminho relativo · o gémeo em runtime
    (`read_to_string(CARGO_MANIFEST_DIR/…)`, que só falha **quando o teste corre** — um `#[ignore]`
    ou um filtro e ele nunca falha) · valores esperados que são **nomes de ficheiro** · a agulha que
    nomeia um endereço de fiação.
  - ⛔ **Fica MUDA** (4): um censo que varre um directório por **prefixo de nome** (`starts_with(
    "field3d_")`) passa a varrer **zero** ficheiros, e `bad.is_empty()` sobre uma lista vazia é
    trivialmente verdadeiro. A cura é um **piso de população** no próprio gate, e ele fica **mais
    forte do que era antes da mudança**.
  - ⚠️ **E três coisas que só a fronteira nova revela:** uma **feature** não viaja com o código (um
    `#[cfg(feature = "x")]` numa crate que não a declara é falso **por construção** — o piloto
    compilou verde com o matcap desligado) · um `#[cfg(test)]` é **invisível** do outro lado da
    crate (⇒ feature `test-support`, do tamanho do que ATRAVESSA: 1 item de 16) · e uma fronteira
    põe um **elo novo** na corrente que nenhum gate mede (a família pergunta ao trait, e uma
    implementação que respondesse `false` deixava os dois lados verdes).
  ⛔⛔⛔ **E há uma TERCEIRA espécie, medida em 20/09: mover um ficheiro troca o REGIME DE
    VISIBILIDADE que o governa — e ela é muda no LINUX.** Num módulo de BINÁRIO `pub` é decoração;
    numa crate de BIBLIOTECA `pub` é API e privado-sem-chamador é **código morto**. Uma função com
    dois braços `#[cfg(target_os)]`, o do Linux `pub` e o outro privado, com o único chamador a ser
    um `#[test]`, passou o portão da linha, os censos da árvore somada **e o `ship.sh`
    (25 844/25 844)** — e o CI reprovou em **macOS E Windows** (o passe dele é `cargo check` **sem
    `--all-targets`** ⇒ `cfg(test)` desligado, e `build.warnings = "deny"`). ⇒ *quem escreve ou move
    um `#[cfg(target_os` cruza-o antes de fechar* (`cargo check -p <crate> --target
    aarch64-apple-darwin`; ⚠️ **não há alvo de Windows nesta máquina** e ali o CI é a única régua) —
    HOWTO **§2.21** e DIRETRIZ §1.5.9 item 5.
  Molde inteiro, com os números: [`HOWTO_partir_uma_familia_da_shell.md`](../../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md).
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
  [`estado-2026-08-18/README.md`](../estado-2026-08-18/README.md).
- ⚠️ **O que fica vivo tem de ser ENDEREÇÁVEL.** Medido: o agente não lê estes docs, ele os **navega** —
  o padrão de busca nº 1 em todos eles é reconstruir o sumário (`'^## '`), depois saltar para um
  endereço (`HR-5`, `Bug #17`, `W6.2`, `§1.5.9`) e ler ~70 linhas. É por isso que o `SKILL_Stack`
  (0% de história, consultado por `HR-N`) funciona e um diário numerado fora de ordem não.
- ⚠️ **As listas de `Smokes:` abaixo são NOMES de variável, não o comando.** Ao passar um smoke ao Enio, escreva-o **inteiro e copiável de uma vez**, com o caminho absoluto da árvore em que você trabalha (§0.8):
  ```
  cd /home/enio/Documentos/Projetos/PH2D && env PH2D_<NOME>=<n> cargo run -p ph2d-host-desktop --profile smoke
  ```
  ⚠️ **`--profile smoke`, não `--release`** (desde 2026-09-10): o `release` optimiza a shell (UMA unidade) num
  só thread e cada correcção pós-smoke custava **161 s**; o `smoke` custa **3 s** (medido, [auditoria](../../DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md) §3.9). O `--release` fica só para smoke de
  **PERFORMANCE** (tectos, milhões de objectos) — o `smoke` não tem LTO e corre mais devagar.
  ⚠️ **Modo L: o caminho é o da SUA worktree** (`.../PH2D/Worktrees/line-<módulo>`), não o do primário — o Enio roda de outro diretório, e sem o `cd` o comando falha ou testa a árvore errada.
- ⚠️ **Nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`** (medido: o mesmo binário deu 11,36 e
  5,50 ms para o mesmo passe). Gates de razão reprovam sob carga sem que uma linha de código tenha mudado.
  ⚠️⚠️ **FLAKES DE RECURSO SOB FAN-OUT — é uma FAMÍLIA, não uma lista: pare de as contar uma a uma.**
  A forma que se lê não é o nome, é o MECANISMO: um gate que mede um RECURSO partilhado — razão de
  dois relógios · contagem de alocações · o que vier — reprova sob 10–18 mil testes em paralelo e
  passa sozinho na máquina calma. O sinal de que é carga: o mesmo teste verde isolado (3–5 de 3–5),
  o diff sem uma linha no módulo dele — e num grupo, o **CONJUNTO de reprovadas MUDA entre corridas
  do mesmo binário** (um defeito de lógica reprova o mesmo caso sempre). ⇒ *re-rode sozinho ANTES de
  olhar para o seu commit*. O `fail-fast = false` é o default do `.config/nextest.toml` desde 10/09 —
  antes o nextest cancelava no 1º ✗ e **escondia o resto da suíte** (uma corrida parou em 11.240 com
  1.007 por correr); numa árvore mais velha, `--no-fail-fast` à mão.
  **Membros confirmados (2026-08-16..23):** `a_round_live_offset_costs_like_the_other_joins`
  ([`ph2d-vec-boolean`](../../../crates/ph2d-vec-boolean/tests/it/offset_live_cost.rs) — o caso canónico: único
  ✗ de 15.323, no pico do fan-out, commit sem uma linha de produção) ·
  `the_cost_of_depth_is_linear_not_explosive` (Timeline) ·
  `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` ·
  `the_mask_stroke_cost_does_not_follow_the_canvas` ·
  `the_brush_snapshot_costs_the_same_on_a_canvas_sixteen_times_bigger` (as três em
  [`ph2d-tool-painter`](../../../crates/ph2d-tool-painter) — uma delas com doc a dizer-se *"imune à deriva
  da máquina"* por medir uma RAZÃO, que é precisamente o que o fan-out quebra) ·
  `only_the_lower_row_breathes_and_it_moves_with_the_playhead` (demos de áudio, «max delta 0») ·
  `an_abandoned_march_returns_nothing_and_returns_fast`
  ([`ph2d-field-render`](../../../crates/ph2d-field-render/src/tests.rs) — mede um relógio de desistência) ·
  `emitter_sim_ceiling_probe` ([`ph2d-gpu-cook`](../../../crates/ph2d-gpu-cook/tests/it/gpu_cpu_parity_sim.rs) —
  ⚠️ **`#[ignore]`, logo o CI nunca o correu**; os dois medidos em 2026-08-29, na subida do stack) ·
  a família `flip_smooth::resample_measurement::precisao::orcamento` — **3 testes** em
  [`flip_fit_budget_tests.rs`](../../../crates/ph2d-app-flip/src/fit_budget_tests.rs), medida 22/08 pela
  `line/3DModeling` e confirmada 23/08 pela `line/sculpt3d`, com a falha a MUDAR de teste entre
  corridas · `the_cost_of_sampling_a_path_is_flat_in_its_anchors` (Timeline) ·
  `the_region_refresh_is_bound_by_the_footprint_not_by_the_mesh`
  ([`ph2d-mesh`](../../../crates/ph2d-mesh/tests/it/measure_normals.rs) — ⚠️ o doc-comment declara-se imune,
  *«o gate é a FORMA, não o relógio»*, e a forma é medida DIVIDINDO dois relógios: *um gate que se
  diz independente do relógio ainda o é, se o numerador e o denominador forem tempos*) ·
  `measure_brush_kernel` ([`ph2d-sculpt3d`](../../../crates/ph2d-sculpt3d/tests/it/measure_brush_kernel.rs) —
  cara, 34 s sozinha, no pico do fan-out por construção) ·
  `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
  ([`ph2d-tool-painter`](../../../crates/ph2d-tool-painter) — promovido 2026-09-20 pela `line/Vector`: único
  ✗ de **15 644** a `load 10,70`, **3 de 3 verde sozinho a `load 4,44`**, e **zero linhas** do diff
  daquela rodada naquela crate) ·
  `packing_a_dense_scribble_is_bounded` ([`ph2d-flip-render`](../../../crates/ph2d-flip-render/tests/it/pack_perf.rs)
  — medido 2026-09-01 numa corrida de **20 309** testes, verde **3 de 3** sozinho e com **zero
  linhas** do diff naquela crate) ·
  `measure_normals_parallel_speedup` ([`ph2d-mesh`](../../../crates/ph2d-mesh/tests/it/measure_normals.rs) —
  o **segundo** membro deste ficheiro; mede a razão paralelo/série sobre uma esfera de 5 M
  triângulos)
  ⚠️⚠️ **E o «sozinho» da assinatura quer dizer *com a CARGA MEDIDA*, não *sem filtro*** — em
  2026-09-02 eu li `measure_normals_parallel_speedup` como **`3 de 3 VERMELHO` sozinho** e quase
  o arquivei como defeito real; a máquina estava a **load 82** (a suíte de 20 316 ainda a
  esvaziar). Com `load 3,2`: **3 de 3 verde**. *Imprima o `/proc/loadavg` AO LADO de cada
  corrida de confirmação, senão a régua que desmente a flake é a própria flake.* · e ⚠️ **as TRÊS de ALOCAÇÃO**, espécie
  própria: `apply_from_doc_is_zero_alloc_steady_state` (ph2d-timeline) ·
  `the_trusted_len_collect_allocates_once` (ph2d-audio-edit) · e
  **`no_expression_allocates_no_link_frame`** (ph2d-timeline, promovido pela integração da W2 Fase C
  em 2026-09-12 a pedido da `line/app-motion`) — um contador de alocações parece
  imune a carga e não é: sob fan-out o alocador global reutiliza arenas de outra maneira.
  ⭐⭐ **A terceira trouxe a assinatura mais limpa da família até hoje: ela PASSA a `load 38–48` e
  reprova no meio de um fan-out de 16 245** ⇒ *o discriminador é o FAN-OUT, não o relógio* — o
  mesmo achado que a nota do `flip_fit_cache` já registava, agora com o controlo do lado que passa.
  ⚠️ E ela é **irmã de ficheiro** da primeira desta lista, que passou na mesma corrida: *duas
  contagens de alocação na mesma crate, uma na lista e outra não, é exactamente como a lista
  envelhece* ·
  `the_shape_match_is_linear_in_the_mesh`
  ([`ph2d-node-motion-soft-body`](../../../crates/ph2d-node-motion-soft-body) — confirmado 2026-09-01
  pelas TRÊS assinaturas: gate de razão · **zero linhas de diff** da linha acusada naquela crate ·
  5 de 5 verde sozinha · e o **conjunto de reprovadas MUDOU** entre duas corridas da mesma árvore,
  onde a corrida anterior tinha acusado outro teste, esse **real**).
  `the_cost_of_a_player_is_linear_in_their_number`
  ([`ph2d-physics-ecs`](../../../crates/ph2d-physics-ecs/tests/it/measure_player_budget.rs) — promovido pela
  integração de 2026-09-07, a pedido da `line/UIUX`; ⚠️ **é o TERCEIRO cujo doc-comment se declara
  imune** (*«gate de FORMA, não de relógio … uma razão entre duas contagens»*) e cujos dois lados são
  `ms_per_tick`: *dividir dois relógios não deixa de ser um relógio por a razão ser adimensional*).
  **Promovidos pela integração de 2026-09-10** (a linha pede, o integrador escreve):
  `the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke` (razão de dois relógios · 3/3 a
  `load 4,69`) · `interaction_dispatch_no_alloc` (contador de alocações · 3/3 a `load 2,20`; ⚠️ o
  ficheiro **não menciona texto** e o diff acusado era da `ph2d-text`) ·
  `the_ui_clock_does_not_allocate_per_frame` (contador de alocações · 3/3 a `load 18,20`/`18,99` —
  ⛔ **o QUARTO deste repo cujo doc-comment se declarava imune**, *«uma contagem de blocos é
  determinística e não flaka»*) · e ⭐ **`o_pen_down_do_filtro_e_linear_nos_vertices`**
  ([`ph2d-sculpt3d`](../../../crates/ph2d-sculpt3d/tests/it/mede_o_filtro_de_tecido.rs)), **medido na própria
  rodada e a espécie mais subtil da família: ele ajusta um EXPOENTE a cinco relógios de parede.**
  Sob os 14 911 testes em paralelo o ponto de `6 836` vértices lê `18,99 ms` (`2,9×` o isolado)
  enquanto os outros quatro leem `~1,4×`, e **um ponto** puxa o ajuste log-log de `1,03` para
  `1,31`, acima da barra de linearidade. *Um gate que ajusta uma curva a relógios é mais frágil que
  um que compara duas medianas: basta UMA amostra deslocada.*
  ⭐⭐ **E esta rodada produziu a assinatura mais forte da família, à vista:** **três** corridas da
  **MESMA** árvore (`line/sculpt3d`, o mesmo commit) devolveram **três conjuntos de reprovadas
  DIFERENTES** — `{an_abandoned_march, o_pen_down}` · `{the_brush_snapshot}` · `{an_abandoned_march}`
  —, todos gates de relógio, todos verdes 3/3 sozinhos. *Um defeito de lógica reprova o mesmo caso
  sempre; só um recurso partilhado troca de vítima entre corridas.*
  ⚠️ **E uma quinta ficou por promover porque NÃO TEM NOME:** a `line/Vector` reportou *«um do shell
  que não reproduziu em 4 corridas»* sem o nomear. *Uma flake sem nome não entra numa lista — quem
  a encontrar outra vez recomeça do zero.*
  **Promovido pela integração de 2026-09-17:** `sub_stepping_costs_what_it_says_it_costs`
  ([`ph2d-physics`](../../../crates/ph2d-physics/tests/it/penetration.rs)) — reprovou no meio de um fan-out de
  **24 368** testes do `ship.sh` e passa **3 de 3 sozinho a 95–96 % de CPU OCIOSA**, com zero linhas
  do diff da rodada naquele caminho. ⛔ É o **SEXTO** deste repo cujo doc-comment se declara imune
  por escrito (*«A RATIO against the single-step cost, not a wall-clock bar»*, e a frase seguinte
  explica que é para não medir o perfil de build) — ⚠️ **verdade sobre o PERFIL e falso sobre o
  FAN-OUT**, que é exactamente a distinção que esta lista existe para guardar.
  *Todo gate que compara duas medianas de um RECURSO é candidato, e a lista nunca estará completa.*
  **Promovidos pela integração de 2026-09-25** (a pedido da `line/PainterWatercolor`):
  `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` ([`ph2d-tool-painter`](../../../crates/ph2d-tool-painter),
  `mask_gate_tests` — 3/3 verde sozinho a `load ~7`, zero linhas de diff) · `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
  (`measure_input_cost.rs`; §31.3 do diário arquivado da linha).
  **Promovido pela integração de 2026-10-02** (a pedido da `line/motion-value`):
  `tres_bonecos_tres_amplitudes_e_o_rapido_acende_a_lampada` ([`ph2d-app-components`](../../../crates/ph2d-app-components) —
  reprovou com os bonecos parados a `load ~60`, 3/3 verde sozinho a `load 26`–`41`, zero linhas de diff; o prazo de um
  quadro por gancho do Luau mede RELÓGIO e sob fan-out estoura) · e (a pedido da `line/components`)
  `text_path_smoke::perf::riding_the_path_costs_about_twice_the_straight_layout` (shell — razão de dois relógios,
  reprovou **uma** vez em `18 735` com outra linha na placa, 3/3 verde sozinho a `load 7,3`, zero linhas de diff
  naquele módulo).
  ⛔ **E um CONTADOR atrás de estado POR THREAD (memo, arena) também** — ele conta quantas threads o escalonador pôs a
  trabalhar: o gate da superfórmula leu `morno 0/4/8` sob fan-out e foi curado numa pool de UMA thread (13/09, ESTADO W2 §6).
- ⛔⛔ **E há uma flake de GPU que NÃO é `#[ignore]` e só existe no LINUX do CI** (medido
  2026-09-17): `atlas::tests::remove_of_missing_key_returns_none` (`ph2d-render`) saiu **SIGSEGV**,
  `1` de `1300`, com **macOS e Windows verdes na mesma corrida** — e a re-corrida do MESMO commit
  passou. ⚠️ **O módulo do atlas não tinha uma linha de diff naquela rodada de seis linhas**, e os
  dois ficheiros de teste de GPU novos dela são todos `#[ignore]`, logo nem correram. ⇒ o que
  estoura é o adaptador por SOFTWARE do runner (`lavapipe`), não a lei: ali o `try_headless_gpu()`
  devolve `Some` — *um adaptador que existe e é instável é pior que nenhum, porque o caminho de
  skip gracioso nunca arma*. ⚠️ Antes de investigar um SIGSEGV de GPU no CI, **pergunte se os
  outros dois OS passaram e se o módulo tem diff**: as duas respostas separam ambiente de defeito
  em dois comandos, e a cura é `gh run rerun --failed`.
- ⚠️ **Gates de GPU são `#[ignore]`** e precisam de adapter — *skip gracioso não é verde*; e o `nextest` só deixou de
  **cancelar na primeira falha** em 10/09 (`fail-fast = false` no `.config/nextest.toml`) — numa árvore mais
  velha, `--no-fail-fast`, senão suítes inteiras nunca chegam a correr. E desde o mesmo dia **todo teste tem
  tecto** (60 s avisa, 180 s mata): quem precisa de mais pede por `[[overrides]]` com o número medido.

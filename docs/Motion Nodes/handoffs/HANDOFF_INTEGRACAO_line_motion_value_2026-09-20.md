# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-09-20

**Para o INTEGRADOR.** Esta é a entrega FINAL da linha. ⛔ Ela **não** integra e **não** faz ship
([`CLAUDE.md §0.7`](../../../CLAUDE.md)) — entrega isto e para.

> ⚠️ **Ela SUPERSEDE o [handoff de 18/09](HANDOFF_INTEGRACAO_line_motion_value_2026-09-18.md)** como
> documento de integração: aquele foi escrito para uma rodada que não aconteceu, e a linha andou
> **mais 51 commits** depois dele. ⛔ **O MECANISMO das waves até 18/09 continua a viver lá** (e o
> das três de 19/09 no [de continuação](HANDOFF_CONTINUACAO_line_motion_value_2026-09-19.md)) —
> este documento não o repete. *O que está aqui é o que o integrador precisa; o que está lá é o que
> a próxima janela precisa.*

---

## §0 — IDENTIDADE

| | |
|---|---|
| branch | `line/motion-value` |
| HEAD | *(ver §7 — o sha final está lá, depois do último commit)* |
| merge-base | **`76bd6de02`** (o tip do `main` no fecho — a linha foi **rebaseada** em 2026-09-20) |
| commits | **113** |
| ficheiros | **364** · `+33 157 / −10 656` |
| jornadas | 17/09 (34) · 18/09 (31) · 19/09 (41) · 20/09 (7) |
| ADR criados | **nenhum** |
| contratos congelados | **nenhum tocado** (prova na §2) |

⚠️ **O rebase conflitou em 2 ficheiros e os DOIS são prosa de memória**
(`project-memory/reference_topic_{mutation_proofs,gate_discipline}.md`): os dois lados
ACRESCENTAM no fim, e a resolução foi **ficar com as duas metades**. Zero conflitos em código.

---

## §1 — O QUE A LINHA ENTREGA, em TRÊS assuntos

### (a) Até 18/09 — ciclo 9 (RIG & CORPOS MOLES) + doc 115 (o COLISOR sai do grafo)

O eixo, wave a wave, está na §1 do [handoff de 18/09](HANDOFF_INTEGRACAO_line_motion_value_2026-09-18.md).
⭐ O que o integrador tem de saber em uma linha: **o passe de contacto passou a correr no fim do
cozimento** com um interruptor no sink, e a perseguição de performance que os reports do dono
abriram fechou em **`46×`** no caminho dele (`157,9 → 6,8 ms` a 500 objectos × 1024 varreduras), com
**`40`–`50` FPS a 1000 objectos** aprovados por ele.

### (b) 19/09 — as POSIÇÕES deixam de virar pixel, e o GIZMO ocupa o lugar

Mecanismo no [handoff de continuação](HANDOFF_CONTINUACAO_line_motion_value_2026-09-19.md) e no
doc 115 §32. ⭐ **`111` das `123` cenas desenham SÓ posições** (medido), e uma corrente de posições
passou a desenhar uma **marca** em vez de uma forma; o gizmo ganhou secção no cartão (tamanho
absoluto + três formas), e o campo *Shape* saiu dos quatro distribuidores por ordem do dono.

### (c) 19–20/09 — o CARTÃO do grafo: o osso, o pivô, e o nó de LONGE

| wave | o que ficou | smoke |
|---|---|---|
| o OSSO | `rig.bones` (crate nova) — a corrente entrega **JUNTAS** e o osso vive **ATRÁS** de uma; o `rot` que sai é o ângulo de MUNDO | ✅ aprovado |
| o PIVÔ | param do cartão na `motion.shape`, unidade = o `size`, com ALVO aceso durante o arrasto | ✅ aprovado |
| a LARGADA | arrastar uma carta sobre outra **TROCA**; sobre um fio, **ENFIA** sem quebrar a cadeia; o fio aterra na porta **PRINCIPAL** (`landing_port`) | ✅ aprovado |
| o ECO | realce antes de largar, piscada depois (3 vezes), e **um vão apertado ABRE-SE** | ✅ aprovado |
| o LOD | o texto do cartão sobrevive **20 %** mais zoom (o número é do dono, e o que o paga é uma re-medição: uma row de param passou de `13,5` para **`2,2 µs`**) | ✅ aprovado |
| a CÁPSULA | abaixo do limiar o nó vira um **RECTÂNGULO do tamanho do nome**, cor do grupo, fonte única **`+60 %`** sobre a 1.ª versão, pinos grandes, e os **FIOS param de encolher** | ✅ aprovado |

⭐⭐ **Três reprovações do dono nesta última wave, e as três eram MINHAS leituras** — cada uma com a
causa medida na §4.

---

## §2 — SUPERFÍCIE DE COLISÃO, medida (`collision-surface.sh`, PÓS-rebase)

```
  merge-base 76bd6de02   ·   113 commit(s)   ·   364 arquivo(s)
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        144   (base: 144)
      └ tripla do gate               (144, 13, 22)   (base: (144, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      23   (base: 23)
▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               91   (base: 91)
    ph2d-render (espelho)                  92   (base: 92)
    ph2d-script (espelho)                  92   (base: 92)
▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — último no disco: 0170 · esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 1 pacote '+name' novo: "ph2d-node-rig-bones"   (aresta INTERNA, não dependência externa)
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⭐⭐⭐ **ZERO em todos os contadores que somam entre linhas.** Esta linha não move um único schema,
não regista um componente, não escolhe um número de ADR e não encosta em contrato congelado.

⚠️ **PRAZO DE VALIDADE:** a tabela mede contra o `main` de **2026-09-20**. Se a integração for
noutro dia, com linhas fundidas no meio, ⛔ **leia o valor do `main` no FICHEIRO** (`git show
main:<arq>`), nunca a coluna `base:` — ela é o merge-base e só muda com o rebase (DIRETRIZ §1.5.9
item 3).

### §2.1 ⛔⛔⛔ O QUE A TABELA **NÃO** MOSTRA — e é o item nº 1 desta integração

**A crate `ph2d-panel-motion-params` foi APAGADA** (commit `d878cf32c`, ordem do dono de 17/09
fixada em *«Só o painel de parâmetros do Motion»*): **35 ficheiros, 7 551 linhas, 66 testes**. Com
ela saem, e cada uma é uma **REMOÇÃO num ficheiro partilhado**:

| onde | o que sai |
|---|---|
| `Cargo.lock` | `-name = "ph2d-panel-motion-params"` |
| `shells/desktop/Cargo.toml` | a dependência (2 linhas) |
| `ph2d-editor-core/tests/it/node_id_collisions.rs` | **2 entradas** do array de espécies de id |
| `ph2d-editor-core/tests/it/hr12_widgets_a11y.rs` | **1 entrada** da lista de isenções |
| `ph2d-editor-core/tests/it/architecture_motion_chrome_never_wraps_a_row_label.rs` | a varredura reaponta |
| `ph2d-editor-core/src/widget/scrollbar_ids.rs` | o id fica **ÓRFÃO, declarado como tal** (não foi apagado: outro painel pode reclamá-lo) |

⛔⛔ **ESTE É O CASO CANÓNICO DO MERGIRAF A APAGAR UMA REMOÇÃO E DIZER «SOLVED»**
([memória](../../../project-memory/feedback_mergiraf_silently_drops_a_deletion_in_a_list_and_says_solved.md)).
Se a fusão textual reintroduzir qualquer uma dessas entradas, o sintoma é **um censo a apontar para
uma crate que não existe** — e o `cargo check` da árvore combinada apanha-o, porque as entradas são
caminhos lidos por `include_str!`/`fs::read_to_string` em tempo de TESTE, não de compilação. ⇒
**confira as seis linhas acima à mão depois do merge.**

⭐ E o `scripts/check-workflow-packages.sh` existe exactamente para o outro lado disto (um pacote
apagado citado por um workflow): **corrido, verde** — `32` nomes citados contra `379` membros.

### §2.2 FOUNDATIONAL / partilhado tocado — todo ADITIVO

| ficheiro | o quê | aditivo? |
|---|---|---|
| `ph2d-core/src/time.rs` | **`FixedStep::wall_seconds()`** — o relógio de PAREDE do laço (tiques × passo). ⚠️ Ele existe porque o `snap.now` do painel é o **PLAYHEAD**, e um eco de gesto preso ao relógio da cena fica aceso para sempre no primeiro pause | ✅ método novo |
| `ph2d-nodegraph/src/attr.rs` | `par_build_if` · `par_build_com_bloco` · `par_preenche_em_blocos` — a costura de rayon continua **UMA**, e as três delegam nela | ✅ append-only |
| `ph2d-node-registry/src/port_landing.rs` | **`landing_port`** (ficheiro novo, cortado por tecto de LOC) — a porta principal se servir, senão a primeira que sirva. ⛔ Nunca imposta: *uma preferência que recusa em vez de ceder transforma um acerto num bloqueio* | ✅ ficheiro novo |
| `ph2d-node-registry/src/fontes_de_posicoes.rs` | o censo derivado de quem produz só posições | ✅ ficheiro novo |
| `ph2d-editor-core/src/screens/layout.rs` | a banda do split que o painel lê | ⚠️ 14 linhas, ver o diff |
| `ph2d-i18n/src/{app_motion,motion_panels}.rs` | **2 chaves novas** (`app.motion.motion_bridge_heal.*`, `app.motion.motion_bridge_rewire.*`) | ✅ append |
| `shells/desktop/src/render_loop/` | 10 ficheiros — a fase nova `fase_motion_gizmos` (o retrato do gizmo sai do cozido DESTE quadro) + o perfilador partido em `MOTION`/`SIMULAÇÃO` | ⚠️ fase nova no quadro |

⚠️ **A fase nova chama-se `fase_motion_gizmos` e o prefixo `fase_` é LOAD-BEARING:** o texto emendado
do quadro colhe só essas, e com outro nome ela desaparece do oráculo de toda lei de ordem da shell,
**em silêncio**.

### §2.3 Símbolos que podem colidir com outra linha

- **`ph2d-node-rig-bones`** — crate nova, id de nó próprio. ⚠️ Se outra linha criou um nó no mesmo
  dia, **conte o id**, não o copie (`CLAUDE.md §5.0`).
- **`ph2d-gizmo-params`** — crate-folha nova (o vocabulário do gizmo no cartão).
- As **5 cenas de smoke novas**: `=121` · `=122` · `=123` · `=124` · `=125`. ⚠️ O gate
  `no_two_smoke_scenes_claim_the_same_level` mede o **piso**, não o teto ⇒ **duas linhas que
  escrevam o mesmo número passam MUDAS**. Confira o `match` do
  [`motion_state_demo_router.rs`](../../../crates/ph2d-app-motion/src/motion_state_demo_router.rs).

---

## §3 — ⛔ O QUE SÓ A ÁRVORE COMBINADA PODE REPROVAR — **corrido, verde**

```
bash scripts/censos-da-arvore-combinada.sh     # depois do `git rebase main`
→ 87 tests run: 87 passed (4 slow), 9163 skipped
  controlo do filtro: 8 de 8 censos correram ✓
```

⭐ Isto é o item **5-bis** da DIRETRIZ §1.5.9, e é o que converte *N descobertas em SÉRIE do
integrador* em *N descobertas em PARALELO das linhas*: o **censo de texto (HR-15)** e o **tecto de
LOC** são propriedades da SOMA, nenhuma linha as vê sozinha, e **o CI não os corre**. Na rodada de
17/09 foram **7 das 8** falhas do integrador.

⛔ **Isto NÃO substitui o portão da árvore combinada** (§1.5.3): o `--ff-only` continua a ser a
única prova de que ninguém aterrou entre este rebase e o merge.

---

## §4 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O `AVANCO_POR_CHAR` da cápsula (`1,06`) NÃO é a mesma estimativa que a fonte usava e que foi
   REMOVIDA.** Ali ela custava **20 % de corpo** (um corte de tamanho); aqui ela custa
   **enchimento** (uma pastilha com mais respiro). *A mesma estimativa é errada num sítio e certa no
   outro, e o que decide é o que o erro paga.* ⇒ ela é calibrada no **pior GLIFO** (`W`, `1,0062`),
   não no pior nome do catálogo, porque *o artista RENOMEIA um nó para o que quiser*.
2. **A `capsula_larguras` não é um cache «por causa da performance».** Ela existe porque a geometria
   (hit-rect e pino) não tem medidor de texto, e a largura passou a depender do NOME. ⚠️ Ela não é
   uma aposta sobre a ordem do quadro: o `interact::process` corre **DENTRO** do `paint`, depois da
   medição ⇒ o dedo e o desenho do mesmo quadro lêem o mesmo número.
3. **O piso do fio é um ZOOM (`1,0`) e não uma largura.** Um tecto em píxeis achatava o fio fino e o
   pesado no mesmo traço, e a largura de um fio **diz quantos elementos passam nele**. Congelar o
   ZOOM congela a família inteira.
4. **O `raio_do_canto` devolver `CARD_RADIUS` não é «reutilizar um número parecido»** — é a MESMA
   constante que o `paint_card` dá ao cabeçalho, de propósito: as duas formas não podem divergir no
   dia em que alguém mexer no raio do cartão.
5. **`nome_cabe_na_capsula` mudou de PERGUNTA** (e o doc dela di-lo): ela media *«cabe na largura do
   cartão»* e hoje mede *«a estimativa é generosa?»*. O censo dos 136 tipos do outro lado continua a
   correr — sobre outra coisa.
6. **O painel de params saiu do app, e o `MotionParamsPanel` NÃO foi «desligado»** — a crate foi
   apagada. O vocabulário de rows que ela guardava (`motion_param_rows.rs`, 522 linhas) mudou-se
   para o `ph2d-app-motion` com **zero imports**, que é a prova de que a fronteira estava certa.
7. **O `scrollbar_ids` do painel de params continua declarado.** Ele é ÓRFÃO e diz isso de si
   mesmo; apagá-lo devolveria o id ao pool, e *um id reciclado é a colisão silenciosa que aquele
   ficheiro existe para impedir*.

---

## §5 — ⛔ As premissas MINHAS que a medição derrubou (nesta jornada)

1. *«A largura de um texto é linear no corpo da fonte.»* — **FALSO**, e o erro é no sentido que
   CORTA: o pior nome mede `7,3608` por unidade a corpo `100` e **`7,6167`** a corpo `21,8`. Uma
   extrapolação pedia `22,33` e o nome sairia a `169,8` de `166`. ⇒ o extremo acha-se por **BUSCA**
   no corpo real.
2. *«O pior caso do catálogo é a população.»* — **FALSO**: o artista renomeia. Dezasseis `W` medem
   `450` onde o pior nome real mede `208`.
3. *«Uma estimativa por caractere serve para a largura.»* — **FALSO** com número: entregaria
   *«Simulation Zone»* numa pastilha de `321` unidades onde o texto mede `232` (`38 %` de
   enchimento).
4. *«O gate mede a lei.»* — o `o_nome_tem_um_tamanho_so_e_nunca_e_cortado` media *«cabe»*, e
   `CAPSULA_FONTE = 1,0` também cabe. **Uma régua que só vê o lado que não estoura aprova o valor de
   ontem**, e foi assim que `20 %` de corpo sobreviveram uma jornada inteira.
5. *«O arnês de mutação reporta o que aconteceu.»* — **FALSO três vezes numa wave**: cinco mutações
   leram `SOBREVIVEU` sobre produto certo (o python morreu, o ficheiro ficou intacto), uma lei
   guardada por `const assert!` contou como *«zero testes»*, e a agulha procurava a prosa ANTIGA do
   rustc. ⇒ o arnês ganhou os três controlos.

---

## §6 — ⏳ O QUE FICA ABERTO (e de quem é)

| item | de quem |
|---|---|
| ⛔⛔ **A rota da PLACA não corre o passe de contacto** — `ph2d-gpu-cook` não tem uma referência a `ph2d-contact`. Hoje invisível (toda cena com forma cai na CPU), *mas o caminho rápido e a colisão são mutuamente exclusivos*. É o **tecto real** do alvo *«milhares de objectos»* | espec própria |
| ⛔ **O tecto de LOD é uma CONTAGEM** (`LOD_COUNT = 16 000` cópias) e a grandeza é a **ÁREA**: mil discos de `200` unidades pintam `~31 M px/quadro` e passam por baixo dela | render/Vector |
| ⏳ **Dois nomes MUITO compridos lado a lado tocam-se** na disposição automática (a pior pastilha mede `232,5` e o `DX` é `220`). Medido, nomeado, **não curado** — a cura é um tecto de largura, e ele custa reticências | decisão do dono |
| ⏳ O custo do passe **não está no cartão** — foi o relógio que informou o dono, não o app (pago duas vezes) | próxima wave |
| ⏳ Os tectos de `motion.boids` e `motion.wave` seguem por medir (herdado) | próxima wave |
| ⏳ Os três defeitos do report de 19/09 estão **fechados**; o que sobra daquele handoff é a §4 dele | — |
| ⭐ **Promoção à lista de flakes do §5.0** (a linha pede, o integrador escreve): `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` (`ph2d-tool-painter`) — único ✗ de `17 086` a `load 18,86`, **zero linhas** do diff desta linha naquela crate, **3 de 3 verde sozinho a `load 21,8`–`25,9`** ⇒ *o discriminador é o FAN-OUT, não o relógio* | integrador |

---

## §7 — A PROVA DE FECHO (corrida nesta árvore, pós-rebase)

| portão | resultado |
|---|---|
| `bash scripts/nextest-impacted.sh` | **20 267 passed**, 0 failed |
| `bash scripts/censos-da-arvore-combinada.sh` | **87 passed**, controlo de filtro `8 de 8` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | limpo |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | limpo |
| `cargo fmt --all --check` | limpo |
| `cargo machete` | limpo |
| `bash scripts/check-standalone-optional.sh` | ✓ (10 crates com dependência interna opcional) |
| `bash scripts/check-workflow-packages.sh` | ✓ (32 nomes contra 379 membros) |
| provas de mutação (última wave) | **10 de 10 a sangrar** — [`mutacao_o_retangulo_a_fonte_e_o_fio_2026-09-20.sh`](../ferramentas/mutacao_o_retangulo_a_fonte_e_o_fio_2026-09-20.sh) |

### §7.1 Auditoria de fecho, TRÊS lentes mecânicas

| lente | resultado |
|---|---|
| gates NOMEADOS em comentário que não existem (a cicatriz de 13/09) | `143` nomes citados, **`0` sem lastro** |
| `#[ignore]` novos que sejam GATES (o CI nunca os corre) | `70` linhas, **todas** declaradas *sonda/medição/gerador*; os `6` «nus» são o texto `#[ignore]` **dentro de doc-comments** |
| dívida silenciosa (`TODO`/`FIXME`/`unimplemented!`) | `27` acertos, **todos falsos positivos** — é a palavra portuguesa *«TODO(S)»*. ⚠️ *Quem repetir esta lente neste repo tem de a escrever em inglês com fronteira de palavra, senão ela mede a língua* |

### §7.2 O binário do smoke — **compilado, com a prova**

```
$ rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental      # item 7, ANTES
    54G target/debug/incremental + 6,6G target/smoke/incremental reclamados  (target: 47G)

$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 49.14s

$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke   # a PROVA
    Finished `smoke` profile [optimized] target(s) in 0.50s
    linhas "Compiling": 0
```

`target/smoke/ph2d-host-desktop` · `84 569 496` bytes. ⚠️ **É a MESMA linha de comando do §8**, na
árvore desta worktree — *o dono não espera build*.

---

## §8 — OS SMOKES (o comando inteiro, copiável)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=<n> cargo run -p ph2d-host-desktop --profile smoke
```

| cena | o que ela mostra | estado |
|---|---|---|
| `=121` | o passe de contacto fora da simulação | ✅ aprovado |
| `=122` | o passe DENTRO de uma simulação | ✅ aprovado |
| `=123` | **A CADEIA** — o único sítio onde a escada de varreduras se VÊ | ✅ aprovado |
| `=124` | as POSIÇÕES e a marca | ✅ aprovado |
| `=125` | o OSSO, o pivô e o cartão de longe (**a cena das últimas seis waves**) | ✅ aprovado |

⚠️ **O `=125` é a cena do grafo:** afastar o zoom é o gesto que mostra o LOD, a cápsula-rectângulo e
os fios que param de encolher.

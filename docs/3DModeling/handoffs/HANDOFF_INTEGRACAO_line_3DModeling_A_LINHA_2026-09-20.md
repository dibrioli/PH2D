# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, a linha inteira (2026-09-20)

> **Este documento é do INTEGRADOR.** Ele não conta a história das waves — isso está nos docs
> [`docs/Render3d/05`..`13`](../../Render3d/README.md), um por assunto. Aqui está o que a fusão
> precisa: a **superfície de colisão medida**, os contadores como **DELTA** com a receita de
> recontar, os ficheiros partilhados onde um merge textual pode colidir, a **prova de fecho** e o
> que **só a árvore combinada** pode reprovar.
>
> ⛔ **O smoke foi APROVADO pelo dono** (a W10, 2026-09-20). Integrar e shipar continuam a ser
> ordem explícita dele (`CLAUDE.md` §0.7).

---

## §1 — O estado, em cinco linhas

| | |
|---|---|
| ramo | `line/3DModeling` |
| **já rebaseado sobre `main`** | ✅ `git rebase main` corrido em 2026-09-20 — ver a §6 |
| merge-base do rebase | `76bd6de02` (o `main` local desse momento) |
| commits | **88** · ⛔ **conte-os, não os leia daqui:** `git log --oneline main..HEAD \| wc -l` |
| ficheiros | **230** · `git diff --name-only main...HEAD \| wc -l` |
| ⚠️ o `main` pode ter andado | *se andou, o §3 diz exactamente o que reconferir — e é UMA coisa* |

---

## §2 — O que a linha entrega

Ela fecha **sete dos oito ingredientes** do [`01_o_alvo_decomposto`](../../Render3d/01_o_alvo_decomposto.md)
do modo Render. Um doc por assunto, e cada um tem a tabela medida dentro:

| wave | o que passou a existir | doc |
|---|---|---|
| `W5` | a luz indirecta por **SONDAS** (a recolha por pixel fica como referência convergida) | [`08`](../../Render3d/08_a_luz_indirecta.md) |
| `W5` | a **cor** que a peça devolve ao chão | [`09`](../../Render3d/09_a_cor_que_a_peca_devolve_ao_chao.md) |
| `W6`/`W7` | a **luz que atravessa a peça** — parede fina e maciça, com o oráculo de traçado convergido | [`10`](../../Render3d/10_a_luz_que_atravessa_a_peca.md) |
| `W7`/`W7b`/`W7c` | o **acabamento**: o brilho (*bloom*) e o que ele lê | [`12`](../../Render3d/12_o_acabamento.md) |
| `W8` | a **camada de estilo** — contorno, tinta por curvatura, grade por zona, saturação da indirecta | [`11`](../../Render3d/11_a_camada_de_estilo.md) |
| — | o **rebordo de um pixel** (duas causas distintas, as duas curadas) | [`13`](../../Render3d/13_o_rebordo_de_um_pixel.md) |
| **`W10`** | a **borda mole da sombra no MODO NORMAL** — o dispositivo assa o canal | [`10` §25](../../Render3d/10_a_luz_que_atravessa_a_peca.md) |

⭐ **Duas crates-folha novas:** [`ph2d-style`](../../../crates/ph2d-style/) e
[`ph2d-bloom`](../../../crates/ph2d-bloom/) — as duas com o molde da `ph2d-view-transform` (lei pura,
gémeo em WGSL, omissão byte-idêntica).

---

## §3 — ⚠️ O ÚNICO contador partilhado que se move: `PROJECT_SCHEMA`

```
main (76bd6de02):  PROJECT_SCHEMA = 144      tripla = (144, 13, 22)
a linha:           PROJECT_SCHEMA = 145      tripla = (145, 13, 22)
                                     DELTA = +1
```

⛔⛔ **Conte o DELTA contra a árvore em que vai aterrar, nunca o literal.** Se o `main` subiu para
`N` entretanto, o degrau desta linha passa a ser `N → N+1` e há **TRÊS** sítios a reconciliar:

1. a **constante** — [`shells/desktop/src/project_schema.rs`](../../../shells/desktop/src/project_schema.rs) (`pub(crate) const PROJECT_SCHEMA: u32 = …`);
2. a **escada** (o degrau de migração), no mesmo ficheiro;
3. a **tripla** do gate — [`shells/desktop/src/project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs), onde ela **CONTÉM** o número.

⭐ **O degrau reconta-se por SCRIPT e não à mão:** `python3 scripts/schema-recount.py <valor-alvo>`
— ⚠️ **ele só corre no meio de um MERGE** (lê `:1:`/`:3:` do índice). Numa árvore já rebaseada ele
estoura com `AttributeError: 'NoneType'`, e isso **não é um defeito dele**.

**O que o degrau `v145` migra:** a subsuperfície do OpenPBR (o `Surface` ganhou os campos que a
[`10`](../../Render3d/10_a_luz_que_atravessa_a_peca.md) descreve). Ele tem migração escrita.

### Os que NÃO se movem — confirmado pelo `collision-surface.sh`

| contador | main | linha |
|---|---:|---:|
| registo `ph2d-ecs` | 91 | **91** |
| espelho `ph2d-render` | 92 | **92** |
| espelho `ph2d-script` | 92 | **92** |
| `VEC_SCENE_SCHEMA` | 22 | **22** |
| `FLIP_SCHEMA` | 13 | **13** |
| `DOC_VERSION` (timeline) | 18 | **18** |
| `FIELD_DOC_VERSION` | 23 | **23** |

⭐ **Contrato congelado (§6): INTOCADO** — `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs`.
⭐ **ADR: nenhum criado** ⇒ esta linha está fora de toda disputa de número (último no disco: `0170`).
⭐ **Cargo.lock: ZERO pacotes EXTERNOS novos.** Os dois `+name` são as crates-folha desta linha
(`ph2d-bloom`, `ph2d-style`) — aresta interna, não dependência de terceiros.

---

## §4 — Os ficheiros PARTILHADOS onde um merge textual pode colidir

**26 ficheiros fora do módulo.** Os restantes vivem em `crates/ph2d-{app-field3d,field-render,field-gpu,panel-model3d,field-ecs,field,field-eval,bloom,style,material}/`, `docs/{Render3d,3DModeling}/` e `project-memory/`.

### 4.1 — ⛔⛔ Infra de AGENTE: a parede clean-room deixou de ser uma promessa

| ficheiro | o que mudou |
|---|---|
| `.claude/settings.json` | ganhou um bloco `permissions.deny` — `Read` negado a `UnrealEngine/Engine/{Source,Shaders}/**` e a `/usr/share/blender/*/scripts/**` |
| `.claude/hooks/tecto-de-recursos.sh` | regra `R3` — o mesmo, para o `Bash` |
| `.claude/hooks/tecto-de-recursos.prova.sh` | o controlo positivo da regra |

⚠️⚠️ **Isto muda o comportamento de TODOS os agentes desta máquina, não só desta linha.** A razão
está no cabeçalho do hook: o `docs/_ComoInvestigarApps/00_o_metodo.md` §0 **afirmava** que os
caminhos estavam negados «porque o `.claude/settings.local.json` nega» — e **medido em 2026-09-18,
não existia lista `deny` nenhuma, em ficheiro nenhum**. *Um doc que declara a lei que o código não
implementa lê-se como auditado.* ⇒ se houver conflito aqui, **os dois lados querem-se somados**: é
uma lista de negação, e perder uma entrada reabre a parede.

`docs/_ComoInvestigarApps/00_o_metodo.md` e `docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md`
levam a correcção correspondente.

### 4.2 — `ph2d-editor-core` (foundational) — 7 ficheiros, todos ADITIVOS

```
src/widget/mod.rs                        src/widget/property_box/mod.rs
src/widget/number_input/mod.rs           src/widget/property_box/paint.rs
src/widget/slider_with_chip/mod.rs       src/widget/slider_with_chip/number_chip.rs
tests/it/architecture_no_restricted_source_citations.rs
```

⭐ A extensão que a `W7` precisou é **aditiva e byte-idêntica no ponto neutro**:
`link_slider_number_curved` (a pista cúbica) ao lado do `linked_slider_mapping`, que fica
**intocado**. ⚠️ O `architecture_no_restricted_source_citations.rs` é a **catraca de citações**: esta
linha acrescentou-lhe as **seis** citações novas ao MaterialX como **ATRIBUIÇÃO** (ele é Apache-2.0,
uma das portas abertas do §0.9), nunca como dívida.

### 4.3 — `ph2d-i18n` — 5 ficheiros

`lib.rs` · `model3d.rs` · **`model3d_bloom.rs` (NOVO)** · `model3d_inert.rs` · `model3d_render.rs`

⚠️ O `lib.rs` **declara e encadeia** as quatro tabelas no `tr`. Um merge que perca a linha
`mod model3d_bloom;` ou o `.or_else(|| model3d_bloom::tr(k))` deixa **19 chaves** a pintar o
identificador cru — e **há gate a apanhá-lo** (ver §5).

### 4.4 — `ph2d-render` — 5 ficheiros · `shells/desktop` — 2 · `CLAUDE.md` — 1

O `CLAUDE.md` leva **as linhas do §5 das waves desta linha**. ⚠️ Ele colide com **toda** linha da
rodada; a resolução é sempre **somar**, nunca escolher.

---

## §5 — ⛔⛔ O que SÓ a árvore combinada pode reprovar — e o que já foi corrido AQUI

⭐⭐ **O item 5-bis da DIRETRIZ §1.5.9 já está feito nesta linha, e ele apanhou UM VERMELHO.**

```
bash scripts/ph2d-run.sh bash scripts/censos-da-arvore-combinada.sh
  → Summary [123.770s] 94 tests run: 94 passed, 9246 skipped
  → controlo do filtro: 8 de 8 censos correram ✓
```

**O vermelho que ele apanhou, e a cura (commit `0cdfd0eea`):** o censo de i18n do
`ph2d-panel-model3d` nomeava **TRÊS** tabelas à mão; a `W7` criou a **quarta**
(`model3d_bloom.rs`), o `lib.rs` declarou-a — e o censo continuou a ler três: `174` declaradas
contra `182` usadas, com **19** chaves do brilho acusadas de não ter tradução **tendo-a**.
⇒ a lista passou a ser **DERIVADA do `lib.rs` que as declara**, com piso de população (`≥ 4`) e
prova de que cada declarado existe no disco. Depois: `194` declaradas · `175` usadas, zero sem
tradução, zero órfãs. **Mutação 2 de 2**, controlo verde.

⚠️⚠️ **Porque só na hora do fecho:** aquele gate vive em `crates/ph2d-panel-model3d/tests/it/` e os
portões das waves correram `-p ph2d-app-field3d -p ph2d-field-gpu -p ph2d-material -p ph2d-field-render`.
É a cegueira que o `CLAUDE.md` §5 já nomeia — *um gate que mede uma crate que a linha editou e vive
noutra que ela não correu*. **Se a rodada tiver outras linhas, espere a mesma forma nelas.**

### O que o integrador ainda tem de correr na SOMA

1. **`bash scripts/censos-da-arvore-combinada.sh`** outra vez, **depois** de fundir cada linha — o
   censo de texto e o tecto de LOC são propriedades da **SOMA**, e nenhuma linha as vê sozinha.
2. **O tecto de LOC por ACUMULAÇÃO.** Nesta linha **nenhum ficheiro passa do tecto**
   (`collision-surface.sh` confirma), mas ⚠️ **dois estão colados a ele** e uma linha vizinha que
   lhes toque estoura-os:

   | ficheiro | LOC | tecto | margem |
   |---|---:|---:|---:|
   | `crates/ph2d-field-gpu/src/trace.rs` | **699** | 700 | **1** |
   | `crates/ph2d-field-gpu/src/paint_wgsl_sondas.rs` | 693 | 700 | 7 |

   ⛔ A cura é **CORTE por responsabilidade**, nunca uma entrada no `FILE_OVERAGE_OK` — que está
   **vazio**. Para o `trace.rs` o corte que cabe está nomeado na
   [`10` §25.7](../../Render3d/10_a_luz_que_atravessa_a_peca.md): o bloco de canalização dos *bind
   groups* (`Saida` · `Pintado` · `uniforme` · `armazem` · `bgl_marcha`, ~68 linhas, 24 consumidores
   na crate).
3. **`the_shell_only_shrinks`** — esta linha acrescenta **2 ficheiros** à `shells/desktop`
   (`project_schema*.rs`, o degrau). Se a soma da rodada fizer a shell crescer, a cura é do
   integrador e é corte, nunca subir o número.

---

## §6 — A PROVA DE FECHO (o que foi corrido, e com que resultado)

Tudo abaixo **na árvore já rebaseada**.

| portão | resultado |
|---|---|
| `git rebase main` (86 commits) | ✅ **dois conflitos, ambos em ficheiros de MEMÓRIA append-only** (`project-memory/{MEMORY.md,reference_topic_gate_discipline.md}`), resolvidos ficando com **os dois lados**. **Zero conflitos em código.** |
| `censos-da-arvore-combinada.sh` | ✅ 94/94, controlo do filtro 8/8 |
| suítes das crates tocadas | ✅ `ph2d-material` 15 · `ph2d-field-gpu` 3 · `ph2d-field-render` 112+ · `ph2d-app-field3d` 461 · `ph2d-panel-model3d` 46 |
| `cargo clippy --all-targets -- -D warnings` | ✅ verde, **pós-rebase**, nas SETE crates que a linha toca (`ph2d-material` · `-field-render` · `-field-gpu` · `-app-field3d` · `-panel-model3d` · `-style` · `-bloom`) |
| `cargo fmt --check --all` | ✅ verde, **pós-rebase** |
| gates de GPU da `W10` | ✅ `as_duas_colunas_da_banda_leem_o_mesmo` (`1,00` / `1,00`) · `a_borda_mole_da_sombra_e_a_mesma_nos_dois_motores` (`100,000 %`, pior byte `1`) · `o_gemeo_da_borda_mole_esta_ligado_no_dispositivo` · `o_material_do_dispositivo_e_o_da_cpu` |
| prova de mutação da `W10` | ✅ **6 de 6** sangram, com controlo verde |
| `nextest-impacted.sh` (BASE=main) | ✅ **15 118 testes, 15 118 passados, zero falhas** (108,6 s; 12 061 saltados) |
| bateria `--ignored` de GPU do `ph2d-app-field3d` | ver §6.2 |

### 6.1 — ⚠️ A flake de carga que esta linha encontrou (e NÃO é dela)

`ph2d-field-render::tests::an_abandoned_march_returns_nothing_and_returns_fast` reprovou **uma vez**
no meio da suíte em lote e passa **3 de 3 sozinho a `load 25,75`** — *o triplo da carga em que
reprovou*. Ele é **membro confirmado** da família de flakes de fan-out que o `CLAUDE.md` §5.0 já
nomeia, e esta linha tem **zero linhas de diff** naquele ficheiro. ⛔ Não o investigue: re-corra-o
sozinho, com o `/proc/loadavg` impresso ao lado.

### 6.2 — ⏳ O que ainda não tem veredito quando este doc foi escrito

- **A bateria `--ignored` de GPU do `ph2d-app-field3d`** (130 gates, um de cada vez): a primeira
  tentativa deu `55` verdes / `0` vermelhos e foi **morta pelo prazo**, e ⚠️⚠️ **o comando ainda
  saiu com código `0`** — a secção do pacote simplesmente não tem linha `test result:`. *Silêncio a
  ler-se como sucesso.* ⇒ **quem integrar corre-a na árvore fundida**, com `PH2D_PRAZO ≥ 3600` e a
  saída CRUA para ficheiro (sem `grep | head`, que bufferiza e esconde o progresso).

---

## §7 — ⛔ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O `return` do ramo pintado no `paint_com` continua lá, e está CERTO.** A `W10` não o removeu:
   a cura foi o **dispositivo passar a assar o canal**, não voltar pelo caminho de CPU (que traria o
   G-buffer pelo barramento). O gate estrutural que afirmava a dívida foi **substituído**, não
   apagado — e a morte da premissa está à vista no diff.
2. **`mole = None` NÃO é «a borda mole desligada».** É *«esta cena não tem material translúcido»*, e
   nesse caso o quadro é byte a byte o de sempre. O passo do buffer nem cresce.
3. **O dispositivo RECUSAR o quadro com dois raios distintos é DESENHO, não uma falha.** O buffer
   tem um raio para a cena inteira; com dois, o `paint_com` devolve `None` e a CPU pinta com a lei
   inteira. *Nenhuma imagem errada em lado nenhum.*
4. **O corpo do pintor são TRÊS fragmentos de WGSL e não dois.** O terceiro
   (`paint_wgsl_mole.rs`) nasceu de um **tecto de LOC**, não de um assunto novo — e há gate a exigir
   que o `format!` os **junte**: *um fragmento declarado que ninguém concatena compila, passa em todo
   gate de texto, e não chega ao shader*.
5. **As `19` chaves de i18n do brilho nunca estiveram «sem tradução».** Elas estavam declaradas o
   tempo todo; o **censo** é que lia três tabelas de quatro (§5).
6. **O bloco `deny` do `.claude/settings.json` não é configuração pessoal.** É a parede clean-room a
   passar de doc para lei executável, e perder uma entrada num merge reabre-a.
7. **`ph2d-style` e `ph2d-bloom` aparecem como `+name` no `Cargo.lock` e NÃO são dependências
   externas.** São crates desta árvore.

---

## §8 — ⏳ O que fica ABERTO, e de quem é

| item | de quem |
|---|---|
| **o relógio das waves `W7`/`W8`/`W10` NÃO foi medido** — elas entram na tabela da `W9` | da `W9` (o dono pô-la ao fim da fila) |
| `W6` — a autoria MaterialX (nós no editor) | fila do módulo |
| `W7d` — profundidade de campo | **proposto FORA pelo dono** |
| o `G-20` do pincel de plano (digitável `≥ 5 000`) | decisão do dono |
| `trace.rs` a **uma** linha do tecto (§5.2) | quem lhe tocar a seguir |
| `project-memory/MEMORY.md` em **36 KB / 236 linhas** contra o tecto declarado de **17 KB / 140** | dívida partilhada, pré-existente; esta linha acrescentou-lhe e **não** a criou |

---

## §9 — As premissas que a MEDIÇÃO derrubou (as que mudam o que o integrador lê)

1. *«a paridade entre os dois motores prova que o canal chega ao pixel»* — **falso**. Dois motores
   que ignoram a mesma feature concordam a `100 %`. A fracção lia `99,579 %` contra a barra de
   `99,5` com o defeito vivo; o discriminador é o **pior byte** (`1` contra `12`) e a régua
   **desenhada para a feição** (a banda do terminador, `1,00` contra `9,20`).
2. *«o `TABLES` do censo descreve as tabelas»* — **falso desde a `W7`**. Uma lista à mão ao lado de
   uma realidade derivada são duas respostas à mesma pergunta.
3. *«o fonte dos alvos amuralhados está negado, porque o `settings.local.json` nega»* — **falso**:
   não existia lista `deny` nenhuma. A parede era uma promessa.
4. *«um vigia `until ! pgrep -f <padrão>` avisa quando o processo acaba»* — **falso**: o shell do
   próprio laço tem o padrão no `cmdline`, logo o `pgrep` **auto-apanha-se** e a condição nunca fica
   falsa. O vigia expirou sem eventos e o `pgrep -c` lia `1` sobre **zero** binários vivos.

# BRIEFING DO AGENTE INTEGRADOR — rodada de 2026-09-10 (SEIS linhas)

> **O que este documento é:** a **ordem** da rodada, **medida**, mais o que cada fusão vai partir e
> onde. Ele **não substitui** os seis handoffs de linha — ele diz **em que ordem os ler** e **um de
> cada vez**.
> **Autoridade:** [`DIRETRIZ §1.5.3`](DIRETRIZ.md) (mecanismo) · [`CLAUDE.md §0.7`](../../CLAUDE.md)
> (integrar e shipar só por ordem explícita do Enio).

---

## §0 — ⛔⛔ LEIA UM HANDOFF DE CADA VEZ. NÃO OS LEIA TODOS AGORA.

Os seis handoffs somam **117 927 bytes**. Lê-los de uma vez enche a janela **antes da primeira
fusão** — e a DIRETRIZ §1.5.3 tem o número: nas 6 maiores sessões de integração deste repo o
integrador arrastou **~8,4 M tokens** para uma janela de 1 M, ou seja **~8 compactações por
construção**, e re-descobria tudo depois de cada uma.

**O protocolo desta rodada é um LAÇO, e cada volta é fechada:**

```
para cada linha, NA ORDEM do §2:
  1. bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh   # DENTRO da worktree dela, AGORA
  2. ler O HANDOFF DAQUELA LINHA (só ele)
  3. bash scripts/foundational-integrate.sh                                  # dentro da worktree dela
  4. curar o que reprovar (o §3 abaixo prevê o quê e onde)
  5. escrever a UMA LINHA daquela linha no CLAUDE.md §5, no PRIMÁRIO
  6. ⛔ ESQUECER o handoff daquela linha e passar à seguinte
```

⚠️ **O passo 1 corre-se OUTRA VEZ a cada volta, inclusive depois de cada fusão anterior** — a tabela
colada em cada handoff mede a linha contra o `main` de **2026-09-10** e cada fusão move o `main` de
novo. *Uma tabela colada não sabe que envelheceu.* A divergência entre as duas leituras é ela
própria um achado, e nomeia quem entrou no meio.

⚠️ **Invoque o `collision-surface.sh` pelo caminho ABSOLUTO do primário** (acima) — uma worktree
forkada antes de o script existir não o tem.

---

## §1 — ⛔ GATE ZERO: o primário está SUJO, e isso bloqueia a rodada inteira

Medido em 2026-09-10, na raiz (`main`, HEAD `39d48cd76`):

```
 M CLAUDE.md
 M docs/_ComoInvestigarApps/{00_o_metodo,01_o_arsenal,README}.md
 M project-memory/{MEMORY.md,project_audio_multichannel_silence.md,
                   reference_canonical_files.md,reference_topic_implicit_field_laws.md,
                   reference_topic_ui_seam_discipline.md}
?? project-memory/  (5 ficheiros novos)
```

A DIRETRIZ §1.5.3 declara a precondição: **«primário limpo e em main (sujo = violação do 1.5.1 —
pare e reporte)»**. O `merge --ff-only` do último passo do script recusa uma árvore suja.

⚠️⚠️ **E não é só mecânica: três linhas editam o `CLAUDE.md` e três editam o `project-memory/MEMORY.md`**
(§4) — exactamente os ficheiros que estão por comitar. Fundir por cima disto mistura trabalho não
comitado de outra sessão com o de seis linhas, e nada nomeia o dono de cada hunk.

⚠️ **E há um terceiro sintoma da mesma sujidade:** `bash scripts/doc-index.sh --check` está
**vermelho agora**, no `main` intocado — `docs/_ComoInvestigarApps/README.md` desactualizado, e a
causa é uma das 9 modificações por comitar. *Um vermelho que já existe antes da primeira fusão vai
ser lido como culpa da linha que estiver a entrar quando alguém o vir.*

⛔ **`git stash` está proibido** (`CLAUDE.md §0.4`). **PARE e pergunte ao Enio** o que fazer com estas
14 entradas antes de tocar em qualquer worktree. A resposta provável é *comitá-las* (parecem escrita
de memória de uma sessão anterior) — mas a decisão é dele, não sua.

---

## §2 — A ORDEM, e o porquê de cada posição

**As seis fazem fork do MESMO `merge-base 39d48cd76`** — nenhuma está atrasada, nenhuma precisa de
rebase prévio para além do que o script faz.

| # | linha | commits | ficheiros de código | por que **aqui** |
|---:|---|---:|---:|---|
| **1** | `line/UIUX` | 44 | 127 | **Zero contadores movidos** (schema, registo, ADR, `Cargo.lock`, LOC: tudo `0`). O risco dela não é numérico — é a **FORMA**: `Panel::ICON` obrigatório, `IconId` com ordem load-bearing, a cadeia do `ph2d-i18n`, e **catracas a zero de folga**. Pô-la primeiro converte *«cinco linhas partem todas juntas na última fusão»* em *«cada linha é julgada pelos gates dela, no gate DELA, com a árvore ainda pequena»*. |
| **2** | `line/Vector` | 36 | 122 | **A única linha com painel novo** (`ph2d-panel-skeleton`) ⇒ é a **única vítima** do `Panel::ICON`, e paga-a na fusão dela. E tem o **maior delta de `PROJECT_SCHEMA` (+4)** ⇒ escreve a escada dela primeiro e **nunca renumera**. |
| **3** | `line/components` | 34 | 225 | **Par mais entrelaçado da rodada com a Vector (11 ficheiros partilhados)** ⇒ adjacente, para o segundo rebasear sobre a deriva **mínima**. E renumera **UM** degrau (`124 → 128`) em vez de quatro. |
| **4** | `line/sculpt3d` | 64 | 216 | **Order-agnostic por medição** (o handoff dela declara-o e a sonda confirma: nenhum contador partilhado se move). Fica antes das duas mais leves porque é a que ainda tem mais churn de shell — *o leve rebaseia sobre o pesado, nunca o contrário*. |
| **5** | `line/motion-value` | 57 | 130 | Tem **uma assinatura QUEBRADA** (`ph2d-gpu-cook::kernel_module`, 8 → 6 params) e **zero vítimas medidas**: nenhuma das outras cinco toca `ph2d-gpu-cook` / `ph2d-nodegraph` / `ph2d-node-registry`. O `.typos.toml` que ela edita não é tocado por mais ninguém. |
| **6** | `line/3DModeling` | 24 | 91 | **A fusão mais barata da rodada, medida:** partilha **1 ficheiro** com cada uma das outras — e a edição dela nele é **uma linha de comentário** (`main.rs`, `PH2D_FIELD_SMOKE=1..29` → `1..32`). O `Cargo.toml` da raiz que ela apêndica **não é tocado por mais ninguém** (`0` em seis). |

### A matriz que decidiu (ficheiros de código em comum, medida hoje)

```
              motion-value    sculpt3d  3DModeling  components        UIUX      Vector
motion-value             -           3           2           3           2           4
sculpt3d                 3           -           1           4           1           4
3DModeling               2           1           -           1           1           1
components               3           4           1           -           6          11
UIUX                     2           1           1           6           -           4
Vector                   4           4           1          11           4           -
```

### ⚠️ As duas decisões de ordem que têm um lado ERRADO plausível

**(a) Porque não `Vector` antes de `UIUX`.** Nessa ordem o `Panel::ICON` em falta e as catracas do
ritmo caem **na fusão da UIUX**, sobre uma crate cujo dono é outro (`ph2d-panel-skeleton`) — duas
classes de vermelho, ambas sobre código da Vector, descobertas enquanto se funde a UIUX. A ordem
escolhida põe as duas dentro do gate da própria Vector, com o `main` já a carregar os 26 glifos
declarados (o que torna `no_two_panels_share_a_glyph` verificável de imediato).

**(b) Porque `Vector` antes de `components`, e não ao contrário.** Há um argumento genuíno para o
contrário — a `components` **move funções** para ficheiros irmãos em `ph2d-panel-vector` e em
`ph2d-panel-inspector`, e *quem move devia entrar depois de quem edita no sítio*. Ele **perde** por
medição: essa sobreposição é de **um dígito** por ficheiro (`ids.rs` −9/−8 · `state.rs` −8/−7 ·
`event_clicks.rs` −9/−1) e há gates de registo a apanhá-la, enquanto a renumeração da escada do
`PROJECT_SCHEMA` seria de **quatro degraus em vez de um**, num ficheiro (`project_schema.rs`) onde
os dois handoffs avisam que *«um degrau escrito no ficheiro errado funde LIMPO e evapora»*.
⇒ **prefere-se o risco pequeno-e-alto ao grande-e-mudo.**

---

## §3 — O que vai reprovar, POR FUSÃO, com endereço

> ⛔ Estas são previsões **medidas hoje**, não palpites. Se alguma **não** acontecer, isso também é
> um achado — quer dizer que a árvore combinada não é a que se mediu.

### Fusão 1 — `line/UIUX`
Nada previsto. A sonda dá **zero em todas as colunas que costumam colidir**; ela funde contra o
`main` intocado. ⚠️ O `typos` **não foi corrido** nesta worktree e ela escreve muito português
acentuado em doc-comments — foi o `typos` que apanhou dois vermelhos na integração de 07/09.

### Fusão 2 — `line/Vector` (a fusão cara da rodada)

| o quê | endereço | cura |
|---|---|---|
| ⛔ **`Panel::ICON` em falta** — erro do compilador `not all trait items implemented` | `crates/ph2d-panel-skeleton/src/lib.rs:58` (`impl Panel for SkeletonPanel`, declara `TITLE` na linha 66, **não** declara `ICON`) | **uma linha**: `const ICON: ph2d_editor_core::icons::IconId = …;` com um glifo que **mais ninguém usa** (gate `no_two_panels_share_a_glyph`) + o SVG em `docs/design/icons/` (gate `every_declared_glyph_draws_something`). ⛔ **Não invente um default no trait** — a ausência é deliberada e está documentada no próprio const |
| ⛔ **`every_stack_of_rows_asks_the_rhythm`** (o `MUTE_OK` está **vazio**, `&[]`) | `crates/ph2d-panel-skeleton/src/paint.rs` — menciona `ROW_H_PX` e chama **ZERO** das 8 portas do ritmo (`area_gap_px` · `control_gap_px` · `icon_label_gap_px` · `list_indent_px` · `list_row_gap_px` · `row_pitch_px` · `section_gap_px` · `tree_chevron_col_px`). Medido: `grep -cE` das 8 devolve **`0`** | **chamar a porta**. ⛔ **NUNCA acrescentar uma entrada ao `MUTE_OK`** — *uma catraca sem censo de obsolescência não desce: vira licença* (`CLAUDE.md §5.0`) |
| `PROJECT_SCHEMA` `123 → 127` (**+4**) | os **TRÊS** sítios: a const em `shells/desktop/src/project_schema.rs` · a **escada** no mesmo ficheiro (+79 linhas) · a **tripla** em `shells/desktop/src/project_schema_tests.rs` (+1/−1) | entra sem disputa (é a primeira das duas a escrever) |
| `Cargo.lock`: 2 pacotes novos (`ph2d-panel-skeleton`, `ph2d-poly2d`) | raiz | ⛔ nunca à mão: `git checkout main -- Cargo.lock` + `cargo check -p` regenera + `git add` |
| textual em `crates/ph2d-editor-core/src/screens/hero/{paint.rs,topbar/mod.rs}` | o item *Window → Bones* contra o `slot_tabs` partido em 4 ficheiros pela UIUX | **união**, sempre. E ⚠️ `tab_rects` **deixou de existir** e `tab_layout`/`paint_slot_tabs` **mudaram de assinatura** |
| ⭐ `crates/ph2d-panel-registry-init/` | UIUX toca só `tests/`, Vector só `Cargo.toml`+`src/lib.rs` | **disjunto — não há conflito aqui** |

### Fusão 3 — `line/components`

| o quê | endereço | cura |
|---|---|---|
| ⛔⛔ **`PROJECT_SCHEMA` tem de ser RE-CONTADO** | base `123` · Vector `+4` · components `+1` ⇒ **o valor certo é `128`, e não está em nenhum dos dois lados** | mover o degrau único da F8 de `124` para **`128`**, nos **três** sítios, e a tripla para **`(128, 13, 22)`** |
| corte de `ph2d-panel-inspector/src/paint*.rs` contra as edições de ritmo que a UIUX já lá pôs | funções movidas para irmãos | ⚠️ **grepe pelo NOME da função, nunca pelo ficheiro.** Se uma chamada de ritmo evaporar, o gate `every_stack_of_rows_asks_the_rhythm` reprova e **nomeia o ficheiro** |
| corte de `ph2d-ui-testkit/src/lib.rs` (235 linhas saem para `sowing.rs`) | idem | idem |
| registo de componentes `80 → 85` (**+5**) nos dois espelhos + `ph2d-ecs` `79 → 84` | `ph2d-render` · `ph2d-script` · `ph2d-ecs/scene/registry.rs` | ⚠️ **ninguém mais lhes toca nesta rodada** ⇒ o `+5` entra tal e qual. ⛔ Mas **conte o delta**: as notas dentro dos ficheiros falam de `77`/`78`, que é o estado **intermédio** desta linha |
| `LIVE_SECTIONS` `[;16] → [;20]` · `ComponentCategory::ALL` `13 → 16` · `any_live_section` `[bool;11] → [bool;15]` | `ph2d-editor-core/src/ids/live_sections.rs` e irmãos | claimant único, entram tal e qual |
| os 11 ficheiros partilhados com a Vector | `app_state.rs` · `input_dispatch.rs` · `render_loop/mod.rs` · `preview_drive.rs` · `main.rs` · `panel-vector/{ids,state,event_clicks}.rs` | textual; a Vector já lá está |

### Fusão 4 — `line/sculpt3d`
| o quê | endereço | cura |
|---|---|---|
| textual em `crates/ph2d-i18n/src/sculpt3d.rs` | UIUX **+5** (a chave `panel.sculpt3d.empty`, no meio) · sculpt3d **+36** (apendadas) | **união dos dois lados** — uma tabela de tradução é append-only por construção |
| comentários em 6 crates de outras linhas (`ph2d-painter-brush`, `ph2d-flip*`, `ph2d-tool-flip`, `ph2d-panel-painter-layers`) | a cura de citações da clean-room | ⚠️ **zero linhas fora de comentário**, medido. *Prefira o lado desta linha nos comentários e o lado do `main` no código* — se a cura de citação se perder, a **catraca de citações de fonte restrita que esta linha traz** volta a acusar (o nome exacto do gate está no handoff dela, §2). ⛔ **O nome NÃO se escreve aqui de propósito, e repô-lo reprova a rodada:** ele só existe no `main` **depois** desta fusão, e o `instructional_docs_only_reference_existing_gates` acusa qualquer doc de `docs/IntegracaoMultiAgente/` que nomeie um `architecture_*` que ainda não é teste — foi ele, e este briefing, que reprovaram a **fusão 1** desta rodada |

### Fusão 5 — `line/motion-value`
| o quê | endereço | cura |
|---|---|---|
| ⭐ `ExtraBuffers` / `kernel_module` 8 → 6 params (23 sítios, todos dela) | `ph2d-gpu-cook` | **zero vítimas medidas** — nenhuma das outras cinco toca essa crate. Se aparecer uma chamada nova, a cura é mecânica (agrupar `grid`/`reduces`/`luts` no struct). ⛔ Não é cosmética: eram 8 sobre um tecto de 7 |
| `.typos.toml` (raiz) | 2 entradas | claimant único |
| `MAX_DEMO_LEVEL` `110 → 114` (cenas `=111..=114`) | `shells/desktop/src/motion_state_demo_router.rs` | claimant único. ⚠️ O gate `no_two_smoke_scenes_claim_the_same_level` mede o **piso**, não o tecto |

### Fusão 6 — `line/3DModeling`
| o quê | endereço | cura |
|---|---|---|
| `main.rs` | **uma linha de comentário** | trivial |
| `Cargo.toml` da raiz — 4 entradas `[profile.dev.package.*]` `opt-level = 2` | apêndice | **ninguém mais lhe toca** (`0` em seis, medido) |
| `FIELD_DOC_VERSION` `18 → 22` | `crates/ph2d-field/src/lib.rs` — ⚠️ **e o const MUDOU de linha** (227 → 287) | claimant único |
| ⚠️ o catálogo **mudou de ficheiro**: `field3d_shapes.rs` → `field3d_shapes_table.rs` | `shells/desktop/src/` | uma entrada acrescentada no ficheiro antigo funde **limpo e evapora** |

---

## §4 — Os contadores, todos, numa tabela — e o que a sonda NÃO vê

⭐⭐ **Medido hoje nas seis worktrees: só UM contador é disputado, e ele é disputado ALTO** (as duas
linhas editam a mesma linha da tripla, logo o git conflita). Todos os outros têm **claimant único**.

| contador | `main` | quem o move | delta | final |
|---|---:|---|---:|---:|
| **`PROJECT_SCHEMA`** | `123` | **`Vector` (+4) E `components` (+1)** | **+5** | ⚠️ **`128`** |
| tripla do gate | `(123, 13, 22)` | as duas, na MESMA linha | — | ⚠️ **`(128, 13, 22)`** |
| registo `ph2d-ecs` | `79` | `components` | +5 | `84` |
| espelhos `ph2d-render` / `ph2d-script` | `80` | `components` | +5 | `85` |
| `LIVE_SECTIONS` | `[;16]` | `components` | +4 | `[;20]` |
| `ComponentCategory::ALL` | `13` | `components` | +3 | `16` |
| **`FIELD_DOC_VERSION`** | `18` | `3DModeling` | +4 | `22` |
| **`MAX_DEMO_LEVEL`** | `110` | `motion-value` | +4 | `114` |
| `PrimitiveKind::ALL` | `58` | `3DModeling` | +4 | `62` |
| catálogo `panel.model3d.add` | `68` | `3DModeling` | +5 | `73` |
| `VEC_SCENE_SCHEMA_VERSION` | `22` | **ninguém** | 0 | `22` |
| `FLIP_SCHEMA` · `DOC_VERSION` (timeline) | `13` · `18` | **ninguém** | 0 | idem |
| ADR (último `0169`, próximo livre `0170`) | — | **nenhuma linha cria ADR** | 0 | — |
| contratos congelados (§6) | — | **nenhuma linha os encosta** | 0 | — |

⚠️⚠️ **Três destes o `collision-surface.sh` NÃO vê** — `FIELD_DOC_VERSION`, `MAX_DEMO_LEVEL` e
`PrimitiveKind::ALL` ficam fora da sonda (ela conta `PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`,
`FLIP_SCHEMA` e o `DOC_VERSION` da timeline). **Nesta rodada cada um tem claimant único e a
colisão é nula** — mas isso foi *medido*, não é propriedade da sonda: numa rodada seguinte duas
linhas que os subam fundem **mudas**.

### Tetos de LOC — o risco ACUMULADO está ilibado
`shells/desktop/src/render_loop/mod.rs` acumula ≈ `13 330 → 14 039` somando as quatro linhas que lhe
tocam. **Não trip**: ele e o `main.rs` carregam o marcador textual `// ph2d-loc-cap:` (isenção
nomeada, não um número congelado). Nenhuma das seis passa do tecto individualmente.

### Ficheiros de doc partilhados
| ficheiro | quem edita |
|---|---|
| `CLAUDE.md` | `motion-value` · `sculpt3d` · `Vector` (**3**) — e está **sujo no primário** (§1) |
| `project-memory/MEMORY.md` | `motion-value` · `3DModeling` · `UIUX` (**3**) — idem |
| `project-memory/reference_topic_gate_discipline.md` | `motion-value` · `UIUX` (**2**) |
⇒ **união, sempre.** Nenhum destes é decisão; são acréscimos a índices.

---

## §5 — O que fica para o FIM da rodada (não durante)

1. **A UMA LINHA de cada módulo no `CLAUDE.md §5`** — escreva-a **na volta daquela linha** (passo 5
   do laço), no primário, uma de cada vez (DIRETRIZ §1.5.5). ⛔ **Não acrescente parágrafo de
   jornada** — cada handoff já traz a linha pronta a colar (`components §11` · `3DModeling §12` ·
   `UIUX §10.1` · e os equivalentes nos outros três). A **UIUX §10.2 pede uma CORRECÇÃO**: a linha
   de *Smokes* do módulo UI/UX afirma uma falsidade.
2. **Promover CINCO flakes de carga ao `CLAUDE.md §5.0`** (precedente: o
   `the_cost_of_a_player_is_linear_in_their_number`, promovido em 07/09 «a pedido da `line/UIUX`»
   — *a linha pede, o integrador escreve*):
   - `the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke` (razão de dois relógios)
   - `interaction_dispatch_no_alloc` (contador de alocações)
   - `the_ui_clock_does_not_allocate_per_frame` (contador de alocações; ⛔ o doc-comment dele
     **declarava-se imune** — é o **quarto** deste repo a fazê-lo)
   - o vermelho do shell reportado pela `line/Vector`, que não reproduziu em 4 corridas
   - o da `line/motion-value` §7.1, já catalogado
   ⚠️ **Antes de arquivar qualquer vermelho como flake, re-corra-o sozinho COM o `/proc/loadavg`
   impresso ao lado** — em 02/09 uma leitura `3 de 3 VERMELHO` a `load 82` quase arquivou um teste
   são como defeito.
3. **`./scripts/ship.sh`** sobre o main integrado — ele é o único que corre `fmt` da árvore inteira,
   `machete`, `deny`, `audit` (RUSTSEC), `typos` e o `nextest --cargo-profile ci-test`. ⚠️ **Nenhuma
   das seis worktrees correu `machete`/`deny`/`audit`**, e só a `sculpt3d` e a `motion-value`
   correram `typos`.
4. ⛔⛔ **DEIXE O BINÁRIO DO SMOKE COMPILADO NO PRIMÁRIO** (DIRETRIZ §1.5.9 item 9). As seis
   worktrees têm o delas quente; o do **primário é de 07/09** e está obsoleto. O Enio smoka o
   **main**, na árvore do **primário**:
   ```
   cd /home/enio/Documentos/Projetos/PH2D && cargo build -p ph2d-host-desktop --release
   cd /home/enio/Documentos/Projetos/PH2D && cargo build -p ph2d-host-desktop --release   # 2ª = a PROVA
   ```
   A 2ª corrida tem de dizer *Finished* em segundos com **zero** linhas `Compiling`.
5. ⛔ **NÃO faça push nem ship sem ordem explícita do Enio** (`CLAUDE.md §0.7`).

---

## §6 — ⛔ O que NÃO é seu nesta rodada

| item | de quem é |
|---|---|
| **`.config/nextest.toml` — o `slow-timeout` global** proposto pela `line/Vector` §9 (`period = "60s"`, `terminate-after = 15`) | **decisão do Enio.** Ele atravessa as seis worktrees e o CI. ⚠️ O `slow-timeout` que existe está cercado por `filter = 'package(ph2d-asset-cooker)'` ⇒ **nem o nextest, nem o `ship.sh`, nem o CI têm guarda de pendura fora daquela crate**. As âncoras medidas: o teste legítimo mais lento do repo custa **375 s** e **PASSA** (`ph2d-quadchain::veto`); os órfãos que motivaram a proposta queimaram **1h55m**. ⛔ **Não o shipe por iniciativa própria** |
| `scripts/cargo-test-narrow.sh` — tecto de tempo + varredura de órfãos | **já vem na `line/Vector`** (fusão 2) e as outras cinco herdam-no a partir daí. Defensivo, defaults generosos, `exit 3` = pendurou |
| a wave que põe as superfícies das outras linhas no ritmo novo | do dono da UI, **não da integração** — excepto o único ficheiro que o gate acusa hoje (§3, fusão 2) |
| os `418` literais pintados nas outras 18 crates (HR-15) | do dono de cada crate |

---

## §7 — Estado de cada linha no fecho (para saber o que é NOVO quando algo ficar vermelho)

| linha | portão de fecho | binário release |
|---|---|---|
| `UIUX` | **22 208 / 22 208** verdes (`--no-fail-fast`, workspace). ⚠️ excluído `binary(the_census_of_every_primitive)` de `ph2d-field-eval` — **2 609 s** sozinho, corrido à parte: 28/28 | ✅ 10/09 15:23 |
| `Vector` | 2 vermelhos, **os dois flakes de carga confirmados** | ✅ 10/09 16:10 |
| `components` | **14 537 / 14 537** verdes, `exit 0` (a `load 12,6`) | ✅ 10/09 15:53 |
| `sculpt3d` | **14 720 / 14 720** verdes + `fmt`/`clippy`/`typos`/`doc-index`/2 vassouras de clean-room, todos `exit 0` | ✅ 10/09 15:51 |
| `motion-value` | verde; a **única** reprovada é flake de carga já catalogada | ✅ 10/09 15:33 |
| `3DModeling` | **22 234 / 22 235** na workspace — a única ✗ era tecto de LOC, **curada por corte** (3 ficheiros) | ✅ 09/09 23:14 |

⚠️ **`line/Sprite` (0 commits à frente) e `line/quadextract` (11 commits, base de 04/09, sem handoff)
NÃO entram nesta rodada.** A `quadextract` está **atrasada** — se entrar depois, precisa de rebase
próprio, e ⛔ medi-la com `git diff main..HEAD` dá um alarme falso (num ramo atrasado isso mostra ao
contrário tudo o que o `main` ganhou entretanto — a `line/3DModeling` apanhou este artefacto a
acusar `50` ficheiros partilhados que eram `1`).

---

## §8 — Os seis handoffs, na ORDEM de leitura (um de cada vez)

| # | ler ANTES da fusão nº | caminho |
|---:|---:|---|
| 1 | 1 | `Worktrees/line-UIUX/docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-10.md` — ⚠️ o **§6** é o que vai partir, e é a secção mais importante da rodada inteira |
| 2 | 2 | `Worktrees/line-Vector/docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A_SEGUNDA_MIDIA_2026-09-10.md` — §6 (sete leituras ao contrário), §9 (a proposta que **não é sua**), §10 |
| 3 | 3 | `Worktrees/line-components/docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_2026-09-10.md` — §2 (os dois cortes), §3 (conte o **delta**), §8 (sete leituras ao contrário) |
| 4 | 4 | `Worktrees/line-sculpt3d/docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-10.md` — §2 (as 5 crates com **zero** linhas fora de comentário), §7 (oito leituras ao contrário) |
| 5 | 5 | `Worktrees/line-motion-value/docs/Motion Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-10.md` — §3.1 (os símbolos públicos novos), §6 (o que o portão apanhou) |
| 6 | 6 | `Worktrees/line-3DModeling/docs/3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-09-10.md` — §2 (os números que se contam), §3 (o `Cargo.toml` da raiz), §6 |

⛔ **Não abra o nº 4 enquanto a fusão nº 3 não estiver verde.**

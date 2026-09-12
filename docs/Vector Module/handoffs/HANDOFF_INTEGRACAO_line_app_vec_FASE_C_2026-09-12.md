# HANDOFF DE INTEGRAÇÃO — `line/app-vec`, W2 **FASE C** (3.ª rodada) · 2026-09-12

> **Para o agente integrador e para a próxima janela desta linha.** O que mudou, o que a medição
> desmentiu, e o que a Fase D herda. Molde: DIRETRIZ §1.5.9.

| | |
|---|---|
| ramo | `line/app-vec` |
| merge-base | `db5a4b7f2` |
| HEAD | `872ee58cd` · **7 commits** |
| ficheiros tocados | 103 |
| shell `shells/desktop` | **373 937 → 350 130** LOC (−23 807, **−6,4 %**) · 1 498 → **1 424** ficheiros |
| crates novas | [`ph2d-app-skeleton`](../../../crates/ph2d-app-skeleton/) (19 f / 5 822 L) · [`ph2d-timeline-preview`](../../../crates/ph2d-timeline-preview/) (1 f / 142 L) |
| [`ph2d-app-vec`](../../../crates/ph2d-app-vec/) | 60 → **120** ficheiros · 13 216 → **31 376** LOC |
| contadores partilhados | **zero** mexidos (`PROJECT_SCHEMA` 128 = base, a tripla `(128, 13, 22)` idêntica, `FLIP_SCHEMA` 13, `DOC_VERSION` 18, os três registos 86) |
| contrato congelado (§6) | **intocado** · zero ADR |

---

## §1 — O que saiu, em sete commits

| commit | o que | LOC |
|---|---|---:|
| `bb3bd9530` | `widget_{drive,edit,value}` + `morph_edit` (os widgets autorados e as setas do Morph) | 2 096 |
| `dc73cdd64` | o **BALDE** e o **exportador de SVG** — o corte *lei ↔ ponte* que o bloco nomeia | 1 588 |
| `f53cdab3a` | a folha [`ph2d-timeline-preview`](../../../crates/ph2d-timeline-preview/) | 142 |
| `8d3f981b7` | **o ESQUELETO vira família** — `ph2d-app-skeleton` | 5 666 |
| `343e31b91` | **AS LEIS VIVAS** — align · blend · connector · contour · morph · offset · pattern · profile · symmetry · widget · expand · paint_dilate + o fx livre | 13 084 |
| `dc5eac2f8` | a edição da estampa + o gate do quadro do lápis | 1 463 |
| `872ee58cd` | `cargo fmt --all` sobre os 90 ficheiros que mudaram de árvore | −8 |

---

## §2 — ⛔⛔ O FECHO RE-MEDIDO, e a régua erra nos DOIS sentidos

O §11 da Fase B dizia *«15 ficheiros / 3 404 LOC movem hoje sem curar nada»*. Medido com o
`scripts/fecho-da-familia.py` da `line/app-motion` (autoteste **6/6** antes de acreditar nele):

| conjunto S | f / LOC | move hoje |
|---|---:|---:|
| a família por **NOME** (`vec_*` + `vector_bridge*`) | 82 / 24 319 | **17 / 4 109** |
| + as 16 leis vivas | 98 / 30 586 | **17 / 4 109** — *elas estão elas próprias presas* |
| + o **ASSUNTO** (`fx_*`, `texture_pattern_*`, `envelope_gesture`, `corner_handles`) | 168 / 50 252 | ⭐ **56 / 15 847** |

⭐ **O bloco mandava medir o fecho do GRUPO e tinha razão**: as leis vivas sozinhas movem **zero**
(cada uma prende a vizinha); como grupo, movem todas.

### ⚠️⚠️ As QUATRO correcções que a régua precisou, e DUAS erram a favor

| # | a sonda dizia | a verdade | o furo |
|---|---:|---:|---|
| 1 | `texture_pattern_live.rs` move | ⛔ é de **outra linha** | eu passei `--extra 'texture_pattern_'` **e** `--preso 'texture_pattern_live.rs'`: o ficheiro entrou em S pela porta da frente e bloqueava pela de trás. *Uma régua com um prefixo largo e uma excepção pontual dá as duas respostas ao mesmo tempo* ⇒ a cópia local ganhou um `--fora`, com o controlo de que um `--fora` que não casa **aborta** |
| 2 | os `*_live.rs` estão presos | movem | eu escrevi os nomes **exactos** (`align_live.rs`) e o irmão de teste ficou fora de S ⇒ o `#[path]` prendeu o par. *A solda é um GRUPO* — a mesma lição que a Fase B pagou, aplicada ao contrário |
| 3 | o esqueleto tem **6 âncoras** | **2** | ⭐ **cinco eram FACHADAS** (ver §3) |
| 4 | 5 pontes `impl App` movem | **ficam** | a régua salta o `::APP::` supondo que todo `impl App` vira trait de extensão. ⛔ Uma ponte que lê `self.gfx` precisaria de um **HANDLE**, e nenhum método do `AppHost` devolve um. **2 ocorrências** nesta rodada |

---

## §3 — ⭐⭐⭐ O ACHADO DESTA VOLTA: a FACHADA é uma crate a usar o nome da shell

O fecho do esqueleto deu **6 raízes da shell alcançáveis**. Lidas uma a uma, **cinco não são a
shell**:

| «âncora» | o que ela é de facto |
|---|---|
| `crate::skeleton_live` | `pub(crate) use ph2d_skeleton_live::skin_live::*;` |
| `crate::vec_bone_smoke` | `pub(crate) use ph2d_app_vec::smoke_bone::{…}` (8 nomes) |
| `crate::build_smoke::shape` | uma delegação de **uma linha** para `ph2d_vec_scene::cook_tinted` |
| `crate::timeline_preview` | a fachada que este mesmo dia criou |

⇒ **62 caminhos re-escritos** para onde a coisa vive, e as âncoras reais eram **duas**.

⚠️⚠️ **A régua só reconhece alias declarado no `main.rs`** — e o doc-comment dela di-lo por escrito.
Estes declaram-se **em si próprios**, num ficheiro com o nome antigo. *Um fecho que resolve caminhos
de módulo textualmente conta uma fachada como shell.*

⭐ **E ela erra no sentido CONSERVADOR** — diz que move MENOS do que a verdade —, que é o oposto do
erro habitual desta wave (a `motion` pagou `30×` no sentido contrário). Por isso ela passa
despercebida: *uma régua que subestima não autoriza nada, só desiste cedo.*

---

## §4 — ⭐ O ESQUELETO É FAMÍLIA PRÓPRIA — `ph2d-app-skeleton`

O bloco nomeava **5 ficheiros / 1 673 L**. O **assunto** são **24 / 6 803** — faltavam o
`skeleton_smart` (423), o `skeleton_reveal` (124), as duas sondas (589) e os 12 irmãos de teste.
*É a mesma forma que a Fase B achou (§4 daquele handoff) e desta vez apanhou-se ANTES de mover.*

- **Sai** (18 f): `bone_{pose,limit,pick,gesture}` · `goal` (+ `goal_tests` que é um **hub** de quatro
  irmãos soldados) · `smart` · `reveal`.
- **Fica** (7 f): as duas sondas `impl App` · duas fachadas de uma linha · o `skeleton_live_tests`
  (o sujeito dele é a **sequência do smoke**) · a ponte nova · os dois gates de sujeito-shell.

### ⛔ A catraca pediu uma ausência declarada — mas não a que ela sabia escrever

Esta família **não tem roteador e nunca vai ter**: a única cena que a exercita é a
`PH2D_VEC_BONE_SMOKE`, que monta um **braço vectorial** e o prende ⇒ ela é da `vec` tanto quanto do
esqueleto, e *quem possui a cena é quem a constrói*. Declará-la nas duas acorda o
`no_two_families_claim_the_same_router`.

⛔ E escrever `"skeleton"` em `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` seria **mentira permanente**:
aquela lista descreve uma extracção **a meio**, e o censo de obsolescência dela dispara quando a
família passa a declarar roteador — coisa que esta nunca fará. *Uma entrada que nada pode apagar é a
catraca que vira LICENÇA* (`CLAUDE.md` §5.0).

⇒ **`FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA`**, e a entrada é **VERIFICADA, não tolerada**: ela diz
*quem* não tem roteador **e** *quem o tem em seu lugar*, e o gate confirma que essa outra família
existe e declara mesmo aquela env. Provado por mutação nas duas pontas (env fantasma → sangra;
entrada apagada → sangra).

⚠️ **Para a `line/app-motion`:** a condição nova é escrita **à parte do `a_meio`** de propósito —
o bloco daquela linha manda apagar a lista antiga **inteira** quando a `motion` sair, e uma condição
que partilhasse a variável iria embora com ela, deixando a `skeleton` a reprovar por uma razão que
não é a dela.

---

## §5 — ⛔⛔ SETE gates partidos, e as TRÊS espécies do HOWTO apareceram todas

| espécie | quantos | como falha | quem o apanhou |
|---|---:|---|---|
| `include_str!` (§2.6) | **4** | ⭐ em **COMPILAÇÃO** — a metade boa | `cargo check --all-targets` |
| o gémeo em **RUNTIME** (§2.6) | **2** | ⛔ compila e explode só ao correr | **`cargo test --test it` À PARTE** |
| o **censo por directório** (§2.7) | **1** | ⛔ ficaria **verde a medir nada** | ⭐ o **piso de população** |
| a agulha que nomeia a **VISIBILIDADE** (§2.13) | **3** | reprova sem a lei mudar uma linha | a corrida |

### ⭐ O piso de população fez exactamente o trabalho dele

O `every_host_that_writes_world_geometry_is_in_the_list` varria `shells/desktop/src` e passou a ler
**1** host, porque os três que a mensagem dele **NOMEIA** (conector, blend, morph) mudaram de crate.
Sem o piso, `missing.is_empty()` sobre uma lista quase vazia seria trivialmente verdadeiro e o gate
ficaria verde a proteger nada. ⇒ ele varre as **duas árvores** agora e imprime os achados:
*ele fica mais forte do que era antes da mudança.*

### ⛔⛔ E um gate apanhou-me a re-apontar para onde o FICHEIRO foi

O `the_release_splices_through_the_same_door_the_preview_asked` mede se **a pré-visualização** lê a
porta da emenda. A pré-visualização é o `refresh_bone_hover`, que é `impl App` e ficou na **ponte**;
eu mandei a agulha para `crates/…/bone_pick.rs` porque foi para lá que o *ficheiro* se mudou.
Compilou e reprovou a correr.

⚠️ **3.ª ocorrência nesta linha** (o marquee na 2.ª volta · as setas do morph neste commit 1 · esta).
⇒ *re-apontar uma agulha é perguntar para onde foi o **SUJEITO** dela* — e aqui o ficheiro **partiu-se
em dois**, com o sujeito de cada metade diferente.

⚠️ **E a reescrita automática partiu uma agulha sozinha, em silêncio:** a regex trocou
`crate::vec_morph_edit` **dentro de uma string** que descreve o que a *shell* escreve. *Uma regex não
distingue «o endereço que este ficheiro usa» de «o endereço que o ficheiro MEDIDO usa».*

### ⭐ E uma cura que sobrevive ao PRÓXIMO movimento

O `the_shape_art_picker_is_wired` resolvia ficheiros por `read_to_string` de caminho fixo. A cura não
foi emendar o caminho — foi o **resolvedor** passar a tentar as duas árvores, com o `panic` a nomear
as duas. *Emendar o caminho cura este movimento; resolver as duas árvores cura o próximo.*

---

## §6 — ⭐ Onde o corte ficou, e porquê a régua é «quantos NOMES atravessam»

A 1.ª tentativa do balde partiu o ficheiro em *«tudo o que não é `impl App`»* — 320 L de lei contra
191 de ponte. O compilador acusou o corte: a ponte continuava a chamar **seis ajudantes privados** e
a ler **três campos** do `BucketCache`. Abri-los publicaria a cozinha inteira do módulo para servir
quatro métodos.

⇒ o que atravessa passou a ser **três funções livres** (`upkeep`, `hover`, `deposit`), com o que
precisam escrito em **TIPOS** (HOWTO §1.5), e a ponte caiu para ~110 L.

> ⚠️ **A régua do corte é quantos NOMES atravessam, não quantas linhas ficam de cada lado.** Um corte
> que obriga a abrir os ajudantes está no sítio errado.

⚠️ E o `armado` atravessa como **`bool`**, não como `DrawMode`: quem sabe que ferramenta está na mão
é a shell, e passar o enum faria esta crate depender do vocabulário do painel para responder a uma
pergunta de sim-ou-não.

---

## §7 — As DEZ dependências invisíveis que só a fronteira revela (HOWTO §1.3)

Esta família tinha **nove** ao fim da Fase B; a Fase C acrescentou **seis** e a do esqueleto trouxe
**onze** próprias.

| crate | quem a revelou |
|---|---|
| `ph2d-i18n` | o editor de widget traduz o rótulo de cada `WidgetKind` |
| `ph2d-morph-machine` | a chave da máquina de estados |
| `ph2d-vec-connect` · `ph2d-color` | o roteamento do conector · a pilha de aparência |
| `ph2d-gpu` + `ph2d-painter-effects` + `wgpu` | o `PH2D_FX_DUMP` lê uma textura de volta da GPU |
| `ph2d-vec-pattern` | a grelha de ladrilhos (`TileKind`, `HEX_ROW_RATIO`) |

⚠️ O `wgpu` escreve-se com a versão **literal** (`"29.0.4"`): este repo não tem
`[workspace.dependencies]`, e a `ph2d-gpu` faz o mesmo. ⚠️ E **nenhuma** das 340 crates declara
`[lints] workspace = true` — a 1.ª redacção do `Cargo.toml` da folha declarava, e alinhou-se.

### ⛔⛔ A FEATURE não viaja com o código (§2.4)

O `PreviewDrive::is_empty` é `#[cfg(any(test, feature = "test-support"))]` e três gates do esqueleto
o leem. Dentro da shell isso funcionava porque **não era do outro lado de nada**. Sem
`features = ["test-support"]` o erro lido é *«no method named `is_empty`»* sobre um método que está
**à vista no ficheiro que o erro cita**.

### ⚠️ E a família irmã entra em `[dev-dependencies]`, nunca em `[dependencies]`

Quatro ficheiros de **teste** do esqueleto exercitam a lei *através* da cena do osso, que vive na
`ph2d-app-vec` — e é assim que ela deve ser exercitada (HOWTO §1.2). Mas em `[dependencies]` isso
seria *uma família a depender da outra inteira*, que é o que aquela secção proíbe.
⭐ Zero ciclo, medido: a `ph2d-app-vec` cita **zero** módulos do esqueleto.

---

## §8 — ⛔ O TERRITÓRIO: o que NÃO se moveu, e de quem é

A regra 8 do §1 manda nomear. **Não toquei em nada fora do meu território.**

| ficheiro | dono | o que ele prende |
|---|---|---|
| `brush_live.rs` · `texture_pattern_live.rs` | **`line/app-motion`** | `fx_live.rs` (530) + `fx_live_memo` (197) + `fx_silhouette` (172) + os testes deles = **1 683 L** desta família, mais o `render_loop/vector_bridge_publish.rs` e o `vec_stroke_paint.rs` |
| `instance_*` (44 f / 13 413 L) | módulo **Components** | `vec_component_general` e os testes dele |
| `app_state.rs` · `input_dispatch.rs` · `render_loop/mod.rs` · `undo.rs` · `project_library.rs` · `build_smoke.rs` | **a shell, por desenho** | ver §9 |

⇒ **quando a `motion` levar o cluster de 632 L dela, esta família liberta mais ~1 900 L sem uma
linha de trabalho novo.**

---

## §9 — O que FICA na shell, e não é dívida

**59 ficheiros `vec_*` / 16 296 L** continuam lá, e a causa-raiz é medida:

| nº | causa | é dívida? |
|---:|---|---|
| 9 | `nomeia build_smoke.rs` | ⛔ **não** — são cenas do `PH2D_BUILD_SMOKE`, e o `build_smoke.rs` fica na shell com motivo medido (`gfx` em 41 sítios, ESTADO §3). Os nove pedem-lhe **uma** coisa, o `shape` |
| 6 | `nomeia app_state.rs` | a `App` — é a 3.ª rodada de famílias |
| 5 | `nomeia fx_live.rs` | ⛔ território da `motion` (§8) |
| 5 + 4 + 4 | `#[path]` com `vec_ui_state_edit` · `render_loop/vector_bridge` · `bool_live` | soldas dentro da família, que caem com a raiz delas |

⭐ **E o fecho chegou ao PADRÃO DE CHEGADA**: os **9 ficheiros / 1 029 L** que a régua ainda diz que
movem são **as pontes** (`vec_app_bridge`, os quatro prólogos de cena, o balde, o SVG) e o
`texture_pattern_smoke`, que recebe `app: &mut crate::App`. *O que sai são os CORPOS; o que decide a
ordem do quadro fica* (HOWTO §4).

---

## §10 — A PROVA

```
cargo nextest list --workspace --cargo-profile ci-test   (antes e depois)
python3 scripts/nextest-list-diff.py antes.txt depois.txt
```

| | |
|---|---|
| testes | **22 668 → 22 668** (22 041 chaves nos dois lados) |
| `MOVED` | **360** |
| **`ONLY-A` (perdidos)** | ⭐ **0** |
| **`ONLY-B` (novos)** | ⭐ **0** — *nada foi inventado* |
| roteadores `PH2D_*_SMOKE` | **106 = 106** (shell + as seis famílias, `main` contra agora) |
| `cargo test -p ph2d-host-desktop --test it` **À PARTE** | **812 / 812** |
| `cargo test -p ph2d-app-registry-init` | **5 / 5** |
| portão batched (`nextest-impacted.sh`) | **16 244 / 16 245** |
| `cargo fmt --all --check` · `typos` | limpos |
| clippy `--all-targets` | limpo — ⚠️ ficam os **dois pré-existentes** que o ESTADO §6 nomeia (`ph2d-app-sculpt3d/src/keys.rs`, `shells/desktop/src/sculpt_source/mod.rs`), e **não são desta linha** |
| `collision-surface.sh` | todo contador partilhado **na base**, contrato intocado, zero ADR, 2 pacotes novos e os dois **internos** |

### ⛔ O ÚNICO vermelho, e ele é esperado

`architecture_the_shell_only_shrinks` reprova pela **metade de OBSOLESCÊNCIA**:

```
a shell tem 350130 linhas e o tecto ainda diz 377937 — uma folga de 27807 linhas.
```

⛔ **Não toquei no `TETO_LOC`** (regra 1 do §1): ele **soma entre linhas** e é do integrador, sobre a
árvore combinada e **depois** do `cargo fmt --all`. ⚠️ O número que a catraca propõe para a **minha**
árvore isolada é `354_130`; o da árvore combinada será **menor**, porque a `motion` e a `physics`
também cortam nesta rodada. *Conte-o, não o copie daqui.*

---

## §11 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **«A `ph2d-app-skeleton` não declara roteador — a extracção está a meio.»** Não está: ela **nunca**
   terá roteador, e a ausência é declarada numa lista **nova e verificada** (§4).
2. **«Os `fx_*_smoke.rs` ficaram por esquecimento.»** Ficaram por **desenho**: são cenas do
   `PH2D_BUILD_SMOKE`, cujo roteador a shell lê.
3. **«O `vec_bucket.rs` continua na shell ⇒ o balde não saiu.»** Saiu: o ficheiro tem hoje **110 L**
   (eram 516) e é só a ponte.
4. **«O `timeline_preview.rs` continua na shell.»** É uma **fachada de 20 linhas**; a lei está na
   folha, e os 8 gates ficaram de propósito porque medem a `ProjectState::capture`.
5. **«A `ph2d-app-skeleton` depende da `ph2d-app-vec` ⇒ família a depender de família.»** É
   **dev-dependency**, e o build do produto não a vê (§7).
6. **«`pub(crate)` → `pub` em 144 sítios afrouxou a API.»** É o que a fronteira **obriga** (§2.13): do
   outro lado de uma crate, `pub(crate)` é inalcançável e o compilador diz `dead_code`, nunca
   *«não encontrado»*.
7. **«O `braco()` saiu do módulo de teste para o de produção.»** Saiu para `goal.rs` **sob
   `test-support`**, porque quatro módulos da crate **e** um gate da shell a partilham — e uma cópia
   do lado de lá divergiria no primeiro ajuste.

---

## §12 — O que a FASE D herda

1. ⭐⭐ **Quando a `motion` levar `brush_live` + `texture_pattern_live` (632 L), esta família liberta
   ~1 900 L** — `fx_live`, `fx_live_memo`, `fx_silhouette`, `vec_stroke_paint` e o
   `render_loop/vector_bridge_publish`. **Zero trabalho novo.**
2. **Os 30 bloqueadores restantes**, por número de pedintes: `build_smoke` (9, **não é dívida**) ·
   `app_state` (6) · `fx_live` (5, da motion) · `vec_ui_state_edit` (5) · `render_loop/vector_bridge`
   (4) · `bool_live` (4+4) · `envelope_live` (4).
3. ⏸️ **Agrupar os 59 `vec_*` restantes em `src/vec/`** — o bloco sugeria-o e **não o fiz**: ele não
   tira uma linha da shell e põe um diff grande no `main.rs`, que é **um dos cinco ficheiros de
   costura onde as três linhas desta rodada se encontram** (regra 9 do §1). ⭐ Fá-lo-ia de bom grado
   numa rodada em que esta linha corra sozinha — e aí a ferramenta do fecho passa a funcionar sem
   `--extra`.
4. ⏳ **O `texture_pattern_smoke` e as cenas `fx_*`** saem no dia em que o `PH2D_BUILD_SMOKE` tiver
   dono — hoje a shell lê-o, e o `const FAMILY` desta crate **declara por escrito** porque não o
   reclama.

---

## §13 — O SMOKE

Compilado nesta worktree, **duas corridas** (`CLAUDE.md` §2: `--profile smoke`, nunca `--release`):

| corrida | tempo | `Compiling` |
|---|---:|---|
| 1.ª (fria) | **19,44 s** | 6 crates |
| 2.ª (quente) | ⭐ **0,34 s** | **zero** |

Binário `target/smoke/ph2d-host-desktop`, **76 MB**.

As cinco cenas desta família continuam a ser as mesmas, com os mesmos nomes:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

…e o mesmo comando trocando a variável por `PH2D_VEC_STACK_SMOKE=1` · `PH2D_VEC_APPEARANCE_SMOKE=1` ·
`PH2D_VEC_FADE_SMOKE=1` · `PH2D_VEC_SVG_SMOKE=1`.

⚠️ **Nada de comportamento mudou nesta rodada** — é tudo mudança de endereço, e a prova do §10
(`ONLY-A = 0`, `ONLY-B = 0`, 360 `MOVED`) é exactamente essa afirmação medida.

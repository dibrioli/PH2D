# HANDOFF DE INTEGRAÇÃO — **A LINHA `line/components` INTEIRA** (2026-09-20)

> ⛔⛔ **SUPERSEDIDO como documento de INTEGRAÇÃO** por
> [`…_OS_ABERTOS_2026-09-20.md`](HANDOFF_INTEGRACAO_line_components_OS_ABERTOS_2026-09-20.md):
> a linha fechou aqui e o dono REABRIU-A no mesmo dia (*«vamos fechar o que está em aberto»*),
> e a superfície de colisão medida neste ficheiro **envelheceu na hora** — o `PROJECT_SCHEMA`
> andou mais **três** degraus desde então. ⚠️ **Leia a superfície LÁ.**
>
> ⭐ **O que fica vivo aqui é o §6** — a lição do rebase (as oito memórias do `main` órfãs num
> índice que esta linha compactou no mesmo dia, e as cinco contagens de família paradas que um
> gate DESTA linha acordou). Ele não foi reescrito no documento novo.

> **Para o agente INTEGRADOR.** Este é o documento do §1.5.9 da DIRETRIZ: a superfície de colisão
> MEDIDA, os contadores como DELTA, onde um merge textual pode colidir, a prova de fecho, e o que só
> a árvore combinada pode reprovar.
>
> ⚠️ **Leia a §4 antes de tocar num número.** A coluna `base:` de qualquer sonda é o MERGE-BASE, e a
> partir da 2.ª fusão de uma rodada ela está desactualizada **por construção**.

---

## §0 — Identidade

| | |
|---|---|
| ramo | `line/components` |
| HEAD | `d60407272` |
| merge-base com `main` | `76bd6de02` — **é o próprio `main`** |
| commits | **108** sobre 500 ficheiros |
| `git merge --ff-only` | ✅ **possível agora** (a linha foi rebaseada em 2026-09-20) |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components` |

⭐⭐ **O rebase já foi feito por esta linha, e ele foi limpo por uma razão medida: desde o fork o
`main` não moveu UMA LINHA DE CÓDIGO** — os 10 commits dele são todos de `project-memory/`. Prova:

```
git diff --stat <HEAD-antes-do-rebase> HEAD -- ':!project-memory'   # vazio
```

⇒ para toda grandeza de código, **a árvore combinada é esta árvore**.

---

## §1 — O que a linha entrega

**Sete waves**, todas com smoke do dono aprovado. O *porquê* de cada uma está no handoff dela (§3):

1. **TOP-20 #20 — `UiCanvas` / `UiLabel` / `UiButton`**: o placar, a vida e o menu sobre o jogo.
2. **SUPLENTE #24 — O GATILHO**: a mão de quem joga vira um sinal (`SignalOnAction`).
3. **#24, 2.ª metade — O SINAL SABE QUEM**: a linha da tabela responde *quando · **de quem** · a
   quem · o quê*, e ganha o verbo que tira da cena.
4. **SUPLENTE #25 — O ABANÃO DA VISTA**: `ShakeEmitter` + `CameraShake`, com a distância a atenuar.
5. **O LAÇO DE UM JOGO**: `SignalVerb::RestartRun` — a corrida recomeça sozinha.
6. **A ARMA DO JOGADOR**: `WeaponFire` — ritmo, pente com recarga, e o espalhamento na fábrica.
7. **AS ÂNCORAS DO HUD** (+ a 3.ª metade de 20/09): um filho de canvas segue as bordas REAIS.

⇒ **os cinco suplentes fecharam**, e o #20 está inteiro.

---

## §2 — ⛔⛔ SUPERFÍCIE DE COLISÃO, **medida** (`collision-surface.sh`, depois do rebase)

### §2.1 — Schemas

| grandeza | base | esta linha | **DELTA** |
|---|---|---|---|
| `PROJECT_SCHEMA` | `144` | `155` | **+11** |
| └ a tripla do gate | `(144, 13, 22)` | `(155, 13, 22)` | **+11** no 1.º |
| `VEC_SCENE_SCHEMA` | `22` | `22` | **0** |
| `FLIP_SCHEMA` | `13` | `13` | **0** |
| `DOC_VERSION` (timeline) | `18` | `18` | **0** |
| `FIELD_DOC_VERSION` | `23` | `23` | **0** |

⚠️ **Esta linha TOCA `project*.rs`** — a escada e a tripla moram em ficheiros IRMÃOS, e *um degrau
escrito no ficheiro errado funde LIMPO e evapora*.

### §2.2 — Registo de componentes: **TRÊS contadores**

| contador | base | esta linha | **DELTA** |
|---|---|---|---|
| `ph2d-ecs` | `91` | `103` | **+12** |
| `ph2d-render` (espelho) | `92` | `104` | **+12** |
| `ph2d-script` (espelho) | `92` | `104` | **+12** |

⛔ **Cada um só corre na suíte da PRÓPRIA crate** — *um portão que só corre o que a linha editou é
cego a todo espelho*, e esta casa pagou isso **quatro** vezes.

### §2.3 — Contrato congelado (§6) e ADR

- `crates/ph2d-nodegraph/src/node.rs` — **intocado** ✅
- `crates/ph2d-editor-core/src/tool.rs` — **intocado** ✅
- **Zero ADR** criado ⇒ a linha está fora de toda disputa de número (o disco está em `0170`).

### §2.4 — `Cargo.lock`: **zero pacote EXTERNO novo**

Os `5` `+name` são **arestas internas** (crates deste repo), não dependências de fora:
`ph2d-hud` · `ph2d-shake` · `ph2d-tween` · `ph2d-sprite-precision` · `ph2d-probe-cursor-grab`.

### §2.5 — Crates NOVAS (5)

| crate | o que é | dependências |
|---|---|---|
| `ph2d-hud` | a lei do canvas de HUD (`Fit`, `place`, `effective_box`, `clique`) | **zero** |
| `ph2d-shake` | a lei do abanão da vista | **zero** (nem `libm`) |
| `ph2d-tween` | a lei do tween | folha |
| `ph2d-sprite-precision` | as três portas de precisão de sprite | `ph2d-color` |
| `ph2d-probe-cursor-grab` | sonda de captura de cursor, **295 linhas que nada chamava** | `winit` |

### §2.6 — Tectos de LOC nos ficheiros tocados

| ficheiro | base | agora | |
|---|---|---|---|
| `shells/desktop/src/app_state.rs` | `976` | `976` | **intocado** — folga já declarada |
| `shells/desktop/src/main.rs` | `1102` | `1106` | `+4`, dentro da folga declarada |

⭐ **A catraca `the_shell_only_shrinks` está VERDE**, e a linha pagou-a **três vezes por CORTE**:
a fase `camera_2d` (`873` L) e a ponte do som para a `ph2d-app-components`, e o `hier_group`
(a lei de agrupar/desagrupar, `238` L com os gates) para a **`ph2d-app-vec`**.
⛔ **Nenhuma entrada nova no `FILE_OVERAGE_OK` em nenhuma das três.**

### §2.7 — Marcadores de conflito

**Nenhum**, e a varredura inclui `|||||||` (diff3), que uma varredura de três marcadores não vê.

---

## §3 — Os handoffs de wave (o *porquê* de cada pedaço)

| wave | handoff |
|---|---|
| #20 HUD | `HANDOFF_INTEGRACAO_line_components_HUD_2026-09-17.md` |
| a janela da cena | `HANDOFF_INTEGRACAO_line_components_A_JANELA_DA_CENA_2026-09-17.md` |
| vigia do contador | `HANDOFF_INTEGRACAO_line_components_COUNTER_WATCH_2026-09-17.md` |
| sequência | `HANDOFF_INTEGRACAO_line_components_SEQUENCE_2026-09-17.md` |
| #24 gatilho | `HANDOFF_INTEGRACAO_line_components_GATILHO_2026-09-18.md` |
| #24 o sinal sabe quem | `HANDOFF_INTEGRACAO_line_components_O_SINAL_SABE_QUEM_2026-09-19.md` |
| #25 abanão | `HANDOFF_INTEGRACAO_line_components_ABANAO_2026-09-19.md` |
| fim de jogo | `HANDOFF_INTEGRACAO_line_components_FIM_DE_JOGO_2026-09-19.md` |
| arma | `HANDOFF_INTEGRACAO_line_components_ARMA_2026-09-19.md` |
| âncoras do HUD (+ §8-ter de 20/09) | `HANDOFF_INTEGRACAO_line_components_ANCORAS_HUD_2026-09-19.md` |

---

## §4 — ⭐⭐⭐ Se outra linha aterrar PRIMEIRO: a receita do recontar

⛔⛔ **A coluna `base:` do `collision-surface.sh` é o MERGE-BASE, e a partir da 2.ª fusão de uma
rodada ela está desactualizada POR CONSTRUÇÃO** — em 10/09 ela dizia `PROJECT_SCHEMA 124 (base:
123)` com o `main` já em `127`, e quem leu *«+1»* landou um **retrocesso de 4 degraus**.
⇒ **leia o valor do `main` NO FICHEIRO, nunca na coluna.**

**O valor certo é `valor_do_main_de_agora + o DELTA da §2`** — nunca o literal desta linha:

```bash
git show main:shells/desktop/src/project_schema.rs | grep 'const PROJECT_SCHEMA'
git show main:crates/ph2d-ecs/src/scene/registry.rs | grep -E 'reg\.len\(\), *[0-9]+'
git show main:crates/ph2d-render/src/registry.rs   | grep -E 'reg\.len\(\), *[0-9]+'
git show main:crates/ph2d-script/src/registry.rs   | grep -E 'reg\.len\(\), *[0-9]+'
python3 scripts/schema-recount.py   # com `assert` em cada passo
```

⚠️ **Renumerar o `PROJECT_SCHEMA` mexe em TRÊS sítios** (o `const`, a escada, a tripla), e a escada
tem de ficar **contígua e na ordem** — um degrau duplicado lê bytes velhos **em silêncio**.

⚠️ **E a colisão passa MUDA quando as duas linhas escrevem o MESMO literal:** o git não sabe o que o
número significa (`CLAUDE.md` §5.0).

---

## §5 — Onde um merge TEXTUAL pode colidir

São **352** ficheiros fora da família. Os que carregam LISTA ORDENADA ou enum serializado são os que
importam — em todos, **a POSIÇÃO é a tag do postcard**, logo a entrada vai no **FIM**:

| ficheiro | o que esta linha lhe faz | a regra |
|---|---|---|
| `ph2d-ecs/src/signal_actions.rs` | `SignalVerb::ALL` **7 → 10** (`AddToCounter`, `Destroy`, `RestartRun`) + o campo `from` | **append-only**; ⚠️ o array de ids do selector tem de crescer com ele, senão o verbo existe e o artista não lhe chega |
| `ph2d-editor-core/src/ids/live_sections.rs` | `LIVE_SECTIONS` **28 → 38** | append-only; há censo a exigir que toda secção viva esteja na tabela |
| `ph2d-editor-core/src/action_bus.rs` | variantes de `EditorAction` | **append-only**; ⭐ o ficheiro **desceu** de `687` para `628` (corte por responsabilidade) |
| `ph2d-runtime/src/lib.rs` | variantes de `SignalOrigin` | append-only |
| `ph2d-preview-drive/src/lib.rs` | variantes de `Driver`/`Driven` | append-only |
| `ph2d-component-desc/src/catalog/*` | entradas novas de catálogo | por assunto; `hud.rs` é ficheiro novo |
| `ph2d-ecs/src/scene/registry.rs` + os dois espelhos | **+12** cada | ver §2.2 e §4 |
| `shells/desktop/src/project_schema*.rs` | **+11** degraus | ver §2.1 e §4 |
| `crates/ph2d-app-vec/{src/lib.rs, src/hier_group*.rs}` | **recebe** o `hier_group` da shell | ⚠️ se a `line/Vector` tocar no `lib.rs` dela, é um `pub mod` a mais — merge trivial, mas **é o único sítio onde esta linha entra na casa de outra** |
| `project-memory/**` | 12 ficheiros | ver §6.3 |

---

## §6 — Prova de fecho

### §6.1 — O que correu, com o número

| portão | resultado |
|---|---|
| `cargo nextest run --workspace --cargo-profile ci-test` | **24 878 / 24 878** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **zero** |
| `censos-da-arvore-combinada.sh` (HR-15 + LOC) | **90 / 90**, com `8 de 8` censos a correr |
| `cargo machete` | **zero** dependências por usar |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | **zero** |
| `check-standalone-optional.sh` | ✅ 10 crates |
| `check-workflow-packages.sh` | ✅ 32 nomes contra 384 membros |
| provas de mutação (só a última wave) | **23 de 23 sangram** |

### §6.2 — ⚠️ Uma flake CONHECIDA reprovou na corrida final

`ph2d-field-render tests::an_abandoned_march_returns_nothing_and_returns_fast` — **membro
confirmado** da família de flakes de fan-out (`CLAUDE.md` §5.0, ela é nomeada lá). As três
assinaturas, medidas:

- **3 de 3 verde sozinha**, com o `load` impresso ao lado (`10,89` · `9,60` · `9,60`);
- **zero linhas** do diff desta linha naquela crate (`git diff --stat main..HEAD -- crates/ph2d-field-render/` vazio);
- reprovou no pico de um fan-out de `24 878`.

⇒ **não é desta linha.** Se reprovar no seu portão, re-rode-a sozinha ANTES de olhar para o diff.

### §6.3 — ⭐⭐⭐ O que só a ÁRVORE COMBINADA apanhou (§1.5.9 5-bis), e a linha curou

O rebase fez aparecer **exactamente** o fenómeno que aquela alínea existe para descrever — *o gate é
escrito pela linha X e o que o acorda pela linha Y; nenhuma das duas árvores contém as duas coisas,
e as duas fecham verdes de boa-fé*:

1. **8 memórias do `main` ficaram ÓRFÃS no índice.** Esta linha **compactou** o `MEMORY.md` (cura de
   um defeito medido: `66` linhas eram cortadas em silêncio pelo carregador) no **mesmo dia** em que
   a `teste-cascadeur` escreveu oito memórias novas — a compactação apagou as linhas delas.
   ⇒ reconciliadas a **2 saltos** (nas famílias da régua e do ofício de gate), que é o desenho da
   compactação. Verificado: **zero órfãs**.
2. **Cinco contagens de família paradas**, apanhadas pelo gate
   `a_contagem_de_cada_familia_e_a_do_ficheiro` — que é **desta linha**. ⭐ **Três delas só cresceram
   no `main`** (`oracle 18 → 21`, `ui_seam 23 → 24`, `mutação 18 → 23`) e as outras duas aqui
   (`régua 132 → 136`, `gate 95 → 101`). Os cinco números são **CONTADOS no ficheiro**.

⛔ **Isto não substitui o portão da árvore combinada** (§1.5.3): o `--ff-only` continua a ser a única
prova de que ninguém aterrou entre este rebase e o merge.

### §6.4 — Higiene (§1.5.9 itens 7 e 7-bis)

- `rm -rf target/*/incremental` — **48 GB** reclamados (`44G` debug + `4,4G` smoke).
- Máquina livre: zero processos `cargo`/`rustc`/`ph2d` meus, zero binários meus na placa, a fatia
  `ph2d` a zero.
- ⭐ **O binário do smoke fica COMPILADO** (`target/smoke/ph2d-host-desktop`, 85,5 MB) — o dono não
  espera build.

---

## §7 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **`SignalVerb::ALL` de 7 para 10 não é «três verbos»** — são três verbos **e** o array de ids do
   segmentado a crescer com eles. Um gate apanhou `left: 9, right: 8` numa wave: sem o id, o verbo
   existe, tem lei, tem gates, e **o artista não lhe chega**.
2. **O `Spawned`, o `WeaponRuntime` e o `StateMachineRuntime` NÃO estão registados, e a ausência é a
   LEI** — *o que nasce numa corrida não é documento*. Registá-los poria as cópias no ficheiro e no
   `Ctrl+Z`.
3. **O `PROJECT_SCHEMA` sobe `+11` e há waves com `+1` e ZERO tipo novo** — entrar num registador
   que não corria no boot muda o ficheiro tanto como um componente novo.
4. **O `hier_group` a mudar de crate não é um refactor de arrumação:** foi a catraca da shell a
   cobrar `114` linhas, e a cura é **mover**, nunca subir o número. ⚠️ O 1.º candidato
   (`morph_set_tests`) foi **revertido** — ele declara um irmão que usa `crate::morph_live`, e *um
   gate que viaja sem o irmão não compila*.
5. **As cinco crates novas não trazem dependência externa nenhuma** — as três primeiras são folhas
   de **zero** dependências.
6. **A `ph2d-probe-cursor-grab` é código que NADA chamava** (295 linhas na shell): ela saiu para
   crate própria em vez de ser apagada porque é uma sonda com sujeito.
7. **O `Fit::Expand` entra no FIM do enum e o `PROJECT_SCHEMA` não se mexe por ele** — a posição é a
   tag do postcard, e um modo apendado deixa os ficheiros gravados legíveis.

---

## §8 — ⏳ Aberto (não bloqueia a integração)

- **No EDITOR, um HUD colado às bordas cai atrás dos painéis** — a vista da câmera é a da JANELA e o
  mundo é desenhado entre os painéis. A cena do HUD é calibrada à volta disto (fecha a timeline,
  recua as peças `5` de `16`) **com a calibração e o limite dela escritos**; a cura geral é wave
  própria.
- **Um botão feito de *sprite* não é pego** — só o caminho vectorial.
- **A `scene_window` é a porta de SEIS gestos vectoriais** e a shell tem `72` chamadas de
  `screen_to_world` em `33` ficheiros contra `19` sítios que já passam pela banda. Esta linha curou
  **as duas do vector** e deixa as outras NOMEADAS.
- **Nenhuma das 26 secções opcionais do Inspector tem gate a provar que ela chega a PIXEL** — a
  presença é gateada, a pintura não, e o arnês não existe naquela crate.
- Os abertos por wave estão no §9 de cada handoff da §3.

---

## §9 — O SMOKE (o binário já está compilado)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_HUD_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

Os outros da linha: `PH2D_TRIGGER_SMOKE=1` · `PH2D_DANO_SMOKE=1` · `PH2D_SHAKE_SMOKE=1` ·
`PH2D_RESTART_SMOKE=1` · `PH2D_WEAPON_SMOKE=1` · `PH2D_COUNTERWATCH_SMOKE=1`.

⚠️ **Todos foram smokados e aprovados pelo dono.** Nada nesta linha espera smoke.

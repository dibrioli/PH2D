# Handoff de INTEGRAÇÃO — `line/app-flip`, W2 **FASE B, 2.ª volta** (2026-09-12)

> **A família saiu.** O bloqueio do §3 do handoff de 11/09 — *«duas funções de outra família
> prendem 84 % de mim»* — **dissolveu** quando a `line/shell-folhas` levou o `vec_transform` e
> o `name_unique` para crates-folha em 12/09. O fecho re-medido autorizou o corte inteiro.
>
> Fase B (1.ª volta): [`…FASE_B_2026-09-11.md`](HANDOFF_INTEGRACAO_line_app-flip_FASE_B_2026-09-11.md) ·
> molde: [`HOWTO_partir_uma_familia_da_shell.md`](../../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md) ·
> estado da wave: [`ESTADO_W2_2026-09-12.md`](../../IntegracaoMultiAgente/ESTADO_W2_2026-09-12.md).

## 1 — Identidade

| | |
|---|---|
| branch | `line/app-flip` |
| merge-base | `27ab89429` (main de 12/09, pós-`line/shell-folhas`) |
| commits | 4 |
| **contadores partilhados** | **NENHUM se move** — `PROJECT_SCHEMA` **128**, `FLIP_SCHEMA_VERSION` **13**, os três registos do `ph2d-ecs` **intocados** (zero linhas de diff naquela crate) |
| contratos congelados (§6) | **intocados** (`node.rs`, `tool.rs` — zero linhas) |
| ADR novo | nenhum ⇒ fora de toda disputa de número |
| pacote EXTERNO novo no `Cargo.lock` | **nenhum** — as 10 entradas novas são todas internas mais o `wgpu`, que já lá estava |
| `TETO_LOC` do `the_shell_only_shrinks` | ⛔ **NÃO tocado** (regra 1). O número medido está no §5 |

## 2 — ⭐⭐⭐ O fecho re-medido, e a régua que se corrigiu TRÊS vezes

**O passo 1 era re-medir, e ele mudou o plano inteiro.** A régua é o *fecho* — *«a partir dos
ficheiros que quero mover, que raízes da shell continuam alcançáveis?»* —, e ela errou três
vezes antes de responder:

| corrida | leitura | o furo |
|---|---:|---|
| 1.ª | **1 241 raízes** | tratou `mod x;` como aresta de **mão dupla**: subindo de `flip/mod.rs` chega-se ao `main.rs`, e dali à shell inteira. ⚠️ Só o **`#[path]`-filho** é aresta dura; a subida por `mod` é o **corte desenhado** (o pai perde a linha) |
| 2.ª | **2 raízes** | a regex de caminhos só apanhava minúsculas (`crate::[a-z_]`) ⇒ **`crate::App` nunca entrou no grafo** — e é precisamente o acoplamento que a lei nº 1 do estado diz que escapa |
| 3.ª | **4 símbolos** | ✅ |

**O resultado.** A família inteira (74 f / 18 845 L) nomeava da shell, fora de si mesma:

| o que | usos | ficheiros |
|---|---:|---:|
| `crate::App` (+ `::new`) | 39 | 35 |
| `crate::undo::ProjectState` | 1 | 1 |
| `crate::project_library::LibraryDoc` | 1 | 1 |

⇒ **zero cascata.** A árvore de 28 módulos / 16 012 L do §3 de 11/09 descrevia uma árvore que
já não existia.

### A pergunta da regra 4, escrita em TIPOS

| pedido | usos | tipo | dona |
|---|---:|---|---|
| `self.flip_state` | 156 | `FlipState` | **a própria família** |
| `gfx.flip` | 82 | `FlipDoc` | `ph2d-flip` |
| `self.playhead` | 46 | `Playhead` | `ph2d-core` |
| `gfx.camera` | 25 | `Camera2d` | `ph2d-render` |
| `self.title_dirty` | 21 | `bool` | — |
| `gfx.tools` | 19 | `ToolRegistry` | `ph2d-editor-core` |
| `gfx.surface` | 17 | `SurfaceContext` | `ph2d-gpu` |
| `self.modifiers` | 16 | → `AppHost::mods()` | já é porta |
| `gfx.toasts` | 8 | `ToastQueue` | `ph2d-editor-core` |
| `gfx.sim` | 8 | `SimWorld` | `ph2d-ecs` |
| `gfx.hero_screen` | 7 | `HeroScreen` | `ph2d-editor-core` |

⭐⭐ **ZERO tipos locais da shell.** ⇒ **ASSINATURA**, nunca um 6.º método no `AppHost` —
a regra 4 à letra. **Zero sextos métodos pedidos.**

## 3 — O que saiu, em três cortes

| # | o que | ficheiros | LOC |
|---|---|---:|---:|
| 1 | os **13 blocos `impl crate::App`** de lei viram funções livres + invólucro fino | — | 5 257 reescritos |
| 2 | **53 ficheiros** por `git mv` para `crates/ph2d-app-flip/` (43 de `flip/`, 10 de `render_loop/flip_*`) | 53 | 17 485 |
| 3 | os gates que a fronteira partiu | 7 | — |

**A shell: `398 037` → `381 328` linhas (−16 709), `1 561` → `1 521` ficheiros.**
A crate: `8 647` → `26 255` linhas.

⚠️ **`shells/desktop/src/render_loop/flip_*.rs` deixou de existir** — os 10 foram todos.
Fica em `flip/` **2 271 L em 36 ficheiros**: os 33 invólucros `impl crate::App`, o `mod.rs`,
e os **dois testes cujo sujeito é a shell** (§4).

⭐ **`ctx.rs` — o agrupador, e por que ele NÃO é um handle.** A shell constrói um
`FlipFrame { flip, playhead, camera, win }` no sítio de chamada que ela controla. O HOWTO
§1.5 proíbe um método do **trait devolver** um handle (aí a família alcança tudo, a qualquer
hora); um parâmetro é a shell a escolher o que entrega — o mesmo precedente do `HeroScreen`
da 1.ª volta. ⭐ E ele apanhou uma lei escrita **cinco** vezes: a conta `px_to_world`
(`height_world.max(EPSILON) / height.max(1)`) estava copiada em cinco blocos.

## 4 — ⚠️ As SETE armadilhas que esta volta pagou

1. ⛔⛔ **As FEATURES não viajam com o código** (§2.4, a MUDA). Dois `cfg` desta família
   (`strip_drag.rs`, `bridge.rs`) governam a fiação com os painéis do Flip. Numa crate que
   não as declara eles são falsos **por construção** e ela compila **verde** com o arrasto
   da tira a não chegar ao painel de quadros. ⇒ a crate declara `panel-flip`/
   `panel-flip-frames`, a shell **repassa-as**, e há **gate novo** com a metade de
   obsolescência (`the_shell_turns_on_the_features_this_family_reads`).
   ⭐ **Provado nos dois sentidos:** com as features desligadas o compilador acusa 4 params
   por usar; com elas ligadas (o caminho do produto, e o `default` da shell) o aviso some.
2. ⛔ **O `#[path]` é aresta dura nos DOIS sentidos, e desta vez o PAI saiu e o filho não
   pôde vir.** O `entities_fixpoint_tests` afirma sobre o **`ProjectState`** — sujeito da
   shell (§2.6) — então deixou de ser filho do `entities` e foi re-alojado em `flip/mod.rs`.
   ⚠️ *Um `#[path]`-filho move-se com o pai, excepto quando o sujeito dele fica.*
3. ⛔ **Os testes seguem o SUJEITO, não o ficheiro** (§1.2). Quatro testes dentro de
   `colorize_tests`/`select_segment_tests` dirigem a `App` headless ⇒ foram para
   `flip/seam_tests.rs`. ⚠️ **E não podiam ir para `shells/desktop/tests/it/`**: aquilo é uma
   crate de integração e `App::flip_state` é `pub(crate)` — um gate de lá não o alcança.
   Em sentido contrário, os dois gates que medem a **lei** (`the_eraser_…`,
   `the_flip_pass_…`) vieram para `crates/ph2d-app-flip/tests/it/` (harness novo, um binário
   só — DIRETRIZ §6.3).
4. ⛔⛔ **A agulha ancora na LEI, e ela mente de DUAS maneiras** (§2.13):
   - pelo **endereço**: `ph2d_app_flip::pass_stage::fingerprint(` vira `crate::pass_stage::…`
     **dentro** da crate, e `flip_bridge::publish` vira `ph2d_app_flip::bridge::publish` na
     shell;
   - pela **visibilidade e o nome**: `pub(crate) fn flip_colorize_apply(&mut self)` era um
     método e hoje é `pub fn apply(state, f, toasts, w2l)`.
   ⚠️⚠️ **E a 1.ª redacção da minha própria correcção mordeu a lei que citava:** escrevi a
   agulha `pub fn apply(` sobre um `pub(crate) fn apply(`. *Ancorar no modificador é o
   defeito, e ele reaparece dentro da cura.*
5. ⚠️ **O `nextest-impacted` NÃO alcança `shells/desktop/tests/it/`** (lei nº 4, e foi ela
   que me reprovou na árvore combinada em 11/09). A corrida à parte apanhou **6** gates
   vermelhos que o portão batched dava por verdes. ⇒ **corra-a sempre.**
6. ⛔ **A minha própria régua de prosa deu um ZERO falso.** O censo de roteadores usava o
   stripper que tira comentários **e strings** — e os roteadores *são* strings (`r("PH2D_…")`).
   Ele leu `0 envs na crate` sobre 23 reais. *Uma régua correcta aplicada à pergunta errada
   mente com a cara de uma medição.*
7. ⚠️ **Uma regex não-gulosa transborda para a função seguinte.** O meu desempacotamento de
   `cursor` aterrou dentro do `delete_selected`, que vem depois. O `assert` de contagem
   apanhou-o; sem ele teria compilado com um `let (x, y)` órfão.

## 5 — A prova (§3 do HOWTO · regra H)

**(a) Nenhum teste se perde** — `cargo nextest list --workspace --cargo-profile ci-test`,
antes (na base) e depois, por `scripts/nextest-list-diff.py`:

```
antes: 22665 testes (22038 chaves) | depois: 22667 (22040)
MOVED (mesma chave, outro pacote/binário): 211   → ph2d-app-flip
ONLY-A (perdidos): 0          ✅
ONLY-B (novos):    2          → os dois gates de feature do §4.1
```

**(b) Nenhum roteador se perde** — censo com os comentários retirados (⚠️ e **não** as
strings, §4.6):

| | |
|---|---|
| `PH2D_FLIP*` lidos por CÓDIGO na shell | **NENHUM** |
| lidos por código na crate | 23 (18 `*_SMOKE` + 5 de diagnóstico) |
| declarados no `const FAMILY` | **18** |
| declarados que ninguém lê / lidos não declarados | **0 / 0** |

⛔ Os 5 de fora são diagnóstico (`PH2D_FLIP_DEMO`, `_FILL_DEBUG`, `_SELECT_DEBUG`,
`_NEW_ENGINE`, `_STATS`) — um registo que os aceitasse prometeria ao dono cenas que não
existem. `cargo run -p ph2d-app-sync`: 4 blocos regenerados, **sem diff**; os 2 gates de
*staleness* passam.

**(c) A shell encolheu** — pela régua do GATE (a pasta da crate inteira, com `tests/`):

| | base | agora | Δ |
|---|---:|---:|---:|
| linhas | **398 037** | **381 328** | **−16 709** |
| ficheiros | 1 561 | 1 521 | −40 |

⛔ **`TETO_LOC` NÃO foi tocado** (regra 1). A catraca **reprova pela metade de
OBSOLESCÊNCIA** — que é o marcador de progresso: ela mede `381 333` contra o tecto de
`402 037` e pede `385 333` (o medido + a folga de composição de 4 000). ⚠️ **O número certo é
do INTEGRADOR, sobre a árvore combinada, e DEPOIS do `cargo fmt --all`** (lei nº 5).
⚠️ A pequena diferença (`381 328` aqui, `381 333` no gate) é o gate contar de outra maneira o
ficheiro final sem newline — leia o número **dele**, não o meu.

**(d) Gate de fecho** — `scripts/nextest-impacted.sh`: **15 020 testes, 15 019 passaram,
1 falhou** — e a única é o `the_shell_only_shrinks` acima. `load 5,55` impresso ao lado.
⚠️ Corrida **à parte** da suíte que o impacted não alcança: `cargo test -p ph2d-host-desktop
--test it` → **812 passaram, 0 falharam, 6 ignorados**. Crate: **270** (lib) + **6**
(`tests/it`). `clippy --all-targets` nas duas: **0**. `cargo fmt --all --check`: limpo.

⛔ **Ficam DOIS avisos de clippy PRÉ-EXISTENTES**, que o `ESTADO_W2` §6 já nomeia
(`ph2d-app-sculpt3d/src/keys.rs`, `shells/desktop/src/sculpt_source/mod.rs`) — não são desta
linha, e o `ship.sh` corre com `-D warnings`.

**(e) Smoke COMPILADO** (DIRETRIZ §1.5.9 item 9 · regra I) — 2.ª corrida:

```
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

**Zero linhas `Compiling`.** Binário em `target/smoke/ph2d-host-desktop` (75 MB).

⚠️ O `target/*/incremental` foi reclamado a seguir (DIRETRIZ §1.5.9 item 7): **14,5 GB**
(12 de `debug` + 2,5 de `smoke`). O perfil `smoke` recria o dele na 1.ª corrida, que é a que
acabou de correr — as duas coisas não se anulam.

## 6 — ⭐⭐ O que esta linha DESTRAVA para a próxima (o item que o briefing pediu)

**O `undo.rs` deixou de ser preso pela família Flip.** Medido depois do corte, com os
comentários retirados:

| | |
|---|---|
| menções a `App` em `undo.rs` | **ZERO** |
| o `FlipEntityMap` que ele usa | `ph2d_app_flip::entities::FlipEntityMap` (uma **crate**) |
| o que ele ainda nomeia da shell | **`crate::project_library`, e só** (2 usos) |

⇒ o bloqueador do `undo` passou a ser **uma folha de 132 linhas**, que por sua vez nomeia
só `project_catalogs` e `asset_index_build` (2 usos cada). ⚠️ O `undo.rs` tem **21**
consumidores na shell e o `project_library.rs` **24** — a wave que os tirar move mais do que
estes dois ficheiros, e **não é desta linha**.

⚠️ E o `ESTADO_W2` §3 diz *«`undo.rs` (atravessa para a família `flip`)»* como motivo de ele
ficar: **essa razão já não vale.** Quem escrever o próximo estado tem de a reescrever.

## 7 — Foundational tocado, e símbolos novos

**Foundational:** `shells/desktop/src/flip/mod.rs` · `shells/desktop/src/render_loop/mod.rs`
(−5 `mod`) · `shells/desktop/src/main.rs` (os 2 chamadores do `select`/`select_points`) ·
`shells/desktop/src/render_loop/present.rs` · `shells/desktop/Cargo.toml` (o **repasse das
duas features**) · `crates/ph2d-app-registry-init/src/lib.rs` (**gerado**, não editado).

| símbolo novo | onde |
|---|---|
| `ph2d_app_flip::ctx::FlipFrame` (+ `to_world`, `px_to_world`) | a crate |
| `FlipColorize::scribble_count()` | acessor, para o gate da costura não precisar do campo |
| `fill::FillOutcome` | o par `(consumido, avisou)` que atravessa por VALOR |
| `crates/ph2d-app-flip/tests/it/` | harness novo (1 binário) + 3 módulos |
| `shells/desktop/src/flip/seam_tests.rs` | os 4 testes que seguiram o sujeito |
| deps novas da crate | `ph2d-ecs`, `ph2d-render`, `ph2d-gpu`, `ph2d-host`, `ph2d-flip-colorize`, `ph2d-vec-entities`, `ph2d-unique-name`, `wgpu` (⚠️ invisível até a crate existir, §1.3) + `ph2d-panel-flip{,-frames}` opcionais |
| features novas da crate | `panel-flip`, `panel-flip-frames` |

⛔ **Nenhum id, const, variant, token ou schema novo.** Nada a colidir.

⚠️ **68 + 18 + 6 itens passaram de `pub(crate)`/`pub(super)` a `pub`** — é o que publicar a
API de uma família custa, e é a causa raiz da armadilha §4.4. Cada um foi alargado **porque o
compilador o nomeou**, nunca em lote.

## 8 — O que smoke-testar (nada mudou de produto — é isso que se confirma)

Os três que mais se mexeram, os dois primeiros porque a lei inteira deles trocou de sítio e o
terceiro porque é o mestre do módulo:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_COLORIZE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_STRIP_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

## 9 — O que só o `ship.sh` apanha

`fmt` e `clippy --all-targets` correram aqui (§5); **não** correram: `machete` (a crate ganhou
**8** deps — é o candidato mais provável a um `✗`), `deny`, `audit`, `typos`, e o `doc-index`
sobre `docs/Flip/handoffs/README.md`, que ganha esta entrada.

## 10 — Para o integrador

1. **Reconte o `TETO_LOC`** sobre a árvore junta, **depois** do `cargo fmt --all` (lei nº 5).
   A minha ponta é `381 333` pela régua do gate.
2. ⚠️ **Corra `cargo test -p ph2d-host-desktop --test it` À PARTE** na árvore combinada — o
   `nextest-impacted` não a alcança, e foi ali que esta linha reprovou em 11/09.
3. ⭐ **O §6 é a única coisa que esta linha PEDE:** a nota do `ESTADO_W2` §3 sobre o `undo.rs`
   está obsoleta, e o bloqueador dele encolheu para uma folha de 132 linhas.
4. ⚠️ **`motion` e `vec` continuam em `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`** — a `flip`
   nunca esteve lá, e continua fora.

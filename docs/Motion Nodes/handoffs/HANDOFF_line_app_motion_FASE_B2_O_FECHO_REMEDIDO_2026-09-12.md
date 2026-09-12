# `line/app-motion` — FASE B (2.ª volta): **o fecho re-medido, e o bloqueador encolheu de 13 módulos para 5 ficheiros** (2026-09-12)

> **Veredito:** a família `motion` **continua a não se cortar**, e agora sabe-se exactamente porquê.
> O fecho re-medido dá **7 âncoras** (eram 26 em 11/09). **3 são minhas e curam-se dentro desta
> linha.** As outras **4 são 5 ficheiros — 1 579 LOC — dentro das árvores das DUAS linhas que estão
> abertas neste momento** (`line/app-flip` e `line/app-vec`).
>
> ⭐⭐ **E o contrafactual é a parte que decide, porque ele mudou de forma:**
>
> | cenário | move | fica |
> |---|---:|---:|
> | **A — hoje** (flip e vec vivas) | **13 f / 2 713 LOC** (2,7 %) | 406 f / 99 347 |
> | **C — só o flip curado**, vec ainda viva | **13 f / 2 713 LOC** | 406 f / 99 347 |
> | **B — flip E vec curados** | **419 f / 102 060 LOC** | **0** |
>
> ⇒ **o tudo-ou-nada de 11/09 CONFIRMA-SE, e o seu preço é agora um NÚMERO PEQUENO e de outra
> gente.** A `motion` é **estruturalmente a última** da W2: ela é a única das três cujo fecho
> depende das **outras duas**.
>
> ⇒ **PARE e reporte**, pela regra 5 do [§1 dos blocos](../../IntegracaoMultiAgente/BLOCOS_REABERTURA_W2_FASE_B2_2026-09-12.md)
> (*não edite a árvore de outra linha*) e pela lei 6 do
> [ESTADO](../../IntegracaoMultiAgente/ESTADO_W2_2026-09-12.md) (*uma linha de folhas abre quando
> ninguém está lá dentro*). **Zero produto movido** — outra vez, e desta vez com o pedido exacto.

---

## §1 — O que esta jornada ENTREGOU (não é zero)

| | |
|---|---|
| ⭐ **`scripts/fecho-da-familia.py`** | a régua do FECHO que a W2 pede desde o HOWTO §2.12 e que **nunca existiu**; 6 controlos positivos (`--autoteste`) |
| **HOWTO §1.1-bis** | o passo 1 da tabela de operações passa a **invocá-la pelo nome** (*ponteiro não é adoção*, `CLAUDE.md` §2) |
| **o fecho de 2026-09-12** | 7 âncoras, com a cadeia causal e a causa raiz por contagem |
| ficheiros de produto movidos | **0** (§5) |
| `shells/desktop/src` | **363 874** LOC — inalterado |
| contadores partilhados | inalterados · `TETO_LOC` **não tocado** · 6.º método no `AppHost` **nenhum** |

---

## §2 — ⛔⛔ A régua mentiu QUATRO vezes, e TRÊS foram a favor

*Este é o núcleo do handoff. A lei nº 1 do ESTADO é desta linha, e eu voltei a pagá-la — mas desta
vez apanhei-a **antes** de mover um ficheiro.*

| # | a mentira | o número | direcção |
|---|---|---|---|
| 1 | ⛔⛔ **branquear strings APAGA o grafo de `#[path]`** | `motion_state.rs` tem **45** `#[path]` e a regex apanhou **0** ⇒ o ponto fixo correu com **zero arestas duras** e devolveu **246 f / 56 866 LOC movíveis**. Com as arestas a sério: **13 f / 2 713**. Erro de **21×** | **A FAVOR** |
| 2 | ler **visibilidade** como dependência | `pub(in crate::render_loop)` deu **7 âncoras falsas** para o laço da shell — o `motion_bridge_tutorial_draw.rs` não tem nenhuma | contra |
| 3 | não reconhecer os **alias-para-crate** de `main.rs` | `crate::vec_glyph` já é `ph2d_app_vec::glyph` desde 11/09 — **16** alias, e sem os ler aparecem 16 âncoras falsas | contra |
| 4 | ⛔ **o acoplamento que viaja por um CAMPO** | `app.flip_state` (tipo `crate::flip::state::FlipState`) é uma **âncora inteira** que nenhuma varredura por `crate::` vê | **A FAVOR** |

⭐⭐ **A nº 1 é a mesma que a `line/app-vec` pagou** (`CLAUDE.md` §5: *«um defeito da própria sonda —
branquear strings apaga o grafo de `#[path]`, que tem 801 arestas»*). **Estava escrita no `CLAUDE.md`
que eu recebo antes da primeira palavra, e eu reproduzi-a na mesma.** ⇒ a cura não é uma nota: é o
`scripts/fecho-da-familia.py` ter **dois strippers com nomes que dizem para que servem** e um
controlo positivo que reprova se alguém os trocar.

⚠️ **E a nº 4 é a minha de 11/09 a repetir-se do outro lado:** naquele dia ela inflacionou o movível
(`15 → 7` âncoras); hoje ela **escondeu uma âncora**. *Uma régua que não segue campos erra nos dois
sentidos, e não se sabe qual antes de a corrigir.*

⚠️ **Uma quinta, menor, dentro do próprio instrumento:** o censo de campos compilou a regex **sem
`re.M`** e imprimiu a secção **VAZIA** — verde a varrer nada (HOWTO §2.7). Tem **piso de população**
agora (`assert tipos`).

---

## §3 — O FECHO, medido (`python3 scripts/fecho-da-familia.py motion --extra 'warp_'`)

`S` = **419 ficheiros / 102 060 LOC**.

⚠️ **São 419 e não os 411 do bloco**: o cluster **`warp_*`** (8 ficheiros) é **código do motion** — o
`warp_gizmo.rs` abre com *«o gizmo de canvas dos DEFORMADORES DE QUADRILÁTERO — as alças que o
`motion.four_point_warp` e o `motion.bezier_warp` passam a ter na tela»*. ⭐ *O censo por PREFIXO de
ficheiro não é a família* — é a §2.7 do HOWTO a morder no sentido oposto ao habitual: ali o prefixo
varre **de menos** na crate nova; aqui ele varreu **de menos** na shell.

### As 7 âncoras

| # | âncora | LOC | cura | dono |
|---|---|---:|---|---|
| 1 | `crate::App` / `crate::AppGfx` (16 f) | — | **`MotionSceneCtx`** — §4 | **eu** |
| 2 | `picker_smoke.rs` | 187 | **só a motion o consome** ⇒ entra na crate | **eu** |
| 3 | `field_gizmo.rs` | 583 | **não é folha** — lê `MotionState` e `motion_bridge::params` ⇒ é código da motion, vai junto (alias na shell para os 7 consumidores) | **eu** |
| 4 | `thumbnail.rs` | 115 | folha **PURA** (`0` `crate::`, `0` crates irmãs) ⇒ `ph2d-thumbnail` | **eu** |
| 5 | `flip/entities.rs` + `flip/transform.rs` + `flip/state.rs` | 228+494+115 | crate-folha | ⛔ **`line/app-flip`** |
| 6 | `render_loop/flip_pass.rs` + `flip_pass_camera.rs` | 598+144 | idem | ⛔ **`line/app-flip`** |
| 7 | `brush_live.rs` → `texture_pattern_live.rs` | 223 → 409 | folha | ⛔ **`line/app-vec`** (plano 33/W4 — é a estampa do vetor) |

### A cadeia causal (porque 2,7 % e não mais)

```
flip/{entities,transform,state}.rs ─┐
render_loop/flip_pass*.rs          ─┼─r1─▶ motion_flip_bake.rs        ─r2─▶ motion_state.rs
brush_live.rs → texture_pattern_live ┘      motion_bridge_objects.rs          │
                                            motion_object_bake.rs             ▼
                                            motion_bridge.rs          106 ficheiros nomeiam-no
                                                                      + 42 são #[path] filhos dele
```

**causa RAIZ por contagem:** `motion_state.rs` **83** · `motion_state_conferencia_mods.rs` (`#[path]`) **45** ·
`motion_state.rs` (`#[path]`) **42** · `motion_demo_legend.rs` **40** · `motion_bridge.rs` **25 + 20**.

⭐⭐ **`MotionState` tem um campo `FlipObjectBake`**, que vive no `motion_flip_bake.rs` ⇒ esse ficheiro
**não pode ficar para trás**, e com ele vêm os 5 símbolos do flip. *Um campo de struct é uma aresta
tão dura como um `#[path]`, e é por isso que 5 ficheiros de outra gente prendem 390 dos meus.*

---

## §4 — ⭐ O `crate::App`/`AppGfx` é ASSINATURA, e está PROVADO (zero sextos métodos)

Pela regra 2 (*«antes de pedir porta, escreva o que a função PRECISA em tipos»*), resolvi **cada
campo de `App`/`AppGfx` que a família toca ao TIPO dele**:

| campo | f | tipo | dono |
|---|---:|---|---|
| `gfx.motion` | 14 | `crate::motion::motion_state::MotionState` | ⭐ **vai comigo** |
| `gfx.tools` | 14 | `ph2d_editor_core::tool::ToolRegistry` | ✓ crate irmã |
| `gfx.sim` | 4 | `ph2d_ecs::sim::SimWorld` | ✓ crate irmã |
| `gfx.vec_scene` | 3 | `ph2d_vec_scene::VecScene` | ✓ crate irmã |
| `app.vec_entities` | 2 | `ph2d_vec_entities::entities::VecEntityMap` | ✓ (a `line/shell-folhas` tirou-o em 12/09) |
| `gfx.flip` | 1 | `ph2d_flip::FlipDoc` | ✓ crate irmã |
| `app.playhead` · `app.timeline` | 1 · 1 | `ph2d_core::Playhead` · `ph2d_timeline::TimelineState` | ✓ crates irmãs |
| `app.motion_shell` | 1 | `crate::motion::motion_shell_state::MotionShellState` | ⭐ vai comigo |
| **`app.flip_state`** | **1** | **`crate::flip::state::FlipState`** | ⛔ **SHELL — a âncora invisível** |

⇒ **`MotionSceneCtx`**, no molde do [`ph2d_app_physics::SceneCtx`](../../../crates/ph2d-app-physics/src/lib.rs) —
uma struct escrita no vocabulário da família, que a shell preenche. **Nenhum método devolve um
handle**, e o `AppHost` fica nos cinco.

⚠️ **Os chamadores são 7 linhas em `render_loop/mod.rs`** — o LAÇO, que fica na shell por desenho
(HOWTO §4). *O que sai são os CORPOS.*

⛔ **Não o construí nesta jornada, de propósito:** com 97 % da família presa, ele seria um refactor de
signature em 16 ficheiros que ninguém pode compilar no sítio de destino, e o `render_loop/mod.rs` é o
ficheiro mais disputado do repo enquanto três linhas estão abertas. Ele é **barato e mecânico** na
janela que fizer o corte inteiro.

---

## §5 — ⛔ Porque NÃO curei as 4 âncoras do flip/vec por assinatura

Esta foi a decisão da jornada, e tem mecanismo medido — não é preguiça.

O que o `motion_flip_bake.rs` precisa do flip: `FlipEntityMap` (iterar + passar), `build_flip_models`,
`new_engine_armed`, **`art_to_world`**, **`camera_raw`**, **`fold_model`**.

- **Os três primeiros curam-se por assinatura** (passar o resultado / um `&[(id, bits)]` / um `bool`).
- ⛔ **Os três últimos NÃO**: são chamados **dentro do laço por-camada** do `bake`, logo o chamador não
  os pode pré-computar. Sobram duas saídas, e **as duas estão proibidas por escrito**:
  1. **duplicar a matemática** — o `flip_pass_camera.rs` diz de si mesmo que é *«o QUARTO consumidor
     da mesma lei»*. Uma **quinta** cópia é exactamente o defeito que o `stroke_uniform.rs` regista
     (*«uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é»*);
  2. **passar as três funções através da fronteira** (ponteiros de função / trait) — é *inventar uma
     abstracção através da fronteira de outra família*, que o HOWTO §1.2 proíbe e que o meu próprio
     handoff de 11/09 já tinha nomeado como a saída errada.

⭐⭐ **E a saída CERTA está medida: elas são um cluster-folha quase puro.**

| ficheiro | LOC | `crate::` para fora |
|---|---:|---|
| `flip/entities.rs` | 228 | **0** |
| `flip/transform.rs` | 494 | 1 (→ `entities`, o irmão) |
| `render_loop/flip_pass.rs` | 598 | 2 (→ `transform`, o irmão) |
| `render_loop/flip_pass_camera.rs` | 144 | **0** |
| `flip/state.rs` | 115 | — |

⇒ **1 579 LOC fechadas sobre si mesmas** — a forma exacta do `ph2d-vec-entities` que a
`line/shell-folhas` tirou em 12/09. **Mas `shells/desktop/src/flip/` é o conjunto que a
`line/app-flip` está a mover AGORA** (ramo em `27ab89429`, reaberto há minutos), e o
`texture_pattern_live.rs` é o da `line/app-vec` (a commitar durante esta sessão). Mexer ali é o
conflito que a regra 5 e a lei 6 existem para impedir.

---

## §6 — ⭐ O PEDIDO (é isto que destrava 100 %)

**Uma frase para o integrador:** *sequencie a `motion` DEPOIS da `flip` e da `vec`, e diga às duas
que estes 5 ficheiros têm um segundo consumidor.*

Concretamente, e nenhuma delas precisa de fazer trabalho extra — só de **não os deixar na shell**:

| para a `line/app-flip` | `flip/entities.rs` · `flip/transform.rs` · `flip/state.rs` · `render_loop/flip_pass.rs` · `render_loop/flip_pass_camera.rs` |
|---|---|
| **para a `line/app-vec`** | `texture_pattern_live.rs` (e com ele o `brush_live.rs` deixa de me prender) |

⚠️ **Elas podem ir para `ph2d-app-flip` / `ph2d-app-vec` — não precisa de ser crate-folha.** *Uma
crate irmã não é a shell* (ADR-0075, e foi a decisão que destravou a `physics` com a `ph2d-timeline`):
a `ph2d-app-motion` pode depender delas. É isso que já aconteceu com os 16 alias `vec_*` de `main.rs`,
e foi assim que **19 das minhas 26 âncoras de 11/09 dissolveram sem eu tocar em nada**.

⛔ **O que NÃO serve:** deixar um alias `pub(crate) use ph2d_app_flip::… as flip_entities;` **e mais
nada**. O alias resolve a *shell*; a minha crate continua sem poder nomear `crate::flip::…`. O que eu
preciso é que o **símbolo viva numa crate**, com qualquer nome.

---

## §7 — O que uma leitura rápida deste diff entende ao contrário

1. ⚠️ **«a linha não fez nada outra vez»** — ela entregou o **instrumento** que as três linhas da W2
   precisam, e fez a medição que diz **exactamente** o que falta (5 ficheiros nomeados) em vez de
   *«13 folhas partilhadas»*. O bloqueador encolheu **13 módulos → 5 ficheiros**, e de *«uma linha
   nova»* para *«duas linhas já abertas não os deixarem para trás»*.
2. ⚠️ **«246 ficheiros moviam-se»** — esse número é de uma régua **partida** e está neste doc só como
   a lição nº 1. O número é **13**.
3. ⚠️ **«o flip é o bloqueador»** — o cenário **C** mede que curar **só** o flip dá exactamente o
   mesmo `13 / 2 713`. São **as duas** linhas, e a `vec` entra por um caminho que ninguém tinha visto
   (`brush_live` → `texture_pattern_live`).
4. ⚠️ **«o `warp_gizmo` é do render_loop»** — é do **motion**; o `motion_*` do censo não o apanhava.
5. ⚠️ **«2,7 % é pouco mas é alguma coisa, mova-o»** — esses 13 ficheiros são folhas dispersas no meio
   de 406 que ficam. Mover 2 713 LOC agora paga merge em três linhas abertas e **será refeito por
   inteiro**; é a mesma decisão (com o mesmo motivo) que o integrador aceitou em 11/09 para a fatia de
   1,8 %.
6. ⚠️ **«o `crate::App` é o problema»** — não é. Os 16 ficheiros que o nomeiam curam-se por
   **assinatura**, com os 9 tipos resolvidos no §4, e **um só** deles (`app.flip_state`) é da shell.

---

## §8 — Premissas do meu handoff de 11/09 que esta medição DERRUBOU

1. ⛔ **«13 módulos residentes na shell»** → são **7 âncoras**, das quais **4** são de outra gente.
   As `vec_entities` (7 âncoras!), `audio` (3), `modal`, `vec_glyph`, `vec_glyph_build`, `pan_diag`,
   `warp_gizmo`, `warp_gizmo_fixtures` **dissolveram** — umas porque a `line/shell-folhas` as tirou,
   outras porque **eram da minha própria família** e o censo por prefixo não as via.
2. ⛔ **«`audio` é a 2.ª maior alavanca, e não cai com o vec»** — **não aparece no fecho de hoje**.
3. ⛔ **«o resíduo que justifica a linha de folhas é ~13 âncoras sobre ~8 módulos»** — é **4 âncoras
   sobre 6 ficheiros**, e **nenhuma** precisa de linha nova: as duas donas estão abertas.
4. ⚠️ **«`vec_transform` NÃO é bloqueador desta família»** — continua verdade, e agora a `vec` **é**
   bloqueadora por outro ficheiro (`texture_pattern_live`), que aquela medição não procurou.

---

## §9 — A prova

| | |
|---|---|
| baseline `cargo nextest list --workspace --cargo-profile ci-test` | **22 665** linhas (capturada na base, antes de tudo) |
| `cargo check -p ph2d-app-motion` | ✅ verde |
| `cargo test -p ph2d-host-desktop --test it` (regra 2, à parte) | ver §9.1 |
| `python3 scripts/fecho-da-familia.py --autoteste` | ✅ **6/6** controlos |
| ficheiros de produto movidos | **0** |
| `shells/desktop/src` | **363 874** LOC · 1 320 ficheiros — **inalterado** |
| `TETO_LOC` / `PROJECT_SCHEMA` / registos | **não tocados** |
| 6.º método no `AppHost` | **nenhum pedido** |

⚠️ **A prova do `nextest-list-diff` não se aplica** — ela compara duas listas à volta de um movimento,
e não houve movimento. A baseline fica capturada para quem retomar.

**O binário do smoke fica COMPILADO** (regra I):

```
$ cargo build -p ph2d-host-desktop --profile smoke     # 1.ª
    Finished `smoke` profile [optimized] target(s) in 26.56s
$ cargo build -p ph2d-host-desktop --profile smoke     # 2.ª — A PROVA
    Finished `smoke` profile [optimized] target(s) in 0.21s
    (linhas "Compiling": 0)
```

---

## §10 — Para quem retomar (a janela do corte, quando flip e vec aterrarem)

1. **Re-corra o fecho** — `python3 scripts/fecho-da-familia.py motion --extra 'warp_'`. ⛔ Não
   acredite neste doc: as âncoras 5–7 devem ter desaparecido, e se não desapareceram o pedido do §6
   não foi cumprido.
2. **`MotionSceneCtx`** (§4) — as 16 assinaturas, molde `ph2d_app_physics::SceneCtx`.
3. **`thumbnail.rs`** → crate-folha (é puro: `0` e `0`).
4. **`picker_smoke.rs`** e **`field_gizmo.rs`** entram na crate; o `field_gizmo` deixa **alias com
   data de validade** na shell (7 consumidores, um deles do flip).
5. **O corte, em fatias que compilam** — `motion.rs` + os 27 grupos-raiz de `motion/`, depois os 13 de
   `render_loop/motion_*`. ⛔ O laço fica; saem os corpos.
6. **O fim da linha:** conte os roteadores `PH2D_*_SMOKE` da família (⛔ `PH2D_GPU_COOK`, `PH2D_LADO`,
   `PH2D_LAYOUT_LEVEL`, `PH2D_DROPS_SCAN_MAX` são **diagnóstico**), `const FAMILY` com cada
   `max_level` **contado** no `match`, e `"motion"` sai de `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`.
7. ⚠️ **`cargo test -p ph2d-host-desktop --test it` à parte** — o `nextest-impacted` não lá chega.

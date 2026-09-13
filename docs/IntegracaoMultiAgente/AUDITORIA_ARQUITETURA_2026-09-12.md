# Auditoria de ARQUITETURA — 2026-09-12 (fim da W2, antes do envio)

> **Pedido do dono:** *«auditoria sobre perfeição na arquitetura»*. Protocolo `/pd-auditoria`:
> cada achado leva **mecanismo · como reproduzir · o gate que faltava**, e, quando um gate existente
> estava verde sobre ele, **por que** estava verde. ⛔ **Nada foi consertado antes de listar** — este
> documento é a lista; as curas são ondas próprias (§4), e quatro delas são decisão do dono.
>
> Árvore medida: `main` em `145e78a67` (222 commits por enviar). Instrumentos no fim (§5).

## §0 — O placar

| # | achado | sev. | cura | de quem |
|---|---|:-:|---|---|
| A1 | **6 arestas sobem a camada** + **3 laterais família→família**, e o gate de ciclos só vigia a `editor-core` e os painéis | P1 | tipos para folhas por assunto + gate de camadas | integrador |
| A2 | **`#![forbid(unsafe_code)]` não viajou com o código**: 43 crates sem ele, 23 nascidas em setembro — **7 famílias (277 644 L)** saíram de debaixo do `forbid` da shell | P1 | gate de censo + 4 `set_var` viram função pura | integrador |
| A3 | **O marcador `ph2d-loc-cap:` não tem tecto**: `render_loop/mod.rs` foi de 1 667 a **14 009** linhas debaixo dele | P1 | o marcador passa a levar um NÚMERO que só desce | integrador (+ emenda HR-18) |
| A4 | **O shim DEPRECADO `ph2d-editor` tem 2 423 usos**, 962 deles nas famílias da W2; a «deleção» aponta para um ADR que é de outra coisa | P2 | varrimento mecânico + apagar a crate + gate | integrador (agora: nenhuma linha aberta) |
| A5 | **Os ids de widget de cada família vivem na fundação** (`editor-core/src/ids`, 13 960 L): 221 dos 437 commits de 30 dias à `editor-core/src` tocaram lá | P2 | medir antes; o hash já dispensa alocação central | **dono** (desenho) |
| A6 | A shell ainda alcança **13 ferramentas e 19 painéis concretos**; e **4 dependências normais só são usadas em `tests/`** | P2/P3 | `[dev-dependencies]`; o resto por assunto | integrador |
| A7 | O `ph2d-app-registry-init` fora do caminho da composição — **4 dos 5 registos estão nele, este não** | P2 | religar | **dono** (já aberto no ESTADO §6) |
| A8 | **Crates sem consumidor**: `ph2d-system-fonts` promete um consumidor que **nunca existiu** e a `ph2d-app-vec` reimplementou-a; 4 stubs de 4 linhas desde maio; `ph2d-audio-stream` à espera | P2/P3 | censo de alcançabilidade | **dono** (destino de cada) |
| A9 | `App`: **247 campos**, 54 `vec_*`; `impl App` em **131** ficheiros | P3 | medir antes de uma 5.ª rodada | — |
| A10 | `ph2d-editor-core`: **107 862 L de src com 43 dependentes**, +28 % desde 18/08 (nada disso da W2) | P2 | medir o leque de recompilação (§3.7) antes de propor corte | **dono** |
| A11 | **Seis afirmações de docs contra o código** (CLAUDE.md ×3, ESTADO ×2, shim/ADR ×1) | P2 | editar | integrador |
| A12 | 62 pacotes externos com >1 versão | — | já governado (ADR-0168, `stack-audit.sh --tetos`) | — |

## §1 — O grafo, medido

- **361 membros**: 165 núcleo/folha · 135 nós · 29 painéis · 19 ferramentas · 12 `ph2d-app-*` · 1 shell.
- **340 alcançáveis** pela shell por dependências normais; os 21 que não são: 12 binários de
  ferramenta (codegen, cooker, spike…), `ph2d-ui-testkit` (dev), e os de A8.
- **Profundidade da cadeia crítica: 12** —
  `host-desktop → app-motion → app-vec → vec-art-live → panel-vector → tool-vector → editor-core →
  tool-registry → vector → vector-doc → vector-traits → color`. **Duas** das onze arestas são de A1.
- Fan-in: `nodegraph` 141 · `node-registry` 137 · `vector` 50 · `color` 48 · `a11y` 46 ·
  **`editor-core` 43** · `tokens` 41 · `core` 40 · `ecs` 28.
- Maiores unidades (todo `.rs`, testes incluídos): shell 186 791 · **`tool-painter` 136 655** ·
  **`editor-core` 135 687** · `app-motion` 109 009 · `physics-ecs` 71 063.

### O que está BEM (e não precisa de onda)

- **Os quatro contratos congelados têm gate** (`contract_surface`, `tool_contract_surface`,
  `vector_contract_surface`, `imageio_contract_surface`).
- **`panel ↛ panel` e `tool ↛ tool` valem**: as 27 + 30 arestas dessas espécies são todas
  `*-registry-init → concreto` ou `concreto → *-registry`. Os 135 nós só falam com o registo.
- Nenhuma crate depende da shell; o `cargo machete` está limpo; o guarda de ficheiros órfãos está vivo.

## §2 — Os achados

### A1 — Arestas que sobem a camada, e família a chamar família (P1)

**Mecanismo.** O ADR-0075 manda os sistemas falarem por componentes/eventos e *«nunca chamarem umas
às outras diretamente»*; a W2 criou a espécie `ph2d-app-*` e as folhas partilhadas, e nenhum gate
diz **em que direcção** elas podem depender. Contados os usos reais (não comentários):

| aresta | usos em `src` | o símbolo | o que é |
|---|--:|---|---|
| `ph2d-pan-diag → ph2d-app-flip` | 1 | `pass_camera::camera_scene` | uma **sonda** arrasta a família inteira |
| `ph2d-vec-art-live → ph2d-panel-vector` | 5 | `PatternArt` (`state_fill.rs:119`) | um **tipo de domínio nasceu num painel** |
| `ph2d-audio-desktop → ph2d-panel-audio-editor` (opcional) | 26 | `AudioEditCmd` (`lib.rs:379`), `delivery_state` | idem: comando e estado de entrega vivem no painel |
| `ph2d-viewport3d → ph2d-app-host` | 1 | `canvas_area::visible` | folha → substrato |
| `ph2d-viewport3d → ph2d-tool-registry` | 2 | `hash_node_id` | um **hash** mora no contrato de ferramentas |
| `ph2d-editor-core → ph2d-tool-registry` | 13 | `hash_node_id{,_runtime}`, `Zone` | idem |
| `ph2d-app-motion → ph2d-app-vec` | 3 | `glyph::walk_glyphs`, `glyph_build`, `font::resolve` | **lateral** |
| `ph2d-app-motion → ph2d-app-flip` | 6 | `FlipEntityMap`, `FlipState`, `selection_gizmo`, `HALO_RGBA` | **lateral** |
| `ph2d-app-components → ph2d-app-physics` | 4 | `physics_seed::seed_attached_collider`, `inspector::player::*`, `common::spawn_floor` | **lateral** |

**Custo.** Não é a profundidade: sem as nove, a cadeia crítica desce de **12 para 11** (a seguinte
passa por `ph2d-asset → ph2d-imageio-registry-init`). O custo é o **leque**: a maior família
(`app-motion`, 109 k) recompila quando a `vec` ou a `flip` mudam, e a lei do ADR-0075 deixa de ser
verdade entre as famílias — que é onde ela mais importa.

**Reproduzir.** `cargo metadata` → arestas normais entre membros, classificadas por prefixo (§5).

**O gate que faltava.** `architecture_cycle_prevention` só afirma `editor-core ↛ painel/shim`,
`editor-core ↛ ferramenta concreta` e `painel ↛ painel`. **Estava verde porque não tem regra para as
espécies novas** — a W2 inventou `ph2d-app-*` e as folhas depois dele. Gate proposto: uma tabela de
camadas (`folha < contrato < ferramenta < painel < família < composição`) com **as nove arestas de
hoje numa catraca que só encolhe**, metade de obsolescência incluída.

**Cura (molde da W2: «uma folha por ASSUNTO»).** `FlipEntityMap` → folha `ph2d-flip-entities`
(o molde da `ph2d-vec-entities`) · o glifo/fonte → a folha de texto vectorial · `PatternArt`,
`AudioEditCmd`/`delivery_state` → as crates de domínio deles · `hash_node_id` → uma folha de hash ·
os três `seed_*` → a folha de física · `camera_scene` → quem é dono da câmera.

### A2 — O `forbid(unsafe_code)` ficou na shell quando o código saiu (P1)

**Mecanismo.** A DIRETRIZ manda `#![forbid(unsafe_code)]` na **primeira linha** do `lib.rs` de toda
crate nova, e o repo **não tem** `[workspace.lints]` nem gate — a regra vive na memória de quem cria.
Medido por mês de nascimento, com / sem: **maio 82/0 · junho 3/1 · julho 152/6 · agosto 51/13 ·
setembro 18/23**. Entre as 23 de setembro estão **7 das famílias** (`flip`, `motion`, `painter`, `physics`, `sculpt3d`, `skeleton`, `vec` — **277 644 L**) e **as 7 folhas partilhadas** da `line/shell-folhas`. A shell tem o
`forbid`; logo o código que morava nela **perdeu a garantia ao atravessar a fronteira**, sem uma
linha a acusar (a HOWTO §2 já regista que uma *feature* não viaja com o código — um atributo de
crate também não).

**Já mordeu.** `ph2d-app-physics/src/smoke_tests.rs:96` e `ph2d-app-sculpt3d/src/scenes_router_tests.rs:127`
têm `unsafe { std::env::set_var(..) }` — **não compilariam dentro da shell**. São os únicos `unsafe`
reais fora do vendor (`ph2d-audio-ml/vendor/deep_filter`, 12).

**Reproduzir.** Para cada `crates/*/src/lib.rs`, `grep -q 'forbid(unsafe_code)'` (§5).

**O gate que faltava.** Nenhum; `ph2d-audio-encode/tests/the_unsafe_stays_in_its_crate.rs` guarda
**uma** crate. Proposto: censo *«toda crate declara `forbid(unsafe_code)`, salvo a lista nomeada
(`ph2d-gpu`, `ph2d-audio-ml`, `ph2d-audio-encode`, o vendor) com o motivo»*, com a metade de
obsolescência. As duas sondas `set_var` passam a **função pura** (`armed_scene_from(Option<&str>)`),
que testa a mesma coisa sem `unsafe`.

### A3 — Uma isenção de tecto sem número é uma licença (P1)

**Mecanismo.** A HR-18 aceita *«`// ph2d-loc-cap: <razão>` nas primeiras 20 linhas»* e o gate
(`shells/desktop/tests/it/file_loc_caps.rs`) só exige **que a razão exista**. Não há número, logo
não há catraca — a lei §5.0 do CLAUDE.md (*«uma catraca sem censo de obsolescência vira LICENÇA»*)
um nível abaixo:

| ficheiro | 15/06 | 18/08 | 11/09 | hoje |
|---|--:|--:|--:|--:|
| `render_loop/mod.rs` | 1 667 | 9 517 | 14 058 | **14 009** |
| `input_dispatch.rs` | 1 284 | 6 408 | 7 180 | **7 266** |
| `app_state.rs` | 700 | 1 597 | 2 104 | 1 881 |

A W2 **não tocou** nestes (−49 no laço). 14 marcadores no repo; os 7 maiores são da shell.

**O gate que faltava.** O marcador passa a `// ph2d-loc-cap: <número> <razão>`, e o gate afirma as
**duas metades** do `the_shell_only_shrinks`: *cresceu acima do número* e *o número ficou para trás
por mais do que a folga*. ⚠️ É emenda à HR-18 — escrever no mesmo commit.
⛔ Isto **não** é «abstrair o laço» (ESTADO §3 proíbe-o): é impedir que ele volte a crescer 8× em
silêncio. Partir o `mod.rs` por **fase do quadro** (sem mudar a ordem) é a onda seguinte, medida.

### A4 — O shim deprecado multiplicou-se em vez de morrer (P2)

**Mecanismo.** `ph2d-editor` é `pub use ph2d_editor_core::*;` desde o ADR-0029 (maio), *«targeted
for deletion in ADR-0030 (~6 months)»*. **O ADR-0030 é o motor de nós multi-domínio** (21/05) — o
número foi tomado dois dias depois, e a deleção **nunca teve endereço**. Enquanto isso: **2 423 usos**
de `ph2d_editor::` — shell 1 413 · `app-vec` 277 · `app-motion` 272 · `app-flip` 134 · `app-field3d`
70 · `app-painter` 67 · `app-physics` 63 · `app-components` 44 · `app-sculpt3d` 35 · `viewport3d` 18 ·
… e **15 crates** a declarar a dependência, **nenhuma** a declarar as duas.

**Por que o gate estava verde.** `panel_crates_depend_only_on_editor_core` proíbe o shim **só aos
painéis**; as famílias nasceram copiando os `use` da shell, que é o maior utilizador.
Consequência lateral já paga: a régua da W2 conta uma fachada como shell (CLAUDE.md §5 Vector, Fase C).

**Cura.** Mecânica: `ph2d_editor::` → `ph2d_editor_core::` + os 15 `Cargo.toml` + apagar a crate +
alargar o gate a *«nenhuma crate depende de `ph2d-editor`»*. ⚠️ **O momento é agora**: com uma linha
aberta, 2 423 sítios são 2 423 conflitos (lei 6 do ESTADO §4).

### A5 — Os ids de cada família moram na fundação (P2, decisão)

`crates/ph2d-editor-core/src/ids/` tem **13 960 L** em ficheiros **por família**: `chrome/vector.rs`
696 · `inspector.rs` 693 · `chrome/painter.rs` 655 · `chrome/sculpt3d.rs` 632 · `chrome/timeline.rs`
529 · `chrome/flip.rs` 476 · `inspector_player.rs` 393 · `inspector_joint.rs` 339 · … Em 30 dias,
**221 dos 437** commits à `editor-core/src` tocaram `ids/` (`feat(vector)` 25 · `feat(components)` 23 ·
`feat(sculpt3d)` 17 · `feat(physics)` 12…). Cada um recompila as **43** crates que dependem da
fundação, e é o ficheiro partilhado onde as linhas do Modo L se encontram.

**A cerca de Chesterton** (cabeçalho de `ids/mod.rs`): os ids passaram a **hash de slug**
(FNV-1a `const fn`) justamente para acabar com a alocação por faixas, e as colisões são apanhadas
por `tests/it/node_id_collisions.rs`, que **enumera todas as consts**. ⇒ o hash **já dispensa** a
alocação central; o que precisa de ver todas é **só o censo de colisões**, e esse pode morar numa crate
que dependa de todas (o molde dos `*-registry-init`). ⏳ **Medir antes:** quantos ids de cada ficheiro
são lidos por **uma** família só — esses são os candidatos. Não proposto sem essa tabela.

### A6 — A shell alcança crates concretas (P2) e declara dependências de teste como normais (P3)

Usos em `shells/desktop/src`: `tool-vector` **147** (35 ficheiros: `DrawMode`, `TextAlign`,
`VectorDrawConfig`, `params::*`) · `panel-vector` 70 · `tool-painter` 34 · `panel-audio-mixer` 31 ·
`panel-inspector` 28 · `tool-runtime` 27 · `tool-flip` 17 · … e uma cauda de **1–4 usos**
(`tool-trim-transparency`, `tool-real-size`, `tool-make-square`, `panel-wet-tuning`, `panel-padding`,
`panel-bgremoval`, `tool-rasterize`, `panel-upscale`, `panel-physics`, `panel-hierarchy`) — a cauda é
barata e cada uma é uma aresta a menos no `Cargo.toml` onde as rodadas colidem.

**Dependência normal usada só em testes** (0 usos em `src`, 3–5 em `tests/`): `ph2d-panel-sculpt3d`,
`ph2d-node-motion-trail`, `ph2d-node-motion-strobe`, `ph2d-node-fx-drop-shadow`.
**Por que o `machete` estava verde:** ele conta `tests/` como uso. Sem custo de compilação marginal
(chegam pelos `*-registry-init`) — é a declaração que mente.

### A7 — O registo das famílias fora da composição (P2, decisão — já no ESTADO §6)

A shell depende dos registos de **ferramentas, painéis, nós e imageio**, e **não** do das famílias
(depende das 9 directamente). O padrão está 4 de 5. Nada novo além desta simetria.

### A8 — Membros que nada consome (P2/P3, decisão)

- **`ph2d-system-fonts`** (224 L): o cabeçalho diz *«The shell instantiates one and hands it to
  `resolve_glyph_font`»*. `git log -S ph2d-system-fonts -- '*/Cargo.toml'` devolve **só o commit que a
  criou** (`48a28f9d2`): **nunca houve consumidor**. Entretanto `ph2d-app-vec/src/font.rs` enumera as
  fontes do sistema **com o `fontique` directo** — duas implementações da mesma responsabilidade, e a
  documentada é a órfã.
- **`ph2d-net`, `ph2d-save`, `ph2d-telemetry`, `ph2d-physics-soft`**: 4 linhas cada, intocadas desde
  `412be19e8` (09/05), declaradas *«⏳ stub (M13+)»* no SKILL. Promessa de roteiro, não código.
- **`ph2d-audio-stream`** (517 L, ADR-0118): completa, sem consumidor, e o §5 Áudio já a regista como
  cerca *«à espera de um consumidor real»*.
- `ph2d-app-registry-init`: A7.

**O gate que faltava.** O `machete` vê dependências mortas, não **crates** mortas; o guarda de órfãos
vê ficheiros, não membros. Proposto: *«toda biblioteca da workspace é alcançável por um binário de
produto, ou está numa lista com motivo»*, com a metade de obsolescência.

### A9 — A `App` (P3, medir antes)

**247 campos**; os prefixos: `vec` 54 · `last` 14 · `ui` 9 · `timeline` 8 · `pending` 7 · `painter` 7.
As famílias no padrão de chegada agrupam o estado num tipo da crate (`VecState`,
`Sculpt3dShellState`, `BakeChannels`, `MasterEcho`); a `vec` tem o `vec_state` **e mais 53 campos
soltos** ao lado. `impl App` em **131 ficheiros** (136 blocos). Os ficheiros `vec_*` do topo de `src/`
somam 12 133 L (5 636 de produto, 6 358 de testes). Não é defeito por si — é o tamanho medido do que
uma 5.ª rodada teria de responder primeiro.

### A10 — A fundação é o maior leque do repo (P2, medir antes)

`ph2d-editor-core/src` = **107 862 L** (`widget/` 28 485 · `screens/` 26 419 · `interaction/` 18 308 ·
`ids/` 13 960 · `gizmo/` 4 698), **43 dependentes**, e 108 ficheiros de teste, 24 deles `architecture_*` —
vários leem a workspace inteira, e a W1b da auditoria de velocidade (`ph2d-arch-gates`) é quem os tira dali.
Cresceu **84 015 → 107 862 (+28 %) entre 18/08 e 11/09**, e **+123** durante a W2: *a W2 não despejou
na fundação*. ⏳ Antes de propor corte, medir o leque com o método do §3.7 do ESTADO (uma linha
editada em `editor-core/src`, `cargo test --no-run --workspace --profile ci-test --timings`).

### A11 — Docs que discordam do código (P2)

| onde | diz | medido |
|---|---|---|
| `CLAUDE.md:92` | a shell custa *«34–45 s sozinha no portão de fecho»* | 5,9 s (`check`) · 11,1 s (`test`) — ESTADO §3.2/§3.7 |
| `CLAUDE.md:94` | *«A W2 (11/09) tirou de lá 61 704 linhas»* | 526 809 → 186 647 (−340 162) |
| `CLAUDE.md:1268` | `editor-core` *«84.015 — … e 53 gates de arquitetura»* | 107 862 L de src · 24 `architecture_*` (108 ficheiros de teste) |
| ESTADO §2 (l. 130) × §6 (l. 329) | *«SETE das nove famílias»* × *«cinco das sete»* | a §2 é a medida; a §6 envelheceu |
| ESTADO §2 (tabela) × §6 | `render_loop/` 105 f / 44 782 L × 141 f / 55 952 L | reconciliar com UMA régua |
| `ph2d-editor/src/lib.rs` + ADR-0029 l. 34 | deleção *«em ADR-0030»* | o ADR-0030 é o motor de nós |
| `ph2d-system-fonts/src/lib.rs` | *«The shell instantiates one»* | nunca instanciou (A8) |

## §3 — O que esta auditoria NÃO mediu

- A lei do ADR-0075 **dentro** de uma crate (sistemas que se chamam) — um censo textual não a separa.
- O custo em segundos de A1/A5/A10 — só a forma estrutural; o método de medição está nomeado.
- macOS e Windows (só a CI).

## §4 — As ondas propostas, por ordem de custo

1. **Mecânica, integrador, sem linha aberta** — A2 (gate + 2 funções puras) · A6 (4 `dev-dependencies`
   e a cauda de 1–4 usos) · A11 (os sete textos) · A4 (varrimento do shim + apagar + gate).
2. **Desenho pequeno** — A1 (as nove arestas por assunto + o gate de camadas) · A3 (o número no marcador
   + emenda HR-18).
3. **Decisão do dono** — A7 (religar o registo) · A5 (ids para as famílias, depois da tabela) · A8 (o
   destino de cada crate sem consumidor) · A10 (medir o leque antes de qualquer corte).

## §5 — Instrumentos (re-correr para reconferir)

- Grafo: `cargo metadata --format-version 1` → arestas normais entre `workspace_members`,
  classificadas pelo prefixo (`ph2d-app-`/`tool-`/`panel-`/`node-`/resto); alcançabilidade a partir de
  `shells/*`; profundidade por DFS com memo.
- Usos reais de uma aresta: `git grep -h -E '\b<crate_snake>\b' -- crates/<a>/src | grep -v '^\s*//'`.
  ⚠️ **Corra-o em `bash`**: em zsh `set -- $par` não parte a variável e a sonda devolve **zero** sobre
  arestas vivas (mordeu nesta auditoria, é a 4.ª ocorrência da memória).
- `forbid`: `grep -q 'forbid(unsafe_code)' crates/*/src/lib.rs` + `git log --diff-filter=A` do `Cargo.toml`.
- Crescimento de ficheiro: `git show $(git rev-list -1 --before=<data> main):<ficheiro> | wc -l`.
- Consumidor de sempre: `git log --oneline -S <crate> -- '*/Cargo.toml'`.

## §6 — O que a jornada FEZ com esta lista (12/09, a seguir)

O dono respondeu à lista com *«eu não decidirei nada — vc sabe mais que eu; qual o padrão ouro? Vamos
até o estado da arte»*. As curas foram escolhidas pelo padrão-ouro e aplicadas na mesma jornada, uma
frente por commit, cada uma com fmt, check da workspace sem aviso, machete e a suíte das crates que
tocou (14 commits depois desta auditoria).

| # | estado | o que ficou | commit |
|---|:-:|---|---|
| A1 | ✅ | **7** arestas curadas por ASSUNTO — a 7.ª (`app-skeleton → app-vec`, em dev) só o gate novo a viu. Três espécies de cura: um TIPO desce para a folha do domínio (`PatternArt` → `ph2d-vec-pattern`; `FlipEntityMap` → `ph2d-flip-entities`), uma LEI desce para o motor (a câmera → `ph2d-flip-render`; o texto → `ph2d-vec-text`; abrir áudio → `ph2d-audio-decode`), uma TABELA é injectada pela composição (as sementes de anexar). O áudio passou a ser a família `ph2d-app-audio`; o conteúdo da cena do osso a folha `ph2d-skeleton-demo`. Gate `architecture_no_dependency_climbs_a_layer` com a catraca **vazia** | `4a1005876` · `0fe02ccab` · `53966e092` · `e5abd0265` · `cbcd1faa0` · `cecefdc44` · `8c2e4de71` |
| A2 | ✅ | `[workspace.lints.rust] unsafe_code = "forbid"`, 358 membros herdam, 2 excepções nomeadas (FFI) com o `allow` confinado aos módulos; 5 `unsafe` curados sem `unsafe` (porta injectável, `OnceLock`, relançamento, derive). Gate `architecture_every_member_inherits_the_workspace_lints`, 3 mutações | `69da236cc` |
| A3 | ✅ | o marcador sem número morreu; os tectos da shell e da workspace são NUMERADOS e têm a metade *«o tecto ficou para trás»* | `4c33194ec` |
| A4 | ✅ | shim `ph2d-editor` apagado, 2 454 usos passam ao nome real (7 menções históricas ficaram de propósito) | `d94e4155c` |
| A5 | ✅ | 138 ids sem citação saíram (102 de um painel que nunca existiu). **A5b** (`line/editor-core`, integrada 13/09): **1 737** ids desceram para as 21 crates que os lêem e o censo de colisões passou a DERIVADO da workspace; na integração desceram os **207** que a cerca prendia, morreram a fachada `screens::hero::ids` e as três cópias de slug repetido (catraca VAZIA), e nasceu o censo «um nome de id, um valor». Fundação: `ids/` 13 683 → **3 391** linhas, `src/` 107 585 → **97 370** | `13fafa554` · `7f15f3e54` · `d3b9ecdc0` |
| A6 | ✅ | as 4 dependências de teste passam a `[dev-dependencies]` | `9274fd3c7` |
| A7 | ✅ | a promessa impossível sai do cabeçalho do registo; gate `the_shell_links_exactly_the_registered_families` (as duas listas concordam), provado por mutação | `8c2e4de71` |
| A8 | ✅ | 4 stubs apagados; `ph2d-system-fonts` ganha consumidor (a `library` de fontes que a `ph2d-app-vec` reimplementava) e o doc diz que o fallback de glifo continua sem chamador; `ph2d-audio-stream` fica (ADR-0118, consumidor por nascer, já registado no §5 Áudio) | `9274fd3c7` · `53966e092` |
| A9 | ✅ | (`line/render-loop`, integrada 13/09) os campos `vec_*` num `app.vec` (`VecState`), os tipos de assunto da família descidos para a `ph2d-app-vec`, o estado do esqueleto num `SkeletonState`, e a `History` do vetor apagada (escrita, nunca lida). `App` 245 → **187** campos, com a catraca `the_app_only_sheds_fields` | `6c058a7d1` |
| A10 | 🟡 | os módulos de topo formam um DAG com catraca de **7** arestas (`architecture_the_foundation_modules_form_a_dag`). ⛔ Partir a fundação foi MEDIDO e fica por fazer de propósito: nem cada módulo numa crate poupa mais de 3,8 % do CPU por commit, e o único corte que compensa (`screens`) pede curar `action_bus → screens`, que move ~40 tipos do Inspector nomeados 2 539 vezes em 208 ficheiros | `e4399fcb5` |
| A11 | ✅ | CLAUDE.md, ESTADO da W2 e HOWTO (§2.19 o atributo não viaja · §2.20 família não chama família) dizem o que o código diz | `7239fc7e9` |
| — | ✅ | achado durante as curas: a feature `panel-vector` da `ph2d-app-vec` era precisão falsa (20 `cfg` sobre uma dependência obrigatória, e um `let _ = (…)` a calar o compilador) — só se via compilando a crate SOZINHA | `044ff1e16` |
| quadro | ✅ | `run_render_frame` 13 685 → **984** linhas: um índice de 125 fases (`render_loop/fase_*.rs`) chamadas pela mesma ordem, provado por movimento verbatim; o tecto da shell subiu por ordem do dono (190 629 → 196 990) | `642d91da7` · `1fd4a4bde` |
| âmbar | ✅ | o realce do editor tem UMA porta (`ph2d_editor_core::editor_highlight`) e um gate de censo | `d3b9ecdc0` |

### Correcções a esta auditoria (o que a lista acima tinha errado)

- **A1 listava 9 arestas e só 6 subiam.** `ph2d-viewport3d → ph2d-app-host` aponta para o SUBSTRATO e
  `ph2d-editor-core`/`ph2d-viewport3d → ph2d-tool-registry` para o CONTRATO — as duas direcções estão
  certas, e o gate de camadas classifica-as assim.
- **A1 não viu a 7.ª aresta** (`ph2d-app-skeleton → ph2d-app-vec` em `[dev-dependencies]`): a medição
  olhou só `[dependencies]`. Quem a achou foi o gate, na primeira corrida.
- **A auditoria de fecho da `line/editor-core` acusou o seletor de cor de «nunca vir à frente»** (§9 achado 11), e o
  integrador passou-o ao dono como defeito visível. Medido na integração de 13/09: o seletor é pintado FORA da ordem das
  janelas, depois de todo painel, e sempre veio à frente — o `NodeId(380)` era uma entrada fantasma na lista. A cura ficou
  pelo defeito latente (um nome de id, um valor), não pelo ecrã.
- **A2 dizia «as nove famílias e as sete folhas» entre as 23 de setembro** — eram 7 famílias e as 7
  folhas partilhadas (corrigido no próprio §2 antes do commit).

### O que fica, MEDIDO, depois da integração de 13/09

1. ✅ **`input_dispatch.rs`** (7 117 L; `on_mouse_input` 3 102 numa função) **partiu-se** — `line/input-dispatch`,
   integrada em 13/09 com a `render-bodies` e a `loc-caps` ([ESTADO W2 §6](ESTADO_W2_2026-09-12.md)): o índice tem
   527 L e nenhuma função do território passa de 200. O molde foi o da `line/render-loop`, e a régua da prova de
   movimento ficou versionada em [`scripts/moved-proof.py`](../../scripts/moved-proof.py).
2. **A10:** a catraca tem 7 arestas, e o corte da fundação só compensa para o `screens` — não agora
   (acima).
3. ⚠️ **Os gates que leem a SHELL pelo caminho moram também fora dela**: três de família reprovaram na
   ponta da `line/render-loop` sem ninguém ver. Quem partir o `input_dispatch` corre
   `git grep -n 'shells/desktop/src' -- crates tools` antes de fechar. ✅ Corrido pelas linhas de 13/09 (a
   `render-bodies` mediu 10 de 10 desses gates verdes na ponta dela), e a árvore combinada passou o `ship.sh` inteiro.

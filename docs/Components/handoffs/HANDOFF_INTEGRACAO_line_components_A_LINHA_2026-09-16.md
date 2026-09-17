# HANDOFF DE INTEGRAÇÃO — **A LINHA `line/components` INTEIRA**

> **Para o agente INTEGRADOR.** Este é o documento da LINHA; os sete handoffs de wave (§3) ficam
> como o *porquê* de cada pedaço. ⛔ **A linha não integra e não pusha** (CLAUDE.md §0.7): ela
> fecha, entrega isto e pára.
>
> **Ordem das linhas, árvore suja e o momento do ship são do INTEGRADOR, não desta linha.** O que
> está aqui são **factos medidos**, com a data e o comando que os produziu.

| | |
|---|---|
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components` |
| **Ramo** | `line/components` · ponta `4f5280c4b` |
| **Merge-base** | `1d43da737` — ⭐ **é a ponta do `main` e do `origin/main` neste momento** (o `main` andou **0** commits desde o fork) |
| **Tamanho** | **67 commits** · **575 ficheiros** · `+54 221 / −1 982` |
| **Árvore** | **limpa** (`git status --porcelain` vazio) |
| **Data do fecho** | 2026-09-16 |

⚠️ **Enquanto o `main` estiver em `1d43da737`, esta linha é um `--ff-only` puro.** Se outra linha
aterrar primeiro, **NADA aqui deixa de ser verdade menos os CONTADORES** — e a §4 diz exactamente
quais e como os recontar.

---

## §1 — O que a linha entrega

O **TOP-20 anda do #9 ao #18**: sete itens, cada um com lei pura, ponte, painel, cena de smoke
**aprovada pelo dono** e provas de mutação.

| # | o que o artista ganha | smoke |
|---|---|---|
| **#9** | **Tags** — um objecto PERTENCE a tags numa árvore (modelo do Blender); um sinal fala com a tag | `PH2D_TAGS_SMOKE=1\|2` |
| **#11/#12** | **Factory** + **Lifetime/DestroyOutside** — uma receita nasce sozinha ao sinal, e a cópia morre por tempo ou por sair do ecrã | `PH2D_FACTORY_SMOKE=1\|2` |
| **#13** | **TopDownPlayer** — o 2.º controlador canónico, com isometria e deslize à velocidade CHEIA | `PH2D_TOPDOWN_SMOKE=1\|2` |
| **#14** | **ProjectileMotion** — o mover arcade: avança, gravidade, ricochete, alcance, perseguição | `PH2D_PROJECTILE_SMOKE=1\|2` |
| **#15** | **StateMachine** — estados com nome e setas `(de, ao ouvir, para)`; cada estado anuncia-se | `PH2D_STATEMACHINE_SMOKE=1` |
| **#16** | **ScriptProperties** — um `.luau` declara os números dele e cada objecto tem os seus | `PH2D_SCRIPT_SMOKE=1` |
| **#18** | **ParticleEmitter** — fogo, fumo, faíscas e explosões sem grafo nenhum | `PH2D_PARTICLES_SMOKE=1\|2` |

⛔ **O #17 (Tilemap) NÃO está aqui, e é decisão do dono** (16/09): ele pertence ao projecto dele em
`docs/Tilling/`, que está no `.gitignore`. *A ausência é o produto de uma decisão, não um buraco.*

---

## §2 — ⛔⛔ SUPERFÍCIE DE COLISÃO, **medida** (2026-09-16)

Produzida por `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` **de dentro
desta worktree, com o caminho absoluto do primário** (uma worktree forkada antes do script não o
tem, e ele mede a árvore de onde foi CHAMADO).

### §2.1 — Schemas

| grandeza | aqui | base | delta |
|---|---|---|---|
| `PROJECT_SCHEMA` | **135** | 128 | **+7** |
| tripla do gate | **(135, 13, 22)** | (128, 13, 22) | — |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` · `FIELD_DOC_VERSION` | 22 · 13 · 18 · 22 | iguais | **0** |

⚠️⚠️ **O `PROJECT_SCHEMA` mora em TRÊS sítios e a escada vive num ficheiro IRMÃO** — um degrau
escrito no ficheiro errado funde **limpo** e evapora:
`shells/desktop/src/project_schema.rs` (o `const`, linha ~485) · a **escada de migração** ao lado ·
a **tripla** em `shells/desktop/src/project_schema_tests.rs`.

⭐ **Os sete degraus têm migração real** (o `SignalActions` do v128 nasce `Named`, etc.) — ⛔ não é
um bump vazio, logo **renumerar é reescrever a escada**, não trocar um literal.

### §2.2 — Registo de componentes: **TRÊS contadores, e cada um só corre na suíte da própria crate**

| contador | aqui | base | delta | ficheiro |
|---|---|---|---|---|
| `ph2d-ecs` | **91** | 85 | **+6** | `crates/ph2d-ecs/src/scene/registry{,_tests}.rs` |
| `ph2d-render` (espelho) | **92** | 86 | **+6** | `crates/ph2d-render/src/registry.rs` |
| `ph2d-script` (espelho) | **92** | 86 | **+6** | `crates/ph2d-script/src/registry.rs` |

⛔⛔ **A cegueira medida QUATRO vezes neste repo:** *um portão que só corre as crates que a linha
EDITOU é cego a todo espelho.* Os espelhos contam `ecs + …`, logo **mexem-se sem esta linha lhes
tocar**. ⇒ depois de qualquer renumeração, corra as **três** suítes.

### §2.3 — Contrato congelado (§6) e ADR

- `crates/ph2d-nodegraph/src/node.rs` — **intocado** · `crates/ph2d-editor-core/src/tool.rs` — **intocado**.
- **Esta linha NÃO cria ADR nenhum** ⇒ fora de toda disputa de número (último no disco: `0169`).

### §2.4 — `Cargo.lock`: **uma família externa nova**

`icu_casemap 2.3.0` + `icu_casemap_data` — puxados por `crates/ph2d-label-fold` (a dobra ICU das
tags). ⭐ **Licença `Unicode-3.0`, já permitida** no `deny.toml` (linha 38) ⇒ `cargo deny` passa
**sem alterar o `deny.toml`**, que esta linha **não toca**.

Os outros oito `+name` são **crates internas** (§2.5). ⚠️ O `Cargo.toml` da raiz **não muda**: os
membros entram por glob.

### §2.5 — Crates NOVAS (8)

`ph2d-label-fold` · `ph2d-label-path` · `ph2d-tags` · `ph2d-panel-tags` (painel docado novo) ·
`ph2d-topdown` · `ph2d-projectile` · `ph2d-sweep` · `ph2d-particles`.

⚠️ **O `ph2d-panel-tags` é um PAINEL novo** — ele declara `Panel::ICON` (obrigatório desde a
`line/UIUX`) e o glifo dele é próprio (`docs/design/icons/`). Nasce **fechado**
(`DEFAULT_VISIBLE = false`).

### §2.6 — Tectos de LOC nos ficheiros tocados

O mapa acusa **dois ✗**, e os **dois já estavam acima no `main`** — esta linha **ENCOLHE** ambos:

| ficheiro | aqui | no `main` |
|---|---|---|
| `shells/desktop/src/app_state.rs` | **998** | 1019 |
| `shells/desktop/src/main.rs` | **1103** | 1118 |

⚠️ **Catraca da shell:** `159 465` linhas contra o tecto de `196 990` (no `main`: `159 260`) ⇒
**+205**, com `37 525` de folga. ⛔ Nenhum tecto de LOC foi SUBIDO nesta linha: os quatro que
ficaram vermelhos durante a construção foram curados por **corte por responsabilidade**.

### §2.7 — Catracas que esta linha MOVEU

| catraca | aqui | no `main` | sentido |
|---|---|---|---|
| `TETO_CAMPOS` (campos da `App`) | **180** | 187 | **desceu** ✓ |
| arestas toleradas do DAG da fundação | **24** | 24 | parada (sem a cura das tags teria ido a **26**) |
| `LIVE_SECTIONS` | **28** | 27 | +1 |

---

## §3 — Os sete handoffs de wave (o *porquê* de cada pedaço)

| wave | handoff |
|---|---|
| #9 Tags | [`…_TAGS_2026-09-14.md`](HANDOFF_INTEGRACAO_line_components_TAGS_2026-09-14.md) |
| #11/#12 Fábrica | [`…_FABRICA_2026-09-14.md`](HANDOFF_INTEGRACAO_line_components_FABRICA_2026-09-14.md) |
| #13 TopDown | [`…_TOPDOWN_2026-09-15.md`](HANDOFF_INTEGRACAO_line_components_TOPDOWN_2026-09-15.md) |
| #14 Projéctil | [`…_PROJECTILE_2026-09-15.md`](HANDOFF_INTEGRACAO_line_components_PROJECTILE_2026-09-15.md) |
| #15 Máquina de estados | [`…_STATEMACHINE_2026-09-15.md`](HANDOFF_INTEGRACAO_line_components_STATEMACHINE_2026-09-15.md) |
| #16 Script | [`…_SCRIPT_2026-09-16.md`](HANDOFF_INTEGRACAO_line_components_SCRIPT_2026-09-16.md) |
| #18 Partículas | [`…_PARTICLES_2026-09-16.md`](HANDOFF_INTEGRACAO_line_components_PARTICLES_2026-09-16.md) |

Cada um traz: os contadores como delta, as premissas que a medição derrubou, **as leituras que o
diff inverte**, e os itens abertos com o mecanismo.

---

## §4 — ⭐⭐⭐ Se outra linha aterrar PRIMEIRO: a receita do recontar

⛔⛔ **A coluna `base:` do `collision-surface.sh` é o MERGE-BASE, e a partir da 2.ª fusão de uma
rodada ela está desactualizada POR CONSTRUÇÃO** — em 10/09 ela dizia `PROJECT_SCHEMA 124 (base:
123)` com o `main` já em `127`, e quem leu *«+1»* landou um **retrocesso de 4 degraus**.
⇒ **leia o valor do `main` NO FICHEIRO, nunca na coluna.**

**Para cada grandeza da §2.1/§2.2, o valor certo é `valor_do_main_de_agora + delta desta linha`** —
e o delta é a coluna «delta», nunca o literal:

```
# o main de agora, lido no ficheiro:
git show main:shells/desktop/src/project_schema.rs | grep 'const PROJECT_SCHEMA'
git show main:crates/ph2d-ecs/src/scene/registry.rs | grep -E 'reg\.len\(\), *[0-9]+'
git show main:crates/ph2d-render/src/registry.rs   | grep -E 'reg\.len\(\), *[0-9]+'
git show main:crates/ph2d-script/src/registry.rs   | grep -E 'reg\.len\(\), *[0-9]+'
```

⚠️ **Renumerar o `PROJECT_SCHEMA` mexe em TRÊS sítios** (o `const`, a escada, a tripla) e, por
serem **sete degraus com migração**, a escada tem de ficar **contígua e na ordem** — um degrau
duplicado lê bytes velhos **em silêncio**.

⚠️ **E a colisão passa MUDA quando as duas linhas escrevem o MESMO literal:** o git não sabe o que
o número significa (CLAUDE.md §5.0).

---

## §5 — Onde um merge TEXTUAL pode colidir (ficheiros partilhados, append-only)

Estes são os ficheiros que esta linha toca **fora da família**, e a regra de cada um:

| ficheiro | o que esta linha lhe faz | a regra |
|---|---|---|
| `crates/ph2d-editor-core/src/action_bus.rs` | **+7 variantes** de `EditorAction` | **append-only**; ⚠️ o ficheiro está a **687/700** — quem acrescentar a seguir orça o corte |
| `crates/ph2d-editor-core/src/ids/live_sections.rs` | `LIVE_SECTIONS` **27 → 28** (array dimensionado) | o tamanho do array é **erro de compilação** se esquecido |
| `crates/ph2d-runtime/src/lib.rs` | `SignalOrigin` **+2** (`Factory`… , `Particles`) | append-only |
| `crates/ph2d-ecs/src/lib.rs` · `-render/src/registry.rs` · `-script/src/registry.rs` | os três contadores | §2.2 |
| `crates/ph2d-component-desc/src/catalog/mod.rs` | famílias novas do catálogo | ⚠️ **ordenado por `canonical_name`** (gate `the_catalog_is_sorted_and_unique`) |
| `crates/ph2d-i18n/src/{lib,tags,factory}.rs` | tabelas de strings novas | append-only, por tabela |
| `crates/ph2d-ui-testkit/src/sowing.rs` | **+1 porta** (`set_widget_color`) | append-only; ela é do TESTKIT, logo **toda linha** pode tocá-la |
| `crates/ph2d-editor-core/tests/it/hr12_widgets_a11y.rs` | **+1 entrada** no `PANEL_A11Y_DELEGATE_OK` | ⚠️ é uma **lista de isenções com censo de obsolescência** — duas linhas a acrescentar colidem no mesmo bloco |
| `crates/ph2d-editor-core/tests/it/hr15_no_hardcoded_ui_strings.rs` · `…_the_foundation_modules_form_a_dag.rs` | catracas | ⛔ **só descem**; a do DAG ficou parada em 24 |
| `crates/ph2d-editor-core/tests/it/main.rs` | `mod` do gate novo | append-only |

---

## §6 — Prova de fecho (o que correu, com o número)

Tudo corrido **nesta worktree, com a árvore limpa**, em 2026-09-16:

| portão | comando | resultado |
|---|---|---|
| varredura impactada | `bash scripts/nextest-impacted.sh` | ✅ **15 475 / 15 475** (10 066 saltados) |
| clippy | `cargo clippy -p … --all-targets -- -D warnings` | ✅ **0** (crates tocadas + shell) |
| formato | `cargo fmt --all` | ✅ limpo |
| índices de doc | `bash scripts/doc-index.sh --check` | ✅ 19 índices em dia |
| provas de mutação | `docs/Components/ferramentas/mutacao_*.sh` (9 ficheiros) | ✅ **todas a sangrar** (a última wave: 25/25) |
| smoke do dono | as 7 cenas | ✅ **aprovadas uma a uma** |

⚠️ **O que esta linha NÃO correu, e é do integrador:** `./scripts/ship.sh` (paridade com o CI:
machete, deny, audit, typos, a crate opcional que compila sozinha) e o
`scripts/foundational-integrate.sh` sobre a **árvore combinada** — é lá que aparecem os vermelhos
que só a combinação pode ter.

⛔⛔ **E há uma classe que só a árvore combinada reprova:** os **tectos de LOC somam entre linhas
sem ninguém a contar** (medido 10/09: `screens/hero.rs` chegou a `709/700` somando três linhas que
fecharam verdes no mesmo dia). ⇒ **a cura é do integrador e é corte**, nunca uma entrada nova no
`FILE_OVERAGE_OK`. Os dois candidatos desta linha estão nomeados na §2.6, e os dois **encolheram**.

---

## §7 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **`+54 221` linhas não são código novo dessa ordem:** ~42 ficheiros são docs (7 handoffs, 6
   planos, um tutorial em PDF e as ferramentas de oráculo/mutação versionadas).
2. **Os oito crates novos são FOLHAS** — `ph2d-sweep` tem **zero** dependências, `ph2d-topdown` só
   `libm`. Nenhuma delas depende da shell.
3. **`ph2d-sweep` não é duplicação:** ele é o orçamento de movimento que o #13 e o #14 partilham
   **letra por letra** — só a re-emissão difere (tangente contra espelho).
4. **O `Spawned` do #11 NÃO se regista de propósito**, e essa ausência **é a lei**: o que nasce numa
   corrida não é documento (não entra no ficheiro, não entra no `Ctrl+Z`, some ao rebobinar).
5. **`rewind_runtime` é foundational novo** (`ph2d-ecs`) e serve quatro componentes — ⛔ não é do
   #15, é o substrato que ele teve de pagar primeiro.
6. **A migração de vocabulário do Inspector COMEÇOU** (`tags_edits`, `factory_edits`,
   `particles_edits`…): é a cura que a tolerância do DAG prescreve por escrito, e é o que impede a
   aresta de subir.

---

## §8 — ⏳ Aberto (não bloqueia a integração)

Cada item tem o mecanismo no handoff da wave respectiva. Os que atravessam:

1. **O caminho de GPU do emissor** (#18) — bloqueador nomeado: o renderer recebe **um** buffer, e o
   `gpu-cook` e o `renderer_draw` estão a ser reescritos por duas linhas vivas.
2. **O WASD de fábrica do #13** — o `W` abre o painel de mundo; **decisão do dono**.
3. **O custo a 1 000 movers** (#13: `263 %` de um quadro) e o **tecto de emissores por quadro**
   (#18) — não varridos.
4. ⚠️ **Um ACHADO numa crate alheia, medido e NÃO curado** (#9 §8): o `ph2d-panel-skeleton` pinta a
   barra dele com o `VECTOR_SCROLLBAR_ID`, e **com o painel de vector fechado o arrasto do polegar
   nunca arma**. É da `line/Vector`/`line/UIUX`, não desta.
5. ⚠️ **Uma flake de carga NOVA para promover** (#14): `glaze_layering_costs_a_ratio_not_an_order_of_magnitude`
   (`ph2d-wet-paint`) — gate de RAZÃO, 3/3 verde sozinho a `load 142–151`, com **zero linhas** do
   diff naquela crate. *Uma flake sem nome não entra numa lista; esta tem nome.*
6. **Nenhuma das 26 secções opcionais do Inspector tem gate a provar que chega a PIXEL** (#15) — a
   presença é gateada, a pintura não, e o arnês não existe naquela crate.

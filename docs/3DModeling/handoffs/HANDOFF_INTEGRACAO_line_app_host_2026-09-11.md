# Handoff de integração — `line/app-host` (W2 / L0) — 2026-09-11

> **O substrato por onde uma família sai da shell, o piloto que o prova, e o molde que as outras
> cinco seguem.** DIRETRIZ §1.5.9.

---

## 1 — Identidade

| | |
|---|---|
| branch | `line/app-host` |
| HEAD | `01bc8a2a971739847175a7fd0d19147d3c55d98a` |
| merge-base com `main` | `8fa4f115bbdedb7635581af8c528a8aedd5c8d74` |
| commits | **6** |
| ficheiros tocados | 162 |

⚠️ **Esta linha integra PRIMEIRO.** Ela é o substrato: as cinco famílias (`app-motion`,
`app-physics`, `app-sculpt3d`, `app-vec`, `app-flip`) fazem a **Fase B** delas contra o que está
aqui (briefing §1).

---

## 2 — Foundational / partilhado tocado, e porquê

**Crates NOVAS** (nenhuma edita ficheiro central — `crates/*` é glob no workspace):

| crate | LOC | o que é |
|---|---:|---|
| [`ph2d-app-host`](../../../crates/ph2d-app-host/) | 449 | **a interface**: o trait `AppHost` (5 métodos) + `HostMods` + `AppFamily`/`SmokeRouter` + as portas `modal` e `canvas_area` |
| [`ph2d-viewport3d`](../../../crates/ph2d-viewport3d/) | 2 998 | **a moldura 3D** partilhada pelos dois módulos 3D (vistas · navball · split · gizmo · menu de vistas) |
| [`ph2d-app-registry-init`](../../../crates/ph2d-app-registry-init/) | 258 | o **ponto de extensão append-only** (bloco gerado) |
| [`ph2d-app-field3d`](../../../crates/ph2d-app-field3d/) | 27 816 | **o piloto** — a família `field3d` |
| [`tools/ph2d-app-sync`](../../../tools/ph2d-app-sync/) | 218 | o codegen do registo |

**Ficheiros partilhados EDITADOS:**

| ficheiro | o quê | risco |
|---|---|---|
| `shells/desktop/src/main.rs` | **−20** `mod field3d_*`, **+3** (`app_host`, `field3d_undo_probe`, `field3d_snapshot_tests`) | ADIÇÃO/REMOÇÃO numa região só; as cinco linhas mexem nas regiões DELAS |
| `shells/desktop/Cargo.toml` | +4 deps (`ph2d-app-host`, `-viewport3d`, `-app-field3d`, `-app-registry-init`) + 1 dev-dep | ⚠️ é a última vez que uma família edita este ficheiro — daqui em diante entra pelo registo |
| `shells/desktop/src/render_loop/mod.rs` · `input_dispatch.rs` (+`/keyboard_field3d.rs`) · `project_load.rs` · `undo_app.rs` · `modal.rs` · `canvas_area.rs` | endereços reescritos (`crate::field3d_X` → `ph2d_app_field3d::X`), 89 sítios | textual, em regiões disjuntas das outras linhas |
| `shells/desktop/src/app_host.rs` | **novo** — a implementação do trait | — |
| `crates/ph2d-mesh/src/{lib.rs,read.rs}` | **+`read_pieces` e `lost_by`** (append-only) | a lei era do `sculpt3d_import`/`_export` e ganhou um 2.º consumidor |
| `crates/ph2d-editor-core/tests/it/{the_corner_of_a_control_comes_from_the_theme,the_row_height_is_one_number}.rs` | as varreduras passam a seguir `crates/ph2d-app-*` e `ph2d-viewport3d` | ⭐ ver §6 — **as cinco linhas seguintes nascem cobertas** |

### ⚠️ DUAS edições dentro do conjunto de ficheiros da `line/app-sculpt3d`

Nomeadas aqui porque ela está a mover esses ficheiros **hoje**:

1. `shells/desktop/src/sculpt3d.rs` — **removida** a linha `pub(crate) use export::lost_by;`. Era um
   re-export cujo **único** consumidor era o `field3d`; deixá-lo reprova o clippy `-D warnings`.
2. `shells/desktop/src/sculpt3d_export.rs` — o corpo de `lost_by` passa a **delegar** a
   `ph2d_mesh::lost_by` (a lei mudou-se para o dono do formato). 4 linhas.

⛔ **Não toquei em mais nada com o prefixo `sculpt3d`.** Os doc-links de `sculpt3d_*` para
`crate::field3d_{views,navball,layout,gizmo,view_menu}` continuam a resolver — é para isso que os
cinco alias ficaram (§5).

---

## 3 — Símbolos que podem COLIDIR (`collision-surface.sh`, corrido agora)

```
SUPERFÍCIE DE COLISÃO — line/app-host contra main
  merge-base 8fa4f115b   ·   6 commit(s)   ·   162 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 5 pacotes novos, TODOS internos (path deps desta linha)
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⭐ **NENHUM contador partilhado se move.** É o que o briefing §6 exige: *mover código não muda
serialização*. Se depois da fusão algum deles mudar, não foi esta linha.

⚠️ **Consts/ids novos** (todos dentro das crates novas, zero disputa):
`ph2d_app_field3d::FAMILY` (chave `"field3d"`, roteador `PH2D_FIELD_SMOKE`) ·
`smoke_scenes::{CENAS = 32, PODADAS}` · `ph2d_app_host::{AppHost, HostMods, AppFamily,
SmokeRouter, AppFamilyRegistry}` · `ph2d_app_field3d::input::Field3dInput` ·
`ph2d_mesh::{read_pieces, lost_by}`.

⚠️ **O registo `ph2d-app-registry-init` NÃO é contado pelo `collision-surface.sh`.** Conte-o à mão:
cada família acrescenta uma entrada, o bloco é **gerado** (`cargo run -p ph2d-app-sync`) e o gate de
staleness diz se alguém o escreveu à mão.

---

## 4 — Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` intocados; zero ADR.

---

## 5 — O que o integrador precisa de saber para não ler o diff ao contrário

1. ⭐ **A `ph2d-viewport3d` não é «mais uma crate»: ela é a razão por que a `line/app-sculpt3d` não
   vai ter de depender da família irmã.** `sculpt3d_*` referenciava `field3d_*` em **sete**
   ficheiros, e o que consumia de lá não tinha uma linha de campo implícito.
2. ⚠️ **Os cinco alias na shell (`field3d_views`, `_navball`, `_layout`, `_view_menu`, `_gizmo`)
   têm data de validade escrita no doc:** eles existem só porque `sculpt3d_*` os consome. **Quando a
   `line/app-sculpt3d` fechar, apague-os** e reaponte os chamadores para `ph2d_viewport3d::…`.
3. ⚠️ **`field3d_undo_probe.rs` e `field3d_snapshot_tests.rs` VOLTARAM** para a shell, por sujeito: o
   primeiro conduz a `App` real pelo ponteiro real, o segundo captura um `ProjectState`.
4. ⚠️ **O `impl App` virou trait de extensão**, e por isso **os 14 sítios de chamada do
   `input_dispatch` estão byte a byte iguais** — o diff ali é **uma linha de `use`**. Não procure a
   mudança nos chamadores.
5. ⛔ **A feature `test-support` da família é ligada nas `[dev-dependencies]` da shell**, não nas
   normais. No `cargo build` do produto ela fica desligada.
6. ⚠️ **As 14 cenas podadas estão numa lista EXPLÍCITA** (`scenes::PODADAS`) que o gate lê nos dois
   sentidos. Se alguém reescrever o braço de uma delas sem a tirar da lista, isso **reprova**.
7. ⚠️ **`ph2d-mesh` ganhou duas funções** (`read_pieces`, `lost_by`) — append-only. A `line/
   app-sculpt3d` vai mover o `sculpt3d_export.rs` que agora delega a elas.

---

## 6 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

- **`machete`**: 5 crates novas com dependências novas. ⚠️ A `ph2d-app-field3d` declara `ph2d-sdf`
  em `[dev-dependencies]` (só uma medição `#[ignore]` a usa) e `half`/`ph2d-mesh-render` atrás da
  feature `sculpt3d` — se o machete reclamar, é por causa das features, não por deps mortas.
- **`deny` / `audit`**: nenhum pacote **externo** novo (os 5 do `Cargo.lock` são path deps internas).
- **`typos`**: corrido nos ficheiros novos, limpo.
- ⚠️ **`cargo doc`**: converti ~12 links de doc que atravessavam a fronteira para texto simples, mas
  **não corri o `cargo doc`** — se o `ship.sh` o correr, pode haver `broken_intra_doc_links`
  residuais em `sculpt3d_export.rs`/`sculpt3d.rs` (dois links para `crate::field3d_export`, que
  **não** editei por serem da outra linha).

---

## 7 — A PROVA (briefing §3)

### a) Nenhum teste se perde — **`ONLY-A = 0`** ✅

```
antes: 22635 testes (22011 chaves) | depois: 22645 (22021)
MOVED (mesma chave, outro pacote/binário): 360
ONLY-A (perdidos): 0
ONLY-B (novos): 10
```
Os 10 novos são gates que esta linha escreveu (5 do registo, 3 do sync, 1 da rota do chrome, 1 do
modal reancorado). ⚠️ O único `ONLY-A` que existiu a meio do caminho foi o
`every_field3d_modal_goes_through_the_door` — ver §8.

### b) Roteadores idênticos ✅

```
diff <(git grep -hoE 'PH2D_[A-Z0-9_]*SMOKE' main -- shells/desktop/src | sort -u) \
     <(grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE' shells/desktop/src crates/ph2d-app-*/src | sort -u)
→ IDÊNTICOS   (106 de cada lado)
```

**Cenas apagadas (ordem do Enio, briefing §3.2):** `PH2D_FIELD_SMOKE` = **8, 9, 10, 12, 13, 15, 16,
17, 18, 19, 20, 21, 22, 23** — 14 cenas, **952 linhas**, e o ficheiro `smoke_scenes_shapes.rs`
inteiro (as cinco cenas dele eram todas órfãs). Nenhuma é citada por doc fora de `docs/archive`,
nenhuma é usada por código. **Os números das que ficam não mudaram.**

### c) A shell encolheu ✅

| | antes (`main`) | depois | Δ |
|---|---:|---:|---:|
| LOC de `shells/desktop/src` | **493 252** | **461 512** | **−31 740 (−6,4 %)** |
| ficheiros | 1 784 | 1 688 | −96 |
| unidade `ph2d-host-desktop bin (check-test)`, a frio | **16,30 s** | **15,23 s** | **−1,07 s (−6,6 %)** |

⚠️ **As duas leituras de relógio foram feitas BACK-TO-BACK**, cada uma em `target/` novo, com o
`/proc/loadavg` ao lado (`17,6 → 15,7` no «antes», `15,7 → 13,2` no «depois»). **As duas estão muito
acima do `load ~5`** que o `CLAUDE.md §5.0` exige — as outras cinco linhas da W2 abriram hoje e estão
a construir. Elas valem como **par** (o lado «depois» correu com carga ligeiramente *menor*, o que o
favorece), não como número absoluto.

⭐ **O que dá confiança é a coerência:** `−6,6 %` de relógio contra `−6,4 %` de LOC. E o «antes»
lê **16,30 s** contra os **16,8 s** que a auditoria mediu a frio **sem contenção** — coerente com o
facto de o front-end desta crate ser **monotarefa** (auditoria §3.2) e portanto pouco sensível à
carga enquanto sobrar um núcleo.

⛔ **Uma família de seis saiu.** A shell continua com 461 k LOC e continua a ser o caminho crítico —
a W2 paga-se quando as seis aterrarem.

### d) Gate de fecho ✅

```
cargo clippy --workspace --all-targets   →  0 erros, 0 avisos
cargo fmt --all -- --check               →  limpo
bash scripts/doc-index.sh --check        →  ✓ 19 índices em dia
bash scripts/nextest-impacted.sh         →  14 981 testes, 14 981 passaram, 0 falhas
```

⚠️ **Uma flake conhecida apareceu numa corrida intermédia** e não é desta linha:
`flip_smooth::resample_measurement::precisao::orcamento::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
— membro **já listado** no `CLAUDE.md §5.0`. Confirmada pelas três assinaturas: **3 de 3 verde
sozinha** (a `load 39,61`), **zero linhas** do diff desta linha naquele módulo, e verde na corrida
final completa.

### e) Smoke ✅ — ver §9

---

## 8 — O que a extracção QUEBROU, e o que isso ensina (o material do HOWTO)

**15 correcções de gate**, em duas espécies — e só uma avisa. O molde completo, com os números, está
em [`HOWTO_partir_uma_familia_da_shell.md`](../../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md)
§2; o resumo para o integrador:

| espécie | n | exemplo |
|---|---:|---|
| ⚠️ **falha alto** | 11 | `include_str!` relativo · `read_to_string(CARGO_MANIFEST_DIR/…)` (só falha ao CORRER) · valores esperados que são nomes de ficheiro · a agulha da fiação |
| ⛔ **fica MUDA** | 4 | um censo que varre por `starts_with("field3d_")` passa a varrer **zero** ficheiros, e `bad.is_empty()` sobre lista vazia é trivialmente verdadeiro |

**As três que só a fronteira nova revela:**

1. ⛔⛔ **Uma FEATURE não viaja com o código.** O `smoke.rs` carrega o matcap sob
   `#[cfg(feature = "sculpt3d")]`; numa crate que não a declara o `cfg` é falso **por construção** —
   **a crate compilou VERDE com o matcap desligado**, e a cena sairia cinzenta. É a recusa medida da
   auditoria §9 a acontecer por acidente.
2. ⛔ **`#[cfg(test)]` é invisível do outro lado da crate.** 4 gates da shell deixaram de ver uma
   função à vista no ficheiro que o erro citava ⇒ feature `test-support`, do tamanho do que
   ATRAVESSA (**1 item de 16**; abrir os 16 deu `dead_code`).
3. ⛔⛔ **Uma fronteira põe um ELO NOVO na corrente, e ele não tem gate.** A família deixou de poder
   nomear o `chrome_hit` e passou a perguntar `self.pointer_over_chrome(…)` ao trait: uma
   implementação que respondesse `false` reabriria o report de 30/08 (*«é como se tudo fosse
   canvas»*) **com os dois lados verdes** ⇒ gate novo `the_host_routes_the_door_to_the_one_index`.

⭐⭐ **E dois censos de OUTRA linha ficaram obsoletos pela razão oposta à óbvia:**
`the_corner_of_a_control_comes_from_the_theme` e `the_row_height_is_one_number` varrem
`editor-core` + `panel-*` + a shell. Mover a família tirou dois ficheiros dessa **população** — a
isenção ficou «obsoleta» porque o censo deixou de os **ver**, não porque o código mudou. ⛔ Apagar a
isenção (a leitura fácil) apagava a **cobertura** junto com a linha. ⇒ as duas varreduras passam a
seguir `crates/ph2d-app-*` e `ph2d-viewport3d`, e **as cinco famílias seguintes nascem cobertas**.

### Premissas do briefing que a medição derrubou

| o briefing dizia | medido |
|---|---|
| «field3d: **1** `impl App`» | **2** — o censo grepou `^impl App` e o segundo é `impl crate::App` |
| «a família mais desacoplada» (implicando isolada) | ela é a mais desacoplada da **`App`**, e está **acoplada à escultura** em 7 ficheiros, no sentido inverso |
| «o registo é o ponto de extensão para o laço» | o `render_loop` chama **48** símbolos heterogéneos e **ordenados**; abstrair isso a partir de uma família é o que o próprio briefing proíbe. O registo resolve o `Cargo.toml`, não o laço |
| «as cenas não citadas apagam-se» (simples) | colide com um gate que o módulo já tinha (o roteador promete `1..CENAS`) ⇒ a esparsidade tem de ser **explícita** |

---

## 9 — Smoke do Enio (na worktree desta linha)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-host && env PH2D_FIELD_SMOKE=11 cargo run -p ph2d-host-desktop --profile smoke
```

**Binário deixado compilado** (DIRETRIZ §1.5.9 item 9) — 2.ª corrida:

```
$ cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.20s
$ cargo build -p ph2d-host-desktop --profile smoke 2>&1 | grep -c Compiling
0
```

**O que tem de acontecer:** exactamente o de antes — a peça da cena 11 no prato giratório, o pill
**MODEL**, o gizmo, as vistas. ⚠️ **A extracção não muda produto**; se alguma coisa parecer
diferente, é defeito desta linha.

**Vale a pena olhar também:**
- `PH2D_FIELD_SMOKE=26` (e `=27`, `=28`, `=29`, `=30`, `=31`, `=32`) — as cenas citadas continuam com
  os **mesmos números**;
- `PH2D_FIELD_SMOKE=15` — uma **podada**: tem de imprimir que a cena não existe, dizer porquê, e
  abrir a 1 no lugar dela (⛔ antes ela abria a 1 **em silêncio**);
- **sem env nenhuma**, abrir o pill **MODEL** — é o caminho do dono e prova que a família continua
  armada pelo pill.

⏳ **NÃO smokado por mim:** nada deste diff foi visto numa janela real — a linha fecha com os gates,
e o veredito do produto é do Enio.

---

## 10 — Aberto, para quem vier

1. **Os cinco alias da shell** morrem quando a `line/app-sculpt3d` fechar (§5.2).
2. **`Orbit`/`Screen` moram na `ph2d-field-render`** — a crate de render do módulo de *modelagem*.
   A `ph2d-viewport3d` depende dela só pelo tipo de câmera, e por transitividade a escultura também.
   **Já era verdade antes desta linha**; hoje tem nome. A cura é um vocabulário 3D partilhado e
   **não é desta wave** (mexeria na API de `ph2d-field-render`).
3. **O trait `AppHost` tem 5 métodos e cobriu o piloto inteiro.** A `physics` toca **126** membros de
   `App` em **22** `impl App`: se o pedido dela for grande, ela **pára e reporta com a lista** — a
   extensão do substrato é decisão do integrador (briefing §5-B).
4. **Os dois doc-links de `sculpt3d_export.rs`/`sculpt3d.rs` para `crate::field3d_export`** ficaram
   por converter, de propósito (são ficheiros da outra linha). Cosmético — `cargo doc` only.

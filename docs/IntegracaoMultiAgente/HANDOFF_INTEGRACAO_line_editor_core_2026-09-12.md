# HANDOFF DE INTEGRAÇÃO — `line/editor-core` (ARQUITECTURA **A5b + A10**) — 2026-09-12

> **Estado: FECHADA.** Nenhum pixel, texto ou comportamento mudou — tudo o que esta linha fez é
> endereço (onde um id mora, quem declara um módulo) e gate. A linha não integra e não faz ship
> (CLAUDE.md §0.7). Bloco de abertura: [`BLOCOS_ABERTURA_ARQUITETURA_2026-09-12.md`](BLOCOS_ABERTURA_ARQUITETURA_2026-09-12.md) §3;
> auditoria: [`AUDITORIA_ARQUITETURA_2026-09-12.md`](AUDITORIA_ARQUITETURA_2026-09-12.md) A5 e A10.

## §0 — Para o INTEGRADOR (leia isto primeiro; o resto é a prova)

**Estado:** fechada, gate batched verde (§8), auditoria de 2 lentes aplicada (§9), **smoke aprovado pelo dono
em 2026-09-13** (§10), binário do smoke compilado e `incremental/` reclamado (§11). `merge-base` = `main`
(`e18e75307`, 0 commits atrás) · 16 commits · HEAD `016ba36cc` + o commit que acrescenta este §0.

### 0.1 — Contra o que está em curso (medido 2026-09-13, `git merge-tree --write-tree`)

Das 18 worktrees, **só a `line/render-loop` tem commits fora do `main`** (141, sobre o mesmo `e18e75307`) —
é a linha com quem esta combinou a CERCA (§3). As outras 16 estão a 0 commits do `main`.

| com | resultado | o que fazer |
|---|---|---|
| `line/render-loop` | **1 conflito textual**: `crates/ph2d-app-vec/src/vector_bridge.rs` (~l. 307). Esta linha só trocou `ph2d_editor_core::ids::VECTOR_WIDTH` → `ph2d_tool_vector::ids::VECTOR_WIDTH` no `width_dragging`; a `render-loop` APAGOU o `width_dragging` inteiro (morreu com a `History` do vetor). ⇒ **fique com o lado da `render-loop`** — o caminho reescrito some com o bloco. Mais o `stroke_paint.rs`, que o **Mergiraf resolveu sozinho** (reveja: `mergiraf review`). Os outros 14 ficheiros tocados pelas duas fundem limpo | resolver pelo lado dela; a ordem das duas fusões é indiferente (a segunda paga o mesmo conflito) |
| `line/render-loop`, semântico | **0** linhas acrescentadas por ela nomeiam um id que desceu ou um caminho que saiu da fundação (sonda sobre `git diff -U0 e18e75307..line/render-loop`, as 6 formas de 0.2) | nada; a árvore combinada confirma por compilação |

### 0.2 — ⚠️ O que NÃO é aditivo (API pública que saiu da `ph2d-editor-core`)

- **1 242 nomes `pub` mudaram de crate** (ids de widget → 21 crates donas: `ph2d-panel-inspector` 386 ·
  `ph2d-tool-vector` 174 · `ph2d-panel-vector` 131 · `ph2d-panel-sculpt3d` 117 · `ph2d-tool-painter` 92 ·
  `ph2d-panel-timeline` 63 · `ph2d-panel-physics` 61 · `ph2d-tool-flip` 53 · …) e **30 foram apagados** (os
  órfãos de `de89e15ca`, §9). Contados por nome declarado nos ficheiros tocados, antes contra depois.
- `text_elide::paint_text_elided` / `paint_text_title_elided` → **`paint::`** (22 leitores já trocados).
- `fnv_node_id_runtime` · `flip_fnv_node_id` · `asset_fnv_node_id` → a porta **`ph2d_tool_registry::hash_node_id_runtime`**.
- `grid_snap::ids::GS_PANEL` (re-export) morreu → `ph2d_editor_core::ids::GS_PANEL`;
  `widget::showcase::SECTION_IDS` deixou de ser `pub`.
- ⇒ **Código NOVO de uma linha futura que nomeie o caminho antigo FALHA A COMPILAR** (alto, nunca mudo).
  A cura é nomear o dono — `python3 scripts/censo-ids.py --lista FICA-EC` diz o que ficou; o resto está na
  crate que o lê. ⛔ **Nunca repor uma fachada** para o fazer compilar: é a regra que esta linha pagou.

### 0.3 — Contratos e contadores (prova, não memória)

- Contratos congelados (§6 do `CLAUDE.md`): `git diff e18e75307 016ba36cc --` `ph2d-nodegraph/src/node.rs` ·
  `ph2d-editor-core/src/tool.rs` · `ph2d-vector-doc` · `ph2d-vector-traits` · `ph2d-imageio/src/color.rs` =
  **vazio**; os gates `architecture_*contract_surface` passaram na suíte de 22 709.
- `PROJECT_SCHEMA` · `VEC_SCENE_SCHEMA_VERSION` · `FLIP_SCHEMA_VERSION` · `DOC_VERSION` · `FIELD_DOC_VERSION` ·
  `register::<`: **0 linhas de código** no diff (as duas únicas menções são a tabela colada no §1).
- `Cargo.lock`: nenhum pacote externo novo. `Cargo.toml`: **18 manifestos**, só arestas INTERNAS (lidas com
  `tomllib`, base contra HEAD, por secção):
  - `[dependencies] ph2d-tool-registry` (a porta do hash que os ids descidos chamam) em **16 painéis**:
    asset-browser · authored · flip-frames · flip · grid-snap · hierarchy · inspector · model3d · padding ·
    physics · sculpt3d · skeleton · timeline · tokens · vector · widget-lab — custo de build **zero**, todos já
    a compilavam pela `ph2d-editor-core`;
  - `ph2d-panel-skeleton → ph2d-tool-vector` (os quatro ids de osso do ponto fixo, §2);
  - `[dev-dependencies]`: `ph2d-editor-core →` `ph2d-panel-flip` · `ph2d-panel-timeline` · `ph2d-panel-vector` ·
    `ph2d-tool-painter` · `ph2d-tool-vector` (os testes da fundação que lêem ids que desceram) e
    `ph2d-panel-sculpt3d → ph2d-panel-model3d`;
  - **saiu** `ph2d-panel-equalize-sizes → ph2d-tool-registry` (sem uso depois de a fachada morrer; `cargo machete`
    verde).
  Todas permitidas pela lei de camadas; ⚠️ as dev-dependências da fundação para os painéis são o re-acoplamento
  que o §5 mede para qualquer corte futuro dela.
- ADR: nenhum. Contrato congelado encostado: nenhum.

### 0.4 — ⭐ Os passos que ficam para DEPOIS das duas fusões (esta + `render-loop`), na árvore combinada

1. **Descer os 227 ids que a cerca prendia** (§3, lista nominal no §12):
   ```
   python3 scripts/censo-ids.py --json target/prova/censo.json
   python3 scripts/mover-ids.py target/prova/censo.json --plano
   python3 scripts/mover-ids.py target/prova/censo.json --aplicar
   python3 scripts/censo-ids.py --json target/prova/censo.json   # ponto fixo: DESCE tem de dar 0 (pode pedir 2 passagens, §2)
   ```
   ⚠️ O censo só os classifica `DESCE` se a `render-loop` tiver deixado de os nomear pelo caminho da fundação;
   o que ela ainda nomear continua `FICA-CERCA` e não é defeito. O `DOC_FACHADA` que o script escreve nos
   cabeçalhos é **nota provisória** — reescreva-a à mão (o texto diz isso).
2. **A fachada `screens::hero::ids`**: trocar os 7 sítios da cerca (`render_loop/hierarchy_rename.rs` 2 ·
   `inspector_strategy.rs` 3 · `mod.rs` 2) por `ph2d_editor_core::ids::…` e apagar o `pub use crate::ids;` do
   `screens/hero.rs`.
3. **Os três slugs repetidos** (`CEQ_PANEL` · `EQS_PANEL` · `UPS_PANEL`): trocar TODO leitor por
   `ph2d_editor_core::ids::…`, apagar as cópias e as três linhas de `SLUGS_REPETIDOS_TOLERADOS` (a nota do gate
   dá o `grep`).
4. **A catraca A10** (`architecture_the_foundation_modules_form_a_dag`): a entrada `action_bus → screens` (24) só
   se cura depois da fusão (os payloads do Inspector que a cerca nomeia por `screens::hero::`). ⚠️ Se a
   `render-loop` mexeu em referências entre módulos da fundação, o gate pode reprovar pela metade «desceu —
   desça o número»: **baixar o tecto é a cura certa**; subir nunca.
5. Re-correr na árvore combinada: `node_id_collisions` (a `render-loop` pode ter trazido literais novos) ·
   `the_painted_control_reaches_a_consumer` · o gate A10 · `the_shell_only_shrinks`.

### 0.5 — Para o `CLAUDE.md` §5 (UMA linha; quem a escreve é o integrador, no primário — DIRETRIZ §1.5.9 item 8)

Troque, no bullet *«As duas maiores crates do repo não eram nomeadas…»*, o parêntese da `ph2d-editor-core` por:

> (**98 633** em `src/`, medido 13/09 — widgets, interaction e os ids que a própria fundação lê, e **27** gates
> `architecture_*` entre os **112** ficheiros de `tests/it/`; ⭐ **A5b+A10 (12/09): 1 737 ids desceram para as
> crates que os lêem e os módulos de topo formam um DAG com catraca de 7 arestas** — uma edição num id de painel
> recompila 7–8 crates em vez de 61; [handoff](docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_editor_core_2026-09-12.md))

### 0.6 — Um defeito de PRODUTO pré-existente, reportado e não curado

`INSP_BLENDER_PICKER` tem dois valores (`ids/menus.rs:179` hash · `interaction/state/chrome_ops.rs:604` um
`NodeId(380)` local) ⇒ trazer o seletor de cor para a frente nunca funcionou. Curar muda o que se vê: decisão
do Enio (§9 achado 11).

## §1 — Identidade

| | |
|---|---|
| branch | `line/editor-core` |
| HEAD | o commit deste handoff, sobre `c6265384b` (o último de código) |
| merge-base | `e18e75307b78f3f9e8540c6cb448e55b5966c6d9` (= `main` no fecho: nada a rebasear) |
| commits de código | 15 (`3fe788108`..`c6265384b`) + o do handoff |
| diffstat (código) | 792 files changed, 25057 insertions(+), 17631 deletions(-) |
| ficheiros por área | painéis 544 · ferramentas 95 · `ph2d-editor-core` 85 · famílias 46 · shell 15 (**zero** na cerca) · `ph2d-param-editors` 3 · `scripts/` 2 · `ph2d-viewport3d` 1 · `Cargo.lock` |

### `collision-surface.sh` (re-corrido depois do último commit de código)

```
SUPERFÍCIE DE COLISÃO — line/editor-core contra main
  merge-base e18e75307   ·   15 commit(s)   ·   792 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
      616 / 700   crates/ph2d-editor-core/src/action_bus.rs  (tem marcador/allowlist — confira o valor congelado)
  ✗  7288 / 600   shells/desktop/src/input_dispatch.rs
───────────────────────────────────────────────────────────────────────────────
```

⚠️ **Os dois tectos da tabela são falsos alarmes desta linha, medidos:** o `input_dispatch.rs` tinha
**7 290** linhas no merge-base e a linha tirou-lhe 2 (é governado pela catraca da shell, não pelo
tecto por ficheiro); o `action_bus.rs` foi de 605 a 616 pelas três declarações `#[path]` dos filhos
(A10), não tem entrada congelada, e o «marcador» que o script vê é prosa do cabeçalho sobre outros
ficheiros. ⛔ **Nenhum contador partilhado se move**: schemas, registos de componentes, contrato e ADR
iguais à base.

---

## §2 — OBRA 1 (A5b): os ids descem para quem os lê

### O que se mediu antes de mover um byte

A auditoria dizia *«754 de 2 533 ids só têm leitor no painel dono e podem descer»*. O censo
(`scripts/censo-ids.py`) lê, por item de `ph2d-editor-core/src/ids/**`, **quem o lê** (tokens de
código — comentários e strings em branco; `pub use` não conta como leitor), **quem ele cita**, e
escolhe o dono = a crate MAIS BAIXA que todo leitor vê, sob a lei de camadas, e reagrupa por ficheiro
(um ficheiro de ids é um ASSUNTO e desce inteiro quando pode).

| censo | 1.ª passagem (v10) | ponto fixo (v14, depois de mover) |
|---|---:|---:|
| DESCE | **1 734** (tool-painter 507 · inspector 389 · tool-vector 178 · panel-vector 162 · sculpt3d 154 · timeline 64 · physics 61 · tool-flip 53 · flip-frames 31 · flip 30 · tool-bgremoval 23 · asset-browser 17 · model3d 15 · widget-lab 12 · tokens 11 · tool-padding 11 · hierarchy 7 · authored 5 · skeleton 3 · padding 1) | **0** |
| FICA-EC (a fundação lê) | 486 | 486 |
| FICA-CERCA (a cerca da line/render-loop cita) | 226 | 227 |
| FICA-CITACAO | 114 | 110 |
| FICA-SEM-DONO | 12 | 11 |
| FICA-EC-TESTE | 10 | 10 |
| ORFAO (`HIER_MAIN_CAMERA`, fica de propósito: faixa numérica 400..411) | 1 | 1 |

⭐ **O censo só chega ao ponto fixo na 2.ª passagem**: na 1.ª, as tabelas `VECTOR_BONE_ACTION_IDS` e
`VECTOR_BONE_DEFORM_IDS` ainda moravam na fundação e prendiam lá os quatro ids que citavam; descidas
elas (para o `ph2d-panel-skeleton`), o dono de todos os leitores desses quatro passou a ser a
`ph2d-tool-vector`. A 3.ª passagem dá zero. *Um censo com citações precisa de iterar até ao ponto fixo —
uma passagem só deixa para trás o que as citações seguravam.*

| | merge-base | HEAD |
|---|---:|---:|
| `ph2d-editor-core/src/ids` | 13 683 L em 84 ficheiros | **4 691 L em 63** |
| itens `pub` em `ph2d-editor-core/src/ids` (contados pela lente de correcção, §9) | 2 578 (em `699cbc583`, depois dos 35 órfãos) | **841** — 1 737 saíram; os `NodeId` que saíram são 1 395, com **0** valores mudados |
| `ph2d-editor-core/src` | 107 585 L em 469 | **98 633 L em 448** |
| crates donas com `src/ids/` | 1 (`ph2d-editor-core`) | **22** (21 donas: inspector 2 623 · tool-painter 2 946 · panel-vector 1 378 · sculpt3d 934 · tool-vector 798 · timeline 510 · … total 10 871 L) |

### As leis que a descida respeitou

- **UMA definição e ZERO re-exportações.** 16 fachadas `pub use ph2d_editor_core::ids::…` (e as de
  painel → ferramenta: bgremoval, color-equalization, equalize-sizes, upscale; e o glob do grid-snap)
  morreram; 2 059 leitores passam a nomear a crate dona. A fundação também tinha a sua
  (`grid_snap::ids::GS_PANEL` re-exportava `crate::ids::GS_PANEL`) e morreu.
- **Os slugs não se tocam.** Texto verbatim; os `HIER_*` numéricos (400..411) ficam numéricos.
- **Três nomes repetem o slug de propósito** e o censo derivado tolera-os por NOME
  (`SLUGS_REPETIDOS_TOLERADOS`): `CEQ_PANEL`, `EQS_PANEL` (na fundação e na ferramenta) e `UPS_PANEL`
  (na fundação e no painel, porque o `render_loop/upscale_bridge.rs` da cerca nomeia a cópia do painel).
  ⚠️ **Os leitores de `CEQ_PANEL`/`EQS_PANEL` nos painéis nomeiam a cópia da FERRAMENTA** — a que a
  fachada lhes dava. O script resolvia o nome para a da fundação e deixava-os por reescrever; foram
  16 sítios à mão.
- **Quatro módulos `ids` de painel ficaram só com prosa e saíram** (bgremoval, color-equalization,
  equalize-sizes, painter-layers). Nove crates ganharam um. Os 12 cabeçalhos que diziam *«os ids ficam
  na editor-core, isto é um re-export de conveniência»* dizem de onde cada id vem agora.

### Arestas novas (todas permitidas pela lei de camadas; nenhuma crate nova no grafo de ninguém)

| aresta | porquê | custo de build, medido |
|---|---|---|
| `ph2d-panel-asset-browser → ph2d-tool-registry` | a porta do hash (`hash_node_id`) | **zero** — o painel já dependia da `ph2d-editor-core`, que depende da `ph2d-tool-registry` |
| `ph2d-panel-authored → ph2d-tool-registry` | idem | zero, idem |
| `ph2d-panel-skeleton → ph2d-tool-registry` | idem | zero, idem |
| `ph2d-panel-skeleton → ph2d-tool-vector` | os quatro ids de osso do ponto fixo | **só o próprio painel** passa a recompilar quando a ferramenta muda: os três dependentes dele (`panel-vector`, `panel-registry-init`, a shell) já compilavam a `tool-vector` |

⚠️ **A tabela acima só nomeia as arestas que pediram argumento; a lista MEDIDA dos 18 manifestos está no
§0.3** (lida com `tomllib`, base contra HEAD): a `ph2d-tool-registry` entrou em **16** painéis, não 3 (os
outros 13 são o mesmo caso, custo zero pelo mesmo motivo); a fundação ganhou 5 dev-dependências para
donas; a `ph2d-panel-sculpt3d` ganhou a `ph2d-panel-model3d` como dev; e a `ph2d-panel-equalize-sizes`
**perdeu** a `ph2d-tool-registry`.

### `hash_node_id`: a opção escolhida e a recusada

- **(a) o painel depende da `ph2d-tool-registry`** — ESCOLHIDA. Custo de grafo zero (acima), e o gate
  `architecture_tool_contract_surface` fica verde por construção (nenhuma assinatura mexida).
- **(b) a lei desce para o lado do `NodeId`, na `ph2d-a11y`** — recusada: o `render_loop/mod.rs` da cerca
  cita `ph2d_tool_registry::hash_node_id` e reescrevê-lo é editar a cerca; e a mudança reescreveria
  ~40 linhas `use` em ~15 crates sem tirar uma unidade de compilação a ninguém.

### O censo de colisões passou a DERIVADO

`ph2d-editor-core/tests/it/node_id_collisions.rs` já não lê uma lista à mão de 619 consts: lê os
literais `hash_node_id("…")` da workspace inteira (e as formas não-literais, nomeadas numa tabela de 20
entradas com a espécie de cada uma), hasheia-os com a MESMA porta, e reprova (1) dois slugs diferentes
com o mesmo hash e (2) o mesmo slug em dois sítios fora da lista de tolerados. Pisos de população
(ficheiros ≥ 7 000 · literais ≥ 3 000 · crates com literal ≥ 15 · moldes ≥ 150). Prova de mutação nas
duas metades. *O id já não precisa de morar ao lado do teste que o vigia* — era essa a premissa que
prendia os ids na fundação.

### Gates que liam `ids/` pelo endereço

| gate | o que mudou |
|---|---|
| `the_painted_control_reaches_a_consumer` | `is_ids_dir` passa a todo módulo de ids (`*/ids/**` e `*/ids.rs`, a convenção do `node_id_collisions`); população de `pub const X: NodeId` **2 117 → 2 254** — 2 118 em `*/src/ids/**` e os 136 dos `ids.rs` que só o outro gate via (auditoria de fecho, §9). Só na fundação seriam 757, e o piso `MIN_IDS = 1 900` reprovava |
| `the_transform_group_is_lit_from_the_snapshot` (`ph2d-panel-sculpt3d`) | ⛔ a descida pô-lo VERMELHO (agulha `&ids::SCULPT3D_TRANSFORM`); curado no produto com um `use crate::ids;` privado — §9 achado 1 |
| shell `every_chrome_backdrop_is_known_to_the_scene` | varria só `ph2d-editor-core/src/ids`; passa a varrer os módulos de ids de todas as crates (os 5 `*_BACKDROP` ficaram na fundação, e o piso `found >= 5` já existia) |
| `architecture_panel_wiring_parity::table_driven_chips_are_registered_too` | ganha a metade justa: ≥ 160 tabelas `[NodeId; N]` (medido 181 antes e depois) |
| `section_headers_are_collapsible` (panel-vector) | lê o `VECTOR_SECTIONS` na própria crate |
| `node_id_collisions` | a forma `dynamic_id` mora em `ph2d-panel-timeline/src/ids/timeline.rs` |
| `the_menu_bar_relocates_the_verbs_it_shows` | inalterado: lê `src/ids/chrome/topbar.rs`, que ficou na fundação (piso ≥ 20 já existia) |
| `every_menu_row_reaches_a_handler` · `hr12_widgets_a11y` · shell `the_draw_pass_asks_the_door_that_has_the_gates` | inalterados, medidos: não leem `ids/` por caminho (o primeiro exclui `/ids/` do corpus e tem piso ≥ 100) |
| `the_gap_between_an_icon_and_its_label` | nomeia o `toast.rs` (A10: o pintor do balão mudou-se para lá) |

### ⚠️ O que a descida partiu nos gates de FORMA (e porque não era produto)

A 1.ª corrida das suítes deu **6 vermelhos, uma causa só**: `ids::X` virou `crate::ids::X` /
`ph2d_editor_core::ids::X`, o rustfmt partiu as linhas longas, e isso fez **três espécies** de gate
medir o efeito — tectos de LOC (ficheiro de painel 600, workspace 700, função 200), marcadores de linha
separados do literal (`// LITERAL-PX-OK`, `// CLAMP-OK`) e uma agulha textual
(`kid(ids::TexPatKnob::Lock),`). Cura: `use` PRIVADOS que dão a cada casa o nome curto (nunca uma
re-exportação); o prelúdio `pub(crate) use` do `sections/mod.rs` do Inspector passou a ligar `ids` à
casa dominante (554 usos do próprio painel contra 89 da fundação, agora `core_ids`); o `event.rs` da
timeline partiu-se por assunto (`event_stack_menus.rs`, 604 → 451); e os dois ficheiros de ids que
passavam o tecto (sculpt3d 711, inspector 673) partiram-se por assunto (brush/shading, physics_body).
⛔ Zero entradas novas em `FILE_OVERAGE_OK`.

---

## §3 — ⛔ A CERCA com a `line/render-loop`: os 227 ids que FICARAM

Um id que desceria e é citado num ficheiro da cerca (`render_loop/**`, `app_state.rs`, `vec_*.rs`,
`ph2d-app-vec/src/state.rs`) ficou na fundação. **O integrador desce-os depois das duas fusões** —
correndo `python3 scripts/censo-ids.py --json …` e `python3 scripts/mover-ids.py … --aplicar` sobre a
árvore combinada (o censo classifica-os DESCE assim que a cerca deixar de os nomear pelo caminho da
fundação).

| ficheiro da cerca | ids que prende | donos que os receberiam |
|---|---:|---|
| `render_loop/mod.rs` | 157 | panel-vector 132 · panel-skeleton 19 · panel-flip 2 · panel-timeline 2 · tool-vector 1 · panel-inspector 1 |
| `vec_layout_edit.rs` | 33 | panel-vector |
| `render_loop/timeline_bridge.rs` / `_tests.rs` | 15 / 15 | panel-timeline |
| `vec_anchor_edit.rs` | 8 | panel-vector |
| `render_loop/timeline_bridge_container_tests.rs` · `vec_layout_edit_tests.rs` · `render_loop/audio_overlay.rs` | 4 · 4 · 4 | panel-timeline · panel-vector · panel-audio-editor |
| `render_loop/inspector_strategy.rs` · `inspector_action_tests.rs` | 2 · 2 | panel-inspector |
| `authored_intents_tests.rs` · `tokens_bridge.rs` · `tokens_bridge_tests.rs` · `sim_extract_sheet.rs` · `inspector_instance.rs` · `inspector_timer_tests.rs` | 1 cada | authored · tokens · tokens · inspector · inspector · inspector |

(20 ids são citados por mais de um ficheiro da cerca.) A lista NOMINAL, por ficheiro, está no §12.

### ⛔ A fachada `screens::hero::ids` FICA — só a cerca a nomeia

O `screens/hero.rs` da fundação tem `pub use crate::ids;` (anterior a esta linha): um caminho-alias dos
ids, que a regra *«zero fachadas»* manda apagar. **A cerca nomeia-o em 3 ficheiros / 7 sítios**
(`render_loop/hierarchy_rename.rs` 2 · `render_loop/inspector_strategy.rs` 3 · `render_loop/mod.rs` 2).
Os outros **24 leitores** (31 sítios: `ph2d-app-field3d`, `ph2d-app-sculpt3d`, `ph2d-panel-hierarchy`,
`ph2d-panel-widget-gallery`, `ph2d-tool-registry-init`, `ph2d-viewport3d`, a `forwarding.rs` e três
testes da shell, e os testes da fundação) já nomeiam `ph2d_editor_core::ids` — reescritos por script com
contagem conferida (21 caminhos + 10 grupos `hero::{…, ids}`), sem tocar em comentários. ⇒ **Depois das
duas fusões, o integrador troca os 7 sítios da cerca e apaga a linha do `hero.rs`.** A outra fachada da
fundação (`widget::showcase::SECTION_IDS`, sem leitor fora do próprio módulo) passou a `use` privado.

---

## §4 — OBRA 2 (A10): os módulos de topo da fundação formam um DAG

### A tabela da base e a do fim

| | a auditoria (antes da A5b) | depois da A5b, pela ÁRVORE de módulos | depois das curas |
|---|---|---|---|
| módulos de topo | — | 35 | **32** |
| ciclos | três pares medidos: `widget ↔ interaction` 73/165 · `screens ↔ interaction` 162/22 · `interaction ↔ ids` 90/10 | **UM ciclo de 23 módulos, 1 188 referências** (o `ids → interaction` já tinha morrido com a A5b) | **UM ciclo de 14 módulos, 835 referências**, fechado por exactamente 7 arestas |

⚠️ **O módulo de um ficheiro lê-se da árvore, nunca do nome:** a 1.ª sonda contou `motion_tests.rs`,
`action_bus_queue.rs`, `paint_shapes.rs` como módulos de topo (50 em vez de 35) — eles são filhos por
`#[path]`, e o gate lê o `lib.rs`.

### As curas por ASSUNTO (quatro commits)

1. **`action_bus_{queue,kinds,hier}` passam a FILHOS do `action_bus`** — eram declarados na raiz da
   crate e re-exportados pelo `action_bus`: três módulos de topo num ciclo com ele. Os ficheiros não
   mudam de sítio; nenhum chamador muda.
2. **Os dois gates régua × porta de comprimento voltam para `ruler_tests.rs`** — tinham ido para
   `length_tests.rs` com a regra que julgam (que fica lá), mas chamam o `label_text` da régua, que está
   acima da porta.
3. **O `impl Paint for ToastQueue` muda-se para o `toast.rs`** — era por ele que o `paint` dependia do
   `toast` e do `progress` (`column_row`). `paint.rs` 650 → 541.
4. **Os pintores de texto cortado sobem para o `paint`** (`paint_text.rs`); a LEI da reticência (`fit`,
   `fit_weighted`, `title_elided_width`, `elide` — este `pub(crate)` com a pré-condição escrita) fica no
   `text_elide`. 22 leitores em 5 crates passam a `paint::paint_text_elided`.

### O gate: `architecture_the_foundation_modules_form_a_dag`

Lê a árvore a partir do `lib.rs` (seguindo `#[path]`), conta em CÓDIGO (comentários e strings em branco;
**testes incluídos**) `crate::X`, cada `X` de `crate::{…}` e cada `super::…::X` que sobe à raiz (com a
profundidade dos `mod` inline), e vê através das re-exportações do `lib.rs`. Medido: 32 módulos · 448
ficheiros · 1 615 referências · 4 só através da raiz.

| aresta tolerada | tecto | a cura escrita na catraca |
|---|---:|---|
| `widget → interaction` | 49 | o substrato (`HitIndex`, `WidgetStore`, `InteractiveState`, `WidgetEvent`, os `*State`) desce ABAIXO dos pintores; o `dispatch` fica em cima |
| `interaction → screens` | 18 | os tipos de doca (`DockSide`, `TaskLayout`, `Slot`, `ChromeBands`, `TIMELINE_DOCK_H`) descem; as tabelas `panel_for_tab`/`MENUS`/`LEGACY_PILL_MENUS` são injectadas pelo hero. ⚠️ `Slot`/`SlotSet` têm 144 leitores em 86 ficheiros fora da fundação |
| `action_bus → screens` | 24 | os payloads do Inspector (`*FieldEdit`, `*Info`, `HierReparentIntent`) descem para um vocabulário abaixo do `action_bus`. ⛔ **bloqueada pela cerca nesta rodada**: o `render_loop` nomeia `HierReparentIntent`, `InspectorPropertiesInfo`, `InspectorInstanceInfo` e `SpriteFieldEdit` pelo caminho `screens::hero::` |
| `panel → screens` | 7 | os mesmos tipos de doca; `HeroSelection`/`HeroLayout` passam ao contrato do painel como os tipos de baixo que são |
| `motion → interaction` | 4 | só em teste; os testes sobem — o `motion_grid_tests` lê privados (`Track`, `value`) que precisam de porta antes |
| `motion → screens` | 1 | só em teste; idem |
| `motion → widget` | 1 | só em teste; um teste do `motion_tests.rs` pinta um `Button` — sobe para o `widget` |

Cada entrada é **necessária sozinha** (tirá-la fecha um ciclo — é a metade de obsolescência do gate).

**Prova de mutação** (controlo verde antes e depois, restauro byte-idêntico):

| mutação | resultado |
|---|---|
| sem a entrada `panel → screens` | ✗ `the_foundation_modules_form_a_dag` (ciclo) |
| tecto `widget → interaction` 49 → 48 | ✗ idem (cresceu) |
| tecto 49 → 50 | ✗ `the_ratchet_only_describes_what_is_still_true` (desça o número) |
| o leitor ignora `#[path]` | ✗ os dois (piso de ficheiros, contagens) |
| strings passam a contar | ✗ `the_reader_sees_what_it_claims_to_see` |
| aresta REAL no produto: `paint` → `crate::screens::slot::Slot` | ✗ `the_foundation_modules_form_a_dag` (ciclo) |
| ⚠️ o leitor deixa de ver através das re-exportações do `lib.rs` | **SOBREVIVEU** na 1.ª prova (nenhuma referência de hoje muda o veredito) → curado com `PISO_VIA_REEXPORT`; a mesma mutação passa a ✗ |

⚠️ **E a auditoria de fecho achou nove formas que o leitor NÃO via** (§9, achados 2 e 3) — nenhuma ocorre
na árvore de hoje, por isso nenhuma mutação da 1.ª prova as tocava. O leitor mudou-se para
`tests/common/foundation_module_tree.rs` (pelo tecto de LOC), ganhou as nove, e a árvore de hoje lê-se
**igual** (32 · 448 · 1 615 · 4). Segunda prova (`target/prova/mutacao_leitor/`, controlo verde antes e
depois, restauro byte-idêntico + `touch`):

| mutação (desfaz UMA forma) | resultado |
|---|---|
| M1 `'\''` volta a fechar na aspa escapada | ✗ `the_reader_sees_the_forms_the_closing_audit_named` |
| M2 o prefixo `b`/`c` volta a esconder o `r` de `br#"…"#` | ✗ idem |
| M3 um grupo depois de `super::…` que chega à raiz volta a ser ignorado | ✗ idem |
| M4 `use super::*`/`crate::*` deixa de abrir o escopo dos caminhos soltos | ✗ idem |
| M5 só `pub use` conta na raiz (`pub(crate) use`, `use` privado passam) | ✗ idem |
| M6 uma glob de módulo de topo na raiz deixa de ser registada | ✗ idem |
| M7 uma linha em branco volta a separar o `#[path]` do `mod` | ✗ idem |
| M8 `mod x;` dentro de `mod {}` inline procura o ficheiro no directório errado | ✗ idem |
| M9 a vírgula de um grupo ANINHADO conta como item de topo | ✗ `the_reader_sees_what_it_claims_to_see` — ⚠️ era um defeito REAL da minha 1.ª redacção do leitor novo, e foi a sentinela ANTIGA que o apanhou |

(O resumo impresso pelo `mutacao_leitor.py` chama «sobreviventes» às nove: ele contou a linha `error: test
run failed` do próprio nextest como erro de compilação. Os nove ficheiros mostram `pass=3` — compilou — e
o teste esperado a reprovar.)

---

## §5 — O leque de recompilação e a proposta de corte (⛔ NÃO se partiu nada)

### A medição: uma edição REAL de uma linha

Protocolo (`target/prova/fanout2/`): acrescentar um `pub const` `#[doc(hidden)]` ao fim de
`ph2d-editor-core/src/zones.rs` (o módulo mais baixo, nomeado por 41 dependentes), `cargo test --no-run
--workspace --profile ci-test --timings`, restaurar e medir de novo — duas vezes, esperando `load < 4`
antes de cada corrida, com o restauro conferido byte a byte (`cmp`). ⛔ Não é `touch` (o incremental vê o
hash igual e o mtime suja todos os alvos).

| corrida | parede | crates recompiladas | `/proc/loadavg` antes → depois |
|---|---:|---:|---|
| edição 1 | 23,8 s | 61 | 3,30 → 24,17 |
| restauro 1 | 24,3 s | 61 | 3,83 → 22,78 |
| edição 2 | 25,7 s | 61 | 3,97 → 19,97 |
| restauro 2 | 23,8 s | 61 | 3,96 → 13,88 |

Soma do `--timings` na edição 1: **344 s de CPU em 61 crates** (⚠️ é parede por unidade e infla sob a
contenção da própria build — leia as PROPORÇÕES). Quem paga: `ph2d-host-desktop` 44,9 s (13 %) ·
`ph2d-editor-core` 23,8 s (7 %, lib + os 110 ficheiros de `tests/it`) · `ph2d-tool-painter` 20,8 s ·
`ph2d-app-motion` 17,2 s · `ph2d-render` 14,2 s · `ph2d-app-vec` 12,0 s · `ph2d-app-physics` 11,9 s ·
`ph2d-panel-vector` 11,5 s · `ph2d-app-field3d` 10,4 s · `ph2d-app-sculpt3d` 10,4 s — as doze primeiras são
57 %. **A fundação é 7 % do próprio leque; o custo é de quem depende dela.**

`cargo metadata` (12/09): **57 dependentes normais** (a auditoria contava 43). Por módulo nomeado:
`zones` 41 · `widget` 37 · `screens` 37 · `interaction` 37 · `ids` 35 · `paint` 33 · `panel` 32 ·
`icons` 29 · `tool` 26 · `action_bus` 20 · `motion` 16.

### A proposta de corte, medida antes de se escrever

Instrumento: `corte_split.py` (scratchpad da linha; a tabela está em `target/prova/fanout2/corte_split.txt`).
Modelo do cargo — uma biblioteca recompila se ela ou o fecho das suas dependências NORMAIS nomeia (em
código) um módulo do pedaço que recompilou; um alvo de teste, se a biblioteca dele recompila, se os
próprios testes nomeiam o pedaço, ou se uma dev-dependência (um salto) recompila. Partida a fundação,
editar um módulo recompila a crate dele e as crates da fundação acima dela na DAG **de depois da catraca
vazia**. Custo: o `--timings` da edição 1. ⭐ **Controlo:** com a fundação inteira o modelo prevê
exactamente as 61 medidas (0 em falta, 0 a mais).

Réplica: os **445 commits de 30 dias** à `ph2d-editor-core/src` (441 com módulo de hoje), cada um
recompilando o que o corte lhe faria recompilar. Os módulos mais editados: `screens` 234 · `ids` 232 ·
`widget` 113 · `interaction` 99 · `action_bus` 59 · `paint` 42 · `motion` 39 · `panel` 33.

| partição (depois da catraca vazia) | editar `screens` | editar `ids` | editar `widget` | média por commit de 30 dias |
|---|---:|---:|---:|---:|
| hoje (uma crate) | 61 · 344 s | 61 · 344 s | 61 · 344 s | 61 · 344 s |
| A · `screens` sobe para uma crate | **46 · 284 s** | 61 · 344 s | 61 · 344 s | 58,9 · 336 s |
| B · `screens`+`panel`+`action_bus` sobem | 47 · 285 s | 61 · 344 s | 61 · 344 s | 58,8 · 335 s |
| C · `ids` desce para uma crate | 61 · 344 s | 61 · 344 s | 61 · 344 s | 61,0 · 344 s |
| D · `ids` desce e `screens` sobe | 46 · 284 s | 61 · 344 s | 61 · 344 s | 58,9 · 336 s |
| **tecto** · cada módulo de topo a sua crate | 46 · 284 s | 61 · 344 s | 61 · 344 s | **57,4 · 331 s** |

Poupadas numa edição a `screens` (partição A), **15**: `ph2d-tool-painter` 20,8 s · `ph2d-render` 14,2 s ·
`ph2d-tool-color-equalization` 3,1 s · `ph2d-tool-runtime` 3,0 s · `ph2d-paint-gpu` 2,3 s ·
`ph2d-tool-vector` 2,3 s · `ph2d-tool-equalize-sizes` 2,0 s · `ph2d-param-editors` 1,9 s ·
`ph2d-tool-upscale` 1,8 s · `ph2d-tool-padding` 1,8 s · `ph2d-tool-move` 1,5 s · `ph2d-tool-flip` 1,5 s ·
`ph2d-tool-bgremoval` 1,4 s · `ph2d-tool-motion` 1,2 s · e a `ph2d-tool-registry-init` (biblioteca).

⛔ **A recomendação é NÃO partir a fundação pelos módulos de topo agora**, e o número diz porquê: nem o
TECTO (cada módulo a sua crate) poupa mais de **3,8 %** do CPU por commit, e o melhor corte possível
(`screens` sobe) poupa **17 %** numa edição a `screens` e **2,3 %** em média. O mecanismo: as 46 que
ficam nomeiam o SUBSTRATO (`widget`, `interaction`, `paint`, `zones`, `ids`), e toda edição ao substrato
é uma edição a todas. ⛔ **E `ids` numa crate própria poupa ZERO:** toda crate que nomeia `ids` nomeia
outra coisa da fundação (e os testes de `ph2d-render`/`ph2d-paint-gpu` re-acoplam por `[dev]
ph2d-tool-painter`).

⇒ **O corte que vale a pena, quando valer, é UM:** `screens` numa crate própria (`ph2d-editor-screens`,
por cima da `ph2d-editor-core`), o módulo mais editado do repo. Ele só exige **4 das 7** entradas da
catraca curadas — as que ENTRAM em `screens` (`interaction → screens` 18 · `action_bus → screens` 24 ·
`panel → screens` 7 · `motion → screens` 1, só teste); `widget → interaction` e as outras duas do
`motion` ficam DENTRO da crate de baixo e não o bloqueiam. ⚠️ O `action_bus → screens` está preso pela
cerca da `line/render-loop` (os payloads do Inspector).

### ⭐ O que a A5b já comprou — a mesma réplica, sobre os ids

**226** commits de 30 dias tocaram `ph2d-editor-core/src/ids`. Lidas as declarações (`pub const|fn|enum|
struct|static`) que cada um acrescentou ou mudou, contra onde elas moram HOJE:

| o commit mexeu em | commits |
|---|---:|
| **só ids que hoje moram numa crate DONA** | **109** (`ph2d-panel-sculpt3d` 40 · `ph2d-panel-inspector` 37 · `ph2d-tool-vector` 9 · `ph2d-panel-model3d` 9 · `ph2d-panel-vector` 7 · `ph2d-tool-painter` 6 · `ph2d-panel-skeleton` 2 · `ph2d-panel-grid-snap` 1) |
| um id que FICA na fundação | 82 |
| só ids que já não existem | 23 |
| nenhuma declaração (prosa, corpo de função) | 12 |

⇒ **metade dos commits de ids deixou de tocar a fundação** — cada um recompilava as 61; hoje recompila a
crate dona e o que depende dela. Medido pelo mesmo protocolo (`target/prova/leque_dono/`, `load < 4`,
restauro byte-idêntico), nos dois donos com mais commits:

| edição de UMA linha em | parede | crates recompiladas | CPU (`--timings`) | previsto pelo `cargo metadata` (normais + dev um salto) |
|---|---:|---:|---:|---:|
| `ph2d-editor-core/src/zones.rs` (a fundação, acima) | 23,8–25,7 s | 61 | 344 s | 61 |
| `ph2d-panel-sculpt3d/src/ids/sculpt3d.rs` | **13,4 s** | **7** | **42 s** | 7 |
| `ph2d-panel-inspector/src/ids/inspector.rs` | **14,3–15,0 s** | **8** | **51–59 s** | 8 |

(As 7: `ph2d-panel-sculpt3d`, `ph2d-app-sculpt3d`, `ph2d-panel-model3d`, `ph2d-panel-registry-init`,
`ph2d-app-registry-init`, `ph2d-host-desktop` e o alvo de teste da `ph2d-editor-core`, que tem os painéis
como dev-dependência. As 8 do Inspector trocam a `ph2d-app-sculpt3d` pela `ph2d-app-physics` e a
`ph2d-panel-painter-layers`.) ⚠️ **A 1.ª leitura da edição no sculpt3d deu 61 crates e é DESCARTADA:** foi
a primeira build da workspace depois de um commit de código, e recompilou o atraso do commit junto com a
sonda — a leitura do restauro, que é a mesma edição ao contrário, bate com o previsto. *Uma medição de
leque sem aquecimento mede o último commit, não a edição.*

⇒ **Em CPU, cada commit de ids que desceu custa hoje ~15 % do que custava** (42–59 s contra 344 s), e na
parede ~55 %. Sobre os 109 commits de 30 dias, isso são ~109 × 294 s ≈ **8,9 horas de CPU por mês**
devolvidas (soma do `--timings`, que infla sob contenção — a ordem de grandeza é o que conta) — **mais de
cinco vezes** o que a melhor partição da fundação pelos módulos de topo compraria sobre os 445 commits
(o TECTO: 13 s × 445 ≈ 96 min, e só depois de a catraca esvaziar).

---

## §6 — Construído, medido e REVERTIDO

- **A 1.ª aplicação do `mover-ids.py` a todos os donos compilou com 481 erros** e foi revertida inteira.
  Os erros tinham cinco espécies, e cada uma era um defeito do script: ligações herdadas por
  `use super::*;` (o prelúdio do Inspector serve 40 ficheiros), o alias `ids` da raiz de uma crate que
  ganha o próprio módulo (ANTES fundação, DEPOIS a crate), `tests/` vs `src/tests/` (a `crate` de um
  teste de integração é outra), imports que só um `#[cfg(test)]` usa, e aliases dentro de funções.
  Corrigido o script, a 2.ª aplicação deu 2 erros (as variantes `use ids::Enum::{…}`), à mão.
- **A 1.ª execução da cura dos pintores de texto abortou num assert de contagem** (o gate
  `architecture_motion_chrome_never_wraps_a_row_label` nomeia a porta DUAS vezes, não uma) — o script
  escreve só no fim, e a árvore ficou intacta.
- **Subir os testes do `motion`** para o `interaction` foi medido e ficou de fora: o `motion_grid_tests`
  usa privados do `motion`, e mover só o `motion_surface_tests` baixava a aresta de 4 para 1 sem a apagar.
- **Mover a régua `column_row` para o `zones`** foi a 1.ª ideia para `paint → progress` e ficou de fora: o
  comentário do `render_loop/mod.rs` (cerca) diz que a régua mora no `progress`; mudar o PINTOR para o
  `toast` curou a aresta sem mentir ao comentário.

## §7 — As premissas DESTE bloco que a medição derrubou

1. *«754 ids podem descer»* → **1 738 desceram**; a régua da auditoria contava leitores por painel dono e
   não via as ferramentas como donas (o precedente do ADR-0040 §3.8).
2. *«43 dependentes»* → **57** hoje.
3. *«`interaction ↔ ids` 90/10»* → a volta (`ids → interaction`) morreu com a A5b; a ida (85) é a
   fundação a ler os ids que ficam lá, e desce com o resto.
4. *«a lei desce para o lado do `NodeId`»* (opção b) → a cerca bloqueia-a, e ela não tira uma unidade de
   compilação a ninguém.
5. *«uma passagem do censo basta»* → o ponto fixo pede duas (§2).
6. *«as fachadas dos painéis re-exportam a fundação»* → quatro re-exportavam a FERRAMENTA, e duas delas
   escondiam um slug repetido (`CEQ_PANEL`, `EQS_PANEL`).
7. *«partir a fundação corta o leque»* → o TECTO (cada módulo de topo a sua crate) poupa 3,8 % do CPU por
   commit de 30 dias; o leque é das dependentes que nomeiam o SUBSTRATO, não do acoplamento entre módulos
   (§5).
8. *«o meu gate vê através das fachadas da raiz»* (a minha prova de mutação, M6) → via UMA forma das
   cinco; a auditoria de fecho achou as outras quatro e mais cinco formas cegas do leitor (§9). *Uma
   mutação prova que o leitor vê a forma MUTADA — não as formas que ninguém escreveu.*

## §8 — A PROVA DO FIM

Sobre `c6265384b` (+ a cura do clippy abaixo), `target/prova/fecho/`:

| prova | resultado |
|---|---|
| `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast` (por `scripts/ph2d-run.sh`) | **22 709 passed, 0 failed** (1 slow), 2 269 skipped — 129,9 s |
| `cargo nextest list --workspace --cargo-profile ci-test` + `scripts/nextest-list-diff.py` | antes 22 700 · depois 22 709 · **ONLY-A = 0** · MOVED 5 · ONLY-B 9 |
| `cargo test -p ph2d-host-desktop --test it` (à parte) | **795 passed**, 0 failed, 6 ignored |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | ⚠️ a 1.ª corrida REPROVOU: `collapsible_if` no `um_ciclo` do gate A10 (já estava no `dde9f0af5` — o gate nasceu sem clippy). Curado com um `if … && let …`; 2.ª corrida: ver a linha seguinte |
| clippy, 2.ª corrida | **exit 0, 0 erros, 0 avisos**; e o gate A10 re-corrido depois da cura: 4 de 4 |
| `cargo check --workspace --all-targets` | exit 0, **0 avisos** |
| `cargo machete` | nenhuma dependência sem uso |
| `typos --force-exclude` nos 789 ficheiros tocados | 0 — depois de curar três (`sem_coment`, `re-exporte`, `hom`) |
| `rustfmt --check` nos `.rs` tocados | 0 diferenças |

**MOVED 5** — testes que desceram COM os ids para a crate dona (a mesma chave, outro pacote):
`cells_are_distinct_from_each_other_and_from_the_fixed_ids`, `the_catalog_row_ladder_round_trips`,
`the_runtime_hasher_agrees_with_the_const_one` → `ph2d-panel-asset-browser`;
`per_layer_ids_are_distinct_by_layer_and_kind`, `runtime_hasher_matches_const_hash_node_id` →
`ph2d-panel-flip`.

**ONLY-B 9** — todos gates novos desta linha, em `ph2d-editor-core::it`: o censo derivado de colisões
(`a_runtime_template_never_spells_a_literal_slug` · `every_non_literal_hash_is_named` ·
`no_slug_is_declared_in_two_places` · `the_census_sees_the_whole_workspace` ·
`the_repeated_slug_ratchet_still_describes_the_tree`) e o da A10 (`the_foundation_modules_form_a_dag` ·
`the_ratchet_only_describes_what_is_still_true` · `the_reader_sees_what_it_claims_to_see` ·
`the_reader_sees_the_forms_the_closing_audit_named`).

## §9 — Auditoria de 2 lentes

Dois subagentes só de leitura sobre `e18e75307..dde9f0af5`, cada um com sondas próprias em Python (o
leitor do gate A10 reconstruído dá os MESMOS números que o gate imprime — 32 módulos, 448 ficheiros,
1 615 referências, 4 pela raiz, ciclo de 14 com 835): **correcção** (dois sítios que devem concordar) e
**costura de UI** (pintado, registado, clicável, o clique a chegar ao consumidor), contra os riscos desta
mudança — homónimos com slugs diferentes, leitor reescrito para a cópia errada, famílias dinâmicas em
duas cópias.

| # | achado | lente | destino |
|---|---|---|---|
| 1 | ⛔ **um gate VERMELHO que a descida criou**: `the_transform_group_is_lit_from_the_snapshot` (`ph2d-panel-sculpt3d`) procura a agulha `&ids::SCULPT3D_TRANSFORM`, e o `mover-ids.py` reescreveu-a para `&crate::ids::…`. Era a ÚNICA guarda dos chips de transformação. A lente conferiu as **790** agulhas de teste que nomeiam um caminho de ids: só esta perdeu o alvo | UI | curado: `use crate::ids;` privado no `paint/mask_tools.rs` — o texto volta a `&ids::…` |
| 2 | o leitor do gate A10 via só `pub use <topo>::…` na raiz: `pub(crate) use`, `use` privado, atributo à frente, `crate::` à frente e glob escondiam uma aresta — exactamente a fachada que o gate diz proibir | correcção | curado: leitor reescrito em `tests/common/foundation_module_tree.rs`; uma glob de módulo de topo na raiz REPROVA |
| 3 | e cego a `super::{…}` que chega à raiz, a `use super::*`/`crate::*` + caminho solto, ao `'\''` (fechava na aspa escapada e abria um literal falso), ao `br#"…"#`, a um doc-comment entre o `#[path]` e o `mod`, e a `mod x;` dentro de `mod {}` inline. Nenhuma forma ocorre hoje | correcção | curados; sentinela `the_reader_sees_the_forms_the_closing_audit_named` e prova de mutação (abaixo) |
| 4 | cabeçalhos desactualizados: `ph2d-tool-bgremoval/src/ids.rs` (dizia que os `BGR_*` moram na fundação e que o painel os re-exporta), `ph2d-tool-color-equalization/src/ids.rs` (fachada morta, censo na crate errada), `sculpt3d_cloth.rs` (`#[path]` que não existe), `vector_bool.rs` (`mod.rs` que é `ids.rs`), `ph2d-panel-upscale/src/ids.rs` (dois parágrafos contraditórios) | correcção | curados |
| 5 | a nota de `SLUGS_REPETIDOS_TOLERADOS` dizia que só a cerca lê as cópias; o `NODE_ID` e o `paint.rs` dos três painéis e um teste da ferramenta também lêem — seguir a nota partia a compilação | as duas | curado: a nota dá o `grep` inteiro |
| 6 | `the_painted_control_reaches_a_consumer` lia só `*/src/ids/**`; o `node_id_collisions` lê também `*/ids.rs` — **136** ids invisíveis ao primeiro (pré-existente) | as duas | curado: a mesma convenção nos dois |
| 7 | *«zero re-exportações»* valia para os ids das crates donas; a fundação guardava `screens::hero::ids` e `widget::showcase::SECTION_IDS` | correcção | §3: a primeira fica pela cerca (24 leitores reapontados), a segunda passou a `use` privado |
| 8 | o gate da shell `every_chrome_backdrop_is_known_to_the_scene` varria só `ph2d-editor-core/src/ids` com o doc a dizer «a árvore inteira» | UI | curado: varre os módulos de ids de todas as crates |
| 9 | *«23 módulos com 1 191 referências»* — a sonda da lente dá **1 188**, e a minha, re-corrida em `173193b13` e `706630fdb`, também | correcção | corrigido no gate e no §4 |
| 10 | a prosa *«a fundação que 43 crates recompilam»* em 88 sítios (a auditoria dizia 43; medido: 57 dependentes, 61 recompiladas) | correcção | curado por script com contagem (78 + 10 frases) |
| 11 | ⚠️ **PRÉ-EXISTENTE e NÃO curado — muda produto:** `INSP_BLENDER_PICKER` tem DOIS valores — `hash_node_id("insp_blender_picker")` em `ids/menus.rs:179` e um `NodeId(380)` local em `interaction/state/chrome_ops.rs:604`. O `set_picker_target` sobe o z de um id que o picker nunca regista ⇒ *trazer o seletor de cor para a frente* não faz nada. Igual no merge-base | as duas | reportado ao Enio (§0.8/G: uma cura que muda o visível PÁRA). ⛔ Nenhum gate pergunta «um NOME, um valor»: o censo compara slugs, e um `const` local num corpo de função passa |

**Conferido limpo pelas lentes:** 1 395 `NodeId` saíram de `ids/` da fundação com **0** valores mudados;
nenhum slug mudou (só os 30 órfãos que `de89e15ca` apagou); nenhum nome passou a estar definido duas vezes
(os 9 nomes repetidos são os mesmos, com os mesmos valores, na base e no HEAD); os três slugs repetidos
têm o mesmo valor nas duas cópias, e os leitores ficaram com a cópia que a fachada lhes dava; as duas
funções da timeline que mudaram de ficheiro são idênticas sem os caminhos; os 25 painéis registam os
mesmos 145 ids na base e no HEAD; nenhuma família dinâmica ficou em duas cópias; o diff da cerca é vazio.

## §10 — O que o dono deve exercitar no smoke

✅ **Smoke APROVADO pelo dono em 2026-09-13** («Smoke OK»), sobre `016ba36cc`, com o binário do §11 e os seis
passos da resposta de fecho (escultura/transformar · Inspector · Hierarquia · Timeline · os três utilitários
de imagem). ⚠️ Integrar não é aprovar: o `main` combinado smoka-se outra vez depois da fusão.

Nada mudou de propósito — o smoke é a prova de que continua tudo igual, nos painéis cujos ids mudaram de
casa. O modo de falha a procurar é o MUDO: um leitor reescrito para a cópia errada de um id compila e dá
um controlo que não responde ao clique (ou faz reagir o vizinho). Os painéis com mais ids descidos, por
ordem: **Sculpt** (os chips de transformação — o gate que a descida pôs vermelho), **Inspector** (abrir e
fechar secções, mexer num campo), **Vector** (secções recolhíveis, booleanas), **Hierarchy** (linha, menu
de contexto, renomear), **Timeline** (menus de lane e de strip), **Painter** (camadas), os três utilitários
de imagem com slug repetido (**Background Removal**, **Color Equalization**, **Upscale**), **Grid Snap**,
**Model 3D** e **Skeleton**. Os passos numerados para o Enio vão na resposta de fecho.

## §11 — O smoke compilado

`rm -rf target/*/incremental` (0 directórios restantes), depois `cargo build -p ph2d-host-desktop --profile
smoke` duas vezes — a 1.ª em 59,83 s, a 2.ª:

```
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

Comando do smoke, na árvore desta linha:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-editor-core && cargo run -p ph2d-host-desktop --profile smoke
```

## §12 — Anexo: os ids presos pela cerca, por ficheiro

Derivado por `scripts/censo-ids.py` (classe FICA-CERCA): cada ficheiro da cerca, quantos ids desceriam
se ele não os nomeasse, e o dono de cada um. É a lista que o integrador desce depois das duas fusões.

- `shells/desktop/src/render_loop/mod.rs` — 157: `FLIP_COLORIZE_APPLY` (ph2d-panel-flip), `FLIP_COLORIZE_CLEAR` (ph2d-panel-flip), `INSP_LIVE_ANCHOR_SECTION` (ph2d-panel-inspector), `VECTOR_BONE_BIND` (ph2d-panel-skeleton), `VECTOR_BONE_EXPAND` (ph2d-panel-skeleton), `VECTOR_BONE_IK_ADD` (ph2d-panel-skeleton), `VECTOR_BONE_IK_CHAIN` (ph2d-panel-skeleton), `VECTOR_BONE_IK_MIX` (ph2d-panel-skeleton), `VECTOR_BONE_IK_REMOVE` (ph2d-panel-skeleton), `VECTOR_BONE_IK_SOFTNESS` (ph2d-panel-skeleton), `VECTOR_BONE_LENGTH` (ph2d-panel-skeleton), `VECTOR_BONE_LIMIT_ADD` (ph2d-panel-skeleton), `VECTOR_BONE_LIMIT_MAX` (ph2d-panel-skeleton), `VECTOR_BONE_LIMIT_MIN` (ph2d-panel-skeleton), `VECTOR_BONE_LIMIT_REMOVE` (ph2d-panel-skeleton), `VECTOR_BONE_RELEASE` (ph2d-panel-skeleton), `VECTOR_BONE_SMART_ADD` (ph2d-panel-skeleton), `VECTOR_BONE_SMART_FROM` (ph2d-panel-skeleton), `VECTOR_BONE_SMART_PICK` (ph2d-panel-skeleton), `VECTOR_BONE_SMART_REMOVE` (ph2d-panel-skeleton), `VECTOR_BONE_SMART_TO` (ph2d-panel-skeleton), `VECTOR_BONE_STRENGTH` (ph2d-panel-skeleton), `TIMELINE_MOTION_PATH` (ph2d-panel-timeline), `TIMELINE_ONION_SETTINGS` (ph2d-panel-timeline), `MAX_ENVELOPE_PRESETS` (ph2d-panel-vector), `MAX_FILTER_ROWS` (ph2d-panel-vector), `MAX_MORPH_STATES` (ph2d-panel-vector), `MAX_TEXT_VARIATION_AXES` (ph2d-panel-vector), `MAX_WIDTH_PRESETS` (ph2d-panel-vector), `TexPatKnob` (ph2d-panel-vector), `VECTOR_ARRANGE_DUPLICATE` (ph2d-panel-vector), `VECTOR_ARRANGE_Z` (ph2d-panel-vector), `VECTOR_BLEND_EXPAND` (ph2d-panel-vector), `VECTOR_BLEND_RELEASE` (ph2d-panel-vector), `VECTOR_BLEND_RESET_SPINE` (ph2d-panel-vector), `VECTOR_BLEND_RUN` (ph2d-panel-vector), `VECTOR_BLEND_STEPS` (ph2d-panel-vector), `VECTOR_BOOL_APPLY` (ph2d-panel-vector), `VECTOR_BOOL_LIVE_OFF` (ph2d-panel-vector), `VECTOR_BOOL_LIVE_ON` (ph2d-panel-vector), `VECTOR_BRUSH_PICK_SHAPE` (ph2d-panel-vector), `VECTOR_COMPOUND_RELEASE` (ph2d-panel-vector), `VECTOR_CONTOUR_ACCEL` (ph2d-panel-vector), `VECTOR_CONTOUR_ACCEL_NUM` (ph2d-panel-vector), `VECTOR_CONTOUR_ADD` (ph2d-panel-vector), `VECTOR_CONTOUR_EXPAND` (ph2d-panel-vector), `VECTOR_CONTOUR_OFFSET` (ph2d-panel-vector), `VECTOR_CONTOUR_OFFSET_NUM` (ph2d-panel-vector), `VECTOR_CONTOUR_REMOVE` (ph2d-panel-vector), `VECTOR_CONTOUR_STEPS` (ph2d-panel-vector), `VECTOR_CONTOUR_STEPS_NUM` (ph2d-panel-vector), `VECTOR_CONTOUR_TO` (ph2d-panel-vector), `VECTOR_CONVERT_TO_CURVES` (ph2d-panel-vector), `VECTOR_CUT_APPLY` (ph2d-panel-vector), `VECTOR_CUT_DISCARD` (ph2d-panel-vector), `VECTOR_ENVELOPE_BEND` (ph2d-panel-vector), `VECTOR_ENVELOPE_CLEAR_PINS` (ph2d-panel-vector), `VECTOR_ENVELOPE_EXPAND` (ph2d-panel-vector), `VECTOR_ENVELOPE_MESH` (ph2d-panel-vector), `VECTOR_ENVELOPE_PERSPECTIVE` (ph2d-panel-vector), `VECTOR_ENVELOPE_PINS` (ph2d-panel-vector), `VECTOR_ENVELOPE_RELEASE` (ph2d-panel-vector), `VECTOR_ENVELOPE_RUN` (ph2d-panel-vector), `VECTOR_EXPAND_OFFSET` (ph2d-panel-vector), `VECTOR_EXPAND_W_END` (ph2d-panel-vector), `VECTOR_EXPAND_W_MID` (ph2d-panel-vector), `VECTOR_EXPAND_W_POS` (ph2d-panel-vector), `VECTOR_EXPAND_W_START` (ph2d-panel-vector), `VECTOR_FILL_RULE_EVENODD` (ph2d-panel-vector), `VECTOR_FILL_RULE_NONZERO` (ph2d-panel-vector), `VECTOR_FRAME_CLIP_OFF` (ph2d-panel-vector), `VECTOR_FRAME_CLIP_ON` (ph2d-panel-vector), `VECTOR_FRAME_PANEL_OFF` (ph2d-panel-vector), `VECTOR_FRAME_PANEL_ON` (ph2d-panel-vector), `VECTOR_GRAD_ADD_POINT` (ph2d-panel-vector), `VECTOR_GRAD_ADD_STOP` (ph2d-panel-vector), `VECTOR_GRAD_ANGLE` (ph2d-panel-vector), `VECTOR_GRAD_INFLUENCE` (ph2d-panel-vector), `VECTOR_GRAD_JITTER` (ph2d-panel-vector), `VECTOR_GRAD_REMOVE_POINT` (ph2d-panel-vector), `VECTOR_GRAD_REMOVE_STOP` (ph2d-panel-vector), `VECTOR_MORPH_PREVIEW` (ph2d-panel-vector), `VECTOR_MORPH_RUN` (ph2d-panel-vector), `VECTOR_MORPH_T` (ph2d-panel-vector), `VECTOR_OBJ_BLEND` (ph2d-panel-vector), `VECTOR_OBJ_OPACITY` (ph2d-panel-vector), `VECTOR_PAINT_BLEND` (ph2d-panel-vector), `VECTOR_PAINT_DILATE` (ph2d-panel-vector), `VECTOR_PAINT_DX` (ph2d-panel-vector), `VECTOR_PAINT_DY` (ph2d-panel-vector), `VECTOR_PAINT_OPACITY` (ph2d-panel-vector), `VECTOR_PAINT_WIDTH` (ph2d-panel-vector), `VECTOR_PATH_CLOSE` (ph2d-panel-vector), `VECTOR_PATH_JOIN` (ph2d-panel-vector), `VECTOR_PATH_REVERSE` (ph2d-panel-vector), `VECTOR_PATH_WELD` (ph2d-panel-vector), `VECTOR_PATTERNPATH_DETACH` (ph2d-panel-vector), `VECTOR_PATTERNPATH_END` (ph2d-panel-vector), `VECTOR_PATTERNPATH_FLIP` (ph2d-panel-vector), `VECTOR_PATTERNPATH_FLIP_OFF` (ph2d-panel-vector), `VECTOR_PATTERNPATH_LINK` (ph2d-panel-vector), `VECTOR_PATTERNPATH_OFFSET` (ph2d-panel-vector), `VECTOR_PATTERNPATH_PICK` (ph2d-panel-vector), `VECTOR_PATTERNPATH_ROTATION` (ph2d-panel-vector), `VECTOR_PATTERNPATH_SLIDE` (ph2d-panel-vector), `VECTOR_PATTERNPATH_SPACING` (ph2d-panel-vector), `VECTOR_PATTERNPATH_START` (ph2d-panel-vector), `VECTOR_PIVOT_EDIT` (ph2d-panel-vector), `VECTOR_RULERS_OFF` (ph2d-panel-vector), `VECTOR_RULERS_ON` (ph2d-panel-vector), `VECTOR_SNAP_CROSS_OFF` (ph2d-panel-vector), `VECTOR_SNAP_CROSS_ON` (ph2d-panel-vector), `VECTOR_SNAP_GUIDES_OFF` (ph2d-panel-vector), `VECTOR_SNAP_GUIDES_ON` (ph2d-panel-vector), `VECTOR_SNAP_OFF` (ph2d-panel-vector), `VECTOR_SNAP_PATH_OFF` (ph2d-panel-vector), `VECTOR_SNAP_PATH_ON` (ph2d-panel-vector), `VECTOR_STATE_DAMPING` (ph2d-panel-vector), `VECTOR_STATE_DURATION` (ph2d-panel-vector), `VECTOR_STATE_MOVE_ALL` (ph2d-panel-vector), `VECTOR_STATE_PREVIEW` (ph2d-panel-vector), `VECTOR_STATE_SPRING` (ph2d-panel-vector), `VECTOR_STATE_STIFFNESS` (ph2d-panel-vector), `VECTOR_STROKE_PRESENT` (ph2d-panel-vector), `VECTOR_TEXTPATH_DETACH` (ph2d-panel-vector), `VECTOR_TEXTPATH_FLIP` (ph2d-panel-vector), `VECTOR_TEXTPATH_FLIP_OFF` (ph2d-panel-vector), `VECTOR_TEXTPATH_LINK` (ph2d-panel-vector), `VECTOR_TEXTPATH_OFFSET` (ph2d-panel-vector), `VECTOR_TEXTPATH_PICK` (ph2d-panel-vector), `VECTOR_TEXT_ALIGN_CENTER` (ph2d-panel-vector), `VECTOR_TEXT_ALIGN_LEFT` (ph2d-panel-vector), `VECTOR_TEXT_ALIGN_RIGHT` (ph2d-panel-vector), `VECTOR_TEXT_FONT_DD` (ph2d-panel-vector), `VECTOR_TEXT_FONT_IMPORT` (ph2d-panel-vector), `VECTOR_TEXT_FONT_NEXT` (ph2d-panel-vector), `VECTOR_TEXT_FONT_PREV` (ph2d-panel-vector), `VECTOR_TEXT_LINE_HEIGHT` (ph2d-panel-vector), `VECTOR_TEXT_SIZE` (ph2d-panel-vector), `VECTOR_TEXT_TRACKING` (ph2d-panel-vector), `VECTOR_TEXT_WEIGHT` (ph2d-panel-vector), `VECTOR_TEXT_WRAP_AUTO` (ph2d-panel-vector), `VECTOR_TEXT_WRAP_FIXED` (ph2d-panel-vector), `VECTOR_TEXT_WRAP_W` (ph2d-panel-vector), `VECTOR_TRANSFORM_R` (ph2d-panel-vector), `VECTOR_TRANSFORM_RESIZE_BOX` (ph2d-panel-vector), `VECTOR_VERT_AVERAGE` (ph2d-panel-vector), `VECTOR_VERT_DELETE` (ph2d-panel-vector), `VECTOR_VERT_SEL_SAME` (ph2d-panel-vector), `VECTOR_VERT_SEL_SUBPATH` (ph2d-panel-vector), `VECTOR_VERT_X` (ph2d-panel-vector), `VECTOR_VERT_Y` (ph2d-panel-vector), `filter_ramp_id` (ph2d-panel-vector), `vector_envelope_preset_id` (ph2d-panel-vector), `vector_text_axis_id` (ph2d-panel-vector), `vector_width_preset_id` (ph2d-panel-vector), `VECTOR_SYM_APPLY` (ph2d-tool-vector)
- `shells/desktop/src/vec_layout_edit.rs` — 33: `VECTOR_LAYOUT_ALIGN_CENTER` (ph2d-panel-vector), `VECTOR_LAYOUT_ALIGN_END` (ph2d-panel-vector), `VECTOR_LAYOUT_ALIGN_START` (ph2d-panel-vector), `VECTOR_LAYOUT_ALIGN_STRETCH` (ph2d-panel-vector), `VECTOR_LAYOUT_COLUMNS` (ph2d-panel-vector), `VECTOR_LAYOUT_DIR_COL` (ph2d-panel-vector), `VECTOR_LAYOUT_DIR_GRID` (ph2d-panel-vector), `VECTOR_LAYOUT_DIR_OFF` (ph2d-panel-vector), `VECTOR_LAYOUT_DIR_ROW` (ph2d-panel-vector), `VECTOR_LAYOUT_DIR_WRAP` (ph2d-panel-vector), `VECTOR_LAYOUT_GAP_CROSS` (ph2d-panel-vector), `VECTOR_LAYOUT_GAP_MAIN` (ph2d-panel-vector), `VECTOR_LAYOUT_ITEM_ABSOLUTE` (ph2d-panel-vector), `VECTOR_LAYOUT_ITEM_GROW` (ph2d-panel-vector), `VECTOR_LAYOUT_ITEM_SHRINK` (ph2d-panel-vector), `VECTOR_LAYOUT_JUSTIFY_AROUND` (ph2d-panel-vector), `VECTOR_LAYOUT_JUSTIFY_BETWEEN` (ph2d-panel-vector), `VECTOR_LAYOUT_JUSTIFY_CENTER` (ph2d-panel-vector), `VECTOR_LAYOUT_JUSTIFY_END` (ph2d-panel-vector), `VECTOR_LAYOUT_JUSTIFY_START` (ph2d-panel-vector), `VECTOR_LAYOUT_MAX_H` (ph2d-panel-vector), `VECTOR_LAYOUT_MAX_W` (ph2d-panel-vector), `VECTOR_LAYOUT_MIN_H` (ph2d-panel-vector), `VECTOR_LAYOUT_MIN_W` (ph2d-panel-vector), `VECTOR_LAYOUT_PAD_ALL` (ph2d-panel-vector), `VECTOR_LAYOUT_PAD_B` (ph2d-panel-vector), `VECTOR_LAYOUT_PAD_L` (ph2d-panel-vector), `VECTOR_LAYOUT_PAD_R` (ph2d-panel-vector), `VECTOR_LAYOUT_PAD_T` (ph2d-panel-vector), `VECTOR_LAYOUT_SIZE_H_FIXED` (ph2d-panel-vector), `VECTOR_LAYOUT_SIZE_H_HUG` (ph2d-panel-vector), `VECTOR_LAYOUT_SIZE_W_FIXED` (ph2d-panel-vector), `VECTOR_LAYOUT_SIZE_W_HUG` (ph2d-panel-vector)
- `shells/desktop/src/render_loop/timeline_bridge_tests.rs` — 15: `TIMELINE_AUTOKEY` (ph2d-panel-timeline), `TIMELINE_CLOSE` (ph2d-panel-timeline), `TIMELINE_FRAME_NUM` (ph2d-panel-timeline), `TIMELINE_GO_END` (ph2d-panel-timeline), `TIMELINE_GO_START` (ph2d-panel-timeline), `TIMELINE_LOOP` (ph2d-panel-timeline), `TIMELINE_MOTION_PATH` (ph2d-panel-timeline), `TIMELINE_NEXT_FRAME` (ph2d-panel-timeline), `TIMELINE_PHYSICS` (ph2d-panel-timeline), `TIMELINE_PLAY` (ph2d-panel-timeline), `TIMELINE_PREV_FRAME` (ph2d-panel-timeline), `TIMELINE_RECORD` (ph2d-panel-timeline), `TIMELINE_RULER` (ph2d-panel-timeline), `TIMELINE_SNAP` (ph2d-panel-timeline), `TIMELINE_TIME_NUM` (ph2d-panel-timeline)
- `shells/desktop/src/render_loop/timeline_bridge.rs` — 15: `TIMELINE_ADD_MARKER` (ph2d-panel-timeline), `TIMELINE_AUTOKEY` (ph2d-panel-timeline), `TIMELINE_FRAME_NUM` (ph2d-panel-timeline), `TIMELINE_GO_END` (ph2d-panel-timeline), `TIMELINE_GO_START` (ph2d-panel-timeline), `TIMELINE_LOOP` (ph2d-panel-timeline), `TIMELINE_NEXT_FRAME` (ph2d-panel-timeline), `TIMELINE_PHYSICS` (ph2d-panel-timeline), `TIMELINE_PINGPONG` (ph2d-panel-timeline), `TIMELINE_PLAY` (ph2d-panel-timeline), `TIMELINE_PREV_FRAME` (ph2d-panel-timeline), `TIMELINE_RECORD` (ph2d-panel-timeline), `TIMELINE_RULER` (ph2d-panel-timeline), `TIMELINE_SNAP` (ph2d-panel-timeline), `TIMELINE_TIME_NUM` (ph2d-panel-timeline)
- `shells/desktop/src/vec_anchor_edit.rs` — 8: `VECTOR_ANCHOR_H_CENTER` (ph2d-panel-vector), `VECTOR_ANCHOR_H_END` (ph2d-panel-vector), `VECTOR_ANCHOR_H_START` (ph2d-panel-vector), `VECTOR_ANCHOR_H_STRETCH` (ph2d-panel-vector), `VECTOR_ANCHOR_V_CENTER` (ph2d-panel-vector), `VECTOR_ANCHOR_V_END` (ph2d-panel-vector), `VECTOR_ANCHOR_V_START` (ph2d-panel-vector), `VECTOR_ANCHOR_V_STRETCH` (ph2d-panel-vector)
- `shells/desktop/src/render_loop/timeline_bridge_container_tests.rs` — 4: `TIMELINE_GO_END` (ph2d-panel-timeline), `TIMELINE_GO_START` (ph2d-panel-timeline), `TIMELINE_LOOP` (ph2d-panel-timeline), `TIMELINE_PINGPONG` (ph2d-panel-timeline)
- `shells/desktop/src/vec_layout_edit_tests.rs` — 4: `VECTOR_FRAME_CLIP_ON` (ph2d-panel-vector), `VECTOR_LAYOUT_DIR_ROW` (ph2d-panel-vector), `VECTOR_LAYOUT_DIR_WRAP` (ph2d-panel-vector), `VECTOR_LAYOUT_JUSTIFY_BETWEEN` (ph2d-panel-vector)
- `shells/desktop/src/render_loop/audio_overlay.rs` — 4: `AUDIO_OVERLAY_DRAG_HANDLE` (ph2d-panel-audio-editor), `AUDIO_OVERLAY_PANEL` (ph2d-panel-audio-editor), `AUDIO_OVERLAY_RESIZE_HANDLE` (ph2d-panel-audio-editor), `AUDIO_OVERLAY_RESIZE_HANDLE_BL` (ph2d-panel-audio-editor)
- `shells/desktop/src/render_loop/inspector_strategy.rs` — 2: `INSP_RENDER_STRATEGY_HANDPACKED` (ph2d-panel-inspector), `INSP_RENDER_STRATEGY_INDIVIDUAL` (ph2d-panel-inspector)
- `shells/desktop/src/render_loop/inspector_action_tests.rs` — 2: `INSP_ACTION_ROW` (ph2d-panel-inspector), `INSP_ACTION_VERB` (ph2d-panel-inspector)
- `shells/desktop/src/render_loop/authored_intents_tests.rs` — 1: `authored_row_id` (ph2d-panel-authored)
- `shells/desktop/src/render_loop/tokens_bridge.rs` — 1: `tokens_swatch_id` (ph2d-panel-tokens)
- `shells/desktop/src/render_loop/tokens_bridge_tests.rs` — 1: `tokens_swatch_id` (ph2d-panel-tokens)
- `shells/desktop/src/render_loop/sim_extract_sheet.rs` — 1: `INSP_SHEET_PREVIEW` (ph2d-panel-inspector)
- `shells/desktop/src/render_loop/inspector_instance.rs` — 1: `MAX_INSTANCE_APPLY_LEVELS` (ph2d-panel-inspector)
- `shells/desktop/src/render_loop/inspector_timer_tests.rs` — 1: `INSP_TIMER_ROW` (ph2d-panel-inspector)

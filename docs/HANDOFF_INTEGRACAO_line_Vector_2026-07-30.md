# Handoff de INTEGRAÇÃO — `line/Vector` → `main` (2026-07-30)

> **Para o agente integrador.** Este documento é o **consolidado da linha inteira**: 57 commits,
> duas metades (o **plano 24** — FX raster — e o **plano 25** — ferramentas de desenho). A metade do
> FX já tem **nove handoffs por-wave** (listados na §2); esta é a única fonte para a metade do
> plano 25, e é a única que traz a **soma** dos números que colidem entre linhas.
>
> **Fork point:** `7ec917506` (2026-07-27). **Tip:** `f68ce2c7e`.
> **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` · branch `line/Vector`.
>
> ⚠️ **Todos os smokes desta linha foram APROVADOS pelo Enio.** O último (`=43`, W3a+W3b) em
> 2026-07-30: *"smoke OK"*.

---

## §1 — O que integrar, em uma frase

O módulo Vector ganhou **a pilha de FX raster completa** (plano 24: mistura por degrau,
turbulência, Grow/Shrink, Color Adjust, Duotone, Luma-to-Alpha, Gradient Map, o atlas de raster) e
**as ferramentas de desenho** (plano 25: o LÁPIS, a LARGURA VIVA + o Width Tool + os perfis salvos,
e o ALCANCE DO NÓ + a escala da seleção).

---

## §2 — Os números que COLIDEM entre linhas (leia isto primeiro)

⚠️ **`PROJECT_SCHEMA` 37 → 38.** UM bump, e ele é do **`VecFilter`** (a pilha de FX raster, plano
24). O resto da linha **não bumpa schema nenhum**.

> **O valor se CONTA, não se escolhe.** Esta linha e a `line/physics` já colidiram **duas vezes**
> no mesmo número (30 em 25/07 · 32/33/34 em 27/07), e as duas vezes o valor certo não estava em
> nenhum dos dois lados. Antes de fundir: leia o `PROJECT_SCHEMA` do `main` **do dia** e some os
> bumps de cada linha que entrar nesta janela. A narrativa da escada vive em
> `shells/desktop/src/project.rs` + `project_tests.rs`, que são a fonte.

⚠️ **`VEC_SCENE_SCHEMA_VERSION` fica em 13** (intocado).

⚠️ **O contador de componentes do ECS é TRÊS, não um** — a família que já ficou **vermelho-latente**
duas vezes nesta linha:

| lugar | main | linha |
|---|---|---|
| `crates/ph2d-ecs/src/scene/registry.rs` | 38 | **39** |
| `crates/ph2d-render/src/registry.rs` (ecs + `Sprite`) | 39 | **40** |
| `crates/ph2d-script/src/registry.rs` (ecs + `LuauScript`) | 39 | **40** |

Cada um roda **só na suíte da própria crate**, então dois merges verdes deixam o workspace
vermelho. Os componentes novos da linha são **`VecFilter`** (plano 24) e **`VecStrokeProfile`**
(ADR-0148).

**ADR novo: um só** — [`0148-vector-live-width-profile-is-an-ecs-component-and-one-baker-serves-preview-and-apply.md`](architecture/decisions/0148-vector-live-width-profile-is-an-ecs-component-and-one-baker-serves-preview-and-apply.md).
⚠️ **E ele NASCEU como 0145 — renumerado para 0148 NA INTEGRAÇÃO de 2026-07-30** (a 5ª vez no
repo): a `line/Painter` reivindicou **0145 · 0146 · 0147** na mesma janela e chegou ao `main`
primeiro, então ela ficou com os três. *Um número de ADR escolhido numa linha paralela é
PROVISÓRIO* — e este é o registro de que o provisório se realizou.

**Handoffs por-wave da metade do FX** (histórico; o conteúdo técnico deles continua válido):

- `HANDOFF_INTEGRACAO_line_Vector_fx_raster_2026-07-26.md`
- `HANDOFF_INTEGRACAO_line_Vector_stroked_silhouette_2026-07-27.md`
- `HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md`
- `HANDOFF_INTEGRACAO_line_Vector_turbulence_2026-07-28.md`
- `HANDOFF_INTEGRACAO_line_Vector_morphology_2026-07-28.md`
- `HANDOFF_INTEGRACAO_line_Vector_colour_adjust_2026-07-28.md`
- `HANDOFF_INTEGRACAO_line_Vector_duotone_2026-07-29.md`
- `HANDOFF_INTEGRACAO_line_Vector_gradient_map_2026-07-29.md`
- `HANDOFF_INTEGRACAO_line_Vector_atlas_2026-07-29.md`

---

## §3 — Contrato congelado (§6): INTACTO, e a prova é por grep

```
git diff --stat main..HEAD -- crates/ph2d-vector-doc crates/ph2d-vector-traits \
                              crates/ph2d-nodegraph crates/ph2d-tool-registry/src/tool.rs
→ vazio
```

E os três gates correm verdes na linha:

| gate | resultado |
|---|---|
| `ph2d-nodegraph::architecture_contract_surface` | 3 passed |
| `ph2d-editor-core::architecture_tool_contract_surface` | 4 passed |
| `ph2d-vector-doc::architecture_vector_contract_surface` | 11 passed |

`NodeOp=2` · `OpResolver=1` · `NodeManifest=8` · `Tool=12` · `RasterEditTool=5` ·
`CanvasPaintTool=1` · `PanelEvent=4` — todos intactos.

---

## §4 — Superfície nova (o que outra linha pode colidir)

**Crate nova: `ph2d-stroke-width`** — leaf, deps `serde` só. Ela é a dona do que um **perfil de
largura** é (`WidthStop`/`WidthStops`/`WidthProfile`/`WidthPreset`/`PRESETS`), e a `ph2d-vec-scene`
a re-exporta. ⚠️ **Nasceu para isolamento** (ADR-0107): é módulo folha, sem dependência de
ninguém do módulo, então uma linha que precise de perfil de largura a consome sem tocar em nada.

**Arestas de Cargo novas:**

| crate | dep nova | porquê |
|---|---|---|
| `ph2d-vec-scene` | `ph2d-stroke-width` | re-exporta o tipo |
| `ph2d-tool-vector` | `ph2d-vec-edit` | o `WidthSource` do lápis (W1d) |
| `ph2d-panel-vector` | `ph2d-vec-edit` | o mesmo enum, para pintar os chips |
| `ph2d-ecs` | `ph2d-stroke-width` | o `VecStrokeProfile` carrega a lista |
| `shells/desktop` | `ph2d-painter-brush`, `ph2d-vector-doc` | (plano 24, ver o handoff do atlas) |

**API pública nova** (aditiva; nada removido):

- `ph2d_vec_scene::{dissolve_vertex, reshape_segment, point_on_segment}` (W3a)
- `ph2d_vec_scene::{WidthPreset, WIDTH_PRESETS}` (W2b)
- `ph2d_vec_edit::pencil_width::{WidthSource, PenDynamics, width_stops, …}` (W1d)
- `ph2d_vec_edit::PenTool::{box_select_with, select_all_verts, select_subpath_verts,
  select_verts_of_same_kind, step_vert_selection, width_stops}` (W1d/W3b)
- `ph2d_vec_boolean::power_stroke` passou a receber `&WidthStops` (era o preset de 4 números)
- `ph2d_vec_render::draw_width_handle` (W2a)
- `ph2d_tool_vector::params::{preset_tracks, active_preset, WPROFILE_DEFAULT}` (W2b)

⚠️ **`ph2d_vec_render::dispatch` NÃO mudou de assinatura** nesta metade da linha (a `LiveGeometry`
já a tinha). O mapa vivo do perfil é FUNDIDO no mesmo argumento — ver §6.

**Ids novos:** um módulo irmão **`ids/chrome/vector_width.rs`** (`MAX_WIDTH_PRESETS = 8` +
`vector_width_preset_id`), mais `vector_pencil.rs` (a seção Pencil) e o crescimento de
`vector_filters.rs` (plano 24). `node_id_collisions` verde.

**i18n:** 10 chaves novas (`panel.vector.section.pencil`, `panel.vector.pencil.width.*`,
`panel.vector.width.preset.*`, `panel.vector.mode.{pencil,width}`).

---

## §5 — O que a metade do PLANO 25 entrega (o que estes handoffs ainda não cobriam)

### W0 — higiene (4 correções, todas com gate)

O memo do FX raster não sabia o que era DESENHADO · o nó de uma Live Shape era descartado em
silêncio · um COZIDO e uma CONSTRUÇÃO por forma (re-encode de 10k formas 1,323 → **0,901 ms**) ·
seleção MISTA de nós não acende chip nenhum.

### W1 — O LÁPIS (`PH2D_BUILD_SMOKE=40`)

Mão livre: gesto → decimador (RDP) → ajuste de **Hobby** que PASSA pelos nós. Dois knobs que são
duas perguntas independentes — **Fidelity** (a tolerância na SAÍDA) e **Stabilizer** (o tremor na
ENTRADA; o decimador preserva extremos locais de propósito, e um tremor é um extremo local).

**W1d — a FONTE da largura** (decisão do Enio): `Uniform` · `Speed` · `Pen`. ⚠️ **`Pen` é oferecida
e hoje não chega** — a shell não recebe pressão de dispositivo nenhum; o rótulo o diz em vez de
deixar o artista descobrir. O caminho do tablet (casar `WindowEvent::Touch` e carregar `force` no
`PointerEvent`) é **INPUT de shell**, afeta o Flip igual, e hoje custa **uma função**
(`App::pointer_dynamics`) — nomeado, não construído.

⚠️ `PenDynamics` **não tem `Default`**: esquecer de passar a dinâmica é erro de COMPILAÇÃO, o
idioma do `ShapeFrame` do Painter.

### W1c/W2 — A LARGURA VIVA (ADR-0148) — `=41`, `=42`

O perfil de largura é um **componente ECS** (`VecStrokeProfile`), não um campo do `StrokeSpec` — um
campo bumparia `VEC_SCENE_SCHEMA_VERSION` **e** `PROJECT_SCHEMA`, e um schema divergente **recusa
todo projeto salvo**. **UMA representação** (a lista de paradas; o preset de 4 números é uma FACE
dela, e a redução é EXATA bit a bit), **UM motor** (`power_stroke` serve preview e Apply — duas
portas fariam a forma SALTAR no clique, o defeito que o ADR-0128 pagou cinco vezes).

**W2a — o Width Tool** (12º modo): alças de largura na curva. **W2b — os perfis salvos**:
`Uniform · Taper · Both · Bulge`, a tabela em `ph2d_stroke_width::PRESETS`, com os multiplicadores
**medidos** em cinco pontos do arco.

⚠️ **Duas correções pós-smoke, as duas do Enio, e a segunda mudou o CRITÉRIO:**
1. a ficha da alça ficava na **borda da fita** (`meia-largura × multiplicador` da curva) e num
   grampo de braços a `0,30` um multiplicador de `3,75` a punha em `y = 0,300` — **sobre o braço
   vizinho**. A ficha mudou-se para a **curva**, com uma haste até a borda;
2. a busca por alça era no **PLANO**, o que é **indecidível** entre linhas mais juntas que o raio.
   Agora há **uma** pergunta de proximidade (`closest_arc`, que escolhe o ramo) e a segunda corre
   em **ARCO** sobre esse ramo — duas linhas que se cruzam estão a milímetros no plano e a **meio
   traço** uma da outra ao longo do percurso.

### W3 — O ALCANCE DO NÓ + A ESCALA DA SELEÇÃO — `=43`

**W3a:** `Delete` **preserva a forma** (porta única `dissolve_vertex`, partilhada com o Simplify —
a divergência anterior aparecia como *"o Simplify preserva e o Delete não"*) · e pressionar sobre a
curva **REFORMA o segmento** em vez de inserir um nó. ⚠️ **A inserção não se perdeu: mudou de
ferramenta** (a divisão do Illustrator — a seta branca reforma, a Pen acrescenta), e há gate
exigindo que a Caneta continue a inserir.

**W3b:** o retângulo deixou de exigir Shift (Shift passa a SOMAR; clique nu desseleciona) · a
preferência pelo caminho selecionado deixou de ser incondicional · `Tab`/`Shift+Tab` · `Ctrl+A` ·
os botões **Select Subpath** e **Select Same** na seção Vertex.

---

## §6 — Os pontos de fusão que exigem CUIDADO

1. **`render_loop/mod.rs`** — a linha acrescenta dois blocos e três drenos de clique:
   - o bloco `// ── A LARGURA VIVA (ADR-0148)` (o espelho da seleção + o armamento por arrasto + o
     catálogo de perfis), que corre **depois do `vec_entities::sync`** e **antes do desenho**;
   - `self.profile_live.recook(...)` e a fusão do mapa vivo no argumento do
     `ph2d_vec_render::dispatch` (ele já recebia `&LiveGeometry` do offset — a linha **acrescenta**
     uma fonte ao mapa, não muda a assinatura);
   - os drenos `VECTOR_VERT_SEL_SUBPATH` / `VECTOR_VERT_SEL_SAME` / `vector_width_preset_id(i)`.

   ⚠️ **Quatro arch-gates leem este arquivo por CONTEÚDO** e falham alto se a ordem se perder:
   `the_width_sliders_author_the_live_profile.rs` (8 asserções) e
   `the_node_selection_scale_is_wired.rs` (4).

2. **`input_dispatch.rs`** — o ramo do marquee no modo Node corre **ANTES** do `on_press_node`
   (ele desseleciona quando não acerta nada). O `press`/`drag`/`release` do Width Tool tem arm
   próprio, **antes** da cadeia de modo.

3. **`input_dispatch/keyboard.rs`** — o bloco `// **A ESCALA DA SELEÇÃO DE NÓS**` entra **antes**
   do bloco das setas. ⚠️ Este arquivo **já cruzou o cap de 600 LOC numa integração anterior por
   soma de duas linhas** (`line/anim` +9 e `line/physics` +13 sobre 582): **meça-o na árvore
   combinada**, não em cada lado.

4. **`ph2d-i18n/src/lib.rs`** — 10 chaves; conflito textual provável e trivial (só ADICIONE).

5. **`ids/chrome/mod.rs`** — dois `mod`/`pub use` novos, em ordem alfabética (o `rustfmt` reordena).

---

## §7 — O gate de fechamento (rode NA ÁRVORE COMBINADA)

```
cd <árvore combinada>
cargo fmt --check --all
cargo clippy --workspace --all-targets            # -D warnings
cargo machete
cargo test --workspace
```

E, **explicitamente**, os que a varredura por-crate não alcança:

| gate | onde | porquê |
|---|---|---|
| `architecture_workspace_file_loc_cap` | `ph2d-editor-core` | só corre isolado |
| `file_loc_caps` | `shells/desktop/tests/` | **o cap da shell é OUTRO gate** (600), e uma linha sozinha não o cruza |
| `architecture_panel_loc_cap` | `ph2d-editor-core` | idem |
| `node_id_collisions` | `ph2d-editor-core` | os ids derivados por chave em runtime |
| `architecture_panel_wiring_parity` | `ph2d-editor-core` | pintado ≠ registrado |
| `hr12_widgets_a11y` | `ph2d-editor-core` | a delegação de a11y dos arquivos de painel novos |
| `architecture_adr_numbers_are_unique` | `ph2d-editor-core` | o **0148** desta linha (nasceu 0145) |
| `arch_safe_clamp_only` | `ph2d-editor-core` | já ficou vermelho-latente noutra linha |

⚠️ **A lição estrutural que esta linha pagou em 2026-07-23:** os gates que moram em
`shells/desktop/tests/` **só correm na varredura impactada**, e um fechamento por `cargo test -p`
por crate **não os alcança** — dois arch-gates chegaram vermelhos ao tip por isso. Rode-os por nome.

---

## §8 — Os smokes (todos APROVADOS; re-rodar na árvore combinada é barato)

Todos com `env PH2D_BUILD_SMOKE=<n> cargo run -p ph2d-host-desktop --release`.
Cada cena **imprime o que montou** — se a linha `[smoke] …` não aparecer, pare.

| cena | o quê |
|---|---|
| `=20`..`=24`, `=34`..`=39` | a metade do **FX raster** (plano 24; detalhe nos 9 handoffs da §2) |
| **`=40`** | o **LÁPIS** (W1) — inclui o passo 10, a FONTE da largura (W1d) |
| **`=41`** | a **LARGURA VIVA** (W1c) — os quatro sliders autoram; passos 10-14 são os perfis salvos (W2b) |
| **`=42`** | o **WIDTH TOOL** (W2a) — passos 11-13 são as duas correções pós-smoke |
| **`=43`** | o **ALCANCE DO NÓ** (W3a) + a **ESCALA DA SELEÇÃO** (W3b, passos 9-15) |

---

## §9 — Aberto e NOMEADO (não é dívida escondida)

- **O caminho do tablet** — `Pen` como fonte de largura é oferecida e não chega. É INPUT de shell
  (casar `WindowEvent::Touch`, carregar `force`), afeta o Flip igual, e custa **uma função**.
- **O lasso** — a caixa cobre o caso comum; o laço quer captura de polígono + overlay próprio.
- **X/Y numérico do nó** — é *precisão*, e cai na **W6** do plano 25.
- **Editar nós de VÁRIAS formas** — ausência **por construção** (`selected_verts` pertence a um
  `selected` único). Tamanho **G**, nomeada no plano.
- **O ajuste do `dissolve_vertex` está a 12% do piso** alcançável com as tangentes preservadas, e
  comprar os 12% custa um fitter **iterativo** — **medido e recusado**, com a tabela no plano 25 §6.
  ⚠️ Um LSQ **ingênuo** é PIOR que o que shipa (0,1675 contra 0,0782); está escrito lá para ninguém
  "melhorar" o fit por esse caminho.
- **W4 (OS CORTES) · W5 (O PATHFINDER BARATO) · W6 (PRECISÃO)** — as três waves restantes do plano
  25, não começadas.

---

## §10 — Estado da linha

**FECHADA.** Nada em voo, working tree limpa, todos os smokes aprovados. A linha **não integra nem
faz ship** — isso é ordem explícita do Enio, executada por um agente integrador dedicado
(DIRETRIZ §1.5.3–1.5.4).

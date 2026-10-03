# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-03 — o menu Add de objectos (spec/06 F0 + F1)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md):
> os 18 commits da escala continuam descritos lá (§3 superfície, §4 gates). Este cobre os commits
> DEPOIS do fecho dela e refaz o gate sobre o diff ACUMULADO desde a base.
>
> **Ordem do dono (03/10):** *«vamos manter nessa linha […] Não vamos integrar agora.»* e, depois
> da F0, *«pode seguir implementando»*.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **26 `+`** (com este ficheiro), nenhum integrado. Rebase: no-op (o `main`
  não andou).

## §0 — Para o `CLAUDE.md` §5.1 (UI/UX)

Trocar o link do «Último:» por este ficheiro. A frase do módulo não muda.

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `161e4cfd5` | o `+` da Hierarquia e o `Shift+A` abrem o **menu Add de objectos** (o `command_palette`, o mesmo modal do *Add shape…*); cada família declara e cria os seus |
| `3c54f6c73` | `PH2D_OBJECT_ADD_SMOKE=1` abre o menu uma vez (para a foto); a F0 medida no `spec/06` §2.1 |
| `dc7117546` | os dois vermelhos do gate batched, curados (ver §4) |
| `d5119f595` | o gate do Jogo deixa de ser circular (a M4 sobrevivia) e o gate do desfazer |

**O menu (foto: 16 itens, 4 grupos):** 2D — `Image…` (abre o diálogo do `Ctrl+N`, que fica como
atalho), `Vector Rectangle/Ellipse/Polygon/Star`, `Flip Drawing` · 3D — `Model Solid`,
`Sculpt Sphere/Cube/Cylinder/Torus` · Game — `Camera`, `Physics Body`, `Sound`, `HUD` · Empty.

**O desenho (desvio documentado no spec/06 §4 F1):**
- `ph2d_editor_core::object_add` — `AddGroup`, `AddEntry { key: TextKey, group }` (id = hash da
  chave i18n), `build(families, why_not)`, `entry_of_pick`, `CORE = [IMAGE, EMPTY]`,
  `take_smoke_open`. O que não pode nascer agora aparece DEPOIS, com a razão no rótulo.
- Cada família: `object_add.rs` com `ENTRIES` + `add(entry, <os seus recursos>) -> Option<Result<…>>`
  (`None` = não é minha). ⛔ **Não há `spawn: fn(&mut World)` comum:** os recursos diferem
  (`VecScene`+`ShapeTool`, `FlipDoc`, a cena GPU do Sculpt), e uma assinatura comum mentiria.
  - **Vetor:** pela `ShapeTool` (press/drag/release sobre um quadrado = ¼ da altura visível) +
    `make_committed_shape_live` — a forma do menu nasce VIVA, igual à desenhada.
  - **Flip:** `push_object` + UMA camada + `flip_entities::sync`. ⚠️ Não havia porta de produto
    para um objecto Flip (o boot só semeia sob `PH2D_FLIP_DEMO`).
  - **Sculpt:** `add_primitive` na cena, ou cena nova pela porta única `mode::new_scene`
    (extraída do pill); a entidade vem da sincronia (`sculpt3d_entities_sync` corrida já).
  - **Model:** o MESMO nascimento do painel (`sync_scene_and_birth` com semente = união + a
    esfera do catálogo) e `ask_open_panel`. ⛔ **Um por cena:** a ponte coze só a 1.ª raiz
    `FieldObject`; o segundo aparece com a razão.
  - **Jogo:** vazio + componente pela porta do `+` do Inspector (`attach_by_name`, com cascata e
    sementes); se o componente não entra, o vazio sai.
- `render_loop/fase_object_add.rs` (shell, composição): `FAMILIES` (const, com `cfg(sculpt3d)`),
  abre, drena condicionalmente, despacha, selecciona o objecto novo e avisa.

## §2 — Foundational tocado (aditivo) e contratos

- `ph2d-editor-core`: módulo NOVO `object_add` (+ `pub mod` em `lib.rs`). `hero.rs` intocado
  (**698/700**), `hero/paint.rs` **700/700**.
- `ph2d-component-desc`: ⚠️ **`ObjectKind::Sculpt3D::marker()` passou de `BakedForm` a
  `Sculpt3dPieceRef`** (o `BakedForm` vai numa SPRITE assada, que é Image).
- `ph2d-i18n`: tabela NOVA `object_add.rs` + 1 linha na cadeia do `tr_ingles`.
- Contratos congelados (§6): **nenhum encostado**.

## §3 — Superfície de colisão (para o integrador)

**Falha ALTO noutra linha (não compila):**
- `shells/desktop/src/render_loop/hierarchy_add_root.rs` **APAGADO**: `spawn_empty_root` vive em
  `ph2d_app_components::object_add::spawn_empty_root(sim, name)` (ganhou o `name`).
- `HierarchyIntents` perdeu `add_root`; a fn do `hierarchy.rs` perdeu o parâmetro `add_root`.
- `kind_of` (`component_attach.rs`) passou a `pub` e a `#[must_use]`.
- `ph2d_app_sculpt3d::mode::new_scene` (nova, `pub(crate)`); o `apply_toggle` chama-a.

**Funde LIMPO e REPROVA depois:**
- O censo `every_non_literal_hash_is_named` ganhou a entrada `object_add.rs :: id`; uma linha que
  mova a fn tem de a reescrever.
- O gate textual `entering_with_no_scene_creates_one_from_the_one_primitive_door` lê agora
  `new_scene(device, size, crate::Primitive::Sphere)` e `let mesh = first.mesh();`.
- Uma linha que acrescente um consumidor do canal de pick com `take_command_pick()`
  INCONDICIONAL come os cliques do menu Add (hoje os seis consumidores perguntam «é meu?»).

**Enums, ids e chaves (append-only):**
- i18n NOVAS: `object_add.*` (21 chaves). ⛔ **APAGADAS:** `shell.hierarchy.added_empty_object` e
  `shell.hierarchy_add_root.object` — uma linha que ainda as use pinta a chave crua.
- Ids: hash de `object_add.*` (sem constantes novas em `ids`).
- Env: `PH2D_OBJECT_ADD_SMOKE=1`.

**Muda comportamento:**
- O `+` da Hierarquia já não cria um vazio direto: abre o menu (o vazio é o item `Empty`, e
  chama-se `Empty`, não `Object`).
- `Shift+A` (sem Ctrl/Alt) abre o menu em qualquer sítio onde a tecla chegue ao ramo do editor.
- ⚠️ O `kind_of` completo muda o `+` do Inspector: um objecto Flip, uma peça de escultura ou uma
  imagem pintada sem `Sprite` deixam de se ler como VAZIO, logo passam a ver os componentes
  `DRAWABLE` (blend, máscara…) que antes lhes eram negados.

**Tectos e a shell:**
- `shells/desktop`: esta onda **+319 −134**; acumulado desde a base **+480 −225** (líquido
  **+255**). A catraca `the_shell_only_shrinks` está verde. O crescimento é a fase de composição
  (`fase_object_add.rs`) e os seus 3 gates; o vazio SAIU para a família.

## §4 — Fecho: gate batched, mutação, auditoria

**Gate batched (1× sobre o diff acumulado desde `1ad60a1ce`, verificador, load 12–20):**
- `nextest-impacted`: **19 649 passaram, 2 falharam** → as duas eram desta onda e foram
  curadas em `dc7117546` (o censo de NodeId pedia a forma nova nomeada; o gate textual do pill
  lia a porta antiga). Re-corridas sozinhas: **verdes**.
- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` · `clippy --all-targets -D
  warnings` · `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh`:
  **verdes**.

**Gates novos (24 testes):** editor-core 5 · app-components 7 (inclui os 3 do vazio, mudados da
shell) · field3d 1 · vec 3 · flip 2 · sculpt3d 2 · shell 4 (inclui
`creating_then_undoing_returns_the_project_to_the_bit`: vazio, câmara, vetor, Flip e Model, sobre
um projecto NÃO vazio).

**Mutação (agente `mutacao`, controlo VERDE antes de cada uma):**

| M | gate | mutação | resultado |
|---|---|---|---|
| M1 | `every_entry_has_one_item_and_its_id_maps_back` | `entry_of_pick` acha qualquer um | VERMELHO |
| M2 | `a_blocked_entry_shows_after_the_ready_ones_with_its_reason` | `[blocked, ready]` | VERMELHO |
| M3 | `every_marker_derives_its_kind` | braço do Flip no `kind_of` → `false` | VERMELHO |
| M4 | `every_game_object_is_born_with_its_component` | o som anexa `GameCamera` | ⛔ **SOBREVIVEU** (o gate lia o `component_of`) → curado em `d5119f595` (tipo concreto) → VERMELHO |
| M5 | `the_menu_model_is_born_once_and_the_second_is_refused` | sem a guarda `why_not` no `add` | VERMELHO |
| M6 | `every_menu_shape_is_born_a_live_vector_object` | sem `make_committed_shape_live` | VERMELHO |
| M7 | `the_menu_flip_is_born_a_flip_object_with_a_layer` | sem `add_layer` | VERMELHO |
| M8 | `no_menu_id_belongs_to_another_palette` | o HUD com a chave `panel.model3d.add.sphere` | VERMELHO |
| M9 | `a_real_click_on_every_entry_reaches_the_drain` | o `apply` da paleta sem `set_command_pick` | VERMELHO |
| M10 | `creating_then_undoing_returns_the_project_to_the_bit` | o `restore` devolve a `VecScene` vazia | VERMELHO |

Controlo VERDE antes de cada uma; árvore limpa no fim.

**Auditoria (DIRETIVA §3):**

```
LENTE:  wiring (o clique até ao objecto)
CLAIM:  cada item do menu, clicado, cria o objecto do seu tipo e selecciona-o
TRAÇO:  + → HierRequest::AddRoot (panel-hierarchy, seam existente) → fase_bus_hierarchy.rs:54
        pd.add_root → fase_hero_commits → fase_object_add(open) → open_command_palette(build)
        → clique: HeroScreen::apply_event → command_palette::apply → set_command_pick
        → take_command_pick_if(entry_of_pick) → add() da família → replace_selection + toast
ASSERÇÃO-VERMELHA: a_real_click_on_every_entry_reaches_the_drain (rota, 16 ids) + o add() de cada
        família (gate por crate) + every_marker_derives_its_kind (o tipo derivado)
NÃO-CHECADO-PELA-COMPILAÇÃO: a fase em si (pede o AppGfx) — o despacho entre famílias é provado
        por partes; o Sculpt com cena NOVA pede placa (só a foto/smoke o vê)
LOC LIDAS: ~900
```

```
LENTE:  correção do tipo (o vocabulário)
CLAIM:  o objecto criado lê-se como o tipo que o menu prometeu
TRAÇO:  kind_of (component_attach.rs) ⇄ ObjectKind::marker (component-desc) ⇄ registo
ASSERÇÃO-VERMELHA: every_marker_derives_its_kind (o tipo concreto E o nome canónico do marcador
        são o mesmo componente, pelos 6) · a_painted_image_is_an_image
NÃO-CHECADO-PELA-COMPILAÇÃO: um 7.º ObjectKind sem braço no kind_of — o match é exaustivo, logo
        isto passa a ser erro de compilação (era um if/else com 3 braços)
LOC LIDAS: ~300
```

## §5 — Premissas do briefing que a medição derrubou

- *«O Model3D tem `FieldObject` por entidade»* — o explorador confirmou-o; a leitura do
  `sync_scene_and_birth` (`q.iter(world).next()`) mostrou que **só a 1.ª raiz é cozida**.
- *«O Sculpt tem peças como entidades (`SculptRowsSeen`) e `BakedForm` é o resultado assado»* —
  as peças têm entidade-espelho com `Sculpt3dPieceRef`, mas o `ObjectKind::Sculpt3D` apontava ao
  `BakedForm`: uma peça de escultura lia-se como VAZIA.
- O `kind_of` conhecia 3 dos 6 marcadores.
- O `ObjectTypeDecl` do plano (`spawn: fn(&mut World, preset)`) não serve: §1.
- O **botão direito do canvas** (escolha 3) não entrou: tem donos hoje (o vetor cancela a forma,
  o Motion e o Painter abrem menus). Vai com a F2, guardado pelo modo Object.

## §6 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Na **Hierarchy**, clicar no **+** → abre a janela **Add Object** com quatro grupos (2D, 3D,
   Game, Empty). Errado: nasce um objecto sem janela, ou a janela vem vazia.
3. Escrever `vector` na busca → só ficam as quatro formas; **Enter** cria a primeira.
4. Clicar **Vector Star** → nasce uma estrela no meio da vista, seleccionada, e a Hierarchy ganha
   uma linha. Errado: nada aparece, ou aparece invisível.
5. Com o rato no canvas, **Shift+A** → a mesma janela. Clicar **Sculpt Sphere** → abre a
   escultura com uma esfera e a Hierarchy mostra a peça.
6. **+** → **Camera** → nasce «Camera» e o Inspector mostra a secção da câmara.
7. **+** → **Model Solid** → abre o painel Model com uma esfera. Um 2.º **+** mostra
   *Model Solid — this scene already has a Model…*, e clicar nele avisa isso em vez de criar.
8. **+** → **Image…** → abre o diálogo de tamanho (o mesmo do Ctrl+N).
9. **+** → **Flip Drawing** → nasce «Flip Drawing»; com a ferramenta Flip, um traço aparece.
10. **Ctrl+Z** depois de cada criação desfaz o objecto (gateado para todos menos a escultura, que
    tem desfazer próprio).

## §7 — O que fica para a próxima janela (F2 em diante)

- **F2 — o modo** (`ObjectMode`, o selector «Object Mode ▾» no cabeçalho da área, Tab, o cadeado,
  e o botão direito do canvas guardado pelo modo Object). ⚠️ Um modo só pode aparecer quando o
  módulo dele já abre sobre a entidade (D6: modo morto ⇒ não aparece), logo a F2 e a F3 do 1.º
  módulo andam juntas. **Ordem medida da F3:** Image (o Painter já é por entidade) → Sculpt → Flip
  → Model → Vector.
- O **Flip** escreve sempre no 1.º desenho; o **Model** é um por cena — os dois são F3.
- Recomendação do plano: uma janela por fase.

## §8 — Perfil do loop (`bash scripts/agent-loop-profile.sh`, 20 sessões)

```
  ✗ paralelismo de ferramenta              1.09/passo   alvo: >= 1,5  (7% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                180   alvo: <= 800
  ✗ cargo test : cargo check               1603 : 532   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  34%   alvo: >= 80%
  ✗ contexto relido por passo (media)         571 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               61 mil   alvo: <= 80 mil
```

## §9 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
__SMOKE__
```

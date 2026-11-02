# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-11-02 — F40 a F45 (osso animável · Length que empurra · tipo por ícone/cor · Effects/Filters/Fill/Stroke no layout do Figma · UMA pilha com ordem livre)

- `line/UIUX` → `main` por `--ff-only`, **só por ordem EXPLÍCITA do Enio**. Sem push, sem ship.
- Ponta `c64df4d06e` (mais o commit deste handoff); merge-base com o main = `422fec2027` (o main estava lá em `git worktree list`, 02/11) ⇒ fast-forward possível; se o main andar, reconte pela §2.
  Delta (`422fec2027..HEAD`, 51 commits antes deste): **487 ficheiros, +16 978 −3 435**; `shells/desktop` **52 f. +909 −288** (cresceu: composição da F42/F45; `the_shell_only_shrinks` verde, mede o total).
- **Smokes do dono:** F40+F41 (3 rodadas) OK 01/11 · F42 OK · F43 «Muito bom» · F44 REPROVOU em 5 pontos → F44b aprovada («muito bom») → F44c (pedidos) → **F45 APROVADA 01/11**.
  O fecho (02/11, auditoria + mutação) não mudou comportamento visível, mas **não foi mostrado ao dono**: integrar não é aprovar.
- A linha editou a entrada UI/UX do `CLAUDE.md` §5.1 (smokes `6..47`, «Último» novo): o integrador reescreve a linha inteira (§7), sem merge textual.

## 1 — O que entrou (mecanismo completo nos handoffs de continuação, `HANDOFF_CONTINUACAO_line_UIUX_<data>_<onda>.md`)

| onda | o quê |
|---|---|
| F40 (29–30/10) | Smoke F38 (G29–G33): recusa de memória da placa é a MESMA recusa; pilha de FX sob escopo de memória; intermédia do Vello recusada nunca instalada. Esqueleto: família de canal OSSO=13 (Length/Strength/Segments com losango), curva do osso por campo. `View::Toggle` no canal passa a 0=desligada (`DOC_VERSION` 24→25). Lei C do dono: nó do Zig Zag num vale é cortado (`fx_zigzag_corta.rs`). 16 cores cruas dos overlays → tokens. Cenas `VEC_BONE_SMOKE=8`. |
| F41 (01/11) | Follow Value sobre caixa lê o mesmo estado (210); regras do mundo/mesa voltam com o Desfazer; matriz de camadas da física animável (36 canais); **Length do osso empurra o filho e estica o desenho** (211); pele mistura o esticão; 1 Ctrl+Z por gesto; arrastar sobre interruptores pinta-os (`toggle_paint.rs`: matriz Collision Layers + ícones da Hierarchy); escolher osso ⇒ Pose. Cena `CONSTRAINT_SMOKE=13`. 3 rodadas de smoke, OK. |
| F42 (01/11) | Um glifo único e uma cor por TIPO de objecto na Hierarchy: 18 tipos (`hier_types.rs`, 18 tokens `obj-*`, 5 glifos novos); a cor sai do glifo; «Painted» antes de gravar. Cena `OBJECT_MODE=42`. Mutação 10/10. |
| F43 (01/11) | Effects e Filters do Vector no layout do Figma: «+» abre lista (21 `PathEffect::KINDS`, 15 `FxOp::SPECS`), linhas com glifo/olho/«−», propriedades à ESQUERDA (`fx_side.rs`); 36 SVGs; light-dismiss em `pointer_down_menus.rs`. Cenas 43, 44. |
| F44 / F44b / F44c (01/11) | Fill e Stroke no layout do Figma; linha única `[cor]◇ [%]◇ [⚙] [👁]◇ [−]`; **olho da base** = `VecPath::fill_hidden/stroke_hidden` (**VEC_SCENE 25→26, PROJECT_SCHEMA 211→212**); `PaintSlot{Base,Layer(i)}` roteia a edição ao ⚙ aberto; Position em duas tintas no corpo do ⚙; gizmo do degradê edita a camada aberta; Weight na linha; largura do traço de camada deixa de ser gravada em metros do mundo (px, `DocPx`). Cenas 45, 46. |
| **F45 (01/11)** | **§1.1.** |

### 1.1 F45 (`8c04643af3`, smoke do dono APROVADO 01/11)

- Fill & Stroke numa só secção; a tinta de BASE tem posição na pilha: `VecPath::base_order: BaseOrder{fill,stroke}` (u8; neutro 0/1 = ordem antiga), `StackItem`/`Place`,
  `draw_order`/`move_paint`/`remove_paint`/`set_draw_order` em `crates/ph2d-vec-scene/src/paint_order.rs`. **Escolha B medida** (base fica nos campos): `.fill` é lido em ~641 ficheiros; a ordem fixa só existia em `paint_stack()`, `draw_path_with` e no passe de instâncias.
- Arrastar o ⚙ reordena: `PanelRowFamily::VectorPaint`, voo `paint_flight`, vão `Bg3`, linha de acento; queda = ranhura sob o cursor (`take_landing`). «+» = menu Fill/Stroke.
- Cena `PH2D_OBJECT_MODE_SMOKE=47` (`PH2D_OBJECT_MODE_GESTO=voo` = foto do arrasto a meio): `img_F45/cena47.png`, `cena47_voo.png`.
- **Defeitos antigos curados:** sem fill de base a geometria do preenchimento não se construía (a camada Fill sumia); camada Stroke sem fill/stroke de base rebentava num `expect` (`path_tess`, `ph2d-vec-render/src/lib.rs`).
- **Seams que já falhavam no HEAD `a020be7be2`** (medido numa worktree temporária): 3 (chips de Position e «só o +» desde a F44c; rachura do Weight atrás de bool) — curados no `8c04643af3`.

### 1.2 Fecho da linha (02/11): `46276eddc2`, `cb246b62aa`, `c64df4d06e`

- Censo `the_gap_between_an_icon_and_its_label`: tiradas 3 superfícies obsoletas (`paint_stack_rows`, `paint_effects`, `paint_filters`) que a F43/F44 desfizeram (`46276eddc2`).
- **Mutação (11, worktree separada):** 9 sangraram (ordem de desenho, `path_tess`, leitores v25/v26, landing Above/Below, ordem publicada, `layers_differ`, recook, `settle_hidden`);
  2 sobreviveram e ganharam teste: `BaseOrder::default()` (afirmado em `the_neutral_order_is_the_pre_f45_order`) e o gizmo pela porta da app (`the_canvas_gradient_gizmo_follows_the_open_layer_through_the_app_door`). Commit `cb246b62aa`.
- **Auditoria (2 lentes), curados:** tinta escondida segue mover/escalar (`path_bake_xform`) · arte de padrão escondido/de camada vai para o ficheiro (`project_texture_pattern::art_ids_named_by`) · fechar caminho com fill escondido não semeia 2.º (`despacho_vetor_ops`) ·
  booleana leva a tinta escondida (`ph2d-vec-boolean` `compound_from`) · instância documento×primitivo por `base_fill()` (`instance.rs` ×3) · selecção múltipla: o painel recebe a pilha À VISTA e `layers_differ` a compara · voo nos dois eixos.
- **REFUTADO por medição:** «arrastar o ⚙ da camada aberta deixa o painel a editar outra» — o voo já fecha; `dropping_closes_the_open_layer_panel` passa sem conserto ⇒ não existe.
- **RECUSADO:** suprimir o clique no fim de um arrasto activo que volta ao ⚙. A mesma supressão nas abas matou o empurrão de mão trémula (gate `a_five_pixel_nudge_on_a_tab_still_switches_it`, `pointer_up.rs:97-110`).
  Custo aceite: arrastar e voltar ao próprio ⚙ abre os ajustes.
- Tecto: testes do `ph2d-vec-boolean` movidos do `lib.rs` para `lib_tests.rs` (714→484), `c64df4d06e`. `apply_event` 198 linhas (`consumed_early`).

## 2 — Formato e contadores (DELTA — conte no código)

- **`PROJECT_SCHEMA` 209 → 213** (`shells/desktop/src/project_schema.rs:523`; main = 209). Degraus que esta linha acrescentou:
  209→210 `5d843292ca` (Follow Value sobre caixa, F41) · 210→211 `031c692b6c` (Length do osso, F41) · 211→212 `61d4cd9bf0` (`fill_hidden`/`stroke_hidden`, F44b) · 212→213 `8c04643af3` (`base_order`, F45).
  Constantes: `PROJECT_SCHEMA_SEM_TINTA_ESCONDIDA = 211`, `PROJECT_SCHEMA_SEM_ORDEM_DA_BASE = 212`; `fio_do_projeto(212) = 26`.
  Tripla em `shells/desktop/src/project_schema_tests.rs:352`: **`(213, 13, 27)`** (main: `(209, 13, 25)`). Degraus 163→166 arquivados em `project_schema_history_v160.rs` (escada viva 595/600).
  Reconte com `python3 scripts/schema-recount.py <valor do main>` DENTRO do conflito, se o main/outra linha subir antes.
- **`VEC_SCENE_SCHEMA_VERSION` 25 → 27** (`ph2d-vec-scene/src/schema.rs:107`; 26 = F44b, 27 = F45). **`DOC_VERSION` (`ph2d-timeline`) 24 → 25** (`doc.rs:153`, F40).
- Fixtures gravadas pelo código ANTIGO (worktree temporária): `vec_scene_v25_tinta.bin` (código `d48b76b8fd`) · `vec_scene_v26_ordem.bin` (código `a020be7be2`) · `vec_scene_v24_pilha_cad.bin` (`5228501c26`) · `timeline_doc_v24.bin` ·
  `valor_caixa_v209_doc.bin`/`valor_caixa_v209_pilha.bin` (`ph2d-app-constraints`).
- Cenas: `PH2D_OBJECT_MODE_SMOKE` `1|2|4|6..47` (main `..41`; próxima **48**) · `PH2D_VEC_BONE_SMOKE` até 8 · `PH2D_CONSTRAINT_SMOKE=13`. Quem integrar outra linha com cena nova RECONTA.
- ADR: nenhum novo (`git diff --name-only 422fec2027..HEAD | grep -c decisions/` = 0).
- Deps: `bumpalo = "3"` novo no `ph2d-app-vec` (já no `Cargo.lock`); `postcard` em `ph2d-app-constraints` e `ph2d-constraints-ecs` (passa de dep a dep+dev conforme o diff); `ph2d-timeline-preview` → `ph2d-skeleton-ecs` + `serde_json`. `Cargo.lock`: só arestas.
- Registos de componentes: não recontados — o integrador reconta pelos gates de registo.

## 3 — Foundational e contratos congelados (§6)

- Foundational tocado, ADITIVO (Modo L):
  - `ph2d-editor-core`: `PanelRowFamily::VectorPaint` (variante APENDADA no fim; `aninha()` false); `interaction/toggle_paint.rs`; `hier_types.rs`; `panel_chrome::paint_segmented_icon_group`, `paint_icons::paint_icon_toned`.
  - **`crates/ph2d-editor-core/src/paint_flight.rs` é CÓPIA BYTE-IDÊNTICA da da `line/PixelArt`** (sha256 `0d7f40f0f34ebabe0ab570af7c53c36a10beba71b4ad06bdc43788f03588a6bb`, conferido) com a MESMA linha no `lib.rs`
    (`/// O voo de um arrasto…` + `pub mod paint_flight;` depois de `pub mod paint_batch;`) ⇒ add/add IDÊNTICO: aceitar um dos lados, uma só cópia da linha.
  - `ph2d-vec-scene`: campos novos no FIM de `VecPath` (`fill_hidden`, `stroke_hidden`, `base_order`); `ph2d-ecs` (`canal_osso.rs`), `ph2d-timeline` (`doc_v24.rs`), `ph2d-tokens` (18 `obj-*`).
- **Contratos §6 NÃO tocados:** `git diff --stat 422fec2027..HEAD -- crates/ph2d-vector crates/ph2d-nodegraph` = vazio; gates `architecture_*` (119) e `arch_*` (24, inclui `arch_safe_clamp_only`) verdes, entre eles `architecture_tool_contract_surface`.

## 4 — O que um merge pode partir

- Literais de struct de outras linhas: `VecPath { .. }` (+3 campos no fim: use `..Default::default()`); `PaintEntry`/`StackItem`; chamadas a `gradient.rs` agora recebem `PaintSlot`.
- `PanelRowFamily` (match exaustivo): a `line/PixelArt` acrescentou `PixelPalette` antes; esta apenda `VectorPaint` no fim — conflito textual esperado no enum, a ordem relativa não importa para o formato.
- Ids de painel removidos (F43/F44): `VECTOR_SECTION_FILL_TYPE`, cartões antigos de Effects/Filters, chaves i18n `panel.vector.fx.add` e `filter.add`; `StackVerb::Open` saiu. Outra linha que os use não compila.
- `VEC_SCENE`/`PROJECT_SCHEMA`/`DOC_VERSION`: se outra linha subir antes, os degraus recontam-se (leitores v25/v26 ficam atrás de `PROJECT_SCHEMA_SEM_*`).
- Ordem de desenho do Vetor: quem desenhar um `VecPath` fora de `paint_stack()`/`draw_path_with`/passe de instâncias assume fill sob stroke (ordem neutra) — correcto só no neutro.
- Colisões prováveis (`collision-surface.sh` pelo caminho ABSOLUTO em cada worktree): `line/PixelArt` (paint_flight, lib.rs, PanelRowFamily).

## 5 — Prova de fecho (02/11) — tudo por NOME, todos VERDES

`cargo fmt --all` · `clippy -D warnings --all-targets` nas **32 crates tocadas** (inclui a shell) · `ph2d-editor-core`: `loc_cap` (7), `the_shell_only_shrinks`, `no_magic`, `canonical_icon_button`, `hit_indexed_ids_are_registered`, `every_editable_field_is_a_channel`, `claude_md`, `hr15`, `arch_` (24), `architecture_` (119), `the_gap_between_an_icon_and_its_label` (7) ·
shell: `file_loc_caps` (4), `project_schema`, `v194` (4), `project_texture_pattern` (10), `path_shape_ids`, `the_app_never_reshapes_a_still_screen` · `ph2d-panel-vector`: seam (228), `f4` (21) · `ph2d-vec-scene`: `vec_v2` (8), `order` (6), `recook`, `paint_stack` ·
`ph2d-vec-render`: `stack_draw` (17), `instance` (7) · `ph2d-app-vec`: `paint_stack` (24), `order_tests` (6), `svg_export` (15), `marcas` (12), `vector_mode` (36) · `ph2d-vec-boolean` (137).
⛔ **Suíte completa e `nextest-impacted` NÃO corridas** (regra do dono 23/10: são da integração).

## 6 — Smokes (comando inteiro; o binário `smoke` já está compilado)

0. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo build -p ph2d-host-desktop --profile smoke` — a 2.ª compilação diz «Finished `smoke` profile [optimized] target(s) in 0.22s» **sem** «Compiling».
1. **F45:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=47 ./target/smoke/ph2d-host-desktop` — no Inspector, secção Fill & Stroke: arrastar o ⚙ de uma linha troca a ordem (a tinta de cima desenha por cima); Ctrl+Z desfaz; «+» abre o menu Fill/Stroke. Errado se: a linha some ao largar, a arte não muda de ordem, o Ctrl+Z não devolve.
2. **F44b/F44c:** o mesmo com `PH2D_OBJECT_MODE_SMOKE=45` e `=46` (linhas iguais com ⚙/olho/«−»; Position dentro do ⚙; gizmo do degradê da camada aberta).
3. F42 `=42` · F43 `=43` e `=44` · F40 `PH2D_VEC_BONE_SMOKE=8` · F41 `PH2D_CONSTRAINT_SMOKE=13`.

## 7 — Integração (instruções ao integrador)

- Só por ordem do dono. `collision-surface.sh` pelo caminho absoluto em cada worktree; a coluna `base:` é o merge-base — leia o `main` no ficheiro.
- Reescreva a linha UI/UX do `CLAUDE.md` §5.1 (≤ 700 bytes; gate `architecture_claude_md_cabe_no_orcamento`): «Último» = este handoff, smokes `6..47` (ou o que a contagem der).
- Reconte `PROJECT_SCHEMA`, tripla, `VEC_SCENE`, `DOC_VERSION`, cenas, registos. Não-rastreados na pasta de handoffs (`img_F30/F33/F34/F40/F41` `.log`, `smoke_30/`, PNG de `img_F41`) NÃO são código: comite só se quiser a foto.

## 8 — ABERTO (próxima rodada, por ordem)

1. ⭐ **Dono 01–02/11: «no modo Draw, ao mudar o TIPO DE ALÇA, a linha que representa o stroke 0 (o fio do caminho, 1 px) não atualiza — fica a linha do caminho antigo».**
   Fotos: `img_F45/dono_linha_residual_1.png` e `_2.png`. Medido: linha (179,204,255) opaca ~1,5 px por cima da arte, retas entre âncoras.
   **NÃO reproduzido** em 2 sondas (cena 47 com Smooth no caminho; cena 47 com todos os pontos a Smooth pelo `VECTOR_VERT_SMOOTH`) — nas duas o fio acompanha.
   Pista: o fio (`ph2d_vec_render::draw_path_wires`, `crates/ph2d-vec-render/src/path_wire.rs`, chamado por `crates/ph2d-app-vec/src/edit_marks.rs:87`) desenha o `vec_scene` do DOCUMENTO, enquanto a arte e o pick usam a geometria VIVA `vec.live_drawn`
   (`LiveGeometry`, 9 produtores, `shells/desktop/src/render_loop/fase_vector_bands.rs:224`): com pontos arrastados + Auto Key/linha do tempo as duas fontes podem divergir.
   Reproduzir com o gesto EXACTO do dono (pontos arrastados antes, tipo de alça pelo botão da barra do Draw) ANTES de consertar.
2. A caixa do Weight corta números de 2 dígitos («16» → «1(») no Inspector estreito (vem da F44c).
3. Esc não cancela o arrasto de linha de painel (toda a família `panel_row_drag`, pré-existente).
4. Perfil de largura não é por camada de traço.
5. Ids de Position fora do censo `seam`.
6. `migra_pilha_v23` sem chamadores.
7. Com nada seleccionado, o ⚙ do traço de base fica activo e o do Fill apagado (comportamento da F44; confirmar com o dono).
8. ⛔ **Janela baixa: a régua de ferramentas FLUTUANTE regista botões FORA da janela** (achado na integração de 02/11,
   frente shell/painéis, pela sonda de `crates/ph2d-panel-registry-init/tests/it/the_vector_control_bar.rs`). Numa janela de
   430 px de altura (⅓ e 0,42 da de fábrica) a régua em `x=320` regista botões de 24 px até `y≈774`: essas ferramentas são
   INALCANÇÁVEIS — nem se vêem nem se clicam. Falta o recorte/rolagem da régua pela altura da região (o mesmo idioma do
   painel do traço, que rola dentro da região). ⚠️ Sem gate ainda: o gate novo tem de medir no ECRÃ (registo de hit vs.
   janela), não no mundo, e reprovar antes da cura. Não curado.

# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 20/10 — F24b: o grupo no dope-sheet e o losango nos campos do vetor

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (o main não andou; sem rebase). Anterior:
  [F24a](HANDOFF_CONTINUACAO_line_UIUX_2026-10-20_F24a_O_MOTOR_DOS_CANAIS.md) (§2 = o motor dos canais; §6 = a lista que esta onda herdou).
- Plano: [`35_plano_F24_o_auto_key_grava_tudo.md`](../vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md) §5 (dope-sheet agrupado + losango).

## 1 — Commits desta onda

| commit | o quê |
|---|---|
| `e35a3e2ef8` | linhas de GRUPO no dope-sheet (a cor inteira, a forma inteira numa linha só) |
| `dd0d4febef` | o LOSANGO ao lado de cada campo do vetor (mostra o canal; o clique grava/apaga a chave do grupo) + consertos que a bateria achou |
| `6024a9f1e5` | fotos `F24_cena_autokey_3.png` / `_4.png` (+ `.log`) |

## 2 — Como foi construído

### 2.1 Grupo (painel timeline, `crates/ph2d-panel-timeline/src/linhas.rs`)

- `FieldKey::grupo()` (`crates/ph2d-ecs/src/canal.rs:52`) = porta única `(familia, grupo, i|0)`; `mesmo_grupo` lê-a. `FieldKey::membro()` (`canal.rs:69`) / `canal_vetor::membro` (`canal_vetor.rs:181`) = nome do campo dentro do grupo, chaves i18n `canal.m.*` (42, em `crates/ph2d-i18n/src/canais.rs`).
- `linhas::layout(snap, tab, expanded, abertos, graph_h, top, scroll) -> Layout{bandas, grupos}` (`linhas.rs:120`) SUBSTITUI `geom::row_bands` (removido). Fonte única de: pintura, hit, retângulo de seleção (`linhas::keys_in_rect`, `:292`) e altura do scroll (`geom::content_h` ganhou `abertos`).
- Grupo com ≥2 membros = cabeçalho (`paint_grupo`) com 1 losango por coluna dos membros (`summary::columns_of`); grupo de 1 membro NÃO tem cabeçalho. O ▶ do grupo abre os membros (ordem (i, j, campo); nomes «Point X 3», «R», «contorno.ponto» quando há 2 contornos; recuo de um ▶; cada membro mantém o ▶ do editor de curva). Estado: `TimelinePanelState.grupos_abertos: Vec<GrupoChave>` (`GrupoChave{entity,familia,grupo,i}`).
- Losango do grupo = coluna do Summary restrita: `linhas::Escopo::{Tudo,Grupo}`, `SummaryPress.escopo`; `summary::apply_gesture`/`column_at` ganharam escopo. Com a trava de coluna ligada vale `Escopo::Tudo`. Por isso os intents multi-chave existentes (mover, apagar, interp, copiar/colar) funcionam sem mudança.
- Hit kinds novos (editor-core, APPEND): `TimelineHitKind::GroupKey{group,t_bits}` / `GroupTwirl{group}`; `group` = `AnimTarget` do 1.º membro, resolvido por `GrupoChave::do_handle`. Ids em `crates/ph2d-panel-timeline/src/ids/timeline_grupos.rs` (`timeline_group_key_id`, `timeline_group_twirl_id`; `dynamic_id` passou a `pub(super)`).
- Rótulo: tinta/traço da entrada i>0 da pilha leva o número («Fill 1»), `tracks_label.rs` `track_label`.

### 2.2 Losango (F23 metade B generalizada)

- `crates/ph2d-editor-core/src/widget/property_box/key_mark.rs`: `KeyMark::Livre` novo (o ponto, mas clicável: o 1.º clique cria a chave); `key_mark_id(campo) = NodeId(campo ^ SAL)` (`:54`); `paint_decorator_for(scene, theme, r, fields) -> Option<NodeId>` (`:103`) — QUEM PINTA registra o alvo no HitIndex (o widget não referencia `interaction`: catraca do DAG widget→interaction, 43); `set_key_marks` também esvazia os alvos; `key_mark_at`/`begin_key_mark_targets` REMOVIDOS; `key_mark_target(id)`, `pedir`, `take_key_mark_click()` (`:95`).
- `dispatch/pointer_down.rs` `dispatch_down`: hit cujo id é um losango ⇒ `pedir` + return (o clique não chega ao campo).
- Ligado nas 4 linhas de `property_row.rs` (`fields_row_inner`, `field_row_value`, `text_row`, `color_row`) e no `slider_with_chip` (`PropertyBox.animado: Option<NodeId>` NOVO — com `Some` a caixa NÃO pinta o ponto, o chamador pinta).
- `TimelineDoc::author_cut(solo, root, t_raw)` (`crates/ph2d-timeline/src/channel_mark.rs:78`) = porta única do relógio do autor; o `autokey_pass` passou a usá-la.
- `crates/ph2d-app-vec/src/marcas.rs`: tabela CAMPOS (Transform X/Y/W/H/R e VERT_X/Y → Forma (`ANCORA_X`): o Transform do painel move os PONTOS; FILL_SWATCH/FILL_OPACITY → tinta do preenchimento; STROKE_SWATCH/STROKE_OPACITY → tinta do traço; WIDTH → `TRACO_LARGURA`; DASH_IDS/GAP_IDS → `TRACO_TRACEJADO`; VECTOR_OBJ_OPACITY → `PropKind::Opacity`). `marcas()` = o mais forte entre membros e objetos selecionados. `clique()` = com chave neste quadro apaga as do grupo (BeginEdit, ClearSelection, AddToSelection…, DeleteSelection, EndEdit); senão grava o grupo inteiro publicado em `CanaisLidos` pela porta da K — num passo de undo.
- Shell: `fase_autokey.rs` (depois de `autokey_pass::run`) lê `take_key_mark_click` e publica `set_key_marks` 1×/quadro. `timeline_bridge::chave_da_k` (`render_loop/timeline_bridge_keys.rs:315`) foi movida da fase da K (que voltou a 185 linhas); a agulha do controle positivo em `fase_timeline_key_insert_tests.rs` passou a `timeline_bridge::chave_da_k(`. `autokey_pass` re-exporta `campos_lidos` `pub(crate)`.

### 2.3 Consertos que a bateria achou

`canais_campos.rs` 794 → 588 linhas (a tinta foi para `canais_campos_tinta.rs`; era defeito da F24a) · `ids/timeline.rs` 610 → 597 (tecto 600 dos painéis) · a isenção `every_stack_of_rows_asks_the_rhythm` viajou de `geom.rs` para `linhas.rs` · `panel.timeline.prop.field` saiu de `canais.rs` para `crates/ph2d-i18n/src/timeline.rs` (o portão `every_key_of_this_crate_exists_on_both_sides` reprovava).

## 3 — Superfície de colisão (um merge pode partir)

- `TimelineHitKind`: 2 variantes (append); os `match` de fora usam `_`.
- **`PropertyBox` literal ganhou o campo `animado`** — 8 sítios: `slider_with_chip`, `showcase/slider.rs`, `tests/it/the_app_default_slider_style_is_the_one_the_owner_chose.rs`, `ph2d-panel-widget-lab` `study.rs` ×5. Outra linha que construa `PropertyBox` literal PARTE.
- `paint_decorator_for` mudou de assinatura; `KeyMark` ganhou variante; `SummaryPress` ganhou campo; `geom::row_bands` removido; `content_h`/`apply_gesture`/`column_at` mudaram de assinatura.
- Dev-dep `ph2d-ecs` no `ph2d-panel-timeline` (`Cargo.lock`).
- Tectos: `pointer_down.rs` 695/700; `tracks.rs` 581/600; shell 188 640 linhas (+142 nesta onda; tecto 189 041 — sobram 401).
- Timeline `DOC_VERSION` 21 e `PropKind::Field` seguem do F24a §3.

## 4 — Smoke do dono (binário smoke já compilado: 2.ª build `Finished smoke profile in 0.63s`, zero `Compiling`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=3 ./target/smoke/ph2d-host-desktop` — estrela laranja selecionada, Auto Key ligado, relógio no quadro 12.
2. No painel Vector (à direita), mude a cor de «Fill».
3. Tem de acontecer: no Timeline aparece UMA linha «Path 0 · Fill» com um losango no quadro 12, e o ponto ao lado de «Fill» vira losango cheio. Mude «Width» e arraste um ponto da estrela ⇒ linhas «Path 0 · Stroke Width» e «Path 0 · Shape». Leve o relógio ao quadro 24 e mude a cor de novo; no quadro 18 a cor está a meio e os losangos ficam vazios. Clique no ▶ de «Path 0 · Fill» ⇒ aparecem R, G, B, A… Clique num losango cheio ao lado de um campo ⇒ a chave daquele quadro some. Clique no ponto de um campo parado (ex.: Opacity) ⇒ nasce a chave.
4. Errado = uma linha por número (Fill ×6, Shape ×81), losango que não muda ao clicar, ou o campo a mexer quando se clica no losango.

`PH2D_AUTOKEY_SMOKE=4` faz sozinho e para no quadro 18. Fotos: `../vetor_nova_ui/fotos/F24_cena_autokey_3.png` (início: ponto em todo campo, dope-sheet vazio) e `F24_cena_autokey_4.png` (q18: 3 linhas de grupo Fill / Stroke Width 0,09 / Shape; losango vazio em Position, Size, Rotation, Fill, Fill Opacity, Width; ponto em Opacity e na cor e opacidade do traço).

Números da sonda (log da foto `=4`): q18 cor (65, 120, 145) = meio exato de (0,200,90) e (130,40,200); traço 0,0900 = meio de 0,14 e 0,04; trilhas: forma 81×1 chave, preenchimento 6×2, `traco_largura` 1×2.

## 5 — Verificação (verde)

- `ph2d-panel-timeline` crate inteira 284 + 122 (linhas 10, seam `grupo_de_canais_seam` 2, strings 13) · `ph2d-editor-core` inteira (lib 1722 + it 542 depois dos consertos; os 5 vermelhos da 1.ª corrida — DAG widget→interaction 44/43, `canais_campos` 794, `ids/timeline` 610, ritmo `linhas.rs`, isenção obsoleta `geom.rs` — repetidos por nome e verdes) · `ph2d-panel-vector` inteira 49 + 273 (+ `seam_key_mark` 1) · `ph2d-panel-inspector` inteira 26 + 446 · widget-lab check · `ph2d-timeline` `channel_mark` 5 / `autokey` 33 · `ph2d-app-vec` `canais` 13 / `marcas` 4 · shell `autokey` 56 (incl. `o_clique_no_losango_grava_e_depois_apaga_a_chave_do_traco`), `key_insert` 4, `undo` 69, `timeline_preview` 8, `opacidade_vetor` 6.
- Vermelhos provados com a cura desligada: `Escopo::Tudo` no braço `GroupKey` (1 reprova); comparar só o representante do grupo em `marcas` (2 reprovam); sem o braço do losango em `dispatch_down` (o seam reprova).
- `cargo check --workspace --all-targets`: re-corrido no fim: verde.
- **NÃO corridos:** clippy, `nextest-impacted`, mutação completa.

## 6 — ABERTO, por ordem

1. **Smoke do dono** (§4).
2. **Campos sem canal na tabela de `marcas.rs`:** linhas da pilha de pinturas (`VECTOR_PAINT_*` — o id não diz qual entrada i), gradiente (`VECTOR_GRAD_*`), contornos (`VECTOR_CONTOUR_*`), o Opacity da barra de controle (`bars_control.rs`, sem coluna de decorator) e os campos do Inspector (`INSP_TRANSFORM_*` etc. — a tabela só cobre o vetor).
3. **Gate `architecture_every_editable_field_is_a_channel`** (plano 35 §5.2) por construir.
4. A linha de grupo não tem menu do botão direito (Delete Track do grupo inteiro) nem valor na coluna de valor.
5. **Perf:** `linhas::layout` aloca um `BTreeMap` por chamada (pintura + altura do scroll, por quadro) e `marcas()` corre por quadro com `binding_for` linear — medir na cena grande do T0(b) ANTES de otimizar.
6. **Lacunas de vocabulário no vetor** (F24a §6.4): CONTEÚDO do texto e FAMÍLIA da fonte (strings — pedem tabela de strings, Texto/Hold), FONTE do padrão (asset), parâmetros do Brush stroke; `StrokePaint` tipo índice 5 = Brush.
7. **Bloco 2** (plano 35 §10): todas as outras famílias §A.2–A.9 + Painter quadro-a-quadro. Antes: colisão com a missão dos pincéis do Painter (`git worktree list`).
8. **Perf T0(b):** cena grande (100 paths × 100 vértices × 8 ≈ 80 mil bindings); critério de morte 2 ms ⇒ avaliação em lote por grupo. `binding_for` é linear; custo de `publicar` por quadro por medir.
9. **ADR** «o canal de qualquer propriedade» (`PropKind::Field` + `CanaisLidos`/`CanaisEscritos` + eco + `ConducaoVetor`): contar o próximo número em `docs/architecture/decisions/README.md`.
10. **Gate de fecho:** clippy `--all-targets`, `nextest-impacted`, `cargo check --workspace`, lista de mutação (plano 35 §6 + leitura devolve a curva · escrita no-op · topologia só na 1.ª chave · Hold desligado · cor por OKLCH · eco desligado · `settle` esquecido) **mais** `paint_decorator_for` sem registrar o alvo · `set_key_marks` sem limpar os alvos · `clique` com `Escopo::Tudo`.
11. `CLAUDE.md` §5.1 UI/UX ao fechar a linha: trocar o link do último handoff e o intervalo do smoke (inclui `PH2D_AUTOKEY_SMOKE=1..4`). Não alterado nesta onda.
12. **Conhecido, não é defeito novo:** o undo da timeline da propagação é separado do undo global da edição do vetor (duas histórias, desenho existente); mudanças de topologia não alinháveis (simplify path) recebem mapeamento LCS de melhor esforço.

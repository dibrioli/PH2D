# Continuação — `line/UIUX`, 2026-10-04 — o OBJECTO VETORIAL (contentor), NESTA linha, sem integrar

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd Worktrees/line-UIUX && pwd && git branch --show-current` ANTES de ler. HEAD esperado = o commit
> deste ficheiro; árvore limpa; base (merge-base = `main`) `1ad60a1ce`, nenhum commit integrado.

## §1 — Estado

- A F3 do Vector foi entregue e fechada em
  [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_VETOR.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_VETOR.md)
  (gate batched verde, mutação 20/20) — e o **smoke do dono a RECUSOU** (04/10):
  > *«não ficou legal desse modo. Melhor seria se houvesse apenas uma opção no modal: objeto vetorial.
  > Ao clicar nele cria-se um objeto vazio e entra-se no modo edit do vector com o menu exatamente como
  > era antigamente com todas as opções de desenho no painel. Sendo assim num mesmo objeto poderíamos
  > criar quantas formas quisermos e fazer as operações booleanas todas dentro de um objeto. As shapes
  > são filhas do objeto vetorial vazio.»*
- ⇒ Esta onda REFAZ o produto do Vector sobre o que lá está. O fecho escreve um handoff de integração
  NOVO sobre o diff acumulado desde `1ad60a1ce`, que cita os SETE anteriores (A_ESCALA, O_MENU_ADD,
  OS_MODOS, O_SCULPT, O_FLIP, O_MODEL, O_VETOR) e diz o que do O_VETOR deixou de valer.

## §2 — A TAREFA (desenho aprovado em frases; escolhas do dono)

1. **Menu Add: UMA entrada, *Vector Object*.** Cria uma entidade VAZIA marcada como objecto vetorial e
   **nasce em Edit** (o `wants`/`born`, como Flip e Model). Saem as entradas Rectangle · Ellipse ·
   Polygon · Star (F1) e Pen · Pencil · Text (O_VETOR).
2. **Edit = a ferramenta `vector` na mão com o painel COMPLETO de antes** (todas as ferramentas, a
   grelha inteira). Tudo o que nasce nesse Edit — caneta, lápis, formas, texto, booleana, soldar,
   corte, colar — nasce **FILHO** do objecto. Só as formas dele se agarram e mostram nós.
3. **Object:** a ferramenta sai da mão; clicar numa forma selecciona o **objecto inteiro** e o gizmo
   move-o com as filhas. `Tab` (sobre o objecto ou uma filha) volta ao Edit.
4. **Hierarquia:** «Vector» com as formas por baixo; em Edit a selecção anda entre elas (as `parts`
   do Model), outro objecto é recusado. `Tab` com vários objectos vetoriais = Edit de todos (`joins`).
5. **Formas SOLTAS** (escolha do dono, 04/10: *«cada uma ganha o seu objecto»*): toda forma vive
   dentro de um objecto vetorial — uma forma solta (projecto antigo, cenas de teste, colar fora de um
   Edit) ganha um objecto NOVO só para ela; **um SVG importado de uma vez entra num só objecto**.
6. O pill VECTOR continua FORA (O_VETOR).

## §3 — O que já foi MEDIDO (04/10, com endereço)

- **Forma como filha:** um grupo vetorial é uma entidade SEM `VecPathRef` com filhas `ChildOf`
  (`crates/ph2d-vec-entities/src/entities_group.rs:57–73`); a moldura é uma forma com filhas.
- **Mover o pai move as filhas:** `world_transform = parent_world ∘ Transform`
  (`crates/ph2d-vec-entities/src/transform.rs:23–30`, `VecXforms` em `:52–65`).
- **Reparentar sem mexer:** `reparent_keeping_world` (`transform.rs:204`, gate
  `transform_reparent_tests.rs:56`) — a porta para «nasce filho» (a forma nasce em mundo).
- **Um path novo nasce RAIZ** em `vec_entities::sync` (`entities.rs:110–128`); booleana e soldar
  também (`bool_gesture.rs:105–118`). ⇒ uma porta só depois do `sync`: entrada nova no mapa com um
  Edit activo → filha do alvo; sem Edit → um objecto novo (a regra das soltas).
- **Clique em Object sobe ao contentor — já há precedente:** o Envelope (`VecEnvelope`):
  `container_of` / `sole_container` (`crates/ph2d-app-vec/src/envelope_live.rs:357/372`), usado na
  ponte `vec_selection.rs:107` (a selecção do gizmo sobe ao contentor, o pen fica com as filhas). O
  pick do canvas: `despacho_clique_gizmo.rs:510` → `vec_gizmo_pick.rs:256`.
- **O marcador do tipo é NOVO:** nenhum existente serve (`GroupedChildren` trava, não marca). Um
  componente `VecObject` em `ph2d-ecs` toca: o tipo + `register_default` em
  `crates/ph2d-ecs/src/scene/registry.rs:314–400`; `ObjectKind::marker()` do `Vector`
  (`crates/ph2d-component-desc/src/lib.rs:390–398`) passa de `VecPathRef` a `VecObject`; `kind_of`
  (`crates/ph2d-app-components/src/component_attach.rs:23–41`); o gate
  `every_marker_derives_its_kind` (`ph2d-app-components/src/object_add_tests.rs:136`); e o degrau do
  **`PROJECT_SCHEMA` (hoje 178, `shells/desktop/src/project_schema.rs:343`)** — parágrafo + tripla em
  `project_schema_tests.rs`; na integração reconta-se por `python3 scripts/schema-recount.py`.
- **As formas DENTRO** passam a ser partes: `kind_of` de uma filha deixa de ser `Vector` (como as
  formas do Model, que não têm `FieldObject`); `owner_of(forma)` = o objecto.
- **As ~50 cenas `PH2D_*_SMOKE` que fazem `set_active("vector")`** criam formas na RAIZ com
  `push_path` (`knot_smoke.rs:66`, `cut_smoke.rs:107`, `brush_corner_smoke.rs:107`) ⇒ pela regra 5
  cada uma ganha um objecto. Corra os gates delas e a foto de umas 3 antes de fechar.

## §4 — O que se MANTÉM e o que se DESFAZ dos commits da onda anterior

Mantém: `030b0a5ed` pill VECTOR fora (TOPBAR_VECTOR, vector_toggle, MODULE_TRUTHS 18, Window) · a
`vector_mode::Family` (alvo fora da `VecScene`, `holds`/`releases`, a porta antiga das cenas) ·
`ModeFamily::joins`/`enter_with` (multi-objecto, fundação, com gate) · `VecViewState::editing` /
`in_edit` (clique E desenho, `8e2aa233c`) e a cópia no `view_state_for_pick` · o censo de famílias com
a tabela D6 de PARES · `PH2D_OBJECT_MODE_SMOKE=6` (refazer a cena: *Add ▸ Vector Object*, desenhar 2
formas, booleana, Tab).

Muda: o alvo do Edit passa de «as formas» a **o objecto** (`EditTarget` guarda o(s) contentor(es); o
`editing` da vista = as formas filhas dele(s)); `parts` = as formas filhas; `holds` = a ferramenta
`vector` na mão (QUALQUER `DrawMode`) + o alvo.

Desfaz: a partição `DrawMode::EDIT_TOOLS`/`object_mode` (`params_mode_object.rs`, e o gate
`the_edit_tools_are_exactly_the_draw_modes_of_edit`) — o Edit usa todas · o filtro da fileira do
painel (`ph2d-panel-vector/src/paint_modes.rs`) e os dois gates de seam que o seguiam
(`every_tool_row_pill_answers_a_real_pointer` volta ao de `e99d5d72e`;
`the_tool_row_shows_only_the_tools_of_the_current_mode` sai) · `object_add::{SHAPES, TOOLS, arm,
tool_of}`, `EditTarget::armed`, `Born::Tool`, i18n `object_add.vector.{pen,pencil,text}` e as quatro
das formas · a catraca da altura do painel `vector` volta a subir (`1 239 → 1 262`, ESCREVA o motivo:
a grelha inteira volta) · a fixtura `the_app_never_reshapes_a_still_screen` abre o Add — com menos
entradas pode voltar a ficar abaixo de `1 024` textos: re-meça.

## §5 — Lições desta linha que valem aqui

- ⛔ **Uma pergunta ao dono sobre ONDE pôr controlos leva o custo MEDIDO no tablet.** A «barra da
  esquerda» do redesenho é a FILA de cima; 4 chips = +~160 px, não cabem no iPad 11 (582) nem no mini
  (521) — só o gate batched o viu, e o dono teve de escolher duas vezes.
- ⛔ **A foto apanha o que nenhum gate vê** (as âncoras das formas fora do Edit desenhadas). Fotografe
  o Edit com DUAS formas, uma de cada objecto.
- ⛔ «O módulo edita o 1.º objecto» nunca é uma linha; o alvo não mora no documento do undo; o
  cadeado exacto derruba um modo de partes no 1.º clique (O_MODEL) — aqui as partes SÃO as filhas.
- ⛔ Apagar um ficheiro de testes pelo NOME apaga cada gate que mora nele (`grep -n '^fn '`).
- Explorador com perguntas ESTREITAS (≤ 30 passos) — dois pararam a meio nesta onda.

## §6 — Restrições (medidas)

`screens/hero.rs` 699/700 · `interaction/state/mod.rs` 696/700 · `left_rail.rs` 693/700 ·
`hero/paint.rs` 700/700 · `ph2d-tool-vector/src/params_mode.rs` perto dos 700 · shell
(`the_shell_only_shrinks`) **196 909 / 196 990 — folga 81** (o degrau do schema e a fase do Add gastam
dela: código do vetor vai para `ph2d-app-vec`). Contratos congelados: o `DrawMode` não está no
`architecture_vector_contract_surface` (lê `ph2d-vector-doc`).

## §7 — Como a onda fecha

DIRETRIZ §1.5.9: mostrar ao dono o plano técnico só se mudar alguma das frases do §2; codar; gate
batched 1× desde `1ad60a1ce` (`ph2d-panel-registry-init` com `--all-features`); mutação dos gates
novos (lado independente: família falsa no editor-core, a real em `ph2d-app-vec`); foto
(`fotografa_cena.sh`, `PH2D_OBJECT_MODE_SMOKE=6`, e `PH2D_OBJECT_ADD_SMOKE=1` com UMA entrada vetorial);
handoff de integração NOVO que cita os sete; `rm -rf target/*/incremental`; e por último o build de
smoke 2× com a 2.ª saída colada. ⛔ Não integrar nem pushar.

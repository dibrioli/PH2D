# HANDOFF de continuação — line/UIUX, F43 (01/11) → F44

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `ff8cac461c` (F43), merge-base com o
main `422fec2027`. 40 commits sobre o main em `ff8cac461c` (o main não andou). **Nada integrado, nada
enviado.** A F43 passou no **smoke do dono em 01/11: «Muito bom»**. Antes de ler: `cd` + `pwd` +
`git branch --show-current`. Handoff anterior: [F42](HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F42.md).

## 1 — O que a F43 fez (commit `ff8cac461c`)

Pedido do dono: Effects e Filters do Vector no layout do Figma (dropdown, lista, propriedades à esquerda).

- **Cabeçalho com «+»**: `section_header_with_action` (`ph2d-editor-core`, `panel/rows.rs`). O «+» abre
  uma lista com TODOS os tipos do motor: **21** `PathEffect::KINDS` e **15** `FxOp::SPECS`.
- **Ícone próprio por tipo**: 36 SVGs novos (`docs/design/icons/fx-*.svg`, `filter-*.svg`), variantes de
  `IconId`; mapeamento em `ph2d-panel-vector/src/glifos_do_motor.rs`.
- **Itens aplicados = linhas** (glifo, nome, olho, «−»), pintadas por `paint_fx_list.rs` (enum `Pilha`
  Efeitos/Filtros). Clicar numa linha abre as propriedades à ESQUERDA do Inspector (`fx_side.rs`:
  `paint_menu`, `paint_props`, `on_click`; cai para a direita se não houver espaço; ↑ ↓ ✕ dentro do painel).
- **Ids novos**: `VECTOR_FX_ADD_MENU`, `vector_fx_row_id`, `VECTOR_FX_PROPS_CLOSE`, `FILTER_ADD_MENU`,
  `filter_row_id`, `FILTER_PROPS_CLOSE`. Ids dos cartões antigos removidos; chaves i18n
  `panel.vector.fx.add` e `filter.add` removidas.
- **Light-dismiss do dropdown** movido para `pointer_down_menus.rs::light_dismiss_dropdown`, com guarda
  `inside_picker` (clique no seletor de cor já não fecha o popover). `event.rs`: `opens_or_closes`.
- **`register_ramp_handles`** extraído em `paint.rs` (arrastar a rampa no painel lateral).

## 2 — Gates e prova de fecho

- Testes novos `ph2d-panel-vector/tests/it/f43_as_listas_do_figma.rs` (5); gate
  `every_kind_the_engine_publishes_has_its_own_glyph` (`glifos_do_motor.rs`).
- Verde: `ph2d-panel-vector` 348 testes; `ph2d-editor-core` por nome: `icons` (8),
  `node_id_collisions` (14), `the_shell_only_shrinks` (2), `dropdown` (31); testes nomeados de
  `ph2d-app-vec`; `clippy -D warnings` limpo nas crates tocadas; `fmt`. Sem suíte completa (do integrador).
- `PH2D_BUILD_SMOKE=20` (`fx_undo_smoke.rs`) adaptado para abrir o «+»/a linha.
- **Mutação: NÃO feita** (provas dos gates novos ficam para o fecho da linha).

## 3 — Smoke (o dono fez, 01/11: OK)

Cenas `PH2D_OBJECT_MODE_SMOKE=43` (painel do Zig Zag aberto) e `=44` (lista do «+» aberta), em
`crates/ph2d-app-vec/src/vector_mode_listas.rs`. Fotos: `img_F43/cena43.png`, `cena44.png`.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=44 ./target/smoke/ph2d-host-desktop`
2. Na secção Effects do Inspector, clicar no «+»: abre a lista com os 21 tipos, cada um com ícone.
3. Escolher um: entra como linha (ícone, nome, olho, «−»); clicar na linha abre o painel à esquerda.
4. Errado se: sobram botões «Add X» à mostra; o painel abre sem campos; clicar no seletor de cor fecha tudo.

Cenas agora `1|2|4|6..44`.

## 4 — ABERTO

- **(a) Provas de mutação** dos gates da F43 (fecho da linha).
- **(b) Integração**: só por ordem do dono (40 commits sobre o main; `collision-surface.sh` pelo caminho
  absoluto). `PROJECT_SCHEMA` = 211 (`shells/desktop/src/project_schema.rs`), a F43 não o mexeu;
  recontar com `python3 scripts/schema-recount.py <valor no main>`.
- **(c) Não rastreados** (`.log`/`.png`) em `img_F30/F33/F34/F40/F41` e `img_F41`: não são da F43; não
  comitados.
- **(d) PRÓXIMO ITEM, F44** (abaixo).

## 5 — F44 (dono, 01/11: «Vamos fazer como sugere» + «modernizar Cap, Join, Align num modo moderno como Figma, com ícones inteligíveis sempre que possível»)

Referências comitadas: `img_F44/ref_hoje_cap_join_align.png` (hoje: segmentados de TEXTO Butt/Round/
Square, Miter/Round/Bevel, Centre/Inner/Outer) e `img_F44/ref_figma_stroke_settings.png` (linha Stroke:
amostra+hex+100%+olho+«−»; dropdown Position «Center» + campo Weight + botão de settings; popover
«Stroke settings» à esquerda com abas Basic/Dynamic/Brush, Style, Width profile, Join em ícones, Miter angle).

Plano:
1. **Fill**: «+» e linhas (amostra, nome/cor, opacidade %, olho, «−») como nos Effects; clicar abre painel
   à ESQUERDA com tipo (Solid/Linear/Radial/Multi/Pattern — hoje secção própria,
   `paint_fill_type.rs:92`), cor e paragens do gradiente. A secção «Fill type» desaparece.
2. **Stroke**: linhas de cor + uma linha fixa Position (Inside/Center/Outside; motor `StrokeAlign`
   Centre/Inner/Outer, `stroke_style.rs:74`) + Weight; botão de settings abre «Stroke settings» à
   ESQUERDA: abas Solid/Pattern/Brush (`StrokePaint`), Cap, Join, Miter, tracejado (`paint_dashes.rs:94`),
   pontas Start/End (`paint_markers.rs:142`), perfil de largura (hoje só no popover da barra,
   `bars_control.rs:274-276`).
3. **Pilha do motor** `VecPath.paints: Vec<PaintEntry>` (`lib.rs:517`, `paint_stack.rs:82-105`; kind
   Fill|Stroke, enabled, opacity, blend) vira linhas extra dentro de Fill/Stroke, em vez de secção
   própria. **RISCO MAIOR**: conferir primeiro como a secção «Aparência» a mostra hoje. A base
   `fill: Option<Paint>` / `stroke: Option<StrokeSpec>` fica como linha 0.
4. **Cap/Join/Align em segmentados de ÍCONE** (SVGs novos: cantos e pontas desenhados), só tokens.
5. Manter losangos de Auto Key em todo campo; o popover de traço da barra (`paint_stroke_popover`,
   `bars_control.rs:283`, partilha `stroke_body`) tem de abrir o mesmo conteúdo.
6. Cenas de smoke 45/46, fotografadas com `fotografa_cena.sh` ANTES de pedir ao dono.

Ordem actual do Inspector: `paint_body_plan.rs:343-356` (Aparência → Fill → Fill type → Textura →
Stroke → Textura do traço). Corpo do Stroke: `paint_sections_stroke.rs:25-260`.

## 6 — Números contados

- Efeitos: **21**; filtros: **15**; SVGs novos: **36**; testes novos: **5**; commits sobre o main: **40**.
- `PROJECT_SCHEMA` = 211. Cenas `PH2D_OBJECT_MODE_SMOKE`: `1|2|4|6..44`.

## 7 — Prompt de continuação (janela seguinte)

```
/pd-linha-assumir line/UIUX — Worktrees/line-UIUX
Próximo item: F44.
Handoff: docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F43.md

F44 (dono 01/11): Fill e Stroke do Vector no layout do Figma; «modernizar Cap, Join, Align num modo
moderno como Figma, com ícones inteligíveis sempre que possível». Referências: img_F44/.
Siga o plano do §5 do handoff; comece por conferir como a «Aparência» mostra a pilha de paints.
Tokens/i18n (zero hex, só inglês); Auto Key nos campos; o popover da barra segue abrindo o mesmo conteúdo.
Antes de pedir aprovação, monte as cenas 45/46 e me mostre a foto.

Feche com /pd-onda-fechar e PARE no meu smoke, em passos numerados. Handoff e prompt da janela
seguinte só DEPOIS do smoke. ⛔ Não integre nem faça push: só por ordem minha.
```

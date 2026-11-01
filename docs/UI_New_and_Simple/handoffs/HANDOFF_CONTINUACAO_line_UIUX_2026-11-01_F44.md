# HANDOFF de continuação — line/UIUX, F44 (01/11) → F44b

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `715bd8522f` (F44), merge-base com o
main `422fec2027` (o main não andou). **Nada integrado, nada enviado.** A F44 foi ao **smoke do dono em
01/11 e REPROVOU em 5 pontos** (§3). Antes de ler: `cd` + `pwd` + `git branch --show-current`. Handoff
anterior: [F43](HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F43.md).

## 1 — O que a F44 fez (commit `715bd8522f`)

- **Fill** (`ph2d-panel-vector/src/paint_fill_stroke.rs`): cabeçalho com «+» (`VECTOR_PAINT_ADD_FILL`,
  botão simples via `RowCtx::section_header_with_action`); linhas = camadas extra da pilha `paints`
  (topo em cima, só `is_fill`) + a linha de BASE (`fill: Option<Paint>`). Linha = caixa (`Dropdown`)
  com amostra + texto (hex, ou o NOME DO TIPO — é isto que o dono reprovou), `%` solto, olho, «−».
  A caixa da base abre `fill_props_body` (ex-`fill_type_section`, `paint_fill_type.rs`) à esquerda.
  `VECTOR_SECTION_FILL_TYPE` removido (VECTOR_SECTIONS 34 → 33).
- **Stroke**: «+» = `VECTOR_PAINT_ADD_STROKE`, ou `VECTOR_STROKE_PRESENT` numa forma sem traço; linhas de
  cor (camadas + base, o «−» da base = `VECTOR_STROKE_PRESENT`); linha fixa Position (3 glifos
  `StrokeInside/Center/Outside`) · Weight (`VECTOR_WIDTH_NUM` solto, px) · ⚙ (`VECTOR_STROKE_SETTINGS`,
  `Dropdown`). O ⚙ abre `stroke_settings_body` (`paint_stroke_settings.rs`): tinta, Cap/Join em glifos
  (`RowCtx::icon_segmented`), Weight/Color token (`token_row_named`), tracejado, pontas, perfil. O painel
  da barra ② (`bars_control::stroke_rows`) pinta `stroke_lines(.., in_bar=true)` + o mesmo corpo.
- **Painéis à esquerda**: `fx_side::side_panel` / `side_head` generalizados (F43 + F44);
  `paint_side.rs` pinta Fill/camada/Stroke settings a partir de `fill_stroke::take_abertos()`,
  `on_click` fecha/acompanha (↑↓ seguem a camada). A camada aberta (`state::open_layer`, lida pela
  shell para largura/dx/dy/blend) é DERIVADA do `Dropdown` `vector_paint_row_id(i)` a cada quadro;
  `vector_paint_row_opacity_id(i)` (novo) aponta a camada e viaja como `VECTOR_PAINT_OPACITY`
  (`event_paint_stack::value_event`). `StackVerb::Open` saiu de `ph2d-app-vec/paint_stack.rs`.
  `pub fn open_paint_layer(store, i)` para cenas sem rato.
- **A Aparência** ficou só com opacidade + mistura do objecto; `paint_stack_rows.rs` ficou só com
  `layer_props` (largura, X/Y, offset de CAD com quina em glifos, mistura) e o popover de mistura.
- **editor-core**: `panel_chrome::paint_segmented_icon_group` (glifo = altura da peça),
  `property_row::paint_icon_choice_row`, `RowCtx::icon_segmented`; 10 SVGs novos `cap-*`, `join-*`,
  `stroke-inside/center/outside`, `stroke-settings`.
- **Auto Key**: losango por canal — cor na coluna (base: cor+opacidade = um canal); olho e `%` de camada
  colados ao controlo; Position colado aos glifos; Weight na coluna. `marcas.rs`:
  `vector_paint_row_opacity_id` → `PILHA_OPACIDADE`.
- **Dívida da linha fechada no bloco**: censo de canais (isenções F43 com nomes velhos →
  `fx_list_row`, `filter_props`; novas `layer_line`, `paint_line`, `token_row_named`), tectos de LOC
  (`ph2d-panel-hierarchy/src/row.rs` 609 → 560 com `row_icon_color_tests.rs`; `ph2d-tokens/src/color.rs`
  732 → 688 com `color_value.rs`).

## 2 — Gates e prova

- Novos: `tests/it/f44_fill_e_stroke_do_figma.rs` (9). Adaptados ao lugar novo: `seam_paint_stack`,
  `seam_tokens`, `seam_key_mark*`, `seam_stroke_*`, `seam_texture_pattern`, `fill_chip_labels_fit`,
  `marker_rows`, `popover_band`, `seam` (33 secções). Helpers em `tests/it/main.rs`: `host_fill`,
  `host_traco`, `host_paineis`, `host_pilha`.
- Verde: `ph2d-panel-vector` 307 + 50; editor-core `architecture*` 119, `icons`, `segmented`;
  `ph2d-app-vec` `paint_stack`, `marcas`, `vector_mode`; clippy `-D warnings` limpo nas 7 crates
  tocadas; `cargo fmt --all`. Sem suíte completa (é do integrador). **Mutação: não feita.**
- ⚠️ Lei REVOGADA nesta onda: «as rows do traço ficam visíveis sem traço» → no Figma, sem traço só o
  «+» (`seam_stroke_present::a_shape_without_a_stroke_shows_only_the_plus`).

## 3 — Smoke do dono (01/11): REPROVADO — o que ele pediu

Fotos: `img_F44/smoke_dono_ref_figma.png` (Figma) × `img_F44/smoke_dono_ph2d_hoje.png` (nós); cenas
`img_F44/cena45.png`, `cena46.png`.

1. **O tipo (Linear…) fora da caixa de cor**: a caixa é SÓ amostra + hex, bem definida (no Figma o
   degradê mostra-se na amostra; o tipo mora no painel de ajustes).
2. **Todas as linhas iguais**: a base e as extras com TODAS as opções da espécie (fill: tipo, degradê,
   padrão; stroke: tinta, cap, join, tracejado, pontas, perfil, largura) e TODOS os botões (olho, «−»).
   ⚠️ Hoje a base não tem olho porque `VecPath::fill: Option<Paint>` / `stroke: Option<StrokeSpec>` não
   guardam «escondido», e as camadas extra (`PaintEntry`) só editam cor sólida/largura/offset/mistura.
3. **Position em padrão CIRCULAR** (um círculo com o traço dentro / no centro / fora — o ícone do Figma).
4. **Botão de ajustes também no Fill**, consistente com o do Stroke.
5. **Alinhamento**: colunas iguais em todas as linhas (caixa, `%`, olho, «−» no mesmo `x`); hoje a base
   sem olho/«−» estica a caixa e desalinha (foto).

## 4 — F44b: plano proposto (a confirmar no código antes de mexer)

- **(a) Linha única** (`paint_line`): sempre `[amostra hex] [%] [⚙] [👁] [−]`, colunas fixas; a amostra
  pinta o DEGRADÊ quando a tinta não é sólida (prever: ColorSwatch com gradiente ou mini-rampa).
- **(b) ⚙ por linha** abre o painel da TINTA daquela linha (fill: tipo/degradê/padrão/token/regra;
  stroke: o `stroke_settings_body` todo); a caixa abre o seletor de cor, como no Figma. Decidir se o
  ⚙ do Stroke continua na linha fixa ou passa às linhas (o dono pediu consistência).
- **(c) Base com olho e «−»**: precisa de MODELO. Opções: (1) campo novo no `VecPath` (`fill_enabled`,
  `stroke_enabled`) — mexe no schema (`PROJECT_SCHEMA` 211, recontar com `scripts/schema-recount.py`);
  (2) a base passa a ser a camada 0 da pilha — refactor maior. ⚠️ Medir o custo e escolher o
  padrão-ouro (decisão técnica é delegada).
- **(d) Camadas extra com tudo**: `PaintEntry::kind` já guarda `Paint`/`StrokeSpec` completos; falta a
  edição por índice (verbos `paint_stack::set_*` na app-vec e a rota da shell — ⚠️ `the_shell_only_shrinks`:
  a rota nova vai para a crate `ph2d-app-vec`, não para a shell).
- **(e) Glifos `stroke-inside/center/outside` circulares** (círculo + anel dentro/centro/fora).
- **(f) Cenas 45/46** refeitas e fotografadas (`docs/Components/ferramentas/fotografa_cena.sh`), mais uma
  linha EXTRA de cada espécie com degradê/tracejado para provar o «todas iguais».

## 5 — ABERTO (fora da F44b)

- Provas de mutação dos gates F43 e F44 (fecho da linha).
- Integração: só por ordem do dono (41 commits sobre o main; `collision-surface.sh` pelo caminho absoluto).
- Não rastreados antigos (`.log`/`.png` em `img_F30/F33/F34/F40/F41`): não são da F44.

## 6 — Números contados

Commits sobre o main: **41**. `VECTOR_SECTIONS`: **33**. SVGs novos F44: **10**. Testes novos F44: **9**.
Cenas `PH2D_OBJECT_MODE_SMOKE`: `1|2|4|6..46`. `PROJECT_SCHEMA` = 211 (a F44 não o mexeu).

## 7 — Prompt de continuação (janela seguinte)

```
/pd-linha-assumir line/UIUX — Worktrees/line-UIUX
Próximo item: F44b (smoke da F44 reprovado em 01/11).
Handoff: docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-01_F44.md (§3 = o que pedi, §4 = plano)
Fotos: docs/UI_New_and_Simple/handoffs/img_F44/smoke_dono_*.png (Figma × hoje)

Fill e Stroke, segunda volta:
1. O tipo (Linear etc.) sai da caixa de cor; a caixa é só cor + hex, bem definida.
2. TODAS as linhas iguais, base e extras: todas as opções da espécie e todos os botões (olho, «−»).
3. Ícones de Position em padrão circular, como o Figma.
4. Botão de ajustes também no Fill, igual ao do Stroke.
5. Tudo alinhado como no Figma: as mesmas colunas em todas as linhas.
Só tokens e i18n, zero hex, app só em inglês; Auto Key em todo campo. Cena de smoke refeita e
fotografada antes de me pedir; corra você mesmo o smoke e me diga o resultado.
Feche com /pd-onda-fechar e PARE no meu smoke, em passos numerados. Handoff e prompt da janela
seguinte só DEPOIS do smoke. ⛔ Não integre nem faça push: só por ordem minha.
```

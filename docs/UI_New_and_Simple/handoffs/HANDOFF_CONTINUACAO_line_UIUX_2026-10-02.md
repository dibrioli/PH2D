# Continuação — `line/UIUX`, 2026-10-02 (a linha NÃO fechou)

> Leitor: a próxima janela desta linha (`/pd-linha-assumir`, bloco do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).
>
> **Estado:** a linha foi reaberta sobre o `main` `1ad60a1ce` com `reset --keep main` (o cherry deu
> 0, por isso não houve rebase). **HEAD `99e88a4a6`, 9 commits, árvore limpa.** O smoke do dono foi
> aprovado em cada onda.
>
> **Ordem do dono para esta janela (02/10):** *«vamos fazer tudo que está em aberto»* — os itens
> do §4, por esta ordem, e depois o fecho (§5). O mecanismo de cada commit está na mensagem dele;
> aqui fica o estado, o que vem e o que já foi decidido.

## §1 — O que a linha entregou hoje (9 commits sobre `1ad60a1ce`)

| commit | o quê |
|---|---|
| `0db5b4754` | A legenda de chip é uma palavra SOLTA: o orçamento é o passo do chip (`widget::paint_caption`, `sub_label_font_px`, `entry_gap_px`). `SETTINGS` → `PREFS`. |
| `a412d9aa5` | O pulldown da fila é tão largo quanto a face mais larga que pode mostrar. Novos: `Compound.faces/row_w`, `size_row_pulldowns`, `entry_advance(.., axis)`, `AreaMenu.faces` e `ph2d-panel-model3d::{VIEW_FACES, SHADING_FACES}`. |
| `5554c0f8a` | O número que não cabe perde CASAS, arredondado (`widget::numero_que_cabe`, na caixa e no chip, fora da edição). Cortes de LOC: `tool_rail/{geometry,caption}.rs`. `text_elide::em_todo_estilo` (cfg test). |
| `238b52654` | Esta continuação (versão anterior). |
| `a0e8e26d0` | Barra de estado apertada: só o TEXTO encolhe, os curtos ficam inteiros (`teto_comum`); `preferred_width(&mut TextSystem)` passa a ser MEDIDA. |
| `4ae8d7e05` | ⭐ **A escala da interface inteira** — *Edit ▸ Preferences ▸ Interface scale*. Ver §2. |
| `463ab3181` | Barra de estado: a largura do texto VIAJA calculada (a cura do ULP, `totais`). |
| `99e88a4a6` | Degraus de 80 % e 90 % (pedido do dono). Menu de `80` a `200 %`; a fábrica fica em `100 %`. |

**Gates novos (17):**
- `nenhuma_legenda_da_fila_de_ferramentas_e_cortada`, `nenhuma_legenda_da_barra_do_topo_e_cortada`
  e `nenhuma_face_de_pulldown_da_fila_e_cortada`;
- `the_view_pulldown_knows_every_face_the_camera_can_give` (`ph2d-app-field3d`);
- `perde_casas_arredondando_e_guarda_a_unidade`, `no_piso_o_numero_sai_inteiro_e_arredondado` e
  `um_decimal_que_nao_cabe_sai_arredondado_e_inteiro`;
- `apertada_cada_texto_cabe_no_seu_segmento` e `na_largura_preferida_nada_se_corta_em_estilo_nenhum`;
- `a_cem_por_cento_e_o_caminho_de_sempre` e `as_vistas_do_mundo_caem_no_mesmo_pixel_em_toda_escala`;
- `a_shell_pergunta_ao_chrome_pelas_portas_fisicas`;
- `the_factory_is_one_and_the_steps_climb` (tokens), `the_factory_is_the_identity_and_the_round_trip_returns`
  (editor-core) e `every_ui_scale_survives_a_round_trip` (shell);
- os dois testes da tabela do menu.

## §2 — Como a escala da interface funciona (leia ANTES de mexer em chrome, ponteiro ou passes)

Spec com oráculos e medições: [`../spec/05_a_escala_da_interface.md`](../spec/05_a_escala_da_interface.md).

- **O chrome vive em píxeis LÓGICOS** (janela / `s`). A janela, a câmera, o ponteiro do winit e os
  passes de GPU ficam FÍSICOS. A porta única entre os dois é `ph2d_editor_core::ui_scale`
  (`UiScaleMap`).
- **Saída.** `ui_scale::pintar_no_chrome(scale, viewport_físico, scene, |vp_lógico, cena| …)` pinta
  numa cena lógica e cola-a sob `Affine::scale(s)` (`VectorScene::append_transformed`). Usam-na
  `screens::hero::paint_hero_screen_na_escala` (+ a forma de onda do áudio) e os toasts/jobs
  (`fase_hero_chrome_tail`). A `100 %` é o caminho de sempre, byte a byte.
- **As vistas do mundo** que a shell publica em físico (`grid.view`, os 7 `GizmoView`,
  `point_view`, `gizmo.drag`, `dragging_files`) entram lógicas durante a pintura e voltam físicas
  (`na_escala.rs::Fisico`). Fica físico: as contas da shell (arrasto, projeção) e as fases que
  pintam em coordenadas do mundo (seleção, física, guias do vetor, anel do pincel, retângulo de
  seleção).
- **Entrada: as portas FÍSICAS do `HeroScreen`** são `chrome_hit`, `chrome_panel_at`,
  `panel_rect_fisico`, `handle_pointer_fisico`, `handle_wheel_fisico` (posição E passo) e
  `escala()`. Na shell, `App::ponto_do_chrome` (`dock_resize.rs`).
  ⛔ **O gate `a_shell_pergunta_ao_chrome_pelas_portas_fisicas` proíbe**, na shell, nos
  `ph2d-app-*` e no `ph2d-viewport3d`: `hit_index.hit(`, `.store.panel_at(`, `.store.panel_rect(`,
  `.handle_pointer*(` e `.handle_wheel(`. **A 100 % (a suíte inteira) os dois espaços coincidem:
  nenhum outro gate apanha um ponto cru.**
- **Rects do chrome que vão a contas físicas** passam por `rect_to_physical`:
  `canvas_area::visible` (câmera, palco, 3D), os obstáculos do navball 3D e a régua de guias.
- **Preferência.** `UiScale` (ph2d-tokens) → `HeroScreen.ui_scale` → `prefs.txt` `ui_scale=` →
  submenu `SettingsScaleSubmenu`. A marca de seleção lê `ui_scale::active()`.

## §3 — Decidido e MEDIDO (não re-litigar)

| decisão | porquê / medição |
|---|---|
| ⛔ `MIN_W_PX 72 → 85` recusado | 13 gates de coluna de rótulo reprovaram (Inspector a 220 px: 18 → 35 nomes cortados); A/B com 72 ⇒ verdes. Escolha do dono: **menos casas**. |
| ⛔ Escala pelos TOKENS (`EDSCALE`) recusada | 1 647 `LITERAL-PX-OK` e ~80 constantes de compilação (`ROW_H_PX` em 73) ficariam de fora. |
| A arte NÃO acompanha a escala | Só o chrome cresce; as réguas marcam ~88 px por 100 unidades a 100 % e a 200 %. |
| Nenhum degrau desativado | A 200 % em 1366×768 (683 lógicos) o layout degrada bem (`⋯`, elisão); fotografado. |
| Pulldowns: botão mais largo | Escolha do dono, contra «só ícone» e «abreviar». |
| Degraus `80·90·100·125·150·175·200` | Oráculos corridos: Blender `ui_scale` 0,5–6,0; Godot `display_scale` Auto·75…200·Custom. |

## §4 — O que fica ABERTO (a ordem do dono: fazer tudo)

1. **HiDPI — a escala do SISTEMA.** O app ignora o `scale_factor()` do winit (lido em
   `shells/desktop/src/winit_host.rs:33`, guardado num `Cell`, nunca usado). Num ecrã 4K ou num
   portátil com escala do SO, tudo sai minúsculo. A escala da interface tornou-o barato: `s` =
   `scale_factor × UiScale`.
   - `UiScaleMap::new(UiScale)` passa a aceitar o factor do ecrã (construtor novo; `is_identity`
     só a 1,0 × 100 %).
   - `pintar_no_chrome` recebe o `UiScaleMap` em vez de `UiScale`.
   - O hero precisa do factor do ecrã. ⚠️ **`hero.rs` está em 700/700 linhas**: um campo novo exige
     corte no mesmo ficheiro (a cura é cortar, nunca subir o tecto).
   - **Corra os oráculos primeiro:** como o Blender combina DPI com `ui_scale` e o que o `Auto` da
     Godot lê.
   - Gate: a combinação dos dois factores e o clique a cair no mesmo id com `scale_factor` 2,0.
   - Foto em tela virtual: o `kwin_wayland --virtual` aceita escala de ecrã? Meça.
2. **O trilho VERTICAL legado (`F9`)** continua a cortar a face dos pulldowns (`G…`), porque a
   coluna tem largura fixa (`RailButtonSize::rail_width_px`). Só a fila horizontal foi curada.
   - É decisão de PRODUTO: pergunte ao dono com as opções medidas. Por exemplo: a face rodada como
     a legenda, abreviar só na coluna, ou a coluna larga o bastante.
   - O gate tem de varrer `RailAxis::Vertical`.
3. **Nitidez do texto em escala fracionária** (`125`, `175 %`). O *hint* e o `snap_x` alinham à
   grelha LÓGICA; sob `Affine::scale` essa grelha deixa de ser a do ecrã. Nas fotos ficou bom, mas
   **não foi medido**.
   - Meça a nitidez das bordas de um rótulo a `100` vs `125` vs `175` (recorte da foto).
   - Decida pela medição: desligar o hint fora de `s` inteiro, ou levar `s` ao `TextSystem`.
4. **Menores, registados:** as palavras das ferramentas de imagem vêm do registry, e os gates do
   `ph2d-editor-core` só vêem a lista de reserva (hoje todas têm ≤ 5 letras e cabem). A pressão da
   caneta não chega ao app (`vec_app_bridge.rs:57`), mas é de outra linha: **não é desta**.

## §5 — O fecho que a linha deve (DIRETRIZ §1.5.9, `/pd-linha-fechar`)

- **Mutação** (agente `mutacao`) de cada gate novo do §1: cada cura desfeita sozinha ⇒ o gate tem
  de reprovar. Já provados:
  - `as_vistas_do_mundo_caem_no_mesmo_pixel_em_toda_escala` (sem a conversão da vista, a alça cai
    157 px fora);
  - o controlo do censo (acusa 16 no próprio núcleo; a 1.ª redação deixava passar 2 consultas
    partidas em linhas).

  Os «antes da cura» do §1 são vermelhos vistos, não mutações.
- **Gate batched completo** sobre o diff acumulado. O último `nextest-impacted` inteiro correu
  ANTES de `a0e8e26d0`…`99e88a4a6`; depois disso só correram os gates tocados (verdes).
  - `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets`
  - clippy `--all-targets -D warnings`
  - `cargo machete`
  - `check-standalone-optional.sh` e `check-workflow-packages.sh`
  - `fmt --check` e o scan `typos` do `ship.sh`
- **Handoff de INTEGRAÇÃO** em `docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_<data>.md`,
  com a superfície de colisão do §6 e a linha do `CLAUDE.md` §5.1 (o link do handoff).
- `bash scripts/agent-loop-profile.sh` (colado no handoff) e `rm -rf target/*/incremental`.
- **Último passo:** `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`
  corrido 2× (a 2.ª saída no handoff, sem `Compiling`).

## §6 — Superfície de colisão acumulada (para o handoff de integração)

- **Falha ALTO noutra linha (não compila):**
  - `ToolRailEntry::Compound` ganhou `faces` e `row_w`; `entry_advance` ganhou `axis`;
    `bar_rail`/`bar_split`/`publish_overflow` pedem `&mut TextSystem`;
  - `AreaMenu.faces`; `StatusBar::preferred_width(&mut TextSystem)`;
  - `HeroScreen.ui_scale` (um literal da struct noutra linha precisa do campo);
  - `paint_topbar_rail_chip` ganhou `vao`.
- **Funde LIMPO e REPROVA depois:** o censo das portas físicas. Uma linha que escreva
  `hero.hit_index.hit(`/`hero.store.panel_at(` na shell ou num `ph2d-app-*` reprova na árvore
  combinada. A cura é a porta física, nunca uma isenção.
- **Enums e ids append-only:**
  - `ContextMenuKind::SettingsScaleSubmenu`;
  - `UiScale` (7 degraus: `P80`, `P90` ANTES do `P100`, que é `#[default]`);
  - ids `CTX_MENU_SETTINGS_SCALE` e `CTX_MENU_SCALE_{80,90,100,125,150,175,200}`;
  - i18n `chrome.menu.interface_scale` e `chrome.menu.scale_*`; `chrome.topbar.pill.settings` = `PREFS`.
- **Muda comportamento:**
  - a fila de ferramentas fica mais larga (pulldowns); os gates de tablet e do `⋯` estão verdes;
  - um decimal que não cabe sai arredondado em vez de `…`;
  - a fixtura de `the_app_never_reshapes_a_still_screen` liga o chrome legado (o app caiu a 1 021
    textos com as legendas inteiras).
- **Shell:** `+147 −88` linhas (líquido **+59**); a catraca `the_shell_only_shrinks` está verde.
  **`hero.rs` 700/700, `paint.rs` (hero) 700/700.**
- Contadores (`PROJECT_SCHEMA`, registos, ADR, `Cargo.lock`): intocados. Contratos congelados: nenhum.

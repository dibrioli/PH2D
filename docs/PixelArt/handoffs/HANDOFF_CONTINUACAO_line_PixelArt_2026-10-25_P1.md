# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P1 (Desenho: 14 ferramentas, pixel-perfect, simetria, Tiled) — 2026-10-25

> Leitor: a próxima LLM que assume a linha numa janela nova. A onda P1 está **fechada e com o smoke do
> dono APROVADO (25/10: «smoke OK»)**; a **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada
> integrado, nada pushado. Lei do módulo: [ADR-0192](../../architecture/decisions/0192-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> · [plano](../02_plano_modulo.md) · [handoff P0b](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md) · [handoff P0](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `c3662cd8e4`.
- **29 commits** na linha (`git log --oneline c3662cd8e4..HEAD`); HEAD `47e2573f7a` (antes deste handoff).
- Desta onda: `70eec37b4d` (núcleo) · `83eabe246a` (tela) · `fbfe971359` (fecho da onda + fotos) · `16e1092acd` (smoke do dono: halo do cursor) · `47e2573f7a` (`PH2D_PIXEL_DIAG`).
- `git diff --stat 76d7fa707d..HEAD`: 51 ficheiros, +3740/−191.

## 2. Estado do módulo

Igual ao do [P0b §2](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md) (Draw, coluna à esquerda, painel `pixel_tools` à direita, formato `PH2DPIXEL` 1.0), com: **14 ferramentas** (eram 4), pincel (círculo/quadrado/linha, tamanho 1..64, ângulo), pixel-perfect ao vivo, simetria com diagonais e eixos arrastáveis, Tiled Mode por eixo, balde com tolerância, spray. Opções por ferramenta no painel Pixel. Smoke `PH2D_PIXEL_SMOKE=1|2`.

## 3. O que a P1 fez

**Passo 1, como o dono pediu: vetores do MVP em JSON ANTES do código.** `docs/PixelArt/ferramentas/extrai_desenho_mvp.mjs <caminho de docs/Pixel Art/app>` → `crates/ph2d-pixel-core/tests/fixtures/mvp_draw.json` (238 casos, 9 famílias: pixel_perfect batch+live, curve, rect, ellipse, stamps 1..16 círculo/quadrado, fill contíguo, imagens da simetria, o funil `expandStamp` com carimbos pares/em L × modos × tiling, `splitmix32`).

**Núcleo `ph2d-pixel-core`:**
- `raster.rs`: `line_path`, `is_l_corner`, `pixel_perfect`, `PixelPerfect` (streaming com retração), `curve_path` (sem tecto de 4096 passos — custo linear), `rect_cells`, `rounded_rect_cells` (raio do Aseprite 1.3.17; a célula do meio de um quadrante pertence aos dois cantos), `ellipse_cells`/`filled` (Zingl por bbox), `square_box`, `snap_pixel_angle` (0/26,57/45/63,43/90), `polygon_cells` (par-ímpar no centro da célula, linhas semiabertas) e `js_round` = `Math.round` do MVP.
- `brush.rs`: `Brush` (Circle/Square/Line + tamanho + ângulo), `MAX_BRUSH=64` (= máximo do slider do Aseprite), `CIRCLE_SLACK 0.1` autoral, quadrado rodado pelo teste do centro da célula, pincel-linha com round-half-down; `SplitMix32`, `hash3`.
- `symmetry.rs`: `Mirrors` h/v/d1/d2, eixos em MEIAS células (`axis2`), imagens = grupo gerado (H+V=4, d1+d2=4, H+d=8), deslocamentos reflectem pela parte linear (MVP W24), `Tiling` por eixo, `expand` = o funil único.
- `fill.rs`: flood scanline com pilha explícita 4/8, `Bucket` com tolerância por canal RGBA, Indexed+tol 0 compara o índice, contíguo/global, amostra Layer/Visible, todo alfa 0 é igual.
- `stroke.rs` reescrito: `Paint {brush, symmetry, tiling, pixel_perfect}`, **CONTAGENS por célula** (a retração do pixel-perfect nunca apaga a célula de outro ponto), `set_shape` (contorno com o pincel, interior liso), fill (sementes simétricas), spray (semente = `hash3` da célula inicial). ⚠️ `Stroke::begin(sprite, layer, frame, value, paint, (x,y))` mudou de assinatura (só as crates pixel a chamam).

**Vocabulário `ph2d-editor-core::pixel_panel`:** `PixelTool` 14 (Pencil, Spray, Eraser, Eyedropper, Bucket, Line, Curve, Rectangle, FilledRectangle, Ellipse, FilledEllipse, Contour, Polygon, Hand), `shortcut()`/`shifted()`/`shortcut_label()` («Shift+» por i18n `pixel.shortcut_shift`), `uses_brush()`, `paints()`; `PixelBrushShape`; `PIXEL_MAX_BRUSH` (teste prova = `MAX_BRUSH`); `PixelOptions` (defaults do Aseprite); `PixelPanelInfo.options`; `PixelRequest::Options`.

**Família `ph2d-app-pixel`:**
- `gesture.rs` novo: `Kind` (Free, Spray, Box, Curve em duas fases, Contour, Polygon, Fill, Axis); `Mods {alt, shift, ctrl}` com `Mods::of(ModifiersState)`; `paint()/symmetry()/bucket()` traduzem as opções; `reshape()` = a mesma lei para os gestos E para a cena de smoke.
- `input.rs`: press/moved/release por `Kind`; `continue_pending` (curva: ctrl; polígono: vértice, o primeiro canto fecha, cada clique refaz a forma); `finish/cancel/set_tool` (trocar de ferramenta termina o pendente); Shift+clique no lápis/borracha traça linha a partir de `Doc.last_point`; arrasto de eixo (`AXIS_GRAB_PX` 4 px de ecrã, encaixa em meias células, `Doc.axis2`); teclas B/Shift+B/E/I/G/L/Shift+L/U (repetir → preenchido)/Shift+U/D/Shift+D/H; Enter termina, Esc cancela.
- `frame.rs::spray_tick` (borrifa a cada quadro enquanto premido); `draw.rs` eixos da simetria + vizinhos do Tiled Mode; `panel.rs` aplica `Options`; `compose.rs`: `pointer/pointer_moved` recebem `Mods`; `PH2D_PIXEL_DIAG=1` regista cada tecla que chega à família.
- `smoke.rs` nível 2 (ver §4).

**Painel `ph2d-panel-pixel-tools`:** ids (TOOLS 14, SHAPES, NUMBERS, CHECKS 8, SEGMENTS 4); NUMBERS com faixas tamanho 1..64, ângulo −180..180, canto 0..64, tolerância 0..255, largura do spray 1..64, velocidade 1..128 — ⚠️ **a faixa é a faixa de arrasto do campo numérico**, ou seja, a precisão do gesto; `seam` (Click em segmentos; Toggled em caixas inverte o valor do MODELO; ValueChanged em números lê `store.number_value`); `paint.rs` secção de opções por ferramenta via `property_row` (`paint_field_row/paint_choice_row/paint_check_rows`, `Seccao::medida`), números sincronizados do modelo a cada quadro com `set_number_value`; coluna em 3 grupos (mão 5, formas 8, vista 1) + cores. **Chave do título renomeada** `panel.pixel.title` → `panel.pixel_tools.title` (gate `the_name_of_a_panel_has_one_source`; família nova nesta linha, sem isenção `PREFIXO_ANTIGO`).

**Ícones:** 7 SVG novos `docs/design/icons/pixel-{contour,curve,ellipse,ellipse-filled,rect,rect-filled,spray}.svg` + variantes `IconId` depois de `Pivot` em `icons.rs` e `icons_tests.rs::ALL_ICONS`.

**`ph2d-pixel-render`:** `draw_symmetry_axes`; `stroke_halo` (anel escuro `HALO` 0,85 de alfa + núcleo claro; **núcleo de largura PAR = 2×`LINE_PX`**: as arestas das células caem em píxeis inteiros e um núcleo de 1 px se esborrava 50 % em cinza) usado pela borda, pelo cursor de célula (`CURSOR_CORE` branco) e pelos eixos.

**Shell:** só `shells/desktop/src/input_dispatch/despacho_pixel.rs` (`Mods::of` em vez de `alt`; mesmo número de linhas). **i18n:** `ph2d-i18n/src/pixel.rs` (ferramentas, dicas, opções do painel, connect4/8, `shortcut_shift`).

## 4. Smoke do dono — APROVADO em 25/10

Fotos: [`smoke_P1_2026-10-25.png`](smoke_P1_2026-10-25.png) (HOME de fábrica) · [`smoke_P1_dono_2026-10-25.png`](smoke_P1_dono_2026-10-25.png) (HOME=/home/enio; `~/.ph2d/layout.txt` copiado antes, sha256 igual antes/depois) · [`smoke_P1_cursor_2026-10-25.png`](smoke_P1_cursor_2026-10-25.png) (halo do cursor).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=2 ./target/smoke/ph2d-host-desktop`
2. Abre um sprite 64×48 com uma casa já desenhada (chão, paredes, telhado, porta, sol, fumaça). Na coluna da esquerda: as 14 ferramentas em três grupos; no painel **Pixel**, as opções da ferramenta activa.
3. Teste: **Line** (`L`) com Shift segurado; **Rectangle** e **Filled Rectangle** (`U`, de novo `U` = preenchido); **Ellipse** (`D`); **Curve** (arraste a linha, depois clique para curvar); **Polygon** (clique cada canto, clique no primeiro para fechar, `Enter` termina, `Esc` cancela); **Bucket** (`G`) dentro da parede; **Spray** com o botão premido.
4. Ligue a simetria no painel, arraste um eixo com o rato (encaixa em meias células); ligue o Tiled Mode: aparecem vizinhos. `Ctrl+Z` desfaz um desenho por vez (6 passos na casa).
5. Cursor e eixos têm anel escuro e centro claro: visíveis sobre qualquer cor.

Deu errado se: uma ferramenta não aparecer na coluna, o polígono não fechar no primeiro canto, o `Esc` não cancelar uma forma pendente, o balde vazar para fora das paredes, o spray só borrifar ao mexer o rato, ou o cursor sumir sobre uma cor parecida.

Os dois reports do dono: (1) cursor/gizmo de cor fixa invisível sobre cores parecidas → halo (`16e1092acd`). (2) «Esc não cancela o polígono» → **NÃO reproduzido**: pelo `App::key_input` injectado dentro do processo (sonda temporária em `fase_app_scene_smokes`, removida), com cliques reais por `on_cursor_moved/on_mouse_input`, o Esc cancela, também com um campo numérico do painel focado. Foi acrescentado `PH2D_PIXEL_DIAG=1`; o dono respondeu «smoke OK». ⛔ **Lição:** para dirigir teclas numa sessão de foto use `App::key_input` por dentro (como `build_smoke_drive::smoke_key_z`); `xdotool`/XTest NÃO chega à sessão `kwin --virtual` (nem uma letra).

## 5. Superfície de colisão NOVA desta onda

Soma-se às de [P0 §3](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md) e [P0b §5](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md). Contadores ECS/schema/`MODULE_TRUTHS`/NodeId não mudaram nesta onda.

- `ph2d-editor-core/src/pixel_panel.rs`: `PixelTool` passou de 4 a 14 variantes; `PixelPanelInfo` ganhou `options` — **todo literal de `PixelPanelInfo` noutra linha precisa dele**.
- `ph2d-editor-core` `icons.rs` + `icons_tests.rs`: 7 variantes inseridas por ordem alfabética depois de `Pivot` — outra linha que acrescente ícones conflita aqui; regenere por ordem de slug. Mais `docs/design/icons/` (7 ficheiros).
- `ph2d-i18n/src/pixel.rs`; `ph2d-panel-registry-init/tests` (2 literais de `PixelPanelInfo`).
- Shell: `input_dispatch/despacho_pixel.rs`.
- `ph2d-pixel-core`: `Stroke::begin` mudou de assinatura (só crates pixel chamam).
- Merge pode partir: outra linha com ícones novos (ordem em `IconId`/`ALL_ICONS`); outra com literal de `PixelPanelInfo`; chave `panel.pixel.title` (agora `panel.pixel_tools.title`).

## 6. Prova de fecho (por nome, tudo verde)

- `ph2d-pixel-core`: `draw_parity` 8 (todos contra o fixture) · `draw_laws` 11 (leis próprias, cada uma com controlo; fill 4096² sem recursão, 0,9 s o teste inteiro) · `stroke` 8 · `codec` 4 · `history` 1.
- `ph2d-app-pixel`: `desenho` 11 · `edit::` 15 · `painel` 6. `ph2d-panel-pixel-tools`: `seam` 7.
- `ph2d-panel-registry-init`: `pixel` 3 (+1 ignorado) · `the_name_of_a_panel` 3.
- `ph2d-editor-core`: `icons` 8 · `pixel` 41 · `tool_bar` 12 · `the_shell_only_shrinks` 1 · `no_authoring_tool_shares_the_canvas` 3.
- `ph2d-app-registry-init`: `every_object_mode_has_a_composed_family` 4. `ph2d-host-desktop`: `pixel` 26 · `literal` 1 · `icon` 1.
- `ph2d-i18n`: `a_tabela_inglesa` 4 · `a_fronteira_dos_motores` 2 · `tests::` 4.
- Clippy `-D warnings --all-targets --keep-going` verde em ph2d-pixel-core, ph2d-pixel-render, ph2d-app-pixel, ph2d-panel-pixel-tools, ph2d-editor-core, ph2d-i18n, ph2d-panel-registry-init, ph2d-host-desktop. `cargo fmt --all --check` limpo. Binário de smoke: `target/smoke/ph2d-host-desktop`.

## 7. ABERTO

- **Mutação** das leis novas, no fecho da linha: retração do pixel-perfect com contagens · célula do meio do rect redondo · regra semiaberta do polígono · balde por índice em Indexed tol 0 · fecho do grupo da simetria + reflexo dos deslocamentos · Tiling por eixo · encaixe do eixo em meias células · `continue_pending` refaz a forma · largura PAR do `stroke_halo`; mais as antigas ([P0 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md), [P0b §7](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md)).
- Falta: pré-visualização da pegada do pincel ao passar o rato com tamanho > 1 (o cursor mostra uma célula).
- Polígono, Contour e Spray **não têm MVP nem oráculo executável** (o Aseprite Trial não salva nem corre scripts): as leis vêm do estudo + testes próprios.
- Licença do Aseprite (plano §7): pergunta ao dono continua aberta.
- Overlay por cima do mundo = P14; conversão RGB→Gray provisória = P2.
- Fecho da linha pendente: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`; a suíte completa é do integrador.

## 8. Próximo item

**P2** (cor, paleta, inks, Indexed completo, RGB↔Gray↔Indexed com dither) segundo plano §5/§6 (P1 ∥ P2 ∥ P3, depois P4); o dono disse «vamos seguir com o plano».

Primeiro passo de qualquer onda: extrair os vetores do MVP (`docs/Pixel Art/app/tests/*.test.js`; para a P2: color, palette, dither, quantize, ramp_quantize, overlay_color, cycling) para JSON, como `extrai_desenho_mvp.mjs`, e só depois o código. Fotografe também com `HOME=/home/enio` (copie antes `~/.ph2d/layout.txt`).

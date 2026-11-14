# HANDOFF — rodada de integração de 13/11 (5 linhas: GameEngineFuncional, BarraDeMenus, TimelinePrincipal, JointEditor, PainterWatercolor)

Leitor: a próxima LLM e o próximo integrador. Ramo `integ/2026-11-13` (worktree `Worktrees/integ-2026-11-13`), main de partida `4f46a3573c`, tip final = HEAD do `main` depois do `--ff-only` (o commit deste handoff é o último), CI = run do `spike.yml` desse push. Curas: `git log --oneline --reverse 4f46a3573c..integ/2026-11-13 --grep='integ 13/11'` (60 commits). Total no ramo: 810 commits desde o main de partida.

## Ordem, medida por merge-tree par a par

| # | Linha | Handoff da linha |
|---|---|---|
| 1 | GameEngineFuncional | [topdown](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-12_TOPDOWN.md) · [adendos](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-12_TOPDOWN_ADENDOS.md) · [projectile e checkbox](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-13_PROJECTILE_E_CHECKBOX.md) |
| 2 | BarraDeMenus | [linha](../../BarraDeMenus/handoffs/HANDOFF_INTEGRACAO_line_BarraDeMenus_2026-11-13.md) |
| 3 | TimelinePrincipal | [plano 17 onda 5](../../Timeline/handoffs/HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md) |
| 4 | JointEditor | [W1–W3](../../JointEditor/handoffs/HANDOFF_INTEGRACAO_line_JointEditor_2026-11-13_W1_A_W3.md) |
| 5 | PainterWatercolor | [missão 4K](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_MISSAO_4K.md) · [pós-4K](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-11_POS_MISSAO_4K.md) · [desempenho total](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md) · [luz e camadas](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md) · [mistura e envio](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_MISTURA_E_ENVIO.md) · [seleção, camadas e ritmo](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_SELECAO_CAMADAS_E_RITMO.md) · [N8/N9](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_N8_N9_DESEMPENHO.md) · [onda 7](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_ONDA7_DESEMPENHO.md) · [primeiro toque 4K](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_PRIMEIRO_TOQUE_4K.md) |

Já no main (0 commits por patch-id, fora): InputMap, ScriptEditor, SfxLab, PixelArt.

## Painter: a ordem «não integrar» era à linha

O handoff do Painter dizia «não integrar: dono 13/11». Era ordem à LINHA; o dono ordenou esta integração. A suíte mostrou os alarmes de pincel vermelhos e o dono mandou regravá-los («corrija tudo, sem smoke»): ver Curas.

## Contadores (DELTA)

- `PROJECT_SCHEMA` 237 → 242, tripla `(242, 13, 29)`. Degraus: 238 Timeline `Interp::Spline` · 239 Timeline `HandleKind` · 240 JointEditor · 241 e 242 Painter. Reconta-se por `python3 scripts/schema-recount.py`.
- ADR: JointEditor tomou o 0201; os ADR do Painter 0201–0204 foram renumerados para 0202–0205 (arquivos renomeados, 137 referências, nota de número, índice regenerado).
- `CLAUDE.md` §5: `PH2D_PHYSICS_SMOKE=120..123`; `PH2D_GAME_SMOKE` com `CENAS` = 25 literal (ver abaixo).
- Cena 123 (JointEditor) → `ph2d-physics-smoke-scenes`.
- Crates novas: `ph2d-joint-assembly` (1 714, tecto 1 814) · `ph2d-painter-canvas-map` (724, tecto 824).
- Tectos de LOC (nenhum SUBIU para esconder): `main.rs` 986 → 984 e depois 1 008 → 977 (tecto baixado para 977) · `ph2d-app-kanban` −6 · `ph2d-app-components` 80 898 → ≤ 80 869 · `ph2d-app-game` 2 093 → 2 869 (recebe o que a components perdeu, −776/+776) · `ph2d-app-physics` 39 545 → 37 658 · `ph2d-app-painter` 24 606 → 23 882, tecto 24 429 → 24 359 · `ph2d-app-pixel` 11 291 → 11 267 (tecto 11 278) · `ph2d-app-joint-editor` 4 750 → 4 749.
- Censos e `recontar.py`: o B2 refaz-os sobre a árvore final.

## O que um merge pode partir

- **Barra × Timeline em `context_menu_overlay.rs`** (composto à mão): a Barra é a base para os menus da barra; a Timeline entra com ícones, `menu_w` medido, Delete vermelho e a marca do item nos menus de contexto. Fonte ÚNICA de «abre submenu»: `menu_row_look::abre_submenu(id)` = `menu_cascade::opens_submenu` + curvas/extrapolação da timeline; `risco_antes(id)` inclui `menu_bar::GROUP_BREAKS`. `paint_scene_list` saiu para `context_menu_scene_list.rs` (tecto de LOC 700). Chamador novo de «abre submenu» usa a porta, não duplica a lista.
- **`ph2d-tokens/src/lib.rs`:** o Mergiraf deixou DOIS `pub use chrome`; fica o da Barra + os três `TIMELINE_KEY_SEL_ANEL_*` da Timeline.
- **`CENAS` do `PH2D_GAME_SMOKE` = 25 literal** + `const assert` contra `projectile::CENA`: o `recontar.py` não segue `const` entre crates. Não volte a derivar.
- **Posse do teclado/rato do jogo** (`posse_do_jogo.rs`, o registo e o `game_mouse`): `ph2d-app-components` → `ph2d-app-game`; a isenção do registo viaja e sai do censo de palavras da components.
- **JointEditor × (GEF, Barra):** imports duplicados em `panel-inspector` e `app-physics` (um bloco só); o comando da aba na `DocMenu::New` com ícone `Joint` (a Barra trocou File por New/Plan e deu `icon` ao `DocCommand`); a sonda da ponte corre as seis vias; HR-15 da `ph2d-joint-assembly` (gate novo, 5 nomes de objecto) e 5 isenções mortas saem da app-physics. 14 mutações W3; 6 sobreviventes mortas por 5 testes.
- **`main.rs`:** o braço `CursorLeft` da linha Painter estava DUPLICADO e o segundo era inalcançável (`on_cursor_left` e `movimento_dos_controles` não corriam); um braço só (perde a mão + `pointer_leave` + controles). 14 doc-comments órfãos saíram: [COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-13.md](../../archive/integracao-jornadas/COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-13.md).
- **Painter:** `CanvasMap` e o gate dele vivem na crate nova, re-exportado como `canvas_map`; gates que leem o ficheiro pelo caminho seguem-no. A miniatura do cartão do Pixel é `ph2d_pixel_io::frame_rgba_reduzido`. Âncoras de `the_build_only_sees_the_visible_region` e do gate do véu recortado (`painter.wetness_veil`, B16.10) atualizadas para a forma do rustfmt: a lei não mudou.
- **Isenção nomeada** da sonda `PH2D_FAIXA_LOG`.
- `collision-surface.sh` dá «SONDA CEGA» para `ph2d-render`/`ph2d-script` (as consts mudaram de ficheiro; `recontar` acha 122 nos dois): a ferramenta precisa de conserto.

## Suíte completa

Quatro suítes completas:

1. Pré-voo vermelho: `clippy::large_enum_variant` (`Interp::Spline` com 216 B) → `Escrita::Chave` leva a `Interp` em `Box`; `typos`.
2. 36 354 testes, **115 vermelhos**.
3. Pré-voo vermelho: import morto, `HashMap` proibido (gate da marca), laço de 1 elemento (censo do Painter), `fmt`.
4. 36 359 testes, **13 vermelhos**.

Depois: as curas abaixo e o B2. Depois das curas: B2 (`integrar-rodada.sh b2`: as falhas por nome + nextest-impacted + censos + architecture_* + recontar) — o ship só sai com ele verde; o registo é `.git/ph2d-suites-verdes.tsv`.

## Curas pós-suíte

60 commits `integ 13/11` no ramo (`git log --oneline 4f46a3573c..HEAD --grep='integ 13/11'`). Agrupados:

**Painter (BUGS_painter #115, [doc](../../Painter/BUGS_painter.md))**
- DECISÃO do integrador: as leis da missão 4K valem **só no Wet Paint**; Digital, Impasto e Watercolor ficam ao byte do main. Restrições: B29.1 (tecto u16, `755e86cd05`; gate `fora_do_wet_paint_o_traco_fica_no_tecto_u8`) · B29.3 · B22.1 (passo fixo de 1/60 s só em aguada e Wet, `1e690abd4c`) · B16.2 cont3 (`4f791f1c1f`) · B31.2 (`a75e48f06d`) · B33.1 (2.ª leitura do papel, `24ff377cb6`, `1df792fe47`; com Tiling o período do sprite, `cb09e97587`).
- Defeito do papel do Wet curado em `5618e94ace` (`alturas_do_papel_de_fabrica` voltou à 2.ª leitura; bissecção do Oil Bristle). A luz do Impasto deixa de decidir o realce pelo meio da ferramenta (`cf553bf327`).
- Regravações citando a lei adotada: cena 6 da Shape Color e Soft Pastel (B32.1, `ea2024a050`), Watercolor Basic (B22.1), `CENAS`, exemplos de preset (só `"schema"` 1→2, B50.1), shape_color/opções/aerógrafo (B33.1).
- Alarmes de pincel regravados por ordem do dono 13/11 «corrija tudo, sem smoke»: relatórios em `docs/Painter/handoffs/relatorio-pinceis-integ-2026-11-13/`, `…-so-wet/`, `…-oil-bristle/` (45 miniaturas WetPainter + larguras; Digital/Impasto/Watercolor intactos no fim).
- Recursos: tecto de memória do Dabber **2 477 MiB** medido sob carga; **ABERTO: reter um rascunho por gesto** (o do Real Oil é retido por operário do rayon). Régua do custo do Filbert 1,5× → 1,2× (as cerdas baixaram o custo; main 58/96 ms → integ 39/53). Rótulo «Edge Darkening (%)» → «Dark Edge (%)». Vista a 400 %: tolerância 3× → 2×.
- Réguas e gates: a Borracha leva o corpo do Impasto (B06.1, BUGS #70); pen-down pelo mínimo de 5 amostras quentes; `arch_color_space_typed`, `arch_mode_has_reconcile`; 1.ª marca do Wet; restauro fora do casco em paralelo; ADR do Painter renumerados.

**Interface**
- Barra de menus: bug de coordenadas físicas × lógicas em `faixa_unica.rs` (a faixa pergunta ao chrome em lógicos). `ramo_faixa_unica` saía cedo no topo de `pointer_button`: passou a correr logo depois de `ramo_aperto_solta_teclado` (4 leis de ordem verdes).
- Caixa em moldura (GEF): `check_em_fila`/`marca_em_fila_em` (barra de uma linha, Flip frames); `HitIndex::register_label`.
- Ciclo `grid_snap → property_row → panel` quebrado: as medidas da caixa descem a `widget::checkbox`; a lei do alvo do cartão da chave sai de `screens::hero::key_card` para `interaction::key_card_alvo.rs` (catraca interaction→screens 24→18).
- Timeline: o «8» da régua recontado (abas de vista fora, colunas do fluxo); censos dos painéis recontados pelo medido; paleta da aba de scripts (`>save script`).

**Jogo e física**
- `posse_do_jogo` → `ph2d-app-game` (e `posse_do_jogo.rs` 737 → 270 com testes em `tests.rs`). Registo da física 49 → 50 (`JointAssembly`). Dash da fixture do smoke no tique 410 (400 é tique de pulo desde o Bug #14 GEF). A Luz 2D remonta a vista quando o lado do dispositivo muda (`f5ebe7e829`). `det-math`: `powi(2)` → produto explícito. Nomes de código = rótulo nos campos novos do JointEditor e da GEF (chaves serde intactas); `Escrita::Chave` com `Interp` em `Box`.

**Tectos movidos (nenhum subiu para esconder)**
- `CanvasMap` → `ph2d-painter-canvas-map` · lei da junta → `ph2d-joint-assembly` (1 814 → 2 175; `ph2d-app-physics` 37 798 → 37 437, soma igual; `ph2d-app-joint-editor` 4 756 → 4 748) · `frame_rgba_reduzido` → `ph2d-pixel-io` · `main.rs` 977 · `ph2d-i18n` lib.rs 702 → 627 (`wet_tuning.rs` irmão).

## Para o dono ver (smoke dele; não feito, por ordem)

- Menu de botão direito composto Barra × Timeline (ícones, Delete vermelho, marca do item, submenus): nenhuma linha o viu inteiro.
- Pincéis Wet regravados: abrir os relatórios `docs/Painter/handoffs/relatorio-pinceis-integ-2026-11-13*/`.
- Rótulo «Dark Edge (%)» (antes «Edge Darkening (%)»).
- Vista a 400 %.
- Merge Down do Pixel Normal Map: fica abaixo da dobra na janela padrão.
- Já pendentes das linhas: barra de menus (menus no W, busca no centro); Timeline acréscimos 13–14; JointEditor (`PH2D_PHYSICS_SMOKE=123`); Painter smokes 8 e 9.

## ABERTO

- Smokes do dono acima. Integrar não é aprovar.
- `collision-surface.sh`: conserto da «SONDA CEGA» (ph2d-render/ph2d-script) pendente.

## Smokes (do repo principal, depois do ship)

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run --profile smoke -p ph2d-host-desktop
cd /home/enio/Documentos/Projetos/PH2D && PH2D_PHYSICS_SMOKE=123 cargo run --profile smoke -p ph2d-host-desktop
cd /home/enio/Documentos/Projetos/PH2D && PH2D_ANIMACAO_SMOKE=1 cargo run --profile smoke -p ph2d-host-desktop
```
Os smokes por linha estão nos handoffs da tabela; sem `PH2D_CARDS=0` no ambiente.

## Prova de fecho

B2 verde e `ship.sh` sobre a árvore com este handoff; CI = run do `spike.yml` desse push.

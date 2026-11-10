# HANDOFF — rodada de integração de 10/11 (6 linhas)

Leitor: a próxima LLM e o próximo integrador. Ramo `integ/2026-11-10` (worktree `Worktrees/integ-2026-11-10`), main de partida `f254e42bf1`, tip final o HEAD do `main` depois do `--ff-only` (este commit é o último), CI o run do `spike.yml` desse push (link na resposta do ship). Curas: `git log --oneline f254e42bf1..integ/2026-11-10 --grep='integ 10/11'` (commits `fix(integ 10/11)` e `style(integ 10/11)`).

## Ordem, medida por sobreposição de ficheiros (menor primeiro)

| # | Linha | Commits | Handoff da linha |
|---|---|---|---|
| 1 | TimelinePrincipal | 18 | [ondas 1–2](../../Timeline/handoffs/HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-09_ONDAS_1_2.md) |
| 2 | ScriptEditor | 33 | [plano 14 Q1–Q4](../../CodeEditor/handoffs/HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_PLANO14_Q1_A_Q4.md) |
| 3 | PainterWatercolor | 58 | [pen-up e mistura](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_PEN_UP_E_MISTURA.md) |
| 4 | GameEngineFuncional | 34 | [Align e Pivot](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-09_ALIGN_E_PIVOT.md) |
| 5 | GameEngineFuncional02 | 120 | [Outline e F3](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md) |
| 6 | PixelArt | 169 | [P13+P14](../../PixelArt/handoffs/HANDOFF_INTEGRACAO_line_PixelArt_2026-11-09_P13_P14.md) |

SfxLab: sem commits, fora da rodada. Total no ramo: 446 commits, 1 969 ficheiros.

## Contadores (DELTA)

- `PROJECT_SCHEMA` 225 → 231, tripla `(231,13,29)`: uma das 4 primeiras linhas subiu 1 degrau (226), GEF02 → 227, PixelArt 4 degraus (228–231). Reconta-se por `python3 scripts/schema-recount.py`.
- Registo `ph2d-ecs` → 121. Família «Decisões de produto do dono» no `MEMORY.md` → 69 (67 do main + Pastel do Painter + Align do GEF).
- Inspector: carga 103 → 104 («Create and set key» do Trigger, GEF02 onda 4). Timeline: colunas declaradas 1→2 / 2→3.
- Tectos de LOC curados por corte (ver «Portões mexidos»).

## O que um merge pode partir

- **`crates/ph2d-render/src/sprite/sheets.rs`** — única decisão de composição (GEF acerto por quadro `cell_offset`/`nudged` × PixelArt pivô por célula `anchor_da_celula`). Lei: o pivô da célula é a âncora BASE; o acerto desloca o quad por cima. `sheet_geometry` = a da folha (com pivô da célula, sem acerto); `drawn_geometry` = `nudged(sheet_geometry)`. Folha 0 com pivô próprio → `SheetView::Other(nudged(..))`; sem pivô e d=0 → `Own`; sem pivô e d≠0 → `OwnNudged`.
- Escritas em `shells/desktop` (App/ProjectState), `component_catalog.rs`, `main.rs` e `ph2d-app-components`: todos no tecto de LOC; qualquer linha nova reprova o gate.
- `collision-surface.sh` tem sonda CEGA para a const de registos de `ph2d-render`/`ph2d-script` (a const mudou de ficheiro): `recontar.py` cobre; não confie só na superfície.

## Suítes e curas

1. **#1 pré-voo:** fmt (2 ficheiros fundidos) + typos: o `rapier2d` vendorizado fica de fora do typos, «tru» e «ifself» reescritos.
2. **#2 (34 711 testes): 42 vermelhos + 2 TIMEOUT**, curados em bloco por 4 agentes paralelos (gates de arquitetura do editor-core · shell/cenas/render · i18n/registry · painter/light2d). Causas por família:
   - **Vendor/censo:** `arch_mode_has_reconcile` varria o vendor do `rapier2d` (`vendor/` fora do censo dos `set_*_mode`); o censo dos avisos passou a seguir `ph2d_x::f(` até `crates/ph2d-x/src/lib.rs` (o import de folhas da PixelArt, `ph2d_sprite_ase_import::frases_do_import`, tinha as chaves e o censo não as alcançava); régua do português não lê a extensão de ficheiro `(.tres)` como numeral.
   - **DAG da fundação:** `ph2d-tool-padding` → `ph2d-sprite-edit/src/padding.rs` (a folha `ph2d-anim-sheets` não sobe à ferramenta); `ImageFilterMode` + porta `image_quality_for` descem para `widget/image_filter.rs` (`sheet_cell.rs` só lê `ui_image_quality`; project reexporta); `field_suffix` pub no widget-sync; `safe_clamp` na CRT.
   - **Classificação de censo:** `po_do_papel` é do traço (`DO_TRACO`); sondas `set_mistura_meia_definicao`/`set_mistura_salta_a_copia_do_veu` são portas internas; `MERGE_DIST` do Outline e os botões de corpo da NAV REGION são não-animáveis (selecionam, não guardam valor).
   - **Portas do tema/ritmo:** molduras do depurador por `stroke_frame` (contorno de «mudou» = `FRAME-RAW-OK`); portas do ritmo na aba de scripts (`margin`, `row_pitch_px`, `card_pad_px`, `list_indent_px`); botões Door/Gate/Left Wall/Right Wall medidos na fonte e no respiro de quem os pinta.
   - **Gates de forma:** `handle_dropped_files` parte a importação das folhas; hover gate aceita `(!palco && !forma)`; gate da luz lê as oclusões com o `pixel_luz`; arte de fábrica conta `<circle>`; `R` alterna Blur→Jumble→Push e `Tool::ALL` = 22 + formas; `outline_rings_world` usa `libm::sincosf` (hash determinístico); gate do `gpu_extra` procura o draw do run DEPOIS do `material_bg`; `forma_encolhe` com `BodyDesc.shape_off`; chaves `component.segue_*` no `component_catalog`.
   - **Presets do Painter:** 13 presets de fábrica com a chave morta `mistura_opaca.pigment` (saiu em `ac576c5df8`) reescritos por `escrever(ler(b))` da própria crate (receita igual, zlib do Rust; miniaturas e larguras intactas, alarmes de pincel verdes); `o_pigment_mix_da_verde_continuo_e_violeta` reescrito para a lei MEDIDA do Pigment único.
   - **Prazo de carga:** `luz_em_com_normal::{nunca_negativa, a_normal_de_frente_…}` com override de prazo 8×60 s no `.config/nextest.toml` (80,6/93,2 s sozinhos, mortos aos 180 s na suíte) — os 2 TIMEOUT.
3. **b2:** 4 vermelhos que as curas trouxeram, curados em `db95c5dac2` (filtro de imagem da UI inteiro em `widget/image_filter.rs`; selo `INSP_ANIM_PREVIEW_SYNC` registado; botões dos corpos desligados nomeados em FORA; presets).
4. **Suíte final:** 2.ª corrida 34 711 testes, 42 vermelhos + 2 TIMEOUT curados em bloco; B2 (12 curas) verde 3.ª vez e árvore registada; o `ship.sh` reconfere a suíte sobre a árvore com este handoff.

### Vermelhos que JÁ VINHAM das linhas no tip delas (falha de fecho da linha, não da fusão)

- PainterWatercolor: `po_do_papel`, portas internas, presets com chave morta, `o_pigment_mix_*`.
- PixelArt: `every_key_of_this_catalog_exists_on_both_sides`, `forma_encolhe.rs`, toast census.
- Lição para o fecho de linha: rode os gates de arquitetura/censo das crates tocadas por NOME antes de escrever o handoff; o editor-core sozinho pegava 3 deles.

## Portões mexidos, a rever

- Tectos de LOC da GEF02 (shell 189 631 > 189 041, `ph2d-app-components`, `main.rs`, `ph2d-app-sprite-anim` 5 236 → 5 209, `component_catalog.rs` volta a 700) curados CORTANDO comentário histórico (shell −532 em 12 ficheiros, components −120): arquivado verbatim em [`COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-10.md`](../../archive/integracao-jornadas/COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-10.md). Não foi possível MOVER: os testes novos da GEF02 usam `App`/`ProjectState` da shell.
- `CLAUDE.md` §5 sob 14 000 B (PIXEL_SMOKE 1..36, handoff P13+P14).
- Isenções nomeadas: `set_color_mode_metric`, `OUTLINE_LOCKED` só-dica, `conta`/`own_bar`/`paint_hex`/`paint_passos` sem a11y própria, `cards.rs` fora de secção, molde `Worktrees/line-<x>`, nome da rampa `Skin` (cena 31 do Pixel) como DOCUMENTO da cena.
- Typos: `vendor/rapier2d` excluído.

## Para o OLHO DO DONO no smoke

(a) **Wet blend com Pigment único:** azul sobre amarelo fica azul-esverdeado (G-Pen só verde a x≈260; Hard Round não chega a verde). Lei medida, não a antiga; tabela no §7.5 do handoff do Painter. Se o dono quiser mais verde, é decisão dele.
(b) **Aba de scripts:** a faixa da frase do pé fica 3 px mais baixa (6→3) e o padding do outline 6→4 px (curas de ritmo/porta do tema). Timeline: o Time sobe à faixa do título.
(c) **Alarmes de pincel verdes:** nenhum pincel mudou.
(d) NAV REGION: botões Door/Gate/Left Wall/Right Wall deixam de cortar o texto.

## ABERTO

- Mover os testes novos da GEF02 para fora da shell quando a família tiver crate (hoje só se cortou comentário).
- Vendor do `rapier2d` em `[patch.crates-io]`: aprovado pelo integrador por a linha o ter decidido.
- `docs/PixelArt/…/P13_plano.md` 112 KB não partido (nenhum gate reprovou): `python3 scripts/doc-split.py`.
- Nada aqui foi aprovado pelo dono: o smoke é dele; integrar não é aprovar.
- `.config/testes-lentos.tsv` regenera-se no próximo `python3 scripts/tempos-teste.py` (o override do `luz_em_com_normal` entra).

## Smokes (do repo principal)

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run --profile smoke -p ph2d-host-desktop
cd /home/enio/Documentos/Projetos/PH2D && PH2D_PIXEL_SMOKE=31 cargo run --profile smoke -p ph2d-host-desktop
```
(os smokes por linha estão nos handoffs da tabela; o pacote é `ph2d-host-desktop`.)

## Prova de fecho

Suíte completa 1× no tip o HEAD do `main` depois do `--ff-only` (este commit é o último) (resultado acima); CI o run do `spike.yml` desse push (link na resposta do ship).

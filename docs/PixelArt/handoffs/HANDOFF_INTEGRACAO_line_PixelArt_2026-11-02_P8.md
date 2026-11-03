# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (P7 + P8: pincéis, dinâmica, filtros, Gradient, Text) — 2026-11-02

> Leitor: o agente integrador (lote, `scripts/integrar-rodada.sh`). A linha **fecha e PARA**: não integra, não faz push.
> ⚠️ **A P7 vai junto e NÃO está integrada**: [handoff P7](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-02_P7.md) continua
> válido (superfície §3, mutações 13/13, hook). Este documento acrescenta a P8. Plano: [P8_plano.md](../P8_plano.md).
> Continuação (o que existe, leis medidas): [P8a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_P8a.md) ·
> [P8b](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_P8b.md).

## 1. Identidade

- Branch `line/PixelArt`; worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt`.
- Merge-base `3df5894489`; `git log --oneline 3df5894489..HEAD` = P7 (até `0f748e8666`) + P8 (de `b63e7758ba`
  em diante) + o fecho. P8c (esta janela): `63272e6bf9` cena 19 num bloco · `79978d81d8` fotos ·
  `fa1f43f9de` o que as fotos mostraram · `181e9b2ff8` lista Kernel pelo rect publicado · `f2ba63987e` e
  `1a321ebf15` sobreviventes das mutações · `febfe9f257` dinâmica Gradient (P7) · `7337a71b22` fecho (§4) · este handoff.
- **Smoke do dono da cena 19 (refeita): PENDENTE.** P7 (cena 18) aprovada em 02/11. Integrar não é aprovar.
- Sem rebase sobre o `main` nesta linha (ordem: não integrar); a árvore combinada é do integrador.

## 2. O que a P8 entrega (resumo; detalhe nos handoffs P8a/P8b)

- **Núcleo `ph2d-pixel-core`** (leis do Aseprite Trial 1.3.18, fixturas `oracle_trial_*.json`, `oracle_trial_round3.json`):
  `filter/` (Invert, Brightness/Contrast `⌊y·255,5⌋` 52 LUTs, Curve, Hue/Saturation 4 modos + Alpha, Despeckle, Outline,
  Blur/Jumble, Convolution `clamp(trunc0(Σw·v/div)+bias)` com 37/39 kernels gerados por `docs/PixelArt/ferramentas/fit_convolucao*.py`),
  `gradient.rs` (Linear/Radial, Solid/Bayer, Palette ramp), alvo Selected|All, Indexed funciona.
- **Crate NOVA `ph2d-pixel-text`**: BDF/HEX/FNT/FON, fábrica Cozette/Tom Thumb/Unscii (licenças junto), rasterizador Mono/AA
  sobre `skrifa` 0.44 com hinting, dropout SCANTYPE, tom do Trial, `carimbo.rs` (Fonte, `tinge`, `over`, lista de fontes).
- **App `ph2d-app-pixel`**: `filtros.rs`, `ajustes.rs`, `degrade.rs` (Gradient Shift+G, Blur/Jumble R), `texto.rs`, `solta.rs`
  (texto solto). **Painel/UI**: Edit ▸ Fill/Stroke/Invert/Adjustments/FX/Insert Text; diálogos com Preview, OK = um passo.
- **P8c (esta janela, do smoke do dono)**: Gradient = região da SELECÇÃO ou do cel inteiro (sobrepõe; `gradient_oracle`
  continua verde — o Trial mediu sobre branco = cel inteiro); Text: 1.º Enter solta numa moldura (arrastar/rodar/escalar),
  2.º Enter ou clique fora = UM passo, Esc cancela; fontes em dropdown com PRÉVIA (fábrica → pixel populares instaladas →
  resto), tamanho válido (múltiplo da linha nas de fábrica; `SYSTEM_MIN_PX = 12` medido), fontes de símbolos mostram o nome na
  fonte da interface; Convolution: kernel num dropdown «Kernel» com rolagem que mostra só linhas inteiras e não encosta na
  borda da janela (1930×984, 1930×560, 1280×420); diálogo volta a 320 px.
- **P7 corrigida aqui** (`febfe9f257`): o carimbo do PREMIR da dinâmica Gradient já leva a fracção (o `with_gradient` vinha
  depois do `with_dab` e repunha a fracção cheia). Teste `the_dynamics_gradient_mixes_the_press_dab_by_the_matrix` (vermelho
  com a ordem antiga). **Deselect não é defeito**: `deselect_from_the_menu_or_ctrl_d_drops_the_mask_and_the_next_drag_selects_anew`.

## 3. Superfície de colisão (além da tabela §3 da P7)

| Onde | Aqui | Nota |
|---|---|---|
| Crate NOVA `crates/ph2d-pixel-text` | glob `crates/*`; `Cargo.lock` | `ph2d-app-pixel` e `shells/pixel-scenes` dependem dela |
| `ph2d-editor-core` FORA da família | `widget/dropdown/largura.rs` (`region_in_whole_rows`), `panel/dropdown_popover.rs` (`paint_dropdown_list_measured`), `widget/curve_editor.rs` NOVO (+ `curve_a11y`), `range_slider` (+ `range_a11y`), `widget/mod.rs` re-exports, `icons.rs`, `ids/chrome/mod.rs`, `dispatch/key.rs`, `pointer_down_menus.rs`, `state/chrome_ops.rs`, `property_row/escolha.rs`, `screens/hero/{paint,context_menu_dialogs}.rs` | append-only; `largura.rs` partilhado: a lei de 25/10 inalterada, função nova |
| `ph2d-editor-core/src/pixel_panel/font_preview.rs` | NOVO (prévia de fonte pela porta `paint_dropdown_list`) | |
| `ph2d-i18n` | `pixel.rs`, `pixel_dialogos.rs`, `pixel_pincel.rs`, `pixel_selecao.rs`, `lib.rs` (tabelas) | chave `panel.pixel.gradient` duplicada removida; colisão textual provável em `lib.rs` — mantenha as duas |
| `ph2d-system-fonts/src/library.rs` | `face_bytes` (nova, pública) | antes do `mod tests` (clippy, §4) |
| `shells/desktop` | `input_dispatch/{despacho_pixel,keyboard}.rs` | conta para `the_shell_only_shrinks` |
| Smoke `PH2D_PIXEL_SMOKE` | `max_level` **19** | |
| `CLAUDE.md §5` Pixel Art | 442 → **434 B** (smoke 1..19, este handoff, frase com filtros/Gradient/Text) | o §5 encolheu 8 B |
| `.claude/hooks/prazo-do-agente.sh` | o conserto `288d03792f` (P7) | ver §6 |

## 4. Gates do fecho (1×, sobre `3df5894489..HEAD`)

**fmt** (`cargo fmt --all -- --check`): verde nos ficheiros da linha. Vermelhos PRÉ-EXISTENTES no merge-base, alheios
(não tocados): `ph2d-panel-pixel-preview/src/paint.rs`, `ph2d-panel-registry-init/tests/it/the_vector_control_bar.rs`,
`ph2d-vec-entities/src/connector_handles.rs` — não comitados aqui.

**clippy** `PH2D_SUITE=fecho … --all-targets -D warnings` das 12 crates tocadas (`ph2d-app-pixel`, `-editor-core`, `-i18n`,
`-painter-brush`, `-panel-pixel-tools`, `-pen-law`, `-pixel-core`, `-pixel-io`, `-pixel-text`, `-system-fonts`,
`ph2d-host-pixel-scenes`, `ph2d-host-desktop`). Falhou em 5 corridas, curado no commit do fecho `7337a71b22` (sem `allow`):
1. `type_complexity` em `carimbo.rs::frame` → aliases `Moldura`/`Cursor`.
2. `items_after_test_module` em `ph2d-system-fonts/library.rs` → `face_bytes` movida para antes do `mod tests`.
3. `manual_contains` em `ph2d-pixel-text/tests/it/font_laws.rs` → `contains(&255)`.
4. `dead_code` `curve_a11y`/`range_a11y` (`bf785418bd` criou-as para o HR-12 sem re-exportar) → re-exportadas em
   `widget/mod.rs`, como o `build_a11y` público dos outros widgets. ⚠️ Nenhum painel as chama ainda (§6).

5. imports mortos (`PanelHostInternal`) em `seam_pixel_{texto,gradiente}.rs`; `int_plus_one` em `cena19.rs`;
   `needless_range_loop` em `texto_fontes.rs`. Depois: as 12 crates verdes.

**Portões por nome** (`ph2d-editor-core`), 1.ª corrida — 3 vermelhos, curados no MESMO commit `7337a71b22`:
- `the_shell_only_shrinks_and_every_app_crate_too`: `ph2d-app-pixel` **13 547 > 13 424** → `tests/it/catalogo.rs`
  (149 linhas) MOVIDO para `shells/pixel-scenes` (+ `Mesa::cena` em `comum.rs`). Agora ≈ 13 397 (SOMA entre linhas).
- `panel_functions_under_loc_cap`: `seam.rs::apply_event` 201 > 200 → o `THRESHOLDS` consumido no braço dos cliques
  que só consomem (199).
- `node_id_collisions::no_slug_is_declared_in_two_places`: `pixel.opt.text.font` em `smoke_efeitos.rs` e `ids.rs` →
  UMA definição `ph2d_editor_core::ids::PIXEL_TEXT_FONT_DD`; o painel re-exporta-a como `TEXT_FONT_DD`.

2.ª corrida, verdes: `no_magic_numeric`, `no_literal_color`, `o_claude_md_cabe_no_orcamento`,
`architecture_workspace_file_loc_cap`, `the_shell_only_shrinks*`, `node_id_collisions`, `architecture_panel_loc_cap`,
`_widget_loc_cap`, `_panel_wiring_parity`, `_interactive_crate_has_behavioral_test`, `hr12_widgets_a11y`, `largura`,
`dropdown_popover`, `curve_editor`, `range_slider` (26) · `ph2d-host-desktop` `file_loc_caps`/`fn_loc_caps` (6) ·
testes do item: `pixel-scenes` `catalogo`/`cena19`/`texto_fontes`/`texto::` (34), `panel-pixel-tools` `seam_pixel` (46),
`pixel-text` `font_laws`/`carimbo` (26), `system-fonts` `library` (7), `app-pixel` `desenho`/`edit` (28).
Censos da árvore combinada (`censos-da-arvore-combinada.sh`) NÃO corridos: sem rebase nesta linha (ordem do dono).

**Mutações (P8c)**: 46 mutações, 38 sangraram, 8 sobreviventes fechadas com testes (`f2ba63987e`, `1a321ebf15`).
**Mutações (plano §6)**: 43, 26 sangraram, 14 fechadas com testes; **O6** e **C2** inofensivos por projecto (C2: o
`skrifa` 0.44 `Charmap::map_impl` já filtra o gid 0), **AA3** sem fixture (aberto). Impactados e suíte completa ficam para
a integração.

## 5. Smoke (para o dono) e fotos

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=19 ./target/smoke/ph2d-host-desktop`
(sem features). O que mudou na cena 19: o boneco numa **camada transparente** por cima da Background e **sem selecção
pronta** (os ajustes e o Outline agem no sprite inteiro; a borracha APAGA nesta camada — na Background pinta o fundo, lei do
Aseprite); o **Gradient sobrepõe** ao 2.º arrasto; o **Text solta** ao 1.º Enter; **fontes com prévia** e tamanho válido;
**Kernel em dropdown**. Sonda `cena19` (por diálogo: B/C, Hue/Sat, Curve, Invert, Despeckle, Convolution, Outline; borracha).
Fotos 1930×984: `smoke_P8_cena19_{fabrica,dono,balde,hue,convolucao,texto,fonte,solto}_2026-11-02.png` (nesta pasta).
A aba «Vector» acesa na foto `dono` vem do `~/.ph2d/layout.txt` do dono (`active=vector`, decisão D3): não é desta linha.
`layout.txt` do dono: sha256 `759b6479…` igual antes/depois.

Binário do smoke (`--profile smoke`), 2.ª corrida:
```
▸ linha line_pixelart · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.38s
```

## 6. Aberto / avisos ao integrador

- `edges-find*`: alfa nunca medido (a lei dá 128 numa imagem opaca com A ligado).
- Divisor 0 da cor junto ao transparente = 1: escolha, não medida no Trial (K6 fixa-a por teste).
- Texto mono: 1–2 células por caso (14/15/24/32 px, Sans 8 «g») contra a FreeType; porte do rasterizador mono pede antes a
  triagem de licença (FreeType FTL/GPL dual; a FTL é permissiva com crédito, §0.9).
- AA3 sem fixture. `curve_a11y`/`range_a11y` existem e são públicas mas nenhum painel publica o nó na árvore AccessKit.
- **Hook `prazo-do-agente`**: o caminho relativo do `--git-common-dir` só está curado NESTA linha (`288d03792f`); até
  integrar, agentes com cwd na RAIZ da worktree.
- **`git commit -- <paths>` com agentes em paralelo** leva o ficheiro INTEIRO (aconteceu: `796645460f` levou edições
  alheias de `selection.rs`): ficheiros disjuntos por agente.
- Tectos: `ph2d-app-pixel` ≈ 13 397 / 13 424 depois do fecho (SOMA entre linhas; reconte na árvore combinada).
- `ph2d-editor-core/src/ids/chrome/pixel.rs`: +`PIXEL_TEXT_FONT_DD` (append-only, mesmo hash de antes).

## 7. `agent-loop-profile.sh` (20 sessões)

```
  ✗ paralelismo de ferramenta              1.08/passo   alvo: >= 1,5  (6% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                158   alvo: <= 800
  ✗ cargo test : cargo check                358 : 151   alvo: <= 1,0  razao 2.4x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             0.6%   alvo: <= 2%  (5 de 775)
  ✓ contexto relido por passo (media)         242 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               60 mil   alvo: <= 80 mil
```

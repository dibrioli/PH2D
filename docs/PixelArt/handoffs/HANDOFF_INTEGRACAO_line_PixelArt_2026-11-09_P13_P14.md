# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (P14 O sprite vive na cena · P13 O que só a PH2D tem) — 2026-11-09

> Leitor: o integrador. A linha fecha e PARA. Detalhe: [P14_plano](../P14_plano.md) §11 · [P13_plano](../P13_plano.md) §12. Antecessor: [P11+P12](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-08_P11_P12.md) · [continuação P14](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-09_P14.md).

## 1. Estado

- Branch `line/PixelArt`; `git rev-list --count main..HEAD` = 167 antes do commit deste handoff (164 das ondas + 3 do fecho, §8). **merge-base = `main` = `f254e42bf1`**: o main não andou ⇒ `--ff-only` sem rebase.
- **P14** — smoke do dono OK em 09/11 (cenas 26 e 27). **P13** — **SEM smoke**: decisão do dono 09/11, o smoke da P13 é DEPOIS da integração (§7). Integrar não é aprovar.
- Suíte completa e `nextest-impacted`: não correram (regra do dono 23/10). A crate `ph2d-panel-registry-init` INTEIRA correu no fecho (96 s) e deu 5 vermelhos do painel Lint, curados (§8).

## 2. O que entrou na P13 (gates principais)

- **Rampas** (`ph2d-pixel-rampa`, núcleo `ramp.rs`): `Palette.ramps`, `gera_rampa` em OKLCH. `a_rampa_gerada_e_a_do_coloraide` (ΔE OKLab 5,4e-5) · `a_rampa_gerada_e_monotona_em_l`.
- **Ciclo** (`cycle.rs`, `compose_window_em`): só Indexed; `t_ms = instante_do_quadro` em todo leitor; sprite de 1 quadro cicla com as voltas do Play. `o_ciclo_roda_so_a_rampa_e_volta_no_periodo` · `em_rgb_o_ciclo_nao_muda_nada`.
- **Quantização** (`trait Metrica` no núcleo; `Oklab`/`PreservaRampas` na rampa): Color Mode ▸ Indexed ▸ Color matching (Exact RGB omissão · Perceptual · Keep ramps); New Palette from Sprite leva as rampas. `o_indexed_rgb_de_hoje_nao_mudou_um_byte` · `o_indexed_rgb_e_o_do_imagemagick`.
- **Dither / Shading / Push**: Ink Dither (Level 0..=64, Bayer 2/4/8, preso ao canvas); Shading anda na rampa do documento; Smudge Push (`R` cicla Blur→Jumble→Push). `o_shading_anda_na_rampa_do_documento` · `push_arrasta_a_cor_e_fica_na_paleta`.
- **Luz na rampa** (`ph2d-pixel-rampa::luz`, `ph2d-pixel-luz::banda_rampa`): «Light: Smooth · Palette»; a luz troca o degrau, todo pixel aceso fica na paleta; +0,86 ms/quadro. `a_luz_neutra_na_rampa_e_o_quadro_de_hoje` · `a_placa_e_o_gabarito_concordam_na_rampa`.
- **Guias** (`ph2d-pixel-guia`, `ponte/alcas`): 1/2/3 PF + iso 2:1; Shift na Linha/lápis segue a guia; alças arrastáveis (1 passo «Move Guide»); secção Guides no menu Edit. `a_linha_guiada_vai_ao_ponto_de_fuga` · `a_iso_e_dois_por_um_exacto`.
- **Lint** (`ph2d-pixel-lint` + `ph2d-panel-pixel-lint`): Orphan, Doubles, Jaggies, Banding, Pillow, Too many colors; Window ▸ Lint. `cada_vicio_e_achado_onde_esta` · `clicar_no_doubles_centra_o_canvas_na_celula_e_corrigir_tira_o_grupo`.
- **CRT** (`ph2d-pixel-crt`): lei em Q14 inteiro, WGSL = gabarito `crt_cpu`; **prévia** (caixa CRT na Preview) e **jogo** (`palco::compoe` em `present_chrome.rs`); vista de sessão, nunca no documento. `a_placa_e_o_gabarito_concordam_na_crt` (0 níveis) · `a_crt_ligada_no_jogo_corre_um_passe_por_quadro_ao_centro`.
- **Efeito de camada** (`efeito.rs`, `ase/efeitos.rs`): Outline vivo = o filtro destrutivo (uma lei); Apply assa tudo num passo; `.ase` leva `ph2d.ramps/cycles/efeitos`, cel assado e `ph2d.cru`. `o_efeito_de_contorno_e_o_filtro` · `the_layer_outline_from_the_panel_shows_bakes_in_one_step_and_undoes_alive` · `o_ase_leva_rampas_ciclos_e_efeitos`.
- **Áudio** (`ph2d-pixel-audio`): `Sprite.audio: Vec<FaixaDeAudio>`; Timeline ▸ Add Audio…, onda, Mute/Remove; Play pede o som ao mixer real; arrastar a faixa = 1 `SetAudio`; scrub soa; `meta.audio` na folha. `o_som_manda_no_relogio` · `arrastar_a_faixa_dez_quadros_muda_o_inicio_num_passo_e_ctrl_z_volta`.
- **Tween** (`ph2d-pixel-tween`): Frame ▸ Tween… gera quadros REAIS (`t = k/(n+1)`), retocáveis; lei em `ponte::quadros::tween`. Gate `tween` (pixel-scenes) + crate inteira (5).

## 3. Superfície de colisão e contadores (DELTA)

- **Blob Pixel MINOR 8 → 9** (`ph2d-pixel-core/src/codec.rs:40`; a P14 deixou 8 com `Layer.normal`/`Op::LayerNormal`). Tags novas (`codec/doc.rs`): Sprite **17** `Ramp*` · **18** `Cycle*` · **19** `Guias` · **20** `Audio*` · **21** `luz_na_paleta` · Layer **9** `Efeito*`. ⚠️ O cabeçalho do `codec.rs` não lista a 21. Sem leitor velho (§0.11).
- **`PROJECT_SCHEMA` 225 → 229:** os 4 degraus são TODOS da P14 (226 ping-pong · 227 `SegueSlice` · 228 `LibraryDoc.normals` · 229 `SpriteCells`/`SegueCelula`); **a P13 não subiu**. Recontar: `python3 scripts/schema-recount.py <valor>`.
- **Smokes:** `PH2D_PIXEL_SMOKE=1..36` (`max_level: 36`, `ph2d-app-pixel/src/smoke.rs`; o `CLAUDE.md` do main diz `1..25`); próxima livre **37**.
- **Crates NOVAS** — P13: `ph2d-pixel-rampa` · `-lint` · `-guia` · `-crt` · `-audio` · `-tween` · `ph2d-panel-pixel-lint`. P14: `ph2d-pixel-cena` · `-cenas` · `-luz` · `ph2d-sprite-ase-import`. `Cargo.lock` já commitado; **regenere se o merge o tocar**.
- **Foundational (append-only):** `ph2d-audio` `StreamPlayhead` · `ph2d-app-audio` `engine_mut` · `ph2d-render` (`luz_rampa.wgsl`; `band_blit` bindings **4** banda de rampa e **5** LUT; da P14 `sprite.wgsl` `fs_main_n`) · `ph2d-editor-core` (`PixelInk::Dither`, `PixelTool::Push`, `PixelFrameAction::Tween`, `PixelEditAction::Guides`, hit `AudioTrack`, `PixelPanelInfo.ramps`, `PixelSpriteCard.luz_na_paleta`, `ColorModeChoice.metric`) · `ph2d-app-light2d`/`ph2d-light2d-*`, `ph2d-physics` (+vendor), `ph2d-ecs`, `ph2d-physics-ecs`, `ph2d-sprite-sheet` (P14).
- **Shell:** `present_chrome.rs` (CRT do palco) · `fase_object_mode.rs` (mixer, saldo 0) · `fase_vector_bands_doc.rs` (bandas normal+rampa) · `janela_do_jogo.rs` · `present_luz.rs`. **Saldo do diff inteiro em `shells/desktop/src`: +606 −708 (encolheu 102)** (`ase_import.rs` −348 foi para `ph2d-sprite-ase-import`).
- **Painel Lint registado em:** `ids/chrome/pixel_lint.rs` · `panel_ids` (nasce fechado) · `menu_bar::MODULE_TRUTHS` **26 → 27** · `menu_tables` · `hero/paint.rs` · `pixel_toggle` · `topbar` · `ph2d-panel-registry-init` (feature `panel-pixel-lint`).
- **i18n:** 9 tabelas Pixel entram por `pixel_tabelas.rs` (`#[path]`; `lib.rs` 700 → 688); novas `pixel_crt/efeitos/som/tween/guias/pintura`.
- **Dependência (P14, TEMPORÁRIA, pede APROVAÇÃO):** `[patch.crates-io] rapier2d = { path = "crates/ph2d-physics/vendor/rapier2d" }` + `exclude` no `Cargo.toml` raiz; rapier2d 0.36.1 com 1 mudança em `narrow_phase/pair_update.rs`, Apache-2.0 © Dimforge (`VENDOR.md`). Conferir `cargo deny`, licença por artefacto, `bash scripts/stack-audit.sh --tetos`.
- **Contrato §6:** nas linhas `+` de `git diff main...HEAD` o grep `trait Tool|NodeOp|OpResolver|NodeManifest|RasterEditTool|CanvasPaintTool|PanelEvent|enum VectorOp` não dá definição nem `impl … for`; as 7 linhas são USOS (`PanelEvent::Toggle` em `ph2d-app-host/src/smoke_drive.rs`) e prosa de `.md`.
- **Linha nova do `CLAUDE.md` §5 (410 bytes; a linha NÃO edita o arquivo):**

```
- **Pixel Art** — editor autocontido no idioma do Aseprite: camadas, quadros, paleta indexada com rampas e ciclo, pincel com dither, tilemaps, Slices, guias, lint, CRT, áudio e tween (`ph2d-pixel-*`, `ph2d-app-pixel`, `ph2d-recovery`). Smoke `PH2D_PIXEL_SMOKE=1..36`. Último: [P13+P14, 09/11](docs/PixelArt/handoffs/HANDOFF_INTEGRACAO_line_PixelArt_2026-11-09_P13_P14.md) · [docs](docs/PixelArt/README.md)
```

## 4. O que um merge pode partir

- `[patch.crates-io]` (wgpu-core + rapier2d) e `Cargo.lock`; escada do `PROJECT_SCHEMA` e a tripla irmã (recontar).
- `ph2d-panel-registry-init` `lib.rs`/`Cargo.toml` são GERADOS: painel de outra linha ⇒ `cargo run -p ph2d-panel-sync`, nunca à mão.
- `MODULE_TRUTHS` 26 → 27 e as contagens de registo de painel (`quantas_entradas_tem_cada_painel`, `the_window_menu_reaches_every_module`, `quantas_mensagens_o_painel_escreve`, `paineis_armados`).
- **Barra da Timeline reagrupada:** `ACTION_GROUPS` 5 → 4 (Delete com New/Duplicate), **Add Audio** no cabeçalho. Gate `os_cartoes::numa_janela_estreita_a_barra_quebra_e_nada_cobre_a_grelha`.
- Censos `every_word`/`every_key_of_this_panel_exists_on_both_sides` (Pixel tools, Timeline, Preview) leem as tabelas por caminho; tabela nova entra no censo.
- `the_shell_only_shrinks`, `architecture_no_dependency_climbs_a_layer`, `architecture_workspace_file_loc_cap` (núcleo: `sprite.rs`/`op.rs` a 697–698, `codec.rs` 667 ⇒ campo ou Op novos vão para ficheiros irmãos), `the_pixel_core_declares_no_dependency`.
- Gates de PLACA (CRT, rampa, luz com normal) pedem a vez ([A_VEZ_DA_PLACA.md](../../DevOps/A_VEZ_DA_PLACA.md)).

## 5. ⛔ Recusas MEDIDAS (não reconstruir)

- Matriz OKLab no `ph2d-pixel-core` / `core → ph2d-color`: duas verdades; o núcleo é sem deps (ADR-0196, `pure_core.rs`) ⇒ a conta vive na `ph2d-pixel-rampa`.
- CRT na CPU por quadro: 25,7 ms a 1 fio em 2048² (rayon 4 fios 8,6 · 8 fios 4,3 · 16 fios 2,65, mas rouba 4–8 fios ao jogo) ⇒ WGSL 0,098 ms; CPU é gabarito/export. Curvatura em f32: FMA muda o `floor` em bordas ⇒ Q14 inteiro.
- `magick -remap` «ao byte» como oráculo do Indexed: o Mapper do ImageMagick 7.1.2-35 erra **124/4096** (Pillow 13: 129; `-ordered-dither o8x8` difere 3529, FS 1089) ⇒ gate «onde diferimos a nossa está estritamente mais perto».
- Rampa como estado da seleção; ciclo em RGB; dither como variante de `Ink`; lint «quase-iguais» em RGB; luz na rampa por posterize por canal (sai da paleta). Tabela inteira: [P13_plano §10](../P13_plano.md).
- Áudio no blob (é asset); tween «vivo»; contorno com lei própria; CRT gravada no ficheiro (é vista).
- Herdadas da P14: ver a lista na [continuação](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-09_P14.md).

## 6. ABERTO (ordenado)

**Integrador / dono**
1. Aprovar o vendor do rapier (§3); se o crates.io publicar a cura, remover o `[patch]`.
2. Decisão de produto aberta (P13_plano §11 e P14): a cena 26 enquadra o herói a ~26 %; a normal importada aparece no Assets como `<cor>_n`; **`pngquant`/`grafx2`/`retroarch` por instalar (`sudo pacman -S --needed …`, senha do dono)** ⇒ `o_indexed_perceptual_nao_e_pior_que_o_pngquant` fica `#[ignore]`.
3. Na árvore integrada: suíte completa, `nextest-impacted`, gates de placa, `censos-da-arvore-combinada.sh`.
4. `P13_plano.md` tem 112 KB (passou do joelho ~100 KB): cortar com `python3 scripts/doc-split.py` e `bash scripts/doc-index.sh`.

**Da P13**
5. Guias: Retângulo em perspectiva; tamanho do losango iso na UI (`Iso2x1.passo` já está no codec); snap à grade iso; guias no `.ase`; a alça agarra com QUALQUER ferramenta (decidido; rever); gate que clique o id `pixel.view.guides.perspective_2`.
6. Lint: Banding só largura 1; Jaggies erra S-curvas bruscas; quase-iguais (OKLab) por fazer.
7. CRT: a janela da Preview de fábrica (altura fixa) **fica sem espaço para a imagem** com o cartão CRT (defeito achado na cena 34; a cena estica-a); a preferência não persiste; a Preview roda na CPU (`CrtGpu` espera porta de textura no `VectorScene`); resolução lógica do jogo não medida em janela real.
8. Efeito de camada: `ph2d.cru` (Vector U8) **dobra o `.ase`**; custo em sprites grandes por medir; **LibreSprite ainda não é fixture com gate**.
9. Rampas/Push: rampa de 1 degrau não anda; Push em camada não Indexed mistura o alfa (desbota); o script do oráculo não grava `pngquant.de_medio`.
10. Som: caminho de asset relativo ao projeto não resolve (`publica(.., None)`).
11. UI: tooltips de **Dither / Push / Keep ramps** sem `set_tooltip`; **Push sem ícone próprio** (usa `Smear`).
12. Cenas: a 31 tem a forma de colisão verde sobrando; a 32 sem grade iso; custo GPU do passe com rampa só medido ponta a ponta.

**Da P14:** o ABERTO 2–11 da [continuação](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-09_P14.md) continua inteiro (destaque: smoke do dono do Export Godot e de largar `heroi.json`).

## 7. Smoke DEPOIS da integração (do dono)

1. `cd /home/enio/Documentos/Projetos/PH2D && cargo build --profile smoke -p ph2d-host-desktop` (depois da integração, no primário).
2. `PH2D_PIXEL_SMOKE=<n> ./target/smoke/ph2d-host-desktop`, `<n>` de 28 a 36, uma cena por vez:

- **28** Rampas: bola e céu em faixas com Shading; Palette com as 3 rampas (o painel abre rolado nelas); a água cicla no Play. Fotos [01](smoke_P13_cena28_2026-11-09.png) · [02, 0,2 s depois](smoke_P13_cena28_2026-11-09_02.png).
- **29** Foto em paleta: Exact RGB (esq.) × Keep ramps (dir.); 292/4096 px diferem. [foto](smoke_P13_cena29_2026-11-09.png)
- **30** Dither e Push: três faixas de dither (16/32/48 de 64) sobre gradiente; cartão «Push (R)». [foto](smoke_P13_cena30_2026-11-09.png)
- **31** Luz na paleta: bola no degrau 2 da rampa Skin com «Light: Palette», degraus concêntricos a partir da lâmpada. [foto](smoke_P13_cena31_2026-11-09.png)
- **32** Guias: casa em 2 pontos de fuga; linha guiada e bloco iso legíveis. [foto](smoke_P13_cena32_2026-11-09.png)
- **33** Lint: `vicios_p13` com a janela Lint, 5 avisos em 5 grupos. [foto](smoke_P13_cena33_2026-11-09.png)
- **34** CRT: herói em Draw, Preview esticada com a caixa CRT marcada e a imagem com scanlines. [foto](smoke_P13_cena34_2026-11-09.png)
- **35** Contorno vivo: bola com Outline amarelo na camada e ícone de efeito na linha. [foto](smoke_P13_cena35_2026-11-09.png)
- **36** Fala e tween: 5 quadros A→B com Ease In-Out e a linha «ph2d-p13-fala.wav» com a onda. [foto](smoke_P13_cena36_2026-11-09.png)

Deu errado se: a 28 não muda entre as duas fotos; a 29 mostra duas fotos iguais; a 31 acende fora da paleta; a 34 mostra a Preview vazia ou sem lugar para a imagem (defeito §6.7, conhecido); a 36 sem onda. Som (36): Play ouve a fala; arrastar a onda e o scrub soam.

## 8. Gate do fecho

1× sobre `f254e42bf1..HEAD`, as 56 crates tocadas. Curas em 3 commits, sem mudar lei:
- **fmt** — 2 ficheiros; deixou `view.rs` em 706/700 e `paint.rs` já estava em 604/600 ⇒ `checker.rs` (xadrez) e `paint_hex.rs` (`hex`), re-export igual (`684ffcdb74`).
- **clippy `-D warnings --all-targets`** — 22 avisos, verde depois (`63acadffea`): estilo, `compose_bands` morta e imports mortos. No mesmo commit, `panel_functions_under_loc_cap` (`apply_event` 204 → `seam_camadas::renomear`) e `no_magic_numeric` (`LITERAL-PX-OK` com a grandeza nas faixas das rampas e no tecto do Color limit).
- **`ph2d-panel-registry-init` INTEIRA** (`fila-pesada.sh`, 96 s): 5 vermelhos do Lint ⇒ `cb13437ca2`: `staleness` ×3 (`cargo run -p ph2d-panel-sync`, só a ORDEM pixel-lint/pixel-prefs), `every_tab_speaks_through_the_string_table` (chave da aba `pixel.lint` ⇒ **`panel.pixel_lint.title`** no painel, Window e tabela), `lei_da_compactacao` lei 1 (as seis regras por `caixas_aos_pares`). Re-corrida: **483 ok · 0 falhas · 38 ignorados**.
- **Por nome, verdes:** os 5 tectos pedidos + `architecture_panel_loc_cap`, `no_magic_numeric_in_widget_or_screens`, `architecture_panel_wiring_parity`; shell `file_loc_caps` e `every_panel_the_shell_drives_is_in_its_registry`; `ph2d-panel-pixel-lint` inteira; os testes editados.
- `check-standalone-optional` ✓ · `check-workflow-packages` ✓ · `cargo machete`: zero. Contrato §6: 0 definições/`impl` (§3).
- Impactados e suíte completa ficam para a integração.
- `agent-loop-profile.sh` (20 sessões): paralelismo 1,12/passo ✗ (alvo ≥ 1,5) · respostas/sessão 134 ✓ · test:check 2,3× ✗ (alvo ≤ 1) · edições sem conferir 0,4 % ✓ · contexto/passo 232 mil ✓ · início 67 mil ✓.

## 9. Build final (item 9)

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` 2× sobre `5135e590ee` (o código final); a 2.ª:

```
▸ linha line_pixelart · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 600s
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

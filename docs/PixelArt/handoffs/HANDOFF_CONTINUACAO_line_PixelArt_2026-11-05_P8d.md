# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P8d (smoke da cena 19) — 2026-11-05

> Leitor: o próximo agente da `line/PixelArt`. A linha CONTINUA; nada desta onda está no `main`.
> Smoke da cena 19 aprovado pelo dono em 05/11, antes e depois dos ajustes abaixo.

## 1. Identidade

- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `c61f228b05` (reabertura de 05/11: os 87 commits
  antigos já estavam no `main`; `git reset --keep main`, HEAD antigo `3f01bdd52d`).
- Commits: `4f4a72456a` (título repetido no cartão) · `c4bb54a58e` (a onda) · `b928e4b365` (clippy) · este handoff.
- Binário do smoke compilado (`--profile smoke`, sem features). 2.ª corrida: `Finished in 0.26s`.

## 2. O que a onda fez

| Ordem do dono | Código | Prova |
|---|---|---|
| Título do cartão não se repete (achado na foto) | `titulo` em vez de `linha` em Gradient, Text, Paint Bucket | `a_tool_card_does_not_repeat_the_name_its_header_already_says` (vermelho antes) |
| Filtros em tempo real, Apply e Cancel, sem Preview | sai `PIXEL_FILTER_PREVIEW` e o campo `preview`; `pixel_filter_preview` = diálogo aberto; `filter_number` lê o buffer a cada tecla; `is_filter()` ⇒ «Apply» | `the_filter_dialogs_choices_reach_the_preview_and_the_ok`, `seam` de `filtros` |
| Girar de volta devolve o original | `Affine::settle_turn` (grau inteiro; ímã `TURN_MAGNET_DEG` = 2,5° nos ângulos retos), chamado em `solta::moved` e em `viva` | `giro_que_volta` (4), vermelho sem `settle_turn` |
| Só 12 fontes, com prévia melhor | `BUILTIN` = 12 com em nativo; `fabrica_contorno`; `font_list()`/`pick(i, px)`; prévia recortada à tinta | `fabrica` (5), `texto_fontes` (5) |

**Mecanismo do giro:** ângulo contínuo com barra arredondada: voltar a 0,4° lia «0» e reamostrava a imagem inteira. `Affine::drag`
NÃO mudou, para manter `affine_parity` (MVP): quem assenta é o app. Escala e inclinação já andavam em passos inteiros.

**Mecanismo das fontes:** cada contorno é rasterizado UMA vez no em nativo, sem hinting, sem Antialias e **sem `box_bottom_cut`** (a
caixa do Aseprite comia a perna do «g» na Micro 5; prova: `the_tail_of_a_descender_is_never_cut` vermelho com o corte), e ampliado
por inteiro. Em nativo, medições e recusas: `crates/ph2d-pixel-text/fonts/README.md`.

**Removidos por consequência:** fontes do sistema (`PIXEL_POPULAR`, `pixel_popular_count`, `SYSTEM_MIN_PX`, `face_bytes`,
`tests/it/minimo_legivel.rs`, dependência `ph2d-system-fonts` do app) e Bold/Italic/Antialias/Hinting (ids, opções, i18n, painel), pois
nenhuma das 12 responde a eles. Saiu o teste `the_floating_text_blends_with_the_background_at_its_new_place_when_it_settles`:
só havia cobertura parcial com fonte do sistema. ⚠️ `carimbo::over`/`tinge` parcial e `texto::mistura` seguem no código sem produtor
no app; `FontChoice::System` e `rasterize_outline` seguem para os oráculos (`trial_text`, `freetype_oracle`). Decida ao fechar a linha
se a mistura parcial fica.

## 3. Gates desta onda

- `cargo fmt --all` e clippy `-D warnings` das 8 crates tocadas (`PH2D_SUITE=onda`): verdes.
- Portões por nome, verdes: `the_shell_only_shrinks`, `no_magic_numeric`, `no_literal_color`, `architecture_panel_wiring_parity`,
  `node_id_collisions`, `architecture_workspace_file_loc_cap`, `panel_functions_under_loc_cap`. Corridos ANTES do commit de clippy, que não
  muda contagem relevante.
- Suíte completa: não corrida (é da integração). `ph2d-i18n` por nome não casou teste; os censos de chave órfã ficam para o fecho.
- Tecto de `ph2d-app-pixel`: a folga era 28 linhas (13 396 / 13 424) antes da onda. **Meça antes da P9.**

## 4. Smoke (aprovado)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=19 ./target/smoke/ph2d-host-desktop`
Fotos: `docs/Components/ferramentas/fotografa_cena.sh "PH2D_PIXEL_SMOKE=19 PH2D_PIXEL_SMOKE_GESTO=<balde|convolucao|hue|texto|fonte|solto>" 1930 984 <saida.png> 5`.

## 5. O próximo item

**P9 — Export / Import** (`02_plano_modulo.md`, `01_estudo_aseprite.md` §5). Comece pelo plano (`docs/PixelArt/P9_plano.md`); o
dono decide o que for de produto.

Levantado em 05/11 (confira antes de construir):
- Ausente no Pixel: Export As, Export Sprite Sheet, Import Sprite Sheet, GIF, APNG, WebP, PNG/sequência, escrita `.ase`, Repeat Last Export.
  `ph2d-pixel-io` só tem `frame_rgba`, `.msk`, paletas e pincéis. O Pixel não tem menu File próprio.
- Reutilizável: `ph2d-imageio-{gif,apng,webp}`; `ph2d-sprite-sheet::to_aseprite_json` (`aseprite.rs:34`), leitor `ph2d-asset::parse_atlas_meta`;
  `ph2d-aseprite::parse` (só quadros RGBA8 e tags, usado em `shells/desktop/src/ase_import.rs`). A API com camadas e a escrita não existem.
- Lock: `gif` 0.14.2, `png` 0.18.1, `image` 0.25.10, `miniz_oxide`, `flate2`. Licenças não conferidas.
- Costura de diálogo: `selecao.rs` atalho → `PixelEditAction` → `filtros::menu` → `paint_pixel_edit_dialog` (já em cartões).
- **Oráculo:** a licença do Aseprite foi recusada pelo dono (25/10), e o Trial não grava. Use o **LibreSprite 1.3** (`/usr/bin/libresprite`,
  GPL-2, corrido, nunca lido; ver `ferramentas/oraculo_msk.md`), Godot e ImageMagick. Logo, o gate «`.ase` aberto no Aseprite» do plano
  não se cumpre: diga isso no plano. O plano cita fixturas do MVP W13: não as achei na worktree; `docs/Pixel Art/` só existe no primário.

Pendências pequenas para resolver junto (P8):
- Texto mono em fontes do sistema contra a FreeType (1–2 células): caiu de importância sem fontes do sistema na lista; registre e feche.
- `edges-find*`: alfa nunca medido. · AA3 sem fixture (idem: sem Antialias no painel, reavalie).
- `curve_a11y`/`range_a11y`: públicas, mas nenhum painel publica o nó na árvore AccessKit.

## 6. Avisos

- `rustfmt` num `main.rs` de testes recorre aos módulos: confira `git status` depois.
- Em zsh, `$VAR` com vários caminhos NÃO se parte em palavras (o `git commit -- $P` falhou): use array ou `$=VAR`.

## 7. Dois defeitos do smoke, curados depois da §1–§6

- **Ajustes sem tempo real.** `filtros::acompanha` trocava a prévia sem mudar o carimbo do documento, e `PixelView::refresh_with` só recompõe quando `(stamp, frame, onion)` muda. Os testes passavam com a tela parada porque `shown_rgba` lê o traço direto. Cura: `app.toca` ao trocar e ao largar a prévia; `filter_number` só lê o buffer com foco e fora de arrasto. Gate: `each_change_of_an_open_filter_tells_the_canvas_to_redraw` (vermelho antes). ⛔ Prova de efeito no canvas tem de olhar o carimbo, não só `shown_rgba`.
- **Painel errado com Pixel Sprite** (Bones ou Physics na frente, reproduzido com cópia do `layout.txt` do dono). `layout_switch::arrange` fecha tudo e abre a lista da área, inclusive o painel Pixel, e o `panel_z_order` só se povoa depois do 1.º paint, por cima de um `bump` do 1.º quadro. Cura: `panel::painel_do_sujeito`, no flanco de `(entidade, em Draw, área de trabalho)`, com a ordem já povoada: Inspector em Object, Pixel em Draw, Bones fora enquanto durar. Gate: `a_pixel_sprite_brings_its_own_panel_and_moves_the_bones_panel_away`. Foto com a arrumação do dono conferida no arranque em Draw. Não coberto por foto: selecionar pela Hierarchy em Object (a tela virtual não recebe clique).
- **Tecto:** os 6 testes de `ph2d-app-pixel/tests/it` foram para `shells/pixel-scenes/tests/it` (`ajustes.rs` → `ajustes_app.rs`); `ph2d-app-pixel` 13 4xx → 11 192, tecto 11 415. A P9 tem folga.
- Com a crate `ph2d-host-pixel-scenes` inteira (motivo: código movido): 240 verdes.

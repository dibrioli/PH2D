# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, P10c (G da grade, balde em P, cena 22 das ilhas) — 2026-11-07

> Leitor: a próxima janela da linha, que abre a **P11 — Slices**. A linha continua; não integra nem faz push (o dono não ordenou).
> Anterior: [P10b](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-06_P10b.md). Integração da P10: [HANDOFF_INTEGRACAO_line_PixelArt_2026-11-06_P10.md](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-06_P10.md). Plano: [P10_plano.md](../P10_plano.md).

## Estado

- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`. A P10 inteira já está no `main` (integrador, 07/11; `5c449c3e07` é a cura de portões dele). A linha foi reaberta do `main` em `652b506b58`, com 1 commit à frente: `79989e9e0b`. Vem junto com este handoff uma nota em `docs/PixelArt/02_plano_modulo.md` (P12), por comitar.
- **Smoke do dono: cena 22, os 8 passos, «smoke OK» em 07/11.** Isso fecha o smoke da P10c, que o §9 do handoff de integração dava como aberto.
- Nada de push. Próximo: **P11**.

## O que mudou nesta onda (a pedido do dono, antes do smoke)

1. **`G` chegava ao balde e nunca à grade do canvas.** Dentro do Pixel, `ramo_teclas_editor_vista_e_transporte` (`shells/desktop/src/input_dispatch/handlers_teclas_editor.rs`) cede o teclado depois de `pixel_owns_the_keyboard`. Agora **balde = `P`, Gradient = `Shift+P`** (`crates/ph2d-app-pixel/src/input_teclas.rs`, `pixel_panel/tools.rs::shortcut`); `G` e `Shift+G` passam para a grade. Escolha do `P`: livre sem modificador no shell e no Pixel (o `K` é a chave da timeline no shell). A exceção ao idioma do Aseprite está anotada em `02_plano_modulo.md` P12.
2. **Cena 22 redesenhada.** `exemplo::terreno_ilhas` (47 formas por regra: distância à água `ate_a_agua`, cantos de fora redondos, de dentro em arco; faixas espuma, areia, relva escura, relva) e `exemplo::agua` na camada de baixo; mapa `ILHAS` 16×10 (256×160). `MAPA_REAL`/`celulas_reais` ficam para a aceitação. `novo_terreno_com_guia` continua o guia do botão New.
   - Porquê 16×10: `FIT_FRACTION` 0,6 em `frame.rs` fazia 320×192 cair a 100 % numa janela de 1930×1040 (640 > 639); com 256×160 abre a 200 %.
   - Porquê sem tileset de artista: os da pasta Downloads do dono não têm autoria nem licença conhecidas.

## Provas

- Testes novos: `g_is_left_to_the_canvas_grid_and_never_picks_a_tool` (`shells/pixel-scenes`); `the_island_art_of_scene_22_matches_across_every_seam` (`ph2d-pixel-tiles`, `p10c.rs`).
- Mutação manual: trocar o arco do canto de dentro em `ate_a_agua` deixou o teste das emendas vermelho; restaurado, verde.
- Fecho: `cargo fmt --all` sem mudanças; clippy `-D warnings` verde em `ph2d-pixel-tiles`, `ph2d-app-pixel`, `ph2d-editor-core`, `ph2d-panel-pixel-tools`, `ph2d-host-pixel-scenes`; `tiles::` 26 verdes; por nome verdes: `g_is_left`, `every_aseprite_shortcut`, `shift_p_picks`, `a_cena_22`, `the_island_art`, `o_balde_e_o_blur`, `the_shell_only_shrinks`.
- ⚠️ **Não correu:** suíte completa (é do integrador).

## Reparo sem conserto

- Em Iso a arte quadrada fica sobreposta (a arte não muda, só o lugar). O dono aprovou assim no smoke; não reabrir.

## Abertos

- Os demais itens do §9 do [handoff de integração](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-06_P10.md): remeta para lá, não copie.

## Abrir a P11 por aqui

**P11 — Slices** (`02_plano_modulo.md` §5, linha 184): Slice tool, 9-slice e pivot arrastáveis, keys por frame, export em `meta.slices` / `--split-slices`.

- ⚠️ O plano diz «Smoke 12», mas está velho: a próxima cena do Pixel é **24** (`max_level: 23` em `crates/ph2d-app-pixel/src/smoke.rs`).
- A P11 **não tem plano próprio**. Comece por escrever `docs/PixelArt/P11_plano.md`: meça o Aseprite Trial e os arquivos `.ase` (oráculo que se CORRE, CLAUDE.md §0.9) e mostre as decisões ao dono no formato dos planos anteriores (P9, P10).
- A P9 deixou para a P11 o `--split-slices` e o slice vazio (`P9_plano.md`, linha 23).

## Regras que continuam

UI só em inglês, cartões compactos; zero hex e zero literal de UI; teste por nome, suíte só na integração; `ph2d-app-pixel` está no tecto de LOC: **mover código para crate, nunca subir o número**; integrar e push só por ordem do dono.

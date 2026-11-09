# Continuação — line/GameEngineFuncional, 09/11 → Align COMPLETO (ondas C e D) + meio texel + filtro; próximo: FECHAR a linha

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` · branch `line/GameEngineFuncional`
· base `main` `f254e42bf1`. ⛔ Não tocar na `line-GameEngineFuncional02`. Nada integrado, nenhum push.
Antecessor: [`HANDOFF_CONTINUACAO_2026-11-09_ALIGN_ONDA_B.md`](HANDOFF_CONTINUACAO_2026-11-09_ALIGN_ONDA_B.md) · plano:
[`28_alinhar_folhas_estudo.md`](../28_alinhar_folhas_estudo.md). Todos os smokes desta janela: **OK do dono**.

## Commits (sobre `f373a7e417`)

| Commit | O quê |
|---|---|
| `64bb60daa9` | **meio texel do átlas**: `AtlasRegion::uv_exact` / `TextureAtlas::region_uv_exact`; o extract (`sim_extract_emit.rs`, os dois caminhos do átlas) corta região e célula do rect EXACTO; o meio texel (`filter_clip`, ou átlas que repete — `sprite_wraps`) move só a célula FINAL (`inset_half_texel`, `ph2d_render::sprite`). Gate puro `every_atlas_sheet_frame_lands_on_its_own_texels` (com controlo); `sprite_sem_risco_gpu` corta do rect exacto: 6/6 na placa. Foto `PH2D_GAME_SMOKE_FOTO=animado_idle_q<k>` |
| `1f6ea871b3` · `a293b33543` | **onda C**: «By feet» / «By centre» (`AnimDoc::align_frames`, `align_point`, `AlignBy`): cada quadro da animação aberta vai, pelo acerto da célula, ao ponto da REFERÊNCIA (pés = meio da base da caixa opaca, na linha do chão se arrastada; centro = meio da caixa); px inteiros da folha do quadro; uma edição = um Ctrl+Z. `ImageUseEdit::NudgeCell` passou a levar `sheet`. Moldura do Align em 3 colunas. Gate `by_feet_e_by_centre_levam_cada_quadro_ao_ponto_da_referencia`. Fotos `animado_align_pes(_desfaz)` (3 acertos, 1 Ctrl+Z → 0) |
| `0a4312e195` | **filtro do app nos painéis** (report do dono): `ph2d_editor_core::{set_ui_image_filter, ui_image_quality}` (atómico; o shell escreve a cada quadro em `fase_extract_inputs.rs`). Leem: folha/tira/prévia Align do editor, `sheet_cell`, cartão Image e folhas do Inspector, miniaturas dos Assets, retrato do jogo, Region reduzido. A prévia Align deixou de fixar `Low`. Gate censo `architecture_a_arte_do_jogo_segue_o_filtro` (mutado ⇒ vermelho). `PH2D_GAME_SMOKE_FILTRO=pixel` |
| `76eb8e3559` | **onda D**: cena 5 ganha «Misaligned Sheets» (`DESALINHADAS`, em `[-6, 4,2]`): run 1,5×, deslocada `DESLOCA_A_RUN`, `TORTOS` (células 1 e 2). Fotos `animado_desalinhadas(_acertadas)`. Gate `misaligned_sheets_nasce_torta` |
| `ca6ac2b354` · `d9629a113b` · `829275ede1` | a marca na tira: por fim a palavra «aligned» (Xs, Info) logo após o número (`paint_align::paint_nudge_mark`). `d9629a113b` também põe no **CLAUDE.md §0.8** a regra do dono: imagem ao dono = link relativo à RAIZ do repo principal (`Worktrees/line-<x>/…`) |

Fotos: `docs/GameEngineFuncional/fotos_28/`.

## Decisões do dono nesta janela (09/11)

- Pixel nítido NÃO é padrão; o filtro é o das Preferences, e agora vale também nos painéis e no editor.
- Marca do quadro acertado = palavra discreta ao lado do número. Ela marca o quadro com acerto PRÓPRIO ≠ 0 (um
  torto que o By feet devolve ao lugar da folha perde a marca) — explicado ao dono, smoke OK.
- By feet / By centre alinham à REFERÊNCIA (não ao Pivot): a referência e o objeto no jogo ficam.

## Para o FECHO (próxima janela, `/pd-linha-fechar`)

- Shell cresceu (~+15 linhas nesta janela: `sim_extract_emit.rs`, `janela_do_jogo_animado.rs`, `fase_extract_inputs.rs`);
  `the_shell_only_shrinks` verde; o integrador reconta.
- Crates tocadas (fmt/clippy 1×, provas de mutação): `ph2d-render`, `ph2d-anim-editor`, `ph2d-anim-sheets`,
  `ph2d-editor-core`, `ph2d-i18n`, `ph2d-panel-inspector`, `ph2d-panel-asset-browser`, `ph2d-app-game`,
  `ph2d-app-sprite-anim`, `ph2d-sprite-anim-scene`, `ph2d-host-desktop`, `ph2d-host-game-scenes`.
- ⚠️ Render partilhado mudou (o rect do átlas): leis de mutação candidatas — o `uv_exact` (voltar ao `uv` ⇒ o gate puro
  e o controlo), o inset na célula final, o `sprite_wraps`. Varrido por consumidor: `region_uv` continua com o
  recuo para o Motion e os testes de placa que o usam direto (sem grelha, sem deslize).
- Memória do dono: o índice `project-memory/MEMORY.md` (no main) já tem a regra dos links atualizada, por commitar
  pelo integrador.

## ABERTO (herdado)

- Após Ctrl+Z, o `ShownSheet` (não registado no undo) volta a 0: o editor e o cartão Image mostram «idle». Antigo.
- `ph2d-app-macros` `tutorial_painter::outra_tela_diz_o_tamanho_gravado` (stack overflow) — alheio, por conferir no main.
- «Delete empty animations» não apaga animações de 0 quadros (deliberado); o «+» do cartão Animation não seleciona a nova.
- A faixa rolada: a ranhura «INSERT HERE» cobre a ponta direita do último quadro (visto nas fotos).

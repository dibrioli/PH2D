# HANDOFF de continuação — GEF 19 ordem e clique, 4.ª janela (2026-11-01)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · base main `422fec2027`
(não andou) · HEAD `9a88830e50`. Anterior: [3.ª janela](HANDOFF_CONTINUACAO_2026-11-01_19_ORDEM_E_CLIQUE_3.md).

## Feito nesta janela (commit `9a88830e50`) — NÃO reconstruir

- **A (cópias de sprite do Motion sem rank):** as cópias iam no `extra`/`gpu_extra` do passe principal.
  Agora, com ordem intercalada (`faixa::sprites_por_faixa` = activo ∧ `needs_banding` ∧ há Motion), colam na
  faixa do dono: `faixa::cola_os_sprites` corta por `faixa::leva` (topo leva o complemento, as outras só as
  delas) e `corta_runs`; desenha `game_rt.clear` → `Renderer::render_instances_da_cena` (com luz) →
  `tonemap.run` → `band_blit` Sprites. `present.rs` desliga `sprites_da_camada` quando `plan.motion_por_faixa`.
- **A (piscar):** a Pile esvazia ~0,6 s a cada 3,7 s; a placa inactiva punha `motion_na_placa=false` e o
  Motion inteiro subia ao topo. Cura: `motion_shape_placa::fica_no_rank(placa, motion)`.
- **B (Pile ausente):** defeito real da E3 (`cc02d5e412`): `cola_a_faixa` guardava em `placa.ativa`, que o
  `desenha` consome por `mem::take` — as faixas Motion abaixo do topo nunca colavam. Guarda = `desenhou()`.
  E a Pile estava fora da vista: `ordem_smoke.rs` `Peca::Pilha => ([1.5, 3.5], [1.0, 1.0])`.
- **C (modo errado ao abrir):** `wants_edit` do Pixel e `born` do Flip sobrepunham Object. Cura:
  `ph2d_app_pixel::smoke::rosto_em_object` + `flip_state.born.take()` em `janela_do_jogo.rs`.
- Fotos refeitas: [`fotos_19/ordem_smoke_1.png`](../fotos_19/ordem_smoke_1.png) (Card em baixo, Pile sobre o
  Card e sob o Ink, Swarm sobre o Face) e [`_2`](../fotos_19/ordem_smoke_2.png) (Card cobre tudo); Object Mode.
- Gates verdes por nome: `os_runs_de_textura_cortam_se_pelos_donos`,
  `o_topo_leva_o_resto_e_as_outras_so_as_delas`, `as_copias_de_sprite_vao_por_faixa_so_com_a_ordem_intercalada`,
  `sem_formas_o_motion_fica_no_rank` + os 5 de `present_placa_tests` + os `ordem` do `ph2d-app-game`.

## Aberto, por ordem

1. ⛔ **Tecto do `ph2d-app-game`:** `architecture_the_shell_only_shrinks` reprova 3 452 > 3 202
   (`TETOS_DAS_CRATES_APP`; main 3 102, a linha +348: miniatura, moldura, cena da ordem). Cura = MOVER
   (ex.: `ordem_smoke.rs` para crate/ficheiro de smoke fora do censo, se o censo o permitir) ou cortar —
   nunca subir o número. Meça o censo antes de escolher.
2. **Item 5 + realce, UMA rodada intercalada** (MEDIR_VELOCIDADE.md): perf do `LayerCompositor` do Flip com
   faixas intercaladas (`PH2D_FLIP_STATS=1`) e o custo da leitura de placa do realce do ponteiro, ambos em
   `PH2D_ORDEM_SMOKE=1`, `--profile smoke`.
3. **Mutações em lote** (agente `mutacao`): as da lista da 3.ª janela; `motion_pick`, `donos_das_imagens`,
   `corta`, `max: [17, 9]`, `pick_target`, `lugar` sem `nivel == 2`; novas: `fica_no_rank` só com `placa`;
   `sprites_por_faixa` sem `needs_banding`; `cola_a_faixa` com `ativa` em vez de `desenhou()`; `leva` com o
   topo trocado; `cola_os_sprites` não chamado.
4. **Testes de placa por nome** (`PH2D_GPU=1 … --profile smoke -- --ignored`): os da 3.ª janela + gates NOVOS:
   `motion_pick::do_dispositivo`; a faixa Motion de baixo cola depois do `desenha` (a mutação `ativa` tem de
   sangrar aqui); cópias de sprite por faixa.
5. Actualizar a tabela de estado de [`19_ordem_e_clique_plano.md`](../19_ordem_e_clique_plano.md).
6. Smoke do dono em passos numerados (cena `PH2D_ORDEM_SMOKE=1|2`); handoff de onda só depois dele.
   Fechar/integrar só por ordem do dono.

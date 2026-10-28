# HANDOFF de continuação — line/UIUX, onda F37 (o editor cabe numa placa com ~950 MiB livres: átlas sob demanda, UM Vello por placa, blocos pequenos), 27/10

> Linha NÃO integrada, sem push. Base: `main` c3662cd8e4 (não andou). Ponta: `git log -1`; a F37 são os
> commits de `git log --oneline 7b7ebf0ecd..HEAD`. **⚠️ Smoke do dono 27/10: PENDENTE — o dono adiou-o («deixe esse
> smoke para o futuro pois tenho agentes trabalhando») e mandou seguir. A F37 NÃO está aprovada até ele (§7).**
> Formato inalterado (`PROJECT_SCHEMA` 197, `project_schema.rs:515`; o diff não toca `shells/`). A shell não cresceu.
> Mecanismo, tabelas, mutações e fotos: [`F37_plano.md`](../F37_plano.md) — leia §0 (a tabela), §1 (átlas), §2 (Vello).
> Antecessor: [`HANDOFF_CONTINUACAO_line_UIUX_2026-10-27_F36.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-27_F36.md)
> (a F37 fecha o seu §8 item 1).

## 1 — Commits da F37

| commit | o quê |
|---|---|
| `e4835889d1` | o átlas nasce em 2048² e cresce no lugar (`TextureAtlas::cresce`); a UV guardada do ladrilho branco segue branca; G22–G24 |
| `e8586d7612` | UM `vello::Renderer` por placa, partilhado por todos os `VelloPass` (`vello_partilhado.rs`); G25–G26 |
| `5f8ca30103` | o alocador da placa reserva blocos de 8–64 MiB, não de 128–256 (`MemoryHints::MemoryUsage`); G27 |
| `91ab545c98` | plano F37: a pegada medida, o caso do dono reproduzido, mutações, fotos (`img_F37/`) |
| `2ab58d0011` | fecho de onda: dois gates da suíte herdados da F36 (a vez da placa no `ocupa_a_placa`; COLOR-RAW-OK dos bytes de upload guardados) |
| `9507f951ea` | fecho de onda: três ficheiros da F35 abaixo do teto de 700 linhas, por corte mecânico |

Delta medido (`git diff --shortstat 7b7ebf0ecd..HEAD`): **32 ficheiros, +1 235 −548**.

## 2 — O que mudou e o número (plano §0)

`nvidia-smi` por processo, RTX 5060 Ti, MiB: cena 1 **831 → 377**; cena 41 (Nuvem com FX) **1 343 → 507**.
Caso do dono (`ocupa_a_placa 950` → 1 029 MiB livres): cena 41 F36 = 283/287 recusas, a cena não monta (`Path 0` em
vez de `Bevel`); F37 = 0/0 e a cena inteira (fotos em `img_F37/`; cena 41 antes × depois = 0 px de diferença).

- **Átlas** (`atlas/cresce.rs`): nasce 2048² (~21 MiB), dobra até `ATLAS_MAX_SIZE_PX` 8192 copiando o nível 0 e
  alargando o `DensePacker`; toda região fica no MESMO pixel. O `regrow_inplace` antigo (re-empacotar por chaves)
  SAIU. O ladrilho branco nasce na ORIGEM de todo átlas e a sua UV é a constante `WHITE_TILE_UV = [0,0,1/256,1/256]`,
  válida para todo tamanho ≤ 8192 ⇒ Motion `default_uv_rect`, partículas, barras de vida e trilha de macro (que a
  guardam) ficam certos SEM mudar uma linha (a pré-condição do F36 §4 resolveu-se assim, não por leitura por quadro).
- **Vello** (`ph2d-render/src/vello_partilhado.rs`): `da_placa` devolve o renderer vivo do dispositivo (registo de
  `Weak`). Chave = IDENTIDADE do `Arc<wgpu::Device>` (o `PartialEq` do `wgpu::Device` deu por iguais dois
  dispositivos distintos). Partilham-se renderer, `registos`, episódio de falta e `refeitos`; ficam por pass o
  intermédio, blitter, keepalive, fundo, `falta_dita` e os ids registados. Nenhum dono de outra família mudou.
- **Blocos** (`ph2d-gpu/src/context.rs`): `MemoryUsage` por omissão; `PH2D_GPU_BLOCOS=desempenho` volta aos grandes.
  Preço medido (cpu-encode, mediana): cena 41 5,85/5,95 ms contra 6,08/8,31; vsync 16,7 ms nos dois — sem diferença.
- Gates novos G22–G27 (nomes no plano §1–§3); mutações M1–M12 sangraram todas (plano §4).

## 3 — Superfície de colisão (DELTA) e o que um merge pode partir

- Alterados: `ph2d-render` (`atlas/`, `renderer*.rs`, `vello_partilhado*.rs`, os `VelloPass`), `ph2d-gpu`
  (`context.rs`), `ph2d-asset-ktx2` (toque do fecho), `examples/ocupa_a_placa`; o fecho cortou `fx_live.rs` →
  `fx_live_textura.rs`, `fx_live_zoom_gpu_tests.rs` → `…_apoio_tests.rs`, `motion_fx.rs` → `motion_fx_passe.rs`.
- Um merge parte: (a) outro átlas criado sem `TextureAtlas::with_filter` — o branco na origem é invariante de
  nascença; (b) quem reintroduzir re-empacotamento do átlas — `WHITE_TILE_UV` deixa de valer; (c) um
  `vello::Renderer` criado fora de `vello_partilhado::da_placa` (volta ~166 MiB por dono); (d) um `GpuContext`
  montado com `Arc<Device>` novo para o mesmo dispositivo ganha renderer próprio — não parte, só não partilha;
  (e) `insert_atlas_sprite_with_regrow` mantém o fetcher MORTO na assinatura (6 chamadores de 4 famílias, §8).
- Contador de cena: `PH2D_OBJECT_MODE_SMOKE` segue até **41** (a F37 não criou cena). `PROJECT_SCHEMA` 197.

## 4 — Prova de fecho

- `cargo fmt` e clippy `-p ph2d-render -p ph2d-gpu -p ph2d-asset-ktx2 --all-targets`: limpos. Os 51 testes de placa
  da onda e o G27: verdes.
- Suíte das crates afetadas (`PH2D_SUITE=onda`, nextest-impacted): **3 vermelhos, TODOS herdados da F35/F36 desta
  linha (nenhum da F37)**, curados: `todo_pedido_de_adaptador_passa_pela_vez` (o exemplo sem
  `ph2d_gpu::vez_da_placa()`) · `no_bare_byte_color_in_ui_or_raster_crates` (`individual_renasce.rs`,
  `anota_pixels_recusados`: COLOR-RAW-OK na linha do parâmetro) · `workspace_src_files_under_loc_cap`
  (`fx_live.rs` 731→659, `fx_live_zoom_gpu_tests.rs` 727→600, `motion_fx.rs` 702→642; 1 768 nomes de teste iguais
  antes/depois; testes de placa fx_live 47 + motion_fx 27 verdes).
- ⚠️ A suíte NÃO foi re-corrida inteira depois das curas — só por nome. Fica para o integrador.
- Binário do smoke compilado (2.ª build `smoke`: «Finished … in 0.37s», zero Compiling); `ocupa_a_placa` compilado.
- Comandos CPU: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && bash scripts/cargo-test-narrow.sh ph2d-gpu the_card_allocator_reserves_small_blocks` (G27);
  `… ph2d-render a_cached_white_tile_uv` (G22) · `… ph2d-render growing_keeps_every_region` (G23). G24–G26 são de
  placa: `PH2D_GPU=1 bash scripts/ph2d-run.sh` (confira `#[ignore]` com `git grep`).

## 5 — Fotos e logs

`img_F37/`: `cena41_1GB_livre_antes.png` · `cena41_1GB_livre_depois.png` · `cena1_1GB_livre_depois.png` · `rodada_antes_depois.log` ·
`rodada_blocos.log` · `rodada_final.log` · `rodada_final.sh` · `rodada_pegada.sh`.
(Untracked alheios não comitados: `img_F30/cena35.log`, `img_F33/cena40.log`, `img_F34/cena41.log`.)

## 6 — Truques de medição

Pré-construir ANTES de encher a placa; rodada dentro de `PH2D_GPU=1 bash scripts/ph2d-run.sh`; `ocupa_a_placa <alvo>
<s>` enche em blocos de 256 MiB (alvo 950 → 1 029 MiB livres); `PH2D_GPU_PEGADA=<s>` imprime o alocado/reservado por rótulo;
`PH2D_GPU_ERROS_TODOS=1` conta as recusas.

## 7 — Smoke do dono — PENDENTE

1. Terminal 1: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/examples/ocupa_a_placa 950 120` — espere «seguro 120 s».
2. Terminal 2: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=41 ./target/smoke/ph2d-host-desktop`.
3. Certo: abre logo, a grande forma verde, «Bevel» na Hierarchy.
4. Errado: «The graphics card ran out of memory…», «Path 0» em vez de «Bevel», ou forma ausente.
   (Opcional: `PH2D_OBJECT_MODE_SMOKE=1` → o quadrado branco «Canvas».)

## 8 — ABERTO, por ordem

0. **Smoke da F37 pendente** (§7) — sem ele a onda não está aprovada.
1. **Recomendado como próximo: medir a F37 na Radeon onboard (item 8)** — é a placa de poucos MiB livres, o alvo real
   da onda; `ocupa_a_placa` + `PH2D_GPU_PEGADA` já são o roteiro (§6). Os itens 2–7 são herdados e independentes.
2. Views do mipgen: não auditadas (F35 §7.1 resto; o censo da F36 não as viu recusadas).
3. F33 §6.2 — seleção pela cena 40 não acende o painel (`vec.pen.select_many` é a porta certa? ainda é chamado
   por `autokey_smoke*.rs`; a cena 40 não foi reconferida).
4. F33 §6.3 — Zig Zag / Sketch no vale côncavo: decisão de desenho do dono, com o oráculo a correr.
5. F32 §6.2 — pose dos estados de UI alinha o MEIO da transição por ÍNDICE (reordenar faz um degrau trocar de tipo).
6. F32 §6.3 — chave sobre `from`/`to` de uma seta guarda o número do estado; apagar um estado anterior não a reescreve.
7. F32 §6.4 — F31 §6.5 (losango da pilha só da forma mostrada) e F30 §6.5–6.9 (`View::Toggle`, faixas sem losango, scrub
   sem reler a curva, mundo/mesa fora do undo, cor crua nos overlays vetoriais).
8. Novo F37 — placa onboard (Radeon) não medida nesta onda.
9. Novo F37 — blocos pequenos não cronometrados numa cena de pintura em tela 4K (cria/larga texturas grandes por
   quadro); se engasgar, `PH2D_GPU_BLOCOS=desempenho` é a prova A/B.
10. Novo F37 — `insert_atlas_sprite_with_regrow(…, fetch)`: o `fetch` está morto; chamadores em outras famílias
    (`ph2d-image-import` ×3, `app-components/code_smoke_props.rs`, `app-motion/glow_dirt_smoke.rs`,
    `shells/desktop`: `app_state_gfx`, `project_assets`, `inspector_strategy`, `slice_smoke`) — simplificar quando
    essas linhas estiverem no main.
11. O episódio OSCILA enquanto outro programa segura a placa (F36 §8.3): desenho, não defeito.
12. `CLAUDE.md` §5 (UIUX): o link «Último» troca só na integração; `PH2D_AUTOKEY_SMOKE` segue `1..33` na linha
    (recontar na integração, F32 §6.5).
13. **Já resolvidos pelo código/handoffs (não reabrir):** F34 §6.1 (texturas do FX só crescem) e §6.2 (VRAM esgotada
    ⇒ editor em branco) — curados na F35; F36 §8.1 (átlas + Vello) — esta onda; F33 §6.1 (zoom perto + filtro) — F34.
14. **Integrar e push só por ordem EXPLÍCITA do Enio**; a suíte completa é do integrador.

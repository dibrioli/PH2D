# F37 — O editor cabe numa placa com ~950 MiB livres

> Aberto pelo handoff F36 §4/§8 item 1 (o caso do dono: o `ollama` com 13,7 GB, **946 MiB livres**, e a
> cena 41 a pedir 1 343 MiB). Uma família só (CLAUDE.md §0.10): **os custos FIXOS na placa**. Medido em
> 27/10, RTX 5060 Ti 16 GB, `kwin --virtual` 1930×1040, binários `smoke`, `nvidia-smi` por processo, sem o
> `ollama`. Roteiros e logs: [`handoffs/img_F37/`](handoffs/img_F37/).

## 0 — A tabela (a meta era ≤ ~700 MiB na cena 41)

`nvidia-smi` por processo, aos 12 e aos 20 s (iguais), duas corridas intercaladas por variante
(`rodada_antes_depois.log`, `rodada_blocos.log`, `rodada_final.log`):

| cena | F36 (antes) | + átlas sob demanda + UM Vello | + blocos de 8–64 MiB (**F37**) |
|---|---|---|---|
| 1 (Canvas) | 831 | 488–490 | **377–379** |
| 41 (Nuvem com FX) | 1 343 | 744–746 | **507** |

Pelo alocador (`PH2D_GPU_PEGADA=12`), cena 41: F36 **alocado 1 022,7 · reservado 1 365,3** → átlas e Vello
curados **alocado 364,6 · reservado 768,0** (`fx_stack tex` 51,6 · `vello.*` ~166 de UM renderer ·
`vello intermediate` 29,3 em 3 · átlas 21,3 · individual 21,3 · RTs 45). Os ~400 MiB que faltavam até à
meta eram blocos RESERVADOS e vazios do `gpu-allocator` — daí o §3.

**O caso do dono, reproduzido** (`ocupa_a_placa 950` → 1 029 MiB livres, `PH2D_GPU_ERROS_TODOS=1`, 25 s):

| programa | cena 41 | recusas de memória | o que se vê |
|---|---|---|---|
| F36 | 830 MiB (tecto) | **283 / 287** | aviso «placa sem memória»; a cena não se monta (`Path 0` em vez de `Bevel`) — [`cena41_1GB_livre_antes.png`](handoffs/img_F37/cena41_1GB_livre_antes.png) |
| F37 | 507 MiB | **0 / 0** | a cena inteira — [`cena41_1GB_livre_depois.png`](handoffs/img_F37/cena41_1GB_livre_depois.png) |
| F37, cena 1 | 377 MiB | 0 | o Canvas branco — [`cena1_1GB_livre_depois.png`](handoffs/img_F37/cena1_1GB_livre_depois.png) |

Imagem: placa livre, cena 41 antes × depois = **0 px** diferentes (`magick compare -fuzz 2%`); na cena 1
a diferença (35 073 px) é a mesma entre duas corridas do MESMO binário antigo (variação da corrida).

## 1 — Átlas sob demanda (`e4835889d1`)

- Nasce em `ATLAS_DEFAULT_SIZE_PX` = 2048 (~21 MiB com mips) e dobra até `ATLAS_MAX_SIZE_PX` = 8192 por
  `TextureAtlas::cresce` (`crates/ph2d-render/src/atlas/cresce.rs`): copia o nível 0 para a origem de uma
  textura maior e alarga o `DensePacker` (`resize` — o plano F36 dizia que o `rect_packer` não crescia no
  lugar; cresce). Toda região fica no MESMO pixel; a free-list segue válida; nenhum fetcher.
- ⛔ O `regrow_inplace` antigo (re-empacotar pela ordem das chaves, bytes pelo fetcher) saiu: com um
  átlas que cresce de verdade ele LARGAVA os ladrilhos reservados (branco, disco), que nenhum fetcher
  conhece, e mudava as regiões de lugar.
- **A pré-condição (UV guardada), sem tocar nas outras famílias**: o ladrilho branco nasce na ORIGEM de
  todo átlas e a sua UV é a constante `WHITE_TILE_UV = [0, 0, 1/256, 1/256]` — dentro dos 64 px dele para
  todo lado ≤ 8192 (sampler clamp-to-edge, a borda `0` é branca). Os quatro que a guardam (Motion
  `default_uv_rect`, partículas `p.uv`, barras de vida `b.uv`, trilha de macro `inicial.rs`) ficam
  certos sem mudar uma linha. As sprites já liam a UV por quadro (`sim_extract_emit.rs`,
  `motion_bridge_objects_appearance.rs`).
- `insert_atlas_sprite`/`_with_regrow` crescem e refazem o grupo do material; o fetcher do segundo
  deixou de ser chamado (assinatura mantida: seis chamadores de quatro famílias — simplificar é para
  quando essas linhas estiverem no main). `insert_reserved_tiles` também cresce. Erro novo `SemMemoria`.
- Gates: **G22** `a_cached_white_tile_uv_still_samples_white_after_the_atlas_grows` · **G23**
  `growing_keeps_every_region_in_place_with_its_pixels_and_free_slots` (`atlas/cresce_tests.rs`) · **G24**
  `the_sprite_renderer_grows_its_atlas_and_rebinds_the_material` (`renderer_tests.rs`).

## 2 — UM `vello::Renderer` por placa (`e8586d7612`)

- `crates/ph2d-render/src/vello_partilhado.rs`: `da_placa` devolve o renderer vivo do dispositivo
  (registo de `Weak`; o último dono a largar larga-o). Dentro do `ph2d-render`: **nenhum dos donos das
  outras famílias (Motion, Luz 2D, Quadro, filtros) muda** — a parte que o handoff F36 julgava cruzar
  famílias não cruza.
- ⛔ A chave é a IDENTIDADE do `Arc<wgpu::Device>` do `GpuContext`: o `PartialEq` do `wgpu::Device` deu
  por iguais dois dispositivos distintos (G25 medido vermelho com ele; em paralelo, pânico `storage.rs:135`).
- Vão para o partilhado: o renderer, os `registos` (o refeito herda as texturas de TODOS os donos), o
  episódio de falta (`refazer_em`: uma recusa envenena o pool de todos) e `refeitos`. Ficam por pass: o
  intermédio, o blitter, o keepalive, o fundo, `falta_dita`, e os ids que ele registou (largados no `Drop`).
- Gates: **G25** `the_passes_of_one_card_share_one_renderer_and_each_draws_its_own_texture` · **G26**
  `a_refusal_seen_by_one_pass_rebuilds_the_shared_renderer_with_every_owners_textures`
  (`vello_partilhado_tests.rs`); o G15 da F35 lê `estado_do_renderer()`.

## 3 — Blocos do alocador de 8–64 MiB (`5f8ca30103`)

- `GpuContext` pedia `MemoryHints::Performance` (blocos de 128–256 MiB): na cena 41 curada ficavam 403 MiB
  reservados e vazios. `MemoryUsage` (8–64 MiB) por omissão; `PH2D_GPU_BLOCOS=desempenho` volta aos grandes.
- Preço medido (`PH2D_FLUID_PROFILE`, janelas de 120 quadros, intercalado, mediana do `cpu-encode`):
  cena 41 desempenho 6,08 / 8,31 ms · uso 5,85 / 5,95 ms; cena 1 4,92 / 5,30 · 4,88 / 5,23; o quadro em
  16,7 ms (vsync) nos dois. **Sem diferença medível.**
- Gate **G27** `the_card_allocator_reserves_small_blocks_unless_asked_for_speed` (`ph2d-gpu`, CPU).

## 4 — Mutações (as 12 sangraram)

| mutação | vermelho |
|---|---|
| M1 o átlas não reserva o branco ao nascer · M2 `insert_white_tile` devolve a UV do lado | G22 |
| M3 `cresce` não copia · M4 não alarga o packer | G22 G23 (M4 também G24) |
| M5 sem refazer o material · M6 `insert` sem crescer | G24 |
| M7 `da_placa` nunca reusa · M8 ignora a chave · M9 `Drop` vazio | G25 (M7 e M8 também G26; M8 G15) |
| M10 `refaz` sem os registos · M11 `refazer_em` nunca limpa | G26 G15 |
| M12 omissão em `Performance` | G27 |

## 5 — Aberto

1. Placa onboard (Radeon) não medida nesta onda.
2. Os blocos pequenos foram medidos nas cenas 1 e 41; uma cena que cria e larga texturas grandes por
   quadro (pintura em tela 4K) não foi cronometrada — se aparecer engasgo, `PH2D_GPU_BLOCOS=desempenho`
   é a prova A/B.
3. `insert_atlas_sprite_with_regrow(…, fetch)`: o `fetch` está morto; seis chamadores em outras famílias.

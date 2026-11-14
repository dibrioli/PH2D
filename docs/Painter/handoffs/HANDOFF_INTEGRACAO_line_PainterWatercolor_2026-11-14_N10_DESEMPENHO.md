# Handoff de integração — linha PainterWatercolor · 14/11 (rodada n.º 10): desempenho 4K em todas as famílias — depósito do dab fora do evento, Ctrl+Z e troca do Wet, memória 4K, água, camadas/placa, build e réguas

> Ordem do dono 14/11: atacar todo o plano de desempenho, **integrar e fazer ship no fim; smoke só depois**. Quem o dono nomear integra
> (a linha NÃO se integra sozinha). Este handoff SOMA-SE ao [N8_N9](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_N8_N9_DESEMPENHO.md)
> e aos anteriores que ele lista: a linha inteira integra-se de uma vez. Leitor: o integrador e a próxima LLM.
> Verdade dos números: [`N10_*.md`](../missao_4k/medicoes/) (toque_undo, memoria, agua_wet, camadas, cadencia, build, medicao,
> cerdas_mistura, carimbo, achados) · base [`N10_BASE_MATRIZ.md`](../missao_4k/medicoes/N10_BASE_MATRIZ.md) · matriz final
> [`N10_MATRIZ_FINAL.md`](../missao_4k/medicoes/N10_MATRIZ_FINAL.md) (de outro agente) · [BACKLOG](../missao_4k/BACKLOG.md) (dobrado nos ids tocados) ·
> [PLANO §Estado](../missao_4k/PLANO_DESEMPENHO_TOTAL_2026-11-12.md) · [regras](../missao_4k/REGRAS_N10.md).
> A continuação: [PROMPT_CONTINUACAO_2026-11-14_desempenho_11.md](PROMPT_CONTINUACAO_2026-11-14_desempenho_11.md).

## 0. Superfície de colisão

- Base: `main` = `36a312482a` = merge-base desta rodada; conferir `git log HEAD..main` antes de rebasear (vazio ⇒ não rebaseie).
- `git log --oneline 36a312482a..HEAD` = **109 commits**. `git diff --stat`: **440 ficheiros, +12 787/−1 472** (docs: 27, +531/−13; código e
  testes: 413, +12 256/−1 459). Por crate (ficheiros, +/−): **`ph2d-tool-painter` 297 (+7 308/−1 060)** · `ph2d-wet-paint` 37 (+2 153/−210) ·
  `ph2d-render` 19 (+666/−26) · `ph2d-app-painter` 13 (+533/−35) · `ph2d-editor-core` 11 (+470/−13) · `ph2d-painter-brush` 10 (+545/−31) ·
  `ph2d-wet-gpu` 9 (+259/−49) · `ph2d-gpu` 3 · `ph2d-app-host` 3 · `ph2d-host` 2 · `ph2d-det-math` 2 · `ph2d-app-components` 1 · **`shells/desktop` 3
  (+8/−6: `main.rs`, `winit_host.rs`, o ficheiro de input do painter)** · `.config/nextest.toml` · `.github/workflows/spike.yml` (+36) · `Cargo.lock` (+1:
  `ph2d-host` como dependência de `ph2d-app-host`; nenhum pacote externo novo).
- Ficheiros novos (código): `git diff --diff-filter=A --name-only 36a312482a..HEAD -- crates` (46; os de desenho: `tool/paint/wetpaint/{fila_do_dab,sessao_do_wet}.rs`, `undo_anda.rs`, `composite_reserva.rs`, `stamp_route/rota.rs`, `ph2d-wet-paint/src/{solver/difusao,solver/regua,drying/por_tile}.rs`, `ph2d-host/src/ritmo.rs`).
- `PROJECT_SCHEMA`: **0** (`git diff 36a312482a..HEAD -- '*schema*'` vazio). ADR 0 · cenas de smoke 0 · registos de painel 0 · pacotes novos 0.
- Shaders tocados (a placa; a paridade corre na integração): `ph2d-wet-gpu/src/{advect,advect_tinta,seca,seca_tinta}.wgsl`, `ph2d-render/src/shaders/preview_premul.wgsl`.
- ⚠️ **Catraca da shell** (`the_shell_only_shrinks…`): +8/−6 em `shells/desktop` (o ritmo do laço/Mailbox) — reconte na integração; a cura é mover, nunca subir o número.
- O que um merge pode partir (desenho mudado):
  - **`WetPaintState::session` agora é `SessaoDoWet`** (`wetpaint/sessao_do_wet.rs`: `OnceLock<WetSession>` + `Mutex<Option<EmVoo>>`; `as_ref`/`as_mut`/`take`
    drenam). Quem mais tocar a sessão do Wet sem passar por ela vê o lote em voo. O depósito (`Deposito`, `LoteDoDab` dono dos dados) viaja ao operário.
  - **Pen-up adiado LIGADO por omissão**: toda edição estrutural nova tem de abrir por **`abre_edicao_estrutural(&mut self)`** (`layers/undo.rs`), não por
    `snapshot_model()` — senão fotografa o `before` antes de o fecho assentar (o gate `o_pen_up_adiado_fecha_o_passo_de_undo_do_sincrono_ao_bit` morde).
  - **Toda marca suja que não seja a da figura da água**: `marca_do_passo`/`PLANOS_DO_PASSO` (todos − relevo − vela) em `historia.rs`; só depósito/canvas_ops/patch
    escrevem relevo/vela. Todo escritor dos planos de aguada alarga `wet_cum_dirty` (invariante do B44.14).
  - `undo_anda.rs` (Ctrl+Z da tela/relevo no lugar: o tool entrega o `Arc`, a história larga cursor e held), `StoredEntry::Aparece` (relevo que nasce), `PlaneDeltas::side`
    toma `Base {tela, relevo}`; `undo_delta.rs` quem mais o editar.
  - `stamp_route/rota.rs` (`RotaDeCor`, porta única da cor do lote) e `pegada_que_levanta` (Blur×Clone); `Compositor` devolve o `Lido` (RGBA com Smooth Edges).
  - **Roteador de portas**: censo `toda_porta_do_painter_tem_decisao` — porta nova declara a decisão (interna ou não).
  - Cadência: `ph2d-gpu::surface::modo_de_boot` (Mailbox + latência 2) e `ph2d-host::ritmo`.
  - **Fecho (§6): o arranque (B43.7) mora em `ph2d-host::arranque`**, não na `ph2d-app-painter` (catraca −110); ficheiros irmãos novos `undo_planes_journal.rs`,
    `measure_dirty_overclaim_dreno.rs`, `layers/mutate_tela_transparente_tests.rs`. Quem mais editar `undo_planes.rs`/`sim.rs` reconte o tecto de 700.
- Chaves A×B em execução (todas = o caminho de antes): `PH2D_PEN_UP_ADIADO=0` · `PH2D_DAB_SINCRONO=1` · `PH2D_WET_ZERA_JANELA=0` · `PH2D_FOTO_FUNDE=0` ·
  `PH2D_FLUXO_SERIE=1` · `PH2D_TROCA_POUPA=0` · `PH2D_TINTA_MARCA_A_JANELA=1` · `PH2D_CHEIA_ALOCA=0` (reusar) · `PH2D_PREMUL_ESCALAR=1` ·
  `PH2D_OTICA_COMPOSITOR_SMOOTH=0` · `PH2D_PRESENT=fifo|mailbox` · `PH2D_QUADRO_A_PEDIDO=0|1` · `PH2D_MASCARA_CHEIA`.

## 1. O que se fez

| família | o quê (commits) | número |
|---|---|---|
| depósito fora do evento (B14.3/W5, B27.4–6) | `LoteDoDab` dono (`98ae9b1f1b`) → operário (`4dd1b26c6b`) → `SessaoDoWet` (`8e0fb4b634`) → lote encadeado atrás do voo (`a31b689914`) → o quadro não espera o voo (`6c74b78735`, `cbfa856890`) → óptica composta no operário (`9d52615243`) → pen-up adiado (`ba12d28020`, ligado em `9fcbf6c075`) | Flat Bristle 240 4K traço máx 629/538 → 68/91 ms; composição no quadro 72–85 → 10–35; latência caneta→tinta 145–238 ms; soltar +1 s ainda 302–952 (indicativo, load 4–23) |
| Ctrl+Z e troca do Wet (B18.11, B17.5, B18.12) | troca sem reescrever linha igual (`84c5611da8`), fluxo no pool (`e2a0076f8d`), máscara de planos por tile (`9ac2e00834`, `5bb9a1e67f`), tela e relevo no lugar (`bd78b026b1`, `957ced81ff`), «+» com a tinta viva (`28d3516061`, `0539b02194`, `4ff538687a`) | troca_os_planos 23,0 → 16,1 (Wet Flat), 42,3 → 30,4 (Dabber); fluxo 5,4–8,4 → 2,7–3,0 ms; cópias do undo Digital 1 → 0, Impasto 4 → 0; «+» à placa 102 400 B → 0 |
| memória 4K (B44.8, B44.14, B17.6/8, B42.3–5) | 1.º traço Impasto guarda só a janela (`b1c7f516b8`), figura de aguada zera só a janela (`ece79d4dfa`), 13 dos 15 `None` do fork viram caixa (`3db09652eb`, `b466634bba`, `5afc1eb6f6`), reserva de planos (`281780b362`), pilha pelos fios (`410902e7b3`), tecto do undo vivo (`686eed6c60`), Digital até 1024 (`3db09652eb`, `b466634bba`) | undo 1.º traço Impasto 4K 100 407 872 → 1 528 256 B (= 1024²); aguada 58 060 800 → 131 871 B/quadro, Drag Dot 1,08 → 0,15 ms; undo vivo Wet 3 203 → 4 135 MB |
| água (B13.4/7/8/9/10/11, B14.1, B11.3) | secagem por tile (`5811bb2690`, `9f1aadc382`), difusão Jacobi (`1535559a91`), tabela de leis do memo (`cf381d4854`, `d8051b501b`), cedência sem Max velocity (`ac5b3bb566`), freio na grade fina (`9807f80aaf`), régua repõe no Drop (`1772a23f75`, `93fe675872`), óptica em pedaços (`ef8447a265`), RGBA do compositor (`f9ad4996c1`) | Jacobi paralelo 33 × Gauss-Seidel 104 ms (3,1×); memo 296/300 (antes ≤ 192); on_tick 4K 50,0 → 0,02 ms; passo 4K a 16 fios 11,6 ms (8×) |
| camadas e placa (B18.8, B19.10/12, B20.5–10) | fundo do 1.º toque aquece no chip e na troca (`c64c9829a2`, `d1d04dbd60`), anel de subida (`9eed3d4d20`), Repeat Image 1 id (`3a9d717ce6`), premul sem divisão (`5e2047d0d6`, `6880cdae15`) | anel CPU 2,68 → 1,28 ms (sem ganho visível no app); Repeat Image 8 ids/8×33 MB → 1; premul opaca 10,05 → 2,08 ms; ruído 4K vazias 145,9 → 89,2 |
| cadência e UI (B21.1, B23.2, B27.1/2, F6c/d) | Mailbox + ritmo do monitor (`87ac595c7e`), Moves em lote (`87b47543ba`), `tick_hover` só nos widgets escritos (`035f481716`), `RowComponents` por nome (`ac264f6621`) | CPU parado 48–70 % → 3–4 % (a pedido); p95 do traço Wet 4K 61,7 → custo do quadro 14,8/14,1 ms |
| carimbo (B25.5/6/8, B26.4) | força das macros nas 3 rotas (`586616e1e7`, `4b90ee4bc2`, `00a34be1e2`), `RotaDeCor` e Blur×Clone pela porta (`227ea4d819`, `0e4bc7b705`), relevo por peças tiradas de fila (`03fe20255b`) | Flat deitado 227 → 149 µs/dab (raio 100, mín de 41), redondo 211 → 184 |
| build e réguas (B43.2/3/4/7, B44.1–10, B07.3/7) | libm → `det_*` (`d56de55d95`), hugetlb (`0575ea5b87`), hash dos pincéis no CI (`3a36dbe32a`), largura do pool (`83cf4d2e98`), sondas dizem perfil/raio/o que medem (`087277aec6`, `831b7b4690`, `cf2bb7b4ae`, …), portões 4K (`b9eed83b73`, `2c719e8e45`) | libm 23 → 0; minflt 2 955 k → 65 k (−14 %); 24 B/px a 4K; cabeçalhos `--release` 93 → 0 |

## 2. Gates novos (por nome; a lista completa: `git diff 36a312482a..HEAD | grep '^+.*fn '` nos `*_tests.rs`)

- `ph2d-tool-painter`: `o_lote_dono_dos_dados` · `o_lote_encadeado_atras_do_voo_pinta_a_tela_do_sincrono_ao_bit` · `a_drenagem_do_quadro_nao_espera_o_lote_em_voo` ·
  `a_otica_composta_no_operario_pinta_a_tela_do_sincrono_ao_bit` · `o_pen_up_adiado_fecha_o_passo_de_undo_do_sincrono_ao_bit` · `a_particao_em_lotes_nao_muda_o_traco` ·
  `o_fundo_do_toque_aquece_{na_troca_de_ferramenta,no_chip_do_meio}_e_e_o_mesmo_bit` · `a_troca_pura_com_a_tinta_viva_nao_marca_nada_e_a_figura_fica` ·
  `o_mais_com_a_tinta_viva_deixa_a_camada_nova_limpa` · `o_desfazer_do_traco_na_camada_nova_redobra_so_a_janela_do_traco` ·
  `o_relevo_que_nasce_guarda_so_a_janela_e_desfaz_ao_bit` · `a_figura_de_aguada_zera_a_janela_e_nao_a_tela` · `a_peca_mais_pesada_cabe_na_fatia_de_um_fio` ·
  `o_mesmo_pincel_pinta_o_mesmo_bit_em_qualquer_largura_do_pool` · `a_porta_alheia_sai_em_pedacos_e_nao_na_envolvente` ·
  `orcamento_4k::o_traco_a_4k_custa_o_mesmo_que_a_1024` · `um_documento_4k_cabe_no_orcamento_de_bytes_por_pixel` · `as_sondas_dizem_o_que_medem` · `toda_porta_do_painter_tem_decisao`.
- `ph2d-wet-paint`: `a_difusao_jacobi_e_a_mesma_em_serie_e_em_paralelo` · `o_memo_acerta_com_mais_de_64_leis` · `maxvelocity_nao_muda_a_cedencia` ·
  `o_freio_sonda_o_mesmo_ponto_em_rf_1_e_2_e_respeita_a_borda` · `o_fluxo_no_pool_da_o_bit_do_fluxo_em_serie` · `as_pocas_secam_sozinhas` ·
  `marca_linhas_cobre_a_descida_da_placa_sem_marcar_a_tela_inteira` · `nenhum_quadro_de_movimento_marca_a_tela_inteira`.
- Placa (`--ignored`): `ph2d-wet-gpu` `a_historia_da_placa_marca_so_as_linhas_da_descida_e_o_undo_e_o_mesmo`; paridade `--test tinta --test paridade`.
- Outros: `ph2d-gpu::o_modo_de_apresentacao_e_valido` · `architecture_toda_razao_de_relogios_corre_na_faixa_isolada` · pranchas `prancha_b13_10_…`, `prancha_b13_4_…` (`--ignored`) → [`B13.10`](../missao_4k/pranchas/B13.10), [`B13.4`](../missao_4k/pranchas/B13.4).

## 3. ABERTO, por ordem

1. **A corrida de fecho no app e na placa** (a rodada não a fez toda): `--ignored` da janela — B44.9 `a_historia_da_placa_marca_so_as_linhas…`, B17.4/B17.5 em ms a 4K,
   B18.7 A×B, B20.8 premul u32 na placa; o app 4K do pen-up adiado e do «soltar +1 s» (Flat Bristle 240: 302–952 ms ainda; só o F2k ligou o adiamento, **não medido no app**).
2. **Gate de tempo do Wet vermelho sozinho** (B26.5): `o_deposito_de_um_pincel_pesado_de_fabrica_vai_pela_rota_que…` — série/produto 1,13 < 1,2 a load 5; não é flake; bissecção por fazer (`b8a42c811d` passou a ler as decisões).
3. **B23.6 bloqueado**: `sudo pacman -S weston` (do dono), depois `mede_pen_up_4k.sh wayland`. **B44.11–B44.13 bloqueados**; **B08.6** é a matriz do fecho.
4. **B26.1–B26.3** (constantes do dab, decidir em série/pintar em bandas, piso por custo): não tocados. **B25.8 cura**: contagem feita, cura muda aparência e exige a paridade ao bit da `ph2d-paint-gpu`.
5. **B17.6**: 2 dos 15 `None` por fechar. **B18.3**: a 1.ª troca de cada pincel Wet 58–91 ms (`troca.commit` 49–72, causa por medir). **B18.11**: Ctrl+Z Wet 60–74 ms no binário base, sem cura que feche o alvo 33. **B17.7**: medido, cura não aplicada. **B19.14**: pré-condição falsa, não se constrói.
6. **Do dono**: B13.3 · B14.2 · B30.5 · B25.7 (malha: smoke) · B15.7 (medição) · B13.12 (palpite ligado muda o bit).
7. A matriz final ([N10_MATRIZ_FINAL.md](../missao_4k/medicoes/N10_MATRIZ_FINAL.md)) decide quais alvos ✗ restam (toque Filbert/Wet Liner 4K ~40–57 ms na base).

## 4. ⛔ Recusas MEDIDAS (não reconstruir)

| ideia | medição | porquê |
|---|---|---|
| reusar o acumulador da composição cheia (B19.12) | −12–17 ms no mínimo, app 4K Wet, 4 blocos intercalados | zerar 133 MB à parte é ida e volta à DRAM; a omissão ALOCA |
| pool próprio para a compressão (B18.7) | 1.º traço mín 23,9 próprio × 18,7 global (Wet Wash 4K, 8 corridas) | volta ao global; chave e gate apagados |
| cache f32 da pilha de baixo (B20.2) | região 256² 0,376 ms × tela 37,4 | já é barato |
| views/bind groups em cache (B20.3) | `run` 0,115 · `run_rect` 0,087 ms | nada a ganhar |
| FilmLut em paralelo (B26.4) | 0,0576 → 0,0526 ms (1,09× com 32 fios) | volta à série; o memo sem trinco fica |
| `cos` por célula fora do laço (B14.3) | −0,1 % instruções (ruído ±0,2 %), hash igual | não compra nada |
| memo da saída da mistura espectral (B28) | +0,3 % instruções no Flat Bristle 240 | pior |
| cura 4 `de_e_repoe` numa passada (B18.11) | 4,69/4,50 → 4,69/4,74 ms | sem ganho; retirada |
| anel de subida no app (B20.6) | CPU 2,68 → 1,28 ms, mas sem ganho visível no app 4K (ABBA×2) | o anel fica por omissão; não se mede de novo |
| sRGB por tabela / `powf` do `_in_linear` (B20.8) | sem chamador no app | recusa por leitura |
| piso serial do advect (B13.6-resto) | 16 fios: passo 93,2 → 11,6 ms (8×) | não há piso; sem ADR |
| reconstruir o 2.º traço frio (B17.7) | a régua não mostra o custo | cura não aplicada |
| a linha `macro_entra` a concluir o fecho (F2j) | o `porta!` embrulha o `on_tick` e anularia o adiamento | recusada |

## 5. Regras aprendidas (novas)

- **O guarda recusa `--ignored` a subagentes**: sondas de placa e A×B no app correm na JANELA PRINCIPAL; o subagente escreve o comando exato e o gate.
- **`cargo-test-narrow.sh` esconde a saída no verde** e o teste da placa «passa» sem adaptador: confirme com `--nocapture`.
- **`git commit -- <paths>` levou edições a meio de outro agente** para o commit do vizinho (árvore partilhada): confira `git diff --cached --stat` e os ficheiros por rastrear antes.
- **Árvore partilhada a não compilar trava os outros** (E0596/E0027/E0599 de WIP alheio): comite em passos que compilam; quem não compila não deixa `M` pendurado.
- **Perf funde genéricos com o mesmo código**: a folha `search_tree<BoardId,u32>` era alias de `search_tree<NodeId,f32>` — o NOME no perfil mente; leia o símbolo mangled.
- **Medir em 4K RETANGULAR** (`PH2D_FPS_TELA=3840x2160`), não 4096²; todo cabeçalho de sonda diz o perfil (`--profile smoke`) e o raio lido (`sonda_raio`).
- **O smoke do app parado a 50 % de CPU era o laço contínuo**, não o Painter: `PH2D_QUADRO_A_PEDIDO=1` o põe a pedido (3–4 %).
- **Espera sem prazo dentro de job do rayon é deadlock** (Bug #66): dentro de job o lote pousa síncrono.

## 6. Prova de fecho

Fecho 14/11 12:02–12:30 sobre `90b2643db2` + 3 commits de cura (`5819afb965` fmt · `f4097b1e31` clippy · `696ebb2f2f` LOC). **Impactados e suíte completa ficam para a integração.**

| passo | comando | resultado |
|---|---|---|
| fmt | `rustfmt --edition 2024 --check` nos `.rs` de `36a312482a..HEAD` | ✗ 23 ficheiros (ordem de `mod`, quebras) → `cargo fmt -p` nas 13 crates → ✓ |
| clippy | `PH2D_SUITE=fecho ph2d-run.sh cargo clippy -p {tool-painter,app-painter,wet-paint,wet-gpu,painter-brush,render,editor-core,app-components,det-math,host-desktop,app-host,gpu,host} --all-targets --keep-going -- -D warnings` | ✗ 15 → ✓ rc 0. Lib: `drying_pass` importado e morto (`sim.rs`), `TileFino::tira` sem chamador, `size_px_to_norm`/`Arc` sem uso, `bytes_zerados`/`forca_zerar_a_janela` só do teste ⇒ `#[cfg(test)]`, `voa`/`fica_atras`/`resto_atras` com tipo mais privado que o método ⇒ `pub(in wetpaint)`. Testes: `mut` a mais, `type Traco`, `ok().expect`, `manual_range_contains`, `as_chunks_mut`, `if let` colapsado. Zero `allow` novo |
| LOC | `architecture_workspace_file_loc_cap` + `the_shell_only_shrinks` | ✗ **a linha chegou ao fecho VERMELHA**: 6 ficheiros > 700 (`undo_planes` 745, `measure_dirty_overclaim` 742, `sim` 741, `mutate` 706, `state` 701, `brush_settings` 701) e `ph2d-app-painter` 24 385 > 24 359. Cura por MOVER (`696ebb2f2f`): **`arranque.rs` (re-exec hugetlb, B43.7) → `ph2d-host::arranque`** (ciclo de vida do processo; `main.rs` chama `ph2d_host::arranque::liga_paginas_grandes()`), `undo_planes_journal.rs`, `measure_dirty_overclaim_dreno.rs`, `sim_step_atomic_reference` → `sim_resumable_tests.rs` (o único chamador), `mutate_tela_transparente_tests.rs` → ✓ |
| outros do fecho | `check-standalone-optional.sh` · `check-workflow-packages.sh` · `cargo check -p ph2d-host --target aarch64-apple-darwin` (deny) | ✓ · ✓ (33 nomes / 526 membros) · ✓ (o `cfg(target_os)` do arranque mudou de crate) |
| gates por nome (`cargo-test-narrow.sh`) | editor-core `--test it`: shell_only_shrinks, claude_md_cabe_no_orcamento (+ §5.1 700 B), workspace/panel/widget LOC, `architecture_toda_{flake_de_carga,razao_de_relogios}_corre_na_faixa_isolada` | ✓ 16 |
| | host-desktop `file_loc_caps` · det-math `o_motor_nao_chama_a_libm` · host `arranque` · gpu `o_modo_de_apresentacao_e_valido` | ✓ 4 · ✓ 3 · ✓ 1 · ✓ 1 |
| | wet-paint: os 8 da §2 + `sim_resumable_tests` | ✓ 10 |
| | tool-painter: os da §2 + `orcamento_4k`, `o_lote_em_voo_nao_suja_a_previa…`, `a_pilha_do_composite_abre_em_serie`, `fila_do_dab`, `tela_transparente_tests`, `journal_delta`, `undo_planes`, `composite_acumulado` | ✓ 46 |
| alarmes de pincel | `cargo-test-narrow.sh ph2d-tool-painter --profile smoke --test it -- a_largura_de_fabrica_e_a_aprovada a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` | ✗ 2 **de propósito** (não regravados): miniatura de **Blender Dots, Blender Hard, Blender Soft, Blender Textured, Ink Brush, Oil Wash, Smear Flat, Smear Shattered, Smear Soft, Smear Spikey, Smear Wet, Water Brush** (12); largura só **Smear Wet**. Relatório `target/relatorio-pinceis/index.html` gerado 12:21 |
| `PROJECT_SCHEMA` | `git diff --name-only 36a312482a..HEAD \| grep -i schema` | vazio ⇒ 0, sem recontagem |
| placa (`--ignored`) | `a_historia_da_placa…`, paridade `--test tinta --test paridade` | não corridos no fecho: integrador |

**Matriz final** ([N10_MATRIZ_FINAL](../missao_4k/medicoes/N10_MATRIZ_FINAL.md), 4K papel, NOVA=1, mínimo de 2 rondas, custo × custo): Flat Bristle 240 traço p95 80 → **30,6**, soltar 60 → 30,4; CPU parado ~50 % → **3–8 %** sem água; Wet de omissão soltar 19,6 → 13,7, Ctrl+Z 27,5 → 20,5; 1.º toque da camada nova Composite 43 → 18. **§Regressões curadas:** `d58329704e` (o lote em voo sujava a prévia sem caixa ⇒ tela 4K inteira por quadro): Filbert traço 15,4 → **10,7**, toque 21,6 → 15,3, Wet Liner toque 15,8 → 14,4; `90b2643db2` (a pilha do Composite abre em série; o `par_clone`/`par_fill` do B17.8 custava +8–9 ms): toque 30,4 (base) → **25,7**. Nenhuma regressão aberta; Ctrl+Z dos Wet segue ~45 ms ✗ (aberto, §3). Chave nova: `PH2D_PILHA_PELOS_FIOS=1`.

**Réguas** (`bash scripts/agent-loop-profile.sh`, 20 sessões):
```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                163   alvo: <= 800
  ✗ cargo test : cargo check                333 : 122   alvo: <= 1,0  razao 2.7x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.7%   alvo: <= 2%
  ✓ contexto relido por passo (media)         244 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               68 mil   alvo: <= 80 mil
```
`rm -rf target/{debug,smoke}/incremental` (54 G + 19 G) antes do binário. **Binário smoke, 2.ª corrida:**
```
▸ linha line_painterwatercolor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 300s
    Finished `smoke` profile [optimized] target(s) in 0.21s
```
(1.ª corrida: 59,3 s, a partir do incremental apagado.)

## 7. Smoke do dono (UI de cartões)

Os alarmes de pincel ficam VERMELHOS de propósito: mudaram **11–12 pincéis do Wet** (Blender Dots, Hard, Soft, Textured, Oil Wash, Smear Flat, Shattered, Soft, Spikey, Wet, Water Brush)
por B13.10 (a água já não prende na borda do quadro) e B13.4 (a secagem de cada poça é só dela). Só a aprovação do dono regrava.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`
2. **Os pincéis que mudaram.** Abra [`target/relatorio-pinceis/index.html`](../../../target/relatorio-pinceis/index.html) no navegador e veja as miniaturas
   dos 11 pincéis acima; depois as pranchas [antes/depois da borda](../missao_4k/pranchas/B13.10/4k_papel_antes_depois.png) e
   [antes/depois da secagem](../missao_4k/pranchas/B13.4/4k_papel_antes_depois.png). Tem de acontecer: a diferença só na borda e nas poças (a água não gruda no limite;
   uma poça calma não seca mais devagar por causa de outra). Deu errado se: o traço inteiro mudou de cor ou de forma.
3. **Água que seca por poça.** Painter, tela 3840×2160, pincel «Smear Wet»: pinte duas poças longe uma da outra e, enquanto uma ainda escorre, olhe a outra. Tem de acontecer: a calma continua a secar no mesmo ritmo.
4. **Pontos de referência do óleo (B44.3).** Quem olha a ficha do papel vê o «Relevo do papel» **3,86** onde dizia 2,24: é a régua que passou a ler o que a tela mostra (nada foi repintado); diga se o grão do papel no óleo continua como gosta.
5. **Blur com a caneta deitada (B25.6).** Pincel «Blur» com a caneta inclinada, sem textura: a ponta agora deita (como o Clone). Deu errado se: a ponta continuar redonda ou o borrão sair fora do traço.
6. **Malha (B25.7).** Pincel com a malha ligada: confira se o carimbo cai onde a ponta está (registado para o seu olho).
7. **Repeat Image sem piscar (B20.10).** Camada com Repeat Image em 4K: mova e solte. Tem de acontecer: as 8 telhas ficam fixas, sem sumir nem piscar. Deu errado se: alguma telha pisca ao parar.
8. **Size até 1024 no Digital (B42.5).** Pincel Digital: arraste o controlo «Size» até ao fim — tem de chegar a 1024 (o Wet e o Óleo ficam em 512). Os tamanhos de antes ficam onde estavam até 0,75 do trilho.
9. **Flat Bristle 240 já não congela.** Pincel «Flat Bristle», Size 240 (`]` ×10) em tela 4K: pinte e solte. Tem de acontecer: a tela responde sem parar; a tinta aparece com um pequeno atraso atrás da caneta e o traço fecha logo após soltar. Deu errado se: a janela trava mais de meio segundo ou o Ctrl+Z logo ao soltar não desfaz o traço inteiro.
10. **Ctrl+Z instantâneo.** Pinte com um pincel Wet (Filbert) e dê Ctrl+Z e Ctrl+Shift+Z várias vezes, também depois de «+» camada nova. Tem de acontecer: desfaz e refaz de imediato, a tinta volta igual. Deu errado se: sobra um contorno molhado ou um retângulo diferente.

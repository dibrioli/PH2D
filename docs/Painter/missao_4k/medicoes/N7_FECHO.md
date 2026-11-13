# N.º 7 — fecho da onda (13/11)

Onda `da83348ee0..234ef11b64` (41 commits; 69 ficheiros, 3 crates: `ph2d-tool-painter`, `ph2d-wet-paint`,
`ph2d-app-painter`). Na linha não correram a suíte completa nem o `nextest-impacted` (CLAUDE.md §2): só testes por nome,
fmt e clippy das crates tocadas 1× e os alarmes de pincel.

## Commits do fecho

| commit | o quê |
|---|---|
| `58db5f05dc` | fmt: `wetpaint/composite/pixel.rs` (o campo `vbase`; a onda tocou-o e desformatou-o, na base estava limpo) |
| `08c3e5101e` | clippy `-D warnings`: os 10 erros, todos em ficheiros da onda (abaixo) |
| `9a6b40f0ad` | censo do preview: `PaintState.aquecimento` (item 3f) classificado como `CACHE` |

## fmt

`rustfmt --edition 2024 --check` (a workspace é edição 2024) sobre os 70 `.rs` do diff: só `composite/pixel.rs`
reprovava (1 linha). Formatado. Os ficheiros editados no clippy voltaram a ser conferidos (só
`watercolor_vidro_journal_tests.rs` precisou de reformatar o alias novo).

## clippy (`PH2D_SUITE=fecho`, `-p` das 3 crates, `--all-targets -D warnings`)

Três corridas: a 1.ª parou em `ph2d-wet-paint` e escondia a `ph2d-tool-painter`. Os 10 erros nasceram todos na onda;
nenhum era anterior. Todos corrigidos; a 3.ª corrida saiu `exit 0`.

| ficheiro | lint | cura |
|---|---|---|
| `wet-paint/solver/advect_lanes_tests.rs:53` | `manual_is_multiple_of` | `!r.u().is_multiple_of(5)` |
| `tool-painter/tool/paint/mais_sem_diff_tests.rs:54` | idem | `k.is_multiple_of(2)` |
| `tool-painter/tool/paint/watercolor_vidro_journal_tests.rs:201` | idem | idem |
| `tool-painter/undo_window_vidro.rs:101` | `dead_code` (`vidro_before`) | só os testes o lêem: `cfg(any(test, debug_assertions))` → `cfg(test)` |
| `tool-painter/tool/paint/watercolor_vidro.rs:471` | `too_many_arguments` (8/7) em `abre_o_vidro` | `allow` local (a crate já o usa 33×; a função recebe os campos por estar com a sessão emprestada) |
| `tool-painter/tool/paint/wetpaint/adormecida/silencio.rs:95` | `let_unit_value` (`Horas = ()` fora de teste) | `Horas` passa a struct unitária fora de teste; `cfg_attr(not(test), allow(missing_const_for_fn))` em `horas_novas`, como já tinha `marca` |
| `silencio.rs:59` · `sonda_partes.rs:6` (×2) · `watercolor_vidro_journal_tests.rs:481,510` | `type_complexity` | aliases `Horas` (reusado no `thread_local`), `Partes`, `Documento` |

## Alarmes de pincel (por nome, sem `PH2D_ASSENTA_PALPITE`, nunca `PH2D_PINCEIS_APROVA`)

`cargo test -p ph2d-tool-painter --test it -- a_largura_de_fabrica_e_a_aprovada a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`
(53 s), comparado com a lista «exato» de [`n7_assenta/README.md`](n7_assenta/README.md):

| | antes (exato, `2c3a2185b0`) | agora (HEAD do fecho) |
|---|---|---|
| miniatura | 75 | 75: **os mesmos nomes** (`diff` vazio); os 15 do Wet com a **mesma contagem de píxeis** |
| largura | 2: Oil Wash, Smear Wet | 2: Oil Wash `[5.38, 14.90, 52.44]` (sem aprovada), Smear Wet `[4.95, 12.28, 50.18]` (= exato) |
| **total** | **77** | **77**, diferença **nenhuma** |

## Os 8 vermelhos, a classificar

Corrida por nome em `da83348ee0` (worktree temporária com `CARGO_TARGET_DIR` próprio, já removida) e na HEAD, filtro
`preset` + os 3 antigos. **O conjunto de falhas é idêntico** nas duas: 7 no alvo lib e 7 no `it` (no `it` estão os 2 alarmes
e mais 5 de presets que já reprovavam, fora desta lista).

| teste | em `da83348ee0` | classificação |
|---|---|---|
| `controlos_e10_tests::cada_preset_de_fabrica_mapeia_por_funcao` (57/46 × 56/45) | vermelho | anterior |
| `receita_e4_tests::cada_preset_de_fabrica_pousa_o_mesmo_pigmento_e_agua_por_dab` (46,10,1 × 45,10,1) | vermelho | anterior |
| `watercolor_noise::tests::slot_paper_preset_tiles_seamlessly_under_tiling` (costura `y=3`; conta como «tiling» e como antigo) | vermelho | anterior |
| `portao_mae_f0::o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` (Filbert Dry) | vermelho | anterior |
| `preview::censo_tests::cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao` | vermelho (`fundo_em_cache`, `traco_relogio`, `movido_desde_o_passo`, `canvas_wet_relogio`) | anterior, **mas a onda juntou `PaintState.aquecimento`** (item 3f). Classificado `CACHE` (`9a6b40f0ad`); a mensagem voltou a ser **igual à da base**. Os 4 campos antigos ficam como dívida anterior |
| `watercolor_flow::o_classic_e_o_paper_edge_zero_sao_o_byte_de_hoje` | vermelho | anterior |
| `substrate_relief_digital_tests::a_primeira_marca_do_wet_refaz_a_imagem_inteira` | vermelho | anterior |

Nenhuma regressão da onda. (A lista dos «5 do filtro `preset`» conta a costura do papel uma vez; com os 3 antigos ficam 7
testes diferentes.)

## Gates da onda, por nome

Os `#[test]` novos do diff (`da83348ee0..HEAD`), numa corrida:

- `ph2d-tool-painter` (lib): **31/31 verdes**: `a_camada_nova_e_o_vidro_elidido_dao_o_bit`,
  `a_compressao_espera_o_silencio_e_acontece`, `acordar_na_espera_cancela_a_compressao`,
  `a_janela_da_tinta_por_palavras_e_a_da_serie`, `apagar_duplicar_e_agrupar_com_a_tinta_viva_do_wet_marcam_so_a_janela`,
  `apagar_duplicar_e_agrupar_recompoem_so_a_janela_da_tinta`, `a_tela_nova_de_4k_nasce_sem_tocar_paginas`,
  `a_troca_com_a_aguada_no_papel_de_cor_nao_recompoe_e_a_figura_fica`, `a_troca_com_a_tinta_viva_do_wet_marca_so_a_janela_da_tinta`,
  `a_troca_com_um_sujo_pendente_nao_recompoe_e_a_figura_fica`, `a_troca_na_pista_da_placa_nao_pede_drenagem_e_a_figura_fica`,
  `duplicar_a_camada_com_vidro_e_pintar_uma_nao_muda_a_outra`, `o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer`,
  `o_ctrl_z_no_meio_da_sessao_molhada_da_o_vidro_da_base_ao_bit`, `o_fundo_cortado_para_entre_faixas`,
  `o_fundo_do_toque_aquece_na_troca_de_pincel_e_e_o_mesmo_bit`, `o_fundo_do_toque_aquece_na_troca_e_e_o_mesmo_bit`,
  `o_fundo_do_toque_compoe_se_num_fio_e_e_o_mesmo_bit`, `o_journal_do_vidro_guarda_os_tiles_que_a_regiao_toca_e_so_a_primeira_vez`,
  `o_mais_sem_diff_da_o_bit_no_desfazer_e_no_refazer_wb`, `o_mais_sem_diff_da_o_bit_no_desfazer_e_no_refazer_wet`,
  `o_segundo_traco_nao_copia_o_plano_do_vidro`, `o_tecto_conta_o_mais_pelo_tamanho_real`,
  `o_tecto_do_desfazer_conta_o_journal_do_vidro`, `o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal`,
  `o_traco_que_cruza_tiles_guarda_so_os_tiles_tocados`, `o_vidro_da_base_da_sessao_e_o_do_inicio_da_sessao_ao_bit`,
  `o_vidro_do_traco_volta_ao_bit_no_desfazer_e_no_refazer`, `quem_segura_o_vidro_no_segundo_pen_down`,
  `sem_a_chave_a_compressao_comeca_logo`, `trocar_de_meio_a_meio_da_sessao_da_o_vidro_da_base_ao_bit`.
  O 32.º, `sonda_gestos_de_camada_na_cena_do_app_a_4k`, é sonda `#[ignore = "sonda de tempo"]`, não é gate.
- `ph2d-wet-paint`: **3/3**: `o_complemento_nos_blocos_da_o_commit_a_parte_ao_bit`, `o_transporte_em_lanes_da_o_escalar_ao_bit`,
  `o_palpite_do_assentar_fica_perto_da_conta_exacta`.
- `ph2d-app-painter`: **1/1**: `apagar_duplicar_e_agrupar_compoem_na_placa_so_o_sujo_ao_bit`.

Ressalva: a corrida dos gates da `ph2d-tool-painter` compilou enquanto as curas do clippy entravam. Essas curas só mexem em
`cfg`, aliases e `is_multiple_of`, sem mudar o que o código faz. Depois delas, o censo voltou a correr por nome e
`cargo build --profile smoke --tests -p` das 3 crates compila (25 s).

**Re-corrida depois das curas (13/11, HEAD `6d13348bb0`):** os 31 da `ph2d-tool-painter` (lib), por nome, numa corrida:
`bash scripts/cargo-test-narrow.sh ph2d-tool-painter --lib -- <os 31>` → **31 passaram · 0 ignorados** (correr 5,9 s). A
ressalva acima fica fechada. ⚠️ Num zsh, `$N` sem aspas NÃO se parte em palavras: os 31 nomes chegam ao libtest como UM
filtro e o veredito é «0 testes» (exit 4 do script). Use `${=N}` ou um array.

## `scripts/agent-loop-profile.sh`

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
  ✗ paralelismo de ferramenta              1.12/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                163   alvo: <= 800
  ✗ cargo test : cargo check                402 : 159   alvo: <= 1,0  razao 2.5x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.8%   alvo: <= 2%  (15 de 819)
  ✗ contexto relido por passo (media)         259 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               67 mil   alvo: <= 80 mil
```

`rm -rf target/*/incremental` feito no fim.

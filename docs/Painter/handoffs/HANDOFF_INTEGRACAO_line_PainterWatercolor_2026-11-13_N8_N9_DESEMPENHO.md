# Handoff de integração — linha PainterWatercolor · 13/11 (janelas n.º 8 e n.º 9): desempenho 4K — gestos de camada no papel de cor e no Impasto, cartão Performance, a cura do vidro do escorrido e a troca do Wet sem a dobra

> A linha NÃO está integrada (dono 13/11: «não integre»); quem o dono nomear integra. Leitor: o integrador e a próxima LLM.
> Este handoff SOMA-SE ao [n.º 7 ONDA7_DESEMPENHO](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_ONDA7_DESEMPENHO.md) (e aos
> anteriores que ele lista): a linha inteira integra-se de uma vez, com os dois.
> Notas (a verdade dos números): [`N8_NOTAS.md`](../missao_4k/medicoes/N8_NOTAS.md) · [`N9_NOTAS.md`](../missao_4k/medicoes/N9_NOTAS.md).
> A continuação: [PROMPT_CONTINUACAO_2026-11-13_desempenho_10.md](PROMPT_CONTINUACAO_2026-11-13_desempenho_10.md).

## 0. Superfície de colisão

- Base: `main` = `4f46a3573c` = merge-base; `git log HEAD..main` vazio em 13/11 22:30. **Não rebaseie** salvo se o `main` andar.
- Estas janelas = `0a0eaa9208..HEAD` (`<este commit>`), 11 commits + o fecho. Código: **45 ficheiros, +1 715/−69** em `ph2d-tool-painter` (28),
  `ph2d-panel-painter-layers` (6), `ph2d-app-painter` (3), `ph2d-wet-paint` (3), `ph2d-i18n` (1) e **`shells/desktop` (4, +21)**: a
  preferência `wet_guess` do cartão Performance (`prefs.rs`, `init.rs`, `forwarding_persist.rs`, teste em `prefs_tests.rs`).
- ⚠️ **`the_shell_only_shrinks_and_every_app_crate_too`**: as +21 linhas da shell são da pref (o padrão de toda pref do app mora lá);
  já estava vermelho na linha por app-painter/app-pixel (n.º 8 §vermelhos). A cura é do integrador: recontar, ou mover a leitura das
  prefs do Painter para a crate da família, se a catraca o exigir.
- Contadores (DELTA destas janelas): `PROJECT_SCHEMA` 0 · ADR 0 (a linha inteira cria 0201–0204; reconte contra o `main` do dia) ·
  cenas de smoke 0 · Cargo.lock sem pacote novo · registos de painel 0 · `quantas_entradas…a_carga_de_comandos…`: `wet_tuning` 12 × 14
  (o cartão Performance; número que soma entre linhas — reconte).
- O que um merge pode partir (desenho mudado):
  - `undo_absorb.rs` (rota `Composto` do vidro, `RotaDoVidro`), `undo_delta_composto.rs` (novo), `undo_delta_absorb.rs` (`absorbed`
    aceita topo `Tiles`), `undo_record.rs` (`base_for_top_com`), `undo_window_vidro.rs` (`vidro_composto`): quem mais escrever a
    absorção do escorrido.
  - `tool/paint/stamp_preview.rs` (`mark_dirty` mantém `sujo_da_figura`; `mark_dirty_so_a_figura`), `tool/layers/preview.rs`
    (`preview_gpu_relevo`), `tool/layers/cache.rs` (as duas marcas da tinta que adormece/acorda): **toda marca suja nova deve ser a
    `mark_dirty` de sempre** — só a ótica que não escreve píxeis pode usar a «só da figura».
  - `ph2d-app-painter/src/painter_gpu_preview.rs` (a dobra usa a parte do relevo; o oráculo `SEMPRE_INTEIRA` dobra a tela inteira)
    e o novo `painter_gpu_preview_chaves.rs` (as chaves A×B, cortadas pelo tecto de LOC).
  - `tool/layers/cache.rs::alcance_da_tinta_de` (tinta ∪ vidro ∪ corpo+1 texel) e `corpo_da_camada`.
- Chaves A×B em execução (todas `=1` = o caminho de antes): `PH2D_TROCA_RECOMPOE`, `PH2D_ESTRUTURA_RECOMPOE`, **`PH2D_DOBRA_A_REGIAO`** (nova).

## 1. O que se fez

| commit | o quê | medido (app 4K, ms) |
|---|---|---|
| `971585a122` | WB apagar/duplicar/grupo com a aguada no papel de cor recompõem só tinta ∪ vidro (B18.6) | dup/apagar/grupo 90/72/78 → 22/16/17 |
| `7649aacc5f` | cartão **Performance** no fim do pincel do Wet: «Faster wet paint» (o palpite da `assenta`), desligado de fábrica, nas prefs (`wet_guess`). **AVX2 medido e recusado** (≈ 0, tecto 4 %) | — |
| `cbd47ce04b` | Impasto apagar/duplicar/grupo recompõem só tinta ∪ corpo + 1 texel | gate 36 casos; app sem régua de calma (§3) |
| `3026ea2e53` | o `[quadro-lento]` imprime as partes do Painter no quadro | — |
| `69158c3890` | **defeito**: o Ctrl+Z logo após um traço Wet sobre aguada não desfazia o vidro do escorrido — a absorção leva-o | — |
| `8a1714c8d0` | a cura acima custava a varredura de 58 MB: o vidro do topo **compõe-se com o journal** nas caixas dos dois (ADR-0204) | sonda 4096²: pen-down 46–64 → **14,2**; Ctrl+Z 71–93 → **35,6** |
| `c166588404` | a troca com a tinta viva do Wet dobra na placa **só a parte do relevo** (a ótica muda, o relevo não) | gate: a dobra sai (1 texel); app sob load 7–20 inconclusivo (troca Filbert 46,5–46,9 × 49,4–50,9) |

Detalhe, mecanismos e mutações: N8_NOTAS §Item 2/3/Cartão e N9_NOTAS §Item 2/3.

## 2. Gates novos (por nome)

- `ph2d-tool-painter`: `apagar_duplicar_e_agrupar_com_a_aguada_no_papel_de_cor_recompoem_so_tinta_e_vidro` · os 36 casos do Impasto
  (`cbd47ce04b`) · `o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer` · `o_vidro_composto_da_o_documento_do_vidro_segurado` ·
  `a_troca_com_a_tinta_viva_muda_o_relevo_so_onde_a_placa_o_redobra`.
- `ph2d-app-painter`: `a_troca_com_a_tinta_viva_dobra_na_placa_so_o_relevo_ao_bit` (placa; sem adaptador sai sem afirmar).
- `ph2d-wet-paint`: `palpite_da_caixa` (tests/). `shells/desktop`: `wet_guess_round_trips_and_an_old_file_is_off`.
- Sondas `#[ignore]` (não são gates): `sonda_custo_da_cura_do_vidro` (`--profile smoke -- --ignored`, escreve `target/sondas/cura_do_vidro.txt`).

## 3. ABERTO, por ordem (para a próxima janela)

1. **A régua de CALMA da matriz NOVA=1** (Impasto e WB dup/apagar/grupo; os gestos do Wet com o item 3): nesta janela outra linha
   compilou e correu o app a meio (load 19–35). Corra `target/n9/bateria1.sh` (ou o molde dele) só com `uptime` < 5 antes e depois.
2. **Os gestos do Wet que sobram**: depois do item 3, o preview da troca ainda compõe a janela da tinta inteira na placa (a ótica
   muda mesmo); e o Ctrl+Z na camada nova (preview 18–24, dobra 12 — aqui o relevo MUDA, a dobra é legítima: medir se a janela é a do
   traço ou maior). E os quadros de 35 ms com o Painter a 0,4 ms (FORA dele: shell/apresentação/placa).
3. **Impasto duplicar**: Painter 7 ms de 35 — o resto fora do Painter.
4. Itens 6–10 do prompt n.º 8: toque Filbert/Composite 34–43 ms; CPU parado 50–54 % (da shell: só registar); B19.10/B20.6/rayon;
   espera do pen-down; `fork_par_raw`.
5. Vermelhos ANTERIORES (não destas janelas, listados em N9_NOTAS e no prompt n.º 9 §6): `a_primeira_marca_do_wet_refaz_a_imagem_inteira`,
   `com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`, `a_secao_desligada_e_o_traco_de_hoje_ao_byte`,
   `cada_preset_de_fabrica_mapeia_por_funcao`, `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao`,
   `o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde`, `receita_e4`, os 4 de Impasto, `h2_os_cartoes::the_top_and_the_basics…`,
   `the_shell_only_shrinks…`, `quantas_entradas…`; flake de isolamento: `o_motor_nao_le_a_ponta_alem_da_pegada…`,
   `a_silhueta_alem_da_pegada_nao_e_avaliada…` (verdes sozinhos).

## 4. ⛔ Recusas MEDIDAS (não reconstruir)

| ideia | medição | porquê |
|---|---|---|
| AVX2 na `assenta` | ≈ 0, tecto 4 % (N8) | não compra nada; e `unsafe_code = "forbid"` |
| `from_journal` separado para saber se o escorrido mudou o vidro | +13 ms no pen-down | a composição responde isso de graça, caixa a caixa |
| clonar o vidro do reset e rebobinar (a cura de `69158c3890` como estava) | pen-down 46–64, Ctrl+Z 71–93 ms | três varreduras de 58 MB; a composta faz o mesmo bit |
| pular a marca da troca do Wet | — | a ÓTICA muda na janela da tinta: a figura tem de recompor; só a dobra sai |

## 5. Regras aprendidas (novas)

- **Um oráculo que partilha uma etapa com a pista nova não a testa:** o `SEMPRE_INTEIRA` compunha a tela inteira mas dobrava pela mesma
  região da ferramenta — duas mutações da dobra saíam verdes. O oráculo tem de refazer CADA etapa que a cura toca.
- **Mutação que não morde pode ser o cenário, não a cura:** a troca para a camada já activa é no-op (o passo 6 não trocava); escreva o
  passo que exerce o caminho (pinta-e-troca, troca-e-pinta sem drenar) e confirme-o com uma impressão antes de confiar no verde.
- **O `cargo-test-narrow.sh` esconde a saída no verde e o teste da placa «passa» sem adaptador:** confirme a placa com `--nocapture`.
- **Um teste corre com o cwd na pasta da CRATE:** `target/sondas/…` relativo cai em `crates/<c>/target/` (apague-o depois).
- **Carga alheia a meio:** grave `uptime` por pincel (`load_*.txt`); não espere calma — meça diferenças no mesmo processo.

## 6. Prova de fecho

- `rustfmt --edition 2024 --check` nos `.rs` de `0a0eaa9208..HEAD`: limpo.
- `clippy -D warnings --all-targets` (`PH2D_SUITE=fecho`) em `ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-panel-painter-layers`,
  `ph2d-wet-paint`, `ph2d-i18n`: 3 erros, todos destas janelas, curados no fecho (import morto em `estrutura_vidro_tests.rs`, `?` em
  `alcance_da_tinta_de`, `mut` a mais no gate do vidro composto); depois limpo. Gates tocados re-corridos por nome: verdes.
- Alarmes de pincel por nome (`--profile smoke --test it`): **77 = 77** (miniatura 75; largura 2: Oil Wash `[5.38, 14.90, 52.44]`,
  Smear Wet `[4.95, 12.28, 50.18]`), os mesmos do n.º 7 §8. Nenhum pincel mudou (estas janelas não tocam motor nem preset).
- `cargo build -p ph2d-host-desktop --profile smoke` 2×: `Finished … in 0.25s` as duas, zero `Compiling`; o único aviso é o do `main`
  (`main.rs:941`, `unreachable pattern`).
- `rm -rf target/*/incremental` feito. `agent-loop-profile.sh`: paralelismo 1,14/passo (✗, alvo 1,5) · test:check 2,3 (✗) · relido
  239 mil/passo (✓).
- A suíte completa e o `nextest-impacted` são do integrador.

## 7. Smoke do dono (UI de cartões)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`
2. No Painter, crie uma imagem de 3840×2160 com o papel «Textured Paper». Escolha o pincel «Filbert» (Wet Paint) e pinte um traço
   grande. Clique em «+» em Layers e pinte outro traço na camada nova.
3. Clique nas duas linhas de camada, uma e outra, várias vezes, logo depois de pintar (com a tinta ainda molhada).
4. Tem de acontecer: a troca é imediata, a tinta molhada continua com o mesmo brilho e o relevo do papel não pisca.
   Deu errado se: o papel ou a tinta mudam de aspecto ao trocar, ou aparece um retângulo diferente onde a tinta estava.
5. Depois: pinte com «Watercolor Basic» numa camada nova e, logo a seguir, Ctrl+Z. Tem de sumir o traço inteiro, sem deixar a mancha
   molhada da água à volta. Deu errado se: fica um contorno ou um brilho onde o traço estava.

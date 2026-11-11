# Provas de mutação, agente A (4 gates novos da missão 4K)

Corridas por nome do gate (`scripts/cargo-test-narrow.sh <crate> -- <nome>`), cada uma precedida de um verde com pelo
menos 1 teste corrido; âncora conferida 1× antes de mutar; ficheiro restaurado do backup + `touch`; `git diff HEAD --
<ficheiro>` vazio no fim de cada gate. A crate `ph2d-painter-brush` inteira foi corrida (595 verdes). A crate
`ph2d-tool-painter` inteira NÃO deu verde de base (um binário de teste morreu antes do agregado, com a máquina carregada
e a árvore partilhada suja), por isso os gates dela correram por nome, com o filtro a casar >= 1 teste.

Placar: 8 mutações, 7 sangraram, 1 sobreviveu.

| Gate (lei) | Mutação | Resultado |
|---|---|---|
| `spray_parado_nao_acumula` e família `ritmo_tests` (agendamento: densidade não segue os fps) | `tick::pulsos`: `dt.clamp(0.0, DT_TIQUE_MAX_S)` -> `dt.max(0.0)` (quadro preso despeja o atraso) | VERMELHO: só `spray_parado_nao_acumula` |
| idem | `tick::pulsos`: `*acum -= periodo` -> `*acum = 0.0` (perde o resto, densidade segue os fps) | VERMELHO: `aerografo_a_30_e_144_fps…`, `spray_parado_a_30_e_144_fps…`, `o_antes_a_30_qs_e_o_motor_novo…` e o velho `airbrush_tick_emits_by_time` |
| `os_ladrilhos_do_po_sao_a_tela_inteira_ao_bit` (paridade ao bit, ladrilho == tela) | `po_na_janela`: apron `3r` -> `2r` | VERMELHO: pânico de índice em `po_do_papel.rs:362` (sangra, mas por pânico e não pela mensagem do gate) |
| idem | `media_em_janela`: divisor `hi-lo+1` -> `2r+1` (sem renormalizar na borda) | VERMELHO: a cláusula `max <= 1` contra `po_da_tela` (`po_do_papel_tests.rs:255`) |
| `uma_textura_que_nao_se_le_deixa_a_ferramenta_intacta` (perda de dados / estado meio-aplicado) | `macro_estado_aplica`: a verificação das texturas ilegíveis desligada (`if false && …`) | VERMELHO: só este teste (os 9 `macro_estado` correram) |
| `um_estado_que_falha_no_trilho_deixa_a_ferramenta_intacta` | `macro_trilho_le`: o confere do tamanho da máscara sempre aceita (`… \|\| true`) | VERMELHO: só este teste |
| `as_caixas_fundidas_cobrem_a_mesma_uniao_e_nao_dependem_da_ordem` (Ctrl+Z: a caixa fundida cobre a união) | `funde_as_caixas`: remove o `sort_unstable_by_key` (ordem de entrada) | VERMELHO: `a != b` |
| idem | `funde_as_caixas`: `v[i] = union_region(v[i], v[j])` -> `v[i] = v[j]` (a fusão perde a cobertura de `i`) | **SOBREVIVEU** |

## Sobrevivente

`funde_as_caixas`, laço de fusão gulosa. A entrada do gate (30 tiles em diagonal + linha de 4) cai, depois do passo
linear por linha, em exatamente 16 caixas = `MAX_CAIXAS` e nenhum par tem acréscimo de área 0; logo o laço sai em
`d > 0 && v.len() <= MAX_CAIXAS` sem fundir nunca. A asserção do teto (`len() <= MAX_CAIXAS`) passa de graça e a fusão
acima do teto, onde um defeito perderia cobertura (dados do undo fora da janela), não é exercida. Gate em falta: uma
entrada com > 16 caixas após o passo linear (p. ex. 40 tiles em diagonal com saltos) que force fundir, e a asserção de
cobertura de cada tile de entrada.

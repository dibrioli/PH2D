# Plano — desempenho total do Painter (ordem do dono 11–12/11)

«Se esforce para resolver todos os problemas de performance do módulo a todo custo.» Censo dos abertos de desempenho
(BACKLOG.md + BACKLOG_AUDITORIA.md + FAIXAS_SEGURAS.md, 12/11), agrupado por FAMÍLIA (CLAUDE.md §0.10): cada onda
cura a família inteira e mede-se numa matriz calma única. Curas sem mudar pixel (alarmes de pincel: a mesma lista
de vermelhos antes × depois). Do DONO, fora do plano: B14.2 (luz cheia no gesto), B13.3 (sono da água), B43.6 (cgu1).

## Ondas

| onda | família | itens | número de partida | régua |
|---|---|---|---|---|
| 1 | toque e Ctrl+Z 4K | envio sem cópia dos planos da luz · Wet: undo sem memset do fluxo, preparo no evento, `poe` dos tiles · B17.5 | Filbert toque ~76 · Ctrl+Z ~62 · Wet toque ~62 · Wet Ctrl+Z ~45 ms | `mede_pen_up_4k.sh matriz` |
| 2 | cerdas largas | B14.3 · W5 (PA-52) · W9 (PA-56) | Flat Bristle 240: dab 1 037 ms p50 · Bristle 16·4000: soltar 1 340 ms · traço p95 190 ms | `mede_o_dab` + perf · matriz com `KNOBS`/`TECLAS` |
| 3 | mistura de cor | B28.1 · B28.2 · B28.3 · B28.4 · B32.A1 | Wet blend 21,6 ms/q (alvo 16,7) · Wash Mix 24 ms/q | `sonda_custo_no_app`, sondas da `mistura_opaca` |
| 4 | água do Wet | B11.3 · B13.5 · B13.6 · B13.7 (= B13.A2) | 34 → 3,6–5 img/s · 26 → 8 img/s do 4.º traço · ms/passo 4,3 → 35 | matriz (img/s), sondas do passo |
| 5 | camadas e compositor | B19.10 · B18.1–18.3 · B39.A3 · B01.A1 · B20.6 · B20.11 · B40.A3–A5 | 16 UHD ~126 ms [E] · troca 66 MB 7–10 ms [E] · salvar 130–190 ms [E] · máscara 17–28 ms/q [E] | matriz `CAMADA_*` · sondas novas |
| 6 | cadência e caneta | B21.1 (Mailbox) · B27.1 · B22.1/22.4 · B23.2 (resto do laço a pedido) | caneta→fotão −8..−25 ms [E] · INPUT p95 61,7 ms | régua G10 · matriz |
| 7 | memória | B42.3 · B17.9 · B17.6–17.10 | undo 4K 533,8 MB ≈ 21 traços | `mede_o_passo_de_undo_a_4k` |

## Recusas MEDIDAS que não se reconstroem

- `planos_de_tamanho` (crescer os planos por páginas zeradas): 62,2 × 61,7 — sem ganho (REGRESSOES 11/11).
- `write_texture` em paralelo: o wgpu-core 30 copia com `pending_writes.lock()` preso.
- SilMemo para o Flat 240: 1,072 → 0,995 ms (B14.3). Paralelizar o pouso do Drag: é Gauss-Seidel.
- `omp critical` à libmypaint na mistura (B28.3). Fifo + latência 1 a 4K: 30 q/s (B21.1).
- `clear()` + resize nos pools: 17,6 → 47,5 ms (RC-10, B19.12). Quantizar `wall_dt` (C9). `pre_present_notify` (B23.6).
- Handler da caneta que só enfileira: o custo migrou, total igual (B27.1).

## Estado

(cada onda acrescenta aqui o commit, a tabela A×B e o que ficou aberto, com o porquê)

# N10 — família 7: memória (agente F1e, 14/11)

Base: HEAD `75597ea691`. Uma linha por id: estado · antes → depois · sha · porquê. Partida: undo 4K do documento 533,8 MB ≈ 21
traços (o piso; `history_budget_here` já o sobe a RAM/32 desde `4e50376cb8`). RAM desta máquina: 132 315 467 776 B (123,2 GiB).

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B42.3 (undo vivo do Wet) | **feito** | tecto do undo vivo a 3840×2160: 3 203 MB (6 × piso, toda máquina) → **4 135 MB** na do dono (RAM/32); 8/32 GiB/sem leitura ficam no piso (3 203 MB) | este commit | `undo_vivo::teto_da_historia_em(w, h, ram) = max(6 × piso, tecto_pela_ram(ram))`, a mesma fração `1/32` do histórico do documento. Gate `o_teto_do_undo_vivo_deriva_da_ram` (piso exacto 3 203 137 536; None/8/32 GiB = piso; 123 GiB = RAM/32). Fica: `memory_budget` 64 MB (IN-116) — arrasta o agregado HR-13 e o `painter.toml`, item próprio. |
| B17.4 | parcial (cura no HEAD; medição pedida é `--ignored`) | 1.º toque digital 4K no app 15,9 ms (o quadro) | — | ver N10_toque_undo §F1. |
| B17.5 | parcial (sonda por correr) | — | — | comando: `bash scripts/cargo-test-narrow.sh ph2d-tool-painter --profile smoke what_a_diagonal_ctrl_z_copies_at_4k -- --ignored --nocapture --test-threads=1` (fila n.º 8; a saída de 14/11 07:26 em `target/n10/f1/s8.limpo.txt`: undo Digital 3,33 ms/1 cópia, Impasto 10,48 ms/4 cópias, todas de `undo_delta.rs:334`). |
| B17.6 | parcial (Clone feito) | o Clone capturava a tela inteira por lote (`None`, 33 MB a 4K) → a pegada do lote com as cópias do Tiling (`blur_route::lote_bounds`, alcance `tip_reach`) | este commit | gate `o_clone_captura_so_a_regiao_do_lote` (3 rotas: redondo, máscara, grão; todo texel mudado no journal, journal < 1/2 da tela, undo ao byte); mutação «região sem as cópias do Tiling» ⇒ vermelho «clone redondo: 117/1338 texels fora do journal»). Restam 18 `None` (grep `fork_canvas` em `tool/paint/**`): mask.rs, sculpt_session/inflate, watercolor_render, wetpaint/composite (F5d), warp/*, region.rs, fill ×3, selection ×3, inpaint ×2. |
| B17.7 | aberto | — | — | não iniciado (`watercolor_backdrop.rs:110`, `tp/state.rs:357-561`). |
| B17.8 | aberto | — | — | não iniciado (`relief_state.rs` SparePlanes, `composite_acumulado.rs:454-507`). |
| B17.9 / B17.10 | aberto, **condicionais** (o BACKLOG: só se Z10 — 50 traços — mostrar o penhasco) | — | — | Z10 não corrido (load 8–46 toda a janela). Estado da arte quando a régua o pedir: os passos ANTIGOS do Registo do Wet comprimidos (lz4) num fio de fundo, fora do quadro; descomprimir só no Ctrl+Z que os alcança; o tecto continua o da RAM (acima). |
| B42.4 | parcial (inalterado) | — | — | falta: raio 512 ×3 (com B42.5), 4096 e 999, `RESERVE_R_MAX` raio × diâmetro. |
| B42.5 | parcial (inalterado) | — | — | próximo passo no BACKLOG (tecto por motor, em calma). |
| B42.6 | aberto | — | — | pede máquina calma (load < 5) e `PH2D_CPU_PCT=100`; load 8–46 nesta janela. |
| B41.3 | aberto | — | — | as guardas são no carimbo/queda (`stamp_color.rs`, `falloff*.rs`, `walk.rs`): pegada do depósito (F2b). |

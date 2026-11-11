# Provas de mutação B (missão 4K)

Cada gate foi visto VERDE antes (corrida limpa, população 1). Âncora conferida 1x antes de mutar; backup e restauro por `cp` + `touch`; `git diff HEAD` dos 4 ficheiros mutados vazio no fim. Corrida por nome, via `ph2d-run.sh` + `cargo-test-narrow.sh`.

| Gate | Mutação | Resultado |
|---|---|---|
| B22.2 `spray_parado_nao_acumula` / `spray_parado_a_30_e_144_fps_da_a_mesma_densidade` / `aerografo_a_30_e_144_fps_da_a_mesma_densidade` (ph2d-painter-brush, `stroke/tick.rs::pulsos`) | `dt.clamp(0.0, DT_TIQUE_MAX_S)` -> `dt.max(0.0)` (quadro preso não é cortado) | SANGROU (spray_parado_nao_acumula) |
| idem | `*acum -= periodo` -> `*acum = 0.0` (o resto do atraso cai, densidade segue os fps) | SANGROU (spray e aerógrafo a 30 e 144) |
| B49.1 `os_ladrilhos_do_po_sao_a_tela_inteira_ao_bit` (`po_do_papel.rs::po_na_janela_do_alfa`) | apron vertical 3r -> 2r | SANGROU, mas por pânico de índice (po_do_papel.rs:362), não pela comparação de bits |
| idem | apron esquerdo das colunas 3r -> 2r | SANGROU, idem (pânico de índice, linha 362) |
| B25.1 `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` (`stamp_banded.rs::caixas`) | porta do alcance real -> raio cru `dab_write_bounds(.., [1.0, 1.0])` | SANGROU (90 bytes, 1 evento/quadro, 2 fios) |
| idem (`stamp_banded.rs`, rejeição por banda) | `b.y + b.h <= band_y0` -> `<= band_y0 + 1` | SOBREVIVEU: a fixtura (ponta reta 45°, r 36, 512²) não tem dab cuja caixa toque a banda numa só linha |
| idem | `<= band_y0 + 8` (mesma cláusula, mutação maior) | SANGROU (18 bytes a 32 fios) |
| B20.7 `a_textura_e_o_espelho` (ph2d-app-painter, `tool/dreno.rs::publica_o_envio`) | o dreno sobrescreve sempre (`\|\| true`, sem soma por união) | SANGROU |
| idem | `union_region(a, b)` -> `b` | SANGROU |

Placar: 8 mutações das duas por gate (mais 1 extra no B25.1): 7 de 9 sangraram na primeira passagem pedida (8 de 9 contando a extra +8); 1 sobrevivente.

Sobrevivente: B25.1, rejeição por banda com folga de 1 linha. Falta uma fixtura com um dab cuja caixa de escrita toque a banda em exatamente uma linha (o gate só vê erros de 8 linhas). Os dois vermelhos do B49.1 são pânico de índice: o gate protege o apron contra encolher, mas por falha de limite, não por divergência de bit; uma mutação que encolha o apron sem sair do domínio lido (p.ex. mudar só um dos três passos) não foi provada.

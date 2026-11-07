---
name: project_first_stroke_undo_memory_cut_next_round_2026_11_07
description: Dono 07/11 — na próxima rodada, depois do CI, a linha Painter reduz a memória do 1.º traço do Wet Paint (registo do undo só na caixa do traço) e baixa o tecto
metadata:
  type: project
---

Dono 07/11: a linha Painter reduz a memória do 1.º traço do Wet Paint **na próxima rodada de implementações, depois do CI desta integração**.

**Why:** a tinta viva (undo do 1.º traço que não mata a tinta) faz `wet_acorda()` antes de `wet_guarda_o_passo()` em `stroke_lifecycle.rs`, e o registo do undo guarda quase a grade inteira no pen-down: pico do Dabber de 367 para 490 MiB a raio 300, de 1802 para 2071 MiB a raio 1000 (+117 MiB de registo a raio 300). Na integração o tecto subiu para (515, 2175) em `tests/it/presets_de_fabrica/memoria.rs`, com a medição ao lado, só para não travar o ship.

**How to apply:** marcar só a caixa do que o traço escreve; depois baixar `TECTO_DA_MEMORIA_MIB` para o medido novo. Cuidado: um erro no registo faz o undo não restaurar a tinta, então correr `undo_auditoria undo_tinta foto_em_tiles` e medir. Tirar o `marca_tudo()` de `land_direct_remainders` já foi tentado e não muda nada (continua 117 MiB). A causa é leitura de código, sem medição contra a base.

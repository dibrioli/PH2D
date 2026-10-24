# Oráculo — a grade de 32 camadas (report do dono, 24/10)

**O defeito:** a grade do «Cull Mask» (e a irmã «Visibility Layer») era uma caixa de marcar por bit, com o número
como RÓTULO à esquerda; num painel estreito os 32 números sumiam ([`ph2d_antes_report_do_dono.png`](ph2d_antes_report_do_dono.png)),
e num médio uns saíam «…».

**O oráculo** (Godot 4.7.2, MIT, `corre.sh` + `sonda.gd`, editor numa Xvfb com um Sprite2D):
[`godot_grade_zoom.png`](godot_grade_zoom.png) — o `EditorPropertyLayers` põe o número DENTRO de um ladrilho, aceso
(acento) quando a camada está ligada, escuro com o número apagado quando não; células pequenas de tamanho fixo.
(A medição JSON da sonda saiu vazia, e o estreitamento do dock não pegou — a foto chega para a lei.)

**A lei aplicada** (`ph2d-editor-core/src/widget/bitmask_grid32.rs`): ladrilho com o número dentro, `8 × 4`
(metade da altura do `4 × 8` antigo), orçamento do número = o ladrilho menos `Xs` (não o de um rótulo). Gate
`todo_numero_cabe_inteiro_no_seu_ladrilho` (varre a linha útil de 196 a 600 px em ¼ px, os 32 números); mutações
`label_budget` e `COLS = 16` → vermelho. Foto do programa: [`ph2d_depois_cull_mask.png`](ph2d_depois_cull_mask.png)
(tudo ligado; e o 3 desligado pelo clique).

Recusa: o Godot mostra só 10 de cada vez e esconde o resto atrás de uma seta; as nossas 32 cabem em 4 linhas.

# Oráculo: Pixelorama 1.2.3 com pressão de caneta simulada

Pixelorama (MIT, Godot 4) corrido por script, sem caneta e sem ecrã real.

    bash docs/PixelArt/ferramentas/pixelorama_pressao/corre.sh /tmp/<scratch>/pixelorama_run

Mecanismo: extensão `PressaoProbe` (zip em `user://extensions/`, ligada por `[extensions] PressaoProbe=true`
no `config.ini` de um `HOME` isolado). O `Main.gd` cria um projecto 440×80, escolhe o Pencil, mexe em
`Tools.dynamics_size/alpha`, `pen_pressure_min/max`, `brush_size_max_increment` e chama `Tools.handle_draw`
com `InputEventMouseMotion.pressure` sintético (o mesmo caminho do `Canvas._input`). Grava o cel com
`Image.save_png` e sai. `medir.py` (sem dependências) mede cada ponto e o traço.

Variantes: `size_default_thr` (limiares 0,2/0,8 de fábrica), `size_thr_0_1`, `alpha_thr_0_1`.
Linha y=20: 21 pontos isolados (pressão 0..1); linha y=60: traço contínuo, pressão 0→1.

Atenção: pressão crua EXACTAMENTE 1,0 é trocada por extrapolação (contorno do Godot #53033 em
`Tools.gd`) — o 1,0 vira 0,95 nesta sequência.

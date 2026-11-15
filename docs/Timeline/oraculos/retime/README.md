# Oráculos do Retime (15/11) — corridas por script, saída gravada

Plano: [19 — pesquisa do Retime](../../19_pesquisa_retime.md) §5. Entrada NOSSA em todas: um trecho de 0 a 1, alça =
(comprimento `a`, velocidade `s` × a média), ponta de saída em `(a, s·a)`. Nenhum fonte lido.

| Fixture | App (artefacto, licença `pacman -Qi`) | Como se reproduz |
|---|---|---|
| `blender.json` | Blender 5.2.3 (`blender 17:5.2.3-5`, GPL — só a saída) | `blender -b --factory-startup --python blender_retime.py -- blender.json` (com `PH2D_GPU=1 bash scripts/ph2d-run.sh`) |
| `godot.json` | Godot 4.7.2 (`godot 4.7.2-2.1`, MIT) | `godot --headless --script godot_bezier.gd -- godot.json` (idem) |
| `natron.json` | Natron 2.5.0 (`natron-bin 2.5.0-1`, GPL2 — só a saída) | 100 quadros PGM com o n.º do quadro no pixel; `MODE=rampa\|volta NatronRenderer -t natron_retime.py`; o quadro de entrada lê-se do canal alfa da saída PNG |

- `mapa` = onde o movimento está (0..1) em 201 amostras do tempo do trecho; `monotono` = nunca volta.
- `blender_foto.py` + `sessao_blender.sh`: foto do Graph Editor dentro de `kwin_wayland --virtual --xwayland`
  (`BLENDER_USER_CONFIG` isolado sem splash; a foto é o `screen.screenshot` do próprio Blender — nunca o ecrã do dono).
- `folha.py` desenha [a folha lado a lado](../../fotos/retime_8_pesquisa_lado_a_lado.png) destes JSON.
- OpenToonz 1.8 (BSD-3) **não corrido**: as curvas do Function Editor não têm via de script sem interface; o modelo
  dele é o mesmo do Blender (alças de valor por chave) e não acrescenta caso novo.

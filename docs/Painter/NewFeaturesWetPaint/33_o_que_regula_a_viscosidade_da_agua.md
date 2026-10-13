# 33 — O que regula a «viscosidade» da água no Wet Paint, além do Water (13/10)

> Pergunta do dono (13/10). Fórmulas conferidas no código. ⭐ **Leveling, Level Clamp, Capillary e Brake saíram
> do Tuning para o cartão Wet Paint do painel do Painter** (ordem do dono, 13/10: ids `PAINTER_WETPAINT_{LEVELING,
> LEVEL_CLAMP, CAPILLARY, BRAKE}`, faixa e passo da `KNOB_DEFS`; portão
> `a_fisica_da_agua_do_cartao_chega_ao_motor_e_muda_a_tinta`); o resto continua no Tuning.

| Tuning (chave) | grupo · faixa · fábrica | o que faz (fórmula) |
|---|---|---|
| **Viscosity** (`viscosity`) | Physics · 0–2 · 0,2 | ⚠️ é um LIMIAR de profundidade, de sentido INVERTIDO: onde o filme é mais fundo que ele, a velocidade vira a média com os 4 vizinhos (`0,2·v + 0,2·Σ4`, `solver/flow_jacobi.rs:310`) — a água move-se como uma massa coesa. **Subir = menos células coesas = água MAIS fluida.** |
| **Brake** (`brake`) | Physics · 0–4 · 1,5 | o freio real: a velocidade é multiplicada por `clamp(filme + 3/255·humidade − Brake, 0,05, 1)` no ponto à frente (`flow_jacobi.rs:366`) — sobre papel seco/raso a água trava. **Subir = mais grossa.** |
| **Brake reach** (`brakeReach`) | Physics · 1–12 · 4 | até onde o freio olha à frente (`flow_jacobi.rs:358`). |
| **Max velocity** (`maxVelocity`) | Physics · 0,2–3 · 1,0 | o tecto da velocidade (`solver/advect.rs:133`). Baixar = escorre devagar. |
| **Leveling** · **Level clamp** | Physics · 0–2 · 0,5 · 0–1 · 0,2 | o quanto a água se NIVELA (espalha do fundo para o raso) e o limite desse empurrão (`solver.rs:270`). |
| **Capillary** · **Capillary gate** | Physics · 0–1 · 0,2 · 100–6000 · 2000 | o empurrão pelo dente do papel, só onde há pouco pigmento (`solver.rs:289`). |
| **Projection** | Physics · 0–1,5 · 0,7 | a força da pressão que mantém o escoamento sem divergência (`solver/project.rs:172`). |
| **Gravity** | Physics · 0–0,05 · 0,005 | o escorrer, só com o Tilt ligado (`sim.rs:255`). |
| **Retention** · **Evaporation** · **Base evaporation** · **Edge darkening** | Water | quanto filme fica por passo e quanto seca (`drying.rs:174`) — água que seca depressa «engrossa» o traço. |
| **Water per dab** · **Water cap** | Water · 0–4 · 1 · 1–30 · 8 | quanta água cada carimbo põe e o tecto dela (`tools.rs:187`). |

Fora do Tuning: o **Water** do cartão (`× 0,35` → a água do carimbo, `painter/doors.rs:27`), o **Live Water** (a água
corre enquanto se pinta) e o **Tilt**. ⛔ **Drag** e **Pickup** (grupo Paint) são do PINCEL sobre a tinta, não da água.

**Em resumo:** para água mais grossa — **Brake ↑**, **Max velocity ↓**, **Leveling ↓**, **Viscosity ↓** (sim, para
baixo); para mais fluida, o contrário. O nome «Viscosity» engana: um rótulo honesto seria «Thin water above»
— decisão do dono (renomear muda só o texto, não o traço).

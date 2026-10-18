# 47 — PROMPT: a W6 — a pressão e a inclinação do PINCEL, agora com a caneta de verdade (17/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo; `bash scripts/btrfs-health.sh`. Leia SÓ: este prompt ·
o [05](05_plano_da_linha_e_o_antes.md) §3 (a linha da **W6** e a barra dela) e §4.2 (sem rotação de barril) · o
[handoff P46](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P46_CANETA.md) §2–§3 e §6 · o
[ADR-0189](../../architecture/decisions/0189-a-caneta-chega-pelo-tablet-v2-ao-lado-do-winit.md) §Decisão ·
`crates/ph2d-painter-brush/src/dynamics.rs` · o [45_medicao](45_medicao_o_esmaecer.md) §3 (a curva da opacidade por
pincel). Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada), R8 (sondas em
`--profile smoke`), §0.9 (o Rebelle corre-se, nunca se lê — o teste dele acaba ~06/11).

## De onde vimos

Smoke do dono 17/10: **OK** em tudo — a Wacom conduz o app inteiro com pressão (P46), salvar com a Shape Color, o
Airbrush e o Randomize Color com a Shape Color (P45). Os blocos A–C do [18 §5](18_mapa_das_midias.md) estão feitos;
voltamos ao plano da linha (05 §3). A **W6** era a onda que só fazia sentido com pressão real: agora ela existe, com o
oráculo dos 180 s do dono (`../ferramentas/caneta/medicoes_2026-10-17/`) e o `PH2D_PEN_LOG` da sessão ao vivo.

## Pedido — o produto final, não um MVP

1. **Medir primeiro, um lote só (§0.10).** A barra da W6 (05 §3): a rampa de pressão — **pontas ≥ 60 % do miolo**
   (o Rebelle 35/42/33; a nossa medida antiga 9/46/4) — e o **lápis 8B a 55°: largura ≥ 1,2× e textura menor** (dele
   37/30, 5,2 contra 9,1). Re-meça a rampa HOJE (a pressão do pincel do P44/P45 já mudou o número). No mesmo lote:
   a **C31** — o KDE aplica a curva de pressão dele ANTES do tablet-v2? Uma sessão com o dono: `sudo libinput
   debug-events` + `PH2D_PEN_LOG` em paralelo, a mesma pincelada; compare evento a evento. Só depois decida se o PH2D
   precisa de piso/tecto/curva próprios ou se a do KDE basta (não duplicar).
2. **A pressão do pincel completa**: piso e tecto da pressão (o que a mão dá → o que o pincel lê), a suavização da
   pressão ao longo do traço, e o «Pulled String» — cada um knob POR NOME no preset, omissão = a lei de hoje ao byte
   (a identidade das 22 cenas de `a_lei_da_tinta` e as miniaturas de fábrica). Uma curva por pincel já existe para a
   opacidade (`pressureOpacityCurve`, P45): a do tamanho segue a MESMA lei, nunca uma segunda.
3. **A inclinação nos dois eixos**: o tilt já chega ao `CanvasPointer` (`[-1, 1]`, P46) e nenhuma lei o lê. A rotação
   do carimbo `{None, FollowTrajectory, PenTilt}` (sem barril, dono 08/10) e o achatamento pela inclinação (o 8B a 55°
   alarga e perde textura). Nos 5 meios onde a lei se aplique; o rato (tilt 0) ao byte.
4. **Gates**: as barras da W6 como portões (vermelhos antes); o oráculo dos 180 s para a suavização (a sequência real
   de pressão entra, a lida sai sem degraus nem atraso acima do que se mede); prova de mutação nas leis que importam.
5. **Fecho**: suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; o 05 §3 com a W6 marcada; smoke ao dono em
   passos numerados, com a caneta (leve × forte e inclinada, em cada meio; o lápis 8B deitado).

## Na fila (do dono, 17/10 — NÃO junte a esta onda)

Lista [26](26_ABERTOS.md): **A13** apagar um pincel do utilizador · **A14** repor pincéis (um, ou todos, ao de fábrica)
· **A15** a prévia do pincel em TODOS os modos (hoje só no Wet Paint). E da caneta: C29 (pad), C30 (dedo), C32
(X11/Windows). O resto do plano depois da W6: a W2 (textura/CV/período), W3–W5, W7, W8 (05 §3).

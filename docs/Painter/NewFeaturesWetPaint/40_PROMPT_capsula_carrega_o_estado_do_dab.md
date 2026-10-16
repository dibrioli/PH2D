# 40 — PROMPT: a cápsula (Continuous) carrega o ESTADO inteiro do dab, não só a posição (15/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo. Leia SÓ: este prompt · lista [26](26_ABERTOS.md) A11
· [BUGS #55](../BUGS_painter_fechados_41_a_46.md) · §8–§9 do [handoff P39](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P39_W5.md).
Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada).

## De onde vimos — a auditoria de 2 lentes (15/10, depois do 3.º smoke do dono)

O dono: *«Impasto ainda não está correto. Auditoria com outro agente»* ([foto](39_smoke_dono_3_impasto_continuous.png)).
- **Lente 2 (costura):** o Continuous está íntegro do clique ao traço, em todas as ordens de passos (Δ 0,00000 contra o
  campo posto à mão). Sondas `tests/auditoria_l2_seam.rs`, `ph2d-panel-painter-layers/tests/it/auditoria_l2_continuous.rs`.
- **Lente 1 (correção):** a foto reproduz-se com o traço de CIMA **com** Continuous e o de BAIXO **sem** (Spacing ≈ 0,25
  nos dois; o binário do smoke tinha a cura). Sonda `tests/auditoria_l1_impasto.rs`. ⇒ **perguntado ao dono** o que
  estava ligado em cada traço, e o Follow dele; se o de baixo tinha a caixa ligada, a causa está no pincel dele (peça-o
  guardado como preset e corra as sondas sobre ele ANTES de curar).

## A família achada (os dois auditores, independentes) — a lei: cada sub-dab é o dab INTERPOLADO

A cápsula de hoje usa o raio, a pegada/rotação, a pressão e o `arc_len` do dab da FRENTE em todos os sub-dabs e só
desloca o centro pela corda; o traço denso interpola tudo. Medido (Δ tela acesa, Continuous 0,25 × denso 0,025):

| # | caso | com Continuous | sem | onde |
|---|---|---:|---:|---|
| 1 | **Follow = Flow** (reto 0,103 / curva 0,113–0,135) — PIORA | 0,135 | 0,0055 | `texture.rs:450-468` (`arc_len` da frente); `SubPasso::silhueta`, `silhouette_capsula_com`, `CorCapsula` |
| 2 | Rake numa curva | 0,043 | — | `height_walk.rs` `wk.dab.footprint` (rotação da frente) |
| 3 | raio por pressão (caneta; latente no desktop, pressão 1,0) | 0,036–0,129 | 0,156–0,230 | `Dab::capsula` (`stroke.rs:80`), `dab.rs` `silhouette_capsula_com`, `SubPasso` (`wk.radius`) |
| 4 | **Speed liga a cápsula SEM a caixa** (indício: 0,046 × 0,142) | — | — | `stroke/speed.rs:189,228` grava `step` sem `lays_capsules()` |
| 5 | Airbrush muda com a caixa (o Spacing não muda o Airbrush) | 0,118 | 0 | `spec/queries.rs:342-346` (`allows_jitter`) — pergunta de produto |
| 6 | Smear: `HeightDab` sem cápsula (`smear_warp.rs:423`) | por medir | | |

## Pedido

1. Confirme o #4 lendo `dab.step` (Speed, caixa desligada ⇒ todo `step == [0,0]`); se confirmado, cure-o na porta.
2. UMA lei para 1–3 (e 6 se o Smear oferece Continuous): o sub-dab `k` leva `lerp(anterior, frente, t_k)` de centro,
   raio, rotação/pegada, pressão e `arc_len` — nos TRÊS leitores (cobertura `silhouette_capsula_com`, cor `CorCapsula`,
   relevo `SubPasso`). Sem Continuous, ao byte.
3. Portão vermelho primeiro: estender `capsula.rs` — Impasto e Digital × {reto, curva} × {Follow Off, Rake, Flow} ×
   {pressão 1, pressão variável}, tela ACESA no Impasto; barra pela medida; mutação por leitor.
4. #5: decida pela lei do Rebelle/medida (o Airbrush não tem passo ⇒ a caixa deve esmaecer nele?).
5. Fecho: suíte afetada + clippy; handoff ≤ 8 KB; smoke ao dono com a prancha (as linhas da tabela acima, antes/depois).

# J13 — o FPS do Real Oil: medido no app, a luz do óleo refeita a cada passo (janela 13, 02/11)

Ordem: o §3.1 do [handoff da janela 12](HANDOFF_02_11_janela12.md) e o [J11](J11_agua_no_oleo.md) §4. Base `8a3c88cbc6`.

## 1. A cena do FPS no app (`6f3dac38c7`)

`PH2D_WETPAINT_SMOKE=3` ([`wetpaint_fps.rs`](../../../crates/ph2d-app-painter/src/wetpaint_fps.rs)): o app pinta SOZINHO,
ao relógio de parede, pela ponte do Painter a cada quadro — seis rodadas de um traço do Dabber (Size 30, 2,5 s a 250
eventos/s), 2 s parado, o Wet Drip pela mesma linha, 5 s parado; as faixas sobem de baixo para cima. O período é o tempo
entre dois quadros seguidos. Real Oil por PROCESSO (`PH2D_FPS_OLEO=0|1`): a grade que já viu óleo fica com a óptica dele,
por isso duas variantes no mesmo processo contaminam-se (a 1.ª corrida intercalada mediu isso). Tela `PH2D_FPS_TELA`.

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_GPU=1 \
  VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json PH2D_WETPAINT_SMOKE=3 PH2D_FPS_OLEO=1 \
  PH2D_PAINT_PERF=1 PH2D_FLUID_PROFILE=1 timeout 110 bash scripts/ph2d-run.sh target/smoke/ph2d-host-desktop
```

Período p50 (ms) por rodada, 1024², Radeon, `--profile smoke`:

| fase | sem Real Oil (2 processos) | com Real Oil |
|---|---|---|
| traço do Dabber | 16,1–16,7 | 15,9–16,6 |
| parado (seca) | 15,8–17,8 | 17,0 · 18,6 · 31,4 · 48,8 · 60,3 · **72,2** |
| traço do Wet Drip | 16,3–16,7 | 15,6–16,5 |
| parado (a água corre) | 15,7–17,4 | 17,5 · 32,2 · 48,3 · 57,0 · 71,0 · **79,0** |

O perfil do quadro (`PH2D_FLUID_PROFILE`) nas fases paradas: `composite media 42–60 ms` (pico 67–83) sobre 0,23–0,29 M
células de poça, no fio principal (o tique do Painter); a água cai de 40 para 15–27 passos/s porque o worker espera o
motor (`away 71–86 %`). Durante o traço o composite é de 14–20 mil células (2,5–4,3 ms). O custo NÃO é do traço nem da
simulação (5 ms/passo, noutra thread): é a óptica do óleo sobre a caixa suja inteira a cada passo.

## 2. O que o composite paga e o que muda (`7718d06194`)

Sonda `diag_j13_fps_real_oil_o_que_muda` ([`fps_j13_tests.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/fps_j13_tests.rs)),
a mesma cena sem tela. Fase «corre», média por composição:

| rodada | campo | pigmento | sombra+grão | píxel | total ms | células da janela | campo ≠ ao bit | refazer (dilatado) | pigmento ≠ |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 4,05 | 7,69 | 4,24 | 0,24 | 16,2 | 127 697 | 34,7 % | 69,0 % | 13,5 % |
| 2 | 9,40 | 21,45 | 11,55 | 0,65 | 43,1 | 384 374 | 35,7 % | 69,5 % | 7,2 % |
| 5 | 12,13 | 27,96 | 14,78 | 0,75 | 55,6 | 554 695 | 27,6 % | 59,7 % | 5,0 % |

- O campo NUNCA vem do `CacheCampo`: a chave inclui a revisão do motor, que anda a cada passo.
- perf (binário com símbolos, `target/perfsym`): `oleo_horizonte::visibilidade_e_ceu` 19 % · `oleo_sombra::rebote_no_papel`
  12,6 % · `oleo_luz::luz_directa` 9,7 % · `oleo_ambiente::Sectores::ate` 7,5 % · libm cos/asin/atan ~10 %. Essas funções
  leem só a altura do campo (até 24 células em 8 raios; a luz directa 24 passos para a luz), o albedo e a Transparency
  do campo (o rebote) e parâmetros fixos do cartão.
- A água sobre o óleo mexe o campo ao bit em ~30 % das células com variações minúsculas; dilatadas pelo alcance da luz
  obrigam a refazer 60–70 % — uma cache EXACTA poupa ~35 %. A saída muda em ~5 %.

## 3. A cache da luz por célula — construída, provada e RECUSADA (ramo `exp/j13-luz-guardada`, `da1728780e`)

`oleo_luz_guardada.rs`: por célula a `LuzCeu` (luz directa, visibilidade, céu) e o rebote, com a referência das entradas
(altura fina, altura da sombra, albedo, Transparency); deriva acumulada, dilatada por 25 células (somas prefixas),
recalcula onde passa ε. `ε = 0` ao byte com o de hoje (`a_luz_guardada_em_epsilon_zero_e_a_tela_de_sempre_ao_byte`,
96 quadros + troca de azimute; mutação: dilatação a raio 2 → vermelho). Varredura (1024², 2 rodadas, intercalada no
mesmo processo, fase «corre»):

| variante | ms/quadro | refeitas | reusadas | máx \|Δ\| u8 | píxeis ≠ |
|---|---|---|---|---|---|
| sem cache | **19,58** | — | — | 0 | 0 |
| ε = 0 | 24,25 | 142 569 | 4 703 | 0 | 0 |
| ε = 1e-3 | 23,06 | 121 812 | 25 459 | 0 | 0 |
| ε = 1e-1 (o maior com \|Δ\| ≤ 1) | 22,38 | 102 158 | 45 114 | 1 | 49 |
| ε = 3e-1 | 21,97 | 90 846 | 56 426 | 8 | 365 |

A água sobre o óleo deriva as alturas mais de 0,1 em quase toda a caixa e o alcance de 25 espalha-o: a cache refaz
≥ 69 % e soma a deriva e a dilatação — mais lenta em todo ε.

## 4. Núcleos

App com Real Oil, fase parada (composite média nas rodadas pesadas): 16 núcleos (fatia da linha) ~55–60 ms · 32
núcleos (`PH2D_CPU_PCT=100`) ~46–49 ms; quadro 79,0 → 72,6 ms na rodada 6. Os núcleos a mais quase não pagam (load
da máquina 9–16 nessas corridas: a régua de escalonamento não decide).

## 5. ⛔ Recusas MEDIDAS

| tentativa | o que deu |
|---|---|
| Real Oil × sem Real Oil intercalados no MESMO processo | contaminado: a grade fica com a óptica do óleo depois do 1.º Dabber (17,6 × 16,5 ms na rodada 0, iguais depois) |
| resumo pelo MÍNIMO das rodadas | esconde a acumulação (as rodadas não são réplicas: a água das anteriores corre) |
| cache exacta (refazer onde o campo mudou ao bit, dilatado) | só ~35 % da óptica: 60–70 % da janela tem de ser refeita |
| cache da luz com limiar ε (§3) | mais lenta que sem cache em todo ε (24,3 → 22,0 contra 19,6 ms/quadro) |
| a sonda com os ganchos de comparação para medir escalonamento | os ganchos correm em série DENTRO dos cronómetros: os ms da 2.ª versão da sonda não valem |

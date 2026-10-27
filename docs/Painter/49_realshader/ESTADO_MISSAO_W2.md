# ESTADO — Missão W2 do Óleo Real (autónoma, 27/10)

> Retoma-se DAQUI. Prompt: [`PROMPT_MISSAO_W2_material_ambiente_pinceis.md`](PROMPT_MISSAO_W2_material_ambiente_pinceis.md).
> Base: `3da5b294c1` (W1c aprovada, tag `backup/oleo-real-W1c-2026-10-27`).

## Decisões da janela

- E1 e o motor da faca (CAP-01/C38, primeiro item da E4) correm em PARALELO: ficheiros disjuntos e a faca não
  depende do material (só o Metallic depende da E3). Os VALORES das 18 presets continuam depois de E1–E3.

- E2: integral da irradiância POR SECTOR de fatia (somas acumuladas por azimute de L·cosθ e L·senθ ao longo da
  elevação → cos(θ−γ) da normal projectada em forma fechada); bent normal + cone recusado sem medir porque perde a
  assimetria da fenda. Espera a E1 (mesmos ficheiros: cartão e `oleo_real.rs`).
- ⛔ **Ordem do dono 27/10: quando TUDO acabar (fecho incluído, mensagem final escrita), desligar o PC:
  `systemctl poweroff`** (polkit `CanPowerOff=yes`, sem senha). Antes: confirmar que nenhum `ph2d-run`/agente desta
  linha ainda corre.

## Etapas

| etapa | estado | commits | números | o que ficou |
|---|---|---|---|---|
| E1 fenda força + alcance | ✅ | `dd80f3b66c` | lei `V/(V+f(1−V))` (V^f recusado: pretos já em f=2); força 0..3 (f=3: min max 78, 0 % pretos; f=4 encosta em 70); alcance 1..24 (tecto = margem 25 da recomposição) | «dramático» moderado (fenda 49→43); mais pede lei com piso pelo tom |
| E4-motor lâmina que gira | ✅ | `d86661c702` | rotação já existia (lei medida `06_LEIS_DAS_LACUNAS` §1: fixa ou segue); rebordo NOVO = rampa nos 12 % de fora da meia-largura (F42); campo `knifeRidge` px 0..16 (satura ~8), 0 = bit-idêntico | rebordo fraco 0,72/1,19 × Rebelle 0,40/1,39 = sombreado do relevo fino no óleo real (fila pós-E2); linha UI `knifeRidge` por fazer (depois da E2, ficheiros do painel); leque no canto do L; crista no início/fim. Vermelho pré-existente `sem_glaze_cada_preset_de_fabrica` (Wet Wash + 2 Wet Liner) — não é desta onda. Detalhe: `E4_lamina.md` |
| E4-fotos oráculo Rebelle | ✅ | `ba7d5d52cb` | os 18 já tinham 6 PNG cada (19–20/10); só o índice `INDICE_W2.md` | — |
| E2 HDRI meia-esfera | ✅ | `5711a69355` | difuso HDRI = `força·(g·V + mr(V) − V) + lâmpada`, g por sector de fatia (tabelas acumuladas, arco FÍSICO — o arco Jimenez via a janela pela parede: fenda 1,08 × 0,53); `.amb` → PH2DAMB3 (radiância bruta, lê v2); «Environment light» 0,6 (Clássico bit-idêntico). Correlação tinta (antes→E2): estúdio 0,556→0,772 · interior 0,777→0,833 · pátio 0,770→0,897; papel 0,31→0,68 · 0,77→0,83 · 0,67→0,82. Custo HDRI 1 870→2 137 ns (+14 %). ⚠️ oráculo tinha o HDRI rodado 180° (corrigido; números HDRI da W1c foram sobre o HDRI rodado) | gate E1 no Estúdio passou de «0 pixels < 70» a «≤ 0,5 %» (Cycles tem 0,30 %: o lado de costas para a janela é escuro de verdade); o Clássico continua a 0. Aberto: difuso Clássico 0,71 × 0,83; fenda a força 1 mais clara (0,78 × 0,67); brilho Estúdio p99 0,26 × 0,48 |
| E3 contrato | ✅ | `5464db050b` | `material.rs` `MaterialCelula` (NEUTRO = hoje, mistura linear pela massa) | A = canal no grid + pincel + painel Wet Paint (+ `knifeRidge` UI, Lock material); B = sombreado Principled + cartão Real Oil + régua Cycles (+ brilho Estúdio, contraste do rebordo). Contrato entre os dois: `Grid::material_da_celula(i)` |
| E3-A canal + pincel | ✅ | `476e24e7c3` | `Grid::material` (vazio até 1.º metal), mistura pela massa; 10 knobs (`KNOB_COUNT` 109→119, grupo Material); viaja: snapshot, patch/undo, clear, transfer_paint, ponta suja, transfer_blend, smear (não na advecção, como relevo/vela); schema NÃO muda (grade não vai ao .ph2dproj; preset por nome); `knifeRidge` visível no Stroke; `tuning/defs.rs` 702→299 linhas | ⛔ FECHO: catracas vermelhas em `ph2d-panel-registry-init`: `a_carga_de_comandos_de_um_painel_so_encolhe` (wet_tuning 26 × 16; já subira com os grupos avançados de 25/10 — medir na base `3da5b294c1`) e `a_altura_de_abertura_de_um_painel_so_encolhe` (painter_layers 1618 × 1570) — cura por CORTE |
| E3-B sombreado | ✅ | `93ba28db1a` `4310db1bc1` `3291258557` `a6796a7264` | lóbulos × Cycles (README §E3-B, `regua_E3.txt`); metal liso R 0,040 × 0,050; clearcoat 1,17 × 1,11; sheen Δ 0,059 × 0,065; aniso 0,50 × 0,45; brilho Estúdio CURADO (alargamento pela curvatura contado 2×): p99 0,53 × 0,48, área 2,12 % × 2,13 %; custo neutro = hoje, material cheio +25 %; ligado ao grid pelo `CampoAltura`; cartão +12 linhas + chip Reflect map (max p/ neutro 0, produto p/ resto) | metal nas encostas fora (p90 0,60 × 1,16) após 2 rondas; rebordo da faca (5b) → agente Knife da E4; rótulos E1/E2 encurtados (Crevice, Crevice size, Ambient), gate dos nomes na coluna verde; ⚠️ cartão maior agrava a catraca de altura de painter_layers |
| E4 18 presets | em curso (Metallic ∥ Dabber ∥ Knife) | | | |
| Fecho | por fazer | | | |

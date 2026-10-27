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
| E2 HDRI meia-esfera | em curso | | | |
| E3 material Principled | por fazer | | | |
| E4 18 presets | por fazer | | | |
| Fecho | por fazer | | | |

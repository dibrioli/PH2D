# HANDOFF — refatoração do Painter, janela 9 (01/11)

Ramo `line/PainterWatercolor`, de `803ba5cf6a` a este commit. Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela8.md](HANDOFF_01_11_janela8.md). Detalhe e números: [J9_meio_no_liquido.md](J9_meio_no_liquido.md).

## 1. Feito

| commit | o quê |
|---|---|
| `73ce4379d6` | **o meio anda com o líquido** (`wp/solver/meio_transporte.rs`): queda do Wet Flat 4 862 → 80 046; régua da queda pela Dilution (0,2 · 0,6 · preset) verde; 3 testes novos com mutação |
| `2d7e44b483` | a «água que sobe a ladeira» era a PEGADA do pincel (o Size é o raio); cena com o pé a `folga` px, tectos a 10 % do controlo sem ladeira — verde |
| `364e5e1457` | a composição só paga a óptica do Óleo Real com tinta de Real Oil (`Grid::oleo_real`): Wet Drip Oil d0,9 7,3 s → 0,5 s em 120 s; prova ao byte + controlo + mutação |
| este | docs J9, foto `fotos/j9/contorna_j8_j9.png`, handoff e prompt da janela 10 |

⛔ Recusado por medição (código removido): o DIQUE no destino (−25 % de travessia, empoça em cima do óleo, queda da
crista −71 %).

## 2. Smoke do dono

App compilado `--profile smoke` no fim da janela (o que mudou no produto: o óleo diluído escorre com o seu meio — mais
lento que a água, mais fluido que antes; a água sobre óleo seco já não «cola» em cima dele, passa e pinga — o
contornar continua aberto; o óleo diluído sem Real Oil pinta muito mais leve). Resultado: **por registar** (se o dono
reportar, anotar aqui).

## 3. Aberto (por ordem)

1. **O escorrer contorna o relevo** (observação 1 do dono): hoje a ladeira é um NIVELAMENTO clampado a `LevelClamp`
   por passo contra a gravidade que acumula na velocidade. Medir primeiro (força da ladeira × gravidade na orla do
   Dabber, por passo) e depois a lei candidata: o declive do chão como FORÇA no momento. Réguas: `escorre_contorna_tests`
   (abaixo → 0 com o Dabber seco), `a_queda_cresce_com_a_diluicao`, `a_agua_nao_sobe_a_ladeira_do_oleo`, `sonda_oleo_derrete`.
2. **A composição do óleo com Real Oil** (Filbert Oil d0,9: campo 5,8 s · pigmento 10,2 s · sombra 4,4 s em 120 s,
   26 000 células por tique): não refazer campo/sombra a cada tique enquanto o filme só assenta.
3. **9 vermelhos antigos** achados nesta janela (J9 §4): `medida_oleo_real_fenda` ×3, `transparencia` ×4,
   `sonda_oleo_furos`, `sonda_oleo_grumos` — bissectar.
4. Do §4 da janela 7: E9 ⚑ furos; chip de Dilution; `assenta.rs`/`trail/transfer.rs` em 700 linhas; E10c; E11 GPU
   (lê o meio E leva-o com o líquido); E12; E13; outros 3 modos; ajuda ilustrada; sondas para a crate dev-only; crateras.

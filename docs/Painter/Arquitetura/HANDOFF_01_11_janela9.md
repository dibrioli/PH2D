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

**Smoke do dono (01/11): OK** — o óleo diluído escorre como pedido. Fotos [`fotos/j9/smoke_dono_1..5.png`](fotos/j9/)
(Dabber azul seco, Wet Drip amarelo por cima). Observações, todas para a fila:

- **FPS 70 → 30 só com um traço de Dabber** (óleo com Real Oil): é o §3.2 abaixo (composição do Real Oil a cada tique).
- **O Wet Drip faz uma ONDA que desloca o óleo do Dabber** (fotos 2 e 4: o amarelo empurra a orla azul e deixa uma
  faixa amarela dura na borda) — pouco realista.
- **Rastro PIXELADO de baixa qualidade** (fotos 1, 3, 5: o verde da mistura em pontos/riscas sobre o óleo).
- ⭐ **A lei do dono para o relevo** (memória `project_watery_paint_against_relief_owner_law_2026_11_01`): relevo alto e
  tinta concentrada ⇒ a aguada MISTURA-SE com a borda; havendo por onde, escorre pelas LATERAIS; não havendo, EMPOÇA
  na borda misturando-se com a tinta dela até ter volume para passar POR CIMA da tinta grossa.

## 3. Aberto (por ordem)

1. **O escorrer contorna o relevo pela lei do dono** (§2: mistura na borda → laterais → empoça → transborda com volume), junto com a ONDA que desloca o Dabber e o rastro pixelado (mesma família, um bloco): hoje a ladeira é um NIVELAMENTO clampado a `LevelClamp`
   por passo contra a gravidade que acumula na velocidade. Medir primeiro (força da ladeira × gravidade na orla do
   Dabber, por passo) e depois a lei candidata: o declive do chão como FORÇA no momento. Réguas: `escorre_contorna_tests`
   (abaixo → 0 com o Dabber seco), `a_queda_cresce_com_a_diluicao`, `a_agua_nao_sobe_a_ladeira_do_oleo`, `sonda_oleo_derrete`.
2. **A composição do óleo com Real Oil** (o FPS 70 → 30 do smoke) (Filbert Oil d0,9: campo 5,8 s · pigmento 10,2 s · sombra 4,4 s em 120 s,
   26 000 células por tique): não refazer campo/sombra a cada tique enquanto o filme só assenta.
   Proposta do dono (01/11): relevo a 24×/s e luz/sombra a 12×/s. Estimativa sobre a sonda (40 composições/s, por
   tique campo 1,2 · pigmento 2,1 · sombra 0,9 ms): só os dois ritmos poupam ~25 % (o pigmento, a maior fatia, lê a luz
   por píxel e fica a 40×/s). Desenho a medir: refazer só as células cujo relevo MUDOU além de um limiar, e o ritmo
   baixo só onde a tinta apenas assenta (sob o pincel e na água que corre, ritmo cheio). A simulação não muda: a
   composição só desenha. Medir no app (tela real, FPS) antes e depois.
3. **9 vermelhos antigos** achados nesta janela (J9 §4): `medida_oleo_real_fenda` ×3, `transparencia` ×4,
   `sonda_oleo_furos`, `sonda_oleo_grumos` — bissectar.
4. Do §4 da janela 7: E9 ⚑ furos; chip de Dilution; `assenta.rs`/`trail/transfer.rs` em 700 linhas; E10c; E11 GPU
   (lê o meio E leva-o com o líquido); E12; E13; outros 3 modos; ajuda ilustrada; sondas para a crate dev-only; crateras.

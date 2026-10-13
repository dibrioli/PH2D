# 31 — O Spacing por meio: por que o Digital fica pontilhado e o Wet Paint não (13/10)

> Relato do dono (13/10): *«Spacing em wet paint produz traços mais uniformes que em digital (que fica
> pontilhado). descubra porque e como melhorar os outros modos sem perder performance»*.
> Sondas: `wetpaint/sonda_shape_casa.rs::{sonda_spacing_por_meio, sonda_pontilhado_das_cerdas}` (`#[ignore]`,
> `--profile smoke`; `PH2D_SONDA_OUT=<pasta>` grava as telas).

## §1 — Medido

**A ponta redonda, raio 10, traço recto de 220 px, o perfil da tinta no EIXO** (ondulação `(máx−mín)/máx` ·
fração de vales abaixo de ½):

| Spacing | Digital | Impasto | Watercolor | Wet Paint (dabs no motor) |
|---:|---|---|---|---|
| 0,05 | 0 · 0 | 0 · 0 | 0,15 · 0 | 0,39 · 0 (65) |
| 0,25 | 0,02 · 0 | 0 · 0 | 0,15 · 0 | 1 · 0,38 (13) |
| 0,5 | 0,25 · 0 | 0 · 0 | 0,15 · 0 | 1 · 0,75 (7) |
| 1 (o tecto) | 1 · 0,50 | 1 · 0,10 | 1 · 0,50 | 1 · 0,83 (4) |

**As cerdas** (o tapete do Wet Round), raio 20 — o coeficiente de variação da tinta AO LONGO das linhas do
miolo (riscas contínuas ≈ 0; filas de pontos ≫ 0). A prancha: [`31_prancha_pontilhado_por_spacing.png`](31_prancha_pontilhado_por_spacing.png)
(linhas: Digital redondo · Digital Bristles · Wet Paint Bristles; colunas: Spacing 0,025 · 0,1 · 0,25 · 0,5).

| Spacing | Digital redondo | Digital Bristles | Wet Paint Bristles (dabs) |
|---:|---:|---:|---:|
| 0,025 | 0 | 0,014 | 0,043 (66) |
| 0,1 | 0 | **0,206** | 0,185 (17) |
| 0,25 | 0,005 | **0,513** | 0,587 (7) |
| 0,5 | 0,097 | 0,763 | — (4, quase vazio) |

## §2 — O porquê

1. **O pontilhado do Digital com cerdas é o MESMO tapete repetido a cada dab.** As cerdas estão presas ao centro do
   dab (a lei do motor, em píxeis): cada dab carimba a mesma constelação de pontos deslocada de `spacing·2r`. Com o
   passo maior que um pelo (~1–3 px), cada pelo vira uma FILA de pontos com o período do passo — o olho lê o padrão
   repetido (linha 2 da prancha, a partir de 0,1). Na redonda é a «conta» clássica do carimbo (0,25 → ondulação; 1 → vãos).
2. **O Wet Paint esconde-o por três razões, nenhuma delas o Spacing:** (a) o pincel de fábrica nasce com Spacing
   **0,025**, 4× mais denso que o 0,10 do Digital; (b) o pigmento não se compõe por dab: acumula no RASTO e pousa em
   lotes (`trail/transfer.rs`), e a ÁGUA corre entre os dabs nos segundos seguintes (o solver) — os pelos viram riscas
   e os vãos enchem; (c) a curva de opacidade satura a massa. Com o MESMO Spacing o Wet Paint tem até MAIS vãos que o
   Digital na redonda (0,25: 38 % contra 0 %).
3. ⚠️ **Achado à parte — o Spacing do Wet Paint não é o do Digital:** o mesmo Spacing dá **3,2× menos dabs** (0,1 →
   17 contra 55 esperados). O hospedeiro passa o raio pela lei da largura (`wetpaint/largura.rs::raio_do_motor`: o raio
   do motor cuja largura MEDIDA é `2·raio` — o traço do motor é mais estreito que o raio) e o passo sai desse raio
   maior. O número do slider significa outra distância em cada meio.

## §3 — Como melhorar os outros meios sem perder desempenho (o desenho da próxima janela)

- **Cerdas que ARRASTAM (o «streak»), Digital/Impasto:** em vez de amostrar o tapete no centro de cada dab, amostrá-lo
  ARRASTADO pelo deslocamento `Δ` desde o dab anterior: `S_Δ(o) = máx_{t∈[0,1]} tapete(o − t·Δ)` — cada pelo desenha um
  risco contínuo entre dois dabs, como no motor molhado. **Custo:** o tapete arrastado coze-se UMA vez por `Δ` inteiro
  (128² × |Δ| ≈ 160 k operações para Δ = 10 px) num cache pequeno por thread (o traço recto reusa-o; uma curva troca
  de `Δ` devagar); por píxel continua UMA leitura. Igual ao custo de hoje, com a aparência do Spacing ≈ 1 px.
  A porta é `texture::bristles::sample_dab` (o Digital); o hospedeiro do Wet Paint continua a ler `bristles_at` sem
  arrasto (os 12 presets ao byte).
- **O dab-cápsula para a redonda (opção):** a queda medida pela distância ao SEGMENTO entre o dab anterior e o atual,
  não ao ponto — apaga as contas a qualquer Spacing com o MESMO custo por píxel (a caixa cresce `Δ`). Muda todo traço
  Digital de hoje (as contas a 0,25–0,5 somem) ⇒ entra como opção do Stroke, desligada, ou ligada com a prancha
  aprovada pelo dono.
- **O Spacing com o mesmo significado nos meios:** o passo do Wet Paint medido sobre a largura PROMETIDA (o raio
  pedido), não sobre o raio do motor — ou o painel mostrar o passo efetivo. Decisão com prancha (muda o traço do Wet
  Paint com Spacing ≠ fábrica).
- ⛔ **Recusado por desenho:** imitar a água (difusão) no Digital — custa um solver; o arrasto dá a continuidade sem ele.

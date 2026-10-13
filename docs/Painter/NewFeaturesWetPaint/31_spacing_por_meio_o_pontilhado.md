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

## §4 — Feito (prompt 32, 13/10)

### §4.1 — As cerdas que ARRASTAM (Digital, Impasto e toda rota que amostra o Shape pela `texture::sample`)

- **O passo chega pela porta do Flow:** `Dab::step` (o centro menos o do dab anterior, no caminho onde a tinta cai,
  pré-jitter; `[0, 0]` no 1.º dab e nas marcas deliberadas — Drag Dot, Anchored, Grid Stamp) → `ShapeFrame::Stroke
  { step }` (13 rotas, todas respondem) → `TexDabBasis::sweep` (o passo em TEXELS, resolvido 1× por dab no
  `shape_basis`). Simetria reflecte/roda o passo; o preenchimento do arremesso reparte-o.
- **A lei:** a MÉDIA do tapete sobre os `n = máx|D|` texels do passo, centrada (`texture/bristles.rs::sample_dab`).
  Os `n` dabs que um Spacing de 1 px poria ali deixam, somados, a exposição deste ⇒ a tinta de hoje, espalhada.
  `n ≤ 1`, Tiled e Flow ⇒ a leitura de sempre AO BIT. O hospedeiro do Wet Paint lê `bristles_at` (não varre).
- **Medido pelo produto** (`sonda_pontilhado_das_cerdas`, raio 20; prancha
  [`32_prancha_digital_cerdas_antes_depois.png`](32_prancha_digital_cerdas_antes_depois.png)):

  | Spacing | CV antes | CV depois | Wet Paint (antes = depois) |
  |---:|---:|---:|---:|
  | 0,025 | 0,014 | 0,002 | 0,043 |
  | 0,1 | 0,206 | **0,014** | 0,185 |
  | 0,25 | 0,513 | **0,052** | 0,587 |
  | 0,5 | 0,763 | **0,102** | — |

- ⛔ **Recusas MEDIDAS** (portão `as_cerdas_do_digital_riscam_em_vez_de_pontilhar`, pincel liso de cerdas): o MÁXIMO
  ao longo do passo (CV 0,086 / 0,170 a 0,1 / 0,25 contra 0,058 / 0,145 da média, e 2–5× a tinta: um pelo mais largo
  que um texel é cruzado por dois passos vizinhos e pinta duas vezes com o período do passo — tracejado visível); o
  varrimento FECHADO `t ∈ [−½, ½]` (conta o texel da ponta duas vezes).
- **Custo** (`measure_custo_do_arrasto`, `--profile smoke`, as variantes intercaladas, o mínimo de 9, `load` 10–16):
  recto 0,96–0,99× em todo Spacing; curva 0,99–1,00× salvo **0,25: 1,15×** (o passo em texels muda de dab para dab e
  cada sweep novo refaz os texels que toca — C11 do [26](26_ABERTOS.md)). Antes da cura «1× por dab» o custo era 1,6×.
- **Aberto:** o 1.º dab sai em pontos (C10).

### §4.2 — O dab-cápsula da redonda: NÃO entra (medido)

A redonda do Digital já é contínua até 0,25 (CV 0 / 0 / 0,005 a 0,025 / 0,1 / 0,25 — default 0,1); só a 0,5–1 as
contas aparecem, e aí o pontilhado é muitas vezes o que o artista pediu. Uma opção nova no Stroke por um caso que
ninguém relatou é um controlo a mais sem dono.

### §4.3 — O Spacing do Wet Paint: o passo efetivo contra o pedido (`sonda_passo_do_wet_paint`)

O passo efetivo ÷ `spacing · 2 · raio do painel`, traço recto de 220 px:

| preset | raio painel → motor | fábrica | 0,1 | 0,25 |
|---|---|---:|---:|---:|
| Express Oil | 32 → 33,7 | 1,00 | 1,07 | 1,06 |
| Acrylic Round | 39 → 45,9 | 1,00 | 1,23 | 1,25 |
| Wet Round · Wash · Drip | 10 → 15–18,5 | 1,00 | 1,55–1,86 | 1,57–1,91 |
| Wet Liner | 14,5 → 15 | 1,00 | 1,04 | 1,05 |
| Wet Dry Brush | 10 → 22,2 | 1,12 | 2,24 | 2,32 |
| Graphite 8B | 3,2 → 8,4 | 1,00 | 1,69 | 2,60 |
| Soft Pastel | 14,5 → 29,8 | 1,50 | 2,11 | 2,17 |
| Oil Pastel | 38,5 → 73,9 | 1,94 | 2,04 | 2,29 |
| Charcoal | 18,5 → 46,6 | 2,34 | 2,59 | 2,64 |
| Oil Bristle (achatado) | 51,5 → 52,3 | 0,97 | 0,33 | 0,33 |

Causa: um só reamostrador (o caminhador do `Stroke`), a `spacing · 2 · raio`, e o hospedeiro troca o raio pelo do
MOTOR (`dab_spec.rs`, a lei da largura); a fábrica mascara-o porque o passo bate no piso de 1 px. O Oil Bristle é
outra lei, deliberada: a ponta achatada encurta o passo ao longo dela (`walk.rs`, `ao_longo`).

**A lei proposta (B5 do [26](26_ABERTOS.md), decisão do dono):** o Wet Paint mede o passo sobre o raio do PAINEL —
uma edição no `dab_spec.rs` (o Spacing escalado por `raio painel ÷ raio motor` antes do caminhador). Muda o traço
com Spacing ≠ fábrica (mais dabs, o traço contínuo até ao fim) e 5 presets de fábrica (Charcoal, Soft/Oil Pastel,
Wet Dry Brush e o Oil Bristle pela razão dele) — a menos que se re-autorem os Spacings deles. ⚠️ O `spacing` também
é lido pelo Accumulate, o Space Attenuation, o `solid_carimbos` e o arremesso (`speed.rs`): a escala aplica-se só ao
spec do caminhador. Prancha: [`32_prancha_spacing_wet_hoje_proposta.png`](32_prancha_spacing_wet_hoje_proposta.png)
(Wet Round, Charcoal, Soft Pastel, Oil Pastel × 0,1 / 0,25; à esquerda hoje, à direita a lei — `sonda_prancha_passo_wet`).

### §4.4 — Smoke do dono (13/10): «não parece funcionar» + «uma opção a mais» — a caixa **Continuous**

- **O que o dono viu** (a captura: tufos rosa repetidos, pincel grande): o arrasto só existia no Digital, e o Wet
  Paint (`bristles_at`) não varria — a 0,25 com o pincel grande chegam 3 dabs ao motor. E o 1.º dab, sem passo,
  deixava os pontos dele nítidos numa boa parte do início (raio 45: ~100 px).
- **Agora é uma OPÇÃO:** `BrushSpec::continuous_spacing` (omissão OFF ⇒ todo `Dab::step` é zero ⇒ o traço de antes
  ao bit, nos quatro meios). Caixa **Continuous** logo abaixo do Spacing (Stroke), em todo meio; esmaece
  (`Dependente::ContinuoSemCerdas`) exatamente quando o arrasto não se arma (`drags_bristles`: sem as cerdas, Tiled,
  Flow) — ⛔ esmaecê-la também onde o Wet Paint não lê as cerdas (Grain armado, ferramenta que não deposita) foi
  REPROVADO pelo censo `a_linha_esmaecida_e_a_que_nao_age`: o aquecimento sozinho muda o momento em que a água
  recebe os dabs, e a tinta muda. O preset
  leva-a (`CAMPOS_DO_PINCEL`, 30).
- **O 1.º dab:** com a opção e as cerdas (`BrushSpec::drags_bristles`), o traço AQUECE como o Rake (`warmup.rs`) e
  o dab de abertura recebe um passo nominal (`dab_spacing_px`) na direcção assente. Preço: o atraso do aquecimento
  do Rake (`heading::warmup_len`), só com a caixa ligada e as cerdas.
- **A lei por PORTA (`Varre`):** o carimbo (Digital/Impasto/…) usa a MÉDIA (§4.1); o Wet Paint usa o MÁXIMO — o
  depósito dele subtrai um portão de papel de `fall · texv` (`trail/deposit.rs::cell`), e a média dilui os pelos
  abaixo do portão: medido, metade da tinta a 0,1 e o traço quase vazio a 0,25.
- **Medido pelo produto** (`o_spacing_continuo_risca_no_digital_e_no_wet_paint`, raio 20, Spacing 0,25, desligada →
  ligada): Digital CV 0,559 → **0,055**, tinta 387 k → 394 k · Wet Paint CV 0,762 → **0,081**, tinta 49 k → 231 k
  (o traço desligado é tufos com vãos; ligado, riscas cheias). Sonda `sonda_pontilhado_das_cerdas` com
  `PH2D_SONDA_CONTINUO=1`, raio 20 — Wet Paint CV 0,1 / 0,25: 0,185 → 0,081 · 0,587 → 0,079; Digital (raio 45)
  0,5: 0,763 → 0,058 (o tufo do 1.º dab já não está).
- Prancha: [`32_prancha_continuous_desligada_ligada.png`](32_prancha_continuous_desligada_ligada.png) (Digital e Wet Paint, raio 20, 0,1 / 0,25, desligada e ligada).
- **Aberto:** no Wet Paint a 0,25 ligado vê-se uma COSTURA vertical fina entre dois dabs (C12). Com pincel grande a 0,25–0,5 o Wet Paint recebe pouquíssimos dabs (o passo sai do raio do MOTOR, B5)
  e o traço acaba antes da mão; a caixa não cura isso — a lei do B5 sim.

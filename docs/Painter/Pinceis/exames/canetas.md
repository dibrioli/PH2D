# Exame da família `canetas` (J3, catálogo C, meio Digital) — 20/10

> Agente da família, JANELA 3. Briefing: [`BRIEFING_FAMILIA.md`](../BRIEFING_FAMILIA.md). Pranchas:
> [`pranchas/canetas/`](../pranchas/canetas/) (cegas e chaves ao lado). Receitas e portão:
> `crates/ph2d-tool-painter/tests/it/familias/canetas.rs`. Medida da identidade (espelho do portão):
> [`canetas_foto.py`](canetas_foto.py). Bancada sempre com `PH2D_EXAME_DIR=/tmp/ph2d_exame_canetas`.
> Critério de veredito: o do briefing §2 — cada diferença visível listada pelo revisor cego é curada e revista por um
> revisor NOVO, ou o pincel sai. 11 revisores no total (3–4 rondas por pincel).

## Resumo

| # | pincel | veredito | porquê, numa linha |
|---|---|---|---|
| C-5 | **Mapping Pen** | ✅ APROVADO | o 4.º revisor acertou o oráculo em 3/6 linhas (moeda ao ar); d 0,869 da biblioteca |
| C-4 | Studio Pen | ❌ reprovado | a rampa de pressão do G-pen é uma curva em S; a nossa lei (potência + tecto) dá «joelhos» — 3 revisores viram-nos; e d 0,223 do Hard Round |
| C-6 | Technical Pen | ❌ reprovado | a velocidade (a única identidade, L12) põe uma BOLHA no início do traço rápido (o 1.º tique tem v = 0); sem ela é o Pixel 1px (d 0,357) |
| C-7 | Ballpoint | ❌ reprovado | 3 revisores: no traço leve o nosso falha em buracos brancos NÍTIDOS (grão Height), o MyPaint esbate em fiapos; o cruzamento sobre o vermelho nem Mix nem Multiply acertam |
| C-11 | Dry Ink | ❌ reprovado | contra o Krita Pen Rough: pontas redondas × cortadas, gomos × serrilhado fino a 200 px, d 0,179 do Hard Round; contra o Rebelle Ink Brush 3 (o do §3 Fusões) falta o ARRASTO da tinta molhada — é do Wet Paint |
| C-10 | Brush Pen | 🔀 fundido | o Krita Ink-7 Brush Rough é o Ink-3 G-pen com pó na orla: a nossa cópia fiel ficou a d 0,187 do Studio Pen, a mesma lei de pressão |

## C-5 Mapping Pen — ✅ APROVADO

- **Receita** (Digital): `hardness 0,2`, `falloff Pow4`, `spacing 0,08` SEM a cápsula (a cápsula satura a orla e
  endurece-a), `taper start 0,2 · tip 0,6` (o início afila como no Krita), `pressureSize 0` (largura FIXA), Strength
  0,99, tamanho de fábrica **3 px** (o `diameter="3"` do `d)_Ink-1_Precision.kpp`, CC-0). A largura que se vê é ~0,7 do
  tamanho (como no Krita: 28 px a «40»).
- **Oráculo**: Krita `d) Ink-1 Precision` (`fixtures/krita_bateria/d__Ink-1_Precision`). Escala do exame 1,167.
- **Prancha**: [`pranchas/canetas/mapping_pen.png`](../pranchas/canetas/mapping_pen.png) · régua **446/454** ✓
  ([`mapping_pen_regua.txt`](mapping_pen_regua.txt)). O perfil transversal casa o do Krita ponto a ponto
  (0,43/0,56/0,69/0,81 contra 0,45/0,59/0,72/0,83 a ½ largura).
- **Vizinho**: Pixel 1px **d 0,869** (≥ 0,596: claramente longe); depois Soft Pastel 1,107, Soft Round 1,135.
- **Razão de uso**: o fio de largura constante e borda macia, para hachura e detalhe fino, que NÃO responde à pressão
  (nenhum outro da biblioteca: o Soft Round alarga com a pressão e é nuvem, o Hard Round é duro e varia).
- **Revisão cega** (ronda 4, revisor novo, `cegas/mapping_pen.png`): «ORIGINAL: 2 — confiança baixa» em todas as
  linhas; cruzamento: escolheu o NOSSO como original em 3 das 6 linhas (gestos B, tamanhos B, sobre A) — indistinguível.
  Diferenças que listou (pequenas): a orla do nosso um pouco mais curta a 200 px; o início em bico (o taper) — que a
  ronda 2 pedira (o Krita começa «em bala»); uma franja cinza-violeta no cruzamento sobre o vermelho. As rondas 1–3
  pediram: perfil mais gaussiano (curado: Pow4 sem cápsula), artefactos de carimbo a 200 px (curados), o fio de 5 px
  pálido (melhorado: pico 0,64 → 0,80, Krita 0,89 — a ponta Pow4 de 2,5 px perde área no AA). O terceiro (Pixel 1px)
  foi «claramente outro» em todas as rondas.
- **Custo**: ver §Custo.
- **Portão**: `familias::canetas::cada_caneta_se_reconhece_pela_foto` (a RECEITA pintada na cena `gestos` à escala do
  oráculo: orla, opacidade, rampa a 0,15 e a 0,5, rugosidade, tinta no traço leve contra a foto, e a lei das IRMÃS: mais
  perto da foto do Precision que das do G-pen, Fineliner, ballpen e Pen Rough). Verde. **Mutação**: M1 (Pow4 → Smooth,
  dureza 0,9) SANGRA (orla 1,0 × 9,0 ± 3,65); M2 (`pressureSize 0,9`) SANGRA (rampa 0,15 × 1,0); M3 (Strength 1,0)
  SOBREVIVE — declarado: com o passo solto o tecto por texel não muda a identidade (o perfil é do dab).

## Os reprovados — o que se mediu e o que falta

- **C-4 Studio Pen** (Krita `Ink-3 Gpen`, MyPaint `pen`): receita medida: `hardness 0,98`, cápsula, `pressureSize 0,93 · curve 2,0 · ceiling 0,7 · smoothing 0,8`,
  5 px. Régua 448/454. Os revisores (3 rondas) acertaram o oráculo e viram sempre: (1) a rampa com «joelhos» onde a
  largura enche (o tecto da `Pen::read` é uma quina; a suavização arredonda-a mas atrasa o fim, que fica rombo); (2) o
  fio inicial em degraus. ⇒ **motor (fila)**: uma curva de pressão em S — um tecto MACIO (smoothstep perto do tecto) ou a
  curva editável do Krita no `Dynamics`; é uma linha nova do cartão Pen (`LINHAS_DA_MAO` 18 → 19, i18n), não
  «pequeno» para uma família. Sem ela o Studio Pen é o Hard Round (A-1) com outra curva (d 0,223, PERTO). CAP-28
  (estabilização): o nosso Digital tem `stabilizer` (0,5 de fábrica) e `pulledString`; a bateria não os exercita.
- **C-6 Technical Pen** (Krita `Ink-2 Fineliner`, MyPaint `liner`): `hardness 0,85–0,98`, `pressureSize 0,22 · curve 1,25
  · ceiling 0,86 · speedSize 0,058` (L12). Os 2 revisores: a «gota» no início do recto e do vertical RÁPIDOS (o Krita
  começa fino) — é o `speed.rs`: a velocidade mede-se por TIQUE e o 1.º tique tem v = 0, então o 1.º dab sai largo.
  ⇒ **motor (fila)**: a velocidade do 1.º segmento no 1.º dab. Sem a velocidade é um disco duro de largura fixa = Pixel
  1px/Hard Round à mesma largura (d 0,357).
- **C-7 Ballpoint** (MyPaint `deevad/ballpen`, d 1,05 — distinta): `falloff Sharp`, dureza 0, Strength 0,68,
  `pressureOpacity 1 · pressureSize 0,45`, grão Noise 100×35 Tiled lei **Height** pela pressão (L9), jitter 0,04, taper de
  opacidade no início, Multiply no plano do traço (L8). Régua 358–402/454. Os revisores das rondas 2–3 escolheram o
  NOSSO como original na maioria das linhas, MAS listaram sempre: no traço leve o MyPaint esbate em fiapos borrados e
  afina (o `offset_by_random` dele CRESCE quando a pressão cai); o Height só faz buracos binários. O cruzamento azul
  sobre vermelho: Mix dá cinza-azulado claro, Multiply quase preto, o MyPaint dá azul-marinho; o pigmento (K–M) escurece
  também o papel (cobertura 0,82 × 0,56). ⇒ **motor (fila)**: jitter de posição pela pressão INVERSA (ou o grão que
  esbate em vez de furar). ⚠️ A «gota no fim» do catálogo não está em nenhum oráculo da bateria (CAP-13, não medida).
- **C-11 Dry Ink**: três desenhos. (a) Krita `Ink-4 Pen Rough`: jitter em píxeis da vista 0,5 + elipse 0,08 que roda,
  `pressureSize 1 · curve 1,05`, 7 px (o `diameter="7"` do kpp); régua 446/454, mas o revisor viu pontas redondas × as
  cortadas do Krita, gomos × serrilhado fino, e a 200 px as bolhas de cada carimbo; e fica a d 0,179 do Hard Round.
  (b) Rebelle `Ink/Brush/Brush 3` (o «Rb Ink Brush 3» do §3 Fusões): 2.ª ponta (L6) de 16 discos 0,1·D em Multiply +
  blend Multiply → d 1,18 (distinta), régua 428/454, mas o revisor viu o ARRASTO do escuro do cruzamento ao longo do
  traço, as bordas fibrosas e a opacidade pela pressão — o Ink Brush 3 é tinta MOLHADA (pickup do Wet Paint), não um
  carimbo. ⇒ **fila**: o Dry Ink do Rebelle vai para o Wet Paint (família `tintas` / J4), não para o Digital.
- **C-10 Brush Pen** fundiu-se: a cópia fiel do Krita `Ink-7 Brush Rough` (grão Random em Color Burn na orla) ficou a d
  0,187 do Studio Pen; as fotos do Krita mostram o mesmo traço com pó na orla.

## Motor: o que se testou e ficou de fora

- **L5 (a cauda)**: com `taper end` o fim do traço sai uma BOLHA redonda mais larga que o traço (recto, verticais,
  espiral) e o início afila em espinho côncavo — [`pranchas/canetas/cauda_L5_bolha_no_fim.png`](../pranchas/canetas/cauda_L5_bolha_no_fim.png)
  (Studio e Technical com `start/end` 1,5 e 0,8 D). ⚠️ Defeito do motor (re-pintar a cauda ao levantar), para quem possui
  `traco_plano.rs`. Nenhum oráculo que corre afila a pressão constante (Krita, MyPaint, ArtRage), por isso as canetas
  aprovadas não usam a cauda; a decisão 5 do `ESTADO.md` continua por perguntar no smoke.
- Nenhum ficheiro de motor foi tocado.

## Custo (Mapping Pen; `exame.sh --custo`, perfil smoke, 60 Hz)

| px | ms/quadro mediana · p95 · pior | > 16,7 ms | RSS pico | load |
|---|---|---|---|---|
| 5 | 0,05 · 0,08 · 0,36 | 0/475 | 21 MiB | 24,9 |
| 40 | 0,16 · 0,22 · 0,74 | 0/475 | 21 MiB | 22,1 |
| 200 | 0,01 · 0,46 · 1,30 | 0/475 | 25 MiB | 19,0 |

⚠️ Load 19–25 (as outras famílias): o relógio acima de ~5 não vale como número absoluto, mas o p95 de 0,46 ms a 200 px
está 36× dentro do orçamento — não muda o veredito. Rato e caneta: a Mapping Pen não lê pressão nem inclinação (a
largura é fixa por desenho); a caneta deitada não faz nada.

## Portões por nome (20/10)

`familias::canetas::cada_caneta_se_reconhece_pela_foto` ✓ · `the_factory_presets_are_counted` ✓ ·
`cada_preset_de_fabrica_volta_ao_byte` ✓ · `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` ✓ ·
`familias::cada_familia_e_um_ficheiro` ✓. `cada_preset_de_fabrica_e_a_sua_receita` vermelho por `Pixel 1px`
(família `basicos`, a meio — regerar o dela); a da Mapping Pen confere-se no gerador (`confere_receita_do_meio`, verde).

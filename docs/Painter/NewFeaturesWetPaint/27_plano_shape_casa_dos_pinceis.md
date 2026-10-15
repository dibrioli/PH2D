# 27 — Plano: a seção **Shape** é a CASA dos pincéis, em todas as mídias (13/10)

> Pedido do dono (13/10): *«a seção Shape [deve ser] a casa onde se cria e configura os pincéis … as
> texturas usadas nos pincéis Wet Paint, sejam em png, sejam procedurais, devem estar no dropdown de textura
> e as configurações devem sair de Tuning e ir para Shape»*. Engloba o **Bug #47** (Shape Noise no Wet
> Paint quase não pinta) e os itens C1/C2 de [26_ABERTOS](26_ABERTOS.md).

## §1 — O que o código É hoje (medido 13/10, com endereço)

1. **O «pincel» do Wet Paint é UM só, e não é uma imagem:** um tapete de cerdas 128×128 GERADO por código
   (`ph2d-wet-paint/src/brush.rs:61` `build_bristle_texture`, semente fixa) a partir de 4 knobs do **Tuning**:
   `felt` (0–0,2), `bristleCount` (0–4000), `bristleStrength` (0–4), `bristleSize` (0,2–4)
   (`tuning/defs.rs`). Nenhum PNG de pincel existe no motor.
2. **Ele não é a FORMA da ponta — é a textura DENTRO dela.** O produto passa SEMPRE a silhueta do
   hospedeiro (`wetpaint/dab_route.rs:~434`: falloff × Shape × pegada) e o motor só põe a cerda no lugar da
   textura (`brush.rs:234`: `match grain { Some(g) => g, None => sample_bristle }`). A cerda é amostrada em
   **píxeis** a partir do centro do dab (ladrilho de 128 px), não na escala da ponta.
3. **Um Grain armado APAGA as cerdas** (decisão W2.4: *«o Grain é A textura»*, `dab_route.rs:178`).
4. **A lei da célula** (`trail/deposit.rs:97`): `stamp = fall · texv · intensity`, `depósito = stamp −
   (1 − tooth)·gate`. A subtração é ABSOLUTA: uma textura de média baixa (o Noise, ~0,4) põe o `stamp` abaixo
   do gate e a célula não recebe nada — **é o Bug #47**. No Digital a mesma textura só multiplica.
5. `BrushShape {Round, Flat, Fan}` do motor: ninguém fora do motor os escolhe (o produto achata pelo
   «Flatten & Rotate» do Shape). Código provavelmente morto — confirmar com o censo.
6. **Tuning**: dos ~40 knobs, só os 4 das cerdas são da PONTA; o resto é física de água, pigmento e papel
   (Paint, Water, Physics, Tools, Paper/Emboss) e FICA no Tuning. O `spacing` já está escondido.
7. **Shape** (`ph2d-panel-painter-layers/src/paint_shape.rs:48`): dropdown com 29 tipos (`TextureKind`,
   `texture/kind.rs`), os procedurais com os seus parâmetros na própria seção. Por mídia só se esconde o
   «Automatic» (Watercolor), o Shape Tone (Wet Paint) e Relief/Shine (Impasto). «Use Texture Colors» aparece
   vivo no Wet Paint e no Watercolor, que não o lêem (26_ABERTOS C1).
8. **Presets**: as cerdas viajam no mapa `engine` do preset (`wetpaint_preset.rs:~406`), não como ponta.

## §2 — Concordo, e o desenho

O Shape descreve **a ponta**: o contorno e a textura dentro dela. Isso vale para o Digital hoje; no Wet Paint
a ponta já vem do Shape, e só a textura de cerdas mora escondida no motor. Trazê-la para o Shape não muda
a física (água, pigmento, papel ficam no Tuning) — muda **quem é dono da ponta**.

- **D1 — «Bristles» vira um tipo de textura do Shape** (o 30.º), com os seus 4 parâmetros (Count,
  Strength, Size, Felt) na seção, como os de qualquer procedural. A lei mora UMA vez (o gerador sai do motor
  para o `ph2d-painter-brush`, ou o motor exporta-o — um só gerador) e é amostrada em **píxeis**, como hoje
  (ladrilho de 128 px que não escala com o tamanho do pincel). Disponível em TODAS as mídias: no Digital é
  uma ponta de cerdas.
- **D2 — No Wet Paint o tipo do Shape chega ao motor pelo papel certo:** as cerdas entram como `texv` (a
  textura), exactamente como hoje ⇒ um preset com Shape = Bristles e os mesmos 4 valores pinta **ao bit**
  o que pinta hoje (portão: a impressão da sessão do motor e as miniaturas da fábrica).
- **D3 — Tuning perde os 4 knobs das cerdas** (escondidos, como o `spacing`); ficam só física.
- **D4 — Migração dos presets:** ler `engine.felt/bristle*` antigos ⇒ Shape = Bristles com esses valores
  (presets de fábrica e do usuário); gravar só no Shape. O Shape «None» no Wet Paint passa a ser uma ponta
  LISA (sem cerdas) — capacidade nova, hoje impossível.
- **D5 — A textura de imagem (PNG) já funciona** como ponta no Wet Paint (silhueta pelo alfa, #52 incluído).
  Uma BIBLIOTECA de pontas em PNG (como as do Rebelle) é um passo seguinte, decisão do dono.
- **D6 — O Bug #47 cura-se na LEI, medida contra o oráculo:** o Rebelle (porta WebSocket, instalado sob
  Wine) pinta com pontas texturadas; medir como a textura da ponta e o dente do papel se combinam lá e
  escolher a lei que dá o MESMO — candidatas: o gate proporcional à envolvente (`fall` sem a textura) ou a
  textura aplicada DEPOIS do gate. ⚠️ A doc do #47 já registou que o gate proporcional muda o traço redondo
  de hoje ⇒ portão de byte nos presets redondos + a prancha antes/depois para o dono.
- **D7 — Shape × Grain:** hoje o Grain apaga as cerdas. Com as cerdas na ponta e o papel no Grain, o
  Rebelle MULTIPLICA os dois (Shape × Grain, doc 02). Medir os dois (o «escurece em dobro» da W2.4 contra o
  oráculo) e decidir pela medida.
- **D8 — A seção coerente por mídia** (26_ABERTOS C1): «Use Texture Colors» esmaecido onde a cor da textura
  não pinta; cada linha do Shape com o seu leitor por mídia no censo `censo_dos_controlos`.

## §3 — As ondas

| onda | o quê | portão |
|---|---|---|
| **W0** | MEDIR primeiro, num lote: os 30 tipos do Shape no Wet Paint × Digital (tinta por tipo, a tabela do #47 inteira); o Rebelle com 3 pontas texturadas sobre os nossos gestos (fixture com cabeçalho); o «Shape × Grain» nos dois; o censo do Flat/Fan | sondas `measure_*` + fixtures |
| **W1** | Bristles no `TextureKind` + o gerador único + os 4 parâmetros + amostragem em píxeis; Digital/Impasto pintam com ele | o ladrilho AO BIT com o do motor de hoje (mesmos knobs) |
| **W2** | Wet Paint lê as cerdas do Shape; Tuning esconde os 4; migração dos presets (fábrica + usuário) | impressão da sessão do motor ao bit · miniaturas da fábrica ao bit · preset antigo carregado = o mesmo traço |
| **W3** | a lei da célula do #47 pela medida da W0 | o Noise pinta (tabela antes/depois) · redondo ao bit OU prancha aprovada pelo dono |
| **W4** | o painel por mídia (D8, C1) + C2 (o Solid no Wet Paint lê a ponta) se a W0 o pedir | censo dos controlos; seam do painel |
| **W5** ✅ | limpeza (Flat/Fan se mortos), doc do Shape, smoke ao dono | suíte afetada + clippy + mutação nas leis novas |
| **W6** | **Shape Color no Wet Paint** (ordem do dono 15/10, lista 26 A7): medir a cor por célula no depósito do motor e o custo por dab; depois as duas camadas pintam na água | o traço de hoje ao byte com a secção desligada (12 miniaturas + `fingerprint`) · a cor segue a textura (régua do P38) · prancha ao dono |

## §4 — Riscos e recusas

- O Wet Paint é a lei do Rebelle já calibrada (presets, miniaturas): **nada de mudar o traço de hoje sem o
  portão ao bit ou a prancha aprovada**.
- A escala das cerdas é em PÍXEIS — se virar escala da ponta (como os outros procedurais), muda todo preset;
  recusado por desenho, salvo medida que o justifique.
- ⛔ Não reconstruir o que a W2.4 decidiu (o Grain substitui as cerdas) sem a medição D7.

## §5 — Decisões do dono

- **Bristles também no Digital/Impasto** — o plano diz SIM (o Shape define o pincel igual em toda mídia).
- **Biblioteca de pontas em PNG** (D5) — depois desta onda.
- **B6 — a ponta LISA** (lisa ÷ cerdas 1,0–6,2 por preset, [prancha 39](39_prancha_b6_ponta_lisa_e_cerdas.png)) —
  perguntado no smoke de 15/10 (§6.10); a resposta entra aqui e na [lista 26](26_ABERTOS.md).

## §6 — W0 medido (13/10) e W1 feita

Sondas: `crates/ph2d-tool-painter/src/tool/paint/wetpaint/sonda_shape_casa.rs` (`#[ignore]`, `--profile smoke`).
Números crus: a saída das duas sondas está reproduzível pelo comando no topo do ficheiro.

### 6.1 — A tabela do #47 inteira (produto real: tela branca 320×96, pincel de fábrica, traço de 220 px, `Σ(255 − min(rgb))`)

| tipo | Wet ÷ None | Digital ÷ None | | tipo | Wet ÷ None | Digital ÷ None |
|---|---:|---:|---|---|---:|---:|
| **None** (cerdas de hoje) | 1 (252 240) | 1 (834 982) | | Crosshatch | 0,153 | 0,522 |
| **Noise** | **0,004** | 0,448 | | Dots | 0,160 | 0,270 |
| Checker | 0,403 | 0,847 | | Grid | 0,136 | 0,331 |
| Voronoi | 0,048 | 0,756 | | Bricks | 0,701 | 0,936 |
| Stripes | 0,415 | 0,728 | | Waves | 0,194 | 0,715 |
| Clouds | 0,013 | 0,642 | | Chevron | 0,625 | 0,692 |
| DistortedNoise | 0,050 | 0,668 | | Diamonds | 0,417 | 0,862 |
| Magic | 0,234 | 0,810 | | Triangles | 0,509 | 0,759 |
| Marble | 0,267 | 0,809 | | Hexagons | 0,523 | 0,937 |
| Musgrave | 0,005 | 0,504 | | Scales | 0,615 | 0,942 |
| Wood | 0,516 | 0,828 | | Weave | 0,452 | 0,937 |
| Stucci | 0,002 | 0,150 | | PaperCold/Rough/Hot | 0,216 / 0,226 / 0,223 | 0,80 |
| Gradient | 0,228 | 0,836 | | **Bristles** (W1) | 0,144 | 0,461 |
| Grain | 0,183 | 0,790 | | | | |

⇒ o #47 é uma FAMÍLIA: todo tipo de média baixa e contínua (Noise, Clouds, Musgrave, Stucci, Voronoi,
DistortedNoise) some no Wet Paint (≤ 5 %); no Digital nenhum passa de ~6× abaixo do redondo.

### 6.2 — As leis candidatas (D6), no pigmento que POUSA (sem a água)

`stamp = env · sv · texv · I`, `g = (1 − dente)·gate + dry_gate·ext·0,6`. **Hoje:** `stamp − g`. **A:** `stamp − g·env`.
**B:** `(env·texv·I − g) · sv` — o padrão do Shape DEPOIS do portão. Dab real do motor (gancho `on_dab`), portão,
papel e cerdas de cada preset de fábrica. A coluna «hoje» reproduz o produto (Wet Round Noise: 0,004 nos dois).

| preset | Noise ÷ None hoje · A · B | Clouds hoje · A · B | redondo: A ÷ hoje (Σ · píxeis) |
|---|---|---|---|
| Wet Round | 0,004 · 0,011 · **0,093** | 0,029 · 0,060 · 0,228 | **1,358 · 1,748** |
| Wet Wash | 0,006 · 0,032 · 0,110 | 0,051 · 0,119 · 0,240 | (≈ Wet Round) |
| Wet Dry Brush | 0,004 · 0,014 · 0,093 | 0,022 · 0,052 · 0,242 | 1,447 · 2,081 |
| Wet Drip | 0,009 · 0,031 · 0,101 | 0,083 · 0,131 · 0,236 | 1,251 · 1,791 |
| Wet Liner | 0,219 · 0,221 · 0,265 | 0,305 · 0,307 · 0,352 | 1,006 · 1,003 |
| Soft/Oil Pastel, Charcoal, Graphite (Grain) | ≈ 0,008 · 0,018 · 0,086 | ≈ 0,021 · 0,028 · 0,211 | **2,7–2,9 · 2,4–2,6** |
| Express Oil, Oil Bristle, Acrylic (gate 0) | 0,25–0,36 · igual · 0,23–0,28 | 0,37–0,50 · igual · 0,33–0,38 | 1 · 1 |

- **B** não muda NENHUM preset redondo (`× 1` exacto — por construção) e multiplica o Noise por ~23 no Wet Round.
  O resto (0,093 e não ~0,4) é a CORRELAÇÃO: as cerdas e o padrão View estão os dois presos ao centro do dab, e as
  poucas pontas que passam o portão leem sempre o mesmo valor do padrão.
- **A** muda todo traço redondo dos meios com portão (Wet Round +36 % de pigmento e +75 % de píxeis; pastéis ×2,7).

### 6.3 — O oráculo (Rebelle 8.3.4, WebSocket sob Wine, os NOSSOS gestos; fixtures `fixtures/rebelle/ponta_textura_*`)

Ponta texturada comum ⇒ **multiplica e ACUMULA, sem corte**: Fan 0,72–1,05× do redondo, Gouache Dry (74 % de
buracos na ponta) 0,77–1,91× com 2 % de miolo vazio; a tinta cai em linha recta com a Opacity e nunca zera. O
limiar duro só aparece no Granulation, cujo grão o AUTOR do preset escureceu (brilho −0,65): zero a ≤ 55 %.
Script `ferramentas/ponta_textura_rebelle.{sh,py}`. ⛔ O papel não se desenha sob Wine ⇒ D6 (ponta × dente) e D7
(Shape × Grain) não se medem no oráculo; o 1.º traço depois de trocar de preset às vezes sai com o anterior
(aquecimento + 2,5 s).

### 6.4 — Censo (D5/W5)

`BrushShape::{Flat, Fan}` do motor: MORTOS no produto — só `Round` é construído (`painter.rs:173`); o elipse
(`brush.rs:283-348`) só corre no ramo sem silhueta, e o hospedeiro passa sempre `Some(&sil)` (`dab_route.rs:419,433`).
Os 4 knobs das cerdas: lidos só por `painter.rs:207` → `build_bristle_texture`; escritos pelo mapa `engine` dos 12
presets (valores em `receitas_de_fabrica.rs`); esconder = `KnobGroup::Hidden` em `tuning/defs.rs` (os dois filtros
`ph2d-panel-wet-tuning/src/rows.rs:73` e `wetpaint_settings.rs:530` seguem). A cerda entra no produto num só sítio:
o `None` de `brush.rs:236` (`stamp_row_shaped`).

### 6.5 — As decisões

| D | decisão (pela medida) |
|---|---|
| D1 | ✅ **feita (W1)**: `TextureKind::Bristles` (29, o 30.º do dropdown), sliders Count · Strength · Size · Felt; UM gerador (`ph2d_wet_paint::brush::build_bristle_texture`, o `ph2d-painter-brush` depende do motor); amostrado em píxeis (`texture::sample` → `sample_bristle`); nunca no carimbo cacheado. Padrão = o tapete do Wet Round com a **Strength 2**: a 1 os picos (0,16–0,32) ficam sob o `W_TAIL` 0,35 do filme do Impasto e o traço do Impasto sai VAZIO (Strength 1 → 0 · 1,5 → 43 688 · 2 → 192 835). |
| D2/D4 | sem mudança — W2. A lei unidade↔knob (`brush/unidade.rs`) encaixa no passo do Tuning e faz a ida-e-volta AO BIT em todo passo e nos 12 presets: a migração pode gravar só no Shape. |
| D6 | **Lei B** (o padrão procedural do Shape multiplica DEPOIS do portão; a envolvente e a imagem ficam antes). Único candidato que deixa todo redondo ao bit; é a forma do oráculo (multiplica, sem limiar). A W3 implementa-a pela porta `stamp_row_shaped` (um terceiro canal `sv`). A lei A está **recusada** (muda os redondos 25–190 %). |
| D7 | não mensurável no oráculo (o papel não desenha sob Wine). A W2 decide medindo o «escurece em dobro» com a lei B; a referência escrita continua a do doc 02 (o Rebelle multiplica Shape × Grain). |
| C2 | sem pedido novo da W0. |
| Flat/Fan | mortos — saem na W5. |

**Estado da W1 no Wet Paint:** o Shape Bristles já aparece e pinta (0,144 do redondo), mas como SILHUETA
(falloff × cerdas) sobre as cerdas do motor — a W2 passa-o para o papel de textura (`texv`).

### 6.6 — Smoke do dono (13/10): o 1.º Size, o Rake e as unidades

*«temos 2 tipos de size no pincel … o primeiro do painel ainda não tem efeito (… os nomes precisam ser
diferenciados). Rake parece que também não faz efeito. Veja com cuidado cada parâmetro da seção. Não deixe nada
do wet paint se perder»*. Medido pelo censo dos controlos com o Shape em Bristles
(`censo_dos_controlos::sonda_cada_controlo_com_as_cerdas_no_shape`): **Size X/Y, Angle, Offset X/Y e o Jitter
Rotate MORTOS nos três meios** — a W1 lia a cerda só pelo desvio cru ao centro do dab. O Rake/Flow nem é gesto
do censo; o portão novo mostrou-os mortos também. O Rake de um procedural (Noise) sempre agiu: num traço recto
a tangente não gira, então ele lê-se «morto» a olho.

- **Cura:** `texture::bristles::sample_dab` aplica a transformação INTEIRA do Shape à cerda, em PÍXEIS — a pegada
  do dab (achatar/rodar, Jitter Rotate, a tangente do Rake/Flow; em unidades de raio e de volta a píxeis), o
  Angle, o Size nos eixos do padrão, o Offset (e o jitter do Random) a 64 px por unidade; Flow pelo comprimento de
  arco, Tiled preso à tela. No neutro cada passo é exacto (`× 1`, `+ 0`): o ladrilho do motor no desvio cru.
- **Nomes:** Bristle Count · Bristle Strength · Bristle Size · Felt (o Size da transformação fica «Size»).
- **Nada se perde:** `param_escala` — o Shape e o Grain MOSTRAM e ACEITAM as cerdas nas unidades do Tuning
  (Count 0–4000 de 10 em 10, Strength e Size de 0,05, Felt de 0,001; faixa e passo da `KNOB_DEFS` do motor).
- **Portões:** `cada_transformacao_do_shape_muda_a_ponta_de_cerdas` (num arco; Bristles e o CONTROLO Noise ×
  Digital · Impasto · Wet Paint × 8 gestos; mutação «lei antiga» → 21 mortos, todos nas cerdas) ·
  `nenhum_nome_da_secao_shape_se_repete` · `as_cerdas_digitam_se_nas_unidades_do_tuning`.

### 6.7 — W2 feita (13/10): o Wet Paint lê as cerdas do Shape

Smoke do dono: *«em Wet Paint, Shape ainda não aparece configurado»* — era a W2.

- **Rota** (`wetpaint/dab_route.rs`): Shape = Bristles ⇒ a silhueta é só o contorno (falloff × pegada) e as cerdas
  entram como o `texv` do motor pela porta do Grain, lidas por `texture::bristles_at` em CÉLULAS (`x − cx`, como o
  motor) com a transformação do Shape; um Grain armado continua a substituí-las (W2.4). Shape ≠ Bristles ⇒ ponta
  LISA (`texv = 1`, D4). O tapete do próprio motor deixa de ser lido pelo produto (fica para o Blend, que usa o
  carimbo do motor).
- **A cerda segue só a rotação POR DAB** (`FootprintDeform::so_o_rotor`: Jitter Rotate, Rake/Flow), não a pegada
  fixa do pincel (achatar + ângulo): um pincel chato é a mesma cerda numa pegada estreita. Medido: com a pegada
  inteira o **Oil Bristle** (achatado 0,68 a 90°) mudava 14 502 px da miniatura; com o rotor, os 12 ao byte.
- **Fábrica e migração:** o pincel Wet Paint nasce com o Shape em Bristles e as cerdas do Wet Round
  (`WetKnobs::cerdas_no_shape`); um preset de ANTES (as cerdas no mapa `engine`, o Shape vazio) chega com o Shape em
  Bristles e os mesmos valores; gravar já não põe as cerdas no mapa — a ausência delas diz que o Shape é a verdade.
  O Tuning esconde os 4 (`KnobGroup::Hidden`).
- **Painel honesto:** `Dependente::CerdasDaAgua` — os sliders e a transformação das cerdas esmaecem no Wet Paint
  com um Grain armado ou uma ferramenta da água que não deposita.
- **D7 decidido:** o Grain SUBSTITUI as cerdas (não multiplica). As cerdas têm média ~0,02 e são calibradas para
  ser o `texv` inteiro com a intensidade 3: multiplicá-las pelo grão apagaria os pastéis (que hoje pintam pelo grão).
- **Tinta por tipo depois da W2** (mesma sonda de 6.1, Wet ÷ None, onde None é agora a ponta LISA = 1 192 616, 4,7×
  as cerdas de fábrica): Noise 0,337 · Clouds 0,702 · Voronoi 0,825 · Musgrave 0,469 · Stucci 0,119 · Bristles
  (Strength 2) 0,464 — **a família do #47 pinta**, nas proporções do Digital. A lei B (W3) fica para o Shape
  procedural SOBRE um Grain (o `texv` é o grão e o portão continua a cortar o padrão).
- **Portões:** `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (os 12 ao byte pela rota
  nova) · `o_wet_paint_le_as_cerdas_do_shape` (Count muda; o `felt` do motor já NÃO — o controlo) ·
  `os_presets_antigos_chegam_com_as_cerdas_no_shape` (antigo → Bristles exacto; gravado sem as chaves; ida-e-volta;
  None gravado continua None).

### 6.8 — W3 feita (14/10): a lei B — o padrão do Shape multiplica DEPOIS do portão (fecha o #47)

Medido antes (sonda `sonda_shape_casa_tinta_por_preset`, o produto, os 12 presets de fábrica × 7 tipos, `÷ None`):
sobre o GRÃO (Soft/Oil Pastel, Charcoal, Graphite 8B) o portão ainda comia o padrão — Noise **0,000–0,001**, Clouds
0,002–0,004, Musgrave/Stucci 0,000.

- **Lei** (`ph2d-wet-paint`): `stamp_row_shaped` ganhou o 3.º canal `padrao: Option<CellFn>`; `DepositLaw::cell(…, sv)`
  faz `depósito = (min(env·texv·I, 1) − g)·sv` — o `sv` fora do `stamp` que o dente corta. `None` = `1` ⇒ o redondo ao bit
  (`× 1.0` exacto). O caminho do próprio motor passa `1.0`; a borracha e a máscara passam `None`.
- **Produto** (`wetpaint/dab_route.rs`): Shape PROCEDURAL ⇒ a silhueta é só a envolvente (falloff × pegada) e o padrão vai
  no canal, pela porta nova `ph2d_painter_brush::dab::shape_value_at` (a mesma que a `silhouette_at` compõe; AA por
  média das sub-amostras). A ponta de IMAGEM fica na silhueta (é contorno); a BORRACHA apaga pela silhueta inteira.
- **Depois** (mesma sonda): os 12 «como vem» e «None» **iguais ao byte de antes**; pastéis Noise 0,08–0,12 · Clouds
  0,22–0,31 · Voronoi 0,29–0,39 · Musgrave 0,13–0,19 · Dots 0,37–0,46 (era 0,14–0,18); Wet Round (ponta lisa) Noise
  0,246 → 0,450 · Musgrave 0,344 → 0,490; pincel padrão Noise 0,337 → 0,605. Onde o portão não cortava (Express Oil,
  gate 0) o padrão passou a pesar inteiro (Noise 0,904 → 0,642): antes o `min(·, 1)` saturava-o.
- ⚠️ **Resíduo medido, não defeito do portão:** sobre o grão o padrão rende 4–5× menos que sobre a ponta lisa (Noise 0,11
  × 0,45–0,60) — o que W0 previu (§6.2, B = 0,086): só os picos do grão passam, e o padrão nesses píxeis não é a média
  dele. Se o dono o achar fraco no smoke, a pergunta é a do D7 (Shape × Grain), não a do portão.
- **Portões:** `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (12 ao byte) · `deposit_rows`,
  `fingerprint`, `product_doors` (33 ✓) · `lei_b_tests::o_padrao_do_shape_pinta_sobre_o_grao_dos_pasteis` (Noise > 0,05,
  Clouds > 0,15, ambos < 1) · `…_sobre_a_ponta_lisa_nao_perde_os_vales` (Noise ≥ 0,5). ⛔ A comparação «a água pesa o
  que o Digital pesa» foi COINCIDÊNCIA do Wet Round (0,450 × 0,448): o pincel padrão dá 0,605 — composições diferentes.

### 6.9 — W4 feita (14/10): o painel por mídia (C1) e o Solid de carimbos na água (C2)

- **C1 — «Use Texture Colors» esmaece no Wet Paint e na Watercolor:** `Dependente::CorDaTexturaSemLeitor` (o id
  `PAINTER_SHAPE_PER_LAYER_COLOR`, os dois rótulos), lei `agua || aquarela` — as mesmas portas que tiram esses meios das
  rotas do carimbo antes do `is_color_mode` (`wet_owns_the_dabs`, `watercolor_render_active`). Dica
  `inerte.texture_colors_media`. O censo ganhou o gesto `PontaColorida` (a ponta de imagem colorida que o «Use as Brush
  Shape» instala) e a pré-condição nos três meios: Digital e Impasto AGEM, Wet Paint esmaece e é inerte.
  ⚠️ As linhas «Layer N Color» (multi-camada com Per-Layer ligado) NÃO esmaecem: ids de fábrica, fora do censo.
- **C2 — medido:** na água a mancha do Solid ia sempre LISA pela máscara (`pousa_na_agua` nunca lia o Shape), e no
  Digital uma ponta de IMAGEM enche de carimbos (#53). **Decisão:** a mesma lei na água — `enche_a_mancha_na_agua`
  pede os carimbos (`solid_fill_stamps`) e pousa pela MÁSCARA a União `1 − Π(1 − c·s)` das silhuetas deles
  (`cobertura_dos_carimbos`, a `silhouette_at` de cada carimbo com rotação e pegada; Tiling dá a volta). ⛔ **Recusado,
  medido:** os carimbos dab a dab pela rota do traço — cada dab isolado pousa ~1/10 da tinta de um traço (o traço
  sobrepõe ~10 por ponto): 39 de 400 píxeis do miolo. Pela máscara: tinta do miolo 119 176 × 125 449 da mancha lisa,
  com a forma dos discos. Portão `mascara_tests::com_ponta_de_imagem_a_mancha_da_agua_e_de_carimbos` (enche > 300/400 e
  desvio > 2× o da lisa; CONTROLO a redonda 400/400 lisa); `diag_c2_mancha_de_carimbos` grava os PPM.
- **A ponta LISA × as cerdas** (§6.7: a lisa deposita 4,7–6,2× as cerdas de fábrica): fica para o SMOKE do dono — se não
  for a lisa que ele espera, a calibração é um número só (`texv = 1`), contra `fixtures/rebelle/ponta_textura_round_*`.

### 6.10 — W5 feita (15/10): Flat/Fan saem, o doc do Shape, o portão do D7, a prancha do B6

- **Corte:** censo reconferido no código de hoje (só `Round` construído; a elipse só sem silhueta; o hospedeiro passa
  sempre `Some(&sil)`; nenhum preset os nomeia) ⇒ saem `BrushShape` inteiro, `Dab::{shape, dir_x, dir_y}`, a direção
  do `Stroke`/`StrokeEvent` (só alimentava a elipse) e os dois `dir` das portas `dispatch_pressure_dab*`.
  `for_each_stamp_pixel` é só o disco. Prova ao bit: `fingerprint` e as 12 miniaturas de fábrica (`27c75b50e3`).
- **Doc:** [40 — o Shape hoje](40_o_shape_casa_dos_pinceis.md) (as 30 formas, os três canais do motor, o painel por
  mídia, a Shape Color, o portão de cada lei).
- **Mutação nas leis W1/W2** (nunca provadas): 5/6 sangram — tapete em píxeis e knobs (W1), Bristles → `texv`, ponta
  lisa, migração, `CerdasDaAgua`. **Sobreviveu** «o Grain SUBSTITUI as cerdas» (D7, W2.4): portão novo
  `shape_bristles::um_grain_armado_substitui_as_cerdas_do_shape` (Soft Pastel, Bristles = None ao byte; o Count não
  muda), vermelho sob «grão × cerdas».
- **B6 medido** (`sonda_prancha_b6_lisa_e_cerdas`, azul do oráculo), lisa ÷ cerdas: Wet Round **6,23** · Wet Wash
  **4,28** · Wet Dry Brush **4,80** · Wet Liner 1,44 · Acrylic Round 1,66 · Oil Bristle 1,00 (os óleos saturam). Na
  [prancha](39_prancha_b6_ponta_lisa_e_cerdas.png) o Round do Rebelle fica ENTRE as nossas cerdas e a nossa lisa.

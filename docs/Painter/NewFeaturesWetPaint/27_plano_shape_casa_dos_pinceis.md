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
| **W5** | limpeza (Flat/Fan se mortos), doc do Shape, smoke ao dono | suíte afetada + clippy + mutação nas leis novas |

## §4 — Riscos e recusas

- O Wet Paint é a lei do Rebelle já calibrada (presets, miniaturas): **nada de mudar o traço de hoje sem o
  portão ao bit ou a prancha aprovada**.
- A escala das cerdas é em PÍXEIS — se virar escala da ponta (como os outros procedurais), muda todo preset;
  recusado por desenho, salvo medida que o justifique.
- ⛔ Não reconstruir o que a W2.4 decidiu (o Grain substitui as cerdas) sem a medição D7.

## §5 — Decisões do dono

- **Bristles também no Digital/Impasto** — o plano diz SIM (o Shape define o pincel igual em toda mídia).
- **Biblioteca de pontas em PNG** (D5) — depois desta onda.

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

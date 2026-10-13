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

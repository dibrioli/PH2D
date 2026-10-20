# 04 — A BANCADA do EXAME do pincel (JANELA 1, entrega D)

> Leitor: quem entrega um pincel (janelas 2–5). A bancada é o §3 da [MISSÃO](00_MISSAO.md) em comandos: a BATERIA de
> gestos ([`ferramentas/bateria.json`](ferramentas/bateria.json), a fonte única — os mesmos pontos que o Rebelle, o
> Krita e o MyPaint recebem) pintada pelos NOSSOS motores, a régua contra o oráculo, a matriz contra a biblioteca
> inteira, as pranchas (legível e cega) e o custo a 60 Hz. ⚠️ Régua verde é condição NECESSÁRIA, nunca suficiente
> ([P53](../../../project-memory/feedback_a_brush_that_passes_the_ruler_can_still_be_useless.md)): a prancha é OLHADA e
> a [revisão cega](05_REVISAO_CEGA.md) é feita.

## §0 — Os comandos

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
F=docs/Painter/Pinceis/ferramentas
# o exame de um pincel contra um oráculo (prefixo numa pasta de muitos presets, ou uma pasta só dele)
bash $F/exame.sh "Oil Bristle" docs/Painter/Pinceis/fixtures/rebelle_bateria/OIL_AND_ACRYLIC/bristle_bristle_flat
bash $F/exame.sh "Graphite 8B" docs/Painter/Pinceis/fixtures/rebelle_bateria/PENCIL/graphite_pencil_8b
bash $F/exame.sh caminho/do/Novo.ph2d-brush.png <oráculo>     # um candidato que ainda não está na fábrica
bash $F/exame.sh meio:Digital                                  # os meios sem presets em ficheiro (§2)
bash $F/exame.sh --matriz                                      # a biblioteca + os oráculos → matriz/ (vizinhos.md, oraculos.md, CSVs)
bash $F/exame.sh --custo "Oil Bristle" 5,40,200                # ms/quadro e memória pico
```

Saídas: `pranchas/<slug>.png` (rotulada) · `pranchas/cegas/<slug>.png` + `pranchas/chaves/<slug>.json` ·
`exames/<slug>_regua.{txt,json}` · `exames/<slug>_custo.txt` · `matriz/`. Trabalho e cache em `/tmp/ph2d_exame`
(`PH2D_EXAME_DIR`). A biblioteca pinta-se uma vez (98 s para 31 pincéis × 3 cenas × 2 fotos, 8 fios) e fica em cache
pela chave (bateria · ficheiros de preset · `HEAD` · diff de `crates/`). Os que já saíram da fábrica entram por
`PH2D_EXAME_EXTRA=<raiz>` (`git show d9e85a11c4~:assets/brushes/WetPainter/Oil/Flat/Filbert.ph2d-brush.png > …`) — a
calibração do limiar precisa deles (§4).

| peça | ficheiro |
|---|---|
| a pintura (Rust, `#[ignore]`) | [`tests/it/exame_bancada.rs`](../../../crates/ph2d-tool-painter/tests/it/exame_bancada.rs): `exame_pinta`, `exame_custo` |
| o encanamento | [`ferramentas/exame.sh`](ferramentas/exame.sh) |
| a régua | [`ferramentas/exame_regua.py`](ferramentas/exame_regua.py) (`mede`, `compara`) |
| a matriz | [`ferramentas/exame_matriz.py`](ferramentas/exame_matriz.py) |
| a prancha e a cega | [`ferramentas/exame_prancha.py`](ferramentas/exame_prancha.py) |
| o custo | [`ferramentas/exame_custo.py`](ferramentas/exame_custo.py) |
| a cena antiga dos 5 gestos (só validação) | [`ferramentas/legado_5gestos.py`](ferramentas/legado_5gestos.py) → `.json` |

## §1 — A pintura (`exame_pinta`)

`PH2D_EXAME_PRESETS="a|b|…"` (nome de fábrica · caminho de `.ph2d-brush.png` · `meio:<X>` · `todos`) e
`PH2D_EXAME_OUT`; perfil `smoke`, pela fatia (`ph2d-run`). Cada op do `bateria.json` tal como está: os pontos EXACTOS
(já reamostrados), um evento cada, 8 ms entre eventos (o `sonda_gestos::gesto` de sempre); o diâmetro `size_px`
⇒ `set_brush_size_px(size_px · escala / 2)` (o slider é o RAIO; a largura que se vê é 2 · raio, a promessa da W2 —
medida: 5/40/200 → 5/40/200 ± 1 px na biblioteca); `simula N` = N tiques de 60 Hz; `foto` = o `run_full`.
Relógio da água FIXO e semente 0: a mesma tinta duas vezes (medido: fnv iguais em duas corridas).

⚠️ **Uma corrida por FOTO.** O `run_full` é o Apply e LEVA a tela (`trait_impls_raster.rs`: `mem::replace` do
`canvas_rgba`) — a 2.ª chamada devolvia 0 bytes. A foto B repinta a cena até ao fim; o motor é determinístico.

## §2 — Os quatro meios: o que `seca` e `simula` querem dizer

| meio | pincel | `simula N` | `seca` |
|---|---|---|---|
| Wet Paint | os presets de ficheiro (todos os 28 são Wet Paint) | N quadros de água a 60 Hz | o botão **Dry Canvas** (`PAINTER_WETPAINT_DRYCANVAS` → `dry_canvas_now`: o `DRY` do Rebelle) + 1 tique (o quadro em que o produto recompõe) |
| Watercolor | `meio:Watercolor` (receita em código) | idem (a lavagem seca no tempo dela) | **Dry now** (`PAINTER_WATERCOLOR_DRY_NOW`) |
| Impasto | `meio:Impasto` | sem efeito (não há água) | nada: o relevo não seca |
| Digital | `meio:Digital` | sem efeito | nada |

Desde a L1 (J2, 20/10) os quatro meios têm presets em ficheiro: a bancada pinta o `.ph2d-brush.png` de QUALQUER
meio no meio do ficheiro (`preset_aplicar`; um `media` desconhecido reprova a corrida) — três exemplos em
`fixtures/presets_exemplo/` (Digital, Watercolor, Impasto). O `meio:<X>` + uma receita em código (`RECEITAS_DE_MEIO`)
continua para o pincel de fábrica de cada meio. Medido sobre a biblioteca: A ≠ B
(o seca muda a imagem) em 11 dos 31 — os Wet/Ink/Dip Pen/Gouache/Flat Oily/Knife; nos lápis, pastéis, óleo e
marcadores A = B ao bit depois dos 240 quadros (a tinta já assentou). No Digital, a faixa «seca» da cena `sobre` é
igual à «molhada» (por desenho do meio, não da bancada).

## §3 — A régua (`exame_regua.py`)

Cada traço é ENDIREITADO: a polilinha exacta do `bateria.json` a 1 px de arco, e em cada ponto a cobertura
(`máx |px − papel| / 255`, a do `mede_traco.py`; o papel é a mediana da moldura de 6 px) amostrada na NORMAL (±H,
bilinear; H = o tamanho, e na espiral metade do passo entre braços). Métricas por traço: largura (a ¼ — a que se vê
— e a ½), cobertura e opacidade do miolo, perfil transversal (33 pts), rugosidade das duas bordas, textura (resíduo
5×5, como o `mede_traco`), variância local, período dominante e a fração dele, cv ao longo, cor média, pontas (5 %
de cada ponta ÷ a mediana) e o que passa do ponto final, a resposta à pressão (largura e cobertura por decil de
pressão ÷ as de p ≥ 0,9) e, sobre outra tinta, o VERMELHO por desmistura das absorvâncias (azul × vermelho) em cima
da faixa e nos 70 px depois (o arrasto), e a cor em cima. As amostras perto de outro traço saem das métricas limpas.
`compara` lê `<cena>_<A|B>` dos dois lados (`.png`/`.ppm`, com prefixo, ou uma foto por cena para oráculos sem água)
e escreve a tabela (oráculo → nosso, ✓ dentro de 3 ruídos) e o JSON. O RUÍDO: o da promessa (largura ± 2 px), o do
P53 (cobertura 0,012), os outros PROVISÓRIOS — e o `compara` mede o do próprio oráculo (o mesmo recto a 40 em três
cenas: recto_lento · recto_40 · papel) e usa o maior; só vale quando o oráculo tiver as três cenas.

A **escala**: com oráculo, a bancada pinta o nosso e o vizinho à largura MEDIDA do oráculo (05 §4.1): escala = a
largura do `recto_lento` dele ÷ a nossa a escala 1 (Graphite 8B 0,825 — o Rebelle pinta 33 px a «40»; Oil Bristle
1,05). `PH2D_EXAME_ESCALA=1` força o tamanho nominal.

## §4 — A matriz (`exame_matriz.py`) — ⭐ o achado

⚠️ **A matriz é TRIAGEM, nunca veredito.** Ela diz onde olhar; o veredito de um pincel é a PRANCHA lado a lado com o
oráculo + a REVISÃO CEGA ([05](05_REVISAO_CEGA.md)). Um «claramente longe» não aprova nada; um «perto» manda olhar.

**O vector** (457 traços em 5 grupos: forma · tinta · textura · dinâmica · mistura; a largura entra RELATIVA ao
próprio recto, mais uma absoluta). **A população**: os nossos, os **228** presets do Rebelle que pintam as três cenas
([`fixtures/rebelle_bateria/`](fixtures/rebelle_bateria/README.md); as ferramentas sem tinta e os Effect ficam de
fora) e os **34** do ArtRage (só a foto A: B ≡ A). **A escala é CONGELADA** em [`matriz/escala.json`](matriz/escala.json):
cada traço ÷ max(1,4826 · MAD sobre os 228 do Rebelle, o CHÃO da sua família), diferença saturada a 6, RMS por grupo,
média dos 5 grupos. Pincel novo não mexe na régua dos outros; `exame_matriz.py --recalibra` refá-la. O recto de 200 px
dos 31 presets cujo Max Size é menor fica fora da identidade deles (o Rebelle pintou-o ao max: não é o pincel a 200).
Medições completas: [`matriz/oraculos.md`](matriz/oraculos.md).

**(c) O ruído do oráculo** — o MESMO recto a 40 pintado duas vezes (recto_lento da cena `gestos` × recto_40 da
`tamanhos`), sobre os 228: cobertura p50 0,006 · p75 0,023 · p90 0,077; textura 0,71 · 2,9 · 10,3 %; rugosidade
0,09 · 0,35 · 0,90 px; largura 0 · 1 · 5 px. O **chão = o p75**. ⛔ O p90 foi recusado: a cauda é o pincel, não a
medida — a carga que acaba no recto mais comprido (880 × 500 px: o Gouache Liner 2 passa de 30 a 21 px) e o
espalhamento sorteado (Grunge/Scatter, textura 12 → 120 %); como chão, a textura de 10 % apagaria a textura de quase
todos os lápis (o 8B tem 6,4 %). ⛔ O p50 também: 62 % das larguras e 47 % das pontas repetem AO PIXEL (a régua é
inteira), o chão seria 0 e uma ponta de 0,94 contra 1,00 valia 13 desvios. ⭐ Em distância, só com os traços do recto:
o mesmo preset repintado dá p50 **0,33** (p90 1,50) e um preset contra o vizinho mais perto **0,65** (p90 1,49) — **o
recto sozinho mal separa dois pincéis do Rebelle do mesmo pincel repintado**: é a lição do P53 (a régua do recto
aprovou 11) em números; a identidade mora nos outros gestos.

**(a) A distância ao vizinho mais perto dentro do Rebelle** (p10 · p25 · p50 · p75 · p90), fora da FAMÍLIA (a série
unida aos duplicados ao byte): **total 0,60 · 0,72 · 0,91 · 1,19 · 1,53** · PENCIL 0,62 · 0,67 · 0,84 · 1,06 · 1,22 ·
OIL_AND_ACRYLIC 0,46 · 0,63 · 0,85 · 1,13 · 1,50 · WATERCOLOR 0,72 · 0,82 · 1,13 · 1,52 · 1,98 · PASTEL 0,74 · 0,84 ·
1,00 · 1,30 · 1,41 · INK_PEN 0,91 · 0,95 · 1,20 · 1,38 · 1,43. ArtRage (dentro dele): 0,34 · 0,76 · 0,86 · 1,08 ·
1,90. ⭐ **O Rebelle vende duplicados**: «Colored Pencil/2H·HB·2B·4B·8B Color» e «Graphite Pencil/2H·HB·2B·4B·8B» dão
fotos IGUAIS AO BYTE com a cor da bateria (sha1 conferido): só a cor de omissão os separa (d 0,000).

**(b) As séries graduadas do Rebelle** (77 passos entre membros vizinhos): p10 · p50 · p90 = 0,43 · 1,22 · 2,05. A
dureza do grafite: **2H → H 0,17 · H → HB 0,13 · HB → 2B 0,41 · 2B → 4B 0,43 · 4B → 8B 0,45**. Olhado nas fotos:
2H, H e HB são o mesmo traço; a partir do 2B o grão abre e do 4B escurece. O eixo NÃO é monótono numa só medida (a
cobertura do recto: 0,66 · 0,64 · 0,63 · 0,61 · 0,74 · 0,87 — o 2B é o mais claro e o mais granulado).

**⭐ A LEI** (o que `vizinhos.md` aplica):
1. **«Claramente longe» = d ≥ 0,596** = o **p10** da distância ao vizinho mais perto dentro do Rebelle, fora da
   família. Porquê: é o menor afastamento que um fabricante profissional dá a dois pincéis que vende como
   DIFERENTES (90 % dos 228 estão mais longe que isto do seu vizinho); fica 1,8 × acima da mediana do ruído do recto
   repintado (0,33). O p10 e não o mínimo: o mínimo do Rebelle são os seus próprios quase-repetidos (Metallic Flat 2 ~
   Round 2, 0,39).
2. **Uma série graduada entra como FAMÍLIA** (`FAMILIAS` no script; hoje Graphite 2H · HB · 2B · 8B): (i) fora dela,
   cada membro tem de estar ≥ 0,596 de tudo; (ii) dentro, cada passo entre membros vizinhos ≥ **0,40** — o menor passo
   da série do Rebelle que a prancha MOSTRA (HB → 2B 0,41; os de 0,13–0,17 não se vêem); (iii) cada membro tem a SUA
   origem como o 1.º do Rebelle (abaixo) — o eixo é o do oráculo, mesmo não monótono.
3. **Fidelidade** (a coluna «origem» de `vizinhos.md`): o preset do Rebelle de que o nosso nasce tem de ser o seu
   **1.º entre os 228** (um duplicado ao byte da origem empata com ela). Os nossos são comparados com os vizinhos de
   ORÁCULO na mesma tabela ([`matriz/matriz_oraculos.csv`](matriz/matriz_oraculos.csv): 31 × 262).
4. **Recusados como âncora** (a lei de 19/10): o Filbert «nada novo» × o Soft Pastel (0,74) — o veredito do dono não
   foi dado contra ele, e um par só não calibra; e o par da receita Filbert × Acrylic Round (1,90). A matriz NOVA
   explica o «nada novo»: o vizinho de oráculo do Filbert é o **Digital/Round Hard** do Rebelle (0,507, abaixo do
   limiar) e a origem dele (Flat/Filbert) é só a 27.ª — era um redondo digital com outro nome.

**O que a lei diz da fábrica (28):** **13 perto** — Colored 4B ~ Oil Pastel 0,299 · Dip Pen ~ Ink Bullet 0,359 ·
Soft Pastel ~ Wax Scrape 0,362 · Graphite 8B ~ Oil Pastel 0,385 · Marker Liner ~ Ink Bullet 0,434 · Graphite HB ~
Colored 4B 0,443 · Marker Bullet ~ Marker Liner 0,452 · Graphite 2H ~ Colored 4B 0,463 · Graphite 2B ~ Colored 4B
0,502 · Square Hard ~ Oil Pastel 0,533 (+ os pares recíprocos Ink Bullet, Oil Pastel, Wax Scrape); na série, o
passo **2H → HB = 0,37 é invisível** (como o do Rebelle). **Fidelidade: a origem é o 1.º em 8 de 29** (Airbrush,
Charcoal, Dip Pen, Gouache Flat, Graphite 2H, Ink Brush, Marker Bullet, Wet Flat); mediana da posição 4; os piores:
Acrylic Round 73.º, Express Oil 58.º, Marker Liner 49.º, Dabber 45.º, Graphite 8B 20.º. No 8B os traços que o afastam da
origem são os da prancha (§6): a pressão (pontas das rampas, 6 desvios), o recto de 5 px (cobertura 0,32 × 0,80).

## §5 — As pranchas

Por cena e foto (A molhada, B seca): ORÁCULO · NOSSO · VIZINHO (o 1.º da matriz, pintado à mesma escala), a meia
escala, 256 cores (≈ 0,5–0,7 MB). A cega sorteia a ordem POR LINHA e só leva as linhas com as três fotos (um «sem
foto» denunciaria o lado em falta). Protocolo da revisão: [05_REVISAO_CEGA.md](05_REVISAO_CEGA.md).

## §6 — A prova de ponta a ponta (19/10): Graphite 8B e Oil Bristle contra o Rebelle

Oráculo: as fotos da bateria que o agente A tirou ([`fixtures/rebelle_bateria/`](fixtures/rebelle_bateria/)) — à
hora desta prova só a cena `gestos` (A e B); `tamanhos` e `sobre` aparecem «sem foto» na prancha rotulada e saem da
cega. Por isso também corri as fotos ANTIGAS dos 5 gestos (`~/Apps/rebelle/oraculo/c_ws_pencil_*`, `c_ws_oil_*`) pela
cena `legado_5gestos.json` — ⚠️ só para validar o encanamento (a molhada, a seca, o cruzamento vermelho × azul): é
outra cena, não a bateria do exame.

**[Graphite 8B](pranchas/graphite_8b.png)** × «Graphite Pencil/8B» (escala 0,825; régua 140/168 ✓; vizinho Colored 4B
d 0,435 — PERTO): o recto, a curva, a espiral e os verticais batem (largura 33/33, borda 1,47/1,47 px, perfil Δ0,05);
a cobertura é 0,05 abaixo (0,82 × 0,87) e o cv o dobro (o nosso grão ondula mais ao longo). ⭐ A diferença que se VÊ:
**a pressão**. No Rebelle a rampa 0→1 começa com a largura cheia e um esbatido transparente (a pressão é opacidade);
no nosso a rampa afunila em ponta (a pressão é largura): largura 26 → 17, pontas da rampa 0,15→1→0,15 a 0 contra
0,35 (o Rebelle esbate, nós estreitamos). Na prancha o vizinho (Colored 4B) é indistinguível do nosso.

**[Oil Bristle](pranchas/oil_bristle.png)** × «Bristle/Bristle Flat» (escala 1,05; régua 148/168 ✓; vizinho Soft Pastel
d 1,233 — longe): rectos, curva e espiral batem (largura 42/42, cobertura 0,92/0,91, a espiral com o mesmo «olho» do
pincel chato a girar); ⭐ o vertical FINO (13 px contra 42) é do Rebelle também — o pincel chato dele não roda com o
traço (o defeito «lâmina fixa» do P53 era só das lâminas). A diferença que se vê: **a pressão** outra vez (rampa
37 → 22 px; pontas 0,875 → 0,375 — o Rebelle mantém a largura e abre as cerdas em fiapos secos na pressão baixa, o
nosso afina) e **as pontas**: o Rebelle termina cada traço em fiapos irregulares de cerda, o nosso em ponta limpa;
as nossas riscas são mais claras e mais regulares (cor do vertical (21,113,199) × (14,100,180)).

Legado (molhada e seca, o cruzamento): Graphite 8B 136/156, Oil Bristle 140/156; ⭐ no óleo, a cor por cima do
cruzamento seco difere — o Rebelle (94,67,115) mistura o azul por baixo do vermelho, o nosso (146,48,91) mostra mais
vermelho (vermelho_em_cima 0,42 × 0,62).

**A revisão cega** (um agente novo, só com a cega de 2 linhas do commit `39967873bd`): ver §6.1. A cega de HOJE tem
as 6 linhas e outra ordem sorteada (a chave nova em `pranchas/chaves/`): a próxima revisão usa-a.

### §6.0 — O reexame com as três cenas do Rebelle (20/10)

As fotos dos 281 presets ficaram completas ([`fixtures/rebelle_bateria/`](fixtures/rebelle_bateria/README.md)); as
duas pranchas foram refeitas (rotulada e cega, 6 linhas cada). Régua: Graphite 8B **334/420**, Oil Bristle
**372/420** dentro de 3 ruídos. Os vizinhos pela matriz nova: 8B ~ Oil Pastel 0,385 (PERTO), Oil Bristle ~ Soft
Pastel 1,177 (longe). O que as cenas novas mostram:

- **Graphite 8B · tamanhos**: o recto de **5 px** do Rebelle é uma linha escura e cheia (cobertura 0,80, cor
  (51,119,188)); o nosso é um pontilhado claro e falhado (0,32, (161,193,224), textura 24 × 9 %, cv 0,30 × 0,07) — o
  grão do nosso lápis não escala para baixo. O de **200 px** não é comparável: o Max Size do 8B no Rebelle é 80 e ele
  pintou ao max (68 px, de borda ondulada e miolo cheio); o nosso pinta 166 px de nuvem granulada. A 40, igual.
- **Graphite 8B · sobre**: o azul por cima do vermelho — no Rebelle o lápis cobre (vermelho em cima 0,10–0,14), no
  nosso o vermelho atravessa o dobro (0,23–0,26) e a cor em cima é mais roxa ((71,81,141) × (54,94,161)); a faixa
  vermelha do Rebelle é mais larga de borda irregular, a nossa mais lisa. Molhada = seca nos dois (lápis).
- **Oil Bristle · tamanhos**: o recto de 5 px do Rebelle tem 8 px e é chapado (cobertura 0,89); o nosso 4 px e
  claro (0,68, (79,137,195)). O de 200 px: o Rebelle corta as pontas RETAS e esfiapadas e começa mais à esquerda; o
  nosso tem as pontas em cápsula e uma banda clara de brilho a meio do comprimento.
- **Oil Bristle · sobre** ⭐: o azul do Rebelle passa pela faixa MOLHADA com a mesma largura e pega vermelho (0,51–0,60
  de vermelho em cima, cor (122,46,84)); o nosso **incha numa lente** mais clara exactamente em cima da faixa molhada
  (e não na seca) e pega metade do vermelho (0,34, (88,82,143)). Na faixa seca os dois batem (0,54 × 0,55). Depois do
  `seca` nenhum dos dois muda.

### §6.1 — A resposta do revisor e o cruzamento

Briefing do 05 §2 colado tal e qual num agente `general-purpose` novo (52 s, 8 leituras). Chave: gestos A
`1 nosso · 2 oráculo · 3 vizinho`; gestos B `1 vizinho · 2 nosso · 3 oráculo`.

| linha | o revisor disse | cruzado |
|---|---|---|
| gestos A | original **2** (confiança média), imitação 1, o 3 «claramente outro» (spray granulado) | oráculo ✓ · imitação = o NOSSO ✓ · o vizinho é outro ✓ |
| gestos B | original **3** (média), imitação 2, o 1 «claramente outro» | oráculo ✓ · imitação = o NOSSO ✓ · vizinho outro ✓ |

Ele achou o oráculo nas duas linhas ⇒ o Oil Bristle é distinguível do Rebelle. O RESUMO dele, por gravidade:
**(1) a pressão** — no original a largura quase não muda e a rampa 0→1 só começa a ~1/8 do caminho, em franjas de
cerdas; no nosso a pressão faz a largura (agulha na 0→1, fuso na 0,15→1→0,15); **(2) as pontas** — cortes retos
esfiapados «em vassoura» × tampas redondas de cápsula em todos os gestos; **(3) a superfície** — o nosso tem relevo
de cilindro (bordas escuras, brilhos claros quase ciano no centro), azul mais claro e saturado e borda limpa; o
original é chapado e mais fundo, riscas irregulares, borda serrilhada com falhas brancas. Também: o rápido do original
abre-se mais que o lento (o nosso igual); molhado × seco: nenhuma mudança em nenhum dos dois. As três batem com a
régua (rampa 37 → 22 px, pontas 0,875 → 0,375, cor do vertical) — e a (2) e a (3) a régua NÃO marcou como ✗ (a
ponta «alem» e a cor estão dentro de 3 ruídos provisórios): é por isso que a revisão cega existe. Pela regra do 05
§3, o Oil Bristle de hoje REPROVA no §3.5 — as três ficam na fila da janela que o refizer (pressão → carga/opacidade
em vez de largura; ponta de cerdas soltas; o brilho do relevo).

## §7 — O custo (`--custo`, perfil smoke, ritmo de 60 Hz na parede, 3 eventos por quadro, a água do ecrã)

| pincel | px | ms/quadro a pintar: mediana · p95 | quadros > 16,7 ms | parado: mediana · p95 | núcleos | RSS pico |
|---|---|---|---|---|---|---|
| Graphite 8B | 5 | 0,26 · 0,45 | 1/475 | 1,00 · 3,84 | 0,09 | 68 MiB |
| Graphite 8B | 40 | 0,92 · 1,80 | 1/475 | 0,79 · 3,51 | 0,13 | 76 MiB |
| Graphite 8B | 200 | 0,00 · 4,68 | 1/475 | 1,11 · 3,87 | 0,49 | 92 MiB |
| Oil Bristle | 5 | 0,32 · 0,59 | 1/475 | 1,65 · 3,97 | 0,15 | 74 MiB |
| Oil Bristle | 40 | 1,13 · 1,81 | 1/475 | 1,95 · 4,71 | 0,48 | 80 MiB |
| Oil Bristle | 200 | 2,62 · 6,07 | 8/475 | 3,40 · 7,54 | 2,07 | 96 MiB |

O quadro > 16,7 ms que todos têm é o 1.º (36–38 ms: a sessão nasce no primeiro carimbo). O Graphite 8B a 200 px tem a
mediana 0 porque carimba a cada ~2 quadros (o espaçamento cresce com o tamanho). `núcleos` = CPU do processo ÷ parede
(a água corre num fio próprio). Load 4,5–7,4 durante a medição (outros agentes): o p95 é o número a ler, o `pior` não.
Um processo por tamanho ⇒ o RSS é deste pincel a este tamanho.

## §8 — Limites e o que fica para as janelas seguintes

- O chão da escala da matriz é o ruído MEDIDO do Rebelle nas famílias do recto; o da pressão, da mistura e das
  razões continua provisório (0,06 · 0,03 · 0,05): o recto repetido não os mede. Os ruídos da tabela da RÉGUA
  (`exame_regua.RUIDO`, o ✓/✗ do `compara`) também são os provisórios — trocá-los pelos da tabela (c) é a próxima.
- A fidelidade pela matriz (a origem como 1.º dos 228) compara o NOSSO a escala 1 com o Rebelle ao tamanho dele; o
  exame de um pincel (as pranchas) é à largura medida. A identidade usa a largura relativa ao próprio recto, mais
  uma absoluta.
- O vector pesa os 5 grupos por igual; não sabe o tamanho de fábrica de cada preset (compara-os à mesma largura e
  cor, de propósito: o artista escolhe o pincel pelo carácter). As famílias (`FAMILIAS`) escrevem-se à mão.
- A caneta real (inclinação, rotação) não está na bateria (pressão só); o custo usa o rato sintético.

## ⛔ Recusas MEDIDAS

| Recusa | Medido |
|---|---|
| Duas fotos por `run_full` na mesma corrida | o `run_full` leva a tela: a 2.ª sai com 0 bytes |
| O custo com o relógio fixo | a água fixa corre no fio do pincel e lê-se como custo dele (p95 parado 29 ms a 200 px); a do ecrã: 7,5 ms |
| O custo sem o ritmo de 60 Hz | 466 quadros em 1,35 s: a água do ecrã simulava 1,35 s do que o artista pinta em 8 |
| Âncora de «igual» = o par NASCE_DE (Filbert × Acrylic Round) | d 1,903 (§4): o limiar daria «nada novo» a 30 de 31 |
| O limiar 0,992 de 19/10 (Filbert ~ Soft Pastel 0,741 × os «bizarros» 1,33) | um par só, e o «nada novo» não foi dado contra o Soft Pastel; a escala era a da nossa biblioteca (mudava com cada pincel novo: 0,976 com o `meio:Digital`) |
| O chão da escala = o p90 do recto repetido | a cauda é a carga que acaba no recto mais comprido e o espalhamento sorteado; textura 10,3 % apagaria a textura dos lápis |
| O chão = o p50 | 62 % das larguras e 47 % das pontas repetem ao pixel: chão 0, uma ponta 0,94 × 1,00 valia 13 desvios (a forma passava a 2–3 em todos os pares) |
| Lei = 3 × a mediana do ruído do recto | o ruído é só dos traços do recto (0,33) e eles mal separam pincéis (0,65 ao vizinho): 3 × daria 1,0 e «perto» a quase tudo |
| O recto de 200 px dos 31 presets com Max Size < 200 na identidade deles | o Rebelle pinta-o ao max (o 8B a 80): era o tecto, não o pincel — o 8B nosso ficava 6 desvios longe só por isso |

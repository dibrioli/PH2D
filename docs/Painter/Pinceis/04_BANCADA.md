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
bash $F/exame.sh --matriz                                      # a biblioteca inteira → matriz/{matriz.csv,vizinhos.md}
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

Os meios Digital/Watercolor/Impasto ainda não têm presets em ficheiro (03_MOTORES): a bancada aceita-os como
`meio:<X>` + uma receita em código (`RECEITAS_DE_MEIO`, uma linha por pincel novo). Medido sobre a biblioteca: A ≠ B
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

Vector de identidade de 457 traços em 5 grupos (forma · tinta · textura · dinâmica · mistura), cada traço ÷ o
espalhamento robusto da biblioteca (1,4826 · MAD, chão = o ruído da régua), diferença saturada a 6, RMS por grupo,
média dos grupos. Saída: [`matriz/matriz.csv`](matriz/matriz.csv) e [`matriz/vizinhos.md`](matriz/vizinhos.md)
(top-3 por pincel e a distância por grupo do 1.º).

**Calibração pelos vereditos do dono** (19/10), cada um contra o vizinho entre os que FICARAM: Filbert ~ Soft
Pastel 0,741 («nada novo» ⇒ igual) · Flat Oily ~ Soft Pastel 1,338 e Knife Smooth ~ Soft Pastel 1,328 («bizarros»
⇒ diferentes) ⇒ **limiar = 0,992** (média geométrica). ⚠️ Medido e recusado como âncora: o par da receita (Filbert ×
Acrylic Round, o NASCE_DE) dá **1,903** — à mesma largura e cor a Acrylic Round pinta um feixe de fios transparente
(cobertura 0,55) e o Filbert um redondo liso (0,91); o «nada novo» é o redondo liso que a biblioteca já tinha. Os 8
do P53 que o dono aceitou também NÃO são âncora de «distinto»: «OK» foi «serve».

**O achado: 22 dos 28 pincéis de fábrica têm o vizinho abaixo do limiar** — à mesma largura e cor, a biblioteca é
quase toda feita de pares: Dip Pen ~ Ink Bullet 0,263 · Colored 4B ~ Oil Pastel 0,293 · Marker Liner ~ Ink Bullet
0,343 · Wax Scrape ~ Oil Pastel 0,374 · Graphite 2H ~ HB 0,386 · Graphite 8B ~ Colored 4B 0,435 · … (a lista em
`vizinhos.md`). Olhado nas pranchas: os lápis 2H/HB/8B, o Colored 4B, o Oil Pastel e o Wax Scrape são o mesmo
grão macio a azul; Dip Pen e Ink Bullet o mesmo redondo chapado. Só 6 ficam longe: Wet Drip (2,60), Acrylic Round,
Airbrush, Wet Dry Brush, Scratch, Oil Bristle (1,17–1,36). É a fila de diferenciação das janelas 2–4. ⚠️ A escala
de cada traço é a da população: pincel novo entra ⇒ o limiar recalcula-se (com o `meio:Digital` na vista, 0,976).
O exame de um pincel corre a matriz com ele dentro.

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

**A revisão cega** (um agente novo, só com [`pranchas/cegas/oil_bristle.png`](pranchas/cegas/oil_bristle.png)): ver
§6.1.

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

- As cenas `tamanhos` e `sobre` do Rebelle ainda não existiam: reexame dos dois quando o agente A as tirar (os
  comandos do §0 tal como estão; o ruído do próprio oráculo passa a valer).
- O ruído de rugosidade, textura, cv, perfil, pressão e mistura é provisório: calibrar sobre o oráculo (as três
  cenas) e sobre fotos repetidas (o mesmo preset duas vezes).
- O vector da matriz pesa os 5 grupos por igual; não sabe o tamanho de fábrica de cada preset (compara-os à mesma
  largura e cor, de propósito: o artista escolhe o pincel pelo carácter).
- A caneta real (inclinação, rotação) não está na bateria (pressão só); o custo usa o rato sintético.

## ⛔ Recusas MEDIDAS

| Recusa | Medido |
|---|---|
| Duas fotos por `run_full` na mesma corrida | o `run_full` leva a tela: a 2.ª sai com 0 bytes |
| O custo com o relógio fixo | a água fixa corre no fio do pincel e lê-se como custo dele (p95 parado 29 ms a 200 px); a do ecrã: 7,5 ms |
| O custo sem o ritmo de 60 Hz | 466 quadros em 1,35 s: a água do ecrã simulava 1,35 s do que o artista pinta em 8 |
| Âncora de «igual» = o par NASCE_DE (Filbert × Acrylic Round) | d 1,903 (§4): o limiar daria «nada novo» a 30 de 31 |

# PH2D × Clip Studio Paint — linhas e mistura (08/11)

Pedido do dono: «estudo comparativo com fotos de traços complexos e múltiplos, nossos pincéis e nossa mescla de cor
contra o Clip Studio». Nenhum código de produto mudou. [M] = medido nas imagens · [D] = descrito a olho.

## v2 (08/11) — a que vale

**Por que a v1 está errada:** (1) a sonda passou o «Brush Size» do CSP (um DIÂMETRO) ao `set_brush_size_px`, que é
um RAIO (`brush_core_settings.rs:154` grava `radius_px`; o Size do painel é `size_px × DIAMETRO_POR_RAIO = 2`,
`paint_wetpaint_presets.rs:36,88`) — por isso «8 px deu 16 px»: não há defeito de produto, o Size 8 do painel é
um fio de 8. (2) A mistura arrastou BRANCO, que não é o caso do dono, e cada app pintou os seus blocos (os nossos
saíram largos e em onda).

**Como (v2):** `material_2026-11-08/cena_v2.py` gera uma tela (`base_csp.png`, 1300×650) com os MESMOS blocos secos
em píxeis (azul `0046BE`, amarelo `F0C800`, vermelho `C81414`) que o CSP abre e que a sonda lê
(`base_celula.rgba`); arrastos: 4 vai-e-volta + um X **amarelo sobre o azul**, o mesmo **azul sobre o amarelo** e
**azul sobre o vermelho**, e uma régua azul em branco para medir a largura. CSP 1.13.2 numa cópia do prefixo, Xvfb
`:76`, zoom 100 %, xdotool (pressão constante); cor por Alt+clique nas amostras da tela; tamanho 30 nos dois.
Sonda: `sonda_csp_v2_linhas` / `sonda_csp_v2_mistura` (`PH2D_CSP_R=<pincel>=<raio>;…` calibra o raio).
Folhas: `compoe_v2.py` (mede e monta). As células CSP das linhas vêm da v1 (o lado CSP estava certo).

Folhas: [A v2 1](handoffs/material_2026-11-08/comparacao_csp_A_linhas_1_v2.png) ·
[A v2 2](handoffs/material_2026-11-08/comparacao_csp_A_linhas_2_v2.png) ·
[B v2 1](handoffs/material_2026-11-08/comparacao_csp_B_mistura_1_v2.png) ·
[B v2 2](handoffs/material_2026-11-08/comparacao_csp_B_mistura_2_v2.png) ·
[tela CSP v2](handoffs/material_2026-11-08/comparacao_csp_tela_inteira_csp_v2.png)

### Larguras [M]

Linhas: extensão vertical no cume do S (luminância < 160), mínimo nas colunas 272–288. Mistura: a régua azul.

| par | CSP (Size) → largura | PH2D Size d/2→raio: largura | PH2D na folha |
|---|---|---|---|
| G-pen ↔ G-Pen | 8 → 7 | r4 → 8 | Size 7 → **7** |
| Turnip | 8 → 7 | r4 → 8 | Size 7 → **7** |
| Pencil ↔ Sketch Pencil | 8 → 3 | r4 → 5 | Size 6 → **4** |
| Charcoal ↔ Charcoal Stick | 16 → 11 | r8 → 7 | Size 20 → **11** |
| Airbrush Soft ↔ Soft Airbrush | 30 → 19 | r15 → 13 | Size 44 → **19** |
| Round watercolor ↔ Round Watercolor / Watercolor Basic | 16 → 16 | r8 → 12 | Size 18 → **16 / 17** |
| Round mixing ↔ Hard Round + Pigment mix | 30 → 30 | r15 → **30** | igual |
| Oil paint ↔ Watercolor Basic (Pigment 1) | 30 → 23 | r15 → **24** | igual |
| Thick oil paint ↔ Dabber | 30 → 27 | r15 → **26** | igual |
| Gouache ↔ Composite do dono | 30 → 23 | r15 → **0** (não pinta) | igual |

Com Size = diâmetro, as canetas e os pincéis de mistura já casam (±1 px). Os de borda macia/grão (lápis, carvão,
aerógrafo) precisam de Size maior no PH2D para a MESMA largura visível: o nosso perfil cai mais cedo.

### A. Linhas (largura igual) [D]

- **G-Pen / Turnip:** com a largura certa o fio do PH2D é limpo, liso e redondo como o do CSP — **empate** no
  corpo. Diferença só nas pontas: o nosso afina (hachura vira «gravatas», pontas do zig-zague em agulha); o CSP não.
- **Por que afinamos e o CSP não (explicado, não mudado):** a receita de fábrica (`tests/it/familias/canetas.rs`
  :70 e :90) liga o `taper` por COMPRIMENTO — G-Pen `start/end 1,5` diâmetros, Turnip `2,0`, ponta 0,15/0,1 — que é
  o Touch Taper do Procreate: independe da pressão de propósito (`spec.rs:275`: «o único caminho para uma linha
  caligráfica sem mesa»), logo afina também com o rato. A receita copiou o «Starting 20 / Ending 20, Specify length»
  do CSP, mas no CSP o alvo desse efeito, «Starting and ending», está em **None** (conferido no G-pen e no Turnip pen,
  e de novo depois de «Reset all settings to default» numa cópia descartável): os 20 px estão lá mas não se aplicam a
  nada — nem com rato nem com caneta (com caneta, a ponta fina vem só da pressão real). Ou seja, o CSP de fábrica
  NÃO afina por comprimento; o nosso sim.
- **Pencil ↔ Sketch Pencil:** na mesma largura o nosso é cinzento médio e uniforme; o do CSP é mais escuro com grão
  de papel serrilhado. O CSP lê-se mais «grafite»; o nosso já não tem as contas da v1 (eram do tamanho 2×).
- **Charcoal:** o CSP tem miolo preto contínuo e franja de pó; o nosso continua em «couve-flor» (manchas redondas
  com buracos). **CSP melhor.**
- **Airbrush:** forma igual; o nosso tem ruído de pó por dentro e escurece mais nos cruzamentos. Gosto.
- **Round Watercolor / Watercolor Basic:** mesmo com Size 18 a água do PH2D alastra e engole a forma (espiral vira
  disco); o CSP mantém o caminho com textura e escurece ao sobrepor. **Para linha, CSP muito melhor.**

### B. Mistura (cor sobre bloco seco) [D]

- **Round mixing / Gouache / Oil paint (CSP) × Pigment mix (PH2D):** o CSP **cobre**: amarelo sobre azul fica
  amarelo, com azul apanhado no início de cada passada que se esvai (o «color stretch»); azul sobre amarelo fica
  azul; mistura RGB (no Oil paint, amarelo sobre azul vira oliva/cinzento, nunca verde). O nosso Pigment mix
  **vela**: amarelo sobre azul = verde, azul sobre amarelo = verde, azul sobre vermelho = **quase preto
  (bordô-escuro)**; cor chapada, sem apanhar nem arrastar a cor de baixo, e os cruzamentos não variam. Para o caso do
  dono («arrastar uma cor por cima»), **o CSP faz o que o pintor espera de guache/óleo opaco**; o nosso faz o que a
  aquarela/velatura faria. O verde é «real», mas o azul sobre vermelho escurece demais e nada arrasta. Nota: o nosso
  Pigment mix trata o branco quase puro como papel (a v1 de branco «sumia» por isso).
- **Composite do dono** (Blur 1 · Smear 0,25 · Brush 0,10 Hardness 0,43, Flow 0,03, cor = a do arrasto, Size 30):
  **a cor do arrasto não aparece** (régua azul = 0 px): os blocos só ganham bordas desfocadas em onda e um leve
  borrão. Com Flow 0,03 × Strength 0,10 a camada Brush deposita ~0,3 % por dab — a receita é um desfoque. O Gouache
  do CSP cobre com estrias secas. **CSP melhor**; a receita só serve como esbatedor.
- **Watercolor Basic, Pigment mixing 1 × Oil paint:** o nosso mistura de verdade (azul+amarelo = verde-azulado,
  amarelo = verde-lima, azul+vermelho = vinho) com flores e bordas de água, mas **foge do caminho** (manchas além das
  passadas) e a régua sai em poça. O Oil paint do CSP fica no caminho, semitransparente, estriado. Bonito como
  aquarela; como «óleo» o CSP ganha.
- **Thick oil paint × Dabber:** ambos cobrem; o Dabber tem relevo e luz (o mais «matéria» de todos) mas não apanha
  nada da cor de baixo e as bordas são mais largas e em franja. O CSP apanha cor no início da passada. Empate de
  gosto; o nosso ganha em matéria, perde em arrasto.
- **Blend × Smear Basic:** o CSP funde (desfoque suave); o nosso desloca com borda nítida (liquify). Ferramentas
  diferentes. **Finger tip × Blender Soft:** o CSP arrasta e esbate a cor; o Blender Soft **não faz nada** sobre
  píxeis secos — **CSP melhor**.

### Veredito v2

- O «2× mais largo» era a sonda, não o produto. Com Size = diâmetro as canetas empatam no corpo; só as pontas
  diferem (taper por comprimento ligado na nossa fábrica, desligado no CSP).
- Na mistura com cor, o que falta ao PH2D é o **pincel opaco que apanha e arrasta** (amount of paint + color
  stretch): o Pigment mix vela, a receita Composite só desfoca, o Dabber cobre sem arrastar, o Blender Soft é mudo
  em seco. O nosso ganha em cor de pigmento (verde) e em relevo.

## v1 (⛔ com defeito — guardada só pelo histórico)

⛔ Tamanho errado (raio = diâmetro do CSP: todo o PH2D 2× mais largo) e arrasto BRANCO. Ver v2.

## Como foi feito

- **Oráculo corrido, não lido:** CSP 1.13.2 (avaliação PRO) sob Wine, cópia própria do prefixo, Xvfb `:76`, llvmpipe.
  Zoom 100 % (1 px de tela = 1 px de tela de pintura). Conduzido por `xdotool`: rato XTest = **pressão constante**
  (sem afinar nas pontas em nenhum dos lados). Fotos com `import -window`.
- **Os mesmos caminhos nos dois:** `material_2026-11-08/cena.py` gera as polilinhas (passo ~3 px); `xd.py` vira-as em
  script xdotool; a sonda `crates/ph2d-tool-painter/tests/it/sonda_csp_comparacao.rs` (`#[ignore]`) lê os mesmos
  pontos (`PH2D_CSP_CENA=<pasta>`), pressão 1, `on_tick(8 ms)` por ponto, presets de fábrica.
- Mesmo tamanho NUMÉRICO de pincel (px) e mesmas cores (azul `0046BE`, amarelo `F0C800`, vermelho `C81414`, branco).
- Cena A: espiral, zig-zague, S, hachura cruzada, preto. Cena B: blocos secos (G-pen opaco 16 px), depois 6 arrastos
  brancos de 30 px vai-e-volta + um X por cima.

Folhas (esquerda CSP, direita PH2D):
[A linhas 1](handoffs/material_2026-11-08/comparacao_csp_A_linhas_1.png) ·
[A linhas 2](handoffs/material_2026-11-08/comparacao_csp_A_linhas_2.png) ·
[B mistura 1](handoffs/material_2026-11-08/comparacao_csp_B_mistura_1.png) ·
[B mistura 2](handoffs/material_2026-11-08/comparacao_csp_B_mistura_2.png) ·
[tela inteira do CSP](handoffs/material_2026-11-08/comparacao_csp_tela_inteira_csp.png)

## ⚠️ Ressalva que muda a leitura: o «tamanho em px» não é o mesmo número

[M] largura do fio no vale do S (coluna limpa, píxeis < 128):

| pincel (tamanho pedido) | CSP | PH2D |
|---|---|---|
| G-pen / G-Pen 8 | 7 px | 16 px |
| Turnip 8 | 7 px | 16 px |
| Pencil / Sketch Pencil 8 | 4 px | 7 px |
| Charcoal / Charcoal Stick 16 | 13 px | 21 px |
| Airbrush Soft / Soft Airbrush 30 | 13 px | 22 px |

O PH2D com `set_brush_size_px(n)` pinta ~2× mais largo que o CSP com «Brush Size n» nas canetas. Ou o nosso número é
um raio (o do CSP é o diâmetro), ou a curva de tamanho do preset satura com pressão 1. **Por medir** antes de mexer
(não é desta sonda). Por isso os blocos do PH2D (G-Pen 16) saíram mais largos e com bordas em onda.

## A. Linhas — por par

- **G-pen ↔ G-Pen** [D]: CSP = fio de largura constante, ponta redonda. PH2D = fio grosso que **afina em espinho** em cada
  início/fim de traço mesmo com pressão constante (afinar por velocidade/entrada do preset): a hachura vira «laços
  de gravata» e o zig-zague ganha pontas de faca. Bonito em letra, mas não respeita a pressão constante.
- **Turnip ↔ Turnip Pen** [D]: no CSP é quase igual ao G-pen; no PH2D também — mesmo afinamento forte nas pontas.
- **Pencil ↔ Sketch Pencil** [D]: CSP = grão fino e seco, fio de 4 px, borda serrilhada de papel. PH2D = cordão mais
  largo, **contas visíveis** (os carimbos aparecem como bolinhas ao longo do traço), grão cinzento mais borrado.
  O CSP lê-se como grafite; o nosso lê-se como giz macio.
- **Charcoal ↔ Charcoal Stick** [D]: CSP = miolo preto sólido com franja de pó. PH2D = **«couve-flor»**: manchas
  redondas grumosas, sem miolo contínuo, contorno de bolhas — mais orgânico, menos carvão.
- **Airbrush Soft ↔ Soft Airbrush** [D]: ambos macios; PH2D tem **ruído de pó** dentro (pontinhos) e escurece mais nos
  cruzamentos; o CSP é liso. Empate de gosto; o nosso é mais «spray real».
- **Round watercolor ↔ Round Watercolor / Watercolor Basic** [D]: CSP = traço fino com textura de papel e escurecimento
  ao sobrepor (multiplicar) — a forma do caminho lê-se perfeitamente. PH2D = a água **alastra e engole a forma**:
  espiral vira mancha, hachura vira nuvem, bordas de flor. Mais aquarela «molhada», mas não serve para linha.

## B. Mistura — por par (arrastos brancos sobre blocos secos)

- **Round mixing brush ↔ Hard Round + Pigment mix** [D]: CSP = branco cobre e arrasta cor do bloco em gradiente
  longo e liso (rosa, azul-claro, creme); **mistura RGB** (azul+amarelo não dá verde). PH2D = **dá verde** onde o azul
  passa pelo amarelo (mistura de pigmento — mais realista), mas **o branco quase não aparece** (vira tinta de
  bloco diluída) e há **placas**: discos de carimbo visíveis na borda e degraus de opacidade entre passadas.
- **Gouache ↔ Composite (receita do dono)** [D]: CSP = cobre com branco riscado, arrasto com textura seca.
  PH2D Composite (Blur 1 · Smear 0,25 · Brush 0,10, Flow 0,03) = **quase nada acontece**: blocos só amaciados
  na borda, o branco não entra. Com Flow 0,03 a receita só desfoca.
- **Oil paint ↔ Watercolor Basic, Pigment mixing 1** [D]: CSP = cobertura com estrias de cerdas e arrasto de cor.
  PH2D = arrasta muito a cor para fora dos blocos (bom «puxar» da tinta) mas o branco vira **névoa azulada**,
  sem cobrir; bordas em flor da aquarela.
- **Thick oil paint ↔ Dabber** [D]: CSP = branco espesso com cor arrastada nas pontas. PH2D = **cobre tudo** de branco
  com relevo e sombra (o mais «óleo» de todos, textura convincente) mas **não arrasta cor nenhuma** para o branco:
  as cores somem em vez de sujar a tinta.
- **Blend ↔ Smear Basic** [D]: CSP Blend = desfoque suave que funde as fronteiras (gradientes azul→amarelo→vermelho).
  PH2D Smear = **liquify**: desloca a cor com bordas nítidas, não funde — serpentes de cor pura.
- **Finger tip ↔ Blender Soft** [D]: CSP Finger tip = arrasto com gradiente, a cor segue o dedo e esbate. PH2D Blender
  Soft = **nenhuma marca visível** com estes arrastos (blocos intactos).

## Veredito honesto

- **Linhas:** o CSP é mais fiel ao caminho e à pressão (largura constante, ponta redonda, grafite fino). O nosso tem
  mais carácter (afinar, pó, água viva), mas impõe-no mesmo onde o artista não pediu.
- **Mistura:** **os pincéis de mistura do CSP ganham** no uso do dia a dia: cobrem, arrastam a cor em gradiente liso
  e nunca mostram placas. O nosso só ganha em **realismo de cor** (o verde do pigmento) e em **relevo** (Dabber).

## O que o PH2D deve melhorar (por ordem)

1. **Tamanho em px igual ao do mundo** — medir porque 8 px dá 16 px; o artista que vem do CSP pede 8 e espera 8.
2. **Pincel de mistura que COBRE e ARRASTA ao mesmo tempo** (o Round mixing do CSP: quantidade de tinta + estirar
   cor): hoje ou cobre sem arrastar (Dabber) ou arrasta sem cobrir (Pigment mix, Watercolor).
3. **Placas/degraus do Pigment mix** — os discos de carimbo e os patamares de opacidade entre passadas.
4. **Blend/Smear que FUNDE** (como o Blend e o Finger tip do CSP), não só desloca; e o Blender Soft não pode ser mudo.
5. **Afinar das canetas só quando há pressão/velocidade** — com pressão constante o fio devia ser constante.
6. **Lápis e carvão com miolo contínuo** (contas e couve-flor são carimbos visíveis).
7. A receita Composite do dono precisa de Flow maior para o Brush aparecer (a 0,03 é só Blur).

Por medir depois: o mesmo com caneta real (pressão), e a mistura com cor de arrasto colorida em vez de branco.

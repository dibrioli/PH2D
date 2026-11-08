# Estudo — os pincéis do Clip Studio Paint (CSP), para escolher quais imitar (07/11/2026)

> **Para quê:** o dono escolhe quais pincéis do CSP o Painter imita. Este doc diz quais são os
> importantes, o que os define (números lidos dos ajustes de fábrica), como o CSP monta um pincel,
> o que a nossa engine já tem e por onde começar.
> **Legenda:** **[M]** = MEDIDO (lido na base de ajustes do CSP ou no nosso código) · **[D]** = DEDUZIDO
> (interpretação, não confirmada correndo o app).
> **O app não foi corrido** (opção da missão): tudo vem dos ajustes de fábrica e da pesquisa web.

## 1. Fonte dos dados

- **[M]** Base de ajustes de fábrica do CSP 1.13.2 (`EditImageTool.todb`, SQLite, aberta só em leitura),
  tabelas `Node` (a árvore Ferramenta › Grupo › Sub-ferramenta) e `Variant` (ajustes de cada
  sub-ferramenta). Os scripts de leitura ficaram no scratchpad da sessão, não no repo.
- **[M]** Comparámos a lista de nomes com a do CSP 5.1.5 (`~/Apps/clipstudio`). Os pincéis de pintura
  são **os mesmos**. Muda só a arrumação: «Pencil» passa a «Sketch», o grupo «India ink» entra em Thick
  paint/Brush, e aparecem «Rough pen» (Pen), «Dynamic Eraser», «Lasso fill» e «Leftover pen».
- **[M]** Os números são os mesmos que o painel *Sub Tool Detail* mostra ao usuário.

## 2. Ranking — os 14 pincéis que importam

Critério: o que a pesquisa diz que os artistas mais usam e recomendam, e o que o CSP põe na frente como
padrão de cada grupo. **«Rosto»** marca os pincéis pelos quais o CSP é conhecido.

| # | Pincel CSP (grupo) | Rosto? | Porquê (fonte) |
|---|---|---|---|
| 1 | **G-pen** (Pen) | ★ | O pincel padrão do mangá e o mais citado para lineart ([tips 4703](https://tips.clip-studio.com/en-us/articles/4703), [Packt, inking tools](https://subscription.packtpub.com/book/game-development/9781800564978/8/ch08lvl1sec57/exploring-inking-tools), [tips 516](https://tips.clip-studio.com/en-us/articles/516)) |
| 2 | **Turnip pen** (Pen) | ★ | Favorita de autores de tips: muita sensibilidade, toque «de pincel» ([tips 4703](https://tips.clip-studio.com/en-us/articles/4703)) |
| 3 | **Pencil** (Pencil) | ★ | O padrão do esboço ([tips 4666](https://tips.clip-studio.com/en-us/articles/4666), [graphixly](https://graphixly.com/blogs/news/default-pencil-tools)) |
| 4 | **Airbrush › Soft** | ★ | Sombreamento suave, a base da pintura anime ([clipstudio.net tool-brush](https://www.clipstudio.net/en/characterart/tool-brush/)) |
| 5 | **Blend** (Blend) | ★ | O esfumar padrão da pintura no CSP ([tips 1881](https://tips.clip-studio.com/en-us/articles/1881)) |
| 6 | **Round watercolor brush** (Watercolor) | ★ | Pincel principal do conjunto de aguarela de 1.8.6+, com borda de água e papel ([tips 1514](https://tips.clip-studio.com/en-us/articles/1514), [tips 1515](https://tips.clip-studio.com/en-us/articles/1515)) |
| 7 | **Round mixing brush** (Thick paint) | ★ | O tipo de quase todo o grupo Thick paint ([tips 1514](https://tips.clip-studio.com/en-us/articles/1514)) |
| 8 | **Gouache** / **Dry Gouache** (Thick paint) | ★ | Guache com mistura de cor; o Dry imita o pincel seco ([tips 1514](https://tips.clip-studio.com/en-us/articles/1514)) |
| 9 | **Oil paint** (Thick paint) | ★ | O «óleo» de fábrica, com tela e mistura ([clipstudio.net](https://www.clipstudio.net/en/characterart/tool-brush/)) |
| 10 | **Mapping pen** (Pen) | | Linha fina e uniforme, a parceira da G-pen ([Packt](https://subscription.packtpub.com/book/game-development/9781800564978/8/ch08lvl1sec57/exploring-inking-tools)) |
| 11 | **Finger tip** (Blend) | | Arrastar a tinta, o «smudge» clássico [D, padrão de fábrica] |
| 12 | **Charcoal** (Pastel) | | O pincel seco de textura mais característico [D] |
| 13 | **Brush pen** (India ink) | | Tinta nanquim com cerda e textura [D] |
| 14 | **Thick oil paint** (Thick paint) | | Óleo grosso com ponta dupla e Running color [D] |

**Limite da fonte:** a pesquisa não achou um ranking por números de uso. São tutoriais oficiais e
depoimentos de artistas. A G-pen e a Turnip são citadas por nome; o resto vem dos tips oficiais.
Ninguém citou um óleo favorito.

## 3. O que define cada pincel (ajustes de fábrica) [M]

**Como ler as colunas:**
- **Tam.** = tamanho em px.
- **Dur.** = dureza, de 1 a 100.
- **Int.** = intervalo, em % do tamanho.
- **Esp.** = espessura (achatamento, 100 = redondo) e ângulo.
- **Ponta** = «img» quando a ponta é uma imagem, ×2 quando há duas imagens.
- **Textura** = nome do material de papel do CSP (só o nome; a imagem não foi copiada).
- **Mistura** = `mix` / `alpha` / `ext`, ou seja «Amount of paint» / «Density of paint» / «Color stretch».
- **Dinâmica**: `P` pressão · `T` inclinação · `V` velocidade · `R` aleatório. A letra de cada fonte é
  [D] (ver §3.1); o mínimo, quando existe, vem entre parênteses.

| Pincel | Tam. | Dur. | Int. | Esp. | Ponta | Textura | Mistura | Outros blocos | Dinâmica |
|---|---|---|---|---|---|---|---|---|---|
| G-pen | 10 | 100 | 10 | 100 | redonda | — | — | entrada/saída 20 | Size P·T |
| Real G-Pen | 10 | 100 | 10 | 85 | img (bico) | — | — | — | Size P·T, Esp P(50) |
| Mapping pen | 10 | 100 | 10 | 100 | redonda | — | — | entrada/saída 20, estab. 10 | Size P·T |
| Turnip pen | 8 | 100 | 10 | 100 | redonda | — | — | entrada/saída 20, estab. 2 | Size P·T |
| Calligraphy | 50 | 100 | 10 | 18 @60° | redonda achatada | — | — | entrada/saída 20 | nenhuma |
| Felt pen | 20 | 100 | 10 | 80 @90° | redonda | — | — | — | Size P·V·R(70), Flow V |
| Flat marker | 50 | 70 | 0,1 | 30 @90° | redonda achatada | — | — | — | Size P(60), Opac P |
| **Pencil** | 10 | 100 | 10 | 100 @180° | img ×2 (grafite) | Rough paper, dens 80 | — | **ponta dupla**, opac 90 | Size P·T(10), Flow P·T·R(50), Esp P·T(60), Textura P·T(30) |
| Mechanical pencil | 5 | 100 | 0,3 | 100 | img | Pencil, dens 45 | — | entrada/saída 20 | Size P·T, Flow P(60), Textura P·T·V(18) |
| **Charcoal** | 20 | 100 | 10 | 100 | img ×2 (carvão) | Drawing paper C, dens 100 | — | ponta dupla, valor −5 por dab | Size P·T(50), Flow P·T(30), Esp(90), Textura P·T·R(10) |
| Crayon | 30 | 100 | 20 contínuo | 80 | img | Rough textured, dens 90 | 60/100/15 | HSV por dab 10/−10/−3 | Size P·T(30), Flow P |
| Pastel | 40 | 1 | 10 | 100 | img suave | Rough textured, dens 60 | 40/40/40 | sat. +15 por dab | Size P·T(40), Flow P(15), Int P |
| Chalk | 30 | 81 | 10 | 50 | img | Drawing paper C, dens 80 | 41/100/15 | HSV 0/5/−10 | Size P·T(50), Flow P·T(50), Textura P·T·R(30) |
| **Round watercolor** | 80 | 100 | 10 | 100 | redonda | Paper, dens 70 | — | **borda de água** r 2, alpha 5; composição 2 | Size P·T, Opac P·V |
| Flat watercolor | 100 | 100 | 0,5 | 45 @180° | img chata | Paper, dens 60 | — | borda de água r 2, alpha 10 | Size P(60), Opac P·V(30) |
| Soft bleed | 120 | 100 | 15 contínuo | 100 | img | Paper, dens 80 | — | borda de água r 2, a 25, v 20, blur 4, **após arrasto** | Size P·T(70), Opac P(10) |
| Wet wash | 30 | 100 | ~1 contínuo | 100 | img | Dappled cloud 1 | 70/50/10 | **spray** 120 dens 4 desvio −50, ponta dupla | Mistura P, Spray P, Opac P(10) |
| Rough wash | 150 | 100 | 15 | 100 | img | Impasto, dens 50 | — | borda de água r 1, após arrasto | Size P·R, Flow P·R(80), Esp P·R(70) |
| Watercolor splash | 300 | 100 | ~9,5 contínuo | 100 | img ×2 | Sand, dens 15 | — | spray 325, ponta dupla, matiz ±15 | Size P, Spray R, Opac P·R(40) |
| **Round mixing brush** | 50 | 100 | 10 contínuo | 100 | redonda | — | **80/50/100** | estab. 2 | Size P(20), Flow P, Mix P, Alpha P(90) |
| **Gouache** | 80 | 100 | 3 | 100 @45° | img (cerda) | Paper, dens 100 | 70/45/15 | blur 0,3 | Size P(10), Flow P(20), Mix P(10), Alpha P(10) |
| **Dry Gouache** | 80 | 100 | 5 | 100 @45° | img seca | Paper, dens 100 | 80/80/0 | flow 50 | Size P(30), Esp R, Mix P(10), Alpha P(50) |
| Thin Gouache | 20 | 100 | 1 | 100 @63° | img | Paper, dens 85 | 100/55/70 | opac R | Size P(10), Textura P, Mix/Alpha P(10), Blur P |
| **Oil paint** | 80 | 100 | 1 | 100 @115° | img (cerda) | Canvas 02, dens 12 | 61/81/87 | HSV por dab 3/50/16 e por traço 4/1 | Size P(10), Esp P·V(10), Textura P(15), Mix/Alpha P(10) |
| Thick oil paint | 60 | 100 | 2 contínuo | 60 @70° | img | Canvas 02, dens 50 | 85/50/60, **modo 2** | ponta dupla, valor −3 | Size P·T(30), Esp P·V(50), Mix P |
| Pointillism | 80 | 100 | 10 | 30 @45° | img | — | 50/50/15 | spray 30 desvio −100, HSV 30/20/10 | Size P·R(10), Esp T·R |
| Paint and apply | 40 | 40 | 10 | 100 | redonda | — | 80/100/50, **modo 1** | entrada/saída 20 | Size T, Flow P(40), Mix/Alpha P |
| Brush pen | 70 | 100 | 2 contínuo | 100 | img ×2 (cerda) | Dappled cloud 1, dens 100 | — | ponta dupla, estab. 15 | Size P·V, Flow P·V(50), Textura P·V(52) |
| Dry ink | 60 | 100 | ~1 contínuo | 100 | img | Dappled cloud 2 | — | — | Size P·V·R(5), Textura P·V(35) |
| **Airbrush Soft** | 150 | 1 | 10 | 100 | redonda | Sand, dens 50 | — | flow 50, entrada/saída 20 | Size T, Flow P·T |
| Spray | 200 | 100 | 10 | 100 | redonda | — | — | spray 5 dens 5 | Spray R, densidade P |
| **Blend** | 80 | 1 | 10 | 100 | redonda | — | 0/0/10 | — | Flow P |
| Blur | 80 | 1 | 10 | 100 | redonda | — | 0/0/0, modo 1 | blur 5 | Size P |
| **Finger tip** | 30 | 50 | 3 | 100 | redonda | — | 0/0/70, **modo 1** | estab. 4 | Size P |
| Painterly blender | 100 | 1 | 1,5 | 100 | img | — | 0/0/70 | flow 30 | Size P, Flow P |
| Textured blender | 100 | 100 | 25 | 100 | img | Impasto, dens 80 | 0/0/10 | borda de água r 0,1, blur 5 | Size/Flow/Esp/Blur P, Opac P·V |
| Kneaded eraser | 50 | 100 | 10 | 100 | img | Impasto, dens 80 | — | blur 10, apagar | Size P·T, Blur P, Opac P·T(10) |

**Observações [M]:**
- **Ribbon e «cor secundária»**: desligados em todos os pincéis de pintura.
- **Variação HSV** («Color jitter»): ligada no Charcoal, Crayon, Pastel, Chalk, Oil, Thick oil, Pointillism e Splash.
- **Ponta dupla** («Dual brush»): Pencil, Charcoal, Thick oil, Brush pen, Wet wash, Wet blotting e Splash.
- **Borda de água**: só no grupo Watercolor novo e no Textured blender.
- **Modo de mistura**: o campo `BrushWaterColor` vale 0, 1 ou 2 e é o modo de «Color mixing». **[D]**
  A leitura é: 0 = Blend, 1 = Running color (Blur, Finger tip, Paint and apply), 2 = Thick oil.
- **Composição**: o grupo Watercolor usa a composição 2, que lemos como Multiplicar **[D]**. As borrachas
  e o Watery brush usam a 27, que lemos como «apagar» **[D]**.

### 3.1 Curvas de efeito (blobs `*Effector`) — decodificação parcial

- **[M]** Formato, em big-endian: um cabeçalho de 44 bytes (palavra 2 = máscara de fontes; palavra 3 =
  mínimo em %; palavra 4 = 100), seguido de N curvas. Cada curva tem um cabeçalho `12, nPontos, 16` e
  depois pares `(x, y)` em double.
- **[M]** A primeira curva é a **curva de pressão**. Na G-pen é linear (0,0)→(1,1). A segunda curva tem
  forma decrescente (0,1)→…→(1,0) e é igual em quase todos os pincéis: parece a curva de
  velocidade/fade padrão **[D]**.
- **[M]** A máscara usa os bits `0x10`, `0x20`, `0x40` e `0x80`. A variação HSV por dab só usa `0x80`.
- **[D]** Por isso, e pela ordem do painel do CSP (Pressão, Inclinação, Velocidade, Aleatório), lemos
  `0x10` = Pressão, `0x20` = Inclinação, `0x40` = Velocidade, `0x80` = Aleatório. A G-pen sairia com
  «Size: pressão + inclinação», e só a abrir o app se confirma. Na tabela acima as letras seguem esta leitura.

## 4. O modelo de pincel do CSP numa página

Cada carimbo (dab) passa por estes blocos, por esta ordem:

1. **Ponta.** Ou é um círculo com dureza (1 a 100), ou uma a N **imagens** em escala de cinzento,
   tiradas em ordem ou ao acaso. A ponta é **achatada** (espessura) e **rodada** (ângulo fixo,
   pela direção do traço ou ao acaso).
2. **Intervalo.** É a distância entre carimbos, em % do tamanho. «Contínuo» quer dizer que o carimbo é
   esticado ao longo do traço, sem pontos visíveis. Os pincéis de tinta usam 1–3 %; as canetas, 10 %.
3. **Textura.** Um papel (imagem que se repete) entra **por carimbo** ou **pelo traço inteiro**, com
   densidade, modo de composição e escala. A densidade é quase sempre ligada à pressão: pressão leve
   mostra o papel, pressão forte tapa-o.
4. **Mistura de cor.** São três números:
   - «Amount of paint» (`mix`): quanta cor nova sai;
   - «Density of paint» (`alpha`): quanto cobre;
   - «Color stretch» (`ext`): quanto a cor apanhada na tela é arrastada.

   O modo «Blend» mistura com o que está por baixo; o «Running color» arrasta a tinta. Um pincel
   **misturador** (Blend, Finger tip) é só a mistura com tinta nova = 0.
5. **Aguado.** A **borda de água** escurece a orla do traço inteiro (raio, força no alfa, força no
   valor, desfoque, e se se aplica «depois de arrastar»). É um efeito **sobre o traço acabado**, não uma
   simulação de fluido.
6. **Spray.** Espalha N carimbos pequenos dentro do diâmetro (tamanho, densidade, desvio para o centro).
7. **Por cima de tudo:**
   - **ponta dupla**: uma segunda ponta com textura própria, que recorta a primeira;
   - **variação HSV** por carimbo ou por traço;
   - **entrada/saída** (afinar as pontas do traço);
   - **estabilização**;
   - **dinâmicas**: cada parâmetro tem a sua curva de pressão e as fontes inclinação, velocidade e aleatório.

**A lição:** o CSP **não simula tinta**. Tudo é carimbo + papel + mistura com o que está por baixo +
pós-efeito de borda. A «aguarela» e o «óleo» dele vêm da **escolha dos números**, não de um motor físico.
É barato e previsível, e é por isso que o desenho de mangá e anime gosta dele.

## 5. Mapa CSP → PH2D [M, código lido]

Os nossos motores:
- **Digital**: `crates/ph2d-painter-brush/src/spec.rs`, o `BrushSpec`.
- **Wet Paint / Watercolor / óleo**: `crates/ph2d-wet-paint/src/tuning/defs.rs`, os knobs.
- **Presets de fábrica**: `assets/brushes/` (21 Digital, 1 Watercolor, 56 WetPainter).

| Bloco CSP | Digital (`ph2d-painter-brush`) | Wet Paint / Watercolor | Falta |
|---|---|---|---|
| Ponta redonda + dureza | sim (`hardness`, 10 curvas em `falloff.rs`) | cerdas (`bristle*`), sem círculo de dureza | — |
| Ponta imagem / N imagens | sim (`texture/shape.rs`, `mangueira.rs`) | **não** | ponta de imagem no Wet |
| Achatamento + ângulo | sim (`dab_flatten`, `dab_angle_deg`, `dab_tilt`) | `dabRotation`, `tuftArc` | — |
| Intervalo / contínuo | sim (`spacing`, `continuous_spacing`) | `spacing` oculto | — |
| Textura de papel por carimbo/traço, densidade por pressão | sim (`GrainMode` com 6 modos, mapeamento Tiled/ViewPlane/Random/Stencil, profundidade por pressão) | papel físico (`paperContrast`, `paperFibres`, `paperRelief`…) | «densidade» como nome de controlo; os papéis «Canvas»/«Drawing paper» feitos por nós |
| Ponta dupla | sim (`dual.rs`) | não | — no Digital |
| Mistura de cor (mix/alpha/stretch) | **parcial**: `wet_charge`/`wet_pull`/`wet_dilution`/`wet_smudge` só com o flag `watercolor`; `pigment_mix` RYB em `blend.rs`; `smear_fio` | `pickup` (≤ 0,2), `drag`, `smear`, `blendForce`, `wetCover`/`dryCover` | **mistura «Amount/Density/Stretch» de pincel opaco no Digital sem o flag watercolor** — é o que todo o Thick paint usa |
| Modo Running color | `wet_smudge` (smear verdadeiro), só com watercolor | `smear`, `drag` | idem |
| Borda de água | `edge_gain`/`edge_spread` (flag watercolor) | `edgeDarkening`, `edgeSoftMass` | o «depois de arrastar» e a força no valor como controlos separados [D] |
| Blur no traço | preset Blur Basic (motor não confirmado) | não | confirmar |
| Variação HSV por carimbo/traço | sim (`color_jitter_*`, `color_jitter_scope`) | **não** | HSV no Wet (Oil/Charcoal usam-no) |
| Spray | sim (`spray_count` ≤ 16) | **não** | spray no Wet; o CSP chega a densidades de 8 com tamanho 325 |
| Entrada/saída | sim (`taper.rs`) | **não** | taper no Wet |
| Dinâmicas P/T/V/R por parâmetro | P, V (`speed_*`), T, aleatório (`jitter*`) | P e T sim; **V e R não** | velocidade e aleatório no Wet |
| Estabilização | sim (`stabilizer`, lazy mouse) | só `pressureSmoothing`/`pulledString` | — |
| Composição por pincel (Multiplicar, Apagar…) | sim (24 modos, `BrushBlend`) | **não** | — |

**Em que modo nosso cairia cada pincel do CSP [D]:**
- **Digital**: todas as Pen e Marker, Pencil, Mechanical pencil, Airbrush (Soft, Spray, Noise, Droplet),
  Blend, Blur, Finger tip, Painterly blender, as borrachas, Pointillism. O Charcoal, Crayon, Pastel e
  Chalk ficam no **Digital com grão + ponta dupla**.
- **Watercolor** (Digital com o flag watercolor): Round/Flat watercolor, Soft bleed, Rough wash, Watery
  brush, Wet bleed blender, Watery ink, Wet blotting ink.
- **Wet Paint**: Wet wash e Watercolor splash (só se quisermos a versão viva; senão Watercolor). Oil
  paint e Thick oil, se o dono quiser o relevo (`oilDepth`, `Volume`).
- **Digital + mistura**: Round mixing, Gouache, Dry/Thin Gouache, Paint and apply. É a escolha natural,
  porque o CSP é carimbo + mistura, mas a mistura opaca no Digital é o buraco da tabela acima.
- **Impasto**: não há pasta «Impasto» nos presets. O óleo vive em `WetPainter/Oil`.

## 6. Proposta — ordem de imitação

**Lote 1 — maior valor, quase nada falta (só presets nossos, no Digital) [D]:**

| # | Pincel | Porque já dá | Precisa |
|---|---|---|---|
| 1 | **G-pen** | redonda dura + pressão no tamanho + entrada/saída: tudo existe | preset (o nosso Mapping Pen já é parente) |
| 2 | **Turnip pen** | igual à G-pen, com tamanho 8 e estabilização 2 | preset |
| 3 | **Airbrush Soft** | dureza 1, flow 50, tamanho por inclinação, grão Sand | preset + papel areia nosso |
| 4 | **Pencil** | ponta de imagem + ponta dupla + grão por pressão + 4 curvas | preset + ponta de grafite e papel «rough» desenhados por nós |
| 5 | **Round watercolor** | flag watercolor + borda (`edge_gain`) + papel + composição Multiplicar | preset; conferir que a borda fica como a do CSP (raio 2, fraca) |
| 6 | **Blend / Finger tip** | Smear Basic/Blend Soft já existem | ajustar à «stretch» do CSP (10 e 70) |
| 7 | **Charcoal** | ponta dupla + grão denso por pressão + jitter de valor | preset + ponta de carvão nossa |

**Lote 2 — precisam do bloco que falta: a mistura opaca no Digital:**
Round mixing brush, Gouache, Dry Gouache, Thin Gouache, Oil paint, Paint and apply, Crayon, Pastel e
Chalk. **O motor novo que falta:** um «mixer» de pincel opaco com os três controlos do CSP
(tinta nova, densidade, esticar a cor), cada um com a sua curva de pressão, e modo Blend × Running color.
Tem de funcionar **sem** o flag watercolor. Hoje as peças existem (`wet_charge`, `wet_pull`, a
mistura RYB), mas presas à aguarela. É o maior buraco: só ele bloqueia **9 dos 14** pincéis de pintura
do CSP.

**Lote 3 — Wet Paint (só se o dono quiser a versão «viva»):**
Wet wash, Watercolor splash e Thick oil paint. Faltam ao Wet Paint: spray, variação HSV, entrada/saída,
dinâmica de velocidade/aleatório e ponta de imagem.

**Fora de escopo:** Decoration (carimbos de padrões, rendas, folhas) — é outra família, nota só.

## 7. Licença e limites

- **Lido [M]:** só os **números dos ajustes** (os mesmos do painel *Sub Tool Detail*), os **nomes** das
  sub-ferramentas e os **nomes** dos materiais de textura («Paper», «Canvas 02», «Sand», «Dappled cloud 1»…).
  A base foi aberta só em leitura, sem gravar nada no prefixo Wine.
- **Não feito:** nada de desmontar nem descompilar DLL/EXE, nada de strings de binário.
- **Não copiado:** nenhuma imagem de ponta, textura nem material da Celsys. Os blobs de ponta e textura
  só foram lidos para tirar o nome do material.
- **Na reprodução:** pontas e papéis **nossos** (procedurais ou desenhados no PH2D), com os mesmos
  números de caráter. O CSP é um **oráculo para observar**: se se quiser paridade fina, corre-se o app
  (Xvfb, `~/Apps/clipstudio1/csp1.sh`) sobre traços nossos e compara-se a imagem. **Não foi feito
  nesta sessão.**
- **Por confirmar correndo o app:** o mapa dos bits P/T/V/R, o significado de `Stickness` (G-pen 4,
  Painterly blender 10, quase todos 0), o modo de `BrushWaterColor` (0/1/2) e os números das
  composições 2, 27 e 31.

# A COR QUE A PEÇA DEVOLVE AO CHÃO — `W6` do render (2026-09-17)

> O chão invisível da `W4` desenha o que a peça **TIRA** (a sombra das lâmpadas, o escurecimento de
> contacto do céu) e nunca desenhou o que ela **PÕE**: um vaso vermelho pousa numa sombra cinzenta,
> onde a verdade é um avermelhado à volta da base. Esta wave é essa metade.
>
> **Código:** [`ph2d_field_render::ground_bounce`](../../crates/ph2d-field-render/src/ground_bounce.rs)
> (a lei) · [`ground_shade`](../../crates/ph2d-field-render/src/ground_shade.rs) (o que o chão põe num
> pixel) · [`paint.rs`](../../crates/ph2d-field-gpu/src/paint.rs) (o dispositivo).

## §1 — ⚠️ Porque só agora, e o que mudou

O [`08` §10](08_a_luz_indirecta.md) nomeou este buraco e escreveu *«a marcha dela é outra»* — verdade
enquanto o ricochete se recolhia **por pixel**, marchando do ponto acertado. Com as sondas
([`08` §14](08_a_luz_indirecta.md)) a resposta num ponto qualquer passou a ser uma consulta a uma
grelha já assada. ⇒ §0.0: *quem move o número que tornava algo inalcançável tem de reconferir a nota.*

## §2 — ⛔⛔⛔ A 1.ª hipótese caiu na 1.ª medição: as sondas da peça NÃO servem ao chão

A pergunta óbvia é *«o chão não pode consultar a grelha de sondas que já está assada?»*. Medido
(bola vermelha de raio `0,5` pousada, luz de lado), sobre a irradiância devolvida no canal vermelho:

| `x` (raios do centro) | VERDADE (`8 192` direcções) | a grelha 3D devolve | `% da luz livre` |
|---:|---:|---:|---:|
| `1,00` (o contacto) | `0,002697` | `0,001941` | `0,52 %` |
| `1,50` | `0,009411` | `0,003075` | `1,72 %` |
| **`1,75`** | **`0,009772`** (o **PICO**) | `0,003063` | `1,76 %` |
| `2,50` | `0,007167` | `0,003052` | `1,27 %` |
| `4,00` | `0,002962` | `0,003046` | `0,58 %` |
| `5,75` | `0,001165` | `0,003044` | `0,28 %` |

Duas leituras, e as duas mandam:

- ⛔⛔ **a grelha 3D SATURA.** Ela cobre a bola da peça com margem `1,05` e a consulta **agarra-se à
  borda** (o `clamp` do `u`) — daí a coluna dela ser uma **constante** de `1,5` raios em diante. Num
  plano que vai até ao horizonte isso pinta o mundo inteiro de vermelho, com a mesma força a `5`
  raios e a `500`;
- ⛔⛔⛔ **e o PICO do sangramento fica FORA dela** (`1,75` raios contra os `1,05` que ela cobre).
  *Uma extrapolação ancorada na borda não pode reproduzir um máximo que acontece depois dela* — logo
  nem alargar a margem (rouba resolução à peça) nem multiplicar por `1/r²` (decai a partir do sítio
  errado) servem.

⇒ **o chão é um PLANO, logo o campo dele é 2D.** É isso que torna a lei própria barata: `32²` pontos
contra os `32³` da peça. É o mesmo precedente que a `W4` já tinha estabelecido para a oclusão do céu
no chão (*«`48` cones num recetor plano desenham ANÉIS ⇒ lei própria por campo de distância»*).

## §3 — ⭐⭐ O estimador: as direcções vão todas para dentro do CONE da peça

Um ponto de chão longe vê a peça num cone estreito. Com direcções **uniformes sobre o hemisfério**
quase nenhuma acerta, e o que se mede é a contagem de acertos: a coluna VERDADE acima precisou de
`8 192` direcções para deixar de saltar (a `1 024` ela lia `0,0067` num ponto e `0,0037` no seguinte
— ruído de `±50 %` sobre uma curva lisa).

⇒ as direcções são sorteadas **dentro do cone** que a bola da peça subtende, e o integral é corrigido
pelo ângulo sólido dele: `E = Ω/(π·N) · Σ L·cos`. **Nada no resto da cena devolve luz** (o chão é
invisível e não entra na marcha), logo fora do cone a radiância devolvida é ZERO **por construção** e
o estimador continua sem viés.

⚠️ **A base do cone é CONTÍNUA sobre o chão, e isso é uma cerca:** a tangente sai de
`cross(eixo, X)`, que nunca degenera porque o eixo aponta do chão para o centro da peça e tem `y > 0`
**sempre**. ⛔ A base construída pelo truque habitual (o ramo que troca de fórmula quando a
componente dominante muda) tem uma **descontinuidade**, e uma descontinuidade num campo interpolado é
uma COSTURA desenhada — a recusa que o [`occlusion`](../../crates/ph2d-field-render/src/occlusion.rs)
já pagou.

## §4 — ⛔⛔⛔ Porque a luz devolvida SOMA e não multiplica, com o literal que o decide

A tentação é tingir a razão do chão: fazer `f` um `vec3` e deixar o vermelho subir. **Isso é
invisível no produto**, e há um literal que o prova: o fundo do modelador é
`ph2d_app_field3d::smoke::BACKGROUND = [0, 0, 0, 0]` — **transparente**. Uma razão multiplica
`bg.rgb`, que ali é ZERO, logo a wave inteira sairia num pixel que não muda um bit.

⇒ a luz devolvida entra **somada em pré-multiplicado com alfa ZERO**, que é o que um compositor lê
como *luz acrescentada*: `resultado = fundo·f + B`. Medido, o pico do campo vale `0,015114` ⇒
**`33/255`** no vermelho sobre o preto do modelador — *vê-se bem*.

⚠️ **E a razão CONTINUA escalar**, que é a decisão que o `ground::LUMA` já declara por escrito (*uma
sombra tingida pela cor de cada luz mudaria de matiz sobre um fundo colorido*). As duas metades não
se misturam: uma diz **quanto escurece**, a outra **que luz mais chega**.

⚠️ **O alfa NÃO sobe com a luz devolvida.** Um chão INVISÍVEL não tem albedo próprio com que se
tornar opaco: a sombra sobe o alfa porque é uma camada preta a TAPAR o que está atrás; a luz
devolvida é luz a SOMAR-SE, que em pré-multiplicado é exactamente `rgb += B, a += 0`.

## §5 — ⭐ Os dois tectos, e os dois saem da SAÍDA DE 8 BITS

**As direcções** (`GROUND_BOUNCE_DIRS`), contra o mesmo campo a `4 096`, com o erro em unidades do
pico (`0,015114` = `33/255`):

| direcções | pior desvio / pico | em bytes |
|---:|---:|---:|
| `16` | `8,73 %` | `4` |
| `64` | `3,53 %` | `2` |
| **`128`** | **`0,96 %`** | **`0`** |
| `256` | `1,27 %` | `1` |

**A grelha** (`GROUND_BOUNCE_GRID`), contra a VERDADE convergida, com o relógio ao lado:

| grelha | pior desvio | em bytes | relógio |
|---:|---:|---:|---:|
| `16²` | `0,002406` | `8` | `1,51 ms` |
| `24²` | `0,001027` | `3` | `3,52 ms` |
| **`32²`** | **`0,000736`** | **`2`** | **`5,33 ms`** |
| `48²` | `0,000492` | `2` | `12,00 ms` |
| `64²` | `0,000401` | `1` | `20,65 ms` |

⛔⛔ **A grelha esteve em `48` sem uma medição por baixo**, e o §0.0 manda medir ANTES de escrever um
limite: o número não medido estava caro no lado errado — ele pagava `6,7 ms` do quadro assente por
nada que a saída soubesse representar. `32²` lê os **mesmos 2 bytes** por `44 %` do relógio.

## §6 — ⏱️ O preço, e onde ele mora

| | `1920×1080`, mínimo de 5, intercalado no MESMO processo |
|---|---:|
| sem a cor no chão | `7,51 ms` |
| **com a cor no chão** | **`12,49 ms`** (`+4,98`) |

⚠️ **Os `+4,98 ms` são TODOS a assadura de CPU** — o lado do dispositivo é um raio de chão a mais e
uma bilinear por pixel de fundo. E a assadura só é viável porque o lote é **repartido**: a
[`march::march_rays`] é sequencial (ela serve passes já paralelizados de fora), e num lote só a
mesma grelha lia **`106,5 ms`**.

⚠️ **Ele viaja na bandeira que JÁ EXISTE** (`antialias`, a lei «grosso a mexer, nítido ao assentar»
da W73, agora com o quarto passageiro): o quadro de MOVIMENTO fica **byte-idêntico** ao de hoje.

⏳ **As duas curas do preço ficam NOMEADAS, com o número:**

1. **a cache por cena-e-luz** — o campo **não depende da câmera**, logo orbitar podia reutilizá-lo
   inteiro; é a mesma cura que o [`08` §14.6](08_a_luz_indirecta.md) já nomeia para as sondas, e aqui
   é mais barata (o campo tem `32² × 3` floats).

   ⛔⛔⛔ **A redacção de 2026-09-20 dizia que a premissa estava medida e era MAIS FORTE do que esta
   linha afirmava — e estava errada.** Ela concluía *«a tolerância também não move o campo»* a
   partir da tabela abaixo; medido em 2026-09-21 com o CONTROLO que faltava, **nas três leituras
   daquela tabela o `hit` esteve preso no tecto** (`HIT_EPS = 2e-4`), porque
   `hit = min(HIT_EPS, half_extent/(2·lado_px))` só desce com `lado_px > 2500 × half_extent`.
   ⇒ *a tabela media o CLAMP e não o eixo*, e esta linha — *«a dependência da câmera é só a
   tolerância de acerto»* — **estava certa**. Abaixo do clamp a tolerância move o campo até
   **`0,91` de um byte**, e a chave da cache leva-a; o que sai da chave é a **orientação**, que move
   `6e-9` (`3` ULP). A tabela do plano, os dois gates e a barra derivada do byte estão no
   [`03` §W9](03_o_plano.md). *A tabela que segue fica como estava, com o que ela de facto mede
   escrito por cima:*

   | eixo | `|Δ|` máximo de uma célula | em fracção do campo |
   |---|---:|---:|
   | **orbitar** (8 azimutes) | `0,000000` | `0,000 %` |
   | **aproximar** (`half_extent` `0,8` · `1,6` · `3,2`) | `0,000000` | `0,000 %` |
   | **CONTROLO: a luz do outro lado** | `0,016357` | `100,000 %` |

   ⇒ *a chave da cache não leva a câmera de todo*, e a cura é **exacta** em vez de aproximada.
   ⚠️ **O CONTROLO é o que dá direito às outras duas linhas** — sem um eixo que MOVE o campo, uma
   sonda de invariância mede a si própria. ⛔ E a leitura do código sugeria o contrário: a
   `radiancia_devolvida` recebe a `ViewBasis`, que é a orientação, e a resposta de um BSDF ao longo
   da vista tem lóbulo especular — *ler a assinatura de uma função não diz se o valor dela se mexe.*
2. **o kernel próprio no dispositivo** — `~0,04 ms` na placa contra os `5` da CPU. ⛔ Ele **não** foi
   feito de propósito: assar na CPU e enviar compra **UMA lei em vez de duas** — não há kernel de
   assadura em WGSL para divergir do da CPU, logo não há paridade de ASSADURA para falhar, só a da
   CONSULTA (uma bilinear), que fecha a `100,000 %`.

## §7 — ⭐ Os portões, e as mutações

| gate | o que afirma |
|---|---|
| `a_peca_tinge_o_chao_a_volta_dela_e_so_a_volta_dela` | ganha luz · a luz é VERMELHA · **fora do campo é intacto ao byte** · MORRE com o campo vazio |
| `o_campo_do_chao_concorda_com_a_convergida` | a lei barata bate `8 192` direcções uniformes no miolo (pior `6,1 %`, barra `10 %`) |
| `a_orla_do_campo_esmorece_em_vez_de_cortar` | a maior queda entre amostras vizinhas ≤ `5 %` do pico, e fora do campo é ZERO |
| `a_luz_devolvida_atravessa_o_material_do_chao` | o pixel vale `material(campo)`, pela rota do produto |
| `a_borda_recebe_a_media_dos_vizinhos_de_fundo` | a lei da borda, por unidade |
| `a_cor_que_a_peca_devolve_ao_chao_e_a_mesma_nos_dois_motores` | **`100,000 %`**, pior `0` bytes, sobre `20 139` bytes movidos |

**Mutações: `6`, `4` sangram.** As duas que sobrevivem estão **medidas e nomeadas**:

- ⛔ **a saída antecipada de fora-do-campo é uma GUARDA DE ÍNDICE e não a lei** — a lei é a orla, que
  chega a zero exactamente na borda; trocar a saída por um `clamp` não muda um byte, porque a orla já
  lá pôs zero. *Ela lê-se como sobrevivência num relatório de mutação e não é: é uma segunda guarda a
  neutralizar a primeira.*
- ⛔ **a ponte pelo material do chão vale menos de UM byte nesta vista** — a resposta do chão branco é
  `~1,00` perto da incidência normal e `0,570` a rasar, e os pixels com sinal são os de incidência
  quase normal. A ponte é a expressão CERTA e é INOBSERVÁVEL aqui; o gate mede-o e afirma-o.

## §8 — ⚠️ As premissas MINHAS que a medição derrubou

1. *«o chão consulta as sondas da peça»* — a grelha satura e o pico fica fora dela (§2).
2. *«a queda é `h/r³` (solid angle × cos)»* — `E·r²` é que fica ~constante; o lóbulo lit da peça que
   o chão vê muda com a distância e cancela parte do cosseno.
3. *«a assadura de CPU custa ~14 ms»* — custava **`106,5`**, porque a marcha de raios é sequencial.
4. *«`branco.indirect(SoIrradiancia(E))` é a identidade»* — é `0,570 · E` numa direcção e `~1,00·E`
   noutra: *uma constante medida numa direcção arbitrária lê-se como propriedade do material*.
5. *«a coluna `0` do ecrã está fora do campo»* — com `half_extent 1,6` e um campo de `±3,0` no mundo,
   **o ecrã inteiro cai dentro dele**. *Uma régua que diz «longe» em píxeis não sabe onde o campo
   acaba.*
6. *«a 2.ª diferença mede o ruído do estimador»* — ela leu `0,78 · 1,96 · 1,38 · 1,66 · 1,44` sobre a
   escada de direcções, **sem tendência**: *o que não cai com a amostragem não é ruído da amostragem*.
7. *«as arestas da silhueta vêem o sangramento»* — elas leem campo `0,000003`, e levantar a peça
   PIORA (`112` arestas com campo a `0` de folga, `29` a `0,5`).

## §10 — ⛔⛔⛔ A LUZ ENCOSTADA desenhava RETÂNGULOS — e a grelha não era grossa, lia um PONTO (2026-09-24)

Report do dono, com foto da cena `=28` e a lâmpada encostada a um nó: *«qualidade do render melhor mas
ainda com áreas retangulares ruins»*. A sonda irmã (`diag_a_luz_encostada_com_chao`, as quatro
variantes do quadro assente) isolou o culpado: **com a cor devolvida ao chão ligada** aparecem
losangos do tamanho de uma célula; sem ela, não.

⛔ **A premissa óbvia — «a grelha é grossa demais» — caiu na primeira medição**
([`diag_a_grelha_da_luz_do_chao`](../../crates/ph2d-app-field3d/src/device_probes_w9_chao_grelha.rs),
contra uma assadura `192²`, no miolo que a câmera vê):

| grelha | pior / pico (bilinear) | p99 / pico |
|---:|---:|---:|
| `32²` | `61,1 %` | `9,4 %` |
| `64²` | `50,9 %` | `9,7 %` |
| `128²` | `57,1 %` | `8,3 %` |

⇒ **o erro não cai com a resolução**. A imagem da referência vista de cima
(`diag_o_campo_do_chao_visto_de_cima`) diz porquê: com a lâmpada a `~0,09` da peça, a mancha acesa é
pequena e age como uma **segunda lâmpada**, e os tubos projectam dela no chão **riscas de sombra**
muito mais finas que uma célula. A grelha lia a irradiância **num ponto** por nó: cada nó apanhava uma
risca ou um vão ao acaso — a grelha *dobrava* (alias) a feição fina em manchas do tamanho da célula —,
e a leitura bilinear desenhava os **vincos** das células por cima. Dois defeitos, duas curas:

1. ⭐⭐⭐ **O PRÉ-FILTRO** — os `128` raios de cada nó nascem **espalhados pela célula** (a sequência
   `R2`, determinística), com o ângulo sólido do cone de CADA raio. O nó passa a valer a **média da
   célula**, que é o que uma grelha precisa para não dobrar o que não consegue representar. **Custo
   zero**: os mesmos raios, com outras origens.
2. ⭐⭐⭐ **A B-SPLINE CÚBICA** na leitura (CPU e shader, os mesmos pesos) — `C²`, pesos não negativos
   que somam `1`. É a reconstrução que acompanha o pré-filtro nas grelhas de irradiância.

⛔ **A quase-interpolação cúbica** (coeficientes `(−v₋ + 8v − v₊)/6`, que tira o borrão próprio da
B-spline) foi construída e medida, e **perde**: o lóbulo negativo erra `21 %` junto ao contacto da bola
contra os `11 %` da B-spline no flanco (`o_campo_do_chao_concorda_com_a_convergida`), e na peça do
report empata no p99.

⚠️ **O preço, declarado:** as duas metades somam um borrão de `~0,65` célula. A barra da concordância
com a convergida subiu de `0,10` para `0,15` (medido `0,111`, no flanco que sobe para o pico) — a troca
de um desvio liso por um chão sem losangos. E o gate da tolerância na chave da cache passou a medir
**pela consulta** (o que o pixel lê: `0,72` byte), porque o nó colado ao contacto move `1,19` — os raios
pré-filtrados nascem também ali, onde a tolerância decide.

**Gates** ([`chao_grelha_gates.rs`](../../crates/ph2d-field-render/src/tests/chao_grelha_gates.rs)), os
dois com o CONTROLO da lei antiga dentro:
- `a_luz_do_chao_nao_desenha_as_celulas_da_grelha` — o salto de inclinação ao atravessar uma linha da
  grelha, num xadrez: B-spline `0,005`, bilinear `2,0`;
- `cada_no_da_grelha_do_chao_vale_a_media_da_celula` — no nó de toro com a lâmpada encostada, cada nó
  contra a média da célula de uma assadura `4×` mais fina: pré-filtrado pior `0,157` do pico, pontual
  `0,612`. ⚠️ O p95 **não** separa as duas (é o ruído das `128` direcções, igual nas duas).
- e a paridade `a_cor_que_a_peca_devolve_ao_chao_e_a_mesma_nos_dois_motores` ganhou o caso do nó: ⛔
  na bola (luz longe, campo liso) trocar os pesos do shader pelos bilineares **sobrevivia** — *uma
  paridade num campo liso não afirma nada sobre a reconstrução*.

Mutação **3 de 3** (a leitura bilinear na CPU · o nó pontual · a leitura bilinear no shader).

## §9 — ⏳ O que fica

- **a cache e o kernel** do §6, com os preços;
- **o campo é de UMA peça** — ele sai da bola que envolve a cena inteira, logo duas peças afastadas
  partilham um campo grosso. A cura tem nome (um campo por peça, somados) e **não** foi medida;
- **o `GROUND_BOUNCE_SPAN`** (`6` raios) e o `GROUND_BOUNCE_FADE` (`0,25`) são os dois números desta
  wave **sem tabela por baixo**: o vão sai de a verdade valer `0,28 %` da luz livre a `5,75` raios, e
  a orla de ser a faixa onde isso vale menos de `5/255`. ⚠️ *Quem os mexer mede-os como os outros
  dois foram medidos.*

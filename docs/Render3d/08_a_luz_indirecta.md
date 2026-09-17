# 08 — A LUZ INDIRECTA (`W5`): a régua, o preço e a rota

> A wave que o [`03_o_plano.md`](03_o_plano.md) §W5 chama de *«a que decide se a engine é bonita, e a
> que só nós podemos fazer assim»*. O que a torna nossa está no [`02`](02_o_estado_da_arte.md) §5.1:
> **o Lumen aproxima cada malha por um campo de distância pré-cozido e nós SOMOS o campo** — a GI
> traça contra o modelo verdadeiro, sem proxy e sem o erro dele.

## §1 — ⭐⭐ A leitura que tornou a wave incremental

Este módulo já calculava metade do problema e ninguém lhe tinha chamado isso.

A luz que chega a um ponto pelo hemisfério tem **duas parcelas**: a que vem do **céu** e a que vem
das **superfícies**. A [`occlusion`](../../crates/ph2d-field-render/src/occlusion.rs) mede a
primeira — *quanto do céu chega* — e o canal dela **ATENUA**. A segunda **SOMA**.

⇒ **a oclusão que este módulo já ship é a luz indirecta com a cor do ricochete posta a PRETO**, e a
`W5` troca esse preto pela superfície que bloqueou.

⚠️ **A diferença é a PERGUNTA, não o raio:** a oclusão pergunta *se* o raio bateu; esta pergunta *no
quê*, porque precisa de lhe perguntar a cor.

## §2 — O substrato que faltava: uma marcha de raios SOLTOS

Até 2026-09-17 a crate sabia lançar raios de três maneiras e **as três respondiam sim/não**: a
marcha da câmera (que diz onde parou, mas só para os raios que a lente gera), a
[`march_shadow_to`] e a [`march_cone_to`] — as duas últimas devolvem visibilidade.

⭐ A [`march_rays`] é o **mesmo código**, não um irmão: a câmera era consultada numa linha só
(`cam.ray_at_plane`), logo a marcha de ecrã passou a ser o caso em que as origens e as direcções
saem da lente. *Um segundo marchador daria duas respostas para «onde este raio para?»* — o passo, a
cerca de `hit`, o `shrink`, as fatias de profundidade e a normal por diferença central mantidos
iguais à mão.

## §3 — A RÉGUA: a caixa de Cornell, no nosso campo

[`tests/cornell.rs`](../../crates/ph2d-field-render/src/tests/cornell.rs) — cinco paredes, duas
peças que pousam no chão, aberta para a câmera, material por folha e o **especular a zero** (ela é
um teste de transporte difuso; um segundo lóbulo poria um segundo caminho de luz na régua).

⭐ **A propriedade que ela mede não é «parece melhor»: é o SANGRAMENTO DE COR**, e o sinal é
conhecido **antes** de medir — o chão junto da parede vermelha fica mais vermelho que verde, e junto
da verde o contrário.

⚠️ **Divergências declaradas** contra a caixa publicada: a luz é **pontual** e não uma área no
tecto; as reflectâncias são as clássicas e não o espectro medido a 4 nm. *O que esta régua afirma é
TRANSPORTE, nunca paridade fotométrica com o laboratório de Cornell.*

### §3.1 — A referência convergida, e a barra que sai dela

A irradiância indirecta num ponto, por força bruta, com amostragem por cosseno. Ela **partilha o
material** com o produto (chama o `Surface::direct` e a mesma lei da lâmpada) e **não partilha o
ambiente**, que é a variável que a wave aproxima — *uma referência que escrevesse a sua própria lei
de luz directa mediria a diferença entre as duas leis e chamar-lhe-ia erro da GI*.

| amostras | tom junto da VERMELHA | tom junto da VERDE |
|---:|---:|---:|
| `64` | `+0,0628` | `−0,0979` |
| `1 024` | `+0,0758` | `−0,0373` |
| `4 096` | `+0,0833` | `−0,0377` |
| `65 536` | **`+0,0824`** | **`−0,0377`** |

⇒ a `4 096` o resíduo é `≤ 0,001`, e a barra do gate é **metade do valor convergido** — `40×` o
ruído do estimador. ⛔ **A 1.ª barra foi escrita de cabeça (`0,05`) e reprovou uma medição
correcta.**

### §3.2 — ⛔⛔ Dois defeitos que a régua apanhou antes de haver GI

1. **O gate da fixtura parava em «90 % dos pixels acertam a caixa»** — que é verdade **também para o
   lado de FORA de uma caixa fechada**. Apertado para *«quantas FACES aparecem»*, ele acusou que a
   câmera de omissão (três quartos) deixa a parede do fundo invisível: a caixa de Cornell vê-se de
   **frente**.
2. **A referência media A CÂMERA.** A [`march_rays`] devolve a normal em espaço de **VISTA** e ela
   era usada como se fosse de MUNDO ⇒ a irradiância mudava **45 %** só por a câmera rodar. ⚠️ E dava
   o **sinal certo à mesma**, porque aquela câmera olha de frente e ali os dois referenciais quase
   coincidem: *um defeito de referencial escondido pela fixtura que o não exercita*. Gate:
   `a_referencia_nao_depende_de_onde_a_camera_esta`, que passou de `45 %` para **`0,0000`**.

⭐ E a cura expôs que a conversão estava escrita **à mão em três sítios** do produto e dos testes —
o passe da sombra, o da oclusão e uma sonda —, cada um a **declarar por escrito** que era a mesma
conta que o vizinho faz. ⇒ `ViewBasis::view_to_world`, com os três a passar por ela.

## §4 — O que shipa: o `bounce_pass`, e o gate de PIXEL

[`bounce.rs`](../../crates/ph2d-field-render/src/bounce.rs). O conjunto de direcções e o peso
`max(0, n·d)` são os **mesmos da oclusão** — dois conjuntos diferentes fariam a soma deixar de ser o
integral de coisa nenhuma —, e o resultado mora no [`Shadows`], onde o doc já escrevia a lei (*«um
segundo canal ao lado faria o pintor perguntar duas vezes a mesma coisa»*).

⚠️ **`bounce_at` devolve `[0,0,0]` fora de alcance, que é o OPOSTO das irmãs**: uma sombra que não
foi calculada é ausência de sombra; uma luz que não foi calculada é **ausência de luz**. *Inventar
luz é a única das duas que acende o que devia estar escuro.* Canal vazio ⇒ o quadro de sempre, **ao
bit**.

O gate mede **bytes**, pelo caminho do produto:

```
o chão da IMAGEM (472 px à esquerda, 169 à direita)
  SEM ricochete   esquerda +0,0000   direita +0,0000
  COM ricochete   esquerda +0,0247   direita −0,0709
```

⛔⛔ **E o CONTROLO reprovou primeiro, sobre produto CORRECTO:** sem ricochete o chão lia `+0,0082`
e `−0,0271` — tingido, num chão **branco** sob uma lâmpada **branca**. O diagnóstico mostrou que os
pixels eram mesmo chão e iam até encostar à parede: a régua estava a medir a **fronteira de cor
suavizada** do `Surfaces::mix_of`, cuja largura é um pixel no mundo. ⇒ a região exclui **três
larguras de pixel**, derivadas do quadro.

## §5 — ⏱️ O PREÇO, medido (`--release`, CPU a `95 %` ociosa, mínimo de 3)

| cena (`640×360`) | traçado | sombra | ricochete `8` | `16` | `32` | `64` |
|---|---:|---:|---:|---:|---:|---:|
| **caixa fechada** (`153 360` px de peça) | `5,39 ms` | `24,55` | `376,8` | `791,9` | `1 636,2` | `3 751,2` |
| **peça no aberto** (`20 008` px) | `2,04 ms` | `0,19` | **`8,8`** | `17,8` | `36,3` | `79,1` |

⭐ **As duas cenas fazem a fronteira da resposta**, e a diferença é `40×`: o custo segue os pixels de
peça **e** o que os raios encontram. Numa caixa fechada todo raio bate e todo acerto paga ainda um
raio de sombra; no aberto a maioria escapa pela bola.

⇒ **no caso do modelador, `8` direcções custam meio quadro** — o que faz esta passagem caber no
quadro **assente**, nunca no de movimento.

## §6 — ⏱️ E QUANTAS DIRECÇÕES ela precisa

A régua é o tom do chão da imagem, contra a mesma imagem a `1 024` direcções (`+0,0171` / `−0,0779`):

| direcções | erro ESQ | erro DIR |
|---:|---:|---:|
| `4` | `−0,0171` | `−0,0632` |
| `8` | `−0,0273` | `−0,0082` |
| `16` | `−0,0058` | `−0,0134` |
| `32` | `+0,0046` | `−0,0137` |
| `64` | `+0,0076` | `+0,0070` |
| `128` | `−0,0004` | `+0,0064` |
| `256` | `+0,0002` | `−0,0025` |
| `512` | `+0,0002` | `−0,0015` |

⛔⛔ **O erro NÃO encolhe monotonamente, e isso é uma propriedade do estimador, não ruído de
medição:** o conjunto de direcções é **determinístico** ([`cone_dir`], um reticulado de Fibonacci
sobre a esfera), logo mudar o número muda o conjunto **inteiro** — cada contagem é uma amostra
enviesada diferente, e não um refinamento da anterior. O envelope encolhe (`0,063` a `4` → `0,013` a
`16` → `0,006` a `128` → `0,0025` a `256`), a sequência não.

⚠️ **E ele converge devagar por construção:** as direcções são uniformes na esfera e o cosseno entra
como **PESO**, enquanto a referência amostra **por cosseno**. Importância dava a mesma resposta com
muito menos direcções — ⛔ e é exactamente o que a [`cone_dir`] recusa por escrito, porque uma base
tangente por pixel tem uma **descontinuidade** que com um conjunto fixo vira uma **costura desenhada
na peça**. *Uma recusa medida da oclusão vale igual aqui: as duas são o mesmo integral.*

## §7 — ⭐⭐⭐ A rota, que sai das duas tabelas e não de uma opinião

O plano nomeia **cascatas de radiância com sondas esparsas** como candidato principal, e a medição
de [`05` §29](05_o_modo_render_do_modelador.md) já tinha mostrado que a força bruta **por pixel** não
cabe. As duas tabelas acima dizem onde ela cabe:

- **`32`–`64` direcções** põem o erro do lado forte em `~10 %` do sinal;
- **no aberto isso custa `36`–`79 ms`**, que é `2`–`5` quadros.

⇒ **o caminho não é um algoritmo novo: é a máquina que a oclusão JÁ TEM.** Ela reparte as direcções
por **passagens do quadro assente** ([`occlusion_slice`] + [`refine_occlusion`]), e o ricochete é a
outra metade do mesmo integral — *a imagem afina enquanto a mão está parada*, que é o idioma de toda
viewport de render.

⏳ **O que fica por fazer, por ordem:**

1. ✅ **o ricochete em fatias — FEITO (§8)**; ⛔⛔ **e a segunda metade desta linha — *«é o que o põe
   no produto»* — está REFUTADA pelo report do dono (§11).** Ela põe-no no caminho de **referência**,
   que nasce DESLIGADO;
2. o segundo **ricochete** (hoje a luz que sai do ponto acertado é só a directa dele);
3. a parcela **especular** do indirecto (hoje é só o céu: uma irradiância por pixel não tem
   direcção, e o lóbulo especular pergunta por uma);
4. o **dispositivo** — esta passagem é CPU, e o quadro assente do modelador é da placa desde a `§36`.

## §8 — ⭐⭐⭐ AS FATIAS: o ricochete entra no produto pelo quadro assente

[`refine.rs`](../../crates/ph2d-field-render/src/refine.rs) — a `refine_occlusion` **mudou de nome
e de trabalho**: hoje é a `refine_hemisphere`, e cada passagem acrescenta **uma direcção às DUAS
metades**.

### §8.1 — ⚠️⚠️ Porque é UM laço e não dois

As duas metades saem do mesmo conjunto de direcções e do mesmo peso `max(0, n·d)`. Uma publicação
em que o céu já consumiu `k` direcções e o ricochete outras tantas **diferentes** somaria dois
hemisférios distintos — *a partilha do conjunto só é uma lei enquanto as duas o percorrerem no mesmo
passo*, e dois laços seriam a forma mais fácil de as deixar divergir sem ninguém dar por isso.

⭐ **Sem materiais ou sem lâmpadas o ricochete degenera para o canal vazio e o quadro fica o de
sempre, AO BIT** — é isso que dispensa um interruptor ao lado da porta, e há gate a afirmá-lo (com o
céu a ser comparado byte a byte entre as duas corridas).

### §8.2 — ⏱️ A CADÊNCIA, e é ela que autoriza o mesmo `k`

A [`sonda_o_preco_do_ricochete`](../../crates/ph2d-field-render/src/tests/cornell.rs) mediu a
sequência INTEIRA (§5) e ela não cabe num quadro. O refinamento paga **uma** direcção:

| cena | UMA passagem: céu | ricochete | razão |
|---|---:|---:|---:|
| caixa fechada `640×360` | `18,23 ms` | `28,62` | `1,6×` |
| caixa fechada `1920×1080` | `186,33` | `270,60` | `1,5×` |
| peça no aberto `640×360` | `0,40` | `1,63` | `4,0×` |
| **peça no aberto `1920×1080`** | **`4,88`** | **`19,06`** | **`3,9×`** |

⭐⭐ **`1,5`–`4` vezes o que a metade do céu JÁ paga na mesma passagem** — e não as dezenas que a
tabela do §5 sugeria, porque ali o ricochete inteiro era comparado com um traçado e aqui é comparado
com o cone da oclusão, que também marcha. ⇒ no caso do modelador o refinamento passa de `~0,23 s`
para `~1,15 s`, publicando `48` vezes pelo caminho e **cancelável em cada uma**.

⚠️ **A régua é a PASSAGEM e não o total:** o que o artista sente é o intervalo entre duas imagens, e
a granularidade do cancelamento é uma passagem.

### §8.3 — ⭐ O borrão é UMA lei com dois consumidores

O doc do [`blur_occlusion`] já escrevia que o que ele suaviza hoje **não é ruído de amostragem** (a
oclusão é determinística e há dois gates a afirmá-lo) mas as **estrias do conjunto discreto de
direcções**. O ricochete corre no MESMO conjunto ⇒ tem a mesma assinatura ⇒ a mesma cura. A guarda
da normal saiu para uma porta ([`para_cada_vizinhanca`]) e os dois canais só somam.

⚠️ **E o gate que prova isso não podia ser o do refinamento:** ele compara a saída do laço com
`blur_bounce(bounce_pass(..))`, e as duas rotas passam pela MESMA função — mutá-la muda os dois lados
e ele fica **verde**. *Uma igualdade entre duas rotas que partilham uma porta não afirma nada sobre
essa porta.* ⇒ a régua é o **céu**: cada canal do ricochete tem de sair exactamente como o
[`blur_occlusion`] o devolveria sozinho.

## §9 — ⛔⛔ O que a construção REFUTOU

### §9.1 — A lei da acumulação vale para fatias de UMA direcção, não para uma partição qualquer

O doc do [`occlusion_slice`] prometia por escrito que *«somar todas as fatias dá EXACTAMENTE o que a
`occlusion` devolve»*. **Falso fora da partição que o gate dela corria** — medido em `3 + 5` contra
`8`, nas duas metades:

| | pior relativo | a associatividade permite |
|---|---:|---:|
| ricochete | `2,541e-7` | `8,345e-7` |
| céu | `2,392e-7` | `8,345e-7` |

A razão é só **associatividade**: `(a₀+a₁+a₂) + (a₃+…+a₇)` é uma dobra em **árvore** e a sequência
inteira é uma dobra à **esquerda**. Fatias de uma direcção reproduzem a dobra à esquerda ⇒ **ao
bit**; uma partição desigual não. ⇒ a barra dessa metade é **derivada do número de parcelas**
(`(total − 1) · f32::EPSILON`) e os dois gates passaram a correr **as duas** partições.

*Um gate que prova o caso que o produto usa não autoriza a frase geral escrita ao lado dele.*

### §9.2 — ⛔⛔⛔ Uma asserção que estava verde por GEOMETRIA DA CÂMERA, e não pela lei

O gate *«uma fatia que começa depois do fim da sequência é vazia»* passava na caixa de Cornell — e
**passava com a cerca do laço apagada**: a mutação SOBREVIVEU.

A sonda diz porquê. Para `k ≥ total` a [`cone_dir`] devolve `[0, 0, z]` com `|z| > 1` (o raio sai
`0` porque `1 − z²` fica negativo), ou seja **`−z` do mundo**; e um pixel VISÍVEL satisfaz
`n·olho > 0` **por construção**, logo com a câmera daquela caixa (olho em `+z`) todo pixel de peça
tem `n_z > 0` e pesa aquela direcção com `≤ 0`. Medido: **`0`** pares `(pixel, direcção fora)` com
peso positivo, em `2 304` pixels de peça.

⇒ a fixtura passou a ser uma **esfera vista de yaw `135°`**, onde metade dos pixels visíveis tem
`n_z < 0` — `3 496` pares em `944` px —, e **o controlo vive DENTRO do gate**: se a fixtura deixar de
conter o fenómeno, ele reprova alto em vez de ficar verde a medir nada.

### §9.3 — E uma barra minha reprovou sobre produto correcto, outra vez

*«a 1.ª passagem já acende mais de `100` pixels»* — escrito de cabeça, e a medição deu **`97`** de
`1 024` (uma direcção de `48` ilumina `~9,5 %` da caixa). A barra é hoje **metade do medido**, como
a do §3.1. *Uma barra escrita antes da medição mede a minha expectativa.*

## §10 — ⏳ O que fica por fazer

Na ordem do §7, menos o item 1 — mais o que as fatias deixaram nomeado:

- **o CHÃO invisível não recebe ricochete.** Os pixels que o mostram falham a peça
  (`g.hit == false`), logo não entram em fatia nenhuma: ele recebe o céu pelo passe da sombra e mais
  nada. *A cor que a peça devolveria ao chão à volta dela é a wave seguinte, e a marcha dela é outra
  — o ponto de partida está no plano, não no campo.*
- **a suavização do ricochete não foi RE-MEDIDA contra a lei nova**, exactamente como o doc do
  [`blur_occlusion`] já declara sobre si próprio. A premissa (*baixa frequência dentro de uma
  superfície*) é a mesma; o **quanto** não está medido nos dois canais.

[`blur_occlusion`]: ../../crates/ph2d-field-render/src/occlusion.rs
[`para_cada_vizinhanca`]: ../../crates/ph2d-field-render/src/occlusion.rs
[`occlusion_slice`]: ../../crates/ph2d-field-render/src/occlusion.rs
[`cone_dir`]: ../../crates/ph2d-field-render/src/occlusion.rs

## §11 — ⛔⛔⛔ O REPORT DO DONO: *«não funciona, não clareia»*

E ele tem razão: **o laço que a `§8` estendeu não corre no produto.**

### §11.1 — A causa, com o endereço

O quadro assente só refina se a [`preview::refines_occlusion`] disser que sim, e ela é

```rust
antialias && plate_parked && cpu_occlusion_enabled()
```

— e a terceira lê `PH2D_FIELD_AO`, que **não está posta**. O doc daquela função escreve a razão por
extenso, e ela é uma decisão do próprio dono:

> ⛔⛔⛔ **O REFINAMENTO DE CPU NASCE DESLIGADO — por veredito do dono e por medição.**
> *«funciona mas com aspecto ruim, muito demorado e em etapas estranhas»* · *«mover os objetos ficou
> muito lento»* … **a cura não é afinar isto — é o DISPOSITIVO** (`1 998 ms` contra **`5,00 ms`**,
> `399,5×`).

⇒ desde então o quadro assente do modelador vem da **placa**: a
[`DeviceGbuffer::to_cpu`](../../crates/ph2d-field-gpu/src/trace_to_cpu.rs) entrega `t`, a normal, uma
sombra por lâmpada e a **oclusão** — e **nenhum canal de ricochete**. O laço de CPU fica como
*referência*, que é o molde *dois motores, uma lei* desta casa.

### §11.2 — ⛔⛔ E a premissa REFUTADA é minha, escrita na §7 deste mesmo doc

A fila dizia *«1. o ricochete em fatias … **é o que o põe no produto**»* e *«4. o dispositivo»*.
**A ordem está invertida:** as fatias põem-no na referência, e quem o põe no produto é o
dispositivo. *A `§7` foi escrita a olhar para a arquitectura do traçado de CPU, que deixou de ser o
que o artista vê — e nada neste doc me obrigou a reconferir.*

### §11.3 — ⛔⛔⛔ E o gate de costura que eu escrevi é a MESMA armadilha, um nível acima

O [`render_bounce_seam_tests`](../../crates/ph2d-app-field3d/src/render_bounce_seam_tests.rs) prova
que a thread do quadro assente passa **os materiais e as lâmpadas** ao refinamento — e é verdade, e
é **inútil**, porque a chamada inteira está atrás de uma bandeira desligada.

⚠️ É a família que o `CLAUDE.md §5.0` nomeia (*«um gate pode provar que o dado existe e que ele
fecha, e não provar que ele CHEGA ao consumidor»*) com uma volta a mais: **eu gateei a costura de
uma chamada que não acontece.** *Antes de gatear que um valor chega a uma porta, meça se a porta é
chamada no caminho de omissão* — um `git grep` pelo predicado que a governa responde em dez
segundos, e eu não o corri.

### §11.4 — ⏳ O que a wave seguinte tem de fazer

O ricochete **no dispositivo**, que é o item 4 da fila e afinal era o pré-requisito do item 1. O
levantamento diz que a peça grande já lá está:

| o que o ricochete precisa | no dispositivo hoje |
|---|---|
| marchar um raio e achar onde ele parou | ✅ [`marcha(r) -> vec4(t, normal)`](../../crates/ph2d-field-gpu/src/trace_wgsl.rs) |
| o conjunto de direcções | ✅ `direccao_do_cone`, o `cone_dir` linha a linha |
| a sombra no ponto acertado | ✅ `visivel(origem, dir, t_max, dureza)` |
| o material do ponto acertado | ✅ a tabela `materiais` + `ler_mat` do [`paint.rs`](../../crates/ph2d-field-gpu/src/paint.rs) |
| a luz directa daquele material | ✅ *«a luz que UM material devolve ao olho»*, o `radiance` da CPU |
| **um canal por pixel para o levar** | ⛔ falta — o buffer de luz tem passo `1 + n_lâmpadas` |
| **a paridade contra a referência** | ⛔ falta — é o que a `§8` acabou de tornar possível |

⇒ *a wave é o laço e o canal, não o motor* — e a referência de CPU contra a qual ela se mede é
exactamente o que esta jornada construiu.

[`preview::refines_occlusion`]: ../../crates/ph2d-app-field3d/src/preview.rs

## §12 — ⭐⭐⭐ O RICOCHETE NO DISPOSITIVO: onde o artista o vê

A `§11` disse que o laço de CPU não corre no produto. Esta é a wave que o põe lá.

### §12.1 — Ele vive no passe que PINTA, e isso é uma decisão

| candidato | tem a marcha? | tem os materiais? |
|---|---|---|
| o kernel da **marcha** (`centro_e_luz`) | ✅ | ⛔ |
| o passe que **pinta** | ⛔ (tinha) | ✅ |

O ricochete precisa dos **dois**: ele marcha um raio e depois pergunta *de que cor é o que eu
acertei*. ⇒ o passe que pinta recebe as **leis** da marcha (o campo, a `marcha`, a `visivel`, o
conjunto de cones), e os dois **kernels** dela ficam de fora — *um módulo com pontos de entrada que
ninguém despacha é código que não se apaga porque compila*. Zero ligações novas: o pintor já ligava
as grades das esculturas e o `k`.

⚠️ **E a marcha ganhou o alcance por ARGUMENTO** (`marcha_ate`): um raio de câmera anda até
`t_max`, um raio de hemisfério anda até sair da bola que contém a peça. *Uma segunda marcha para a
segunda pergunta seria a segunda resposta a «onde este raio para?».*

### §12.2 — ⭐⭐ O ambiente passa a ser um DESPACHO, e a álgebra autoriza-o

Na CPU o ricochete entra por uma **segunda chamada** à lei indirecta, com um ambiente falso
(`SoIrradiancia`: irradiância = o ricochete, radiância = `0`). No dispositivo a lei indirecta lê
duas funções globais ⇒ elas passam a despachar:

```wgsl
fn env_irradiance(n) { if (ambiente_e_ricochete) { return ricochete; } return ceu_irradiance(n); }
fn env_radiance(..)  { if (ambiente_e_ricochete) { return vec3(0.0); } return ceu_radiance(..); }
```

⚠️⚠️ **Tinham de ser DUAS chamadas e não um ambiente somado**, e a razão não é de gosto: a parcela
do céu leva a oclusão por cima (`* ceu_vis`) e a do ricochete não. *A oclusão é a sombra do CÉU, e a
luz que vem das superfícies não é céu.*

⛔ É por isso que o `PaintSetup::env_source` passou a declarar `ceu_*` em vez de `env_*`: quem manda
no ambiente é o despacho, e o céu de quem chama é **um dos dois braços** dele.

### §12.3 — ⭐⭐ Ele é um CANAL, e não um valor local

O ricochete precisa de ser **suavizado** pela mesma razão que a oclusão (as estrias do conjunto
discreto — `§8.3`), e suavizar precisa dos **vizinhos**. ⇒ um valor calculado e consumido na mesma
invocação não tem onde ser suavizado:

- o passo do buffer de luz passa de `1 + n_lâmpadas` para **`1 + n_lâmpadas + 3`**;
- uma passagem nova (`pinta_ricochete`) calcula e **guarda**;
- a pintura lê a vizinhança `3×3` guardada pela normal — a mesma lei do céu.

⚠️ **E o fundo recebe `0` nos três slots, não `1`:** uma sombra que não foi calculada é *ausência de
sombra*; uma luz que não foi calculada é **ausência de luz**.

### §12.4 — ⭐⭐⭐ A prova: os dois motores, e o par de gates que se controla

| gate | o que afirma |
|---|---|
| `a_imagem_do_dispositivo_e_a_da_cpu` | a imagem do dispositivo **é** a da referência: `100,000 %` dos canais a `≤ 1` nível, pior `1` |
| `o_ricochete_chega_a_imagem_do_dispositivo` | e ela é **mais clara** do que a mesma imagem sem ricochete: `179,4 → 184,4` níveis num CANTO |
| `o_quadro_de_movimento_nao_paga_o_ricochete` | com a bandeira em baixo ela volta a ser a de sempre |

⛔⛔ **E a fixtura do segundo teve de ser TROCADA:** com a peça da paridade — três formas convexas
no aberto — o brilho subia `+0,513` níveis, que é ruído: ali quase todo raio do hemisfério
**escapa**. Com um CANTO (duas placas em ângulo recto, cada uma a ver a outra a meio hemisfério) ele
sobe `+4,935`. *Não era o produto — era a fixtura a não conter o fenómeno*, pela segunda vez nesta
wave (a primeira foi a `§9.2`).

⛔⛔ **O segundo existe porque o primeiro é cego ao caso que abriu esta wave:** a referência de CPU
dele passou a calcular o ricochete também, logo *se ninguém o calculasse em lado nenhum eles
continuariam a concordar* — preto contra preto. É a mesma forma do `§11.3`, e desta vez está
gateada.

### §12.5 — ⚠️ E ele viaja na bandeira que JÁ EXISTE

O quadro assente (`antialias`, a lei da W73 — *grosso a mexer, nítido ao assentar*) ganha o
**quarto** passageiro, a seguir ao contorno fino, ao anti-serrilhado e à sombra directa.

⛔⛔ **Sem isso o ricochete corria no quadro de MOVIMENTO**, que é exactamente a regressão que o dono
já reprovou uma vez (*«mover os objetos ficou muito lento»*). Com a bandeira em baixo a passagem não
compila, não despacha, o canal fica vazio e o quadro que a mão arrasta é **byte-idêntico** ao de
hoje.

### §12.6 — ⏳ O que fica

- ✅ **o custo está MEDIDO** (ver a `§12.7`);
- **o caminho de LEITURA continua sem ricochete** (`DeviceGbuffer::bounce` vem a zero) — quem enche
  o canal é a passagem do pintor, e aquele caminho existe para quando o pintor não corre. O canal
  atravessa-o na mesma, e um canal vazio é o quadro de sempre ao bit;
- o **chão invisível** continua sem receber ricochete (`§10`);
- e o **segundo** ricochete e a parcela **especular** continuam na fila (`§7`).

### §12.7 — ⏱️ O PREÇO, e a razão que ele fecha

Medido a `1920×1080` com `48` direcções, `load 2,5`–`5,0`, mínimo de 7 corridas — A/B pela fonte
(`ao_rays: 0` no `PaintSetup` desliga só a passagem do ricochete):

| | min | mediana |
|---|---:|---:|
| marcha + sombra + oclusão + pintura | `6,14 ms` | `6,62` |
| **e mais o ricochete** | **`9,86`** | `10,44` |

⇒ **`+3,72 ms`**, `1,61×` o quadro assente — e o quadro inteiro continua **abaixo** de um de
`16,7 ms`, noutra thread.

⭐⭐⭐ **A mesma resposta na CPU custava `~1,15 s`** (a `§8.2`: `19,06 ms` por direcção × `48`) ⇒
**`309×`**. É o número que fecha a `§11`: *o caminho mais lento definia o tecto do mais rápido, no
módulo cuja razão de existir é o mais rápido* — e por isso o refinamento por fatias da `§8` fica
onde pertence, como **referência**, e não como o que o artista recebe.

⚠️ **E a flake que a rodada apanhou:** o `na_faixa_do_produto_a_placa_ganha_com_margem` reprovou com
outra sessão a correr um fan-out nesta máquina, e passa **3 de 3** a `load 3,0`–`4,3` com a CPU a
`96`–`99 %` ociosa (razões `3,90×`–`8,63×` contra a barra de `2`). ⛔ Ele mede o quadro de
**MOVIMENTO**, onde o ricochete nem despacha — *o diff não toca no caminho que ele mede*. Membro da
família do `CLAUDE.md §5.0`.

---

## §13 — ⛔⛔⛔ O SEGUNDO REPORT DO DONO: *«funciona, é rápido, mas é de baixa qualidade (como se fosse muitas sombras duras)»*

**2026-09-17**, com a foto do interior do vaso (cena `=5`): **arcos concêntricos** dentro dele.

### §13.1 — A primeira leitura, e porque ela decide a cura

⚠️ **Ruído é desvio INDEPENDENTE por pixel; aquilo são ARCOS, que é desvio CORRELACIONADO.** A
distinção não é académica: a cura do ruído está **proibida por medição** desde 2026-09-15, com o
report ANTERIOR do dono na mão (`ph2d_field_render::occlusion::cone_dir`) — sortear direcções por
pixel foi **apagado** porque o sorteio semeado no índice do pixel fazia a peça **ferver** ao rodar a
câmera.

⇒ *a pergunta não é «como suavizo isto», é «porque é que o céu, com AS MESMAS `48` direcções, não tem
terraços e o ricochete tem».*

### §13.2 — ⭐ A régua: a metade do CÉU é o CONTROLO da metade das SUPERFÍCIES

As duas metades correm o mesmo conjunto de direcções, com o mesmo peso `max(0, n·d)`, na mesma cena,
no mesmo `k`, e passam pelo mesmo borrão ⇒ **tudo o que difere entre as duas medições é o que se
pergunta por direcção**. Uma régua que as leia lado a lado não pode ser acusada de medir a cena, a
câmera, o número de direcções ou o borrão.

A grandeza é a **QUEBRA** — a segunda diferença ao longo de uma linha ([`ph2d_field_render::banda`]):
ela é cega a toda rampa suave (a irradiância indirecta sobe e desce numa parede curva, e isso é
sinal) e acende onde a resposta **salta**.

### §13.3 — ⛔ A 1.ª hipótese foi construída inteira e REFUTADA por medição

*«A metade das superfícies pergunta um RAIO (indicador binário) onde o céu pergunta um CONE
(cobertura contínua com dureza `1/(n·d)`)»* — verdade sobre o código, e **não é a causa**.

A marcha do cone foi escrita (`march_cone_rays`: cobertura `1 − vis` + o representante na
aproximação mais próxima, recuado ao longo do gradiente para aterrar na superfície) e medida:

| cena | com raio binário | com cobertura de cone |
|---|---:|---:|
| Cornell fechada, 48 dir | `0,8584` | `0,8229` |
| vaso do dono, 48 dir | `0,5933` | `0,5877` |

⇒ **`1 %`–`4 %`.** Numa caixa fechada não existe raio que escape, e mesmo no vaso a esmagadora
maioria das direcções acerta solidamente ⇒ *a cobertura é praticamente binária na prática, e a lei
do cone degenera na do raio.* **REVERTIDA.**

### §13.4 — ⭐⭐⭐ A decomposição, que nomeou DUAS causas — uma por cena

A sonda `sonda_de_onde_vem_o_degrau` abre o pior terraço **direcção a direcção**.

**Na caixa de Cornell**, UMA direcção de `48` faz `64,8` de um salto total de `65,6`:

| dir | contribuição A/B/C | `r` à lâmpada | `1/r²` |
|---:|---|---|---|
| **13** | `26,3` / **`59,9`** / `28,8` | `0,079` / **`0,060`** / `0,077` | `222` / **`385`** / `236` |
| 24 | `0,37` / `0,00` / `0,00` | `0,53` / `0,55` / `0,66` | `5,0` / `4,6` / `3,2` |

⇒ **o polo `1/r²` de uma lâmpada PONTUAL**, amostrado por um conjunto fixo como um pico: fireflies —
e, por o conjunto ser o mesmo em todo pixel, fireflies **COERENTES**, que desenham arcos.

⚠️⚠️ **Mas essa lâmpada NÃO é a do produto.** A `ph2d_app_field3d::lights::opening_distance` põe a
primeira luz a `2 × half_extent` do alvo, **fora da peça**; medido no vaso, a superfície mais próxima
dela está a `0,99` ⇒ `1/r²` ≤ `~1` e **não há polo nenhum**. *Curar o polo da caixa de Cornell podia
não tocar num pixel do que o dono fotografou* — a lei que esta casa já pagou cinco vezes: **a
fixtura tem de ser a da cena que o dono usou** (`ph2d_app_field3d::render_bounce_vaso_tests`).

### §13.5 — ⭐⭐⭐ A causa NA PEÇA DO DONO: o LÓBULO ESPECULAR do ponto acertado

O controlo na mesma corrida — o mesmo material com o especular desligado — parte a tabela em duas:

| direcções | céu | ricochete | **ricochete sem especular** |
|---:|---:|---:|---:|
| `16` | `0,0332` | `3,9124` | **`0,9689`** |
| `32` | `0,0292` | `2,4566` | **`0,8272`** |
| `48` | `0,0308` | `0,8719` | **`0,5877`** |
| `96` | `0,0303` | **`6,2921`** | **`0,4502`** |

⭐⭐⭐ **A coluna do meio é CAÓTICA — `96` direcções leem PIOR que `48`.** A da direita é **monótona**,
que é o que um estimador consistente faz. ⇒ *o lóbulo especular é uma quase-delta em direcção, e um
recolhedor de `48` direcções FIXAS não a amostra: carregá-lo faz o estimador deixar de convergir.*

⇒ **o ricochete recolhe a parte DIFUSA do ponto acertado** ([`ph2d_material::Surface::matte`]).
⚠️ **Divergência DECLARADA:** a luz que sai de um ponto inclui mesmo o especular dele; o que este
produto não faz é **transportá-lo** por um recolhedor difuso — a mesma fronteira que o cabeçalho do
`ph2d_field_render::bounce` já declarava do lado de cá (*«a parte DIFUSA … o especular indirecto
continua a ser o céu»*), agora também do lado de lá.

⚠️ **Ela viaja EMPACOTADA para o dispositivo** e não se deriva no shader: o pacote é **preparado**
(o `specular_weight` entra no `modulated_eta_s`, no `main_alpha` e no escurecimento da base), logo
zerar o campo `a` do `emission_specweight` **não** é a superfície que a `matte()` prepara.

### §13.6 — ⭐⭐ A segunda metade: o borrão passa a ser DUAS passagens

A tabela está no doc-comment de [`ph2d_field_render::BOUNCE_BLUR_PASSES`]. O resumo: `2` é onde a
coluna do **desvio contra a convergida** tem o **mínimo** (`15,44 %`), e acima dela os terraços ainda
caem mas a exactidão **inverte**.

⛔ **E um núcleo maior numa passagem só não serve**, apesar de custar o mesmo despacho: `5×5` e `7×7`
baixam o `p99` e **sobem o MÁXIMO** (`1,81 → 2,84` / `3,21`). *A guarda da normal aplicada a CADA
salto é transitiva — define uma vizinhança GEODÉSICA, que não atravessa um vinco.* À-trous (2.ª
passagem com vizinhos afastados) é pior ainda (`0,4695` / `0,5398`).

⚠️ No dispositivo isso custa **três** slots a mais por pixel no canal de luz e um despacho novo
(`borra_ricochete`): escrever no mesmo sítio de onde os vizinhos estão a ler é uma corrida.

### §13.7 — ⭐ O RESULTADO, na peça do dono (`256×256`, `48` direcções, 1 borrão → 2)

| | terraços p99 | máx | desvio da convergida |
|---|---:|---:|---:|
| **antes** (especular, 1 passagem) | `0,7028` | `9,32` | `22,97 %` |
| + lei fosca | `0,4006` | `1,81` | `15,48 %` |
| **+ 2 passagens (o que shipa)** | **`0,2934`** | **`1,50`** | **`15,44 %`** |
| *o céu, para comparar* | `0,0308` | `0,15` | — |

⇒ **`2,4×` no `p99` e `6,2×` no MÁXIMO, com a exactidão a MELHORAR de `23 %` para `15 %`.**

### §13.8 — ⏳ O que FICA, com o mecanismo medido

O resíduo (`0,29` contra `0,031` do céu) está **decomposto e nomeado**: na peça do dono **uma
direcção faz `97 %`** do pior terraço, com contribuições `0,022 / 0,185 / 0,000` — o raio acerta em
dois pixels e **falha** no terceiro, e entre os dois que acertam o valor salta `8×`.

⇒ é um raio a atravessar a tigela em **incidência rasante**: o ponto que ele acerta desliza muito
para um passo pequeno do pixel sombreado, e o que ele lá encontra (`N·L`, a sombra da lâmpada, a
parede) muda depressa. ⛔ **Nem mais direcções nem mais borrão curam isto** — `256` direcções custam
`5,3×` e ainda leem pior que o céu a `16`.

⭐ **A cura publicada é a RADIÂNCIA PRÉ-FILTRADA:** o raio representa um cone, e o que ele devia
buscar é uma versão **mip-mapped** do campo de radiância (o que o *cone tracing* contra uma
representação volumétrica faz). É **wave com espec própria**, e o substrato natural aqui é o que
torna esta casa boa nisto: *o nosso modelador JÁ É um campo de distância*.

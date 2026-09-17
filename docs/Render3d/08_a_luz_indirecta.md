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

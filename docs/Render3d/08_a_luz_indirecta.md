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

1. o ricochete em **fatias**, irmão do `occlusion_slice` — é o que o põe no produto;
2. o segundo **ricochete** (hoje a luz que sai do ponto acertado é só a directa dele);
3. a parcela **especular** do indirecto (hoje é só o céu: uma irradiância por pixel não tem
   direcção, e o lóbulo especular pergunta por uma);
4. o **dispositivo** — esta passagem é CPU, e o quadro assente do modelador é da placa desde a `§36`.

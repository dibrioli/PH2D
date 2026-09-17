# 07 — O CHÃO QUE SÓ RECEBE (a `W4` fecha, 2026-09-16)

> **Ordem do dono (2026-09-16), perguntado com as quatro saídas na mesa:** *chão invisível* — ele não
> aparece; aparecem a **sombra** e o **escurecimento de contacto** que ele recebe. É o *shadow
> catcher* do KeyShot e do Marmoset: a peça fica pousada sem um piso a ocupar a vista.

A `W4` do [plano](03_o_plano.md) chama-se *«sombras que POUSAM o objecto»* e a régua dela é **«um
objecto a `0`, `1` e `10 cm` do chão tem de dar três sombras diferentes»**. A metade da peça (ela
tapa-se a si própria) fechou em 14/09 ([`05` §27](05_o_modo_render_do_modelador.md)); a outra metade
não tinha sujeito, porque **não havia chão** — e a decisão não era da linha.

## §1 — As quatro leis, e onde cada uma vive

| a pergunta | a lei | onde |
|---|---|---|
| **onde o chão está** | o ponto mais baixo da peça, achado olhando-a DE BAIXO com o próprio traçador | [`ph2d_field_render::lowest_point`](../../crates/ph2d-field-render/src/ground.rs) |
| **quando ele é pousado** | uma vez, quando o modo Render LIGA — e fica | [`ph2d_app_field3d::floor`](../../crates/ph2d-app-field3d/src/floor.rs) |
| **onde o raio o toca** | um plano horizontal, visto só de cima | `Ground::hit` |
| **quanto ele escurece** | `luz que chega COM a peça / luz que chegaria SEM ela`, sobre uma difusa branca | `shade_render::catcher` |

⚠️ **O chão não tem geometria no campo.** Ele não entra na marcha da câmera nem na de sombra: existe
só onde um raio de câmera **falha** a peça. *Um chão que tapasse coisas deixava de ser invisível* — e
é isso que deixa uma peça atravessá-lo sem artefacto nenhum.

⭐ **Longe da peça a razão é exactamente `1`**, porque as duas somas correm as mesmas contas na mesma
ordem e a de cima multiplica por `1,0`. ⇒ o pixel de fundo sai com **os bytes de sempre**, e não com
um arredondamento deles (gate `a_esfera_pousada_deita_sombra_e_longe_dela_o_fundo_e_intacto`).

## §2 — Onde o chão está: porque a CAIXA não serve

A caixa da `bounding_ball` é **conservadora**. Num cilindro inclinado ela soma os dois eixos
transversais separadamente e desce `0,02` abaixo da borda da peça — um chão pousado nela deixava-a a
flutuar, com a sombra descolada, que é o defeito que esta wave existe para curar.

⇒ **uma câmera paralela virada para cima**, que cobre a caixa: o acerto mais baixo da imagem é o ponto
mais baixo da peça. Três olhares de `64 × 64`, cada um com quatro pixels do anterior de largura.

⛔ **Dois não bastavam, e foi a quina que o disse:** o olhar amostra o CENTRO de cada pixel, e num cubo
de pé numa quina a altura sobe `√2` por unidade de distância ao vértice — o ponto achado ficava
`2,9e-4` **acima** dele. Com o terceiro olhar o erro cai para `< 1e-4`, e ele tem os dois sinais (a
marcha pára ligeiramente **abaixo** da superfície; o centro do pixel fica ligeiramente **acima** de
uma quina). Nenhum dos dois é visível: o chão não se desenha.

## §3 — Quanto escurece: a razão, e porque ela é de uma DIFUSA BRANCA

O factor é medido com a [`catcher_surface`] — uma difusa branca sem especular — e com a **mesma** lei
do material que pinta a peça: a irradiância do céu e o `N·L` de cada lâmpada saem das mesmas funções,
nas mesmas unidades. ⛔ Uma difusa escrita à mão ao lado da `ph2d-material` seria a segunda resposta à
pergunta *«quanta luz chega aqui?»*.

⚠️ **A razão é ESCALAR** (luminância Rec. 709): uma sombra tingida pela cor de cada luz mudaria de
matiz sobre um fundo colorido, e o que o chão entrega é só *quanto* escurece. E ela compõe-se como uma
camada preta de opacidade `1 − f`: num fundo opaco a cor escurece; no fundo **transparente** do
modelador ela sai como tinta preta com alfa, que é o que um *shadow catcher* entrega ao canvas.

## §4 — ⛔⛔ O CÉU do chão NÃO são os cones da peça — e a medição que o decidiu

A oclusão da peça são `48` cones fixos ([`cone_dir`](../../crates/ph2d-field-render/src/occlusion.rs)).
Postos a correr num recetor **plano**, eles desenham **anéis**: cada direcção entra e sai da peça de
repente conforme o ponto se afasta, e o chão mostra as `24` direcções de cima como `24` sombras fracas
sobrepostas. Medido numa linha do chão: **`17` extremos**, contra `1` da referência de `2 048` cones.
Na peça isto não se vê (a superfície é curva e o material mistura); num chão liso vê-se tudo.

⇒ a lei do chão é a **oclusão por campo de distância, amostrada na vertical**:

```text
céu(q) = clamp(1 − κ · Σₖ γ^(k−1) · clamp(1 − d(q + ŷ·hₖ) / (α·hₖ), 0, 1) / Σₖ γ^(k−1), 0, 1)
hₖ = alcance · k / N,  k = 1..N
```

Contínua por construção (o campo é contínuo, não há direcções), independente da câmera, `N`
avaliações por ponto.

### §4.1 — As constantes saíram da REFERÊNCIA, não do olho

Ajustadas contra a oclusão por **`2 048` cones** (os mesmos cones, convergidos e sem a cerca da bola),
numa esfera e num cubo pousados — `214 308` pixels de chão, `98 805` com a referência abaixo de `0,98`:

| `N` | `α` | `γ` | expoente | `κ` | erro quadrático | pior |
|---:|---:|---:|---:|---:|---:|---:|
| `4` | `1,0` | `1,0` | `1` | `1,07` | `0,082` | `0,226` |
| `8` | `2,0` | `1,0` | `1,5` | `0,79` | `0,043` | `0,307` |
| **`6`** | **`1,5`** | **`0,8`** | **`1`** | **`0,80`** | **`0,045`** | **`0,245`** |

⚠️ A linha escolhida não é a de menor erro: é a **mais simples** (linear) entre as melhores e a de
menor pior caso. E o gate corre-a numa **segunda cena que não entrou no ajuste** (uma cruz de três
eixos): erro `0,028`, p90 `0,043`, pior `0,114`.

## §5 — ⛔⛔ A cerca da bola cortava a penumbra numa ELIPSE DURA

O estimador de penumbra escurece um raio que passa a menos de `t/k` da peça. Um raio que parte do
**chão** viaja `t ~ 1` até lá, logo a penumbra estende-se `~0,12` **para fora** da bola que contém a
peça — e a cerca da sombra (a saída da bola) cortava-a: um raio que raspava a ponta de um cilindro,
fora da bola, não marchava e lia `1`; o vizinho que a tocava lia `0,4`. A imagem era um disco preto com
a borda dura à volta da sombra.

⇒ para um raio que parte do chão a cerca é a bola **alargada por `distância à luz / dureza`**, que num
campo de distância é exacta. Medido na cruz pousada, sobre vizinhos cujo chão está fora do contacto:

| cerca | pior salto | saltos `> 0,2` |
|---|---:|---:|
| a bola simples | `0,968` | `87` |
| **+ a folga** | **`0,093`** | **`0`** |

⚠️ **Os pixels de PEÇA ficam com a bola simples, e não por economia:** ali ela é o que impede o
estimador de ler a superfície de onde o raio saiu (a acne da esfera, `05` §27.5). *Um ponto do chão não
está sobre superfície nenhuma.*

## §6 — A silhueta de baixo não tem FIO CLARO

As sub-amostras de uma borda que falham a peça vêem o **chão**, não o fundo limpo. A propriedade é
exacta — um pixel de borda é `cobertura + (1 − cobertura)·sombra`, logo nunca é mais transparente do
que o chão escuro ao lado dele. Com o fundo limpo ele sairia com o alfa da cobertura só: um anel claro
à volta do pé, exactamente onde a sombra é mais escura. O factor de uma borda cujo centro **acerta** é
a média dos vizinhos de cruz que falham — aproximação declarada, da família das outras da borda (`05`
§39).

## §7 — Os dois motores

A CPU e o dispositivo escrevem a mesma lei, com as constantes **lidas** do ficheiro que as declara
(nunca transcritas), a mesma ordem de soma e os mesmos saltos (um raio que nem toca a bola alargada não
marcha em nenhum dos dois — um raio marchado com cerca `0` ainda avaliaria o campo uma vez).

Medido (`192×108`, a peça pousada, gate `o_chao_do_dispositivo_e_o_da_cpu`): o chão muda **`12 878`
bytes** da imagem, e os dois motores concordam em **`100,000 %` dos canais, com o pior a `0` níveis**.

## §7-bis — O preço na CPU, medido (e porque ele não é o do produto)

O passe da sombra com o chão, na cena da esfera e do cubo (mínimo de 3, `load 12,7`, as duas
configurações intercaladas):

| | sem chão | com chão | os pontos | o céu do chão |
|---|---:|---:|---:|---:|
| `640×360` | `1,8 ms` | `12,5` | `2,6` (`214 308` pixels) | `1,1` |
| `1920×1080` | `15,7 ms` | `102,0` | `20,4` (`1 928 714`) | `5,4` |

⭐ **O céu do chão é BARATO** (`5,4 ms` a `1080p`): a cerca por amostra corta as que não podem tocar a
peça. O que pesa são os **raios de sombra que partem do chão** — a mesma natureza do custo que a
sombra da peça já tem (`56 ms` a `1080p`, `05` §27.2), e o chão tem mais pixels do que a peça.

⚠️ **E este não é o preço do produto:** o caminho de omissão do modo Render é o **dispositivo**, onde
o quadro inteiro custa `5`–`20 ms`; a CPU é a referência e a rede. ⏳ Fica nomeado que os `20 ms` dos
pontos são uma intersecção raio–plano por pixel, que é uma função projectiva da coordenada do pixel e
podia ser incremental. O quadro de MOVIMENTO da CPU fica **byte-idêntico** (ele não tem sombras).

## §7-ter — O preço no DISPOSITIVO, que É o do produto

A §7-bis acima declara este buraco por escrito (*«este não é o preço do produto: o caminho de omissão
do modo Render é o dispositivo»*). Medido, na mesma cena, pelo **pintor do produto**
(`gpu_frame::paint`, o mesmo que o quadro assente chama):

| quadro assente | sem chão | com chão | o chão |
|---|---:|---:|---:|
| `640×360` | `3,49 ms` | `3,50` | **`+0,01`** — abaixo do ruído |
| `1920×1080` | `10,52 ms` | `10,95` | **`+0,43`** (`+4,1 %`) |

Mínimo de **5 corridas**, cada uma já o mínimo de 7 quadros, com as duas configurações **intercaladas**
(`sem · com · sem`) e a **ociosidade da CPU entre `66` e `92 %`**. Medianas ao lado: `3,58`/`3,57` e
`10,89`/`11,23`.

⭐⭐ **O chão no dispositivo é praticamente de graça, e a comparação é com a coluna da CPU acima:** a
`1080p` a CPU paga `+86,3 ms` pela **mesma resposta** (`15,7 → 102,0`) e a placa paga `+0,43` —
`200×` menos. *O caminho de referência só precisa de computar a mesma resposta; quem manda no tecto é
o dispositivo.*

⚠️ **A escala é o número de pixels de FUNDO, e é isso que explica o `640×360`:** há `9×` menos, logo o
previsto ali é `~0,05 ms` — debaixo do ruído dos controlos. A leitura honesta daquela célula é
**«não se mede»**, nunca «é zero».

⛔⛔ **E toda leitura de relógio de dispositivo feita com a máquina ocupada estava inflada `3,4×`:** o
mesmo `1080p` leu `41,2 ms` com `load 23` e `10,5 ms` com a CPU a `92 %` ociosa. ⚠️ **A grandeza que
decide é a OCIOSIDADE da CPU, não o `loadavg`** — que é uma média de um minuto e, a decair, diz `7,5`
com a máquina ainda a trabalhar (a média de cinco minutos lia `32` no mesmo instante). O `05` §43.9 já
o tinha medido no gate do divisor; aqui ele mordeu outra vez, num relógio diferente.

## §8 — Os gates, e as mutações

| gate | o que ele prende |
|---|---|
| `o_raio_toca_o_chao_so_vindo_de_cima` | a lei do raio, com os casos de baixo e o `NaN` |
| `o_ponto_mais_baixo_de_uma_esfera` · `..._de_um_cubo_de_pe_na_quina` · `..._de_duas_pecas_e_o_da_de_baixo` | onde o chão fica, à tolerância do olhar fino |
| `num_cilindro_inclinado_a_caixa_desce_abaixo_da_peca_e_a_busca_nao` | ⭐ o CONTROLO: porque a caixa não serve |
| `sem_chao_a_imagem_e_a_de_sempre` | `None` é o caminho de sempre, ao byte |
| `a_esfera_pousada_deita_sombra_e_longe_dela_o_fundo_e_intacto` | a sombra existe, e longe dela os bytes são os do fundo |
| `tres_alturas_tres_sombras` | ⭐⭐⭐ **a régua da `W4`**: a sombra afasta-se do pé e a penumbra alarga |
| `sem_lampadas_o_contacto_escurece_pela_oclusao` | o céu é uma fonte, e a peça tapa-o junto do pé |
| `a_oclusao_do_chao_segue_os_cones_convergidos` | a lei nova contra a referência, em duas cenas |
| `o_ceu_do_chao_nao_tem_aneis` | contra os `48` cones do produto, no mesmo chão |
| `a_sombra_do_chao_nao_tem_degrau_longe_da_peca` | a cerca alargada (§5) |
| `a_silhueta_de_baixo_nao_tem_fio_claro` | §6 |
| `um_olho_debaixo_do_chao_nao_ve_sombra` | o chão só existe visto de cima |
| `o_chao_fica_onde_foi_pousado_ate_o_render_voltar_a_ligar` · `fora_do_render_nao_ha_chao` · `ligar_o_render_esquece_a_ancora` | a âncora |
| `o_chao_do_dispositivo_e_o_da_cpu` (`#[ignore]`, placa) | os dois motores |

| mutação | resultado |
|---|---|
| `Ground::hit` aceita um raio de baixo | ⚪ **SOBREVIVEU — e a cerca SAIU**: ela era implicada pelo `t ≤ 0`, e *uma cerca que não muda nenhuma resposta é ruído no código* |
| o ponto mais baixo é a CAIXA | 🔴 (o cilindro inclinado e as duas peças) |
| a cerca do chão sem a folga | 🔴 `a_sombra_do_chao_nao_tem_degrau_longe_da_peca` |
| o `catcher` ignora a oclusão do céu | 🔴 — ⚠️ **e a 1.ª redacção do gate deixava-a passar**, ver §9 |
| a força do céu do chão a `0` | 🔴 |
| a borda usa o fundo limpo | 🔴 |
| o pintor não escurece o fundo | 🔴 (dois gates) |
| a âncora é relida a cada quadro | 🔴 `o_chao_fica_onde_foi_pousado…` |
| a marcha do dispositivo não escreve os canais do chão | 🔴 (o gate da placa, pela POPULAÇÃO: *«o chão só mudou 0 bytes»*) |

## §9 — ⛔⛔ A régua que media a silhueta e chamava-lhe chão

O `sem_lampadas_o_contacto_escurece_pela_oclusao` lia **o máximo do alfa entre os pixels de fundo** — e
nesse conjunto entravam os pixels de **silhueta**, que levam tinta das sub-amostras que acertam a peça.
Com a oclusão do chão **ignorada** ele ainda lia `128` (a cobertura da borda) contra os `224` da
verdade, e a mutação **sobrevivia**.

⇒ a régua passa a excluir as bordas e a medir **duas** coisas: quão escuro fica o contacto (`> 150`) e
**quanto chão** ele cobre (`> 1 000` pixels; com a mutação, `39` — todos de silhueta). *Um máximo sobre
uma população que inclui outra coisa mede a outra coisa.*

## §10 — ⏳ O que fica aberto

- **O chão não recebe luz INDIRECTA** nem a devolve à peça: ele escurece o fundo e não acende nada.
  A luz que ricocheteia é a `W5`, e o chão será o primeiro recetor dela;
- **Um só chão, horizontal.** Sem inclinação, sem parede, sem recorte — e a altura é de CENA (não há
  um chão por viewport);
- **O caminho da CPU sem refinamento** (o de omissão é o dispositivo) põe a sombra directa e o céu do
  chão na passagem `0`; o que ele não faz é a oclusão da PEÇA, que continua a vir das passagens;
- ⏳ **Sem lâmpada nenhuma o dispositivo recusa o quadro** (uma cerca anterior a esta wave: `mundos.is_empty()`
  cai na CPU), logo uma cena só com céu pinta o chão pelo caminho da CPU;
- ⏳ **Um botão de «pousar outra vez»** — hoje a re-ancoragem é ligar o Render de novo.

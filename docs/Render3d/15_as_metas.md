# 15 — AS METAS: bater todas as game engines em BELEZA e PERFORMANCE

> **Ordem do dono, 2026-09-20:** *«escolha o que for melhor para vc mas documente as metas: bater
> todas as game engines em beleza e performance»*.

⚠️ **«Bater» não é uma especificação, e «beleza» menos ainda.** Esta página parte as duas palavras
em colunas que se MEDEM, com o número de cada concorrente ao lado — medido nesta máquina, não
citado. ⛔ Uma meta sem número é uma opinião com data.

⚠️ **O produto que estas metas descrevem é o do [`14` §5](14_a_ordem_de_superar.md):** uma engine
**2D** — jogabilidade, física e colisor em duas dimensões — cujos **objectos** podem ser malhas 3D
reais, iluminadas e animadas em 3D, projectadas no canvas. *A simplicidade do 2D com o melhor da
aparência 3D.*

---

## §1 — Os cinco concorrentes, e como foram medidos

| motor | versão | como o medi |
|---|---|---|
| **Godot** | 4.7.2 (MIT) | **corrido** — `--doctool` sobre 810 classes |
| **Blender EEVEE Next** | 5.2.2 | **corrido** — `--background --python-expr`, API em tempo de execução |
| **Unreal** | 5.8.2 | instalado (73 GB) · ⛔ o fonte é restrito: só a API e a execução |
| **Unity HDRP** | 6000.7.0b1 | instalado · pacotes de pipeline lidos do editor |
| **Frostbite** (o ALVO) | BfN, 2019 | ⛔ não licenciado — só fontes públicas ([GDC 2018](https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/gdc2018-precomputedgiobalilluminationinfrostbite.pdf)) |

---

## §2 — ⭐ BELEZA: as colunas, com a barra de cada uma

| # | coluna | nós HOJE | o melhor deles | **META** |
|---|---|---|---|---|
| B1 | **campos de material** (norma OpenPBR = 41) | **21** | Blender **32** (Principled BSDF) | **41** — a norma inteira |
| B2 | **material por PIXEL** (texturas) | ⛔ **zero** — uma cor por objecto | todos têm | albedo · normal · rugosidade · metal · oclusão |
| B3 | **luz indirecta** | sondas, **só com a câmera parada** | Godot SDFGI · EEVEE ecrã+temporal | **ligada em movimento** |
| B4 | **subsuperfície** (a assinatura do alvo) | ✅ dois caminhos, paridade `100 %` | Blender ✅ · Godot ⛔ | manter, e levá-la ao SPRITE |
| B5 | **camada de estilo** com botões | ✅ 4 (contorno · curvatura · zona · saturação) | ⛔ **nenhum tem nativo** | manter a vantagem |
| B6 | ⭐⭐⭐ **luz re-derivada da FORMA, por quadro, num objecto 2D** | ⛔ | ⛔ **NENHUM tem** — todos usam normal map FIXO | **a rota B** |

### ⭐⭐⭐ B6 é a meta que nos torna únicos, e a razão é estrutural

Unity, Godot e Unreal fazem 2D iluminado com **mapas de normais fixos** — uma fotografia da forma.
Gira-se o sprite e a luz **não acompanha**. O [`02.2`](../3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md)
chama-lhe, por escrito, *«o efeito que nenhum sprite normal-mapeado comum consegue»*.

⇒ Eles não podem fazê-lo porque **não têm a forma em tempo de execução**. Nós temos — e é a mesma
razão do [`14` §4](14_a_ordem_de_superar.md): eles traçam contra aproximações (proxy, voxel, ecrã,
pré-cálculo) e nós contra a forma verdadeira.

---

## §3 — ⏱️ PERFORMANCE: as barras, e o recurso de cada uma

| # | coluna | nós HOJE | eles | **META** |
|---|---|---|---|---|
| P1 | quadro com a câmera a MEXER, **luz ligada**, `1920×1080` | ⛔ a luz está **desligada** | `16,7 ms` com tudo | **≤ 16,7 ms com tudo ligado** |
| P2 | quadro **parado**, `1920×1080` | `~61 ms` (medido, uma cena) | — | **≤ 16,7 ms** |
| P3 | objectos em **rota B** (3D ao vivo) num quadro | ⛔ não existe | — | ⏳ **a medir** — é pergunta aberta do `02.2` |
| P4 | objectos em **rota A** (assado) | = um sprite | = um sprite | manter: **roda em telemóvel** |

⚠️ **`16,7 ms` é `60` imagens por segundo, e é a barra porque é a deles.** Não é um número escolhido:
é o orçamento de um quadro a 60 Hz, que é o que os cinco entregam.

⛔ **Toda leitura desta tabela vale apenas com `loadavg < 10`, em `--release`, com a ociosidade real
impressa ao lado.** A `line/3DModeling` já pagou duas vezes por medir sob carga
([handoff §10.3](../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-20.md)).

---

## §4 — ⛔ O que NÃO prometemos, e dizê-lo poupa uma jornada

Uma meta honesta declara o que fica de fora. Estas quatro ficam, **por medição ou por desenho**:

| fora | porquê |
|---|---|
| **escala de conteúdo** (Nanite, virtual textures, streaming) | anos-pessoa de engenharia de dados; e o alvo não os usa — [`02` §4](02_o_estado_da_arte.md) |
| **GI dinâmica a 60 Hz** | ⭐ **o próprio ALVO não a tem**: o Frostbite pré-calcula com Enlighten. Persegui-la é gastar onde o alvo não gastou |
| **jogabilidade 3D** | é o DESENHO da engine, não uma falta ([`14` §5](14_a_ordem_de_superar.md)) |
| **hardware de raios** | se a resposta certa for ReSTIR, eles chegaram primeiro e melhor |

---

## §5 — A ordem de ataque, e porque ESTA

⭐ **Escolhida por «salto visual por unidade de trabalho», sobre infraestrutura que já existe.**

| ordem | obra | porquê primeiro | fecha |
|---|---|---|---|
| **1.ª** | **a lei que acende o sprite passa a ser o PIPELINE BOM** | ⭐ a rota A **já está construída e no ECS**; o que faz o objecto parecer de 2010 é a LEI, não a infraestrutura | **B1 · B4 · B5** no sprite |
| 2.ª | **a rota B** — o objecto 3D ao vivo (o catavento) | produz o mesmo G-buffer, agora por quadro | **B6** · `P3` |
| 3.ª | **a luz sobrevive ao movimento** (sondas persistentes) | toda medida e à espera; é `P1` | **B3 · P1 · P2** |
| 4.ª | **texturas** | o maior buraco contra os cinco numa comparação lado a lado | **B2** |
| 5.ª | **animação 3D** do objecto | é o que faz a rota B valer a pena | fecha o catavento |

### Porque a 1.ª é a 1.ª, com o número

O sprite com forma **já é aceso hoje** — por [`impasto_light.wgsl`](../../crates/ph2d-render/src/shaders/impasto_light.wgsl),
que é um **modelo de TINTA** do Painter: difuso envolvido + especular por tabela, **sem GGX e sem
conservação de energia**. Ao lado, o modelador tem o OpenPBR inteiro, a subsuperfície e o estilo.

E o G-buffer do sprite **já traz o que o PBR precisa**
([`baked_form`](../../crates/ph2d-form-donation/src/baked_form.rs)): `base` (albedo) · `form`
(normais) · `form_occ` (oclusão). O material é **por objecto**, que é exactamente como o modelador
já o trata.

⇒ *é ligar leis que existem a dados que existem* — e é o passo que **prova a tese do
[`14` §1](14_a_ordem_de_superar.md)** (as leis são portáveis), do qual todo o resto depende.

---

## §6 — ⛔ Kill-criteria, declarados ANTES do build (DIRETIVA §5)

| obra | o que a mata |
|---|---|
| **1.ª (PBR no sprite)** | se a saída não for **byte-idêntica** no ponto neutro, ou se o custo por sprite subir acima de **`2×`** o passe de tinta de hoje |
| **2.ª (rota B)** | se **menos de `8`** objectos ao vivo couberem num quadro de `16,7 ms` a `1080p` — abaixo disso é um efeito, não uma feature |
| **3.ª (sondas)** | se, com as sondas persistentes **e** a oclusão a meia resolução, o quadro de movimento com luz ligada não descer abaixo de `16,7 ms` nas cenas hoje nítidas |

⚠️ **E a régua de cada meta vive num GATE**, não nesta página: *uma barra escrita em prosa não
reprova ninguém.*

---

## §7 — O ESTADO da 1.ª obra, e a medição que corrigiu o preço dela

> Esta secção é escrita **depois** de a obra começar, e existe porque a §5 estimou o preço e a
> medição o desmentiu. *Uma ordem de ataque que não se corrige com o que se mediu é um plano sobre
> outro projecto.*

### ✅ Feito e provado

A lei vive na folha [`ph2d-form-pbr`](../../crates/ph2d-form-pbr/) — o **laço por texel** que acende
a forma doada com o OpenPBR, com **UMA** dependência de produção (`ph2d-material`, o port do
OpenPBR) e **zero** linhas de óptica próprias, nem no Rust nem no gémeo em WGSL. `13` gates, cada um
com o controlo ao lado; prova de mutação **12 de 12 a sangrar**.

⭐ **O que ela achou, e nenhuma das três era procurada:**

| achado | mecanismo |
|---|---|
| **a lâmpada anti-paralela à vista devolvia `NaN`** | `v + to_light` é o vector nulo. No modelador tem **medida nula** (a vista varia por pixel); num canvas 2D a `VISTA` é constante ⇒ é uma configuração que o artista **escreve**, e pinta a peça inteira |
| **`ptr<storage, …>` não é WGSL do núcleo** | a `naga` recusa-o (`InvalidArgumentPointerSpace`) ⇒ as lâmpadas entram por VALOR, e o tecto delas entra por **MARCA** (`{MAX_LAMPADAS}`), porque quem o sabe é o rig e esta crate não depende dele |
| **o meu doc de montagem era falso** | ele dizia que bastava concatenar as duas fontes; a da lei traz um `{ENV}` por preencher ⇒ a concatenação crua **não parsa** |

### ⛔ E a §5 subestimou o preço: o passe de dispositivo é OBRIGATÓRIO

A §5 escreveu *«é ligar leis que existem a dados que existem»*. Medido (`load 3,3`, `--release`,
32 núcleos, a sonda `diag_quanto_custa_acender_um_sprite`), acender **na CPU em paralelo** um sprite
de `1024²` custa `11,1 ms` com **uma** lâmpada e `34,1 ms` com **quatro** — e o orçamento de um
quadro é `16,7 ms`.

⇒ **a CPU paralela atravessa o orçamento à SEGUNDA lâmpada**, e a `2048²` estoura com uma só. O
passe de dispositivo não é aceleração: é a condição de a re-acendida continuar a ser o **gesto
contínuo** que o `relight_stale` promete por escrito.

⚠️⚠️ **E foi a coluna PARALELA que tornou esse veredito honesto.** Com o número de um núcleo
(`111 ms`, `6,6×` um quadro) a conclusão seria a mesma **pela razão errada** — o §0.0 ao contrário,
o caminho lento a definir o produto. A margem real não é `6,6×`, são **duas lâmpadas**, e é um
número que outra pessoa pode mudar (mais núcleos, ou cozer por tiles): *quem o mover reconfere esta
nota.*

### ✅ E O PASSE EXISTE — a obra 1 fechou, e a medição mudou a leitura do problema

[`ph2d_form_donation::baked_form::passe_da_forma`](../../crates/ph2d-form-donation/src/baked_form/passe_da_forma.rs)
— um compute que **compõe** três fontes que já existiam e acrescenta só o ponto de entrada (ler três
texels, chamar a lei, escrever um pixel). ⛔ **Zero linhas de óptica**, e a `naga` prova-o: ela
resolve o `mx_direct` contra a fonte composta, logo o que o laço chama é o do `ph2d-material`.

**Medido na placa** (`--release`, mínimo de 9, com `poll` — sem ele o relógio mede a fila de
submissão e não o trabalho):

| lado | lâmpadas | CPU paralela | **placa** | ganho |
|---|---|---|---|---|
| `1024²` | 1 | `11,1 ms` | **`1,74 ms`** | `6,4×` |
| `1024²` | 4 | `34,1 ms` | **`2,08 ms`** | `16,4×` |
| `2048²` | 1 | *estourava* | `6,64 ms` | — |
| `2048²` | 4 | — | `7,95 ms` | — |

⭐⭐⭐ **E o achado é a FORMA da coluna, não o ganho:** na CPU a 4.ª lâmpada custa `3,1×` a primeira
(`11,1 → 34,1`); na placa custa **`1,2×`** (`1,74 → 2,08`). *O gargalo mudou de sítio* — ele já não é
a lei, é o **transporte**: os três canais sobem a cada acendida e a forma é `Rgba32Float`, ou seja
`16 bytes` por texel. ⇒ o orçamento de quadro deixou de ser a pergunta, e a que fica está nomeada
em baixo.

### ✅ E a obra 3 — a PARIDADE do laço — fechou, com o número

`a_placa_e_a_regua_concordam_no_pixel` (`#[ignore]`, precisa de adapter): a régua é a
`pixels_pela_forma_na_cpu` e a placa é o `acende_com(Lei::Forma, …)`, **as duas pela porta do
produto**. Medido a `256²`: **`100,000 %` dos `262 144` bytes idênticos**, pior desvio `0`.

⛔⛔ **A barra NÃO é `0`, e dizê-lo é honestidade:** um backend pode contrair `a*b + c` num `fma`
(que é *mais* exacto, logo diferente), e os três mecanismos estão medidos e escritos no cabeçalho da
`ph2d-style`. ⇒ o tecto portátil é **um byte**, o degrau da quantização.

⚠️⚠️ **E um tecto de `1` sozinho deixou uma mutação SOBREVIVER:** apagar o `+ 0.5` do shader
(arredondar → truncar) desloca metade da tela por um byte e passa. *Uma barra larga não é só uma
afirmação fraca — é o sítio onde uma régua errada sobrevive.* ⇒ a segunda barra é a **POPULAÇÃO**, e
ela separa as duas causas pelo mecanismo:

| causa | pior | quantos bytes |
|---|---|---|
| contracção `fma` no backend | `1` | os que caem a ~1 ULP de uma fronteira |
| uma LEI diferente (a truncagem) | `1` | **todos** os que não são exactos |

**Prova de mutação: 4 de 4** — a exposição que não chega (`pior 223`), a cobertura cravada a `1`
(`46`), a truncagem (a população), e a oclusão, que é a quarta e está **NOMEADA** logo abaixo.

### ✅ FECHADO em 2026-09-20: a OCLUSÃO chega ao pixel, e o que faltava era o CÉU

A redacção anterior desta secção dizia *«ABERTO, e é do dono»* e a cura *«não é inventar aqui um
termo que nenhuma referência declara — é decisão de produto»*. ⛔ **Estava errada, e a razão escrita
ao lado dela também:** ela afirmava que *«o rig é `KEY + 3 × FILL` e as lâmpadas de preenchimento
SÃO o ambiente dele»* — e as três de preenchimento nascem **`on: false`**.

**Medido** com a configuração de fábrica (uma lâmpada acesa de quatro), sobre uma bola com fresta:

| | antes | agora |
|---|---|---|
| texels **PRETOS ao bit** | `8 243` de `32 928` — **`25,03 %`** | **`0`** (`0,000 %`) |
| luminância média na SOMBRA | `0,000` | `48,54` |
| na FRESTA (`occ < 0,5`) | `165,49` | **`122,03`** |

⇒ *a cavidade × os dois AOs que o objecto assado guarda desde que existe passam a ser lidos, e isso
não custou uma linha de lei nova: custou o céu.*

**E a lei da casa já nomeava este defeito antes de ele acontecer**, no doc do `ph2d_light::AMBIENT`:
*«os dois consumidores (tinta e forma) têm de dobrar a razão do MESMO jeito, senão a mesma lâmpada
deixaria a escultura mais escura na sombra que a pintura ao lado dela, e ninguém saberia dizer por
quê»*.

#### ⛔⛔ A 1.ª tradução foi um ERRO DE CATEGORIA — `AMBIENT` não é uma radiância

Pôr o `env_ambient` como irradiância absoluta **satura a peça**: com a exposição calibrada sem céu, a
média do miolo cinzento salta de `186` para `255`. O `AMBIENT` declara-se, à letra, como *«o que uma
face totalmente virada PARA LONGE da luz ainda devolve»* — uma **fracção da resposta plana**.

⭐ **A tradução certa é DERIVADA e não tem constante escolhida:** a resposta plana do rig é
`Σ max(l·z, 0) · tint`, e o piso do modelo RELATIVO entra como termo ADITIVO por `f = A/(1 − A)` — a
forma fechada que faz `sombra/plano` voltar a valer exactamente `A`. Gate
`a_sombra_vale_ambient_do_plano`, medido no **horizonte** da rampa (ela redistribui à volta dele: uma
face virada para baixo lê `0,255`).

⭐⭐ **E isso responde à objecção que a redacção antiga levantava** (*«o artista veria a peça a não
escurecer por mais que apagasse lâmpadas»*): com o céu derivado do rig, **apagar as lâmpadas apaga o
céu**. O estúdio é o rig.

⚠️ **A exposição teve de ser re-tirada:** `OLHAR_DA_FORMA` passa de `3,00` para **`2,10`** stops
(escada no doc dele). *Quem move o número que tornava outro correcto tem de reconferir a nota.*

⛔ **E a fixtura sintética desta crate acendia POR BAIXO** — ela escrevia o `y` da normal em espaço de
VISTA e o canal é escrito em CANVAS pelo `canvas_normal`. A paridade nunca o podia ver (os dois
motores leem os MESMOS planos), e só passou a importar quando o ambiente ganhou direcção.

### ⏳ O que fica, com o preço medido

| # | obra | preço |
|---|---|---|
| 1 | **não re-enviar a forma quando só o rig mudou** | é onde o tempo está: o canal **não depende do rig** (é o que torna arrastar a lâmpada barato) e sobe na mesma a cada quadro. A diferença entre `1,74` e `2,08 ms` diz que a lei custa `~0,3 ms`; o resto é transporte |
| 2 | a **escolha por objecto** (`PROJECT_SCHEMA`) | gateada no veredito do dono sobre o §8 |
| 3 | ~~a **oclusão**~~ | ✅ **FECHADA** acima — a cura era o céu, não uma decisão de produto |

⚠️ **E o endereço do passe não é o que esta secção escreveu.** Ela mandava-o para a `ph2d-render`,
*«~600 linhas, a medida do passe irmão»* — medido, as duas metades estão erradas pela mesma razão: o
irmão carrega região, janela de planos, `planes_seeded` e a LUT especular, e esta acendida é **a tela
inteira, uma vez, sem tabela**. Ele vive na `ph2d-form-donation` com **zero dependências novas** (a
crate já declarava a lei, o gémeo, a vista, o rig e o `wgpu`), ao lado do seu único consumidor. Ver o
cabeçalho do módulo para os três argumentos.

---

## §8 — A 1.ª obra tem IMAGEM: as duas leis, lado a lado

![as duas leis](imagens/as_duas_leis_2026-09-20.png)

> Quatro bolas, a **mesma** forma e o **mesmo** nível de brilho (a exposição da lei nova foi medida
> para igualar o da de sempre — ver o `OLHAR_DA_FORMA`), acesas pelas duas leis num adaptador real.
> Gerado pela sonda `as_duas_leis_sobre_a_mesma_forma`.

**O que muda, e é a razão da obra:** à esquerda o destaque tem a **cor da bola** e a peça lê-se
chapada; à direita ele é **branco** e há um terminador de material. *Um plástico vermelho tem
destaque branco; só um metal o tinge* — e é isso que separa um modelo de tinta de um pipeline
fisicamente correcto.

E a diferença é MEDIDA, não uma impressão — a razão `R/B` no destaque (o topo `3 %` mais brilhante
de cada bola):

| bola | a lei de sempre | a lei nova |
|---|---|---|
| vermelha | `2,686` | **`1,485`** |
| azul | `0,372` | **`0,673`** |
| **cinzenta (controlo)** | `1,000` | `1,000` |

⭐ O `1,485` é, ao terceiro decimal, o número que a sonda de CPU previu (`1,50`) **antes de haver
placa** — e a bola cinzenta lê `1,000` nas duas, que é o controlo que dá direito às outras linhas.

⚠️ **O que a imagem NÃO decide** é se a sombra mais funda da direita é o que se quer: a lei de
sempre levanta os pretos e esta não. Isso é produto, e é do dono.

### Como o ver no app

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && \
  env PH2D_FORM_PBR=1 cargo run -p ph2d-host-desktop --profile smoke
```

Depois: pôr uma imagem no canvas, escolhê-la, esculpir uma peça, e carregar em **`Light the
Selected Sprite`**. ⚠️ **Sem a variável, tudo fica exactamente como está hoje** — a lei nova shipa
desligada, e um projecto gravado abre com a aparência com que foi gravado.

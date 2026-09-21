# 17 — A ROTA B: o catavento

> **A 2.ª obra da fila** ([`15` §5](15_as_metas.md)): *um objecto 3D ao vivo dentro do canvas 2D —
> ele roda, e a luz acompanha.* A arquitectura foi desenhada em
> [`docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`](../3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md)
> e o que faltava era construí-la. Esta página começa onde a lei desta casa manda
> (`CLAUDE.md` §5.0): **medir o que a composição já dá, antes da 1.ª linha de produto.**

---

## §1 — A medição de §5.0, e o que ela decidiu

**Instrumentos — são DOIS, e a razão está na §1.2-bis:** a corrente do produto é *rasterizar E
acender*, e as duas metades vivem em crates que não se conhecem.

| bloco | o que mede | onde |
|---|---|---|
| A · B · C | rasterizar · o readback · a silhueta | [`ph2d-mesh-render/tests/it/mede_o_que_a_composicao_ja_da_ao_catavento.rs`](../../crates/ph2d-mesh-render/tests/it/mede_o_que_a_composicao_ja_da_ao_catavento.rs) |
| **D** | **acender** | [`ph2d-form-donation/src/mede_o_acender_por_quadro.rs`](../../crates/ph2d-form-donation/src/mede_o_acender_por_quadro.rs) |

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && \
PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-mesh-render --release \
  --test it -- --ignored --nocapture --test-threads=1 catavento

cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && \
PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-form-donation --release \
  mede_o_acender -- --ignored --nocapture --test-threads=1
```

⚠️ **`--test-threads=1` é load-bearing** e não estilo — ver §1.4.

### §1.1 — Bloco A: rasterizar a malha filha para o G-buffer, **por quadro**

Textura **reaproveitada** (o que um dirty-flag entrega) contra **alocada a cada quadro** (o que a
porta de assar faz hoje). Medido a `load 44` — ou seja, **um PISO**: a contenção só pode ter tornado
estes números maiores.

| malha | lado `128` | `256` | `512` | `1024` |
|---|---|---|---|---|
| leve (`3 010` v) | `0,085` | `0,076` | `0,074` | `0,078` ms |
| **fábrica (`98 306` v)** | `0,131` | `0,116` | **`0,133`** | `0,124` ms |

⛔ **A coluna «objectos por quadro» que aqui esteve dividia o orçamento por ESTE número e lia
`126` — ver §1.2-bis: é metade da corrente, e a conta honesta está na §4.**

⭐⭐⭐ **A leitura que decide a obra: o custo é dos VÉRTICES e NÃO dos pixels.**
`8×` de lado (`64×` de área) não move o relógio; `33×` de vértices move-o `1,6×`. E a coluna
*alocada* empata com a *reaproveitada* (`0,115`–`0,137`) ⇒ **o slot de textura não é o preço**.

### §1.2 — Bloco B: o CONTROLO — a porta que existe HOJE, com o readback dentro

[`MeshRenderer::form_plane`] rasteriza **e traz os dois planos de volta para a CPU** (`Vec<f32>`),
porque quem a chama é o assado (rota A, uma vez, num botão).

| malha | `128` | `256` | `512` | `1024` |
|---|---|---|---|---|
| leve | `0,383` | `0,987` | `3,700` | `9,721` ms |
| fábrica | `0,422` | `1,038` | **`4,180`** | `11,568` ms |

⛔⛔ **Ela escala com a ÁREA (`~4×` por duplicação do lado) e é indiferente à malha** — o oposto
exacto do Bloco A. ⇒ **o readback é o preço inteiro**, e a `512²` na peça de fábrica ele vale
**`31×`** a rasterização (`4,180 / 0,133`).

⭐⭐⭐ **É este par de tabelas que diz que a obra EXISTE.** A pergunta era *«a rota B é a rota A
chamada mais vezes?»*, e a resposta é **não**: chamar a porta de hoje por quadro dá **`4` objectos**
a `512²`; manter o G-buffer na placa dá **`26`** (§4 — e `126` se se contasse só este elo).

### §1.2-bis — ⛔⛔⛔ Bloco D: **ACENDER**, que é a outra metade da corrente — e ela corrige o Bloco A por `5×`

**Os blocos A–C mediam RASTERIZAR, e a corrente do produto é *rasterizar E acender*.** *Uma régua
que mede o primeiro elo e divide o orçamento por ele devolve uma contagem de objectos que o app
nunca vai ver* — e foi exactamente o que o `126` da §1.1 era.

Medido pela porta do produto ([`baked_form::acende_com`], `load 14`):

| lado | lei `Tinta` (a de até 21/09) | lei **`Forma`** (a que ship) |
|---|---|---|
| `128` | `0,171` | **`0,150`** ms |
| `256` | `0,330` | **`0,254`** ms |
| `512` | `0,823` | **`0,505`** ms |
| `1024` | `4,029` | **`1,829`** ms |

⭐⭐⭐ **As duas metades escalam em grandezas DIFERENTES, e é isso que decide o orçamento:**

| | rasterizar (§1.1) | acender (aqui) |
|---|---|---|
| escala com | **os VÉRTICES** (plano no lado) | **a ÁREA** (`~4×` por duplicação) |
| a `512²`, peça de fábrica | `0,133 ms` | `0,505 ms` |

⇒ **a corrente a `512²` custa `0,638 ms` ⇒ `26` objectos por quadro**, e não os `126` que a §1.1
sozinha prometia. **O elo que manda é o ACENDER, e o tamanho que o governa é o do SPRITE.**

⭐⭐ **E um facto que eu não esperava: a lei que o dono aprovou é a mais BARATA das duas** — a
`Forma` (OpenPBR) bate a `Tinta` em todos os lados, e a `1024²` por **`2,2×`**.

### §1.3 — Bloco C: a resolução do G-buffer — o rectângulo do sprite, ou uma fracção?

⚠️ **A régua NÃO é a cobertura parcial** — ver §1.4. Ela é o **desacordo de silhueta**: rasteriza-se
no rectângulo cheio, rasteriza-se na fracção, amplia-se por vizinho e contam-se os pixels do **ecrã**
que ficam do lado errado. Sprite de `512²`, silhueta de `52 593` px.

| fracção | lado | px errados | % da silhueta | px / √área (≈ contorno) |
|---|---|---|---|---|
| `1/1` | `512` | `0` | `0,00 %` | `0,00` |
| `1/2` | `256` | `375` | `0,71 %` | `1,64` |
| `1/4` | `128` | `787` | `1,50 %` | `3,43` |
| `1/8` | `64` | `1 495` | `2,84 %` | `6,52` |

Ele **dobra a cada metade**, que é o que a geometria prevê (o erro é um texel da grelha grossa ao
longo do contorno).

⭐⭐ **E a pergunta do `02.2` DISSOLVE-SE, mas não pelo motivo que ela supunha** — e a razão final é
a do Bloco D, não a do Bloco A:

- **Uma fracção não compra rasterização**, porque ela já é plana no lado (`0,133` a `512²` contra
  `0,131` a `128²`).
- ⛔ **E não compra acendida NENHUMA**, que é o elo caro: o passe despacha sobre os pixels do
  **SPRITE** — ele escreve o slot visível — logo o custo dele é do rectângulo que o artista escolheu,
  e um G-buffer mais pequeno por baixo não muda uma linha disso.

⇒ *pagar silhueta para poupar um tempo que não existe é uma troca sem lado bom.*

⛔ **Mas ela reabre noutro recurso, e esse é exacto e não precisa de medição — a MEMÓRIA.** O
G-buffer é `RGBA16F` (8 B/texel) + `R16F` (2 B/texel):

| lado | por objecto | a `26` objectos |
|---|---|---|
| `1024` | `10,0 MiB` | `260 MiB` |
| `512` | `2,5 MiB` | `65 MiB` |
| `256` | `0,625 MiB` | `16 MiB` |

⇒ **a fracção é uma decisão de VRAM, não de relógio**, e o lado certo é o do rectângulo que o sprite
de facto ocupa no ecrã — nunca um número escolhido.

### §1.4 — ⛔⛔⛔ Três defeitos da 1.ª redacção desta sonda

*Registados porque cada um tem nome nesta casa, e os três produziram uma tabela plausível e errada.*

1. **Os três blocos correram em PARALELO sobre a mesma placa.** O Bloco A leu `0,243 ms` a `128²` e
   `0,053` a `1024²` — *o lado grande mais barato que o pequeno*, que é a assinatura de estar a medir
   CONTENÇÃO e não trabalho. ⇒ `--test-threads=1`.
2. **A malha tinha `3 010` vértices** e a peça com que o módulo abre tem **`98 306`**
   ([`scenes::mesh::peca_de_fabrica`]). *Uma fixtura `33×` mais leve que o produto mede o overhead de
   submissão e chama-lhe o preço do objecto* — e como o custo desta lei é **de vértices**, era
   exactamente a grandeza sob teste que a fixtura não continha.
3. **A régua da resolução não podia ver o fenómeno.** Ela contava texels de cobertura PARCIAL, e o
   pipeline desta crate é `sample_count: 1` ⇒ **sem MSAA a cobertura é BINÁRIA**, e ela leu `0` nas
   quatro fracções, sobre um produto correcto. *Uma régua que lê zero sobre uma fixtura que não pode
   conter o fenómeno lê-se exactamente como «não há defeito».*

---

### §1.5 — ⭐⭐⭐⭐ Bloco E: **DE QUE ROTAÇÃO é a rota B?** (a pergunta que decide o componente)

O `02.2` promete *«rodar um sprite e ver a luz acompanhar — o efeito que nenhum sprite
normal-mapeado comum consegue»*. ⚠️ **Essa frase é verdadeira para metade das rotações e FALSA para
a outra metade**, e nenhuma medição a tinha partido.

A régua é o **ângulo entre normais** onde os dois planos cobrem, mais o desacordo de silhueta. O
sucedâneo 2D é aplicado a **`90°` de propósito**: ali ele é uma **permutação EXACTA de pixels** mais
a troca `(x, y, z) → (y, −x, z)` das normais — *sem reamostragem, logo o que sobra é o fenómeno e
não o filtro*.

| malha | rotação de `90°` | mediana | p99 | silhueta |
|---|---|---|---|---|
| com relevo | **no plano, com o sucedâneo 2D** | **`0,00°`** | `0,03°` | `0,00 %` |
| com relevo | no plano, com o plano FIXO | `28,58°` | `130,93°` | `6,16 %` |
| com relevo | **fora do plano, com o plano FIXO** | **`31,69°`** | `125,14°` | `6,71 %` |
| esfera lisa (**CONTROLO**) | as quatro linhas | `0,00°` | `0,03°` | `0,00 %` |

⭐⭐⭐ **A rota A JÁ DÁ a rotação NO PLANO, e dá-a EXACTAMENTE.** Uma rotação `R` em torno do eixo da
vista leva as normais a `R·n` e a imagem a `R·imagem` — **duas operações 2D sobre o plano já
assado**, e a medição lê `0,00°` sobre `0,00 %` de silhueta discordante. ⛔ **Fora do plano não há
sucedâneo nenhum:** aparecem faces que não estavam na imagem, e nenhuma operação 2D as inventa.

⭐ **E o CONTROLO é o que dá direito à leitura:** uma esfera lisa **não tem orientação** — o campo de
normais dela visto de uma câmera não depende de como ela está rodada —, logo ela lê `0,00°` nas
quatro linhas. *Uma fixtura sem o fenómeno tem de ler zero, senão a régua está a medir o
instrumento.*

⛔⛔⛔ **E isto decide o COMPONENTE, não só a cena.** O
[`ph2d_ecs::Transform`](../../crates/ph2d-ecs/src/transform.rs) tem `rotation: f32` e exprime
**apenas** a rotação no plano — que é exactamente a que a rota A já dá. ⇒ **a pose 3D tem de viajar
no componente da malha**, porque a hierarquia 2D não sabe exprimir a rotação que justifica a obra.
E a cena de smoke tem de mostrar uma pá a **VIRAR**, nunca um sprite a girar no plano: *uma cena que
mostrasse a rotação no plano estaria a demonstrar uma coisa que a rota A já faz.*

---

## §2 — O que a composição dá, o que ela NÃO dá, e onde está a costura

⭐⭐⭐ **As duas metades já vivem na placa, e a costura entre elas passa pela CPU.**

| peça | existe? | onde |
|---|---|---|
| rasterizar a malha para um G-buffer fora de ecrã | ✅ | [`MeshRenderer::render_gbuffer`](../../crates/ph2d-mesh-render/src/pipeline_gbuffer.rs) |
| acender um sprite a partir de um G-buffer | ✅ | [`baked_form::passe_da_forma`](../../crates/ph2d-form-donation/src/baked_form/passe_da_forma.rs) |
| **o G-buffer ficar na placa entre os dois** | ⛔ | — |
| `Mesh3D` / `MeshShading` no ECS | ⛔ | zero ocorrências em `ph2d-ecs` |

Hoje o percurso é:

```
malha → [placa rasteriza] → readback → BakedForm.form: Vec<f32> → write_texture → [placa acende]
```

⚠️ **Duas travessias GPU↔CPU por acendida.** Para a rota A isso é irrelevante — ela corre **uma vez,
num botão**, e o `Vec<f32>` é o que **viaja no documento** e sobrevive ao ficheiro (é o `form` do
[`BakedForm`], ao lado do `rig` e da `lei`). Para a rota B é o custo inteiro, e é o que o Bloco B
mede.

⇒ **a obra não é «construir um renderizador»: é abrir uma costura RESIDENTE** entre duas leis que já
existem, e pendurá-la em dois componentes do ECS.

---

## §3 — O que fica por decidir com medição, e não com opinião

- **O dirty-flag**: a §1.1 mede o custo de re-rasterizar sempre. Quanto ele poupa numa cena real
  depende de quantos objectos rodam por quadro, que é facto da cena e não do motor.
- **O tecto de VRAM da cena** não foi medido — a §1.3 tem a aritmética por objecto e falta a soma.
- **A corrente foi medida com os dois elos SEPARADOS**, cada um com o seu aquecimento. Uma medição
  da corrente INTEIRA, com o G-buffer residente, só existe depois de a costura existir.

---

## §4 — O ORÇAMENTO, na forma em que se usa

`custo por objecto por quadro = rasterizar (plano, ~0,13 ms na peça de fábrica) + acender (∝ área)`

| lado do sprite | rasterizar | acender | total | **objs/quadro** |
|---|---|---|---|---|
| `128` | `0,131` | `0,150` | `0,281` ms | **`59`** |
| `256` | `0,116` | `0,254` | `0,370` ms | **`45`** |
| `512` | `0,133` | `0,505` | `0,638` ms | **`26`** |
| `1024` | `0,124` | `1,829` | `1,953` ms | **`8`** |

⚠️ **Tudo isto medido a `load 14`–`44`** (`CLAUDE.md` §5.0: um relógio desta workstation acima de
`load ~5` não vale nada) ⇒ **são PISOS**, e a máquina calma só pode dar mais.

⭐ **A leitura de produto:** o orçamento da rota B não é do motor, é do **tamanho que o artista dá ao
sprite**. Um catavento de `256²` cabe `45` vezes num quadro; o mesmo catavento a `1024²` cabe `8`.

---

## §5 — ⭐⭐⭐ W1: **a costura residente** (construída em 2026-09-21)

A §2 diz que as duas metades já vivem na placa e que a costura entre elas passa pela CPU. A W1 abre
essa costura, e ela custou **uma porta e nenhuma linha de shader**.

### §5.1 — A porta

[`PasseDaForma::acende_residente`](../../crates/ph2d-form-donation/src/baked_form/passe_da_forma.rs)
— irmã da `acende`, e a diferença é **de onde vem a forma**:

| | `acende` (rota A, o assado) | **`acende_residente`** (rota B, o catavento) |
|---|---|---|
| a forma | fatias da CPU, **carregadas** a cada acendida | **vistas de textura** que já vivem na placa |
| o `base` | carregado | **carregado** — ele é a arte, não um subproduto da malha |
| corre | uma vez, num botão | **por quadro** |

⭐ **As duas entram no MESMO `despacha`**, e a única diferença é um `unwrap_or` sobre os dois
bindings. *Com dois corpos paralelos, a próxima linha que mexesse no bind group teria de ser escrita
nas duas, e a que alguém esquecesse divergia em silêncio.*

⚠️ **E as duas recebem a luz num TIPO** (`LuzDaCena`: material · lâmpadas · céu · olhar), que é
exactamente a lista de argumentos do `Globais::novo`. *Um grupo que já é a lista de argumentos de
uma função é um tipo que faltava* — quem o achou foi o clippy a acusar `9/7` argumentos na porta
nova, e ⛔ a saída barata (um `#[allow]`) deixava a 2.ª porta a escrever a mesma quádrupla por
extenso, que é a segunda ortografia da mesma lei.

### §5.2 — ⭐⭐ E ela não custou uma linha de shader

O layout já declarava `Float { filterable: false }` e o shader só faz `textureLoad` ⇒ **as texturas
`Rgba16Float`/`R16Float` que a rasterização produz ligam-se ali sem nada mudar**, embora a `acende`
crie as dela em `Rgba32Float`/`R32Float`.

⚠️ **Isso é um facto sobre o `wgpu` e sobre este shader, e um facto assim afirma-se e não se supõe**
— é o gate `a_rota_residente_aceita_a_meia_precisao_da_rasterizacao`.

### §5.3 — As TRÊS afirmações, com o número

| gate | o que afirma | medido |
|---|---|---|
| `a_rota_residente_e_a_mesma_lei_ao_bit` | com as MESMAS texturas (`f32`), as duas rotas saem **byte-idênticas** | **`0` componentes fora**, sem folga nenhuma |
| `a_rota_residente_aceita_a_meia_precisao_da_rasterizacao` | e ela aceita o `f16` da rasterização | **pior byte `1`**, `168` de `65 536` componentes (`0,26 %`) |
| `a_cerca_do_tamanho_do_base_recusa_em_voz_alta` | um `base` do tamanho errado é **recusado**, e a recusa diz o quê e por quanto | escrito por uma **mutação sobrevivente** (§5.4) |

⛔ **A primeira não tem barra de propósito:** a rota residente não é uma aproximação da carregada —
é a mesma lei com outra fonte para dois bindings. *Uma folga ali deixaria passar uma segunda
redacção do despacho.*

⭐ **E a segunda tem o CONTROLO na primeira:** com `f32` a mesma montagem lê `0`, logo o `1` mede o
**FORMATO** e mais nada. ⇒ **a meia precisão da rasterização chega ao passe da luz a um byte no pior
pixel, e não precisa de passe de conversão nenhum.**

### §5.4 — ⛔⛔ A prova de mutação, e as DUAS que ela devolveu contra o meu próprio instrumento

| # | o que a mutação apaga | onde | veredito |
|---|---|---|---|
| `M1` | a rota residente ignora as vistas e lê as texturas do passe | produto | ✅ **sangra** (2 de 3) |
| `M2` | o `base` da rota residente nunca sobe | produto | ✅ **sangra** (2 de 3) |
| `M3` | a cerca do tamanho do `base` deixa de recusar | produto | ⛔ **sobreviveu** → curada, hoje ✅ (1 de 3) |
| `M4` | a fixtura fica com o `base` PRETO | arnês | ⛔ **sobreviveu** → **NOMEADA**, ver abaixo |
| `M4-bis` | a **cobertura** da fixtura vai a `0` | arnês | ⛔ **sobreviveu** → curada, hoje ✅ (1 de 3) |
| `M4-ter` | a fixtura fica sem relevo nenhum | arnês | ✅ **sangra** (1 de 3) |

⛔ **`M3` — a cerca não tinha gate nenhum.** Ela recusa um `base` de tamanho errado, e sem ela o
`write_texture` do `wgpu` faz **panic** com fatia curta: esta porta corre **por quadro**, onde um
`Err` é uma cena que continua e um panic é o app a fechar. ⇒ `a_cerca_do_tamanho_do_base_recusa_em_voz_alta`,
com o **controlo positivo primeiro** — *sem ele, uma porta que recusasse SEMPRE passava o gate e a
mensagem lia-se exactamente igual.*

⛔⛔ **`M4` e `M4-bis` são a MESMA lição em duas camadas, e as duas são sobre a RÉGUA.** O vácuo que o
gate da igualdade ao bit tem de excluir é *as duas rotas concordarem por não haver FORMA nenhuma*, e
eu escrevi a grandeza errada **duas vezes**:

1. A 1.ª contava pixels **acesos** (`p[0] > 8`). Um `base` preto não os apaga — a luz acrescenta
   ambiente e especular. ⇒ a grandeza passou a ser a **excursão** do canal, não o brilho.
2. A 2.ª media a excursão da **imagem inteira**. Com a cobertura a `0` o shader devolve o albedo
   cru, ou seja um disco **chapado** sobre fundo transparente — e a excursão continuava a ler o
   degrau entre o disco e o fundo. *Uma régua que soma o fundo mede o **RECORTE** do objecto e não a
   forma dele*, e o recorte não é o que esta porta pode estragar. ⇒ a excursão passou a ser medida
   **dentro da silhueta**, com piso de população.

⭐⭐ **E a barra sai de um VALE MEDIDO que inclui o lado bom:**

| fixtura | pixels na silhueta | excursão no vermelho |
|---|---|---|
| a boa | `9 289` | **`188`** |
| `M4` — o `base` todo PRETO | `9 289` | **`246`** |
| `M4-bis` — a cobertura a `0` | `9 289` | **`0`** |
| `M4-ter` — sem relevo nenhum | `16 384` | **`0`** |

⛔⛔⛔ **A 2.ª linha é porque a `M4` não se cura: ela NÃO é um vácuo.** Um `base_color` preto tira a
**COR** e não a **FORMA** — o destaque especular não sai do albedo —, e a imagem fica **mais**
contrastada que a boa (`246` contra `188`). *Uma mutação que AUMENTA a grandeza sob teste não alcança
a propriedade que o gate afirma, e apertar a régua para a matar mediria outra coisa.* ⇒ ela fica
**nomeada** no gate e aqui, nunca silenciada.

⚠️ **O gate IMPRIME o número mesmo quando passa** — uma barra calibrada num vale cujo valor corrente
ninguém vê é a que envelhece em silêncio no dia em que a fixtura mudar.

---

## §6 — ⭐⭐⭐ W2: **a corrente viva** (construída em 2026-09-21)

A §5 abriu a costura; esta liga os dois elos e põe a promessa num PIXEL.

### §6.1 — As três peças, e o que cada uma custou

| peça | onde | o que ela é |
|---|---|---|
| a porta da LUZ | [`baked_form::forma_viva::acende_vivo`](../../crates/ph2d-form-donation/src/baked_form/forma_viva.rs) | a irmã da `light`, com a forma em vistas residentes |
| o DONO das texturas | [`vivo::FormaViva`](../../crates/ph2d-app-sculpt3d/src/vivo.rs) | as duas texturas que ficam na placa, mais o carimbo |
| a terceira saída da rasterização | `donation::gbuffer_vivo` | escreve nas vistas do chamador, com carimbo |

⭐⭐ **A costura já estava desenhada nas DUAS assinaturas e faltava um DONO.** O
`MeshRenderer` **não guarda** G-buffer nenhum e não expõe vista nenhuma — ele **aceita** as do
chamador; e a `acende_residente` **aceita** as mesmas. *O que a obra acrescenta não é um algoritmo,
é a posse.*

⭐⭐⭐ **E a POSE entra pela CÂMERA, o que custa ZERO.** As normais do G-buffer são de **VISTA**
(`canvas_normal(cam.view * model * n)`), logo orbitar a câmera em torno do alvo roda-as no
referencial em que o rig vive — que é exactamente *«o objecto virou-se e a luz acompanhou»*. Rodar
a MALHA daria a mesma imagem e pagaria um reenvio de vértices por quadro.

⚠️⚠️ **E o `Camera3d` não ter ROLL deixou de ser um limite:** o roll é a rotação **no plano**, e a
§1.5 mediu que essa a rota A já dá **exactamente**. *A câmera não sabe exprimir precisamente aquilo
de que esta rota não precisa.*

### §6.2 — O que a §5.0 POUPOU, medido antes de escrito

| o que o `02.2` pedia | o que a medição respondeu |
|---|---|
| *«com **dirty flag**: só re-renderiza se a pose, a malha ou a câmera mudarem»* | ⭐ **já existia** — o `donation::FormStamp` cobre as três, e o doc dele já escrevia a disciplina |
| o componente **`MeshShading`** (`sss`, `ao`, `cavity`, `material`) | ⛔ **não nasce**: a `material_da_forma()` **não recebe argumentos** (o material é GLOBAL) e a escolha por objecto que já é gravada é a `Lei` do assado ⇒ seriam **quatro knobs sem consumidor** |

⚠️ **O carimbo é do PAR `(malha, câmera-do-OBJECTO, tamanho)`** e não do da cena: dois objectos
vivos partilham a malha e têm poses diferentes, logo um carimbo da cena diria *«nada mudou»* ao
segundo depois de o primeiro ter rasterizado.

### §6.3 — A FRONTEIRA declarada: a rota B é da lei da FORMA

O despacho por [`Lei`](../../crates/ph2d-form-donation/src/lei_da_luz.rs) tem um braço de **TINTA**
que é o `ImpastoLightPass`, e ele **recebe fatias da CPU**. Não há por onde ele consumir uma vista de
textura ⇒ **a rota B não existe naquela lei**, e isso é uma propriedade do passe antigo e não uma
escolha desta porta. ⚠️ *Um despacho que aceitasse as duas e caísse em silêncio na de sempre
entregaria um catavento que não gira, sem dizer porquê.*

### §6.4 — A promessa, afirmada no PIXEL

| | componentes fora de `262 144` | pior byte |
|---|---|---|
| com relevo, virado `90°` fora do plano | **`31 678`** | `197` |
| esfera lisa (**CONTROLO**) | **`0`** | `0` |

⛔⛔ **O CONTROLO é o que dá direito à leitura**, e é a mesma fixtura do Bloco E: uma esfera lisa
**não tem orientação** — o campo de normais dela visto de uma câmera não depende de como ela está
rodada —, logo virá-la tem de deixar o sprite **igual**, e lê **zero ao bit**. *Sem essa metade, um
gate que medisse «mudou» passaria com uma rota B que re-rasterizasse ruído.*

⭐ **E a economia do carimbo tem CONTADOR** (`FormaViva::rasterizacoes`): ela é **invisível a toda
régua de valor** — com e sem carimbo a imagem é a mesma —, e *uma poupança que nenhum número mede é
uma poupança que ninguém defende*.

### §6.5 — O que fica para a wave seguinte

Os componentes no `ph2d-ecs` (com os quatro contadores de registo e a descrição no catálogo), a fase
do quadro e a cena que o dono possa smokar. ⚠️ **A cena tem de mostrar uma pá a VIRAR** — uma que
mostrasse a rotação no plano estaria a demonstrar uma coisa que a rota A já faz.

---

## §7 — ⭐⭐⭐ W3: **o catavento chega à cena** (construída em 2026-09-21)

O motor da §6 tinha tudo menos um dono: nada no mundo dizia *«este objecto é um catavento»*. A W3 é
o componente, a fase do quadro que o varre, e a cena `=52`.

### §7.1 — ⛔⛔ A pose 3D mora no COMPONENTE, e isso é uma MEDIÇÃO

O [`ph2d_ecs::Mesh3D`] carrega `piece`, `yaw`, `pitch` e `spin`. ⚠️ **O `Transform` desta casa tem
`rotation: f32`** e exprime **só** o plano do ecrã — que é, à letra, a rotação que a §1.5 mediu a
rota A a entregar **ao bit** (`0,00°` de desacordo de normais, contra `31,69°` fora do plano). ⇒ *a
rotação que justifica a rota B é exactamente a que FALTA àquele campo*, e é por isso que ela viaja
num componente novo em vez de num campo que já existe.

⛔ **E a PRESENÇA é a decisão, não um `live: bool`:** o `02.2` diz que a escolha entre as duas rotas
é *«uma propriedade do objeto»*, e ter ou não ter o componente **é** essa propriedade. Um campo ao
lado seria um segundo sítio a dizê-lo, e os dois divergiriam no primeiro dia em que alguém
escrevesse um sem o outro.

### §7.2 — ⭐⭐⭐ O `spin` é DERIVADO por quadro e NUNCA escrito de volta

`Mesh3D::yaw_em(t) = yaw + spin · TAU · t`. ⚠️ **A porta existe para o ângulo efectivo não ser
escrito no componente:** um componente registado reescrito a 60 Hz seria **um passo de `Ctrl+Z` por
quadro**, que é o defeito que o `preview_drive` desta casa existe para impedir. ⇒ o `yaw` gravado
continua a ser o que o artista autorou, e o giro compõe-se com ele **na leitura**.

⭐ **E o relógio é o do DOCUMENTO** (`Playhead::time`), não o da parede: rebobinar devolve a pá ao
sítio e um scrub mostra o quadro pedido. *Com o relógio de parede a peça continuaria a girar com a
régua parada, e o smoke não teria controlo nenhum.*

⚠️ **Custa um degrau de schema** (`162 → 163`) apesar de o componente ter nascido no degrau
anterior **no mesmo dia**: o postcard é POSICIONAL, logo um ficheiro gravado sem o campo lido com
ele sairia errado **em silêncio**. *Um campo num componente que já viaja custa um degrau, sempre.*

### §7.3 — A fase do quadro, e a ORDEM contra a irmã assada

A `fase_cataventos` corre **depois** da `fase_relight_baked_forms`. ⚠️ Um catavento é também um
objecto **assado** — a matéria (`base`) e o slot (`texture_id`) vêm do `BakedForm`, porque o albedo
que a luz multiplica é a ARTE do sprite e é a mesma nas duas rotas —, logo a irmã também o vê. Quem
escreve por último ganha o slot, e a rota B tem de ser essa.

⭐ **E a porta CARIMBA o `lit_with` do assado com o rig que acabou de usar**, o que faz a irmã
saltá-lo no quadro seguinte. *O carimbo diz a verdade literal (ele FOI aceso com este rig); um
remendo seria pôr ali um valor que ninguém usou.*

⛔ **Esta fase está atrás da `feature`, ao contrário da irmã, e a assimetria é a diferença entre as
duas rotas:** a rota A promete acender **sem** o módulo 3D no build (a forma dela viaja no
documento); esta RASTERIZA por quadro, logo precisa da malha.

### §7.4 — A cena `=52`, e o controlo dentro dela

A mesma mesa da `=11` (esfera com **cristas** + tela branca), com o componente semeado na tela.
⭐⭐ **O CONTROLO é o botão de PLAY:** com o transporte parado a peça fica no `yaw` autorado — que é,
ao bit, o que a rota A entregaria —, e a mesma tecla que a põe a girar é a que mostra a diferença.
*Uma cena que precisasse de uma irmã ao lado para ter controlo obrigaria o dono a comparar duas
sessões de memória.*

⛔⛔ **E a FOTO apanhou DOIS defeitos que os gates da cena não viam** (o roteiro é conduzido e
fotografado antes de ir ao dono): a tela nascia a dizer *«esculpa, aperte D até ler LUZ, pegue o
Painter e pinte»* — o texto da DOAÇÃO — enquanto o roteiro manda `Shift+B` (*uma cena que imprime
dois caminhos ensina o errado a metade de quem a lê*); e ⛔⛔⛔ **ela abria SEM a régua do tempo**,
com o passo (5) a mandar dar PLAY. Sem transporte o `playhead` fica em `0` ⇒ **o catavento nunca
gira** e o dono julgaria a wave sem nunca a ver — com os seis gates verdes, porque eles medem a
LEI e o que faltava era um PAINEL. ⭐ E a mesma foto mostrou `Dur(s) = 4` na régua, o que mudou o
`GIRO_DA_CENA` de um número do OLHO (`0,125`, meia volta) para um DERIVADO do recurso (`0,25`, uma
volta na duração que a régua abre).

⛔⛔ **A malha da cena NÃO pode ser lisa, e há gate:** uma esfera de raio constante é invariante a
toda rotação em torno do centro, logo o catavento giraria e **a imagem não mudaria** — a cena
ensinaria que a rota B não faz nada, que é a espécie que o `CLAUDE.md` §5.0 chama de *pior que uma
cena ausente*. A régua é o RAIO (exacta, barata, sem GPU): a `ridged_sphere` lê `0,262` de excursão
e uma esfera lisa lê `0,000`.

### §7.5 — ⏳ O que fica ABERTO, com o mecanismo

| o que | porquê fica | quem decide |
|---|---|---|
| ~~**o componente não tem CONTROLO**~~ | ✅ **FECHADO pela W4** (§8), por ordem do dono (*«sim. quero»*): a secção **`Live Mesh`** do Inspector, e o descritor passou de `Machinery` a `Authored` porque **a paleta é a única rota** — uma secção só pintada COM o componente não tem como o anexar | — |
| `piece` é sempre `0` | a rota B rasteriza a peça ACTIVA da cena 3D; o campo existe para o dia em que houver mais de uma, e declará-lo com um índice inventado prometeria uma escolha que o passe não faz | wave própria |
| ~~o giro só é autorável pela cena~~ | ✅ **metade FECHADA pela W4:** ele é um número do objecto (`Spin`, voltas/s). ⏳ A faixa da **timeline** — um catavento que acelera e pára — continua por fazer | o dono |
| o custo por quadro com N cataventos | medido para **UM** (§1.2: a corrente cabe `26×` num quadro a `512²`); a varredura em N não foi feita | wave própria |

## §8 — ⭐⭐⭐ W4: **o Live Mesh tem controlo**, e os dois reports do dono (2026-09-21)

A W3 fechou com o componente vivo e **inalcançável**: só a cena `=52` o
semeava. O dono respondeu ao item aberto com **«sim. quero»**, e a wave é a
secção do Inspector mais as duas correcções que ele devolveu no mesmo turno.

### §8.1 — ⛔⛔ A PALETA é a única rota, e é isso que decide o `Attach`

O descritor do `Mesh3D` vivia em `Attach::Machinery`, cujo doc diz por
escrito: *«nunca oferecido na paleta, **nunca uma secção do Inspector**»*.
Com a secção a existir isso deixa de ser verdade — e a pergunta seguinte não
é de arrumação, é de **ALCANCE**:

> uma secção do Inspector só é pintada **COM** o componente (ADR-0166) ⇒
> ela nunca pode ser a superfície que o **anexa**.

Logo, sem entrada na paleta, o artista tem o componente **exactamente** nas
cenas que já o semeiam, e em mais nenhuma. ⇒ `Attach::Authored { applies_to:
ObjectKind::IMAGE }`, com `insert_default` (que o `Authored` exige), e a
catraca `PORTAS` do catálogo de `11` para **`12`**, com o mecanismo escrito
na entrada.

⚠️ **O `applies_to` é `IMAGE` e não «tudo»:** a rota B acende **um sprite**
com a forma assada dele; oferecê-lo a um corpo de física ou a um caminho
vectorial seria um item de paleta que nunca produz pixel nenhum.

### §8.2 — A secção, e as três fileiras

`Live Mesh`, três fileiras e nada mais: `Yaw` · `Pitch` (graus, `−180..180`)
e `Spin` (voltas por segundo, `−2..2`). O molde é o da casa — `ids/` +
`sections/` + `sync_` (semente **por ARESTA**: a mão do artista ganha ao
instantâneo enquanto ele edita) + `populate_` (sem ele o widget é pintado e
**morto sob o dedo**) + `event_` (que lê o **SNAPSHOT**, nunca o store).

⚠️ **A unidade do `Spin` é a `Unit::PerSecond` que já existia.** Uma variante
nova (*voltas/s*) teria um sufixo terminado em `s` e teria de **PRECEDER** o
`Seconds` no `parse_suffix` — *uma tabela de sufixos é sensível à ordem, e
uma entrada nova escrita no fim lê-se como inerte*.

⭐ A conversão graus↔radianos vive numa **porta só**
([`vivo_inspector`](../../crates/ph2d-app-sculpt3d/src/vivo_inspector.rs)),
que constrói o instantâneo e dreno as edições; e ela **só escreve se o valor
mudou**, senão cada quadro de arrasto entrava no `Ctrl+Z`.

### §8.3 — ⭐⭐ A QUEIXA, da mais específica para a mais geral

Um `Live Mesh` pode estar quieto por **duas** razões que se leem iguais na
tela, e as curas são opostas:

| `Mesh3dQueixa` | o que o artista lê | a cura |
|---|---|---|
| `SemForma` | *Bake this sprite first — without a baked form nothing turns.* | `Shift+B` |
| `Parado` | *It is standing still — give it turns per second, or set the angles by hand.* | o `Spin` |

A ordem é **load-bearing** e vive na porta: *dizer «está quieto» a quem
também não tem forma manda o artista resolver a metade errada* — a mesma lei
do `recusa::Entradas` da escultura, uma família acima.

### §8.4 — Os seis vermelhos que o portão apanhou

Nenhum deles é visível no laço interno (`cargo check -p`), e é por isso que
estão escritos aqui:

| vermelho | cura |
|---|---|
| o censo do catálogo (`PORTAS` 11 contra 12) | a entrada nova, com a razão §8.1 dentro |
| `paint_optional_sections` a `212` LOC | **CORTE**: `paint_a_cauda_da_rodada` (arma + catavento), nunca uma isenção |
| `fase_snapshots_publish` a `202` LOC | **CORTE**: a função livre `ja_esta_assado` |
| `180.0` acusado como número mágico | é **geometria** (meia volta em GRAUS) e não desenho ⇒ `LITERAL-PX-OK` com a razão |
| `toda_porta_do_inspector_e_armada` | a secção entra na lista, e o `set_current_inspector_mesh3d` arma com `assado: true` **de propósito** — *uma fixtura no estado degenerado mede a metade que o artista menos vê* |
| `…_e_desarmada` | o irmão: `desarma_tudo()` limpa-a |

### §8.5 — ⛔⛔⛔ O report do bake, e a RECUSA que eu escrevi e que foi REFUTADA no dia seguinte

Ele escreveu *«em sprite transparente o bake fica invisível»*. Eu **impedi o gesto** — e a foto
seguinte (*«o objeto continua sem assar»*, com o aviso na tela) era o mesmo pedido pela segunda
vez: ⛔ *ele não queria ser impedido; ele queria que funcionasse.*

⭐ **A rota da cena foi ILIBADA por medição antes da 1.ª linha de cura:** o
`the_bake_gesture_lights_the_selected_sprite` (GPU, `#[ignore]` — é por isso que o portão nunca o
correu) percorre a rota exacta do dono e lê **verde**, `10 237` de `65 536` texels fora da chapa
branca. ⇒ a sprite que ele assou era **dele**, e a minha cerca negava arte legítima.

**A lei que fica** ([`albedo::veste_a_forma`](../../crates/ph2d-app-sculpt3d/src/albedo.rs)): *onde
o sprite não tem nada e a peça tem, a matéria passa a ser a **neutra***. A silhueta que faltava já
estava rasterizada ao lado — o G-buffer é `[nx, ny, nz, **cobertura**]` por texel —, e o **branco**
é o que as cenas de bake desta casa já põem na mesa (`bg: 2`), porque *a luz da forma MULTIPLICA*.

| | opacos | sombreados | cantos |
|---|---|---|---|
| tela **branca** (`bg: 2`) | — | `10 237` de `65 536` | — |
| tela **vazia** (`bg: 0`) | `12 284` | **`10 237`** | **`0`** |

⭐ Os `10 237` são os **mesmos**: a forma acende exactamente os mesmos texels, e o que muda é só a
silhueta. ⭐⭐ E a metade dos **cantos** é a que separa *«vestir a forma»* de *«pintar a tela de
branco»* — a peça é uma bola, logo uma lei que pintasse o rectângulo passaria em tudo o resto.

⛔⛔ **A cerca é «o sprite INTEIRO está vazio»**, e sem ela isto seria uma regressão grave: um
personagem **recortado** sobre transparente é o caso normal deste app, e texel a texel a peça
pintaria branco na zona recortada — *o recorte deixaria de ser recorte*.

### §8.6 — ⛔⛔⛔ O report da língua, e os TRÊS defeitos que uma foto tinha dentro

`✓ [sculpt3d] nao assou: this sprite is fully tra…`

**(a) A isenção que o deixou passar tinha a premissa FALSA.** O `bake.rs` estava isento do censo de
texto como *«as linhas `[sculpt3d]` do ASSAR no **terminal**»* — e a fase faz `eprintln!(…)` **e**
`toasts.push(…)` com a mesma `String`. ⚠️ E o `albedo.rs` saiu com ele pela razão que a própria
isenção dele escrevia (*«a cauda da MESMA frase»*): **uma isenção que herda a premissa de outra
herda o erro dela**. Retiradas as duas, o gate achou na hora uma **terceira** frase que elas
escondiam.

**(b) Uma recusa tinha a cara de um sucesso.** A porta devolvia uma `String`, logo a fase não podia
saber o que acontecera. ⇒ [`bake::Veredito`]. ⛔⛔ **E uma mutação sobrevivente mostrou que a cura
ficou sem régua durante uma hora:** trocar `Err(…) => Recusado` por `Assado` passava os **dois**
gates de GPU do gesto e a suíte inteira — *eles só percorrem o caminho que ASSA*.

**(c) O gate novo apanhou o IRMÃO antes do dono.** O gesto da imagem-padrão tinha o mesmo defeito
no mesmo ficheiro: três frases portuguesas e as duas recusas de verde.

### §8.7 — O que o portão cobrou pelo corte, e o que cada cura ensinou

| cobrança | cura |
|---|---|
| tecto de **função** (`218` de `200`) | a fase tinha **dois assuntos** ⇒ `fase_sculpt3d_alpha`, por responsabilidade. ⭐ Declarada e chamada pela **irmã**, porque o índice do quadro estava a UMA linha do tecto dele e o `splice` do texto emendado é **recursivo** |
| tecto da **shell** (`+85`) | por **MOVER**: a lei do padrão vive agora em `ph2d_app_sculpt3d::alpha_pedido`, com as duas fontes por **assinatura** — o molde que o `bake::drain` já usava |
| **três** isenções (downcast · precisão · texto) | elas **VIAJAM com o código**, e as duas metades acusaram na mesma corrida: o órfão de um lado, o sem-abrigo do outro |
| a entrada da **precisão** | ⛔ quase mudou por engano: com as duas fases a ler pixels a pergunta passou a ser *«qual deles ESCREVE de volta»*. O alpha só lê ⇒ `PRECISION-READONLY`. **Enquanto partilhavam ficheiro, a entrada da irmã abrigava os dois** |

⭐⭐ E **dois gates de texto viraram leis MEDIDAS** ao mudarem de casa: a ordem das duas fontes (as
camadas vivas antes da imagem guardada) passa a ser afirmada **contando as leituras**.

⚠️ E caiu hoje a **terceira** âncora de gate presa a prosa traduzível (`"escala {scale:"`, depois
de `"=52 O CATAVENTO"`).

---

## ⛔ Recusas MEDIDAS

| o que | porquê | onde |
|---|---|---|
| chamar a porta `form_plane` por quadro | `4` objectos a `512²` contra `26` — o readback é `31×` a rasterização | §1.2 |
| baixar a resolução do G-buffer para poupar relógio | não compra rasterização (ela é plana no lado) **nem** acendida (o passe despacha sobre os pixels do SPRITE) | §1.1 + §1.2-bis + §1.3 |
| medir a rasterização com uma esfera leve | o custo é de VÉRTICES: `33×` de malha vale `1,6×` de tempo, e a fixtura não continha a grandeza | §1.4 |
| dividir o orçamento pelo custo de RASTERIZAR | é metade da corrente: a conta dá `126` objectos e a corrente inteira dá `26` | §1.2-bis |
| o componente `MeshShading` do `02.2` | o material é GLOBAL (`material_da_forma()` não recebe argumentos) e a escolha por objecto já é a `Lei` gravada ⇒ quatro knobs sem consumidor | §6.2 |
| um *dirty flag* próprio para a rota B | o `FormStamp` da doação já cobre malha · câmera · tamanho | §6.2 |
| rodar a MALHA para exprimir a pose | as normais são de VISTA ⇒ orbitar a câmera dá a mesma imagem e custa zero reenvio de vértices | §6.1 |
| a rota B para a rotação NO PLANO | a rota A dá-a **exactamente** (`0,00°` contra `28,58°` de um plano fixo): são duas operações 2D sobre o plano assado | §1.5 |
| pôr a pose 3D da malha no `Transform` do filho | ele tem `rotation: f32` e só exprime o plano do ecrã — a rotação que justifica a obra é **inexprimível** ali | §1.5 |
| apertar o controlo de vácuo até a fixtura PRETA o disparar | ela não é um vácuo: tira a COR e não a FORMA, e lê `246` de excursão contra `188` da boa — apertar mediria outra grandeza | §5.4 |
| dar folga ao gate da igualdade `f32` | a rota residente não é uma aproximação: uma barra ali deixa passar uma 2.ª redacção do despacho | §5.3 |
| escrever o ângulo efectivo de volta no `Mesh3D` | um componente registado reescrito a 60 Hz é **um passo de `Ctrl+Z` por quadro** | §7.2 |
| pôr o giro no `Default` do componente | toda peça do app passaria a girar; o giro é da CENA, e há gate nas duas metades | §7.4 |
| abrir a `=52` com uma esfera LISA | raio constante ⇒ invariante à rotação ⇒ a cena ensinaria que a rota B não faz nada | §7.4 |
| usar o relógio da PAREDE para o giro | a peça continuaria a girar com a régua parada, e o controlo da cena (o botão de Play) deixaria de existir | §7.2 |
| uma `Unit` nova para *voltas por segundo* | o sufixo dela acabaria em `s` e teria de **PRECEDER** `Seconds` no `parse_suffix` — *uma tabela de sufixos é sensível à ordem, e uma entrada nova no fim lê-se como inerte*; a `Unit::PerSecond` já exprime a grandeza | §8.2 |
| deixar o descritor em `Machinery` e oferecer o componente por um botão | `Machinery` proíbe por escrito a secção do Inspector, e a secção só é pintada COM o componente (ADR-0166) ⇒ **não existe superfície sempre visível que o anexe**: sem a paleta o artista nunca lá chega | §8.1 |
| ⛔ ~~RECUSAR o bake de um sprite transparente~~ | **REFUTADA no dia seguinte pelo próprio dono** (*«o objeto continua sem assar»*): ele queria que funcionasse, não ser impedido. A lei que ficou é vestir a silhueta da peça | §8.5 |
| a cerca de vazio texel a texel | um personagem **recortado** é o caso normal: a peça pintaria branco na zona recortada e *o recorte deixaria de ser recorte* | §8.5 |
| `255` no alfa da matéria vestida | a cobertura da borda é fraccionária — um `255` chapado devolve a peça **serrilhada**, e o canal já tem a resposta suave | §8.5 |
| pôr a lei do vestido no `materia_para` | a cobertura só existe depois do `form_plane_for`, e a matéria é lida ANTES porque é ela que decide o TAMANHO da rasterização | §8.5 |
| declarar o `bake.rs` como código de TERMINAL | a fase faz `eprintln!` **e** `toasts.push` com a mesma `String` — a isenção era metade da verdade, e a outra metade era a foto do dono | §8.6 |
| medir a ORDEM das fontes do padrão pelo escrutínio de um `match` | uma régua de texto sobre quem chama a porta; contar as leituras é a lei, e sobrevive à mudança de casa | §8.7 |
| subir o tecto da shell para caber os gates novos | a cura é MOVER a lei para a crate da família, e foi ela que pagou as `85` linhas | §8.7 |
| uma âncora de gate feita do TÍTULO de uma cena | ela reprova no dia da tradução, e o defeito que o gate existe para apanhar continua vivo — a âncora é o **número** (`=52 `) | §8.6 |

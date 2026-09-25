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
o componente, a fase do quadro que o varre, e a cena `=53`.

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

### §7.4 — A cena `=53`, e o controlo dentro dela

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

A W3 fechou com o componente vivo e **inalcançável**: só a cena `=53` o
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
de `"=53 O CATAVENTO"`).

## §9 — ⭐⭐⭐ O smoke seguinte: o barro PRETO e o painel sem controlos (2026-09-21)

O dono aprovou a lei do vestido com **duas** observações, e as duas eram reais.

### §9.1 — ⛔⛔⛔ *«retiro a sprite branca, coloco um transparente e o objecto 3D fica PRETO»*

A causa é a **mesma porta**, no **outro consumidor** dela. O [`albedo::sincroniza`] pinta o barro
com a matéria que o bake vai acender — é a promessa escrita do módulo — e usa a mesma
`materia_para`. Uma matéria `[0,0,0,0]` subida ao device é **preto opaco**, não invisível.

⚠️⚠️ **E a RECUSA de ontem escondia isto por ACIDENTE:** com o `Err`, a matéria vazia nunca chegava
ao `set_albedo_source`. ⇒ *retirar uma recusa devolve todos os caminhos que ela calava, não só o
que a motivou* — e quem os conta é o `grep` pelos chamadores da porta, antes de a mexer.

| | cobertura | porquê |
|---|---|---|
| **bake** | `Cobertura::DaForma` (o canal `w` do G-buffer) | ali a peça tem silhueta, e é ela que o objecto veste |
| **visor** | `Cobertura::Toda` | o sujeito é o **barro inteiro**, e o visor não rasteriza forma nenhuma — ele corre por quadro |

⛔⛔ **E uma MUTAÇÃO SOBREVIVENTE apanhou a metade que faltava:** apagar a chamada no `sincroniza`
deixava o gate **verde** — *um gate que chama a função afirma que a lei existe, nunca que o
consumidor a usa*. ⇒ a 2.ª metade lê o corpo por `include_str!`, que deixa de **compilar** se o
ficheiro mudar de sítio.

### §9.2 — ⭐⭐ *«ao assar com a sprite transparente, não aparece no Inspector os controlos»*

A secção `Live Mesh` só é pintada **com** o componente (ADR-0166), e até aqui **só a cena `=53` o
semeava** — o artista que assava a peça dele tinha a forma 3D no objecto e **nenhuma superfície
para a virar**. *Um motor com a lei certa e o artista sem lhe chegar lê-se, da cadeira dele, como
um motor sem a lei.*

⇒ **assar carimba o `Mesh3D`.** O que o bake produz **é** uma forma 3D doada, e virá-la é a razão
de a rota B existir.

⭐ **É barato:** ele nasce em `yaw = pitch = spin = 0`, a identidade da rota B **ao bit**.
⚠️ **E só na PRIMEIRA vez** (`get` antes do `insert`): re-assar não pode devolver a pose ao zero,
senão um `Shift+B` apagava o giro que o artista acabou de pôr — o mesmo argumento que o
`lei_ao_assar` e o slot da textura já fazem no mesmo ficheiro.

---

---

## §10 — ⭐⭐⭐⭐ **O QUE SE VÊ É O QUE SE ASSA**, e a LENTE (2026-09-21)

**Report do dono, com foto, três queixas numa mensagem:**

1. *«Só temos a visão em perspectiva em sculpt. Não temos Ortográfica. Precisamos de ambas.»*
2. *«O Bake não é feito projetando o objeto 3d exatamente como o posiciono sobre a sprite e tem
   perspectiva, posição e escala diferente do que eu coloquei.»*
3. *«Depois do Bake e antes de apertar o D para retirar o objeto 3d original, o bake 3d recebe zoom
   e fica como na imagem: um fundo deslocado do objeto 3d.»*

⭐ **As duas últimas são UM defeito com dois relatos**, e a terceira é a metade que se vê primeiro —
porque a rota B re-rasteriza por quadro, logo o enquadramento errado aparece antes de a malha sair
da tela.

### §10.1 — O mecanismo, em aritmética

A porta de assar rasterizava a forma no alvo **INTEIRO**. Logo a peça ocupava, dentro dos texels do
sprite, a mesma fracção que ocupava **da altura do VIEWPORT** — e o sprite é um rectângulo *dentro*
dele. Daí saem exactamente as duas queixas:

| grandeza | erro |
|---|---|
| escala | `altura da vista ÷ altura do sprite no ecrã` |
| posição | a distância entre o centro da vista e o centro do sprite |

⛔⛔ **Duas curas foram descartadas antes da primeira linha:** um *dolly* (afastar a câmera) muda a
CONVERGÊNCIA e não só o enquadramento, logo com lente convergente ele não é exprimível como um
recorte; e redimensionar o sprite para casar com a vista é reescrever o documento do artista.

⭐ **A cura é um frustum FORA DO EIXO** — a única forma exacta de dizer *«a mesma imagem, dentro
deste outro rectângulo»* sob uma lente convergente. Ele vive na
[`ph2d_mesh_render::ViewRegion`](../../crates/ph2d-mesh-render/src/view_region.rs), com
`to_clip()` a devolver um afim de espaço de clip (as constantes multiplicam `w`, porque
`ndc = clip/w`).

⚠️ **O aspecto é sempre o da VISTA INTEIRA, nunca o do sub-rectângulo:** a forma do frustum é da
vista, e o recorte só diz *que pedaço dela este alvo desenha*. Trocar os dois estica a peça.

### §10.2 — As duas metades que não se conhecem (a regra 2 da W2)

| quem | sabe |
|---|---|
| a **shell** | onde a arte de um sprite aterra na janela — ela tem a câmera 2D, a janela e o afim partilhado |
| a **família** | onde a vista 3D está, e como recortar o frustum do escultor |

⭐ O afim `imagem-px → ecrã-px` já tinha **quatro** consumidores, logo a shell pergunta-o à folha
partilhada ([`ph2d_sprite_screen::rect_no_ecra`](../../crates/ph2d-sprite-screen/src/lib.rs)) — uma
quinta cópia divergiria no dia em que uma folha desdobrada ou uma rotação mudassem de lei.

⛔⛔ **E a janela que ele recebe é a da CENA, não a da JANELA** — apanhado pelo
`quem_desenha_no_mundo_tambem_usa_a_banda`: com a ferramenta Motion na mão o chrome da cena é
desenhado numa **banda**, e a superfície inteira devolveria um rectângulo `~340 px` fora do sítio
(o defeito que o HUD pagou em 17/09). *Aqui ele seria pior que um clique perdido: o assado ficaria
enquadrado sobre um pedaço de ecrã em que o sprite não está.*

### §10.3 — O recorte VIAJA no arquivo, e o motivo é a rota B

`BakedFormDocument::recorte` (degrau `164` do `PROJECT_SCHEMA`). ⛔ **Sem ele o catavento SALTAVA ao
reabrir:** a rota B re-rasteriza por quadro, e um objecto vivo cujo recorte morresse no disco
voltaria a encher o sprite no primeiro quadro em que o relógio andasse.

⚠️ **`None` num documento anterior é a leitura honesta** — até aqui a forma era sempre rasterizada
com a vista inteira, logo um ficheiro velho reabre **sem uma linha de diferença**. O degrau existe
na mesma, porque *o postcard é POSICIONAL*.

⛔⛔ **E o tipo é escrito DUAS vezes de propósito:** a `ph2d-form-donation` é a fronteira que o
runtime atravessa **sem o módulo 3D**, e um `use ph2d_mesh_render::Framing` traria o `wgpu`, os
matcaps e o `imageio` com ele. ⇒ dois tipos de dados puros e **uma** travessia, com gate de
ida-e-volta (`ph2d_app_sculpt3d::recorte`). *O precedente é o `ph2d_pose::pesos`.*

### §10.4 — A LENTE

[`ph2d_mesh_render::Lens`](../../crates/ph2d-mesh-render/src/lens.rs): `Perspective` (fábrica) e
`Ortho`. ⭐ **Não há vocabulário novo:** o modelador implícito desta casa já shipava a mesma escolha
com a mesma tecla (`Numpad5`, a do Blender) e a mesma lei — **as duas lentes coincidem exactamente
no plano do alvo**, com a meia-extensão da paralela a ser `distance · tan(fov/2)`
([`Camera3d::view_height`](../../crates/ph2d-mesh-render/src/camera.rs)).

**Duas portas, uma função:** o `Numpad5` (lido da MESMA tabela do módulo vizinho,
`ph2d_viewport3d::views::is_lens_key`) e a fileira `Lens` na secção *Shading* do painel. ⚠️ Ela é
**da câmera ACTIVA e não da cena** — com a divisão em quatro cada quadrante tem a sua, que é o que
torna útil olhar a mesma peça de duas lentes ao mesmo tempo.

⚠️ **A lente entra no `FormStamp`**, e sem isso trocá-la deixava a forma viva a descrever a peça
vista pela lente de antes, **em silêncio**: nenhuma das outras sete entradas do carimbo se mexe com
ela.

### §10.5 — ⛔⛔ O achado de RÉGUA: uma paridade é cega a um erro COMUM

**DUAS mutações SOBREVIVERAM** à primeira ronda: dobrar o `view_height() * 0.5` **da projecção** e
**do lançador de raios** passava os `97` testes da crate.

⭐ *A causa é estrutural, e vale para toda lente que alguém acrescente:* tudo o que havia sobre a
paralela ou era uma **RELAÇÃO** (o recorte medido contra a vista inteira — cega a um factor comum
aos dois lados) ou uma propriedade de **FORMA** (*sob raios paralelos o que varia com o pixel é a
origem, não a direcção* — cega à escala). **Nenhuma régua dava à paralela um valor ABSOLUTO.**

⇒ dois gates novos, cada um com o CONTROLO que impede a lei de ser satisfeita por
`ortho == perspectiva`:

| gate | o que afirma | o que o controlo proíbe |
|---|---|---|
| `as_duas_lentes_coincidem_no_plano_do_alvo` | três pontos do plano do alvo caem no mesmo NDC nas duas | fora daquele plano elas TÊM de divergir |
| `o_raio_de_um_pixel_fura_o_plano_do_alvo_no_mesmo_ponto` | o pick da paralela mira onde ela desenha | as duas origens TÊM de se afastar |

⚠️ *Sem o segundo, o defeito seria «o lugar onde o mouse toca não corresponde ao local na malha»* —
a família que esta casa já nomeia —, na lente nova e sem régua nenhuma.

### §10.6 — ⏳ Dívida DECLARADA

⛔ **O estêncil de alfa continua a tratar a vista como convergente sob a lente paralela**
(`Camera3d::view_height_per_depth` devolve uma razão por profundidade, que sob raios paralelos é
constante). A cura tem endereço — `span = base + depth × ratio` no `AlphaStencil` — e é wave
própria; hoje o que se vê é o padrão por imagem a mudar de tamanho com a profundidade numa lente em
que ele não devia.

---

---

## §11 — ⭐⭐⭐⭐ **A MATÉRIA DA PEÇA**, e a «máscara» que era uma silhueta CONGELADA (2026-09-21)

### §11.1 — O report, e a leitura que ele convida

> *«veja: O algoritmo que vc criou tem esse fundo branco na sprite transparente. logo que roda o
> objeto o fundo aparece. OU seja: parece que vc criou uma máscara. Encontre o modo de tirar a
> máscara. que o objeto pleno no fundo transparente.»* — o dono, com foto e uma seta vermelha.

⛔⛔ **A palavra *«máscara»* aponta para o sítio errado, e é por isso que ela merece esta secção.**
Não há máscara nenhuma no caminho: o que há é uma **SILHUETA CONGELADA**.

A §8.5 desta wave pôs o `albedo::veste_a_forma` — quando o sprite chega **inteiramente
transparente**, o bake veste-o da peça: branco (o neutro multiplicativo) com o alfa da **cobertura**
do G-buffer. ⚠️ **E ele escreve isso no `base`, que é o plano que o documento GRAVA.**

A rota A (o assado) é coerente com isso, porque a forma e o `base` foram escritos no MESMO gesto. A
rota B **não é**: ela re-rasteriza a forma **por quadro** e lê o `base` do primeiro gesto ⇒

| o que segue a peça a virar | o que NÃO seguia |
|---|---|
| as normais (re-rasterizadas) | — |
| a luz e a sombra (recalculadas) | — |
| a oclusão de forma | — |
| — | **o ALFA**, que vinha do `base` de ontem |

⇒ *a peça rodava por baixo do recorte dela própria*, e o que aparecia por trás era o branco que o
vestir tinha deixado. **Ele descreveu o sintoma com precisão e o mecanismo tinha outro nome.**

### §11.2 — A medição que fechou o diagnóstico

A primeira leitura do código disse o contrário: o doc do `acende_faixa` promete *«o ALFA atravessa
intacto»*, o que se lê como *«o bake não escreve alfa nenhum»*. A sonda `diag_o_alfa_do_assado`
(versionada, `#[ignore]`) mediu os três fundos de tela:

| fundo da tela | assou? | opacos | vazios | canto | centro |
|---|---|---|---|---|---|
| transparente | sim | `12 284` | `53 252` | `[0,0,0,0]` | `[211,213,218,255]` |
| preto | sim | `65 536` | `0` | `[0,0,0,255]` | `[26,27,29,255]` |
| branco | sim | `65 536` | `0` | `[255,255,255,255]` | `[211,213,218,255]` |

⇒ **o bake DE FACTO escreve alfa** numa tela transparente — `53 252` texels vazios de `65 536` —, e
quem o escreve é o vestir. *A promessa do `acende_faixa` é verdadeira sobre a LEI e a lei não era o
sítio onde o alfa nascia.*

### §11.3 — A cura: um facto por OBJECTO

`BakedForm::materia_da_forma` (e o gémeo `ph2d_form_pbr::imagem::Planos::materia_da_forma`) —
*este sprite não tinha arte, logo o que ele mostra é a peça 3D e mais nada*. Com ele ligado:

* o **albedo** é o **neutro** (branco) — exactamente o que o vestir escreve, logo na pose do bake a
  saída não se mexe um bit;
* o **alfa** é a **cobertura DESTE quadro** — a silhueta passa a seguir a peça.

⭐ **Ele nasce no GESTO que assa** (`materia_da_forma: vestidos > 0`), viaja no documento (degrau
`165`) e atravessa **quatro** sítios em duas crates até à porta da rota B. ⚠️ *Um motor com a lei
certa e a shell a não a ligar lê-se como um motor sem a lei* — e é por isso que a fiação tem um
censo derivado próprio (`materia_da_forma_tests`), com a coluna da contagem.

⛔⛔ **É por OBJECTO e nunca por TEXEL, e a diferença tem um defeito com nome.** Uma regra
por-texel — *«onde o `base` é transparente, usa o alfa da forma»* — parece equivalente e não é: num
sprite com arte **DESENHADA** ela encheria de branco toda a volta do desenho sempre que a malha
fosse maior do que ele. *A pergunta é «este sprite tem arte?», e ela responde-se UMA vez.* É a mesma
cerca que a §8.5 já tinha pago para o vestir.

### §11.4 — O gémeo na placa, e a paridade

O bit sobe no `y` do `vec4<u32>` da vista, que era reserva declarada ⇒ **a disposição do uniform não
se mexe** e o gate que a prende fica intacto. Medido num adaptador real, com a fixtura das quatro
bolas, **nos dois lados do interruptor**:

| `materia_da_forma` | `|Δ| = 0` | pior |
|---|---|---|
| `false` | `100,000 %` de `262 144` bytes | `0` |
| `true` | `100,000 %` de `262 144` bytes | `0` |

⚠️ **A segunda linha só existe porque o gate passou a percorrer as DUAS.** Sem o laço, *nenhuma
corrida deste repo tocava nas duas linhas novas do shader* — é letra por letra a cegueira que o
matcap pagou com um report do dono (`docs/3D`, §101: *«uma suíte que nunca arma o canal não pode ver
o canal a ser deitado fora»*).

### §11.5 — E a CENA tinha de mudar, senão a wave era invisível

⛔⛔⛔ A `=53` pedia uma tela **BRANCA** (`bg: 2`). Sobre branco o vestir **nunca arma** — o sprite
já tem alfa em todo o lado —, logo `materia_da_forma` seria sempre `false` e a cena mostraria a peça
dentro de um cartão branco: **exactamente o que a cura existe para tirar**.

⇒ o fundo passa a ser da CENA (`donation::fundo_da_tela`): a `=11` continua a julgar a **LUZ** sobre
branco, que é o neutro multiplicativo, e a `=53` pede **transparente**, porque o que ela julga é o
**RECORTE a virar**. ⚠️ *Uma cena que não contém o fenómeno é o mesmo que uma cena ausente*, e o dono
aprova-a sem nunca julgar a metade que importa.

### §11.6 — A régua do produto, e o eixo que foi MEDIDO

O gate `catavento_com_materia_da_forma_a_silhueta_segue_a_peca` corre a rota B inteira num
adaptador e lê o ALFA do sprite. A fixtura é um **TORO** e a escolha é a régua: a silhueta de uma
esfera não muda com pose nenhuma — ela é o CONTROLO dos gates vizinhos, e seria exactamente a
fixtura que **não contém** o fenómeno.

⛔⛔ **E o EIXO foi medido, não escolhido.** A 1.ª redacção virava pelo `yaw` e leu **`0` de
`65 536`** texels a mexer: este toro assenta no plano do `yaw`, logo rodá-lo por ali é simétrico.
Pelo `pitch` ele passa de `6 624` para `13 776` texels opacos, com `12 680` a mudar de alfa.

⚠️ *Uma fixtura que não contém o fenómeno lê-se exactamente como uma lei que não chega ao pixel*, e
as duas curas eram opostas — o que as separou foi a **linha dos opacos ficar impressa ao lado do
veredito**.

⭐ E o CONTROLO é o report: com a lei desligada o alfa lê `255` nas duas poses, em todo o texel.

### §11.7 — ⛔⛔⛔ A ORLA BRANCA: a premissa que eu declarei e a foto derrubou

**A redacção anterior desta secção dizia:**

> ⚠️ *Fora da cobertura o RGB passa de `[0,0,0]` a branco, com o alfa a `0` nos dois casos. Isso é
> **deliberado e melhor**: é o halo que uma amostragem bilinear puxa para dentro da borda, e branco
> ao pé de branco não deixa orla escura.*

⛔⛔ **O report seguinte do dono, com foto:** *«funcionou mas o objeto fica com uma outline branca
pixelada indesejada»*.

⚠️⚠️ **O erro é de COMPARAÇÃO e cabe numa frase:** *branco ao pé de branco* comparava o exterior com
o **ALBEDO**, e o que está do outro lado da borda é a **SAÍDA** — o cinzento **ACESO**. Medido no
caminho da lei, num disco:

| | R |
|---|---|
| miolo da peça (aceso) | `183` |
| logo fora da silhueta | **`255`** |
| ⇒ degrau que a filtragem arrasta para dentro da borda | **`72` códigos** |

*Uma troca declarada sem a medição ao lado é um palpite com cara de decisão.*

### §11.7-bis — O mecanismo, e porque ele é a COBERTURA aplicada duas vezes

A mistura do `acende_texel` é uma **COMPOSIÇÃO sobre o albedo**:

```
saída = albedo × (1 − cobertura) + aceso × cobertura
```

Ela está **certa** quando existe arte por baixo: num texel de borda meio coberto, metade do pixel é
o cartão do artista e metade é a peça. ⛔ Com a matéria a ser a forma **não existe nada por baixo** —
o albedo é o neutro e quem diz *«aqui não há nada»* é o **ALFA**. ⇒ a cobertura era aplicada **duas
vezes**, uma na cor (a puxar para o branco) e outra no alfa, e fora da silhueta sobrava o albedo
verbatim, que é branco puro.

⭐ **A cura é uma linha nas duas redacções da lei:** com a matéria a ser a forma a cobertura entra
**CHEIA**, porque ela já viaja no alfa. Degrau **`72 → 1`**, e a paridade placa↔régua continua
**`100,000 %` com pior `0`** nos dois lados do interruptor.

⚠️ **E o gate que prometia byte-identidade na pose do bake tinha a mesma premissa**: ela é verdadeira
num texel **CHEIO** (onde `c = 1` torna a mistura a identidade) e falsa num de **BORDA**. Ele foi
reescrito nas duas metades, e a metade nova exige que o texto de borda fique **mais escuro ou
igual** — *a mudança é a cura, e ela tem sentido*.

⛔⛔ **E a orla NÃO nasceu nesta wave: ela nasceu com o VESTIR (§8.5), um dia antes.** Qualquer bake
de sprite vazio já escrevia branco onde a peça não está. O que esta wave fez foi tornar a borda
**visível e móvel** — antes ela estava congelada dentro de um cartão. ⭐ A cura cobre exactamente a
população certa **por construção**: `materia_da_forma` é `vestidos > 0`, logo *vestido ⟺ curado*.

### §11.7-ter — ⛔⛔⛔ E ela NÃO estava curada: o 2.º report, e a régua que não continha o fenómeno

> *«não resolveu. é o branco de baixo com alpha ruim»* — o dono, com foto.

**Ele acertou no plano.** Medido no caminho do PRODUTO e na placa (a sonda `diag_o_perfil_da_borda`,
versionada), a linha do meio de uma esfera:

| x | R,G,B | alfa | |
|---|---|---|---|
| `190` | `131,135,143` | `255` | último texel da peça |
| `191` | **`255,255,255`** | `0` | |
| `192`+ | `255,255,255` | `0` | |

⇒ a cura da §11.7-bis endireitou a **mistura** e não tocou no que está **fora**: ali a normal do
G-buffer é **degenerada** (o rasterizador deixa zeros), a lei cai no **albedo verbatim**, e com esta
bandeira ele é branco puro.

⚠️⚠️ **O alfa `0` não basta, e é isso que engana.** Uma amostragem bilinear entre `190` e `191`
devolve `mix(131, 255) = 193` com alfa `128`, que sobre o fundo dá **`160` contra `131`** do miolo —
*um fio mais claro que a peça*, com a largura de um texel e a forma da grelha. É a orla, e é por isso
que ela é **pixelada**.

⭐ **A cura é a da indústria e tem nome: *edge padding* / *alpha bleed*.** O texel vazio herda a
FORMA dos vizinhos cobertos — a média das normais e da oclusão — e a lei acende-o com ela; o **alfa
continua `0`**, logo nada de novo se vê. Medido no mesmo sítio: `191` passa a ler **`131,135,143`**,
o degrau através da borda vai a **`0`**, e a paridade placa↔régua continua **`100,000 %`, pior `0`**.

⛔ **A MÉDIA e não «o primeiro vizinho»:** com a cobertura BINÁRIA que o G-buffer entrega, escolher
um seria escolher pela ORDEM da varredura, e as duas redacções da lei teriam de concordar nessa
ordem para a paridade fechar. *Uma média é simétrica e não tem ordem* — e o que faz a paridade fechar
ao bit é a varredura ser a mesma dos dois lados (`dy` de `−1` a `1`, `dx` dentro).

⚠️ **UM anel, e o recurso é a AMOSTRAGEM:** a magnificação bilinear lê no máximo um texel de
distância. ⏳ **Mipmaps pediriam mais** — declarado, não medido, e o gate afirma-o pelo lado
positivo: a dois texels a lei **tem** de voltar ao branco.

⛔⛔⛔ **E A 1.ª REDACÇÃO DO GATE FICOU VERDE SOBRE ISTO.** A fixtura dela escrevia uma normal
**válida** fora da silhueta, logo a lei acendia-a em vez de cair no albedo — *uma fixtura que não
contém o fenómeno lê-se exactamente como uma lei que já o cura*. Quem o apanhou foi a foto do dono,
**pela segunda vez no mesmo assunto**. ⇒ hoje a forma fora da peça é **ZERO** na fixtura, que é o que
o rasterizador deixa, e o gate leva **três** controlos: o degrau na borda · o branco a dois texels
(a prova de que ele reproduz o defeito) · e o cartão arted, onde o degrau é uma lei CERTA.

⏳ **ABERTO e declarado:** a cobertura que o G-buffer entrega é **BINÁRIA** (`255` e depois `0`, sem
um valor pelo meio) — não há anti-serrilhado na rasterização da forma. ⇒ a silhueta de um recorte é
**dura**, e num zoom alto isso lê-se como escada. É outra grandeza e outra wave (MSAA ou super-
amostragem no `form_plane_for`), com custo por medir.

⚠️ **E o vestir tem memória:** depois de o artista pintar um traço na tela, o sprite deixa de estar
vazio ⇒ o bake seguinte não veste, `materia_da_forma` volta a `false`, e a silhueta passa a ser a da
ARTE. *É a resposta certa, e é a razão de o facto ser do gesto que assa e não da cena.*

**17 provas de mutação, todas a sangrar** — `docs/Render3d/ferramentas/mutacao_materia_da_forma_2026-09-21.sh`.

---

---

## ⛔ Recusas MEDIDAS

| o que | porquê | onde |
|---|---|---|
| ⛔ ~~a cura da mistura (§11.7-bis) como suficiente~~ | **REFUTADA pela foto seguinte**: ela endireita a MISTURA e não toca no que está FORA, onde a normal é degenerada e a lei cai no albedo branco | §11.7-ter |
| escolher «o primeiro vizinho coberto» no preenchimento | com cobertura binária isso é escolher pela ORDEM da varredura, e as duas redacções teriam de a partilhar para a paridade fechar | §11.7-ter |
| o anel de preenchimento entrar no ALFA | ele dá COR e nunca visibilidade — no alfa, a silhueta engordaria um texel a cada acendida | §11.7-ter |
| ⛔ ~~o branco fora da silhueta como halo «melhor»~~ | **REFUTADA pela foto do dono no dia seguinte**: a comparação era com o ALBEDO (branco) e não com a SAÍDA (o aceso, `183`) — `72` códigos de degrau, que é a orla | §11.7 |
| aplicar a cobertura na COR quando a matéria é a forma | ela já viaja no alfa; aplicá-la duas vezes puxa a borda para o branco do vestido | §11.7-bis |
| uma regra de vestir/recortar por TEXEL | num sprite com arte desenhada ela enche de branco a volta do desenho sempre que a malha for maior — *a pergunta é «este sprite tem arte?» e responde-se UMA vez* | §11.3 |
| ler o doc do `acende_faixa` (*«o alfa atravessa intacto»*) como *«o bake não escreve alfa»* | a promessa é verdadeira sobre a LEI, e o alfa nascia no VESTIR, a montante dela — a sonda mediu `53 252` de `65 536` texels vazios | §11.2 |
| deixar a `=53` com a tela BRANCA | sobre branco o vestir nunca arma ⇒ a cena mostraria a peça num cartão e a cura seria invisível a quem a smoka | §11.5 |
| medir a silhueta a virar com uma ESFERA | ela é invariante a toda pose — é o controlo dos gates vizinhos e a fixtura que não contém o fenómeno | §11.6 |
| virar o toro pelo `yaw` | ele assenta nesse plano ⇒ `0` de `65 536` texels a mexer, que se lê como uma lei que não chega ao pixel | §11.6 |
| uma paridade de placa com a matéria DESLIGADA só | as duas linhas novas do shader nunca eram percorridas — a cegueira do matcap, que custou um report | §11.4 |
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
| um *dolly* (afastar a câmera) para enquadrar o sprite | ele muda a CONVERGÊNCIA e não só o enquadramento — sob lente convergente não é exprimível como recorte | §10.1 |
| redimensionar o sprite para casar com a vista | o rectângulo e a arte do sprite são do ARTISTA | §10.1 |
| o aspecto do frustum sair do sub-rectângulo | a forma do frustum é da VISTA; o recorte só diz que pedaço dela este alvo desenha — trocá-los estica a peça | §10.1 |
| `use ph2d_mesh_render::Framing` na `ph2d-form-donation` | ela é a fronteira que o runtime atravessa SEM o módulo 3D, e o `use` traria `wgpu` + matcaps + `imageio` | §10.3 |
| recomputar o enquadramento por quadro em vez de o congelar | a forma passaria a seguir o OVERLAY 3D e a deslizar dentro do sprite quando o canvas 2D fizesse pan | §10.3 |
| pôr o giro no `Default` do componente | toda peça do app passaria a girar; o giro é da CENA, e há gate nas duas metades | §7.4 |
| abrir a `=53` com uma esfera LISA | raio constante ⇒ invariante à rotação ⇒ a cena ensinaria que a rota B não faz nada | §7.4 |
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
| deixar o VISOR não pintar quando a matéria está vazia | a promessa do módulo é *o visor pinta a matéria que o bake vai ACENDER* — não pintar mostraria o barro de sempre, que é outra coisa do que o bake vai fazer | §9.1 |
| usar a cobertura do G-buffer também no visor | ele corre **por quadro** e não rasteriza forma nenhuma; ali a peça é o barro inteiro | §9.1 |
| a cobertura fora do plano cair na última lida | um `base` maior que o G-buffer é um defeito de TAMANHO, e vesti-lo escondê-lo-ia com uma cauda de branco | §9.1 |
| carimbar o `Mesh3D` em TODO bake, mesmo re-assando | apagaria o giro que o artista acabou de pôr — o argumento que o `lei_ao_assar` e o slot já fazem | §9.2 |
| uma âncora de gate feita do TÍTULO de uma cena | ela reprova no dia da tradução, e o defeito que o gate existe para apanhar continua vivo — a âncora é o **número** (`=53 `) | §8.6 |

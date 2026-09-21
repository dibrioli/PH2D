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

## ⛔ Recusas MEDIDAS

| o que | porquê | onde |
|---|---|---|
| chamar a porta `form_plane` por quadro | `4` objectos a `512²` contra `26` — o readback é `31×` a rasterização | §1.2 |
| baixar a resolução do G-buffer para poupar relógio | não compra rasterização (ela é plana no lado) **nem** acendida (o passe despacha sobre os pixels do SPRITE) | §1.1 + §1.2-bis + §1.3 |
| medir a rasterização com uma esfera leve | o custo é de VÉRTICES: `33×` de malha vale `1,6×` de tempo, e a fixtura não continha a grandeza | §1.4 |
| dividir o orçamento pelo custo de RASTERIZAR | é metade da corrente: a conta dá `126` objectos e a corrente inteira dá `26` | §1.2-bis |
| apertar o controlo de vácuo até a fixtura PRETA o disparar | ela não é um vácuo: tira a COR e não a FORMA, e lê `246` de excursão contra `188` da boa — apertar mediria outra grandeza | §5.4 |
| dar folga ao gate da igualdade `f32` | a rota residente não é uma aproximação: uma barra ali deixa passar uma 2.ª redacção do despacho | §5.3 |

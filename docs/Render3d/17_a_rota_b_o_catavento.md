# 17 — A ROTA B: o catavento

> **A 2.ª obra da fila** ([`15` §5](15_as_metas.md)): *um objecto 3D ao vivo dentro do canvas 2D —
> ele roda, e a luz acompanha.* A arquitectura foi desenhada em
> [`docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`](../3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md)
> e o que faltava era construí-la. Esta página começa onde a lei desta casa manda
> (`CLAUDE.md` §5.0): **medir o que a composição já dá, antes da 1.ª linha de produto.**

---

## §1 — A medição de §5.0, e o que ela decidiu

**Instrumento:** [`mede_o_que_a_composicao_ja_da_ao_catavento.rs`](../../crates/ph2d-mesh-render/tests/it/mede_o_que_a_composicao_ja_da_ao_catavento.rs)
(três blocos, `#[ignore]`, precisa de adaptador).

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && \
PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-mesh-render --release \
  --test it -- --ignored --nocapture --test-threads=1 catavento
```

⚠️ **`--test-threads=1` é load-bearing** e não estilo — ver §1.4.

### §1.1 — Bloco A: rasterizar a malha filha para o G-buffer, **por quadro**

Textura **reaproveitada** (o que um dirty-flag entrega) contra **alocada a cada quadro** (o que a
porta de assar faz hoje). Medido a `load 44` — ou seja, **um PISO**: a contenção só pode ter tornado
estes números maiores.

| malha | lado `128` | `256` | `512` | `1024` | objs/quadro a `512` |
|---|---|---|---|---|---|
| leve (`3 010` v) | `0,085` | `0,076` | `0,074` | `0,078` ms | **`225`** |
| **fábrica (`98 306` v)** | `0,131` | `0,116` | **`0,133`** | `0,124` ms | **`126`** |

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
a `512²`; manter o G-buffer na placa dá **`126`**.

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

⭐⭐ **E a pergunta do `02.2` DISSOLVE-SE, mas não pelo motivo que ela supunha.** Ela perguntava a
fracção *«onde a silhueta começa a serrilhar»* — e o Bloco A diz que **uma fracção não compra
relógio nenhum** (`0,133` a `512²` contra `0,131` a `128²`). ⇒ *pagar silhueta para poupar um tempo
que não existe é uma troca sem lado bom.*

⛔ **Mas ela reabre noutro recurso, e esse é exacto e não precisa de medição — a MEMÓRIA.** O
G-buffer é `RGBA16F` (8 B/texel) + `R16F` (2 B/texel):

| lado | por objecto | a `126` objectos |
|---|---|---|
| `1024` | `10,0 MiB` | `1 260 MiB` |
| `512` | `2,5 MiB` | `315 MiB` |
| `256` | `0,625 MiB` | `79 MiB` |

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

- **Qual o lado do G-buffer por objecto** — §1.3 diz que é uma conta de VRAM contra o rectângulo do
  sprite no ecrã. Falta a régua que o deriva.
- **O dirty-flag**: a §1.1 mede o custo de re-rasterizar sempre. Quanto ele poupa numa cena real
  depende de quantos objectos rodam por quadro, que é facto da cena e não do motor.
- **O tecto de VRAM da cena** não foi medido.

---

## ⛔ Recusas MEDIDAS

| o que | porquê | onde |
|---|---|---|
| chamar a porta `form_plane` por quadro | `4` objectos a `512²` contra `126` — o readback é `31×` a rasterização | §1.2 |
| baixar a resolução do G-buffer para poupar relógio | uma fracção não move o relógio (`0,133` contra `0,131 ms`) e custa silhueta | §1.1 + §1.3 |
| medir a rasterização com uma esfera leve | o custo é de VÉRTICES: `33×` de malha vale `1,6×` de tempo, e a fixtura não continha a grandeza | §1.4 |

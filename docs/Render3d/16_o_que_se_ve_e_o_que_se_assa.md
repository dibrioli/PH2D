# 16 — O QUE SE VÊ É O QUE SE ASSA: três reports, três causas, e nenhuma era a mesma

> **O dono, 2026-09-20/21, três vezes seguidas — cada uma DEPOIS de uma correcção enviada:**
> 1. *«o bake não é idêntico ao que se vê em 3d»* (foto)
> 2. *«Para mim nada mudou. Veja: a malha 3d parece ter mais luz indireta que a imagem do Bake.
>    Mas precisa ser idêntica.»* (foto)
> 3. *«nada ainda»* (foto) — e depois *«siga»*

⚠️ **Três reports com a mesma frase não são três vezes o mesmo defeito.** Eram **três causas
empilhadas**, cada uma escondida pela anterior, e cada correcção só tornava a seguinte visível. Esta
página é a ordem em que elas caíram, com o número de cada — para que a próxima leitura de *«ainda
não está igual»* comece por medir em vez de recomeçar.

---

## §1 — A partição: onde cada causa vivia

Há **três** respostas independentes a *«de que cor sai este pixel?»*, e as três tinham de coincidir:

| # | a pergunta | o visor respondia | o bake respondia |
|---|---|---|---|
| 1 | **com que LUZ?** (o modo da vista) | um **matcap** (a luz do OLHO) | a lei que assa |
| 2 | **com que LEI?** (o motor da acendida) | OpenPBR | a lei da **TINTA** do Painter |
| 3 | **de que MATÉRIA?** (o albedo) | um barro cravado no shader | **os pixels da sprite** |

⭐ **Cada uma foi fechada por uma wave, e a ordem não podia ser outra:** com o visor num matcap
(causa 1) nenhuma medição de lei (causa 2) diz nada, e com duas leis diferentes a matéria (causa 3)
é ruído ao lado delas.

---

## §2 — Causa 1: o app ABRIA no modo errado

O `DEFAULT_LIGHTING` era `Matcap(0)` — o índice do SculptGL, posto ali por uma ordem do dono de
**2026-08-09** (*«SculptGL: só tem um tipo; busque e coloque como o padrão do app»*).

**Medido:** `0,347` por canal de desvio contra a lei que assa — **`1 615×`** o resíduo do modo `Pbr`
e `177×` a barra de meio código de oito bits.

⛔⛔ **E o defeito de MÉTODO que o revelou está registado:** o gate de paridade que eu tinha escrito
**cravava** `Lighting::Pbr` na vista de teste. Ele media a lei certa sobre um modo que **o app não
abre** — *um gate que escolhe o próprio modo mede um programa que ninguém corre*. A cura é o gate
`o_que_o_app_mostra_de_fabrica_e_a_lei_que_assa`, que entra pelo `DEFAULT_LIGHTING`.

⚠️ **A ordem anterior do dono fica ESCRITA em vez de apagada** (no doc-comment do
`DEFAULT_LIGHTING`): ela respondia a *qual matcap abre*, e esta troca responde a outra pergunta, que
ele levantou depois e duas vezes. Uma palavra dele devolve o matcap ao sítio.

---

## §3 — Causa 2: o BAKE corria a lei da tinta

O valor de fábrica da `ph2d_form_donation::lei_da_luz::Lei` era `Tinta` — o `ImpastoLightPass` do
Painter — e não `Forma` (o OpenPBR). ⇒ o visor e o bake corriam **motores diferentes**.

**Medido:** `0,055042` de desvio médio, **`28×`** a barra de meio código.

⛔⛔ **E eu já tinha esse número e tinha-o explicado ao contrário.** A primeira redacção da sonda
comparava o visor contra o harness que estava à mão (`super::compare`), cujo nome — *«as duas luzes
sobre a mesma forma»* — **não diz QUAL**; e eu li a tabela dele como *«as duas implementações da
mesma lei divergem `0,055`»* quando o que ela media eram **duas leis diferentes**. *Um oráculo
escolhido pelo harness que estava à mão mede o que aquele harness mede.*

---

## §4 — Causa 3: a MATÉRIA, e é ela que valia `61×`

Com a luz e a lei iguais dos dois lados, sobrava a superfície: o visor pintava o `CLAY` cravado no
shader (`0,74 · 0,70 · 0,66`) e o bake pintava **a arte da sprite**.

**Medido** (mesma forma, mesma luz, os enquadramentos do produto):

| o que difere | desvio médio, em códigos de 8 bits |
|---|---|
| lei + enquadramento + oclusão de tela | `0,52` |
| **só o albedo** | **`31,68`** |

⇒ **`61×`**. ⭐ *Uma diferença de MATÉRIA não se corrige com luz* — e é por isso que as duas waves
anteriores, ambas correctas, não podiam fechar o report.

### §4.1 — A correspondência é de ECRÃ, e é exacta

O bake **rasteriza a malha com a MESMA câmera, no tamanho da sprite** ⇒ o texel `(i,j)` da sprite é
o ponto da malha que aquela rasterização vê ali. O visor reconstrói esse `uv` do próprio fragmento:

* o `y` é o mesmo — o `fov_y` é preservado;
* o `x` escala pela **razão dos aspectos** (`aspect_vista / aspect_sprite`).

⚠️⚠️ **O `@builtin(position)` chega em coordenadas do ALVO e não da ÁREA** — o `set_viewport` move o
rasterizador, não a aritmética do shader —, logo a origem da área tem de viajar no uniform
(`cam.viewport.zw`). ⛔ **Duas mutações SOBREVIVERAM à primeira redacção do gate** (a origem e a
razão dos aspectos), porque a fixtura tinha a vista **quadrada, do tamanho da sprite e na origem**:
os três números eram o elemento neutro ao mesmo tempo. ⇒
`a_projeccao_da_fonte_sobrevive_ao_enquadramento` usa sprite padronizada `1024²`, alvo `1400×800` e
uma área deslocada — e as quatro sangram.

### §4.1-bis — ⚠️ A matéria fica presa ao ECRÃ enquanto se orbita, e isso é FIEL

Ao orbitar, a arte da sprite **não roda com a malha** — ela continua colada ao quadro. ⛔ Isto lê-se
como um defeito e não é: o bake produz um **sprite 2D aceso por uma forma 3D**, logo o albedo dele é,
por construção, alinhado ao ecrã da câmera que assa. Como essa câmera é a do escultor, o visor está
a mostrar, em cada instante, *exactamente o que aquele `Shift+B` gravaria agora*.

⚠️ **E é por isso que os matcaps continuam a existir:** eles são a luz do OLHO e continuam a ser o
que melhor lê FORMA enquanto se esculpe. O eixo é *«estou a esculpir»* contra *«estou a julgar o que
vou gravar»*, e o modo de fábrica responde ao segundo desde que o dono o reportou duas vezes.

### §4.2 — A textura é `Rgba8Unorm` e **não** `Rgba8UnormSrgb`

⛔ O matcap ao lado é `Srgb` e copiar a convenção dele teria sido um defeito **sem erro nenhum**: a
`ph2d_form_pbr::imagem` lê os bytes da sprite como `px/255.0` (LINEAR), logo uma textura `Srgb`
desfaria a curva uma vez a mais e o visor sairia mais claro que a sprite.

---

## §5 — A lei que ficou, e porque ela tem UMA porta

⛔⛔ **Uma sprite JÁ ASSADA não devolve a matéria dela pela porta de leitura.** Depois do primeiro
bake os pixels são `base × luz`; lê-los como fonte faria a acendida seguinte acender o que já está
aceso, e **o objecto escureceria a cada gesto**.

Essa lei tinha **um** leitor (o gesto) e nenhuma régua. Com o visor a lê-la também, ela mudou-se
para [`ph2d_app_sculpt3d::albedo::materia_para`] — a tabela dos assados primeiro, o device depois —
e ganhou o gate `re_assar_nao_le_a_tela_de_volta`, cuja régua é **o fecho não ser chamado**.

⚠️ **E a assinatura dela perdeu o mundo e o renderizador de propósito:** com eles lá dentro a função
só era testável com um adapter, e o que ela afirma não tem um pixel dentro.

### §5.1 — O recurso que escolheu a memória

Ler os pixels de uma sprite `Individual` é um **`readback`** — uma volta completa à placa mais uma
cópia de `w × h × 4` bytes (`4 MiB` numa sprite de `1024²`). A **60 Hz** isso é um estol por quadro
para responder a uma pergunta cuja resposta só muda quando o artista **escolhe outro objecto**.

⇒ a memória é `(sprite, já-assada?)`, e as duas metades são load-bearing: assar TROCA a matéria.

⛔ **DIVERGÊNCIA DECLARADA:** pintar a sprite em 2D com o visor aberto **não** re-lê a matéria — não
há hoje contador de revisão numa textura individual, e inventar um aqui seria a segunda resposta a
*«esta imagem mudou?»*. A cura do artista é escolher outro objecto e voltar; a cura deste código é
aquele contador, **na crate que é dona da textura**.

---

## §6 — O que fica ABERTO

* ⏳ o contador de revisão da textura individual (a divergência do §5.1) — `ph2d-render`;
* ⏳ o material **por PIXEL** (a coluna B2 do [`15`](15_as_metas.md)): hoje a matéria é o albedo e
  mais nada — a rugosidade, o metal e a normal continuam uma cor por objecto;
* ⏳ o `Lighting::Flat` não entra no bake nem na doação de forma — ele é VISTA.

---

## ⛔ Recusas MEDIDAS

| o que foi tentado | porque NÃO ficou |
|---|---|
| fechar o report ajustando a LUZ (3.ª vez) | o albedo vale `61×` tudo o resto somado (§4) |
| a textura do albedo em `Rgba8UnormSrgb` | a leitura do bake é LINEAR ⇒ o visor sairia mais claro, sem erro nenhum (§4.2) |
| pôr a decisão da memória dentro da `sincroniza` | ela precisaria de um device, nasceria `#[ignore]` e o CI nunca a correria (§5) |
| re-ler a matéria por quadro | `4 MiB` de `readback` por quadro para uma resposta que muda num gesto (§5.1) |

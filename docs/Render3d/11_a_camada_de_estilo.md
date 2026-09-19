# 11 — A camada de ESTILO: os botões para mentir de propósito

> **Ordem do dono, 2026-09-19:** *«8 e depois do smoke o 7»* — o ingrediente **`8`** do
> [`01`](01_o_alvo_decomposto.md) antes do **`7`**, que é o pós.

A [`W8`](03_o_plano.md) do plano. O [`01`](01_o_alvo_decomposto.md) §1 mede a tese do alvo e ela é
esta: *«o que faz aquele jogo ser bonito não é um shader — é o pipeline ser fisicamente honesto para
a direcção de arte poder depois mentir de propósito»*. As cinco waves anteriores construíram a
honestidade; esta constrói os botões para a trair.

```text
material + luz  →  [ESTILO]  →  olhar (exposição + vista)  →  sRGB
 (a física)        (a mentira)    (a ph2d-view-transform)
```

## §1 — Os quatro botões, e porque são estes quatro

| botão | o que ele mente | o que o olho lê |
|---|---|---|
| **Rim** (contorno) | acrescenta luz onde a superfície **foge** do olhar, `(1 − \|N·V\|)^largura` | a peça **descola do fundo** — é o que separa um personagem de um cenário |
| **Curvature** (aresta / cova) | tinge pelo que a peça **É**: aresta contra vinco | o aspecto pintado à mão, **sem ninguém pintar um mapa** |
| **Zones** (a grade) | uma tinta para o que é escuro, outra para o que é claro | a grade de cor de um filme — sombras frias, luzes quentes |
| **Indirect Saturation** | satura (ou lava) **só** a luz que ricocheteia | o sangramento de cor da [`W5`](08_a_luz_indirecta.md) deixa de ser tímido, sem tocar nas lâmpadas |

⛔ **E um quinto ficou de fora com o mecanismo escrito: o CONTORNO desenhado** (a tinta na
silhueta). Ele **não é uma lei por ponto** — precisa dos vizinhos no ecrã, logo é um **passe**, e
pô-lo na crate da lei obrigaria-a a receber um G-buffer. *Ela deixaria de ser a lei que os dois
motores partilham, que é a única razão de ela existir.* Fica nomeado na [`W8`](03_o_plano.md).

## §2 — ⭐⭐⭐ A omissão é a identidade, e por CONSTRUÇÃO

Toda tinta é uma **cor** cujo valor de fábrica é o **branco**; toda soma tem um peso cujo valor de
fábrica é **zero**. ⇒ um quadro que não pediu estilo nenhum sai **byte a byte** o de antes, **sem um
ramo pelo caminho** — e é isso que faz as paridades já pagas ([`08` §12](08_a_luz_indirecta.md), a
`100,000 %`) ficarem de pé sem serem re-medidas.

Medido nos dois motores: `0` píxeis fora, pior `0`.

### ⛔⛔ E uma premissa minha caiu aqui, por DUAS mutações SOBREVIVENTES

O cabeçalho da crate dizia que a forma ingénua `a·(1−w) + b·w` **não serve**, porque `(1−w) + w` não
seria `1` para todo `w`. Duas mutações que a instalavam nas tintas passaram os gates todos, e a
varredura explicou porquê:

> `(1−w) + w` dá **`1,0` exactamente** em `2 044 824` amostras de `f32` em `[0, 1]` e em `200 000`
> acima — como tem de dar, porque o erro daquela soma é no máximo **meia ULP** de `1` e o desempate
> é para o par, que é o próprio `1,0`.

⭐⭐⭐ **O perigo real é o que a mutação que SANGROU nomeia: reconstruir `b` a partir de `a + (b − a)`
quando `a ≠ b`.** Nas tintas os dois extremos são iguais no ponto de fábrica (as duas brancas), logo
o parêntesis é **zero** e as duas redacções são exactas; na saturação eles são a **luminância** e o
**rgb**, que são diferentes, e ali a reconstrução perde bits por cancelamento — `s = 1` deixaria de
devolver a entrada.

⇒ a lei que fica: **multiplicar o valor que se quer de volta, nunca reconstruí-lo por diferença.**

## §3 — ⭐⭐⭐ O SINAL da curvatura já existia, e era deitado fora uma linha antes

`H = ∇²f / 2` é **positivo numa bossa e negativo numa cova**. O consumidor que a estreou — a
subsuperfície MACIÇA da [`10`](10_a_luz_que_atravessa_a_peca.md) — pede um **comprimento** (a
referência do MaterialX estima-o por `length(fwidth(N))`, que é `≥ 0` por construção), e por isso o
porte escrevia `abs(…)` **dentro** da porta.

⇒ *ali era a resposta certa, e a perda ficava dentro da função em vez de no chamador.* Hoje a
subsuperfície toma o módulo do lado dela — a imagem que ela pinta é **byte a byte a mesma**, porque
tomar o módulo antes ou depois de guardar dá o mesmo `f32` — e a tinta por curvatura lê o sinal, que
é **a diferença entre uma aresta e um vinco**: entre um contorno e uma sujidade.

**Medido na fixtura da wave** (`128×96`, câmera de omissão): `3 631` píxeis acertados, `584` deles
com curvatura **negativa** (`16 %`), `min = −3,361` — que é `−1/0,30`, o raio da cratera **ao
terceiro decimal** —, `p50 = +1,819` (a bola, `1/0,55`) e `max = +11,7` (o lábio com filete).

### ⚠️ E a curvatura é adimensional antes de chegar ao estilo

Ela mede-se em `1/comprimento`. Um botão calibrado nisso **mudaria de sentido ao escalar a peça** —
a mesma quina daria outra tinta numa peça de `0,3` e numa de `3`. Multiplicada pelo raio da bola que
envolve a peça, o que o estilo lê é *«quantas vezes esta zona é mais curva do que a peça inteira»*.
⭐ Numa esfera ela vale **`1`** em todo o lado; num filete de `1/10` do raio, **`10`**; numa face
plana, **`0`**.

## §4 — ⭐⭐⭐ A lição do §24, aplicada ANTES de escrever a primeira linha

O [§24 da `10`](10_a_luz_que_atravessa_a_peca.md) é a auditoria mais cara que este módulo pagou:
duas waves seguidas foram entregues ao dono com tabela, gates e prova de mutação, e viviam **num ramo
que o produto não corre**. Esta wave foi escrita contra isso:

1. **O estilo entra na ASSINATURA.** `ph2d_field_render::Presentation { look, style, piece_radius }`
   substitui o `Look` solto no `shade_render` **e** no `gpu_frame::paint` — os dois motores recebem o
   **mesmo tipo**, e esquecê-lo é **erro de compilação**. ⛔ Uma função-irmã *«com estilo»* seria a
   segunda porta pela qual o defeito volta.
2. **Uma apresentação, montada UMA vez e ANTES do ramo do dispositivo.** Com gate **estrutural**, que
   corre sem placa e sem cena.
3. **A curvatura é medida quando ALGUÉM a lê, e são DUAS portas somadas** — o material ou o estilo.
   ⛔ Perguntar só ao material faria o artista mexer na tinta de aresta e a peça não mudar um pixel:
   *a grandeza que o botão escolhe nunca teria sido medida.*

## §5 — ⛔⛔⛔ A paridade achou uma divergência PRÉ-EXISTENTE que só um consumidor sensível revela

O gate de placa reprovou à primeira, com `3`–`5` bytes em oito píxeis. A **atribuição botão a botão**
diz que a lei do estilo não tem culpa nenhuma:

| o que se liga | píxeis acima de `1` | pior |
|---|---:|---:|
| a fábrica | `0` | **`0`** (byte-idêntico) |
| só o contorno | `0` | `1` |
| só as zonas e a saturação | `0` | `1` |
| a tinta por curvatura, `nitidez 0,2` | `0` | `1` |
| a tinta por curvatura, `nitidez 1` | `163` | `7` |
| a tinta por curvatura, `nitidez 2` | `168` | `13` |
| a tinta por curvatura, `nitidez 8` | `168` | **`45`** |

⭐⭐⭐ **A contagem SATURA em ~`165` e a magnitude cresce LINEARMENTE com a nitidez** — a assinatura de
*uma diferença pequena na CURVATURA, amplificada pelo ganho*. Os dois motores medem-na por caminhos
diferentes (a fita achatada da `ph2d-field-eval` contra o `field()` do WGSL), e **a divergência já lá
estava**: o único consumidor que ela tinha passa a curvatura por uma **tabela pré-integrada com
piso** (`max(κ, 0,01)`), que a **satura**. *Esta tinta é o primeiro consumidor LINEAR nela, e por
isso o primeiro instrumento que a vê.*

⏳ **Dívida NOMEADA, e não é desta wave:** falta o instrumento que mede a **curvatura** nos dois
motores, e não o pixel. ⛔ **A barra não foi afrouxada para engolir os `13` bytes** — a tinta é medida
onde ela não amplifica, e quem fechar aquela dívida sobe a nitidez desta linha.

## §6 — ⛔ O MATCAP fica de fora, por decisão

Ele é *a luz do OLHO* — um auxiliar de modelação que lê **forma** —, e o estilo é direcção de arte
sobre um pipeline fisicamente honesto. **Tingir o viewport de modelagem com a grade do filme faria o
artista medir a peça através de uma mentira.**

⇒ *o olhar governa os dois* (ele é a gestão de cor da cena, [`05` §1](05_o_modo_render_do_modelador.md));
*o estilo governa o Render.* Há um **censo** com a excepção nomeada — e ela apareceu porque o gate a
acusou: a 1.ª redacção exigia zero passagens do olhar por fora da apresentação e reprovou sobre
produto correcto.

## §7 — Onde o estilo VIVE, e a dívida que ele partilha

Ele é **estado de VISTA**, ao lado do olhar (`field3d_view::View::style`): sobrevive a fechar o
painel, **não** entra no undo nem no arquivo. ⚠️ O gate do `View::of` disparou ao vê-lo nascer, que é
o que o `CLAUDE.md` §5.1 promete, e a resposta está escrita no campo.

⏳ **Ele passa a DOCUMENTO no mesmo dia que o olhar** — quando houver uma saída de render que os
grave ([`04` §4](04_a_remedicao_contra_a_arvore.md)). *A dívida é uma e é partilhada; separá-las agora
poria metade da apresentação no ficheiro e metade fora.*

## §8 — O que o artista vê

Dez fileiras na secção **Style**, no fim do painel do modelador, **só no modo Render**: cinco cores
(contorno · aresta · cova · sombras · luzes) e cinco números (força e largura do contorno · nitidez
da curvatura · pivô das zonas · saturação da indirecta).

⭐ **A tabela do painel é DERIVADA da arrumação do uniforme** (`ph2d_style::wgsl::pack`): o
`Param::Style(n)` carrega a **posição**, e uma escrita é *desempacota, escreve a posição, empacota*.
⛔ Uma numeração própria seria a segunda resposta à mesma tabela.

⚠️ **Cada tecto nomeia o recurso** (`CLAUDE.md` §0.0), e os três que ainda não têm tabela ficam
**nomeados como dívida da [`W9`](03_o_plano.md)**, que é a wave da medição.

**Smoke:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=35 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **O enquadramento da cena foi FOTOGRAFADO** (`docs/Components/ferramentas/fotografa_cena.sh`) e a
foto apanhou três coisas que a suíte não vê — as covas viradas de lado, a barra a sair do ecrã, e a
armadilha de fotografar com o `$HOME` do dono (que fotografa a **bancada** dele e não a cena). Ver o
doc-comment da `cena_35`.

## ⛔ Recusas MEDIDAS

| o que foi recusado | porquê, com o número |
|---|---|
| a forma ingénua `a·(1−w) + b·w` ser «errada» | **refutada**: ela é exacta em `2 044 824` amostras. O perigo é `a + (b − a)` com `a ≠ b` |
| o contorno desenhado (a tinta na silhueta) nesta crate | ele lê os VIZINHOS no ecrã ⇒ é um passe, e a crate deixaria de ser a lei partilhada |
| afrouxar a barra da paridade para os `13` bytes | a divergência é da CURVATURA e é pré-existente; a tinta mede-se onde não amplifica |
| o estilo no MATCAP | ele é a luz do OLHO; tingi-lo faria o artista medir a peça através de uma mentira |
| um `shade_render_com_estilo` ao lado do outro | é a segunda porta pela qual o defeito do §24 volta |

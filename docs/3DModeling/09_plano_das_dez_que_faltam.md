# ⭐ O PLANO DAS DEZ QUE FALTAM — a fila depois da integração de 06/09

> **Enio, 06/09:** *«Guarde o plano de implementação do restante pois depois da integração
> continuaremos»*.
>
> ⚠️ **O placar CONTA-SE**, nunca se copia: [doc 08 §7.6](08_formas_por_formula.md) soma as três
> tabelas com os riscados fora. Este plano é a **ordem** e o **preço** de cada uma, não o número.

---

## 0-bis. ⛔ O ITEM QUE VEM ANTES DA FILA

**A revisão da performance da superfórmula**, adiada pelo Enio para depois da integração:
*«não houve melhora significativa»*. As hipóteses e o que medir antes de tocar em código estão no
[handoff de 06/09 §6-bis](handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-09-06.md). ⚠️ *Uma
melhoria que o dono não vê ainda não chegou ao produto* — e a régua que discorda dele é a suspeita
número um.

---

## 0. Onde a fila está hoje

| de onde | faltam | quais |
|---|---:|---|
| catálogo **2D** | **0** | ~~Polygon(N)~~ · ~~Triangle~~ · ~~Bezier~~ · ~~Parabola~~ · ~~Circle Wave~~ |
| catálogo **3D** | **1** | Plane · ~~Death Star~~ · ~~Vesica Segment~~ (W138) |
| **famílias** fora de catálogo | **0** | ~~Nó de toro~~ · ~~Rosca / knurling~~ |
| **total** | **3** | |

✅ **O LOTE 9 FECHOU em 06/09** — o *Triangle* na W131 ([doc 06 §132](06_resultados_cena_e_gizmo.md)) e
o *Polygon(N)* na W132 ([§133](06_resultados_cena_e_gizmo.md)).

⛔ **Fora da conta, de propósito:** os **dois modificadores** (grade hexagonal, metabolas) e os
**fractais** — este último é wave com medição própria, porque o custo é por **iteração** e não por
nó, e isso muda o preço do quadro.

---

## 1. A ORDEM proposta, e por quê

### ✅ Lote 9 — **Polygon(N) e Triangle** — FECHADO em 06/09

> ⚠️ **A pergunta de desenho que este lote nomeava tinha resposta, e era «metade»** — e vale relê-la
> antes do lote seguinte, porque o molde repete-se. O plano perguntava *«onde vivem os vértices? um
> `Vec` dentro do `Primitive` mexe no `PROJECT_SCHEMA`»*: **não mexe** (o `Extrude` carrega um
> `Profile` com `Vec<Vec<[f32;2]>>` desde sempre, e o `Primitive` é `serde`). E perguntava *«a
> composição já o exprime?»*: a **geometria** sim, inteira — o que faltava era a **AUTORIA**, que é o
> que decide se os pontos podem ter linha no painel. Mecanismo: [doc 06 §133.1](06_resultados_cena_e_gizmo.md).
>
> ⛔⛔ **E a wave achou dois defeitos que não eram dela:** a nota do `sd_extrude` prometia uma
> *«abertura morfológica»* que a medição refutou (o filete é do **ARO**), e o
> `fillet_inflates(Extrude) == false` **nunca tinha sido medido** — o censo reprovou a `1,0216` com o
> chanfro sozinho sobre uma quina que não é recta. Os dois estão curados com a tabela ao lado
> ([§133.2](06_resultados_cena_e_gizmo.md) e [§133.6](06_resultados_cena_e_gizmo.md)).

<details><summary>O que este lote dizia antes de ser feito</summary>


O polígono **irregular** e o triângulo **escaleno**. Hoje só se chega lá pelo editor vetorial, e
desenhar custa **por segmento**.

- **Mecanismo:** a distância a um polígono convexo é `max` de meias-fatias — 1-Lipschitz por
  construção, como o prisma da W101. Para **não-convexo** a fórmula do Quílez usa o par
  `(distância ao segmento, winding)`, e o `min` de segmentos com o sinal do winding **é exacto**.
- ⚠️ **A pergunta de desenho que decide a wave:** *onde vivem os vértices?* Um `Vec<[f32;2]>` dentro
  do `Primitive` é um campo de tamanho variável — e o `Primitive` viaja **posicionalmente** num
  `ComponentBlob`. ⛔ Isso mexe no `PROJECT_SCHEMA` e possivelmente na forma do blob. **Meça antes
  de escolher**: o `Profile` do `Extrude` já resolve exactamente isto, e a resposta pode ser *«um
  polígono É um perfil de 3..N pontos, e o que falta é o BOTÃO»* (§5.0 do `CLAUDE.md`: meça se a
  composição já o exprime).
- **Cercas a medir:** `MAX_POLYGON_VERTICES` (o preço por segmento é linear e já está medido para o
  `Extrude` — `1,27×` com 6 lados, `134×` com 192).

</details>

⇒ **O que shipou:** `MAX_POLYGON_VERTICES = 27`, e ⛔ **o recurso NÃO é o preço** — é a família de
linhas do painel (`2N + 10 ≤ MAX_ROWS = 64`), medida na cena e gateada nas duas pontas. Um teto de
preço aqui poria o caminho lento (desenhar, `2,6×`–`3,1×`) a mandar no rápido.
⏳ **Fica aberto e nomeado:** o filete do polígono é do **aro**, então a quina que o artista digita
fica viva — e a cura não é afinação (um polígono côncavo não é uma intersecção de semiplanos).

### Lote 10 — **Nó de toro** ✅ *(FECHADO em 07/09)* **e Rosca**

✅ **O NÓ DE TORO SHIPOU na W134** ([doc 06 §135](06_resultados_cena_e_gizmo.md)) — e ⛔ **três das
previsões deste plano estavam erradas**, o que vale reler antes da Rosca:

| o plano dizia | o que a implementação mediu |
|---|---|
| *«a volta mais próxima sai de um `round()`»* | ⛔ **não** — são `p` fios a cortar cada plano meridiano, e a resposta é um **`min` sobre `p` ramos**, a forma do polígono da W132 |
| *«esperar a mesma lição da superfórmula: um `m` fraccionário racha a peça»* | ⛔ a costura **não existe**: o conjunto `{ψ_n}` é invariante a `φ → φ − 2π`. E `p` não pode ser fraccionário — ele é a contagem de ramos |
| *«o minorante sai de dividir pelo gradiente máximo»* | ⛔⛔ **isso ENGORDA a peça**: `(minorante) − corda` desloca a superfície para fora, e a `(2,3)` saía um toro maciço. A folga tem de multiplicar o campo **inteiro**, cujo zero não se mexe |

✅ **E A ROSCA SHIPOU na W135** ([doc 06 §136](06_resultados_cena_e_gizmo.md)) — **o lote 10 fechou**.
⛔ **Três previsões deste plano estavam erradas outra vez**, e vale relê-las:

| o plano dizia | o que a implementação mediu |
|---|---|
| *«a hélice VARRIDA num cilindro»* | ⛔ **não é a hélice** — é um **perfil** varrido por movimento de parafuso. O `sd_helix` mede a distância a uma CURVA; uma rosca é uma face, e é isso que faz o factor da recta tangente fechar em forma fechada (`k = 1/√(1 + β²n_w²)`) em vez de pedir a correcção de curvatura que o nó pagou |
| *«falta a intersecção com o cilindro»* | ⚠️ é uma **UNIÃO**, não uma intersecção — o filete assenta sobre o núcleo. ⛔ E foi por eu escrever a cunha do filete **sem fundo** que o campo lia `‖∇f‖ = 2,4562` dentro da peça, onde nenhuma régua de forma olha |
| *«medir o quadro, não a amostra»* | ✅ certo, **e por outro mecanismo**: aqui `starts` **não** acrescenta um ramo à árvore (ao contrário do `p` do nó) — ele engorda o divisor, e quem paga é a **marcha** |

⏳ **FICA ABERTO e nomeado:** a `sd_helix` continua a usar a corda crua e a **engordar o tubo `1/c`**
(invisível só porque numa mola típica `c = 0,992`). ⚠️ A ferramenta para a curar está agora escrita
**duas** vezes (W134 e W135) e a conta dela, para uma curva, é `hypot(dr, dz·sin β)` — que é
**exactamente 1-Lipschitz**, e portanto estritamente melhor que o `c` de hoje. *Ela não entrou na
W135 porque mexer numa forma que já shipou pede a sua própria régua de antes/depois.*

### Lote 11 — **Bezier, Parabola e Circle Wave** ✅ *(FECHADO em 07/09)*

✅ **SHIPOU na W136** ([doc 06 §137](06_resultados_cena_e_gizmo.md)) — e com **DUAS** formas
construídas, não três. ⛔ **Três previsões deste plano estavam erradas outra vez:**

| o plano dizia | o que a implementação mediu |
|---|---|
| *«a cúbica tem TRÊS ramos, e o caminho é o `min`/`max` sobre os três»* | ⛔ são **DOIS** (o sinal do discriminante escolhe entre Cardano e Viète), e eles **não se sobrepõem** — um `min` daria o `NaN` do outro. A saída é o `Tree::compare`, e ele é contínuo porque em `h = 0` a raiz é **dupla** |
| *«Parabola: idem, uma cúbica mais simples»* | ⭐ **ela É a Bezier**, medido a `5,5e-17` — duas portas da paleta, uma primitiva. A fila cai `3` com `2` formas |
| *«Circle Wave: o minorante já tem a lei escrita»* | ⛔⛔ tinha metade. Foram **QUATRO** construções e **três recusas medidas** — a lei escrita (divisor constante) lê `‖∇f‖ = 2,46`, e o divisor local, que curaria o filete, lê **`2 156`** |

⭐ **E o preço:** o plano mandava medi-lo antes de prometer, e ele deu razão à lei da W128 — a Bezier
custa `6,8×` uma esfera **por amostra** e `1,0×` **por quadro**.

### Lote 12 — **Death Star e Vesica Segment** ⭐

✅ **SHIPOU na W138** ([doc 06 §139](06_resultados_cena_e_gizmo.md)) — e com **ZERO primitivas
novas**. ⛔ **A premissa deste lote foi medida e está METADE errada:**

| o plano dizia | o que a medição deu |
|---|---|
| *«a nossa subtracção não dá a distância exacta na cratera»* | verdade para a **cratera** (`0,5495` do que a distância é) e **falsa** para a **lente** (`0,9674`, exacta a menos da amostragem do oráculo) |
| *«são formas pequenas»* (⇒ construam-se) | o §5.0 manda perguntar se a composição já as exprime: ela exprime, com `‖∇f‖ = **1,000**` nas duas — a marcha nunca atravessa |
| *«o mecanismo é o mesmo dos `plate_joint`»* | o mecanismo que faltava não era um operador: era **uma entrada de catálogo que devolve uma ÁRVORE** (`Make::Composed`), sobre portas que já existiam (`add_leaf` + `wrap_in_op`) |

⭐⭐ **E a composição entrega o que uma primitiva não entregaria:** a cratera continua a ser uma
esfera na Hierarquia — move-se, redimensiona-se, e podem pôr-se **três**. É isso que a cena `=31`
mostra na terceira peça.

### ⏳ Fora de lote — **o Plane**

⛔ **Não é uma forma a construir: é a bola de recorte admitir uma peça INFINITA.** Hoje toda peça
tem `bounding_radius`, e o traçado, o gizmo, a exportação e o recorte por região dependem dele.
⚠️ *É maquinaria, e a wave dela começa por medir o que se parte quando o raio é `∞`* — não por
escrever `sd_plane`.

---

## 2. ⚠️ AS LEIS QUE ESTA JORNADA PAGOU — leia antes de pegar qualquer item

| lei | onde ela foi paga |
|---|---|
| **O módulo nunca precisou da distância EXACTA — precisa de um MINORANTE** | doc 06 §124 |
| **Um divisor sai em forma fechada quando a medida é homogénea de grau 1** — `∇g` é constante ao longo de cada raio ⇒ o máximo na superfície **é** o global | §128.1, §129.1 |
| ⛔ **Um máximo AMOSTRADO que vira limite de segurança erra sempre PARA BAIXO** — e a variável da varredura é parte da correcção (`16,3 % → 0,0000 %`) | §129.2 |
| ⛔⛔ **O divisor é do FORMATO e a árvore é reconstruída por LADRILHO** — meça o **quadro**, com um **contador**, e com um DESENHO na cena (que é o que liga a especialização) | §130.1 |
| ⭐ **Os expoentes `1` e `2` são exactos sem transcendental**, e `n2 = n3 = 2` faz a curva ser um círculo para qualquer `m` | §130.2 |
| ⛔ **Uma escrita que deixa a peça inválida apaga a CENA INTEIRA** — a porta repõe as invariantes, derivada da tabela de faixas | §127 |
| ⛔ **Um `m` fraccionário não faz forma nova: faz uma peça rachada** — a costura do `atan2` | §129.4 |
| ⚠️ **A peça tem de ter o TAMANHO que o painel diz** — normalize a curva, senão um expoente muda a escala `8×` | §129.3 |
| ⚠️ **Uma forma nova nasce no sítio em que ela é ELA**, nunca no ponto neutro (que é o sósia de outra entrada) | §128, §129 |
| ⛔ **Um gate CERTO que barra a forma nova: a FORMA é que sai** | §126.4 (a escada) |
| ⚠️ **Toda linha de painel precisa de rótulo** — uma chave sem tradução pinta o identificador cru **e vaza por quadro** | §130.4 |

---

## 3. O que fazer ANTES de a próxima wave escrever uma linha de código

1. **Re-contar o placar** no [doc 08 §7.6](08_formas_por_formula.md) — esta lista já esteve inflada
   em quatro, e duas saíram por recusa medida.
2. **Perguntar se a composição já a exprime** (`CLAUDE.md` §5.0) — foi assim que a ferradura, o
   túnel, o X redondo e a cápsula desigual saíram da fila **sem uma linha escrita**.
3. **Ler as ⛔ Recusas MEDIDAS** do [doc 08](08_formas_por_formula.md) — o ovo e a escada já foram
   construídos e medidos até à recusa.
4. **Correr `probe_gielis` / `probe_superquadric`** como molde: toda forma nova por fórmula precisa
   do mesmo par — *o divisor contra a medição* e *as cercas contra o preço*.

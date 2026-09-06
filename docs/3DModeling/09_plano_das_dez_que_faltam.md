# ⭐ O PLANO DAS DEZ QUE FALTAM — a fila depois da integração de 06/09

> **Enio, 06/09:** *«Guarde o plano de implementação do restante pois depois da integração
> continuaremos»*.
>
> ⚠️ **O placar CONTA-SE**, nunca se copia: [doc 08 §7.6](08_formas_por_formula.md) soma as três
> tabelas com os riscados fora. Este plano é a **ordem** e o **preço** de cada uma, não o número.

---

## 0. Onde a fila está hoje

| de onde | faltam | quais |
|---|---:|---|
| catálogo **2D** | **5** | Polygon(N) · Triangle · Bezier · Parabola · Circle Wave |
| catálogo **3D** | **3** | Plane · Death Star · Vesica Segment |
| **famílias** fora de catálogo | **2** | Nó de toro · Rosca / knurling |
| **total** | **10** | |

⛔ **Fora da conta, de propósito:** os **dois modificadores** (grade hexagonal, metabolas) e os
**fractais** — este último é wave com medição própria, porque o custo é por **iteração** e não por
nó, e isso muda o preço do quadro.

---

## 1. A ORDEM proposta, e por quê

### Lote 9 — **Polygon(N) e Triangle** ⭐⭐ *(as que hoje obrigam a desenhar)*

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

### Lote 10 — **Nó de toro e Rosca** ⭐⭐ *(a família da hélice, que já está paga)*

- **Nó de toro `(p, q)`:** dois inteiros dão uma família inteira. ⚠️ **O mecanismo é o da W123/W124**
  — a volta mais próxima sai de um `round()`, e o minorante sai de dividir pelo gradiente máximo.
  A diferença é que aqui a curva-guia fecha sobre si própria em `p` voltas, o que **muda a costura**:
  esperar a mesma lição da superfórmula (*um `m` fraccionário racha a peça*).
- **Rosca / knurling:** a hélice **varrida** num cilindro — o parafuso a sério e o punho serrilhado.
  ⭐ O `sd_helix` já existe; o que falta é a **intersecção com o cilindro** e o perfil do filete.
- ⚠️ **Os dois herdam o custo da família:** a mola custou `11,9×` uma esfera na W124. **Medir o
  quadro, não a amostra** — foi exactamente esse o erro da W128 (doc 06 §130).

### Lote 11 — **Bezier, Parabola e Circle Wave** ⭐ *(as curvas com espessura)*

- **Bezier quadrático:** o Quílez tem a forma fechada (resolve uma cúbica). ⚠️ **A cúbica tem três
  ramos** e o `Tree` não tem `if` — o caminho é o `min`/`max` sobre os três, e **cada ramo é uma
  raiz cúbica**, que é `exp(ln/3)`. Preço a medir antes de prometer.
- **Parabola:** idem, uma cúbica mais simples.
- **Circle Wave:** a onda em anel — irmã directa da onda do `Document` (W123), e o minorante já tem
  a lei escrita (`lip = √(1+(a·ω)²)`).

### Lote 12 — **Death Star e Vesica Segment** ⭐

As duas são **composição com a distância certa no encontro**: a nossa subtracção dá a forma e **não**
dá a distância exacta na cratera. São formas pequenas e o mecanismo é o mesmo dos `plate_joint`.

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

# 35 — Accumulate: o estudo, e a comparação com o Blender (2026-08-12)

> **Ordem do Enio (2026-08-12):** *"Apenas Accumulate deve ser estudo e comparado com o blender."*
>
> Complementa o [doc 20](20_accumulate_na_mesma_pincelada.md), que desenhou a metade do **RELEVO**.
> Este documento mede a metade que **shipa** — a COR — e a compara com o Blender.
>
> ⚠️ **Tudo o que é número aqui saiu do PRODUTO**, pela sonda
> `crates/ph2d-tool-painter/src/tool/paint/accumulate_probe.rs`:
> `cargo test -p ph2d-tool-painter --release accumulate_probe -- --ignored --nocapture`
>
> ⚠️ **E tudo o que é afirmação sobre o BLENDER é sobre COMPORTAMENTO, nunca sobre código.** A
> referência vendorizada (`reference/blender-texture-paint/`) é **gitignored e não existe nesta
> máquina** — o que está escrito aqui sobre ele vem do comportamento documentado e dos nomes que o
> nosso próprio código já cita como referência de comportamento (`paint_stroke.cc`,
> `gimp_gegl_combine_mask_weird`), e está **marcado como tal** em cada parágrafo.

---

## 1. O veredito curto

**O checkbox Accumulate do PH2D está INERTE na configuração que o app abre.**

O default do pincel é `strength: 1.0` ([`spec_default.rs:27`](../../crates/ph2d-painter-brush/src/spec_default.rs)),
e a porta que decide se um traço rastreia a própria cobertura é

```rust
// stamp_route.rs — stroke_cover_wanted
!brush.accumulate && (brush.strength < 1.0 || brush.film_aa_wanted(…))
```

Com `strength = 1.0` e sem o AA do filme, **`stroke_cover_wanted` é falso nos dois estados do
checkbox** ⇒ os dois tomam a MESMA rota. Medido no perfil perpendicular inteiro, não só no eixo:

```
strength 1.0  n=1   accumulate off  d0=1.000 d2=1.000 d4=0.910 d6=0.294 d8=0.000
strength 1.0  n=1   accumulate ON   d0=1.000 d2=1.000 d4=0.910 d6=0.294 d8=0.000
strength 1.0  n=15  accumulate off  d0=1.000 d2=1.000 d4=0.996 d6=0.980 d8=0.000
strength 1.0  n=15  accumulate ON   d0=1.000 d2=1.000 d4=0.996 d6=0.980 d8=0.000
```

**Idêntico casa a casa.** O artista marca o checkbox, esfrega, e nada muda — porque o modo que ele
está desligando já estava desligado. E o que o pincel FAZ nesse regime é o comportamento do
**Accumulate ON** (o ombro endurece de `0,294` para `0,980` em quinze passadas), então o produto
default oferece só a metade acumulativa da escolha.

⚠️ **Medi isto no OMBRO de propósito.** No eixo (`d0`) tudo satura em `1.000` em qualquer lei, e uma
tabela medida só ali diria *"1.0000 contra 1.0000, o flag é inerte"* pelo motivo errado — é a lição
que o [doc 25 §13.10](25_avaliacao_gpu.md) pagou com um smoke (*"as contas vivem no ombro"*).

---

## 2. As duas leis, com a álgebra

A lei mora numa função só, [`stroke_cover.rs::cover_add`](../../crates/ph2d-painter-brush/src/stroke_cover.rs).

**Accumulate OFF — o TETO por texel.** O traço carrega um buffer de cobertura `m`; cada dab move o
texel uma *fração* do caminho até um teto:

```
m ← m + w·(cap − m)          cap = grain × coverage,  coverage = falloff × flow × strength
⇒  m_n = cap · (1 − Π(1 − w_k))          →  cap
```

O perfil do dab é uma **TAXA**. Mais dabs aproximam o mesmo teto; nunca o passam.

**Accumulate ON — source-over por dab.** Sem buffer, cada dab compõe por conta:

```
1 − A_n = Π(1 − w_k · g · coverage)      →  1
```

O perfil do dab é uma **QUANTIDADE**. Mais dabs = mais opacidade, sem teto além de 1.

⚠️ **A diferença estrutural não é "quanto", é DE QUE o resultado é função.** O teto é função do
*caminho*; o source-over é função da *lista de dabs*, logo do **Spacing** e da taxa de polling.

---

## 3. O que foi medido

### 3.1 Dentro de UMA pincelada (esfregar vai-e-volta sem soltar)

| strength | modo | n=1 | n=2 | n=5 | n=15 |
|---|---|---|---|---|---|
| 0,3 | off | 0,0902 | 0,0902 | 0,0902 | **0,0902** |
| 0,3 | **ON** | 0,4039 | 0,7255 | 0,9765 | **0,9804** |
| 0,5 | off | 0,2510 | 0,2510 | 0,2510 | **0,2510** |
| 0,5 | **ON** | 0,7843 | 0,9804 | 0,9922 | **0,9922** |
| 1,0 | off | 1,0000 | 1,0000 | 1,0000 | 1,0000 |
| 1,0 | **ON** | 1,0000 | 1,0000 | 1,0000 | 1,0000 |

O flag **funciona**, e funciona bem — **abaixo de `strength = 1`**. A última dupla é a §1.

### 3.2 Pinceladas SEPARADAS (o controle)

| strength | modo | n=1 | n=2 | n=5 | n=15 |
|---|---|---|---|---|---|
| 0,5 | off | 0,2510 | 0,4392 | 0,7608 | 0,9843 |
| 0,5 | **ON** | 0,7843 | 0,9529 | 0,9922 | 0,9922 |

Entre traços o build-up existe nos **dois** modos (cada traço recomeça a própria cobertura). É isso
que o flag governa: *o que uma segunda passada significa **dentro** do mesmo gesto*.

### 3.3 A dependência de ESPAÇAMENTO — e o knob que a compensa

Mesmo CAMINHO, `strength 0.5`, uma passada; só muda quantos dabs o motor emite:

| space_atten | accumulate | sp 0,05 | sp 0,10 | sp 0,20 | sp 0,40 | razão |
|---|---|---|---|---|---|---|
| off | off | 0,2510 | 0,2510 | 0,2510 | 0,2471 | **1,02×** |
| off | **ON** | 0,9137 | 0,7843 | 0,5373 | 0,3098 | **2,95×** |
| **ON** | off | 0,0235 | 0,0510 | 0,0980 | 0,1922 | **8,17×** |
| **ON** | **ON** | 0,2000 | 0,2471 | 0,2510 | 0,2392 | **1,25×** |

⚠️ **A linha 3 é um defeito, e ele tem mecanismo.** *"Adjust Strength for Spacing"* promete
consistência entre espaçamentos e, no modo que **shipa por default** (accumulate OFF), ela leva o
traço de **1,02× (praticamente independente)** para **8,17× (fortemente dependente)** — o inverso
exato do que o nome diz.

**Por quê:** o `space_overlap_factor` é o port do `paint_stroke_integrate_overlap`, e ele devolve
`1 / max(Σ kernels vizinhos)` — *"normaliza um traço denso a opacidade unitária em vez de EMPILHAR"*
(o doc-comment dele). **Empilhar é o que a lei ON faz.** Sob a lei OFF nada empilha; o fator entra em
`coverage = strength × pressure × overlap` ([`stroke.rs:594`](../../crates/ph2d-painter-brush/src/stroke.rs)),
e `coverage` **É o teto** — então atenuar não normaliza nada, só **abaixa o teto** por um fator que
depende do espaçamento (a 5 % ele vale ~1/10,7 ⇒ 0,2510 → 0,0235).

As duas linhas centrais dizem a mesma coisa pelo lado positivo: **o par certo é
`accumulate ON + space_atten ON`** (razão 1,25×) ou **`accumulate OFF + space_atten OFF`**
(razão 1,02×). As duas combinações cruzadas são as que erram.

### 3.4 A COR acumula, o RELEVO não — no MESMO traço

| modo | n | COR (alpha) | RELEVO (h) |
|---|---|---|---|
| off | 1 | 1,0000 | 0,5554 |
| off | 5 | 1,0000 | 0,6143 |
| **ON** | 1 | 1,0000 | 0,5554 |
| **ON** | 5 | 1,0000 | **0,6143** |

O relevo **ignora o flag** (as duas colunas de `h` são idênticas) — é o que o [doc 20 §11](20_accumulate_na_mesma_pincelada.md)
já dizia. E o `0,5554 → 0,6143` que aparece nas duas linhas **não é acúmulo**: é a cauda do envelope
`max` capturando dabs de ida-e-volta ligeiramente deslocados. O número do acúmulo verdadeiro está no
doc 20 §1: dentro de um traço **1,00×**, entre traços **2,00×**.

---

## 4. O que o Blender faz (comportamento, não código)

Três fatos de comportamento, e o que cada um implica:

1. **`Accumulate` (o `BRUSH_ACCUMULATE`) é per-dab.** Com ele ligado os dabs se compõem, então
   passar de novo *dentro* do mesmo traço escurece; com ele desligado o traço satura na **Strength**
   do pincel. É a mesma dicotomia teto-vs-quantidade da §2 — o nosso motor implementa as duas leis
   certas, e o próprio `stroke_cover.rs` cita o modelo do GIMP (`if (opacity > dest) dest +=
   (opacity − dest) * mask * opacity`) como a forma do teto.
2. **`Adjust Strength for Spacing` existe porque o modo acumulativo depende do espaçamento.** É o
   compensador da lei que empilha — e é por isso que, ligado sobre a lei que **não** empilha, ele
   não tem o que normalizar (§3.3).
3. **Os defaults do Blender são `Accumulate` OFF + `Adjust Strength for Spacing` ON.** Os nossos são
   **OFF + OFF** (`space_attenuation: false`, *"Enio 2026-06-24"*).

⚠️ **A leitura honesta do par de defaults:** se o Blender de fato aplicasse a atenuação sobre o teto
como nós aplicamos, o par default dele produziria traços ~10× mais claros em espaçamento fino —
o que seria uma queixa famosa, e não é. Então **ou** ele não aplica a atenuação no caminho capado,
**ou** o cap dele mora num ponto do pipeline que a atenuação não alcança. **Não posso decidir qual
sem a fonte, e não vou afirmar**; o que a medição autoriza a dizer é que *no PH2D* a combinação é
contraditória, e o §5 trata isso como defeito nosso, não como divergência de gosto.

---

## 5. As três divergências, cada uma com número

| # | o que difere | número | natureza |
|---|---|---|---|
| **D1** | **O flag é inerte em `strength = 1.0`** — que é o default | perfil inteiro idêntico (§1) | **defeito de produto**: o controle não faz nada onde o artista o encontra |
| **D2** | **`space_atten` sobre `accumulate OFF` inverte a própria promessa** | 1,02× → **8,17×** | **defeito**, com mecanismo medido (§3.3) |
| **D3** | **O relevo não vê o flag** | cor acumula, `h` não (§3.4) | **lacuna conhecida**, desenhada no doc 20, não construída |

⚠️ **E a D1 é o MESMO mecanismo do endurecimento da borda** que o [doc 25 §13.10.4](25_avaliacao_gpu.md)
mede como item aberto — visto do outro lado. A borda endurece sob muitas passadas *porque em
`strength = 1` não existe teto*; as duas leis de acúmulo que foram tentadas lá (produto vs envelope)
atacavam a forma do dab, e o que decide é **se o traço tem teto**. Uma cura da D1 é uma cura dos dois.

---

## 6. Recomendação

**Ordem de ataque, do que é correção para o que é feature.**

### D2 primeiro — é a mais barata e é estritamente uma correção

A atenuação é o compensador da lei acumulativa; ela deve **valer só quando essa lei está ligada**:

```rust
// space_overlap_factor: o gate ganha a condição que o mecanismo já implica
if !(self.space_attenuation && self.accumulate && spacing_pct < 100.0) { return 1.0; }
```

Custo: uma linha, e **byte-idêntico no default de hoje** (`space_attenuation: false` ⇒ o ramo já
retornava `1.0`). Gate: a razão da §3.3 na linha 3 volta de 8,17× para 1,02×.

⚠️ A alternativa — deixar como está e **esconder** o checkbox quando `accumulate` está off — é pior:
o knob passa a existir e sumir sem o artista saber por quê, e ele é legítimo no modo ON.

### D1 depois — e ela é uma pergunta de PRODUTO, não de engenharia

O teto existe hoje só abaixo de `strength 1`. Três saídas, e **a escolha é do Enio**:

- **(a) O teto vale sempre.** `stroke_cover_wanted` perde a cláusula `strength < 1.0`. Em
  `strength = 1` o teto é 1, então o *centro* não muda um bit — o que muda é o **OMBRO**, que para
  de endurecer (`d6`: 0,980 → ~0,294 em quinze passadas). **É a cura da D1 e do §13.10 de uma vez**,
  e é uma mudança de aparência de todo traço macio esfregado. ⚠️ Custo medido a nomear: o buffer de
  cobertura passa a existir em todo traço (hoje ele é pulado em `strength 1`), e com ele a rota
  cacheada de orientação constante **não** é elegível — o `measure_route_cost` já vigia essa razão.
- **(b) O default de `strength` desce.** Blender-like em espírito (o artista escolhe a opacidade),
  mas muda a primeira pincelada de todo mundo e não cura o §13.10 em `strength 1`.
- **(c) Aceitar e DIZER.** O checkbox fica desabilitado com um motivo visível em `strength = 1`
  (*"sem efeito na força máxima"*). Honesto e barato; não entrega o que o Blender entrega.

**Minha recomendação é (a)**, porque é a única que responde ao report *e* fecha o item da borda —
e porque um teto de 1 é exatamente o que "sem cap" significa hoje, então a metade cara da mudança
já está escrita.

### D3 por último — é a wave do doc 20, e ela tem uma bifurcação sua

O desenho está completo no [doc 20 §9.1/§12/§13](20_accumulate_na_mesma_pincelada.md): acumular a
**CARGA** (não a altura, que assaria o Depth) num plano próprio, com o ciclo de vida no mesmo commit.
A bifurcação do §6 continua de pé: **arco puro** (parar o pincel não deposita nada) contra **relógio
de parede** (demorar constrói, que é o que tinta faz).

---

## 7. O que NÃO refazer

- ⛔ **Trocar o `max` do relevo por `+=` / tirar o `clamp`** — entrega o efeito e traz de volta as
  três doenças que esta linha curou (doc 20 §4): o relevo passa a depender do Spacing e da taxa de
  polling, e o re-stamp por-frame dos shape editors empilha enquanto o artista só *olha*.
- ⛔ **Dar à máscara (ou a qualquer meio) uma lei de cobertura PRÓPRIA** — foi construído e
  **reprovado na tela** (doc 25 §13.10): sem a saturação do produto, a modulação por-dab fica
  visível e o traço sai em CONTAS ao longo do ombro. Ordem do Enio: *"a máscara deve pintar
  exatamente como o brush digital normal"*.
- ⛔ **Medir esta feature no EIXO do traço.** Em `strength = 1` tudo satura em `d0`; o discriminante
  é o perfil perpendicular (§1).

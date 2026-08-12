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

> ⚠️ **CORRIGIDO em 2026-08-12, no mesmo dia, por MEDIR A PRÓPRIA CURA.** A 1ª escrita deste
> documento chamou a inércia de **defeito** e recomendou tirar a cláusula `strength < 1.0` do
> `stroke_cover_wanted`. Construí essa cura e medi: **byte-idêntica**. A §1 e a §5/D1 abaixo estão
> reescritas com o que a medição diz; a receita refutada está pinada na §7 para ninguém a refazer.

**O checkbox Accumulate do PH2D está INERTE na configuração que o app abre — e isso é ARITMÉTICA,
não um defeito.**

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

**Idêntico casa a casa.** O artista marca o checkbox, esfrega, e nada muda.

⚠️ **E o motivo não é a cláusula — é que em `strength = 1` as duas leis SÃO a mesma lei.** Com
`cap = 1` o passo do teto é `m ← m + w·(1 − m)`, e o chamador compõe em
`a = add/(1 − m) = w` ⇒ **source-over por dab**, que é literalmente a lei do Accumulate ON. A
cláusula `strength < 1.0` é portanto uma **OTIMIZAÇÃO** — pular o buffer de cobertura onde ele
provadamente não faz nada — e não a causa. Removê-la é byte-idêntico (medido: as quatro linhas
acima não movem um bit). O gate `at_full_strength_the_two_laws_are_the_same_law` pina a
coincidência.

⚠️ **E o Blender é igual nisto:** lá o Accumulate OFF capa a opacidade do traço na *alpha* do
pincel; com alpha 1 o teto é 1, e o flag também não tem o que fazer. A inércia em força máxima é
**paridade**, não divergência.

O que sobra de verdadeiro na tabela é o outro fato: **o ombro endurece** de `0,294` para `0,980` em
quinze passadas, nos dois modos — e isso é o item aberto do [doc 25 §13.10.4](25_avaliacao_gpu.md),
não deste documento.

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

### 3.4 A COR acumula, o RELEVO não — no MESMO traço ⚠️ *(medição PRÉ-D3; a §6 a superou)*

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
como nós aplicávamos, o par default dele produziria traços ~10× mais claros em espaçamento fino —
o que seria uma queixa famosa, e não é. Então **ou** ele não aplica a atenuação no caminho capado,
**ou** o cap dele mora num ponto do pipeline que a atenuação não alcança. **Não posso decidir qual
sem a fonte, e não vou afirmar**; o que a medição autoriza a dizer é que *no PH2D* a combinação era
contraditória — e é isso que a D2 conserta.

---

## 5. O que de fato difere, depois de medir as curas

| # | o que se pensou | o que a MEDIÇÃO diz | estado |
|---|---|---|---|
| **D1** | *o flag é inerte em `strength = 1` ⇒ defeito* | **não é defeito: as duas leis COINCIDEM ali** (`cap = 1` ⇒ o teto reduz a source-over por dab). A cura proposta foi construída e mediu **byte-idêntica**. E o Blender tem a mesma inércia em alpha 1 | ⛔ **REFUTADO** — vira gate de coincidência |
| **D2** | *`space_atten` sobre `accumulate OFF` inverte a própria promessa* | **1,02× → 8,17×**, com mecanismo: o fator entra em `coverage`, que **é** o teto (§3.3) | ✅ **CORRIGIDO** — o knob passa a valer só com Accumulate ON; **1,02×** de volta |
| **D3** | *o relevo não vê o flag* | cor acumula, `h` não (§3.4) — a assimetria era real | ✅ **FECHADA** (§6): a lei do arco alcança o corpo; `n=15` vai de **0,6153 para 11,9602**, com a passada simples a **3,0%** e o espaçamento a **1,02×** |

⚠️ **E a D1 tinha uma consequência que também cai:** eu escrevi que curá-la curaria o endurecimento
da borda do [doc 25 §13.10.4](25_avaliacao_gpu.md). Cai junto — se as duas leis coincidem em
`strength = 1`, **nenhum ajuste do teto muda o ombro ali**, e o §13.10.4 continua exatamente onde
estava, pelo motivo que ele mesmo já mede.

⇒ **A D3 era a única divergência real, e era a pergunta original** ([doc 20](20_accumulate_na_mesma_pincelada.md),
abertura: *"a possibilidade de accumulate na mesma pincelada em todo o sistema Impasto"*). Ela está
construída — §6.

⚠️ **E o PH2D passa o Blender aqui, não o iguala:** o Accumulate do Blender é **por dab**, logo
função do Spacing; o nosso é uma **integral de arco**, logo função do CAMINHO (razão medida 1,02×
contra os 2,95× que a lei por-dab dá na cor). Um artista que baixe o Spacing não engrossa a tinta
por acidente.

---

## 6. Recomendação

**Ordem de ataque, do que é correção para o que é feature.**

### ✅ D2 — FEITO (2026-08-12)

A atenuação é o compensador da lei acumulativa; ela passou a **valer só quando essa lei está ligada**:

```rust
// space_overlap_factor: o gate ganha a condição que o mecanismo já implica
if !(self.space_attenuation && self.accumulate && spacing_pct < 100.0) { return 1.0; }
```

Custo: uma linha, e **byte-idêntico no default de hoje** (`space_attenuation: false` ⇒ o ramo já
retornava `1.0`). Medido depois: a linha 3 da §3.3 vai de **8,17× para 1,02×**, e a linha 4 (o modo
em que o knob serve) fica em **1,25×**, intocada.

Gates, com as duas metades (senão *"desligar a atenuação em todo lugar"* passaria e mataria a
feature): `the_spacing_knob_never_makes_a_capped_stroke_spacing_dependent` ·
`the_spacing_knob_still_flattens_the_law_that_piles_up` ·
`spec_tests::the_spacing_attenuation_only_applies_to_the_law_that_piles_up`. **Mutação** (tirar o
`&& self.accumulate`): sangra os dois primeiros, deixa o controle verde.

⚠️ **Um gate PRÉ-EXISTENTE mudou de premissa junto**, e isso é parte da correção:
`space_attenuation_reduces_coverage_below_full_spacing` afirmava a atenuação sobre a fixture default
(`accumulate: false`) — ou seja, sobre exatamente a combinação que estava errada. Ele passou a
**declarar a premissa** (`accumulate: true`, com o porquê ao lado); a metade *"e é neutra sob o
teto"* é o gate novo em `spec_tests`.

⚠️ A alternativa — deixar como está e **esconder** o checkbox quando `accumulate` está off — é pior:
o knob passaria a existir e sumir sem o artista saber por quê, e ele é legítimo no modo ON.

### ⛔ D1 — construída, MEDIDA e REFUTADA (2026-08-12)

Tirar a cláusula `strength < 1.0` do `stroke_cover_wanted` foi implementado como experimento e
medido no perfil perpendicular: **as quatro linhas de `strength 1.0` não moveram um bit**. A cura
não existe porque o defeito não existe (§1). O que fica é o gate
`at_full_strength_the_two_laws_are_the_same_law`, que afirma a coincidência.

### ⛔ D3 — CONSTRUÍDA e REPROVADA NO SMOKE, pela **SEGUNDA** vez (2026-08-12)

> **Enio, pós-smoke:** *"Accumulate para Impasto não ficou bom. Vamos desativar para o modo de
> Impasto."* ⇒ **o motor foi revertido inteiro**; o que fica é o guarda
> `accumulate_tests::the_body_of_the_paint_never_sees_the_accumulate_flag`.

⚠️ **É a segunda vez que esta mesma feature é construída e reprovada.** A primeira foi em
**2026-07-18** (integral de arco num acumulador próprio, `stroke_accum`/`live_accum`, 3 mutações;
*"não gostei, vamos desfazer"*), e foi ela que fez o checkbox **deixar de ser oferecido** sob
impasto. **Antes de a construir uma terceira vez, leia isto inteiro.**

⚠️ **E a 2ª tentativa tinha um defeito que o smoke nem precisou ver: a capacidade era INALCANÇÁVEL
pelo produto.** A row do Accumulate está escondida sob impasto desde 07-18 (gate
`impasto_hides_the_accumulate_row_but_it_is_alive_without_it`), então o motor novo respondia a um
controle que o painel não pinta — a sonda só chegou lá porque arma o flag por baixo do pano. Pior:
ele **era** alcançável pelo caminho torto (marcar Accumulate em **Digital** e depois trocar para
Impasto — o flag sobrevive nos slots e a row some), o que faria um checkbox **invisível** mudar o
relevo. *Um controle escondido que age é pior que um controle que falta.*

**O que a tentativa mediu antes de morrer** (fica porque o número é bom e a próxima pessoa merece
saber que a lei funcionava — o que reprovou foi o RESULTADO na tela, não a matemática):

| modo | n=1 | n=2 | n=5 | n=15 |
|---|---|---|---|---|
| off (envelope `max`) | 0,5554 | 0,6083 | 0,6143 | **0,6153** |
| ON (integral de arco) | 0,6323 | 1,4195 | 3,8523 | **11,9602** |

Uma passada reta saía a **3,0%** do envelope (a norma `2·∫₀^ρ perfil` funcionava), o espaçamento a
**1,02×** (I1 honrado), e o re-carimbo recomeçava do zero (I2). O TAP custava `0,0936` contra
`0,4679` — o preço da decisão (i), e um candidato ao que "não ficou bom".

<details>
<summary>O desenho que funcionava, para não ser re-derivado do zero numa terceira tentativa</summary>

### O desenho (revertido — descrição, não código vivo)

**O relevo vê o flag.** Medido pela porta do artista (pincel macio r=8, impasto, ida-e-volta na MESMA
pincelada, altura no meio do traço):

| modo | n=1 | n=2 | n=5 | n=15 |
|---|---|---|---|---|
| off (envelope `max`) | 0,5554 | 0,6083 | 0,6143 | **0,6153** |
| **ON (integral de arco)** | 0,6323 | 1,4195 | 3,8523 | **11,9602** |

E os três números que dizem que a lei está certa, não só que ela cresce:

- **UMA passada reta**: `off 0,6153` contra `ON 0,5968` — **3,0%**. É o que a norma
  `2·∫₀^ρ perfil` compra: ligar o toggle **não repinta** a arte de quem passa o pincel uma vez.
- **Independência de espaçamento (I1)**: razão **1,02×** sobre `sp = 0,05 / 0,10 / 0,20`, contra
  1,01× do envelope. A doença que esta linha curou três vezes não volta.
- **Idempotência sob re-stamp (I2)**: por construção (a integral é função do caminho) + o
  `reset_stroke_height`, que zera a carga antes de cada re-carimbo dos shape editors.

⚠️ **O preço da decisão (i), medido:** um **TAP** deposita `h = 0,0936` contra `0,4679` sob o
envelope — *uma unidade de espaçamento*, ~5× mais fino. A lei pura do arco daria **zero** (um toque
não percorre nada), e um pincel que não carimba parado é ferramenta quebrada. Quem quer o toque
grosso desliga o flag.

⚠️ **Consequência irmã, NÃO resolvida por (i):** um **Airbrush parado** também não engrossa — ele
emite dabs no TEMPO e a integral é de ESPAÇO. É a metade que só o relógio de parede resolveria, e
ele viola I2 (doc 20 §6).

**Gates:** 5 no motor (`ph2d-painter-brush::height_tests`) + 5 de comportamento
(`accumulate_tests`). **Mutações: 5 rodadas, 4 sangram** — o piso nominal (`TAP = 0`), a norma
(`passada reta 0,6153 → 4,7743`), o passo de arco (`razão 3,21×`), o `accum_step` sempre `None`
(`0,5554 → 0,6153`), e a cápsula mantida sob a integral (`0,7175`, a contagem dupla). ⚠️ A 5ª é
**inválida e está registrada como tal** no gate.

**Custo estrutural: ZERO.** Nenhum plano novo, nenhum ciclo de vida, nenhum `PROJECT_SCHEMA`,
nenhuma superfície pública além de `accum_norm`/`accum_step`. E o card **Body continua vivo**.

</details>

### O que a D3 CUSTOU descobrir — e por que o doc 20 a orçou tão maior

O desenho está no [doc 20 §9.1/§12/§13](20_accumulate_na_mesma_pincelada.md): acumular a **CARGA**
(não a altura, que assaria o Depth).

⚠️ **E a metade CARA daquele plano dissolveu na varredura de consumidores (2026-08-12).** O §12 do
doc 20 diz *"acumular dentro de `fields.paint` não funciona (clampado)"* e conclui que a feature
precisa de um **plano novo**, cujo custo *"não é a alocação, é o CICLO DE VIDA"* (snapshot, restore,
commit, undo — a cicatriz do `mats`). Medido: **o clamp mora dentro do `derive_height`, que é o
FUNIL ÚNICO do plano.** A varredura de todos os leitores de `paint[i]` na árvore dá:

| leitor | o que faz com `paint[i]` |
|---|---|
| `impasto.rs:527` · `impasto_live.rs:230` | `derive_height(spec, paint[i], grain)` |
| `height_walk.rs:257/296/298` | o próprio envelope + a mordida do Push |
| `height_push.rs` (4 sítios) | **já `.clamp(0.0, 1.0)`** |
| a fusão em `covers` (`impasto_live.rs:65`) | lê o **`film`**, nunca o `paint` |

⇒ Um `paint > 1` **não alcança nada que não o clampe**, exceto o `derive_height` — e é exatamente
lá que a extensão tem de estar. Isso troca *"plano novo + ciclo de vida em 8 sítios"* por
**uma função estendida + um ramo no walk**, e — de graça — o card Body continua **VIVO** no modo
acumulativo, porque os ingredientes armazenados seguem sendo `paint`/`grain`/`radius`.

A extensão é: `m ≤ 1` **inalterado** (byte-idêntico, e é o que torna o OFF gratuito); `m > 1` cresce
**linearmente** (`a = a(1) + (m − 1)`), que é o platô continuar engordando.

⚠️ **O que FALTA é uma decisão de produto, e ela é a mesma que o doc 20 §6 previu.** A lei do arco
é `L = Σ (perfil · Δs) / NORM`, com `NORM = 2∫₀^ρ perfil` (assim UMA passada reta tem exatamente o
pico de hoje — o gate que torna o toggle honesto). Ela satisfaz I1 e I2. Mas `Δs` é **distância
percorrida**, então:

- **um TAP deposita ZERO** (`Δs = 0`), e um pincel que não carimba parado é uma ferramenta quebrada;
- **o AIRBRUSH parado deposita ZERO** pelo mesmo motivo (ele emite dabs no tempo, não no espaço).

Duas saídas, e **a escolha é do Enio**: **(i)** o 1º dab de um traço recebe `Δs = espaçamento
nominal` (um tap deposita "uma unidade de espaçamento" — fino, mas não nulo); **(ii)** a integral é
de **relógio de parede**, que resolve os dois casos e **viola I2** (o tempo não é propriedade do
caminho ⇒ um shape editor que re-carimba a figura a cada quadro não reproduz o mesmo relevo).

**Minha recomendação é (i)**, com a consequência NOMEADA: sob Accumulate um toque é mais fino que
sob o envelope, e quem quer o toque grosso desliga o flag.

⚠️ **E há um segundo detalhe do kernel que a lei do arco exige:** em modo acumulativo o corpo **não
pode ser varrido** (a cápsula). A cápsula existe para tornar o *envelope* independente do
espaçamento; sob a integral quem faz isso é o `Δs`, e manter as duas **conta duas vezes** (o texel
somaria `perfil(d) · comprimento_do_traço`, que cresce sem limite com um traço reto longo). No ramo
acumulativo o `sweep` é `None` — o perfil volta a ser a distância ao CENTRO, que é o que a integral
de linha pede.

⚠️ **A bifurcação do doc 20 §6 pode ser RESOLVIDA pelos invariantes, e eu recomendo resolvê-la
assim:** *arco puro* satisfaz I1 **e** I2 (a integral é função do caminho, logo idempotente sob
re-stamp); *relógio de parede* **viola I2 por construção** — quanto tempo o artista demorou não é
propriedade do caminho, então um shape editor que re-carimba a figura a cada quadro não tem como
reproduzir o mesmo relevo. Como os shape editors são um caminho de primeira classe deste Painter, a
variante por tempo é **inexprimível** aqui, não meramente indesejada.

⇒ **arco**, com a consequência honesta e já-verdadeira-hoje: *parar o pincel não deposita mais nada*
(é o que o envelope `max` faz agora), então nenhuma regressão. **Veto do Enio pendente**, e é a
única coisa que falta para a D3 começar.

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
- ⛔ **Tirar a cláusula `strength < 1.0` do `stroke_cover_wanted`** esperando que o ombro pare de
  endurecer. **Foi construída e medida: byte-idêntica** (§5/D1). Ela é uma otimização, não a causa —
  e o gate `at_full_strength_the_two_laws_are_the_same_law` existe para que a próxima pessoa leia o
  fato em vez de repetir o experimento.
- ⛔ **Ligar `space_atten` "porque é o default do Blender"** sem olhar a §3.3: o par
  `space_atten ON + accumulate OFF` era o que abaixava o teto por um fator do espaçamento. Hoje ele
  é neutro por construção, mas o raciocínio de *"copiar o default"* é o que reintroduz a classe.

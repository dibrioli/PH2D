# SPEC — o pincel de CONTORNO (clean-room, comportamento observado)

```
Alvo: Blender 5.2.1 LTS (binário /usr/bin/blender) · fonte lido: tag v5.2.0 · Licença: GPL-2.0-or-later · Degrau: T2
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-boundary.md, 2026-09-13
Patente (§8.1): buscado em 2026-09-13 — termos e resultado no ledger; nenhuma patente viva alcança o método
  (uma cerca NOMEADA fica: ⛔ nunca acrescentar «reposição de volume por inflação» ao modo de suavizar — §10.6)
Filtragem §4.3: executada em 2026-09-13 · Sweep: verde em 2026-09-13 (vassoura de 163 entradas)
Auditoria §4.2 (R-pré): ⏳ pendente — condição de abrir a janela que implementa
Mapa de leitura da literatura: nenhum paper. As fontes livres usadas são (a) o manual público do alvo,
  (b) as mensagens de commit públicas e (c) o rastreador de defeitos público — todas re-ditas em palavras
  nossas, com o endereço ao lado; e (d) a SAÍDA do binário sobre malhas NOSSAS (§19).
Denylist de URLs (⛔ o Implementador NUNCA abre): projects.blender.org/blender/blender (fonte, commits, PRs),
  qualquer espelho do fonte (github.com/blender/blender, git.blender.org), developer.blender.org,
  qualquer code-search que devolva o fonte do alvo (grep.app, searchcode, sourcegraph).
"Este documento descreve comportamento; não contém expressão do alvo."
```

---

## §0 — O que esta espec é, e como se lê

Descreve **o que o pincel de contorno faz**, fase a fase, com cada número acompanhado da sua
proveniência: **F** = derivado de fórmula/matemática, **M** = medido na saída do oráculo (com a
fixture ao lado), **D** = documentado pelos autores em fonte pública (manual, mensagem de commit,
rastreador), **N** = decisão nossa.

⚠️ **A decomposição em fases abaixo é NOSSA** — escolhida para descrever o comportamento. Não
reproduz a organização do alvo, e o Implementador é livre de a repartir por ficheiros/funções como
quiser.

⚠️ **Nomes PÚBLICOS conservados de propósito** (SKILL §4.1.13 — são o que a API do alvo expõe e o
que a nossa UI deve poder nomear, e são a chave para correr o oráculo outra vez): os valores de
`boundary_deform_type` (`BEND` · `EXPAND` · `INFLATE` · `GRAB` · `TWIST` · `SMOOTH`), os de
`boundary_falloff_type` (`CONSTANT` · `RADIUS` · `LOOP` · `LOOP_INVERT`), e `boundary_offset` /
`deform_target`. ⛔ Nenhum nome interno do alvo aparece neste documento.

---

## §1 — O que o pincel é (uma frase) e o que fica FORA

**O artista aponta para perto de uma borda ABERTA da malha, arrasta, e a borda inteira — ou um
troço dela — deforma-se, com a deformação a esmorecer para dentro da peça.**

⛔ **FORA desta espec, por ordem do dono:** tudo o que depende de *face sets* (não existem aqui) e
todo pincel/opção de COR. ⚠️ Uma consequência dessa exclusão é **load-bearing** e está medida em
§4.3: aqui, contorno é **só a borda aberta da malha** (e a borda que o esconder produz, §4.2) —
⛔ o alvo NÃO trata a fronteira entre dois face sets como contorno deste pincel.

⛔ **FORA por decisão desta espec** (o alvo tem, nós não temos o substrato): as duas representações
alternativas de malha do alvo (subdivisão residente e malha dinâmica). A lei é a mesma; o que muda
é quem responde *«quem é vizinho de quem»*. Onde o alvo diverge entre representações, está dito
em §13.6.

---

## §2 — Vocabulário (NOSSO; use-o no código)

| termo | significado |
|---|---|
| **aresta de borda** | aresta com **menos de duas** faces incidentes |
| **vértice de borda** | vértice de uma aresta de borda |
| **vértice âncora** | o vértice de borda escolhido sob o cursor — a origem de tudo (§5) |
| **cadeia** | o conjunto de vértices de borda alcançados a partir da âncora (§6) |
| **distância de cadeia** | comprimento ACUMULADO de arestas desde a âncora, andando **pela borda** (§6) |
| **anel `k`** | os vértices a `k` passos topológicos da cadeia, para dentro da malha (§7) |
| **alcance `K`** | o maior `k` que a propagação atingiu (§7.3) |
| **patrono** de um vértice interior | o vértice da cadeia de onde a propagação chegou até ele (§7.2) |
| **ponto-origem** | a posição do vértice do anel `K` na coluna da âncora — o que o alvo desenha como linha branca (§7.4) |
| **peso** de um vértice | o escalar em `[−1, 1]` que gradua a deformação dele (§8) |
| **avanço** `s` | o escalar com sinal que o arrasto da mão produz (§9) |

---

## §3 — Arquitectura de fases

Ao **pen-down** (e uma vez por passo de simetria — §12) correm as fases **A→E**, que produzem uma
estrutura constante para o resto do traço. A cada passo do traço correm **F→H**.

```
A  censo de bordas na malha            (uma vez por malha; cache)
B  escolher o vértice âncora           (pen-down)  → pode RECUSAR o traço inteiro
C  andar a borda a partir da âncora    (pen-down)  → cadeia + distância de cadeia
D  propagar para dentro                (pen-down)  → anel, patrono, alcance, ponto-origem
E  pesos por vértice                   (pen-down)  → curva × máscara × automáscara × queda-no-contorno
--- por passo do traço ---
F  avanço s a partir do arrasto        → um escalar (ou o vector inteiro, no GRAB)
G  aplicar a lei do modo               → posição nova de cada vértice, a partir do REPOUSO
H  converter em translação e aplicar   → com filtro de peso zero, recorte de eixos e simetria
```

⚠️ **A estrutura A–E é fotografada no pen-down e NÃO é recalculada durante o traço.** É isso que
torna o resultado função só do arrasto TOTAL e não do caminho (prova em §14.2).

---

## §4 — Fase A: o que conta como contorno

### §4.1 — A regra (F, confirmada por M)

Uma aresta é **aresta de borda** se tiver **menos de duas faces incidentes** — ou seja `0` ou `1`.
⚠️ Uma aresta solta (*wire*, zero faces) conta. Os dois vértices dela são **vértices de borda**.

**Medido** nas fixtures de repouso (o alvo também tem testes públicos que afirmam o mesmo — D):

| peça | vértices de borda | arestas de borda | fixture |
|---|---|---|---|
| casca fechada (esfera) | `0` | `0` | `esfera.repouso` |
| grelha `3×3` de quadrados | `8` (o perímetro) | `8` | — (teste público do alvo, D) |
| tira de `2` quadrados (`3×2` vértices) | **todos os `6`** | `6` de `7` | `faixa.repouso` |

⭐ **Numa tira de uma fileira TODO vértice é de borda** — e é isso que a torna o caso de recusa da
§5.3.

### §4.2 — Geometria ESCONDIDA também produz contorno (D + leitura)

Um vértice cujas faces incidentes **não estão todas visíveis** é tratado como vértice de borda,
mesmo que a malha seja fechada ali. ⚠️ Não temos fixture para isto (o harness não esconde
geometria) — é **afirmação documentada** pelos autores (manual público e mensagem de commit de
2020-09-06) e confirmada por leitura. ⇒ **quem implementar o esconder tem de reconferir isto com
uma corrida nova do oráculo.**

### §4.3 — ⛔ A exclusão C, dita com precisão

O alvo tem uma segunda família de «contornos» — a fronteira entre conjuntos de faces marcados —
que **este pincel não usa**. Aqui ela **não existe** e não deve ser construída.

### §4.4 — O elo que a régua ingénua perde (D, 2025-11-19)

⚠️ **«São vizinhos de borda os vértices de borda ligados por uma aresta»** é a regra certa, e
`ambos são vértices de borda` **não** a exprime: numa tira de faces, dois vértices de borda podem
estar ligados por uma aresta **interior**. ⇒ a pergunta certa é *«esta ARESTA é de borda?»*, o que
obriga a guardar o **conjunto das arestas de borda**, não só a marca por vértice. Os autores
pagaram este defeito e curaram-no exactamente assim.

### §4.5 — Custo

O censo é `O(arestas)` e é **cacheado por malha** (invalidado quando a topologia muda). M: numa
grelha de `263 169` vértices ele não aparece no relógio do traço (§15).

---

## §5 — Fase B: escolher o vértice âncora, e quando o pincel RECUSA

### §5.1 — A escolha (leitura + M)

1. Toma-se o vértice da malha mais próximo do ponto de contacto do cursor (é o vértice activo que
   o resto do editor já calcula).
2. **Se esse vértice já é de borda, ele é a âncora.**
3. Senão, faz-se uma busca em largura a partir dele, **limitada pelo raio do pincel** — um vizinho
   só entra na fronteira se `|posição(vizinho) − posição(inicial)|² < raio²` —, contando **passos
   topológicos**. A âncora é o vértice de borda com **menos passos**; empates ficam com o primeiro
   encontrado na ordem de visita.
4. Se a busca não achar nenhum vértice de borda, **não há traço**: nada se move, e nem sequer se
   desenha a pré-visualização (§17).

⚠️ **O raio que limita esta busca é o raio INICIAL do traço** (o do pen-down), não o raio dinâmico
que a pressão possa estar a modular — os autores corrigiram precisamente isto (D, defeito público
#130101).

### §5.2 — As duas recusas (F/D, confirmadas M)

Depois de achada a âncora, o traço é **recusado por inteiro** (nada se move) se o vértice **sob o
cursor** falhar qualquer destes dois testes — contando só vizinhos visíveis:

| teste | recusa quando | porquê (D) |
|---|---|---|
| **grau** | o vértice tem **≤ 2** vizinhos | numa quina é ambíguo qual das duas bordas activar |
| **borda ao redor** | **> 2** dos vizinhos são de borda | a borda é não-manifold ali e o resultado seria imprevisível |

⚠️⚠️ **O sujeito destes dois testes é o vértice SOB O CURSOR, não a âncora** — e é assim mesmo que
o alvo faz: a âncora pode ser perfeitamente sã e o traço ser recusado por causa do vértice que o
artista apontou. O próprio alvo marca este ponto como duvidoso no código; **nós reproduzimo-lo**
(N: reproduzir, com gate que o nomeia — §19.4), porque é o comportamento que o corpus mede.

⚠️ **O MESMO par de testes é o critério de PARAGEM do passeio da fase C** (§6.2), aí aplicado ao
vértice que se vai visitar.

### §5.3 — As três recusas MEDIDAS

| fixture | peça | o que acontece | mecanismo |
|---|---|---|---|
| `esfera_agarrar_constante` | casca fechada, `738` v | **`0` vértices movidos** | não há aresta de borda ⇒ a busca da §5.1 falha |
| `grade_canto_agarrar_constante` | grelha, cursor na QUINA | **`0` movidos** | grau `2` ⇒ teste do grau |
| `faixa_agarrar_constante` | tira de uma fileira, `66` v | **`0` movidos** | todo vizinho é de borda ⇒ teste da borda ao redor |

⭐ E o controlo que impede ler isto como «recusa demais»: `grade_quina_interior_agarrar_constante`
(cursor num entalhe interior da mesma grelha) move **`165`** vértices, como o caso normal.

---

## §6 — Fase C: andar a borda (a cadeia e a distância de cadeia)

### §6.1 — A regra (F)

Busca em largura a partir da âncora, **restrita a vértices de borda** (§4.4). Cada vértice novo
`w`, alcançado a partir de `v`, recebe

```
distância_de_cadeia(w) = distância_de_cadeia(v) + |posição(w) − posição(v)|
distância_de_cadeia(âncora) = 0
```

⭐ **É comprimento de arco REAL, somado aresta a aresta** — não contagem de arestas e não distância
euclidiana em linha recta. Numa borda curva (o aro de um tubo) as duas divergem, e é a primeira que
manda.

### §6.2 — Onde o passeio pára

O passeio **não atravessa** um vértice que falhe os testes da §5.2 (grau ≤ 2, ou > 2 vizinhos de
borda). ⇒ numa peça rectangular, a cadeia **pára nas quinas** e a deformação fica confinada ao lado
que o artista apontou. Numa peça com várias bordas separadas, só a borda da âncora entra.

⚠️ O vértice que faz parar **entra na cadeia** (recebe distância) e só não propaga a partir dali.

### §6.3 — Bordas fechadas (M)

Uma borda que fecha sobre si mesma (o aro de um tubo) é percorrida inteira e a distância de cadeia
cresce pelos dois lados até se encontrarem. **`tubo_*`: `128` vértices movidos = `32` do aro ×
`4` anéis.** ⛔ **Não existe tratamento especial de «a borda fecha um laço»**: o alvo teve código
para isso, ele **nunca funcionou** (a bandeira que o armava jamais era escrita) e foi **removido**
(D, defeito público #125350, ainda ABERTO como pedido de feature). ⇒ **N: não o construa.**

---

## §7 — Fase D: propagar para dentro da malha

### §7.1 — A regra (F)

Busca em largura **multi-origem**: a fronteira inicial é a **cadeia inteira** (todos com anel `0`).
A cada passagem, todo vizinho ainda não visitado recebe `anel = anel(origem) + 1` e
`patrono = patrono(origem)`. ⚠️ Um vértice é atribuído **uma só vez** — quem chegar primeiro fica
com ele; é isso que reparte a malha em «colunas», uma por vértice da cadeia.

### §7.2 — O patrono é o que liga o interior à borda

`patrono(v)` é o vértice da cadeia de onde a onda chegou a `v`. É por ele que:
- a **queda ao longo do contorno** (§8.3) é resolvida (a coluna inteira herda o valor do seu
  vértice de cadeia);
- o **eixo/direcção** dos modos `BEND` e `EXPAND` é propagado (§10.1, §10.2).

### §7.3 — Onde a propagação pára, e o que é o alcance `K`

Antes de cada passagem testa-se **`distância_acumulada > raio_de_propagação`**; sendo verdade, ou
estando a fronteira vazia, pára-se e `K` = o número de passagens completadas.

A `distância_acumulada` **não** é a profundidade de todos: é somada **só ao longo da coluna da
âncora** (uma parcela por vizinho novo dessa coluna, por passagem). ⇒ o alcance é decidido pelo
tamanho das arestas **onde o artista carregou**, e uma malha com densidade irregular alcança mais
fundo onde as arestas são curtas.

**Raio de propagação** = `raio_inicial × (1 + boundary_offset)` (F; ver §8.4).

**Medido** (grelha de lado `2,0`; `raio_inicial = 0,25`):

| aresta | `raio_prop / aresta` | anéis com peso ≠ 0 | `K` | fixture |
|---|---|---|---|---|
| `0,0625` (`32×32`) | `4` | `5` (`165 = 5×33` movidos) | `5` | `grade_agarrar_constante` |
| `0,0625`, offset `1` | `8` | `9` (`297 = 9×33`) | `9` | `grade_dobrar_constante_origem1` |
| `0,1` (grelha `4×4`) | `2,5` | `3` (`15 = 3×5`) | `3` | `grade_pequena_dobrar_origem0` |
| `0,0833` (tubo) | `3` | `4` (`128 = 4×32`) | `4` | `tubo_dobrar_constante` |

⇒ **anéis com peso ≠ 0 = `floor(raio_prop / aresta) + 1`** e `K` = esse número (M, 4 pontos).
⚠️ Os vértices do anel `K` **têm** entrada de propagação mas recebem peso `0` (§8.1) — por isso a
contagem de «movidos» é `K` anéis, não `K+1`.

### §7.4 — O ponto-origem (a linha branca)

À medida que a coluna da âncora avança, guarda-se a posição do vértice mais recente dela; no fim,
essa posição é o **ponto-origem** — o vértice do anel `K` na coluna da âncora. Ele é:
- o que o alvo **desenha como uma linha branca** do vértice âncora até ele, para o artista ver *até
  onde a deformação chega* (D, manual);
- o ponto que define a **direcção do avanço** (§9.1) e o **eixo do `TWIST`** (§10.5).

---

## §8 — Fase E: o peso de cada vértice

O peso de um vértice com entrada de propagação é o produto de quatro coisas; um vértice **sem**
entrada (fora do alcance) tem peso **`0`** sempre.

```
peso(v) = queda_de_profundidade(anel(v), K)      §8.1
        × (1 − máscara(v))                        §8.2
        × automáscara(v)                          §8.2
        × queda_no_contorno(patrono(v))           §8.3   [1 quando CONSTANT, ou quando patrono = âncora]
```

### §8.1 — Queda de profundidade (F, M ao bit)

É a **curva de queda do pincel**, avaliada com `distância = anel(v)` e `comprimento = K`:

```
se d ≥ L:  0
senão:     p = 1 − d/L, e a curva:
           CONSTANT  1          SHARP  p²        LINEAR  p
           SMOOTH    3p² − 2p³  SMOOTHER p³(p(6p − 15) + 10)   ROOT  √p   (e a curva autorada)
```

⭐ **Medido ao bit** na grelha (`K = 5`, `raio 0,25`, arrasto `0,1`, `GRAB`), colunas `anel 0..5`:

| curva | `0` | `1` | `2` | `3` | `4` | `5` | fixture |
|---|---|---|---|---|---|---|---|
| `SMOOTH` | `0,1` | `0,0896` | `0,0648` | `0,0352` | `0,0104` | `0` | `grade_agarrar_constante` |
| `LINEAR` | `0,1` | `0,08` | `0,06` | `0,04` | `0,02` | `0` | `…_curvalinear` |
| `SHARP` | `0,1` | `0,064` | `0,036` | `0,016` | `0,004` | `0` | `…_curvasharp` |
| `CONSTANT` | `0,1` | `0,1` | `0,1` | `0,1` | `0,1` | `0` | `…_curvaconstante` |

⚠️ **A curva `CONSTANT` também dá `0` no anel `K`** — a cláusula `d ≥ L ⇒ 0` vem **antes** da
escolha da curva. *Um implementador que ponha o `1,0` da constante antes do corte produz um anel a
mais e um degrau na borda da deformação.*

### §8.2 — Máscara e automáscara (D, M)

- **Máscara**: multiplica por `1 − máscara`. M: máscara uniforme `0,5` ⇒ todos os deslocamentos a
  metade, ao bit (`grade_agarrar_constante_mascara05`: `0,05 / 0,0448 / 0,0324 / 0,0176 / 0,0052`
  contra os dobros da tabela acima).
- **Automáscara**: multiplica pelo factor da automáscara do traço. ⚠️ **Este pincel tem queda
  PRÓPRIA**, logo a automáscara não lhe chega pelo caminho comum dos outros pincéis e tem de ser
  aplicada aqui, explicitamente — os autores corrigiram exactamente esta omissão (D, defeito
  público T84896). ⛔ **Não medido** (o harness desliga toda a automáscara): quem a ligar reconfere.

### §8.3 — Queda AO LONGO do contorno — os quatro modos

Dado `t = distância_de_cadeia(patrono(v))` e o **raio inicial** `R` (⚠️ **não** o raio de
propagação — §8.4):

| `boundary_falloff_type` | factor | sinal |
|---|---|---|
| `CONSTANT` | `1` em toda a cadeia | `+1` |
| `RADIUS` | `curva(t, R)` | `+1` |
| `LOOP` | `curva(f, R)`, com `f = t mod R` se `⌊t/R⌋` é par, senão `R − (t mod R)` | `+1` |
| `LOOP_INVERT` | igual ao `LOOP` | `−1` quando `⌊t/R⌋ ∈ {1,2, 5,6, 9,10, …}`, senão `+1` |

⭐ **A regra de sinal do `LOOP_INVERT` é «dois SIM, dois NÃO», deslocada de um** — o lóbulo `0` não
inverte, os lóbulos `1` e `2` invertem, `3` e `4` não, `5` e `6` invertem… Medido lóbulo a lóbulo
em `grade_agarrar_laco_invertido` (o `uy` muda de sinal exactamente em `⌊t/R⌋ = 1` e volta em `3`):

| `⌊t/R⌋` | `0` | `1` | `2` | `3` | `4` |
|---|---|---|---|---|---|
| sinal medido | `+` | `−` | `−` | `+` | `+` |

⚠️⚠️ **A coluna do artista é ISENTA desta queda:** todo vértice cujo **patrono é a própria âncora**
salta a §8.3 por completo (fica com factor `1` e sinal `+1`), **em todos os quatro modos**. ⇒ sob
`RADIUS`, o ponto onde a mão carregou recebe sempre a deformação cheia, mesmo que a curva já
estivesse a esmorecer. Medido: em `grade_agarrar_raio` o pico é exactamente `0,1` (o valor cheio) na
coluna da âncora e `0,05` a duas células dali.

**Medido, perfil ao longo da borda** (`R = 0,25`, aresta `0,0625`, `SMOOTH`, arrasto `0,1`):

| `t` | `0` | `0,0625` | `0,125` | `0,1875` | `0,25` | `0,375` | `0,5` |
|---|---|---|---|---|---|---|---|
| `RADIUS` | `0,1` | `0,0844` | `0,05` | `0,0156` | `0` | `0` | `0` |
| `LOOP` | `0,1` | `0,0844` | `0,05` | `0,0156` | `0` | `0,05` | `0,1` |

⇒ `RADIUS` move `35 = 7 colunas × 5 anéis`; `LOOP` move `145 = 29 × 5` (as quatro colunas de zero
da onda triangular ficam de fora). Fixtures `grade_agarrar_raio`, `grade_agarrar_laco`.

### §8.4 — ⚠️⚠️ `boundary_offset` alonga a PROPAGAÇÃO e NÃO a queda no contorno

Há **dois** raios em jogo e confundi-los é o erro caro:

| onde | valor | efeito |
|---|---|---|
| profundidade da propagação (§7.3) e portanto `K` e o ponto-origem | `R × (1 + boundary_offset)` | a deformação entra mais fundo e o braço de alavanca cresce |
| queda ao longo do contorno (§8.3) | **`R`**, sem offset | o troço de borda afectado **não muda** |

É isso que os autores descrevem como *alongar o braço sem mexer na queda* (D). Medido: offset `1`
leva os anéis de `5` para `9` (`165 → 297` movidos) e o deslocamento máximo do `BEND` de `0,367`
para `0,661`, enquanto o `EXPAND` mantém o máximo em `0,1` — *a lei do `EXPAND` não tem braço de
alavanca, logo só a contagem de anéis se move* (`grade_{dobrar,expandir}_constante_origem1`).

Faixa pública: `0,0 … 30,0`; omissão `0,0`.

---

## §9 — Fase F: o avanço `s` que a mão produz

### §9.1 — O arrasto e a sua projecção (M, cruzado em 4 modos)

O vector de arrasto `Δ` é medido **desde o ponto do pen-down** (âncora do gesto), em espaço de
objecto, no plano paralelo ao ecrã que passa pelo ponto de contacto. ⚠️ **É o arrasto TOTAL**, não
o incremento desde o evento anterior; o ponto que o editor usa para «onde está o pincel» fica
**preso** no contacto inicial durante todo o traço.

Seja **`n̂`** o unitário do **ponto de contacto para o ponto-origem** (§7.4) — isto é, **apontando
para DENTRO da peça**. Então

```
s  =  Δ · n̂                                                                (M)
```

⚠️⚠️ **O sinal é o ponto em que se erra, e está MEDIDO nos dois sentidos.** Arrastar **para dentro**
da peça dá `s > 0`; arrastar **para fora** dá `s < 0`. Consequências medidas, com fixtures dos dois
lados (`sinal_*`, construídas para isto):

| arrasto | `GRAB` | `EXPAND` | `INFLATE` | `BEND` |
|---|---|---|---|---|
| para **fora** (`−ŷ`) | `−0,1 ŷ` (segue a mão) | `+0,1 ŷ` | `−0,1 ẑ` | `(0, +0,2159, −0,2972)` |
| para **dentro** (`+ŷ`) | `+0,1 ŷ` (segue a mão) | `−0,1 ŷ` | `+0,1 ẑ` | `(0, +0,2159, +0,2972)` |

⭐ **O `GRAB` é o único que segue a mão literalmente.** Os outros quatro são conduzidos pelo escalar
`s`, e **o `EXPAND` anda ao CONTRÁRIO da componente de `n̂`**: puxar para fora da peça encolhe a
borda para dentro. *Não é defeito — é a lei, medida nos dois sentidos.*

⭐ **Arrasto TANGENTE ao contorno não faz nada.** `grade_dobrar_tangencial` (arrasto `0,1 x̂`, ao
longo da borda): **`0` vértices movidos**. É a prova directa de que só a projecção conta.

### §9.2 — A força do traço (M)

```
força_do_traço = Força_do_pincel × esbatimento_de_simetria
```

⚠️ **Sem pressão e sem inversão de sinal**, e **sem elevar ao quadrado** a Força (vários pincéis do
alvo usam `Força²`; **este não**). Medido:
- Força `0,5` ⇒ deslocamentos exactamente metade (`grade_agarrar_constante_forca05`);
- pressão `0,3` ⇒ **saída idêntica** à de pressão `1` (`grade_agarrar_constante_pressao03`);
- `esbatimento_de_simetria` só é `≠ 1` com simetria ligada **e** a opção de esbatimento ligada
  (§12.3).

### §9.3 — O que o modificador de inversão (Ctrl) faz — e o que NÃO faz

⚠️⚠️ **Ele NÃO inverte o sinal da deformação.** Medido: `grade_agarrar_constante_inverter` é
**idêntico** ao traço normal.

O que ele faz é **encaixar o ÂNGULO** nos modos `BEND` e `TWIST` (§10.1, §10.5): o factor de ângulo
é truncado a **décimos**,

```
factor_encaixado = ⌊factor × 10⌋ / 10                                       (F)
```

Medido: `grade_dobrar_constante_curto` (arrasto `0,07`) dá máximo `0,266112`, e o mesmo traço com
inversão dá `0,283744` — *um valor DIFERENTE e MAIOR*, que é a assinatura de uma truncagem sobre um
factor **negativo** (aqui `s < 0`, logo `⌊·⌋` afasta-se de zero). ⛔ *Quem implementar isto como
«arredonda para o décimo mais próximo» diverge; e quem o implementar como `abs` diverge no outro
sentido.*

---

## §10 — Fase G: as seis leis de deformação

Notação: `P₀(v)` = posição de `v` **no repouso do traço** (§11.1); `w = peso(v)` (§8);
`F = força_do_traço` (§9.2); `s` = avanço (§9.1); `R` = raio do pincel (o dinâmico, §10.7).

### §10.1 — `BEND` — roda a coluna à volta de um eixo POR-COLUNA

Para cada vértice da cadeia, um par (**ponto de rotação**, **eixo**) é lido **do anel `K`** da sua
coluna e herdado por toda a coluna:

```
ponto(coluna) = P₀(vértice do anel K dessa coluna)
eixo(coluna)  = normalizar( (P₀(cadeia) − P₀(anel K)) × normal_do_vértice_do_anel_K )      (F)
```

O ângulo é **partilhado** pelo traço todo e o peso gradua-o:

```
ângulo_total = π × F × s / R      (e, com Ctrl, o factor F·s/R é encaixado — §9.3)
P(v) = ponto(coluna(v)) + rodar( P₀(v) − ponto(coluna(v)), eixo(coluna(v)), ângulo_total × w )
```

⭐ **Verificado ao 7.º decimal:** grelha, `K = 5`, aresta `0,0625` ⇒ braço `0,3125`;
`|ângulo| = π × 0,1/0,25 = 1,25664`; corda `= 2 × 0,3125 × sin(0,62832) = 0,3673655`, e a fixture
`grade_dobrar_constante` mede **`0,36736549`**. E o vector inteiro do vértice da borda bate nas três
componentes nos dois sentidos de arrasto (§9.1).

⚠️ **A normal usada é a do vértice do anel `K`**, não a do vértice que se move nem a da borda. Os
autores tiveram um defeito exactamente aqui (D, 2024-08-01: *a deformação usava as normais
erradas*) — ⇒ **gate nosso que nomeie de que vértice sai a normal**.

⚠️ O manual diz que o `BEND` *roda à volta do eixo Y local* (D). **Isso é a descrição do efeito, não
a lei**: o eixo é derivado por-coluna pela fórmula acima e só coincide com um eixo do objecto em
casos alinhados. ⇒ **não implemente o texto do manual.**

### §10.2 — `EXPAND` — desliza ao longo de uma direcção POR-COLUNA

```
direcção(coluna) = normalizar( P₀(cadeia) − P₀(anel K da coluna) )      (aponta para FORA)
P(v) = P₀(v) + direcção(coluna(v)) × (F × s × w)                                   (F)
```

M: deslocamento **exactamente linear** no arrasto — os oito degraus de
`grade_expandir_constante.porpasso` sobem de `0,014286` em `0,014286` (`= 0,1/7`), sem desvio.

### §10.3 — `INFLATE` — anda pela normal do próprio vértice

```
P(v) = P₀(v) + normal_de_repouso(v) × (F × s × w)                                  (F)
```

⚠️ A normal é a **do repouso do traço**, não a recalculada no quadro. M: na grelha plana o
deslocamento é puramente `±ẑ` e igual a `0,1` (`grade_inflar_constante`, `sinal_inflar_dentro`).

### §10.4 — `GRAB` — translada pelo arrasto inteiro

```
P(v) = P₀(v) + Δ × (F × w)                                                         (F)
```

⭐ **O único modo que usa o vector `Δ` e não o escalar `s`** — e portanto o único cujo movimento
segue a mão em qualquer direcção, inclusive a tangente. (⚠️ mas o arrasto tangente continua a não
produzir nada nos outros cinco — §9.1.)

### §10.5 — `TWIST` — roda à volta de UM eixo, partilhado

```
eixo   = normalizar( ponto-origem − P₀(âncora) )            (da borda para dentro)
centro = média das P₀ de TODOS os vértices da cadeia
P(v)   = rodar_em_torno_de_eixo( P₀(v), centro, eixo, ângulo_total × w )           (F)
```

com `ângulo_total` como no `BEND` (§10.1), Ctrl incluído.

⭐ **Verificado:** na grelha, `centro = (0, −1, 0)` e `eixo = +ŷ`; um vértice a `1,0` do eixo com
`|ângulo| = 1,25664` dá corda `2 sin(0,62832) = 1,1755706`, e a fixture mede **`1,17556958`**.
⭐⭐ E o controlo que prova o eixo: `grade_torcer_constante` move **`160`** de `165` — os **`5`**
que não se movem são exactamente a coluna que está **em cima do eixo**.

⚠️ O manual diz *eixo Z local* (D); vale a mesma ressalva da §10.1.

### §10.6 — `SMOOTH` — média SÓ com os vizinhos do MESMO anel

```
média(v) = média das posições ACTUAIS dos vizinhos u de v com anel(u) == anel(v)
se v não tiver nenhum tal vizinho:  peso(v) := 0
P(v) = posição_actual(v) + (média(v) − posição_actual(v)) × (F × w)                 (F)
```

⭐⭐ **Três coisas separam este modo dos outros cinco, e as três estão medidas:**

1. ⭐⭐ **Ele não usa o avanço `s`** — a força do traço é `F` directamente. ⇒ ele actua **já no
   primeiro passo do traço**, quando todos os outros estão parados (§14.1). **A prova é mais forte
   do que isso: as cinco fixtures de `SMOOTH` têm arrasto ZERO** (`0 0 0` no cabeçalho) e mesmo
   assim deformam — `grade_suavizar_constante` move `72` vértices com o cursor **parado**. Nos
   outros cinco modos, arrasto zero ⇒ `s = 0` ⇒ nem um vértice se mexe.
2. **Ele lê a posição ACTUAL, não a de repouso** ⇒ **acumula ao longo do traço**. M, mesma peça e
   mesmo arrasto, só variando o número de passos: `1 → 0,0681`, `2 → 0,0640`, `8 → 0,1372`,
   `16 → 0,1966` (soma dos deslocamentos `0,88 → 0,76 → 1,95 → 3,35`). ⛔ *Não é função do arrasto
   total*; é função do NÚMERO DE EVENTOS — veja §14.3.
3. **Ele alisa ao LONGO do contorno, não através dele** (D: *só os vértices paralelos ao contorno
   entram*). M, grelha regular plana: os vértices do meio de cada anel **não se movem** (a média dos
   dois vizinhos do mesmo anel é o próprio vértice); só as PONTAS das fileiras andam, e andam em
   `x̂` — a fileira contrai-se sobre si mesma (`grade_suavizar_constante`: `72` movidos, `u` na
   ponta `+0,1367 x̂`, `0` no meio).

⛔ **CERCA DE PATENTE, nomeada (§8.1 do ledger):** ⛔ **nunca** acrescentar a este modo uma
*estimativa de volume + reposição por inflação* — é a reivindicação de uma patente VIVA de
terceiros (US 9 830 743 B2, até 2034). A média de vizinhos do mesmo anel, sem termo de volume, fica
fora dela.

### §10.7 — ⚠️ Os dois raios, outra vez

`R` em `ângulo = π F s / R` (§10.1, §10.5) é o raio **dinâmico** do traço; o raio que limita a busca
da âncora (§5.1), a queda no contorno (§8.3) e a propagação (§7.3, via offset) é o **inicial**. Com
a pressão a modular o tamanho, os dois divergem dentro do mesmo traço.

---

## §11 — Fase H: aplicar

### §11.1 — O «repouso» é o do TRAÇO

As posições `P₀` são as do **início do traço** — congeladas no pen-down, não as do quadro anterior.
⇒ cada passo recalcula a deformação TOTAL a partir do repouso, em vez de a acrescentar.
⛔ **Excepção: o `SMOOTH`** (§10.6.2).

⚠️ E, ao contrário de outros pincéis que usam repouso, este **não** repõe as posições a cada passo
do traço — a reposição estragaria o alvo de simulação (§16.3). ⇒ quem implementar tem de garantir
que o resultado é o mesmo **sem** repor, o que a lei acima já dá.

### §11.2 — ⚠️⚠️ Peso zero tem de virar translação ZERO, explicitamente

A posição nova `P(v)` é convertida em translação `P(v) − posição_actual(v)` e aplicada. **Um
vértice de peso `0` tem de ser forçado a translação zero**, e não basta que o peso zere o termo da
lei: nas leis que rodam à volta de um ponto (`BEND`, `TWIST`), `P(v)` com ângulo `0` é `P₀(v)`, que
**não** é a posição actual se um passo anterior (ou uma passagem de simetria) já a moveu ⇒ a
translação seria `P₀(v) − actual(v)`, **desfazendo** o trabalho já feito.

⭐ Os autores pagaram exactamente este defeito, e a descrição pública dele (D, 2026-03-04, defeito
#154678) é a melhor prova de que a cura é obrigatória: *a primeira passagem aplica uma translação
que as seguintes desfazem*. Eles também registam que **o `SMOOTH` é imune**, por ler a posição
actual. ⇒ **gate nosso com duas passagens de simetria sobre vértices de peso zero.**

### §11.3 — Recorte final

Depois do filtro, a translação passa pelo recorte/trava de eixos do editor (as opções de bloqueio
`X/Y/Z` e o espelho de recorte). N: manter, é ortogonal a este pincel.

---

## §12 — Simetria

### §12.1 — O modelo

Cada eixo de simetria activo duplica as **passagens**. Cada passagem:
1. reflecte o ponto de contacto e o arrasto para a sua região;
2. **refaz as fases A–E do zero** para essa região — incluindo escolher uma âncora nova: na
   passagem espelhada toma-se o **vértice mais próximo** do ponto reflectido, dentro do raio
   (⚠️ pode não existir ⇒ aquela passagem simplesmente não deforma nada);
3. aplica G/H.

### §12.2 — ⚠️ O filtro por REGIÃO é obrigatório quando a queda não é `CONSTANT`

Cada passagem só pode tocar vértices que estejam **na sua região de simetria**. A regra (F):
para cada eixo espelhado `i`, o vértice é aceite se `v[i] × pivot[i] ≥ 0`, e quando `pivot[i] = 0`
aceita-se só `v[i] ≤ 0`.

⭐ Com `CONSTANT`, as duas passagens escrevem o mesmo valor e a ausência do filtro não se nota; com
`RADIUS`/`LOOP` elas escrevem valores diferentes e a segunda **apaga** a primeira. Os autores
acharam isto no dia seguinte ao lançamento do pincel (D, 2020-08-11). ⇒ **o gate desta lei usa
`RADIUS`, nunca `CONSTANT`.**

### §12.3 — Esbatimento

Com a opção de esbatimento ligada, a força do traço é dividida pela sobreposição das passagens. M:
`grade_agarrar_raio_simetriax_perto` dá máximo `0,1`; o mesmo com esbatimento dá **`0,066344`**.

### §12.4 — Medido

| fixture | efeito |
|---|---|
| `grade_dobrar_constante_simetriax` | máximo `0,304098` contra `0,367365` sem simetria — *a simetria MUDA o resultado mesmo com o traço em cima do plano de espelho*, porque cada metade é deformada pela sua própria passagem |
| `grade_agarrar_raio_simetriax` | `70` movidos (dois troços de `35`) |
| `grade_deslocada_agarrar_raio_simetriax_r0{15,20}` | `30` e `56` movidos — a passagem espelhada acha ou não acha vértice conforme o raio, numa grelha deslocada meia célula |

### §12.5 — ⛔ Simetria RADIAL: defeito conhecido e ABERTO no alvo

O pincel **ignora** a simetria radial e, com ela ligada, deforma em direcção diferente do traço
(D, defeito público #141625, ABERTO, *«provavelmente nunca funcionou»*). ⇒ **N: não copiar. Ou
implementamos radial a sério, ou a recusamos em voz alta — ⛔ nunca em silêncio.**

---

## §13 — Casos de borda e recusas, um a um

| # | situação | o que o alvo FAZ | proveniência |
|---|---|---|---|
| 1 | malha fechada (sem aresta de borda) | nada se move, sem aviso | M `esfera_agarrar_constante` |
| 2 | cursor numa quina da borda | **traço inteiro recusado** | M `grade_canto_agarrar_constante` |
| 3 | tira de uma fileira | **traço inteiro recusado** | M `faixa_agarrar_constante` |
| 4 | não há borda dentro do raio | nada se move, e não há pré-visualização | leitura + §17 |
| 5 | primeiro passo do traço | **nada se move** nos 5 modos conduzidos por `s`; o `SMOOTH` **move** | M (§14.1) |
| 6 | arrasto tangente ao contorno | nada se move (excepto `GRAB`) | M `grade_dobrar_tangencial` |
| 7 | **a propagação consome a malha inteira** | ver §13.1 ⬇️ | M + D |
| 8 | malha triangulada | funciona, com resultado **muito** diferente | M (§13.2) |
| 9 | borda fechada (aro) | funciona, sem tratamento especial | M `tubo_*` (§6.3) |

### §13.1 — ⛔⛔ Quando o alcance excede a peça, o `EXPAND` deixa de deformar

Se a propagação parar por **acabarem os vértices** (e não por exceder o raio), `K` fica **maior do
que o anel mais fundo que existe** ⇒ **nenhum vértice está no anel `K`** ⇒ os dados que só são
semeados ali (a direcção do `EXPAND`, o par ponto/eixo do `BEND`) ficam por preencher.

**Medido**, grelha `4×4` com `boundary_offset = 2` (raio de propagação `0,75` sobre uma peça de
`0,4`):

| modo | movidos | fixture |
|---|---|---|
| `EXPAND` | **`0`** | `grade_pequena_expandir_origem2` |
| `BEND` | `24` de `25`, máximo `0,054` | `grade_pequena_dobrar_origem2` |

⚠️ **Os dois modos partilham a mesma propagação e respondem diferente** — a espec regista o facto e
**não** a explicação (as duas leis semeiam dados de forma diferente a partir do anel `K`, e o
comportamento sob dados por-preencher difere).

⚠️ Este é um defeito **ABERTO** do alvo: *o pincel parte-se quando é maior do que a geometria
disponível*, e o pedido do relator é **limitar o ponto-origem à peça** (D, defeito público #146793;
há também um commit de 2020-10-18 que curou **metade** disto — o caso em que o alcance nem sequer
era escrito). ⇒ ⭐ **N: a nossa implementação ATA o alcance ao anel mais fundo que de facto existe**
(`K := min(K, anel_máximo_atribuído)`), o que torna as duas leis bem definidas e faz o `EXPAND`
deformar em vez de ficar mudo. **Divergência declarada, com gate que a nomeia** e com a fixture
acima como o lado do alvo.

### §13.2 — Topologia triangulada (M)

A mesma grelha, triangulada por leque, com o mesmo traço:

| peça | `BEND` máximo | movidos | `EXPAND` máximo | movidos |
|---|---|---|---|---|
| quads | `0,367365` | `165` | `0,100000` | `165` |
| triângulos | **`0,960244`** (`2,6×`) | `128` | `0,098995` | `119` |

⇒ o manual avisa que triângulos dão *resultados imprevisíveis* (D) e **o número diz de quanto**: o
`BEND` amplifica `2,6×`, porque a triangulação muda o grau dos vértices e portanto a profundidade
em anéis — o braço de alavanca da §10.1 cresce. ⚠️ **Não é ruído: é a lei a ler outra topologia.**
⇒ um gate de paridade que só use grelhas de quads não mede nada sobre isto.

### §13.3 — A malha CURVA (M)

Cúpula aberta (`385` v, borda no equador): `96 = 3 × 32` movidos; `BEND` `0,155319`,
`EXPAND`/`INFLATE` `0,038268` — iguais entre si, como a álgebra manda (os dois escalam `F·s·w` e
aqui a normal e a direcção de deslize quase coincidem).

### §13.4 — Determinismo (M)

⭐ **`dispersao_entre_realizacoes = 0,00000000` em TODAS as fixtures com duas realizações.** Duas
corridas do mesmo traço sobre a mesma malha dão bits idênticos. ⇒ a lei não tem aleatoriedade
nenhuma, e a barra de paridade não precisa de tolerância por ruído do alvo.

### §13.5 — Precisão

O alvo calcula e guarda em `f32`. Os nossos dumps foram gravados em `f64` a partir de valores
`f32` (round-trip exacto, verificado pelo montador: as malhas de repouso batem ao bit). ⇒ **a barra
de paridade é derivável de `f32`** (§19.3), nunca um epsilon de conforto.

### §13.6 — O que diverge entre representações de malha do alvo

Na malha base, a visibilidade das faces entra no teste «isto é borda?»; nas outras duas
representações **não entra** (o próprio alvo marca isso como limitação por resolver). ⇒ como só
implementamos a malha base (§1), a nossa regra é a **com** visibilidade, e não há escolha a fazer.

---

## §14 — Relógio, eventos e o que depende da taxa de amostragem

### §14.1 — O primeiro passo não deforma (M)

Nos cinco modos conduzidos por `s`, o arrasto é `0` no primeiro evento do traço ⇒ deslocamento
`0`. Medido no fatiamento: `passos = 1 ⇒ máximo 0,000000` em `BEND`, `EXPAND` e `TWIST`.
⛔ O `SMOOTH` **não** obedece a isto (§10.6.1) — e a fixture que o prova tem arrasto **zero**, não
apenas um passo.

⚠️ Consequência de produto: **um toque sem arrasto não faz nada** em cinco dos seis modos. Se o
nosso pincel tiver um gesto de «clique» (sem arrasto), ele é **mudo** por construção — decida se
isso é o que se quer, e diga-o na UI, ⛔ nunca em silêncio.

### §14.2 — ⭐⭐ O resultado NÃO depende do caminho nem do número de eventos

Medido de duas maneiras independentes:
- `grade_dobrar_constante_2passos` (2 eventos) e `grade_dobrar_constante` (8 eventos), **mesmo
  arrasto total**: máximo `0,3673655` nos dois, **ao 7.º decimal**;
- a prova de fatiamento das sete fixturas `*.porpasso`: `prova_do_fatiamento = 0,00000000` — a
  corrida truncada ao último passo reproduz a corrida cheia **ao bit**.

⇒ **a deformação é função do arrasto TOTAL.** Isto é o que torna o pincel testável sem simular
tempo — e é consequência directa da §3 (a estrutura A–E é fotografada) e da §11.1 (repouso do
traço).

### §14.3 — ⛔ A excepção que estraga a regra: o `SMOOTH`

Ele é função do **número de eventos** (§10.6.2). ⇒ **qualquer gate de paridade do `SMOOTH` tem de
fixar a contagem de passos**, e uma implementação nossa que amostre o caminho com outra densidade
diverge **por construção**, mesmo estando certa. ⭐ É o mesmo mecanismo que este repo já pagou seis
vezes no Painter: *um produto por-evento depende da taxa de amostragem*. ⇒ **N: considerar expor o
alisamento como número de iterações determinístico**, e registar a divergência.

### §14.4 — Custo medido

Grelha plana, `BEND`, `8` eventos, máquina do dono, tempo do traço INTEIRO:

| vértices | movidos | s / traço | s / evento |
|---|---|---|---|
| `16 641` | `2 193` | `0,0016` | `0,00020` |
| `66 049` | `8 481` | `0,0041` | `0,00051` |
| `263 169` | `33 345` | `0,0079` | `0,00099` |

⇒ **~`1 ms` por evento a `263 k` vértices** — `6 %` de um quadro de `16,7 ms`. Escala **sublinear**
no total de vértices (`16×` os vértices custa `4,9×`) porque o trabalho é proporcional aos vértices
**alcançados**, não à malha. `GRAB` e `SMOOTH` custam `~1,3×` o `BEND` na mesma peça
(`0,00133`/`0,00130` contra `0,00099`).

⚠️ **Mas o pen-down é O(malha):** as fases A–E varrem a malha inteira (o censo de bordas, e três
arrays do tamanho do número de vértices). ⛔ **Não medimos o pen-down isolado** — o relógio acima é
do traço todo, e a `263 k` ele já o inclui. ⇒ item aberto (§20).

⚠️ O alvo declara por escrito que este pincel **processa a árvore espacial INTEIRA** (não consegue
saber de antemão onde a deformação vai cair) e marca isso como optimização por fazer (D). ⇒ a nossa
implementação pode fazer melhor: depois da fase D o conjunto alcançado **é conhecido**.

---

## §15 — Superfície de opções PÚBLICA (o que a UI deve oferecer)

| opção | valores | omissão | onde entra |
|---|---|---|---|
| `boundary_deform_type` | `BEND` `EXPAND` `INFLATE` `GRAB` `TWIST` `SMOOTH` | **`BEND`** | §10 |
| `boundary_falloff_type` | `CONSTANT` `RADIUS` `LOOP` `LOOP_INVERT` | **`CONSTANT`** | §8.3 |
| `boundary_offset` | `0,0 … 30,0` | **`0,0`** | §8.4 |
| `deform_target` | geometria · simulação de pano | geometria | §16.3 |
| Força | a do pincel | — | §9.2 (⚠️ **linear**, sem quadrado, sem pressão) |
| curva de queda | os presets + curva autorada | `SMOOTH` | §8.1 |
| Ctrl (inversão) | — | — | §9.3 (**encaixe de ângulo**, não inversão) |

⚠️ **O painel do alvo tem EXACTAMENTE estes quatro controlos próprios** (alvo de deformação, tipo de
deformação, queda no contorno, offset) e nada mais — o resto vem dos ajustes gerais do pincel. ⇒
*um painel nosso com um quinto knob próprio está a inventar, e um com três está a esconder.*

---

## §16 — Três coisas que o pincel herda do editor

### §16.1 — Ele precisa de TODOS os nós da árvore espacial
Ver §14.4.

### §16.2 — Ele desfaz-se como os outros
O registo de desfazer é o do editor (instantâneo de posições por nó), sem nada de próprio.

### §16.3 — Alvo de deformação = simulação de pano (M, parcial)

Com o alvo posto em simulação, a lei do modo **não escreve posições**: escreve as posições-alvo das
restrições do solver de pano, e é o solver que produz o resultado final. ⭐ Medido:
`grade_dobrar_constante_pano_{local,dinamica}` movem **`1 089`** vértices (a malha inteira, porque o
solver espalha) com máximo `0,080781`, e as duas áreas de simulação dão **o mesmo resultado** nesta
peça.
⚠️ **O solver de pano NÃO é desta espec** — ele já foi especificado pela obra do pincel de tecido
desta mesma linha. Aqui fica só a costura: *o modo escreve alvo, não posição*.

---

## §17 — A pré-visualização no cursor

Enquanto o cursor passeia sobre a malha (sem carregar), o alvo:
1. corre as fases **A–D** a cada quadro, com o raio do cursor;
2. **desenha a cadeia** — os segmentos de borda alcançados — na cor do cursor do pincel;
3. **desenha uma linha branca** do vértice âncora até ao **ponto-origem** (§7.4), que é o que diz ao
   artista *até onde a deformação vai chegar*;
4. **desenha um ponto** no ponto-origem.

⚠️ Se não houver contorno ao alcance, **não se desenha nada** (o alvo trata a ausência como normal,
não como erro). ⇒ **a pré-visualização é a resposta à recusa da §13.1#4**, e é ela que faz as
recusas da §5.3 serem legíveis em vez de misteriosas.

⚠️⚠️ **Isto é `O(malha)` por QUADRO de sobrevoo** (§14.4) — o alvo recalcula tudo a cada quadro.
⇒ **N: medir antes de copiar**; um cache por (vértice activo, raio) é óbvio e o alvo não o tem.

⭐⭐ **E há uma armadilha de harness aqui que nos mordeu:** o ponto de superfície sob o cursor só
existe depois de um quadro de sobrevoo ter corrido. Um traço scriptado que chegue antes lê lixo —
mediu-se `NaN` em ~20 % das corridas antes da cura, **alternando entre realizações da mesma
corrida**. ⛔ *Não é comportamento do produto* (um artista passa sempre o cursor antes de carregar),
mas **é** comportamento de qualquer teste nosso que dispare o pincel sem um quadro de sobrevoo.

---

## §18 — Defeitos ABERTOS do alvo (não os copie sem decidir)

| # | o defeito | o que fazemos |
|---|---|---|
| #146793 | parte-se quando o pincel é maior que a geometria | **curamos** — §13.1, divergência declarada |
| #141625 | ignora a simetria radial e deforma na direcção errada | **recusar em voz alta** ou implementar — §12.5 |
| #125350 | borda em laço no `TWIST` nunca funcionou; o código morto foi removido | **não construir** — §6.3 |
| #131122 | com alvo de pano, a simetria só afecta um lado | herdado do solver de pano; fora desta espec |

---

## §19 — Vectores de teste e a barra de paridade

### §19.1 — O corpus

`docs/3D/cleanroom/fixtures/boundary/` — **61** traços (`*.deformado.txt.gz`), **7** séries por-passo
(`*.porpasso.txt.gz`) e **9** malhas de repouso (`*.repouso.txt.gz`), com cabeçalho por fixture. Ver
o `README.md` de lá para a proveniência e o formato.

### §19.2 — Cobertura, por família

| família | quantas | o que fecha |
|---|---|---|
| seis modos × queda `CONSTANT` | 6 | §10 |
| quatro quedas no contorno | 4 | §8.3 |
| quatro curvas de queda | 4 | §8.1 |
| sinal do avanço, os dois sentidos | 6 | §9.1 ⭐ |
| força · pressão · inversão · máscara | 5 | §9.2, §9.3, §8.2 |
| offset da origem | 5 | §8.4, §13.1 |
| recusas | 4 | §5.3 |
| simetria (+ esbatimento, grelha deslocada) | 5 | §12 |
| topologia (triângulos, tubo, cúpula, ruidosa) | 9 | §13.2, §13.3 |
| contagem de passos / fatiamento | 7 | §14.1, §14.2, §14.3 |
| alvo de pano | 2 | §16.3 |

⚠️ As linhas não somam `68` porque **uma fixture serve mais do que uma família** (a de simetria com
esbatimento conta nas duas, a do offset na peça pequena conta no offset e na §13.1). A régua que
diz o que cada fixture tem de facto no cabeçalho é
[`confere_cabecalhos.py`](../fixtures/boundary/confere_cabecalhos.py) — ⛔ leia o cabeçalho da
fixture, nunca esta tabela.

### §19.3 — A barra, DERIVADA (N, do §13.5)

O alvo guarda posições em `f32`. Sobre as nossas peças, `|posição| ≤ 2`, o ULP de `f32` é
`2⁻²³ × 2 = 2,4e-7`. Uma cadeia de ~`20` operações (normalizar, rodar, somar) acumula, em pior
caso, `~20 × ULP ≈ 5e-6`. ⇒ **barra sugerida: `1e-5` em posição absoluta**, com estas ressalvas:

- ⛔ **NÃO prometer paridade bit-a-bit** (ADR-0162 e SKILL §5: num clean-room T2 ela é
  indesejável como narrativa probatória, mesmo sendo lícito o comportamento coincidir);
- ⚠️ o `SMOOTH` precisa da contagem de passos fixada (§14.3) ou fica fora da barra por construção;
- ⚠️ **a barra tem de incluir o lado APROVADO**: antes de a fixar, corra-a sobre as fixtures do
  próprio alvo em duas realizações — medimos `0,0` de dispersão (§13.4), logo qualquer barra
  positiva já é folgada em relação ao ruído do alvo, e uma barra que reprove o próprio alvo é um
  defeito da barra (⚠️ a obra do tecido desta linha retirou **duas** barras por isto).

### §19.4 — Gates que a espec pede pelo nome

1. a malha fechada não deforma · a quina recusa · a tira de uma fileira recusa (§5.3);
2. o sujeito dos testes de recusa é o vértice **sob o cursor** (§5.2);
3. a curva `CONSTANT` **também** dá zero no anel `K` (§8.1);
4. a coluna da âncora é **isenta** da queda no contorno, nos quatro modos (§8.3);
5. o offset move os anéis e **não** move o troço de borda (§8.4);
6. arrasto tangente ⇒ zero, em cinco dos seis modos (§9.1);
7. o sinal do `EXPAND` é **oposto** ao do `GRAB` na direcção de `n̂` (§9.1) ⭐;
8. a inversão **não** inverte: ela **encaixa o ângulo** (§9.3);
9. a normal do `BEND` sai do vértice do anel `K` (§10.1);
10. o eixo do `TWIST` deixa **a coluna em cima dele** parada (§10.5);
11. `SMOOTH` move ao longo do anel e **não** através dele (§10.6.3);
12. peso zero ⇒ translação zero, provado com **duas passagens de simetria** (§11.2) ⭐;
13. o filtro de região de simetria, provado com queda `RADIUS` e nunca `CONSTANT` (§12.2);
14. resultado invariante ao número de eventos, nos cinco modos conduzidos por `s` (§14.2);
15. o alcance atado à peça faz o `EXPAND` deformar onde o alvo fica mudo (§13.1, divergência).

---

## §20 — O que esta espec NÃO fecha

1. **Geometria escondida** como contorno (§4.2) — documentado, **sem fixture**.
2. **Automáscara** (§8.2) — o harness desligou-a em todas as corridas.
3. **Curva de queda AUTORADA** — só os quatro presets foram medidos.
4. **Custo do pen-down isolado** (§14.4) e da pré-visualização por quadro (§17).
5. **Simetria radial** (§12.5) — defeito aberto do alvo; decisão nossa por tomar.
6. **O solver de pano** (§16.3) — outra espec desta linha.
7. As duas representações alternativas de malha (§1).

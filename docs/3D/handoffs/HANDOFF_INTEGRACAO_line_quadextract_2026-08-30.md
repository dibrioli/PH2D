# HANDOFF — `line/quadextract` (2026-08-30) — a RÉGUA LOCAL, e o `Follow Curvature` vivo

> Continuação de [29/08](HANDOFF_INTEGRACAO_line_quadextract_2026-08-29.md). A linha reabriu
> sobre o `main` integrado, com o stack subido (`wgpu` 29 · `vello` 0.10 · `rapier` 0.35) e o
> cache de compilação mudado de disco.
>
> ⚠️ **Método novo, por ordem do Enio (30/08):** *«não em micro passos; cada etapa termina num
> smoke; se tiver complexidade, é auditada antes de sugerir o smoke.»*

---

## §1 — O que a reabertura mediu, antes de tocar em código

| | |
|---|---|
| a linha no `main` | ⭐ tudo integrado; `line/quadextract` = `main` (`066b4f92e`), árvore limpa |
| stack novo | ⭐ workspace compila a frio em `2 min 31`; **4 078** testes do shell, `0` falhas |
| o botão na peça do artista | ⭐ `χ = 2` · `0` bordo · `0` não-manifold · `100 %` quads, e **4,7× mais rápido** (`27,8 s → 5,9 s`) |
| ⛔ `ph2d-quadbench/` | **não sobreviveu à mudança de disco** — nunca esteve no git (o oráculo é restrito). O corpus e as fases gravadas dele **não existem nesta máquina**; as sondas que o lêem falham **alto** (`expect`), não em silêncio |

---

## §2 — ⭐⭐⭐ A RÉGUA LOCAL (`ph2d_quadfill::local`)

O report de 29-30/08 não era visto por régua nenhuma: na mesma peça o botão relatava
`χ = 2` · `0` bordo · `0` não-manifold · `1` ilha · `100 %` quads · `>60° = 0`. **Toda régua verde.**

A `shape` mede os **cantos** de cada face e resume em percentis. A `local` mede o que os cantos
não podem ver, **e onde**:

| coluna | o que apanha |
|---|---|
| `warp_deg` | o quad **não-plano** (ângulo entre as normais das duas metades) |
| `kind` | **gravata** — o quad que se auto-intersecta (contagem de sinal sobre a normal de Newell) |
| `squareness` | a **lasca** (área ÷ aresta média²) |
| `radial` | **onde** — `0` é o centro, `1` é a ponta |

⚠️ **Cada gate traz o CONTROLO**: a mesma fixtura medida pela `shape`, a provar que ela fica
**verde**. Uma gravata mede `45°` de desvio de canto — abaixo da barra de `60°` — e aspecto `√2`.

### ⛔ Uma mutação sobreviveu e derrubou uma afirmação minha

O doc dizia que o `max` sobre as duas diagonais existe porque *«uma sela é plana ao longo de uma
e torcida ao longo da outra»*. **Falso** — quatro pontos ou são coplanares ou não são:

| fixtura | diagonal `0–2` | diagonal `1–3` |
|---|---|---|
| sela | `109,47` | `109,47` |
| canto levantado | `63,20` | `70,25` |
| assimétrico | `68,67` | `60,19` |

A razão certa é outra (*o número não pode depender da diagonal que o renderizador triangulou*), e
⚠️ **o gate precisou de DUAS fixturas, uma de cada ORDEM**: com só a assimétrica — onde `0–2` é a
maior — a mutação *«olha só a `0–2`»* sobrevivia. *Uma fixtura em que o máximo calha ser o
primeiro argumento não distingue `max` de «o primeiro».*

---

## §3 — ⛔⛔⛔ TRÊS hipóteses REFUTADAS pela medição

Medido nas três malhas que o artista mandou:

| malha | faces | defeitos | gravatas | na ponta |
|---|---|---|---|---|
| a escultura dele (entrada) | `15 275` | `0,47 %` | `4` | `2,83 %` |
| ⭐ **a nossa saída** | `15 426` | **`0,11 %`** | `1` | `0,15 %` |
| Blender / QRemeshify | `8 291` | `0,42 %` | `1` | `0,20 %` |

⇒ **a nossa malha é a mais limpa das três.** Torção, gravata e lasca **não** são o mecanismo.

E a quarta hipótese — a **orientação**, que este handoff nomeia como *«a que se lê literalmente
como buraco»* (face virada renderiza pelo lado de dentro e sai preta) — dá **`0` arestas viradas
nas três malhas**.

---

## §4 — ⭐⭐⭐ O QUE É: a densidade RADIAL, com alvo DERIVADO

| aresta-equivalente mediana | corpo | **ponta** | razão |
|---|---|---|---|
| Blender (o que ele aprovou) | `0,0439` | `0,0261` | ⭐ **`0,59`** |
| **nós** | `0,0306` | `0,0361` | ⛔ **`1,18`** |

E a contagem: a ponta dele tem **`674`** faces, a nossa **`370`** — metade, numa malha `1,9×` maior.
⇒ *«as pontas têm menos densidade de faces e perdem detalhes»* (Enio, 28/08) é **literalmente
verdade**, e o alvo passa a ser **derivado** (`0,59`), não escolhido.

⚠️ **Isto não reabre as recusas de 28/08 sem qualificação:** elas foram medidas com o
`relief_density` (expoente global de `aresta ∼ curvatura`) — uma correlação sobre a peça inteira,
instrumento muito mais fraco que a casca radial.

---

## §5 — ⭐⭐ A TRANSFERÊNCIA, medida com UMA lei nos dois lados

`ph2d_quadfill::tip_body_ratio` tem **dois consumidores de propósito**: o **pedido** (um valor por
vértice da malha de trabalho) e a **entrega** (a raiz da área por face da saída). Domínios
diferentes, lei igual — *medi-los com duas funções daria dois números que ninguém pode dividir.*

Na peça do artista, `Detail 0,85`, `Follow Curvature = 1`:

| | razão ponta/corpo |
|---|---|
| o campo **PEDE** | ⭐ `0,486` — *melhor que o `0,59` do oráculo aprovado* |
| a cadeia **ENTREGA** | ⛔ `1,144` |

⇒ **o pedido está certo; a cadeia descarta-o e ainda inverte o sinal.**

---

## §6 — ⛔ A cura por ALISAMENTO: construída, medida, REFUTADA — e o que ela revelou

**Hipótese:** o G3 resolve `min ‖∇f − R/h‖²`, cuja condição de óptimo é `Δf = ∇·(R/h)` — um
**passa-baixo**. Um `h` de alta frequência sairia lavado. ⇒ alisar o pedido em log.

⛔ **Refutada, e pela razão certa:** `48` rondas movem o pedido de `0,486` para `0,502` — **`3 %`**.
*O pedido nunca foi de alta frequência*, e alisar não é o que move a densidade.

⚠️ **A 1.ª versão desta varredura não podia dizer isso:** a sonda do `PEDE` corria **antes** de
`smooth_in_log`, e as quatro corridas imprimiram o mesmo `0,486`. *Uma sonda posta antes do passo
que ela devia medir mede o passo anterior.*

⭐⭐ **Mas a varredura devolveu outra coisa: o alisamento compra a FORMA.** As faces com canto pior
que `60°` caem de `8` para `0` — melhor que a linha de base. *A adaptação passa a ser de graça em
qualidade.* ⇒ `SIZING_SMOOTH_ROUNDS = 8`, com a tabela no doc.

⚠️ **Um gate que já existia apanhou uma regressão minha:** a 1.ª versão alisava **depois** de a
contagem prevista estar calculada, e `a_densidade_segue_a_curvatura_sem_mudar_a_contagem` reprovou
com `−3,1 %` (barra `2 %`). *Normalizar por um número medido sobre um campo que já não existe é
normalizar para nada.* ⇒ o alisamento corre **antes** da contagem.

---

## §7 — ⭐⭐⭐ O RESULTADO: o `Follow Curvature` deixa de ser um knob morto

Peça do artista, `Detail 0,85`:

| | quads | razão ponta/corpo | faces na ponta | `>60°` | `χ` / bordo / não-manif. |
|---|---|---|---|---|---|
| knob **desligado** (hoje) | `9 188` | ⛔ `1,533` | `82` | `2` | `2 / 0 / 0` |
| ⭐ knob **ligado** | `8 257` | ⭐ **`1,062`** | ⭐ **`142`** | ⭐ **`0`** | `2 / 0 / 0` |

**A ponta passa de `53 %` mais grossa para `6 %`, com `+73 %` de faces lá, zero faces péssimas, e
a topologia intacta.** Preço: `−10 %` de quads (o slider redistribui, não cria).

### ⛔⛔ E porque ele NASCE DESLIGADO

Na fixtura sintética de espinhos (`espinhos:6`) o mesmo knob **parte a topologia**:

| | quads | razão | `>60°` | `χ` | bordo | não-manif. |
|---|---|---|---|---|---|---|
| desligado | `9 469` | `1,597` | ⭐ `0` | `2` | ⭐ `0` | ⭐ `0` |
| ligado | `8 122` | `1,064` | ⛔ `9` | `2` | ⛔ `4` | ⛔ `1` |

⇒ *uma fase medida sozinha pode melhorar e piorar o produto* — a lei que esta linha já pagou três
vezes. **O knob fica alcançável e o default fica em `0`**; a decisão é do dono do produto, com a
tabela na mão.

---

## §8 — ⏳ ABERTO

- ⛔ **A transferência continua rota** (`0,486` pedido → `1,062` entregue). O alisamento foi
  ilibado; o mecanismo é o do §8-quater de 28/08 (a projecção de mínimos quadrados), e a cura
  publicada é o **factor de escala conforme por construção** (`Δ log h` contra a curvatura de
  Gauss). ⚠️ **Agora ela tem régua para ser julgada** — `tip_body_ratio` nos dois lados.
- ⏳ **Porque o knob parte a topologia na fixtura sintética e não na peça do artista** — não
  medido. É o gatilho que decide se ele pode nascer ligado.
- ⏳ o `ph2d-quadbench` **não existe nesta máquina** (§1); toda comparação fase-a-fase com o
  oráculo está indisponível até ele voltar.
- ⏳ o motor **`Fast`** do menu continua a um clique, com a saída pior (herdado).

## §8-bis — ⭐ A AUDITORIA (antes do smoke, pela regra de 30/08) achou UM buraco

**Lente «a régua mente numa entrada degenerada?»** — e ela mentia:

⛔⛔ Quando **todos** os pontos estão à mesma distância do centro (um ponto só, ou uma peça sem
relevo nenhum), eles caem todos na casca `0`, a casca da ponta fica **vazia**, e
`tip_body_ratio` devolve **`0,0`** — que impresso ao lado do alvo `0,59` **lê-se como o melhor
resultado possível**.

⇒ o contrato passa a ser: **`n == 0` quer dizer NÃO MEDIDO**, está no doc da porta, tem gate
(`a_regua_recusa_entrada_degenerada_em_vez_de_inventar`) e **os dois consumidores olham a
contagem antes do número** (o da entrega imprime `NAO MEDIDA`).

⚠️ *É a terceira vez que esta linha paga «um zero de não-medido e um de perfeito são o mesmo
byte»* — as duas anteriores foram as réguas de valência (o balde que nunca era preenchido) e o
`edge_max` cego ao quad de `0,02 × 0,30`.

⭐ **A outra lente passou:** numa esfera lisa a razão dá `1,000000` — *se a régua desviasse com o
campo constante, todo desvio numa peça com pontas seria indistinguível do artefacto dela.*

## §8-ter — A prova de que o caminho de OMISSÃO não se mexeu

| `Follow Curvature = 0`, peça do artista, `Detail 0,85` | |
|---|---|
| quads | `9 188` (idêntico antes e depois da wave) |
| forma | aspecto p50 `1,06` p99 `1,38` · envies. p50 `3,4` p99 `21,6` · `>60° = 2` |
| razão ponta/corpo | `1,533` |

⚠️ **A única reprovada do portão é a flake de carga já nomeada no `CLAUDE.md` §5.0**
(`only_the_lower_row_breathes_and_it_moves_with_the_playhead`, demos de áudio): verde **3 de 3**
sozinha, e o diff toca **zero** ficheiros de áudio.

## §8-quater — ⛔⛔⛔ «PRATICAMENTE UMA REGRESSÃO» (Enio, 30/08, com foto) — e era

> *«Ainda muito ruim. Praticamente uma regressão. Faces completamente fora do lugar nas pontas.»*

**Reproduzido, e a causa é minha.** No `Detail` de **FÁBRICA** (`0,50`), na peça dele:

| `Follow Curvature` | quads | `χ` | **bordo** |
|---|---|---|---|
| `0` | `1 316` | `2` | `0` |
| `1` | `1 252` | ⛔ `1` | ⛔ **`4`** |

⚠️⚠️ **A wave que introduziu o knob mediu tudo a `Detail 0,85`, onde fica limpo.** *Afinar e
validar num ponto do slider que não é o de fábrica é medir a configuração que ninguém usa.*
⛔ E a fixtura sintética de espinhos **já tinha avisado** (bordo `0 → 4`); a leitura foi *«na peça
dele fica limpo»* — que era verdade **só naquele ponto**.

### ⭐ A cura: o campo adaptativo tem de poder PERDER

Ela tem a forma que a terceira tentativa da escada já tinha, e a mesma garantia: **se ainda há
furo e o knob estava ligado, corre-se a corrida outra vez sem o campo, e a decisão passa pelo mesmo
`worse`.** *A adaptação não pode piorar a escolha; só pode não ser escolhida.*

| | quads | `χ` | bordo | razão ponta/corpo |
|---|---|---|---|---|
| `Detail 0,50` · knob `0` | `1 316` | `2` | `0` | `1,060` |
| `Detail 0,50` · knob `1` | `1 316` | `2` | ⭐ **`0`** | `1,060` — *recua sozinho* |
| `Detail 0,85` · knob `1` | `8 257` | `2` | `0` | `1,062` — *mantém o ganho* |

⚠️ **Preço:** quando a recaída corre, o clique passa de `5,9 s` para `14,3 s`. É a mesma política
da terceira tentativa, e é **de graça** com o knob desligado ou com a saída já fechada.

### ⚠️ Três coisas que a cura precisou de aprender, e as três foram mutações

1. **A recaída pedia UMA candidata** (a do `w` vencedor) e a peça continuou com `4` bordo: *a linha
   de base não é uma corrida, são duas* — a alinhada e a suave — e é o `worse` entre elas que dá a
   malha limpa.
2. **O gate usava `contains` num fonte com DOIS ramos** (paralelo e `PH2D_RETOPO_SERIAL`): a
   mutação que apagava metade da corrida paralela sobrevivia porque a string continuava no ramo
   serial. ⇒ **contagem**, não `contains`.
3. **Um `contains("worse(")` casa igualmente com `!worse(`** — a decisão invertida. ⇒ a varredura
   afirma também a **ausência da negação**, sobre o ficheiro inteiro (a fatia de `1 200` chars não
   alcançava a linha da decisão).

⚠️ **E ela NÃO descasca comentários:** documentar a cura no ficheiro do produto com esse token
deixa o gate vermelho sobre código correcto. *Nesse dia a cura é descascar, não afrouxar.*

## §8-quinquies — ⭐⭐⭐ «ALGUMAS PONTAS BOAS, ALGUMAS AMPUTADAS» (Enio, 30/08, seta VERDE e VERMELHA)

A foto tinha **duas setas na mesma peça**, e isso derruba a régua do alcance: ela é a distância
**máxima** ao centroide — *um único extremo global*. **Uma ponta que sobrevive esconde outra
cortada.** É a terceira vez que esta linha paga a mesma forma (o `edge_max` cego ao quad de
`0,02 × 0,30`; o `χ` cego à almofada).

### ⭐ A régua nova: `ph2d_quadfill::tip_survival`

Um **ápice** é um vértice cujo raio ao centroide é maior que o de **todos** os vizinhos (máximo
local no grafo). ⚠️ *Não é um limiar de raio*: numa peça com espinhos de comprimentos diferentes,
um limiar apanha dois vértices do mais longo e nenhum do mais curto.

A comparação é a **função de suporte**: para cada direcção de ápice `d`, `max(v · d)` na entrada
contra a saída. ⚠️ **Sobrevive à malha ser outra** — os vértices não se correspondem, as direcções
sim.

### ⛔⛔ E o primeiro número já refutou a hipótese que estava escrita

Na peça `sculpt_t003.obj`, `Detail 0,50`:

| | ponta 0 (`r = 1,95`) | ponta 1 (`r = 1,79`) | pontas 2–11 |
|---|---|---|---|
| **fase zero (F1)** | ⭐ `−0,2 %` | `−5,4 %` | — |
| **saída final** | ⛔ `−20,0 %` | ⛔ `−21,5 %` | `−0,4 %` a `−0,0 %` |

⇒ **a fase zero entrega a ponta mais longa INTACTA e a cadeia a jusante come `20 %`.** A nota do
`CLAUDE.md` §5 que atribui a amputação à fase zero está **refutada para estas pontas**.

⭐ E o alcance global lia `−16,2 %` enquanto **dez das doze** pontas estavam a `−0,1 %`.

### ⭐⭐ A causa é RESOLUÇÃO, e a combinação que a resolve está medida

| config | ponta 0 | ponta 1 | cortadas | `χ` / bordo |
|---|---|---|---|---|
| `Detail 0,50` · knob `0` | ⛔ `−20,0 %` | ⛔ `−21,5 %` | `2` | `2 / 0` |
| `Detail 0,50` · knob `1` | ⛔ `−20,6 %` | ⛔ `−22,7 %` | `2` | `2 / 0` |
| `Detail 0,85` · knob `0` | `−5,0 %` | ⛔ `−14,8 %` | `2` | ⛔ `1 / 4` |
| ⭐ **`Detail 0,85` · knob `1`** | ⭐ **`−0,2 %`** | `−6,4 %` | **`1`** | ⭐ **`2 / 0`** |

⇒ **a célula da grade tem de caber na ponta.** A `Detail 0,50` o alvo é `0,100` e a ponta é muito
mais fina — *nenhum knob a salva ali*. A `0,85` (alvo `0,038`) com o campo adaptativo a ponta mais
longa sai **praticamente intacta** e a topologia é a **melhor das quatro**.

⚠️ **E a guarda de §8-quater comporta-se como desenhada:** a `0,50` ela recua (a saída é a mesma do
knob desligado) e a `0,85` ela mantém o ganho — porque ali o campo adaptativo é melhor em **todas**
as colunas.

### ⭐⭐⭐ O que ficou no PRODUTO: o botão diz quando amputou

`QuadRemeshReport::{tips_cut, tips_total, tips_worst_pct}`, e a linha do log:

```
 -- ⚠️ 2 de 12 ponta(s) AMPUTADA(S) (a pior -22 %); suba o `Detail` ou ligue o `Follow Curvature`
```

⛔⛔ **É a única coluna do relatório que o artista não conseguia derivar de nenhuma outra.** A
amputação sai com casca fechada, `χ = 2`, `100 %` de quads e quads bonitos — *ele descobria-a
fotografando o ecrã, três vezes*.

⚠️ **E ela diz o QUE FAZER**, porque a causa é resolução e não defeito. *Sem a frase, subir o
`Detail` é um palpite que ninguém tem razão para dar.*

### Gates novos (todos provados por mutação)

| gate | o que morre sem ele |
|---|---|
| `a_linha_do_artista_nomeia_as_pontas_amputadas` | a linha desaparece · perde o denominador e a acção |
| `uma_ponta_cortada_distingue_se_de_uma_reamostrada` | a barra `TIP_CUT_PCT` a `0` ou a `−50` · o suporte a comparar a entrada consigo mesma |

⚠️ **O 2.º gate nasceu de uma mutação que SOBREVIVEU:** o gate da linha constrói a contagem à mão,
então ele **não exercita a barra**. *Um gate sobre o relatório não é um gate sobre a régua.*

⚠️ E o 1.º reprovou primeiro por **arredondamento**: `−21,5` imprime `−22` com `{:.0}`, e a fixtura
procurava `−21`. A fixtura passou a usar um valor sem ambiguidade.

## §9 — Ponto cego novo na ferramenta do laço interno

`scripts/cargo-check-narrow.sh` corre `cargo check -p` **sem `--all-targets`** ⇒ não compila
`#[cfg(test)]` nenhum, e imprimiu *«compila»* sobre um `use` que não resolvia. É a **quarta**
variante da mesma cegueira (memória actualizada). ⇒ ao mexer em código de teste, `--all-targets`.

---

## §8-sexies — ⛔⛔⛔ «PÉSSIMO» (Enio, 30/08, 3.ª foto) — a peça SOLTA reproduz-se, e é a RE-ENTRADA

**O report:** a mesma foto de espinhos, sete setas vermelhas — um quad a flutuar **solto** acima de
uma ponta cortada, outro a flutuar longe da peça, e pontas amputadas em baixo. Palavra dele:
*«péssimo. é preciso investigar o código original.»*

### A reprodução, e o que ela CORRIGE do que eu disse antes

| configuração | quads | `χ` | bordo · não-manif. | ⭐ **peças** | pontas cortadas |
|---|---|---|---|---|---|
| `Detail 0,50` · knob `0` · **1 clique** | `1 353` | `3` | `0` · `2` | `1` | `2 de 12` (pior `−21,5 %`) |
| `Detail 0,85` · knob `1` · **1 clique** | `8 623` | ⭐ `2` | `0` · `0` | `1` | `1 de 12` (pior `−6,4 %`) |
| ⛔⛔ `Detail 0,85` · knob `1` · **2 cliques** | `7 483` | ⛔ `4` | `0` · `0` | ⛔ **`2`** — solta de `22` faces | ⛔ `2 de 12` (pior **`−35,0 %`**) |
| `Detail 0,50` · knob `0` · **2 cliques** | `1 454` | `1` | `6` · `0` | `1` | `2 de 12` (pior `−24,4 %`) |

⇒ ⭐⭐⭐ **É a RE-ENTRADA numa configuração fina que parte a peça**, e um clique só não o faz.
⚠️ **E o roteiro que a janela anterior lhe deu convidava a isso** (*«clique · Ctrl+Z · mude os
knobs · clique»*): um artista que não desfaz está no caso de dois cliques, que é exactamente o que
ele fotografou.

### ⛔⛔ Por que NENHUMA régua o via — e a acusação é ao `open_edges`

Um pedaço que se desprende sai **fechado**: leva as suas arestas com ele, cada uma com as duas
faces. ⇒ `bordo = 0`, `não-manifold = 0`, e a chave da frente do [`worse`] — que o doc dela chama
de *«as DUAS formas de a casca não fechar»* — **dá zero nas duas peças**. *Uma superfície fechada
pode conter uma segunda superfície fechada, e contar arestas nunca o revela.*

⚠️ **A minha 1.ª medição foi na direcção errada, e deu «limpo»:** medi a distância de cada face da
SAÍDA à escultura, e o pior era `3,4 ×` a aresta de entrada em toda a parte. Verdade, e
irrelevante — ver §8-septies.

### A cura, em duas metades

1. **`rulers::components`** (união por **ARESTA**, não por vértice) entra como **2.ª chave** do
   `worse`, atrás dos furos. ⛔ **A ordem é uma decisão:** os furos ficam à frente porque *foi isso
   que se mediu*, e **não existe medição que ordene um estilhaço contra um furo**. O gate
   `os_furos_continuam_a_decidir_antes_das_pecas` prende a ordem.
2. ⭐⭐⭐ **`rulers::shattered` é um VETO absoluto, DEPOIS da escada** — `RemeshRefusal::Shattered`.
   *O `worse` escolhe entre tentativas e **nunca** compara com a malha que o artista já tinha;
   quando todas partem a peça, a melhor delas ainda é uma peça partida.* É a mesma lei que pôs o
   `catch_unwind` naquele ficheiro: **o artista não perde a escultura porque a retopologia falhou.**

⚠️ **O veto é RELATIVO (`saiu > entrou`)**, então uma cena com dois objectos soltos continua a poder
sair com dois. ⚠️ **E não sobre-bloqueia:** medido, os dois cliques a `Detail 0,50` **não** partem a
peça e passam na mesma.

**Verificado:** clique 2 a `0,85`+knob devolve `Shattered { pieces: 2, was: 1 }` e a escultura do
clique 1 fica intacta. Os dois cliques únicos são **byte a byte o que eram**.

---

## §8-septies — ⭐⭐⭐ A COBERTURA: a régua que NINGUÉM tem, e a DIREÇÃO é a lei inteira

O subagente-E atravessou o alvo restrito com uma pergunta de três alíneas
([`SPEC_feicoes_finas.md`](../cleanroom/SPEC_feicoes_finas.md), commit `4283c385e`, sweep verde).
A resposta à alínea (b) é o achado:

> ⛔ **Não há protecção anti-amputação nem régua de fidelidade nenhuma — nem no produto nem nos
> instrumentos de medição do alvo.** Eles medem topologia e qualidade de forma por face. **Nenhuma
> distância à entrada. Nenhuma cobertura. Nenhum Hausdorff.**

⇒ *a nossa amputação de `−20 %` a `−35 %` é invisível a **toda** régua que os dois lados têm hoje*,
e a espec nomeia a construção que falta (§6.2 item 1). Ela existe agora:
**`ph2d_quadfill::coverage(entrada, saída)`**.

### ⚠️ A DIREÇÃO é tudo

| direcção | o que responde | numa ponta amputada |
|---|---|---|
| **saída → entrada** | *«a malha nova está pousada na escultura?»* | ⛔ **`0` — passa** |
| ⭐ **entrada → saída** | *«a escultura toda foi coberta?»* | ⭐ **grande — acusa** |

O gate `uma_ponta_amputada_acusa_numa_direccao_e_e_invisivel_na_outra` prova as duas metades na
mesma fixtura (uma pirâmide e a mesma pirâmide com a ponta cortada e tapada — **os vértices do topo
cortado caem exactamente sobre as arestas da inteira**, e é por isso que a direcção inversa mede
`0`).

### ⭐ E a CASCA é o que a torna acionável — medido na peça do artista

| `r / Rmax` | `Detail 0,50` | `Detail 0,85` + knob |
|---|---|---|
| `[0,00 · 0,50)` | `0,44 %` | `0,17 %` |
| `[0,50 · 0,75)` | `0,50 %` | `0,19 %` |
| `[0,75 · 0,90)` | `2,72 %` | `0,21 %` |
| ⭐ `[0,90 · 1,00]` | ⛔ **`6,01 %`** (pior `9,46 %`) | ⭐ **`0,095 %`** (pior `3,05 %`) |

⇒ **monótono no raio, e `63×` entre as duas configurações** — e a régua chega lá **sem saber o que
é uma ponta**, ao contrário da `tip_survival`, que tem de achar um ápice primeiro.
`COVERAGE_DEFECT = 2 %` fica `7×` acima da limpa e `3×` abaixo da amputada.

⚠️ **Distância ao TRIÂNGULO, exacta** (ponto-a-triângulo fechado + grelha por caixa envolvente).
Amostrar por vértices sobre-estima em até meia aresta de quad — *na peça dele, da ordem do próprio
defeito*: a versão por amostras lia `0,280 %` onde a exacta lê `0,095 %`.

⭐ **Ela fala SÓ onde a `tip_survival` está muda** (`tips_cut == 0`), porque duas frases para o
mesmo defeito é ruído — e o que ela acrescenta é o caso que a outra **não pode** ver: uma perda numa
crista ou numa saliência larga não tem ápice.

---

## §8-octies — ⛔⛔ A cura que a espec do alvo REFUTOU antes de eu a construir

A alínea (a) devolveu um mecanismo tentador: o alvo da remalhagem de preparação do alvo restrito
**não é dado pelo utilizador** — é derivado da malha, e tem um termo de **esfericidade** que faz uma
peça espinhosa nascer `2×`–`5×` mais fina **na malha inteira**.

```
esfericidade = π^(1/3)·(6V)^(2/3)/A        L₀ = aresta do equilátero de área (A/2000)·esf²
                                           L₁ = aresta do equilátero de área (A/10000)
alvo = min(L₀, L₁)
```

⛔⛔ **Medido na peça do artista antes de escrever uma linha: a esfericidade dela é `0,84`**, e o
ramo espinhoso (`L₀`) só ganha abaixo de `0,45`. ⇒ **o termo não dispararia**, e a regra poria a
peça em `L₁` — `1,49×` mais fino que o nosso `ALPHA × diagonal`, não `2×`–`5×`.

⭐ *Uma bola de espinhos com corpo gordo tem esfericidade ALTA:* os espinhos são finos e contribuem
pouca área e pouco volume. **O descritor é global e cego a uma protuberância fina num corpo
redondo** — que é exactamente a peça do report. ⇒ **não adoptado**, e a espec já o avisava (§6.2
item 4: *«isto é um GLOBAL, e o report do Enio é LOCAL»*).

⚠️ **E a espec nomeia o que medir ANTES de tentar densidade outra vez** (§6.2 item 5): *o nosso
campo **acorda** numa ponta fina — há singularidade ali? — e o traçado **reage**?* Se não, densidade
não salva a ponta; encarece a peça inteira. **É a wave seguinte, e a régua para a julgar existe
agora.**

---

## §8-nonies — ⛔ PROVENIÊNCIA FALSA numa constante NOSSA (achado colateral do E)

`ph2d_remesh_iso::ALPHA` atribuía o `0,02` a um preset do alvo restrito. ⛔ **Nesse alvo o número
não é um comprimento de remalhagem em leitura nenhuma** — num programa é a mistura
*regularidade ↔ isometria* da quantização, no outro o peso de alinhamento à curvatura de um
alisador de campo.

⚠️ **O valor fica** (o doc dele diz que foi MEDIDO, com tabela sobre o cubo); **a frase saiu.**
*Um número com proveniência falsa lê-se como número com medição*, e quem quisesse afinar a
densidade iria buscar autoridade a um knob que controla outra coisa.

---

## §8-decies — ⏳ O ABERTO, reconciliado no fim da jornada

O §8 acima foi escrito a meio do dia. O que MUDOU depois dele:

- ✅ **A peça solta tem cura** (§8-sexies): `components` na escolha + `shattered` como veto. ⛔ O
  que **não** está curado é a causa — *por que a re-entrada numa configuração fina parte a peça* —
  e o veto é uma rede, não uma explicação.
- ✅ **A régua de cobertura existe** (§8-septies) e é a que a espec do alvo nomeia como ausente
  dos dois lados. ⏳ **Ela ainda não DECIDE:** o `worse` não a consulta. *Uma régua que só imprime
  não é uma régua que decide* — foi essa exactamente a lição que a contagem de componentes cobrou
  hoje, dois dias depois de a sonda dela ter sido construída. **A wave seguinte põe a cobertura na
  escolha entre candidatas**, e o corpus para a julgar é o `ph2d-quadbench`, que continua ausente.
- ⏳ **A transferência continua rota**, e o §8-octies fecha a saída fácil: o descritor global de
  esfericidade do alvo **não dispararia** nesta peça (`0,84`, e o ramo fino só ganha abaixo de
  `0,45`). ⇒ resta o factor de escala conforme, e **antes dele** a medição que a espec pede: *o
  nosso campo acorda numa ponta fina, e o traçado reage?*
- ⏳ **O `Detail` de fábrica (`0,50`) é o pior ponto do slider nesta peça** e é onde o artista
  clica primeiro: `2` pontas amputadas, `χ = 3` com `2` não-manifold, cobertura de casca `6,0 %`.
  A `0,85` com o knob: `χ = 2` limpo, cobertura `0,095 %`. ⛔ **Mexer no default é decisão do
  dono** (custa `6×` os quads e `1,6×` o relógio) — a cura desta jornada foi o botão **dizê-lo**.
- ⏳ o `ph2d-quadbench` continua ausente desta máquina · o motor **`Fast`** continua a um clique
  com a saída pior (herdados, sem mudança).
- ⏳ **NÃO MEDIDO: um 2.º clique com o `Detail` DIFERENTE.** A reprodução do §8-sexies usa as
  **mesmas** definições nas duas passagens (é o que a sonda permite). O fluxo natural do artista
  — clicar, olhar, mexer no slider, clicar — pode ou não estilhaçar, e disso depende se o veto é
  invisível ou se ele obriga a um `Ctrl+Z` a cada tentativa. ⚠️ **A recusa já manda desfazer**,
  então o pior caso é chato, não destrutivo; mas o número falta e a sonda precisa de aceitar
  `Detail` por clique.

---

## §8-undecies — ⛔⛔ O PORTÃO DE FECHO achou um gate que não sabia LER o enum que mede

`the_remesh_refuses_with_the_stack_built_instead_of_flattening_it` (em
`shells/desktop/tests/`, logo **fora** do que `cargo test --bins` alcança — a 6.ª vez que essa
distinção morde esta linha) reprovou com:

> a causa `pieces: usize` não tem frase própria em `explain`

⭐ **A `Shattered { pieces, was }` é a PRIMEIRA variante com campos nomeados daquele enum.** O
parser de causas conhecia **duas** formas — unitária (`Nome,`) e tupla (`Nome(T)`) — e sobre a
terceira colhia os **campos** como se fossem variantes, e **perdia a variante real**. ⇒ ele
exigia uma frase em `explain` para um campo e, ao mesmo tempo, **deixava de exigir** a frase da
causa nova: *acusava um defeito que não existe e ficava cego ao que existe.*

⛔ **A cura NÃO foi trocar a variante por uma tupla** — `Shattered(usize, usize)` faria o gate
passar sem o tocar, e dois `usize` nessa ordem são exactamente o par que alguém troca. ⇒ **o
parser passou a ser estrutural**: uma variante é uma linha no nível **zero** de chavetas do corpo,
começada por maiúscula. ⚠️ E as chavetas contam-se **só fora de comentários** — um `{` num
doc-comment deslocaria a profundidade e esconderia tudo o que viesse depois.

⭐ **Com fixtura das três formas:** o laço `for forma in ["MultiresStack", "Extract", "Shattered"]`
prende as três, então uma regressão para o parser antigo reprova pelo nome da forma que ele
deixou de ler — *e não por um campo com nome estranho, que foi o sintoma que custou o diagnóstico.*

---

## §8-duodecies — ⭐⭐⭐ A foto do ZOOM tem número, e o defeito NÃO está no ápice

**O report:** foto aproximada de UM espinho, cinco setas — fendas escuras junto à ponta, faces
emaranhadas, uma aba escura a sair da superfície. É o item que o `CLAUDE.md` §5 tem em ⏳ ABERTO
com a nota *«NENHUMA RÉGUA O VÊ»*.

⭐ **Vê-se agora, medindo a torção por CASCA RADIAL** (`Detail 0,85` + `Follow Curvature 1`, a peça
do artista):

| `r / Rmax` | faces | torção p50 | p99 | máx | `>90°` | gravatas | lascas |
|---|---|---|---|---|---|---|---|
| `[0,00 · 0,50)` | `3 221` | `1,7°` | `32,2°` | `180,0°` | `4` | `0` | `2` |
| `[0,50 · 0,75)` | `5 226` | `1,1°` | `35,4°` | `180,0°` | `11` | `3` | `3` |
| ⛔ **`[0,75 · 0,90)`** | **`149`** | ⛔ **`10,2°`** | ⛔ **`169,1°`** | `177,1°` | `3` | **`1`** | `3` |
| `[0,90 · 1,00]` | `27` | `12,8°` | `43,9°` | `43,9°` | `0` | `0` | `0` |

⭐⭐ **O pior não é o ÁPICE — é o OMBRO do espinho** (`0,75`–`0,90`): a torção mediana é `6×`–`9×`
a do corpo, e é ali que mora a gravata e as dobras de `169°`–`177°`. *Uma face dobrada a `177°`
desenha-se como a fenda escura da foto; uma gravata desenha-se como a aba.* No ápice a malha está
**torcida mas não dobrada** (máx `43,9°`).

⚠️ **E é por isso que toda régua mediana é cega:** as duas cascas exteriores somam `176` faces de
`8 623` — **2 %**. Uma mediana de milhares não se move com elas, e foi essa a nota que este §5
carregava sem número.

⚠️ **A segunda leitura da tabela é uma FOME DE RESOLUÇÃO:** a casca do ápice tem `27` faces. Não é
que a grade lá seja má — é que quase não há grade. *A transferência de densidade (§8-octies) e este
defeito são o mesmo problema visto de dois lados.*

⇒ **A wave seguinte tem alvo e régua:** levar a torção `p99` da casca `[0,75 · 0,90)` de `169°` ao
nível do corpo (`~35°`), com a contagem de faces das cascas exteriores ao lado — ⛔ e **não** a
mediana global, que já está em `1,1°` e não tem para onde melhorar.

---

## §8-terdecies — ⭐⭐⭐ O CAMPO ACORDA NA PONTA — medido, e isso fecha a pergunta da licença

A espec do alvo (`SPEC_feicoes_finas.md` §2.4) nomeia a **única** defesa que a referência tem
contra amputar uma ponta, e ela é **indirecta**: *um espinho geometricamente significativo cria
singularidades no campo; o traçado parte o retalho ali; a ponta ganha fronteira própria e, com
ela, contagem própria de quads.* ⚠️ E os autores dizem o resto: **se o campo não acordar, a
referência degrada-se exactamente como nós.**

⇒ A espec pede (§6.2 item 5) que se meça isso **antes** de tocar em densidade. Medido agora
(`does_the_field_wake_up_at_a_thin_tip`, peça do artista):

| `r / Rmax` | verts | ⭐ singulares | faces | arestas-parede | patches |
|---|---|---|---|---|---|
| **`Detail 0,85`** | | | | | |
| `[0,00 · 0,50)` | `3 570` | `104` | `7 162` | `1 300` | `126` |
| `[0,50 · 0,75)` | `7 000` | `53` | `13 979` | `1 628` | `112` |
| `[0,75 · 0,90)` | `290` | `6` | `577` | `92` | `13` |
| ⭐ `[0,90 · 1,00]` | `69` | ⭐ **`4`** | `136` | `31` | ⭐ **`5`** |
| **`Detail 0,50`** | | | | | |
| ⭐ `[0,90 · 1,00]` | `9` | ⭐ **`3`** | `17` | `4` | ⭐ **`2`** |

⭐⭐⭐ **O campo VÊ a ponta, e o traçado PARTE o retalho ali — nas duas densidades.** A ponta
**não** cai dentro de um retalho grande, que é o modo de falha que a referência declara.

⇒ ⛔⛔ **CONSEQUÊNCIA PARA A PERGUNTA DE LICENÇA (Enio, 30/08):** o mecanismo em que a defesa da
referência assenta **já funciona aqui**. *Ter o código deles não curaria a foto do ombro* — eles
não têm protecção de ponta nenhuma nem régua de fidelidade nenhuma (§8-septies), e a defesa que
têm é esta, que nós já temos a disparar. **A decisão de abrir os algoritmos vale por outras
razões; ela não é a cura deste defeito.**

⇒ ⭐ **E o diagnóstico aperta:** com o campo a acordar e o retalho a ser partido, o defeito do
ombro (torção `p99` de `169°`, uma gravata — §8-duodecies) nasce **a jusante** do campo e do
traçado. Os suspeitos que restam são o **mapa** (G3/G5) e o **acabamento**, sobre um retalho que
está correctamente isolado mas é geometricamente extremo (um cone com `~14` faces por retalho na
casca exterior). ⏳ **A próxima medição é a das DOBRAS DO MAPA por casca radial** — o relatório já
conta `27` dobras na peça inteira e ninguém sabe onde elas estão.

---

## §8-quaterdecies — ⭐⭐⭐ AS DOBRAS DO MAPA CONCENTRAM-SE NO OMBRO: o defeito tem morada

Com o campo e o traçado **ilibados** (§8-terdecies), a mesma sonda mede as **dobras do mapa** por
casca — um triângulo cuja imagem no domínio se vira do avesso. Passo **uniforme** (o caminho de
fábrica), peça do artista, `Detail 0,85`:

| `r / Rmax` | triângulos | ⛔ dobras | % |
|---|---|---|---|
| `[0,00 · 0,50)` | `7 162` | `54` | `0,754 %` |
| `[0,50 · 0,75)` | `13 979` | `19` | `0,136 %` |
| ⛔⛔ **`[0,75 · 0,90)`** | `577` | **`18`** | ⛔ **`3,120 %`** |
| `[0,90 · 1,00]` | `136` | `1` | `0,735 %` |

⭐⭐⭐ **`23×` a banda do corpo, e é EXACTAMENTE a casca onde a torção `p99` é `169°`**
(§8-duodecies). A cadeia de prova fecha:

```
campo ACORDA (4 singularidades)  ->  traçado PARTE (31 arestas de parede)
   ->  ⛔ o MAPA DOBRA (3,12 %, 23× o corpo)  ->  extracção emite face dobrada/gravata
   ->  a foto do zoom
```

⇒ ⭐ **O defeito é do MAPA (G3/G5) sobre um retalho de ombro** — correctamente isolado pelo
traçado, mas geometricamente extremo (um cone com poucas faces). ⛔ **Não é do campo, não é do
traçado, e não é de licença.**

⚠️ **As duas contagens de sinal saem no log de propósito** (`+7108 / −54`): a dobra é a MINORIA, e
uma convenção de sinal invertida leria «tudo dobrado» com toda a confiança do mundo.

⏳ **A wave seguinte tem alvo, régua e barra:** levar `[0,75 · 0,90)` de `3,12 %` ao nível do corpo
(`0,14 %`), com a torção `p99` da mesma casca ao lado (`169° → ~35°`). ⛔ E **não** a percentagem
global de dobras, que já é `0,3 %` e não se move com isto.

---

## §8-quindecies — ⭐⭐⭐ A OBRA GRANDE EXISTE E O MAPA CONTÍNUO ZERA — e o produto sai PIOR

**Crates novas/tocadas:** `ph2d-untangle` (exporta `energy`, `energy_and_gradient`,
`lbfgs_direction`, `History`) · `ph2d-gridmap` (`injective_solve.rs` novo, `Step::at` público,
`RoundReport::injective`, chamada em `round_welded` atrás de `PH2D_GRIDMAP_INJECTIVE`) ·
`shells/desktop` (a sonda passa o passo).

### §8-quindecies.1 — A mudança de variável, que é a obra

`ph2d_gridmap::make_injective` desce a energia de barreira regularizada **nas raízes das classes
de costura**. Cada cópia é `uv = R^k · raiz + t`, com `k` e `t` extraídos **UMA vez** do mapa
consistente que entra (⛔ não reconstruídos da `Weld`: `t` é a composição de translações ao longo
do caminho até à raiz, e recalculá-la aqui seria uma segunda aritmética a divergir da primeira).

⇒ **todo estado que a descida visita satisfaz a costura EXACTAMENTE.** Não há projecção a
desfazer o trabalho — que era o mecanismo do planalto oscilante da sonda anterior
(`seam_free_probe`, §11 do plano). Gate: `a_costura_sai_exacta_porque_ela_e_a_variavel`, que
verifica **cópia a cópia** que ela continua a ser a imagem da raiz.

### §8-quindecies.2 — ⛔⛔⛔ O DEFEITO DE UNIDADES que escondeu o resultado inteiro

A 1.ª redacção construía o referencial de repouso a partir do triângulo 3D **em unidades do
mundo**. O termo `g(J) = (det²J + 1)/det J` é minimizado em **`det J = 1`** ⇒ ele pedia *uma
célula por unidade de área do mundo*, contra o alvo do G3 de *uma célula por `h`* (`h ≈ 0,038`).

| | dobras no contínuo | `min det` | iterações | relógio |
|---|---|---|---|---|
| repouso em unidades do MUNDO | `120 → 33` | `−1,977e3 → −1,581e1` | `64` externas (**o tecto**) | `4,6 s` |
| ⭐⭐⭐ repouso em CÉLULAS | **`120 → 0`** | ⭐ **`−2,870 → +1,245e−4`** | **`5`** de `64` | ⭐ **`352 ms`** |

⚠️ **E uma varredura de orçamento inteira foi gasta a medir o problema errado** (§12 do plano:
`64×32` `33` · `256×32` `32` · `64×128` `40` · `256×128` `31`). O `33` foi lido como *«o limite do
método»* e era **o limite de uma unidade errada** — §0.0 do `CLAUDE.md`: *um limite legítimo diz
de que recurso ele é.* Da varredura sobrevive só a forma da célula `64×128`: **mais trabalho no
eixo errado piora** (`40` contra `33`), porque quem faz o `ε` encolher é o laço **externo**.

⭐ **Gate: `a_energia_nao_tem_opiniao_sobre_a_densidade`** — a régua é a **invariância** (ampliar a
malha `s×` e o passo `s×` tem de dar o mesmo `uv`), com **controle** (sem escalar o passo tem de
divergir). ⚠️ E `s` tem de ser **potência de dois**: com `s = 7` o gate reprova por aritmética
(`√(49a)` ≠ `7·√a` por um ULP, e a descida a partir de emaranhado é caótica — um ULP vira `0,15`
na saída). *Uma invariância exacta exige uma transformação exacta.*

### §8-quindecies.3 — ⛔⛔⛔ O A/B ponta a ponta, e a aritmética que localiza o dano

A tabela viva mora no doc de [`injective_solve::enabled`] — é onde alguém a lê antes de ligar
isto. Resumo: quads `9 598 → 14 521` · enviesamento p50 `6,4° → 21,3°` · `>60°` `2 → 1 191` ·
defeitos locais `0,48 % → 4,83 %` · `χ` `1 → 0` · faces dobradas na extracção **`22 → 415`**.

⭐⭐⭐ **O mapa que entra na escada tem ZERO dobras e a extracção devolve `415` faces dobradas.**
*Um input impecável a produzir um output pior põe o dano a jusante sem margem de interpretação* —
nenhuma sonda extra é precisa. A escada gulosa prega os inteiros um a um e re-relaxa entre pregos;
a partir de outro ponto de partida faz outras escolhas, e piores.

⚠️ **Segundo mecanismo, de desenho:** o G3 minimiza `‖∇f − R/h‖²`, que fixa a escala **e a
ORIENTAÇÃO contra o campo cruzado**. A barreira fixa a escala (`g`) e a conformidade (`f`), e
**não tem termo nenhum a amarrar o mapa ao campo** ⇒ as linhas de grade rodam. É isso que o
enviesamento lê.

⇒ ⭐⭐ **A obra seguinte:** a barreira entra **somada** ao objectivo do G3
(`‖∇f − R/h‖² + w · barreira`), nunca a substituí-lo. É o que o §10 do plano pedia; o que esta
jornada entrega é a **maquinaria** dela, toda gateada.

⚠️ **Duas colunas MELHORARAM e não se apagam:** pontas cortadas `2 → 1` de `12` (a queixa do
dono) e cobertura p50 `0,271 % → 0,061 %` (**`4,4×`** de fidelidade). *A direcção está certa.*

### §8-quindecies.4 — O que shipa

⛔ **DESLIGADO.** `PH2D_GRIDMAP_INJECTIVE=1` liga; sem ela o botão é **byte-idêntico**. Sete
gates, entre eles os dois lados do interruptor: `nasce_desligado_e_a_tabela_da_recusa_esta_ao_lado`
(o default **e** os números que o justificam) e `a_porta_do_produto_esta_atras_da_env` (que o
caminho do produto **consulta** o interruptor, e que a chamada corre **antes** da escada — *um
interruptor que o produto não consulta é decorativo*).

---

## §8-sexdecies — ⛔⛔⛔ O VEREDITO DO DONO, e a régua que via o estrago estava NA PRATELEIRA

**Report (Enio, 30/08, 2 fotos):** *«destruiu completamente a malha e demorou minutos»*. A saída
com `PH2D_GRIDMAP_INJECTIVE=1` vem **rasgada de alto a baixo** — bandas contínuas de faces viradas
do avesso, pretas no ecrã. ⇒ **a obra fica DESLIGADA, sem apelo**, e a tabela do §8-quindecies.3
passa a levar este veredito ao lado (já está no doc de [`injective_solve::enabled`]).

### §8-sexdecies.1 — ⛔ O erro foi meu, e foi de LEITURA da severidade

Os números que eu tinha diziam-no e eu li-os como *«pior»*:

| coluna | controlo | com a obra | como se lê |
|---|---|---|---|
| gravatas (faces auto-intersectadas) | **`0`** | **`125`** | ⛔ **zero natural de um lado** |
| torção máxima | `105,8°` | **`180,0°`** | ⛔ **extremo saturado** = quad do avesso |
| defeitos locais | `0,48 %` | `4,83 %` | ⚠️ `10×`, e **concentrados em bandas** |

⇒ ⭐ *Uma coluna com zero natural, um extremo saturado, ou uma razão acima de `2×` numa coluna de
qualidade **não descreve um troco — descreve uma recusa**.* Eu escrevi-lhe um smoke de 4 passos a
pedir julgamento sobre um troco que a minha própria medição já tinha decidido. Memória:
`feedback_do_not_ask_the_owner_to_judge_a_trade_already_measured_as_destroyed`.

### §8-sexdecies.2 — ⭐⭐⭐ E a CURA que fica: o botão passa a consultar a régua que já existia

⛔⛔⛔ **`ph2d_quadfill::local_shape` vive numa crate do PRODUTO desde 30/08 e o único leitor dela
era a SONDA da foto.** O [`sculpt3d_retopo_rulers::worse`] — que escolhe entre as tentativas do
botão — lia bordo, peças, `>60°` e enviesamento, e **nenhuma delas vê uma face cruzada**.

⇒ `worse` ganha a **3.ª chave**: `bordo → peças → gravatas → forma`, via
`sculpt3d_retopo_rulers::bowties`, **pela porta** da crate (⛔ a lei não é reimplementada no shell:
uma 2.ª cópia divergiria no dia em que uma fosse corrigida).

⚠️ **ORDINAL e não veto, de propósito:** uma tentativa com gravatas perde sempre para uma sem, mas
se **todas** as tiverem ainda se escolhe a menos má. *Um veto absoluto pede prova de corpus que
esta linha ainda não tem, e inventar um limiar sem medir é o que o §0.0 proíbe.*

**Gates (2, os dois red-first, com prova de mutação):**
- `a_face_em_oito_perde_e_a_regua_antiga_nao_a_via` — fixtura de **um quad solto** em ordem contra
  trocado em oito, com **controlo** de que as duas chaves anteriores empatam, e a forma dada
  **perfeita na torta** e péssima na sã (o desempate que escolheria o estrago). ⚠️ **A 1.ª fixtura
  era um CUBO e reprovou:** permutar os cantos de uma face **muda as arestas que ela contribui** e
  abre `4` de bordo ⇒ o controlo lia `0` contra `4`. *Uma fixtura que também mexe na chave anterior
  não isola a nova.* Mutação: apagar o bloco da chave ⇒ **vermelho**; restaurar ⇒ verde.
- `a_ordem_das_chaves_e_furos_pecas_gravatas_forma` — a ordem lida **no fonte**, porque a fixtura
  que a isolaria não existe (uma malha fechada com face cruzada **abre bordo**, como acima).
  ⚠️ **E a 1.ª redacção deste gate fatiava a partir do `fn`**, e a lista de parâmetros nomeia
  `a_over60` antes de tudo ⇒ reprovava sobre a ordem certa. *Um gate que lê o fonte tem de saber
  onde acaba a declaração.*

### §8-sexdecies.3 — ⏳ O que fica ABERTO

- ⛔ A obra injectiva **fica no repo, desligada, com a tabela** — ela é a maquinaria que a wave
  seguinte precisa (barreira **somada** ao objectivo do G3), e reconstruí-la é a despesa que a lei
  das recusas medidas existe para evitar.
- ⏳ **O veto absoluto sobre gravatas** espera prova de corpus: se o caminho de omissão der `0` em
  todas as peças da bancada, ele passa de ordinal a recusa (com o `RemeshRefusal` a nomeá-lo, como
  o `Shattered`).
- ⏳ **«demorou minutos»** — medido `57,7 s → 80,1 s` no arnês; o relógio dele é maior. Não
  investigado: a obra fica desligada e o caminho de omissão não a paga.

### §8-sexdecies.4 — ⭐⭐⭐ A OUTRA METADE: as gravatas ARMAM outra tentativa

⛔ **O [`worse`] sozinho não chega, e a distinção é a que decide se o dono vê o estrago:** ele
**ordena** as candidatas que existem; se a primeira sair cruzada e nenhuma outra for pedida, a
cruzada é o que se entrega. Quem pede mais uma é a condição de **armar**, e até 30/08 ela era
**só** o bordo (`open_edges(&out) > 0`, escrito em **dois** sítios).

⇒ Nasce `sculpt3d_retopo_rulers::still_broken` = *«as chaves da frente do `worse` ainda não estão
satisfeitas»* — bordo/não-manifold **ou** faces auto-intersectadas. ⛔ A contagem de **peças**
fica de fora **por não ser absoluta**: ela só significa algo contra a entrada ([`shattered`]), e
aqui não há entrada.

⭐⭐ **É estritamente melhor que um VETO, com risco ZERO de trancar o botão:** as candidatas extra
passam todas pelo `worse`, logo *só vencem onde são melhores*; se **todas** saírem cruzadas ainda
se entrega a menos má. *Uma recusa absoluta transformaria um defeito raro numa ferramenta
inutilizável.*

**A evidência de corpus que ficou** (`the_artists_piece_through_the_button`, caminho de omissão):

| peça | `Detail` | quads | **gravatas** | lascas |
|---|---|---|---|---|
| `espinhos:6` | `0,50` | `1 361` | ⭐ `0` | `0` |
| `espinhos:6` | `0,85` | `9 469` | ⭐ `0` | `0` |
| `espinhos:12` | `0,70` | `3 367` | ⭐ `0` | `0` |
| peça do artista | `0,50` | `1 353` | ⭐ `0` | `0` |
| peça do artista | `0,85` | `9 598` | ⭐ `0` | `0` |

⚠️ **`5` corridas sobre `3` peças não são «o corpus»** — é evidência a favor, não prova, e é por
isso que o veto **não** entrou. ⏳ A bancada (`ph2d-quadbench/`) **não existe nesta worktree**;
quem a tiver e medir `0` em todas as peças pode promover a guarda a `RemeshRefusal`, no molde do
`Shattered`.

**Gates (2, os dois red-first):**
- `a_face_em_oito_arma_outra_tentativa` — com **dois** controles: a malha fechada **sã** não arma
  (senão pagava-se sempre) e a mesma fechada **com uma face cruzada** arma.
- `os_dois_sitios_que_armam_perguntam_pela_mesma_porta` — conta `2` chamadas a `still_broken` e
  **zero** `open_edges(&out) > 0`. ⚠️ *A pergunta estava escrita duas vezes, e a 3.ª chave a
  entrar só numa delas era precisamente a divergência que este gate impede.*

⚠️ **E o compilador apanhou uma dependência invisível:** tirar `open_edges` do `use` do ficheiro
pai **partiu gates noutro ficheiro** (o `mod tests` irmão chamava-o por `super::`). ⛔ A cura é
apontá-lo ao dono (`super::rulers::open_edges`) — **nunca** reter um `use` morto para um teste o
alcançar, nem calar o aviso. *Um import que só existe para um teste chegar a um nome é um fio
entre dois ficheiros que ninguém vê.*

---

## §8-septendecies — ⛔⛔⛔ O VETO SOBRE GRAVATAS: MEDIDO E REFUTADO (o item §8-sexdecies.3 FECHA)

O §8-sexdecies.3 deixou aberto *«se o caminho de omissão der `0` em todas as peças da bancada, a
guarda passa de ordinal a recusa»*. ⭐ **A pergunta certa não era essa, e a resposta é NÃO.**

Medido com `the_local_ruler_across_files` sobre os ficheiros do próprio dono:

| ficheiro | faces | irreg. | **gravatas** | o que é |
|---|---|---|---|---|
| `Sculpt_Blender.obj` | `8 291` | `86` | ⛔ **`1`** | ⭐⭐⭐ **a saída que ele APROVOU** (*«preserva as pontas»*) |
| `sculpt_Depois.obj` | `15 426` | `48` | ⛔ **`1`** | outra retopologia dele |
| `sculpt_t003.obj` | `20 235` | `39` | ⛔ **`2`** | ⭐ **a ENTRADA dele no nosso botão** |
| a nossa saída (`d=0,85`) | `9 598` | `40` | ⭐ **`0`** | — |

⭐⭐⭐ **Um veto absoluto teria RECUSADO a malha que o dono elogiou.** ⇒ *«uma face cruzada é
inaceitável» é uma barra que a ferramenta de referência não cumpre* — e a nossa saída, nesta
coluna, é **mais limpa que as três**.

⇒ A guarda fica **ordinal + a armar tentativa**, e agora com prova em vez de cautela. ⛔ **Não
volte a propor o veto** sem medir de novo estes três ficheiros: a lei foi verificada contra o que
o dono ACEITA, que é o único denominador honesto que existe aqui.

⚠️ **E isto explica porque a arma quase nunca dispara:** a nossa saída tem `0`, logo o custo do
passe extra é pago só no caso raro. *Uma rede que raramente se arma é barata; um veto que
raramente acerta é caro.*

---

## §8-octodecies — ⭐⭐⭐ AS PONTAS: o botão RECEBE a densidade certa e DEVOLVE-A AO CONTRÁRIO

Report aberto do dono (29/08): *«buracos nas pontas, faces emboladas nas pontas»* e (28/08) *«as
pontas finas perdem detalhe»*. ⭐ **Ele tem número, alvo e DIREÇÃO**, na coluna `ENTREGA` que já
existia e que já nomeia o alvo (`0,59`, derivado do oráculo aprovado):

| malha | razão ponta/corpo | aresta por casca radial (dentro → ponta) |
|---|---|---|
| ⭐ `Sculpt_Blender.obj` (**aprovado**) | **`0,580`** | `0,0547 → 0,0450 → 0,0439 → 0,0261` (**afina `−52 %`**) |
| ⭐ `sculpt_t003.obj` (**a entrada dele**) | `0,675` | `0,0275 → 0,0265 → 0,0310 → 0,0179` (**afina `−35 %`**) |
| `sculpt_Depois.obj` | `1,145` | `0,0306 → 0,0316 → 0,0301 → 0,0361` (plano) |
| ⛔ **a nossa saída** (`d=0,85`) | ⛔ **`1,300`** | `0,0390 → 0,0416 → 0,0506` (⛔ **engrossa `+30 %`**) |

⭐⭐⭐ **As duas curvas correm em SENTIDOS OPOSTOS.** Não é *«a nossa grade é uniforme»* — é
**anti-adaptativa**: os quads crescem em direcção à ponta enquanto os da referência encolhem. A
razão erra o alvo por **`2,2×`**.

⭐⭐ **E o reenquadramento que esta medição compra:** a peça que o dono mete no botão **já tem a
densidade certa nas pontas** (`0,675`, quase o alvo `0,59`). ⇒ *o trabalho não é inventar
adaptação do nada — é parar de deitar fora a que CHEGA.* ⚠️ A fase zero (`ph2d-remesh-iso`)
remalha **isotropicamente por construção**, logo ela é o primeiro suspeito com endereço, e a
informação morre lá antes de qualquer fase a jusante a poder usar.

⛔⛔ **NÃO confundir com as duas recusas medidas que já existem, que respondem a OUTRA pergunta:**
- o `Follow Curvature` (`ScaleField` adaptativo no G3) foi construído e **não adoptado** — ele
  move o expoente `7 %` e paga `15 %` da contagem e o dobro das faces `>60°`, porque *o G3 resolve
  um mapa escalar cujo gradiente alvo com `h` variável deixa de ser integrável* (§8-quater).
- «o F1 tem de seguir o alvo» foi **REFUTADA** (`PH2D_F1_TARGET=1`: `χ = 1`, `4` bordo, `123`
  dobras contra `χ = 2`, `0`, `21`).

⇒ ⏳ **O que fica NOMEADO para a próxima janela:** *preservar* a graduação que a entrada traz é uma
terceira coisa, distinta de ambas — nenhuma das duas recusas a mediu. A régua para saber quando
está feito **já existe e já tem o alvo escrito**: a coluna `ENTREGA razao ponta/corpo`, com `0,59`
do oráculo aprovado e `1,300` de hoje.

⚠️ **E uma cautela sobre a régua da PONTA que esta corrida expõe:** o `LOCAL na PONTA` conta
`0`–`3` faces nas nossas saídas (`0/101`, `1/427`, `3/272`, `1/196`, `3/208`). *Uma razão tirada de
`3` contra `1` não é `3×`, é ruído* — a coluna que decide aqui é a `ENTREGA`, que tem centenas de
faces de cada lado, não a contagem de defeitos.

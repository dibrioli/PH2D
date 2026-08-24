# SPEC — a EXTRAÇÃO de malha quad a partir de um mapa de grade inteira

```
Alvo funcional: extração robusta de malha quad de um mapa de grade inteira · Degrau: T2-por-papers
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md, aberto 2026-08-24
Patente (§8.1): buscado 2026-08-24 — nenhuma viva alcança este caminho; detalhe em TRIAGEM §3
Filtragem §4.3: executada 2026-08-24 · Sweep: verde 2026-08-24
Auditoria §4.2 (R-pré): ⏳ PENDENTE — exige janela que NÃO seja a E
Mapa de leitura da literatura:
  · Ebke, Bommes, Campen, Kobbelt — "QEx: Robust Quad Mesh Extraction", SIGGRAPH Asia 2013.
    ⭐ A fonte principal desta espec. Cópia local: ~/Referencias/papers/qex2013.pdf (+ .txt).
    ⛔ NÃO existe apêndice de listagem: o paper publica pseudo-código de NÍVEL DE PAPER,
       que é lícito (§4.1.10). Ainda assim esta espec RE-DESCREVE, nunca transcreve.
  · Bommes et al. — "Mixed-Integer Quadrangulation", SIGGRAPH 2009 (o arredondamento).
  · Bommes et al. — "Integer-Grid Maps for Reliable Quad Meshing", SIGGRAPH 2013 (o mapa).
  · Shewchuk — "Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric
    Predicates", 1996 (o predicado de orientação exacto).
  · Ray — "On Quad Mesh Extraction From Messy Grid Preserving Maps", arXiv 2507.15404, 2025.
    ⚠️ FUNDAÇÃO, não algoritmo — o próprio texto o diz. Leitura de contexto, não de receita.
Denylist de URLs (⛔ NÃO abrir): qualquer hospedagem de código, issue tracker, PR ou
  code-search de `libQEx`, `CoMISo`, `vcglib`, `xfield_tracer`, `quadretopology`,
  `quadwild`, `quadwild-bimdf`, `blossom5`.
"Este documento descreve comportamento; não contém expressão do alvo."
```

> ⭐⭐⭐ **Por que esta espec existe, com número:** medido em 2026-08-24, o **nosso** campo
> cruzado alimentado a uma extração deste tipo dá **enviesamento mediano `3,0°`** — contra
> `6°` do remalhador de produção de referência e **`27°` do nosso preenchimento por patch de
> hoje**. ⇒ *a extração não fecha a diferença: ela ultrapassa a referência.*
> A tabela completa está em [`TRIAGEM_quad_remesh.md` §5-bis](TRIAGEM_quad_remesh.md).

---

## §0 — O que entra, o que sai, e o vocabulário desta casa

**Entra:** a malha de triângulos + um **mapa de grade inteira** — uma parametrização
por-triângulo `f_t : t → R²` em que as **isolinhas inteiras**, trazidas de volta à
superfície, *são* a malha quad.

**Sai:** uma malha de **quads puros**, com posições em `R³`.

⚠️ **O mapa NÃO é uma função contínua no plano.** Cada triângulo tem a sua **carta**; entre
duas cartas vizinhas há uma **função de transição** `g` que é sempre da forma

```
g(x) = R(k)·x + t        com  k ∈ {0,1,2,3}  (múltiplos de 90°)  e  t ∈ Z²
```

⭐ **É essa forma — rotação de quarto de volta mais translação INTEIRA — que faz a grade
inteira de uma carta casar com a da vizinha.** É também exactamente o que a nossa
`ph2d-gridmap` produz, e o que o **arredondamento inteiro** (§5) tem de garantir.

**Vocabulário (nosso, com o termo da literatura entre parênteses só para quem for ao paper):**

| nosso nome | o que é | (literatura) |
|---|---|---|
| **nó** | um vértice da malha de saída | *q-vertex* |
| **saída** | um talo de aresta a sair de um nó, ainda sem par | *q-port* |
| **aresta da grade** | duas saídas emparelhadas | *q-edge* |
| **célula** | uma face da malha de saída | *q-face* |
| **canto** | o par (vértice da malha de entrada, triângulo) — um vértice tem uma imagem por triângulo | *corner* |
| **dobra** | um triângulo cuja imagem no domínio tem área **negativa** | *fold-over* |

---

## §1 — ⛔ A lei que decide tudo: NADA de epsilon

⭐⭐⭐ **A extração inteira é uma cadeia de decisões discretas** — *este ponto de grade cai
dentro deste triângulo, sobre esta aresta, ou fora?* Com aritmética aproximada, decisões
vizinhas **discordam**, e a malha sai com buracos, faces repetidas ou nada.

⇒ **Duas exigências, e são a espinha:**

1. ⛔ **Um predicado de orientação EXACTO.** Uma única primitiva basta: o **sinal** do
   determinante `det(b−a, c−a)` para três pontos do domínio, **sempre correcto**, inclusive
   quando é zero. Dele derivam *no sentido horário* · *no sentido anti-horário* ·
   *colinear* · *sinal da área do triângulo*.
   - ⭐ **Em Rust, a rota permissiva existe e é barata:** `num-bigint` + `num-rational`
     (MIT/Apache) para o caminho exacto, com **filtro de ponto flutuante à frente**
     (calcule em `f64`; só caia no exacto quando o erro do resultado puder mudar o sinal).
     ⛔ **Não** use uma biblioteca de precisão múltipla sob licença copyleft — é
     desnecessário e contamina.
2. ⛔ **A parametrização tem de ser SANEADA antes** (§2), para que o predicado exacto
   responda sobre números que **de facto** satisfazem as leis que o algoritmo assume.

⚠️ **A alternativa — tolerância com `ε`** — obriga a intersectar uma bola em torno de cada
ponto com toda a vizinhança e a tomar decisões consistentes no meio de ambiguidades. É
**mais** código, **mais** lento, e não fecha.

---

## §2 — Fase 1: SANEAR a entrada (a fase que quase todos saltam)

O mapa que chega de um solver satisfaz as leis **até um erro pequeno**. Este passo torna-as
**exactas**. ⚠️ Ele é a razão de o resto poder ser discreto.

### §2.1 — Colapsar o que degenerou no domínio

**Antes de tudo**, colapse toda aresta cuja imagem no domínio tenha comprimento zero.
⭐ **Guarde a posição em `R³` original nos CANTOS**, não no vértice: a geometria da
superfície não pode perder-se quando dois vértices se fundem no domínio.

### §2.2 — Extrair as funções de transição

Para duas faces vizinhas, com as imagens dos dois vértices partilhados em cada carta:

- **A parte rotacional** sai do **argumento do quociente complexo** dos dois vetores de
  aresta (a imagem da aresta numa carta contra a da outra), dividido por `π/2`. Dá um número
  em `[0,4)`.
- **A parte translacional** sai por substituição, depois de a rotação estar fixada.
- ⛔ **Arredonde a rotação PRIMEIRO, depois a translação** — nesta ordem. O inverso da
  transição obtém-se com a rotação complementar aplicada à diferença.

⚠️ **Se o produtor do mapa já sabe a rotação, aceite-a como entrada.** ⭐ **É o nosso caso:**
a `ph2d-gridmap` já calcula o **salto de período** por aresta, e o `ph2d-crossfield` já o
guarda por construção. *Não re-derive o que já temos — passe-o.*

### §2.3 — Igualar a PRECISÃO (o passo contra-intuitivo)

⛔⛔ **Este é o passo que ninguém adivinha, e sem ele o resto não fecha.**

O problema: as coordenadas do mesmo vértice em cartas diferentes podem ter **expoentes
diferentes** em vírgula flutuante. Aplicar a transição de uma carta para outra **perde bits
baixos**, e ao dar a volta ao leque de triângulos em torno do vértice **não se regressa ao
valor de partida**. ⇒ um ponto de grade cai *na fenda numérica* entre dois triângulos, ou
**nos dois**.

⚠️ **E isto não é raro:** onde há alinhamento a uma feição, pontos de grade caem
**necessariamente** sobre arestas da malha de entrada.

**A cura, por vértice:**

1. Determine o **maior expoente** entre todas as imagens daquele vértice em todas as cartas
   incidentes.
2. **Trunque a mantissa** da imagem de partida, deixando só os bits que **toda** carta
   incidente consegue representar.
   - ⭐ O truque é um **não-operador algébrico** que a vírgula flutuante executa como
     truncagem: some uma potência de dois grande o suficiente e subtraia-a a seguir.
     ⚠️ **Em Rust isto exige cuidado:** o otimizador pode eliminar `(x + s) - s`. Use
     `black_box`, `volatile`, ou faça a truncagem **explicitamente sobre os bits** —
     e ponha um **gate** que prove que a truncagem aconteceu.
3. **Propague** o valor truncado ao longo do leque de triângulos, aplicando as transições.

### §2.4 — Pregar as singularidades no PONTO FIXO

Num vértice **regular**, a composição das transições à volta do leque é a identidade.

⚠️ Numa **singularidade** não é — e aí **arredondar para o inteiro mais próximo não basta**:
ao fechar a volta, o valor não regressa. ⇒ **mova a imagem para o PONTO FIXO** da transição
acumulada, que se resolve em forma fechada e depende só do resto da rotação acumulada:

| rotação acumulada | onde o ponto fixo cai |
|---|---|
| `0` (mod 4) | o **inteiro mais próximo** (a transição é só translação) |
| `1`, `2`, `3` (mod 4) | uma combinação de **metades** das duas componentes da translação, com sinais trocados conforme o resto |

⚠️ **Como saber que um vértice é singular:** a rotação acumulada ser zero **não** prova
regularidade — uma singularidade de valência múltipla de 4 também dá zero.
⭐ **Aceite a valência como entrada** se o produtor a souber. **Nós sabemos**: o índice
por-vértice já é calculado (`ph2d_crossfield::vertex_index`).
⚠️ Basta distinguir **valência 4** de **≥ 8** ⇒ uma estimativa grosseira serve.

### §2.5 — Alinhamento a feições, e o colapso final

- Se houver alinhamento a feições, **encoste** as arestas de feição à isolinha inteira mais
  próxima, **aqui** — não depois.
- ⭐ **Colapse outra vez** as arestas que ficaram com comprimento zero *depois* da truncagem.
  ⚠️ Isto **não muda** a parametrização; serve só para o resto não ter casos especiais.

---

## §3 — Fase 2: os NÓS

Com a entrada saneada, o predicado exacto decide **sem ambiguidade** onde cada ponto de
grade inteira intersecta a malha parametrizada. Há **três** espécies de nó, e a varredura é
literalmente três laços:

| espécie | de onde vem | como se acha |
|---|---|---|
| **de vértice** | coincide com um vértice de entrada | a imagem do vértice **é** inteira |
| **de aresta** | cai sobre uma aresta (e não é o caso acima) | os pontos inteiros no interior do segmento entre as duas imagens |
| **de face** | cai no interior de um triângulo | os pontos inteiros no interior do triângulo-imagem |

A posição em `R³` de cada nó é a **mesma combinação convexa** que o localizou no domínio,
aplicada aos vértices em `R³`.

⛔⛔ **NÃO há correspondência 1:1 entre pontos inteiros e nós.** Cartas podem **sobrepor-se**
⇒ o mesmo ponto inteiro pode gerar **vários** nós. Isso é esperado, e a fusão (§6.3) trata.

---

## §4 — Fase 3: as SAÍDAS, e a ORDEM delas

De cada nó saem tantas saídas quantas as intersecções das isolinhas inteiras com uma
vizinhança infinitesimal da malha ali:

- nó regular interior ⇒ **quatro**
- singularidade de valência `v` ⇒ **`v`** saídas

⭐⭐⭐ **A ordem em que as saídas são guardadas é LOAD-BEARING, e é a ordem no sentido
horário SOBRE A SUPERFÍCIE — nunca no domínio.** É ela que faz *«virar à esquerda»*, na
extração de células (§6.1), ser simplesmente *«a saída seguinte na lista»*.

⚠️ **Duas coisas quebram a correspondência entre a ordem no domínio e a ordem na superfície,
e as duas acontecem:**
1. transições não-identidade introduzem **saltos** na direção entre triângulos vizinhos;
2. um triângulo **dobrado** (área negativa no domínio) **inverte** a ordem das suas saídas
   quando volta à superfície.

**O procedimento, para um nó de vértice:** percorra os cantos do vértice **em sentido
anti-horário**. Em cada um:
- se a área da imagem for **zero**, **salte** o canto;
- se for **positiva**: encontre primeiro uma direção cardinal que **NÃO** aponta para dentro
  do triângulo (é o que garante começar de fora e emitir em ordem correcta), depois emita as
  saídas **em sentido horário** para as direções que apontam para dentro **ou** que são
  colineares com a aresta de partida;
- se for **negativa** (triângulo dobrado): o mesmo, com a **ordem dos dois outros vértices
  trocada** e a varredura das direções no sentido oposto.

⚠️ **«Aponta para dentro»** é uma conjunção de dois testes de orientação — e é onde o
predicado exacto se paga.

⚠️ **Na FRONTEIRA** a condição de aceitação relaxa: uma isolinha colinear com a aresta é
aceite quando o triângulo não tem vizinho do lado anti-horário. ⛔ **Sem isto, malha com
bordo perde saídas** — e o nosso corpus tem uma peça com **38 arestas de bordo**.

**Nós de aresta e de face são simplificações:** o de aresta percorre só as **duas** faces
incidentes; o de face emite **uma saída por direção cardinal**, com a lista **invertida** se
a face estiver dobrada.

Cada saída guarda **quatro** coisas: o nó, a imagem dele **naquela carta**, o triângulo para
onde aponta, e a direção no domínio.

---

## §5 — ⚠️ O PRÉ-REQUISITO que nos alcança: o arredondamento INTEIRO

⛔ **A extração assume que as translações de transição são INTEIRAS.** Se não forem, a grade
de uma carta não casa com a da vizinha e a §2 apenas *arredonda o erro para dentro*.

⚠️⚠️ **É exactamente o nosso bloqueador nomeado:** a `ph2d-gridmap` mede hoje um resíduo de
**`0,291` de célula** nas translações de ciclo.

**A lei (do MIQ, 2009), e ela tem um nome que é a receita:** *misto-inteiro* significa
**arredondar UMA variável de cada vez e RE-RESOLVER**, nunca em lote.

1. Resolva o sistema contínuo (mínimos quadrados com as costuras).
2. Entre as variáveis ainda livres que têm de ser inteiras, escolha a **mais próxima de um
   inteiro** — a de menor `|x − round(x)|`.
3. **Pregue-a** nesse inteiro e **re-resolva** o sistema com essa restrição.
4. Repita até não sobrar variável livre.

⭐ **Por que uma-a-uma:** pregar todas de uma vez desloca todas as outras ao mesmo tempo, e
o erro **soma**. Re-resolver depois de cada uma deixa o sistema **absorver** o deslocamento
nas que ainda estão livres.

⚠️ **Duas modalidades, e a escolha é de produto:**
- **arredondar as COSTURAS** — as variáveis de translação. Mais rápido.
- **arredondar as SINGULARIDADES** — as imagens dos vértices singulares. Dá mapas melhores
  em peças com asas/alças, e é a modalidade que a medição de 2026-08-24 usou.
- ⚠️ **Caso de canto medido:** quando todas as singularidades já foram pregadas mas ainda
  restam costuras por arredondar (acontece em **peças com alça**, e o nosso corpus tem um
  toro), é preciso **passar à modalidade das costuras** em vez de terminar. ⛔ Sem isso, o
  mapa fica *quase* inteiro e a extração produz lixo.

⚠️ **Injectividade local:** o solver não a garante (a restrição é não-linear). ⭐ **E a §7
explica por que isso é aceitável:** a extração é **tolerante a dobras** por construção.

---

## §6 — Fase 4: a CONECTIVIDADE

### §6.1 — Traçar cada saída até à sua parceira

Para cada saída ainda solta: caminhe do ponto de partida na direção dela, triângulo a
triângulo, acumulando as transições, até o **alvo** cair dentro do triângulo corrente.

- **Escolher a próxima aresta:** a aresta do triângulo corrente, diferente daquela por onde
  se entrou, que intersecta o segmento. ⚠️ **Se as DUAS intersectam** (o segmento passa por
  um vértice), escolha a que tem **menos vértices sobre o segmento**; empate ⇒ tanto faz.
  ⭐ *É esta regra que faz isolinhas que passam exactamente por vértices, e triângulos
  degenerados numa linha, deixarem de ser casos especiais.*
- ⛔ **Bordo:** se a aresta só tem uma face, **aborte** o traço e deixe a saída **pendente**.
  Saídas pendentes são **ignoradas** na extração de células.
- ⛔⛔ **Mudança de orientação:** se ao passar de um triângulo para o seguinte o **sinal da
  área** mudar, **troque origem e alvo e inverta a direção**. *Sem isto, o traço atravessa
  uma dobra e sai a andar para trás.*

Cada traço bem-sucedido cria **duas** meias-ligações, uma em cada sentido, cada uma com a
transição acumulada.

### §6.2 — Fechar as células, e as coordenadas locais

Partindo de qualquer ligação não visitada: siga-a até ao nó seguinte, **vire à esquerda**
(= a saída seguinte na lista horária daquele nó), e repita até voltar ao nó de partida.

⭐ **Ao mesmo tempo, acumule a transição** e registe, para cada nó da célula, a sua imagem
**na carta do triângulo da saída inicial**. São as **coordenadas locais da célula** — uma
parametrização própria de cada célula.

⚠️ **A transição acumulada tem DOIS ingredientes por passo:** a da ligação percorrida **e** a
do **leque**, ao rodar do talo de chegada para o talo de saída dentro do mesmo nó, no sentido
horário. ⛔ Esquecer a segunda é o erro que dá células com coordenadas locais impossíveis.

⚠️ Se a saída seguinte for **pendente**, abandone esta célula e siga para a próxima ligação.

⭐⭐ **Por que as coordenadas locais são a chave:** com traço correcto, os nós de uma célula
só podem cair num de **quatro** valores locais. Um número de mudanças de orientação **par**
leva ao valor esperado; **ímpar** devolve ao mesmo valor de partida. ⇒ *coordenadas locais
repetidas são a assinatura, medível, de uma dobra* — e é assim que se limpa sem tocar nos
vizinhos.

### §6.3 — Fundir nós repetidos

Construa um grafo sobre os nós e ligue dois sempre que **partilhem as mesmas coordenadas
locais dentro da mesma célula**. Cada componente conexo colapsa num nó único, no
**centroide**; as arestas incidentes migram, sem duplicar.

⭐⭐⭐ **O resultado é um teorema, não uma esperança:** como dentro de uma célula só há **no
máximo quatro** valores locais distintos, **depois da fusão só podem existir quads, bígonos e
monógonos** — e estes dois últimos colapsam trivialmente.
⛔ **Triângulos não podem ocorrer:** exigiriam uma ligação **diagonal** no quadrado unitário,
e a fusão não cria ligações novas.

### §6.4 — Recuperar arestas perdidas

⚠️ **Um caso especial sobrevive:** um leque inteiro de triângulos em torno de um nó pode,
com dobras, abranger **menos de 180°**. Aí duas saídas consecutivas apontam na mesma direção
sem que nenhuma esteja num triângulo dobrado, e o nó fica com **saídas a menos** (no limite,
uma só). ⇒ **detecte e insira a aresta em falta** antes da extração de células.

### §6.5 — Contar valência

⚠️ A valência de uma singularidade é necessária na §2.4 e não está disponível antes de a
parametrização estar saneada. ⭐ **Só é preciso distinguir `4` de `≥ 8`** ⇒ uma contagem
grosseira sobre coordenadas não saneadas **serve**, e é isso que quebra a circularidade.
⭐⭐ **No nosso caso a circularidade nem existe:** o índice por-vértice já é um facto do campo.

---

## §7 — ⭐⭐ A tese do método, e por que ela nos serve

⛔ **A maioria dos remalhadores gasta a maior parte do tempo a IMPEDIR dobras** — endurecendo
o sistema e re-resolvendo até não haver nenhuma.

⭐ **Este método aceita a dobra e extrai mesmo assim.** ⇒ o solver pode parar mais cedo, e um
mapa antes considerado defeituoso passa a ser utilizável.

**As espécies de dobra, e quem as trata:**

| espécie | quem trata |
|---|---|
| contida no interior de uma célula da grade | ⭐ **ninguém** — o traço nunca a atravessa |
| atravessa uma isolinha sem tocar num ponto de grade | o traço (§6.1) |
| **contém** um ponto de grade | a enumeração de saídas + a extração de células + a fusão |
| leque com menos de 180° (valência colapsada) | a recuperação de arestas (§6.4) |

---

## §8 — O que JÁ EXISTE do nosso lado (⛔ não reconstrua)

| a fase | onde já vive | estado |
|---|---|---|
| campo cruzado 4-RoSy com decisão global | `ph2d-crossfield` | ⭐ **medido melhor que a referência**: `3,0°` contra `5,0°` de enviesamento mediano |
| índice/valência por vértice | `ph2d_crossfield::vertex_index` | ✅ — **alimenta a §2.4 e a §6.5** |
| salto de período por aresta | `ph2d-crossfield` / `ph2d-gridmap` | ✅ — **alimenta a §2.2** |
| corte em discos | `ph2d-gridmap` (G1) | ✅ |
| pentear + salto de período | `ph2d-gridmap` (G2) | ✅ |
| solver global do mapa | `ph2d-gridmap` (G3) | ✅ contínuo |
| ⛔ **arredondamento inteiro uma-a-uma com re-solve** | — | ⛔ **§5 — a obra 1** |
| ⛔ **a extração inteira (§2–§6)** | — | ⛔ **a obra 2** |
| predicado de orientação exacto | — | ⛔ `num-bigint`/`num-rational` (MIT/Apache) + filtro em `f64` |
| régua por-face | `ph2d_quadfill::QuadShape` | ✅ — **é a barra** |

---

## §9 — Os GATES, e a barra de cada um (derivada, nunca de conforto)

| # | o gate | a barra, e de onde ela vem |
|---|---|---|
| 1 | a truncagem de precisão **acontece** | ⛔ o otimizador do Rust pode apagar um não-operador algébrico — **prove por mutação** que remover a truncagem fica **vermelho** |
| 2 | dar a volta ao leque de um vértice **regressa ao valor de partida**, ao bit | é a definição do §2.3; `==` exacto, não `approx` |
| 3 | numa singularidade, a transição acumulada **fixa** a imagem saneada | ponto fixo, §2.4 |
| 4 | toda translação de transição é **inteira** depois do §5 | `x == x.round()`, exacto |
| 5 | o predicado de orientação **concorda com o exacto** em casos adversariais | gere pontos quase-colineares em escala de `f64`; o filtro rápido tem de **desistir**, não errar |
| 6 | **toda** face da saída é um quad | é o teorema do §6.3 — se falhar, a fusão está incompleta |
| 7 | a característica de Euler da saída **iguala** a da entrada | ⚠️ a família de defeitos que a `line/sculpt3d` já pagou: `χ` do toro tem de dar `0`, não `2` |
| 8 | ⭐ **a forma por-face** bate a barra do oráculo | `QuadShape`, medido pelo **mesmo código** sobre a saída dele, **nas nossas peças**: enviesamento p50 **`4,8°`–`7,1°`** · faces com canto pior que 60° = **`0`** · aspecto p50 `1,08`–`1,22`. ⚠️⚠️ **A cadeia de referência montada em 24/08 NÃO atinge esta barra no nosso corpus** — ela dá `9,1°`–`12,4°` e `5`–`6` faces péssimas. ⛔ **A barra é o oráculo, não a cadeia de referência**: bater `12°` seria copiar um resultado pior |
| 9 | uma malha com **bordo** produz saída | a peça com 38 arestas de bordo do corpus |
| 10 | uma malha de **género 1** produz saída com `χ = 0` | o toro do corpus |
| 11 | ⛔ **o caminho antigo continua byte-idêntico** enquanto o interruptor estiver desligado | a lei desta linha: tudo o que é novo shipa **desligado** com a tabela ao lado |

⚠️ **Comparação fase a fase é mais forte que comparar o fim** — e ela está **disponível**: o
arnês em `~/Referencias/directional-bench/` corre uma implementação independente sobre a
**nossa** malha e o **nosso** campo, e escreve a malha resultante. ⇒ *cada fase nossa pode
ser cobrada contra a dela, na mesma peça.*

---

## §9-bis — ⭐⭐ OS FIXTURES JÁ EXISTEM: a extração pode ser construída SOZINHA

⛔ **Não espere pelo §5 para começar o §2–§6.** Em
[`fixtures/`](fixtures/README.md) estão **mapas de grade inteira de referência**, sobre a
**nossa** malha e o **nosso** campo, já **verificados**:

| peça | triângulos | arestas interiores | ⭐ costuras | resíduo de translação máx |
|---|---|---|---|---|
| gancho orgânico (fechado) | 6 768 | 10 151 | **247** | `3,55e-15` |
| toro (**género 1**) | 4 096 | 6 143 | **138** | `3,55e-15` |

⇒ **A obra parte em duas de verdade:** a extração consome estes mapas e é gateada contra a
régua por-face **hoje**; o arredondamento inteiro (§5) é cobrado, mais tarde, contra estes
mesmos mapas como **saída esperada**.

⭐ **O verificador é o gate nº4, executável**, e foi provado por **dois** controlos positivos
(deslocar uma face por não-inteiro ⇒ reprova; rodar uma face 90° ⇒ **aprova**, porque é
transição legítima).

⚠️⚠️ **Duas cercas que a preparação dos fixtures descobriu, e as duas mudam a obra:**

1. ⛔ **Uma peça SEM COSTURA não gateia nada.** A primeira medição saiu sobre um mapa com
   **zero** arestas de rotação não-nula — ele teria aprovado uma extração que ignorasse
   transições por completo. *Fixture só prova o que contém.* ⇒ **os dois fixtures acima têm
   costuras de propósito.**
2. ⛔⛔ **O gate nº9 (malha com BORDO) NÃO TERÁ ORÁCULO.** Medido em duas peças: a integração
   da implementação de referência **cai com falha de segmentação** em malha com bordo. ⇒ a
   nossa extração tem de resolver o bordo **sem gabarito**, e a §4 já diz como (a condição de
   aceitação relaxa quando o triângulo não tem vizinho do lado anti-horário).

⚠️⚠️ **E dois números para calibrar a ambição, medidos no nosso corpus:**

- **Robustez:** de **7** peças, a implementação de referência extraiu **4** (em `8–15 s`),
  **recusou 1**, **estourou `900 s` em 1**, e **caiu com falha de segmentação** no toro
  (género 1) — ⭐ **cujo mapa saiu perfeito**, logo o defeito é da extração.
  ⛔ *Robustez é precisamente o que o método promete; a implementação de referência não a
  entrega no nosso corpus.*
- **Qualidade:** nas três que passaram ela dá `9,1°`–`12,4°` de enviesamento mediano, contra
  `4,8°`–`7,1°` do oráculo de produção **nas mesmas peças**. ⇒ ⛔ **não copie o comportamento
  dela; a barra é o oráculo** (gate nº8).
- ⚠️ **A hipótese nomeada para essa diferença** é o **curl** do campo: o nosso é liso mas
  nunca foi tornado integrável. ⇒ **medir e reduzir o curl é pré-condição**, não diagnóstico
  opcional (§5-bis.5 da triagem).

---

## §10 — ⛔ Recusas MEDIDAS

| recusa | mecanismo | onde |
|---|---|---|
| ⛔ **Não usar tolerância com `ε` no lugar de predicado exacto** | obriga a intersectar uma bola com toda a vizinhança e a decidir consistentemente em ambiguidades: mais código, mais lento, e não fecha | §1 |
| ⛔ **Não saltar o saneamento de precisão** | pontos de grade caem na fenda entre dois triângulos **ou nos dois**, e isso é o caso NORMAL onde há alinhamento a feição | §2.3 |
| ⛔ **Não arredondar as variáveis inteiras em LOTE** | o erro soma; o nome do método é *misto-inteiro* precisamente por ser uma-a-uma com re-solve | §5 |
| ⛔ **Não terminar o arredondamento ao esgotar as singularidades** | numa peça com alça sobram costuras por arredondar e o mapa fica *quase* inteiro — que é pior que contínuo | §5 |
| ⛔ **Não guardar as saídas na ordem do DOMÍNIO** | transições e triângulos dobrados quebram a correspondência com a ordem na superfície, e *virar à esquerda* deixa de funcionar | §4 |
| ⛔ **Não esquecer a transição do LEQUE** na acumulação | dá coordenadas locais impossíveis e a fusão não converge | §6.2 |
| ⛔ **Não tentar impedir as dobras antes de extrair** | é onde a maior parte do tempo dos remalhadores se vai, e este método torna-o desnecessário | §7 |
| ⛔ **Não usar biblioteca de precisão múltipla copyleft** | `num-bigint`/`num-rational` são MIT/Apache e bastam | §1 |
| ⛔ **Não re-derivar a rotação da transição** | o nosso salto de período já é um facto do campo | §2.2 |
| ⛔ **Não construir contagem de valência antes do saneamento** | basta distinguir `4` de `≥8`, e nós já temos o índice | §6.5 |
| ⛔ **Não esperar pelo arredondamento (§5) para começar a extração** | os mapas de referência já existem, verificados, sobre a nossa malha e o nosso campo | §9-bis |
| ⛔ **Não gatear com peça sem costura** | um mapa de rotação toda-nula aprova uma extração que ignore transições | §9-bis.1 |
| ⛔ **Não contar com oráculo para o caso de BORDO** | a integração de referência cai com falha de segmentação ali, medido em duas peças | §9-bis.2 |

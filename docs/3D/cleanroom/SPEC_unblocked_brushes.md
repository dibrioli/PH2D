# SPEC — os quatro pincéis cujo bloqueio de substrato CAIU

```
Alvo: Blender 5.2 (fonte lido na tag v5.2.0; oráculo corrido = binário 5.2.1 LTS) · Licença: GPL-2.0-or-later · Degrau: T2
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-unblocked.md, 2026-09-13
Patente (§8.1): buscado em 2026-09-13 — projecção por raycast em escultura · apagar/esfregar
  deslocamento multirresolução · decimação local sob pincel · shrinkwrap interactivo · pincel de
  projecção entre sub-ferramentas · relaxação multirresolução. Resultado: NENHUMA patente viva
  alcança os métodos (a mais próxima, a de Kelvinlets, é solução analítica de elasticidade e não
  descreve nenhum destes quatro). Arte anterior pública abundante desde 2009–2013. Veredito: prosseguir.
Filtragem §4.3: re-executada em 2026-09-13, DUAS vezes (3 achados na 1.ª passagem do R-pré, mais
  4 na 2.ª) · Sweep: ✅ VERDE em 2026-09-13 sobre **206** entradas (181 do R-pré + 25 sentinelas
  do texto removido nas duas rondas).
  ⚠️⚠️ **O verde ANTERIOR, sobre 175, não provava filtragem nenhuma:** a vassoura estava só na
  língua do ALVO e esta espec escreve-se na NOSSA, logo toda tradução de comentário passava por
  baixo dela. *Uma vassoura monolingue não vigia uma espec traduzida* — e é por isso que as
  entradas novas do R-pré são prosa em PORTUGUÊS.
  ⭐ **Controlo positivo, para o verde significar alguma coisa:** a mesma vassoura sobre a versão
  ANTES da reescrita (`git show <commit anterior>:…`) dá **19 hits** — os 5 do 1.º bloco, os 8 do
  2.º, e os 6 da prosa; sobre esta versão dá **0**.
Auditoria §4.2 (R-pré): ⛔ REPROVADA DUAS vezes em 2026-09-13, e as duas com achado substancial.
  · **1.ª passagem — 3 achados:** 2 blocos de pseudo-código que espelhavam o original passo a passo ·
    1 família de prosa de comentário traduzida, em 4 sítios. ⭐ **CURADOS:** o bloco da §3.2 SAIU (a
    prosa em volta já dizia o mesmo, e ele só acrescentava FORMA); o da §5.2 virou **média ponderada
    em forma fechada**, com o desvio **matematicamente inerte** demovido a nota de custo (§4.1.8); os
    4 sítios foram re-ditos como **REQUISITO**.
  · **2.ª passagem — 4 achados** (ela confirmou as três curas acima uma a uma e achou mais):
    ⛔ **SUBSTANCIAL, e byte-idêntico à redacção que já tinha sido reprovada — a 1.ª emenda não lhe
    tocou:** a §3.1 sustentava um facto de comportamento descrevendo o **INVENTÁRIO e a FORMA** do
    código do alvo, em três orações. ⭐ **CURADO, e o facto ficou MAIS FORTE:** ele é agora **medido**
    (`densidade_detalhe_manual`, `max │Δposição│ = 0` exactamente, `0` de `1 681` vértices movidos) —
    *ele não tinha fixture nenhuma, e a atribuição ao fonte estava a fazer o trabalho de uma*.
    Mais: a atribuição do expoente da força a texto do fonte (§1.1 ⇒ aponta à §9.1, que já declara a
    proveniência lícita) · a caracterização do pincel pela FORMA DA CONDIÇÃO (§3.2 ⇒ **tabela-verdade**,
    que as 4 fixtures da secção provam) · e a **política de nomes**, que vivia só no ledger e o
    protocolo veda ao Implementador (⇒ está no bloco acima).
  ⚠️ **A lição das duas rondas:** *uma emenda cura o que o auditor NOMEOU, e a §3.1 sobreviveu à
  primeira por ninguém lhe ter apontado o dedo.* Quem reescrever daqui em diante varre o documento
  inteiro pela pergunta da §4.3.1 — **«isto descreve o que o programa FAZ, ou como ele está
  ESCRITO?»** — e não só os parágrafos citados no relatório.
  ⛔⛔ **A janela NÃO implementa nem LÊ esta espec enquanto esta linha não disser «auditada contra
  §4.2 por R-pré em <data>»** — e quem a escreve é um **R-pré NOVO sobre esta redacção**, nunca o
  autor da reescrita: *autofiltragem não é auditoria*, e foi exactamente a autofiltragem verde de
  há uma hora que deixou passar os três.
Política de NOMES (§4.2 + §4.1.13) — o quadro que o Implementador confere, porque o ledger é-lhe
  vedado por protocolo e esta é a única página onde ele pode lê-la:
  · ⛔ **ZERO identificador interno do alvo** nesta espec — nenhum nome de função, variável, tipo,
    ficheiro ou constante do fonte dele. O que não é interface pública é dito em **vocabulário do
    DOMÍNIO** («a superfície de referência», «o passe de topologia», «a fracção mínima de aresta»).
  · ⭐ **Tudo o que esta espec NOMEIA é interface PÚBLICA**, e o modo de obtenção é o que a torna
    lícita: ela saiu de **CORRER** o alvo sem interface e despejar as definições que ele publica a
    quem o programa de fora — os identificadores de propriedade e os valores de enumeração que
    qualquer script de utilizador usa —, **nunca de ler o fonte**. São eles: `SIMPLIFY` ·
    `DISPLACEMENT_ERASER` · `DISPLACEMENT_SMEAR` · `SCENE_PROJECT` · `project_ray_direction_type`
    (`VIEW_NORMAL` · `PLANE_NORMAL`) · `minimum_distance` · `use_bidirectional` ·
    `smear_deform_type` (`DRAG` · `PINCH` · `EXPAND`). Mais os **rótulos que aparecem na tela**,
    que são texto público do manual.
  · ⚠️ **No corpo, o RÓTULO lidera** — títulos e tabelas dizem o que o artista vê (*Density*,
    *Scene Project*, *Minimum Distance*…), e o identificador público fica **só** onde regenerar uma
    fixture exige a cadeia exacta (o cabeçalho de cada fixture, e a §7).
  · ⚠️ **Os identificadores do NOSSO repo** (`ph2d_mesh::…`, `crates/…`) são nossos e não têm cerca.
Constantes do passe de topologia (§3.4–§3.6): as três leis de tamanho de detalhe, a fracção mínima
  `0,4`, a constante de escala do detalhe relativo `0,4`, as prioridades de bordo `1,50`/`1,25` e os
  dois factores da propagação `1,2`/`1,6` entram como **FACTO LIDO**, não medido pela porta do
  produto. ⚠️ Declarado aqui de propósito: cada um deles é um **número**, não uma expressão, e a
  §4.1.3 aceita-os com proveniência — mas quem os gatear deve saber que a proveniência é *leitura*, e
  que o caminho para os **medir** existe e não foi corrido (varrer o knob e contar o que a topologia
  faz, como a §3.2 faz para as quatro células da tabela-verdade).
Mapa de leitura da literatura (tudo PÚBLICO e livre; nenhum apêndice a pular):
  · Catmull & Clark 1978, «Recursively generated B-spline surfaces on arbitrary topological meshes»
    — o esquema de subdivisão de quads.
  · Halstead, Kass & DeRose 1993, «Efficient, fair interpolation using Catmull-Clark surfaces»
    (SIGGRAPH) — §3: a MÁSCARA DE LIMITE, que é a peça que falta ao nosso substrato (§4.4).
  · Loop 1987, tese de mestrado, «Smooth subdivision surfaces based on triangles» — o esquema de
    triângulos e o ponto-limite dele.
  · Stam 1998, «Exact evaluation of Catmull-Clark subdivision surfaces at arbitrary parameter
    values» — a avaliação paramétrica, que é o que o alvo usa para amostrar o limite numa grelha.
  · Garland & Heckbert 1997, «Surface simplification using quadric error metrics» — ⚠️ leia-o para
    saber o que o alvo NÃO faz: ali a escolha de aresta é por erro quadrático, e aqui é por
    COMPRIMENTO puro (§3.5).
  · Botsch & Kobbelt 2004, «A remeshing approach to multiresolution modeling» — remalhagem
    incremental por partir/colapsar/flip sob um alvo de comprimento de aresta; é a família a que o
    passe de topologia do alvo pertence.
Denylist de URLs (⛔ o Implementador NÃO abre nenhuma destas):
  · projects.blender.org/blender/blender (o repositório, as issues e os PRs — issues renderizam diffs)
  · developer.blender.org (o tracker anterior, idem)
  · qualquer espelho de código do alvo (GitHub mirrors, code-search, grep.app, sourcegraph)
  · /home/enio/Documentos/Recursos/BlenderSculpt/ e ~/Referencias/blender-unblocked/
  ⭐ O manual PÚBLICO (docs.blender.org/manual/…/sculpt_paint/sculpting/tools/) é LÍCITO e está
    destilado aqui; abri-lo é permitido, transcrevê-lo não.
"Este documento descreve comportamento; não contém expressão do alvo."
```

⚠️ **Nenhuma citação verbatim, de propósito.** O direito de citação (§4.1.12) permitiria trechos
curtos de prosa entre aspas, e esta espec **não usa nenhum**: várias das frases que valeria a pena
citar estão na vassoura do sweep (é para isso que elas lá estão), e uma espec que só passa o portão
porque alguém se lembrou de escolher outra frase não é auditável. Tudo o que os autores escreveram
chega aqui **re-dito**, com o endereço público ao lado.

---

## §0 — O que esta espec entrega, e o veredito por pincel

Quatro pincéis do catálogo público do alvo que o nosso [plano de ferramentas](../21_plano_modos_e_ferramentas.md)
§5.2 deixou de fora **por bloqueio de substrato**. A missão pediu que cada bloqueio fosse
**reconferido contra o NOSSO código** antes de a espec existir. Foi, e o resultado não é uniforme:

| pincel (o rótulo na tela) | identificador público | bloqueio de 2026 | estado do NOSSO substrato hoje | degrau |
|---|---|---|---|---|
| **Density** | `SIMPLIFY` | «falta o decimate do dyntopo» | ⭐ **CAIU, e por inteiro** — `ph2d_mesh::collapse_in_sphere` + `collapse_target` + `edge_target` existem, e o nosso passe de topologia já corre **colapso antes de refino**, que é a ordem do alvo | **T0 sobre o motor, T2 sobre a semântica** (§3.9) |
| **Erase Multires Displacement** | *Erase Multires Displacement* | «falta multires» | ⚠️ **CAIU PELA METADE** — a pilha existe (`ph2d_mesh::Multires`), mas a grandeza que ela guarda **não é a mesma** (§4.4). Falta **uma** peça, nomeada e barata | **T2** |
| **Smear Multires Displacement** | *Smear Multires Displacement* | «falta multires» | ⚠️ **idem** — mesma peça em falta, mesmo motivo | **T2** |
| **Scene Project** | *Scene Project* | «o viewport de escultura é de um objeto» | ⭐ **CAIU** — `cena.rs` tem `objects: Vec<SceneObject>` com `active: usize`, cada peça com a própria pose, e `ph2d_mesh::ray` já lança raio contra uma malha com octree | **T2** (a lei é do alvo; o motor de raio é nosso) |

⛔ **Fora de escopo por ordem do dono, e nomeados para não serem procurados:** tudo o que depende
de *face sets* (não existem nesta casa) e todo pincel ou opção de **cor** (*Paint*, *Smear*, *Blur*).
Nenhum dos quatro os toca — o de cor que partilha o rótulo «Smear» é **outro pincel**, e a confusão
de nome é a única coisa que os liga.

---

## §1 — A LEI QUE ATRAVESSA OS TRÊS QUE DEFORMAM (a cadeia de peso)

Os três pincéis que movem vértices (`DISPLACEMENT_ERASER`, `DISPLACEMENT_SMEAR`, `SCENE_PROJECT`)
derivam, por vértice, **um peso em `[0, 1]`** antes de qualquer lei própria. A cadeia é a MESMA dos
pincéis que esta casa já portou, e a ordem dos passos é observável:

1. **Piso**: `1` para todo vértice; `0` se ele está escondido; `1 − máscara` se ele está mascarado.
2. **Recorte de região** (simetria / plano de corte do modo).
3. **Só faces de frente**, se o pincel o pedir: zera o vértice cuja normal aponta para longe do
   observador (produto com a normal da vista `< 0`).
4. **Distância** ao centro do dab — euclidiana (pegada esférica) ou **projectada no plano da vista**
   (pegada tubular). ⚠️ A escolha da pegada é a MESMA porta que decide a região do passe de
   topologia do `SIMPLIFY` (§3.3): não são dois ajustes.
5. **Corte pelo raio**: distância `> raio` ⇒ peso `0`.
6. **Dureza** — remapeia a DISTÂNCIA, nunca o peso, e a fórmula é exacta:

   ```
   limiar = dureza · raio
   dureza = 0  ⇒  d' = d                                   (sem efeito)
   dureza = 1  ⇒  d' = 0        se d < limiar,  senão raio  (degrau)
   0 < dureza < 1 ⇒
       d' = 0                                   se d < limiar
       d' = raio · (d/raio − dureza)/(1 − dureza)   caso contrário
   ```

   *i.e.* a dureza **encosta a curva de queda na borda**: o miolo fica chapado e a transição
   comprime-se no anel que sobra.
7. **Curva de queda**: o peso é multiplicado pela curva do pincel avaliada em `d'/raio`.
   ⚠️ **Só a curva. A força NÃO entra aqui** — é o passo seguinte que a traz, e é aí que mora o §1.1.
8. **Auto-máscara** (cavidade, face de frente, topologia…), se ligada.
9. **Textura de máscara**, se houver.

### §1.1 — ⭐⭐ A FORÇA ENTRA AO QUADRADO

O número que a UI chama *Strength* **não é o factor**: ele é elevado ao quadrado antes de ser usado.
⇒ o efeito de uma posição do slider é a **fracção ao quadrado**, e a metade de baixo do curso ganha
resolução às custas da de cima. A proveniência de que isto é **deliberado**, e não um acidente
numérico, está na §9.1.

```
factor_de_força = força_UI²  ×  pressão  ×  sobreposição_de_simetria  ×  feather
                  [ × sinal_de_inversão, SÓ no SCENE_PROJECT ]
```

⚠️ **E o quadrado é a ÚNICA vez que a força aparece**, mas ela é multiplicada *depois* da cadeia
de peso, não dentro dela. O resultado por dab é `peso × factor_de_força`.

**Medido** (curva *Constant* ⇒ peso `1` no miolo; UM dab; fixtures `*_constante_1passo`):

| pincel | força UI | fracção do alvo percorrida | `força¹` | `força²` |
|---|---|---|---|---|
| *Scene Project* | `1,0` | **`1,000000`** | 1,000 | 1,000 |
| *Scene Project* | `0,5` | **`0,250000`** | 0,500 | **0,250** ✓ |
| *Erase Multires Displacement* | `1,0` | **`1,000000`** | 1,000 | 1,000 |
| *Erase Multires Displacement* | `0,5` | **`0,250000`** | 0,500 | **0,250** ✓ |

⇒ *duas famílias independentes, o mesmo expoente*. Implementar `força¹` faz o pincel parecer o dobro
de forte a meio curso, e é o erro que a medição de um pincel só não distinguiria de um bug de curva.

### §1.2 — A INVERSÃO (Ctrl) só existe num dos três

| pincel | o que o Ctrl faz | medido |
|---|---|---|
| *Erase Multires Displacement* | ⭐ **NADA** — o sinal de inversão não entra na força dele | `max |d − d'| = 0,000e+00` sobre o traço inteiro: a saída é **byte-idêntica** |
| *Smear Multires Displacement* | **NADA**, pelo mesmo motivo | (mesma forma; o factor não tem sinal) |
| *Scene Project* | **troca o sentido do raio** (o sinal entra no factor, logo a translação nega) | ver §6.6 |

⚠️ *Um pincel que ignora o Ctrl não é um pincel a que falta uma feature* — apagar deslocamento tem
um só sentido (o deslocamento zero), e «apagar ao contrário» não nomeia nada.

⭐⭐ **E há DUAS portas para o mesmo sinal, que dão o MESMO resultado ao bit:** o gesto (segurar Ctrl
durante o traço) e a propriedade de **sentido** do pincel (*Add* / *Subtract*). Medido na projecção,
`projectar_invertido` (pelo gesto) contra `projectar_subtrair` (pela propriedade):
**`max |d − d′| = 0,0`**. ⇒ *não são dois caminhos com duas leis* — é um sinal com dois interruptores,
e quem implementar um tem o outro de graça. ⚠️ Um gate que só exercite o gesto deixa a propriedade
por cobrir, e vice-versa: **os dois têm fixture**.

### §1.3 — Os tectos, que diferem entre os três

| pincel | tecto sobre o factor de força |
|---|---|
| *Erase Multires Displacement* | `min(f, 1)` — nunca passa do alvo (não há ultrapassagem) |
| *Smear Multires Displacement* | `clamp(f, 0, 1)` — idem, e o piso importa (§5.4) |
| *Scene Project* | ⛔ **nenhum** — o factor vai cru. Com `força² ≤ 1` ele não passa de `1` por si, mas a sobreposição de simetria pode empurrá-lo acima, e aí a peça **ultrapassa o alvo** |

---

## §2 — A SUPERFÍCIE DE REFERÊNCIA (o que «deslocamento» quer dizer no alvo)

Esta secção é a **espinha** dos dois pincéis de multirresolução e é onde a nossa casa diverge. Leia-a
antes de §4 e §5.

No alvo, uma peça com multirresolução guarda, por elemento da grelha do nível de topo, um
**deslocamento em espaço de OBJECTO** contra a **SUPERFÍCIE-LIMITE** da subdivisão da malha-base:

```
posição_do_vértice  =  Limite(base, u, v)  +  deslocamento
```

onde `Limite(base, u, v)` é o ponto da superfície-limite de Catmull-Clark avaliado na coordenada
paramétrica `(u, v)` daquele elemento da grelha — **não** o resultado de `k` passos de subdivisão.

⭐ **A palavra «limite» é literal e é dos autores:** a mensagem de commit que introduziu o apagador
descreve-o como repor a malha na superfície-limite da subdivisão, e a do esfregão descreve-o como
esfregar o deslocamento **sobre** a superfície-limite
([o histórico público do alvo](../../3D/cleanroom/LEDGER_blender-unblocked.md) tem os dois endereços;
o Implementador não precisa deles — o facto está aqui).

### §2.1 — Por que isto não é um detalhe de implementação

`subdivide^k(base)` **não é** a superfície-limite. A diferença não tende a zero na densidade que um
artista usa: ela é o resíduo do esquema de subdivisão, é maior perto de vértices irregulares, e é
exactamente a grandeza que o nosso próprio `shapes.rs` já regista por escrito — *o limite de um cubo
de lado 1 tem meia-extensão `0,4198`, não `0,5`*. Um apagador que reponha o vértice em
`subdivide^k(base)` em vez de no limite **encolhe a peça** de um modo que o artista lê como «o
apagador comeu a forma».

### §2.2 — ⛔⛔ O NOSSO MODELO É OUTRA GRANDEZA, nas DUAS pontas

O nosso [`ph2d_mesh::Multires`](../../../crates/ph2d-mesh/src/multires.rs) guarda, por vértice do
nível de cima, o detalhe contra a **previsão de UM passo** de subdivisão do nível de baixo, e
expresso num **referencial local por vértice** `(normal, tangente, binormal)`:

| | o alvo | nós |
|---|---|---|
| **superfície de referência** | o **limite** da subdivisão (infinitas iterações, avaliado em forma fechada) | a **previsão de um passo** (`predict`) |
| **coordenadas do deslocamento** | espaço de **objecto** (um vector cru) | referencial **local** do vértice |
| **o que um deslocamento zero dá** | o ponto do limite | o ponto que a subdivisão poria |
| **o que acontece ao torcer a base** | o deslocamento NÃO gira (ele é um vector de mundo) | ⭐ o detalhe **gira junto** (é isso que mantém uma verruga perpendicular à pele) |

⚠️ **Nenhuma das duas é «a certa»** — a nossa escolha de referencial local está medida e defendida
no cabeçalho daquele módulo, e é melhor para o que ela foi feita. O que a espec afirma é mais
estreito e mais duro: **as duas grandezas não são conversíveis por uma mudança de unidade**, logo
uma fixture de posições do alvo **não pode ser um golden numérico nosso sobre a mesma entrada**
(§8.2), e os dois pincéis de multirresolução têm de ser definidos contra a NOSSA referência.

### §2.3 — A peça que falta, nomeada e com preço

O que o nosso substrato não tem é **um avaliador de ponto-limite**. Ele é uma máscara de pesos sobre
o anel de um vértice, publicada, em forma fechada, sem iteração:

- **Catmull-Clark**, vértice de valência `n` (Halstead–Kass–DeRose 1993 §3):

  ```
  P∞ = ( n²·V  +  4·ΣE  +  ΣF ) / ( n·(n + 5) )
  ```

  onde `V` é o vértice, `ΣE` a soma dos pontos médios das arestas do anel e `ΣF` a soma dos
  centroides das faces incidentes. Para `n = 4` isto é `(16V + 4ΣE + ΣF)/36`.

- **Loop**, vértice de valência `n` (Loop 1987): com `β(n)` o peso de suavização do esquema,

  ```
  P∞ = ( V  +  (3/(8·β(n)))⁻¹ … )  ⇔  P∞ = ( (3/(8β) − n)·V + Σ vizinhos ) / ( 3/(8β) )
  ```

  — a forma prática é o **auto-vector à esquerda dominante** da matriz de subdivisão local, e a
  derivação está no capítulo 3 da tese; ⚠️ **derive-a, não a copie de uma tabela**: uma LUT de
  pesos por valência é exactamente o artefacto que §4.2 proíbe e que a fórmula torna desnecessário.

- **Fronteira**: ao longo de um bordo, o limite é o da **curva B-spline cúbica** do bordo,
  `(P_{i−1} + 4P_i + P_{i+1})/6`, independente do interior.

⚠️ **Ele é `O(anel)` por vértice e não itera.** O nosso `predict` já percorre exactamente a mesma
adjacência — é o mesmo passeio, com outra tabela de pesos.

### §2.4 — Um defeito do alvo que a nossa versão NÃO deve herdar

O alvo avalia sempre o limite **suave**, mesmo quando o modificador de subdivisão está em modo
simples/linear. O resultado é que o apagador «repõe» a malha numa superfície que não é a que o
artista escolheu, e o efeito é visível num cubo. É um defeito **público e aberto** do alvo
(reportado contra a 4.0, ainda vivo na linha que lemos). ⇒ **a nossa lei é *a referência é a do
esquema em vigor***, e isso é uma divergência **deliberada** a declarar no gate, não um acidente.

---

## §3 — *Density* — o pincel que não tem lei por vértice

> Identificador público do tipo de pincel: `SIMPLIFY`.

### §3.1 — ⭐⭐ Ele não move um único vértice, e isso é MEDIDO

**Este pincel não desloca vértice nenhum; todo o efeito dele é sobre o passe de TOPOLOGIA.**

⭐ **É observável, e a fixture é o controlo:** corra um traço dele com o passe de topologia
**desarmado** (o *Detailing* em **Manual** desarma-o por inteiro — §3.2) e **nenhuma posição muda**.
Medido em `densidade_detalhe_manual` (esfera de `1 681` vértices, traço de 6 pontos, auto-alisamento
a `0`):

| grandeza | leitura |
|---|---|
| `max │posição_saída − posição_repouso│` | **`0` exactamente** — não «pequeno», **zero** |
| vértices movidos | **`0` de `1 681`** |
| contagem de vértices e faces | inalterada |

⇒ com a única metade que ele arma fora de jogo, o pincel é **inteiramente inerte**. ⚠️ **O
auto-alisamento a zero é load-bearing na fixture:** ele é um passe **separado** que corre depois de
todo pincel, logo com ele ligado há movimento — e esse movimento não é deste pincel
(`densidade_autosuave05` existe para o contraste).

⚠️ **Consequência que muda o desenho:** ele não pertence à família `Dab`/`Grip` do nosso
[`brush_verb.rs`](../../../crates/ph2d-sculpt3d/src/brush_verb.rs). Um verbo novo que caia no
`dab_core` herdaria uma lei de peso que aqui não existe. O sítio dele é **o arm do passe de
topologia** ([`dyntopo.rs`](../../../crates/ph2d-app-sculpt3d/src/dyntopo.rs)), não o carimbo.

### §3.2 — O que ele faz, inteiro

O passe de topologia do alvo recebe um **modo** com duas bandeiras independentes (partir arestas
longas · colapsar arestas curtas). O modo sai das preferências de *Refine Method* da cena —
**excepto** que este pincel **acrescenta a bandeira de colapso**, aconteça o que acontecer com a
preferência. E o conjunto inteiro é ignorado quando o *Detailing* está em **Manual**.

⭐ **É isto e mais nada**, e a tabela-verdade cabe em três linhas — as quatro fixtures abaixo provam
as quatro células que importam:

| *Detailing* | a preferência de *Refine Method* pede… | **colapsar** corre? | **partir** corre? |
|---|---|---|---|
| Manual | (qualquer) | **não** | **não** |
| ≠ Manual | partir | **SIM** (é este pincel a pedi-lo) | sim |
| ≠ Manual | colapsar | sim | não |
| ≠ Manual | partir + colapsar | sim | sim |

⇒ *o colapso corre quando a preferência o pede **ou** quando este pincel está em mãos; o partir
corre **apenas** quando a preferência o pede; e o ajuste manual de detalhe desarma os dois.* É por
isso que o manual público pode dizer que ele consegue sempre colapsar mesmo com o método em
*Subdivide Edges*.

**Medido** (esfera triangulada de `1 681` vértices, detalhe constante, traço de 6 pontos):

| fixture | *Refine Method* | *Detailing* | vértices | leitura |
|---|---|---|---|---|
| `densidade_base` | *Subdivide Collapse* | Constant | `1 681 → 1 372` | a omissão |
| `densidade_refino_so_subdivide` | ***Subdivide Edges*** | Constant | `1 681 → 1 372` | ⭐ **idêntico ao anterior**: o colapso aconteceu na mesma |
| `densidade_grossa_so_colapsa` | *Collapse Edges* | Constant | `81 → 81` | malha grossa: **nada**. Ele nunca ACRESCENTA |
| `densidade_grossa_subdivide_colapsa` | *Subdivide Collapse* | Constant | `81 → 101` | a mesma malha grossa, com partir ligado: cresce (o controlo) |
| `densidade_detalhe_manual` | *Subdivide Collapse* | **Manual** | `1 681 → 1 681` | ⭐ **zero**. O *Detailing* Manual desarma o passe inteiro |

⇒ **duas leis, não uma**: *ele liga o colapso* **e** *ele não liga o partir*. Um teste que só
verificasse a primeira passaria com um pincel que também subdivide, que é outro produto.

### §3.3 — A região do passe

- **Esfera** de centro no dab e raio do pincel — o triângulo entra se o **ponto mais próximo dele**
  ao centro estiver dentro do raio (⛔ não é «algum vértice dentro»: um triângulo grande que
  atravessa a esfera sem ter vértice nela **conta**).
- ou **círculo projectado**, quando a pegada é tubular: os três vértices são projectados no plano
  da vista e a mesma pergunta é feita em 2D.
- opcionalmente **só faces de frente**.
- ⚠️ Os nós da árvore espacial são recolhidos com o raio multiplicado por **`1,25`** antes do passe —
  a região efectiva de *procura* é maior que a de *acção*, e é isso que deixa o refino recursivo
  (§3.6) alcançar vizinhança.

### §3.4 — O alvo de comprimento de aresta — as três leis, exactas

O passe recebe um comprimento máximo, e o mínimo é **derivado** dele:

```
DETALHE CONSTANTE / MANUAL:  max = 1 / ( resolução · escala_do_objecto )
DETALHE PELO PINCEL:         max = raio_do_pincel · percentagem / 100
DETALHE RELATIVO:            max = (raio_do_pincel / raio_em_pixels) · (detalhe · tamanho_do_pixel) / 0,4

min = 0,4 · max
```

⚠️ **O `0,4` do detalhe relativo e o `0,4` da razão mínimo/máximo são constantes DIFERENTES** que por
acaso têm o mesmo valor no alvo. Escritas como uma só, mexer numa move a outra — e só uma delas é a
histerese.

⛔⛔ **A nossa razão é OUTRA e está medida:** o nosso `collapse_target` usa `1/2,05 ≈ 0,4878`, e o
número não é arbitrário — o cabeçalho de
[`collapse.rs`](../../../crates/ph2d-mesh/src/collapse.rs) traz a varredura que mostra o **joelho
entre `1,8` e `2,0`**, abaixo do qual o par partir/colapsar **não tem ponto fixo** e a malha treme
para sempre. O alvo escolhe `1/0,4 = 2,5`, que está **acima** do nosso joelho e portanto também
assenta. ⇒ ⭐ **as duas são válidas; não há aqui nada a «corrigir»**, e trocar a nossa pela dele
custaria re-medir o ponto fixo com o nosso colapso, que tem outras recusas (§3.8).

### §3.5 — O critério do colapso é COMPRIMENTO, nunca erro de forma

Toda aresta **mais curta que o mínimo** dentro da região entra numa fila; nenhuma métrica de
curvatura, de erro quadrático ou de volume participa. ⚠️ Isto é o oposto da família de
Garland–Heckbert, e é uma escolha, não uma omissão: sob um pincel, o artista quer densidade
uniforme, não o menor erro geométrico.

**A ordem da fila** é o comprimento ao quadrado, **penalizado por proximidade a bordo**:

```
prioridade = comprimento²  ×  1,50   se a aresta é de BORDO
prioridade = comprimento²  ×  1,25   se algum extremo dela TOCA um bordo
prioridade = comprimento²            caso contrário
```

com a fila a servir sempre a **menor prioridade primeiro**. ⇒ *as arestas interiores são todas
tratadas antes de qualquer aresta que toque o bordo*, e as de bordo por último. A razão é
funcional: colapsar interior não distorce o contorno; colapsar bordo, sim.

⚠️ **«Bordo» aqui é mais do que um buraco:** conta como bordo uma aresta marcada como costura, uma
aresta marcada como dura (não-suave) **e** uma aresta não-manifold.

### §3.6 — O partir de arestas longas (a outra metade, que este pincel NÃO liga)

Fica aqui porque o mesmo passe a corre e porque o nosso `refine_in_sphere` é o par dela: a fila de
arestas longas serve **a mais longa primeiro** (prioridade `−comprimento²`), e ela **propaga-se para
fora da região** por uma recursão com duas constantes:

```
uma aresta vizinha entra se  comprimento² > max( 1,2 · comprimento²_da_que_a_chamou ,
                                                 (1,6 · limite_da_geração)² )
```

⚠️ **Os dois factores são REQUISITOS, e cada um responde por uma coisa diferente:**
- **`1,6` garante TERMINAÇÃO.** O limite cresce geometricamente a cada geração, logo a condição
  deixa de poder ser satisfeita a uma distância finita do dab, qualquer que seja a malha. Sem ele a
  propagação é ilimitada e um dab pode reescrever a peça inteira.
- **`1,2` é um PISO RELATIVO.** Uma vizinha só entra se exceder por esse factor a aresta que a
  chamou ⇒ a propagação **exclui** vizinhas de comprimento comparável e só segue um gradiente
  estrito de comprimento.

⛔ **O colapso não tem recursão nenhuma**: ele vê só o que está na região.

### §3.7 — O colapso de uma aresta: quem sobrevive e onde ele fica

1. **Se algum extremo toca um bordo**, sobrevive o que toca (se ambos, o primeiro).
2. **Senão**, sobrevive o **MAIS mascarado** — apaga-se o menos mascarado. ⚠️ Isto é o contrário do
   palpite: a máscara protege o vértice, e proteger significa *ficar*.
3. As faces em volta do apagado são recosidas ao sobrevivente. ⚠️ **Invariante exigida da saída: a
   malha não pode conter duas faces com a MESMA tripla de cantos.** Onde o recoser produziria uma
   repetida, **ambas** saem — a que nasceria e a que já lá estava.
4. ⭐ **O sobrevivente move-se para o PONTO MÉDIO dos dois**, e a normal dele passa a ser a soma
   normalizada das duas — **excepto** se ele toca um bordo, caso em que **não se mexe**, para o
   contorno não mudar de forma sozinho.

**A máscara também filtra a ENTRADA na fila**, e a regra é generosa: uma aresta entra se **algum**
dos dois extremos tem máscara `< 1` (só um par totalmente mascarado é recusado). Vértices
**escondidos** são recusados sempre.

⛔ **Um limiar a meio caminho (`0,5`) esteve em vigor e foi RETIRADO** — recusa medida por
terceiros, registada na história pública do alvo. O defeito que ela produzia é **estrutural e
previsível**: a máscara atenua por **rampa** e um limiar decide por **degrau**, logo a densidade
saltava ao longo de uma curva de nível da máscara, onde a geometria não salta. ⇒ a lei que fica é
***só a máscara SATURADA protege a topologia***. ⚠️ E isso **não** deixa a região atenuada
desprotegida: a cadeia de §1 já escala todo movimento por `1 − máscara`, logo ali a geometria muda
pouco e o passe encontra menos aresta fora de faixa — *a protecção é consequência da lei de peso,
não uma segunda cerca*.

**Medido:** `densidade_mascara_metade` (metade da esfera mascarada) — dos vértices ao alcance,
`110` de `114` sobrevivem no lado mascarado e **`0` de `112`** no lado livre.
`densidade_borda` (malha com contorno aberto) — `1 681 → 1 501` contra `1 681 → 1 372` sem bordo.

### §3.8 — ⛔ As quatro recusas do NOSSO colapso, e por que elas não se apagam

O nosso [`collapse.rs`](../../../crates/ph2d-mesh/src/collapse.rs) recusa colapsar quando: a aresta
não tem exactamente duas faces · **algum dos quatro vértices está na beira** · os dois anéis
partilham mais que os dois vértices opostos (a condição de elo) · um vizinho já foi tocado nesta
rodada. A **segunda** é mais dura que a do alvo, que em vez de recusar *escolhe o sobrevivente*.

⚠️ **Alinhar-se ao alvo aqui é uma mudança de produto, não um bug-fix**, e tem um preço nomeado: a
condição de elo (a nossa terceira recusa) é o que impede duas faces com os mesmos três cantos, e o
alvo resolve o mesmo perigo **a jusante**, apagando a face duplicada depois de a criar. As duas
estratégias são válidas; a nossa é a barata. ⇒ **decisão de produto**, com as duas frases que a põem
e sem terceira saída:
- *o `Density` colapsa também no bordo, como o alvo, e herdamos o cuidado de mover-ou-não o
  sobrevivente*; ou
- *o `Density` respeita as recusas que o nosso colapso já tem, e no bordo ele simplesmente não come*
  — mais conservador, e observável como «o pincel não afina a borda».

### §3.9 — O degrau, por metade

- **O motor** (achar arestas curtas numa esfera, colapsar sem partir a malha) é **T0**: ele já vive
  no repo, portado do SculptGL (MIT) com atribuição, e não há nada a reimplementar.
- **A semântica** — *quem é este pincel*, o que ele liga, o que ele não liga, o desarme pelo
  *Detailing* Manual, a ordem por bordo, quem sobrevive, o ponto médio — é **T2**: ela vem desta
  espec e dos vectores de §7.

---

## §4 — *Erase Multires Displacement*

> Identificador público do tipo de pincel: `DISPLACEMENT_ERASER`.

### §4.1 — A lei, inteira

Para cada vértice da grelha do nível de topo, num dab:

```
p  ←  p  +  f · ( R(p) − p )
```

com `f = peso(§1) × min(factor_de_força, 1)` e `R(p)` o ponto da superfície de referência (§2)
correspondente àquele elemento de grelha. Depois, o recorte/travas do modo, e escreve-se.

⇒ **é uma interpolação linear pura em direcção à referência**, sem direcção privilegiada, sem
normal, sem acumulador.

**Medido** (`apagador_*_constante_1passo`, curva *Constant*, UM dab, esfera com três níveis):

| força UI | fracção percorrida até à referência (p50 / min / max) |
|---|---|
| `1,0` | `1,000000` / `−0,000002` / `1,000000` — ⭐ o vértice pousa **na** referência |
| `0,5` | `0,250000` / `−0,000002` / `0,250029` |

⚠️ O `−0,000002` é o piso: vértices cujo deslocamento já era ~zero, onde a razão é `0/0` numericamente.

### §4.2 — A componente perpendicular é zero (e é isso que prova a lei)

Se a lei fosse *«mover ao longo da normal até à referência»*, o movimento teria componente
perpendicular a `R(p) − p`. Medido sobre `apagador_base`: a componente perpendicular máxima é
**`3,48e-08`** — ruído de `f32`. ⇒ o movimento é **colinear** com `R(p) − p`, sempre.

### §4.3 — Fronteiras e recusas

- ⛔ **Só em modo de multirresolução.** Sem pilha, o alvo não tem sequer o dado de entrada; o
  irmão-filtro dele (o *Erase Displacement* do *Mesh Filter*) **estoirou** publicamente por não
  verificar isso (reportado contra a 4.3). ⇒ a nossa versão **recusa em voz alta** e o gate é a
  recusa, não o resultado.
- ⛔ **Inverter não faz nada** (§1.2), com fixture byte-idêntica.
- O tecto `min(f, 1)` significa que ele **nunca ultrapassa** a referência — não há «apagar demais».
- O manual sugere usá-lo depois de *Apply Base*. Isso é conselho de fluxo, não uma pré-condição do
  algoritmo: ele corre sobre qualquer pilha.

### §4.4 — ⭐⭐ O que ele significa NA NOSSA CASA

Com o §2.3 construído, este pincel é **quatro linhas** sobre o que já temos:
`R(p)` = ponto-limite do nível de baixo avaliado na posição de grelha do vértice; o resto é a
interpolação e a cadeia de peso que o `dab_core` já entrega.

⛔ **Sem o §2.3, a tentação é usar `predict` como referência — e isso é OUTRO pincel.** Ele
existiria, funcionaria, e encolheria a peça a cada uso (§2.1). Se o dono quiser a versão barata,
ela tem de ter **outro nome** e o doc tem de dizer de que superfície ela repõe; ⛔ chamar-lhe
*apagador de deslocamento* seria a cena de smoke que ensina o contrário do que acontece.

---

## §5 — *Smear Multires Displacement*

> Identificador público do tipo de pincel: `DISPLACEMENT_SMEAR`.

### §5.1 — O que ele é

Ele **não move o vértice para onde a mão vai**: ele move o **campo de deslocamento** sobre a
superfície de referência, como quem arrasta uma textura sobre uma forma fixa. A malha-base nunca é
tocada — o que muda é quanto cada elemento da grelha se afasta da referência.

### §5.2 — A lei, passo a passo

**No início do traço** (uma vez, e só uma) o estado do traço passa a ter:
1. a superfície de referência (§2) da peça **inteira**, `R`;
2. um campo de deslocamento `D`, **cujo valor inicial é zero em todo vértice** — ⚠️ zero, e não «o
   deslocamento actual»: a diferença é observável e está no §5.4.

**Em cada dab**, sobre os nós tocados:
3. `D[u]` passa a valer o deslocamento actual desses vértices: `D[u] = p[u] − R[u]`;
4. o deslocamento de cada vértice `v` da região passa a ser uma **média ponderada do campo sobre a
   vizinhança de `v` na grelha**, e o vértice é movido uma fracção do caminho até lá.

**A média, em forma fechada.** Seja `d̂` a direcção unitária do modo (§5.3) e, para cada vizinho `w`
de `v`, seja `ê_w` a direcção unitária de `R[v]` para `R[w]` **medida na superfície de referência**.
O peso de cada vizinho é a **parte negativa do cosseno** entre as duas,

```
g(w) = max( 0, −( d̂ · ê_w ) )          ∈ [0, 1]
```

e o vértice entra na própria média com peso **exactamente 1**:

```
D′[v] = ( D[v]  +  Σ_w g(w)·D[w] )  /  ( 1 + Σ_w g(w) )

p[v] ← lerp(  p[v] ,  R[v] + D′[v] ,  peso(§1) × clamp(factor_de_força, 0, 1)  )
```

⭐ **Três coisas que a leitura rápida inverte:**
- `g` é a parte **negativa** do cosseno, logo **só os vizinhos a montante contribuem** (quem está do
  lado para onde a mão vai tem peso zero) — é isso que faz o deslocamento *viajar* em vez de borrar
  por igual;
- a vizinhança é medida **na superfície de referência**, nunca nas posições deslocadas: é por isso
  que esfregar repetidamente não deforma a topologia;
- o peso próprio é **`1` fixo**, não normalizado com os outros — é o travão que impede a vizinhança
  de dominar a média por mais vizinhos que haja a montante.

⚠️ **Nota de custo (§4.1.8), não de resposta:** um vizinho com `d̂ · ê_w ≥ 0` tem `g(w) = 0` e
portanto **não contribui para nenhuma das duas somas**. Testá-lo antes de calcular poupa a leitura
de `D[w]` e a divisão, e é o que uma implementação quente faz — ⛔ mas é **optimização**, e a
resposta é a mesma com ou sem ela. *Um ramo que não muda o valor não é lei; escrevê-lo como lei é
como o alvo está organizado, não o que ele calcula.*

### §5.3 — Os três modos (`smear_deform_type`)

| valor público | direcção | o que o artista vê |
|---|---|---|
| `DRAG` | o movimento do cursor entre dabs | o detalhe **acompanha a mão** |
| `PINCH` | do vértice **para** o centro do dab | o detalhe adensa-se no centro; endurece feições **sem** apertar a malha |
| `EXPAND` | do centro **para** o vértice | o detalhe espalha-se para fora; alisa |

⚠️ **`DRAG` com o cursor parado tem direcção nula** ⇒ `d̂` é degenerado e **nenhum** vizinho passa o
teste do cosseno ⇒ o acumulado é só `D[v]` e `novo == p[v]`: o pincel fica **inerte**. Medido:
`esfregao_drag_parado` move `140` vértices com `max|Δ| = 1,3e-03` (contra `2,4e-02` no traço que
anda). ⛔ `PINCH` e `EXPAND` **não** têm essa degenerescência — a direcção deles nasce da geometria,
não do movimento; `esfregao_pinch_parado` mede `2,0e-02`, quinze vezes mais.

### §5.4 — O artefacto que os autores escolheram MANTER

O campo `D` só é posto em dia **nos nós tocados** pelo dab, mas a média lê vizinhos que podem estar
**fora** deles. Esses vizinhos entram com o valor que o campo tinha — no primeiro dab, **zero**. ⇒
na orla da pincelada o esfregão mistura deslocamento real com zeros, e isso **come** deslocamento na
borda.

⭐ **O valor inicial zero é a cura PUBLICADA de uma regressão**, não uma escolha de conveniência:
houve uma versão em que vértices fora dos nós tocados chegavam à média **sem valor definido** e
propagavam **NaN** pela malha, e o defeito foi fechado fixando o zero. ⚠️ **O que a cura NÃO fez foi
alargar a actualização à vizinhança** ⇒ a orla é um artefacto **tolerado**, com a fronteira do
mecanismo à vista. A nossa versão escolhe: reproduzi-la (paridade) ou pôr o campo em dia **um anel
para fora** (melhor, e divergência a declarar).

### §5.5 — A alegação dos autores sobre conservação, MEDIDA

Os autores descrevem o pincel dizendo que o deslocamento total da área afectada não muda — é o que
justifica poder esfregar repetidamente sem acumular artefactos. **Medido sobre a peça inteira**
(soma de `|p − R|` antes e depois de um traço de 6 pontos):

| fixture | antes | depois | deriva |
|---|---|---|---|
| `esfregao_drag` | `154,6165` | `154,5045` | **`−0,07 %`** |
| `esfregao_pinch` | `154,6165` | `154,7007` | **`+0,05 %`** |
| `esfregao_expand` | `154,6165` | `154,0278` | **`−0,38 %`** |
| `esfregao_drag_longo` | `154,6165` | `153,6649` | **`−0,62 %`** |

⇒ ⭐ **a alegação é boa a menos de 1 %, e não é uma invariante exacta** — a média com peso próprio
`1` e a orla do §5.4 tiram um pouco, e a deriva **cresce com o comprimento do traço**. ⚠️ Um gate
escrito como igualdade exacta reprova o próprio alvo; a barra honesta é **`< 1 %` num traço curto**,
com a direcção da deriva livre (ela muda de sinal entre modos).

### §5.6 — Fronteiras

- ⛔ **Só multirresolução**, como o apagador.
- ⛔ **Inverter não faz nada.**
- ⚠️ Combinar com topologia dinâmica é **incoerente por construção** (uma pilha de níveis e uma
  malha que muda de contagem não coexistem) e há relato público de estoiro nessa combinação. A nossa
  casa já tem a recusa escrita para o caso gémeo — o `refine_for_dab` recusa com a pilha montada.
- Defeitos públicos do alvo neste pincel: buracos e áreas a desaparecer, reportados contra 4.3 e
  4.3.2, o segundo com a causa NaN do §5.4. ⇒ **a barra de paridade tem de incluir o lado aprovado**
  (§8.3): fixtures que reproduzam a orla não são «o nosso defeito».

---

## §6 — *Scene Project*

> Identificador público do tipo de pincel: `SCENE_PROJECT`.

### §6.1 — Quem são os alvos

Todo objecto da camada de vista que **não** é o activo, **é** malha, e **não** está escondido. De
cada um toma-se a **geometria AVALIADA** (modificadores aplicados) e constrói-se uma árvore de
raios sobre os triângulos dela.

**Medido:** a mesma esfera-alvo grossa, sem e com subdivisão aplicada, dá deslocamento máximo
`0,480385` contra `0,499005` ⇒ ⭐ **é a geometria avaliada, não a base**. (Houve relato público do
contrário contra a 5.2 Alpha; a versão que medimos lê a avaliada.)
Sem nenhum outro objecto, ou com o único alvo escondido: **zero vértices movidos**.

### §6.2 — A direcção do raio (`project_ray_direction_type`)

| valor público | direcção |
|---|---|
| `VIEW_NORMAL` | o **oposto** da normal da vista — i.e. para dentro do ecrã, afastando-se do observador |
| `PLANE_NORMAL` | o **oposto** da normal do plano do pincel (a normal da área sob o dab) |

⭐ **É UMA direcção para o dab inteiro**, não uma por vértice. A normal do vértice foi testada pelos
autores e rejeitada como inutilizável (registado no pedido público que introduziu o pincel).

**Medido:** com o plano-alvo virado para `+X` e `PLANE_NORMAL`, o deslocamento é inteiramente em
`−X` (`dx ∈ [−1,200, −0,005]`, `dy = dz = 0`).

### §6.3 — A distância, e as três regras que a decidem

Para cada vértice com peso ≠ 0:

1. lança-se um raio da posição dele, na direcção do dab, contra **cada** alvo, e guarda-se o `d`
   de **menor valor absoluto** entre todos os alvos;
2. se `use_bidirectional`, lança-se **também** na direcção oposta, e o resultado desse é
   **`−d`** — entrando na mesma comparação por valor absoluto;
3. **nenhum acerto em nenhum alvo ⇒ `d := 0`** (o vértice não se move — não há «projectar para o
   infinito»);
4. **houve acerto ⇒ `d := d − minimum_distance`**.

⚠️ **O raio é lançado no espaço do ALVO** (a árvore dele foi construída lá) e **`d` não se
converte de volta.** A justificação é uma derivação de duas linhas, e está aqui porque ela é
também o **requisito de implementação**: se o impacto satisfaz `Q = P + d·N`, então para qualquer
afim `M` vale `M·Q = M·P + d·(M·N)` — o **mesmo** `d` resolve a equação do outro lado. ⇒ `d` é o
**parâmetro da recta**, e não um comprimento; só voltaria a ser um comprimento se `M` preservasse
norma. ⚠️ **O requisito que daí sai:** a direcção tem de atravessar a matriz como **direcção**
(sem translação) e a posição como **ponto** — trocar os dois dá um `d` plausível e errado.
**Medido:** `projectar_alvo_inclinado_escalado` atinge o plano exactamente (`|dz|max = 0,500000`).

### §6.4 — ⛔⛔ As DUAS armadilhas do `minimum_distance`, as duas medidas

O passo 4 subtrai a folga de um `d` **com sinal**, e daí saem dois comportamentos que ninguém
prevê lendo o rótulo:

**(a) Folga maior que o vão ⇒ a peça AFASTA-SE.** Com o alvo a `0,5` abaixo e a folga a `0,6`:
`d = 0,5 − 0,6 = −0,1`, e o vértice move-se `0,1` **no sentido contrário**.
Medido, `projectar_mindist06`: `dz ∈ [+0,000507, +0,100000]` — **positivo**, contra
`dz ∈ [−0,400, −0,002]` da mesma cena com folga `0,1`.

**(b) Num acerto para trás, a folga é SOMADA em magnitude.** Com o alvo **acima** e
`use_bidirectional`, o acerto vem com `d < 0`; `d − folga` fica **mais negativo**, ou seja o vértice
vai **mais longe** em vez de parar antes.
Medido, `projectar_mindist01_acima_bidir`: `|dz|max = 0,600000` contra `0,500000` sem folga —
a folga **cresceu** a excursão em exactamente `0,1`.

⇒ ⚠️ **A folga só é «distância mínima» no sentido de avanço.** Se a quisermos simétrica, a lei é
`d := sign(d) · max(0, |d| − folga)`, que é uma **divergência deliberada** a declarar com gate nos
dois sinais; adoptar o alvo tal-e-qual é reproduzir um defeito. **Decisão de produto**, com as duas
frases que a põem e sem terceira saída.

### §6.5 — A translação

```
translação = direcção_do_raio · d · peso(§1) · factor_de_força(§1.1)
```

Depois, recorte/travas do modo, e aplica-se. ⛔ **Não há normalização por área, nem acumulador, nem
memória entre dabs** — cada dab re-mede a distância a partir de onde o vértice está agora.

### §6.6 — Inverter, e por que a simetria do traço NÃO é exacta

Por **dab**, o sinal de inversão entra no factor de força e a translação **nega exactamente**.
Sobre um **traço**, não: medido, `projectar_base` contra `projectar_invertido` dá
`max |d + d′| = 1,415e-01` sobre uma excursão de `0,5`.

⭐ **O mecanismo é a re-medição do §6.5:** o dab `k+1` mede a distância a partir da posição que o dab
`k` deixou. Na direcção do alvo essa distância **encolhe** (o vértice converge para a superfície e
pára); ao contrário, ela **cresce**. ⇒ *a lei é antissimétrica e o processo não é*, e um gate que
exija espelho exacto sobre um traço reprova o alvo.

### §6.7 — Fronteiras e casos de borda

| caso | o que acontece | fixture |
|---|---|---|
| nenhum outro objecto | nada se move | `projectar_sem_alvo` |
| único alvo escondido | nada se move | `projectar_alvo_escondido` |
| alvo do lado errado, sem os dois sentidos | nada se move | `projectar_acima_sem_bidir` |
| o mesmo, com os dois sentidos | alcança | `projectar_acima_bidir` |
| dois alvos do mesmo lado | ganha o mais perto | `projectar_dois_abaixo` |
| um alvo de cada lado, dois sentidos | ganha o de menor `|d|` | `projectar_dois_lados_bidir` |
| simetria ligada | a lei corre por passe de simetria, e o número de vértices movidos **dobra** (`527` contra `301`) | `projectar_simetria_x` |

### §6.8 — O que ele significa NA NOSSA CASA

⭐ **Tudo o que ele precisa já existe**: as peças (`cena.rs`: `objects` + `active`), a pose por peça
(logo a matriz activa→alvo), e o lançamento de raio com octree (`ph2d_mesh::ray::raycast`). O que
não existe é **a árvore de raios das peças NÃO activas mantida durante o traço** — hoje o octree é
construído para a peça que se esculpe.

⚠️ **A lista de alvos fotografa-se no pen-down**, não a cada dab: é a mesma lei que o nosso pincel
de tecido já pagou para a colisão (uma peça que se mova a meio do traço não se move para o efeito).
⚠️ **Custo:** um raio por vértice ao alcance **por alvo** e **por dab**, dobrado se
`use_bidirectional`. A nossa própria medição de colisão de tecido mediu `2,6×`–`6,1×` o custo de um
dab por **um** obstáculo — ⇒ este pincel nasce com o mesmo perfil e o tecto tem de ser **medido**,
nunca escolhido (CLAUDE.md §0.0).

---

## §7 — OS VECTORES DE TESTE

`docs/3D/cleanroom/fixtures/unblocked/` — **60** fixtures, uma por gate, em quatro famílias
(`densidade` 14 · `apagador` 10 · `esfregao` 12 · `projectar` 24). Formato, proveniência e a régua de
excepções: o [`README.md`](fixtures/unblocked/README.md) da pasta.

Cada ficheiro é texto comprimido com um cabeçalho de `# chave: valor` (em vocabulário nosso, mais os
identificadores **públicos** da API que tornam a corrida regenerável) e blocos de uma linha por
elemento:

| prefixo | o que é |
|---|---|
| `r` | posição de **repouso** (antes do traço) |
| `l` | posição na **superfície de referência** (§2) — só nas famílias de multirresolução |
| `s` | posição de **saída** (depois do traço) |
| `c` | os pontos do **cursor**, pela ordem |
| `fr` / `fs` | **faces** antes / depois — só na família `densidade`, onde a topologia muda |

⭐ **O bloco `l` é o que torna as famílias de multirresolução utilizáveis sem termos o §2.3 pronto:**
ele é a superfície de referência do alvo, medida, o que deixa a lei de §4.1 e §5.2 ser cobrada
**isoladamente** da questão de quem calcula o limite. ⇒ *duas waves, não uma*: primeiro a lei contra
o `l` dado, depois o nosso avaliador de ponto-limite contra o `l` dado.

---

## §8 — A BARRA DE PARIDADE, E DE ONDE ELA SE DERIVA

### §8.1 — ⭐⭐ O próprio alvo NÃO é reprodutível ao bit

A mesma entrada, a mesma versão, duas corridas:

| família | `max |Δposição|` | faces iguais |
|---|---|---|---|
| `densidade` (detalhe **constante**) | **`0,000e+00`** | **sim** |
| `projectar` | **`0,000e+00`** | sim |
| `apagador` | **`1,788e-07`** | sim |
| `esfregao` | **`2,384e-07`** | sim |
| `densidade` (detalhe **relativo**) | ⛔ **contagens diferentes** (`1 360` · `1 362` · `1 367` · `1 371` · `1 368` em cinco corridas) | — |

⇒ **a barra dos dois pincéis de multirresolução não pode ser mais apertada que `~2,4e-07`** numa
peça de raio `1`, e o número **não é uma escolha**: é a não-associatividade de `f32` sob a
paralelização por nó do alvo. Uma barra de `1e-8` reprovaria o alvo contra ele mesmo.

⚠️ **A quinta linha é sobre a nossa MEDIÇÃO, não sobre o algoritmo:** o detalhe relativo depende do
raio **em pixels** e do tamanho do pixel, e o nosso arnês não prega o estado do viewport entre
corridas. ⇒ ⛔ **não construa gate sobre o detalhe relativo** até alguém pregar a vista; o detalhe
**constante** é byte-idêntico e é o caminho sobre o qual se mede.

### §8.2 — ⛔ O que NÃO pode ser um golden numérico

Os blocos `s` das famílias `apagador` e `esfregao` **não são goldens da nossa saída** sobre a mesma
entrada, e a razão é o §2.2: a referência é outra superfície, logo a resposta certa é outra. Eles
são goldens **da lei** quando alimentados com o bloco `l` da própria fixture. ⚠️ Ignorar isto produz
um gate vermelho que parece um bug do produto e é um bug da régua — o defeito que esta casa já pagou
cinco vezes.

### §8.3 — A barra inclui o lado APROVADO

O alvo tem, nestes quatro pincéis, defeitos públicos vivos ou recentes: a referência errada em
subdivisão simples (§2.4), a orla que come deslocamento (§5.4), a folga assimétrica (§6.4). ⇒ um
gate de artefacto calibrado só contra a NOSSA saída mede os nossos defeitos. **Toda barra de
artefacto desta obra tem de ser corrida contra a saída do alvo primeiro**, e a que reprovar o alvo
**sai** — é a lei que o corpus do pincel de tecido já impôs a duas barras desta mesma linha.

### §8.4 — Comparar POR PASSO

As famílias `apagador`, `esfregao` e `projectar` têm fixtures de **um dab** (`*_constante_1passo`) e
de **traço inteiro**. ⭐ Comece pelo de um dab com curva *Constant*: ali o peso é `1` no miolo e a
lei fica **sozinha** no numerador. Um traço inteiro com curva *Smooth* mistura seis aplicações da
curva com a re-medição do §6.6, e um desvio nele não diz **qual** dos dois falhou.

---

## §9 — A SABEDORIA DOS AUTORES (§4.1.12), re-dita

Tudo aqui saiu de mensagens de commit, do pedido público que introduziu o pincel, do relatório de
defeitos público e do manual. **Nada é transcrito**; os endereços ficam no ledger.

1. **A força ao quadrado é deliberada** e o motivo é ergonómico: dar curso útil à metade de baixo do
   slider. ⇒ ⛔ não «corrigir» para linear sem decidir o produto.
2. **A direcção de raio por normal do vértice foi construída e rejeitada** pelo autor do
   `SCENE_PROJECT` como inutilizável. ⇒ **recusa medida por terceiros**: não a reconstrua sem um
   motivo novo.
3. **O pincel foi portado de um ramo experimental** onde ele usava o sistema de *snap* do
   aplicativo; a versão que shipa trocou-o por lançamento de raio directo **por custo**. ⇒ a rota
   barata é a que sobreviveu.
4. **A ideia declarada e não construída** é projectar sobre uma **fotografia do próprio objecto**
   (uma versão passada de si mesmo) e um pincel-irmão que desfaça a projecção por região. ⚠️ Isto é
   **exactamente** o que a nossa pilha de undo por traço já sabe guardar — pode sair mais barato
   aqui do que lá.
5. **O corte a 50 % de máscara no passe de topologia foi tentado e retirado** por deixar uma borda
   visível na malha. ⇒ a regra generosa do §3.7 é uma cura, não um descuido.
6. **A conservação do deslocamento é uma alegação de desenho**, não uma invariante — §5.5 mede-a.
7. **O manual recomenda *Apply Base*** antes dos dois pincéis de multirresolução. É conselho de
   fluxo: os dois correm sobre qualquer pilha, mas a referência fica mais perto da forma autorada
   depois daquele passo, e o resultado é mais previsível.
8. **Existe um pedido público aberto** para tirar os ajustes de topologia dinâmica da cena e pô-los
   **no pincel** (resolução, tamanho de detalhe, fracção do pincel, método de refino, modo de
   detalhe). ⚠️ ⭐ **A nossa casa já está do lado certo dessa mudança** — o
   [`Dyntopo`](../../../crates/ph2d-app-sculpt3d/src/dyntopo.rs) guarda `detail` como **fracção**
   contra o raio do pincel, exactamente para o número não mudar de significado ao trocar de pincel.
   ⇒ ⛔ não copie o modelo de cena do alvo: ele está a caminho de o abandonar.

---

## §10 — O QUE FICA ABERTO (e o que é decisão do dono)

| # | item | estado |
|---|---|---|
| 1 | **O avaliador de ponto-limite** (§2.3) — a única peça de substrato que falta, e ela serve os DOIS pincéis de multirresolução | ⏳ espec pronta acima; fórmula pública; `O(anel)` |
| 2 | **O bordo no `Density`** (§3.8): recusar como o nosso colapso, ou escolher o sobrevivente como o alvo | ⏳ **decisão do dono**, com as duas frases |
| 3 | **A folga do `SCENE_PROJECT`** (§6.4): reproduzir a assimetria ou torná-la simétrica | ⏳ **decisão do dono**, com as duas frases |
| 4 | **A orla do esfregão** (§5.4): reproduzir o artefacto ou alargar a actualização um anel | ⏳ divergência a declarar, qualquer que seja |
| 5 | **A referência em subdivisão simples/linear** (§2.4) | ⭐ divergência **deliberada**: seguimos o esquema em vigor, e o gate di-lo de si mesmo |
| 6 | **O tecto de custo do `SCENE_PROJECT`** (§6.8) | ⏳ **por MEDIR** — um raio por vértice por alvo por dab |
| 7 | **O detalhe relativo** (§8.1) | ⛔ sem gate até alguém pregar o estado da vista |
| 8 | **O irmão-filtro do apagador** (a peça inteira, sem pincel) — o alvo tem-no no *Mesh Filter* | ⏳ fora desta espec; a nossa família de filtros tem nove leis e este seria a décima |
| 9 | **A auditoria R-pré** desta espec | ⏳ **condição de abrir a janela que implementa** |

---

## ⛔ Recusas MEDIDAS (não as reconstrua)

| o quê | por quê | onde |
|---|---|---|
| Força linear nos três que deformam | o alvo eleva ao quadrado **de propósito**, e a medição dá `0,250000` a meio curso | §1.1 |
| Inverter o apagador / o esfregão | o sinal não entra no factor deles; saída **byte-idêntica** medida | §1.2 |
| Pôr o `Density` na família `Dab`/`Grip` | ele não tem lei por vértice; a força dele é **zero e inutilizada** | §3.1 |
| Fazer o `Density` também subdividir | medido: malha grossa com refino «só colapsar» fica **`81 → 81`** | §3.2 |
| Trocar a nossa histerese `1/2,05` pela `1/2,5` do alvo | a nossa saiu de uma varredura com joelho medido em `1,8–2,0`; as duas assentam | §3.4 |
| Critério de colapso por erro de forma (quádricas) | o alvo usa **comprimento puro**; sob pincel quer-se densidade uniforme | §3.5 |
| Usar `predict` como superfície de referência do apagador | encolhe a peça; é **outro pincel**, e o limite do cubo unitário mede `0,4198` | §2.1 · §4.4 |
| Direcção de raio por normal do VÉRTICE | construída e rejeitada pelo autor do pincel como inutilizável | §9.2 |
| Gate de conservação exacta no esfregão | medido `−0,07 %` a `−0,62 %`: reprovaria o próprio alvo | §5.5 |
| Gate de espelho exacto ao inverter a projecção sobre um TRAÇO | `max|d + d′| = 1,415e-01`: a lei é antissimétrica, o processo não | §6.6 |
| Barra de paridade abaixo de `~2,4e-07` nos de multirresolução | o alvo não se reproduz a si mesmo melhor que isso | §8.1 |
| Gate sobre o detalhe **relativo** | cinco corridas, cinco contagens; a vista não está pregada | §8.1 |
| Copiar o modelo de ajustes de topologia da CENA | o alvo tem pedido público aberto para os mover para o pincel; nós já estamos do lado certo | §9.8 |

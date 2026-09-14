# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, **o PINCEL DE CONTORNO** (2026-09-14)

> **Estado:** a linha FECHOU esta fatia e PAROU. ⛔ Não integrou, não pushou (§0.7).
> **Merge-base:** `1d43da737`. **Contadores partilhados: ZERO se movem** —
> `PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`, `FLIP_SCHEMA`, `FIELD_DOC_VERSION`, o
> `DOC_VERSION` da timeline e os três registos de componentes estão **iguais ao
> merge-base** (`git diff 1d43da737 -- shells/desktop/src/project_schema.rs
> crates/ph2d-vec-scene/src/schema.rs` devolve **0** linhas). Zero contrato
> congelado tocado, zero ADR, zero pacote externo novo.
>
> As jornadas anteriores desta linha têm handoff próprio:
> [13/09 — os gates nomeados](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-13.md) ·
> [13/09 — os pincéis (tangenciais + POSE + o osso + o undo)](HANDOFF_INTEGRACAO_line_sculpt3d_PINCEIS_2026-09-13.md).
> Este cobre **só** o que veio depois.

---

## §1 — O que fechou

1. ⭐⭐⭐ **O PINCEL DE CONTORNO existe** — crate-folha nova
   [`ph2d-boundary`](../../../crates/ph2d-boundary/) (**zero dependências**, como
   a `ph2d-cloth` e a `ph2d-pose`), clean-room sob
   `SPEC_boundary_brush.md` (atestada à **3.ª** passagem do R-pré). O artista
   aponta para perto de uma **borda aberta** e arrasta; a borda inteira — ou um
   troço dela — deforma-se, e a deformação **esmorece para dentro** da peça.
   `Verb::Boundary` é o **28.º** verbo (conte-o em `Verb::ALL`, nunca aqui).
2. ⭐ **A cena `=42`** (`scenes_boundary.rs`) — uma TIGELA com a boca aberta,
   roteiro em 6 passos, com o `Move / Grab` no mesmo sítio para o dono
   **comparar** de onde a região sai.
3. ⭐⭐ **Uma PORTA nova em `ph2d-mesh`** — [`compact_for_faces`](../../../crates/ph2d-mesh/src/compact.rs),
   que é o que faz a `=42` funcionar (§5).
4. ⭐ **Dois censos novos**, cada um nascido de um defeito medido nesta jornada
   (§6).

---

## §2 — A lei, em fases (o que torna este pincel testável)

```text
A  censo de bordas na malha            (uma vez por malha)   → Topologia
B  escolher o vértice âncora           (pen-down)            → pode RECUSAR
C  andar a borda a partir da âncora    (pen-down)            ┐
D  propagar para dentro                (pen-down)            ├ Estrutura
E  pesos por vértice                   (pen-down)            ┘
--- por passo do traço ---
F  avanço s a partir do arrasto
G  a lei do modo
H  translação, com filtro de peso zero e de região
```

⭐ **A–E são FOTOGRAFADAS no pen-down.** É isso que torna o resultado função só
do **arrasto TOTAL** — o mesmo arrasto em `2` e em `8` eventos dá o mesmo valor
ao 7.º decimal, e a prova de fatiamento fecha a `0,00000000`.
⛔ **Uma excepção, e ela é grande: o `Modo::Suavizar`** lê a posição **actual** e
por isso **acumula com o número de eventos**; todo gate dele fixa os passos.

⚠️⚠️ **O verbo desvia antes do `dab_core` E antes do espelho, e as duas coisas
têm razões DIFERENTES** ([`stroke_boundary.rs`](../../../crates/ph2d-sculpt3d/src/stroke_boundary.rs)):
não há núcleo por-vértice nenhum (a região não sai do cursor, sai da **BORDA**);
e cada passagem de simetria **refaz A–E do zero**, incluindo escolher âncora nova
a partir do ponto reflectido. ⚠️ **A segunda razão é observável:** com a queda no
contorno diferente de `CONSTANT` as duas passagens escrevem valores **diferentes**
na sobreposição, e sem o filtro de região a segunda **apaga** a primeira.

⚠️ **O undo passa pela porta que o tecido e a pose já usam** — escrita em três
passos (*quem muda* · `capture` · escrever), com o `capture` **antes** da
escrita. Sem isso a janela do undo sai vazia e o `close_stroke` devolve cedo, que
foi exactamente o report do dono sobre a pose.

---

## §3 — ⭐⭐⭐ A espec ATESTADA foi REFUTADA por medição, e a cegueira tem nome

A `SPEC_boundary_brush.md` §9.1 dá uma fórmula para `n̂`, a direcção contra a qual
o arrasto é projectado para dar o avanço `s`. **Quatro fixturas do corpus a
refutam.** A lei que o corpus impõe é

```
n̂ = − unit(ponto_origem)
```

e com ela o corpus foi de **43 para 49** de 61 dentro da barra.

⚠️⚠️ **O ponto cego que deixou a fórmula passar pelo R-pré é do CORPUS, não do
auditor:** a fórmula da espec e esta concordam em toda fixtura cuja peça esteja
**centrada na origem**, que é a maioria do corpus. O que as separa são as peças
deslocadas — e elas eram quatro. *Uma espec atestada é uma afirmação sobre o que
o revisor podia ver; o corpus é que decide.*

⚠️ **A divergência está declarada no `lib.rs`** da crate, num doc longo sobre
`direccao_do_avanco`, com a tabela das quatro fixturas ao lado. ⛔ Não a
"corrija" para a fórmula da espec sem re-correr o corpus.

Outras duas divergências **declaradas** (com o motivo escrito no sítio):
`propagar()` usa `e.alcance = anel_actual.min(maximo)` (espec §13.1) e as médias
de anel do alisar são **Jacobi** (determinístico em toda a malha).

---

## §4 — A régua: 51 de 61, com o censo das excepções

`cargo test -p ph2d-boundary --test it mede_o_corpus -- --nocapture` imprime a
tabela. Hoje: **51 de 61 dentro de `BARRA = 1e-5`**, corpo entre `5,96e-8` e
`~1e-6`.

⛔ **As 10 excepções são NOMEADAS numa tabela com censo de obsolescência nos dois
sentidos** (`o_censo_das_excepcoes_nao_descreve_nada_de_obsoleto`): o nome tem de
continuar no corpus **e** tem de continuar a divergir. Uma entrada que passou a
fechar **sai** — senão a tabela vira licença (`CLAUDE.md` §5.0).

As piores, com a ordem de grandeza: `grade_triangulada_dobrar_constante` `9,6e-1`
· as duas `grade_dobrar_constante_pano_*` `3,0e-1` · `grade_pequena_dobrar_origem2`
`2,4e-1` · `grade_triangulada_expandir_constante` `7,6e-2` · as duas de torcer
`1,2e-2`.

⚠️ **Duas classes de fixtura mordem ANTES de medirem o produto**, e estão
documentadas no arnês: a **cúpula** chega com o enrolamento para dentro (o
harness orienta a malha para fora por uma regra de maioria **medida**, declarada
como propriedade da FIXTURA e não como lei), e a máscara `x_positivo` precisa de
`>= 0` — com `>` movemos `85` vértices e o alvo `80`, e os `5` a mais são
exactamente a coluna `x = 0`.

---

## §5 — ⛔⛔ A `=42` não funcionava, e a causa era a FIXTURA: `721` órfãos

O gate da cena reprovava com *«o contorno moveu só **0** vértices na boca da
tigela»*. Diagnosticado com uma sonda que imprime a estrutura (âncora ·
ponto-origem · alcance · cadeia · a recusa):

| o que a sonda leu | |
|---|---|
| posições | **1 490** |
| faces | **768** |
| vértices de borda | 48 |
| cursor da cena (o ponto mais alto) | `[0, 1, 0]` |
| âncora escolhida | vértice `0`, **`e_borda = false`** |
| veredito | `Recusa::SemBordaAoAlcance`, nos três raios varridos |

⭐⭐ **Escolher faces não tira posições do pool.** A metade deitada fora deixava
**`721` vértices órfãos**: nenhuma face os cita, logo o `Mesh::from_parts` aceita-os
**calado** e nada na tela muda — mas o cursor da cena é *o ponto mais alto da
peça*, e o ponto mais alto passou a ser o **pólo norte da metade que não existe**,
a `1,0` da boca. A busca da âncora aterrava nele, ele não tem aresta nenhuma,
portanto não é de borda, e a lei recusava o traço inteiro. *A queixa apontava
para o verbo e a causa era a fixtura.*

Depois de compactar: **769** posições para 768 faces, âncora **de borda**, cadeia
de **48** (a boca inteira), **144** vértices com peso não-nulo no raio de omissão,
e o gate fecha.

### ⭐⭐ A cura é uma PORTA, porque a lei já estava escrita — num sítio só e privada

[`ph2d_mesh::compact_for_faces`](../../../crates/ph2d-mesh/src/compact.rs)
devolve `(posições, faces reindexadas, de onde cada posição veio)`. A terceira
saída é o que a torna porta e não utilitário: **todo canal paralelo** (cor,
máscara, um vector por vértice de quem chama) compacta-se com um `map` sobre ela.

⚠️ **Ela nasceu dentro do importador de OBJ**, onde o doc-comment já descrevia
este defeito por escrito (*«uma nuvem de vértices órfãos que o `from_parts`
aceita, o octree indexa e a caixa da câmera enxerga»*) — e estava **privada**,
que é exactamente como o segundo consumidor a reescreve mal. Hoje o `obj::compact`
**delega**; o que sobra lá é o que é do OBJ (a cor).

Consumidores: `obj.rs` · `scenes_boundary::tigela()` · e o
`stroke_tests::sphere_with_open_cap()`, que tinha o **mesmo** defeito latente e
só escapava porque o contacto daquele gate está longe da calota.

---

## §6 — Os dois censos que esta jornada pagou

### 6.1 `toda_cena_com_roteiro_e_anunciada`

A `=42` nasceu com o roteiro de seis passos escrito e **nenhum chamador**: o
despacho em `scripts.rs` enumera as cenas **à mão**. O artista abriria a cena e
veria a peça **sem uma palavra** sobre onde clicar — que é a metade do smoke em
que ele *aprende* a ferramenta (§0.8).

⚠️⚠️ **O que o apanhou foi um `warning: function is never used`, e isso é sorte
que não se repete:** a função é `pub(crate)`, logo o `dead_code` ainda a alcança;
bastava ela ser `pub`, ou ser citada por um teste, e o aviso desaparecia com o
roteiro na mesma mudo. *Um aviso do compilador não é um gate: ele mede
visibilidade, não a lei.* O censo tem **piso de população** (`>= 14`), porque uma
varredura partida devolve lista vazia e lê-se como aprovada.

### 6.2 `o_cabecalho_de_cada_fixtura_concorda_com_o_percurso_dela`

O clippy acusava `field caminho is never read` na bancada. O percurso do cursor
ponto a ponto (as linhas `c`) era **parsado e deitado fora**, e a leitura fácil é
*«sobra do formato»*.

⚠️ **Ele não é sobra: é o ÚNICO controlo independente do cabeçalho.** Toda a
paridade desta crate é medida contra números que a própria linha `#` declara; se
um `arrasto` estivesse copiado do traço vizinho, nós reproduziríamos fielmente o
gesto ERRADO e a tabela fecharia verde. Medido: **60 de 61** fixturas têm
`contacto == c[0]` e `arrasto == c[n] − c[0]` dentro de `1e-5`.

⭐ **A 61.ª é uma CLASSE, não um defeito:** `grade_ruidosa_suavizar_1passo` tem
**um** ponto de percurso — o alisar age com o cursor **parado**, logo o percurso
é nulo e o cabeçalho traz `arrasto = 0`, e a régua trata-a **sem um ramo
próprio**. O piso `parados == 1` está lá para a excepção não evaporar em silêncio.

---

## §7 — Provas de mutação (6 de 6 sangram)

| # | mutação | quem sangra |
|---|---|---|
| M1 | `compact_for_faces` devolve o pool inteiro (deixa de compactar) | os **2** gates da porta **+ o gate da cena `=42`** |
| M2 | o remapeamento passa a tocar o 4.º slot (a sentinela `TRI`) | os 2 gates da porta |
| M3 | a `=42` deixa de ser anunciada | `toda_cena_com_roteiro_e_anunciada`, a nomear `["boundary"]` |
| M4 | o censo varre `cenas_*` em vez de `scenes_*` (**controlo negativo da varredura**) | o **piso**: *«varreu só 0 cenas»* |
| M5 | o percurso deixa de ser lido (`ultimo := primeiro`) | `o_cabecalho…`, **56 de 61** |
| M6 | o gesto parado deixa de ser contado | o piso: *«tinha UM, agora tem 0»* |

⚠️ **M5 devolve `56` e não `61`, e isso é informação:** as outras `5` fixturas
têm arrasto de cabeçalho **nulo** — são o resto da família do alisar. *Um
controlo que mata menos que a população nomeia a subpopulação que ele não
discrimina.*

---

## §8 — O que uma leitura rápida do diff entende ao contrário

1. **`compact_for_faces` não é arrumação de código.** Sem ela a `=42` move zero
   vértices. O modo de falha não se parece com a causa.
2. **A `ph2d-mesh` foi tocada, e é aditiva** — um `mod` novo mais um `pub use`;
   o `obj::compact` **encolheu** para uma delegação. É ponto de extensão, não
   mudança de contrato.
3. **O `Modo::Suavizar` acumular com o número de eventos NÃO é defeito** — é a
   lei, e está declarada no cabeçalho da crate. Um gate dele que não fixe os
   passos mede outra coisa.
4. **O `Origin offset` alonga a PROPAGAÇÃO, não a queda no contorno.** São dois
   raios; a tabela medida está no doc de [`BOUNDARY_OFFSET`](../../../crates/ph2d-panel-sculpt3d/src/rows_boundary.rs).
   O tecto `4` é de **produto** com a medição ao lado — ⛔ não é o `30` do alvo
   copiado.
5. **A tigela é cortada em `y <= 0` e a irmã do `stroke_tests` em `y <= 0.5`, e
   as duas estão certas** — cada corte tem a medição que o escolheu no
   doc-comment ao lado. ⛔ Uni-las faria uma delas deixar de conter o fenómeno.
6. **`51 de 61` não é o mesmo número que `49 de 61` de um dia antes** — entre os
   dois está a orientação medida da fixtura da cúpula e o `>= 0` da máscara, e
   nenhuma das duas é lei do produto.
7. **O gate da `=42` tem DUAS metades**, e a segunda é o controlo: a esfera
   **fechada** tem de mover **zero**. Sem ela a primeira metade é um número solto.

---

## §9 — Premissas minhas que a medição derrubou

1. *«A `=42` falha porque o arrasto `[0, d, d]` projecta a `s ≈ 0`»* — **falso**:
   os cinco arrastos varridos movem os mesmos 144 vértices. O gesto nunca chegou
   à lei.
2. *«O vértice sob o cursor falha o teste de grau ou de ramificação»* — **falso**:
   a recusa era `SemBordaAoAlcance`, e em três raios diferentes (`0,3`, `0,5`,
   `1,0`), o que já dizia que não era o raio.
3. *«O percurso da fixtura é sobra do formato»* — **falso**: ele concorda com o
   cabeçalho em 60 de 61, e é a única prova independente que o corpus tinha.
4. *«O aviso de dead-code cobre esta classe»* — **falso**: ele mede
   visibilidade. Um `announce` público teria ficado mudo sem um aviso.

---

## §10 — ⏳ ABERTO, e o que é decisão do dono

- **As 10 excepções do corpus** (§4), cada uma com a ordem de grandeza e o censo
  que as obriga a sair quando fecharem.
- **A ocultação de vértices não está ligada**, e é ausência **DECLARADA** no
  `boundary_comecar`: a lei aceita-a e a espec diz que geometria escondida
  *produz* contorno, mas essa é a única regra da espec **sem fixtura** — o
  harness do oráculo nunca esconde nada. Quem a ligar tem de encomendar corrida
  nova.
- **O censo de bordas é `O(malha)` e é refeito a cada traço.** Guardá-lo entre
  traços pede um carimbo de topologia que a `Mesh` não tem.
- ⛔⛔ **UM ACHADO DE SWEEP QUE NÃO É MEU E QUE EU NÃO DEVO CURAR SOZINHO:** a
  vassoura `blender-pull` acusa **`tip_roundness`** em `7` ficheiros de
  `ph2d-sculpt3d` e `ph2d-panel-sculpt3d`. Medições: (a) esta linha **não
  acrescentou** nenhuma dessas linhas (`git diff 1d43da737 | grep -c` das
  adições = **0**); (b) o nome sobreviveu ao commit `8c5f2a7c5`, que levou a
  `ph2d-sculpt3d` a **zero** citações — ou seja, **a vassoura foi estendida
  depois**, que é a lei de 13/09 (*o sweep é propriedade do PAR (código,
  vassoura)*). ⚠️ Ele é um **nome de controlo PÚBLICO**: renomeá-lo muda um
  `NodeId` **hasheado** (`hash_node_id("sculpt3d.tip_roundness")`) e uma chave de
  i18n (`panel.sculpt3d.tip_roundness`), logo é mudança de produto. ⇒ **ou ele é
  uma das isenções de API pública já registadas no ledger** (o handoff de 13/09
  nomeia seis, saídas de CORRER o alvo e nunca de o ler), **ou é dívida real da
  linha dona**. ⛔ O I não pode ler o `LEDGER_*` para decidir: **isto é para o R
  resolver**, e fica aqui nomeado em vez de silenciado.
- ✅ **O outro achado de sweep FOI curado nesta jornada:** um comentário do
  `ph2d-mesh/src/collapse.rs` citava um símbolo interno do alvo restrito. A frase
  foi re-dita em vocabulário do domínio (o argumento — *«a troca de diagonal é
  reparação a mais, e nós já rodamos o `dyntopo_flip` no mesmo dab»* — está
  intacto). Ele é **pré-existente** (nasceu em `77add77ad`) e escapava porque
  `ph2d-mesh` está **fora** da lista `FAMILIA` do censo da família.
  ⇒ *a família da escultura tem uma sexta crate que os sweeps de 13/09 não
  varriam.*

---

## §11 — O portão de fecho

| passo | resultado |
|---|---|
| `cargo fmt --all --check` | ✓ (formatou **a crate nova inteira**, que nunca tinha sido formatada) |
| `cargo clippy --workspace --all-targets --all-features` | ✓ **0 warnings** — três curados na `ph2d-boundary`: dois `needless_range_loop` e o `caminho` nunca lido, que virou o gate §6.2 |
| `scripts/nextest-impacted.sh` | ✓ |
| `scripts/doc-index.sh --check` | ✓ 19 índices em dia |
| as **6** vassouras clean-room sobre a família | ✓ limpo, com **um achado curado** e **um nomeado** (§10) |
| tectos de LOC (workspace + shell) | ✓ |

⚠️ **A varredura das vassouras corre-se com os paths a passar por `bash`**: numa
janela cujo shell é o `fish`, `$ALVOS` **não** se parte em palavras e o script
recusa com *«path não existe»* sobre a string inteira — que se lê como uma
vassoura partida e não como um erro de invocação.

---

## §13 — ⛔⛔⛔ O SMOKE DO DONO REPROVOU, e o primeiro report era a PONTE

> *«aparentemente errado. Não é a borda que está dobrando, mas a região interna.»*

Ele tem razão, e a medição é inequívoca. Na tigela da cena, com o cursor na boca
(`K = 4` anéis), o peso e o deslocamento por anel liam:

| anel (`0` = a BORDA) | peso | deslocamento |
|---|---|---|
| **0** | **`0,0000`** | **`0,00000`** |
| 1 | `0,1562` | `0,09623` |
| 2 | `0,5000` | `0,19720` |
| 3 | `0,8438` | `0,15244` |
| 4 | `0,0000` | `0,00000` |

Uma **corcova no miolo** com a beirada parada — a frase dele, em números.

### ⭐ O oráculo decide, e não por pouco

Medido nas fixturas do próprio corpus, o deslocamento do **alvo** por anel:

```
grade_agarrar_constante (K=5)   0,1000 · 0,0896 · 0,0648 · 0,0352 · 0,0104 · 0
grade_dobrar_constante_origem1  0,6613 · 0,5702 · 0,4566 · 0,3366 · 0,2239 · … · 0
```

Máximo **no anel 0** e monótono para dentro — e os números são exactamente
`curva(1 − anel/K)` com a *smoothstep*. ⇒ **a lei da crate estava CERTA.**

### O defeito: duas convenções opostas, ligadas sem a inversão

| quem | o argumento é | vale `1` em |
|---|---|---|
| `ph2d_boundary::Curva` | **quanto FALTA** (`1 − anel/K`) | o argumento `1` |
| `crate::Falloff::weight` | **quanto já se ANDOU** (`d/R`) | o argumento `0` |

A ponte escrevia `|p| brush.falloff.weight(p)`. Na borda isso lê `curva(1) = 0`.
**Uma linha** (`weight(1.0 - p)`), e o perfil passa a `1,212 · 1,033 · 0,825 ·
0,587 · 0,339 · 0,133 · 0,021 · 0`.

⚠️ **E ela cura DUAS coisas**, porque a mesma curva alimenta a queda de
profundidade **e** a queda ao longo do contorno: sob `Radius` o troço aceso era o
**oposto** do que o artista apontava.

### ⛔⛔ Porque `51 de 61` fecharam sobre isto

**O corpus corre a crate DIRECTAMENTE**, com a convenção dela — ele nunca
atravessa a ponte. *Uma paridade medida a montante de uma conversão não afirma
nada sobre a conversão.* E o gate da cena `=42` era **verde**: ele contava
**quantos** vértices se movem (`> 30`), e `144` movem-se nas duas orientações.
⇒ *uma régua que conta QUANTOS nunca vê QUAIS* — a mesma forma do `edge_max`
global cego ao quad fino e do `χ` cego à almofada.

⇒ o gate novo é `a_borda_e_quem_mais_se_move_e_o_efeito_morre_para_dentro`
(`ph2d-sculpt3d`, no caminho do **produto**, `SculptStroke::dab`), e ele afirma a
frase inteira: a borda move-se, o perfil é **monótono**, e ele **morre**.
⚠️ Monótono e não *«o primeiro é o maior»*: o defeito produzia uma **corcova**, e
um simples `max` deixá-la-ia passar.

⚠️ **A fixtura não continha o fenómeno à primeira**, e reprovou a apontar para o
verbo — a mesma armadilha da irmã do `stroke_tests`. A sonda que a escolheu ficou
(`diag_varre_o_puxao`): numa grelha plana o avanço é a projecção no plano, então
`[0, 0,4, 0]` (perpendicular) e `[0,4, 0, 0]` (ao longo da borda) leem **zero**.

---

## §14 — O segundo report: *«e não tem gizmo»*

Verdade: a única coisa desenhada era o **anel do cursor**, e para este verbo ele
é ainda pior descritor do que para a pose — a região não sai do cursor, sai da
**BORDA**.

⇒ [`boundary_previa`](../../../crates/ph2d-sculpt3d/src/boundary_previa.rs) (a
lei) + [`boundary_gizmo`](../../../crates/ph2d-app-sculpt3d/src/boundary_gizmo.rs)
(a figura), irmãos dos da pose e com o **mesmo orçamento**. A figura responde às
duas perguntas que o painel faz:

| o que o artista mexe | o que ele vê mudar |
|---|---|
| `Falloff along the edge` | a **fita** ao longo da beirada — a alfa de cada pedaço é o **peso** dele |
| `Origin offset` | a **linha** que mergulha na peça, com o **anel** no eixo |

E um quarto estado: **sem beirada ao alcance**, só um anel vermelho — *«daqui
este pincel não faz nada»*, onde o alvo se cala.

### ⛔⛔ A cadeia NÃO vem em ordem de passeio, e ligá-la pela ordem dela desenha CORDAS

A fase C anda a borda **nos dois sentidos ao mesmo tempo** e devolve a `cadeia`
ordenada por **distância à âncora**, alternando os lados: medido na boca da
tigela, `[94, 0, 92, 1, 90, 4, …]` com distâncias `[0, 0,131, 0,131, 0,262,
0,262, …]` — **`45` de `47`** pares consecutivos **não** são vizinhos de borda.
Uma polilinha `windows(2)` sobre ela desenha um ziguezague **através** da boca.

⚠️ **E as réguas que eu já tinha escrito ficavam verdes sobre isso:** a contagem
de pedaços é a mesma, os pesos são os mesmos, o laço fecha na mesma. ⇒ o gate é
`nenhum_pedaco_atravessa_a_boca`, e ele afirma a **RELAÇÃO**: cada pedaço
desenhado liga dois vértices **vizinhos na borda**. *Quem desenha uma ligação
tem de gatear a relação, não o número de linhas.*

O traçado passou a ser reconstruído pela adjacência: dois passeios a partir da
âncora, um por sentido, enquanto o vizinho ainda pertence à cadeia — mais o
pedaço de **fecho** quando a boca é um laço (sem ele falta sempre um vão, e é
logo o do lado oposto ao cursor).

### ⭐ O custo, MEDIDO no perfil do smoke (malha da cena `=42`, `769` vértices)

| raio | pedaços | 1.ª construção | quadro repetido |
|---|---|---|---|
| `0,30` (omissão) | 48 | `0,096 ms` | `0,29 µs` |
| `0,60` | 48 | `0,083 ms` | `0,03 µs` |
| `1,00` | 48 | `0,084 ms` | `0,03 µs` |

⇒ cabe **17×** no orçamento, logo a fita segue o cursor quadro a quadro.
⚠️ **O custo é da MALHA e não do pincel** — o raio mal o move.

### O quarto consumidor do `pick`, e porque ele é a prova de que o corte era desenho

O gate `a_stroke_belongs_to_the_piece_it_started_on` conta quem consulta a lista
de peças. O indicador do contorno é `&mut self` (a cache vive no traço) e herdou
**sem uma linha de discussão** o molde que a pose pagou: a escolha da peça sai
para uma função `&self` (`boundary_gizmo_alvo`), e a prova continua a ser do
compilador. *Um corte que o segundo caso reutiliza é um desenho; um que ele
contorna era um remendo.* Censo `3 → 4`.

---

## §15 — Provas de mutação da 2.ª jornada (5 de 5 sangram)

| # | mutação | quem sangra |
|---|---|---|
| M7 | a curva volta ao contrário (o defeito do dono) | `a_borda_e_quem_mais_se_move_e_o_efeito_morre_para_dentro` |
| M8 | a fita volta a ligar pela ORDEM da cadeia | `nenhum_pedaco_atravessa_a_boca` **e** `o_indicador_desenha_a_boca_e_a_profundidade` |
| M9 | o pedaço de FECHO desaparece | `o_indicador_desenha_a_boca_e_a_profundidade` |
| M10 | o PESO deixa de entrar na tinta | `the_weight_of_each_piece_reaches_the_ink` |
| M11 | o bloco da fita é apagado do overlay | os **3** do `the_boundary_ribbon_is_painted_under_the_ring` |

⚠️ **M11 teve de ser reescrita:** a 1.ª redacção renomeava o método e **não
compilava** — uma mutação que não compila não prova nada, e lê-se como sangrar.

---

## §16 — ⭐⭐ A METADE DA MÁSCARA NO DYNAMIC TOPOLOGY está CURADA

> Item da fila que o dono pôs em 14/09: *«algumas tools que não deveriam fazer a
> subdivisão de polígonos no modo Dynamic Topology estão fazendo (como smooth)
> enquanto algumas que deveriam criar subdivisões não estão»*.

A tabela inteira é pergunta de **ORÁCULO** e continua aberta
([plano 22](../22_plano_quem_subdivide_no_dyntopo.md) §4). ⭐ **Uma célula dela
não precisa de alvo nenhum, e é a que esta jornada fecha.**

### A causa: a porta respondia à pergunta errada

O refino tem **um** chamador de produto — o braço do carimbo —, logo a pergunta
que o produto respondia era *«este gesto passou pelo caminho do carimbo?»* e não
*«este verbo cria superfície nova?»*. ⚠️ É a mesma família de defeito que este
módulo já pagou três vezes ao contrário: **inferir uma propriedade do VERBO a
partir do CAMINHO que o gesto tomou.**

### Medido, e o número é grosseiro

Com o dyntopo armado e o detalhe no extremo fino, **um** dab de máscara na cena
de teste levava a peça de **`830` para `1 331` vértices** — `+60 %` de topologia
num gesto que **não move um único vértice**. O artista que só queria proteger uma
zona pagava uma malha diferente.

### A cura, e o que ela deliberadamente NÃO faz

A porta passou a **receber o verbo** e a ler **duas colunas**
(`Verb::refina_no_dyntopo` · `Verb::colapsa_no_dyntopo`), com o `&&` a
curto-circuitar — um verbo que diga `false` nem chega a chamar o motor.

⛔ **E mais nada mudou, com gate a afirmá-lo**
(`o_mask_e_a_unica_correccao_que_esta_tabela_faz`): todos os outros verbos leem
exactamente o que já liam. ⇒ o §5 do plano já não é um esboço — a porta existe, e
o estudo preenche **células** em vez de re-fiar.

⚠️ **O `false` dos 8 verbos com âncora é o valor CONSERVADOR, não uma resposta:**
hoje eles nem chegam à porta, e assim ligá-la aos gestos ancorados antes do
estudo **não muda nada em silêncio**. Eles são precisamente os que mais
**esticam** superfície, logo os candidatos mais fortes a mudar de valor quando a
tabela for medida — e é por isso que a mudança tem de ser deliberada.

⚠️ **As duas colunas coincidem hoje em TODOS os verbos**, e isso é um facto sobre
o produto de hoje (as duas leis vivem numa porta só), **não** uma lei. O gate
`as_duas_colunas_ainda_coincidem_e_isso_nao_e_uma_lei` reprova no dia em que o
estudo as separar — que é quando se quer que ele fale.

⛔ **O veredito é lido como BOOLEANO e não por um estado novo nos enums da
`ph2d-mesh`:** `Collapse::Enough` significa *«nenhuma aresta está sob o limiar»*,
que é um facto sobre a MALHA — usá-lo para dizer *«o verbo não pediu»* poria duas
coisas diferentes no mesmo byte.

### As réguas, e as duas metades de cada uma

| gate | onde | o que afirma |
|---|---|---|
| `a_mascara_nao_muda_a_topologia_e_o_desenho_muda` | cena (GPU, `#[ignore]`) | a máscara não muda a contagem **e** o `Draw` no mesmo arranjo muda |
| `a_mascara_continua_a_pintar_o_canal` | cena (GPU) | *curar um defeito desligando o verbo é a forma mais barata de o esconder* |
| `o_mask_e_a_unica_correccao_que_esta_tabela_faz` | unidade | o delta desta tabela é **um** verbo |
| `nenhum_verbo_com_ancora_refina_ou_colapsa_hoje` | unidade | o valor conservador, com piso de população |
| `as_duas_colunas_ainda_coincidem_e_isso_nao_e_uma_lei` | unidade | reprova no dia do estudo |
| `the_dyntopo_door_asks_the_verb` | shell | a porta recebe o verbo e lê as **duas** colunas; o chamador passa o pincel **armado** |

⚠️ **O controlo positivo é o que torna a primeira uma medição:** sem o `Draw`, um
`assert_eq!` de contagem ficaria verde sobre um dyntopo **inerte** (detalhe
grosso, esfera já fina, raio errado). *Uma régua que não vê o fenómeno acontecer
não prova que ele não aconteceu.* E o detalhe corre no **extremo fino** de
propósito — um refino que só aparece ali lê-se como *«não refina»* num corpus
grosso, que é a armadilha que o plano nomeia para o estudo.

### Mutações (2 de 2 sangram, nas três camadas)

| # | mutação | quem sangra |
|---|---|---|
| M12 | o `Mask` volta a refinar | o censo de unidade **e** o gate de cena |
| M13 | a porta deixa de perguntar ao verbo | o gate de cena **e** o da shell |

---

## §17 — ⭐⭐⭐ O *Density* — o primeiro dos quatro pincéis desbloqueados, e o único verbo que NÃO MOVE UM VÉRTICE

Primeira entrada de [`SPEC_unblocked_brushes.md`](../cleanroom/SPEC_unblocked_brushes.md) §3,
a que a própria espec classifica como **T0 no motor, T2 na semântica** — o motor
(colapsar arestas curtas numa esfera) já vivia no repo, portado do SculptGL com
atribuição; o que vem da espec é *quem é este pincel*.

### ⭐⭐ Ele caiu exactamente na porta que a jornada anterior construiu

O `Density` **liga o colapso e não liga o partir** — que é, à letra, a segunda
coluna que o [§16](#§16) acabara de criar para curar a máscara. *Uma porta cujo
segundo consumidor chega no mesmo dia e não pede uma linha de mudança é um
desenho; uma que ele contorna era um remendo.*

⭐ E ele **matou a premissa de um gate que eu tinha acabado de escrever**: o
`as_duas_colunas_ainda_coincidem_e_isso_nao_e_uma_lei` dizia *«elas coincidem
hoje em todos os verbos, e isto reprova no dia em que alguém as separar»*. O dia
foi o seguinte, e não foi o estudo — foi este pincel. O gate reprovou, foi
reescrito com o nome novo (`as_duas_colunas_separam_se_em_exactamente_um_verbo`),
e **a morte da premissa ficou visível no diff**, que é para isso que ele existia.

### As duas leis, e porque são duas

| | |
|---|---|
| **ele liga o colapso** | onde o pincel passa, a malha afina |
| **ele NÃO liga o partir** | ele **nunca acrescenta** superfície |

⚠️ *Um gate que só verificasse a primeira passaria com um pincel que também
subdivide, que é **outro produto**.* Medido (um dab, raio `160 px`):

| arranjo | verbo | vértices |
|---|---|---|
| malha fina (`48×72`), alvo grosso | `Density` | **`3 386 → 3 352`** |
| malha grossa (`8×12`), alvo fino | `Density` | **`86 → 86`** |
| a MESMA, alvo fino | `Draw` | **`86 → 359`** (o controlo) |

### ⛔ Ele não tem lei por-vértice, e isso partiu SEIS censos — cada um com razão

`Verb::sem_lei_por_vertice()` é a porta nova, irmã do `resolve_a_propria_regiao`.
O corte: *aqueles têm lei própria **noutro sítio**; este não tem lei nenhuma
sobre posições.*

Ao nascer, o verbo reprovou **seis** gates, e as mensagens deles são a descrição
correcta dele: *«dab inerte»* · *«não tocou nada»* · *«o dab não fez nada em
canal nenhum»* · *«deu EXATAMENTE o mesmo resultado com e sem alpha»*. ⭐ **Cada
um é um piso de população a fazer o trabalho dele**, e a cura foi **por PORTA,
nunca por nome**:

- **quatro** já filtravam por `writes_through_applicator()` ⇒ bastou essa porta
  passar a ter **duas metades** (*há duas maneiras de não haver aplicador a
  julgar*);
- **um** varria `Verb::ALL` sem filtro ⇒ ganhou a mesma filtragem **mais um piso
  de população**, senão a filtragem esvazia o censo em silêncio;
- **um** era uma contagem de perfis `B` ⇒ o `Density` não tem perfil de
  referência, e por uma razão **mais forte** que a das irmãs: às outras faltam
  **números**, a esta falta **grandeza**. Não há força nem curva de queda a
  herdar de ninguém.

### ⛔⛔ Uma mutação SOBREVIVEU, e o que ela expôs foi uma segunda resposta

Apagar o desvio do `stroke_symmetry` deixava o gate *«não move um vértice»*
**verde**: o `stroke_target` tem um braço `Verb::Density => live` — a resposta
defensiva *«e se alguém chegar aqui mesmo assim?»*, a mesma que o tecido e a pose
têm. *Duas respostas à mesma pergunta, e a de baixo mascarava a de cima.*

⇒ a régua passou a ser a **JANELA DO TRAÇO**: o `dab_core` fotografa (`capture`)
todo vértice ao alcance **antes** de decidir o que fazer com ele, logo um
`touched` não-vazio prova que a cadeia de peso correu mesmo quando ela não move
nada. Com a mutação, o gate lê *«a densidade fotografou **35** vértices»*.

### ⛔ DECISÃO DE PRODUTO por decidir, e ela é observável

O nosso colapso recusa mexer numa aresta em que *algum dos quatro vértices está
na beira* — mais duro que o alvo, que em vez de recusar **escolhe o
sobrevivente** (espec §3.8, que declara isto decisão de produto com duas frases e
sem terceira saída). **Shipa a conservadora**, e o que o artista vê é *«o pincel
não afina a borda»*. ⇒ é também o que explica a colheita modesta da primeira
linha da tabela (`34` vértices).

### A tecla: ele NÃO entra na fila do `L`

A fila dos pretendentes continua em **oito**. Ele fica de fora por uma razão de
espécie diferente: as oito são **gestos de forma** que se alternam enquanto se
esculpe; este não esculpe, e só faz alguma coisa com o passe de topologia
**armado** ⇒ uma tecla nua seria **inerte na configuração de fábrica**. A
vizinhança dele no teclado é o `P` e o `U`, não a fileira de pincéis — registado
para a decisão não ter de o redescobrir.

### Mutações (2 de 2 sangram, depois de a régua ser corrigida)

| # | mutação | quem sangra |
|---|---|---|
| M14 | a densidade passa a refinar também | **2** censos de unidade **e** o gate de cena |
| M15 | ela volta a cair no `dab_core` | o gate de cena, pela metade nova (`touched`) |

### ⛔⛔ E TRÊS vermelhos só a varredura IMPACTADA os viu

Todos vivem em `tests/it/` de **outras** crates, que nenhum `cargo test --lib` da
crate editada alcança — a família que o `CLAUDE.md` §2 nomeia (*o `--bins` não
chega aos gates que vivem em `tests/`*).

1. ⭐ **O verbo novo não tinha CHIP.** O `SCULPT3D_VERB` é um array de ids
   **escrito à mão** e o gate `every_verb_has_a_chip_that_selects_it` compara-o
   com `Verb::ALL` — leu `28` contra `29`. *Sem ele o pincel existe, tem lei,
   tem gates e o artista não lhe chega.* (O irmão
   `every_painted_control_is_clickable_where_it_is_drawn` caiu por
   `index out of bounds` no mesmo array: **um defeito, dois relatos**.)
2. ⚠️ **O tecto de LOC do `brush_verb.rs`** — `719` contra `700`, estourado pelo
   doc do verbo novo. ⛔ **Curado por CORTE e nunca por uma entrada nova no
   `FILE_OVERAGE_OK`**: a lei do gesto (`grip` + `anchors`) saiu para o irmão
   `brush_verb_grip.rs`, e o corte é melhor do que o ficheiro era — ela passa a
   ler-se inteira num sítio, em vez de ser o rabo de uma tabela de nomes.
   `719 → 636`.

---

## §18 — ⛔⛔⛔ *«não vejo efeito com density»* — o pincel estava CERTO, a RÉGUA e o ROTEIRO é que não

Report do dono sobre o smoke do §17. Reproduzido dab a dab (sonda
`diag_o_percurso_do_dono`, o percurso dele na `=14`):

| passo | vértices |
|---|---|
| a cena abre | `128` |
| 8 dabs de `Draw` no detalhe **fino** | `822` |
| 10 dabs de `Density` no detalhe **grosso** | **`399`** (`−51 %`) |
| 10 dabs de `Density` no detalhe **médio** | `396 → 396` — **zero** |

⇒ **o pincel funciona.** O que falhou foram três coisas minhas.

### 1. ⛔ A minha régua afirmava a DIRECÇÃO e nunca a MAGNITUDE

O gate do §17 dizia `depois < antes`. Com ele **verde**, a colheita medida era de
`34` vértices em `3 386` — **`1 %`**, invisível a olho nu, e eu escrevi no
handoff *«a colheita é modesta»* como se fosse uma nota de rodapé.

⇒ o gate novo é uma **fracção**, com a barra tirada do percurso do próprio dono
(`−51 %` medido, barra em **`−25 %`**, margem de `2×`). *Uma régua que só vê o
SINAL não vê a MAGNITUDE* — a mesma família do `edge_max` cego ao quad fino e da
contagem cega a **quais** vértices se movem.

### 2. ⛔⛔ O meu roteiro levava-o a um estado em que o passe NÃO CORRE

O passo `(12)` mandava usar o `Density` *«depois de ter adensado no passo (3)»* —
mas entre os dois estão o passo **(9)**, que **desliga** o modo (`P`), e o
**(10)**, que monta uma pilha de multiresolução (`K`) sobre a qual o passe
**recusa** com a pilha montada. ⇒ *seguindo o roteiro em ordem, no passo (12) o
modo está desarmado ou a recusar*, e o artista vê exactamente o que o dono viu.

⚠️ **É a espécie de defeito que o `CLAUDE.md` §5.0 nomeia como pior que uma cena
ausente** — a ausente não é acreditada. O passo `(12)` passou a ser
**auto-contido**: ele reconstrói o estado que precisa (`J` · `P` · `U` fino ·
adensar · `Density` · `U` grosso) e **diz o `U` em maiúsculas**, porque sem ele o
pincel não tem o que fazer.

### 3. ⭐⭐⭐ E o pincel era MUDO sobre a própria inércia

Há **três** razões diferentes para o passe não correr — o modo desligado · a
pilha montada · não haver aresta fora da faixa — e o artista vê as três
**iguais**: nada acontece. *Foi assim que o report nasceu.*

⇒ `queixa_do_passe`: uma linha de log **por traço** (não por dab), **só** para os
verbos sem lei por-vértice. ⛔ Num `Draw` um passe que não parte nada é o caso
**normal** — ele esculpe à mesma —, e uma linha por dab seria um log que ninguém
lê. Gate na shell a contar as **três** chamadas: *a que faltar é um pincel que
parece partido e não diz porquê*.

### ⚠️⚠️ E um CONTROLO NEGATIVO meu era VÁCUO, provado por mutação

A primeira redacção do controlo *«o `Draw` fica calado»* usava uma esfera densa
com o detalhe grosso — onde o `Draw` **colapsa**, logo ele nunca chegava ao
caminho da queixa. Tornar a queixa **universal** (a mutação) **não o reprovava**.

⇒ *um controlo negativo tem de percorrer o MESMO caminho que a metade positiva*,
senão está a afirmar sobre código que não corre. Ele usa agora o caminho do modo
**desligado**, que é onde a queixa nasce.

### Mutações (3 de 3 sangram, depois de o controlo ser corrigido)

| # | mutação | quem sangra |
|---|---|---|
| M16 | a queixa cala-se | `quando_a_densidade_nao_faz_nada_ela_diz_porque` |
| M17 | a queixa passa a valer para TODO verbo | o mesmo, pelo **controlo negativo** |
| M18 | `MAX_PASSES = 0` no colapso (a colheita volta a um punhado) | `a_densidade_tira_uma_fraccao_visivel_e_nao_um_punhado` |

---

## §12 — Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=42 cargo run -p ph2d-host-desktop --profile smoke
```

E a **`=14`** para as duas metades da topologia dinâmica — o passo **(11)** é a
máscara (§16) e o **(12)** o `Density` (§17):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=14 cargo run -p ph2d-host-desktop --profile smoke
```

O roteiro dos **7** passos é impresso pelo próprio app ao abrir, e o passo
**(1)** é o INDICADOR: passar o rato **sem carregar** perto da boca. ⚠️ Rode também
**uma vez sem a env var** — é a metade que prova a inércia.

---

## §19 — ⭐⭐⭐ *«apenas no grosso vi alguma coisa acontecendo»* — o pincel TEM as duas direcções, e a espec dizia-o numa tabela que eu li pela metade

**Report do dono (2026-09-14, o SEGUNDO sobre este pincel):**

> *«apenas no grosso vi alguma coisa acontecendo. Porque não temos um slider neste pincel para
> definir a densidade da malha. Por que não pode aumentar a densidade também?»*

São **três observações e as três estavam certas**, cada uma sobre uma coisa diferente. A primeira é a
**consequência** da terceira; a segunda é um buraco de superfície que existia desde que o modo nasceu.

### §19.1 — ⛔⛔ O erro foi MEU e estava numa LEITURA, não no código: a tabela-verdade da espec

O [`Verb::refina_no_dyntopo`] afirmava, **com o comentário ao lado**, que

> *«a densidade NUNCA acrescenta superfície, e é lei e não omissão: ela liga o colapso e **não** liga
> o partir. Um pincel que também subdividisse é outro produto.»*

A espec `SPEC_unblocked_brushes.md` §3.2 diz outra coisa, e di-lo numa tabela-verdade de três linhas:
o pincel **ACRESCENTA a bandeira de colapso** ao modo do passe — ele não **RETIRA** a de partir, que
continua a ser do ajuste de *método de refino*:

| *Detailing* | o ajuste de refino pede… | **colapsar** corre? | **partir** corre? |
|---|---|---|---|
| Manual | (qualquer) | não | não |
| ≠ Manual | partir | **SIM** (é este pincel a pedi-lo) | sim |
| ≠ Manual | colapsar | sim | não |
| ≠ Manual | partir + colapsar | sim | sim |

E ela **mede as duas células na mesma malha grossa**: `81 → 81` com o ajuste em «só colapsar», e
**`81 → 101`** com «partir + colapsar» — *duas leituras do mesmo pincel, com o ajuste diferente*.

⚠️ **A recusa medida da espec continua de pé, e é OUTRA pergunta.** A linha *«fazer o `Density`
também subdividir»* da tabela de recusas é sobre o pincel **FORÇAR** o partir, como ele força o
colapso. Ele não força — ele **obedece**. A célula (b) do gate novo é exactamente esse controlo.

⭐⭐⭐ **A leitura que fica:** *um gate pode pinar a leitura errada de uma espec tão bem como pina um
defeito, e o que o separa de uma medição é ninguém ter corrido a outra célula.* O gate anterior
chamava-se `a_densidade_afina_a_malha_e_nunca_a_engrossa` e tinha um controlo positivo a sério
(o `Draw` no mesmo arranjo) — ele só nunca correu o mesmo pincel com o outro ajuste, porque **o outro
ajuste não existia**.

### §19.2 — ⭐⭐ O ajuste vive no PINCEL, e não na cena — espec §9.8

O alvo guarda-o na cena. A §9.8 da espec regista que **existe um pedido público aberto** para o tirar
de lá e o pôr no pincel, e escreve que *«a nossa casa já está do lado certo dessa mudança»*.
⛔ **Não copiámos o modelo que a referência está a caminho de abandonar:**
[`ph2d_sculpt3d::DensityModo`] é um campo do `Brush`, com dois valores:

| | o que faz | corresponde a |
|---|---|---|
| **`Equalise`** (omissão) | leva a malha **ao** alvo nos dois sentidos | *partir + colapsar* |
| `Thin Only` | só colapsa, nunca acrescenta | *só colapsar* |

⭐ **A omissão é `Equalise` por DUAS razões que apontam ao mesmo lado:** é o ajuste que a referência
ship, e é a que responde ao report — com o pincel só a afinar, **ele só tem o que fazer quando o alvo
pedido está mais grosso que a malha**, e em toda a outra metade do curso ele é indistinguível de uma
ferramenta partida. *É literalmente a frase do dono: «apenas no grosso vi alguma coisa acontecendo».*

### §19.3 — ⭐⭐ O slider: a superfície que faltava, e a nota que a barrava tinha a premissa expirada

O alvo de densidade existia desde que o modo nasceu e era alcançável **só pela tecla `U`**, em **três
degraus** (*grosso · médio · fino*), com o doc da tabela a justificar-se assim:

> *«três e não um slider contínuo, porque a UI aqui é o teclado (a aba Topologia é wave de UI)»*

A premissa **expirou** quando a secção *Topology* do painel ganhou os knobs do remesh — e ninguém
releu a nota. Hoje é uma **pista contínua** (`panel.sculpt3d.dyn_detail`, `Place::AfterDyntopo`,
colada ao interruptor que a arma), e a tecla `U` fica como **atalho** a ciclar os três valores com
nome, exactamente a relação que o `[`/`]` tem com a pista do raio.

⛔ **Os dois NÃO coexistem como superfícies:** os três chips foram **apagados** (`SCULPT3D_DETAIL`
morreu, `SCULPT3D_DYN_DETAIL` nasceu). Eles escreviam o mesmo número, e duas superfícies sobre um
valor só divergem no dia em que uma ganhar clamp e a outra não.

⚠️ **A faixa não é escolhida — é o domínio da lei:** o `ph2d_mesh::edge_target` faz
`detail.clamp(0.0, 1.0)` e devolve `raio × √((1,1 − d) × 0,2)`, logo `0` pede `0,469 × raio` e `1`
pede `0,141 × raio`. Uma faixa de **3,3×**, e um tecto mais apertado seria um limite sem recurso.

### §19.4 — ⭐ O que a sonda mede, no percurso do próprio dono (cena `=14`)

`diag_o_percurso_do_dono`, malha da cena (`128` vértices), dab a dab:

| gesto | ajuste | pista | vértices |
|---|---|---|---|
| 8 dabs de `Draw` | — | fino | `128 → 822` |
| 10 dabs de `Density` | `Equalise` | grosso | `822 → **400**` (−51 %) |
| 10 dabs de `Density` | `Equalise` | médio | `400 → 419 → **409**` — ⭐ ele **ACRESCENTOU** e assentou |
| 10 dabs de `Density` | `Thin Only` | grosso | `409 → 386` |
| 10 dabs de `Density` | `Thin Only` | fino | `386 → 386` — **zero**, e é a lei daquele ajuste |

⭐⭐ **E o gesto NOVO, a partir da malha CRUA e sem `Draw` nenhum** — `Equalise` na pista no máximo:
`128 → 239 → **719**` em dois dabs, a assentar em `682`. *Adensar uma zona sem lhe mexer na forma
passou a ser um gesto que existe.*

⚠️ **A linha do médio é o report inteiro numa célula:** antes desta wave ela lia `396 → 396` — zero —,
e é exactamente o que o dono viu.

### §19.5 — Os gates, e o que cada um impede

| gate | onde | o que morre sem ele |
|---|---|---|
| `a_densidade_colapsa_sempre_e_parte_conforme_o_ajuste` | `ph2d-sculpt3d` | a tabela-verdade da espec, célula a célula — incluindo a recusa (`Afinar` separa as duas colunas) |
| `o_ajuste_de_densidade_so_alcanca_a_densidade` | `ph2d-sculpt3d` | o ajuste novo alcançar verbo a mais (varredura, não leitura) |
| `so_a_mascara_se_afasta_do_comportamento_de_hoje` | `ph2d-sculpt3d` | ⚠️ **a densidade SAIU desta lista** — no ajuste de omissão ela faz o que os outros 27 fazem |
| `a_densidade_obedece_a_tabela_verdade_do_passe` | `ph2d-app-sculpt3d` | as quatro células medidas na malha, com o `Draw` como controlo positivo |
| ⭐ `a_pista_do_detalhe_chega_ao_motor` | `ph2d-app-sculpt3d` | **o ponto cego do §5.0**: um slider pintado, registado e vivo cujo número nunca sai do painel — as três metades são ida · volta · **duas posições dão malhas diferentes** |
| `every_density_control_is_clickable_where_it_is_drawn` | `ph2d-panel-sculpt3d` | a fileira nascer morta sob o ponteiro (a fixtura arma `Density`, porque a genérica arma `Crease`) |
| `a_fileira_da_densidade_e_ausente_com_outro_pincel` | `ph2d-panel-sculpt3d` | a metade oposta — dois chips a aparecer em 28 ferramentas que não os leem |
| `cada_chip_de_densidade_arma_o_seu_modo` | `ph2d-panel-sculpt3d` | o **dreno de um braço só**: um `ALL[0]` cravado deixa o 2.º chip pintado e a mentir |
| `the_panel_offers_every_density_mode_the_brush_has` | `ph2d-panel-sculpt3d` | um modo novo nascer inalcançável |
| `the_dyntopo_door_asks_the_verb` | `shells/desktop` | a porta deixar de receber o ajuste, ou o chamador passar um literal |

**Provas de mutação: 6 de 6 sangram.**

1. `refina_no_dyntopo` ignora o ajuste → **3** gates (2 na crate + a tabela-verdade no app).
2. `parte_arestas_longas` sempre verdade → `o_ajuste_de_densidade_so_alcanca_a_densidade`.
3. `offers_density_controls` → `false` → `every_density_control_is_clickable_where_it_is_drawn`.
4. o 2.º chip arma `ALL[0]` → `cada_chip_de_densidade_arma_o_seu_modo`.
5. `apply_ui` crava `0.5` → `a_pista_do_detalhe_chega_ao_motor` (metade 2).
6. o passe crava `edge_target(radius, 0.5)` → `a_pista_do_detalhe_chega_ao_motor` (metade 3).

⭐ **As duas últimas são o par que justifica a terceira metade daquele gate:** a mutação 6 deixa as
metades 1 e 2 **verdes** — o campo é escrito e publicado, e o motor ignora-o.

### §19.6 — ⛔ Um teto de LOC ficou vermelho, e foi curado por CORTE

`ph2d-panel-sculpt3d/src/paint/brush.rs` chegou a **`623`** contra o teto de `600` ao ganhar a
fileira nova. ⛔ **Nada foi para o `FILE_OVERAGE_OK`:** as quatro fileiras **próprias de cada pincel**
(contorno · densidade · pose · tecido) saíram para o irmão `paint/brush_fileiras.rs`
(`623 → 367` + `283`), e o corte é de **responsabilidade** — lá fica *a moldura do pincel*, aqui *o
vocabulário próprio de cada ferramenta*, que é a lista que cresce um bloco por pincel novo.
⚠️ Nenhum pixel muda de sítio: as quatro são chamadas na mesma ordem, do mesmo sítio, com os mesmos
argumentos.

### §19.7 — ⚠️ Duas coisas que uma leitura rápida do diff entende ao contrário

1. **O `Density` NÃO passou a subdividir «como o `Draw`».** Ele continua sem lei por-vértice: não
   move um vértice, não acumula, não tem curva. O que ele faz é **pedir ao passe de topologia** que
   leve a malha ao alvo — e a diferença com o `Draw` é que aquele *também* deposita barro.
2. **A queixa do passe não desapareceu — ela ENCOLHEU, e o texto dela mudou com isso.** Ela dizia
   *«não há aresta fora da faixa aqui — baixe o detalhe com U»*, que é conselho de um pincel que só
   afina. Hoje diz *«a malha aqui já está no ponto que o `Detail` pede»*, porque com o ajuste de
   omissão chegar ali quer mesmo dizer isso. As **três** razões continuam a ser três, com gate a
   contá-las.

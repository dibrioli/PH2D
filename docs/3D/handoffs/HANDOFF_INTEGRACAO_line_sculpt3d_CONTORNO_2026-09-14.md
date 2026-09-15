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

> ⚠️⚠️ **LEIA A §20 ANTES DE ACREDITAR NO QUE VEM A SEGUIR.** Duas coisas desta secção foram
> **retiradas pelo dono no mesmo dia**: o ajuste `DensityModo` (*«não precisamos do modo Thin Only»*)
> e a âncora do slider no raio do pincel (*«a densidade da malha deve ser independente do zoom»*). O
> que fica de pé aqui é a **leitura errada da espec** que o report expôs, e a tabela medida.

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


---

## §20 — ⭐⭐⭐ TRÊS ORDENS DO DONO NO MESMO DIA, e as duas últimas mudaram a ÂNCORA de duas coisas

**Reports (2026-09-14, a seguir ao da §19):**

> *«Não precisamos do modo Thin Only. Deve ser sempre Equalise. Independente se Dynamic topology
> está ligado ou não, Density faz o seu trabalho. Dynamic topology é para os outros pincéis.»*
>
> *«A densidade da malha deve ser independente do zoom.»*

### §20.1 — O `Thin Only` SAIU, e com ele a coluna volta a ser função só do verbo

O `DensityModo` viveu **doze horas**, entre dois reports do mesmo dia. Ele nasceu certo — o primeiro
report expôs que eu lera a tabela-verdade da espec §3.2 pela metade — e morreu por **veredito de
produto**: a metade que só afina não é produto.

⚠️ **O que fica registado é o CICLO da premissa.** O gate
`as_duas_colunas_coincidem_hoje_e_isso_nao_e_uma_lei` existia a dizer exactamente isso, **reprovou**
quando o ajuste as separou, foi reescrito com a morte da premissa no diff — e **voltou** quando o
ajuste saiu. *Uma premissa que morre e renasce em doze horas é a melhor prova de que ela tinha de
estar num gate e não num comentário.*

⛔ **A recusa medida da espec continua de pé e é OUTRA pergunta:** *«fazer o `Density` também
subdividir»* é sobre o pincel **FORÇAR** o partir onde o ajuste da cena o desliga. Nós não temos
esse ajuste — nem o queremos, pela ordem acima —, logo não há nada que ele possa forçar.

### §20.2 — ⭐⭐ Ele corre SEM o interruptor, e isso tinha uma peça escondida

`Verb::corre_sem_o_interruptor()`, **derivado** de `sem_lei_por_vertice()` — quem não tem lei
por-vértice não tem nada a ganhar com um interruptor que responde *«o meu traço também muda a
topologia?»*.

⚠️ **DIVERGÊNCIA DECLARADA da referência:** a espec §3.2 tem o desarme na **primeira linha** da
tabela-verdade (o modo de detalhe em *Manual* desliga o passe inteiro, este pincel incluído). O
argumento do dono vence: aquele interruptor é sobre traços, e este verbo não tem traço.

⛔⛔ **E a peça que quase ficou por fazer: a TRIANGULAÇÃO.** Os dois motores recusam quads por
geometria (`Refine::NotTriangles` — partir a aresta de um quad devolve um triângulo e um pentágono),
e quem os triangulava era o `toggle_dyntopo`. Sem herdar esse trabalho, o pincel seria um **no-op
silencioso** em toda peça que ainda é de quads — uma primitiva acabada de nascer, ou a saída do
botão de retopologia. ⇒ ela corre no **pen-down**, **depois da foto do desfazer**, para o gesto
inteiro (triangular *mais* adensar) desfazer num passo só.

⛔ **E o registo do desfazer tinha de mudar com isso:** ele filtrava por `vert_count`, e **triangular
um quad parte uma face em duas sem criar um vértice** — um traço que só triturou lia contagem igual e
saía **sem entrada nenhuma**. *Duas grandezas estavam a ser lidas como uma.*

⚠️ **As razões do silêncio passaram de TRÊS para DUAS**, e a que saiu não foi apagada: ficou
**inalcançável**. A queixa do modo desligado só falava por quem não tem lei por-vértice — ou seja,
exactamente por quem já não passa naquele ramo. *Curar um defeito pode APAGAR uma queixa, e um censo
que continue a contar três fica a mentir para o lado seguro.*

### §20.3 — ⭐⭐⭐ A densidade deixou de depender do ZOOM, e o defeito estava medido

O alvo de aresta saía do `Brush::radius`, que é **derivado do raio em PIXELS através da câmera** a
cada dab (`armed_brush_on`). Medido na cena `=14`, mesma peça, mesmo pincel (`160 px`), mesmo slider
(`0,5`):

| distância da câmera | raio em mundo | alvo VELHO | alvo NOVO | mediana na esfera |
|---|---|---|---|---|
| 1,5 | 0,1198 | **0,0415** | `0,0804` | `0,0601` |
| 3,0 | 0,2752 | 0,0953 | `0,0804` | `0,0539` |
| 6,0 | 0,5858 | **0,2029** | `0,0804` | `0,0543` |

⇒ **`4,9×` de alvo só por aproximar ou afastar**, e a cura leva a dispersão do alvo a **zero** e a da
densidade alcançada a **`±11 %`** (o resíduo é a discretização: uma aresta parte-se ao meio ou não se
parte).

⭐⭐ **A cura não é uma invenção — é a mesma que esta casa já tinha escrito para o mesmo defeito.**
Quando o `Quad Size` absoluto do botão de retopologia foi refutado com foto pelo dono, a lição ficou
no código: *«um mesmo `0,02` é destrutivo numa malha grossa e conservador numa fina: o número não era
da malha»*, e a cura foi **ancorar na ÁREA**. O alvo da topologia dinâmica tinha a mesma doença uma
âncora acima, e leva a mesma cura:

```
tris   = MIN_TRIS · (MAX_TRIS/MIN_TRIS)^detalhe        (geométrico)
aresta = √( 4·área / (√3 · tris) )                     (triângulo equilátero)
```

⚠️ **Os dois extremos têm o RECURSO NOMEADO, e nenhum foi escolhido:**

* **`MIN_TRIS = 200`** é o `MIN_QUADS` do botão **em triângulos** — aquele saiu de uma varredura de
  volume medida (`96` faces guardam 91 % · `40` guardam 68 % · `23` guardam 50 % ⇒ joelho em ~`100`),
  e a pergunta é a mesma nos dois sítios. *Escrever um segundo número seria duas medições da mesma
  grandeza.*
* **`MAX_TRIS = 100 000`** é o **relógio do dab**: cada passe termina num `rebuild` inteiro, e a
  tabela do módulo mede-o — `97 922` vértices custam `5,50 ms` contra um orçamento de `8`. Numa malha
  fechada os triângulos são ~`2×` os vértices, logo `100 000` triângulos são ~`50 000` vértices,
  `~2,8 ms`, **35 %** do orçamento. *É o ponto mais fino ainda confortável, não aquele em que o
  relógio ainda se aguenta.*

⭐ **E a área virou uma PORTA** (`ph2d_mesh::Mesh::surface_area`): ela vivia na `ph2d-quadflow`, e o
segundo consumidor chegou de outra crate. *Duas somas de triângulos em duas crates seriam a segunda
resposta à mesma pergunta* — hoje a quadflow delega.

⚠️⚠️ **E isto muda o que o PINCEL significa, de propósito:** o doc antigo dizia *«um pincel pequeno
detalha fino e um grande detalha grosso, que é o que a mão espera»*. Isso é a *fração do pincel* da
referência (espec §3.4); o que fica é a *detalhe CONSTANTE* dela, que é a que não muda com a vista.
⇒ **o pincel diz ONDE, o slider diz QUÃO FINO.** Enquanto as duas perguntas partilhavam um número,
mexer numa mexia na outra.

### §20.4 — Os gates, e as provas de mutação (6 de 6 sangram)

| gate | o que morre sem ele |
|---|---|
| `a_densidade_leva_a_malha_ao_alvo_nos_dois_sentidos` | as duas direcções, medidas na malha |
| `so_quem_nao_tem_lei_por_vertice_corre_sem_o_interruptor` | a porta nova alcançar verbo a mais (varredura + piso de população) |
| `a_densidade_corre_com_o_interruptor_desligado` | **quatro** metades: ela trabalha · tritura os quads · o controlo (o `Draw` **não** trabalha) · o `Ctrl+Z` devolve tudo |
| ⭐ `a_densidade_nao_depende_do_zoom` | o report inteiro: alvo idêntico (igualdade) + densidade alcançada na banda medida |
| ⭐ `a_densidade_nao_depende_do_tamanho_da_peca` | a outra invariância que ancorar na área compra — e **só ela** apanha a área cravada numa constante |
| `a_pista_do_detalhe_chega_ao_motor` | o ponto cego do §5.0 — as três metades, e só a terceira sangra com o alvo cravado |
| `as_duas_colunas_coincidem_hoje_e_isso_nao_e_uma_lei` | o ciclo da premissa ficar invisível no diff |
| `the_edge_target_comes_from_the_piece_never_from_the_brush` | a âncora voltar ao raio, na shell |

**Mutações:** (1) o interruptor volta a prender toda a gente · (2) ninguém tritura a peça · (3) o
desfazer volta a perguntar só pelos vértices · (4) o alvo volta a sair do raio · (5) a área vira
constante · (6) a densidade deixa de partir. **Todas sangram.**

⚠️⚠️ **E DUAS delas exigiram correcção do MÉTODO, as duas já registadas neste repo:**

* **A (3) SOBREVIVEU à primeira tentativa** porque o meu gate media o desfazer sobre um dab que
  **também** mudava a contagem de vértices — reverter a metade `face_count` deixava-o verde. *Um
  gate que não contém o caso não afirma nada sobre ele.* O arranjo que o contém saiu de uma medição
  desta linha: com a câmera perto, o pincel de `160 px` vale `0,12` de mundo contra arestas de
  `~0,3` — **menor que um triângulo** —, logo o dab não parte nem funde e a única coisa que acontece
  é a triangulação. ⚠️ A minha primeira tentativa foi escolher um ponto do slider por aritmética
  sobre a aresta *média*, e mediu `+21` vértices: **numa esfera UV as arestas encolhem para zero nos
  pólos**, logo não existe posição do slider em que nada parta E nada funda. *O caso não era um
  número; era a geometria do alcance.*
* **A (4) NÃO CHEGOU A APLICAR-SE** na primeira corrida — o `cargo fmt` reformatara a linha e o
  filtro casou `0×` — e o teste imprimiu `ok`. *Uma mutação que não entra lê-se exactamente como uma
  que sobreviveu*; é a terceira vez que este repo a paga, e a cura é o `assert` de contagem no
  próprio filtro, que aqui apanhou.

### §20.5 — ⛔ O ARNÊS estava a medir outro programa

`um_dab` fazia `aim` + `stroke.begin` + `sculpt_at` + `close_stroke` — e **não** chamava o
`open_dyntopo_stroke`, que é o que o pen-down do produto faz. Consequência: a foto da malha de antes
**nunca era tirada**, logo o `close_stroke` nunca gravava a entrada `Remeshed` e **nenhum gate desta
linha cobria o desfazer de um traço que muda a topologia**. *Um arnês a que falta um passo do produto
mede outro programa* — e o que ele media aqui era um pincel sem desfazer.

### §20.6 — ⛔ Três tectos de LOC, curados por CORTE

| ficheiro | antes | depois |
|---|---|---|
| `ph2d-panel-sculpt3d/src/paint/brush.rs` | `623` | `367` + `brush_fileiras.rs` |
| `ph2d-mesh/src/dyntopo.rs` | `825` | `690` + `dyntopo_alvo.rs` |
| `ph2d-app-sculpt3d/src/dyntopo_tests.rs` | `827` | `180` + `densidade_tests.rs` + `densidade_ancora_tests.rs` |

Os três cortes são de **responsabilidade** e nenhum entrou no `FILE_OVERAGE_OK`: *as fileiras
próprias de cada pincel* contra *a moldura do pincel*; *quanto deve medir uma aresta* contra *como se
parte uma malha*; *o que o pincel faz* contra *de onde vem o número que ele persegue*.

### §20.7 — ⚠️ Três coisas que uma leitura rápida do diff entende ao contrário

1. **O `edge_target` da `ph2d-mesh` NÃO morreu** — ele é a lei da referência e é o que as bancadas de
   paridade medem. O que mudou é que o **produto** já não o chama.
2. **A contagem final continua a variar com o zoom, e isso é o CERTO.** Um pincel de `160 px` cobre
   menos peça quando a câmera se aproxima — isso é o pincel a dizer ONDE. O que não pode variar é a
   densidade *onde ele tocou*, e é isso que o gate mede. *Uma régua global mediria as duas perguntas
   somadas e não saberia qual delas se mexeu.*
3. **Zoom muito perto pode deixar o pincel menor que um triângulo**, e aí não há o que pegar — o log
   diz-o, e a cura do artista é o `]`. Com a lei antiga isto não acontecia porque o alvo encolhia
   junto; é o preço declarado de a densidade deixar de depender da vista.

---

## §21 — ⭐⭐ DOIS SLIDERS `Detail`, e é de propósito — a consequência directa da §20.2

**Ordem do dono (2026-09-14, a quarta sobre este pincel):**

> *«Deixe o slider Detail para o dynamic Retopology e coloque outro slider Detail exclusivo para o
> pincel, nas propriedades do pincel.»*

⭐ **Ela fecha o raciocínio que a ordem anterior começou.** Se *«Dynamic topology é para os outros
pincéis»*, então o ajuste dela também é — e o pincel de densidade tem de trazer o seu. Eram **duas
perguntas a partilhar um número**:

| pergunta | quem responde | onde vive |
|---|---|---|
| *quão fina a malha fica debaixo de um **TRAÇO*** | o ajuste da cena (`Dyntopo::detail`) | secção **Topology** |
| *quão fina eu quero **esta zona, agora*** | o pincel (`Brush::density_detail`) | **propriedades do pincel** |

⚠️ **Os dois têm o mesmo rótulo e a mesma unidade, e isso é deliberado:** a régua é a mesma — uma
contagem de triângulos ancorada na ÁREA (§20.3), logo independente do zoom e do tamanho da peça.
*Dois sliders, uma régua.*

⛔ **E eles NÃO são duas superfícies sobre um valor** — a armadilha que os três chips de detalhe
pagaram na §19.3 desta mesma jornada. São **dois campos**, um na cena e outro no `Brush`, e quem
escolhe entre eles é **uma porta**.

### §21.1 — A porta, e os seus dois consumidores

`Brush::offers_density_controls()` (⭐ ela **voltou**: existiu para a fileira `Thin Only` e saiu com
ela na §20.1) responde *«este pincel traz o próprio alvo?»*, e tem dois consumidores em crates
diferentes:

* **o painel**, para decidir se pinta a pista;
* **o passe**, através da `Sculpt3dScene::detalhe_do_gesto`, para decidir **qual dos dois números
  ler**.

⚠️ *Duas cópias divergiriam num slider visível a governar outra coisa* — que é a forma mais cara de
um controlo mentir, porque ele **funciona**, só que noutro sítio.

⚠️ **E a porta tem um TERCEIRO consumidor que quase ficou de fora: a tecla `U`.** Ela cicla os três
degraus com nome, e agora cicla **o slider do gesto em mãos** — com o log a **nomear qual deles**
(`detalhe do Density:` contra `detalhe do Dynamic Topology:`).

### §21.2 — ⛔⛔ A mutação que SOBREVIVEU, e o que ela nomeia

Cravar o `cycle_detail` a escrever sempre em `self.dyntopo.detail` **passava a suíte inteira**.

⚠️ *Um atalho que escreve no controlo errado é indistinguível de um atalho morto, e nenhum gate de
fiação o vê: ele está ligado.* O artista carregaria no `U`, veria um slider mexer-se na tela, e o
gesto dele não mudaria de densidade nenhuma.

⇒ `a_tecla_do_detalhe_cicla_o_slider_do_gesto_em_maos`, com **as duas metades**: a que escreve **e**
a que **não toca no vizinho**. ⛔ Sem a segunda, um `cycle_detail` que escrevesse nos **dois** ficaria
verde — e mexer no atalho com um pincel na mão estragaria o ajuste do outro.

### §21.3 — Os gates e as mutações (3 de 3 sangram)

| gate | o que morre sem ele |
|---|---|
| `cada_gesto_le_o_seu_proprio_slider` | a troca — e a régua põe os dois em valores **OPOSTOS**, porque *dois números iguais não distinguem duas leis*; varre os **dois sentidos** |
| `a_tecla_do_detalhe_cicla_o_slider_do_gesto_em_maos` | o atalho mexer no slider errado, ou nos dois |
| `a_pista_do_detalhe_chega_ao_motor` | ganhou a segunda pista: ida e volta dos **dois** campos |
| `the_edge_target_comes_from_the_piece_never_from_the_brush` | o passe ler um dos dois **directamente** em vez de perguntar à porta (a régua é o CORPO da função, não o cluster — o ficheiro tem leituras legítimas: a tecla escreve, o retrato publica) |

**Mutações:** (1) a densidade volta a ler o slider da cena · (2) o painel deixa de oferecer a pista
do pincel · (3) a tecla `U` escreve sempre no da cena. As três sangram — a terceira **só depois** do
gate que ela encomendou.

### §21.4 — ⚠️ Duas coisas que uma leitura rápida do diff entende ao contrário

1. **Dois controlos com o mesmo rótulo não é um descuido** — é o mesmo nome para a mesma grandeza em
   dois assuntos, como *Radius* existe em mais de uma ferramenta. O que os separa é a **secção** em
   que vivem, e o log da tecla `U` nomeia-os quando fala.
2. **O arnês dos testes escreve nos DOIS**, e isso não enfraquece os gates: é o que mantém todos os
   outros a medir a densidade que pedem. *Quem prova que cada verbo lê o seu é o gate dedicado, que
   os põe em valores diferentes.*

### §21.5 — ⏳ ABERTO e nomeado

Com o `Density` na mão o painel ainda oferece **`Strength`** (e a curva), que este verbo **não lê** —
ele não tem lei por-vértice. É um **controlo morto** da espécie que o §5.0 do roteador descreve, e a
porta para o curar já existe (`Brush::offers_density_controls`). ⚠️ Fica **por decidir do dono** se
ele prefere a row escondida ou apagada em cinzento: *esconder é divulgação progressiva, e sumir sem
rasto já foi recusado uma vez nesta casa* (a curva do falloff, que voltou depois de um smoke).

---

## §22 — ⭐⭐⭐ O **APAGADOR DE DESLOCAMENTO** (`SPEC_unblocked_brushes.md` §4), e o substrato que ele exigiu

Seguindo a fila («Siga»): dos quatro pincéis desbloqueados, o `Density` shipou em 14/09 e os **dois
de multirresolução** tinham **uma** peça em falta nomeada — a §2.3. Esta secção fecha a peça **e** o
primeiro dos dois.

### §22.1 — ⭐⭐ O AVALIADOR DE PONTO-LIMITE, e ele REFUTOU a espec

`ph2d_mesh::limit_point` — onde um vértice pousa se a malha for subdividida **para sempre**, em forma
fechada, `O(anel)`, sem iterar.

⛔⛔ **Porque ele é a espinha (§2.1):** `subdivide^k(base)` **não é** a superfície-limite, e a
diferença não tende a zero na densidade que um artista usa. Medido no canto de `cube(1,0)`:

| superfície | coordenada do canto |
|---|---|
| a malha de partida | `0,5000` |
| **um passo** de subdivisão (a previsão) | `0,2778` |
| ⭐ o **LIMITE** | **`0,2500`** |

⇒ a previsão fica **`11 %` acima**, e um apagador ancorado nela deixaria esse resíduo **a cada
passagem**: ele *encolheria a peça*, e o artista leria isso como *«o apagador comeu a forma»*.

⛔⛔ **E A ESPEC §2.3 ESTÁ REFUTADA NUM PONTO, POR MEDIÇÃO.** Ela dá a máscara de Catmull-Clark como
`(n²V + 4·ΣE + ΣF)/(n(n+5))` com *«ΣE a soma dos pontos MÉDIOS das arestas do anel e ΣF a soma dos
CENTROIDES das faces incidentes»*. Escrita assim, o canto do cubo calcula `0,375` e o nosso
`subdivide` iterado sete vezes pousa em `0,250` — **estável** em `k = 4` e `k = 7`. A forma certa é a
do **ANEL**: `(n²V + 4·Σanel + Σdiagonais)/(n(n+5))`, que para `n = 4` é o estêncil clássico
`(16, 4, 1)/36`.

⚠️ *Uma espec atestada afirma o que o revisor podia ver, e ninguém tinha corrido a fórmula contra um
esquema.* É a **segunda** vez nesta linha (a primeira foi o `n̂` da §9.1 do contorno).

⭐ **A lei é DERIVADA e CONFERIDA, nunca citada** — a espec proíbe copiar uma tabela de pesos por
valência, e o gate `o_limite_e_onde_a_subdivisao_de_facto_pousa` mede as máscaras contra o nosso
**próprio** `subdivide` iterado. Isso é possível por uma propriedade do nosso porte: ele mantém os
vértices originais nos índices `0..V`. Medido (`k = 4` → `k = 7`):

| peça | esquema | `k = 4` | `k = 7` |
|---|---|---|---|
| `cube` | CC, valência 3 | `1,29e-4` | **`5,66e-7`** |
| `octahedron` | Loop, valência 4 | `1,95e-3` | **`3,05e-5`** |
| `uv_sphere 12×16` | CC + Loop | `1,41e-4` | **`2,21e-6`** |

⚠️ **A barra é a CONVERGÊNCIA e não um epsilon escolhido:** uma máscara errada **estabiliza** numa
distância ≠ 0 em vez de encolher — foi exactamente assim que a leitura da espec caiu.

⛔ **O anel MISTO recusa** (`LimitPoint::None`): o nosso `even` interpola dois esquemas ali e essa
mistura não tem limite publicado. *Inventar um seria pôr o vértice numa superfície que não é a de
esquema nenhum* — o defeito do alvo que a §2.4 manda **não** herdar.

### §22.2 — O pincel, e as duas coisas que o definem

**A lei (§4.1):** `p ← p + f·(R − p)` — uma interpolação **linear pura** em direcção à referência,
sem direcção privilegiada, sem normal, sem acumulador.

⭐ **A prova de que não é *«mover ao longo da normal»*** é a componente perpendicular a `R − p`,
medida no oráculo em `3,48e-08`. ⚠️ **E a fixtura tem de discriminar:** a primeira que escrevi usava
uma referência **radial**, onde `R − p` e a normal apontam para o mesmo lado e as duas hipóteses dão
o mesmo resultado. *Uma fixtura em que duas hipóteses coincidem não escolhe entre elas.* A que fica
desloca a malha por um vector **constante**.

**A força ao quadrado (§1.1),** medida: força `1,0` → fracção `1,000000`; força `0,5` → **`0,250000`**.

⛔⛔ **E o padrão do `Verb::Thumb` NÃO TRANSFERE, apesar de parecer o mesmo.** Lá o quadrado sai de
graça da composição (o alvo leva um peso, o aplicador multiplica pelo `accum`, que leva o outro) —
mas aquele verbo é `Grip::Hold`, e a tabela do `GripLaw` dá-lhe `unit_accum = false`; o `Grip::Stamp`
— o nosso — dá **`true`**: *o alvo já traz o peso e o `accum` vale 1*. Escrito à maneira do polegar,
este pincel media `0,500` a meio curso, que é literalmente o *«parecer o dobro de forte»* de que a
§1.1 avisa. ⇒ o quadrado é **escrito no alvo** (`w × strength`), e `w × strength` e não `w × w`:
`w` já é `peso × força × pressão`, logo o produto dá **força ao quadrado e pressão linear**, que é a
fórmula da espec letra por letra.

**A referência (§2), e a equivalência que a torna barata:** subdividir **não muda** a superfície
limite, logo o limite do vértice `v` de `subdivide(nível_de_baixo)` é um ponto da superfície-limite
do nível de baixo, **na posição paramétrica de `v`**. ⇒ a máscara de vértice chega; não é preciso
avaliar o limite em coordenadas arbitrárias.

**A recusa (§4.3):** sem pilha, o dado de entrada **não existe**, e o produto **diz-o**. ⚠️ *O
irmão-filtro do alvo estoirou publicamente por não fazer esta verificação* — é por isso que o gate
desta fronteira é a **recusa**, não o resultado. ⭐ E o roteiro da cena `=43` **começa por ela**: o
passo (1) manda usá-lo sem pilha, porque é ali que o artista aprende de que família este pincel é.

**Inverter não faz nada**, byte-idêntico (§1.2). *Um pincel que ignora o `Ctrl` não é um pincel a que
falta uma feature* — apagar deslocamento tem um só sentido.

### §22.3 — ⛔⛔ CINCO mutações, e TRÊS delas mudaram o método

| # | mutação | o que aconteceu |
|---|---|---|
| 1 | a força volta a ser linear | sangra |
| 2 | o alvo segue a NORMAL | sangra |
| 3 | a referência vira a PREVISÃO | ⛔ **SOBREVIVEU** |
| 4 | sem pilha ele deixa de recusar | ⚠️ **neutralizada** por uma 2.ª guarda |
| 5 | o apagador volta a acumular | ⛔ **SOBREVIVEU** |

**A (3)** sobreviveu porque a minha régua era a **caixa da peça**, e a fixtura era uma esfera UV
subdividida — onde quase todo vértice é regular e as duas superfícies distam menos que a tolerância
da silhueta. *Uma régua indirecta pode ser cega exactamente onde a escolha acontece.* ⇒ o gate novo
mede a **própria tabela de referência** sobre uma base de **CUBO** (valência `3` em todo canto), e
afirma as duas metades: ela **é** o limite, e **difere** da previsão por muito mais que o ruído.

**A (5)** sobreviveu por uma causa que este repo já regista por escrito: ***um corpus no ponto NEUTRO
de um knob não testa esse knob*** — todas as fixturas herdam `Brush::default()`, onde o `accumulate`
nasce desarmado. ⭐ E ao construir a fixtura que o arma, o achado foi **onde** a lei é observável: no
`Grip::Stamp` a coluna aditiva do `GripLaw` é `false` de qualquer maneira, logo o predicado não muda
**geometria nenhuma** — *ele governa uma ROW*. ⇒ o gate mudou de crate: ele passou a afirmar que o
painel **não oferece** o interruptor a quem não o lê, que é a espécie de controlo morto que *todo
gate de registo atravessa verde*.

**A (4)** não é uma sobrevivência: a mutação é **neutralizada** por uma segunda guarda (a contagem de
vértices não bate). Derrubadas as duas juntas, o gate sangra. *Defesa em profundidade lê-se como
sobrevivência num relatório de mutação, e a diferença importa.*

### §22.4 — ⛔ O ARNÊS estava a medir outro programa (a SEGUNDA vez no mesmo dia)

O `um_dab` dos gates de cena não chamava o `open_reference_stroke`, que é a **segunda metade** do
pen-down. Resultado: o apagador movia **zero** vértices mesmo com a pilha montada. ⚠️ A primeira
metade (a foto do desfazer) mordeu horas antes, com o pincel de densidade. *Um arnês a que falta um
passo do produto mede outro programa*, e desta vez o que ele media era um pincel inerte.

### §22.5 — ⚠️ Três coisas que uma leitura rápida do diff entende ao contrário

1. **`Verb::ALL` foi a 30, e a fila do `L` continua em OITO.** O apagador está fora dela pela razão
   que a densidade **perdeu** em 14/09: ele é **inerte na configuração de fábrica** — precisa de uma
   pilha, e uma peça nova tem um nível só. Uma tecla nua responderia com a linha de recusa em vez de
   um gesto.
2. **O `edge_target` da `ph2d-mesh` não morreu** — ele é a lei da referência e o que as bancadas
   medem; o produto é que já não o chama (ver §20.3).
3. **A referência é fotografada no PEN-DOWN, não por dab** — ela é função do nível de baixo, que o
   traço não toca. Recalculá-la por dab custaria um `subdivide` inteiro por evento de ponteiro.

### §22.6 — ⏳ Fica a fila

* *Smear Multires Displacement* (§5) — **destravado**: ele usa a mesma referência que acabou de
  nascer.
* *Scene Project* (§6) — nunca esteve travado; o que falta é o **tecto de custo**, por medir.
* E os dois itens que já eram decisão do dono: o **bordo no `Density`** (§3.8) e a **folga do
  `SCENE_PROJECT`** (§6.4).

---

## §23 — ⭐⭐⭐ O **ESFREGÃO DE DESLOCAMENTO** (`SPEC_unblocked_brushes.md` §5), e a lei que se fecha à mão

O terceiro dos quatro. Ele **não move o vértice para onde a mão vai**: move o **campo de
deslocamento** sobre a superfície de referência, como quem arrasta uma textura sobre uma forma
fixa. A forma grande fica exactamente onde estava; o que viaja é o relevo.

### §23.1 — A lei, e as três coisas que uma leitura rápida inverte

```text
D[u]  = p[u] − R[u]                          // nos nós TOCADOS, uma vez por dab
D′[v] = ( D[v] + Σ_w g(w)·D[w] ) / (1 + Σ_w g(w))
        com  g(w) = max(0, −(d̂ · ê_w))
p[v] ← lerp( p[v], R[v] + D′[v], peso × força² )
```

1. **`g` é a parte NEGATIVA do cosseno** ⇒ só os vizinhos **a montante** contribuem. É isso que
   faz o deslocamento *viajar* em vez de borrar por igual — quem está do lado para onde a mão vai
   tem peso zero.
2. **A vizinhança é medida na REFERÊNCIA**, nunca nas posições deslocadas: é por isso que esfregar
   repetidamente não deforma a topologia.
3. **O peso próprio é `1` FIXO**, fora da normalização dos outros — o travão que impede a
   vizinhança de dominar a média por mais vizinhos que haja a montante.

A lei vive em [`stroke_smear.rs`](../../../crates/ph2d-sculpt3d/src/stroke_smear.rs), irmã do
`stroke_hc` e **pela mesma razão estrutural**: ela lê o campo dos VIZINHOS, e depois de um vértice
se mexer o `D` dele já não é recuperável da posição ⇒ passe de preparação por dab (irmão do
`fit_plane`, do `alpha_frame` e do `fill_hc_disp`), e o alvo sai inteiro de um passe só.

### §23.2 — ⭐⭐⭐ A LEI É COBRADA POR FORMA FECHADA, e a fixtura é um DEGRAU

Num plano triangulado com a referência plana e o relevo a valer `h` só em `x < −0,05`, com a mão a
andar em `+X`, a espec prevê — e o motor entrega, a `< 1e-5` — exactamente isto:

| `x` | `D` antes | `D′` depois | porquê |
|---|---|---|---|
| `−0,20` | `h` | **`h/(1+G)`** | a **ORLA** (§5.4): os vizinhos a montante estão FORA da pegada e entram com **zero** |
| `−0,10` | `h` | **`h`** | a montante é tudo `h` ⇒ a média devolve `h` |
| `0,00` | `0` | **`h·G/(1+G)`** | o relevo **VIAJA** para onde a mão vai |
| `+0,10` | `0` | **`0`** | a jusante não contribui |

com `G = 1 + 1/√2` — o vizinho de aresta (`ê = −x̂`, `g = 1`) mais o **diagonal** que esta
triangulação dá (`g = 1/√2`); os vizinhos em `±ŷ` têm `g = 0` por ortogonalidade.

⛔⛔ **A linha `+0,10` é a lei inteira numa célula:** com `g` escrito como `|cos|` em vez da parte
negativa, ela lê `h·G/(1+G)` e o pincel passa a **BORRAR** em vez de **TRANSPORTAR**. *É a
diferença entre as duas ferramentas, e ela cabe num número.*

⭐ **E a linha `−0,20` reproduz o artefacto que os autores do alvo escolheram MANTER** (§5.4): o
campo só é posto em dia nos nós tocados, e um vizinho de fora entra com o valor que tinha — zero no
primeiro dab. O `0,0739` daquela célula é `h/(1+G)` **à letra**. ⚠️ O zero não é conveniência: é a
cura **publicada** de uma regressão em que vizinhos sem valor definido propagavam `NaN` pela malha;
o que a cura NÃO fez foi alargar a actualização à vizinhança.

### §23.3 — ⛔⛔ A BANCADA DE PARIDADE NÃO PODE EXISTIR HOJE, e há gate com catraca a dizê-lo

A espec §7 declara que o bloco `l` *«torna as famílias de multirresolução utilizáveis sem termos o
§2.3 pronto»*, e o `README` das fixturas repete-o. ⭐ **Isso é verdade para o APAGADOR e FALSO para
este**, e a diferença é a lei:

| pincel | a lei lê | as fixturas trazem |
|---|---|---|
| *Erase* | **um ponto** (`p`, `R[v]`) | tudo o que ela precisa |
| *Smear* | **o ANEL** (`R[w]` de cada vizinho) | ⛔ **nenhuma conectividade** |

As **12** fixturas de `esfregao` trazem `r`, `l`, `s` e `c` — e **nenhum bloco de faces**, apesar de
a linha `blocos:` do cabeçalho delas listar `f=faces` na legenda genérica.

⛔ **E a conectividade NÃO é recuperável — medido, não suposto.** A contagem bate exactamente com um
cubo subdividido cinco vezes (`8 → 26 → 98 → 386 → 1 538 → **6 146**`) e a caixa do bloco `l` é
perfeitamente simétrica (`±0,93268955` nos três eixos), que é a assinatura de uma superfície-limite
de cubo. ⚠️ **Mas a bijecção por posição falha:** escalando a nossa superfície-limite pelo factor
que iguala as caixas (`k = 2,2220`), o emparelhamento por vizinho mais próximo dá **`528` colisões**
e pior distância **`3,04e-2`** — da ordem do espaçamento da grelha (`~4,2e-2`). ⇒ *a malha do
oráculo não é a nossa subdivisão reescalada, e adivinhar a permutação produziria uma bancada que
mede outra malha.*

⏳ **Dívida NOMEADA, e é acto do E:** uma emenda às fixturas que emita o bloco de faces do nível de
topo. O gate [`oraculo_do_esfregao.rs`](../../../crates/ph2d-sculpt3d/tests/it/oraculo_do_esfregao.rs)
tem a **catraca** com as duas metades que a casa exige — o **piso de população** (`12` fixturas) e a
**obsolescência** (o dia em que o bloco `f` aparecer ele reprova e manda construir a bancada).

### §23.4 — ⛔ O `Accumulate` fica de FORA, e MEDIDO antes de escondido

Nesta casa o `Accumulate` é o `from_live` do `Grip::Stamp` — *de onde a curva de queda mede a
distância*. Medido (`o_acumular_do_esfregao_e_uma_lei_que_ninguem_declara`), ele está **VIVO**:
ligá-lo muda a saída.

⇒ **e é exactamente por isso que ele não pode ser oferecido.** Com um alvo ancorado na superfície de
referência, mandar a queda medir da posição JÁ esfregada faria a pegada do pincel depender de quanto
relevo ele já transportou — e a espec §5 escreve a lei **inteira** sem um acumulador. *Um chip cuja
lei nós inventámos é uma LEI vestida com a autoridade de uma fonte que não a declara*, que é a cerca
que o `L` do kelvinlet já paga por escrito.

⚠️⚠️ **Esconder um knob VIVO e esconder um knob MORTO leem-se igual numa tabela de dívida**, e o que
os separa é a medição escrita ao lado. ⛔ A razão do apagador (*«o alvo é absoluto, o segundo dab
tem o mesmo destino que o primeiro»*) **não serve aqui** — o campo `D` é relido a cada dab.

### §23.5 — ⛔⛔ O gate de costura apanhou um defeito ANTES de ele shipar

Os três chips de *Deformation* nasceram **pintados, hit-indexados e MORTOS SOB O DEDO**: faltava a
fileira no `populate`. ⚠️ *Um controlo nunca pintado e um morto sob o dedo dão o MESMO report*, e só
o gesto **REAL** (carregar no centro do rect que o painel registou) os separa — um `Click` sintético
passa com o chip morto. É a sétima vez que esta família o escreve, e a primeira em que o gate
existia **antes** do defeito.

### §23.6 — ⛔ O censo dos gates NOMEADOS apanhou uma recaída minha na mesma hora

Ao cortar o `brush.rs` eu escrevi no cabeçalho do ficheiro novo que a prova de que *«nenhum número
muda»* era o gate `the_factory_brush_is_the_verb_it_declares` — **que nunca existiu**. É exactamente
a forma que a jornada de 13/09 curou nesta família (oito citações dessas), e o censo que ela deixou
reprovou no portão de fecho, **com o endereço e a linha**.

⇒ a nota passa a nomear os censos que de facto defendem a propriedade, e a citação histórica entra
em `MEMORIAS` com o motivo. *Uma promessa de gate lê-se exactamente como um gate, e a diferença só
aparece no dia em que ele devia sangrar.*

### §23.7 — O que mudou, e os dois cortes

| ficheiro | o quê |
|---|---|
| [`smear_mode.rs`](../../../crates/ph2d-sculpt3d/src/smear_mode.rs) | `SmearMode` (`Drag`/`Pinch`/`Expand`) e a porta **única** `direction(path, centre, ponto)` |
| [`stroke_smear.rs`](../../../crates/ph2d-sculpt3d/src/stroke_smear.rs) | o campo `D`, o passe de preparação e o alvo |
| [`brush_verb_campo.rs`](../../../crates/ph2d-sculpt3d/src/brush_verb_campo.rs) | **corte**: o campo elástico sai do `brush_verb.rs` (`678 → 601`) |
| [`brush_default.rs`](../../../crates/ph2d-sculpt3d/src/brush_default.rs) | **corte**: os valores de fábrica saem do `brush.rs` (`693 → 548`) |
| [`scenes_smear.rs`](../../../crates/ph2d-app-sculpt3d/src/scenes_smear.rs) | a cena **`=44`**, cujo passo (1) é a recusa |
| [`esfregao_tests.rs`](../../../crates/ph2d-app-sculpt3d/src/esfregao_tests.rs) | os 3 gates de cena (GPU) |
| [`verb_smear_tests.rs`](../../../crates/ph2d-sculpt3d/src/verb_smear_tests.rs) | os 8 gates de lei (sem GPU) |

⛔ **Os dois cortes são por RESPONSABILIDADE e nenhum entrou no `FILE_OVERAGE_OK`** — a lei do §5.0
do roteador, que esta casa já pagou em cinco painéis.

⚠️ **O `direction` recebe o caminho do gesto E a geometria**, e é isso que o torna a resposta
inteira: o arrasto lê o gesto e ignora onde o vértice está; os outros dois leem a geometria e
ignoram o gesto. ⛔ *Uma porta que só soubesse a metade geométrica devolveria um sentinela para o
arrasto, e o chamador teria de o substituir — que é a segunda resposta à mesma pergunta.*

### §23.8 — As provas

* **Mutação 10 de 10**, cada uma com controlo do próprio filtro: `g` como `|cos|` · o peso próprio
  na normalização · o anel nas posições vivas · a força linear · a orla a ler o slot velho · o
  arrasto parado a ganhar direcção · o modo ignorado · o `Accumulate` de volta · a fileira a sumir
  do painel · a referência a deixar de ser pedida.
* **Portão de fecho:** `fmt` OK · `clippy --workspace --all-targets` **0 avisos** ·
  `nextest-impacted` **15 181 testes: 15 181 passaram** · `doc-index --check` ✓ 19 · as **seis**
  vassouras limpas excepto o `tip_roundness` **pré-existente** (isenção já nomeada no §18 deste
  handoff; `0` linhas de código novas no diff).
* **Zero contador partilhado** (`PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`, `FLIP_SCHEMA`,
  `FIELD_DOC_VERSION`, os registos de componentes), zero contrato congelado, zero ADR, zero pacote
  novo no `Cargo.lock`, zero linha de `shells/desktop/src`.

### §23.9 — O que fica ABERTO

* ⏳ **A bancada de paridade** (§23.3) — bloqueada numa emenda do E, com a catraca escrita.
* ⏳ **A conservação da §5.5 NÃO é gate, e é uma decisão registada:** a espec mede `< 1 %` de deriva
  do deslocamento total **nas fixturas do alvo**, e na nossa (uma esfera com o dab em cima do pico
  do relevo) a mesma lei dá `+5,6 %` a `−21,2 %`. ⛔ Uma barra de `1 %` aqui seria **calibrada sem o
  lado aprovado**, que é a lei que o corpus do tecido já impôs a duas barras desta linha. *O
  mecanismo é conhecido e não é defeito:* a média com peso próprio `1` mais a orla tiram um pouco, e
  a deriva cresce com o comprimento do traço.
* ⏳ **O último dos quatro:** *Scene Project* (§6), que precisa do tecto de custo medido (§10 item 6).

---

## §24 — ⛔⛔⛔ Os **DOIS reports** do dono sobre o esfregão tinham **UMA raiz**, e nenhuma era a lei

> *«O efeito (resultado) parece OK, mas meio travado, até na hora de rotacionar o canvas dá uma
> travadinha»* · *«a intensidade parece baixa mesmo no máximo»* (2026-09-14)

⭐ **A pista está na segunda metade da primeira frase:** rodar o canvas **não corre a lei do
pincel**. ⇒ o custo que ele sente ao rodar não pode ser do alvo por-vértice, e a primeira coisa a
medir não era o kernel — era **o tamanho da peça que o roteiro manda construir**.

### §24.1 — ⛔⛔ A CENA FABRICAVA A PEÇA

As `=43` e `=44` caíam no **default do módulo** (`sculpt_sphere`, **98 306** vértices) e o roteiro
das duas manda apertar **`K` duas vezes** ⇒ **`1 572 866`**. Medido em `--release`:

| vértices | `Draw` | `Smooth` | `Smear` |
|---|---|---|---|
| `98 306` | `0,52 ms` | `0,49 ms` | **`0,65 ms`** |
| `393 218` | `2,88 ms` | `2,75 ms` | **`4,42 ms`** |
| `1 572 866` | `9,72 ms` | `9,47 ms` | `21,9 ms` |

⇒ **a `1,57 M` um dab de `Draw` já custa `9,7 ms` contra o *kill* de `8`.** *Toda* ferramenta
estoura o orçamento ali, e a câmara engasga. ⛔ *A cena ensinava que o pincel é lento quando quem é
pesada é a peça que ela própria mandou construir* — a espécie que o `CLAUDE.md` §5.0 chama de **pior
que uma cena ausente**.

⇒ as duas passam a abrir com o **cubo subdividido três vezes** (`386`), e a escada do roteiro fica
**`386 → 1 538 → 6 146`**. ⭐ O `6 146` **não é escolhido**: é a densidade das **fixturas do oráculo**
destes dois pincéis (espec §7), ou seja o regime em que a lei deles foi medida. ⚠️ É o **cubo** e
não uma `uv_sphere` pela razão que fez o default do módulo mudar por ordem do dono em 2026-08-10: o
leque de pólo dá ao mesmo pincel uma superfície por dab dez vezes menor no pólo que no equador — e
estas cenas tocam no pólo.

### §24.2 — ⛔⛔⛔ E O PINCEL SEGUIA A TESSELAÇÃO — o 2.º report, com número

A média do anel transporta o relevo **UMA ARESTA por dab** (medido: `0,15 ×` a aresta média). ⇒ o
efeito é **inversamente proporcional à densidade da malha**:

| vértices | aresta | raio em arestas | 12 dabs **antes** | 12 dabs **depois** |
|---|---|---|---|---|
| `98 306` | `0,01154` | `30,3` | `0,0297` | **`0,0840`** |
| `393 218` | `0,00577` | `60,6` | `0,0149` | **`0,0848`** |

*Dobrar a densidade cortava o efeito **ao meio, exactamente**.* Contra uma bossa de `0,200`, doze
dabs moviam `15 %` dela na peça de omissão, `7,5 %` na subdividida e **`~3,7 %`** na peça que o
roteiro construía — invisível. ⚠️ É a doença que esta casa já nomeou três vezes: **um efeito que
segue a TESSELAÇÃO em vez da geometria**.

⇒ a cura é `n − 1` relaxações do campo `D` na preparação, mais a do alvo, transportando `n`
arestas, com

```text
n = clamp( round( FRACCAO_DO_RAIO · raio / aresta_da_pegada ), 1, MAX_PASSAGENS )
```

Depois: **`2,8×` mais forte** na peça de omissão, e a diferença entre as duas densidades cai de
`2,00×` para **`1 %`**.

⭐⭐ **A FRACÇÃO NÃO É ESCOLHIDA — ela é a densidade do próprio ORÁCULO.** As fixturas correm com
`6 146` vértices e raio `0,35`, ou seja **`8,3` arestas por raio**, e `1/8,3 = 0,12`. ⇒ ali a lei
devolve **`n = 1`** e a nossa saída é a lei nua **ao bit**: *a correcção não toca no regime onde a
paridade foi medida, e corrige só o que as fixturas nunca cobriram.* Gate
`na_densidade_do_oraculo_a_lei_devolve_uma_passagem`, e a prova dele é o **produto** (a tabela de
forma fechada do §23.2 continua exacta), não a constante.

⚠️ **O TECTO nomeia o recurso, e é o RELÓGIO DO DAB:** a `393 218` vértices um `Draw` custa
`2,88 ms` e cada passagem `0,22` ⇒ `24` passagens põem o esfregão nos `8 ms` do *kill*. `MAX_PASSAGENS = 24`.

### §24.3 — ⚠️ Duas decisões de estrutura que a cura impôs

* **A média do anel virou PORTA ÚNICA** (`media_do_anel`), com **dois** chamadores — a relaxação e o
  alvo. ⛔ Escrita duas vezes, a `n`-ésima passagem deixaria de ser a mesma lei da primeira, e o
  pincel teria uma lei para o meio do dab e outra para o fim.
* **O buffer da relaxação é DUPLO**, e tem de ser: a média lê o `D` dos vizinhos, e escrever no
  sítio a meio do laço faria metade dos vértices ler o valor novo e metade o velho — um
  Gauss-Seidel **dependente da ORDEM da pegada**, a mesma doença que o `stroke_hc` documenta ter
  evitado. *A mutação que o instala sangra.*

### §24.4 — ⛔⛔ E uma MUTAÇÃO SOBREVIVENTE expôs que o gate da cena media a PORTA e não o FIO

O primeiro gate da peça chamava `peca_de_multirresolucao()` **directamente** ⇒ trocar o **despacho**
para devolver o default do módulo deixava-o **verde**. ⚠️ *Um gate que chama a função em vez de
percorrer a rota afirma que a peça certa existe, nunca que a cena a usa* — é o ponto cego que o
`CLAUDE.md` §5.0 nomeia por escrito: **nenhuma sonda deste repo pergunta se o VALOR chega a um
consumidor**.

⇒ a segunda metade lê o despacho por **`include_str!`** e não `read_to_string`, pela lição do HOWTO
§2: o gémeo em runtime só falha *quando o teste corre*, e este ficheiro tem de deixar de **COMPILAR**
no dia em que o irmão mudar de sítio.

### §24.5 — As provas

* **Mutação 14 de 14** (eram 10; as quatro novas: a lei das passagens morta · as passagens
  desancoradas do raio · o Gauss-Seidel · a peça da cena de volta ao default).
* **Gates de lei: 10** (eram 8) — os dois novos são `o_transporte_do_esfregao_nao_segue_a_densidade_da_malha`
  (a malha fina contra a grossa na mesma peça) e a calibração acima.
* **Portão:** `fmt` OK · `clippy` **0 avisos** · `nextest-impacted` **15 184: 15 184 passaram** ·
  `doc-index --check` ✓ 19 · as seis vassouras limpas excepto o `tip_roundness` pré-existente.
* ⚠️ **As medições vivem no caminho do produto** (`mede_o_esfregao.rs`, `#[ignore]`), e **em
  `--release`**: o debug lê `~20×` mais lento e daria um tecto de passagens cinco vezes menor — *a
  mesma cura mede-se cinco vezes menor no perfil errado* (a lição da `line/3DModeling` em 10/09).

---

## §25 — ⛔⛔⛔ O **PANIC ao desacoplar a janela maximizada**, e por que ele era ESTRUTURAL

> ```text
> In a CommandEncoder, label = 'ph2d-mesh view'
>   In a set_scissor_rect command
>     Scissor Rect { x: 244, y: 96, w: 1399, h: 926 }
>     is not contained in the render target (1024, 768, 1)
> ```
> — report do dono, 2026-09-14

### §25.1 — Não é um acidente de um quadro

O `size` que o passe recebe é o do quadro de **AGORA** (a superfície acabou de ser reconfigurada
pelo gestor de janelas) e a `ScreenRect` é a área que o painel **PUBLICOU no quadro ANTERIOR**.

⚠️ **E isso é assim de propósito**, com a razão escrita no `view.rs` desde a wave dos quatro
viewports: *«o desenho e o pick derivam do mesmo rectângulo, e desenhar num de recurso enquanto o
pick usa outro é a família inteira de “o lugar onde o rato toca não corresponde ao sítio na
malha”»*.

⇒ **todo redimensionamento tem um quadro em que as duas discordam.** A discordância era um `panic`,
e reordenar não a remove — ela é o preço de o desenho e o pick partilharem a fonte.

### §25.2 — A cura é RECORTAR

`ScreenRect::clip_to(size) -> Option<Self>` é a porta. Os **três** passes da crate (cor, SSAO,
G-buffer) recortam à entrada, e o `set_area` recebe o alvo e recorta também — *ele deixa de poder
ser chamado errado*.

⭐ **A linha que decide que esta cura é segura é a do REGIME NORMAL:** uma área que já cabe volta
**ela mesma, ao bit**. Sem ela, esta correcção mudaria toda a imagem que já shipava — e é por isso
que ela é a primeira asserção do gate sem GPU.

⚠️ **O `set_viewport` é recortado também**, e não só o scissor: o `wgpu` valida os dois. O preço é
um quadro de transição com a peça ligeiramente achatada — invisível ao lado de um `panic`.

### §25.3 — ⛔⛔ Uma mutação SOBREVIVENTE mostrou que os dois recortes não são redundantes

Matar **só** o da entrada não estoura — e abre um defeito **pior que o `panic`**: com a área
inteiramente fora do alvo, o `set_area` devolve cedo e **não chega a chamar `set_scissor_rect`** ⇒ o
passe fica com o scissor por omissão, que é **o alvo inteiro**, e a malha é pintada por cima da
janela toda.

⇒ *um recorte que desiste em silêncio não recorta nada*, e quem decide é a **entrada**: sem vista, o
passe não se abre. Gate `uma_vista_inteiramente_fora_do_alvo_nao_pinta_a_janela_toda`.

### §25.4 — ⛔ E o ARNÊS da mutação mentiu duas vezes antes de dizer a verdade

As duas formas que este repo já tem escritas, as duas na mesma corrida:

1. **Um `-- --ignored` sobre um teste que NÃO é `#[ignore]` corre ZERO testes e imprime `ok`** — e
   lê-se exactamente como *SOBREVIVEU*. (`feedback_a_mutation_proof_needs_a_control_on_its_own_filter`.)
2. **A redundância dos dois recortes** fazia a mutação de um deles ser **inobservável** — e a cura
   não foi afrouxar a régua, foi escrever o gate que observa o que só aquele recorte impede.

⇒ o arnês passa a **CONTAR quantos testes de facto correram**, e acusa `zero` como falha própria.

### §25.5 — As provas

* **Mutação 3 de 3**, e a do meio **devolve o `panic` do dono à letra** (`not contained in the
  render target`).
* Três gates: a intersecção sem GPU (com a linha do regime normal) · o `panic` do report
  reproduzido · a vista inteiramente fora.
* `clippy` **0 avisos** · `nextest-impacted` **15 185: 15 185 passaram**.
* ⚠️ **A crate é partilhada com o modelador 3D** (`ph2d-app-field3d`) — nenhuma assinatura pública
  mudou, só o comportamento no regime que antes estourava.

---

## §26 — ⭐⭐⭐ **QUEM SUBDIVIDE NO DYNAMIC TOPOLOGY** — a tabela inteira, com a proveniência de cada célula

> *«algumas tools que não deveriam fazer a subdivisão de polígonos no modo Dynamic Topology estão
> fazendo (como smooth) enquanto algumas que deveriam criar subdivisões com Dynamic Topology não
> estão criando.»* — o dono, 2026-09-14

### §26.1 — ⭐ A triagem parou na primeira porta ABERTA, e metade da tabela saiu no mesmo dia

| fonte | licença | como | o que deu |
|---|---|---|---|
| a **livre** | **MIT** | ⭐ lê-se e porta-se, com atribuição (§0.9) | **13** ferramentas, na hora |
| a **medida** | GPL | ⛔ corre-se **sem interface**, por uma janela **E**; a saída é DADO (GPLv2 §0) | **27** tipos, **343** células |

⚠️ **As duas confirmaram o report nos dois sentidos** — e o `Smooth`, que ele nomeou à letra, é
`441 → 441` na medida e não tem uma única chamada à topologia dinâmica na livre.

### §26.2 — A tabela

| verbo | mexe? | de onde |
|---|---|---|
| Draw · Clay · Inflate · Flatten · Fill · Scrape · Pinch · Crease · Blob · Clay Strips · Clay Thumb · Multiplane Scrape | ✅ | as duas |
| **Snake Hook** | ✅ | **as duas** — precisou de **fiação** |
| **Nudge** | ✅ | a medida (`441 → 2 853`) — **fiação** |
| **Twist · Local Scale** | ✅ | ⚠️⚠️ **as duas DISCORDAM** (§26.4) |
| **Smooth** | ⛔ | **as duas** |
| **Slide Relax · Surface Smooth · Layer** | ⛔ | a medida (`441 → 441` nos dois extremos) |
| Move · Thumb · Pose · Boundary · Cloth | ⛔ | as duas |
| **Mask** | ⛔ | ⭐ **domínio**, curado um dia ANTES — e as duas confirmaram |
| Density | ✅ | ⛔ construção: ele **É** o passe |
| Erase · Smear Displacement | ⛔ | ⛔ construção: multirresolução exclui dyntopo |
| **Sharpen · Magnify** | ✅ | ⚠️ **nenhum oráculo os responde** — conservador, **com gate a nomeá-los** |

### §26.3 — ⭐⭐ A MÁSCARA: a cura que o oráculo depois confirmou

Ela foi curada em 14/09 por **raciocínio de domínio** — *um gesto que não escreve posição não tem
porque mudar a topologia* — sem consultar alvo nenhum. A referência livre **sobrescreve a lei geral
dela** só para esta ferramenta, para dizer o mesmo.

⇒ *uma cura que o oráculo depois confirma é a melhor prova de que o raciocínio que a produziu era do
DOMÍNIO e não do programa.*

### §26.4 — ⚠️⚠️ As duas células em que as referências DISCORDAM

`Twist` e `Local Scale`: a livre diz que mexem, a medida devolve `441 → 441`.

⭐ **Fica o `true`, e o argumento é GEOMÉTRICO e não de voto:** uma torção com queda **cisalha** a
superfície e uma escala local **estica-a radialmente** — os dois produzem aresta longa, que é
exactamente a metade do report. E o irmão em que as duas **concordam** (o gancho) é da mesma
família: *quem transporta matéria estica a malha atrás de si*.

⛔ **A outra leitura está registada e é legítima:** na referência medida, `ROTATE` e a escala
elástica pertencem à família que move uma região **como um corpo**, onde a densidade viaja com o
material — e ali eles não refinam, como o agarrar. **É decisão de produto, e o dono tem os dois
números.**

### §26.5 — ⛔ Nenhuma das duas separa as duas colunas

Quem refina também colapsa, nas duas referências. ⚠️ **É facto sobre elas e NÃO uma lei** — a
separação **é exprimível** (a medida tem o modo de refino da cena com três estados) e nós podemos
querê-la antes delas. É por isso que [`Verb::colapsa_no_dyntopo`] continua a existir separado.

### §26.6 — A fiação, e a armadilha nº 1 do plano que NÃO se aplicou

Os quatro que passaram a mexer têm **âncora**, logo não passam pelo braço do carimbo — a porta foi
ligada no `hook_step` e no `turn_at`. ⛔ **Quem decide continua a ser o VERBO**; *ligar o fio não é
responder à pergunta, é deixar a resposta chegar*.

⭐ **A armadilha nº 1 do [plano 22](../22_plano_quem_subdivide_no_dyntopo.md) §5 (a pegada CONGELADA
que não sabe crescer) não se aplica a estes quatro:** ela é lida **só pelo polegar**
(`congela = matches!(verb, Thumb)`), e o `grow_with`/`shrink_with` já cobre todo o resto do estado
por-índice do traço. *Uma armadilha nomeada que a medição dissolve vale tanto como uma que morde.*

### §26.7 — ⛔⛔ Duas coisas que o ARNÊS da mutação apanhou em mim

1. **Um braço `Density => true` que era REDUNDANTE com o fallback** — apagá-lo não mudava um bit.
   *Uma linha que a mutação não consegue matar não é lei, é comentário com sintaxe de código*; ela
   saiu e o comentário ficou.
2. **Uma mutação minha que era um NO-OP** (acrescentar à lista dos `true` um verbo que o fallback já
   punha em `true`). O controlo do arnês apanhou-a, e ela foi reescrita para o sentido que de facto
   muda o produto.

### §26.8 — ⛔⛔⛔ E um ACHADO DE INSTRUMENTO que vale para TODAS as fixturas comprimidas do repo

**A vassoura era CEGA ao conteúdo de um `.gz`.** O sweep lê binários por `strings`, e um ficheiro
gzipado é **opaco** a isso — medido: um canário plantado dentro de um `.gz` passava despercebido.

⇒ *um sweep verde sobre um `.gz` não provava nada sobre o conteúdo dele* — e este repo guarda os
corpora de oráculo comprimidos: o do **tecido** (86 traços), o da **pose** (69), os dos **pincéis
desbloqueados** (60) e agora o do **dyntopo** (343 células).

A cura descomprime **em memória** (nada do alvo toca o disco) e varre o texto, com o nome do `.gz`
no rótulo. ⚠️ **O reconhecimento é pelos dois bytes mágicos, nunca pela extensão** — um corpus
comprimido com outro nome escaparia a uma régua que olhasse o sufixo.

⭐ **Controlo positivo conferido**, e ele é o que separa esta cura de uma afirmação: com um canário
plantado o instrumento acusa e sai `exit = 1`. Corrido sobre as **seis** vassouras vivas contra
todas as fixturas do repo: **limpas** — *elas sempre estiveram, e até hoje o instrumento não o
conseguia provar*.

### §26.9 — As provas

* **Mutação 10 de 10** (a metade livre) + **10 de 10** (a tabela completa).
* Gate de produto (GPU) com **seis** células e **dois controlos** dentro: o `Draw` refina (senão o
  arranjo é inerte) e o `Move` não (senão ligar a porta a todo gesto ancorado passaria).
* **Três** gates de tabela: proveniência por célula · os nove ancorados · os dois sem oráculo.
* Tecto de LOC curado por **corte** (`brush_verb_dyntopo.rs`), nunca por isenção.
* `clippy` 0 · `nextest-impacted` **15 186: 15 186 passaram** · `doc-index` ✓ 19 · seis vassouras
  limpas (com o instrumento curado) excepto o `tip_roundness` pré-existente.

### §26.10 — O que fica ABERTO

* ⏳ **A decisão do dono sobre `Twist` / `Local Scale`** (§26.4) — as duas referências discordam e
  ele tem os dois números.
* ⏳ **`Sharpen` e `Magnify`** — sem oráculo nenhum, nomeados por gate.
* ⏳ **Separar as duas colunas** — hoje nenhuma referência o faz, e nós podemos querer.

---

## §27 — ⭐⭐⭐ O **VEREDITO DO DONO** sobre cinco células do dyntopo, e a **CADEIA** que a pegada congelada expôs

> *«acho que layer, move/drag deve subdividir. Thumb se for possível, deveria
> subdividir. Twist com dynamic topology fica com resultado muito ruim.»*
> — o dono, 2026-09-14, depois de correr o smoke `=14`

### §27.1 — A TERCEIRA fonte da tabela, e por que ela ganha das duas

A tabela do §26 tinha duas fontes: a **livre** (MIT, lê-se e porta-se) e a
**medida** (GPL, corrida sem interface por uma janela E). O dono é a terceira, e
ela ganha das duas **pelo mesmo princípio que dá o lugar ao oráculo**: *uma
referência responde o que outro programa FAZ; o dono responde o que este produto
TEM DE fazer.*

| célula | era | fica | de onde |
|---|---|---|---|
| **Layer** | ⛔ | ✅ | o dono — divergência da medida (`441 → 441`) |
| **Move / Grab** | ⛔ | ✅ | o dono — divergência das **DUAS** |
| **Thumb** | ⛔ | ✅ | o dono — custou a peça da §27.3 |
| **Twist** | ✅ | ⛔ | o dono **desempata** as duas, a favor da medida |
| **Local Scale** | ✅ | ⛔ | ⚠️ **ele NÃO a nomeou** — ver §27.2 |

⭐ **O que o veredito do Twist derruba é um argumento MEU.** Eu tinha escrito ao
lado do `true`: *«uma torção com queda cisalha a superfície, logo produz aresta
longa»*. Ela cisalha, sim — e o que o refino faz com esse cisalhamento é
**estragar a forma**, não acompanhá-la. ⇒ a referência **medida** (`441 → 441`
nos dois extremos do slider) estava certa, e o voto geométrico estava errado.

⭐ **E o argumento das duas referências contra o agarrar também cai, com o
mecanismo:** elas dizem que ele *«move uma região como um corpo, e a densidade
viaja com o material»*. Isso é verdadeiro no **MIOLO** do agarrar e **falso no
ANEL**, onde o barro que anda encontra o barro que ficou — e é ali que a aresta
estica.

### §27.2 — A célula que ele NÃO nomeou, e ela fica NOMEADA em vez de silenciosa

A `Local Scale` seguiu a `Twist`: mesmo `Grip::Turn`, mesma célula nas duas
referências, mesma família na medida (*mover uma região como um corpo*).
Deixá-la sozinha do outro lado seria **fabricar uma divergência que nenhuma das
três fontes pede**.

⚠️ *Uma herança silenciosa e uma decisão leem-se igual numa tabela* — é a mesma
doença do `❌ recusado com motivo` contra o `❌ ninguém fez` que o §5.0 do
`CLAUDE.md` já nomeia. ⇒ ela está escrita, com a razão, em
`o_veredito_do_dono_sobre_cinco_celulas`. **Se ele a quiser de volta a adensar, é
essa linha que muda.**

### §27.3 — ⭐⭐ O VEREDITO É UM GATE, e não um braço de `match`

**Três das cinco células coincidem com o valor de fábrica** (`_ => !self.anchors()`):
o `Layer` é de carimbo e já respondia `true`; a `Twist` e a `Local Scale` têm
âncora e já respondiam `false`.

- ⛔ **Escrever-lhes um braço** seria uma linha que **a mutação não consegue
  matar** — o defeito exacto que o `Density` pagou neste mesmo ficheiro, e que o
  §26 registou (*«uma linha que a mutação não consegue matar não é lei, é
  comentário com sintaxe de código»*).
- ⛔ **Deixá-las sem nada** faria a decisão do dono depender de o `anchors()`
  nunca mudar, e **nada liga as duas perguntas**: quem mexer num grip amanhã
  inverte um veredito de produto sem que uma linha do diff o diga.

⇒ **a decisão vai para onde ela pode ser AFIRMADA — um gate —, e não para onde
ela por acaso já é verdade.**

⚠️ **E `os_nove_ancorados_tem_resposta_e_quatro_deles_mexem` mudou de POPULAÇÃO
sem mudar de CONTAGEM:** continuam a ser quatro, e são outros dois. *É
exactamente como um piso segura o número enquanto a lista que ele descreve muda
por baixo dele* (a armadilha que o `every_host_that_rewrites_verts` pagou na W2)
— e é por isso que o corpo daquele gate compara a **LISTA**, nunca o tamanho.

### §27.4 — A fiação: o TERCEIRO caminho, e ele faltava

A wave da manhã (§26) ligou a porta da topologia aos dois gestos ancorados que
**PERCORREM** (`hook_step`) e ao que **GIRA** (`turn_at`). Quem **SEGURA**
(`Grip::Hold`) não percorre nem gira — ele **regista** (`pending_grab`) e quem
carimba é o **quadro** —, logo continuava a ser o único gesto de escultura que
não passava pela porta. *Um fio ligado em dois dos três ramos lê-se como ligado.*

⛔ Quem decide continua a ser o **VERBO**: o `Pose` e o `Boundary` seguram também
e a tabela responde-lhes `false`.

### §27.5 — ⛔⛔ O ARNÊS media outro programa, e o contrafactual tem NÚMERO

O `um_traco_ancorado` conduzia os **três** grips ancorados pelo `hook_step`.

⚠️⚠️ **Medido, e é o que justifica a correcção:** com o arnês de antes **e** o
`refine_for_dab` apagado do `grab_at`, o gate `o_smooth_deixou_de_subdividir…`
fecha **VERDE**. Ou seja: o caminho que o agarrar e o polegar de facto tomam
podia ter o fio da topologia desligado **sem uma linha vermelha em lado nenhum**.

⛔ **Uma mutação só no arnês NÃO sangra** (medido: sobrevive), e é exactamente
por isso que ela é precisa — *ela não corrige um defeito, torna um defeito
OBSERVÁVEL*. Este ficheiro já tinha pago a frase duas vezes, pelo pen-down do
desfazer e pelo da superfície de referência.

⭐ **E o gate ganhou DOIS controlos negativos, um por caminho:**

| controlo | caminho | o que ele impede |
|---|---|---|
| `Twist` | quem **gira** | o fio está **LIGADO** e quem recusa é a **TABELA** — *um controlo negativo sobre um caminho desligado não afirma nada* |
| `Pose` | quem **segura** | o fio NOVO desta wave passar a responder pelo **grip** em vez do verbo |

### §27.6 — ⭐⭐⭐ O «se for possível» do polegar: a pegada congelada atravessa a topologia

O `Thumb` é o **único** verbo que congela a pegada no pen-down
(`congela = matches!(brush.verb, Verb::Thumb)`), e um índice guardado não
sobrevive sozinho a uma malha que muda de tamanho. ⚠️ O §26 declarava esta
armadilha **dissolvida** (*«a pegada congelada não entra nisto e não precisa»*) —
e a frase era verdadeira **enquanto o polegar não declarava nenhuma das duas
colunas**. O dono mandou-o declarar.

**As duas metades**, em `crates/ph2d-sculpt3d/src/stroke_pegada.rs`:

- **CRESCE** — a pegada acolhe quem nasce entre **DOIS** membros. ⭐ A regra é
  **EXACTA e não conservadora**: a pegada é o resultado de uma consulta por
  **ESFERA**, uma esfera é **convexa**, e o vértice novo nasce no ponto médio dos
  pais. ⛔ *«Um pai basta»* está errado por duas vias — admite pontos médios fora
  da bola, e faz a pegada **CRESCER para fora** a cada refino, reabrindo o
  defeito que congelá-la existe para não ter (*o conjunto amostrado voltaria a
  depender de quantos eventos o traço teve*).
- **ENCOLHE** — aplica a renumeração e larga os mortos.

### §27.7 — ⛔⛔⛔ E a segunda metade estava ERRADA: **a tradução de um colapso é uma CADEIA**

**O modo de falha foi o bom:** `index out of bounds` na máscara de alcance
(`dab_alcance.rs`), no primeiro dab a seguir a um colapso — `len 2856, índice
2859`. *A pegada guardava um índice maior que a malha.*

⚠️⚠️ **O plano do colapso (`ph2d_mesh::Remap`) é uma SEQUÊNCIA de trocas, e ela
aplica-se por ordem:**

```text
(11 → 10), (10 → 9), (9 → 8)
```

quer dizer que o vértice que começou em `11` acaba em **`8`**, passando por dois
endereços que **não são dele**. ⇒ *um índice que é destino de uma troca pode ser
origem da seguinte*, e **não existe** a tabela plana `origem → destino` que eu
escrevi primeiro. Com ela, um índice do meio de uma cadeia ficava guardado como
se fosse final.

⛔ **Se a cadeia tivesse acabado DENTRO do novo tamanho, o carimbo teria pegado
no barro errado, em silêncio** — o `panic` foi sorte, não desenho.

⭐ **Duas propriedades do plano tornam a cura barata e provável:**

1. **A cadeia é finita e não tem ciclos por construção** — as origens das trocas
   descem estritamente (o plano varre os mortos do maior para o menor) e todo
   destino é menor que a sua origem.
2. **A morte consulta-se PRIMEIRO** — um índice que é sobrescrito **e** é origem
   de uma troca é sempre sobrescrito **antes** de se mudar (o contrário obrigaria
   uma origem a crescer ao longo do plano, e elas só descem).

⚠️ **As duas tabelas são do tamanho do COLAPSO e nunca da malha:** um vetor de
tradução por VÉRTICE seria `O(malha)` por dab — numa peça de um milhão de
vértices, megabytes a limpar por carimbo para traduzir umas centenas.

### §27.8 — Os gates, e o que cada piso pagou

| gate | o que afirma |
|---|---|
| `a_traducao_de_um_colapso_segue_a_cadeia_e_nao_uma_tabela_plana` | o oráculo é a aplicação **SEQUENCIAL** das trocas, perguntada a **todos** os vértices de **quatro** planos |
| `a_pegada_congelada_do_polegar_atravessa_a_topologia` | ponta a ponta, com a malha a mexer-se nos **dois** sentidos |
| `o_veredito_do_dono_sobre_cinco_celulas` | as cinco células, com o que ele disse ao lado de cada uma |

⛔⛔ **A segunda espécie de morte foi achada por uma MUTAÇÃO SOBREVIVENTE.** Com
um plano só — três mortos seguidos —, apagar a cerca do novo tamanho **não era
observável**: aquele plano resolve todos os seus mortos por sobreposição, e
nenhum vértice chega a ser **truncado**. ⇒ o corpus passou a quatro planos e o
gate **exige as três espécies** (sobrescrito · truncado · em cadeia), cada uma
com o seu piso. *Uma régua que não vê o fenómeno acontecer não prova que ele não
aconteceu.*

⚠️ **E o piso do crescimento é um nascimento A CAVALO na fronteira:** sem ele,
*«os dois pais»* e *«um pai basta»* dão a **MESMA** pegada, e aquela metade
ficaria verde sobre as duas leis.

⭐ **O crescimento tem ORÁCULO INDEPENDENTE** — a regra reconstruída à mão no
gate, incluindo a parte que a torna sequencial (*um recém-nascido acolhido passa
a ser membro para os que nascerem depois dele no mesmo passe*).

**Mutação: 12 de 12.** (7 da pegada · 1 do polegar na tabela · 2 das outras
células · 1 da fiação · e a 12.ª, a do arnês, **documentada como não-sangrante
de propósito** — ver §27.5.)

### §27.9 — O corte de LOC, e ele foi meu

`stroke.rs` foi de **698 para 739** linhas por minha causa ⇒ curado por **CORTE
e nunca por uma entrada no `FILE_OVERAGE_OK`**. O assunto *«a pegada congelada»*
tinha a declaração num ficheiro, a razão de existir noutro e as duas metades da
manutenção num terceiro; hoje vive em **`stroke_pegada.rs`** (227 L), irmão do
`stroke_growth.rs` — ⭐ e o corte é de **ASSUNTO**: lá o `pre` de cada vértice a
sobreviver à topologia, aqui o **CONJUNTO** de vértices do gesto a sobreviver a
ela. São peças diferentes do estado de um traço, com leis diferentes: o `pre`
**herda-se** de pais, a pegada **acolhe** e **larga**.

Ficheiros depois do corte: `stroke.rs` **699** · `stroke_growth.rs` **271** ·
`stroke_pegada.rs` **227** · `stroke_pegada_tests.rs` **318** ·
`brush_verb_dyntopo.rs` **633**.

### §27.10 — ⚠️ Uma FLAKE DE CARGA nova, com as três assinaturas

`the_frame_is_hoisted_out_of_the_vertex_loop`
(`ph2d-sculpt3d/tests/it/`) reprovou **uma vez** no pico de um fan-out de
**15 189** testes, a `load 20,10`.

| assinatura | medida |
|---|---|
| único ✗ da corrida | `15 188 / 15 189` |
| zero linhas do diff naquele caminho | o alpha/padrão não é tocado por esta jornada |
| verde sozinho | **3 de 3**, e a **`load 60,46`** |

⭐⭐ **A terceira é a mais forte que esta linha já mediu:** ele passa a uma carga
**três vezes maior** do que aquela em que reprovou ⇒ *o discriminador é o
FAN-OUT, não o relógio* — a mesma assinatura que o `no_expression_allocates…`
trouxe em 12/09.

⛔ **E ele é o QUINTO gate deste repo cujo doc-comment se declara imune:** *«é
uma RAZÃO e não um kill de relógio de propósito … os dois lados são o MESMO
trabalho no MESMO perfil»*. É verdade sobre o **perfil de build** e falso sobre o
**fan-out**: os dois lados são relógios de parede, e sob 15 mil testes em
paralelo o escalonador não os trata por igual. ⇒ **promoção pedida** à lista do
`CLAUDE.md` §5.0.

⭐⭐⭐ **E a SEGUNDA corrida da MESMA árvore deu a assinatura mais forte da
família, à vista:** ela reprovou **três** testes e **nenhum** deles é o da
primeira.

| corrida | reprovadas | verde sozinho |
|---|---|---|
| 1.ª (`load 20,10`) | `the_frame_is_hoisted_out_of_the_vertex_loop` | 3/3 a `load 60,46` |
| 2.ª | `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` · `an_abandoned_march_returns_nothing_and_returns_fast` · `measure_normals_parallel_speedup` | 3/3 cada, a `load 51,25` · `35,81` · `33,26` |

⛔ **Intersecção VAZIA entre as duas corridas do mesmo commit** — *um defeito de
lógica reprova o mesmo caso sempre; só um recurso partilhado troca de vítima
entre corridas.* E as **três** da segunda já estavam nomeadas na lista do §5.0,
o que deixa **uma** promoção a pedir e não quatro.

⚠️ **E a 2.ª corrida foi POLUÍDA por mim:** eu pus a build do smoke a correr em
cima dela. *Uma varredura de fecho corre sozinha, ou o número que ela devolve é
sobre outra máquina* — fica registado porque explica por que a segunda teve três
reprovadas e a primeira uma.

### §27.11 — O que NÃO se mexeu

- **Zero** contadores partilhados (`PROJECT_SCHEMA`, `FIELD_DOC_VERSION`,
  `VEC_SCENE_SCHEMA`, `FLIP_SCHEMA`, os três registos de componentes).
- **Zero** contratos congelados (§6), **zero** ADR, **zero** pacote externo.
- **Zero** portas novas no `AppHost`, **zero** linhas de shell.
- ⛔ **Nenhuma assinatura pública mudou** — `PegadaCongelada` e as duas metades
  são `pub(super)`.
- ⏳ **ABERTO e NOMEADO, pré-existente:** o `tip_roundness` da
  `VASSOURA_blender-pull.txt` (7 ficheiros, **zero adições** neste diff) — é a
  mesma isenção do §8, e continua a ser dívida da linha dona.

---

## §28 — ⭐⭐⭐ O **QUARTO e último** pincel desbloqueado: **PROJECTAR NA CENA** (`SPEC_unblocked_brushes.md` §6)

O barro é empurrado até **encostar noutra peça da cena** — a ferramenta de
sentar uma peça numa mesa, de apertar um rosto contra uma parede. Cena **`=45`**.
Com ele, `SPEC_unblocked_brushes.md` fecha: `Density` · `Erase` · `Smear` ·
`Scene Project`.

### §28.1 — ⭐⭐⭐ A bancada EXISTE, e a do esfregão não — a diferença está MEDIDA

As `24` fixturas **não trazem o objecto-alvo**: nem o cabeçalho nem o `README`
dizem onde ele está. Era a **mesma forma de bloqueio** que parou a bancada do
esfregão (§23: lá faltava a conectividade), e a resposta honesta era **medir
antes de declarar**.

| o que | como se recupera | resíduo |
|---|---|---|
| a malha de entrada | grelha `41×41` sobre `[−1,1]²` em `z = 0`, **row-major** | **`0,000000`** contra a nominal, nas `24` |
| a direcção da vista | o deslocamento é inteiramente em `z` | — |
| o plano-alvo | a fixtura de curva **Constant** e força `1` move o miolo o vão INTEIRO, e os `146` vértices pousam **todos** em `z = −0,500000` | `0` de dispersão |

⭐⭐ **E a recuperação é CONFIRMADA por uma segunda fixtura que não a produziu:**
`projectar_forca05_constante_1passo` pousa em `−0,125000`, que é `0,5 × 0,5²` —
*a lei da força ao quadrado (§1.1) e a posição do alvo a confirmarem-se uma à
outra, ao último dígito impresso.*

⚠️⚠️ **E as `24` partilham a MESMA malha de entrada** (o mesmo `sha256` no bloco
`r`), logo a calibração **transfere por construção** e não por suposição. ⛔ Sem
esse facto isto seria *«adivinhar a permutação»*, que é exactamente o que a
bancada do esfregão recusou fazer — e ali com razão.

### §28.2 — O placar: o que está PROVADO, e é mais que um número de paridade

**As `16` reconstrutíveis movem EXACTAMENTE o mesmo conjunto de vértices que o
oráculo** (`301` contra `301`, zero discordâncias fora da banda de empate).
⭐ Isso não é um detalhe: é a pegada, a direcção do raio, a regra dos dois
sentidos, a escolha entre dois alvos, a inversão e as **três recusas** da §6.3 —
todas estruturalmente certas. *Um conjunto igual com magnitudes diferentes é um
diagnóstico muito mais preciso que um número agregado.*

Dentro da barra apertada (`2e-6`): **`3`** — as duas de **UM** dab (`7,078e-8`) e
a que não move nada. As outras `13` correm **seis** dabs e desviam `2,8e-3` a
`2,6e-1`.

⭐ **A partição é LIMPA e diz onde procurar:** tudo o que corre em **um** dab bate
ao sétimo decimal; tudo o que corre em **seis** desvia. ⇒ o que falta **não é a
lei do raio** — é a **composição por dab**, que a espec §8.4 nomeia como a coisa
que um traço inteiro mistura.

⛔⛔ **A barra NÃO foi afrouxada para os engolir.** Uma barra de `3e-1` faria o
gate ficar verde sobre qualquer coisa — *uma barra que aceita o desvio que se tem
mede o desvio que se tem*. A tabela inteira vive no cabeçalho da bancada, com
catraca nos **dois** sentidos (ela reprova se o placar descer **e** se ele subir
sem a tabela ser actualizada).

⛔ **As SEIS que ficam de fora, e o que falta a cada uma:** as três do alvo-esfera
(a **tesselação** do alvo *é* a medição), a do alvo inclinado e escalado (a nossa
`Pose` **não tem rotação** para a exprimir), a `normal_plano_x` (a orientação que
faz a normal da área apontar `+X`) e a `simetria_x` (a altura do alvo não é
separável do efeito da simetria). ⏳ **Dívida nomeada, acto do E:** uma emenda que
emita a geometria e a pose de cada alvo.

### §28.3 — ⛔⛔ TRÊS leis que o corpus deu e a espec não dizia assim

1. **INVERTER VIRA O RAIO, não o sinal do deslocamento.** Escrito como negação no
   fim, `projectar_invertido` movia **`0` vértices contra `301`**: o alvo dela
   está ACIMA e os dois sentidos estão DESLIGADOS ⇒ *não há o que negar quando o
   raio não acertou em nada*. ⭐ Do lado do artista: `Ctrl` aqui não quer dizer
   *«afasta»*, quer dizer **«procura do outro lado»**.
2. **O `w × strength` do APAGADOR não transfere, e a linha que os separa é
   invisível a olho:** quem **declara** um perfil de referência já recebe a força
   ao quadrado no `w`. Medido a `0,5` de slider — `Scene Project` em `B` dá
   **`0,25`**, o apagador dá `0,5`. Escrito à maneira dele, este media `0,0625`
   onde o oráculo mede `0,125`: a força ao **CUBO**.
3. **Com a curva CONSTANTE um empate de último bit vale o VÃO INTEIRO.** Três
   vértices da grelha caem a `6e-9` do raio, e uma barra de posição lia `5,0e-1`
   sobre uma lei certa ao sétimo decimal ⇒ a régua tem **duas metades**: a LEI no
   miolo comum, e a BORDA contra uma banda de empate **derivada da geometria**.

⚠️ **E o ARNÊS mentiu antes de dizer a verdade:** ele comparava a nossa saída
contra o repouso **DELES**, e a reconstrução bate a do corpus a `<1e-6` — exacto
para a geometria e **enorme** para um teste de igualdade. `1 069` dos `1 681`
liam-se como movidos por nós sem se terem mexido. *Cada lado mede-se contra o
próprio repouso.*

### §28.4 — O substrato: uma PORTA que já existia com o nome errado

O instantâneo das outras peças da cena existia como `cloth_colliders`. Ele passa
a ser **`pecas_da_cena`**, com um predicado por verbo
(`Brush::precisa_das_pecas_da_cena`): o tecido só quer as peças com a colisão
ligada (ela é cara), a projecção quer-as sempre — *sem elas ela não tem lei
nenhuma*. ⛔ Escrito como um `if` de duas pernas no sítio da fotografia, o
terceiro consumidor herdaria a perna errada em silêncio.

⭐ Mais **`pose_activa`**, e ela não é conforto: o `t` de um acerto vem em
unidades da peça **consultada** (o `Ray` normaliza a direcção), logo dois
candidatos em peças com escalas diferentes **não são comparáveis** pelo `t` cru —
e o `min |d|` da espec escolheria pelo número errado, em silêncio, porque os dois
são `f32` plausíveis.

### §28.5 — ⛔⛔⛔ O TECTO DE CUSTO, e ele achou um defeito de OUTRA CRATE

| vértices | `Draw` | 1 alvo | 2 alvos | 4 alvos | 1+bidir | alvo 160k | 160k+bidir |
|---|---|---|---|---|---|---|---|
| `9 902` | `0,028` | `0,021` | `0,019` | `0,025` | `0,019` | `0,051` | `0,046` |
| `39 802` | `0,100` | `0,075` | `0,075` | `0,081` | `0,071` | `0,218` | `0,190` |
| `159 602` | `0,401` | `0,313` | `0,326` | `0,335` | `0,299` | `0,744` | **`0,697`** |

⇒ o pior caso é **`0,697 ms`** contra um orçamento de `8` — `1,7×` um dab de
`Draw`. ⛔ **A advertência da espec (os `2,6×`–`6,1×` da colisão do tecido) NÃO
transferiu**, e é por isso que ela mandava medir.

⭐⭐⭐ **Mas a PRIMEIRA medição devolveu `15,423 ms`**, e a sonda isolou-o em dois
números: na mesma malha, um raio que **acerta** custava `0,436 µs` e um que
**erra** custava **`1 021,9 µs`** — `2 343×`. **Eram duas promessas não
cumpridas, uma escondida pela outra:**

1. o `Aabb::ray_slab` **documentava** que um eixo com `NaN` não restringe o
   intervalo, e o código fazia o contrário (`NaN.min(+∞) = +∞` ⇒ `t0 = ∞`, que é
   uma **rejeição**);
2. o `Octree::ray_visit_leaves` empilhava a **RAIZ** sem a testar, e a poda dos
   filhos (`t0 <= best`) é inerte enquanto nada foi acertado (`∞ <= ∞` é
   **verdade**) ⇒ *num raio que erra, cada folha era visitada*.

⛔⛔ **O (2) era a REDE que escondia o (1)**, e a prova é que assim que a raiz
passou a acreditar no slab, o gate
`an_axis_aligned_ray_grazing_a_box_plane_is_not_lost_to_nan` — que existe há
muito a defender exactamente aquele caso — **reprovou**. *Uma promessa de doc que
o código não cumpre pode viver anos debaixo de uma rede noutro ficheiro.*

⚠️⚠️ **Quem o expôs foi este pincel** (um raio por vértice, e metade deles erra de
propósito), **mas quem já pagava era o PICK**: o cursor fora da peça é um raio
que erra. *Um defeito de custo que só aparece quando alguém erra de propósito
pode viver anos numa crate que toda a gente usa.*

### §28.6 — Os censos, e o que cada um custou

- **DOIS censos passaram a MEDIR o verbo em vez de o excluir** (`o dab não fez
  nada em canal nenhum`): a cura é a irmã da `referencia_sintetica` — um alvo
  sintético que **ENVOLVE** a peça, para que qualquer raio acerte. *Um censo que
  exclui um verbo deixa de o testar*, e aqui custaria duas propriedades reais.
- **O gate de costura apanhou a caixa PINTADA E MORTA SOB O DEDO na primeira
  corrida** — a forma exacta que o esfregão pagou (§23).
- **O teclado:** o verbo entra na lista dos que shipam **só com chip**, pela razão
  dos três irmãos — ele é **inerte na configuração de fábrica**. ⚠️ Mas ele **não**
  é da família da manutenção da malha: é de FORMA, e *se um dia a cena de fábrica
  tiver duas peças é este o verbo a reconsiderar primeiro*.
- **Três tectos de LOC curados por CORTE**, nunca por isenção: `stroke.rs`
  (`698 → 700`, com a prosa a encolher e o assunto a apontar para o módulo),
  `stroke_target.rs` (`692 → 761 → 700`, com o alvo a migrar para o irmão) e
  `panel-sculpt3d/rows.rs` (`601 → 594`).

### §28.7 — ⏳ O que fica ABERTO

| # | item | estado |
|---|---|---|
| 1 | a **composição por dab** sobre um traço (`13` fixturas, `2,8e-3` a `2,6e-1`) | ⏳ medido, com a tabela e a catraca |
| 2 | as **6** fixturas que precisam da geometria do alvo | ⏳ **acto do E** — uma emenda ao emissor |
| 3 | a **folga simétrica** (espec §10.3) | ⏳ **decisão do dono** — a lei alternativa está escrita, medida e `#[cfg(test)]` |
| 4 | a regra *«o alvo ESCONDIDO não conta»* (§6.1) | ⏳ o `SceneObject` não tem visibilidade; ela vive no mundo ECS e o pen-down não lhe chega |
| 5 | o `tip_roundness` da vassoura | ⏳ **pré-existente**, zero adições neste diff — dívida da linha dona |

⚠️ **`stroke.rs` está EXACTAMENTE no tecto (`700` de `700`)**, com zero folga: o
próximo campo obriga um corte por assunto, e o candidato natural é o grupo *o que
o pen-down fotografa* (`persistent_base` · `pecas_da_cena` · `pose_activa` ·
`reference`), que pede uma sub-struct e toca ~20 sítios.

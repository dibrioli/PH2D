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

## §12 — Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=42 cargo run -p ph2d-host-desktop --profile smoke
```

O roteiro dos 6 passos é impresso pelo próprio app ao abrir. ⚠️ Rode também
**uma vez sem a env var** — é a metade que prova a inércia.

# HANDOFF — `line/quadextract`: a régua POR PONTA, e o BICO que encosta no ápice (2026-09-04)

> **Estado:** a linha entrega a resposta ao report de 04/09 (*«muitas pontas boas no mesmo mesh
> e apenas uma ruim — como tornar uniforme?»*): a régua que diz **qual**, o mecanismo medido, e
> a cura.
> ⛔ Nada foi integrado nem pushado (`CLAUDE.md` §0.7).
> Worktree: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract` · ramo `line/quadextract`.
> Mecanismo completo com as tabelas: [plano §108](../quad-remesh/PLANO_a_graduacao_da_ponta.md).
> O que veio antes: [a calota e a gravata](HANDOFF_line_quadextract_A_CALOTA_E_A_GRAVATA_2026-09-03.md).

## §1 — O que mudou, numa frase

As três réguas da ponta passam a ter uma **tabela por ápice** ([`ph2d_quadfill::tip_rows`], de
que as duas agregadas são **dobras**), e o acabamento ganha um **remate**
([`ph2d_quadfill::snap_tips`]): o vértice mais próximo de cada bico afiado encosta **no ápice da
escultura** quando está a menos de meia célula dele. Na peça do dono as cinco pontas passam a
ler `gap 0,00` — antes `0,08`–`0,20` conforme a ponta, e `0,47` na realização que ele exportou.

## §2 — Como reproduzir

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract
cargo test -p ph2d-quadfill --lib tip_rows:: tip_snap::                 # 9 gates, < 1 s
cargo test -p ph2d-quadfill --test pontas_do_dono -- --nocapture        # o portão dos dois lados
# a tabela por ponta de QUALQUER par de malhas, sem correr o botão:
cargo run -p ph2d-quadfill --release --example pontas -- \
    --entrada ~/Downloads/sculpt-pre.obj ~/Downloads/sculpt003.obj
# e o botão de ponta a ponta (imprime a tabela em F1 e na SAIDA):
CARGO_INCREMENTAL=0 cargo test -p ph2d-host-desktop --release --bins --no-run
env PH2D_PIECE=/home/enio/Downloads/_base_sculpt.obj PH2D_RECENTER=1 PH2D_DETAIL=1.0 PH2D_ADAPT=1.0 \
    ./target/release/deps/ph2d_host_desktop-<hash> \
    sculpt3d::history::undo::photo_probes::button::the_artists_piece_through_the_button \
    --ignored --nocapture --exact --test-threads=1
```

⚠️ **O binário é o que ESTA linha imprime** no fim do `--no-run`; nunca `ls -t`.

⭐ **O smoke fica COMPILADO** (`cargo build -p ph2d-host-desktop --release`, corrido `2 ×`):

```text
    Finished `release` profile [optimized] target(s) in 3m 14s     # 1.ª
    Finished `release` profile [optimized] target(s) in 0.34s      # 2.ª — nada a fazer
```

⇒ o `cargo run -p ph2d-host-desktop --release` do dono abre sem compilar.

## §3 — A régua: uma linha por ponta

⛔ **Nenhuma das três réguas sabia dizer QUAL.** A [`tip_deviation`] devolve o pior `p50`
**entre** as pontas, a [`tip_density`] a mediana e o pior, a [`tip_survival`] um extremo global
— *as três já mediam ponta a ponta e deitavam fora o índice antes de devolver*. É a **quarta**
vez que esta linha paga a mesma forma (o `edge_max` global, o `χ` cego à almofada, a `ENTREGA`
cega à ponta que engrossou).

`tip_rows(entrada, saída, unit)` devolve, por ápice: `gap` (a amputação), `grade` (a grade do
bico), `cone` (a forma), `dev` (`p50`/`p90`/máximo), `blind` (a ponta comida por inteiro, cujo
número é um **piso**) e o **pólo** (irregulares a `≤ 2 h` e a valência do bico, a coluna do
mecanismo do §101). ⚠️ As duas agregadas são dobras dela e **os números do produto não mudam**.

## §4 — O que ela achou nas malhas do PRÓPRIO dono

`sculpt-pre.obj` → as duas saídas que ele exportou (`h = 0,0167`):

| ápice | raio | `sculpt003` (03/09) | `sculpt-Pos-Remesh` (a anterior) |
|---|---|---|---|
| ⛔ **`8042`** | **`1,808`** | `gap` **`0,47`** · grade `0,79` | ⛔⛔ **comida** (`gap ≥ 3` · grade `3,49`) |
| `9218` | `1,070` | `0,27` · `0,66` | `0,15` · `0,66` |
| `12279` | `1,052` | `0,08` · `0,64` | `0,26` · `0,40` |
| `11108` | `0,924` | `0,08` · `0,76` | `0,05` · `0,62` |
| `15622` | `0,846` | `0,15` · `0,73` | `0,10` · `0,85` |

⭐⭐⭐ **A ponta ruim é sempre a MAIS LONGA**, e é a pior em todas as colunas nas duas saídas. As
agregadas liam `0 de 5` na primeira: *o relatório dizia «perfeito» sobre a ponta da foto.*

## §5 — ⭐⭐⭐ O mecanismo: o defeito nasce no ACABAMENTO, e é FASE, não célula

A **mesma** extracção (`21 914` faces), acabada de duas maneiras (`PH2D_CANDIDATE_DUMP`):

| acabamento | `gap` no `8042` | grade no bico | as outras quatro |
|---|---|---|---|
| ⭐ cerca de viagem apertada (a entregue) | `0,18` | `0,46` | `0,08`–`0,20` |
| ⛔ cerca larga | **`0,51`** | `0,88` | `0,09`–`0,29` |

⭐⭐ **A grade do bico está dentro da barra nas duas** — a calota de 03/09 deu-lhe resolução. O
que muda é onde a última volta da grade **pára**, e a causa tem nome: *o acabamento pousa cada
vértice na escultura, e a projecção ao ponto mais próximo de uma superfície **nunca escolhe o
ápice*** — o bico é um ponto de medida nula e o pé da perpendicular cai no **flanco**. Cada
ronda embota a ponta, e *quanto* depende da cerca que aquela saída calhou de usar. **É por isso
que uma ponta sai boa e a vizinha não.**

## §6 — A cura, e as três cercas que a tornam honesta

[`snap_tips`] corre no fim das **três** saídas do acabamento (gate:
`as_tres_saidas_do_acabamento_desfazem_o_avesso`, que agora conta as três curas):

1. **`gap ≤ TIP_GAP_MAX`** (meia célula) — ⛔ acima da barra o que falta é **célula**, e mudar um
   vértice esconderia do selector um defeito que ele tem de ver;
2. **viagem `≤ 1` célula** — o `gap` mede ponto→FACE e o vértice mais próximo pode estar mais
   longe que ele;
3. **o censo GLOBAL não pode subir** (faces `> 60°` e faces do avesso), **uma ponta de cada
   vez** — aceitar as cinco de uma vez faria uma má vetar quatro boas.

⚠️ **A cerca 3 não é uma precaução: é a diferença entre duas fixturas.** Num bico que é uma
**lasca** (leque de `8` a `15°`) devolver o ápice ao sítio leva as faces péssimas de `8` para
`16` e o remate **recusa**; na malha que o produto entrega o censo fica **intacto** (`>60`
`8 → 8`, avesso `2 → 2` nas cinco pontas). Os dois casos são gate.

## §6-bis — A tabela que autorizou ligar (peça do dono, `Detail 1` · `Curv 1`)

| | sem o remate | ⭐ com o remate |
|---|---|---|
| pior `gap` entre as `5` pontas | `0,18` | ⭐ **`0,00`** (as cinco) |
| grade no bico (pior) | `0,74` | `0,83` (barra `1,0`) |
| aspecto p50 / p99 | `1,15` / `1,88` | ⭐ **`1,09` / `1,57`** |
| enviesamento p50 / p99 | `4,3°` / `29,6°` | ⭐ **`3,0°` / `20,0°`** |
| faces `> 60°` | `8` | ⭐ **`4`** |
| **tentativas do botão** | `9` | ⭐⭐ **`4`** |
| **relógio** | `337 s` | ⭐⭐⭐ **`123 s`** |
| topologia | `χ 2` · `0` bordo · `0` não-manifold · `21 914` quads | **idêntica** |

⭐⭐⭐ **O relógio e a forma são consequência.** A cascata pára quando uma candidata satisfaz as
réguas: com a fase fechada, a **segunda** tentativa já as satisfaz. Antes eram precisas **nove**
para achar uma que ganhava por `0,18` contra `0,51` no bico — e essa pagava a diferença em
forma, porque a cerca de viagem que protege a ponta é a mesma que impede a relaxação de
trabalhar. *O remate tira a ponta da disputa, e o selector passa a escolher pela forma.*

⚠️ `PH2D_TIP_SNAP=0` bissecta (a porta mora no acabamento e alcança a bancada de propósito: a
comparação precisa das duas metades a correr a MESMA cadeia).

⭐ **E o CONTROLO na outra peça** (`sculpt_antes`, a da agulha): as **três** pontas sãs passam a
`gap 0,00` e a agulha `4849` **fica acusada** a `2,57` — o remate recusa-se por desenho, que é a
cerca 1 a funcionar no produto. Topologia `χ 2` · `0` bordo · `0` não-manifold · **`>60 = 0`**,
`21 512` quads.

## §7 — ⛔ Recusas MEDIDAS desta wave

- **`PH2D_TIP_ALIGN=5` COM a calota** — a célula `(1,1)` que o [§103](../quad-remesh/PLANO_a_graduacao_da_ponta.md)
  encomendou. Corrida: `3` de `7` candidatas com furos (`40` e `26` arestas de bordo), **todas**
  as sete com `p90 3,00` numa ponta, `2`–`5` de `5` com a grade grossa. ⇒ o reforço fica
  **instrumento**, não cura.
- **Rematar acima da barra** (uma ponta comida) — seria trocar um bico chato por um **espeto de
  uma célula**, e apagar do selector um defeito de células. Gate.
- **Rematar um bico-lasca** — ver §6, cerca 3. Gate.

## §8 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **`tip_deviation` e `tip_density` mudaram de corpo e NÃO de resultado** — elas são dobras da
   tabela, e há gate que dobra a tabela à mão e exige o mesmo número.
2. **O remate não é o *«puxar o vértice mais avançado»* refutado em 31/08.** Aquele deslocava
   `0,6198` de mundo (**`23` células**) sobre um bico cuja grade estava a `3,85 ×` o alvo, e
   levava o aspecto a `12,11`. Este desloca **meia célula** sobre uma grade que já tem
   resolução, e o censo global tem de ficar igual ou melhor.
3. **A `unit` do remate é a MEDIANA da malha, não o alvo do slider** — quem acaba não conhece o
   slider (a `ph2d-quadchain` e os gates chamam a porta sem ele).
4. **A coluna do `pólo` na tabela não é a régua de nada, ainda** — ela é o instrumento do §101 e
   não discrimina as pontas boas das más nesta peça (a `8042` lê `0,4` como duas pontas sãs).
5. **A fixtura dos gates tem DOIS espinhos de propósito** — com um só, *«a tabela diz qual»* é
   vazio: a única linha é sempre a acusada.

## §9 — O que fica ABERTO, com endereço

- ⏳ A **`sculpt_antes`** (a agulha) continua com `1/4` amputada — ali o que falta é **célula**, e
  o remate recusa-se por desenho.
- ⏳ O **sorteio dos últimos bits** continua (§104): a mesma escultura noutra escala dá outra
  realização, e o remate fecha a fase mas não muda a candidata que vence.
- ⏳ O `pólo` medido (`0,4` no bico da ponta mais longa) diz que a grade **atravessa** o bico em
  vez de fechar nele; a wave que o cura é a do §101–§103 e não esta.

## §10 — O diff

| ficheiro | o quê |
|---|---|
| `crates/ph2d-quadfill/src/tip_rows.rs` + `_tests.rs` | a tabela por ponta e 4 gates |
| `crates/ph2d-quadfill/src/tip_snap.rs` + `_tests.rs` | o remate e 5 gates |
| `crates/ph2d-quadfill/src/tips.rs` | as duas agregadas viram dobras |
| `crates/ph2d-quadfill/src/finish_extract.rs` | o remate nas três saídas + `FinishReport::snapped` |
| `crates/ph2d-quadfill/src/untangle_tests.rs` | o censo das três saídas passa a contar o remate |
| `crates/ph2d-quadfill/examples/pontas.rs` | o instrumento (`--recentrar`, `--unit`, `--rematar`) |
| `shells/desktop/src/sculpt3d_photo_button.rs` | a tabela na sonda, em `F1` e na `SAIDA` |
| `shells/desktop/src/sculpt3d_retopo_one.rs` | o acabamento diz o que fez (`untangled` · `snapped`) |

## §11 — ⭐⭐⭐ A SEGUNDA METADE DO DIA: o knob que abria em ZERO (plano §109)

**O smoke do §108 voltou aprovado** (*«resultado do remesh muito melhor, sem problemas
graves»*) com **uma** queixa: *«a ponta problemática ainda não tem a densidade de faces adequada
como as outras»*, com foto e seta.

⛔ **A régua do §108 não a via, e a razão é o RAIO:** ela mede as três células do bico, e ali as
cinco pontas daquela peça são iguais (`gap 0,00`, aspecto `1,08`–`1,39`). A foto é sobre o
espinho **inteiro**. ⇒ [`TipRow::shaft`] (a faixa `3`–`12 h`), a porta [`tip_band`] e o
`--bandas` do exemplo, que varre `0–3`, `3–6`, `6–12`, `12–24 h` **e conta o anel** — quantas
faces dão a volta ao espinho. *É o anel que o olho lê como densidade: um espinho fino com quads
pequenos ainda pode ter cinco faces em volta.*

⭐⭐⭐ **A causa é o ponto do painel:** o `Follow Curvature` nasce em `0`, e com `0` **dois dos
cinco espinhos saem com `3` a `5` faces dando a volta** (contra `17`–`58` com `1`), voltam `2`
pontas amputadas e `4` com a grade grossa. Com `1` a peça dele fica melhor em **todas** as
colunas e **mais rápida** (`150 s → 123 s`). ⇒ o knob passa a nascer no máximo, e o botão deixa
de guardar uma segunda cópia do default.

⚠️ **A troca está dita na segunda peça** (`sculpt_antes`): o `1` paga enviesamento mediano
`2,9° → 4,2°` (dentro da banda do oráculo) e `+50 %` de relógio, e compra `>60` de `5` para
`0` e a grade do bico abaixo da barra em todas as pontas.

⛔ **A recusa de 28/08 (*«pede-se 400 % e a saída move-se 7 %»*) já não responde** — desde ela a
fase zero passou a graduar com renormalização, ganhou a calota e o acabamento ganhou o remate.
*Uma recusa medida responde UMA pergunta.*

Gate: `the_curvature_knob_opens_where_it_was_measured` (o valor **e** a ausência do literal) +
`a_faixa_do_corpo_nao_conta_o_bico`.

## §12 — ⭐⭐⭐ A TERCEIRA metade do dia: o MESH na Hierarquia, e o `Delete` que não dizia nada

**Os dois reports:** *«corrija o deletar com a tecla del»* e *«implemente o mesh na Hierarchy
pois ele ainda não aparece lá. Implemente as funções na hierarquia para o mesh como del,
duplicate, etc»*.

### §12.1 — A ponte: uma peça ⟺ uma entidade

[`ph2d_ecs::Sculpt3dPieceRef`] (componente novo, registado) + `sculpt3d::entities` — a mesma
forma do `flip_entities` e do `vec_entities`. Com ela a peça é uma `Entity` como outra
qualquer, e **`Name`, `ChildOf`, `RootOrder`, `Visibility`, `Locked` e os verbos de linha valem
sem uma lei nova**: apagar a linha apaga a peça, renomear renomeia, agrupar agrupa.

⭐⭐⭐ **E o estado guardado é um CONJUNTO DE IDS, não um mapa de bits** — a decisão que separa
esta ponte das irmãs. Elas guardam `id → bits` e precisam de um `rebuild_map` a seguir a cada
restore; aqui isso seria **apagar a escultura a cada Ctrl+Z** (o undo respawna com bits novos, e
o plano leria *«a entidade sumiu»*). ⇒ o mundo é lido a cada quadro, e o que se guarda é só
*«que peças já tiveram linha»* — o único facto que o mundo não sabe. Gate:
`depois_de_um_restore_com_bits_novos_o_plano_e_vazio`.

### §12.2 — Os verbos

| verbo | como | nota |
|---|---|---|
| **Delete** | a entidade some ⇒ o plano remove a peça | de graça, pela direcção 1 da ponte |
| **Duplicate** | a cópia profunda **larga** o `ref` (`instance_docs::DROPPED`) ⇒ um PEDIDO duplica a peça no quadro seguinte e põe o `ref` novo na cópia | ⛔ copiar o id daria duas entidades sobre a mesma peça |
| **Rename** | escreve o `Name` da entidade | a peça não tem nome próprio: o nome **é** o da linha |
| **Escolher a linha** | põe a mão na peça (`scene.active`) | ⚠️ **só na MUDANÇA** da selecção — `active` tem dois escritores (a linha e o `aim` do pen-down), e aplicar a cada quadro tornaria impossível esculpir noutra peça |

### §12.3 — O `Delete` que morria em silêncio

⛔ A tecla morria num de **três** guardas (`sculpt3d_keys_dead_reason`) e **nenhum deles dizia
nada** — nem ao artista, nem a quem foi diagnosticar. Agora:

1. a recusa é **impressa**, com o guarda que a causou (é a lei que o `Delete` lá dentro já
   escrevia, aplicada ao guarda);
2. o `sculpt3d_keys_live` passa a **derivar** da razão — ⛔ duas respostas à mesma pergunta
   divergem no dia em que alguém acrescenta um quarto guarda a só uma delas;
3. **sobre um painel o `Delete` não é da escultura**: este teclado corre antes de toda a cadeia,
   e sem a cerca de área ele comia o `Delete` da Hierarquia (onde o mesh agora tem linha), do
   Flip, da timeline e do Painter sempre que houvesse barro na tela.

A decisão saiu para uma porta **pura** (`sculpt3d::keys_delete::claim_delete`) — forçada pelo
tecto de LOC (`611 / 600`) e melhor por isso: ela ganhou **três gates a sério** em vez de um
censo textual.

### §12.4 — ⚠️ O que uma leitura rápida entende ao contrário

1. **O `Sculpt3dPieceRef` é `owned_document`** e por isso a cópia profunda **não** o leva — a
   duplicação da peça é feita pela ponte, não pela cópia.
2. **A `seen` é REDERIVADA do mundo** a cada sincronia, e não incrementada: um contador que só
   cresce era o que faria a segunda leitura discordar da primeira.
3. **O `Delete` sobre um painel devolve `false` sem tocar na cena** — não é um no-op silencioso,
   é a cadeia a seguir para quem de direito.

## §13 — ⛔⛔ O WIREFRAME: o que a foto mostra NÃO é geometria escondida (medido)

**O report:** *«a opção de visualizar wireframes permite que a malha que deveria estar oculta
apareça, confundindo a visualização»* (04/09, foto com duas setas na banda rasante).

### §13.1 — A medição que refuta a hipótese óbvia

Instrumento novo: `PH2D_MESH=<obj>` na sonda `probe_wire_continuity`
(`a_peca_do_artista_vaza_wireframe`), **seis vistas incluindo close-up**, sobre a peça do
próprio dono (`sculpt003.obj`, `21 928` faces, **fechada**):

| vista | tinta VAZADA (onde nenhuma aresta da FRENTE passa) |
|---|---|
| frente · 3/4 · perfil · de cima | **`0,00 %`** |
| perto 3/4 · perto perfil | **`0,01 %`** (`2`–`3` px) |

⇒ **o descarte das arestas de costas (`obj.wire_cull`) faz o seu trabalho** — a malha de trás
não atravessa. ⛔ *A hipótese que eu tinha ao ler a foto estava errada, e foi a medição que o
disse.*

### §13.2 — ⚠️ E DUAS réguas minhas mediram a coisa errada antes de acertar

1. **A câmera da sonda vinha de dentro do `render`**, então rodar a vista comparava a MÁSCARA de
   uma câmera com a TINTA de outra: li `33`–`51 %` de fuga onde a verdade é `0 %`. Curado —
   a câmera passa a vir do chamador (`render_at`).
2. **A régua de cobertura contava PRESENÇA** (o pixel mudou um bit) e por isso **não via um
   desvanecimento**. Curada para a **FORÇA** (quanto a tinta escurece o pixel).

### §13.3 — O que a régua honesta diz

Com a força: a tinta do wireframe escurece **`10,8 %`** do miolo da peça e **`17,8 %`** da borda
(razão `1,65`) — e a `de cima`, `13,8 %` / `26,0 %`. *Onde as células se projectam com menos de
um pixel, o wireframe deixa de ser linhas e vira preenchimento* — e uma mancha cheia lê-se como
«a malha de trás».

### §13.4 — ⛔ A cura ÓBVIA foi construída, medida e REVERTIDA

Desvanecer a aresta pelo ângulo (`smoothstep` sobre `|facing|`) funciona na régua nova:

| `LO`–`HI` | miolo | borda | razão |
|---|---|---|---|
| *(sem)* | `10,8 %` | `17,8 %` | `1,65` |
| `0,25`–`0,65` | `7,9 %` | **`8,5 %`** | **`1,08`** |

⛔ **E reprova o gate `the_grazing_edges_are_not_eaten_by_their_own_surface`**: `24,4 %` contra
a barra de `78 %`. Esse gate defende **o report de 12/08 do mesmo dono** (*«os wireframes saem
todos cortados»*) — o desvanecimento por ângulo cura este report **reabrindo aquele**.

⇒ *A variável certa não é o ÂNGULO, é a DENSIDADE por pixel*: uma aresta rasante sozinha numa
malha esparsa tem de aparecer inteira; cinquenta arestas no mesmo pixel não podem saturar. As
duas leituras só se separam pelo **tamanho projectado da célula**, que hoje não existe no
shader.

### §13.5 — ⏳ A wave que resolve, com endereço

Um atributo por-vértice com a **aresta incidente média** (a CPU já a calcula em
`ph2d_quadfill::tip_rows`), e o alfa da linha a desvanecer quando
`célula_mundo / mundo_por_pixel < ~2 px`. Isso separa os dois reports por construção: a aresta
rasante isolada mede muitos pixels por célula e fica; a banda saturada mede menos de um e
desvanece.

⚠️ **O que fica no repo desta investigação:** a porta `PH2D_MESH` na sonda, a câmera vinda de
fora, e a régua da FORÇA — as três são ganho puro e ficam, com a hipótese refutada escrita ao
lado.

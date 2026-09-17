# 113 — CICLO 8: FONTES & DADOS, de onde vêm as coisas

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, e o **tutorial é o smoke**.
> **Premissa do tutorial:** *«De onde vêm as coisas»*.
>
> ⚠️ Este doc é o do CICLO. O que ele mede vive nas sondas de
> [`motion_fontes_probe.rs`](../../crates/ph2d-app-motion/src/motion_fontes_probe.rs) e
> [`motion_bridge_fontes_costura.rs`](../../crates/ph2d-app-motion/src/motion_bridge_fontes_costura.rs).

---

## §1 — O grupo, DERIVADO da paleta

A sonda `the_source_palette_census` lista cada tipo oferecido com a categoria e o nome do cartão.
A categoria **`Source`** tem **20** tipos, e **catorze têm dono noutro ciclo**:

| ciclo | os da categoria `Source` que ele já contou |
|---|---|
| 1 (arranjo) | `grid` · `scatter` · `distribute_radial` · `fibonacci` · `lattice` · `voronoi` · `distribute_curve` |
| 5 (simulação) | `sim.spawn` |
| 6 (valor) | `value.pattern` |
| 9 (rig & corpos moles) | `boids` · `soft_body` · `verlet_rope` · `wave` · `rig.skeleton` |

Os seis que sobram são o grupo — e é o mesmo da linha do doc 103 §5 (uma **verificação**, não a
fonte):

| família | nós | cartão |
|---|---|---|
| `source.*` | 5 — `lsystem` · `object` · `shape` · `table` · `text` | `L-System` · `Object` · `Shape` · `Table` · `Text` |
| `motion.*` | 1 — `emitter` | `Emitter` |

⚠️ **O sub-grupo `source.*` é o que o artista vê** como *«as fontes que partem de uma coisa que EU
fiz»* — um desenho, um objecto da cena, um texto, um ficheiro, uma gramática. O emissor entra pela
pergunta do tutorial (*de onde vêm as partículas*), que o ciclo 5 não fez. Gate
`the_source_group_is_derived_and_not_empty` (piso de `6`, as duas metades).

---

## §2 — O RETRATO (sondas `audit_the_source_group` e `what_the_source_card_shows`, 2026-09-16)

```text
  nó                   | params | no cartão | device | portas | efeito
  ---------------------|--------|-----------|--------|--------|---------
  motion.emitter       |     24 |        15 |  sim   | 0->1   | Temporal
  source.lsystem       |     31 |        15 |  NAO   | 0->1   | Pure
  source.object        |      2 |         3 |  NAO   | 0->1   | Pure
  source.shape         |     42 |        10 |  NAO   | 0->1   | Pure
  source.table         |      1 |         2 |  NAO   | 0->1   | Pure
  source.text          |      6 |         8 |  NAO   | 0->1   | Pure
```

**O que ele diz, sem uma linha de código:**

1. ⛔ **Cinco de seis fora do dispositivo** — as cinco `source.*`. O emissor está lá.
2. ✅ **O vocabulário está LIMPO** — as três perguntas das sondas de vocabulário devolvem zero linhas;
   as chaves partilhadas (`angle`, `seed`, `size`) pintam o mesmo rótulo em todos.
3. ⚠️ **As diferenças `params`/`no cartão` são, na maioria, GATES DE MODO e cores** — o `source.shape`
   mostra `10` de `42` porque cada família de forma tem os seus controlos (`sides` num polígono,
   `star_depth` numa estrela…) e os oito `r/g/b/a` são duas linhas; o `source.lsystem` mostra `15`
   de `31`, com duas das quatro secções fechadas (`Leaves` e `Lean & Look`) (a lei de 31/08: *nenhum molde mostra um knob que a gramática
   dele não sabe ler*). ⏳ A confirmar pelo censo do alcance (W2).
4. ⚠️ **O cartão do `Shape` abre com `Own Fill`**, antes de `Shape` — a primeira linha de um cartão
   de fonte devia ser *qual forma*. ⏳ W2.
5. ✅ **A conferência já comparou os seis com as referências**: a folha
   [14 (SOURCE)](89_conferencia/14_source.md) e a [01](89_conferencia/01_distribuicao_emissao.md)
   (o emissor) estão a zero, e o `source.lsystem` tem auditoria própria
   ([doc 96](96_auditoria_do_lsystem_2026-08-31.md)). ⇒ *o poder que falta* (W3) parte do que elas
   deixaram RECUSADO, não de uma leitura nova.

---

## §3 — ⛔⛔ O ACHADO QUE DECIDE O CICLO: a costura cai NA FONTE — e ela ENVIA por quadro

A lei 1 do protocolo, com a sonda `probe_does_a_source_chain_stay_on_the_device` (cada nó à cabeça de
`X → scale → output`):

```text
  X -> scale -> output   | onde corre  | stages | costura
  motion.grid (controlo) | dispositivo |      3 | —
  motion.emitter         | dispositivo |      3 | —
  source.lsystem         | ⛔ CPU       |      2 | source.lsystem:0
  source.object          | ⛔ CPU       |      2 | source.object:0
  source.shape           | ⛔ CPU       |      2 | source.shape:0
  source.table           | ⛔ CPU       |      2 | source.table:0
  source.text            | ⛔ CPU       |      2 | source.text:0
```

⭐⭐ **É o espelho do ciclo 7, e o preço é OUTRO.** Lá a aparência era o ÚLTIMO nó e puxava a cadeia
inteira para a CPU; aqui a fonte é o PRIMEIRO, a costura cai nela e **o resto fica na placa** (`2`
estágios). O que se paga por quadro é:

1. **cozer a fonte na CPU** — barato quando ela não muda (as cinco são `Pure`, e o memo responde);
2. **ENVIAR o stream dela para a placa** — ⚠️ **SEMPRE**: `GpuCook::cook` faz *«one upload per
   boundary node»* a cada chamada, e nada guarda o envio do quadro anterior. Um texto parado, uma
   tabela, uma forma — o mesmo stream é copiado para a placa sessenta vezes por segundo.

⚠️ **E as cinco não pesam igual:** o `Object` e o `Shape` emitem **uma** linha (o custo deles está no
DESENHO e no carimbo — o item 10 do doc 103 §5.1, nas suas duas metades); a `Table`, o `Text` e o
`L-System` emitem **uma linha por elemento** (linhas do ficheiro, glifos, ramos), e é nesses que o
envio escala. ⚠️ Os «`0` elementos» da sonda acima são as fontes SEM conteúdo (sem ficheiro, sem
texto, sem objecto publicado) — o preço mede-se com conteúdo, na §3.1.

### §3.1 — O preço, medido pela ponte do produto

Sonda `measure_the_source_seam`: a MESMA cadeia (`fonte → scale → output`) pela **`cook_gpu` do
app**, com o `publish_all` das membranas antes, e a `motion.grid` do mesmo tamanho como **controlo**
(a cadeia inteira na placa). ⚠️ **Os dois relógios separados** — publicar e cozer+enviar — porque são
duas repetições diferentes de trabalho sobre a mesma coisa parada, e um número só não diz qual.

⚠️ **A máquina não estava calma** (`load 20,5`; outras linhas a compilar), então **o que se cita é a
RAZÃO contra o controlo**, não o absoluto — e ela é o achado:

```text
  ANTES da cura (load 20,5)                    │ publicar │ cozer+enviar │ quadro │ vs. grade
  grade       10 000 (a cadeia toda na placa)  │    0,00  │        0,07  │  0,07  │   —
  tabela      10 000                           │    0,06  │        0,09  │  0,15  │  2,1×
  grade      100 000                           │    0,00  │        0,18  │  0,18  │   —
  tabela     100 000                           │    0,60  │        0,38  │  0,98  │  5,4×
  grade    1 000 000                           │    0,00  │        1,36  │  1,36  │   —
  tabela   1 000 000                           │    6,05  │        3,57  │  9,62  │  7,1×
```

⇒ **a costura de um milhão de linhas PARADAS custava `7×` a mesma contagem no dispositivo**, e a
maior metade nem sequer era o envio: eram `6,05 ms` a **republicar** a tabela — o `set_external`
percorre o conteúdo para derivar a revisão, e a membrana republica tudo a cada quadro.

---

## §6 — ✅ W1: as duas repetições morrem pela MESMA régua

*Uma coisa que não mudou partilha as alocações com a versão de ontem* — clonar um `Stream` é um
refcount e escrever substitui a coluna inteira (lei do `ph2d-nodegraph`, com gate desde sempre). ⇒
uma régua só, [`Stream::shares_storage_with`](../../crates/ph2d-nodegraph/src/attr.rs): as mesmas
contagens e **os mesmos ponteiros** em todas as colunas.

⚠️ **É identidade, nunca igualdade**, e o lado em que ela erra é o barato: dois streams com o mesmo
conteúdo em alocações diferentes respondem `false` e pagam o trabalho que já pagavam. ⚠️ **E ela é
segura porque uma coluna nunca é escrita no sítio** (não há `get_mut`) — quem guarda a resposta
guarda o `Stream` ao lado, segurando os `Arc`, e uma alocação que não pode ser libertada não pode
reaparecer noutro sítio com o mesmo endereço.

**Os dois consumidores:**

1. **`Cook::set_external`** — republicar o MESMO stream não o rehasha (a revisão que sairia seria a
   mesma). Isto vale para TODAS as membranas de uma vez (forma · texto · áudio · tabela · L-System),
   sem uma linha em nenhuma delas.
2. **`GpuCook::cook`** — a costura que não mudou reutiliza o envio do quadro anterior
   (`sent_boundaries`). ⚠️ Reutilizar é seguro por uma propriedade que este módulo já declarava no
   cabeçalho — *cada kernel escreve buffers FRESCOS* —, e o gate prova-a com dois quadros seguidos.

### O que a cura comprou (a MESMA sonda, com a máquina ainda PIOR: `load 30,3`)

```text
  DEPOIS                                       │ publicar │ cozer+enviar │ quadro │ vs. grade
  grade    1 000 000                           │    0,00  │        1,41  │  1,41  │   —
  tabela   1 000 000                           │    0,01  │        1,61  │  1,62  │  1,15×
  tabela     100 000                           │    0,00  │        0,20  │  0,20  │  0,95×
  tabela      10 000                           │    0,00  │        0,09  │  0,09  │  0,75×
```

⇒ **`9,62 → 1,62 ms`** a um milhão de linhas, e a fonte de dados passa a custar **o mesmo que a
grelha** (`1,15×`) — a costura deixou de ser uma taxa por quadro e passou a ser um preço por
MUDANÇA.

### Os gates, e por que eles precisam de um CONTADOR

⛔⛔ **O ganho é invisível ao comportamento:** nos dois caminhos a revisão é a mesma e o quadro
desenha o mesmo — um gate que olhasse só o resultado passaria com e sem a cura, e ela evaporaria na
primeira refactoração. ⇒ cada lado ganhou um par de contadores
(`Cook::external_publish_counts` · `GpuCook::boundary_upload_counts`), e os gates leem-nos:

| gate | o que afirma | mutação |
|---|---|---|
| `republishing_the_same_stream_skips_the_hash` | dez quadros parados ⇒ **um** hash; conteúdo novo ⇒ hasha e a revisão muda | — |
| `an_unchanged_boundary_is_uploaded_once_and_draws_the_same_bits` | o 2.º quadro **reutiliza**, o quadro sai **byte a byte igual**, e uma costura diferente volta a ser enviada | desligar a reutilização ⇒ **RED** |
| `shared_storage_is_identity_and_never_equality` | o clone partilha · escrever desfaz (mesmo escrevendo o mesmo conteúdo) · conteúdo igual noutra alocação responde `false` | — |

⚠️ **A metade dos BITS é a que prova a premissa**, não o contador: se algum estágio escrevesse no
buffer que recebeu, o segundo quadro leria o buffer já mexido e a imagem derivaria em silêncio.

---

---

## §4 — ✅ W2 (a primeira metade): o cartão do `Shape` abre pela FORMA

O censo dos cartões (§2) achou **uma** ordem errada no grupo, e o comentário ao lado dela já dizia a
lei que ela violava — *«de que cor é e para que lado aponta são o que se pergunta de uma forma
DEPOIS de escolher qual ela é»* — com a lista a começar em `Own Fill`:

```text
  antes  | Own Fill · Rotation · Stroke Width · Shape · Size · Corner Radius · Sweep · Start · Inner · Collide
  depois | Shape · Size · Rotation · Corner Radius · Sweep · Start · Inner · [Look] Own Fill · Stroke Width · Collide
```

⇒ os dois essenciais (`Shape`, `Size`) abrem a lista, a `Rotation` fica com eles (ela é POSE, não
aparência) e as quatro linhas de aparência (`Own Fill` · `Fill` · `Stroke Width` · `Stroke`) passam a
uma secção **`Look`** — o mesmo molde do ciclo 7. ⚠️ Os outros cinco cartões do grupo já abrem pelo
que produzem (`Object` · `Table File` · `Text` · e as duas secções do `L-System`), e ficam como estão.

### ⛔⛔ E a mudança destapou um VERMELHO PRÉ-EXISTENTE — de uma ORDEM DO DONO

O `the_material_rows_take_their_range_from_the_column_ceiling` (crate da forma) reprovava com
*«o dono pediu o dobro SÓ no salto: 1 contra 1»*: o controlo dele comparava **dois** tectos
(`BOUNCE_MAX > FRICTION_MAX`), e o 8.º report de 2026-09-15 mandou o salto **voltar a `1`**
([doc 111 §8.1](111_o_motor_de_contacto_com_memoria.md)), revertendo a ordem do próprio dono de
13/09. *Um controlo escrito sobre dois valores morre quando um deles muda por decisão de produto.*
⇒ ele passa a exigir tectos **distintos** pela linha do rolamento (`1,5`, medido no ponto em que a
curva satura), que não depende de escolha nenhuma.

⚠️ **Ele estava vermelho na árvore desde 15/09 e nenhum portão desta linha o via** — a crate da
forma não é tocada por um ciclo há semanas, e o fecho de uma linha corre as crates que ela EDITA.

---

## §5 — ✅ W3: o poder que faltava — a VISTA entra no grafo (`source.camera`)

A folha 14 tem o placar a **zero**, e mesmo assim havia um **P1 aberto** — porque ele vivia na §3
(*«espécies de fonte que faltam»*), que é PROSA, e o placar conta **linhas de tabela**:

> *«**CÂMERA / a vista.** Blender GN `Active Camera`/`Camera Info` … sem ela, **orientar para a
> câmera**, **escalar com o zoom**, **distribuir na área visível** e o **culling** são todos
> inexprimíveis. ⚠️ E o mecanismo de publicar já existe a UMA linha de distância … **Exprimível?
> NÃO** (nada publica a câmera). **P1, custo quase zero.»**

⚠️⚠️ ***Uma folha «a zero» pode ter um P1 dentro dela***, e a régua que a lê não o vê: o gate
`every_node_has_a_conference_row_or_is_named_in_the_debt` mede linhas de TABELA pela mesma razão. ⇒ a
cura levou também uma **linha de tabela** para a `source.camera`, e a §3 item 2 ficou marcada.

### O nó

Crate nova [`ph2d-node-source-camera`](../../crates/ph2d-node-source-camera/) — **UMA** instância que
É a vista: `P` (o centro), `size` (a extensão visível, em unidades de mundo) e `zoom` (px de ecrã por
unidade). Zero params, e a ausência é a decisão: *a vista não se autora aqui — ela é o que o artista
já fez com o rato*.

⭐ **A porta é a do cursor**, como a folha previu: o external reservado `$camera`
([`external::CAMERA`](../../crates/ph2d-nodegraph/src/external.rs)), publicado pela mesma membrana e
no mesmo instante do `$cursor` — que por isso deixou de se chamar `publish_cursor` e passou a
`publish_editor_inputs` (*uma função que publica duas coisas com o nome de uma é como a segunda
deixa de ser lembrada*). A janela é a da **CENA** e não a crua (a mesma armadilha que o cursor já
documentava), e uma janela degenerada **não publica** em vez de publicar `NaN`.

### ⭐⭐⭐ E o gate que prova que ele SERVE apanhou um defeito do `motion.drive`

*Um nó que publica e ninguém consegue usar é um controlo morto com cara de feature.* A primeira das
quatro coisas que a folha nomeou — **escalar com o zoom** — é uma CADEIA:

```text
  source.camera → value.attribute(zoom) → motion.drive(Size, Divide, Scale = 1/px)
                             grid ↗                                   ↘ output
```

com a régua certa (o produto `tamanho × zoom`, que é o tamanho em PIXELS: `40` px em qualquer zoom,
e o controlo de que no MUNDO ele quadruplica quando a câmara se afasta `4×`).

⛔⛔ **A metade «e sem vista publicada?» reprovou — e o defeito não era da câmara:** o
`motion.drive` resolvia um campo de valor **VAZIO** pela identidade `0`, e em `Set` isso escreve
`Size = 0` — **a arte desaparece**, sem erro nenhum, nos dois motores. Acontece com a porta
desligada, com uma fonte sem conteúdo publicado e com um `value.attribute` de uma coluna que não
existe.

⇒ **a lei que fica é uma só, nos dois lados:** *um fio sem valor não escreve* (`vals.is_empty()` na
CPU; o `has` do `drive_resolve` no kernel, que é a porta única onde a lei já morava). Provas de
mutação: apagar a guarda da CPU ⇒ **RED**; apagar a do device ⇒ **RED** no gate de paridade novo
(`an_unconnected_value_writes_nothing_on_either_engine`).

⚠️⚠️ **E o gate que devia ter apanhado isto passava pela RAZÃO ERRADA há meses:** o
`an_unconnected_value_leaves_the_channel_untouched` varria só o modo de omissão (`Add`), onde o `0`
inventado **é** o neutro — o canal ficava igual por acidente aritmético. Hoje ele varre os oito
modos e os dois canais, e o `Set` é o que o teria apanhado. *Um controlo que passa pelo motivo
errado é um gate que não existe.*

⚠️ **Fica NOMEADO o que não foi curado:** um `value.math` com um operando ausente continua a lê-lo
como `0` (lei declarada do nó — é o que faz `a + <nada>` ser `a`), então a mesma cadeia com um
`Math` no meio ainda colapsa. Distinguir *«porta desligada»* de *«porta ligada a um campo vazio»*
exige connectividade no `EvalCtx` **e** no plano do dispositivo — wave de substrato, com este caso
como gate de partida. A cadeia que o tutorial ensina é a curta, que não tem esse estado.

---

## §5-bis — A fila do ciclo

1. ✅ **W1 — a costura** (§6) — `9,62 → 1,62 ms` a um milhão de linhas, pela régua «isto é o mesmo armazenamento».
2. 🟡 **W2 — o cartão e o alcance** — o cartão do `Shape` feito (§4); o censo do alcance é gate do catálogo inteiro e está verde.
3. ✅ **W3 — o poder que falta** (§5) — a VISTA no grafo, e a lei *«um fio sem valor não escreve»* que ela destapou.
4. ⏳ **W4 — a MEDIÇÃO.**
5. ⏳ **W5 — a cena e o TUTORIAL** *«De onde vêm as coisas»* — o smoke do dono.

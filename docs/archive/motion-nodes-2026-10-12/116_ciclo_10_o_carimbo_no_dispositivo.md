# ARQUIVO — 116_ciclo_10_o_carimbo_no_dispositivo.md (história, 436 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`116_ciclo_10_o_carimbo_no_dispositivo.md`](../../Motion%20Nodes/116_ciclo_10_o_carimbo_no_dispositivo.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-144,258-463,604-698,882-952` do original.
>
> ⚠️ **A única alteração ao corpo:** 5 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §4.3 — ✅ O RELÓGIO: a escada do encode — e a CURA, que custa ZERO nitidez

⛔⛔ **A 1.ª redacção desta secção media OUTRO PROGRAMA, e o número dela está corrigido em baixo.**
A sonda construía uma `VectorScene::new()` por medição; a shell **reaproveita** a cena de um quadro
para o seguinte (`fase_frame_open.rs`: `vector_scene.reset()`) e o `reset` do Vello limpa os fluxos
**mantendo a capacidade** ⇒ um quadro em regime **não paga o crescimento dos buffers**, e a sonda
pagava-o em toda leitura. *É a mesma família do sucedâneo que a `line/components` registou — «uma
sonda que mede um sucedâneo para sempre mede outro programa» —, e ela mordeu aqui.* A `10⁶` cópias
a diferença é de `93,7` (frio) para `56,2 ms` (quente); a `102 400` é de `10,0` para `5,66`.

⚠️ **E as três sondas de relógio deste ficheiro mediam-se UMAS ÀS OUTRAS**: o `libtest` corre os
testes de um binário em paralelo, e a mesma célula leu `9,79`, `11,89` e `23,39 ms` nas três da
MESMA corrida (`2,4×` que não é lei nenhuma). Hoje há uma **fatia** (`Mutex`) que cada sonda toma —
⛔ a cura não podia ser pedir `--test-threads=1` no cabeçalho, que é a nota que o `CLAUDE.md` §2
mede a morrer.

#### A cura: **a forma é encodada UMA vez e CARIMBADA `N` vezes**

⭐⭐⭐ **A ordem do dono (2026-09-20) foi «manter a nitidez e ir procurar uma cura que não a custe»**,
depois de RECUSAR assar a forma numa imagem acima de um tecto. Ela existe, e é exacta.

⭐ **A lei que a torna possível:** no Vello a pose de uma forma **não entra no caminho** — o
`Scene::fill` escreve a transformação num fluxo, o estilo noutro, o caminho em mais dois e o pincel
noutros dois. ⇒ *os bytes do caminho são os mesmos para todas as cópias*, e o que muda por cópia é
a **pose** e a **cor**. A porta nova
([`ph2d_vector::scene_prepared`](../../../crates/ph2d-vector/src/scene_prepared.rs)) guarda os bytes do
caminho e reproduz o `Scene::fill` passo a passo — `encode_transform` · `encode_fill_style` · o
caminho · `encode_brush` —, só que o terceiro passo é um `extend_from_slice` em vez de um percurso
elemento a elemento.

⭐⭐ **A prova não é esse parágrafo: é o gate.** `o_carimbo_preparado_escreve_os_MESMOS_bytes`
encoda `N` cópias pelas duas rotas — **com uma tinta diferente em cada cópia** — e compara os
**seis** fluxos do Vello mais os dois contadores, em três formas e nas duas regras de
preenchimento. *Um desvio de um byte num fluxo do Vello não devolve erro nenhum: devolve outro
desenho, e num quadro de `102 400` cópias ninguém o vê a olho.* **7 de 7 mutações sangram.**

⇒ **nitidez: ZERO de custo.** Não há bake, não há tecto, não há resolução: é o MESMO vector, com os
MESMOS bytes, encodado mais depressa.

| cópias | `fill` por cópia (o de antes) | carimbo PREPARADO | % de um quadro | razão |
|---:|---:|---:|---:|---:|
| `1 000` | `0,05 ms` | `0,02`–`0,03 ms` | `~0 %` | `~2×` |
| `10 000` | `0,55 ms` | `0,17 ms` | `1 %` | `3,2×` |
| **`102 400`** | **`5,66 ms`** | **`1,76 ms`** | **`34 % → 11 %`** | **`3,2×`** |
| `1 000 000` | `56,2 ms` | `17,4 ms` | `337 % → 104 %` | `3,2×` |

*(`--release`, quadro QUENTE, mínimo de três por linha, duas corridas independentes com `73`–`85 %`
de CPU ociosa; `PH2D_CARIMBO_PREPARADO=0` devolve a coluna da esquerda — as duas rotas são
byte-idênticas, logo a diferença entre as colunas é **só** relógio.)*

⭐⭐⭐ **O custo por cópia é PLANO e passou de `0,055` para `0,017 µs`** sobre um intervalo de
`1000×` ⇒ o encode continua **linear em `N`**, como a
[ADR-0154](../../architecture/decisions/0154-motion-shapes-are-live-gpu-vector-not-baked-tiles.md)
declara — o que mudou foi a constante. ⇒ **o tecto de contagem de uma forma VIVA subiu `3,2×`**: a
um quarto de um quadro de 60 fps cabiam `~76 000` cópias e cabem **`~245 000`**; num quadro inteiro
cabiam `~303 000` e cabem **`~980 000`**.

⛔⛔ **RECUSA MEDIDA — o `Scene::append` de um fragmento NÃO é a cura.** Foi a 1.ª candidata (ela
lia `4,1×` contra a rota de então, e foi ela que abriu a investigação). Contra o carimbo preparado
ela lê `1,10×` a `102 400` e **PERDE** a `10⁶` (`26,7` contra `18,3 ms`) — e um fragmento carrega o
`brush` **ASSADO**, logo só serve cópias da MESMA cor. *Duas razões independentes, e qualquer uma
chega.* A sonda `audit_the_stamp_encode_routes` fica, para a recusa ser uma medição e não uma
opinião.

#### E o mesmo quadro pelas PORTAS DO PRODUTO — o encode era `86 %` do custo de CPU

⭐⭐⭐ **A escada mede o encode SOZINHO; esta mede o quadro do report** (`audit_the_stamp_frame_split`:
a cadeia `grade 320×320 + source.shape(Star) + motion.duplicator`, cozida pelo `pump` e desenhada
pelo `motion_shape_gen::encode`; `12` segmentos por cópia, mínimo de três por coluna, as duas rotas
corridas de seguida):

| rota | cozer | desenho | (pose) | CPU do quadro |
|---|---:|---:|---:|---:|
| `fill` por cópia | `0,94`–`1,04 ms` | **`6,37 ms`** | `0,13 ms` | `44 %` |
| carimbo PREPARADO | `0,64`–`1,36 ms` | **`2,25 ms`** | `0,18 ms` | **`17`–`22 %`** |

⇒ **o quadro passa a menos de metade**, e a cura caiu exactamente na metade que manda. ⚠️ O `cozer`
**não** muda entre as duas rotas (é o mesmo cozimento), e é isso que prova que a diferença é lei e
não máquina.

⛔⛔⛔ **E esta secção só existe com estes números porque uma FIXTURA mentia: toda esta auditoria
mediu um CÍRCULO julgando medir uma estrela.** O `monta` escolhia o `kind` procurando o **rótulo**
`"Star"` na lista de opções do nó — e os `KIND_LABELS` da `source.shape` passaram a ser **chaves de
i18n** (`"node.opts.node_motion_shape.kind_labels.7"`) quando a fronteira dos motores fechou, no
mesmo dia. A procura devolvia `None` e o `if let Some(k)` caía **calado** na forma de omissão.
⭐ **Quem o denunciou foi a contagem de SEGMENTOS que esta sonda passou a imprimir** (`4` — que é um
círculo — onde uma estrela de cinco pontas tem `12`). ⇒ o índice passa a derivar do **próprio enum**
(`ALL_KINDS`, alinhado ao `KIND_LABELS` por gate na crate do nó) e a função **falha alto**; o
`indice_do_quadrado` do `motion_custo_do_quadro_probe` tinha o mesmo defeito e foi curado com ela.
*Um censo que classifica por string tem de provar que a string existe* — e uma tabela sem a
contagem da fixtura ao lado não deixa ninguém ver que ela mudou.

⛔⛔ **E uma hipótese MINHA caiu aqui, medida:** eu escrevera que o que sobra do desenho é o
`instance_pose` (compor base · tamanho · âncora · câmera por instância) e que ele valia ~`40 %`. A
coluna `(pose)` mede-o isolado, pela porta do produto: **`0,13`–`0,19 ms`**, ou seja **`~7 %`**.
*O resto é o encode propriamente dito, e a porta do produto já está no chão da escada* ⇒ **não há
degrau seguinte do lado do CPU**; o que sobra é a RASTERIZAÇÃO.

⚠️ **E o que NENHUMA destas colunas mede é a PLACA.** A lei já estava escrita no
`motion_custo_do_quadro_probe`: *se o encode for barato, o que sobra é a placa, e o que a governa
não é o número de formas — é quantos PIXEIS elas cobrem*. A `102 400` cópias minúsculas o encode
era `44 %` do orçamento de um quadro e passou a `~20 %`; quem quiser o degrau seguinte mede a
rasterização, não o encode.

⛔⛔ **E a W2 tem uma peça que a §5.6 não podia ver, achada a ler o código da ponte:** a marca de
vector vivo é **por TIPO de nó** (`NodeRegistry::register_live_vector_source`, um conjunto de
`NodeTypeId`), e a recusa pergunta *«este grafo contém um nó DESSE TIPO?»*. ⇒ um *bake fallback* não
pode «desligar a marca»: a pergunta da ponte tem de passar de **tipo** para **documento** (*«este
tique desenha geometria viva?»*). *Uma cura que assa a forma e deixa a pergunta onde está entrega
uma cena assada que continua a recusar.*

---

### §5.5 — ✅ A W1(a) FECHOU: o `motion.clone` CORRE NO DISPOSITIVO

O kernel é o molde do §5.3 ponto 1 com uma **translação** no lugar da rotação — *slice-major*
(`copia = i / n`, `linha = i % n`), `write_cp_rows`, `read_P(linha)`, `write_P`, e o sequenciador a
colher todas as outras colunas do template nesses índices, que é exactamente o `replicate` da CPU.

**Três números saem do hospedeiro por `DerivedUniform`, calculados pelas MESMAS funções do `eval`:**

| uniform | a lei | quem mais a lê |
|---|---|---|
| `cl_k` | `copies_within_budget(param_as_count(count), n, RECOMMENDED_MAX_ELEMENTS)` | a `count_law` deste kernel |
| `cl_step_x` / `cl_step_y` | `radial::linear_step(angle, distance)` | o `Placement::of` da CPU |

⛔ **O passo NÃO podia ser trig no dispositivo** (HR-5: a seno parabólica da casa portada seria a
segunda cópia de uma lei, e o `sin` do WGSL é outra curva) — e ele é um número **por NÓ**, logo
derivá-lo não custa um ciclo à placa. ⛔ **E o `k` também não:** reescrito em WGSL ele seria a
segunda resposta a *«quantas cópias?»*, e duas respostas que discordem **desenham um número
diferente de coisas**. ⭐ O `linear_step` nasceu desta wave como **porta com dois leitores** — o
`eval` multiplica-o pelo posto, o uniform entrega-o resolvido.

#### ⛔⛔ E a §5.4 estava INCOMPLETA por UMA variante — quem o disse foi um gate que já existia

A `Count` da CPU vale `total` em toda a linha: *ela não depende do que entrou*, logo o corpo
escreve-a e **nunca a lê**. Ligada com a cruza — a única variante que então tinha as duas metades
de que ela precisa —, o módulo declarava `in_Count`, o corpo não chamava `read_Count`, **a naga
apagava esse buffer do layout derivado** e o bind group do sequenciador ficava com uma entrada a
mais: `create_bind_group` estoura, e **só no tique em que a coluna nasce**.

⭐⭐ **Quem o apanhou foi o `every_registered_kernel_validates_across_the_whole_presence_space`**,
que varre as `2ⁿ` máscaras de presença de todo kernel registado **sem placa nenhuma** — e cuja
mensagem já nomeava o mecanismo inteiro, escrita por outra wave. *Eu tinha considerado este risco e
dispensado-o por raciocínio; o gate mediu-o em sete segundos.*

⇒ **`ColumnAccess::SourceWriteExisting`** (append-only, o 11.º verbo): escreve só se presente ·
porta length-decoupled · **não lê**. As três ligações da família passam a ser

| verbo | lê? | escreve? | porta |
|---|---|---|---|
| `SourceRead` | na fonte | ⛔ nunca | template |
| `SourceReadWriteExisting` | na fonte | só se presente | template |
| **`SourceWriteExisting`** | ⛔ **não** | só se presente | template |

⚠️ **E o predicado mudou de NOME porque a pergunta dele nunca foi sobre LER:** o `column_present`
pergunta *«com que comprimento julgo esta porta?»* ⇒ `is_source_read` → **`is_source_mapped`**.
*Enquanto ele se chamou pelo que dois dos três faziam, o terceiro não cabia no nome sem o tornar
falso.*

⭐⭐ **Duas decisões do codegen deixaram de ser LISTAS e passaram a ser DERIVAÇÕES**, e as duas por
o verbo novo não caber nelas:

1. o braço do `WriteDropped` enumerava `ReadWriteExisting | SourceReadWriteExisting` — *a forma
   exacta que o comentário três linhas acima condena por escrito*; hoje pergunta
   `writes(true) && !writes(false)`, que é **ser condicional**;
2. a **assinatura de presença** tinha o bit preso a `reads()`. Uma escrita condicional sem leitura
   produz módulos diferentes (`WriteBuffer` contra `WriteDropped`) **com o mesmo conjunto de
   ligações declaradas** ⇒ presos à mesma entrada da cache, o wgpu valida um bind group contra o
   layout errado — o crash que o doc daquela função descreve. ⭐ Para todo acesso que já existia a
   troca é **byte-idêntica**: os condicionais de então também liam.

#### A BANCADA: a renumeração não aparece na POSIÇÃO

⛔⛔ `Index`/`Count` são colunas que o desenho não carrega, e a cópia `c` põe as peças exactamente
onde a translação manda **quer o `Index` tenha sido renumerado quer não**. ⇒ a cadeia da bancada
projecta **`Index / Count`** num campo (`value.attribute` × 2 + `value.math ▸ Divide`) e pinta-o com
o `motion.color_ramp`: `t` varre `[0,1)` **uma vez sobre o conjunto multiplicado**, e a TINTA passa
a ser a renumeração à vista. ⭐ O discriminador é o par `(j, j + n)` — o mesmo elemento da fonte em
duas cópias vizinhas —, afirmado no lado da CPU **antes** de se comparar seja o que for.

| caso | pior \|Δpos\| | pior \|Δtint\| |
|---|---|---|
| `count 6` · dist 2 · 0° | `0` | `5,8826e-3` |
| `count 4` · dist 1,25 · 37° | `4,77e-7` | `5,8824e-3` |
| `count 5` · dist 0,9 · −110° · centrado | `0` | `5,8824e-3` |
| **`count 1`** (o cloner em PASSAGEM) | `0` | **`5,8824e-3`** |

⭐⭐ **A última linha é o CONTROLO, e é ela que torna a tabela legível:** com uma cópia só o
multiplicador não multiplica nada e a tinta lê **o mesmo** desvio ⇒ *ele é a quantização da LUT da
rampa* — a `LUT_RESOLUTION` dela é **`256`**, logo a célula mede `1/255 = 3,92e-3` e o canto de uma
parada a cair dentro de uma célula vale `1,5/255 = 5,88e-3`, que é o número medido; tudo isto
pré-existente —, **não** uma deriva do kernel novo. ⛔ E a barra (`1e-2`)
continua a discriminar por duas ordens de grandeza: a avaria que o gate existe para apanhar — a
renumeração a evaporar-se — desloca `t` em `1/k`, que a `k = 6` é `17×` a barra.

⚠️ **A cadeia inteira é reclamada pelo dispositivo** (`grelha → clone → 2× atributo → divisão →
rampa → saída`), com o `plan.is_fully_gpu()` a estourar se não for — *uma fronteira de CPU faria a
bancada comparar a CPU consigo própria*.

#### As RECUSAS, com a população ao lado

Dos `6` cartões de `motion.clone` no produto (§5.2) este kernel alcança **`2`**; as outras quatro
são `applicable` a devolver `false`, **com gate de PLANO a prová-lo e o controlo dos valores de
fábrica na mesma corrida**:

| recusa | cartões | mecanismo |
|---|---|---|
| `time_offset ≠ 0` | `1` | **COZIMENTO e não kernel** — ele re-cozinha a entrada em N instantes (`TimeFans`, ADR-0163). O que muda não é a conta, é quantas vezes o grafo a montante corre |
| `mode = Radial` | `1` | uma **variante** por escrever (o `wgsl_lib` do kaleidoscope já tem a seno parabólica portada), não um bloqueio |
| taper (`scale`/`rot`) | `3` | o único que **CUNHA**: com o knob ligado e a coluna ausente o `eval` cria `size`/`rot`, e uma escrita `…Existing` é por definição a que não cunha. ⚠️ O `applicable` só vê PARAMS, logo a recusa é pelo knob — inclusive nas cadeias em que a coluna existe |

**Gates:** `a_lei_de_contagem_e_a_do_eval` · `o_passo_derivado_e_o_da_cpu_ao_bit` ·
`o_k_derivado_e_o_do_orcamento` · `as_quatro_recusas_sao_as_medidas` ·
`a_renumeracao_entra_pela_cruza` · `o_corpo_chama_o_que_as_ligacoes_declaram` (na crate do nó) ·
`a_escrita_sem_leitura_e_de_porta_template_e_nao_cunha` (no enum) ·
`a_escrita_sem_leitura_nao_liga_buffer_de_leitura` · `a_presenca_de_uma_escrita_sem_leitura_entra_na_chave`
(no codegen) · `o_cloner_concorda_com_a_cpu_dentro_do_epsilon` ·
`as_recusas_do_cloner_entregam_o_no_a_cpu`.

**Mutação: 13 de 13 sangram**, com **seis** controlos verdes — quatro sobre o KERNEL pela paridade
na placa (o `Index` deixa de renumerar · a `Count` fica a do template · a fatia deixa de ser
slice-major · o posto ignora o `center`), uma sobre o **verbo** pelo naga (a `Count` volta a ser a
cruza — *a que reproduz o defeito que esta wave encontrou*), duas sobre os uniformes derivados, uma
sobre as recusas, três sobre o enum e duas sobre a derivação do codegen.

⏳ **O que a W1(a) deixa aberto:** o modo `Radial` como variante (a trig já está portada no irmão),
e o taper — que precisa de um verbo que **CUNHE** numa porta template, exactamente a pergunta que
esta wave respondeu do lado oposto.

#### ⛔⛔ E a pista `--ignored` da GPU tem DOIS vermelhos PRÉ-EXISTENTES, atribuídos por ABLAÇÃO

Correr a bateria de placa inteira (`226` testes que **nem o CI nem o `ship.sh` correm**) devolveu
três reprovados. A atribuição foi feita **antes** de olhar para o commit, ablando a única metade
desta wave que podia tocar noutro nó — o plano derivado do `codegen` e o bit da assinatura de
presença:

| teste | com a minha mudança | ABLADA | veredito |
|---|---|---|---|
| `both_routes_composite_the_same_document_in_the_same_blend` | vermelho | **vermelho** | pré-existente |
| `value_slope_kernel_matches_the_cpu_on_the_device` | vermelho | **vermelho** | pré-existente |
| `crossing_the_reach_boundary_does_not_step_the_cost` | **verde** | **verde** | flake de carga (reprovou no meio dos 226, passa sozinho) |

⚠️ **Os dois são de espécies diferentes e nenhum é um ε a derrapar:** o primeiro devolve **`0`
instâncias contra `64`** (uma avaria estrutural — o dispositivo não emite nada), o segundo lê
`max |d| = 1,05e-4` contra uma barra de `1e-4` (um fio acima). ⇒ *dívida NOMEADA, de quem os
escreveu; esta wave não os toca e não os afrouxa.*

⭐ **E o resto da mudança é inerte por construção, não por promessa:** o verbo novo é **append-only**
e **nenhum kernel existente o declara**; o `is_source_mapped` responde ao mesmo conjunto de sempre
mais ele; e o plano derivado coincide com a lista que substituiu para **todos** os dez verbos
anteriores, porque os únicos condicionais de então também LIAM (`here == presente`).

### §5.8 — ⭐⭐⭐ A CENA QUE MOSTRA (ordem do dono, 2026-09-20): `=126`, e a população NÃO é a do report

> ⛔⛔⛔ **LEIA A [§5.9](#59--o-report-do-corner-radius-e-a-segunda-derivação-errada-da-mesma-cena)
> ANTES DE CITAR QUALQUER NÚMERO DESTA SECÇÃO.** A tabela do «o que o app entrega»
> (`33 fps · 29.4 ms · 34 raw` pela rota antiga) **não reproduz**: ela foi obtida escalando por
> regra de três uma medição feita a `313 600` estrelas, e aquele regime tem um multiplicador — a
> dívida de tiques — que **não existe** abaixo da fronteira do vsync. Medido no app à população
> desta cena, a rota antiga custa `14,67 ms` e corre a `60` fps. *A secção fica inteira, com a
> refutação ao lado, porque o erro dela é o achado da §5.9.*

O dono mandou construí-la (*«construa uma cena de demonstração»*), e a §5.7 já dizia porquê: **as
onze cenas com carimbo não contêm o fenómeno** — a pior desenha `190` linhas e custa `0,1 %` de um
quadro. A cena vive em [`motion_state_carimbo_demo.rs`](../../../crates/ph2d-app-motion/src/motion_state_carimbo_demo.rs)
e a cadeia é a do report e **nada mais**: `motion.grid → motion.duplicator ← source.shape (Star) →
motion.output`. *Um oscilador a mais entra na conta do quadro e a cena passa a medir outra coisa.*

⛔⛔ **E a população que a §5.7 encomendou — `320 × 320 = 102 400`, a do report — NÃO SERVE para o
olho do dono, e a razão é a `raw`:** pelas portas do produto (§4.3) aquela população custa
`7,36 ms` pela rota antiga e `3,24` pela de hoje, ou seja **as duas cabem num quadro de 60 fps** ⇒
as duas corridas mostram `60` no número que ele lê, e a diferença só aparece no terceiro campo da
barra (`raw = 1000/cpu`). *Ela prova a lei e não a mostra.*

⛔⛔⛔ **E a derivação a partir das portas do produto errou por `~3×` — o achado desta wave.** A
1.ª redacção desta cena escolheu `560 × 560 = 313 600` porque a recta dos `0,0719` / `0,0316 µs` por
cópia prometia `22,6` contra `9,9 ms`. Medido **no app**, com o perfilador e em `--release`:

| rota | quadro medido | ~fps | do qual o COZER | previsto pela recta |
|---|---:|---:|---:|---:|
| `fill` por cópia (ANTES) | `90,4 ms` | `11` | `18,0 ms` | `22,6 ms` |
| carimbo preparado (HOJE) | `53,6 ms` | `19` | `19,7 ms` | **`9,9 ms`** |

⚠️⚠️ **O `audit_the_stamp_frame_split` mede DUAS fases** (cozer · encode) **e o quadro do app tem
mais** — resolver, publicar, os gizmos, a moldura. *Uma derivação que só conta as fases que a sonda
mede prevê um quadro que o app não tem*, e as duas rotas saíram do mesmo lado da fronteira: `11` e
`19` fps, ou seja **as duas a engasgar**, que é exactamente a metade da janela que a cerca de
compilação existia para impedir — **e ela deixou passar, porque as constantes dela é que estavam
erradas**. ⇒ *uma cerca é tão boa como o número que lhe puseram dentro*.

⭐ **O que a mesma tabela ILIBA:** o **cozer é igual nas duas** (`18,0` contra `19,7`, a rota não lhe
toca) ⇒ a distância inteira é o DESENHO. E é por o cozer ser comum que a razão do **quadro**
(`1,69×`) é muito menor que a do **encode** (`3,2×`) — *é a do quadro que manda na população, porque
é o quadro que o dono lê*.

⇒ **`300 × 300` = `90 000`**, com as constantes da cerca re-escritas a partir do APP
(`0,288` / `0,171 µs` por cópia) e a janela a exigir que a rota de hoje caiba num quadro e a antiga
peça **mais de um quadro e meio**. O que o app entrega, com a máquina a `88 %` ociosa:

| rota | **a barra de baixo diz** | quadro | cozer |
|---|---|---:|---:|
| carimbo preparado (HOJE) | **`59 fps · 16.7 ms · 95 raw`** | `16,64 ms` (preso ao ecrã) | `1,8 ms` |
| `fill` por cópia (ANTES) | **`33 fps · 29.4 ms · 34 raw`** | `28,6`–`38,7 ms` | `6,7`–`9,0 ms` |

⭐⭐ **E o cozer caiu de `19,7` para `1,8 ms` sem ninguém lhe tocar:** acima de `16,67 ms` a shell
cozinha **um quadro por tique em dívida** (`MOTION = N × cozer + 1 × separar`, a aritmética que o
`motion.contact` deixou nomeada), logo a `313 600` o cozer já trazia `N ≈ 3` dentro. *A cura do
encode paga-se DUAS vezes: no encode, e no cozer que ela deixa de arrastar.*

⭐⭐⭐ **E as duas fotografias são a MESMA IMAGEM, pixel a pixel** — as duas rotas escrevem os mesmos
bytes, com gate na `ph2d-vector` — **e só o número da barra muda**. É isso que faz desta cena uma
demonstração, e não a comparação de duas coisas diferentes.

⛔ **E o campo é MAIOR que o ecrã por ARITMÉTICA, não por descuido:** uma estrela só se lê como
estrela com `~6 px` (`0,108` de mundo a `55,5 px`/unidade, a régua medida da `=124`) e a câmara de
arranque mostra `21,8 × 6,8` unidades ⇒ cabem **`~10 000`** estrelas legíveis no ecrã, e a cena
precisa de `90 000` para o relógio se mexer. *Encolher a estrela até tudo caber entrega um
rectângulo cinzento*, e uma cena onde o dono não vê estrelas não ensina que aquilo são estrelas —
é por isso que o passo (2) do roteiro manda **aproximar com a roda** até as pontas aparecerem. A
conta é paga pelas `90 000` estejam à vista ou não: **o desenho não tem recorte por câmara**, e é
isso que faz a cena medir o que ela diz medir.

⚠️ **O smoke é `--release`** (`CLAUDE.md` §5: o `smoke` não tem LTO), e por isso a ferramenta de
fotografia ganhou a porta `FOTO_PERFIL` — **por omissão continua `smoke`**, e `release` é só para
um smoke de PERFORMANCE, senão as duas colunas de um A/B medem o perfil de build.

⚠️⚠️ **E a MÁQUINA decidiu metade do dia:** a 1.ª tentativa de medir apanhou a placa **reservada
por outra linha** (o guarda recusou, e bem) e depois `2`–`16 %` de ociosidade; a corrida do
`audit_the_stamp_frame_split` feita nessa janela leu `cozer 4,75 ms` contra os `0,99` da máquina
calma — **`4,8×`**, e foi deitada fora. *Os números desta secção são todos de `88 %` ocioso, e a
sonda imprime a carga ao lado de cada um precisamente para que uma tabela dessas nunca entre num
doc por engano* (`CLAUDE.md` §5.0).

---

### §5.9 — O report do `Corner Radius`, e a SEGUNDA derivação errada da mesma cena

O dono correu o smoke da §5.8 e devolveu **uma frase com duas coisas dentro**: *«O caminho mais
novo é mais rápido mas nenhum dos dois tolerou modificar o corner radius das estrelas. travou»*.

#### §5.9.1 — O que o `Corner Radius` faz, medido

⭐ **A causa é de POPULAÇÃO DE SEGMENTOS e nada mais.** O
[`round_closed_corners`](../../../crates/ph2d-vec-scene/src/corners.rs) troca **cada** vértice de quina
por dois — ou **três**, quando a quina vira mais de `90°`, que é o caso das pontas de uma estrela —
⇒ a estrela de `5` pontas passa de **`12` para `30`** vértices assim que o `corner` sai de zero, e
fica lá (a contagem é `30` em todo o resto do curso do slider).

Medido no app (`--release`, `1930×1040`, o `cpu-encode(raw)` do perfilador, que é o **quadro
inteiro de CPU**), sobre as `90 000` estrelas desta cena:

| rota | `corner = 0` (12 vértices) | `corner = 0,5` (30 vértices) | razão |
|---|---:|---:|---:|
| carimbo preparado | `9,99 ms` | **`19,53 ms`** | `1,96×` |
| `fill` por cópia | `13,36 ms` | **`30,30 ms`** | `2,27×` |

⇒ **o quadro dobra**, e as duas rotas passam o orçamento de `16,67 ms`. *A `90 000` cópias não há
rota que salve `2,5×` de segmentos.*

⚠️ **E o GESTO está ilibado com número:** o custo de um arrasto de slider é o `publish` com uma
chave de conteúdo nova (a forma reconstruída, internada e medida para o colisor), e ele mede
**`0,01 ms`** — dentro do ruído. *Arrastar não é pior do que o valor parado; o que trava é o que
fica desenhado.* O instrumento `PH2D_CARIMBO_CORNER=<f>` semeia o valor sem clicar, e é o que torna
isto medível (um gesto de painel **não é alcançável de um teste**).

⛔⛔ **E as sondas de encode NÃO continham o fenómeno, por uma razão que vale para toda a casa.** A
[`audit_the_corner_radius_cost`](../../../crates/ph2d-app-motion/src/motion_carimbo_relogio_probe.rs)
mede, pelas portas do produto, `cozer + desenho`: ela lê `3,10 → 4,08 ms` (`+32 %`) pela rota de
hoje enquanto o app lê `+96 %`. O que falta na conta é a **preparação da cena no Vello**, que corre
depois do encode, escala com os segmentos e **nenhuma sonda deste módulo mede** — o cabeçalho do
`motion_carimbo_relogio_probe` já o dizia por escrito (*«o que sobra é a RASTERIZAÇÃO, que nenhuma
destas sondas mede»*) e eu li a frase como uma nota de rodapé em vez de uma fronteira de régua.

#### §5.9.2 — ⛔⛔⛔ A tabela da §5.8 não reproduz, e a causa é o REGIME

Ao reproduzir o report, o **controlo** contradisse a §5.8: a rota antiga com `corner = 0` lê
**`13,4`–`14,7 ms` de CPU e corre a `60` fps**, onde aquela secção promete `33 fps · 29,4 ms`. Duas
corridas independentes, a `91 %` de CPU ociosa, com o mesmo binário.

⭐⭐⭐ **A causa está na ESCADA, e ela é o achado:** com o knob `PH2D_CARIMBO_LADO` mediu-se a
população no app, nas duas rotas —

| lado | estrelas | carimbo preparado | `fill` por cópia | razão |
|---:|---:|---:|---:|---:|
| **`300`** | **`90 000`** | **`10,37 ms`** | **`14,67 ms`** | **`1,41×`** |
| `320` | `102 400` | `16,77` | `21,26` | `1,27×` |
| `400` | `160 000` | `25,64` | `33,40` | `1,30×` |
| `500` | `250 000` | `45,70` | `56,94` | `1,25×` |
| `600` | `360 000` | `72,58` | `88,89` | `1,22×` |

De `90 000` para `102 400` o relógio sobe **`1,62×`** para **`1,14×`** de população; de `102 400`
para `360 000` a inclinação é **`3,3×` menor**. ⇒ *a descontinuidade não é a população — é o
vsync*: acima de `16,67 ms` a shell cozinha **um quadro por tique em dívida**
(`MOTION = N × cozer + 1 × separar`) e esse multiplicador desaparece do outro lado.

⇒ **um custo medido acima da fronteira NÃO se extrapola para baixo dela**, e foi exactamente isso
que a §5.8 fez: ela mediu a `313 600` (onde o quadro já estava em dívida) e dividiu por `3,48`.

⚠️⚠️ **É a MESMA classe de erro da §5.8, um nível acima.** Lá a lição foi *«uma derivação que só
conta as fases que a sonda mede prevê um quadro que o app não tem»* e a cura foi **medir no app**.
Aqui a medição É do app e continua a não transferir, porque **o regime muda**. ⇒ a lei que fica:
*medir no sítio certo não basta — tem de ser no REGIME em que o produto vai correr*.

#### §5.9.3 — ⚠️⚠️ E a fronteira não é só abrupta: ela é INSTÁVEL

Três corridas na fronteira, a `305`, `310` e `315` de lado, pela **mesma** rota e com o mesmo
binário: `15,62`, `11,77` e `16,44 ms` — **não monótonas**. Ali o app cai de um lado ou do outro da
dívida conforme o ruído do arranque, e uma cena naquele ponto daria ao dono **um número diferente a
cada corrida**. ⇒ *nenhuma cena deste repo pode viver na fronteira do vsync*, e a cerca de
compilação da `=126` passou a exigi-lo.

#### §5.9.4 — O que a cena passa a ser

A `300 × 300` fica (a razão **cai** com a população, logo aumentar dilui a cura em vez de a
mostrar), e o que muda é **o que o roteiro manda ler**:

| rota | a barra de baixo diz | CPU do quadro |
|---|---|---:|
| carimbo preparado (HOJE) | `59 fps · 16.7 ms · **~96 raw**` | `10,37 ms` |
| `fill` por cópia (ANTES) | `59 fps · 16.7 ms · **~68 raw**` | `14,67 ms` |

⚠️ **Os `fps` são iguais nas duas e isso é o ECRÃ, não a cura** — as duas cabem no quadro e ficam
presas ao vsync. A coluna que se move é o **`raw`** (`1000 / cpu`), que é a folga. *Um roteiro que
mandasse comparar os `fps` ensinaria que a cura não faz nada* — a espécie que o `CLAUDE.md` §5.0
chama de **pior que uma cena ausente**, e que a §5.8 shipou sem saber.

A cerca de compilação passou de duas metades para **quatro**, e uma delas estava **invertida**: ela
exigia que a rota antiga custasse *mais de um quadro e meio* e passava só porque as constantes
vinham do regime com dívida. Hoje: *a de hoje cabe* · *a antiga também cabe* (a cena não vive na
zona instável) · *a antiga usa pelo menos três quartos do quadro* (abaixo disso o que é fixo domina
e a diferença dilui-se) · *e a distância entre as duas é grande o bastante para se ler*.
**10 mutações, 10 sangram**, três delas por **não compilar**.

---


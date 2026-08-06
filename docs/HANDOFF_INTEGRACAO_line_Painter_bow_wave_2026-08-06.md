# HANDOFF DE INTEGRAÇÃO — `line/Painter`, o bow wave gateado no knob + as bandas por trabalho (2026-08-06)

> **9 commits · 13 arquivos · nenhum `Cargo.toml` · nenhum ADR · `project.rs` intocado.**
>
> ⚠️ **PENDENTE DE SMOKE.** A jornada tem **duas metades independentes**: a do **bow wave** (§1-§9,
> sete commits, um deles muda o produto — leia o §3) e a das **BANDAS POR TRABALHO** (§10, dois
> commits, um deles muda o produto e é **byte-idêntico**). As duas podem ser integradas juntas; elas
> não se tocam.

---

## §1 O que a jornada respondeu

O smoke de 05/08 mandou a fronteira para o `on_canvas_pointer`, e a decomposição por **meio**
(`what_a_shape_move_is_made_of`, levada ao raio do produto) disse que o **Impasto custa 19× o Digital**
e a **Aquarela 14×**, sobre a MESMA figura e o MESMO pincel. Nenhuma das quatro frentes que o
[plano 34 §5](Painter/34_plano_smokes_e_cerca.md) listava era a maior — **a fronteira é o MEIO**.

Descer um nível exigiu ablação **dentro do laço que o produto roda**, e o resultado a 4096², raio 185,
`DrawTo::Depth` (ms por traço):

| raio | camada | full | cauda | silhueta | filme | miolo |
|---|---|---|---|---|---|---|
| 185 | virgem | 111,33 | 15,40 | 19,36 | 21,97 | 48,85 |
| 185 | tela nua | 136,64 | **53,74** | 19,84 | 21,52 | 34,22 |
| 185 | sobreposto | 135,84 | **54,79** | 18,74 | 18,42 | 32,19 |

**A cauda é o BOW WAVE**, e ele custava o mesmo cruzando a parte **nua** de uma camada suja e cruzando
a própria tinta. Gateado no knob: **136,64 → 96,93 ms/traço**, com a cauda voltando aos 14,33 da
camada virgem.

---

## §2 Os sete commits

| sha | o que |
|---|---|
| `7b5d68042` | a sonda do shape move passa a **conter o fenômeno** (raio 24 → varredura até 185) |
| `33ba483c6` | **o pino byte-a-byte** do depósito — 3 gates, 6 mutações |
| `bc7c2452b` | a chave de **ablação** do laço de altura + a decomposição + o gate de que ela nunca é armada em produto |
| `b85c24a61` | a fixture media o regime errado — separa camada virgem de tinta-sobre-tinta |
| `ab5b3d91f` | duas curas construídas e **refutadas** (o early-out por-texel · a forma fechada) |
| `661340c23` | **o gate no knob** — o único commit que muda o produto |
| `b34e18833` | o comment do `draw_to_split` se refutava no próprio parênteses |

---

## §3 ⚠️ A MUDANÇA DE COMPORTAMENTO — leia isto antes de integrar

**`impasto_push` deixou de ser um knob live pós-traço.**

Um traço deitado com **Push = 0** (que é o *default* do pincel de depósito) não guarda mais o
ingrediente, então **alcançar o slider depois não re-deriva nada**. Push virou decisão de **antes** do
traço, e é **o único knob do card Body que não é live** — uma exceção num card de cinco, deliberada,
com o número ao lado.

**O que NÃO muda:** o frame que shipa é **byte-idêntico**. A re-derivação é
`field[i] = deposit + push * push_plane[i]`, então com `push == 0` tudo que a mordida, o banco e a onda
escreveriam é multiplicado por zero. O **pino plano-a-plano ficou verde** em toda a mudança — é ele
que torna esta afirmação verificável em vez de argumentada.

**A cerca executável foi REESCRITA, não apagada.** O `impasto_push_is_a_live_knob_and_never_erodes_the_ground_twice`
caiu — que é o comportamento certo dele — e virou
`impasto_push_is_live_on_a_stroke_that_was_laid_with_it_armed`: as três afirmações (LIVE · IDEMPOTENTE ·
REVERSÍVEL) seguem pinadas, agora sobre um traço deitado com o knob **armado**. *A capacidade não sumiu;
a precondição dela mudou.* E nasceu o irmão `a_stroke_laid_with_push_off_has_no_ingredient_to_re_derive`,
que pina a outra metade **para ninguém restaurar o gate antigo por acidente e devolver os 30 % em
silêncio**. Mutação: tirar o gate do knob deixa **só o irmão** vermelho — o par certo, porque apenas o
destino *Push-off* testa a lei nova sozinho.

**Decisão do Enio, 2026-08-06**, tomada depois de as outras duas saídas serem medidas e fechadas (§5).

---

## §4 Duas afirmações do próprio repo que a medição derrubou

1. **`impasto.rs`:** *"a first stroke on bare canvas has no ground, so it pays nothing — **the cost falls
   exactly where the feature is, on paint laid over paint**"*. A segunda metade é **FALSA**: `ground` é
   `self.heights.get(&active)`, ou seja da **CAMADA**. Corrigida no lugar onde estava.
2. **`the_impasto_draw_to_split`:** o comment dele dizia *"as faixas têm de ser distintas, senão o 2º
   traço encontra o relevo do 1º e o bow wave entra na conta"* e **no mesmo parênteses** enunciava o
   mecanismo que o refuta (*o `ground` é da CAMADA*). ⚠️ **Logo a coluna `Depth` da tabela dele está
   contaminada**, e o *"a altura custa 2,3× a 12× o pigmento"* que o módulo cita é um número com o bow
   wave dentro. Marcado no lugar onde a tabela vive; **não re-medido**.

---

## §5 ⛔ Medido e rejeitado — não refaça

* **O early-out por-texel na mordida** (`if q != 0` antes da divisão). Construído, byte-idêntico,
  medido: **53,74 → 53,24**, dentro do ruído de corrida (~5 %). O porquê é o mecanismo:
  `q = ground + plane`, e o **`plane` recebe o BANCO do próprio traço** ⇒ `q ≠ 0` sob quase todo texel
  mesmo com `ground = 0`. A mordida **não está ociosa, está transportando**. Revertido inteiro.
* **A forma fechada** (`take = g·m_final`, que o doc do kernel prova). Descartada **por leitura**, antes
  de custar código: o `plane` acumula o banco entre dabs, então `q` não é `ground·(1−m)` e a identidade
  quebra.
* **Fundir as duas varreduras** (a candidata que o `measure_impasto_cost` nomeava): ela ataca a
  **silhueta**, o **menor** dos quatro itens — teto de **14 %**.

---

## §6 Tabela de colisão

| item | estado |
|---|---|
| `PROJECT_SCHEMA` | **INTOCADO** — `project.rs` não é tocado (`git diff --name-only`) |
| contrato congelado | **4/4 verde** (rodado, não auto-relatado) |
| ADR | **nenhum** — a linha fica fora de toda disputa de número |
| `Cargo.toml` | **zero** |
| crate nova | nenhuma · dep nova: nenhuma |
| ids / tokens | nenhum |

**Superfície pública nova:** `ph2d_painter_brush::ablate` (`SILHOUETTE` · `FILM_AA` · `TAIL` · `set` ·
`get` · `with`) — ⚠️ `pub` **por necessidade**, porque `cfg(test)` desta crate não vale quando quem roda
o teste é a `ph2d-tool-painter`; o preço é o arch-gate
`the_ablation_switch_is_only_armed_by_measurements`, que varre as **duas** crates com controle positivo
nas duas pontas.

**Suítes:** `ph2d-painter-brush` 288 · `ph2d-tool-painter` 993 (lib) · shell **106 blocos verdes** ·
clippy limpo · LOC sob o teto.

---

## §7 O SMOKE

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --release
```

Três perguntas, nesta ordem:

1. **Com o pincel de impasto padrão, NADA pode ter mudado na tinta.** É byte-idêntico por construção e
   o pino prova; o smoke é a testemunha independente.
2. **Arme o Push ANTES de um traço** — o arado, o canal e a crista continuam lá, e o slider segue vivo
   depois (subir e descer devolve o chão bit a bit).
3. **Deite um traço com Push zerado e depois suba o slider: ele não faz mais nada.** É o desenho novo;
   se isso te incomodar na mão, o veredito é seu e a reversão é uma linha.

---

## §8 O que segue aberto, com número

| frente | estado |
|---|---|
| **o cap de Accumulate no WGSL** | a rota do DEVICE exige `!accumulate_cap` (`stamp_route.rs:454`); o log mediu o device **1,1-2,75× mais barato por visita** com a CPU levando **31-54 %** dos lotes. **Ganho não medido.** |
| **o filme (AA)** — **20,89 ms** medidos | era a "frente 2" estimada em ~17; o LUT **aplica** no raio do produto, então não é multi-tap. Decisão de **LOOK**. |
| **os quatro sítios fora da porta** | `compose.rs:218` · `selection_overlay.rs:91` · `stamp_color_cache.rs:362` · `transform_float.rs:346`. Os quatro carregam **`1 << 17` idêntico** + contagem constante — a forma exata que a wave das bandas curou. ⚠️ E o `compose` decide **todo quadro**: o rect sujo de um move é **15 625 px**, logo `15625 < 131072` ⇒ **sempre serial**. ⚠️ **O `SPAWN_EQUIV_VISITS = 808` NÃO é transferível** — ele é `custo_de_spawn ÷ 13 ns`, e os 13 ns são do kernel do DAB; emprestá-lo ao compositor seria a constante-emprestada que esta linha vive pegando. Precisa de fixture de `LayerStack`, que não existe na camada de sondas ⇒ **wave própria**. |
| **a Parte B do plano 34** (a cerca) | não iniciada |
| **o `PH2D_MASK_SMOKE=1`** (§2.1 do plano 34) | não há registro de ter rodado nesta jornada |

⚠️ **O plano 34 está obsoleto na Parte A:** ele lista quatro frentes candidatas e o log escolheu
**nenhuma** delas. A frente que ele abriu — o MEIO — está fechada por este handoff.

---

## §9 Nota de processo

⚠️ **A cwd do Bash escorregou para a árvore PRIMÁRIA uma vez**, num lote de `grep` de fechamento: os
números que ele devolveu (`3 files changed`, `0 commits`) eram do **primário sujo**, não da linha.
Nada foi commitado lá — todo commit saiu `[line/Painter …]` — e os fatos do §6 foram refeitos dentro da
worktree. É a **quarta** ocorrência registrada nesta linha, e a regra continua a mesma:
*todo comando começa com o `cd` da worktree.*

---

## §10 A SEGUNDA METADE — o sistema de SHAPES VIVAS, e as bandas por TRABALHO

Ordem do Enio: *"em vez de investigar a tinta investigue o sistema de shapes vivas (freehand, elipse,
Polygon, Curve, Line). Descubra o custo do sistema mesmo que a pintura não esteja envolvida. Descubra
se é possível otimizar."*

### §10.1 A separação não precisou de ablação nova

O `stamp_drag_preview` já cronometra as suas quatro fases **no código que shipa**
(`stamp_banded::diag::note_restamp`), então `sistema = evento − carimbo`, e a geometria — captura,
clone do conjunto parqueado, offset/flatten/trim da espinha, `fill_*_preview` — sai por subtração.
Sonda nova: `measure_shape_system` (cinco tabelas).

### §10.2 O que a máquina custa (e o que ela NÃO custa)

* **O quadro OCIOSO é livre:** ~**20 µs** com 16 formas parqueadas e uma curva de 512 âncoras — 0,1%
  de um quadro. O `stroke_op_badges` constrói a lista de dabs inteira de cada forma parada só para
  tirar um min/max, e mesmo assim o número absoluto é ruído.
* **A máquina é NEUTRA ao método.** Line · Ellipse · Polygon · Curve · FreeHand custam ~1 ns/visita
  cada; o que difere entre as colunas é **quantos dabs a figura tem**, não o editor.
* Num move normal (figura de 400 px, 4096²) o sistema é **13-18%** do evento e o depósito é o resto,
  na estrada em **banda**.

### §10.3 ⬛ O ACHADO: o mesmo lote custa 2,15× só por estar ESPALHADO

O `stamp_plain_dabs_banded_run` dividia a **ALTURA** da união dos dabs em N fatias iguais e abria uma
thread por fatia. Uma figura viva raramente enche a própria caixa: uma elipse no centro mais uma
forma parqueada num canto dá uma união quase do tamanho da tela **com o miolo vazio**, e o corte por
altura entrega a maioria das bandas a linhas onde nenhum dab escreve — elas terminam
instantaneamente, a banda da figura faz tudo, e o lote roda no tempo de UMA banda depois de pagar N
spawns.

Medido com o **MESMO lote em dois lugares** (mesmos dabs, mesmas visitas, mesmos pixels — só a
esparsidade da caixa muda), 4096², pela porta do artista:

| | colado | espalhado | |
|---|---|---|---|
| 2 formas paradas | 4,72 ms | 8,01 ms | 1,70× |
| 8 formas paradas | 11,21 ms | 24,10 ms | **2,15×** |

⚠️ **E não é cena exótica:** **Tiling** embrulha as cópias para a borda oposta e **Symmetry** as
espelha para o outro lado do canvas — as duas produzem exatamente esta união rala **com uma figura
só**.

### §10.4 A cura, e por que ela é byte-idêntica

O corte passa a sair de um **perfil de trabalho por linha** (`row_work`, a MESMA `dab_write_bounds`
que o lote já usa para se limitar e que a banda já usa para rejeitar dab) cortado em **quantis iguais**
(`work_bands`). Byte-idêntica pelo argumento do **ADR-0109** que o módulo já carregava: bandas são
linhas **disjuntas** e cada uma percorre TODOS os dabs na ordem da lista, então muda quem **avalia** a
linha, nunca o que a linha diz.

Medido depois, mesma porta, máquina calma (`load 1,25`):

| cena | antes | depois | |
|---|---|---|---|
| uma figura só | 3,58 ms | **2,52** | 1,42× |
| 2 formas, coladas | 4,72 | **2,71** | 1,74× |
| 2 formas, espalhadas | 8,01 | **5,17** | 1,55× |
| 8 formas, espalhadas | 24,10 | **6,64** | **3,63×** |

⚠️ **O `ns/visita` ficou CONSTANTE em 0,69-0,85 nas cinco cenas** — era 1,16 a 3,78. É essa coluna que
prova que o que sumiu foi desequilíbrio, e não trabalho.

### §10.5 Gates, e as duas fixtures que nasceram cegas

Cinco gates novos em `stamp_banded_work_tests.rs` (irmão por ASSUNTO: lá *"dividir as linhas não move
um byte"*, aqui *"as linhas são divididas de modo que as bandas paguem o mesmo"*) — a identidade sobre
um lote **esparso** · o corte reparte o trabalho · o perfil por linha soma a área declarada · nenhuma
banda nasce com altura zero · e a **consequência, como RAZÃO** entre as duas posições.

**4 mutações, 4 sangram** (corte por altura ⇒ 2,42× no gate de razão · perfil constante ⇒ o gate do
perfil · sem a guarda ⇒ `[4, 0]` · a fatia da máscara desalinhada da tinta ⇒ **6 gates**, incluindo o
fingerprint byte-a-byte do impasto).

⚠️ **Duas fixtures minhas nasceram sem o fenômeno, e as duas mutações PASSARAM antes de eu as
consertar:** o gate de razão usava a tela de 512² dos gates de identidade, onde a união rala tem
poucas centenas de linhas e a fatia vazia de cada banda é curta; e o gate da banda vazia punha o
trabalho no **meio** do perfil, onde a guarda nunca chega a ser consultada (ela só morde com o
trabalho no **fim**).

⚠️ **E a fixture do gate de identidade irmão é um arco DENSO** — nela toda linha da união recebe
trabalho, então cortar por altura e cortar por trabalho dão bandas parecidas: ela **não conseguia** ver
esta wave, e por isso o lote esparso entrou como fixture própria.

### §10.6 O que sobra, com o número

Depois desta wave o resíduo é **UMA coisa só, em todas as tabelas**: o `save_region`/`restore_region`
copiam a **bbox da união**, enquanto a tinta de uma figura fechada é um **anel**.

* figura de raio **1600**: `restore 3,68 + save 6,25 = 9,93 ms` de um evento de 19,24 — **54%**;
* com formas parqueadas: `1,05 + 1,01 = 2,06 ms`, hoje **40-43%** do evento.

Desperdício medido (bbox ÷ tiles de 64 px que a tinta toca): Ellipse/Polygon **1,7× a 5,5×** conforme
a figura cresce — mas ⚠️ **Line e Curve medem 0,7×**, isto é, uma pegada por tile seria **PIOR** para
elas. Uma cura tem de escolher a mais barata das duas, não trocar de régua.

⚠️ **E há uma frente maior atrás dessa:** com formas paradas, o depósito ainda **re-carimba todas** a
cada move — a baseline do preview é pristina, então a tinta parqueada é apagada pelo restore e
re-laid. Fazer a baseline **incluir** as formas paradas (só a ativa fica no preview) é o desenho
correto e é **wave própria**: mexe no ciclo de vida do baseline (park/activate/offset/booleana) e
interage com o relevo, o sculpt e o Wet Paint.

### §10.7 Outros números que a varredura deixou nomeados

* **Impasto: 19,7 ns/visita** contra 0,91 do Digital, na MESMA figura e MESMO pincel — 21×. É o
  depósito de altura, território da metade anterior desta jornada.
* **Aquarela: n/medido.** Ela entra pela porta própria (`stamp_drag_preview_watercolor`), que não
  chama o `note_restamp` — a sonda **declara** isso em vez de deixar a subtração inventar 73 ms de
  "geometria".
* ⚠️ **`set_brush_size_px` escreve o RAIO, não o diâmetro.** A sonda irmã `measure_shape_cost` passa
  `radius * 2.0` e rotula a coluna `r=24`: o pincel que ela roda tem raio **48**, e a varredura que o
  doc dela apresenta como *"o ponto do log de 05/08, pincel r~185"* roda com raio **370** — quatro
  vezes a área por dab. O veredito de **RAZÃO** daquela tabela (Impasto ÷ Digital) sobrevive, porque
  os dois lados pagam a mesma duplicação; os **ABSOLUTOS** dela descrevem outro pincel.

### §10.8 O SMOKE desta metade

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
env PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --release
```

Canvas 4096, ferramenta Painter, um método de figura (Ellipse/Polygon/Curve/Line/Free Hand).

1. **A tinta não pode ter mudado.** A wave é byte-idêntica por construção e o fingerprint do impasto
   prova; o smoke é a testemunha independente.
2. **Desenhe uma figura, deixe-a na tela, comece outra.** É aqui que o ganho vive — antes, a segunda
   figura ficava progressivamente mais lenta a cada forma parada na tela.
3. **Ligue o Tiling ou a Symmetry e arraste uma figura.** É o caso de UMA figura só que produz a
   união rala; o arrasto tem de ficar liso.

---

## §11 O BOOLEAN das shapes — era CARO, e da forma errada

Pergunta do Enio: *"avalie se o boolean das shapes é barato ou caro"*.

### §11.1 O veredito

**Caro, e da forma errada.** Escolher **Add** ou **Remove** numa forma tira a figura do caminho de dabs
e a manda pelo `stroke_boolean_contours`, que roda a **cada move de ponteiro** — e ele alocava e
percorria um buffer supersampleado do **canvas inteiro**. Com `SS = 3` isso é **12288² = 151 MB** a
4096², zerado uma vez para o `crisp`, **outra vez POR FORMA** para o `region`, mais duas dentro do
traçado; e o `scanline_fill` percorria as 12288 linhas para desenhar uma figura de 400 px.

Medido pela porta do artista, com a **MESMA figura de 200 px** nas três telas (um move, mediana):

| tela | Overlay | 1 Add | 2 Add | 4 Add | Add ÷ Overlay |
|---|---|---|---|---|---|
| 1024 | 1,30 ms | 21,7 | 37,4 | 39,0 | 16,7× |
| 2048 | 1,40 | 77,8 | 118,9 | 127,3 | 55,8× |
| 4096 | 1,50 | **284,4** | 450,3 | 483,3 | **190×** |

⚠️ **A coluna Overlay é PLANA** — a figura não mudou, o custo não muda; é o comportamento certo. A de
Add cresce **4× por 4× de área**: o custo era do **BUFFER**, não do desenho. Marcar `Add` custava 190×
um move normal por uma razão que não tem nada a ver com o que o artista pediu.

⚠️ **E uma forma sozinha já paga:** com `active_is_bool` a figura ativa entra sozinha no composite (o
contorno dela passa a vir do traçado, não dos próprios dabs). A coluna `1 Add` não é caso de canto —
é o que acontece assim que alguém escolhe a Operation no painel. **284 ms num move** são 17 quadros.

### §11.2 A cura

A janela sai da **caixa das formas que ADICIONAM** — o resultado de um boolean está contido na união
dos Add, então um Remove distante não pode mudar um texel —, presa ao canvas. O rasterizador recebe as
coordenadas deslocadas e a largura da janela: uma **tela virtual**, o mesmo truque da banda do
`stamp_banded`, sem segunda resposta a *"que pixels esta forma cobre?"*. O `region` passou a ser um só,
reusado.

| tela | Overlay | 1 Add | 2 Add | 4 Add |
|---|---|---|---|---|
| 1024 | 1,36 ms | 7,62 | 15,3 | 18,9 |
| 2048 | 1,53 | 8,15 | 15,6 | 17,7 |
| 4096 | 1,64 | **8,08** | 16,7 | 17,9 |

⚠️ **A coluna Add ficou PLANA na tela** — o boolean virou um fato da FIGURA. A 4096²: **284,4 → 8,08
ms, 35×**.

### §11.3 Gates

A identidade é contra a **rota de TELA CHEIA congelada sob `cfg(test)`** (`stroke_boolean_contours_whole_canvas`)
— o oráculo é *o que shipava*, não uma reimplementação escrita para o teste. 12 cenas × 3 offsets: uma
Add sozinha · duas que se cruzam · duas separadas (duas componentes) · Add menos Remove · **Remove
longe** · a figura **saindo** do canvas · **colada** na aresta · **elipse girada** · polígono · **curva
fechada com as alças fora da caixa dos pontos** · e as duas com Remove. Mais o early-out sem Add, e a
**consequência como razão** entre duas telas.

**3 mutações, 3 sangram** (a caixa cega à rotação · o casco sem as alças · a janela saindo dos Remove).

### §11.4 ⚠️ Uma 4ª mutação acusou a minha AFIRMAÇÃO, não o gate

Eu pus uma folga de 2 texels em volta da janela com o comentário de que **era ela** que mantinha o
traçado idêntico (*"uma componente colada na borda do buffer não é percorrida como uma que tem zeros
em volta"*), e armei a mutação que deveria sangrar. Ela **passou**, nos dez casos — inclusive numa
figura no meio do canvas cuja caixa apertada a encosta na coluna 0 da janela.

O motivo está no `selection_trace::inside`: ele **confere os limites** e devolve `false` fora do
buffer, ou seja **fora lê como FUNDO** — exatamente o que uma borda de zeros daria. A folga saiu, em
vez de ficar com uma justificativa falsa ao lado. O que de fato mantém a identidade é o `clamp` ao
canvas.

⚠️ **E a fixture da mutação das alças nascia cega:** Polygon e Line fechada produzem `Freehand` com as
alças **em cima** dos pontos (`[q, q]`, quinas vivas), então dropar as alças não muda uma coordenada.
Só com uma **curva fechada cujas alças saem da caixa dos pontos** ela morde.

### §11.5 O que sobra

Um Add ainda custa **~8 ms** contra 1,6 do Overlay — **5×**. Isso é o preço intrínseco de rasterizar a
figura a `SS = 3` (9× os pixels dela), inundá-la e andar a fronteira; agora é **proporcional à
figura**, que é a forma certa, mas 8 ms ainda é meio quadro. As alavancas óbvias, **não medidas**:
baixar o `SS` (é decisão de LOOK — ele existe para o contorno não sair serrilhado) · traçar sem o
flood-fill por componente (o `trace_all_contours` aloca um buffer do tamanho da janela **por
componente** e faz um `find` linear a cada volta) · e cachear o contorno enquanto a geometria não muda.

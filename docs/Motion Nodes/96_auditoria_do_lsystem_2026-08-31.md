# 96 — Auditoria de SEIS LENTES ao `source.lsystem` (2026-08-31)

> Pedido do Enio: *"Se finalizamos o L-system, faça auditoria completa com múltiplos agentes"*.
>
> Seis lentes independentes sobre os **26 commits** da `line/motion-value`
> (`git log --oneline main..HEAD`, de `eae967cbd` a `6b5c73fbf`). Nenhuma podia responder pela
> outra, e **cinco das seis acharam coisas que as outras não viram**.
>
> ⚠️ **Este doc é o produto da auditoria, e existe para ela não evaporar.** O que uma auditoria
> descobre e ninguém endereça é indistinguível de nunca a ter feito.

## Como ler

Cada achado traz **mecanismo · reprodução · o gate que faltava**. Onde um gate EXISTENTE estava
verde sobre o achado, ele é nomeado com o motivo de estar verde — que é a metade que rende mais.

**`✅ verificado`** = re-medido pelo autor deste doc, não só reportado por uma lente.

---

## §0 — A forma comum, e ela explica quase metade da lista

⭐⭐⭐ **O commit `774b5806c` mudou a FIXTURA — o texto das oito gramáticas, ao pôr `[J]` em todas —
e nenhuma das medições escritas sobre a fixtura antiga foi re-corrida.**

Daí saem, de uma vez: o `step` do `Wild` (§1.1), cinco parágrafos de doc-comment (§5.1), as
contagens de marca de três moldes (§5.2), a faixa de razões do `growth.rs` (§5.3), e três linhas
da tabela de recusas (§5.4).

⇒ **o gate que teria apanhado os cinco é o mesmo:** um censo por molde, DERIVADO, afirmado contra
as constantes que o molde carrega. Nenhum deles precisava de ser descoberto por uma auditoria.

⚠️ E a segunda forma comum, com três ocorrências (§3.1, §4.1, §4.2): **uma isenção escrita numa
lista de saltos, cuja justificação nunca é re-medida.** As três dizem *"este param é lido pela
shell"* / *"no Custom a gramática é a que o artista escreveu"* / *"estes estourariam a sonda"* —
e as três estão erradas para parte da população que isentam.

---

## §1 — O QUE SE VÊ NA TELA

### 1.1 ⛔⛔ O molde `Wild` sai **15 % mais pequeno** que os irmãos ✅ verificado

**Mecanismo.** `presets.rs` declara a lei: *«O `step` e o `width` CONTAM-SE, não se escolhem»*,
com alvo = mediana dos que o dono aprovou. `774b5806c` inseriu `[J]` na regra do `Wild` e deixou
`step: 0.478`. O `Wild` é o **único molde estocástico** (3 produções com peso) — inserir um módulo
desloca o fluxo de sorteios e escolhe outras produções.

| molde | mundo | step | step ideal |
|---|---|---|---|
| Tree · Fern · Bush · Weed · Koch · Dragon | 1,76–1,81 | — | dentro de 2 % |
| **Wild** | **1,50** | 0,4780 | **0,5663** |
| Sprig | 1,83 | 0,5240 | 0,5077 |

Os outros sete moldes deram **zero** posições novas com o `[J]`; o `Wild` deu **32**.

**Reproduzir.** `cargo run -q -p ph2d-node-source-lsystem --example preset_report --release`,
secção `== O ENQUADRAMENTO ==`.

**Gate verde, e porquê.** `every_preset_frames_itself_like_its_siblings` com `K = 1.6`. A barra
foi dimensionada pela dispersão **anterior à cura** (*«2,7..3,9 ⇒ 1,44×»*) e admite ±60 %; o
`Wild` está a 1,185× da mediana. ⚠️ **Uma barra tirada da doença que se está a curar tolera a
doença a voltar.**

**Gate que falta.** Afirmar a **derivação** que o doc declara, não uma banda:
`|step − step·mediana/tamanho| ≤ 5e-4`. Nasce vermelho no `Wild`, que é o que se quer.

### 1.2 ⛔⛔ `Step Scale` e `Grow Angle` nascem MORTOS num nó recém-largado ✅ verificado

**Mecanismo.** É o defeito da wave de ontem (`dbba4c344`). Os dois `ParamGate` novos incluem
`PRESET_CUSTOM`, que é o **default do manifesto**, e o modo default é **`Guided`**. Ali a
gramática é a **derivada** (`A(s) -> F(s)[J]![+A(s*length_scale)][-A(s*length_scale)]`):
paramétrica ⇒ o `Setup::step` nunca é lido; cresce pela ponta ⇒ o braço faz `ang_frac = frac`
**sempre** e nunca lê o `continuous_angle`.

Medido no estado de fábrica, as duas leituras (geração inteira | fraccionária):

| param | saídas distintas | veredito |
|---|---|---|
| `step_scale` | 1 \| 1 | ⛔ **INERTE nas duas** |
| `continuous_angle` | 1 \| 1 | ⛔ **INERTE nas duas** |
| `continuous_length` | 1 \| 2 | dependente — correcto mostrá-lo |
| `width_scale` · `length_scale` | 9 \| 9 | vivos |

**Gate verde, e porquê.** `every_preset_gate_lists_exactly_the_grammars_that_read_that_knob` —
o laço é `for (i, p) in ls::PRESETS.iter().enumerate()` (8 moldes) e o `Custom` está
**explicitamente fora**, com o argumento *«no Custom a gramática é a que o artista escreveu, então
esconder ali é adivinhar»*. ⚠️ **Esse argumento é verdade para uma gramática ESCRITA À MÃO e falso
para a GUIADA, que o app deriva e conhece exactamente.** A bancada tem o mesmo buraco: o caso
`Custom` dela usa `DEFAULT_RULES` em `MODE_GRAMMAR`, nunca o guiado.

**Gate que falta.** Uma célula `Guided` no corpus dos dois — e o `Custom` deixa de ser uma
isenção em bloco: ele é *«escrito à mão»* (isento) **ou** *«derivado dos sliders»* (medível).

### 1.3 ⛔ Em `Geometry = Segments`, **quatro** controlos de folha são pintados e inertes ✅ verificado

**Mecanismo.** `motion_lsystem_gen.rs:318` faz `continue` para todo nó cujo `geometry != Branches`
⇒ o `LeafLook` nunca é construído, e ele é o **único** leitor de `leaf_front`, `leaf_size`,
`leaf_size_jitter`, `leaf_pos_jitter`. Os `ParamGate` por `GEOMETRY` cobrem **quatro** params
(`tip_taper` + os três nomes de folha) e mais nada.

Medido: 9 moldes × 4 params = **36 células, 36 inertes**, os 4 pintados.

⚠️ **A 2.ª leitura é o que impede a cura errada:** `leaf_effects`, `leaf_first_level`,
`leaf_angle` e `leaf_spread` estão **VIVOS** em `Segments` (o `turtle` escreve o `TINT_MASK` e o
`mark_grow`/`rot` ali) ⇒ ⛔ **nunca** esconder a secção *Leaves* inteira.

⚠️ **E a leitura que a torna pior:** em `Segments` os três campos *Leaf (J/K/M)* estão escondidos
(correctamente) ⇒ a secção mostra **8 knobs e nenhuma forma de nomear uma folha**.

**Gate que falta.** Quatro `ParamGate { when: GEOMETRY, values: &[GEOMETRY_BRANCHES] }`, mais um
censo irmão do de moldes com o `geometry` como sujeito.

### 1.4 ⛔⛔ A cena `=12` **ensina o contrário** do que faz ✅ verificado

⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente* — a lei
já está escrita no `CLAUDE.md` §5.0, e esta é uma ocorrência dela.

**Três textos**, todos no caminho que o Enio lê:
1. `motion_object_smoke_leaf.rs:17-19` — *«o `Leaves In Front` só tem sujeito quando a folha é uma
   FORMA DESENHADA»* (texto pré-terceira-média);
2. `motion_object_smoke_leaf.rs:30-33`, dez linhas abaixo — diz o **oposto**, e está certo;
3. `motion_object_smoke_leaf.rs:81` — o `eprintln` imprime *«UMA ARVORE COM FOLHAS DESENHADAS»* e
   a folha é `Sprite::atlas`.

E a membrana (`motion_lsystem_leaves.rs:278`) avisa *«as imagens desenham-se sempre ATRÁS dos
galhos — deixe o knob em 0»* **enquanto metade da copa está à frente**. O gate
`an_image_leaf_can_be_drawn_in_front_of_the_branches` está VERDE e afirma o contrário do aviso.

**Gate que falta.** `motion_lsystem_says_tests.rs` tem um gate por cada um dos outros três `say_*`,
com a metade *«tem de CALAR no caso vizinho»*. Para este **não há nenhum** — com ele, a wave da
terceira média tê-lo-ia posto vermelho no mesmo commit.

---

## §2 — PERFORMANCE: o caminho de OMISSÃO regrediu

### 2.1 ⛔⛔⛔ Uma planta PARADA re-deriva a geometria inteira, todo quadro ✅ verificado

**Mecanismo.** O nó é `Effect::Pure` **exactamente para o cook o memoizar** — o `//!` dele diz que
`Temporal` *«mata o memo — recozeria a reescrita exponencial a 60 fps»*. O modo `Branches` — o
**default** desde `df45c95aa` — deita isso fora: `motion_lsystem_gen.rs:327-328` chama
`ls::skeleton()` e `ls::branch::branches()` **incondicionalmente**, e o memo só é consultado 74
linhas depois (`handle_for`, `:401`).

Medido (load `0,26`, mediana, mesmo processo, gramática do Bush):

| gerações | elementos | `skeleton` | `branches` | **por quadro** |
|---|---|---|---|---|
| 3 | 157 | 0,006 | 0,007 | 0,013 ms |
| 4 | 782 | 0,024 | 0,028 | 0,052 ms |
| 5 | 3 907 | 0,148 | 0,162 | 0,310 ms |
| 6 | 19 532 | 0,594 | 0,650 | **1,244 ms** |

⭐ O `Segments` (o default anterior) é **plano em ~0,001 ms em qualquer tamanho** — é o memo do
cook a acertar. **A razão não tem tecto: ela é o tamanho da planta.**

⚠️ **Não é um one-liner:** as âncoras e as folhas também consomem o esqueleto todo quadro (a
aparência da folha muda sem a geometria mudar), então mover a consulta do memo para cima obriga a
decidir o que é memoizável em separado.

**Gate que falta.** `a_static_plant_derives_once_and_the_next_frame_derives_nothing` — contador de
derivações; publicar 2× sem mexer em nada exige `0` na segunda. **Hoje falha.**

### 2.2 ⛔ A arrastar o `Generations`, uma planta grande passa o orçamento

Medido: a `78 124` ramos, quadro quente `6,18 ms`, **quadro animado `17,88 ms`** — contra os
`16,67`. Uma planta, sem folhas, sem GPU. E a cena convida ao gesto (*«arraste devagar»*).

⚠️ **Uma geração fraccionária deriva a geração SEGUINTE** (`frac > 0 ⇒ n = floor + 1`): tirar o
slider do inteiro custa **5×** a planta nesta gramática. ⇒ o tecto efectivo **enquanto se anima** é
uma geração ABAIXO do autorado, e o `MAX_MODULES` foi escolhido de uma tabela de derivações
**inteiras**.

⛔ **A justificação do `MAX_MODULES` está desactualizada nos dois lados:** a tabela diz `6,47 ms` =
*«38,8 % de um quadro»*, mas (i) a animar, todo quadro é fraccionário ⇒ 2,2–2,9× aquilo, e (ii)
desde que `Branches` é o default o quadro paga também `branches()`. A bancada que escolheu o número
mede **só `probe_build`**, declarando no doc que isso é *«exactamente o que um quadro paga»*.

### 2.3 ⛔⛔ O aviso «uma vez só» sai **a cada quadro**, e o conjunto cresce para sempre

**Mecanismo.** `SAID` é chaveado pelo `ribbon_key`, que mistura os 31 params **pelos bits**,
`generations` incluído. Com o `Generations` animado a chave é nova todo quadro ⇒ o `eprintln!` sai
60×/s e o `BTreeSet` cresce **~280 bytes por quadro, sem varredura**.

Reproduzido: 320 quadros ⇒ **320 impressões**.

⚠️ É a forma que o doc do `VecPathStore` já nomeia: *«um cache cuja chave pode mudar a 60 Hz não é
um cache — é uma fuga com memória»*. E é o mesmo defeito de **canal largo demais** que esta linha
já pagou no `falloff`: a pergunta é *«a gramática mudou?»* e o que ela lê são 31 params.

### 2.4 ⛔ O memo das fitas repete a chave e **não evita uma única reconstrução**

Na cena `=108`, a planta com LFO dá **175 chaves distintas em 240 quadros** (27 % de repetição) —
e **`0 de 237` quadros evitaram uma reconstrução**, porque o `VecPathStore::sweep()` despeja toda
chave não pedida **naquele mesmo quadro**. Quando o LFO volta a um valor já visto, a entrada já foi
varrida. *Uma chave que repete e um memo que acerta são coisas diferentes, e só a segunda se mede
contando construções.*

### 2.5 ⚠️ A leitura de pixels da folha lê **o atlas inteiro** — 268 MB, e guarda-os para sempre

`ATLAS_DEFAULT_SIZE_PX = 8192` ⇒ `8192² × 4 = 268 MB` de staging + um `Vec<u8>` de 268 MB retido
pela vida do processo, numa cópia GPU→CPU **síncrona**. O doc diz *«lê-se INTEIRO, e uma vez só»* —
o que não diz é que «inteiro» é um quarto de gigabyte, independentemente do tamanho da folha. Nunca
é varrido nem invalidado (pintar na folha serve pixels velhos para sempre).

⚠️ **Não medido com relógio** (precisa de adapter) — são contagens e constantes lidas do código.
E **não é gateável hoje**: `resolve` recebe `gpu`/`atlas`/`individual` directamente, sem costura.

### 2.6 ⚠️ `Growth < 1` custa **2,6×–31×** a derivação

`measure_ratio` corre **duas derivações completas em gerações FIXAS 4 e 6**, independentemente do
`Generations` da planta. Bush `0,019 → 0,589 ms` (31×); soma dos oito `0,24 → 1,50 ms`
(1,4 % → **9 %** de um quadro). E o `Growth` é justamente o knob feito para ser arrastado.

### 2.7 ✅ Os `55×` e os `13×` dos commits de perf **AINDA SÃO VERDADE**

Re-medidos na fixtura dos próprios commits (carga 4,40): publicar quente `0,244 ms` (afirmado
`0,245`), reconstrução completa `0,614 ms` (afirmado `0,737`), fitas por quadro quente **`0`**,
frio **`3 124` exacto**.

⚠️ **Nota metodológica que vale por si:** a `load 1,27` os números absolutos saíram **~25 % maiores**
que a `load 14,9`, enquanto as **razões** se mexeram menos de 2 %. *Neste hardware a razão sobrevive
à carga e o relógio absoluto não sobrevive nem à calma.*

---

## §3 — O QUE PODE DESTRUIR TRABALHO

### 3.1 ⛔⛔⛔ `SIGABRT` por estouro de pilha: **~14 KB de texto matam o editor** ✅ verificado

**Mecanismo.** `ph2d-expr-parse` é descida recursiva **sem tecto de profundidade**; o `Expr` é uma
cadeia de `Box`, então o `eval` **e o `Drop`** também recursam.

Reproduzido: `A(s) -> F( ((((…7 000×…1…)))) )` ⇒ `fatal runtime error: stack overflow, aborting`,
**exit 134**. Bissectado: 6 500 passa, 7 000 aborta.

⚠️ **Cabe num paste e cabe num `.ph2dproj`: abrir o ficheiro mata o editor, e não há como o
reabrir.** ⚠️ E o parser é **partilhado** — o `motion.expression` e a timeline têm a mesma porta.

Irmão, mesmo modo de falha, outro sítio: `derive::right_match` é recursivo por `[` e aborta a
~150 000 (só alcançável por regra com contexto direito).

**Gate que falta.** Um tecto de profundidade **no `ph2d-expr-parse`** com o número medido, e um
gate que passe 10 000 `(` e exija `Err`, não abort.

### 3.2 ⛔⛔ Trocar de molde e voltar a `Guided` **apaga o fio** e esconde controlos vivos

**Mecanismo.** Os `ParamGate` novos têm `when: PRESET`. O `preset` só é **escrito** em `Grammar`,
mas é **lido** em qualquer modo, e nada o repõe ao voltar. Em `Guided` a gramática derivada contém
`!` **e** `length_scale` ⇒ os dois estão **vivos** (10 saídas distintas cada) e o painel esconde-os
porque `preset = Koch`.

E `Visibility::mode_hides` responde `true` ⇒ o `drop_hidden_drivers` **solta o fio** na primeira
edição seguinte, com um toast a dizer *«this shape has no such control»* sobre um controlo que a
gramática LÊ. Medido pelo despacho real: `fio=true → escolher Koch → fio=false`.

⚠️ **Em `Guided` não há gesto que devolva `preset = Custom`** — a row do `Preset` não é pintada.

### 3.3 ⛔ Duas plantas iguais com folhas diferentes **partilham a corrente**

**Mecanismo.** `ribbon_key` itera `MANIFEST.params` (31 f32) + axioma + regras. Os **três nomes de
objecto de folha são text params** e não estão no manifesto — mas a shell lê-os para construir a
corrente e publica-a sob essa chave. ⇒ duas plantas com os mesmos números e a mesma gramática, uma
com *Leaf (J) = folha* e outra *= flor*, partilham chave e a segunda **sobrescreve** a primeira.

⚠️ O doc-comment da própria função declara a invariante que ela quebra: *«um param novo entra na
chave sozinho»*. A lista sai do manifesto — e o manifesto é **f32-only** por contrato congelado ⇒
**todo canal de texto tem de ser acrescentado à mão**, e dois dos cinco foram.

### 3.4 ⛔ `NaN`/`Inf` saem para a corrente a jusante, de uma gramática que o parser ACEITA

`F(s/0)` ⇒ **zero queixas**, `P = [NaN, inf]`. E por fio, **12 dos 27 knobs** põem não-finitos na
saída. ⚠️ A linha que interessa: **`angle = 1e30` (finito!) produz 120 `NaN`**, porque o heading
acumula até `inf` e o `frac` o converte.

O crate já sabe desta classe — guarda `seed`, `generations`, `shape::count` e
`growth_generations`. **Quatro de vinte e sete.**

### 3.5 ⛔ `MAX_GENERATIONS = 32` é cerca de PAINEL; o modelo aceita `65 535`

O doc dele diz-se *«só para a caixa não aceitar um `1e9`»*. Mas o valor **conduzido por fio** entra
cru (`resolved_params`/`EvalCtx::param` não fazem clamp nem `is_finite`), e `generation_plan` coage
a `u16::MAX`. Medido: 100 000 módulos × 65 535 passagens ⇒ **> 120 s numa cozedura**. A 12 s o KDE
oferece forçar o encerramento (precedente registado).

⚠️⚠️ **E a casa TEM a porta certa:** `ph2d_nodegraph::node::param_as_count(value, max)`, cujo doc
diz *«não-finito → 0 … para um valor de cena corrompido nunca poder disparar um abort»*.
**17 crates de nó usam-na. Esta usa ZERO vezes.**

---

## §4 — OS GATES DESTA LINHA: 19 de 24 mutações morreram, 5 sobreviveram

⭐ **A maioria é genuinamente forte.** `a_dropped_rule_says_why` é um gate-modelo (a queixa nasce no
mesmo `return Err` que descarta, e está amarrada à contagem de módulos, o que mata o `Vec::new()`
na porta). As listas de visibilidade por molde são **oráculo verdadeiro**, não espelho.
`grows_by_refining` é um estrangulamento bem coberto — uma mutação de duas linhas mata **9 testes
em 2 ficheiros**. Os controlos de filtro existem e são apertados.

Os cinco sobreviventes, cada um numa camada diferente:

### 4.1 ⛔ `only_the_lsystem_rules_box_can_carry_a_complaint` **não pode reprovar**

**Mecanismo: fixtura sem o fenómeno.** O teste cria um `motion.expression` e **nunca lhe escreve
fórmula**. Com texto vazio a porta sai em `queixas.first()?` → `None` **independentemente das
guardas**. As três remoções (só o tipo, só o param, ambas) ficam **verdes**.

Que o fenómeno é real: `grammar_complaints("sin(t)*2")` devolve **1 queixa** (`NoArrow`). E os
axiomas dos oito moldes são acusados **8/8** ⇒ perder a metade `param != RULES_PARAM` põe uma linha
vermelha **falsa** debaixo da caixa *Axiom* em todo estado normal.

### 4.2 ⛔ O gate de PIXEL mede a **linha reservada**, não a tinta

A altura é escrita por `y += ROW_H_PX + row_gap`, que é **outra linha** que não o
`paint_text_elided`. Apagar a pintura inteira deixa o gate **verde**.

⚠️ Morde na promessa do próprio ficheiro (*«nenhuma das duas prova que ela chega a pixel»* — e
propõe-se ser a terceira). O arnês não permite mais hoje: `MockPanelHost` expõe `store()`,
`paint()` e `painted_rect()`, e **nada de texto**. ⇒ ou se põe um observador de texto no testkit,
ou o doc-comment passa a dizer que a régua é **espaço reservado**.

### 4.3 ⛔ `tip_taper` não tem gate em camada nenhuma

A **lei** tem gate (`branch_tests.rs` chama `branches(..., taper)` directamente). A **entrega** não:
substituir `get(ls::param::TIP_TAPER)` por `0.0` em `motion_lsystem_gen.rs:336` e **os 57 testes de
`lsystem` nos bins passam**.

⚠️ **E a isenção que o permite é falsa em metade:** `SHELL_SIDE` justifica-se com *«a sonda
estouraria neles»* — medidos, **três não estouram**, e o `leaf_effects` dá **2 saídas distintas**,
ou seja o `build` **lê-o** e ele não é «shell-side» de todo.

### 4.4 ⛔ O `Seed` não alcança os dois sorteios de folha nem o frente/trás

`LeafLook` não tem campo de seed; `is_in_front` e `LeafLook::at(i)` usam só `hash01_lane(i, lane)`,
onde `i` é a **identidade da marca**, não o param. ⇒ o botão *re-roll* do `Seed` não muda uma folha.

⚠️ **E a isenção do `seed` no meu gate invoca exactamente esse alcance:** *«ele é também semeado
pelo `Leaf Size Jitter` e pelo `Leaf Pos Jitter`»*. **A promessa não tem leitor.**

### 4.5 ⚠️ O censo do tecto de linhas é CEGO ao `ParamGateAbove`

`row_census()` mede logo após `add_node` ⇒ `tropism = 0` ⇒ o *Tropism Direction* está escondido.
Medido: fábrica **32** rows; com `Tropism = 30` (um gesto de slider) **33** — que é o
`MAX_PARAM_ROWS` exacto. ⇒ **a folga real é zero**, e o próximo param faz o `.take(MAX_PARAM_ROWS)`
descartar a última linha **em silêncio**, com o gate verde.

---

## §5 — NOTAS QUE ENVELHECERAM

### 5.1 ⛔ Um doc-comment com **cinco** afirmações desmentidas pelo código à volta dele

`presets.rs:128-169`, sobre `TREE_WITH_LEAVES`:

| afirma | medido |
|---|---|
| *«Texto PRÓPRIO, e não o `DEFAULT_RULES`»* | **byte-idênticos** |
| *«O `DEFAULT_RULES` fica INTOCADO… obrigaria a pô-la na derivação guiada»* | as duas coisas aconteceram |
| *«A âncora é VISUALMENTE NEUTRA — `0` posições novas»* | verdade em 7; **falsa no `Wild`** (32 novas) |
| *«O preço é a CONTAGEM, e é ~3× (`32 → 94`; `256 → 766`)»* | **`32 → 63` (1,97×)**, `256 → 511` |
| *«A âncora só entra em Tree, Fern, Wild; Bush/Weed/Koch/Dragon ficam de fora»* | **os oito** a têm |

⚠️ O 4.º tem **irmão a discordar dentro da mesma crate** (`lib_marks_tests.rs` diz `~2×`, que é o
número certo); o 5.º tem irmão a dizer *«a medição desmentiu-a»*.

### 5.2 ⛔ Cinco de oito comentários de `leaf_first_level` não se reproduzem

O do **`Bush`** carrega **os números do `Weed`** (`121`/`96` contra `156`/`48` reais); o do
**`Dragon`** diz `512` marcas e são **2 048** (4×); o do `Fern` erra a faixa e o total.

Consequência prática: quem lê o comentário do `Bush` espera que `First Level = 3` mostre **96 de
121** folhas (79 %); o produto mostra **48 de 156** (31 %).

**Gate verde, e porquê.** `no_preset_silences_its_own_leaves` afirma só `!marcas.is_empty()` e
`vivas > 0`. **Uma contagem 4× errada não move nenhum dos dois predicados** — é a mesma cegueira
que o doc-comment desse gate acusa no gate anterior, reaparecida um nível acima.

### 5.3 ⚠️ Outros números stale

- `growth.rs`: a faixa *«`1,053` .. `1,154`»* é a de ANTES do `[J]`, e o `1,053` era o `Wild`
  (hoje `1,0992`). O piso é o `Sprig`, `1,0971`. ⚠️ E o `Wild` estava a **0,3 %** da guarda
  `r > 1.05`.
- `derive.rs`: *«varridas 8 100 combinações… `3,5 %` de separação»* — sobre as faixas que o `ui.rs`
  **de facto oferece** (33 750 células) o máximo do guiado é `1,9028` contra `1,4791` do Dragon:
  **8 100 células (24 %) ficam ACIMA**. ⚠️ A conclusão do doc **sobrevive e sai reforçada**; o que
  erra é o número que a justifica, e erra a subestimar.
- `presets.rs`: a tabela «ANTES» falha em 2 de 8 (`Wild` 3,7→3,13; `Sprig` 3,4→3,49). O `963×`
  continua certo.
- `lib.rs`: o doc-comment de `DEFAULT_RULES` **soletra a regra sem a âncora que a constante tem**,
  4 linhas acima dela, e diz *«três coisas»* onde hoje são quatro.
- `probe.rs`: a ilustração `16 384` vs `32` descreve o parser a falhar **ABERTO** — modo de falha
  **curado**. A conclusão mantém-se; os dois números não se reproduzem em gramática nenhuma.
- `CLAUDE.md` §5: a **Data Source** está marcada *«nunca começada»* e **esta linha construiu-a**
  (`627f8b1aa` criou `ph2d-node-source-table`, `ph2d-node-value-table` e `ph2d-table`).
  *Uma ausência afirmada sem olhar a API é um palpite com cara de medição* — e aqui a API foi
  escrita pelo próprio autor da nota, no mesmo ramo.

### 5.4 ⛔ A tabela `⛔ Recusas MEDIDAS` tem uma recusa **revogada pelo código**, viva

- *«Âncora no `DEFAULT_RULES`»* — **viva, sem rasura**, e a revogação dela está **duas linhas
  abaixo** (*«Deixar o `DEFAULT_RULES` sem âncora… ⛔ Revertido no mesmo dia»*). As duas discordam
  também sobre o preço (~3× contra ~2×; a medição diz 2×).
- A mesma recusa está escrita uma **terceira** vez, também viva, no §5.1 do doc 95.

⚠️ **O §5.0 do `CLAUDE.md` manda consultar esta tabela antes de propor qualquer mudança de
desenho.** Uma recusa viva sobre trabalho já feito faz a próxima janela recusar-se a fazer o que já
está no `main`. O precedente de formatação existe **na mesma tabela** (a do `Tropism Angle`,
rasurada em §14.3) — é a irmã que ficou por rasurar.

### 5.5 ⚠️ Os pickers de folha oferecem **chaves de máquina** como *"Drawn shapes"*

`source_options` filtra só o namespace reservado (`$`). O `publish` publica cada planta sob
`ribbon_key(...)`, que começa por `"lsysrib"` e **embute a gramática crua**. Medido: a chave está
entre os chips. Na cena `=108` são **5 chips de lixo**, e clicar num planta a **própria planta**
como folha.

⚠️ **É família pré-existente** (o `motion.shape` publica `"shape:…"`) — mas as três rows de picker
são novas desta linha.

---

## §6 — O QUE FOI MEDIDO E ESTÁ SÃO

Para o «nada encontrado» ser distinguível de não ter procurado:

- **`HashMap`/`HashSet`: ZERO** no crate e nos 9 ficheiros da shell — a espinha do determinismo
  está intacta. 8 corridas da mesma gramática estocástica ⇒ checksum idêntico.
- **As cercas do `//!`**: `Effect::Pure` ✓ · `lowerings = [Cpu]` ✓ · `{ } . & ^ / \ $ , ~ @` todos
  **medidos inertes** (12/12) ✓ · sem índice de cor ✓. ⚠️ **As três últimas são verdade por
  ACIDENTE** — nada as gateia.
- **`MAX_MODULES` é o teto exemplar do crate** — no tecto o processo pica em `VmHWM = 25 196 kB`,
  e a afirmação *«não é memória: 6,3 MB»* confirma-se com folga. **`WIDTH_DIRECTIONS = 16`** é o
  único limite com tabela de escolha **e** A/B no produto — é o modelo.
- **A linha de queixa** (`TextRow.problem`) não dessincroniza rows nem rouba clique — verificado
  nas duas metades, com controlo de filtro próprio.
- **`Leaf Angle`/`Leaf Spread` não são projectados fora**: o `turn` sobrevive nos dois modos de
  `Orient`, e a shell lê a coluna `rot`.
- **Chip → intenção** está gateado nas duas condições. ⚠️ Falta um gate que **atravesse** chip →
  folha desenhada; é lacuna de cobertura, não defeito medido.
- **`Reads::of`** derivado do texto e gateado ✓; **`PRESETS_GROWING_BY_TIP`/`_BY_REFINEMENT`**
  concordam com `probe_grows_by_refining` nos oito ✓; a **saturação em `78 124`** reproduz-se ✓;
  o painel **`28 → 20`** controlos bate com a bancada linha a linha ✓.

---

## §7 — A ORDEM RECOMENDADA

| # | achado | por quê primeiro |
|---|---|---|
| 1 | **§2.1** planta parada re-deriva todo quadro | é o default, cresce sem tecto com a planta, e desfaz a razão de o nó ser `Pure` |
| 2 | **§1.1** `Wild` 15 % menor | o dono vê, e desfaz metade de uma cura já paga |
| 3 | **§1.2** dois knobs mortos de fábrica | é o primeiro ecrã de um nó novo, e é defeito da wave de ontem |
| 4 | **§1.4** a cena ensina o contrário | lei do §5.0, e é o caminho que o dono lê |
| 5 | **§3.1** abort por texto | um `.ph2dproj` assim é irrecuperável — mas exige entrada patológica |
| 6 | **§1.3** 4 knobs mortos em `Segments` | cura conhecida (4 gates), risco zero |
| 7 | **§4.1–4.5** os cinco gates fracos | cada um deixa passar o defeito que diz guardar |
| 8 | **§5.1–5.5** notas envelhecidas | custa uma janela à próxima linha, e a §5.4 pode fazê-la desfazer trabalho |

⛔ **Nada disto foi consertado.** O protocolo desta casa é listar antes de curar, e a lista é esta.

---

## ⛔ Recusas MEDIDAS (desta auditoria)

| Item | Motivo |
|---|---|
| Esconder a secção *Leaves* inteira em `Segments` | ⛔ **4 dos 8 knobs dela estão VIVOS ali** (`leaf_effects`, `leaf_first_level`, `leaf_angle`, `leaf_spread` — o `turtle` escreve-os no esqueleto). A cura é 4 gates, não um |
| Um limiar no `Seed` sobre o `Leaf Spread` | ⛔ Ele é também semeado por dois params que a **shell** lê; um limiar sobre um dos três apagaria o knob para quem usasse os outros dois |
| Gates de RAZÃO ou de RELÓGIO para o §2 | ⛔ Entram na família de flakes de carga do §5.0 — reprovam sob fan-out e passam sozinhos. **Todos os gates propostos aqui são de CONTAGEM** (derivações, fitas construídas, chaves distintas, tamanho de conjunto) |
| Medir o §2.5 com relógio | ⛔ Precisa de adapter de GPU; os números daquela secção são **contagens e constantes**, e estão marcados como tal |
| Filtrar params do `ribbon_key` em silêncio | ⛔ O doc da função nomeia o modo de falha oposto (*«uma chave que ignorasse um param faria a shell servir a geometria ANTIGA»*). A cura tem de ser uma **lista declarada com censo** |

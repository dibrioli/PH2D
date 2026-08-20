# HANDOFF DE CONTINUAÇÃO — `line/motion-value` · **o PLANO e as tarefas em aberto**

**Data:** 2026-08-19 · **Para:** o próximo agente desta linha · **Worktree:**
`/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`

> ⚠️ **Isto não é um handoff de integração** (aquele é o
> [`…_FECHO_2026-08-18.md`](HANDOFF_INTEGRACAO_line_motion_value_FECHO_2026-08-18.md), já
> integrado). Isto é o que a próxima janela precisa para **continuar a implementar**: onde
> parar de reconstruir, o que fazer a seguir, em que ordem, e as leis que esta linha pagou
> para aprender.

---

## §0 — Os primeiros três comandos, antes de abrir arquivo nenhum

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && pwd && \
  git branch --show-current && git log --oneline -1
```

⚠️ **A janela abre na raiz (= `main`) e o MESMO caminho relativo existe nas duas árvores** —
editar a errada compila e commita **sem erro**. E a `cwd` do Bash **volta ao primário entre
turnos**: prefixe **todo** comando com o `cd`
([`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).

Depois:

```bash
bash scripts/hw-profile.sh          # tier → MODO (aqui: workstation ⇒ Modo L)
cargo check --workspace | tail -3   # a base compila?
python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"   # o placar VIVO
```

---

## §1 — Onde a linha está

| | |
|---|---|
| base | `main` `ee1432203` — a linha foi **reaberta por fast-forward** depois de a integração ter entrado |
| commits desta janela | `git log --oneline main..HEAD` (⚠️ não se pina aqui — o commit que escreve o número muda o número) |
| estado | **verde** — `fmt` 0, `clippy` 0, LOC 0, suítes das crates tocadas 0 falhas |
| smoke pendente | **`=69`** (a família transform) — os anteriores foram aprovados pelo Enio |

⚠️ **Dois smokes de ontem nunca foram vistos pelo Enio** e já estão no `main`:
`=58` (re-smoke depois da correção do relógio que expirava) e `=59` (a porta de tempo).
Se ele reportar algo sobre eles, o mecanismo está no handoff do FECHO.

### §1.1 — O que as JANELAS de 2026-08-19/20 fecharam (a conferência foi de **P1 59 → 28**)

⚠️ **O saldo de P1 não é o número de células fechadas**, e é bom que não seja: a janela fechou
**vinte e oito** e ABRIU **duas**, ambas por medição no meio de uma wave. Uma conferência que só
descesse seria uma que parou de olhar para os lados.

| grupo | entrega | smoke |
|---|---|---|
| **S** | o 2º `fx.glow` mudo passa a **avisar**; o aviso cego ao tipo de coluna **dissolveu-se** por medição e virou teste de classe | — |
| **T** | ✅ **folha 17 fechou** — o `motion.integrate` **declara `substeps`**, e o motor já existia (as rotas (c)/(d) da célula caíram em 12/08 sem ninguém reconferir). Achado no caminho: a palavra `substeps` tinha **dois donos** e o app corria as duas leis — a corda caía **4,8× menos** que os gates dela medem | `=61` ✔ · `=52` ✔ |
| **T** | ✅ **folha 15 fechou** — o `value.switch` de N entradas é **exprimível** (`2·⌈(N−4)/3⌉+1` nós, per-elemento preservado); e a ramificação MORTA de um switch ganhou badge | `AUTOFIX=8` ✔ |
| **U** | `source.shape`: **sweep/start/inner** + **raio por canto/smoothing**; e o `corner` deixou de ser da caixa para ser do **catálogo** (4 → **38** espécies) | `SHAPE_SMOKE=3` ✔ |
| **U** | ⛔ **três refutações medidas**: o *trim/dash* (a cura, não o item — 42 de 47 formas são fechadas), o *`fill_rule`* (a estrela citada **nunca** auto-intersecta) e o *Pick Instances* (já existe: `combine` + `duplicator(pick)`) | — |
| **V** | folha 08: `motion.sort` ganha **direção arbitrária** (`axis_angle`) e a **chave como CAMPO** (porta `weight`, modo 5) | `=63` ✔ |
| **V** | folha 08: ✅ **o `reindex` do `motion.sort`** — a ordenação não chegava ao efector indexado. **Aberta por um smoke do Enio**, não por uma tabela | `=63` ✔ |
| **V** | 🆕 **duas células ABERTAS por medição** (folha 08 `motion.cull` · folha 05 `motion.mirror`/`kaleidoscope`): a mesma lei da identidade nos vizinhos que encolhem e que crescem | — |
| **V** | folha 08: o **taper por cópia** do `motion.clone` e o **peso por entrada** do `motion.mixer` — a folha desce a **3 P1**, e os três que sobram têm espécie declarada | `=64` |
| **V** | folha 08: o **`followMouse`** — fechado por um nó novo (`value.cursor`, o **primeiro do repo com duas saídas**) mais a rota do param dirigido, e **não** pelo toggle que a célula pedia. A folha desce a **2 P1** | `=65` |
| **V** | **folha 10 (`field.*`): QUATRO células**, e vieram juntas porque duas partilhavam uma causa — o `clamp` inline do `Add` era o defeito, e era ele que tornava o `Average` inexprimível. Mais o **anel** (`inner_radius`) e a **força com sinal** (`strength`). A folha desce de **6 para 2 P1** | `=66` |
| **V** | 🔬 **UM INSTRUMENTO NOVO, e é o achado do dia:** `conferencia_vs_manifesto.py` cruza a coluna «params hoje» de cada célula com o MANIFESTO do nó (sonda `measure_node_params`, derivada do registry) e sai vermelho quando discordam. Ele acusou **31 células ABERTAS em 16 nós** a descrever um nó que já mudou | — |
| **V** | folha 01: **cinco células fecharam e QUATRO não custaram código** — `mode`/`spacing` do `distribute_curve`, `align` do `distribute_radial`, `size_random` e `dir_mode` do emitter já tinham shipado. A quinta é o **`probability`** do emitter, construído hoje. A folha desce de **6 para 1 P1** | `=67` |
| **V** | 🔬 **e a SEGUNDA passagem do instrumento**, com o sinal FORTE (o param que a coluna «default que reduz» nomeia já está no manifesto): 7 acusadas, **2 verdadeiras** (`probability` do emitter — uma SEGUNDA célula pedia o mesmo — e `lacunarity` do `motion.noise`), 5 falsos positivos **todos da mesma forma**, agora tabelados no próprio instrumento | — |
| **V** | **folha 04 (deformers): CINCO células numa wave** — a `direction` do `bend`, o `radius` e o `profile` do `twist`, o `radius_y` do `spherize` e o `mode` (Fit/Keep Length) do `spline_wrap`. As cinco são adição de param com default literal. A folha desce de **6 para 1 P1**, e o que sobra é de outro tamanho (as arestas de Bézier + patch de Coons do `four_point_warp`) | `=68` ✔ |
| **V** | ✅ **folha 05 (transform) FECHOU: cinco células, cinco nós** — `space` (World/Local) do `move` · `use_falloff_y` do `scale` **+** `mask_channel` do `falloff` (uma célula, dois nós) · `flip_rot` do `mirror` · `reindex` do `mirror` **e** do `kaleidoscope` · `carry_rotation` do `orbit`. ⚠️ **As cinco eram a MESMA pergunta em cinco lugares**: o nó sabe o que cada ELEMENTO é (orientação, posição na lista, máscara) e respondia como se a lista fosse um bloco. Os 3 P2 que sobram não são dessa família | `=69` |

⚠️ **E uma correção de GEOMETRIA em `ph2d-vec-scene`, que o smoke do Enio devolveu:** a
borda que fecha uma fatia **abaulava** 19–25% do raio, porque o handle do arco sobrava na
ponta — e era o mesmo defeito que fazia o motor de quinas não ver quina nenhuma ali. Os 387
testes daquela crate passaram sem uma edição de asserção.

---

## §2 — O que NÃO se reconstrói (feito e integrado)

- **A porta de tempo** em `oscillator`/`noise`/`wiggle` — porta VALUE opcional, índice **1**,
  desligada ⇒ `ctx.playhead()` bit-a-bit, ligada ⇒ **um relógio por elemento**. CPU + GPU.
- **`TimeMode::Curve`** (índice 5) no `ph2d-nodegraph`, com a janela a **REPETIR** — ele é o
  superset cíclico do `Loop`/`PingPong`.
- **`motion.drive`**: canais `Size X` (10) e `Size Y` (11).
- **`value.attribute`**: os chips `Position X` · `Position Y` · `Radius` · `Angle`.
- **`motion.noise`**: o **espaço do campo** — `rotation` + `uniform`/`scale_y`. ⚠️ O *offset*
  e o *scale uniforme* **não são params de propósito**: saem da composição e do próprio
  `scale` (medido em `measure_noise_space`).
- **A folha 06 FECHOU** — 0 P0, 0 P1, 12 ✅, 18 P2.

---

## §3 — O PLANO, em quatro grupos, nesta ordem

> A regra de cadência é do Enio: **implementar em GRUPOS de nós, e a cada grupo UMA cena de
> smoke**. A próxima cena livre é a **`=61`** — ⚠️ e esse número se **CONTA lendo o `match`**
> do [`motion_state_demo_router.rs`](../../../shells/desktop/src/motion_state_demo_router.rs),
> nunca esta linha (ela envelhece no primeiro grupo).

### Grupo S — os DEFEITOS, antes de qualquer knob

Um defeito silencioso vale mais que uma feature, e há **dois** nomeados e medidos:

1. ⛔ **Um SEGUNDO `fx.glow` é silenciosamente INERTE** (folha 11) — `from_graph` faz
   `.find(…)` e o segundo nó nunca corre. O artista empilha dois glows, vê um, e conclui que
   o parâmetro não funciona.
2. ✅ **FEITO em 2026-08-19, e a medição DISSOLVEU o item como estava escrito.** O
   diagnóstico de nome de facto não olha o modo — mas o buraco é menor do que a nota dizia:
   as colunas não-escalares do repo são **seis** (`P` · `size` · `vel` · `accel` · `tint` ·
   `sim_d`), **quatro têm chip** e as **duas** restantes estão na denylist `INTERNAL` do
   picker. Cair nele exige digitar à mão um transiente que o picker esconde. ⇒ em vez do
   badge, um **gate de classe**
   (`every_non_scalar_column_is_reachable_or_deliberately_hidden`) que torna a situação
   impossível de nascer. *Um aviso de runtime cura o caso; um gate cura a classe.*

✅ **O Grupo S está FECHADO** (2026-08-19). Nenhum dos dois precisou de cena nova: o smoke do
primeiro é *"empilhe dois glows e veja o badge no segundo"*, e o segundo virou gate.

⚠️ **E a lição do segundo vale para o resto do plano:** o item foi escrito como *"há um
defeito, cure-o"* e a medição mostrou que o defeito era **alcançável só por um gesto que o
produto esconde**. *Meça o TAMANHO do buraco antes de escolher o tamanho da cura* — vale para
todas as células dos grupos abaixo.

### ~~Grupo T~~ — ✅ **FECHADO (2026-08-19): as duas folhas não têm mais P1**

⚠️ **Nenhuma das três células era o que dizia ser, e as três lições valem para o resto:**

1. **`motion.integrate` sub-steps** — o motor já existia (`Cook::substep`, folha 13, 12/08) e
   a célula listava rotas recusadas que **caíram quatro dias depois** sem ninguém reconferir.
   Faltava o nó **declarar**. ⚠️ *Sempre que uma célula diz «inalcançável», datar a afirmação
   e ver o que aterrou desde então.*
2. **`value.unary` Ceil/Round/Truncate** — o `P1` era um **PONTEIRO** para o item do
   `value.quantize`, que fechou em 15/08. Uma contagem que soma ponteiros conta duas vezes.
3. **`value.switch` N entradas** — a nota dizia *"contrato congelado"* e conflaciava duas
   coisas: `&'static [PortSpec]` barra a arity **dinâmica**, não uma lista estática maior (o
   §6 congela a contagem de CAMPOS do `NodeManifest`). E a composição já o exprime, medido.

⚠️ **A frase «mexe no MANIFESTO, leia a lei das portas apendadas» estava neste handoff e
estava ERRADA** — não era preciso mexer em porta nenhuma.

### Grupo U — `source.shape` · **parcialmente fechado (7 → 3 P1)**

✅ **Feito:** sweep/start/inner · raio por canto + smoothing · o `corner` geral (4 → 38
espécies) · e a correção de geometria que o smoke do Enio devolveu (a borda da fatia
abaulava 19–25% do raio). ⛔ **Refutados por medição:** `fill_rule` (a estrela citada nunca
auto-intersecta; só 2 de 43 espécies distinguem as regras, e nelas a actual é a certa) e
*Pick Instances* (já existe: `combine` + `duplicator(pick)`).

**O que SOBRA na folha 14 — três, e cada um com a nota que a medição deixou:**

| item | o que a medição já disse |
|---|---|
| **TRIM / dash** | ⚠️ **A cura da célula está refutada, o item não.** `trim_path` recusa contorno fechado, e **42 das 47** formas da biblioteca são fechadas — as 5 que ele corta (Spiral · Line · Arc · NoteBracket · Brace) **não estão** neste nó. Ligar a função daria dois sliders inertes em 100% do catálogo dele. A cura é a do AE: **abrir o contorno** antes de cortar, e o resultado só se vê com o `stroke_width` (que existe) |
| **`size` é GEOMETRIA, não coluna** | ⚠️ **Meça ANTES de mexer:** o `encode` já compõe `pose = T(P)·R(basis)·S(size)`, então a escala da INSTÂNCIA já existe e cozer a geometria em tamanho 1 daria **um** `VecPath` por descritor em vez de um por valor visitado. O `corner` é fracção, o `aspect` é razão e sweep/start/inner são invariantes de escala ⇒ a imagem seria a mesma. **O que muda é a SEMÂNTICA**: um `motion.drive(Size, mode = Set)` a jusante passaria a APAGAR o tamanho autorado em vez de o multiplicar. Isso é decisão de produto, não refactor |
| **a POSE do objeto não viaja** (`source.object`, *Transform Space*) | por medir |

⚠️ Este grupo **encostou no módulo Vector e foi certo encostar**: a correção de
`cap_arc_ends` é em `ph2d-vec-scene` (foundational, Modo L permite) e os 387 testes daquela
crate passaram sem uma edição de asserção. Contrato congelado (§6) continua a ser parar e
reportar.

### Grupo V — as folhas grandes, por ORDEM DE DEFEITO

`08_stream_utilidade` (8) · `01_distribuicao` (6) · `04_deformers` (6) · `10_field` (6) ·
`02_force` (5) · `11_fx_raster` (5) · ~~`05_transform`~~ ✅ · `03_simulacao` (3) ·
`07_tempo` (3) · `09_cor` (3) · `14_source` (3, na tabela acima).

⚠️ **A contagem por folha se DERIVA** (`python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"`),
nunca se lê daqui: esta lista envelhece a cada célula fechada.

⚠️ **Não ataque por tamanho.** Dentro de cada folha, o que vem primeiro é o que a célula
descreve como **comportamento errado** (o `fx.glow` inerte, o `motion.duplicator` que perde a
escala do ponto, o `motion.step` com limitação auto-declarada), e só depois o que é knob
ausente.

---

## §4 — As VINTE E SETE LEIS que esta linha pagou para aprender

⚠️ **Cada uma destas custou um gate vermelho, um smoke reprovado ou uma medição** — elas não
são estilo.

1. **TRAP 1 SEMPRE, e ele vale para a FOUNDATION também.** Dez células da folha 06
   envelheceram — a última **em metade**: o *scale uniforme* do campo do ruído já era o param
   `scale`, bit-a-bit. E na porta de tempo o orçamento listava **três saídas caras** porque o
   seletor de variante só vê params — e o canal certo (`ColumnAccess::ReadBroadcast` + o
   `const HAS_<porta>_<col>` do codegen) **já existia**. *Meça se o substrato já exprime,
   antes de orçar um mecanismo novo.*
2. **Um ✅ de MECANISMO não é um ✅ de ARTISTA.** A folha 15 marcava as lanes de uma `Vec2`
   como fechadas porque o degrau existia — e não havia gesto que chegasse lá. *Um degrau sem
   chip é inalcançável.*
3. **Uma fixture só prova o que ela CONTÉM.** A fileira de teste do `motion.noise` tem
   `y = 0` em toda peça, e um gate de `scale_y` reprovou sobre código correcto. ⚠️ E a
   rotação **mostra-se** numa fileira, que é o que esconde o buraco de quem só olha para um
   dos dois eixos.
4. **A régua tem de ser a coisa REAL.** O oráculo da cena `=60` subtraiu a *média* para tirar
   a grade; a grade varre 4,48 de mundo em Y e a razão do controle deu **0,21** em vez de ~1.
5. **A DIREÇÃO de um knob pode ser contra-intuitiva — meça-a.** Escala maior num eixo =
   feição **menor** nele (`dx/dy` cai de 0,976 para 0,341). O rótulo tem de dizer o que o
   artista vê, não o que o número sugere.
6. **Nenhum controle pode EXPIRAR.** O `TimeMode::Curve` clampava a janela e a sub-árvore
   congelava para sempre; os gates mediam **dentro** da janela e ficavam verdes sobre produto
   morto. Mesma classe do `fade` do oscilador. *Um gate que só olha para dentro da janela não
   pode ver uma janela que não repete.*
7. **Uma exceção por NÚMERO DE LINHA quebra em silêncio.** A tabela `HAND` do
   `placar_conferencia.py` era chaveada por `(arquivo, nº)`; acrescentar uma linha desalinhou
   tudo e o placar imprimiu **um ✅ a menos**. Hoje a chave é um TRECHO e cada uma tem de
   casar **exactamente uma** linha, senão a ferramenta sai vermelha.
8. **UM GATE MEDE O QUE A CENA PRODUZ; SÓ O OLHO MEDE O QUE ELA MOSTRA.** A `=60` foi
   reprovada **duas vezes** com todos os gates verdes, e a segunda é a lição maior.
   **v1 (posição):** ⚠️ um sprite **sem coluna `size` desenha a `1,0`** (o `SIZE_IDENTITY` do
   shell) contra um vão de `0,32` — o bloco era uma placa sólida · o deslocamento valia
   **1,31×** o vão · e havia **2,5** manchas no bloco inteiro.
   **v2 (tamanho):** o padrão estava lá e o olho não o via — com o bloco a 220 px e 21 pontos
   de lado, os pontos iam de **3,4 px a 9,5 px**. ⚠️ **E a aritmética mostrou que não havia
   saída por números:** para as manchas terem pontos que cheguem **e** os pontos serem
   grandes seria preciso mais janela do que existe. *Duas exigências que puxam a mesma folga
   em direções opostas não se resolvem afinando; resolvem-se trocando o CANAL.*
   **v3 (cor):** `motion.color_ramp` pinta o `tint` a partir do campo lido de volta
   (`value.attribute(Size)` → `value.map_range`). Luminância medida: **0,073 a 0,923**.
   ⚠️ **Os números viraram três gates** — `the_dots_never_touch_so_the_field_is_readable`,
   `the_block_holds_enough_blobs_for_a_rotation_to_read` e
   `the_colour_carries_the_field_all_the_way_to_the_instance` (que lê o `tint` **no
   instance**, depois do lowering). *Ao desenhar uma cena, meça o elemento em PÍXEIS e
   pergunte que canal perceptual carrega o sinal — antes de a mandar.*
9. **UMA SUBAMOSTRA UNIFORME PODE DESENHAR UMA FIGURA QUE NÃO ESTÁ LÁ.** O carimbo dos cards
   (`preview_points`) subamostrava por **passo fixo**, e sobre uma GRADE isso **alia numa
   reta**: com 441 pontos e 21 colunas o passo é 10, `10·k mod 21` anda −1 por linha, e
   **21 das 45** amostras caíam na MESMA diagonal (5 das 21 diagonais tocadas). Os cards
   mostravam um traço enquanto o canvas mostrava manchas — e **o gate que existia ficava
   verde**, porque ele media se o carimbo *abrangia* a grade, e uma diagonal abrange todas as
   linhas e todas as colunas. ⚠️ **A cobertura também não acusa**: dividido em nove ladrilhos,
   o passo fixo enchia os nove. *O que se mede é ESTRUTURA, não cobertura.* Cura: um jitter
   determinístico dentro do balde (4 de 45 na pior diagonal), com o passo fixo como **controle
   negativo dentro do próprio gate**.
10. **PARIDADE PROVA QUE OS DOIS LADOS FAZEM O MESMO, NUNCA QUE O MESMO É CERTO.** A ordem
    do espaço do `motion.noise` (`escala→rotação`) foi escrita, **defendida num comentário** e
    coberta pelo gate de paridade CPU×GPU — e estava **errada**: com `M = R·S` as feições do
    mundo são `S⁻¹R⁻¹(círculo)`, cujos eixos são os do MUNDO, então **a rotação não gira as
    faixas** de um campo anisotrópico. O olho do Enio apanhou (*"não há faixas diagonais"*);
    a paridade nunca poderia. ⚠️ **E o gate «próprio» que existia dava falso conforto**: ele
    construía as duas ordens à mão e provava que elas **diferem** — verdade, e inútil, porque
    nunca perguntou **qual** delas o nó embarcava. *Um gate que prova que duas escolhas são
    distintas não defende a escolha.* Cura: um gate que mede a **direção das faixas**, que é
    a afirmação que o produto faz.
11. ⛔ **OLHE o arquivo antes de escrever nele.** Nesta janela eu sobrescrevi a cena `=51`
   inteira ao criar um módulo com um nome que já existia (`…_demos_space.rs`). Recuperou-se
   com `git checkout --`, mas só porque a árvore estava limpa. *Um `ls` antes do `Write`
   custa nada.*
12. ⚠️ **A suíte inteira é um relógio.** Duas corridas marcaram falhas que eram **carga**
   (`the_cost_of_depth_is_linear_not_explosive` e
   `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`), com `load average` em 14,8.
   Sozinhas passam. *Nada desta workstation vale acima de `load ~5`.*
13. **A porta de tempo é uma COLUNA, não um escopo** — ela não herda a recusa
    `CookError::SequentialInTimeScope`. Se acrescentar uma porta a outro nó, o gate
    `the_time_port_is_a_column_not_a_cook_scope` é o molde.
14. ⚠️ **O oráculo da cena é o que o OLHO lê, nunca o que o nó emite** — e esta é irmã da 10,
    apanhada pelo mesmo Enio no mesmo dia. A cena `=63` tinha quatro gates verdes que mediam a
    **permutação das posições** à saída do `motion.sort`; o que o smoke mostra é a **cor**, e
    a cor vinha do `motion.tint`, que lê a coluna `Index` — que o `sort` levava consigo. As
    três bandas saíam com **a mesma pintura** e a suíte estava verde, porque cada gate media o
    lado certo da costura **do lado errado da fronteira**. *Se a cena existe para ser olhada,
    o gate mede o pixel — ou pelo menos a última coluna antes dele.* O sintoma tem forma
    reconhecível: **a cena mostra a ordem de nascimento** (aqui, a grelha por linhas de baixo
    para cima), porque é isso que sobra quando a operação não alcança o consumidor.
15. ⚠️ **A PEÇA TEM DE CABER NO PASSO, e o passo é o `gap_*` do `motion.grid`.** Uma instância
    sem coluna `size` é desenhada com `SIZE_IDENTITY` = **1,0 unidade de mundo**, e as cenas
    desta conferência autoram passos de **0,12 a 0,6** — ou seja, quase todas desenham peças
    sobrepostas, e a cena `=63` tinha **5,7 peças empilhadas em cada ponto**. ⛔ **Isso não é
    um defeito universal e não se varre:** numa cena de campo denso a sobreposição é o look, e
    o Enio já aprovou vários. Ela é fatal **quando o assunto da cena é a ORDEM DE SAÍDA** —
    porque aí a ordem de desenho é a mesma variável, e a metade desenhada por último **tapa**
    a primeira. Sintoma: as bandas de ordem espacial (X, diagonal) leem-se bem, e a de ordem
    embaralhada sai *quase toda da cor final, com manchas*. Cura: um `motion.scale` com
    `amount < gap` antes da ordenação. Gate `no_piece_is_wide_enough_to_hide_its_neighbour` —
    ⚠️ ele mede o **cozido** (lado da peça contra a menor distância entre vizinhos), não os
    dois literais lado a lado, e **exige** a coluna `size` em vez de a tolerar: a ausência
    dela é exactamente o estado que reprovou.
16. ⚠️ **TODO modificador honra o `falloff`, e um `motion.move` de LAYOUT posto depois de um
    campo é mascarado por ele.** A cena `=66` punha as seis bandas nos seus quadrantes com um
    `move` no fim da cadeia; as peças no cheio do campo andaram o vão inteiro e as de fora
    ficaram onde estavam, então a banda **espalhou-se** em vez de se mudar (medido: um vão de
    `5,6` deu um deslocamento efectivo de `4,6`). Os quatro gates da cena reprovaram e o
    diagnóstico veio de uma SONDA que imprimiu o alcance de `x`/`y` por banda — a asserção
    sozinha só dizia *"os dois lados são iguais"*. *Layout antes do campo; e aí o centro do
    campo tem de seguir a banda.* Irmã da 14 e da lei do quadro da cena `=65`.
18. ⚠️ **A CONFERÊNCIA MEDE O PRODUTO CONTRA UMA FOTOGRAFIA DELE, e a fotografia envelhece.**
    Cada célula tem uma coluna «params hoje»; uma wave que acrescenta um param fecha a célula
    DELA e deixa as vizinhas a descrever um nó que já não existe — e o placar passa a contar
    como aberto o que já shipou. Medido em 2026-08-19: a célula do `motion.emitter` dizia
    **10** params, o manifesto tinha **20**, e **cinco das seis** linhas P1 daquela folha já
    estavam feitas. ⛔ **Isto não se resolve com disciplina, resolve-se com instrumento:**
    `python3 "docs/Motion Nodes/ferramentas/conferencia_vs_manifesto.py"` cruza as duas
    contagens e sai vermelho. ⚠️ **Ele imprime DOIS sinais e eles têm forças diferentes.** O
    **forte** é o param que a coluna «default que reduz» nomeia já estar no manifesto — ele
    aponta o ITEM. O **fraco** é a contagem de «params hoje» discordar — diz só que o nó mudou,
    e o que mudou pode ser de outra célula (31 linhas em 16 nós, quase todas benignas).
    ⚠️ **Calibração medida sobre as 7 que o sinal forte acusou: 2 verdadeiras, 5 falsos
    positivos, e os cinco da MESMA forma — o nome existe com outro significado ou com menos
    valores** (um `emit_mode` a que falta um terceiro valor; um `start` que é o do *sweep* e
    não o do *trim*). A tabela está no doc-comment do instrumento. *Um homónimo e um enum com
    um valor a menos leem igual num nome — o que decide é ler a célula.*
20. ⚠️ **O `applicable` virou a saída padrão para o device, e a lista já tem TRÊS** — cada uma
    por um motivo estrutural diferente, e vale atacá-las juntas um dia: o `reindex` do
    `motion.combine` (a concatenação no device é um `copy_buffer_to_buffer` sem shader), o
    `probability` do `motion.emitter` (a `count_law` é aritmética e um portão por hash torna a
    contagem dependente de DADOS — pede o prefix-sum que o `motion.cull` já tem) e a
    `direction` do `motion.bend` (a expressão de um `ReduceSpec` só alcança `params`, então um
    extent rodado exigiria o polinómio do `trig.rs` escrito uma segunda vez dentro da string).
    ⚠️ **O contra-exemplo importa tanto quanto a lista:** o `radius` do `motion.twist` mexe no
    MESMO tipo de redução e **não** recua — porque a redução não muda de expressão, só o
    consumidor dela escolhe entre o valor medido e um param. *O que força a recusa é a redução
    (ou a contagem) mudar de FORMA, nunca o param existir.*
21. ⚠️ **Um gate que empurra params por uma FRAÇÃO do curso acusa todo enum de estar morto.**
    O `every_control_the_write_on_scene_offers_does_something_in_it` cutucava cada param por
    37% do curso; o `mode` do `spline_wrap` é lido com `.round()`, então `0,37` voltava a `0` e
    o gate reprovou um controle vivo. A cura é do gate e não do param: ele arredonda o empurrão
    ao `step` do hint quando o passo é inteiro, o que vale para **todo** param discreto.
22. ⚠️ **Ao escolher a amostra de um gate, evite os pontos FIXOS da lei que mede.** Dois gates
    do `twist` nasceram vermelhos sobre produto correcto porque eu amostrei em `t = 0,5`: o
    smoothstep e o smootherstep **fixam o meio** (`0,5 → 0,5`), então três dos quatro perfis
    dão ali o mesmo número e o gate acusa de decorativo um enum que funciona. A amostra tem de
    ser onde as leis DISCORDAM. ⚠️ E irmã disto: **a senoide parabólica do HR-5 não é
    norma-preservante** (`c² + s² ≠ 1` ao bit) — uma rotação por ângulos diferentes perturba o
    raio em ~0,1%, então um gate de *"a rotação não muda o raio"* precisa de uma barra relativa
    e não de um épsilon de `f32`.
23. ⚠️ **Ao consertar um nó de ESTRUTURA, MEÇA os irmãos que mexem na mesma lista** — a sonda
    `measure_identity_after_structure` foi escrita depois do conserto do `sort` e achou o mesmo
    defeito em três vizinhos (`cull` encolhe e deixa `Count` velho; `mirror` e `kaleidoscope`
    crescem e deixam `Index` **e** `Count` velhos; `clone` faz o certo). ⛔ **E não corrija
    meio:** no `mirror`, arrumar só o `Count` faz a rampa alcançar **metade, duas vezes** — as
    duas colunas são uma pergunta só, e a resposta é de família.
24. ⚠️ **Uma FIXTURE sobre um eixo não distingue uma rotação da sua TRANSPOSTA.** A mutação que
    troca `dx·c − dy·s` por `dx·c + dy·s` **sobreviveu** a cinco gates do `space` do
    `motion.move`, porque todos usavam `dy = 0` — e com `dy = 0` as duas expressões são a mesma
    linha. O comprimento também não separa (as duas são isometrias). *Um oráculo de rotação
    precisa de um vetor OBLÍQUO e de um ângulo OBLÍQUO*, e o gate que nasceu disso fixa o
    SENTIDO (`(0,1)` a 90° tem de ir para `(−1,0)`).
25. ⚠️ **Um canal que ninguém consegue ESCREVER não existe** — e a célula que pede o leitor
    paga o escritor. O `use_falloff_y` do `motion.scale` sem o `mask_channel` do `motion.falloff`
    seria um toggle que o smoke não distingue de um bug: nada no catálogo escrevia `falloff_y`.
    As duas metades são **uma** célula da conferência, e é assim que ela foi fechada.
    ⚠️ E a metade irmã: **ligue por PARAM, nunca pela presença de uma coluna.** Presença faz um
    nó a montante mudar o resultado de um nó que ninguém tocou (invisível no painel), e dá ao
    device uma pergunta que ele não pode ver — o kernel não sabe se a coluna existia. Com o
    toggle, a ausência tem um significado previsível (`1.0`, a identidade da binding) e as duas
    portas resolvem a MESMA expressão.
26. ⚠️ **Uma PALAVRA que já tem dono não se reusa, mesmo com sentido diferente — há gate.** Pus
    um param `channel` no `motion.falloff` (que máscara escrever) e o
    `no_param_of_a_channel_driven_node_is_declared_a_fixed_length` reprovou: neste app `channel`
    significa *a GRANDEZA dirigida* (`drive`/`oscillator`/`noise`/`wiggle`), onde um comprimento
    vale metros em Position e **graus** em Rotation. O gate varre a palavra, e estava certo sobre
    a palavra. ⛔ **A cura é RENOMEAR (`mask_channel`), não abrir excepção no gate**: uma palavra
    com dois sentidos é a falha de duas-portas no vocabulário, e cada excepção num censo é um
    buraco que o próximo nó atravessa.
27. ⚠️ **Ao comparar duas bandas de uma cena, compare a forma RELATIVA ao centro de cada uma.**
    Três gates da `=69` nasceram vermelhos porque eu comparei `P` absoluto de bandas que vivem em
    `x = ±5,6` — estavam a medir o layout, não o nó. E depois de subtrair o centroide, a
    igualdade tem de ser APROXIMADA: `q − centroide` cancela ~5,6 de ~5,9 em `f32`, e os dois
    lados perdem bits diferentes (medido: `0,3000002` contra `0,2999997`).

---

## §5 — O ritual de cada célula (o que fazer, na ordem)

1. **Leia a célula inteira**, inclusive a coluna *"exprimível?"* — ela costuma trazer o
   mecanismo, e é onde as dez que envelheceram estavam erradas.
2. **Escreva uma SONDA `measure_*`** em `crates/ph2d-node-registry-init/tests/` que tenta as
   rotas de composição e **IMPRIME** (`#[ignore]`, `--nocapture`). Se ela mostrar que o
   catálogo já dá, a célula **envelheceu** — reescreva o veredito com o número e siga.
3. **Só então** escreva o param, com o default que **reduz** ao mundo de antes, e um gate que
   peça `==` sobre isso.
4. **CPU e GPU juntos**, com paridade. Se o nó tem kernel, o corpo WGSL é port linha-a-linha e
   a paridade é quem guarda a igualdade das duas cópias.
5. **Prova de mutação** — RED só conta sobre algo visto VERDE antes.
6. **Uma cena** por grupo, com **CONTROLE** dentro dela. Números que a mensagem cita vivem em
   `const` presos por um gate que lê o fonte da narração.
7. **Reconcilie a `Contagem`** da folha rodando o placar (ele **imprime e sai vermelho**;
   `--write` não existe).
8. **`CLAUDE.md §5` recebe UMA LINHA** — a narrativa vai no handoff.

---

## §6 — Comandos que esta linha usa

```bash
# inner loop
bash scripts/cargo-check-narrow.sh ph2d-node-motion-<nó>

# a suíte de uma crate (exit 0 verde · 1 teste vermelho · 2 não compilou)
bash scripts/cargo-test-narrow.sh ph2d-node-motion-<nó>

# a sonda de uma célula
CARGO_INCREMENTAL=0 cargo test -p ph2d-node-registry-init --test measure_<x> -- --ignored --nocapture

# paridade CPU×GPU (⚠️ skip gracioso NÃO é verde — confirme que o adapter apareceu)
CARGO_INCREMENTAL=0 cargo test -p ph2d-gpu-cook --test gpu_cpu_parity -- --ignored --test-threads=1 <filtro>

# o gate batched, 1× no fim do grupo
CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast

# a superfície de colisão, antes de fechar
bash scripts/collision-surface.sh main
```

---

## §7 — O smoke que está pendente

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=60 cargo run -p ph2d-host-desktop --release
```

Quatro blocos em **2×2**, o **mesmo** ruído; muda só o espaço. Julga-se **PARADO**, e o
campo **É o tamanho do ponto** — cada bloco é um retrato dele, não um movimento.
Em cima: controle (manchas redondas) · rodado 45°. Em baixo: listras deitadas · listras na
diagonal. ⚠️ Se um bloco parecer ter pontos **maiores** que os outros, a cena perdeu o
controle — o que muda é ONDE o campo é amostrado, nunca quanto ele vale.

⚠️ **Esta é a v2.** A v1 foi reprovada no smoke (*"não tem nada girado nem na diagonal"*) —
o porquê, e os dois gates que nasceram dele, estão na lei **8** do §4.

---

## §8 — Onde ler

- **Estado do módulo:** `CLAUDE.md §5` (roteador, não história).
- **A conferência:** [`89_conferencia/README.md`](../89_conferencia/README.md) — 17 folhas; o
  placar é **derivado**.
- **O mecanismo das waves desta linha:** [`handoffs/README.md`](README.md) — o índice
  cronológico (⚠️ ele estava **oito** atrás em 18/08; se acrescentar um handoff, reconcilie a
  contagem lendo a pasta).
- **Processo:** DIRETRIZ §1.5 (Modo L) · §1.5.9 (fechar a linha).

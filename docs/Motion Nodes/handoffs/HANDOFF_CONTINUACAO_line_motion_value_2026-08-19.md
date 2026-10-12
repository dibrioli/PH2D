# HANDOFF DE CONTINUAÇÃO — `line/motion-value` · **o PLANO e as tarefas em aberto**

> História verbatim: [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-08-19.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Trim/dash como item novo | Refutado: o *Trim Paths* é o `fx_trim` (arco exato, abre o contorno), na pilha de efeitos; 42 de 47 formas são fechadas | vivo §1.1 / Grupo U |
| `fill_rule` | Refutado por medição: a estrela citada nunca auto-intersecta | vivo Grupo U |
| *Pick Instances* | Refutado: já existe (`combine` + `duplicator(pick)`) | vivo Grupo U |
| Confiar em rotas recusadas listadas na célula | Caíram quatro dias depois sem ninguém reconferir | vivo Grupo T |
| Um SEGUNDO `fx.glow` no grafo | É silenciosamente INERTE (`from_graph`): defeito do Grupo S, não recusa | vivo Grupo S |
| «Corrigir» ao passar acusações sem cena de smoke | Não: cada acusação tem a sua cena e o seu gate | vivo Grupo W |
| Sobrescrever uma cena de smoke sem olhar o arquivo | A lei 11 (cena `=51` sobrescrita) | vivo §4 lei 11 |

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
| smoke pendente | **`=71`** (a família `force.*`, e ela SÓ se julga com PLAY) — a `=70` foi aprovada depois de duas correcções |

⚠️ **Dois smokes de ontem nunca foram vistos pelo Enio** e já estão no `main`:
`=58` (re-smoke depois da correção do relógio que expirava) e `=59` (a porta de tempo).
Se ele reportar algo sobre eles, o mecanismo está no handoff do FECHO.

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
> do [`motion_state_demo_router.rs`](../../../shells/desktop/src/motion/motion_state_demo_router.rs),
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

### ~~Grupo U~~ — ✅ **FECHADO (2026-08-21): a folha 14 não tem mais P1**

✅ **Feito:** sweep/start/inner · raio por canto + smoothing · o `corner` geral (4 → 38
espécies) · e a correção de geometria que o smoke do Enio devolveu (a borda da fatia
abaulava 19–25% do raio). ⛔ **Refutados por medição:** `fill_rule` (a estrela citada nunca
auto-intersecta; só 2 de 43 espécies distinguem as regras, e nelas a actual é a certa) e
*Pick Instances* (já existe: `combine` + `duplicator(pick)`).

**As TRÊS que fecharam em 2026-08-21, e o que cada uma ensinou:**

| item | como fechou |
|---|---|
| **TRIM / dash** | A recusa medida apontava a **função errada**. O *Trim Paths* é o [`fx_trim`](../../../crates/ph2d-vec-scene/src/fx_trim.rs) — arco exato, **abre** o contorno, testado desde os Live Path Effects. Entra na **pilha de efeitos** do `VecPath` (o `cooked()` corre-a), então alcança contornos compostos sem uma linha por espécie, e o neutro `{0,1,0}` volta a `Cow::Borrowed`. O tracejado já morava no `StrokeSpec`. **Lei 59.** |
| **`size` é GEOMETRIA, não coluna** | Geometria em **raio 1** + coluna `size`. A linearidade da receita foi **medida nas 43 espécies**, e a medição achou a exceção: o **`Speech Rect`** tirava o raio de quina dos `defaults()` em unidades de MUNDO (erro de 23%) — a quina do balão não crescia com o balão. ⚠️ **A mudança de semântica que esta tabela previu ACONTECEU e é a certa** (medido, com o tamanho autorado 2 e um valor 3): `Add = 5`, `Set = 3`, `Multiply = 6`. Sem a coluna, o `drive` partia da identidade unitária e `Set` e `Multiply` davam **o mesmo número** — o modo era um controle sem diferença. |
| **a POSE do objeto não viaja** | ✅ fechada na janela anterior (`Transform = Position Only / Object Pose`, canal `external::pose_of`). |

⚠️ **O que a wave ABRIU, com o número medido:** um param de forma animado interna **uma
geometria por quadro** (`measure_shape_store_growth`: 600 em 600 quadros, ~500 B cada ⇒
**~30 KB/s por forma animada**). O `size` saiu da chave e por isso não cresce; o Trim, o
`sweep` e os outros crescem, porque mudam a geometria de facto. ⛔ **A cura NÃO é uma
allowlist nem um teto adivinhado:** é podar o `VecPathStore` para as chaves publicadas no
quadro — e isso está **bloqueado** por o `push()` sem chave do `source.object` partilhar o
mesmo `by_handle` indexado por posição, então podar invalidaria handles vivos. O passo é
handle com geração, e é uma wave própria.

⚠️ Este grupo **encostou no módulo Vector e foi certo encostar**: a correção de
`cap_arc_ends` é em `ph2d-vec-scene` (foundational, Modo L permite) e os 387 testes daquela
crate passaram sem uma edição de asserção. Contrato congelado (§6) continua a ser parar e
reportar.

### Grupo V — as folhas grandes, por ORDEM DE DEFEITO

`08_stream_utilidade` (8) · `01_distribuicao` (6) · `04_deformers` (6) · `10_field` (6) ·
`02_force` (2) · ~~`11_fx_raster`~~ ✅ · ~~`05_transform`~~ ✅ · `03_simulacao` (3) ·
`07_tempo` (2) · ~~`09_cor`~~ ✅ · ~~`14_source`~~ ✅.

⚠️ **Derivado, não escrito:** rode `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"`. Em 2026-08-21 o TOTAL é **P1 = 5**: folha 01 (1) · 03 (1) · 04 (1) · 07 (2).

⚠️ **A contagem por folha se DERIVA** (`python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"`),
nunca se lê daqui: esta lista envelhece a cada célula fechada.

⚠️ **Não ataque por tamanho.** Dentro de cada folha, o que vem primeiro é o que a célula
descreve como **comportamento errado** (o `fx.glow` inerte, o `motion.duplicator` que perde a
escala do ponto, o `motion.step` com limitação auto-declarada), e só depois o que é knob
ausente.

### Grupo W — **a CAÇA AOS KNOBS MORTOS** (multi-agente) · ⚠️ NO FIM DA FILA

> **Pedido do Enio, 2026-08-21**, imediatamente depois de o smoke do `=73` ter achado um:
> *"Coloque no fim da fila de implementação auditoria multiagêntica a busca de parâmetros
> mortos como esse."* ⚠️ **Ele disse NO FIM** — as células abertas da conferência vêm
> primeiro. Isto abre quando a fila acima esvaziar, ou quando ele mandar.

**O que é um knob morto:** um controle que o painel pinta e que **não muda a imagem**. A
linha já encontrou **quatro espécies distintas**, e elas não se acham com a mesma sonda —
é por isso que isto é uma varredura e não um grep:

| espécie | caso medido | como se acha |
|---|---|---|
| **morto no braço DEFAULT** | `field.remap::curve_offset` — `curve.map_or(t, …)` saía antes de o deslocamento correr, e *sem curva autorada* é o estado em que o nó nasce | mutar o param e cozinhar **a partir do documento em branco**, não de uma fixture já configurada |
| **inerte no MODO que o painel mostra** | `curvature`/`steps` do mesmo nó, vivos com o contorno em `Curve` | varrer o espaço de MODOS: inerte em *algum* modo e sem `ParamGate` ⇒ acusa |
| **descartado a JUSANTE** | `motion.color_array::offset` lido por `.first()` — um campo por-instância evaporava | mutar o param **por elemento** e exigir que a saída varie por elemento |
| **declarado e nunca LIDO** | — (nenhum medido ainda) | cruzar `MANIFEST.params` com os `ctx.param("…")` da crate |

**O método, e é ele que torna a varredura paralelizável:** para cada nó × cada param ×
cada modo, cozinhar duas vezes com valores diferentes do param e comparar as colunas de
saída **ao bit**. Idênticas em todo o espaço alcançável ⇒ morto. Cada nó é independente
de todos os outros ⇒ um agente por família de nós.

⚠️ **A armadilha, e ela já mordeu:** *inerte* não é *morto*. Um param legitimamente
inerte noutro modo é o que o `ParamGate` existe para esconder — o veredicto tem de
separar **«inerte em TODO o espaço»** (defeito no nó) de **«inerte em ALGUM modo e não
gateado»** (defeito no painel). Um audit que colapse os dois devolve uma lista de falsos
positivos do tamanho do catálogo.

**O que já existe e não se reconstrói:** `measure_node_params` (a sonda que deriva os
params do registry) · `conferencia_vs_manifesto.py` (cruza a conferência com o manifesto)
· `measure_scene_layout` (cozinha uma cena e mede o que ela desenha) ·
`each_pair_actually_differs_in_the_cooked_result` (o molde do gate: comparar colunas
cozidas, sobre TODAS as colunas).

⚠️ **Isto é uma AUDITORIA, não uma wave de features** — o produto dela é uma tabela de
acusações com o mecanismo ao lado de cada uma, e as curas entram na fila depois, uma a
uma, cada qual com o seu gate. ⛔ Não «corrija» ao passar: uma acusação sem cena de smoke
é uma mudança de comportamento que ninguém olhou.

---
---

## §4 — As OITENTA E CINCO LEIS que esta linha pagou para aprender

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
8. **⚠️ CORRA OS **DOIS** GATES DE LOC**, pelos nomes, ao fechar o grupo:
   ```
   cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap   # crates/, teto 700
   cargo test -p ph2d-host-desktop --test file_loc_caps                        # shells/, teto 600
   ```
   ⚠️ **São DOIS, com tetos diferentes, em crates diferentes** — e a lei 76 mordeu duas vezes
   na mesma jornada por causa disso: em 22/08 corri o primeiro, dei-o por verde, e o segundo
   apanhou o `motion_state_demo_conferencia.rs` a **614**. O primeiro não está na suíte do
   shell; o segundo ESTÁ, mas eu tinha filtrado a suíte pelo nome da cena. *Um gate que corre
   dentro de um filtro também não corre.* A cura de um teto é sempre um **split por
   responsabilidade**, nunca uma entrada na allowlist (HR-18).
9. **`CLAUDE.md §5` recebe UMA LINHA** — a narrativa vai no handoff.

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

## §7 — Os smokes que estão pendentes

### `=81` — O VOCABULÁRIO DA UTILIDADE (folha 08, **quatro células**)

```
env PH2D_GPU_COOK_DEMO=81 cargo run -p ph2d-host-desktop --release
```

⚠️ **Ela desenha FIGURAS, não perfis** — é a primeira cena de conferência em que o oráculo é a
FORMA, e por isso não mora no `..._grafico.rs`. Julga-se **parada**.

Quatro linhas, esquerda/direita: a reta contra a tenda (`Avg` × `Min`) · a lente contra o
círculo (a geometria presa a uma lane) · a diagonal contra a diagonal partida (o `shift`) · a
diagonal contra a volta (o mesmo par de números em `Cartesian` e em `Polar`).

### `=80` — O METRÓNOMO (folha 12, que **FECHOU POR INTEIRO**)

```
env PH2D_GPU_COOK_DEMO=80 cargo run -p ph2d-host-desktop --release
```

Oito fileiras aos pares. Cada uma sobe **um degrau por batida** — a altura das peças é quantas
batidas passaram. ⚠️ **Precisa de Play**: parada, a cena inteira fica no chão, e é isso que
uma cena de metrónomo é.

O `pulse.counter` está em **`Clamp`** de propósito: com `Wrap` a fileira que PAROU voltaria ao
chão e ficaria indistinguível de uma que deu a volta.

Duplas: a régua (`0,5 s` contra `120 BPM`, que têm de subir em lock-step) · a fase por-linha
(bloco contra onda) · a janela (eterno contra 4 batidas) · o limiar (único contra por-elemento —
o par que mostra o colapso silencioso do fio-no-param).

### `=79` — A FAIXA QUE O NOME PROMETE (folha 06, **três células + uma refutação**)

```
env PH2D_GPU_COOK_DEMO=79 cargo run -p ph2d-host-desktop --release
```

Oito fileiras aos pares, e **duas marcas maiores à esquerda de cada uma** — a régua, exactamente
em `min` e `max`. A pergunta é uma só: *o movimento encosta nas duas?*

⚠️ **A dupla 1 tem as duas metades IGUAIS de propósito** — a conta que o artista faz de cabeça
está CERTA numa forma bipolar, e a cena não está lá para o acusar de um erro que ele não comete.
São as duplas 2 e 3 (o `Spike` e o `Ridged`) que divergem, e é o PISO que diverge.

Precisa de **Play** (as três primeiras duplas são temporais). A dupla 4 são os modos apendados
do `motion.drive` — `Add` contra `Min` —, e as outras três (`Subtract`/`Divide`/`Max`) saem pelo
dropdown do nó.

### `=78` — OS NOVE KNOBS do domínio de valor (folha 15, **dez células**)

```
env PH2D_GPU_COOK_DEMO=78 cargo run -p ph2d-host-desktop --release
```

Dezoito fileiras em dois blocos, **aos pares**: a de cima é o nó como era, a de baixo é o
botão novo ligado. Cada fileira é um gráfico (o Y de cada peça É o valor) — o mesmo idioma da
`=41`, de propósito.

⚠️ **As duas últimas da direita só se julgam com o PLAY** (a rampa de entrada da `value.lfo`
dura 3 s; depois disso as duas ficam idênticas, que é o que ela promete).

Cobertura: `invert` do `step` · `offset` do `quantize` · `offset`+`interp` do `pattern` ·
`blend` do `switch` · `factor` do `curve` · `clamp_result` do `mix` · `Multiply Add` (3ª porta)
e `Smooth Min` (`distance`) do `math` · `fade_in` do `lfo`. Cada par tem o gêmeo medido no
device em `gpu_cpu_parity_arith`.

### `=77` — o OPERADOR por linha (folha 07)

```
env PH2D_GPU_COOK_DEMO=77 cargo run -p ph2d-host-desktop --release
```

Duas linhas, e **as duas precisam de PLAY**. **RASTRO**: a mesma bolinha percorre um OITO
(um caminho que se CRUZA — sem cruzamento não haveria o que somar, e há gate a exigi-lo);
à esquerda a cauda TAPA, à direita ela SOMA e o cruzamento acende. **FLASH**: o mesmo
strobe, à direita a somar. ⚠️ O primeiro item dos dois dropdowns (`Sink`) **não é um modo**
— é *"o mesmo do Output"*, e é o default.


### `=76` — a folha 14 inteira (o Trim, o tracejado, e o traço que apagava o miolo)

```
env PH2D_GPU_COOK_DEMO=76 cargo run -p ph2d-host-desktop --release
```

Três linhas, duas colunas. **BORDA**: a mesma estrela azul, à direita com borda laranja — e o
miolo continua azul (antes, `stroke_width > 0` deixava a forma OCA). **APARADO**: um anel
branco inteiro contra um trecho de 28% dele que **dá a volta** (o `trim_offset` é conduzido
por um fio — precisa de PLAY). **PICOTADO**: o mesmo retângulo com linha contínua e com linha
picotada. ⚠️ **Pôr o `Stroke Width` em 0 faz o Trim, o Dash e a cor da borda SUMIREM do
painel** — é o `ParamGateAbove`, e a razão é que sem traço aparar a forma a faz desaparecer.


### `PH2D_MOTION_OBJ_SMOKE=8` — a POSE do objeto (folha 14)

```
\
  env PH2D_MOTION_OBJ_SMOKE=8 cargo run -p ph2d-host-desktop --release
```

⚠️ **Não é uma cena `=N`, e não podia ser:** o `source.object` lê um objeto da CENA
(um external publicado do mundo ECS), não do grafo — então o smoke tem de spawnar o
objeto, e isso é o que a família `PH2D_MOTION_OBJ_SMOKE` faz.

### `=75` — duas das três da folha 03 (o pin que rasga, o bando que desvia)

```
\
  env PH2D_GPU_COOK_DEMO=75 cargo run -p ph2d-host-desktop --release
```

Duas linhas, rotuladas no canvas. ⚠️ **SÓ com o PLAY** — as duas são simulação. As
pedras são desenhadas de propósito.

⚠️ **Esta é a v4, e as três correcções vieram de smokes.** A v1 foi reprovada (*"tudo
foi levado pelo vento, nada rasgou"*): o pin estava no laço do integrador, que lê o
`inv_mass` do `rest`. A v2 foi reprovada por *"porque usar grid se temos nós de
tecido?"* — a cortina é agora um `motion.soft_body`. A v3 por *"não rasga"*: o rasgo
estava certo mas acontecia no primeiro quadro, e o vento passou a SUBIR com o tempo.
Leis **52** a **56**.

### `=74` — a folha 02 inteira (o alvo do atrator, e a mira)

```
\
  env PH2D_GPU_COOK_DEMO=74 cargo run -p ph2d-host-desktop --release
```

Duas linhas, rotuladas no canvas. ⚠️ **SÓ se julga com o PLAY** — uma força acumula em
`accel` e é o integrador que aplica; parada, as quatro nuvens são iguais. Os alvos são
os pontos **brancos**, desenhados de propósito: um alvo invisível faz a cena virar
adivinha.

### `=73` — as folhas 08 e 10 inteiras (quatro células, e um nó NOVO)

```
\
  env PH2D_GPU_COOK_DEMO=73 cargo run -p ph2d-host-desktop --release
```

Quatro linhas, cada uma com o nome escrito **no canvas** e uma pergunta de sim/não:
**CORTE** (a fileira acende até o fim?) · **BANDA** (as peças acesas ficam juntas ou
espalhadas?) · **RAMPA** (ela recomeça no meio?) · **FORMA** (o pentágono acende cheio
ou só a borda?). Esquerda = como era, direita = o knob novo.

⚠️ **Esta é a v2.** A v1 foi reprovada (*"tudo misturado e bagunçado"*) e a causa não
era feature nenhuma — era a colocação das bandas correr **depois** do campo, logo
mascarada por ele. Mecanismo e instrumento nas leis **43** e **44** do §4.

### `=72` — a folha 09 inteira + o pareamento do `motion.step`

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=72 cargo run -p ph2d-host-desktop --release
```

Três pares. ⚠️ **O par 3 é o único cujos dois lados têm de ficar IGUAIS** — a leitura é
*"as duas fileiras sobem em bloco"*, e o modo de falha a nomear é *"só a primeira peça da
fileira de baixo anda"* (era esse o defeito: o batimento global chegava ao elemento 0 e a
mais nenhum). ⚠️ **A quarta célula da folha 09 — o kernel de GPU do `color_array` — prova-se
pela AUSÊNCIA de diferença:** rodar de novo com `PH2D_GPU_COOK=0` na frente tem de dar a
mesma imagem.

### `=60` — o ESPAÇO do campo do `motion.noise`

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

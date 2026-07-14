# HANDOFF de CONTINUAÇÃO — `line/Vector` (2026-07-14, 6ª passagem)

> **Para:** o **próximo implementador** da linha `line/Vector`.
> **De:** o agente que auditou o algoritmo de Blend — o giro do quadrado, os defeitos que ele
> escondia, a costura do 2º Blend, e as regressões que uma 2ª lente adversarial pegou.
> **Estado:** linha **aberta**, **6999/6999 verde**, clippy/fmt/typos limpos. Nada foi integrado,
> nada foi shipado. (A crate `ph2d-vec-blend` tem **26 gates**; a costura do painel, **+2**.)
>
> **O smoke do Enio está PENDENTE** (§2). Ele é a próxima coisa a acontecer — não comece a fila
> antes dele.

---

## §0 — Como se trabalha aqui (Modo L) — **não é opcional**

Você é **uma linha autônoma** numa jornada multi-agente
([GUIA_JORNADA_MODO_L.md](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) ·
[DIRETRIZ §1.5](IntegracaoMultiAgente/DIRETRIZ.md) · ADR-0106 · ADR-0107).

| | |
|---|---|
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector`, branch `line/Vector` |
| **Você commita** | `git commit --no-verify -m "..."` — local, à vontade, em blocos |
| **Você NÃO** | **integra** · **pusha** · roda **`ship.sh`**. Nunca. Por conta própria é violação de protocolo (CLAUDE.md §0.7) |
| **Foundational** | você **PODE e DEVE** tocar (ADR-0107). Ao **criar** foundational novo, projete para isolamento |
| **PARE e reporte ao Enio** | só em 2 casos: **contrato congelado** (CLAUDE.md §6) ou **rebase conflitando fora dos seus arquivos** |
| **Você fecha** | escreve o **handoff de integração** (DIRETRIZ §1.5.9) e **PARA** |

> **O `cwd` do shell DERIVA no meio do turno.** **Todo comando começa com
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector &&`**, e **toda mutação de arquivo
> usa caminho ABSOLUTO**. Para ler o `main`, use refs (`git show main:arquivo`), nunca o filesystem.
> ([[feedback_sed_relative_path_hits_primary_cwd]] · [[feedback_perl_utf8_mojibake_use_edit_tool]] —
> texto acentuado **só** via ferramenta Edit · [[feedback_backticks_in_commit_message_are_command_substitution]].)

**Ritmo:** inner loop = só `cargo check -p`. Gate batched **1× no fechamento**
(`cargo nextest run --workspace --no-fail-fast` + `cargo clippy --workspace --all-targets` +
`rustup run 1.95 cargo fmt --all -- --check` + `typos`). Smoke 1× no fim, **com o comando pronto,
incluindo o `cd`** ([[feedback_ready_to_smoke_example]]).

---

## §1 — O que eu fiz (a auditoria que o Enio pediu)

A ordem era *"comece pela auditoria desse algoritmo, que ainda não é perfeito"*. O defeito
reportado era **um**; a auditoria achou **três**, e dois deles ninguém tinha visto.

### 1.1 — O giro (o que o Enio viu)

**Medido primeiro, teorizado depois** — o probe imprimiu a virada de cada âncora do catálogo:

```
quinas do quadrado:  -135° -45°  45° 135°   virada (sen ±1, cos 0)   ← quinas de verdade
âncoras do círculo:     0°  90° 180° -90°   virada (sen  0, cos 1)   ← PERFEITAMENTE SUAVES
casamento escolhido: cada quina → a âncora 45° adiante   ⇒ giro de 45°, nas quatro
```

**A causa não estava na busca — estava no CONJUNTO DE CANDIDATOS.** As 4 âncoras do círculo existem
porque a elipse é **cozida em 4 cúbicas**; o artista nunca as autorou. Mas o motor obrigava âncora a
casar com âncora, e a resposta certa — a quina a 45° vai para o ponto do círculo a 45°, que cai **no
MEIO de um segmento** — **não estava no conjunto**.

**Fix (2 peças):**

1. **`features()`** — só uma âncora com **virada acima de um limiar** é candidata a nó. O limiar é
   escrito em **cosseno**, não em seno, e isso não é estilo: uma **cúspide** vira ~180°, onde o seno
   volta a zero — com `|sen| > ε`, o bico mais afiado que existe seria classificado como suave.
2. **`phase_only()`** — sem quina dos dois lados, a correspondência é uma **fase contínua**, achada
   por **correlação circular** (256 amostras de arco) + **refino parabólico**. A fase deixa de estar
   presa às âncoras e pode cair onde tiver de cair.

### 1.2 — Os dois que o giro escondia

- **Dois círculos iguais ENCOLHIAM 7,6%** no meio do caminho (raio 0,924) quando as
  parametrizações não estavam alinhadas: os pontos atravessavam o disco em vez de caminhar
  radialmente. **O catálogo escondia** — as elipses nascem todas com as âncoras nas mesmas posições
  de arco. Só aparece quando o artista desenha a segunda **à mão**.
- **Uma aresta inteira SUMIA do pareamento.** Os cortes de B são as **imagens** dos cortes de A, e a
  imagem que devia cair na origem volta do ida-e-volta `map_backward`→`map_forward` como
  **`1 − 3,7e-14`**: o `f64` não fecha o ciclo. O `cut` então truncava a peça em `1.0`, e o arco
  entre a origem e o corte seguinte ficava **sem peça nenhuma**. O invariante que o `cut` documenta
  (*"a origem está na lista"*) era **esperado do chamador**; agora é **estabelecido por quem depende
  dele**.

### 1.3 — E um terceiro, achado indo procurar

**Picar uma aresta reta em 20 pedaços mudava a correspondência.** Geometria idêntica, âncoras a mais
— e as quinas do quadrado casavam com **outros** vértices da estrela. O centro e a escala que
normalizam o custo saíam da **média das âncoras**, e âncora é **parametrização**.

Agora o quadro sai de amostras equiespaçadas em **arco**. Não é caso de laboratório: todo caminho
traçado, importado ou passado por um `Simplify` tem âncoras onde o algoritmo as deixou. **Duas
formas que se veem iguais têm de blendar igual.**

### 1.4 — `Plan` (público, novo)

A correspondência é função do **par**, não do `t`. Buscá-la por passo custava **5,9 ms** por blend
de 10 passos depois que o varrimento de fase entrou (256×256), e o artista re-roda o blend a cada
frame enquanto arrasta o slider de Steps. Agora é **1 busca por blend** (0,6 ms), e o gate **conta**
em vez de cronometrar ([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]]).

**É a mesma estrutura que o morph vivo (§4.1) vai querer**: monte o `Plan` quando a relação mudar,
avalie um `t` por frame.

### 1.5 — E a COSTURA, que estava quebrada (achado pela auditoria de 2ª lente)

O motor certo não serve de nada se o produto não o alcança. **O 2º Blend não funcionava**, e é o
primeiro gesto que qualquer artista faz (blenda → acha pouco → arrasta **Steps** → clica **Blend**):

- **Era RECUSADO.** O `Run` exigia *"exatamente duas formas fechadas selecionadas"* — mas **depois de
  um Blend, o que está selecionado são os PASSOS** (o `select_many` no fim do `apply`). O Run não
  achava as fontes, recusava, e imprimia a queixa num terminal que o artista não está lendo. **Na
  tela: nada acontecia.**
- **Com `Steps = 2` era pior que recusar:** a seleção **era** duas formas fechadas — os dois passos —
  e o Blend seguinte **blendava os próprios passos**, em silêncio. A arte derretia para o meio a cada
  clique.
- **E o `produced` era zerado** no braço do `Run`: re-rodar sobre as mesmas fontes (re-selecionando-as
  à mão, o que é possível) **não removia os passos velhos** — empilhava um jogo novo por cima.

Fix em `shells/desktop/src/vec_blend.rs`: a pergunta certa não é *"há duas fechadas?"*, é **"de quem
são as fontes?"**. Enquanto a seleção for a que a sessão produziu, o artista está iterando no MESMO
blend (`selection_is_produced`). +2 gates, os dois mutation-tested.

> **A intenção já estava no código** — o comentário dizia *"clicar Blend de novo não deveria jogar
> fora o trabalho dele"* —, e o braço que a implementava era **inalcançável**. Intenção documentada
> não é comportamento; só um teste que CLICA prova o contrário.

### 1.6 — Os gates novos do motor (todos mutation-tested)

| Gate | O que morre sem ele |
|---|---|
| `a_square_does_not_spin_on_its_way_to_a_circle` | o giro de 45° (a correspondência) |
| `the_middle_shape_keeps_the_orientation_of_the_square` | o giro **no produto** (a forma na tela) |
| `only_a_real_corner_is_a_candidate_node` | a peneira: círculo = 0 features, estrela = 10 |
| `the_phase_is_the_number_not_the_nearest_sample_of_the_grid` | o refino (erro 3e-8 → 1,8e-3) |
| `a_circle_walking_to_a_rotated_circle_stays_a_circle` | o encolhimento de 7,6% |
| `extra_anchors_on_a_straight_edge_do_not_change_the_correspondence` | a dependência de parametrização |
| `the_correspondence_is_searched_once_per_blend_not_once_per_step` | o hoist do `Plan` |

E o `every_piece_of_the_pairing_is_a_real_piece` ganhou o oráculo que faltava: **cobertura**. A
peça-ponto era o *sintoma*; o dano é o **arco que ficou sem peça**, e dá para acontecer sem deixar
peça-ponto nenhuma para trás.

**6 mutações do CÓDIGO** (não do teste) foram rodadas e **todas morrem**:
`features` devolvendo todas as âncoras · fase só nas âncoras · `cut` sem o snap na origem · sem o
refino parabólico · `TURN_WEIGHT = 0` · quadro pela média das âncoras.

### 1.7 — Rotate / Reverse Match (o smoke SEGUINTE do Enio: "resultados estranhos")

O Enio testou **estrela → círculo** e viu os intermediários **rasgados/colapsados**; pediu para
conferir os dois botões de escape. Medido, por par de formas, o menor `|área|` ao longo de `t`:

| | Reverse | (default) |
|---|---|---|
| estrela → círculo | **0,026** (colapso) | 1,32 (limpo) |
| quadrado → estrela | **0,040** | 1,32 |
| círculo → círculo | **0,000** | π |

- **Reverse Match REMOVIDO.** Inverter o sentido de percurso de B inverte o **winding** (a área do
  círculo final vira **−π**), e interpolar entre windings opostos cruza área zero no meio → colapso.
  Todo o catálogo nasce com o mesmo winding, então o botão colapsava **sempre**. E era **redundante**:
  dei uma B em sentido horário e o `search` escolheu `reversed=true` **sozinho**, meio limpo — o
  sentido correto já é automático. Nenhuma ferramenta profissional tem um "reverse" que inverte
  winding. Removido de 9 sítios (campo `BlendOpts.reverse`, ação, botão, const, gate…); a lente de
  completude varreu e o sweep de código ficou limpo.
- **Rotate Match: quantum das QUINAS.** Era `1/âncoras-do-círculo = 90°`/toque (as âncoras do círculo
  são artefato de cozer a elipse), e o 2º toque (180°) colapsava. Agora `1/quinas = 36°` — passos
  finos. A torção em rotação **grande** segue intrínseca ao lerp (o gap Sederberg/Alexa). Gate nos
  **dois sentidos** (estrela→círculo e círculo→estrela: o `max(features)` é simétrico).

Gates novos: `rotate_steps_by_the_corners_not_by_the_smooth_shapes_anchors` (mutation-tested). Cena de
smoke **`PH2D_BUILD_SMOKE=9`** (estrela → círculo, para clicar Rotate). BUGS #17.

---

## §2 — ⚠️ COMECE AQUI: o SMOKE (pendente)

**Você não deveria mexer em nada antes do veredito do Enio.** A cena está pronta e é a do defeito:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  PH2D_BUILD_SMOKE=8 cargo run --release --bin ph2d-host-desktop
```

**O que olhar (uma coisa só):** o app abre com um **quadrado → círculo** e **5 passos** entre eles.
As quinas do quadrado têm de caminhar **RETO para fora**, derretendo no círculo. Se elas rodarem
(45°, e voltarem), o defeito voltou.

Cenas irmãs: `PH2D_BUILD_SMOKE=9` (estrela → círculo — o escape, **Rotate Match** em passos finos; o
**Reverse Match** foi removido) · `=7` (quadrado → estrela — o caso da DP, Rotate ciclando as quinas
sem colapsar) · `=1` (Shape Builder) · `=6` (a repro do undo).

**Se ele reprovar:** meça **antes** de teorizar. Os três handoffs anteriores desta linha apontaram um
suspeito nº 1 cada um, e os três estavam **errados**. O que resolveu, todas as vezes, foi montar a
cena dentro do app / imprimir os números do motor e **olhar**.

---

## §3 — Estado da linha

| | |
|---|---|
| **Base** | tip do `main` (a jornada de 6 linhas integrou) |
| **Commits não integrados** | 6 |
| **Gate no HEAD** | **6999/6999 verde** · clippy 0 · fmt limpo · typos limpo |
| **Arquivos tocados hoje** | `crates/ph2d-vec-blend/*` · `shells/desktop/src/build_smoke.rs` · `.typos.toml` · `CLAUDE.md` · `docs/Vector Module/BUGS_vector.md` |

**O que o Enio JÁ aprovou:** Live Corners · Shape Builder · undo/redo · a lasca do Build.
**Pendente de smoke:** o Blend (o giro — §2).

---

## §4 — A FILA (a ordem é do Enio)

1. **O smoke do §2.** (Você está aqui.)
2. **Morph vivo** (o `t` animável) — é o que transforma o Blend numa feature de **animação**. O
   desenho é o do **conector**: uma entidade cuja geometria é função pura da relação, re-cozida por
   frame. **O motor já serve**: monte um `Plan` (§1.4) quando a relação mudar e chame `plan.at(t)`
   por frame — a busca (a parte cara) não roda mais no caminho quente.
3. **Envelope / puppet warp**.
4. Do Illustrator, o que falta no Blend: **Replace Spine** (os passos seguem um caminho desenhado) e
   **Smooth Color** (o nº de passos sai do degradê).
5. Aberto de antes: **Live Path Effects como nós** (o multiplicador; a costura fonte≠cozido do
   ADR-0121 é o pré-requisito) · tipos de quina (chamfer é quase de graça) · texto em caminho · trim
   path · repeater · largura variável.

---

## §4.5 — A auditoria adversarial (2ª lente) — o que ela achou, o que eu consertei

Uma lente adversarial montou o motor de `main` como crate standalone e rodou **as mesmas fixtures**
nos dois lados, para separar regressão de pré-existente. Consertei tudo que era **regressão minha**;
o que é pré-existente ou patológico está na §5 como dívida honesta.

**Consertei (3 regressões minhas, cada uma com gate mutation-tested):**

1. **[CRÍTICO] O limiar `cos(15°)` sentava em cima do 24-gon.** Um polígono regular de 24 lados vira
   **exatamente** 15°, a comparação é estrita, e o `f64` erra o cosseno no último bit — então as 24
   quinas idênticas caíam dos dois lados por ruído. Medido: **transladar a cena** mudava a forma do
   intermediário em **5,5%**; perturbar o raio em 1e-13 dava **68 resultados distintos**. Fix: o
   limiar virou **`cos(16°)`** — `360/16 = 22,5` não é inteiro, então **nenhum** polígono regular
   consegue sentar nele (o gate enumera N até 128). É, de novo, *não se escolhe um desempate melhor —
   não se tem empate*.
2. **[MÉDIO] `offset` gigante panicava** (`c_auto + offset` estourava o `i32`; `BlendOpts.offset` é
   público). A soma virou `i64`.
3. **[BAIXO] `t = NaN` vazava NaN** para a geometria (`f64::clamp` propaga NaN). `Plan::at` agora trata
   NaN como 0. **Não é alcançável por `steps()`, mas É a API do morph vivo** — o `t` virá de uma curva
   animada.

**A lente confirmou o que JÁ estava certo** (rodou e passou): caminhos abertos · 2/1/0 vértices ·
colinear/área-zero · coords 1e-9/1e-12 · NaN/inf num handle (devolve `None` ou finito) · **o platô do
`parabolic()`** (formas idênticas concêntricas: o guard `denom ≈ 0` funciona) · o snap da origem no
`cut` · `Plan::at` **bit-idêntico** ao `morph`.

## §5 — Dívidas e minas que eu declaro

- **[DÍVIDA, regressão minha, baixo impacto] Figura-8 (forma auto-intersectante) deixa 1 âncora
  DUPLICADA.** Em `main` não deixava (a auto-interseção põe o mesmo ponto em duas posições de arco, e
  o meu `cut` produz uma peça degenerada ali). Uma forma auto-intersectante já é geometria patológica,
  e o conserto (dedup de âncoras coincidentes consecutivas em `path_from`) mexeria na costura de
  handles cruzados que eu **acabei** de estabilizar (BUGS #17, a aresta que sumia). Não é o que o Enio
  pediu, e o risco é maior que o ganho. **Decisão dele.** O sinal é visível a olho: o vértice aparece
  no modo Node.
- **[PRÉ-EXISTENTE, idêntico em `main`, mas real] Compound path perde o BURACO em silêncio.**
  `Outline::of` lê só o contorno externo; `path_from` devolve `subpaths` vazio. E o shell **aceita**:
  `two_selected_closed` filtra por `p.closed`, e uma rosquinha (a saída típica da booleana!) É
  `closed` — então o artista blenda uma rosquinha e ela vira um disco. Não é regressão; é o motor
  nunca ter suportado compound path. Vale um item de fila.
- **[PRÉ-EXISTENTE] Precipício de escala.** `ARCLEN_EPS = 1e-11` é **absoluto**: numa forma de tamanho
  ~1e5 o kurbo é forçado a precisão relativa 1e-16 e a subdivisão adaptativa **trava** (>6 s); em ~1e12
  panica dentro do kurbo. Idêntico em `main`. Um documento em unidades grandes trava no clique do
  Blend. O conserto é tornar o eps **relativo ao tamanho da forma** — mas mexe no motor todo, e não é
  desta passagem.
- **[RESOLVIDO nesta passagem] `Reverse Match` colapsava a área a ZERO.** Era um bug de design (inverter
  o winding colapsa qualquer par de mesmo winding = todo o catálogo), e o sentido correto já é
  automático — **removido** (§1.7). O **quantum do Rotate** também: vinha das âncoras da forma
  lisa (90°/toque, colapsava no 2º); agora vem das quinas (36°). A torção em rotação GRANDE segue
  intrínseca ao lerp de coordenadas (o horizonte as-rigid-as-possible, Sederberg/Alexa) — não é bug, é
  o gap do motor.
- ⚠️ **Os botões Arrange de z-order (To Front / Backward / …) continuam MORTOS.** Eles chamam
  `VecScene::reorder_path`, que muta a ordem do **vetor da cena** — e a projeção de z reescreve essa
  ordem a partir da **árvore** a cada frame (ADR-0110). **Quem quer mandar no z escreve no
  `RootOrder`** (`vec_zorder::restack` é o exemplo). Não é bug do Blend; é item de fila.
- **O motor é o lerp de coordenadas.** Ele encolhe a forma no meio do caminho e pode
  auto-intersectar numa **rotação grande** — o horizonte é Sederberg & Greenwood 1992 (trabalho
  mínimo) / Alexa 2000 (as-rigid-as-possible). **A correspondência era o pré-requisito dos dois, e
  ela agora está de pé.**
- **`crates/ph2d-vec-blend/src/tests.rs` está a 595 LOC do teto de 600.** O próximo gate que nascer
  ali **não cabe** — ele vai para `tests_phase.rs` (393) ou para um irmão novo. E lembre: `cargo fmt`
  **re-expande**; formate ANTES de medir ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).
- **`vec_history` é fila MORTA** (o undo global a subsumiu; ainda é populada e não lida).
- **`ADR-0115` está duplicado no `main`** (áudio espectral × composição de clips). Não é desta linha;
  a exceção é auto-limpante.
- **O Blend é destrutivo** (o `Make` + `Expand` do Illustrator num passo só), e isso é deliberado
  (ADR-0108: booleana e afins são *edit-time*). A **sessão** é o que dá a sensação de vivo: Steps,
  Rotate e Reverse **re-rodam** sem desfazer.
- **`.typos.toml`**: acrescentei `fases` e `candidata` (pt-BR) ao lado das entradas que já existiam.
  É arquivo compartilhado — se der conflito de merge, é append de 2 linhas. **Chave duplicada mata o
  TOML no parse e o gate inteiro fica mudo** ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]);
  conferi que não há.

---

## §6 — As lições desta passagem

1. **Quando um algoritmo escolhe entre candidatos e a resposta é sempre ruim, desconfie do
   CONJUNTO antes de desconfiar do critério.** O giro não era um erro de conta: era uma pergunta mal
   formulada ao dado. O critério estava certo o tempo todo.
2. **Um defeito visível esconde os invisíveis.** O giro era gritante (45°); o encolhimento de 7,6% e
   a aresta que sumia estavam ali ao lado, e ninguém os tinha visto — porque o **catálogo** os
   escondia (as elipses nascem alinhadas). Ao consertar o que o usuário vê, **vá procurar o que ele
   ainda não viu**.
3. **A fixture do gate é escolhida, e a escolha pode torná-lo incapaz de falhar.** O refino da fase:
   em θ = 45° a resposta cai **exatamente** sobre uma amostra da grade, o erro é **zero mesmo com o
   defeito ligado**, e o gate ficaria verde. Os ângulos do gate são **fora da grade** de propósito
   ([[feedback_zero_valued_fixture_is_a_gate_that_cannot_fail]]).
4. **Duas portas para a mesma pergunta divergem.** O gate reconstruía a lista de cortes à mão, sem a
   normalização que o produto faz — e por sorte isso **escondia** um defeito em vez de inventar um.
   Agora o produto e o gate chamam a **mesma** função (`pair_up`).
5. **Um `f64` nunca deve decidir de que lado de um empate ele caiu.** É a terceira vez que esta linha
   aprende isso (a borda da peça · a origem do ciclo). Onde há fronteira, há empate; onde há empate,
   **estabeleça o invariante em vez de esperar por ele**.

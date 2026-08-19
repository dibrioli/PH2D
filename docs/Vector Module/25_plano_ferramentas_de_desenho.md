# Plano — as ferramentas de DESENHO que faltam ao módulo vetorial

> `line/Vector`, 2026-07-29. Nasce de uma **auditoria de cinco agentes** pedida pelo Enio:
> *"No que se refere a desenho vetorial, nossa ferramenta ainda não tem alguma feature importante
> que deve ter para ser considerada um app de fronteira? … Por exemplo: percebo que não temos a
> ferramenta Lápis, nem faca, etc. … Podemos ter um pincel como em flip que aumenta a largura do
> stroke de forma suave onde pintamos?"*
>
> Escopo que ele deu: **ferramentas de desenho, manipulação de curvas, Boolean, Build**. Cada
> afirmação abaixo foi conferida no CÓDIGO (`arquivo:linha`) — os docs deste módulo têm notas
> obsoletas comprovadas, três delas escritas por mim nesta mesma semana.
>
> ⚠️ **Recorte de 2026-08-18.** As waves **W1 · W2 · W3 · W4 · W5 · W6** (§4-§9) fecharam, e o
> corpo delas foi movido **verbatim** para
> [`docs/archive/docs-2026-08-18/Vector Module/25_plano_ferramentas_de_desenho.md`](../archive/docs-2026-08-18/Vector%20Module/25_plano_ferramentas_de_desenho.md)
> — vá lá para *"como isto foi construído"* e para os números de cada uma. Ficou aqui o veredito
> (§1-§3), tudo o que segue **ABERTO** ou **⛔ recusado com medição**, a **§10 higiene**, a **§11
> fora desta rodada** e a **§12 ordem**. ⛔ Nada foi resumido — as duas metades remontam o original
> byte-a-byte (sha256).

## §1 — O veredito da auditoria em uma frase

O módulo é **excepcional em MODIFICAR** geometria não-destrutivamente (≈48 Live Shapes · 21 Live Path
Effects · 12 FX raster · Blend · Envelope com Coons+MLS · Contour · Pattern along path) e **quase não
tem como PRODUZIR** geometria à mão. **Todo caminho deste app nasce de cliques discretos** (a caneta)
**ou de um arrasto dimensionado** (as shape tools). Não há lápis, não há pressão, não há faca, não há
tesoura, não há borracha de caminho — e o ponteiro **nem carrega a pressão** até o código do vetor,
embora a shell já a tenha.

E o eixo de PRECISÃO nunca foi pesquisado: o `20_pesquisa_ferramentas_de_artista.md`, que dirigiu as
últimas seis waves, tem **zero linhas** sobre snap, guias, réguas, simetria, estabilizador, medição ou
autotrace.

## §2 — As decisões do Enio (2026-07-29), e o que cada uma fecha

| decisão | consequência |
|---|---|
| **"não haverá boolean recomputado por-frame"** — a D4 do ADR-0108 **MANTIDA** | ⛔ **O boolean VIVO sai do plano.** Nada roda o sweep por frame. O pathfinder da §8 é **destrutivo, edit-time**, e não toca a D4 |
| **Linha de runtime — agora não** | o `[lib]` da shell, os memos em espaço LOCAL e o dirty-tracking ficam **nomeados na §10, fechados a esta rodada** |
| **Width Tool completo — criar** | §5, **com ADR na frente** (é o único item onde a pesquisa do repo diz que não há caminho de prateleira) |
| **W6 (precisão) — implementar** | §9 entra: snap a caminho/interseções, guias e réguas, Mirror vivo |

⚠️ **A rota que a D4 não proíbe, registada para não ser re-descoberta:** união, XOR e
buraco-dentro-da-forma são **exatos e de graça HOJE** pela regra de preenchimento — concatenar
contornos num `BezPath` (o `build_path` já o faz) e escolher `NonZero`/`EvenOdd`, com o winding a
resolver antes do antialiasing. É a receita que a doc do **Rive** dá ao artista, e **nenhum boolean é
computado**. Fica **não construído** porque a pergunta que ela responde ("boolean vivo") foi fechada;
se um dia reabrir, esta é a porta, e a **interseção** não passa por ela (exige clip, e paga o
*conflation artifact* do `vello#49`, cujo caso de falha **é** o caso normal de uma diferença).

## §3 — O que a auditoria mediu, e que decide o TAMANHO de cada wave

**A maior parte do trabalho que falta é fiar o que já existe.** Estes são os ativos enterrados,
todos verificados:

| ativo | onde | estado |
|---|---|---|
| **`fit_hobby_open`** — solver de Hobby, spline interpolante, tridiagonal | `ph2d-vector-doc/src/hobby.rs:99` | **533 LOC, testado, ZERO chamadores.** O doc dele nomeia o consumidor que nunca chegou: *"the Pencil tool smooths a recorded freehand stroke"*, e a decisão de design ao lado (**não** é o least-squares do Schneider, *"the Inkscape default the plan flags as the anti-pattern"*) |
| **`pressure` / `tilt`** | `ph2d-editor-core/src/tool.rs:219-225` | Existem no `CanvasPointer` (*"Apple Pencil is first-class"*); o Painter e o Flip consomem. **Zero ocorrências de `pressure` nas quatro crates `ph2d-vec-*`** — o vetor descarta na fronteira |
| **`pressure_width_factor(pr, min, response)`** | `ph2d-tool-flip/src/params.rs:288` | A curva do Flip, **já calibrada em smoke e aprovada pelo Enio** (defaults 0,05 / 0,5) |
| **`lazy_mouse_step`** | `ph2d-painter-brush/src/stroke/stabilize.rs:83` | Estabilizador: função **livre, pública, `#[must_use]`, pura**. O doc já diz *"Shared by the Space-method stabilizer and the Free Hand capture"* — dois consumidores, **nenhum no vetor** |
| **`merged_segment_fit(prev, mid, next)`** | `ph2d-vec-scene/src/reshape.rs:242` | *A cúbica que passa pelas mesmas tangentes.* O **Simplify chama-a**; o **Delete ignora-a**. É o motor de **duas** ausências (§6) |
| **`nearest_point_on_path`** + **`split_segment`** | `ph2d-vec-scene/src/geometry.rs:277,239` | A tesoura são **estas duas chamadas** |
| **`subtract_all`** | `shells/desktop/src/shape_build.rs:326` | *fonte − união do que está acima* — o **Trim** do pathfinder, que o Build já usa |
| **`Arrangement`** | `ph2d-vec-boolean/src/arrangement.rs:70` | Faces planares de N formas; **já computa toda interseção** (o alvo de snap da §9) |
| **`fx_repeat`** com `spin`/`orbit` | `ph2d-vec-scene/src/effect_params.rs:234` | Multiplica contornos **vivo**; o **Mirror** da §9 é um irmão dele |
| **`VecLabel`** | `shells/desktop/src/label_live.rs` | Guarda uma **relação** e re-deriva a pose por frame, com undo e save de graça. Uma **cota** é exatamente isso |
| **`ph2d-flip-colorize`** | `crates/ph2d-flip-colorize/` | Já faz **raster → região → curva** tolerante a vão. O autotrace tem ~70% do motor no repo, **espalhado** ⇒ segue **G**, fora desta rodada |

⚠️ **E um ativo que NÃO é reuso:** o `WidthProfile` de 4 números (`ph2d-vec-scene/src/width_profile.rs`)
tem **dois consumidores, os dois destrutivos** (`Expand::PowerStroke` e a fiação dele). O próprio
header confessa: *"As alças na linha são um GESTO de canvas… outra wave"*. Ele é **semente**, não
solução — ver §5.

### ⚠️ W1b.2 — A PRESSÃO: bloqueada por um fato do SHELL, e é decisão de produto

O item 2 acima diz *"o `CanvasPointer` já a traz … mouse reporta `1.0`, então nada muda sem tablet"*.
**Metade disso está errada, e foi MEDIDO:** esta shell **constrói o `PointerEvent` com `pressure: 1.0`
literal** nos seus dois únicos sítios (`input_dispatch.rs`, `source: PointerSource::Mouse`), e o laço
de eventos **não casa `WindowEvent::Touch`** — o único evento do winit que carrega `force`. O
`CursorMoved`, que é o que a shell escuta, **não tem pressão nenhuma no protocolo**. Logo: hoje
nenhum dispositivo entrega pressão a este app, tablet incluído. Fiar a pressão no perfil produziria
um **fio morto** — a feature ficaria correta e invisível, e ninguém saberia se está quebrada.

**As duas saídas foram apresentadas e o Enio escolheu a (1)** (2026-07-30):

1. ✅ **Uma FONTE de largura** (`Uniform | Speed | Pen`) — o modelo do Krita/GP. *Speed* é o único
   que um mouse de facto dirige, então o artista vê a largura viva **hoje**, e a rota de pressão
   fica construída e gateada para o dia do tablet. **ENTREGUE, ver W1d abaixo.**
2. **O caminho do tablet** — casar `WindowEvent::Touch` e levar o `force` até o `PointerEvent`. É
   trabalho de INPUT da shell (não do vetor), afeta o **Flip do mesmo jeito** (o `flip_draw.rs`
   também recebe um `1.0` literal), e não pode ser verificado sem hardware. **Segue aberto**, e
   agora custa **UMA função** (`App::pointer_dynamics`) em vez de uma varredura.

---

# O QUE SEGUE ABERTO, E O QUE FOI RECUSADO COM MEDIÇÃO

> Recortes das waves fechadas. Cada bloco continua a valer; o contexto está no
> [arquivo](../archive/docs-2026-08-18/Vector%20Module/25_plano_ferramentas_de_desenho.md).

## §5 (W2) — a decisão de schema, e o que NÃO se reconstrói

### ⚠️ A decisão que o ADR tem de tomar: ONDE o perfil vivo mora

| rota | preço | veredito |
|---|---|---|
| campo novo no **`StrokeSpec`** | ele é `Serialize` e a `VecScene` vai **EMBUTIDA** no `ProjectState` (o `project.rs` o diz na entrada v10) ⇒ bumpa **`VEC_SCENE_SCHEMA_VERSION` 13→14 E `PROJECT_SCHEMA` 38→39**, e **recusa todo projeto já salvo** | ⛔ |
| **componente ECS `VecStrokeProfile`** | cunha `stable_type_id` próprio (`blake3(NOME)[..8]`), **não move layout posicional de nada** ⇒ **zero bump** | ✅ **recomendado** |

O componente é o padrão que este repo já usou **sete vezes** (`VecOffset` · `VecTextPath` ·
`VecEnvelope` · `VecBlend` · e a família de área da física), e a lei que o justifica está escrita sete
vezes também: **um bump recusa todo projeto já salvo**, e jogar fora trabalho real para evitar um
componente é o trade errado. Semanticamente também casa: um perfil de largura é atributo **por-path**,
que é exatamente o que o `VecOffset` é.

**Conteúdo:** lista `(posição, largura)` arbitrária (o Power Stroke do Inkscape) em vez dos 4 números ·
alças arrastáveis na curva (o `InteractiveState::CurvePoint` é o dispatch 2D compartilhado — o
precedente do repo) · perfis salvos · e o render a variar a largura **sem assar**. O `Expand::PowerStroke`
de hoje **fica** como a porta para quem quer a forma preenchida.

### ⚠️ O que a W1c JÁ entregou desta wave (não reconstrua)

A **representação inteira** e o **motor vivo** já estão de pé (ver §4, W1c): a lista de paradas
arbitrárias existe (`WidthStops`), o componente existe e é salvo/desfeito de graça, o `power_stroke`
já a consome, o cozimento vivo já roda por frame no z da forma, e o par *arrasta-e-vê* / *Apply* já é
o do Offset. **Zero bump** — a decisão da tabela acima foi tomada e executada.

**O que sobra para a W2, e só isto:** as **alças no canvas** (adicionar/mover/apagar uma parada
apontando a curva) e os **perfis salvos** (a lista por nome). O que NÃO sobra é decidir onde o
perfil mora nem escrever um segundo motor: quem o fizer estará a construir a segunda porta que o
ADR-0148 §3 proíbe.

## §6 (W3/W6.5) — o laço de TOQUE

**Aberto:** o laço de **TOQUE** (o `Alt+drag` do Inkscape: selecionar tudo o que o *caminho* cruza,
em vez do que ele cerca) é outra pergunta e outro gesto — não construído, e nomeado aqui para não
ser confundido com este.

## §7 (W4) — a borracha de caminho

**Aberto da W4:** a **borracha de caminho** — e a receita está PROVADA, não é pesquisa: *dois
cortes + apagar o pedaço do meio*, com `remove_path`/`remove_contour`, que já existem. O que falta
é o GESTO (press na curva → arrastar ao longo dela → release), não a geometria.

## §9 (W6.1) — ⛔ o cache que este plano previa NÃO se constrói

⛔ **A localização substituiu o cache que este plano previa.** Cruzamentos são `O(n²)` sobre os
segmentos, e a §9 dizia "cachear por gesto". Não é preciso: o snap só quer os cruzamentos a até
`max_d` do cursor, e filtrar os segmentos por essa vizinhança **antes** do laço par-a-par deixa o
custo em `O(n + k·n)` com `k` ≈ 0..4. Um cache seria memória que envelhece; a localização não tem
estado nenhum.

⚠️ **Um cruzamento numa PONTA é descartado**: ele é a âncora que dois segmentos vizinhos
compartilham por construção. Sem essa regra toda junta do desenho reportaria um "cruzamento", e o
ímã dos cruzamentos disputaria cada canto com o das âncoras.

## §9 (W6.3) — o MIRROR

**Aberto nesta wave, nomeado:**
- **"Discard original"** (o 3º modo do Inkscape — só o reflexo) **não foi construído**: como LPE
  ele é um *flip vivo*, e o repo já tem o Flip H/V destrutivo. Um parâmetro custa uma row para
  sempre; se o smoke o pedir, é uma linha.
- **A fusão aplica-se ao PRIMEIRO eixo**; com `Axes = 2` o segundo espelha o que saiu do primeiro
  (que é o que compor significa) — não há uma segunda fusão a decidir.

## §9 (W6.6) — as guias e a régua

**Aberto nesta wave, nomeado com o preço:**
- **A guia INCLINADA não existe**, e é decisão. Uma reta oblíqua é uma restrição 1-D que **não se
  decompõe em eixos** — encaixar nela move `x` E `y`, e o resultado é uma projeção perpendicular,
  não um alinhamento. É uma **terceira espécie** de reivindicação (a *linha*), com gesto próprio
  (criar, girar) e matemática própria; enfiá-la no tipo de hoje obrigaria o `pos: f64` a virar
  `(origem, direção)` e o laço de snap a ramificar em duas leis onde hoje tem uma.
- **A ORIGEM MÓVEL** (arrastar o canto para mudar o zero) **não foi construída, e o preço está
  medido**: a régua já lê a origem da GRADE (porta única), mas `GridSnapState` **não é
  persistido** — então uma origem movida seria um ajuste que ESQUECE. Fazê-la direito exige levar o
  estado da grade ao arquivo, que é decisão sobre uma struct foundational com 9 configs, não um
  gesto de canto.
- **O consumo é do Vector**: quem encaixa nas guias é o motor de snap vetorial. Levá-las ao gizmo
  de sprite dos outros modos é wave própria — o gesto já é tool-agnóstico, o ímã ainda não.

**A W6 FECHOU** — as seis linhas da tabela §9 estão entregues (W6.1 a reivindicação 2-D · W6.2 as guias e a régua · W6.3 a simetria · W6.4 a seleção de nós com dono · W6.5 o laço · W6.6 o rótulo de distância).

---

## §9 (W6.7) — ⚠️ a lista de ABERTOS foi RE-MEDIDA, e duas entradas já estavam fechadas

### ✅ W6.7 — O PAINEL DO VETOR FALA A UNIDADE DO ARTISTA (e a lista aberta é RE-MEDIDA)

A wave nasceu de uma varredura da lista de pendências, e **duas das três candidatas já estavam
fechadas** — as notas é que tinham envelhecido. Fica o registo, porque uma lista velha faz a
próxima LLM propor construir o que existe:

| item que a lista dava como ABERTO | medido |
|---|---|
| *"o hit-test só recebe o produtor de OFFSET"* | **FECHADO.** `App::vec_live_drawn` guarda o mapa FUNDIDO que o `dispatch` desenhou e os **seis** sítios de pick da `input_dispatch` leem-no. Dez produtores falam `LiveGeometry`; nove entram na fusão (o `fx_live` CONSOME o mapa e produz pixels, não geometria) |
| *"editar nós de VÁRIAS formas — ausência POR CONSTRUÇÃO"* | **FECHADO.** `selected_verts` é `Vec<(VecPathId, usize)>` — o dono viaja no par, e o `multi_probe` já foi reconferido (*"uma sonda escrita contra a ausência tem de ser reconferida no dia em que a ausência fecha"*) |
| *"o caminho do TABLET custa **uma função**"* | ⚠️ **MAL PRECIFICADO.** `winit 0.30.13` emite `force: None` nos **quatro** sítios de toque do Wayland e no do X11 (`// TODO`), e **não tem `zwp_tablet_v2`**. Não é fiação: é integração de plataforma ou upgrade do winit |

## §10 — W0: A HIGIENE (defeitos que a auditoria achou, não features)

Barata, e paga-se sozinha dentro da W1:

1. ⚠️ **O memo do `fx_live` não inclui a GEOMETRIA** (`p.ops == ops && p.w == w && p.h == h`) e
   `fx_live_tests.rs` **não tem um único gate de invalidação de memo** (18 testes, todos sobre
   resolução de ops/cores/hit). Translação é correta **por construção** (o rect é recomputado e o
   conteúdo é idêntico); o caso alcançável é **mudar a cor do fill** de uma forma filtrada, que
   mantém os pixels velhos. **Repro antes de fix.**
2. ⚠️ **`has_derived_verts` tem UM consumidor de produção** (o press das ferramentas de quina,
   `input_dispatch.rs:3448`) ⇒ arrastar âncora de uma **Live Shape** no modo Node talvez seja aceito e
   **revertido em silêncio** pelo recook do frame seguinte. **Repro antes de chamar defeito** — eu vi
   a ausência de chamada, não o sintoma, e a diferença importa.
3. ⚠️ **O `cooked()` roda DUAS vezes por forma preenchida por frame** (`build_bezpath` +
   `build_fill_bezpath`, ambos por `build_path` → `path.cooked()`) e **não memoiza**: clona e roda
   `run_stack`. Uma forma com efeitos coze a pilha inteira duas vezes por frame.
4. ⚠️ **A bench de encode regrediu 1,66× sem ninguém notar** — o spike de 2026-07-05 registra 10k
   formas em **0,77 ms** (ADR-0108 §5 apoia o kill-criterion de N=10.000 nele) e hoje mede **1,278**.
   A bench é `#[ignore]` e **nenhum gate pina o número**. Candidato de mecanismo: o item 3.
5. Comentário que **contradiz o código shipado** (`ph2d-vec-edit/src/lib.rs:295` diz que a alça de raio
   é do modo Node; `:330` diz que não é mais) · o segmentado de tipo de vértice publica só o kind do
   **primário**, então seleção mista mostra um dos três como se fosse o todo.

**Tamanho: P**, exceto o gate de razão da bench (**P–M**).

### ✅ W0 FECHADA (2026-07-29) — e a auditoria errou em dois pontos, os dois corrigidos

| item | veredito | commit |
|---|---|---|
| 1 · memo do `fx_live` sem geometria | **DEFEITO REAL, curado.** A chave era `(pilha, w, h)` ⇒ mudar a cor do fill de uma forma filtrada **não mudava a tela**. A decisão virou porta única headless (`fx_live_memo::job_for` → `FxKey`) e carrega o que é DESENHADO. A translação fica FORA de propósito (pan não re-cozinha). 7 gates, 3 mutações | `5ad38ee27` |
| 2 · `has_derived_verts` com um consumidor | **FENÔMENO REAL, mecanismo ERRADO na nota.** O `recook_into` **não roda por frame**: um nó arrastado numa Live Shape sobrevive até a próxima **edição de parâmetro**. Curado com a política que o par Fillet/Chamfer já tinha (congelar a receita no press), pela porta nova `PenTool::node_edit_hit_at` — porque o retorno do press **não distingue** *agarrou um vértice* de *selecionou a forma*. ⚠️ Os hosts de RELAÇÃO ficam ABERTOS e nomeados no `corner_handles.rs` (congelar não serve ali; a cura é RECUSAR, decisão de produto) | `d64e0…` |
| 3+4 · `cooked()` 2× por forma · bench regredida 1,66× | **UM defeito, não dois.** O `draw_path` construía o caminho completo **incondicionalmente** e o jogava fora numa forma só-preenchida, e cada construção COZIA. Medido: **10k formas 1,323 → 0,901 ms** (1,47×). ⚠️ O resíduo contra os 0,77 do spike (**1,17×**) fica NOMEADO, não atribuído | `43b1c…` |
| 5 · comentário mentiroso · seleção mista | **OS DOIS reais.** A leitura devolvia o tipo do vértice PRIMÁRIO e o painel afirmava-o sobre a seleção inteira ⇒ `SelectedKind::{Uniform,Mixed}`, e misto **não acende chip** (com os chips ainda VIVOS — retipar é o gesto que sai do misto) | `dd0828c98` |

⚠️ **E um gate de perf nasceu não-discriminante e foi REESCRITO:** a 1ª versão do orçamento de encode
era uma **razão de tempo** entre a mesma geometria só-preenchida e só-traçada, e a mutação
**SOBREVIVEU** — medido com o defeito reinstalado, `fill 4,862 ms` contra `stroke 5,255 ms` = 0,93×.
Um encode de FILL e um de STROKE não fazem o mesmo trabalho no Vello, e a diferença é maior que a
construção inteira que o gate queria isolar. O oráculo virou o **NÚMERO** de construções e de
cozimentos (contadores `#[cfg(test)]`), que é exato e é literalmente a propriedade em questão.

## §11 — Fora desta rodada, por decisão, com o preço nomeado

- **A linha de RUNTIME** (*"agora não"*): a shell é **`[[bin]]` sem `[lib]`** e todo produtor vivo é
  módulo **privado do binário** ⇒ um jogo que embarque o PH2D **não alcança** o cozimento
  não-destrutivo.
- **⚠️ E o memo chaveado no MUNDO foi RE-MEDIDO pela porta do PRODUTO — a nota anterior estava
  300× errada no item mais caro dela** (2026-08-12, sonda `live_memo_probe` na shell,
  `cargo test -p ph2d-host-desktop --release live_memo -- --ignored --nocapture`). A premissa
  continua verdadeira — todo produtor bake a pose DENTRO da chave, então uma forma que anda
  **re-cozinha em todo quadro** —, mas o preço não é o que estava escrito. Medido, `--release`,
  máquina calma (`load 0,87`), estrela de 5 pontas, 60 quadros por coluna, **mediana**:

  | produtor | PARADO | ANIMADO | razão | saída |
  |---|---|---|---|---|
  | contour (16 anéis, Round) | 0,004 | **0,039** | 10× | 17 caminhos |
  | offset (`d = 0,12`) | 0,000 | **0,686** | 5273× | 1 |
  | profile (0,2/1,8/0,2) | 0,000 | **1,655** | 7522× | 1 |
  | symmetry (1 eixo) | 0,000 | 0,000 | 3,7× | 2 |

  ⚠️ **O `12,25 ms` do contour vinha do `probe_contour_cost`, que chama `offset_path` — a
  BOOLEANA.** O produto não a chama no caso comum: `contour_live::cook_piece` tenta primeiro o
  **`offset_ring`** (o offset DIRETO, *~668× mais barato*, que o próprio header do módulo declara
  como *a cura de verdade*) e só cai na booleana no fallback (compound · Inner · Both). Quem moveu
  o número foi a wave do offset direto; a nota nunca foi reconciliada, e declarava *"não cabe num
  quadro"* uma feature que cabe **400× dentro** dele. *O número que vira decisão de produto tem de
  sair da porta do produto* — a mesma lição que o Painter pagou três vezes (doc 28 §5.40).
  ⚠️ **A sonda tem CONTROLE, e ele mordeu na 1ª corrida:** o `require` recusa uma coluna cujo
  produtor não produziu geometria — a fixture do profile armava `VecStrokeProfile::default()`,
  cujo `stops` é **VAZIO** (o neutro que a shell REMOVE), então ela media o custo de um `continue`
  e teria reportado *"o perfil é grátis"*. E a coluna **saída** existe pela mesma razão: `0,04 ms`
  sobre dezasseis anéis e `0,04 ms` sobre um caminho degenerado são leituras opostas.
  **O que sobra ABERTO, com o número certo:** o **profile a 1,655 ms/forma/quadro** (10% de um
  quadro de 60 Hz por forma animada — o maior da tabela, e um número que ninguém tinha) e o
  **offset a 0,686**. A **simetria erra o memo e não custa nada** (`xform` está na chave e o cook
  dela nem o consome — ela coze em LOCAL e assa a pose na SAÍDA; tirar o `xform` da chave é exato
  e **compra zero medido**, então fica nomeado e não construído). A cura dos dois caros é a mesma
  que a nota antiga já apontava — memoizar em espaço **LOCAL** e assar o afim na saída, exato para
  **similaridade** (`d_local = d / s`; uma translação é `s = 1`, que é o caso que paga hoje) com
  fallback ao mundo sob escala não-uniforme ou skew — mais o **dirty-tracking** que o ADR-0108 D3
  chama de *a* alavanca e que nunca foi construído. ⚠️ **É wave própria**, então não entra de
  carona numa medição.
- **⚠️ E a PREMISSA dessa cura foi MEDIDA no mesmo dia — ela não é aproximação, e a frase
  *"muda o DESENHO na fronteira da aproximação"* que estava aqui foi CORRIGIDA por isto**
  (sonda `live_memo_commutation_probe` + `..._profile_probe`,
  `cargo test -p ph2d-host-desktop --release live_memo_commutation -- --ignored --nocapture`).
  A rota local e a rota de mundo dão o **MESMO** resultado sob qualquer similaridade, e o
  controle não-uniforme bate maior que o próprio offset:

  | pose | offset, desvio máx. de vértice | perfil, largura crua | perfil, largura / s |
  |---|---|---|---|
  | translação | 0,000000 | 0,000000 | 0,000000 |
  | rotação | 0,000000 | 0,000000 | 0,000000 |
  | escala uniforme 1,6 | 0,000000 | **0,043200** | 0,000000 |
  | similaridade completa | 0,000000 | **0,043200** | 0,000000 |
  | não-uniforme (**CONTROLE**) | **0,140290** | 0,088193 | 0,078133 |

  ⚠️ **O perfil responde a pergunta de ESCOPO da wave**, e a aritmética confere à mão: o
  `bake_xform` escala *todo comprimento escalar do path* (o raio do gradiente, o `corner_radius`)
  e **não** escala o `StrokeSpec.width` — então a rota local INGÊNUA erra por exactamente a
  meia-largura no stop do meio, `(s − 1) · (W/2) · mid = 0,6 · 0,04 · 1,8 = 0,043200`. Dividir a
  largura pela escala antes de cozer restaura a exactidão: é o **gêmeo do `d_local = d / s`**, e
  não uma segunda regra. ⇒ **a wave não muda desenho nenhum**; o que ela custa é o ramo de
  fallback (não-similaridade) mais o bake na saída.
  ⚠️ **E o ORÁCULO estava errado antes de o produto estar:** a 1ª versão comparava os dois
  contornos **índice a índice** e reportava a rotação a **1,25 unidades numa forma de raio 1** —
  grande demais para ser aritmética, e o offset é equivariante por rotação por construção. Um
  contorno fechado é uma sequência **CÍCLICA** e o `linesweeper` elege o vértice inicial pelas
  **coordenadas**; só translação e escala uniforme preservam a ordem lexicográfica, que é
  exactamente por que só essas duas liam zero. *A rotação da LISTA estava a ser lida como desvio
  geométrico*, e a conclusão pronta seria **"a cura não comuta sob rotação"** — falsa. O oráculo
  passou a ser a **distância de Hausdorff discreta dos dois lados**, que não conhece ordem
  nenhuma.
- **Boolean vivo** (D4 mantida) — a rota por regra de preenchimento está descrita na §2 para não ser
  re-descoberta como novidade.
- **Divide · Outline · edição de nós multi-forma · autotrace · grade perspectiva** — todos **G**, todos
  nomeados no lugar onde a wave que os tocaria os encontraria.
- **Vetor → collider de física não existe** (zero referência a `VecPath` em `ph2d-physics-ecs`) e a
  **ADR-0063 é letra morta** — a ADR-0131 §12 a rejeita explicitamente por estar amarrada ao
  vector-runtime que a 0108 aposentou.

## §12 — A ordem recomendada

**W0** (higiene, e desarma armadilhas para as seguintes) → **W1 A MÃO** (o pedido do Enio; todas as
peças existem) → **W2 O WIDTH TOOL** (ADR primeiro; compartilha a espinha do perfil vivo com a W1) →
**W3 O ALCANCE DO NÓ** (as duas ausências que dividem um motor pronto) → **W4 OS CORTES** → **W5 O
PATHFINDER BARATO** → **W6 PRECISÃO**.

⚠️ **Estado da linha:** `line/Vector` está **FECHADA**, com **sete** handoffs a compartilhar
`PROJECT_SCHEMA` 38 e nenhuma integração autorizada. Nada deste plano começa sem ordem explícita do
Enio — e se a W2 tomar a rota do componente ECS (§5), **nenhuma destas waves bumpa schema**, o que as
mantém fora da disputa de número com as outras linhas.

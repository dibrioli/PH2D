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

## §4 — W1: A MÃO (o pedido do Enio)

**O que o artista ganha:** desenhar à mão livre, com a largura a responder à pressão suavemente, e com
a mão estabilizada — o gesto que hoje existe em **raster** neste app e não em vetor.

Três peças, e **nenhuma pede matemática nova**:

1. **O lápis** — o gesto grava amostras, decima, e ajusta por **`fit_hobby_open`**. Um 11º `DrawMode`.
   ⚠️ O fitter fala `glam::Vec2` (f32) e a engine nova é `[f64;2]`: **portar o solver para f64 num
   módulo leaf** é mais limpo que criar a aresta de dependência para a crate congelada. O gate do §6
   conta **variantes de enum e campos de struct** (`VectorOp` ≤ 16 · `Vertex` ≤ 5 · `Segment` ≤ 6 ·
   `Region` ≤ 5 · `VertexKind` = 4) ⇒ **nada disto o toca**, mas o port evita a discussão inteira.
2. **A pressão atravessa a fronteira** — o `CanvasPointer` já a traz; o press/drag do vetor passa a
   carregá-la. **Mouse reporta `1.0`**, então nada muda sem tablet.
3. **O estabilizador** — `lazy_mouse_step` fiada no laço de captura + um slider. **Porta única:** a
   MESMA função que o Painter usa; uma segunda suavização divergiria do que o artista já aprendeu.
   ⚠️ **Não confundir com o RDP + Catmull-Rom do Flip**, que é suavização **pós-hoc** e não substitui.

**A largura viva é o que faz disto um pincel** e é a espinha compartilhada com a §5: a pressão escreve
num **perfil de largura VIVO** (não no bake). Ver §5 para onde ele mora.

**Tamanho: M.** Smoke: desenhar um S com pressão variável e ver a largura acompanhar; com estabilizador
a 0 e a 1, a mesma mão dá traços diferentes.

## §5 — W2: O WIDTH TOOL (aprovado pelo Enio — **ADR na frente**)

**O que o artista ganha:** o Width Tool do Illustrator — **pontos de largura arbitrários** na curva,
adicionáveis, movíveis, duplicáveis e apagáveis, com **perfis salvos**; e o traço fica **VIVO**, não
assado.

**Por que exige ADR:** é o único item da auditoria onde a pesquisa do próprio repo
(`20_pesquisa_ferramentas_de_artista.md:96-118`) diz que **não há de prateleira** — nem kurbo nem Skia
têm largura variável; existe o caminho do Levien (*stroke expansion*, arXiv 2405.00127) e mais nada. E
há uma decisão de representação a registar.

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

**Tamanho: G.** Smoke: um traço com 5 pontos de largura, arrastados; o mesmo traço sob Expand tem de
dar a MESMA silhueta (a paridade vivo↔assado é o gate que impede duas respostas).

## §6 — W3: O ALCANCE DO NÓ

Duas ausências que **compartilham um motor que já está no repo** (`merged_segment_fit`):

1. **Apagar um nó preservando a forma.** Hoje `delete_selected_vertex` é literalmente
   `verts.remove(local)` + limpeza de contorno degenerado (`selection.rs:306-316`) — **nenhum handle
   vizinho é tocado**, e a curva morre com o ponto. É a operação de nó mais usada em qualquer app.
   ⚠️ Bônus: o tracker do Inkscape (issue #5077) admite que o refit deles *"fica muito chato"* —
   fazer o nosso melhor é **diferencial**, não paridade.
2. **Arrastar o SEGMENTO.** O `enum Part` (privado, `ph2d-vec-edit/src/lib.rs:57`) tem `Anchor` ·
   `In` · `Out` · `Radius` — **não há `Segment`**, e pressionar sobre a curva **INSERE um nó**. Não se
   pode reformar uma curva sem alterar a topologia dela, que é o gesto que Illustrator e Inkscape
   documentam como o normal.

E **a escala da seleção**, que é o que faz o app não parecer de fronteira: o marquee **exige Shift**,
**substitui** em vez de somar, e vê **um path só**; não há **lasso**, `Tab`, select-all de nós, select
por tipo, select de sub-caminho, nem **X/Y numérico do nó**. Trabalhar uma forma de 40 nós é
clique-a-clique.

⚠️ **Editar nós de VÁRIAS formas é ausência POR CONSTRUÇÃO** (o `selected_verts` pertence a um
`selected` único) ⇒ **G**, fica **fora** desta wave e nomeada aqui.

**Tamanho: M.** Smoke: apagar o nó do meio de um arco e a curva ficar; arrastar o meio de um segmento
e a topologia não mudar; marquee sem Shift, aditivo, sobre duas formas.

## §7 — W4: OS CORTES

**Tesoura** (`nearest_point_on_path` + `split_segment` — **P**) · **borracha de caminho** (a matemática
por arco do `fx_trim`, destrutiva e local) · **faca** (o `Arrangement` cobre fechadas; ⚠️ **abertas
exigem rota nova** — o motor devolve `Error::NonClosedPath` e a nossa política é `Closing::Always`) ·
**Join como comando de seleção** (o `merge_path_into`/`weld_junction` já fundem; falta o botão e a
regra de seleção) · **Average** de nós (3 linhas sobre `selected_verts`) · e o botão do
**`reverse_path`**, que existe com **um chamador interno e nenhum id de UI**.

**Tamanho: M.** Smoke: cortar uma forma fechada em duas abertas com a tesoura; a faca a atravessar
três formas de uma vez; Join a fechar duas pontas.

## §8 — W5: O PATHFINDER BARATO (edit-time, não toca a D4)

Temos **4 das 10** operações (`Union`/`Intersect`/`Subtract`/`Exclude`). Quatro das seis que faltam são
baratas sobre primitivas que já existem:

| op | como | tam. |
|---|---|---|
| **Minus Back** | a fatia de z **invertida**; o fold já é `base − (∪ resto)` | **P** |
| **Trim** | por fonte, subtrai a união do que está **acima** — é o `subtract_all` do Build, verbatim | **P–M** |
| **Crop** | `Intersect` com a forma do topo + descartar o topo. Zero geometria nova | **P–M** |
| **Merge** | Trim + classe de equivalência por `fill` — e `VecPath.fill` já deriva `PartialEq` | **M** |

**As duas caras ficam nomeadas e fora:** **Divide** exige a varredura **N-ária única**
(`Topology::from_paths`, que existe no crate e o nosso `arrangement.rs:36-42` já nomeia como o escape
hatch; via `Arrangement` são `2^N` regiões a ~140 µs ⇒ **segundos** no teto de 16 formas) e **Outline**
exige saída de caminho **ABERTO**, hoje estruturalmente impossível (`compound_from` crava
`closed: true`; a conversão recusa < 3 vértices).

⚠️ **Enquadramento:** o **Rive não tem boolean nenhum** — nas 4 ops que temos já estamos à frente da
referência do ADR-0108; o gap é contra Illustrator/Figma/Affinity/Inkscape.

**Também nesta wave, e é higiene de robustez:** o `linesweeper::Error` é **engolido**
(`binary_op(...).ok()?`, `lib.rs:147`) ⇒ **falha do sweep e resultado legitimamente vazio são
indistinguíveis**, num crate que se autodeclara *early beta*, e o artista vê o mesmo nada. Propagar +
toast: **P**. O núcleo não tem **um único gate** de degeneração (tangência, área zero, NaN,
auto-interseção) — e a cura já está escrita duas portas ao lado (`the_offset_never_takes_the_app_down.rs`,
`probe_offset_fine_sweep.rs`, `robust_tests.rs`): **M**.

## §9 — W6: PRECISÃO (aprovada pelo Enio)

**O motor de snap existe, é bom e ESTÁ ligado** — módulo puro com eixos X/Y **independentes** (a lei do
Figma), a grade universal do editor injetada por closure, ativo no press da caneta, no arrasto de
âncora, nas shape tools, no translate e no scale do gizmo. **O que falta é a LISTA DE ALVOS:** o
`collect_targets` empurra exactamente **a âncora de cada vértice** e os **9 pontos-chave da bbox**.

| item | o que falta | reuso | tam. |
|---|---|---|---|
| **Snap a CAMINHO** | pousar um nó **sobre** uma curva — reivindicação **2D**, não por-eixo | `nearest_point_on_path` | **M** |
| **Snap a INTERSEÇÕES** | expor os cruzamentos + cachear por gesto | `Arrangement` já os computa | **M** |
| **Guias e réguas** | **ZERO hoje** (o único "guide" no repo é o caminho-guia do pattern) — guia arrastável, persistida, 3ª classe de alvo, régua nas bordas, origem móvel | `VecLabel` é o molde de componente derivado | **G** |
| **Mirror / simetria VIVA** | espelho ao desenhar; hoje só há **Flip H/V destrutivo** | o `fx_repeat` já multiplica com `spin`/`orbit` ⇒ um `Mirror` na pilha de LPE. É o que a Illustrator shipou em 2021 como *live symmetry* | **M** |
| **Rótulo de distância** nos smart guides | a linha-fantasma já existe (e é melhor que a média: segmento **entre** os pontos, não linha infinita) | `VecLabel` | **M** |
| **Snap a SPRITES** | nenhum competidor tem, porque nenhum mistura raster e vetor na mesma árvore — **nós misturamos** (ADR-0110) | a bbox já está na shell | **P** |

**Fora desta wave, nomeados:** grade **perspectiva** (**G**) · grade **polar** (10º `GridKind`, **M**) ·
os *planos* da isométrica · **Transform Again** (⚠️ **`KeyD` já é o boolean Subtract** — não pode nascer
com o atalho do Illustrator) · **Measure tool / cotas** (**M** sobre `VecLabel`) · **autotrace** (**G**,
motor espalhado).

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

## §11 — Fora desta rodada, por decisão, com o preço nomeado

- **A linha de RUNTIME** (*"agora não"*): a shell é **`[[bin]]` sem `[lib]`** e todo produtor vivo é
  módulo **privado do binário** ⇒ um jogo que embarque o PH2D **não alcança** o cozimento
  não-destrutivo. E **todo memo de geometria é chaveado no MUNDO**, que é o que a animação muda:
  medido pelas sondas do repo, em release, um **Contour de 16 anéis animando custa 12,25 ms/frame**
  (74% do quadro, sozinho) e 32 anéis **23,58** (não cabe); offset 0,40–1,07 ms/forma/frame; silhueta
  até 1,78; **pattern não tem memo**. A cura é local (memoizar em espaço **LOCAL** e assar o afim na
  saída — para similaridade offset e silhueta **comutam** com o afim) mais o **dirty-tracking** que o
  ADR-0108 D3 chama de *a* alavanca e que nunca foi construído.
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

# Pesquisa — nesting (containers de animação aninhados)

> Wave de 4 frentes, limitada (uma por pergunta, sem recursão), 2026-07-18.
> Alimenta o [ADR-0133](../architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md).
> As 3 perguntas vêm do [`BRIEFING_line_nesting.md`](../BRIEFING_line_nesting.md) §2.

---

## §0 — O que a pesquisa mudou no plano

Duas coisas, e as duas encolhem o trabalho:

1. **A pergunta 1 e a pergunta 2 têm a MESMA resposta.** O relógio é do pai (4 dos 5
   produtos), e o campo que o pai usa para mandar no filho é exatamente o conjunto que o
   nosso `ClipStrip` já tem. Um container instanciado não é um mecanismo novo — é um strip
   cuja fonte é um container em vez de um clip.
2. **A tensão "aba vs breadcrumb" era falsa.** Os dois eixos são ortogonais: a breadcrumb
   diz *onde você está*, a aba diz *qual metade você olha*. Nenhum produto escolhe entre eles.

E uma correção ao briefing: **o Figma não tem breadcrumb** (é deep-select + salto para a
fonte). Ele não serve de precedente para o gesto de "entrar", ao contrário do que o §2.3 sugeria.

---

## §1 — De quem é o relógio

| Produto | Quem responde | Composição, em ordem |
|---|---|---|
| **After Effects** | **o PAI**, sempre | `comp_time` → in-point/offset da layer → Time Stretch → Time Remap (quando ativo, **substitui**: o valor da propriedade *é* o `source_time` absoluto, não um delta) → `source_time`; dentro do precomp a cadeia recomeça por layer |
| **Rive** | **o PAI empurra Δt; o filho escala** | `NestedArtboard::advanceComponent()`: `localElapsed = parentElapsed × speed()` → avança cada `NestedAnimation` → `advanceInternal()` |
| **Animate — graphic** | **o PAI**, frame a frame | `child_frame = f(parent_frame − instance_start, First, modo)`, `modo ∈ {Loop, Play Once, Single Frame, Reverse Loop, Reverse Once}` — indexação inteira, sem Δt |
| **Animate — movie clip** | **o FILHO** (relógio próprio) | `mc_time += Δt_do_player`; nenhuma relação com o frame do pai |
| **Harmony — symbol** | **o PAI**, por *exposure* | key exposure escolhe a célula do símbolo e a repete até a próxima |
| **Cavalry** | **o PAI** | Time Remapping em **%** da duração; `Preserve Frame Rate` quantiza; `Scheduling Group`/`Child Offset` deslocam em segundos |

**Quatro dos cinco dão o relógio ao pai.** O único filho com relógio próprio — o movie clip —
é justamente o que gera o maior corpus de confusão (§1.1).

### 1.1 Os modos de falha, por produto

**Animate** é o campeão, e sempre pelo mesmo motivo: *"Movie Clips will NOT show their nested
animations unless you double-click to go inside"*; *"animated symbols with own timeline not
showing on main stage timeline"*; *"why movie clips dont play on preview?"*. O relógio próprio
torna o filho **invisível em autoria** — ele só toca em runtime.
([1](https://community.adobe.com/t5/animate-discussions/animations-in-movieclip-not-shown-on-stage-and-not-exported-as-gif/m-p/12638571) ·
[2](https://community.adobe.com/t5/animate-discussions/animated-symbols-with-own-timeline-not-showing-on-main-stage-timeline/td-p/13328549) ·
[3](https://community.adobe.com/t5/animate/why-movie-clips-dont-play-on-preview/td-p/12076840))

O 2º modo de falha do Animate é o oposto: um *graphic* que "não anima" porque o span do pai é
curto demais — *"you need to give a graphic symbol enough room in its parent timeline to play
out its animation"*. O pai **trunca** o filho.

**After Effects** tem quatro, e o último é o que mais importa para nós:
- `Collapse Transformations` **desliga em silêncio** `Preserve Frame Rate When Nested`
  ([CreativeCOW](https://creativecow.net/forums/thread/collapse-transformations-disables-preserve-frame-r/));
- *"Preserving frame rate option in nested compositions has no effect"*
  ([CreativeCOW](https://creativecow.net/forums/thread/preserving-frame-rate-option-in-nested-composition/));
- retimar dentro do precomp **não muda o comprimento da layer no pai**;
- ⚠️ **bug aberto**: Essential Properties **não se aplicam depois do frame 0 quando Time
  Remapping está ativo no precomp**
  ([Adobe](https://community.adobe.com/t5/after-effects-bugs/essential-properties-on-precomp-are-not-applied-after-frame-0-when-time-remapping-enabled-on-precomp/idi-p/14587569)).
  **É a classe exata de bug a evitar: um canal do pai morre quando o filho é remapeado.**

### 1.2 Dois relógios na UI — quem faz, e como

**O After Effects faz, literalmente com duas réguas.** Ao ligar Time Remapping, *"a second
time ruler appears in the Layer panel above the default time ruler… On the upper time ruler,
the remap-time marker indicates the frame currently mapped to the time indicated on the lower
time ruler."* Régua de baixo = tempo da comp; de cima = tempo da fonte; um marcador liga as duas.
Além disso, a preferência **Synchronize Time Of All Related Items** (ligada por default)
propaga o playhead entre comps aninhadas.

**Harmony** sinaliza por *forma*, não por régua: o símbolo aparece como **"movie strip"** na
célula — a textura diz "aqui dentro há outro tempo".

**Animate** não mostra duas réguas, e é por isso que a confusão do §1.1 existe: o segundo
relógio é o campo `First` + um dropdown no Properties.

**É a única mitigação de UI que alguém de fato implementou.**

---

## §2 — Entidade com filhos, ou asset referenciado?

| Produto | Modelo | Instância |
|---|---|---|
| Animate | referência-a-asset (Library) | compartilha a definição; overrides na instância |
| After Effects | referência-a-asset (comp é item de projeto) | compartilha; overrides via Essential/Master Properties |
| Rive | referência-a-asset | **COPIA**: `NestedArtboard::clone()` → `m_referencedArtboard->instance()` |
| Unity | híbrido (prefab = árvore salva como asset) | ponteiro + diff serializado |
| Godot | híbrido, o mais explícito (`PackedScene` = árvore serializada) | re-hidrata; edição local vira diff na cena PAI |
| Blender | híbrido (`Empty.instance_collection`) | compartilha; instância só tem transform |

**Ninguém escolheu árvore-de-objetos pura.** Unity, Godot e Blender parecem árvore, mas os três
promovem a árvore a **ID endereçável** e a instância a **referência + diff**. O híbrido é a maioria.

### 2.1 Detecção de ciclo — o padrão é duas camadas, e nenhuma é em runtime

**Godot** — duas camadas, verificadas no fonte:
- ao instanciar: `SceneTreeDock::_cyclical_dependency_exists()` (DFS) → *"Cannot instantiate the
  scene '%s' because the current scene exists within one of its nodes."*
  ([scene_tree_dock.cpp](https://github.com/godotengine/godot/blob/4.3-stable/editor/scene_tree_dock.cpp));
- ao **salvar**: `_validate_scene_recursive()` → *"This scene can't be saved because there is a
  cyclic instance inclusion."*
  ([editor_node.cpp](https://github.com/godotengine/godot/blob/4.3-stable/editor/editor_node.cpp)).

A 2ª existe porque a árvore vira cíclica por caminhos que **não passam pelo "add"** (reparent,
script, edição externa).

**Unity** — pré-check no gesto: `CheckIfAddingPrefabWouldResultInCyclicNesting(...)` +
`ShowCyclicNestingWarningDialog()`, chamados do drag da Hierarchy, do `SceneView`, do
`GameObjectInspector` e do `PrefabStage`; na camada de API,
`throw new ArgumentException("Cyclic nesting detected")`
([PrefabUtility.cs](https://github.com/Unity-Technologies/UnityCsReference/blob/master/Editor/Mono/Prefabs/PrefabUtility.cs)).

**Blender** — três camadas, incluindo **reparo**: recusa silenciosa em `collection_object_add()`,
erro de UI em `BKE_collection_object_cyclic_check()`, e **`BKE_collection_cycles_fix()` chamado
no LOAD do arquivo**, que acha o ciclo e o conserta zerando `instance_collection`
([collection.cc](https://github.com/blender/blender/blob/main/source/blender/blenkernel/intern/collection.cc)).

**After Effects** recusa em silêncio no drag (cursor "não permitido"), sem diálogo — mensagem de
erro: não encontrada. **Rive**: detecção de ciclo **não documentada** em lugar nenhum.

### 2.2 O conjunto MÍNIMO de override por instância

O que **todos** oferecem sem forkar o asset:
1. **Transform** (a pose da instância);
2. **qual tempo/animação toca dentro** (Animate: loop mode + `First`; AE: time-remap/stretch;
   Rive: qual animação + `speed`/`mix`);
3. **visibilidade / opacidade**.

Comum a 3+: cor/tint e blend mode. O AE é o mais restritivo de propósito — só o que o autor
**expôs** (Essential Properties) é overridável: curadoria explícita, não "tudo".

### 2.3 Quem escolheu árvore, e que dor teve

**Blender** é o mais próximo de árvore-primeiro, e é quem tem a dor documentada: os ciclos
**acontecem mesmo assim**, então o código carrega um conserto automático no load que **apaga o
`instance_collection` do usuário em silêncio**; e a instância é *"non-editable"*, o que exigiu um
subsistema inteiro à parte (Library Overrides) só para ter override por instância.

---

## §3 — O que a UI mostra ao "entrar"

| Produto | Modelo | Entrada | Saída |
|---|---|---|---|
| **Animate** | **edit-in-place com breadcrumb** | duplo-clique no palco | clica o nome da cena na *Edit bar* |
| **After Effects** | **aba nova** | duplo-clique na layer de precomp | aba do pai · Composition Navigator · Mini-Flowchart (Tab) |
| **Harmony — grupo** | **expande em linha** (não se "entra") | seta na Timeline | colapsa |
| **Harmony — símbolo** | **entra noutra cena, timeline independente** | duplo-clique; `Ctrl+E` | botão **Top**; `Ctrl+Shift+E` |
| **Rive** | vai à fonte (sem edit-in-place documentado) | não encontrado | n/a |
| **Figma** | ⚠️ **deep-select + salto** — *não* é breadcrumb | duplo-clique desce um nível **dentro da instância** | *Go to main component* te tira do contexto |

**Edit-in-place com breadcrumb é a linhagem da animação 2D** (Flash/Animate, Harmony);
**aba nova é a linhagem da composição** (AE).

Consenso formal de que a aba nova é pior: **não encontrado**. O que existe é o *sintoma*,
repetido — perde-se o contexto do pai (*"editing a precomp… you can no longer see the effect of
the change in the main comp"*), e a comunidade **reconstrói manualmente** o edit-in-place
travando um viewer ou abrindo `View > New Viewer`. A Adobe respondeu com **navegação**
(Composition Navigator, Mini-Flowchart) e **sincronia de tempo** — nunca com edição in-place.

**Ao entrar, a timeline troca inteira** em todos os que "entram", e a régua passa a medir o tempo
local do container. Harmony é explícito: *"you are entering another scene where you have an
independent timeline for your symbol"*.

⚠️ **A exceção que importa:** os **grupos** do Harmony não trocam nada — expandem como linhas
aninhadas, sob o mesmo relógio. **Harmony separa deliberadamente organização (grupo, expande) de
nesting temporal (símbolo, entra).**

---

## §4 — Custo

### 4.1 As três armadilhas mais citadas

**(a) Cada nível não-colapsado materializa um raster intermediário, e a estrutura morre ali.**
No AE, sem *Collapse Transformations*: *"Comp 2 receives only the composited frame (a 'flattened'
image) and has no history of the layers in the first comp"*
([ProVideo Coalition](https://www.provideocoalition.com/cmg_hidden_gems_chapter_20_-_collapsing_transformations/)).
Custo = um buffer + reamostragem **por nível**, e escalas aninhadas se compõem destrutivamente.

**(b) O container é avaliado INTEIRO, todo frame, mesmo que nada mude.** No Rive é literal:
`advanceComponent()` chama `advanceInternal()` — *"the full artboard update"* — recursivamente,
**sem cache de saída**. E instâncias **não compartilham estado**: cada `NestedArtboard` tem seu
`unique_ptr<ArtboardInstance>`. **N instâncias = N avaliações completas, sem desconto.**

**(c) Uma propriedade no nó pai derruba o caminho rápido da subárvore inteira — em silêncio.**
Adobe: uma máscara fechada, um layer style ou um efeito num precomp colapsado força render
separado antes da composição. Pior: efeitos **desativados** ainda quebram o collapse — desligar
não basta, tem que deletar ([Adobe Community](https://community.adobe.com/t5/after-effects-discussions/problem-with-collapse-layers-precomps/td-p/9017339)).

### 4.2 Limites de profundidade: ninguém publica um número medido

| Produto | Limite | Recurso | Fonte |
|---|---|---|---|
| After Effects | não encontrado | — | [helpx](https://helpx.adobe.com/after-effects/using/precomposing-nesting-pre-rendering.html) |
| Rive | não encontrado — **sem depth guard e sem detecção de ciclo** documentados | — | [DeepWiki](https://deepwiki.com/rive-app/rive-runtime/2.4-nested-artboards-and-component-lists) |
| Animate | *"practically any number of movie clips inside of each other"* | — | [helpx](https://helpx.adobe.com/animate/using/multiple-timelines.html) |
| Spine | **nesting não existe** (enhancement aberto desde 2016) | — | [spine-editor#8](https://github.com/EsotericSoftware/spine-editor/issues/8) |
| Lottie | não encontrado | — | [lottie-web#793](https://github.com/airbnb/lottie-web/issues/793) |

**Nenhum limite justificado por stack, memória ou tempo, em produto nenhum.** Um teto nosso terá
de vir da nossa própria medição — que é o que o CLAUDE §0.0 já exige.

⚠️ **Sinal do Spine, e é um aviso de escopo:** o motor 2D esqueletal mais maduro do mercado
**nunca implementou nesting em 10 anos**. A razão estrutural está na doc: cada skeleton é
desenhado inteiro antes do próximo, não dá para intercalar draw order entre skeletons — *"if that
is needed, it is easiest to use a single skeleton"*
([esotericsoftware](http://en.esotericsoftware.com/spine-skeletons)).
**Nesting e ordem de desenho global brigam.**

### 4.3 Cache e flatten

- **AE cacheia frames de composição**, e o cache é reaproveitado entre instâncias da mesma
  precomp *quando os parâmetros batem*
  ([Composition Profiler](https://helpx.adobe.com/after-effects/using/composition-profiler.html)).
  A invalidação é **não-destrutiva**: *"frames in the RAM cache are not automatically erased and
  are reused if you undo the change"* — o cache é keyed por estado, não limpo por edição.
- **Rive não cacheia nada** de container aninhado. O que existe é *virtualização*: com
  `ScrollConstraint`, só artboards visíveis têm instância viva.
- **Flash cacheia por bitmap, e o preço é memória, medido**: um movie clip 250×250 px em cache usa
  ~**250 KB** contra ~**1 KB** sem cache
  ([Adobe](https://helpx.adobe.com/animate/using/best-practices-optimizing-fla-files.html)).
  É o único trade-off cache↔memória com número real que a wave achou.
- **Uma avaliação, N instâncias** — o desconto que o Rive *não* dá: o *Master Pose Component* do
  Unreal roda a lógica uma vez e as demais copiam os transforms finais, *"paying the performance
  cost of one animation evaluation while getting dozens of characters animating perfectly in sync"*
  ([Epic](https://dev.epicgames.com/documentation/unreal-engine/animation-optimization-in-unreal-engine)).

**Não encontrado:** qualquer benchmark publicado de custo por nível de aninhamento, em qualquer produto.

---

## §5 — O que a pesquisa NÃO respondeu

- Nenhum produto rotula a régua ("esta régua mede o relógio X"). O AE dá **duas réguas** e uma
  preferência de sincronia; ninguém nomeia o relógio em texto.
- Regras de sync pai↔símbolo no Harmony: a doc não descreve.
- Custo de nesting no Rive e no Cavalry: sem corpus de reclamação, sem benchmark.
- Mensagem exata de recusa de ciclo no AE e no Animate: não existe (a recusa é silenciosa).

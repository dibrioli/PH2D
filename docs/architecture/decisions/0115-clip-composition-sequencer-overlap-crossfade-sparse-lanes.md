# ADR-0115 — Composição de clips: faixas de instâncias, crossfade por sobreposição, canais esparsos

- **Status:** ACEITO (ratificado pelo Enio, 2026-07-12) — em implementação
- **Data:** 2026-07-12
- **Linha:** `line/anim` · sucede a ETAPA 3 (seletor de clip)
- **Plano de implementação:** [`docs/Timeline/02_plano_composicao_clips.md`](../../Timeline/02_plano_composicao_clips.md)
- **Substitui:** a 1ª versão deste ADR ("NLA: strips do Blender, dois modos exclusivos"), **descartada
  pela pesquisa** — ver §1.

---

## §0 — O pedido

Clips hoje são **alternativos** (edita-se um por vez; o dropdown troca qual dirige a cena). O Enio pediu
**empilhar clips**. Antes de portar o NLA do Blender, ele mandou pesquisar o padrão-ouro: *"Blender nem
sempre é o melhor"*. Ele estava certo, e a pesquisa inverteu o desenho.

---

## §1 — A pesquisa (5 frentes) e o que ela matou

### 1.1 O Blender está abandonando o próprio NLA — a estrutura, não a matemática

O módulo Animation & Rigging concluiu, no workshop de 2023, *"we should move the NLA functionality into
the Animation"*. O projeto (**Baklava** / Layered Actions) tem 3 fases: **Slotted Actions** shipou no
**4.4**; **Layered Animation** está em WIP agora (blending correto ainda não mergeado, sem UI);
**strips dentro da Action** é **2027+**. Não há aviso de remoção — mas há um veredito, deles:

> *"Technically possible, but **not a pleasure to work with**."* — e a nota de release que classifica o
> próprio NLA como *"now **fairly** usable"*.

O que eles escolheram **com folha em branco**, ponto a ponto contra o que eu ia portar:

| NLA atual (= o que o ADR-0115 v1 propunha) | Layered Actions (folha em branco) |
|---|---|
| blend + influence **no strip** | blend + influence **na CAMADA** — *"having this setting per layer makes it simpler to manage"* |
| 5 modos (Replace/Combine/Add/Sub/Mult) | **2 modos** (Replace + Combine) |
| **tweak mode** (entrar "dentro" do strip) | dado *sempre* num strip → **sem penhasco de modo** |
| — | camadas **esparsas** (prop não animada passa incólume) |
| — | **mute ≠ influence 0** (mute remove; influence 0 mistura pro default) |

E o item que eu tinha subestimado: **inverse blending**. Keyar uma pose com uma camada não-Replace ativa
exige *inverter a pilha* pra achar o valor que a curva precisa guardar. Os dois sistemas do Blender têm
isso (`BKE_animsys_nla_remap_keyframe_values()` no velho; *"crazy space"* no novo) e os dois têm casos
**insolúveis**. A resposta nova deles: **recusar a key e emitir erro.**

**Fontes:** [code.blender.org/2023/07/animation-workshop-june-2023](https://code.blender.org/2023/07/animation-workshop-june-2023/) ·
[code.blender.org/2025/01/layered-animation-workshop-2024](https://code.blender.org/2025/01/layered-animation-workshop-2024/) ·
[developer.blender.org/docs/features/animation/nla](https://developer.blender.org/docs/features/animation/nla/) ·
[issue #154504 (tracking)](https://projects.blender.org/blender/blender/issues/154504)

### 1.2 Nenhum runtime da indústria usa o strip-stack

Rive, Unity (Mecanim/Playables), Godot (AnimationTree), Unreal (AnimGraph) e Spine: **nenhum** compõe por
pilha de strips. São **grafos de mixers ponderados** ou **pilhas ordenadas de trilhas**. O strip-stack
sobrevive **só como ferramenta de autoria**. Motivo estrutural: o tempo de início de um strip não
significa nada quando o gatilho é desconhecido.

### 1.3 Os sequenciadores convergiram — no gesto que falta ao Blender

Quatro linhagens independentes chegaram na mesma coisa: **sobrepôs, cruzou.**

- Unity Timeline: *"Mix mode creates blends between intersecting clips and **is the default**."*
- Unreal Sequencer: *"**Intersecting** two animation sections **creates an automatic blend curve** between them."*
- Maya Time Editor: *"**The more they overlap, the longer the crossfade.**"*
- MotionBuilder: *"When two clips **overlap**, the blend is called a **cross-blend**."*

**O que falta ao Blender não é expressividade — é o gesto.** Strips numa mesma faixa **não podem se
sobrepor**: você empilha em faixas separadas e *digita* `blend_in`/`blend_out` nas duas, e faz os dois
números concordarem na mão.

Duas lições finas dessa família, que valem código:
- **Ease e blend são a MESMA curva.** A Unity prova renomeando o campo ("Ease In Duration" vira "Blend In
  Duration") e tornando-o read-only quando a sobreposição define a duração. *Ease = blend contra o nada.*
- **Normalize, ou blends parciais afundam.** A Unity mantém um `AnimationOutputWeightProcessor` cujo
  comentário de classe é o rótulo de perigo: *"normalize the mixer weights **so that blending does not
  bring default poses**"*.

**Fontes:** [Unity — Blending clips](https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/clip-blend.html) ·
[Unreal — Cinematic Animation Track](https://dev.epicgames.com/documentation/en-us/unreal-engine/cinematic-animation-track-in-unreal-engine) ·
[Maya — Create transitions](https://help.autodesk.com/cloudhelp/2023/ENU/Maya-Animation/files/GUID-8AAE8C17-C5C7-4EC0-9748-10B2105501FC.htm)

### 1.4 No 2D, "empilhar e blendar" não é o idioma

- **Adobe Animate, Toon Boom Harmony, After Effects: ZERO blend de animação.** "Blend" neles é
  compositing de **pixel**. Uma propriedade tem exatamente um fluxo de keys.
- **Moho** — cujas *Actions* são o parente 2D mais próximo do NLA — permite **sobrepor** actions e resolve
  por **canais disjuntos, último-escritor-vence**. Sem peso, sem crossfade, sem pilha.
- **Cavalry** — única ferramenta 2D de motion graphics com blend real — escolheu **Keyframe Layers**
  (camadas *por-atributo*, com **Strength** e modo Normal/Overwrite). **Sem clips, sem sobreposição.**

O idioma 2D de reuso é: **container aninhado com relógio próprio** (símbolo do Flash, precomp do AE,
artboard aninhado do Rive) + **hierarquia** (parenting) para somar movimento + **posse disjunta de canal**.
Blend de verdade no 2D existe só em **rigs** (Spine, DragonBones) e no **Rive**, e sempre em canais
contínuos.

**Consequência aceita:** este ADR entrega o **empilhamento** que o Enio pediu, na forma do sequenciador
(§2). Mas o **nesting** — a lacuna 2D real, e nós temos **zero** — fica nomeado em §5 como o próximo ADR,
não varrido pra debaixo do tapete.

### 1.5 A pergunta que decide tudo: contra o que um clip parcial mistura?

Nossos clips são **esparsos** (keyam algumas props), como Rive e Spine — **não** poses densas de esqueleto
como Unity/Unreal. E os dois sistemas 2D esparsos escolheram a resposta barata — *lerp contra o valor
atual* — e **os dois tiveram que remendar**:

- **Rive** documenta o próprio blend como *"not linear, but is additive and **could give you unexpected
  results**"* e lançou o **Capture Base State** (snapshot das props do blend) pra tornar o Blend 1D linear.
- **Spine** precisou de `holdPrevious` **mais cinco** modos internos (`SUBSEQUENT`/`FIRST`/
  `HOLD_SUBSEQUENT`/`HOLD_FIRST`/`HOLD_MIX`) só pra derrotar o **"dipping"**.

A resposta cara-e-correta é a do **Godot** (`deterministic` + animação `RESET`): cada contribuidor é um
**delta ponderado contra uma base fixa**, acumulados sem normalizar — a mesma matemática do COMBINE do
Blender, e a que o Blender novo carrega adiante sem mudar.

**Mas a base do Godot/Blender é a rest pose de um osso — e nós não temos rest pose.** `TranslationX` é
**posição absoluta local**: misturar um sprite em x=3.0 contra um "default 0" com influence 0.5 o
**joga pra origem do pai**. É exatamente o "afundar pro default" que a Unity normaliza. §2/R3-R5 resolvem.

**Fontes:** [rive-runtime `keyframe_double.cpp`](https://github.com/rive-app/rive-runtime/blob/main/src/animation/keyframe_double.cpp) ·
[Rive — Capture Base State](https://rive.app/changelog/capture-base-state) ·
[Spine — Applying animations](https://esotericsoftware.com/spine-applying-animations) ·
[Godot — AnimationMixer.deterministic](https://docs.godotengine.org/en/stable/classes/class_animationmixer.html)

---

## §2 — Decisão

Composição **sequencial E simultânea** no gesto do sequenciador, com **canais esparsos**, **sem rest-pose
implícita** e **sem modo**.

```
TimelineDoc.stack: Vec<ClipLane>            // VAZIO = comportamento de hoje, byte-idêntico

ClipLane  { name, muted, weight: f32, mode: Override | Additive, strips: Vec<ClipStrip> }
ClipStrip { clip: u16,                      // índice em doc.clips()
            t_start, t_end,                 // onde toca na timeline
            src_in, src_out,                // que fatia do clip é usada
            speed: f64,
            loop_mode: Once | Loop | PingPong,
            ease_in, ease_out }             // = blend_in/out quando há vizinho (R1)
```

**`StripLoop` tem TRÊS variants, não quatro** (corrigido na implementação, A1): `Hold` seria idêntico a
`Once` (as duas seguram o último valor quando a fonte acaba antes do span), e o `Nothing` do Blender — o
strip **para de contribuir** enquanto ainda cobre o tempo — é justamente o buraco por onde a pilha cai pro
default e puxa o sprite. Um strip que cobre tempo que não consegue preencher é um strip mal-aparado, não um
recurso: **apare o strip**. Variant morto = o que os nossos gates existem pra impedir.

Faixas empilham **de baixo para cima**. O join key é **de graça**: bindings são *document-wide*, então o
mesmo `AnimTarget` significa o mesmo `(entity, prop)` em **todo** clip ([doc.rs:140](../../../crates/ph2d-timeline/src/doc.rs)).

### As regras (cada uma com um porquê que a pesquisa pagou)

**R1 — Sobreposição na MESMA faixa = crossfade automático.** A duração *é* a sobreposição. Curva default =
ease S (Unity), **complementar** (soma 1) → **nenhuma base é necessária**. `ease_in`/`ease_out` e
`blend_in`/`blend_out` são **o mesmo campo**: vira "blend" e fica read-only quando um vizinho define a
duração. *(Unity/Unreal/Maya/MotionBuilder.)*

**R2 — Esparsidade = máscara de graça.** Uma faixa só toca os canais que o clip dela keya. Prop não coberta
por faixa nenhuma **não é escrita** — a cena manda. Zero Avatar Mask, zero filtro. *(Rive, Spine, Moho: "a
action do braço só keya o braço".)*

**R3 — Normalize DENTRO da faixa.** `v_lane = Σ(wᵢ·vᵢ) / Σwᵢ` (quando `Σw > 0`). É isto que mata o "afundar
pro default". Cobertura da faixa = `clamp(Σw, 0, 1)` — **o que ela diz** e **quanto ela afirma** são duas
grandezas separadas. *(Unity `AnimationOutputWeightProcessor`.)*

**R4 — Entre faixas, de baixo pra cima:**
- `Override`: `acc = lerp(acc, v_lane, coverage · weight)`
- `Additive`: `acc = acc ⊕ (v_lane ⊖ base_clip) · coverage · weight`

onde `base_clip` = o valor do canal **no primeiro frame do clip** — additive é **delta relativo ao próprio
primeiro frame** *(regra Maya: "evaluates the clip relative to its first frame"; Unity: reference pose =
frame 0)*. **Nenhuma rest-pose de cena entra aqui.**

**R5 — `rest` por binding, CAPTURADO.** `TargetBinding.rest: f32` = o valor que a prop tinha quando você a
animou pela 1ª vez. É contra ele que um fade-in "do nada" mistura (faixa de baixo com `coverage < 1`).
É o **Capture Base State** do Rive e a **Base Pose** do Unreal — os dois precisaram; não vamos fingir que
não. Sem ele, *"o objeto fica onde eu o pus e a animação entra por cima"* é impossível de expressar.

**R6 — O relógio compõe PRA DENTRO, nunca ao lado.** `t_src = clip.remap(entity, strip.map(t))` — o strip
mapeia tempo-de-timeline → tempo-de-clip; o `TimeRemap` **daquele clip** mapeia tempo-de-clip → tempo-fonte
da entidade. É o modelo precomp do AE, que a timeline já segue. `remapped_time` passa a receber o `&Clip`
em vez de ler `doc.active_clip()` ([apply.rs:98](../../../crates/ph2d-timeline/src/apply.rs)) — hoje, sob
uma pilha, *"qual clip dá o relógio da entidade?"* é **indefinido**, e essa classe de bug já nos custou
três rodadas ([[feedback_derived_coordinate_seed_must_match_sample]]). Sem pilha, `strip.map` = identidade
→ byte-idêntico.

**R7 — INVARIANTE: todo `PropKind` é um escalar blendável.** Um prop discreto (frame de sprite-sheet,
visibilidade, z-order, desenho do Flip) **não entra na pilha** sem um modo Replace-only. Hoje isso é
verdade (os 7 `PropKind` são `f32`; o Flip é doc separado), mas `AnimValue::lerp` "blendaria" um `Bool` com
step em `t<0.5` e um par de tipos diferentes devolvendo `b` — os dois **errados e silenciosos**. **Gate
executável**, não comentário.

**R8 — Sem modo, sem tweak mode.** O dope-sheet mostra o **clip ativo** (o dropdown que a ETAPA 3 já
entregou); a cena mostra a **pilha**. O tweak mode do Blender existe *só* porque os editores dele ficam
presos a uma Action por vez — **nós já não temos esse problema.** A "decisão dos dois modos exclusivos" da
v1 deste ADR era dívida escondida disfarçada de simplificação.

> **R8 — emenda (2026-07-16): duas ABAS, porque uma régua mede um relógio.**
>
> A rejeição do *tweak mode* **fica de pé, e pelo motivo original** — ela é sobre um MODO, e uma aba não é
> um modo: ela muda *o que se vê e o que a régua mede*, nunca o que uma edição significa, e o **dropdown de
> clip segue sendo quem escolhe o clip**. Nada de `Tab` pra entrar e sair.
>
> O que o R8 errou foi um corolário, e ele é estrutural: concluiu que, sem tweak mode, as duas metades
> podiam **coabitar uma vista**. Elas não podem, porque coabitam **uma régua com dois significados** — uma
> key é carimbada no tempo do **CLIP**, um strip senta no tempo da **TIMELINE**. Escolha `Right` no dropdown
> e as keys dele desenham em 0..3 enquanto o strip dele toca em 2..5: a mesma coluna de pixels, dois
> instantes, e nada na tela admitindo isso. **Sem pilha os dois relógios são um só** — que é exatamente por
> que passou despercebido: a feature que os separa era a feature que ninguém tinha usado ainda.
>
> O Enio leu isso da tela antes de qualquer pesquisa (*"a timeline com keys e com strips misturadas é
> confusa"*), e o padrão-ouro concorda — e **separa mais do que nós**: o Unity nem deixa editar keyframe na
> janela do Timeline (manda pra Animation window); o NLA Editor e o Dope Sheet do Blender são **editores
> diferentes**; o Premiere põe as keys no Effect Controls. O AE mistura num nível só, mas dá **aba** a cada
> nível de nesting. O único que mistura à vontade — o Sequencer do Unreal, o mais parecido conosco — tem
> usuário pedindo socorro (*"scrolling through endless expanded tracks of keyframed data can be
> fatiguing"*).
>
> **Desenho:** abas **Keys** (dope sheet + graph do clip ativo, régua no relógio do CLIP) e **Arrange**
> (lanes + strips, régua no relógio da TIMELINE), em `ph2d_panel_timeline::tab`. O relógio do clip é
> publicado no snapshot (`clip_time`) por `ph2d_timeline::clip_playhead`, que sai da **mesma porta** que o
> K (`sole_strip_of`): se o clip não toca aqui, ou toca **duas vezes**, não há playhead — porque não há
> onde apontar, e é a mesma razão pela qual o K recusa (R9). Sob pilha a régua do clip é **read-only** (sem
> scrub, sem braces de loop, sem markers): o inverso não existe — um strip em loop manda muitos instantes
> da timeline no mesmo instante do clip.
>
> **Sem pilha nada muda**: o clip É a timeline, então a aba Keys é o painel que sempre foi.

**R10 — A LACUNA não é silêncio: é o strip anterior SEGURANDO** (2026-07-16, Enio).
*"O fade das strips quando não há sobreposição ainda provoca saltos — a sprite não faz a transição a
partir de onde está mas pula para mais perto da posição inicial da outra strip."* **Medido:** `Left[0,3)`,
lacuna, `Right[4,7)` com fade-in — em `t=3.9` a sprite está em `-3`, em `t=4.0` em `0.000`. **3 unidades
num frame.**

A causa não era o fade: era **duas respostas discordando através de um pixel de régua.** Onde nenhum strip
cobria, a lane era *silenciosa* (ninguém escrevia, o objeto segurava a pose); no primeiro instante do
fade-in — strip cobrindo com peso **zero** — ela respondia **repouso**. O fade rampeava a partir do
repouso, e a sprite não estava no repouso.

**Não se conserta silenciando o peso zero** — isso é o que quebrava a path-independence (a pose passava a
depender do lado de onde o playhead chegava; o gate `seam_determinism` existe por isso e **fica**). A
lacuna é que nunca foi silêncio: o strip que acabou **segue afirmando o último frame dele**
(`ClipLane::hold_at`, peso = complemento do que está vivo), e o fade-in **cruza a partir dali**. É a
extrapolação `Hold` do Blender / clip extrapolation do Unity — e faz o fade solitário se comportar como a
sobreposição que o Enio já aprovou.

Consequências que valem saber: **a sobreposição é intocada** (dois strips somam exatamente 1 no overlap →
nada é segurado) · o hold é **forward-only** (nada atrás do primeiro strip pra segurar, e entrar de fade a
partir do repouso no topo da timeline é coisa legítima de querer) · um strip **held NÃO está tocando**
(`sole_strip_of` o pula, senão o **K** keyaria num strip que já acabou) · depois do último strip a lane
segura a última pose para sempre — o que já era o efeito visível, agora por afirmação e não por inércia.
Trajetórias em `tests/lone_fade.rs`; o número que este R10 mudou está anotado no
`seam_determinism::a_zero_weight_fade_edge_reports_the_held_pose_not_a_stale_one`.

**Aberto:** hold **backward** e por-strip (`Hold`/`Hold Forward`/`Nothing` são um enum no Blender) — hoje é
uma política só, e nenhuma cena pediu a escolha ainda.

**R9 — Autokey sob pilha: inverta, ou RECUSE.** Pra gravar a pose vista, inverte-se as faixas acima do clip
ativo — `Override` com peso `w`: `v = (alvo − w·acima) / (1 − w)`; `Additive`: `v = alvo − delta`. Com
`w → 1` não é inversível → **recusa a key + toast**. Nunca mover o objeto em silêncio. *(É a resposta nova
do Blender: "Blender will simply reject keying and issue an error".)*

### Escala não soma — multiplica

Somar dois clips de escala 1.0 dá **2.0**: foi esse bug ([T47035](https://developer.blender.org/T47035))
que fez o Blender inventar o COMBINE. Aqui, `⊕`/`⊖` são **escolhidos pelo `PropKind`** (`blend_op()`, uma
função, não um `if` espalhado):

| canal | base neutra | Additive |
|---|---|---|
| `TranslationX/Y`, `Rotation` | 0 | soma: `acc + Δ·w` |
| `ScaleX/Y` | 1 | razão: `acc · (v/base)^w` |
| `Opacity` | 1 | razão (e clamp [0,1]) |
| `TimeRemap` | — | **não empilha** (é relógio, R6) |

**Rotação sai de graça:** as voltas acumuladas (§6.1, já landou) tornam `Rotation` um escalar sem
ambiguidade de ±2π — sem isso, cruzar 350°→10° blendaria pelo caminho longo.

---

## §3 — Conjunto de aceitação (concreto, congelado — DIRETIVA §5)

**PRONTO** quando, e só quando:

1. Documento com `stack` vazia é **byte-idêntico** ao de hoje (gate executável). ✅ **(A6)**
2. Dois strips sobrepostos na mesma faixa: crossfade **contínuo, sem salto e sem afundar** — incluindo o
   caso *"o clip A keya X, o clip B não keya X"* → X segue A (R2), **não** cai pro default. ✅ **(A5)**
3. `src_in`/`src_out` recorta; `speed` retima; `Once`/`Loop`/`PingPong` dobram a fonte. Cada um provado
   no **valor amostrado**, não na existência do campo. ✅ **(A1-A3, `tests/clip_stack.rs`)**
4. Faixa `Additive` soma **DELTA**: um clip de pose **constante** contribui **ZERO** (o teste que pega
   "somei o valor absoluto"). ✅ **(A4/A5)**
5. Escala additive **multiplica**: dois clips de escala 1.0 dão **1.0**, não 2.0. ✅ **(A4)**
6. Faixa **mutada** não contribui. A **ordem** de empilhamento importa, e o teste prova qual. ✅ **(A5)**
7. **Autokey sob pilha**: com um `Override` de `w=1` acima, a key é **recusada**; com `w<1`, a key gravada
   **reproduz a pose vista** (round-trip pelo apply real). ✅ **(A8)** — e mais duas que a implementação
   revelou: pose **em cima do blend** não keya NADA (o diff lê o que o apply escreveu, não a curva do clip
   ativo — senão minta uma key por frame com o objeto parado), e clip tocando **duas vezes** ao mesmo tempo
   = "aqui" não tem resposta única → recusa (o mesmo muro que o Blender documenta). **Falta a superfície
   visual da recusa** (toast) — vai na fatia B.
8. **Gate R7**: nenhum `PropKind` não-blendável alcança a pilha. ✅ **(A9)** — executável: todo `PropKind`
   no meio de um crossfade tem de cair na **média exata** dos dois clips (ou seja, interpolou). Um canal
   discreto (frame de sprite-sheet, visibilidade, desenho do Flip) faria esse teste falhar, que é o ponto:
   é o arame de tropeço.
9. **Seam de UI**: criar strip, arrastar, redimensionar, **sobrepor** (o crossfade aparece), mudar modo/peso
   da faixa — cada um dirigido por `WidgetEvent` real, cada um **um** undo step.
10. Smoke do Enio.

## §4 — Kill-criterion (o baseline é MEDIDO, não prometido)

O apply roda **todo frame** e é **zero-alloc gateado** (HR-3, `tests/no_alloc_bridge.rs`).

### 4.1 — A0: o hoist (FEITO, 2026-07-12)

O baseline era **duas** quadráticas, não uma — e a segunda só apareceu porque o probe **reprovou o
primeiro conserto**:

1. `remapped_time` re-varria a lista de bindings *por binding* → o clock de uma entidade com 6 props era
   resolvido 6 vezes. Hoistado em `clock.rs` (uma resolução por **entidade**).
2. **A dominante, que eu não tinha visto:** `Clip::track()` era uma **varredura linear** sobre
   `Vec<(AnimTarget, Track)>` — e como cada binding cria uma track, T ≈ B, então o *lookup* sozinho era
   O(B²), rodando para **todo** binding (com ou sem Time Remap). `Clip` agora mantém as tracks **ordenadas
   por target** e busca em binário (invariante restaurada também na desserialização — um save antigo pode
   ter as tracks fora de ordem, e uma busca binária sobre ele erraria em silêncio, matando a animação).

**Medido** (release, pior caso — *toda* entidade com Time Remap; `tests/apply_perf.rs`, `--ignored`):

| bindings | µs/apply | **µs/binding** |
|---:|---:|---:|
| 175 | 4,4 | **0,025** |
| 1 400 | 51,8 | **0,037** |
| 5 600 | 299,9 | **0,054** |

**32× os dados, 2,2× o custo POR BINDING** — linearítmico (as buscas binárias + cache). Sob a lei
quadrática essa última coluna cresceria *proporcional a B*, ou seja **32×**. 5 600 bindings custam **1,8%**
de um frame a 60 Hz.

> **A lição de método:** a razão do tempo TOTAL não distingue linear (4×) de `B log B` (~4,8×) — o ruído da
> máquina cobre essa distância. O **custo por binding** distingue: é plano no linear, sobe de leve no
> `B log B`, e cresce *proporcional a B* no quadrático. O gate assere essa coluna.

### 4.2 — O kill (para a pilha)

Se a avaliação da pilha não for **zero-alloc**, ou custar **> 2× este baseline** num doc de 50 bindings ×
4 faixas, **a feature não existe nesta forma** — o caminho passa a ser **pré-composição** (assar a pilha num
clip cacheado quando ela muda), e isso é um **ADR novo**, não um remendo.

---

## §5 — O que este ADR NÃO é (e o que vem depois)

- **Nesting** — container animado reusável com relógio próprio (símbolo/precomp/artboard aninhado). É o
  idioma 2D de reuso, **nós temos zero**, e é a lacuna real que a pesquisa expôs. O `TimeRemap` já é meio
  caminho (o relógio por-entidade, modelo precomp do AE) e a Hierarchy já dá o parenting (a soma de
  movimento que o 2D usa de verdade). **Próximo ADR.**
- **Blend por parâmetro / state machine** (Rive Blend 1D, Smart Bone do Moho). A pesquisa deixou um insight
  que vale ouro pro nosso norte node-centric: *blend-paramétrico e frame-pick são a MESMA UX* — "um número
  escolhe a pose"; um interpola porque o canal é contínuo, o outro salta porque é discreto. Casa com os
  Motion Nodes (`motion.mixer`, `value.switch` já existem) **e** com o Flip. Follow-up nomeado.
- **Combine com rest-pose de esqueleto** — não temos rig (deferido pro fim de tudo, ADR-0108).

## §6 — Consequências

- `DOC_VERSION` **3 → 4** (`TimelineDoc.stack` + `TargetBinding.rest` apendados; postcard posicional).
- `remapped_time` muda de assinatura (recebe `&Clip`) — R6.
- `apply_from_doc` ganha **um** ramo; o caminho de clip único continua o mesmo código.
- **Nenhum contrato congelado é tocado** (CLAUDE.md §6 intacto).

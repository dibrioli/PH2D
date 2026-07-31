# ADR-0152 — Timeline expressions are a first-class LANE SOURCE that fades (they compose inside the blend, not after it)

Status: **Accepted e CONSTRUÍDO** (`line/anim`, W0–W7 + o follow-up das vistas, todas gateadas e mutação-provadas; aguardando smoke do Enio e ordem de integração). Plano: [`docs/Timeline/08_plano_expressoes_no_blend.md`](../../Timeline/08_plano_expressoes_no_blend.md).
Supersedes the *isolation* clause of [ADR-0144](0144-timeline-expressions-frozen-ir-separate-post-composition-pass.md); completes [ADR-0151](0151-timeline-expressions-are-per-clip-so-a-strip-windows-them.md).
Date: 2026-07-27 · Line: `line/anim`

> ⚠️ O número **0146** é PROVISÓRIO — se CONTA na integração (o main de hoje para em 0144; o 0145 desta linha ainda não integrou). Se outra linha reivindicar o mesmo número, quem chegar ao main primeiro fica com ele.

---

## Resumo em uma frase (para o Enio)

Hoje uma expressão é aplicada **depois** que o fade já compôs tudo, e por isso ela **sobrescreve** o resultado — não sabe fadear, cruzar (crossfade), somar (aditivo) nem entrar num container. A decisão é: **a expressão passa a ser o valor que a strip entrega ao blend** (o mesmo lugar onde uma track keyada entrega o dela), então **fade / crossfade / aditivo / container passam a valer para ela automaticamente, de graça, pela mesma máquina** — e o **fade extraordinário fica byte-idêntico** onde não há expressão, porque nenhum código novo roda nesse caminho.

**O que muda para o artista:**
- Uma expressão numa strip agora **fadeia com a strip** (antes ligava/desligava seco), **cruza** com a strip vizinha, e **soma** numa lane aditiva — como uma animação keyada.
- Uma expressão que segue outro objeto (`value + Sprite.x`) agora **acompanha o objeto mesmo enquanto ELE fadeia**, e a própria strip do seguidor fadeia o resultado — **duplo fade**.
- `value` numa expressão passa a significar *"o valor keyado desta strip"* (antes era o valor já composto). É o modelo do After Effects (o `value` por-camada), e é mais correto — **mas é uma mudança de comportamento que TEM de ser vista num smoke, não afirmada.**

---

## 1. Contexto e a força que obriga a decidir

O ADR-0144 pôs a avaliação de expressões num **passe pós-composição separado** (`expr_pass::run`, chamado ao fim do apply), que lê o valor já composto e **sobrescreve** a propriedade (`write_prop`, última-escrita-vence). Essa escolha foi deliberada: manter o `eval_frame`/blend intocado, para o **fade fingerprint** (`crates/ph2d-timeline/tests/fade_fingerprint.rs`, hash cravado `0x69dca8811eb0f8f8`) ficar trivialmente seguro.

O preço dessa isolação é **exatamente** a incompatibilidade que o Enio quer eliminar. Sobrescrever fora do blend significa que uma expressão **nunca**:
- cruza (crossfade) com outra strip — dá última-escrita-vence, não média ponderada;
- some numa lane aditiva — sobrescreve o valor composto em vez de contribuir um delta;
- respeita cobertura/fade — a amplitude não encolhe conforme a strip some;
- toca no self-crossfade — a mesma clip em dois strips sobrepostos fica **quieta** (`sole_strip_of` → `PlaysTwice`).

A força: **queremos o padrão-ouro — expressões plenamente integradas ao sistema clips/strips/container/arrange/fade, sem perder um grama da fidelidade do fade.** Custo não é restrição.

**O estado da arte concorda com a direção e refuta a forma atual.** Houdini (CHOPs: tudo é um canal/sinal que compõe no grafo) e Unreal (o valor computado é produzido **a montante**, entra no blend como qualquer outro) tratam um sinal computado como **fonte**, nunca como um patch pós-blend. Os dois **recusam** precisamente a forma "sobrescreve depois" que o PH2D ship hoje. After Effects e Cavalry documentam a mesma parede (não se crossfadeia uma expressão pós-composição). A única forma estrutural de fazer uma expressão participar de fade/overlap/aditivo/nesting é **movê-la de *depois* do blend para *na fonte***.

---

## 2. A decisão (uma frase que sobrevive fora de contexto)

**Uma expressão de propriedade passa a ser o valor que um clip contribui no único ponto de amostragem do blend (`stack_eval::eval_frame`, o sítio `:155-165`), gateado por presença** — um `AnimSource { Track, Expr }` na fonte da lane, um apply em **duas fases** (canais keyados verbatim, depois canais de expressão em ordem topológica de dependência), com **um-frame-de-atraso determinístico** para ciclos entre objetos — **e o fade permanece byte-idêntico onde não há expressão, porque nenhum código novo roda nesse caminho.**

### 2.1 O mecanismo, em prosa

- **Per-clip = fonte de lane que FADEIA.** A fórmula vive no clip (`NamedClip.expr`, ADR-0151). No sítio de amostra, para cada strip cujo clip carrega a expressão daquele canal, a **contribuição da strip é o resultado da expressão** (`E(t_src)`), avaliado no tempo local da strip, com `value` = a amostra keyada **daquela strip** (ou o repouso). A partir daí o valor flui pela mesma normalização de lane (`num/den`), pelo mesmo crossfade complementar e pelo mesmo aditivo — **fade/overlap/aditivo/container são herdados, não reimplementados.**
- **Global (`binding.expr`) = transformação do canal inteiro, que NÃO fadeia.** Um driver global não tem strip para fadeá-lo; ele é aplicado como uma transformação final sobre o valor composto do canal (`composed = eval_expr(global_ir, value = composed, …)`). É a separação limpa que o ADR-0151 já insinuava: *per-clip fadeia (é fonte de lane); global não fadeia (é fórmula do canal).*
- **Prop-links (`Sprite.x`) fadeiam DUAS vezes.** O objeto-fonte compõe **antes** (ordem topológica) e grava seu **valor já fadeado** no mapa do frame; o leitor lê esse valor fadeado, e a **própria strip do leitor** fadeia o resultado inteiro. Impossível no passe pós-composição.
- **Ciclos entre objetos (A↔B) = um-frame-de-atraso.** A maioria acíclica resolve topologicamente, exata no frame (sem lag). Ciclos genuínos leem a aresta-de-volta semeada do mundo (o precedente `snap` do `expr_pass`) — `N_CYCLE = 1` **é** o um-frame-de-atraso da indústria (Houdini Feedback CHOP, Unreal off-by-one), determinístico e reproduzível.

---

## 3. Alternativas REJEITADAS (com o motivo medido, não "achamos pior")

1. **Manter o passe pós-composição que sobrescreve (ADR-0144, o de hoje).** Rejeitado: é a causa da incompatibilidade. Estruturalmente não fadeia (sobrescreve o composto), e o SOTA (Houdini/Unreal) recusa exatamente essa forma.

2. **Uma camada pós-composição que LÊ os pesos do blend** (`ActiveStrip.w` / `ClipLane.weight` / `den`) e recompõe a saída da expressão com eles. Rejeitado por **impossibilidade estrutural, não gosto**: os strips keyados de uma lane já são colapsados em `num/den` no `stack_eval.rs:235` **antes** de qualquer camada pós poder ver o denominador — reconstruir um overlap misto keyado+expressão é impossível, e o self-crossfade (mesma clip, dois strips) **não pode** ser recuperado (a matriz de pontuação o marca `0`). Além disso é uma **segunda cópia do blend**, o antipadrão *"duas portas de uma aritmética divergem"* que este módulo já pagou cinco vezes.

3. **Um grafo de nós (DAG) onde keys, strips, fades e expressões são todos nós de sinal.** Rejeitado: re-expressar o blend como nós **reordena as operações de ponto flutuante** e **move o hash cravado** `0x69dca8811eb0f8f8` (IEEE-754 é comutativo mas não associativo, e o Rust nunca contrai FMA — a estabilidade de ulp vem de *as mesmas ops na mesma ordem*); e um nó arbitrário **não carrega a garantia afim-em-`value`** de que o `invert_stack` (keying) depende. Troca a joia da coroa por uniformidade.

4. **Um fixpoint no mesmo frame, iterado até convergir, para ciclos.** Rejeitado por **medição matemática**: um sistema VEX não-linear (`A = B + wiggle`, `B = 0.9·A`) **não tem valor-fixo único garantido**; iterá-lo oscila e viola a reprodutibilidade (HR-5). *"Custo não é restrição"* não pode fabricar um valor estável onde nenhum existe. A superfície honesta é o um-frame-de-atraso (`N_CYCLE = 1`), com um `N_CYCLE > 1` opcional e limitado só para um ciclo **contrativo** futuro (nunca iterado até convergir).

---

## 4. O PREÇO da decisão escolhida (explícito)

Nenhum destes é opcional; todos entram na wave e no plano de gates.

- **`value` vira per-strip, não per-composição.** `value + wiggle` passa a significar *"wiggle sobre a amostra keyada DESTA strip"*, e sob overlap as duas strips oscilam sobre os próprios valores e cruzam. Mais correto (o `value` por-camada do AE; uma strip é nossa camada), **mas é mudança de comportamento** para expressões per-clip existentes. **Tem de ser SMOKADO, não afirmado** — o fingerprint sem-fórmula não a enxerga.
- **Per-clip agora FADEIA e COBRE o canal.** Era liga/desliga; agora fadeia. E uma expressão pura (sem track) passa a `speaks = true` ⇒ **cobre o canal** (mascara lanes de baixo/repouso) onde a esparsidade antes o deixava transparente. Mudança de comportamento real — nota de smoke.
- **Prop-link é INERTE em lane aditiva.** O termo de link cancela na referência (`(v(t)+X) − (v(src_in)+X) = v(t)−v(src_in)`). Consistente com "expressão constante contribui 0", mas é uma limitação a documentar.
- **Assimetria autoria×playback no plays-twice.** O `K` recusa (`PlaysTwice`), mas o playback dirige a expressão **duas vezes** (uma por instância). Deliberado (playback quer o valor; autoria não sabe onde pousar a chave), mas precisa de gate próprio.
- **Keying recusa mais, e melhor explicado.** Canais dirigidos por fórmula não-linear ou independente-de-`value` **recusam honestamente** (`KeyRefusal::ExpressionDriven`, com mensagem *"limpe/reescreva a fórmula"*, nunca o "delete a lane" do `Overridden`). `value + g(time)` — o idioma mais usado do AE — **key e pré-compensa** o offset.
- **Ciclos entre objetos: um-frame-de-atraso; ciclo NÃO-CONTRATIVO diverge ENTRE frames.** Um ciclo com ganho ≥ 1 (`A = B + wiggle`, `B = 1.1·A`) não explode no frame mas **diverge através da semente do mundo**, e um scrub re-baseliza da pose viva ⇒ *o mesmo `t` dá poses diferentes conforme o lado de onde você chega*. Isto viola *"a pose é função do PLAYHEAD"* **só para canais cíclicos**, é padrão da indústria (Houdini/Unreal), e é **re-baselizado na descontinuidade** (scrub/jump/load — o precedente do ring da física / pin do autokey). A divulgação diz *diverge entre frames*, não "história-dependente por um frame".

### 4.1 Correções ADVERSÁRIAS que viraram restrição de PRIMEIRA CLASSE

A fase de verificação (três agentes atacando o design contra o código real) derrubou a moldura ingênua de *"uma única edição no `eval_frame`"*. As quatro correções abaixo são obrigatórias — sem elas o design está **errado**, não meramente incompleto:

- **(C1) SÃO DOIS SÍTIOS DE AMOSTRA, não um — e o segundo é o caso COMUM.** O `eval_frame` compõe o blend a partir de `doc.stack()`, que é **vazio num documento NÃO-EMPILHADO** (uma animação keyada comum, sem strips — o caso mais frequente): `apply.rs:73` amostra `doc.active_clip().track()` **direto**, nunca chama `eval_frame`; e o strip sintético do caso solo (`stack_frames.rs:214-224`) é **ignorado** pelo `eval_frame` (que itera `doc.stack()`, vazio). ⇒ Sem tratamento explícito, **toda expressão num doc sem strips pararia de ser dirigida em silêncio.** A fase não-empilhada TEM de resolver canais de expressão via `eval_expr` sobre a amostra direta da track + o `LinkFrame` (exatamente o que o `ExprWindow::ActiveClip` faz hoje). **Nenhum teste atual cobre isso** (todos fazem `add_lane`/`add_strip_to`) — gate obrigatório.

- **(C2) O mapa composto (`LinkFrame`) TEM de ser PERSISTIDO e LIDO, nunca recomputado.** A cura do bug seed==sample (o autokey não minta chave fantasma) só vale se `shown_value`/`position_shown`/`pose_at` **lerem** o valor exato que o escalonador escreveu, em vez de re-derivar. Para uma expressão local (time/wiggle) a re-derivação pode casar; para um **prop-link entre objetos** a porta single-entity **não tem o grafo** ⇒ `shown ≠ world` todo frame ⇒ **chave fantasma por frame pausado em todo canal com prop-link**. Fix: **stashar o `LinkFrame` composto no doc** (o idioma `put_scratch`/`take_scratch` que já existe em `apply.rs:46/128`); `shown_value`/`position_shown`/`pose_at` **leem** esse mapa para canais dirigidos. O gate do fantasma **tem de exercitar um canal com prop-link entre objetos**, não só uma expressão local — senão certifica nada.

- **(C3) O keying no caminho NÃO-EMPILHADO tem de INVERTER a expressão afim.** `key_value_in_active_clip` (`autokey.rs:304`) faz `return Some(want)` cedo quando não-empilhado. Para uma clip não-empilhada com expressão afim `value + g(time)`, isso guarda `want` **sem pré-compensar** ⇒ a pose pousa em `want + g(t)` (chave confiante-e-errada). O early-return não pode disparar para um canal de expressão; a rota afim guarda `stored = want − g(t_key)`.

- **(C4) O buraco `read_prop` (Position/Morph) é do caminho de prop-link E do repouso.** `read_prop` devolve `None` para Position/Morph, então um prop-link `Name.position`/`Name.morph` resolve a `0.0`. Pior: a fonte que **NÃO é** de expressão (keyada) precisa ser lida na **semente da Fase 1** através da trajetória (espelhar `apply_path::read_rest`, já usado em `apply.rs:169`), não só "estender `read_prop`". ⚠️ E o `read_prop` é chamado por `refresh_liveness_and_rest` **todo frame, independente de haver expressão** — então estendê-lo cru muda o `rest` de um canal Morph (o *fade-in-from-rest*) **num documento com ZERO expressões** ⇒ quebra a byte-identidade sem-fórmula que o fingerprint (só-`TranslationX`) não vê. A extensão **tem de ser inerte no caminho de captura de repouso**, ou o corpus do fingerprint tem de incluir Morph/Opacity/Position sob um fade.

---

## 5. O que fica GATEADO (para ninguém re-litigar por prosa)

Todo gate é mutação-provado (RED-first onde marcado). A ordem é por severidade.

**O fade (a joia — byte-identidade E cobertura do que a wave de fato toca):**
1. `the_fade_surface_is_byte_stable` — `fade_fingerprint == 0x69dca8811eb0f8f8` no corpus sem-fórmula; re-rodar na árvore intocada **antes e depois** da wave, no mesmo commit.
2. `the_track_arm_adds_no_float_op` — mutação: rotear o caminho `Track` pela aritmética do `Expr` ⇒ o hash move. Prova que a generalização é um **braço-par**, não uma modificação.
3. `no_expression_allocates_no_link_frame` — dhat: um apply sem-fórmula não constrói `LinkFrame` nem topo-sort (HR-3).
4. **(RED-first) Um SEGUNDO fingerprint sobre uma região de expressão fadeada** — `time*10` per-clip sobre um crossfade de 1 s; mutação: contornar o braço da expressão (sobrescrita) ⇒ o crossfade colapsa em liga/desliga e o hash move. **Prova que a expressão FADEIA, não só que existe.**
5. **(RED-first, Hole A) Um TERCEIRO fingerprint: um canal keyado+fadeado NÃO-expressão CO-RESIDENTE com uma expressão** — o caso comum do mundo real, que roda o caminho `scheduled==true` de duas fases. Sem ele, uma mutação que perturbe a composição keyada só sob `scheduled` passa nos gates 1 e 4.
6. **(Hole B / C4) O corpus do fingerprint inclui Morph/Opacity/Position sob um fade** — OU um gate prova que a extensão do `read_prop` é **inerte** no caminho de captura de repouso.

**A participação no blend (RED-first):**
7. `an_additive_expression_contributes_a_delta` — constante ⇒ 0 (Sum)/1 (Ratio); em movimento ⇒ `E(t)−E(src_in)`. Mutação `base=0` ⇒ RED.
8. `a_prop_link_reads_the_faded_source` — `value + Sprite.x` com `Sprite.x` no meio de um crossfade; o leitor acompanha a fonte fadeada no mesmo instante. Mutação: compor o leitor antes da fonte (quebra topo) ⇒ 1-frame-stale RED.
9. `an_expression_self_crossfades` — mesma clip em dois strips sobrepostos ⇒ `lerp` das duas contribuições, não liga/desliga; mutação: manter a recusa `PlaysTwice` ⇒ RED.
10. `lead_out_with_expr_fades_out` e `plays_twice_with_expr_drives_each_instance` — os dois flips de semântica que hoje ficam quietos; ambos RED-first (o fingerprint sem-expressão não os vê).

**Keying e o fantasma (a cura do seed==sample):**
11. Trio de keying — `value_plus_g_of_time_keys_and_pre_compensates` (e o **caminho não-empilhado** pré-compensa, C3); `a_pure_formula_refuses_ExpressionDriven` (não `Overridden`); `a_value_nonlinear_formula_refuses`. Mutação: pular o terceiro probe ⇒ o caso não-linear minta chave errada, RED.
12. `auto_key_mints_no_phantom_key_on_a_PROP_LINKED_channel` — o `LinkFrame` é persistido e lido; `shown_value == frame.links == {apply; read}`. **Tem de usar um canal com prop-link entre objetos** (C2), não uma expressão local. Mutação: recomputar `shown_value` com uma expressão com `wiggle`/prop-link ⇒ fantasma por frame, RED.
13. `a_skipped_entity_is_left_alone_but_readable_by_a_prop_link` — entidade `skip`/`displaced` não é dirigida mas fica no `LinkFrame`; mutação: ignorar `skip` ⇒ deriva monotônica ao pausar, RED.

**Ordem, ciclos, determinismo, e o caso não-empilhado:**
14. `the_scene_evaluates_in_dependency_order` — acíclico lê fresco; ciclo leva `N_CYCLE=1` (um-frame-de-atraso), estável e reproduzível; re-scrub ao mesmo `t` concorda para acíclico.
15. `the_cross_os_hash_of_wiggle_plus_prop_link` — hash estilo `physics_ecs_c9` na matriz 3-OS. ⚠️ **Só `wiggle` (hash inteiro), NUNCA `sin`/`cos`** — os transcendentais do `std` (`eval.rs:42-43`) **não são** cross-OS; incluí-los diverge o hash entre OSes.
16. **(RED-first, C1) `an_expression_drives_a_non_stacked_document`** — o caso comum sem strips; mutação: rotear tudo pelo `eval_frame` ⇒ a expressão fica sem-dirigir, RED.
17. **Aposentar `the_expression_pass_never_enters_the_blend`** (`tests/expressions.rs`) — decisão **load-bearing**, registrada aqui: a isolação do ADR-0144 é **trocada** pela participação no fade *porque os gates 1+2 provam o que a isolação afirmava*. Não é delete silencioso.

---

## 6. Notas de projeto que a wave herda

- **Sem bump de `DOC_VERSION`.** `binding.expr` e `NamedClip.expr` já existem; a distinção per-clip/global é **derivada**, não guardada. O layout do postcard não move. Um doc *com* expressões mudando de comportamento (fadeando, `value` per-strip) é o upgrade pretendido, não uma quebra de formato.
- **A frozen contract não é tocada** (§6 do CLAUDE.md lista Nodes/Tools/Vector-doc; o blend/fade da timeline não é contrato congelado). `ph2d-expr` **é** FROZEN (ADR-0039) e **não é tocado** — o parser vive no `ph2d-expr-parse` e o avaliador é reusado como está.
- **`expr_pass.rs` é majoritariamente aposentado:** o laço de sobrescrita (`:251-256`) e a colocação pós-composição são **deletados**; `collect_links`/`resolve_link`/`topo_order`/`ExprBindings` **migram** para um `frame_solve.rs` novo (o escalonador da Fase 2). `clip_expr_clock`/`ExprWindow`/`sole_strip_of` sobrevivem só para a checagem `PlaysTwice` do K-authoring.
- **Custo — MEDIR antes de declarar** (§0): o escalonador re-roda `eval_frame` por canal dirigido em ordem de dependência + parse por-frame (medido 335 ns; cache foi medido e rejeitado em `expr_pass.rs:141-146`). O caminho sem-expressão é intocado. **Re-medir no gatilho que o próprio código nomeia: centenas de canais com prop-link.**

---

## 7. Referências

- Design completo + as três verificações adversárias: transcrição da orquestração multi-agente (14 agentes, 4 fases) desta linha, 2026-07-27.
- Código-âncora (worktree `line/anim`): `crates/ph2d-timeline/src/stack_eval.rs` (`:94` `eval_frame`, `:142` recursão, `:155-165` sítio de amostra, `:210-246` blend, `:348-382` `invert_stack`), `src/expr_pass.rs` (`:100-256` o passe a aposentar), `src/apply.rs` (`:48/:53/:73/:104-106/:127/:181-200`), `src/stack_frames.rs` (`:214-224` strip sintético, `:294` held, `:355-369` pesos), `src/autokey.rs` (`:251/:264/:304`), `tests/fade_fingerprint.rs`.
- SOTA: Houdini CHOPs (Feedback CHOP, canal uniforme), Unreal (Anim Blueprint/Control Rig, produzir-a-montante), After Effects (`value` por-camada), Blender (mover blend do strip para a CAMADA — Baklava), Cavalry (falloff/behaviours).

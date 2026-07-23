# HANDOFF — `line/anim`: os bugs da DURAÇÃO EXPLÍCITA + autokey (2026-07-23)

> ## ⬛ FECHADO NA SESSÃO SEGUINTE (2026-07-23, commits `ae99a27bd` + `743c8ef11`) — AGUARDANDO RE-SMOKE
>
> **Os 3 bugs têm fix commitado, gate red-first e mutação provada.** O que a investigação
> (4 lentes paralelas, linha a linha) confirmou/refinou sobre este brief:
>
> - **Bug B (superbug)** — mecanismo confirmado, escopo MENOR que o §2 dizia: sob pilha/container
>   reais o seed==sample **já fechava** pelo scratch primado (cortado por dentro); a doença morava
>   só nos **caminhos escalares** — solo/Keys (`t_src` cru em `autokey_pass.rs`) e o **fallback
>   raiz-vazia** (3 gates idênticos: `key_home`/`shown_value`/`key_value_in_active_clip`). Os
>   repros (ii)/(iii) do §2 **não existiam** (o diff empilhado ignora `t_secs`). E o **K manual
>   compartilhava o buraco** (`key_time` no Arrange, `key_authoring_solo` no Keys) — fechado junto.
>   Fix: `t_cut` por-modo no passe (espelho exato de `apply.rs:208`/`:82`/frame-0 do scratch) +
>   corte dentro do gate raiz-vazia de `key_home` + corte no `key_authoring_solo`. Edição
>   deliberada além do corte **keya NA fronteira** (o frame visível) — a surdez foi rejeitada.
>   Todos os cortes são `t.min(len)` ⇒ comutam/idempotentes; o par `prime`/`debug_assert_scratch_at`
>   ficou coerente até com clamp furado. Gates: `a_key_authored_beyond_the_cut_lands_on_the_boundary`
>   (crate) · `autokey_cut_clock_tests.rs` (split HR-18: 2 vistas de scrub + edição na fronteira) ·
>   `solo_k_beyond_the_cut_keys_at_the_boundary` (K). **3 mutações, cada uma sangra o seu.**
> - **Bugs A+C** — **H2 CONFIRMADA como mecanismo único; H1/H3/H4 REFUTADAS** (véu visível e de
>   altura cheia, clamp são, router são, commit-always são — o véu ausente do screenshot é
>   consequência: a duração autorada era **22**, não 2). Não havia select-all ao focar
>   (`init_number_buffer` colapsava a âncora; os testes antigos confessavam com `Backspace × 5`).
>   Fix: **foco seleciona tudo** (clique E Tab), o Down que focou não re-colapsa via
>   `place_text_caret`, 2º clique posiciona o caret (modelo Blender/AE). ⚠️ **Vale para TODO
>   chip numérico do app** (a direção que este handoff prescreveu sem escopo) — o re-smoke deve
>   dar uma passada em outros chips (Inspector/physics) para confirmar que substituir-ao-digitar
>   é o esperado em todo lugar. Gates que **dirigem o gesto real** (clique no rect pintado +
>   `dispatch_text_input` + Enter): `number_input_focus_replaces.rs` (editor-core) ·
>   `duration_chip_gesture.rs` (painel com populate REAL; a mutação reproduz `Some(22.0)`).
> - Suítes: **1440 crates + 1014 shell (debug) · 1440 + família timeline (release) · clippy 0 ·
>   LOC caps verdes** (split `autokey_cut_clock_tests.rs`).
> - **§7 corrigido:** o gate `the_duration_chip_writes_the_scope_on_screen` citado ali **não existe
>   no HEAD** (nasceu em `642c46fca`, morreu em `1e9017eaa` — papel migrou pro router do painel).
>
> **RE-SMOKE (fila §6.3):** aba Keys, clip com keys até N, Dur = M&lt;N →
> (a) digitar M na caixa (clique simples, SEM limpar) autora M — a caixa continua mostrando M;
> (b) além de M a área escurece e o playhead PARA em M;
> (c) na aba Arrange (cena sem Dur própria), AutoKey armado, arrastar o playhead além de M não
> cria NENHUMA key com o objeto parado; (d) mover o objeto de propósito além de M keya EM M;
> (e) um clique na metade direita do chip é o stepper (±1 frame) — comportamento pré-existente,
> ver "Aberto".
>
> **Aberto (cantos nomeados, não-bloqueantes):**
> - **K numa track Time além do corte** segue no relógio cru (os braços TimeRemap de
>   `key_value_for`/`key_authoring_solo` não cortam) — canto ultra-estreito, decisão de design
>   (autorar o MAPA além do corte pode ser deliberado).
> - **D3 (pré-existente, documentado no fix):** no Arrange raiz-vazia o próprio apply compõe
>   `clip_cut(raw)` no ramo escalar e `clip_cut(cut_scene(raw))` no scratch — divergem só com
>   clamp furado + `scene_length` autorada + entidade com remap.
> - **O stepper do chip Dur** (`±1/fps` no terço direito, sem `mark_chip_no_stepper`): um clique
>   "para focar" que caia ali autora derivado±1 frame em silêncio. Legítimo como stepper;
>   ergonomia a julgar no smoke.
>
> O texto abaixo é o brief ORIGINAL da sessão anterior, mantido como registro.

---

> **Você assumiu a linha `line/anim`.** ANTES de ler qualquer arquivo:
> ```
> cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim
> pwd && git branch --show-current   # tem de dizer .../Worktrees/line-anim  e  line/anim
> ```
> A janela abre na raiz (`/home/enio/Documentos/Projetos/PH2D` = `main`) e o **mesmo path relativo existe nas duas árvores** — editar a errada compila e commita sem erro. O primário tem `?? docs/Tilling/` intocável; **não toque nele.**
>
> **Modo L:** você FECHA a linha, escreve o handoff de integração e PARA. **NUNCA** integra, faz push, nem `git checkout`/`git stash`/`git add -A`. Commit local com `git commit --no-verify -m "msg" -- <seus paths>`. Rode `--release` E debug (o `ship.sh` roda release; um pânico só-de-debug fica invisível — lição do Flip Colorize).

Rode o app assim (feche a instância aberta antes — o binário em execução é anterior ao `935cc71e9`):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop --release
```

---

## 0. Contexto: o que essa linha construiu (e o que o Enio reprovou)

A `line/anim` implementou a **duração explícita por-vista** (o modelo *composition duration* do After Effects): cada **clip**, **container** e a **cena (Arrange)** ganha um `length_override: Option<f64>`. Uma duração autorada:
- define "o fim" (go-to-end, loop fresco);
- **CORTA** o excesso **não-destrutivamente** — todo relógio entregue a um strip/track passa por `cut(t) = t.min(length)` (`TimelineDoc::clip_cut`/`container_cut`/`cut_scene`/`cut_source`, `crates/ph2d-timeline/src/doc_extent.rs`);
- deveria **escurecer** a área além do fim (o *véu*) e **prender** o playhead em `[0, fim]`.

`DOC_VERSION = 11` (campo apendado; v10 recusado no load). `PROJECT_SCHEMA` fica em 29 (o blob do `TimelineDoc` carrega a própria versão).

**Commits desta sessão** (todos locais, `--no-verify`):
```
935cc71e9 fix(anim): digitar no Dur(s) o valor MOSTRADO agora AUTORA a duracao (veu + clamp ligam)
fcdd97194 fix(anim): depois do ULTIMO strip a lane SOLTA
15841212e fix(anim): o chip Dur escreve A VISTA + loop preso a duracao
1e9017eaa feat(anim): a duracao autorada FECHA a vista -- veu, clamp, chip por escopo, lista sem playhead
642c46fca feat(anim): duracao EXPLICITA por clip/container/arranje (modelo AE) + chip Dur(s)
```
(Antes disso, nesta MESMA linha: additive de container, gap de pingpong, rename de lane, relógio próprio do container — já smokados OK; **não são o assunto deste handoff**.)

**O Enio SMOKOU o `935cc71e9` e REPROVOU com três defeitos.** Palavras dele:

1. **A caixa numérica de Duração não está correta.** *"Talvez nem esteja correspondendo à duração real do clip, embora no auto key ela pareça se settar corretamente."*
2. **SUPERBUG — autokey volta a mintir keys.** *"Quando coloco um valor baixo na duração e o keyhead está fora da área de duração, o autokey produz um bug que já havia sido resolvido: cria keyframes se arrasto em toda timeline por onde arrasto o playhead, mesmo o objeto parado."*
3. **O véu não escurece e o playhead não é preso.** *"Não resolveu o véu (área escura na timeline) nem a limitação do playhead (não permitir ir além da área definida na duração)."* No screenshot: aba **Keys**, **Dur(s) = 2**, keys terminando em t=2, playhead em **3 s**, **nenhuma** área escura.

---

## ⚠️ 1. A DISCIPLINA antes de tudo (leia isto ou vai repetir o meu erro)

**Eu segui a cadeia de dados INTEIRA lendo o código e ela está CORRETA no papel** — Enter → `commit_number_buffer(force=true)` → `ValueChanged` → intent `SetClipLength` → drain → `apply_length` → `set_clip_length_override`; e o véu, o clamp e o corte **gateiam todos no MESMO `clip_length_override(active).is_some()`**. Tudo consistente. **E mesmo assim o produto falha.**

Isto é o caso *"verde no papel, vermelho no produto"* que este projeto documenta à exaustão (costura não-testada · "audit" = compilar). **NÃO confie em ler o código. INSTRUMENTE e RODE o smoke real** (`--release`). A causa mora num detalhe de runtime que a leitura não revela. O plano de instrumentação está na §5.

---

## 2. BUG B (o SUPERBUG) — autokey volta a mintir keys além do corte

**Este é o defeito com causa mais clara e o mais grave. Comece por ele.** Confirmado com evidência de linha (investigação paralela): **seed ≠ sample reintroduzido pelo corte, mascarado — mas NÃO corrigido — por um clamp de posição.**

### ⚠️ Repro que de FATO dispara (não é o scrub trivial de Keys)
O clamp de playhead do `run()` (§ abaixo) **esconde** o repro óbvio na aba Keys, porque ali o braço do clamp (`clip_length_override(active)`) é IDÊNTICO ao corte do apply (`clip_cut(active)`) → o playhead nunca passa do corte → `raw t == cut t` → sem divergência. O bug sobrevive onde o clamp e o corte **discordam**:
- **Arrange com pilha VAZIA + o clip com `length_override`=2 e `scene_length` NÃO setado.** O braço Arrange do clamp lê `scene_length = None` → **não clampa** → escrube pra t=3; o apply congela em `clip_cut` (`apply.rs:82`) mas o autokey amostra `curve(3)` cru → key espúria por posição. **Este cenário reproduz os DOIS sintomas do Enio de uma vez** (playhead invade **e** autokey minta keys).
- Qualquer **strip empilhado** cuja fonte tenha override menor que o clamp da vista.
- **Dentro de um container** cujo strip interior seja menor que o override do container.

### Mecanismo (CONFIRMADO — `feedback_derived_coordinate_seed_must_match_sample`)
O apply escreve `pose = curve(remapped_time(entity, cut(t)))`; o autokey lê `curve(remapped_time(entity, t CRU))`. Além do corte, `curve(cut(t))` (congelado na duração autorada) ≠ `curve(t cru)` (ainda animando) → o compare exato reporta "fora da curva" e minta uma key **no tempo cru** (além da duração) a cada frame.

- **APPLY corta:** `crates/ph2d-timeline/src/apply.rs:82` (`apply_from_doc_except`, solo: `t_solo = doc.clip_cut(active, t)`) · `apply.rs:208` (`apply_active_clip`: `clip_t = doc.clip_cut(active, clip_t)`) · `stack_frames.rs:310` (`doc.cut_source(strip.source, t_local)` por strip; frame-0 `cut_scene`/`container_cut` em `:200-201`, solo `:221`, interior `:417`).
- **AUTOKEY NÃO corta:** `shells/desktop/src/render_loop/autokey_pass.rs:194` (`t_src = remapped_time(&doc, entity, playhead.time())` — CRU) · `:241` (`t_diff = t_src`) · o diff em `crates/ph2d-timeline/src/autokey.rs:130` (compare **exato** `v != sampled`, `v` = pose viva) via `track.sample(t_secs)` (`autokey.rs:271`). O tempo de INSERÇÃO do key também é cru: `key_home` / `apply.rs:310`. O passe é chamado em `render_loop/mod.rs:4550`, relógio escolhido em `:4543-4549`.
- O corte (`length_override`) é um transform NOVO (commits `642c46fca`/`1e9017eaa`) inserido **só do lado do apply**; o relógio de seed/diff do autokey não foi atualizado junto — os próprios comentários do módulo já citam a lição (`apply.rs:266-268`, `:355-357`, `autokey.rs:45-59`).

### O clamp de posição MASCARA, não corrige (`timeline_bridge.rs:57-77`)
O `run()` clampa o playhead ativo em `[0, end]` com `end = clip_length_override(active)` (Keys) / `container_length_override(c)` / `scene_length` (Arrange), **antes** do autokey. Em Keys o braço == o corte do apply, então esconde o repro trivial. Mas é um **band-aid posicional num eixo só**: o apply corta CADA fonte (strip/solo), o clamp só a override top-level da vista. Onde discordam (repro acima), a doença fica intacta.

### Fix (direção — seed == sample, NÃO alargar o clamp)
Em `autokey_pass.rs::apply_samples`, **corte `playhead.time()` pela MESMA regra do apply** antes de derivar `t_src`/`t_diff` (`:194`) e `home`/`t_e` (`:216-227`):
- solo/Keys: `doc.clip_cut(doc.active_index(), t)` (espelha `apply.rs:208`);
- empilhado/container: rote pela `cut_source` do strip exatamente como `stack_frames.rs:310` (o scratch já está primed rooted em `apply_samples:154`).

Assim `curve(cut(t)) == curve(cut(t))` além do corte → zero delta → zero key espúria, **independente** do clamp. O clamp vira UX (prender o playhead), não a fronteira de correção. **Alargar o clamp só re-mascara** — não restaura o invariante seed==sample que o módulo exige.

**Alternativa (pior, se o escopo tiver de ficar mínimo):** suprimir o autokey quando `playhead.time() > authored_end` — mas aí a região cortada deixa de aceitar edição deliberada de pose, e continua sendo um band-aid posicional em cima do seed≠sample. Prefira o corte do relógio (seed==sample).

**Gate red-first obrigatório:** monte um clip com keys 0..4, duração 2, AutoKey armado, e **escrube** o playhead por vários t>2 SEM tocar no objeto; o oráculo é **`clip.track.keys().len()` inalterado** (nasce VERMELHO revertendo o fix). O fixture TEM de conter o fenômeno: duração **menor que o conteúdo** e a região cortada **mid-animation** (não chapada — uma curva chapada esconde o bug).

### Corolário (defesa em profundidade)
Se o **clamp do playhead** (Bug 3) funcionasse, o playhead nunca ESTARIA além do corte e o superbug não teria como disparar. Mas o fix correto é seed==sample (há rotas — scrub direto — que podem furar o clamp; e o clamp é UX, o seed==sample é correção).

---

## 3. BUGS A + C — a caixa mostra valor errado; sem véu; playhead invade

Trato A e C juntos porque **muito provavelmente compartilham raiz**: **a duração autorada não "gruda"** no caso em que o valor digitado é IGUAL ao valor derivado mostrado. Se a duração não é autorada, `length_override` fica `None` e então — de uma vez só — **não há corte, não há véu, não há clamp**, e a caixa **volta** ao valor derivado no frame seguinte (o `mirror_number` reescreve `value` do snapshot toda vez que a caixa não está focada). Isso explica A ("não corresponde / não gruda") e C (véu + clamp) com um mecanismo só.

> ⚠️ Note a coexistência com o Bug B: o superbug (corte ativo) prova que **em ALGUM cenário a duração É autorada** — provavelmente quando o valor digitado **difere** do derivado (o delta-gate normal dispara). O que falha é o caminho **valor-igual-ao-mostrado** (o "commit-always" que eu adicionei no `935cc71e9`), que é EXATAMENTE o cenário do screenshot do Enio (digitou 2, a caixa já mostrava 2).

### A cadeia INTENDIDA (está toda correta no papel — confira cada elo COM LOG)
1. **Enter** com a caixa focada → `crates/ph2d-editor-core/src/interaction/dispatch/key.rs:110`: `commit_number_buffer(store, id, &mut events, /*force=*/true)`.
2. **`commit_number_buffer`** (`dispatch/mod.rs:249`): se o valor digitado == `prev_value` (no-change), cai no ramo `dispatch/mod.rs:335`: `if force && store.number_commit_always(id) { events.push(ValueChanged(id)); }`.
3. **A flag** `number_commit_always(TIMELINE_LENGTH_NUM)` é setada UMA vez em `crates/ph2d-panel-timeline/src/populate.rs:103` (`set_number_commit_always`). O `WidgetStore` é único e persistente (populate roda 1× no install; nada limpa o `BTreeSet`).
4. **Router** (`crates/ph2d-panel-timeline/src/event.rs:127`): `ValueChanged(TIMELINE_LENGTH_NUM)` → lê `number_value(id)`, `len=(v>0).then_some(v)`, escolhe escopo por `transport::length_scope(snap.container_open, state.tab)` → na aba Keys = `LengthScope::Clip` → `push_intent(SetClipLength{len})`.
5. **Drain** (`shells/desktop/src/render_loop/mod.rs:1040`): `ph2d_panel_timeline::drain_intents()` → `self.timeline_intents`.
6. **Apply** (`render_loop/timeline_bridge.rs:54`): `run()` faz `intents.drain(..)` → `apply_intent` → `SetClipLength` (`intent_apply.rs:92`) → `apply_length(ActiveClip)` (`intent_loop_sync.rs:79`) → `doc.set_clip_length_override(active, len)` (`doc_extent.rs:183`, filtra `>0.0`).
7. **Snapshot** do frame seguinte (`crates/ph2d-timeline/src/snapshot.rs:328`): `view_length_explicit = clip_length_override(active).is_some()` = **true**.
8. **Véu** (`crates/ph2d-panel-timeline/src/ruler.rs:80`): pinta `BgScrim` de `beyond_end_shade` quando `view_length_explicit` (gate `clock.loop_band`, que é `true` na aba Keys — `ruler_clock.rs:84`).
9. **Clamp** (`timeline_bridge.rs:63`): `(None, true) => clip_length_override(active)`; se `playhead>end`, `pause()+seek(end)`. Roda DEPOIS do drain (mesmo frame), então pega até um scrub.

**Tudo isto está consistente:** véu (passo 8), clamp (passo 9), corte (Bug B) e `view_length_seconds` (a caixa, passo 4) leem/escrevem o MESMO `clip_length_override(active_index())`. `solo = keys_mode` e `container` confirmados no call-site (`mod.rs:1231-1232`).

### Onde a cadeia PODE furar em runtime (hipóteses ranqueadas — INSTRUMENTE)

- **(H1, mais provável) O commit-always não gruda no caso valor-igual.** Instrumente o passo 2/3: um `eprintln!` no ramo `dispatch/mod.rs:335` (`force`? `number_commit_always(id)`? disparou o `ValueChanged`?) e no router (`event.rs:127`: qual `len`, qual escopo?). Se o `ValueChanged` NÃO dispara: o Enter não chega com `force=true`, ou `prev_value != parsed` (ver H2), ou a flag é `false`.
- **(H2) O foco NÃO faz select-all, então digitar APENDA ao buffer.** Clicar 1× na caixa foca sem selecionar (só double-click / Ctrl+A selecionam — `dispatch/mod.rs:211`). O `init_number_buffer` semeia o buffer com `format_number(value)` (ex.: `"2"`); digitar `"2"` vira `"22"` → parse `22.0` ≠ prev `2.0` → o delta-gate NORMAL autora `Some(22.0)`. A caixa então mostra **22** (Bug A: "não corresponde"), o corte em 22 (>conteúdo) não faz nada, o véu em 22 s sai da tela (Bug C: "nada escuro"), e o clamp em 22 não prende um playhead em 3. **Isto sozinho explica os três sintomas de A+C.** Verifique o comportamento REAL de foco+digitação da caixa (select-all no foco? o Enio espera substituir, não apender). Se for isso, a correção é **selecionar-tudo ao focar o chip numérico** (ou o Enio digita já limpando).
- **(H3) O `BgScrim` é invisível no tema da timeline.** `crates/ph2d-tokens/src/color.rs:288`. Confirme que resolve para um overlay escuro com alpha visível SOBRE o `TimelineRulerBg`/rows. Se o alpha for ~0, "nada escuro" é literal mesmo com `view_length_explicit=true`.
- **(H4) O `region` passado a `ruler::paint` cobre só a tira de 22 px**, não as rows. O véu usa `region.h` (`ruler.rs:227`) igual à linha do playhead. Se o playhead cruza rows mas o véu não, o `region` diverge entre os dois. Confira `paint.rs` (quem chama `ruler::paint`).

### Fix (direção — depende de qual H a instrumentação confirmar)
- Se **H2**: chip numérico deve **select-all ao focar** (a caixa é um readout que o artista SUBSTITUI). Aí o commit-always (que eu adicionei) volta a ser necessário só pro caso "digitou exatamente o mesmo número" e o resto do desenho está certo.
- Se **H1**: conserte o elo que não dispara (provavelmente `event.rs`/drain/apply — mas confirme com log, não com leitura).
- Se **H3/H4**: token/região.

**Gate red-first para C:** com uma duração autorada `< conteúdo` e o playhead além dela, (a) `beyond_end_shade` devolve `Some(rect)` com `x` on-screen (já existe: `ruler.rs::the_shade_starts_at_the_authored_end_and_only_when_authored`) **e** (b) depois de `run()`, `playhead.time() == end` (o clamp já tem gate em `timeline_bridge_container_tests.rs::an_authored_duration_pins_the_playhead_and_pauses_the_run_past_it`). **Esses gates PASSAM hoje** — é a prova de que o bug é product-red/unit-green: o fixture não contém o gesto real (foco+digitação+Enter na caixa). **Escreva um gate que DIRIGE o gesto real** (o `dispatch_key` de Enter sobre a caixa focada, não `set_clip_length_override` na mão).

---

## 4. A caixa "não corresponde à duração real" (Bug A, a metade de leitura)

Independente de H2, confirme que `view_length_seconds` na aba Keys é a duração REAL do clip:
- `snapshot.rs:327`: `view_length_seconds = doc.view_end_seconds(keys_mode)`.
- `view_end_seconds(true) = end_seconds() = clip_end_seconds(active_clip)` (`doc_extent.rs:19,87`): `override` se autorado, senão `max(clip.duration(), último_key)`.
- Confirme que `active_clip` (campo) e `active_index()` (método) concordam, e que a caixa mostra o clip cujas keys estão na tela. Se `clip.duration()` estiver desatualizado vs. os keys (o `insert_key` não estende `duration()`), `max(..., last_key)` cobre — mas verifique com log.

---

## 5. Plano de INSTRUMENTAÇÃO (faça isto ANTES de mexer no código)

Ponha `eprintln!` temporários (remova antes de fechar) nos 5 pontos e rode o smoke real digitando na caixa:

1. `dispatch/mod.rs:335` (ramo commit-always): logue `force`, `number_commit_always(id)`, se empurrou `ValueChanged`, e `parsed`/`prev_value` do ramo de cima (H2 aparece aqui: `parsed=22, prev=2`).
2. `event.rs:127` (router da Dur): logue `v`, `len`, o `LengthScope` escolhido.
3. `intent_loop_sync.rs:79` (`apply_length` ActiveClip): logue `ix` e `len` — o override foi setado, em qual índice?
4. `snapshot.rs:328`: logue `view_length_explicit` e `view_length_seconds` a cada rebuild.
5. `timeline_bridge.rs:63-76`: logue `authored_end`, `playhead.time()`, e se o clamp disparou.

O primeiro ponto da cadeia onde o número esperado NÃO aparece é a causa. **Não adivinhe; meça.**

Para o Bug B, um log em (a) o relógio que o apply usa e (b) o relógio que o autokey amostra, no MESMO frame de um scrub além do corte, mostra a divergência `curve(3)` vs `curve(2)` na hora.

---

## 6. Fila de implementação (ordem sugerida)

1. **Bug B (autokey seed==sample via cut)** — causa clara, fix contido, gate red-first óbvio. Fecha o superbug **e** blinda contra o "playhead além do corte".
2. **Bug A+C** — instrumente (§5), confirme H1/H2/H3/H4, conserte a raiz. Provavelmente **H2 (select-all ao focar o chip)** é a peça grande; o resto do desenho (véu/clamp/escopo) já está certo e gateado.
3. **Re-smoke** com o Enio: aba Keys, clip com keys até N, Dur = M<N, arraste o playhead além de M → deve escurecer além de M, o playhead deve PARAR em M, e o AutoKey **não** deve mintir key nenhuma ao escrubar.

**Não** re-litigue o desenho da duração explícita (modelo AE, corte não-destrutivo, escopo = a VISTA) — o Enio aprovou o modelo (item 3 dele); o que falha é a EXECUÇÃO (autokey + a caixa não gruda + véu/clamp por consequência).

---

## 7. Estado / suítes / flake

- Todas as suítes verdes em `cargo test -p` (`ph2d-timeline`, `ph2d-panel-timeline`, `ph2d-editor-core`, shell) no fechamento do `935cc71e9`, **exceto** a flake conhecida e PRÉ-EXISTENTE `the_cost_of_depth_is_linear_not_explosive` (`ph2d-timeline/tests/nesting_clock.rs`) — gate de RAZÃO sensível a carga; **passa isolado**, re-rode sozinho antes de suspeitar de merge.
- Clippy 0.
- **Os gates existentes de véu/clamp PASSAM** e o produto FALHA — essa é a bandeira vermelha da §3: os fixtures chamam `set_clip_length_override` na mão e nunca dirigem o gesto real (Enter na caixa). Os gates novos têm de CLICAR/DIGITAR.
- Gates relevantes já no repo: `ruler.rs::the_shade_starts_at_the_authored_end_and_only_when_authored` · `timeline_bridge_container_tests.rs::{the_duration_chip_writes_the_scope_on_screen, an_authored_duration_pins_the_playhead_and_pauses_the_run_past_it}` · `number_input_mapped_link.rs::{a_flagged_chip_commits_an_unchanged_value_on_enter, ...}` · `seam.rs::the_length_chip_writes_the_scope_the_one_door_names`.

## 8. Mapa de arquivos (quem faz o quê)

| Assunto | Arquivo |
|---|---|
| corte / cut / overrides / getters/setters | `crates/ph2d-timeline/src/doc_extent.rs` |
| `DOC_VERSION`, `NamedClip.length_override`, `scene_length` | `crates/ph2d-timeline/src/doc.rs` |
| apply do clip/stack/container (cortam) | `crates/ph2d-timeline/src/apply.rs`, `stack_frames.rs` |
| intents `Set*Length` | `crates/ph2d-timeline/src/intent.rs` |
| `apply_length` + `clamp_loops_to_lengths` | `crates/ph2d-timeline/src/intent_apply.rs`, `intent_loop_sync.rs`, `doc_loops.rs` |
| snapshot `view_length_seconds`/`view_length_explicit` | `crates/ph2d-timeline/src/snapshot.rs` |
| chip Dur(s), `length_scope`, `LengthScope` | `crates/ph2d-panel-timeline/src/transport.rs` |
| mirror do chip, `mirror_number` | `crates/ph2d-panel-timeline/src/transport_widgets.rs` |
| router da Dur (ValueChanged→intent) | `crates/ph2d-panel-timeline/src/event.rs` |
| `set_number_commit_always` (populate) | `crates/ph2d-panel-timeline/src/populate.rs` |
| véu (`beyond_end_shade`) + gate `loop_band` | `crates/ph2d-panel-timeline/src/ruler.rs`, `ruler_clock.rs` |
| commit-always (`commit_number_buffer`, `force`) | `crates/ph2d-editor-core/src/interaction/dispatch/mod.rs`, `key.rs` |
| flag `number_commit_always` (store) | `crates/ph2d-editor-core/src/interaction/state/{mod.rs,store_core.rs}` |
| `run()` (drain + clamp + apply) | `shells/desktop/src/render_loop/timeline_bridge.rs` |
| drain dos intents + `solo`/`container`/`keys_mode` | `shells/desktop/src/render_loop/mod.rs` (~1040, ~1224) |
| **autokey pass (Bug B mora aqui)** | `shells/desktop/src/render_loop/autokey_pass.rs` + `mod.rs` (~4539) |

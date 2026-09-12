# HANDOFF de INTEGRAÇÃO — `line/app-physics`, **FASE C** (o corte final)

> **Data:** 2026-09-12 · **Ramo:** `line/app-physics` · **Merge-base:** `db5a4b7f2` · **6 commits**
> · **101 ficheiros** · **Fase A e B:** 11/09 ([A](HANDOFF_INTEGRACAO_line_app-physics_FASE_A_2026-09-11.md)
> · [B](HANDOFF_INTEGRACAO_line_app-physics_FASE_B_2026-09-11.md))
>
> ⚠️ **Isto descreve o mundo em 12/09.** O estado vivo é o `CLAUDE.md §5`.

---

## §1 · O veredito, em números

| grandeza | antes (main de 12/09) | depois | Δ |
|---|---:|---:|---:|
| **shell** (`shells/desktop`, como a catraca mede) | 373 937 L / 1 498 f | **360 599 L / 1 449 f** | **−13 338 L · −49 f** |
| `shells/desktop/src/physics/` | 60 f / 15 625 L | **13 f / 3 182 L** | **−47 f · −12 443 L** |
| `crates/ph2d-app-physics` | 167 f / 36 592 L | **217 f / 50 224 L** | +50 f |

**A prova (`scripts/nextest-list-diff.py`), exacta nos dois sentidos:**

```
antes: 22668 testes (22041 chaves) | depois: 22668 (22041)
MOVED (mesma chave, outro pacote/binário): 169
ONLY-A (perdidos): 0
ONLY-B (novos):    0
```

**Portão batched:** `scripts/nextest-impacted.sh` → **16 245 testes, 16 245 passed, 0 failed.**
**`cargo test -p ph2d-host-desktop --test it` à parte (regra 2 do §1):** **812 passed · 0 failed.**
**Clippy** (`-p ph2d-app-physics -p ph2d-host-desktop --all-targets`): **zero avisos desta linha** (§7).
**`cargo test -p ph2d-app-registry-init`:** verde. **`cargo run -p ph2d-app-sync`:** `ok: 4 blocos`, registo
inalterado (nenhuma família nasceu).

⭐ **Zero contador partilhado se move** (`collision-surface.sh`, corrido nesta worktree): `PROJECT_SCHEMA`
**128** (base 128) · tripla `(128, 13, 22)` igual · `FLIP_SCHEMA` 13 · `DOC_VERSION` 18 · os dois espelhos
de registo em **86** · contratos §6 intocados · **nenhum `+name` novo no `Cargo.lock`** · zero marcadores de
conflito · nenhum tecto de LOC estourado.

---

## §2 · ⭐⭐⭐ A RESPOSTA À PERGUNTA DO BLOCO: zero sextos métodos, outra vez

O bloco avisava: *«é aqui que um sexto método parece necessário, e não é»*, porque gesto de canvas e
inspector **são** a metade que toca a `App`. A régua 4 do §1 foi o instrumento, e a resposta é a mesma da
Fase B — **o que bloqueia nunca é a `App`**:

| o que era | escrito em TIPOS | de onde vem |
|---|---|---|
| `self.input_actions` · `self.input` | `&mut ActionState` · `&InputState` | `ph2d-input` |
| `self.gfx…hero_screen` | `Option<&mut HeroScreen>` | `ph2d-editor` |
| `self.gfx.{camera,surface,sim,physics,present,toasts}` | o **`CanvasCtx`** | 4 crates irmãs |
| `self.modifiers` | `ctrl: bool` **já resolvido** | a shell tem o teclado |
| `self.any_input_this_frame` | **fica no INVÓLUCRO** | é da `App`, e só ela o lê |

⭐⭐ **E o achado que vale mais que a tabela: o `std::mem::take` DESAPARECEU.** O
`resolve_player_input` era um `impl crate::App` cujo doc **explicava o truque** — *«o mapa vive no
`HeroScreen` (dentro de `self.gfx`) e o estado resolvido vive na `App`; tocar nos dois ao mesmo tempo seria
emprestar `self` duas vezes»*. Sem `self` não há duplo empréstimo: quem faz o empréstimo **disjunto** é a
shell, no sítio da chamada, que é onde os campos vivem.

> *O truque não era uma lei do domínio — era o preço de a função estar na struct errada.*

⛔⛔ **E o MESMO mecanismo mordeu na cura.** A 1.ª versão do invólucro tinha um
`fn physics_canvas_ctx(&mut self) -> Option<CanvasCtx<'_>>`, que empresta **`self` INTEIRO**; o
`joint_draw_move` a seguir pediu `&mut self.physics` e o compilador respondeu `E0499`. ⇒ o construtor do
contexto fica **repetido nos quatro invólucros**, com a razão escrita no ficheiro:
*o empréstimo disjunto só existe quando os campos são nomeados no MESMO escopo que os usa.*

### ⛔ Por que não um trait de extensão (o que o HOWTO §1.5 prefere)

O molde do piloto é *«o `impl App` vira um trait de extensão sobre `AppHost`»*, e ele **não serve aqui**:
o corpo destes gestos precisa do `gfx`, e nenhum método do `AppHost` devolve um handle — o trait proíbe-o
**por escrito**, e é essa proibição que segura a fronteira inteira. A saída é a outra metade da mesma
regra: os **CORPOS** saem, e a shell fica com invólucros. A objecção do piloto a funções livres era
**14 sítios de chamada**; aqui são **quatro**.

---

## §3 · O fecho RE-MEDIDO, e o que ele mudou no plano

⭐ **O §6 do handoff da Fase A descrevia uma árvore de 11/09, e o bloco mandou reconferi-lo.** Reconferido:

- As **três folhas** que aquele §6 nomeava (`inspector_ordering` · `preview_drive` · `name_unique`)
  **dissolveram de facto** — `0` citações de código a partir de `physics/`. O único resíduo era **prosa**:
  a tabela do doc-comment do meu próprio `mod.rs`.
- ⇒ o fecho passou de *«19 ficheiros presos por três folhas»* para **6 âncoras = 6 SÍMBOLOS**, e **cinco
  deles só são tocados por TESTES**:

| âncora | símbolo | quem toca | veredito |
|---|---|---|---|
| `app_state.rs` | `GroupDragSnapshot` | 1 produto + 1 teste | **curada** → `ph2d-editor-core` |
| `render_loop/point_gizmo.rs` | `joint_anchor_handles` | 1 teste | **curada** → o ficheiro era física |
| `init.rs` | `build_component_registry` | 2 testes | **fica** (composição) |
| `component_attach.rs` | `attach_by_name` | 2 testes | **fica** (porta de produção) |
| `undo.rs` | `ProjectState` | 1 teste | **fica** (unidade de undo) |
| `project_library.rs` | `LibraryDoc` | 1 teste | **fica** |

**Ponto fixo medido:** sem cura nenhuma, `MOVE 39 f / 11 007 L`; curando as seis, **`60 f / 15 625 L`,
zero fica**. ⇒ as 15 625 linhas estavam presas por **seis símbolos**, não por seis módulos.

### ⭐ As duas curas, e por que são curas e não contorno

**`GroupDragSnapshot`** é `{ u64, TransformSnapshot, TransformSnapshot }` — dados puros sobre um tipo que
já vivia em `ph2d-editor-core/src/gizmo/drag.rs`. Era o caso do HOWTO §1.2 à letra: **estava na shell por
inércia**, escrito pelo gizmo, que era da shell. ⇒ o **TIPO** mudou-se para junto do conteúdo dele; o
**CAMPO** `App::group_drag_starts` ficou, porque a shell é quem possui um arrasto em curso.

**`render_loop/point_gizmo.rs`** tinha nome genérico e era **100 % física**: os `use` dele são
`ph2d_physics_ecs::{JointSide, PhysicsBridge}` e — decisivo — o `joint_glyphs` da **própria crate da
família**. As seis funções são junta, corda, roldana e âncora. ⇒ **+921 linhas de encolhimento que não
estavam no alvo de 15 625.**

---

## §4 · O que FICA na shell, e é DESENHO (13 ficheiros / 3 182 L)

| o quê | f | porquê |
|---|---:|---|
| `physics_smoke.rs` — **o PRÓLOGO** | 1 | `impl crate::App`: rebobinar · armar o toggle · abrir a timeline · play/pause. Mexe no `Playhead`, nas `flags` e no `HeroScreen`, que são da **composição**. É a decisão da Fase B, inalterada |
| `joint_gestures_app.rs` — **os INVÓLUCROS** | 1 | a costura `&mut App` → a assinatura da crate |
| `inspector_player_tests.rs` + 6 `#[path]` | 7 | atravessam a **porta de produção** do `+` do Inspector |
| `physics_gesture_tests.rs` + `_zone_` | 2 | a mesma porta |
| `bridge_tests.rs` | 1 | mede o `ProjectState` e o `LibraryDoc` — a captura desta shell |
| `mod.rs` | 1 | as declarações |

⛔⛔ **As quatro âncoras que ficam NÃO são curáveis por uma porta, e a razão é a mesma nas duas famílias:**
o `build_component_registry` regista os componentes de **CINCO crates irmãs** (`ph2d_render`,
`ph2d_physics_ecs`, `ph2d_field_ecs`, `ph2d_skeleton_ecs` + os do `ph2d-ecs`) — **ele é composição**, pela
mesma lei que mantém o `render_loop` na shell. E os gates atravessam-no **de propósito**: o doc-comment
deles já dizia *«um atalho de teste que constrói o componente por outro caminho é a segunda porta que
diverge»*. Pela tabela do HOWTO §2.6, *o gate que mede a `App`, o `ProjectState` ou a porta desta shell
vive na shell*.

⚠️ **Se alguém quiser fechá-los um dia**, o alvo é uma **folha de assunto** — *«anexar um componente pelo
nome canónico»* — e ela leva o `attach_by_name` **e** o `component_seed`; ⛔ o `build_component_registry`
**não** vai, porque é o censo do app. Isso é uma linha própria, como a `shell-folhas` foi para as outras
três — **não é trabalho desta**.

⭐ E o padrão de chegada é o da `line/app-flip` (`ESTADO_W2` §2): *«o que sobra na shell são invólucros
`*_app.rs` … isso é a costura, e fica por desenho»*. Aqui é **1** invólucro e **1** prólogo — os outros 11
são gates.

---

## §5 · ⛔⛔ AS ARMADILHAS QUE ESTA FASE PAGOU

### 5.1 ⛔⛔⛔ **O `check --all-targets` verde e **15** gates vermelhos — a regra 2 do §1 pagou-se inteira**

`cargo check -p ph2d-host-desktop --all-targets`: **0 erros, 0 avisos.**
`cargo test -p ph2d-host-desktop --test it`: **15 FAILED.**

Os 15 são gates textuais que leem um ficheiro e afirmam sobre o conteúdo. Eles vivem em
`shells/desktop/tests/it/`, que é **exactamente o directório que o `nextest-impacted` do portão FILTRA**.

> *Um `check` verde não diz nada sobre um gate que só corre.*

⇒ **toda linha desta wave corre `--test it` à parte, e não é conselho: é a diferença entre 0 e 15.**

### 5.2 ⛔⛔ **O gémeo MUDO do §2.6 sobreviveu DENTRO da crate, e a minha varredura errou de UNIDADE**

Eu varri `shells/desktop/` e reapontei **14** agulhas de caminho. O portão batched achou **mais uma**, em
`crates/ph2d-app-physics/src/joint_world_tests.rs`, ainda a ler `…/src/physics/joint_draw.rs`.

⚠️ **Dois furos, não um:**
1. varri a árvore ERRADA (só a shell — mas o corte move ficheiros *para dentro* da crate, e eles levam as
   agulhas consigo);
2. o `grep` filtrava pela linha que contém `read_to_string(` — e o caminho estava num **`concat!` de duas
   linhas**. ⇒ *uma varredura por CHAMADA não vê um caminho construído; a que vê é por **LITERAL**.*

⭐ **Curado com a metade BOA do par, não com o caminho novo:** passou a
`include_str!("joint_draw.rs")`, que **falha em tempo de COMPILAÇÃO**. O HOWTO §2.6 chama-lhe *«boa
propriedade»* — remove a classe de falha inteira em vez de corrigir uma instância dela.

### 5.3 ⛔ **Uma agulha nomeava a VISIBILIDADE — a QUARTA vez neste repo** (§2.13)

`"pub(crate) fn joint_draw_release"`. Publicar a API da crate obrigou o modificador a mudar; **a lei não
moveu uma linha** e o gate reprovava na mesma. Hoje é `"fn joint_draw_release"`.

> *Visibilidade é precisamente o que uma fronteira nova muda por construção.*

⚠️ **E a promoção foi NOMEADA, nunca em massa:** 42 declarações passaram a `pub`, cada uma apontada pelo
compilador numa varredura JSON das mensagens `E0616`/`E0603`/`E0624`. Publicar tudo teria funcionado e
mentido sobre a superfície da crate.

### 5.4 ⭐⭐ **O que NÃO partiu é a prova de que os invólucros foram a decisão certa**

`self.joint_draw_press(` · `self.joint_draw_move(` · `self.joint_draw_release(` ·
`self.joint_draw_cancel_key(` · `self.advance_joint_anchor_drag();` são nomeados por **cinco** gates em
**quatro** ficheiros — e os cinco ficaram **VERDES**: os sítios de chamada são byte-idênticos.

O único que partiu foi o `resolve_player_input`, que é o **único sem invólucro**. ⭐ E a agulha nova dele é
**melhor**: ela nomeia a PORTA (`ph2d_app_physics::player_input::resolve_player_input(`) em vez do `self.`,
que mede o **receptor** — a mesma lei da §2.13, um nível acima.

### 5.5 ⭐ **§1.3 outra vez: QUATRO dependências invisíveis, e três eram MÓDULOS DA SHELL**

`ph2d-inspector-ordering` · `ph2d-preview-drive` · `ph2d-unique-name` · `ph2d-panel-inspector`.

As três primeiras são **exactamente** as que o §6 do handoff da Fase B nomeou como *«três folhas
partilhadas, com 11/38/14 consumidores de famílias diferentes»* — e a `line/shell-folhas` fez-as crates em
12/09. ⇒ **a Fase C só foi possível porque a nota da Fase B foi lida e executada por outra linha**, que é o
`CLAUDE.md` §0.0 a funcionar em ambos os sentidos.

### 5.6 ⚠️ **Seis colisões de NOME, e elas são obra minha da Fase B**

Seis ficheiros da shell tinham o **mesmo nome** que ficheiros já na crate — as «metades de fronteira» que a
Fase B criou de propósito, em crates diferentes. Um `git mv` cru teria colidido. Renomeadas pelo que
**exercitam** (`…_bake_tests`, `…_panel_tests`, `…_edit_tests`, `…_wheel_tests`); **duas** já tinham esse
nome escrito no `mod.rs` da Fase B.

⛔ **E os outros 42 mantiveram o nome de propósito:** um rename por nome corrompe a prosa que cita o
ficheiro, e a Fase A pagou **76** citações numa varredura só (§9.9).

### 5.7 ⚠️ **Quatro links de rustdoc apodreciam DENTRO da crate, e o `cargo check` não os vê**

`[`crate::physics::physics_smoke`]` em três ficheiros **pré-existentes** e
`[`ph2d_app_physics::…`]` (o nome externo, de dentro da própria crate). Só o `cargo doc` os valida. ⇒
*um link de rustdoc partido é dívida silenciosa; ele não reprova nada.*

### 5.8 ⭐ **Um invólucro ORFÃO foi APAGADO no mesmo dia em que nasceu**

`App::disarm_joint_draw` ficou com **zero chamadores de produto** quando o `cancel_key` passou a chamar a
porta da crate. O doc dele dizia-se *«a PORTA ÚNICA de desarmar — o botão, o Esc e qualquer futuro
consumidor passam por aqui»*, e nenhum passava.

> *Retirar um gesto deixa a lei dele viva e órfã, e nenhuma sonda deste repo pergunta se uma PORTA tem
> chamador* (a lição que a `line/UIUX` escreveu em 10/09, paga aqui em 12).

⚠️ **Quem o apanhou foi um `warning: method is never used`** — o que só aconteceu porque a promoção de
visibilidade limpou os outros 81 avisos de `dead_code` primeiro. *Uma dívida grande esconde o achado no
meio dela.*

---

## §6 · ⚠️ PREMISSAS MINHAS QUE A MEDIÇÃO DERRUBOU

1. ⛔ **«A régua do fecho é cega ao `impl App`.»** — **FALSO, e eu quase o escrevi no handoff.** Eu vi que
   os quatro ficheiros com `impl crate::App` apareciam no conjunto `MOVE` e concluí que o
   `scripts/fecho-da-familia.py` não os via. **Ele vê** (linha 303: `elif n in ("App","AppGfx"): anota("::APP::", p)`)
   e **decide não os contar como âncora** (linhas 277 e 363), **de propósito** — porque a premissa desta
   wave inteira é que um `impl App` é *curável*. O meu próprio trabalho confirmou-a: **três dos quatro
   moveram-se**. ⇒ *antes de acusar um instrumento de cegueira, leia o que ele faz com o dado — registar e
   descartar não é o mesmo que não ver.*
2. ⚠️ **O que SOBRA dessa leitura é menor e real:** a régua responde *«o que o compilador permite»* e não
   sabe dizer *«este ficheiro é a COSTURA»*. Ela dá hoje `MOVE 2 f / 291 L`, e os dois são **o prólogo e os
   invólucros** — os dois que, por desenho, não podem sair. Quem leia aquele número como uma lista de
   tarefas tenta mover a raiz de composição. ⇒ a subtracção é da família, e para a `physics` está nomeada
   no `mod.rs`.
3. ⛔ **«O `point_gizmo.rs` é chrome genérico do `render_loop`.»** — **FALSO.** O bloco avisava que os 24
   `inspector_*` do `render_loop` não são meus, e eu quase estendi isso a todo o directório. O
   `point_gizmo.rs` importa `ph2d_app_physics::overlay::joint_glyphs`: ele **já dependia da minha crate**.
   *A unidade da posse é o ASSUNTO, nunca a pasta* — a mesma lei que a `line/app-vec` pagou em 12/09 sobre
   um prefixo de nome de ficheiro.
4. ⚠️ **«A shell mede-se por `shells/desktop/src`.»** — **FALSO**, e a minha primeira medição leu
   **339 853** onde a verdade é 373 937. A catraca mede `shells/desktop` **inteiro** (`tests/`, `build.rs`).
   *Meça com a régua do gate, nunca com uma sua.*

---

## §7 · Para o INTEGRADOR

1. ⛔⛔ **A CATRACA vai ficar VERMELHA, e eu não lhe toquei** (regra 1 do §1). Números exactos:
   `TETO_LOC = 377_937`, shell depois deste corte = **360 599** ⇒ folga **17 338**, e a metade de
   obsolescência dispara acima de **20 000**. ⇒ **este corte sozinho deixa-a a 2 662 linhas do vermelho**, e
   a `motion` (99 845) e a `vec` (22 104) entram na mesma rodada. A ordem é **integrar → `cargo fmt --all`
   → medir → escrever**.
2. ⚠️ **QUATRO avisos de clippy que NÃO são desta linha** (`git diff main...HEAD` vazio nos quatro), e o
   `ship.sh` corre clippy com `-D warnings`:
   - `crates/ph2d-app-sculpt3d/src/keys.rs:33` — *too many arguments (8/7)* ← já no `ESTADO_W2` §6
   - `shells/desktop/src/sculpt_source/mod.rs:368` — *needless borrow* ← já no `ESTADO_W2` §6
   - ⚠️ `shells/desktop/src/skeleton_goal.rs:46` — *empty line after doc comment* ← **NÃO está na lista**
   - ⚠️ `crates/ph2d-preview-drive/src/lib.rs:493` — *`len` sem `is_empty`* ← **NÃO está na lista**
3. ⚠️ **Ficheiros de costura que eu toquei** (regra 9 — conflito em região diferente é legítimo):
   `app_state.rs` (só o tipo de um campo), `main.rs`, `input_dispatch.rs` + `input_dispatch/keyboard*.rs`,
   `render_loop/mod.rs`, `render_loop/snapshots.rs`, `shells/desktop/Cargo.toml` (um bloco novo em
   `[dev-dependencies]`), `Cargo.lock`.
   ⚠️ **E `crates/ph2d-editor-core/src/{lib.rs, gizmo/mod.rs, gizmo/drag.rs}`** — três edições
   **append-only** (uma struct nova e o nome dela em duas listas de re-export). É foundational sob
   ADR-0107, e foi desenhado para fundir: nada existente mudou de forma.
   ⚠️ **E UMA linha do `CLAUDE.md`** (o link do `player_input.rs` na secção Vector), porque o gate
   `instructional_docs_only_cite_paths_that_exist` reprovava com o meu corte.
4. **Símbolos novos que podem colidir:**
   ```
   ph2d_editor_core::gizmo::drag::GroupDragSnapshot   (struct movida do app_state.rs)
   ph2d_app_physics::CanvasCtx                        (struct nova)
   ph2d_app_physics::overlay::point_gizmo             (módulo, vindo do render_loop)
   ph2d_app_physics::bridge::dispatch                 (módulo; o ficheiro colidia com a pasta)
   ph2d_app_physics::inspector::body                  (módulo)
   crate::physics::joint_gestures_app                 (ficheiro novo, shells/desktop)
   feature "test-support" em ph2d-app-physics         (2 itens)
   ```
5. **Os dois vermelhos do portão que NÃO são desta linha**, confirmados pelas três assinaturas do §5.0
   (3/3 verdes sozinhos com o `loadavg` ao lado — 6,42 · 9,16 · 8,50 —, **zero linhas** do diff naquelas
   crates, e **nomeados verbatim** no `CLAUDE.md`): `the_cost_of_depth_is_linear_not_explosive`
   (`ph2d-timeline`) e `the_mask_stroke_cost_does_not_follow_the_canvas` (`ph2d-tool-painter`).
   ⭐ **Na 2.ª corrida do portão os dois passaram** — 16 245/16 245.

---

## §8 · A UMA LINHA para o `CLAUDE.md` §5 (módulo **Física**)

> ⭐⭐⭐ **E a FASE C FECHOU O CORTE (12/09):** os 48 ficheiros que sobravam em
> `shells/desktop/src/physics/` vivem em [`ph2d-app-physics`](crates/ph2d-app-physics/) (mais o
> `render_loop/point_gizmo{,_tests}.rs`, que era **física com nome genérico** — ele já importava
> `ph2d_app_physics::overlay::joint_glyphs`), e a shell desce de **373 937 para 360 599** linhas com
> `ONLY-A = 0`, `ONLY-B = 0` e 169 `MOVED`. ⭐ **As 15 625 linhas estavam presas por SEIS SÍMBOLOS**, e
> cinco só eram tocados por testes: duas curaram-se (o `GroupDragSnapshot` foi para o `ph2d-editor-core`,
> onde o `TransformSnapshot` dele já vivia) e **quatro ficam por DESENHO** — o `build_component_registry`
> regista os componentes de cinco crates irmãs, logo é **composição**, e os gates que atravessam essa porta
> moram com o que exercitam (HOWTO §2.6). ⛔ **Zero sextos métodos no `AppHost`**: escritos em tipos, os
> quatro `impl App` pediam coisas que a `App` por acaso segurava, e ⭐⭐ **o `std::mem::take` do
> `resolve_player_input` DESAPARECEU** — *o truque não era lei do domínio, era o preço de a função estar na
> struct errada*. ⚠️⚠️ **E o `cargo check --all-targets` estava VERDE com 15 gates vermelhos**: eles vivem
> em `shells/desktop/tests/it/`, que o `nextest-impacted` **filtra** — entre eles o gémeo mudo do §2.6 que
> **sobreviveu dentro da crate** (curado com `include_str!`, que falha a compilar) e a QUARTA agulha deste
> repo a nomear a **visibilidade** em vez da lei.
> [Handoff da Fase C](docs/Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_C_2026-09-12.md)
> (⚠️ o §5 tem as **oito** armadilhas e o §6 as **quatro** premissas minhas que a medição derrubou — entre
> elas uma acusação de cegueira à régua do fecho que **era falsa**).

---

## §9 · O que fica ABERTO

1. ⏳ **A folha «anexar um componente pelo nome canónico»** — `component_attach::attach_by_name` +
   `component_seed`. Ela liberta os **9** gates que hoje ficam na shell por atravessarem a porta de
   produção. ⛔ O `init::build_component_registry` **não** vai com ela (é o censo do app). Isto é uma linha
   própria, no molde da `line/shell-folhas` — **não** é trabalho da `physics`.
2. ⏳ **O `bridge_tests.rs`** fica por medir o `ProjectState`; libertá-lo pede que a unidade de undo da
   shell seja endereçável de fora, que é decisão de arquitectura e não desta wave.
3. ⚠️ **A régua do fecho não sabe dizer «este ficheiro é a COSTURA»** (§6.2). Uma `--costura <ficheiro>`,
   irmã do `--preso`, resolveria; a ferramenta vive na worktree da `line/app-motion` e **eu não a editei**.
4. ⚠️ **Dívida nomeada da Fase B que continua de pé:** nenhuma das portas do `AppHost` lê `gfx.sculpt3d`
   (armadilha §2.10) — não é desta família, mas o `CanvasCtx` que esta fase criou é o primeiro
   agregado deste repo a **empacotar seis campos do `AppGfx`**, e um sexto consumidor dele deve ser lido
   com essa armadilha na mão.

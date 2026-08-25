# HANDOFF de INTEGRAÇÃO — `line/Vector` → `main` (2026-08-24) — **O INPUT MAP**

> **Entregável de fecho de linha** (DIRETRIZ §1.5.9). A linha **fechou e PAROU**: não integra, não
> faz ship, não pusha ([`CLAUDE.md §0.7`](../../../CLAUDE.md)). Smoke aprovado pelo Enio em
> 2026-08-24 (*"SMOKE ok"*), depois de **três** rondas de report com foto.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/Vector` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| HEAD | **este commit** (o do handoff). ⚠️ O **código** termina em `d7ae6c0aa`; o 15.º commit é só este documento e o índice da pasta. |
| merge-base com `main` | `5038249c6` |
| commits | **15** (14 de código + este handoff) |
| ficheiros | **71** (`+5 720 / −354`) |
| gate batched | **5 446/5 446 verdes**, 257 `skipped`, clippy `--all-targets` **0** (`ph2d-editor-core` · `ph2d-host-desktop` · `ph2d-i18n` · `ph2d-input`) |

### Os 14 commits, em ordem

```
149e5378d docs(reabertura)  a linha reabre e a PRIMEIRA coisa medida foi a NOTA
86bafc8fc docs(fila)        as duas features de 25/08 entram na fila
ab5a1e42d docs(plano)       a pesquisa profunda, e a ORDEM inverte-se
5590ea73f feat(input)  W1   accoes NOMEADAS sobre a crate que JA' EXISTIA + o teclado
6a77af598 feat(input)  W2   o mapa atravessa o ficheiro  (PROJECT_SCHEMA 95 -> 96)
b3dd71f40 feat(input)  W3a  a JANELA FLUTUANTE abre sobre o canvas, do menu Settings
40c623aeb feat(input)  W3b  o press-to-bind FECHA a sequencia
477433f40 feat(input)  W5   o JOGADOR le' o mapa; o `PlayerKeys` cravado morre
7775e2257 feat(input)       a janela ARRASTA e a escuta apanha BOTAO DE COMANDO
1a6254a4a feat(input)  W6   a haste, o botao e os DOIS NUMEROS DA ZONA
fb105812b fix(input)        os DOIS reports do Enio (widgets da casa + guarda do Bind)
838a3f557 fix(input)        «estreito e sem scroll»
f6828ccce fix(input)        a AUDITORIA MULTIAGENTICA (25 achados, 15 defeitos)
d7ae6c0aa fix(input)        os TRES reports com foto (campo em cima · foco · labels)
```

⚠️ **A ordem importa em dois pontos e só dois:** `5590ea73f` (a crate) antes de tudo, e `6a77af598`
(o schema) antes de qualquer coisa que salve. O resto é aditivo.

---

## 2. Foundational / partilhado tocado, e **por quê**

### 2.1 `crates/ph2d-input/` — a crate **JÁ EXISTIA** ⚠️

⛔⛔ **A primeira leitura desta linha CRIOU-A do zero, e ela existia desde a M8** (`7329d63d5`). O
`Write` respondeu *"updated"* e não *"created"* — o aviso documentado — e eu li por cima. A árvore
foi reposta por `git checkout` e a wave reconstruída **por cima** do que lá estava.

| ficheiro | o quê |
|---|---|
| `keyboard.rs` **(novo)** | `Key(u32)` · `KeyboardState` (Vec ordenado, ⛔ nunca `HashMap`) · `Key::label()` — `"Left Arrow"`, `"Space"`, F1–F12, `[ ] , . Home` |
| `action.rs` **(novo)** | `Binding{Key,PadButton,PadAxis}` · `ActionId(u32)` · `InputAction` · `set_zone` (coerção `press_point >= dead_zone`) |
| `map.rs` **(novo)** | `InputMap` + `with_player_defaults()` + as `PLAYER_*` |
| `resolve.rs` **(novo)** | `Sample` · `ActionState` · `Input<'a>` (`pressed`/`just_pressed`/`just_released`/`strength`/`axis`) |
| `gamepad.rs` · `event.rs` · `state.rs` | **aditivo**: derives `serde`, `::ALL` + `label()`, `Event::{KeyDown,KeyUp,FocusLost}`, `InputState.keyboard` |

⚠️ **Dep nova: `serde` no `ph2d-input`** — a allowlist da crate é de **UM**, e
`tests/the_input_map_stays_a_leaf.rs` **lê o `Cargo.toml`**: qualquer dep a mais fica vermelha antes
de alguém a usar. A proibição escrita naquele ficheiro era de dependência de **plataforma**
(gilrs/winit/evdev) — o `serde` é portável e serve `no_std`. **Não há pacote externo novo no
`Cargo.lock`** (o `serde` já é da workspace); a única aresta nova é interna.

### 2.2 `crates/ph2d-editor-core/` — **dep nova `ph2d-input`**

`HeroScreen::input_map: ph2d_input::InputMap` (o holder do editor). ⚠️ **Tem de estar ao alcance do
pintor**: `paint_hero_screen` só recebe o `HeroScreen`. Mesmo precedente de `HeroScreen::project`.

| ficheiro | aditivo? | o quê |
|---|---|---|
| `screens/hero.rs` | **+campo** | `input_map`, semeado com `with_player_defaults()` |
| `screens/hero/chrome/input_map.rs` + `input_map/apply.rs` + `input_map/layout.rs` | **novos** | a janela (pintor · despacho · layout) |
| `screens/hero/chrome/mod.rs` | aditivo | `mod` + `pub use` + o elo no `dispatch_all` (**gerado** pelo `ph2d-chrome-sync`, z=181) |
| `screens/hero/paint.rs` | aditivo | uma chamada, entre o menu de contexto e o `onion_modal` |
| `screens/hero/pre_populate.rs` | **+1 linha numa LISTA** | `CTX_MENU_SETTINGS_INPUT_MAP` no `populate_global_context_menu` |
| `screens/hero/menu_rows.rs` | **+1 linha numa LISTA ORDENADA** | `"Input Map…"` no fim do `SettingsMenu` |
| `interaction/state/input_map_ops.rs` | **novo** | o estado transiente + `capture_if_listening` |
| `interaction/state/mod.rs` | **+4 campos** no `WidgetStore` | `input_map_window` · `input_map_listening` · `input_map_captured` · `input_map_scroll` |
| `interaction/state/store_core.rs` | **+4 linhas** no construtor | idem |
| `interaction/mod.rs` | aditivo | `pub use … capture_if_listening` |
| `interaction/dispatch/key.rs` | **+1 ramo NO TOPO** | ⚠️ ver §3 |
| `widget/scrollbar.rs` + `widget/mod.rs` | **+1 id no registo da casa** | ⚠️ `NodeId(842)` — ver §3 |
| `ids/chrome/input_map.rs` **(novo)** + `ids/chrome/mod.rs` + `ids/menus.rs` | aditivo | os ids |
| `Cargo.toml` | **+1 dep interna** | `ph2d-input` |

### 2.3 `shells/desktop/` — **a persistência e a cadeia de teclado**

| ficheiro | Δ | o quê |
|---|---|---|
| `project.rs` · `project_save.rs` · `project_load.rs` | +58 | o mapa entra no `ProjectFile`; o load **INSTALA, nunca funde**, e zera o `ActionState` |
| `project_schema.rs` + `project_schema_tests.rs` | +21 | ⚠️⚠️ **`PROJECT_SCHEMA` 95 → 96** — ver §3 |
| `keymap.rs` | +161 | `winit_to_input_keycode` — normalizador **TOTAL** (o do editor deixa cair de propósito) |
| `input_dispatch.rs` + `input_dispatch/keyboard.rs` + `keyboard_bind_capture.rs` **(novo)** | +92 | ⚠️ ver §3 |
| `player_input.rs` | +52/−318 | ⭐ o `PlayerKeys` **cravado morreu**; o jogador resolve o mapa |
| `app_state.rs` | +14 | `input_actions: ActionState` (derivado) |
| `input_map_drag.rs` **(novo)** | +164 | arrasto da janela · escuta de comando · roda |
| `render_loop/mod.rs` · `main.rs` · `input_log.rs` | +31 | os elos de quadro |

---

## 3. Símbolos que podem COLIDIR — **leia esta secção inteira**

### 3.1 `bash scripts/collision-surface.sh` (colado, **não escrito de memória**)

```
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 5038249c6   ·   14 commit(s)   ·   71 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                         96   (base: 95)
  ⚠   └ tripla do gate               (96, 13, 14)   (base: (95, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               69   (base: 69)
    ph2d-render (espelho)                  70   (base: 70)
    ph2d-script (espelho)                  70   (base: 70)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0163   próximo livre: 0164
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3)
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
```

> ⚠️ **PRAZO DE VALIDADE (§1.5.9 item 3).** Isto mede a linha contra o `main` de **2026-08-24**.
> **RE-RODE `collision-surface.sh` nesta worktree imediatamente antes de fundir** — se a coluna
> `base` divergir, a divergência é ela própria um achado e aponta para a linha que integrou no meio.

### 3.2 ⚠️⚠️ `PROJECT_SCHEMA` **95 → 96** — o número que **SOMA**, e são **TRÊS** sítios

Se outra linha desta jornada também subiu o schema, **o valor certo não está em nenhum dos dois
lados do conflito: CONTA-SE.** E ⛔ **a colisão passa MUDA quando as duas escrevem o mesmo literal.**

| sítio | ficheiro |
|---|---|
| a constante | `shells/desktop/src/project_schema.rs` |
| o degrau da escada | `shells/desktop/src/project_schema.rs` (⚠️ um degrau escrito no `project.rs` funde **limpo** e **evapora**) |
| a tripla do gate | `shells/desktop/src/project_schema_tests.rs` — `(96, 13, 14)` |

⚠️ **O `ProjectFile` é postcard, que é POSICIONAL:** o campo `input_map` foi apendado **no fim**.
Se outra linha também apendou um campo, a ordem tem de ser reconciliada **antes** de correr o gate
de ida-e-volta, senão os dois campos trocam de valor em silêncio.

### 3.3 `INPUT_MAP_SCROLLBAR_ID = NodeId(842)` — **id literal no registo da casa**

`crates/ph2d-editor-core/src/widget/scrollbar.rs`. ⚠️ **Há um censo de colisão a defender esse
registo** — se outra linha tomou o `842`, o certo é o **próximo livre**, e o censo diz qual. O
`pub use` em `widget/mod.rs` é uma **lista ordenada** e vai conflitar textualmente: é resíduo, funde
por ordem alfabética.

### 3.4 Os outros ids — **todos derivados de string, fora da disputa**

`INPUT_MAP_{HANDLE,CLOSE,NEW_NAME,ADD,SURFACE,BIND_CAPTURED,LISTEN_CANCELLED}` e
`CTX_MENU_SETTINGS_INPUT_MAP` são `hash_node_id("…")`; as sete famílias por-linha
(`input_map_listen_id(row)` etc.) são funções. **Nenhum número escolhido à mão.**

### 3.5 Duas **listas ordenadas** que conflitam textualmente (resíduo, não semântica)

* `menu_rows.rs` — `"Input Map…"` no fim do `SettingsMenu`;
* `pre_populate.rs` — `CTX_MENU_SETTINGS_INPUT_MAP` no `populate_global_context_menu`.

⛔ **Se uma das duas se perder no merge, o item de menu fica MORTO SOB O PONTEIRO** e todos os
gates continuam verdes. Confira as **duas**.

### 3.6 ⚠️ **`crates/ph2d-editor-core/src/interaction/dispatch/key.rs` — um ramo NO TOPO**

`capture_if_listening` entra **antes** de tudo, porque enquanto a escuta está armada **a próxima
tecla é conteúdo, não atalho**. Uma linha que também tenha inserido um ramo no topo desse ficheiro
colide **por posição**, e a ordem entre os dois é uma decisão, não um merge.

### 3.7 ⚠️ **`shells/desktop/src/input_dispatch.rs` — o topo do `key_input`**

`self.capture_binding_if_listening(...)` é o **primeiro** ramo. ⛔ **Estar em `dispatch_key` NÃO
BASTA** — foi report do Enio (*"os atalhos de editor estão em conflito com o Bind"*): a shell tem
**~20 `return`** antes de chegar ao editor-core (`P` radial, `W` painel de mundo, Espaço transporte,
peek do Flip). **Uma lei, dois chamadores.** Se este ramo escorregar para baixo de qualquer outro,
ligar `S` **salva o projecto**.

### 3.8 ⛔⛔ **`CLAUDE.md` — esta linha EDITOU o §5, e a DIRETRIZ diz que isso é da integração**

Regra (DIRETRIZ §1.5.6, tabela): *`CLAUDE.md §5` só na integração, no primário, uma linha de
trabalho por vez.* Esta linha **quebrou-a** no commit de reabertura `149e5378d` — está declarado
aqui em vez de escondido. São **duas hunks**, ambas no bullet **Vector**:

1. a linha **`Aberto:`** ganhou **a FILA em ordem** (1 Input Map · 2 Morph SM · 3 Texture pattern) e
   ✅ corrigiu a nota do *tether*/`DRAG_RATE_X = 50` (que dizia *"feel sem medição"* e era falsa);
2. o bullet da **D2 (partículas)** — o estudo mentia sobre **NOVE** das próprias linhas, e a D2 está
   **mispreçada nos dois sentidos**.

⚠️ **`CLAUDE.md` é o ficheiro nº 1 de conflito entre linhas.** Se outra linha o tocou, funda por
assunto: as duas hunks acima são **só do bullet Vector**.

**A linha que a integração deve escrever no §5** (item 8 — *uma linha, nunca um parágrafo*):
substituir o `**(1)** ⭐ **O INPUT MAP** …` do bullet Vector por

> ✅ **(1) O INPUT MAP FECHOU** ([plano 30](docs/Vector%20Module/30_plano_input_map.md) W1–W7,
> [handoff](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_input_map_2026-08-24.md)) —
> acções **nomeadas** à la Godot, janela flutuante em *Settings > Input Map…*, *press-to-bind* de
> tecla **e** de comando, e o mapa viaja no `.ph2dproj` (`PROJECT_SCHEMA` 96). ⭐ **A falha
> documentada do Godot está corrigida:** a `deadzone` dele tem **dois papéis** (ponto de disparo *e*
> offset de normalização — proposta #3709); aqui são **dois números** (`dead_zone` · `press_point`),
> os dois lêem o valor **CRU**, e `press_point >= dead_zone` é **coagido na porta**. ⛔ **LEI Nº 1
> honrada:** a `InputTape` grava a **acção resolvida**, nunca a tecla — remapear não reescreve o
> passado nem parte o `physics_ecs_c9`. ⚠️ **Faltam os CONTEXTOS com prioridade** (o que o Unreal
> tem e o Godot não): **bloqueado** — só têm sentido com um modo de jogo, e o `shells/game`/R1 está
> adiado pelo Enio. ⏳ Falta também o *override* por-jogador em `~/.ph2d/`.

---

## 4. Contratos congelados (§6)

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` **intocados** (confirmado pelo `collision-surface.sh`). **Nenhum ADR criado** ⇒ fora da
disputa do `0164`.

---

## 5. O que só o `ship.sh` apanha (o gate de integração **não** roda)

* **`typos`** — os doc-comments desta linha são densos e em PT; nunca correram sob o `typos`.
* **`cargo machete`** — a dep nova `ph2d-input` no `ph2d-editor-core` **é** usada (`HeroScreen`), mas
  o `machete` só o confirma no ship.
* **`cargo deny` / `RUSTSEC`** — `serde` já estava na workspace; sem risco esperado.
* **`clippy --all-targets --all-features`** — aqui correu **sem** `--all-features`.
* **fmt pré-fork** — `cargo fmt -p ph2d-editor-core -p ph2d-i18n` correu nesta linha e **normalizou
  três ficheiros que já estavam por formatar** (`widget/mod.rs`, `chrome/mod.rs` e o gate
  `the_input_map_window_binds_a_key.rs`) — todos desta linha, mas o diff pode surpreender.

---

## 6. Ordem, dependências, e **o que smokar**

### Ordem
`5590ea73f` (crate) → `6a77af598` (schema) → o resto. Fora isso, aditivo.

### Smoke **aprovado pelo Enio** (2026-08-24)

```
cargo run -p ph2d-host-desktop --release
```

1. Botão direito no canvas → **Settings** → **Input Map…**
2. A janela abre no canto superior esquerdo. **Arraste-a pela faixa do título.**
3. Escreva um nome no campo **do topo** e carregue **Add** (ou **Enter**).
4. Carregue no **+** da linha: o título da janela passa a dizer, em rosa,
   `Listening for <nome> — Press a key or a gamepad button. Esc cancels.`
5. Carregue numa tecla (ou num botão/haste do comando). A ligação aparece com o **nome legível**
   (`Left Arrow`, `A / Cross`, `Left Stick X (+)`), nunca um código.
6. `Ctrl+S` / `Ctrl+O`: o mapa **sobrevive** ao ficheiro.
7. As setas / `A`·`D`·`Z` continuam a mover o jogador — porque ele **lê o mapa**, não teclas cravadas.

### ⛔ O que **NÃO** foi smokado

* **`Save As…` com um mapa grande** — a ida-e-volta está gateada, mas nunca passou por um ficheiro real grande.
* **Um `.ph2dproj` de schema 95** (pré-linha) a abrir no 96 — o degrau existe e tem gate, mas nunca correu sobre um ficheiro do disco.
* **A janela numa viewport MUITO baixa** (`< 200 px`): o clamp e a rolagem têm gate, o olho não.
* **Comando de outra marca** que não o testado; o `gilrs` normaliza, mas o mapeamento de botões não foi conferido noutro dispositivo.
* ⚠️ **O chevron do menu:** o laço do `SettingsMenu` põe um `ChevronRight` em **cada** linha por ser esse o tipo de menu, e `Input Map…` **não tem submenu**. Está declarado no código; se o Enio o vir como errado, a cura é o laço perguntar pela LINHA, não pelo tipo.

---

## 7. Incremental reclamado

`rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental` — corrido no fecho
(§1.5.9 item 7). **20 GB** reclamados (`target/debug/incremental`); a worktree ficou em **28 GB**.

---

## 8. A NARRATIVA — o mecanismo que o §5 **não** recebe

### 8.1 As sete waves
O plano vivo é [`30_plano_input_map.md`](../30_plano_input_map.md) §7, com **W1..W7 marcadas ✅** e o
mecanismo de cada uma. A pesquisa (Godot · Unreal *Enhanced Input* · o caso de **abandono** do Unity)
está no §1 do mesmo doc.

### 8.2 As leis que esta linha pagou

1. ⛔⛔ **A crate `ph2d-input` JÁ EXISTIA e eu criei-a por cima.** O `Write` disse *"updated"*. É o
   **nono** incidente de nota envelhecida desta linha — e o primeiro que **eu** produzi.
2. ⭐ **A falha do Godot corrigida:** `deadzone` de duplo propósito → **dois números**, os dois a ler
   o valor **CRU**, coerção na porta.
3. ⭐ **`ActionId` é um CONTADOR estável** — não índice (sobrevive a reordenar), não hash do nome
   (sobrevive a renomear).
4. ⛔ **`TapeWire = (f32 drive, u8 bits)`** — semântico, nunca keycode. Remapear **não** reescreve
   gravações.
5. ⚠️ **`#[serde(skip)]` no `next_id` sobreviveu a TODAS as afirmações de ida-e-volta**, porque elas
   só olhavam para as acções que já existiam. A cura foi perguntar **pela porta pública**: criar na
   cópia recarregada e exigir que não haja colisão de id.
6. ⚠️ **Uma lei, dois chamadores** (§3.7) — a guarda do Bind estava no topo de um **pedaço** da
   cadeia.
7. ⚠️ **`slider_with_chip` EMPILHA o rótulo sob pressão** e o doc dele diz *"quem chama tem de
   avançar"*. Eu não avancei ⇒ *"estreito e sem scroll"*. A largura sai agora do **piso real do
   widget** (`ZONE_MIN_TRACK = 60`), com `debug_assert` e gate.
8. ⚠️ **`HitIndex::push_clip` recorta CLIQUES; `VectorScene::push_clip` recorta PIXELS.** São coisas
   diferentes e o comentário anterior confundia-as — o conteúdo rolado **desenhava** por cima do
   título.
9. ⛔⛔ **O achado mais grave da auditoria:** o cartão não tinha fundo registado ⇒ clicar no espaço
   vazio dentro da janela **pintava no canvas por baixo**.
10. ⛔⛔⛔ **A lei de processo desta linha:** dos **25 achados confirmados** pela auditoria
    multiagêntica, **nenhum dos meus doze gates olhava para o que foi DESENHADO** — todos mediam o
    mapa e o `WidgetStore`. É por isso que doze verdes conviviam com uma janela a desenhar por cima
    do próprio título, e por isso que os **três** reports com foto tiveram de vir do Enio.
11. ⚠️ **O quinto elo da costura é quem PINTA** — as quatro condições verdes e o campo lia-se morto,
    porque o pintor desenhava um rect à mão e **não lia** o `TextInputState::Focused` que o
    `pointer_down` já escrevia. Memória:
    [[feedback_the_fifth_seam_link_is_whoever_paints]].
12. ⚠️ **Uma sonda que SOMA dois sinais não diz qual dos dois falhou** — o gate de *"armar pinta um
    aviso"* media a tinta da janela inteira e **sobreviveu à mutação**, porque armar muda a tinta
    por duas razões independentes. A lei mudou-se para uma função pura (`title_text`) com gate
    próprio. Memória: [[feedback_a_probe_that_sums_two_signals_cannot_say_which_failed]].
13. ⚠️ **Uma função que se diz "a porta única" só o é quando o outro lado a CHAMA:** o doc da
    `input_map_window_size` dizia *"a mesma conta que o pintor faz"* e havia **48 px** de divergência
    entre o rectângulo que a roda testava e o desenhado.
14. ⚠️ **Cinco gates pré-existentes foram RE-ANCORADOS, nenhum apagado** — duas vezes por
    refactorações minhas (o corte de LOC moveu o endereço; a W5 semeou o mapa de fábrica e partiu
    gates que assumiam `row 0` e lista vazia).

### 8.3 O que fica ABERTO (e por quê)

| item | estado |
|---|---|
| **contextos com prioridade** (Unreal) | ⛔ **bloqueado** — só têm sentido com um modo de jogo; `shells/game`/R1 adiado pelo Enio |
| *override* por-jogador em `~/.ph2d/` | ⏳ não construído |
| colunas com cabeçalho · dobrar por acção · filtro por nome · *Show Built-in Actions* · ícone de dispositivo | ⏳ identificados na comparação com o Godot, **não** construídos |
| o chevron de `Input Map…` no menu | ⚠️ decisão de produto (§6) |

---

## 9. Resumo colável

> Linha `Vector` pronta (HEAD = o commit deste handoff; código até `d7ae6c0aa`, **15** commits, base `5038249c6`). **O INPUT MAP**, W1–W7,
> smoke aprovado. Foundational: `ph2d-input` (a crate **já existia** — reconstruído por cima; dep
> nova `serde`, allowlist de UM com gate a lê-la), `ph2d-editor-core` (+dep `ph2d-input`, +campo no
> `HeroScreen`, +4 campos no `WidgetStore`, **um ramo no topo do `dispatch_key`**), `shells/desktop`
> (persistência + **topo do `key_input`** + o `PlayerKeys` cravado **removido**).
> ⚠️⚠️ **`PROJECT_SCHEMA` 95 → 96 — CONTE, nos TRÊS sítios; o `ProjectFile` é postcard POSICIONAL e
> o campo foi apendado no fim.** ⚠️ **`NodeId(842)`** literal no registo de scrollbars. ⚠️ **Duas
> listas ordenadas** (`menu_rows` + `pre_populate`) — perder uma deixa o item **morto sob o
> ponteiro** com todos os gates verdes. ⛔ **`CLAUDE.md` §5 tocado** (duas hunks, só o bullet
> Vector — §3.8 tem a linha a escrever). Contratos congelados: **nenhum**. ADR: **nenhum**.
> Gate batched **5 446/5 446** · clippy **0**. **Aguardo ordem de integração.**

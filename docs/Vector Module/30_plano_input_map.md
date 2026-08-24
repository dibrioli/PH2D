# 30 — PLANO: o **Input Map** (o sistema de entradas nomeadas)

> Pedido do Enio em **2026-08-24**: *"Antes da máquina de estados, criaremos o sistema de Inputs
> (como o input Map do Godot). Aqui também faça pesquisa sobre os melhores existentes. Neste caso
> sei que o sistema da godot é muito bom e se houver dificuldade de definir um melhor, vamos usar a
> godot como referência. Primeiro o input map."*
>
> ⭐ **Ele vem primeiro por uma razão que a pesquisa confirmou:** as entradas de uma máquina de
> estados (Rive: *booleans · triggers · numbers*) **são** o que um input map produz. Construir a
> máquina antes teria obrigado a inventar uma fonte falsa para as condições dela.

---

## §0 — ⭐⭐ EMENDA DE ESCOPO (Enio, 2026-08-24, depois da W1)

> *"precisamos do input Map **completo** não apenas para o jogador mas para **qualquer objeto do
> game via UI**. (…) Nosso input map deve ser **equivalente ao da godot com janela flutuante
> abrindo sobre o canvas**."*

**Três consequências, e elas reordenam o plano:**

### 0.1 O mapa **não é do jogador** — é do PROJECTO

O plano original tratava o `player_input.rs` como o consumidor. Ele passa a ser **um** consumidor
entre N: qualquer objecto do jogo lê acções pelo nome, e a máquina de estados do Morph (doc 31) é
o segundo consumidor já nomeado.

⭐ **A W1 já nasceu certa para isto** — o `InputMap` é uma folha, sem uma linha sobre jogador, e o
`Input::pressed("jump")` não sabe quem pergunta. *O que muda não é o modelo; é a ordem em que o
resto se constrói.*

### 0.2 ⭐ A UI é uma **JANELA FLUTUANTE sobre o canvas**, à la Godot

Não um painel na doca. O precedente exacto existe e é um caminho **scaffoldado** (DIRETRIZ §3.B.3):
um *chrome handler* com marcador `z=NN`, metade de **pintura** e metade de **despacho**, geradas no
`dispatch_all` pelo `ph2d-chrome-sync`. O irmão mais próximo é o
[`fill_modal.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/fill_modal.rs) — cartão
arrastável pela faixa do título, preso à viewport — e o
[`command_palette.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/command_palette.rs)
para a parte de **lista**.

⚠️ **Medido 2026-08-24:** o espaço de `z` tem 36 marcadores tomados e folgas (`181..184`,
`186..189`, `195`). O número escolhe-se **contando**, nunca de memória — é a mesma família dos
números que somam entre linhas.

### 0.3 ⛔ E isso **promove a UI e a persistência** à frente do jogador

A ordem antiga fazia a fita do jogador (W2) vir antes do painel. Com o escopo novo isso está
errado: uma janela que o artista abre e que **não grava** é uma mentira, e ligar o jogador antes de
existir UI deixaria o mapa inalcançável para todos os outros objectos. Ver **§7 reordenado**.

---

## §1 — Pesquisa: o estado da arte, e o que foi TENTADO E ABANDONADO

### 1.1 Godot — *Input Map* (a referência que o Enio nomeou)

| Peça | Como é |
|---|---|
| **Ação** | um **nome** (`"jump"`, `"ui_left"`) agrupando **zero ou mais** `InputEvent`. O código lê o nome, nunca a tecla |
| **Força** | `get_action_strength()` devolve **0..1**. Tecla dá `0` ou `1`; eixo analógico dá o intermédio, medido **a partir da deadzone** |
| **Eixo** | ⭐ **não existe eixo — existem DUAS ações.** `−1..1` sai de `strength("right") − strength("left")`. É menos explícito, e a documentação assume isso: *"pode confundir ao início, mas tem várias vantagens"* |
| **Deadzone** | por ação, no mapa |
| **Remap em runtime** | o singleton `InputMap` cria/reatribui ações **sem gravar** — é assim que um menu de controlos funciona |
| **Injecção sintética** | `InputEventAction` + `Input.parse_input_event()` — código pode **fingir** uma ação |
| **Propagação** | oito estágios em ordem (`_input` → GUI → `_shortcut_input` → `_unhandled_key_input` → `_unhandled_input` → picking), e o primeiro que consome **para** |

⛔ **A falha documentada, e vamos corrigi-la:** a `deadzone` é **de duplo propósito** — serve ao
mesmo tempo de *ponto de disparo* do `pressed` **e** de *offset* da normalização da força. A
proposta aberta [godot-proposals#3709](https://github.com/godotengine/godot-proposals/issues/3709)
pede exactamente separá-los, e o fórum oficial tem a pergunta *"How is the Deadzone from Input Map
supposed to work?"* repetida. ⇒ **um número que responde a duas perguntas é a assinatura de que
faltam dois números** — a mesma lei que este repo já pagou.

### 1.2 Unreal — *Enhanced Input* (o que Godot não tem)

Quatro conceitos: **Input Action** · **Input Mapping Context** · **Modifiers** · **Triggers**.

- ⭐⭐ **`Input Mapping Context` com PRIORIDADE** é a peça que Godot não tem e que **este app
  precisa**: contextos são adicionados/removidos por jogador, e a prioridade **resolve o conflito
  quando a mesma tecla dispara duas acções**.
- **Modifiers** = pré-processadores do valor (deadzone, suavização, negação, swap de eixos) — uma
  **pipeline**, não um campo.
- **Triggers** = decidem se o valor, já modificado, *arma* a acção (`Pressed`, `Hold`, `Tap`,
  `Chord`, `Combo`).
- Uma `InputAction` reporta até **três** eixos, independentemente do que a disparou.
- ⛔ Substituiu o sistema de *axis/action mappings* legado da UE4, que foi **deprecado** — é o
  precedente de um input system inteiro ser abandonado por não ter contextos nem pipeline.

### 1.3 Unity — o caso de ABANDONO, e a lição é sobre CUSTO, não sobre poder

O *Input System* novo é objectivamente mais capaz que o *Input Manager* antigo (baseado em
strings) — e a comunidade rejeita-o em massa: *"New Input System is a MESS"*, *"so Convoluted?"*,
*"mais confuso, mais código para o mesmo resultado, mais tempo a montar"*, relatos de instabilidade
até 2025. Programadores descrevem gastar horas para conseguir um clique de rato.

⭐ **A lição, e ela decide o nosso desenho:** *um sistema de input cujo **custo de montagem** excede
o do que substitui perde a adopção mesmo sendo mais poderoso.* O `Input.GetAxis("Horizontal")` do
sistema antigo ganhava por **uma linha**. ⇒ o nosso caminho comum tem de ser **uma linha**.

### 1.4 O que a indústria mede sobre *game feel* (e que NÃO pertence ao mapa)

**Coyote time** (janela de graça depois de sair da borda) e **input buffer** (o pulo pedido cedo
demais é honrado) são *"as duas mecânicas de perdão mais impactantes"*. Valores medidos:
plataforma de precisão **70-100 ms / 70-110 ms**; plataforma de acção **90-140 / 100-150**; casual
**110-170 / 120-180**. **Celeste = 5 quadros.**

⛔ **Isto NÃO entra no input map** — é lei do controlador, e o
[`ph2d-platformer`](../../crates/ph2d-platformer/) **já deriva as bordas sozinho** de propósito
(*"quem a derivasse do lado de fora precisaria de uma segunda memória do mesmo fato"*). Fica
registado aqui para **não** ser reconstruído no sítio errado.

### 1.5 ⭐ A escolha, derivada dos cinco princípios

**Forma do Godot + contextos com prioridade do Unreal.** Porquê, princípio a princípio:

| Princípio | O que ele escolhe |
|---|---|
| **Fácil de usar** | Godot: ler uma acção é **uma linha**; o eixo sai de duas acções sem tipo novo. Rejeita a árvore de assets do Unity (medido: é o que mata a adopção) |
| **Intuitivo para artistas** | uma tabela de **nomes** e as teclas ao lado — o Enio nunca vê um `KeyCode` |
| **Poderoso** | contextos com prioridade (Unreal), sem os quais o **editor e o jogo brigam pela mesma tecla** — e neste app isso **já é facto medido** (§2.4) |
| **Estado da arte** | modifiers/triggers do Unreal ficam como **ponto de extensão nomeado**, não na v1: a v1 entrega deadzone + as bordas, que é o que o platformer consome hoje |
| **Padrão-ouro** | corrige a falha conhecida da referência: **deadzone e ponto de disparo passam a ser DOIS números** (§1.1) |

---

## §2 — O desenho, com a PORTA ÚNICA de cada pergunta

### 2.0 ⛔⛔⛔ EMENDA — **a crate `ph2d-input` JÁ EXISTIA**, e este plano não a tinha visto

**Medido em 2026-08-24, ao começar a W1, e por não ter medido antes:** `crates/ph2d-input/` existe
desde o commit `7329d63d5` (*"M8: ph2d-input (gamepad via gilrs + Pencil stub) + ph2d.input Luau
binding + first prod bench in CI"*). O plano acima foi escrito a falar de uma crate a criar.

| O que a M8 já tinha | Consequência para este plano |
|---|---|
| **folha pura, zero deps** (só `criterion` em dev), *"no gilrs / IOKit / evdev / winit"* | ⭐ a disciplina de folha que a §7 W1 mandava construir **já estava lá**, escrita e justificada |
| `GamepadButton` (**34** variantes) · `GamepadAxis` (6) · `GamepadState` com `pressed`/`held`/`released`/`axis`/`iter_held`/`iter_axes` | ⛔ o `Devices` que este plano desenhava era **um duplicado** — uma segunda fotografia dos mesmos botões |
| `Event` + `InputState::{begin_frame, apply_event}` | ⭐ o protocolo de quadro **já existe**, e o `ph2d.input` do Luau já o lê |
| **consumidores vivos**: `ph2d-script`, `ph2d-render`, `ph2d-editor-core/zen.rs`, `shells/desktop` (`gilrs_adapter`, `input_log`, `integration`, `main`), **e um bench no CI** | qualquer mudança de superfície é paga por sete sítios |
| ⛔ **NÃO tinha teclado** | é o buraco real, e a maior parte das ligações que um artista autora é uma tecla |

⭐⭐ **A leitura certa, e ela melhora o desenho:** o que existia é a camada **`InputEvent`** do
Godot (dispositivo cru); o que o Enio pediu é a camada **`InputMap`** (acções nomeadas) **por
cima**. As duas são a mesma divisão que a referência faz — ⇒ as acções entram **na mesma crate**,
lendo o `InputState` que já lá está, e não ao lado dele.

⇒ **A §2.2 e a §2.3 abaixo ficam válidas na forma; o que muda é de onde vem o valor cru.** O
`Devices` deste plano **não foi construído**: quem responde *"este botão está em baixo?"* continua a
ser o `InputState` da M8, agora com um `KeyboardState` irmão do `GamepadState`.

⚠️ **A lição, e ela é a nona da mesma família nesta linha:** o
[estudo §6.6](Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md) manda **medir um item
antes de o pegar**, e isto foi escrito sem um `ls crates/ | grep input`. O sinal existia e passou
despercebido — a ferramenta de escrita respondeu **"updated"** em vez de **"created"** ao gravar o
`Cargo.toml`, que é exactamente o que este repo tem registado como o aviso de que o ficheiro já
existia. *Nada se perdeu (o git restaurou os dois ficheiros sobrescritos), mas o plano descreveu por
uma hora um terreno vazio que não estava vazio.*

### 2.1 ⛔⛔ O que JÁ existe aqui, e que decide quase tudo (medido 2026-08-24)

**Não estamos a construir num terreno vazio.** A cadeia de hoje:

```
  winit KeyEvent
     |
     v
  PlayerKeys            shells/desktop/src/player_input.rs   -- 6 bools, TECLAS CRAVADAS
     |                    (Arrow/WASD, Z=pulo, Q=arranque, R=agarrar)
     v
  PlayerInput           ph2d-platformer/src/sense.rs         -- 5 campos FIXOS
     |                    drive: f32 · jump · down · dash · grab
     v
  InputTape             ph2d-physics-ecs/src/bridge/tape.rs  -- UMA ENTRADA POR TICK
     |
     v
  drive_players()  <- corre nos DOIS lacos: play E replay
```

⭐⭐⭐ **A `InputTape` é o facto mais importante deste plano.** Ela existe porque, sem ela, *"a
trajetória de um scrub e a de um play discordavam sobre o mesmo tick"*. Ela faz o jogador ser
função de `(tick, fita)`, e é isso que o **`physics_ecs_c9`** prova com um **hash de replay que
roda na matriz de 3 OS no CI**.

⇒ ⛔ **LEI Nº 1 DESTE PLANO: a fita grava a AÇÃO RESOLVIDA, nunca a tecla crua.**
Se a fita gravasse teclas, remapear `jump` de `Z` para `Espaço` **mudaria o significado de toda
gravação anterior**, e o hash de replay partiria por uma edição de preferências. Gravar a acção
torna o remap **invisível** ao replay, que é o comportamento certo *e* o que mantém o gate verde.
*Um mapa que se mete entre o dedo e a fita tem de se meter **antes** dela.*

### 2.2 O modelo (a porta única de "o que é uma entrada?")

```rust
/// Uma AÇÃO: o nome que o jogo lê, e as teclas/botões que a produzem.
pub struct InputAction {
    pub name: ActionName,            // "jump", "move_left" — o que o codigo le'
    pub bindings: Vec<Binding>,      // N ligacoes; ZERO e' valido (acao declarada, por atribuir)
    /// ⭐ DOIS numeros, e nao o de duplo proposito do Godot (§1.1):
    pub dead_zone: f32,              // abaixo disto o valor e' ZERO (ruido do analogico)
    pub press_point: f32,            // acima disto `pressed` e' true (o gatilho)
}

/// O que produz o valor de uma acao.
pub enum Binding {
    Key(KeyCode),
    MouseButton(MouseButton),
    PadButton(PadButton),
    PadAxis { axis: PadAxis, positive: bool },   // meia-haste: a outra metade e' OUTRA acao
}
```

⚠️ **`Vec<Binding>` e não um par fixo:** teclado + gamepad + a segunda tecla do jogador canhoto são
**a mesma acção**. É o que torna o código agnóstico ao dispositivo, e é o ponto inteiro do Godot.

### 2.3 A leitura — **uma linha**, e é a porta única

```rust
input.pressed("jump")            // bool  — esta' segurada AGORA
input.just_pressed("jump")       // bool  — a BORDA deste tique
input.strength("jump")           // 0..1  — tecla da' 0/1; analogico da' o intermedio
input.axis("move_left", "move_right")   // -1..1, a subtracao do Godot
```

⛔ **Não haverá um `Axis` de primeira classe.** É a decisão do Godot e ela ganha aqui pelo mesmo
motivo: um eixo de primeira classe obriga a decidir **agora** o que fazer quando as duas metades
são dispositivos diferentes, e a subtracção responde **sozinha** (as duas seguradas dão zero — que
é exactamente a lei que o [`PlayerKeys::drive`](../../shells/desktop/src/player_input.rs) já
implementa à mão, e que este mapa passa a dar de graça).

### 2.4 ⛔ Contextos com prioridade — **e aqui não é teoria, é um defeito já medido**

O `player_input.rs` tem **testes a afirmar que certas teclas NÃO são do jogador**:

```rust
assert!(!k.key(KeyCode::Space, true));
assert!(!k.key(KeyCode::KeyW, true), "W abre o painel de MUNDO");
```

⇒ o conflito editor↔jogo **já existe e hoje é resolvido por uma lista negra escrita à mão** —
exactamente a forma que este repo já mediu como apodrecida
([[feedback_a_hand_written_list_beside_a_predicate_is_two_answers]]).

**A cura é o `InputContext` do Unreal:**

| Contexto | Quem o arma | Prioridade |
|---|---|---|
| `Editor` | sempre, enquanto o editor tem o canvas | **alta** |
| `Game` | quando há um player vivo e o foco é do jogo | baixa |

Uma tecla é resolvida pelo contexto **de maior prioridade que a reclame**; a lista negra
desaparece porque `W` está no contexto `Editor` e nunca chega ao `Game`.

### 2.5 Onde o mapa MORA

No **projecto** (viaja no `.ph2dproj`), porque é **conteúdo autorado**: o Enio define `jump`, e o
ficheiro tem de o levar. ⇒ move o **`PROJECT_SCHEMA`** (§3).

⚠️ **Um segundo ficheiro, de OVERRIDE por-jogador**, fica **fora do repo** (`~/.ph2d/`), como o
`prefs.txt` — é o remap que um jogador faz no menu, e ele **não** pode sujar o projecto. O
`InputMap` do Godot faz exactamente esta separação (project settings vs runtime remap).

---

## §3 — Onde encosta em contrato congelado (§6) e schema

| | Estado (a re-verificar no dia) |
|---|---|
| **Contrato de nós** (`NodeOp`/`OpResolver`/`NodeManifest`) | **não encosta** |
| **Contrato de tools** (`Tool=12`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`) | **não encosta** — o mapa não é uma `Tool` |
| **`ph2d-vector-doc`** (congelado) | **não encosta** |
| **`PROJECT_SCHEMA`** | ⛔ **MOVE** — hoje **95**; o número **conta-se** contra o `main` do dia, nos **três** sítios ([`CLAUDE.md §5.0`](../../CLAUDE.md)) |
| **Registro de componentes** | provável **+1** (`ph2d-ecs` em 65, com dois espelhos em 66) — número que **soma entre linhas** |
| ⛔ **`physics_ecs_c9` (replay-hash, 3 OS no CI)** | **não pode mudar de valor.** Se mudar, a §2.1 foi violada |

**Prova por grep, a colar no handoff:**
```bash
git diff --name-only <base>..HEAD -- crates/ph2d-nodegraph/src/node.rs \
    crates/ph2d-editor-core/src/tool.rs crates/ph2d-vector-doc/ crates/ph2d-vector-traits/
```

---

## §4 — A UI, e as **quatro condições independentes**

Painel **Input Map** (lista de acções → ligações), com o gesto **press-to-bind**.

| # | Condição | O que tem de ser verdade |
|---|---|---|
| 1 | **o componente EXISTE** | linha de acção · campo de nome · fileira de ligações · botão `+` · o estado **"a ouvir…"** |
| 2 | **é pintado E registrado** | ⚠️ é a costura que este módulo já falhou **duas** vezes em 23/08 (a fileira nunca pintada; os chips mortos sob o ponteiro). Gate de **gesto real**, não `Click` sintético |
| 3 | **o clique chega ao barramento** | `+` arma a escuta; a próxima tecla **é capturada e não executada** |
| 4 | ⭐ **a SEQUÊNCIA leva a algum lugar** | criar acção → ligar tecla → **o jogador anda com ela**. É a 4.ª pergunta que a auditoria de 23/08 nomeou, e a que fica verde com a feature inalcançável |

⚠️ **O estado "a ouvir" é o gesto inteiro:** enquanto ele dura, a tecla capturada **não pode**
disparar o atalho do editor. Sem isso, ligar `S` a `dash` **salva o projecto**.

---

## §5 — Os gates, **red-first**, e a fixtura que contém o fenómeno

| Gate | O que morre se ele não existir |
|---|---|
| `the_tape_records_the_action_never_the_key` | ⛔ **o mais importante**: remapear `jump` e provar que o **hash de replay não muda**. Fixtura: uma fita gravada, um remap, o mesmo hash |
| `two_keys_held_give_zero` | a lei do `drive` que o `PlayerKeys` já tem, agora derivada do mapa |
| `a_lost_key_release_never_walks_forever` | o modo de falha que o doc-comment do `player_input.rs` nomeia (foco roubado) |
| `the_editor_context_wins_the_key_the_game_wants` | `W` não move o jogador enquanto o editor tem o canvas — **substitui a lista negra** |
| `the_dead_zone_and_the_press_point_are_two_numbers` | a correcção à referência (§1.1): um valor abaixo da deadzone dá `0` **e** `pressed == false`, e há um intervalo em que dá `>0` **com** `pressed == false` |
| `an_action_with_zero_bindings_is_silent_not_absent` | declarada e por atribuir ≠ inexistente |
| `binding_capture_does_not_fire_the_editor_shortcut` | §4 condição 3 |
| `seam_input_map.rs` (**gesto real**) | as 4 condições de §4, com um pixel carregado |
| ⛔ `physics_ecs_c9` | **valor inalterado** — o controlo de que a §2.1 foi honrada |

⚠️ **Cada um red-first, e com prova de mutação** — os três controlos vão no arnês
(verde-antes · `Compiling <pkg>` · `running 1 test`).

---

## §6 — A cena de smoke

`PH2D_INPUT_MAP_SMOKE=1` — **nasce PRONTA** (auto-play: *feature nova = auto-play*):

1. Uma cena com um jogador que já anda, com o mapa **já povoado**.
2. O painel **Input Map** aberto, mostrando `move_left` · `move_right` · `jump` e as teclas.
3. O Enio troca `jump` de `Z` para outra tecla **em três cliques** e o boneco pula com a tecla nova.
4. ⭐ **O controlo:** rebobinar mostra o replay **idêntico** — o remap não reescreveu o passado.

⚠️ **Os números medidos (custo por tique, tamanho do mapa no ficheiro) entram nesta secção ANTES da
mensagem ao Enio** — sonda headless primeiro, `CLAUDE.md §0.0`.

---

## §7 — Ordem de execução

1. ✅ **W1 — FEITA em 2026-08-24.** O modelo + a resolução, na crate-folha `ph2d-input` que **já
   existia** (§2.0), sem `winit` e sem shell. Entregue: `keyboard.rs` (o dispositivo que faltava) ·
   `action.rs` (`Binding`/`ActionId`/`InputAction`, sobre os tipos de gamepad da M8) · `map.rs`
   (`InputMap` + o contador estável) · `resolve.rs` (`ActionState`/`Sample`/`Input`, lendo o
   `InputState` da M8) · `Event::{KeyDown, KeyUp, FocusLost}`. **40 gates verdes**, **10 provados
   por mutação** com os três controles, clippy limpo, e **zero pacote externo novo** no
   `Cargo.lock` (o `serde` já lá estava).
   ⚠️ **Fica NOMEADO o que a W1 não fez:** o rato não é modelado (esta crate nunca o modelou; o
   editor trata dele pelo despacho próprio), e nada disto está **ligado ao jogador** ainda — é a W2.
2. ✅ **W2 — PERSISTÊNCIA — FEITA em 2026-08-24.** O mapa viaja no `.ph2dproj`:
   **`PROJECT_SCHEMA` 95 → 96** nos **três** sítios, campo `input_map` **apendado ao fim** do
   `ProjectFile` (postcard é posicional), fora do `ProjectState` (um Ctrl+Z do canvas não rebobina
   controlos). A `App` ganhou o mapa **autorado** ao lado do retrato de dispositivos, e o estado
   **resolvido** ao lado dos dois. **4 138 gates verdes**, **4 provados por mutação**.
   ⭐⭐ **E a mutação achou um buraco real:** um `#[serde(skip)]` no contador de ids passava por
   **todas** as afirmações da ida-e-volta, porque elas só olhavam para as acções que já existem —
   um mapa recarregado reatribuiria ids **já gravados**. A cura é perguntar pela **porta** (criar
   uma acção nova no mapa que voltou e exigir que o id dela não colida), e é a armadilha que o
   doc-comment do `ActionId` nomeia.
   ⛔ **O que a W2 NÃO fez, e é sequenciamento e não lacuna:** o **override por-jogador** em
   `~/.ph2d/` — ele só tem consumidor quando existir a janela para remapear (W3), e construí-lo
   antes seria um ficheiro que ninguém escreve.
3. ⭐⭐ **W3 — A JANELA FLUTUANTE** (era W5), §0.2. *Chrome handler* com `z` **contado**, metade de
   pintura + metade de despacho, `cargo run -p ph2d-chrome-sync` a gerar o `mod` e o `dispatch_all`.
   É a entrega que o Enio nomeou, e é o que torna o mapa alcançável para **qualquer objecto**.
   ⚠️ As **quatro condições** de §4 medem-se aqui, e a quarta (a SEQUÊNCIA) é a que fica verde com
   a feature inalcançável.
4. **W4 — CONTEXTOS com prioridade** (§2.4). Assim que a janela liga uma tecla qualquer, o conflito
   editor↔jogo deixa de ser hipótese: a lista negra à mão do `player_input.rs` **morre** aqui.
5. ✅ **W5 — O JOGADOR — FEITA em 2026-08-24.** O `PlayerKeys` **cravado desapareceu**: o dedo do
   jogador é resolvido do `InputMap` do projecto (`App::resolve_player_input`), e o mapa de fábrica
   traz os **seis** verbos com as teclas de ontem **ao bit**.
   ⭐⭐ **E a LEI Nº 1 deste plano era um risco que o desenho JÁ evitava.** Medido: o `TapeWire`
   grava `(drive: f32, botões: u8)` — **semântico, nunca um keycode**. Remapear `jump` de `Z` para
   `Espaço` não toca em gravação nenhuma, e o hash do `physics_ecs_c9` ficou **byte-idêntico**
   (`2d7f9d51…`), com as crates `ph2d-physics-ecs` e `ph2d-platformer` **intocadas** pela linha.
   *Uma nota de risco escrita sem medir descreve um perigo que o desenho já tinha evitado.*
   ⚠️ **Dois normalizadores de tecla, e um gate a impedi-los de divergir:** o do editor devolve
   `None` de propósito para o que ele deixa **cair** (e alargá-lo mudaria todo atalho que depende
   dessa queda); o do Input Map mapeia tudo, no **mesmo** espaço `u32`.
   `the_two_normalizers_never_disagree` prova que onde um diz `Some(v)`, o outro diz o mesmo `v`.
   ⚠️ **Cinco gates pré-existentes foram RE-ANCORADOS, nenhum apagado** — a lei deles continua a
   valer, só mudou de endereço (`the_space_bar_is_not_a_jump_key` passou de scanner de texto a
   medição de comportamento, que é mais forte).
6. **W6 — gamepad ao vivo** e a deadzone real no dispositivo. ⚠️ **Só aqui entra dependência nova**
   (o `gilrs` já existe na shell); antes disso o `Cargo.lock` não ganha pacote externo.

> ⚠️ **Meça cada linha deste plano antes de a honrar.** Escrito em 2026-08-24; *quem move o número
> reconfere a nota* ([estudo §6.6.1](Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md)).

---

## Fontes

- [Godot — Introducing the new axis handling system](https://godotengine.org/article/handling-axis-godot/) ·
  [InputEvent tutorial](https://docs.godotengine.org/en/stable/tutorials/inputs/inputevent.html) ·
  [InputMap class](https://rokojori.com/en/labs/godot/docs/4.3/inputmap-class) ·
  [proposta da deadzone configurável (#3709)](https://github.com/godotengine/godot-proposals/issues/3709) ·
  [fórum: "How is the Deadzone supposed to work?"](https://forum.godotengine.org/t/how-is-the-deadzone-from-input-map-supposed-to-work/70685)
- [Unreal — Enhanced Input (documentação oficial)](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine) ·
  [Enhanced Input: What you need to know](https://unrealdirective.com/articles/enhanced-input-what-you-need-to-know/)
- [Unity — "New Input System is a MESS"](https://discussions.unity.com/t/new-input-system-is-a-mess/1606485) ·
  ["so Convoluted?"](https://discussions.unity.com/t/new-input-system-is-so-convoluted/879754) ·
  [Using Unity's new Input System (devlog)](https://loveglitchcoffee.itch.io/overgem/devlog/148752/using-unitys-new-input-system)
- [Coyote Time, Input Buffering, and the Art of Forgiving Controls](https://www.gamejuice.co.uk/articles/coyote-time-input-buffering) ·
  [Input Buffering and Coyote Time in 2D](https://gamineai.com/blog/input-buffering-and-coyote-time-in-2d-a-godot-4-and-unity-friendly-timing-primer)

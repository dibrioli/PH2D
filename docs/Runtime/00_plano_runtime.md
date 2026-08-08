# Plano — o RUNTIME do PH2D

> **Pedido do Enio (2026-08-08):** *"Precisamos de algum modo criar o Runtime pois vários outros
> módulos têm a mesma necessidade e dizem a mesma coisa. Então vamos fazer de forma muito séria e
> planejada."* · e depois: *"buscando sempre o estado da arte, o padrão ouro."*
>
> Escrito na `line/Vector` em 2026-08-08. **Todo número interno foi medido nesta worktree**, com o
> comando ao lado; toda referência externa tem a fonte no §2.

---

## §0 — ⭐ O achado que reescreve a premissa

O plano do vector diz, em quatro handoffs, que o W8a está *"bloqueado por AUSÊNCIA: `ph2d-runtime`
não existe"*. A frase está certa sobre o **nome** e errada sobre a **situação**.

**Medido hoje:** o runtime não é uma coisa a construir do zero. **Três módulos já construíram, cada
um por conta, o PRODUTOR de um canal de eventos — e nenhum tem consumidor.**

| módulo | o produtor, construído | o consumidor, hoje |
|---|---|---|
| **timeline** | `signals_crossed()` (ADR-0143 — *"um marker EMITE um evento, não uma chamada"*) | **um toast** |
| **física** | `PhysicsBridge::signal_events()` (`SignalOnHit`/`SignalOnLeave`) | **um toast** |
| **script** | `ph2d_script::MessageBus` — barramento **estilo Defold**, hash-interned, FIFO, handlers por id, alvo HR-4 de **100 000 msg/frame ≤ 1,5 ms** | ⚠️ **NINGUÉM** (grep: zero usos fora da própria crate) |
| **áudio** | mixer que embarca em jogo + streaming (ADR-0118) | 3 itens abertos com a mesma cerca: *"fica para quando houver um consumidor real"* |

E os dois primeiros **dizem a mesma frase, escrita por linhas diferentes**:

> `timeline_bridge.rs`: *"v1 consumer: a toast … **Audio/gameplay/Luau are the deferred cross-line
> consumers of the SAME outbox**; the timeline emits an event and never calls any of them."*
>
> `render_loop/mod.rs`: *"os quatro canais de leitura dela existem desde o W7 e **nenhum fazia nada
> ACONTECER**; o que faltava era o publicador, não o consumidor … **Duas fontes, um consumidor, e é
> aqui que elas se encontram.**"*

⇒ **É exactamente o que o Enio observou.** O trabalho não é inventar um runtime: é **fechar o laço
que quatro módulos já deixaram meio-feito**, e depois dar-lhe uma casa onde o editor não existe.

### 0.1 E a fronteira já está limpa — medido

`grep -c "ph2d-editor" crates/<X>/Cargo.toml` sobre as nove crates de modelo:

| `ph2d-vec-scene` | `ph2d-timeline` | `ph2d-anim` | `ph2d-physics-ecs` | `ph2d-ui-state` | `ph2d-tokens` | `ph2d-script` | `ph2d-audio` | `ph2d-ecs` |
|---|---|---|---|---|---|---|---|---|
| **0** | **0** | **0** | **0** | **0** | **0** | **0** | **0** | **0** |

**Nenhuma delas arrasta o editor.** O runtime é BARATO — o que falta não é desacoplar, é **montar**.

---

## §2 — O estado da arte, e o que cada um resolve

| motor | como o jogo roda sem o editor | o que herdamos |
|---|---|---|
| **[Godot](https://www.godotbuilder.com/guides/export-templates)** | **export template**: um binário pré-compilado do motor **sem editor nem ferramentas de debug**, + um `.pck` com os assets | ⭐ **o binário é OUTRO**, não o mesmo com um flag |
| **[Rive](https://rive.app/docs/runtimes/state-machines)** | a state machine roda no runtime; **listeners** disparam inputs e **eventos**, e o host liga *delegates* para os ouvir | o modelo *evento → delegate do host*, que é o nosso outbox |
| **Defold** | barramento de mensagens hash-interned | ⚠️ **já herdado**: o `messaging.rs` diz *"Defold-style"* na primeira linha |
| **[Fyrox](https://openapps.pro/apps/fyrox)** | workspace de crates com **acoplamento unidirecional**; `fyrox-ui` é standalone e agnóstico de API gráfica | ⭐ é a arquitetura que **já temos** (o §0.1 mede-a) |

### 2.1 ⚠️ E a armadilha que a pesquisa nomeia: **feature unification**

O caminho tentador é *"o runtime é uma feature do mesmo binário"*. A
[RFC 3692](https://rust-lang.github.io/rfcs/3692-feature-unification.html) e o
[pitfall de workspace](https://nickb.dev/blog/cargo-workspace-and-the-feature-unification-pitfall/)
dizem por que ele é frágil: o cargo toma a **união** das features pedidas em todo o workspace, então
*"uma feature ligada num pacote afeta as features ligadas noutros"* — um build de jogo por flag num
monorepo **não é o build que o `Cargo.toml` daquele pacote descreve**.

⚠️ **E este repo já tem a cicatriz, escrita:** a feature `sculpt3d` está na lista **`default`** do
shell **de propósito**, porque *"o `ship.sh` roda clippy **sem** `--all-features`, e atrás de uma
feature desligada este código não seria lintado"*.

⇒ **O runtime é uma SHELL, não uma feature** — o `shells/game` ao lado do `shells/desktop`, que é a
resposta do Godot e a que o ADR-0075 já implica (*plugin em runtime foi pesquisado e **rejeitado***).

---

## §3 — A espinha: **UM outbox, N consumidores, e o produtor nunca chama ninguém**

É a lei que os dois módulos já escreveram e que este plano só torna executável:

> A timeline **emite** um evento e não chama o áudio. A física **publica** o seu e não importa o
> tipo de sinal da timeline. Quem os funde é **o host**.

⚠️ **Isto NÃO é um design novo — é o ADR-0075 aplicado a eventos** (*desacoplar por ECS: components
+ events/resources, systems não se chamam*). O que falta é o **barramento** ser o do repo
(`MessageBus`) em vez de um `Vec<Toast>`.

⚠️ **E o toast FICA.** Ele é o *readout* do canal no editor, e o editor precisa de ver o que
disparou. O que muda é quem é a fonte da verdade: hoje o toast **é** o consumidor; depois ele passa
a ser **um** consumidor, ao lado do bus.

---

## §4 — As waves, na ordem, com o que cada uma DESTRAVA

### **R0 — o barramento ganha os dois produtores que já existem** ⭐ *começa aqui*

Os sinais de timeline e de física entram no `MessageBus`. Nenhuma crate nova, nenhuma shell nova.

**Destrava, de uma vez:**
- o *"sinal de gameplay"* que a **física pediu DUAS vezes** (W7 e W-ContactEvents, as duas marcadas
  *"cross-line, decisão do Enio, precisa do desenho do consumidor"*);
- o **primeiro consumidor de produto** do `MessageBus`, que hoje tem zero;
- e o **teste do desenho antes da casa**: se o outbox único não servir para os dois produtores, é
  aqui que se descobre — e não depois de uma shell existir.

⚠️ **A ordem dentro do frame já foi decidida e está documentada**, e ela é load-bearing: o dreno da
timeline roda **antes** do dispatch da física, então ler os sinais de física ali entregaria os do
quadro **anterior** — *"um atraso de um quadro é invisível num toast e deixa de ser invisível no dia
em que o consumidor for som."* **Esse dia é o R3.**

⚠️ **MEDIR nesta wave:** o custo do bus com o tráfego REAL do editor (o alvo do HR-4 é 100 k
msg/frame ≤ 1,5 ms, e o tráfego de dois produtores de sinal é de outra ordem — o número não é o
teto, é o piso da folga).

### **R1 — a SHELL de jogo mínima**

`shells/game`: abre uma janela, carrega um `ProjectFile`, roda `sim` + `render` + o bus, e **não
compila um único painel**.

⚠️ **O gate que a define não é o que ela faz, é o que ela NÃO alcança:** um arch-gate sobre o
`Cargo.toml` dela, no molde do `no_ml_runtime_reaches_the_mixer` e do `ph2d-paint-gpu` — *a shell de
jogo não depende de `ph2d-editor-core` nem de nenhuma `ph2d-panel-*`*. A contenção fica
**estrutural, não disciplinar**.

⚠️ **O que ela expõe é o valor real da wave:** hoje o `ProjectFile` é lido por um `App` que nasce com
`window`/`gfx` em `None` — o load é dirigível sem janela (o `project_tests.rs` já o faz), mas
**`project_save` exige `gfx`** (nomeado como dívida na §5 da timeline desde 2026-07-19). A shell de
jogo é **read-only**, então ela não paga essa dívida — mas é ela que a torna visível.

### **R2 — o EMPACOTAMENTO** (o `.pck` do Godot)

Hoje um projeto é um `postcard` com os pixels embutidos e `PROJECT_SCHEMA` **55**. Um jogo precisa de
*"o que roda"* sem *"o que se autora"*.

⚠️ **A decisão que esta wave toma e que nenhuma outra pode:** o formato de jogo é o **mesmo**
`ProjectFile` (e a shell de jogo simplesmente ignora o que não usa) **ou** é uma projeção dele? O
segundo é o Godot; o primeiro é mais barato e não tem um segundo formato para divergir.
**Recomendação: o mesmo arquivo**, até haver um número que o condene (tamanho ou tempo de load).

### **R3 — os consumidores diferidos**

- **Áudio** — o ADR-0118 §5 tem **três** itens à espera de *"um consumidor real"*. O R0 cria-o.
- **Luau** — o `ScriptRuntime` e o `Scheduler` existem; o que falta é o handler de `MessageId` ser
  registrável de um script.
- **UI (o W8a do vector)** — a máquina de estados de UI rodando sem editor. ⚠️ **Ela é a que menos
  falta:** o `ph2d-ui-state` não depende do editor (§0.1) e o `advance(dt)` já é determinista.

---

## §5 — O que fica FORA, e por quê

- **Uma segunda representação de cena para o jogo.** [[feedback_two_engines_one_state_is_worse_than_a_slow_engine]] — o §0.1 mostra que não é preciso.
- **Plugin em runtime / WASM.** Rejeitado pelo **ADR-0075**, com a pesquisa já feita.
- **Um formato de asset por módulo.** O `ProjectFile` já é o envelope; um segundo seria a segunda
  porta para *"o que é um projeto"*.
- **Determinismo cross-OS como requisito do R1.** Ele já existe onde foi pedido (`physics_ecs_c9`
  na matriz de 3 OSes); estendê-lo ao runtime inteiro é um requisito que ninguém formulou ainda.

---

## §6 — As decisões que são do ENIO

1. **A shell de jogo é o alvo, ou basta o editor rodar o jogo em Play?** O R1 só se justifica se o
   produto final for um binário que se distribui. (O pedido de 2026-08-01 diz *"também será usada
   para criar a UI dos games"* — o que implica o binário, mas não o data.)
2. **O que um sinal PODE fazer.** O R0 entrega *o sinal chega ao bus*. Quem o transforma em
   **som**, em **script** ou em **troca de estado de UI** é o R3, e a ordem entre os três é de
   produto: o áudio é o mais barato (o mixer existe), o Luau é o mais poderoso, a UI é a do
   pedido de 01/08.
3. **O nome.** O plano do vector chama-lhe `ph2d-runtime`. Medido, ele **não é uma crate** — é uma
   **shell** mais uma fiação. Sugiro `shells/game`, e que `ph2d-runtime` deixe de ser citado como
   crate ausente nos quatro handoffs que o nomeiam.

---

## §7 — Tabela-resumo

| wave | o que é | crate nova | schema | dep nova | destrava |
|---|---|---|---|---|---|
| **R0** | os dois produtores entram no `MessageBus` | **nenhuma** | **nenhum** | **nenhuma** | o *sinal de gameplay* pedido 2× pela física; o 1º consumidor do bus |
| **R1** | `shells/game` — janela, load, sim, render, bus | shell nova | nenhum | nenhuma | o binário que se distribui |
| **R2** | o empacotamento | — | ⚠️ decisão do §4 | — | o jogo carrega só o que roda |
| **R3** | áudio · Luau · UI como consumidores | — | — | — | 3 itens do ADR-0118; o W8a do vector |

⚠️ **A ordem não é por tamanho — é porque o R0 TESTA o desenho** com dois produtores reais, e é o
único que não pode ser invalidado por uma decisão posterior.

# HANDOFF — o RUNTIME, para quem assume (`line/runtime`)

> **Para quem retoma.** Faça a **FASE 0** do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> **antes de abrir qualquer arquivo** — a janela abre na raiz (que é `main`) e os mesmos paths
> relativos existem nas duas árvores: editar a errada **compila e commita sem um único erro**.
>
> **Módulo:** Runtime. **Worktree:** `Worktrees/line-runtime/`. **Branch:** `line/runtime`.
> **HEAD deste handoff:** `37ff53467` (3 commits à frente de `main`).
>
> ⚠️ **Este handoff foi escrito NA `line/Vector`**, que é onde a investigação aconteceu. Ele é
> **plano e medição**, não código: a `line/Vector` **não escreveu uma linha de runtime**, de
> propósito (§4).

---

## 1. Comece por aqui, nesta ordem

1. [`00_plano_runtime.md`](00_plano_runtime.md) — **o plano**, com o achado do §0 e o estado da arte
   do §2.
2. [`01_o_formato_medido.md`](01_o_formato_medido.md) — **o seu próprio doc**, e ele já derrubou um
   plano aprovado. Não o re-derive.
3. Este arquivo — **o estado e a ordem**.

---

## 2. ⭐ O achado, em quatro linhas

**O runtime não está ausente. Ele está MEIO-FEITO em quatro módulos, e cada um parou no mesmo
lugar: construiu o produtor e não teve consumidor.**

| módulo | produtor construído | consumidor hoje | prova |
|---|---|---|---|
| timeline | `signals_crossed()` (ADR-0143) | **um toast** | `timeline_bridge.rs:48` |
| física | `PhysicsBridge::signal_events()` | **um toast** | `render_loop/mod.rs:1831` |
| **script** | **`MessageBus`** estilo Defold, alvo HR-4 de **100 k msg/frame ≤ 1,5 ms** | ⚠️ **NINGUÉM** | `grep MessageBus` → zero usos fora da própria crate |
| áudio | mixer que embarca em jogo (ADR-0118) | — | 3 itens abertos: *"fica para quando houver um consumidor real"* |

E os dois primeiros **escreveram a mesma frase, em linhas diferentes**:

> *"Audio/gameplay/Luau are the deferred cross-line consumers of the **SAME outbox**; the timeline
> emits an event and never calls any of them."* — `timeline_bridge.rs`
>
> *"os quatro canais de leitura dela existem desde o W7 e **nenhum fazia nada ACONTECER**; o que
> faltava era o publicador, não o consumidor … **Duas fontes, um consumidor, e é aqui que elas se
> encontram.**"* — `render_loop/mod.rs`

### 2.1 E a fronteira já está limpa — medido, não suposto

`grep -c "ph2d-editor" crates/<X>/Cargo.toml` sobre as **nove** crates de modelo (`vec-scene`,
`timeline`, `anim`, `physics-ecs`, `ui-state`, `tokens`, `script`, `audio`, `ecs`) → **0 em todas**.

⇒ **o runtime é BARATO.** O que falta não é desacoplar; é montar.

---

## 3. A wave que começa: **R0 — os dois produtores entram no `MessageBus`**

Nenhuma crate nova, nenhuma shell nova, nenhum schema.

**Por que ela primeiro:** é a única que **testa o desenho antes de existir a casa**. Se um outbox
único não servir aos dois produtores, descobre-se aqui — não depois de uma shell existir.

**O que ela destrava de uma vez:**
- o *"sinal de gameplay"* que a **física pediu DUAS vezes** (W7 e W-ContactEvents, as duas marcadas
  *"cross-line, decisão do Enio, precisa do desenho do consumidor"*);
- o **primeiro consumidor de produto** do `MessageBus`.

### 3.1 ⚠️ Quatro coisas que já estão decididas — não re-litigue

1. **A ORDEM DENTRO DO FRAME é load-bearing e está documentada.** O dreno da timeline roda **antes**
   do dispatch da física, então ler os sinais de física ali entregaria os do quadro **ANTERIOR** —
   *"um atraso de um quadro é invisível num toast e deixa de ser invisível no dia em que o consumidor
   for som."* **Esse dia é o R3.**
2. **O toast FICA.** Ele é o *readout* do canal no editor. O que muda é que ele deixa de ser **o**
   consumidor e passa a ser **um**.
3. **A física NÃO importa o tipo de sinal da timeline**, e vice-versa. *"Fazer o motor de colisão
   depender do editor de animação para dizer 'algo bateu' é o oposto do ADR-0075."* Cada uma publica
   o seu; quem funde é o **host**.
4. **`HashMap` no `messaging.rs` é lookup-only e está isento**, com o motivo escrito lá (o ADR-0022
   bane a **iteração**, não o lookup; o dispatch quente é `Vec<Vec<Handler>>` indexado).

### 3.2 O que MEDIR nesta wave (antes de escrever o número, §0 do CLAUDE.md)

O `messaging.rs` declara o alvo HR-4 de **100 000 msg/frame ≤ 1,5 ms**. O tráfego de dois produtores
de sinal é de **outra ordem de grandeza** — meça-o. O alvo do HR-4 não é o teto desta wave: é o
**piso da folga**, e citá-lo como se fosse o custo seria vender o que não foi medido.

---

## 4. ⚠️ Por que este trabalho NÃO ficou na `line/Vector`

Três razões, e a primeira sozinha basta:

1. **O R0 toca quatro módulos de que a `line/Vector` não é dona** — `render_loop/mod.rs` ·
   `ph2d-script` · `ph2d-timeline` · `ph2d-physics-ecs` —, e dois deles têm donos ativos
   (`line/physics`, `line/anim`). CLAUDE.md §0.2.
2. **A `line/Vector` já tem um diff grande por integrar** (`PROJECT_SCHEMA` **62** contra `main`
   **55**, crates novas, sete waves). Somar o runtime aumentaria a superfície de colisão de uma
   integração que já é grande.
3. **Você já é dono do assunto e já fez o R2.**

---

## 5. ⚠️ Uma correção que este handoff traz para o SEU plano

A primeira versão do [`00_plano_runtime.md`](00_plano_runtime.md) §4/R2 dizia: *"o formato de jogo é
o mesmo `ProjectFile` … **recomendação: o mesmo arquivo**, até haver um número que o condene."*

**Ela foi RETIRADA**, porque o número já existia — e é seu. O
[`01_o_formato_medido.md`](01_o_formato_medido.md) classifica os **37 bumps** por onde a mudança
pousou (**18** dentro de blob de componente · **12** no Flip · **3** no Vec · **4** no topo) e mostra
que um envelope de topo previne **4 de 37 (11%)**, enquanto **versão por `ComponentBlob`** alcança os
18 da linha A.

⇒ **o R2 é da `line/runtime` e já tem desenho medido.** O plano do runtime aponta para ele em vez de
competir com ele.

---

## 6. O que a `line/Vector` deixa pronto e você não precisa refazer

- **O CENSO dos pedintes** (§2 acima), com o comando de cada medição.
- **O estado da arte** ([`00_plano_runtime.md`](00_plano_runtime.md) §2): Godot (export template =
  **outro binário**) · Rive (evento → *delegate* do host) · Defold (o messaging que já herdamos) ·
  Fyrox (a arquitetura de crates que já temos).
- ⚠️ **A armadilha que decide a FORMA:** a **feature unification** (RFC 3692) faz o cargo unificar
  features no workspace ⇒ um build de jogo por feature-flag **não é o build que o `Cargo.toml`
  daquele pacote descreve**. E este repo já carrega a cicatriz: a feature `sculpt3d` está na
  `default` **de propósito**, porque *"o `ship.sh` roda clippy sem `--all-features`, e atrás de uma
  feature desligada este código não seria lintado"*.
  ⇒ **o runtime é uma SHELL (`shells/game`), não uma feature.**
- **O gate que define a shell** quando ela nascer (R1): não é o que ela faz, é **o que ela não
  alcança** — arch-gate sobre o `Cargo.toml` dela, no molde do `no_ml_runtime_reaches_the_mixer` e
  do `ph2d-paint-gpu`: *a shell de jogo não depende de `ph2d-editor-core` nem de nenhuma
  `ph2d-panel-*`*. Contenção **estrutural, não disciplinar**.

---

## 7. As decisões que são do ENIO (não avance sem elas onde marcado)

1. **O alvo é um binário distribuível, ou basta o Play do editor?** O **R1 depende disto**; o R0
   não.
2. **O que um sinal PODE fazer primeiro** (R3): **áudio** é o mais barato (o mixer existe e tem 3
   itens à espera), **Luau** é o mais poderoso (`ScriptRuntime` + `Scheduler` existem), **UI** é a do
   pedido de 2026-08-01.
3. **O nome** — medido, não é uma crate. `ph2d-runtime` é citado como *"crate ausente"* em **quatro**
   handoffs da `line/Vector`; se o nome mudar, essas quatro notas passam a mentir.

---

## 8. A lei da linha

Cada wave fecha com **UI na mesma wave** (quando houver UI), **smoke próprio**, **bateria batched**,
**mutações**, e **handoff de integração** — e então **PARA**. A linha **não integra e não pusha
sozinha** (CLAUDE.md §0.7).

⚠️ **E rode a suíte em DEBUG e em RELEASE.** Precedente registrado: um gate de wall-clock reprovou
só em debug (21,65 contra 1,92 ms), e o `ph2d-flip-colorize` panicava só em debug — a nota disso
sobreviveu ao fato por três integrações.

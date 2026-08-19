# HANDOFF DE INTEGRAÇÃO — `line/physics` (2026-08-01, 2ª jornada do dia)

**Status:** FECHADO 2026-08-01 · no `main` em `8cabda9a3` (o commit que trouxe este arquivo).

> ⚠️ **SUPERSEDIDO por [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-02.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-02.md)**, que cobre os **38**
> commits da linha inteira. Este aqui cobria só os 24 primeiros, e **os números de
> identidade dele estão DESATUALIZADOS** — a linha ganhou mais uma wave (W-LeadDrag)
> e o limitador (W-RopeStop) depois que ele foi escrito. Fica como histórico das três
> waves do fim daquela jornada.

> **Para o agente integrador.** A linha está **FECHADA**. **24 commits, sete waves,
> todos os smokes aprovados pelo Enio.** Ela **não** integrou e **não** pushou —
> DIRETRIZ §1.5.9.
>
> ⚠️ **Este handoff NÃO é o de `HANDOFF_INTEGRACAO_line_physics_2026-08-01.md`.** Aquele
> cobre as quatro waves da manhã (W-JointCopy · W-Rig · W-SoftWeld · W-Compound) e **já
> está no `main`**. Este cobre o que veio **depois** delas, e a maior parte existe
> *por causa* delas.
>
> **Tracker por-wave:** [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) ·
> **Mapa:** [`00_plano_waves.md`](../00_plano_waves.md)

---

## §1 — O que integrar

**Branch:** `line/physics` · **tip `7811fdca7`** · **base `a9f5977e9`**

⚠️ **A linha JÁ FOI REBASADA sobre o `main` de agora** (`git rebase main`, 24/24 limpo). O
`main` tinha andado **um** commit desde o fork — `a9f5977e9`, **memória apenas**
(`project-memory/`), **zero sobreposição de arquivo** com a linha (medido, não suposto).
Um `--ff-only` deve passar direto.

| # | commit | wave |
|---|---|---|
| 1 | `804dac9d9` | as duas escadas de schema tinham um degrau não documentado (doc) |
| 2-8 | `4e94bb95b` … `fe750e0be` | **W-PartFace** — a peça vira **editável** (cenas `=69`/`=70`) |
| 9 | `10ee82198` | tracker |
| 10-11 | `80f05104c`, `f35d7481d` | **W-PartSensor** — ser sensor é propriedade da **FORMA** (cena `=71`) |
| 12-13 | `cfb9d91e2`, `f402749f4` | **W-CompoundZone** — uma zona vê o corpo composto inteiro (cena `=72`) |
| 14-15 | `929dbb316`, `f2e958476` | **W-PartMass** — o seed Auto→Manual conhece as peças |
| 16-18 | `6cd9e0985`, `9b7748729`, `ba1b1eeef` | **W-CompoundContact** — um corpo composto toca **UMA** vez |
| 19-20 | `e70e5ea23`, `249d27a7c` | **W-WorldPinGlyph** — a hachura de chão (cena `=65`) |
| 21-22 | `25d239fc5`, `e22f29118` | **W-WorldPinLocal** — a alça de *onde no corpo* (cena `=65`) |
| 23-24 | `21c2b8a09`, `7811fdca7` | **W-Signal** — uma colisão FAZ algo acontecer (cena `=73`) |

**80 arquivos · 7589 inserções · zero `Cargo.toml` tocado** (nenhuma dep nova, nenhuma
crate nova, nenhum ADR novo — tudo sob a [ADR-0131](../../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)).

---

## §2 — Os números de identidade (MEDIDOS agora, nesta árvore, pós-rebase)

| o quê | valor | como conferir |
|---|---|---|
| **`PROJECT_SCHEMA`** | **48, INTOCADO** | `git diff main..HEAD -- shells/desktop/src/project.rs` = vazio |
| **tripla-pin** | **`(48, 13, 13)`, intocada** | `project_schema_tests.rs` |
| **`FLIP_SCHEMA_VERSION`** | **13, intocado** (só o **doc** da escada) | o const não aparece no diff |
| **registro `ph2d-physics-ecs`** | **24 → 25** (`SignalOnHit`) | `registers_every_physics_component` (`lib.rs:134`) |
| **`physics_ecs_c9`** | **99 corpos** · `16ba80e807ebc8097ffe1b6da87fb651ed4914ce34408a46629bccda596f75c8` | roda o bin; **debug ≡ release**, conferido nas duas |
| **ids novos** | **um**: `INSP_PHYS_SIGNAL` | `git diff main..HEAD -- crates/ph2d-editor-core/src/ids/` |
| **gizmo ids** | **nenhum novo** (último **971**, próximo livre **972**) | a W-WorldPinLocal **reusa** `GIZMO_JOINT_ANCHOR`/`_B` |
| **contrato congelado** | **intacto, 4/4 verde** | os quatro `architecture_*_contract_surface` rodados |

### ⚠️ Esta jornada fica FORA da disputa de número

**Nenhum schema se moveu.** É a primeira jornada desta linha em muito tempo com essa
propriedade, e ela não é sorte: o **único** componente novo (`SignalOnHit`) cunha
**blob-key própria** — componente NOVO não custa bump, **apendar campo a um que já existe**
custa, porque postcard é posicional. As waves de peça/composto não acrescentaram um campo
sequer: elas corrigiram **consumidores** que liam o mundo pela premissa velha.

⚠️ Se ainda assim você precisar mexer no `PROJECT_SCHEMA` por conta de outra linha na mesma
janela: **o valor se CONTA a partir do `main` do dia, não se escolhe** — esta linha já pagou
isso **três** vezes com a `line/FLIP` (30 em 25/07 · 32/33/34 em 27/07 · **47 hoje de manhã**,
e aquela quase passou **muda** porque o `project.rs` não conflita quando os dois lados
escrevem o mesmo literal). [[feedback_numbers_that_sum_across_lines_count_dont_pick]]

---

## §3 — A espinha da jornada: **uma premissa que envelheceu**

A W-Compound (integrada de manhã) tornou falsa uma frase que estava escrita em docs de
módulo, em nomes de função e em código:

> *"um corpo tem exatamente um collider"*

Ela nunca foi uma decisão — era **verdade por construção**, e por isso ninguém a declarou.
Quando ela caiu, cada consumidor que a assumia passou a estar **errado em silêncio**, e nenhum
deles tinha gate capaz de perceber, porque **toda fixture do repo era de uma forma só**.

A jornada é a **varredura desses consumidores**, um por um, cada um com o mesmo formato:
achar o consumidor · medir o defeito **contra um CONTROLE de uma peça** · corrigir · gate.

| # | canal | o que a premissa quebrou | número medido |
|---|---|---|---|
| 1 | **autoria** (W-PartFace) | a peça era criada e **não podia ser editada**; a porta que a criou a apagava | a 3ª face do §11 |
| 2 | **trigger** (W-PartSensor) | *ser sensor* era propriedade do CORPO ⇒ um sensor de pé nunca acendia | cena `=71` |
| 3 | **zonas** (W-CompoundZone) | a zona via **uma forma** ⇒ a jangada capotava dentro d'água | cena `=72` |
| 4 | **massa** (W-PartMass) | o seed Auto→Manual somava **só a forma do corpo** | — |
| 5 | **contato** (W-CompoundContact) | **duas** entradas por toque, **metade** da carga em cada | `0,030677` × `0,061313` |

⚠️ **O nº 5 é o mais instrutivo, e vale para quem for escrever o próximo consumidor:**
`contact_pairs()` itera pares de **COLLIDER**. Enquanto um corpo tinha um só, *"uma entrada
por par de collider"* e *"uma entrada por par de corpos"* eram **a mesma frase** — e a lei
que o módulo declarava (*"dois objetos se tocando é UM evento"*) continuava escrita e
continuava sendo violada. A fusão passa a ser por par de corpos, com o ponto **mais profundo**
das formas e a carga **somada**.

⚠️ **A ordem da soma é fixada por construção** (ordena por par-de-corpos, depois por
par-de-collider) porque a ordem do `narrow_phase` é interna ao rapier e `f32` **não é
associativo** (HR-5). ⚠️ **E isto NÃO está gateado, de propósito:** medido, com **dois**
somandos a mutação que tira o desempate **não sangra** — a adição IEEE-754 *é* comutativa;
o que falha é a associatividade. Documentado em vez de gateado, o precedente do CAS da
ADR-0145. Um terceiro somando (um corpo de três formas) o torna observável.

---

## §4 — As três waves do fim (as que o Enio pediu por nome, nesta ordem)

### 4.1 W-WorldPinGlyph — a ponta que é o CENÁRIO ganha figura

**Medido antes de desenhar uma linha:** um pino de mundo e um pino entre dois corpos
produziam caminhos **byte-idênticos**. O produto dizia *"world"* por uma **AUSÊNCIA**
(`centre_b == anchor_b` ⇒ a tracejada de posse tem comprimento zero), e **ausência é
ambígua** — um corpo B centrado na âncora desenha exatamente a mesma coisa.

A figura é a **hachura de apoio fixo** dos diagramas de mecanismo (a notação universal de
*"esta ponta é a moldura"*), e ela mora **dentro do `ownership_lines`** — a função que já
responde *de quem é esta ponta* —, então ela apaga num joint desligado e avermelha num
rompido **sem uma linha a mais**. Desce pela **GRAVIDADE**, nunca pelo eixo da tela; sem
gravidade não há chão e ela não é desenhada.

⚠️ **A mutação que apaga os riscos SOBREVIVEU aos cinco primeiros gates** — eles pinavam que
a ponta de mundo *difere* e que ela *segue a gravidade*, e **uma barra nua satisfaz as duas**.
O gate que faltava lê a **geometria desenhada** (≥3 segmentos, ≥2 inclinados em relação à barra).
⚠️ E um gate meu nasceu **verde-sobre-errado**: `pts(&world)[0] > 2` é satisfeito pela
tracejada de 22 pontos do mundo **pré-wave**; o oráculo virou o **crescimento com a
gravidade** (2 → 12 pontos).

### 4.2 W-WorldPinLocal — a alça de ONDE NO CORPO ele prende

O `local_a` de um pino de mundo era **inalcançável** — nem alça, nem row. ⚠️ **E medir para
desenhar a alça expôs um vão maior: a porta LIA um número e ESCREVIA outro.** O dot era
desenhado em `world_from_local(corpo, local_a)` e o arrasto escrevia o **prego**, então
pedir `[0,5; 2,0]` deixava a leitura em `[0,0; 2,0]` com o `Transform` já em 0,5 — **a alça
não seguia o mouse**.

Agora **A é *onde no corpo*** e **B é a outra ponta** (que num pino de mundo é o prego), e
por lerem e escreverem o **mesmo** número as duas seguem o cursor. Cai numa infra que já
existia — *A pega o quadrado interno e B a banda de fora, que é o todo de como um par
coincidente continua sendo duas alças*.

⚠️ **MUDANÇA DE COMPORTAMENTO, aprovada no smoke:** o gesto de mover o prego mudou do
**dot** para o **anel**. A mensagem da cena `=65` foi reescrita para dizê-lo.
⚠️ E o gate antigo ficava **verde sobre o dot que não segue o mouse**, porque media o
`Transform` e **nunca perguntava onde a alça é desenhada** — *ler == escrever* é a
propriedade, e ela só é testável de um gate que conhece as duas metades.

### 4.3 W-Signal — uma colisão passa a FAZER alguma coisa acontecer

⚠️ **A decisão que o tracker chamava de *"cross-line, decisão do Enio"* já estava escrita no
produto**: o `render_loop` declara, em doc-comment, que *gameplay* é um dos consumidores
diferidos do **MESMO outbox** dos sinais da timeline (ADR-0143). **O consumidor existia; o
que faltava era o PUBLICADOR.** [[feedback_a_capability_without_a_door_passes_every_gate]]

`SignalOnHit(String)` carrega um **nome** e a física **nunca sabe quem escuta** — é o
desacoplamento da ADR-0075 (*systems não se chamam*). Duas fontes: **contato que começa** ·
**entrada em sensor** (canal novo — sem ele a PORTA fica muda, porque um sensor **nunca gera
contato**). A porta `signal_events` é **DERIVADA** dos dois canais que já existiam, nunca uma
terceira lista a manter de acordo.

⚠️ **Duas correções que a MEDIÇÃO impôs, as duas minhas:**

1. **A sonda achou um vazamento no meu próprio código** — `measure_sensor_blind_speed`
   reportou **58 sinais para UMA travessia** a 60 m/s. O early-out de grafo vazio do
   `rebuild_triggers` pulava o `diff_trigger_entries`, então o canal **retinha o último
   evento** e o publicador o re-emitia para sempre. Virou o gate
   `something_passing_through_a_sensor_shouts_once_and_then_the_channel_goes_quiet`.
2. **Um número que eu escrevi estava errado por uma ordem de grandeza** — documentei a
   cegueira de um sensor em `2h/dt` = **60 m/s**; medido, ele ainda dispara a **280** e só
   cega a **320**. A fase larga usa **AABBs PREDITAS**: um par entra no grafo por
   **MOVIMENTO**, não pelas poses de fim de tick. Doc corrigido com a tabela medida (§0).

⚠️ **E um gate meu afirmava o OPOSTO da lei do módulo e falhou sobre código CORRETO:**
*"colocar algo dentro de um sensor com o relógio desarmado não é uma chegada"* — a lei
declarada do canal irmão é a leitura da **Unity** (*a baseline nasce vazia e contínua ⇒ o
primeiro tick simulado relata o que encontrar*), e durante o `hold` a fase estreita está
**CONGELADA**. Substituído por `a_scrub_back_into_an_overlap_is_silent`, contra o qual a
mutação que tira o `discard_trigger_history` do `rewind` sangra.

---

## §5 — Verificação (rodada nesta árvore, pós-rebase)

| gate | resultado |
|---|---|
| `cargo test -p ph2d-physics -p ph2d-physics-ecs --release` | **verde**, 0 falhas |
| `cargo test -p ph2d-host-desktop -p ph2d-panel-inspector --release` | **1903 passed · 0 failed · 73 ignored** |
| `architecture_workspace_file_loc_cap` | **verde** (2) |
| `file_loc_caps` (shell) | **verde** (2) |
| `architecture_contract_surface` (nodegraph) | **verde** (3) |
| `architecture_tool_contract_surface` | **verde** (4) |
| `architecture_vector_contract_surface` | **verde** (11) |
| `clippy --all-targets --release` nas 4 crates tocadas | **limpo** |
| `physics_ecs_c9` | **99 corpos**, `16ba80e8…`, **debug ≡ release** |

⚠️ **O `c9` é byte-idêntico ao do `main`** — e isso é **afirmação, não sorte**: as sete waves
são **readouts** (contato, sinal, trigger), **autoria** (a peça editável, a alça) ou
**consumidores de leitura** (zona, massa-seed). **Nada entra no solver.** Se ele mudar na sua
árvore combinada, alguma outra linha mexeu na física — não esta.

---

## §6 — O que re-conferir se o `main` andar antes de você integrar

1. **`PROJECT_SCHEMA`** — esta linha **não o toca**, então um conflito ali é de **outra**
   linha. ⚠️ Confira mesmo assim que a **tripla-pin** (`48, 13, 13`) e o `const` concordam:
   quando dois lados escrevem o **mesmo literal** o git não conflita, e um bump evapora
   **calado**.
2. **`registers_every_physics_component`** — a contagem **25**. Se outra linha registrar um
   componente, o valor se **conta**.
3. **`shells/desktop/src/render_loop/mod.rs`** — o dreno dos sinais foi posto **depois** do
   `physics_bridge::dispatch` e dos `break_reports`. A **ordem é load-bearing**: drenar antes
   do dispatch publica os eventos do tick anterior.
4. **`shells/desktop/tests/every_physics_component_is_authorable.rs`** — `WRITERS` **6 → 7**
   (entrou `inspector_commits.rs`). Esta lista **falha alto** de propósito.
5. **`physics_smoke.rs`** — as cenas `70`..`73` e as quatro entradas em `PAUSED_SCENES`.
   Lista compartilhada: **só ADICIONE**.
6. **`crates/ph2d-flip/src/lib.rs`** — a linha acrescenta **nove linhas de doc-comment** (o
   degrau `v13` que faltava na escada). Se a `line/FLIP` mexer no mesmo doc, o resíduo é
   textual e o Mergiraf resolve; **o const não é tocado**.

---

## §7 — Aberto (não bloqueia; nada aqui é dívida desta jornada)

- **Explosão sem torque** · **campo de atração arremessa para fora de quadro** · **Rigid e
  Rope atravessam parede** — todos **medidos e nomeados** em jornadas anteriores, todos
  decisões de produto.
- **Um Ctrl+Z para as duas metades do bake** — não é mecânico: são duas pilhas com roteamento
  próprio, e a cura mora no **roteador de undo** do editor, outro domínio.
- **O consumidor de gameplay do sinal** — o publicador existe e o **toast é o consumidor v1**.
  Um script ou um marker de timeline reagindo ao mesmo outbox é a próxima porta, e ela é
  **cross-line**.
- **Falloff dentro da área** · **arrasto de área por região** · **skew no frame da zona** —
  família das zonas, nomeados com o motivo.
- O horizonte do [`02_plano_joints_ui_authoring.md`](../02_plano_joints_ui_authoring.md) §8
  (IK multibody · params keyframáveis · preset Wheel · copiar-colar entre cenas) segue
  **não escalonado**.

---

## §8 — Smokes (todos `--release`, todos APROVADOS pelo Enio)

```
env PH2D_PHYSICS_SMOKE=69 cargo run -p ph2d-host-desktop --release   # a peça EDITADA
env PH2D_PHYSICS_SMOKE=70 cargo run -p ph2d-host-desktop --release   # A CHAVE E A FENDA
env PH2D_PHYSICS_SMOKE=71 cargo run -p ph2d-host-desktop --release   # O SENSOR DE PÉ
env PH2D_PHYSICS_SMOKE=72 cargo run -p ph2d-host-desktop --release   # A JANGADA COMPOSTA
env PH2D_PHYSICS_SMOKE=65 cargo run -p ph2d-host-desktop --release   # a hachura + as DUAS alças
env PH2D_PHYSICS_SMOKE=73 cargo run -p ph2d-host-desktop --release   # A PORTA (door 1 · bell 1 · quiet 0)
```

⚠️ Toda cena **imprime o que montou**. Se a linha de resumo não aparecer, **pare** — o resto
do smoke não significa nada.

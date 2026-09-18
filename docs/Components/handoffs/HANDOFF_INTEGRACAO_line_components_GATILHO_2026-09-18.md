# HANDOFF DE INTEGRAÇÃO — `line/components`, **O GATILHO** (suplente #24) · 2026-09-18

> **Para o INTEGRADOR.** A linha está FECHADA e **não** integrou nem pushou (§0.7). Este documento é
> o que a fusão precisa: os contadores como DELTA, a superfície de colisão medida, o que só a árvore
> COMBINADA pode reprovar, e as leituras do diff que se invertem.

---

## §1 — O que a jornada entrega, em uma frase

**A mão de quem joga passa a ser um produtor de sinal**, e **a cópia que uma fábrica faz nascer sai
apontada para onde a fábrica aponta**. Compostas com o `#11` (fábrica), o `#12` (ciclo de vida) e o
`#14` (projéctil), as duas dão a arma inteira — com alcance, ricochete, arco e higiene já pagos.

Plano: [`docs/Components/18_plano_gatilho.md`](../18_plano_gatilho.md).

---

## §2 — Os CONTADORES, como DELTA contra o `main` (⛔ nunca o literal)

⚠️ **Conte o delta contra a árvore em que vai aterrar.** A coluna `base:` do `collision-surface.sh`
é o **merge-base** e está desactualizada por construção a partir da 2.ª fusão de uma rodada.

| contador | DELTA desta linha | onde |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** | [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) + a **escada** + a **tripla** dos `_tests` |
| registo do `ph2d-ecs` | **+1** | `crates/ph2d-ecs/src/scene/registry.rs` |
| espelho do `ph2d-render` | **+1** | ele conta `ecs + render` |
| espelho do `ph2d-script` | **+1** | ele conta `ecs + script` |
| `LIVE_SECTIONS` | **+1** | `crates/ph2d-panel-inspector/` |
| `any_live_section` (o array de flags) | **+1** | idem |
| `SignalOrigin` | **+1** (`Action`, **append-only**) | `crates/ph2d-runtime/src/lib.rs` |
| `EditorAction` | **+1 e −11** (ver §4) | `crates/ph2d-editor-core/src/action_bus.rs` |
| ADR | **0** | esta linha não cria ADR ⇒ fora de toda disputa de número |
| contrato congelado (§6) | **intocado** | conferido pelo `collision-surface.sh` |
| pacote externo novo | **0** | o `+ph2d-hud` do `Cargo.lock` é da jornada anterior desta MESMA linha |

⭐ **Um componente registado novo e um degrau só** — o `SignalOnAction` é o único tipo novo, e ele
**não tem runtime** (§3).

---

## §3 — A lei, e a AUSÊNCIA que é a decisão

```rust
pub enum ActionEdge { Press, Release, Hold }         // a POSIÇÃO é a tag serializada
pub struct ActionTriggerRow { action: String, edge: ActionEdge, signal: String }
pub struct SignalOnAction(pub Vec<ActionTriggerRow>);   // REGISTADO (config)
```

⛔⛔ **Não há `SignalOnActionRuntime`, e não há entrada no [`rewind_runtime`]** — ao contrário das
CINCO irmãs registadas desta linha (`Timer` · `Factory` · `CounterWatch` · `StateMachine` ·
`GameCamera`). *A aresta já é trabalho do INPUT:* a `ph2d_input::ActionState` guarda um tique atrás
de propósito (é o que paga o `just_pressed` dela), logo um estado vivo aqui seria **a segunda
resposta** a *«ela já estava premida?»*.

⚠️ **Sem gate, essa ausência lê-se como esquecimento** — o report que o `#14` pagou quando o
`projectile_state` ficou fora do `rebuild_from_rest`. ⇒ `o_gatilho_nao_guarda_estado_e_isso_e_declarado`,
um censo textual sobre os DOIS ficheiros, com piso de população **medido depois de a prosa ser
despida** (ele reprovou na 1.ª corrida sobre produto CERTO, porque o doc-comment que EXPLICA a
ausência era lido como a presença dela).

---

## §4 — ⚠️⚠️ O que este diff MEXE em ficheiro PARTILHADO (leia antes de fundir)

### 4.1 — `EditorAction` perdeu ONZE variantes e ganhou UMA

O tecto de LOC do [`action_bus.rs`](../../../crates/ph2d-editor-core/src/action_bus.rs) ficou
**vermelho por acumulação desta linha** (`709/700`; o `main` tem `687`). ⇒ cura por **CORTE**, e o
corte é o que o irmão `action_bus_hier.rs` já pagou em 2026-09-01, uma família adiante:

```rust
// ANTES: onze variantes soltas, todas `{ entity_bits: u64, edit: <X>FieldEdit }`
EditorAction::InspectorTagsEdit { .. }  … InspectorActionTriggerEdit { .. }
// DEPOIS:
EditorAction::InspectorComponentEdit { entity_bits: u64, edit: ComponentEdit }
```

⭐ **A fronteira da família é MEDIDA e não é o calendário:** o enum tem **27** variantes com essa
forma exacta; as **11** que saíram são as que carregam um vocabulário de `crate::<x>_edits` (o
módulo que a catraca do DAG obriga a existir). As outras **16** carregam um
`crate::screens::hero::*FieldEdit` e **ficam onde estão** — são edições do modelo de SPRITE e de
AUTORIA. *O que as separa é o assunto.*

⛔ **Para quem funde:** um `EditorAction::InspectorXEdit` de OUTRA linha, se a família dele for um
`<x>_edits`, deve entrar como um braço de [`ComponentEdit`] e **não** como variante nova do
`EditorAction` — senão o ficheiro volta ao tecto na rodada seguinte. Ficheiros tocados: **19** (os
11 `event_*.rs` do painel, o `sync_particles.rs`, o `fase_bus_inspector.rs` da shell, 6 gates de
costura e o doc-link do `ph2d-panel-tags`).

- `action_bus.rs` **709 → 628** · novo `action_bus_component.rs` (54 L).
- `fase_bus_inspector.rs` **570 → 561** — os onze braços viraram **um**, com um `match` interior de
  uma linha por secção. ⚠️ A 1.ª redacção deixou os onze braços e o `rustfmt` reflow-os para seis
  linhas cada: `+47 LOC` **para os mesmos onze destinos**, e o ficheiro foi a `617/600`. *Um corte
  numa ponta que engorda a outra não é um corte.*

### 4.2 — A secção LIFECYCLE saiu do ficheiro da FACTORY

`sections/factory.rs` foi a `622/600` ao ganhar a linha da MIRA. ⇒ `sections/lifecycle.rs`
(**622 → 464** + 181). ⭐ O cabeçalho do ficheiro **já dizia** que os sujeitos são outros (*«a
`Factory` vive em quem fabrica; a `Lifetime` e o `DestroyOutside` vivem na RECEITA»*) — o corte é
por responsabilidade, e a lei já estava escrita. O `warn` (o pintor de uma linha de aviso, gémeo do
da câmera) fica com o primeiro dono e é `pub(super)`.

### 4.3 — `SignalOrigin::Action` é APPEND-ONLY, e um `match` de outra crate reprovou

⚠️ O [`one_outbox_many_readers.rs`](../../../crates/ph2d-runtime/tests/it/one_outbox_many_readers.rs)
tem um `match` sobre `SignalOrigin` **sem `_ =>`, de propósito** — é o que torna uma origem nova um
erro de compilação. Ele reprovou e foi estendido. **Quem trouxer uma 15.ª origem paga o mesmo.**

### 4.4 — A `Factory` ganhou um campo

`aim_from_spawner: bool`, **append-only** e `false` de fábrica; `Birth::aim: Option<f32>`.
⚠️ `Option` e **não** um `f32` com neutro: `0` é um ângulo legítimo (apontar para a direita).

---

## §5 — O que só a árvore COMBINADA pode reprovar

1. **O `PROJECT_SCHEMA`.** Outra linha da rodada escreve um degrau no mesmo dia ⇒ o desta **renumera**.
   ⭐ Recontar é por SCRIPT: `python3 scripts/schema-recount.py` (a âncora da escada e a da tripla
   **não** são iguais, e a tripla CONTÉM o número).
2. **Os TRÊS contadores de registo.** Os dois espelhos (`ph2d-render`, `ph2d-script`) **não são
   editados por esta linha** e mexem-se na mesma — é a cegueira que já mordeu **quatro** vezes.
3. **Os tectos de LOC**, que são a única grandeza que SOMA entre linhas sem ninguém a contar. Esta
   linha entrega o `action_bus.rs` a **628/700** e o `fase_bus_inspector.rs` a **561/600** — folga
   de propósito, porque a rodada de 17/09 gastou uma corrida de portão inteira nisto.
4. **Os 30 censos de texto do HR-15** (`bash scripts/censos-da-arvore-combinada.sh`, DIRETRIZ §1.5.9
   item **5-bis**): um literal de UI novo reprova na SOMA, e o **CI não os corre**.

---

## §6 — A prova de fecho

| portão | resultado |
|---|---|
| `cargo fmt --all --check` | ✅ |
| clippy `-D warnings` (crates tocadas) | ✅ |
| `scripts/nextest-impacted.sh` | **15 116 testes** — os vermelhos curados abaixo; os 2 que sobram são flakes de carga |
| `scripts/censos-da-arvore-combinada.sh` (depois do `git rebase main`) | ✅ **90/90**, com o controlo do filtro a dizer `8 de 8 censos correram` |
| provas de mutação | **21/21 sangram** — `docs/Components/ferramentas/mutacao_gatilho_2026-09-18.sh` (as duas últimas são do report do dono, §8-bis) |
| gates novos | 6 (lei) + 1 (mira, com controlo) + 4 (ponte do painel) + 4 (costura, clique REAL) + 6 (cena) + 3 (fiação da shell) + **2 (a TECLA, com controlo positivo)** |

### 6.1 — O que a varredura apanhou, e que nenhum `check` da linha vê

| vermelho | causa | cura |
|---|---|---|
| `architecture_panel_loc_cap::panel_functions_under_loc_cap` | `sections/factory.rs::factory_body` a `215/200` | ⇒ `onde_rows` (o selector + a área + a tag + o sorteio respondem à MESMA pergunta) |
| `fn_loc_caps::shell_functions_respect_their_ceiling` | `fase_inspector_commits` `201/200` · `fase_signal_outbox` `205/200` | ⇒ `fase_inspector_commits_audio.rs` (a fronteira já estava no comentário: **duas naturezas**) e a porta `App::accoes_do_quadro` |
| `one_outbox_many_readers` não compilava | o `match` sobre `SignalOrigin` **não tem `_ =>`, de propósito** | arm novo — *é o gate a funcionar* |
| `o_indice_cabe_no_orcamento_do_carregador` | o `MEMORY.md` a `22 198` contra `22 000` | três ganchos desceram para o `reference_topic_*` deles (`−426`) |

### 6.2 — ⚠️ DOIS membros NOVOS da família de flakes de carga (`CLAUDE.md` §5.0), a promover

As duas assinaturas completas, **medidas**:

| gate | crate | o que ele mede | diff da linha ali | sozinho |
|---|---|---|---|---|
| `measure_input_cost::the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` | `ph2d-tool-painter` | RAZÃO de dois relógios (`pen-down 7,15 ms` contra `copiar o canvas 36,92 ms`, `0,19×`) | **ZERO linhas** | **3 de 3 VERDE** a `load 13,99` |
| `text_path_smoke::perf::riding_the_path_costs_about_twice_the_straight_layout` | `ph2d-host-desktop` | RAZÃO de dois relógios (`8,91×` contra a barra de `~2×`) | **ZERO linhas** | **3 de 3 VERDE** a `load 8,50`–`8,76` |

⚠️ **As duas reprovaram no meio de um fan-out de 15 116** e nenhuma reproduz sozinha — a assinatura
que o §5.0 nomeia. ⭐ E o `pen-down` **não voltou a reprovar na segunda varredura**, que é a prova
direta de que o discriminador é o fan-out e não o commit.

⛔⛔ **TRÊS mutações correram vermelhas na 1.ª passagem, e as três eram defeitos MEUS de régua:**

1. **`Release => just_pressed` SOBREVIVEU** — o gate afirmava *«o `Release` fala UMA vez por
   toque»*, e a lei mutada também fala uma vez, **no quadro errado**. *Uma régua que conta QUANTOS
   nunca vê QUAIS.* ⇒ hoje afirma-se o PERFIL do toque, quadro a quadro.
2. **A guarda de igualdade do `signal` SOBREVIVEU** — o gate cobria `Action` e `Edge` e deixava o
   terceiro braço sem régua. ⇒ os três, com CONTROLO nos três.
3. **A 7.ª mutação NÃO COMPILAVA** (lida pelo arnês como *«filtro vazio»*): inserir a struct antes
   da `SignalOnAction` rouba-lhe o `#[derive(Component, …)]`.

---

## §7 — ABERTO, e de quem é cada item

| item | de quem |
|---|---|
| **A acção não se cria a partir da secção** — a linha diz se o nome é órfão, e quem cria a acção é o *Input Map*, noutra janela. O botão *«criar esta acção»* escreve no `HeroScreen`, e a secção do Inspector só fala com o mundo | wave própria |
| **Nenhum gesto de canvas cria um gatilho** (ele entra por *Add Component → Trigger*) | produto |
| **Sem `once`** — a `ActionEdge::Press` já é *uma vez por toque*, e um *«uma vez por vida»* não teve quem o pedisse | §5.0: não construir antes do consumidor |
| **Sem repetição automática** — a composição já a dá (`Hold` + o `Timer` do `#2`) | ⛔ recusa medida |
| O *override* de mapa por-jogador em `~/.ph2d/` | aberto do Input Map desde 24/08 |

---

## §8 — Sete leituras do diff que se invertem

1. **`SignalOnAction` não ter runtime NÃO é esquecimento** — é a lei da wave, e tem censo (§3).
2. **`aim_from_spawner` nascer `false` NÃO é timidez** — escrever a rotação sempre partiria toda
   fábrica que já existe (uma chuva cujas gotas nascem viradas para onde o emissor calhou estar), e
   o gate tem as duas metades.
3. **As onze variantes que saíram do `EditorAction` NÃO foram apagadas** — mudaram de casa, e a
   fronteira da família é o vocabulário, não a data (§4.1).
4. **A torreta da cena estar rodada `90°` NÃO é estilo** — no neutro, *«herda a rotação da fábrica»*
   e *«fica com a do molde»* dão a MESMA imagem e o controlo não controla nada.
5. **As duas armas ouvirem a MESMA tecla é o desenho** — duas teclas não ensinariam nada: o dono não
   saberia se o que mudou foi a mira ou o gesto.
6. **O prólogo criar a acção `fire` NÃO é conveniência de cena** — o `with_player_defaults` tem sete
   acções e nenhuma é disparar (medido), e a lei CALA uma acção que o mapa não conhece. Sem esse
   passo o dono lê *«o gatilho não funciona»* sobre um componente correcto.
7. **A cerca do relógio NÃO é uma optimização** — sem ela cada espaço que o artista escreve num
   campo publica um sinal. *As teclas do jogo são as teclas do editor.*

---

## §8-bis — ⛔⛔ O REPORT DO DONO, e o que ele mudou (2026-09-18)

> *«espaço é o atalho do play da timeline e há conflito. Mas o smoke está OK. Funciona.»*

**A feature está aprovada; o que estava errado era a TECLA que o smoke liga** — e a minha afirmação
ao lado dela (*«a tecla que ninguém do editor usa no canvas»*), que era falsa e nunca foi medida.
`KEY_SPACE if !cmd => GraphKey::TogglePlay`: um toque parava a corrida **e** disparava.

⇒ a acção passa a ligar o **`Q`**, com a medição, a proveniência e o gate no
[§4-bis do plano](../18_plano_gatilho.md). **21/21 mutações sangram** (eram 19; as duas novas são
o ESPAÇO de volta e o `P` do menu radial).

⚠️⚠️ **E há um efeito colateral que o INTEGRADOR deve conhecer:** correr qualquer smoke reescreve o
`~/.ph2d/layout.txt`, que vive **fora do repositório**. As minhas corridas deixaram lá
`active=nodes` — e com o layout *Nodes* a ferramenta MOTION é a dona do canvas, que é **o que parte
a banda do centro** (a correcção que este mesmo handoff regista no §4 do HUD). O ficheiro foi
**reposto** em `active=drawing_2d`, que era o valor que a 1.ª foto mostrava.

⛔ **Isto não é desta wave e atinge TODA cena de smoke:** o doc do
[`hero::layout_switch::install_at_startup`] já o mede por escrito (*«com o layout `Nodes` gravado, a
cena dos ossos pegava o vetor e o dreno trocava-o … vale para TODA cena que escolhe uma
ferramenta»*). Nenhuma cena do TOP-20 fixa o próprio espaço de trabalho — **item aberto, de quem for
dono do assunto**, não desta linha.

## §9 — O SMOKE (o que o dono vai correr)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_TRIGGER_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

A cena abre com o relógio **a andar**, a acção `fire` ligada ao **`Q`** (§8-bis) e o Inspector à frente.
⚠️ Se a linha `[trigger-smoke]` não aparecer no terminal, **a cena não montou — PARE**.

# Handoff de integração — `line/runtime` R0: a saída de sinais

**Status:** FECHADO 2026-08-08 · no `main` em `0be47bc6a` (o commit que trouxe este arquivo).

> **A saída de sinais existe: os dois produtores publicam nela, e o consumidor deixou de ser um
> toast escrito à mão duas vezes.** Pendente de smoke. A linha NÃO integra e NÃO faz ship.

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/runtime` |
| HEAD | `c9dd7de11` |
| Base (merge-base com `main`) | `a4018d203` |
| Commits | **3** |
| Arquivos | 13 (`+931 / −9`) |

---

## 2. O que foi construído, e o que **não** foi

**Construído (R0, e só ele):**

- **Crate nova `ph2d-runtime`** — o núcleo de eventos: `Signal` · `SignalOrigin` · `EntityBits` ·
  `SignalOutbox` · `SignalReader`.
- Os **dois produtores** publicam nela: a timeline (ADR-0143) e a física (`SignalOnHit`).
- **Um dreno**, depois dos dois, com **dois consumidores**: o toast (que fica, por ordem) e um
  diagnóstico atrás de `PH2D_SIGNAL_LOG=1`.
- Cena de smoke **`PH2D_SIGNAL_SMOKE=2`** — as duas fontes na mesma saída.

**⛔ NÃO construído, de propósito:** a shell `shells/game` (R1 — o Enio adiou: *"por enquanto o
play do editor"*) · os consumidores de áudio/Luau/UI (R3) · qualquer tabela de ligação
*nome → ação* · qualquer schema.

**⚠️ E o `ph2d-script::messaging` NÃO foi tocado.** Ele é o primitivo certo para o consumidor
**Luau** do R3 (entidades scriptadas se endereçando, o modelo do Defold) e o errado para fan-out
de subsistemas do host — os dois coexistem, e a razão está no doc do módulo da crate nova.

---

## 3. Foundational / compartilhado tocado

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-runtime/**` | **crate NOVA** (glob member ⇒ zero edição de `Cargo.toml` central) | novo |
| `shells/desktop/Cargo.toml` | +1 aresta de `path` | **sim** |
| `shells/desktop/src/app_state.rs` | +3 campos no `App`, no fim do bloco da timeline | **sim** |
| `shells/desktop/src/main.rs` | +3 linhas no construtor, na mesma posição | **sim** |
| `shells/desktop/src/render_loop/mod.rs` | **3 hunks** (o `advance_frame`; os 2 `for` de toast viram `publish`; o dreno) | 2 substituem, 1 acrescenta |
| `shells/desktop/src/signal_smoke.rs` `_tests.rs` | o router `=1`/`=2` + a metade da física | **sim** (o `=1` não se move) |
| `shells/desktop/tests/the_signal_frame_has_one_order.rs` | arch-gate novo | novo |
| `Cargo.lock` | a aresta da crate nova | **sim** |

⚠️ **O `CLAUDE.md` NÃO foi tocado, de propósito.** Ele é o arquivo mais disputado do repo (seis
linhas ativas, e a §5 já exigiu edição cirúrgica numa integração anterior). A entrada de §5 desta
wave sai deste handoff, escrita pelo integrador junto com as das outras linhas da janela.

⚠️ **O ponto de merge sensível é o `render_loop/mod.rs`**, e ele é estreito: os dois hunks que
substituem são exatamente os dois `for … toasts.push(Toast::info(format!("Signal: …")))`. Uma
linha que edite **outra** parte daquela função funde limpo; uma que edite **aqueles dois laços**
conflita, e a resolução certa é manter o `publish`.

---

## 4. Símbolos que podem COLIDIR (o que grepar)

**Nomes novos:**

- crate `ph2d-runtime` · tipos `Signal` `SignalOrigin` `SignalOutbox` `SignalReader` `EntityBits`
- campos do `App`: `signals` · `signal_toast_reader` · `signal_log_reader`
- env var **`PH2D_SIGNAL_LOG`** · nível de cena **`PH2D_SIGNAL_SMOKE=2`**
- `fn smoke_level` (privada em `signal_smoke.rs`)

**⚠️ Colisão de VOCABULÁRIO, não de símbolo — vale o grep do integrador:**

- **`ph2d-tool-runtime` já existe** e não tem nada a ver com isto (helpers de `RasterEditTool`,
  Wave 10 / ADR-0041). Duas crates com "runtime" no nome; os `use` não colidem.
- **`ph2d-audio::bus` já existe** e é *roteamento de áudio*. Foi por isso que este tipo se chama
  `SignalOutbox` e não `SignalBus` — "outbox" é a palavra que o próprio ADR-0143 usa.

⚠️ **E a colisão que importa é com uma linha VIVA, não com o passado** (achado depois de a linha
fechar, 2026-08-08): o [plano de UI/UX](../../Vector%20Module/Estudos/PLANO_UI_UX_padrao_figma.md)
aponta o nome **`ph2d-runtime` para o runtime de UI** — a §1.2 dele lista *"Runtime que toca UI sem
editor | `ph2d-runtime` não existe (Front 2 não construída)"*, e a W8a diz *"é aqui que este plano
encontra a Front 2 (`ph2d-runtime`)"*. A `line/Vector` está a **80 commits** do `main` com a W6.3 e
a W7 construídas (`ph2d-ui-state`, `ph2d-ui-codegen`), e a W8a é a próxima.

**Hoje não há conflito** — nenhuma das duas crates dela menciona esta (conferido: são folhas de
zero dependências) e o nome está livre no `main`. O que existe é uma **expectativa**: quem abrir a
W8a vai procurar um runtime em `ph2d-runtime` e vai achar um canal de eventos.

**A decisão é do Enio, e são duas saídas:** ou a `ph2d-runtime` CRESCE para hospedar o runtime de
UI — e aí o gate `the_event_core_has_no_dependencies_at_all` tem de ser **deliberadamente
revogado**, com o preço medido na §8 (8 consumidores = 1,00× o custo de 2, sem arrastar VM nem ECS)
— ou o runtime de UI nasce numa crate irmã e esta fica sendo só a saída de sinais.
**Recomendação da linha: a segunda.**

**⚠️ `docs/Runtime/` não existe no `main`** — ele é criado por esta linha **e** pela `line/Vector`
(que carrega o `00_plano_runtime.md`, o `01_o_formato_medido.md` e o handoff de R0). Os **nomes de
arquivo diferem**, então o git não conflita; as duas metades convivem no mesmo diretório.

**NADA disto foi tocado:** nenhum id numérico · nenhum token · nenhuma chave i18n · nenhum
variant de enum compartilhado · nenhuma lista ordenada · **nenhum schema** (`PROJECT_SCHEMA`,
`FLIP_SCHEMA`, `VEC_SCENE_SCHEMA`, `DOC_VERSION`, `SCULPT_DOC_VERSION` intocados — um sinal é
evento de quadro e não é persistido por nada) · registro do `ph2d-ecs` intocado · **nenhum ADR**.

---

## 5. Contratos congelados (§6)

**NENHUM encostado.** `NodeOp` / `OpResolver` / `NodeManifest` e `Tool` / `RasterEditTool` /
`CanvasPaintTool` / `PanelEvent` seguem intactos — esta linha não tem uma `Tool`, não tem um nó,
e não publica painel.

---

## 6. O que só o `ship.sh` pega

- **Nenhum pacote externo novo.** `ph2d-runtime` tem **zero dependências** (gate estrutural
  `the_event_core_has_no_dependencies_at_all` lê o `Cargo.toml` dela), e a única aresta nova do
  `Cargo.lock` é a própria crate ⇒ **nada para `deny`/`audit`/RUSTSEC**.
- **machete:** a aresta `ph2d-runtime` do shell **é usada** (`app_state.rs`, `main.rs`,
  `render_loop/mod.rs`).
- **fmt:** rodado com `rustfmt --edition 2024` nos arquivos tocados.
- **clippy `--workspace --all-targets --all-features -D warnings`: exit 0, zero saída** (rodado
  na árvore da linha, não auto-relatado).
- **typos:** não rodado pela linha (é do `ship.sh`).

---

## 7. Mudança de comportamento — **UMA**, cosmética, e nomeada

⚠️ **A ordem dos toasts dentro de um quadro mudou.** Antes: *sinal da timeline* → *aviso de joint
que partiu* → *sinal da física*. Agora: *aviso de joint que partiu* → *os dois sinais*.

É consequência direta de haver **um dreno só** (ele roda depois dos dois produtores, e o
`break_reports` fica entre eles). Só é observável quando um joint parte **no mesmo quadro** em que
o play cruza um marker; a ordem *entre* os dois sinais não mudou (timeline, depois física).

---

## 8. Números MEDIDOS (não herdados)

`cargo test -p ph2d-runtime --release --test measure_the_channel -- --ignored --nocapture`:

| sinais/quadro × consumidores | µs/quadro | % de um quadro de 60 fps |
|---|---|---|
| 0 × 2 (o caso comum: nada acontece) | 0,003 | 0,000% |
| **4 × 2 (o produto: 2 markers + 2 contatos)** | **0,054** | **0,000%** |
| 32 × 2 (cena movimentada) | 0,588 | 0,004% |
| 256 × 2 (centenas de contatos marcados) | 7,945 | 0,048% |
| **256 × 8 consumidores** | **7,694** | 0,046% |
| 4096 × 2 (para achar a forma da curva) | 125,7 | 0,754% |

**⚠️ O número que decide o R3: `8 consumidores / 2 consumidores = 1,00×`.** O custo mora no
**PRODUTOR** (a alocação do `Arc<str>` do nome, ~39 ns/sinal); ler é um cursor a percorrer um
`Vec`. Consequência para a resposta *"todos"* do Enio: **áudio + Luau + UI custam
aproximadamente nada**, e o trabalho de verdade do R3 é a **tabela de ligação nome → ação**, que
é conteúdo autorado e precisa de UI.

**⚠️ A latência sinal→consumidor não tem número porque não se mede com relógio:** os dois
produtores e o dreno rodam em linha reta na mesma função ⇒ **zero quadros por construção**, e
quem prova isso é o gate `both_producers_land_in_one_outbox_in_one_frame` mais o arch-gate de
ordem do shell.

**⚠️ E o alvo herdado foi recusado com motivo:** o `ph2d-script::messaging` declara num
doc-comment *"100.000 mensagens/quadro em ≤ 1,5 ms (HR-4)"*. Aquele número **nunca foi medido
neste repo** — a crate não tem `benches/`, o módulo não tem um único `#[test]`, e o único teste
que existe é um proptest de determinismo de intern. Herdá-lo seria carregar uma aspiração como
se fosse um piso.

---

## 9. Gates e mutações

**21 gates**: 10 na `ph2d-runtime` + 1 de contenção + 1 sonda com 2 asserções + 4 no
`signal_smoke` + 1 arch-gate de ordem no shell.

**9 mutações, 9 sangram:**

| # | Mutação | Gate que sangra |
|---|---|---|
| M1 | `read` não avança o cursor até o presente | re-entrega: 3 gates |
| M2 | `advance_frame` sem duplo-buffer | o consumidor que corre cedo demais |
| M3 | sem contabilidade de perdidos | o consumidor que dorme |
| M4 | `read` nunca pula o já-visto | re-entrega: 2 gates |
| M5 | `SignalReader::at` não começa no presente | o leitor ligado no meio |
| M6 | a folha ganha uma dependência | `the_event_core_is_a_leaf` |
| M7 | o quadro vira DEPOIS do 1º produtor | o arch-gate de ordem |
| M8 | o dreno roda ANTES do produtor da física | o arch-gate de ordem |
| M9 | a cena `=2` sem o relógio da física | o gate da cena |

⚠️ **O gate que carrega a wave não afirma um número — ele COMPILA**
(`a_consumer_reads_the_outbox_while_mutating_its_own_state`): um consumidor escreve em
`self.toasts` **dentro** do laço que lê `self.outbox`. Trocar o `&self` do `read` por `&mut self`
não falha uma asserção, falha o `cargo check` — e é exatamente a coisa que um `Box<dyn FnMut>`
guardado dentro do barramento torna impossível.

---

## 10. O SMOKE (o que julgar)

```
env PH2D_SIGNAL_SMOKE=2 PH2D_SIGNAL_LOG=1 cargo run -p ph2d-host-desktop --release
```

⚠️ **Se a linha `[signal-smoke 2]` não aparecer, PARE:** a cena não montou e o resto não diz nada.

1. **Quatro toasts, um canto.** A timeline grita `footstep`@1,0 s e `beat`@2,5 s; a física grita
   `door` (a bola atravessa o sensor rosa) e `bell` (a bola bate na plataforma âmbar). A terceira
   plataforma, cinza, é o **CONTROLE** — sem componente, e ela não grita.
2. **No terminal, a origem de cada um.** `[signal] footstep <- timeline @ 1.000s` ·
   `[signal] door <- fisica, A tocou B`. Esse log é o **segundo consumidor**, com cursor próprio:
   ele imprime enquanto os toasts sobem por um cursor **diferente**, e nenhum dos dois consome o
   sinal do outro.
3. **A cena `=1` tem de continuar igual** (`env PH2D_SIGNAL_SMOKE=1 …`) — ela é a cena aprovada da
   timeline e esta wave não a moveu.
4. Pause (Space): nada dispara. Arraste a régua para trás: nada dispara.

---

## 11. Aberto, nomeado

- **A tabela de ligação `nome → ação`** é o R3 inteiro, e é conteúdo **autorado** (precisa de UI).
  Sem ela, um consumidor de áudio/Luau/UI é uma demo hard-coded. Medido acima: o canal não é o
  custo; a autoria é o trabalho.
- ⚠️ **O envelope por SEÇÕES (F1.W0) não existe no `main`, e a W8a da `line/Vector` depende dele.**
  Ela diz, verbatim, que *"o documento de UI é uma **seção** do envelope que a `line/runtime`
  construiu (F1.W0), não um segundo formato"* — mas aquele envelope vivia na `line/runtime`
  **antiga**, destruída por ordem do Enio em 2026-08-08 sem nunca ter integrado. Medido: nenhum
  `SectionKind` / `LEGACY_SCHEMA_FINAL` no `main`, e a crate `ph2d-project-format` que ele criava
  **não existe** lá.
  **É recuperável**: o commit `37ff53467` sobreviveu à remoção da branch (`project_envelope.rs` 556
  linhas + 277 de teste + o gate `the_shell_carries_sections_it_does_not_understand`). ⚠️ Mas o
  **desenho** volta e o **diff não** — ele reescrevia o `project.rs`, que desde então andou de
  `PROJECT_SCHEMA` 48 para **55**. Reconstruir contra o `main` de hoje é wave própria, desta linha,
  e **não foi começada** (a linha está fechada).
- **R1 (`shells/game`)** segue adiado por decisão do Enio. ⚠️ O argumento estrutural do plano
  (*"o runtime é uma SHELL, não uma feature"*, vindo da feature unification RFC 3692) está
  **suspenso, não resolvido** — e é por isso que o núcleo NÃO foi enterrado no `render_loop`
  (8430 linhas, editor-only): quando `shells/game` nascer, ele consome a mesma crate-folha.
- **`EntityBits` é `u64` cru** para a crate seguir folha. Bits são ids de ALOCAÇÃO e só valem
  dentro do quadro que publicou; está escrito no tipo.
- **O `missed` de um leitor não é lido por ninguém hoje.** Ele existe para que perda deixe de ser
  silenciosa; o primeiro consumidor que ficar para trás vai querer que alguém o mostre.

# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, o `motion.wave` com N PRODUTORES

**Data:** 2026-08-18 · **Branch:** `line/motion-value` · **Worktree:** `Worktrees/line-motion-value`
**Commits:** 5 (`49452031b` … `f0af132cb`) · **13 arquivos, +1.801/−261**
**Item:** folha [`06_animadores`](../89_conferencia/06_animadores.md) **linha 35** — o último `P0/P1` da conferência.

> ⚠️ **A linha NÃO integra e NÃO pusha** (CLAUDE.md §0.7). Isto é o handoff; a ordem é do Enio.

---

## 1. O que integrar, numa frase

O **`motion.wave` já exprime N produtores por COMPOSIÇÃO** — a célula que pedia
isso como omissão **envelheceu**, e a medição a derrubou. O que shipou é: o
veredito medido (`P0/P1 → P2`), **dois defeitos reais** que a medição achou pelo
caminho, a **cena `=57`** que mostra a composição, e a folha corrigida.

**Placar da conferência depois disto:** `P0 = 0 · **P0/P1 = 0** · P1 = 68 · P2 = 120 · ⏸ 18 · ✅ 121 · ⛔ 119`
(450 linhas, derivado por `docs/Motion Nodes/ferramentas/placar_conferencia.py`).

---

## 2. O trabalho, por commit

### `49452031b` — `fix(motion.wave)`: ausente não é zero

`drive_value` devolvia `0.0` quando **nenhum valor de fonte chegava**, e o passo
cravava a célula do centro nesse zero **incondicionalmente**.

⚠️ **Consequência medida:** todo campo dirigido pela cadeia de estado nascia com
um **BURACO no centro** — a célula central lia `+0,000000` EXACTO entre vizinhas
de `+0,062` e `+0,020`. *O pino de Dirichlet não é um valor, é a resposta a
"chegou uma fonte?"* ⇒ `Option<f32>`, e o pino corre quando um valor **chegou**,
não quando ele calha de ser zero.

- Uma fonte ligada em amplitude `0` **continua pinando** (o valor chegou).
- Uma porta solta **não pina nada**.
- **Byte-idêntico** para todo grafo que já shipava: **os 7 gates que existiam
  passam SEM uma edição de asserção**, e há gate provando que sobre um campo
  não-excitado as duas rotas dão o mesmo campo (cravar zero num campo já chato é
  a identidade).

⚠️ **Este defeito só nasceu com o Grupo P** (`motion.drive(Custom…)`, 16/08) —
antes dele não havia como um segundo produtor alcançar `wave_h`, então o pino
incondicional nunca era observável.

**LOC:** `lib.rs` cruzou 700 ⇒ `mod tests` para o irmão `lib_tests.rs` por
`#[path]` (segue **FILHO**, então `use super::*` alcança `step`/`drive_value`/`Params`).
408 + 305. **4 gates novos**, 11 no total.

### `db6b99856` — `test(conferencia)`: a célula 35 envelheceu

A folha marcava `P0/P1` com *"NÃO — dois `motion.wave` montam duas grades"*.
Ela é de **2026-08-10**; o Grupo P mudou o catálogo **seis dias depois**.
É a **sétima** célula desta conferência a envelhecer antes de alguém voltar a ela.

**Medido pela porta do produto ANTES de construir nada** (240 tiques, grade 21×21):

```
wave.out --pre--> field.box --> value.attribute("falloff")
  --> motion.drive(Custom "wave_h", Add) --> wave.state
```

- o berço das ondas move do centro (`x = -0,50`) para **exactamente a caixa** (`x = -3,00`);
- **440 das 441 células** mudam ⇒ o que o injector deposita **PROPAGA**, logo é
  fonte e não tinta;
- `wave_h` **não** está no `is_bookkeeping_column`, e é por isso que o canal
  `Custom…` o aceita (a auditoria de 16/08 pôs `id`/`sim_t`/`sim_d`/`dl_*` lá).

⚠️ **E a rota que parece óbvia — `wave A --> wave B.state` — é um NO-OP BIT-A-BIT:**
o `dt` chega em **zero** no segundo nó, o ramo de hold devolve a entrada verbatim,
e o drive de B (5× mais forte) é **engolido em silêncio**. Isso fica **GATEADO**,
não só em prosa: o fio é legal de fazer e o artista não recebe erro nenhum.

**Sonda** `measure_wave_producers` (`#[ignore]`, imprime e não afirma, 4 rotas) +
**gate** `wave_producers` (2 gates).

### `c586921fe` — `feat(cena =57)`

Duas bandas do MESMO campo; só a de baixo tem a cadeia. Em cima os anéis saem do
MEIO da grade, em baixo um segundo berço nasce à ESQUERDA e as frentes se cruzam.

⚠️ **O `scale = 0,25` do injector é MEDIDO, e a varredura REFUTOU o alvo óbvio.**
A tentação era casar a amplitude das duas bandas (o precedente do Grupo N: *se
elas diferirem, "a de baixo mexe mais" responde por qualquer coisa*) — mas abaixo
de ~0,25 o produtor injetado **nunca vira o berço**:

| `scale` | comp/ctrl | pico | peças > passo |
|---------|-----------|------|---------------|
| 0,20 | 0,97 | **+0,50** (o CENTRO) | 21 |
| 0,25 | **1,24** | **−3,00** (a CAIXA) | **18** |
| 0,60 | 3,61 | −3,00 | 62 |

⇒ 0,25 é o **menor** valor em que o berço se MOVE, e ali as duas bandas são
*comparáveis* (com **menos** peças estouradas que o próprio controle).
**Igualdade exacta é inalcançável por FÍSICA, não por afinação:** dois produtores
de mesma força deixam o pico com quem estiver mais alto naquele instante.
A barra do gate é `1,0..1,5`, e ⚠️ **ela não pode ser 1,0** — a varredura mostra
que ali o pico volta ao centro e a cena deixa de mostrar o item.

**6 gates + 1 sonda de cena · 5 mutações, 5 sangram** (a que põe o `scale` de
volta em 0,6 sangra **exactamente um**, o da comparabilidade).

⚠️ **LOC — e o primeiro corte estava ERRADO, fica registrado:**
`motion_state.rs` estava a **UMA linha** do teto de 600 no `main` (599) e as duas
linhas do módulo novo o cruzaram. Tentei mover o **manifesto de gates** para um
`mod gates` aninhado: **144 erros de compilação** (`unresolved import super::MotionState`,
`super::strobe`, …), porque os 12 módulos movidos fazem `use super::*` esperando
`super == motion_state`, e aninhá-los **muda o que `super` significa**.
Revertido; o corte que ficou é por **ASSUNTO**, seguindo o precedente que já mora
naquele arquivo (`motion_state_fixture.rs`): a família de **CLIPBOARD** do grafo
(`GraphClip`/`ClipNode`/`ClipSubgraph`/`ClipEdge` — NodeId-free, sem mencionar
`MotionState`) sai para `motion_state_clip.rs` **com re-export**, porque mover o
arquivo não pode mover o caminho de quem chama. **550 + 68.**

### `f0af132cb` — `docs`

A folha 06 linha 35 com o veredito, a cadeia, os números e o ⛔ do encadeamento;
`Contagem` reconciliada com a saída da ferramenta (`P1 = 12`).

⚠️ **E uma correção na §5 do CLAUDE.md que me custou tempo hoje:** ela dizia que
a linha de Contagem de cada folha é *"gerada"* pela ferramenta. Ela é **DERIVADA**
— o `placar_conferencia.py` **IMPRIME e sai vermelho**, `--write` **não existe**,
e quem reconcilia a linha é quem roda. Eu passei a flag, li a tabela e não conferi
o **ESTADO** da folha; a nota me convidou a isso.

---

## 3. Superfície de colisão — MEDIDA, não auto-relatada

| Item | Estado |
|---|---|
| `PROJECT_SCHEMA` | **84 INTOCADO** — `project.rs` **e** `project_schema.rs` **e** `project_schema_tests.rs` com diff **vazio** (⚠️ os três: a `line/physics` PARTIU aquele arquivo em 15/08, e um degrau escrito no antigo funde **limpo** e evapora) |
| Contrato congelado (§6) | **INTOCADO** — `git diff` vazio em `ph2d-nodegraph/src/node.rs` e `ph2d-core/src/tool.rs` |
| Registro do `ph2d-ecs` | **INTOCADO** ⇒ os **três** espelhos também |
| `Cargo.toml` / `Cargo.lock` | **ZERO** — nenhuma crate nova, **nenhuma dep externa nova**, nenhuma aresta interna |
| ADR | **nenhum** ⇒ a linha fica **FORA de toda disputa de número** |
| ids / scrollbar / i18n | **nenhum novo** |
| Cenas de smoke | **`=57`** (a `=56` era a última do `main`) ⇒ **próxima livre: 58** |

⚠️ **O número da próxima cena se CONTA lendo o `match` do
`motion_state_demo_router.rs`**, nunca uma nota — e ali um número repetido é
`unreachable pattern` do compilador, não uma cena inalcançável em silêncio.

### O ÚNICO ponto de merge sensível

O corte do **clipboard** em `shells/desktop/src/motion_state.rs`. Uma linha que
acrescente um `mod` ou toque no `GraphClip` **funde limpa** contra um arquivo de
onde a família saiu — a mesma classe do corte do `project.rs`. O caminho de quem
chama **não muda** (`motion_state::GraphClip` segue válido pelo re-export), então
o modo de falha é *o `mod` novo aterrar num arquivo que encolheu*, e não uma
quebra de compilação.

⚠️ **`main` está a ZERO commits do fork** ⇒ merge fast-forward trivial **hoje**.
Esta caixa **envelhece**: reconfira antes de integrar.

---

## 4. Gate de fechamento — rodado

| Gate | Resultado |
|---|---|
| `cargo fmt --all -- --check` | **limpo** |
| clippy nas 3 crates tocadas, `--all-targets -D warnings` | **zero output** |
| `ph2d-node-motion-wave` | **11/11** |
| `ph2d-node-registry-init` (`wave_producers` + suíte) | verde |
| shell (`--bins`) | **2684 passed, 0 failed, 178 ignored** — reproduzido em 4 corridas |
| `file_loc_caps` (shell, 600) | **2/2** |
| `every_demo_scene_ends_in_an_output_node` | **1/1** |
| `no_tofu_glyphs` · `architecture_panel_wiring_parity` · `architecture_workspace_file_loc_cap` | **1/1 · 2/2 · 2/2** |

⚠️ **UMA falha NÃO reproduzível, e ela fica escrita em vez de ser varrida:**
numa corrida combinada de 3 crates, sob carga dobrada, o resultado foi
`2683 passed; 1 failed` — **o nome não foi capturado**. Quatro corridas seguintes
deram `2684/0` com `load 2,43`. *Não a chamo de flake porque não a identifiquei;
chamo-a de não-diagnosticada.* Se ela reaparecer na árvore combinada, o nome é a
primeira coisa a guardar.

⚠️ **Rode `--ignored` com `--test-threads=1` e a máquina calma** — a sonda
`measure_wave_producers` cozinha 240 tiques × 4 rotas.

---

## 5. Mudanças de comportamento — nomeadas

1. **Um `motion.wave` sem fonte ligada não pina mais o centro.** Antes ele
   cravava zero ali; agora a célula central participa da onda como qualquer
   outra. Só é observável num campo que **outra coisa** excita — que é
   exactamente o caso que o Grupo P criou, e é o defeito curado.
2. **Cena `=57` é nova.** As `=1..=56` **têm de continuar iguais**.

---

## 6. O smoke — **APROVADO pelo Enio em 2026-08-18**

```
env PH2D_GPU_COOK_DEMO=57 cargo run -p ph2d-host-desktop --release
```

⚠️ **A cena imprime as duas bandas nomeadas; se a lista não aparecer, PARE.**
⚠️ **DÊ PLAY** — uma onda é uma forma no TEMPO, e uma foto de um instante mostra
dois campos ondulados sem dizer *onde eles nascem*.

**A leitura, e ela é de ONDE, não de QUANTO:**

- em cima, os anéis saem do **MEIO** da grade;
- em baixo, um segundo berço nasce à **ESQUERDA** (na caixa) e as duas frentes se
  **cruzam** no caminho;
- ⚠️ **se a de baixo só parecer mais agitada**, o injector virou um segundo
  controle de amplitude e o item não está demonstrado — o oráculo é o **pico**
  (`x = −0,50` contra `x = −3,00`), não a energia.

---

## 7. Aberto, com o preço ao lado

- ⏸️ **O gesto.** O veredito `P2` é literal: *não falta capacidade, falta o
  GESTO*. Hoje são **quatro nós e três arestas à mão**, e o artista tem de saber
  que a coluna se chama **`wave_h`** — um nome de **ESTADO** que **nenhum picker
  oferece** (o `value.attribute` lista os canais VIVOS do stream, e `wave_h` só
  existe depois de o campo cozinhar). Fechar isto é UI (um preset de cadeia, ou
  o picker aprender colunas de estado), não motor.
- ⚠️ **O `#[ignore]` `the_ceiling_is_honoured_on_every_tick_including_the_turn`**
  (cena `=53`) segue aberto com o número e o mecanismo escritos — ⛔ **não
  afrouxe a barra**.
- ⚠️ **A composição sub-passos × `damping`** da `motion.verlet_rope` segue
  **MEDIDA e não curada de propósito**.
- **Re-smoke das cenas `=50..=53`** é decisão do Enio: as quatro correções da
  auditoria multiagêntica de 16/08 nunca passaram por smoke (integrar não é
  aprovar).
- **Folha 03: 6 P1 · folha 07: 3 P1.** ⚠️ **E a primeira coisa de toda wave desta
  conferência é MEDIR se a composição já exprime o item** — esta é a **sétima**
  célula a envelhecer: *o que se perde ao não reconferir não é tempo, é construir
  o que já existe.*

---

## 8. Checklist de fechamento (DIRETRIZ §1.5.9)

- [x] Commits locais, `--no-verify`, **sem push**
- [x] `fmt` + clippy + suítes das crates tocadas
- [x] Arch-gates de shell e de `editor-core` que a varredura impactada alcança
- [x] Superfície de colisão medida (§3)
- [x] Handoff escrito (este arquivo)
- [x] `rm -rf target/*/incremental`
- [x] **Smoke da cena `=57` aprovado pelo Enio** (2026-08-18)
- [ ] Ordem de integração do Enio ⇒ agente integrador dedicado (DIRETRIZ §1.5.3)
- [ ] **Integração e ship — ordem do Enio, por agente integrador dedicado**

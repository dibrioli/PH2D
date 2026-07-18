# Handoff de CONTINUAÇÃO — `line/physics` → W2b (o painel global de mundo)

> Para o **próximo agente**, em janela de contexto nova. Este documento existe para você **não
> re-pagar** o que já foi levantado. Leia nesta ordem: aqui → [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md)
> (estado técnico completo) → [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)
> (o *porquê*) → [`00_plano_waves.md`](00_plano_waves.md) §W2.
>
> **Não re-litigue o norte:** runtime-truth + bake opcional; rígido primeiro; o solver é o
> `rapier2d 0.28` que já existia. Esta linha escreve **integração e autoria**, não solver.

---

## 0. Onde a linha está

**W0 · W1 · W1.5 · W2a estão INTEGRADOS na `main`** (2026-07-18), com todo o smoke aprovado pelo Enio.
Worktree `Worktrees/line-physics`, branch `line/physics`, sincronizada com a `main` por fast-forward,
árvore limpa.

O que já funciona no produto: um sprite ganha corpo pelo Inspector e **cai**; Play/Pause/Reset dirigem a
sim; arrastar a régua pra trás re-simula bit-exato; os colliders têm contorno visível (tecla `B`).

**⚠️ TAREFA ZERO, antes do W2b — o roteador não sabe que a física existe.**
`CLAUDE.md` §5 tem **zero** referência a `docs/Physics/`, e o `docs/SESSION_ACTIVE.md` também não. Uma LLM
nova lendo o ponto de entrada do repo **nunca é roteada** para o ADR, o plano ou o tracker — três waves
integradas ficam invisíveis. Escreva a entrada de §5 no padrão dos módulos vizinhos (Timeline, Áudio,
Painter): o que landou, os gotchas que custam smoke, e os links. É barato e é o que faz o próximo achar
tudo isto.

---

## 1. O que o W2b entrega

Crate **`ph2d-panel-physics`** docada, **categoria MUNDO** (ADR-0131 D8): gravidade (vetor),
substeps/iterações do solver, damping global, sleep thresholds, matriz de camadas de colisão.

⚠️ **A escala do mundo NÃO entra aqui.** É `ProjectSettings.pixels_per_meter`, setting do **projeto** — o
painel **exibe, não duplica**. Uma segunda porta para o mesmo número diverge (foi a correção do D4 no W1).

**Boa parte já existe e o W2b só liga:**

| Já pronto | Onde |
|---|---|
| `set_gravity(x, y)` (e já invalida o checkpoint ring) | `PhysicsBridge` |
| `set_substeps` · `set_contact_frequency` · `set_contact_response` · `set_solver_iterations` | `PhysicsWorld` |
| `App.show_colliders` (o toggle da tecla `B`) | `shells/desktop/src/app_state.rs` |

⚠️ **O checkbox "Show Colliders" deve LER o `App.show_colliders`, não criar o seu.** Duas portas para a
mesma pergunta divergem — a tecla e o checkbox têm de discordar *nunca*.

---

## 2. O mapa de fiação de painel — **levantado, não re-derive**

Isto custou duas varreduras. Precedente canônico em tudo: **`ph2d-panel-vector`**.

### 2.1 Os 5 sites de registro
1. **`impl Panel`** em `src/lib.rs` — `type State`, `ID = "physics"`, `NODE_ID`, `DEFAULT_VISIBLE = false`,
   `paint`/`apply_event`/`populate`. ⚠️ O **nome da struct é load-bearing para o codegen**: o
   `ph2d-panel-sync` faz parse de `pub struct <Nome>Panel` e entra em pânico se não achar ⇒
   **`pub struct PhysicsPanel;`** exatamente.
2. **`ph2d-panel-registry-init`** — o push é **GERADO** por `cargo run -p ph2d-panel-sync` (3 regiões com
   marcador: o `lib.rs` e dois blocos do `Cargo.toml`). ⚠️ **Duas coisas o sync NÃO faz e você faz à mão:**
   a const `EXPECTED_TYPED` (hoje **19**, some 1) e a entrada na lista `default` do `Cargo.toml`.
3. **Feature Cargo** `panel-physics = ["dep:ph2d-panel-physics"]` (gerada) + a lista `default` (à mão).
4. **⚠️ A lista de fallback de z-order** em `screens/hero/paint.rs` (hoje linha ~341). **Sem a entrada, o
   painel fica registrado, visível e NUNCA é pintado — nada quebra, nada avisa.** Insira logo após
   `ids::TIMELINE_PANEL` e **antes** da cauda flutuante (`INSP_BLENDER_PICKER`/`GAL_PANEL`): o que vem
   depois pinta por cima.
5. **Visibilidade pelo shell** — `hero.panel_visibility.insert("physics", …)` num bridge do `render_loop`.
   O idioma de "tomo o slot do Inspector" é o edge-trigger com `static LAST_ACTIVE: AtomicBool` (veja
   `vector_bridge.rs`), mas **decida se o painel de mundo deve mesmo tomar o slot** — ele não é por-seleção,
   então talvez não deva.

### 2.2 Ids
Não há "próximo inteiro livre": os ids são **FNV-1a de um slug** (`hash_node_id("physics.panel")`), em
`crates/ph2d-editor-core/src/ids/chrome/<domínio>.rs` + `mod`/`pub use` no `chrome/mod.rs`.
⚠️ **Toda const nova entra na tabela à mão de `tests/node_id_collisions.rs`** (arrays: expanda elemento a
elemento, senão os membros não são checados entre si).
**A exceção é o scrollbar, que ainda é inteiro à mão:** `PHYSICS_SCROLLBAR_ID = NodeId(836)` —
**verificado LIVRE em 2026-07-18** (o topo ocupado é 835). Há também uma tabela de auto-checagem no
`widget/scrollbar.rs` para atualizar.

### 2.3 Se o painel rolar, são **4 edits e só 3 falham alto**
O `NodeId` do thumb, o braço em `scrollbar_panel_for_id` (`interaction/dispatch/scroll.rs`), o painter
publicando `content_h`/`visible_h`, **e** `|| inside(PHYSICS_PANEL)` em `cursor_over_hero_panel`
(`shells/desktop/src/forwarding.rs`). Esquecer o 4º: a roda **zooma a câmera** por baixo do painel em
silêncio. Há gate (`scrollable_panels_intercept_the_wheel`) e ele faz parse dos argumentos de `inside(`,
então importar o id não engana.

### 2.4 Gates que um painel novo precisa satisfazer
| Gate | Exige |
|---|---|
| `architecture_panel_wiring_parity` | todo `ids::X` passado a `.register(` em arquivo de *paint* tem de aparecer no `populate.rs`. ⚠️ Ele **não** vê registro dentro de laço. |
| `architecture_interactive_crate_has_behavioral_test` | painel com `event.rs` **precisa** de `tests/**` usando `ph2d_ui_testkit`. A lista de débito está **VAZIA** — não há isenção. |
| `architecture_panel_loc_cap` | 600 LOC/arquivo, 200 LOC/função. **Split, nunca allowlist** (é catraca: as allowances só encolhem). |
| `hr15_no_hardcoded_ui_strings` · `no_literal_color` · `no_magic_numeric` | tudo por `tr()` / `ColorToken` / tokens. ⚠️ O marcador `// LITERAL-PX-OK: <razão>` vai **na linha**, não acima do bloco. |
| `node_id_collisions` · `architecture_cycle_prevention` | ids únicos; painel→tool ok, tool→painel não (harness em `[dev-dependencies]`). |

### 2.5 i18n
A tabela inteira é **um `match` em `crates/ph2d-i18n/src/lib.rs`**. Convenção: `panel.physics.title`,
`panel.physics.section.<snake>`. Chave faltando devolve a própria chave (aparece na tela — é proposital).
⚠️ **O Inspector é a exceção do repo** (strings em inglês hardcoded, i18n migra depois); um **painel novo
não herda essa exceção** — passe por `tr()`.

---

## 3. Como esta linha trabalha (o que o Enio espera)

- **Meça antes de mexer.** Duas vezes nesta linha a medição matou a hipótese: o kill-check do ring (que
  apontou o stride para o lado *oposto* do Motion) e a interpenetração (nenhum knob de solver movia a
  profundidade — era `v × dt`). Não otimize nem "corrija" o que não mediu.
- **Todo gate nasce VERMELHO sobre o bug real e morre por uma razão nomeável**, com os números do
  PRODUTO. Depois **mute o código** e confirme que sangra — nesta linha um gate ficou verde sob uma
  mutação real (oráculo de endpoint num sistema amortecido) e só a trajetória o pegou.
- **Seam verde ≠ produto vivo.** Todo controle de UI precisa de um teste que o **CLICA** e afirma o
  efeito, e do lado de lá um e2e cujo oráculo é a aparência ("o sprite está no chão"), não "o componente
  existe".
- **Defesa em camadas ⇒ gate POR camada.** E defesa que você não consegue observar **remova** — nesta
  linha eu quase shipei um `ring.clear()` cujo comentário prometia proteção que ele não dava.
- **Fecha a wave → atualiza o tracker → PARA.** Integração e ship **só por ordem explícita do Enio**, via
  agente integrador dedicado. Nunca `git push`, nunca `git add -A`/`-a`/`stash`.
- Commits: `git commit --no-verify -F <arquivo>` (crase em `-m` é substituição de comando).
- Gate batched **1× no fechamento**: `fmt` (⚠️ **antes** de medir LOC — o rustfmt re-expande) · `clippy
  --all-targets` · `cargo check --workspace` · `bash scripts/nextest-impacted.sh`.

---

## 4. Armadilhas desta linha que já custaram caro

- **`PROJECT_SCHEMA` hoje é 18** (a integração **recontou** o 17 desta linha + o bump da FLIP). Se o W2b
  persistir qualquer coisa nova, o valor **se conta**, não se escolhe — e nenhum gate consegue ver um
  campo *apendado* a um componente, porque nenhuma constante muda. Postcard é posicional e devolve lixo
  bem-formado.
- **Um número de ADR escolhido numa linha paralela é PROVISÓRIO.** O 0130 desta linha virou **0131** no
  merge (dois donos). Cada referência em doc-comment é custo de rename.
- **Componente de física é CONFIG, nunca estado vivo de solver.** O `canonicalize` do undo ordena por
  bytes do componente ⇒ guardar velocidade/sleep ali faria **cada frame** virar um passo de undo.
- **A pose de repouso é a pose autorada no tick 0** — lida todo frame, não lembrada. Se o W2b acrescentar
  campos ao `Collider`, o `reconcile_structure` já os re-descreve no tick 0 de graça.
- **`dt()` é o TICK, `substep_dt()` é o do integrador.** Um nome, um significado.
- **Campo sem consumidor é órfão** (DIRETIVA §2): não adicione knob ao painel antes do fio que o lê
  chegar no rapier, no mesmo commit.

---

## 5. Depois do W2b

**W3 — Joints** (pino/mola/motor/distância; pêndulo, corrente, ragdoll). Bumpa `PROJECT_SCHEMA` **18 → 19**
+ a tripla-pin. **W4 — Bake-to-timeline** (a sim vira curva editável via `ph2d-anim::fit_fcurve`).
Fora de todas as waves (ADR-0131 D9): soft-body XPBD, fluidos, collider-gen vetorial.

**Aberto e conhecido, sem gate:** o `reconcile` de corpos obsoletos é O(N²) (trivial nos counts de hoje) ·
o `readback` só trata corpo raiz (corpo filho quer `parent_world_transform`) · `Kinematic` não existe.

---

## 6. Comando para começar

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics && cargo check -p ph2d-physics-ecs
```

Cenas de smoke vivas: `PH2D_PHYSICS_SMOKE=1` (queda) · `=2` (pilha + scrub) · `=3` (autoria no
Inspector). **A cena do W2b é a `=4`** — e renumere W3/W4 para `=5`/`=6` no plano quando chegar lá.

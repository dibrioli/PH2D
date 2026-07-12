# HANDOFF — continuação da linha `line/motion-value` (Motion Nodes)

**Data:** 2026-07-12 · **Para:** o **próximo agente-de-linha** (você) · **De:** o agente que fechou as 18 fatias
integradas em 2026-07-11/12 · **Modo:** **L** (worktree, DIRETRIZ §1.5 · [MODELO_ABERTURA_LINHA](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md))

> **A integração ANTERIOR está CONCLUÍDA.** Os 26 commits da jornada passada estão no `main` (`3805f650`) e a
> branch `line/motion-value` foi fast-forwardada até lá — **a linha está limpa, sincronizada e pronta pra
> continuar**. Este documento **supersede** o `HANDOFF_line_motion_value_integracao_2026-07-11.md` (aquele era
> pro integrador; já foi consumido).

---

## 0. ABERTURA DA LINHA (faça isto ANTES de qualquer coisa)

A worktree **JÁ EXISTE e já está sincronizada** — você **não** precisa criar nada (é a rota "linha reaberta" do
[MODELO_ABERTURA_LINHA](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) passo 4).

```bash
# 1. o hardware define o MODO (tem que dizer `workstation`; se disser `constrained`, PARE — Modo C proíbe linhas)
cd /home/enio/Documentos/Projetos/PH2D && bash scripts/hw-profile.sh

# 2. entre na SUA worktree (todo read/edit/git/cargo acontece AQUI DENTRO, sempre)
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
git branch --show-current          # DEVE imprimir: line/motion-value
git status -sb                     # DEVE estar limpa

# 3. rebase no main (início de TODA jornada) — se o main andou desde 2026-07-12
git -C /home/enio/Documentos/Projetos/PH2D fetch origin && git rebase main

# 4. warm-up (o target/ desta worktree é próprio; se estiver frio o 1º build demora — é ESPERADO, não investigue)
cargo check -p ph2d-eval-motion -p ph2d-panel-motion-params

# 5. leia INTEIROS, aqui dentro:
#    docs/IntegracaoMultiAgente/DIRETRIZ.md            -> §0, §1.5, §2, §6
#    docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  -> TUDO (e RELEIA a cada passo, como ela manda)
#    docs/Motion Nodes/01_plano_modulo_motion_nodes.md -> §3 (roadmap M0..M5)
```

### As regras permanentes da sessão (Modo L — valem até o fim, SEM exceção)

| | Regra |
|---|---|
| **A** | **TODO** read/edit/git/cargo acontece **dentro da worktree**. A raiz do repo é o checkout primário COMPARTILHADO e o **mesmo path relativo existe nas duas árvores** — editar `crates/...` na raiz é editar a árvore **ERRADA**. Na dúvida: `pwd`. **Mutação de arquivo = SEMPRE caminho absoluto** (um `mkdir -p crates/foo` relativo já vazou pro repo primário e quebrou o workspace do `main` — [[feedback_sed_relative_path_hits_primary_cwd]]). |
| **B** | Edite a pasta do módulo à vontade. **Foundational você PODE e DEVE tocar** (com cuidado) sob o protocolo testado (ADR-0107) — a integração roda `scripts/foundational-integrate.sh` + Mergiraf. **PARE e reporte ao Enio** só se: (a) for **contrato congelado** (CLAUDE.md §6 — exige ADR), ou (b) o rebase conflitar em código **FORA** dos seus arquivos. Nunca negocie com outra linha. |
| **B'** | Ao **CRIAR** foundational novo, projete pra **ISOLAMENTO**: módulo/arquivo **IRMÃO** novo > engordar arquivo compartilhado; ponto de extensão **append-only**. Todo id/const/variant novo: pegue o próximo livre e **ANOTE no handoff** (regra H) pro integrador detectar colisão. |
| **C** | Commits locais frequentes: `git commit --no-verify`. **NUNCA** `push`. **NUNCA** `--force`. **NUNCA** `git add -A`. |
| **D** | `git rebase main` no início de cada jornada. Conflito em `Cargo.lock` ou **arquivo GERADO** (`ph2d-node-registry-init`): **NUNCA** resolva na mão — **regenere** (`cargo run -p ph2d-node-sync`). |
| **E** | Fechamento = **gate batched** (§6 abaixo). Depois **PARE** — **NÃO integre nem faça ship**. Quem funde é um **agente integrador dedicado**, e só por **ORDEM EXPLÍCITA do Enio**. Você **não** roda `foundational-integrate.sh`. |
| **F** | **Ship (ship.sh + push + babysit CI): NUNCA por conta própria.** Integrar/pushar sem ordem = **violação do protocolo**. |
| **G** | **UI canônica sempre:** zero hex, zero `f32` literal de UI, zero string hardcoded — tokens/i18n (HR-15). **UI do app em INGLÊS.** |
| **H** | **Handoff de integração é entregável obrigatório** ao fechar (DIRETRIZ §1.5.9): branch/HEAD/base · foundational tocado + por quê · ids/consts/variants novos **com valores** · contratos encostados (deve ser **nenhum**) · o que só o `ship.sh` pega · o que smoke-testar. Reporte *"linha pronta + handoff"* e **ESPERE**. |

---

## 1. O método de trabalho desta linha (o Enio já ratificou — NÃO reinvente)

A cadência é: **o Enio diz "próximo" e VOCÊ escolhe a fatia.** Cada fatia:

1. **REGRA-OURO — pesquise ANTES de codar.** Pesquise (a) o **algoritmo padrão-ouro da indústria** e (b) o
   **melhor nome de nó** (Houdini / Cavalry / After Effects / Cinema4D MoGraph / Blender GN são as referências).
   **Porte por SEMÂNTICA, não por código.** Sem pesquisa = fatia rejeitada.
2. **2 drop-crates novas por fatia** (`crates/ph2d-node-<familia>-<nome>/`), dependendo **só** de
   `ph2d-nodegraph` + `ph2d-node-registry` (+ leaf copiado, se preciso — copiar um `curve.rs`/`hash.rs` de 60
   linhas é MELHOR que criar foundational novo pra 1 consumidor: [[project_brush_along_path_satellite_not_node]]).
3. **HR-5 — produção transcendental-free** (`sin`/`cos`/`exp`/`pow`/`atan2` proibidos; `sqrt` permitido). Use as
   aproximações polinomiais já existentes (`trig.rs` — parabolic sin/cos, Rajan atan2) — **copie o leaf**.
   Exceção documentada: `ph2d_expr::eval` é **HR-5-EXEMPT** por contrato próprio (presentation-side).
4. **Demo auto-playing pequena** em `shells/desktop/src/motion_demo_strobe.rs` — o documento Motion **default**,
   que dá boot já animando os nós NOVOS. Regra permanente do Enio: **"simplifique o exemplo"** — a demo isola os
   nós da fatia; não vire um monstro de 40 nós. E: **feature nova = exemplo pronto pro smoke**, nunca "monte você".
5. **Testes de integração FALSIFICÁVEIS** (não "compila = verde"): o teste tem que **falhar** se a costura
   quebrar. Prove a **CORRENTE INTEIRA** (source → …  → `motion.output`), não só o nó. Ver §5 (a armadilha que
   já mordeu).
6. **Nota-ADR numerada** em `docs/Motion Nodes/NN_<tema>_nota_adr.md` (a próxima livre é a **34**) — a pesquisa,
   a decisão, a superfície nova, o que fica aberto.
7. **Gate de fechamento completo** (§6) → **commit** → **PARE**.

---

## 2. Onde a linha parou (estado VERIFICADO em 2026-07-12, não é chute)

- **67 crates-nó** da família Motion no `main` (`ls crates/ | grep -E '^ph2d-node-(motion|value|pulse|force|falloff|adapt|color)'`).
- **M0 · M1 · M2 · M3: fechados** (inclusive **M2.N2** — `Cook::checkpoint`/`restore` + `CheckpointRing` no pump +
  `advance_or_scrub_scoped` = **scrub para trás bit-exato**; doc 11. *O CLAUDE.md listava isso como "aberto" — era
  informação obsoleta, já corrigida.*)
- **A conquista arquitetural da última jornada** (fatias 32/33, docs 32/33) — **memorize, é o padrão canônico**:
  o **canal de TEXT PARAM**. O `ParamSpec` é **f32-only** e o `NodeManifest` está **CONGELADO** (8 campos), mas
  os params **vivem no `Graph`, não no manifest** → deu pra abrir um `node_text_params: BTreeMap<NodeId,
  BTreeMap<String,String>>` **paralelo** (`Graph::set_text_param` + `EvalCtx::text_param` + record `x` / header
  `v2` no formato textual + `ParamWidget::Text` no painel) **sem encostar no contrato**
  (`architecture_contract_surface` **provado 8/2/1** depois da mudança). Isso destravou a `motion.expression`
  (parser VEX-lite → `ph2d-expr`, editável no painel).
  **➜ Se precisar de um param não-f32 (string, path, curva…), use ESTE padrão — NÃO bumpe o `NodeManifest`.**
  Ele **supersede o plano M4.N1** (que previa descongelar o contrato).
- **Verificado no HEAD atual** (`3805f650`, pós-integração + pós-fixes de ship):
  `cargo nextest -p ph2d-nodegraph -p ph2d-panel-motion-params -p ph2d-node-motion-expression -p ph2d-eval-motion`
  → **115/115 verde**; `-p ph2d-host-desktop -E 'test(motion)'` → **26/26 verde**.

---

## 3. AS ETAPAS PLANEJADAS (a fila — escolha a próxima fatia daqui)

Fonte: [`docs/Motion Nodes/01_plano_modulo_motion_nodes.md`](Motion%20Nodes/01_plano_modulo_motion_nodes.md) §3
(M3 tail · M4 · editor F2/F3). **Ordem recomendada** (do mais barato/isolado ao que exige decisão):

### ETAPA A — cauda do M3 (nós, self-contained · **a próxima fatia natural**)

| Fatia | Nós | Pesquisa obrigatória |
|---|---|---|
| **A1** | `motion.distribute_poisson` + `motion.pin_constraint` | **Poisson-disk = Bridson 2007** (dart-throwing O(N) com grid de aceleração; blue-noise). `pin_constraint` = fixar instâncias (por índice/falloff) contra `verlet_rope`/`soft_body`/`integrate` — o padrão é uma coluna `pinned` que o integrador respeita (Houdini: `pintoanimation`) |
| **A2** | `motion.slit_scan` + (decidir `motion.path`) | slit-scan = amostrar o campo em tempos DEFASADOS por instância (`t - i·delay`) — o efeito clássico de "varredura temporal". **`motion.path` precisa de DECISÃO:** o plano diz "integra vector.*", mas o sistema vetorial de nós foi **RETIRADO** (ADR-0108) — hoje a geometria vive em `ph2d-vec-scene`. Ler `VecScene` de dentro de um nó é **cross-module**: siga [[project_brush_along_path_satellite_not_node]] (crate satélite que só LÊ) **ou** defira e reporte ao Enio. **Não invente foundational pra 1 consumidor.** |

### ETAPA B — M4 FX por-instância (nós, self-contained)

`fx.mirror` · `fx.rgb_split` · `fx.drop_shadow` — **são operadores de STREAM** (duplicam/deslocam/tingem
instâncias), então cabem na cadência normal de 2 crates/fatia, **sem** compositor.
⚠️ **Os FX de PASSE** (`fx.glow`/`fx.bloom`/`fx.blur` dual-Kawase/`fx.vignette`/`fx.levels`/`fx.hue_shift`) são
**outra coisa**: exigem o compositor HDR (cross-module com `ph2d-painter-effects`) + `layer_fx` no documento →
**PARE e reporte ao Enio** antes de começar; isso é decisão de arquitetura, não fan-out.

### ETAPA C — M4 Rig (nós; exige a decisão M4.N3 primeiro)

`rig.skeleton` · `rig.fk` · `rig.ik_2bone` (lei dos cossenos) · `rig.fabrik` · `rig.rubber_hose` ·
`rig.skin_deformer`.
**A decisão (M4.N3):** o plano cogitava um `Domain::Rig` novo — **isso encostaria no contrato congelado.**
Antes de codar, **pesquise a alternativa isolada**: representar o esqueleto como um `Stream` normal
(`Domain::Instances`) com colunas `parent`/`rot`/`len` — aí rig é **fan-out puro**, zero ADR. Se (e só se) essa
representação apertar de verdade, **PARE e reporte**. (O `rig+skinning` LBS-port-do-Rive do módulo Vector é
**outro** sistema — não confunda.)

### ETAPA D — Editor F2 (o grafo usável; crate `ph2d-panel-motion-graph`, é nosso)

Hoje o `interact.rs` diz literalmente *"Duplicate / knife / probe land later"* e *"backdrops land later"* —
**estão TODO**. O maior desperdício da fila: temos **67 nós** e um editor pela metade.
- **Backdrops** (add/move/resize/rename) — **barato**: o `Backdrop` **já existe** em `ph2d-motion-doc` e o
  `[backdrop]` **já serializa**; falta **só a UI**.
- **Duplicate (Ctrl+D)** · **Knife** (cortar fios) · **Probe + sparkline** (ring de 60) · **smart-connect popup**
  (busca fuzzy dos compatíveis + auto-inserção de adapter) · **waypoints/branches** · **readouts inline no body** ·
  template "nó sequencial" (self-loop `pre` pré-ligado).

### ETAPA E — Editor F3 (polish "wow")

activity-fire (`value_hash` + glow + dash marchando + orbs) · influence (BFS por `AttrAccess`) · live-preview
flaps (throttle por `cook_epoch`, máx 4 LRU) · taper `variable_width_band` · gradiente interno em portas Field.

### ⛔ FORA DESTA LINHA (não enxerte aqui — mataria a integração limpa)

- **GPU / M5** ([`docs/plans/2026-07-gpu-resident-node-pipeline.md`](plans/2026-07-gpu-resident-node-pipeline.md)):
  o cook é CPU-single-thread; o plano (rayon → WGSL lowering → JFA/spatial-hash → renderer lê buffers GPU) exige
  **linha foundational DEDICADA** (`line/cook-parallel`, depois `line/gpu-nodes` **com ADR**, porque a Fase 1
  descongela o contrato). **É ordem do Enio abrir.** O que esta linha cria herda a GPU de graça na Fase 2.
- **Dock da timeline** (`motion_timeline_slot` · relógio único `MotionTransport` ← `Playhead`, W4.T4/T7):
  **coordena com a linha `anim`** — não é decisão unilateral.
- **Keyframes de Motion:** **deferidos** até a timeline ([memória](../project-memory/project_motion_keyframes_deferred_timeline_integration.md)).

---

## 4. Fan-out de nó novo — o checklist mecânico (o que quebra se esquecer)

1. `crates/ph2d-node-<familia>-<nome>/{Cargo.toml, src/lib.rs}` — molde: qualquer nó recente
   (`ph2d-node-motion-collide` é um bom espelho; `ph2d-node-debug-wave` é o template oficial).
2. **`MANIFEST`** (`NodeManifest` const) + `impl NodeOp` + **`pub fn register(reg: &mut NodeRegistry)`** com
   `register_ui` (`NodeUiManifest`: display_name **inglês**, `NodeUiCategory`, `NodeSilhouette`) +
   `register_param_ui` (`&[ParamUiHint]` — label inglês, min/max/step, `ParamWidget`).
3. **`cargo run -p ph2d-node-sync`** → regenera `ph2d-node-registry-init`. **É o ÚNICO conflito de merge esperado
   no rebase** — sempre **regenere**, nunca resolva à mão.
4. Adicione a crate ao `Cargo.toml` do workspace se o glob não pegar; `cargo check -p <crate>` (inner loop —
   **nada de test/clippy por task**).
5. Ligue na demo (`motion_demo_strobe.rs`) + teste falsificável em `motion_state_tests.rs`.

---

## 5. Gotchas que JÁ custaram caro nesta linha (leia — são cicatrizes, não teoria)

- **A costura não-testada** ([[feedback_painter_inefficiency_4_causes]]): esquecer de ligar a **saída** do nó novo
  até o `motion.output` faz o grafo **VALIDAR** (nenhuma edge inválida) e cozinhar **0 instâncias**. Só um
  `assert_eq!(pos.len(), N)` pega. **Sempre asserte a CORRENTE INTEIRA.**
- **Tipo de porta descasado mata a cena inteira, em silêncio:** um `out` `INST_VEC2` ligado num `in` `VALUE`
  reprova a validação e o doc de boot inteiro vira sinks vazios. `VALUE = PortType::new(Domain::Instances,
  Dim::Scalar, Clock::Frame)` (coluna `"v"`), `INST_VEC2 = (Instances, Vec2, Frame)`.
- **Gate `no_tofu_glyphs`** escaneia **string literal** (não comentário): um `→` dentro de uma mensagem de
  `assert!`/`expect()` **reprova**. Comentário e doc podem. (`×` é permitido.)
- **HR-5 no sweep pega até TESTE:** `.powi(2)` reprova — escreva `dx*dx + dy*dy`.
- **Caps de LOC dos painéis:** arquivo ≤ **600**, **fn ≤ 200** (`architecture_panel_loc_cap`). O `cargo fmt`
  **re-expande** → **formate ANTES de medir**. Estourou? **Extraia módulo irmão** ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]),
  nunca allowlist. (Foi assim que nasceram `rows_paint.rs` e `text_rows.rs`.)
- **`rustfmt` avulso quebra no `cook.rs`** ("let chains are only allowed in Rust 2024") — o binário puro assume
  edição 2021: use **`rustup run 1.95 rustfmt --edition 2024 <arquivo>`**.
- **UI = espelhe o widget existente, nunca improvise chrome** ([[feedback_ui_source_of_truth_gallery_inspector]]).
  O campo de texto da fórmula custou **zero** linha de teclado porque reusei o `TextInput` + o dispatch global
  (`WidgetEvent::{TextChanged,Submit,Blur,Cancel}`). Antes de construir widget: **procure o precedente**.
- **Painel: botão pintado ≠ botão vivo** — falta o `register` no `populate.rs` e o clique é dropado
  ([[feedback_panel_populate_register]]).
- **`Cargo.lock` / `registry-init` em conflito: REGENERE.** Nunca à mão.

---

## 6. Gate de fechamento da fatia (rode TUDO; é 1× por fatia, não por task)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value

# contrato congelado — TEM que dar NodeManifest=8 / NodeOp=2 / OpResolver=1
cargo nextest run -p ph2d-nodegraph -E 'test(architecture_contract_surface)'
# codegen em dia (staleness) + os arch-gates do workspace/painel
cargo nextest run -p ph2d-host-desktop -E 'test(staleness) or test(no_tofu) or test(loc_cap) or test(clamp) or test(no_magic) or test(wiring_parity)'
# os testes de verdade
cargo nextest run -p ph2d-nodegraph -p ph2d-eval-motion -p ph2d-panel-motion-params -p ph2d-node-<seus nós>
cargo nextest run -p ph2d-host-desktop -E 'test(motion)'
# clippy no fim (não por task)
cargo clippy -p <suas crates> --all-targets -- -D warnings
```

⚠️ **O gate da linha NÃO é o `ship.sh`.** O ship (fmt / clippy `--all-targets` do workspace / machete / deny /
audit / typos) roda **só na integração** e **sempre acha latentes** — é esperado, o integrador absorve
([[project_integrator_ship_catches_latents_budget_iterations]]). **Não rode ship. Não pushe.**

**Smoke (o Enio roda, não você — dê o comando pronto com o `cd` junto):**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
```

---

## 7. Ao fechar a jornada

1. Nota-ADR (`docs/Motion Nodes/NN_...`) de cada fatia — a próxima livre é a **34**.
2. **Handoff de integração NOVO** (regra H / DIRETRIZ §1.5.9), no molde do
   `HANDOFF_line_motion_value_integracao_2026-07-11.md` (está no `main`, use como forma): branch/HEAD/base ·
   foundational tocado · ids/consts/variants novos **com valores** · contratos encostados (**deve ser nenhum**) ·
   o conflito mecânico esperado (`ph2d-node-registry-init` → `node-sync`) · o que smoke-testar.
3. Reporte **"linha pronta + handoff"** ao Enio e **PARE**. **Ele** abre o agente integrador.

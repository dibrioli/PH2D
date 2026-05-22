# Handoff + Loop de operação autônoma — PH2D sistema de nós

**Data:** 2026-05-22
**Para:** a LLM que vai construir o sistema de nós sozinha, em loop, enquanto o Enio está fora.
**Como usar:** §0 e §1 são INSTRUÇÕES que governam seu comportamento. §2+ são referência. Leia tudo + o contrato (`crates/ph2d-nodegraph/src/`) + o briefing de fan-out antes de tocar em código.

---

## 0. MANDATO (lê isto primeiro — governa tudo)

> **Padrão-ouro. Puro-sangue. O definitivo. Sem economias, sem gambiarras.**

- Construa **a melhor versão que existe**, não um "v1 que dá pro gasto". Se a forma definitiva é viável agora, é ela que você faz.
- **Proibido:** corner-cut disfarçado de v1; `unwrap`/falha silenciosa onde cabe `Result`/erro real; assumir paridade/correção sem prova; `TODO: depois` em coisa que dá pra fazer certo agora; copiar-colar com gambiarra em vez de extrair a abstração certa.
- **Determinismo (HR-5)** respeitado onde aplica; documentado onde é isento. **Contratos minúsculos e gateados.** **Toda superfície pública documentada.** Testes cobrem feliz + edge + a classe-de-bug.
- O Enio confia em você. A barra é "melhor que Unity/Godot em 2D". Gambiarra é "obrigado, mas não".

---

## 1. O LOOP (o protocolo — siga fase a fase, sem parar até precisar de smoke)

Para CADA fase do plano ([`docs/plans/2026-05-node-waves.md`](plans/2026-05-node-waves.md)):

1. **Escolha a próxima fase** do plano. Atualize o todo list.
2. **Build isolado:** sempre `CARGO_TARGET_DIR="$PWD/target/slot-coord" cargo ...` (não contende no lock do `target/`).
3. **Implemente no padrão-ouro** (§0). Cite o princípio no código quando ajudar a próxima LLM.
4. **Auto-verifique, tudo verde:** `cargo test -p <crate>` + `cargo clippy -p <crate> --all-targets -- -D warnings` + `cargo fmt -p <crate> -- --check`.
5. **AUDITE — adversarial e independente.** Lance **≥2 auditores em paralelo** (Agent, `general-purpose`), cada um com lente distinta (corretude/edge-cases · paridade/determinismo · consistência docs↔código · qualidade gold-standard). Instrua-os a serem **duros, caçar bugs/lacunas, dar severidade, NÃO validar por cortesia**. Sem limite de créditos.
6. **CORRIJA TODOS os achados** (Crítico→Baixo). **Nada adiado** — exceto follow-ups genuinamente não-bloqueantes, que você registra explicitamente no plano §follow-ups com justificativa.
7. **RE-AUDITE até erro zero.** "Sem achados" = de verdade nenhum, não "bom o bastante". Repita 5→6 quantas rodadas precisar.
8. **Commit** (`git commit --no-verify` em background; local; um commit limpo por fase). Atualize plano + todos.
9. **Próxima fase.** Volte ao 1.

### Quando PARAR (e só então)
- A fase precisa de **smoke visual** (`./play.command`) ou integração com o editor/app que roda na tela.
- Uma decisão de **FREEZE** do contrato (congelar = evento Coordenador-only).
- Mudança em **foundational genuinamente compartilhado** que exige coordenação humana.
- Ao parar: relatório curto pro Enio — o que ficou pronto, o que ele precisa olhar, e por quê.

### NÃO faça autonomamente
- **`git push` / CI** — é o "ship" de fim-de-jornada, sob ordem do Enio (ele acabou de shippar o W1). Acumule commits locais; só ele dispara o push.
- Tocar `editor-core`/`shells/*` sem necessidade clara (são foundational/visual).

---

## 2. Estado atual (TL;DR)

Engine virou **node-centric** (ADR-0030..0038). Modelo = **funil**: neck serial (contrato) → **FREEZE** → fan-out paralelo (N agentes, 1 node-crate cada, sem colisão).

**W1 (o neck) COMPLETO + SHIPADO** — contrato + registry + codegen + compute compartilhado + template + save verificado. Auditado 3× e remediado. **CI 9/9 verde cross-platform** (incl. replay-hash de determinismo nos 3 OS); push em main `0c0e934`.

**W2.T1+T2 (vertical Motion, parte HEADLESS) CONSTRUÍDO + verde, mas NÃO auditado ainda** — commits `8c6bb4f` (avaliador `ph2d-eval-motion`: grafo→`Vec<RenderInstance>`) + `74c8254` (3 nós reais `motion.{grid,transform,clone}` + teste de integração da vertical: grid→transform→clone→27 instâncias, sem GPU). Commits **locais, não pushados**.

> ### ⏯ PONTO DE ENTRADA (próximo agente — comece AQUI)
> Seu **primeiro passo** é o passo 5 do loop (§1) sobre o que já está construído mas não auditado: **AUDITE a vertical Motion** (`8c6bb4f`+`74c8254`: `ph2d-eval-motion` + os 3 `ph2d-node-motion-*`) com ≥2 auditores adversariais → **corrija TUDO até erro zero** → commit. **Depois** construa o arch-gate da membrana (W2.T3 parte headless) e **PARE em W2.T3** (view de editor + live-preview + smoke visual = precisam do Enio). Não pushe (ship é do Enio). Releia §0 (mandato) e §1 (loop) antes.

Tese: o objetivo "multi-agente sem colisão" **é** o sistema de nós, via **isolamento FBP** (nó = caixa-preta: portas tipadas + efeito, zero estado compartilhado). Adicionar feature = largar um crate isolado. Houdini é referência de poder, não design final (síntese: atributos Houdini + Fields Blender + compile-to-shader Unreal + UX Substance + **inventado: membrana de determinismo como tipo** + formato textual diffável).

Docs: [`Migracao/2026-05-node-centric-architecture.md`](Migracao/2026-05-node-centric-architecture.md) · [`Migracao/2026-05-foundational-parallelism-three-bottlenecks.md`](Migracao/2026-05-foundational-parallelism-three-bottlenecks.md) · [`plans/2026-05-node-waves.md`](plans/2026-05-node-waves.md).

---

## 3. Mapa de crates

| Crate | Papel | Tipos-chave | Deps |
|-------|-------|-------------|------|
| `ph2d-nodegraph` | **contrato** (leaf, dep-free, estável) | `PortType{domain,dim,clock}`, `Effect{Pure,Temporal,Stateful}`, `Graph` (acíclico + `pre`), `Stream`/`Column` (SoA), `Cook` (incremental), `NodeOp`/`NodeManifest`, `format`, `LoweringKind`/`ParamSpec` | — |
| `ph2d-expr` | "VEX/VOP": IR de compute por-elemento | `Expr`, `eval`, `to_wgsl`+`wgsl_prelude`, `eval_column`, `Bindings`/`StreamBindings` | ph2d-nodegraph |
| `ph2d-node-registry` | registry = `OpResolver` + colisão | `NodeRegistry`, `RegistryError` | ph2d-nodegraph |
| `ph2d-node-registry-init` | `register_all_nodes` **gerado** | — | registry + node-crates (gerado) |
| `tools/ph2d-node-sync` | codegen do wiring (bin+lib) | `scan_node_crates`, `splice_lines`, `render_*` | — |
| `ph2d-node-debug-const` | 1º nó (gerador Pure trivial) | `MANIFEST`, `register` | nodegraph + registry |
| `ph2d-node-debug-wave` | **template canônico** (input+param+Temporal+expr+golden) | idem + `eval_column` | nodegraph + registry + expr |
| `ph2d-eval-motion` (W2.T1) | avaliador do domínio Motion: cook→`Vec<RenderInstance>` (headless) | `evaluate_motion`, `lower_to_instances` | nodegraph + **ph2d-render** |
| `ph2d-node-motion-grid` (W2.T2) | generator (grid N×M no `P`) | `MANIFEST`/`register` | nodegraph + registry |
| `ph2d-node-motion-transform` (W2.T2) | modifier (scale+offset do `P`, passthrough) | idem | nodegraph + registry |
| `ph2d-node-motion-clone` (W2.T2) | cloner (multiplica stream ×count, ADR-0035) | idem | nodegraph + registry |

`workspace.members` é **glob** (`crates/*`,`tools/*`) → adicionar crate = zero edit central. ~85 testes no sistema de nós, todos verdes (parte de 1466 no workspace).

---

## 4. O contrato que um autor de nó vê (ADR-0031/0032)

```rust
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("<domínio>.<slug>"),     // FNV-1a estável
    name: "<domínio>.<slug>",
    inputs:  &[PortSpec { name, ty }],          // ty = PortType{domain,dim,CLOCK}
    outputs: &[PortSpec { name, ty }],
    effect:  Effect::Pure | Temporal | Stateful,
    clock:   Clock::Frame | Audio | Static | Event,
    params:  &[ParamSpec { name, default }],
    lowerings: &[LoweringKind::Cpu /* | Wgsl | Luau */],
};
impl NodeOp for X {
    fn manifest(&self) -> &'static NodeManifest { &MANIFEST }
    fn eval(&self, ctx: &mut EvalCtx) { /* puro: lê ctx.input(i)/playhead, emite */ }
}
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> { reg.register(Box::new(X)) }
```

- **Membrana:** só `Stateful` (gameplay) escreve `SimWorld` → só ele exige HR-5; `Pure`/`Temporal` isentos. `Graph::validate(&resolver)` aplica (não `connect`): recusa `Stateful`→pull, tipos incompatíveis, portas inexistentes.
- **`pre`:** feedback = aresta `delayed` (Lustre); `Cook::advance_tick(graph, ops, playhead)` cozinha as fontes de `pre`.
- **Math por-elemento:** `ph2d-expr` (`Expr` + `eval_column`). Paridade CPU↔WGSL provada (`fract`/`mix`/noise alinhados; `wgsl_prelude()`).

---

## 5. Como adicionar um nó (fan-out, pós-FREEZE)

Briefing pronto-pra-colar: [`IntegracaoMultiAgente/briefing-node-crate.md`](IntegracaoMultiAgente/briefing-node-crate.md). Exemplo: `crates/ph2d-node-debug-wave/`.
Fluxo: criar `crates/ph2d-node-<dom>-<slug>/` → MANIFEST+eval+register+golden → `cargo run -p ph2d-node-sync` → `cargo test -p ph2d-node-registry-init` (gate) → `cargo test -p ph2d-node-<dom>-<slug>`. Zero edit central, sem colisão.

---

## 6. Armadilhas (lições das auditorias — NÃO repita)

- **`gen` é palavra reservada (edition 2024).** Nunca nomeie identificador `gen` (use `generator`/`grid`).
- **`replace_all` de token curto corrompe** ("a**gen**ts"→"agridts"). Use padrões longos/únicos.
- **Membrana é por `Graph::validate`, não `connect`** (connect só garante estrutura).
- **`pre` sem consumidor forward:** a fonte ainda é cozida por `advance_tick` (bug C1) — não confie no cook do alvo alcançá-la.
- **Determinismo:** só `Stateful`/gameplay. `BTreeMap` (nunca iteração de `HashMap`), `to_bits` em chave. Transcendentais não bit-determinísticos → `Func::is_deterministic()` gateia o lowering Luau.
- **`NodeManifest` muda → ripple em TODOS os literais** (arch-gate de cap protege). Mudança de contrato = o FREEZE (raro).
- **Build por slot** sempre (Gargalo 1; `MAC_EXTERNO` 1.7Ti ≈ 20 slots).

---

## 7. Build / verificar / ship

```bash
CARGO_TARGET_DIR="$PWD/target/slot-coord" cargo {check,test} -p <crate>
CARGO_TARGET_DIR="$PWD/target/slot-coord" cargo clippy -p <crate> --all-targets -- -D warnings
cargo fmt -p <crate> -- --check
cargo run -p ph2d-node-sync && cargo test -p ph2d-node-registry-init   # wiring de nó
./scripts/ship.sh    # paridade-CI completa — SÓ no ship do Enio, depois: git push origin main + gh run watch
```

---

## 8. W2 — vertical Motion → FREEZE (a próxima fase do loop)

Prova o contrato inteiro num caminho real antes do fan-out. (Plano §W2.)
- **W2.T1 — avaliador motion** (`ph2d-eval-motion`) ✅ CONSTRUÍDO (headless, verde, `8c6bb4f`) — pull-no-playhead → `Vec<RenderInstance>`; o upload+draw na tela é smoke. **Falta auditar.**
- **W2.T2 — 3 nós reais** ✅ CONSTRUÍDO (verde, `74c8254`): `motion.grid` (generator) · `motion.transform` (modifier) · `motion.clone` (cloner = multiplicador de stream, ADR-0035). Vertical headless provada por teste de integração. **Falta auditar.**
- **W2.T3 — arch-gate da membrana** (rodar `validate` no load) + view de editor mínima + **live-preview**. ⚠ toca editor-core + **PARA aqui: precisa de smoke visual do Enio** (`./play.command`).
- **W2.T4 — 🔒 FREEZE** — congelar `ph2d-nodegraph`+`ph2d-expr`. **Decisão do Enio.**

**Ao retomar (ordem exata):** (1) **AUDITE** W2.T1+T2 (`8c6bb4f`+`74c8254`) com ≥2 auditores → corrija até erro zero → commit. (2) Construa o arch-gate da membrana de W2.T3 (parte headless: um teste que roda `validate` e recusa `Stateful`→pull num grafo de exemplo). (3) **PARE** — view de editor + live-preview + smoke + FREEZE são do Enio. Relatório curto a ele.

---

## 9. Fan-out (pós-FREEZE, paralelo)

Tracks (briefing §): mais nós Motion · Shader (→WGSL; precisa do avaliador shader + WGSL runtime em `ph2d-gpu`, hoje inexistente) · Sound (sync-dataflow → `ph2d-audio` greenfield) · Gameplay (blocos + node-programming → Luau, `ph2d-collision2d`) · ops de `ph2d-expr` · cook path (estático→asset) · ferramentas imperativas (ADR-0027).

---

## 10. Commits desta sessão

**Pushados (CI verde) — até `0c0e934`:** `9d6a7ec` substrato W1.T2 · `6489f70` remediação auditoria 1 · `35fd1f3` NodeRegistry · `751b974` registry-init+codegen+gate · `7b0306f` remediação auditoria 3 · `b4a98f3` ph2d-expr · `7d18519` NodeManifest params/lowerings · `a19a141` remediação auditoria expr · `e87edb0` template+briefing · `0c0e934` verificação W1.T1.

**Locais, NÃO pushados (próximo ship do Enio):** `ce8cd40` handoff playbook · `8c6bb4f` W2.T1 avaliador motion · `74c8254` W2.T2 3 nós + vertical headless. ⚠ W2.T1+T2 ainda **não auditados** (1º passo do próximo agente).

---

## 11. Follow-ups diferidos (não-bloqueantes; das auditorias)

- `would_cycle` O(V²) por `connect` — otimizar (BTreeSet) só se surgirem grafos grandes.
- **Param overrides por-instância:** hoje nós leem só o default do `ParamSpec`. Falta `NodeInstance.params` + `EvalCtx` expor params. Necessário quando autoria precisar editar valores.
- **Lowering Luau** + gate HR-5 — com o domínio gameplay.
- **Avaliador shader + WGSL runtime** em `ph2d-gpu` (hoje só wgpu safe API).
- **Paridade CPU↔GPU real** (device headless) — hoje coverage-gate + semântica fixada.

---

**Confiança:** o neck está sólido — auditado 3×, bugs corrigidos, ship.sh CI-clean. Rode o loop (§1) no padrão-ouro (§0), fase a fase, até o smoke do W2.T3. A engine cresce por adição de crate isolado, em paralelo.

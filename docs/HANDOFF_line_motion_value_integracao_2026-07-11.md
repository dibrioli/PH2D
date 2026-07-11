# HANDOFF de integração — linha `line/motion-value` (Math + Compare, doc 16) — smoke PENDENTE (Enio)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (= o tip ATUAL
  do main no momento da abertura → fork fresco, **sem drift pré-fork**).
- **Commit de FEATURE:** `9ba2f6ad` (*feat(motion): value.math + pulse.compare — 4ª fatia…*). Este
  handoff é o commit de docs no topo. **2 commits na linha** (feature + handoff).
- **Auto-contida:** sem dependência de outra linha → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot: +4ª cadeia (Rotation round-trip), 15→21 nós | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + contagem 15→21 + 1 teste novo + pre-loops 4→6 | shell, módulo Motion; baixo |
| `Cargo.lock` | só as 2 crates PATH novas | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado — esta fatia é
100% fan-out de nós + cena. (Contraste com a linha anterior, que mexeu em `cook.rs`.)

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` (+2) e `Cargo.toml` (+2) são **GERADOS** (região `<ph2d-node-sync>`). QUALQUER outra
  linha que adicione um nó conflita aqui.
- **Resolução — NÃO fundir à mão:** depois de juntar as árvores, rode **`cargo run -p ph2d-node-sync`**
  — regenera determinísticamente dos `crates/ph2d-node-*`; o gate `staleness` (em
  `ph2d-node-registry-init`) prova sync.

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- 2 crates-nó novas, tipos (string, namespaced, únicos): **`value.math`**, **`pulse.compare`**.
  Grep de colisão: `grep -rn '"value.math"\|"pulse.compare"' crates/`.
- pub consts locais: `value_math::VALUE`, `pulse_compare::{VALUE,PULSE}` (mirrors do tipo, não símbolos
  compartilhados). Coluna de stream nova `cmp_armed` (local ao stream do compare, sem registro global).
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos. **ZERO dep EXTERNA nova**
  (só path crates) → machete/deny/audit/RUSTSEC não mexem.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (`NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8) — rodado no
fechamento. Fan-out aditivo (caminho A). Sem ADR necessário.

### 5. O que só o `ship.sh` pega (o gate `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip atual do main (`1c7c9a22`), então fmt (style_edition 2024,
  pin 1.95)/typos batem com o main de hoje. ([[project_integration_prefork_lines_ship_drift]] não morde
  aqui.) Ainda assim rode **`ship.sh` completo** na árvore combinada.
- **`nextest-impacted` funciona** — a linha só ADICIONA crates (sem rename/delete), então o filterset
  não quebra (contraste com [[feedback_ship_parity_gaps_ci_only]], que mordeu no cutover que deletava
  crates). `advisory-db` local pode envelhecer → `ship.sh` roda audit fresco.
- fmt/machete/clippy `--all-targets`/HR-5/LOC/typos rodados verdes no fechamento com o toolchain pinado
  (§ "Gates" abaixo).

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → fast-forward limpo,
  depois `ship.sh`.
- **Se main moveu (outra linha integrou antes):** rebase em cima do main novo → esperar UM conflito
  mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}` (resolve com `cargo run -p
  ph2d-node-sync`, **não à mão**) + `Cargo.lock` (regenera). O `motion_demo_strobe.rs`/`motion_state*`
  só conflitam se outra linha também editar a cena Motion (improvável — cada módulo tem a sua). Depois
  `scripts/foundational-integrate.sh` (gate da árvore combinada) + `ship.sh`.
- **Smoke: PENDENTE (Enio, manual)** — headless já provado (`the_value_to_pulse_round_trip_ratchets_the_rotation`
  cozinha o registry real). Comando:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → a grade 4×3 mantém X (passo no beat) · Y (sample-and-hold) · Size (gradiente) e
  agora a **metade de cima dos quadrados CATRACA a rotação** (gira em degraus, dá a volta em 90°) num
  ritmo mais rápido (~0.5 s) que o beat, enquanto a metade de baixo fica parada. No editor: dropar um
  `value.math` entre dois campos (ex.: `instance_field × lfo`) e um `pulse.compare` depois de qualquer
  valor (LFO/math) → um pulso quando o valor cruza o limiar (com histerese Rise/Fall/Both).

**Resumo:** *Linha `motion-value` pronta (feature `9ba2f6ad`, fork em `1c7c9a22`, 2 commits). Aditiva:
2 crates-nó (`value.math`, `pulse.compare`) + a 4ª cadeia da cena boot (Rotation round-trip). Único
conflito mecânico = codegen `registry-init` → `ph2d-node-sync`. Zero substrato, zero contrato congelado,
zero dep externa. Smoke pendente (Enio). Aguardo ordem de integração.*

---

## 0. O que a linha entrega (fatia 4 do domínio de valor — doc 16)

Fecha os 2 follow-ups nomeados que sobravam dos docs 12/13/14 §5, pesquisando o padrão-ouro ANTES de
codar (DIRETIVA §1). Detalhe técnico + a pesquisa: [`docs/Motion Nodes/16_math_compare_nota_adr.md`](Motion%20Nodes/16_math_compare_nota_adr.md).

- **`value.math`** (crate `ph2d-node-value-math`, tipo `value.math`): o **1º combinador de DOIS campos**
  de valor — 6 ops (Add/Subtract/Multiply/Divide/Min/Max) via param `op` enum, **exercendo a regra de
  broadcast 1→N entre dois campos** (até aqui só o `motion.drive` a exercia, contra o stream). UM nó
  multi-op (TD Math CHOP / Cavalry Math / Nuke Merge convergem nisso), não uma explosão por-op.
  Divide guardado (`|b| < 1e-9 → 0`, nunca `inf`/`NaN`). HR-5, `Effect::Pure`.
- **`pulse.compare`** (crate `ph2d-node-pulse-compare`, tipo `pulse.compare`): a **ponte valor→pulse
  genuína** (dual do `sample_hold`) — Schmitt com 2 thresholds (`rise`/`fall`) + `edge` (Rise/Fall/Both).
  Núcleo Schmitt portado **verbatim** do `pulse.threshold`; a ÚNICA diferença é o domínio de entrada
  (o campo `v` vs um canal de transform) — por isso os dois **coexistem sem duplicar**. Sequencial
  (`cmp_armed` no `pre` do porto `state`), `Effect::Pure`.
- **Cena boot — a 4ª cadeia (Rotation round-trip):** `instance_field × lfo_g → math → compare →
  counter_r → rot_range → drive_rot`. O domínio de valor contínuo alimentando o discreto e voltando,
  na tela (o ganho que os docs 12–14 mapearam como o fechamento do vocabulário).

## 1. Gates no fechamento (paridade §7)
- **Unit:** `value.math` 7 + `pulse.compare` 7 (falsificados dos 2 lados).
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bins motion` = **27 passed**
  (inclui o teste novo `the_value_to_pulse_round_trip_ratchets_the_rotation` + os 6 anteriores +
  motion_bridge params). `ph2d-eval-motion` + membrane gate verdes.
- **Contrato:** `ph2d-nodegraph --test architecture_contract_surface` = 3 pass (NodeOp=2/OpResolver=1/
  NodeManifest=8, intacto).
- **Registry:** `ph2d-node-registry-init --test staleness` = 2 pass (em sync após `node-sync`).
- **Lint/estilo:** `clippy --all-targets` (2 crates + registry-init + shell `--bins`) = 0 warnings ·
  `cargo fmt` pin 1.95 (style_edition 2024) rodado (crates novas + 3 arquivos do shell) · `typos` 0 ·
  `cargo machete` 0 nas 2 crates · sweep HR-5 (`\.(sin|cos|tan|atan2|exp|ln|log|powf|powi)\b`) = 0 na
  produção das crates novas.
- **LOC:** `value.math` 380 / `pulse.compare` 351 (cap 700); shell `motion_demo_strobe.rs` 373 /
  `motion_state_tests.rs` 424 / `motion_state.rs` 125 (cap 600). Todos folgados.

## 2. Follow-up restante (doc 16 §5)
- **`value.switch`/`gate`** — roteia um de N campos por seletor (o último utilitário do vocabulário de
  valor mapeado nos docs 12–14). Com Math + Compare o núcleo do domínio de valor está **completo**.
- **Utilitários do M2:** `motion.delay` · `pulse.on_change` — os últimos do M2 antes do **M3**
  (distribuições avançadas + deformers, doc 01 §3).

*"Linha `motion-value` pronta (feature `9ba2f6ad`, fork em `1c7c9a22`, 2 commits). Handoff acima.
Aguardo ordem de integração."*

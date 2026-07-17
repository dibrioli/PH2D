# HANDOFF de INTEGRAÇÃO — `line/Painter` (2026-07-17)

> **Para o agente INTEGRADOR** (DIRETRIZ §1.5.9). A linha está **FECHADA e PARADA**: integração e ship só
> por **ordem explícita do Enio** (§0.7) — que já a deu para esta rodada. O implementador **não** integra,
> **não** pusha, **não** monitora CI.
>
> **Ordem do Enio (2026-07-17):** *"melhor integrar ao main antes de continuar"* — banir no main tudo que já
> está aprovado, ANTES do próximo passo (a troca do kernel do Inflate pela bola limitada, que **NÃO** está
> nesta integração — é linha nova; ver §6).

---

## 1. Base e forma da integração — **fast-forward limpo**

- **Base:** `main` = **`12ccaecd`**.
- **`git merge-base main HEAD` = `12ccaecd`** = o próprio tip do main ⇒ **o main NÃO divergiu**. A linha está
  estritamente à frente por **52 commits**, sem divergência ⇒ **`git merge --ff-only line/Painter` é um
  fast-forward trivial, sem conflito possível**. (Se o main andou desde este handoff, rebase primeiro; hoje
  não andou.)
- **Escopo:** 88 arquivos, +12866 / −1094.
- **Tip da linha:** **`bf3e18da`**.

Os **10 commits pré-jornada** + a **jornada de 2026-07-15** (até `2e1806fb`) já estão descritos em
[`HANDOFF_line_Painter_integracao_2026-07-15.md`](HANDOFF_line_Painter_integracao_2026-07-15.md) — **não
re-listo aqui**. Este handoff cobre só o **DELTA desde então** (W5b + a borda do Inflate + o click-through do
chrome + o diagnóstico da junção).

---

## 2. O que esta integração BANCA (delta desde 2026-07-15)

### 2.1 — W5b: o FILTRO de camada inteira e de traço — **SMOKADO OK**
| commit | |
|---|---|
| `57d9881e` | **feat(sculpt): Filter Layer** — o verbo selecionado aplicado na camada inteira, na Strength do pincel, 1 undo step. Zero kernel novo: o filtro preenche `amount` uniforme e chama o MESMO `render_sculpt`. Recusa os verbos de PLANO (Flatten/Scrape/Fill/Chisel — sem pegada) via `SculptMode::filters_layer()` |
| `493665c2` | test(probe): cenas 7/8 do filtro |
| `f5af6246`,`595d49ce` | docs: W5 fechada |
| `ea0a5c02` | **feat(sculpt): Filter Stroke** — o mesmo verbo escopado ao último traço, mascarado pelo envelope de tinta dele (`relief.live_paint`); 2 escopos = UM fator por texel (`strength × selection × envelope`). 9 gates tool + 4 de seam que CLICAM os botões |
| `2ca44257` | docs: o `Layer` cortado da lista (knob morto: a luz lê `∇h`, uma constante não move pixel) |

**Smoke do Enio:** *Filter Layer no Inflate = "ficou muito bom!"* — **APROVADO**. (A borda do Inflate ele
**reprovou** — ver §2.2 e §6.)

### 2.2 — A borda do Inflate: o taper da MATÉRIA — **landou, mas NÃO fecha o sintoma da junção**
| commit | |
|---|---|
| `f8902dfc` | docs(handoff): diagnóstico da borda (a bola tinha 2 bordas — altura tapera, matéria era binária) |
| `8ea5f91c` | **fix(sculpt): a matéria segue o MESMO taper da altura** — porta única `ball_taper()` em `sculpt_offset.rs`; cobertura `255→62`, mais macia que o próprio depósito. **Byte-idêntico fora do alcance** (gate) |
| `6f4c84ea`,`611f4f22`,`6b7cb183`,`a07b30ca` | sondas + split de LOC (`inflate_edge` 729→gates+sondas) + a sonda do knob Smooth |
| `dcb5ea7a` | **feat(sculpt): DECISÃO — o padrão-ouro é a BOLA VERDADEIRA** (`√(ρ²−d²)`), provada; o knob Smooth **não** é a resposta (medido) e o default fica **0** — agora por medição, não compatibilidade. `sculpt.rs` em 700/700 |

⚠️ **O `8ea5f91c` é um avanço real e byte-identity-gateado, mas a borda da JUNÇÃO continua visivelmente
quebrada** (o rasgo branco nas axilas). A causa é **outra** e só ficou clara nesta sessão — ver §2.4 e §6.
O integrador **banca o `8ea5f91c`** (é estritamente melhor); o sintoma da junção é o item ABERTO.

### 2.3 — O clique num botão é do botão — **FOUNDATIONAL tocado — SMOKADO OK**
| commit | |
|---|---|
| `518c91a5` | **fix(shell): click-through do chrome** — *"os painéis laterais permitem que o clique sobre botões pinte a pintura atrás"* (Enio). A pergunta *"isto é chrome?"* tinha DUAS metades (`store.panel_at` + `hit_index.hit`) e 4 sítios de canvas pediam UMA. Porta única `chrome_hit::chrome_claims`/`pointer_over_chrome` (novo `shells/desktop/src/chrome_hit.rs`, 203 LOC); `forwarding.rs` encolheu 614→422. 4 arch-gates em `tests/the_chrome_swallows_the_click_it_was_given.rs` |
| `10bd2a10` | **fix(shell): REGRESSÃO minha** — o gizmo tem DOIS esquemas de id (canônico + **keyed** `canonical ^ hash(bits)` p/ extras + global) e a porta só conhecia o canônico ⇒ handles keyed viravam "chrome" ⇒ o pincel virava gesto de mover. Fix: a porta pergunta ao `gizmo_hit_map` também |
| `b2ba6c8f` | docs: §9 do handoff da borda corrigido (esta linha TOCOU foundational) |

**Smoke do Enio:** *"smoke da propagação do clique OK"* + *"smoke parece OK como está agora"* (a regressão
do mover-em-vez-de-pintar corrigida). **APROVADO.**

**⚠️ FOUNDATIONAL — o integrador roda `scripts/foundational-integrate.sh`:**
- `crates/ph2d-editor-core/src/gizmo/hit.rs` — **`pub fn is_gizmo_id(id) -> bool` APENDADO** (`is_gizmo_handle_id(id) || id == ids::GIZMO_PIVOT`). Não altera nada existente; é a porta que faltava (o pivô não é um drag-kind, então todo chamador de *"é do gizmo?"* remendava `|| == GIZMO_PIVOT` à mão). Re-exportado em `gizmo/mod.rs` + `lib.rs`.
- `crates/ph2d-editor-core/src/ids/chrome/painter_sculpt.rs` + `painter_deform.rs` — **consts de id APENDADAS** (`PAINTER_SCULPT_RAKE`/`_CONSERVE`/`_FILTER`/`_FILTER_STROKE`/`_SMOOTH_*` + arrays `PAINTER_SCULPT_CLICKS`/`_FIELDS` da jornada do sculpt). São ids de widget, **não** superfície de contrato congelado.
- **Nenhum contrato congelado (§6 do CLAUDE.md) foi tocado** — `Tool=12`/`CanvasPaintTool=1`/`NodeManifest`/`Vector*` intactos. O gate `architecture_tool_contract_surface` + `architecture_contract_surface` + `architecture_vector_contract_surface` passam (editor-core 740 verde).
- Como o main **não divergiu**, mesmo o foundational funde por ff sem resíduo textual — o `foundational-integrate.sh` é a rede de segurança (gate da árvore combinada), não um merge disputado.

### 2.4 — O diagnóstico da JUNÇÃO — **test-only, ZERO mudança de produto**
| commit | |
|---|---|
| `58601135` | diag(sculpt): MEDIDO — o Blob não ENGORDA o flanco, só preenche AXILAS (flanco +0, axila +4) |
| `10288837` | diag(sculpt): **CAUSA RAIZ** — a parábola CAPTURA `√(H/D)×` mais longe do que consegue SERVIR; split do módulo de sondas (845>700 → `inflate_edge_probes` + `inflate_junction_probes`) |
| `bf3e18da` | diag(sculpt): **PROVA em pixels** — a bola LIMITADA enche a axila e engorda a aba na cruz do Enio; e paralela custa **3,0 ms/move** |

Tudo `#[ignore]` (sondas de medição) + novo `sculpt_tests/inflate_ball_candidate.rs` (311 LOC). **Não muda
um byte do produto** — é a prova que decide a próxima linha (§6).

---

## 3. Gates — o que o integrador roda (1× sobre a árvore combinada)

1. **`scripts/foundational-integrate.sh`** — gate da árvore combinada (o toque em `ph2d-editor-core`, §2.3).
2. **`./scripts/ship.sh`** — paridade EXATA com o CI (fmt, clippy `--all-targets`+features, machete, deny,
   audit, nextest `--cargo-profile ci-test`, typos). Corrigir todo `✗` **antes** de qualquer push.
3. Verde local confirmado nesta ponta (fechamento da linha, não substitui o ship):
   - `cargo test -p ph2d-tool-painter` → **711 passed, 35 ignored**
   - `cargo test -p ph2d-editor-core` → **740 passed** (contract surfaces + LOC cap)
   - `cargo test -p ph2d-host-desktop --test the_chrome_swallows_the_click_it_was_given` → **4 passed**
   - `cargo clippy -p ph2d-tool-painter --all-targets` → 0 warnings
- **LOC cap:** `sculpt.rs` está em **700/700 exatos** — qualquer campo novo exige split (o raciocínio da
  decisão do Smooth mora no handoff, não no arquivo, justamente por isso).

---

## 4. Estado de SMOKE (o que está aprovado vs pendente)

**APROVADO pelo Enio:** click-through do chrome (+ a regressão do mover) · Conserve · Push · a âncora do aro ·
Filter Layer / Filter Stroke · a cápsula (dabs = contas) · Anchored/Line-undo. (Detalhe e datas: os handoffs
`integracao_2026-07-15` e `push_rim_anchor_2026-07-15`.)

**Pendente de smoke** (código verde, apenas sem o olho do Enio ainda): W4 (advecção de relevo do Deform) · a
fase D (display pipeline) · o `8ea5f91c` isolado (o taper da matéria — o faceamento da junção domina a vista,
então nunca foi smokado sozinho).

Nada disso bloqueia a integração — o gate é lint+test, não smoke. O smoke gateia a CORRETUDE do trabalho, e o
CLAUDE.md §5 é o tracker vivo desse estado.

---

## 5. `git status` da worktree

Árvore **limpa** (`git status --short` vazio). Sem WIP alheio, sem arquivo não-rastreado a decidir.

---

## 6. ⚠️ FORA DESTA INTEGRAÇÃO — a troca do kernel do Inflate (a próxima linha)

O único item **ABERTO** do sculpt. **Diagnóstico FECHADO e provado nesta sessão; a correção está PROJETADA e
MEDIDA, mas NÃO construída.** Não faz parte deste ff.

- **Por que a junção não infla:** a parábola separável tem **suporte ilimitado** — uma fonte de altura `H`
  vence o envelope até `√(H/a)` mas só entrega até `ρ√2`. Numa junção (o ponto mais alto da tela) ela
  reivindica **2,1×** mais longe do que serve (medido na cruz: 47,5 vs 22,6 texels) e entrega NADA ao vão; a
  fronteira dessa célula de Voronoi é o rasgo branco. A advecção está inocentada (0 texels perdem cobertura).
- **A correção provada:** a **BOLA LIMITADA** (`√(ρ²−d²)`). Em pixels, na cruz do Enio, ela **enche a axila
  inteira e engorda a aba de 15→29 colunas** onde a parábola deixa o rasgo (`diag_does_the_bounded_ball_fix_the_cross`).
- **A perf:** a bola exata é `O(área·ρ²)` = **44 ms/move** (fora do kill), MAS é embaraçosamente paralela
  (linhas disjuntas, sem RNG, byte-idêntica — a MESMA propriedade que o ADR-0109 admitiu p/ o watercolor) e
  **paralelizada custa 3,0 ms/move** em 32 cores, abaixo do alvo de 4 (`diag_exact_ball_per_move_cost`).
- **O bônus:** a bola limitada com raio por-fonte `ρ·amount` **apaga as 4 camadas de contenção** do
  `render_inflate` (sentinela · orçamento por-fonte · taper · piso-próprio) — todas existem só pra conter o
  suporte ilimitado da parábola. É a representação apagando o caso especial.
- **Custo arquitetural a decidir na próxima linha:** estender a exceção do **ADR-0109** (rayon é *"Used ONLY
  there"* hoje) para o `render_inflate` + deletar 4 defesas gateadas (red-first + mutação em cada uma).

As sondas que provam tudo isso estão nesta integração (`sculpt_tests/inflate_edge_probes.rs`,
`inflate_junction_probes.rs`, `inflate_ball_candidate.rs`, todas `#[ignore]`). O
[`HANDOFF_line_Painter_inflate_edges_2026-07-16.md`](HANDOFF_line_Painter_inflate_edges_2026-07-16.md) §8
descreve a etapa anterior do raciocínio (o taper da matéria) — **superado** pelo achado da junção acima
quanto à *causa do sintoma visível*; o taper daquele handoff (`8ea5f91c`) continua correto e integra.

---

## 7. Resumo para o integrador

Bancar 52 commits por **ff limpo** (main não divergiu). **1 toque foundational** (`is_gizmo_id` apendado +
ids de chrome apendados na `ph2d-editor-core`) → roda `foundational-integrate.sh`. **Nenhum contrato
congelado tocado.** Roda `ship.sh` até verde. A **borda da junção do Inflate segue ABERTA** (diagnosticada +
correção provada, kernel novo é a próxima linha, §6) — **não** é regressão desta integração, é o estado
herdado que o próximo passo fecha.

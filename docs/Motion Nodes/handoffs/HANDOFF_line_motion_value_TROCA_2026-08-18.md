# HANDOFF DE TROCA — `line/motion-value` · quem assume a linha lê isto primeiro

**Data:** 2026-08-18 · **Branch:** `line/motion-value` · **Worktree:**
`Worktrees/line-motion-value/` · **Base:** `main` @ `692ee2039` (linha **reaberta**, 0
commits próprios — a jornada dos grupos I..P JÁ INTEGROU)

> Este é o item **5 da FASE 2** do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> *"onde o agente anterior deixou o que já foi decidido, medido e REPROVADO"*. As regras
> permanentes (A–H) NÃO estão copiadas aqui de propósito — elas vivem no
> [`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md), e
> *duas cópias da mesma regra divergem*.

---

## 0. A linha já está PREPARADA — o que foi feito e conferido

| passo | estado (medido em 2026-08-18) |
|---|---|
| `pwd` / branch | `…/Worktrees/line-motion-value` · `line/motion-value` ✅ |
| alinhamento com o `main` | **0 à frente, 0 atrás** (`git merge --ff-only main` já rodou) |
| árvore | **limpa** (`git status --short` vazio) |
| tier | **`workstation`** (Modo L autorizado) |
| `cargo check -p ph2d-node-motion-wave` | ✅ **0,58 s** (o `target/` desta bancada está quente) |
| `target/*/incremental` | já reclamado no fecho anterior |
| `/dev/shm` (o `target/` **É** RAM aqui) | **38 G livres** de 62 |

⇒ **Não repita a FASE 1 de setup.** Faça só o `cd` + `pwd` + `git branch --show-current`
antes de ler qualquer arquivo, que é a defesa que o MODELO existe para instalar.

⚠️ **A integração foi por REBASE.** Os shas que o handoff de integração nomeava
(`c71b880b5`…) **não existem** no `main` — foram reescritos. Se você for procurar o
trabalho da linha por identidade de commit, vai concluir que ele se perdeu; a pergunta
certa é sobre o **CONTEÚDO** (a crate `ph2d-node-motion-proximity` está lá, as cenas
`=49..=56` estão no roteador). É a mesma armadilha que o §6 do TROCA de 2026-08-10 já
registrou.

---

## 1. O que a jornada anterior ENTREGOU (já está no `main` — não reconstrua)

Os **grupos I..P** da segunda volta da [conferência dos nós](../89_plano_conferencia_dos_nos.md),
mais a **auditoria multiagêntica** de fecho. 53 commits, integrados.

| grupo | o que fechou | cena |
|---|---|---|
| **I** | `motion.proximity` — a vizinhança vira NÚMERO (`neighbours` + `overlap`) | `=49` |
| **J** | o PINO alcança `verlet_rope` / `soft_body` / `boids` (`inv_mass` pela cadeia de estado) | `=50` |
| **J′** | a prescrição do corpo mole + o espaço pessoal do bando | `=51` |
| **K** | peso por partícula (`soft_body`) + SUB-PASSOS (`verlet_rope`) | `=52` |
| **L** | o TETO DA TAXA (`motion.delay`: `max_step` + `max_accel`) | `=53` |
| **M** | a contagem da conferência deixa de ser escrita à mão (`placar_conferencia.py`) | — |
| **N** | `motion.wiggle` ganha oitavas, multiplicador e LAÇO | `=54` |
| **O** | `motion.oscillator` o PULSE WIDTH · `motion.stagger` o OFFSET | `=55` |
| **P** | `motion.drive` escreve COLUNA NOMEADA — **a §10.0 do plano FECHOU** | `=56` |

**As oito cenas foram smokadas e aprovadas pelo Enio**, cada uma à medida que o grupo
dela fechou. O mecanismo de cada uma está na **§5 do `CLAUDE.md`**, escrito no commit
de cada grupo — não foi copiado para cá.

⚠️ **A auditoria correu DEPOIS dos smokes**, e as quatro correções que ela produziu
(o WGSL do boids elevando ao quadrado um param com sinal · o teto de taxa INERTE numa
posição alcançável do slider · o `motion.drive(Custom…)` podendo sobrescrever coluna de
escrituração · o gate de sub-passos TAUTOLÓGICO) **não passaram por smoke**. Re-smoke
das cenas `=50..=53` é barato e é decisão do Enio.

---

## 2. ⛔ MEDIDO E REJEITADO — não reconstrua, não re-litigue

| item | por quê |
|---|---|
| **A tally da média PONDERADA** no `motion.collide` (grupo H) | Pôr o peso do par no numerador **e** no divisor o faz CANCELAR num par isolado: `falloff = 0,5` desenhava exatamente o mesmo que `1,0` (0,6 e 0,6) — o knob seria decorativo. A tally é uma CONTAGEM. |
| **A MOLA DE COMPRESSÃO** no `motion.boids` (`~(r − d)` em vez de `1/d²`) | Construída e medida **PIOR**: mediana 0,387 contra 0,748. *Um equilíbrio entre forças FINITAS cai sempre ABAIXO do raio onde a repulsão se anula*; para o vão pousar num número é precisa uma RESTRIÇÃO — e ela já existe, é o `motion.collide` (mediana 0,9878 de um diâmetro 1,0). |
| **`motion.lag` como NÓ NOVO** (doc 63 §2.2, marcado P0) | Metade dele já obsoleta quando a célula foi escrita: `Average` e `Blend` **existem no `motion.delay`** desde que ele nasceu, e o `ticks_down` do grupo F já era o *Lag up/down* do MESMO CHOP. |
| **Massa por elemento no `motion.spring`** (folha 03 linha 65) | A composição já a dá: cada `motion.spring` tem os SEUS `tension`/`friction` e o `falloff` o torna transparente onde vale zero. Medido: o elemento da mola rápida sai em 7,0213 — **idêntico ao controle**, ao dígito. `P1 → P2`. |
| **A faixa de barras do `value.pattern`** (`ParamWidget::Steps`) | Construída, smokada e **REVERTIDA inteira** por veredito do Enio (*"ficou pior. Volte como estava antes"*). ⚠️ **Nenhum mecanismo foi nomeado** ⇒ uma segunda tentativa **começa perguntando o que ficou pior**, não reconstruindo. A árvore sobrevive em `ae35416bd`, com 13 gates e 8 mutações já escritos. |
| **O Simplex** no `value.noise` | A anisotropia que ele cura (1,83% no Perlin-2002) é **menor que a diferença entre dois vizinhos do próprio menu** (0,78% do Value, 1,95% do Cellular). |
| **A variância de UM passo** no `value.reduce` | Num campo CONSTANTE de magnitude `1e5` ela reporta desvio **71,6**. Correção acima de velocidade. |
| **`iterations` no `value.smooth`** | A relação estava **INVERTIDA**: `N` box são uma B-spline de grau `N−1`, logo **o peso é o parâmetro geral**. |

**E DUAS coisas foram medidas e NÃO curadas de propósito** — a cura mudaria um smoke já
aprovado, então elas são decisão do Enio, não trabalho pendente:

- ⛔ **A composição sub-passos × `damping`** na `motion.verlet_rope`. A cena `=52` não
  escreve `damping`, então o smoke aprovado **já continha a composição**; as duas curas
  candidatas moram no doc-comment do param, com um ⛔ explícito.
- ⚠️ **O gate `#[ignore]` da cena `=53`** (`the_ceiling_is_honoured_on_every_tick_including_the_turn`):
  o teto é honrado **ao dígito na rampa** (`0,0800`) e sobe a **`0,1678` no tique da
  inversão**. A lei do kernel **não pode** produzir isso (ela clampa `|out − prev|` por
  construção, e cinco gates de unidade sangram sob mutação) ⇒ a diferença mora **entre o
  kernel e o que a CENA monta**, com o `prev_out` do gather como candidato nomeado.
  ⛔ **NÃO afrouxe a barra** — o precedente é o par `watercolor_app_params_incremental`
  do Painter: *uma barra afrouxada sobre um mecanismo que ninguém entendeu é um gate que
  deixou de perguntar*.

---

## 3. O que está ABERTO — e o placar é DERIVADO, não escrito

⚠️ **Rode você mesmo antes de escolher a wave** (o grupo M existe exatamente para a
contagem deixar de ser uma frase que envelhece):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"
```

**Medido em 2026-08-18:** `P0 = 0 · P0/P1 = 1 · P1 = 68 · P2 = 119 · ⏸ RIG 18 · ✅ 121 ·
⛔ 119`, em 450 linhas de conferência.

### 3.1 O item mais alto — o ÚNICO `P0/P1` vivo

**`motion.wave` com N PRODUTORES** ([folha 06](../89_conferencia/06_animadores.md), linha 35).
Hoje dois `motion.wave` fazem **duas grades** (`rows×cols` próprios) e **não somam no
mesmo campo**; a fonte é a célula do centro, e só ela. A referência é o *Wave World* do
AE (Effect ▸ Simulation ▸ Wave World: **Producers**, Type Ring/Line, Position,
Height/Width, Angle, Amplitude, Frequency, Phase). A folha classifica como **omissão**,
com o motivo escrito: *é a diferença entre "uma onda" e "ondas"*.

⚠️ **As outras duas células `P0/P1` da folha 06 já estão riscadas** (`motion.drive` fechou
no grupo P, `motion.expression` fechou pelas lanes `x`/`y`) — o placar concorda com a
linha de `**Contagem:**` escrita na folha, então **nenhuma envelheceu desta vez**.

### 3.2 Os 68 P1, por família

`06_animadores` 8 · `08_stream_utilidade` 8 · `14_source` 7 · `01_distribuicao_emissao` 6 ·
`04_deformers` 6 · `10_field` 6 · `11_fx_raster` 6 · `02_force` 5 · `05_transform` 4 ·
`03_simulacao` 3 · `07_tempo_estilisticos` 3 · `09_cor` 3 · `15_value` 2 ·
`17_zero_param_debug` 1 · `12_pulse` 0 · `13_sim_stack` 0 · `16_rig` **⏸ 18, família DEFERIDA**.

### 3.3 Nomeado com preço, fora de folha

- A coluna **`neighbours`** do `motion.boids` — o número que o bando computa e joga fora
  é a vizinhança **DELE** (raio próprio, cone, pesos), e emiti-la daria a um consumidor
  genérico uma resposta que só o boids sabe interpretar. Foi por isso que o grupo I a
  fechou com **outro nó**.
- O **`lookahead`/avoid obstacle** do boids pede geometria de obstáculo alcançável de
  dentro do nó ⇒ wave própria.

---

## 4. Armadilhas que custaram tempo — leia antes de mexer

1. ⚠️ **A PRIMEIRA coisa de toda wave é MEDIR se a composição já exprime o item.**
   **SEIS** células envelheceram antes de alguém voltar a elas só na jornada I..P (o
   `max_force`, o *wander*, a idade normalizada, o `motion.lag` inteiro, a posição como
   variável do `motion.expression`, a colisão da corda). *O que se perde ao não reconferir
   não é tempo, é construir o que já existe.*
2. ⚠️ **Os `pre` self-loops de um documento são escritos à MÃO.** O editor os plumba ao
   SOLTAR um nó; `Graph::add_node` não. Uma fixture de simulação **sem `advance_tick`**
   nunca carrega estado pela aresta `pre`, a cena fica morta, e **todo gate de feedback
   fica verde por vácuo** — foi assim que duas mutações passaram no grupo J e o gate da
   corda nasceu medindo a pose RETA.
3. ⚠️ **O `--ignored` do nextest CANCELA na primeira falha.** Duas suítes de GPU NOVAS
   (`gpu_proximity`/`gpu_boids`) **nunca tinham corrido** por causa disso; use
   `--no-fail-fast` + `--test <nome>`.
4. ⚠️ **Gates de GPU só significam alguma coisa em SÉRIE** (`--test-threads=1`): em
   paralelo cada binário abre o próprio device e o wgpu morre com `Out of Memory` — a
   linha mediu **três** assim. E *skip gracioso não é verde*: confira que o adapter foi
   ENCONTRADO.
5. ⚠️ **Os gates de `shells/desktop/tests/` e `ph2d-editor-core/tests/` só correm na
   varredura impactada.** Um fechamento por `cargo test -p` por crate **não os alcança** —
   é a família do vermelho-latente que esta linha já pagou quatro vezes.
6. ⚠️ **Nenhuma leitura de relógio desta workstation significa coisa nenhuma acima de
   `load ~5`.** O mesmo binário mede 11,36 ms e 5,50 ms para o mesmo passe sob `load 41`
   contra `load 0,6`. Confira o load ANTES de acreditar num gate de razão.
7. ⚠️ **Uma busca negativa precisa de CONTROLE POSITIVO.** Duas mutações desta linha
   "passaram" sem nunca terem sido aplicadas — uma por indentação errada na âncora do
   `python`, outra por um `echo` com crases que matou o comando —, e o `cargo test`
   seguinte imprimiu VERDE sobre a árvore **não-mutada**.
8. ⚠️ **A cwd do Bash volta para a árvore PRIMÁRIA entre turnos.** Todo comando começa
   com o `cd` da worktree — e um `grep` read-only responde da árvore errada **sem erro
   nenhum**.
9. ⚠️ **O número da próxima cena se CONTA lendo o `match`**, nunca uma nota:
   `shells/desktop/src/motion_state_demo_router.rs`. Medido hoje: cenas **1..56
   contínuas** ⇒ **próxima livre 57**. O roteador é a **ÚNICA** lista de níveis (o irmão
   `motion_state_demo_conferencia.rs` **não tem `match` nenhum, de propósito** — dois
   `match` em dois arquivos deixariam um nível reivindicado duas vezes passar em
   silêncio).
10. ⚠️ **`value_slope_kernel_matches_the_cpu_on_the_device` é RED PRÉ-EXISTENTE.** Ele
    mede `1,05023384e-4` contra uma barra de `1e-4` e **reprova no `main` com o número
    idêntico a todos os dígitos**; é `#[ignore]`, logo fora do ship. Não o persiga.
11. ⚠️ **Todo canal novo é side-metadata no REGISTRY, nunca contrato.** `NodeOp=2` /
    `OpResolver=1` / `NodeManifest=8` seguem intactos, e o padrão é o `register_ui`:
    default neutro, zero churn nos kernels existentes. Param não-`f32` viaja como **text
    param** (o `Graph` o guarda), que é o padrão canônico desde a `motion.expression`.

---

## 5. Como provar que a base está sã antes de começar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
pwd && git branch --show-current          # a defesa do MODELO, sempre

ph2d-run cargo check -p ph2d-nodegraph
ph2d-run cargo test -p ph2d-nodegraph  --test architecture_contract_surface        # 3/3
ph2d-run cargo test -p ph2d-editor-core --test architecture_tool_contract_surface  # 4/4
ph2d-run cargo test -p ph2d-node-registry-init                                     # inclui staleness

# o censo — rode você mesmo, o número envelhece
ph2d-run cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture
#   esperado na integração de 2026-08-16: 125 nos - 545 params, 526 com hint, 158 com unidade
```

**As oito cenas da jornada anterior, para confirmar que a base não regrediu:**

```bash
env PH2D_GPU_COOK_DEMO=49 ph2d-run cargo run -p ph2d-host-desktop --release   # ... ate =56
```

⚠️ **Toda cena de grupo imprime a lista de bandas nomeadas. Se a lista não aparecer,
PARE** — o resto do smoke não diz nada.

---

## 6. Nota de processo — o que esta reabertura mediu

⚠️ **A entrada de INTEGRAÇÃO desta jornada NÃO está na §5 do `CLAUDE.md`.** As quinze
entradas por-wave (grupos A..P) e a auditoria estão — foram escritas durante a jornada —,
mas a última linha `line/motion-value INTEGROU (…)` é de **2026-08-16**, a jornada
anterior. O integrador escreveu a da `line/sculpt3d` (`460c1d630`) e não a desta.

**Não a escreva por conta própria:** o valor dela é a superfície de colisão medida na
**árvore COMBINADA**, e quem a mediu foi o integrador. Se o Enio quiser, ela se escreve
com o que é verificável hoje — e aí é uma ordem, não uma iniciativa.

⚠️ **E o `main` mexeu-se DENTRO da sessão de fechamento** (`26b2d81ab` → `692ee2039`),
entre a tabela de rebase do handoff de integração ser medida e a ordem chegar. É a
terceira vez que este repo paga isso, e a lição é a mesma: *a caixa de rebase de um
handoff envelhece entre o fechamento e a ordem* — o integrador re-mede, nunca herda.

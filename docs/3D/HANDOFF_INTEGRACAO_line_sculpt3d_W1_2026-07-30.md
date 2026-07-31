---
titulo: "HANDOFF DE INTEGRAÇÃO — line/sculpt3d, W1 (a malha)"
tags: [modulo/3d, tipo/handoff, assunto/integracao, status/ativo]
status: ativo
modulo: 3D
atualizado: 2026-07-30
resumo: "O que o agente integrador precisa saber para fundir a W1 do módulo 3D: identidade, o que foi tocado fora do módulo, símbolos que podem colidir, e o que só o ship.sh pega."
---

# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, W1

> **A linha NÃO integra e NÃO faz ship.** Este documento existe para o agente
> integrador que o Enio abrir (DIRETRIZ §1.5.9). A W1 está fechada: M1 (a malha),
> M2 (a malha na tela) e M3 (as medições) entregues.
>
> ✅ **SMOKE APROVADO pelo Enio (2026-07-30).** A esfera aparece, gira, desloca e
> aproxima; o app 2D fica intocado sem a env var.

## 0. O bloco a colar no agente integrador

```
═══════════════════════════════════════════════════════════════════
TAREFA — integrar `line/sculpt3d` (W1 do modulo 3D) na main
═══════════════════════════════════════════════════════════════════

A linha JA ESTA REBASEADA sobre a main de hoje (cd8513b76) e o gate
da arvore combinada JA RODOU VERDE nela. Confira e integre:

    cd /home/enio/Documentos/Projetos/PH2D
    git merge --ff-only line/sculpt3d

O `--ff-only` tem de funcionar. Se ele recusar, a main andou de novo
desde o rebase: peca a rebase E o re-gate, nao force nada.

DEPOIS DO MERGE, rode o gate de fechamento:
    cargo check --workspace
    cargo test -p ph2d-mesh -p ph2d-sculpt3d -p ph2d-mesh-render --release
    cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored
    cargo test -p ph2d-host-desktop --tests
    cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
    ./scripts/ship.sh        # so por ordem EXPLICITA do Enio

LEIA ANTES: as secoes 2, 3 e 5 deste documento. Em uma frase cada:
  * ZERO crate foundational tocada; 3 crates NOVAS, drop-crate.
  * ZERO contrato congelado, ZERO schema, ZERO id/token/variant.
  * O unico simbolo disputavel e o NUMERO DO ADR (0145) — §3.
  * A deriva de `typos` que o ship.sh vai acusar e PRE-FORK (§5).
```

## 1. Identidade

| | |
|---|---|
| Branch | `line/sculpt3d` |
| HEAD | **`a708cd11e`** |
| Base (já rebaseada sobre a `main` de hoje) | **`cd8513b76`** |
| Commits | **6** |

```
f3df7426c  docs(3d): o cofre docs/3D verbatim (pré-guinada)
f3b3a1b4e  docs(3d): a guinada — sem MVP, direto em Rust, SculptGL de referência
da4c265fb  feat(3d): W1/M1 — a malha residente (ph2d-mesh)
6fe178769  feat(3d): W1/M2 — a malha na tela (ph2d-mesh-render + órbita no shell)
9fe6bc005  feat(3d): W1/M3 — as sondas que decidem K1/K2, e o que elas acharam
a708cd11e  docs(3d): handoff de integração da W1
```

### ✅ O rebase já foi feito, e o gate já rodou sobre ele

O fork era `7ec917506`; a `main` andou **449 arquivos** desde então. A linha foi
rebaseada sobre `cd8513b76` e:

| | |
|---|---|
| Conflitos no rebase | **ZERO** — inclusive nos 6 arquivos sobrepostos (§3) |
| `cargo check --workspace` | ✅ |
| Suítes das 3 crates (release) | ✅ 10 binários, 0 falhas |
| Gates de GPU (`-- --ignored`, na RTX) | ✅ **4/4** |
| Gates de shell | ✅ **59 binários, 0 falhas** |
| LOC caps (workspace + shell) | ✅ |

⚠️ **Isto não dispensa o integrador de re-rodar** — *merge textual limpo pode
estar semanticamente quebrado*, e se outra linha entrar na `main` antes desta, o
estado que eu gateei deixa de ser o estado que vai ser fundido.

⚠️ **Os dois primeiros commits movem o cofre `docs/3D/` para dentro da linha.**
Ele estava **não-rastreado na árvore primária** (invisível a toda outra linha), e
por isso foi commitado *verbatim* antes da guinada — para o redesenho ler como um
diff. **As cópias antigas já foram removidas da primária** (byte-idênticas ao
`457e24cb0`, conferido por `git archive` + `diff -rq`), então não há arquivo
não-rastreado nos caminhos que esta linha adiciona e o `--ff-only` não recusa.

## 2. Foundational / compartilhado tocado, e por quê

**Nenhuma crate foundational foi tocada.** As três crates do módulo são novas e
drop-crate. O que sai da pasta do módulo:

| Arquivo | O que mudou | Aditivo? |
|---|---|---|
| `shells/desktop/Cargo.toml` | 2 deps `optional` + a feature `sculpt3d` (na lista `default`) | ✅ só adiciona |
| `shells/desktop/src/main.rs` | `#[cfg(feature)] mod sculpt3d;` | ✅ |
| `shells/desktop/src/sculpt3d.rs` | **arquivo novo** — cena, gesto e passe | ✅ |
| `shells/desktop/src/app_state.rs` | 1 campo em `AppGfx` (`sculpt3d`), sob `cfg` | ✅ |
| `shells/desktop/src/init.rs` | `sculpt3d: None` no literal de `AppGfx` | ✅ |
| `shells/desktop/src/render_loop/mod.rs` | `sculpt3d: _` no destructuring + a chamada do smoke | ⚠️ **o destructuring é ponto de colisão — ver §3** |
| `shells/desktop/src/render_loop/present.rs` | `sculpt3d` no destructuring + o "Pass 1d" | ⚠️ **idem** |
| `shells/desktop/src/input_dispatch.rs` | 3 interceptações (move / botão / roda), todas sob `cfg` | ✅ early-return |
| `.typos.toml` | 5 palavras pt-BR, **apendadas ao FIM** da seção | ✅ só adiciona |
| `.gitignore` | `docs/3D/.obsidian/` | ✅ |
| `docs/architecture/decisions/0145-*.md` | ADR novo | ⚠️ **número provisório — ver §3** |

## 3. Símbolos que podem COLIDIR com outra linha

| Símbolo | Valor | Risco |
|---|---|---|
| **`ADR-0145`** | o arquivo `0145-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md` | ⚠️ **PROVISÓRIO, e é o ÚNICO símbolo de fato disputável desta linha.** O 0145 estava livre no `main` de hoje, mas **um número de ADR escolhido numa linha paralela é provisório** — já aconteceu duas vezes (0130→0131 na física, 0134→0140 no gpu-nodes). Como os nomes de arquivo diferem, **o git nunca conflita**: quem chega ao `main` primeiro fica com o número e o gate `architecture_adr_numbers_are_unique` é quem acusa. **Se houver colisão, renumere ESTE** e conserte os 9 ponteiros (`git grep -l "ADR-0145\|0145-3d-sculpt"`). |
| feature `sculpt3d` | `shells/desktop/Cargo.toml` | Nome único; a lista `default` é append-only. |
| Os dois `AppGfx { … }` destructurings | `render_loop/mod.rs` e `render_loop/present.rs` | ⚠️ Toda linha que acrescenta campo em `AppGfx` toca as MESMAS duas listas. **No rebase de hoje não conflitou**; se conflitar com uma linha futura, a resolução é trivial (as duas metades entram). |
| `PH2D_SCULPT3D_SMOKE` | env var | Nome único. |

**Zero id de widget, zero token de tema, zero chave i18n, zero variant de enum
compartilhado, zero entrada em lista ordenada** — a W1 não tem UI de painel.

**`PROJECT_SCHEMA` e todo schema: INTOCADOS.** Nada desta linha é serializado.

## 4. Contratos congelados encostados

**NENHUM** — conferido por grep, não por auto-relato:

- `Tool = 12` / `RasterEditTool = 5` / `CanvasPaintTool = 1` / `PanelEvent = 4` — intactos.
  A navegação orbital mora **no shell**, nunca numa `Tool`, e é essa decisão que
  mantém a superfície congelada fora do caminho (ADR-0145).
- `NodeOp = 2` / `OpResolver = 1` / `NodeManifest = 8` — nem tocados.
- `ComponentRegistry` do `ph2d-ecs` — **não** mudou de contagem (a W1 não registra
  componente nenhum; a malha ainda não é uma entidade).

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

| Item | Estado |
|---|---|
| **Deps novas** (machete) | `ph2d-mesh` ganhou **`rayon = "1"`**; `ph2d-mesh-render` ganhou **`glam = "0.30"`**, `wgpu = "28"`, `bytemuck`, `pollster` (dev). Todas já no `Cargo.lock` do workspace — **nenhuma aresta nova de crates.io**. `cargo machete` limpo nas três crates. |
| `Cargo.lock` | Mudou (as 3 crates novas + as arestas). Conflito provável; **regenerar em vez de resolver à mão** (DIRETRIZ §1.5.6). |
| **typos** | ⚠️ **O `ship.sh` vai acusar 19 erros, e NENHUM é desta linha.** Medido: `typos` dá **19 erros no `main` (`cd8513b76`) e 19 na linha** — a deriva é 100% pré-fork ([[project_integration_prefork_lines_ship_drift]]). Ela mora em `ph2d-wet-paint` (4 arquivos), `ph2d-vec-edit/src/hobby.rs`, `ph2d-render/tests/fx_stack_blend_gpu.rs`, `ph2d-panel-painter-layers/tests/seam_curve_drag_ownership.rs` e dois `fx_*_smoke.rs` do shell — palavras `conservativo · editores · fing · inh · mediam · tese`. **É dívida dos donos originais**; o integrador só a *encontra*. Desta linha, o `.typos.toml` ganhou 5 palavras pt-BR (`termine`, `duplicatas`, `multiplicativo` ×3), apendadas ao fim da seção ⇒ fundem limpo. |
| **clippy** | Limpo com `-D warnings` nas 3 crates + shell. ⚠️ O `ship.sh` roda clippy **sem** `--all-features`, e é exatamente por isso que a feature `sculpt3d` está na lista `default` — atrás de uma feature desligada este código não seria lintado. |
| **fmt** | `rustfmt --edition 2024` passado em tudo que a linha criou. |
| **RUSTSEC / deny** | Sem dep externa nova ⇒ sem superfície nova. |
| **LOC caps** | `architecture_workspace_file_loc_cap` e `shells/desktop/tests/file_loc_caps` verdes. |

## 6. Ordem, dependências e o que smoke-testar

**Os 5 commits são sequenciais e não têm dependência externa.** Nada nesta linha
depende de outra linha da jornada.

### ✅ O smoke — APROVADO em 2026-07-30, e o comando fica para o re-smoke

```bash
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_SCULPT3D_SMOKE=1 \
  cargo run -p ph2d-host-desktop --release
```

A cena imprime o que montou (`esfera com 6050 vértices / 6144 faces / 12096
triângulos`) e as instruções. ⚠️ **Se essa linha não aparecer, pare** — o resto
não significa nada. **O que julgar:**

1. **Aparece uma esfera de barro** sombreada, ocupando boa parte da tela.
2. **Arrastar com o ESQUERDO gira** — e gira na direção que a mão espera
   (arrastar para baixo olha o modelo de cima).
3. **Arrastar com o MEIO desloca**; a **RODA aproxima** e o gesto tem o mesmo
   efeito aparente perto e longe.
4. Nada do app 2D mudou. ⚠️ **Sem a env var, `AppGfx.sculpt3d` é `None` e cada
   porta devolve `false` no primeiro `if`** — o frame 2D é byte-idêntico. Vale
   rodar uma vez SEM ela: é a metade do smoke que prova a inércia.

### Gates que precisam de adapter (`#[ignore]`, o integrador roda na RTX)

```bash
cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored
```
4 gates; sem adapter fazem *skip gracioso*, **que não é verde**.

### As sondas (não são gates de regressão — são medições a LER)

```bash
cargo test -p ph2d-mesh      --release --test measure_memory  -- --nocapture
cargo test -p ph2d-mesh      --release --test measure_normals -- --nocapture --test-threads=1
cargo test -p ph2d-sculpt3d  --release --test measure_brush_kernel -- --nocapture
```

## 7. ⚠️ O que a W1 mediu, e a decisão que ficou para o Enio

**A aposta central do ADR-0145 está MEDIDA e vale:** com a pegada FIXA, 10× a
malha custa **0,79×** (dab) e **1,04×** (refresh de normais). O custo é da
**pegada**, não da malha.

**Mas os dois kill-criteria do `docs/3D/03.5` disparam — e o K2 aponta para o
lugar errado:**

| | 5 M triângulos, pincel de 30% (484k vértices) |
|---|---|
| dab completo | **20,2 ms** contra o K1 de 8 ms |
| └ normais de vértice | 0,76 |
| └ normais de face | 0,90 |
| └ **descobrir a vizinhança** | **11,5 (88% do refresh)** |
| consulta no octree | 3,9 |

No pincel de **detalhe** (2% do modelo) o mesmo dab custa **0,566 ms** — 14× sob
o teto. O K1 só dispara com um pincel que cobre **19% da malha inteira**.

⚠️ **O K2 diz *"recomputar as normais passa de 2 ms → migre para a GPU"*, e as
normais custam 1,66 ms dos 13,1.** Seguir o critério ao pé da letra consertaria
13% do problema. O custo real é **descobrir a vizinhança** — dois passes seriais
de dedup, que ainda não foram paralelizados (dedup concorrente pede `AtomicBool`
ou sort+dedup, e é uma wave própria).

**Três caminhos, e a escolha é de produto:**

1. **Paralelizar a descoberta** (a alavanca de CPU que sobrou; os 32 threads já
   renderam 6,4× nas normais).
2. **Migrar o kernel para a GPU** atrás da porta única — o que o critério manda,
   mas mirando o alvo certo desta vez.
3. **Aceitar**: o regime que dispara é o pincel gigante numa malha de 5 M, e é
   exatamente onde a **multiresolução** (`docs/3D/04.3`) existe para pôr o
   artista num nível mais baixo.

⚠️ **A porta `sculpt_kernel_device` do `03.5` NÃO foi construída, de propósito:**
uma porta com uma resposta só e um variant inalcançável é um controle que não faz
nada. Ela nasce junto com o caminho de GPU, se ele nascer.

## 8. Aberto e nomeado (não é dívida escondida)

- **O octree não é atualizado por um dab** — depois de deslocar vértices ele
  descreve as posições anteriores. Enquanto o dab move menos que a folga das
  caixas frouxas isso é invisível; a atualização incremental por região é da W2.
- **A W2 muda a lei do traço**, não o custo: `apply_dab` lê a normal **viva**, o
  que é correto para um toque isolado e errado para uma sequência (a doença do
  produto-por-dab que a `line/Painter` curou quatro vezes). O `pre` congelado no
  pen-down é da W2.
- **`ph2d-sdf` e `ph2d-light` seguem vazias** — nenhuma wave as tocou ainda.
- O **matcap é procedural**; a textura entra com o painel que a escolhe.
- **Sem culling**, de propósito (casca aberta / winding misto de OBJ de terceiro).

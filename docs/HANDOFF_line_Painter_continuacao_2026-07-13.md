# HANDOFF — `line/Painter`: continuação do Impasto (2026-07-13)

> **Para o agente NOVO que vai tocar esta linha.** A jornada anterior fechou e **a integração já foi
> feita** (`main` contém tudo). Este documento te dá: **como se trabalha aqui** (Modo L), **o estado
> real**, e **a fila de implementação, em ordem, com detalhe**.
>
> Leia inteiro antes da primeira linha de código. É longo de propósito — a alternativa é você redescobrir
> na marra o que já custou caro.

---

## PARTE I — O MODO DE TRABALHAR (Modo L)

### 1. O que é, em uma frase

Esta máquina é `workstation` (desktop 128 GB) ⇒ **Modo L** ([ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)):
**N linhas paralelas, cada uma numa `git worktree` própria, SEM coordenador**. Você é uma linha.

**Docs que mandam** (não duplique, consulte):
- **O seu protocolo:** [`DIRETRIZ.md §1.5`](IntegracaoMultiAgente/DIRETRIZ.md)
- **O guia do operador (o Enio):** [`GUIA_JORNADA_MODO_L.md`](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) — leia-o
  também: entender o que o Enio faz do outro lado é o que te impede de fazer o trabalho dele.
- **A cada passo de qualquer implementação:** [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
- **O roteador:** `CLAUDE.md` (raiz) — os 7 inegociáveis.

### 2. As 5 regras que, se você quebrar, quebrou o protocolo

1. **🔴 VOCÊ NÃO INTEGRA. VOCÊ NÃO FAZ `git push`. VOCÊ NÃO RODA `./scripts/ship.sh`.**
   Integração e ship são **ordem EXPLÍCITA do Enio**, executadas por um **agente integrador dedicado**.
   Você **fecha a linha, escreve o handoff de integração (DIRETRIZ §1.5.9) e PARA.** Fazer qualquer um
   dos três por conta própria é **violação de protocolo** (CLAUDE.md §0.7).

2. **Você PODE tocar foundational** (ADR-0107) — é o que distingue o Modo L do Modo C. Mas:
   **ao CRIAR foundational novo, projete-o para ISOLAMENTO** (módulo irmão, ponto de extensão
   append-only), porque várias linhas vão estendê-lo em paralelo. **PARE e reporte ao Enio** em só
   **2 casos**: (a) **contrato congelado** (CLAUDE.md §6 — exige ADR); (b) **rebase conflitando fora dos
   seus arquivos** (colisão de mesmo-símbolo).

3. **Fast mode, o dia inteiro:** `git commit --no-verify` (instantâneo), **zero push, zero CI**. O gate
   pesado roda **1× no fechamento do módulo**, nunca por task.

4. **Inner loop = `cargo check -p <crate>`.** Nada de `--workspace`, nada de clippy, nada de teste por
   task. Isso é velocidade, e velocidade aqui é uma regra, não um gosto.

5. **`cd` em TODO comando.** O `cwd` volta pro repo primário a cada turno. Um `sed -i` relativo **escreve
   no repo errado** (já aconteceu). Mutação **sempre por caminho absoluto**.

### 3. Como você abre a sua linha (o repo primário está em `main`, limpo)

O `main` já tem tudo integrado. Comece uma worktree nova a partir dele:

```bash
cd /home/enio/Documentos/Projetos/PH2D
git worktree list                      # confira que ninguém já está no Painter
# A worktree Worktrees/line-Painter AINDA EXISTE, mas está na base ANTIGA (0 à frente, 167 atrás).
# Duas saídas, escolha uma:

# (a) REAPROVEITAR (recomendado — o target/ está quente, o build é incremental):
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
git fetch --all && git rebase origin/main      # deve ser um fast-forward limpo (0 commits seus pendentes)
git log --oneline -1                            # tem de bater com o main

# (b) DO ZERO: git worktree remove Worktrees/line-Painter && git worktree add ... (build frio: caro)
```

**Regra de higiene:** a janela do repo **primário** (`main`) é só pra setup/integração/ship. **Todo o seu
trabalho acontece dentro de `Worktrees/line-Painter/`.**

### 4. O ciclo de trabalho

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
cargo check -p ph2d-tool-painter          # ← o loop. Só isto.
# … implementa …
git add -- <seus paths> && git commit --no-verify -m "msg"

# 1× no FECHAMENTO do módulo (não por task):
cargo test --workspace                    # ⚠️ NÃO use nextest-impacted — ver §6
cargo clippy -p <crates> --all-targets
```

### 5. Como se prova uma coisa aqui (isto NÃO é negociável)

**Um gate verde que você não sabe derrubar não é um gate.** Toda afirmação vira um teste, e todo teste
tem um **VERMELHO provado por MUTAÇÃO**: você quebra o código de propósito, roda, e o gate cai. Se não
cair, o gate é decorativo — e você o reescreve.

Nesta linha eu escrevi 3 gates para o rig de luzes, rodei as mutações, e **os 3 passaram**. Tive que
reescrever os 3. Sem esse passo, teriam ido pro `main` como enfeite.

**Desfaça a mutação com `cp` de um backup, NUNCA com `git checkout -- <arquivo>`** — o checkout apaga a
sua feature não-commitada junto, o gate "passa" (porque a feature sumiu) e você lê isso como sucesso.
**Aconteceu 3× nesta linha.** Memória: [`feedback_mutation_undo_with_cp_never_git_checkout`](../project-memory/feedback_mutation_undo_with_cp_never_git_checkout.md).

### 6. ⚠️ As 3 armadilhas que já custaram caro NESTA linha

| Armadilha | O que acontece |
|---|---|
| **`nextest-impacted` não vê os gates de contagem de registry** | Registrar um componente no ECS muda a contagem afirmada em `ph2d-render/src/registry.rs` e `ph2d-script/src/registry.rs`. O impacted **não os toca**. Eu reportei "gate verde" com os dois **VERMELHOS**. **Use `cargo test --workspace` no fechamento.** |
| **Pipe mascara o exit code** | `./scripts/x.sh \| grep foo` faz `$?` virar o do `grep`. O script falha e você lê 0. **Verifique o ESTADO, não o código de saída.** |
| **Crase na mensagem de commit** | `fish`/`zsh` **executa** o conteúdo e a palavra some em silêncio. Use `git commit -F <arquivo>` e **releia o log**. |

### 7. Comunicação com o Enio

- **pt-BR, direto.** Recomendação primeiro, opções concretas depois.
- **Decida, não pergunte.** Escolha o padrão-ouro e execute; reporte a decisão. Nada de
  `AskUserQuestion`-spam.
- **Padrão-ouro sem adiamento:** gaps no escopo fecham **na sessão atual**. A melhor opção técnica vence
  custo de build e cronograma.
- **A UI do app é em INGLÊS.** Labels/toasts sempre. (O código e os docs, não.)
- **Quando o Enio contradiz um gate verde: RENDERIZE E OLHE.** Um teste `#[ignore]` que despeja um PNG e
  um `Read` da imagem mataram em 1 minuto o que 2 horas de teoria não mataram. **O pixel é o oráculo.**

---

## PARTE II — O ESTADO REAL (verificado agora, não de memória)

### 8. Onde está o quê

- **`main` = `6c623b67`**, e **contém tudo** desta linha (conferido: `impasto_rig.rs` e o Bug #15 estão
  lá). O integrador rebaseou e fundiu 6 linhas; o `ship.sh` rodou (`9385a85e` drenou 3 latentes).
- A worktree `Worktrees/line-Painter` está **0 à frente / 167 atrás** — rebase antes de qualquer coisa.
- **Gates no fechamento (na worktree antiga):** `cargo test --workspace` → **5684 passed, 0 failed**;
  clippy `--all-targets` → **0**. Depois da integração o número muda (as outras 5 linhas somam testes).

### 9. O que o Impasto JÁ é (para você não reconstruir nada)

O relevo é o **segundo output da MESMA lista de dabs** — é isso que faz Symmetry / Tiling / Shape /
Grain / Jitter / shape-editors esculpirem o relevo **de graça**, e é a regra que você **não pode
quebrar** (um passe de altura pendurado numa rota, ou com geometria própria, é como
*"Tiling não funciona no Impasto"* nasce daqui a seis meses).

Peças, em ordem de leitura:

| Arquivo | O que é |
|---|---|
| `ph2d-painter-brush/src/height.rs` | o **material**: `derive_height(spec, paint, grain)`, `accumulate_dab_height` (a varredura por cápsula), `plow_dab_height`, `erase_dab_height` |
| `ph2d-painter-brush/src/height_film.rs` | **o FILME**: `body_profile` (platô+parede), `film_opacity` (Beer-Lambert), `film_coverage`, `solid_paint`. **Onde a tinta ACABA.** |
| `ph2d-painter-brush/src/height_push.rs` | a **conservação de volume** (o Push) |
| `ph2d-tool-painter/.../impasto.rs` | o depósito por-traço + o commit na camada (janela `O(stroke)`) |
| `ph2d-tool-painter/.../impasto_light.rs` | **o passe de luz**: `Rig` / `Lamp`, a matemática relativa **por canal** |
| `ph2d-tool-painter/.../impasto_rig.rs` | o **modelo** do rig: `ImpastoLight` / `LightRig`, `MAX_LIGHTS = 4` |
| `ph2d-tool-painter/.../relief_state.rs` | todo o estado por-traço do relevo |
| `ph2d-panel-painter-layers/.../paint_impasto.rs` + `paint_impasto_rig.rs` | os 2 cards (Body / Lighting) |

**Os 3 invariantes que os gates defendem — se um cair, é regressão, não flake:**

1. **Tinta plana é BYTE-IDÊNTICA** (a sombra é *relativa*: dividida pela resposta de uma superfície
   plana). Vale **até sob 4 lâmpadas coloridas saturadas** — a divisão é **por canal**.
2. **Um pincel que não deposita corpo não deposita tinta**, e a tinta que deposita é **opaca** (o filme).
3. **Um limiar pertence à FORMA (a silhueta); a dinâmica MULTIPLICA depois.** Errei isso em dois lados
   opostos do mesmo cano (o corte do pigmento e o peso da luz) e as duas vezes o sintoma foi **um knob
   morto em pressão parcial**.

**Docs vivos:** [`docs/Painter/16_impasto_plano_implementacao.md`](Painter/16_impasto_plano_implementacao.md)
(§14 o filme · §15 a luz por canal · §16 opacidade ≠ espessura · **§17 A FILA** · §18 o rig) ·
[`17_impasto_deposito_pesquisa2.md`](Painter/17_impasto_deposito_pesquisa2.md) (a pesquisa: Photoshop /
ArtRage / Rebelle / Krita) · [`BUGS_painter.md`](Painter/BUGS_painter.md) (**#14** e **#15**).

---

## PARTE III — A FILA DE IMPLEMENTAÇÃO (ordem do Enio, não reordene)

### FILA 0 — 🔴 A UI do rig de luzes está MORTA (**faça isto primeiro**)

**Enio, 2026-07-12 (print):** *"UI não funciona, nem o checkbox nem se pode selecionar outra luz. Mas
coloque na fila para amanhã."*

**Sintoma exato:** no card **Lighting**, os chips `1 2 3 4` **pintam** — e pintam **com o estado certo**
(o print mostra `2· 3· 4·` com o pontinho de "desligada", ou seja, **o snapshot chega bem no painel**) —
mas **não respondem ao clique**. O checkbox **Enable** também não; mas isso pode ser *consequência*: ele
só é pintado quando a lâmpada selecionada é ≠ 1, e não dá pra selecionar outra.

**A matemática está CERTA e gateada** (6 gates, 3 mutações vermelhas). **É seam de UI puro.**

#### O que eu já descartei (não refaça)

1. **Colisão de id** — passei `PAINTER_IMPASTO_LIGHT_1` como `group_id` do segmented **e** como id da
   opção 1. → **DESCARTADA**: `paint_segmented_adaptive` **ignora** o `group_id`; ele só mapeia
   `widget.options` para `paint_segmented_group_adaptive`.
2. **Falta de `store.register` em `populate.rs`** ([[feedback_panel_populate_register]]) → **DESCARTADA**:
   os segmentos de **Depth Source** e **Draw To** também **não** estão em `populate.rs` e **funcionam**.

#### O candidato mais forte, ainda NÃO checado

**A altura do `card_frame`.** O segmented **reflui** (4 chips num painel estreito podem virar 2 linhas —
`measure_segmented_adaptive` existe exatamente pra isso), mas eu dimensionei o card por uma contagem
**FIXA** de linhas:

```rust
// crates/ph2d-panel-painter-layers/src/paint_impasto.rs
let rows = if brush.impasto_rig.selected > 0 { 7 } else { 6 };
let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Lighting", rows);
```

`card_frame` calcula `card_h = pad + title + n_rows * row_adv + pad` e devolve `next_y = y + card_h`.
**Se o conteúdo estourar o card, o CARD SEGUINTE é pintado por cima** — e os hit-rects dele **ganham**.
O print do Enio reforça: o card parece **curto demais**, terminando logo abaixo dos chips.

**Segundo candidato:** a ordem dos arms em `ph2d-panel-painter-layers/src/event.rs::handle_event` (um arm
anterior pode estar engolindo o `Click`).

#### 🔴 A ORDEM DE EXECUÇÃO (não negociável)

1. **ESCREVA O GATE DO SEAM PRIMEIRO.** Headless, via `ph2d-ui-testkit`: **clica** o chip 2 → afirma
   `tool.paint.impasto_rig.selected == 1`; **clica** Enable → afirma `lights[1].on`. **Ele nasce
   VERMELHO** e é a sua prova de diagnóstico. Sem ele, qualquer fix é chute.
2. Só então diagnostique (candidatos acima).
3. Conserte. É **UI pura**: não toca a matemática, e **nenhum dos 6 gates do rig deve se mexer**.

#### A LIÇÃO — e ela é a razão de este item existir

Eu gatei a **MATEMÁTICA** do rig com 6 gates e 3 mutações vermelhas, e escrevi **ZERO gates no SEAM da
UI**. Um teste que **clicasse no chip** teria saído vermelho **antes de o Enio abrir o app**.

> **Um widget novo não está pronto quando PINTA — está pronto quando um teste CLICA nele.**

O seam painel↔tool tem **4+ elos silenciosos** (pintar · registrar hit-rect · registrar no `WidgetStore`
· encaminhar o `Click` em `event.rs` · rotear no tool). Memórias:
[[feedback_widget_is_done_when_a_test_clicks_it]] · [[feedback_painted_is_not_populated_paint_gate]] ·
[[feedback_tool_unit_green_integration_dead]] · [[feedback_panel_populate_register]].

**Enquanto estiver mexendo aí:** o card **Lighting** ganhou 3 linhas novas; **confira o layout inteiro**
(o Body card, o Shine, o `next_y`) — não só o chip.

---

### FILA 1 — Passe de luz na **GPU**

**Por que:** hoje o passe é CPU e custa **~3,4 ms/movimento @2048² · ~3,7 @4096²** (alvo ≤4, kill 8) —
está **dentro** do orçamento, então isto **não é uma emergência de perf**; é a próxima peça de qualidade
e o que libera orçamento pras luzes/IBL futuros.

**O caminho já existe — VERIFIQUEI AGORA:**

- O compositor GPU vive em **`crates/ph2d-render/src/layer_compositor/`** (⚠️ **foundational** — projete
  pra isolamento; ADR-0107).
- Ele já tem `LayerOp::Adjustment { kind: ADJ_* }` e o comentário do contrato diz:
  *"an unknown code is an identity no-op in the shader"* — ou seja, **um `LayerOp` novo é
  retrocompatível por construção**.
- `AdjustmentKind` tem **24 variantes** (contadas agora). O contrato congelado é **≤ 32** ⇒ **8 slots
  livres**. O plano estava certo.
- **O compositor GPU não sabe NADA de impasto** hoje (`grep -rn impasto crates/ph2d-render/src/layer_compositor/` = vazio).

**O que o passe precisa ler:** o relevo composto (`heights` por camada, dobrado em z-order com
`impasto_depth` + `ReliefComposite::Add/Level`, com teto `H_CEIL`), a **cobertura** (`covers` — que é a
**tinta sólida**, `solid_paint`, e **é o peso da luz**), e o **rig** (até 4 lâmpadas × dir/half/tint +
os flats).

**⚠️ O gate que decide se isto pode landar** (DIRETIVA §4): **reconciliação BIT-A-BIT contra a CPU**.
O precedente existe e funcionou: `project_painter_w4_spatial_gpu_bloom_sh` (Bloom/S-H GPU reconciliados
bit-a-bit contra a CPU via dev-dep). **Sem esse gate, não landa.**

**Cuidados que a linha já pagou:**
- **HR-5 (determinismo):** o passe CPU é **transcendental-free** (rotor de 1°, LUT de `pow`). O shader
  tem que dar o **mesmo** número, não um "parecido".
- O contrato **relativo** (tinta plana byte-idêntica, **por canal**) tem que sobreviver ao port — é o
  gate `a_coloured_light_rig_leaves_flat_paint_byte_identical`, e ele não pode ficar CPU-only.
- **Meça em `--release`** (memória: `project_painter_composite_perf_2026_06_03` — GPU `LayerOp` WGSL deu
  1,7 ms contra 55 ms de CPU no Metal; medir em debug mente).

---

### FILA 2 — Persistência do `h` no `ProjectState`

Hoje o relevo **viaja com o documento pintado** (`PaintedDoc` no ECS, Ctrl+S salva camadas+relevo). O que
**falta** é o gap **herdado** (não é bug meu): o save **não persiste os pixels** de
`SpriteSource::Individual` (Painter/Apply) nem `CookedTexture` (KTX2) — só imagens **importadas**
(Atlas). **Fechar isso é o MESMO work item** (CLAUDE.md §5, entrada "Persistência de projeto").

Comece lendo `shells/desktop/src/project.rs` + `project_painter.rs`.

---

### FILA 3 (**exige ordem NOVA do Enio** — NÃO comece sozinho) — Relevo do PAPEL

Faria a luz do impasto ler a altura do papel (`watercolor_noise::paper_height`). **Isso ACOPLA
impasto ↔ aquarela**, e a aquarela é **implementação à parte que o Enio mandou NÃO TOCAR** (plano §2).
**Só com ordem explícita nova.**

---

### FILA ÚLTIMA — 🔴 A TINTA EMPURRADA (o **Push**)

**Enio, 2026-07-12:** *"a tinta empurrada ainda não resolveu. Adiar para o final de toda essa
implementação. Fim da fila."*

**A mecânica está CERTA e é boa** (§13 do plano): real-time (a crista sobe **sob** o pincel), conservativa
(o campo `R₁` soma exatamente zero por construção), **viva** (linear em Push ⇒ mexer no knob é uma
multiplicação) e idempotente (imune ao re-stamp por-frame dos shape editors).

**O que não convence é o DESENHO da tinta deslocada.** **Não foi diagnosticado.** Não mexa antes de tudo
o resto — é ordem.

Quando chegar a hora: comece **renderizando e olhando** (§7), não pela teoria. E o candidato que a
pesquisa aponta e que **não** implementamos: o **bow wave** — a tinta empurrada **à frente** da ponta
(hoje o traço desloca por-dab, lateralmente; uma lâmina em movimento não deixa tinta em pé na frente de
si, mas empurra uma onda **adiante**). Ver `17_impasto_deposito_pesquisa2.md` **§3, mecanismo 6** (IMPaSTo NPAR 2004:
advecção conservativa + velocidade de pressão `v_p = −c∇p` ⇒ *a tinta empurrada acumula na FRONTEIRA do
traço*; e WetBrush SIGGRAPH Asia 2015).

---

## PARTE IV — Herdados (não são desta linha; não comece por eles)

- **Bug #11** — Per-Layer Color, listras retangulares **intermitentes**. Dormente, armadilha armada,
  composite CPU **provado limpo**. `BUGS_painter.md`.
- **`HANDOFF_per_layer_color_perf_artifacts.md`** — perf de camadas-como-brush (re-stamp da forma inteira
  por-move × N camadas).
- **Bug #13** — abertos da varredura; nenhum é crash.

---

## PARTE V — Como você FECHA a linha

1. Gate batched **1×**: `cargo test --workspace` (⚠️ **não** o impacted) + `cargo clippy --all-targets`.
2. Perf, se tocou o passe de relevo/luz:
   ```bash
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
     cargo test --release -p ph2d-tool-painter --lib -- impasto_perf_kill_criterion --ignored --nocapture
   # alvo <=4 ms/movimento, kill 8
   ```
3. **Smoke** — feature nova **ship com o exemplo que a demonstra**; não peça pro Enio montar:
   ```bash
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
     PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
   ```
4. **Escreva o handoff de integração** (DIRETRIZ §1.5.9). Modelo: o desta jornada,
   [`HANDOFF_line_Painter_integracao_2026-07-12_FECHAMENTO.md`](HANDOFF_line_Painter_integracao_2026-07-12_FECHAMENTO.md)
   — ele lista o que o integrador precisa (base, nº de commits, superfície foundational tocada, contratos,
   gates, **as armadilhas**, e o que ficou aberto).
5. **PARE.** Reporte *"linha pronta + handoff"* e **espere a ordem do Enio.**

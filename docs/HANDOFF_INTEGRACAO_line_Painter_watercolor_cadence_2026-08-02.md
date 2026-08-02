# HANDOFF DE INTEGRAÇÃO — `line/Painter`, a cadência da aquarela (2026-08-02)

> Para o **agente integrador**. A linha NÃO integrou e NÃO fez push.
> Detalhe técnico e as medições: [`docs/Painter/28_otimizacoes_o_que_funcionou.md` §5.71](Painter/28_otimizacoes_o_que_funcionou.md).

## 1 — Identificação

| | |
|---|---|
| Branch | `line/Painter` |
| Worktree | `Worktrees/line-Painter` |
| HEAD | `6a6a21bcf` |
| Base | `main` a `a9f5977e9` (rebase era no-op ao assumir; **re-rodar `git rebase main` antes de integrar**) |
| Commits desta sessão | **4** (`56f9f372f`, `fd0155916`, `873d68806`, `6a6a21bcf`) |
| Commits acumulados da linha | **57** (53 herdados do motor de undo + 4 desta sessão) |

## 2 — O que entra

**A tarefa era *avaliar* o modo Watercolor e tentar otimizá-lo.** A avaliação achou dois defeitos de
custo, os dois medidos pela porta do produto (`on_canvas_pointer` / `paint_tick`), e os dois curados.

### 2.1 A lavagem reconstruía por EVENTO de ponteiro, e o doc dela dizia QUADRO

`apply_watercolor` reconstrói a lavagem inteira sobre a base congelada, e a janela dela é padeada pelo
**raio de influência** — do tamanho da pegada. Ela rodava dentro de **cada `PointerPhase::Move`**, então
encolher o passo do mouse não encolhia a passada: só multiplicava quantas vezes ela acontecia.

Mesmo traço, 30 quadros (0,5 s), r=100, canvas 4096² — variando só quantos eventos caem em cada quadro:

| dispositivo | ev/quadro | antes | **agora** | ganho |
|---|---|---|---|---|
| 120 Hz | 2 | 130,9 ms | **92,2 (1,00×)** | **1,42×** |
| 240 Hz | 4 | 146,6 | **89,9 (0,97×)** | 1,63× |
| 480 Hz | 8 | 179,0 | **90,6 (0,98×)** | 1,98× |
| 960 Hz | 16 | 234,1 (1,79×) | **91,3 (0,99×)** | **2,56×** |

A rota nova é **plana em 8× a taxa do dispositivo**: o custo passa a depender do DESENHO, não do mouse.

⚠️ **Byte-idêntico, e MEDIDO:** o mesmo caminho em 15 e em 120 eventos pinta telas que diferem em
**0 bytes**. Isso também refuta a hipótese pior — a aparência da aquarela **não** dependia do hardware.

⚠️ **Latência ZERO:** o tick roda em `render_loop` ~1198, depois do flush de ponteiro (~698) e **antes**
do upload do preview (~3397). O quadro que recebeu os Moves é o quadro que mostra a tinta.

### 2.2 O pen-down alocava 268 MB para reproduzir uma cor chapada

`composite_below` preenchia o acumulador `[f32;4]` **antes** de perguntar se há algo abaixo da âncora.
Num documento de UMA camada não há — e 335 MB de tráfego produziam a cor de papel.

**pen-down 81,5 → 26,4 ms** (r=20) · 82,1 → 36,7 (r=100) · 112,3 → 62,2 (r=400), a 4096².

⚠️ O ganho é do documento de **uma camada**. Com camadas abaixo o caminho longo continua — corretamente.

## 3 — Foundational tocado

**NENHUM.** Todo o diff mora em `crates/ph2d-tool-painter/`. Zero `Cargo.toml`, zero dep nova, zero
crate nova, nenhum ADR, nenhum id/token/i18n, **nenhum schema** (`PROJECT_SCHEMA` **não** foi tocado).

## 4 — Contratos congelados (CLAUDE.md §6)

**Nenhum encostado.** Conferido por `cargo test -p ph2d-editor-core --release` (inclui
`architecture_tool_contract_surface`): `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` /
`PanelEvent=4` intactos.

## 5 — Superfície nova (para o integrador detectar colisão)

| símbolo | onde | o que é |
|---|---|---|
| `paint::watercolor_field::WashCadence` | `watercolor_field.rs` (fim) | `{ per_event: bool, composites: u32 }` — sub-estado, no padrão do `WetSessionStyles` que já mora ali |
| `PainterTool.wash` | `tool/mod.rs`, ao lado de `canvas_rgba` | o campo |
| `pub(crate) mod watercolor_field` | `paint.rs` (era `mod`) | visibilidade alargada para o `PainterTool` alcançar a struct |
| `compose::encode` | `compositor/compose.rs` | era privado, virou `pub(super)` para o gate da substituição |
| `watercolor_cadence_tests` | filho de `watercolor_field` | 3 gates novos |

⚠️ **Não há campo `owed`.** *"Chegou um move neste quadro?"* já é `moved_this_frame`, que o `paint_tick`
lê como `parked` — um segundo campo seria um segundo lugar para o mesmo fato, com ciclo de vida próprio
a acertar.

## 6 — Gates novos e as provas de mutação

**`watercolor_cadence_tests`** (3): a **CONTA** (um composite por quadro — um CONTADOR, não um relógio:
uma razão sobre passadas de ~1 ms mede o escalonador desta máquina) · o **QUADRO** (a lavagem está viva
no quadro que recebeu os Moves) · a **FIGURA** (byte a byte em qualquer taxa de polling).

**`compositor::tests`** (2): o round-trip de byte do sRGB é a **identidade nos 256 valores** · o
preenchimento chapado **é** o que o acumulador teria codificado (alfa incluso).

**2 mutações, 2 sangram:**

| mutação | efeito |
|---|---|
| o tick não paga a dívida (`stamped \|\| …` → `stamped`) | **8 testes caem**, incluindo o gate do QUADRO |
| `WashCadence { per_event: true }` por default | cai **só** o gate de cadência — os outros dois passam nas duas rotas, **por desenho** |

⚠️ **A rota de ablação (`WashCadence::per_event`) existe por dois serviços**, no precedente do
`Sim::order_invariant` (ADR-0147): um A/B cross-process atribuiria a deriva desta máquina (o mesmo passo
de produto já foi medido a 14,5 e 30,2 ms) à mudança; e o gate de cadência precisa de uma alavanca que o
faça ir VERMELHO. Produto é sempre `false`.

## 7 — ⚠️ Seis fixtures existentes foram corrigidas

Elas dirigiam Moves e **nunca fechavam um quadro**, então mediam a cadência ANTIGA — e o doc de uma
delas já dizia *"local to the **frame's** new dabs (wet_edges `renderFrame`)"*. Fecham quadros agora
(helper `frame()` em `tests.rs`, uma regra num lugar só) e **ficaram mais fortes**: provam também que o
tick paga a dívida. As seis: `watercolor_wash_is_live_before_pen_up` ·
`watercolor_live_recomposite_is_local_to_the_frame` · `watercolor_moving_preview_restores_the_old_position` ·
`watercolor_incremental_composite_matches_full_recompose` · `..._with_water` ·
`watercolor_granulation_bake_settles_beyond_the_live_preview`.

## 8 — O que só o `ship.sh` pega

- **Dívida de `fmt` PRÉ-EXISTENTE, já paga no commit `56f9f372f`:** cinco arquivos desta linha estavam
  commitados sem passar pelo rustfmt **pinado** (1.95, via `rust-toolchain.toml`) — resíduo dos 53
  commits `--no-verify`. Puro reflow, conferido diff a diff. **Sem isso o integrador herdaria um `✗`.**
- `cargo machete` / `deny` / `audit` / `typos` não foram rodados aqui (nenhuma dep mudou, então o risco
  é baixo, mas o `ship.sh` é a autoridade).

## 9 — Verde local

| gate | resultado |
|---|---|
| `cargo test -p ph2d-tool-painter` **debug** | 958 · 0 falhas |
| `cargo test -p ph2d-tool-painter --release` | **959** · 0 falhas |
| `cargo test -p ph2d-editor-core --release` | verde (inclui contratos + LOC) |
| `cargo test -p ph2d-host-desktop --release` | **77 binários**, 0 falhas |
| `cargo clippy -p ph2d-tool-painter --all-targets` | limpo |
| `cargo fmt -p ph2d-tool-painter --check` | limpo |
| `architecture_workspace_file_loc_cap` | verde |

⚠️ **`paint.rs` estava EXATAMENTE em 700 (o teto)**, então qualquer linha o quebrava. Foi por isso que o
estado virou sub-struct num irmão e os gates viraram filhos de `watercolor_field`: **o arquivo volta a
700, idêntico ao HEAD.**

## 10 — O que SMOKE-TESTAR

```
env PH2D_WETPAINT_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
⚠️ O smoke abre em **Digital** de propósito — escolha **Watercolor** no dropdown de Paint Mode.

1. **Canvas 4096, pincel GRANDE (raio 200-400), traço longo.** A pergunta é de mão: o traço tem de sair
   **liso**, e o começo do gesto não pode engasgar. Se o seu mouse/tablet for de alta taxa, é
   exatamente aí que a cura paga mais.
2. **O pen-down.** O primeiro toque de cada traço era ~80-112 ms; deve ter sumido como hitch.
3. **A APARÊNCIA não pode ter mudado** — nem a borda, nem a granulação, nem o escorrido. Isto está
   gateado em byte-identidade, mas *o olho é o oráculo final*: se algo mudou, é bug meu.
4. **Um documento com VÁRIAS camadas** (o early-out do pen-down não se aplica ali): a lavagem tem de
   continuar lendo as camadas de baixo como chão.

## 11 — Aberto, com número

- **O `pour_canvas_wet` ainda caminha o rect CUMULATIVO** uma vez por quadro ⇒ o custo por quadro cresce
  **1,23× / 1,32× / 1,51×** do 1º para o 4º quarto (traços de 24/48/96 quadros). A cura tem a mesma
  forma, mas **a premissa não foi verificada** (o filtro de dono por recência pode mudar a elegibilidade
  de um texel no meio do traço) ⇒ wave própria, com gate de byte-identidade de `canvas_wet`. Sonda
  pronta: `measure_whether_the_frame_cost_grows_along_the_stroke`.
- **O WARP segue sendo 56%** do que a aquarela cobra sobre o Digital e **não tem caminho de CPU**: os 9
  taps de AA foram a CURA da borda serrilhada e cortá-los está fora de discussão. Aproximar o warp
  dentro do texel é a classe que este repo já mediu e **rejeitou duas vezes** ⇒ exige oráculo de
  APARÊNCIA e ordem do Enio.
- **`DragDot`/`Anchored`/`Line` compõem por evento mesmo com a cura** (o `clear_wet_coverage` dobra o
  rect cumulativo no do quadro). No app eles são **coalescidos** pela shell, então não é defeito vivo.
- ⚠️ **A coluna "razão" da varredura de raio é inutilizável a r≥200:** o `moves` do **Digital** cai
  (34,3 → 11,8) em vez de crescer, o que é a assinatura do confound de contagem de dabs. Os números
  ABSOLUTOS da aquarela valem; a razão contra o Digital ali, não. **Não expliquei essa anomalia** — ela
  é do controle, não da aquarela, e pode ser um achado do Digital por conta própria.

# HANDOFF DE INTEGRAÇÃO · `line/motion-value` · **bloco Z** — 2026-08-23

> **A linha NÃO integrou e NÃO pushou** (`CLAUDE.md` §0.7). **Vinte e um** commits locais, à espera
> de ordem explícita do Enio. **Quatro blocos** no mesmo dia: os TETOS (§1), a folha 11 (§0-bis, §9),
> o defeito que o smoke da folha 11 devolveu (**§0-ter — leia-o primeiro se você integra**: ele é o
> único item deste handoff que muda o pixel de **toda sprite do app**) e a folha 06 (§0-quater).

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` · **branch:**
`line/motion-value` · **base:** `main` em `35f937cb2`.

---

## §0-bis — SEGUNDO BLOCO no mesmo dia: **a folha 11 (fx raster)**

Depois do bloco Z, a mesma linha fechou **seis das sete** células da folha 11 —
7 P2 → **1**, e a conferência de 82 para **76**. Registo: as próprias células, que ficaram
densas de propósito (é a forma da conferência), e o §9 abaixo.

| célula | cura | onde |
|---|---|---|
| modo da sombra | `fx.drop_shadow::shadow_blend` | STREAM |
| eixo da lente | `fx.rgb_split::center_x`/`center_y` | STREAM |
| raio limpo | `fx.rgb_split::start` | STREAM |
| operação do halo | `fx.glow::operation` (`Add`/`Screen`) | TELA |
| fonte do bright-pass | `fx.glow::source` (`Luminance`/`Alpha`) | TELA |
| cor do halo por rampa | `fx.glow` + LUT de 512 texels | TELA |
| ⏳ *dirt texture* | **fica**, com o preço corrigido por medição | — |

**Cena de smoke: `=84`.**

---

## §0-ter — O smoke devolveu um DEFEITO, e ele é do renderer, não do nó

**Enio, 2026-08-23, sobre a `=84`:** *"shadow multiply parece não obedecer o alpha da cor"*.

⚠️ **Este é o item de MAIOR alcance do handoff inteiro** — ele muda como **toda sprite do
app** com `BlendMode::Multiply` compõe em alfa parcial (`ph2d-render`, caminho partilhado).
Registo completo, com hipóteses e lições:
[`BUGS_motion_nodes.md` Bug #4](../BUGS_motion_nodes.md).

**O nó estava inocente.** O `fx.drop_shadow` escreve a alfa do fantasma correctamente. O
defeito era o par de fatores do `Multiply` em
[`ph2d_render::pipeline::blend_state_for`](../../../crates/ph2d-render/src/pipeline.rs).

**Medido antes de tocar** (fundo 55, frente 128, byte do centro):

| modo | α=0,00 | α=0,25 | α=0,50 | α=0,75 | α=1,00 |
|---|---|---|---|---|---|
| `Add` · `Subtract` · `Screen` · `Mix` | **55** | … | … | … | … |
| **`Multiply` (antes)** | **0** | 3 | 6 | 9 | 12 |
| **`Multiply` (depois)** | **55** | 44 | 34 | 23 | 12 |

Não era *"não obedece"*: era **invertido**. `α = 0` pintava **preto**, subir a alfa
**clareava**, e não havia valor em que a sombra sumisse.

**Mecanismo.** O `sprite.wgsl` emite `vec4(rgb·α, α)` — fonte **pré-multiplicada**, que
codifica *"não contribui"* como **zero**. Isso dá a resposta à alfa **de graça** a todo modo
cujo elemento neutro é `0` (`Add`, `Subtract`, `Screen`, o `over`). O neutro do `Multiply` é
**`1`**: com `dst_factor: Zero` a pré-multiplicação levava o produto para preto em vez de
para nada. Cura: `src: Dst`, `dst: OneMinusSrcAlpha` ⇒ `dst·(α·src + 1 − α)`.

⚠️ **As duas colunas coincidem em `α = 1`** — é isso que garante que nada opaco mudou, e é
exactamente o ponto que o gate antigo media.

**Para quem integra:**

1. ⚠️ **Um golden/regressão de imagem de OUTRA linha que contenha uma sprite `Multiply` com
   alfa parcial vai mudar de valor, e a mudança é a CURA.** Nenhum caso opaco se move.
2. O gate `blend_modes_composite_as_advertised` **fica como está e continua verde** — ele
   não estava errado, estava incompleto.
3. Nada de contrato congelado foi tocado: `BLEND_PIPELINE_COUNT` continua `6`, a assinatura
   de `blend_state_for` é a mesma, e `ph2d_ecs::BlendMode` não se moveu. Só o par de
   fatores de **uma** tag mudou.

**Gates novos** ([`blend_mode_regression.rs`](../../../crates/ph2d-render/tests/blend_mode_regression.rs)):

| gate | o que prende |
|---|---|
| `zero_alpha_is_absence_in_every_mode` | `α = 0` devolve o fundo **medido no mesmo passe** (`fg = None`), nos seis modos, com controle positivo |
| `the_multiply_alpha_slider_runs_from_the_backdrop_to_the_full_product` | monotonia do curso + excursão real |
| `measure_alpha_response_of_every_mode` | a sonda que imprime a tabela (não afirma nada) |
| `the_alpha_row_varies_the_alpha_and_nothing_else` (shell) | a linha 3 da `=84` varia **só** a alfa |

**Prova de mutação.** Os dois primeiros foram vistos **VERMELHOS sobre o defeito real**,
antes da cura — a espécie mais forte, porque a mutação não foi sintética. O gate da cena foi
mutado duas vezes (alfas iguais ⇒ `0.85 vs 0.85`; modos diferentes ⇒ `so' a alfa muda`),
vermelho nas duas, restaurado por `git checkout` sobre commit limpo.

**E a fronteira fica registada** no `keep_dst_alpha`: um par de fatores fixos **não exprime**
o `Cs' = (1−αb)·Cs + αb·B` da W3C (precisa da alfa do DESTINO como termo). A fórmula inteira
já existe e está **correcta** onde o fundo é translúcido de propósito
([`layer_composite.wgsl`](../../../crates/ph2d-render/src/shaders/layer_composite.wgsl)).
⛔ *Quem tentar "consertar" a divergência de faixa parcial com outro par de fatores não vai
conseguir — o caminho é o passe programável.*

**A cena `=84` ganhou a terceira linha (ALFA)**, porque o defeito só é julgável a olho num
PAR: as duas metades MULTIPLICAM e só a alfa muda (15% × 85%). Uma metade só diria *"está
escuro"*.

---

## §0-quater — QUARTO BLOCO: **a folha 06 (animadores)**

Uma pergunta só, e cinco células: *o animador não sabia a FORMA que o artista desenha,
nem escrever os DOIS eixos.* A folha 06 vai de **11 P2 para 7** (e o placar de 76 para
**72**); ✅ 228 → **232**.

| célula | cura | onde |
|---|---|---|
| onda `Custom` | `motion.oscillator::wave = 5` + text param `curve` | CPU + **device (LUT)** |
| ease `Custom` | `motion.stagger::ease_curve = 8` + text param `curve` | CPU + **device (LUT)** |
| `Separate Channels` | `motion.wiggle::channel = 4` — **FECHA a célula** | CPU + device |
| `Separate Channels` | `motion.noise::channel = 4` — metade (falta *Use Layer as Seed*) | CPU + device |
| alinhar pela NORMAL | `motion.path::align = 2` | CPU |

**Cena de smoke: `=85`** · **`PH2D_MOTION_NODE_PATH_SMOKE=3`** (a normal precisa da curva
desenhada, que só o smoke próprio do `motion.path` encena).

### O que o integrador tem de saber

1. ⚠️ **Nada é breaking.** Os três enums são **apendados** (`wave` 0..4 · `ease_curve`
   0..7 · `channel` 0..3 continuam a valer o que valiam) e o `align` era um `Toggle` cujos
   `0`/`1` continuam a ser *nada*/*tangente* — o `align >= 0.5` de ontem e o `!= 0` de hoje
   concordam em todo documento que existe.
2. ⚠️ **`ph2d-curve` é dependência NOVA de `ph2d-node-motion-oscillator` e
   `-stagger`**, e `ph2d-gpu-cook` ganhou-a como dev-dep. O `Cargo.lock` mexeu.
3. ⚠️ **O `apply_channel_delta` continua BYTE-IDÊNTICO nas cinco crates-folha.** O canal
   novo não passa por ele: quem despacha é o `eval`, para uma `apply_channel_delta_xy`
   irmã. *Uma cópia deliberada que só duas das cinco recebem deixa de ser uma cópia* — e
   as três cópias do WGSL de um destes nós **já divergiram uma vez**, com um grafo a
   correr uma taxa no device e outra na CPU.
4. ⚠️ **Dois números vivem escritos DUAS vezes** (const de Rust + literal na string WGSL):
   `AXIS_SEED_OFFSET = 7919` e `AXIS_ROW_OFFSET = 7919,5`. Quem os prende são os gates
   `the_wgsl_carries_the_same_axis_offset_as_the_rust` (que derivam a agulha da const) **e**
   a paridade no device — nunca a memória de quem edita.

### As decisões que custaram medição

**A curva vai ao DEVICE, não derruba o nó para a CPU.** A saída fácil era o kernel
declarar-se `applicable: false` na forma `Custom`; está **rejeitada** — o `field.remap` já
mostra que um `LutSpec` resolve sem tirar o nó do caminho rápido, e um animador é
precisamente o nó que não se quer perder dali. ⚠️ **E o preço foi medido, porque a LUT é
assada em TODO cozimento** (o `build_luts` não pergunta o modo): **0,0029 ms por cozimento
por nó = 0,018% de um quadro**. Fica sem predicado. Sonda `measure_lut_build_cost_per_cook`.

**A resolução da LUT é 512, o dobro do `field.remap`.** Uma tabela uniforme converge com
`1/n` numa esquina e não converge de todo num degrau — o que a densidade encolhe é a
LARGURA da banda errada. Aqui a tabela é lida como MOVIMENTO ao longo de um ciclo (não como
a cor de um pixel), então essa banda vira um solavanco visível.

**A `natural_range` teve de responder pela forma nova.** A `Custom` é unipolar `[0,1]` (o
quadrado do editor) e as quatro clássicas são bipolares: sem declarar a polaridade, a conta
que toda a gente faz de cabeça entregaria `Min/Max = [−2,3]` como `[0,5, 3,0]` — metade da
excursão com o piso ao CENTRO, em silêncio. É a armadilha do `Spike` que esta folha já
pagou, reaberta por uma forma nova.

**A ease `Custom` ignora o `ease_dir`**, como o `Linear`, e por razão de PRODUTO: a curva
desenhada é a declaração de intenção do artista. ⭐ **E isso não custou uma linha de UI** —
o `PARAM_GATES` enumera as famílias que *usam* a direção (`1..7`), então a nona nasce com o
`ease_dir` escondido: *o gate estava escrito na forma certa antes de existir o nono caso.*

**A célula do `motion.path` dizia «NÃO» sobre um facto que o código do lado já tinha.** A
justificativa era *"a normal, que nada publica"* — e o `motion.spline_wrap` computa
`un = [-ut.y, ut.x]` da MESMA curva desenhada, com o doc deste nó a chamar-lhe *"o segundo
consumidor"*. **A nona célula desta folha a envelhecer assim.**

### ⚠️ Duas RÉGUAS corrigiram-se antes do algoritmo, e as duas por RESOLUÇÃO

1. **A barra da decorrelação estava colada em ZERO.** Primeira versão: `0,25`, e o device
   mediu `−0,222` — 12% de folga. A sonda `probe_r_vs_scale` disse porquê: o resíduo cai
   de `0,120` para `0,009` só ao AFINAR o campo. Um acoplamento real não se dissolve assim;
   **ruído de amostragem sim** — um campo COERENTE tem tantas amostras independentes
   quantas FEIÇÕES, não quantos elementos. ⇒ a barra passou a medir a distância a **`1`**
   (o valor exacto do defeito), não a proximidade de `0`; uma barra em `0,15` mediria o
   tamanho da grelha e reprovaria numa fixture legítima mais grosseira.
2. **A régua da cena exigia mais precisão do que a fixture tem.** Ela media a POSIÇÃO DO
   PICO com margem `0,1`; com 15 peças o índice anda de `1/14 = 0,071`, e os dois picos
   ficam a **um passo** um do outro. A régua nova é a que o olho usa (a curva autorada é
   unipolar e fica toda ACIMA da linha de repouso; a senoide atravessa-a), com a assimetria
   como segunda metade e margem de **meio passo**.

### Prova de mutação (as cinco células)

| mutação | o que morre |
|---|---|
| a `Custom` deixa de declarar a polaridade | o piso vira **`0,5`** — o número exacto que o gate nomeia como a conta errada |
| a `Custom` do stagger cai no caminho da direção | `the_custom_ease_is_the_authored_shape` |
| a onda `Custom` ignora a curva autorada | `a onda desenhada É a curva autorada` |
| o stagger nunca lê o text param | `an_unset_custom_ease…` + a forma |
| os dois eixos com o MESMO seed | `r = 0,9999999` — o defeito, exacto |
| o deslocamento de linha deixa de ser fracionário | **dois** gates (a propriedade **e** o gémeo do WGSL) |
| o modo `Normal` cai no ramo da tangente | `diferenca 0` onde tem de ser 90° |

### ⚠️ E um erro meu que REPETIU

Escrevi a cena nova em `motion_state_conferencia_demos_shape.rs` — e aquele nome já era a
cena **`=55`** (Pulse Width / Offset), **da mesma folha 06**. Dois arquivos por cima, o
`Write` a responder *"updated"* nos dois. É exactamente o que a memória escrita **ontem**
manda conferir, e o gatilho é o mesmo: *quanto melhor o nome, maior a chance de ele estar
ocupado*. O compilador acusou (`defined multiple times`), restaurei por `git checkout --`
com o meu copiado ao lado, e o módulo passou a chamar-se `drawn`. ⇒ **a memória ganhou o
passo que faltava**: um `ls` do diretório é GATILHO de todo `Write` num caminho não lido,
não o item 1 de uma lista — *uma lista não corre*.

---

## §1 — O que entrou, em uma frase

**Todo teto deste catálogo passa a dizer de que RECURSO ele é** (`CLAUDE.md` §0.0) — 27 params
curados em 11 crates, os dois grampos de integração medidos, e a legenda das cenas de smoke
mudou-se para o canvas. Conferência 89: **89 → 82 P2** (7 células, 4 folhas).

Registro completo: [`docs/Motion Nodes/91_os_tetos_que_ninguem_mediu.md`](../91_os_tetos_que_ninguem_mediu.md).

---

## §2 — Os commits (ordem de aplicação)

| sha | o que |
|---|---|
| `311176ba8` | folha 04: o `falloff` do `motion.kaleidoscope` REFUTADO por medição |
| `1d1634cdb` | a porta que promete um CAMPO e lê só o elemento 0 (`spline_wrap`, `lattice`) |
| `e6e324509` | `motion.wave`: a altura escolhe o canal (`height_channel`) |
| `029a939be` | cena `=83` — o campo que era um número |
| `079305096` | **bloco Z**: 25 tetos de precisão + o `rate` + o ângulo + os dois `MAX_DT` |
| `ef97264af` | a legenda das cenas de smoke vai para o CANVAS |
| `e947b6c98` | o teto de paradas do gradiente ganha derivação (e a medição confirmou o 8) |
| `e04f092b5` | o doc 91 + as 7 células fechadas + as contagens reconciliadas |
| `7538b0d3d` | o handoff do bloco Z + duas flakes novas no §5.0 |
| `32d813b84` | **folha 11**: o modo da sombra e a lente do `rgb_split` (STREAM) |
| `ea8818872` | **folha 11**: a operação e a fonte do halo + o gate de WGSL que faltava |
| `57e3174ec` | **folha 11**: a cor do halo por RAMPA (LUT de 512 texels, medida) |
| `+2` | a cena `=84`, a folha 11 fechada e o split do `motion_fx.rs` por HR-18; e duas memórias |

---

## §3 — Superfície de colisão (o que o integrador tem de saber)

### §3.1 — ⚠️ **Um TOKEN novo** (o único item foundational-ish)

`docs/design/tokens.json` ganha **`chrome.panel-min-w: 220`**, re-exportado como
`ph2d_tokens::PANEL_MIN_W_PX`. O valor **já existia** — era um `const MIN_W` privado *dentro* de
`clamp_panel_rect`, e o comportamento não muda um pixel.

⚠️ **Se outra linha tocou `tokens.json`, este é o ponto de merge.** É um acréscimo de uma linha
num bloco ordenado; a lista de re-export em `ph2d-tokens/src/lib.rs` é alfabética e o nome novo
entra entre `PANEL_HEAD_PAD_PX` e `PANEL_RADIUS_PX`.

### §3.2 — Crates tocadas (11 crates-nó, 3 de infra, 1 shell)

`ph2d-node-{field-box, field-radial-sweep, field-remap, force-attractor, force-buoyancy,
force-vortex, force-wind, motion-emitter, motion-four-point-warp, motion-integrate, motion-move,
motion-spherize, motion-voronoi, sim-spawn, sim-step, value-lfo, registry-init}` ·
`ph2d-{tokens, editor-core, panel-motion-params, gpu-cook}` · `shells/desktop`.

⚠️ **Quase toda a edição é APENDAR uma `static PARAM_HARD_MAX`/`PARAM_HARD_MIN` e uma linha de
`register_*`** — colisão de mesmo-símbolo é improvável, e um merge textual que perca uma das duas
metades é apanhado pelo gate (que exige a entrada E o número).

### §3.3 — Dois splits por HR-18

- `ph2d-node-field-remap/src/params.rs` (novo) — hints/gates/grupos/teto saem do `lib.rs` (686 → 503)
- `ph2d-node-motion-four-point-warp/src/bounds.rs` (novo) — os 16 limites dos 8 cantos (681 → 693)

### §3.4 — Uma dev-dependency nova

`ph2d-node-registry-init` ganha `ph2d-core` em `[dev-dependencies]` (só o gate dos tetos, que mede
à cadência de `DEFAULT_HZ`). **`Cargo.lock` mexe.**

---

## §4 — MUDANÇAS DE COMPORTAMENTO (leia antes de integrar)

### §4.1 — ⚠️ O `MAX_DT` dos dois integradores: `0,1` e `0,05` → **`0,03`**

**Em regime é byte-idêntico** (o tique fixo é `1/60 = 0,0167`, muito abaixo do grampo, e o
`FixedStep` da casa entrega um tique por cozedura mesmo num ecrã lento). O que muda é **quanto de
um SCRUB o sim absorve** — e absorver é a resposta certa ali.

A medição (doc 91 §5): a `0,1`, o laço fechado real com uma força que se alcança **arrastando**
atira uma grelha nascida em raio `1,0` a **127,19**; o irmão `sim.step`, a `0,05`, segurava a mesma
cena em `2,49`.

⛔ **`motion.spring`, `motion.boids` e `motion.wave` NÃO foram tocados.** O `spring` deriva do
`MAX_DT` dele **três** tetos medidos e é wave própria; os outros dois ficam registados como dívida.

### §4.2 — ⚠️ Três testes deixaram de estar pinados a literais

`motion.integrate` tinha três gates com `dt` escrito à mão (`0.1`, `0.05`, `0.1`) que reprovavam
**sobre produto correcto** quando o grampo desceu. Passam a derivar do `MAX_DT`.

### §4.3 — ⚠️ A fixtura do mar da paridade CPU/GPU: `density 40 → 90`

Com o grampo mais apertado, dois tiques deixaram de mover o campo acima do piso de **vacuidade**
(`0,0923` contra `0,1`) — e esse piso é o que impede o gate de comparar dois campos congelados.
⛔ **Alargar o PERCURSO está REFUTADO por medição** (4 tiques dão `0,00526 > ε`, 3 dão `0,00266`):
aquela fixtura é caótica **em número de passos**, e o comentário dela já o dizia. A cura é a FORÇA
(paridade final: `2,3e-4`, 8× de folga sob o ε).

### §4.4 — Nada de UI muda de aparência

Os `ParamHardMax`/`ParamHardMin` só alargam o campo **digitável**; nenhum `ParamUiHint` foi tocado,
então **todo arrasto é idêntico**. A legenda das cenas de smoke é chrome novo e é **no-op** quando
nenhuma cena publicou (todo arranque normal do editor).

---

## §5 — Gate de fecho (batched, 1×)

| | |
|---|---|
| `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast` | **17.893 testes · 17.892 ✓ · 1 ✗** |
| clippy `--all-targets --all-features`, alvo DERIVADO do diff (**29 crates**) | **0** |
| `cargo fmt --all` | limpo |
| `typos crates/ shells/ docs/Motion Nodes/ CLAUDE.md` | **0** |
| `file_loc_caps` · `architecture_widget_loc_cap` · `architecture_panel_loc_cap` · `architecture_workspace_file_loc_cap` | ✓ |
| `placar_conferencia.py` | verde · **76 P2 · 4 P1 · ✅ 228** |
| paridade CPU/GPU de sim (`--ignored`, RTX) | **29/29 ✓** |
| pipelines do halo num **device real** (as 2 operações, as 2 fontes, com e sem LUT) | ✓ |
| `doc-index.sh` · `architecture_docs_paths_and_smokes_resolve` | ✓ |

⚠️ **O `1 ✗` da corrida final é a flake nº 2 do §5.0** (`a_round_live_offset_costs_like_the_other_joins`,
`ph2d-vec-boolean`) — verde **3 de 3** sozinha, em crate que esta linha não toca. As duas do bloco Z
(a máscara do Painter e o zero-alloc da timeline) não reapareceram nesta corrida, o que é o
comportamento de uma flake e não de uma regressão.

### ⚠️ Os 2 ✗ são FLAKES pré-existentes, em crates que esta linha não toca

| teste | crate | sozinho |
|---|---|---|
| `the_mask_stroke_cost_does_not_follow_the_canvas` | `ph2d-tool-painter` | **4 de 5 ✓** (é flaky mesmo sozinho) |
| `apply_from_doc_is_zero_alloc_steady_state` | `ph2d-timeline` | **3 de 3 ✓** |
| `a_wet_move_costs_what_the_footprint_costs...` | `ph2d-tool-painter` | **3 de 3 ✓** (já listada no §5.0) |

As duas primeiras foram **acrescentadas ao `CLAUDE.md` §5.0** como a 5.ª e a 6.ª — a de zero-alloc
é espécie nova naquela lista.

⚠️⚠️ **E há um achado de PROCESSO ali:** a primeira corrida (com fail-fast) parou em **11.240 com
1.007 testes por correr**. *Um vermelho de flake esconde o resto da suíte.* O `--no-fail-fast` é o
que torna o gate batched uma medição em vez de uma amostra — está escrito no §5.0 agora.

---

## §6 — Prova de mutação (gates novos)

| gate | mutação | RED |
|---|---|---|
| `every_scene_labels_both_halves_on_opposite_sides` | as duas fichas do mesmo lado | ✓ |
| `every_caption_is_chip_sized` | a frase inteira do terminal numa ficha (70 chars) | ✓ |
| `every_precision_bound_param_types_to_the_measured_ceiling` | apanhou, ao vivo, um erro MEU de sinal (`-A - B` em vez de `-(A - B)`) na reescrita dos literais | ✓ |

⚠️ **O terceiro não é uma mutação encenada: é o gate a fazer o trabalho dele durante este bloco.**
Ao trocar os literais truncados pela aritmética exacta (`2_097_152.0 - 0.125`), o `replace` deixou
os pisos negativos como `-2_097_152.0 - 0.125`, que é outro número. *Um piso simétrico escrito como
literal é um sinal à espera de se perder;* a forma segura é a do `bounds.rs`, com um `const REACH` e
`-REACH`.

---

## §7 — O que fica ABERTO (dívida nomeada)

1. ⏳ **`motion.boids::MAX_DT` e `motion.wave::MAX_DT`** continuam a `0,1` **por medir** — copiaram
   o número sem derivação, como os dois que este bloco curou. A sonda `excursion` já está escrita
   (`integrator_ceilings.rs`) e serve-lhes com outro grafo.
2. ⏸️ **`motion.spring`** — mexer no `MAX_DT` dele move **três** tabelas medidas (`friction`,
   saturação do sub-passo, `tension`). Wave própria, com o oráculo de três braços dele.
3. **`MAX_CURVE_POINTS = 8`** (o irmão do teto de paradas, no mesmo arquivo) continua sem
   derivação: *"matches the field.remap text param's practical ceiling"*. O editor de curva tem o
   MESMO `GRAB_R = 9,0`, então a conta é a mesma — só não foi feita.
4. **A legenda no canvas cobre 2 cenas** (`=82`, `=83`). Cena nova = uma `captions()` pura + uma
   linha no `publish`. A lista está em `motion_demo_legend_tests.rs::scenes()`.
5. ⏳ **A *dirt texture* do `fx.glow`** — a única célula da folha 11 que fica, e a estimativa dela
   foi **corrigida por medição**: ela não é «um asset». Uma máscara de sujidade é um overlay de
   TELA no passe do halo e precisa de uma textura que o composite consiga LIGAR — e a textura de
   uma sprite é uma de **três** coisas (`Atlas{key}` · `Individual{texture_id}` · `CookedTexture`,
   ver `sprite_appearance`), das quais só a primeira é um rectângulo no atlas partilhado. Cobrir só
   essa daria uma feature que funciona com umas imagens e falha em silêncio com outras.
6. ⚠️ **Sinal pré-existente:** `conferencia_vs_manifesto.py` sai vermelho na metade *"já existe no
   manifesto"* com 4 células. Lidas uma a uma, são **falsos positivos**: a ferramenta casa o nome
   do param mencionado na CURA proposta (*"um 9º `ease_curve = Custom`"*), não um param que já
   fecharia a célula. A metade das CONTAGENS está verde (127 nós).

---

## §8 — O smoke, para o Enio

Ele já smokou a `=83`. O que este bloco lhe dá para conferir é **o que estava trancado**:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=13 cargo run -p ph2d-host-desktop --release
```

1. Clique no nó **Spherize** (a lente).
2. No painel, ache **Radius**. Ele diz **320**.
3. Arraste o slider: ele salta para **20 ou menos**, e daí não volta.
4. Escreva `320` na caixa e dê Enter. **Antes deste bloco a caixa recusava** (parava em 20); agora
   ela aceita, e a lente volta ao que a cena tinha.

**Deu errado se** a caixa recusar o 320, ou se aceitar e a lente não voltar ao que era.

E a legenda nova:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=83 cargo run -p ph2d-host-desktop --release
```

Cada figura tem agora **uma ficha em cima dela** dizendo o que é. **Deu errado se** as fichas não
aparecerem, ou se aparecerem sobre a figura errada.


---

## §9 — O que a folha 11 custou, em erros meus

1. ⚠️⚠️ **Escrevi por cima de um arquivo que já existia.** A cena nova foi para
   `motion_state_conferencia_demos_fx.rs`, que **é** a cena `=70` (a família `fx.*`, 140 linhas de
   gates). O `Write` respondeu *«updated»* e não *«created»*, e eu li a resposta como sucesso sem
   reparar no verbo — só o compilador acusou, três passos depois. Restaurado do git antes de
   continuar; a cena nova chama-se `_fx_modes`. ⚠️ **O gatilho é estrutural:** um nome BOM para uma
   cena de FX é exactamente o nome que a cena de FX antiga já escolheu, pela mesma boa razão.
   Memória: `feedback_write_on_an_existing_path_says_updated_not_created`.
2. **A régua da rampa corrigiu-se DUAS vezes** (§0-bis): a representação (uma grelha uniforme não
   representa a esquina de uma parada) e depois o critério (num degrau o que encolhe com a
   densidade é a LARGURA da banda, não a altura do erro). Memória:
   `feedback_a_uniform_grid_cannot_represent_a_corner`.
3. **Dois nós ganharam um argumento a mais** e nenhum dos 21 gates antigos o herdou por default: o
   caso neutro tem NOME (`sink_blend`, `Lens::CENTRED`), a lei do `unlimited` do `sim.step`.

# Handoff de integração — `line/Vector`: a LEI DE MISTURA por degrau (plano 24 W6a)

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `7ec917506`
**Commits desta wave:** 4 (`09cf3e7fe` · `e9a63bed0` · `2f94b0705` · a cena + os splits de LOC)
**Estado:** fechada, **pendente de smoke**

Um degrau da pilha de FX raster passa a dizer *como a cor dele encosta na que já está ali*.
Multiplicador, não um décimo tipo: as **vinte leis** vezes os degraus que já existem.

---

## O que o artista ganha

| par | o que muda |
|---|---|
| Inner Shadow em `Multiply` | a borda **escurece** em vez de **lavar** para a cor do efeito |
| Inner Glow em `Screen` | acende em vez de repintar |
| Bevel em `Overlay` | lê como **material** |
| Color Overlay em `Color` | troca a **matiz** preservando a luminosidade |

⚠️ **O último fecha um item que a fila do plano 24 listava À PARTE** (color-matrix / tint /
duotone): ele sai daqui sem um tipo novo e sem uma linha de kernel nova.

---

## A cura, e as portas

| # | onde | o quê |
|---|---|---|
| 1 | `ph2d-render/src/shaders/blend_modes.wgsl` (**novo**) | as 22 leis, extraídas do `layer_composite.wgsl` quando ganharam o 2º consumidor |
| 2 | `ph2d-ecs::FxOp` | `blend: u8` (código cru) + `takes_blend` na `FxKindSpec` + `blend_code()` |
| 3 | `ph2d-render::fx_stack_shader` | `fx_blend`, chamado do `inner_tint` (os 3 de dentro) e do Color Overlay |
| 4 | `ph2d-panel-vector` | a fileira **Blend** no card + o popover de 20 opções |

**As quatro perguntas, uma porta cada:**

| pergunta | porta | consumidores |
|---|---|---|
| este tipo tem lei a honrar? | `FxOp::takes_blend` | o painel (OFERECER) · o produtor (HONRAR) |
| que código vai ao dispositivo? | `FxOp::blend_code` | `fx_live::resolve_ops` |
| como duas cores se combinam? | `blend_modes.wgsl` | o compositor de camadas · a pilha de FX |
| que WGSL a pilha compila? | `fx_stack::module_sources` | o pipeline · o gate de naga |

---

## MEDIDO antes de decidido (§0)

**Quem toma a lei, e por que os outros não.** A lei pesa pelo alfa do FUNDO (a fórmula do W3C,
`Cs' = (1−ab)·Cs + ab·B(Cb,Cs)`), e um halo **EXTERNO** entra POR BAIXO:

| onde | `ab` | a lei alcança | o halo aparece |
|---|---|---|---|
| fora da forma | 0 | nada | sim |
| rampa de AA | ~0,5 | **0,25** (o pico) | metade |
| dentro | 1 | tudo | **nada** |

Um controle cujo efeito inteiro é uma orla de 1 px lê como quebrado. **Quem TINGE alcança 1,0 no
MIOLO** — quatro vezes mais. Daí a lista de quatro (Inner Shadow · Inner Glow · Bevel · Color
Overlay); Blur e Feather não têm cor própria.

Gate: `the_blend_of_an_outer_halo_only_reaches_the_antialiased_fringe`.

**VINTE leis, e o `BlendMode` tem 22.** `Behind`/`Clear` são operações de COBERTURA, e um degrau
aplica a lei dele onde a cobertura já está decidida pela lei DELE. Oferecê-las e dobrá-las em
Normal no dispositivo seria a opção que despacha e mente.

---

## O que a medição corrigiu em mim — quatro vezes

**1. Uma mutação não sangrou e as leis HSL não tinham cobertura nenhuma.** Neutralizar `is_hsl`
deixava os três gates verdes (eles usam Multiply/Screen, que são separáveis) — e `Color` é a lei
que justifica a wave inteira. Gate novo.

**2. Outra não sangrou porque a FIXTURE não continha o fenômeno.** Jogar fora o peso do fundo
(`mix(colour, b, ab)` → `b`) passava em tudo, porque todos os gates mediam o MIOLO, onde `ab = 1` e
`mix(x, y, 1)` **é** `y`. Um quadrado de borda dura não tem cobertura parcial. Agora há uma RAMPA
(252 → 204 → 128 ao longo dela).

**3. O oráculo do HSL nasceu VERMELHO sobre produto CORRETO.** Ele media Rec.709
(`0,299/0,587/0,114`) sobre os BYTES sRGB, e a lei preserva `0,3/0,59/0,11` sobre LINEAR. Dois
pesos, dois espaços — preservar um não preserva o outro, e a conclusão fácil teria sido *"a lei HSL
não chegou ao dispositivo"*. Medido certo: base **0,2159** → `Color` **0,2163**.

**4. A cena de smoke afirmava uma diferença que ninguém veria — duas vezes.** O par do Inner Shadow
media IDÊNTICO (128,7 × 128,6) porque eu o media no MIOLO, e uma sombra interna vive na BORDA; e na
borda continuava a 2,3 níveis porque o roxo era ESCURO (uma cor já escura multiplicada dá quase o
mesmo que uma interpolação até ela). Com uma lavanda CLARA o par separa e mostra exactamente o
defeito que o default do Photoshop existe para evitar: Normal leva a borda a **166,0**, mais clara
que a base (**148,2**) — *uma sombra que ilumina*; Multiply leva a **130,0**.

### E um gate que eu ia shipar e que NÃO PODIA falhar

`the_neutral_law_is_byte_identical_...` comparava `blend = 0` com `blend = 0`. Deletado em vez de
contrabandeado — a byte-identidade do early-out é provável por ARITMÉTICA, e é onde ela ficou
(`the_normal_early_out_is_load_bearing_because_mix_is_not_the_identity`: `mix(x,x,a)` é
`x·(1−a) + x·a`, que em `f32` **não é `x`**).

---

## Gates

**23 novos.** 3 no `ph2d-ecs` · 6 no `ph2d-render` (CPU, `fx_stack_tests.rs` **novo**) · 4 GPU
(`fx_stack_blend_gpu.rs` **novo**) + 1 sonda · 2 no `ph2d-panel-vector` (+ a presença-por-tipo na
varredura da tabela) · 2 no shell.

⚠️ **A pilha de FX não tinha gate de naga** (o compositor de camadas tem dois). Um erro de WGSL nela
só aparecia quando um pipeline REAL era construído — isto é, na máquina do smoke. Esta wave, que
concatena um bloco NOVO nos dois módulos, é exactamente a classe que esse vão deixava passar.
`fx_stack_tests.rs` fecha-o, e o `module_sources()` é porta única para o gate não validar a própria
concatenação.

**9 mutações, 9 sangram:**

| mutação | sangra |
|---|---|
| `blend = ""` (o bloco compartilhado não é prefixado) | 2 naga |
| `blend_sep → cs` (a lei ignorada) | 2 GPU |
| `is_hsl → false` | 2 GPU |
| `mix(colour, b, ab) → b` (o peso do fundo) | 3 GPU |
| Color Overlay ignora a lei | 3 GPU |
| Color Overlay deixa de a tomar | 3 ecs |
| produtor manda `blend` cru | 3 shell |
| opções fora do `is_filter_button` | 3 seam |
| o chip não é pintado | 5 seam |
| `BLEND_KINDS = 22` | 3 render |

---

## Deltas que a integração precisa conferir

⚠️ **`PROJECT_SCHEMA` 37→38** — o `FxOp` ganhou `blend` APENDADO; postcard é posicional, e
`serde(default)` não salva (o formato não tem NOMES de campo). Tripla-pin **`(38, 12, 13)`**:
`FLIP_SCHEMA_VERSION` e `VEC_SCENE_SCHEMA_VERSION` **intactos** — a lei é do componente ECS, e a
geometria do caminho não mudou uma vírgula.

⚠️ **O número se CONTA, não se escolhe.** A `line/physics` e a `line/FLIP` já colidiram **duas
vezes** nesta escada (o 30 de 25/07, o 32 de 27/07). Se outra linha reivindicar o 38 nesta janela,
o valor certo é o do `main` do dia + 1, e não está em nenhum dos dois lados
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

**Intactos, conferidos por grep:** contratos congelados (§6 — `NodeOp`/`OpResolver`/`NodeManifest`,
`Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`, a superfície do vector-doc) ·
registro do `ph2d-ecs` · tokens · i18n · nenhum `Cargo.toml` tocado (nenhuma dep nova).

**Superfície pública ADITIVA:**
- `ph2d_ecs::FxOp::{blend, BLEND_NORMAL, BLEND_KINDS, takes_blend, blend_code}` +
  `FxKindSpec::takes_blend`;
- `ph2d_render::FxOpGpu::blend`;
- `ph2d_editor_core::ids::{MAX_FILTER_BLENDS, filter_blend_id, filter_blend_option_id}`;
- `ph2d_panel_vector::set_filter_blend_names` + `FilterRowView::blend` + `FilterKindView::takes_blend`.

⚠️ **Um arquivo do compositor de camadas foi CORTADO** (`layer_composite.wgsl` perdeu as linhas
325-483 e o `F32_EPSILON`, que foram para `blend_modes.wgsl`). Se a `line/Painter` tocar a
aritmética de blend na mesma janela, é aqui que o merge conflita — e o resíduo é textual, não
semântico: as funções são as mesmas, noutro arquivo.

---

## LOC — dois caps, os dois por RESPONSABILIDADE

| arquivo | antes | depois | o corte |
|---|---|---|---|
| `ph2d-ecs/src/vec_filter.rs` | 827 | 521 | `mod tests` → irmão por `#[path]` (segue FILHO ⇒ `use super::*` alcança os privados) |
| `shells/desktop/src/build_smoke.rs` | 603 | 515 | a cadeia de DELEGAÇÃO → `build_smoke_router.rs` |

⚠️ O segundo não é hack de tamanho: o `build_smoke.rs` ROTEAVA níveis para outros módulos **e**
HOSPEDAVA as cenas que ainda vivem nele — dois assuntos. A **ORDEM** (específicos antes dos
genéricos) foi preservada e está escrita no doc do módulo novo, porque é ela que faz um nível
roteado nunca chegar ao `match f` genérico.

---

## Arquivos

```
crates/ph2d-render/src/shaders/blend_modes.wgsl          NOVO (as 22 leis, extraídas)
crates/ph2d-render/src/shaders/layer_composite.wgsl      − o bloco de blend
crates/ph2d-render/src/layer_compositor/{mod,tests}.rs   composite_source() + os 16 gates
crates/ph2d-render/src/layer_compositor/compositor/mod.rs o pipeline usa a porta
crates/ph2d-render/src/fx_stack{,_shader}.rs             module_sources + fx_blend + Globals.blend
crates/ph2d-render/src/fx_stack_tests.rs                 NOVO (6 gates CPU, incl. naga)
crates/ph2d-render/tests/fx_stack_blend_gpu.rs           NOVO (4 gates + a sonda da cena)
crates/ph2d-render/tests/fx_stack_*.rs                   + `blend: 0` nos literais
crates/ph2d-ecs/src/vec_filter{,_tests}.rs               o modelo + o split de LOC
crates/ph2d-editor-core/src/ids/chrome/vector_filters.rs os ids do chip e das opções
crates/ph2d-panel-vector/src/{paint,populate,event,state}_filters.rs  a fileira Blend
crates/ph2d-panel-vector/src/{paint,state,ids,lib,paint_sections}.rs  o popover diferido
crates/ph2d-panel-vector/tests/seam_filters.rs           + 2 gates, fixtures atualizadas
shells/desktop/src/fx_live{,_tests}.rs                   blend_code() + 2 gates
shells/desktop/src/render_loop/mod.rs                    o edit + o publish dos nomes
shells/desktop/src/fx_blend_smoke.rs                     NOVO (a cena =34)
shells/desktop/src/build_smoke{,_router}.rs              a rota + o split de LOC
shells/desktop/src/{main,project,project_tests}.rs       mod + PROJECT_SCHEMA 38
docs/Vector Module/24_plano_fx_raster.md                 §12 (W6a) + a lista de waves
```

---

## Smoke

**`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_BUILD_SMOKE=34 cargo run -p ph2d-host-desktop --release`**

Quatro PARES — em cada um a mesma cor e a mesma opacidade, e só a LEI muda. A cena imprime os
números medidos.

**O que olhar:**
1. **Par 1** (overlay ciano): `Normal` chapa a estrela; `Multiply` deixa o relevo do bevel
   ATRAVESSAR.
2. **Par 2**: `Color` troca a matiz e mantém a luminosidade — é o tint/duotone.
3. **Par 3** (inner shadow lavanda): `Normal` deixa a borda mais CLARA que a base (uma sombra que
   ilumina); `Multiply` escurece.
4. **Par 4**: branco em `Overlay` PUXA o contraste do relevo em vez de o apagar.
5. **No painel** — selecione uma estrela, abra **FILTERS**: o card tem a fileira **Blend** logo
   abaixo de Color, e o chip abre a lista de vinte. ⚠️ Ela aparece **só** em Inner Shadow, Inner
   Glow, Bevel e Color Overlay.

E os gates GPU (rodam na RTX):
```
cargo test -p ph2d-render --release --test fx_stack_blend_gpu -- --ignored --nocapture
```

---

## Verde

- `cargo test` em `ph2d-ecs` · `ph2d-panel-vector` · `ph2d-render` · `ph2d-host-desktop` — ok
- `cargo clippy --all-targets` nas quatro — **zero** warnings
- `architecture_workspace_file_loc_cap` + o `file_loc_caps` da shell — ok
- os 4 gates GPU do blend, na RTX — ok

⚠️ **Não rodado:** o `ship.sh` completo e os demais gates GPU `#[ignore]` do `ph2d-render` (nada
nesta wave toca o compositor de camadas a não ser por MOVER o bloco de blend, e os 16 gates dele —
incluindo os dois de naga — passam).

---

## Aberto

- **A lei de um halo EXTERNO contra a CENA** (o Drop Shadow em Multiply do Photoshop) exigiria que
  a textura de saída do FX carregasse uma lei para o composite da cena — outra camada, outro dono.
  Nomeado, não construído.
- **O Bevel tem UMA lei para as duas faces.** O Photoshop tem duas (Highlight: Screen · Shadow:
  Multiply). Uma só já é coerente; o par é refino de produto.
- **W6b** (o resto da wave): turbulência + deslocamento — o eixo ORGÂNICO, e o que ainda pede
  maquinaria nova.

# Handoff de integração — `line/Vector`: a TURBULÊNCIA (plano 24 W6b)

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `78d770370`
**Commits desta wave:** 4 (`29edb4380` · `15c3bdd52` · `489be1ba7` · `0d0d319a8`)
**Estado:** fechada, **pendente de smoke**

⚠️ **A linha tem uma TERCEIRA wave** — o Grow / Shrink (W7), handoff
[`HANDOFF_INTEGRACAO_line_Vector_morphology_2026-07-28.md`](HANDOFF_INTEGRACAO_line_Vector_morphology_2026-07-28.md).
As três partilham o bump de schema; integre as três juntas.

⚠️ **Esta é a SEGUNDA wave da mesma abertura de linha.** A primeira (a **lei de mistura**, W6a) tem
handoff próprio — [`HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md`](HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md)
— e as duas **compartilham o bump de schema** (ver a seção *Schema*). Integre as duas juntas: o
`main` nunca viu nenhuma delas.

O eixo **ORGÂNICO** que a fila do plano 24 nomeava como *"o que ainda pede maquinaria NOVA"*: a
imagem de um degrau é deformada por um campo de ruído procedural.

---

## O que o artista ganha

Um degrau **Turbulence** com **Amount · Size · Detail · Seed** e dois modos (**Smooth** / **Creased**):

| Amount | o que se vê |
|---|---|
| pequeno (~0,05) | a borda fica **rasgada / desenhada à mão** — o *Roughen* |
| médio | ondulação orgânica; com um contorno na pilha, a LINHA serpenteia |
| grande (~0,25) | a forma **liquefaz** |

E ela **compõe**: `Outline → Turbulence` ondula o contorno; `Turbulence → Glow` acende a forma já
deformada. Nenhum tipo existente ganhou parâmetro, nenhum kernel existente mudou.

---

## A pesquisa decidiu a FORMA, não só a matemática

O SVG separa `feTurbulence` (gera) de `feDisplacementMap` (deforma). **Todo mundo depois fundiu os
dois** — o AE tem *Turbulent Displace* com o ruído dentro; o Photoshop tem o *Displace* com mapa em
ARQUIVO, que é justamente a interface que ninguém usa. E aqui a fusão é **obrigatória**: a pilha é
uma LISTA em que *todo op é imagem → imagem*, então um degrau que só GERASSE ruído teria de escrever
por cima da imagem que o seguinte espera receber.

Detalhe e tabela comparativa: [`docs/Vector Module/24_plano_fx_raster.md`](Vector%20Module/24_plano_fx_raster.md) §13.

---

## As portas únicas

| pergunta | porta | consumidores |
|---|---|---|
| este tipo lê um campo de ruído? | `FxKindSpec::noise_labels` | o painel (OFERECER) · o produtor (HONRAR) |
| quantas oitavas ele soma? | `FxOp::detail_clamped` | o painel (mostra) · `resolve_ops` (manda) |
| como este degrau é executado? | `plan_of` → `Plan::Warp` | os globals · o dispatch |
| quanto ele espalha? | `op_reach` | `stack_reach` → o tamanho do scratch |
| onde a grade do ruído está ancorada? | `stack_reach(ops).0/.1` | o `run` (a MESMA que dimensiona a textura) |

### ⚠️ A colisão de modos que esta wave expôs (e que já estava armada)

O `mode` é um índice na lista **DO TIPO**, então o mesmo `1` é `MODE_CONTOUR` num degrau de dentro e
`MODE_CREASED` na turbulência. O `plan_of` roteava por *"tem modos, e escolheu o 1?"* — uma
turbulência **Creased** cairia no **campo de distância** e desenharia outra coisa inteira, sem erro
em lugar nenhum. Agora o **tipo decide antes do modo**, com gate nos dois modos
(`the_turbulence_warps_in_both_of_its_modes`).

### ⚠️ A ancoragem, que é o ponto não-óbvio do desenho

A coordenada do ruído é `(pixel − org)/escala_px`, com `org` = a margem que a pilha reservou. Sem
esse termo a grade fica presa ao canto do **scratch** — e a margem é função de TODA a pilha, então
mexer no raio de um Glow faria o padrão da turbulência **andar** por baixo da forma, um efeito
colateral entre degraus que ninguém consegue atribuir. Com ele a coordenada é
`(mundo − caixa_da_forma)/escala_mundo`: **invariante ao zoom e imune aos outros degraus**.

Gate: `the_noise_is_pinned_to_the_shape_not_to_the_scratch` — desvio medido **0,0000 px**. O
instrumento é um **Glow inerte** (`tint.a = 0`, `opacity = 0`), que muda a margem sem mudar um pixel.

---

## MEDIDO antes de decidido (§0)

**`MAX_DETAIL = 6` não é teto de custo** — isso foi medido e é falso: 1 a 12 oitavas movem o passe
de **0,058 para 0,12 ms** a 512², que é a própria dispersão da medição. É teto de **REPRESENTAÇÃO**:

| oitava | a borda anda |
|---|---|
| 4 → 5 | 0,072 px |
| 5 → 6 | 0,044 px |
| **6 → 7** | **0,019 px** |
| 9 → 10 | 0,002 px |

**`op_reach` = o próprio Amount** (`+1` pelo vizinho bilinear), não `3σ`: o campo vive em `[-1,1]`,
então nenhum texel viaja mais que isso, e `3σ` seria margem paga por um borrão que não existe.

Sondas: `measure_the_turbulence_octave_cost` · `measure_what_an_extra_octave_still_moves` ·
`measure_the_smoke_scene_pairs` (todas em `crates/ph2d-render/tests/fx_stack_turbulence_gpu.rs`).

---

## Gates — 13 novos, 10 mutações, **10 sangram**

| gate | onde |
|---|---|
| `a_zero_amount_is_byte_identical_to_no_turbulence_at_all` | GPU |
| `the_turbulence_moves_the_edge_by_about_the_amount_it_was_given` | GPU |
| `the_size_is_the_size_of_the_ripples` | GPU |
| `the_detail_adds_fine_structure_without_changing_the_scale` | GPU |
| `another_seed_is_another_drawing_of_the_same_kind` | GPU |
| `the_creased_mode_breaks_the_slope_where_the_smooth_one_rolls` | GPU |
| `the_warp_reads_between_texels_instead_of_snapping_to_one` | GPU |
| `the_two_axes_are_independent_fields_not_one_field_used_twice` | GPU |
| `the_noise_is_pinned_to_the_shape_not_to_the_scratch` | GPU |
| `the_turbulence_warps_in_both_of_its_modes` | CPU (`ph2d-render`) |
| `the_turbulence_reach_is_the_amount_it_displaces` | CPU (`ph2d-render`) |
| `the_three_noise_knobs_reach_the_bus_when_dragged` + as rows em `a_row_paints_only_the_controls_its_kind_uses` | seam (painel) |
| `the_noise_knobs_reach_the_pass_and_the_detail_arrives_clamped` · `hit_of_decodes_the_three_noise_knobs` · o teto `FILTER_DETAIL_MAX == FxOp::MAX_DETAIL` | shell |

**Rodar os de GPU:** `cargo test -p ph2d-render --test fx_stack_turbulence_gpu --release -- --ignored`
(sem adapter eles fazem *skip gracioso*, que **não é verde**).

### ⚠️ Duas coisas que a medição corrigiu em mim

1. **A fixture continha OUTRA coisa.** As linhas do topo e da base amostram FORA do scratch (o `dy`
   puxa de onde não há nada) e os cruzamentos espúrios delas inflavam a amplitude **3×** — a curva
   ia de `[17,75; 65,28]` com o miolo INTEIRO em 63,3. **Um** defeito de fixture reprovava **três**
   gates, e sem olhar a curva eu teria "consertado" três coisas certas.
2. **A semente POR OITAVA é load-bearing, e o mecanismo que eu escrevera era outro.** Eu dizia que
   ela impede os zeros de se alinharem — falso (Perlin vale zero em todo nó da própria grade, em
   qualquer semente). O que ela impede é as oitavas lerem a MESMA tabela de gradientes em células
   relacionadas: sem ela a rugosidade do **Smooth** sobe de **0,419 para 0,609** e **encosta na do
   Creased** (0,602) — o modo liso deixa de ser liso, e os dois modos desenham a mesma coisa.

### ⚠️ E dois defeitos nos meus próprios gates

- O gate de arrasto **nasceu VERMELHO sobre um slider vivo**: o `ValueChanged` sai no **DOWN**, e eu
  aplicava só os eventos do Up.
- O gate de paridade do `Globals` comparava o WGSL contra uma **lista literal escrita à mão** — a
  terceira cópia do mesmo fato, que quem acrescenta um campo atualiza no mesmo commit em que
  introduz a divergência. Agora ele lê o struct do Rust no fonte; e o **controle positivo** dele
  pegou o próprio parser quando o `Globals` mudou de arquivo (a visibilidade entrou no nome).

---

## Schema, contratos, ids

- **`PROJECT_SCHEMA` fica em 38** — o mesmo bump da W6a. O `FxOp` ganhou `scale`/`detail`/`seed` na
  mesma janela em que ganhou `blend`, e um save v37 **já é recusado** pelo 38: pôr a segunda leva num
  39 jogaria fora exatamente os mesmos arquivos e custaria mais um degrau para ninguém. **Uma linha,
  um bump.** ⚠️ O valor se **CONTA** a partir do `main` do dia — a `line/physics` e a `line/FLIP` já
  colidiram DUAS vezes nesta escada.
- **Contrato congelado §6: INTACTO** (conferido por grep) — `NodeOp=2` / `OpResolver=1` /
  `NodeManifest=8` / `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4`.
- `VEC_SCENE_SCHEMA_VERSION`: **intocado** (a turbulência é componente ECS, não geometria).
- **`MAX_FILTER_KINDS` 9 → 10** (o gate de tetos da shell pegou: sem isso o "Add Turbulence" não
  seria pintado, porque o `paint` faz `.take()`).
- **Ids novos** (6, todos derivados por linha, bloco append-only): `filter_scale_id{,_num}` ·
  `filter_detail_id{,_num}` · `filter_seed_id{,_num}`.
- **Superfície pública nova:** `ph2d_panel_vector::FILTER_DETAIL_MAX` (para o gate de teto da shell)
  e `ph2d_render::make_output_texture` mudou de módulo (mesmo caminho de import).

---

## LOC — dois splits, os dois por responsabilidade

| arquivo | antes | depois |
|---|---|---|
| `fx_stack_shader.rs` | 800 | **673** + `fx_stack_noise.rs` (149) — *o FOLD* × *o CAMPO* |
| `fx_stack.rs` | 757 | **676** + `fx_stack_res.rs` (102) — *o fold* × *o que o passe ALOCA* |

⚠️ O split órfãou um doc-comment (a cauda do doc do `FxStackPass`), pego pelo clippy e
**reancorado** — um doc que documenta a função errada é pior que doc nenhum.

---

## Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=35 cargo run -p ph2d-host-desktop --release
```

Quatro pares — em cada um a MESMA estrela com o MESMO contorno por baixo, e só um knob difere.
Os números da mensagem saem da sonda, rodada ANTES de a mensagem os afirmar.

**A cena `=34` (a lei de mistura, W6a) continua válida e precisa do mesmo smoke.**

---

## Aberto, nomeado

- **A turbulência não anima.** O `Seed` é um número autorado; um *Evolution* (caminhar pelo espaço
  de ruído ao longo do tempo, o do AE) exigiria uma 3ª dimensão no campo e um vínculo com o
  playhead — decisão de produto, e provavelmente a mesma conversa do morph vivo do ADR-0128.
- **Falta o `feMorphology`** (dilate/erode) da lista de primitivas do SVG — mas o **Outline** já
  cobre a dilatação, e a erosão é o mesmo campo de distância com o sinal trocado: é uma wave curta,
  não maquinaria nova.
- **O `Falloff` (ADR-0132) NÃO modula a turbulência** — ele é da pilha de LPE (geometria), e esta é
  a pilha raster. Um campo espacial que module a força de um degrau RASTER é uma ideia boa e uma
  wave própria.

# HANDOFF — `line/Painter`: **o modelo de rotação** (Blender × nosso) — 2026-07-19

> Continuação da mesma linha (Rake lag · Random Angle · Shape FLOW). **Pendente de smoke do Enio.**
> A linha NÃO integra nem pusha sozinha (§0.7 / §0.2).

## 0. Estado

| | |
|---|---|
| Branch | `line/Painter`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| Ahead of `origin/main` | 8 commits |
| Árvore | limpa · `cargo check --workspace --all-targets` 0 · clippy 0 · **`cargo test --workspace` VERDE** · LOC cap verde |

## 1. O pedido

> *"Percebi uma diferença entre a implementação original do blender de Angle e talvez por isso rake também
> fica estranho. Uma coisa que o blender não tinha e nós temos é Flatten & Rotate. Talvez tenhamos conflitos
> nas múltiplas referências a rotate. Estude a implementação original Blender e nossa implementação.
> Descubra inconsistências e relate. Descubra melhoramentos e relate."* (Enio)

Depois do relatório: **D1 sim · D2 não · GO** (itens 1-6).

## 2. O que o Blender faz (fonte real, `main` + `2.93`)

- **UMA** rotação: `brush_rotation`, global, por evento de mouse.
- Ela gira **só a coordenada de lookup da textura** — nunca o carimbo.
- O footprint é `f(distância ao centro)` via LUT 1D: **radialmente simétrico por construção**. Não existe
  dab elíptico nem ângulo de dab em lugar nenhum do texture paint.
- Composição: `rotation = −(mtex->rot + brush_rotation)` — Angle e Rake **somam**; Rake nunca sobrepõe.
- Suavização do Rake: **ZERO**. Deadband de 20 px (4 px antes do traço começar), senão **segura** o ângulo.
- `mtex->size` (não-uniforme) é aplicado **depois** da rotação, dentro do `RE_texture_evaluate`.

## 3. O desenho novo (uma frase)

> **A tangente entra no dab UMA vez, no FRAME — nunca num slot.**

`BrushSpec::dab_rotor(&Dab)` = Jitter Rotate ∘ *follow rotor*; `dab_footprint(rotor)` o aplica ao footprint
que **todo** sampler já lê (falloff + Shape + View-Grain — "the deform is brush-wide", que o painel já
prometia). `dab_basis` passou a carregar **só o Angle do slot**.

É a forma do Blender (um `brush_rotation`, aplicado uma vez), adaptada ao fato de que **nós temos um
footprint elíptico e ele não**: como o nosso carimbo tem orientação própria, o rotor pousa no **frame**
em vez da coordenada de lookup.

**Por que isso é o conserto e não um refactor:** a esticada do flatten mora **ENTRE** as duas rotações
(`D · R(θ_dab)` e depois `R(θ_slot)`), então elas **não comutam** — um bico caligráfico raked saía
*cisalhado de forma diferente em cada ponto da curva*. Com uma rotação só o problema não existe.

## 4. Os seis itens

| # | item | o que mudou |
|---|---|---|
| **1** | Flow re-fasava com a pressão | o eixo *along* agora divide por um comprimento **constante no traço** (`Dab::stroke_radius_px`), não pelo raio vivo. Medido antes: **0,42 unidade de tile** (≈21 % de uma listra) saltando entre dabs vizinhos com o `size_pressure: true` que shipa. Flow funcionava no mouse e quebrava na caneta |
| **2** | `arc_len` chegava em 2 de 7 rotas | `with_arc_len` (builder opcional) **morreu**; nasceu a porta `texture::shape_basis(..., ShapeFrame)`. `ShapeFrame` **não tem `Default`** — impasto/sculpt/smear/watercolor/blur/clone agora recebem o frame real. Arch-gate `the_shape_slot_goes_through_the_shape_door` (com **controle positivo**) impede a próxima rota de usar a porta do Grain |
| **3** | Rake "funcionava antes" | **a causa raiz nunca era o estimador, era o CHAMADOR.** `walk_space` alimentava `heading::advance` com o *resto dentro da corda de 3 px* enquanto o spacing pode ser dezenas de px — e cordas que não emitiam dab **não moviam o heading**. O EMA rodava com passo até 16× pequeno demais (comprimento de suavização efetivo ~240 px em vez de 12). Agora recebe o percurso real, e o `from_centers` (o secante de um spacing, 3-5× mais ruidoso) foi **removido** |
| **4** | Flow ignorava o footprint | o ramo de Flow passa pelo `footprint.apply` como todo mundo — o falloff e o Shape voltaram a concordar sobre a forma do dab |
| **5** | **D1** | o bico achatado **gira com o traço** |
| **6** | higiene | sinal do `tiled.rs` (Paper girava ao contrário de todo o resto) · **Size depois da rotação** (ordem do Blender: antes, Size não-uniforme + Angle **cisalhava**) · `extra_rot` morto removido (`[1,0]` nos 21 sítios) · fixture de `arc_len` ganhou uma perna que **volta** |

**D2 (adotar os +90° do Blender) — NÃO**, por ordem do Enio. Nossa convenção põe a tangente no eixo **X**
do padrão; a do Blender põe no **Y** (*"motion direction points down the brush's Y axis"*). Divergência
deliberada, agora **documentada** no `texture.rs` em vez de silenciosa.

## 5. Gates (red-first + mutação)

| Gate | Crate | Mutação que sangra |
|---|---|---|
| `the_heading_is_a_fact_of_the_path_not_of_the_dab_spacing` | brush | chamador antigo ⇒ lag **3,7° → 52,1°** e o spread por spacing 0,1° → 19,4° |
| `flow_gives_adjacent_dabs_a_continuous_phase` | brush | unidade = raio vivo ⇒ diff **1,0** (máximo possível) |
| `the_stroke_tangent_enters_the_dab_exactly_once` | brush | `follows_stroke = false` ⇒ RED |
| `a_following_tip_turns_the_flattened_footprint_with_the_stroke` | brush | idem ⇒ RED |
| `rake_turns_the_sampled_pattern_with_the_stroke` | brush | idem ⇒ RED |
| `jitter_rotate_composes_on_top_of_the_follow_rotor` | brush | idem ⇒ RED |
| `the_shape_frame_reaches_the_sampler_and_is_inert_without_flow` | brush | — |
| `a_non_uniform_size_rotates_the_pattern_instead_of_shearing_it` | brush | Size antes da rotação ⇒ pior caso **1,0** |
| `the_two_tiling_samplers_agree_on_which_way_the_angle_turns` | brush | sinal antigo ⇒ RED (**e mais nada** — a suíte inteira era cega) |
| `a_following_shape_paints_the_stroke_not_the_canvas` | tool | `follows_stroke = false` ⇒ resíduo **0,0 → 98,4** |
| `the_shape_slot_goes_through_the_shape_door` + controle positivo | editor-core | o controle **pegou o scanner quebrado na hora** (`before.contains("fn ")` casava com o `fn` da função envolvente ⇒ ele reportava zero) |

**O oráculo do e2e** substituiu dois `assert_ne!`. Girar o traço 90° em torno do centro do canvas mapeia
centro-de-pixel em centro-de-pixel **exatamente**, então: *uma Shape que segue o traço tem de pintar a
rotação da pintura*. Medido: **0,0 seguindo · 42,2 estática**. Fosso enorme, zero detecção de feature.
Os `assert_ne!` que ele substitui eram satisfeitos por **0,55 %** dos texels.

## 6. Notas de integração (DIRETRIZ §1.5.9)

- **`Dab` ganhou `stroke_radius_px: f32`** (além do `arc_len` do commit anterior). Literais de `Dab` em
  linha paralela conflitam textualmente; resolver adicionando o campo.
- **`texture::dab_basis` mudou de assinatura** (6 → 4 args: saíram `dab_dir` e `extra_rot`) e nasceu
  `texture::shape_basis` (+ `ShapeFrame`). ~29 sítios de produto + ~22 de teste foram reescritos.
- **`BrushSpec` ganhou** `follows_stroke` / `follow_rotor` / `dab_rotor` / `dab_footprint` (em
  `spec_frame.rs`). `heading::from_centers` e `Stroke::last_emit_pos` **removidos**.
- **Splits por LOC cap:** `texture.rs` 730 → 551 (`texture/kind.rs`) · `spec.rs` 717 → 675 (`spec_frame.rs`).
- **Nenhum contrato congelado tocado**; nenhum `PROJECT_SCHEMA`/`DOC_VERSION` bumpou (`BrushSpec` não é serde).

## 7. Mudanças de comportamento que o smoke vai ver

1. **Grain Rake agora gira o dab inteiro** (não só o lookup do Grain) — coerente, e é o que "brush-wide"
   sempre significou. Com `flatten = 0` (o default) o falloff é invariante, então na prática muda o Shape.
2. **Paper Angle gira para o outro lado** (era o único invertido do motor).
3. **Size não-uniforme + Angle** agora rotaciona um padrão esticado em vez de cisalhar. Em Angle 0 e em
   Size `[1,1]` é **bit-idêntico**.
4. **Rake** deve seguir a curva sem atraso **e** sem tremer (as duas metades, medidas).

## 7b. O GIZMO DO PINCEL GIRA (Enio, pós-smoke 2026-07-19)

> *"Permite que o círculo que representa o pincel (gizmo do pincel) rotacione em tempo real conforme flow e rake."*

O anel do cursor já desenhava a elipse de **Flatten & Rotate**, mas travada no `dab_angle_deg` de repouso
— então com Rake/Flow o bico girava e o desenho dele **ficava parado**. Aim de nib caligráfico com um
cursor que aponta para outro lugar é pior do que cursor nenhum.

- O tool publica **`BrushSettings::dab_rotor`** = `Angle ∘ follow_rotor(heading VIVO do motor)`.
- ⚠️ **O anel não deriva direção nenhuma** — ele lê o rotor que sai do **heading do próprio motor**
  (`Stroke::heading()`, novo acessor). Uma segunda estimativa (por ex. filtrar o cursor no shell) iria
  divergir da tinta, e quem o artista vê é o cursor.
- ⚠️ **Jitter Rotate fica FORA de propósito**: é aleatoriedade por-dab; um anel piscando reportaria ruído
  como mira.
- Sem traço em voo o heading é `[0,0]` ⇒ `follow_rotor` é a identidade ⇒ o anel descansa no Angle, que é
  exatamente o que o primeiro dab do próximo traço vai usar. Deform continua disco puro.

Gates: `the_brush_ring_rotor_turns_with_the_stroke_only_when_a_slot_follows` (tool: dirige o traço real
numa curva e exige o rotor na tangente ±8°; e **bit-idêntico ao Angle** quando nada segue) + o arch-gate
**`the_brush_ring_wears_the_live_dab_rotor`** (shell). ⚠️ O segundo existe porque o anel é **puro desenho
no shell**: mutar o shell para reconstruir o ângulo do `dab_angle_deg` deixa **toda a workspace verde** —
o gate de comportamento para no valor publicado.

## 8. Smoke pedido

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && cargo run --release -p ph2d-host-desktop`

1. Shape → textura direcional (Stripes/Dots ou imagem de bico) → **Follow = Flow**; pinte uma **curva** com
   pincel grande: linhas contínuas e paralelas seguindo a curva. Repita com **caneta** (pressão variando) —
   era aqui que quebrava.
2. **Follow = Rake** na mesma curva: deve girar acompanhando o traço, sem tremor entre carimbos.
3. **Flatten & Rotate** com flatten alto + Follow ligado: o bico achatado tem de **virar com o traço**.
4. Grain e Paper continuam funcionando; combinações Shape×Grain×Paper intactas.
5. **O anel do cursor gira junto** com Rake/Flow (mais visível com Flatten alto, onde a elipse é óbvia);
   com Follow=Off ele fica parado no Angle.

## 9. ⛔ NÃO integrei nem pushei (protocolo §0.7 / §0.2)

Fechei, escrevi este handoff, **PAREI**.

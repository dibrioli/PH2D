# ADR-0145 — O PH2D tem um POST STACK: uma grade HDR de frame inteiro

> **Status:** aceito (Enio, 2026-07-28) e implementado em fatia 1 (o passe render + a
> vinheta + o smoke). Linha `line/motion-value`. ⚠️ **Número PROVISÓRIO** — ADR escolhido
> numa linha paralela renumera no merge (2ª vez no repo: 0130→0131 na physics); o gate
> `architecture_adr_numbers_are_unique` decide quem fica com o 0145 no `main`.

## Contexto — a cerca que o doc 66/67 deixou, e o que o Enio decidiu

O [doc 66](../../Motion%20Nodes/66_fx_de_passe_a_premissa_do_plano_e_FALSA.md) provou que a
premissa do plano de FX de passe era falsa (o compositor do Painter é 8-bit e **destrói** o HDR
de que o bloom vive) e ofereceu **duas** formas para os efeitos de passe:

- **Opção B** — o Motion desenha num RT HDR **próprio**, o FX roda ali, compõe de volta. Blast
  radius zero fora do Motion. Foi o que o [doc 67](../../Motion%20Nodes/67_fx_de_passe_glow_opcao_B_nota_adr.md)
  entregou: o **glow** (`fx.glow`), um nó no grafo do Motion.
- **Opção A** — **pós-processo do FRAME INTEIRO**: um passe HDR no `game_rt` que faz bright-pass
  bloom, **vignette**, levels, hue. Muda **TUDO** (sprites, Painter, Flip, Vector). O doc 67 §6 a
  nomeou explicitamente e a barrou da linha do Motion: *"vignette / levels / hue **NÃO entram
  aqui** — são grades do frame inteiro (aplicá-las 'só na camada Motion' não é operação real, e
  forçá-las quebraria o z). São a Opção A do doc 66 — um post stack do app, que **merece o ADR
  próprio**."* — e declarou que *"não é uma decisão que eu deva tomar sozinho"* (o blast radius é
  as 6 linhas; muda a cara do produto inteiro).

⚠️ **Por que o glow pôde ser um nó e a vinheta NÃO pode.** O glow é **luz ADITIVA z-agnóstica** —
soma um halo sobre o que estiver na frente, correto em qualquer z; então re-renderizar só as
instâncias do Motion e somar o halo funciona. Uma **vinheta é o oposto**: ancorada na **moldura**
e **subtrativa**. As instâncias do Motion são fundidas e z-ordenadas no mesmo passe do ECS, sem tag
de origem ([`sprite_collect`](../../../crates/ph2d-render/src/sprite_collect.rs)); *"escurecer só a
camada Motion"* não é uma operação real. O mesmo raciocínio que fez o glow ser a escolha certa para
a Opção B faz a vinheta ser a errada — ela é intrinsecamente Opção A.

**O Enio pediu uma vinheta e decidiu abrir a Opção A** (2026-07-28). Este ADR é o "ADR próprio" que
o doc 67 exigiu.

## Decisão

**O PH2D ganha um POST STACK: uma grade de cor HDR aplicada ao `game_rt` inteiro, entre o glow do
Motion e o tonemap.** Cena-referida (a grade roda em luz LINEAR, antes do display transform — o
idioma ACES/OCIO/AgX), então ela precede o tonemap AgX que já existe.

### O passe é AUTOCONTIDO — o tonemap fica intocado, byte-idêntico no neutro

`ph2d_render::PostStack` (irmão de `MotionFx`/`Tonemap`) faz **1 render pass + 1 copy**, sem tocar
o tonemap:

```text
game_rt (Rgba16Float) ──copy──▶ scratch ──[grade fullscreen]──▶ game_rt   (tonemap lê game_rt como sempre)
```

`copy_texture_to_texture(game_rt → scratch)` (o `game_rt` já tem `COPY_SRC`, o scratch nasce com
`COPY_DST`) e então um fullscreen amostra o scratch, aplica a grade em HDR e reescreve o `game_rt`.
O tonemap **não é re-apontado** (rejeitei o `rebind_game_view` a cada frame): o passe é uma
operação *"grade o game_rt no lugar"*, exatamente análoga ao glow ser *"some o halo sobre o
game_rt"*. Blast radius fora do post-stack: **zero**.

⚠️ **A byte-identidade no neutro é garantida pelo SHELL PULAR o passe**, não pela matemática do
shader — exatamente como o glow só roda com `intensity > 0`. `GradeParams::is_neutral()` é `true`
para a grade de fábrica (exposição 0, contraste 1, saturação 1, tint branco, vinheta 0); o
`present.rs` só encoda o passe quando `!is_neutral()`. Uma grade *quase*-neutra (vinheta 0,0001)
**roda** o passe — a paridade CPU×GPU cobre a correção; o skip-no-neutro cobre a identidade.

### A grade da fatia 1 — cinco knobs, em ordem cena-referida

`GradeParams { exposure, contrast, saturation, tint[3], vignette, vignette_radius,
vignette_softness }`, aplicados nesta ordem (a ordem de um color-corrector antes do display
transform):

1. **Exposição** (stops) — `c *= 2^exposure`. Neutro 0. O CPU pré-calcula `2^exposure` (o shader só
   multiplica; zero transcendental no device).
2. **Tint / white balance** — `c *= tint_rgb`. Neutro `[1,1,1]`.
3. **Contraste** em torno do pivô 0,18 (o cinza médio cena-referido) — `c = (c-0,18)·contraste +
   0,18`, clampado ≥ 0. Neutro 1.
4. **Saturação** — `mix(luma, c, sat)`, `luma = Rec.709`. Neutro 1 (a forma `mix` é EXATA em sat=1,
   e é a mesma no CPU e no shader — a paridade é bit-a-bit nesse eixo).
5. **Vinheta** (o pedido do Enio) — escurece a moldura por um falloff radial **elíptico-nos-pixels**
   (a distância corrige pelo aspecto). `factor = 1 − amount·smoothstep(radius, radius+softness, d)`,
   `d ∈ [0,1]` (0 centro, 1 canto). Neutro amount 0 → factor 1. Aplicada por ÚLTIMO (é óptica: um
   falloff de lente sobre a radiância antes do display transform).

**A referência é o `grade_pixel` do CPU** — o WGSL o espelha linha a linha, e o gate `#[ignore]` de
paridade lê de volta o device e compara (a disciplina impasto-light / value-node). Sem hue nesta
fatia (é uma rotação no plano cromático — knob de follow-up).

### Onde a grade É AUTORADA — decidido, construído em fatia 2

O estado autorado de uma grade de **app inteiro** não pode ser um nó do Motion (que é do módulo
Motion) nem um componente ECS (que é por-objeto). O idioma do repo para estado autorado,
persistido e fora do `ProjectState` (para o Ctrl+Z do canvas não rebobinar a cor do frame) é o
**`ProjectSettings`** — exatamente onde as settings de MUNDO da física moram (ADR-0131 W2b), e onde
o `pixels_per_meter` já vive. **Fatia 2:** `ProjectSettings.grade: GradeParams` (persistido →
`PROJECT_SCHEMA` bump) + UI no menu Settings + o passe lendo dali em vez do `App.grade` runtime.

**Fatia 1 (esta):** o passe render + a vinheta funcionando, dirigidos por um `App.grade` runtime
que o smoke arma — o hard-part de GPU (a grade HDR, byte-idêntica-quando-pulada, costurada no
`present.rs`) provado; a UI/persistência é a fatia seguinte, e é honesta como fatia porque o passe
é a fundação reutilizável (a física fez o mesmo: W2a construiu o passe/componente antes de o W2b
construir o painel).

## Consequências

- **Positivo:** o app ganha um post stack de verdade, no slot HDR que já foi construído vazio pra
  isso (`present.rs`, entre o glow e o tonemap). A vinheta que o Enio pediu, no lugar
  arquitetural certo. Neutro = byte-idêntico (o passe é pulado).
- **Custo:** o passe (1 copy + 1 fullscreen no `game_rt`, ~sub-ms a resolução de editor) só se paga
  quando a grade é NÃO-neutra. VRAM: um scratch `Rgba16Float` do tamanho da tela (~16 MB a 1080p),
  alocado no boot ao lado do `game_rt`/`motion_fx` — o preço de o app ter um post stack.
- **Blast radius:** o `present.rs` (compartilhado pelas 6 linhas) ganha um bloco (Pass 1d). O
  tonemap, o compositor, o glow e o passe de sprite ficam intocados. `AppGfx` ganha um campo
  (`post_stack`), `App` ganha um campo (`grade`).
- **Aberto (fatia 2+):** `ProjectSettings.grade` + persistência + UI · hue rotation · vinheta em
  LDR (pós-tonemap) como modo alternativo · bloom de frame inteiro (distinto do glow do Motion —
  seria a peça cara, e coexiste com o glow: um é do frame, o outro é do módulo).

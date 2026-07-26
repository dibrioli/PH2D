# Handoff de integração — `line/Vector`: a PILHA de FX RASTER (Blur / Glow / Drop Shadow)

**Plano:** [`docs/Vector Module/24_plano_fx_raster.md`](Vector%20Module/24_plano_fx_raster.md) ·
**Data:** 2026-07-26 · **W1 + W2** na `line/Vector`.

O FX raster de alta qualidade para formas vetoriais — a resposta ao pedido *"efeitos FX de alta
qualidade, estado da arte, compatível com o que temos"*, e depois ao *"aqui tudo é para o game em
runtime, precisamos de total performance"*. Melhor que o Rive, cujo FX é **estreito** (feather +
blend, com sombra e brilho DERIVADOS do feather, sem pilha nenhuma).

- **W1 — a forma filtrada** (Blur · Outer Glow · Drop Shadow). **SMOKE APROVADO** pelo Enio
  (`=33`, incluindo maximizar + zoom: FPS liso, sem panic).
- **W2 — a PILHA componível** (escolha do Enio). **PENDENTE DE SMOKE.**

## A costura (o inegociável)

Um FX raster produz **PIXELS**, não `VecPath` ⇒ **não é PathEffect** (`effect::run_stack` é
`VecPath->VecPath`, puro, sem GPU, dentro da `ph2d-vec-scene`) **nem `LiveGeometry`**. É uma
`FxImages` que o **shell produz** e o `dispatch` só **encoda** no z da forma. É por isso que a
seção do painel se chama **Filters**, distinta de **Effects** (deformadores vetoriais, ADR-0132).

## ⚠️ ARQUITETURA: 100% RESIDENTE NA GPU

O 1º corte foi CPU-first (render→**readback GPU→CPU**→Gaussiana na CPU→**re-upload**) — padrão de
PREVIEW de editor. Em runtime a forma anima, e esse roundtrip roda por frame por forma: o readback
bloqueia o pipeline e o re-upload **vaza o atlas do Vello** (Blob nova por frame = id novo = upload
que cresce sem fim — medido: recook 37→793 ms num smoke parado). **Reescrito GPU-resident:**

- **`ph2d_render::FxStackPass`** (`fx_stack.rs`) — o fold da pilha na GPU: `2n+1` dispatches
  (Gaussiana separável H + V-finalize-composite por degrau, mais um `resolve` no fim).
  Intermediários em **`Rgba16Float`** (guardar premultiplicado em 8 bits e des-premultiplicar
  depois quantiza justamente a borda macia que o borrão existe para produzir). Globals de todos os
  passes escritos de uma vez e indexados por **offset dinâmico** — senão um `write_buffer` por
  passe antes de um único `submit` deixaria o último a valer para todos.
- **`VelloPass::register_texture`/`unregister_texture`** — uma textura da GPU vira imagem
  desenhável por **id ESTÁVEL**, sem upload de CPU; re-cozinhar escreve NA MESMA textura ⇒ zero
  churn de id no atlas. ⚠️ **`override_image` foi REMOVIDO**: ele troca a textura e **NÃO** atualiza
  `width`/`height` da `ImageData` ⇒ o Vello copia além da textura nova (`Copy 0..167 overruns
  source size 166`) — foi o *"panic ao zoom / deforma ao maximizar"*. **Resize RE-REGISTRA.**
- **`fx_live`** — por forma: um scratch `VelloPass` (render isolado, sem readback) → `FxStackPass`
  → textura de saída persistente registrada no **renderer PRINCIPAL** (o que desenha
  `vector_scene`; registrar noutro faz o Vello entrar em pânico). O memo é OTIMIZAÇÃO, não
  correção. Desregistro enfileirado (`forget` não tem `vello_pass`; o `recook` drena).

Custo por frame: 1 render isolado + `2n+1` passes + 1 cópia GPU→GPU no atlas — tudo na placa.
Medido no smoke: **6 pilhas, 0 re-cozidas, recook 0,01 ms** em cena parada.

## ⚠️ W2 — a PILHA, e o que ela MUDA no que já estava aprovado

**Todo op é imagem → imagem** (a frase que governa a pilha de geometria, traduzida para raster) ⇒
**Glow e Drop Shadow compõem o halo POR BAIXO da entrada DENTRO do próprio op**. Um op que
devolvesse duas camadas não poderia alimentar o seguinte.

⚠️ **Consequência de PRODUTO, nomeada:** o `FxMode::Below`/`Replace` **morreu**, e uma forma com
Glow/Drop Shadow passa a ser desenhada INTEIRAMENTE a partir da textura (a W1 desenhava o vetor
crisp por cima). O scratch rasteriza na escala EXATA da tela e o retângulo é alinhado ao pixel,
então é 1:1 — **mas é o olho do smoke que decide**, e a estrela 4 (`=33`) é o controle disso.

⚠️ **`ph2d_vec_render::FxImage` perdeu o campo `mode`** e `FxMode` deixou de existir. Único
chamador do `dispatch` é a `render_loop`.

## Deltas que a integração precisa CONFERIR (o número se conta, não se escolhe)

| Item | Antes | Depois | Como |
|---|---|---|---|
| `ph2d-ecs` registry | 37 | **38** | `VecFilter` registrado (blob-key) |
| espelhos `ph2d-render`/`ph2d-script` | 38 | **39** | ecs+Sprite / ecs+LuauScript |
| `PROJECT_SCHEMA` | 31 | **31 (INTOCADO)** | componente por blob-key = sem bump posicional |
| `VEC_SCENE_SCHEMA` | 13 | **13 (intocado)** | — |
| **§6 contrato vetorial** | — | **INTACTO** | `architecture_vector_contract_surface` verde |
| `VECTOR_SECTIONS` (painel) | 26 | **27** (append) | Filters (gate de contagem atualizado) |

⚠️ Se outra linha mexer no registry/`PROJECT_SCHEMA` na MESMA janela, o número final **se
recalcula** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). O `VecFilter` não move
`PROJECT_SCHEMA` (blob-key), então esse eixo não conflita; o registry (38/39) pode.

## O que landou

- **`ph2d-ecs::FxOp` + `VecFilter { ops: Vec<FxOp> }`** ([vec_filter.rs](../crates/ph2d-ecs/src/vec_filter.rs)).
  `MAX_OPS = 6` — **o teto é do PAINEL, não da GPU**, e está MEDIDO (RTX, 512×512, sigma 8 px):
  `1 → 0,084 ms · 2 → 0,149 · 3 → 0,220 · 4 → 0,336 · 6 → **0,429 ms**`, linear a ~0,07 ms/degrau
  = **2,6 % de um frame de 60 fps** com a pilha cheia.
- **`ph2d-render::FxStackPass` + `FxOpGpu` + `stack_reach` + `kernel_half`** — `stack_reach` é a
  porta única da margem (as reaches SOMAM ao longo da pilha; assimétrica para a sombra), e ela e o
  shader perguntam o MESMO `kernel_half`.
- **`ph2d-vec-render`**: `FxImages`/`FxImage` + `dispatch(...,fx,...)` + `path_screen_bounds` +
  `draw_path_isolated`.
- **`fx_live`** (shell) — o produtor + `resolve_ops` (mundo→pixel, a câmera mora ali e em mais lado
  nenhum) + **`hit_of`** (id→controle, a porta que os TRÊS sítios da ponte usam).
- **Painel "Filters"** — uma LISTA de cards (o idioma da seção Effects): Add por tipo, e por card
  ✕ / ↑ / ↓ / 👁 + os controles do TIPO da linha. Ids **por-linha** (`filter_*_id(row)`).

## Gates

- **GPU (`ph2d-render/tests/fx_stack_gpu.rs`, 8, `#[ignore]`):** rampa alarga com sigma · o halo é
  do EFEITO e a FORMA sobrevive por cima · **a ORDEM da pilha muda o desenho** (o gate da wave) ·
  pilha vazia é a identidade · o custo por degrau (medição) · `stack_reach` (puro, roda em qualquer
  runner) · os 3 de render/register/resize que reproduzem os panics da W1.
- **Shell (5):** os tetos painel↔motor concordam · degrau desligado nunca chega ao passe · o raio é
  de MUNDO (2× de zoom = 2× de borrão) · o offset cruza a câmera e cai em pixel inteiro · `hit_of`
  decodifica cada controle **e nada mais**.
- **Modelo (4, `ph2d-ecs`):** reordenar troca vizinhos e as pontas são no-ops · a pilha só está
  ativa com algum degrau ligado · um degrau novo nasce VISÍVEL · o teto é resposta da pilha.
- **Seam (6, `seam_filters.rs`):** os Add ao bus · os ícones do card ao bus · as setas das PONTAS
  **não** são desenhadas · cada linha pinta só os controles do TIPO dela (presença E ausência) · a
  swatch é alvo de PICKER (não botão) · a seção não é oferecida sem forma.
- Fechamento: `cargo fmt` · clippy limpo · LOC caps (workspace + shell + painel) · §6 ·
  `node_id_collisions` (agora cobrindo as DUAS famílias por-linha) · `panel_wiring_parity` ·
  `cargo test --workspace --no-run` exit 0.

⚠️ **Os gates GPU são `#[ignore]`** — o integrador roda
`cargo test -p ph2d-render --release --test fx_stack_gpu -- --ignored` na RTX (**8/8 verdes**; sem
adapter fazem *skip gracioso*, que não é verde).

## ⚠️ Três lições que a wave pagou

1. **A minha 1ª mutação estava ERRADA e sobreviveu.** *"Todo op vira o primeiro"* não reproduz *"a
   ordem é ignorada"* — produz outra coisa errada (`[glow,blur]`→glow-glow e `[blur,glow]`→blur-blur
   **continuam diferentes**). A mutação honesta é **ORDENAR** a pilha: aí os dois lados ficam
   idênticos, `0 bytes diferentes`, RED. *Uma mutação que não sangra pode acusar a MUTAÇÃO, não o
   gate.*
2. **O seam nasceu VERMELHO e apontou um erro meu:** a swatch de cor registada como `button()`, e
   **um id só pode ter UM tipo de widget no store** — o Down abria o picker e nenhum `Click` saía.
   A mesma lição que o `vector_fx_toggle_id` já documentava.
3. **O `node_id_collisions` não cobria nem `vector_fx_*` nem a família nova** — as duas partilham o
   prefixo `vector.f…`, então *"os nomes são diferentes"* era uma afirmação por provar exatamente
   onde é duvidosa. As duas entraram no MESMO conjunto (é isso que prova a distinção).

## Smoke

`cd <worktree> && env PH2D_BUILD_SMOKE=33 cargo run -p ph2d-host-desktop --release` — **sete
estrelas em duas fileiras**:

- **CIMA (a regressão da W1):** controle nítido · **Blur** · **Glow** ciano · **Drop Shadow**.
- **BAIXO (a W2):** a **PILHA INTEIRA** (`Shadow → Blur → Glow`, três degraus numa forma só) · e o
  **PAR DE ORDEM** (`Glow → Blur` × `Blur → Glow`), que tem os MESMOS dois degraus trocados —
  **se as duas parecerem iguais, a pilha não está compondo**.

Dê ZOOM (o borrão cresce — o raio é de MUNDO) e MAXIMIZE (o resize re-registra).
`PH2D_FX_PERF=1` imprime `re-cozida(s)` + ms do recook.

**E o smoke do PAINEL:** abra o Vector, desenhe uma forma, selecione → seção *Filters* →
**Add Blur** / **Add Glow** / **Add Drop Shadow** → afine Radius/Offset/Color/Opacity → use as
**setas** do card para reordenar (o desenho tem de mudar) → o **olho** desarma sem perder os
números → o **✕** apaga (e o último ✕ tira o filtro da forma).

## Aberto / follow-ups (nomeados, não contrabandeados)

- **Só três TIPOS.** O `apply_op` é a porta por onde um tipo novo entra com um braço de shader e um
  `kind_name` — color-matrix (tint/duotone), morphology (dilate/erode), displacement + turbulence,
  bevel. **Nenhum deles muda a pilha**, e é isso que a W2 comprou.
- **W3 — o feather analítico** (erf da distância / SDF via JFA), quando a nitidez em zoom extremo
  importar.
- **Radius é slider em unidades de MUNDO** (`FILTER_RADIUS_MAX = 2.0`) — fração-do-tamanho seria
  mais robusto para formas de tamanhos diferentes (a mesma nota que o Contour faz do Offset).
- **O deslocamento da sombra é arredondado ao PIXEL** (o halo é amostrado por `textureLoad`, sem
  sampler). Invisível numa sombra; nomeado por honestidade.
- **`MAX_HALF = 96`** (sigma ≈ 32 px de tela): acima, o borrão satura — limite de CUSTO do passe.
- **A pilha de filtros não compõe com a de Effects numa ordem escolhida** (o filtro roda sempre
  DEPOIS da geometria cozida). É decisão de produto se um dia precisar.

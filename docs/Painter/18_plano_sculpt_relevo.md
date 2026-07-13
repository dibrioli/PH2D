# 18. SCULPT do relevo — plano de implementação

> **Ordem do Enio (2026-07-13):** *"É hora de mais ferramentas como as do sculpt do Blender. […]
> Falta inflate, pinch, grab, Thickness, e a mais valiosa e importante: **Smooth**."*
>
> Este documento **decide**. Onde ele diz "decisão", a implementação não reabre — reabrir custa uma
> ADR ou uma ordem nova do Enio. Onde ele diz "bifurcação", a decisão está **adiada de propósito** e o
> critério de desempate está escrito.
>
> Pré-requisito de leitura: [`16_impasto_plano_implementacao.md`](16_impasto_plano_implementacao.md)
> (o relevo, o filme, a luz, o material) e a regra-mãe que sustenta os dois:
> **o relevo é o segundo output da MESMA lista de dabs.**

---

## §1 — A restrição que decide a lista inteira

**`h` é uma FUNÇÃO** `h(x, y)` — um valor por pixel. Um campo de altura **não faz overhang**. Isso não
é uma limitação da implementação; é o que o objeto **é**. Três consequências, e elas separam o
compatível do impossível antes de qualquer linha de código:

1. **Não há topologia.** Não existem vértices, nem vizinhança, nem borda. Tudo que o Blender resolve
   caminhando na malha (Pose, Cloth, Boundary, Slide Relax, Face Sets, Dyntopo, Voxel Remesh) **não tem
   versão fiel aqui**. Não é "difícil": é *outra coisa com o mesmo nome*. Não portar.
2. **Não há matéria dobrada sobre si mesma.** O Snake Hook não engancha. Ele degenera para "arrastar uma
   crista", que é o Nudge. **Não portar sob esse nome** — batizar de Snake Hook o que não engancha é
   prometer o que não se entrega.
3. **A normal é DERIVADA** (`∇h`), não guardada. "Deslocar ao longo da normal" vira "deslocar em `z`,
   escalado por `n_z`". O Inflate 3D empurra os lados; o nosso **não pode**. Ele vira um *estufar* que
   arredonda cristas. Compatível e distinto do Draw — mas seja honesto no tooltip: é um parente pobre.

Tudo o que **sobra** é uma de três coisas, e é por isso que a lista abaixo é curta e o custo é baixo:

| Classe | O que o pincel faz com `h` | Motor |
|---|---|---|
| **A. Aditiva** | soma uma função da máscara do dab | já existe (é o Draw do relevo) |
| **B. Local** | lê `h` na pegada e reescreve (blur, plano, clamp) | **novo** — é o núcleo desta linha |
| **C. Advectiva** | **move** `h` lateralmente | **já existe** — o motor de warp do Deform |

---

## §2 — O que já temos (não reconstruir)

Levantado no código, não de memória (2026-07-13):

| Blender | Nosso | Onde |
|---|---|---|
| Multires Displacement **Smear** | **Plow** — o Smear já arrasta o relevo | `height::plow_dab_height` |
| Multires Displacement **Eraser** | a borracha de relevo | `height::erase_dab_height` |
| **Mask** | a Selection do painter | `selection_coverage_at` |
| **Pinch** (de PIXELS) | `DeformMode::Pinch` | `paint/warp/field.rs` |
| **Smooth** (o kernel!) | o *settle* — box blur separável de `h` | `impasto_settle::settle` |
| deslocamento **conservativo** | o Push (soma exatamente zero) | `height_push::bank_dab_push` |

E o **"Thickness"** que o Enio citou: o nome vem do sculpt do **Grease Pencil**, não do 3D. No impasto,
espessura **é** o `h` — então "Thickness" é o Draw restrito à altura, e isso **já é um modo**:
`DrawTo::Depth` (*"a palette knife: body, no pigment"*). **Não é ferramenta nova.**

---

## §3 — DECISÃO 1: onde o sculpt mora

**`PaintMode::Sculpt`, espelhando `PaintMode::Deform` linha por linha.** Não é tool nova, não é crate
nova: o painter já é o dono do relevo, e o Deform já provou a forma.

O molde (copiar, não inventar):

- `SculptState` em `PaintState` (mode-exclusive, como `DeformState`), com `mode: u8` + os knobs.
- Sub-modos num **segmented** (o Deform tem 6; nós teremos ~10) — `SculptMode::from_u8` clampa.
- Botão no rail (`PAINTER_RAIL_SCULPT`, ao lado do Deform) — **`PAINTER_RAIL_TOOL_IDS` vai de 11 → 12.**
- Seção no painel dirigida por um flag `is_sculpt` no snapshot `BrushSettings` (como `is_deform`).
- Setters = **a única fonte de clamp** (`apply_ui_edit`), roteados de `handle_panel_event`.
- Undo: **uma entrada estrutural por traço** (`close_stroke` → `commit_structural_edit`). O `heights` já
  é capturado pelo `ModelSnapshot`; a **sessão** do sculpt (§4) precisa ser capturada também — como
  `deform_disp`/`deform_pre` são.

> ⚠️ **A armadilha que esta linha já pagou duas vezes:** todo widget novo tem que ser **registrado no
> `WidgetStore`** (`populate.rs`) ou o ponteiro nunca vira `Click` e o controle nasce **morto**. E o gate
> obrigatório é um teste que **CLICA** nele (`MockPanelHost::click_at`). Ler
> [[feedback_widget_is_done_when_a_test_clicks_it]] **antes** de pintar o primeiro chip.

---

## §4 — DECISÃO 2: o padrão POR-TRAÇO (a mais importante do documento)

**Um dab de sculpt NÃO edita `h` no lugar.** Ele acumula num **campo de intensidade por-traço**, e o
resultado é re-renderizado **do `h` congelado no pen-down**.

```
sculpt.pre     : Vec<f32>   — o h da camada no pen-down (o "src" congelado)
sculpt.amount  : Vec<f32>   — quanto de efeito cada texel acumulou neste traço
                              (o dab SOMA aqui; nada mais)
re-render      : h[i] = kernel(sculpt.pre, sculpt.amount[i])   sobre a bbox do dab
```

É **exatamente** o que o motor de warp já faz (`warp/apply.rs`: *"each dab does NOT re-gather the
already-warped canvas […] it ACCUMULATES its displacement into a per-stroke map and re-renders the dab
bbox from the frozen stroke-start pixels"*). Copiar a disciplina, não só a ideia.

**Por que isto não é negociável — três razões, em ordem de força:**

1. **Idempotência sob re-stamp.** Os shape editors (Curve / Line / Polygon) **re-carimbam o traço
   inteiro a cada frame**. Um Smooth que borrasse `h` no lugar derreteria o relevo progressivamente
   enquanto o artista só *olha* para a curva. Esta linha já pagou esse preço uma vez — é a razão de o
   `stroke_push` ser idempotente por construção (§13 do doc 16). **Um efeito local que não é idempotente
   é incompatível com os shape editors, e os shape editors não são negociáveis.**
2. **Composição.** Dabs se sobrepõem (spacing 10% ⇒ ~10 dabs por texel). Blur composto 10× ≠ blur ×10:
   é difusão, e a força efetiva passa a depender do **espaçamento**, não do que o artista pediu.
   Acumular a *intensidade* e aplicar **uma vez** dá um resultado que é função da INTENÇÃO, não da
   geometria do carimbo. (E compõe com o `space_attenuation`/`accumulate` que o pincel de tinta já tem —
   reusar essa semântica, não inventar outra.)
3. **Ele destrava o "Adjust Last Stroke" de graça.** Com `pre` + `amount` guardados, mexer no Strength
   depois do traço **re-renderiza** — igual a Depth e Body. Sem eles, o traço de sculpt é destrutivo e o
   toggle não alcança o sculpt, o que seria uma inconsistência que o artista sente na hora.

**Gate que torna isso executável (escrever ANTES do kernel):**
`o_mesmo_traço_carimbado_duas_vezes_dá_o_mesmo_canvas` — carimbar o traço, re-carimbar (o que o shape
editor faz por frame), e afirmar **byte-idêntico**. Na implementação ingênua isso nasce **vermelho**.

---

## §5 — DECISÃO 3: o sculpt escreve `h`, e só `h`

Não escreve RGBA, não escreve `covers`, não escreve `mats`.

**Razão:** raspar o **corpo** da tinta não tira o **pigmento** — e essa distinção já está no modelo
(`DrawTo::Depth`: *"body, no pigment"*). Um Scrape que apagasse a cor seria a borracha, que já existe.

**Consequência que o implementador vai encontrar:** a luz pesa por `cover` (o `body`), então relevo em
cima de cobertura zero **não acende**. Isso é *certo*: sculpt não cria tinta onde não há tinta. Vira
gate: `o_sculpt_não_acende_papel_nu`.

**Exceção — a família ADVECTIVA (W4).** Empurrar tinta **move a matéria**, e matéria carrega cor. Grab /
Pinch / Nudge têm que advectar **`h` + `covers` + `mats` + RGBA juntos**. É o que separa "sculpt numa
imagem" de "mexer em tinta", e é uma decisão de **modelo**, não de código.

---

## §6 — BIFURCAÇÃO 1 (adiada): para onde vai a tinta raspada?

O Blender **deleta** matéria: o Scrape some com ela, o Draw cria do nada. **Tinta não faz isso.** E nós
temos um motor de deslocamento **conservativo** (`height_push`, soma exatamente zero por construção).

Então o Scrape/Flatten podem **empilhar** o volume removido na borda da espátula — a *bow wave* que a
pesquisa do Push já apontou (doc 17 §3, mecanismo 6). Isso não seria portar o Blender: seria **estar
mais certo que ele**, porque o que estamos esculpindo é tinta.

**Decisão de escopo:** W2 entrega o Scrape **não-conservativo** (como o Blender), **mas o kernel COMPUTA
o volume deslocado** (`Σ Δh` na pegada) e o joga fora explicitamente. Assim o `Conserve` de W5 é um
**flag**, não uma reescrita. *Critério de desempate:* se o Enio olhar o Scrape e disser "cadê a tinta que
eu tirei", W5 sobe na fila.

---

## §7 — DECISÃO 4: o plano local (o núcleo de Scrape / Fill / Flatten / Clay)

Quatro pincéis, **um kernel**. O Blender ajusta um "area plane" à pegada; nós ajustamos o mesmo, e num
campo de altura ele é barato:

```
ajuste de mínimos quadrados sobre a pegada, ponderado pela máscara do dab:
    plano(x, y) = h̄ + gx·(x − cx) + gy·(y − cy)
três acumuladores (h̄, gx, gy); zero transcendental (HR-5)
```

**O plano é INCLINADO, não plano-horizontal**, e isso é o que faz a espátula **seguir a superfície** em
vez de cavar uma cratera chata numa encosta. É a diferença entre uma ferramenta e um bug.

Com `plane_offset` (subir/descer o plano, o Blender tem) os quatro verbos caem de uma equação:

| Verbo | Regra |
|---|---|
| **Flatten** | puxa `h` para o plano, nos dois sentidos |
| **Scrape** | idem, **só para baixo** (`min`) — a espátula raspando |
| **Fill** | idem, **só para cima** (`max`) — preenchendo vales |
| **Clay** | Draw **+** Flatten: deposita *e* achata contra o plano |

---

## §8 — As ondas

### W1 — SMOOTH ✅ **FECHADA (2026-07-13)** — entregou Smooth **e Sharpen**

> Handoff: [`HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md`](../HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md).
> Sharpen entrou junto porque é este mesmo kernel com o sinal trocado (W3 já dizia "cai de graça") e um
> segmented de UM chip é cheiro de design. **Três coisas que este plano previu errado, e o código corrigiu:**
>
> * **§3 manda "espelhar o Deform linha por linha".** O painel do sculpt é **ADITIVO**, não mode-exclusive
>   — porque o §10.1 (o invariante) diz que ele monta na lista de dabs do pincel, e então **Size / Spacing /
>   Falloff / Shape / Grain do PINCEL são a espátula**. Escondê-los, como o Deform esconde, deixaria o
>   artista com os ajustes de uma ferramenta que ele não consegue mirar. O §10.1 ganha do §3.
> * **§3/§8 preveem um Strength do sculpt.** Não existe: o **pincel já tem um**, e `Dab::coverage` já o
>   carrega. Dois knobs disputando o mesmo número é bug de design.
> * **§8 W1.3 diz `amount` = máscara × Strength.** O `amount` acumula a máscara × coverage (com o fold da
>   casa); o que fica **vivo depois do traço** são os knobs do CARD (Radius, Smooth↔Sharpen), não o gesto
>   da mão — que é exatamente a divisão que o card Body já faz.

### (histórico) W1 — o escopo como foi planejado

O item de **maior valor e menor custo** da lista: o kernel já existe (`impasto_settle::box_blur`,
separável, e — leia o comentário dele — **cada texel re-soma a janela do zero de propósito**, porque a
soma corrida acumulava erro de float *ao longo da linha* e quebrava a byte-identidade do crop).

O trabalho **não é o blur**. É:
1. `PaintMode::Sculpt` + `SculptState` + rail + seção do painel (§3) — o grosso.
2. O motor por-traço (`pre` + `amount` + re-render) (§4) — a peça que decide tudo depois.
3. O kernel: `h = lerp(pre, blur(pre, r), amount)`, com `amount` da máscara do dab × Strength.

**Gates (o vermelho vem antes do verde):**
- `o_mesmo_traço_carimbado_duas_vezes_dá_o_mesmo_canvas` (§4) — **o gate que define o desenho**.
- `strength_zero_é_byte_idêntico`.
- `o_smooth_reduz_a_magnitude_do_gradiente` (ele faz o que diz).
- `o_smooth_não_acende_papel_nu` (§5).
- `o_smooth_respeita_a_seleção` (reusa `selection_coverage_at`, como o Deform).
- **O seam:** um teste que **CLICA** no botão do rail e no chip do sub-modo (§3 ⚠️).
- Perf: kill criterion espelhando o do impasto (≤4 ms/movimento @2048², kill 8).

### W2 — A ESPÁTULA: Scrape · Fill · Flatten (+ Multi-plane Scrape)
Um kernel (§7), quatro verbos. O `plane_offset` é o que dá mordida a Scrape/Fill.
Gates: cada verbo só move no seu sentido · **o plano segue a encosta** (o gate que mataria o ajuste
horizontal) · o volume deslocado é computado (§6).

### W3 — Clay · Clay Strips · Layer · Sharpen · Draw Sharp · Inflate
Todos **composições dos kernels de W1/W2** — nenhum motor novo:
- **Clay** = Draw + Flatten. **Clay Strips** = o mesmo com falloff quadrado.
- **Layer** = `clamp` contra o `pre` congelado (que W1 já guarda) — "2 mm de tinta, nem mais".
- **Sharpen** = o kernel do Smooth com sinal invertido (`h += k·(h − blur(h))`). *Cai de graça.*
- **Draw Sharp** = Draw que desloca a partir do `pre` (crista afiada, não acumulada).
- **Inflate** = `h += k·n_z` (§1.3 — honesto sobre o que ele não é).

### W4 — A família ADVECTIVA: Grab · Pinch · Nudge · Rotate · Thumb
**Não construir um motor novo.** Fazer o motor do Deform **carregar os planos do relevo** (`h`,
`covers`, `mats`) junto do RGBA (§5, exceção). Isso destrava **cinco pincéis de uma vez** e unifica
"Liquify" e "sculpt-warp" numa engine só.
Decisão de superfície a tomar em W4: os warps de relevo são **sub-modos do Sculpt** ou um **toggle
"afeta o relevo" no Deform**? Recomendação: o segundo — um motor, um lugar.

### W5 — Conserve (a *bow wave*, §6) + filtros de camada inteira (Smooth/Sharpen/Inflate/Relax)

---

## §9 — O que NÃO entra (e por quê)

Pose · Cloth · Boundary · Slide Relax · Simplify/Dyntopo · Voxel Remesh · Face Sets — **precisam de
topologia ou volume que um campo de altura não tem** (§1.1). Snake Hook — **degenera** (§1.2).

Escrever isso aqui é metade do plano: uma lista de compatíveis sem a lista de **incompatíveis** convida
alguém a tentar, e a tentativa custa uma semana antes de bater na parede que este parágrafo já descreve.

---

## §10 — Invariantes que valem para toda a linha

1. **A lista de dabs é uma só.** O sculpt consome a MESMA (`stamp_dabs_height` é o choke point) ⇒
   Symmetry, Tiling, os shape editors, pressão, jitter, falloff, Shape e **Grain** saem de graça — e uma
   espátula com Grain é uma espátula texturizada, que é um presente. **Um passe com geometria própria é
   como nasce "Tiling não funciona no Sculpt" daqui a seis meses.**
2. **HR-5:** zero transcendental no laço (box blur = somas; plano = 3 acumuladores).
3. **Teto:** `H_CEIL` continua valendo — o sculpt não pode estourá-lo.
4. **Undo:** uma entrada por traço; a sessão (`pre` + `amount`) entra no `ModelSnapshot`.
   *(Precedente fresco: o `mats` **ficou de fora** do snapshot quando o material landou, e o buraco se
   escondia — na tela vazia a cobertura zera e a luz pesa o material obsoleto por zero. Só apareceu em
   tinta-sobre-tinta. **Ao adicionar um plano, adicione-o ao snapshot no mesmo commit.**)*
5. **A luz não muda.** O sculpt mexe em `h`; o passe de luz (doc 16 §14-18) já sabe o que fazer com ele.

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
3. ~~**A normal é DERIVADA** (`∇h`), não guardada. "Deslocar ao longo da normal" vira "deslocar em `z`,
   escalado por `n_z`". O Inflate 3D empurra os lados; o nosso **não pode**.~~ **ERRADO — corrigido em
   2026-07-14 (smoke do Enio).** Um campo de altura **PODE** empurrar os próprios lados: o deslocamento
   exato ao longo da normal é a **dilatação morfológica por uma BOLA**, e é justamente daí que sai o
   crescimento lateral. O `Depth·n_z` que este parágrafo prescreveu tinha a normal **de cabeça pra baixo**
   (o correto é `Depth·S = Depth/n_z`, a *secante*), e — o que importa mais — **nenhuma** fórmula por-texel
   pode inflar coisa alguma: `h + d·S` é UM passo de Euler da PDE de offset, e um passo não move matéria
   *de lado*. Sobre o relevo que o depósito de fato deixa, `n_z = 1.000` na mediana, então o Inflate era
   **Layer, ao bit**. Ver `crates/ph2d-tool-painter/src/tool/paint/sculpt_offset.rs`.

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
3. ~~**Ele destrava o "Adjust Last Stroke" de graça.**~~ — **RAZÃO ERRADA. Riscada no smoke do Enio
   (2026-07-13).** Ela dizia: com `pre` + `amount` guardados, mexer nos knobs depois do traço
   re-renderiza — igual a Depth e Body. Foi implementado assim, e o Enio derrubou em uma frase: pegar o
   **Sharpen** (para afiar em *outro lugar*) convertia o Smooth que ele acabara de fazer no oposto dele.

   O erro foi raciocinar **por analogia com o depósito sem checar se a analogia vale**. Tinta é uma
   **substância**: Depth/Body são propriedades *da tinta que aquele traço depositou*, então "me deixa
   continuar afinando" é uma oferta coerente — e o checkbox a faz. Um traço de sculpt é uma **operação**:
   não deixa para trás nada que tenha propriedades, só o relevo, como ele está agora. Não existe "o
   smoothing" ali parado para ser re-parametrizado. Operações se **desfazem**, não se re-discam. E o
   **Mode nem é parâmetro** — é *qual ferramenta*: um verbo que reescreve o passado quando você o
   seleciona não é ajuste, é destruição que o artista não pediu.

   **O motor (§4) sobrevive intacto — as razões 1 e 2 são as que o sustentam, e elas não dependem disto.**
   O que morreu foi o que eu fiz com ele. A regra que ficou: **a sessão vive exatamente enquanto o gesto
   não foi comitado** — pen-up (freehand) ou Apply (shape) a mata. O que *continua* re-renderizando ao
   vivo é um **shape aberto**, que tem botão Apply justamente por ainda não ser tela; um card que ficasse
   inerte *ali* deixaria a curva na tela discordando do card que a descreve.
   Gates: `the_sculpt_knobs_do_not_touch_a_finished_stroke` + o irmão de presença
   `the_sculpt_knobs_re_render_an_open_shape` (sem ele, apagar o refresh inteiro passa no primeiro).

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

**Exceção — a família ADVECTIVA.** Empurrar tinta **move a matéria**, e matéria carrega cor. Quem move
matéria tem que advectar **`h` + `covers` + `mats` + RGBA juntos**. É o que separa "sculpt numa imagem" de
"mexer em tinta", e é uma decisão de **modelo**, não de código.

> ### ⚠️ O **INFLATE** É DESSA FAMÍLIA — e eu o arquivei do lado errado da minha própria regra
>
> (2026-07-14, 2º smoke do Enio: *"inflate não engorda"*. Não engordava.)
>
> A regra acima estava escrita **antes** de o Inflate existir, e prevê exatamente o que aconteceu. Mas eu a
> li como sendo sobre **W4**, e o Inflate parecia um verbo de *reshape*. Não é: o Inflate **CRESCE A FORMA**
> — a dilatação empurra a borda pra fora, sobre texels que **não tinham tinta**. E a consequência que este
> mesmo parágrafo já anota (*"a luz pesa por `cover`, então relevo sobre cobertura zero não acende"*) é o
> bug: o campo de altura engordava, e **a tela não mudava**. Os gates mediam o `heights`.
>
> Agora a bola responde a **DUAS** perguntas — *que altura* e **de onde veio a matéria** (o argmax,
> `memo_src`) — e o que chega num texel traz a **cobertura, o material e a COR** de onde veio.
> `SculptMode::moves_matter()` é o único lugar que diz quem é dessa família; W4 entra ali.
>
> **A lição não é "faltou um gate". É que a regra estava escrita e eu não a apliquei ao caso novo.**
> Uma exceção documentada só protege quem *reconhece* que está dentro dela.

> ### ⚠️ 3º SMOKE — VIROU O BLOB (2026-07-14)
>
> *"não é inflate … puxa as faces na direção das normais usando para intensidade o falloff … estude o Blob
> do Blender."* Corretíssimo. A bola de raio **constante** (mesclada pelo falloff no render) engordava mas
> deixava **topo chato** — dilatação morfológica é filtro de máximo, plana onde a bola cabe: o *"mistura de
> inflate com layer"*. Estudei o Blender: **Inflate** move o vértice pela normal (o componente **lateral** é
> o que engorda — 2.5D não tem); **Blob** impõe uma **esfera** moldada pelo falloff.
>
> Fix: o raio da bola **segue o falloff** — `|Depth|·amount·DEPTH_UNIT_PX`. Centro cheio, borda→0. Domo
> **redondo** (pico=Depth no centro; no chato `[0.5, 0.45, 0.34, 0.19, 0.06, 0]`), **engorda** (a borda
> sobe), **afunila** na borda do pincel. É o Blob no 2.5D.
>
> **Não é memoizável** (o raio depende do `amount` vivo). A esfera exata é `O(área·ρ²)` = **73 ms/move** num
> pincel grande. Uma esfera é bem aproximada perto do topo por uma **parábola**, e dilatação por parábola é
> **separável** (`O(N)`, Felzenszwalb 2004) — mesma aparência, **4,2 ms/move**. Pico da parábola =
> `pre±|Depth|·amount`; curvatura constante = a esfera no ápice. A matéria segue a bola pelo **argmax
> composto nos 2 passes**. O **sinal do lift** já me virou o domo do avesso uma vez — gate byte-idêntico vs
> força-bruta O(N²) pega (`the_parabolic_blob_matches_the_brute_force_dilation`). Código morto da bola
> constante (memo `Offset`, `ball_offset_into`) removido.
>
> ### ⚠️ 3º SMOKE, PARTE 2 — *"falloff de influência retangular bizarra"* (2026-07-15)
>
> O domo *funcionava* mas vinha cercado por um **quadrado duro** em tinta **grossa**. Causa: a parábola
> separável **não tem suporte** — um `pre` alto (paint acumulado) lança um "saia" de `√(H/a)` ≈ **100+ texels**,
> e escrita só dentro de `kr = brush + 2ρ` isso é **recortado no retângulo**. Uma bola de raio ρ alcança ρ,
> por mais alto que seja o penhasco em que ela rola; a parábola não. Fix (`render_inflate`, **um `sqrt` por
> texel recortado**): o **argmax composto já diz a distância** que a matéria vencedora viajou, então
> `dx²+dy² > 2ρ²` (o raio ρ√2 onde a parábola já caiu o `|Depth|` inteiro — onde a bola de verdade termina)
> **re-crava a fonte no aro da bola e lê a altura ali**. É **circular** (limita `dx²+dy²`, não cada eixo →
> nunca desenha um quadrado) e **simétrico**: dilatação de tela nua ao lado dum muro cai em `pre` (a fonte do
> aro é baixa), erosão dum sulco fino ainda alcança o ombro dentro de ρ√2 (a fonte do aro **é** o ombro, e a
> cavada sobrevive — um `fall-to-pre` cru mataria a erosão porque a forma a ser cavada **cerca** o texel). Em
> `pre` chato o aro coincide com `pre` ⇒ o clamp não muda **nada** (só corta a cauda de runaway). Gate:
> `the_inflate_offset_reach_is_bounded_not_a_runaway_rectangle` (mutação = clamp nunca dispara → probe a 35 px
> sobe pra **19,78 loads**); a erosão de sulco fino (`a_negative_depth_erodes…`) prova que o **re-sample no
> aro** é a decisão certa vs `fall-to-pre`. Perf intacto (**4,57 ms/move** a 4096 px).

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

### W2 — A ESPÁTULA ✅ **FECHADA (2026-07-13)** — Flatten · Scrape · Fill

Um kernel (§7), e agora **cinco verbos numa expressão só**: `h = pre + k·Δ`, onde o verbo escolhe *de
onde vem o alvo* e *qual SINAL de Δ passa*.

| verbo | alvo | Δ |
|---|---|---|
| Smooth | `blur(pre)` | os dois lados |
| Sharpen | `pre + (pre − blur(pre))` | os dois lados |
| Flatten | o **plano** ajustado | os dois lados |
| Scrape | o plano | **só pra baixo** (`min(Δ,0)`) |
| Fill | o plano | **só pra cima** (`max(Δ,0)`) |

Scrape e Fill **não são motores novos** — são o Flatten com metade do número jogada fora.

**O que o plano não previa (3 coisas):**

1. **O alvo do plano é uma MÉDIA PONDERADA por-texel, não um plano por-dab.** Cada dab soma
   `plane_sum[i] += w·plano_d(i)` com o MESMO peso que já soma em `amount[i]`; o render divide. Isso é o
   que preserva o motor por-traço do §4 — guardar "o plano" exigiria a lista de dabs no render, que os
   shape editors jogam fora e reconstroem a cada frame. E a divisão torna o alvo **independente de
   Strength e Flow** (eles escalam os dois lados): Strength decide *quão longe* você vai até o plano, não
   *onde o plano está*.
2. **O Offset NÃO entra na acumulação.** É um deslocamento rígido, então
   `Σw·(plano+off) = plane_sum + off·amount` — o render soma no fim. Custo: o slider fica vivo num shape
   aberto **sem re-carimbar um dab**.
3. **`blurred` e `plane_sum` são mutuamente exclusivos** (um verbo pertence a uma família só) ⇒ a sessão
   segue em **12 B/px** com cinco verbos, não 16. Trocar de família num shape aberto **re-carimba**
   (`set_sculpt_mode` → `refill_open_shape`), porque `plane_sum` é função da LISTA DE DABS e não dá pra
   reconstruir do `pre` — sem isso, Smooth→Scrape dividia por zero e puxava a tinta pro **chão** do canvas.

**Volume deslocado (§6): computado, exposto (`sculpt_displaced_volume`), descartado de propósito e
GATEADO agora** — o Conserve de W5 é um flag, e um número que ninguém conferiu no dia em que foi escrito
não seria.

**A lição do gate (vale mais que o código):** o fit horizontal — o bug que este §7 inteiro existe pra
evitar — **é invisível AO LONGO do traço**. A média móvel dos planos horizontais reconstrói a encosta por
acidente (cada plano fica na altura média do seu footprint, que acompanha o morro). O dano só aparece
**perpendicular ao traço**, onde nada o reconstrói. O 1º gate de produto media a inclinação *ao longo* e
ficou **verde sob a mutação**. Gates: `flatten_on_a_pure_ramp_is_a_no_op` (rampa em 2 eixos) +
`the_scrape_takes_the_marks_off_the_hillside_and_leaves_the_hill` (encosta **atravessando** o traço).

**Multi-plane Scrape: NÃO entrou** — e não é adiamento por cansaço. Ele precisa de (a) um ângulo, que é
`tan(θ)` = transcendental (HR-5), (b) a direção do traço, que é `[0,0]` no 1º dab, e (c) um 4º knob no
card. É W3, junto do Clay, e cabe no mesmo fit (dois ajustes nas duas metades do footprint).

### W3 ✅ **FECHADA (2026-07-13)** — Chisel · Layer · Inflate (e **três que NÃO são verbos**)

**Oito verbos, uma expressão.** `h = pre + k·Δ`. Chisel é Scrape com um `abs`; Layer é o kernel com alvo
**constante** (é isso que o limita: `k ≤ 1`, então nunca passa de `pre + Depth`); Inflate é o kernel com
alvo `offset(pre, Depth)` — o relevo **dilatado (ou erodido) por uma bola** de raio `Depth · DEPTH_UNIT_PX`.

> **CORREÇÃO 2026-07-14 (smoke do Enio: *"Inflate parece fazer a mesma coisa de Layer"*).** Ele fazia.
> O alvo era `pre + Depth·n_z`, e (a) a normal estava **invertida** — o offset verdadeiro sobe pela
> *secante* `Depth·S`, não pelo cosseno, e é por isso que uma parede **anda de lado** e a forma **engorda**;
> (b) mesmo com o sinal certo, `n_z = 1.000` na **mediana do texel pintado** (o miolo de um traço é chapado),
> então qualquer `Depth·f(|∇h|)` é `Depth`. **A família mudou:** Inflate saiu de `Height` (sem buffer) e
> entrou no **memo** (o mesmo maquinário de tiles do blur, outro kernel) — 12 B/px, e `Height` agora é só o
> Layer. Detalhe e prova: `sculpt_offset.rs` + `sculpt_tests/inflate.rs`.
>
> **Inflate e Layer CONCORDAM num plano — e isso é geometria, não bug**: deslocar um plano ao longo da
> normal é transladá-lo (no Blender, Inflate num plano chato também é Draw). A diferença é o que fazem com a
> **forma**, e há gate pinando a identidade pra ninguém "consertá-la" de volta pra uma inversão.

**Três da lista original NÃO entraram — e cada ausência é um ACHADO, não um adiamento:**

- **Clay = `Flatten` com Offset > 0.** O plano fica *acima* da superfície: os vales sobem até ele, as
  cristas descem até ele. Material adicionado, superfície achatada — **isso É clay**. Os dois knobs já
  estão na tela. Um chip pra isso seria **preset de outro chip**, e um card não sabe te dizer qual de duas
  ferramentas idênticas você está segurando.
- **Clay Strips = Clay + dab quadrado.** A forma do dab é do **PINCEL** (10 falloffs, slot de Shape,
  flatten, ângulo). Um falloff quadrado é um buraco do pincel, não um verbo do sculpt.
- **Draw Sharp COLAPSA no Layer.** O Blender precisa dele como pincel separado porque o Draw dele lê a
  malha *deformada* e arredonda a própria crista; **nosso motor por-traço lê o `pre` CONGELADO e não
  consegue fazer diferente** (§4). Todo verbo aditivo aqui já é "sharp" por construção — não sobra nada
  pro segundo ser.

**Famílias e custo:**

| família | verbos | alvo | knobs | custo |
|---|---|---|---|---|
| Smooth | Smooth · Sharpen | `blur(pre)` — memo, reconstruível | Radius | 12 B/px |
| Plane | Flatten · Scrape · Fill · **Chisel** | `Σw·plano(i)` — função da LISTA DE DABS | Offset (+ **Angle** no Chisel) | 12 B/px |
| **Height** | **Layer** · **Inflate** | função de `pre` e um knob — **sem buffer** | Depth | **8 B/px** |

### ⚠️ A LIÇÃO DE W3 (vale pra linha inteira, e o gate cobrou)

**Os dois eixos de um campo de altura NÃO são a mesma unidade.** `x` é texel; `h` é *carga de tinta*. Um
**ângulo** é razão de COMPRIMENTOS, e uma **normal** também — então qualquer grandeza *geométrica* só
significa alguma coisa depois que a altura vira comprimento. O conversor é o que a **LUZ** usa:
`DEPTH_UNIT_PX = 16` (uma carga de tinta tem 16 px de altura).

~~Eu acertei no **Inflate** porque fui procurar qual normal a luz usa.~~ **Não acertei** — fui buscar a
normal certa e a apliquei **invertida** (`·n_z` onde era `/n_z`), e ainda assim todo gate ficou verde,
porque o gate se chamava *"inflate arredonda a crista"* e arredondar a crista **É** o bug. Um gate prova
que o código faz o que você **disse**; nada nele te avisa que o que você disse está errado. **Errei no
Chisel** pelo motivo oposto — não fui buscar nada: o
1º corte usou `tan(36°)` cru, que inclina o plano em **0,73 load por texel** — 8,7 loads ao longo do
footprint, **4× o teto de vidro**. O "ângulo" era um número num espaço sem geometria dentro, e o V que ele
cortava era um penhasco. O gate `the_chisel_carves_a_crease` pegou (0,36 load "poupado" *no próprio eixo*,
que é meio texel de lado — o número denunciou a escala).

**Regra:** *toda grandeza geométrica — normal, ângulo, inclinação que o artista VÊ — cruza
`DEPTH_UNIT_PX` na entrada.* As duas mutações estão gateadas (M4 e M7), porque uma lição só gateada onde
você já sabia olhar não está gateada.

### W4 e W5 (FECHADAS — 2026-07-15/16)

### W4 — A família ADVECTIVA: Grab · Pinch · Nudge · Rotate · Thumb
**Não construir um motor novo.** Fazer o motor do Deform **carregar os planos do relevo** (`h`,
`covers`, `mats`) junto do RGBA (§5, exceção). Isso destrava **cinco pincéis de uma vez** e unifica
"Liquify" e "sculpt-warp" numa engine só.
Decisão de superfície a tomar em W4: os warps de relevo são **sub-modos do Sculpt** ou um **toggle
"afeta o relevo" no Deform**? Recomendação: o segundo — um motor, um lugar.

### W5 ✅ **FECHADA** — Conserve (a *bow wave*, §6) + o filtro de camada inteira

**Conserve** landou 2026-07-15 (`b9d0ef28`) e foi **smokado OK** pelo Enio (2026-07-16, depois do fix da
âncora do aro mover o desenho dele pra dentro).

**Filtro** landou 2026-07-16 (`57d9881e` + `ea0a5c02`): **dois botões** no card — **Filter Layer** e
**Filter Stroke** — aplicam o verbo SELECIONADO sem traço nenhum, na **Strength do pincel**, honrando a
Selection, em 1 passo de undo.

**Os 2 escopos são UM fator por texel** (`amount = strength × selection × envelope`), não duas features:
`Layer` é o caso degenerado onde o envelope é 1. **Filter Stroke** escopa ao **último traço**, mascarado
pelo **envelope de tinta dele** (`relief.live_paint`) — o registro que o Painter JÁ mantém, porque é dele
que o card do Body re-deriva o traço depois do pen-up. É a resposta honesta pra *"onde o último traço
passou"*, e carrega a **borda macia do próprio falloff**: o filtro feather exatamente onde a tinta feather.
(Um bbox era a máscara óbvia e estaria errada 2×: borda dura, e retangular.) Oferecido só quando existe
último traço **nesta camada** — um botão que só pode recusar é um botão que mente.

**Não há kernel novo, e isso é o desenho inteiro.** O render já é `h = pre + k·Δ(verbo)` lendo o `amount`
pro `k`; um traço preenche `amount` andando dabs, e **não contribui mais nada que um filtro precise**.
Então o filtro preenche `amount` DIRETO (uniforme) e chama o MESMO `render_sculpt` — §10.1 respeitado ao
pé da letra (*"um passe com geometria própria é como nasce 'Tiling não funciona no Sculpt' daqui a seis
meses"*). Tudo que o traço comprou vem junto de graça: o memo (Smooth/Sharpen), a bola + advecção de
matéria (Inflate), a idempotência do `pre` congelado, o restore dos 4 planos, o teto morando na luz.

**Zero knob novo:** os chips dizem QUAL verbo, o knob do verbo diz QUANTO, e a **Strength do pincel É o
`k`** — que é literalmente *"quão longe ao longo do trajeto vamos"* (o comentário do render já dizia).
Um slider próprio seria um 2º modelo de um número que o card já mostra (§2: *"o Strength do pincel é o do
sculpt"*).

**Recusa os verbos de PLANO** (Flatten/Scrape/Fill/Chisel) — e a recusa é a resposta honesta, não uma
limitação: o alvo deles é um plano **ajustado por mínimos quadrados à PEGADA do dab**, e uma camada não
tem pegada. *"O plano da camada inteira"* é **outra operação** (achatar a arte no plano médio dela) — um
**verbo a projetar**, não um flag a virar. O Chisel é recusado 2×: o V dobra no **eixo do traço**, e um
filtro não tem traço (W3 pagou pela regra de que o eixo nunca é um ajuste do pincel). Porta única
`SculptMode::filters_layer()`: o **painel** pergunta pra OFERECER o botão, o **tool** pergunta pra HONRAR
o clique — botão dimmed que despacha é mentira, e duas cópias da lista de verbos divergem.

**⚠️ `Layer` foi CORTADO — o Enio pegou um knob morto meu** (2026-07-16). Eu o incluí raciocinando que
*"cai de graça: o alvo dele é a constante `pre + Depth`"*. Cai — num knob que não faz **nada**: filtrar a
camada com Layer soma `k·Depth` em TODO texel = **translação uniforme** do campo, e **a luz lê `∇h`**, que
uma constante não muda. Não moveria um pixel. Escopado ao traço ele varia (por `live_paint`), e aí
**duplica o slider Depth**, que já re-deriva esse mesmo traço do mesmo plano. Invisível ou redundante —
nunca uma ferramenta. É a MESMA espécie que este plano matou no `DepthSource::Shape` (*"um knob que não faz
nada"*), e eu repeti; a lição é que *"cai de graça"* é como um knob morto entra.

**⚠️ `Relax` foi CORTADO, com motivo** (o 4º nome que esta linha da W5 citava). Relaxar é **redistribuir
VÉRTICES** preservando a forma; num campo de altura a grade é fixa e não há distribuição pra consertar —
então Relax **colapsa em Smooth**, e seria um knob morto (a mesma espécie que matou o `DepthSource::Shape`
e os chips de Clay/Clay Strips/Draw Sharp). O **§9 já recusava Slide Relax pela mesma razão**: precisa de
topologia que um campo de altura não tem. A lista honesta da W5b é **Smooth · Sharpen · Inflate** — os três que
**RESHAPE**.

**Um bug REAL que o gate novo pegou:** `live_stroke_envelope` dizia SIM num tool sem canvas — `n = 0`, o
`live_paint` vazio "casa" com 0, e `live_relief_layer == layers.active()` porque os DOIS são `None`. O botão
aparecia numa tela virgem. É a lei da fixture pelo avesso: **zero não falha, a menos que você faça falhar.**

**Gates:** 9 no tool (`sculpt_tests/filter.rs` — alcança a camada toda incl. os cantos onde pincel nenhum
foi · recusa os de pegada e não move NADA · a Selection é a borda, fora dela byte-idêntico · Strength = `k`
(metade = metade do trajeto, medido 0,5±0,02) · 1 undo devolve o relevo · camada sem relevo recusa em vez
de inventar) + **2 de seam que CLICAM o botão de verdade**. 5/5 mutações sangram. Sondas de render: cenas
7/8 do `push_look_probe`.

**O sweep do seam perdeu a premissa *"o Chisel é o card mais cheio"*** — a W5b a matou (o Chisel tem
Rake+Conserve, o Smooth tem Filter Layer, nenhum tem o do outro). Ele agora pergunta a **CADA verbo** e
exige que **ALGUM** card pinte o widget: a propriedade que importa, sem tabela id→verbo escrita à mão pra
driftar.

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
3. ~~**Teto:** `H_CEIL` continua valendo — o sculpt não pode estourá-lo.~~ **REVISTO 2026-07-14 (smoke do Enio: *"em 3 pinceladas toda escultura é achatada no teto"*).** O teto **não é um clamp** — é uma **compressão assintótica na LUZ** (`impasto::soft_ceiling`). O clamp duro não era vidro, era **borracha**: mapeava tudo acima do teto no MESMO valor ⇒ as marcas do topo eram **apagadas**, o platô ficava com **gradiente zero**, e a luz renderizava a área mais trabalhada como **chapa morta**. Agora o **buffer guarda o relevo verdadeiro** (o sculpt ajusta planos e rola bolas sobre geometria honesta) e só a **aparência** topa. **REVISTO 2× no mesmo dia:** primeiro `H_KNEE=2`/assíntota `8`; depois (2º smoke: *"em 3 pinceladas achata no teto … subir na proporção real do peso"*) **`H_KNEE=24` / assíntota `128`** — a faixa alcançável é **LINEAR** (sobe na proporção do peso) e a compressão vira **guarda far-field** (só pra a luz nunca ver inclinação infinita). Junto: **a altura do depósito escala com o raio** (`derive_height × radius/10`), senão um pincel grande picava chato e estourava o joelho de 2 num dab.
4. **Undo:** uma entrada por traço; a sessão (`pre` + `amount`) entra no `ModelSnapshot`.
   *(Precedente fresco: o `mats` **ficou de fora** do snapshot quando o material landou, e o buraco se
   escondia — na tela vazia a cobertura zera e a luz pesa o material obsoleto por zero. Só apareceu em
   tinta-sobre-tinta. **Ao adicionar um plano, adicione-o ao snapshot no mesmo commit.**)*
5. **A luz não muda.** O sculpt mexe em `h`; o passe de luz (doc 16 §14-18) já sabe o que fazer com ele.

# HANDOFF — a borda do Inflate (`line/Painter`, 2026-07-16)

> ## ⚠️ LEIA ISTO PRIMEIRO — o 2º smoke do Enio REPROVOU a borda de novo, e o fix abaixo NÃO a alcança
>
> **Medido, não teorizado:** na cena do Enio o render é **byte-IDÊNTICO** com e sem o fix desta linha.
> O que sobrou **não é a matéria — é a ALTURA**, e é PRÉ-EXISTENTE:
>
> ```text
> depósito              maior salto entre vizinhos  0,38 load     0 texels acima de 1 load
> Blob (taper LIGADO)   maior salto                 1,39-1,69     188-444 texels acima de 1
> Blob (taper DESLIG.)  maior salto                 4,91          cobertura volta ao penhasco 255
> ```
>
> **A luz lê a DERIVADA: um degrau de 1,4 load entre 2 texels É uma parede.** Os degraus e as fissuras
> brancas da foto são essa parede, iluminada. Duas coisas que a medição REFUTOU (não re-derive):
> * **não é espessura** — blob de 4 a 40 loads faz a mesma rampa (passo 62-88);
> * **não é o taper** — desligá-lo TRIPLICA o salto (4,91) e devolve o penhasco de 255. O taper é a
>   **mitigação**; sem ele o cap de alcance é um precipício sem nada que o suavize.
>
> **O residual é a COSTURA do argmax:** atravessando-a o envelope é contínuo (é o que um lower envelope
> É), mas a **distância ao vencedor** não é — e o taper é função dela. Uma forma sozinha já faceta (444
> texels): não precisa de junção. Detalhe + instrumentos: `sculpt_tests/inflate_edge_probes.rs`,
> `PH2D_PUSH_LOOK_DIR=<dir> ... probe_push_render_and_look -- --ignored` (cena **11** = o arranjo da foto).
>
> **Isto é decisão de ARQUITETURA no kernel, não um fix de carona** — §8 abaixo.
>
> **Estado do que ESTE handoff fechou (a cobertura): corrigido, gateado, medido e OLHADO. Pendente smoke.**
> A linha está commitada e verde. **NÃO integrei, NÃO pushei, NÃO rodei ship** (§0.7).
>
> Este arquivo SUBSTITUI a versão anterior (o diagnóstico + direção de fix). O diagnóstico estava
> **correto ao pé da letra** e o fix prescrito (§3.1) era o certo — implementado como prescrito, com
> 3 desvios que a medição obrigou (§4). O §3.2 foi **decidido pela medição**, como o handoff mandava.

## 0. O que mudou (commit `8ea5f91c`)

**A matéria segue o MESMO taper da altura, por porta única.** `sculpt_offset::ball_taper(d2, reach2)` —
os **dois** sítios perguntam a ela (o post-pass da altura e a advecção da matéria).

- **Cobertura** = PRESENÇA → `max`, desvanecida pelo taper (`pre_cover[si] · t`).
- **Material e pigmento** = IDENTIDADE → compõem **`over`** na opacidade que de fato chegou (`t`), pelo
  mesmo operador do depósito (`impasto_live::commit_stroke_height`) e pela porta do próprio pigmento
  (`ph2d_painter_brush::blend_over(Mix)` — straight-alpha, o espaço em que o `canvas_rgba` já é
  guardado e blendado).
- **Em `t = 1` sobre fonte opaca os dois REDUZEM à cópia literal que shipou, bit a bit** — o interior
  que o Enio aprovou não se move.

## 1. Os números (medidos, no caminho real do produto)

Perfil radial de cobertura — blob grosso (12 loads), Filter Layer + Inflate, Depth 1:

```text
depósito (uma borda de tinta REAL)   255, 251, 222, 125,  11, 0      passo máx 114
inflado ANTES                        255, 255, 255, 255, 255, 0      passo máx 255   ← o cortador de biscoito
inflado DEPOIS                       255, 193, 193, 137,  88, 48, 0  passo máx  62   ← uma rampa
```

**A borda crescida ficou mais MACIA que qualquer borda que o pincel consegue depositar** (62 < 114) —
e é esse o oráculo do gate: a referência não é um número inventado, é *o que tinta parece* neste
produto. O Inflate faz crescer tinta; se a borda que ele cria é mais dura que qualquer borda que o
pincel consegue pintar, o que cresceu não é tinta.

**RENDER-AND-LOOK (o método desta linha, não teoria):** sonda `push_look`, **cena 10 = o blob do Enio**
(o handoff pediu; a laje da sonda esconde o fenômeno — borda alinhada aos eixos, o padrão do argmax ao
longo dela é regular e a escada não tem o que subir). **A escada sumiu** — antes: silhueta em blocos,
serrilhada; depois: aro redondo e macio. Vale para os DOIS (Filter Layer e pincel Inflate — mesmo
`render_inflate`). O slab também melhorou visivelmente (delta máx 107).

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_PUSH_LOOK_DIR=/tmp/look cargo test -p ph2d-host-desktop probe_push_render_and_look -- --ignored
# 10a_blob_before · 10b_blob_inflated (filtro) · 10c_blob_inflate_brush (pincel)
```

**Perf:** INFLATE **3,30 ms/move @2048² · 3,79 @4096²** (alvo ≤4, kill 8). Era 3,36/3,73 — de graça.

## 2. Smoke do Enio (o que decide)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
Tinta grossa → **SCULP** → **Inflate** → **Filter Layer**. O certo: a forma engorda com a borda **macia
e redonda**, sem serrilhado. **Re-smoke declarado do Inflate por-PINCEL** (compartilha o kernel).

⚠️ **O que este fix NÃO faz:** a borda continua **imune ao Smooth**. O §5 do plano 18 (*"o sculpt escreve
`h` e SÓ `h`"*) segue valendo para os outros 7 verbos, e **nenhum verbo consegue EDITAR `covers`** — só o
Inflate escreve. A metade da queixa que dizia *"nada pode corrigi-la"* é **estrutural e continua de pé**;
o que sumiu foi a irregularidade que dava vontade de corrigir. Ver §3.

## 3. O §3.2 (borda editável) — DECIDIDO PELA MEDIÇÃO, como o handoff mandava

> *"O 3.1 pode DISSOLVER o sintoma… **Meça primeiro:** se depois do 3.1 a borda ficar boa, o 3.2 vira uma
> capacidade a nomear, não uma urgência — e o Enio decide."*

**Medido: a borda ficou boa.** ⇒ **O 3.2 NÃO foi construído**, e a recomendação é que continue assim até
o Enio pedir. Motivos, na ordem em que pesam:

1. **O sintoma sumiu.** Um Smooth que come a borda de uma pincelada que o artista pintou à mão é um bug
   PIOR que o serrilhado (o próprio handoff anterior já dizia isto), e agora não há prêmio para pagar
   esse preço.
2. **Não é um fix, é um ADR.** Exige a porta simétrica (`edits_matter()`), decidir se borra a ARTE ou só
   a franja que o Inflate fabricou, e o que "editar cobertura" significa no undo/restore dos 4 planos.
3. Se o Enio ainda quiser a capacidade, ela deve nascer **nomeada** (um verbo/knob que o artista escolhe),
   nunca como efeito colateral do Smooth.

## 4. Os 3 desvios da prescrição (a medição obrigou) — LEIA, é onde estão as lições

### 4.1 — O `over` compõe sobre o plano CONGELADO, nunca sobre o pixel vivo

O caminho freehand re-renderiza o traço INTEIRO a partir do `pre` **a cada batch de pointer-move, SEM
restore** (`sculpt_session.rs:349`). Isso só é sadio porque toda escrita da advecção é **ASSIGNMENT
sobre estado congelado** — um render é uma resposta nova a *"o que este traço, no `amount` de agora, faz
com a tela em que começou"*, exatamente como a altura (`target[gi] = f(pre, amount)`).

`over` no pixel VIVO tomaria uma demão por batch ⇒ **a borda escureceria com a lentidão da mão**. É a
acumulação sequencial que esta linha já pagou duas vezes ([[feedback_a_sequential_accumulation_is_sampling_dependent]]).
**Medido:** com destino vivo o desacordo incremental-vs-refresh vai de **6 → 2520 bytes**.

O guard também foi para o congelado (`v <= pre_cover[gi]`), pela mesma razão — e foi ELE que derrubou os
2520 de volta a 6.

### 4.2 — GHOST PRÉ-EXISTENTE (não é meu; medido no kernel shipado `f8902dfc`)

`a_knob_touched_mid_stroke_does_not_move_the_picture` nasceu **VERMELHO no código correto**. Investigado:
o caminho incremental e o mesmo traço re-renderizado uma vez discordam em **2 texels a 4/255** — e
**discordavam ANTES deste fix também** (6 bytes no kernel shipado). São (103,78) e (103,141) no fixture,
simétricos, na borda do próprio pincel: um render antigo advectou um sopro ali, um posterior não advecta
mais, e **a advecção não sabe DES-pintar** (escreve onde a bola entrega e `continue` no resto).

- **Fecha assim:** tornar o render TOTAL — atribuir cada texel de `kr`, restaurando os planos congelados
  onde a bola não entrega nada. É a forma honesta ("função pura de (congelado, amount)").
- **Por que não aqui:** escreve a janela inteira a cada pointer-move ⇒ é uma **decisão de perf** contra o
  kill criterion, não um drive-by dentro de um fix de borda. **Fica ABERTO** (§6).
- **O que o gate faz enquanto isso:** guarda o que ESTE commit é responsável por — *o taper não pode fazer
  o histórico de render importar mais do que já importava*. As constantes `GHOST_TEXELS`/`GHOST_DEPTH`
  são o orçamento do defeito herdado, **nunca licença para aumentá-lo**.

### 4.3 — LOC cap: `sculpt_blur.rs` 736/700 → split `sculpt_inflate.rs` (399 + 363)

O gate mora na `ph2d-editor-core` e **não roda com `cargo test -p ph2d-tool-painter`** — exatamente o
aviso do handoff anterior. Split, nunca allowlist ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]),
e na costura que o código **já defendia**: o Blob **não é um blur e nunca foi** (raio na `amount` VIVA ⇒
não é constante do traço ⇒ não memoizável; e é o ÚNICO verbo que move matéria). O motor dele já morava
ao lado (`sculpt_offset.rs`); agora o render mora junto.

## 5. Os gates (`sculpt_tests/inflate_edge.rs`, 7) — e o que eles ensinaram

Fixture = **o repro do Enio**: blob de relevo alto (12 loads) e borda **CURVA**, pelo depósito REAL.

| gate | a lei |
|---|---|
| `the_inflated_rim_is_feathered_not_cut` | a borda crescida não é mais dura que a que o depósito lava (oráculo = o produto) |
| `the_matter_fades_where_the_ball_fades` | a transição TEM texels parciais; o último sopro é sopro (255 → 48) |
| `the_grown_rim_wears_the_paints_material_and_fades_with_it` | material e cobertura são **o mesmo `255·t`** — um taper, dois planos |
| `the_inflate_does_not_repaint_the_forms_interior` | byte-identidade do miolo (o `over` não lava o aprovado) |
| `a_faster_mouse_does_not_grow_a_different_rim` | a metade da MATÉRIA da lei que `a_faster_mouse_does_not_sculpt_deeper` já diz da altura |
| `a_knob_touched_mid_stroke_does_not_move_the_picture` | o traço é o que o `amount` diz, não o que o histórico diz |
| `diag_inflate_edge_profile` (`#[ignore]`) | o instrumento: imprime os perfis acima |

**Mutações 6/7 matam.** A que sobrevive (`a8 = (t*255.0) as u32`, arredondar p/ baixo) mexe ≤1/255 no
material NA FRANJA, onde a cobertura é ~0 e **a luz pesa material POR cobertura**: está abaixo da
resolução do dado e de qualquer oráculo de aparência. Gate para ela seria modelar a ARITMÉTICA em vez da
figura — o que a disciplina de oráculo desta linha proíbe. Documentado no gate, não gateado.

### ⚠️ 4 armadilhas que quase passaram (as lições reais desta jornada)

1. **EU ESCREVI UMA EXPLICAÇÃO CONFIANTE E ERRADA num doc comment — e o repo já tinha o gate que a
   desmentia.** Afirmei que *"o mesmo caminho, 2 amostragens, é a mesma figura"* seria **falso p/ o sculpt
   por design**, porque `amount[i] += w` é SOMA e "demorar esculpe mais". **Errado:** o motor espaça dabs
   por **DISTÂNCIA** ⇒ a lista de dabs é IDÊNTICA em qualquer taxa de polling; só o *batching* muda. A
   soma é sobre dabs SOBREPOSTOS ao longo do caminho, não sobre a taxa do mouse. O
   `a_faster_mouse_does_not_sculpt_deeper` (sculpt_tests.rs:200) já dizia isso, com todas as letras, para
   a ALTURA. Corrigido: escrevi o irmão dele para a **MATÉRIA**
   (`a_faster_mouse_does_not_grow_a_different_rim`) — mutação do pixel vivo o derruba com **45/255** de
   diferença entre um mouse lento e um rápido. **Antes de declarar que o design rejeita um invariante,
   grepe: pode existir um gate afirmando o contrário.**
2. **Dois outros gates ÓBVIOS ficaram VERDES contra código quebrado.**
   * *"renderizar o mesmo estado 2× pinta 1×"* — o guard pula o 2º render ⇒ idempotente **qualquer que
     seja** o destino do composite.
   * o mesmo drive em `strength = 0.25` — **ZERO advecção acontece**: a bola não bate o próprio piso
     (`own = |Depth|·amount`), `sbuf` zera, o laço `continue` em tudo. O gate mediu NADA e passou.
   A divergência precisa do `amount` **CRESCER** entre 2 renders do mesmo texel: em **0.4** são 4090
   composites sobre 1346 texels (**2744 re-composites**). **Instrumentei o produto e CONTEI** — 3 palpites
   erraram antes. *O fixture tem de conter o fenômeno*, pela quarta vez nesta linha.
3. **O MATERIAL não tinha gate nenhum.** Nada lia `mats` depois de um sculpt; um mutante sobrevivente
   denunciou. Agora a lei é uma IGUALDADE (`mat == cov`, ambos `255·t`), que é o que a torna afiada.
4. **O fixture do blob na SONDA não reproduzia** (byte-idêntico antes/depois) enquanto o unit test
   reproduzia. Causa: sem `set_brush_impasto_depth(1.0)` e no tamanho de pincel errado, a borda do
   RELEVO fica bem dentro da borda da TINTA ⇒ a bola cresce sobre tela já pintada ⇒ o guard pula ⇒ o
   taper nunca se aplica. Instrumentar (`matter_ok`, `touched`, `pre_peak`) resolveu em 1 tentativa.

## 8. ⚠️ A BORDA AINDA ESTÁ QUEBRADA — o achado do 2º smoke (o item nº 1 da fila)

**O que este handoff fechou:** a **matéria** (a cobertura) parava seca — cortador de biscoito. Fechado,
medido (255 → 62), olhado.

**O que sobrou, e o Enio viu na hora:** a **ALTURA** do Blob é descontínua. Números no topo deste arquivo.
O depósito nunca passa de 0,38 load entre vizinhos; o Blob salta **1,39-1,69**, em 188-444 texels.

**A mecânica, medida:** o envelope cru (`blob_dilate`) é contínuo — é a definição de lower envelope. O
**cap de alcance** (`d² ≥ reach2_s`) é um precipício, e o **taper** existe para suavizá-lo (por isso
desligá-lo piora 3×). Mas o taper é `t(d²)` — função da **distância ao vencedor** — e o vencedor é um
**argmax DISCRETO**: atravessando uma costura de célula, `hbuf` é contínuo mas `d` **salta**, então `t`
salta, e `lifted = p0 + t·(hbuf − p0)` herda o salto. **O taper quebra a continuidade que o envelope
garantia.**

**O Enio pediu "subdivisões antes de inflar".** É a analogia de malha (subdividir antes do Inflate do
Blender). Num campo de altura não há topologia para subdividir — a "resolução" é o canvas (1024² no smoke,
verificado: **não** é o caso do canvas 64px). Traduzido para cá, o pedido dele é: *o offset tem de ser
computado numa representação que não grude na grade do argmax*.

**As 3 saídas (nenhuma é drive-by — o Enio decide):**
1. **O envelope maximiza o objetivo TAPERADO.** Correto por construção (um max de contínuas é contínuo),
   mata a costura na raiz — e **não é separável**: perde-se Felzenszwalb e volta-se ao `O(área·ρ²)` que
   custou **73 ms/move** e foi rejeitado. Precisaria de outra estrutura (pirâmide? jump-flood?).
2. **Suavizar o campo tapered depois** — o knob **Smooth** do Inflate já faz isso e é `0` por default. Se
   um Smooth ≥ 2 px dissolve os degraus, a saída barata é **mudar o default** (e o Enio decide o número
   olhando). ⚠️ **MEÇA antes de propor**: é a única das três que cabe numa sessão.
3. **Taper por uma grandeza CONTÍNUA na costura** em vez da distância. Projeto aberto: `hbuf` é contínuo,
   `d` não; achar o substituto é a pesquisa.

**Regra desta linha antes de tentar:** *bateu na 2ª reconstrução de topologia → PARE e prove o modelo*
(DIRETIVA §5, two-strikes). O Blob já foi reconstruído 2× (raio-no-falloff; a bola que move matéria).
Uma 3ª exige o modelo provado ANTES do código.

## 6. Aberto depois disto (a fila do impasto)

0. **A BORDA (§8) — o item nº 1**, e é decisão do Enio entre as 3 saídas.
1. **O ghost pré-existente do §4.2** — render TOTAL da advecção (des-pintar). É decisão de PERF: mede
   contra o kill 8 antes.
2. **§3.2 — a borda editável**: só por ordem do Enio, e como capacidade NOMEADA (ADR), não como efeito
   colateral do Smooth.
3. **Passe de luz na GPU** — a luz é CPU e, enquanto for, **relevo visível DESLIGA o compositor GPU
   inteiro** (`painter_gpu_preview::gpu_eligible` → `None` se `impasto_visible()`). Exige reconciliação
   bit-a-bit contra a CPU (doc 16 §6).
4. **Relevo do PAPEL** — acopla impasto↔aquarela: **exige ordem nova do Enio** (§2 do doc 16 é barreira).
5. Dar ao **BANCO** do Push a cura que a mordida ganhou (residual medido 0,0286 — invisível hoje).
6. Conserve p/ Flatten/Fill (design) · perf do Deform não é gateada · knob de `forward_share`?

## 7. Estado da linha (tudo commitado, verde)

| | |
|---|---|
| `10bd2a10` | **REGRESSÃO minha, corrigida** — o gizmo tem 2 esquemas de id (canônico + **keyed**) e a porta só conhecia um ⇒ o pincel virava gesto de mover. **smoke OK** |
| `a07b30ca` | sonda do knob Smooth — **2 px já zera os degraus** (188 → 0 texels), forma intacta; perf +35% |
| `6f4c84ea` · `611f4f22` | sondas da borda quebrada (a altura salta 1,39 vs 0,38 do depósito) + split gates/sondas |
| `518c91a5` | **o clique num botão é do botão** — a porta única do chrome (4 braços pediam meia pergunta) — **smoke OK** |
| `d6616168` | o irmão da MATÉRIA p/ "a faster mouse does not sculpt deeper" (+ correção de um doc comment meu que negava um gate existente) |
| `8ea5f91c` | **a matéria segue o taper** (+ porta única `ball_taper`, split `sculpt_inflate.rs`, 6 gates, cena 10 da sonda) |
| `f8902dfc` | o handoff do diagnóstico (substituído por este) |
| `ea0a5c02` · `57d9881e` | W5b — filtro de camada + 2 escopos |
| `2e1806fb` · `fd77f9c5` | a mordida é função do caminho · âncora do aro no corpo — **smoke OK** |

Gates: tool **711** · clippy **0** · LOC cap **verde** · `check --workspace --all-targets` verde ·
perf INFLATE 3,30/3,79 (kill 8). Mutações da jornada: **6/7** (a 7ª é sub-LSB, §5).

## 9. Para o INTEGRADOR (DIRETRIZ §1.5.9) — o que esta linha criou

⚠️ **Esta linha TOCOU FOUNDATIONAL** (`ph2d-editor-core`) — projetado para isolamento (ADR-0107): é uma
**fn irmã append-only no módulo que já é dono dos ids**, nada removido, nada renomeado.

| onde | o quê | risco de colisão |
|---|---|---|
| **`ph2d-editor-core`** `gizmo/hit.rs` | **`is_gizmo_id(id)`** (fn `pub` nova, ao lado de `is_gizmo_handle_id`) + re-export em `gizmo/mod.rs` e `lib.rs` | baixo — nome novo; a linha que também exportar de `gizmo` colide **textualmente** nas 2 linhas de `pub use` |
| **shell** | módulo NOVO `shells/desktop/src/chrome_hit.rs` + `mod chrome_hit;` em `main.rs` | o `mod` em `main.rs` é o único ponto compartilhado |
| **shell** | `forwarding.rs` **encolheu** 614 → 422 (a porta saiu para `chrome_hit.rs`) | quem editar `forwarding.rs` na mesma jornada conflita — **avisar** |
| **shell** | 4 braços passaram a chamar `chrome_hit::pointer_over_chrome` (`painter_canvas_input`, `eyedropper`, `protect_brush` ×2) | baixo |
| `ph2d-tool-painter` | `sculpt_offset::ball_taper` (`pub(super)`), módulos novos `paint::sculpt_inflate`, `sculpt_tests::inflate_edge{,_probes}` | baixo — só a linha Painter |
| env novos | `PH2D_CHROME_DIAG=1` (quem recusou o canvas) · `PH2D_DIAG_SMOOTH=<0..1>` (sonda) | nenhum |
| gates novos | `shells/desktop/tests/the_chrome_swallows_the_click_it_was_given.rs` (4) | nenhum |

**Nenhum contrato congelado tocado** (`Tool`/`CanvasPaintTool`/`NodeOp` intactos); nenhum id de UI, i18n
ou token novo. **`smooth_norm` default segue 0** — a mudança está RECOMENDADA (§8) e é decisão do Enio.

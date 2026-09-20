# 40 — A costura dura do retorno sobre o próprio traço (Watercolor, Charge < 1): análise (2026-09-20)

> **Pergunta do Enio (com foto):** *"Paint Mode: Watercolor, com Charge < 1 temos o efeito de perder a
> carga no decorrer do traço e é muito bom. Contudo, ao traçar voltando sobre o próprio traço (sem mouse
> up), o pincel lava parte da borda escura que é a responsável pelo belo aspecto da borda suave mas deixa
> por baixo uma borda plana dura pixelada. Mesmo se aumento ao máximo Rewet e Smudge, não consigo me
> livrar dessa borda. Curiosamente se eu finalizar o traço e pintar sobre essa borda com Rewet e Smudge
> altos, consigo resolvê-la."* — e as duas encomendas: **(1)** a borda ficar menos dura mesmo com Rewet e
> Smudge em 0; **(2)** Rewet e Smudge funcionarem sobre a borda lavada pelo próprio traço.
>
> ⚠️ **ESTADO: ANÁLISE. Zero linha de código tocada, por ordem** (*"não mude o código de nada"* — seis
> linhas acabavam de entrar no `main` e a CI ainda ia correr). Este arquivo nasceu **sem commit** na árvore
> primária e **fora do `00_INDEX.md`** (que é rastreado): quem abrir a linha que implementa o commita e
> acrescenta a linha no índice.
>
> ⚠️ **Tudo aqui saiu de LER o código, não de correr o produto.** A aritmética do §2.3 é derivada das leis
> escritas no fonte — ela não é medição. O §8 lista o que ficou por medir, e o §7 diz qual é o instrumento.
> A única coisa CORRIDA é o banco do §9 (o custo do item 4) — um **sucedâneo declarado**, fora do repo,
> calibrado a −15 % contra a sonda do produto.

## 0. Resumo em cinco frases

1. **A borda escura some por desenho, e está certo:** o aro é derivado da silhueta da **união** da lavagem
   (*um wash, um aro*); quando a volta cobre o flanco da ida, aquele flanco deixa de ser fronteira e o aro
   migra para o contorno novo.
2. **O que fica por baixo é o degrau do mapa de reserva de pigmento** (`stroke_deplete`): ele é composto por
   **máximo**, e o máximo **comprime** a rampa de 15 % do raio que existe na borda de cada dab — a transição
   visível entre *pincel cheio* e *pincel quase vazio* mede `0,15·r·(1 − v₂/v₁)`, ou seja **~6 px num
   pincel de raio 45 e 1–3 px num pincel pequeno**.
3. **Ele é "pixelado" porque esse mapa `u8` é lido por vizinho-mais-próximo na coordenada DEFORMADA pelo
   Ragged Edge** (amplitude de fábrica: 6 px, do tamanho da costura inteira), e o anti-serrilhado só
   enxerga a silhueta da cobertura — uma costura interna não é silhueta.
4. **Rewet, Smudge e o pickup do Charge leem todos uma base CONGELADA no pen-down** (a tinta que já estava
   no papel). A tinta do traço em curso mora só nos acumuladores do traço até o pen-up — para os três, ela
   **não existe**. Isso é deliberado (*no self-feeding*), então a cura **não** é "ler o canvas vivo".
5. **A cura recomendada é UM mecanismo com dois botões:** difundir o campo de reserva dentro do corpo
   molhado (um borrão mascarado e normalizado, o molde do `build_wet_field`) com raio-base pequeno a
   Rewet 0 **(resolve 1)** e raio crescendo com `Rewet × Bleed` **(metade do 2)**; o Smudge ganha o gêmeo
   escalar do smear sobre os acumuladores **(outra metade do 2)**.

## 1. O que a foto mostra, lido contra o código

Traço em U: desce pela esquerda (vermelho escuro — a cabeça do traço, reserva cheia), vira embaixo e sobe
pela direita (rosa — a cauda, reserva gasta), com a subida sobrepondo o flanco direito da descida. As setas
amarelas apontam a fronteira **interna** entre o escuro e o claro: dura, em degraus de 1 px, ondulada como
o contorno externo.

Três fatos que o mecanismo abaixo tem de explicar — e explica:

| fato da foto | explicação |
|---|---|
| a costura é **forte em cima e some embaixo**, junto da curva | a razão `v₂/v₁` entre as duas passadas é ~0,04 no topo (cabeça contra cauda) e ~0,8 na curva (passadas vizinhas no arco) — a largura e o contraste do degrau saem dela (§2.3) |
| a costura **ondula igual ao contorno externo** | as duas são lidas na mesma coordenada deformada pelo Ragged Edge (§2.4) |
| o lado de fora da mancha continua **suave e bonito** | a silhueta tem aro + anti-serrilhado; a costura interna não tem nenhum dos dois (§2.2, §2.5) |

## 2. Mecanismo da borda dura

### 2.1 Os quatro acumuladores do traço, e quem os compõe como

Sob `watercolor_render_active()` o depósito por-dab é pulado; o traço acumula planos do tamanho do canvas e
a aparência é **reconstruída opticamente a cada quadro** sobre uma base congelada
([watercolor_render.rs](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs)).

| plano | o que guarda | regra de composição entre dabs | onde |
|---|---|---|---|
| `stroke_coverage` | a **geometria da água** (silhueta) | **máximo** (envelope — "one pass") | [accum:457](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_accum.rs#L457) |
| `stroke_deplete` | a **reserva de pigmento** do pincel no instante do dab (`fresh ∨ carry`) | **máximo**, com rampa nos 15 % externos do raio | [accum:487](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_accum.rs#L487) |
| `stroke_color` | a cor depositada (mixer) | source-over, alfa `wgt·prio·depl` | [accum:600](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_accum.rs#L600) |
| `wet_styles.owner` | qual traço da sessão é dono do texel | recência | [accum:499](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_accum.rs#L499) |

A densidade óptica por texel é `((cw·fill + edge)·gran) × reserva`
([render:454-461](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs#L454)) — a reserva
multiplica corpo **e** aro, *depois* de o aro derivar da cobertura intacta (a lição (a) do MIX-1,
[doc 12 §W-C](12_aquarela_auditoria_pos_f123_padrao_ouro.md)).

### 2.2 "O pincel lava a borda escura" — é o aro da UNIÃO, e está certo

O aro é um unsharp da cobertura endurecida: `edge = gain·(cw − inner)`, `inner = blur(hard)`
([render:382-413](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs#L382),
[watercolor_rim.rs](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_rim.rs)). Ele só existe onde a
cobertura **tem fronteira**. Quando a volta estende a união para a direita, o flanco direito da ida vira
**interior** (`cw = 1`, `inner → 1` ⇒ `edge → 0`) e o aro reaparece no contorno novo — fraco, porque ali a
reserva é a da cauda.

⛔ **Isto não é o defeito, e não deve ser "consertado":** é a lei *um wash, um aro* (EDGE-1), e é física —
enquanto o papel está molhado não existe borda interna para o pigmento migrar. O defeito é **o que sobra
quando o aro sai de cima**.

### 2.3 O que sobra: o degrau do mapa de reserva — e a aritmética dele

A reserva de um dab a `travel` px do início do traço
([mixer:226](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_mixer.rs#L226)):

```text
fresh = (1 − travel/span)²          span = 120 · r · charge/(1 − charge)
mapa(p) = max_i [ v_i · min(1, (1 − dn_i)/0,15) ]          // DEPL_RIM_RAMP = 0,15
```

Numa passada reta o perfil transversal do mapa é `v₁` até `0,85·r` e cai linearmente a `0` em `r`. Quando a
volta (reserva `v₂ < v₁`) cobre essa casca com o **interior** dela, o máximo corta a rampa onde ela cruza
`v₂`:

```text
largura da transição v₁ → v₂  =  0,15 · r · (1 − v₂/v₁)
```

**O máximo COMPRIME a rampa.** A rampa de 15 % foi posta ali (take 3 do MIX-1) para curar o degrau binário
`255 → 0` na borda do disco; contra `v₂ > 0` só sobrevive a fatia de cima dela.

Aritmética derivada das duas leis acima, para o U da foto (pernas de ~11 raios, curva de ~2 raios) —
⚠️ **conta feita sobre as fórmulas do fonte, NÃO medição do produto**:

| raio | Charge | onde | `v₁` (ida) | `v₂` (volta) | `v₂/v₁` | largura da costura |
|---|---|---|---|---|---|---|
| 45 | 0,20 | topo | 1,000 | 0,040 | 0,04 | **6,5 px** |
| 45 | 0,20 | meio | 0,667 | 0,147 | 0,22 | 5,3 px |
| 45 | 0,20 | curva | 0,401 | 0,321 | 0,80 | 1,4 px (e contraste ~nulo) |
| 45 | 0,50 | topo | 1,000 | 0,640 | 0,64 | 2,4 px |
| 24 | 0,20 | topo | 1,000 | 0,040 | 0,04 | **3,5 px** |
| 12 | 0,20 | topo | 1,000 | 0,040 | 0,04 | **1,7 px** |

Perfil `u8` atravessando a costura (raio 45, Charge 0,2, topo): `… 255 255 226 188 151 113 75 37 10 10 …` —
de cheio a quase vazio em **seis texels**, com quina viva nas duas pontas. É a "borda plana dura": plana
porque o aro saiu de cima (§2.2) e sobrou só `cw·fill·reserva`; dura porque a transição tem a largura de um
anti-serrilhado, não de uma aguada.

### 2.4 "Pixelada": `u8` + vizinho-mais-próximo + coordenada deformada

O composite lê o mapa de reserva assim
([render:367-369](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs#L367),
[render:459](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs#L459)):

```rust
let wgx = (rx0 as f32 + sx).clamp(..) as usize;   // (sx, sy) = posição DEFORMADA pelo warp
let wgy = (ry0 as f32 + sy).clamp(..) as usize;
density *= f32::from(depl_buf[wgy * fw + wgx]) / 255.0;   // NEAREST
```

O Ragged Edge de fábrica tem amplitude **6 px**
([spec_default.rs:142](../../crates/ph2d-painter-brush/src/spec_default.rs#L142)) — do tamanho da costura
inteira. Uma rampa de 2–6 texels lida por truncamento numa coordenada que desliza suavemente produz
**escada de 1 px** ao longo de toda a costura. A cobertura não sofre disso: ela é lida **bilinear** de um
campo `f32` (`sample_bilinear`) e ainda passa pelo `aa_coverage`.

⭐ **A mesma doença já foi paga DUAS vezes neste módulo, e a cura de uma delas está a 100 linhas de
distância:**

* a lição (b) do MIX-1 ([doc 12 §W-C](12_aquarela_auditoria_pos_f123_padrao_ouro.md)): *"qualquer mapa
  por-pixel novo lido pelo composite precisa de taper na borda do dab (nearest + warp transforma degrau em
  escada)"* — o taper curou `255 → 0` e **não** `v₁ → v₂`, porque o máximo o comprime (§2.3);
* o `water_at` ([watercolor_field.rs:481](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_field.rs#L481)):
  *"BILINEAR read of the u8 pool: a nearest read left 1-px stairs on the ring's inner edge … (Enio smoke
  2026-07-09)"*.

### 2.5 Por que o *Smooth Edges* não ajuda

O `aa_coverage` decide *o que é uma silhueta* pela variação da **cobertura endurecida**
([watercolor_aa.rs](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_aa.rs)). Na costura interna a
cobertura vale `1` dos dois lados — para o anti-serrilhado não há fronteira nenhuma ali.

## 3. Por que Rewet e Smudge não alcançam a borda — e por que alcançam depois do pen-up

### 3.1 Os três "instrumentos de água" leem uma base CONGELADA

| instrumento | o que ele lê / muta | onde | a tinta do traço em curso está lá? |
|---|---|---|---|
| **Rewet** (lift · dissolve · pool) | campos de **presença** = `base da SESSÃO` contra o fundo | [render:123-135](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs#L123), [field:231-253](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_field.rs#L231) | **não** — em papel virgem `pres = 0` ⇒ `lift = dissolve = pool = 0` |
| **Smudge** (true smear) | arrasta os pixels RGBA da **base** ao longo da cadeia de dabs | [smudge:50-81](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_smudge.rs#L50) | **não** — *"Blank paper smears nothing"* |
| **Pickup do Charge** (mixer) | amostra `watercolor_base` congelada no pen-down | [mixer:103](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_mixer.rs#L103) | **não** — *"NEVER the live canvas"* |

A tinta do traço em curso vive **só** nos acumuladores do §2.1 até o `apply_watercolor(true)` do pen-up
([stroke_lifecycle.rs:397](../../crates/ph2d-tool-painter/src/tool/paint/stroke_lifecycle.rs#L397)).

O único efeito do Rewet sobre a **própria** lavagem é escalar (afinar o miolo, engordar o aro:
`WET_THIN`, `WET_EDGE_BOOST`) — multiplica os dois lados do degrau pelo mesmo fator. **Escala o degrau;
nunca o borra.** É exatamente o relato: *"mesmo no máximo não me livro dela"*.

⚠️ **Isto é deliberado, e a cerca tem dono:** o reservatório que lia o canvas vivo foi construído e
**retirado** (*cadence-bound, self-feeding, perceptualmente fraco* — 2026-07-06,
[doc 11](11_aquarela_avaliacao_padrao_ouro.md)), e a base re-congelada por traço **envenenou a
reprodutibilidade** da sessão (o *retângulo que clareia a poça vizinha*, 2026-07-09). ⇒ a cura do §5 não
pode ser "leia o canvas".

### 3.2 Depois do pen-up há DOIS regimes, e eu não medi em qual o Enio estava

* **Papel já seco** (janela de secagem de fábrica ≈ 10 s,
  [backdrop:263](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_backdrop.rs#L263)) ou sessão
  quebrada: o traço novo congela uma base que **contém a costura assada**. Aí o Rewet a levanta e a
  dissolve num borrão de raio = Bleed, e o Smudge arrasta os pixels dela. **Funcionam de verdade.**
* **Sessão molhada contínua** (`wet_session_continues`,
  [backdrop:281](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_backdrop.rs#L281)): o composite
  continua lendo a base **da sessão** (anterior ao primeiro traço), então Rewet e Smudge **continuam sem
  ver** o traço anterior. O que resolve a costura aí é outra coisa: (i) o pincel novo chega com reserva
  **cheia** e o máximo iguala os dois lados do degrau em `1,0` — a costura some por **re-entintar**, com
  qualquer Rewet; (ii) com Charge < 1 o **pickup** lê a base re-congelada (que já tem o bake) e arrasta cor
  através da costura com alfa emplumado.

⚠️ O comentário do pen-down ainda diz que a base re-congelada alimenta *"the mixer pickup + rewet"*
([stroke_lifecycle.rs:104](../../crates/ph2d-tool-painter/src/tool/paint/stroke_lifecycle.rs#L104)); o
composite diz e faz o contrário para o Rewet (lê a base da sessão). A nota envelheceu — quem abrir a wave
corrige na passagem.

## 4. Encomenda (1) — menos dura com Rewet 0 e Smudge 0

### S1-A · ler o mapa de reserva BILINEAR (cura o "pixelada") — ✅ recomendado, quase grátis

Trocar a leitura truncada por uma bilinear na mesma coordenada deformada. É a cura do `water_at` aplicada
ao irmão que ficou para trás. Sozinha **não** cura o "dura" (6 px continuam 6 px), só tira a escada.
Subsumida pela S1-B se o campo difuso for lido por `sample_bilinear` — mas vale como degrau barato e como
controle da medição.

### S1-B · DIFUNDIR o campo de reserva dentro do corpo molhado (cura o "dura") — ✅ a recomendada

**A física:** dentro de uma mancha molhada, diferença de concentração de pigmento **difunde** — nunca
existe uma linha nítida entre "muito pigmento" e "pouco pigmento" no mesmo corpo d'água. Hoje o modelo tem
difusão para a tinta **velha** (o dissolve do Rewet) e nenhuma para a **própria**.

**A forma:** um campo janela-local, gêmeo exato do `build_wet_field`
([rewet_px:115](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_rewet_px.rs#L115)) — *borrão
mascarado e normalizado*:

```text
F = blur(reserva · m, R) / blur(m, R)        m = 1 onde o texel pertence à lavagem
reserva_px = sample_bilinear(F, sx, sy)      // no lugar do depl_buf[nearest]
```

* a máscara impede a reserva de "vazar" para fora da lavagem (e o zero de fora de entrar na média);
* num trecho onde a reserva é localmente constante, `F` devolve a própria constante — o **fade ao longo do
  traço fica intacto**; o campo só age onde há gradiente forte: **a costura de auto-sobreposição** (e a
  casca de 15 % — ver ⚠️ abaixo);
* degrau de 6 px vira rampa de `2R+1` px. Duas passadas de box (tenda) tiram as quinas das pontas da
  rampa — calibração.

**As cercas que esta solução tem de respeitar (cada uma já tem gate no repo):**

1. **`incremental ≡ full`** (`watercolor_incremental_composite_matches_full_recompose`): o borrão precisa de
   suporte completo sob toda amostra deformada ⇒ **`R ≤ reach`**. Hoje, seco, `reach = core_r`
   ([window.rs:110-117](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render/window.rs#L110)) ⇒
   ou `R ≤ core_r`, ou o `reach` sobe para `max(core_any, R)` **quando o mapa existe** — e o máximo entra no
   `session_maxima`, senão um dono com raio maior re-renderiza cortado.
2. **Não-contato entre companheiros de sessão** (`watercolor_session_rerender_reproduces_the_bake_byte_exact`
   e `watercolor_session_brush_changes_do_not_touch_baked_washes`, gap de 10 px): um box não é geodésico. Ou
   `R ≤ 8` com máscara "qualquer dono" (o pacto ratificado do `WET_FIELD_BLUR_PX`), ou — se `R` for maior —
   **máscara POR DONO**, no molde do `inner_blur_set`
   ([rewet_px:297](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_rewet_px.rs#L297)): um par de
   borrões por dono distinto presente na janela (normalmente um).
3. **Reprodutibilidade do bake:** o raio de cada dono tem de sair da **tabela de estilos capturada no
   pen-down** (`WetStrokeStyle`), nunca do pincel vivo — senão mexer no slider re-renderiza a poça assada.
4. **Charge = 1 byte-idêntico:** sem mapa, o campo nem é construído. Por construção.
5. **Independência da taxa de polling** (`measure_whether_the_wash_depends_on_the_polling_rate`): o campo é
   função pura dos acumuladores ⇒ herda a propriedade. Por construção.

⚠️ **O que NÃO é byte-idêntico, e o dono tem de ver:** um traço com Charge < 1 **sem** sobreposição muda um
pouco na casca externa, porque a rampa de 15 % também é gradiente e o campo a alisa. Sinal esperado: o aro
de um traço desbotado fica **um pouco mais pigmentado** (hoje a rampa come o aro entre `dn = 0,85` e `1`).
Tem de ser **medido** e aprovado no smoke — não prometido.

**Custo:** +2 box blurs da janela de leitura por composite (por dono), só com Charge < 1. Ordem de grandeza
medida neste módulo: ~0,8 ms por blur no cenário pesado (raio 250 @ 4096², [doc 32](32_aquarela_o_que_custa_hoje.md));
o blur já está no piso de largura de banda. Instrumento: `measure_watercolor_cost.rs`.

**O raio-base `R₀`** é parâmetro de LOOK, não de recurso ⇒ sai de um smoke em escada (três valores lado a
lado), com a barra do §7. Ponto de partida: a escala que o aro já usa (`core_r = min(Bleed, r/2)`), que
cabe no `reach` sem mexer na janela.

### S1-C · alargar a rampa do splat (`DEPL_RIM_RAMP 0,15 → 0,4`) — ⛔ recusada, com a geometria ao lado

Parece a cura de uma linha, e cobra no lugar errado. A silhueta endurecida vive entre `dn ≈ 0,75`
(`cov = SS1`) e `dn ≈ 0,95` (`cov = SS0`) — **é onde o aro mora**. A rampa atual já entra nessa faixa (vale
`1` até `0,85` e `0,33` em `0,95`); alargá-la para `0,4` a faria valer `0,62` em `0,75` e `0,12` em `0,95`
⇒ **tira pigmento do aro de TODO traço com Charge < 1**, sobreposto ou não — justamente *"a borda escura
responsável pelo belo aspecto"*. E continua comprimida pelo máximo (a costura só iria de 6 para ~17 px no
melhor caso, a pagar o aro inteiro).

### S1-D · compor a reserva por MÉDIA ponderada em vez de máximo — ⏸️ decisão de produto, não cura

`reserva = Σ(wᵢ·vᵢ)/Σwᵢ` (razão de duas integrais de arco ⇒ independente do espaçamento, honra a lei do
módulo). Suave por construção, sem silhueta de disco no mapa. **Mas muda duas coisas que o dono aprovou:**
(i) a volta **clareia o miolo escuro** da ida (a média puxa `v₁` para baixo em toda a sobreposição, não só
na costura); (ii) re-entintar um rastro desbotado deixa de **restaurar** (*"a fresh head dab wins"* vira uma
média). E dobra a memória do plano (dois acumuladores `f32`/`u16` no lugar de um `u8`; a 4096² são dezenas
de MB). Fica registrada para o dia em que o look pedido for outro.

## 5. Encomenda (2) — Rewet e Smudge agindo sobre a tinta do próprio traço

### S2-A · o Rewet governa o RAIO da difusão da reserva — ✅ recomendada (é a S1-B com um botão)

```text
R = R₀ + Rewet · Bleed          (e, como no dissolve da tinta velha, 2× onde o pincel demorou — soak)
```

É a leitura honesta de "Rewet" para a tinta própria: *mais água ⇒ o pigmento viaja mais longe dentro do
corpo molhado* — a mesma frase, e o mesmo raio (`spread`), que o dissolve já usa para a tinta velha.
⭐ **A janela já paga por isso:** com `Rewet > 0` o `reach` já é `spread` (e `2×` sob soak)
([window.rs:110](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render/window.rs#L110)) ⇒ a cerca
1 do §4 fecha **por construção**, sem crescer a janela. Aqui a máscara **por dono** deixa de ser opcional
(`R` chega a 48 px, muito além do gap de não-contato), e `wet`/`spread_px` por dono **já estão** na
`WetStrokeStyle`.

Sem self-feeding e sem cadência: é função pura dos acumuladores, recalculada do zero a cada composite.

### S2-B · o Smudge arrasta os ACUMULADORES, não só a base — ✅ recomendada, com uma ressalva de física

O gêmeo escalar do `smear_dab` aplicado ao `stroke_deplete` (e ao `stroke_color` quando ele tem conteúdo),
na mesma cadeia `prev → centro` que o `smear_wet_base` já percorre.

* ⚠️ **Tem de ser intercalado POR DAB, em ordem** (`arrasta(mapa, prev→i)` e só então `max(mapa, vᵢ)`),
  dentro do laço do `accumulate_wet_coverage` — hoje a rota é em LOTE (*cobertura de todos → smear da base
  → cor de todos*, [stamp_route.rs:118-134](../../crates/ph2d-tool-painter/src/tool/paint/stamp_route.rs#L118)).
  Em lote, o resultado dependeria de onde o lote corta, isto é, **da taxa de eventos** — o defeito que o gate
  de polling existe para impedir.
* Isto **não** é o self-feeding recusado: aquele era o reservatório lendo o **composite** numa cadência de
  quadro; este é uma mutação dos acumuladores na ordem do caminho. A dependência de espaçamento que ele
  herda é a que o smear da base **já tem** e que foi aceita.
* ⚠️ **A ressalva, e ela vale HOJE também depois do pen-up:** smear é **translação ao longo do gesto**.
  Esfregar **atravessando** a costura a borra; correr **paralelo** a ela (o U da foto) quase não — o valor
  é arrastado ao longo da própria costura. Quem cura o U é a S2-A (isotrópica). Dizer isto ao dono no
  smoke, senão a S2-B lê-se como "não funcionou".
* Kernel novo em `ph2d-painter-brush` (irmã do módulo, não é foundational de contrato).

### S2-C · o pincel gasto RECOLHE a própria tinta molhada ao voltar — ⏸️ decisão do dono (a mais física; cara de CONSTRUIR, barata de RELÓGIO — §9)

Na vida real o pincel quase seco que volta sobre o próprio escuro molhado **pega pigmento e o carrega**
(Pull) — a volta sairia com um degradê puxado do flanco escuro. Exige distinguir *"o pincel VOLTOU"* de
*"o dab vizinho"* (com espaçamento de 5–10 % do raio, quase todo texel sob o pincel foi coberto pelo dab
anterior): um **plano novo do tamanho do canvas** com o **arco da última cobertura por texel**, e o pickup
só de texels mais velhos que ~2–3 diâmetros de percurso. Sem esse plano é self-feeding literal — o pincel
se reabastece do próprio rastro e o Charge deixa de gastar.

Arte prévia: [doc 20 §9.2 / §10](20_accumulate_na_mesma_pincelada.md) chegou **ao mesmo plano** pelo lado do
Impasto e o precificou como o caminho **caro**. Se um dia as duas features forem pedidas, o plano é um só.

### S2-D · "pen-up automático" quando o traço volta sobre si — ⛔ recusada

Re-congelar a base no meio do traço faria Rewet, Smudge e pickup "simplesmente funcionarem" — é o que o Enio
observou funcionando. Recusada por três razões já pagas: (i) é o **mesmo gatilho** da S2-C (precisa do plano
de arco) **mais** um commit no meio do gesto e o acerto com o **undo** (o traço é um passo só) —
[doc 20 §10](20_accumulate_na_mesma_pincelada.md) reverteu a própria recomendação por isso; (ii) a base
re-congelada é exatamente o que **envenenou a reprodutibilidade** em 2026-07-09; (iii) o bake do meio
**assa o aro interno** que a lei *um wash, um aro* existe para não ter.

## 6. Recomendação e ordem

| wave | o quê | encomenda | risco | muda o look aprovado? |
|---|---|---|---|---|
| **W1** | S1-A + S1-B (campo difuso da reserva, `R₀` por smoke em escada) | (1) | baixo | só a casca de traços com Charge < 1 — **medir e mostrar** |
| **W2** | S2-A (`R = R₀ + Rewet·Bleed`, máscara por dono, soak 2×) | (2) Rewet | baixo — a janela já reserva o raio | Rewet 0 ⇒ igual à W1 |
| **W3** | S2-B (smear intercalado dos acumuladores) | (2) Smudge | médio — reordena o laço do splat e traz kernel novo | Smudge 0 ⇒ byte-idêntico |
| — | S2-C | realismo extra | alto de CONSTRUIR (plano novo + reordenar o splat); relógio **≈ +1 %** do carimbo na variante certa (§9) | sim — **decisão do dono** |

W1 e W2 são **um mecanismo**; separá-las só serve para o smoke isolar "ficou suave sem botão nenhum?" de
"o Rewet agora manda?".

## 7. O instrumento e os gates (red-first) que a wave deve escrever ANTES da cura

⛔ **Nenhuma régua deste módulo vê uma costura INTERNA hoje** — as sondas de look medem alfa ao longo do
traço ou na silhueta. Passo zero:

* **A régua:** traço em U determinístico (ida + volta sobrepondo ~⅓ do raio, Charge baixo, **Ragged Edge de
  fábrica** — sem o warp a fixtura não contém o "pixelado"); ao longo de cortes transversais à costura,
  medir **(a)** o maior salto de byte entre texels vizinhos e **(b)** a largura 10–90 % da transição.
* **A barra sai do lado APROVADO, não de um número escolhido:** *a costura interna não pode ser mais dura
  que a borda EXTERNA do mesmo traço*, que o dono aprova — medida pela mesma régua, no mesmo render. (E o
  controle: hoje a régua tem de ler a costura **mais dura** que a borda externa, senão ela não vê o
  fenômeno.)
* **Controles que têm de ficar verdes / byte-idênticos:** Charge = 1 (sem mapa) · traço sem sobreposição
  longe da casca · `incremental ≡ full` · independência de polling · não-contato da sessão ·
  `watercolor_wet_session_survives_charge_slider_change`.
* **W2:** mesma fixtura em Rewet 0 / 0,5 / 1 ⇒ a largura 10–90 % tem de ser **monótona** no Rewet (uma
  régua só de "mudou algo" aprovaria um Rewet que endurece). **W3:** gesto que **atravessa** a costura com
  Smudge 0 contra 1; e o gesto **paralelo** como divergência **declarada** (mede pouco, e o gate diz que é
  esperado).
* **Mutações que têm de sangrar:** leitura de volta a `nearest` · máscara removida (reserva vaza para fora /
  zero entra na média) · raio lido do pincel vivo em vez do dono · `R > reach` (parte o `incremental ≡ full`)
  · smear em lote em vez de por dab (parte o gate de polling).

## 8. O que NÃO foi medido (e uma observação lateral)

* **Nada foi corrido.** A tabela do §2.3 é aritmética sobre as leis do fonte; os números reais da foto
  (raio, Charge, Bleed) são desconhecidos. A régua do §7 é o que transforma isto em medição.
* **O regime pós-pen-up** em que o Enio "resolveu" a borda (§3.2: papel seco *vs.* sessão contínua) não foi
  determinado. Não muda o diagnóstico — nos dois a tinta só vira visível aos instrumentos **depois** do bake.
* **Observação lateral, NÃO verificada em execução:** com o mixer ligado sobre papel **virgem**, a
  prioridade de depósito é `t = pickup·w = 0` ⇒ o `stroke_color` nunca é escrito ⇒ `col_a = 0` ⇒ o ganho do
  aro leva o fator `0,5 + 0,5·col_a = 0,5` do EDGE-4
  ([render:399-409](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs#L399)), contra `~1,0`
  com o mixer desligado. Se for assim, passar o Charge de `1,00` para `0,99` **corta o aro pela metade** num
  degrau — uma emenda que o doc do `MIX_DEPLETE_SPAN` diz não existir. Pode ser look já calibrado e
  aprovado; fica anotado para quem tiver a régua na mão.

## 9. O custo em PERFORMANCE da S2-C (o "item 4"), avaliado a pedido do Enio (2026-09-20)

> **Veredito:** o item 4 é **caro de CONSTRUIR e barato de RELÓGIO** — desde que seja construído na variante
> certa. O §5 chamava-o "a mais cara" sem separar as duas coisas; esta seção separa, com número.
> Há **duas armadilhas** que o tornam caro de verdade, e as duas têm preço medido abaixo.

### 9.1 De onde saem os números — um SUCEDÂNEO declarado, calibrado contra a sonda do produto

⚠️ Por ordem (*"não mude o código de nada"*) nada foi compilado na árvore. Os números vêm de um banco
**fora do repo**: réplica literal da FORMA dos dois laços de splat do
[watercolor_accum.rs](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_accum.rs) (caminho
*Automatic*, sem gates, serial, canvas 4096², espaçamento de fábrica 10 %), `rustc -O`, mínimo de 7
corridas, `loadavg 2,75`. **É outro programa** — serve para RAZÕES e ordens de grandeza.

**Calibração contra o produto:** a sonda `measure_whether_the_carimbo_follows_the_canvas_or_the_footprint`
registra `0,488 ms × 43 chamadas ≈ 21 ms` para ~41 dabs a raio 100 com Charge 1 ⇒ **~0,51 ms/dab**; o
sucedâneo prevê para o mesmo caso `(2,6 + 8,3) ns × 40 000 texels ≈ 0,44 ms/dab` — **−15 %**. ⇒ o passo zero
da wave é re-medir com a sonda do produto, agora em três colunas: Charge 1 · Charge < 1 em papel virgem ·
Charge < 1 voltando sobre o próprio traço.

### 9.2 Um achado que muda a conta: HOJE o traço com Charge < 1 em papel virgem é o MAIS BARATO

O passe de COR faz source-over em ponto flutuante por texel — é **~75 % do carimbo** de um pincel padrão
(Charge 1). Com o mixer ligado sobre papel virgem a prioridade é `t = 0` ⇒ todo texel sai pelo
`if a <= 0 { continue }` ([accum:600](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_accum.rs#L600))
e o passe de cor só paga distância + feather:

| 40 dabs | raio 45 | raio 250 (o pincel do log do Enio) |
|---|---|---|
| passe de COBERTURA hoje (cobertura + reserva + dono) | 0,92 ms | 26,1 ms · 0,65 ms/dab |
| passe de COR, papel virgem com Charge < 1 (**pula**) | 0,50 ms | 15,1 ms |
| passe de COR **depositando** (Charge 1, ou cruzando tinta) | **2,69 ms** | **80,3 ms · 2,0 ms/dab** |

### 9.3 O que o item 4 acrescenta, peça a peça

| peça | custo | recurso |
|---|---|---|
| **plano novo** `u16` — o arco da **PRIMEIRA** cobertura por texel | **+33,6 MB a 4096²** (8,4 a 2048² · 134 a 8192²) — ~+8 % do conjunto de trabalho da aquarela | **memória** |
| escrever o arco dentro do laço de cobertura (`if arc == 0 { arc = t }`) | **+1,2 % / −0,1 %** do passe de cobertura (= ruído) ⇒ **≲ +1 % do carimbo** | largura de banda |
| zerar o plano a cada pen-down | **0,41 ms** o canvas 4096² inteiro · **0,13 ms** só o rect do traço anterior | largura de banda |
| o pickup por dab, **5 taps** (a forma do `sample_surface` de hoje) lendo reserva + arco | **0,0005–0,0017 ms por 40 dabs** — nada | — |
| o composite | **zero** — lê os mesmos planos de hoje | — |
| rodar o mixer POR DAB antes do splat (exigência de cadência, §9.5) | **zero de relógio** — é a mesma conta noutra ordem; custo é de construção | — |

⚠️ **Tem de ser o arco da PRIMEIRA cobertura, não da última:** com espaçamento de 10 % o dab `i−1` já
cobriu ~95 % do disco do dab `i`. Um plano de "última cobertura" seria sobrescrito pela cabeça da própria
volta e leria *idade ≈ 0* em quase tudo — o pickup nunca armaria. Guardando a primeira, o texel da ida
continua "velho" enquanto o disco da volta estiver sobre ele, que é o tempo físico de contato. (O laço não
explode: `carry ≤ pickup·w·pigmento_da_fonte < fonte` ⇒ contração, converge.)

⚠️ **O plano aloca UMA vez por sessão e é zerado por `memset`** (0,41 ms), nunca `clear()` + `vec![0; n]`
por traço: um plano preguiçoso paga a primeira escrita em **faltas de página dentro do laço quente** — a
~2 páginas por linha, um dab de raio 250 estrearia ~1 000 páginas.

### 9.4 As DUAS armadilhas que o tornam caro — cada uma com o preço

1. ⛔ **Pickup por INTEGRAL DO DISCO em vez de 5 taps.** Parece "mais correto" e custa uma segunda
   caminhada do disco por dab: **+58 %** do passe de cobertura nos dois raios (0,53 ms / 40 dabs a raio 45;
   **14,7 ms / 40 dabs a raio 250 = +0,37 ms por dab**). O `sample_surface` já vive com 5 taps e a
   subamostragem dele em pincel grande é dívida conhecida e separada ([doc 12](12_aquarela_auditoria_pos_f123_padrao_ouro.md)).
2. ⛔ **Deixar o auto-pickup acender o passe de COR em papel virgem.** Se a volta passar a ter `t > 0`, os
   dabs dela trocam o *pula* pelo *deposita*: o carimbo desses dabs vai de `0,65 + 0,38 = 1,03` para
   `0,65 + 2,0 = 2,65 ms/dab` a raio 250 — **2,6×** (2,5× a raio 45). E é trabalho **inútil**: num traço de
   uma cor só, a cor recolhida **é** a do pincel, que o composite já usa como fallback.

   ⇒ **a variante certa recolhe a RESERVA sempre, e a COR só onde o próprio depósito TEM cor**
   (`stroke_color` com alfa > 0 — o traço cruzou tinta alheia antes e a carrega). Em papel virgem o passe de
   cor continua no *pula*; sobre tinta ele já estava no *deposita* hoje. **Custo marginal do item 4 nessa
   variante: o ≲ +1 % do §9.3.**

   **O teto, se alguém escolher a variante cheia:** os dabs da volta passam a custar o que um traço de
   **Charge 1 já custa hoje** (+~3 %) — o regime que o dono declarou fluido a 4096² *"nos parâmetros
   padrão"* ([doc 32 §1](32_aquarela_o_que_custa_hoje.md)). Não existe cenário em que o item 4 custe mais
   que o pincel padrão. Em quadro: a raio 250, mão a ~1 500 px/s ≈ 0,5 dab/quadro ⇒ **+0,8 ms/quadro**;
   num risco rápido de ~5 000 px/s ≈ 1,7 dab/quadro ⇒ **+2,7 ms/quadro**, só enquanto a volta durar.
   ⚠️ No regime pesado do log do Enio (Rewet 0,4 ⇒ composite **18,5 ms**) o quadro já está acima de 16,7 ms
   — ali esses 2,7 ms **não cabem**, e é mais uma razão para a variante certa.

### 9.5 O custo que NÃO é de relógio (e é o que de fato encarece o item)

* **A aproximação por LOTE do carry deixa de servir.** Hoje o passe de cobertura re-executa a cadeia de
  percurso e usa o carry do **início do lote**
  ([mixer:198-202](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_mixer.rs#L198) — *"approximate
  with the CURRENT stored state"*). Inofensivo enquanto o carry é raro; com o item 4 ele **é** o efeito
  visível, e um carry quantizado por lote faz o resultado depender de onde o evento de ponteiro corta ⇒
  **da taxa de polling** (o gate `measure_whether_the_wash_depends_on_the_polling_rate`). Com pincel pequeno
  (raio 5 ⇒ limiar de idade de 20–30 px) um único evento já passa do limiar — não é caso raro. ⇒ o mixer
  roda **por dab, antes do splat daquele dab**: reordenar as duas funções mais cicatrizadas do arquivo e a
  disciplina de replay do rng. Relógio zero; risco de construção alto.
* **Observação lateral, não medida no produto:** o passe de cor no *pula* ainda recalcula distância +
  feather de todo texel — **~35 % do carimbo** de um traço Charge < 1 em papel virgem (0,50 de 1,42 ms).
  Uma única caminhada do disco servindo os dois passes (que a reordenação acima quase obriga) **devolveria**
  esse tempo. Nenhuma recusa medida no [doc 28](28_otimizacoes_o_que_funcionou.md) cobre isto; é hipótese
  até a sonda do produto dizer.

### 9.6 Em uma linha

| variante | relógio | memória | construção |
|---|---|---|---|
| **reserva sempre, cor só onde há cor** (a certa) | **≲ +1 % do carimbo**, composite intocado | +33,6 MB a 4096² | alta — plano novo + mixer por dab |
| cheia (cor sempre) | dabs da volta **2,6×** ⇒ o custo de um traço Charge 1 de hoje | idem | idem |
| ⛔ com integral do disco | **+58 %** do passe de cobertura em TODO dab | idem | idem |

# BUGS do módulo 3D / Sculpt — os que a CAUSA enganava

> Irmão do [`BUGS_physics.md`](../Physics/BUGS_physics.md) e do
> [`BUGS_painter.md`](../Painter/BUGS_painter.md): aqui só entra o defeito cuja
> **causa apontava para o lugar errado**, ou cujo gate estava **VERDE sobre
> ele**. O log cronológico das waves é o
> [`21_plano_modos_e_ferramentas.md`](21_plano_modos_e_ferramentas.md) §7 — este
> arquivo existe para a próxima LLM não repetir a investigação, não para
> duplicá-la.

---

## #1 — "Modo L: o Falloff parece ter borda dura" (2026-08-13)

**Sintoma (screenshot do Enio):** uma escada ao longo de um arco que cruza o anel
do cursor e segue pela esfera, no modo `L` do Grab.

**O que enganou, em ordem:**

1. **A grandeza errada.** Eu li o `rigid_profile` em `r/ε = 3` — **0,00011** — e
   declarei a hipótese refutada em voz alta. O `rigid_profile` é só o ESCALAR do
   kernel; o que o artista vê é `|grab|`, que inclui o termo anisotrópico
   `(r·f)r`, e ele vale **0,03472**. Os dois diferem **300×** na borda. *A
   tabela da §7.10 estava certa e eu tinha medido outra coisa.*
2. **O gate que certificava o defeito.**
   `the_rim_residual_is_what_chose_the_scale_family` mede exatamente esse
   0,0347 e afirma `< 0,036` — ele não estava cego, ele **aprovava** o resíduo,
   com uma mensagem (*"o Tri é o que torna a borda do CURSOR honesta"*) que era
   verdadeira enquanto `ε = raio/3` e falsa desde a §7.11.
3. **A cura óbvia é a errada.** Esticar `KELVINLET_REACH` mede 4 → 1,19 % ·
   5 → 0,48 % · 6 → 0,215 % — **nunca zero**, com vértices a crescer como `r²`.
   Um kernel regularizado tem cauda infinita por construção.

**A causa real:** a curva que o `stroke` entrega a um verbo de campo era a
**indicadora do suporte** (`dist <= query_r`, um corte C0), e o corte caía onde o
campo ainda carrega 3,47 % do bico. O degrau sempre existiu — a **§7.11 mudou-o
de LUGAR**, do anel do cursor (10 vértices, onde se lê como *a borda do pincel*)
para 3× o anel (114 vértices, onde nada o explica). É a §0 mordendo a wave
anterior da própria linha.

**A cura:** `kelvinlet::rim_landing` — uma janela C¹ no **CONSUMIDOR**, com o
kernel do paper intacto. Detalhe, números e as três mutações no §7.13 do plano.

**A lição que sobrevive ao fix:** *um gate pode estar verde porque CERTIFICA o
número, e o veredito dele é calibrado para uma colocação que outra wave pode
mudar.* Quem move `ε`, `REACH` ou o raio da consulta reconfere este gate.

---

## #2 — "Pinch em B e S são idênticos" — o chip `B` vestia a lei de OUTRA ferramenta (2026-08-15)

**Report:** *"Pinch em B e S bons mas idênticos ou quase idênticos."*

**Por que a causa enganava:** o `B` do Pinch carregava `LateralPull::Tangential`,
e o doc dessa variante afirmava, em letras garrafais, que ela **não** era a lei do
Blender — logo o chip parecia legítimo por construção (*"três leis, não duas"*) e
o defeito parecia ser de calibração. Medido, o que separava os dois em força
`1,00` era **0,0125 r, 9 % do pico**: dois apertos radiais separados por um
arredondamento.

⚠️ **A causa raiz é uma leitura de fonte alheia feita pelo COMENTÁRIO e não pelo
código.** A nota descrevia o `pinch.cc` a partir do comentário dele — *"Project
the displacement into the X vector (aligned to the stroke)"* — e essa frase é
**falsa no próprio Blender**: o código monta `X = cross(area_no, grab_delta)`, que
é **perpendicular** ao traço. O erro inverteu o mapa inteiro e propagou-se para
**três** docs (a variante, o gate do verbo, o corpo do `lateral_pull`), cada um a
citar o anterior.

**A verdade, lida do `cross`:** o `crease.cc:112` faz **exatamente** a nossa
projeção tangencial (*"pinched towards a **line** instead of a single point"*), e
o `pinch.cc` remove a componente **ao longo do traço**. Nós coincidíamos com um e
faltava-nos o outro.

**A lição:** *quando a fonte tem um comentário e um `cross`, o `cross` é a fonte.*

⚠️ **E a nota que declarava isto bloqueado tinha ENVELHECIDO:** ela dizia *"fechar
a dele pede o frame do traço dentro do `Dab` — wave própria"*, e o `Dab::path`
chegou na wave da FAIXA sem ninguém reconferir.

---

## #3 — "Pinch/Blob em L são ruins" — o gate afirmava a coisa certa sobre o lugar errado (2026-08-15)

**Report:** *"Blob modo L ruim … em L Pinch ruim."*

**Por que a causa enganava:** havia um gate,
`the_elastic_pinch_gives_back_along_the_normal_what_it_takes_from_the_plane`,
verde, cujo nome afirmava exatamente a propriedade que justificava o `l-mode`
existir — e ele media **0,5043 contra 0,1515** do modo que já shipava. A leitura
natural era *"a lei está certa, o problema é afinação"*.

⚠️ **Ele somava o deslocamento normal sobre a ESFERA INTEIRA.** Decomposto por
banda, a espirrada que ele media vive **toda fora do anel do cursor**; dentro
dele a normal é **negativa** (−0,00078 na banda 0,5-0,75 r contra um lateral de
+0,00761). *Uma soma global disse o contrário do que acontece sob o cursor* — a
mesma doença que o Painter 2D pagou ao medir a ondulação no EIXO do traço em vez
do ombro.

**A causa real, e ela é geometria:** o traço zero da `F` reparte `+s` na normal e
`−s/2` no plano, mas os vértices de uma **malha** vivem na superfície
(`r · n ≈ 0`), então o termo normal é ~zero. **Uma casca não tem material fora do
plano para receber o que sai de lado.** Consequência medida: o campo removia
**4,8× mais** volume que o modo sem ele — o oposto exato do que existia para
fazer —, com **62,4 %** do gesto fora do anel.

⚠️ **E os outros dois gates da família EXIGIAM o defeito.** Com o campo já
removido, a mensagem de falha de um deles foi: *"o empurrão elástico não alcança
além do anel — sem isso o `l-mode` do Blob é um domo mais fraco e nada mais"*.
Ele estava certo sobre o mecanismo e errado sobre o veredito.

**A lição:** *um gate pode pinar exatamente aquilo que o artista vai reprovar — e
o nome dele soa como uma virtude enquanto isso.*

**Fechamento:** o `Field::Pinch` saiu. A razão final é de REFERÊNCIA, não de
número: o `elastic_deform.cc` do Blender porta o mesmo paper e declara cinco
famílias (`GRAB`, `GRAB_BISCALE`, `GRAB_TRISCALE`, `SCALE`, `TWIST`) — **nenhuma é
o pinch**. Detalhe inteiro na §7.24 do plano 21.

---

## #4 — "Falloff provavelmente errado" na DEMÃO — o falloff estava certo e a FORÇA vinha da referência errada (2026-08-16)

**Sintoma (dois screenshots do Enio, no smoke da `=33`):** a camada do Layer sai
com uma **parede quase vertical** e a borda **escadeada**, contra uma referência
de bordas macias. Veredito dele: *"Falloff provavelmente errado. resultado muito
diferente e pior"*.

**O falloff foi INOCENTADO por medição, e a régua não é o kernel — é o PERFIL
RADIAL do produto** (sonda `measure_layer_law::the_radial_profile_of_a_coat`).
A lei satura: `disp += f·s·(1,05 − |disp|)`, então **todo peso converge para 1** e
o falloff decide só *quão depressa*. Com força `0,50`, o ombro caminha para fora
e se comprime — a 64 dabs a fração da altura em `t = 0,5 … 0,95` é
`1,000 1,000 1,000 0,715 0,277`. **Não há curva que impeça isso**: a forma é da
LEI, e o que decide se o artista vê ombro ou degrau é a **taxa**.

**A causa, com número.** `Brush::weight` — a porta única entre o slider e o peso
do dab — perguntava `verb.profile(self.mode)` **sem o recuo** que o
`RefMode::kernel_for` e o `RefMode::lateral_for` têm desde que a lei foi escrita.
O `S` **não declara** a demão (`layer.cc` é do Blender), então `profile(S, Layer)`
é `None`, o `map_or` caía no slider CRU, e a referência deste verbo eleva ao
QUADRADO (`sculpt.cc:2337-2339`). Slider `0,50` entregava **0,5000 contra 0,2500
— o dobro da taxa de depósito**, e numa lei que satura o dobro da taxa não
deposita mais alto: ele **colapsa o ombro**, que é exatamente o que a foto mostra.
Perfil medido lado a lado a 64 dabs:

| modo | t=0,5 | t=0,7 | t=0,8 | t=0,9 | t=0,95 |
|---|---|---|---|---|---|
| `S` (o que shipava) | 1,000 | 1,000 | **1,000** | 0,715 | 0,277 |
| `B` (o que a referência pede) | 1,000 | 1,000 | **0,919** | 0,460 | 0,151 |

**E o defeito não era da demão — era de SETE verbos.** O
`Sculpt3dUi::default()` do painel **já derivava** o modo de nascimento (com o
porquê escrito no doc-comment desde a W6), e a **shell** — que é quem de facto
faz o estado nascer — escrevia `[RefMode::default(); Verb::ALL.len()]`. Censo
(`P8` da mesma sonda): **7 dos 23** nascem num modo que não os declara —
`Blob` · `ClayStrips` · `ClayThumb` · `MultiplaneScrape` · `SlideRelax` ·
`SurfaceSmooth` · `Layer` —, todos a `0,5000` onde a referência pede `0,2500`.
*Duas respostas para uma pergunta, e a que ganhava era a que ninguém tinha
escrito de propósito.*

**Por que 277 gates ficaram VERDES sobre isto:** **64 das 86 fixtures** desta
crate usam `strength: 1.0`, e **`1² == 1`** — o quadrado é a IDENTIDADE ali. A
suíte inteira era cega por escolha de fixture, e o `lateral_for` já tinha pago
esta lição uma vez (o `#2` acima: *"em força 1,00, onde o `x²` do `B` é a
identidade…"*). O gate novo usa slider **0,40** e diz por quê no doc-comment.

**A cura, nas duas metades — e elas cobrem superfícies diferentes:**

1. `RefMode::birth_for(verb)` — *em que modo este verbo nasce* —, e a shell E o
   painel a PERGUNTAM. Sem ela o chip da faixa nasce **apagado** (o painel só
   pinta os oferecidos) e o gesto *"carimbar a referência em todos"* não alcança
   o verbo.
2. `RefMode::for_verb` — o `if` que estava escrito à mão em dois sítios e
   **ausente num terceiro** — e o `weight()` passa por ele. É esta metade que
   torna o defeito **estruturalmente impossível** em vez de consertado num sítio
   de nascimento.

**⚠️ E o destino do recuo continua o `Self::B` LITERAL, porque eu tentei derivá-lo
e a medição me travou:** trocá-lo pelo `birth_for` parece a mesma frase (*"a
referência que TEM a ferramenta"*) e **não é** — o `Sharpen` é declarado pelos
DOIS modos, então o derivado o manda para o `S` e o literal para o `B`, e o gate
`the_geometric_operator_does_not_leak_into_the_verb_next_door` (W4) reprovou
**sobre produto correto**. O par `(L, verbo que o L não declara)` é inalcançável
pelo produto; re-decidir o destino dele dentro de um fix de outra wave seria
mexer numa decisão vizinha sem medição que a justifique.

**Gates:** `every_verb_is_born_in_a_mode_that_declares_it` ·
`the_weight_asks_the_reference_that_has_the_tool` ·
`at_full_strength_the_square_is_the_identity_and_proves_nothing` (a cegueira
PINADA, para ninguém "simplificar" o gate de volta à força cheia) ·
`the_panel_opens_every_verb_in_a_mode_that_declares_it` · e o arch-gate de fonte
`shells/desktop/tests/every_verb_is_born_in_a_mode_that_declares_it.rs` (de FONTE
porque o `Sculpt3dScene::new` recebe um `wgpu::Device` e não há como construir o
nascimento headless — ele afirma a PROPRIEDADE, com controle positivo).
**4 mutações, 4 sangram** (a do `birth_for` sangra DUAS).

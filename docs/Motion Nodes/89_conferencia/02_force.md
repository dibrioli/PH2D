# 89 · CONFERÊNCIA — Família 2: FORCE (6 nós)

**Data:** 2026-08-09 · **Plano-mãe:** [89_plano_conferencia_dos_nos.md](../89_plano_conferencia_dos_nos.md) §3/§4
**Nós:** `force.attractor` · `force.buoyancy` · `force.curl` · `force.drag` · `force.vortex` · `force.wind`
**Status:** conferência (claims). Nada implementado, nada priorizado em definitivo (§5/§7 do plano são do Enio).

---

## §0 — O que a família é hoje (lido do `MANIFEST`, não do doc)

| nó | params | efeito | lê `falloff` | escreve |
|---|---|---|---|---|
| `force.attractor` | 6 — `target_x` `target_y` `strength` `radius` `curve`(enum 4) `repel`(toggle) | Pure | ✅ | `accel` |
| `force.buoyancy` | 7 — `level` `density` `depth` `drag` `wave_amplitude` `wave_length` `wave_speed` | Temporal | ✅ | `accel` |
| `force.curl` | 5 — `strength` `scale` `speed` `octaves` `seed` | Temporal | ✅ | `accel` |
| `force.drag` | **1** — `coefficient` | Pure | ✅ | `accel` |
| `force.vortex` | 5 — `center_x` `center_y` `strength` `radius` `clockwise`(toggle) | Pure | ✅ | `accel` |
| `force.wind` | 5 — `angle` `strength` `gust` `gust_freq` `seed` | Temporal | ✅ | `accel` |

**29 params.** Os seis multiplicam a contribuição pela coluna `falloff` e **só** acumulam em `accel`
(ADR-0155 `Coupling::Produces("accel")`); `motion.integrate`/`sim.step` integram.

### §0.1 — As quatro fronteiras do substrato (medidas, e é o que decide quase todo item abaixo)

Antes de julgar um gap eu medi **o que o grafo consegue entregar a uma força**. São quatro
canais, e os dois primeiros existem:

1. **Modulação por-TICK de QUALQUER param** — ✅ existe. [Doc 58](../58_params_dirigidos_nota_adr.md):
   `Graph::drive_param(node, param, src)`, e **os 118 nós ficaram dirigíveis sem uma linha de
   mudança em nenhum deles** porque todos leem por `EvalCtx::param`. O próprio doc dá
   `value.lfo → force.wind.strength` como exemplo. ⚠️ **Um param dirigido é UM número por tick**,
   não por instância.
2. **Gate multiplicativo por-INSTÂNCIA** — ✅ existe: a coluna `falloff`, escrita por
   `motion.falloff` (Circle/Rect/Linear), `field.box`, `field.index_range`, `field.radial_sweep`,
   composta por `field.combine` e reformada por `field.remap`. ⚠️ **E ela pode ser NEGATIVA**
   (`field.remap` tem `clamp` como param, default 1.0 mas **desligável**, com `min`/`max` livres):
   `mapped = (min + t·(max−min))·multiplier` sem clamp ⇒ `falloff ∈ [−1, 1]`. Como todas as seis
   forças fazem `mag = … · falloff`, **um campo negativo INVERTE a direção da força**. Ver §3.
3. **Escrever `accel` a partir de um vetor arbitrário** — ❌ **não existe**. Só as seis `force.*`
   escrevem `accel`. `motion.drive` escreve `X/Y/Rotation/Size/Opacity` (medido: `labels: &["X",
   "Y", "Rotation", "Size", "Opacity"]`); `motion.expression` produz um campo escalar `v`.
4. **Derivar um `falloff` de um ATRIBUTO** (velocidade, idade, um `v` do domínio de valor) — ❌
   **não existe**. `value.attribute` LÊ qualquer coluna (inclusive `vel` em modo `Length` = a
   *velocidade escalar*), mas **nada converte um campo de valor de volta em `falloff`**. A família
   `field.*` computa peso a partir de GEOMETRIA (caixa, banda ordinal, varredura angular) e de mais
   nada.

*(3) e (4) são as duas paredes desta família.* Quase todo item marcado **inexprimível** abaixo bate
numa delas, e o §4 SUPERAR propõe derrubar (4) com **um** nó.

---

## §1 — A TABELA

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `force.curl` + `force.wind` | curl 5 (`strength·scale·speed·octaves·seed`) · wind 5 (`angle·strength·gust·gust_freq·seed`) | **cluster NOISE completo**: `type` · `lacunarity` · `roughness` · `offset_x/y` · **`loop_period`** — Houdini POP Force/Wind/CurlNoise *"Attenuation / Roughness / Turbulence / Pulse / Offset"* ([houdini §B](../referencia_pesquisa_houdini_mops.md), tabelas fetchadas; o §A.147 chama de *"cluster de noise padrão, repetido em POP Force/Wind/Curl"*) · Niagara Curl Noise `Pan Noise Field: vec3` ([niagara item 3](../referencia_pesquisa_niagara_stardust.md)) · VFXG Turbulence `Noise Type Value/Perlin/Cellular · Octaves 1..8 · Roughness · Lacunarity` (item 18) · Cavalry Noise `Octaves, Lacunarity, Gain` + **`Looping + Loop Length`** ([cavalry §154](../referencia_pesquisa_cavalry.md)) | **NÃO.** ⚠️ E é REUSO, não engenharia: **`motion.noise` JÁ tem** `octaves·roughness·type(fBm/Turbulence/Ridged)·speed·**loop_len**·seed` — só que escreve `X/Y/Rotation/Size`, nunca `accel` (parede 3). Medido no `force.curl`: `freq = freq*2.0; amp = amp*0.5` **cravados no laço** ⇒ lacunarity e gain não são params, são literais | ✅ **FECHADO** (2026-08-12) — e a célula estava certa sobre o CLUSTER e seria errada sobre o KERNEL: o `motion.noise` usa ruído de **GRADIENTE** e o `force.curl` de **VALOR**, de propósito e com o porquê escrito nos dois ⇒ unificar o ruído de base mudaria a **aparência**; o que se compartilha é a **LEI**, e ela já tinha DUAS implementações **divergentes** (o `motion.noise` desloca o seed por oitava, o `force.curl` não; um multiplica a coordenada, o outro a frequência; um tem os três tipos, o outro só o fBm). Folha nova **`ph2d-fbm`** (zero deps) com o `NoiseType`, a acumulação e a costura do laço. ⚠️ **O SEED fica FORA da lei** — quem decide se as oitavas se decorrelacionam é o ruído de base, que recebe o índice; uma lei que o deslocasse mudaria o curl, uma que nunca o deslocasse mudaria o noise. ⚠️ **O `force.wind` fica SEM `offset_x/y`, com motivo:** o campo de rajada dele tem eixos **tempo × índice da instância** — não é espacial —, então deslizar o eixo do índice **é** um re-seed e o `seed` já é esse knob; dois knobs a duplicar um terceiro é o botão morto que este repo caça. E ele **não tinha fBm nenhum** (uma oitava), então ganhou `octaves` junto | ~~**P0**~~ ✅ | `type=fBm` · `lacunarity=2.0` · `roughness=0.5` · `offset=(0,0)` · `loop_period=0` (=sem loop) ⇒ a expressão de hoje **AO BIT**, medida por gate (`to_bits()`) contra o laço `freq *= 2.0 / amp *= 0.5` que estava cravado |
| `force.drag` | 1 (`coefficient`) | **`directional_scale`** (arrasto anisotrópico per-eixo) — POP Drag `Directional Scale vec3 (1,1,1)` ([houdini §B](../referencia_pesquisa_houdini_mops.md#pop-drag)) | **NÃO** — exigiria `accel += (−kx·vx, −ky·vy)`; nada escreve `accel` de um vetor arbitrário (parede 3), e o `falloff` é **um escalar**, isotrópico por construção (não há como frear mais em Y que em X) | **omissão** — é o que faz uma folha cair balançando em vez de reta | **P1** | `scale_x = scale_y = 1` ⇒ `−k·v` bit a bit |
| `force.vortex` | 5 | **perfil de borda autorável**: `Soft Edge (0.25)` + `Inner/Outer Strength (1 / 0)` — POP Axis Force, tabela **fetchada** ([houdini §B](../referencia_pesquisa_houdini_mops.md#pop-axis-force)) | **PARCIAL, com hack.** O vortex tem `(1 − d/R)` **linear cravado** e **nenhum `curve`** — o irmão `force.attractor` tem quatro (`Linear/Quad/Smooth/Smoother`). Cadeia tentada: `motion.falloff(Circle) → field.remap(contour=Curve) → force.vortex(radius = 1e3)` — com o raio enorme o termo interno vira ≈1 e o campo dá o perfil inteiro. Funciona, mas *"ponha o raio em mil"* é um truque, e o corte duro em `d > radius` continua lá | **omissão, e é uma ASSIMETRIA INTERNA** — a nossa própria família discorda de si: o attractor ganhou `curve`, o vortex não | **P1** | `curve = 0` (Linear) ⇒ o `(1 − d/R)` de hoje |
| `force.attractor` | 6 | alvo = **outro STREAM** (`Attraction Type: Particles / Points / Surface Points`, POP Attract fetchado) · Cavalry **Goal** modifier ([cavalry §98](../referencia_pesquisa_cavalry.md), marcado FALTA lá também) | **NÃO** — o alvo são dois params escalares; nada no catálogo entrega *"o ponto mais próximo daquele outro stream"* a uma força. `motion.pin_constraint`/`motion.look_at` leem alvo, mas não escrevem `accel` (parede 3) | **omissão** | **P1** | modo `Point` ⇒ o `target_x/y` de hoje |
| `force.attractor` | 6 | **Follow / Predict Intercept** (`Force Method`, POP Attract fetchado; + `Ambient Speed`/`Speed Scale`) | **PARCIAL.** Um alvo GLOBAL que lidera É exprimível hoje: `value.slope` (derivada) → `value.math` → `drive_param(target_x)` — doc 58, um número por tick. O que **não** é exprimível é o intercepto **POR PARTÍCULA** (cada uma liderando pelo próprio tempo-de-chegada), que precisaria de um param por instância — parede (3)+(4) | **omissão** (o global) / **natureza** (o por-partícula, que pede um canal que o substrato não tem) | **P1** | `method = Accelerating` ⇒ hoje |
| `force.buoyancy` | 7 | **`density` e `depth` por-INSTÂNCIA** — Cavalry tem **`Buoyancy-FIELD`** (um *field* é por-instância por definição, [cavalry §98](../referencia_pesquisa_cavalry.md)) · Unity `BuoyancyEffector2D` é global mas a densidade vem do corpo (a referência que o próprio doc-comment do nó cita) | **NÃO a custo pagável.** Cadeia tentada: `field.index_range(0..0.5) → force.buoyancy(density=A) → field.index_range(0.5..1) → force.buoyancy(density=B)` — **4 nós por material**, e a partição é por RANK ordinal, não por "que objeto é este". O `falloff` não serve: ele escala empuxo **e** arrasto juntos, então não separa *quão denso* de *quanto está submerso*. ⚠️ O próprio doc do nó confessa a lacuna: *"This is what `size` would be if the substrate had one true notion of it"* | **omissão** — uma rolha e uma pedra na mesma água é o caso de uso, não o exótico | **P1** | um `density_scale` por instância com neutro `1` ⇒ o global de hoje |
| `force.attractor` | 6 | **`Kill Radius`** — Niagara Point Attraction Force ([niagara item 4](../referencia_pesquisa_niagara_stardust.md): *"o detalhe que o attractor PH2D não tem"*) | **SIM.** `motion.falloff(Circle, center=target, radius=kill_r) → motion.cull(mode=Falloff, amount≈1, invert=1)` — o `motion.cull` tem literalmente o modo *"keeps the elements whose `falloff` column is ≥ `amount`"* + `invert`. ⚠️ **Ressalva não medida:** isto muda a CONTAGEM (ADR-0136) dentro do laço `pre` do `motion.integrate`, que pareia por `id` — se é estável ali, não conferi | omissão de ergonomia (3 nós + o centro digitado duas vezes) | **P2** | `kill_radius = 0` |
| `force.attractor` | 6 | **`Reversal Distance`** · **`Peak Force Distance`** · **janela `Min/Max Distance`** — POP Attract, tabela fetchada (defaults `0` / `0` / `0`–`1e30`) | **SIM, e as três são UM gesto de campo.** `motion.falloff(Circle@target) → field.remap(inner_offset = platô, contour = Step/Curve, min = −1, max = 1, **clamp = 0**) → force.attractor`. O `clamp` desligado deixa o `falloff` NEGATIVO, e `mag = strength·w·sign·falloff` ⇒ **o sinal inverte**. `max` já É o `radius` (a força zera em `d > radius`); `inner_offset` **é** o platô | omissão de ergonomia — e há uma **duas-portas real**: o centro do campo e o `target` da força são o mesmo número digitado em dois nós | **P2** | os três em `0` |
| `force.wind` + `force.drag` | 5 + 1 | **modelo alvo-velocidade** `a = airresist·(targetv − v)` — POP Wind ([houdini §B](../referencia_pesquisa_houdini_mops.md#pop-wind), `Wind vec3` + `Air Resistance 1`) · Niagara Wind Force (*"satura: só acelera até a partícula igualar o vento"*, item 6) · POP Drag `Goal Velocity`, POP Axis Force `Treat as Wind` | **SIM, EXATO — dois nós e uma multiplicação de cabeça.** `force.wind(angle, strength = k·\|targetv\|)` + `force.drag(coefficient = k)` soma `k·targetv − k·v ≡ airresist·(targetv − v)`, **termo a termo**. Com o MESMO `falloff` w nos dois, `w·(k·targetv − k·v)` continua exato. ⚠️ Quebra se o artista puser campos diferentes em cada um | omissão de ergonomia (um knob contra dois nós) | **P2** | um modo `Force \| Target Velocity` nascendo em `Force` |
| `force.vortex` | 5 | `Treat as Wind + Air Resistance` (POP Axis Force, fetchado) | **SIM** — a mesma álgebra da linha acima: `force.vortex` + `force.drag` no mesmo trecho da cadeia | omissão de ergonomia | **P2** | idem |
| `force.curl` | 5 | **`Add Collision Objects`** — *"o campo CONTORNA SDFs de colisores"* (POP Curl Noise, [houdini §B](../referencia_pesquisa_houdini_mops.md#pop-curl-noise)) | **NÃO** — exigiria o campo consultar a geometria de `sim.collide`/`motion.collide`, que hoje age DEPOIS, sobre `vel`. É caro e o doc 63 já o marcou P2; concordo | omissão, mas de outra wave | **P2** | lista vazia ⇒ hoje |
| `force.buoyancy` | 7 | espectro de ondas (várias senoides somadas — o mar real; Gerstner) | **NÃO** — empilhar dois `force.buoyancy` **não soma as superfícies**: cada um calcula a própria `submersion` e satura no `clamp(…,0,1)` | natureza parcial: uma senoide já lê como mar; o espectro é refinamento | **P2** | `waves = 1` |
| `force.attractor` | 6 | `Falloff Exponent` contínuo (Niagara item 4) vs. o nosso enum de 4 curvas | ⛔ **RECUSADO — já temos coisa mais geral.** `field.remap` tem `contour = Curve` (curva DESENHADA pelo artista, `ph2d-curve` + `ParamWidget::Curve`), que é o superconjunto de qualquer expoente | — | ⛔ | — |
| todos os 6 | — | **`Ignore Mass`** (POP Force/Wind/Drag/Attract — está em TODOS os quatro) | ⛔ **RECUSADO por MEDIÇÃO:** `grep '"mass"'` no substrato de nós e no `ph2d-nodegraph` = **zero ocorrências**. Não há coluna `mass`, logo força e aceleração são a mesma grandeza aqui e o toggle seria um controle morto | natureza (o substrato não tem a grandeza) | ⛔ | — |
| `force.attractor` | 6 | `Number of Clusters` · `Match Method` (POP Attract) | ⛔ **RECUSADO por dependência:** os dois qualificam o modo *"alvo = outro stream"*, que não existe (linha P1 acima). Se aquele nascer, estes voltam com ele | — | ⛔ | — |
| `force.attractor` | 6 | `swirl` (Stardust Spherical Force) | ⛔ **RECUSADO com CERCA:** é `force.vortex` no mesmo centro, e a composição está **escrita no código** do vortex | — | ⛔ | — |
| `force.vortex` | 5 | **`Suction Speed` / `Origin Pull Amount`** — POP Axis Force (fetchado) · Niagara Vortex Force (item 5, *"o que evita o espalhamento"*) | ⛔ **RECUSADO com CERCA REGISTRADA.** O doc-comment do `force.vortex` já diz: *"purely tangential (no radial component — a free vortex spirals outward by the centrifugal drift; the classic stable-orbit combo is **Vortex + Attractor at the same centre + Drag**)"*. A composição é nomeada no próprio arquivo. ⚠️ O custo é a duas-portas do centro (mesmo número em dois nós) | — | ⛔ | — |
| `force.vortex` | 5 | **`Lift Speed`** (POP Axis Force) | ⛔ **RECUSADO por GEOMETRIA, não por escopo:** o eixo de um vórtice 2D é **Z** (fora do plano); *"velocidade ao longo do eixo"* é movimento para fora da tela. Não é adiamento, é dimensão | natureza | ⛔ | — |
| `force.vortex` | 5 | `Type: Sphere \| Torus` (eixo-círculo, POP Axis Force) | ⛔ **RECUSADO por dimensão** (mesmo motivo) | natureza | ⛔ | — |
| `force.drag` | 1 | `Rotational Drag` (Niagara item 2) · **POP Drag Spin** | ⛔ **RECUSADO por ordem estrutural:** não existe velocidade ANGULAR no stream — `motion.integrate` integra `vel`/`sim_d`, e a rotação é um canal **autorado** (`motion.drive → Rotation`), não um estado dinâmico. Primeiro teria de existir um `spin` integrado; a força vem depois | natureza (hoje) | ⛔ | — |
| `force.wind` | 5 | `Coordinate Space: Local \| World \| Simulation` (Niagara Linear Force, item 7) | ⛔ **RECUSADO:** não há hierarquia de espaço no stream de instâncias — "local" não tem referente | natureza | ⛔ | — |
| `force.buoyancy` | 7 | corrente horizontal / `flowAngle` (Unity `BuoyancyEffector2D`) | ⛔ **RECUSADO com CERCA REGISTRADA:** o doc do nó diz *"A horizontal current is **not** a param here: that is `force.wind` with `angle = 0`, the same argument by which there is no separate gravity node."* | — | ⛔ | — |

**Placar:** 6 nós · 29 params · **0 P0 · 5 P1 · 6 P2 · 10 recusados-com-motivo** · **13 gaps REFUTADOS por composição** (contados no §2).

> ⚠️ **O único P0 desta folha FECHOU em 2026-08-12** (o cluster de ruído do `force.curl`/`force.wind`,
> crate-folha `ph2d-fbm`) e este placar dizia **1** até 2026-08-13 — a linha de contagem sobreviveu ao
> próprio fechamento. Uma contagem velha não é ruído: ela faz a próxima LLM propor construir o que já
> existe, que é exatamente o que esta sessão encontrou **quatro vezes** nas células de expressibilidade.

---

## §2 — Os gaps que a composição REFUTOU (o teste tentado, e o que ele derrubou)

Um gap refutado vale tanto quanto um confirmado — ele impede a próxima varredura de o propor.
Cada linha abaixo foi **tentada** e **fechou**:

| # | O gap | A cadeia que o mata |
|---|---|---|
| 1 | modelo alvo-velocidade (POP Wind / Niagara Wind) | `wind(strength = k·\|targetv\|) + drag(k)` = `k·(targetv − v)`, exato |
| 2 | `goal_velocity` do drag (POP Drag) | **o mesmo item do #1** — são um, não dois |
| 3 | `Treat as Wind` do Axis Force | `vortex + drag`, mesma álgebra |
| 4 | `suction` / `Origin Pull` do vortex | `vortex + attractor` no mesmo centro — **cerca escrita no código** |
| 5 | `swirl` do attractor | `attractor + vortex` — o espelho do #4 |
| 6 | `Reversal Distance` | `motion.falloff → field.remap(clamp=0, min=−1)` — o campo com SINAL inverte a força |
| 7 | `Peak Force Distance` | `field.remap.inner_offset` **é** o platô |
| 8 | janela `Min/Max Distance` | `max` = o `radius`; `min` = `remap(invert + inner_offset)` |
| 9 | `Falloff Exponent` contínuo | `field.remap(contour = Curve)` — arbitrário, não um expoente |
| 10 | `Kill Radius` | `motion.falloff → motion.cull(mode = Falloff, invert)` ⚠️ ressalva do laço `pre` |
| 11 | **`force.gravity`** (doc 63 **P0**) | `force.wind(angle = 270, gust = 0)` — e a cerca está no doc do nó |
| 12 | corrente horizontal na buoyancy | `force.wind(angle = 0)` — cerca registrada |
| 13 | *"a força não é animável"* | **todo** param de força é dirigível por fio (doc 58), sem porta e sem tocar o nó |

⚠️ **Onze dos treze custam entre 2 e 4 nós e um número digitado duas vezes.** Isso não os torna
falsos — torna-os **P2 de ergonomia**, que é exatamente o degrau da régua §7 do plano. E o padrão
que sai deles é um só: *a duas-portas do CENTRO* (`falloff.center_x` ↔ `attractor.target_x` ↔
`vortex.center_x`) aparece em **cinco** das treze linhas.

---

## §3 — SUPERAR

> §4.8 do plano: *SUPERAR não é retórica — é derivar do que só nós temos.*

### 3.1 — O campo tem SINAL (já existe, e ninguém sabe)

`field.remap` expõe `clamp` como **param** (default `1.0`, desligável) com `min`/`max` livres, e a
saída vai para a coluna `falloff` que as **seis** forças multiplicam. Logo `falloff ∈ [−1, 1]` e
**uma força muda de direção onde a geometria autorada mandar**.

Nenhuma referência tem isto: no **C4D** e na **Cavalry** um Field é um peso `[0,1]` (a Cavalry
documenta *"Falloff desligado devolve 1, nunca mata a cena"*); no **Houdini** o falloff de uma POP é
`radius + soft edge` **por nó**; na **Niagara** a máscara é um módulo no stack. O `Reversal Distance`
do Houdini é **um número** — aqui é uma **forma**: um attractor que repele dentro de uma casca e
atrai fora, um `force.drag` que vira **tração** (`−k·v` com `falloff < 0` acelera) numa faixa
autorada, um vórtice que gira ao contrário do outro lado de uma linha.

⚠️ **Isto não é trabalho de construção — é trabalho de PROVA.** Não há gate afirmando o sinal, não
há cena de smoke mostrando-o, e nenhum doc-comment de força menciona que o multiplicador pode ser
negativo. *Uma superioridade latente que ninguém pode citar não é uma superioridade.*

### 3.2 — `field.attribute`: o campo que lê o STREAM (um nó, cinco itens)

A parede (4) do §0.1 — não há ponte `value → falloff` — é a que mais custa nesta família. `value.attribute`
já lê **qualquer coluna**, inclusive `vel` em modo `Length` (= a velocidade escalar). O que falta é o
espelho dele: um nó que **escreve** a coluna `falloff` a partir de um campo de valor. Com ele caem,
sem tocar em nenhuma força:

- **força por IDADE** — a *"force over-life graph"* que a Stardust vende como UI dedicada **por
  força** ([niagara §111](../referencia_pesquisa_niagara_stardust.md): *"Force over-life graphs …
  a curva INLINE é UX que falta"*) vira **um nó para TODOS os effectors**, não só para forças;
- **`force.speed_limit` na forma SUAVE** — um `force.drag` que só age acima de uma velocidade
  (assíntota em vez de clamp), que é o que o artista quer de um estabilizador;
- **a densidade por-instância da buoyancy** (P1 acima) — o campo lê `size` e o empuxo o segue;
- **attractor que solta quem já é rápido** (o `Kill Radius` por velocidade em vez de por distância);
- e o **`field.attribute` compõe** com os campos espaciais pelo `field.combine` que já existe.

Nenhuma referência tem um campo **genérico sobre atributo, composável, aplicável a toda força**: o
Houdini tem `Group` (booleano por regra) mais ramps por-nó; a Cavalry tem Falloff **espacial**; a
Niagara tem curvas por-módulo. Um campo assinado sobre atributo é **estritamente o superconjunto dos
três** — e é *um* nó, porque a família `field.*` já é o composer.

### 3.3 — A força é função pura do playhead, e é bit-exata cross-OS

O gust do `force.wind` é `noise(t·gust_freq, i·0.5 + seed)`; o campo do `force.curl` é `ψ(x, y, t)`
por hash inteiro + smootherstep — **nenhum acumulador em nenhuma das seis**. O único estado da
cadeia é o integrador, e ele já tem `CheckpointRing` (M2.N2, scrub bit-exato).

As referências precisam **assar** para scrubbar uma tempestade: a Cavalry tem `Cache Solver →
.sdcache` ([cavalry §163](../referencia_pesquisa_cavalry.md)), a Niagara re-simula, a Stardust
re-renderiza — e **nenhuma promete o mesmo quadro em duas máquinas**. Aqui a rajada do segundo 15 é
a mesma indo e voltando, e a mesma no Mac e no Windows (HR-5: zero transcendental nas seis forças —
`sqrt`, polinômios e hash inteiro; determinismo cross-OS gateado).

**A consequência de produto:** o vento pode fazer parte da OBRA. O artista pode dizer *"esta
rajada, neste segundo"* e ela será essa rajada para sempre, sem assar nada — o que num app de motion
graphics é a diferença entre um efeito e uma decisão.

⚠️ E é isso que torna o **`loop_period` do P0** mais valioso aqui do que na referência: numa força
**stateless** um loop perfeito é uma propriedade da FÓRMULA (o `loop_len` que o `motion.noise` já
tem), não um artefato de cache que precisa casar nas pontas.

---

## §4 — CERCAS (decisões já registradas que encontrei — greppadas antes de propor)

| onde | a cerca | o que ela recusa |
|---|---|---|
| `force-wind/src/lib.rs:15-17` | *"With `gust = 0` this is a constant directional force — i.e. **gravity** … **A separate gravity node would be this node minus two params, so it does not exist.**"* | o `force.gravity` **P0** do doc 63 §2.2 |
| `force-buoyancy/src/lib.rs:39-40` | *"A horizontal current is **not** a param here: that is `force.wind` with `angle = 0`, **the same argument by which there is no separate gravity node**."* | corrente/flow na buoyancy — e re-afirma a cerca acima |
| `force-vortex/src/lib.rs:7-10` | *"purely tangential (no radial component — a free vortex spirals outward by the centrifugal drift; **the classic stable-orbit combo is Vortex + Attractor at the same centre + Drag**)"* | `suction` / `Origin Pull` (e, pelo espelho, o `swirl` do attractor) |
| `force-drag/src/lib.rs:10-13` | *"The reference had to sum 'upstream + own' velocity … because each force integrated privately; here the single integrator owns the one true `vel` column — **the composition workaround dissolves by architecture**."* | qualquer proposta de dar `vel` própria a uma força |
| `force-curl/src/lib.rs:90-91` | *"Time drifts the field along its own x-axis (**the noise is 2D; a third lattice axis is the follow-up when the noise grows one**)"* | ⚠️ **cerca com pergunta ABERTA** — nomeia o pré-requisito do `offset`/pan do P0 |
| `force-buoyancy/src/lib.rs:97` | *"This is what `size` would be if **the substrate had one true notion of it**."* | ⚠️ **cerca com pergunta ABERTA** — é literalmente o P1 da densidade por-instância |
| `force-attractor/src/lib.rs:11-14` | o `(1−d/R)^power` livre da referência foi trocado pelo vocabulário de curvas *"deterministic polynomials, endpoint-exact"* (HR-5) | o `Falloff Exponent` contínuo |
| `force-vortex/src/lib.rs:13-16` | *"Y-up world … the visual **clockwise** tangent of radial `(dx,dy)` is `(dy,−dx)`. Anchored by test."* | qualquer "correção" do sinal do `clockwise` |
| ADR-0155 (`register_couplings`) | os seis declaram `Coupling::Produces("accel")` — inertes sem integrador, **diagnosticados e curados** | um 7º canal de saída para forças |

⚠️ **Duas dessas cercas são PERGUNTAS, não recusas** (curl `offset` · buoyancy `size`) — e as duas
apontam exatamente para itens desta conferência. Uma cerca que nomeia o próprio pré-requisito é o
melhor sinal que um plano pode receber.

---

## §5 — O DOC 63 ERROU EM

O doc 63 é de 2026-07 e a coluna `status vs PH2D` dele envelheceu **nos dois sentidos**:

1. ⛔ **`force.gravity` como P0 (§2.2) — REFUTADO duas vezes.** (a) A cerca está **escrita no
   código** desde que o `force.wind` nasceu; (b) o qualificador *"massa-aware"* é **vácuo medido**:
   `grep '"mass"'` nos nós e no `ph2d-nodegraph` dá **zero**. Sem coluna `mass`, gravidade e força
   constante são a mesma aritmética. **Rebaixar a ⛔ recusado-com-motivo.**
2. ⚠️ **`force.buoyancy` NÃO EXISTE no doc 63.** Nem na §2.2 (nós novos) nem na §3.2 (gaps por nó).
   O nó com **MAIS params da família** (7) tem **zero linha** de análise — ele veio do
   [doc 60](../60_poisson_e_buoyancy_nota_adr.md), depois do doc 63. É o caso *"item que já existe e
   não está listado"*, e por isso os dois P1 dele (densidade/calado por-instância) nunca foram vistos.
3. ⚠️ **A baseline está morta:** doc 63 §0 diz **"87 nós · 318 params"** e a §3 se intitula *"o gap
   nó-a-nó dos **87** EXISTENTES"*. O censo de 2026-08-09 diz **118 nós · ~420 params**. A régua da
   §3 não é mais a árvore.
4. ⛔ **§3.2 sobre `motion.falloff`: *"→ dissolve na família `field.*`; o nó atual vira alias/compat"*
   — PERIGOSO, e é o achado mais consequente desta conferência.** Medido: `motion.falloff` é o
   **ÚNICO campo RADIAL do catálogo** (`field.box` é caixa, `field.radial_sweep` é angular,
   `field.index_range` é ordinal, `field.remap` é remapeador). **Cinco** das treze composições do §2
   dependem dele. Dissolvê-lo sem um `field.circle` nascer **no mesmo commit** apaga a metade
   espacial desta família inteira.
5. ⚠️ **§3.2 wind (*"modelo alvo-velocidade"*) e §3.2 drag (*"`goal_velocity`"*) são o MESMO item**,
   e ele é **exprimível EXATAMENTE hoje** (`wind + drag`, álgebra no §2 #1). Não é capacidade;
   é um knob. **Rebaixar a P2.**
6. ⚠️ **§3.2 vortex lista `lift` sem ressalva** — ele é **impossível em 2D** (o eixo do vórtice é Z).
   Listá-lo ao lado de `suction` (que é composição de 2 nós) e de `soft_edge` (que é omissão real)
   mistura três naturezas diferentes numa linha só.
7. ⚠️ **§3.2 attractor lista `swirl`** — é o `force.vortex` no mesmo centro, com a cerca escrita no
   código do vortex. **⛔.**
8. ✅ **§2.2 `force.speed_limit` P0 — CONFIRMADO, e agora com o mecanismo que o doc não nomeia.**
   São **duas** barreiras independentes: nada além de `motion.integrate`/`sim.step`/`sim.collide`/
   `motion.emitter`/`motion.boids` escreve `vel`, **e** não há ponte `value → falloff`. ⚠️ E a versão
   **suave** dele (drag proporcional ao excesso de velocidade) cai **de graça** do `field.attribute`
   do §3.2 — o que muda a ordem de construção: o campo primeiro, o nó depois (ou nunca).
9. ⚠️ **§3.1 (cluster NOISE, *"definir UMA vez"*) subestima o que já existe.** Medido: `motion.noise`
   **já tem** `octaves · roughness · type(fBm/Turbulence/Ridged) · speed · loop_len · seed`. Faltam
   **`lacunarity` e `offset`** — e a adoção nas forças. A task **C1** do doc 63 é **reuso + 2 campos**,
   não construção de cluster.
10. ⚠️ **Um fato novo que nenhuma nota registra:** no `force.curl`, `psi = fbm(x + drift + seed, y)` —
    **o `seed` e o `drift` entram no MESMO slot**. Logo dois curls com seeds diferentes são **o mesmo
    campo deslocado em X**, e um seed `Δ` é indistinguível de estar `Δ/speed` segundos adiante. O
    widget diz `Seed`; o que ele faz é um offset temporal. Isso é parte do item `offset_x/y` do P0,
    e é a razão pela qual ele **não** é cosmético.

---

## §6 — O que esta conferência NÃO fez

- Não mediu se `motion.cull` é estável **dentro** do laço `pre` do `motion.integrate` (a ressalva do
  `kill_radius`) — é a única linha da tabela com um "não conferi" explícito.
- Não abriu wave, não decidiu prioridade final, não escreveu código (§4.9 do plano).
- Os nós NOVOS da §2.2 do doc 63 (`force.curve`, `force.line_attract`, `force.follow`,
  `force.conform`, `force.vector_field`) **não são desta família** — mas todos os cinco batem na
  parede (3) do §0.1 (*nada escreve `accel` de um vetor arbitrário*), e vale registrar que é **uma**
  parede, não cinco problemas.

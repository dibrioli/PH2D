# 89 · CONFERÊNCIA — Família 6: **ANIMADORES** (9 nós)

**Data:** 2026-08-09 · **Linha:** `line/motion-value` · **Briefing:** [plano 89 §3](../89_plano_conferencia_dos_nos.md) · leis §4.
**Nós:** `motion.noise` · `motion.oscillator` · `motion.stagger` · `motion.wiggle` · `motion.wave` ·
`motion.drive` · `motion.expression` · `motion.time_remap` · `motion.path`.

**Params lidos do `MANIFEST` (não do doc), 2026-08-09:** noise **9** · oscillator **9** · wave **7** ·
stagger **6** · wiggle **4** · expression **4** (+ o text param `expr`) · time_remap **4** ·
drive **3** (+2 portas) · path **3** (+1 porta) = **49 params**.
**GPU-resident** (WGSL no crate): noise · oscillator · stagger · wiggle · drive.
**CPU-only:** wave · expression · path · time_remap.

---

## A tabela

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.noise` | `channel·amplitude·scale·octaves·roughness·type(fBm/Turb/Ridged)·speed·loop_len·seed` | **`lacunarity`** — Blender *Noise Texture* (manual 4.5 `render/shader_nodes/textures/noise.html`); TD *Noise CHOP* **Spread** (docs.derivative.ca/Noise_CHOP); Cavalry Noise *Cubic/Simplex/Value → Lacunarity* ([cavalry §B l.154](../referencia_pesquisa_cavalry.md)) | **PARCIAL** — empilhar N `motion.noise` (octaves=1, `scale`×L, `amplitude`×r) soma o fBm à mão; ⚠️ **não é exato**: o kernel dá a cada oitava um seed próprio (`seed + o·1013`, `noise.rs:130`), que nós à mão não reproduzem | ⚠️ **CERCA** (`noise.rs:100`: *"rarely-touched … stays internal"*) — a referência a contradiz em **3** ferramentas | P2 | `lacunarity = 2.0` (o `const LACUNARITY` de hoje) |
| `motion.noise` | idem | **transform do CAMPO** (offset · rotation · scale do espaço do ruído) — TD Noise CHOP *Transform*; AE *Fractal Noise* → Transform (rotation, scale W/H, offset turbulence); Cavalry *Noise Position/Rotation/Scale* ([l.154](../referencia_pesquisa_cavalry.md)) | **offset SIM** (`motion.move(+d) → noise → motion.move(−d)`, 3 nós: a amostragem é `P·scale`, o segundo move devolve a pose e preserva o delta). **rotation/scale NÃO** — verificado: `motion.rotate` escreve a coluna **`rot`** (`lib.rs:116`), não gira `P` | omissão | P1 | `offset=0 · rot=0 · scale=1` |
| `motion.noise` | idem | **base de ruído** ≠ Perlin: Cellular/Worley (+Jitter, métrica Euclid/Manhattan/Natural), Simplex, Value, **Curl** — Cavalry ([l.154](../referencia_pesquisa_cavalry.md)); TD Noise CHOP *Type* (Sparse/Hermite/Harmonic/Perlin/Simplex/Alligator/Random); Blender (4.1 fundiu Musgrave: fBM/Ridged/Hybrid/Hetero) | **NÃO** — `type` escolhe a **retificação por oitava**, não o ruído-base; `motion.voronoi` produz células/geometria (não campo escalar para deslocar canal) e `value.noise` é *value* noise no domínio `v`, sem posição | omissão (família nova, não knob) | P2 | tipo `0 = fBm` (o de hoje) |
| `motion.noise` | idem | **min/max** (RANGE) e **Offset** em vez de `amplitude` sozinha — Cavalry *Minimum/Maximum/Offset* ([l.154](../referencia_pesquisa_cavalry.md)); C4D aba Effector *Minimum/Maximum* ([c4d l.111](../referencia_pesquisa_c4d_fields.md)) | **SIM, 2 nós** — o DC é o nó de transform do canal (`motion.move` p/ X-Y, `motion.rotate` p/ Rot, `motion.scale` p/ Size) | omissão (reparametrização) | P2 | `min=−amplitude · max=+amplitude` |
| `motion.noise` | idem | **`stagger`** (defasa a fase TEMPORAL por índice) — Cavalry Noise *Stagger* ([l.154](../referencia_pesquisa_cavalry.md)) | **NÃO** — não há porta de tempo e `seed` é global; o campo é por-POSIÇÃO **por desenho** (doc-comment: *"Field, not jitter — a distinção de `motion.wiggle`"*) | natureza (o irmão por-índice é o `wiggle`) — ⇒ vira **SUPERAR 1** | P1 | `stagger = 0` |
| `motion.noise` | idem | **Separate Channels** · **Use Layer as Seed** — Cavalry ([l.154](../referencia_pesquisa_cavalry.md) + *Defaults inteligentes* [l.228](../referencia_pesquisa_cavalry.md)) | **SIM** — dois nós com `channel` X/Y e seeds distintos; e o `ParamWidget::Seed` já traz **re-roll** (a mesma dor por outra via) | omissão de ergonomia | P2 | — |
| `motion.oscillator` | `channel·wave(5)·amplitude·frequency·phase_stagger·offset·phase·time_mode·bpm` | **onda CUSTOM por curva** — Cavalry *Wave Style: … **Custom (Graph)*** ([l.148](../referencia_pesquisa_cavalry.md)); TD LFO CHOP não tem, mas o *Pattern CHOP* traz 12 janelas (Gaussian/Hann/Hamming/Blackman/Kaiser) | **SIM, 3 nós, EXATO** — `value.lfo(Saw, phase_stagger) → value.curve(desenhada) → motion.drive(channel, Add)`; o `value.curve` já usa `ParamWidget::Curve` e o `drive` é falloff-masked | omissão de ergonomia | P2 | curva não-setada = identidade (é o que o `value.curve` já faz) |
| `motion.oscillator` | idem | **Pulse Width / Bias** — TD *LFO CHOP* (`Pulse Width`, `Bias`) | **NÃO** — o `SPIKE_WIDTH: f32 = 0.08` é **const no kernel** (`lib.rs:127`); nenhuma composição muda uma const | **omissão** — é um número que já existe no arquivo, sem knob | ~~**P1**~~ ✅ **FECHADO em 2026-08-16** (grupo O, cena `=55`) — **UM knob**, `pulse_width`, default `0.5`. ⚠️ **A referência dá DOIS nomes ao mesmo número:** o TD tem *Pulse Width* (a fatia da Square em cima) e *Bias* (onde o pico da Triangle/Saw se senta), e eles são a **mesma fatia do ciclo** — dois nomes é como um artista aprende que são coisas diferentes. A lei é um **WARP DE FASE** aplicado antes da forma, então as **CINCO** formas o herdam de uma vez. ⚠️ **O neutro é a identidade por ARITMÉTICA:** `0.5 / 0.5` é exactamente `1.0` e `0.5 + (f − 0.5)` reconstrói `f` ao bit (Sterbenz). ⚠️ **E a wave achou um DEFEITO VIVO no device:** os três variants de WGSL carregavam a biblioteca **literal** e já tinham divergido — só o de `P` tinha o `osc_cycles_per_second`, então **dirigir a ROTAÇÃO em BPM corria a uma taxa na CPU e a outra na GPU**, sem erro e sem aviso; gate red-first `the_bpm_ruler_reaches_every_oscillator_channel` (RED com `|dif| 0,0397`) e a cura foi extrair a biblioteca para UM lugar | `pulse_width = 0.5` ⇒ o warp é a identidade, BIT A BIT |
| `motion.oscillator` | idem | **Minimum/Maximum** — Cavalry ([l.148](../referencia_pesquisa_cavalry.md)) | **SIM, aritmética do artista** para as 4 formas **bipolares** (`amp=(max−min)/2`, `offset=(min+max)/2`); ⚠️ **NÃO** para o *Spike*, que é unipolar `[0,1]` (`lib.rs:106`) ⇒ trocar de onda muda a faixa em silêncio | omissão (reparametrização + armadilha real) | P2 | `min/max` derivados de `amplitude/offset` |
| `motion.oscillator` | idem | **Time (porta) · Time Offset · Time Scale** — Cavalry *"Time (auto-conectado; desconectável)"* ([l.148](../referencia_pesquisa_cavalry.md)) | **NÃO** — `t` é sempre `ctx.playhead()` | ⚠️ **CERCA com condição de revogação escrita** (ver CERCAS) ⇒ **SUPERAR 1** | P1 | porta desconectada ⇒ `ctx.playhead()`, byte-idêntico |
| `motion.stagger` | `channel·min·max·ease_curve(8)·ease_dir(3)·reverse` | **Graph** (curva X=índice, Y=min→max) — Cavalry *Stagger → Graph* ([l.150](../referencia_pesquisa_cavalry.md)); C4D *Step effector* "rampa 0→1 pelo índice **com spline de forma**" ([c4d A2](../referencia_pesquisa_c4d_fields.md)) | **SIM, 3 nós, EXATO** — `value.instance_field(Ramp) → value.curve(in 0..1 → out min..max) → motion.drive(channel, Add)`; a rampa é o mesmo `i/(N−1)` e o `drive` mantém o falloff | omissão de ergonomia (na fronteira dos "3 nós para um knob" da §7) | P2 | um **9º `ease_curve = Custom`** + curva não-setada ⇒ a família enumerada de hoje (o padrão do 5º contour do `field.remap`) |
| `motion.stagger` | idem | **Offset** (desliza a rampa, ciclicamente) — Cavalry ([l.150](../referencia_pesquisa_cavalry.md)); **Cycles/Taper** — TD *Pattern CHOP* | **SIM mas caro (5 nós)** — `instance_field(Ramp) → value.math(Add/Mul) → value.wrap(Repeat) → value.curve → drive` | omissão | ~~**P1**~~ ✅ **FECHADO em 2026-08-16** (grupo O, cena `=55`) — o param `offset`, faixa `0..1` (a rampa fecha em si mesma; um teto maior seria uma volta que ninguém distingue). ⚠️ **O `frac` só corre com o knob ARMADO, e a razão é a PONTA:** o último elemento senta exactamente em `1.0` e `frac(1.0)` é `0.0` — aplicado sempre, o **neutro** mandaria a peça do fim para o começo da rampa, e só ela notaria. ⚠️ **E duas afirmações minhas caíram no gate:** *"uma volta inteira é o neutro"* e *"o alcance não encolhe"* são as duas **falsas** — uma rampa INCLUSIVA não é um ciclo (rolar trata `0` e `1` como o mesmo ponto), e o que é verdade é que ela **ROLA**, o que o gate afirma ponto a ponto. ⚠️ **E a mutação *"o offset é ignorado"* SOBREVIVEU à primeira fixture**, porque ela re-implementava a lei em vez de cozer o NÓ — o oráculo-espelho | `offset = 0` ⇒ o `frac` nem corre |
| `motion.stagger` | idem | **auto-invert `min > max`** — Cavalry *"Min>Max auto-invertido"* ([l.150](../referencia_pesquisa_cavalry.md), e o §*Defaults inteligentes* [l.228](../referencia_pesquisa_cavalry.md)) | **SIM** — o artista troca os dois números | omissão de ergonomia | P2 | — (comportamento só muda onde hoje o resultado é a rampa invertida) |
| `motion.wiggle` | `channel·amplitude·frequency·seed` — **o mais magro da família** | **`octaves` + `amp_mult`** — a assinatura da própria função da referência: AE `wiggle(freq, amp, **octaves=1**, **amp_mult=0.5**, t=time)` (Adobe, *Expression language reference*); TD Noise CHOP *Harmonics/Gain*; C4D *Random effector* modos Random/Gaussian/Noise/Turbulence + **Indexed/Synchronized** ([c4d A2](../referencia_pesquisa_c4d_fields.md)) | **PARCIAL** — 3 `motion.wiggle` (freq ×2, amp ×0.5, seeds distintos); ⚠️ estatisticamente **diferente** do fBm (campos independentes × oitavas do mesmo campo) | **omissão** — o irmão `motion.noise` tem os dois, e os dois partilham `channel.rs` | ~~**P1**~~ ✅ **FECHADO em 2026-08-16** (grupo N, cena `=54`) — os dois params, com a **assinatura do AE verbatim** (`octaves` e `amp_mult`). ⚠️ **A metade CARA já estava paga, a sexta vez nesta conferência:** a LEI fractal é a folha `ph2d-fbm`, que já tem TRÊS consumidores, e adotá-la **não muda a aparência** porque ela recebe o ruído de base por closure — o `hash2` daqui continua sendo o daqui, e a **4ª cópia da lei não nasceu**. ⚠️ **O default reduz por ARITMÉTICA:** com uma oitava o `eval` devolve `sum/total` com `total = 1.0`, e `n / 1.0` é `n` em IEEE-754 ⇒ os **doze gates que já existiam passam sem uma edição**. ⚠️ **E a mutação me corrigiu:** eu escrevera que o deslocamento de oitava tinha de ir no eixo X senão dois elementos tremeriam juntos — medido, **não** (o `eval` escala as duas coordenadas, e a mutação do eixo SOBREVIVEU aos cinco gates); o que ele compra é o **canto degenerado** (`t = 0, seed = 0`, elemento `0`: `px` e `py` valem zero em toda oitava, a soma colapsa e o valor fica preso em **`-1.000000`**, o extremo do alcance — com o deslocamento, `-0.521581`) | `octaves = 1` ⇒ o campo de sempre, BIT A BIT |
| `motion.wiggle` | idem | **loop** (Looping + Loop Length) — Cavalry Noise ([l.154](../referencia_pesquisa_cavalry.md)); AE *Fractal Noise → Cycle Evolution* | **NÃO** — nada fecha o ciclo de um campo temporal a jusante | omissão — ⚠️ **a lei já está paga**: o `loop_len` do `motion.noise` tem cross-fade smoothstep + gate de convergência escritos | ~~**P1**~~ ✅ **FECHADO em 2026-08-16** (grupo N, cena `=54`) — `loop_len`, pela MESMA porta do irmão (`ph2d_fbm::loop_times`), com o porquê de o peso ser smoothstep e não linear a viajar com ela. ⚠️ **O tempo WRAPA antes de virar coordenada** — e a unidade é `Seconds` e **não** `FromChannel`, o precedente do `ticks` do `motion.delay`: *uma duração não muda de unidade com o que ela fecha* | `loop_len = 0` ⇒ `loop_times` devolve `(t, t, 0)` e a 2ª amostra nem é avaliada |
| `motion.wiggle` | idem | **min/max · Separate Channels** — Cavalry ([l.154](../referencia_pesquisa_cavalry.md)) | **SIM** — mesma resposta do oscilador / dois nós com seeds distintos | ergonomia | P2 | — |
| `motion.wave` | `rows·cols·spacing·speed·damping·center_x·center_y` (+portas `drive`, `state`) | **N PRODUTORES** com tipo/posição/amplitude/frequência/fase — AE *Wave World* (Effect ▸ Simulation ▸ Wave World: **Producers**, Type Ring/Line, Position, Height/Width, Angle, Amplitude, Frequency, Phase) | ⚠️ **SIM, e a célula ENVELHECEU** — ela é de 2026-08-10 e o **Grupo P** (`motion.drive(Custom…)`, 2026-08-16) mudou o catálogo seis dias depois. A metade sobre grades separadas continua verdadeira (*dois `motion.wave` são duas grades*), mas ela não é a única rota: um `motion.drive` escreve numa coluna que o artista batiza, e `wave_h` — a coluna de ESTADO do campo — **não** está no `is_bookkeeping_column`. **MEDIDO** (240 tiques, grade 21×21): `wave.out --pre--> field.box --> value.attribute("falloff") --> motion.drive(Custom "wave_h", Add) --> wave.state` move o berço das ondas do centro (`x = −0,50`) para **exactamente a caixa** (`x = −3,00`), com **440 das 441** células a mudar ⇒ o que ele deposita **PROPAGA**, logo é fonte e não tinta. ⛔ E a rota que parece óbvia — `wave A --> wave B.state` — é um **no-op bit-a-bit**: o `dt` chega em zero no segundo nó, o ramo de hold devolve a entrada verbatim, e o drive de B (5× mais forte) é **engolido em silêncio** | ergonomia | **P2** — não falta capacidade, falta o GESTO: são quatro nós e três arestas à mão, e o artista tem de saber que a coluna se chama `wave_h`, um nome de ESTADO que nenhum picker oferece | cena `=57`, gates `wave_producers.rs` |
| `motion.wave` | idem | **Reflect Edges** (on/off) · **Pre-roll** · **falloff/decay espacial** · **Narrowness/Width** — AE *Wave World* (Simulation: Reflect Edges, Pre-roll); Blender *Wave modifier* (manual 4.5 `modeling/modifiers/deform/wave.html`: **Falloff**, Time **Offset/Life/Damping**, **Height/Width/Narrowness**) | **NÃO** — todos vivem dentro do kernel (borda Neumann hardcoded, sim começa fria) | omissão | P1 | `reflect=1 · preroll=0 · falloff=∞` |
| `motion.wave` | idem | **qual canal a altura escreve** (o height map é escolhível na referência; aqui é `size`) | **PARCIAL/torto** — `value.attribute("size", mode=Length) → motion.drive(Y)` lê a MAGNITUDE (√2·h), não a altura | omissão | P2 | canal `Size` (o de hoje) |
| `motion.drive` | `channel(X/Y/Rot/Size/Opacity)·scale·mode(Add/Set/Mul)` | **escrever numa COLUNA NOMEADA** (qualquer atributo) — C4D aba Parameter, *"o CORAÇÃO do modelo"*, com **Weight Transform** (*"ESCREVE a weightmap — effectors a jusante leem"*, marcado **FALTA**) ([c4d l.117](../referencia_pesquisa_c4d_fields.md)); Cavalry *"connect this value to that attribute"* | **NÃO — e a assimetria está MEDIDA:** `value.attribute` **LÊ qualquer coluna** por nome (widget `Channels` + escape *"Custom…"*), e o `drive` só **ESCREVE em cinco**. Verificado por grep: só `motion.falloff` e as 5 `field.*` escrevem `falloff`; **nada** converte `v → falloff` | **omissão**, e é a mais estrutural da família | ~~**P0/P1**~~ ✅ **FECHADO em 2026-08-16** (grupo P, cena `=56`) — o canal **`Custom…`**, índice **9**, com o NOME num **text param** (`column`), o espelho exacto do `attr` do `value.attribute`, que é o LEITOR deste escritor. ⚠️ **É o ÚLTIMO índice e isso não é arrumação:** o `channel` é um param que o **documento guarda**, então renumerar o que já existe re-aponta em silêncio todo grafo salvo. ⚠️ **Ele RECUSA o device**, e o motivo é ESTRUTURAL: uma `ColumnBinding` carrega o nome como `&'static str` — o sequenciador precisa dele **antes** de cozer, para montar o bind group — e um nome que o artista digita só existe em tempo de cook; o recuo passa pela porta que a `Median` do `value.reduce` já usa. ⚠️ **E a recusa de escrita é feita ao STREAM, não a uma lista de nomes proibidos:** escrever um `Scalar` sobre um `P` (`Vec2`) mudaria o TIPO da coluna e todo leitor a jusante passaria a ler outra coisa — *uma lista apodrece na décima convenção*, e esta casa já tem `texture_id`, `geometry_id`, `uv_rect` e `nrm`. ⚠️ **A LEI é a mesma dos nove canais do enum** (o `blend` por `falloff`, o broadcast do campo de valor, o `scale`), partilhada por helpers e não copiada — há gate para cada metade. ⇒ **a §10.0 do plano fecha**: *ler componente* tinha fechado em 14/08, *escrever coluna arbitrária* fecha aqui | canal novo = variante **apendada** no Enum ⇒ grafo salvo intocado (o precedente `JointKind::Weld` / `Cap::Square`) |
| `motion.drive` | idem | **Size X ≠ Size Y** (Uniform × non-uniform) — C4D *"Scale + S.XYZ **ou** Uniform Scale (1 número)"* ([c4d l.115](../referencia_pesquisa_c4d_fields.md)) | **NÃO** — verificado: o braço Size escreve `si[0]` **e** `si[1]` com o mesmo `v` (`channel.rs:167-170`); dois drives também escrevem os dois | omissão | P1 | canal `Size` (uniforme) segue sendo o default |
| `motion.drive` | idem | **Subtract/Divide/Min/Max** no `mode` — C4D *Blending Mode: Mix/Add/Subtract/Multiply/**Divide*** ([c4d l.116](../referencia_pesquisa_c4d_fields.md)) | **SIM** — `value.math` a montante (negar ⇒ Subtract, recíproco ⇒ Divide) | ergonomia | P2 | modos apendados |
| `motion.drive` | idem | **Transform Space** (Node/Effector/Object) e **Transform Mode** (Relative/Absolute/**Remap**) — C4D ([c4d l.115](../referencia_pesquisa_c4d_fields.md)) | **PARCIAL** — `Add`≈Relative e `Set`≈Absolute existem; **Remap** (*"partindo de 0"*) e o espaço do próprio elemento **não** | omissão | P2 | `Space = World` (o de hoje) |
| `motion.expression` | `a·b·c·d` + text param **`expr`**; vars `i·n·t·f` + colunas **escalares** por nome | **a POSIÇÃO como variável** — C4D *Formula effector*: *"vars id, count, t, **posição**…"* ([c4d A2 l.33 + l.129](../referencia_pesquisa_c4d_fields.md)); Houdini VEX `@P.x`; Niagara *Dynamic Input* com link para atributos de partícula ([niagara l.17](../referencia_pesquisa_niagara_stardust.md)) | **NÃO era — verificado nos dois lados quando esta célula foi escrita** (o escopo só aceitava `Column::Scalar` e `P` é `Vec2`), **e o mundo mudou por baixo dela** | **omissão** — a mesma raiz do gap do `drive`: faltava **ler componente** e falta **escrever coluna** | ~~**P0/P1**~~ ✅ **FECHADO — e por uma wave que não era desta folha.** As **lanes `x`/`y`** existem em `motion-expression/src/lib.rs:119-125`: a busca por coluna escalar falha, e o *fallback* lê o `P` (`Vec2`) e devolve a componente. ⚠️ **A ORDEM é load-bearing e tem gate** (`a_real_column_named_x_beats_the_position_lane`): um stream que de facto carregue uma coluna escalar chamada `x` significa **aquela** coluna, e a resposta mais específica tem de vencer — as lanes sentam **depois** da busca, no mesmo assento que o *fallback* de `param` já ocupava. ⚠️ **A célula é a SEXTA desta conferência a envelhecer** (achada na reconciliação do grupo M, 2026-08-16), e a metade dela sobre o `drive` **continua de pé** — *ler componente* fechou, *escrever coluna* não | `x`/`y` como vars novas: uma fórmula que não as menciona é byte-idêntica |
| `motion.expression` | idem | **coeficientes NOMEADOS** (não `a..d` fixos) · **saída múltipla** — Houdini Wrangle escreve N atributos num snippet; Niagara *Dynamic Inputs* encadeiam recursivamente ([niagara l.17](../referencia_pesquisa_niagara_stardust.md)) | **PARCIAL** — um campo extra chega por COLUNA escalar do `in` (lido por nome, ilimitado); a SAÍDA é um `v` só | natureza na saída (o consumidor é o `drive`, por desenho) · omissão nos nomes | P2 | 4 coeficientes seguem existindo |
| `motion.time_remap` | `mode(Scale/Loop/PingPong/Freeze/Reverse)·scale·offset·duration` | **retiming por INSTÂNCIA** — Cavalry *Shape Time Offset*: *"retima a animação de cada cópia — par canônico com Stagger"* ([l.131](../referencia_pesquisa_cavalry.md)), receita `Stagger → Shape Time Offset` ([l.227](../referencia_pesquisa_cavalry.md)); C4D aba Other *Time Offset [s]*: *"desloca o RELÓGIO da animação do filho **por clone**"* ([c4d l.117](../referencia_pesquisa_c4d_fields.md)) | **NÃO** — o remap é um **escopo de COOK** (`Cook::cook_scoped`), um relógio por sub-árvore; K offsets pediriam K cozimentos, e nada no grafo os interleava | natureza **hoje** (a arquitetura é por-escopo) → **SUPERAR 1** dá a versão contínua | P1 | — |
| `motion.time_remap` | idem | **curva arbitrária tempo→tempo** — AE *Time Remapping* (a propriedade é **keyframável**, não um scale+offset) | **NÃO** — os 5 modos são fechados e não há porta de tempo | omissão — ⚠️ o enabler existe (`ph2d-curve` + `ParamWidget::Curve`, o 5º contour do `field.remap` é o precedente) | P1 (barato) | 6º modo `Curve`, apendado ⇒ grafo salvo intocado |
| `motion.path` | `count·offset·align` (+ porta `offset`; caminho pelo canal externo, `ParamWidget::Source`) | **Start/End · Slide · Offset perpendicular · Side · Spacing (contagem AUTOMÁTICA)** — ⚠️ **o próprio app**: o *Pattern along path* do módulo Vector shipa os seis ([`Vector Module/23_plano_pattern_along_path.md`](../../Vector%20Module/23_plano_pattern_along_path.md), integrado 2026-07-23); Blender GN *Curve to Points* modo **Length** (manual 4.5) + saídas Tangent/**Normal**/Rotation; Cavalry *Text Path (+Loop/**Travel**/Push)* ([l.172](../referencia_pesquisa_cavalry.md)) | **NÃO** — `offset` desliza o conjunto **com wrap**, não recorta o intervalo; `motion.cull` remove elementos sem **redistribuir**, que é o ponto; e o deslocamento perpendicular precisaria da NORMAL, que nada publica (`motion.move` é mundo) | **omissão**, e o caso mais forte da conferência: **o mesmo app responde a mesma pergunta com 6 controles num módulo e 3 no outro** | P1 | `start=0 · end=1 · offset_perp=0 · side=0` |
| `motion.path` | idem | **modo por ESPAÇAMENTO** (em vez de contagem) — Blender *Curve to Points* mode **Length**; Cavalry Duplicator/Pattern (*Spacing* com contagem automática) | **NÃO** — o comprimento de arco da curva não é publicado por nada, então o artista não consegue nem calcular `count = L/spacing` à mão | omissão | P2 | modo `Count` (o de hoje) |
| `motion.path` | idem (⚠️ o irmão `motion.distribute_curve` tem **os mesmos 3** controles + 8 de geometria) | alinhar pela **NORMAL** além da tangente — Blender GN *Curve to Points* → Normal/Rotation | **NÃO** | omissão | P2 | `align = Tangent` (o de hoje, `align=1`) |

**Contagem (DERIVADA por `ferramentas/placar_conferencia.py`, re-medida em 2026-08-18):** 30 linhas — **P0 = 0** · **P0/P1 = 0** · **P1 = 8** · **P2 = 16** · ✅ fechadas **6** · ⛔ recusadas/refutadas **0**.

⚠️ **A ferramenta IMPRIME, ela não reescreve esta linha** — o `--write` não existe; ela é o ORÁCULO contra o qual esta contagem é conferida à mão. Rode `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` e compare.

Re-medir: `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` — ⚠️ **esta linha é DERIVADA da coluna `P` da tabela acima; não a edite à mão** (a contagem desta conferência envelheceu SEIS vezes, e a folha 13 chegou a contradizer a própria prosa três parágrafos abaixo).

---

## `SUPERAR:`

**1. A PORTA DE TEMPO — e ela é per-INSTÂNCIA. (o item da família)**
Cavalry tem, no Oscillator **e** no Noise, uma entrada *"Time (auto-conectado; desconectável)"* mais
*Time Offset* e *Time Scale* ([l.148/l.154](../referencia_pesquisa_cavalry.md)) — **um relógio por
behaviour**. Nós temos duas coisas que eles não têm, já construídas: **`value.time` com `stagger`
per-instância** (doc 80 — *"connected → a length-N field with a per-instance stagger"*) e a **regra de
broadcast 1→N** do `motion.drive` (doc 12). Uma porta `time` **opcional**, de tipo `VALUE`, em
`oscillator`/`noise`/`wiggle` — desconectada ⇒ `ctx.playhead()`, **byte-idêntico** (o precedente exato é
a porta `drive` do `motion.wave` e a `offset` do `motion.path`, as duas opcionais) — entrega de uma vez:

- **Time / Time Offset / Time Scale do Cavalry**, sem um knob novo. ⚠️ E a cerca do próprio arquivo
  **declara a sua condição de revogação**: *"sem uma porta de tempo externa, `sin(2π(s·t)f) ≡ sin(2π·t·(s·f))`"*
  — com a porta, o Time Scale deixa de ser identidade algébrica.
- **A cascata `Stagger → Shape Time Offset`** (a receita canônica do Cavalry, hoje **inexprimível**):
  `value.time(stagger) → oscillator.time`. E a nossa é **contínua** (o deles desloca a cópia inteira em
  frames) e **bit-exata sob scrub** (o ring de checkpoints, doc 11).
- **O LOOP pela mesma porta:** `value.time → value.wrap(Repeat/Mirror) → time` fecha o ciclo **por
  construção** (t e t+L são o mesmo número) — enquanto um `loop_len` por nó é um cross-fade, que
  aproxima. Um mecanismo em vez de N knobs (hoje só o `noise` tem `loop_len`; wiggle/oscillator/wave não).
- **O que nenhuma referência tem:** TD/AE/Cavalry/C4D dão um relógio por *objeto* ou por *cópia*; aqui
  o relógio é um **campo** — pode vir de áudio, de um `field.*`, de uma distância. *Um relógio por elemento.*

**2. O `motion.drive` escreve numa COLUNA NOMEADA — o mesmo widget dos dois lados.**
Hoje `value.attribute` **lê** qualquer coluna (widget `Channels` + *"Custom…"*) e o `drive` **escreve em
cinco**. Fechar a assimetria com o MESMO widget no destino transforma cada `value.*` num escritor
universal — inclusive de **`falloff`**, o mask que TODA behaviour desta família já multiplica. Isso é
literalmente o **Weight Transform** do C4D (*"ESCREVE a weightmap — effectors a jusante leem"*), que a
pesquisa marca **FALTA** ([c4d l.117](../referencia_pesquisa_c4d_fields.md)) — e nós chegaríamos lá por
**composição**, não por um effector especial. Nenhuma referência usa o mesmo controle para ler e para
escrever atributo.

**3. O par que falta é *ler componente* / *escrever coluna*, e ele é UM item.**
`motion.expression` não vê `P.x`; `value.attribute` só vê `|P|`; `motion.drive` não escreve coluna
arbitrária. As três frases são a mesma lacuna vista de três lados — e resolvê-la de um lado só deixa
metade. ⚠️ **Cross-family:** o consumidor é desta família, o produtor é VALUE (15) e o alvo é FIELD (10);
**conte uma vez** na tabela mestra.

---

## `CERCAS:`

1. ⛔ **`fade` (o *Strength Fade to Zero* do Cavalry) foi CONSTRUÍDO e REMOVIDO no mesmo dia**
   (`motion-oscillator/src/lib.rs:165-186`, smoke do Enio, doc 88 B3), com **três** defeitos, o primeiro
   MEDIDO: *expirava* (slider até 10 s ⇒ a partir de ~10 s de relógio toda a faixa entregava zero) ·
   *a régua era invisível* (a duração da composição **não existe neste nível**) · *era uma SEGUNDA PORTA*
   (`value.time → value.map_range → amplitude` já É um fade, com a régua visível no grafo).
   E o gate **`no_control_of_this_oscillator_expires_with_the_clock` guarda a CLASSE**: qualquer knob
   futuro cuja unidade seja *"segundos desde um zero que ninguém vê"* nasce vermelho.
2. ⛔ **`Time Scale` NÃO foi construído de propósito** (mesmo arquivo, `l.154-162`) — identidade algébrica
   com `frequency` **enquanto não houver porta de tempo externa**. ⚠️ A cerca traz a própria condição de
   revogação; é o SUPERAR 1.
3. ⚠️ **`lacunarity` é uma cerca, não um esquecimento** (`motion-noise/src/noise.rs:100`): *"exposing it is
   a rarely-touched advanced knob every tool defaults to 2, so it stays internal"*. Reabrir exige a
   evidência — e ela existe (Blender, TD *Spread*, Cavalry) —, mas a proposta é **reabrir uma cerca**, não
   preencher um vazio.
4. ⚠️ **O `loop_len` do `motion.noise` tem lei escrita e gate de convergência** (`noise/loop`): *"o
   cross-fade ingênuo — misturar `t` com `t − L` — NÃO fecha o ciclo"*, o peso é **smoothstep e não linear**,
   e a tolerância é **MEDIDA, não escolhida**. Portar o loop para os irmãos é copiar esta lei, nunca
   re-derivá-la.
5. ⚠️ **`motion.noise` é o de CAMPO e `motion.wiggle` é o de ÍNDICE** — a distinção está no doc-comment
   (*"Field, not jitter"*: gradient noise sobre `P·scale` × value noise sobre `(t, i)`). Um `stagger` no
   noise ou um `scale` espacial no wiggle apaga a razão de os dois existirem.
6. ⛔ **`CookError::SequentialInTimeScope`** — o cook **RECUSA** um nó sequencial (`spring`/`integrate`) a
   montante de um escopo de tempo, e a recusa é feita **antes de commitar o fio** no editor (auditoria
   2026-07-10). Toda proposta de retiming per-instância tem de dizer o que faz com isto.
7. ⚠️ **`motion.path`: o NOME é a referência, não um lookup** — o artista digita o nome que a forma tem na
   Hierarquia, e renomear a forma faz o nó seguir. Não propor picker por id. Forma ausente ⇒ **stream vazio**.
8. ⚠️ **`motion.expression`: a fórmula é TEXT PARAM** (doc 32) e `ph2d-expr` é **FROZEN** (ADR-0039) — o
   parser é único (`ph2d-expr-parse`, gate `the_motion_node_delegates_to_the_one_parser`). E o CLAUDE.md §5
   registra que **a AUTORIA de expressões da timeline foi RETIRADA** ([`Timeline/14`](../../Timeline/14_a_autoria_de_expressoes_foi_retirada.md))
   porque a folha era write-once — quem propuser um catálogo de receitas AQUI lê aquilo primeiro.
9. ⚠️ **`motion.drive`: a Opacity é CLAMPADA** (*"an alpha the renderer cannot use is not a brighter
   particle — it is a bug"*) e a **regra de broadcast 1→N** é a decisão load-bearing do doc 12.
10. ⚠️ **`amplitude` do `motion.noise` NÃO é declarada `FromChannel` de propósito** — *"este nó ainda não
    passou pela varredura de unidades"*; é dívida **nomeada**, não omissão.

---

## `O DOC 63 ERROU EM:`

1. **`motion.oscillator` → "Time Mode: Seconds/BPM"** — **JÁ EXISTE**. `time_mode` + `bpm` estão no
   `MANIFEST` (doc 88 B; conferido no código, não no doc). Riscar.
2. **`motion.noise` → "`loop_period`"** — **JÁ EXISTE** como `loop_len`, com unidade `Seconds` declarada e
   a lei do cross-fade gateada. Riscar.
3. **`motion.oscillator` → "`strength_fade_to_zero`"** — **REFUTADO COM GATE**. Foi construído, medido e
   removido; o gate guarda a classe. Propor de novo é desfazer decisão (cerca 1).
4. **O cluster "TIME local (`speed`·`offset` por nó)"** — **parcialmente refutado**: `speed` é identidade
   algébrica com `frequency` *enquanto não houver porta de tempo* (cerca 2). A cura certa não é o knob por
   nó — é a **porta** (SUPERAR 1), que entrega os três de uma vez e ainda os torna per-instância.
5. **`motion.stagger` → "Graph (curva D2)"** — o **enabler LANDOU** (`ph2d-curve` + `ParamWidget::Curve`,
   2026-07-25) e o `value.curve` já o usa. Deixou de ser *"D2 a construir"* e virou *"mais um contour"*, o
   padrão do `field.remap`. ⚠️ E a cadeia de 3 nós já o exprime EXATAMENTE hoje — o item cai de capacidade
   para ergonomia.
6. **A tabela §3.2 não cobre 5 dos 9 nós desta família** (`wave`, `drive`, `expression`, `time_remap`,
   `path`) — e é justamente no **`motion.drive`** que mora o gap mais estrutural (SUPERAR 2/3).
7. **Fora do 63, mesma classe de envelhecimento** (ambos citados nas linhas acima):
   `referencia_pesquisa_blender_gn.md` l.23 marca **Float Curve = FALTA** (existe: `value.curve`, widget
   `ParamWidget::Curve`); `referencia_pesquisa_c4d_fields.md` A2 marca **Time effector = PARCIAL
   ("verificar `motion.drive`")** — está fechado: `value.time` (doc 80) + `motion.drive` **é** o Time
   effector, e com `stagger` per-instância que o C4D não tem.

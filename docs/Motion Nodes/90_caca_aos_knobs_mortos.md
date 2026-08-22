# 90 · A CAÇA AOS KNOBS MORTOS — a tabela verificada

**Data:** 2026-08-22 · **Linha:** `line/motion-value` · **Pedido:** Enio, 2026-08-21, logo depois
de o smoke da cena `=73` ter encontrado um: *"Coloque no fim da fila de implementação auditoria
multiagêntica a busca de parâmetros mortos como esse."*

> **Um knob morto é um controle que o painel PINTA e que não muda a imagem.**

⚠️ **Este doc é o PRODUTO da auditoria, não a fila de trabalho.** Cada linha da §2 é um defeito
**confirmado à mão, com o mecanismo e o file:line**. As curas entram na fila uma a uma, cada qual
com o seu gate e a sua cena — ⛔ *uma acusação sem cena de smoke é uma mudança de comportamento
que ninguém olhou* (Grupo W, handoff de 2026-08-19).

---

## §0 — Os instrumentos (e o que cada um NÃO vê)

| instrumento | espécies que apanha | como correr |
|---|---|---|
| [`dead_knob_sweep.rs`](../../crates/ph2d-node-registry-init/tests/dead_knob_sweep.rs) | morto no braço default · inerte no modo que o painel mostra · descartado a jusante | `cargo test -p ph2d-node-registry-init --test dead_knob_sweep --release -- --ignored --nocapture` |
| [`knobs_declarados_nunca_lidos.py`](ferramentas/knobs_declarados_nunca_lidos.py) | declarado e nunca LIDO | `python3 "docs/Motion Nodes/ferramentas/knobs_declarados_nunca_lidos.py"` |

**A quarta espécie não existe neste catálogo:** os **613** params declarados têm **todos** uma
rota de leitura (`param("x")`, ou uma constante, ou um array de nomes varrido, ou a lista de um
`GpuKernel`). ⚠️ E isso é uma medição com **controle positivo** — o instrumento vê 613 de 613
leituras, logo o silêncio dele sobre a 614ª significaria alguma coisa. *A 1ª versão dele contava
menções entre aspas e imprimia "nenhuma acusação" por construção, porque todo param é nomeado
também no `ParamUiHint` que o pinta: **um instrumento cujo verde é garantido pela forma dos dados
não mede nada.***

---

## §1 — A LEI que atravessa cinco dos onze nós: o fBm nasce com UMA oitava

Três nós diferentes pintam `roughness`/`lacunarity`/`amp_mult` — os controles do caráter fractal —
e nos três eles são **provadamente inertes no estado em que o nó nasce**, porque
[`ph2d-fbm`](../../crates/ph2d-fbm/src/lib.rs) aplica `amp *= gain` e `px *= lacunarity`
**depois** de somar a oitava corrente: com `octaves = 1` (o default dos três) nenhum dos dois toca
a saída.

⚠️ **É uma lei, uma cura e três nós** — e o sintoma é o pior possível para quem aprende a
ferramenta: *dois sliders inertes lado a lado ensinam que o **bloco** de ruído não funciona, e não
que falta subir um terceiro número.*

---

## §2 — Os defeitos CONFIRMADOS (19 params · 11 nós)

Todos são da mesma espécie: **inerte no modo/estado que o painel mostra, e sem `ParamGate`**.

| nó | param(s) | por que é inerte | file:line do mecanismo |
|---|---|---|---|
| `motion.stagger` | `ease_dir` | `ease_curve` nasce em `Linear`, e `Linear` devolve `t` antes de olhar a direção | [`ease.rs:53`](../../crates/ph2d-node-motion-stagger/src/ease.rs) |
| `motion.tint` | `r2` `g2` `b2` `a2` | a cor `End` só é lida no braço `Gradient`; `mode` nasce em `Solid`. ⚠️ **O hint declara o defeito por escrito** (*"it paints regardless — v1 has no per-mode row hiding"*) | [`lib.rs:387-399`](../../crates/ph2d-node-motion-tint/src/lib.rs), confissão em `:456` |
| `motion.wiggle` | `amp_mult` | é o `gain` do fBm — §1 | [`ph2d-fbm/src/lib.rs:159-178`](../../crates/ph2d-fbm/src/lib.rs) |
| `force.wind` | `lacunarity` `roughness` | §1 — e a crate **não chama `register_param_gates` nenhuma vez** | [`lib.rs:314-333`](../../crates/ph2d-node-force-wind/src/lib.rs) |
| `value.noise` | `roughness` `lacunarity` | §1 — `PARAM_GATES` esconde `feature`/`jitter` e esquece estes dois | [`lib.rs:356-366`](../../crates/ph2d-node-value-noise/src/lib.rs) |
| `motion.emitter` | `shape_w` `shape_h` | `birth_offset` sai cedo com `Shape::Point` (o default). ⚠️ **O `dir_mode` ao lado é gateado pela MESMA condição** — estes dois ficaram para trás | [`birth.rs:52`](../../crates/ph2d-node-motion-emitter/src/birth.rs) vs [`params_ui.rs:397-423`](../../crates/ph2d-node-motion-emitter/src/params_ui.rs) |
| `motion.boids` | `avoid_radius` `lookahead` | `avoid = 0` (default) esvazia a lista de obstáculos e os dois só são lidos lá dentro. ⚠️ **O doc-comment já o diz** (*"`0` desliga — e desliga os três params da família"*) e o nó **não registra gate nenhum** | [`avoid.rs:13-14,55-58`](../../crates/ph2d-node-motion-boids/src/avoid.rs) + [`lib.rs:616-643`](../../crates/ph2d-node-motion-boids/src/lib.rs) |
| `fx.rgb_split` | `strength` | `offsets` devolve o deslocamento uniforme **antes** de ler `strength`; ela só vale em `Aberration`, e `mode` nasce em `Split` | [`lib.rs:133-144`](../../crates/ph2d-node-fx-rgb-split/src/lib.rs) |
| `value.instance_field` | `seed` | só o braço `Random` o lê; o default é `Ramp`. ⚠️ **O doc-comment AFIRMA que ele já é gateado, e é falso** — `key` e `unique_per_node` somem, o `Seed` fica | [`lib.rs:219,391-405`](../../crates/ph2d-node-value-instance-field/src/lib.rs) |
| `value.map_range` | `clamp` | `Smooth`/`Smoother` clampam **incondicionalmente**; o doc até nota que *"o Blender cinzenta a caixa Clamp para exactamente estes dois"* | [`lib.rs:148-156,330-334`](../../crates/ph2d-node-value-map-range/src/lib.rs) |
| `value.step` | `width` | o braço `Hard` — o **default** — decide por `v >= threshold` e nunca lê a largura; a crate não tem `ParamGate` nenhum | [`lib.rs:110-117,174-177`](../../crates/ph2d-node-value-step/src/lib.rs) |

### ⚠️ O par que a cura tem de tratar JUNTO

`fx.rgb_split::x` e `::y` são o **espelho exacto** do `strength`: em `Aberration` é a eles que o
`offsets` não olha ([`lib.rs:142-144`](../../crates/ph2d-node-fx-rgb-split/src/lib.rs)). *Gatear só
o `strength` troca um par de knobs mortos por outro.*

---

## §3 — O que NÃO é defeito (e por que a distinção é o trabalho todo)

*Inerte não é morto.* Estes foram acusados pela sonda e **absolvidos** — cada um tem o desenho
que o justifica:

| caso | por que está certo |
|---|---|
| `motion.noise::range_mode` · `motion.oscillator::range_mode` | com os defaults, a régua alternativa é **byte-idêntica por construção** (`gain_offset_for_range` devolve a identidade); e o painel já troca o par `amplitude` ↔ `min`/`max` por `ParamGate` |
| `value.gain::mode` · `value.step::mode` · `value.smooth::weight` | um SELETOR só se distingue com a magnitude que ele modula fora do neutro — e ⛔ um seletor **não pode** ser gateado pela magnitude que ele modula, senão desaparece exactamente quando o artista vai procurá-lo |
| `motion.transform::pivot_mode` | escalar por `1` (o default) em torno de qualquer ponto **é** a identidade — não há nada a pivotar |
| `value.mix::clamp_result` | um clamp que só morde fora da faixa é a natureza de um clamp |
| `value.pattern::v2..v7` | **exceção deliberada e declarada**: são slots que o artista tem de preencher ANTES de subir `steps`; escondê-los seria pior |

---

## §4 — Os OITO pontos cegos da sonda (medidos, cada um com o caso que o revelou)

⚠️ **Esta lista é o valor durável deste doc.** Uma bancada não prova a ausência de um efeito, só a
ausência dele **naquela bancada** — e cada linha abaixo custou um lote de falsos positivos.
Quem correr a sonda outra vez lê isto primeiro.

| # | o ponto cego | o caso que o revelou | estado |
|---|---|---|---|
| 1 | **fixture no PONTO FIXO** — alimentar VALUE com o `debug.const` (`[1.0]`) | `value.gain::strength` acusado, e `1.0` é ponto fixo de toda curva de ganho | ✅ curado (cadeia `motion.grid → value.instance_field`) |
| 2 | **modo testado com a magnitude no NEUTRO** | `value.gain::mode`: com `strength = 0` os dois modos são a identidade | ✅ curado (contexto `magnitudes-quentes`) |
| 3 | **laçar TODAS as portas do tipo da saída** esvazia o nó | `motion.trail` tem `in` **e** `state`: laçar os dois deixou 5 params sem fonte | ✅ curado (`loop_port` é um índice, não um booleano) |
| 4 | **uma fonte que VARIA pode ser pior que uma constante** | porta lida por `.first()`: o elemento 0 de uma rampa é `0.0`, e `amount = 0` desligou os **13** knobs do `motion.spline_wrap` | ✅ curado (bancada irmã com `const_first`) |
| 5 | **ler só o ÚLTIMO quadro** é cegueira temporal | `pulse.adsr::attack_shape` e `release_shape` já tinham entregado o mesmo sustain | ✅ curado (compara o TRAÇO dos 48 quadros) |
| 6 | **12 quadros** acabam antes de um envelope | o release do `pulse.adsr` só começa aos `0,35 s` = 21 quadros | ✅ curado (`TICKS = 48`) |
| 7 | **`min`/`meio`/`max` são CONGRUENTES num param angular** | `−360°`, `0°` e `+360°` são o mesmo ângulo: **nenhuma rotação do catálogo podia ser provada viva** | ✅ curado (os TERÇOS, quatro pontos) |
| 8 | **cadeias idênticas por porta** dão `a ≡ b` ao bit | `value.math::epsilon` compara `\|a−b\|`; no `field.shape` a nuvem e o polígono eram o MESMO conjunto | ✅ curado (rotação dos candidatos por índice de porta) |
| — | **ligar as portas OPCIONAIS** muda a lei do nó | meia dúzia de nós dizem que um param só vale com a porta homónima **desligada**; e um `reset` alto congela um contador | ✅ curado (bancada irmã que lê `required_inputs`) |
| ⚠️ | **o efeito pode não estar nas COLUNAS** | um `fx.*` de raster produz uma imagem | ⛔ **por curar** — estes saem como `BANCADA-SUSPEITA` e não acusam nada |
| ⚠️ | **um nó pode precisar de uma CENA** | `motion.look_at` em `Object`/`Cursor` resolve por `ctx.external` | ⛔ **por curar** — idem |

### A CALIBRAÇÃO — o que as oito curas valeram, medido

| | sonda v1 (antes da verificação) | sonda v2 (com as oito curas) |
|---|---|---|
| `VIVO` | 362 | **430** |
| `SO-EM-MODO` | 67 | 62 |
| `MORTO` | **102** | **57** |
| `BANCADA-SUSPEITA` | 68 | 50 |

⚠️ **E os dois lados foram conferidos, não só o que encolheu.** Os falsos positivos que os
verificadores derrubaram (`motion.spline_wrap::height_scale`, `motion.trail::fade`,
`motion.clone::angle`, `value.math::epsilon`, `pulse.adsr::release`) **passaram sozinhos a
`VIVO`/`SO-EM-MODO`** — e os **15** defeitos da §2 **continuam todos acusados**, todos com a
coluna `gate?` vazia, que é a assinatura exacta da espécie. *Uma sonda que só encolhe a lista
pode estar a ficar cega; o que a calibra é ela largar os falsos **e segurar os verdadeiros**.*

⚠️ **Um resíduo conhecido:** `motion.cull::amount` continua `MORTO` e **é** falso positivo — o
param só vale com a porta homónima desligada, e a bancada irmã que desliga as opcionais depende
de o nó declarar `required_inputs`, que esta crate não declara. *A cura do ponto cego é tão boa
quanto a declaração que ela lê.*

### ⚠️ E o ponto cego SIMÉTRICO, que é o pior

Um param pode estar **vivo na sonda e morto para o artista**, se o valor que o acorda for
inalcançável pela UI. Achados de passagem:

- `value.curve::factor` extrapola de propósito acima de `1` (com gate a defendê-lo), e o slider
  para em `1.0` sem `ParamHardMax` — **a caricatura da curva não é digitável**.
- `pulse.counter::reset_to` tem slider `0..32` enquanto o `count_max` irmão ganhou
  `ParamHardMax = 1e6`: **resetar um contador de 1000 para 500 é inalcançável**.
- `pulse.counter::step` tem slider `−8..8` e nenhum hard max: **contar de 10 em 10 não se digita**.
- `motion.spring::tension` tem `ParamHardMax = 1_600_000` contra um slider que para em `60`, e
  acima de ~1,6 M o guard de NaN prega a mola no alvo: **a região digitável tem um topo que parece
  quebrado**.
- `motion.clone::angle` e `motion.noise::rotation` vão de `−360°` a `+360°`: **metade do curso é
  uma segunda volta idêntica à primeira**.

---

## §5 — Os achados que NÃO são knobs mortos mas saíram da mesma varredura

- ⚠️ **`motion.spline_wrap::amount` é uma porta por-ELEMENTO lida no elemento 0**
  ([`lib.rs:159-161`](../../crates/ph2d-node-motion-spline-wrap/src/lib.rs)). O gesto óbvio — ligar
  um `value.instance_field(Ramp)` para o embrulho crescer ao longo do layout — dá ao nó inteiro o
  valor do elemento 0, que numa rampa é exactamente `0.0`, e **a curva inteira deixa de existir**.
  O irmão `motion.lattice` (jitter) tem a mesma forma. ⚠️ O `motion.cull` faz o mesmo mas
  **DECLARA-o** (`ColumnAccess::ReadBroadcast`) — a cura mínima é declarar; a cura certa é ler a
  coluna.
- ⚠️ **`motion.wave` escreve `size` a partir de `z.abs()`** — o canal default do nó **descarta o
  sinal** da altura, e nenhum param controla isso.
- ⚠️ **A família que lê `vel` é silenciosamente inerte fora de uma zona de simulação**
  (`sim.collide::restitution`/`friction`, `force.buoyancy::drag`, `force.attractor::lead`). Os
  doc-comments dizem-no; o painel não.
- ⚠️ **`motion.kaleidoscope` IGNORA o `falloff`** — é o único deformer da família que não lê o
  contrato de campo, e o gate `falloff_declaration` **não o vê**, porque ele só reprova quem LÊ sem
  declarar, nunca quem não lê.
- ⚠️ **Contradições doc × código** encontradas de passagem: o hint do `motion.tint` confessa o
  próprio defeito; o doc do `value.instance_field` afirma que o `Seed` é gateado e não é; a nota do
  `field.remap` diz que `curvature`/`steps` não são escondidos e eles são (a nota envelheceu na wave
  que a contradisse).

---

## §6 — O molde da cura

Todos os 19 pedem a mesma coisa, e ela já existe no repo — o `motion.transform` resolve o caso
idêntico em duas linhas
([`lib.rs:603-614`](../../crates/ph2d-node-motion-transform/src/lib.rs)):

```rust
ParamGate { param: "<o inerte>", when: "<o seletor>", values: &[/* os índices em que ele age */] }
```

Para os que dependem de um LIMIAR e não de um índice (o `avoid > 0` do `motion.boids`, o
`octaves >= 2` do fBm), o mecanismo é o `ParamGateAbove`, já usado pelo `motion.shape`
([`ui.rs:332`](../../crates/ph2d-node-registry/src/ui.rs)).

⚠️ **A cura de cada um precisa da sua cena de smoke**, e a cena tem de mostrar as duas metades: o
knob a aparecer quando age e a sumir quando não age. *Um gate que só prova a ausência prova metade.*

---

## §7 — A FILA DAS CURAS (o plano, com o gate exacto de cada uma)

⚠️ **O índice errado é PIOR que o defeito.** Um `values` mal escrito esconde o knob exactamente
quando ele age, e aí o artista não tem gesto nenhum para o alcançar. Por isso a tabela abaixo traz
os rótulos **lidos do código**, e por isso a prova (§7.2) é **derivada por medição**, nunca escrita
à mão.

### §7.1 — As duas famílias de gate

**A. `ParamGateAbove { param, when, above }`** — aparece quando `when > above` (estrito). É a
família do limiar contínuo, onde arredondar a inteiro não diz nada.

| nó | param(s) | `when` | `above` | faixa do `when` (lida) |
|---|---|---|---|---|
| `motion.wiggle` | `amp_mult` | `octaves` | `1.0` | `1..MAX_OCTAVES`, IntSlider |
| `force.wind` | `lacunarity` · `roughness` | `octaves` | `1.0` | `1..4`, IntSlider |
| `value.noise` | `roughness` · `lacunarity` | `octaves` | `1.0` | `1..8`, Slider |
| `motion.boids` | `avoid_radius` · `lookahead` | `avoid` | `0.0` | `0..20`, Slider |

**B. `ParamGate { param, when, values }`** — `values` são os índices do enum em que o param
**APARECE**.

| nó | param(s) | `when` | rótulos do `when` (lidos do código) | `values` |
|---|---|---|---|---|
| `motion.stagger` | `ease_dir` | `ease_curve` | `Linear·Quad·Cubic·Quart·Quint·Circ·Back·Bounce` | `[1,2,3,4,5,6,7]` |
| `motion.tint` | `r2` (âncora do swatch `End`) | `mode` | `Solid·Gradient` | `[1]` |
| `fx.rgb_split` | `strength` | `mode` | `Split·Aberration` | `[1]` |
| `fx.rgb_split` | `x` · `y` | `mode` | idem | `[0]` |
| `value.instance_field` | `seed` | `mode` | `Index·Ramp·Random` | `[2]` |
| `value.map_range` | `clamp` | `interpolation` | `Linear·Stepped·Smooth·Smoother` | `[0,1]` |
| `value.step` | `width` | `mode` | `Hard·Smooth·Smoother` | `[1,2]` |
| `motion.emitter` | `shape_w` · `shape_h` | `shape_mode` | `Point·Disc·Ring·Rect` | `[1,2,3]` |

⚠️ **O `motion.tint` gateia SÓ a âncora.** Os outros três canais (`g2`/`b2`/`a2`) já não pintam
linha própria — o construtor do painel dobra-os no swatch (`consumed`), e é a âncora que emite a
row. Gatear os quatro seria escrever três linhas que não decidem nada.

### §7.2 — A PROVA, em duas metades (nenhuma delas basta sozinha)

| metade | pergunta | onde vive |
|---|---|---|
| **A — o knob não AGE** | fora do gate, mudar o param não muda um bit da saída; dentro do gate, muda | `crates/ph2d-node-registry-init/tests/` — é onde o registry inteiro existe |
| **B — o painel não PINTA** | fora do gate a row não é construída; dentro, é | `shells/desktop/` — é onde o construtor de rows vive |

⚠️ **A metade A é DERIVADA: ela cozinha o nó em cada modo e mede.** Escrever o `values` no teste
seria repetir o mesmo palpite duas vezes e chamar-lhe prova. Se um índice desta tabela estiver
errado, é a metade A que reprova — e é para isso que ela existe.

⚠️ **E a metade A tem de exigir os DOIS lados.** Um teste que só verifique *"é inerte fora do
gate"* passa num gate que esconde o knob sempre (`values: &[]`) — o defeito oposto, e pior. A
asserção é um par: **inerte fora, vivo dentro de pelo menos um**.

### §7.3 — Ordem

1. Os gates (11 crates, side-metadata — ⛔ nenhum toca o `NodeManifest` congelado, §6 do CLAUDE.md).
2. A prova A, sobre os 19.
3. A prova B, sobre os 11 nós.
4. Uma cena de smoke em que o Enio veja o painel a mudar de tamanho ao girar o modo.

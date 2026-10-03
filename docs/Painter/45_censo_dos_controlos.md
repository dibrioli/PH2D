# 45 — O censo dos controlos do Painter (2026-10-03)

> Ordem do dono (2026-10-02): *«Detectar cada parâmetro morto ou mal utilizado em cada seção do painel
> Painter para cada um dos modos de pintura. Precisamos que cada parâmetro funcione em cada um dos
> módulos.»* Este doc é a **etapa (a)**: a tabela medida. A etapa (b) (as curas) só remove um
> controlo da tela depois da decisão do dono (§4).
>
> Estado por linha: **vivo** · **morto** (o meio não lê) · **mudo** (nada chega à ferramenta) ·
> **inerte até X** (age, mas só com a pré-condição X — e a tela não o diz) · **mal utilizado** (age
> ao contrário do nome, ou destrói) · **não-pixel** (age sobre outra coisa que a tinta, por desenho).

## §1 — O instrumento (o que se mede, e como a régua mentiu três vezes)

Sonda: [`censo_dos_controlos.rs`](../../crates/ph2d-panel-painter-layers/tests/it/censo_dos_controlos.rs)
(`#[ignore]`, `--ignored --nocapture`); nomes e classes:
[`ferramentas/censo_dos_controlos/`](ferramentas/censo_dos_controlos/) (`nomes_dos_ids.py` →
`classifica.py`, `arma.py`).

1. **A população é a TELA.** O painel pinta-se como a ponte o publica (`set_current_brush`,
   `set_current_dock_shows_layers(false)`), com todas as secções abertas, num ecrã de `1600 × 6000`.
   Cada id com rectângulo é um gesto pela porta do PONTEIRO: clique no centro, número digitado no
   chip (`type_into_number`, 60 % do caminho até à ponta mais longe da faixa), slider, arrasto da
   pega de uma curva, opção de menu (o menu abre-se e a opção clica-se). O barramento vai à
   ferramenta (`Tool::handle_panel_event`), o painel re-pinta, dois quadros.
2. **A exploração desce.** Um gesto que faz APARECER ids novos abre um estado; o censo visita-o
   (BFS, profundidade 3). Cada id mede-se no 1.º estado onde aparece. Cobertos: Digital **508**,
   Aquarela **482**, Impasto **508**, Wet Paint **508** gestos.
3. **O traço de fábrica** (`risca`): tela `128²` branca com um bloco vermelho; traço 1 ondulado de
   `x = −8` a `136` (atravessa as duas bordas verticais), pressão `0,3 → 1`; traço 2 numa SEGUNDA
   COR, diagonal de `y = 140` a `−12` (as bordas horizontais), pressão `1 → 0,4`, caneta inclinada;
   30 quadros parados. Um `on_tick` por amostra e a drenagem da pré-visualização (a ordem da ponte).
4. **Três classes por gesto:** o instantâneo do pincel (`BrushSettings`) mudou? chegou algo à
   ferramenta? a imagem mudou? ⇒ `já-era` (a opção já escolhida, o Reset na fábrica) separa-se de
   `CANDIDATO` (ajuste mudou, imagem igual).
5. **A pré-condição PROCURA-SE** (`procura_as_pre_condicoes`, `CENSO_ARMAR=1|2`): para cada
   candidato, tenta-se como pré-condição um traço já pintado (em cada posição da sequência) e cada
   gesto vizinho do cartão (±160 px), depois os do estado inteiro (2 opções por menu); o primeiro
   que faz a imagem mudar — contra a base armada com ele — é a pré-condição. Nenhum ⇒ **morto**.

**⛔ As três réguas que mentiram nesta medição (cada uma leu «morto» onde não havia):**

| régua | o que leu | a cura |
|---|---|---|
| Wet Paint sem dormir | Water, Flow, Gravity, Tilt… «mortos» (a água anda numa thread ao relógio REAL e nunca andou) | relógio FIXO, abaixo |
| Wet Paint a dormir 16,7 ms | ao relógio real, sob carga, **1 148** texels entre duas corridas iguais ⇒ um «vivo» podia ser ruído | `PainterTool::set_wet_relogio_fixo` (feature `test-support`): o tique avança `dt / STEP_S` passos síncronos pela porta dos gates de física (`wet_step_sync`); ruído **0** em 16 corridas |
| dois traços da MESMA cor | Pickup do Wet Paint «morto» (apanhar azul de cima de azul) | o 2.º traço noutra cor |

E uma do próprio instrumento: o Stop Pos das rampas leu-se «mudo» porque o stop SELECCIONADO vive num
`thread_local` do painel e passa de ensaio para ensaio — o valor escrito era o que o chip já
mostrava, e um commit igual não emite nada. O alvo do número recalcula-se agora no momento do gesto.

## §2 — A tabela (só o que NÃO é vivo; o resto mudou a imagem)

Leitor = onde a decisão mora (`file:line` no ramo de hoje). `D` Digital · `A` Aquarela · `I` Impasto
· `W` Wet Paint.

### §2.1 — Defeitos (mal utilizado / destrói)

| controlo | meio | medido | leitor | cura |
|---|---|---|---|---|
| ferramenta **Blow** (e Smear) da água | W | **pânico** com o 1.º carimbo fora da tela (`tools.rs:330`, `snap_w * snap_h`); release: pedido de memória astronómico | `ph2d-wet-paint/src/tools.rs` `rect_around` | **CURADO** (`9dbf23ba2`): `rect_around_nao_vazio`; gate `a_dab_outside_the_grid_touches_nothing_in_any_tool`, mutação da Smear sangra |
| **Accumulate + Space Attenuation** juntos | W | o traço pinta **2 428 → 26** texels | `ph2d-painter-brush/src/stroke.rs:268,302` (`space_overlap_factor`, a lei do Blender para alfa que ACUMULA) aplicada ao carimbo da água; o Wet Paint não lê `accumulate` (`wetpaint/dab_route.rs`) | decisão §4-1 |
| **Reset** da secção Watercolor · Impasto · Wet Paint | A I W | na fábrica muda a pintura (**3 934 · 3 853 · 3 590** texels): o Reset DESLIGA o meio (`media 1/2/3 → 0`) | `watercolor_settings.rs:500` (`b.watercolor = d.watercolor`), `impasto_settings.rs:490`, `reset_brush_wetpaint` — e três gates o AFIRMAM (`watercolor_settings/tests.rs:229`, `impasto_body/deposit.rs:627`, `wetpaint/tests.rs:861`), escritos quando o meio era uma caixa DENTRO da secção | decisão §4-6 |
| **Reset** da Shape | I | `3 730` texels na fábrica: Falloff `2 → 0` (o Reset repõe o default GERAL, não o do meio) | `jitter_settings.rs:45` | decisão §4-6 |
| **Dry Time** | A | `0` texels em 1,3 s, com e sem Wet/Smudge/Charge armados | só o véu de humidade (`canvas_wet` → `painter_bridge_wetness.rs:30`); nenhum depósito o lê | decisão §4-4 (o nome promete que a tinta seca mais devagar) |

### §2.2 — Mortos num meio (o painel mostra, o meio não lê)

A procura confirma-os da mesma forma: o único gesto que os acorda é **trocar o meio** (`PAINTER_BRUSH_MEDIA[0/4]` ou o preset Digital).

| controlo | meio(s) | leitor ausente |
|---|---|---|
| **Blend** (24 opções) | W | `wetpaint/dab_route.rs:24` (`stamp_dabs_wetpaint`) não lê `brush.blend` |
| **Composite Brush** (Enable, `+`, Size, Strength, Hardness, remover) | W | idem — a pilha não chega ao motor da água |
| **Accumulate** | W | idem (e com o Space Attenuation destrói, §2.1) |
| **Space Attenuation** | A · I · W | o par dele (Accumulate) não aparece na A nem no I; no W não é lido |
| **Solid** (linha) | A · W | `solid_deposit.rs:58`: `solid_owns_the_gesture` exclui aquarela; a água não tem ramo |
| **Line Type Sketchy** (+ Reach, Density, Width, Opacity, Magnetify) | A · W | os fios (`stroke/threads.rs`) só os deposita o caminho digital. **No I age só com Solid ligado.** |
| **Line Type Wire** (+ History, Connection) | A · W | idem |
| **Ribbon → Rungs** | A · W | o Ribbon age; o número de degraus não |
| **Shape Color Ramp** inteira (Enable, `+`, `−`, Invert, B/W, Mode, Interp, pegas, Stop Pos) | A · W | a aguada lê só a LUMINÂNCIA da Shape (`watercolor_accum.rs:245`); a água idem |

### §2.3 — Inertes até uma pré-condição (a tela não o diz)

| controlo | meio(s) | só age com | medido armado |
|---|---|---|---|
| Taper **Tip** · **Opacity** | D I W | o comprimento do Taper > 0 (a pega da curva) | `2 370` · `1 022` texels (D) |
| **Accumulate** | D | Strength < 1 (sem ele não há tecto para ultrapassar) | `3 118` |
| **Space Attenuation** | D | Accumulate ligado (`spec/queries.rs:324`) | `3 499` |
| **Jitter Unit** | todos | Jitter > 0 | `6 876` |
| **Dash Length** | todos | Dash Ratio < 1 | `1 628` |
| **Roughness** do papel · **opções do Paper** · parâmetros do Paper | todos | Relief > 0 (`substrate_relief.rs:118`); na A também um papel escolhido | `10 975` (D) · `11 017` (A) |
| **Jitter Rotate** | todos | uma Shape não redonda; na A, Automatic desligado | `3 433` (D) · `2 986` (A) |
| 4.º parâmetro do Grain/Shape **Noise** (Roughness) | todos | Detail > 0 (uma oitava só não tem persistência) | `2 875` |
| **Grain** (as 28 opções) | A | *Same as Paper* desligado | `1 720–3 507` |
| **Spread** | A | Wet > 0 ou Dilution | `3 339` · `2 722` |
| **Pull** | A | Charge < 1 (o pincel volta a apanhar) | `669` (fraco: máx. 8 níveis) |
| **Automatic** (Shape) | A | Smudge > 0, ou uma Shape escolhida | `566` |
| **Depth Source = Grain** | I | um Grain escolhido | `2 711` |
| **Enable** da luz 2–4 | I | a luz seleccionada (o chip) | `3 769` — **o Bug #15 está curado** (gate `seam_impasto_rig.rs:214`) |
| Depth · Body · Shine **na tinta já pintada** | I | *Adjust Last Stroke* ligado — **por desenho** | `2 267` · `1 189` · `662` (desligado: `0`) |
| ferramentas de esculpir (Smooth, Flatten, Chisel, Layer, Inflate) e Knife/Plow — os parâmetros delas | I | tinta com volume já na tela | `482–2 006` |
| **Erase** (o número) | W | tinta molhada na tela e a ferramenta Erase | `1 026` |
| **Offset** (linha) | todos | tinta já na tela | `2 846` (D) |
| **Stroke Op +/−** | todos | o método 7 | `4 841` |
| eixo **Custom** da Simetria | todos | um eixo escolhido | `5 053` |
| **Shape/Grain = Image** | todos | uma imagem carregada (o app abre o selector) | — |
| Modos da **Color Ramp** (Mode, Alpha Mode, B/W) | D I | stops com COR (a rampa de fábrica é cinzenta) | a sonda não escolhe cor |
| **Offset Trim** | todos | não armado pela procura (pede offset ≠ 0 e uma linha que se cruza) | — |

### §2.4 — Não-pixel, por desenho (correctos)

*Sync with other tools* (liga as ferramentas entre si) · *Repeat Image* (pré-visualização 3×3) ·
*Grid Show* · *Line Dimensions* (sobreposições) · *Wet Preview* (o véu de humidade) · *Tuning*
(abre o painel lateral) · os chips 1–4 das luzes (escolhem QUAL luz se edita) · o menu ao lado do `+`
do Composite (escolhe o que o `+` acrescenta) · *Falloff Add* (o ponto novo nasce sobre a curva) ·
*Falloff Remove* (sem ponto escolhido não há o que tirar) · *Stop Index* e as amostras de cor
(selecção/selector) · os cabeçalhos e setas das secções.

## §3 — Abertos desta medição

- **O gate permanente** (a etapa (b) pede-o): a sonda vira teste com a lista §2 escrita — controlo
  novo sem motivo = vermelho; entrada que passa a viver = vermelho (a lista só encolhe). Fica para
  depois das decisões §4, porque cada decisão muda a lista.
- **A procura completa** (`CENSO_ARMAR=2`) não cabe nos 30 min do `ph2d-run` sob carga no Impasto e
  no Wet Paint; ali correu só a dos vizinhos, e os mortos que sobraram leram-se à mão (§2).
- **Composite: as pilhas atravessando os meios** não foram exploradas além da 1.ª camada.
- **As ferramentas da barra lateral (Smudge, Blur, Clone, Fill…)** mudam o painel; não foram
  exploradas como estados próprios (só o que o painel do pincel oferece).

## §4 — Decisões do dono (antes de qualquer remoção)

1. **Wet Paint: Accumulate e Space Attenuation.** Esconder os dois no Wet Paint (como a Aquarela e o
   Impasto já fazem com o Accumulate), ou fazer o Wet Paint ignorá-los. Hoje, ligados juntos, o
   pincel deixa de pintar.
2. **Wet Paint: Blend, Composite Brush, Solid, Sketchy/Wire, Rungs.** Esconder no Wet Paint, ou
   ensinar a água a cada um (trabalho grande por item).
3. **Aquarela: Solid, Sketchy/Wire, Rungs, Shape Color Ramp; e o Space Attenuation na Aquarela e no
   Impasto** (o par dele, o Accumulate, não aparece lá). Idem.
4. **Aquarela: Dry Time.** Hoje só decide quanto tempo dura o brilho de «molhado». Renomear/mover
   para junto do Wet Preview, ou fazê-lo agir na tinta (molhado-sobre-molhado dependente do tempo).
5. **As pré-condições sem dica (§2.3).** Proposta técnica: a linha fica ESMAECIDA, com a dica do que
   a liga, enquanto a pré-condição for falsa (DIRETIVA §2: *«pré-condição ausente = UI mostra
   desabilitado»*). Não remove nada.
6. **O Reset de cada meio.** Hoje o Reset da Aquarela/Impasto/Wet Paint DESLIGA o meio (o pincel
   volta a Digital) e o da Shape repõe o Falloff do Digital. Recomendo: o Reset repõe os valores de
   fábrica DO MEIO e o meio fica escolhido.

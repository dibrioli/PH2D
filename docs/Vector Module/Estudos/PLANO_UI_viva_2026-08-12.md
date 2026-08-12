# PLANO — A UI VIVA: substrato, carácter, scrub e tether

> **Companheiro de** [`ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md`](ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md).
> O estudo **mediu e triou**; este documento diz **como**, com as possibilidades consideradas, o
> custo de cada uma, os algoritmos escritos por extenso e os gates red-first.
>
> ⚠️ **Nada aqui começa sem ordem explícita do Enio.** O plano é escrito ANTES de uma linha de
> código, por pedido dele (2026-08-12).

---

## §0 — As quatro coisas que a medição já decidiu, e que encolhem o plano

Escritas primeiro porque **cada uma mata um ramo de projeto** que este plano teria de discutir.

| medição | consequência |
|---|---|
| **A mola já sub-passa em `STEP = 1/240` fixo** e consome o `dt` real em fatias (`spring.rs:64-66`) | ⛔ **morre** o ramo "solução analítica do oscilador amortecido". A independência de taxa de quadros **já está resolvida**, com o motivo escrito. |
| **`SpringState::resuming(v)` existe e é gateado** (`a_resumed_spring_carries_the_velocity_a_curve_would_have_dropped`) | ⛔ morre o ramo "escrever herança de velocidade". A F3 do estudo é **reuso**, não construção. |
| **`Machine::go_to` já enuncia a lei da interrupção** (*"o caminho começa na pose VIVA, nunca na autorada"*) | o chrome copia a **lei**, em 1 dimensão em vez de N poses. |
| **A shell nunca prendeu o cursor** (zero `set_cursor_grab` em `shells/desktop/src/`) | ⚠️ o scrub numérico **não pode prometer** arrasto infinito sem uma sonda de plataforma primeiro (§5.2). |

⇒ **F0 + F1 + F2 são plumbing de peças provadas, não motor novo.** É o melhor achado possível para
um plano: a wave de maior alcance é também a de menor risco.

---

## §1 — F0: O SUBSTRATO (`UiMotion`)

O degrau que desbloqueia o eixo inteiro. Sem ele nada da §11 do estudo é exprimível.

### 1.1 Possibilidades para ONDE mora o estado contínuo

| # | onde | custo | veredito |
|---|---|---|---|
| A | **dentro do `InteractiveState`** (o store) | zero estrutura nova | ⛔ **NÃO.** Aquele é o estado **semântico**, e dezenas de gates comparam-no. Misturar animação faz cada gate passar a ver ruído, e um `assert_eq!` de estado passaria a depender de *quando* foi lido. |
| B | **mapa PARALELO no store**, keyed por `NodeId` | um `BTreeMap` | ✅ **SIM** — é o idioma que este repo já usa três vezes para estender sem colidir (`bypassed_subgraphs` é `BTreeSet` paralelo, `node_text_params` é mapa paralelo). |
| C | no pintor | — | ⛔ o pintor é sem estado por quadro, por desenho. |
| D | numa crate-folha nova | isolamento | ⛔ não há segundo consumidor; um módulo em `editor-core` é o tamanho certo. Reavaliar se a shell de jogo nascer. |

### 1.2 A estrutura

```rust
/// Só o que se MOVE agora. Um app parado tem este mapa VAZIO.
pub struct UiMotion {
    live: BTreeMap<NodeId, Track>,
    character: UiCharacter,   // Discreto | Expressivo   (§2)
    reduced: bool,            // eixo INDEPENDENTE       (§2.2)
}

struct Track {
    from: f32, to: f32,       // as duas pontas do percurso ACTUAL
    s: SpringState,           // x∈[0,1] normalizado, v — a peça que já existe
    drive: Drive,             // Mola | Curva{dur, easing}
    role: Role,               // quem decide a lei é a PORTA, não o chamador
}
```

⚠️ **`Role`, e não `duration`.** O chamador diz **o que a coisa É** (`Travel · Fade · Number ·
Decoration`), nunca **como se move**. Um chamador que passasse uma duração teria **re-implementado o
carácter** no sítio dele, e no dia seguinte metade do app estaria em Expressivo e metade não.

### 1.3 O algoritmo

```
retarget(id, to, role):
    match live.get(id):
        None if to == valor_semântico_actual  -> NO-OP        # o caso comum: custo ZERO
        None                                   -> Track{from: actual, to, s: at_rest(), role}
        Some(t)                                -> # INTERRUPÇÃO: a lei do Machine
                                                  from = value(t)            # a pose VIVA
                                                  v    = velocidade_actual   # em unidades de VALOR
                                                  span = |to - from|
                                                  s    = resuming(v / span)  # re-normaliza para o percurso NOVO
                                                  t    = Track{from, to, s, role}

advance(dt_parede):                              # UMA chamada por quadro, no topo do frame
    for (id, t) in live:
        settled = t.s.advance(dt, spring_de(t.role))     # a mola do repo, verbatim
        if settled:
            escreve o valor EXACTO  e  EVICT(id)          # a lei do `arrive`

value(id, fallback) -> f32:                      # o pintor pergunta
    live.get(id).map(|t| lerp(t.from, t.to, t.s.x)).unwrap_or(fallback)
```

⚠️ **A re-normalização `v / span` é a linha que faz a interrupção funcionar**, e é onde uma
implementação ingénua erra: a `SpringState` mede o caminho em `[0,1]`, então uma velocidade em
unidades de **valor** tem de ser dividida pelo **novo** comprimento antes de entrar. Sem isso, um
alvo próximo herda uma velocidade enorme e estala.

⚠️ **`EVICT` é o que torna verdadeira a afirmação de custo.** O mapa é *o conjunto do que se mexe*,
não *o conjunto de widgets* — tipicamente 0-3 entradas. Sem despejo ele cresce monotonamente e a
alegação `O(vivos)` vira falsa em silêncio.

### 1.4 O custo — e o que este plano NÃO sabe

Afirmação: **`O(vivos)`**, com `vivos` ≈ 0-3 em uso normal e um pico no `stagger` de uma lista.
Um passo de mola são ~4 flops × `dt/(1/240)` fatias ⇒ a 60 fps, **4 sub-passos** por track.

⚠️ **Não medido.** A sonda que decide chama-se `measure_ui_motion` e mede **pela porta do produto**
(o `advance` do quadro real), com as colunas *parado* · *um hover* · *cascata de 40 rows*. Nenhum
número deste plano vale antes dela.

### 1.5 O gate que carrega a wave

**`a_chrome_without_motion_paints_what_it_paints_today`** — com o mapa vazio, `value()` devolve o
`fallback` e a tela é **byte-idêntica** à de hoje. É a neutralidade que torna a F0 segura de landar
sozinha, antes de qualquer efeito. *Mutação: `value` a devolver `from` em vez do fallback ⇒ sangra.*

Irmãos: `an_idle_app_has_no_live_tracks` (propriedade, sem relógio; mutação: não despejar ⇒ o mapa
cresce) · `an_interrupted_target_inherits_the_live_value_and_velocity` (mutação: `at_rest()` em vez
de `resuming` ⇒ a segunda metade do percurso arranca parada).

### 1.6 ⭐ E a F0 arrasta o defeito vivo do §1 do estudo

`ToastQueue` conta **quadros**. Passa a consumir o mesmo `dt` de parede, e o gate é o mais barato e
mais exato desta lista:

**`a_toast_lives_three_seconds_at_any_frame_rate`** — dirige a fila a 30 e a 120 fps e exige o mesmo
tempo de vida. *Mutação: voltar a `age += 1` ⇒ sangra com 6,0 s contra 3,0.*

---

## §2 — F1/F2: a mola chega ao chrome, e o CARÁCTER é uma porta

### 2.1 A porta única

```rust
impl UiMotion {
    /// A ÚNICA função que sabe o que cada carácter faz. O pintor pergunta-lhe;
    /// o dispatch pergunta-lhe. Duas cópias divergem no primeiro caso especial.
    fn law(role: Role, ch: UiCharacter, reduced: bool) -> Drive { … }
}
```

| `Role` | Expressivo | Discreto | + Reduced (sobre QUALQUER carácter) |
|---|---|---|---|
| `Travel` (posição, tamanho) | **mola** ζ≈0,75 | curva 120 ms `ease-out` | **0 ms — salta** |
| `Fade` (opacidade, cor) | mola ζ≈1,0 | curva 90 ms linear | 90 ms — **fica** |
| `Number` (readout, valor) | **instantâneo** | instantâneo | instantâneo |
| `Decoration` (§11 D·F) | mola | **ausente** | ausente |

⚠️ **`Number` é instantâneo nos três.** Uma posição pode balançar; um **número lido** que balança
está **errado durante 200 ms**, e alguém vai lê-lo. É a cerca que impede a wave de virar contra si.

⚠️ **Reduced mata PERCURSO, não fade.** É a distinção vestibular: o que faz mal é a área grande a
deslocar-se, a paralaxe e a rotação — não a opacidade. Colapsar as duas entregaria uma garantia de
acessibilidade disfarçada de gosto (estudo §10.2).

### 2.2 Os dois eixos, e o gate que os prova independentes

**`the_taste_and_the_guarantee_are_two_axes`** — as **quatro** combinações são alcançáveis, e
*Expressivo + reduced* tem de manter o som e o material e perder o percurso. *Mutação: um seletor de
três posições ⇒ a combinação some e o gate sangra.*

### 2.3 F2 — os 49 widgets herdam de graça

Nenhum widget é reescrito. A herança acontece na **porta de pintura**: onde hoje o pintor lê
`state == Hovered` e escolhe uma cor, passa a ler `motion.value(id, alvo)` — **um sítio por
propriedade animada**, não 49 sítios.

⚠️ **Arch-gate obrigatório:** `the_character_is_asked_once` — o pintor e o dispatch resolvem pela
MESMA `law`. É a cicatriz do `TimelineInterpScope::menu_table()` e a do `stroke_cover_wanted`, e ela
custa um gate para não se repetir uma terceira vez.

---

## §3 — A peça em falta: preferências de UTILIZADOR

Medido (estudo §10.3): **não existe**. As `SavedSettings` (v69) viajam dentro do `ProjectFile`.

| # | possibilidade | custo | veredito |
|---|---|---|---|
| A | pôr em `SavedSettings` (v69) | zero | ⛔ **o gosto viaja com o documento** — abrir o ficheiro de um colega muda como o **seu** app se mexe. |
| B | ficheiro próprio no config dir, **schema próprio** | ~60 linhas + IO | ✅ **SIM** |
| C | dep `directories`/`dirs` | dep externa nova | ⛔ desnecessário: `XDG_CONFIG_HOME` → `HOME/.config` → `APPDATA` resolve-se com `std::env`, **zero deps** |

**Forma:** `~/.config/ph2d/prefs.postcard`, `PREFS_SCHEMA` **próprio** (⚠️ **nunca** o
`PROJECT_SCHEMA` — são coisas com donos e ciclos de vida diferentes). Ausente ou ilegível ⇒
**defaults**, sem erro: uma preferência que recusa arrancar é pior que uma preferência perdida.

**Primeiros inquilinos:** carácter · reduced motion · volume do som de UI (§11 G do estudo).

⚠️ **A row nova no pill Settings** é `CTX_MENU_SETTINGS_MOTION`, irmã exacta das cinco que já lá
estão (PPM · UNIT · FILTER · DISPLAY · TEXT) — id por **hash de string** ⇒ **nenhum contador de
gate** se move.

---

## §4 — E1: o SCRUB NUMÉRICO (o maior ganho de eficiência, e não é animado)

### 4.1 O algoritmo — e a parte que todos erram

O difícil não é mudar o valor: é que **o mesmo campo é também um campo de texto**.

```
Down no campo            -> PendingScrub{ id, origin_px, valor_inicial }   # NÃO decide ainda
Move, |dx| <= THRESH     -> continua pendente
Move, |dx| >  THRESH     -> vira SCRUB;  o posicionamento de caret é CANCELADO
Up ainda pendente        -> é um CLIQUE: põe o caret (o comportamento de hoje, intacto)
Up em scrub              -> commit; UM passo de undo para o gesto inteiro
```

⚠️ **Decidir no Down destrói uma das duas metades:** comprometer com scrub torna o campo
indigitável; comprometer com caret torna o scrub impossível. `THRESH = 4 px` (a medir no smoke).

### 4.2 A lei da resposta

```
Δvalor = dx_px · sensibilidade · modificador
sensibilidade = max( (max-min) / LARGURA_UTIL_PX , step )
```

⚠️ **A sensibilidade tem de sair da FAIXA do campo**, não ser uma constante: um campo `0..1` e um
`0..5000` não podem partilhar píxeis-por-unidade, e uma constante torna um dos dois inutilizável.
O piso em `step` impede que um campo de faixa minúscula fique morto.

**Modificadores** — ⚠️ **a colisão tem de ser conferida antes**: neste app `Shift` já significa
*restringir* em vários gestos. Proposta: **`Shift` = ×0,1 (precisão)** e **`Ctrl` = encaixa no
`step`**; a varredura de colisão é parte da wave, não um detalhe.

### 4.3 O ponteiro — três possibilidades, e a honesta é a primeira

| # | como | custo | veredito |
|---|---|---|---|
| A | o cursor **viaja** | zero | ✅ **v1.** O curso acaba ao fim de ~600 px; para 95% dos ajustes chega. |
| B | `set_cursor_grab(Locked)` + esconder ⇒ arrasto **infinito** | ⚠️ **desconhecido** | a shell **nunca** prendeu um cursor; `Locked` **não é suportado em todas as plataformas** (X11/Wayland/macOS divergem). ⇒ **sonda primeiro**, promessa depois. |
| C | dar a volta na borda do ecrã | médio | ⛔ pisca e confunde |

### 4.4 Gates

`a_down_that_does_not_move_still_places_the_caret` (a metade que se perde primeiro) ·
`a_drag_past_the_threshold_scrubs_and_never_places_a_caret` · `the_whole_gesture_is_one_undo_step` ·
`the_sensitivity_comes_from_the_range` (mutação: constante ⇒ o campo `0..1` fica inutilizável e o
gate mede-o).

---

## §5 — C1: o TETHER (o pedido do Enio)

### 5.1 Possibilidades

| # | motor | custo | veredito |
|---|---|---|---|
| A | **`rapier`** (já temos) | zero código | ⛔ **NÃO.** É simulador de **MUNDO**, com contrato de determinismo (`physics_ecs_c9`, hash comparado em 3 SOs) e schema. Um enfeite de chrome passaria a poder **mover um hash de determinismo**. |
| B | o `verlet_rope` dos nós | zero código | ⛔ mesma família: é conteúdo cozido do documento, com fingerprint. |
| C | **Verlet próprio, em espaço de tela** | ~80 linhas, zero deps | ✅ **SIM** — descartável por construção, que é exactamente o que uma decoração deve ser. |

### 5.2 O algoritmo — Verlet corrigido no tempo

```
// integração (os extremos 0 e n-1 são PINADOS, não integram)
for i in 1..n-1:
    vel   = (p[i] - q[i]) * (dt / dt_prev) * DAMP     //  ⚠️ TCV: o factor dt/dt_prev
    q[i]  = p[i]
    p[i] += vel + g * dt * dt

// restrição de distância, 2..4 iterações
for _ in 0..ITERS:
    for (a, b) in segmentos:
        d = p[b] - p[a];  l = |d|
        if l > EPS:
            corr = d * (0.5 * (l - rest) / l)
            if !pinado(a) { p[a] += corr }
            if !pinado(b) { p[b] -= corr }
    p[0] = controlo;  p[n-1] = efeito        // re-pinar DEPOIS de cada iteração
```

⚠️ **O factor `dt / dt_prev` é a wave inteira numa linha.** Verlet clássico assume passo **fixo**;
com passo variável, um engasgo de quadro faz a corda **saltar**. É literalmente a lei que este
repositório já pagou quatro vezes no relevo do Painter — *o desenho é fato do relógio, nunca de quão
depressa a máquina amostrou* —, aqui pela primeira vez no chrome.

**Parâmetros:** `n = 12..16` · `ITERS = 3` · `DAMP ≈ 0,98` · `rest = dist_reta × folga` (a folga > 1
é o que a faz **pendurar**). **Custo:** `n·ITERS` ≈ 48 projecções/quadro — irrelevante, e ainda assim
**a medir** pela sonda, não pela aritmética.

**Desenho:** polilinha → o pintor de traço que já existe; opcionalmente Catmull-Rom para suavizar
(temos `resample_smooth` no Flip como precedente de forma, não de código).

**Degenerados nomeados:** controlo ≡ efeito (comprimento zero ⇒ não desenha) · as duas pontas a
moverem-se mais depressa do que a restrição apanha (⇒ **clamp de deslocamento por quadro**, senão a
corda estica e volta com estalo).

### 5.3 Em Discreto

**Uma linha reta entre os mesmos dois pontos.** O *significado* sobrevive inteiro (a relação
continua visível); o que sai é o peso. ⇒ o tether **não** é um efeito só-Expressivo: é um efeito com
duas expressões.

### 5.4 Gate

⭐ **`the_rope_is_a_fact_of_the_wall_clock_not_of_the_frame_rate`** — o MESMO gesto (mesmas posições,
mesmo tempo total) dirigido a **30 e a 120 fps** produz a mesma forma dentro de ε. *Mutação: tirar o
`dt/dt_prev` ⇒ as duas formas divergem e o gate nomeia quanto.* É o gate que impede a corda de ser
bonita na máquina de quem a escreveu.

Irmãos: `a_pinned_end_is_exactly_the_control` (as pontas não derivam) · `the_discrete_character_draws
_a_straight_line_and_simulates_nothing` (mutação: simular e desenhar reto ⇒ custo sem efeito).

---

## §6 — A ordem, com o que cada wave desbloqueia

| # | wave | depende de | desbloqueia | tam. |
|---|---|---|---|---|
| **1** | **F0 substrato** + o toast em segundos | — | **tudo** o eixo 1 | **M** |
| **2** | **F1+F2+R1 juntos** — a mola chega ao chrome, os 49 widgets ganham vida, e o interruptor que a desliga nasce no MESMO commit | 1 | A · B · E · F | **M** |
| **3** | **Preferências de utilizador** + a row do pill Settings | 2 | o carácter deixa de ser constante | **P** |
| **4** | ⭐ **E1 scrub numérico** | — (independente!) | eficiência | **M** |
| **5** | ⭐ **C1 o TETHER** | 1 | a família C2·C3·C4 | **M** |
| **6** | o resto do catálogo, por gosto | 1-3 | — | — |

⚠️ **A wave 2 tem de trazer a R1 dentro dela.** Um efeito que nasce sem o interruptor nasce dívida —
e a acessibilidade retro-encaixada é a que fica meio-feita.

⚠️ **A wave 4 não depende de nada** e pode correr em paralelo, ou primeiro se o objectivo for
eficiência antes de encanto.

---

## §7 — O que NÃO entra (cercas plantadas antes)

- ⛔ `rapier` / `verlet_rope` para decoração (§5.1).
- ⛔ Mola em **números** (§2.1).
- ⛔ Animação que atrase a aceitação de um clique. Um clique durante uma transição é **sempre** aceite.
- ⛔ Um seletor de **três** posições para carácter+reduced (§2.2).
- ⛔ O carácter dentro do `PROJECT_SCHEMA` (§3).
- ⛔ Confetti, e animar o **conteúdo** do canvas (estudo §7).
- ⛔ Som ligado por omissão.

---

## §8 — O que este plano NÃO sabe (as sondas a correr ANTES de prometer)

1. **`measure_ui_motion`** — o custo real da F0 pela porta do produto (§1.4). *Nenhum número de custo
   deste plano vale antes dela.*
2. **A sonda de `set_cursor_grab`** por plataforma (§4.3) — decide se o scrub promete arrasto infinito
   ou 600 px.
3. **A varredura de colisão de modificadores** (§4.2) — `Shift` já significa *restringir* neste app.
4. **O `n` e a folga do tether** (§5.2) — são números de **aparência**, e o oráculo deles é o
   RENDER, não um teste. Saem do smoke, como o `RESAMPLE_STEP_FRACTION` do Flip saiu.
5. **Nada aqui bumpa `PROJECT_SCHEMA`, toca contrato congelado ou acrescenta dep externa** — e isso
   é afirmação a **conferir por `git diff` no fecho**, não a acreditar agora.

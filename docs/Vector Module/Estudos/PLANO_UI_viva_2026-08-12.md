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

**Forma:** ~~`~/.config/ph2d/prefs.postcard`, `PREFS_SCHEMA` **próprio**~~ → **`~/.ph2d/prefs.txt`,
texto, sem número de versão.** Ausente ou ilegível ⇒ **defaults**, sem erro: uma preferência que
recusa arrancar é pior que uma preferência perdida.

> ⚠️ **CORRIGIDO NA CONSTRUÇÃO (wave 3), e as duas correcções são do repo, não de gosto.**
>
> **(a) O lar já existia.** Este plano mandava abrir `~/.config/ph2d/` — e a shell **já tem** um
> ficheiro de preferências de utilizador: o `palette_persist`, em **`~/.ph2d/palettes.txt`**. Abrir
> uma segunda pasta ao lado seriam **duas casas para a mesma categoria de facto**, que é a falha que
> este repo paga em ciclos. O módulo novo é **irmão** daquele: mesma pasta, mesmo estilo (texto,
> std-only, best-effort, sem serde) e o **mesmo detector de mudança** (derivar → comparar → gravar),
> que o `persist_palettes_if_changed` já shipava.
>
> **(b) O número de versão era ceremónia, e o trade estava invertido.** Num formato **posicional**
> (postcard, o `ProjectFile`) a versão é obrigatória e recusar é a leitura honesta. Num
> `chave=valor` a compatibilidade é grátis nos **dois** sentidos: um build antigo lê as chaves que
> conhece e salta as que não conhece. Um `PREFS_SCHEMA` aqui faria o build antigo **recusar** o
> ficheiro inteiro que o novo escreveu — exactamente o oposto do que se quer de uma preferência.
> A propriedade que ele substituiria é agora **executável**
> (`a_key_from_a_newer_build_is_skipped_and_the_rest_survives`).
>
> O veredito de (C) — *zero deps externas* — sobreviveu, e ficou mais barato: `$HOME` e nada mais.

**Primeiros inquilinos:** carácter · reduced motion · volume do som de UI (§11 G do estudo).

⚠️ **A row nova no pill Settings** é `CTX_MENU_SETTINGS_MOTION`, irmã exacta das cinco que já lá
estão (PPM · UNIT · FILTER · DISPLAY · TEXT) — id por **hash de string** ⇒ **nenhum contador de
gate** se move.

---

## §4 — E1: o SCRUB NUMÉRICO (o maior ganho de eficiência, e não é animado)

> ⚠️ **CORRIGIDO NA CONSTRUÇÃO (wave 4), e a correcção é da medição, não de gosto: esta secção
> prometia construir o que já shipava.** O scrub numérico existe completo desde a M14.A —
> `NUMBER_INPUT_DRAG_THRESHOLD_PX = 4.0` (o mesmo 4 que a §4.1 propunha *«a medir no smoke»*), o
> `crossed_threshold` que resolve no Down a ambiguidade caret-contra-scrub (a parte que a §4.1
> chamava *«a que todos erram»*), o bloqueio de eixo, o delta incremental e o `DRAG_SHIFT_MUL` na
> tecla que a §4.2 propunha. **Escrever a wave sem `git grep` teria sido reconstruir por cima de
> código shipado** — a §0 do `CLAUDE.md` a morder em casa: *«fora de escopo porque não existe» é uma
> afirmação sobre código que outra pessoa pode ter escrito*, e desta vez a outra pessoa era este repo.
>
> **A §4.2 acertou pelo motivo certo e é lá que estava o buraco real.** *«A sensibilidade tem de sair
> da FAIXA do campo»* estava implementado — para **duas** das **quatro** fontes de intervalo que o
> app tem. O clamp do `dispatch::pointer_move` conhecia as quatro (taxa registada · `number_range` ·
> a projeção afim do **slider ligado** · o `(0,1)` de um chip de canal do picker); a **taxa**
> conhecia duas e caía no atalho histórico para as outras. Uma caixa **clampada num intervalo
> conhecido** era arrastada a `DRAG_RATE_X · step`, que não sabe nada sobre esse intervalo.
>
> **Medido pela sonda `census_of_how_many_pixels_cross_a_whole_field`** (que pergunta *quantos
> pixels atravessam o campo INTEIRO*, e não *quem registou faixa*): **295** campos com intervalo
> conhecido, **43 a cruzarem-se inteiros em menos de 20 px** — o pior em **0,01 px**, um único pixel
> a saturar o campo cem vezes — e um a 510 px, lento pela mesma causa. Todos os servidos pelo
> `number_range` cruzavam em **250,00**, o alvo. *A lei funcionava; só era consultada da lista curta.*
>
> ⇒ A cura é a **porta única** `WidgetStore::number_scrub_law`, que devolve a taxa **e** os limites
> da mesma travessia. **Depois: 295 campos, todos a 250,00 px. 43 → 0.** E ela **não inventa número
> nenhum** — faz a taxa ler a fronteira que o clamp ao lado dela já lia, que é o que separa esta
> wave do *«inventar um tecto por campo»* que a sonda anterior proibiu.
>
> ⚠️ **Fica ABERTO, com o número:** **146** campos não têm intervalo nenhum (posição em px,
> contagens sem tecto) e continuam no atalho histórico. Um tecto para eles tem de ser **MEDIDO**,
> nunca escolhido; o primitivo para *«tem mínimo e não tem máximo»* é o `set_number_drag_rate`, que
> já existe e não inventa fronteira.
>
> ⚠️ **E a §4.3 continua por decidir**, intocada por esta wave: o cursor **viaja** (v1), e o
> `set_cursor_grab` segue sem sonda de plataforma.

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

> ⚠️ **CORRIGIDO NA CONSTRUÇÃO (wave 5): o `dt/dt_prev` foi construído, MEDIDO e é insuficiente.**
> O mesmo gesto a 30 e a 120 fps diverge **29,7 px** com ele. O TCV cura **uma** das três
> dependências do relógio e deixa duas de pé: o **amortecimento** por-quadro (`DAMP^120` contra
> `DAMP^30` num segundo) e — a maior, e a que ninguém vê — a **RIGIDEZ**, porque a restrição relaxa
> `ITERS` vezes por QUADRO e portanto puxa quatro vezes mais por segundo a 120 fps. *Um solver
> iterativo é tão rígido quanto o número de passagens que o relógio lhe paga.*
>
> ⇒ O que shipa é um **passo interno FIXO** com acumulador: as três morrem de uma vez, e a
> igualdade deixa de ser aproximada — **0,0000 px** entre 30 e 120 fps com as pontas paradas. O
> `dt/dt_prev` sai junto (com passo fixo ele é `1.0` por construção). O resíduo de **1,68 px** com
> as pontas a mover-se é da **amostragem da ENTRADA**, não do solver, e tem gate próprio a dizê-lo.
>
> ⚠️ E o acumulador traz a lição que a água já pagou: um quadro lento **não** compra passos sem
> tecto, e o resto é deitado fora em vez de guardado como dívida.

⚠️ ~~**O factor `dt / dt_prev` é a wave inteira numa linha.**~~ Verlet clássico assume passo **fixo**;
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
| ~~**2**~~ ✅ | **F1+F2+R1 juntos** — **FEITA (F2 fechou 2026-08-14)**, e por uma rota que esta tabela não listava: o `t` é **publicado no store**, não passado a cada pintor. 108 `.visual(..)` em 19 crates; as quatro famílias de botão fecharam, e as três que puderam pôr o par na assinatura são fechadas **pelo compilador** em vez de por um gate. ⚠️ **Pendente de smoke:** ~20 caixas e ~10 interruptores passam a mostrar hover/press pela primeira vez. Ver a nota abaixo | 1 | A · B · E · F | **M** |
| ~~**3**~~ ✅ | **Preferências de utilizador** + a row do pill Settings — **FEITA** (`~/.ph2d/prefs.txt`, irmão do `palette_persist`; ver a correcção na §3) | 2 | o carácter deixa de ser constante | **P** |
| ~~**4**~~ ✅ | ⭐ **E1 scrub numérico** — **FEITO, e diferente do que esta linha dizia**: o gesto já shipava desde a M14.A; o buraco real era a taxa a consultar **duas** das **quatro** fontes de intervalo que o clamp já conhecia (43 campos cruzavam-se inteiros em < 20 px). Cura = a porta única `number_scrub_law`; 43 → 0. Ver a correcção na §4 | — (independente!) | eficiência | **M** |
| ~~**5**~~ ✅ | ⭐ **C1 o TETHER** — **FEITA**, com a lei do relógio CORRIGIDA por medição (ver a nota na §5.2) e o card de Fill como primeiro consumidor | 1 | a família C2·C3·C4 | **M** |
| **6** | o resto do catálogo, por gosto — **em curso**: a **F5 cascata** FEITA na paleta (⚠️ `ε` reprovado uma vez no smoke — 0,020 lia-se simultâneo; o valor é **0,050 s**, §6.3 do estudo), a **E2 rolagem suave** (a porta `panel_scroll` passou a devolver o vivo ⇒ ~130 leitores de graça) e as **rows dos menus** da barra entraram na paleta global (33 verbos, 9 menus), fechando a cauda que o commit da E3 nomeou. ⚠️ E corrigindo-o: *«a barra de topo fica de fora»* era verdade do **PILL** e falsa da **ROW**. Ver a §6.1 do estudo. ⚠️ A **E2 foi REPROVADA no 1º smoke** (*«o balanço das labels ficou bem artificial e pouco suave»*) e curada por um `Role` novo — o **`Role::Surface`**, a superfície cujo lugar o dedo COMANDA: a causa era o `~/.ph2d/prefs.txt` dizer `motion_character=expressive`, onde o `Travel` ultrapassa **31,08 px** numa volta de 200. §6.4. ⚠️ E a **tabela da §6 do estudo mentia em cinco linhas** (F1·F3·R1·C1·E1 apareciam pendentes e estavam feitas) — auditada na §6.5, com os três itens que sobram MEDIDOS | 1-3 | — | — |

~~⚠️ **A wave 2 NÃO está feita, e a medição de 2026-08-13 diz quanto falta:** a F1 chegou (o chrome
tem a mola) e a **F2 não** — `.hover_t()` é passado em **2** sítios contra **161** que pintam um
botão, então a mola é integrada e a pintura deita-a fora.~~ ✅ **A F2 FECHOU em 2026-08-14, e a
bifurcação resolveu-se para a TERCEIRA opção, que não estava na lista.**

O plano oferecia *cada sítio pede* × *o compilador enumera*. Medido antes de escolher, a primeira
custa **~150 assinaturas em 17 crates** (**56 só no `ph2d-panel-inspector`, para 20 botões**) — o
número que diz que a corrente é o lugar errado para o relógio viajar. A que shipou é o precedente
desta mesma linha: **o `t` é PUBLICADO no store** (`WidgetStore::button_visual`, o gêmeo do
`panel_scroll_live`, que já dera suavidade a ~130 pintores sem uma assinatura mudar) — e a posse do
relógio **não se move**: o `UiMotion` mantém as tracks, o carácter e o *reduced motion*.

**Estado medido:** **108 chamadas `.visual(..)` em 19 crates**, e **157 leituras** das três portas
(`button_visual` · `checkbox_visual` · `toggle_visual`). As quatro famílias fecharam:

| família | porta | quem a fecha |
|---|---|---|
| `Button` | `.visual((estado, t))` | o gate `every_button_wears_the_live_hover` (nenhum `Button::new(..)` de produção chega a `.state(`) |
| `IconButton` | `paint_icon_button(.., visual, ..)` | **o compilador** — o par está na assinatura, um `ButtonState` solto não compila |
| `Checkbox` · `Toggle` | `.visual((estado, t))` | idem, mais `motion::hover_axis` como guarda única do eixo |

⚠️ **E a F2 achou trabalho que não era o `t`:** o `Checkbox` **nunca reagiu ao rato** (nasce em
`Normal` e nenhum sítio de produção chamava `.state(..)`), três botões de ícone do `wet-tuning`
pintavam `Normal` cravado com `hit_index.register` ao lado, e a derivação hot/active estava
inventada **seis** vezes em privado. ⚠️ **A rack de FX do áudio fica inerte e agora está NOMEADA no
código:** ela pinta sem `WidgetStore` no caminho todo — acordá-la é wave própria.

⚠️ **A wave 2 tem de trazer a R1 dentro dela.** Um efeito que nasce sem o interruptor nasce dívida —
e a acessibilidade retro-encaixada é a que fica meio-feita.

### 6.1 A GRADE DE PIXELS — o terceiro defeito com o mesmo sintoma (2026-08-14, pós-smoke)

O smoke da F2 passou com um report ao lado: *«as labels ainda têm um movimento incómodo ao rolar
os painéis»*. ⚠️ **É o TERCEIRO defeito distinto a produzir a mesma queixa**, e cada um só ficou
visível depois de o anterior sair da frente:

1. a rolagem shipou em `Role::Travel` ⇒ **ultrapassava 15,5%** (curado pelo `Role::Surface`);
2. — nada, entre os dois, que a wave da F2 tenha tocado;
3. **a label encaixa no pixel e a linha dela não.**

O `paint_text` arredonda a origem do texto ao pixel inteiro (o *hinting* precisa da baseline no
grid); os retângulos vão para o Vello sem arredondar. Parados concordam — é para isso que o snap
existe. Em movimento contínuo, **a linha desliza e a label fica parada até cruzar meio pixel**.
Medido pela porta do produto, numa rolagem de 40 px:

| rota | a label afasta-se | o passo desencontra-se | quadros parada |
|---|---|---|---|
| **cru** (o que shipou) | 0,481 px | 0,820 px | **3** (50 ms) |
| **grade** (a cura) | **0,000** | **0,000** | **0** |

⚠️ **Os números do «cru» são IDÊNTICOS nos dois carácteres** — a `Role::Surface` é criticamente
amortecida em ambos —, e é isso que explica por que o (1) não o tocou.

**A cura é `motion::on_pixel_grid`, aplicada no PUBLICAR**: o relógio integra contínuo (uma mola
alimentada com entrada quantizada pode estagnar) e o **alvo** guarda o valor exato (é ele que soma
os deltas fraccionários de um trackpad). Dois consumidores, a mesma porta: a rolagem de painel e o
`cascade_rise` (o cartão translada e **leva a própria label**). ⚠️ **O `hover_lift` fica de fora,
com o motivo escrito no código:** ele cresce o retângulo por igual nos quatro lados ⇒ o centro não
se move ⇒ o glifo centrado não viaja; quantizá-lo só tornaria o crescimento aos degraus.

⚠️ **De graça, e por isso vale a pena nomear:** isto cura também a **cintilação dos filetes** —
uma linha de 1 px numa fracção reparte a cobertura por dois pixels e muda de aparência a cada
quadro.

### 6.2 F4a — a SECÇÃO dobra-se: o chevron roda e a placa desvanece (2026-08-14)

A **F4** estava na §6.5 do estudo como *«36 `is_collapsed` em 32 arquivos, e é mudança de LAYOUT —
mais cara que a contagem sugere»*. Medida antes de escolher, ela tem **duas metades com preços
muito diferentes**, e só uma delas tem porta:

| metade | preço medido | tem porta? |
|---|---|---|
| **o CHEVRON** (e a placa) | **23** cadeias `.collapsible(..)`, e o `paint_section_header` é **um só** | ✅ |
| **o CORPO** (a altura interpola) | ~20 pintores, cada um com aritmética de `y` própria; pede medir-lembrar-recortar por painel | ❌ |

**Esta wave fez a primeira.** O `t` da dobra é publicado no store (`section_open_live`, o terceiro
membro da família `hover_live`/`panel_scroll_live`) e o cabeçalho veste-o.

⚠️ **O FACTO que a torna sem costura foi medido, não assumido:** os dois glifos que o app já
mostra são **a mesma seta**. Rodando `chevron-down` por −90° em torno de `(12,12)`:

| `chevron-down` | rodado | `chevron-right` |
|---|---|---|
| `6,9` · `12,15` · `18,9` | `9,18` · `15,12` · `9,6` | `9,18` · `15,12` · `9,6` |

— **ao ponto**. A trajectória atravessa exactamente os dois desenhos que já shipavam; se não
fossem rotações um do outro, a forma daria um salto em cada extremo. ⚠️ **E o gate apanhou um erro
de SINAL meu antes do smoke:** a kurbo roda pela matriz `[[cos,−sin],[sin,cos]]` sobre um espaço
**y-para-baixo**, então o `+π/2` que a conta de papel dá produz `chevron-LEFT` — a seta giraria ao
contrário durante o gesto inteiro.

⚠️ **Nos dois repousos o desenho é BYTE-IDÊNTICO** (o glifo é pintado sem rotação nenhuma, e só
entre eles é que roda): `cos(90°)` em `f64` vale `6,1e-17`, e um glifo parado alinhado aos eixos é
o que o encaixe no grid de pixels consegue afiar — *nítido parado, suave em movimento*.

**E a PLACA escura foi junto**, porque sem ela o cabeçalho teria duas metades a discordar sobre
quando a secção fechou. ⚠️ A primeira versão gateava-a no flag **semântico** e era assimétrica —
desvanecia a FECHAR e **sumia de repente a ABRIR**; quem manda é o `t`.

**A estreia de cada secção precisa de uma PARTIDA.** A lei do substrato é que *a primeira vista de
um id chega ao alvo*, então sem semear o `toggle_collapsed` a **primeira** dobra de cada secção
saltaria — um defeito de uma-vez-por-secção-por-sessão, o mais difícil de reproduzir que há.
⚠️ E a metade oposta é igualmente lei: uma secção que **nasce** fechada (`set_collapsed` no
`populate`) tem de aparecer fechada, não fechar-se sozinha no primeiro quadro. Os dois gates são
independentes, e cada mutação sangra só o seu.

**Higiene que a wave arrastou, e que vale por si:**

- **três `event.rs`** (physics · sculpt3d · wet-tuning) re-escreviam `toggle_collapsed` à mão
  (`set_collapsed(id, !is_collapsed(id))`) — pela cópia, a estreia daqueles painéis saltaria;
- as **doze secções do Inspector** passaram a construir o cabeçalho por **uma porta**
  (`sections::section_header`), o que pagou a dívida de LOC das duas `fn` mais gordas do painel
  (307→305 · 279→277: *as tolerâncias encolhem, nunca crescem*);
- a **família dos ícones saiu do `paint.rs`** para o irmão `paint_icons.rs` (o tecto de 700), e a
  allowlist do `canonical_icon_button` **seguiu o código** — senão o gate passaria a proibir o
  ficheiro que define a função que ele protege;
- `section_header.rs` virou **directório** (518 > 500) com a dobra no irmão `fold.rs`. ⚠️ Um
  `section_fold.rs` solto em `src/widget/` viraria um **widget** para o `ph2d-widget-sync` — a
  cicatriz que o `command_palette_tests.rs` pagou.

### 6.2-bis F4b — o CORPO interpola (2026-08-16, por ordem do Enio)

A nota que aqui estava dizia *"o corpo fica por fazer … o smoke decide se ele é sentido em falta"*.
O Enio mandou construí-lo, e a previsão dela — *precisa de `body_h` **antes** de pintar, logo de
medir-lembrar-e-recortar por painel* — estava certa; o que ela não previa é que a memória cabe
numa porta só.

**A porta é `widget::SectionFold`** (`section_header/body.rs`), e ela precisa de **TRÊS** coisas ao
mesmo tempo, não de uma:

| metade | sem ela |
|---|---|
| recorte de **CENA** (`push_clip`) | o corpo desenha inteiro por fora da banda |
| recorte de **HIT** (`HitIndex::push_clip`, novo) | uma row invisível responde nos vãos entre os widgets da secção de baixo, que já subiu por cima dela |
| `y` de saída **ESCALADO** | tudo o que está por baixo não sobe junto — a dobra não move nada |

⚠️ **A altura é MEDIDA num quadro e LEMBRADA para o recorte do seguinte** (`WidgetStore.fold_body_h`,
uma `RefCell`) — e a interior mutability é **ESTRUTURAL, não conveniência**: quem mede é o pintor,
e a API do próprio host (`store_and_hit_index_mut`) garante que ele **nunca** segura
`&mut WidgetStore` e `&mut HitIndex` juntos. Uma medição obsoleta é inofensiva por construção: ela
alimenta **só o recorte**, e o layout usa sempre a fresca. Memória ausente ⇒ recorta a zero **e
mede na mesma** — a estreia de uma secção custa um quadro invisível, não um salto de corpo inteiro.

⚠️ **Os dois repousos são byte a byte o mundo pré-wave:** aberto não empurra camada nenhuma e
devolve o `y` **verbatim** (não `body_top + medido·1.0`, que arredonda); fechado e parado devolve
`None` e o pintor devolve o `body_top`, exactamente o `if collapsed { return y + header_h; }` de
sempre.

⚠️ **E o `t` substitui o `is_collapsed` nos pintores, o que é load-bearing:** o flag semântico vira
no quadro do clique enquanto o `t` ainda desce, então um corpo gateado nele sumiria de repente por
baixo de um chevron a rodar.

⚠️ **Duas entradas, uma lei** (`begin` · `begin_at`): o `ph2d-panel-audio-editor` **fotografa** os
oito `t` num array antes de o paint tomar os empréstimos, e um `begin` que relesse o store daria à
dobra um estado e ao chevron outro. Medido pelo gate daquele painel: aberto e fechado davam a
**mesma altura** (1842,62). Quem já tem a resposta passa-a; quem não tem chama o `begin`.

**Dez painéis migrados.** As contagens **estáticas**, medidas por chamador da porta: inspector
**12** (as 11 secções + a grade de 32 bits da §8) · vector **35** · painter-layers **11** (mais o
`paint_texture`, que é o caso especial) · audio-editor **8** · physics **7** · audio-mixer **5**.
⚠️ **Nos outros quatro a contagem é do DOCUMENTO ABERTO, não do código** — sculpt3d, wet-tuning,
motion-params e authored abrem a dobra dentro de um **laço**, então quantas secções dobram depende
do que o artista tem na tela; um número escrito aqui seria afirmação, não medição.

⚠️ **motion-params · authored · audio-editor são laços PLANOS** — ali uma secção não é escopo
léxico, vai de um cabeçalho até o **próximo** —, então a dobra vive num `Option` do laço e fecha
**antes** de o cabeçalho seguinte ser pintado.

⛔ **O `widget/showcase` fica DE FORA com motivo:** ele nunca recebeu a F4a (o cabeçalho dele não lê
`open_t`), é galeria de dev e não chrome do app. Migrá-lo pediria a F4a primeiro.

⚠️ **E a wave achou TRÊS gates que passaram a medir a coisa errada, todos da mesma família:** um
gate de dobra afirmava *"depois do clique a row não está pintada"*, o que era verdade quando a
dobra era binária e passou a ser verdade **só depois de a animação acabar** — e um harness de
painel não tem o tique do `HeroScreen`. Sem cura eles reprovariam a animação em vez de a medir.
A porta nova é **`MockPanelHost::settle_section_folds()`** (o relógio correu até ao fim), método
**NOMEADO** e nunca um `store_mut()` — o mesmo argumento do `set_panel_scroll`: ela responde a
UMA pergunta (*e se o artista esperar?*) em vez de abrir o store para um gate semear o que depois
vai "provar". ⚠️ Ela não tem gate próprio **de propósito**: os três consumidores reprovam sem ela,
e um self-test só afirmaria o que o método literalmente faz.

⚠️ **E o `Drop` do `SectionFold` pagou-se sozinho:** ele apanhou um `return` no `paint_texture`
— o *"esconde os controlos quando não há textura atribuída"* — que a migração deixou sem fechar a
dobra, **nomeando o sítio exacto**. Um recorte de cena pendurado não dá erro: ele corta o resto
do painel em silêncio.

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
   ou 600 px. **Segue por correr.**
3. ~~**A varredura de colisão de modificadores** (§4.2)~~ — **MOOT**: o `Shift` já é o multiplicador de
   precisão do scrub (`DRAG_SHIFT_MUL`) desde a M14.A, e a wave 4 não acrescentou modificador nenhum.
   O `Ctrl = encaixa no step` que a §4.2 propunha **não foi construído** e continua por decidir.
5. **O tecto dos 146 campos SEM intervalo** (§4, correcção da wave 4) — a sonda
   `census_of_how_many_pixels_cross_a_whole_field` lista-os; o número de cada um tem de sair de uma
   medição, e o primitivo é o `set_number_drag_rate`, que não inventa fronteira.
4. **O `n` e a folga do tether** (§5.2) — são números de **aparência**, e o oráculo deles é o
   RENDER, não um teste. Saem do smoke, como o `RESAMPLE_STEP_FRACTION` do Flip saiu.
5. **Nada aqui bumpa `PROJECT_SCHEMA`, toca contrato congelado ou acrescenta dep externa** — e isso
   é afirmação a **conferir por `git diff` no fecho**, não a acreditar agora.

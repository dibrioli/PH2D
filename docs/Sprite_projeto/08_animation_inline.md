# 08 — Animation inline (SpriteFrames + Tags + per-frame overrides)

## 8.1 Princípio

Spec do `SpriteFrames` asset e do `SpriteAnimator` Component. Cobre o modelo de animação **frame-based** (sprite-sheet sequential). Onion skin + timeline editor + frame events com payload tipado **NÃO** vivem aqui — vivem no módulo Animation futuro. Inspector do Sprite só mostra **estado atual da animação**.

## 8.2 Modelo Aseprite: frames = pool, animações = tags (ranges nomeados)

**Decisão estrutural:** frames são UM SÓ array sequencial; animações são **tags com range** dentro. Idle-1..idle-4 e walk-1..walk-8 podem ser frames 0..3 e 4..11 do mesmo sprite, **sem duplicação**.

Modelo superior a Godot AnimatedSprite2D (que tem N arrays separados) e Paper2D Flipbook (idem). Aseprite acerta; PH2D adopta.

## 8.3 `SpriteFrames` asset schema

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpriteFrames {
    /// Schema version (HR-14).
    pub version: u32,
    
    /// Pool sequencial de frames (atlas-referenced ou individual texture refs).
    pub frames: Vec<SpriteFrame>,
    
    /// Tags: ranges nomeados (animações).
    pub tags: Vec<AnimationTag>,
    
    /// Default animation que toca em autoplay (vazio = primeira tag).
    #[serde(default)]
    pub default_animation: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpriteFrame {
    /// Reference para textura (Atlas key, Individual texture_id, ou source uv).
    pub texture_ref: TextureRef,
    
    /// Duração desse frame em milissegundos. ❌ NÃO FPS global. ⭐⭐⭐
    /// Default 100ms (10 FPS — placeholder; usuário ajusta).
    pub duration_ms: u32,
    
    /// Per-frame pivot override (caso o pivot mude com a animação).
    /// Aseprite slice per-frame pivot + Construct hotspot per-frame.
    #[serde(default)]
    pub pivot_override: Option<[f32; 2]>,
    
    /// Per-frame Named Anchors (socket/slice mudam com animação).
    /// Vide [07_named_anchors.md §7.4].
    #[serde(default)]
    pub named_anchors: SmallVec<[NamedAnchor; 4]>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnimationTag {
    /// Nome da animação ("idle", "walk", "attack").
    pub name: String,
    
    /// Range de frames inclusive [from, to].
    pub from: u32,
    pub to: u32,
    
    /// Direção de playback.
    pub direction: AnimDirection,
    
    /// Loop count (None = infinite, Some(N) = N repetições, Some(1) = once).
    #[serde(default)]
    pub repeat: Option<u32>,
    
    /// Hold ms — pausa no último frame antes do loop. Phaser ⭐.
    #[serde(default)]
    pub hold_ms: u32,
    
    /// Repeat delay ms — pausa entre loops. Phaser ⭐.
    #[serde(default)]
    pub repeat_delay_ms: u32,
    
    /// Cor visual no timeline (decorativo, Aseprite import).
    #[serde(default)]
    pub label_color: Option<[f32; 4]>,
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AnimDirection {
    Forward,
    Reverse,
    PingPong,        // forward depois reverse, repete
    PingPongReverse, // reverse depois forward, repete
}
```

## 8.4 `SpriteAnimator` Component

Estado runtime que muta a cada tick. **`SimComponent`** — replay deve reproduzir frame avançado deterministicamente (HR-5 cross-OS). Anexado ao entity quando animação ativa.

**Tipo de tempo (corrigido pós-Lens-C C4):** `elapsed_ticks: u64` fixed-point (não `f32` acumulador que diverge cross-OS via FMA contraction). Tick = 1μs ou 100μs (decidir com bench em W4); `f64` ✗ (FP cross-OS issues). `frame_progress: f32` é **derivado** em extract phase (PresentWorld, HR-5 exempt).

```rust
#[derive(Component, Clone, Debug)]
pub struct SpriteAnimator {
    /// Reference para SpriteFrames asset.
    pub frames_ref: AssetHandle<SpriteFrames>,
    
    /// Tag atual sendo tocada.
    pub current_animation: String,
    
    /// Frame absoluto no pool (não relativo ao tag).
    pub frame: u32,
    
    /// Multiplicador de velocidade em Q16.16 fixed-point.
    /// 65536 = 1.0×; 131072 = 2.0×; -65536 = reverse 1.0×; 0 = pausa.
    /// Determinístico cross-OS (multiplicação inteira).
    pub speed_scale_q16_16: i32,
    
    /// True = tocando; false = pausado.
    pub playing: bool,
    
    /// True = toca automaticamente no spawn.
    pub autoplay: bool,
    
    /// Direção override (None = usa do tag, Some = override).
    pub direction_override: Option<AnimDirection>,
    
    /// Loop override (None = inherit, Some(true) = force loop, Some(false) = force once).
    pub loop_override: Option<bool>,
    
    /// Estado interno: tempo desde último frame change (μs, fixed-point u64).
    /// 1 tick = 1μs; 1ms = 1000 ticks; 60Hz frame = 16667 ticks.
    /// `+= delta_ticks_u64 × speed_scale_q16_16 / 65536` (saturating, all-integer).
    pub elapsed_ticks: u64,
    
    /// Estado interno: ping-pong direction atual (false=forward, true=reverse).
    pub pingpong_reverse: bool,
    
    /// Estado interno: contador de repetições já tocadas.
    pub repeat_count: u32,
    
    /// Estado interno: in-hold phase (durante Hold ms ou Repeat Delay ms entre loops).
    pub in_hold_phase: bool,
}
```

**`frame_progress: f32` em extract phase:**

```rust
// crates/ph2d-render/src/extract.rs (extract phase, PresentWorld — HR-5 exempt).
let frame_def = frames.frames[anim.frame as usize];
let duration_ticks = frame_def.duration_ms as u64 * 1000; // ms → μs
let frame_progress = anim.elapsed_ticks as f32 / duration_ticks as f32; // PresentWorld OK
sprite.frame = anim.frame;  // SimComponent update; deterministic
```

**Por que NÃO `f32 elapsed_ms`:** `anim.elapsed_ms += time.delta_ms() * anim.speed_scale` acumula erro de arredondamento que NÃO é bit-identical cross-OS — compiler pode emitir `vfmadd` (FMA) em x86_64 mas não em aarch64. Após N frames, divergência ULP-level → divergência no tick em que frame avança → `Sprite.frame` diverge → `RenderInstance.atlas_uv` diverge → replay hash quebra. `u64` ticks elimina o problema (multiplicação inteira é deterministic cross-OS via IEEE 754 basic operations).

## 8.5 Sistema de animação (runtime, fora do escopo Inspector) — fixed-point time

Sistema ECS que avança `SpriteAnimator` a cada tick. **Tudo integer arithmetic** para HR-5 cross-OS determinism:

```rust
fn animate_sprites(
    time: Res<Time>,                                // delta em μs (u64)
    asset_server: Res<Assets<SpriteFrames>>,
    mut query: Query<&mut SpriteAnimator>,           // sprite.frame atualizado via extract
) {
    let delta_ticks = time.delta_micros();           // u64, fixed-point
    for mut anim in &mut query {
        if !anim.playing { continue; }
        
        let frames = asset_server.get(&anim.frames_ref).unwrap();
        let tag = frames.tags.iter().find(|t| t.name == anim.current_animation).unwrap();
        let frame_def = &frames.frames[anim.frame as usize];
        
        // Fixed-point multiplication: delta_ticks × (speed_scale_q16_16 / 65536).
        // Saturating arithmetic, all-integer, bit-identical cross-OS.
        let signed_delta = (delta_ticks as i64) * (anim.speed_scale_q16_16 as i64) / 65536;
        if signed_delta >= 0 {
            anim.elapsed_ticks = anim.elapsed_ticks.saturating_add(signed_delta as u64);
        } else {
            anim.elapsed_ticks = anim.elapsed_ticks.saturating_sub((-signed_delta) as u64);
        }
        
        let duration_ticks = (frame_def.duration_ms as u64) * 1000;  // ms → μs
        if anim.elapsed_ticks >= duration_ticks {
            anim.elapsed_ticks -= duration_ticks;
            advance_frame(&mut anim, tag);
        }
    }
}
```

**`frame_progress` em extract phase (PresentWorld, HR-5 exempt):**

```rust
fn extract_sprite_animator_progress(/* ... */) {
    let progress_f32 = anim.elapsed_ticks as f32 / duration_ticks as f32;
    // Output em RenderInstance ou similar, GPU consume.
}
```

Sub-frame progress (`frame_progress`) permite tween entre frames pra slow-motion ou time-warp custom — feature opcional Godot tem. Computado em extract phase para evitar f32 em SimWorld.

## 8.6 Hold + Repeat Delay (Phaser pattern)

Animações de jogo bem feitas têm "respiração":
- **Hold (ms)** — pausa no último frame antes do loop (idle de char "respira" antes de repetir).
- **Repeat Delay (ms)** — pausa total entre loops (idle pisca a cada 3 segundos).

```rust
fn advance_frame(anim: &mut SpriteAnimator, tag: &AnimationTag) {
    let new_frame = match tag.direction {
        AnimDirection::Forward => anim.frame + 1,
        AnimDirection::Reverse => anim.frame.saturating_sub(1),
        AnimDirection::PingPong => { /* toggle pingpong_reverse */ },
        AnimDirection::PingPongReverse => { /* idem inverso */ },
    };
    
    if reached_end_of_tag(new_frame, tag) {
        if tag.hold_ms > 0 && !anim.in_hold_phase {
            anim.in_hold_phase = true;
            anim.elapsed_ms = -tag.hold_ms as f32;  // negative = waiting
            return;
        }
        
        anim.repeat_count += 1;
        if let Some(max) = tag.repeat {
            if anim.repeat_count >= max {
                anim.playing = false;
                emit_signal_animation_finished(anim);
                return;
            }
        }
        
        if tag.repeat_delay_ms > 0 {
            anim.elapsed_ms = -tag.repeat_delay_ms as f32;
        }
        
        anim.frame = tag.from;
        emit_signal_animation_looped(anim);
    } else {
        anim.frame = new_frame;
    }
}
```

## 8.7 Inspector layout

Seção 11 "Animation" (collapsible — só ativa se entidade tem `SpriteAnimator`):

```
▼ Animation                                            [+ Add Animator]
  SpriteFrames asset:  [walk_sheet.spriteframes ▾]    [Open in Timeline]
  Current Animation:   [walk ▾]
  Frame: 4 / 7
  ▒▒▒▒▒▒▒░░░░░░░  Frame Progress: 0.66
  
  Speed Scale: 1.0   [-1] [0] [+1]
  ☑ Playing       Space (toggle)
  ☐ Autoplay
  
  Direction override: [Inherit ▾ | Forward | Reverse | PingPong | PingPongRev]
  Loop override:      [Inherit ▾ | On | Off]
  Hold ms:            0          (read-only se Inherit)
  Repeat Delay ms:    0          (read-only se Inherit)
```

Botões:
- **[Open in Timeline]** abre o timeline editor (módulo Animation futuro) com o SpriteFrames carregado.
- **[+ Add Animator]** anexa Component `SpriteAnimator` ao entity (quando ausente, seção mostra só esse botão).

## 8.8 Default duration_ms vs FPS global

Aseprite usa `duration_ms` absoluto por frame. Godot 4 SpriteFrames suporta **per-frame relative duration multiplier + animation_fps multiplier** (não "só FPS global" como originalmente afirmava o spec — verificado em docs oficiais [`class_spriteframes`](https://docs.godotengine.org/en/stable/classes/class_spriteframes.html)). **PH2D adopta `duration_ms` absoluto** (não relative) porque:
- Permite per-frame timing direto (anticipation hold = duration_ms grande no key frame; snap rápido = duration_ms pequeno) sem precisar mental math de multiplier.
- Compatibilidade lossless com Aseprite import (Aseprite usa ms direto).
- Phaser anim `frames` array também aceita per-frame `duration` ms absoluto.
- Modelo absoluto é estritamente mais expressivo que relative+multiplier — não há informação que relative carrega e absolute não.

UI Inspector mostra duração como leitura (read-only). Edição no Timeline editor (módulo futuro).

## 8.9 Animação simples sem SpriteAnimator (caso default)

Sprite com `hframes/vframes > 1` mas SEM `SpriteAnimator` Component → **estático na frame atual** (`Sprite.frame`). Cycling via código:

```rust
// Game tick (sem SpriteAnimator):
sprite.frame = (sprite.frame + 1) % (sprite.hframes * sprite.vframes);
```

Útil para casos triviais (toggle 2 frames idle, etc.). Inspector mostra `frame` editável; user pode scrubber.

## 8.10 Signals (events disparados pelo SpriteAnimator)

Quando `SpriteAnimator` muda estado, sistema emite signals via ActionBus:

```rust
EditorAction::SpriteFrameChanged { entity, old_frame, new_frame }
EditorAction::SpriteAnimationFinished { entity, animation_name }
EditorAction::SpriteAnimationLooped { entity, animation_name, repeat_count }
EditorAction::SpriteAnimationChanged { entity, old_name, new_name }
```

Frame events com payload tipado (footstep SFX no frame 3 com surface_type) **NÃO** estão aqui — vive no módulo Animation/Timeline editor futuro. Inspector v2 só emite signals básicos de transição.

## 8.11 Caps gateados

| Cap | Valor | Razão |
|---|---|---|
| `SpriteFrames.frames` count | ≤ 4096 | Sanity; >4096 sugere atlas separado |
| `SpriteFrames.tags` count | ≤ 256 | Anim count típico < 50 |
| `AnimationTag.name` length | ≤ 64 chars | Identificador |
| `frame.duration_ms` range | `[1, 60_000]` | 1ms a 60s; <1ms é absurdo, >60s é "estático" |
| `speed_scale` range | `[-100.0, 100.0]` | Clamp sanity |

## 8.12 Aseprite import (lossless)

Aseprite → PH2D mapping completo:
- `.ase` frames + frame durations → `SpriteFrames.frames` com `duration_ms`.
- `.ase` tags + direction + repeat → `SpriteFrames.tags`.
- `.ase` slices → `SpriteFrames.frames[n].named_anchors` (per-frame, vide [07](07_named_anchors.md)).
- `.ase` layers → flatten by default (ou separate sprites se importer config).
- `.ase` blend modes → IGNORADOS no Inspector v2 (precisa shader FX; vide [12_fora_de_escopo.md](12_fora_de_escopo.md)).

## 8.13 Anti-padrões evitados

1. **FPS global multiplier em vez de per-frame ms absoluto** ❌ — Godot 4 tem per-frame relative + FPS multiplier (não "só FPS global" como originalmente afirmado); PH2D adopta ms absoluto por simplicidade e Aseprite parity.
2. **Animação = N arrays separados de frames** ❌ — Godot, Paper2D. Aseprite tags em pool único é estritamente melhor.
3. **Frame events com single-param** ❌ — Unity AnimationEvent. PH2D tipos payload via struct (módulo Timeline futuro, não Inspector).
4. **Onion skin no Inspector** ❌ — Inspector mostra estado; timeline editor mostra contexto temporal.
5. **AnimatedSprite2D ⇄ AnimationTree desconectados** ❌ — Godot proposal #567 aberto há anos. PH2D unifica via `SpriteAnimator` + `AnimationStateMachine` Component (módulo Animation futuro) compartilhando schema.

---

# ⭐ CONSTRUÍDO em 2026-08-23 — e o que MUDOU em relação a esta spec

> Esta seção é **normativa** e sobrepõe-se ao que está acima onde os dois discordarem. O que está
> acima é o desenho de 2026-05; o que está aqui é o que existe, com a medição que moveu cada peça.

## ⛔ O `SpriteFrames` asset NÃO foi construído — o pool já existia

A §8.3 desenha um asset com um `Vec<SpriteFrame>` próprio. Medido antes de o construir, ele
**duplicaria duas coisas que o app já tem**:

- a **grelha** (`Sprite::hframes × vframes`, índice `Sprite::frame`) — o pool que a §4 Sprite Sheet
  autora hoje, e **o único que o renderer amostra por quadro**;
- a **folha autorada** (`ph2d_sprite_sheet::AuthoredSheet`) — regiões nomeadas e ordenadas.

⚠️ E a segunda **não é um sink vivo**: o `SpriteSheetRef` é *proveniência de autoria* («os meus
pixels são a região R da folha F»), e mudá-lo não muda o que se desenha sem reatar a textura.

⇒ **UM pool, UM sink: a grelha e o `Sprite::frame`** — que é o que a própria §8.9 escreve para o
caso sem animador. Uma animação é um **intervalo nomeado sobre as células que a sprite já tem**.
Zero asset novo, zero pixels duplicados, e a persistência vem do registro de componentes.

| a spec dizia | o que existe |
|---|---|
| `SpriteFrames` asset + `AssetHandle` | ⛔ não construído — ver acima |
| `SpriteFrame { texture_ref, duration_ms, … }` | ⛔ o frame é uma **célula da grelha** |
| `AnimationTag` | ✅ `ph2d_ecs::AnimationTag`, com `frame_ms` por TAG |
| `SpriteAnimator` | ✅ `ph2d_ecs::SpriteAnimator` |
| `elapsed_ticks: u64` fixed-point | ✅ e a lei pura **nunca vê um float** |
| `speed_scale_q16_16` | ✅ `SPEED_ONE_Q16 = 65 536`, teto `±100×` |
| `in_hold_phase: bool` | ⛔ **dobrado na duração** do último frame — mesmo resultado observável, menos um estado a dessincronizar |
| tags ≤ **256** | ⚠️ **64** — ver abaixo |
| duração por-FRAME | ⛔ ver abaixo |
| signals no ActionBus (§8.10) | ✅ **existem** desde 2026-08-23 — mas no **outbox** (`ph2d-runtime`), não no ActionBus; ver abaixo |
| Aseprite import (§8.12) | ✅ **existe** desde 2026-08-23 — `ph2d-aseprite` + `ase_import.rs` |

## ✅ Os SINAIS (§8.10) existem — e saem pelo OUTBOX, não pelo ActionBus

A spec desenha quatro `EditorAction`. ⚠️ **Ela é anterior ao `ph2d-runtime`**, que é onde este app
resolveu a mesma pergunta desde então: o produtor **publica** e cada consumidor lê com o próprio
cursor ([ADR-0143](../architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md)
+ ADR-0075). Um sinal no ActionBus faria o motor de animação **chamar** o editor, que é o oposto do
norte. A timeline e a física já publicam ali; a §11 passou a ser o terceiro produtor.

**O que shipa, e o que ficou de fora:**

| a spec pede | o que existe |
|---|---|
| `SpriteAnimationFinished` | ✅ `AnimationTag::signal_on_finish` — um nome **autorado**, vazio = calada |
| `SpriteAnimationLooped { repeat_count }` | ✅ `signal_on_loop`, e a contagem viaja em `SignalOrigin::Animation::cycles` |
| `SpriteFrameChanged { old, new }` | ⛔ **fora, por FREQUÊNCIA**: ele dispara ~12×/s **por sprite**, e o que consome um frame já lê o `Sprite::frame` direto. A própria §8.10 põe os eventos por-frame no editor de timeline futuro |
| `SpriteAnimationChanged { old, new }` | ⛔ **fora, por NATUREZA**: trocar a animação é um gesto do artista no Inspector, não um facto da cena. Publicá-lo faria um clique no painel parecer gameplay |

⚠️ **Dois nomes AUTORADOS e não um mais uma fase** — é a lei que a física já escreveu para os
contatos (`SignalOrigin::Contact`): *acabar* e *dar a volta* distinguem-se por serem **nomes
diferentes, autorados em dois sítios**. Um consumidor casa numa string e nunca precisa perguntar a
origem.

⚠️ **Vazio = calada, e é o default.** A maioria das animações não tem nada a dizer; uma que grita
por omissão enche o canal de eventos que ninguém pediu — e o consumidor não teria como distinguir
os que importam.

⚠️ **Um tique atrasado colapsa num sinal só, com a contagem dentro** (a lei que o
`SignalOrigin::Motion` já escreveu: *o colapso é lossy e o número é o que ele descarta*). Uma janela
que esteve parada fecharia dez ciclos de uma vez, e dez passos que ninguém deu é ruído.

⚠️ **A pré-visualização é MUDA:** uma folha em pintura toca sobre uma cópia do animador, e ela
existe para mostrar o movimento — não para fazer acontecer coisas na cena. Sem esta metade, pegar
no pincel tocaria um som.

## ⏳ Duração por-FRAME: a recusa foi REABERTA no mesmo dia, e agora tem quem a produza

**A recusa original, verbatim:** *ela existe na spec por paridade com o Aseprite — e não há
importador de `.ase`, por isso ninguém a produziria. A própria §8.8 diz que editá-la é do editor de
timeline futuro, não do Inspector. O caso de uso que ela nomeia (*anticipation hold*) é servido pelo
`hold_ms` que a §8.6 já especifica. «Um campo sem quem o escreva é autoria sem consumidor».*

⚠️ **A premissa dissolveu-se em 2026-08-23:** o importador de `.ase` foi construído, e ele **produz
exactamente esse dado** — um `.ase` traz `duration_ms` por quadro, e um artista que ponha um *hold*
no meio de uma tag tem uma tag que o nosso `frame_ms` único não sabe exprimir. *Quem move o número
que tornava algo inalcançável tem de reconferir a nota.*

**O que shipa hoje:** o importador **aproxima pela duração mais comum da tag e DIZ**, nomeando a tag
e o número (`AseTag::uniform_duration_ms` devolver `None` **é** a informação). ⛔ Aproximar em
silêncio seria a resposta errada com a certeza da certa.

**O que a decisão de produto custa, para quem a tomar:** um `Vec<u32>` por tag (ou um override
esparso), o `PROJECT_SCHEMA` a mover, e a UI — que a §8.8 já diz não ser do Inspector. O `hold_ms`
continua a servir o caso do último frame; o que ele **não** serve é um *hold* no MEIO da tag, que é
o que um `.ase` real traz.

## ⚠️ O cap de tags desceu de 256 para 64, e a razão é a da própria spec

A §8.11 justifica o 256 com *«a contagem típica de animações é < 50»*. O que ela não pesa é que o
painel tem de **alcançar** cada uma: um modelo que aceita o que o painel não mostra produz estado
**inalcançável por gesto nenhum**, e este módulo já pagou essa classe duas vezes (os quatro modos
mudos da §9 Sampling; o cap de âncoras que o gate
`the_model_stops_exactly_where_the_panel_runs_out_of_rows` passou a prender).

64 é o que a lista da §11 mostra, é o mesmo teto das âncoras, e está acima do «típico < 50».

## As leis que a implementação fixou (cada uma com gate e mutação)

1. ⭐ **Tocar UMA vez pára no ÚLTIMO frame**, não volta ao primeiro. A pose final de um *attack*
   **é** o resultado do gesto; voltar deixaria a sprite em repouso. É o que o Godot e o Phaser
   fazem, e a spec não o diz.
2. **Ping-pong não repete as pontas** — `0 1 2 1 0`, e não `0 1 2 2 1 0`.
3. **`hold_ms` alonga só o último frame do ciclo; `repeat_delay_ms` só conta quando ainda vem
   outro ciclo.** Cobrá-lo na última volta prenderia a animação depois de ela já ter acabado.
4. **Velocidade negativa toca ao contrário — ela não faz o tempo andar para trás.** Somar um delta
   negativo ao acumulador obrigaria a «desavançar» frames com o resto do frame anterior, que é um
   segundo modelo de tempo dentro do mesmo estado.
5. **Um intervalo de uma célula não avança e não conta ciclos** — senão um `repeat` finito
   terminaria à velocidade do relógio.
6. **O intervalo é DERIVADO contra a grelha de hoje.** Encolher `hframes` não deixa uma tag
   gravada a apontar para fora; ela lê-se como «out of grid» na lista e o tocador di-lo.
7. **O tique corre no PASSO FIXO** e escreve `Sprite::frame` **só quando ele muda** — o `Sprite` é
   `SimComponent` e o undo regista por diff.
8. ⚠️ **Recuperar um engasgo numa chamada é igual a recuperá-lo passo a passo**, porque a lei tem
   o seu próprio laço interno. A primeira versão corria N chamadas com uma justificação **falsa**,
   e foi uma mutação que a derrubou; hoje há gate a prender a equivalência.

### As leis do TRANSPORTE (auditoria de 2026-08-23 — [doc 21](21_auditoria_da_animacao_2026-08-23.md))

9. ⭐ **«Pausado» e «terminado» leem-se igual no `playing == false` e NÃO são a mesma coisa.**
   [`SpriteAnimator::is_finished`] distingue-os, e é o que faz o interruptor ser um gesto vivo:
   sem ele, ligar uma tag de uma volta já gasta deixa a imagem na ponta com o contador cheio e o
   primeiro passo de `advance` volta a fechar o ciclo — **no mesmo tique**.
10. ⭐ **REBOBINAR é repor o ciclo E pôr a IMAGEM no princípio**, e a ponta certa segue a direção
    efetiva ([`entry_frame`]). Repor só os contadores é um botão que não faz nada — e, com um
    `repeat` finito, um botão que não faz nada **duas** vezes.
11. ⭐ **Escolher outra animação começa-a do PRINCÍPIO dela**, mesmo quando os intervalos se
    sobrepõem. O `advance` só reposiciona um frame que caia **fora** do intervalo, e a tese do
    modelo (§8.2) é justamente que as animações **partilham** o pool — *a propriedade que faz o
    modelo valer a pena era a que partia esta lei*.
12. ⭐ **A reprodução que se ESGOTOU volta a tocar quando alguém lhe toca — e escolher outra
    animação é tocar-lhe.** Uma **pausa explícita** não é tocada por isto, então folhear a lista em
    silêncio continua possível. Uma lei, duas portas (a caixa e a lista); o gate afirma as duas
    metades.
13. ⚠️ **A caixa «Playing» pergunta à CENA, nunca à própria memória.** A §11 pinta *e decide* a
    partir do snapshot; o `WidgetStore` só publica o estado para a acessibilidade. O contrário era
    uma dupla fonte de verdade que divergia no instante em que o **motor** mudava o facto — e foi
    exatamente o que o Enio reportou como *«às vezes preciso clicar mais de uma vez»*.
14. ⭐ **A BARRA DE FRAMES É UM SLIDER, e ela mede POSIÇÃO** (`passo / (total-1)`), não progresso
    (`(passo+1) / total`). Uma barra que só informa pode medir «quanto já passou»; uma que se
    agarra tem de medir «onde está», senão o polegar não pousa em cima da célula. ⚠️ **Agarrá-la
    PAUSA** — enquanto a reprodução corre, o tique também escreve o `Sprite::frame`, e o dedo e o
    relógio disputariam o mesmo campo. ⚠️ A posição e a célula são **uma lei em dois sentidos**
    (`scrub_position` ↔ `scrub_cell`), depois de uma mutação sobrevivente ter mostrado que ela
    vivia em três cópias.
15. **A seleção múltipla DIZ-SE.** As edições da §11 não se espalham (o índice que carregam só
    significa alguma coisa na biblioteca da entidade ativa), e a seção avisa-o **antes** de
    oferecer controlo nenhum.

[`SpriteAnimator::is_finished`]: ../../crates/ph2d-ecs/src/sprite_anim.rs
[`entry_frame`]: ../../crates/ph2d-ecs/src/sprite_anim.rs

## O Inspector §11, e onde ele difere da §12

**Clicar numa linha da lista ESCOLHE a animação que toca** — e isso vai ao barramento. Na §12,
clicar numa linha muda só a ficha aberta e é um facto da UI. A diferença é do domínio: numa
biblioteca de animações, *a que se vê* e *a que toca* serem a mesma é o que o artista espera (o
`AnimationPlayer` do Godot faz assim), e separá-las pediria um seletor com as mesmas entradas da
lista logo abaixo dele.

⛔ **Sem tocador, a seção mostra um botão** (`+ Add Animator`) e mais nada.

**Smoke:** `PH2D_ANIM_SMOKE=1` — uma tira de 8 células e três animações que **se sobrepõem** sobre
ela, que é a tese do modelo do Aseprite (§8.2) e o que N arrays separados não exprimem sem
duplicar pixels.

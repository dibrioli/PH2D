# 10 — Animation Assist (timeline frame-based)

> Frame-by-frame timeline embutido no Painter. **NÃO** confundir com Procreate Dreams (motion tracks, keyframes) — esse é out-of-scope, vide [12_fora_de_escopo.md](12_fora_de_escopo.md) §12.4.

## 10.1 Modelo conceitual

- **Cada layer = 1 frame.** OU
- **Cada grupo de layers = 1 frame** (via Combine Down / Group).

Não há keyframes interpoláveis, não há tracks, não há audio. Modelo é frame-by-frame puro (estilo flipbook Disney). Espelha exatamente Procreate Animation Assist clássico.

Justificativa: animação sofisticada é responsabilidade de **outro tool / outro app**. O Painter resolve "preciso animar 4 frames de loop walk cycle" e similares — workflow de ilustração com animação leve, não pipeline de animação profissional.

## 10.2 Ativação

Acesso: Actions → Canvas → **Animation Assist** (toggle on/off).

Quando ativado:
- Timeline horizontal aparece no bottom do canvas.
- Layer panel ganha colunas de timing (vide §10.3).
- Sidebar ganha botão "Play" (preview animation).

Toggle off oculta timeline mas mantém o estado (não destrutivo).

## 10.3 Timeline (UI)

```
┌──────────────────────────────────────────────────────────────────┐
│                          CANVAS                                  │
│                                                                  │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│  [BG] │ F1 │ F2 │ F3 │ F4 │ F5 │ F6 │ F7 │ F8 │ [FG] │ [+ Frame] │
│   ▣   │ ▣  │ ▣  │ ●  │ ▣  │ ▣  │ ▣  │ ▣  │ ▣  │  ▣   │           │
├──────────────────────────────────────────────────────────────────┤
│  [▶] [⏸] [⏭] [⏮] [↻]    24 fps   Loop ▼   Onion 2/2 ▼            │
└──────────────────────────────────────────────────────────────────┘
```

- **Frames** (F1...Fn) — cada um é uma layer (ou group) com thumb.
- **`●`** indica o frame ativo (sendo editado).
- **`[BG]`** — Background layer (sempre visível, locked).
- **`[FG]`** — Foreground layer (sempre visível por cima, locked).
- **`[+ Frame]`** — adiciona novo frame (cria layer nova abaixo do ativo).
- **Drag frames** — reorder.
- **Long-press frame** — frame properties popover (hold duration, etc.).

## 10.4 Playback controls

| Control | Ação |
|---------|------|
| **Play** (▶) | Inicia preview animado |
| **Pause** (⏸) | Pausa preview |
| **Frame next** (⏭) | Avança 1 frame (paused state) |
| **Frame prev** (⏮) | Retrocede 1 frame |
| **Reset** (↻) | Volta ao frame 1 |
| **fps select** | Frame rate 1–60 (default 12) |
| **Loop mode select** | Loop / Ping-Pong / One Shot |
| **Onion config** | Onion skin frames + colors |

### 10.4.1 Atalhos

| Atalho | Ação |
|--------|------|
| `Space` | Play/Pause |
| `→` (right arrow) | Next frame |
| `←` (left arrow) | Previous frame |
| `Home` | First frame |
| `End` | Last frame |
| `Ctrl+Shift+→` | Move active frame right (reorder) |
| `Ctrl+Shift+←` | Move active frame left |

## 10.5 Frame rate

Range 1–60 fps. Default **12 fps** (clássico animação 2D).

Tipo comum:
- 6 fps — limited animation, choppier feel
- 12 fps — classic 2D animation
- 24 fps — cinematic
- 30 fps — broadcast
- 60 fps — smooth, high-end

## 10.6 Loop modes

3 modos:

| Modo | Comportamento |
|------|---------------|
| **Loop** | F1 → F2 → ... → Fn → F1 (cycle) |
| **Ping-Pong** | F1 → F2 → ... → Fn → Fn-1 → ... → F1 → F2 → ... (bounce) |
| **One Shot** | F1 → F2 → ... → Fn → stop (play uma vez) |

## 10.7 Onion skinning

Visualização de frames antes/depois do ativo, fade-out.

### 10.7.1 Configuração

```
┌─────────────────────────────┐
│ Onion Skin                  │
├─────────────────────────────┤
│  Previous frames: [2 ──●──] │  (0-6)
│  Next frames:     [2 ──●──] │
│  Opacity prev:    [40% ●─]  │
│  Opacity next:    [40% ●─]  │
│  Colors:                    │
│    ● Auto (fade)            │
│    ◯ Red prev / Blue next   │  (classic Disney)
│    ◯ Custom                 │
├─────────────────────────────┤
│  ☐ Secondary color          │  (additional tint)
│  ☐ Loop onion               │  (mostra frames wrapping em modo Loop)
└─────────────────────────────┘
```

### 10.7.2 Render

Onion frames composted **antes** do frame ativo no compositor, com opacidade reduzida + tint color aplicado via blend mode "Color".

Não afetam saving — onion skin é puramente preview visual.

### 10.7.3 Atalhos

| Atalho | Ação |
|--------|------|
| `O` | Toggle onion skin on/off (mantém config) |
| `Shift+O` | Open onion config popover |
| `]` em timeline | Increase onion next |
| `[` em timeline | Increase onion prev |

## 10.8 Background / Foreground layers

Frames especiais:

### 10.8.1 Background layer

- Visível em **todos os frames**, embaixo de tudo.
- Locked por default (não-editável até unlock).
- Útil para: céu fixo, cenário background, gradient base.
- Criado via Long-press primeiro frame → "Set as Background".

### 10.8.2 Foreground layer

- Visível em **todos os frames**, em cima de tudo.
- Locked por default.
- Útil para: overlay UI, vinheta, lighting effect.
- Criado via Long-press último frame → "Set as Foreground".

Apenas **1 BG + 1 FG** por canvas.

## 10.9 Hold duration por frame

Cada frame tem `hold = N` (default 1). O frame fica visível N "ticks" durante o playback.

Permite timing irregular sem duplicar frames:
- Frame de "personagem parado" segura por 6 ticks.
- Frames de "ação rápida" seguram por 1 tick.

UI: long-press no frame thumb → slider "Hold duration: 1–10". Mostrado como número pequeno em cima do thumb (e.g., "×3").

## 10.10 Frame operations

Long-press no frame thumb:

```
┌─────────────────────────────────┐
│  Duplicate                      │
│  Delete                         │
│  Clear                          │
│  ─────────────────────────────  │
│  Hold:    [3 ──●─────]          │
│  ─────────────────────────────  │
│  ○ Background                   │
│  ○ Foreground                   │
│  ○ Regular                      │ (current)
│  ─────────────────────────────  │
│  Move to start                  │
│  Move to end                    │
└─────────────────────────────────┘
```

## 10.11 Group como frame

Quando frame é um group (múltiplas layers combinadas), Painter sabe via metadata. Grupo aparece como 1 thumb na timeline, mas usuária pode expandir o group no Layer panel para editar layers internas. Útil quando 1 frame tem múltiplas layers (line art + color + shading) que se beneficiam de organização.

## 10.12 Animation export

Vide [09_export_interop.md](09_export_interop.md) §9.5.

Export dialog Animation:

```
┌──────────────────────────────────────────┐
│ Export Animation                         │
├──────────────────────────────────────────┤
│ Format:                                  │
│   ◯ Animated GIF                         │
│   ◯ Animated PNG (APNG)                  │
│   ● MP4                                  │
│   ◯ HEVC                                 │
│   ◯ Animated WebP                        │
│   ◯ Frame sequence (PNG folder)          │
├──────────────────────────────────────────┤
│ Frame range:                             │
│   ● All frames                           │
│   ◯ Range: from [1 ●─] to [N ─●]        │
├──────────────────────────────────────────┤
│ Loop mode (for export):                  │
│   ● Match timeline (Loop)                │
│   ◯ Loop                                 │
│   ◯ Ping-Pong  ← bake; export it duplicate│
│   ◯ One Shot                             │
│   ◯ Loop N times: [3 ──]                 │
├──────────────────────────────────────────┤
│ Frame rate (for video): [24 ──]          │
│ Resolution: [Full / Half / Quarter]      │
│ Quality (lossy): [80 ───●────]           │
├──────────────────────────────────────────┤
│         [ Cancel ]    [ Export ]         │
└──────────────────────────────────────────┘
```

## 10.13 Animation Assist + Drawing Guides

Compatibilidade: drawing guides funcionam dentro de frames. Symmetry assist faz cada frame ter symmetry independente (usuária pode rotacionar entre frames com symmetry on/off via guide visibility).

## 10.14 Animation Assist + Reference Companion

Reference window útil durante animação para mostrar:
- **Frame anterior** em "still mode" (manualmente importada).
- **Image reference** estática.
- **Canvas mirror** (mas pulando frame — exibe o canvas main, que mostra o frame atualmente sendo editado).

## 10.15 Limitações conscientes

| Limitação | Razão |
|-----------|-------|
| Sem keyframes interpoláveis | Out-of-scope (Procreate Dreams territory) |
| Sem motion tracks | Idem |
| Sem audio | Idem |
| Sem easing curves | Idem |
| Sem cameras virtuais | Idem |
| Max 240 frames (timeline cap) | Frame-based muito longa = bitmap heavy; usuária deve segmentar |
| Tudo é raster | Painter é raster (§12.1) |

## 10.16 Schema persist

`AnimationAssistState` em [`09_export_interop.md`](09_export_interop.md) §9.1.1:

```rust
#[derive(Serialize, Deserialize)]
pub struct AnimationAssistState {
    pub version: u32,                 // = 1
    pub frame_rate: u32,              // 1..=60
    pub loop_mode: LoopMode,
    pub onion_config: OnionConfig,
    pub frames: Vec<FrameMeta>,       // ordered; each refs a LayerId or GroupId
    pub background_layer: Option<LayerId>,
    pub foreground_layer: Option<LayerId>,
    pub holds: HashMap<LayerId, u32>, // override default hold=1
}

#[derive(Serialize, Deserialize)]
pub struct OnionConfig {
    pub prev_frames: u8,              // 0..=6
    pub next_frames: u8,              // 0..=6
    pub opacity_prev: f32,
    pub opacity_next: f32,
    pub color_mode: OnionColorMode,   // Auto | RedBlue | Custom { prev: Oklch, next: Oklch }
    pub secondary_color: bool,
    pub loop_onion: bool,
}
```

## 10.17 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `anim_frame_each_layer` | `ph2d-painter-anim` | Cada layer raster vira 1 frame na timeline |
| `anim_group_as_frame` | idem | Group de N layers = 1 frame thumb |
| `anim_bg_fg_locked` | idem | BG/FG layer não-editável por default; unlock toggle libera |
| `anim_onion_prev_next_render` | idem | Onion config 2/2 renderiza 2 frames antes + 2 depois com opacity correta |
| `anim_loop_mode_pingpong` | idem | Ping-pong playback ordem F1→F2→...→Fn→Fn-1→...→F1 |
| `anim_hold_duration_persistence` | idem | Hold=3 num frame → playback fica 3 ticks lá |
| `anim_export_gif_frame_rate` | idem | Export GIF 12fps gera GIF com delays corretos |
| `anim_export_mp4_loop_n` | idem | Export MP4 "loop N=3" gera 3 cycles concatenados |
| `anim_max_frames_cap_240` | idem | Tentar criar frame 241 falha com erro claro |
| `anim_state_persist_in_painter_file` | `ph2d-tool-painter` | AnimationAssistState salva no `.ph2d-painter` e carrega de volta |

**Continua em:** [11_ux_chrome.md](11_ux_chrome.md) — UX, chrome, layout, multi-plataforma.

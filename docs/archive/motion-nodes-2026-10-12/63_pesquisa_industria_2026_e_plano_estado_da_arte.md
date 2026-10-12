# ARQUIVO — 63_pesquisa_industria_2026_e_plano_estado_da_arte.md (história, 57 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`63_pesquisa_industria_2026_e_plano_estado_da_arte.md`](../../Motion%20Nodes/63_pesquisa_industria_2026_e_plano_estado_da_arte.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-182,214-228,255-390` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### 3.2 Gap por nó existente (os que a referência expõe mais fundo)

| Nó (params hoje) | ADICIONAR (da referência) |
|---|---|
| `force.wind` (angle·strength·gust·gust_freq·seed) | **modelo alvo-velocidade** (`airresist·(targetv−v)` — não explode; POP Wind) · cluster NOISE completo · per-axis |
| `force.drag` (coefficient) | `directional_scale` (anisotrópico) · `goal_velocity` (arrasta rumo a v≠0) · `rotational_drag` (se rotação ganhar velocidade angular) |
| `force.attractor` (target·strength·radius·curve·repel) | `kill_radius` (mata ao chegar) · `reversal_distance` (inverte perto — não orbita) · `peak_force_distance` (platô) · modos **Follow/Predict Intercept** · janela min/max · `swirl` |
| `force.curl` (strength·scale·speed·octaves·seed) | `lacunarity`·`roughness`·`offset`·`pulse` (cluster) · *collision avoidance* (contorna `sim.collide` — P2, caro) |
| `force.vortex` (center·strength·radius·clockwise) | **`lift` + `suction`** (o Axis Force do Houdini unifica órbita+elevação+sucção com UM falloff) · `soft_edge` · `inner/outer_strength` · `treat_as_wind` |
| `motion.noise` (channel·amplitude·scale·octaves·roughness·type·speed·seed) | `lacunarity` · `offset` · modos de RANGE · **`loop_period`** · transform do campo (pos/rot/scale) · controles cellular (jitter, métrica euclid/manhattan) · `stagger` (defasa por índice) |
| `motion.oscillator` (channel·wave·amplitude·frequency·phase_stagger·offset·phase) | **Time Mode: Seconds/BPM** · **custom wave por CURVA** (D2) · min/max (RANGE) · `strength_fade_to_zero` (entra do zero, não do min) |
| `motion.stagger` (channel·min·max·ease_curve·ease_dir·reverse) | **Graph (curva D2)** no lugar do ease enumerado · `offset` · auto-invert min>max |
| `motion.wiggle` (channel·amplitude·frequency·seed) | `octaves` · `loop_period` · separate channels |
| `motion.spring` (channel·tension·friction) | modos **Average/Blend/Spring** (o Delay do C4D generalizado: qualquer propriedade chega com atraso/mola) · input effect Position/Force (CHOP Spring) |
| `motion.emitter` (rate·life·speed·angle·spread·x·y·seed·max·size) | **shape** (point/circle/rect/curva/perímetro) · direction inward/outward · **burst** (count/time/period/probability) · `probability` · `inherit_velocity` · duration/interval · emissão por distância (§2.3) |
| `motion.falloff` (shape·curve·center·radius·invert) | → dissolve na família `field.*` (D1); o nó atual vira alias/compat |
| `motion.clone` (count·distance·angle·center) | multi-fonte iterate/random/blend/sort · **time_offset por clone** · step cumulativo (espiral) |
| `motion.trail` (length·fade·shrink) | limite por tempo · path type (pontas/linha contínua — ribbon) · time offset |
| `value.map_range` (in/out·clamp) | **interpolation**: linear/stepped(+steps)/smoothstep/smootherstep (Blender) · curva (D2) |
| `motion.grid` (rows·cols·gap) | `form` (círculo/forma recorta a grade) · `fill` (casca) — C4D Grid Array |
| `motion.look_at` (offset) | `strength` (peso 0..1) · up/flip |
| `motion.sort` (key·descending·center·seed) | `group_id` (ordena DENTRO de grupos) · peso-como-coluna (Sort Elements) |
| `motion.mixer` (mode) | peso POR entrada |
| `motion.color_ramp` (2 stops) | N stops via Gradient (D2) · interpolações por stop |
| `sim.spawn` (rate·scatter·seed) | burst · probability (espelha o emitter) |
| `motion.distribute_poisson` (radius·width·height·seed) | `density` como campo (Blender Density Max × field) |

**Auditoria D12 junto:** para cada nó da tabela, conferir o DEFAULT (nada nasce inerte) e o neutro (`enabled=off`/strength=0 ⇒ identidade exata, byte-idêntica — a lei que a rack de áudio já pratica).

---

### 4.2 Tabela roubar/temos (com custo; ranking cruzado dos 6 dumps)

| Padrão | Origem | Status | Custo |
|---|---|---|---|
| Header cor por categoria · véu inerte · marcha (dashes) · massa · influência · probe+sparkline · readouts · stamps · backdrops+paleta · reroute/waypoints · knife · busca no add · subgrafos · smart-connect | — | **✓ temos** (docs 35–62) | — |
| **Params essenciais no nó** (1–3 rows, declarados no `register_ui`) | Cavalry DG rows · Blender | FALTA | Médio — **maior alavanca visível** |
| **Proveniência por-param com cor + storage sem destruição** (constante/expressão/dirigido) | TD 4 modos | mecanismo ✓ (params dirigidos doc 58, expression) — falta a UI | Médio |
| **Gesto "drive by…"** no param (cria+liga lfo/expression/random) | Niagara Dynamic Inputs | FALTA (1 clique sobre mecanismo existente) | Baixo |
| **Value ladder** (MMB: degraus 0.001→100, grosso+fino num gesto) | Houdini/TD/Nuke | FALTA | Baixo |
| **Forma de socket** = coluna (◇) vs valor uniforme (○); link ilegal = vermelho | Blender | FALTA (temos cor por tipo?) — conferir `paint_wire` | Baixo |
| **Pips de borda** Bypass/Solo/Viewer clicáveis (alvos escalam com zoom — anti-padrão TD anotado) | TD flags | FALTA | Médio |
| **Cook-time inline** sob o nome (toggle de profiling) | Notch · Houdini MMB | FALTA (Cook já mede — doc 53 mediu fps) | Baixo |
| **Drop-no-fio com highlight** + **shake-to-disconnect** (fio se cura) | Nuke | FALTA (knife ✓) | Baixo/Médio |
| **Criação com modificadores**: Shift=branch · Ctrl=replace · `A`,Enter=repete último | Nuke | FALTA (busca ✓) | Baixo |
| **Ctrl+F find-navega** (mesma busca do add, apontada aos nós existentes) | Notch | FALTA | Baixo |
| **Zone strap** — a "cinta" visual do par `sim.zone` in/out abraçando o miolo | Blender zones | PARCIAL (sim.zone existe; a cinta é desenho) | Baixo |
| **Inspection Index** — qual passo/iteração o probe mostra dentro de zona | Blender | FALTA | Baixo |
| **Spreadsheet de colunas** (painel: instância×coluna, filtros, estado cru/cozido) | Niagara/Blender | FALTA (D8) | Médio |
| **Static-frame stamps + toggle global** (política de custo de preview) | Nuke (a lição: stamps vivos matam o script) | FALTA | Baixo |
| **Gizmo de canvas bidirecional** (falloff/attractor/curvas arrastáveis NA TELA) | Blender 4.3 · Cavalry | FALTA (Camada 6 do design v1 previa overlays; o gizmo é o passo além: ESCREVE) | Médio/Alto |
| **Heatmap por instância** do campo selecionado (azul→vermelho no canvas) | C4D Fields Color · Camada 4 design v1 | FALTA | Baixo (1 tint por coluna) |
| **Expose de subgrafo com dialog** (widget/range/grupo/`visible_if`/batch) | Substance | FALTA (subgrafo ✓) | Alto — a fundação de assets |
| **Menu/enum autorável** em subgrafo (Menu Switch) | Blender 4.1 | FALTA | Médio |
| **Minimap opt-in domável** | Notch/Fusion | FALTA | Médio |
| **Warning node + propagação** (subgrafo avisa pra cima, erro borbulha) | Blender 4.3 · Houdini | PARCIAL (erros por nó?) | Médio |


# O estado MEDIDO da nossa UI (2026-08-30)

> Toda a tabela abaixo saiu de comando corrido nesta worktree (`line/UIUX`, base `066b4f92e`).
> ⛔ **Nenhum número aqui foi escrito de memória.** O comando que produz cada um está ao lado,
> porque estes números envelhecem e a próxima janela tem de os poder recontar.

## §1 — O tamanho da superfície

| grandeza | medido | comando |
|---|---:|---|
| **ids de widget registados** (`pub const … : NodeId`) | **2 073** | `grep -rho 'pub const [A-Z0-9_]*: NodeId' crates/ph2d-editor-core/src/ids/ \| wc -l` |
| ficheiros em `ids/` | 71 | `find crates/ph2d-editor-core/src/ids -name '*.rs' \| wc -l` |
| LOC em `ids/` | 12 113 | idem + `xargs cat \| wc -l` |
| **itens de menu de contexto** (`CTX_MENU_*`) | **148** | `grep -rho 'CTX_MENU_[A-Z0-9_]*' … \| sort -u \| wc -l` |
| pills do rail superior (`RAIL_*`) | 29 | idem com `RAIL_` |
| crates de painel (`ph2d-panel-*`) | 25 | `ls crates/ \| grep -c ph2d-panel` |
| widgets primitivos | 53 | `ls crates/ph2d-editor-core/src/widget/ \| wc -l` |
| gates de arquitetura no editor-core | 67 | `ls crates/ph2d-editor-core/tests/ \| wc -l` |
| LOC de `ph2d-editor-core` | 90 440 | `find … -name '*.rs' \| xargs cat \| wc -l` |
| LOC de `ph2d-tokens` | 5 190 | idem |

⭐ **O número que manda é o 2 073.** Uma superfície desse tamanho **não se redesenha à mão** —
qualquer proposta que exija tocar cada id um a um está morta antes de começar. A spec nova tem
de ser **derivável**: ou os widgets nascem de uma tabela, ou o redesenho não acontece.

*Isto não é teoria: o `CLAUDE.md` §5 já regista que **o único painel 42/42 limpo na caça aos
knobs mortos foi o gerado por TABELA** — «um painel derivado de uma tabela não tem onde esconder
um knob morto».*

## §2 — Os painéis, por tamanho

| crate | LOC | ficheiros de secção |
|---|---:|---:|
| `ph2d-panel-inspector` | **27 724** | 33 |
| `ph2d-panel-vector` | 27 696 | — |
| `ph2d-panel-timeline` | 23 594 | — |
| `ph2d-panel-painter-layers` | 23 285 | — |
| `ph2d-panel-motion-graph` | 12 390 | — |
| `ph2d-panel-audio-editor` | 7 839 | — |
| `ph2d-panel-sculpt3d` | 7 708 | — |
| `ph2d-panel-motion-params` | 7 286 | — |

`for c in $(ls crates/ | grep ph2d-panel); do echo "$(find crates/$c -name '*.rs' | xargs cat | wc -l) $c"; done | sort -rn`

⚠️ **Quatro painéis passam dos 23 000 LOC cada.** O Enio chamou-lhes *"extremamente grandes e
mal organizados"* — a medição concorda, e diz **onde**: o Inspector sozinho tem **33 ficheiros
de secção**, e cada um é uma decisão de layout tomada isoladamente.

## §3 — Os tokens

`docs/design/tokens.json` é a fonte; `crates/ph2d-tokens` deriva dela.

| grupo | tokens |
|---|---:|
| **themes** (4 temas × cor + sombra) | **273** |
| typography | 24 |
| chrome | 21 |
| spacing | 9 |
| radius | 7 |
| motion | 7 |
| z | 6 |
| stroke | 5 |
| density | 3 |
| **TOTAL** | **355** |

Slots distintos no macro `color_tokens!`: **83** (`crates/ph2d-tokens/src/color.rs:299`).
Temas: `forge` (escuro+magenta, default) · `workshop` (escuro+ciano) · `sunstone` (claro+laranja)
· `blueprint` (claro+azul).

⭐ **`273 / 355 = 77 % dos nossos tokens são cor.** Quando o Enio diz *"menos cores na paleta"*,
ele está a apontar para 77 % do sistema. Cortar os 83 slots para ~40 e os 4 temas para 2 tira
~150 tokens — mas o corte só é seguro depois de contar **quantos slots cada tema realmente usa
de forma distinta**, que ainda não foi medido (⏳ ver §6).

## §4 — ⭐ Dois eixos existem, e os dois estão presos ao sítio errado

### 4.1 — `PanelLayout` está preso ao TEMA

```rust
// crates/ph2d-tokens/src/theme.rs:53
Self::Forge | Self::Workshop | Self::Sunstone => PanelLayout::Floating,
Self::Blueprint                               => PanelLayout::Sidebar,
```

⛔ **O layout de painéis é uma propriedade da PALETA.** Para ter painéis ancorados o artista tem
de trocar para o tema `blueprint` — e leva o azul claro junto. Não há como ter *"forge escuro
com painéis ancorados"*.

⭐ **A parte boa: o modo ancorado JÁ EXISTE e está testado.** Isto não é uma feature a
construir do zero; é um eixo a **libertar** do enum errado. Consumo hoje: 2 sítios
(`theme.rs:53-54` + o teste).

*É exatamente a separação que Godot e Blender fazem: no Godot o **layout de docks** e o **tema do
editor** são duas coisas independentes e salvas separadamente; no Blender **Workspace** e
**Theme** também.*

### 4.2 — `Density` é preferência do ARTISTA, e não temos escala de HARDWARE

```rust
// crates/ph2d-tokens/src/spacing.rs:89
pub enum Density { Compact, Cozy, Comfortable }   // default: Comfortable
```

Consome exatamente **um** valor: `row_h_px()`. E o doc-comment do `num.rs:15` diz-lho na cara:
*"`Density` já É uma escolha do artista (o modo de linha), não um valor de escala."*

⛔ **Logo: não temos eixo de hardware nenhum.** Não há nada no sistema que diga *"isto está a ser
tocado com uma caneta"* ou *"com um dedo"* — e o app foi desenhado para iPad/Wacom.

⭐ **O Spectrum (Adobe, Apache-2.0) tem exatamente esse eixo, medido e em produção** — e a lei
dele é contra-intuitiva:

| token | desktop | mobile | razão |
|---|---:|---:|---|
| `component-height-50` | 20 px | 26 px | **1,30×** |
| `component-height-100` | 32 px | 40 px | **1,25×** |
| `component-height-200` | 40 px | 50 px | **1,25×** |
| `component-height-300` | 48 px | 60 px | **1,25×** |
| `base-padding-horizontal-2x-large` | 18 px | **14 px** | **0,78×** |
| `base-padding-horizontal-extra-large` | 16 px | **12 px** | **0,75×** |

⭐⭐ **A regra é: o ALVO cresce ~1,25×, o PADDING INTERNO encolhe ~0,77×.** A caixa fica maior
para o dedo/caneta acertar, e o recheio aperta para o conteúdo não se afastar. Ingenuamente
escalar tudo por 1,25 dá o oposto do que a Adobe ship-a — e eles ship-am o Fresco em iPad.

⚠️ E é **opt-in por token**: `base-padding-horizontal-extra-small` não tem `sets` nenhum, é
`8px` nos dois. *A escala não é global; é uma propriedade de cada token que precisa dela.*
(Fonte: `referencias/spectrum-design-data/packages/tokens/src/layout.json`.)

## §5 — A barra superior não tem menus, e o custo está contado

O Enio: *"Na ausência de Menus na barra superior, os painéis se tornaram extremamente grandes e
mal organizados."*

Medido: **148 itens** vivem num menu de contexto global (`CTX_MENU_*`) e **29 pills** no rail.
Há **40 handlers de chrome** (`crates/ph2d-editor-core/src/screens/hero/chrome/`), e a leitura
deles mostra o que são: `settings_text`, `settings_filter`, `settings_motion`, `settings_ppm`,
`settings_present`, `settings_unit`, `theme`, `io_menu`, `view_toggles`, `transport`, 8 toggles
de módulo (`vector_toggle`, `motion_toggle`, `flip_toggle`, `physics_toggle`, `model3d_toggle`,
`sculpt3d_toggle`, `image_tools_toggle`, `tokens_toggle`)…

⭐ **Isso é uma barra de menus — já escrita, espalhada por 40 ficheiros e sem barra.** O
`io_menu.rs` é literalmente *"os itens do menu Ficheiro"*. O trabalho não é inventar menus: é
dar-lhes **um sítio**.

## §6 — ⏳ O que NÃO foi medido nesta passagem (nomeado de propósito)

1. **Quantos dos 83 slots de cor cada tema usa de forma distinta** — sem isto, «cortar a paleta»
   é um palpite. O corte tem de ser derivado de uso, não de gosto.
2. **Quantas LINHAS o maior painel pinta na tela** (não LOC — linhas visíveis). O `CLAUDE.md` §5
   regista o teto do painel de nós a subir de 20 para 24 e um nó a desenhar **1083 px num dock de
   880**; o número equivalente para o Inspector não existe.
3. **Quanto do canvas os painéis flutuantes tapam**, em px e em %. É a foto 1 do Enio,
   transformada num número — e é o que torna a queixa dele um gate em vez de uma opinião.
4. **Quantos dos 2 073 ids são alcançáveis** hoje. A caça de 2026-08-30 achou **34 controlos
   mortos** sobre ~504 seguidos; os outros ~1 570 não foram seguidos.

⚠️ **Os itens 1 e 3 são pré-requisito da spec**, não trabalho posterior: uma decisão de paleta
sem o censo de uso, e uma decisão de docking sem a área tapada, são as duas exatamente o tipo de
«número escolhido em vez de contado» que o `CLAUDE.md` §0.0 proíbe.

# 12 — Painel Vector em cartões: censo, fatias, decisão de Fill/Stroke

Medido no código em 04/11 (`paint_body_plan.rs::corpo`). O brief dizia ~41 secções: são **30 entradas** (29 secções + o
grupo Advanced; as outras saíram em D10, F44, F45, F48d, F50). O painel só existe com a ferramenta vector activa, isto é, no
**Draw** do Vector (`vector_bridge.rs:171`), como aba do dock ao lado do Inspector. «Modo» abaixo = aba do título (F17) e a
condição de `state` que a faz aparecer. «Cham.» = chamadas de controlo no código (um laço conta 1).

| # | Secção | Aba | Aparece com | Cham. | Especial | Repete no Inspector |
|---|---|---|---|---|---|---|
| 1 | Component | Interface | `component_state` | 9 | verbos | não |
| 2 | Transform | Ambas | `current_transform` | 5 | R é arrasto relativo; marca `FORMA` | outra coisa: lá é a pose (ids `INSP_*`), aqui a caixa dos pontos |
| 3 | Anchors | Interface | `anchor_state` | 2 | 4 opções × 2; marca `ANCORAS` | não |
| 4 | Snap | Ambas | sempre | 4 | nasce dobrado; ajuste de ferramenta | não |
| 5 | Frame | Interface | `frame_present` | 2 | grade de presets (tabela) | não |
| 6 | Clip | Interface | `frame_clip` | 1 | marca `MOLDURA_RECORTA` | não |
| 7 | Layout | Interface | moldura ou item | 16 | Grid, Padding, min/max | não |
| 8 | Appearance | Drawing | `current_appearance` | 2 | lista de 19 modos | **sim, MESMOS ids** (T14) |
| 9 | Fill & Stroke | Drawing | sempre | 22 | F44/F45: linhas, ⚙, painel à esquerda, tokens, tracejado, pontas | **sim, MESMOS ids** (T14) |
| 10–11 | Pattern (fill, stroke) | Drawing | `current_texture_pattern` | 11 | reticulado | não |
| 12 | Brush | Drawing | `current_brush` | 4 | | não |
| 13 | Connector | Drawing | `current_connector` | 2 | campos próprios | não |
| 14–16, 18 | Text, Font, Paragraph, Axes | Drawing | `text_visible` | 13 | lista de fontes | não |
| 17 | Text on Path | Drawing | `text_visible` | 5 | | não |
| 19 | Vertex | Drawing | `current_vertex_type` | 5 | | não |
| 20 | Effects | Drawing | `has_target` | 5 | F43: lista, «+», painel à esquerda | não |
| 21 | Filters | Drawing | pilha ou `can_add` | 14 | F43; rampa, mistura | não |
| 22 | ▸ Advanced | Drawing | um membro | 0 | grupo, nasce dobrado | não |
| 23–24 | Blend, Morph | Advanced | sempre | 5 | grade de passos | não |
| 25 | Envelope | Advanced | seleção | 7 | | não |
| 26 | Pattern on Path | Advanced | linkado ou possível | 10 | | não |
| 27 | Contour | Advanced | presente ou possível | 9 | 3.ª amostra | não |
| 28 | Widget | Interface | `widget_skin_state` | 7 | lista de ícones | não |
| 29 | States | Interface | `ui_states_state` | 13 | nasce dobrado; sinais, curvas | não |
| 30 | Morph States | Interface | `morph_states_state` | 5 | nasce dobrado; setas | não |

Fora do corpo, e a não perder: as abas no título (`paint_tabs`), os popovers diferidos (`paint.rs:490-526`), os painéis à
esquerda (`fx_side`, `paint_side`), o arrasto do ⚙ (`fill_stroke_list::deferred`), os popovers da barra (`bars`).

## Como o painel fica coerente a meio

**Todo o painel passa a ser cartões na fatia 1**, de uma vez: uma lista, com as abas no título. Secção por converter é um
cartão com o **corpo de hoje dentro**, o cabeçalho absorvido (o mecanismo do Inspector, `cards_componentes.rs`). Nunca há
cartão ao lado de secção antiga: o que muda de fatia em fatia é só o corpo de cada cartão. Mesmos ids de secção ⇒ dobra, ordem,
tema e notas não se perdem. `populate` e `event` não mudam. Por isso as fatias são independentes e qualquer ordem serve.

## Advanced

Figma não tem Advanced e o dono pediu revisão (02/11), mas a forma nova (Path Effects em lista) não está decidida. Até lá: **um
cartão Advanced**, e os cinco membros dentro dele como subsecções com a cara de grupo (05, «nunca um 2.º título»), cada uma com a
dobra de hoje. Um arrasto continua a não tirar um membro do grupo, como F17 exigia. A pergunta da forma fica para o smoke.

## Decisão: Fill/Stroke e Appearance repetidos

**Ficam nos dois sítios.** F44/F45 foram aprovados pelo dono para o painel Vector, e são a única casa do tipo, degradê, padrão,
token, tracejado e pontas; o cartão do Inspector só tem amostra, opacidade, largura, Cap/Join/Align. Tirá-los do Vector mudaria
o aprovado e mandava o artista sair do Draw para mudar um degradê. Não há defeito de duas portas: é um comando com um id (T14), e
as duas caras nunca divergem. As duas abas não se vêem ao mesmo tempo, salvo cartão solto. **Pergunta para o smoke** registada no
tracker: «Fill e Stroke no Inspector e no Vector: quer manter nos dois?». O Transform não é repetição: pose × caixa dos pontos.

## Fatias (cada uma ≈ 35 min, por ordem de valor; independentes depois da G1)

| Fatia | Conteúdo | Corpos nativos | Risco |
|---|---|---|---|
| **G1** | Lista + abas + hospedeiro + `PH2D_CARDS=0` + Advanced + popovers e painéis laterais | Transform, Anchors, Snap, Frame, Clip | hospedeiro; catracas de abertura |
| **G2 feita 04/11** | Aparência e tinta | Appearance, Fill & Stroke (F44/F45: mesmo layout, tinta e moldura), `paint_side`, Stroke settings | feita: `cards_tinta`, `cards_traco`, `cards_lado` |
| **G3 feita 04/11** | Listas do Figma | Effects, Filters, `fx_side`, rampa; fora do plano: o painel do traço da barra ② | feita: `cards_fx`, `cards_lado::flutua` |
| G4 | Texto | Text, Font, Paragraph, Axes, Text on Path | lista de fontes |
| G5 | Tinta e pontos | Pattern ×2, Brush, Connector, Vertex | baixo |
| G6 | Advanced | Blend, Morph, Envelope, Pattern on Path, Contour | baixo; pergunta do smoke |
| G7 | Interface 1 | Component, Layout, Widget | Layout: 16 |
| G8 | Interface 2 + fim | States, Morph States; retirar o hospedeiro e a passagem de presença; censo «zero corpo antigo» | sinais |

Leis que nenhuma fatia desfaz: F43/F44/F45 (no cartão ganha a tinta e a moldura, não outro layout) · Offset Path, regras CAD e
Boolean/Convert NÃO voltam ao painel · unidade no rótulo · marcas de animação: o `.key` usa os ids de `ph2d-app-vec/src/marcas*.rs`.

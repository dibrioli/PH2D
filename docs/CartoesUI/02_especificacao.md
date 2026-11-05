# 02 — Especificação

> ⚠️ **A especificação COMPLETA e vigente é [05_ESPECIFICACAO_UI_COMPLETA.md](05_ESPECIFICACAO_UI_COMPLETA.md)** (linha fechada em 02/11); este documento é a base da maquete e o 05 prevalece onde divergirem.

A referência visual vinculativa é a [maquete](maquete/bones.png) ([D11](01_decisoes_do_dono.md):
«idêntico à maquete»). As medidas abaixo são as da maquete (CSS px a escala 1 = px lógicos do app);
na implementação viram tokens (HR-15 — zero literal de UI), preferindo os tokens `Kanban*` /
métricas do Kanban que já existem quando o valor coincide.

## 1 — Anatomia

```
┌ canvas (sem fundo de dock — D16) ───────── a lista FLUTUA sobre o canvas, margem 8, sombra de pairar
│ ┌ LISTA ──────────────────────────────────┐ raio 12, cor kanban-list, padding 0·8·8
│ │ TÍTULO DO PAINEL (CAIXA ALTA)       …   │ cabeçalho 44 px; 13 px/700, tracking 0.06em; «…» = menu do painel (D21: Reset Positions · Reset Colors · Reset All)
│ │ ┌ CARTÃO ─────────────────────────────┐ │ raio 8, cor kanban-card, padding 8·10·10, gap entre cartões 8
│ │ │ Título da secção        [ícones]    │ │ cabeçalho 26 px; 12 px/600; ícones 24×24 (glifo 16), à direita
│ │ │ Rótulo (unid.)   Rótulo (unid.)     │ │ 11 px, cor texto-terciário; margem 8 acima, 4 abaixo
│ │ │ [ic│ valor    ]  [ic│ valor     ]    │ │ campo 28 px, raio 6; célula do ícone 26; gap 6
│ │ │ [ segmentado: A │ B │ C          ]  │ │ 28 px, raio 6, padding 2, segmento ativo raio 4 + sombra leve
│ │ └─────────────────────────────────────┘ │
│ │ ┌ CARTÃO DOBRADO / VAZIO ─────────────┐ │ só o cabeçalho; título esmaecido; «+» (vazio) ou resumo (dobrado)
│ │ └─────────────────────────────────────┘ │
│ │  + Add component                        │ pé da lista (só onde há «adicionar»), 32 px, como «+ Add a card»
│ └─────────────────────────────────────────┘
```

- **Largura**: dock de referência 320 px (lista ≈ 300). O desenho tem de funcionar **estreito** (iPad,
  [D13](01_decisoes_do_dono.md)): as caixas encolhem, o número nunca colide com o ícone, e o rótulo
  pode reticenciar; nunca a unidade dentro da caixa.
- **Sem fundo de dock** ([D16](01_decisoes_do_dono.md)): o azul do quadro do Kanban NÃO entra — a
  lista flutua sobre o canvas (margem 8, raio 12, sombra 6/18 + 1/2), como os painéis do Figma UI3, e
  a sua altura **abraça o conteúdo** até à altura da janela (depois rola). Abaixo dela é canvas útil.
- **Cores** (tema forge, `docs/design/tokens.json`): `kanban-list`,
  `kanban-list-fg`, `kanban-card`, `kanban-card-fg`, `kanban-shadow-*`, e as famílias
  `kanban-<cor>-subtle/-bold` para o tema por secção. Campo = um degrau abaixo do cartão
  (maquete: `oklch(0.235 0.028 267)`; hover `0.255`) — criar token próprio (`card-field`,
  `card-field-hover`), não reaproveitar outro por semelhança. Foco = `kanban-blue-bold`.
- **Tipografia** ([D11](01_decisoes_do_dono.md)): **Noto Sans** por omissão (`NotoSansVariable.ttf`,
  já embutida em `crates/ph2d-text/fonts/`); Inter, Atkinson Hyperlegible Next e JetBrains Mono
  continuam escolhíveis. Números com algarismos tabulares (`tnum`). Tamanhos: título da lista 13/700,
  título do cartão 12/600, valor 12/400, rótulo 11/400.
- **Ícones**: família Lucide (ISC, já é a do app — `crates/ph2d-editor-core/src/icons.rs`), traço
  1.5, glifo 14 (campos/segmentos) ou 16 (cabeçalhos). Os que faltam desenham-se no mesmo estilo
  (a maquete tem os desenhos em `<symbol>` no topo do HTML).

## 2 — Regras de layout (lei)

1. **Rótulo obrigatório** em todo campo que não é perfeitamente inteligível sem nome
   ([D10](01_decisoes_do_dono.md)). Dois campos lado a lado = um rótulo por coluna. Um par X/Y
   partilha o rótulo do grupo («Position (px)») e usa as letras X/Y como ícone. Dispensa-se só quando
   o próprio texto do widget já diz (segmentado «Create · Transform · Weight»).
2. **Unidade no rótulo, nunca na caixa** ([D13](01_decisoes_do_dono.md)): «Length (px)»,
   «Rotation (°)», «Opacity (%)». Campo sem unidade = rótulo sem parênteses.
3. **O ícone dentro da caixa é a pega de arrastar** (scrub) — o rótulo não o substitui.
4. **Ações da secção = ícones no cabeçalho do cartão** (Rest Pose, Mirror, Release, Remove IK,
   Look At, adicionar/tirar camada…), com tooltip; nunca botões largos no corpo (T2).
5. **Secção ausente** (o componente/recurso não existe ainda) = cartão só-cabeçalho com título
   esmaecido e **«+»** que o cria (T3). Tirar = ícone **«–»** no cabeçalho.
6. **Secção dobrada** mostra um resumo à direita do título (ex.: «Z 3») (T4).
7. **Camadas empilháveis** (Fill, Stroke — modelo Figma): cabeçalho com «grelha» (estilos) e «+»;
   cada camada é uma linha `[amostra│código] [opacidade] 👁 –`, com rótulos «Color» e «Opacity (%)».
8. Texto da UI só em **inglês** ([memória](../../project-memory/feedback_app_ui_english_only.md)),
   por `ph2d-i18n`.

## 3 — O campo numérico (o Figma, e onde o superamos)

| gesto | comportamento |
|---|---|
| pairar no ícone | cursor ↔ (`ew-resize`); a caixa clareia (hover) |
| **arrastar no ícone** | slider: o valor muda com o deslocamento horizontal desde o início; **o cursor esconde-se e fica preso**, então o arrasto não tem fim — o número continua a crescer ([D7](01_decisoes_do_dono.md)); ao soltar, o cursor reaparece onde estava |
| Shift durante o arrasto | passo ×10 (convenção do Figma) |
| Alt durante o arrasto | passo ×0,1 (fino) |
| modificador a meio do arrasto | muda a taxa A PARTIR daquele ponto, sem salto no valor |
| Esc durante o arrasto | devolve o valor de partida |
| clicar no valor | edita como texto, com tudo selecionado |
| clicar no ícone sem arrastar (F10) | o mesmo: o que se digita a seguir nunca cai nos atalhos do app |
| duplo clique no ícone (F10) | repõe o valor de fábrica, quando há, e fica a editar com tudo selecionado |
| texto | aceita **contas** (`0.35*2`, `120+16`, `(a)`), e sufixo de unidade tolerado (`12px`, `45deg`) |
| Enter | aplica e mantém a seleção; **Tab / Shift+Tab** = próximo/anterior campo do cartão |
| Esc a editar | desfaz a edição e sai |
| ↑ / ↓ com foco | ±1 passo; Shift ±10; Alt ±0,1 |
| seleção múltipla com valores diferentes | mostra **«Mixed»**; editar escreve o mesmo valor em todos |
| um gesto = **um** passo de Desfazer | o arrasto inteiro, não cada movimento |

**Onde superamos o Figma/Godot** (a confirmar no smoke): duplo clique no ÍCONE repõe o valor de
fábrica (a Godot tem uma seta de «revert» à parte; o Figma não tem); o resumo na secção dobrada; o
cartão solto no canvas (§8).

⚠️ **Conflito com o app de hoje:** no PH2D atual o Shift deixa o arrasto **mais fino** (×0,001,
`interaction/drag.rs:147`) e o arrasto começa no CORPO do campo, não no ícone. A especificação acima
segue o Figma ([D5](01_decisoes_do_dono.md)/[D7](01_decisoes_do_dono.md)); a mudança definitiva tem
de varrer os consumidores do arrasto numérico (a regra
[«mudar a aritmética de um widget partilhado varre-se por consumidor»](../../project-memory/feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site.md)).

### Marca de animação — ponto/losango ([D17](01_decisoes_do_dono.md))

Quase todas as propriedades são animáveis (memória `project_autokey_records_everything_2026_10_19`).
Cada propriedade animável tem a marca **na linha do rótulo, acima da caixa, alinhada à borda direita
da caixa** (não da linha: com botões ao lado — Rotation, a corrente do Scale — a marca fica sobre a
caixa). Um par X/Y = **uma** marca, sobre a borda direita do par. Sempre visível (não só ao pairar).
Estados = o `KeyMark` que já existe
(`crates/ph2d-editor-core/src/widget/property_box/key_mark.rs:23`):

| estado | desenho | cor |
|---|---|---|
| `Livre` (pode animar, sem curva) | **ponto 6 px** (dono 01/11: o dobro do de hoje, depois −25 %) | texto terciário |
| `Animado` (curva, sem chave neste quadro) | losango vazio 6 px de lado (≈ 8,5 de ponta a ponta), traço 1 | `TimelineKey` |
| `Chave` (chave neste quadro) | losango cheio | `TimelineKey` |
| `Mudado` (valor fora da curva, por gravar) | losango cheio | `Warn` (âmbar) |

Clicar na marca grava/tira a chave (o comportamento de hoje). Ajustes de FERRAMENTA (Tool, Weight
Brush) não são propriedades do documento ⇒ sem marca. Maquete: Position = Animado, Rotation e Length
(A) = Chave, Length a ser arrastado (B) = Mudado.

### Arrasto infinito — mecanismo por plataforma

O cursor preso para de emitir `CursorMoved`; o deslocamento chega pelo canal CRU
(`DeviceEvent::MouseMotion`), que a shell de hoje não consome. Mapa do `winit` 0.30
(lido em `crates/ph2d-probe-cursor-grab/src/lib.rs`):

| modo | X11 | Wayland | macOS | Windows |
|---|---|---|---|---|
| `Locked` (preso + cru) | ⛔ | ✅ | ✅ | ✅ |
| `Confined` (confinado) | ✅ | ✅ | ⛔ | ✅ |

⇒ `Locked` + esconder + somar `MouseMotion` onde existe; no X11, `Confined` + **reposicionar**
(`set_cursor_position`) o cursor ao chegar à borda, somando o salto (a técnica do Blender). iPad
(toque): o dedo não precisa — o arrasto é relativo e não tem borda física relevante; arrastar no ícone
= scrub, toque no valor = teclado. A sonda `probe_can_the_cursor_be_locked` tem de correr nesta
máquina (KDE Wayland) antes de prometer — a metade 2 (o cru chega?) precisa de uma mão.

## 4 — Arrastar cartões (o Kanban)

- **Agarrar**: pelo cabeçalho do cartão (título ou zona livre; não pelos ícones). Limiar ~4 px.
- **No ar**: inclinação **4°** com pivô a 60 % da altura (`FLIGHT_TILT_DEG`, `FLIGHT_PIVOT_Y`),
  sombra de voo (longe 16/32 claro · 14/30 escuro + perto 3/8), **sem escala**. O cartão segue o
  ponteiro sem mola (posição = ponteiro − ponto de pega).
- **Vão**: abre-se onde o cartão cairia; os vizinhos deslizam com **mola criticamente amortecida**
  (`ph2d-spring::Follow`, lei `UiMotion::law(Role::Travel)`; movimento reduzido = salto imediato).
- **Auto-scroll** a 64 px da borda, até 900 px/s.
- **Largar**: o cartão assenta no vão por mola a partir de onde a mão o soltou; a inclinação volta a 0
  pela mesma mola.
- **Persistência**: a ordem das secções é a que já existe (`sections.txt`, `order=`), por painel.
- Repouso: sombra perto (1/1) + aro (0/1); hover: aro de 1 px (escuro) ou sombra 4/8 (claro).

## 5 — Tema, contorno e notas por secção (preservados, D6)

O que já existe passa a viver no CARTÃO: o tema da secção pinta o cartão INTEIRO — fundo (`bg-1` do
tema), caixas um degrau abaixo, hover, segmento aceso e lista suspensa um degrau acima, rótulos,
acento e pontos no tema dele ([T25](01_decisoes_do_dono.md), `widget::card_ink`); um cartão sem tema, ou com o
mesmo da lista, fica no Kanban da lista; o contorno decorativo envolve o cartão **e** as suas notas a 4 px; as notas
são post-its dentro do cartão, abaixo dos campos. Entrada: o menu do botão direito no cabeçalho do
cartão (o mesmo de hoje na pega) e o «…» do painel. Nada disto é gravado de novo — reaproveita os
mesmos dados (ver [03](03_mapa_do_codigo.md)).

## 6 — Painel Bones (piloto 1)

Cartões, por ordem (os que dependem de estado só aparecem quando valem):

| cartão | conteúdo |
|---|---|
| **Tool** | segmentado com ícone + texto: Create · Transform · Weight |
| **Weight Brush** (Weight ativo) | Mode: Cumulative · Absolute + Direction: ⊕ · ⊖ (escondido em Absolute) · Radius (px) · Strength (%) (ou Target Weight em Absolute) |
| **Skin** | Bind to Skeleton (botão de destaque, só modo Object, quando nada preso); preso: ícones Keep Pose · Release no cabeçalho; «Deform by»: Artwork · Bone reach |
| **Bone** (osso em foco) | ícones Rest Pose · Set Rest Pose · Mirror Branch no cabeçalho; Length (px) · Segments; Strength (só com envelope); «Curve handles»: Manual · From chain; Curve in · Curve out (e Along in/out quando aplicável); Curve tip (lista) |
| **Angle Limit** | vazio = «+»; com limite: 👁 · – no cabeçalho; Min (°) · Max (°) |
| **Smart Bone** | vazio = «+»; com: Action · Object · From/To |
| **IK** | vazio = «+» e ícone Look At; com: Look At · – no cabeçalho; Mix (%) · Chain length; Softness · Bend (Auto · ↺ · ↻ · Mixed); Aim offset (°) com aim |

## 7 — Inspector principal (piloto 2)

| cartão | conteúdo |
|---|---|
| **Objeto** (topo, sem título) | ícone do tipo · nome editável · 👁 visível · 🔒 travado |
| **Transform** | reset no cabeçalho; Position (px): X · Y; Rotation (°): valor + girar 90° · espelhar H · espelhar V; Scale: X · Y + corrente (proporção); Skew (°): X · Y |
| **Appearance** | 👁 no cabeçalho; Opacity (%) · Blend (lista) |
| **Fill** | camadas (§2.7) |
| **Stroke** | camadas; Position (lista: Center/Inside/Outside) · Weight (px); Cap · Join com desenhos |
| **Ordering** (dobrado) | resumo «Z n» |
| **Material & Blend** (dobrado) | — |
| pé | **+ Add component** |

⚠️ Fill/Stroke **não** estão no Inspector de hoje (vivem no painel Vector, aba Drawing); entraram na
maquete para avaliar o layout ([D12](01_decisoes_do_dono.md)). Se ficarem, o painel Vector deixa de
os repetir — decisão para a mudança definitiva.

## 8 — Cartão solto no canvas (D15)

Ver [maquete D](maquete/canvas.png).

- **Descolar**: a arrastar um cartão, quando o ponteiro sai da lista (borda do dock) o cartão
  **descola** — mantém a inclinação de voo e passa a pairar sobre o canvas; o vão na lista **fecha-se
  logo** pela mola — o painel não reserva espaço para o cartão que saiu, fica só a linha-fantasma
  (34 px). Dono 01/11: uma área vazia a «roubar espaço» confunde.
- **Largar no canvas**: o cartão fica flutuando ali, sem inclinação, com sombra de pairar
  (8/20 + 1/3). Posição em coordenadas de **ecrã** (não de mundo: não anda com o zoom/pan), presa ao
  rectângulo visível do canvas.
- **No painel fica uma linha-fantasma** tracejada «<Título> · on canvas»: clicar faz o cartão
  piscar e o traz à vista; arrastar o cartão de volta para QUALQUER lista o reencaixa com o vão do
  Kanban. Fechar o cartão no canvas (ou «Return to panel» no menu) devolve-o ao sítio de onde saiu.
- **O que ele mostra**: por omissão **segue a seleção**, como o painel de onde saiu (o subtítulo no
  cabeçalho diz de quem é, ex.: «Bone 3»). O **alfinete** no cabeçalho prende-o ao objeto atual —
  então mostra sempre esse objeto, mesmo com outra seleção (útil para afinar o IK de um osso enquanto
  se seleciona outro). Se a secção não se aplica à seleção atual, o cartão mostra-se esmaecido com
  «Not available for <nome>».
- **Igual em tudo**: tema, contorno e notas viajam com o cartão; campos, scrub e Auto Key funcionam
  da mesma forma; vários cartões podem estar soltos ao mesmo tempo, e reordenam-se por profundidade
  (o último tocado fica por cima).
- **Persistência**: o layout (que cartões estão soltos, onde, se presos e a quem — pelo `StableId`,
  nunca bits) grava-se por **área de trabalho** junto do layout dos painéis
  (`~/.ph2d/layout.txt`); o alfinete a um objeto que já não existe solta-se sozinho.
- **Clique**: o cartão no canvas é chrome — recebe o clique antes do canvas na sua área (a regra
  «o 1.º clique é do que se vê por cima» (memória `project_hierarchy_order_rules_drawing_and_click_2026_11_01`)).
- iPad: pressão longa no cabeçalho levanta o cartão (como o Trello no toque).

⚠️ Seguir a seleção vs. prender por omissão (T7) é a primeira coisa a confirmar no smoke.

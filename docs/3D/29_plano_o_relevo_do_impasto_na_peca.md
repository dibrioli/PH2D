# 29 — Plano: o relevo do IMPASTO na peça (etapa 3b do Painter na peça)

> **Ordem:** o dono, 30/09 (*«siga com o que está em aberto»*), sobre a decisão
> de produto dele de 24/09: **relevo de LUZ** — a tinta ganha espessura que pega
> luz e sombra, **a forma da peça não muda**, desfaz-se com `Ctrl+Z`, **não entra
> na silhueta nem no ficheiro exportado**, e *«a luz usada é a luz do cenário 3d»*.
> Contexto: [25](25_avaliacao_o_painter_na_malha.md) · etapa 2 no
> [handoff](handoffs/HANDOFF_line_sculpt3d_O_PAINTER_NA_PECA_ETAPA_2_2026-09-24.md).

## 1. O que existe hoje (medido pelas três leituras de 30/09)

- **No Painter** a altura do impasto é um `f32` por píxel e POR CAMADA
  (`PainterTool::heights`, `+` levanta, `−` cava; `h = 1` é um traço de
  profundidade cheia, e a luz converte-a a píxeis com `DEPTH_UNIT_PX = 16`). O
  traço escreve-a num plano próprio (`relief.stroke_height`) e o `close_stroke`
  soma-a na camada. ⚠️ **Na tela da vista ela é apagada a cada traço** — o
  `set_source`, por onde passam o `clear_screen_canvas` e o `seed_screen_canvas`,
  limpa `heights`/`covers`/`mats`.
- ⛔ **A drenagem sai ACESA**: o `take_screen_canvas` devolve o composto já com
  o `apply_impasto_light` (a luz 2D do Painter) por cima, e não há acessor da cor
  SEM luz. Pousar isso na peça pintaria a sombra 2D como COR, e a luz 3D por
  cima dela contaria a luz duas vezes.
- **Na peça** não há canal de relevo: o shader não perturba a normal em sítio
  nenhum (nem `dpdx`/`dpdy`). A normal de sombreado é `canvas_normal(n_view)`, lida
  UMA vez no `fs_core` e usada pelos modos `Rig`, `Pbr` e `Matcap`; o `Flat` não a
  usa (e não deve: ele é a cor sem luz).

## 2. As decisões

### D1 — A altura mora no PLANO DE TINTA FINA, ao lado da cor

`Tinta` ganha `alturas: Option<Vec<f32>>`, uma por amostra, endereçada e
interpolada pelos MESMOS pesos da cor (`leitura_tri`/`leitura_quad` + `indice`).
⭐ Com isso o empréstimo ao traço, a ranhura por degrau, a cerca de identidade do
plano e a exportação que o ignora vêm **de graça** — tudo o que já transporta o
plano transporta a altura.

- ⚠️ **`Option` de propósito:** a `256x` um plano tem dezenas de MB, e uma peça
  que nunca levou impasto não paga um byte.
- ⛔ **Sem tinta fina armada (`Paint Detail = Mesh`) não há relevo**, e isso é
  DITO no pen-down (a mesma família de vozes da tinta fina). A alternativa — uma
  altura POR VÉRTICE — daria a derivada constante por triângulo (relevo
  facetado à resolução da malha) e custaria um 10.º buffer de vértice, o
  `MeshData` partilhado e o formato do `ph2d-mesh`. *O relevo de uma pincelada é
  mais fino que a malha, que é exactamente o que a tinta fina existe para ser.*

### D2 — A unidade é o MUNDO: a espessura que a tinta tinha no ECRÃ

A altura na peça é medida em unidades de objecto, ao longo da normal:
`altura = h · DEPTH_UNIT_PX · (mundo por píxel no ponto, na vista do traço)`.
⇒ o relevo é **geométrico**: rodar a vista e aproximar não o mudam, e um traço dado
de perto fica tão espesso, em mundo, quanto a tinta parecia no ecrã. O factor sai
da `Vista` da sessão (a mesma que projecta a amostra).

### D3 — A lei na peça: a altura SOMA-SE à de antes do traço

A tela da vista começa cada traço com altura zero (o `set_source` limpa-a), logo o
que ela entrega é o relevo DESTE traço: `nova = antes + altura(p)`. Pintar por cima
empilha tinta; cavar (`−`) desce. A janela de desfazer guarda a altura de antes,
num canal ao lado da cor.

### D4 — A cor que se pousa é a SEM luz

Porta nova no Painter: a drenagem da tela da vista entrega a cor **sem** o
`apply_impasto_light` e, ao lado, a altura da região mudada (camada activa, com o
traço vivo). A luz passa a ser só a da cena 3D.

### D5 — O sombreado: *bump mapping* sem parametrização (Mikkelsen, 2010)

Referência publicada e portada: M. Mikkelsen, *«Bump Mapping Unparametrized
Surfaces on the GPU»* (2010) — a normal perturbada sai das derivadas de ecrã da
posição e da altura, **sem tangentes e sem UV**, que é o que uma malha esculpida
não tem:

```
σs = ∂p/∂x   σt = ∂p/∂y        (posição em vista, derivadas de ecrã)
R1 = σt × n  R2 = n × σs       det = σs · R1
∇  = sign(det) · (∂H/∂x · R1 + ∂H/∂y · R2)
n' = normalize(|det| · n − ∇)
```

Ela substitui o `nc` no `fs_core` (para `Rig`, `Pbr`, `Matcap`), e as derivadas
chamam-se **fora** de ramos divergentes (o `tinta_no_ponto` ramifica por
`topo`). Com a altura toda a zero, `∇ = 0` e `n' = n` — ⚠️ mas **não ao bit** (o
`normalize(|det|·n)` arredonda), logo o caminho SEM relevo armado **não passa por
ela**: ele é byte-idêntico ao de hoje por construção, com gate.

- ⛔ **Fora:** o G-buffer (a normal doada ao 2D e ao *bake*) fica SEM relevo nesta
  onda — a doação é GEOMETRIA, e *«não entra na silhueta»*; se o dono quiser o
  relevo no sprite assado, é pergunta dele e onda própria.

## 3. As ondas

| Onda | O quê | Gate que a fecha |
|---|---|---|
| **W1** | `Tinta::alturas` + leitura `altura_tri/quad` + `footprint` + a janela de desfazer (`TintaDoTraco` com a altura de antes, `JanelaFina` com o canal) + o documento `v4` (corridas de `f32` por bits, com migração do `v3`) | ida-e-volta do documento ao bit; `Ctrl+Z` devolve a altura; uma peça sem relevo grava o MESMO número de bytes de hoje |
| **W2** | o device: `@binding(7)` com as alturas, subida inteira e incremental, o `armado` a dizer se há relevo, e o *bump* no `fs_core` | paridade da leitura (compute) contra `altura_tri/quad`; sem relevo o quadro é byte-idêntico; com um degrau de altura numa chapa a luz muda **só** na borda dele e no sentido que a luz pede |
| **W3** | o Painter: a drenagem sem luz + a altura da região; a pousada escreve `antes + h·16·mundo_por_px` | censo de fiação; um traço de impasto na peça sobe a altura das amostras que cobriu, no valor da conta D2; a cor pousada é a sem luz |
| **W4** | a cena e o roteiro (`=52` ganha o passo, ou cena própria), a voz do pen-down sem tinta fina, e o smoke | a foto: a pincelada lê-se em relevo com a luz da cena, e rodar a luz move as sombras dela |

## 4. Critério de desistência (antes de construir)

- Se a derivada de ecrã da altura, com o plano a `8x`, der **facetas visíveis por
  sub-célula** que nenhuma interpolação dentro do `tinta_no_ponto` cure, a forma
  D5 pára e a pergunta passa a ser a normal lida por diferenças CENTRAIS no
  plano (o que o Painter faz em 2D) — com o custo medido antes.
- Se a subida incremental das alturas custar mais de `1 ms` por quadro durante um
  traço na peça de fábrica, a W2 não fecha nessa forma.

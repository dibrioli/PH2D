# W5 + W6 — a peça vira uma CENA de objetos, e ganha o gizmo (2026-08-20)

> **O que este doc é:** o mecanismo das duas waves e os números que decidiram o desenho.
> O estado do módulo vive no [README](README.md); a história, no handoff.
>
> **§1–§4 são a W5** (a cena de objetos + o gizmo de MOVER) · **§5 é a W6** (rodar, escalar, e o
> undo que estava partido).

Enio, no smoke de 19/08:

> *"na hierarchy apenas um objeto e não 3 cilindro. Não há gzimo 3d para mover os objetos.
> Precisamos de uma como o do blender."*

⚠️ **As duas frases são UM defeito.** Um objeto que a cena não enumera **não tem pose que um gizmo
agarre** — o gizmo não era a segunda tarefa, era a consequência da primeira. Foi por isso que a
ordem foi esta e não a inversa.

---

## §1 — A hierarquia da cena É a árvore de modelagem

Cada primitiva e cada operação passou a ser uma **entidade**: nome, forma, pose, salva e desfeita.
O `FieldDoc` que o traçador avalia é **cozido** do mundo a cada quadro.

Isto é lei da casa dita duas vezes — [ADR-0110](../architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md)
(*todo path é entidade ECS, uma hierarquia*) e ADR-0121/0132 (*fonte ≠ cozido*).

### As três medições que decidiram o desenho

| Pergunta | Onde está a resposta | Medido |
|---|---|---|
| O `Transform` da casa serve para uma pose 3D? | [`transform.rs`](../../crates/ph2d-ecs/src/transform.rs) | **Não** — `translation: Vec2`, `rotation: f32` (ângulo **escalar**), `scale: Vec2`. Não há onde pôr uma rotação 3D |
| A Hierarquia enumera um filho **sem** `Transform`? | `build_hierarchy_snapshot` | **Sim** — só a RAIZ é filtrada por `With<Transform>`; o DFS desce por `Children` |
| O snapshot (save + undo) captura esse filho? | `world_to_snapshot`, fase 1 | **Sim** — desce `Children` **sem filtro nenhum** |

⇒ a pose 3D é o componente `FieldPose`, e os nós **não** carregam `Transform`. ⛔ Escrever meia
pose no `Transform` e a outra metade noutro sítio seria a segunda verdade na forma mais cara: o
Inspector mostraria uma posição que a peça não tem.

⚠️ **O gate do snapshot mudou de tamanho, e é de propósito.** Antes a peça era um blob num
componente e o gate media a viagem de bytes. Agora o que pode partir-se é a **hierarquia** — um
filho que volta sem pai, irmãos fora de ordem (a subtração é `children[0]` menos os seguintes) —,
então o que se compara é a **peça cozida** dos dois lados.

### Uma classe de erro trocada por nada

Uma operação com **zero filhos não é emitida**; se isso esvaziar o pai, ele também não; se esvaziar
a raiz, não há peça. ⭐ Apagar o último filho de um grupo na Hierarquia é um gesto **normal**, e o
resultado normal dele é não haver mais nada ali — devolver `EmptyCombine` transformaria o gesto num
erro que o artista teria de desfazer para entender.

### Duas duplicações de aritmética que morreram — e as duas eram armadilhas de gizmo

| O quê | Estava em | Passou a |
|---|---|---|
| quaternion (`mul`, `rotate`, `normalize`) | uma cópia privada na câmera do traçador | [`ph2d_field::xform`](../../crates/ph2d-field/src/xform.rs), junto do tipo cuja rotação ela compõe |
| pixel ↔ plano da câmera | o `Plane` privado do traçador | `ph2d_field_render::Screen`, **público**, e é o mesmo que `Orbit::project` usa |

⚠️ A segunda é a que importa para este doc: se a projeção do gizmo e a construção dos raios da
marcha fossem duas contas, elas divergiriam meio pixel e o sintoma seria **uma seta que agarra ao
lado da superfície que ela diz mover** — o tipo de defeito que ninguém chama de bug de projeção.
O gate `a_point_projects_where_the_march_actually_hits_it` **traça de verdade** e compara o
centroide da silhueta (sob projeção ortográfica o centroide da silhueta de uma esfera é a projeção
exata do centro dela — o alvo não é aproximado, é o número).

---

## §2 — O gizmo

Sete alças, na **ordem de apontar** (de dentro para fora): o disco de vista · os três quadrados de
plano · as três setas. A ordem é load-bearing: `pick` devolve a primeira que casa, e sem ela apontar
o centro escolheria um eixo à sorte.

### ⭐ Três números DERIVADOS, e nenhum escolhido

| Número | Derivação |
|---|---|
| `MIN_ARM_PX = INNER_PX + 2·GRAB_PX` | Abaixo disso a haste projetada **não tem um único pixel que seja só dela** — a folga central come um lado e o raio de agarre o outro. Aí ela não é um controle, é uma lotaria entre três. É o que o Blender faz, e o efeito colateral é bom: com a seta escondida sobra o quadrado do plano perpendicular, que é exatamente o gesto que aquele enquadramento pede |
| alça de plano viva | O **lado mais estreito** do losango projetado ≥ `GRAB_PX`. A pergunta certa não é a área: é se ainda se pode **apontar** |
| realce de hover | O **próprio token**, com a luminosidade levantada em OKLCH por uma fração do que falta para o branco. Uma soma fixa estouraria para branco nos temas claros, que é onde o realce é mais difícil de ver |

⚠️ **Uma seta apontada ao observador é escondida E não arrasta.** As duas metades: a conta do
arrasto divide pelo comprimento projetado, e sem o corte um pixel de rato valeria um salto
arbitrário — a peça sairia da janela num toque.

### O arrasto

- **Eixo**: projeção escalar do movimento do rato sobre a direção projetada do braço, em frações do
  braço. ⭐ O gate `one_arm_of_mouse_is_one_arm_of_world` afirma o **número**, não a direção: um
  fator errado passa despercebido num gate de direção e sente-se como *"a peça foge da mão"*.
- **Plano** e **vista**: a diferença entre dois encontros do raio do cursor com o plano. O disco de
  vista **nunca degenera** (o plano dele é o da tela) — é a rede de segurança de todo enquadramento.
- **Escrita**: o arrasto **não** escreve na peça; ele acumula um deslocamento de **mundo** que a
  ponte com a cena aplica no início do quadro seguinte, pelo mesmo caminho dos intents do painel.
  *O mundo tem um só escritor.*

⚠️ Os pedidos **acumulam** entre quadros. Guardar só o último faria a peça andar menos do que a mão
— devagar, e **só quando o rato vai depressa**, que é o defeito mais difícil de acreditar quando
alguém o reporta.

### `translate_world`: mundo → local

O gizmo desenha e agarra em **mundo**; o nó guarda a pose **local** ao pai. Somar o deslocamento de
mundo direto na translação local funcionaria **exatamente** enquanto nenhum pai tivesse rotação ou
escala — o que é verdade na primeira cena de smoke e em **nenhuma peça real**. O sintoma seria a
seta X mover a peça na diagonal, e ele só apareceria depois de alguém rodar um grupo. Gate com pai a
um quarto de volta e escala 2.

### Os eixos são os do MUNDO

Como o default do Blender ("Global"). ⏸️ A orientação **local** (mover ao longo do próprio eixo de
um nó rodado — os cilindros da cena 1 estão rodados) é item **ABERTO**, e não uma omissão: escolher
a orientação é decisão de produto e o Blender expõe-na num seletor. Entregar só uma seria escolher
por quem não pediu.

---

## §3 — Costura, que é onde a semana se perde

A `DIRETIVA_IMPLEMENTACAO` §1 chama-lhe a causa nº 1: *a alça está pintada, o arrasto está correto,
e ninguém liga os dois* — e as duas metades passam em todo teste de unidade.

A decisão do ponteiro saiu dos métodos de `App` para `begin`/`advance` sobre o `Smoke`, e **6 gates
percorrem o caminho real**:

| Gate | O que ele impede |
|---|---|
| `pressing_on_an_arrow_grabs_it_instead_of_orbiting` | a alça pintada e morta |
| `pressing_away_from_the_gizmo_still_orbits` | o gizmo sequestrar a navegação da janela inteira |
| `the_right_button_orbits_even_over_a_handle` | ter de tirar o rato de cima da peça para girar a vista |
| `pointer_events_between_two_frames_add_up` | a peça andar menos do que a mão com o rato rápido |
| `hover_lights_the_handle_without_swallowing_the_event` | não saber o que se vai agarrar · e a janela 3D comer o hover do app 2D |
| `the_grabbed_handle_stays_lit_while_the_cursor_walks_away` | o realce apagar-se no instante em que o gesto começa a valer |

**Provas de mutação** (as duas restauradas):

| Mutação | Reprovou |
|---|---|
| tirar o fator do braço na conta do arrasto | `one_arm_of_mouse_is_one_arm_of_world` |
| deixar toda seta viva | `an_axis_that_points_at_the_camera_is_not_a_handle` |
| inverter o sinal do `y` em `Screen::pixel_of` | `a_pixel_survives_the_round_trip` **e** `a_point_projects_where_the_march_actually_hits_it` |

---

## §4 — Onde a seleção mora

⭐ É a do **app** (`hero.gizmo.selection`): clicar numa linha da Hierarquia faz as setas aparecerem.
⛔ Uma seleção própria deste módulo seria uma segunda ideia de *"o que está selecionado"* dentro do
mesmo aplicativo, e as duas divergiriam no primeiro clique.

E a peça **nasce com o primeiro filho selecionado**, uma vez e só nessa — *feature nova = auto-play*,
sem tirar da mão do artista o direito de escolher outro objeto.

⏸️ **Selecionar clicando na peça em 3D** é item aberto. Ele é alcançável e barato de descrever
(marchar o pixel do cursor, depois perguntar a cada folha o valor do campo dela naquele ponto e
escolher o menor módulo), mas o preço real está em **compilar uma árvore por folha**, e isso pede
medição antes de código.

---

## §5 — W6: os outros dois verbos, e um undo que estava partido (20/08)

### ⚠️ O undo de um arrasto estava partido, e a nota da W5 estava ERRADA

A §5 anterior dizia que *"um arrasto longo vira N passos de undo"* e mandava agrupar por gesto.
**Medir primeiro mostrou as duas metades da verdade**, e nenhuma era a da nota:

| Facto | Onde |
|---|---|
| O `post_frame_undo` **já** tem a lei: *"um gesto em andamento espera o fim"* | [`undo.rs`](../../shells/desktop/src/undo.rs) |
| Ela lê o `held_button`, que este módulo **nunca chega a pôr** | o gancho consome o `Down` e volta na linha 3183; o `held_button` é escrito na 3202 |
| Logo, nem a supressão nem a **marca de entrada** alcançavam o gesto | o passo só se registava colado à próxima ação do artista, seja ela qual for |

⭐ *A lei estava certa e não alcançava este gesto.* A cura não foi um sistema de agrupamento — foi
`gesture_in_progress()` (só o arrasto do **gizmo** conta: orbitar não autora nada) consultada pelo
`post_frame_undo`, mais `any_input_this_frame` marcado no *move* **e** no *up*. O segundo é
load-bearing: um arrasto cujo último movimento caiu num quadro anterior ficaria sem quadro nenhum a
marcar entrada.

⚠️ Um dos dois gates **lê a fonte** do `undo.rs` e diz por extenso o que prova: que o cano está
ligado. Ele não prova que a supressão funciona (isso é a lei do shell, que já tem os gates dela) —
impede a **fiação órfã**, que foi o modo de falha exato.

### Rodar

3 argolas de eixo + a **argola de vista**. ⭐ **O ângulo é medido no PLANO DE ROTAÇÃO**, levando o
cursor lá pelo raio — e não em pixels em torno do centro projetado.

⚠️ O atalho comum (ângulo na tela) **mente fora do eixo da vista**: a projeção de um círculo é uma
elipse, e o ângulo na elipse não é o ângulo no círculo. O gesto ficaria rápido de um lado e lento do
outro, e uma volta inteira **não fecharia**. O gate soma 36 passos **em torno da elipse projetada** e
exige 2π a 10⁻³.

`rotate_world` faz a conjugação `inv(R_pai) ⊗ Q ⊗ R_pai`. ⚠️ Sem o sanduíche, um giro em torno do X
do **mundo** aplicado a um filho de pai rodado giraria em torno do X **do pai** — o eixo errado, e
ninguém diria que o culpado é o gizmo.

### Escalar — ⛔ UMA alça, e é uma decisão medida

[ADR-0161 §6]: escala não-uniforme **destrói a propriedade de distância**. Três caixas por eixo
(as do Blender) seriam três controles a **prometer o que o modelo não entrega** — arrastar a de X
escalaria os três, e o artista concluiria que o app tem um bug que ele não tem.

A alça é **um punho de tamanho**, não um eixo: por isso não leva cor de eixo, e por isso o rótulo diz
**"Size"** e não "Scale". A direção do punho (canto superior direito) é **cosmética** — a lei do
arrasto depende só do raio ao centro.

Lei: **razão de raios**. ⭐ Duas metades de um arrasto **multiplicam** — somar diferenças daria ×2,2
onde tem de dar ×1,21, e o defeito só apareceria com o rato depressa.

`Motion` substituiu o `[f32; 3]` do pedido: os três verbos compõem de formas diferentes (somar ·
compor · multiplicar), e um vetor para os três obrigaria quem recebe a adivinhar qual é qual pelo
modo em que o gizmo estava.

### ⭐ Um defeito que o gate apanhou antes do smoke

A argola de **vista** fica, por construção, exatamente no plano da câmera: a profundidade de todo
ponto dela é zero — e em `f32` esse zero sai como ±10⁻⁷ aleatório. Com o teste `>= 0` ela saía com
**3 pontos de 48**: uma fieira de pedaços soltos onde devia estar um anel.

Duas curas na mesma linha, e as duas nomeiam o que curam:

1. a profundidade passa a sair do **deslocamento**, nunca de `ponto − origem` — a subtração cancela
   dois números grandes e o erro que sobra escala com a distância da peça ao zero;
2. `RING_FRONT_EPS = 10⁻⁵` nomeia o recurso (**precisão da representação**), duas ordens acima do
   ruído e cinco abaixo de qualquer fronteira real de meia-argola. É o irmão do `PRECISION_FLOOR`.

### As duas portas do verbo

O **seletor no painel** (grupo segmentado adaptativo — num painel estreito ele reflui em vez de
quebrar o texto dentro do botão) e as teclas **`G` / `R` / `S`**, as do Blender. ⚠️ As teclas são
guardadas por *"ponteiro sobre a janela 3D"*, que é a diferença entre um atalho e três letras comuns
sequestradas de todo campo de texto do app — a mesma nota que o `Home` já tinha, e que este módulo
já viu envelhecer uma vez.

⚠️ A lista de verbos que o painel mostra é **derivada de `Mode::ALL`**: o painel não conhece o enum,
e o intent devolve a **posição**. Acrescentar um quarto verbo é uma linha no shell.

### Provas de mutação (as duas restauradas)

| Mutação | Reprovou |
|---|---|
| ângulo de rotação pela metade | `a_full_turn_of_the_cursor_is_a_full_turn_of_the_part` |
| conjugação do pai removida | `a_world_axis_spin_stays_on_the_world_axis_under_a_rotated_parent` |

---

## §6 — Aberto

- ⏸️ orientação **local** dos eixos (produto — o Blender expõe um seletor)
- ✅ **rotacionar e escalar FECHARAM** na W6 (§5)
- ⏸️ clicar na peça em 3D para selecionar (§4)
- ⏸️ **snap** e leitura numérica do deslocamento durante o arrasto
- ✅ **o undo de um arrasto FECHOU** na W6 (§5) — e a nota que estava aqui estava **errada**: a lei
  do shell já existia e o que faltava era o módulo dizer-se. *Meça o mecanismo antes de construir o
  que a nota prescreve.*
- ⏸️ **rotação e tamanho não têm leitura numérica nem snap** durante o arrasto (o mover também não)
- ⏸️ o **pivô** é sempre o centro do nó. Um pivô escolhido (centro da seleção, cursor 3D) é produto,
  e entra com a UI que o escolhe
- ⏸️ perspectiva (herdado da W2) — quando entrar, entra **num sítio**: `Orbit::project`

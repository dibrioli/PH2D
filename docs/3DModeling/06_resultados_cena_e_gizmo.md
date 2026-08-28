# W5 → W13 — a peça vira uma CENA de objetos, e o módulo vira um modelador (2026-08-20)

> **O que este doc é:** o mecanismo destas nove waves e os números que decidiram cada desenho.
> O estado do módulo vive no [README](README.md); a história, no handoff.
>
> | § | Wave | O quê |
> |---|---|---|
> | 1–4 | **W5** | a cena de objetos, e o gizmo de **mover** |
> | 5 | **W6** | **rodar** e **escalar** — e o undo que estava partido |
> | 6 | **W7** | o **clique** que escolhe o objeto, e os eixos Global/Local |
> | 7 | **W8** | o gesto **preso à grelha**, e o número dele |
> | 8 | **W9** | **criar** formas e **combiná-las** |
> | 9 | **W10** | as **dimensões** de cada forma |
> | 10 | **W11** | **duplicar** e **apagar** |
> | 11 | **W12** | as mesmas duas ações, pela **Hierarquia** |
> | 12 | **W13** | digitar a **posição**, e dois gates que não provavam |

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

## §6 — W7: o clique escolhe o objeto, e os eixos podem ser dele (20/08)

### ⭐ Como se pergunta a um CAMPO quem ele é

Uma malha traz consigo a resposta — cada triângulo sabe de que objeto é. Um campo implícito **não**:
o que se avalia é um número, e o número da peça já é a união de todos os nós.

A pergunta é feita em dois passos, e **nenhum é estrutura nova**:

1. **Onde** a superfície está sob o cursor — uma marcha de **um** raio, pela mesma função que
   desenha o quadro. A marcha passou a devolver também o ponto de mundo (ela já sabia o `t`), o que
   impede uma segunda marcha de existir só para isto.
2. **De quem** é aquele ponto — pergunta-se a cada folha o valor do campo **dela** ali, e ganha a de
   menor módulo. Numa superfície de união o vencedor vale ~0 e os outros valem a distância a que
   estão: a resposta não é apertada, é a diferença entre tocar e não tocar.

### ⛔ A alternativa recusada, com o número

| Rota | O que custa | Onde |
|---|---|---|
| **Escolhida:** uma árvore por folha, avaliada num ponto | **0,10 ms** por clique (3 folhas, quadro 1600×1000) | `measure_pick_cost` |
| **Recusada:** *id-buffer* (a marcha devolve o id do nó) | um segundo canal por **todo** operador da árvore — `min` de dois números passaria a `min` de dois pares | cada pixel de **cada quadro** |

⭐ O *id-buffer* é o que um renderizador de malha faz, e aqui ele espalharia o custo por toda a
imagem para responder a uma pergunta que só se faz **num clique**. 0,10 ms fecha a discussão.

### As regras do gesto

- **Clique no fundo limpa a seleção**, como em todo modelador. Manter deixaria o gizmo aceso em cima
  de nada.
- ⚠️ **Soltar uma ALÇA nunca é seleção.** Sem isto, mover um objeto trocaria a seleção para o que
  estivesse por baixo dele no fim do gesto — e o artista perderia o que acabou de posicionar.
- O limiar clique-vs-arrasto é o número da **casa** (`NUMBER_INPUT_DRAG_THRESHOLD_PX`). Ele tem o
  nome do campo numérico porque foi lá que a casa o mediu primeiro, mas a grandeza é a mesma
  pergunta física — *quanto a mão treme entre carregar e soltar* — e já havia três respostas no
  shell. Uma quarta seria a quarta a envelhecer.

### Eixos Global / Local

⭐ **A `Anchor` passou a carregar os três eixos já resolvidos.** Assim a lei do gizmo deixa de saber
que existe uma escolha de referencial, e quem a faz é a ponte, que é quem tem a pose do nó.

⚠️ Sem isso, cada função da lei teria de perguntar *"global ou local?"* — o mesmo `if` repetido em
cinco sítios, que é exatamente como um deles fica para trás. O gate
`the_law_moves_along_the_anchors_axes_not_the_worlds` impede o seletor de ser decorativo.

O segundo seletor tem **família de ids própria**: partilhá-la com o dos verbos faria um clique em
«Local» disparar o verbo da mesma posição, e o sintoma seria *"trocar de eixos troca a ferramenta"*.

### ⚠️ Um comentário velho que mentia

O doc do `ph2d-field-render` dizia que o G-buffer devolve *"máscara, normal e **profundidade**"*.
Ele **nunca a teve**. A seleção por clique foi a primeira coisa a precisar dela e a descobrir que
não estava lá. *Comentário velho e código morto mentem* — corrigido no mesmo commit.

### Provas de mutação (as quatro restauradas)

| Mutação | Reprovou |
|---|---|
| o pick avalia a folha na pose **local** | `the_answer_uses_the_world_pose_not_the_local_one` |
| `Frame::Local` devolve os eixos do mundo | `local_axes_follow_the_node_and_global_ones_do_not` |
| a lei lê `WORLD_AXES` em vez da âncora | `the_law_moves_along_the_anchors_axes_not_the_worlds` |
| o limiar de clique deixa de morder | `dragging_the_camera_is_not_a_click` |

---

## §7 — W8: o gesto preso à grelha, e o número dele (20/08)

### ⭐ O arrasto passou a medir o TOTAL desde a pegada

Era incremental. **Prender incrementos a uma grelha e somá-los acumula o erro de cada
arredondamento**: o gesto sai da grelha depois de uns segundos, com a ficha a mostrar um número
redondo que a peça não tem.

Agora a pegada **congela** a âncora e o pixel, e cada movimento mede o total contra eles.

⚠️ **A âncora TEM de ser congelada.** Ela é republicada a cada quadro a partir da pose do objeto —
que o próprio arrasto está a mudar. Medir contra uma âncora que se move seria medir contra o
resultado, e o gesto perseguiria a própria cauda.

⭐ `Motion::since` é a **inversa exacta** de `merge` — `total.since(a).merge(a) == total`, com gate.
Se as duas não forem inversas, cada evento de ponteiro deixa um resíduo, e o resíduo cresce com a
duração do gesto.

### Três passos, três razões — e nenhuma é «pareceu bem»

| Verbo | Passo | Por quê |
|---|---|---|
| **Translação** | **derivado do enquadramento** (escada 1-2-5) | Dois degraus vizinhos têm de estar mais afastados do que a tolerância do próprio ponteiro (`GRAB_PX`) — abaixo disso escolher entre eles é sorteio. Sobe-se a escada até o primeiro degrau que passa. ⚠️ Um passo fixo em unidades de mundo é inútil nos dois extremos: aproximado, dois pontos da grelha ficam a meia tela; afastado, ficam dentro do mesmo pixel. A grelha do Blender subdivide com o zoom pela mesma razão |
| **Ângulo** | **15°** | É o **maior** passo que ainda contém 30, 45, 60 e 90 — os ângulos que um artista diz em voz alta. Mais fino não os perde, mas obriga a mira; mais grosso perde o 45 |
| **Tamanho** | **0,1** | É o que a **leitura** consegue exprimir. A ficha mostra duas casas; um passo mais fino mostraria «x1,50» para dois tamanhos diferentes |

O `Ctrl` é lido **a cada movimento**, e não congelado na pegada: mira-se à mão até perto e
prende-se no fim, como no Blender. Congelá-lo obrigaria a soltar e repetir o gesto.

### ⚠️ A ficha, e a lei que eu ia violar

O `gizmo/readout.rs` da casa já tinha escrito a lei:

> *O readout é DERIVADO do resultado APLICADO, nunca re-calculado do cursor.* Uma segunda derivação
> a partir do cursor discordaria do encaixe e mostraria `12,03` enquanto a forma pousou em `12,00`.

Aqui a ficha sai do `Grip::applied` — o que é honesto **porque** o mundo recebe exatamente esse
valor. ⭐ Mas *"exatamente"* é o tipo de afirmação que apodrece num comentário, então ela é um
**gate**: `the_readout_is_the_pose_the_world_took` aplica os três verbos com totais **presos** e
compara a pose resultante com o que a ficha afirma. Se um dia a escrita recusar, limitar ou
arredondar um pedido, ele cai antes de alguém ver.

⛔ Sem `Δ` nem setas no texto: o repositório já pagou tofu por um caractere que a fonte não tinha.

### Provas de mutação (as três restauradas)

| Mutação | Reprovou |
|---|---|
| passo de grelha fixo em vez de derivado | `the_grid_step_is_derived_from_the_framing` |
| `since` como identidade | `since_is_the_exact_inverse_of_merge` **e** `pointer_events_between_two_frames_add_up` |

---

## §8 — W9: criar formas e combiná-las (20/08)

### ⚠️ O buraco maior não era nenhum dos que eu tinha listado

A lista de abertos dizia *snap · pivô · perspectiva*. Nenhum era o maior: **o módulo não sabia CRIAR
uma forma.** Ele só editava a cena de demonstração — e um modelador a que não se pode juntar uma
caixa é um visualizador.

*Uma lista de abertos é escrita por quem já sabe usar a coisa.* O que falta ao **primeiro** gesto de
alguém não aparece nela.

### Criar

Quatro botões — Box · Sphere · Cylinder · Torus. A forma nasce:

| O quê | Como | Por quê |
|---|---|---|
| **onde** | `cam.target` — o centro do quadro | é o «onde estou a olhar», e o artista controla-o com o pan |
| **do tamanho** | `half_extent / 4` | ⭐ As duas metades são a **mesma condição: ela tem de ser VISTA.** Um tamanho fixo em unidades de mundo nasce invisível numa peça grande e tapa a janela numa pequena — e nos dois casos o artista conclui que o botão não funcionou |
| **onde na árvore** | operação selecionada → adota · folha selecionada → **irmã** · nada → raiz | pendurar uma forma numa esfera não quer dizer nada |
| **selecionada** | sim | o gizmo já fica em cima dela |
| **com `round`** | uma fração do tamanho | este é o módulo cujo argumento **é** o arredondamento; uma caixa de aresta viva ao nascer esconderia o que ele faz melhor do que o Blender |

⛔ Uma forma pendurada **fora** da peça é recusada: ela apareceria na Hierarquia e o traçado
ignorá-la-ia — um objeto que existe e não existe.

### Combinar — a autoria da booleana

⭐ A fileira **só é pintada quando pode agir**: um controle que aparece e não faz nada é pior do que
um que não aparece.

| Selecionado | O que os botões fazem |
|---|---|
| uma **operação** | trocam-na — e ⭐ **o raio da mistura sobrevive**. Ele é do nó, não da operação: perdê-lo obrigaria a re-encontrá-lo, e o gesto passaria a custar dois |
| **dois ou mais irmãos** | embrulham-nos numa operação nova, no lugar deles. ⚠️ **A ORDEM que entra é o significado**: `children[0]` menos os seguintes |

⛔ **Pai comum é exigido**, e não é conveniência: mover um nó para debaixo de outra operação muda o
que ele é subtraído de — um segundo gesto, com o seu próprio desfazer. Um «embrulhar» que o fizesse
em silêncio seria dois gestos com um nome só.

### ⭐ Um gate que estava VERDE POR ACIDENTE

A primeira versão de `wrapping_refuses_nodes_that_do_not_share_a_parent` usava **a raiz e um filho
dela**. Ele passava — mas porque a raiz não tem pai nenhum e a função sai mais cedo, **não** porque
os pais fossem diferentes.

A prova de mutação apanhou-o: retirar a exigência de pai comum deixava-o **verde**. Reescrito com
dois nós que *têm* pai, e pais diferentes; a mesma mutação agora reprova.

*Um gate que passa pelo motivo errado não prova nada* — e a única coisa que o distingue de um que
prova é a prova de mutação.

### ⚠️ Nota de ambiente

`check --workspace --all-targets` deu um **ICE do rustc** num alvo de teste que esta linha não toca.
`rm -rf target/debug/incremental` e ele passa: era o **cache incremental**, não o código. O
precedente já estava registado — *um `✗` pode ser o ambiente*.

---

## §9 — W10: as dimensões de cada forma (20/08)

### ⚠️ Só o filete era editável

A única coisa que se podia escrever numa primitiva era o **raio do filete**. Não havia como dizer
*"este cilindro tem 20 de raio e 50 de altura"* — só escalar uniformemente com o gizmo. **Um
modelador de precisão em que não se escreve um número é um modelador de escala.**

### O painel deixa de listar a árvore

| Antes | Agora |
|---|---|
| uma linha por nó, com o raio dele | as **dimensões do que está selecionado** |

⚠️ A lista antiga era uma segunda vista da **estrutura**, a competir com a Hierarquia — e sem onde
pôr largura, altura e profundidade. A divisão passou a ser a da casa: a **Hierarquia** mostra *o que
existe*, o **painel** mostra *os números do escolhido*. Sem seleção, ele di-lo.

### ⭐ A divisão dos limites: o documento dá a PAREDE, a vista dá o CONFORTO

Cada dimensão diz se tem limite de **validade** (`Dim::limit`): o filete de uma caixa não pode chegar
à meia-extensão dela, porque a fonte encolhida deixaria de existir. Isso é do documento e não se
negoceia.

⛔ **O teto de um slider não é isso.** A largura de uma caixa não tem limite nenhum — escrever um
seria inventar um número que a física não pede (`CLAUDE.md §0`). Quem escolhe até onde o **gesto**
vai é a vista, e a resposta é *o que cabe no enquadramento*: uma dimensão maior do que o quadro é uma
cujo efeito não se vê.

O **campo numérico continua sem teto**: digitar 1000 é uma afirmação sobre a peça, não sobre a
janela.

### Meias-extensões não aparecem

O documento guarda **meias**-extensões (é a forma que a distância assinada quer). Ninguém diz que uma
caixa tem «meia-largura 5»: a lista devolve a largura **inteira** e a escrita volta a dividir. ⚠️ A
conversão mora **num sítio** — sem isso, cada painel que a mostrasse teria a sua, e uma ficaria para
trás.

### ⭐ Encolher uma forma ENCOLHE o filete, e não é recusado

Um filete que deixa de caber é a situação **normal**, não um erro: o artista pediu o tamanho, e o
filete é o que **decorre** dele. Recusar obrigaria a desfazer o filete primeiro — dois gestos onde há
um, e é o que todo CAD resolve limitando em silêncio.

⚠️ **Em silêncio, mas não invisível**: o número do filete é uma linha do mesmo painel, e ela muda à
vista. Um valor que muda sozinho **sem aparecer** seria outra coisa.

### Um nome que passou a mentir

`RadiusBound` → **`Bound`**. O nome descrevia o primeiro uso e deixou de dizer a verdade no dia em
que as outras dimensões ficaram editáveis: a largura de uma caixa tem um alcance de gesto tanto
quanto um filete tem uma parede. *Um nome que descreve o primeiro uso passa a mentir no segundo.*

### Provas de mutação (as duas restauradas)

| Mutação | Reprovou |
|---|---|
| a caixa reporta meias-extensões | `a_box_reports_full_extents_not_half_ones` |
| o filete não é limitado ao encolher | `shrinking_a_shape_shrinks_its_fillet_instead_of_refusing` |

---

## §10 — W11: duplicar e apagar (20/08)

Fazer quatro furos iguais era: criar um cilindro, posicionar, criar outro, **re-digitar as
dimensões**, posicionar. **Duplicar é a alavanca que falta em qualquer modelador**, e ela não
existia.

### Duplicar copia a SUBÁRVORE

Não só o nó: o caso útil é copiar um **furo** que já é ele próprio uma subtração de várias formas.
Copiar só o topo daria um grupo vazio — e o artista veria o botão «funcionar» sem nada aparecer.

⚠️ **A ordem dos filhos é preservada**, e ela é o significado numa subtração (`children[0]` menos os
seguintes). Uma cópia que a baralhasse seria a mesma forma **só às vezes**.

### ⭐ A cópia sai UM DEGRAU da grelha, para a direita da TELA

Não é decoração, e a alternativa foi considerada: **duplicar em cima do original** é o que o Blender
faz — e ele resolve o resto entrando logo em modo de mover. Aqui não há esse modo, então uma cópia
exatamente por baixo seria **um botão que parece não fazer nada**: a única prova seria uma linha nova
na Hierarquia.

| Metade | De onde vem |
|---|---|
| **quanto** | o degrau da grelha — derivado do enquadramento (o menor número redondo que ainda se consegue mirar, §7) |
| **para onde** | a **direita da câmera** — é para onde «o próximo» vai em qualquer arrumação |

### ⛔ A raiz não se duplica nem se apaga pelo painel

Ela **é** a peça. Apagá-la deixaria o módulo sem nada para onde voltar (a cena inicial só existe no
primeiro quadro), e duplicá-la seria criar uma **segunda peça** — um gesto da *cena*, não uma edição
desta. Remover a peça continua a ser da **Hierarquia**, que é de onde o desfazer a traz de volta.

Apagar **limpa a seleção**: o gizmo ficaria aceso sobre uma entidade que já não existe.

### Provas de mutação (as duas restauradas)

| Mutação | Reprovou |
|---|---|
| duplicar copia só o topo | `duplicating_copies_the_whole_subtree_as_a_sibling` |
| a raiz passa a ser apagável | `the_root_is_neither_duplicated_nor_deleted_from_the_panel` |

---

## §11 — W12: as mesmas duas ações, pela Hierarquia (20/08)

Pedido do Enio: *"faça duplicate e delete funcionar tb na hierarchy"*.

### ⚠️ Duplicar dava um SÓSIA que não desenha nada

O braço genérico da Hierarquia copia `Transform` + `Sprite` + `Name`. Um nó de campo **não tem
nenhum dos dois** (só a raiz leva `Transform`): saía uma linha na Hierarchy sobre geometria nenhuma,
invisível para o traçado.

⭐ **É o MESMO defeito que a nota vetorial daquele bloco já descreve**, um módulo adiante:

| Entidade | O que o braço genérico produzia | Quem duplica de verdade |
|---|---|---|
| um **path vetorial** | um sósia sem geometria — ou, pior, dois donos do mesmo path | o documento vetorial, pela porta do painel |
| um **nó de modelagem 3D** | uma linha sem `FieldNode` nem `FieldPose` | `field3d_scene::duplicate_node`, a porta do painel |

*Uma entidade cuja geometria não está nela não se duplica clonando-a.* A decisão passou a ter nome
(`duplicate_kind`) e a tabela dos dois precedentes vive no doc dela — o terceiro módulo que caia
nisto encontra-a antes de a pagar.

E a cópia sai pela **mesma porta** do botão do painel: uma lei, dois chamadores. Duas contas para
*"onde vai a cópia?"* divergiriam no primeiro ajuste, com o artista a ver o mesmo gesto fazer duas
coisas conforme por onde o pediu.

### ⚠️ Apagar a peça fazia-a VOLTAR no quadro seguinte

*Delete* já funcionava para um nó. Para a **peça**, não: a ponte oferecia o documento **cozido** como
semente, e o comentário ao lado afirmava que ele *"deixa de existir"* — o que **nunca foi verdade**,
ele é reescrito a cada quadro. Sem raiz, a ponte replantava o que tinha acabado de cozer.

`Smoke::seed` separou-se do `Smoke::doc`, e a ponte **tira** a semente em vez de a copiar. *Uma
semente usa-se uma vez.*

### ⭐ Dois gates que estavam VERDES POR ACIDENTE

A prova de mutação apanhou os dois — e **nenhum provava o que dizia**:

| Gate | Por que passava com o bug reposto | A cura |
|---|---|---|
| o da semente | chamava `sync_scene_and_birth` com `None` **à mão** — nunca chegava à decisão de *o que a ponte oferece* | passa agora pelo `ecs_bridge` |
| o de duplicar | os gates chamavam a porta diretamente, então apagar o braço da Hierarquia **não reprovava nada** | a decisão saiu para `duplicate_kind`, com gate próprio |

⚠️ O segundo é a **costura não-testada** da `DIRETIVA_IMPLEMENTACAO` §1 — a causa nº 1 que este doc
cita desde a §3 — e ela estava neste arquivo, escrita por mim, no mesmo dia. *A prova de mutação é a
única coisa que distingue um gate que prova de um que acompanha.*

O gate novo afirma também o caso **negativo** (uma entidade comum continua a ir para o braço
genérico), senão ficaria verde por reclamar de mais.

---

## §12 — W13: digitar a posição, e dois gates que não provavam nada (20/08)

Par da W10: dava para digitar o **tamanho** e não a **pose**. *"Move isto exatamente 10 para a
direita"* era impossível — arrastar com grelha, sim; escrever o número, não.

### Posição, e a convenção da casa

`Position X/Y/Z` entram no painel, em coordenadas **LOCAIS** — que é o que o Inspector da casa
mostra (`Transform.translation` é local, e o readout do gizmo 2D diz por extenso que o delta é local
*"porque é isso que o Inspector mostra"*). ⚠️ Um painel que mostrasse **mundo** contradiria o número
ao lado no dia em que alguém agrupasse. O gate usa um pai deslocado, onde mundo ≠ local.

`Param::{Pos, Scale, Dim}` substitui o índice cru: uma convenção implícita entre duas crates
(«0..2 é a posição») sobrevive até alguém acrescentar uma linha no meio — e aí o controle escreve
noutro número, em silêncio.

### ⛔ Escala não aparece numa folha, e o gizmo também não a usa lá

Uma folha escalada teria **duas verdades sobre o mesmo tamanho visível**: a largura que o painel
mostra e o fator da pose. Uma caixa de 1 escalada 2× mede 2 na tela e continua a dizer «1» — e o
artista não tem como saber qual das duas o gesto seguinte mexe.

| Nó | O que «crescer» mexe | Por quê |
|---|---|---|
| **folha** | as **dimensões** (`scale_primitive`) | mesma forma, **um** número a mudar — e é o que o painel mostra |
| **grupo** | o fator da **pose** | ele não tem dimensões próprias: ali a pose é a única resposta e não compete com nada |

### ⭐ Dois gates que não provavam nada

**1. 30 gates deixaram de correr e a suíte ficou VERDE.** Um corte de bloco levou a declaração do
módulo de gates da cena, e o terminal disse `✓ 52 passaram` — de 86. *Um teste que não é compilado
não reprova: ele desaparece.* Só a contagem o denunciou.

Cura: `every_field3d_test_file_is_declared_by_a_module`, que lê o diretório e exige que cada
`field3d_*_tests.rs` seja **nomeado** por código.

**2. E o gate novo passou pelo motivo errado.** Ele procurava a declaração no **texto** dos arquivos
— e encontrava-a **no próprio doc-comment dele**, que a cita como exemplo. A prova de mutação
apanhou-o: apagar a declaração de verdade deixava-o verde.

⭐ *Um gate que procura texto tem de procurar **código**.* Ele passou a ignorar linhas de comentário.

### Provas de mutação (as quatro restauradas)

| Mutação | Reprovou |
|---|---|
| a declaração dos gates da cena é removida | `every_field3d_test_file_is_declared_by_a_module` (**depois** de curado) |
| escalar uma folha volta a mexer na pose | `scaling_a_leaf_grows_its_dimensions_and_leaves_the_pose_alone` · `the_readout_is_the_pose_the_world_took` |
| a posição escrita sai deslocada | `typing_a_position_moves_the_node_in_its_parents_frame` |

---

## §14 — W14: a rotação em números, e um piso que faltava a TODA linha (20/08)

A pose já se movia e crescia por número; **rodar** só existia com o rato. E ao ir escrever a linha
descobriu-se que a faixa de uma linha só tinha **uma** ponta.

### ⭐ O trio é um NOME do quaternion, não uma segunda verdade

A pose guarda um quaternion. O painel mostra o **XYZ Euler canónico** dele, em graus, na ordem do
Blender (`R = Rz · Ry · Rx`) — e o que fecha o ciclo é a **orientação**, nunca o trio:

| | |
|---|---|
| `quat_from_euler(quat_to_euler(q))` | `== q` **como rotação** — é o gate |
| `quat_to_euler(quat_from_euler(e))` | `== e` **só para `e` canónico** — e não é o gate |

Uma orientação tem infinitos trios que a nomeiam (mais uma volta em qualquer eixo; nos polos, uma
família inteira). Exigir a segunda linha do código seria exigir uma coisa que **não é verdade**, e a
cura seria guardar o trio.

⛔ **Guardar o trio autorado ao lado do quaternion foi pesado e recusado.** O gizmo roda em torno de
eixos **arbitrários** (a argola da vista não é X, Y nem Z) e escreve o quaternion — o trio guardado
seria um cache invalidado por **todo** arrasto, e o documento passaria a ter duas respostas para
*"como é que isto está rodado"*. O que se paga por não o guardar está medido e escrito: um ângulo
além de meia volta é **renomeado** (200° aparece como −160°, o mesmo sítio).

> ⚠️ **Esta secção dizia também que «o eixo do meio reflete ao passar de 90°, e a peça vai sempre
> para onde foi pedida». A segunda metade estava ERRADA** — e o Enio viu-o no smoke. Ali não havia
> um nome novo para o mesmo sítio: havia **duas orientações a alternar**. Ver §15.

**Trava de cardan:** em `β = ±90°` o X e o Z deixam de ser distinguíveis — só a soma (ou a diferença)
é um facto. Ali a extração põe `γ = 0` e dá o resto ao X: determinístico, e não três números a tremer
numa peça parada. O limiar `EULER_LOCK_EPS = 1e-4` é derivado do **`f32`** (as entradas da matriz
carregam ~1e-7 de erro; a `|cos β| = 1e-4` o `atan2` já vale ~0,06°), e é ele que fixa a tolerância
`2e-4` do gate de ida-e-volta — não o `f32::EPSILON`, que reprovaria o caso que a lei trata de
propósito.

### ⭐ Uma faixa tem DUAS pontas — a regressão que a W13 embarcou

`Dim` só dizia o **teto**, e o painel punha o piso em **zero** para todas as linhas. Numa largura
está certo (o documento recusa `≤ 0`); numa **posição** é um defeito com sintoma **mudo**: digitar
`-0,5` era reescrito para `0` pelo espelho do controle, e a peça ia para a origem. *Um número que a
UI recusa em silêncio é a pior forma de recusa* — e ele sobreviveu ao smoke da W13 porque o valor
experimentado foi positivo.

A cura é o [`Span`]: cada grandeza diz a **forma** da faixa e de que recurso vem cada ponta, e quem
fecha as pontas abertas é a **vista**, num sítio só ([`param_rows`]).

| `Span` | piso | teto | de que recurso |
|---|---|---|---|
| `Positive` | 0 | vista | o documento recusa `≤ 0`; o teto é o que cabe no quadro |
| `Wall(w)` | 0 | `w` | a única ponta que o **documento** impõe (o filete) |
| `Free` | −vista | +vista | a origem não é um canto do mundo |
| `Turn(180)` | −180 | +180 | a própria **representação** — nem documento nem vista têm voto |

⚠️ E eram **duas portas** sobre a mesma faixa: o `paint` instala o mapeamento e o `event` faz a conta
à mão. Um par destes só falha quando **discordam**, e cada lado lido sozinho parece certo — daí o
gate `the_dispatched_value_is_the_one_the_painted_mapping_promises`, que lê o mapeamento do *store*
(o que a pintura de facto deixou lá) e o compara com o número que saiu pela fila, nas duas pontas.

### As casas decimais passaram a ser DERIVADAS

`RADIUS_DECIMALS = 3` foi escrito quando a única linha era o filete, e passou a servir cinco
grandezas. A regra que não é palpite: **o número na tela tem de distinguir dois passos consecutivos
do arrasto** — `ceil(−log10 passo)`, mais uma casa para o que se digita entre dois passos (senão
escrever `45,5` mostraria `46`, e o painel mentiria sobre o documento).

| linha | passo | casas |
|---|---|---|
| filete (curso 0,12) | 0,0012 | 4 |
| largura / posição (curso 2,4) | 0,024 | 3 |
| ângulo (curso 360) | 3,6 | 1 |

### ⚠️ Um portão que estava VERMELHO desde antes desta linha

O `shell_files_respect_hr18_loc_cap` reprovava com **cinco** arquivos acima de 600 LOC — três deles
sem uma linha minha esta wave. Ele nunca apareceu porque as corridas anteriores levavam **filtro**, e
o filtro é onde a resposta se perde (`CLAUDE.md` §2: 98,9% das corridas com filtro escrito à mão).

Curado por **corte, não por exceção** — e por **assunto**, não por tamanho:

| Antes | Depois |
|---|---|
| `field3d_gizmo.rs` 819 | 547 + `field3d_gizmo_drag.rs` 298 (*onde estão as alças* vs *o que o arrasto pede*) |
| `field3d_smoke.rs` 803 | 454 + `_scenes.rs` 199 + `_draw.rs` 180 |
| `field3d_scene.rs` 664 | 450 + `field3d_scene_panel.rs` 236 (a ponte com o painel) |
| `field3d_gizmo_tests.rs` 739 | 303 + `field3d_gizmo_drag_tests.rs` 449 |
| `field3d_scene_tests.rs` 1469 | 538 + `_gesture_tests.rs` 434 + `_edit_tests.rs` 518 |

⭐ **Todos são módulos-filhos com `use super::*` e re-export**: as fixtures continuam a existir **uma
vez** (duas cópias divergiriam na primeira mudança, e os dois arquivos mediriam cenas diferentes com
o mesmo nome), e nenhum chamador mudou uma linha. *Cortar um arquivo não pode custar uma reescrita em
cada sítio que o chamava.*

### Provas de mutação

| Mutação | Reprovou |
|---|---|
| a extração de Euler perde o ramo da trava | `at_the_pole_the_extraction_is_finite_and_the_third_angle_is_pinned` |
| a ordem passa a `Rx · Ry · Rz` | `the_order_is_the_blender_xyz_euler` |
| o despacho volta a `track * teto` | `the_dispatched_value_is_the_one_the_painted_mapping_promises` · `a_row_whose_floor_is_negative_can_reach_it` |
| as casas decimais voltam a ser constantes | `two_neighbouring_drag_steps_never_read_the_same` · `a_value_typed_between_two_steps_is_not_rounded_away` |

⚠️ **A quarta mutação passou VERDE à primeira, e o defeito era do gate.** Ele sondava os cursos de
0,12 a 360 — e três casas fixas de facto distinguem dois passos em todos eles. A tabela não continha
a ponta que a regra existe para servir. Curado com o **filete de uma peça de 2e-4** (nada impede
digitar um raio desse tamanho) e com sondas **relativas ao curso**: sondar um passo de `2e-6` a
partir de `45,0` mediria o ULP do `f32` em 45 (`3,8e-6`), não a formatação.

⭐ E o teto das casas passou a ser derivado no mesmo movimento: **6**, porque o ULP de uma coordenada
de ordem 1 é `1,19e-7` — a sexta casa é a última que ainda distingue dois valores de verdade.
*Uma fixture só prova o que ela contém.*

[`Span`]: ../../crates/ph2d-field/src/dims.rs
[`param_rows`]: ../../shells/desktop/src/field3d_scene_panel.rs

---

## §15 — W14.1: o bug do eixo do meio, e a lei que faltava (20/08)

Enio, no smoke da W14: *"bug em rot y. Acima de 70 muda x e z e treme"*.

### A medição, antes de qualquer hipótese

Uma sonda varreu o Y como um arrasto varre, nos três cilindros da cena 1, escrevendo **o mesmo alvo
duas vezes** — que é o que um arrasto faz, quadro após quadro:

```
Y= 90.0 -> [0.0, 90.0, 0.0]      / repetido [0.0, 90.0, 0.0]
Y= 93.6 -> [180.0, 86.4, 180.0]  / repetido [~0, 86.4, ~0]   <<< NAO IDEMPOTENTE
```

⭐ **A escrita não era ponto fixo.** A segunda escrita do mesmo valor produzia **outra orientação**,
porque partia do trio já **renomeado** pela leitura anterior. Num arrasto isso é um **ciclo de dois**:
a peça alterna entre duas poses com o dedo parado. O «treme» tinha nome, e o «muda x e z» era o
renome a piscar a 60 Hz.

⚠️ **O «acima de 70» não era 70** — a quebra é em 90. O que se vê abaixo disso é o ciclo já em curso
durante o arrasto. *O número que o utilizador reporta é onde ele NOTOU, não onde o mecanismo parte:
a sonda é que diz onde.*

### A lei que faltava

> **Escrever o mesmo valor duas vezes não pode mexer a peça.**

E a única forma de a cumprir sem guardar um segundo estado é **o alvo entrar já canónico**, para a
leitura seguinte o devolver intacto:

| eixo | faixa canónica | alvo fora dela |
|---|---|---|
| X, Z | `(−180°, 180°]` | **enrola** — 200° é o mesmo sítio que −160° |
| Y (o do meio) | `[−90°, 90°]` | **prende** |

⚠️ **Prender o do meio não perde orientação nenhuma**: toda orientação tem um trio canónico com
`|β| ≤ 90°`. Perde-se o **nome** — «Y = 120» deixa de ser digitável, e o mesmo sítio escreve-se
`X = 180 · Y = 60 · Z = 180` (há gate a prová-lo alcançável). É a diferença face ao Blender, e é o
preço — agora concreto — de não guardar o trio.

⭐ E foi o mesmo defeito na faixa da linha: `Span::Turn(180)` nas três oferecia ao slider do meio
metade de um curso que a leitura seguinte renomeava. `ROT_SPAN_DEG = [180, 90, 180]`.

### Na trava de cardan o Z é INERTE — e a linha DIZ-SE inerte

Em `β = ±90°` o X e o Z são o **mesmo** eixo físico e a forma canónica dá tudo ao X. Aplicar um Z ali
fazia o X escorregar mais um tanto a **cada** escrita — o mesmo ciclo por outro caminho, e este
girava sozinho.

⛔ **As três saídas sem memória foram pesadas e nenhuma serve:**

| | por que não |
|---|---|
| aplicar o Z encaminhado para o X | não é ponto fixo: o X escorrega a cada quadro — o ciclo, outra vez |
| reinterpretar o Z como o parâmetro livre | idempotente **e destrutivo**: escrever o Z deitaria fora o X que lá estava |
| impedir o Y de chegar a 90 | `90` é o ângulo que mais se digita; pô-lo fora de alcance é pior do que a trava |

Sobra guardar o trio autorado (o que o Blender faz), que é a segunda verdade recusada acima. Então o
Z é **recusado** — e, porque *uma affordance que não pode ser honrada é pior do que nenhuma*, a linha
deixa de ser pintada como controle: ela sai como **facto** (rótulo + número, apagado), sem slider,
sem campo e **sem entrada no índice de acerto**.

⭐ **A mesma porta decide as duas coisas** ([`rotation_axis_is_free`]): quem recusa a escrita é quem
diz ao painel que não há controle. Duas respostas para *"este eixo responde?"* dariam um controle
vivo sobre uma escrita recusada — o número a saltar e a voltar —, e há gate a prendê-las.

⚠️ E a linha **não desaparece**: o valor continua a ser um facto a ler, e uma linha que some faria o
painel saltar de tamanho a cada travessia dos 90°.

[`rotation_axis_is_free`]: ../../crates/ph2d-field/src/xform.rs

### Provas de mutação

| Mutação | Reprovou |
|---|---|
| o eixo do meio volta a enrolar | `writing_the_same_angle_twice_does_not_move_the_part` · `the_middle_angle_is_pinned_at_a_quarter_turn_and_the_outer_two_wrap` |
| a trava deixa de recusar o Z | `at_the_pole_the_third_angle_is_refused_instead_of_creeping` |
| a faixa do meio volta a 180 | `a_position_admits_negatives_a_dimension_does_not_and_an_angle_is_half_a_turn` |
| a linha inerte volta a ser pintada como controle | `an_inert_row_registers_nothing_to_click` |
| o painel deixa de perguntar se o eixo responde | `at_the_pole_the_third_angle_reaches_the_panel_as_a_fact` |

⚠️ O gate de idempotência escreve **três vezes** no caso da trava e varre **por cima** do quarto de
volta: uma escrita só não distingue *"recusado"* de *"aplicado uma vez"*, e uma varredura que parasse
em 90 ficaria verde sobre o defeito.

---

## §16 — W15: a lente, e o raio que era construído duas vezes (20/08)

Em projeção paralela não se julga forma: um cilindro de frente é um retângulo, e duas peças a
profundidades diferentes medem o mesmo. A perspectiva era item aberto desde a W2.

### ⭐ A lente é SÓ uma lente

`Lens::{Ortho, Perspective { half_fov }}`, e o `half_extent` **continua a querer dizer a mesma
coisa nas duas**: quantas unidades de mundo cabem em meia altura de quadro **no plano do alvo**.
Daí sai a propriedade que mantém o resto do módulo intacto — as duas imagens **coincidem
exatamente** naquele plano, e zoom, enquadramento, o passo da grelha e a lei do pan não mudam de
lei. Uma perspectiva que mudasse o significado do `half_extent` obrigaria a reconferir cada número
que dele deriva, e **nenhum deles ficaria vermelho** ao mudar.

A distância do olho sai da definição: `dist = half_extent / tan(meia abertura)`. E a abertura é
**derivada da referência declarada**, escrita como a conta e não como o resultado: o Blender abre
uma câmera com 50 mm sobre um sensor de 36 mm ⇒ `atan(18/50)`.

### ⭐ O raio era construído DUAS vezes

A marcha de raios reconstruía a aritmética do `Orbit::ray` com um afastamento próprio — no mesmo
módulo cujo doc promete que *"a projeção é a MESMA do traçador"*. Enquanto as duas eram paralelas,
elas concordavam por acidente; a lente convergente teria deixado **uma delas paralela**, com a peça
traçada de uma forma e as alças noutra, e nada vermelho.

Passou a haver uma porta: `Orbit::ray_at_plane`, que a marcha chama. E um gate que a prende —
manda um ponto pela projeção e pergunta ao raio daquele pixel se ele passa por lá, **nas duas
lentes**.

| o que mudou | por quê |
|---|---|
| `project` devolve `Option` | um ponto ao lado do olho ou atrás dele **não tem pixel**; inventar um seria oferecer um gesto que agarra noutro sítio. Encaixa no `live` do gizmo, que já existia |
| `px_per_world_at(ponto)` | com convergência a escala **depende da distância**; um braço dimensionado pela constante do quadro encolheria com a peça a afastar-se, e o `MIN_ARM_PX` passaria a morder por distância em vez de por ângulo |
| `ORTHO_START` nomeado | o `4.0` solto da marcha ganhou o recurso que o justifica, e a nota de que a convergente **não** o usa (o olho já está recuado) |

### ⚠️ A tecla é a comparação que a nota pedia

A nota da câmera dizia por extenso que a perspectiva *"merece a sua própria comparação lado a lado,
não uma troca silenciosa"*. **`Numpad5`** alterna as duas — a tecla do Blender para a mesma coisa.
O default é a convergente, que é o que um modelador espera, e há gate no default (a tecla não o
prova).

⭐ E a guarda *"o ponteiro está sobre a janela 3D?"* — escrita à mão em cada porta de tecla, com a
nota só numa delas — virou uma função. A tecla seguinte a nascer teria copiado a condição e deixado
a nota para trás.

### ⚠️ Duas coisas que a lente quebrou, e nenhuma era a lei

**1. Recursão infinita, e ela aparece como pilha estourada.** `from_yaw_pitch` passou a herdar
`..Self::default()`, e o `Default` é escrito **em termos dela**. Não é erro de compilação: é um teste
a abortar com `stack overflow`.

**2. O gate do pan reprovou, e a causa era a FIXTURE.** Ele media o centroide do traçado a
`half_extent = 0,2`; com a lente convergente isso põe o olho a 0,55 da peça, ela transborda o quadro,
e o centroide de um quadro cheio é o centro dele — **parado, com a lei correta por trás**. A pergunta
certa é sobre um ponto do **plano do alvo**, que é onde a lei do pan fala, e ali a afirmação é
**exata** (`dx` pixels) em qualquer zoom e com qualquer lente. *Uma fixture só prova o que ela
contém* — segunda vez nesta linha.

### Provas de mutação

| Mutação | Reprovou |
|---|---|
| o raio da convergente volta a ser paralelo | `the_ray_of_a_pixel_passes_through_what_projects_onto_it` (+4 gates do traçado) |
| o braço volta à escala do quadro | `a_screen_sized_arm_measures_the_same_at_any_distance` |
| um ponto atrás do olho ganha pixel | `a_point_at_or_behind_the_eye_has_no_projection` |

⚠️ O gate das duas lentes mede **as duas** com a mesma medida: um gate só sobre a convergente
passaria com a paralela também convergente.

---

## §17 — W16: a casca e o afastamento — a tese em duas linhas de aritmética (20/08)

O plano da linha (§6, W4) pede *"união/diferença/intersecção com raio por operação, **casca,
offset**, draft, padrões"*. As booleanas fecharam na W9; estes dois não existiam — e são onde a tese
do módulo mais aparece.

### ⭐ Por que estes dois, e não outros dois

| verbo | a conta | por que ela não pode falhar |
|---|---|---|
| **casca** | `\|f\| − t/2` | o módulo de uma distância **é** a distância à mesma superfície, vista dos dois lados |
| **afastamento** | `f − d` | deslocar a superfície por uma distância é o que uma distância assinada **é** |

⚠️ **Numa malha, a casca é a operação que FALHA**: ela pede um offset da superfície, e um offset de
malha auto-intersecta em toda concavidade mais apertada do que a espessura. Todo modelador de malha
tem um botão de casca com uma lista de exceções ao lado. *Aqui a lista não existe*, e é essa a razão
de o módulo ser um campo.

⭐ **E o gate prova-o com um oráculo INDEPENDENTE**, na disciplina desta crate: a casca de uma esfera
é comparada com a **subtração de duas esferas analíticas** — escrita com as primitivas e a booleana
que já existiam. A igualdade é **exata**, não aproximada. Um gate que comparasse a casca consigo
mesma provaria que o código faz o que o código faz.

### A PILHA, e de onde a forma dela vem

Os modificadores de um nó são uma **lista ordenada**, não um grafo: encascar-e-afastar não é
afastar-e-encascar, e a ordem tem de ser dita. É a mesma forma que os *Live Path Effects* do
vetorial mediram ([ADR-0132]: *"uma pilha por path, não um grafo de nós"*) — um grafo paga um editor
de grafo para exprimir uma sequência. Há gate: as duas ordens têm de **discordar** no mesmo ponto,
senão um conjunto sem ordem teria entrado no lugar da lista sem nada acusar.

### Onde cada peça foi parar, e por quê

| decisão | razão |
|---|---|
| `FieldMods` é **componente próprio e opcional** | quase nenhum nó tem modificador; e apendar campo a um componente **posicional** quebraria todo projeto já gravado. Um componente **novo** custa zero degraus de `PROJECT_SCHEMA` — precedente do `VecStrokeProfile`/ADR-0148, escrito na escada |
| a pilha corre **antes** da pose | assim a espessura é um número **local**, e um ancestral escala-a junto com tudo o mais do nó — *uma regra para todo número deste módulo* |
| os botões são **interruptores** | um modificador é um **estado** do nó; um botão que só acrescenta deixaria o artista a empilhar cascas sem perceber, e sem forma de tirar a não ser desfazendo |
| tirar o último **remove o componente** | o undo compara **bytes**: um componente presente e vazio não muda a forma e muda os bytes, e ligar-e-desligar deixaria um passo a mais do que o artista fez |
| a casca **nasce numa fração da peça** | um número absoluto é invisível numa peça grande e engole uma pequena — nos dois casos o botão parece não ter feito nada. Há gate com duas peças de escalas diferentes |

⚠️ **`FIELD_DOC_VERSION` 2 → 3**, e a migração é **vazia** — o que tem de estar escrito: nada
persiste um `FieldDoc` (ele é cozido da cena a cada quadro; o arquivo guarda os *componentes*). O
degrau sobe na mesma, porque a alternativa é o número deixar de querer dizer alguma coisa no dia em
que alguém o persistir. Os dois gates de bytes pinados foram **re-pinados com a conta ao lado**
(145 → 148 e 84 → 85: um byte de comprimento da pilha vazia, por nó) — *legítimo porque a versão
subiu junto e a conta bate*.

### Provas de mutação

| Mutação | Reprovou |
|---|---|
| a casca deixa de centrar a parede | `the_shell_of_a_sphere_is_exactly_the_difference_of_two_analytic_spheres` · `the_wall_measures_the_thickness_that_was_asked_for` |
| a pilha deixa de correr | os quatro gates do avaliador |
| a espessura de nascimento vira constante | `a_shell_is_born_as_a_fraction_of_the_part_not_a_fixed_number` |
| o interruptor volta a só acrescentar | `the_modifier_button_is_a_switch_not_a_stack_of_shells` |

⚠️ **E uma prova de mutação foi interrompida pelo tempo LIMITE com o código mutado na árvore.** O
`cp` de restauro é a última linha do laço, e o corte veio antes dela. Só o `git status` o denunciou.
*Uma mutação que não é desfeita é uma mutação que shipa* — a defesa é a mesma de sempre: conferir a
árvore, não a saída.

[ADR-0132]: ../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md

---

## §18 — W17: os PADRÕES — e a pilha aguentou-os sem arquitetura nova (20/08)

Fecha a W4 do plano menos o *draft*: **espelho**, **matriz linear** e **matriz radial**. Entraram
como mais três [`Unary`] na pilha da W16 — *zero* arquitetura nova, que é o sinal de que a forma da
W16 estava certa.

> ⚠️ **Achado do Enio, no smoke: o espelho é INVISÍVEL nas cenas que temos.** *"mirror não pode ser
> visto pois usa o centro do objeto e nossos objetos são simétricos."* Ele está certo, e o doc-comment
> do gate já dizia metade disto (*"espelhar uma folha centrada em si é um no-op por construção"*) —
> **sem tirar a conclusão de produto**: o único alvo útil é um **grupo** descentrado, e os grupos
> das cenas de smoke também são simétricos. O verbo funciona e **não se consegue demonstrar**.
> Adiado por decisão dele (*"depois resolveremos"*); fica no §13.

### ⭐ O que um campo dá de graça, e uma malha cobra N vezes

Uma matriz num modelador de malhas são **N cópias da geometria**. Aqui é **uma dobra do domínio**:
leva-se o ponto para a célula dele e avalia-se a forma **uma vez**. Uma matriz de 64 custa o mesmo
que uma de 2 — e o gate prova-o com o **oráculo independente** de sempre: uma matriz de N é,
byte a byte, a **união de N cópias transladadas** escritas à mão.

⚠️ **E a matriz é FINITA.** A receita clássica (`mod`) repete para sempre, e uma matriz infinita não
é uma peça: ela enche o quadro e não há como a parar. O índice preso (`clamp`) é o que a torna um
objeto — com gate nas duas pontas.

### ⚠️ Duas células, e por quê

A receita clássica de repetição limitada olha **só a célula do ponto**, e ali ela **superestima** a
distância quando a forma está descentrada: existe uma cópia vizinha mais perto do que a da célula, e
o campo não a vê. *Superestimar é o erro caro numa marcha de raios* — o passo salta por cima da
superfície, e o sintoma é a peça **com buracos**, não um erro. Olhar também a célula **do lado para
onde o ponto pende** custa duas avaliações e devolve a distância exata enquanto a forma couber em
1,5 células. Medido: em `x = 0,31`, a célula própria dá **0,54** e a vizinha **0,06** — nove vezes
menos.

### ⚠️ E o gate desse mecanismo ensinou duas coisas, as duas sobre o GATE

**1. A primeira versão media `‖∇f‖ = 1` na costura e reprovava sobre um campo CORRETO.** No plano
entre duas cópias está o **eixo medial**, onde uma distância assinada é legitimamente
não-diferenciável — `∂f/∂x` é zero ali por simetria. `‖∇f‖ = 1` vale *quase* em todo lado, e o gate
escolheu exatamente o ponto da exceção.

**2. E a fixture não continha o fenómeno.** Com uma **esfera centrada na célula** a receita de uma
célula só **já é exata**: com `round`, a célula do ponto é a do centro mais próximo, e para uma forma
radialmente simétrica o centro mais próximo é a cópia mais próxima. O defeito só aparece com a forma
**descentrada** — um grupo com a peça pendurada de lado. *Uma fixture só prova o que ela contém* —
terceira vez nesta linha, e das três esta foi a que mais custou a ver.

### ⭐ O eixo é o do NÓ, e isso apagou uma UI inteira

Espelho e matriz trabalham no **X local**; quem quer outro eixo **roda o nó**. Não é economia: é a
lei que o [`Primitive::Cylinder`] já escreve (*"outro eixo se obtém pela rotação do nó"*) e que o
`Revolve` repete. Um seletor de eixo por modificador seria um **terceiro vocabulário de orientação**
no mesmo painel, ao lado do gizmo e da pose.

### O que a matriz forçou, e que era melhoria

| antes | depois | por quê |
|---|---|---|
| um modificador tem **um** número | `Unary::dims()` / `set_dim(field, …)` | uma matriz tem *quantas cópias* **e** *que espaçamento*; separá-las em dois modificadores seria partir uma coisa em duas para caber num campo. É a mesma forma que `dims` já usa para uma primitiva — **um vocabulário, não dois** |
| `Param::Mod(u16)` | `Param::Mod { slot, field }` | achatar a pilha numa lista de números faria inserir um modificador no meio **renumerar** tudo depois — com um arrasto a meio a escrever noutro campo |
| toda linha é fracionária | `Span::Count { max }` + `ParamRow::integral` | três coisas mudam de uma vez: passo **1**, **zero** casas decimais, piso **1**. Deduzir *"parece inteiro, logo é"* daria uma linha que muda de comportamento quando o valor calha em `3,0` |

### Provas de mutação

| Mutação | Reprovou |
|---|---|
| a matriz volta a olhar só a célula do ponto | `an_off_centre_shape_still_measures_to_the_nearest_copy` |
| a coroa volta a olhar só a fatia do ponto | `a_radial_array_of_n_is_exactly_the_union_of_n_rotated_copies` |

### A coroa, e a MESMA armadilha uma quarta vez

A matriz radial é a mesma ideia noutra coordenada: em vez de dobrar o `x`, dobra-se o **ângulo**
(`θ − Δ·k`, com `Δ = 2π/count`). Duas fatias pelo mesmo motivo, e o eixo é o **Z** — o do
[`Primitive::Cylinder`], porque uma coroa de parafusos gira em torno do eixo do flange.

⚠️ **No eixo (`x = y = 0`) não há ângulo.** A conta não divide por `r` — ela reconstrói o ponto por
`r·cos θ'` —, então em `r = 0` o resultado é a origem, sem caso especial e sem `NaN`. Um `NaN` ali
envenenaria o traçado **inteiro**: a marcha compara com `NaN` e nenhum pixel acerta.

⛔ **E a prova de mutação da coroa passou VERDE à primeira, pela quarta vez pelo mesmo motivo.** A
fixture tinha as esferas **centradas na fatia** — e aí a receita de uma fatia só já é exata. A classe
tem agora nome escrito: *a repetição só erra com a forma **descentrada na coordenada repetida***.
Curado pondo a esfera a **meia fatia** do centro dela.

⚠️ E o gate do espelho leva a mesma armadilha da matriz escrita por extenso: a pilha corre **antes**
da pose do nó, então deslocar a folha pela pose dela **não** a tira do plano do espelho — a fixture
tem de ser um **grupo**. Espelhar uma folha centrada em si é um no-op por construção, e um gate que
o fizesse passaria sem nada a defender.

---

## §19 — W18: a INCLINAÇÃO — o primeiro operador que não é exato, e a conta que a medição refutou (20/08)

Fecha a W4 do plano. `Taper` inclina a secção transversal ao longo do **Y**: `k(y) = 1 + declive·y`,
o ponto vai para o espaço não-inclinado e o valor volta multiplicado por `k` — a mesma receita de
duas metades que a pose usa para a escala uniforme.

### ⛔ Ele NÃO devolve uma distância exata, e é o primeiro do módulo

A escala **varia com `y`**, e é essa variação que estraga: `∇g` ganha um termo que a multiplicação
por `k` não cancela. Perto da superfície o erro desaparece — que é onde a marcha mais precisa dele —,
mas longe ele **superestima**, e superestimar é o erro que faz o raio saltar por cima da peça.

A cura é dividir, o que torna o campo um **bound conservador**. E é aqui que a wave ensinou:

### ⭐ A minha conta estava errada, e a sonda refutou-a

Derivei à mão que dividir por `1 + |declive|` bastava. A medição de `‖∇f‖`:

| declive | com `1 + s` | com `1 + 2s` |
|---|---|---|
| 0,25 | **1,12** ⛔ | 0,93 ✅ |
| 0,50 | **1,20** ⛔ | 0,90 ✅ |
| 1,00 | **1,30** ⛔ | 0,87 ✅ |
| 2,00 | **1,40** ⛔ | 0,84 ✅ |

⚠️ **Acima de 1 o campo superestima** — a falha exata que a divisão existe para evitar, em **todo**
o alcance. O `2` saiu da tabela, não da álgebra. *Uma derivação à mão é uma hipótese; a tabela é o
facto.*

### ⭐ E a primeira medição do CUSTO também enganava

A sonda do gradiente diz que o **pior passo** no declive 1 é 1/300 de um passo cheio. Isso sugeriria
um teto muito mais baixo. O quadro traçado (320×240) diz outra coisa:

| declive | ms/quadro | razão |
|---|---|---|
| 0,00 | 9,89 | 1,00× |
| 0,25 | 12,22 | 1,24× |
| 0,50 | 15,09 | 1,53× |
| **1,00** | **20,00** | **2,02×** |
| 1,50 | 24,77 | 2,51× |

**Pouquíssimos pixels pagam o pior passo.** *O pior caso não é o custo; o quadro é.*

Não há joelho — o custo sobe liso —, então o teto é uma escolha de **orçamento**, e o número escrito
é o que ele compra: **no teto, o traçado custa o dobro**. `MAX_TAPER_SLOPE = 1` são 45°, generoso
para o que um draft de moldagem pede (1° a 5°) e suficiente para dar forma.

⚠️ **O piso em `k` impede a inversão**: em `y = −1/declive` a secção colapsa, e passando disso ela
viraria do avesso — a peça sairia com o interior para fora. Preso a `0,01`, o que acontece além do
ápice é a secção ficar congelada nele: uma forma, não um defeito.

### `Span::Walls` — simétrica e fechada pelo DOCUMENTO

O declive é adimensional: a vista não tem o que dizer sobre ele, e as duas pontas são um **facto**
(o custo de marcha). É a irmã da `Span::Free` com as pontas fechadas, e a diferença é de onde vem o
número.

### Provas de mutação

| Mutação | Reprovou |
|---|---|
| o divisor volta à conta derivada à mão | `the_taper_never_overestimates_the_distance` |
| a inclinação perde o sinal (um `abs()` a mais) | `the_taper_narrows_one_way_and_widens_the_other` |

⚠️ O segundo gate mede os **dois** sentidos de propósito: sem a metade negativa, um `abs()` a mais
passaria despercebido e o artista nunca conseguiria a forma oposta.

---

## §20 — W19: a peça SAI — e a porta existia, sem ninguém a abrir (20/08)

Metade da W5 do plano. `ph2d_field_eval::mesh` estava escrita, testada na crate e com **zero
chamadores**: o módulo sabia extrair uma malha e nada no app a pedia. *Uma porta que ninguém abre
não é uma porta.*

### ⭐ Exportar é a primeira vez que o módulo PERDE informação de propósito

Um campo tem resolução **infinita** — o filete é uma fórmula, e ampliar não revela serrilha. Uma
malha não: ela é uma escolha de quantos triângulos. Todo o resto foi construído para **não** decidir
isso cedo (ADR-0161 §2), e aqui a decisão é inevitável. Então ela é **explícita e medida**.

`measure_export_resolution`, cena 1 (três cilindros com filete), release:

| prof | triângulos | ms | |
|---|---|---|---|
| 4 | 1 752 | 3,1 | |
| **5** | **6 888** | **3,7** | ← *Draft* |
| 6 | 27 716 | 8,2 | |
| **7** | **61 540** | **17,9** | ← *Fine* |
| 8 | 91 710 | 46,0 | |
| **9** | **130 914** | **119,5** | ← *Max* |

⭐ **Os triângulos SATURAM e o relógio não.** De 4 para 6 a contagem quadruplica por degrau; de 7
para 9 ela só duplica, enquanto o tempo multiplica por **6,7**. A eficiência cai de **1 861**
triângulos/ms no degrau 5 para **1 096** no 9 — a superfície é finita, e a partir de certo ponto
paga-se tempo por pouco detalhe novo. ⛔ Acima de 9 não há degrau que compense, e *um nível que
ninguém escolheria não é um nível*.

### Os três são AÇÕES, não um modo guardado

Um seletor de qualidade guardado obrigaria o artista a lembrar em que ficou — e a resposta certa
está na peça que ele tem à frente, não numa preferência de ontem. O rótulo diz o nível; o **toast**
diz o que saiu de facto (triângulos, KB, ms), porque o número depende da peça e prometê-lo no botão
seria uma promessa que só o resultado pode fazer.

### ⚠️ Nada de segunda tabela de formatos

O diálogo, os três formatos (OBJ · PLY · STL) e o aviso do que se perde vêm todos da
`ph2d_mesh::MeshFormat` e do `lost_by` que a **escultura** já tinha — promovido a `pub(crate)` em
vez de copiado. Uma cópia local diria *"cor preservada"* sobre um STL no dia em que alguém trocasse o
escritor, e **um aviso errado é pior que aviso nenhum**, porque o artista confia nele.

⭐ E vem com a lição que aquele módulo pagou: **um filtro por formato**, não um filtro único com as
três extensões — com o filtro único o diálogo nativo completa o nome com a **primeira** delas, e
`volta.ply` sai `volta.ply.obj`.

### Duas coisas de costura que a wave escolheu

| decisão | razão |
|---|---|
| `field3d_export` recebe os **toasts**, não o `App` | ela é chamada de dentro do quadro, onde o `gfx` já está emprestado — pedir `&mut self` ali é um empréstimo duplo. *Pedir só o que se usa* é o que a torna chamável de onde precisa |
| a pose da peça exportada é a **identidade** | o documento cozido já tem a cadeia inteira dentro do campo (`cook` compõe, `place` aplica): a malha sai em **mundo**. Uma pose ali aplicaria a transformação duas vezes |
| o pedido atravessa por um **canal próprio** | escrever um arquivo é assunto do app; a ponte com a cena recebe o mundo. É o mesmo caminho que o pedido de *abrir o painel* já usava — uma porta, dois pedintes |

### Provas de mutação

| Mutação | Reprovou |
|---|---|
| os três níveis dão a mesma malha | `the_export_levels_the_panel_offers_come_from_one_source` · `the_part_becomes_a_mesh_and_more_resolution_gives_more_of_it` |
| o botão não chega ao canal | `the_export_button_reaches_the_request_channel` |
| o pedido fica no canal em vez de ser tirado | `the_export_button_reaches_the_request_channel` |

⚠️ A terceira é a que mais importa e a menos óbvia: um pedido que ficasse no canal **reabriria o
diálogo em todo quadro seguinte**, e o artista não conseguiria fechá-lo.

⚠️ **A cwd do Bash voltou ao repositório primário a meio da medição** e a edição da sonda foi
aplicada na árvore errada — a memória do projeto nomeia exatamente isto. O sintoma foi um
`FileNotFoundError`, e não um número errado; se tivesse sido a segunda, a tabela acima estaria a
medir o `main`.

---

## §21 — W20: a malha exportada, e a quina que estava aberta desde a W0 (21/08)

> **O smoke da W19 aprovou a exportação e reprovou a MALHA.** O Enio abriu o `.obj` no Blender, deu
> *Shade Smooth* e viu manchas escuras num reticulado regular; de perto, triângulos sobrepostos. E
> perguntou o que era a decisão certa: **esperar pelo *quad remesh*** que a linha do sculpt está a
> escrever, **ou consertar já**.
>
> ⭐ **A resposta veio da medição, e ela inverte a pergunta: um remesh a jusante não cura NADA disto
> — ele herda a entrada.** Um remesh consome uma malha; faces dobradas e arestas não-manifold entram
> nele e saem dele. A entrada tem de estar limpa **antes**, e por isso esperar não era uma opção,
> era um adiamento.

### §21.1 — O diagnóstico, e ele separou dois defeitos que pareciam um

Um relatório de artista (*"baixa qualidade, sobreposição de faces"*) não é um mecanismo. A sonda
`quality::measure_export_mesh_quality` transforma-o num, porque **o remédio depende de qual defeito
é**. Sobre a cena 1 (junção de três cilindros com filete), extrator da `fidget`:

| prof | tris | q<0,10 | q_min | **invertidas** | % | grandes | área média | ℓ máx | cos pior | arestas ruins | dups |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 5 | 6 888 | 68 | 0,0186 | **96** | 1,4 | 96 | 0,238 cél² | 2,00 | −1,000 | 0 | 0 |
| 6 | 27 716 | 768 | 0,0282 | **364** | 1,3 | 364 | 0,174 | 2,06 | −1,000 | 0 | 0 |
| 7 | 61 540 | 2 660 | 0,0076 | **2 161** | 3,5 | 1 584 | 0,146 | **4,11** | −1,000 | 0 | 0 |

**A topologia estava perfeita e a malha estava sobre a superfície** (0 arestas ruins, 0 vértices
repetidos, erro ≤ 0,07 de célula). O defeito era um só e era grande: **3,5 % dos triângulos com a
normal ao contrário**, com área média de 0,15 célula² — não lascas invisíveis, faces inteiras. É
exatamente o que *Shade Smooth* mostra como mancha.

⚠️ **`ℓ máx = 4,11 células` é o mecanismo.** Em dual contouring, dois vértices duais vizinhos ficam a
~1 célula um do outro. Um triângulo que mede 4 células diz que **um vértice fugiu da sua célula** —
e o código da `fidget` confirma: `qef.rs::solve` promete *"increase the **likelihood** that the
vertex is bounded in the cell"* (uma probabilidade), e o `bounds.contains` só existe no caminho de
**colapso**, nunca na folha. Quando o vértice foge, o leque de quatro triângulos em torno da
interseção da aresta enrola uma ou duas delas ao contrário.

### §21.2 — O extrator da casa: `ph2d_field_eval::extract`

Dual contouring sobre **grade uniforme**, saída em **quads**. Três propriedades, e cada uma existe
por uma medição:

1. **Um vértice por célula, PRESO à célula.** Apaga a face dobrada por construção, em vez de a
   consertar a jusante.
2. **Recuo de 1 % da parede** (`CELL_INSET`). Não é folga de segurança: torna as caixas de duas
   células **disjuntas**, e é isso que impede dois vértices vizinhos de coincidirem. Sem ele, as
   cenas 4 e 5 nasciam com 38–156 triângulos de área **exatamente** zero e 15–55 vértices repetidos.
3. **Quads de verdade** (`ph2d_mesh::Face::quad`), valência 4 quase em toda a parte — a topologia que
   subdivide bem e que um *remesh* consegue comer.

⚠️ **A diagonal do quad é escolhida por PONTUAÇÃO, e duas regras mais simples foram refutadas.** O
consumidor parte todo quad por `a–c` (`Face::tri_at`, a única resposta da casa, e ela **não** se
toca) — então quem escreve o quad escolhe a diagonal ao **girá-lo**:

| regra | resultado |
|---|---|
| a diagonal mais **curta** | ⛔ nas 32 faces dobradas do fundo do vaso ela escolhia justamente a de fora (0,0113 contra 0,0175) |
| `n₁ · n₂ < 0` (as metades discordam) | ⛔ num quad em **sela** dispara nas duas diagonais e a regra não sabe qual está errada — foi assim que o copo do torno apareceu com 32 faces dobradas ao entrar na fixture |
| ✅ `min(n̂ᵢ · N̂)` contra a normal **do quad** (Newell) | responde às duas: a normal do quad é a média, e quem discorda dela é quem está virado. Empate (quad plano) → a mais curta, que dá triângulos menos finos |

### §21.3 — ⭐ A quina viva: o kill-criterion da W0 estava aberto, e o culpado era `sqrt(0)`

A W0 mediu, e o achado dela era um **mecanismo**, não um sintoma: *o desvio da aresta é igual à
fração de célula em que a face cai* — 0,10 → 0,10 · 0,50 → 0,50 · 0,80 → 0,80, com **0/49** faixas
capturadas. A hipótese registada era o leque da `fidget`.

**Quatro medições depois, três hipóteses estão refutadas e a quarta é a certa:**

| hipótese | veredito | como se soube |
|---|---|---|
| o **leque** da `fidget` quantiza | ⛔ **REFUTADA** | o extrator da casa não tem leque nenhum e reproduzia o **mesmo** desvio, dígito a dígito |
| a **interpolação linear** da travessia | ⛔ **REFUTADA** | 10 bisseções antes de interpolar não mexeram **um dígito** na tabela — e pioraram a esfera em 25 %, porque empurram `f` para dentro do ruído do `f32` |
| perguntar a normal **sobre** a superfície | ⛔ **REFUTADA** | afastar o ponto 1/1000 de aresta para dentro não mexeu no 5º dígito |
| ✅ **`sqrt` tem derivada infinita em zero** | **é esta** | `box_raw` é `length3(max(q,0)…)`: dentro da peça **inteira** os três termos são zero, o gradiente automático é `NaN`, a célula fica sem QEF e o vértice cai no **baricentro das travessias** — que é literalmente `0,72 × fração de célula` |

A cura é do **campo**, não do extrator: `ops::safe_sqrt` põe um piso de `1e-30` no argumento
(`sqrt(1e-30) = 1e-15`, oito ordens de grandeza abaixo do ULP de um `f32` de ordem 1 — o valor não
muda num bit, e a derivada passa a ser a de uma constante, zero e **finita**). ⛔ `sqrt(s + ε)` não
serve: muda o valor em `√ε` em toda a parte, e um raio de filete deixaria de ser o pedido.

**O resultado, no mesmo formato da W0 §2.1** (`probe_sharp_edge_capture`):

| meia-aresta | prof | célula | face em células | fração | desvio médio | pior | capturadas |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0,5 | 6 | 0,03125 | 16,00 | 0,00 | ~~0,72~~ → **0,01** | 0,01 | ~~0/32~~ → **32/32** |
| 0,45 | 6 | 0,03125 | 14,40 | 0,40 | ~~0,29~~ → **0,00** | 0,00 | ~~0/29~~ → **29/29** |
| 0,4609375 | 6 | 0,03125 | 14,75 | 0,75 | ~~0,54~~ → **0,00** | 0,00 | ~~0/30~~ → **30/30** |
| 0,45 | 7 | 0,01562 | 28,80 | 0,80 | ~~0,57~~ → **0,00** | 0,00 | ~~0/58~~ → **58/58** |
| 0,45 | 8 | 0,00781 | 57,60 | 0,60 | ~~0,44~~ → **0,00** | 0,00 | ~~0/116~~ → **116/116** |

*O cubo sai com o fio exato.* E `sqrt(0)` era também a razão de o `sd_profile` não ter gradiente
**sobre o próprio contorno** — as cinco raízes de soma de quadrados da crate passaram todas por
`safe_sqrt`, que é a única resposta da casa a essa pergunta.

### §21.4 — O corte de posto do QEF: o `1e-3` era da `fidget`, e é o primeiro degrau que reprova

`QEF_RANK_CUTOFF` decide abaixo de que fração do maior autovalor uma direção é ignorada. Varrido nas
seis fixtures, profundidades 6 e 7:

| corte | faces dobradas (total) | aresta viva | erro médio da esfera (prof. 5) |
|---|---:|---|---:|
| 1e-1 | **0** | 116/116, desvio 0,00 | 9,010e-4 |
| **3e-2** ← escolhido | **0** | 116/116, desvio 0,00 | 9,010e-4 |
| 1e-2 | **0** | 116/116, desvio 0,00 | 9,010e-4 |
| 1e-3 (o valor da `fidget`) | 32 | 116/116, desvio 0,00 | 9,353e-4 |
| 1e-4 | 328 | 116/116, desvio 0,00 | — |
| 1e-6 | 624 | 116/116, desvio 0,00 | — |

⚠️ **A coluna da aresta viva não se mexe** — a suspeita de que este corte governasse a quina está
**refutada** pela própria tabela, e o erro da esfera até **melhora** com o corte maior: não há troca
a fazer. O patamar de zero vai de 1e-2 a 1e-1 e o valor fica no meio dele. ⛔ O `1e-3` anterior veio
do `qef.rs` da `fidget`, onde está escrito *"somewhat arbitrarily"* — **valor de terceiro herdado
sem medição própria**, e o primeiro degrau a reprovar.

### §21.5 — ⭐ Um bug de CAMPO que a malha revelou: a costura do torno não é uma parede

A sonda mediu o campo do vaso (cena 5) e encontrou, **sobre o eixo, dentro do sólido**:
`f = −0,0000` com `‖∇f‖ = 0,000`, onde devia ler `−0,02 … −0,08`.

**O mecanismo:** um contorno desenhado tem de fechar, e num vaso ele fecha **descendo pelo eixo**.
Essa aresta existe no desenho e, ao girar, varre uma **linha** — medida zero, superfície nenhuma. Ela
é a costura do desenho, não uma parede da peça. Enquanto contava para a distância, havia um **nível
zero fantasma dentro do sólido**, e a extração encontrava-o e malhava-o.

A cura (`sd_profile_inner(.., axis_seam: true)`) tira da conta da **distância** as arestas cujos dois
extremos estão a menos da tolerância do perfil do eixo — e **só** da distância: o enrolamento
continua a ver a aresta inteira, porque é ele que sabe o que é dentro, e abrir o contorno inverteria
o sinal de meia peça.

⚠️ **A peça traçada nunca mostrou isto** (o raio bate na parede externa primeiro). Quem o via era a
malha exportada. *Um defeito de campo pode ser invisível no caminho que o artista usa para julgar.*

### §21.6 — O resultado, lado a lado

`measure_export_mesh_quality`, quatro cenas, quatro profundidades. Todas as colunas de topologia
(área zero, arestas ruins, vértices repetidos) são **0** em todas as linhas, nas duas colunas de
quads é **100 %**, e o que sobra é:

| cena | prof | quads | **invertidas (fidget)** | **invertidas (casa)** | q_min (casa) | fora da superfície |
|---|---:|---:|---:|---:|---:|---:|
| 1 — junção com filete | 5 | 1 830 | 96 | **0** | 0,585 | 0,042 cél |
| | 6 | 7 446 | 364 | **0** | 0,532 | 0,053 |
| | 7 | 29 502 | 2 161 | **0** | 0,695 | 0,021 |
| | 8 | 117 198 | — | **0** | 0,678 | 0,016 |
| 2 — cubo arredondado | 5 | 1 326 | 0 | **0** | 0,725 | 0,073 |
| | 6 | 5 022 | 18 | **0** | 0,789 | 0,023 |
| | 7 | 19 422 | 144 | **0** | 0,767 | 0,016 |
| | 8 | 78 846 | — | **0** | 0,714 | 0,009 |
| 4 — cantoneira desenhada | 5 | 406 | — | 6 ⚠️ | 0,093 | 0,373 |
| | 6 | 1 644 | — | **0** | 0,093 | 0,095 |
| | 7 | 6 238 | — | **0** | 0,442 | 0,112 |
| | 8 | 25 260 | — | **0** | 0,517 | 0,050 |
| 5 — torno (vaso oco) | 5 | 1 146 | — | 68 ⚠️ | 0,068 | 0,437 |
| | 6 | 4 514 | — | **0** | 0,025 | 0,363 |
| | 7 | 17 994 | — | **0** | 0,022 | 0,154 |
| | 8 | 71 814 | — | **0** | 0,001 | 0,201 |

⚠️ **As duas linhas com ⚠️ são a profundidade 5, e o motivo não é o extrator: é a grade.** A célula
mede 0,0625 e a parede do vaso mede 0,06 — *uma grade não representa uma feição mais fina que a
própria célula*, e a mesma linha traz **40 arestas não-manifold**, que é a assinatura disso. Foi esta
medição que subiu o **Draft de 5 para 6**.

### §21.7 — O custo, e a troca que ele representa

`measure_export_resolution`, cena 1, release:

| prof | quads | triângulos | ms |
|---:|---:|---:|---:|
| 4 | 438 | 876 | 0,8 |
| 5 | 1 830 | 3 660 | 1,9 |
| **6** | **7 446** | **14 892** | **7,9** ← Draft |
| **7** | **29 502** | **59 004** | **35,5** ← Fine |
| 8 | 117 198 | 234 396 | 214,6 |
| **9** | **467 334** | **934 668** | **1 463** ← Max |

⭐ **A contagem quadruplica a cada degrau e o relógio segue.** Isto é diferente do que a tabela da
W19 dizia — lá os triângulos **saturavam** (91 710 → 130 914 do degrau 8 para o 9), porque o octree
da `fidget` colapsa células e `depth` era um **teto**, não uma resolução. Aqui os três níveis são o
que dizem ser, e a esfera prova-o: 416 → 1 760 → 6 920 → 27 824 → 111 080 vértices (×4,2 · ×3,9 ·
×4,0 · ×4,0), com o erro médio a cair **por 4** a cada degrau — segunda ordem, que é o que uma
superfície lisa deve dar.

O preço é que a grade é **uniforme**: a extração é ~1,8× mais lenta que a da `fidget` na mesma
profundidade nominal, e o Max leva 1,5 s. Para uma ação de clique com aviso, é aceitável; o eixo a
abrir quando não for é a **poda por aritmética de intervalos**, que a `fidget` já expõe e que a
secção *"o que ele NÃO é"* de `extract.rs` nomeia.

### §21.8 — ⛔ Recusas MEDIDAS desta wave

| o que | por que não | onde |
|---|---|---|
| esperar pelo *quad remesh* da linha do sculpt | um remesh **herda** a entrada; face dobrada e aresta não-manifold passam por ele | §21 (topo) |
| bisseção da travessia antes de interpolar | não mexe um dígito na quina e **piora a esfera em 25 %** (empurra `f` para o ruído do `f32`) | §21.3 |
| perguntar a normal deslocada para dentro | não mexe no 5º dígito | §21.3 |
| diagonal do quad pela **mais curta** | escolhe a de fora nos quads côncavos | §21.2 |
| diagonal do quad por `n₁ · n₂ < 0` | dispara nas duas diagonais de um quad em sela | §21.2 |
| `QEF_RANK_CUTOFF = 1e-3` (o da `fidget`) | 32 faces dobradas; o patamar de zero começa em 1e-2 | §21.4 |
| `sqrt(s + ε)` em vez do piso | muda o valor em `√ε` em toda a parte; um raio de filete deixaria de ser o pedido | §21.3 |
| Draft na profundidade 5 | a parede do vaso é mais fina que a célula: 68 faces dobradas e 40 arestas não-manifold | §21.6 |

### §21.9 — ⚠️ O ORÁCULO foi o que reprovou primeiro, pela terceira vez nesta linha

O gate *"nenhuma face sai virada do avesso"* precisa de saber para onde a superfície olha, e as duas
primeiras respostas **reprovaram geometria correta**:

1. `∇f` no **baricentro** do triângulo — o baricentro não está sobre o nível zero; numa parede fina
   ele cai dentro do material e o gradiente aponta para a face **do outro lado**.
2. `∇f` na superfície mais próxima do baricentro (Newton) — cura a parede fina e **reprova a quina
   viva**: num canto de 90° a projeção pousa numa das duas faces, enquanto o quad que atravessa o
   canto tem a normal **entre** elas. Medido no copo do torno: 32 faces corretas lidas como dobradas,
   com `n̂ = (0, 0,94, 0,35)` contra `ĝ = (−0,33, 0, −0,94)` — dois vetores certos, de faces
   diferentes.
3. ✅ a **média das normais nos três vértices** — os vértices estão sobre a superfície por
   construção, e a média deles é justamente a direção "entre as faces". Uma face realmente invertida
   continua a dar `n̂ · ĝ ≈ −1`.

*Onde o campo não é liso, o oráculo é o que reprova primeiro.*

### §21.10 — Provas de mutação

| lei quebrada | gate que ficou vermelho |
|---|---|
| `sqrt` sem o piso (`safe_sqrt` → `sqrt`) | `a_live_edge_lands_on_the_edge_not_on_the_grid` |
| vértice do QEF sem a prisão à célula | `the_exported_mesh_never_folds_a_face` |
| `CELL_INSET = 0` (prisão até à parede) | `the_exported_mesh_is_a_watertight_quad_grid` |
| diagonal sempre `a–c` | `the_exported_mesh_never_folds_a_face` |
| costura do torno de volta à conta da distância | `the_seam_of_a_lathe_lies_on_the_axis_and_is_not_a_wall` |

⚠️ **A fixture do gate das dobras tem SEIS peças, e duas delas entraram por causa desta tabela**: com
só primitivas e booleanas, mutar a regra da diagonal passa **verde** — os quads só ficam côncavos nas
peças de **perfil desenhado**. *É a quinta vez que esta linha vê uma mutação passar verde porque a
fixture não continha o fenómeno.*

### §21.11 — O que a `fidget` deixou de fazer

A feature `mesh` saiu do `Cargo.toml`. Ela continua a fazer o que faz melhor que ninguém — **avaliar**
o campo, em lote e com JIT —, e o `jit` fica ligado pela mesma medição de sempre.

---

## §22 — W21: a ponte da escultura, e o achado que o plano não previa (21/08)

> Plano W5: *"malha esculpida → campo (via `ph2d-sdf`) → entra na booleana"*. A metade da **saída**
> fechou na W19/W20; esta é a **entrada**.

### §22.1 — ⛔ Uma malha NÃO pode virar uma árvore de avaliação

`fidget::context::TreeOp` é uma álgebra **fechada**: `Input(Var)` · `Const` · `Binary` · `Unary` ·
remapeamentos. **Não há operação de consulta a dados.** Um campo em voxels não é exprimível ali.

| caminho | veredito |
|---|---|
| a malha vira **expressão** (um termo por triângulo) | ⛔ ~10 nós por triângulo — meio milhão de nós para uma escultura média, avaliados milhões de vezes por quadro |
| a booleana acontece na **malha** | ⛔ é exatamente o que falha, e a tese do módulo é que não falha |
| ⭐ folha **amostrada** dentro da árvore de operações | ✅ **1,39×** por ponto |

O número que decide (`measure_sculpt_to_field_bridge`): **uma amostra trilinear custa 1,39× uma
avaliação da árvore com JIT** — 7,6 ms contra 5,5 ms por milhão de pontos. Misturar uma escultura
custa aproximadamente o mesmo que uma folha analítica a mais.

### §22.2 — ⛔ O campo do `ph2d-sdf` é uma BANDA ESTREITA

A distância só é escrita nas células dentro da caixa de algum triângulo; o resto fica em `±INFINITY`,
que o amostrador de lá doma para a **diagonal da grade**. Está certo para os consumidores dele —
oclusão, espessura e remesh só perguntam **junto** da superfície — e é **fatal** para uma marcha de
esfera: um passo do tamanho da diagonal salta a peça inteira. Medido: erro de **89 células** contra a
esfera analítica.

⭐ **Cura: propagação de chanfro** (duas varreduras, pesos `1 / √2 / √3`), na crate-ponte. Ela anda
**pela grade**, logo **superestima** — e uma marcha só é correta contra um **minorante**:

| resolução | célula | maior razão `chanfro / verdadeiro` |
|---:|---:|---:|
| 48 | 0,02500 | 1,1151 |
| 96 | 0,01250 | **1,1174** |
| 192 | 0,00625 | 1,1000 |

⛔ **A primeira volta escreveu `CHAMFER_SAFETY = 1,05` por palpite.** A sonda mediu **1,1174** — o
campo teria superestimado 5 % e a marcha atravessaria a peça. O valor é **1,15** (o limite teórico da
métrica em 3D é ~1,14). *Um número de segurança que não veio de uma tabela é o defeito que ele diz
prevenir.*

Mais: a caixa cresce **8 células** além da malha (`PAD_CELLS`, com a resolução a crescer junto para o
passo não mudar). Sem isso a grade encosta na malha com 1,51 células, e **fora dela a resposta é a
distância à CAIXA** — seguro para a marcha, errado para um **filete**, que acontece até um raio para
fora da peça.

### §22.3 — O avaliador HÍBRIDO, e a fusão que protege o caminho rápido

`ph2d_field_eval::hybrid`: o documento compila para uma árvore de operações cujas folhas são **ou**
uma fita de JIT **ou** um campo amostrado. ⭐ **Um `Combine` cujos filhos são todos analíticos volta a
ser analítico** — logo um documento sem escultura produz **uma** fita e o caminho rápido não muda um
bit. O gate `an_all_analytic_document_stays_one_tape` prende isso, e a mutação que desliga a fusão
põe-no vermelho: sem ele o traçado ficaria várias vezes mais lento **sem um único teste vermelho**,
porque o resultado continuaria certo.

⚠️ **A booleana existe agora DUAS vezes** — como árvore (`ops`) e como aritmética `f32` (`hybrid`).
Não há como fugir: um `min` entre uma fita de JIT e uma grade de voxels não cabe dentro de nenhuma
das duas. O que torna isso seguro é o gate de **paridade**: a mesma peça montada com a esfera como
primitiva e como escultura, comparada ponto a ponto, nos **três** operadores × **três** caracteres de
mistura. *Dois motores, uma lei — e a lei tem juiz.*

⚠️ **O gradiente**: um campo amostrado não tem gradiente analítico. Quando há escultura, a normal sai
por **diferença central**; quando não há, o caminho **exato** fica — e isso é load-bearing, porque a
diferença central numa quina viva devolve a média dos dois lados e desfaria o achado da W20 (§21.3).

### §22.4 — O documento: `NodeKind::Sampled { key }`

⚠️ **Um `NodeKind`, e não uma `Primitive`.** Uma primitiva é uma forma com **números**, e o painel
deriva as linhas dela; uma escultura não tem números. E o documento guarda o **NOME**, nunca a grade:
128³ pesa 12 MB, o documento é cozido **por quadro**, e um projeto guardado tem de **regenerar** a
grade da malha, que é a fonte.

⚠️ **Um nome desconhecido lê como espaço VAZIO**, nunca como sólido — numa união some, numa subtração
não corta. O oposto encheria a cena de um bloco que ninguém autorizou.

⛔ **Modificadores sobre uma escultura são RECUSADOS** (`FieldError::ModsOnSampled`), e recusar é a
resposta honesta: aplicá-los exigiria a casca, a matriz e a inclinação escritas uma segunda vez em
números, cada uma com o seu gate de paridade. Deixá-los passar daria um botão que não faz nada, que é
o modo de falha que nenhum smoke apanha.

`FIELD_DOC_VERSION` 3 → **4**.

### §22.5 — ⛔ Recusas MEDIDAS e provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| sem a propagação de chanfro | `the_sampled_field_marches_like_a_distance` |
| `CHAMFER_SAFETY = 1,0` | `inside_the_box_the_field_never_overshoots` |
| fora da caixa superestima | `outside_the_box_the_distance_never_overshoots` |
| `PAD_CELLS = 0` | `the_box_leaves_room_for_a_fillet` |
| De Morgan da diferença trocado | `the_numeric_law_is_the_same_law_as_the_tree` |
| sinal do filete exato trocado | `the_numeric_law_is_the_same_law_as_the_tree` |
| fusão analítica desligada | `an_all_analytic_document_stays_one_tape` |
| ausente lê como sólido | `an_unknown_sculpture_reads_as_empty_space` |
| sem a 2ª metade da pose (`× escala`) | `the_pose_of_a_sculpture_is_undone_on_the_sample` |

⭐ **E uma mutação passou VERDE, o que é o achado**: a barreira que impedia a propagação de chanfro
de atravessar a superfície era **inerte**. O motivo é aritmética — `|d|` é 1-Lipschitz **inclusive
através do nível zero** —, então um caminho que atravessa a parede continua a dar um majorante
válido; a barreira só **removia caminhos**, e numa parede fina deixava o meio dela sem vizinho útil.
Removida. *Uma precaução que a medição diz não fazer nada não é grátis: ela é a próxima pessoa a
acreditar que ela faz.*

### §22.6 — ⛔ O smoke REPROVOU: um cubo furado, e a costura entre os dois regimes

> Enio, 21/08: *"a cena que vc criou mostra um objeto texturizado dentro de um cubo furado, com
> artefatos de imagem, lento e bastante estranho"*.

⭐ **O campo estava certo. A costura entre os dois regimes é que não estava.** A sonda mediu, ao
longo de `+x`: `−0,445` no centro · `+0,000` em 0,54 (a casca) · `+0,088` em 0,66 — sinal certo,
distância certa, malha fechada (0 arestas ruins), 37,8 % de interior. Nada disso é o defeito.

**O defeito é um passo depois.** Junto da parede da caixa, por dentro, o campo valia **+0,10**; um
passo para fora, o `at()` devolvia a distância à caixa, que **nasce em zero na própria parede**.

⚠️ **Um campo que cai a zero numa superfície TEM uma superfície ali.** A marcha encontrava-a e
parava: a caixa da grade virava um cubo sólido, e a peça só aparecia por dentro do furo que a
booleana abrira.

A cura é `max(distância à caixa, valor na parede)`: na parede vale exatamente o valor de dentro
(**contínuo**), longe vale a distância à caixa (**cresce**), e continua a ser um minorante — para `p`
fora, o ponto mais próximo da caixa fica **entre** `p` e qualquer ponto da malha, logo
`d(p) ≥ d(parede)`.

| | pixels de peça (640×480) | fração do quadro | relógio |
|---|---:|---:|---:|
| com o cubo falso | 215 921 | **70,3 %** | 20,0 ms |
| curado | 128 608 | **41,9 %** | 23,1 ms |

⚠️ **O cubo era mais RÁPIDO** — os raios paravam mais cedo, na parede —, e é por isso que o relógio
não serve de gate: quem separa os dois casos é a **área**.

#### ⚠️ Os sete gates da ponte passavam sobre o código quebrado

Um media o regime de **dentro**, outro o de **fora**, e cada um estava certo no seu lado. Nenhum
media a **emenda**. O gate que faltava —
`there_is_no_false_surface_at_the_wall_of_the_box` — atravessa as seis paredes em passos de meia
célula e exige duas coisas: continuidade (nenhum salto maior que uma célula) e um piso (o campo nunca
chega perto de zero ali). Mutado o `at()` de volta, ele é o **único** dos oito a ficar vermelho.

*Dois regimes certos não fazem um campo certo. Quem mede um de cada vez nunca vê a emenda.*

#### ⛔ E a malha da cena estava errada por outro motivo

A `ph2d_mesh::shapes::uv_sphere_noisy` desloca cada vértice por ruído **branco** — ela existe para os
gates da escultura medirem malha irregular, não para alguém a olhar, e o resultado é **espinhoso**
(*"bastante estranho"*). A cena passa a construir a bolha com uma onda **suave**
(`1 + 0,16·sin(3·azimute)·sin(2·polar)`): forma grande e lisa, que é o que torna visível o que a cena
existe para mostrar — o filete a acompanhar a curvatura da peça.

### §22.6-ter — ⛔ A 2ª volta: uma FACE SOLTA, e ela era a mesma parede pelo outro lado

> Enio, 21/08 (2ª volta): *"temos uma face solta, com artefatos de imagem, quando vista por trás
> aparecem apenas pontinhos"*.

O `max(distância à caixa, valor na parede)` estava certo — e **não podia funcionar**, porque o valor
de parede que ele recebia já era **zero**. Dentro do amostrador havia uma **guarda**: se o índice
caísse fora do alcance, ele desistia e devolvia a distância à caixa. Parece defensivo e é o
contrário: os dois chamadores garantem que o ponto está na caixa, então cair fora só pode ser
**arredondamento** — e a resposta dela nesse caso era **zero na própria parede**.

Medido, campo da bolha ao longo de `+x` (caixa até `0,5943`):

| x | 0,55 | 0,57 | 0,59 | **0,61** | 0,63 |
|---|---:|---:|---:|---:|---:|
| com o defeito | +0,044 | +0,061 | +0,079 | **+0,016** ⛔ | +0,036 |
| curado | +0,044 | +0,061 | +0,079 | **+0,082** | +0,082 |

⚠️ **A esfera dos gates não expunha nada**: ela tem caixa **simétrica**, e ali o índice da parede caía
exato. Foi a bolha — de caixa assimétrica — que empurrou o arredondamento para o outro lado. O gate
passou a **varrer a resolução** (60 a 79), porque é a combinação `dims`/`passo` que decide: com o
defeito, **4 das 20** reprovam (62, 68, 76, 77). *A guarda não era uma guarda: era um caminho
alternativo escondido.*

#### ⭐ E por que a MALHA exportada saía LIMPA enquanto a tela mostrava um plano

Porque os dois consumidores procuram coisas **diferentes**:

| consumidor | acha superfície onde |
|---|---|
| a extração | o campo **muda de sinal** |
| a marcha de raios | o campo fica **pequeno** |

O defeito punha um zero **na parede** sem que o sinal mudasse dos dois lados — uma folha de espessura
zero. A extração não via **nada** (nenhuma travessia; a sonda mediu caixa `±0,52` e todos os vértices
entre raio 0,2 e 0,6) e a marcha parava em cheio.

⚠️ *Uma sonda que extrai malha não substitui uma que marcha.* A primeira medição desta volta — a que
contou vértices e caixas — devolveu **"está tudo limpo"** sobre o código que produzia a imagem que o
Enio tinha à frente.

#### A silhueta, e por que o relógio não serve de gate

| | pixels de peça (640×480) | fração do quadro | relógio |
|---|---:|---:|---:|
| **cubo** falso (a costura caía a zero) | 215 921 | **70,3 %** | 20,0 ms |
| **plano** falso (a parede lia zero) | 128 608 | **41,9 %** | 23,1 ms |
| curado | 80 581 | **26,2 %** | 25,0 ms |

⚠️ **Os dois defeitos eram mais RÁPIDOS que o certo** — os raios paravam mais cedo, na parede. Quem
separa os três casos é a **área**, e a barra do gate fica entre 26,2 % e 41,9 %.

### §22.6-bis — O smoke, e o que ele prova

`PH2D_FIELD_SMOKE=6`: uma **bolha orgânica de 8 192 triângulos** vira campo amostrado (112 de
resolução, ~171 ms) e leva um **furo de 0,20 com a boca arredondada em 0,05**. Traçar custa **25,0 ms
a 640×480**, contra 14,9 ms da cena 1 — **1,7×**, que é o preço de uma folha amostrada mais o
`CHAMFER_SAFETY` a encurtar cada passo. A malha é gerada na
cena de propósito: o que se prova aqui é a **ponte**, e uma escultura vinda do módulo de sculpt traria
consigo a pergunta de **autoria** — como o artista cria um destes —, que é wave própria e tem UI.

### §22.7 — ⏸️ O que fica aberto, nomeado

- ~~**A autoria**: não há gesto que crie um `Sampled` — nem botão, nem importação, nem ligação à
  escultura viva do módulo 3D. A cena 6 é o único sítio onde um existe.~~ ✅ **FECHOU na W22** (§23) —
  o botão `+ Sculpt…`. ⏸️ A ligação à escultura **viva** continua aberta.
- ~~**Persistência**: o `key` viaja no documento, mas nada regenera a grade ao carregar um
  projeto.~~ ✅ **FECHOU na W23** (§24) — a reconciliação mora no cozimento, e o que não voltar fala.
- **Modificadores** sobre uma escultura (recusados, §22.4) e **pose/modificadores de um `Combine`
  MISTO** (o `Combine` é avaliado na pose dele próprio).
- **A escala característica** de uma escultura é `INFINITY` no `subtree_scale` — o teto do slider de
  filete não a conhece. Só morde quando a escultura for a **única** peça sob o nó.

---

## §23 — W22: a AUTORIA — uma escultura entra pela porta (22/08)

> O motor da W21 fechou com um aviso escrito: *"não há gesto que crie um `Sampled`"*. A cena 6 do
> smoke fabricava a malha sozinha, e era o único sítio do app onde uma escultura existia como campo.
> *Uma porta sem corredor é código morto com a suíte verde.*

### §23.1 — O gesto, e onde ele coube sem plumbing

⭐ **A escultura é uma forma que se acrescenta** — então ela entra na lista `SHAPES`, ao lado da
caixa e da esfera, e o painel segue **sem uma linha de mudança** (a fileira é derivada da lista, a
mesma lei do `Mode::ALL` e do `ExportLevel::ALL`). O rótulo leva reticências (`+ Sculpt…`), que é a
convenção de *isto abre um diálogo*: as outras quatro criam na hora.

⚠️ **São TRÊS saltos, e o motivo é o mundo**: quem tem o `&mut World` (a ponte com a cena) não pode
abrir um diálogo, e quem abre o diálogo (o app) não tem o mundo.

| salto | onde | o que faz |
|---|---|---|
| 1 | ponte com a cena | o intent do botão **anota** o pedido |
| 2 | app | diálogo · lê o arquivo · constrói o campo · **regista** · anota o nome |
| 3 | ponte com a cena | o nome vira **nó**, com a escala do enquadramento |

É a mesma divisão que a exportação já fazia, pela mesma razão. O atraso é de um quadro.

⚠️ **Nada de segundo leitor de malha**: os três formatos vêm do `sculpt3d::import::read_pieces`, que
a escultura já tinha com os gates dela. Uma cópia local diria uma coisa e a original outra no dia em
que qualquer um dos três parsers mudasse.

### §23.2 — ⭐ A chave é o CAMINHO, e é isso que torna a persistência possível

`NodeKind::Sampled { key }` guarda o caminho do arquivo. O documento continua a pesar bytes em vez de
megabytes, e um projeto carregado sabe **de onde regenerar** cada escultura. ⏸️ Regenerar ao carregar
continua por fazer — mas deixou de precisar de um desenho novo.

### §23.3 — As duas metades da pose, e o que NÃO é reescrito

| o quê | onde vive | porquê |
|---|---|---|
| o **centro** | reescrito na malha (`recenter`) | a caixa da grade nasce da caixa da malha: uma peça longe da origem paga uma grade quase toda vazia |
| o **tamanho** | na **pose do nó** | o campo é construído nas unidades do autor — é isso que faz a célula da grade ser a resolução real do arquivo —, e um clique desfaz a pose |

`FRAMING_FRACTION = 0,5`: a peça nasce com metade do enquadramento. ⚠️ Sem isso um arquivo de 300
unidades ao lado de uma caixa de 1 não aparece grande — ele aparece **como nada**, porque a câmera
enquadra a caixa, e o artista conclui que a importação falhou.

### §23.4 — ⛔ Duas provas de mutação que passaram VERDE, e o que cada uma ensinou

**(a) `SCULPT_SLOT` literal em vez de derivado.** Trocar `SHAPES.len() - 1` por `3` não punha nada a
vermelho — porque o gate da costura **empurra o intent com a própria constante**, e mutá-la muda a
produção e a entrada do teste ao mesmo tempo. *Um teste que lê a constante que testa não testa a
constante.* O gate novo mede a **relação**: `SHAPES[SCULPT_SLOT]` é a chave da escultura, `shape_at`
devolve `None` ali e `Some` em todos os outros.

**(b) A fixture do tamanho estava no ponto neutro.** A esfera de raio 0,4 dá extensão 0,8, e o
enquadramento por omissão também: a escala de convivência saía **exatamente 1**, e o gate passaria com
o código de escala apagado. Quem o denunciou foi o **controlo** (`assert!(escala ≠ 1)`) que estava no
próprio gate. *Uma peça que já cabe no quadro não prova que alguém a fez caber.*

### §23.5 — ⚠️ E uma mutação SOBREVIVEU a um timeout, duas vezes

O laço de mutação foi interrompido pelo teto de tempo do shell com a árvore **mutada**, e a
verificação por `diff` contra o backup apanhou-o — as duas vezes no mesmo arquivo. É a segunda vez
que esta linha paga o mesmo pedágio, e a lição não muda: **uma mutação que não é desfeita é uma
mutação que shipa**. A cura que funcionou: filtro de teste **estreito** (o nome do gate, não `field3d`
inteiro), `timeout=` no subprocesso, `finally` a restaurar, e um `diff` explícito no fim.

### §23.6 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| o botão da escultura cai no `shape_at` | `the_sculpt_button_asks_for_a_file_instead_of_making_a_shape` |
| a escultura vira uma primitiva | `a_loaded_sculpture_becomes_a_node_the_evaluator_resolves` |
| sem a escala de convivência | `the_imported_sculpture_arrives_at_the_framing_size` |
| `SCULPT_SLOT` literal | `the_sculpt_slot_points_at_the_sculpt_button` |
| escala fixa, sem enquadramento | `the_size_follows_the_camera_not_a_constant` |

---

## §24 — W23: o REGRESSO — a escultura volta com o arquivo (22/08)

> A W22 fechou com uma linha na lista de aberto: *"regenerar a grade ao carregar continua por fazer"*.
> Ela lia-se como acabamento e era um **buraco de correção**: o projeto salvava a peça, reabria com a
> Hierarquia certa — e a escultura **não estava lá**, sem uma palavra.

### §24.1 — O defeito tinha três caras, e a pior não parece deste módulo

| onde a escultura estava | o que aparecia ao reabrir |
|---|---|
| sozinha, ou numa união | **nada** — o nó existe, a linha está na Hierarquia, a tela é vazia |
| como base de uma subtração | **nada**, pela mesma conta |
| numa **interseção** | ⚠️ **a peça INTEIRA some** — `max(a, ABSENT)` é `ABSENT` |

⭐ **A decisão de fundo está certa e é anterior a esta wave**: um nome que o registo não conhece lê
como espaço **vazio**, nunca como sólido ([`hybrid::ABSENT`](../../crates/ph2d-field-eval/src/hybrid.rs)) —
o oposto encheria a cena de um bloco que ninguém autorizou e que o artista não teria como apagar. O
que faltava é que **ler como vazio EM SILÊNCIO** é, para quem olha, indistinguível de o app ter
perdido o trabalho.

⚠️ **A premissa foi gateada, não assumida.** De uma escultura o documento guarda **uma coisa só**: o
caminho do arquivo, como `String` — enquanto todas as outras formas viajam como números. Se ele não
atravessasse o snapshot (ou atravessasse truncado), regenerar procuraria o arquivo errado e o sintoma
seria o mesmo da wave inteira. O gate `a_sculpture_crosses_the_snapshot_carrying_its_file_name` usa um
nome **com espaço e com acento**, que é o que um caminho de verdade tem.

### §24.2 — ⭐ A reconciliação mora no COZIMENTO, e não no load do projeto

O gancho óbvio era `project_load_from` (o Ctrl+O). Ele está errado por **alcance**: quem *nomeia* as
esculturas é o **documento**, e o documento nasce do `cook` da cena — que corre a cada quadro. Um
gancho no load cobriria o Ctrl+O e deixaria de fora tudo o resto que traz um nó de volta:

| caminho | passa pelo load? | passa pelo `cook`? |
|---|---|---|
| Ctrl+O | ✅ | ✅ |
| **undo de um apagar** | ❌ | ✅ |
| duplicar a subárvore | ❌ | ✅ |
| um documento semeado (smoke, cena nova) | ❌ | ✅ |

…e cada buraco teria o **mesmo sintoma mudo**. *Um invariante se impõe na derivação, não em cada
gesto* — a mesma lei que o `canonicalize()` do shell paga.

⚠️ Isto tem a vantagem lateral de manter o gancho **fora** de `project_load.rs`, que é arquivo
compartilhado com outras linhas.

### §24.3 — ⭐ Recarregar é a MESMA função que importar

`field3d_import::field_from_file` passou a ser a única resposta a *"que campo este arquivo dá"*, e o
diálogo ficou a ser o que sempre foi: um diálogo. ⚠️ **Uma segunda cópia — com outra resolução, ou sem
o `recenter` — daria uma peça que muda de forma ao reabrir o projeto**, e nada na tela o diria. O
documento guarda o caminho e não a grade, então *a função que lê o arquivo é parte do formato*.

⚠️ **A agulha do gate é o LEITOR, não o voxelizador**, e a diferença apareceu na primeira corrida
dele: a cena 6 do smoke também chama `SampledField::from_mesh`, sobre uma malha que ela **fabrica**.
Essa não pode divergir do arquivo — não há arquivo. O que não pode existir duas vezes é *ler um
arquivo de malha e chamar-lhe escultura*.

### §24.4 — O que não volta FALA — e fala UMA vez

O arquivo pode ter sido movido, renomeado, ou estar noutra máquina. A resposta continua a ser espaço
vazio (§24.1), mas com o **nome do arquivo** num aviso: `Sculpture blob.obj is missing: could not
read it (…)`.

⚠️ **A cena 6 do smoke é o caso-limite honesto**: a escultura dela é **fabricada** (a chave é `blob`,
não um caminho), então salvar e reabrir essa cena não a pode trazer de volta — e o aviso di-lo, em vez
de a peça sumir calada. *Uma escultura sem arquivo não tem de onde voltar.*

⚠️ **E uma tentativa por nome.** A reconciliação corre no cozimento, que corre a cada quadro, e um
caminho que falha **falha sempre** — o arquivo não volta sozinho. Sem o conjunto `TRIED` seriam 60
leituras falhadas e 60 avisos **por segundo**, e a tela ficaria ilegível exactamente no caso em que o
artista precisa de a ler.

### §24.5 — O custo, medido (`measure_the_cost_of_coming_back`)

**Regenerar** (o que se paga uma vez, ao abrir):

| triângulos | KB do `.obj` | ms (ler + merge + voxelizar) |
|---:|---:|---:|
| 288 | 14 | **274,3** |
| 2 048 | 109 | 255,5 |
| 8 192 | 452 | 409,5 |
| 32 768 | 1 919 | **468,3** |

⭐ **O custo é da GRADE, não da malha**, e a tabela di-lo sozinha: 288 triângulos já pagam 274 ms, e
**114× mais triângulos** custam **1,7×** mais tempo. A grade é 144³ (`DEFAULT_RESOLUTION` 128 mais
2×`PAD_CELLS`) ≈ 3,0 M células, e a propagação chanfrada varre 13 vizinhos nos dois sentidos. *Uma
escultura mais pesada não torna abrir o projeto mais lento; uma resolução maior torna.*

**Varrer** (o que se paga por quadro, no caso normal — nada em falta):

| nós no documento | µs por varredura |
|---:|---:|
| 1 | 0,007 |
| 8 | 0,008 |
| 64 | 0,026 |
| 512 | **0,162** |

0,162 µs é **0,001 %** de um quadro de 16,7 ms: o caso normal não toca no disco e não se mede.

⚠️ **O que isto NÃO diz:** a regeneração corre **dentro do quadro**, então abrir um projeto com uma
escultura tranca a janela por ~0,3 s (com quatro, ~1,5 s). É o mesmo preço que o import já cobra, e o
irmão da casa faz o mesmo (a escultura decodifica o documento dela no próprio `project_load`). ⏸️ **O
gatilho para o tirar do quadro é esse número** — várias esculturas, ou uma resolução maior —, não uma
preferência.

### §24.6 — Uma consequência que é produto: a escultura é um VÍNCULO, não uma cópia

Como o que se guarda é o **caminho** e o campo é reconstruído do arquivo, editar o `.obj` no
escultor e reabrir o projeto traz a versão **nova** para dentro da peça — a booleana e o filete
seguem-na. Isso não é acidente do desenho, é o desenho; e é a metade barata da ⏸️ *ligação à escultura
viva*, que continua aberta (hoje o vínculo passa pelo disco, e acorda ao abrir, não ao vivo).

### §24.7 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| a ponte não reconcilia | `a_project_that_reopens_gets_its_sculpture_back` (**vermelho antes de existir a costura** — red-first) |
| `first_try` sempre verdadeiro | `a_sculpture_whose_file_vanished_speaks_once` |
| `missing_keys` sem `dedup` | `missing_keys_names_exactly_what_the_registry_cannot_answer` |
| `missing_keys` ignora o que o registo já sabe | `missing_keys_names_exactly_what_the_registry_cannot_answer` |
| a ponte não **entrega** o aviso ao app | `the_missing_file_reaches_the_artist_through_the_bridge` |

⚠️ **O último gate nasceu de olhar para a lista antes de a correr**: `a_sculpture_whose_file…` chama
`resolve_missing` de frente e mede o que ela devolve — apagar a linha que **entrega** esse resultado
ao canal do app deixá-lo-ia verde e o artista mudo, que é o modo de falha desta wave inteira uma
emenda mais à frente. A última emenda (o canal → `Toast`) continua sem gate: é uma linha ao lado de
duas idênticas, no meio do quadro.

### §24.8 — ⏸️ O que fica aberto

- **Um arquivo que mudou de sítio não se reencontra** — a chave é o caminho absoluto, então mover a
  pasta (ou abrir noutra máquina) perde a escultura, com aviso. Religar pede UI (*"onde está este
  arquivo?"*), que é a mesma pergunta que o resto do app ainda não faz por nenhum asset.
- **A regeneração corre no quadro** (§24.5): o gatilho para a tirar de lá está escrito e é um número.
- Um nome que falhou **não é re-tentado** nesta sessão, mesmo que o arquivo reapareça. Re-importar
  pelo botão resolve, e é o gesto que o artista já tem.

---

## §25 — W24: o preview responde à mão — e a resolução sai do RELÓGIO (22/08)

> Enio, no smoke da cena 6: *"lento e bastante estranho"*. O **estranho** era um bug de campo e foi
> curado na hora (§22.6). O **lento** ficou por medir durante duas waves — e era o defeito maior dos
> dois.

### §25.1 — O que ele estava a ver, medido

Traçado no tamanho **real** de uma janela (release, máquina calma; `probe_scene_trace_cost`):

| cena | 640×480 | 1280×800 | **1920×1080** | 2560×1440 |
|---|---:|---:|---:|---:|
| 1 — três cilindros com filete | 15,0 ms | 25,7 | **39,2** | 58,8 |
| 2 — cubo arredondado | 10,5 | 20,5 | **33,1** | 47,5 |
| 6 — escultura com furo | 22,3 | 60,8 | **113,9** | 198,8 |

⚠️ **640×480 é um oitavo de uma janela de 1080p**: a sonda antiga media ali e respondia a uma
pergunta que ninguém faz. No tamanho a que o artista de facto usa o módulo, a imagem atualiza a
**21 fps** numa peça simples e a **8 fps** com uma escultura dentro.

⚠️ **A janela nunca trava** — o traçado corre noutra thread e a chrome continua a 60 fps. O que fica
lento é a **imagem**, que é a única coisa que diz onde a peça está.

### §25.2 — ⛔ Duas hipóteses medidas e REFUTADAS antes de escrever uma linha

**(a) "Há um custo fixo por traçado — a fita é recompilada a cada quadro."** A tabela sugeria-o (6,75×
os pixels custavam 2,6× o tempo, o que só se explica com uma parcela constante). `Hybrid::new` mede
**0,02–0,10 ms**, ou **0,2 %** do quadro. A sub-linearidade é outra coisa: o anti-serrilhado corre
sobre as **arestas**, e numa imagem pequena a fração de pixels de aresta é maior — o custo *por
pixel* **sobe** quando a imagem encolhe. Isso não é um custo fixo a eliminar; é a curva a conhecer.

**(b) "O piso do divisor é onde a forma começa a mudar."** A sonda `probe_how_coarse_a_preview_can_be`
mediu a deriva da silhueta até D=8 e ela **não existe** — **0,15 %** no pior caso. *A métrica não
contém o fenómeno*: a área é dominada pelas formas grandes, e o que uma resolução grosseira come é a
**feição fina**, que quase não pesa em área. Chamar àquilo uma medição do piso seria dressar um
palpite. O piso teve de vir de outro sítio (§25.4).

### §25.3 — ⭐ A lei: o divisor sai da MEDIÇÃO, e o laço fecha-se sozinho

Um traçado devolve quanto custou; dividir pelos pixels dá o custo por pixel **desta máquina, desta
peça, deste momento**. O pedido seguinte escolhe o maior tamanho cujo custo previsto cabe no
**orçamento de um quadro a 60 Hz** (16,7 ms — o número é do monitor, não uma preferência).

⚠️ **A previsão erra, e o laço não se importa.** Prever o grosso a partir do cheio é otimista (§25.2a);
a medição seguinte corrige. Convergência, com os números reais:

| cena | cheio | 1ª escolha | assenta em | custo final | ganho |
|---|---:|---:|---:|---:|---:|
| 1 | 46,0 ms | D=2 (17,8) | **D=3** | **11,0 ms** | **4,2×** |
| 6 | 121,0 ms | D=3 (16,6) | **D=3** | **16,6 ms** | **7,3×** |

Enquanto a mão mexe, a imagem passa de 21/8 fps para **60 fps** nas duas. É o mesmo motor — *não há
segundo avaliador*, e a recusa medida do plano continua de pé.

### §25.4 — ⚠️ O piso é o ORÇAMENTO, e o gate mede a RELAÇÃO

`MAX_PREVIEW_DIVISOR = 3` porque **a D=3 a cena mais pesada já cabe** (16,6 de 16,7 ms). Descer mais
não compra nada que o orçamento peça e custa nitidez. Se um dia uma peça não couber a D=3, o laço fica
**preso no piso e a imagem fica lenta em vez de virar papa** — a direção conservadora para um módulo
cuja razão de existir é a aresta.

E as duas metades da lei, porque uma sozinha é meio gate: uma peça **cara** tem de sair mais grossa, e
uma peça **barata nunca é suavizada** (o traçado cheio já cabe: baixar a resolução seria perder
nitidez de graça).

### §25.5 — O primeiro traçado é sempre CHEIO, e isso é produto

Sem medição não há previsão, e o primeiro traçado **é** a medição. O efeito: a primeira coisa que se
vê é a peça **nítida**; a suavização só aparece depois, em movimento, que é onde não se nota. *A
propriedade caiu da lei, não de um caso especial.*

### §25.6 — ⚠️ A costura que esta wave podia partir

O tamanho do traçado e o da área eram **o mesmo número** desde sempre, e o desenho projetava o gizmo
a partir do do traçado — correto por coincidência. Com o preview grosso os dois divergem: as alças
sairiam a um terço do tamanho e agarrariam longe da superfície **só durante o movimento** — o defeito
mais difícil de reproduzir que este módulo poderia ter. A projeção passou a ter **um dono**
(`field3d_input::area_screen`), e o gate proíbe o desenho de construir a dele.

### §25.7 — ⛔ Uma prova de mutação passou VERDE (a 2ª vez nesta linha)

Pôr `MAX_PREVIEW_DIVISOR` a 8 não punha nada a vermelho: o gate comparava o resultado **com a própria
constante**, então mutá-la movia a produção e a expectativa ao mesmo tempo. É o gémeo exacto do
`SCULPT_SLOT` da W22, e a cura é a mesma — medir a **relação** com a tabela medida: a `D` a cena mais
pesada cabe no orçamento, e a `D−1` **não** cabe. *Um teste que lê a constante que testa não testa a
constante.*

### §25.8 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| `preview_size` ignora a medição | `the_loop_settles_inside_the_budget_on_the_measured_scenes` |
| devolve sempre o piso | `a_cheap_piece_is_never_softened` |
| assentar não refina | `a_settled_view_refines_to_full_exactly_once` |
| refina a cada quadro | `a_settled_view_refines_to_full_exactly_once` |
| o piso sobe para 8 | `the_floor_is_the_shallowest_divisor_that_fits_the_budget` |
| o desenho projeta o gizmo sozinho | `the_draw_does_not_build_its_own_projection` |

### §25.9 — ⏸️ O que fica aberto

- **Um traçado em voo não se cancela**: se a mão recomeça a mexer no meio de um refinamento cheio, a
  resposta espera por ele — até **121 ms** medidos na cena mais pesada. A cura é um traçado
  cancelável (o worker teria de olhar uma bandeira por linha), e o gatilho é este número.
- O divisor **não aparece na tela**, de propósito (o artista vê a peça, não a régua). Quem quiser
  vê-lo tem `PH2D_FIELD_TRACE_LOG=1`.
- ⛔ **Um segundo motor em GPU continua recusado** e agora por mais uma razão: o que fazia falta era a
  **taxa de atualização em movimento**, e ela foi comprada sem duplicar o avaliador.

---

## §26 — W25: a peça que não cozinha DIZ porquê — e o clique que a partia deixou de existir (22/08)

> Um clique. Selecionar uma escultura, carregar em **Shell**, e a peça **inteira** desaparecia da
> tela — com a Hierarquia intacta e sem uma palavra. *Um erro engolido é pior do que um erro: ele
> parece um problema de câmera.*

### §26.1 — O mecanismo, e ele tinha três camadas todas «certas»

| camada | o que fazia | veredito |
|---|---|---|
| o **documento** | recusa modificadores sobre uma escultura (`ModsOnSampled`) | ✅ certo, e a regra estava escrita desde a W21 |
| o **mundo** | `add_mod` escrevia o componente sem perguntar | ⛔ a regra existia e ninguém a consultava |
| o **cozimento** | `cook(…).and_then(Result::ok)` | ⛔ **deitava o `Err` fora** |
| o **painel** | oferecia a fileira de modificadores para tudo | ⛔ oferecia o que não se pode fazer |

⭐ *Uma invariante que só o validador conhece é uma invariante que a UI descobre partindo-se.*

### §26.2 — A cura tem duas metades e as duas eram necessárias

**A porta recusa** (`add_mod` devolve `false` numa escultura, e o painel **não pinta** a fileira) — a
mesma lei que a fileira de operações já segue: *um controle que aparece e não faz nada é pior do que
um que não aparece*.

**E a voz existe para o resto**: o `Err` do cozimento passou a virar uma frase. ⚠️ **Isto não é sobre
este bug, é sobre a classe dele** — um projeto de uma versão anterior, um perfil desenhado a cruzar o
eixo, ou um escritor novo produzem o mesmo documento inválido, e o sintoma seria o mesmo
desaparecimento mudo.

⚠️ **`None` e `Err` são coisas diferentes e só uma é um problema**: apagar o último filho de uma peça
devolve `None` e é um gesto normal, cujo resultado normal é a tela ficar vazia.

### §26.3 — ⭐ O módulo passa a falar por UMA boca

A W23 abriu o primeiro canal de avisos (a escultura que não voltou do arquivo) e esta wave precisava
do segundo. Dois canais paralelos teriam **duas** leis de repetição, dois drenos e dois sítios onde
alguém se esquece de drenar — então há **um** ([`field3d_notice`](../../shells/desktop/src/field3d_notice.rs)),
com uma lei: **não repete a última coisa que disse**. O cozimento corre a cada quadro e uma peça
inválida continua inválida; sem isso seriam 60 avisos por segundo sobre a mesma frase. Uma frase
*diferente* passa sempre, e a peça voltar a ficar válida **esquece** a última — senão o mesmo
problema, se voltasse, ficaria mudo na segunda vez.

⚠️ **As frases são para o Enio**: dizem o que está errado na peça, nunca o nome da variante ou do nó.
E o `match` que as escolhe **não tem braço `_`**, de propósito: um erro novo no documento tem de
fazer aquele arquivo não compilar, que é o momento certo para escolher as palavras.

### §26.4 — ⚠️ E o achado da jornada não foi do módulo: era da FERRAMENTA de mutação

Uma corrida do gate novo saiu **vermelha sobre código correto**, e a causa não estava no código: o
laço de mutação restaurava o arquivo com `shutil.copy2`, que **preserva o mtime**. O cargo decide por
mtime — o arquivo restaurado ficava a parecer mais **velho** que o objeto compilado da mutação, e a
corrida seguinte servia a **build mutada**. O `rustfmt` costumava mascarar isto (ele reescreve o
arquivo e carimba a data), e mascarou-o em todas as waves anteriores; nesta o arquivo já estava
formatado, o `rustfmt` não lhe tocou, e o fantasma apareceu.

⛔ **A cura: `shutil.copy` + `os.utime(path, None)`.** E a lição é maior do que a ferramenta: *uma
prova de mutação mede o binário, não o arquivo* — se a restauração não for observável pelo sistema de
build, o verde e o vermelho seguintes valem zero. Este laço já tinha pago duas vezes o irmão deste
defeito (uma mutação sobreviver a um timeout); é a terceira vez que a **restauração** é onde o
método fura.

### §26.5 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| o cozimento volta a engolir o `Err` | `a_piece_that_cannot_cook_says_why` |
| o canal repete a última frase | `the_same_notice_is_not_said_twice_in_a_row` |
| o painel volta a oferecer a fileira | `the_panel_offers_no_modifiers_for_a_sculpture` |
| a porta volta a aceitar o modificador | `a_sculpture_refuses_a_modifier_and_the_world_is_left_alone` |
| a porta recusa **toda a gente** (cura larga demais) | o mesmo gate, pelo controle |

### §26.6 — ⏸️ O que fica aberto

- A frase aparece como aviso no canto. Uma peça inválida **também não desenha**, e a Hierarquia não
  marca **qual** nó é o culpado — o erro do documento traz o índice, e traduzi-lo de volta para a
  entidade é trabalho de outra wave.
- ⛔ **Os outros oito erros do documento não têm caminho conhecido a partir da UI** — as portas
  recusam antes. As frases existem para o dia em que tiverem, e o gate garante que nenhuma nasce
  vazia.

---

## §27 — W26: o número digitado no meio do gesto — o `G X 0,5` (22/08)

> A ficha do gesto **mostra** o número desde a W8 e nunca o **aceitou**: para pôr uma peça a 0,5
> exactos era preciso largar a alça e ir procurar a linha certa no painel. O painel continua a ser a
> porta para *"quanto ela mede"*; isto é a porta para *"anda exactamente isto, agora"* — outro gesto,
> e o do modelador.

### §27.1 — ⭐ A lei: o que se digita é o TOTAL, e a álgebra já existia

O arrasto deste módulo mede **o total desde a pegada** (`Grip::applied`, W6) e manda ao mundo
`total.since(applied)`. Um número digitado é simplesmente **outro total** — mesma álgebra, outra
fonte. É isso que faz digitar `0,5` depois de já ter arrastado `0,37` mandar `0,13` ao mundo, **sem
uma linha de caso especial** e sem o gesto saltar.

⚠️ O gate-mãe começa com o ponteiro a ter aplicado 0,37 de propósito: tratado como incremento, o
mundo receberia 0,87 — a peça parava onde ninguém pediu **com a ficha a dizer o número certo**.

### §27.2 — ⚠️ As três cedências que fazem isto funcionar

| quem cede | o quê | porquê |
|---|---|---|
| o **rato** | enquanto há um número aberto, o ponteiro não mexe na peça | o dedo nunca está parado: sem isto o quadro seguinte sobrescreve o que se acabou de escrever, e o defeito lê como *"digitar não faz nada"* |
| o **roteador de teclas** | a entrada numérica vem **antes** do `G`/`R`/`S` | com uma entrada aberta, um `5` é um cinco |
| a **entrada** | um `Backspace` que a esvazia **sai** dela | um campo vazio com o rato mudo prendia o gesto sem nada na tela a dizer porquê |

⚠️ **Impossível esta porta comer a tecla de outra pessoa:** para ela abrir é preciso ter o botão do
rato **em baixo, sobre uma alça do gizmo**. As guardas antigas (ponteiro sobre a janela, sem
`Ctrl`/`Alt`/`Super`) continuam todas.

### §27.3 — As unidades são as da FICHA

Unidades de mundo numa seta, **graus** numa argola, **fator** no punho — exactamente o que
[`field3d_gizmo_paint::readout`](../../shells/desktop/src/field3d_gizmo_paint.rs) já escrevia. Um
número que se digita em radianos e se lê em graus seria a segunda verdade clássica, invisível até
alguém medir a peça. ⛔ E um fator não-positivo não é um tamanho: o texto fica na tela, o mundo não
recebe nada.

### §27.4 — ⭐ `Esc` desfaz o gesto INTEIRO, e pela própria álgebra

O inverso do que já foi aplicado escreve-se `applied.neutral().since(applied)` — que dá `−d` numa
translação, `−θ` num giro e `1/f` num tamanho, sem uma segunda tabela. *Uma conta nova de «como se
desfaz um giro» divergiria da primeira no dia em que um verbo novo entrasse.*

### §27.5 — ⚠️ Só onde um número tem UM significado

As setas, as argolas e o punho aceitam; os **planos** e o **plano da tela** não — ali um número
sozinho não diz para onde. É a mesma razão pela qual o Blender pede um eixo antes do número, e a
tecla **passa adiante** em vez de ser engolida.

### §27.6 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| o número vira incremento | `a_typed_number_is_the_total_not_one_more_step` |
| o rato não cede | `the_pointer_gives_way_while_a_number_is_open` |
| `Esc` larga sem desfazer | `escape_puts_the_piece_back_where_it_was` |
| um plano aceita um número | `a_plane_handle_takes_no_number` |
| a argola lê radianos | `the_typed_number_speaks_the_units_of_the_readout` |
| a tecla é engolida em qualquer arrasto | `a_number_is_only_taken_while_a_handle_is_held` |

### §27.7 — ⏸️ O que fica aberto

- **Um eixo só se escolhe agarrando a alça dele.** O `X`/`Y`/`Z` do Blender (teclar o eixo *depois*
  de `G`) exigiria o gesto modal sem botão premido, que é outro modelo de interação — e não o que
  este módulo tem.
- A entrada **não faz contas** (`0.5*2`). O painel também não; é a mesma decisão, num sítio novo.

---

## §28 — W27: a SELEÇÃO é o sujeito do gesto — e o pivô é o meio dela (22/08)

> Escolher duas peças na Hierarquia acendia as duas, e arrastar movia **uma**. A fileira de operações
> contava com a seleção múltipla desde a **W9** (*"embrulhar irmãos numa nova operação"*); o gizmo
> nunca soube dela. *Duas linhas acesas, uma a andar, lê-se como o gizmo estar partido.*

### §28.1 — ⭐ A lei nova CONTÉM a antiga, e é isso que a torna segura

Rodar e escalar passaram a aceitar um **pivô**
([`rotate_world_about`](../../crates/ph2d-field-ecs/src/edit.rs) / `scale_about`) — e com o pivô em
cima da origem do nó, a parcela de translação sai **exactamente zero**: o resultado é byte-a-byte o
de antes. Não há um ramo *"se for um só"*; há uma lei mais geral cujo caso particular é a antiga, e
um gate (`with_one_node_the_law_is_the_old_one`) que o prende.

⚠️ **Orbitar é TRANSLADAR**, e é por isso que as duas se escrevem com as portas que já existiam — a
orientação por `rotate_world`, a posição por `translate_world`. Uma terceira conta de pose divergiria
das outras duas no dia em que a hierarquia mudasse de forma.

### §28.2 — ⭐ O PIVÔ SOBREVIVE AO GESTO QUE ELE APLICA

O pivô é recalculado a cada quadro a partir das origens (**uma** função, usada tanto pelo gizmo como
pelo gesto — duas contas de *"onde está o pivô"* fariam a peça girar em torno de um ponto que não é
aquele onde as argolas estão desenhadas). Mas o gesto **move as origens**.

A propriedade que salva o desenho: **rodar e escalar em torno do centróide preservam o centróide**.
Se ela não valesse, um arrasto contínuo descreveria uma **espiral** em vez de um arco — e o defeito só
apareceria num gesto longo. Ela é matemática, mas a implementação pode parti-la: por isso está
**medida** (`the_pivot_survives_the_motion_it_applies`, três gestos encadeados) e não escrita num
comentário.

### §28.3 — ⚠️ Um filho de outro escolhido NÃO anda duas vezes

O defeito clássico de mover uma seleção: com o grupo **e** uma peça dele acesos, a peça recebe o gesto
*e* herda o do grupo pela hierarquia — anda o dobro, e só ela. `ph2d_field_ecs::top_level` deixa
passar só o topo de cada ramo, **preservando a ordem de entrada** (quem chama depende dela para saber
quem é o principal).

⚠️ **Quem está agarrado entra sempre**, mesmo que a seleção do app já não o contenha: o gesto foi
começado nele e o `Grip` congelou-o.

### §28.4 — O que ficou como estava, de propósito

| | decisão |
|---|---|
| os **eixos** do gizmo | continuam a ser os do **principal** — é o que mantém o seletor Global/Local a significar alguma coisa numa seleção (o «Local» de um conjunto é o do objeto ativo, como em todo modelador) |
| o **pivô** | a média das **origens**, não o centro das caixas: a caixa de um campo implícito custa uma varredura, e o que o artista agarra é o que ele vê — as setas estão sobre as origens |
| a **identidade** do arrasto | a do principal: é ela que o `Grip` congela |

### §28.5 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| o gesto volta a mover só o principal (**o defeito**) | `a_drag_moves_every_selected_node` |
| o filho de um escolhido anda duas vezes | `a_child_of_a_selected_node_does_not_move_twice` |
| o giro ignora o pivô | `rotating_a_selection_swings_them_around_the_shared_pivot` |
| o tamanho ignora o pivô | `scaling_a_selection_spreads_them_from_the_shared_pivot` |
| o pivô passa a ser o do principal | `the_gizmo_sits_at_the_middle_of_the_selection` |
| rodar **um** objeto tira-o do sítio | `with_one_node_the_law_is_the_old_one` |

### §28.6 — ⚠️ Um pedágio de ambiente, e o diagnóstico enganava

Uma corrida de `cargo test -p ph2d-host-desktop <filtro>` morreu com `mold: failed to write to an
output file. Disk full?` e `clang: Bus error`. **O disco tinha 435 GB livres** — a mensagem é do
linker a falhar num pico de recursos ao ligar ~200 binários de teste de integração de uma vez. A cura
é `--bin ph2d-host-desktop`: os gates deste módulo vivem todos no binário de unidade, e compilar os
alvos de integração para os correr é trabalho que ninguém pediu. *É a irmã da lição já registada em
[`feedback_a_ship_x_can_be_the_environment_not_the_code`](../../project-memory/feedback_a_ship_x_can_be_the_environment_not_the_code.md):
um `✗` pode ser do ambiente — e a mensagem dele pode nomear o recurso errado.*

### §28.7 — ⏸️ O que fica aberto

- **O pivô continua a ser derivado**, nunca escolhido: o *cursor 3D* do Blender (pousar o pivô num
  ponto qualquer) é produto, e entra com a UI que o põe lá.
- A seleção múltipla **não tem gesto de canvas**: escolhe-se na Hierarquia com `Ctrl`. Um laço de
  seleção na janela 3D é wave própria.

---

## §29 — W28: o olho da Hierarquia apaga o nó da peça (22/08)

> Clicar no olho de um cilindro escrevia o componente, **acendia o ícone** — e a peça na tela ficava
> igual. Um controle pintado, despachado, e mudo. É a terceira wave seguida da mesma família (o
> modificador da W25, a seleção da W27), e a mais barata das três.

### §29.1 — ⭐ A composição já exprimia isto: zero componente novo

O app tem **uma** ideia de *escondido* — [`ph2d_ecs::Visibility`], o ícone do olho — e ela já é
escrita pela Hierarquia em **qualquer** entidade, já é persistida, já é desfeita, e já tem a lei da
casa: **a ausência é visível** (HR-5). O que faltava era **o cozimento perguntar**.

⚠️ **A alternativa que não foi construída:** um `FieldHidden` próprio. Ele teria de ser registado,
persistido, desfeito e mostrado — e daria ao app **duas** ideias de escondido, que divergiriam no
primeiro gesto que tocasse só numa. *Antes de construir um item de lista aberta, meça se a composição
já o exprime* (CLAUDE.md §5.0) — aqui ela exprimia, e a wave inteira é **uma linha** de produção mais
os gates.

### §29.2 — ⚠️ A recusa é na DESCIDA, e é isso que esconde a subárvore

A travessia nunca chega aos filhos de um nó escondido, e o pai vê um filho a menos — exactamente o
caminho de um nó apagado. Com a pergunta na **subida**, esconder um grupo deixaria os filhos dele
emitidos na arena como órfãos.

⚠️ **A fixture teve de mudar para conter o fenómeno:** com uma união rasa, *descida* e *subida* dão
o mesmo resultado e o gate passa com as duas. Com um grupo **aninhado**, a contagem separa-as —
**2 contra 4**. *Uma fixture só prova o que contém.*

### §29.3 — E o buraco que o olho abria, fechado no mesmo passo

Um nó escondido continuava a ter **setas** — e arrastá-las é um gesto **sem resposta na tela**, que é
a família de defeito que a wave veio fechar, um passo à frente. Agora: um nó escondido **não tem
gizmo** e **não anda com a seleção**. A linha da Hierarquia continua lá, e é por ela que se volta a
acender o olho.

### §29.4 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| o cozimento volta a ignorar o olho (**o defeito**) | `the_hierarchy_eye_takes_the_node_out_of_the_piece` |
| a pergunta muda para a subida (o grupo esconde-se, os filhos ficam) | o mesmo gate, **2 contra 4** |
| a ausência do componente conta como escondido | o mesmo gate |
| o escondido continua a andar / a ter setas | `a_hidden_node_has_no_gizmo_and_does_not_move_with_the_selection` |

### §29.5 — ⛔ A FERRAMENTA de mutação mentiu duas vezes hoje, de duas formas

**(a)** A restauração com `shutil.copy2` **preserva o mtime**: o arquivo restaurado ficava a parecer
mais velho que o objeto compilado da mutação, e a corrida seguinte servia a **build mutada** (§26.4).
Cura: `copy` + `os.utime`, e **exigir ver o `Compiling`** na saída — *uma prova de mutação mede o
binário, não o arquivo*.

**(b)** Um `if "subida" in label` sobre um rótulo escrito `SUBIDA`: a mutação **nunca entrou**, o
`assert` de "o arquivo mudou" passou (um comentário mudou-o), e o veredito saiu **VERDE**. À mão, a
mesma mutação dá `Some(4)` contra `Some(2)`.

⭐ **A cura é estrutural, não um remendo:** cada mutação passou a ser uma **lista de edições**, todas
com `assert` de contagem, sem caso especial por rótulo — a forma que torna a classe inteira
impossível. *Um falso VERDE numa prova de mutação é pior do que não a ter: ele certifica um gate que
não mede nada.*

### §29.6 — ⏸️ O que fica aberto

- Esconder a **base** de uma subtração faz o cortador virar a base (a mesma regra de quando ela é
  apagada). É consistente e pode surpreender; se um dia incomodar, a resposta é de produto.
- Não há **isolar** (mostrar só o escolhido) — é o gesto irmão, e um botão.

---

## §30 — W29: o cadeado trava a peça de modelagem (22/08)

> A quarta wave seguida da mesma família — e a que menos precisou de decidir seja o que for: **a lei
> já estava escrita, decidida pelo Enio, e o resto do app já a consultava.**

### §30.1 — O predicado é o da CASA, e ele tem duas metades

[`ph2d_ecs::is_locked_for_edit`](../../crates/ph2d-ecs/src/transform.rs) é consultado pelo gizmo 2D,
pelo Flip, pelas juntas e pelo vetorial. O gizmo **3D** era o único que não perguntava, e por isso o
cadeado era mudo **só aqui**.

E ele **sobe a cadeia**: `Locked` no próprio nó, ou `GroupedChildren` num antepassado. É a decisão do
Enio, escrita no doc do componente em 2026-05-26 — *"Cadeado trava apenas o objeto"*: os filhos de um
nó trancado continuam editáveis, e quem tranca a descendência é o grupo.

⚠️ **É por isso que a mutação «ler o cadeado à mão» tem gate próprio:** um `get::<Locked>` escrito
aqui compilaria, passaria o caso óbvio, e nasceria **já sem a metade do grupo**.

### §30.2 — ⚠️ Onde este módulo DIVERGE do gizmo 2D, e porquê

| | gizmo 2D da casa | aqui |
|---|---|---|
| um objeto trancado | o desenho **fica**, o *Down* é recusado | **não tem alças** |

Lá o gizmo é chrome permanente da seleção; aqui as alças são **o único sinal** de que o gesto existe,
e alças que não agarram seriam a mesma coisa que o botão pintado e morto que as três waves anteriores
foram fechar. O que se ganha lá — *«ele está ali»* — este módulo ganha na Hierarquia, que é onde o
cadeado e o olho se veem.

⭐ E as duas leis passaram a ser **uma pergunta**: `movable(world, e)` = não escondido **e** não
trancado. Duas listas de condições em dois sítios divergiriam no primeiro gesto novo.

### §30.3 — ⛔ O que o cadeado NÃO tranca, e quem o decidiu

Os **números do painel**. O doc do componente diz *"this entity's `Transform` is locked against gizmo
edits"* — o cadeado é sobre o **gesto**, e o painel é a outra porta. Não foi uma escolha desta wave:
foi lida. *Uma decisão que já existe não se re-decide num módulo.*

### §30.4 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| o módulo volta a ignorar o cadeado (**o defeito**) | `the_padlock_stops_the_gesture_and_the_group_locks_its_children` |
| o cadeado passa a ser lido à mão (perde a metade do grupo) | o mesmo gate |
| o gizmo volta a aparecer em cima do trancado | o mesmo gate |

---

## §31 — W30: arrastar uma linha na Hierarquia não teleporta a peça (22/08)

> A quinta da série — e a primeira que **não** era um controle mudo: era um **salto**. Arrastar um nó
> de modelagem para dentro de outro grupo mudava o pai e a peça **aparecia noutro sítio**.

### §31.1 — O mecanismo, e ele é de TIPO

O re-parentar da casa já preserva o mundo, e a decisão é do Enio (2026-06-08): *"a sprite that is
rotated / displaced must NOT jump when it gains or loses a parent"*. Ele fá-lo capturando o
**`Transform`** antes e re-resolvendo o local depois.

⚠️ **Um nó de modelagem não tem `Transform`** — a pose dele é `FieldPose`, com rotação em quaternion
e escala uniforme. Então `old_world` saía **`None`**, a preservação não corria, e a mesma pose
**local** passava a ser lida debaixo de outra cadeia de pais. *A lei estava certa e não alcançava
este tipo.*

### §31.2 — ⭐ A inversa vive ao lado da composição

`Xform::local_under(parent, world)` é a inversa exacta de `Xform::compose`, e está **no mesmo
arquivo**. Uma inversa escrita na crate da ponte seria uma segunda convenção sobre a mesma álgebra —
e a divergência dela é invisível até alguém re-parentar um nó **girado e escalado**, que é
precisamente o caso que o gate contém.

O par mede-se pelo **regresso**: `composing_and_undoing_a_pose_round_trip` compõe 16 combinações de
pai×filho e exige que a inversa devolva o filho — comparando o **efeito** da rotação, porque um
quaternion e o seu simétrico são a mesma rotação.

### §31.3 — ⛔ Uma prova de mutação achou um buraco no MEU gate

A mutação *"a inversa esquece a rotação do pai"* passou **VERDE** contra o gate da costura: ele
conferia a **posição** e o **tamanho** depois do re-parentar, e nunca a **orientação**. A peça ficava
no sítio certo, do tamanho certo, e **virada**.

⭐ *Uma prova de mutação não mede só o código: ela mede o gate.* O gate ganhou a terceira asserção, e
a mutação passou a vermelho.

### §31.4 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| a pose de mundo não é reposta (**o defeito**) | `reparenting_a_node_keeps_it_where_it_is` |
| a inversa esquece a **rotação** do pai | o mesmo gate, **depois de ele ganhar a asserção que faltava** |
| a inversa esquece a **escala** do pai | `composing_and_undoing_a_pose_round_trip` |
| o arrasto da Hierarquia deixa de perguntar | `the_hierarchy_drag_asks_this_module_where_the_node_was` |

### §31.5 — ⏸️ O que fica aberto

- ⚠️ **Re-parentar muda a PEÇA, não só a arrumação**: pôr um cilindro dentro de uma subtração passa a
  cortar com ele. É o que a árvore significa neste módulo, e é a razão de o gesto ser útil — mas
  ninguém o **diz**. Um aviso ao mudar um nó de operação é produto.
- O arrasto continua a não ter **desfazer próprio**: ele entra na fila do shell como qualquer edição
  do mundo (é a mesma captura), o que é o certo — mas não foi medido nesta wave.

---

## §32 — W31: só uma OPERAÇÃO pode ter filhos — e criar um grupo passa a ser um gesto (22/08)

> Enio, no smoke da W30: *"ainda não temos como criar novos grupos. Se coloco um objeto como filho do
> outro ele some."*
>
> ⭐ **As duas frases são UM defeito.** No idioma do campo, uma **forma** é uma folha: o cozimento
> emite-a e **nunca olha para os filhos dela**. Um nó largado ali fica no mundo, aparece na
> Hierarquia, e não entra em documento nenhum. E a única forma de aninhar era o botão de operação,
> que exigia **dois** selecionados. *Uma árvore que a UI aceita e a linguagem não exprime é um objeto
> que desaparece em silêncio.*

### §32.1 — A cura PROMOVE o anfitrião, em vez de recusar o gesto

A forma que recebeu o filho passa a viver dentro de uma **união** nova, no lugar dela — e o filho
entra ao lado. ⭐ **A peça na tela não muda com isso**: os dois já lá estavam, e a união deles é
exactamente o que se via. O artista ganha o aninhamento que pediu e não perde nada.

⚠️ **A ordem dos irmãos é preservada, e não é cerimónia:** em `children[0] menos os seguintes`, a
primeira posição é a **base** da subtração. Um grupo acrescentado no fim faria o cortador virar base —
a peça inverter-se-ia por alguém ter arrastado uma linha.

⚠️ **A lei impõe-se na DERIVAÇÃO**, no cozimento da cena, e não no drenar do arrasto: assim ela
apanha o arrasto da Hierarquia, um `add_child` de outro caminho, e o que vier a seguir. Um remendo no
drenar deixaria os outros de fora — é a mesma decisão da W23 (a escultura que volta) e da W28 (o olho).

### §32.2 — Criar um grupo: `wrap_in_op` passa a aceitar UM

O `>= 2` vinha de o gesto ter nascido como *«juntar os escolhidos»* (W9). Uma operação com um filho é
a mesma coisa que ela sempre foi — um `Union` de um é esse um —, e passa a ter **onde receber o
segundo**. Escolher uma forma e carregar numa operação **cria o grupo**.

⚠️ **O braço de trás precisou de guarda:** com uma seleção de um, o intent chamava `set_op`, que numa
folha era recusado **em silêncio** — o clique não fazia nada. Agora: uma **operação** sozinha troca de
operação (o gesto mais usado do módulo, com gate de controle), uma **forma** sozinha vira grupo.

⚠️ **Um gate antigo prendia a lei que mudou** (`wrapping_refuses_nodes_that_do_not_share_a_parent`,
na sua última linha). Ele foi **corrigido, não apagado**: as outras metades dele — pais diferentes, a
raiz — continuam a valer. *Uma lei que muda muda o gate.*

### §32.3 — ⛔ E o gate-mãe passou VERDE com a cura desligada

Ele contava **folhas na arena** — e um nó que ninguém referencia continua **escrito** lá: o cozimento
emite-o na subida, e é o **pai** que o deixa de fora. A contagem não continha o fenómeno.

⭐ A cura foi trocar a métrica pela pergunta que o Enio de facto fez: **a peça tem matéria neste
ponto?** — avaliar o campo onde a forma largada está. *O que ele viu foi a TELA, e é a tela que se
tem de medir.* (Segunda vez em duas waves que uma prova de mutação encontra o buraco no gate, e não
no código.)

### §32.4 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| a promoção não corre (**o defeito**: o objeto some) | `a_shape_dropped_onto_a_shape_is_not_lost` |
| a promoção põe o grupo no fim (o cortador vira base) | `promoting_a_host_keeps_who_cuts_whom` |
| embrulhar volta a exigir dois | `one_selected_shape_becomes_a_group` |
| uma operação sozinha passa a embrulhar-se | `an_operation_selected_alone_still_swaps_its_op` |

### §32.5 — ⏸️ O que fica aberto

- A promoção escolhe **união** por ser a operação neutra (a peça não muda). Trocar para subtração é
  um clique a seguir — mas ninguém **diz** que um grupo nasceu.
- ⏸️ Continua sem existir **grupo vazio**: cria-se a partir de uma forma, nunca do nada. É a mesma
  pergunta de produto do *isolar* (W28) e do aviso de re-parentar (W30).

---

## §33 — W32: o refinamento cede à mão — a última espera do preview (22/08)

> A W24 fechou com um número escrito: *"um traçado em voo não se cancela — se a mão recomeça a mexer
> no meio de um refinamento cheio, a resposta espera por ele, até **121 ms** medidos na cena mais
> pesada"*. Este é esse item, e o gatilho era o próprio número.

### §33.1 — ⛔ A regra óbvia mata a imagem

*"Mudou? abandona o que está a correr"* tem um modo de falha fatal: numa **órbita contínua** a câmera
muda a cada quadro, e um traçado grosso que leve mais do que um quadro seria cancelado antes de
acabar — **sempre**. O artista arrastaria o rato contra uma imagem **congelada**, e o defeito seria
muito pior do que a espera que se queria curar.

⭐ **A regra que sobrevive nomeia o caso medido:** um **refinamento** cede à mão; um traçado de
**movimento** corre até ao fim. E ela é consistente por construção — um refinamento só começa quando
nada está a mudar, então ele nunca está no caminho de si mesmo.

| em voo | pede-se | veredito |
|---|---|---|
| cheio (refinamento) | grosso (a mão voltou) | ⭐ **abandona** |
| grosso (movimento) | grosso | ⛔ nunca — a imagem congelaria |
| grosso (movimento) | cheio | ⛔ nunca — a imagem grossa é a que está a chegar |
| cheio | cheio | ⛔ um refinamento não se cancela a si próprio |

### §33.2 — A bandeira é lida POR LINHA, e o gate mede o tempo

Uma marcha abandonada custa **o resto das linhas a zero**, não o resto da imagem. ⚠️ **Se a bandeira
fosse lida uma vez no fim**, a função cumpriria o contrato (*devolve nada*) e não pouparia **um único
milissegundo** — que era a razão de existir. Por isso o gate mede as duas metades: `is_none()` **e**
`cut_ms < full_ms / 2`. O passe de anti-serrilhado cede pela mesma bandeira.

⚠️ **Um corpo só**: `trace`, `trace_with` e `trace_cancellable` passaram a delegar num
`trace_inner` com a bandeira opcional. Duas marchas seriam dois caminhos por onde a imagem pode
divergir, e a paridade delas não teria como ser medida sem uma terceira função para as comparar.

### §33.3 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| a bandeira é lida só no fim (o contrato cumpre-se, o tempo não) | `an_abandoned_march_returns_nothing_and_returns_fast` |
| a marcha abandonada devolve a imagem a meio | o mesmo gate |
| cancela-se **tudo** (a imagem congela) | `a_refinement_yields_to_the_hand_and_a_motion_trace_never_does` |
| **nunca** se cancela nada (a espera de 121 ms volta) | o mesmo gate |

### §33.4 — ⏸️ O que fica aberto

- O trabalho abandonado é **deitado fora**, não reaproveitado. Um refinamento que já traçou 80 % das
  linhas podia guardá-las — mas isso é um cache de linhas por câmera, e o gatilho para o construir
  seria uma medição que ainda ninguém fez.

---

## §34 — W33: a caixa da grade é a da PEÇA — e um corte silencioso a menos (22/08)

> O extrator montava a grade sobre `[-1, 1]` **fixo**, e a nota ao lado dizia: *"trocar por uma caixa
> apertada à peça multiplica a resolução efetiva e é wave própria"*. Ela não dizia a outra metade —
> **uma peça fora daquela caixa é cortada na exportação, sem uma palavra.**

### §34.1 — Duas consequências, e a primeira é muda

| | antes | agora |
|---|---|---|
| peça fora de `[-1, 1]` | ⛔ **cortada, em silêncio** (o artista abre no Blender e não está lá) | cabe |
| peça pequena no meio | resolução gasta em espaço vazio | a célula encolhe com a peça (**>3×** medido numa esfera de 0,25) |

⚠️ **O preço declarado:** `n = 2^depth` continua a ser o número de células por eixo — a **contagem**
de quads da tabela de exportação continua a valer —, mas o **tamanho** de cada célula passa a ser
relativo à peça. Para uma peça que enchia a caixa antiga, nada muda.

### §34.2 — ⭐ O bordo é uma ESFERA, e conservador é a direção segura

A esfera é **invariante à rotação**: subir a cadeia de poses custa `centro' = pose(centro)` e
`raio' = raio · escala`. Uma **caixa** teria de ser re-envolvida a cada nível girado, e cada
re-envolvimento cresce — três agrupamentos rodados dariam uma caixa muito maior do que a peça. *A
moeda certa para compor bordos é a que a composição não estraga.*

E toda aproximação erra **para cima**, por uma assimetria que é o critério inteiro: um bordo maior do
que a peça custa **resolução**; um bordo menor **corta a peça e não diz nada**.

⚠️ Duas leis que o gate prende: uma **subtração** não cresce com o cortador (o que se corta não
acrescenta matéria — um cortador enorme e distante não pode inflar a caixa), e os **modificadores**
crescem o bordo (casca, afastamento, matriz linear, radial, *taper*), cada um com a sua conta.

### §34.3 — ⛔ O gate apanhou um erro de desenho meu, na hora

A primeira versão punha a folga em **células** (`half = r·n/(n−4)`). Com isso a caixa mudava com a
profundidade, e **dobrar a resolução deixava de quadruplicar os vértices** — 4,61× medido. O gate
`the_exported_mesh_sits_on_the_surface` reprovou imediatamente: *uma grade uniforme tem de escalar
como uma grade uniforme.* A folga passou a ser uma **fração do raio**.

### §34.4 — ⛔ E o mesmo gate estava a medir a estatística ERRADA

Depois da cura, ele continuou vermelho: *"dobrar a resolução tem de cortar o erro ao meio"*, e o
máximo caiu só 1,75×. ⭐ **A tabela escrita no próprio gate já continha a resposta:** a coluna do
máximo cai ×3,8 · ×3,3 · ×3,1 e depois **×1,46** — porque um máximo sobre 4× mais vértices amostra
melhor a própria cauda. A lei da segunda ordem é do **erro médio**, e é o médio que a nota do gate
sempre citou; ele passava por a varredura parar na profundidade 7, e a caixa apertada (células
menores) trouxe o regime da cauda para dentro dela.

O gate foi **reconstruído, não afrouxado** — cada estatística medida pela lei que ela obedece:

| estatística | lei | barra |
|---|---|---|
| erro **médio** | segunda ordem (cai por 4) | ≥ 2× por degrau, e ≥ 20× de prof. 4 a 7 |
| erro **máximo** | é uma fração da **célula** | < 10 % da célula (medido: 1,1 % a 5,4 %) |

⚠️ A célula é **perguntada ao extrator** (`extract::cell_size`), nunca recopiada: desde esta wave ela
depende da peça, e uma segunda cópia da fórmula mediria uma grade que não existe.

### §34.5 — ⛔ Uma prova de mutação SOBREVIVEU, e a sobrevivência virou a nota

Pôr a folga a **zero** — a superfície a encostar na parede — passa **verde** em tudo. A razão é
geométrica: uma superfície tangente à esfera de bordo toca-a em **pontos isolados**, e uma travessia
com `f = 0` exactamente na parede continua a ser detectada.

⭐ Então **o que a folga protege não é a superfície: é o BORDO** — conservador por construção, mas
composto de aproximações (o *taper*, as matrizes, a caixa de uma escultura amostrada). 5 % do raio é
a margem dessas aproximações e custa 5 % da resolução. *Um número que nenhum gate faz falhar tem de
dizer de que ele é margem.*

⭐ E a procura por esse gate produziu o melhor da wave: **`the_exported_mesh_is_closed`** — toda
aresta em exactamente duas faces. *Uma média não vê seis buracos; a topologia vê* — e um buraco não é
erro de posição, é uma malha que a fatiadora recusa.

### §34.6 — Provas de mutação

| lei quebrada | gate vermelho |
|---|---|
| a caixa volta a ser `[-1,1]` (**o corte silencioso**) | `a_piece_far_from_the_origin_is_exported_whole` |
| a folga volta a depender da resolução | `the_exported_mesh_sits_on_the_surface` |
| o bordo esquece a **escala** da pose | `the_ball_contains_every_point_the_field_calls_solid` |
| o bordo esquece os **modificadores** | o mesmo gate |
| a união de bordos vira o primeiro filho | o mesmo gate |
| ⚠️ folga zero | **sobreviveu** — ver §34.5, com a razão escrita |

### §34.7 — ⏸️ O que fica aberto

- O bordo de uma **escultura** é a meia-diagonal da caixa da grade dela **mais** a distância do
  centro: conservador com folga a mais quando a malha importada não estava centrada. Apertá-lo pede
  a caixa da malha no trait, e o gatilho é uma medição que ainda ninguém fez.
- A **exportação não diz o tamanho** da peça. Agora que o bordo existe, dizê-lo é uma linha — e é a
  primeira pergunta de quem leva o arquivo para outro programa.

---

## §35 — W34: o gesto de criar grupo existia e ninguém lhe chegava (22/08)

> A W31 (§32) fechou dizendo *"criar grupo passa a ser um gesto"*. Era meia verdade. O **tratador**
> aprendeu a aceitar uma forma sozinha; o **painel** continuou a exigir duas, e por isso os três
> botões de operação **nunca eram pintados** nesse caso. O gesto existia no código, tinha gates
> verdes, e o artista não tinha como o disparar.

### §35.1 — ⛔ Por que os gates da W31 não notaram

Todos eles empurram a intenção diretamente:

```text
push_intent(ApplyOp { slot }) → sync_scene → o documento mudou?  ✅ VERDE
```

**Isso prova o TRATADOR, nunca a ALCANÇABILIDADE.** Empurrar a intenção encena um clique que o
artista não tem como dar — e um clique impossível passa em qualquer teste que o simule. É a **quinta**
reincidência da família da costura muda deste módulo (o modificador na escultura §26, a multi-seleção
§28, o olho §29, o cadeado §30, o reparentar §31) — e a primeira em que o buraco estava no **gate**,
não no código.

⚠️ *Uma cura pode entrar pela metade e a prova ficar verde, quando a prova entra pelo lado que a cura
já tinha.*

### §35.2 — ⭐ A cura é a LEI, não o caso

Acrescentar o caso que faltava curaria hoje e nada mais. O que ficou preso é a relação:

> ⭐ Para **toda fileira** de controles e **toda seleção**, *«o painel publica a fileira»* tem de valer
> **exatamente** *«a intenção daquela fileira muda o documento»*.

Ela apanha os dois sentidos: `oferecido=false age=true` é um gesto inalcançável (o defeito desta
wave); `oferecido=true age=false` é um botão pintado e mudo (a affordance que mente). E é lida do
**retrato publicado** (`state::current()`), não das funções que o montam: um `ops_for` correto que
ninguém ligasse ao `publish_snapshot` continua vermelho.

**Estruturalmente**, o que impede a divergência de voltar é não haver duas cópias da regra. Duas
funções novas em `ph2d-field-ecs`, cada uma consumida pelo gesto que ela guarda **e** pelo painel:

| pergunta | função | quem a consome |
|---|---|---|
| *"estes nós embrulham-se?"* | `can_wrap` | `wrap_in_op` + a fileira de operações |
| *"este nó destaca-se da peça?"* | `can_detach` | `duplicate`, `remove` + a fileira de ações |

⚠️ O gate `can_wrap_answers_exactly_what_wrap_in_op_does` mede o predicado **contra a função que ele
guarda**, não contra uma tabela escrita à mão: uma regra nova que entre num lado e não no outro fica
vermelha mesmo que ninguém se lembre de vir escrever o caso.

### §35.3 — ⭐ A generalização pagou no mesmo dia

Escrita para as operações, a lei foi apontada às outras duas fileiras que dependem da seleção — e
encontrou uma **irmã que ninguém procurava**:

| fileira | seleção | oferecido | age |
|---|---|---|---|
| operações | uma forma sozinha | ⛔ **false** | true ← o defeito da W31 |
| ações (*Duplicar/Apagar*) | a **raiz** da peça | ⛔ true | **false** ← o achado |

Com a peça inteira escolhida, os dois botões eram pintados e **os dois recusam a raiz** — por decisão
escrita em cada função (*"a raiz **é** a peça"*: duplicá-la seria uma segunda peça, apagá-la deixaria
o módulo sem nada para onde voltar). ⚠️ **A recusa era uma decisão; a affordance que a ignorava era
um defeito.** *Uma lei sobre uma fileira teria curado uma; sobre todas, encontrou a irmã.*

⚠️ **Quais fileiras entram:** só as que dependem da seleção (operações, modificadores, ações). As
formas, os verbos do gizmo e a exportação são ações sempre disponíveis — a de exportar nem sequer
toca o documento (ela anota um pedido), e medi-la por *«o documento mudou»* seria a pergunta errada.

### §35.4 — A tabela, medida

Estado depois da cura, sobre duas fixtures (`A ∪ B` plana e `A ∪ (B − C)` aninhada):

| seleção | operações | modificadores | ações |
|---|---|---|---|
| nada | — | — | — |
| uma forma sozinha | **cria grupo** | casca/afastamento | duplicar/apagar |
| dois irmãos | **embrulha-os** | do primeiro | do primeiro |
| a **raiz** da peça | troca o verbo | casca/afastamento | ⛔ **nenhuma** (recusadas) |
| um grupo **interno** | troca o verbo | casca/afastamento | duplicar/apagar |
| formas de **pais diferentes** | ⛔ nenhuma (mover é outro gesto) | do primeiro | do primeiro |

### §35.5 — Provas de mutação

| mutação | gate que ficou vermelho |
|---|---|
| a fileira de operações volta a exigir **dois** (o defeito da W31) | `the_gestures_the_product_promises_are_all_reachable` |
| a fileira de operações é publicada **sempre** | `the_rows_stay_silent_where_the_gesture_is_refused` |
| a fileira de ações volta a olhar só se **há seleção** | `the_panel_offers_exactly_what_the_gesture_does` |
| o `can_wrap` esquece o **pai comum** | `wrapping_refuses_nodes_that_do_not_share_a_parent` |
| o `can_detach` esquece que a **raiz não tem pai** | `the_rows_stay_silent_where_the_gesture_is_refused` |
| o `wrap_in_op` deixa de **consumir** o `can_wrap` | `can_wrap_answers_exactly_what_wrap_in_op_does` |

6 mutações, 6 vermelhas.

### §35.6 — ⏸️ O que fica aberto

- A lei cobre as fileiras que **dependem da seleção**. Um controle novo que dependa de outra coisa
  (um modo, um estado de câmera) não é apanhado por ela — e o padrão para o apanhar é o desta wave:
  publicar a partir do **predicado que o gesto consome**.
- ⚠️ Continua sem existir quem **diga** que um grupo nasceu (W31), e não há grupo **vazio**.
- ⚠️ Os botões de operação com uma forma sozinha mostram *Union/Subtract/Intersect* — o rótulo diz o
  **verbo**, não *"criar grupo"*. É legível para quem já sabe; a pergunta de produto é se devia
  haver um rótulo para o caso de um só.

---

## §36 — W35: a peça JÁ atravessava o arquivo — o que não atravessava era a memória (22/08)

> O primeiro item da fila dizia *"o `FieldDoc` não persiste no `ProjectFile` — fechá-lo move o
> `PROJECT_SCHEMA`"*. **Medido antes de construir: a peça persiste, e o `PROJECT_SCHEMA` não se
> mexe.** A nota estava velha. O que de facto faltava era outra coisa, mais estreita e mais
> silenciosa — e foi ela que esta wave curou.

### §36.1 — ⭐ Por que a peça já persistia sem uma linha de persistência

Porque ela **não é um documento**: é uma **árvore de entidades ECS**, e o `ProjectState` é o mundo
inteiro. A cadeia toda já existia, cada elo posto por uma wave anterior por outra razão:

| elo | onde | posto por |
|---|---|---|
| os componentes do campo entram no registro | `init.rs` → `register_field_components` | W1 |
| o registro é o que o `ProjectState::capture` usa | `undo.rs` | a casa |
| `ProjectFile.state` **é** o `ProjectState` | `project.rs` | a casa |
| a peça sobrevive ao snapshot | gate `the_whole_part_survives_the_world_snapshot_round_trip` | W5 |

⚠️ **`project_load.rs` não menciona o módulo uma única vez** — e isso é o desenho a funcionar, não
um buraco. *A pergunta certa não era «como faço a peça persistir», era «ela já persiste?».*

### §36.2 — ⚠️ A dependência que ninguém tinha escrito, e que quase se partia sozinha

O `ProjectState::restore` **apaga a cena antes de re-spawnar**, e a consulta que ele usa para apagar
é `With<Transform>`. Os nós deste módulo **não carregam `Transform`** — a pose deles é `FieldPose`,
porque o `Transform` da casa é uma afim **2D** (decisão medida na W5).

O que salva é que a **raiz** da peça leva `Transform` (para a Hierarquia a enumerar) e o despawn
**cascateia por `ChildOf`**. ⛔ São dois arquivos e duas razões diferentes, e **nada obrigava a
segunda a continuar verdadeira**. Se um dia um nó de campo nascer sem raiz com `Transform`, ele
sobrevive à limpeza e o load passa a **empilhar** a peça velha com a nova — em silêncio.

⭐ O gate novo `the_part_crosses_the_project_file_and_the_load_replaces_it_instead_of_stacking`
prende exatamente isso: ele **carrega sobre uma cena que já tem uma peça** (o caso real do Ctrl+O,
não o load sobre o vazio) e exige que sobre **uma**. A mutação que tira o `Transform` da raiz
deixa-o vermelho.

### §36.3 — ⭐ O defeito REAL: a memória de tentativas era do PROCESSO, e o limite é o DOCUMENTO

Quando o documento nomeia uma escultura que o registo não conhece, o módulo lê o arquivo e diz o que
não voltou (W23, §24). Para o aviso não repetir em todo quadro, ele guarda o que já tentou — num
conjunto **do processo**.

| passo | antes | agora |
|---|---|---|
| abro o projeto, a escultura falhou (arquivo movido) | o aviso sai ✅ | igual |
| **conserto o arquivo no disco** e abro outra vez | ⛔ **nunca relê** — e **não diz nada** | relê ✅ |

⛔ **O segundo silêncio é idêntico ao de quando estava tudo certo.** O artista consertou o que lhe
foi pedido e a peça continua a abrir sem a escultura, sem uma palavra.

A cura é **uma linha**, e ela entra numa família que já existia em `project_load` — a de *"o que o
documento anterior possuía e não pode atravessar"*:

```rust
ph2d_timeline::expr_owed::forget_owed_poses();   // as poses que uma expressão devia
self.forget_live_producers();                    // os produtores vivos
crate::field3d_reload::forget_tried();           // ⭐ as esculturas já tentadas (esta wave)
```

*Um Ctrl+O é o começo de um documento novo.* O `forget_tried` deixou de ser `#[cfg(test)]`.

### §36.4 — Provas de mutação

| mutação | gate que ficou vermelho |
|---|---|
| o load deixa de **esquecer** as tentativas | `a_load_starts_the_sculpture_reads_over` |
| o esquecimento vira **no-op** | o mesmo |
| a **raiz** da peça perde o `Transform` | `the_part_crosses_the_project_file_and_the_load_replaces_it_instead_of_stacking` |

3 mutações, 3 vermelhas. A terceira é a que prova que o gate do arquivo não é tautológico: ele mede
a lei do empilhamento, não a viagem dos bytes.

### §36.5 — ⏸️ O que fica aberto

- ⚠️ **O caminho de app não é alcançável de um gate:** `App::project_save` exige `self.gfx`
  (`capture_project` devolve `None` sem ele), então um save headless retorna cedo. O que se mede
  aqui é a **captura** e o **restore** reais, com os bytes pelo meio; o Ctrl+S de verdade é smoke.
  *Um arnês headless para o save fecharia isto, e o preço é do lado do `AppGfx`.*
- ⏸️ O módulo **não se abre sozinho** quando o projeto carregado tem uma peça: ela está no mundo e na
  Hierarquia, e o traçado só aparece depois de o artista carregar no pill **MODEL**.
- ⛔ **A nota do `PROJECT_SCHEMA` estava errada e foi corrigida** — persistir a peça **não** move
  degrau nenhum, porque ela nunca foi um campo do arquivo. *Uma nota que prescreve o preço errado
  faz a fila inteira ser ordenada errado.*

---

## §37 — W36: a exportação diz o TAMANHO — e havia dois números, não um (22/08)

> O toast da exportação já dizia quads, triângulos, KB e milissegundos. Não dizia **que tamanho a
> peça tem** — a primeira pergunta de quem leva o arquivo para outro programa. A W33 (§34) deixou a
> nota *"agora que o bordo existe, dizê-lo é uma linha"*. ⚠️ **Era uma linha, mas não a que a nota
> supunha:** o bordo é o número errado.

### §37.1 — ⭐ Os dois candidatos, e por que o óbvio mente

| candidato | o que é | veredito |
|---|---|---|
| a caixa do **bordo** (`bounds::bounding_ball().aabb()`) | o cubo que envolve a **esfera** que contém a peça — e a grade ainda lhe soma 5 % (`PAD_FRACTION`) | ⛔ é **andaime**: conservador por construção, e **cúbico** |
| a caixa da **malha** (`Mesh::bounds()`) | o que de facto foi escrito no arquivo | ⭐ é a resposta à pergunta que foi feita |

Dizer o do bordo seria responder *"que tamanho tem a caixa em que eu desenhei"* a quem perguntou
*"que tamanho tem a peça"*.

### §37.2 — A medição, e a fixture que quase escondeu tudo

Sonda `measure_the_grid_box_against_the_real_piece` (`--ignored --nocapture`):

| peça | bordo (x,y,z) | malha (x,y,z) | razão |
|---|---|---|---|
| esfera r=0,40 | 0,800 · 0,800 · 0,800 | 0,800 · 0,800 · 0,800 | **1,00× 1,00× 1,00×** |
| caixa fina 0,80 × 0,80 × 0,04 | 1,132 · 1,132 · 1,132 | 0,800 · 0,800 · **0,040** | 1,42× 1,42× **28,30×** |
| duas esferas afastadas | 1,500 · 1,500 · 1,500 | 1,500 · **0,300** · **0,300** | 1,00× **5,00×** 5,00× |

⛔⛔ **A esfera dá 1,00× nos três eixos** — porque numa esfera o bordo **é** a peça. Uma verificação
feita só nela teria confirmado o número errado com folga. *A fixture que concorda é a que não prova
nada*, e é por isso que a peça **fina** e a peça **afastada** estão na sonda: uma separa o eixo curto,
a outra separa a peça do espaço que ela ocupa.

⚠️ O gate `the_reported_size_is_the_mesh_that_shipped_not_the_grid_that_built_it` **exige que os dois
números divirjam** na fixture dele (`bordo > 5 × malha` no eixo curto). Sem essa metade ele passaria
numa fixture onde as duas leis coincidem, que é a mesma armadilha um nível acima.

### §37.3 — O que o artista lê

```
Exported 5 128 quads = 5 128 tris, 0.80 x 0.80 x 0.04, 412 KB in 38 ms -- model.obj (…)
```

⚠️ **Sem unidade, de propósito.** O documento deste módulo é adimensional — a única px→m do projeto é
a `ProjectSettings.pixels_per_meter`, que é da **física 2D** e não descreve esta peça. Carimbar "mm"
ou "m" aqui seria inventar uma escala que ninguém autorou. O número é o mesmo que o outro programa
vai mostrar, que é o que se queria.

### §37.4 — Provas de mutação

| mutação | gate que ficou vermelho |
|---|---|
| o tamanho passa a ser o do **andaime** | `the_reported_size_is_the_mesh_that_shipped_not_the_grid_that_built_it` |
| a malha **vazia** deixa de ser tratada (a caixa invertida vaza) | `an_empty_mesh_reports_zero_instead_of_a_negative_size` |

2 mutações, 2 vermelhas.

### §37.5 — ⏸️ O que fica aberto

- O `field3d_export` abre um diálogo nativo (`rfd`) e **não é alcançável de um gate**. O que se
  prende é a metade pura (`piece_size`); a linha do toast em si é smoke.
- ⏸️ **Nada diz onde a peça ESTÁ**, só que tamanho ela tem. Para quem monta várias peças no mesmo
  arquivo, o canto mínimo importa tanto quanto a extensão — e o número já está na mesma `Aabb`.

---

## §38 — W37: a mensagem vivia UM quadro — o diálogo que a precede congela o relógio (22/08)

> Enio, no smoke da W36: *"não vejo em nenhum lugar a mensagem"*. Ela **estava** a ser escrita e
> **estava** a ser pintada. Vivia 16 ms. ⛔ E o defeito não é do módulo de modelagem: é da casa, e
> atinge **toda** mensagem escrita depois de um diálogo de arquivo.

### §38.1 — O mecanismo

```text
quadro N:   wall_dt = agora − início do quadro anterior     (render_loop, linha 1088)
            toasts.tick(wall_dt)                             ← envelhece o que já existia
            …
            [ DIÁLOGO MODAL ABERTO — o loop CONGELA 20 s ]
            toast criado, idade 0                            ← pintado 1× no fim deste quadro
quadro N+1: wall_dt ≈ 20 s  →  toasts.tick(20 s)            ⛔ idade > TTL 3 s → MORRE
```

⚠️ **O `wall_dt` mede o quadro INTEIRO, e o diálogo acontece dentro dele.** A mensagem que devia
durar 3 segundos aparece num quadro e desaparece. Da cadeira, isso é *não aparecer* — e é por isso
que o sintoma do Enio é literalmente exato.

### §38.2 — ⭐ Um número a responder DUAS perguntas

| pergunta | quem lê | o congelamento conta? |
|---|---|---|
| *quanto durou o último quadro?* | medidor de fps · acumulador da sim | **sim** — foi mesmo esse tempo |
| *quanto a UI ANIMOU?* | os toasts · `hero.tick_motion` | ⛔ **não** — nada se moveu, a tela estava parada |

⚠️ **A nota do `render_loop` (linha 1082) está certa sobre o caso que ela curou** — o `ToastQueue`
contava **quadros**, e um toast de "3 s" durava 6 s a 30 fps — e foi ela que unificou o relógio. O
que ela não previu foi o **congelamento**: aí as duas perguntas divergem.

⭐ A cura **não é um segundo relógio nem um teto mágico**: é a mesma medição com a parte parada
**nomeada por quem a causou**. `crate::modal::note_stall` declara; `chrome_dt(wall_dt, stall)`
desconta; o medidor de fps e o `fixed_step.advance` continuam a ler o `wall_dt` inteiro (e a sim já
se protegia sozinha — ela limita os sub-passos e larga o excesso).

### §38.3 — ⚠️ A porta, e o que ela ainda não cobre

Um `rfd::FileDialog` aberto à mão volta a congelar **sem declarar**. Por isso ele passa por
`modal::save_file` / `modal::pick_file`, com gate (`every_field3d_modal_goes_through_the_door`).

⛔⛔ **MEDIDO: há 25 chamadas de `rfd::FileDialog` em 12 arquivos deste shell.** Esta wave liga as
**duas** do módulo. As outras **23** continuam a perder a mensagem que escrevem a seguir:

| arquivo | chamadas |
|---|---|
| `render_loop/mod.rs` | 12 |
| `forwarding.rs` · `render_loop/tokens_bridge_dtcg.rs` | 2 cada |
| `image_export.rs` · `render_loop/image_edit.rs` · `render_loop/painter_bridge_assets.rs` · `sculpt3d_export.rs` · `sculpt3d_import.rs` · `sheet_export.rs` · `vec_text.rs` | 1 cada |

⚠️ **O gate NÃO foi alargado a elas de propósito:** seria entregar ao integrador um vermelho sobre
código de outras linhas. *O defeito tem endereço, a porta existe, e a próxima linha que tocar num
desses arquivos tem uma linha para escrever.*

### §38.4 — Provas de mutação, e a que SOBREVIVEU primeiro

| mutação | gate que ficou vermelho |
|---|---|
| o relógio do chrome volta a cobrar o congelamento | `a_frozen_loop_does_not_age_the_message_it_was_about_to_show` |
| a porta deixa de **declarar** o congelamento | `the_door_times_what_goes_through_it` |
| o congelamento fica **pousado** (o `take` não zera) | `the_stall_is_taken_once_and_then_it_is_gone` |
| o **loop** volta a andar com o `wall_dt` inteiro | `the_chrome_clock_reads_the_discounted_dt` |
| o export volta a abrir o diálogo **fora** da porta | `every_field3d_modal_goes_through_the_door` |

⭐ **A segunda sobreviveu na primeira corrida**, e o achado é o de sempre um nível abaixo: com o
cronómetro **dentro** do `save_file`, tirá-lo de lá deixava tudo verde — os gates chamavam
`note_stall` à mão, e a **porta**, que é a única coisa que liga o diálogo ao relógio, não era
exercida por nenhum. Um `rfd::FileDialog` não abre num teste, mas *"o que passa por aqui é
cronometrado"* abre: o cronómetro saiu para um `timed(f)` e ganhou gate. **O que sobra sem gate é
uma linha por porta, e ela não tem lógica nenhuma.**

⚠️ E o gate da porta reprovou primeiro sobre o **próprio comentário** que explica a regra — o texto
que diz *"nunca chame isto direto"* contém, por construção, exactamente a agulha. *Um gate que lê a
prosa sobre a lei em vez do código que a obedece reprova quem a documenta.* Comentários fora.

### §38.5 — ⏸️ O que fica aberto

- ⏸️ As **23** chamadas das outras linhas (tabela acima).
- ⏸️ O `ph2d_editor::Toast` não sabe distinguir *"mensagem de resultado"* de *"aviso passageiro"*: um
  resultado de exportação talvez devesse durar mais do que 3 s, ou ficar até ser lido. É produto.

---

## §39 — W38: isolar não precisou de lei nenhuma — e as duas vozes que faltavam (22/08)

> Dois itens abertos de uma vez: **isolar** (mostrar só o escolhido) e **ninguém diz que um grupo
> nasceu**. O primeiro parecia a obra da wave; acabou por ser uma **generalização de uma função que
> já existia**.

### §39.1 — ⭐ A lei foi LIDA, não decidida

O módulo irmão já tinha um isolar (`sculpt3d_objects::toggle_isolate`), e ele responde às três
perguntas de desenho antes de alguém as fazer:

| pergunta | a resposta do irmão, verbatim |
|---|---|
| toggle ou modo com saída? | *"um «sair do isolamento» separado seria uma segunda porta para o mesmo fato, e a que o artista não acha quando a cena some"* |
| entra no undo? | *"nada aqui entra na história — isolar não move um vértice"* |
| e isolar «nada»? | recusado: *"apagaria a cena da tela sem nada para devolver"* |

⇒ Estado de **vista**, ao lado do verbo do gizmo. Não é componente do mundo, não viaja no arquivo,
não é passo de desfazer. *Quinta vez que este módulo lê uma lei da casa em vez de inventar uma.*

### §39.2 — ⭐ E o mecanismo era zero

O `cook` diz, desde a W5: *"coze a **subárvore** de `root`"*. **Isolar é cozer a partir daquele
nó** — os irmãos ficam de fora porque a travessia nunca chega a eles, e a operação que os juntava
fica de fora porque está acima. Nenhum filtro novo, nenhuma segunda lei sobre *"o que está na peça"*.

Faltava **uma linha**: a pose da cadeia **acima** do nó. Sem ela, isolar um nó dentro de um grupo
deslocado atirava a peça para a origem — e da cadeira isso lê como *"isolar mexeu no meu modelo"*.

⚠️ **A linha é inerte na raiz verdadeira** (não há nada acima dela), e há gate a exigi-lo
byte-a-byte: `cooking_from_the_real_root_is_unchanged_by_the_chain`. *Uma generalização que muda o
caso que já funcionava não é uma generalização, é uma regressão.*

### §39.3 — ⚠️ O isolamento é CONFERIDO, não obedecido

Ele guarda **bits de entidade**, e os bits morrem num undo (o restore respawna tudo com ids novos).
Obedecer a um alvo que já não existe apagaria a peça da tela **sem nada a explicar** — o modo de
falha exacto que este módulo já pagou cinco vezes com outro nome. Um alvo morto é **largado**, com
aviso, e a peça inteira volta.

### §39.4 — A outra voz: o grupo que nascia mudo

A W31 fez o gesto e não o disse. A Hierarquia ganha uma linha, o objeto escolhido passa a estar um
nível abaixo, e nada explica porquê. O aviso diz **quantos** entraram — o que distingue *"criei um
grupo com esta forma"* de *"embrulhei as três"*.

### §39.5 — Provas de mutação, e a que passou VERDE

| mutação | gate que ficou vermelho |
|---|---|
| o `cook` perde a **pose da cadeia** | `the_isolated_piece_stays_where_it_was` |
| o `cook_root` **obedece** a um alvo morto | `an_isolation_pinned_to_a_dead_object_is_dropped` |
| o `cook_root` **ignora** o isolamento | `isolating_shows_only_that_subtree_and_the_whole_part_comes_back` |
| isolar «nada» passa a **sair** | `the_isolation_law_toggles_swaps_and_refuses_nothing` |
| isolar outro passa a **sair** em vez de trocar | o mesmo |
| o grupo que nasce volta a ser **mudo** | `a_born_group_says_so` |

⛔ **A primeira passou VERDE na primeira corrida**, e a causa é a que esta mesma linha escreveu
**duas vezes hoje**: a fixture do gate tinha a **raiz na identidade**, então a pose local do grupo
interno e a de mundo coincidiam — apagar a composição da cadeia era indistinguível de não a haver.
A raiz saiu da identidade e o gate ganhou um **controle explícito** (`local ≠ mundo`), que é o que
impede a fixture de voltar a concordar por acidente. *A fixture que concorda é a que não prova nada.*

### §39.6 — ⚠️ Por que a lei do toggle é uma função PURA

O `Smoke` só nasce com o módulo **armado** (env var ou pill), e o estado dele é `thread_local` —
que com `--test-threads=1` é **partilhado**. Armar o módulo para exercer uma regra de três linhas
contaminaria todos os gates que corressem depois. Então a lei vive em `next_isolation(atual,
pedido)`, sem estado nenhum, e o smoke apenas a consome.

### §39.7 — ⏸️ O que fica aberto

- ⏸️ **A costura painel↔smoke do isolar não tem gate comportamental** — pela razão do §39.6. O que
  está preso é a lei (pura), o cozimento (`cook_root`) e a fileira do painel (a lei da W34, com o
  `slots` agora **derivado** do `ACTS` — um literal `2` teria deixado o botão novo fora da varredura).
- ⏸️ Isolar **não tem tecla**. O irmão tem; aqui só o botão.
- ⏸️ **Nada mostra, na Hierarquia, que há um isolamento em curso** — o botão aceso está no painel, e
  quem olha para a lista de objetos não vê porque metade deles não está na tela.

---

## §40 — W39: a escultura da cena entra sem passar pelo disco — e o «vivo» era impossível (22/08)

> O item dizia: *"o vínculo à escultura **viva** do módulo 3D (hoje passa pelo disco)"*. **A medição
> mudou o que a palavra «vivo» podia significar**, e é ela que desenhou a wave.

### §40.1 — ⛔ O que o número proibiu

Sonda `measure_sculpt_to_field_bridge` (`--ignored --nocapture`), voxelizar uma malha em campo:

| triângulos | res | células | MB | ms |
|---|---|---|---|---|
| 1 024 | 128 | 2 352 637 | 15,7 | **228,6** |
| 10 000 | 128 | 2 352 637 | 15,7 | **258,7** |
| 50 176 | 128 | 2 352 637 | 15,7 | **388,5** |
| 50 176 | 256 | 17 779 581 | 118,7 | **1 504,8** |

Um quadro tem **16,7 ms**. ⛔ **Um vínculo contínuo — editar a escultura e o modelo acompanhar — são
14 a 23 quadros de congelamento por pincelada.** Não é afinação: é a classe da operação.

⚠️ E a decisão **já estava escrita**, no doc da `DEFAULT_RESOLUTION`: *"o custo é pago **uma vez**,
na importação, não por quadro"*. Esta wave é essa decisão a valer também para a cena. ⇒ «vivo» aqui
significa **um gesto que traz a escultura como ela está agora**, não um espelho.

### §40.2 — O que passou a existir

Um segundo botão, **`+ Sculpt from scene`**, ao lado do `+ Sculpt…`. ⚠️ **Sem reticências, e a
convenção é o que diz a diferença antes do clique**: aquele abre um diálogo, este não pergunta nada.

⚠️ **Ele só é oferecido quando há uma escultura na cena** — a lei da W34 aplicada à única forma cuja
disponibilidade não é constante. As outras cinco são sempre possíveis: uma caixa não depende de nada.

⭐ **E as duas portas partilham UMA voxelização** (`field_from_mesh`). Não é arrumação: o documento
guarda uma **chave**, não a grade, e duas voxelizações diferentes dariam uma peça que muda de forma
conforme por onde entrou — **sem nada na tela a dizê-lo**.

### §40.3 — ⚠️ A chave `scene:` não é um arquivo, e o resolvedor precisava de o saber

Reabrir um projeto cuja peça usa a escultura da cena daria *"Sculpture scene:sculpt is missing"* — um
aviso a mandar o artista **procurar um arquivo que nunca existiu**.

Quem a pode reconstruir é o **shell** (a escultura viva está no `AppGfx`; a ponte com a cena recebe o
mundo). Então o resolvedor **pede**, pela mesma caixa de correio que o botão usa — e o `first_try` da
W23 garante **um pedido por documento**, não um por quadro.

### §40.4 — Provas de mutação

| mutação | gate que ficou vermelho |
|---|---|
| o botão da cena é oferecido **sempre** | `the_scene_sculpture_button_appears_only_when_there_is_one` |
| as duas posições da escultura **colidem** | `the_two_sculpture_slots_are_distinct_and_in_range` |
| a chave da cena deixa de casar o **prefixo** | `the_scene_key_is_the_one_the_resolver_recognises` |
| a chave `scene:` volta a ser lida do **disco** | `a_scene_key_is_asked_for_instead_of_being_read_from_disk` |

4 mutações, 4 vermelhas. ⚠️ A segunda existe porque `SCULPT_SLOT` passou de `len()-1` a `len()-2`
com um irmão ao lado: um `-1` esquecido faria os dois botões serem **o mesmo slot**, e o diálogo
abriria no botão errado **sem erro nenhum**.

### §40.5 — ⏸️ O que fica aberto

- ⏸️ **A escultura entra como estava, e não se atualiza sozinha** — trazê-la outra vez atualiza. É o
  que os 229–389 ms permitem, e o botão não diz isso: um artista que esculpa mais depois de a trazer
  não tem nada na tela a lembrá-lo de a re-trazer.
- ⏸️ **Uma escultura por peça**: a chave é fixa (`scene:sculpt`). Duas cópias da mesma escultura
  partilham o campo — o que é certo — mas duas esculturas **diferentes** na cena não são exprimíveis.
- ⏸️ O caminho `scene:` **não foi smokado com um projeto reaberto** — o pedido está gateado, o
  reencontro depende do shell servir com a escultura já instalada.

---

## §41 — W40: o modelador não cedia o canvas a ninguém (22/08)

> Enio, 2026-08-22: *"o modo Modelagem nunca é desativado e não consigo usar nenhum outro modo do
> app. Se eu entro no modo sculpt ou vector ou qualquer outro, o Modelagem deve ceder. **Não consigo
> esculpir nada** pois o modo de modelagem permanece interferindo."*
>
> ⛔ Não é um incómodo: com o modelador aberto, **o resto do app fica inutilizável**.

### §41.1 — ⚠️ É o MESMO report de duas waves da escultura, um nível acima

O módulo irmão já pagou este defeito **duas vezes**, e as duas frases estão escritas no
`input_dispatch`:

| data | o report | a lição |
|---|---|---|
| 09/08 | *"não consigo configurar a textura da sprite já que não posso sair do modo escultura"* | um modo de canvas precisa de **saída** |
| 17/08 | *"depois de abrir outros módulos como Sculpt, o Motion não consegue usar os atalhos"* | *"o **ponteiro** já cedia, o **teclado** não — uma assimetria entre duas portas que respondem à MESMA pergunta"* |

⭐ **Aqui a assimetria é maior: nenhuma das duas cedia.** O modelador é armado pela **visibilidade do
painel** (`set_armed_by_panel`), e **nada no app fechava esse painel**. Enquanto ele estivesse
aberto, o traçado desenhava por cima do canvas e o ponteiro era dele — para sempre.

*Uma lei que o módulo vizinho já pagou duas vezes não é uma descoberta: é uma leitura que não foi
feita.*

### §41.2 — ⭐ A lei: tomar o canvas LIBERTA quem o tinha

Não é *"o modelador desliga-se sozinho"*. É que o canvas tem **um** dono, e pegar nele é um gesto
que solta os outros — **duas metades simétricas**:

| quem entra | quem cede |
|---|---|
| uma **ferramenta** é pegada no rail, ou o **barro** aparece na tela | o painel MODEL **fecha** |
| o pill **MODEL** é aberto | o **barro** sai da tela |

⚠️ **Fecha-se o painel, não se desarma em silêncio:** o pill *é* o interruptor do módulo, então um
desarme invisível deixaria o botão aceso a mentir sobre o estado. O artista vê por que o modelador
saiu — e há um aviso a dizê-lo.

⚠️ **A saída do barro é a porta do PRÓPRIO módulo de escultura** (`toggle_clay`), nunca uma escrita
aqui: ela conhece a ordem do ciclo (sair do barro vai para a **luz**, não para o desligado), e essa
ordem é uma decisão de produto com um dono.

### §41.3 — ⚠️ Por que a lei é de BORDA, e não contínua

A regra contínua — *"MODEL cede enquanto houver ferramenta em mãos"* — é mais simples e **cria um
impasse**: uma ferramenta pegada **fica** em mãos (`set_active` no mesmo id é no-op, e não há gesto
de largar), então o modelador **nunca mais abriria**. A borda diz o que foi pedido (*"se eu entro
noutro modo"*) sem tirar o caminho de volta.

E as duas bordas são **independentes**: no quadro em que se pega uma ferramenta *e* o MODEL abre,
juntá-las num estado só faria uma mascarar a outra — e um dos dois modos ficaria de pé sem ninguém
ter decidido qual.

### §41.4 — Provas de mutação

| mutação | gate que ficou vermelho |
|---|---|
| a lei deixa de ver a **ferramenta** (o caso do Vector) | `entering_another_mode_takes_the_canvas` |
| a lei deixa de ver o **barro** (o caso do Sculpt) | o mesmo |
| a lei vira **contínua** (o painel fecha a cada quadro) | `nothing_changing_is_not_taking` |
| **largar** a ferramenta passa a fechar o painel | `dropping_a_tool_is_not_taking_the_canvas` |
| a borda do MODEL **deixa de ser borda** | `opening_the_model_panel_is_its_own_edge` |
| o **loop** deixa de fechar o painel (desarma em silêncio) | `the_render_loop_actually_makes_the_modes_cede` |

6 mutações, 6 vermelhas. ⚠️ A última é a lei da W34 outra vez: *provar o cálculo não prova a
alcançabilidade dele* — sem ela as funções podiam estar perfeitas e não ser chamadas por ninguém.

### §41.5 — ⏸️ O que fica aberto

- ⏸️ **Reabrir o MODEL com uma ferramenta ainda em mãos** deixa as duas de pé: o modelador desenha e
  a ferramenta continua registada. Ele não *interfere* (o artista está a modelar), mas ninguém
  largou nada — e não há gesto de largar uma ferramenta neste app.
- ⏸️ **O modelador não tem um pill próprio**: ele é armado pela visibilidade do painel. Isso funciona
  e é o que torna esta cura barata, mas mistura *"o painel está aberto"* com *"estou a modelar"* —
  duas perguntas que o módulo de escultura já separou (o pill é binário, o `D` percorre o ciclo).
- ⏸️ A `line/sculpt3d` ganhou **uma palavra** (`pub(super)` → `pub(crate)` no `toggle_clay`), pela
  mesma razão que a irmã `clay_on_screen` já era `pub(crate)`.

---

## §42 — W41: o crash que o smoke da W40 encontrou na escultura (22/08)

> ⚠️ **Não é um defeito deste módulo** — mas foi a lei de ceder (§41) que levou o Enio até ele, e ele
> derruba o app. O registro fica aqui porque é aqui que a linha o mediu e o fechou pela porta.

```text
[sculpt3d] APAGOU: sobram 0 pecas -- Ctrl+Z a devolve INTEIRA
PH2D PANIC frame=2523 ... sculpt3d_input.rs:173
message="index out of bounds: the len is 0 but the index is 0"
```

### §42.1 — ⭐ A causa: um estado LEGÍTIMO que um caminho supõe impossível

O `delete_active` **produz** a cena vazia e promete o Ctrl+Z de volta — é uma escolha, escrita na
mensagem que ele imprime. O que faltava era a outra metade: os caminhos de **gesto** indexam
`objects[active]` **direto**, e com a lista vazia o `active` (que o delete prende em 0) aponta para
nada.

*Um estado que o módulo declara legal e um caminho que o supõe impossível é um pânico à espera do
primeiro clique.*

### §42.2 — ⛔ A extensão MEDIDA, e por que a cura completa não é desta linha

| medida | valor |
|---|---|
| indexações `objects[…]` **sem guarda** | **42** |
| arquivos do módulo irmão | **9** (`filter` · `dyntopo` · `pull` · `transform` · `input` · `space` · `objects` · `import`) |
| portas de gesto que lá chegam | 5 (`pointer_down` · `pointer_move` · `wheel` · `key` · `apply_panel_intent`) |
| a porta segura que elas deviam usar | **já existe** (`obj()` / `obj_mut()`) |

⇒ A `line/3DModeling` fecha **a porta que o artista bateu** (`sculpt3d_pointer_down`, uma guarda que
recusa **e reporta** — a lei que o próprio módulo já segue no `Delete`) e **nomeia o resto**.
Reescrever 42 sítios de um módulo alheio, com a linha dele viva, não é uma decisão desta.

### §42.3 — ⚠️ O gate é de FONTE, e ele diz porquê

A `Sculpt3dScene` precisa de um `Device` de GPU para existir, então **nenhum gate deste repositório
a constrói** — a suíte inteira do módulo irmão testa as funções *puras* à volta dela, e diz isso no
cabeçalho. O que sobra é medir o fonte: a guarda existe **e vem antes** da primeira indexação. O
gate exige as duas coisas — sem a segunda metade ele passaria com a guarda escrita depois do
pânico. Mutação: tirar a guarda → vermelho.

### §42.4 — ⏸️ O que fica aberto

- ⏸️ **As outras 4 portas** (`pointer_move`, `wheel`, `key`, `apply_panel_intent`) e as 41
  indexações restantes — trabalho da `line/sculpt3d`, com o endereço acima.
- ⏸️ ⚠️ **O pânico terminou em `SIGSEGV`**: o processo não morreu pelo `panic!`, morreu a desenrolar
  a pilha (provavelmente a superfície `wgpu` a ser largada durante o unwind). *Um app que crasha ao
  crashar perde o relatório que explicaria o primeiro crash* — e isso é da casa, não de um módulo.

---

## §43 — W42: desarmar não desarmava — a cerca cuja razão dissolveu (22/08)

> Enio, depois do smoke da W40: *"ainda não consigo usar outros modos como vector."*
>
> A W41 (§41) fechou o painel quando outro modo entra — e **não bastou**. O painel fechava e o
> módulo continuava a comer o ponteiro. ⚠️ **A frase certa era a primeira que ele escreveu:** *"o
> modo Modelagem nunca é desativado"*, à letra.

### §43.1 — ⛔ Duas causas empilhadas, e as duas eram notas a mentir

**1. O `with_smoke` prometia no doc o que não fazia no código.**

> *"Devolve `None` quando o smoke não está armado — e é isso que faz cada gancho de entrada ser
> **inerte** (e portanto invisível) fora dele."*

O `armed_scene()` era consultado **só dentro do `boot()`** — isto é, **só enquanto o smoke ainda não
existia**. Nascido uma vez, ele vivia para sempre.

**2. A bandeira TRAVAVA LIGADA, e havia uma razão escrita ao lado:**

> *"Fechar o painel fecha o PAINEL; a peça continua na cena… Fazer o X do painel apagar o modelo da
> tela seria um segundo significado para o mesmo gesto, e o artista perderia a peça sem a ter
> apagado."*

⭐ **A metade protegida estava certa; a conclusão não.** O medo era perder a **peça** — e a peça não
vive ali: **desde a W5** ela é uma **árvore de entidades ECS** (Hierarquia, save, undo), e o `Smoke`
é só o cache do quadro para a thread do traçado. Largá-lo perde o cache e a câmera, nada mais.

⚠️ **A razão da cerca dissolveu na W5 e ninguém reconferiu a nota** — e a cerca continuou a cobrar o
preço dela: o app inteiro.

### §43.2 — ⭐ Por que esculpir funcionava e o Vector não

```text
input_dispatch.rs:3174   sculpt3d_pointer_down   ← a escultura vê o clique
input_dispatch.rs:3186   field3d_pointer_down    ← a modelagem come-o aqui
        …                o Vector, o gizmo, a seleção  ← nunca vêem nada
```

*A ordem do despacho transformou um bug em dois sintomas diferentes, e o segundo parecia outra
coisa.* Foi por isso que a W40 «passou» no smoke da escultura e reprovou no do Vector.

### §43.3 — A cura, e o gate que a nota devia ter sido

| o que mudou | porquê |
|---|---|
| `with_smoke` consulta `armed_scene()` **sempre**, e larga a cena quando desarmado | o doc dele já o prometia |
| `set_armed_by_panel` segue o painel **nos dois sentidos** | a razão da trava dissolveu na W5 |
| gate `rearming_does_not_replant_the_demo_over_the_artists_piece` | ⭐ **o gate que a nota devia ter sido** |

⚠️ **O terceiro é o mais importante desta wave.** A nota afirmava um risco *sem prova*, e a
afirmação bastou para prender a bandeira. Um gate no lugar dela teria mostrado, **no dia**, que já
não havia o que proteger. *Uma nota é uma afirmação; só um gate a mantém verdadeira.*

### §43.4 — Provas de mutação

| mutação | gate que ficou vermelho |
|---|---|
| a bandeira volta a **travar ligada** | `disarming_the_module_actually_disarms_it` |
| o `with_smoke` volta a **não reler** o estado (a W40 sozinha) | o mesmo |
| rearmar passa a **replantar** a semente por cima da peça | `rearming_does_not_replant_the_demo_over_the_artists_piece` |

3 mutações, 3 vermelhas. ⚠️ A terceira **sobreviveu à primeira tentativa**: eu mutei a linha errada
(`seed.take()` → `clone()`), e ela não exprime o replantio — quem o impede é a raiz **ser
encontrada**, não a semente ser consumida. *Uma mutação que não exprime o defeito é um verde que não
prova nada, e a agulha certa era duas funções ao lado.*

### §43.5 — ⏸️ O que fica aberto

- ⏸️ Fechar o painel larga a **câmera** do módulo: reabrir volta ao enquadramento inicial. A peça é
  preservada (há gate), o ponto de vista não. É o preço da lei, e é barato — mas não é zero.
- ⏸️ **A W40 sozinha não era suficiente e o gate dela passava**: ele mede que o *loop fecha o
  painel*, e fechar o painel não desarmava. *Um gate de costura prova a costura que nomeia, não a
  consequência que se espera dela.*

---

## §44 — W43: a VISTA sobrevive ao fecho — e a categoria já estava escrita em três sítios (23/08)

> O ⏸️ que a W42 deixou, à letra: *"fica: fechar o painel larga a **câmera** (a peça não)"*.

A W42 fez o pill desarmar de verdade, e cobrou o preço na hora. O artista pousa a peça num ângulo,
pega no Vector (o painel fecha, W40), volta ao MODEL — e encontra a peça certa **vista de outro
sítio, a girar sozinha**. A peça atravessa porque é uma árvore de entidades; a vista não atravessa
porque vivia no cache do quadro, que é precisamente o que a W42 passou a deitar fora.

### §44.1 — ⭐ Não era «a câmera»: eram cinco campos, e o `Smoke` já os classificava

O que se ia construir era *"guardar a `Orbit`"*. A medição mudou o entregável: os doc-comments do
próprio `Smoke` **já diziam a que categoria cada campo pertence**, e diziam-no três vezes.

| campo | o que o doc dele já dizia, antes desta wave |
|---|---|
| `gizmo_mode` | *"É estado de **vista**, e não do documento: por isso vive aqui e não num componente"* |
| `gizmo_frame` | *"Estado de **vista**, como o verbo"* |
| `isolated` | *"Estado de VISTA, e a lei é a do módulo irmão"* |
| `cam` | **é** a vista |
| `manual` | *"o prato para de girar assim que o artista toca nele"* |

⚠️ **Um doc-comment repetido em N campos é uma estrutura por nascer.** A categoria existia, com
nome, escrita à mão em cada membro — e sem nenhum sítio onde a lei dela (o *tempo de vida*) pudesse
ser dita uma vez. Guardar só a `Orbit` teria deixado quatro campos a morrer no mesmo fecho, cada um
com o próprio doc a afirmar que era da mesma família.

### §44.2 — ⚠️ O `manual` viaja com a câmera, ou nenhum dos dois vale

Restaurar a câmera **sem** o `manual` não restaura nada: o prato volta a girar e afasta-se do ângulo
restaurado a partir do quadro seguinte. O número certo estaria na tela durante 16 ms, e o defeito
leria como *"restaurar a câmera não funciona"* — com a cura já lá dentro. Há gate separado
(`the_turntable_stays_stopped_across_a_close`), porque *provar só uma metade de um fato engana*.

### §44.3 — ⭐⭐ Como um campo NOVO não se perde: destructuring sem `..`

O modo de falha desta wave, daqui a três meses, é alguém acrescentar um campo de vista ao `Smoke` e
ele morrer em silêncio no fecho. A cura é estrutural e custa zero em tempo de execução:

```rust
fn of(s: &Smoke) -> Self {
    let Smoke { cam, manual, gizmo_mode, gizmo_frame, isolated,
                doc: _, seed: _, matcap: _, /* … os 22 restantes, um a um … */ } = s;
    …
}
```

**Sem `..`.** Um campo novo no `Smoke` passa a ser **erro de compilação** exatamente no sítio onde a
pergunta tem de ser respondida: *isto é vista, ou é cache do quadro?*

⚠️ É a lição do `Shade::default()` da `line/sculpt3d` (CLAUDE.md §5: *"a vista agora é escrita por
nome, os 7 campos, então um termo novo é erro de compilação ali"*), com uma diferença que importa:
lá a escrita por nome está na **saída** (quem lê), aqui está na **entrada** (quem desmonta). *O lado
da entrada apanha quem acrescenta o campo; o da saída só apanha quem o consome.*

### §44.4 — ⭐ E o gate comportamental que a W38 declarou impossível passou a ser escrevível

A W38 deixou este buraco escrito: *"sem gate comportamental da costura painel↔smoke — o estado do
smoke é `thread_local` e armá-lo contaminaria os vizinhos"*, e foi por isso que a `next_isolation`
nasceu como lei pura. **A W42 dissolveu essa restrição sem o saber:** desde que desarmar desarma de
verdade, um gate pode armar o módulo, exercer o caminho do artista e **desarmar no fim** — o estado
limpa-se a si mesmo. Os quatro gates desta wave percorrem o gesto inteiro (abrir · pousar · fechar ·
reabrir), e não uma função pura.

⚠️ *Uma restrição escrita numa nota tem uma data.* Aquela era verdadeira quando foi escrita, e
deixou de ser quatro waves depois — pela mesma mecânica da cerca que a W42 encontrou.

### §44.5 — ⛔ O buraco que ESTA wave abriu, e a cura preguiçosa que um gate mau aceitaria

Fazer a vista atravessar o fecho fá-la atravessar também um **Ctrl+O**. E o `isolated` guarda **bits
de entidade**: o mundo novo realoca-os, então um isolamento herdado ou aponta para nada (o
`cook_root` já o larga) ou — pior — **acerta noutro nó**, e a peça nova abre quase toda escondida
sem uma palavra. É a lei da casa sobre bits dentro de bytes, outra vez.

`forget_isolation_across_documents` entra em `project_load` como a **quarta** da família, ao lado de
`forget_owed_poses`, `forget_live_producers` e `field3d_reload::forget_tried` — todas respondem *"o
que o documento anterior possuía e não pode atravessar"*.

⚠️ **A câmera NÃO se esquece aqui**, e o gate diz as duas metades no mesmo teste de propósito: um
gate que só exigisse *ausência do isolamento* passaria com `LAST.set(None)` — a cura preguiçosa, que
deita fora a câmera que a wave inteira existe para guardar. *Uma cura que apaga tudo passa em
qualquer gate que só peça ausência.* A mutação 3 é exatamente essa cura, e o gate apanha-a.

### §44.6 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | desarmar deixa de **lembrar** a vista | `closing_the_panel_keeps_the_view_it_had` |
| 2 | o `boot` ignora o `manual` lembrado (o prato volta a girar) | `the_turntable_stays_stopped_across_a_close` |
| 3 | ⭐ a **cura preguiçosa**: o documento novo larga a vista inteira | `a_new_document_forgets_the_isolation_and_keeps_the_camera` |

Red-first: os três gates de comportamento reprovaram antes da costura existir, e o controle
(`the_first_open_gets_the_default_view`) ficou **verde** — a primeira abertura do app não herda nada,
e é isso que separa *"a memória funciona"* de *"o `boot` mudou de padrão"*.

⚠️ **E o arnês de mutação mentiu na primeira corrida, nas três linhas.** Ele invocava
`cargo test -- --exact <nome_curto>`, e o caminho completo destes gates é
`field3d_smoke::view::tests::<nome>`: com `--exact`, um filtro que não casa corre **zero** testes e o
cargo sai **0**. As três mutações foram declaradas **SOBREVIVENTES** sem que um único teste tivesse
corrido. É a armadilha do filtro escrito à mão que o `CLAUDE.md` §2 já mede (*"797 corridas
devolveram literalmente NADA"*), aqui dentro de uma prova de mutação — o sítio onde ela é mais cara,
porque a saída **parece um resultado**.

⭐ A cura é um **controle positivo** no próprio arnês: além de exigir `Compiling ph2d-host-desktop`
(o binário é novo), exigir `running 1 test` (o teste existiu). *Um veredito de mutação tem de provar
que houve um teste, e não só que houve uma corrida.* ⚠️ A polaridade salvou-nos desta vez — o erro
gritou «sobreviveu» em vez de «RED» —, e a versão simétrica deste mesmo defeito seria **confiança
falsa**, silenciosa.

### §44.7 — ⏸️ O que fica aberto

- ⏸️ Nada mostra na Hierarquia que há um **isolamento em curso** (⏸️ herdado da W38, e agora ele
  também sobrevive a um fecho — o mesmo buraco, com mais alcance).
- ⏸️ A vista é de **processo**, não do projeto: ela não é salva no arquivo. Um Ctrl+O mantém a
  câmera do projeto anterior, o que é o comportamento certo por omissão — mas reabrir o app começa
  sempre do padrão.
- ⏸️ O `snapping` fica deliberadamente de fora (é relido do `Ctrl` a cada movimento) e o gesto em
  curso também: reabrir com um arrasto pendurado aplicaria à peça um movimento que ninguém fez.

---

## §45 — W44: «isolado» é um ESTADO — ele diz-se, e sai-se dele de qualquer sítio (23/08)

> Os dois ⏸️ que a W38 deixou: *"sem tecla"* e *"nada mostra na Hierarquia que há um isolamento em
> curso"*. ⚠️ A W43 tornou o segundo **mais caro**, ao fazer o isolamento sobreviver a um fecho.

### §45.1 — ⛔ A medição encontrou um terceiro item, e é ele o defeito

O que estava na fila era *«um indicador»*. A leitura do código encontrou uma coisa pior: **o único
sinal de isolamento estava preso à SELEÇÃO.**

```rust
active: i == ISOLATE_SLOT && isolated.is_some()
     && isolated == selection.first().map(|e| e.to_bits())
```

| o artista faz | o que a tela dizia |
|---|---|
| isola `A`, olha para `A` | o chip aceso — ✅ correto |
| isola `A`, **escolhe `B`** | o chip **apaga**. Metade da peça fora de vista, e nada o diz |
| isola `A`, **escolhe a raiz** (ou nada) | ⛔ **a fileira inteira desaparece** — nem indicador, nem porta |

⭐ *Um estado da **vista** não se pode anunciar por um controle da **seleção**.* É a mesma família
das cinco costuras mudas que este módulo já pagou, com a variação de estar do lado da **leitura**.

### §45.2 — ⛔⛔ E a porta de saída não cumpria a razão pela qual foi escolhida

A W38 escolheu o *toggle* com a razão escrita ao lado, lida do módulo irmão:

> *"um «sair do isolamento» separado seria uma segunda porta para o mesmo fato, e a que o artista
> não acha quando a cena some."*

⚠️ **O chip não é essa porta.** Ele só é pintado quando o escolhido *se destaca da peça*
(`can_detach`), então com a **raiz** escolhida — o estado em que a peça inteira está selecionada, e
que é justamente para onde alguém vai quando *"sumiu tudo"* — não existe gesto nenhum que devolva a
peça. *A porta escondia-se exactamente no caso em que a cena some.* A cura é a tecla, e ela é a
metade que faltava para a razão de 2026-08-22 ser verdadeira.

### §45.3 — ⭐ Duas leis, porque são duas perguntas

| quem chama | pergunta | `Some(A)` isolado, `B` escolhido |
|---|---|---|
| o **chip numa linha** (`next_isolation`) | *"mostra-me ESTE"* | **troca** para `B` |
| a **tecla** (`key_isolation`) | *"dentro ou fora"* | **sai** |

⛔ Unificá-las com uma bandeira faria uma das duas mentir sobre o gesto que a chamou. É a mesma
disciplina que já separa `toggle_isolate` de `forget_isolation` (*sair* e *o alvo morreu* são fatos
diferentes) — e o gate `the_key_law_is_a_global_in_or_out_never_a_swap` tem um `assert_ne!` a
prender a divergência, para que uma «simplificação» futura reprove em vez de convergir em silêncio.

⚠️ A tecla é **`Shift+I`**, e ela foi **lida** do módulo irmão (`sculpt3d_keys`), não escolhida:
duas janelas 3D no mesmo app com teclas diferentes para o mesmo gesto seria o artista a aprender
duas vezes o que é uma coisa só.

### §45.4 — A voz traz o NOME, e larga o nó morto

O painel publica `isolated: Option<String>` — o **nome**, não um `bool`: *"estás a ver só uma
parte"* deixa o artista à procura de qual. ⚠️ E ela **confirma que o nó ainda existe** antes de o
nomear: o isolamento guarda `Entity::to_bits()`, e um undo respawna tudo com bits novos. O
cozimento já largava o alvo morto (`cook_root`); esta wave faz a **voz** largar com ele — senão ela
anunciaria um nome que já não está na Hierarquia, ou o de outro nó que herdou os bits.

### §45.5 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | a tecla **reaproveita** a lei do chip (troca em vez de sair) | `the_key_law_is_a_global_in_or_out_never_a_swap` |
| 2 | a ponte não drena o pedido da tecla (a porta de saída some) | `an_isolated_part_always_has_a_way_back` |
| 3 | a voz volta a depender da **seleção** | `the_isolation_announces_itself_whatever_is_selected` |
| 4 | a voz não confirma que o nó ainda existe | `a_dead_isolation_is_not_announced` |

⚠️ **A ordem foi cura→gate→mutação, e não red-first**, e isso fica dito: a evidência de que cada
gate *sabe* reprovar veio das quatro mutações, que é a mesma prova feita ao contrário. O arnês
carrega os dois controles positivos que a W43 pagou (`Compiling` **e** `running 1 test`).

### §45.6 — ⚠️ E o clippy do fecho cobrou o preço da regra que o diz

`cargo-check-narrow.sh ph2d-host-desktop --all-targets` ficou **verde** com o campo novo no
`ModelSnapshot`. O clippy do fecho — **derivado do diff**, três crates — encontrou
**sete erros** em `ph2d-panel-model3d/tests/seam.rs`: a crate do painel tem suíte própria, e nela o
retrato é construído **por nome**, nove vezes.

⭐ *Um `-p` escrito à mão mede a minha memória do que toquei.* A regra («o alvo sai do diff, nunca da
cabeça») estava na memória do projeto por causa de um integrador que pagou isto — e aqui ela pagou-se
sozinha, na mesma sessão. ⚠️ E os nove sítios ficam **explícitos**, sem `..Default::default()`: foi
essa suíte que apanhou o campo, e um `..` teria feito exactamente o oposto.

### §45.7 — ⏸️ O que fica aberto

- ⏸️ A **Hierarquia** continua sem marca própria: quem anuncia é o painel do módulo. Uma linha
  esmaecida por nó fora do isolamento seria o sinal no sítio onde a estrutura se lê.
- ⏸️ Isolar **vários** nós de uma vez (o campo guarda um `Option<u64>`).
- ⏸️ As duas frases do aviso são literais em inglês, como as irmãs do canal
  ([`field3d_notice`](../../shells/desktop/src/field3d_notice.rs) é texto livre por desenho — ele
  carrega `explain(&FieldError)`); ⚠️ o **rótulo do painel** passa por i18n, que é onde a HR-15 pega.

---

## §46 — W45: a porta estava trancada por dentro — um projeto que traz uma peça abre o painel dela (23/08)

> O ⏸️ da W35: *"o módulo **não se abre sozinho** quando o projeto carregado traz uma peça"*.

A W35 mediu que a peça **atravessa o arquivo sem uma linha de código de persistência** — ela é uma
árvore de entidades e o `ProjectState` é o mundo inteiro. O que faltava não era guardar: era
**mostrar**. Reabrir o projeto trazia a obra de volta ao mundo, com o painel fechado, e a tela ficava
**vazia**. ⚠️ *Indistinguível de ter perdido o trabalho.*

### §46.1 — ⛔ O mecanismo: a única porta exigia estar do lado de dentro

```rust
pub(crate) fn take_open_panel_request() -> bool {
    if with_smoke(|_| ()).is_none() { return false; }   // ⇐ só pede se o módulo está ARMADO
    PENDING.with(|p| p.replace(false))
}
```

A guarda é **correta** para o que ela protege (o auto-play do smoke não pode abrir o painel em toda
sessão do app, para não mostrar nada). ⛔ Mas o **único** caminho que arma o módulo é a visibilidade
do painel (`set_armed_by_panel`, W42). ⇒ *para pedir a abertura era preciso já estar aberto.*

⭐ A cura é uma **segunda razão** para abrir, que não passa por aquela guarda — e não relaxá-la: o
caso que ela protege continua verdadeiro, e há gate a exigi-lo
(`a_project_without_a_part_opens_nothing`).

### §46.2 — A lei foi LIDA no módulo irmão

> ⚠️ *"Um projeto com escultura **ARMA o módulo**, mesmo sem a env var do smoke — a alternativa seria
> abrir o arquivo, descartar a obra em silêncio e gravá-la fora no save seguinte."*
> — `sculpt3d_doc::sculpt3d_install_pending`

Aqui a obra **não** se perde (o save leva o mundo inteiro). Mas o **silêncio** é o mesmo, e é o
silêncio que o artista lê como perda.

### §46.3 — ⚠️ O load não pode olhar para o mundo, e a forma vem do mesmo sítio

`apply_project` **volta cedo quando não há `gfx`** — e o mundo vive lá dentro. O load é dirigível
sem janela (o `App` nasce com `gfx` em `None`; o winit só a cria no `resumed`), então perguntar
*"há peça?"* no load daria **não** em todo load headless. É a razão que o irmão já tinha escrita, e
a forma é a mesma: **o load deixa a PERGUNTA, o quadro responde-a** — uma vez, quando já há mundo.

### §46.4 — ⚠️ Dois defeitos meus, e os dois eram de POSICIONAMENTO

1. **A condição olhava para o arquivo anterior.** A primeira escrita perguntava `sculpt3d_pending
   .is_none()` **antes** da linha que o atribui a partir deste load — ou seja, sobre o **documento
   anterior**. ⚠️ *Uma condição sobre estado mutável tem um instante, e o instante faz parte da lei*
   — e este erro é **verde em todo gate que abra um projeto de cada vez**.
2. **A função nasceu `#[cfg(test)]` sem eu a marcar.** Ela foi inserida **entre um `#[cfg(test)]` e o
   item dele**, e herdou o atributo — o `sync_scene` de teste ficou sem ele. O compilador disse
   *"cannot find function"* a partir do caminho de produção, que é o sintoma certo e não parece o
   que é. *Um atributo não pertence ao que está por cima dele: pertence ao próximo item.*

### §46.5 — ⚠️ E cede a um projeto que também traz escultura

Os dois querem o canvas, e a lei do dono único (W40) diz que ele é de um só. Quem chegou pelo mesmo
arquivo não se disputa: se o load traz escultura, ela arma o módulo dela e o **MODEL fica a um
clique**. ⏸️ Um projeto que traga os dois continua a exigir uma escolha do artista — e isso está
certo enquanto não houver um sinal que diga *"este arquivo tem as duas coisas"*.

### §46.6 — ⭐⭐ A pergunta certa foi escrita pela MUTAÇÃO, não por mim

A função nasceu a perguntar *"há uma raiz?"*. Corrigi-a a meio para *"há um **nó**?"*, com uma nota
ao lado a explicar a diferença (*"apagar o último filho deixa a raiz de pé"*). ⛔ **A mutação 3
sobreviveu**, e a razão é que as duas perguntas **são a mesma**: o `spawn_doc` dá `FieldNode` à raiz
**sempre** (ela nasce nó e recebe o `FieldObject` depois). *A minha nota descrevia uma diferença
inalcançável.*

⭐ O que separa de facto os dois casos é o **cozimento**: a peça esvaziada coze para `None`. A
pergunta passou a ser ***"há alguma coisa PARA VER?"*** — e a fixtura que a mutação exigiu (apagar as
duas folhas e reperguntar) é a que dá sentido à função.

⚠️ E `Some(Err)` **conta como peça**, de propósito: uma peça que não cozinha é exactamente quando o
artista mais precisa do painel — é lá que o módulo diz **porquê** (W25).

*Uma fixtura que concorda não prova nada; e uma condição cujo caso distintivo é inalcançável é peso
morto com uma nota a mentir ao lado.*

### §46.7 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | o pedido explícito volta a passar pela guarda do armado | `a_loaded_project_asks_to_open_the_panel_even_with_the_module_disarmed` |
| 2 | o load não deixa a pergunta | *(o mesmo)* |
| 3 | *«há alguma coisa para ver»* passa a ser *«há uma raiz»* | `a_world_has_a_part_only_when_there_is_something_to_see` |

⚠️ **E o controle positivo apanhou o arnês outra vez, à primeira corrida:** o caminho dos testes é
`project::tests::field::…` e eu escrevi `project::tests::field_tests::…` — zero testes correram, e
sem a linha `running 1 test` as três teriam sido lidas como sobreviventes. *O caminho de um teste
deriva-se (`cargo test -- --list`), não se escreve de memória.*

### §46.8 — ⏸️ O que fica aberto

- ⏸️ Um projeto com **peça e escultura** abre a escultura; nada diz que também há uma peça.
- ⏸️ O painel abre, mas **não enquadra** a peça: a vista é a lembrada da sessão (W43) ou a padrão, e
  uma peça longe da origem pode nascer fora do quadro. `Home` repõe.

---

## §47 — W46: a peça nasce ENQUADRADA — e o `Home` não sabia onde ela estava (23/08)

> O ⏸️ que a W45 deixou uma hora antes: *"o painel abre, mas **não enquadra** a peça"*.

A W45 fez o painel abrir com o projeto. ⚠️ **Mas uma peça longe da origem abre fora do quadro** — e
aí a tela volta a ficar vazia, que é **exactamente o defeito que a W45 existiu para curar**, com
outra roupa. *Uma cura que deixa o mesmo sintoma alcançável por outro caminho está meia feita.*

### §47.1 — ⛔ E o `Home` não era a saída — eu disse ao Enio que era

No relatório de smoke da W45 escrevi *"aperte Home para centrar"*. **Está errado**, e a medição
mostrou-o:

```rust
pub(crate) fn home(cam: &mut Orbit) {
    …
    cam.target = [0.0; 3];   // ⇐ a ORIGEM, não a peça
}
```

⇒ uma peça a `x = 3` continua fora do quadro **depois** da tecla. *A tecla que existe para desfazer
«estou perdido» era a única que não sabia onde a peça estava.*

⭐ **A lei é a da referência, e nós tínhamos herdado a tecla e metade do significado:** no Blender,
`Home` é *View All* — enquadrar tudo, não repor um ângulo fixo. Agora o `Home` faz as duas coisas, em
ordem: repõe a orientação **e** enquadra.

### §47.2 — ⭐ O bordo não é calculado aqui

`ph2d_field_eval::bounds::bounding_ball` — **o mesmo** que o exportador usa desde a W33. *Duas
réguas para a mesma grandeza é a doença que este módulo já nomeou três vezes*, e a esfera é a moeda
certa porque a composição não a estraga (§34.2).

### §47.3 — A FOLGA foi MEDIDA, e `1,00` não chega

Critério: *nenhum pixel da peça toca a moldura*. Varredura sobre o **pior caso** — uma esfera
sozinha, onde o bordo **é** a silhueta:

| folga | pixels na moldura | fração do quadro |
|---:|---:|---:|
| 0,90 | 252 | 79,3 % |
| 1,00 | **144** | 66,4 % |
| 1,05 | 72 | 60,5 % |
| **1,10** | **0** | **54,6 %** |
| 1,40 | 0 | 32,3 % |
| 1,80 | 0 | 19,0 % |

⭐ **`1,00` não chega, e a razão é a lente:** ela é convergente, e o lado da esfera virado para a
câmera projeta maior do que o raio. Um bordo conservador não compensa isso — ele é conservador no
**mundo**, e o corte acontece na **projeção**.

⚠️ **A varredura só disse isto depois de a fixtura mudar.** A primeira usava a união de duas esferas
e deu **zero pixels na moldura em TODAS as folgas, `0,90` incluída** — porque a união de duas bolas
é muito menor do que a bola que a contém. *Uma fixtura que concorda não prova nada* — a terceira vez
nesta linha, e a segunda hoje.

### §47.4 — ⚠️ O pedido de enquadrar NÃO se tira até ser servido

Os irmãos em `field3d_smoke_requests` são eventos (`Cell::take`). Este não: enquadrar precisa do
documento **cozido**, e no quadro do load ele ainda não existe — o módulo pode nem estar armado. Um
`take` deitaria o pedido fora no primeiro quadro e a peça nasceria onde a câmera anterior calhasse.
*O instante em que um pedido pode ser servido faz parte da forma dele.*

⚠️ E ele **sobrepõe-se à vista lembrada** da W43, de propósito: a câmera lembrada é do documento
anterior, e um documento novo merece o próprio enquadramento.

### §47.5 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | a folga volta a `1,00` (a peça fica cortada) | `the_chosen_margin_cuts_nothing_and_the_one_below_it_does` |
| 2 | a folga foge para `5,00` (não corta, mas a peça vira um ponto) | *(o mesmo)* |
| 3 | enquadrar passa a mexer na orientação | `framing_never_touches_the_orientation` |
| 4 | enquadrar deixa o alvo onde estava | `home_finds_a_part_that_is_far_from_the_origin` |

⚠️ A mutação 2 é a metade que separa *"o número funciona"* de *"qualquer número funcionaria"*: sem a
barra de cobertura, um `FRAME_MARGIN` de 5 passa com a peça a ocupar 2 % do quadro.

### §47.6 — ⏸️ O que fica aberto

- ⏸️ Enquadrar a **seleção** (o `.` do Blender), e não sempre a peça inteira.
- ⏸️ O enquadramento não é **animado**: ele salta. O app tem motor de animação de UI; usá-lo aqui é
  uma decisão de produto, não uma ausência.
- ⏸️ `MAX_HALF_EXTENT = 4,0` (o alcance da marcha) **trunca** o enquadramento de uma peça enorme: ela
  abre cortada e nada o diz. O limite nomeia o seu recurso e está certo; o que falta é a voz.

---

## §48 — W47: as SEIS VISTAS existem, e a câmera passou a ser alcançável (23/08)

> Enio, 23/08: *"siga implementando blocos maiores de features"*. E o item que o plano nomeia como
> **⭐ «É o produto»** desde 19/08: o canvas 3D de primeira classe.

### §48.1 — ⭐ A medição mudou o entregável, e para melhor

O que se ia construir era um **cabeçalho pintado dentro do canvas**, à Blender. Medido antes de
escrever: o módulo **já pinta no viewport inteiro** (`viewport.x/y/w/h`, antes da chrome) — a área
nunca foi o problema. O que falta é ser um **modo com controles**, e:

⛔ **Um cabeçalho dentro do canvas seria uma SEGUNDA superfície de UI do mesmo módulo** — com o seu
hit-test, os seus ids, a sua lei de alcançabilidade, e uma classe de defeito nova (o clique que
orbita em vez de premir). O painel **já é o dono** de *"o que o módulo oferece"*, e a lei da W34 está
escrita sobre ele.

⇒ Os controles de câmera vão para o **painel**. *É a decisão que o próprio plano já tinha tomado* —
a tabela da W2 diz que o canvas *"é trabalho de UI… pertence à wave do painel (W4), junto do resto da
chrome do módulo"*.

### §48.2 — ⛔ O buraco real: a câmera nunca passou pela lei da alcançabilidade

| gesto | como se alcançava, antes desta wave |
|---|---|
| vistas de frente/topo/lado | ⛔ **não existiam** |
| trocar a lente | `Numpad5` — tecla, e só |
| enquadrar | `Home` — tecla, e só |
| isolar | `Shift+I` (W44) — tecla + chip |

A W34 escreveu *"o painel oferece EXATAMENTE o que o gesto faz"* e aplicou-a às fileiras que dependem
da **seleção**. A **câmera** nunca foi auditada por ela: três gestos alcançáveis só por quem já sabia
que existem, e seis que não existiam.

### §48.3 — ⚠️ Lemos as TECLAS do Blender, não os EIXOS

O Blender é **Z para cima**; este módulo é **Y para cima**. Copiar os eixos dele daria uma «frente» a
olhar para o chão. O que se herda é a **memória de dedo**: `Numpad1` frente · `Numpad3` direita ·
`Numpad7` topo, com **`Ctrl`** a dar a oposta de cada uma (`Numpad5` já era a lente desde a W15,
como lá).

⚠️ **A orientação é escrita em `yaw`/`pitch`** — a porta que a casa já tinha, e cujo doc **já previa
esta wave à letra**: *"é como se escreve um enquadramento nomeado (o inicial, a vista de frente, a de
topo)"*. A função existia e ninguém a tinha chamado para isso.

⭐ E o gate não confere a aritmética: confere o **eixo do olho** (`Orbit::basis`). Um sinal trocado dá
uma vista chamada *Frente* que mostra as **costas**, e nenhum teste de forma notaria — a peça
aparece, inteira e enquadrada, do lado errado.

### §48.4 — ⭐ A vista é um FATO DERIVADO, nunca um modo guardado

`named_view(cam)` responde olhando para a **orientação**. Guardar *"estou em Frente"* daria um chip
aceso sobre uma vista que o artista já torceu — o espelho de estado a mentir, que este módulo já
pagou no cache do traçado.

### §48.5 — ⛔ A tolerância que eu escrevi contradizia-se, e o gate apanhou-a à primeira

Primeira escrita: `RECOGNISE = 1e-4`, com a justificação *"~1,6° de desvio; um arrasto de utilizador é
sempre maior do que isso (0,57° por pixel)"*. ⚠️ **0,57° é MENOR que 1,6°** — a frase carregava os
dois números e eu não os comparei. O gate reprovou na primeira corrida.

O número medido, com o que ele tem de separar:

| | `1 − \|q·q′\|` | em graus |
|---|---:|---:|
| re-normalizar (o ruído de `f32`) | ~1e-7 | ~0,05° |
| **a barra** | **1e-6** | **0,16°** |
| **um pixel** de arrasto | 1,25e-5 | 0,57° |

⇒ uma ordem de grandeza acima do ruído, **12×** abaixo do menor gesto que existe. E o gate prende as
**duas** margens: re-normalizar não solta a vista, um pixel solta.

*Uma justificação com dois números só vale depois de os comparar.*

### §48.6 — ⚠️ Uma lei de alcançabilidade tem uma RÉGUA POR ESPÉCIE DE GESTO

A lei da W34 mede *"a intenção muda o **documento**"*. As fileiras de câmera **não podem** mudar o
documento: olhar a peça de frente não é uma edição, não entra no undo, não viaja no arquivo. Medi-las
por aquela régua daria **todas mudas** — e a conclusão errada seria *remover os botões*.

⇒ o gate novo mede a **câmera**: cada chip de vista põe a orientação prometida **e enquadra**; o da
lente troca a lente **e o retrato di-lo**; o de enquadrar traz o alvo à peça.

### §48.7 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | a *Frente* põe o olho no eixo errado | `every_named_view_puts_the_eye_on_the_axis_its_name_promises` |
| 2 | o produto interno perde o módulo (`q` ≠ `−q`) | `the_negated_quaternion_is_the_same_view` |
| 3 | a tolerância volta a `1e-4` | `a_named_view_is_recognised_and_the_smallest_drag_lets_it_go` |
| 4 | o `Ctrl` deixa de dar a vista oposta | `the_keys_are_the_reference_ones_and_ctrl_gives_the_opposite` |
| 5 | o chip aceso vira modo guardado | `the_lit_view_chip_goes_out_when_the_camera_leaves_it` |
| 6 | o chip da vista **não enquadra** | `every_camera_chip_moves_the_camera` |

⚠️ **A 6 SOBREVIVEU à primeira corrida**, e apanhou um buraco real: o gate conferia a orientação
depois do `SetView` e **nunca o enquadramento** — e a fixtura, centrada na origem, nunca o
denunciaria sozinha. A cura foi levar o alvo para `[9,9,9]` antes de cada chip. *Terceira vez nesta
linha em que uma fixtura que concorda escondeu metade da lei.*

### §48.8 — ⏸️ O que fica aberto

- ⏸️ **O canvas continua sem cabeçalho próprio** — a decisão foi deliberada (§48.1), mas o nome da
  vista atual só se lê no painel. Um rótulo no canto do canvas seria uma **segunda voz** para o mesmo
  fato; se um dia entrar, o chip tem de sair.
- ⏸️ Não há vista **oposta rápida** (o `Numpad9` do Blender) nem *"enquadrar a seleção"* (o `.`).
- ⏸️ As vistas **não forçam a lente paralela**. É a lei da referência (ela mantém a lente), mas para
  um modelador que use as vistas para medir, ortográfica seria o esperado — decisão de produto.
- ⏸️ Dividir a janela em várias vistas (o quad-view) continua fora: é docking, não modelagem.

---

## §49 — W48: «nenhum botão funcionou» — o quinto sítio, e o gate que eu escrevi a cometer o pecado que ele condena (23/08)

> Enio, 23/08, no smoke da W47: *"nenhum botão funcionou."*

### §49.1 — O mecanismo, em uma linha

`populate()` regista cada família de botões no `WidgetStore`. A W48 acrescentou duas fileiras e eu
toquei **quatro** dos cinco sítios que um controle de painel precisa:

| sítio | toquei? |
|---|---|
| o campo no retrato (`state.rs`) | ✅ |
| a linha no `paint.rs` | ✅ |
| o braço no `event.rs` | ✅ |
| a família de ids | ✅ |
| **o `populate.rs`** | ⛔ **não** |

⇒ os chips pintavam, o índice de acerto tinha-os, o clique caía em cima deles — e `apply_click` faz
`match store.get_mut(id)`, que devolve `None` para um id não registado. **O evento nunca nascia.**

⚠️ *«Registro de painel (5 sites)»* é uma memória do projeto, com o número no título.

### §49.2 — ⛔⛔ E os meus gates estavam todos verdes, pelo motivo que o arquivo deles condena

Os gates da W47 empurram a intenção por `push_intent_for_test`. A frase que os invalida está escrita,
**à letra**, no topo do `field3d_reach_tests.rs` — o arquivo onde eu os acrescentei:

> ⛔ **Isso prova o TRATADOR, nunca a ALCANÇABILIDADE.** Empurrar a intenção é encenar um clique que
> o artista não tem como dar — e um clique impossível passa em qualquer teste que o simule.

E o cabeçalho do `tests/seam.rs` já nomeava as **três** causas exatas, incluindo esta:

> *"um braço em falta em `event.rs`, um id fora da família ou **uma leitura errada do store**
> deixariam o controle pintado, arrastável e silenciosamente morto, com todos aqueles testes
> verdes."*

*O arnês certo existia, apontado a este defeito, e eu escrevi o gate errado ao lado dele.*

### §49.3 — ⭐ Duas leis, uma por vão da costura

A costura de um controle tem **dois** vãos, e um gate só cobre um:

| vão | lei | o que ela apanha |
|---|---|---|
| pintado ⇒ **evento** | `every_painted_button_answers_a_real_click` | o `populate` esquecido |
| evento ⇒ **intenção** | `a_click_on_a_camera_chip_dispatches_that_exact_slot` | o braço em falta, ou o slot errado |

⭐ **A primeira não tem lista de famílias**: ela varre *o que o painel de facto registou ao pintar*
(`host.paint` devolve os retângulos) e exige que cada um responda a um **clique de verdade**
(`host.click_at`). Uma fileira nova entra na varredura **sozinha**, no dia em que for pintada.

⚠️ **E ela confere a IDENTIDADE do evento, não só que saiu algum.** Na corrida red-first apareceram
**3** mudos, não 4: um dos chips estava a ser dado por vivo pelo evento de um registo **vizinho** que
se sobrepõe. *Um gate que só conta eventos aceita o do vizinho.*

### §49.4 — A cura estrutural: a lista num sítio só

Enquanto o `populate` eram sete `store.register` copiados, esquecer o oitavo era a coisa mais natural
do mundo — sem erro de compilação, sem teste a notar, e com um sintoma que é um botão bonito e morto.
Agora é um `CHIP_FAMILIES` sobre o qual o `populate` itera: **acrescentar a família é registá-la**.

### §49.5 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | o `populate` volta a esquecer as **vistas** (o defeito do Enio) | `every_painted_button_answers_a_real_click` |
| 2 | …e a **câmera** | *(o mesmo)* |
| 3 | o braço despacha sempre o slot `0` | `a_click_on_a_camera_chip_dispatches_that_exact_slot` |
| 4 | o braço deixa de existir | *(o mesmo)* |

---

## §50 — W49: o GIZMO DE NAVEGAÇÃO — e a pesquisa que o Enio mandou fazer antes (23/08)

> Enio, 23/08: *"não seria melhor criar um gizmo moderno como o do Fusion para isso? […] Antes de
> construir, melhor fazer uma pesquisa de qual é o melhor entre os app e usar como ref."*

### §50.1 — A pesquisa, e os dois factos que decidiram

| | quem usa | gesto |
|---|---|---|
| **ViewCube** (cubo) | Fusion 360, AutoCAD, Inventor, Maya, Onshape, SolidWorks | clica face/aresta/canto; arrasta orbita |
| **bolas de eixo** | Blender, Unity, Godot, Cinema 4D, Plasticity | clica a bola; arrasta orbita |

⛔ **1. O ViewCube está sob patente VIVA.** [US 7.782.319](https://patents.google.com/patent/US7782319B2/en)
— *"Three-dimensional orientation indicator and controller"*, Autodesk, depositada **2007-03-28**,
estado **«Active — expires 2029-03-06»**, com família à volta
([US 8.314.789](https://patents.google.com/patent/US8314789B2/en),
[US 9.021.400](https://patents.google.com/patent/US9021400)). ⚠️ Uma patente cobre o que a
reivindicação diz, não «cubos em geral» — mas isto é um facto que se apura **antes** de construir, e
a decisão é do dono do produto.

⭐⭐ **2. O que faz o widget funcionar não é o formato — é o ARRASTO.** A própria pesquisa da Autodesk
que criou o ViewCube mediu os utilizadores *"quase 2× mais rápidos"* a arrastar do que a clicar,
**«independentemente das várias representações examinadas»**
([paper](https://www.research.autodesk.com/publications/viewcube-a-3d-orientation-indicator-and-controller/)).
⇒ o ganho medido **não vem do cubo**; vem de o widget ser uma alça que se puxa.

**Decisão do Enio, com os dois factos na mão: bolas de eixo** — e por isso aqui *arrastar orbita* é o
caminho principal e o clique é o secundário, não o contrário.

### §50.2 — ⚠️ Os eixos são os NOSSOS

Este módulo é **Y para cima**; o Fusion e o Blender são Z para cima. Cada bola **é** uma `Standard`
da W47 — a mesma lista, a mesma lei de reconhecimento —, então não há uma segunda ideia de *"o que é
a vista de frente"*.

### §50.3 — Os números são DERIVADOS, não escolhidos

| | de onde vem |
|---|---|
| raio da bola | `field3d_gizmo::GRAB_PX` — *"a que distância do traço um clique ainda é daquela alça"* |
| braço do gizmo | `4 × raio da bola` — dois eixos perpendiculares ficam a `5,7×` o raio |
| espessura do anel e do talo | `field3d_gizmo::SHAFT_HALF_W_PX` |
| cores | tokens `axis-x/y/z`, os mesmos das setas do gizmo 3D |

⭐ *O widget fala a mesma língua que as setas que o artista já usa* — se as duas discordassem, a cor
deixaria de ser legenda.

### §50.4 — ⭐⭐ As duas mutações que sobreviveram encontraram buracos REAIS nos gates

**1. Espelhar o widget na vertical passava em tudo.** Trocar o sinal do `y` da projeção e **nenhum**
gate reprovava: a bola do meio fica no meio de qualquer forma, e a lei do «cabe na área» é simétrica.
⚠️ Um gizmo espelhado é a pior falha possível dele — *ele diz uma orientação e a tela mostra outra,
com toda a confiança*. Gate novo: `up_on_screen_is_up_in_the_world`, com a vista de frente como régua.

**2. A guarda `nav.is_none()` era CÓDIGO MORTO**, e a mutação foi quem o disse. Para chegar àquele
ramo é preciso `nav.filter(still)` ser `None` — isto é `nav.is_none() || !still` — e o `still`
exigido lá dentro colapsa isso em `nav.is_none()`. ⇒ a cura foi **apagá-la**, não inventar um gate
para ela. *Uma condição que não pode mudar o resultado é uma afirmação falsa sobre o código para quem
o ler a seguir* — e ela sobrevive a toda mutação, por construção.

### §50.5 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | o `pick` olha de trás para a frente (responde pelo eixo escondido) | `a_click_where_two_balls_overlap_takes_the_front_one` |
| 2 | as bolas deixam de ser ordenadas por profundidade | `the_view_we_are_in_is_the_ball_at_the_centre_and_the_frontmost` |
| 3 | o `y` da tela deixa de ser invertido | `up_on_screen_is_up_in_the_world` |
| 4 | o gesto deixa de lembrar a bola | `clicking_a_navball_takes_the_camera_to_that_view` |
| 5 | o clique ganha do arrasto | `dragging_from_the_navball_orbits_instead_of_snapping` |

⭐ **E os gates são de COSTURA, não de tratador** — a lição que a W48 cobrou: o caminho é
`begin` → `advance` → `finish`, pelo ponteiro, sem uma intenção empurrada à mão.

### §50.6 — ⏸️ O que fica aberto

- ⏸️ **Sem letras nas bolas.** O Blender escreve X/Y/Z nas positivas; aqui a cor é a única legenda.
  Escrevê-las é uma linha de i18n por eixo — ficou de fora para não decidir sozinho se um símbolo
  matemático passa pelo i18n.
- ⏸️ **Sem cantos nem arestas**: as bolas dão as seis vistas retas; a de três-quartos volta com o
  `Home`. O cubo dá 26 direções, e é a única coisa em que ele ganha.
- ⏸️ O salto para a vista **não é animado** (o mesmo ⏸️ da W46).
- ⏸️ O widget não tem gate de **pintura** — a lei mede posição, ordem e clique; que a bola positiva
  saia cheia e a negativa vazada é lido por olho.

---

## §51 — W50: a moldura do app EMPURRA o gizmo — e o arnês de mutação mentiu com um vermelho (23/08)

> Enio, no smoke da W49: *"funcionou bem mas veja que fica escondido entre botões. Quando houver
> painel à direita melhor deslocar o gizmo para esquerda e abaixar um pouco para não sobrepor os
> botões superiores."*

### §51.1 — A causa

A área que o módulo recebe é o **viewport inteiro** — e a moldura do app (a faixa de botões do topo,
os painéis da direita) é pintada **por cima** dele. Pôr o gizmo na quina daquela área é pô-lo
**debaixo** da moldura.

⇒ o gizmo passa a viver na **parte livre**: a área menos o que a moldura de facto cobre, calculada de
retângulos que o shell publica (`hero.store.panel_rects()` e o índice de acerto da faixa) e por uma
lei **pura** no módulo (`field3d_navball::safe_corner`).

### §51.2 — ⭐ A FUGA MAIS BARATA, e por que a primeira lei estava errada

A primeira escrita classificava por *«toca a aresta»*: quem toca a direita empurra para a esquerda,
quem toca o topo empurra para baixo. ⛔ **Um painel da altura toda toca as DUAS** — ele contava como
faixa do topo e empurrava o gizmo 600 px para baixo. O gate reprovou à primeira corrida.

⭐ A lei certa não precisa de saber o que o obstáculo **é**: para cada um, sair pela direita ou pelo
topo tem um preço, e toma-se o menor. Um painel encostado à direita é **estreito e alto** — sair por
ela custa 300 px e pelo topo custa a janela; numa faixa do topo é ao contrário. *A forma do
obstáculo diz por onde se sai dele.*

⚠️ E é **iterativo**: escapar de um obstáculo põe a caixa do widget noutro sítio, onde pode haver
outro. Há gate a exigir que **a ordem dos obstáculos não mude o resultado** — uma lei iterativa
dependente da ordem daria um gizmo que salta conforme o painel que abriu primeiro.

⚠️ **Localidade:** só conta quem se sobrepõe à caixa onde o widget **está**. Sem isso, a tira de
quadros do Flip (larga, encostada à direita, lá em baixo) empurraria o gizmo pela largura inteira da
janela. Gate: `a_bottom_strip_does_not_move_the_gizmo`.

### §51.3 — ⚠️ Um acessor novo em vez de uma segunda lista

`WidgetStore::panel_rects()` — todos os retângulos publicados neste quadro, sem lista de ids. A
alternativa era copiar a lista de ~25 painéis que o `cursor_over_hero_panel` já carrega, e **uma
lista que é preciso lembrar é uma lista que se esquece** (a lição da W48, no mesmo módulo e no mesmo
dia). Um painel flutuante no meio do canvas não move o gizmo — a lei da localidade trata dele.

### §51.4 — ⛔⛔ O ARNÊS DE MUTAÇÃO DEU POR APANHADA UMA MUTAÇÃO QUE NÃO APANHOU

O gate de costura que eu escrevi chamava `note_safe(...)` **de dentro** de um `with_smoke` — e
`note_safe` entra pelo mesmo `with_smoke`. `RefCell` re-entrante: o teste **entrava em pânico
sempre**, mutado ou não.

⚠️ E o arnês declarou `RED (recompilou: True · correu 1 teste: True)`. **Os dois controles positivos
que a W43 e a W45 pagaram passaram os dois** — houve compilação, houve um teste, e ele ficou
vermelho. *Só que ficaria vermelho de qualquer maneira.*

⭐ A regra que faltava já existia na memória do projeto — *"RED só conta sobre algo visto VERDE
antes"* — e o que faltava era ela estar **no arnês**, não na cabeça de quem o corre. Agora ele corre
o teste **antes** de mutar e exige verde; a linha de saída diz `verde antes: True`.

⇒ E com o verde exigido, a verdade apareceu: **a mutação 4 sobrevive mesmo**. A linha que publica a
parte livre vive no laço de quadro e **não é alcançável de um gate** (precisa de janela e da moldura
pintada). Ela sai da tabela e entra nos ⏸️ — *declarada, não coberta*.

### §51.5 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | a fuga mais barata vira a mais cara | `the_chrome_pushes_the_gizmo_left_and_down` |
| 2 | a lei deixa de iterar | *(o mesmo)* |
| 3 | a lei perde a localidade | `a_bottom_strip_does_not_move_the_gizmo` |

### §51.6 — ⏸️ O que fica aberto

- ⏸️ **A chamada que publica a parte livre não tem gate** (§51.4) — o gate alcança da porta do
  módulo (`note_safe`) para a frente; a linha no laço de quadro é do mesmo tipo do «Ctrl+S de
  verdade» da W35.
- ⏸️ O gizmo **não se move quando o painel é arrastado**… move-se, e a cada quadro: um painel a ser
  arrastado leva o gizmo com ele. É o comportamento certo, mas **não é animado**.

---

## §52 — W51: a VIAGEM entre vistas — e a curva não é minha (23/08)

> Enio, 23/08: *"falta um Lerp() rápido para mudança suave das views como no blender."*

### §52.1 — ⭐ A curva e a duração são as da CASA

A casa tem um sistema de movimento por **mola**, com carácter (`Discrete`/`Expressive`), com
*reduced motion*, e com **papéis** que dizem o que uma coisa **é** (`ph2d_editor::motion::Role`).
Inventar aqui uma duração e uma curva seria uma segunda ideia de *como as coisas se mexem neste app*.

⭐⭐ **O papel é `Role::Surface`**, e o doc dele descreve este caso à letra:

> *"Viaja (o reduced motion mata-a) e **nunca ultrapassa**, nos DOIS carácteres. […] Uma roda nomeia
> um **destino**, e passar dele e voltar não lê como peso — lê como a régua a mentir."*

**Uma vista nomeada é um destino.** Com `Role::Travel` (que ultrapassa 15,5% no Expressivo) a peça
passaria da frente e voltava — a janela inteira a balançar.

⚠️ E o **reduced motion sai de graça**: ali a lei é `None`, o progresso chega a `1` no primeiro
quadro, e a viagem vira o salto que existia antes desta wave. *Uma animação que ignora essa
preferência é um defeito de acessibilidade, e a casa já tem a preferência.*

### §52.2 — As três grandezas viajam de maneiras DIFERENTES

| grandeza | interpolação | porquê |
|---|---|---|
| orientação | **slerp**, caminho curto | é uma rotação |
| alvo | linear | é um ponto |
| enquadramento | **geométrico** | o zoom deste módulo é multiplicativo (`ZOOM_PER_STEP`, *"para que cada passo aproxime a mesma fração"*) — linear dispara longe e rasteja perto |
| lente | **do destino, já** | não há meia-lente |

⚠️ **O caminho curto tem gate próprio**, medido pelo **comprimento do percurso**: sem o `dot < 0`,
metade das viagens dá a volta pelo lado comprido — 300° para chegar a um sítio a 60°.

### §52.3 — ⭐ Chegar EXATAMENTE ao destino é exigência, não detalhe

O chip da vista acende por `named_view`, que reconhece a orientação com uma barra de **0,16°**. Uma
viagem assintótica pousaria *perto* e o botão **nunca acenderia** — com a peça no sítio certo à
frente do artista. O sistema da casa já resolve isto (*"assentar põe o valor EXACTO e larga o voo"*)
e esta metade honra-o: em `t ≥ 1` a câmera é o destino, **escrito**, não interpolado.

### §52.4 — ⭐ UMA porta, e não cinco

Os cinco caminhos que escolhiam uma vista escreviam `s.cam.rotation` **cada um por si** (a tecla, o
chip do painel, a bola do gizmo, o `Home`, o chip *Frame*). Enquanto fossem cinco escritas, uma delas
ia ficar a saltar — e o defeito leria como *"às vezes é suave, às vezes não"*, que é dos mais
difíceis de acreditar. Todas passam agora por `fly_to`.

⚠️ **E a mão CANCELA**: orbitar, deslocar, aproximar ou agarrar uma alça larga o voo. É a lei que o
módulo já aplica ao refinamento do preview (*"um refinamento cede à mão"*) — uma câmera a viajar por
baixo de um arrasto é o app a disputar o rato.

### §52.5 — ⚠️ O `RefCell` re-entrante, DUAS vezes no mesmo dia

`with_smoke` pega o `RefCell` do estado, e chamá-lo de dentro de outro `with_smoke` é um
`borrow_mut` re-entrante: **pânico**, não erro de compilação. A W50 pagou-o num gate de costura (e
lá o arnês de mutação chegou a dar isso por «mutação apanhada»); esta wave voltou a pagá-lo **na
hora seguinte**, nos gates da viagem.

⭐ A cura desta vez é **estrutural, não memória**: o corpo (`advance_flight(&mut Smoke, t)`) separado
da porta (`note_flight_progress(t)`), que é o padrão que o módulo já usa no `finish`/`finish_for_test`.
*Quando uma porta de módulo tem de ser chamada de dentro dele, a cura é o corpo separado — não
lembrar-se.*

### §52.6 — ⚠️ Três gates existentes mudaram, e passaram a exigir MAIS

`clicking_a_navball_takes_the_camera_to_that_view`, `every_camera_chip_moves_the_camera` e
`the_lit_view_chip_goes_out_when_the_camera_leaves_it` afirmavam que a câmera estava **já** na vista.
Com a viagem, isso deixou de ser verdade no mesmo quadro.

⛔ *Atualizar um gate para caber no comportamento novo é como uma regressão se encobre.* Por isso eles
passaram a exigir **as duas** metades: que a viagem foi **pedida** (a câmera **não** salta) **e** que
ao terminar ela pousa exatamente na vista. A alegação visível ao artista fica intacta, e a nova junta-se.

### §52.7 — Provas de mutação (todas com **verde antes**, a lição da W50)

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | o slerp perde o caminho curto | `the_trip_takes_the_short_way_round` |
| 2 | a viagem não aterra exatamente | `the_trip_lands_exactly_on_the_destination` |
| 3 | o enquadramento viaja linear | `the_framing_travels_in_fractions_not_in_units` |
| 4 | a viagem não larga o voo ao chegar | `clicking_a_navball_takes_the_camera_to_that_view` |
| 5 | a mão deixa de cancelar | `the_hand_cancels_a_trip_in_flight` |
| 6 | cada viagem reusa a track anterior | `each_trip_gets_a_fresh_track_id` |

### §52.8 — ⏸️ O que fica aberto

- ⏸️ A linha do laço de quadro que serve o progresso **não é alcançável de um gate** (o mesmo ⏸️ da
  W50, e o mesmo tipo do «Ctrl+S de verdade» da W35).
- ⏸️ **O traçado corre a cada quadro da viagem** — é o mesmo custo de orbitar à mão (a resolução do
  preview já cede ao movimento, W24), mas ninguém o mediu especificamente para a viagem.

---

## §53 — W52: a viagem NÃO é do *reduced motion* — e o papel dela nasceu disso (23/08)

> Enio, no smoke da W51: *"funciona. Mas o lerp não deve estar vinculado ao Reduced Motion. Mas deve
> ser o único modo."*

### §53.1 — O smoke anterior leu «não funcionou», e o código estava certo

A W51 shipou com `Role::Surface` — que **morre** no *reduced motion* —, e a preferência do Enio
(`~/.ph2d/prefs.txt`) diz `reduced_motion=1`. Com ela ligada a lei é `None`, o progresso chega a `1`
no primeiro quadro, e a viagem **é** o salto de antes. O report dele foi *"não funcionou. está como
antes"*, e era a descrição exata do comportamento correto.

⚠️ **A armadilha estava anotada** no `CLAUDE.md` §5, com estas palavras: *"um `reduced_motion=1`
esquecido **reprova smokes sobre produto correto**"*. Eu tinha a nota, ia pedir um juízo **sobre
movimento**, e não li o arquivo antes de escrever o relatório. *Uma condição que decide se o smoke
faz sentido não é um rodapé — é o passo 1, e é verificável com um comando.*

### §53.2 — ⭐ A cura é um PAPEL novo, não uma excepção

O `motion.rs` prega que *"o `Role` diz o que a coisa É"*, e a própria `Role::Surface` nasceu assim
(*"este é o terceiro membro dessa família, não uma excepção nova"*). Então a decisão do Enio vira
`Role::Viewpoint`, com o critério escrito ao lado:

> ⭐ **Aqui o CORTE é pior do que o movimento.** O gatilho vestibular que o *reduced motion* existe
> para apagar é uma área grande a **deslocar-se** — mas o que fica no lugar desta animação não é
> sossego, é um **salto**: a cena inteira troca de orientação entre dois quadros, sem uma pista de
> continuidade. É exactamente o momento em que alguém se perde. *Tirá-la não devolve a calma —
> devolve o corte.*

⛔ **E o critério é estreito de propósito**, para não virar uma porta de *«a minha animação é
especial»*: só passa o que, ao ser removido, deixa **um corte que desorienta mais do que o
movimento**. Uma decoração, um realce ou um painel a deslizar não passam — o que os substitui é a
coisa **já no sítio**, que é sossego a sério. Há gate a exigir que um percurso **comum** continue a
morrer.

⚠️ Ele **não ultrapassa** nos dois carácteres (reusa a rigidez da `Surface`): um ponto de vista é um
**destino**, e passar dele e voltar seria a janela inteira a balançar.

### §53.3 — ⚠️ O papel é NOMEADO do lado do módulo

`field3d_flight::ROLE`. Enquanto ele vivesse só na linha do laço de quadro, a alegação que interessa
ao artista — *"a viagem acontece mesmo com o movimento reduzido ligado"* — não teria gate nenhum:
aquela linha é do tipo que um teste headless não alcança (o mesmo ⏸️ da W50). Com o papel nomeado
deste lado, o gate mede-o.

### §53.4 — Provas de mutação (com **verde antes**)

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | a viagem volta ao papel que o *reduced* mata (o smoke da W51) | `the_trip_survives_reduced_motion` |
| 2 | o `Viewpoint` passa a morrer no *reduced* | *(o mesmo)* |
| 3 | a excepção vira uma **porta** (o *reduced* deixa de matar um percurso comum) | *(o mesmo)* |
| 4 | o ponto de vista passa a ultrapassar | `the_trip_never_overshoots_its_destination` |

### §53.5 — ⏸️ O que fica aberto

- ⏸️ O `Viewpoint` é hoje usado **só** por este módulo. A janela 3D da escultura tem o mesmo
  problema e a mesma resposta — mas ligá-la é decisão da linha dela.

---

## §54 — W53: o PERFIL DESENHADO vira peça — uma família de features completa e invisível (23/08)

### §54.1 — ⛔ O achado, e o tamanho dele

`Primitive::Extrude` e `Primitive::Revolve` existem no motor **desde a W3**, medidos contra oráculos
independentes (um `n`-gono extrudado **é** o `Cylinder` analítico; um revolvido **é** o `Torus`,
errando pela flecha exata que a geometria prevê), com o arredondamento das quinas verticais a vir do
*corner widget* do editor vetorial. O plano do módulo chama-lhes a **razão de existir**:

> *"É aqui que o fluxo do MoI renasce, com a caneta que a casa já tem."*

⛔ **E nenhum botão os alcançava.** Só as cenas de smoke os construíam. A `SHAPES` do painel tinha
seis entradas e **nenhuma** era o perfil.

### §54.2 — ⚠️ Por que o gate da alcançabilidade não a apanhou

A lei da W34 tem uma **exclusão escrita**, e ela é razoável:

> *"Só as que dependem da seleção: operações, modificadores e ações. As formas (`adds`) […] são ações
> sempre disponíveis."*

⇒ a fileira das formas nunca foi medida, e a pergunta que faltava não é a daquela lei. A daquela é
*«o painel oferece o que a seleção permite?»*; a que faltava é ***«o painel oferece tudo o que o
MOTOR sabe fazer?»***. Uma exclusão correta numa lei escondeu a ausência de outra.

⭐ Gate novo: `every_primitive_the_engine_can_make_has_a_button`, com **as duas metades** — cada
primitiva tem botão, **e** o painel não promete formas que o motor não tem.

### §54.3 — A ponte já existia inteira

`ph2d_field_profile::cook_path_auto(&VecPath) -> Profile` faz a travessia toda, quinas vivas
incluídas. **Esta wave não escreveu geometria nenhuma** — escreveu o **gesto**:

| peça | de onde veio |
|---|---|
| cozer o contorno | `cook_path_auto`, já existia (W3) |
| o contorno escolhido | `blend_live::selected_closed_in_z`, já existia |
| os três saltos até ao mundo | o padrão da escultura importada (W22) |
| a escala pelo enquadramento | `field3d_import::framing_scale`, já existia |

### §54.4 — As decisões que foram tomadas, e não herdadas

- **Os botões só aparecem com um contorno FECHADO escolhido** (a lei da W34): *Extrude* sem nada
  para extrudar é a affordance que mente.
- **A altura sai da extensão do CONTORNO**, não do enquadramento: a espessura de uma peça extrudada é
  uma proporção da forma dela — uma cantoneira de 10 cm não tem 3 m de espessura. O tamanho de
  **convivência** sai da pose, como na escultura.
- **Aro vivo por omissão** (`round: 0`): o filete do aro é uma linha do painel, e o das quinas
  **verticais** já veio do editor vetorial. *Uma quina, um dono.*
- **O erro é traduzido**, não repassado: `Rejected(SelfIntersecting)` no ecrã é o mesmo que silêncio
  para quem está a modelar — a lei que o `field3d_notice` já carrega.

### §54.5 — ⭐ Dois gates existentes reprovaram, e os dois estavam a trabalhar

- `the_scene_sculpture_button_appears_only_when_there_is_one` contava a lista. **Correção honesta:**
  o novo sinalizador fica **constante e ligado** nos dois lados — um gate que mede a filtragem da
  escultura não pode mudar de significado quando outra feature entra.
- `the_sculpt_slot_points_at_the_sculpt_button` exige que todo slot que não é exceção produza uma
  primitiva. ⭐ **E o comentário dele previa isto à letra:** *"uma porta nova entra aqui ou o gate
  reprova, que é a ordem certa"*. As formas de perfil também não são construíveis a partir de um
  raio — precisam do contorno —, e ele reprovou no minuto em que entraram.

### §54.6 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | o motor volta a ter uma forma **sem botão** | `every_primitive_the_engine_can_make_has_a_button` |
| 2 | os botões aparecem sem contorno escolhido | `the_profile_buttons_appear_only_with_a_closed_outline_selected` |
| 3 | dois slots derivados colidem | `the_four_derived_slots_are_distinct_and_in_range` |

⚠️ **A mutação 1 mentiu à primeira**: apagar uma entrada da `SHAPES` faz o array de tamanho fixo
**não compilar**, e o vermelho vinha daí — `correu 1 teste: False`. O controle positivo apanhou-a. A
mutação expressiva **troca a chave** e mantém o tamanho.

### §54.7 — ⏸️ O que fica aberto

- ⏸️ **Um contorno de cada vez**: com vários escolhidos, é o primeiro em z. Vários perfis numa peça
  só (ou furos como contornos interiores) pede uma decisão de produto.
- ⏸️ O nó **não se religa** ao desenho: editar o contorno depois não muda a peça (o perfil é cozido
  uma vez). É a mesma escolha da escultura importada, e a mesma ⏸️.
- ⏸️ O *Revolve* gira em torno de **Y** e um contorno com `x < 0` é recusado pelo documento — o aviso
  di-lo, mas nada mostra o eixo **antes** do clique.

---

## §55 — W54: a régua da suavidade é a NORMAL, não a silhueta — e a tabela velha estava desmentida por 2,4× (23/08)

> Enio, no smoke da W53, com duas fotos: *"contudo sem ajustes de resolução"* — silhueta lisa, **luz
> em degraus**.

### §55.1 — ⭐ A minha primeira hipótese estava errada, e a aritmética disse-o antes do código

Suspeitei da polilinha do perfil, e a conta refutou na hora: a tolerância que shipava erra a silhueta
em **0,079 % da peça**. Isso é invisível — não podia ser o que ele estava a ver.

⭐ Olhando as fotos outra vez: **a silhueta está lisa e as bandas estão na LUZ.** O campo de um
contorno poligonal tem **gradiente constante por segmento**, então a normal salta de faceta em
faceta — e o olho lê um salto de normal muito antes de ler um erro de posição.

### §55.2 — A tabela (círculo r=0,5; 640×480; mediana de 7; `load < 3`)

| fração | arestas | flecha (% de D) | **salto de NORMAL** | extrusão | torno | px com degrau |
|---:|---:|---:|---:|---:|---:|---:|
| `1e-3` **(o que shipava)** | 56 | **0,079 %** | **6,43°** | 53,3 ms | 65,5 ms | **4 417** |
| `3e-4` | 96 | 0,027 % | 3,75° | 92,5 ms | 104,7 ms | 6 765 |
| **`1e-4` (shipa)** | **168** | 0,009 % | **2,14°** | **139,3 ms** | **178,8 ms** | **79** |
| `3e-5` | 305 | 0,003 % | 1,18° | 237,3 ms | 286,0 ms | 86 |
| `1e-5` | 528 | 0,0009 % | 0,68° | 409,3 ms | 511,1 ms | 70 |

⭐ **`1e-4` é o joelho:** os degraus caem **56×** (4 417 → 79 pixels) e o passo seguinte custa
**+70 %** para não melhorar nada (79 → 86, ruído). O custo é **linear**: ~1 ms por aresta.

⚠️ **A coluna dos degraus é acoplada ao limiar** com que é contada (3° entre vizinhos), e **diz isso
alto ao não ser monótona**: em `3e-4` há mais facetas e cada uma ainda passa dos 3°, então o número
**sobe** antes de colapsar. Quem decide é o **salto de normal**; a contagem ilustra o cruzamento, não
o prova. E o limiar de 3° veio **das duas fotos do Enio** — um oráculo de aparência, declarado como tal.

### §55.3 — ⛔⛔ A tabela de 2026-08-19 estava desmentida por **2,4×**, e não era o perfil

O doc do `TOLERANCE_RATIO` dizia *"64 arestas → 24,1 ms"* e justificava o `1e-3` com *"o baseline do
módulo custa 25 ms, então ~64 arestas é o orçamento"*. Medida hoje, **a mesma extrusão com 56 arestas
custa 53,3 ms**.

⇒ ⚠️ **o traçado ficou ~2,4× mais caro desde a W3 e ninguém o reconferiu.** Medi as duas primitivas
lado a lado exatamente para separar *"o torno é mais caro"* de *"o traçado engordou"* — e é a
segunda. O suspeito nomeado é o **anti-serrilhado adaptativo**, que re-amostra cada pixel de borda
**quatro** vezes e entrou depois daquela medição. ⏸️ **Fica como achado por explicar**, não como
consequência desta wave.

⭐ E o **preço interativo** que aquele orçamento protegia deixou de ser este número: desde a **W24** a
resolução do preview sai do relógio (grosso a mexer, nítido ao assentar) e desde a **W32** o traçado
cede à mão. A tabela mede o traçado **assente**, que se paga uma vez. *Quem move o número que tornava
algo inalcançável tem de reconferir a nota* — e foram a W24 e a W32 que o moveram.

### §55.4 — ⛔ O gate que estava lá defendia o número velho

`the_automatic_tolerance_follows_the_size_of_the_drawing` exigia que o círculo caísse num **orçamento
de arestas** (`24..=80`) — que é a grandeza do **custo**, e não diz nada sobre o que se vê. *Um gate
assim defende o número antigo contra a medição que o desmente*, e foi ele que reprovou primeiro
quando o número mudou.

⭐ O gate novo mede o **ângulo entre facetas** (`≤ 2,5°`, entre o `2,14°` que shipa e o `3°` das
fotos) **e** o outro lado (`≤ 400` arestas — a ~1 ms cada, meio segundo de traçado assente não é
«melhor», é uma espera que ninguém pediu). A metade da invariância de escala fica onde estava.

### §55.5 — ⚠️ A sonda ficou, e é por isso que a tabela não volta a envelhecer

`field3d_profile::tests::the_table_that_chose_the_tolerance`, `#[ignore]` (mede relógio; ~8 s), com o
comando no doc-comment. *Uma tabela num doc sem a sonda ao lado envelhece em silêncio* — foi
exactamente o que aconteceu com a de 19/08.

### §55.6 — Provas de mutação

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | a tolerância volta ao número da régua errada | `the_automatic_tolerance_keeps_the_normal_smooth` |
| 2 | a tolerância foge para o fino (meio segundo por traçado) | *(o mesmo)* |
| 3 | a tolerância deixa de ser fração | `the_automatic_tolerance_follows_the_size_of_the_drawing` |

⚠️ **A 3 mentiu à primeira** (`correu 1 teste: False`): a mutação deixava dois `let` por usar e a
crate não compilava. Terceira vez hoje que o controle positivo apanha uma mutação inexpressiva.

### §55.7 — ⏸️ O que fica aberto

- ⏸️ **Não há knob de resolução**, e o pedido do Enio era literalmente esse. O impedimento é
  concreto: o nó guarda o perfil **cozido**, e mudar a tolerância exige **re-cozer a partir do
  desenho** — que é o mesmo ⏸️ de *"o nó não se religa ao contorno"* (W53). Com o default certo o
  defeito relatado desaparece; o knob pede aquela ligação.
- ⏸️ **O traçado 2,4× mais caro** desde a W3 (§55.3) — achado por explicar.
- ⏸️ Uma normal **suavizada** entre facetas curaria o sintoma com 56 arestas em vez de 168. ⛔ Seria
  sombrear uma superfície diferente da que se marcha — uma segunda verdade sobre a forma —, e por
  isso não foi feito. Fica nomeado.

## §56 — W55: o contorno continua a ser a FONTE — e o knob que faltava era a mesma ausência (23/08)

### §56.1 — Dois itens abertos, uma causa

O §55.7 fechou com dois ⏸️ escritos em linhas diferentes:

| o que o §5 dizia | o que o artista via |
|---|---|
| *"não há knob de resolução"* (o pedido literal do Enio) | o painel não tem o número |
| *"o nó não se religa ao contorno"* (W53) | editar a curva não muda a peça |

⭐ **São o mesmo item.** O `+ Extrude` cozia o contorno **uma vez** e o `Primitive::Extrude { profile }`
era tudo o que sobrava — a peça deixava de conhecer o desenho no instante em que nascia. O knob não
era uma linha de painel a faltar: era **inexprimível**, porque afinar a conversão exige ter a fonte
para reconverter. Construir um sem o outro seria meia feature nos dois sentidos.

### §56.2 — O vínculo é um componente, e o que ele NÃO guarda

[`FieldProfileSource { path: u64, level: u32 }`](../../crates/ph2d-field-ecs/src/lib.rs) —
**opcional**, registado, salvo e desfeito como os irmãos. Três decisões:

- ⚠️ **`u64` e não `VecPathId`.** A `ph2d-field-ecs` é a ponte ECS do modelador e **não** conhece o
  documento vetorial — a mesma lei pela qual o `ph2d_field::Profile` **copia** a `FillRule` em vez de
  a importar. Quem traduz é o shell, que é quem tem as duas cenas.
- ⚠️ **O NÍVEL, e não a tolerância.** Guardar o número cozido prenderia uma peça salva hoje ao joelho
  de hoje: reabri-la depois de a lei se mover daria uma finura que já ninguém escolheria. *Guarda-se
  a intenção, deriva-se o número.*
- ⚠️ **Componente próprio e não um campo no `FieldNode`** — as duas razões do `FieldMods`: quase
  nenhum nó tem um, e o blob é postcard **posicional**, então apendar quebraria todo projeto gravado.

⚠️ **Ele segue a FORMA, nunca a POSE.** O desenho tem pose própria no canvas 2D; a peça tem a dela no
espaço 3D, posta pelo artista com o gizmo. Arrastar a curva **não** teleporta a peça, e o tamanho de
convivência (o enquadramento da W53) continua a ser dele. *Uma pose, um dono.*

### §56.3 — Sem cache — e a medição é o que o dispensa

A [`field3d_profile_live::reconcile`](../../shells/desktop/src/field3d_profile_live.rs) **recoze e
compara**, todo quadro. A alternativa óbvia — guardar um resumo e só reconverter quando ele mudar —
precisa de um sítio, e os dois possíveis são maus: num **componente** é estado derivado a viajar no
save e a envenenar o undo (o `canonicalize` ordena por bytes ⇒ todo quadro vira um passo); numa
**tabela lateral** é indexado por bits de entidade, que morrem em cada desfazer.

⭐ **E medido, o cache não compra nada** (sonda `the_table_that_chose_the_resolution_ceiling`):

| nível | recozer | comparar |
|---:|---:|---:|
| 1 (168 arestas) | **7,4 µs** | 0,18 µs |
| 8 (472) | 7,7 µs | 0,23 µs |
| 32 (940) | 13,2 µs | 0,43 µs |

Contra um quadro de 16,7 ms, o pior caso é **0,08 %**. ⚠️ E as mesmas colunas lidas a `load ≈ 22` deram
**6–13 µs / 0,18–0,43 µs** — praticamente idênticas, porque isto é trabalho de microssegundos que cabe
inteiro numa fatia de escalonamento. *Quando a conclusão é «é desprezável», até uma medição pessimista
chega.*

### §56.4 — O nível: a lei, o piso e o teto

`tolerance_ratio_for(level) = TOLERANCE_RATIO / level` — e numa curva suave a contagem de arestas
anda com `tol^-1/2`, então o preço de um nível cresce com a **raiz** dele:

| nível | tolerância | arestas | traçado assente | *idem*, calmo |
|---:|---:|---:|---:|---:|
| **1** (omissão) | `1e-4` | 168 | 184,1 ms | *139 ms* |
| 2 | `5e-5` | 236 | 241,4 ms | *183 ms* |
| 4 | `2,5e-5` | 332 | 336,0 ms | *254 ms* |
| 8 | `1,25e-5` | 472 | 450,3 ms | *341 ms* |
| **16** (teto) | `6,3e-6` | 664 | 648,7 ms | *491 ms* |
| 32 | `3,1e-6` | 940 | 900,5 ms | *682 ms* |

⭐⭐ **E a tabela trouxe uma medição de graça sobre o próprio instrumento.** A corrida saiu com
`load ≈ 4,7`, e a linha do nível 1 é a **mesma configuração** que a W54 mediu a `load < 3` em
**139,3 ms**. Duas leituras do mesmo trabalho: **184,1** e **139,3** — ⭐ *32 % de diferença só de
carga, sem uma linha de código mudar*. É a lei do `CLAUDE.md` §5 com o número dela ao lado, e torna o
teto escolhido sobre a coluna **medida** conservador de propósito. A coluna «calmo» é a medida
escalada por 0,757, e está marcada como derivada.

⚠️ **O custo é linear nas arestas** (0,95 a 1,10 ms/aresta ao longo da tabela inteira) ⇒ **não há
joelho onde se esconder**: ao contrário da W54, o teto aqui é uma escolha de produto sobre uma reta.
O que a torna legítima é dizer **de que recurso** é — o tempo de assentar — e trazer a medição.

- ⭐ **O piso é 1, e não é conforto:** abaixo do joelho estão exactamente os degraus de luz que a W54
  matou, e oferecê-los seria devolver o defeito com um rótulo por cima. Quem quer mais barato já tem
  o preview grosso, que sai do **relógio** e não da autoria (W24/W32).
- ⭐ **O teto é 16 porque é onde o assentar deixa de parecer instantâneo** — meio segundo, e este knob
  **arrasta-se**, então cada passo do arrasto o paga. O 32 não compra nada que se veja (o salto de
  normal já está em 0,54° no 16) e paga **39 %** a mais.
- ⚠️ **É limite de RECURSO e não de validade:** um perfil de 940 arestas é correcto, e o documento
  aceita-o pela porta de baixo (`Profile::new` com a tolerância à mão). O que o número fecha é a
  **faixa do controle**, que é onde um teto pertence.

O `Span::Count { max }` era o molde exacto e já existia (a matriz usa-o): passo de arrasto **1**, sem
casas decimais, piso 1. ⇒ **o painel não mudou uma linha** — a fileira nasce de `params_of`, e o
`SetParam` do intent já carrega o `Param` inteiro.

### §56.5 — A costura: onde a reconciliação corre, e porquê ali

A `VecScene` mora na fase do shell que serve os pedidos de perfil; o mundo é escrito na ponte
(`ecs_bridge`, *"o mundo é a verdade e este é o único sítio que a escreve"*). Havia duas saídas:

1. reconciliar **do lado da cena** e publicar o resultado por caixa de correio ⇒ a peça fica **um
   quadro atrás** da mão que edita a curva, e nasce uma segunda escrita do mundo;
2. **passar a cena à ponte** ⇒ mesma fase, mesmo quadro, uma escrita só.

Escolhida a 2: `ecs_bridge`/`sync_scene_and_birth` recebem `&VecScene`, e os 32 sítios de gate que já
existiam passam `&crate::field3d_scene::no_drawing()` — uma função **nomeada** e não um
`&VecScene::default()` repetido, porque o que ela diz é *esta gate não tem desenho, e a reconciliação
não tem o que fazer aqui*.

⚠️ **A voz sai depois do cozimento, e não onde é produzida:** o `cook` chama `notice::clear()` quando
a peça está bem (para que um problema corrigido e recriado volte a ser dito), e a peça **cozinha bem**
sem o desenho — ela guarda a última forma. Uma frase dita antes seria apagada e re-dita para sempre.
A memória do `MISSING` é a outra metade da mesma lei, e ela **esquece quando o desenho volta**, senão
um desenho apagado → desfeito → apagado ficaria mudo na segunda vez (o modo de falha exacto que o
`forget_tried` da W23 já pagou).

### §56.6 — ⛔ O defeito PRÉ-EXISTENTE que a leitura encontrou

`copy_subtree` (a duplicação da Hierarquia) nascia na W26 com uma lista **escrita à mão** — `Name`,
`FieldNode`, `FieldPose` — e o **`FieldMods` nunca lá esteve**. Duplicar um cilindro **oco** devolvia
um maciço: sem erro, sem aviso, e com a linha da espessura simplesmente ausente do painel da cópia.
*Uma lista escrita à mão ao lado do registo de componentes são duas respostas à pergunta «o que é um
nó».*

⚠️ O `bevy_ecs` não copia componentes sem reflexão, então a enumeração é inevitável — o que se pode
fazer é **prendê-la ao registo**. O gate `a_duplicate_carries_every_optional_component_of_a_node`
começa por uma **censura** (`ComponentRegistry::len() == 5`) com a instrução na mensagem: quem
acrescentar o sexto vê o vermelho antes de o artista ver a cópia mutilada.

⚠️ E o `len()` que ela usa **já existia** — foi acrescentado por engano e removido: o meu `grep` da
API parou 25 linhas antes dele. *A peça que falta pode já estar construída.*

### §56.7 — Provas de mutação

Dez, todas RED, todas com os três controles (verde-antes · `Compiling` · `running N tests`):

| # | o que se partiu | gate que ficou RED |
|---|---|---|
| 1 | a cópia larga o modificador (**o defeito de W26**) | `a_duplicate_carries_every_optional_component_of_a_node` |
| 2 | a cópia larga o vínculo ao desenho | *(o mesmo)* |
| 3 | o componente novo não é registado | *(o mesmo — o braço da censura)* |
| 4 | a reconciliação reescreve o nó sem mudança | `an_unchanged_outline_never_writes_the_node` |
| 5 | a reconciliação nunca escreve | `editing_the_outline_reshapes_the_piece` |
| 6 | o nível é ignorado pela lei da tolerância | `the_resolution_level_buys_edges_through_the_panel_door` |
| 7 | o desenho ausente fala em todo quadro | `a_deleted_outline_speaks_once_and_the_shape_keeps_its_form` |
| 8 | a linha «Resolution» é oferecida a todo nó | `the_resolution_row_is_offered_exactly_where_the_link_is` |
| 9 | a forma nasce sem vínculo (**o estado da W53**) | `a_drawn_shape_is_born_linked_to_the_outline_it_came_from` |
| 10 | o teto do nível não se impõe na escrita | `the_resolution_is_bounded_where_it_is_written_not_only_where_it_is_dragged` |

⚠️ **A 4 mentiu no lote** (`correu 0 testes`): o `rustc` falhou a **ligar** sob a carga do fan-out, e
sem o controle positivo aquele vermelho teria contado como prova. Corrida sozinha: `running 1 test`,
`FAILED`. *Segunda vez que o controle apanha um vermelho que não era do gate* — a primeira foi a W50.

⚠️ **A régua da 4 é a marca de escrita do ECS (`Changed<FieldNode>`), não o conteúdo.** Um perfil
reescrito com o mesmo valor tem os mesmos bytes ⇒ uma comparação de conteúdo passaria com a guarda
arrancada, que é exactamente a guarda que o gate defende.

### §56.8 — ⛔ O gate de LOC estava VERMELHO há waves, e ninguém o corria

O fecho desta wave correu a suíte **inteira** do shell pela primeira vez em várias waves, e o
`shell_files_respect_hr18_loc_cap` estava vermelho com **quatro** arquivos — três deles **antes** de
esta wave tocar em nada:

| arquivo | em `main` | ao abrir a W55 | causa |
|---|---:|---:|---|
| `field3d_smoke.rs` | 506 | **790** | W38, W44, W45, W46, W51, W52 |
| `field3d_scene.rs` | 555 | **659** | idem |
| `input_dispatch/keyboard.rs` | 585 | **606** | as seis teclas do módulo |
| `field3d_isolate_tests.rs` | — | 582 → **618** (W55) | o `fmt` re-expandiu o argumento novo |

⚠️ **O mecanismo do último é o que a memória já registava:** um argumento acrescentado a 32 chamadas
não muda o número de linhas — o `cargo fmt` é que parte as chamadas longas e **cria** linhas. *Medir
LOC antes do `fmt` mede outra coisa.*

⛔ **E o mecanismo dos três primeiros é o fecho estreito:** cada wave correu os testes que tocou, e
este gate não está em nenhum deles — ele varre `shells/desktop/src/` inteiro. É a irmã exacta da
lição do clippy da W44 (*o alvo do fecho deriva do DIFF, nunca da minha memória*), aplicada ao
**teste**: um gate de árvore não é alcançado por um filtro de nome.

A cura foi **cortar para o irmão**, nunca uma allowlist, e as quatro fronteiras já existiam por
dentro:

| novo arquivo | o que levou | a fronteira |
|---|---|---|
| `field3d_scene_gizmo.rs` | arrasto, pick, âncora, duplicar | *o que o gesto AGARRA* ≠ *o que a peça É* |
| `field3d_smoke_state.rs` | `Smoke`, `Grip`, `Drag`, `Ready`, `InFlight`, a célula | *o que existe* ≠ *o que se faz* |
| `input_dispatch/keyboard_field3d.rs` | as seis teclas do módulo, numa porta | o irmão de escultura já tinha a dele |
| `field3d_profile_reach_tests.rs` | os gates de alcance do perfil (W53) | *o painel oferece?* ≠ *o isolamento diz-se?* |

⚠️ **A ordem das seis teclas viajou inteira e está dita no doc do módulo novo**: a entrada numérica
vem antes da tecla de verbo, senão um `5` digitado no meio de um gesto vira um pedido de lente.
*Reordenar ali é mudar comportamento, não estilo.*

### §56.9 — ⏸️ O que fica aberto

- ⏸️ **A tabela do teto foi medida a `load ≈ 4,7`**, não abaixo de 3 (§56.4). A coluna calma é
  escalada pelo fator que a própria tabela mede, e o teto sai conservador — mas uma corrida com a
  máquina parada é trabalho por fazer. ⚠️ O que ela pode mover é o **teto**, não a lei.
- ⏸️ **O traçado 2,4× mais caro** desde a W3 (§55.3) continua por explicar — e agora tem mais um
  consumidor.
- ⏸️ **O contorno é UM.** Um `+ Extrude` com várias formas escolhidas usa a primeira (herdado da W53).
- ⏸️ **Nada na Hierarquia mostra que uma forma está ligada a um desenho** — o painel diz (a linha
  «Resolution» só aparece com vínculo), mas quem olha a árvore não vê a diferença entre uma extrusão
  viva e uma solta.
- ⏸️ **Não há gesto para LARGAR o vínculo** nem para o **religar** a outro contorno. Largar é a
  metade fácil (tirar o componente); religar é a mesma pergunta de UI que a escultura que mudou de
  sítio já tinha — *o app ainda não pergunta «qual asset?» por asset nenhum*.

---

## §57 — W56: o perfil deixa de ser uma FITA e passa a ser uma CONSULTA — o alicerce, e a receita que foi refutada (24/08)

### §57.1 — O gatilho disparou, e quem o disparou fui eu

O [`04_resultados_perfis.md`](../3DModeling/04_resultados_perfis.md) §7 escreveu em **2026-08-19** o
gatilho e as duas direções, e fechou com: *"⛔ **Nenhuma das duas foi feita**, e é de propósito: o
número que as pediria — um perfil real acima de 128 arestas, num fluxo real — ainda não existe."*

⭐ **A W55 criou esse número**: o default shipa **168** arestas e o knob de `Resolution` vai a **664**.
*Quem move o número que tornava algo inalcançável tem de reconferir a nota* (`CLAUDE.md` §0.0) — e
esta wave é a reconferência.

### §57.2 — ⛔ E a cura PRESCRITA não serviria

A nota prescrevia: *"aceleração espacial **dentro da árvore** — partir o perfil numa hierarquia de
`min`/`max` por caixa, para que **a poda por intervalo** volte a morder."*

⚠️ **Ninguém avalia intervalos neste caminho.** O `Hybrid` monta `float_slice_tape` (ponto a ponto) e
`grad_slice_tape`; não há passe por ladrilho, não há `simplify`, e a extração varre uma grade
uniforme — também ponto a ponto. Uma hierarquia de `min`/`max` numa fita ponto-a-ponto é percorrida
**inteira**: ela não moveria o traçado um milissegundo.

⭐ *Meça o mecanismo antes de construir o que a nota prescreve* — a mesma lição da W54, onde a
aritmética refutou a minha primeira hipótese antes do código. Aqui foi a **leitura do caminho de
avaliação** que refutou uma prescrição de cinco dias antes.

### §57.3 — O TETO de qualquer cura (o limite de Amdahl)

Sem este número, uma aceleração de `k×` no perfil é uma promessa sobre o quadro que ninguém mediu
(sonda `the_ceiling_of_any_profile_cure`, 640×480, mediana de 7, máquina calma):

| | traçado | fração que é o PERFIL | teto |
|---|---:|---:|---|
| um cilindro analítico | **10,7 ms** | — | é o **piso**: marcha, normais, anti-serrilhado |
| 56 arestas | 56,8 ms | 81,2 % | 5,3× |
| **168** (o default) | **133,6 ms** | **92,0 %** | **12,5×** |
| 664 (o teto do knob) | 531,9 ms | 98,0 % | **49,8×** |

### §57.4 — ⚠️ E a barra é ALTA: a fita custa 0,95 ns por ponto por aresta

| arestas | ns/ponto | × cilindro | ns/ponto/**aresta** |
|---:|---:|---:|---:|
| — | 2,0 | 1,00× | — |
| 56 | 52,5 | 26,6× | 0,937 |
| 168 | 155,8 | 79,0× | 0,927 |
| 664 | 636,2 | 322,7× | 0,958 |
| 940 | 877,7 | 445,1× | 0,934 |

⭐ **Linear perfeito.** ⚠️ E `0,95 ns` são ~20 operações por aresta em **oito faixas de SIMD com
JIT** ⇒ *por aresta, a fita é quase óptima*. O que se pode ganhar não é fazer cada aresta mais
barata — é **tocar menos arestas**.

### §57.5 — A consulta: duas estruturas, porque as duas metades têm naturezas diferentes

[`ph2d-field-eval/src/profile_index.rs`](../../crates/ph2d-field-eval/src/profile_index.rs):

| metade | estrutura | custo medido |
|---|---|---|
| **distância** | BVH sobre os segmentos, ramo-e-limite | 85 → 148 → 281 ns (56 → 168 → 664) |
| **sinal** | grelha sobre a caixa, enrolamento **pré-somado** por célula | **14,5 ns, PLANO em `n`** |

⭐⭐ **O sinal fora da caixa é ZERO e é exacto** (o enrolamento de uma curva fechada contida na caixa
é nulo fora dela) — é isso que dispensa a grelha de cobrir o espaço onde a marcha passa a maior parte
do tempo. ⭐ E dentro da caixa o enrolamento é um **invariante de caminho**: `w(p) = w(canto) +
atravessamentos`, e o caminho canto→ponto não sai da célula ⇒ só as arestas que a atravessam contam.

### §57.6 — ⚠️ A primeira nuvem NÃO continha o fenómeno

A nuvem uniforme sobre 1,8² sobre-representa o **miolo**, e é lá que a busca do segmento mais próximo
é patológica: no centro de um círculo **todas** as arestas estão à mesma distância, e nenhuma
estrutura poda o que é equidistante. Uma esfera-marcha caminha de fora para dentro e **pára na
casca** — ela quase não amostra o miolo:

| arestas | longe | **colado à casca** | no miolo |
|---:|---:|---:|---:|
| 56 | 62,4 ns | 54,9 ns | 275,5 ns |
| 168 | 109,3 ns | **80,9 ns** | 612,8 ns |
| 664 | 191,8 ns | **125,3 ns** | 1304,2 ns |

*Uma fixture que não contém o fenómeno mede outra coisa* — e aqui ela mediria a cura como 1,0× onde
ela vale 1,9×.

### §57.7 — O que fecha a distância até ao tecto: cortar por LOTE

Uma busca escalar dá **1,9×** contra uma fita de oito faixas. O que muda de ordem de grandeza é
**cortar as arestas para um lote compacto** e depois correr um laço tenso — e o corte tem de ser
**conservador**, ou a marcha atravessa a peça (deitar fora a aresta mais próxima faz a distância sair
**maior** que a verdadeira, e uma esfera-marcha que sobre-estima o passo salta a superfície). A regra
exacta usa a convexidade da distância a um segmento:

```text
dmax = min sobre as arestas de (maior distância de um CANTO da caixa àquela aresta)
fica  = toda aresta cuja MENOR distância à caixa é <= dmax
```

Com lotes de 1024 pontos (o tamanho de um ladrilho):

| arestas | pegada | arestas após o corte | ns/ponto | vs fita |
|---:|---:|---:|---:|---:|
| 168 | 1,000 | 134,5 | 442,0 | 0,3× |
| 168 | 0,250 | 24,3 | 101,3 | 1,5× |
| **168** | **0,062** | **6,5** | **40,5** | **3,8×** |
| 664 | 0,250 | 91,2 | 318,4 | 1,9× |
| **664** | **0,062** | **23,1** | **115,3** | **5,3×** |

⚠️ **O corte mede a compacidade de quem o chamou.** Uma linha inteira de ecrã tem pegada larga e
corta **nada** (0,3×, pior que a fita); um punhado de raios vizinhos corta quase tudo. ⇒ *o consumidor
tem de marchar em ladrilhos, não em linhas* — e é isso que falta construir.

⭐ No quadro, 3,8× e 5,3× sobre 92 % e 98 % dão **3,1×** e **4,9×**: 134 → 43 ms e 532 → 109 ms.

### §57.8 — ⛔ O obstáculo que uma leitura revelou, e que muda o desenho

O `Hybrid` já mistura fita com folha **amostrada** (é assim que a escultura entra). ⚠️ Mas o
gradiente exacto só existe quando `sampled.is_empty() && trees.len() == 1`: **qualquer** folha
amostrada derruba a normal para diferença central — e é a diferença central que **apaga a quina
viva**, que é a razão deste módulo existir. ⇒ A consulta **não pode** pegar boleia daquele caminho:
ela tem de trazer o próprio gradiente analítico, e o `Hybrid` tem de aprender a reduzi-lo.

### §57.9 — Provas de mutação

Sete, todas RED, todas com os três controles (verde-antes · `Compiling` · `running N tests`):

| # | o que se partiu | gate |
|---|---|---|
| 1 | o sinal do atravessamento invertido | `the_query_is_the_same_law_as_the_tape` |
| 2 | o enrolamento pré-somado da célula é ignorado | *(o mesmo)* |
| 3 | as arestas que atravessam a célula são ignoradas | *(o mesmo)* |
| 4 | o irmão direito do BVH volta a ser `left + 1` | *(o mesmo)* |
| 5 | o corte deita fora a aresta mais próxima | `the_cull_never_drops_the_nearest_edge` |
| 6 | o sinal do lote cortado é sempre positivo | *(o mesmo)* |
| 7 | fora da caixa deixa de ser fora | `the_query_knows_inside_from_outside` |

⚠️ **A 4 é um defeito REAL da primeira versão**, apanhado por leitura: a construção é pós-ordem, então
o irmão direito **não** fica em `left + 1` — a versão que o supôs lia um nó de outra sub-árvore.

⚠️ **E o arnês mentiu duas vezes antes de dizer a verdade.** A linha-base dava `running 0 tests` com
`rc == 0`: o binário de teste estava **obsoleto** (o `cargo` não reconstruía ao apender no arquivo de
gates montado por `#[path]`). Sem o controle de *«correu N testes»*, aquilo teria contado como sete
mutações apanhadas — e nenhuma teria sido.

### §57.10 — ⛔⛔ O incidente: a cwd escorregou para o primário

⚠️ **Metade desta wave foi escrita na árvore ERRADA.** Os `python3 - <<'PY'` com caminho **relativo**
foram parar ao checkout primário (`/…/PH2D/`) em vez da worktree, e como o mesmo caminho relativo
existe nas duas árvores, **tudo compilou e todos os gates passaram lá** — o sintoma só apareceu quando
o arnês de mutação, que usa caminho **absoluto**, mediu um binário sem os testes novos.

É a armadilha que a memória [[feedback_bash_cwd_resets_and_slips_to_the_primary]] nomeia, e o
`CLAUDE.md` §1 avisa em maiúsculas: *"editar a errada compila e commita **sem erro**"*. Recuperado por
cópia + `git apply` do diff, com o primário reposto ao que era. ⇒ **Todo comando de shell desta linha
leva o `cd` da worktree à frente**, e um caminho de edição é **absoluto**.

### §57.12 — ⭐⭐⭐ E a saída é ESPECIALIZAR A ÁRVORE, não sair dela

Uma leitura do `Builder` do [`hybrid`](../../crates/ph2d-field-eval/src/hybrid.rs) fechou a rota da
folha nativa, e não pela velocidade — pelo **produto**:

- ⛔ uma folha amostrada **não passa pela pilha de modificadores** (`FieldError::ModsOnSampled`) ⇒
  uma peça desenhada perderia *Hollow*, *Offset*, *Array*, *Taper*;
- ⛔ o `Combine` **misto** não aplica pose nem pilha (item aberto nomeado no §22) ⇒ um grupo com uma
  peça desenhada dentro perderia a própria pose;
- ⛔ e o gradiente exacto morre (§57.8).

⭐ **A saída não era sair da árvore: era especializá-la.** A mesma lei, com uma fração das arestas —
e as duas metades precisam de **conjuntos diferentes**:

| metade | de que arestas precisa | porquê |
|---|---|---|
| **distância** | as que podem ser a mais próxima de **algum** ponto da região | `min`: uma aresta longe pode ganhar |
| **sinal** | só as que **atravessam** a região | o enrolamento é invariante de caminho |

⭐⭐ **O enrolamento vira uma CONSTANTE mais um punhado de termos:** `w(p) = w(c) + atravessamentos
do caminho c→p`, com `c` o canto da região. `w(c)` calcula-se na construção e entra como número; o
caminho não sai da região ⇒ só uma aresta que a atravessa o pode cruzar — tipicamente **três**.

| arestas | pegada | dist+cruz | montar | ns/ponto | vs fita completa |
|---:|---:|---:|---:|---:|---:|
| 168 | 0,250 | 20+10 | 0,23 ms | 15,1 | **10,3×** |
| 168 | 0,125 | 9+4 | 0,11 ms | 7,0 | **22,2×** |
| **168** | **0,062** | **5+3** | **0,07 ms** | **4,8** | **32,5×** |
| 664 | 0,125 | 37+17 | 0,40 ms | 25,9 | 24,6× |
| **664** | **0,062** | **20+10** | **0,23 ms** | **14,9** | **42,7×** |

⭐⭐⭐ **Contra o tecto de Amdahl de 12,5× e 49,8×, isto SATURA-O** — e mantém tudo: fusão numa fita
só, JIT, gradiente exacto, modificadores, poses e booleanas. A consulta nativa (§57.5–57.7) fica
sendo o que ela é de facto: o **índice** que escolhe as arestas, e o **juiz** que prova o corte.

⚠️ **A árvore especializada só vale DENTRO da região dela** — fora, a distância pode sair maior (o
modo de falha que atravessa a peça) e o sinal pode ter a base errada. O gate
`the_specialised_tree_agrees_inside_its_region` mede as duas pontas: concorda a `1e-5` dentro, **e**
guarda menos de metade das arestas (senão a cura degenerada — especializar guardando tudo — passaria).

### §57.13 — A ponte: o DOCUMENTO especializado para uma região do mundo

A `sd_profile_in_region` especializa **um** perfil no plano dele. Um documento tem poses, pilhas e
booleanas por cima — e é a [`compile_in_region`](../../crates/ph2d-field-eval/src/lib.rs) que leva
uma caixa do **mundo** até ao plano de cada perfil:

1. **de cima para baixo**, o mapa afim mundo→local de cada nó (a arena tem os filhos antes dos pais,
   então descer por índices decrescentes visita todo pai antes dos filhos dele);
2. a caixa do mundo transformada pelos **oito cantos** (exacto para um mapa afim);
3. o perfil baixado para essa caixa — `(x, y)` numa extrusão, e um **anel em `u = √(x²+z²)`** num
   torno, que ainda tira a costura do eixo da distância.

⛔ **E ela DESISTE debaixo de quatro modificadores**, com a razão escrita: `Mirror`, `Array`,
`Radial` e `Taper` **remapeiam coordenadas**, e aí a caixa do mundo não mapeia para uma caixa no
plano do perfil — uma matriz dobra meio espaço numa célula. Ali o perfil é baixado **inteiro**:
correcto, só não mais rápido. *Uma especialização que erra a pré-imagem não fica lenta: fura a peça.*
⚠️ `Shell` e `Offset` agem no **valor** e não desistem.

⚠️ **Dois furos de gate apanhados por mutação, e os dois eram FIXTURE:**

- a costura do eixo não estava coberta — nenhum torno da fixture tinha aresta em `x = 0`;
- e a desistência sob `Array` **sobreviveu duas vezes**: primeiro porque o contorno era um
  **círculo** (equidistante de todos os lados ⇒ guardar as arestas erradas devolve o mesmo número), e
  depois porque numa região **longe** o corte guarda quase tudo e a discrepância some no ruído.
  ⇒ a lei passou a ter um gate **directo** (uma censura com `match` exaustivo: um `Unary` novo é erro
  de compilação ali). *Quando a versão comportamental não morde de forma fiável, a lei tem de ser
  afirmada onde ela é decidida.*

### §57.14 — O CONSUMIDOR: a marcha por ladrilho, e o que ela de facto comprou

O traçado passa a marchar em **ladrilhos** e a pedir uma árvore por região. Duas metades, e as duas
são de correcção:

1. **o raio prende-se à caixa da peça** (`Scene::clip`): ele começa na entrada e pára na saída, então
   **nenhuma amostra cai fora** da região para que a árvore foi construída;
2. **a região é o tubo do ladrilho ∩ a caixa da peça**, com o `t` tirado do **recorte pela caixa** e
   inflada pela sonda da normal.

| arestas | por linha | por ladrilho | ganho |
|---:|---:|---:|---:|
| 56 | 65,6 ms | **36,8 ms** | 1,8× |
| 168 | 166,6 ms | **91,8 ms** | 1,8× |
| 664 | 545,7 ms | **337,8 ms** | 1,6× |

⚠️ **1,8×, e não os 5–6× que o §57.12 projectava.** O mecanismo está medido: a projecção supunha uma
pegada de `0,125` da peça, e um ladrilho de 64 px não a alcança — **um raio de viés varre em `(u, v)`
muito mais do que a largura do ladrilho**, porque ele atravessa a espessura da peça. A pegada efectiva
fica em ~`0,4`, que a tabela do §57.12 já dizia valer ~3×, e o resto vai na montagem das fitas
(≈ 0,29 ms por ladrilho, medido).

⭐ ⇒ **O próximo degrau é ladrilhar também em PROFUNDIDADE** — uma árvore por fatia de `z` além de por
ladrilho de ecrã. É o mesmo mecanismo, um eixo a mais, e é o que fecha a distância até ao tecto.

### §57.15 — ⛔⛔ Três defeitos que só o gate de IMAGEM apanhou

O caminho novo passou em todos os gates de avaliação e **desenhou errado**. Os três, por ordem de
descoberta:

1. **A região era a peça inteira.** O tubo do ladrilho ia até `T_MAX`, e a caixa dele engolia tudo ⇒
   toda região guardava todas as arestas. ⚠️ **Nada ficou errado na imagem** — foi só lento (`1,3×` em
   vez de `5×`), que é a forma de defeito que um gate de paridade **não vê**. Gate novo:
   `a_tile_region_is_much_smaller_than_the_piece`.
2. **As sondas da NORMAL saíam da região.** Ela é uma diferença central em `ponto ± ε`, e um `ε` fora
   da região faz a árvore especializada responder onde ela não vale: **90 pixels apagaram-se** (o
   gradiente saía nulo e a marcha desistia do acerto). ⇒ a região é inflada pela sonda. *Uma região
   tem de conter tudo o que é avaliado — inclusive o que é avaliado DEPOIS de o raio parar.*
3. ⭐⭐⭐ **A regra do atravessamento tinha de ser SEMIABERTA.** Um caminho que passa **por um
   vértice** do contorno era contado **zero** vezes — as duas arestas que o partilham veem produto
   nulo e ambas desistem. O enrolamento sai errado por um numa cunha fina à volta daquele vértice, o
   sinal inverte-se lá dentro, e a esfera-marcha **inventa uma superfície**. Um pixel, de ~800 mil
   amostras. É a mesma disciplina que o raio `+x` já seguia (Dan Sunday): *uma regra de fronteira
   escrita duas vezes tem de ser a mesma nas duas.*

⚠️ **E os três foram encontrados por uma SONDA DE MECANISMO, não por leitura:** marchar o raio do
pixel divergente imprimindo `spec` e `full` lado a lado deu, no passo 13, *a mesma magnitude com o
sinal trocado* — que aponta para o enrolamento e para mais nada. *Um pixel errado tem um raio, e um
raio tem passos.*

### §57.16 — ⚠️ E três gates que passavam sem medir nada

Cada um dos defeitos acima sobreviveu a gates verdes, e a razão foi sempre a **fixture**:

| o que não estava coberto | porquê passava | a cura |
|---|---|---|
| caminho por um **vértice** | 4 000 caminhos aleatórios nunca passam por um | caminhos **construídos** por cada vértice |
| âncora **em cima** de uma aresta | as regiões aleatórias nunca caem lá | um quadrado, com a região a começar na aresta |
| região do tamanho da **peça** | as regiões do gate eram pequenas | as duas primeiras passaram a ser a peça inteira |

⭐ E o custo de não as ter: **21 das 22 mutações ficaram vermelhas à primeira; as três que
sobreviveram são exactamente estas.** *A prova de mutação não mede o código — mede a fixture.*

## §58 — W56e: a PROFUNDIDADE, e o que ela obrigou a admitir (24/08)

> A W56d fechou com uma nota: *"o degrau seguinte é ladrilhar também em **PROFUNDIDADE** — é o mesmo
> mecanismo, um eixo a mais, e é o que fecha a distância até ao tecto."* Esta wave foi construí-lo.
> ⚠️ **A nota lia o mecanismo certo e errava o preço por quatro vezes**, e o caminho até o descobrir
> devolveu um **defeito de correcção** que estava latente desde a W56c.

### §58.0 — ⛔ O portão de LOC estava VERMELHO desde a W56d

`architecture_workspace_file_loc_cap`: `ph2d-field-render/src/lib.rs` a **889/700** e
`ph2d-field-eval/src/profile_index_tests.rs` a **795/700** — os dois escritos pela W56.

⚠️ **É a família da memória [`feedback_a_tree_scanning_gate_is_never_reached_by_a_name_filter`], uma
wave depois de ela ser escrita.** O gate VARRE `crates/*/src/` e mora em `ph2d-editor-core`; o fecho
da W56d correu `-p ph2d-field-eval` e `-p ph2d-field-render`, e **filtro de nome nenhum o alcança**.

Cura: quatro cortes por responsabilidade (nunca allowlist) — `tiles.rs` (o repartir), `edges.rs` (a
segunda passagem), `march.rs` (o laço), `profile_index_tables.rs` (as varreduras, separadas dos
gates). `lib.rs` **889 → 482**.

### §58.1 — A régua primeiro: o que uma fatia compra, ANTES de a construir

Arestas guardadas por região, no quadro real (640×480, 70 ladrilhos sobre a peça):

| N | Σ das fatias (montar) | média (avaliar) | máx |
|---:|---:|---:|---:|
| 1 | 128,6 | 128,6 | 168 |
| 2 | 185,0 | 92,5 | 168 |
| 4 | 268,9 | 67,2 | 168 |
| 8 | 427,3 | 53,4 | 168 |

⇒ repartir **divide** o custo de avaliar e **multiplica** o de montar. Qual manda decide a wave, e
isso mediu-se: **montar é 18% do quadro, marchar 82%** (`the_table_of_which_half_the_tiled_frame_pays`).

⚠️ **A 1.ª versão dessa sonda imprimiu `197%`** — um número impossível, e ele só dizia que o
denominador corria em 32 núcleos e o numerador em série. *Uma régua tem de correr no mesmo regime do
que ela mede.*

⭐ **E a montagem é 96% JIT**: por ladrilho, a 168 arestas, `96 µs` para construir a árvore e
**`2 334 µs`** para a `fidget` a compilar em máquina (`the_table_of_where_the_tile_assembly_goes`).
É esse número que põe o tecto em quantas fatias cabem.

### §58.2 — ⛔ A hipótese do círculo, medida e REFUTADA

O corte guarda toda aresta a menos de `dmax = min_e (máx distância de um CANTO da região a e)`. Numa
peça **redonda** todas as arestas são equidistantes do centro, então uma região interior guarda as
168 — e a coluna `máx` da tabela acima dizia-o de frente em todo `N`. A fixtura de **toda a W56** é
um `n`-gono regular. ⇒ hipótese: o `1,8×` de manchete é um **piso**, e um contorno real cortaria melhor.

**Medido em três contornos** (`the_table_of_what_the_shape_of_the_outline_does`):

| contorno | arestas | guardadas | % |
|---|---:|---:|---:|
| círculo | 168 | 128,6 | 77% |
| **estrela** | 168 | **144,0** | **86%** |
| pente | 172 | 138,1 | 80% |

⛔ **Refutada — a estrela corta PIOR que o círculo.** ⭐ E a refutação nomeia a causa verdadeira:
para uma região de diâmetro `D` com a aresta mais próxima a `a`, `dmax ≈ a + D`. *O corte não é
fraco porque a peça é redonda: ele é fraco porque a REGIÃO é grande.*

⭐⭐ E daí sai a lei que fecha a varredura do `TILE` da W56d: a região mede
`lado + profundidade · |direcção|`, e o segundo termo **não sabe o lado** — foi por isso que
encolher o ladrilho deu um **vale** e não uma descida. **Só a profundidade sobra.**

### §58.3 — ⛔⛔ O defeito que a fatia acordou: os quatro cantos NÃO bastam

O doc do `tile_region` dizia, desde a W56c, que os quatro raios de canto bastam *"e não é
aproximação"*. **É**, e só na **paralela** é exacta.

- Na ortográfica a direcção é constante ⇒ o ponto é bilinear na posição de ecrã ⇒ um raio interior é
  **combinação convexa** dos cantos. ✔
- Na **convergente** — que é o default de `Orbit::from_yaw_pitch` — a direcção é **normalizada**:
  `d̂(s)` percorre um quadrilátero **esférico**, que abaúla para fora da corda dos quatro cantos.

⛔ **Medido**, câmera de frente: a fuga vai de `2,80e-4` com uma fatia a **`4,03e-4` com oito**, e
**passa a folga** (`4e-4`) exactamente quando a fatia aperta. *A premissa não mordia porque o tubo
era grande; fatiar é o que a acorda — e ela é de uma wave que já shipou.*

⭐ **A cura tem prova e custa dois produtos internos.** Todo ponto a parâmetro `t` está na esfera de
raio `t` em torno do olho, dentro do cone do ladrilho; a distância dele à corda dos quatro cantos no
mesmo `t` é no máximo a **flecha** `t·(1 − cos α)`, com `α` o ângulo do canto mais afastado ao raio
central. E `hull{p_j(t)} ⊆ hull{p_j(t₀), p_j(t₁)}` porque `p_j(t)` é linear em `t` ⇒ a caixa dos oito
pontos contém a corda em toda a faixa. ⚠️ Na paralela `α = 0` e a inflação é **exactamente zero** —
a lente sem o defeito não paga por ele.

⭐⭐ **E há um segundo, do mesmo tipo:** o `t` de entrada na caixa é `max` de funções afins da
posição de ecrã ⇒ **convexo** ⇒ o mínimo dele pode ser **interior** ao ladrilho. Medido: um raio
interior entra até **`7,4e-2` antes** do `t_lo` que os cantos dão, e sai até `1,2e-1` depois do
`t_hi` — sobre uma peça que mede `1,0`. ⇒ a 1.ª fatia começa em **`0`** e a última acaba em
**`T_MAX`**, o que torna a cobertura **trivialmente** completa. Como a montagem é preguiçosa, elas
custam **zero** quando ninguém lá chega. *A cerca não é uma afirmação sobre onde os raios entram: é
a ausência de uma.*

### §58.4 — A marcha por fatia, e o grampo que era pior de duas maneiras

`march_slabs` (em `crates/ph2d-field-render/src/march.rs`) é agora **o** núcleo, e a marcha de sempre
é o caso `N = 1` — *uma marcha, um lugar*. Ela recebe as fronteiras e um `shape_of(k)` que só é
chamado **quando algum raio de facto chega à fatia `k`**, e a normal é avaliada na fita **da fatia em
que o raio parou**, antes de ela morrer.

⭐ **O grampo na fronteira era desnecessário — e uma mutação disse-o.** A 1.ª versão fazia
`t = lim` "para não sair da região". Uma mutação que apagou essa linha **SOBREVIVEU**: o passo é a
**distância verdadeira**, então nada existe no intervalo saltado, e o filtro da fatia seguinte manda
o raio para a fatia que de facto o contém — saltando por cima das que ele atravessou **sem as
montar**. Tirá-lo ficou mais rápido **e apagou todos os pixels divergentes** (eram 1–4 por quadro a
`N ≥ 2`; passaram a **zero**): a amostra extra na fronteira caía por vezes rasante e disparava o
teste de acerto. *Uma guarda que mutação nenhuma mata estava a comprar uma avaliação por travessia,
e um pixel.*

### §58.5 — A varredura, e o número honesto

640×480, mediana de 5, `ms / pixels ≠ da marcha de linha`:

| contorno | linha | N=1 | **N=2** | N=3 | N=4 | N=6 | N=8 |
|---|---:|---:|---:|---:|---:|---:|---:|
| círculo 56 | 53 | 35/0 | **33/0** | 38/0 | 44/0 | 54/0 | 64/0 |
| círculo 168 | 131 | 81/0 | **69/0** | 73/0 | 80/0 | 93/0 | 107/0 |
| estrela 168 | 207 | 191/0 | **168/0** | 183/0 | 181/0 | 190/0 | 206/0 |
| círculo 664 | 510 | 318/0 | **257/0** | 253/0 | 259/0 | 283/0 | 301/0 |

⇒ **`SLABS = 2`**, melhor ou empatado nas quatro. Contra a marcha de linha: **1,89×** no detalhe
padrão (era `1,8×`), **1,98×** no máximo. Contra o ladrilho sem fatias: **1,06×–1,24×**.

⚠️ **É `1,17×`, não os `5×` que a nota da W56d prometia.** A nota lia o mecanismo certo — a pegada —
e errava o preço, porque a montagem é **JIT** e cresce com a **soma** sobre as fatias enquanto a
avaliação cai com a **média**. *Uma nota que nomeia o mecanismo certo ainda pode errar a ordem de
grandeza, e o que a corrige é medir as duas contas separadas.*

### §58.6 — ⚠️ O que as mutações apanharam: uma lei escrita duas vezes

10 mutações. Na 1.ª volta, **quatro sobreviveram** — e três pela mesma causa: o gate
`every_sample_lies_inside_the_region_that_built_its_tape` **reconstruía as fronteiras dentro do
teste**. Mexer na cópia do produto não movia a do gate. ⇒ `slab_bounds` e `slab_region` passam a ser
**uma porta só**, chamada pelos dois. *Duas cópias de uma lei é uma lei que gate nenhum defende.*

⭐ E a quarta sobrevivente era o próprio gate a cortar-se: ele media as amostras **dentro das
fronteiras**, então apagar a 1.ª e a última fronteira nunca aparecia — o pedaço de raio que ficava de
fora não era medido. A metade que faltava é a **COBERTURA** (`bounds[0] ≤ entrada` e
`saída ≤ bounds[último]`). *Um invariante avaliado dentro do domínio que ele define não diz nada
sobre a fronteira dele.*

⚠️ **A décima não morre, e o motivo fica escrito:** fazer o `shape_of` devolver sempre `None`
desliga a especialização inteira e a imagem sai **idêntica** — é defeito só de relógio, a mesma
família do «a região era a peça inteira» da W56d. Quem o defende é a tabela medida, que é relógio
por natureza. *A paridade prova a IMAGEM; a tabela prova o PREÇO.*

### §58.7 — ⏸️ O que fica medido e por construir

- ⏸️ **O passo da marcha é do DOCUMENTO, não uma constante.** `SAFE_STEP = 1/√2` é o recíproco de uma
  constante medida na W0 (`‖∇f‖` chega a `√2` no arredondamento exacto). Um extrude sem `round`
  sobre uma distância de polígono exacta é uma distância **verdadeira**, e nele andar `d` inteiro é
  seguro: **medido `1,16×` no mesmo processo, com a imagem idêntica** (0 pixels de diferença a 168 e
  664 arestas; 2 a 56). ⚠️ **Não shipa por falta da AUDITORIA dos operadores** — quem infla o
  gradiente: `round`, os `Smooth*`, o `Taper`, uma escala não-uniforme no `Xform`; quem não infla:
  `Union`/`Intersect`/`Subtract`, `Shell`, `Offset`, `Mirror`. `Array`/`Radial` são o caso subtil (a
  repetição de domínio só é um limite válido se a peça couber na célula). ⛔ Errar isto não fica
  lento: **fura a peça**. O `Scene::step` já existe e já viaja — falta a função que o deriva.
- ⏸️ **Ladrilhar em `(u, v)` sem AABB.** A pegada real de um ladrilho no plano do perfil é um
  **paralelogramo**, e o corte só sabe caixas. Cortar contra o quadrilátero apertaria a região **sem
  pagar JIT nenhum** — é o único eixo que resta que não multiplica a montagem.
- ⏸️ O tecto do `MAX_PROFILE_RESOLUTION = 16` foi derivado com o custo **antigo** e pede recontagem.
- ⏸️ A marcha ficou ~2,4× mais cara que na W3 e ninguém explicou porquê (herdado da W54).

## §59 — W56f: o passo da marcha é do DOCUMENTO, não uma constante (24/08)

> A W56e deixou isto ⏸️ com o preço medido (`1,16×`) e a razão de não shipar escrita: *"falta a
> AUDITORIA dos operadores — errar isto não fica lento: **fura a peça**"*. Esta wave é a auditoria.

### §59.1 — A pergunta, e por que ela é de `CLAUDE.md` §0

A marcha de esferas anda `d · s` e é segura enquanto `s · ‖∇f‖ ≤ 1`. O traçador andava `1/√2` **em
todo documento**, e o número é o recíproco de uma constante medida na W0: `‖∇f‖ = √2` no
arredondamento **exacto**.

⚠️ Mas o `Xform::scale` deste módulo é **uniforme de propósito**, e o doc-comment dele já escrevia a
fundação: *"‖∇f‖ = 1 destrói-se com escala não-uniforme, e ela é a fundação de tudo neste módulo"*.
⇒ se quase todo construtor honra a fundação, o passo curto é **o caminho mais lento a definir o teto
do mais rápido**, que é a primeira coisa que o §0 proíbe.

### §59.2 — A auditoria, medida construtor a construtor

Pior `‖∇f‖` sobre uma grelha de 48³ (`the_table_of_who_inflates_the_gradient` + a varredura irmã):

| construtor | pior `‖∇f‖` |
|---|---|
| as 6 primitivas, com e sem `round` | `1,000` |
| `Union` / `Intersection` / `Difference` **`Sharp`** | `1,000` |
| `Shell` (`0,01`–`0,5`) · `Offset` · `Mirror` | `1,000` |
| `Array` (espaçamento `0,1`–`1,0`) · `Radial` (2–64 cópias) | `1,000` |
| `Organic` (`k` de `0` a `1,2`), nas três operações | `1,000` |
| escala uniforme (`0,2`–`4,0`) | `1,000` |
| **`Taper`** (declive `0` a `4`) | `1,000` → **`0,844`** |
| ⛔ **`Union` / `Intersection` `Exact`**, todo `r > 0` | **`1,4142`** |
| ⛔ **`Difference` `Exact`** | `1,000` até `r = 0,1`, **`1,143`** a `r = 0,6` |

⭐ **Três achados:**

1. **Só o arredondamento exacto infla** — e exactamente a `√2`, confirmando a W0 quatro waves depois.
2. **O `Taper` DESCE.** O doc dele avisa que é *"o primeiro modificador que não devolve uma distância
   exacta"*, e a inexactidão é para **baixo** — ele subestima, o que é seguro para a marcha. *Nem
   toda inexactidão é perigo: a que subestima é folga.*
3. ⚠️ **A `Difference Exact` quase escapou.** A 1.ª tabela media `r = 0,1` e lia `1,000` **exacto**.
   Só a varredura no parâmetro a apanhou. *Um valor não é uma família.*

### §59.3 — A lei, e por que ela é grosseira de propósito

`ph2d_field_eval::safe_march_step(doc)`: **sem nenhum arredondamento exacto, anda `1,0`; com
qualquer um, fica no `1/√2` de sempre.**

⛔ **Não se compõe um limite por nó.** Encadear misturas pode compor os factores, e essa pergunta
**não foi medida** — ficar no valor de hoje quando há um `Exact` não piora nada. O que a função faz é
**deixar de castigar quem não o usa**. ⚠️ Uma escultura (`NodeKind::Sampled`) também fica no curto: o
campo dela é interpolado de uma grelha, e ninguém mediu o gradiente da interpolação.

### §59.4 — Os dois gates, e o lado que faltava

- **`the_step_times_the_worst_gradient_never_exceeds_one`** (aritmética): mede o **produto** dos dois
  lados sobre ~70 documentos varridos nos parâmetros. A classificação classifica, a sonda mede, e o
  que se afirma é a relação. ⛔ É este que impede a peça de furar.
- **`the_full_march_step_draws_the_same_piece_as_the_short_one`** (produto): compara a IMAGEM do
  passo inteiro com a do curto e caça o **buraco interior** — um pixel que o curto acerta e o longo
  fura, longe da silhueta. É a coisa que o artista vê, e que norma de gradiente nenhuma exprime.
  ⭐ Ele fecha também a **ligação**: `crate::trace()` tem de desenhar o que a lei mandou, nos dois
  sentidos (documento liso ⇒ passo inteiro; documento arredondado ⇒ passo curto). *Uma lei que o
  caminho do produto não chama não é uma lei.*

⚠️ **E o gate da aritmética tinha só metade da afirmação.** Uma mutação que marcava o `Organic` como
inflador **sobreviveu**: ela só torna a marcha mais lenta. A metade que faltava é a **justiça** —
*um documento cujo gradiente não passa de 1 tem de receber o passo inteiro*. ⚠️ Ela não se pergunta
da família `Exact` (a reserva dela é do CONSTRUTOR, não do valor que uma fixtura lhe deu), e em troca
gateia-se que a reserva é **merecida**: `Union`/`Intersection Exact` têm de medir acima de `1,2`,
senão o passo curto deles virou cerca sem medição atrás.

7 mutações, **7 vermelhas** com os três controles.

### §59.5 — O número

A/B no mesmo processo, máquina calma (`load 1,2`), 640×480:

| arestas | `1/√2` | `1,0` | ganho | pixels ≠ |
|---:|---:|---:|---:|---:|
| 56 | 34,3 | 30,9 | **1,11×** | 2 |
| 168 | 70,9 | 65,0 | **1,09×** | 0 |
| 664 | 263,5 | 235,0 | **1,12×** | 0 |

⚠️ **É `1,1×` e não o `1,41×` que a razão dos passos sugere** — porque o passo só governa a parte da
marcha que é *dar passos*: montar a fita, construir os raios e as normais não encolhem com ele.

⭐ **E ele vale para as DUAS marchas** (a de linha e a por ladrilho), então o quadro inteiro desce
sem a razão entre elas se mexer: a peça de 168 arestas está hoje em **65–67 ms**, contra os **167 ms**
com que a W56 começou.

### §59.6 — ⏸️ O que fica

- ⏸️ **A composição de dois `Exact` encadeados** não foi medida — a lei é conservadora ali de
  propósito, e refiná-la pede a medição.
- ⏸️ **O campo de uma escultura** (`Sampled`) fica no passo curto sem ninguém ter medido o gradiente
  da interpolação trilinear. É a única classe do documento sem número.
- ⏸️ **A `Difference Exact` podia andar `1/1,143` em vez de `1/√2`** — mas o limite dela depende da
  geometria, não só do raio, e um número por-fixtura não é um limite.

## §60 — W57: o vínculo ao desenho VÊ-SE e SOLTA-SE — e o item que não precisava de wave (24/08)

> A W55 fez a peça seguir o desenho e o `§56.5` fechou com três ⏸️: *um contorno de cada vez* · *nada
> na Hierarquia mostra que uma forma está ligada* · *não há gesto para largar nem para religar*.
> ⭐ **Um dos três já estava construído**, e medi-lo antes foi o que impediu a wave errada.

### §60.1 — ⛔ «Um contorno de cada vez» — a composição JÁ o exprimia

A nota pedia *"vários perfis numa peça só (ou **furos como contornos interiores**) pede uma decisão
de produto"*, e eu ia construir uma fonte de perfil com N desenhos.

⚠️ **A `CLAUDE.md` §5.0 manda medir antes:** *"antes de construir um item de lista aberta, MEÇA se a
composição já o exprime"*. Ela exprime. O `ph2d_vec_scene::VecPath` tem `subpaths` + `fill_rule`
desde a **v6** do formato (compound paths), e a `cook_path` percorre `contour_count()` ⇒ **um
contorno interior já é um furo**, e a regra de preenchimento do desenho é que decide.

⭐ **O que faltava não era código: era um gate a dizer que isto funciona** —
`a_drawing_with_an_inner_contour_becomes_a_piece_with_a_hole` mede o campo em três pontos (o centro
é FORA, a parede é DENTRO, o exterior é fora) e afirma que o furo tem o **tamanho desenhado**. ⚠️ Com
o controle da regra ao lado: em `NonZero` e os dois contornos no mesmo sentido o furo **fecha** — é
o desenho a mandar, e é o que o artista já espera do editor vetorial. Duas mutações, duas vermelhas.

⇒ **Sobra a metade que a nota não separava**: vários `VecPath` *separados* escolhidos. Essa continua
⏸️ — e a composição também a cobre pelo outro lado (o `Union` do modelador, ou uma booleana viva do
editor vetorial, que produz **um** path com os dois contornos).

### §60.2 — O selo: o vínculo vê-se sem abrir painel

A linha «Resolution» do painel dizia-o, **e ninguém abre um painel para perguntar**. Quem olhava a
árvore não via diferença nenhuma entre uma extrusão **viva** (que muda quando a curva muda) e uma
**solta** (uma fotografia dela) — duas coisas que se comportam de forma oposta e liam-se igual.
*Um estado que só o inspector conta é um estado que se descobre por acidente.*

⇒ selo **`LNK`** na Hierarquia, tom `Success` (uma capacidade a mais, nunca um aviso). ⚠️ Ele é
**publicado** pela travessia que o módulo já faz por quadro, não perguntado: uma consulta da
`bevy_ecs` pede o mundo **mutável**, e quem pinta a Hierarquia tem-no emprestado no meio do quadro.
*Um empréstimo mutável pedido só para ler é onde um `RefCell` de shell nasce.*

### §60.3 — Os dois gestos, e a fileira que deixou de ser fixa

- **`Unlink`** — a forma deixa de seguir o desenho e **fica com a última que teve**. ⚠️ Tirar o
  componente é tudo o que é preciso, e é o que o torna desfazível **de graça**: o
  `FieldProfileSource` viaja no retrato do mundo e o undo regista por DIFF. Uma cópia da geometria
  «para não perder» seria um segundo dono da forma.
- **`Link Drawing`** — liga ao contorno escolhido agora. ⚠️ **A resolução recomeça no default**:
  herdar o nível do vínculo antigo faria um desenho novo nascer com a finura de outro.

⚠️ **E o `has_profile` teve de virar `profile_pick`.** Até aqui o shell publicava um `bool` — bastava
para os dois botões `+ Extrude`/`+ Revolve` aparecerem. Religar precisa do **id**, e ele não é
redescobrível do lado de lá (quem drena as intenções recebe o mundo, nunca a cena vetorial).
*Publicar um `bool` onde a fonte tinha um id é deitar fora a metade que a próxima feature ia pedir.*

### §60.4 — ⭐⭐⭐ O slot passa a resolver-se em CHAVE

A fileira de ações era **três, sempre as mesmas**, e quem drenava a intenção casava o `slot` por
**número** (`0`, `1`, `ISOLATE_SLOT`). Com verbos que só aparecem às vezes, o índice de um verbo
passa a depender do que foi publicado.

⛔ **E a colisão é concreta:** sem vínculo o slot `3` é *ligar*; com vínculo é *largar*. Um despacho
por número faria o botão executar **o verbo do vizinho**, sem erro de compilação e sem teste
vermelho. ⇒ a lista publicada e a lista despachada são **a mesma função** (`panel::acts_for`).

⚠️ **E o gate disso quase mediu a coisa errada, duas vezes:**

1. A 1.ª asserção exigia *"o verbo novo vai para o FIM"* (`slot >= plain.len()`) — uma afirmação
   sobre a **ordem** da lista, não sobre o perigo. Ela reprovou, e o número que ela imprimiu era a
   colisão real. *Um gate escrito contra o sintoma que eu imaginei mede a minha suposição.*
2. Mesmo corrigido, ele lia **a lista** e não **o botão**: a mutação que devolvia o despacho ao
   `ACTS` fixo **SOBREVIVEU**. A metade que faltava é empurrar a intenção pelo dreno de verdade e
   ver o componente sair. *A mesma família da W56f — uma lei que o caminho do produto não chama não
   é uma lei.*

8 mutações, **8 vermelhas** com os três controles.

## §61 — W58: a seleção múltipla nasce no CANVAS (24/08)

> O `§28` fechou o gesto do gizmo sobre **a seleção inteira** e deixou ⏸️: *"um laço de seleção na
> janela 3D — hoje escolhe-se na Hierarquia com `Ctrl`"*. A capacidade existia; a única forma de a
> **exprimir** era sair da janela onde a peça está.

### §61.1 — O modificador é o vocabulário, e ele já existia

⛔ **Arrastar em espaço vazio NÃO podia virar laço.** Arrastar orbita, e é o gesto principal do
módulo — a pesquisa do navball mede os utilizadores *quase 2× mais rápidos* a arrastar do que a
clicar. Roubá-lo em espaço vazio faria o mesmo botão fazer duas coisas conforme o que estivesse por
baixo, e o artista descobriria a diferença ao girar a peça e ver um rectângulo.

⭐ **`Shift`/`Ctrl` é a MESMA tecla que o canvas 2D já usa** para falar da seleção
(`input_dispatch`: `shift_key() || super_key() || control_key()` → `toggle_in_selection`). Segurada
com um **clique**, ela alterna um objeto; segurada com um **arrasto**, alterna um rectângulo.
*Um vocabulário, não dois.* ⚠️ E ela **perde** para uma alça do gizmo: `Shift`+arrastar uma seta
continua a mover a peça, senão o modificador tiraria ao artista o gesto que ele tem debaixo do dedo.

### §61.2 — Como se pergunta a um CAMPO o que está dentro de um rectângulo

Uma malha traz consigo os vértices, e um laço testa-os contra o rectângulo. Um campo implícito
**não tem vértices**: a única coisa que existe é *"o que está sob este pixel"*. ⇒ o laço faz a mesma
pergunta do clique, **em muitos pixels de uma vez** — e o que ele apanha é exactamente **o que se
vê** dentro do rectângulo, que é o que um laço de viewport faz em todo modelador.

⛔ **E sem içar as compilações, isso não é lento — é impossível.** A `surface_under` compila a árvore
do documento a **cada chamada** (um JIT: `2,3 ms` num contorno de 168 arestas), e o `node_under`
compila **uma árvore por folha por chamada**. Um rectângulo de 300 amostras numa peça de 5 folhas
custaria `300 × 6 = 1 800` JITs — **quase um segundo por gesto**.

| | por chamada, antes | por chamada, agora |
|---|---|---|
| a árvore do documento | **uma por pixel** | **uma** |
| a árvore de cada folha | **uma por folha por pixel** | **uma por folha** |

⇒ `ph2d_field_render::surfaces_under` (a marcha já recebia um **lote** de raios — só faltava a porta)
e `field3d_pick::owners_under`. ⭐ O `node_under` de sempre passa a ser **o caso de um**, pela mesma
porta. *Uma função escrita para um ponto costuma ter o custo no sítio certo — até alguém a chamar
num laço.*

⚠️ **O passo é o recurso, e ele diz de que é**: `LASSO_STRIDE_PX = 6`, porque cada amostra é uma
**marcha de raio**. Um rectângulo de 400×300 pixel a pixel seriam 120 000 marchas; com passo 6 são
~3 300. ⭐ O que o passo pode deixar escapar é uma forma que se veja num quadrado menor que `6 × 6`
px — e a `CLICK_SLOP_PX` do próprio módulo é `3`, então *o laço não é mais cego do que a mão*.

### §61.3 — As três decisões de produto, cada uma com o seu porquê

- **O laço ALTERNA, não substitui.** A tecla que o abriu já significa *«estou a falar da seleção»* —
  um laço que limpasse tudo contradiria a tecla que o pediu.
- **Um laço que não apanhou nada não mexe na seleção.** O artista falhou a mira; limpar seria
  castigá-lo.
- **Um clique aditivo no FUNDO não limpa.** Sem a tecla, o fundo limpa — como em todo modelador.
- ⭐ **E `Shift`+clique sem arrastar é um clique aditivo, não um rectângulo de área zero** — sem
  isto, o gesto morria entre os dois ramos.

### §61.4 — ⚠️ A mutação que apagou uma condição morta (a segunda nesta função)

A 1.ª versão guardava a tecla num campo (`additive_press`) lido no `Down` e consumido no `Up`.
⛔ **A mutação que o punha sempre a `false` SOBREVIVEU** — e a razão é que o campo era
**inalcançável**: um `Down` com o modificador em baixo vai para `Drag::Lasso` **antes** de o ramo que
o lia (`Drag::Orbit`) existir. O campo saiu, e o ramo passa `false` com o porquê escrito ao lado.

⚠️ **É a segunda condição morta que uma prova de mutação apanha nesta mesma função** — a primeira foi
um `nav.is_none()` na W49. *Uma condição que não pode mudar o resultado é uma afirmação falsa sobre o
código para quem o ler a seguir.*

10 mutações, **10 vermelhas** com os três controles.

## §62 — W58b: «não seleciona mais de 2» — a causa não era um teto, era a PERGUNTA (24/08)

> Enio, no smoke da W58: *"o retângulo de seleção não seleciona mais de 2 objetos ao mesmo tempo"*.

### §62.1 — Três hipóteses medidas e refutadas, uma a uma

⚠️ **Nenhuma delas era a causa, e medi-las foi o que evitou curar o sítio errado:**

| hipótese | como foi medida | veredito |
|---|---|---|
| um **teto** no pedido do laço | varredura 2/3/4/5 bolas em fila | ⛔ o laço pediu **todas** |
| um **teto** no consumidor | `toggle_in_selection` × n num `GizmoStateGroup` limpo | ⛔ ficaram **todas** |
| algo **a jusante** come a seleção | quatro quadros de `ecs_bridge` com a seleção viva | ⛔ sobreviveu |

⇒ o defeito não estava em nenhuma das metades que eu tinha gateado. Ele estava na **fixtura**: as
três punham as bolas **em fila**, cada uma com o seu pedaço de silhueta.

### §62.2 — A causa: um laço que só pergunta «o que se vê»

⭐ **`+ Box`/`+ Sphere` nascem no ALVO DA CÂMERA.** Um artista que acrescenta três formas antes de as
mexer tem **três no mesmo sítio** — e a pergunta do laço era *"de quem é este pixel?"*, que numa
união tem **uma** resposta por pixel. Medido, com a sonda do afastamento:

| formas | afastamento 0,00 | 0,05 | 0,15 |
|---:|---:|---:|---:|
| 3 | **1** | 3 | 3 |
| 4 | **1** | 4 | 4 |
| 5 | **1** | 5 | 5 |

*Perguntar só «o que se vê» torna inalcançável, por gesto de canvas, tudo o que está atrás.*

### §62.3 — A cura: a lei do modo de OBJETO

Um objeto entra no laço se **a superfície dele foi amostrada dentro do rectângulo** **OU** se a
**origem dele** projecta dentro dele. É o que o *box select* do Blender faz em modo de objeto, e as
duas metades cobrem casos opostos:

- a **superfície** apanha a forma grande cuja origem ficou de fora do rectângulo;
- a **origem** apanha a forma **tapada** por outra.

⚠️ A origem é a de **MUNDO**, e o `project` a devolver `None` (atrás do olho) **descarta**: um ponto
às costas do artista projecta-se num sítio qualquer do ecrã. ⚠️ E só **folhas** — uma operação não é
um objeto que o artista aponta, a mesma lei do `node_under`.

Com a cura: `3/3`, `4/4`, `5/5` empilhadas. ⛔ **O que ela não promete** (e não é defeito): uma forma
**fora do enquadramento** não é apanhada — um rectângulo de ecrã não pode apanhar o que não está no
ecrã.

### §62.4 — ⚠️ A metade nova tornou a antiga INOBSERVÁVEL

⛔ **Três mutações que estavam vermelhas passaram a SOBREVIVER** ao acrescentar a metade da origem:
sabotar a amostragem da superfície ficava verde, porque a origem apanhava tudo na mesma nas fixturas
de bolas em fila. *Uma metade nova pode apagar o gate da antiga — e uma metade que nenhum gate
observa é uma metade que se apaga sem ninguém reparar.*

⇒ fixtura que as separa, e é **exigente de propósito**: **duas** esferas cujas origens ficam à
esquerda do rectângulo e cujos corpos entram nele em **alturas diferentes**, com o canto de partida
no fundo. Ela prende as três de uma vez (a cobertura da amostragem, o dono de cada pixel, e a própria
metade).

⚠️ **E uma quarta mutação estava MAL ROTULADA:** *"o laço só amostra o canto de partida"* deixava na
verdade uma **coluna inteira** de pé (o laço tem dois `while`), e duas colunas chegavam para apanhar
as duas esferas. *Uma mutação que não faz o que o nome dela diz mede outra coisa.*

15 mutações, **15 vermelhas** com os três controles.

### §62.5 — ⛔ E um TERCEIRO portão de LOC

`shells/desktop/tests/file_loc_caps.rs` (HR-18, **600** LOC para o shell) — o arquivo de gates do
laço chegou a **687**. ⚠️ **A corrida com filtro (`cargo test -p … field3d`) não o alcança**, e foi a
corrida **sem filtro** do pacote que o apanhou — que é exactamente a cura escrita na memória
[`feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate`] uma wave antes.
*A prescrição funcionou; o que faltava era a linha executá-la.* Corte por responsabilidade: o
**gesto** fica, **o que o laço apanha** vai para o irmão.

## §63 — W58c: a moldura do laço estava do lado ERRADO de uma lei escrita uma linha acima (24/08)

> Enio: *"funcionou mas o desenho do retângulo de seleção deixou de aparecer"*.

### §63.1 — A causa, e o parágrafo que já a proibia

A moldura era pintada **dentro** de `if let Some(anchor) = smoke.gizmo` — a guarda de **seleção**.
Sem nada escolhido não há âncora de gizmo, o bloco inteiro é saltado, e o laço mais comum de todos —
**o primeiro**, com a peça acabada de abrir — desenhava nada.

⚠️ **E a lei estava escrita uma linha acima**, no gizmo de navegação: *"ele é pintado **sempre**, e
não dentro da guarda de seleção que vem a seguir: ele diz de que lado do modelo se está a olhar, e
essa pergunta não depende de haver algo escolhido"*. A moldura é da mesma espécie — ela diz **o que
a mão está a fazer** — e eu pus o código do outro lado da lei. *Ler a regra não é o mesmo que estar
do lado certo dela.*

### §63.2 — ⚠️ O gate que faltava, e a régua que teve de ser corrigida

A W58 gateou **o gesto** (o modificador abre o laço, o zero-área vira clique) e **a captura** (o que
o rectângulo apanha). Não gateou a **PINTURA** — e foi exactamente ali que o defeito se meteu.
*As três perguntas de costura desta casa são pintado / populado / clicado, e esta wave só tinha
respondido às duas últimas.*

⛔ **A 1.ª régua media o RECORTE.** Ela comparava o tamanho do `path_data` com e sem laço — e a
mutação que apagava a chamada a `paint_lasso` **SOBREVIVEU**, porque o `push_clip` que a envolve
também escreve um caminho na cena. ⇒ a régua passa a comparar **dois rectângulos DIFERENTES**: o
recorte é o mesmo nos dois (é a área), então qualquer diferença nos bytes vem da moldura, e uma
moldura não pintada dá dois resultados **idênticos**.

3 mutações, **3 vermelhas** — a primeira delas é o defeito reportado, reposto tal e qual.

## §64 — W58d: o laço SOMA, o clique alterna — e a assimetria é a lei (24/08)

> Enio: *"se uma peça estiver selecionada e outra não, o retângulo não seleciona todas, mas inverte a
> seleção — a que estava selecionada é desselecionada"*.

### §64.1 — O raciocínio da W58 estava certo até meio caminho

A W58 escreveu: *"a tecla que o abriu já significa «estou a falar da seleção» — um laço que limpasse
tudo contradiria a tecla que o pediu"*. ✔ Isso justifica **não limpar**. ⛔ E eu dei um passo a mais:
de *"não limpa"* para **"alterna"**, que é outro verbo.

⭐ **A assimetria com o clique é a lei, não uma inconsistência:**

| gesto | alvo | verbo | porquê |
|---|---|---|---|
| **clique** com modificador | **um**, visível | **alterna** | o artista vê exactamente o que vai mudar; alternar é preciso e reversível |
| **rectângulo** com modificador | **vários**, alguns já escolhidos | **soma** | alternar mistura estados que ele **não vê** |

*Um gesto cujo resultado depende de estado invisível não é usável* — o mesmo laço, sobre a mesma
tela, dava resultados diferentes conforme o que estivesse selecionado por baixo. E é o que todo
editor faz: o laço com modificador **soma**.

⏸️ **O laço que SUBTRAI fica por fazer**, e o motivo é de vocabulário: neste app `Shift` e `Ctrl` são
a **mesma** tecla (as duas dizem *"selecção"*), e separá-las aqui criaria um terceiro vocabulário de
modificador — exactamente o que a W58 recusou fazer. Ele pede uma decisão de produto.

### §64.2 — ⚠️ Nenhum dos nove gates do laço podia ver isto

Todos começavam com a seleção **vazia** — e com ela vazia, *alternar* e *acrescentar* são a **mesma
coisa**. *Uma fixtura que começa do zero não distingue dois verbos que só diferem sobre estado
prévio.* ⇒ o gate novo põe **uma peça já selecionada** antes do laço.

### §64.3 — ⛔ E a lei do consumidor estava escrita DUAS vezes (a terceira nesta linha)

As duas primeiras mutações — *o consumidor volta a alternar* (o defeito reportado, reposto tal e
qual) e *o consumidor substitui a seleção* — **SOBREVIVERAM**: o gate lia o `SelectRequest` e
aplicava-o com uma **cópia** da lei escrita dentro do teste.

⇒ `field3d_scene::apply(&mut gizmo, req)` — **uma porta**, chamada pelo `render_loop` e pelos gates.
⚠️ E o gate do clique tinha o mesmo buraco: ele afirmava que o **pedido** era `Toggle`, e não que
aplicá-lo **tira** o que já estava. *É a terceira vez nesta linha que a metade que falta é a de quem
executa.*

20 mutações, **20 vermelhas** com os três controles.

## §65 — W59: a região do corte é o CASCO, não a caixa dele (24/08)

> A W56e deixou ⏸️: *"ladrilhar em `(u, v)` contra o **paralelogramo** em vez da AABB — o único eixo
> que não multiplica a montagem de JIT"*. ⚠️ Isso é uma afirmação sobre o **preço**; o **ganho**
> ainda não tinha número, e esta linha já pagou quatro vezes por construir o que uma nota prescreve.

### §65.1 — A régua primeiro, e ela teve de ser corrigida

| contorno | câmera | fatias | caixa | casco | ganho | área caixa/casco |
|---|---|---:|---:|---:|---:|---:|
| círculo 168 | **de viés** | 2 | 94,3 | **78,2** | **1,21×** | 1,97× |
| círculo 168 | de viés | 4 | 68,7 | 56,8 | 1,21× | 1,58× |
| círculo 168 | de frente | 2 | 54,3 | 50,8 | 1,07× | 1,19× |
| estrela 168 | **de viés** | 4 | 77,7 | **60,6** | **1,28×** | 1,58× |
| estrela 168 | rasante | 2 | 104,7 | 90,7 | 1,15× | 1,52× |

⭐⭐ **A área cai `1,97×` e o ganho é só `1,21×`** — e essa distância é a lição: o corte guarda toda
aresta a menos de `dmax = min_e (máx distância de um VÉRTICE da região a e)`, e o `dmax` cresce com
o **diâmetro**, não com a área. *A diagonal de uma caixa e o comprimento do tubo que ela envolve são
parecidos; as áreas não são.*

⛔ **E a 1.ª versão da régua mediu duas coisas diferentes:** ela comparava o casco **cru** com a
caixa **∩ peça e inflada**, e imprimiu *"área do casco maior que a da caixa"* — impossível para um
casco dentro da própria AABB. Foi esse número absurdo que a denunciou. *Uma régua que compara duas
regiões tem de as recortar e inflar do mesmo modo.*

### §65.2 — A obra, e a ordem que é load-bearing

`ProfileIndex::distance_edges_hull` — a **mesma** regra, com a região a ser um polígono convexo. A
região passa a viajar como **caixa + os oito cantos do tubo** (`tiles::Region`), e o
`RegionCompiler::compile_at` mapeia os cantos pela cadeia de poses (um mapa afim leva ponto a ponto).

⚠️ **Só o `Extrude` a consome, e não é esquecimento:** o `u` do `Revolve` é `√(x²+z²)`, e a região
dele em `(u, v)` é um **rectângulo** por construção — não há polígono a apertar.

⚠️ **Só a DISTÂNCIA a consome.** O conjunto do **sinal** e a **âncora** continuam a sair da caixa: o
enrolamento é um invariante de **caminho**, e o caminho âncora→ponto anda dentro da caixa. *A metade
que compra está na distância; a outra é risco sem prémio.*

⛔ **INFLA e só depois RECORTA.** Ao contrário, o casco espeta para fora da caixa por até `pad` e
deixa de estar contido nela — e um gate de **monotonia** apanhou isso (uma região guardava **mais**
arestas com o casco do que com a caixa, o que é impossível para um subconjunto).

### §65.3 — ⚠️ Três mutações sobreviveram a um gate de amostragem

Elas deitavam fora arestas a mais (medido: a média caiu de `39,7` para `37,8`, o mínimo de `9` para
`5`) e **nenhum** dos 300 × 24 pontos amostrados perdeu a sua aresta vencedora.

⭐ **A causa é a mesma de sempre: num círculo todas as arestas estão à mesma distância do interior**,
então um corte agressivo raramente deita fora *a vencedora*. Uma estrela de 96 pontas ajudou — e não
chegou. ⇒ **parei de caçar fixtura e gateei a PROPRIEDADE**, onde ela é definida:

1. **`seg_hull_dist2` é um MINORANTE verdadeiro** — a distância² região↔aresta nunca passa a de um
   ponto qualquer da região a ela.
2. **`dmax` MAJORA a distância ao vizinho mais próximo em toda a região** — a prova usa que o máximo
   de uma função convexa sobre um polígono está num **vértice**, e por isso ele olha **todos**.

*Uma fixtura de amostras prova o que ela amostrou; a propriedade prova-se onde ela é definida.*

⚠️ **E o controle dessa gate reprovou por medir a lei ao contrário:** ele exigia que a **maioria**
dos pontos ficasse perto da barra, e a folga medida foi de `82%` — que é da **lei**, não da fixtura
(`dmax` é um majorante deliberadamente generoso, tem de valer para o **pior** ponto da região). Ele
passa a exigir que **alguns** cheguem lá.

10 mutações, **10 vermelhas** com os três controles.

### §65.4 — E o teto de LOC voltou a estourar — três cortes

`ph2d-field-eval/src/lib.rs` 936/700 e `profile_index.rs` 728/700. Cortes por responsabilidade:
`hull.rs` (a geometria convexa da região), `affine.rs` (o mapa de poses), `profile_dist.rs` (as
primitivas de distância 2D). `lib.rs` **936 → 687**.

## §66 — W60: reconferir a nota que o custo tornava inalcançável — e o eixo do zoom não existe (24/08)

> `CLAUDE.md` §0: *"quem move o número que tornava algo inalcançável tem de reconferir a nota"*. As
> W56e–W59 baixaram o traçado ~`2,2×`; o teto de `Resolution` foi escolhido **por causa** daquele
> custo. Esta wave é a reconferência — e ela **não muda o número**.

### §66.1 — O teto tinha duas pernas; uma caiu

| nível | arestas | traçado, 23/08 | **traçado, 24/08** | ms/aresta |
|---:|---:|---:|---:|---:|
| 1 | 168 | 184,1 ms | **81,0 / 83,9** | 0,48–0,50 |
| 8 | 472 | 450,3 ms | **201,5 / 209,2** | 0,43–0,44 |
| **16** | 664 | 648,7 ms | **288,4 / 317,0** | 0,43–0,48 |
| 32 | 940 | 900,5 ms | **439,7 / 460,1** | 0,47–0,49 |
| 64 | 1328 | — | **705,6 / 727,8** | 0,53–0,55 |

Pela regra escrita — *meio segundo é onde o artista lê «está a afinar» em vez de «o app prendeu»* —
o teto de hoje seria **32**. ⚠️ E o `128` deu `2 071,8 ms` na 1.ª corrida e `1 184,0` na 2.ª: o
"joelho" **era carga**. *Uma leitura só não é um joelho.*

### §66.2 — ⛔ Mas a perna que segura o número é a do OLHO, e três réguas falharam a medi-la

1. **A régua das bandas SATURA.** Ela conta vizinhos com salto de normal acima de **3°**, e o salto
   do nível 1 já é `2,14°` ⇒ ela devolve o mesmo do nível 1 ao 64 (`91 · 117 · 98 · 102 · 97 · 98 ·
   96`). *Uma régua com limiar não distingue nada que esteja todo abaixo dele* — e os ~100 pixels
   que ela conta são o **aro** da extrusão, uma quina de 90° que é geometria de verdade.
2. **Sem limiar, o aro engole tudo.** O `p99,9` do salto dá `11°` a `half_extent = 0,8` e `79°` a
   `0,4` — plano em todos os níveis.
3. ⛔ **E a câmera ENTRA na peça.** `olho = half_extent / tan(0,3454)` ⇒ `0,556` a
   `half_extent = 0,2`, contra uma bola de raio `0,539`. As três linhas de baixo da tabela eram
   **quadros vazios**.

### §66.3 — ⭐⭐⭐ E a razão de fundo: o facetamento é INVARIANTE À ESCALA

O cozimento **não conhece a câmera** — `cook_path_at` deriva a tolerância de `span × ratio`, com
`span` a extensão do **desenho**. ⇒ o perfil é o mesmo em qualquer enquadramento, e o salto de normal
de um círculo de `n` lados é `360/n` **sempre**.

⇒ *«O knob existe para a peça vista de perto»* — a frase que o doc do default escreve — é uma
afirmação sobre a **SILHUETA**, não sobre a luz. E a W54 já mediu as duas: a silhueta erra `0,079 %`
da peça (invisível) enquanto a normal salta `6,43°` (visível).

### §66.4 — O que fica

- **O número fica em 16**, agora com **uma perna só** e ela nomeada no doc-comment.
- ⏸️ **Subir o teto é decisão de produto**, e a medição que falta pede um contorno de **curvatura
  variável** (uma quina apertada ao lado de um arco longo) — não um círculo, onde a resposta é
  `360/n` e não depende de mais nada.
- ⭐ A sonda fica no repo como **registo das três refutações**, não como régua viva.

## §67 — W61: o PLACAR da malha extraída, contra o estado da arte (24/08)

> Enio: *"o tempo não é problema. Busque a qualidade, o estado da arte no resultado da malha, tente
> superar os melhores do mundo"*. ⛔ **Sem um placar, «estado da arte» é uma intenção.**

### §67.1 — Duas hipóteses minhas, refutadas por LEITURA antes de escrever código

1. ⛔ *"o extrator é Surface Nets e arredonda quina"* — **falso**: é **Dual Contouring com QEF**,
   vértice preso à célula, quads de verdade. E a quina viva está em `116/116` com desvio `0,00` de
   célula desde a W20.
2. ⛔ *"um vértice por célula ⇒ não-manifold"* (a fraqueza clássica do DC) — **falso na medição**:
   `0` arestas com ≠2 faces e `0` de bordo em **todas** as fixturas, incluindo o toro (género 1) e
   uma booleana.

### §67.2 — O placar (profundidade 6)

| peça | faces | ≠2 faces | bordo | `\|f\|` médio (cél) | p99 | aspecto p50/máx | **skew p50/p99** | >60° | não-quads |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cubo | 7 350 | **0** | **0** | **0,0000** | 0,0000 | 1,00 / 1,10 | **0,0 / 0,0** | **0** | **0** |
| esfera | 17 550 | **0** | **0** | 0,0050 | 0,0117 | 1,48 / 2,42 | **26,6 / 55,2** | 120 | **0** |
| toro | 10 680 | **0** | **0** | 0,0088 | 0,0213 | 1,49 / 18,32 | **24,8 / 53,6** | 16 | **0** |
| cubo − esfera | 7 362 | **0** | **0** | 0,0018 | 0,0225 | 1,00 / 7,12 | 0,0 / 52,2 | 33 | **0** |
| desenho puxado | 6 318 | **0** | **0** | 0,0045 | 0,0615 | 1,06 / 7,97 | 0,0 / 74,0 | 80 | **0** |

⭐ **Onde já estamos no nível, ou acima:** topologia **perfeita**; geometria com `\|f\|` médio de
`0,000`–`0,009` **célula** (o cubo é exactamente `0` — analítico); **100 % quads**; quina viva exacta.
⚠️ E a régua da geometria é uma coisa que **nenhum remalhador malha-a-malha tem**: o campo é o
oráculo, e o erro é exacto e não aproximado.

⛔ **O buraco é UM: a FORMA da face.** `25–27°` de enviesamento mediano contra os `4,8–7,1°` do
oráculo de produção `quadwild-bimdf` (a barra que a `line/sculpt3d` calibrou), e `16`–`120` faces com
canto pior que 60° contra **zero**.

### §67.3 — ⛔⛔ E o buraco é ESTRUTURAL: a forma da face segue a GRADE

O **mesmo cubo**, rodado em torno de Z:

| ângulo | aspecto p50 | skew p99 | faces >60° |
|---:|---:|---:|---:|
| 0° | **1,00** | **0,0°** | **0** |
| 15° | 1,07 | 43,8° | 0 |
| 30° | 1,28 | 69,3° | 168 |
| **45°** | **1,41** | **90,0°** | **192** |

⭐⭐ **`1,41` é `√2`** — o quad dual de uma grade sobre uma superfície a 45° é um rectângulo de
`1 × √2`, exactamente. *A forma da face segue a grade, não a superfície*, que é a **definição** de uma
malha dual e não um defeito de afinação.

⇒ ⛔ **Nenhum parâmetro cura isto**, e o repo já o dizia pelo outro lado: a `line/sculpt3d` mediu 16
rondas de relaxação por ajuste de quadrado a levarem a mediana de `27°` para `26°`, pagando `3,4×` as
dobras (`SQUARE_ROUNDS = 0`). *Se mover vértices 16× não move a mediana, o defeito está na
CONECTIVIDADE.*

### §67.4 — A cura existe nesta árvore, e eu corri a metade ERRADA

A cadeia de quads da casa tem **duas** metades, e só uma é a boa:

| metade | o que dá | medido |
|---|---|---|
| **preenchimento por patch** (`ph2d-quadfill::fill`) | `27°` | ⛔ corri-a sobre a nossa malha: esfera `26,6° → 23,2°`, **toro `24,8° → 30,3°`** (pior), faces >60° de `16` para **675** |
| ⭐ **extracção** (`ph2d-gridmap` + `ph2d-quadextract`) | **`5,1°`–`5,5°`** | a classe do oráculo, que ela **ultrapassa** numa das peças |

⚠️ **Que a metade errada reproduza os `27°` do registo é o que VALIDA o arnês** — ela não
contradisse a nota, confirmou-a.

⛔ **A metade certa não é alcançável da minha linha:** ela é orquestrada em
`shells/desktop/src/sculpt3d_history_retopo_extract.rs`, `pub(in crate::sculpt3d)`, e shipa
**desligada** (`PH2D_RETOPO_EXTRACT`). Ligá-la à exportação deste módulo é um movimento **entre
linhas** — e a orquestração que duas linhas consomem pertence a uma **crate**, não ao shell de uma
delas.

### §67.5 — ⏸️ O que fica, e o que é decisão do Enio

- ⭐ **O placar fica** (`the_scorecard_of_the_extracted_mesh`), e com ele «estado da arte» passa a
  ser um número que se re-corre.
- ⏸️ **Ligar a exportação à extracção de quads** é o passo que fecha o único buraco. Ele pede uma
  ordem do Enio porque atravessa duas linhas, e o desenho certo é **içar a orquestração para uma
  crate** que as duas consomem.
- ⛔ **Não afinar o DC** — a tabela do cubo rodado é a recusa medida.

## §68 — W61b: a exportação passa pela cadeia de quads — e ela bate o oráculo (24/08)

> Enio, depois do placar: *"pode fazer"*.

### §68.1 — O número

A malha que a exportação entrega, com a cadeia adoptada:

| peça | extraída (grade dual) | **pela cadeia** | oráculo `quadwild-bimdf` |
|---|---|---|---|
| **esfera** | `1,48` / `26,6°` / 120 péssimas | ⭐ **`1,08` / `6,4°` / 4** | `1,08` / `4,8–7,1°` |
| toro | `1,49` / `24,8°` / 16 | `1,20` / `9,0°` / 9 | — |
| ⛔ cubo rodado 45° | `1,00` / **`0,0°`** / 0 | `1,35` / `17,9°` / 112, **+6 bordo** | — |

⭐⭐⭐ **Na esfera acertamos o aspecto do oráculo ao centésimo (`1,08`) e ficamos a `6,4°` da banda
dele (`4,8–7,1°`)** — dentro dela.

### §68.2 — ⛔ Mas «sempre» seria errado, e o cubo prova

Numa peça **dura** (faces planas, quinas vivas) a grade dual **já é** a resposta certa: o quad pousa
na face e sai a `0°`. O campo cruzado não tem a que se alinhar, e o que ele inventa é pior — **e
abre a peça** (6 arestas de bordo onde havia zero).

⇒ [`ph2d_quadchain::quads_or_keep`] — a cadeia corre e **só troca a malha se a troca for uma
melhoria**, por duas metades que não são pesos arbitrários:

1. ⛔ **Uma peça fechada continua fechada** — bordo ou não-manifold novo é veto **duro**. *Nenhum
   ganho de forma paga um buraco.*
2. Depois disso, troca-se **se a forma melhorar**.

### §68.3 — ⛔⛔ Dois defeitos a jusante, e o que a porta pode fazer

1. **O `ph2d-gridmap` ESTOURA numa malha válida.** Um cubo subdividido — fechado, manifold, 100 %
   quads — dá `index out of bounds: the len is 129 but the index is 157` (`solve.rs:336`).
   ⚠️ **Não é uma pré-condição conferível**: a malha satisfaz tudo o que se sabe exigir.
2. **E estoura noutro sítio numa peça com bordo** (`solve.rs:343`).

⭐ A porta apanha o estouro e devolve `Verdict::Panicked` — ela oferece uma **melhoria opcional**, e
um `panic` a jusante não pode derrubar quem exportou uma peça. ⛔ **Isto não é a cura**: o defeito é
do `ph2d-gridmap`, a `line/quadextract` está **viva sobre aquele arquivo**, e tocá-lo daqui seria
colisão de mesmo-símbolo (`DIRETRIZ` §1.5.5). Ele vai **nomeado no handoff, com a fixtura**.

### §68.4 — ⚠️ Por que uma crate NOVA, e não a migração da que existe

A ordem da cadeia vivia em `shells/desktop/src/sculpt3d_history_retopo_extract.rs`,
`pub(in crate::sculpt3d)` — alcançável por um módulo só. ⛔ E a `line/quadextract` tem **7 commits em
curso exactamente naquele arquivo**.

⇒ `crates/ph2d-quadchain` nasce **aditiva**: a lista de membros da workspace é um **glob**, então não
há sítio central a editar, e o shell da escultura fica **intocado**. ⚠️ Ela é escrita para que aquela
metade adopte esta porta **numa linha**, quando aquela linha quiser. *Duas cópias de uma lei é uma lei
que gate nenhum defende — e esta nasce sabendo disso, e diz no doc-comment.*

### §68.5 — ⚠️ E duas fixturas minhas não continham o fenómeno

- Uma esfera UV feita à mão já sai a **`2,8°`** ⇒ sobre ela a regra devolve «sem ganho», e o gate
  reprovava sobre produto correto. ⇒ o gate da adopção mudou-se para `ph2d-field-eval`, onde a
  **entrada de verdade** (a malha do *Dual Contouring*, `26,6°`) existe.
- E a barra do gate é a do **oráculo** (`≤ 10°`, `aspecto ≤ 1,15`), não «melhor que antes» — *uma
  barra relativa aceitaria `20°`*.

### §57.11 — ⏸️ O que falta para o produto ver isto

- ✅ **A MARCHA POR REGIÃO EXISTE** (§57.14) e dá **1,8×** no quadro. ⏸️ O degrau seguinte é
  ladrilhar também em **PROFUNDIDADE**: um raio de viés varre em `(u, v)` muito mais do que a largura
  do ladrilho, e é isso que separa o `1,8×` medido do tecto de `12,5×`. Hoje a marcha é por **linha de ecrã**; ela tem de passar a pedir uma árvore
  especializada por região e a marchar dentro dela. ⚠️ **A região é em espaço LOCAL do nó, e não do
  ecrã** — logo ela **não depende da câmera**, e as fitas especializadas podem ser construídas uma
  vez por documento em vez de uma vez por quadro (a montagem é 0,07–0,23 ms por região).
  ⚠️ **E há um caso a medir antes**: um raio que atravessa a peça de viés varre uma faixa larga em
  `(u, v)`. Numa peça **fina** (uma extrusão) a varredura é pequena; numa espessa, não — e aí a
  especialização degenera **graciosamente** na árvore completa (correcta, só não mais rápida).
- ⏸️ **Re-derivar o teto de `Resolution`** com o custo novo — o **16** foi escolhido contra o custo
  linear de hoje, e deixa de ser uma parede quando o custo deixar de ser linear.
- ⏸️ A consulta nativa fica como **índice e juiz**; o laço dela não é vectorizado e o `inside`
  (14,5 ns) já é 36 % do que sobra — só volta a interessar se alguém a puser no caminho.

---

## §13 — Aberto

> ⚠️ **Esta lista parou na W56 durante catorze waves** (auditada em 2026-08-26). O que se seguiu
> viveu só em mensagens de commit — e *uma lista de aberto que envelhece manda reconstruir o que já
> está pago*, que foi exactamente o que aconteceu com quatro itens dela em 25/08.

### §13.0 — O que está ABERTO agora (o resto desta seção é o histórico por wave)

| O quê | Estado | Onde |
|---|---|---|
| ✅⭐⭐⭐ **A pré-visualização ALCANÇA 60 Hz** — mediana `~12 ms` contra `16,7`, e independente do `Resolution` | o item nº 1 desde a §70. ⛔ **o `14,2 ms` da §90 foi medido com a câmera PARADA** (corrigido na §91.1); num arrasto real o quadro custa `10`–`27 ms` | §90.4, §91.1 |
| ✅⭐⭐⭐ **A TRAVADINHA do Enio: a cache despejava `1 700` fitas debaixo do cadeado** | ⭐ `94 %` do preço era a **árvore**, que na rota do produto é lastro. Máximo do regime `364,6 → 21,7 ms` (`17×`), despejo no cadeado `~3 000×` mais barato | §91.2–§91.4 |
| ✅⭐ **A lei do cancelamento perguntava ao TAMANHO** desde a W73 | ⭐ passa a perguntar à **espécie**: numa hesitação de um quadro o erro angular vai de `2,97°` para `1,50°` | §91.7 |
| ✅ **`FRAMES_KEPT = 3` era derivado; agora é MEDIDO** | o joelho está lá: `1` e `2` piores, `4` e `6` compram `≤0,4 ms` por `1,4`–`2,5×` a memória | §91.8 |
| ✅⭐⭐⭐ **O CANVAS DIVIDE-SE EM QUATRO VISTAS** (`Ctrl+Alt+Q` ou o chip *Quad View*) | o item que o plano chama *«o produto»* desde a W2; falta a outra metade da frase dele, o **cabeçalho** | §92 |
| ✅ **E só a vista ACTIVA ficava lisa** (smoke do Enio, 27/08) | ⭐ cada viewport comparava-se com o pedido do **activo**; os 5 gates da wave mediam a GEOMETRIA e passaram todos | §92.8 |
| ✅⭐⭐ **Cada vista diz o NOME dela**, derivado da câmera (orbitar a *Top* fá-la *User*) | a metade do cabeçalho que não rouba pixels ao traçado | §92.10 |
| ⏳ O cabeçalho **CLICÁVEL** (menu por vista) | pede a faixa reservada, que obriga a porta do layout a devolver dois retângulos por vista | §92.10 |
| ✅⭐⭐⭐ **As divisórias ARRASTAM-SE** (o cruzamento move as duas) | ⚠️ e a nota que dizia depender do cabeçalho estava **errada** | §92.12 |
| ✅⭐⭐ **O custo de uma EDIÇÃO com a divisão aberta** — medido: quatro juntas custam `3,93×` uma (elas só se fatiam) | ⭐ curado por **ORDEM**: a activa tem prioridade e chega em `64 ms` em vez de `254` | §92.9 |
| ⛔ A **varredura linear** do `TapeCache::get` — **MEDIDA**: `3,0 %` do quadro de movimento, `10,7 %` a `640×360`, e cresce ~quadraticamente | não paga um índice **ainda**; o gatilho está nomeado | §92.11 |
| ✅ **W82: a cache de fitas entre quadros EXISTE** | ⭐ `1,15×`–`1,23×` no quadro de movimento, com `84 %`–`93 %` de acerto e `226` compilações/quadro a cair para `16`–`44`. ⛔ **A estimativa de `1,7×` estava errada por dois motivos nomeados** | §83.7, §83.8 |
| ⏳ A cache contra o **CASCO** e não a caixa | o `1,11×` que ela deixa na mesa; pede um teste em **dois níveis** (a caixa rejeita, o casco confirma) | §83.9 |
| ✅ **O assentar: o que sobrava a compilar era o ANTI-SERRILHADO** | ⭐ `29` fitas por degrau → **`1`**; o custo dele caiu de `1,34×` para **`1,11×`**. A recusa da W70 dissolveu porque a W82 apagou a premissa dela | §84 |
| ✅ O contorno cheio era `3,39×` no assentar de uma peça de resolução ALTA | ⭐ curado pela §86: o assente engrossa até ao erro que a imagem mostra | §84.4, §86.1 |
| ✅ **O decimador do preview apagava QUINAS** — e quem sobrevivia era uma lotaria de índice | ⭐ decima por **GIRO**: a estrela vai de `134` pontos partidos para **`10` exactos**, imagem idêntica | §85 |
| ✅ **O preview pede um ERRO, e a contagem sai da forma** | ⭐ o assente engrossa até `0,5°` de erro de normal: peça de omissão **intocada**, peça de `Resolution` alto **`2×`–`3×`** mais barata a assentar, com `≤3/255` de mudança | §86 |
| ⏳ **A MARCHA** — o que sobra do quadro; custo **por aresta tocada** | ⛔ sobre-relaxação fora (`8,0` amostras por raio) · ⛔ **o perfil como CONSULTA fora**: a fita especializada ganha `2×`–`8×` na região real | §73, §82.1, §87 |
| ⭐⭐⭐ **O quadro de movimento usa `~30 %` da máquina** — o buraco até 60 Hz é de ESCALAMENTO, não de algoritmo | ⛔ o ladrilho **não** é a alavanca (`48 ≈ 64`) · ⛔⛔ **e o JIT também não era**: tirá-lo não mudou a forma da curva (§88.2). A causa está por achar | §82.8, §88.2 |
| ⛔ A **ordenação dos ladrilhos** — **MEDIDA e fechada**: o tecto caiu de `1,76×` para `1,14×` com o ladrilho a `24`, e o custo do quadro **anterior** atinge-o (`1,00×`) | recusada: `1,01×` a 8 threads, e o preço é uma tabela por ladrilho entre quadros. Gatilho nomeado | §92.16 |
| ✅⭐⭐ **O `TILE` estava a ser escolhido pelo TECTO DA MINHA CACHE** | ⭐ tecto **derivado** do que o quadro pede ⇒ `TILE` de `64` para **`24`**, `1,44×`–`1,51×` | §90.2, §90.3 |
| ⏳ O `SLABS` — **reconferido**: a imagem é idêntica nas 5, mas o óptimo MOVE-SE com o tamanho do quadro (`2`–`3` a `426×240`, `4`–`6` a `640×360`) | fica em `4` (o compromisso); decide-se com uma corrida a `load < 5`, e a resposta pode ser **derivá-lo** do tamanho | §92.14 |
| ⛔ O estêncil de **quatro** amostras para a normal (`7 %` de todas as amostras) | **RECUSA MEDIDA** — numa quina de navalha move a normal `14°`–`35°` | §82.6 |
| ⏸️ As duas fatias de FORA: `8,7 %` da montagem por `0,18 %` da marcha | as três saídas medidas, nenhuma se paga | §82.3 |
| ⏸️ Baixar as arestas do contorno a mexer (`PREVIEW_MAX_EDGES`) | preço medido na tabela; muda a FORMA, decisão de quem vê | §73.1 |
| ⏸️ O 2.º degrau do assentar custa `504 ms` numa peça densa | a escada tirou-o do caminho; o número fica | §74.2 |
| ⛔ Reaproveitar o avaliador na **2.ª passagem** (anti-serrilhado) | construído 2×, medido `0,97×`–`1,01×`, **revertido** | §71.4 |
| ✅ Ladrilhar em `(u, v)` contra o paralelogramo | ⭐ **já feito na W59** (o casco); apertar mais está fora do vale | §79.1 |
| ⏸️ Um laço que **SUBTRAI** | ✔ mecanismo medido e as 4 saídas com preço; decisão do Enio | §79.3 |
| ✅ Vários `VecPath` separados | ⭐ era um defeito MUDO, curado: uma peça por forma, todas ligadas | §75 |
| ✅ Religar uma escultura que mudou de sítio | ⭐ `Relink Sculpture…`, com a chave nova escrita no nó | §77 |
| ✅ O `Mirror` «não se consegue demonstrar» | ⭐ **demonstra-se** — o modificador na OPERAÇÃO dobra um filho fora do eixo | §79.2 |
| ✅ A composição de dois `Exact` encadeados | ⛔ **medida: eles COMPÕEM** — a cerca estava errada e a marcha furava | §76 |
| ✅ O gradiente de uma **escultura** | medido: máx `1,0852` (cubo), `30 %` de folga para o `√2` | §78 |
| ⏸️ A barra **demonstrável** da interpolação trilinear é `√3`, e ship-se o `√2` medido | dívida nomeada | §78.3 |
| ⛔ Os níveis de exportação **não** podem mandar na densidade dos quads | recusa MEDIDA, revertida | §70 |
| ⛔ Dois `panic` do `ph2d-gridmap` com reprodutor | **dono: `line/quadextract`** | §68, §70 |
| ✅⭐⭐⭐ **UM VERBO POR FORMA** — a operação sai do grupo e entra em cada objeto (etapa **1** de 3) | a receita lê-se na Hierarquia (`UNI`/`SUB`/`INT`/`BSE`, os selos do vetorial) · ausência = **herança** · a base **semeia** e guarda o verbo dela | §93 |
| ✅⭐⭐⭐ **(2) O RAIO POR OBJETO** — linha **Joint**, derivada, e escrever nela **materializa** o verbo | ⭐ o painel não mudou (as linhas saem do `params_of`) · ⚠️ **Fillet** = as arestas da forma · **Joint** = o encontro, e o grupo passa a dizer `Joint` (é o padrão) | §94 |
| ✅⭐⭐⭐ **(3) O CHANFRO** — `Fillet · Chamfer · Organic`, **três** chips (a aresta viva é o raio zero) | ⭐ uma fórmula só, exacta · ⚠️ **DUAS réguas** (recuo · mordida) e nenhum carácter bate as duas: o orgânico calibra-se pela **mordida**, o chanfro **não se calibra** · ⛔ 2 mutantes sobreviveram e um era defeito VIVO | §95 |

- ⭐⭐⭐ **W97 (§93): UM VERBO POR FORMA — e o desenho já era LEI na metade 2D deste app.** Pedido do
  Enio (*«a hierarquia fica mais confusa criando vários parentescos… colocar a operação dentro de cada
  objeto»*), que é **o mesmo** que ele desenhou para o vetorial em 22/08 — logo esta wave é as duas
  metades do app passarem a falar a mesma língua, com os **mesmos selos**. ⭐ Foi barata porque os dois
  avaliadores **já eram uma dobra à esquerda**: o que estava fixo era só o verbo. ⭐⭐ **As duas idéias
  do pedido são UMA:** uma mistura pertence a uma JUNÇÃO, e é a dobra que dá a cada forma exactamente
  uma — *«um raio por objeto» não tinha onde existir antes*. ⚠️ **Ausência é HERANÇA**, então toda peça
  anterior avalia igual e o seletor do pai vira o **padrão**; ⛔ e o acumulado **não** começa vazio
  (uma subtração no topo apagaria a peça). ⛔⛔ **Um mutante SOBREVIVEU e o achado é maior que o gate:**
  apagar a herança inteira passou em todos, porque eles **comparavam duas construções** e a mutação
  afectava as duas igual — *um controlo que partilha o defeito não é um controlo*; a cura é medir
  contra **oráculo**, e aí os 5 morrem. ⚠️ E o gate de alcance apanhou um defeito **meu** na tabela
  dele: as asserções endereçavam as fileiras por **índice**, e inserir uma no meio re-apontou-as em
  silêncio.
- ⭐⭐⭐ **W89 (§91): A TRAVADINHA TINHA NOME.** De `~12` em `12` quadros de arrasto a cache chegava ao
  tecto e despejava `1 738` fitas **debaixo do cadeado de escrita**: `274,8 ms` num quadro cujo
  orçamento é `16,7`, com as outras 31 threads à porta. ⭐⭐⭐ **`94 %` desse preço era a ÁRVORE que
  cada fita guardava** — e o único leitor dela é o `fork` da rota de bissecção, desligada por omissão.
  *Guardar «para o caso de» tem preço, e aqui ele era um terço de segundo de imagem congelada.*
  Máximo do regime **`364,6 → 21,7 ms`**. ⛔ Três hipóteses caíram por medição (a hesitação da mão · a
  contenção · o despejo em fatia de `1/8`, **pior nos três números**), e ⚠️ **a régua corrigiu-se três
  vezes**: uma sonda que media o boot, outra que media milissegundos onde a pergunta é angular, e a
  varredura da fase que correu com a tempestade ainda ligada. ⛔⛔ **E a §90 estava optimista: o
  `14,2 ms` que reportei ao Enio foi medido com a câmera PARADA.**
- ⭐⭐⭐ **W88 (§90): O QUADRO DE MOVIMENTO ENTROU NO ORÇAMENTO — `14,2 ms` contra `16,7` (⛔ ver a
  correcção da §91.1: esse número é de câmera parada).** Primeiro o
  **oráculo**: gravar o custo verdadeiro de cada ladrilho e **simular** o escalonamento (*simule antes
  de construir*) — a ordem perfeita dá `1,00×` a 8 threads e `1,02×` a 16, logo **a ordem é o
  mecanismo**; e a 32 sobra um piso de `1,52×` que nenhuma ordem passa, porque *uma ordem não parte um
  ladrilho*. ⛔⛔ **E aí achou-se que o `TILE` estava a ser escolhido pelo TECTO DA MINHA CACHE**: com
  `CAPACITY = 2048` fixo, um ladrilho de `32` a `1600×900` pedia `~5 800` regiões e a cache despejava
  metade a cada quadro (`677 ms` num quadro!) — *o «óptimo» era o maior ladrilho que ainda cabia no
  meu tecto*. Com o tecto **derivado do que o quadro pede**, a resposta inverteu-se e o `TILE` foi de
  `64` para **`24`** (`1,44×`–`1,51×`). ⚠️ *Um limite que não diz de que recurso é acaba a escolher a
  constante do lado.*
- ⛔⛔ **W87 (§89): a perda de escala é a DECOMPOSIÇÃO — e a minha cura foi medida e recusada.** O
  discriminador clássico (`T` quadros **independentes**, um por thread e cada um serial, contra **um**
  repartido por `T`) dá `16,99×` contra `11,65×` a 32 threads ⇒ **a decomposição custa `1,47×`**, e o
  resto é o chão honesto da máquina. ⭐⭐ E isso bate o **`1,52×`** que a §82.5 já tinha previsto por
  **contagem** — *o número certo estava na página ao lado, e a §82.8.2 deu a culpa ao JIT porque era o
  mecanismo que estava à mão.* ⛔⛔ **Ordenar os ladrilhos caros primeiro é neutro a pior**: a régua
  que usei (a profundidade da peça sob o ladrilho) está anti-correlacionada — o caro é o da
  **silhueta**, onde os raios passam rasantes, e não o do meio, onde eles acertam cedo.
- ⛔⛔ **W86 (§88): o ciclo medido de ponta a ponta — e uma atribuição MINHA caiu.** O quadro de
  movimento é hoje **`24 ms` seja qual for o `Resolution`** (era isso o objectivo da W84/W85), e
  continua `1,45×` acima do orçamento. ⛔⛔ **E o JIT NÃO era a causa da má escala:** a curva de
  eficiência é a **mesma** com e sem cache (`31 %` contra `30 %` a 32 threads) — a cache desloca o
  nível e não muda a forma. *Um mecanismo confirmado em isolamento não é, por isso, a causa do que se
  via*, e o erro foi dar por causa o mecanismo que estava à mão.
- ⛔⛔ **W86 (§87): a RECUSA da W56 confirmada, e por outra razão — o perfil como CONSULTA perde.**
  Com a montagem quase eliminada, o quadro é quase só marcha, e a direcção nomeada era trocar a fita
  por um BVH (*«`40 ns` contra `155`»*). Medido na região que de facto ocorre: **a fita especializada
  ganha `2×`–`8×`**, e o BVH só ganha na peça inteira a `672` arestas — o regime que a própria W56
  eliminou. ⭐ O mecanismo é geral: *uma estrutura de aceleração amortiza-se sobre o trabalho que ela
  poda; quando outro mecanismo já o podou, ela paga a própria descida por nada* (o cruzamento é a
  ~`150` arestas guardadas, e a especialização guarda `24`–`202`). ⚠️ E a sonda corrigiu-se **duas**
  vezes antes de dizer isto: media a função errada (a linear, não o BVH) e usava regiões **centradas**,
  em que o corte não corta nada.
- ⭐⭐⭐ **W85 (§86): o preview pede um ERRO, e a contagem de arestas sai da forma.** A decimação por
  giro (W84) tornou a contagem uma consequência: o que o orçamento fixa é o **erro da normal**, que é
  o que a luz mostra. ⇒ dois orçamentos — `1,0°` a mexer (que **reproduz o que já shipava**: `168`
  arestas num círculo dão `1,056°`) e `0,5°` ao assentar, que é onde a §85.1 mediu a imagem parar de
  mudar. ⭐⭐ **Uma peça de omissão não muda nada; uma de `Resolution` alto paga `2×`–`3×` menos no
  assentar**, que é exactamente o custo de que o Enio se queixou. ⚠️ E a pergunta que isso abre
  (*«o `Resolution` ainda serve?»*) tem gate: ele governa a **malha exportada**, e o engrossamento do
  preview tem de continuar com **um** chamador.
- ⭐⭐⭐ **W84 (§85): o decimador do preview apagava QUINAS, e quem sobrevivia era uma lotaria.** Ele
  tirava um em cada `k` vértices — certo para **curvatura**, que é distribuída, e errado para uma
  **quina**, que é um vértice só com todo o ângulo dentro. Medido numa estrela: com o
  `PREVIEW_MAX_EDGES = 168` que ship (passo `3`) a normal saltava **`126,8°`**; com passo `2` ou `5`
  não mudava nada — *as quinas caíam em múltiplos de `40`*. ⭐ A cura é decimar por **GIRO**: `400`
  pontos passam a **`10`** (as quinas exactas) e a imagem sai **idêntica**. ⭐⭐ E a §85.1 mediu o que
  o `Resolution` alto compra: a silhueta quase não mexe, a **normal** mexe `∝ 1/n`, e acima de `~336`
  arestas o pixel muda `≤3` níveis de `255`. ⛔ Isso **refuta** derivar o tecto do tamanho do pixel —
  o erro que se vê é angular, e um ângulo não encolhe com a resolução da tela.
- ⭐⭐⭐ **W83 (§84): o que sobrava a compilar no assentar era o ANTI-SERRILHADO — e eram cinco
  linhas.** A 2.ª passagem constrói um avaliador por lote de 64 pixels de borda, e cada um
  **recompilava a árvore inteira**: num assentar a `640×360` isso era `29` das `29` fitas do quadro,
  com a passagem primária a **100 %** de acerto na cache. ⭐ O `fork` passa a **clonar** a fita
  (`Arc<Mmap>`) em vez de a recompilar ⇒ `29 → 1`, e o anti-serrilhado passou de `1,34×` para
  **`1,11×`**. ⚠️ **A W70 já tinha medido isto e achado NEUTRO** — a nota dela dizia porquê («as
  dezenas de fitas desta passagem são ruído ao lado das `917` regiões»), e **a W82 apagou aquele
  `917` no dia anterior**. ⛔ E duas suposições minhas caíram antes do primeiro número: uma peça na
  resolução de omissão **não alterna documento nenhum**, e a minha sonda **avançava a câmera** no
  assentar, que acontece precisamente porque ela parou.
- ⭐⭐⭐ **W82 (§83): a cache de fitas entre quadros — e a parede da §82.9 cedeu `1,2×`, não `1,7×`.**
  Uma fita construída para a região `R` serve toda a sub-região de `R` ⇒ construída para `R`
  **inflada**, ela serve o quadro seguinte: `84 %`–`93 %` de acerto, `226` compilações por quadro a
  cair para `16`–`44`. ⛔⛔ **E o defeito mais caro foi meu:** a 1.ª versão carimbava a idade de uso
  com o cadeado de **escrita** — o acerto era `87 %`, as compilações caíam `7,5×` e o quadro **não
  mexia** (`0,74×` num caso). *Uma cache que serializa os leitores devolve na trava o que poupou no
  JIT.* ⛔ E a minha estimativa de `14 ms` estava errada por **dois** motivos medidos: montagem e
  marcha **sobrepõem-se** (não se somam), e a fita da cache é **mais gorda** (caixa em vez de casco).
  ⭐ O `INFLATE = 1,25` é um **mínimo medido**: `1,00` e `2,00` os dois **perdem**.
- ⭐⭐⭐ **W81 (§82): a marcha ganhou um contador — e a NORMAL era um quinto do quadro.** `21,1 %` de
  todas as amostras de campo são a diferença central (seis por acerto), e ela não estava em conta
  nenhuma — o doc dizia que elas *«saem noutro sítio»* e o sítio não existia. ⛔ **O estêncil de
  quatro foi medido e RECUSADO** (grátis em tudo menos numa quina de navalha, onde move a normal
  `14°`–`35°`). ⭐⭐ **Ladrilhar e fatiar não custam UMA amostra** — `2 121 060` em todos os
  tamanhos, ao dígito, e eu tinha escrito o contrário. ⭐⭐⭐ E o **ladrilho mais caro** vale `1,52×`
  a fatia perfeita de todo o quadro. ⭐⭐⭐ **E a máquina calma disse onde a perda está (§82.8): a
  MONTAGEM não escala** — a mesma fita custa `1,93×` mais CPU a 32 threads que a 1 (o JIT mapeia
  memória executável, e `mmap` é do kernel) —, ela é **`39 %`** do quadro serial e não os `20 %`
  publicados (aquele número foi medido **com** anti-serrilhado, que a W72 tirou no dia seguinte). O
  quadro usa **`36 %`** da máquina; a `76 %` ele caberia no orçamento. ⛔ **O tamanho do ladrilho
  está fechado** — `48 ≈ 64` e a minha própria receita da §82.5 caiu. ⚠️ **Quatro hipóteses minhas
  caíram antes de custarem código, e uma quinta depois de a escrever.**
- ⭐⭐ **W80 (§81): a caça às listas que se dizem exaustivas — a segunda estava na lei mais cara do
  módulo.** O gate que a W53 escreveu para impedir *«uma família de features completa e invisível»*
  percorria uma **lista literal**: uma primitiva nova ficava sem botão e ele ficava **verde**. ⭐ A
  corrente fecha-se agora no compilador (`PrimitiveKind`), e a mutação achou a metade que faltava —
  **duas famílias a partilhar um botão** passavam.
- ⭐⭐ **W79 (§80): o espelho passa a ter TRÊS botões** (`Mirror X/Y/Z`, pedido do Enio) — e a cerca
  que dizia *«roda o nó»* era falsa: o modificador age **antes** da pose, então rodar exigiria um nó
  intermédio. Variantes **append-only** ⇒ zero migração. ⛔ E **duas mutações sobreviveram** aos
  primeiros gates, nos dois sítios que **cortam** (a caixa) e **furam** (a pré-imagem) a peça — mais
  um gate que prometia *«erro de compilação»* sobre uma lista escrita à mão.
- ⭐ **W78 (§79): a auditoria da lista viva — DUAS entradas eram trabalho já feito.** Ladrilhar em
  `(u, v)` estava feito desde a W59 (o casco), e o **`Mirror`** — que o Enio adiou porque *«não se
  consegue demonstrar»* — **demonstra-se**: o modificador vai na **operação** e dobra um filho fora
  do eixo. ⚠️ Sexta nota velha desta sessão.
- ⭐ **W77 (§78): a segunda cerca do passo — e a nota mentia sobre si mesma.** Ela dizia «ninguém
  mediu» e havia um gate a medir: **numa esfera, numa banda**. A generalização (formas com **vinco**,
  a caixa inteira, a barra da marcha) dá `1,0852` no pior caso — `30 %` de folga para o `√2`.
  ⚠️ E a barra **demonstrável** é `√3`: a dívida fica escrita.
- ⭐ **W76 (§77): a escultura que perdeu o arquivo pode ser RELIGADA** — o aviso da W23 era um beco
  (a única cura era pôr o arquivo de volta no caminho exacto). O verbo aparece **só a quem perdeu**,
  a chave **nova** é escrita no nó (senão a peça abre hoje e falha amanhã), e a **pose fica**.
- ⭐⭐⭐ **W75 (§76): a cerca do passo da marcha estava ERRADA — e a cena 1 do smoke marchava acima
  do seguro desde que existe.** Arredondamentos exactos **encadeados** compõem o factor (`1,96` a
  três níveis contra o `√2` que o passo supunha), e ⭐ **um nó de `n` filhos já é uma corrente de
  `n − 1`** (o lowering dobra aos pares) — que é a forma da cena 1. O passo passa a ser `1/√2^k`;
  preço medido: `1,01×`/`1,06×`/`1,23×` a um, dois e três níveis.
- ⭐⭐ **W74 (§75): com duas formas escolhidas, a segunda desaparecia em silêncio.** Duas perdas
  em série (a função cozia só a primeira; a caixa de correio era um slot que a segunda apagava).
  ⭐ Uma peça **por forma**, cada uma ligada ao seu desenho — e não uma peça com todas, porque o
  vínculo vivo aponta para **um** desenho e o componente viaja no arquivo.
- ⭐⭐ **W73 (§74): «ao parar ficou mais lento para alisar» — o assentar vira uma ESCADA.** O
  traçado assente nunca mudou; o que mudou foi **onde o alisamento vive**. Dois degraus: primeiro o
  **mesmo tamanho** com o contorno inteiro e o anti-serrilhado (`131 ms`), depois o cheio (`504`) ⇒
  o que ele espera chega **`3,8×` mais cedo**. ⚠️ E a `wants_antialias` da W72 **morreu com um dia**:
  ela perguntava pelo tamanho, e o degrau que alisa pede o mesmo tamanho do de movimento.
- ⭐ **W72 (§73): o quadro de movimento também não paga o anti-serrilhado** — `35,7 → 26,7 ms`
  (`1,34×`), e o corte muda a **borda de um pixel** em vez da forma. ⭐ E a marcha foi medida antes:
  `8,7` amostras por pixel ⇒ **a sobre-relaxação não tem de onde tirar**; o custo é *por aresta
  tocada*.
- ⭐ **W71 (§72): a montagem é `20 %` do quadro — medida, não dividida** (o produto passou a contar
  o tempo dela). ⛔ Isso **fecha** as duas direcções que a W70 tinha nomeado e manda o alvo para a
  **marcha**. ⭐ E o `SLABS` foi de `2` para **`4`**: ele fora escolhido quando montar custava o
  dobro (`1,09×` no caso do preview, `1,19×` no mais pesado).
- ⭐⭐⭐ **W70 (§71): a montagem de fitas era o quadro inteiro — e três em cada quatro fitas não
  eram avaliadas por ninguém.** Por região especializada pagavam-se **quatro** compilações (árvore ·
  fita float · fita de **gradiente**, que só a exportação consome · e um **`fork`** que recompilava
  as duas) e o traçado avalia **uma**. ⭐ `1,65×` a 168 arestas e `1,92×` a 672, com as fitas de
  gradiente a caírem de `293` para **zero** por quadro. ⛔ E a terceira cura — a 2.ª passagem — foi
  construída duas vezes, medida e **revertida**.
- ⭐⭐⭐ **W69: a CURA do report de fps — o contorno também ENGROSSA enquanto a mão mexe.** A lei que
  o módulo já ship (*grosso a mexer, nítido ao assentar*) aplicada onde o custo estava. Medido a
  `640×360`: `168` arestas `55,3 → 52,1 ms` · `472` `133,4 → 54,6` · `940` **`266,1 → 53,7`**
  (**4,96×**) ⇒ ⭐ **o custo de movimento passou a ser CONSTANTE** (~`53 ms`) qualquer que seja o
  nível, e o teto de `Resolution` voltou a **64** com o recurso certo (o quadro **assente**).
  ⚠️ Ela **DECIMA, não recoze** — recozer exigiria a curva de origem, que vive na cena vetorial —, e
  tem três metades que a impedem de mentir: ao parar volta inteiro · um **furo pequeno fica intacto**
  (⛔ um furo de 6 lados desligava a cura para a peça inteira, e foi uma **prova de mutação** que o
  mostrou) · o laço compara o documento **real**. ⏸️ Fica a base, que é a W70
- ⛔⛔ **W68 (§70): a lentidão que o Enio viu é PRÉ-EXISTENTE, e o teto só a tornou visível.** O
  traçado paga **`0,22 ms` por aresta, CEGO AOS PIXELS** (4× menos pixels ⇒ `1,3×` menos tempo) —
  assinatura de **montar**, não de marchar. ⛔ Mesmo na resolução de omissão o piso é `39 ms` contra
  um orçamento de `16,7`. ⚠️ **A minha nota da W67 media o relógio ERRADO** (o quadro assente, pago
  uma vez, em vez do de movimento, pago sempre). Quatro hipóteses refutadas por medição: o
  recozimento (`23 µs`) · o preview mais grosso (tem **piso**, e volta a subir depois de `D≈6`) · o
  teto sozinho · a montagem **base** (`Hybrid::new`) — ⏸️ e o suspeito que ficou (a especialização
  por ladrilho) foi **ilibado** na W70: sem ela o traçado vai de `58` para `565 ms`
- ✅ **W67 (§70): as duas decisões do Enio, e elas deram respostas OPOSTAS.** O teto de `Resolution`
  sobe **16 → 64** (a régua que faltava é um contorno de curvatura **variável**; `θ ≈ √(8·tol/R)`
  confirma-se em quatro pontos ⇒ *dobrar o nível divide o salto de normal por `√2`*) · e a **escada
  de densidade dos quads é RECUSADA por medição**: `Fine` `49 691 ms` com **42 arestas de bordo**,
  `Max` **`27 min 29 s`** com `316` bordo e `6` não-manifold — *o limite não é o tempo, é a
  TOPOLOGIA da extracção*. ⛔ A 1.ª medição atravessou a própria trava (o `clamp` da
  `tolerance_ratio_for`) e leu «o achatamento saturou»
- ✅ **W66: a exportação diz ONDE a peça está** (só quando a origem cai fora da caixa dela — uma peça
  centrada continua calada), e a auditoria do roteador do módulo achou **quatro** itens que já
  estavam fechados ou desactualizados
- ✅ **W65: a Hierarquia diz qual linha está ISOLADA** (selo `ISO`). ⚠️ A decisão foi a
  **PRECEDÊNCIA** — o selo é um por linha e `ISO`/`LNK` caem na mesma: ganha o `ISO`, que é um estado
  da **VISTA** a explicar por que o resto desapareceu, contra uma propriedade do nó
- ⛔ **W64 (§69): a nota do «traçado 2,4× mais caro» estava errada por 4×, e o suspeito era
  inocente.** O anti-serrilhado custa **22–34 %** (não `2,4×`), e especializar a **segunda passagem**
  por ladrilho é **neutro a pior** — *a especialização paga-se por AMORTIZAÇÃO*: `4 096` raios por
  ladrilho na primária contra `~256` na de borda. **Revertido**
- ✅ **W63: a exportação SAI da thread que desenha** — *«o linux fica cinza»*: a 12 s o loop não
  responde ao ping do compositor e o KDE oferece **forçar o encerramento**. ⚠️ **Declarar o
  congelamento cura a MENSAGEM e não cura o congelamento** — são dois observadores. Bancada com uma
  de cada vez, recusa do segundo em alto, e um sentinela que a liberta no `Drop`. ⭐ E tirar o
  trabalho da thread **abriu a porta que o congelamento fechava** (fechar o app a meio deixava meio
  arquivo com o nome certo) ⇒ gravação por temporário + `rename`, **na pasta do destino**
- ⭐⭐⭐ **W62: a exportação caiu de 8 min 17 s para 6,4 s (77×), e o arquivo que sai é o MESMO.**
  *O alvo da cadeia de quads sai da CAIXA, nunca da densidade* (`target_edge = alpha · diagonal`) —
  a grade fina era mastigada pela fase zero e **deitada fora depois de paga**. ⛔ E não é só preço:
  nas profundidades 7-8 a fidelidade medida no campo *piora* e a esfera é **destruída** (`55,5°`)
- ✅ **W61b (§68): a exportação passa pela cadeia de quads** — e ela **bate o oráculo** na esfera
- ✅ **W61 (§67): o PLACAR da malha extraída** contra o estado da arte — topologia e geometria no
  nível, a **forma da face** não
- ✅ **W60 (§66): reconferir a nota que o custo tornava inalcançável** — e o eixo do zoom **não
  existe**
- ✅ **W59 (§65): a região do corte é o CASCO, não a caixa dele** — `1,21×` menos arestas
- ✅ **W58/58b/58c/58d (§61-§64): a selecção múltipla nasce no CANVAS** — `Shift`+clique **alterna**,
  `Shift`+arrasto **SOMA** (e a assimetria é a lei), apanhando também o que está tapado. ⚠️ «não
  seleciona mais de 2» não era um teto: era a **pergunta**
- ✅ **W57 (§60): o vínculo desenho→peça VÊ-SE e SOLTA-SE** (selo `LNK`, `Unlink` / `Link Drawing`)
- ✅ **W56e/W56f (§58, §59): a PROFUNDIDADE** (`SLABS = 2`, medido) e **o passo da marcha é do
  DOCUMENTO**, não uma constante

- 🔶 **W56 (§57): o perfil deixa de ser uma FITA e passa a ser uma CONSULTA — o ALICERCE está posto,
  o produto ainda não o vê.** ⭐ O gatilho que o `04_resultados_perfis` §7 deixou armado em 19/08
  disparou — e **quem o disparou foi a W55** (168 arestas por omissão, 664 no teto do knob). ⛔ **E a
  cura que aquela nota prescrevia foi REFUTADA por leitura**: ela pedia poda por intervalo, e
  **ninguém avalia intervalos neste caminho** — a fita é ponto-a-ponto, sem ladrilho e sem
  `simplify`. ⭐⭐ O tecto foi medido primeiro (Amdahl): o perfil é **92 %** do quadro no default e
  **98 %** no teto ⇒ uma cura perfeita vale **12,5×** e **49,8×**. ⚠️ E a barra é alta: a fita custa
  **0,95 ns por ponto por aresta** — oito faixas de SIMD com JIT, quase óptima *por aresta*; o que se
  ganha é **tocar menos arestas**. A consulta nova (BVH para a distância + grelha com o enrolamento
  pré-somado para o sinal, ambas exactas, com a fita como **juiz**) dá 1,9× sozinha e **3,8×/5,3×**
  quando o lote é compacto ⇒ **3,1×/4,9× no quadro**. ⛔ Duas metades faltam para o produto ver isto:
  ⭐⭐⭐ **e a saída não era sair da árvore, era ESPECIALIZÁ-LA** (§57.12): a mesma lei com uma
  fracção das arestas — distância pelas que podem ganhar o `min`, sinal por uma **constante** mais os
  atravessamentos da região — dá **32,5×** (168) e **42,7×** (664) e **satura o tecto**, mantendo
  fita única, JIT, gradiente exacto, modificadores e poses. ⛔ A folha nativa foi **recusada por
  produto**, não por velocidade: ela perderia os modificadores e a quina viva. ⏸️ Falta **uma** metade
  — a marcha por região —, e a região é em espaço **local**, logo independente da câmera.
  ⭐⭐ **E o CONSUMIDOR existe** (§57.14): a marcha passa a ser por **ladrilho**, com o raio preso à
  caixa da peça — **1,8×** no quadro (167 → 92 ms a 168 arestas). ⚠️ Não os 5–6× projectados, e o
  mecanismo está medido: um raio de viés varre em `(u, v)` muito mais do que a largura do ladrilho ⇒
  a pegada efectiva é ~`0,4` e não `0,125`. ⏸️ O degrau seguinte é ladrilhar em **profundidade**.
  ⛔⛔ **Três defeitos só o gate de IMAGEM os apanhou** (§57.15) — a região que era a peça inteira
  (lento, e invisível a um gate de paridade), as sondas da normal a saírem da região (90 pixels
  apagados), e ⭐ a regra do atravessamento que tinha de ser **semiaberta**: um caminho que passa por
  um **vértice** contava zero em vez de um, o sinal invertia-se numa cunha fina e a marcha **inventava
  uma superfície**. ⚠️ **E os três sobreviveram a gates verdes por causa da FIXTURE** (§57.16): 21 das
  22 mutações ficaram vermelhas à primeira, e as três sobreviventes eram exactamente estas.
  ⚠️ **E metade da wave foi escrita na árvore ERRADA** — a cwd
  escorregou para o primário e tudo compilou lá; quem o apanhou foi o caminho absoluto do arnês de
  mutação.

- ✅ **W55 (§56): o contorno continua a ser a FONTE — e o knob que faltava era a mesma ausência.** ⭐
  Os dois ⏸️ que a W54 deixou (*"não há knob de resolução"* e *"o nó não se religa ao contorno"*) eram
  **um só**: o `+ Extrude` cozia uma vez e a peça deixava de conhecer o desenho, então afinar a
  conversão era **inexprimível**. O `FieldProfileSource { path, level }` resolve os dois — editar a
  curva remodela a peça, e a linha **Resolution** aparece exactamente onde tem onde escrever. ⚠️ Sem
  cache, de propósito: recozer custa **7 µs** e comparar **0,2 µs** contra um quadro de 16,7 ms, e um
  resumo guardado seria estado derivado a envenenar o undo. ⚠️ O nível guarda a **intenção**, nunca a
  tolerância cozida. ⚠️ O teto do nível (**16**) sai de uma tabela medida, e ela trouxe de graça a
  medida do **próprio instrumento**: o mesmo traçado deu **184,1 ms** a `load ≈ 4,7` e **139,3 ms** a
  `load < 3` — ⭐ *32 % só de carga*. ⛔ E a leitura desenterrou um defeito de W26: `copy_subtree`
  copiava uma lista escrita à mão e **largava o `FieldMods`** — duplicar um cilindro oco devolvia-o
  maciço, em silêncio; a cura leva uma **censura** presa ao registo de componentes. ⛔⛔ E o fecho
  correu a suíte **inteira** do shell pela primeira vez em waves: o gate de **LOC** estava vermelho
  com **quatro** arquivos, três deles antes desta wave (§56.8) — *um gate de árvore não é alcançado
  por um filtro de nome*, que é a irmã da lição do clippy da W44. Curado por **quatro cortes para o
  irmão**, cada um numa fronteira que já existia por dentro. ⏸️ Fica: a tabela do teto pede uma
  corrida com a máquina parada · nada na Hierarquia mostra que uma forma está ligada · não há gesto
  para largar nem para religar o vínculo · um contorno de cada vez
- ✅ **W54 (§55): a régua da suavidade é a NORMAL** — Enio, com duas fotos: *"sem ajustes de
  resolução"*. ⭐ A minha primeira hipótese (a polilinha na silhueta) foi **refutada pela aritmética**:
  ela erra **0,079 % da peça**, invisível. As bandas estão na **LUZ** — o campo de um polígono tem
  gradiente constante por segmento, e a normal salta **6,43°**. A tolerância passa a `1e-4` pelo
  **joelho medido** (degraus **56×** menores; o passo seguinte custa +70 % para nada). ⛔⛔ E a tabela
  de 19/08 estava **desmentida por 2,4×** — o traçado engordou desde a W3 e ninguém reconferiu
  (suspeito nomeado: o anti-serrilhado, que re-amostra a borda 4×). ⛔ O gate que lá estava media
  **arestas** (o custo) e defendia o número velho. ✅ **O knob FECHOU na W55** (§56), e pela ligação
  que esta nota previa. ⏸️ Fica: o traçado 2,4× mais caro por explicar · a normal suavizada, nomeada
  e recusada
- ✅ **W53 (§54): o PERFIL DESENHADO vira peça** — ⛔ `Extrude` e `Revolve` existiam no motor **desde
  a W3**, medidos contra oráculos, e **nenhum botão os alcançava**: uma família de features completa
  e invisível, e o plano chama-lhes a razão de existir do módulo (*"é aqui que o fluxo do MoI
  renasce"*). ⚠️ O gate da W34 não a apanhava por uma **exclusão correta**: ele pergunta *"o painel
  oferece o que a seleção permite?"*, e o que faltava é *"o painel oferece tudo o que o MOTOR sabe
  fazer?"*. ⭐ A ponte já existia inteira (`cook_path_auto`) — a wave escreveu o **gesto**, não
  geometria. Dois gates existentes reprovaram e os dois estavam a trabalhar (um deles **previa** isto
  no próprio comentário). ✅ **O religar FECHOU na W55** (§56). ⏸️ Fica: um contorno de cada vez ·
  nada mostra o eixo do *Revolve* antes do clique
- ✅ **W52 (§53): a viagem NÃO é do *reduced motion*** — Enio: *"o lerp não deve estar vinculado ao
  Reduced Motion. Mas deve ser o único modo."* ⚠️ O smoke da W51 leu *"não funcionou, está como
  antes"* e **o código estava certo**: a preferência dele diz `reduced_motion=1`, e o papel de então
  (`Surface`) morre ali. ⛔ A armadilha estava anotada no `CLAUDE.md` §5 — *"um `reduced_motion=1`
  esquecido reprova smokes sobre produto correto"* — e eu não li o arquivo antes de pedir um juízo
  **sobre movimento**. ⭐ A cura é um **papel novo** (`Role::Viewpoint`), não uma excepção, com o
  critério escrito: *aqui o CORTE é pior do que o movimento*; e ele é estreito, com gate a exigir que
  um percurso comum continue a morrer. ⏸️ Fica: a janela 3D da escultura tem o mesmo problema, e
  ligá-la é decisão da linha dela
- ✅ **W51 (§52): a VIAGEM entre vistas** — pedido do Enio (*"falta um Lerp() rápido […] como no
  blender"*). ⭐ **A curva e a duração são as da CASA**, não minhas: `Role::Surface`, cujo doc
  descreve este caso à letra (*"viaja… e **nunca ultrapassa**; uma roda nomeia um destino, e passar
  dele e voltar lê como a régua a mentir"*) — uma vista nomeada é um destino. O *reduced motion* sai
  de graça. Slerp pelo **caminho curto** (com gate a medir o comprimento do percurso), alvo linear,
  enquadramento **geométrico**. ⭐ **UMA porta** (`fly_to`) em vez das cinco escritas de câmera, e a
  **mão cancela**. ⚠️ O `RefCell` re-entrante mordeu **duas vezes no mesmo dia** — a cura agora é
  estrutural (corpo separado da porta), não memória. ⏸️ Fica: a linha do laço que serve o progresso
  não é alcançável de um gate · o custo do traçado durante a viagem não foi medido
- ✅ **W50 (§51): a MOLDURA do app empurra o gizmo** — Enio, no smoke da W49: *"fica escondido entre
  botões"*. A área é o viewport inteiro e a moldura pinta por cima; o gizmo passa a viver na **parte
  livre**. ⭐ A lei é a **fuga mais barata** (um painel alto sai pela direita, uma faixa larga sai
  pelo topo) — a primeira, por *«toca a aresta»*, contava um painel da altura toda como faixa do topo
  e baixava o gizmo 600 px. Iterativa, com gate de ordem, e **local** (a tira do Flip não o move).
  ⚠️ Acessor novo `WidgetStore::panel_rects()` em vez de uma segunda lista de ids. ⛔⛔ **E o arnês de
  mutação deu por apanhada uma mutação que não apanhou**: o gate rebentava sozinho (`RefCell`
  re-entrante) e os dois controles positivos passaram — faltava o **verde antes do vermelho**, que a
  memória do projeto já exigia e o arnês não. ⏸️ Fica: a chamada que publica a parte livre não é
  alcançável de um gate
- ✅ **W49 (§50): o GIZMO DE NAVEGAÇÃO (bolas de eixo)** — pedido do Enio, e ele mandou **pesquisar
  antes de construir**. ⛔ O ViewCube do Fusion está sob patente **viva até 2029-03-06** (Autodesk,
  US 7.782.319). ⭐ E a própria pesquisa da Autodesk mede que o ganho vem do **arrasto**, não do
  cubo (*"quase 2× mais rápidos, independentemente das representações examinadas"*) — por isso
  arrastar orbita é o gesto principal. Decisão dele: bolas de eixo. Números **derivados** do gizmo
  3D (raio de agarre, espessura, cores dos eixos). ⚠️ Duas mutações sobreviventes acharam buracos
  reais: **espelhar o widget na vertical passava em tudo**, e a guarda `nav.is_none()` era **código
  morto** (apagada, não gateada). ⏸️ Fica: sem letras nas bolas · sem cantos/arestas (as 26 direções
  do cubo) · o salto não é animado · sem gate de pintura
- ✅ **W47 (§48): as SEIS VISTAS existem, e a câmera passou a ser alcançável** — o item que o plano
  chama **⭐ «É o produto»** desde 19/08. ⭐ A medição mudou o entregável: o módulo **já pinta no
  viewport inteiro**, então um cabeçalho dentro do canvas seria uma **segunda superfície de UI** do
  mesmo módulo (hit-test, ids e lei de alcançabilidade próprios) — os controles vão para o **painel**,
  que é a decisão que o plano já tinha tomado. ⛔ O buraco real: **a câmera nunca passou pela lei da
  W34** — as vistas não existiam, e a lente e o enquadrar eram alcançáveis só por tecla. ⚠️ Lemos as
  **teclas** do Blender, não os **eixos** (ele é Z-up, nós Y-up), e o gate confere o **eixo do olho**,
  não a aritmética. ⭐ A vista é **derivada** da orientação, nunca guardada. ⏸️ Fica: sem vista oposta
  rápida nem *enquadrar a seleção* · as vistas não forçam a lente paralela (produto) · quad-view fora
- ✅ **W46 (§47): a peça nasce ENQUADRADA, e o `Home` passou a encontrá-la** — o ⏸️ que a W45 deixou
  uma hora antes. ⚠️ Uma peça longe da origem abria **fora do quadro**, e a tela voltava a ficar
  vazia: *o mesmo sintoma que a W45 existiu para curar, por outro caminho*. ⛔ E o `Home` **não** era
  a saída (eu disse ao Enio que era): ele punha o alvo na **origem**. A lei é a da referência — no
  Blender `Home` é *View All*; tínhamos herdado a tecla e metade do significado. ⭐ O bordo é o
  **mesmo** do exportador (W33). ⭐ A folga saiu de uma varredura: `1,00` deixa **144 pixels** da
  peça na moldura (a lente é convergente e o lado virado à câmera projeta maior que o raio), e
  **1,10** zera. ⚠️ A varredura só o disse depois de a fixtura mudar para uma esfera **sozinha** —
  com a união de duas, todas as folgas davam zero. ⏸️ Fica: enquadrar a **seleção** · o salto não é
  animado · o teto de `half_extent` trunca uma peça enorme **em silêncio**
- ✅ **W45 (§46): um projeto que traz uma PEÇA abre o painel dela** — o ⏸️ da W35. ⛔ **A porta estava
  trancada por dentro:** o pedido de abrir só era aceite com o módulo **armado**, e o único caminho
  que o arma é a visibilidade do painel — *para pedir a abertura era preciso já estar aberto*. A obra
  atravessava o arquivo (W35) e a tela ficava vazia, indistinguível de a ter perdido. ⚠️ O load
  deixa a **pergunta** e o quadro responde-a (o mundo vive no `gfx`, e o load corre sem janela — a
  forma é a do `sculpt3d_install_pending`). ⭐ E a pergunta certa foi escrita por uma **mutação
  sobrevivente**: *"há raiz"* e *"há nó"* são a mesma coisa (o `spawn_doc` dá `FieldNode` à raiz
  sempre), e o que separa é o **cozimento** — *«há alguma coisa PARA VER?»*. ⏸️ Fica: um projeto com
  peça **e** escultura abre a escultura e nada diz que também há peça · o painel abre mas não
  **enquadra** a peça
- ✅ **W44 (§45): o isolamento DIZ-SE e SAI-SE de qualquer sítio** — os dois ⏸️ da W38, e a medição
  encontrou um terceiro item que era o defeito: ⛔ **o único sinal estava preso à SELEÇÃO**
  (`isolated == selection.first()`), então isolar `A` e escolher `B` apagava-o, e com a **raiz**
  escolhida a fileira inteira desaparece — nem indicador, nem porta. ⚠️ E o *toggle* fora escolhido
  com a razão *"a porta que o artista não acha quando a cena some"*: **o chip não a cumpria**, e é
  justamente para a raiz que se vai quando sumiu tudo. ⭐ `Shift+I` (a tecla do módulo irmão, lida)
  responde de qualquer sítio, com lei **própria** — a tecla é global (*dentro ou fora*), o chip é de
  uma linha (*mostra-me ESTE*), e há `assert_ne!` a prender a divergência. ⏸️ Fica: a **Hierarquia**
  não tem marca própria (quem anuncia é o painel) · isolar **vários** nós
- ✅ **W43 (§44): a VISTA sobrevive ao fechar o painel** — o ⏸️ que a W42 deixou. ⭐ E não era «a
  câmera»: o próprio `Smoke` já classificava **cinco** campos como *estado de vista*, em três
  doc-comments distintos (*"é estado de **vista**, e não do documento"*) — *um doc-comment repetido
  em N campos é uma estrutura por nascer*. A `cam` viaja com o `manual`, ou o prato desfaz o ângulo
  restaurado no quadro seguinte. Um campo novo no `Smoke` é **erro de compilação** no sítio onde a
  pergunta *"vista ou cache?"* tem de ser respondida (destructuring sem `..`). ⭐ E a W42 tornou
  escrevível o gate comportamental que a W38 declarara impossível: desarmar limpa-se a si mesmo.
  ⏸️ Fica: a vista é de **processo** (não é salva no arquivo), e a Hierarquia continua sem mostrar
  que há um isolamento em curso — agora com mais alcance, porque ele também atravessa um fecho
- ✅ **W33 (§34): a caixa da grade do EXPORTADOR passou a ser a da peça** — uma peça fora de
  `[-1,1]` era cortada em silêncio, e uma peça pequena gastava resolução em vazio (**>3×** de
  ganho medido). ⏸️ Fica: a exportação **não diz o tamanho** da peça, e agora que o bordo existe
  isso é uma linha
- ✅ **o OLHO da Hierarquia passou a valer na W28** (§29) — esconder um nó tira-o da peça, e um
  grupo leva a subárvore consigo; um nó escondido não tem gizmo nem anda com a seleção.
  ⏸️ Fica **isolar** (mostrar só o escolhido), que é o gesto irmão
- ✅ **o CADEADO passou a valer na W29** (§30) — pelo predicado da casa, com a metade do
  `GroupedChildren`. ⛔ Ele **não** tranca os números do painel, e isso é lido do doc do componente,
  não decidido aqui
- ✅ **arrastar uma linha na Hierarquia deixou de TELEPORTAR a peça na W30** (§31) — a lei do
  mundo-preservado da casa não alcançava o tipo da pose deste módulo. ⏸️ Fica: re-parentar muda
  a **peça** (um cilindro dentro de uma subtração passa a cortar) e ninguém o diz
- ✅ **W42 (§43): DESARMAR não desarmava** — a W40 fechava o painel e o módulo continuava a comer o
  ponteiro. Duas notas a mentir: o doc do `with_smoke` prometia inércia que o código não fazia (o
  estado de armado só era lido **antes** de o smoke nascer), e a bandeira **travava ligada** por uma
  razão que **dissolveu na W5** (o medo era perder a peça; ela é uma árvore de entidades desde
  então). ⭐ E a ordem do despacho explicava a assimetria: a escultura vê o clique **antes** da
  modelagem, o Vector **depois**. ⏸️ Fica: fechar o painel larga a **câmera** (a peça não)
- ✅ **W41 (§42): o CRASH que o smoke da W40 encontrou na escultura** — apagar a última peça e
  clicar derrubava o app (`index out of bounds`, `sculpt3d_input.rs:173`). A cena vazia é um estado
  **legítimo** (o `delete` promete o Ctrl+Z) e os caminhos de gesto supunham-na impossível. ⛔ A cura
  completa são **42** indexações sem guarda em 9 arquivos da `line/sculpt3d`; esta linha fecha **a
  porta que o artista bateu** e nomeia o resto. ⏸️ Fica: as outras 4 portas · e o pânico terminou em
  **SIGSEGV** (o app crasha ao crashar, e perde o relatório)
- ✅ **W40 (§41): o modelador CEDE o canvas** — Enio, 22/08: *"não consigo esculpir nada pois o modo
  de modelagem permanece interferindo"*. ⚠️ **É o mesmo report que a escultura já pagou duas vezes**
  (09/08 e 17/08), um nível acima: lá *"o ponteiro cedia e o teclado não"*; aqui **nenhum dos dois**.
  A lei é *tomar o canvas liberta quem o tinha*, em duas metades simétricas e **de borda** (contínua
  criaria impasse: não há gesto de largar uma ferramenta). ⏸️ Fica: reabrir o MODEL com ferramenta em
  mãos deixa as duas de pé · o modelador não tem pill próprio (o painel É o interruptor)
- ✅ **W39 (§40): a escultura da cena entra SEM passar pelo disco** — botão `+ Sculpt from scene`,
  oferecido só quando há uma. ⛔ **E a medição proibiu o «vivo» contínuo**: voxelizar custa
  **229–389 ms** a 128³ (1,5 s a 256), contra um quadro de 16,7 — são 14 a 23 quadros por pincelada.
  A decisão já estava escrita no doc da `DEFAULT_RESOLUTION` (*"o custo é pago uma vez, na
  importação"*). ⏸️ Fica: a escultura **não se atualiza sozinha** e nada o diz · **uma** escultura
  por peça (a chave é fixa) · o reencontro ao reabrir o projeto **não foi smokado**
- ✅ **W38 (§39): ISOLAR (mostrar só o escolhido) e o grupo que nascia MUDO** — os dois itens da
  fila. ⭐ A lei do isolar foi **lida** no módulo irmão (toggle · não entra na história · isolar
  «nada» é recusado), e o mecanismo era **zero**: o `cook` já cozia *"a subárvore de `root`"*, então
  isolar é cozer a partir daquele nó. Faltava **uma linha** — a pose da cadeia acima, senão a peça
  isolada salta para a origem. ⏸️ Fica: sem tecla · sem gate comportamental da costura painel↔smoke
  (o estado do smoke é `thread_local` e armá-lo contaminaria os vizinhos) · nada mostra na
  **Hierarquia** que há um isolamento em curso
- ✅ **W37 (§38): a mensagem deixou de viver UM quadro** — o diálogo de arquivo congela o loop, e o
  `wall_dt` do quadro seguinte cobrava esse congelamento ao relógio do chrome, matando o toast antes
  de alguém o ver. ⛔ **É defeito da CASA:** 25 chamadas de diálogo em 12 arquivos, e esta wave liga
  as 2 do módulo. ⏸️ Fica: as outras **23** (tabela no §38.3), e se um resultado de exportação devia
  durar mais do que os 3 s de um aviso passageiro (produto)
- ✅ **W36 (§37): a exportação diz o TAMANHO da peça** — e havia **dois** números disponíveis, não
  um: o bordo é **andaime** (cúbico e conservador; até **28×** o eixo curto de uma peça fina), e o
  que se diz é a caixa da **malha que saiu**. ⛔ Numa esfera os dois coincidem, então uma conferência
  feita só nela teria confirmado o número errado. ⏸️ Fica: nada diz **onde** a peça está (o canto
  mínimo), que importa a quem monta várias peças no mesmo arquivo
- ✅ **W35 (§36): a peça ATRAVESSA o arquivo — e a nota que dizia o contrário era velha.** Ela é uma
  árvore de entidades, o `ProjectState` é o mundo inteiro, e o `PROJECT_SCHEMA` **não se mexe**. O
  que faltava era estreito: a memória de *"já tentei ler esta escultura"* era do **processo**, então
  um arquivo consertado no disco nunca era relido — e o segundo silêncio era idêntico ao de quando
  estava certo. ⏸️ Fica: o Ctrl+S de verdade **não é alcançável de um gate** (o save exige `gfx`), e
  o módulo **não se abre sozinho** quando o projeto carregado traz uma peça
- ✅ **W34 (§35): o painel passou a oferecer EXATAMENTE o que o gesto faz** — a fileira de operações
  aparece com **uma** forma escolhida (o gesto de criar grupo, que a W31 escreveu e ninguém
  alcançava), e a de *Duplicar/Apagar* deixa de ser pintada sobre a **raiz**, que as recusa. A lei
  vale para as três fileiras que dependem da seleção. ⏸️ Fica: um controle que dependa de outra coisa
  que não a seleção não é apanhado por ela
- ✅ **W31 (§32): um objeto largado em cima de outro deixou de SUMIR** — só uma operação pode ter
  filhos, e a forma anfitriã é promovida a grupo (a peça na tela não muda). E **criar grupo** passa
  a ser um gesto: uma forma sozinha + um botão de operação — ⚠️ **alcançável só desde a W34**, que é
  quando o painel passou a pintar a fileira nesse caso. ⏸️ Fica: ninguém **diz** que um grupo
  nasceu, e não há grupo VAZIO
- ✅ **orientação Global/Local FECHOU** na W7 (§6)
- ✅ **rotacionar e escalar FECHARAM** na W6 (§5)
- ✅ **clicar na peça para selecionar FECHOU** na W7 (§6) — e o custo está medido: **0,10 ms**
- ✅ **snap e leitura numérica FECHARAM** na W8 (§7)
- ✅ **o undo de um arrasto FECHOU** na W6 (§5) — e a nota que estava aqui estava **errada**: a lei
  do shell já existia e o que faltava era o módulo dizer-se. *Meça o mecanismo antes de construir o
  que a nota prescreve.*
- ✅ **duplicar e apagar FECHARAM** na W11 (§10)
- ✅ **a rotação em números FECHOU** na W14 (§14) — e com ela o piso de toda linha
- ⏸️ **o ESPELHO não se consegue demonstrar** (Enio, smoke da W17): ele dobra em torno do centro do
  nó, e toda peça das cenas — folhas *e* grupos — é simétrica. O verbo está correto e gateado; o que
  falta é um alvo descentrado, ou um pivô de espelho autorado. Adiado por decisão dele
- ✅ **draft/taper FECHOU** na W18 (§19), e a W4 do plano com ele — o primeiro operador não-exato do
  módulo, com as duas tabelas ao lado do número
- ✅ **a W5 FECHOU**: o motor na W21 (§22), a **autoria** na W22 (§23) e o **regresso** na W23 (§24) —
  o botão `+ Sculpt…` traz um arquivo de malha para dentro da peça, a booleana corta-o, e reabrir o
  projeto **regenera-o do arquivo que o nomeia** (o que não voltar **fala**, com o nome do arquivo).
  ⏸️ Fica: um arquivo que **mudou de sítio** não se reencontra (religar pede UI, e é a pergunta que o
  app ainda não faz por asset nenhum), a ligação à escultura **viva** do módulo 3D (hoje o vínculo
  passa pelo disco e acorda ao **abrir**), e os modificadores sobre uma escultura
- ✅ **a LENTIDÃO que o Enio nomeou no smoke da cena 6 FECHOU na W24** (§25): a resolução do preview
  passa a sair da **medição** — 4,2× e 7,3× mais rápido em movimento, sem um segundo motor. ✅ E a
  espera que sobrava (até 121 ms) **FECHOU na W32** (§33): um refinamento cede à mão, um traçado de
  movimento nunca. ⏸️ Fica: o trabalho abandonado é deitado fora, não reaproveitado
- ✅ **o ERRO na UI FECHOU na W25** (§26) — a peça que não cozinha diz porquê, e o clique que a
  apagava (um modificador sobre uma escultura) deixou de existir. ⏸️ Fica: o aviso não aponta **qual**
  nó é o culpado
- ✅ **digitar o número durante o arrasto FECHOU na W26** (§27) — a ficha passou a aceitar, o número
  é o **total** e `Esc` desfaz o gesto inteiro. ⏸️ Fica: escolher o eixo por tecla (exige gesto modal)
  e contas na entrada
- ✅ **o gesto passou a ser da SELEÇÃO INTEIRA na W27** (§28), e com ela o **pivô do centro da
  seleção**. ⏸️ Fica o pivô **escolhido** (o cursor 3D do Blender), que é produto e pede a UI que o
  põe lá — e um laço de seleção na janela 3D (hoje escolhe-se na Hierarquia com `Ctrl`)
- ✅ **perspectiva FECHOU** na W15 (§16) — entrou num sítio, como a nota previa, e revelou que o
  raio era construído em dois. `Numpad5` alterna as lentes

## §69 — ⛔⛔ A nota do «traçado 2,4× mais caro» estava errada por 4×, e o suspeito era inocente

O §55.3 deixou em aberto *"o traçado ficou ~2,4× mais caro desde a W3 e ninguém o reconferiu — o
suspeito nomeado é o **anti-serrilhado adaptativo**"*. Medido hoje (`measure_the_edge_pass_share`,
release, 640×480, mediana de 7 corridas **no mesmo processo**):

| arestas | s/AA | c/AA | a quota do AA |
|---|---|---|---|
| 64 | 29,0 ms | 36,4 ms | **26 %** |
| 128 | 46,3 ms | 60,9 ms | 32 % |
| 256 | 86,8 ms | 105,9 ms | 22 % |
| 512 | 171,3 ms | 229,6 ms | 34 % |

⭐ **O AA custa 22–34 %, não os 140 % de que era acusado.** E o número da W3 (`24,1 ms` a 64
arestas, medido **antes de o AA existir**) compara-se com os `29,0 ms` de hoje **sem AA**: **1,2×**,
não `2,4×`.

⚠️ **A nota envelheceu porque as waves de perf seguintes a desmentiram e ninguém a reconferiu** — a
W56e (fatias de profundidade, `2,5×`), a W56f (o passo do documento, `1,10×`) e a W59 (o recorte
pelo casco, `1,21×`). *Quem move o número que sustenta uma nota tem de reconferir a nota* — e desta
vez quem o moveu fui eu, três vezes.

### §57.1 — ⛔ A régua estava errada ANTES da resposta

A primeira leitura desta pergunta subtraiu **dois relógios de ~30 ms**, medidos em **corridas
separadas**, para ler um delta de ~10 ms. Ela devolveu `+34 %` numa corrida e `+22 %` noutra
**sobre o mesmo código**. *Subtrair dois números ruidosos não dá um número menos ruidoso: dá a soma
dos dois ruídos.*

⚠️ E a lição já estava escrita **neste crate**, na porta irmã do passo (`trace_stepped_for_test`):
*"ela existe para que as duas respostas sejam medidas no mesmo processo — entre duas corridas desta
workstation a montagem mexeu-se `14,4 → 22,1 ms`, e um A/B nessas condições mede o relógio da
máquina, não a mudança"*. Foi paga outra vez.

### §57.2 — ⛔ RECUSA MEDIDA: especializar a segunda passagem por ladrilho

Achado real por trás dos 26 %: os pixels de borda são **3,0 %** dos raios e custam **~11×** o de um
raio primário. Duas causas, e só uma tinha cura aparente:

1. **Geometria, sem cura:** um raio de silhueta é **rasante**, e a esfera-marcha dá muitos mais
   passos quando caminha quase paralela à superfície.
2. **A W56e especializou a marcha primária por ladrilho × fatia e deixou a segunda passagem a
   marchar a árvore INTEIRA.** *A cura que acelerou metade da conta inflacionou a fracção da outra
   metade.*

⇒ Implementei (2): agrupar os pixels de borda por ladrilho e marchá-los com as **mesmas**
fronteiras de fatia da passagem primária (`tile_t_range` / `slab_bounds` / `slab_region`). A/B no
mesmo processo, mediana de 7:

| arestas | s/AA | c/AA simples | c/AA por ladrilho |
|---|---|---|---|
| 64 | 29,0 ms | **36,4 ms** | 39,0 ms ⛔ |
| 128 | 46,3 ms | 60,9 ms | 60,0 ms (empate) |
| 256 | 86,8 ms | **105,9 ms** | 109,7 ms ⛔ |
| 512 | 171,3 ms | 229,6 ms | 225,1 ms (empate) |

⛔ **Neutro a pior. REVERTIDO.**

⭐ **O mecanismo é a AMORTIZAÇÃO, e ele é aritmético:** a montagem da fita custa o mesmo por
ladrilho nas duas passagens, e a primária dilui-a por **4 096** raios (`64×64`) enquanto a de borda
só tem **~256** naquele ladrilho (a silhueta atravessa-o em ~64 pixels × 4 amostras). **16× menos
raios para amortizar a mesma montagem** — e é exactamente essa razão que come o ganho da árvore
menor.

⏸️ **A saída que sobra, não construída:** **reaproveitar** as fitas que a passagem primária já
montou, em vez de as remontar. Ela remove a montagem em vez de a diluir, mas paga memória (480
ladrilhos × 2 fatias de fita vivas ao mesmo tempo) e um cache com tempo de vida. ⚠️ O tecto do ganho
é **uma parte dos 26 %** do quadro **assente** — que se paga uma vez, quando a câmera pára, e que
desde a W24 já não é o preço interativo. *Medir antes de construir vale para a segunda tentativa
também.*

## §70 — As duas decisões do Enio (26/08): o teto SOBE, a escada de densidade é RECUSADA por medição

### §58.1 — ⭐⭐⭐ `MAX_PROFILE_RESOLUTION` 16 → 64

A W60 deixou o teto com **uma perna só** (o olho) e disse que subir precisava de um contorno de
**curvatura variável** para ser medido. Ele não existia. A fixtura nova é uma **elipse `4:1`**, cuja
ponta é `16×` mais curva que o lado, e a régua é o **maior salto de normal**
(`the_table_of_the_sharpest_corner`, release, máquina calma a `load 1,8`):

| nível | arestas | salto MAX | salto mediano | traçado |
|---:|---:|---:|---:|---:|
| 1 | 112 | 8,84° | 2,09° | 43,6 ms |
| 8 | 320 | 3,11° | 0,73° | 109,6 ms |
| ~~16~~ | 448 | 2,21° | 0,52° | 145,1 ms |
| 32 | 636 | 1,55° | 0,37° | 216,6 ms |
| **64** | **896** | **1,11°** | **0,26°** | **303,2 ms** |
| 128 | 1 268 | 0,78° | 0,19° | 483,4 ms |

⭐ **A lei é `θ ≈ √(8·tol/R)`** e a tabela confirma-a em quatro pontos: dobrar o nível divide o salto
por `√2`. ⇒ **cada duplicação do teto deixa desenhar um canto duas vezes mais apertado** antes de
facetar — com a barra de `3°` das fotos, o limite era `~5,5:1` no `16` e é `~22:1` no `64`.

⭐⭐ **Não há joelho, e é por isso que o número é o RELÓGIO**: cada degrau custa `×√2` e compra `×2`,
então a razão benefício/preço **melhora** ao subir. O `128` cabe na regra de meio segundo (`483 ms`)
e é o **último** que cabe; o `64` fica a `303 ms`, e a folga é para uma cena com **mais de uma
peça** — ⚠️ premissa **declarada**, não medida, e é ela que separa o `64` do `128`.

⛔ **A 1.ª medição atravessou a PRÓPRIA TRAVA.** A `tolerance_ratio_for` clampa no
`MAX_PROFILE_RESOLUTION`, então os níveis acima recebiam a tolerância do `16` e a tabela saía com
`448` arestas quatro vezes — lida como *"o achatamento saturou"*. *Uma sonda que atravessa o limite
que quer medir mede o limite.* A cura foi partir a lei em duas: o **span** continua a ser o do
produto (`span_of`, agora público), o **teto** é o que a sonda contorna, com o motivo ao lado.

### §58.2 — ⛔⛔ RECUSA MEDIDA: os níveis de exportação NÃO podem mandar na densidade dos quads

O Enio decidiu que os três botões deviam escolher a densidade da retopologia. **Implementei a escada
completa** (`chain_alpha` × `feed_depth`, com a razão `célula/alvo` invariante por construção) e
medi-a pelo caminho do produto (`measure_the_export_wall_clock`, release, esfera):

| nível | aresta pedida | grade | espera | veredito |
|---|---:|---:|---:|---|
| Draft | 2 % | 6 | **4 717 ms** | ✅ adoptada — 2 539 quads, `6,42°`, `0` bordo |
| Fine | 1 % | 7 | **49 691 ms** | ⛔ **Rejected { boundary: 42 }** |
| Max | 0,5 % | 8 | ⛔ **1 648 579 ms** (27 min 29 s) | ⛔ **Rejected { boundary: 316, non_manifold: 6 }** |

⛔ **REVERTIDO.** O limite da cadeia **não é o tempo — é a TOPOLOGIA da extracção**: ela fecha a peça
na densidade grossa e **rasga** na fina, e o veto (correctamente) deita fora o trabalho. Deixar a
escada no produto seria um botão que gasta meia hora para devolver o que o botão anterior já dava.

⚠️ **E o custo cresce ~33× por degrau de 2×** — `4,7 s → 50 s → 1 649 s` —, o que é super-linear de
uma forma que nem a bancada em segundo plano resolve.

⇒ **A decisão do Enio está certa como produto e bloqueada pelo motor.** O achado é para quem possui
a extracção (`line/quadextract`, a mesma dos dois `panic!` já nomeados): *a densidade fina tem de
FECHAR a peça antes de o botão fazer sentido*. Quando fechar, esta tabela é o que se reconfere.

## §71 — W70: três em cada quatro fitas do quadro não eram avaliadas por ninguém (26/08)

**De onde vem.** A W69 curou o quadro de MOVIMENTO tirando-lhe o eixo do teto de detalhe, e nomeou o
que sobrava: *«mesmo agora o movimento custa ~53 ms contra um orçamento de 16,7 — a pré-visualização
nunca alcançou 60 Hz numa peça de perfil, em nível NENHUM»*. A W68 já tinha medido o **mecanismo**:
o traçado paga `0,22 ms` **por aresta do contorno**, e esse custo é **cego aos pixels** (4× menos
pixels ⇒ `1,3×` menos tempo) — assinatura de **montar**, não de marchar.

⚠️ **A W68 acusou a especialização por ladrilho e estava errada.** Sem ela o traçado a `640×360` vai
de `58` para `565 ms` (10×) e o `TILE = 64` já está no óptimo. A pergunta certa não era **quantas**
fitas se montam — era **o que cada montagem monta**.

### §71.1 — A soma partida nas partes de que é feita

`ph2d_field_eval::tests::measure_the_four_tapes_of_one_specialisation`, máquina a `load < 5`:

| arestas | árvore (`compile_at`) | fita **float** | fita **grad** | `from_tree` hoje | `fork` hoje |
|---:|---:|---:|---:|---:|---:|
| 168 | `0,107` | `1,309` | `1,427` | `1,311` | `1,304` |
| 336 | `0,211` | `2,248` | `2,441` | `2,245` | `2,250` |
| 672 | `0,415` | `4,508` | `4,883` | `4,510` | `4,589` |

**Antes da wave, `from_tree` era `float + grad`** — e a marcha chamava `fork()` logo a seguir, que
recompilava o par outra vez. ⇒ **quatro compilações por região especializada, e o traçado avalia
UMA.** A árvore, que é a parte que *pensa*, custa `4 %` da conta; o resto é a `fidget` a emitir
código de máquina.

⭐⭐⭐ **E o consumidor da fita de gradiente é UM, e não é o traçado:** `Hybrid::gradients` tem um
único chamador em toda a árvore — a extração de malha (`extract.rs`), na **exportação**. A normal do
traçado sai de **seis amostras na fita float** (`march::normals_into`, diferença central).
*Uma fita que ninguém avalia não aparece em gate nenhum: a imagem é byte-idêntica, só o relógio
muda.*

### §71.2 — Duas ausências (a cura não acrescenta nada — retira)

**(a) A fita de gradiente nasce vazia e monta-se no primeiro pedido.** ⚠️ A subtileza é que
`grad.is_none()` **deixou de significar** *«este documento não tem gradiente exacto»* e passou a
significar *«ainda não foi pedido, ou não há»* — a pergunta antiga vive agora em
`Hybrid::grad_is_exact`, e a distinção está **dentro de `gradients`**, o único sítio onde ela
importa. A extração continua com o gradiente **exacto**, e paga a fita uma vez, como pagava.

**(b) Quem monta a própria fita não a forka.** O `shape_of(k)` devolve um `Hybrid` acabado de
construir **para aquela fatia**, que mais ninguém vê. *Um `fork` é para partilhar o que é de outro;
o que já é nosso avalia-se directamente.* ⚠️ O caminho **não** especializado continua a forkar, e
tem de continuar: ali o `scene.shape` é partilhado pelas threads do lote (a condição do ADR-0109).

⚠️ **Uma terceira ausência foi construída e REVERTIDA por medição** — a segunda passagem. Ver
§71.4: ela é o sítio onde a intuição aponta e a medição não confirma.

### §71.3 — O que (a) e (b) compram

A/B **intercalado** (`A,B,A,B,A,B`, mediana de 3 rondas × mediana de 7 traçados), quadro do produto
a `640×360` **com** anti-serrilhado, `load < 5`:

| arestas | antes | depois | ganho | fitas float | fitas de gradiente |
|---:|---:|---:|---:|---:|---:|
| 168 | `69,0 ms` | **`41,8 ms`** | **`1,65×`** | `293 → 161` | `293 → 0` |
| 672 | `274,8 ms` | **`142,9 ms`** | **`1,92×`** | `293 → 161` | `293 → 0` |

⚠️ **O A/B faz-se trocando o CÓDIGO, não um interruptor** — as duas leis são ausências, e um
interruptor para as ligar de volta seria produto a carregar a versão lenta para sempre. Quem alterna
é o arnês de mutação, e a alternância é intercalada para a deriva da máquina não se colar a um lado.

### §71.4 — ⛔ A segunda passagem: duas curas construídas, medidas e REVERTIDAS

O anti-serrilhado marcha uma vez por lote de `EDGE_CHUNK = 64` pixels de borda, e cada marcha
**forkava** a árvore partilhada ⇒ uma fita por lote.

⛔ **A cura óbvia — `map_init` da rayon — é NEUTRA, e foi medida:** `0,99×` e `1,02×`, com a contagem
de fitas a não se mexer (`161 → 161`). ⭐ **O mecanismo:** a rayon parte um `par_chunks` até **uma
tarefa por lote**, e o `init` corre uma vez por *tarefa* — ou seja, uma vez por lote, que é
exactamente o que se queria evitar. *Reaproveitar por tarefa não compra nada enquanto a tarefa for
do tamanho do lote.*

⇒ segunda tentativa: a tarefa passa a ser `pixels / threads` (e o `EDGE_CHUNK` volta a ser só o
tamanho do lote que a marcha avalia de uma vez). Ela **funciona** — corta as fitas de `1000` para
`950` a `1920×1080` — e **também não move o relógio**:

| tamanho | arestas | com fork por lote | com avaliador por tarefa | ganho | fitas |
|---|---:|---:|---:|---:|---:|
| `640×360` | 168 | `42,4 ms` | `43,5 ms` | `0,97×` | `161 → 161` |
| `640×360` | 672 | `151,9 ms` | `153,7 ms` | `0,99×` | `161 → 161` |
| `1920×1080` | 168 | `170,3 ms` | `172,9 ms` | `0,98×` | `1000 → 950` |
| `1920×1080` | 672 | `554,0 ms` | `547,6 ms` | `1,01×` | `1000 → 950` |

⭐ **E a tabela explica-se sozinha:** a `1920×1080` o quadro especializa **917** regiões. As dezenas
de fitas da passagem de borda são **ruído ao lado disso** — *o alvo estava certo (a montagem) e o
sítio estava errado*. ⇒ **REVERTIDO**, com as duas recusas escritas no doc do `EDGE_CHUNK`, que é
onde a próxima pessoa a ter esta ideia vai olhar. E o gate saiu com o código: *um gate que defende
código revertido é um gate a defender nada.*

### §71.5 — Os gates, e o teto que deixava passar o defeito

- `a_frame_pays_one_tape_per_region_and_no_gradient_tape_at_all` — o quadro compila **uma** fita por
  região (mais a base, mais no máximo uma por ladrilho para a rota não especializada) e **zero**
  fitas de gradiente; e a segunda passagem inteira cabe num avaliador só;
- `the_gradient_tape_is_still_there_for_whoever_asks_and_is_built_once` — a outra metade: adiar só é
  correcto se ela ainda acontecer quando alguém a pede, **uma vez**.

⛔ **Eles vivem em dois binários de teste, não em `src/tests.rs`.** Os contadores (`FLOAT_TAPES`,
`GRAD_TAPES`) são do **processo** e o `cargo test` corre a suíte em paralelo: o primeiro gate exige
**zero** fitas de gradiente enquanto o segundo constrói uma. ⚠️ Um cadeado **não** resolveria —
teria de ser tomado por *todos* os testes que traçam, incluindo os que ainda não existem. *Um
contador global só é legível onde ninguém mais escreve nele.*

⛔⛔ **E o primeiro teto da segunda passagem PASSAVA COM A MUTAÇÃO POSTA.** Ele dizia
`fitas ≤ regiões + ladrilhos + 2`, e a folga dos ladrilhos (**60**) era maior que os lotes de borda
(**28**) — a rota não especializada existe, mas naquela fixtura nunca dispara. *A folga que se põe
«por segurança» é exactamente o tamanho do defeito que o gate deixa de ver.* A cura é medir a
**DIFERENÇA** entre dois traçados (com e sem anti-serrilhado), e ambos **seriais**: em paralelo quem
decide quantos avaliadores nascem é o escalonador da rayon — *um gate sobre uma contagem só é um
gate se a contagem for do produto.*

⛔ **O balde tem de estar cheio:** o gate afirma `regiões > 0`, que a peça foi desenhada, e que há
mais de quatro lotes de borda, **antes** de aplicar qualquer teto.

**Provas de mutação — 4/4 mataram** (controlo verde antes do vermelho):

| # | mutação | gate que a apanhou |
|---|---|---|
| M1 | a fita de gradiente volta a nascer com a float | `a_frame_pays_one_tape_per_region…` |
| M2 | a marcha volta a forkar a fita que montou | `a_frame_pays_one_tape_per_region…` |
| M3 | a fita de gradiente nunca se monta | `the_gradient_tape_is_still_there…` |
| M4 | ela monta-se em **cada** pedido | `the_gradient_tape_is_still_there…` |

### §71.6 — O que fica, e o que isto NÃO é

⛔ **Isto não reabre a recusa da W64** (§69). Lá o que foi medido e rejeitado foi **especializar** a
segunda passagem por ladrilho — construir uma árvore *própria* para a região da borda, que não se
amortiza (`4 096` raios por ladrilho na primária contra `~256` na de borda). Aqui não se constrói
árvore nenhuma: pergunta-se **quantas vezes se compila a fita que já existe**. *São duas perguntas
diferentes sobre a mesma passagem.*

⛔ **E a base continua acima do orçamento.** O quadro de movimento a `640×360` custa hoje
**`41,8 ms`** contra os `16,7` de um quadro de 60 Hz — `2,5×` acima, contra os `4,1×` de ontem. *A
pré-visualização continua a não alcançar 60 Hz numa peça de perfil.*

⚠️ **E as duas direcções que esta secção nomeava — reaproveitar a fita entre quadros de uma órbita e
especializar em espaço LOCAL — têm agora um TECTO MEDIDO, no §72: `20 %`.** Elas atacavam a
montagem, e a montagem deixou de ser a maioria do quadro quando esta wave a cortou ao meio. *Uma
direcção nomeada antes de a fracção ser medida é uma aposta com cara de plano.*

## §72 — W71: a montagem é `20 %` do quadro, e a fatia mudou de número (26/08)

Duas perguntas que a W70 deixou abertas, medidas juntas porque partilham a fixtura.

### §72.1 — ⛔ A fracção de montagem: `20 %`, e ela fecha duas direcções

O A/B da W70 admitia **duas divisões** da mesma medição: ela removeu `132` fitas float **e** `293`
de gradiente e ganhou `27,2 ms`. Dividir por `132` diz que a montagem que sobra é `79 %` do quadro;
dividir por `425` diz `25 %`. *Duas divisões da mesma medição não são uma medição* — e elas mandam
em waves opostas.

⇒ o produto passou a **contar o tempo** (`SPECIALISE_NS`, ao lado do `SPECIALISED`), lido num
traçado **serial** — a soma é de CPU, e só contra um relógio de parede serial é que ela é uma
fracção:

| arestas | quadro (serial, com AA) | montagem | fracção | regiões |
|---:|---:|---:|---:|---:|
| 168 | `436,7 ms` | `87,9 ms` | **`20,1 %`** | 132 |
| 672 | `1 737,7 ms` | `335,8 ms` | **`19,3 %`** | 132 |

⭐⭐ **A leitura de `79 %` está refutada, e com ela o plano que ela sustentava.** As duas direcções
que o §71.6 nomeava — *reaproveitar a fita entre quadros de uma órbita* e *especializar em espaço
LOCAL* — atacam a montagem, logo **nenhuma delas pode comprar mais do que `20 %`**, e as duas são
obras grandes (a região é a caixa do frustum do ladrilho: ela move-se com a câmera, e o casco que a
poda são oito pontos em 3D — uma chave de cache que quase nunca acerta). ⇒ **o alvo passa a ser a
MARCHA**, que são os outros `80 %`.

⚠️ *Uma direcção nomeada antes de a fracção ser medida é uma aposta com cara de plano* — e esta
custou uma medição a desfazer.

### §72.2 — ⭐ A fatia: `2 → 4`, porque o preço de montar caiu para metade

Repartir o tubo de um ladrilho em fatias de profundidade **divide** o custo de avaliar (cada região
guarda menos arestas) e **multiplica** o de montar. A W56e mediu `2` como óptimo — com o preço de
montar que havia então. A W70 cortou esse preço ao meio ⇒ *o vale tinha de se mover, e moveu*.

Varredura **intercalada** (`N = 2,3,4,6` × 3 rondas × mediana de 5, tile `64`, `load < 5`):

| tamanho | arestas | N=2 | N=3 | **N=4** | N=6 |
|---|---:|---:|---:|---:|---:|
| `640×360` | 168 | `37,6` | `35,0` | **`34,6`** | `35,0` |
| `640×360` | 672 | `131,2` | `126,6` | `116,1` | **`114,9`** |
| `1920×1080` | 168 | `157,9` | `149,0` | `146,7` | **`143,5`** |
| `1920×1080` | 672 | `504,1` | `447,7` | `424,8` | **`415,9`** |

⭐ **`4` ship porque o caso que o artista SENTE é o primeiro** — o quadro de movimento a `640×360`
na resolução de omissão, onde `4` ganha e `6` já volta a subir. Nos outros três `6` é melhor por
`1 %`–`2 %`, que é a largura do vale. Ganho: **`1,09×`** no caso do preview e **`1,19×`** no mais
pesado.

⚠️ **A tabela antiga fica registada porque ela não estava errada** — ela media outro preço. *Uma
constante que se move é uma medição a acontecer; o que não pode mover-se em silêncio é a razão.*

⚠️ E o **`TILE` não se moveu**: a varredura por ladrilho com `N = 4` dá `32 → 48,8` · `48 → 37,8` ·
**`64 → 33,8`** · `96 → 39,9` · `128 → 55,6`. ⭐ E ela mostra o mecanismo de lado: **quanto maior o
ladrilho, mais fatias ele quer** (a `96` e `128` o ganho de `N=1` para `N=4` é `1,6×`), que é
exactamente o que a fórmula da região prevê — ela mede `lado + profundidade × |direcção|`.

### §72.3 — ⚠️ E um vermelho que NÃO era desta wave

`an_abandoned_march_returns_nothing_and_returns_fast` reprovou na corrida da suíte e passou **3 de
3** sozinho. Ele é uma **razão entre dois relógios** (a família que o `CLAUDE.md` §5.0 descreve), a
máquina estava a `load 16` por causa da própria varredura, e ⭐ **a fixtura dele é uma esfera — que
não tem perfil, logo nem sequer entra no caminho especializado que esta wave tocou**. *Antes de
olhar para o commit, corra-o sozinho.*

## §73 — W72: o quadro de movimento também não paga o anti-serrilhado (26/08)

A §72.1 mandou o alvo para a **marcha** (`80 %` do quadro). Antes de a atacar, a forma dela:

| | medido |
|---|---|
| amostras de campo por pixel | **`8,7`** |
| custo por amostra, 168 arestas | `147,5 ns` |
| custo por amostra, 672 arestas | `558,0 ns` |

⭐⭐ **A marcha já está apertada** — `8,7` passos por pixel não deixam nada para a *sobre-relaxação*
da **Enhanced Sphere Tracing** ir buscar (ela ataca a **contagem de passos**, e a contagem já é
quase o mínimo de uma esfera-marcha com normal). ⇒ **o custo é por ARESTA TOCADA**: quadruplicar o
contorno multiplica a amostra por `3,8×`. E isso confirma, por outro caminho, o que a W56 já tinha
medido — `0,95 ns` por ponto por aresta, oito faixas de SIMD com JIT, *quase óptima por aresta; o
que se ganha é tocar menos arestas*.

### §73.1 — Os dois botões, medidos lado a lado

`measure_the_two_knobs_of_the_moving_frame`, `640×360`, mediana de 3 rondas × 5, `load < 5`:

| arestas | 48 | 64 | 96 | 128 | 168 |
|---|---:|---:|---:|---:|---:|
| **com** anti-serrilhado | `14,6` | `17,5` | `22,3` | `28,8` | `35,7` |
| **sem** | `10,9` | `12,5` | `17,1` | `20,6` | `26,7` |
| o que ele custa | `1,33×` | `1,40×` | `1,30×` | `1,40×` | `1,34×` |

⭐ O quadro é **linear nas arestas** e o anti-serrilhado é um factor **constante** — dois botões
independentes, e a tabela dá o preço de cada um.

### §73.2 — ⭐ Liga-se o segundo, e não o primeiro

**O anti-serrilhado sai enquanto a mão mexe** (`field3d_preview::wants_antialias`) — `35,7 → 26,7 ms`
no caso do preview, **`1,34×`**, sem uma linha de geometria mudar.

⚠️ **Ele é o melhor dos dois para se cortar a mexer, e a razão é o que cada um estraga:** tirar
arestas muda a **FORMA** (uma curva fica facetada), tirar o anti-serrilhado muda a **borda de um
pixel** — numa imagem que está a mover-se e que volta inteira mal a mão pare. ⏸️ O botão das arestas
fica com o preço na tabela acima, para quem vir a peça a mexer decidir.

⚠️ **A pergunta é o TAMANHO, não «a mão está a mexer»** — o laço já responde a isso: um traçado do
tamanho **cheio** *é* o refinamento que corre depois de a cena assentar. Uma segunda fonte para o
mesmo facto divergiria no caso da peça barata, cujo preview já pede o cheio **enquanto** a mão mexe
— e ali a resposta certa é *«ligado»*, que é o que a pergunta pelo tamanho dá de graça.

⚠️ **A costura não é alcançável de um teste** (ela vive dentro do `spawn` do traçador, que precisa de
uma janela): o que a defende é o **compilador** — o `antialias` entrou na assinatura da
`trace_cancellable`, então nenhum chamador pode deixar de responder à pergunta. A lei em si tem gate
(`the_moving_frame_does_not_pay_for_the_antialias`) e **duas** provas de mutação (nunca ligado ·
sempre ligado).

⛔ **E o que isto NÃO é:** a segunda passagem continua a correr no quadro assente, que é o que fica
na tela. *Cortar nos dois seria trocar a razão de existir do módulo — a quina afiada — por
milissegundos.*

## §74 — W73: «ao parar ficou mais lento para alisar» — o assentar vira uma ESCADA (26/08)

**Report do Enio, sobre a W72:** *«ao parar ficou mais lento para alisar (500 ms)»*.

### §74.1 — ⛔ O mecanismo é meu, e ele não é «mais lento»: é **noutro sítio**

O traçado assente **não** mudou com a W72 — ele sempre correu no tamanho cheio e sempre custou o que
custa (`504 ms` a `1920×1080` com 672 arestas). O que mudou foi **onde o alisamento vive**: até à
W72 o quadro de movimento já vinha com anti-serrilhado, e ao parar só faltavam *pixels*; depois
dela, o que falta ao parar é **o alisamento inteiro** — e ele só chega no fim do degrau mais caro
que existe.

⚠️ *Uma cura pode mudar o SÍTIO de uma espera em vez de a cortar* — e o sítio novo era justamente
aquele em que o artista está a olhar para a peça à espera de a ver lisa.

### §74.2 — ⭐ A cura: dois degraus, e o barato primeiro

O assentar deixa de ser um salto e passa a ser uma **escada**:

| degrau | o que corre | custo medido (672 arestas) | o que o artista vê |
|---|---|---|---|
| movimento | tamanho do preview · contorno grosso · **sem** AA | `26,7 ms` | responde à mão |
| **1.º ao parar** | **mesmo tamanho** · contorno inteiro · **com** AA | **`131 ms`** | ⭐ a peça **lisa** e com a forma certa |
| 2.º | tamanho cheio · contorno inteiro · com AA | `504 ms` | os pixels nítidos |

⭐ **O que ele espera chega `3,8×` mais cedo**, e o degrau caro deixa de estar no caminho dele.

⚠️ **O preço está nomeado:** o total sobe `~26 %` (o degrau do meio é trabalho a mais). É a troca
clássica do refinamento progressivo — **latência percebida contra trabalho total** — e aqui ela é
decidida pelo que a mão faz a seguir: se ela voltar a mexer, o degrau caro **nunca chega a correr**
(`cancels_the_inflight`), e o trabalho a mais foi zero.

### §74.3 — ⚠️ Uma bandeira, dois cortes — e a `wants_antialias` MORREU com um dia

A W72 tinha perguntado *«o tamanho pedido é o cheio?»* para decidir o anti-serrilhado. ⛔ **Essa
pergunta não sabe exprimir a escada:** o degrau que alisa pede **o mesmo tamanho** do de movimento, e
os dois seriam indistinguíveis. ⇒ a resposta passa a **viajar com o pedido**: `next_trace` devolve
`(largura, altura, é_de_movimento)`, e o contorno grosso e o anti-serrilhado desligado lêem **a
mesma** bandeira.

⭐ *Duas perguntas para o mesmo facto é uma delas a envelhecer* — e esta envelheceu em **24 horas**,
o que é o argumento mais curto que este módulo já teve para a lei da fonte única.

⚠️ **A memória entra no estado** (`Smoke::requested` ganha o `bool`): sem ela o degrau que alisa e o
que aumenta são indistinguíveis, porque os dois pedem o mesmo `(câmera, tamanho, documento)`.

**Provas de mutação — 3/3 mataram:** saltar o degrau que alisa · o traçado de movimento sair fino ·
o degrau do meio continuar grosso.

## §75 — W74: com duas formas escolhidas, a segunda desaparecia em silêncio (26/08)

O item *«vários `VecPath` separados numa peça só»* estava na lista de aberto como uma **feature que
falta**. Ao olhar para o código, ele era outra coisa: **um defeito mudo, em série**.

### §75.1 — ⛔ Duas perdas silenciosas, uma atrás da outra

1. `from_selection` cozia `closed.first()` e **ignorava o resto** sem uma palavra;
2. e mesmo que cozesse todas, a caixa de correio (`PENDING_PROFILE`) era um **slot** `Option<_>` —
   a segunda escrita **apagava** a primeira.

⇒ o artista escolhia duas formas, carregava em `+ Extrude`, e via **uma** peça e nenhuma explicação.
⚠️ *Um slot com um escritor é um slot; com dois, é uma perda silenciosa* — e aqui havia duas em
série, o que faz a segunda ser invisível mesmo depois de a primeira ser curada.

### §75.2 — ⭐ Uma peça por FORMA, e não uma peça com todas

A nota pedia *«uma peça só»*. ⛔ **A medição escolheu o contrário, e a razão é o VÍNCULO VIVO:** o
`FieldProfileSource` aponta para **um** desenho (W55), então uma peça de `N` contornos ou perdia o
vínculo de `N−1` deles ou obrigava o componente — que **viaja no arquivo** — a mudar de forma, com
migração de `PROJECT_SCHEMA` atrás.

⭐ Com uma peça por forma, **todas** continuam a seguir o desenho delas, e juntá-las numa só é a
**booleana que o módulo já tem**. *A composição já exprimia «uma peça»; o que ela não exprimia era
«o resto existe».* (A lei do §5.0 do `CLAUDE.md`: *antes de construir um item de lista aberta, meça
se a composição já o exprime* — aqui ela exprimia, e o que faltava era outra coisa.)

### §75.3 — E o que fica de fora é DITO

A mensagem passa a **contar**: `Extruded 2 shapes (824 edges)`. ⚠️ **O singular fica com o texto de
sempre** — ele é o caso normal, e trocá-lo por *«1 shape»* tornaria a frase de toda a gente mais
fria para servir a excepção. E o que o documento recusa entra na frase (`. One was skipped: …`), em
vez de sumir: ⭐ **a peça boa nasce na mesma** — uma recusa que abortasse o lote faria um contorno
mal desenhado apagar o trabalho dos outros.

**Provas de mutação — 3/3 mataram:** só a primeira forma vira peça · a caixa de correio volta a ser
um slot · a recusa deixa de ser dita.

⚠️ **O gate tem três metades**, e a do meio é a que apanha o laço mal escrito: que nascem `N`, que
**cada uma aponta para um desenho diferente**, e que a mensagem conta. Duas peças a apontar para o
mesmo contorno passariam nas outras duas.

## §76 — W75: a cerca do passo da marcha estava ERRADA, e a cena 1 do smoke marchava acima do seguro (26/08)

O `safe_march_step` carregava, escrita no próprio doc, uma cerca por medir: *«⛔ Não se compõe um
limite por nó: encadear misturas pode compor os factores, e essa pergunta **não foi medida**»*.

⚠️ **A consequência de ela estar errada não é lentidão — é a peça FURAR.** A marcha anda `d · s` e só
é segura enquanto `s · ‖∇f‖ ≤ 1`; acima disso o passo é maior que a distância até à superfície, o
raio atravessa-a, e o sintoma é pixel de fundo no meio da peça.

### §76.1 — ⛔ Eles compõem, e a tabela é esta

`the_table_of_the_gradient_of_a_composition` (grelha de `40³` sobre `[-1, 1]³`):

| composição | `‖∇f‖` | `passo × ‖∇f‖` com o `1/√2` de ontem |
|---|---:|---:|
| `Union Exact 0,05` × 2 encadeados | `1,4142` | `1,00` |
| `Union Exact 0,2` × 2 | `1,5076` | **`1,07`** ⛔ |
| `Union Exact 0,5` × 2 | `1,6873` | **`1,19`** ⛔ |
| `Union Exact 0,2` × 3 | `1,7778` | **`1,26`** ⛔ |
| `Union Exact 0,5` × 3 | `1,9588` | **`1,39`** ⛔ |
| `Difference Exact` sobre `Union Exact` | `1,4142` | `1,00` |
| qualquer **modificador** sobre `Union Exact` | `1,4142` (o `Taper` **desce** a `0,8333`) | `1,00` |

⭐ **O que compõe é o exacto que recebe um campo JÁ INFLADO no ramo que ele arredonda** — a
`Difference` lê o segundo operando pelo lado de fora, e um modificador lê o campo sem o voltar a
arredondar. ⇒ o expoente conta **níveis encadeados**, não nós inflantes soltos.

### §76.2 — ⭐⭐⭐ E o nó de N FILHOS é uma corrente disfarçada — a cena 1 do smoke

O `combine_trees` dobra os filhos **da esquerda para a direita**: um `Union Exact` com `n` filhos são
**`n − 1`** arredondamentos encadeados **dentro de um nó só**.

| um nó, `Exact 0,2` | `‖∇f‖` |
|---|---:|
| 3 filhos | `1,5411` ⛔ |
| 4 filhos | `1,7321` ⛔ |
| 5 filhos | `1,9585` ⛔ |

⛔⛔ **É exactamente a forma da cena 1 do smoke** — três cilindros numa união exacta de raio `0,12` —,
o que quer dizer que a cena que o Enio abre desde o primeiro dia marchava **acima do passo seguro**.
⚠️ *Uma fixtura de dois filhos não vê a corrente que o lowering constrói*, e foi por isso que a
primeira redacção desta wave — que contava **um** nível por nó — ainda estava errada.

### §76.3 — A lei nova, e por que a barra é a PROVÁVEL e não a medida

`passo = 1/√2^k`, com `k` = o maior número de níveis inflantes num caminho raiz→folha
(`ph2d_field_eval::inflation_depth`), e um nó de `n` filhos a contar `n − 1`.

⚠️ **A barra é `√2` por nível porque isso se PROVA** (o arredondamento exacto de dois campos
`L`-Lipschitz é `√2·L`), e as medições ficam **abaixo** dela (`1,96` contra `2,83` a `k = 3`). *Um
teto de segurança prova-se, não se ajusta a um corpus* — apertá-lo até à medição seria transformar
«as peças que eu testei» em «as peças que existem».

⚠️ **Uma escultura conta um nível e continua sem medição própria** (o campo dela é interpolado de uma
grelha). Ela já era classificada como inflante; nada mudou para ela.

### §76.4 — O preço, medido

`measure_what_the_safe_step_costs` (mesmo processo, `640×360`, mediana de 3 × 5):

| níveis | passo seguro | com ele | com o `1/√2` inseguro | a segurança custa |
|---:|---:|---:|---:|---:|
| 1 | `0,7071` | `6,3 ms` | `6,2 ms` | `1,01×` |
| 2 | `0,5000` | `7,1 ms` | `6,7 ms` | `1,06×` |
| 3 | `0,3536` | `9,2 ms` | `7,5 ms` | `1,23×` |

⭐ **`6 %` na forma da cena 1.** Muito menos do que a razão dos passos (`1,41×`) faria esperar — a
marcha está presa à caixa da peça, e a maioria dos raios acerta cedo.

### §76.5 — E as duas lições de método

⚠️ **A cerca vivia num doc-comment e nenhum gate a atravessava.** O
`the_step_times_the_worst_gradient_never_exceeds_one` varria construtores **soltos** e por isso
passava havia meses. ⇒ as composições entraram **dentro** dele (`composition_cases`, partilhada com
a sonda), e não numa tabela à parte: *uma cerca que nenhum gate atravessa é uma nota, não uma cerca.*

⚠️ **E uma mutação leu-se como SOBREVIVENTE por um filtro mal escrito** (`-- --exact A --exact B`
corre só um teste). Contra a suíte inteira ela morre em **dois** gates. *O filtro é onde a resposta
se perde* — a mesma lição que o `CLAUDE.md` §2 mede em 797 corridas que devolveram nada.

**Provas de mutação — 3/3 mataram:** a lei velha (qualquer exacto ⇒ `1/√2`) · a profundidade a
ignorar o que está por baixo · um nó de N filhos a contar `1`.

## §77 — W76: a escultura que perdeu o arquivo pode ser RELIGADA (26/08)

O aviso existia desde a W23 — reabrir um projeto cuja malha mudou de sítio diz *«Sculpture bunny.obj
is missing»* e a peça abre sem ela. ⛔ **E era um beco:** a única cura era pôr o arquivo de volta no
caminho **exacto**. *Um aviso que nomeia o problema e não oferece o gesto manda o artista consertar o
disco.*

### §77.1 — O verbo, e a quem ele aparece

`Relink Sculpture…` entra na fileira de acções — a mesma que já carrega `Unlink` e `Link Drawing`, e
pela mesma lei (uma fileira que não é fixa, com o slot a resolver-se em **chave** e não em número).

⚠️ **Ele aparece só a quem PERDEU o arquivo**, e a pergunta é feita ao **registo** — a mesma resposta
parcial que o `field3d_reload::missing_keys` lê. Oferecê-lo a uma escultura que está lá seria um
verbo sem o que consertar.

⚠️ **E uma chave `scene:` fica de fora**: ela nomeia a escultura **viva** da cena, que não veio de
arquivo nenhum — pedir um `.obj` para a substituir seria mandar o artista procurar o que nunca
existiu. Quem a repõe é o `+ Sculpt from scene`, e o `resolve_missing` já lho pede sozinho.

### §77.2 — ⭐ A chave NOVA é o caminho novo, e é isso que faz a cura durar

Registar o arquivo novo sob a chave **velha** faria a peça abrir hoje e falhar outra vez amanhã: a
chave é o que o `ProjectFile` guarda e o que a próxima abertura vai ler. ⇒ ela é **reescrita no nó**,
o que também a torna um passo de **undo** — como tem de ser: é uma decisão do artista sobre o
documento.

⚠️ **A escala fica como está**, ao contrário da importação: a peça já tem a pose que o artista lhe
deu, e um arquivo novo que a re-enquadrasse desfazia esse trabalho. *Religar troca a fonte, não a
colocação.*

### §77.3 — Três saltos, e o do meio não é alcançável

*o verbo pede* (a fileira, com o mundo emprestado) → *o app escolhe o arquivo* (um diálogo nativo) →
*a ponte com a cena escreve a chave* (quem tem o `&mut World`). É a forma que a importação já tinha,
com um salto a mais porque o alvo é um nó que já existe.

⚠️ **O salto do meio não tem gate** — um diálogo nativo precisa de um app. O que os gates provam são
os dois que sobram: que o verbo **pede com a entidade certa** e que a resposta **reescreve a chave**.

**Provas de mutação — 3/3 mataram:** religar nunca é oferecido · a escultura viva da cena também o
oferece · a chave nova não é escrita no nó.

### §77.4 — E o teto de LOC cortou o painel por ASSUNTO

`field3d_scene_panel.rs` passou os `600` do HR-18. O corte é por responsabilidade, e a fronteira já
existia por dentro: o irmão monta o **retrato** que o painel lê; o
[`field3d_scene_acts.rs`](../../shells/desktop/src/field3d_scene_acts.rs) responde às duas perguntas
que dependem de **quem está escolhido** — *que verbos este objecto oferece?* e *que selo ele veste?*.

⚠️ **E o armazenamento dos selos ganhou uma PORTA de escrita** (`publish_badges`): o produtor ficou
no irmão e o estado mudou de casa, e um `thread_local` visível de fora seria a fronteira a não dizer
nada. *Quem escreve chama uma função; o estado não sai de casa.*

## §78 — W77: a segunda cerca do passo — e a nota dizia «ninguém mediu» com um gate a medir (26/08)

Depois da W75, o `inflation_depth` tinha uma segunda afirmação por confirmar: *«uma escultura conta
como um nível e continua sem medição própria — o campo dela é interpolado de uma grelha, e ninguém
mediu o gradiente da interpolação»*.

### §78.1 — ⛔ A nota estava errada sobre si mesma

**Um gate media desde sempre:** `the_sampled_field_marches_like_a_distance` mede `‖∇f‖` de um campo
amostrado — **numa esfera, numa banda de três células fora da casca**, contra um alvo com folga de
`0,2`. ⚠️ *Uma nota que diz «não medido» quando existe um gate estreito é pior que nenhuma: ela manda
medir de novo e esconde o que já se sabe.* **É a quinta nota deste módulo a envelhecer contra o
código** (as outras quatro estão auditadas no §13.0).

### §78.2 — O que faltava era a GENERALIZAÇÃO

| forma | `res` | `‖∇f‖` máx | contra `√2` |
|---|---:|---:|---:|
| esfera | 128 | `1,0000` | `0,71×` |
| **cubo** | 128 | **`1,0852`** | `0,77×` |
| octaedro | 128 | `0,8778` | `0,62×` |

⭐ **O cubo é o pior, e tinha de ser:** a interpolação trilinear só pode subir mais depressa que a
distância onde o campo tem **vinco**, e uma esfera não tem nenhum. O gate irmão media exactamente a
forma que não podia falhar.

⇒ o gate novo (`a_sculptures_field_never_out_climbs_the_march_step`) acrescenta as três coisas que
faltavam: **formas com vinco**, a **caixa inteira** em vez de uma banda, e a barra que de facto
importa — o `√2` da marcha, e não um alvo com tolerância.

### §78.3 — ⚠️ E a barra DEMONSTRÁVEL é `√3`, não `√2`

Cada componente do gradiente de uma interpolação trilinear é um quociente de diferenças ≤ `1`, e três
delas somam em quadratura ⇒ **`√3 ≈ 1,732`**, que está **acima** do `√2` que o nível concede. As
medições ficam em `1,09` porque saturar as três ao mesmo tempo exigiria a superfície perpendicular
aos três eixos **no mesmo ponto**, e uma distância com sinal não faz isso.

⚠️ **A folga medida é de `30 %` e a diferença para a barra demonstrável fica ESCRITA** — é o
contrário da W75, onde a barra provável era mais apertada que o corpus e por isso ship. *Quando a
prova é mais frouxa que a medição, o que se ship é a medição, e a distância entre as duas é uma
dívida nomeada, não um esquecimento.*

**Prova de mutação:** um campo `2×` mais inclinado **longe** da superfície (que o oráculo de perto
não vê) fica vermelho aqui — e ⚠️ **a primeira mutação que tentei foi apanhada por outros dois
gates**, o que teria feito este parecer útil sem o ser. *Um gate só se prova com a mutação que só
ele mata.*

## §79 — W78: a auditoria da lista viva — duas entradas eram trabalho JÁ FEITO (26/08)

O §13.0 manda auditar a lista antes de pegar um item dela, e esta sessão deu-lhe razão **seis
vezes**. Esta wave audita as entradas que sobravam, **contra o código**.

### §79.1 — ✅ *Ladrilhar em `(u, v)` contra o paralelogramo* — **feito na W59**

A entrada pedia apertar a região do perfil trocando a caixa pelo paralelogramo projectado. ⭐ É
exactamente o que o `hull_uv` faz desde a W59 (§65): a região de um `Extrude` é o **casco dos oito
cantos do tubo** projectados em `(u, v)`, e mediu-se `1,21×` menos arestas. ⛔ O `Revolve` fica de
fora **por construção** — o `u` dele é `√(x² + z²)` e a região ali é um rectângulo.

⚠️ E apertar mais **não é livre**: a varredura da W71 (§72.2) mediu que a granularidade das regiões
já está no vale — regiões mais finas pagam mais montagem do que poupam em avaliação.

### §79.2 — ✅ *O `Mirror` não se consegue demonstrar* — **demonstra-se, e é uma linha**

A entrada vem da W17 e o Enio **adiou o item por causa dela**: *«ele dobra em torno do centro do
objecto, e o que falta é um alvo descentrado ou um pivô de espelho autorado»*.

⭐ **Um alvo descentrado é exprimível hoje**, e por duas portas que já existem: o modificador entra em
**qualquer nó menos uma escultura** (`mods_for`), e um nó de **operação** tem filhos com pose própria.
⇒ pôr o `Mirror` na **operação** dobra os filhos em torno do centro dela, e uma caixa fora do eixo
aparece dos dois lados.

Gate: `ph2d_field_eval::tests::a_mirror_on_an_operation_folds_an_off_centre_child` — e ele afirma as
duas metades (sem espelho o outro lado está **vazio**; com espelho tem o **mesmo campo**, não uma
cópia aproximada).

⚠️ *A nota afirmava uma ausência sem a medir, e custou o adiamento de uma feature que estava pronta.*

### §79.3 — ⏸️ *O laço que SUBTRAI* — a entrada está CERTA, e agora com o mecanismo

Medido no `field3d_input`: `additive = shift || super || control` — **os três modificadores são um
vocabulário só**, e de propósito (é o mesmo do canvas 2D; *um terceiro vocabulário no mesmo app é
onde a mão aprende errado*). ⇒ não há tecla livre para *subtrair*, e as saídas são quatro:

| saída | preço |
|---|---|
| `Alt`+arrasto | ⛔ o KDE rouba o `Alt` (registado no `CLAUDE.md` §5, módulo Timeline) |
| arrasto com o botão direito | ⛔ é o Orbit — o gesto principal da janela |
| um modificador **durante** o arrasto | ⚠️ muda o verbo a meio de um gesto já começado |
| um **chip de modo** no painel (*Add / Subtract*) | ⚠️ peso de UI para um gesto raro, mas **descobrível** e sem colisão |

*A decisão é de produto e continua a ser do Enio — o que esta wave acrescenta é que ela deixou de
precisar de investigação.*

### §79.4 — O resto da lista, conferido

| entrada | veredito |
|---|---|
| ⏳ a **marcha** é `80 %` do quadro | ✔ medido hoje (§72.1, §73) |
| ⏸️ `PREVIEW_MAX_EDGES` | ✔ preço na tabela do §73.1; decisão de quem vê |
| ⏸️ o 2.º degrau do assentar (`504 ms`) | ✔ medido hoje (§74.2) |
| ⏸️ `√3` demonstrável contra o `√2` medido | ✔ escrito hoje (§78.3) |
| ⛔ os dois `panic` do `ph2d-gridmap` | ✔ dono é a `line/quadextract` |

## §80 — W79: o espelho passa a ter TRÊS botões — e a cerca que dizia «roda o nó» era falsa (26/08)

**Enio, ao ver a demonstração da W78:** *«funciona para x. Melhor 3 botões para x, y e z»*.

### §80.1 — ⛔ A cerca estava escrita, e o argumento dela não se aplicava

O doc do `Unary::Mirror` dizia: *«sem escolha de eixo — quem quer outro **roda o nó**, e uma escolha
de eixo por modificador seria um terceiro vocabulário de orientação no mesmo módulo»*, por analogia
com o `Cylinder` (*«outro eixo se obtém pela rotação do nó»*).

⚠️ **A analogia falha, e falha no ponto que decide:** o modificador age no espaço **local**, *antes*
da pose do nó. Rodar o nó roda a **peça inteira**; para espelhar em Y por rotação seria preciso um nó
**intermédio** só para rodar, espelhar e desrodar. *Uma equivalência que exige uma terceira entidade
não é uma equivalência: é um contorno.*

⚠️ E os irmãos de eixo — `Array` (X) e `Radial` (Z) — **ficam como estão**: eles têm número, e uma
matriz por eixo é outra pergunta (três botões × dois números). O pedido era o espelho.

### §80.2 — ⭐ Variantes NOVAS, e no FIM da lista

`Unary::MirrorY` e `Unary::MirrorZ` são variantes **novas**, acrescentadas no **fim** — e as duas
coisas são a mesma razão: o documento serializa por **posição**, então um campo `axis` dentro do
`Mirror` (ou uma variante no meio) mudaria o significado dos bytes de **toda peça já gravada**.
*Append-only é o que faz uma extensão não ser uma migração.* ⇒ zero mexida no `PROJECT_SCHEMA`, e um
ficheiro de ontem abre igual.

Os rótulos passam a **`Mirror X` / `Mirror Y` / `Mirror Z`**, e os botões saem do
`UnaryKind::ALL` — o painel não mudou uma linha.

### §80.3 — ⛔⛔ E duas mutações SOBREVIVERAM, nos dois sítios que cortam ou furam a peça

Os primeiros gates apanhavam o **campo** (o espelho dobra no eixo certo) e deixavam passar:

| mutação | o que ela faz ao produto |
|---|---|
| a **caixa** do espelho usa sempre o eixo X | a caixa não alcança a cópia ⇒ a marcha recorta-a e o exportador **corta a peça** |
| os eixos novos dizem que **não** remapeiam coordenadas | a especialização constrói o perfil sob um domínio dobrado ⇒ **fura a peça** |

⇒ dois gates novos: `the_bounding_box_follows_the_axis_of_the_mirror` (com o **controlo sem espelho**,
porque a caixa sai de uma **bola** e uma bola cresce nos três eixos — *uma régua que o representante
não consegue exprimir mede a representação, não o produto*) e a censura, agora **derivada**.

### §80.4 — ⛔⛔ O gate da censura PROMETIA o que não fazia

Ele diz, no próprio doc: *«um `Unary` novo é **erro de compilação** aqui»*. **Não era.** A lista era
escrita à mão e a contagem no fim (`remaps.len() == 6`) só a defendia **de si mesma**: os dois
espelhos entraram no documento, o gate ficou **verde**, e a mutação que os classificava como *«não
remapeia»* sobreviveu.

⇒ a lista passa a ser **derivada** de `UnaryKind::ALL`, com a expectativa num `match` exaustivo — que
é onde o compilador de facto pára quem acrescenta um modificador. ⚠️ *Uma lista escrita à mão ao lado
de um enum é duas respostas, e uma contagem só guarda a que se escreveu.* (A mesma família do
importador que oferecia 4 extensões e roteava 11.)

**Provas de mutação — 4/4 mataram** (o `MirrorY` a espelhar em X · o `MirrorZ` a não dobrar · a caixa
a ignorar o eixo · os eixos novos a não remapear); e a quinta — apagar os dois botões do painel — é
**erro de compilação**, porque `ALL` tem tamanho fixo.

## §81 — W80: a caça às listas que se dizem exaustivas — e a segunda estava na lei mais cara do módulo (26/08)

A W79 encontrou um gate cujo doc prometia *«um `Unary` novo é erro de compilação aqui»* sobre uma
lista **escrita à mão**. O padrão tem nome e vale a pena varrer: *uma lista literal ao lado de um
`enum`, com uma contagem no fim que só a defende de si mesma.*

### §81.1 — ⛔ A segunda estava no gate que a W53 escreveu

`every_primitive_the_engine_can_make_has_a_button` promete, no doc: *«uma primitiva nova aparece aqui
**sozinha**, no dia em que nascer»*. ⛔ **Não aparecia** — o gate percorria uma lista literal, com o
comentário a admiti-lo (*«uma de cada, construída à mão: é a enumeração que o `Primitive` não
oferece»*), e a contagem `SHAPES.len() == all.len() + 2` fechava o círculo sobre a própria lista.

⚠️ **E este é o gate mais caro do módulo:** ele existe porque a W53 descobriu que o `Extrude` e o
`Revolve` viviam no motor **desde a W3** sem nenhum botão a alcançá-los — *uma família de features
inteira, completa e invisível*. O gate escrito para impedir a repetição **não a impedia**.

### §81.2 — ⭐ A corrente que fecha, e ela é a do `UnaryKind`

`ph2d_field::PrimitiveKind` + `Primitive::kind()`:

*um `Primitive` novo* → **erro de compilação** no `kind()` → *obriga uma variante em `PrimitiveKind`*
→ **`ALL` é um array de tamanho fixo** → não compila sem ela → *o gate percorre-a e exige o botão*.

⚠️ A enumeração já existia para os **modificadores** (`UnaryKind`) e não existia para as **formas** —
e era justamente nas formas que o buraco tinha custado uma família inteira.

### §81.3 — E a mutação achou a metade que faltava

| mutação | veredito |
|---|---|
| o painel perde o botão do `Torus` | ⭐ **MATA** — era o que o gate antigo deixava passar |
| duas famílias com a **mesma chave** | ⛔ **sobreviveu** na 1.ª versão |

A segunda passava porque `ends_with` encontrava o botão **da outra**: o `Revolve` keyed como `"box"`
achava o botão do `Box` e o gate dizia que estava tudo alcançável. ⇒ o gate passou a exigir um botão
**próprio** por família (índices distintos). *Duas formas a partilhar um botão é uma delas
inalcançável — e é o mesmo defeito, com outra roupa.*

### §81.4 — O resto da varredura

As outras promessas de *«erro de compilação»* do módulo foram conferidas e **cumprem-se**: a do
`field3d_view::View::of` (destructuring sem `..`), a do `field3d_export_job` (`Send + 'static`), e a
do `UnaryKind::ALL` (array de tamanho fixo). ⚠️ E o `ExportLevel`/`MeshFormat` já derivam os
seletores do `ALL` deles.

## §82 — W81: a marcha ganha um contador — e a normal era um quinto do quadro (27/08)

**Pedido do Enio:** o item 1 da lista — a base da pré-visualização (o quadro de movimento em
`26,7 ms` contra um orçamento de `16,7`).

⚠️ **A máquina esteve a `load 12`–`25` a jornada inteira** (outra linha a correr), e a lei do
`CLAUDE.md §5.0` diz que nenhuma leitura de relógio vale nada acima de `~5`. ⇒ **esta wave inteira é
de CONTAGEM**, que é load-independente. As tabelas de ms ficam para uma máquina calma, e a §82.7 diz
exactamente qual medição falta.

### §82.1 — ⛔ Três hipóteses minhas, refutadas antes de custarem código

| hipótese | como caiu |
|---|---|
| *«o `8,7` amostras/pixel da §73 divide por um quadro que é sobretudo fundo»* | ⛔ **falsa** — `209 299` dos `230 400` raios (**91 %**) entram na caixa e marcham. O denominador da §73 estava certo |
| *«o caminho de recuo (`fork` da árvore inteira, `2,89 ms`) dispara e ninguém o vê»* | ⛔ **falsa** — contador novo, **`0` recuos** num quadro |
| *«o `TILE` e o [`SLABS`] foram escolhidos num traçado serial»* | ⛔ **falsa** — as duas varreduras chamam `trace_tiled_for_test`, que é **paralelo**. ⚠️ O que sobra de verdade é que as duas mediram **com anti-serrilhado**, e a W72 tirou-o do quadro de movimento |

*Três refutações a custo de leitura são mais baratas que uma wave construída sobre a primeira.*

### §82.2 — ⭐⭐⭐ O instrumento: sete contadores, e o quadro passou a ser legível

Até aqui o quadro tinha **duas** contagens (`SPECIALISED`, `STEP_SAMPLES`) e um cronómetro
(`SPECIALISE_NS`). A W81 acrescenta o resto:

| contador | o que responde |
|---|---|
| `MARCH_RAYS` | quantos raios de facto marcham (o denominador que faltava) |
| `NORMAL_SAMPLES` | as amostras da **normal** — a parcela que faltava ao numerador |
| `STEP_HIST` | a **curva de sobrevivência** da marcha (uma média não escolhe entre curas opostas) |
| `FORKED` | os recuos para a árvore não especializada |
| `TILE_MAX` | o **ladrilho mais caro** — o chão que o relógio não fura |
| `SLAB_SAMPLES` · `SLAB_SPEC` | amostras e fitas **por fatia de profundidade** |

Todos custam **um atómico por passo por ladrilho** ou menos — nada por amostra.

### §82.3 — ⭐⭐⭐ O quadro de movimento, contado (`640×360`, 168 arestas, sem anti-serrilhado)

```
pixels 230 400 · raios que marcham 209 299 · acertos 74 417
marcha  1 674 558 amostras   (8,0 por raio)
normal    446 502 amostras   (6 por acerto)  ⇒  21,1 % de TODAS as amostras do quadro
```

⭐⭐⭐ **A normal é um quinto do trabalho de campo do quadro, e não estava em conta nenhuma.** O doc
da `STEP_SAMPLES` dizia que as amostras da normal *«saem noutro sítio»* — e o sítio **não existia**.
⇒ o `147,5 ns/amostra` que a §73 publicou dividia por um numerador a que falta `21 %`.

**A curva de sobrevivência** (amostras dadas ao `k`-ésimo passo):

| passos | 0–3 | 4–7 | 8–15 | 16–31 | 32–63 |
|---|---:|---:|---:|---:|---:|
| fracção das amostras | `63,4 %` | `20,2 %` | `11,3 %` | `3,3 %` | `1,8 %` |

**Por fatia de profundidade:**

| fatia | `0` (de fora) | 1 | 2 | 3 | 4 | `5` (de fora) |
|---|---:|---:|---:|---:|---:|---:|
| fitas montadas | **12** | 59 | 59 | 52 | 51 | **9** |
| amostras | **2 651** | 170 024 | 740 815 | 627 331 | 133 410 | **327** |
| amostras por fita | **220** | 2 881 | 12 556 | 12 064 | 2 615 | **36** |

⚠️ **As duas fatias de FORA custam `8,7 %` de toda a montagem para fazer `0,18 %` da marcha.** O doc
delas diz que *«custam zero quando ninguém lá chega»* — verdade, e incompleta: quando **um** raio lá
chega, elas compilam uma fita de JIT inteira, igual à de uma fatia cheia. *Uma fatia preguiçosa é
barata em média e não é barata em nenhuma unidade.* ⛔ **Não curado**, e o motivo é medido: as três
saídas (recuar para o `fork`, fundir com a vizinha, encurtar a faixa) custam entre `1,7 %` do quadro
e uma premissa sobre onde os raios entram — que é exactamente a premissa que a W56e removeu.

### §82.4 — ⭐⭐⭐ Ladrilhar e fatiar não custam UMA amostra — e eu tinha escrito o contrário

A primeira leitura da curva dizia: *«`35 %` das amostras são o passo `0` de uma fatia ⇒ fatiar cobra
`380 k` primeiros-passos»*. ⛔ **Falso, e a sonda apanhou-o**: o total de amostras é **`2 121 060`
em todos os tamanhos de ladrilho medidos** — `16`, `32`, `48`, `64`, `96`, `128` —, ao dígito.

⭐ O mecanismo é o que o doc da marcha já dizia sem tirar esta consequência: um raio carregado para a
fatia seguinte **não é reavaliado na fronteira**, ele retoma no `t` onde estava. ⇒ o número de
avaliações ao longo de um raio é o número de passos até convergir, e a fronteira de fatia não
acrescenta nenhum.

⭐⭐ **Isso simplifica o compromisso do [`SLABS`] até ao osso:** as amostras são constantes, logo o
único eixo é **quantas arestas cada amostra toca** (`128,6` a `N=1`, `67,2` a `N=4`) contra **quantas
fitas se montam** (`~60` contra `242`). *Uma constante cujo compromisso tem dois termos e não três é
uma constante que se pode raciocinar em vez de varrer.*

A lei tem gate — `the_tiling_changes_what_a_sample_costs_and_never_how_many_there_are`, em
[`tests/march_budget.rs`](../../crates/ph2d-field-render/tests/march_budget.rs). ⚠️ Ela é **mais
forte que a paridade de imagem**: a paridade diz que o raio chega ao mesmo sítio, esta diz que ele
percorre o **mesmo caminho**.

### §82.5 — ⭐⭐⭐ O PISO que o tamanho do ladrilho põe debaixo do quadro

Um ladrilho é **indivisível**: ele compila a própria fita e marcha os próprios raios, e nenhuma
thread o parte ao meio. ⇒ `relógio ≥ max(trabalho_total / threads, ladrilho_mais_caro)`.

`measure_the_floor_that_the_tile_size_puts_under_the_frame` (`640×360`, 168 arestas, 32 threads,
ideal por thread `66 283` amostras):

| lado | ladrilhos | fitas | mais caro | **PISO** |
|---:|---:|---:|---:|---:|
| 16 | 920 | 2 933 | 11 518 | **`1,00×`** |
| 32 | 240 | 849 | 34 710 | **`1,00×`** |
| 48 | 112 | 406 | 57 522 | **`1,00×`** |
| **64 (o que ship)** | **60** | **242** | **100 462** | **`1,52×`** |
| 96 | 28 | 118 | 199 012 | `3,00×` |
| 128 | 15 | 68 | 302 278 | `4,56×` |

⭐ **O ladrilho mais caro sozinho vale `1,52×` a fatia perfeitamente equilibrada de todo o trabalho
do quadro**, e o joelho da contagem está em `48`.

⛔⛔ **E a receita que este parágrafo escreveu — *«nenhum escalonador o cura, só partir o
ladrilho»* — foi MEDIDA e está REFUTADA.** Ver a §82.8: com a máquina calma, `48` e `64` empatam
dentro do ruído no caso do preview (duas corridas, vencedores opostos: `24,95` contra `25,11`, e
depois `25,70` contra `24,59`) e `64` ganha claramente na peça pesada. ⇒ **o tamanho do ladrilho não
é a alavanca**, e o `TILE = 64` sobrevive à reconferência no quadro que hoje ship.

⚠️ *O piso é real e não é alcançável por onde eu disse.* Ele é um **minorante** contado em amostras,
e a perda a sério é maior e tem outra causa — que a §82.8 nomeia com número.

### §82.6 — ⛔ RECUSA MEDIDA: o estêncil de QUATRO amostras para a normal

A normal é `21 %` das amostras e custa **seis** avaliações por acerto (diferença central nos três
eixos). O estêncil do **tetraedro** custa **quatro** — `1,5×` menos, `7 %` de todas as amostras do
quadro. `measure_what_the_four_sample_normal_changes` mediu o ângulo entre as duas normais, pixel a
pixel, em sete peças:

| peça | LISO máx | SILHUETA (n · máx) | VINCO (n · máx) |
|---|---:|---:|---:|
| **caixa afiada** (`round = 0`) | `0,028°` | `923` · **`18,05°`** | `1041` · **`21,92°`** |
| **cilindro afiado** (`round = 0`) | `0,048°` | `777` · **`27,22°`** | `533` · **`35,08°`** |
| **extrusão quadrada** (quinas do contorno) | `0,458°` | `954` · **`13,75°`** | `260` · **`16,55°`** |
| caixa com filete | `0,056°` | `862` · `0,048°` | `0` |
| esfera | `0,048°` | `792` · `0,044°` | `0` |
| toro | `0,048°` | `1088` · `0,044°` | `0` |
| extrusão 168 | `0,746°` | `867` · `0,480°` | `0` |

⭐ **A fronteira é exacta, e tem DUAS fontes independentes:** onde a peça tem uma quina de navalha —
de `round = 0` **ou** de um canto do contorno desenhado — o estêncil de quatro move a normal
`14°`–`35°`; em tudo o resto ele muda `≤ 0,75°`, que é invisível.

⛔ **RECUSADO.** O mecanismo: numa quina a normal verdadeira **não existe**, e o que a imagem precisa
ali é da **bissectriz** — a média das duas faces, que é o que faz a aresta ler-se como uma linha e
não como um degrau. A diferença central devolve-a por **simetria** (cada eixo é sondado nos dois
sentidos, e sobre a aresta os dois sentidos pertencem a faces opostas); os quatro sentidos do
tetraedro caem desigualmente nas duas faces e a normal inclina-se para a que apanhou mais.
*A quina afiada é a razão de existir deste módulo* (§1 do `lib.rs` do traçador).

⛔ **E a cura condicional não se paga:** ligar o estêncil barato só em peças sem quina exigiria um
predicado sobre o documento que soubesse das **duas** fontes — incluindo os cantos de um contorno
autorado, que é geometria de outro módulo. O prémio é `7 %` das amostras `≈ 1,5 ms` de `26,7`, e o
preço é uma heurística no caminho da feature de capa. *Não se põe um palpite a decidir a coisa que o
módulo existe para acertar.*

⭐ **O que FICA é a lei unificada:** `Stencil` é uma **tabela de deslocamentos**, e o gradiente é
`Σ dᵢ · f(p + ε·dᵢ)` — a diferença central colapsa nela exactamente (`[g₀−g₁, g₂−g₃, g₄−g₅]`), com o
caminho que ship **byte-idêntico**. *Um terceiro estêncil passa a ser uma linha de tabela, e a recusa
fica executável ao lado dela.*

### §82.7 — Gates, mutações, e o que fica aberto

| gate | mutação que **só ele** mata |
|---|---|
| `the_shipping_stencil_reads_a_crease_as_the_bisector_of_its_two_faces` | `NORMAL_STENCIL = Tetra4` · o par `−x` da tabela vira `+x` |
| `on_a_smooth_face_the_two_stencils_agree` | o tetraedro deixa de ser um tetraedro |
| `the_stencil_never_moves_the_silhouette` | uma guarda de magnitude calibrada num estêncil só |
| `the_tiling_changes_what_a_sample_costs_and_never_how_many_there_are` | o contador da normal conta **normais** em vez de **amostras** |

⭐ **E um gate antigo apertou:** o teto do `tape_budget` era `regiões + LADRILHOS + 1`, e a folga
media **`60`** — *uma folga num teto é o tamanho do ponto cego que ele tem*. Com o `FORKED` a contar
os recuos, o teto passa a `regiões + recuos + 1`, e os recuos são **`0`**. ⚠️ Aquele gate media
também com `SLABS = 2` desde a W70, enquanto o produto ship `4` desde a W71 — passou a ler
`slabs_for_test()`. *Um gate que escolhe a configuração mede a configuração que escolheu.*

⚠️ **Uma mutação SOBREVIVEU e não é um ponto cego** — foi conferido: erodir a conservadorismo do
corte por casco (`dmax × 0,9`) passa a suíte inteira, **e é inerte** — imagem, amostras (`1 674 558`)
e fitas por fatia saem byte-idênticas. A `0,5` já morde (mata um gate). ⇒ a margem do corte é
**folgada**, não vigiada de perto; e *uma mutação que não muda o produto não prova nada sobre os
gates* (o controlo é obrigatório).

**Aberto, na ordem** — ver a §82.8, que a máquina calma reescreveu:

1. ⭐⭐⭐ **Reaproveitar a fita entre quadros / especializar em espaço LOCAL.** É a alavanca, e o
   tecto dela **não** é os `20 %` que a §72.1 escreveu.
2. ⏸️ As duas fatias de fora (`8,7 %` da montagem por `0,18 %` da marcha) — as três saídas medidas e
   nenhuma se paga sozinha. ⚠️ Com a montagem barata elas deixam de importar; com a montagem cara
   elas são o mesmo problema.
3. ⛔ **`TILE` e `SLABS` estão FECHADOS** (§82.10): `64` e `4` sobrevivem à reconferência no quadro que hoje ship.

## §82.8 — ⭐⭐⭐ A máquina calma: a MONTAGEM não escala, e é ela a parede (27/08)

`measure_where_the_parallel_frame_stops_scaling`, `load < 2`, `640×360`, 168 arestas, **sem
anti-serrilhado** (o quadro que hoje ship), mediana de 3 rondas × 5, intercalado.

### §82.8.1 — A curva de escalamento

| threads | 1 | 2 | 4 | 8 | 16 | **32** |
|---|---:|---:|---:|---:|---:|---:|
| ms | `274,9` | `142,8` | `78,7` | `45,2` | `30,8` | **`23,8`** |
| ganho | `1,00×` | `1,93×` | `3,49×` | `6,08×` | `8,94×` | **`11,56×`** |
| eficiência | `100 %` | `96 %` | `87 %` | `76 %` | `56 %` | **`36 %`** |

⭐⭐⭐ **O quadro de movimento usa `36 %` da máquina.** ⚠️ E a queda começa **antes** do SMT: de 8
para 16 threads — núcleos físicos os dois — o ganho é só `1,37×`.

⭐ **A conta que isto reenquadra:** a `76 %` de eficiência o mesmo trabalho daria
`274,9 / (32 × 0,76) = 11,3 ms`, **abaixo do orçamento de `16,7`**. ⇒ *o buraco até aos 60 Hz é de
ESCALAMENTO, não de algoritmo.*

### §82.8.2 — ⭐⭐⭐ E a causa tem número: uma fita custa `1,93×` mais a 32 threads

| threads | 1 | 2 | 4 | 8 | 16 | **32** |
|---|---:|---:|---:|---:|---:|---:|
| montagem, ms de **CPU** | `106,8` | `108,7` | `111,2` | `112,5` | `149,2` | **`206,3`** |
| **ns por fita** | `441 335` | `449 112` | `459 489` | `464 797` | `616 395` | **`852 324`** |

⭐⭐⭐ **Compilar UMA fita custa quase o dobro do CPU a 32 threads do que a 1**, com as mesmas `242`
fitas. A montagem é **96 % JIT**, e um JIT mapeia memória **executável**: `mmap`/`mprotect` são
recursos do **kernel**, partilhados por todas as threads. ⇒ *a montagem não é uma fracção que se
divide entre núcleos: é uma fracção que em parte se **serializa**, e núcleos a mais tornam-na pior.*

⚠️⚠️ **E ela é `39 %` do quadro serial (`106,8` de `274,9`), não os `20 %` da §72.1.** Aquele `20 %`
foi medido **com anti-serrilhado** — a 2.ª passagem acrescenta marcha e **nenhuma** montagem, então
ela dilui a fracção. A W72 tirou o anti-serrilhado do quadro de movimento no dia seguinte, e a nota
ficou. *Quem move o número que sustenta uma nota tem de reconferir a nota* (§0.0) — quinta vez neste
módulo.

⇒ ⭐⭐⭐ **Reaproveitar a fita entre quadros vale MUITO mais que os `20 %` publicados:** ela é `39 %`
do trabalho serial **e** é a parte que degrada com o paralelismo. ⚠️ O número exacto é uma
extrapolação até alguém a construir, e a extrapolação está declarada: a marcha sozinha
(`274,9 − 106,8 = 167,1 ms` serial) a `70 %` de eficiência daria `~7,5 ms`.

### §82.8.3 — ⛔ O tamanho do ladrilho está fechado

Varredura de `TILE` no quadro que **hoje** ship (paralelo, sem anti-serrilhado, duas corridas):

| arestas | 32 | **48** | **64** | 96 |
|---|---:|---:|---:|---:|
| 168 (1.ª corrida) | `34,81` | **`24,95`** | `25,11` | `31,80` |
| 168 (2.ª corrida) | `35,29` | `25,70` | **`24,59`** | `31,47` |
| 672 | `114,49` | `87,08` | **`80,61`** | `112,75` |

⭐ **`48` e `64` empatam dentro do ruído no caso do preview** (os vencedores trocam entre corridas) e
`64` ganha por `7,4 %` na peça pesada. ⇒ **`TILE = 64` fica**, e o piso de `1,52×` da §82.5 **não é
alcançável por aí**. ⚠️ *Um piso contado em amostras é um minorante: ele diz que existe perda, não
onde ela está.*

### §82.9 — ⭐⭐⭐ O controlo: é o JIT, e ele satura às 16 threads

A §82.8.2 mediu a montagem **dentro** de um quadro, com as outras threads a marchar — e duas
explicações dão o mesmo número: *o JIT contende* ou *a marcha satura a memória e a compilação, que
corre ao lado dela, apanha a factura*. ⛔ **As duas mandam em waves opostas.**

`measure_whether_the_jit_contends_on_its_own` compila **242 regiões e não marcha uma única
amostra**:

| threads | 1 | 2 | 4 | 8 | **16** | **32** |
|---|---:|---:|---:|---:|---:|---:|
| ms (calma) | `130,5` | `66,3` | `34,2` | `19,2` | **`13,90`** | **`13,78`** |
| ms (2.ª corrida) | `128,6` | `64,9` | `33,3` | `19,2` | **`11,43`** | **`10,15`** |
| ns de CPU por fita (calma) | `539 079` | `548 212` | `565 874` | `633 566` | `919 025` | **`1 822 138`** |

⭐⭐⭐ **De 16 para 32 threads a compilação ganha `1 %`** (e `13 %` na 2.ª corrida): ela **satura**. O
tecto paralelo do JIT é `~9×`–`13×` e é atingido às **16** threads. ⇒ *a montagem não é uma vítima da
marcha: ela contende sozinha.* O mecanismo é o que se espera de um JIT — ele mapeia memória
**executável**, e `mmap`/`mprotect` são recursos do **kernel**, serializados entre todas as threads
do processo.

⚠️ **As duas corridas foram feitas com a máquina em estados diferentes** (`load < 4` e `load 41`) e
os números absolutos diferem; **a saturação aparece nas duas**, e é ela a afirmação. *Uma saturação é
robusta ao ruído de um jeito que uma diferença de `5 %` nunca é.*

### §82.10 — ⇒ O que isto responde, e o que fica

⭐⭐⭐ **O quadro de movimento (`~24 ms`) tem `~10`–`14 ms` de compilação de JIT que nenhuma thread
acelera.** É `~50 %` do relógio, é a parte que não escala, e é **exactamente** o trabalho que se
repete inteiro a cada quadro enquanto a mão mexe. ⇒ *o buraco até aos 60 Hz é a montagem, e a cura é
não a repetir* — reaproveitar a fita entre quadros / especializar em espaço **LOCAL**, que a §72.1
nomeou e precificou em `20 %` a partir de um quadro **com anti-serrilhado** que o preview já não
desenha.

⛔ **E as duas constantes sobreviveram à reconferência no quadro que hoje ship:**

| constante | varredura nova (paralela, sem AA) | veredito |
|---|---|---|
| `TILE` | `168`: `32→40,0` · `48→27,8` · **`64→25,4`** · `96→32,5` | **`64` fica** (`48` empata dentro do ruído em duas de três corridas) |
| `SLABS` | `168`: `N=2→51,9` · `N=3→44,8` · **`N=4→35,0`** · `N=6→52,0` | **`4` fica** (na peça de `672` o `6` ganha por `2 %`) |

⚠️ **Uma das corridas da varredura foi descartada** — a máquina ficou ruidosa a meio (o quadro de 1
thread saltou de `274,9` para `347,2 ms` e a linha de `672` arestas mudou por `3×`). Ficaram as
comparações **intercaladas** dessa corrida, que são o que ela ainda mede, e os absolutos da corrida
calma. *Um A/B intercalado sobrevive ao ruído comum; um absoluto não.*

### §82.11 — ⚠️ Um membro novo da família das flakes de recurso

`an_abandoned_march_returns_nothing_and_returns_fast` reprovou uma vez na suíte com a máquina a
`load 42` (outra linha a correr) e passou **3 de 3** sozinha, sem uma linha de produto mexida entre
as duas coisas. Ele mede *«e volta DEPRESSA»*, que é um relógio — a assinatura exacta da família que
o `CLAUDE.md §5.0` descreve: *um gate que mede um recurso partilhado reprova sob carga e passa
sozinho na máquina calma.*

⚠️ **Não é uma lista para crescer** (o §5.0 diz-lo): é o **mecanismo** que se reconhece. Fica aqui
porque este módulo passou a ter um, e porque a wave que o encontrou é precisamente a que mede
relógios.

### §82.12 — ⭐⭐⭐ A fita de um quadro serve o quadro seguinte? — a medição que desenha a W82

⭐ **Uma fita construída para a região `R` é válida em toda a sub-região de `R`** — é a cerca que a
W56 já escreveu, e ela é o mecanismo inteiro: se a fita for construída para `R` **inflada** por `f`,
ela serve o quadro seguinte sempre que a região nova ainda lá caiba. ⇒ *a cache não precisa de chave
nenhuma; precisa de um teste de contenção.*

`measure_whether_one_frames_tape_serves_the_next` (`640×360`, tile `64`, `SLABS = 4`, arrasto = uma
órbita de `g` graus por quadro):

| arrasto | `f = 1,00` | `f = 1,25` | `f = 1,50` | `f = 2,00` | `f = 3,00` |
|---|---:|---:|---:|---:|---:|
| `1°` | `9,0 %` | **`92,8 %`** | `95,5 %` | `96,7 %` | `98,5 %` |
| `2°` | `9,0 %` | `84,3 %` | **`92,8 %`** | `94,9 %` | `96,7 %` |
| `4°` | `7,5 %` | `48,9 %` | **`82,9 %`** | `91,0 %` | `93,8 %` |
| `8°` | `6,1 %` | `19,8 %` | `49,1 %` | **`81,7 %`** | `90,2 %` |
| **arestas por região** | `74,6` | `88,3` | `102,1` | `124,6` | `148,0` |
| **preço por amostra** | `1,00×` | **`1,18×`** | `1,37×` | `1,67×` | `1,98×` |

⭐⭐⭐ **A `f = 1` a cache acerta `9 %`** — cachear a região *exacta* não serve para nada, e é a
inflação que é o mecanismo. A `f = 1,25` ela acerta `84 %`–`93 %` às velocidades de arrasto reais
(um quadro de `24 ms` a `90°/s` é `2,2°`) por **`1,18×`** no custo de uma amostra.

**A conta, com os números da §82.9** (quadro `~24 ms` ≈ `14` de montagem + `10` de marcha):

| f | montagem que sobra | marcha | total estimado |
|---|---:|---:|---:|
| hoje | `14,0` | `10,0` | `24,0 ms` |
| `1,25` a `2°` | `2,2` | `11,8` | **`14,0 ms`** |
| `1,50` a `4°` | `2,4` | `13,7` | `16,1 ms` |

⇒ ⭐⭐⭐ **abaixo do orçamento de `16,7 ms`**, e sem tocar num algoritmo. ⚠️ **É uma estimativa** — o
`14 + 10` não é uma separação limpa (as duas coisas intercalam-se por ladrilho), e o número a sério
só sai depois de construído.

⚠️ **O que a construção ainda tem de resolver, e está nomeado:** (1) a cache **vive entre quadros**,
logo ela não cabe no `RegionCompiler`, que nasce e morre com o quadro — ela é do dono do traçado;
(2) ela tem de **morrer com o documento** (editar a peça invalida tudo); (3) precisa de despejo, ou
cresce com cada grau que a câmera roda; (4) a fita compilada é `Arc<Mmap>` na `fidget` ⇒ **cloná-la é
um bump de contador**, e o avaliador (que é o estado mutável) nasce por uso — *é isso que torna uma
cache partilhada entre threads possível de todo*.

## §83 — W82: a cache de fitas entre quadros — a cura da parede da §82.9 (27/08)

A W81 mediu a parede: compilar as `242` fitas de um quadro custa `~14 ms` de um quadro de `~24` e
**satura às 16 threads** (de 16 para 32 o ganho é `1 %`), porque um JIT mapeia memória **executável**
e `mmap`/`mprotect` são recursos do kernel. ⇒ metade do relógio de um quadro é trabalho que nem
escala nem muda, refeito inteiro a cada quadro enquanto a mão mexe.

### §83.1 — ⭐⭐⭐ O mecanismo: a cache não tem chave, tem um teste de CONTENÇÃO

A cerca que a W56 escreveu é *«a árvore especializada só vale DENTRO de `[lo, hi]`»* — e ela lê-se ao
contrário: **uma fita construída para `R` serve toda a sub-região de `R`**. ⇒ construindo a fita para
`R` **inflada**, ela serve o quadro seguinte sempre que a região nova ainda lá caiba, e a cache não
precisa de chave nenhuma: ela precisa de uma comparação de caixas.

⭐⭐ **A `f = 1` a cache acerta `9 %`** (§82.12): guardar a região *exacta* não serve de nada. *A
inflação é o mecanismo, não uma afinação*, e o `INFLATE = 1,25` é o número medido.

### §83.2 — ⚠️ O que ela deixa cair de propósito, e o preço

O caminho sem cache especializa contra o **casco** do tubo do ladrilho (W59), que é mais apertado que
a caixa. Uma fita guardada tem de valer numa forma que se possa **testar depressa e sem
ambiguidade** — e essa forma é a **caixa**. ⇒ uma fita da cache guarda mais arestas que a de hoje,
mesmo antes de a inflar. *A cache troca aresta por compilação.*

### §83.3 — ⭐⭐⭐ O que ela custou: o substrato dizia sim, e o compilador prova-o

A fita da `fidget` é um **`Arc<Mmap>`** por dentro ⇒ **cloná-la é um incremento de contador**; o que
**não** se partilha é o avaliador (`ShapeBulkEval`), que é o estado mutável, e esse nasce por uso, de
graça. *Uma fita serve todas as threads; um avaliador serve uma.* A `ph2d_field_eval::hybrid`
ganha o `RegionTape` (compilar) e o `Hybrid::from_region_tape` (usar sem compilar) — e a afirmação
não é um comentário: um `const _: () = is_send_sync::<RegionTape>();` **deixa de compilar** se um
campo futuro lhe tirar o `Send + Sync`.

### §83.4 — ⛔⛔ O defeito que eu mesmo pus lá, e que só o relógio via

A 1.ª versão carimbava a idade de uso (a régua do despejo) tomando o cadeado de **escrita** — `~200`
acertos por quadro, vindos de 32 threads. Medido:

| | acerto | fitas compiladas | quadro |
|---|---:|---:|---:|
| sem cache | — | `225` | `26,0 ms` |
| **com cache, cadeado de escrita** | **`87 %`** | **`30`** | `26,1 ms` · e num caso **`146,7`** contra `109,0` |

⭐⭐⭐ **O acerto era `87 %`, as compilações caíram `7,5×`… e o quadro não mexeu — num caso ficou
`0,74×`.** *Uma cache que serializa os leitores dela devolve na trava o que poupou no JIT.* A cura é
o carimbo ser um **atómico**, que cabe debaixo do cadeado de **leitura** — o que 32 threads tomam ao
mesmo tempo. ⚠️ E o defeito era **invisível a todo gate de saída**: a imagem, os acertos e as
compilações estavam todos certos.

### §83.5 — ⭐⭐⭐ A correcção: a cache não muda a imagem, e a barra é o CONTROLO

A fita guardada é construída para outra região (inflada, e caixa em vez de casco) ⇒ **a árvore
especializada não é a mesma árvore**. A pergunta não é retórica.

Medido sobre um arrasto de 8 quadros, `200×120`:

| régua | cache |
|---|---|
| pixels que mudaram de acerto | **`0`** |
| normal, pior ângulo | `0,056°` |

⚠️ **`0,056°` é o ÚLTIMO BIT, e não uma folga.** A normal sai de uma diferença central com passo
`1e-4` sobre um valor que ali é quase zero: um ULP de `f32` (`1,2e-7`) sobre uma componente de
`~2e-4` dá `~0,03°`. E a causa está escrita desde a W56, no gate irmão: *«as duas árvores são
algebricamente a mesma, mas o `min` corre sobre subconjuntos diferentes: a soma e a raiz caem em
ordens diferentes e o resultado difere no último bit»*.

⭐⭐⭐ **⇒ a barra do gate não é absoluta: é o CONTROLO.** `the_cache_never_changes_the_image`
compara, no mesmo arrasto, o desacordo *cache contra especialização* com o desacordo
*especialização contra a marcha por linha* — e exige que o primeiro **não seja maior**. *Uma barra
absoluta aqui seria uma afirmação sobre o escalonador do JIT.*

### §83.6 — Gates, mutações, e a margem medida

| gate | mutação que **só ele** mata |
|---|---|
| `the_cache_never_changes_the_image` | a contenção afrouxada em `0,2` |
| `the_cache_dies_with_the_document` | o `begin` deixa de limpar quando o documento muda |
| `a_drag_stops_recompiling_the_tapes_it_already_has` | a cache guarda tudo e **não serve nada** |

⚠️ **A contenção tem margem, e ela está medida:** afrouxá-la em `0,005`, `0,02` e `0,1` (numa peça
que mede `~1,2`) é **inerte** — nem a imagem nem as contagens mexem; a `0,2` só o gate da imagem cai;
a `0,4` caem os dois. *Uma mutação que não muda o produto não prova nada sobre os gates*, e por isso
o controlo de inércia é obrigatório antes de se declarar um ponto cego.

⚠️ **O contador é o que separa uma cache boa de uma inútil.** O `TAPE_HITS` existe porque *contar o
trabalho feito não é contar o trabalho poupado*: uma cache que nunca acerta passa em **todos** os
gates de imagem com nota máxima.

### §83.7 — ⭐⭐⭐ O que ela compra, medido — e a minha estimativa estava ERRADA

`measure_what_the_tape_cache_buys`, `640×360`, 168 arestas, sem anti-serrilhado, arrasto de 12
quadros, **A/B intercalado** ronda a ronda, mediana de 3×11, o 1.º quadro de cada arrasto descartado
(ele enche a cache do zero e não representa o regime):

| arrasto | `f` | sem | **com** | ganho | fitas/quadro | acertos |
|---|---:|---:|---:|---:|---:|---:|
| `1°` | `1,00` | `28,00` | `29,08` | **`0,96×`** | `193` | `33` |
| `1°` | `1,10` | `27,51` | `24,31` | `1,13×` | `52` | `173` |
| `1°` | **`1,25`** | `29,40` | **`23,99`** | **`1,23×`** | `16` | `209` |
| `1°` | `1,50` | `29,22` | `24,79` | `1,18×` | `5` | `220` |
| `2°` | `1,00` | `26,45` | `29,55` | **`0,90×`** | `203` | `29` |
| `2°` | `1,10` | `28,75` | `28,28` | `1,02×` | `130` | `102` |
| `2°` | **`1,25`** | `26,82` | **`22,67`** | **`1,18×`** | `41` | `190` |
| `2°` | `1,50` | `27,21` | `24,74` | `1,10×` | `11` | `220` |
| `4°` | **`1,25`** | `23,20` | **`20,12`** | **`1,15×`** | `44` | `181` |
| `4°` | `1,50` | `21,96` | `22,47` | `0,98×` | `10` | `216` |
| `4°` | `2,00` | `23,08` | `26,62` | **`0,87×`** | `2` | `223` |

⭐⭐⭐ **`1,25` ganha nas três velocidades**, e as duas pontas **perdem**: a `1,00` a cache quase não
acerta e paga a fita mais gorda por nada; a `2,00` ela acerta quase sempre (`223` de `226`) e **ainda
assim perde `0,87×`**, porque a fita que ela serve guarda arestas a mais. *As duas pontas são a mesma
conta lida nos dois sentidos, e é isso que faz do `1,25` um mínimo e não um palpite.*

### §83.8 — ⛔ E a minha estimativa da §82.12 estava ERRADA por dois motivos, os dois nomeados

Eu escrevi *«`14 + 10 = 24`, com cache `2,2 + 11,8 = 14 ms`»*. Medido: **`20`–`24 ms`**, um ganho de
`1,15×`–`1,23×` e não de `1,7×`.

1. ⛔ **`24 = 14 + 10` não é uma decomposição.** A montagem e a marcha **não correm em série**: cada
   ladrilho monta a sua fita e marcha logo a seguir, nas mesmas 32 threads. Os `~14 ms` da §82.9 são
   o que `242` compilações custam **sozinhas e todas ao mesmo tempo**; dentro de um quadro elas
   sobrepõem-se à marcha, e tirá-las não devolve `14 ms` de relógio. *Somar duas medições feitas em
   regimes diferentes não é uma conta — é duas contas encostadas.*
2. ⛔ **A fita da cache é mais GORDA, e o preço estava na tabela desde o início.** Ela é construída
   para a **caixa** (e não para o casco do tubo) e ainda **inflada**: `88,3` arestas por região
   contra as `67,2` do casco de hoje — **`1,31×`** por amostra. A §82.12 pôs `1,18×` naquela linha
   porque comparou caixa com caixa; *a coluna estava certa e a comparação era com o número errado*.

⭐ **A prova de que o item 2 é real está na própria tabela:** a `f = 1,00` a cache troca casco por
caixa **sem** ganhar acerto nenhum, e o quadro fica **`0,90×`–`0,96×`**. Isso é o preço do casco,
medido sozinho.

### §83.9 — ⇒ O que fica aberto, e ele está nomeado

⏳ **A cache contra o CASCO, e não contra a caixa.** O que ela deixa na mesa é o `1,11×` de
`74,6 → 67,2` arestas, e a razão de a caixa ter sido escolhida é o **teste**: uma caixa compara-se em
seis desigualdades, um casco convexo pede um ponto-em-polígono por vértice. ⭐ A saída é um teste em
**dois níveis** — a caixa rejeita quase tudo, e só os sobreviventes pagam o casco. ⚠️ Com `~1 500`
fitas guardadas e `242` consultas por quadro, o custo do teste **é** parte do orçamento, e essa é a
segunda razão para o primeiro nível ser barato.

⏸️ **O custo da consulta em si por medir**: a varredura é **linear** sobre as fitas guardadas
(`~1 500` entradas × `242` consultas = `~460 k` testes de caixa por quadro). A `f = 1,50` a cache
guarda `394`–`754` entradas e **não** ganha ao `1,25` com `1 167`–`1 891` — o que sugere que a
varredura não domina, mas as duas variáveis estão **confundidas** naquela linha (a fita também é mais
gorda). *Uma comparação em que duas coisas mudam ao mesmo tempo não mede nenhuma das duas.*

### §83.10 — ⛔⛔ O smoke do Enio: *«não parece ter melhorado»* — e ele tinha razão duas vezes

**Report (27/08):** *«não tenho certeza mas não parece ter melhorado»*.

#### §83.10.1 — O defeito: a cache era deitada fora DUAS VEZES POR GESTO

⚠️ **O app alterna DOIS documentos, por construção.** `field3d_preview::coarse_doc` dá o contorno
**grosso** enquanto a mão mexe e o **cheio** corre ao parar (a escada da W73) — e a cache morria com
o documento, de propósito. ⇒ *cada paragem e cada retoma custavam um quadro frio.*

`measure_the_stop_and_go_cycle_the_app_really_does` (o ciclo a sério: 4 quadros a girar + os 2
degraus do assentar, três vezes):

| | fitas compiladas | acertos |
|---|---:|---:|
| **antes** — 1.º quadro depois de cada transição | `68` | **`0`** |
| **antes** — regime dentro de um arrasto | `2`–`18` | `50`–`67` |
| **depois** — 1.º quadro depois de uma retoma | `5`–`14` | `54`–`61` |
| **depois** — regime | `3`–`12` | `57`–`64` |

⭐⭐⭐ **Dois quadros em cada seis eram frios**, e nenhuma bancada de arrasto **contínuo** podia
vê-lo — a minha media um documento só. *Uma cache mede-se no ciclo que o artista faz, e o ciclo dele
tem uma paragem.*

⭐ A cura é a cache guardar **dois** documentos (`DOCS = 2`) com uma etiqueta por fita
(`Entry::doc_id`), e o `2` não é folga: é a contagem dos degraus do preview. ⚠️ E o gate que dizia
`the_cache_dies_with_the_document` passou a chamar-se
`a_cached_tape_is_never_served_to_another_document` — *um nome que descreve o mecanismo envelhece com
ele; um que descreve a lei não.*

#### §83.10.2 — ⭐ E mesmo curado, ele continua a ter razão: `1,2×` não se vê

Ciclo inteiro, A/B **intercalado**, mediana de 5: `774,55 → 628,43 ms` = **`1,23×`**.

⚠️ **`1,23×` está abaixo do que uma pessoa distingue**, e há duas razões estruturais para ele não
aparecer na tela:

1. **A escada do preview tem degraus de `4×` em pixels** (`preview_size` escolhe um **divisor**
   inteiro). Um ganho de `1,23×` **nunca** chega para descer um degrau ⇒ a imagem sai exactamente do
   mesmo tamanho, `1,23×` mais depressa — e `18 ms` contra `22 ms` não é uma coisa que se veja.
2. **O tempo do ciclo não está onde eu olhei.** Medido: os quadros a **girar** custam `13`–`23 ms` e
   os dois degraus do **assentar** custam `52`–`102 ms` cada. ⇒ *o assentar é `~2,2×` o custo de
   girar, e é ele que o artista espera.*

⭐⭐⭐ **⇒ A wave seguinte não é mais velocidade no quadro de movimento: é o ASSENTAR.** Ele corre com
o contorno **cheio** (`672` arestas contra `168`) e é onde o relógio de facto está.

⚠️ *O report «não parece ter melhorado» é um dado, e ele mediu duas coisas que eu não tinha medido:
que a cache não sobrevivia ao gesto, e que `1,2×` não é visível.*

## §84 — W83: o assentar — e o que sobrava a compilar era o ANTI-SERRILHADO (27/08)

O smoke do Enio mandou olhar para o **assentar** (§83.10.2: girar custa `13`–`23 ms` por quadro e
cada degrau do assentar custa `52`–`102`). A primeira coisa que a sonda fez foi corrigir **duas
suposições minhas**.

### §84.1 — ⛔ Duas correcções antes do primeiro número

1. **A peça na resolução de OMISSÃO não alterna documento nenhum.** O `PREVIEW_MAX_EDGES` é `168`,
   que é *exactamente* o que o contorno já tem por omissão ⇒ `coarse_doc` devolve `None` e o
   documento é o mesmo o tempo todo. ⇒ *a cura da W82b (dois documentos) só morde numa peça cuja
   `Resolution` o artista subiu.* A minha bancada tinha escolhido a peça errada nas duas direcções.
2. ⛔ **A minha sonda avançava a câmera no assentar**, e o assentar acontece precisamente porque ela
   **parou**. *Uma sonda que muda uma variável a mais mede outra coisa* — e neste caso media a
   cache a falhar por um movimento que o app não faz.

### §84.2 — ⭐⭐⭐ O que sobrava a compilar, contado

`measure_the_settle_of_a_default_resolution_piece` (peça de omissão, um documento só, câmera parada
no assentar):

| o que o app faz | fitas compiladas | acertos na cache | pixels de borda |
|---|---:|---:|---:|
| gira (regime) | `1`–`36` | `202`–`240` | `0` |
| **DEGRAU 1** (mesmo tamanho, com anti-serrilhado) | **`29`** | `240` (**100 %**) | `1 762` |
| **DEGRAU 2** (tamanho cheio, com anti-serrilhado) | **`70`** | `835` | `3 526` |

⭐⭐⭐ **As `29` são `1 762 ÷ 64 = 28` lotes de pixels de borda, mais uma** — e a passagem primária
estava a **100 %** de acerto. *Todo o que sobrava a compilar era o anti-serrilhado*, e cada lote
compilava a árvore **INTEIRA**, que é a mais cara que existe (sem especialização nenhuma).

### §84.3 — ⭐⭐⭐ A cura são cinco linhas, e a recusa que a bloqueava dissolveu por medição

O `Hybrid::fork` **recompilava**. Não precisa: a fita da `fidget` é um `Arc<Mmap>` e o que o lote
precisa é de um **avaliador** (o rascunho mutável), não de um compilador.

⚠️ **A W70 mediu isto e achou-o NEUTRO** — e a nota dela dizia porquê: *«o quadro tem `917` regiões
especializadas nesse tamanho: as dezenas de fitas desta passagem são ruído ao lado delas»*. ⭐ **A W82
apagou aquele `917`**, e com ele a premissa: de ruído, esta passagem passou a ser a **totalidade**.
*Quem move o número que sustenta uma nota tem de reconferir a nota* — e quem o moveu fui eu, um dia
antes.

⚠️ **E a cura não é a que a W70 tentou.** Ela tentou reaproveitar o **avaliador**, que é estado
mutável e não atravessa threads; o que atravessa é a **fita**. *O que se partilha é o código; o que
se duplica é o rascunho.*

| | fitas compiladas |
|---|---:|
| DEGRAU 1, antes | `29` |
| **DEGRAU 1, depois** | **`1`** |
| DEGRAU 2, antes | `70` |
| **DEGRAU 2, depois** | **`14`** |

### §84.4 — A decomposição do assentar, medida

`measure_where_the_settle_goes` (`load ~6`, os absolutos valem pouco, as **razões** dentro da corrida
valem):

| o que corre | sem cache | com cache | contra o movimento |
|---|---:|---:|---:|
| movimento (grosso, sem AA) | `30,0` | `21,1` | `1,00×` |
| **+ contorno cheio** (`672` contra `168`) | `101,8` | `75,3` | **`3,39×`** |
| **+ anti-serrilhado** | `33,4` | `27,2` | **`1,11×`** |
| DEGRAU 1 (os dois) | `115,8` | `90,0` | `3,86×` |
| DEGRAU 2 (+ tamanho cheio, `4×` os pixels) | `275,0` | `199,4` | `9,17×` |

⭐⭐ **O anti-serrilhado custava `1,34×` (§73.1) e passou a custar `1,11×`** — é a W83, medida.

⭐⭐⭐ **E o factor que manda no assentar de uma peça de resolução ALTA é o CONTORNO (`3,39×`), não o
anti-serrilhado.** Numa peça de omissão o contorno não muda ⇒ o degrau 1 custa `~1,1×` um quadro de
movimento e o degrau 2 custa o que os **pixels** custam. *O que sobra ali não é desperdício: é a
imagem final.*

**Gate:** `the_antialias_pass_compiles_no_tape_of_its_own` (binário próprio), com a mutação que só
ele mata — o `fork` volta a compilar.

## §85 — W84: o decimador do preview apagava QUINAS, e quem sobrevivia era uma lotaria (27/08)

**Report do Enio (27/08):** *«piorou muito»*, logo depois de eu lhe pedir para subir o `Resolution`.
E a corrida seguinte dele fechou metade do diagnóstico: **desligar a cache é PIOR** (`PH2D_FIELD_TAPE_CACHE=0`,
*«mais lento»*) ⇒ a cache ajuda, e o que ele viu não foi ela.

### §85.1 — ⭐ O que o `Resolution` alto de facto compra

`measure_how_many_contour_edges_are_visible` — o contorno decimado contra um de `2048` pontos, e a
régua final é o **PIXEL sombreado** (níveis de 8 bits), não a normal:

| arestas | pixels que mudam | normal p99 | **pixel p99** | **pixel máx** |
|---:|---:|---:|---:|---:|
| `672` | `0,000 %` | `0,266°` | `1` | `1`–`2` |
| `336` | `0,009 %` | `0,529°` | `1` | `2`–`3` |
| `168` | `0,015 %` | `1,056°` | `3` | `4` |
| `84` | `0,067 %` | `2,110°` | `5` | `9`–`10` |
| `42` | `0,283 %` | `4,218°` | `9` | `19`–`21` |

⭐⭐ **A silhueta quase não mexe; o que mexe é a NORMAL**, e ela escala **exactamente** com `1/n`
(`4,218 → 2,110 → 1,056 → 0,529 → 0,266`). *A resolução do contorno não é sobre o contorno: é sobre
a luz.*

⚠️ **E a régua é independente do tamanho da imagem** — os mesmos números a `640×360` e a `1600×900`.
⇒ ⛔ **a ideia de derivar o tecto do tamanho do pixel está REFUTADA**: o erro que se vê é angular, e
um ângulo não encolhe com a resolução da tela.

### §85.2 — ⛔⛔ E o decimador apagava QUINAS

O [`ph2d_field::coarsen`] tirava **um em cada `k`** vértices, com esta justificação no doc dele: *«um
contorno achatado por tolerância tem os pontos densos onde a curvatura é alta — então tirar um em
cada `k` preserva o carácter da forma»*. ⭐ **Isso é verdade para curvatura**, que é distribuída por
muitos vértices.

⚠️ **Uma QUINA não é curvatura distribuída: é um vértice só, com todo o ângulo dentro.**

`measure_whether_the_preview_decimation_eats_corners` — uma estrela de 5 pontas, `400` pontos, as
quinas em múltiplos de `40`:

| tecto | passo | arestas depois | pixels que mudam | normal p99 | **normal máx** |
|---:|---:|---:|---:|---:|---:|
| `336` | `2` | `200` | `0` | `0,034°` | `0,048°` |
| **`168`** | **`3`** | **`134`** | **`509` (`0,87 %`)** | **`28,1°`** | **`126,8°`** |
| `84` | `5` | `80` | `0` | `0,034°` | `0,048°` |

⭐⭐⭐ **Se uma quina sobrevive depende de o índice dela ser divisível pelo passo.** Com `2` e `5`
elas vivem; com `3` **três em cada cinco morrem** — e o `PREVIEW_MAX_EDGES` que ship é `168`, que dá
passo `3`. *Uma forma que sobrevive ou não conforme a aritmética do índice não é uma lei: é um
acidente.*

### §85.3 — ⭐⭐⭐ A cura: decimar por GIRO, e a quina fica por CONSTRUÇÃO

O orçamento passa a ser de **ângulo**: a curvatura total da peça (sem sinal) repartida pelo número de
arestas pedido, e um vértice é mantido quando o giro acumulado desde o último chega ao orçamento.

⭐ **Por que o giro é a grandeza certa:** o erro de uma corda que substitui um arco é fixado pelo
**ângulo** que o arco varre, e o erro que se **vê** é o da normal — que é esse mesmo ângulo (a §85.1
mediu-o: `∝ 1/n`). ⇒ repartir o giro por igual distribui o erro por igual, e **um vértice que sozinho
gasta o orçamento — uma quina — é mantido sem uma regra própria a dizê-lo.**

| a estrela, decimada a `168` | antes | **depois** |
|---|---:|---:|
| pontos | `134` | **`10`** |
| pixels que mudam | `509` | **`0`** |
| normal máx | `126,8°` | **`0,283°`** |

⭐⭐⭐ **`400` pontos passam a `10` — exactamente as quinas — e a imagem sai IDÊNTICA.** Os `390`
vértices ao longo das arestas rectas não tinham forma nenhuma para preservar, e o decimador antigo
gastava metade do orçamento neles enquanto perdia a forma que existia.

⚠️ **O tecto passa a ser um ALVO, não uma parede:** uma peça com mais quinas do que o orçamento
mantém as quinas. *Não há como representar uma forma de duzentas quinas em cento e sessenta arestas
sem deixar de ser aquela forma*, e a resposta certa é gastar mais e não mentir.

**Gates:** `a_corner_survives_the_coarsening` · `the_coarsening_spends_its_budget_on_turn_not_on_length`,
com a mutação que os mata (voltar ao passo por índice). Os três gates antigos do `coarsen` ficam
verdes.

## §86 — W85: o preview pede um ERRO, e a contagem de arestas sai da forma (27/08)

A W84 fez a decimação repartir **giro**, e isso tornou a contagem de arestas uma **consequência** em
vez de uma lei: o que o orçamento de giro fixa é o **erro da normal**, que é metade do ângulo que uma
corda substitui — e a normal é o que a luz mostra. ⇒ *pedir um erro é pedir a coisa que se vê; pedir
uma contagem é pedir um número que só a esperança liga ao que se vê.*

### §86.1 — ⭐⭐⭐ E isso destrava o que o Enio pagou: o assente também engrossa

O `PREVIEW_MAX_EDGES = 168` era uma contagem e só valia **a mexer**. Agora são **dois orçamentos de
erro**, e o que muda entre os dois quadros é *quanto erro de sombreado se tolera*:

| | orçamento | num círculo |
|---|---:|---:|
| a mexer (`MOVING_NORMAL_ERR_DEG`) | `1,0°` | `168` arestas — **o que já shipava** |
| **ao assentar** (`SETTLED_NORMAL_ERR_DEG`) | **`0,5°`** | **`336`** |

⭐ **O `1,0°` reproduz o quadro de movimento ao bit para uma peça de omissão** (medido: `168` arestas
dão `1,056°` de erro p99), e o `0,5°` é onde a §85.1 mediu a imagem parar de mudar (`≤3` níveis de
`255`, contra o **dobro** do preço).

**O que isso corta, medido** (`coarsen_to_normal_error` sobre círculos):

| contorno autoral | a mexer | ao assentar | corte do assente |
|---:|---:|---:|---:|
| `168` (omissão) | `168` | `168` | **`1,00×`** |
| `336` | `168` | `336` | `1,00×` |
| `672` | `168` | **`336`** | **`2,00×`** |
| `940` | `157` | **`314`** | **`2,99×`** |

⭐⭐⭐ **Uma peça de omissão não muda nada; uma de `Resolution` alto paga `2×`–`3×` menos no
assentar** — que é exactamente o quadro que o artista espera, e exactamente o custo de que ele se
queixou.

### §86.2 — ⚠️ A lei que mudou, e o gate que a diz

O gate chamava-se `the_contour_only_coarsens_while_the_hand_is_moving` e dizia: *«um preview que
engrossasse sempre entregaria ao artista uma peça que nunca fica nítida»*. ⭐ **O que faltava era
saber onde a nitidez para de aparecer**, e a §85.1 mediu-o. A lei nova é mais forte que a antiga:
**o assente engrossa até onde a imagem deixa de mudar, e nem um bocado mais.** *O que se corta ali
não é nitidez: é trabalho que ninguém vê.*

Ele passou a chamar-se `both_frames_coarsen_but_the_moving_one_coarsens_more`, e as duas metades
continuam a ser o gate — sem a segunda, *grosso a mexer, nítido ao assentar* deixou de existir.

### §86.3 — ⛔ E a pergunta que isso abre tem gate próprio

*Se o assente também engrossa, o `Resolution` do artista ainda serve para alguma coisa?* **Serve, e é
aqui: ele governa a malha que sai para o ARQUIVO**, que é onde ele não é desperdício — uma malha
exportada é lida de perto, medida e reimportada; um pixel de um preview não.

⚠️ `the_export_never_goes_through_the_preview_coarsening` é uma **varredura de fonte**, porque o que
se defende é a **ausência** de uma chamada: nenhum teste de saída prova que um caminho não foi
tomado. Mutação que o mata: um segundo chamador do `coarse_doc`.

⛔ *Se esse gate cair, subir o `Resolution` deixa de ter qualquer efeito observável* — e o knob vira
um controle que consome o gesto e não faz nada, que é o defeito que este módulo já pagou três vezes.

## §87 — W86: ⛔ RECUSA CONFIRMADA — o perfil como CONSULTA perde para a fita especializada (27/08)

Com a montagem quase eliminada (W82/W83), o quadro de movimento passou a ser **quase só marcha**, e o
custo dela é `amostras × arestas por região`. A direcção nomeada desde a W56 era trocar a fita por uma
**consulta** ao índice do contorno (um BVH), e a nota dela dizia: *«a `ProfileIndex` responde em
`40 ns` o que a fita responde em `155`»*.

⚠️ **Metade da recusa da W56 tinha dissolvido:** ela dava dois motivos — os **modificadores** (que não
passam numa folha amostrada) e a **quina viva** (o gradiente exacto). ⭐ O segundo caiu na W70: o
traçado lê a normal por **diferença central** na fita float, e quem consome o gradiente exacto é a
**extração**, que não passa pelo caminho por região.

### §87.1 — A medição, na região que de facto ocorre

`measure_the_query_against_the_specialised_tape` (`200 k` pontos por célula, mediana de 5):

| arestas do contorno | região (lado) | guardadas | fita (ns/pt) | consulta BVH | **fita/BVH** |
|---:|---:|---:|---:|---:|---:|
| `168` | `1,20` (a peça) | `168` | `246,9` | `283,2` | `0,87×` |
| `168` | `0,60` | `168` | `125,8` | `489,3` | `0,26×` |
| `168` | `0,30` | `52` | `44,7` | `262,3` | **`0,17×`** |
| `168` | `0,15` | `24` | `22,2` | `181,9` | **`0,12×`** |
| `672` | `1,20` (a peça) | `672` | `1083,2` | `397,6` | **`2,72×`** |
| `672` | `0,60` | `672` | `432,0` | `723,3` | `0,60×` |
| `672` | `0,30` | `202` | `187,8` | `372,8` | `0,50×` |
| `672` | `0,15` | `90` | `77,2` | `249,8` | `0,31×` |

⭐⭐⭐ **A fita especializada ganha `2×`–`8×` em toda região que o traçado usa.** O BVH só ganha na
**peça inteira** com `672` arestas — que é exactamente o regime que a especialização da W56
**eliminou**.

### §87.2 — ⭐⭐⭐ O mecanismo, e ele é geral

O custo do BVH é **quase plano** (`180`–`720 ns`, ele desce uma árvore por ponto); o da fita **escala
com as arestas guardadas** (`22`–`1 083`). ⇒ eles cruzam-se por volta de **`150` arestas guardadas**,
e a especialização guarda **`24`–`202`**.

⭐ *Uma estrutura de aceleração amortiza-se sobre o trabalho que ela poda. Quando outro mecanismo já
podou esse trabalho, ela fica a pagar o custo da própria descida por nada.* Aos `24` arestas a fita
faz vinte e quatro `min` numa faixa SIMD (`22 ns`); o BVH desce uma árvore (`182 ns`).

⚠️ **E a nota da W56 não estava errada — ela media outro regime.** O `40 ns` contra `155` é sobre o
perfil **INTEIRO**, e a wave que a escreveu foi a que tirou esse regime do caminho. *Uma recusa pode
ser superada pelo número da própria wave que a registou, e quem a for reler tem de reconferir de que
regime ele era.*

### §87.3 — ⚠️ E a sonda corrigiu-se DUAS vezes antes de dizer isto

1. ⛔ **Ela media a função errada.** O `sd_batch_culled` faz uma varredura **LINEAR** sobre o conjunto
   cortado; quem desce a árvore é o `sd_batch`. *Comparar a errada é comparar outra coisa* — e a
   linear sai `2×`–`7×` pior que o BVH em quase todas as células.
2. ⛔ **As regiões eram CENTRADAS na peça**, e num círculo isso mantém **todas** as arestas (o `dmax`
   do corte é o mesmo para todas). ⇒ a 1.ª corrida comparava uma fita cheia com uma consulta cheia,
   nas oito células. *Uma sonda que não reproduz o fenómeno mede outra coisa.*

## §88 — W86: onde o quadro está — e uma atribuição minha que caiu (27/08)

### §88.1 — O ciclo inteiro, com o produto de hoje

`measure_where_the_frame_stands_after_all_of_it` (`load ~4`, a cache aquecida com um arrasto):

| contorno autoral | | movimento `640×360` | assentar 1 `640×360` | assentar 2 `1280×720` |
|---|---|---:|---:|---:|
| `168` (omissão) | `mov 168 · ass 168` | **`24,1 ms`** | `28,0` | `77,8` |
| `940` (`Resolution` alto) | `mov 157 · ass 314` | **`24,2 ms`** | `58,0` | `136,3` |

⭐⭐⭐ **O quadro de movimento passou a ser o MESMO nos dois** — `24 ms`, independente do que o
artista pôs no `Resolution`. E o assentar de uma peça pesada custa hoje `58` onde o contorno autoral
custaria `~3×` isso.

⚠️ **E ele continua `1,45×` acima do orçamento** (`24,1` contra `16,7`).

### §88.2 — ⛔⛔ E a causa que eu tinha nomeado para a má escala **NÃO era o JIT**

A §82.8.1 mediu `36 %` de eficiência a 32 threads e a §82.9 nomeou a causa com um controlo: *o JIT
satura às 16*. ⭐ A W82 e a W83 tiraram quase toda a compilação de um quadro (de `226` fitas para
`5`–`15`) ⇒ a nota tinha de ser reconferida.

`measure_whether_the_cached_frame_scales_better`, o **mesmo** quadro com e sem cache, no mesmo
processo:

| threads | 1 | 2 | 4 | 8 | 16 | 32 |
|---|---:|---:|---:|---:|---:|---:|
| **sem cache** | `100 %` | `96 %` | `89 %` | `80 %` | `57 %` | **`31 %`** |
| **com cache** | `100 %` | `94 %` | `87 %` | `76 %` | `56 %` | **`30 %`** |

⛔⛔ **As duas curvas são a MESMA.** A cache desloca o nível inteiro para baixo (`308 → 218 ms` em
série, `31,2 → 23,1` a 32 threads) e **não muda a forma**. ⇒ *o que estraga o escalamento estraga-o
igualmente com e sem compilação*, e a atribuição da §82.8.2 está **refutada**.

⚠️ **A §82.9 não estava errada** — ela mediu, num controlo isolado, que o JIT satura às 16 threads. O
que ela não mediu é se ele é o **termo que manda** dentro de um quadro, e o teste que o decide é este:
tirá-lo e ver a curva. *Um mecanismo confirmado em isolamento não é, por isso, a causa do que se via.*

⚠️ **Os ABSOLUTOS desta corrida valem pouco** (`load 18`): o que ela decide é a **comparação entre as
duas curvas**, que correu no mesmo processo e sob a mesma carga.

⏳ **Fica aberto: a causa a sério.** Os dois candidatos nomeados são a **largura de banda de memória**
(a marcha percorre buffers de `f32` grandes, e o joelho está entre 8 e 16 threads, que são núcleos
físicos os dois) e o **desequilíbrio de ladrilhos** (a §82.5 mediu que o mais caro vale `1,52×` a
fatia perfeita). ⛔ *Nenhum dos dois está medido, e o erro de hoje foi precisamente dar por causa o
mecanismo que estava à mão.*

## §89 — W87: a perda é a DECOMPOSIÇÃO — e a minha cura para ela foi medida e recusada (27/08)

A §88.2 refutou o JIT como causa da má escala e deixou dois candidatos, que mandam em waves opostas:
**a decomposição** (o trabalho existe e está mal repartido ⇒ repartir melhor) ou **um recurso
partilhado** (a máquina não tem mais para dar ⇒ fazer menos trabalho).

### §89.1 — ⭐⭐⭐ O discriminador, e ele é clássico

Correr `T` quadros **independentes** — um por thread, cada um **serial** — contra **um** quadro
repartido pelas mesmas `T` threads. O trabalho total é o mesmo e a decomposição desaparece.

`measure_whether_the_loss_is_the_decomposition_or_a_shared_resource` (`640×360`, sem cache nos dois
braços de propósito — uma cache partilhada seria uma terceira variável):

| threads | 1 quadro repartido | `T` independentes | ganho repartido | **ganho independentes** |
|---:|---:|---:|---:|---:|
| 2 | `156,1` | `149,3` | `1,94×` | `2,03×` |
| 4 | `79,9` | `77,3` | `3,79×` | `3,91×` |
| 16 | `32,3` | `24,6` | `9,37×` | **`12,31×`** |
| 32 | `26,0` | `17,8` | `11,65×` | **`16,99×`** |

⭐⭐⭐ **A decomposição custa `1,47×`** (`16,99 / 11,65`), e o resto (`17,0×` de `32`, isto é `53 %`)
é o chão honesto da máquina — SMT e largura de banda.

⭐⭐ **E a previsão por CONTAGEM já estava escrita:** a §82.5 mediu que o ladrilho mais caro vale
`1,52×` a fatia perfeitamente equilibrada. `1,47` contra `1,52`. ⚠️ *O número certo estava na página
ao lado, e a §82.8.2 deu a culpa ao JIT porque era o mecanismo que estava à mão.*

### §89.2 — ⛔⛔ RECUSA MEDIDA: pôr os ladrilhos CAROS PRIMEIRO não cura

Se o mal é um ladrilho gordo a sair tarde da fila, a cura óbvia é ordená-los por custo descendente
(LPT). Implementado, e o A/B correu no **mesmo processo**:

| threads | ordem natural | caros primeiro |
|---:|---:|---:|
| 4 | `81,87` | `81,34` |
| 16 | `30,78` | **`32,30`** |
| 32 | `25,62` | **`26,66`** |

⛔ **Neutro a pior.** ⚠️ **A régua que usei está provavelmente ANTI-correlacionada com o custo:** a
profundidade da peça sob o ladrilho (`t_hi − t_lo`) é máxima no **meio** da peça, onde os raios
**acertam cedo**; o ladrilho caro é o da **silhueta**, onde eles passam rasantes e dão dezenas de
passos. *Um escalonamento por um palpite de custo escalona pelo palpite.*

⭐ Revertido. O que fica no lugar é o comentário com a tabela, para a próxima tentativa começar por
onde esta parou.

⏳ **Aberto:** uma régua de custo que sirva — a candidata é **o custo medido do quadro anterior**, que
o `TILE_MAX` já sabe recolher — e, antes dela, saber se é mesmo a **ordem** que falta ou a
**granularidade** (⛔ o tamanho do ladrilho já foi varrido e fechado, §82.10).

## §90 — W88: ⭐⭐⭐ O QUADRO DE MOVIMENTO ENTROU NO ORÇAMENTO (27/08)

> ⛔⛔ **CORRIGIDO PELA §91.1 (27/08): o `14,2 ms` desta secção foi medido com a CÂMERA PARADA.**
> A `measure_where_the_frame_stands_after_all_of_it` aquece com um arrasto e depois cronometra `5`
> traçados de `Orbit::default()` **repetidos** — o único caso em que a cache de fitas acerta `100 %`.
> Num arrasto a sério o quadro de movimento custa `10`–`27 ms`, mediana `~12`. ⇒ *este número foi
> reportado ao Enio e estava optimista;* o que a W88 de facto comprou (o `TILE`, o tecto derivado) é
> real e mede-se na mediana, não no `14,2`. **Uma bancada que repete a mesma pose não mede
> movimento.**


### §90.1 — Primeiro o oráculo: *simule antes de construir*

A §89 mediu que a decomposição custa `1,47×` e que ordenar por profundidade não cura. ⚠️ **A pergunta
que ficou não era «que estimador de custo usar»: era se a ORDEM é sequer o mecanismo.**

`measure_what_a_perfect_tile_schedule_would_buy` grava o custo **verdadeiro** de cada ladrilho
(serial) e simula o escalonamento — aritmética pura, sem uma linha de produto:

| threads | ordem natural / ideal | **LPT / ideal** |
|---:|---:|---:|
| 8 | `1,13×` | **`1,00×`** |
| 16 | `1,40×` | **`1,02×`** |
| 32 | `1,76×` | `1,52×` |

⭐ **A ordem É o mecanismo** — com o custo verdadeiro ela chega ao ideal a 8 e 16 threads. ⚠️ E a 32
sobra um piso de `1,52×` que **nenhuma ordem passa**: o pior ladrilho vale `4,74 %` do quadro e a
fatia ideal é `3,1 %`. *Uma ordem não parte um ladrilho.* ⇒ o que faltava era ladrilhos **menores**.

### §90.2 — ⛔⛔ E o `TILE` estava a ser escolhido pelo TECTO DA MINHA CACHE

A §82.10 fechou o `TILE` em `64`, e a reconferência com a cache ligada ainda deu `64`. ⚠️ **Porque o
tecto FIXO da cache (`CAPACITY = 2048` fitas) estrangulava exactamente os tamanhos pequenos:**

| com o tecto fixo | ladrilho 24 | 32 | 48 | 64 |
|---|---:|---:|---:|---:|
| `640×360` | `43,3` | `43,0` | `50,0` | `58,8` |
| **`1600×900`** | **`859,2`** | **`677,8`** | `60,8` | `65,1` |

A `1600×900` um ladrilho de `32` pede `~5 800` regiões por quadro contra um tecto de `2 048`: **a
cache despejava metade a cada quadro e recompilava tudo.** ⇒ *o «óptimo» que a varredura devolvia era
o maior ladrilho que ainda cabia no meu tecto.*

⭐⭐⭐ **Um limite que não diz de que recurso é acaba a escolher a constante do lado** (`CLAUDE.md §0`:
*nunca deixe o fallback definir o produto*). O tecto passou a ser **derivado do que o quadro pede** —
`FRAMES_KEPT = 3` vezes as regiões de um quadro (o corrente, o anterior de onde vêm os acertos, e o
do outro documento que o preview alterna), com um tecto absoluto que **nomeia o recurso**
(`CAPACITY_MAX = 16 384` fitas de memória executável).

### §90.3 — ⭐ A varredura, com o tecto derivado — e a resposta inverteu-se

| | 16 | **24** | 32 | 48 | 64 | 96 |
|---|---:|---:|---:|---:|---:|---:|
| `640×360` 168 ar | `12,5` | **`13,3`** | `13,8` | `15,8` | `19,1` | `32,8` |
| `640×360` 672 ar | `38,2` | **`41,1`** | `43,8` | `53,3` | `62,1` | `111,5` |
| `1600×900` 168 ar | `62,6` | **`60,8`** | `58,2` | `61,8` | `66,5` | `76,7` |

⭐ **`TILE = 24` ship**: `1,44×` contra o `64` no caso do preview e `1,51×` na peça pesada. O `16`
ganha `6 %` a `640×360`, **perde** a `1600×900` e guarda o **dobro** das fitas.

### §90.4 — ⭐⭐⭐ E o quadro de movimento entrou no orçamento

`measure_where_the_frame_stands_after_all_of_it`, ⚠️ medido com a máquina a **`load 25,8`** (ou seja,
o número real é **melhor** que este):

| | §88.1 (antes) | **agora** | do orçamento de `16,7` |
|---|---:|---:|---:|
| **movimento**, peça de omissão | `24,1` | **`14,2`** | **`0,85`** |
| **movimento**, `Resolution` alto | `24,2` | **`13,8`** | **`0,83`** |
| assentar 1, omissão | `28,0` | `20,6` | `1,24` |
| assentar 1, pesada | `58,0` | `33,9` | `2,03` |
| assentar 2, omissão | `77,8` | `55,2` | `3,30` |
| assentar 2, pesada | `136,3` | `94,0` | `5,63` |

⭐⭐⭐ **`60 Hz` no quadro que a mão sente**, e independente do `Resolution` — que era o item nº 1 da
lista desde a §70.

⏳ **Fica aberto:** o `SLABS` foi escolhido com o ladrilho de `64` e a forma da região mudou — ele
pede a mesma reconferência. E o escalonamento por custo (§90.1) continua por construir: com `TILE=24`
há `27×15×6 ≈ 2 430` ladrilhos-fatia e o pior pesa muito menos, então o piso de `1,52×` encolheu
sozinho — *é preciso re-medir o que ele ainda vale antes de o construir.*

## §91 — W89: ⭐⭐⭐ A TRAVADINHA TINHA NOME — a cache despejava 1 700 fitas debaixo do cadeado (27/08)

> **Report do Enio (27/08), depois do smoke da W88:** *«de tempos em tempos dá pequenas travadinhas»*.

### §91.1 — ⚠️ Porque nenhuma sonda desta linha a podia ver (e uma delas mentia)

Todas as bancadas do módulo medem **medianas de um tipo de quadro**, com a cache quente daquele
tipo. Três cegueiras compostas:

1. **A travadinha é cauda, não centro** — uma mediana de 5 nunca a contém.
2. **Ela precisa de `~24 quadros de arrasto contínuo`** para a cache chegar ao tecto. As bancadas
   mediam 5 a 15 traçados: *o fenómeno estava sempre um quadro depois do fim da medição*.
3. ⛔⛔ **A `measure_where_the_frame_stands_after_all_of_it` (§90) mediu o quadro de «movimento»
   com a CÂMERA PARADA** — ela aquece com um arrasto e depois cronometra `5` traçados de
   `Orbit::default()` repetidos, que é o único caso em que a cache acerta `100 %`. ⇒ **o `14,2 ms`
   que a §90 anuncia, e que foi reportado ao Enio, é o custo de re-traçar a MESMA pose.** Num
   arrasto a sério o quadro custa `10`–`27 ms`, com mediana `~12`.

### §91.2 — A medição que nomeia o defeito

`crates/ph2d-field-render/tests/the_eviction_storm.rs` — arrasto de `2°/quadro` a `426×240`, peça de
omissão, `90` quadros, cache nova:

| quadro | 20 | 21 | 22 | **23** | 24 |
|---|---:|---:|---:|---:|---:|
| ms | `12,3` | `14,2` | `11,6` | **`274,8`** | `12,1` |
| despejos | `0` | `0` | `0` | **`1` (1 738 fitas)** | `0` |

E o regime confirma que **não é um acidente de arranque** — de `12` em `12` quadros:

| regime (quadros 40+) | mediana | média | **MÁXIMO** | despejo dentro do cadeado |
|---|---:|---:|---:|---:|
| antes | `11,5` | `31,8` | **`364,6`** | `269`–`353 ms` |

⇒ **a cada `~0,25 s` de arrasto contínuo a imagem congela um terço de segundo.** `97 %` desse tempo
está **dentro do cadeado de ESCRITA**, com as outras 31 threads do quadro à porta.

### §91.3 — ⭐⭐⭐ O mecanismo: 94 % do preço de despejar uma fita é a ÁRVORE

`the_price_of_freeing_a_tape.rs` mede a libertação isolada de `1 700` fitas:

| o que se liberta | total | por fita |
|---|---:|---:|
| só as **árvores** | `179,8 ms` | `105,8 µs` |
| as fitas inteiras (máquina parada) | `191,7 ms` | `112,8 µs` |
| as fitas inteiras (31 threads vivas) | `302,5 ms` | `177,9 µs` |

⛔ **A hipótese da contenção foi REFUTADA**: com a máquina parada o preço já lá está (a contenção
acrescenta `1,6×`, não a ordem de grandeza). ⇒ tirar o despejo do cadeado **não** era a cura.

⭐⭐⭐ **A `RegionTape` guardava `{ tree, tape }`, e o único leitor da árvore é o `Hybrid::fork` da
rota de bissecção** (`PH2D_FIELD_SHARE_TAPE=0`). Na rota do produto ela é **lastro** — e o lastro era
a travadinha. *Guardar «para o caso de» tem preço, e aqui ele era um terço de segundo de imagem
congelada.*

### §91.4 — O A/B, e a armadilha que NÃO mordeu

| regime (quadros 40+) | antes | depois | |
|---|---:|---:|---|
| mediana | `11,5` | `12,2` | — |
| média | `31,8` | **`13,2`** | `2,4×` |
| **MÁXIMO** | `364,6` | **`21,7`** | **`17×`** |
| despejo no cadeado | `269`–`353 ms` | **`0,1 ms`** | `~3 000×` |

⚠️ **A armadilha nomeada era a RELOCAÇÃO** (*«uma cura pode mudar o sítio de uma espera em vez de a
cortar»*, §85 da W73): a árvore continua a nascer em cada compilação, e largá-la ali podia pôr
`100 × 106 µs = 10,6 ms` em **cada** quadro. **Não aconteceu — a mediana não se mexeu** (`11,5 →
12,2`, dentro do ruído a `load 15`). A árvore especializada é um DAG de `Arc` partilhado com a do
documento: largada no instante em que nasce, o desmonte é quase todo decremento de contador; largada
horas depois, ela paga o estado do alocador. *A medição isolada apontou o suspeito certo e errou o
preço; quem decidiu foi a série do produto.*

### §91.5 — ⛔⛔ Três recusas MEDIDAS (não as reconstrua)

1. **O despejo em FATIA de `1/8`** (em vez de metade) é **pior nos três números**: mediana
   `11,5 → 13,9`, média `12,8 → 16,3`, máximo `21,2 → **61,7**`. Guardar `7/8` mantém a população
   colada ao tecto (`3 158` contra `2 234` fitas) e o `TapeCache::get` é uma **varredura linear** que
   paga esse tamanho em cada uma das ~600 regiões do quadro. ⇒ *o que uma cache guarda a mais não é
   de graça: alguém a percorre.*
2. **A hesitação da mão NÃO era o mecanismo.** A 1.ª hipótese desta wave era o degrau do meio do
   assentar não ser cancelável; a sonda mediu as duas leis lado a lado e deu **o mesmo número**
   (§91.7 mostra o que ele de facto vale, que é outra coisa e muito menor).
3. **A contenção nas 31 threads** — ver §91.3.

### §91.6 — ⭐ A dispersão de COORTES, e a sonda que mediu outra coisa

As regiões compiladas no mesmo quadro têm a mesma folga em todas as direcções ⇒ saem da caixa no
**mesmo quadro**, e o lote auto-sustenta-se. A cura desloca a **fase** (o centro da caixa dentro da
folga que a inflação já pagou): o volume não muda, logo o preço por amostra não muda, e as coortes
dispersam-se — `tape_cache::inflate_phased`.

⚠️⚠️ **A 1.ª varredura desta constante mediu OUTRA COISA**: correu antes de a árvore sair da fita, e
as cinco amplitudes deram `~290 ms` de máximo porque **o máximo era a tempestade**. *Uma cura medida
numa fixtura onde o defeito dominante é outro lê-se como inútil.* Remedida no regime (duas corridas):

| fase | mediana | média | **máximo** | compilações |
|---|---:|---:|---:|---:|
| `0,0` | `16,3` · `12,8` | `16,6` · `13,8` | `31,0` · `24,6` | `5 177` · `5 189` |
| **`0,3`** | **`12,8` · `12,5`** | **`13,7` · `12,8`** | **`21,3` · `20,0`** | `5 181` · `5 204` |
| `0,5` | `13,2` · `11,9` | `14,4` · `13,0` | `23,0` · `21,9` | `5 266` · `5 305` |
| `0,8` | `14,9` · `12,9` | `15,8` · `13,4` | `28,0` · `24,1` | `5 743` · `5 744` |

⭐ A `0,3` compila **o mesmo** que a `0,0` (`+0,3 %`): a dispersão é de graça. ⛔ A `0,8` compila
`+11 %` — uma região com pouca folga do lado da deriva expira quase todo o quadro, e a convexidade de
`1/vida` cobra-o. *Há uma amplitude óptima e ela não é a maior.* Ganho honesto: `4`–`10 ms` no
máximo, não uma ordem de grandeza.

### §91.7 — ⭐⭐ E a lei do cancelamento perguntava ao TAMANHO desde a W73

O `cancels_the_inflight` identificava *refinamento* por `inflight == cheio` — o que **era** a
definição até a W73 partir o assentar em dois degraus e pôr o primeiro no tamanho **grosso**. A
partir daí duas espécies partilhavam a grandeza, e o degrau do meio **nunca era abandonado**. O
`InFlight` já dizia em comentário que sabia a espécie; hoje ela viaja de facto (`refinement: bool`).

*Um predicado que identifica uma ESPÉCIE por uma GRANDEZA fica errado no dia em que duas espécies
partilham a grandeza.*

⚠️ **E a régua desta medição corrigiu-se duas vezes.** A 1.ª contava o **boot** (o primeiro traçado é
sempre cheio por lei, `183 ms`) e ele dominava todas as linhas ⇒ as duas leis imprimiam o mesmo. A
2.ª contava o atraso em **milissegundos**, e durante uma pausa a imagem «velha» mostra uma câmera que
**não se mexeu** — ela está certa, e o tempo dela acusava de defeito o gesto do artista. A régua é o
**erro angular** entre a pose na tela e a da mão:

| pausa da mão | pelo tamanho | **pela espécie** |
|---|---:|---:|
| `0 ms` | `1,50°` | `1,50°` |
| **`17 ms`** | **`2,97°`** | **`1,50°`** |
| `34`–`136 ms` | `1,50°` | `1,50°` |

⇒ o defeito vive na hesitação do tamanho de **um quadro** — a que uma mão faz sem dar por ela — e ali
**dobra** o atraso.

### §91.8 — ⭐ O `FRAMES_KEPT = 3` era derivado; agora é MEDIDO

Ele foi escrito por raciocínio e nunca varrido, e os dois números que o sustentavam mudaram nesta
mesma jornada. `how_many_frames_to_keep.rs` (regime, `load` alto ⇒ só se lê o que é grande):

| quadros guardados | mediana | média | máximo | fitas na cache |
|---|---:|---:|---:|---:|
| `1` | `15,3` | `15,8` | `27,2` | `922` |
| `2` | `13,3` | `13,8` | `22,8` | `1 624` |
| **`3`** | **`12,1`** | `13,4` | `22,1` | `2 585` |
| `4` | `12,0` | `13,0` | `21,1` | `3 653` |
| `6` | `11,7` | `12,9` | `23,4` | `6 390` |

⇒ **o joelho está no `3`**: `1` e `2` são piores, `4` e `6` compram `≤0,4 ms` por `1,4`–`2,5×` a
memória e não melhoram o máximo. *Medir um tecto pode confirmá-lo, e isso é um resultado.*

### §91.9 — Gates e provas de mutação

| gate | onde | mutação que ele mata |
|---|---|---|
| `a_cached_tape_carries_no_tree_on_the_product_path` | `ph2d-field-eval` | `tree: Some(tree)` sempre → ✗ |
| `the_phased_box_still_contains_its_region` | `ph2d-field-render` | `PHASE = 1.5` → ✗ |
| `the_eviction_drops_half_and_the_cache_never_grows_past_its_ceiling` | `ph2d-field-render` | `k = len/1000` (o despejo não despeja) → ✗ |

⚠️ **Os três são de ESTRUTURA ou de CONTAGEM, de propósito** — o defeito que apanham é de população,
e um gate de relógio sobre ele reprovaria sob fan-out sem nada ter mudado.

### §91.10 — ⚠️ Um membro NOVO da família de flakes do `CLAUDE.md §5.0`

`an_abandoned_march_returns_nothing_and_returns_fast` (`ph2d-field-render`) compara **dois relógios**
(`cut_ms < full_ms · 0,5`). A `load 10`–`15` ele mudou de resposta **entre corridas do mesmo
binário**, e sozinho deu verde `3` de `3`. O diff desta wave não toca o caminho dele (ali a cache nem
existe: `cache = None`).

## §92 — W90: ⭐⭐⭐ O CANVAS DIVIDE-SE EM QUATRO VISTAS (27/08)

> O plano fecha o canvas de primeira classe como *"modo de viewport próprio, com **cabeçalho e
> divisão**"* ([`03_plano_implicito.md`](../3DModeling/03_plano_implicito.md)), e o Enio levantou-o
> em 19/08 (*"melhor fazer como foi feito o Sculpt ou já criar o canvas 3D estilo Blender"*). A
> **divisão** é a metade que não existia; o resto do canvas de primeira classe (as seis vistas
> nomeadas, o gizmo de navegação, a viagem entre vistas) fechou nas W47–W52.

### §92.1 — W90a: o estado de UMA vista sai do `Smoke`

A fronteira já estava escrita, campo a campo, nos doc-comments — e a W43 já lhe tinha dado nome do
lado da vista (`View::of`). Nove campos passam para uma `Viewport`: `cam`, `frame`, `inflight`,
`since`, `requested`, `last_trace_ms`, `measured`, `area`, `manual`. Fica no `Smoke` o que é do
**documento** ou do **gesto**: a peça, o gizmo, o arrasto, o laço, a isolação, o verbo.

⭐ `Smoke::vp()`/`vp_mut()` prendem o índice ao alcance **na porta**. *Um `Option` ali obrigaria os
~30 sítios que perguntam pela câmera a responder «e se não houver vista nenhuma?», que é uma pergunta
que o produto não tem.* O preço é uma invariante — a lista nunca é vazia — e ela tem censo.

Neutro e medido: `3 834` passaram (os `3 833` de antes + o censo), `0` falharam.

### §92.2 — A porta do layout, e a lição que esta casa pagou com um bug de produto

`field3d_layout::rects` devolve os retângulos em pixels **inteiros**, e eles ladrilham a área
**exactamente**. ⚠️ Não é preciosismo: o `CenterSplit::scene_viewport` (o divisor cena/grafo) devolvia
`h · t` fraccionário, o passe de sprites recebia `422,4` e o `set_scissor_rect` ao lado `422` — a
diferença de `0,095 %` era invisível parada e **um movimento** num pan (report do Enio, 25/08).
⇒ *um valor que é pixels não pode sair fraccionário da porta que o define.* Aqui as **arestas** é que
são arredondadas (nunca as larguras), que é o que faz cada aresta interior ser o **mesmo número** para
os dois vizinhos.

⚠️ **O divisor não é uma folga na geometria** — insetar os retângulos faria os quatro traçados perder
pixels e o fundo aparecer por baixo. A linha é pintada por cima, no chrome.

### §92.3 — As decisões, e de onde cada uma veio

| decisão | de onde |
|---|---|
| `Top` em cima-esq · `Right` em cima-dir · `Front` em baixo-esq · **a perspectiva do artista em baixo-dir** | a disposição do Blender — e é onde a mão dele já está |
| as três nomeadas nascem **ortográficas** | uma vista nomeada existe para **medir**, e a perspectiva estraga exactamente isso |
| …e nascem `manual: true` | o `manual` é *«o prato já foi tocado»*; um `Top` que girasse sozinho deixava de ser o Top no quadro seguinte. ⭐ *A lei que já existia responde à pergunta nova sem um campo a mais* |
| o botão do rato escolhe o viewport **activo**, e o `Move`/`Up` não repetem a pergunta | a lei de captura que todo gizmo deste shell segue: um arrasto que mudasse de câmera ao atravessar a costura orbitaria duas peças com um gesto só |
| ao FECHAR, fica a vista **activa** | o artista fecha a olhar para o quadrante que lhe interessa; ficar com outro desfaz-lhe o gesto |
| o gizmo e o navball só no activo | as alças são a projecção do gesto; pintá-las nas quatro convidaria a agarrar numa vista e arrastar noutra |
| a moldura do activo é **obrigatória** | com quatro vistas iguais, *«qual delas o teclado comanda?»* passa a ser uma pergunta com resposta — e *um estado que muda o que a tecla seguinte faz e não se vê é a definição de uma interface que mente* |

### §92.4 — ⚠️ A cache de fitas passa a ser DO VIEWPORT, e isto é a W89 a reabrir pela porta do lado

O tecto da cache é **derivado do que um quadro pede** (§91.8). Com quatro viewports a chamar
`TapeCache::begin`, o último a passar dimensionava a cache para **um quarto** da tela — a debulha que
a §91 acabou de curar, reaberta por uma feature que nada tem a ver com ela. ⇒ a cache muda de dono e
passa a viver na `Viewport`.

⭐ **E o total não sobe:** quatro viewports de um quarto da área pedem, somados, as regiões de uma
tela. *O que muda é quem contabiliza.*

⭐⭐ **E o custo em regime é ~UM viewport a mexer:** as três vistas nomeadas são **estáticas** — elas
só re-traçam quando o documento muda. Orbitar a perspectiva não lhes toca.

### §92.5 — ⭐⭐ Dois gates recusaram esta wave, e os dois tinham razão

1. **`every_camera_chip_moves_the_camera`** reprovou o chip novo. A régua daquela fileira é *«o chip
   mexe na câmera»* e o meu muda a **divisão** — quantas câmeras há. ⛔ A cura **não** foi afrouxar a
   lei: foi dar-lhe a régua da espécie (a contagem de viewports abre em `4` e fecha em `1`, e o chip
   **acende**). *Uma lei de alcançabilidade tem uma régua por espécie de gesto* — a própria nota da
   W47 já o dizia, e o gate obrigou-me a lê-la.
2. **O meu censo `nothing_can_empty_the_viewport_list` tinha um buraco:** listava `clear`/`pop`/
   `remove`/`drain`/`truncate` e **não** a ATRIBUIÇÃO, que é o modo mais óbvio de todos — e é
   exactamente o que a divisão veio a fazer (`smoke.vps = novos`). *Um censo por texto só apanha o que
   alguém se lembrou de escrever, e o que se esquece é o caso normal.* Hoje a lista inclui `vps = ` e
   o `ensure_viewports` é o **único** sítio autorizado, com o motivo escrito ao lado. ⚠️ A primeira
   corrida do censo apanhou também a **própria lista de verbos** que ele define — o modo de falha
   clássico de um censo por texto, e a fronteira certa não era *«este ficheiro»* mas *«código que
   corre no app»*.

### §92.6 — Gates

| gate | o que prende |
|---|---|
| `the_four_pieces_tile_the_area_exactly` | pixels inteiros e soma das áreas = área, em origens fraccionárias e tamanhos ímpares |
| `a_point_on_the_seam_belongs_to_exactly_one_viewport` | o teste semi-aberto: a costura tem **um** dono |
| `the_count_and_the_named_views_agree` | a contagem e a disposição saem da mesma fonte |
| `each_viewport_stores_the_rect_the_layout_gave_it` | ⭐ **a costura**: desenha de verdade e depois pergunta ao **ponteiro** — as duas travessias têm de concordar (mutação: `viewport_at` a devolver sempre `Some(0)` → ✗) |
| `nothing_can_empty_the_viewport_list` | a invariante que torna o `vp()` infalível |
| `every_camera_chip_moves_the_camera` | agora com a régua da divisão dentro |

### §92.8 — ⛔⛔ O smoke do Enio: só a vista ACTIVA ficava lisa (27/08)

> *«apenas a janela activa fica com o objecto desenhado liso, as demais ficam no modo de baixa
> resolução»*

Cada viewport decide o traçado seguinte com `next_trace`, que compara **o pedido anterior** com o
estado de agora. O passe perguntava pelo pedido do viewport **ACTIVO** e comparava-o com a câmera
**deste**:

```text
smoke.vp().requested   ←  o activo
&smoke.vps[i].cam      ←  a câmera de outro
```

⇒ para toda vista não-activa, *«a câmera mudou?»* era **sempre sim**, e ela ficava presa no quadro de
movimento (grosso, sem anti-serrilhado) **para sempre**, sem nunca subir os dois degraus do assentar.

⚠️ **A causa é mecânica e vale como aviso:** a reescrita que retargetou o passe de `smoke.vp()` para
`smoke.vps[i]` casava o padrão **numa linha**, e o `cargo fmt` tinha partido exactamente as
expressões longas em três. *Uma reescrita por padrão de LINHA é cega à mesma expressão embrulhada
pelo formatador — e o formatador embrulha precisamente as maiores.* (As três multi-linha deste
ficheiro foram corrigidas à mão **antes** do retarget, e por isso ficaram com o alvo antigo.)

⚠️⚠️ **E os cinco gates da W90 passaram todos**, porque mediam a **geometria** (os retângulos
ladrilham · a costura tem um dono · cada viewport guarda a sua área). *Uma divisão certa pode
alimentar quatro laços errados* — o defeito não estava em onde as vistas ficam, mas em **com quem
cada uma se compara**.

⇒ Gate novo, com a régua que faltava — o **estado em que cada vista PÁRA**:
`every_still_viewport_settles_not_only_the_active_one`. Ele abre a divisão, desenha até as quatro
terem quadro pronto e nenhuma ter traçado em voo, e exige que **nenhuma** esteja a pedir um quadro de
movimento com a cena quieta. Mutação (repor `smoke.vp()`): ✗ em `3,12 s` — ele nem chega a
convergir, que é o próprio sintoma.

⭐ A pergunta é feita por `Viewport::probe_resting_state`, e não abrindo os campos: *a pergunta vive
onde os dados vivem.*

### §92.9 — ⭐⭐⭐ W90c: quatro traçados juntos não ganham NADA, e a activa passa a ter prioridade

O item que a §92.7 deixou *«por medir»*. `the_price_of_four_views.rs`, máquina calma (`load 1,7`),
uma **edição** a `1280×720` (a cache de fitas é inútil de propósito: o documento mudou, nenhuma fita
antiga serve):

| | ms | |
|---|---:|---|
| uma vista, área inteira `1280×720` | `156,2` | |
| uma vista, um quarto `640×360` | `64,5` | `0,41×` do inteiro |
| **quatro ao mesmo tempo** | **`253,7`** | `1,62×` do inteiro · **`3,93×` uma sozinha** |

⭐⭐ **`3,93×` de quatro é o mesmo que somá-las:** cada traçado já satura a máquina com o `rayon`, então
correr quatro só os **fatia**. *Não há paralelismo por colher — ele já foi colhido dentro de cada um.*

⭐ E o trabalho total sobe `1,65×` para os **mesmos pixels** (`258` contra `156`): é o custo **fixo**
de um traçado — a re-amostragem do contorno, a montagem das fitas — pago quatro vezes. Ele não
encolhe com a área.

⇒ **A cura não é acelerar, é ORDENAR.** Uma vista não-activa não começa um traçado enquanto a activa
tiver um em voo: o trabalho total é o mesmo e a vista onde a mão do artista está chega em `64 ms` em
vez de `254`. *Latência percebida `3,9×` melhor, sem uma linha de algoritmo.*

⚠️ **Não há fome:** as vistas nomeadas são estáticas, então só têm trabalho quando o **documento**
muda — e nesse instante a activa também tem, e acaba primeiro.

⛔ **E o gate apanhou um defeito na minha própria guarda:** ela pergunta *«a activa tem traçado em
voo?»*, e numa ordem natural (`0..n`) as vistas que correm **antes** dela escapam-lhe no primeiro
tique — naquele instante a activa ainda não começou. *Uma prioridade que depende de quem chega
primeiro não é uma prioridade.* ⇒ a activa passa a ser **a primeira do laço**, e a ordem de pintura é
indiferente porque os retângulos não se sobrepõem.

Gate: `the_active_viewport_gets_its_image_first` — uma afirmação de **ORDEM**, nunca de relógio (do
frio, a imagem da activa aparece pelo menos um quadro antes de qualquer outra, por construção).
Mutação (desligar a guarda): ✗.

### §92.10 — ⭐⭐ W90d: cada vista diz o NOME dela, e o nome sai da câmera

A outra metade da frase do plano (*«cabeçalho e divisão»*), na forma que **não rouba pixels ao
traçado**: uma faixa reservada encolheria as quatro imagens e obrigaria a porta do layout a devolver
**dois** retângulos por vista. O rótulo mora na quina, por cima da imagem.

⚠️ **Derivado da CÂMERA, nunca do quadrante** (`field3d_views::label_key`): a vista de cima nasce no
quadrante de cima-esquerda, mas o artista pode orbitá-la — e a partir daí ela **não é** a vista de
cima. *Um rótulo preso ao sítio continuaria a dizer «Top» sobre uma vista qualquer, e nada na tela o
desmentiria.* É a lei do Blender e é a certa.

⚠️ **Chaves i18n PRÓPRIAS**, e não as dos botões: o rótulo de um botão traz o atalho de propósito
(*"Top (7)"* — é a única forma de a tecla ser descoberta por quem não sabe que ela existe), e um
`(7)` na quina da imagem seria a promessa de um controlo que ali não existe. *A mesma palavra em dois
sítios pode ter de dizer coisas diferentes.*

⚠️ **É um MOSTRADOR, não um controlo** — e isso é uma decisão, não uma omissão: trocar a vista de um
quadrante já é alcançável (clicar nele, que o torna activo, e `Numpad1/3/7` ou o botão do painel).
*Antes de construir um controlo, meça se a composição já o exprime.* ⏳ Um cabeçalho **clicável** (com
menu por vista) fica aberto, e é ele que pede a faixa reservada.

⚠️ E ele só aparece **com a divisão aberta**: com uma vista só a pergunta *«qual é qual?»* não existe,
e um rótulo permanente seria ruído sobre a peça.

Gate: `the_viewport_label_follows_the_camera_and_not_the_quadrant` — as seis nomeadas dizem nomes
**distintos**, todas as chaves **traduzem** (um rótulo que mostra a própria chave é pior que nenhum),
e uma câmera orbitada passa a *User*. Mutação (prender o rótulo): ✗.

### §92.11 — ⛔ A varredura linear da cache: MEDIDA, e não vale a complexidade (ainda)

O item que a §91.5 abriu (*«o que uma cache guarda a mais não é de graça: alguém a percorre»*) e que
nunca tinha sido medido sozinho. `where_the_frame_goes_now.rs` (⚠️ `load 20`, então leem-se **razões**
e não relógios):

| | `get` | especializar | `get` / (`get`+`spec`) | regiões/quadro |
|---|---:|---:|---:|---:|
| `426×240` (movimento) | `10,1` ms-thread | `325` | **`3,0 %`** | `604` |
| `640×360` | `76,0` | `633` | **`10,7 %`** | `1 278` |

⭐ **O custo cresce ~quadraticamente**: dobram as regiões e a varredura faz `7,5×`. É a forma
esperada — ela percorre a população **por região**, e a população é o tecto derivado, que é
proporcional às regiões (`FRAMES_KEPT × regiões`).

⛔ **A `3 %` no quadro de movimento ela não paga um índice.** A cura desenhada seria um mapa
directo `(ladrilho, fatia) → entrada` consultado antes da varredura (a mesma região pede quase a
mesma caixa todo quadro, então acertaria ~85 % em `O(1)`), com invalidação no despejo. É trabalho
real, com um gate novo, por `3 %` de um quadro que já cabe no orçamento.

⚠️ **O GATILHO está nomeado:** ela passa a valer quando as regiões de um quadro crescerem —
resolução maior, ladrilho menor, ou o assentar a `1280×720` com a divisão aberta. *Uma recusa medida
responde uma pergunta; quando a sua for outra, remeça.* O instrumento fica no sítio
([`GET_NS`](../../crates/ph2d-field-render/src/tape_cache.rs)).

### §92.12 — ⭐⭐⭐ W92: as divisórias ARRASTAM-SE

⚠️ **A minha nota da W90 dizia que isto dependia do cabeçalho clicável, e estava errada** — um
divisor precisa de uma **zona de pega** na costura, de um `t` no `Split` e da porta do layout. Nada
disso é o cabeçalho. *Uma dependência afirmada sem a desmontar é um adiamento com cara de
arquitectura.*

**As fracções vivem DENTRO da variante** (`Split::Quad { tx, ty }`), e não num campo ao lado: a
divisão *é* as duas costuras, e um `t` guardado noutro sítio seria um estado que pode discordar do
modo.

⭐ **A trava é a da CASA, lida e não re-decidida:** `CenterSplit::clamp_t` (`T_MIN = 0,25`,
`T_MAX = 0,75`, `NaN`-aware) já fixa *«cada painel guarda sempre um quarto»* para o divisor
cena/grafo. *A lei é a mesma; escrevê-la outra vez seria ter duas.*

⚠️ **As costuras da pega são lidas dos RETÂNGULOS**, nunca recalculadas do `t`: eles são arredondados
na porta, e uma segunda conta erraria por meio pixel — que é exactamente a largura de um gesto que
falha de vez em quando.

⚠️ **A faixa de pega é maior do que a linha desenhada** (`±5 px`), e é a lei de todo divisor de
janela: *a pega é uma afirmação sobre o que o DEDO alcança, não sobre o que o olho vê.* O cruzamento
agarra **as duas** costuras, como no Blender.

⚠️ **A costura GANHA de tudo, e é o único gesto que não pertence a viewport nenhum** — ela está
*entre* eles. Sem essa precedência (e antes da escolha do activo), apontar para a linha do meio
orbitaria a vista de um dos lados e o divisor seria inalcançável.

⭐⭐ **E o arrasto mede o TOTAL, nunca incrementos** — a mesma lei que o gizmo deste módulo paga com a
âncora congelada (W26). Uma soma de deltas acumula o erro de **cada** trava: quem arrasta até ao
batente e volta encontraria a costura permanentemente deslocada da mão.

⛔ **E os meus dois primeiros gates provavam a lei PURA — se o `advance` somasse incrementos eles
ficavam verdes.** *A causa nº 1 da semana perdida no Painter foi esta: os dois lados corretos e
ninguém a ligar os dois.* ⇒ o gate que fica é o da **costura**
(`the_real_gesture_moves_the_divider_and_does_not_drift`): `begin` na costura → `advance` a bater nos
dois limites → a linha volta a estar debaixo do dedo. Mutação (somar deltas): ✗.

Gates: `the_seam_is_grabbable_and_nothing_else_is` (o meio de um quadrante **não** é pega — ali o
arrasto é a órbita, que é o gesto principal do módulo) · o ladrilhamento passa a ser varrido em
**quatro** posições do divisor.

### §92.13 — ⭐ W93: a seta de redimensionar sobre a costura (report do Enio)

> *«faltou uma seta bidirecional indicadora quando o cursor está em cima da linha (vertical para a
> linha horizontal, horizontal para a linha vertical)»*

⭐ **A casa já tinha o sítio e o precedente**: `update_eyedropper_cursor` decide o cursor por
prioridade, e o divisor do grafo do Motion já lá está com exactamente esta lei — *`NsResize` ↕ para
um divisor horizontal, `EwResize` ↔ para um vertical*. A costura do canvas 3D entra na mesma cadeia.

⚠️ **A seta sai do MESMO `seam_grab` que o arrasto usa**, e a razão está escrita ao lado do divisor
do Motion: *o cursor e o gesto leem a mesma fonte, senão discordam sobre onde a faixa está.* Duas
contas dariam a seta um pixel ao lado de onde o arrasto pega, e o defeito lê-se como *«às vezes não
agarra»* — dos piores de reproduzir.

⭐ **No cruzamento é o `Move`**: ali as duas costuras vão juntas, e uma seta de um eixo só prometeria
metade do gesto.

⚠️ **A lei vive no módulo** (`field3d_layout::seam_cursor` + `field3d_viewports::divider_cursor`) e o
despacho apenas pergunta — assim ele não precisa de conhecer o layout, e a resposta cabe numa
chamada de `with_smoke`.

Gate: `the_cursor_over_a_seam_points_across_it` — a seta é **perpendicular** à linha, o cruzamento é
`Move`, o meio de um quadrante não tem seta, e ⭐ **uma varredura afirma que o cursor e o arrasto
concordam pixel a pixel** sobre haver costura. Mutação (trocar as duas setas): ✗.

### §92.14 — W94: o `SLABS` reconferido — e o óptimo MOVE-SE com o tamanho do quadro

O item que a §90.4 abriu: o `SLABS = 4` foi escolhido com o ladrilho a `64`, e ele é `24` desde a
W88 — mais a cache de fitas, que mudou o que uma região custa. *Uma varredura envelhece com o custo
que ela pesava*, e esta já tinha mudado de veredito uma vez (a original deu `2`; a W71 deu `4`).

`how_many_slabs_now.rs`, regime de um arrasto, ×3 intercalado (⚠️ `load 32` — leem-se **ordens**):

| | `2` | `3` | **`4`** (ship) | `6` | `8` |
|---|---:|---:|---:|---:|---:|
| contorno 168 · `426×240` | **`13,2`** | `13,7` | `14,2` | `16,4` | `20,3` |
| contorno 940 · `426×240` | `66,1` | **`65,5`** | `72,0` | `80,1` | `91,7` |
| contorno 168 · **`640×360`** | `35,2` | `33,0` | **`28,9`** | `28,8` | `64,6` |

⭐⭐ **As três fixturas discordam, e isso É o resultado:** ao tamanho do quadro de movimento (`426×240`)
ganham `2`–`3` por `1,08×`–`1,10×`; a `640×360` ganham `4`–`6`. ⇒ **não há um número certo — há uma
função do tamanho do quadro**, e o tamanho do quadro é escolhido pelo **orçamento**
(`field3d_preview`), logo ele muda de máquina para máquina.

⭐ **A imagem é IDÊNTICA em todas as fatias** (`0` pixels de silhueta diferentes, normal `0,00e0` nas
cinco): o `SLABS` é puramente um botão de custo, e não uma troca de qualidade. *A coluna que a
varredura original nomeia como «o que separa ficou rápido de ficou rápido e errado» está limpa.*

⛔ **NÃO se mexe no `4`, e a razão é honesta:** ele é o compromisso (nunca pior que `1,2×` em nenhuma
das três linhas), e a `load 32` eu não separo `1,2×` de ruído — a linha de `640×360` tem um máximo de
`101,92` para `6` fatias contra `40,55` para `4`, que é ruído a falar.

⚠️ **O que decidiria:** uma corrida a `load < 5` nas duas resoluções. E se a discordância se
confirmar, a resposta não é outra constante — é **derivar** o `SLABS` do tamanho que o laço do
preview escolheu, que é a única forma de ele estar certo nas duas.

### §92.15 — W95: a divisão sobrevive a fechar o painel — e a razão que a excluía era FALSA

⛔⛔ **A W90 deixou o `split` fora da [`View`] com esta razão escrita:** *«restaurar a divisão
obrigaria a restaurar as quatro câmeras, e o que a W43 promete é uma vista»*. **É falso** — três das
quatro são **DERIVADAS** (nascem da orientação que o nome promete) e a `ensure_viewports` já as
reconstrói a partir da câmera do artista, que é a única autorada e que a `View` **já guardava**.

⚠️ *Uma dependência afirmada sem a desmontar é uma feature adiada com cara de arquitectura* — e esta
é a **segunda** desta wave, depois da que dizia que o divisor arrastável precisava do cabeçalho
(§92.12). As duas custaram uma linha de código cada, depois de desmontadas.

⭐ E ela **pertence** à vista pela mesma razão que a câmera: a divisão é uma preferência de bancada.
Um artista que trabalha em quatro vistas, pega no editor vetorial e volta não quer encontrar uma.

⚠️ **A lista nasce já com a divisão lembrada** (`ensure_viewports` no fim do `boot`). Ela seria
reconciliada no primeiro desenho de qualquer forma — mas então haveria **um quadro** em que o `split`
diz «quatro» e a lista tem uma, e *um estado que só é verdade a partir do segundo quadro é um estado
que alguém vai ler no primeiro*.

⚠️ O que **não** viaja é a POSIÇÃO das costuras: o que se lembra é *«eu trabalho dividido»*, e a
posição volta ao meio. Ela é barata de repor com a mão e cara de justificar guardada — nenhum outro
campo da `View` é uma coordenada de layout.

Gate: `the_split_survives_closing_the_panel`, provado por mutação.

### §92.16 — W96: a ordenação dos ladrilhos — o estimador EXISTE, e o prémio encolheu para `1,13×` (só a 32 threads)

O item aberto desde a §89 com o número `1,47×` colado a ele. Duas medições fecham-no, **na máquina
calma** (`load 1,3`).

**1. O tecto colapsou com o ladrilho a `24`** (`measure_what_a_perfect_tile_schedule_would_buy`):

| threads | ordem natural / ideal (ladrilho **24**) | (era com **64**) |
|---|---:|---:|
| 8 | `1,01×` | `1,13×` |
| 16 | `1,03×` | `1,40×` |
| 32 | **`1,14×`** | **`1,76×`** |

O pior ladrilho vale hoje `1,07 %` do total (eram `4,74 %`). ⇒ *o `1,47×` que este documento
carregava era um número medido com outro ladrilho, e a W88 mudou-o sem que a nota o soubesse.*

**2. O estimador que faltava EXISTE, e atinge o tecto** (`stale_costs_as_an_oracle.rs`). ⛔ A minha
primeira tentativa foi recusada por medição (a profundidade da peça sob o ladrilho está
**anti-correlacionada** com o custo — o caro é o da silhueta). O que ali não existia: **o custo do
MESMO ladrilho no quadro anterior**.

| arrasto | threads | natural/ideal | **anterior/ideal** |
|---|---:|---:|---:|
| `1°` | 32 | `1,14×` | **`1,00×`** |
| `2°` | 32 | `1,13×` | **`1,00×`** |
| `4°` | 32 | `1,12×` | **`1,01×`** |

⭐ *Um estimador «velho de um quadro» escalona tão bem como o oráculo* — num arrasto a câmera anda
`~2°`, e um ladrilho caro continua caro.

⚠️ **A régua é o ESCALONAMENTO, não a correlação:** o que se paga é o *makespan*, e uma correlação
alta não promete um. Foi assim que se mediu.

⛔⛔ **RECUSA MEDIDA, com o gatilho:** o prémio é `1,13×` a **32** threads, `1,04×` a 16 e **`1,01×`
a 8** — *não existe na máquina de oito núcleos*, que é a outra máquina deste projecto. E o preço é
uma tabela de custos por ladrilho a viver **entre quadros** (mais um dono por viewport, mais a
recolha sem contenção no caminho quente). ⇒ *`1,4 ms` de um quadro de `12`, numa máquina só.*

⚠️ **O gatilho:** ela volta a valer se os ladrilhos ficarem **menos e mais gordos** (o pior volta a
pesar) ou se o orçamento apertar. O instrumento fica no sítio, e re-corre em dois comandos.

### §92.7 — ⏳ O que fica aberto

- **O CABEÇALHO** — a outra metade da frase do plano. Hoje o chrome do módulo é o painel lateral e o
  gizmo de navegação; um cabeçalho por viewport (com o nome da vista e o modo de sombreado) é a wave
  seguinte, e é ele que destrava o **divisor arrastável** (uma divisão em `N` livre pede uma pega).
- **A isolação e o verbo do gizmo são do MÓDULO, não do viewport.** No Blender a *local view* é por
  vista. Ficam globais por decisão: são sobre a **selecção** e a **ferramenta**, não sobre a câmera.
- **O custo de uma edição com a divisão aberta** — quatro traçados disparam ao mesmo tempo e cada um
  quer a máquina toda. Inerente à divisão (o Blender faz o mesmo), **por medir**.

---

## §93 — W97: ⭐⭐⭐ UM VERBO POR FORMA — a operação sai do grupo e entra em cada objeto (28/08)

> **Pedido do Enio, 2026-08-28:** *"precisamos investigar um modo mais intuitivo e fácil de combinar
> as formas. Atualmente quando se cria uma nova operação a hierarquia fica mais confusa criando
> vários parentescos. Minha idéia é colocar a operação dentro de cada objeto. […] A partir do segundo
> objeto aparecem os modos booleanos para aquele objeto que será aplicado ao resultante das operações
> anteriores. […] as operações booleanas continuam obedecendo a ordem da hierarquia."*

### §93.1 — A lei, e por que ela já era metade da casa

> ⭐⭐⭐ **As formas dobram na ORDEM da hierarquia, e cada uma traz o verbo com que se junta ao
> resultado das anteriores.** `((c₀ ⊕₁ c₁) ⊕₂ c₂) …`, onde `⊕ᵢ` é o verbo de `cᵢ` — ou o **do pai**,
> quando `cᵢ` não trouxe nenhum.

⚠️ **Isto não é um desenho novo: é o desenho que o VETORIAL desta casa já shipou em 2026-08-22**
([`27_um_verbo_por_forma.md`](../Vector%20Module/27_um_verbo_por_forma.md)), a partir do mesmo pedido
do mesmo dono. O valor maior desta wave não é a feature — é as **duas metades do app passarem a
falar a mesma língua**, com o mesmo vocabulário de selos.

⭐ **E foi barata pela mesma razão que lá:** medido antes de escrever uma linha, **os dois
avaliadores já eram uma dobra à esquerda** — `combine_trees` faz `acc = trees[0]; for rhs in
&trees[1..] { acc = op(acc, rhs, b) }` e o `Plan::Combine` faz o mesmo em números. O que estava fixo
era **só o verbo**.

| | antes | agora |
|---|---|---|
| onde vive a operação | no **pai** (`NodeKind::Combine { op }`) | continua lá, como **padrão**; cada filho pode trazer o seu |
| onde vive o raio da junção | no **mesmo** `op` do pai — **um** para todos os filhos | viaja **dentro do verbo**, logo um por forma |
| dois furos com raios diferentes | **dois grupos aninhados** (a queixa do Enio) | dois verbos, zero parentescos |

### §93.2 — ⭐ As duas idéias do pedido são UMA, e a segunda é a razão da primeira

O Enio pediu duas coisas (a operação por objeto · o fillet por objeto). Medido: **a segunda só é
exprimível depois da primeira.**

Uma mistura pertence a uma **JUNÇÃO**, não a um objeto. Sob a árvore N-ária um `Combine` de 3 filhos
tem **duas** junções e **uma** `Blend` — «um raio por objeto» não tem onde existir. A dobra dá a cada
forma exactamente **uma** junção (a que ela faz com o acumulado) ⇒ *é a dobra que torna «por objeto»
e «por junção» a mesma coisa.*

⇒ E por isso o `Op` (que **carrega** o `Blend`) é o que viaja no filho: o raio veio junto, de graça, e
o `each_shape_carries_the_radius_of_its_own_joint` mede-o.

### §93.3 — ⚠️ Ausência é HERANÇA, não «sem verbo»

`Node::verb` é `Option<Op>` e o componente [`FieldVerb`] é **opcional**. As duas leituras coincidem de
propósito:

| no mundo | no documento | quer dizer |
|---|---|---|
| sem `FieldVerb` | `Node::verb == None` | *«use o do meu pai»* |
| com `FieldVerb` | `Node::verb == Some(op)` | *«eu dobro assim»* |

Isto compra duas coisas, e as duas pesam para o mesmo lado:

- **toda peça anterior avalia igual** — ninguém se pronunciou (gate
  `silence_is_the_boolean_of_always`, com o controlo a ser a árvore **aninhada de dois filhos**, que
  é como a mesma peça se exprimia antes e **não usa o campo novo**);
- **o seletor do pai não morre**: deixa de ser *a* operação e passa a ser o **padrão** de quem não se
  pronunciou. Sem esta escolha ele ficaria inerte, que é o defeito *«parâmetro que não muda nada»*.

### §93.4 — ⚠️ Onde eu discordei do pedido: o primeiro objeto GUARDA o verbo dele

O pedido diz *«o primeiro objeto é a base e logo não deve ter modos booleanos»*. O comportamento é
esse; a **representação** não.

⛔ **Não é «o acumulado começa vazio»**, que seria a outra forma de o dizer: com ela, uma subtração no
topo apagaria a peça inteira (`∅ − a = ∅`) — uma reordenação a destruir o modelo em silêncio.
⇒ O primeiro filho **semeia** o acumulado e o verbo dele **nunca é perguntado**
(`the_first_shapes_verb_is_never_asked`).

⭐ E ele é **guardado na mesma**: *reordenar não pode destruir a escolha de quem passou pelo topo.*
Arrastar o terceiro para cima torna-o base sem nada a consertar, e arrastá-lo de volta devolve o
verbo que ele tinha.

⚠️ **E a BASE é a primeira que CONTRIBUI, não a primeira da lista** — esconder a primeira **promove a
segunda**, porque é isso que o cozimento faz. A resposta sai do `contributes`, que é a **mesma**
função que o `emit` usa: uma segunda cópia da regra no painel poria a Hierarquia a escrever `BSE`
numa linha escondida e `SUB` na que de facto semeia.

### §93.5 — A receita LÊ-SE na Hierarquia (a metade que faz o desenho funcionar)

O verbo aparece como **selo na linha**, e não só no painel lateral: *a Hierarquia já mostra a ordem;
o selo acrescenta o verbo; **ordem + verbo são a receita.*** Com o verbo só no inspector, entender uma
peça de cinco formas custa cinco cliques — que é exactamente a queixa que abriu esta wave.

⭐ **Os códigos são os do vetorial** (`UNI` · `SUB` · `INT` · `BSE`) e a tabela de tons de
`paint_hierarchy_row` **já os conhecia**: ela foi escrita em 22/08 pela outra metade do app, e o
comentário dela já dizia *«a BASE não tem verbo»*. Zero linhas novas no pintor da Hierarquia.

⚠️ **O selo mostra o verbo EFECTIVO** — quem herda sela `UNI` na mesma. A pergunta da lista é *«o que
acontece?»*; *«quem escolheu?»* é a do painel, e as duas têm superfícies separadas.

⭐⭐ **A PRECEDÊNCIA tem UMA excepção, e a regra que a decide vale para a fileira toda:**

> *O selo diz o que a linha não consegue dizer sozinha.*

⇒ **`ISO` > `SUB`/`INT`/`UNI` > `LNK` > `BSE`.** O `BSE` é **derivável da posição** (é a primeira
linha que conta), então cedê-lo ao vínculo não custa informação nenhuma; os outros verbos não são
deriváveis de nada na tela, e sem eles a **receita ganha um buraco** — uma sequência com uma linha
ilegível deixa de se ler **inteira**, ao contrário de uma marca de proveniência, que é um facto
independente por linha. O `ISO` continua acima dos dois porque é o único que explica uma **ausência**.

⚠️ **A 1.ª redacção desta secção dizia «o verbo ganha do `LNK`, e é uma perda deliberada» — e estava
errada por não ter a excepção.** Dois gates do vínculo apanharam-na, e a regra que os manteve
**intactos** é a que ficou. ⛔ **E o preço da cura de verdade está MEDIDO, não afirmado:** um segundo
selo por linha é `HierarchyEntity::badge: Option<String>` a virar lista, com N produtores (o vetor, a
sprite, a física, este módulo) — mudança foundational, e não «uma função». É esse o gatilho.

### §93.6 — A fileira do painel, e por que ela tem QUATRO chips

`Inherit` · `Add` · `Cut` · `Common`.

- ⚠️ **`Inherit` é o primeiro, e é ele que faz o modelo caber na fileira**: sem um gesto que devolva a
  forma ao padrão do grupo, escolher um verbo uma vez seria **irreversível**. *Um modo em que só se
  entra é um modo errado.*
- ⚠️ **Palavras diferentes das da fileira de cima**, e de propósito: as duas aparecem juntas com
  **sujeitos diferentes** (o grupo · a forma), e repetir "Union/Subtract/Intersect" faria as duas
  lerem-se como a mesma pergunta feita duas vezes. As escolhidas são as do *Shape Mode* do
  Illustrator, que é o padrão-ouro deste desenho.
- ⚠️ **A fileira NOMEIA o sujeito** (`This shape: Cylinder`). Tocar um filho pode acender o grupo
  inteiro no canvas, e sem o nome o artista escolhe o verbo sem saber de qual forma o painel fala —
  foi **exactamente** esse o defeito que o vetorial pagou em 22/08, e o report *«os botões não
  funcionaram»* não bastava para o localizar.
- ⚠️ **Herdar acende o `Inherit`**, e não o verbo herdado: o `active` diz *o que foi escolhido*.
  Acender o herdado faria um clique nele parecer inerte e no entanto mudar o estado.

⭐ **A MISTURA em vigor viaja com o verbo escolhido** (`verb_at(slot, blend)`): uma forma que herdava
a subtração de um grupo com filete `0,12` e passasse a subtrair com **aresta viva** mudaria de forma
ao clique, sem ninguém ter tocado num raio. É a lei que o `set_op` já escrevia para o grupo — *o raio
é do nó, não da operação*.

### §93.7 — Os gates, e a MUTAÇÃO QUE SOBREVIVEU

**20 gates novos**, e os 5 de alcance da W34 a varrer a fileira nova de graça.

| onde | o que prende |
|---|---|
| `ph2d-field-eval/src/verb_tests.rs` (7) | a dobra, **pelas duas rotas** (árvore + números, conferidas ponto a ponto dentro do `at`) |
| `ph2d-field-ecs/src/verb_tests.rs` (7) | a travessia mundo → documento, a base por **contribuição**, `union_all` a não herdar |
| `ph2d-panel-model3d/tests/seam.rs` (+1, e a varredura) | o **gesto real** (Down+Up) chega a `SetVerb` com o slot certo, e **nunca** ao `ApplyOp` |
| `field3d_reach_tests.rs` (a tabela) | oferecer ⟺ fazer, nos **seis** casos de seleção |

⛔⛔ **UM MUTANTE SOBREVIVEU, e o achado é maior que o gate.** Trocar `child.unwrap_or(parent)` por
`child.unwrap_or(Op::Union(Blend::Sharp))` — isto é, **apagar a herança inteira** — passou em todos os
gates então escritos.

⚠️ A causa não foi falta de cobertura: eles **comparavam duas construções**, e a mutação afectava as
duas da mesma maneira. *Um controlo que partilha o defeito do sujeito não é um controlo.*
⇒ A cura é o `inheriting_means_the_parents_verb_and_the_parents_blend`, que mede contra um **oráculo**
e não contra um irmão: com o pai a subtrair, o coração da 2.ª forma está FORA da peça — e isso é
verdade ou falso sozinho. Com ele, **os 5 mutantes morrem**.

⚠️ **E o gate de alcance apanhou um defeito silencioso MEU, na própria tabela dele:** as asserções
endereçavam as fileiras por **índice** (`ROWS[1]`, `ROWS[2]`), e inserir a do verbo no meio
re-apontou-as para outra fileira — elas continuaram a compilar e a correr, a medir a coisa errada.
Passaram a endereçar por **nome**. *Um índice para dentro de uma lista que cresce é um endereço que
muda de dono sem avisar.*

### §93.8 — O que mudou, em código

`ph2d_field::{fold_verb, Node::verb}` (`FIELD_DOC_VERSION` 4 → **5**) · `combine_trees` e
`Plan::Combine` passam a levar o verbo de cada filho **em pares** (⛔ não dois `Vec` paralelos: seriam
duas respostas a *«quantos filhos há»*) · `ph2d_field_ecs::{FieldVerb, set_verb, verb_of, verb_role,
VerbRole, contributes}` + `edit_verb.rs` (arquivo irmão — o `edit_tree.rs` está em 436 LOC, e a regra
é *split, nunca allowlist*) · a fileira `verbs`/`verb_subject` no retrato do painel, com id, braço,
`CHIP_FAMILIES` e i18n próprios (⚠️ **os cinco sítios** que a W48 pagou por esquecer um).

⚠️ **O `verb` é campo do `Node` e não uma lista no pai**: uma lista paralela a `children` seria uma
segunda resposta a *«quantos filhos há»*, e ficaria obsoleta em todo sítio que desloca índices — o
`union_all` é um deles. Preso ao nó, ele viaja de graça.

⚠️ **E o `union_all` LIMPA o verbo da raiz que adopta**: um verbo autorado dentro de uma peça fala dos
**irmãos dela**, e ali passaria a falar das **outras peças** da cena — uma peça inteira a subtrair-se
de outra sem ninguém o pedir.

### §93.9 — ⏳ O que fica para as etapas 2 e 3

- **(2)** ✅ **FEITA — ver §94.**
  ⚠️ **E uma correcção a esta lista:** a calibração do `Blend::Organic` **não** pertence à etapa 2.
  Medido em 28/08: *ele não tem produtor nenhum na UI* — nada o constrói fora dos testes, e o
  `set_radius` só o **preserva** se já lá estiver. A mentira dos 3/4 é **latente**, e quem a acorda é
  o **chip de carácter da etapa 3** ⇒ ela passa para lá, onde é obrigatória.
- **(3)** o **Chamfer** como 4.º carácter ao lado de `Sharp`/`Exact`/`Organic` — ⭐ **uma fórmula só**,
  porque a intersecção e a subtração saem por De Morgan (a nota do `Op` já o diz).

### §93.10 — ⚠️ O que o PORTÃO DE FECHO apanhou (quatro ✗, e três eram defeitos a sério)

O gate-mãe desta wave não foi nenhum dos 20 que escrevi: foram os que **já existiam**.

| ✗ | o que era | veredicto |
|---|---|---|
| `a_duplicate_carries_every_optional_component_of_a_node` | o `copy_optional` não conhecia o `FieldVerb` ⇒ **duplicar um furo devolvia uma forma que SOMA** | ⭐⭐ **defeito real**, curado. A mensagem do gate previa-o por escrito desde a W55 |
| `every_registered_component_has_a_descriptor` | o catálogo de componentes não tinha a linha do `FieldVerb` | ⭐ defeito real, curado (`machinery`, porque a **ausência** é que significa herança — um `+` do Inspector escreveria um verbo que ninguém escolheu) |
| `the_shape_of_a_saved_{field,profile}_is_pinned` | `148 → 151` e `85 → 86` | ✅ **esperado, e a conta bate exactamente**: `Option::None` custa 1 byte de discriminante, e são 3 nós e 1 nó. ⚠️ Re-pinar só é legítimo porque a **versão subiu no mesmo commit** — é o que o doc do gate exige |
| `shell_files_respect_hr18_loc_cap` | `field3d_scene_panel.rs` **606** · `field3d_smoke.rs` **606** | ⛔ **o segundo já estava a 606 no HEAD** — vermelho pré-existente que os outros ✗ escondiam (o nextest cancela na 1.ª falha). Os dois **partidos**, nunca allowlist |

⚠️ **E o `no_tofu_glyphs` apanhou uma seta `→` que EU deixei na W89**, num `println!` de sonda. Ela
viveu duas waves porque aquele arquivo não entrava na varredura impactada até esta o tocar.

⭐⭐ **Os dois splits são por ASSUNTO e não por tamanho:**
[`field3d_smoke_isolate.rs`](../../shells/desktop/src/field3d_smoke_isolate.rs) (as duas leis do
isolamento — o chip e a tecla — que são **duas perguntas**) e
[`field3d_scene_verb.rs`](../../shells/desktop/src/field3d_scene_verb.rs) (a fileira e o selo).
⚠️ E o primeiro desenterrou um defeito de documentação: o doc-comment do `gesture_in_progress`
estava **colado ao `toggle_isolate`, 200 linhas acima** — a família do *«atributo separado do item
por um doc-comment muda de dono»*. Devolvido.

### §93.11 — ⭐ E NENHUM gate construía as cenas do smoke

Achado ao acrescentar a cena 7: o `scene()` termina em
`.expect("as cenas do smoke são documentos válidos")`, e **nada** o exercia. Uma cena com um raio que
não cabe entrava em pânico **ao arrancar**, e o primeiro a descobri-lo seria o Enio — com a janela a
fechar e uma mensagem que não é para ele.

⚠️ O segundo lado é mais silencioso: um `n` **sem braço no `match` cai no `_`** e desenha a cena 1.
O artista pede a cena nova, vê a de sempre, e conclui que a feature não foi feita. É a família do
`no_two_smoke_scenes_claim_the_same_level` dos outros módulos, aqui pela primeira vez —
[`field3d_smoke_scene_tests.rs`](../../shells/desktop/src/field3d_smoke_scene_tests.rs), 3 gates.

⚠️ E o terceiro apanhou uma nota velha: o `main.rs` anunciava **`PH2D_FIELD_SMOKE=1..3`** com **seis**
cenas construídas. *Uma nota que diz o alcance de um roteador é a primeira coisa que alguém lê e a
última que alguém actualiza* — hoje há gate a ligá-la ao `match`.

### §93.12 — ⚠️ E DOIS gates mediam uma premissa que esta wave dissolveu

`the_hierarchy_says_which_row_is_isolated` e `a_badge_pinned_to_a_dead_object_is_not_painted`
perguntavam ao **mapa inteiro** de selos (`badges.len() == 1`, `is_empty()`), e isso valia enquanto o
isolamento e o vínculo eram os únicos selos. O verbo pôs um em **toda** linha que participa da
receita, de propósito — e as duas ficaram vermelhas sobre produto correcto.

⛔ **Não se baixou a barra: disse-se com precisão o que ela sempre foi.** A afirmação dos dois é *«uma
linha, e só uma, diz que está ISOLADA»*, e hoje é isso que eles contam (`isolados()`). Ela continua a
apanhar o defeito que os escreveu (selar todas) e o da irmã (selar uma entidade morta).

⇒ *Uma mudança de modelo obriga a re-perguntar o que cada gate ainda mede* — e o sinal de que é este
caso, e não uma regressão, é o gate falhar **a contar** e não **a afirmar**.

---

## §94 — W98: ⭐⭐⭐ UM RAIO DE JUNÇÃO POR FORMA — e as duas palavras que ele obrigou (28/08)

A etapa 2 das três que o Enio pediu em 28/08. O **motor** já a entregava desde a W97 (o `Op` carrega
o `Blend`, então quem traz o verbo traz o raio); o que faltava era a **linha** e o **nome**.

### §94.1 — A linha é DERIVADA, e é por isso que o painel não mudou

O painel monta as linhas de número a partir de `params_of(world, e)` — chave i18n, valor e faixa. ⇒
acrescentar `Param::Joint` ali fez a linha **aparecer sozinha**, com slider, campo numérico, undo e
persistência, **sem uma linha de mudança no painel**. *Um painel derivado é o que faz uma grandeza
nova custar o que ela de facto é.*

### §94.2 — ⚠️ UMA FORMA TEM DOIS RAIOS, e a colisão de nomes era real

| linha | de quem | existe quando |
|---|---|---|
| **Fillet** (`Param::Dim`) | das arestas **dela própria** — as 12 de uma caixa, o aro de um cilindro | sempre, mesmo numa peça de uma forma só |
| **Joint** (`Param::Joint`) | do **encontro** com o resultado das anteriores | só porque há alguma coisa antes |

Uma caixa arredondada que corta com aresta viva mostra os **dois números ao mesmo tempo**, e antes
desta wave os dois chamavam-se `Fillet`. *Dois rótulos iguais na mesma coluna são dois controles que
o artista não sabe separar.*

⭐ **E o GRUPO passou a dizer `Joint` também**, de propósito: depois do verbo por forma, o raio dele
**é** o raio de junção **padrão** — o que as formas caladas usam. *Uma grandeza, uma palavra.*

⚠️ **Um gate apanhou a imprecisão da 1.ª redacção**, e a distinção que ele forçou é a que importa: a
raiz **tem** a chave `field.dim.joint` (o padrão dela) e **não** tem o `Param::Joint` (*«o meu próprio
encontro»*). ⇒ duas coisas diferentes com o mesmo nome na tela, e está certo — *o que as separa é
quem escreve onde, não como se chamam*.

### §94.3 — ⭐⭐⭐ Escrever o raio MATERIALIZA o verbo, e é isso que faz a wave valer

A linha aparece **também para quem herda**, com o valor herdado: *«quero a boca deste furo mais
macia»* não pode exigir que o artista entenda o modelo do verbo primeiro. Escrever nela dá à forma o
verbo **por escrito** — o mesmo verbo que ela já usava, com o raio novo — e o chip `Inherit` apaga-se
à vista.

⛔ **Sem esta metade, arrastar a linha de uma forma calada escreveria no GRUPO**, e as outras caladas
mudariam com ela — que é exactamente o defeito que o verbo por forma existe para curar. ⚠️ E um gate
que só medisse *«o valor mudou»* passaria com essa escrita: **é o irmão calado que separa as duas
hipóteses**, e ele está nos dois gates (o da lei e o da costura).

⚠️ Zero é a **aresta viva** e não uma recusa (a lei do `set_shape_radius`, copiada de propósito para
não haver duas); negativo é recusado e deixa o nó **como estava**. O **carácter** da mistura
sobrevive ao raio novo.

### §94.4 — A prova

**7 gates novos** (6 na `ph2d-field-ecs` + 1 de **costura** no shell, que mede o retrato publicado →
o intent → o documento **cozido**, porque *uma lei correcta que o painel não publica é um gesto que
ninguém alcança*). **8 mutantes, 8 mortos** — entre eles *«a linha é oferecida a toda a gente»*,
*«materializar esquece qual verbo era»* e *«o carácter não sobrevive»*.

⚠️ E o portão de fecho voltou a apanhar o **tecto de LOC**: o gate de costura pôs o
`field3d_scene_tests.rs` em **676**. Partido por assunto para
[`field3d_verb_seam_tests.rs`](../../shells/desktop/src/field3d_verb_seam_tests.rs), **filho** do
módulo de testes para lhe usar as fixturas — duplicá-las seria a segunda cena de teste a envelhecer
sozinha. ⛔ *Split, nunca allowlist* — a terceira vez em duas waves.

### §94.5 — ⚠️ Uma correcção ao plano das três etapas

A nota da §93.9 mandava corrigir o `Blend::Organic` (que entrega **3/4** do número que mostra) nesta
etapa. **Medido: ele não tem produtor nenhum na UI** — nada o constrói fora dos testes e do smoke, e
o `set_radius`/`Param::Joint` só o **preservam** quando já lá está.

⇒ A mentira é **latente**, e quem a acorda é o **chip de carácter da etapa 3**. Ela passa para lá,
onde deixa de ser opcional. *Um defeito inalcançável é uma armadilha armada, não um defeito — e a
wave que o torna alcançável é a dona dele.*

---

## §95 — W99: ⭐⭐⭐ O CHANFRO — e as DUAS RÉGUAS que ele obrigou a separar (28/08)

A etapa 3 das três que o Enio pediu em 28/08.

### §95.1 — Uma fórmula, e ela é exacta

`min(min(a, b), (a + b − r)·√½)` — o plano do chanfro num canto de 90° é `a + b = r`, e a distância a
ele é `(a+b−r)/√2`, **exacta**. Intersecção e subtração saem por **De Morgan**, como as outras: o
`Op` já tinha a nota a dizer que *só a união precisa de fórmula própria*, e ela pagou-se aqui.

> ⭐ No CAD, filete e chanfro são duas máquinas com modos de falha diferentes. Aqui são a mesma conta
> com um termo trocado, e **nenhuma pode falhar**.

### §95.2 — ⭐⭐⭐ DUAS RÉGUAS, e confundi-las era a nota antiga da crate

O `union_smooth` trazia escrito: *«o `k` não é um raio — medido, entrega 3/4 do número»*. A medição
desta wave diz que **há duas grandezas**, e a nota media uma e falava da outra:

| carácter | **recuo** na parede | **mordida** no canto |
|---|---|---|
| `Fillet` (arco) | `1,0000` | `1,0000` |
| `Chamfer` (corte reto) | **`1,0202`** | `1,7071` |
| `Organic` (derretido) | `1,1644` | **`1,0000`** |

⇒ **nenhum carácter bate as duas**, e escolher qual calibrar é uma decisão de produto:

- **o orgânico calibra-se pela MORDIDA** (`Blend::ORGANIC_REACH`), porque é a **silhueta** que o
  artista vê: trocar `Fillet` ↔ `Organic` com o mesmo número deixa o canto onde está e muda só a
  forma da transição. ⇒ o recuo dele fica em `1,16×`, e isso é **divergência declarada**, com barra
  dos dois lados no gate — um borrão derretido não tem linha de tangência nítida para alinhar.
- ⛔ **o chanfro NÃO se calibra:** um corte reto e um arco de mesmo recuo arrancam material diferente
  no meio, e é essa diferença que o artista escolhe. Calibrá-lo daria **quatro chips com três
  formas**, e há um gate a proibi-lo pelo número (`mordida > 1,3`).

### §95.2-bis — ⭐⭐⭐ Eram TRÊS réguas, e o gate que escreveu a nota velha previu isto por escrito

A varredura de fecho reprovou o `the_organic_blend_falls_short_by_exactly_k_over_four` — **o gate que
estabeleceu o «3/4»**. O doc-comment dele dizia, palavra por palavra:

> *«Se alguém "consertar" isto em silêncio, o gate acusa; se alguém o **calibrar** de propósito
> (×4/3), o gate é o lugar onde a decisão fica escrita.»*

⇒ Ele reprovou no dia certo e pelo motivo certo. ⛔ **Mas o `×4/3` que ele sugeria era sobre uma
TERCEIRA grandeza:** o **valor do campo** no cotovelo — nem o recuo, nem a mordida. *Três réguas, e a
que decide é a que o artista vê.*

⭐⭐⭐ **E a constante fechou-se em forma analítica:** o smooth-min vale `d − k/4` onde as duas
superfícies distam `d`, e a mordida do filete exacto põe a silhueta em `d/√2`; igualar dá

> **`ORGANIC_REACH = 4(1 − 1/√2) = 4 − 2√2 = 1,171573`**

que é o número que a varredura tinha medido a `1,0000`. O gate passa a pinar `d/√2`, que é a mesma
conta do outro lado. *Uma constante analítica, confirmada por medição — e não um decimal ajustado até
o teste passar.*

### §95.3 — Três chips, e o quarto não existe de propósito

`Fillet · Chamfer · Organic`. ⚠️ **A aresta viva não é um carácter, é o raio ZERO** — e o slider já o
exprime. Um chip `Sharp` seria uma segunda porta para o mesmo facto, e as duas podiam discordar.

⚠️ A fileira aparece **onde há mistura**: numa operação (o carácter do filete dela, que é o padrão
dos filhos calados) e numa forma que se junta ao resto. Na **base** não, pela mesma razão do raio.
⭐ E numa forma ela **materializa o verbo**, como o raio de junção: escolher a forma da própria junta
*é* pronunciar-se.

⚠️ **Trocar de carácter não mexe no número.** Quem carrega no chip escolheu a forma; ver o raio
saltar junto seria o painel a decidir por ele.

### §95.4 — ⚠️ O que a marcha exigiu, e ele foi MEDIDO

O `march_depth` conta **arredondamentos exactos** porque `‖∇f‖` chega a `√2` neles. O chanfro entra
no **mesmo balde**, e o balde é medido (`the_chamfer_is_measured_against_the_march`): pô-lo no do
`Sharp` seria o erro que **fura** a peça, porque o termo do corte tem gradiente acima de `1` onde as
duas normais se alinham.

### §95.5 — ⛔⛔ DOIS MUTANTES SOBREVIVERAM, e um era um defeito VIVO

| mutante | por que sobreviveu | o que ele revelou |
|---|---|---|
| apagar o `min` do `union_chamfer` | as duas réguas medem **no canto**, e ali o termo do corte já é o mínimo — o `min` só protege **longe** dele | a propriedade *«é sempre um minorante»* estava escrita no doc-comment **sem gate nenhum**. Hoje tem: `the_chamfer_never_overstates_the_distance`, sobre uma janela `6×` maior que o alcance |
| `with_amount` devolver sempre `Exact` | **ninguém a chamava** | ⛔ **defeito VIVO:** a minha edição que rotearia o raio de junção por ela **nunca entrou** (o script abortou num `assert` e eu só refiz metade). O `Param::Joint` ficou com uma **cópia** da escada que **não conhecia o chanfro** ⇒ mudar o raio de uma junta chanfrada transformava-a em filete, **em silêncio** — e o comentário ao lado jurava que a porta era única |

⚠️ *Uma régua que só olha onde o fenómeno é forte não vê a guarda que o segura noutro sítio.*
⚠️ *Uma lei escrita em dois sítios ainda não é uma lei — e um comentário a dizer que é, é pior.*

⭐ E o gate do carácter passou a varrer **todos** os que sobrevivem (orgânico **e** chanfro): provar
um só foi o que deixou o defeito passar. ⛔ O `Exact` fica de fora **de propósito** — ele é o
**destino** de um erro, e incluí-lo faria o gate passar com a escada apagada.

**12 mutantes, 12 mortos.**

### §95.6 — ⭐ E o gate de alcançabilidade apanhou um defeito no mesmo dia

`set_character` chamava o `ph2d_field_ecs::set_op` para escrever no grupo — e **aquele preserva a
mistura de propósito** (é a porta de trocar o VERBO; o raio é do nó, não da operação). A troca de
carácter era engolida em silêncio: chip pintado, aceso, e a peça igual.

⇒ *Uma porta que guarda um campo não serve para escrever nesse campo.* Quem o disse foi o
`the_panel_offers_exactly_what_the_gesture_does`, com `oferecido=true age=false`.

### §95.7 — O formato

`FIELD_DOC_VERSION` **5 → 6**: variante nova no `Blend` **e** mudança de significado de um número
(`Organic { k }` → `Organic { radius }`). ⚠️ Um documento v5 leria o alcance cru de um orgânico como
se fosse raio, e a peça mudaria de forma em silêncio. ⭐ Os dois pinos de bytes **não se mexeram** — a
variante nova cabe no mesmo discriminante e o campo tem o mesmo tamanho.

**Smoke:** cena **`=8`** — as três colunas com a **mesma** junta de `0,18`, lado a lado.

### §95.8 — ⚠️ E o tecto de LOC, pela QUARTA vez em três waves

`ph2d-field/src/lib.rs` passou a **739** contra o tecto de **700** da workspace. Partido por assunto
para [`blend.rs`](../../crates/ph2d-field/src/blend.rs) — tudo o que responde *«que forma tem esta
junta, e de que tamanho»*; o `Op` e o `fold_verb` (*«quem se junta a quem»*) ficaram onde estavam.

⛔ *Split, nunca allowlist.* A conta das três waves: `field3d_scene_panel.rs` · `field3d_smoke.rs`
(pré-existente) · `field3d_scene_tests.rs` · `ph2d-field/src/lib.rs`.

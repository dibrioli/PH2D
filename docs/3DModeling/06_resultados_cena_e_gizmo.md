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

## §13 — Aberto

- ✅ **orientação Global/Local FECHOU** na W7 (§6)
- ✅ **rotacionar e escalar FECHARAM** na W6 (§5)
- ✅ **clicar na peça para selecionar FECHOU** na W7 (§6) — e o custo está medido: **0,10 ms**
- ✅ **snap e leitura numérica FECHARAM** na W8 (§7)
- ✅ **o undo de um arrasto FECHOU** na W6 (§5) — e a nota que estava aqui estava **errada**: a lei
  do shell já existia e o que faltava era o módulo dizer-se. *Meça o mecanismo antes de construir o
  que a nota prescreve.*
- ✅ **duplicar e apagar FECHARAM** na W11 (§10)
- ⏸️ **digitar o número** durante o arrasto (o `G X 0.5` do Blender) — a ficha mostra, mas não aceita
- ⏸️ o **pivô** é sempre o centro do nó. Um pivô escolhido (centro da seleção, cursor 3D) é produto,
  e entra com a UI que o escolhe
- ⏸️ perspectiva (herdado da W2) — quando entrar, entra **num sítio**: `Orbit::project`

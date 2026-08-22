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

## §13 — Aberto

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
- ✅ **W31 (§32): um objeto largado em cima de outro deixou de SUMIR** — só uma operação pode ter
  filhos, e a forma anfitriã é promovida a grupo (a peça na tela não muda). E **criar grupo** passa
  a ser um gesto: uma forma sozinha + um botão de operação. ⏸️ Fica: ninguém **diz** que um grupo
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

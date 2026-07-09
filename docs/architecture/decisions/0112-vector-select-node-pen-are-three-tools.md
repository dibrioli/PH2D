# ADR-0112 — Select, Node e Pen são três ferramentas, e o pivô nasce no centro da forma

- **Status:** aceito (Enio, 2026-07-09)
- **Fecha:** [ADR-0111](0111-vector-shapes-have-transforms-and-use-the-sprite-gizmo.md) §6

## Contexto

O ADR-0111 deu `Transform` às formas e pôs o gizmo de sprite a manipulá-las. Só que a
ferramenta Vector continuava sendo **uma coisa só**: a caneta. E o gizmo, quando a
forma estava selecionada, **registrava as suas alças no hit-index** — então a pressão
no canvas era consumida por ele antes de chegar ao pen. No smoke:

> *"ao voltar para o modo de edição com pen, os nós não eram selecionados mas o gizmo
> sim. E se eu clico em cima de uma forma com a pen (sem tocar em um ponto) cria uma
> linha em vez de selecionar."*

Os dois sintomas são a mesma coisa: **caneta e gizmo disputando o mesmo clique.** Não
há como arbitrar isso por prioridade de hit-test — o que o clique significa depende do
que o usuário está fazendo, e isso é o que uma *ferramenta* codifica.

## Decisão

`DrawMode` ganha dois modos que **não desenham**, e o padrão passa a ser o primeiro:

- **Select** (seta preta) — seleciona e **transforma** pelo gizmo. A ferramenta nem
  captura o canvas: o clique cai no caminho de sempre (picking + gizmo), o mesmo de um
  sprite. Não há âncoras na tela.
- **Node** (seta branca) — edita âncoras e handles. **Nunca cria um path.** Clique numa
  âncora agarra; perto de um segmento insere um vértice; no preenchimento seleciona a
  forma; no vazio desseleciona.
- **Pen** — cria um path novo e edita os nós que ela mesma pôs.

**O gizmo da forma só existe no modo Select** (e fora da ferramenta). Em Node e nos
modos de desenho ele não publica `GizmoView`, logo não pinta e não registra alça
nenhuma — o clique do nó chega inteiro ao pen. É a regra que dissolve a disputa.

Consequência menor mas necessária: `path_at` passou a aceitar o **preenchimento**, não
só a proximidade do traço. Sem isso a seta branca só pegaria a borda da forma.

## O pivô nasce no centro da forma

Um sprite tem a origem no centro do quad por construção (`anchor = 0`). Um path nasce
com a geometria em coordenadas de **mundo** e a entidade na identidade — a origem, e
portanto o pivô, caía no centro do **mundo**.

`vec_transform::settle_origins` conserta isso assim que a forma pára de crescer: a
translação vai para o centro da bbox e a geometria local recua o mesmo tanto. A forma
não se move um pixel; só passa a ter uma origem que significa algo. Só toca quem está
na identidade e sem pai — um path já movido, escalado ou parentado tem a origem que o
usuário lhe deu. É idempotente, e o path que a caneta ainda constrói fica de fora (a
origem ficaria pulando a cada vértice).

O botão **"Set Center"** do painel volta — eu o havia removido no ADR-0111 achando-o
redundante, e estava errado: ele é o único jeito de mover a origem de uma forma.
Agora ele arma, e a próxima pressão no canvas põe a origem sob o cursor. É o **mesmo
código** de `settle_origins` (`move_origin_to`), com outro alvo: a translação vai para
o ponto, a geometria recua na medida certa — desfazendo a rotação e a escala próprias,
que é onde um erro de espaço passaria despercebido.

### O preço de assentar a origem

Assentar tem um efeito que não se vê até quebrar: **nenhum path fica mais na
identidade**, e a bbox local de toda forma assentada está **centrada na origem**.
Qualquer código que lia a bbox local como se fosse mundo passa a mentir. Dois grupos:

- **Continuam certos:** o que lê local e escreve local — geometria de gradiente,
  `rotate_path_by` (gira a forma em torno do próprio centro), flip. O número nunca sai
  do espaço do path.
- **Passaram a mentir:** o que compara formas **entre si** ou mostra um número ao
  usuário — `align`, `distribute`, e os campos X/Y/W/H do painel. Sem correção, alinhar
  empilharia todas as formas no mesmo ponto, porque as bboxes locais são idênticas.

Daí `VecScene::path_world_curve_bbox` e `translate_path_world`: a leitura sobe pelo
afim, o delta desce por ele. É a mesma regra do pen (mundo fora, local dentro),
aplicada ao painel.

E os **gestos** (a caneta e a ferramenta de forma) reescrevem geometria em coordenadas
de mundo a cada frame — `ShapeTool::on_drag` regenera `verts` inteiro. Assentá-los no
meio do gesto faz geometria e `Transform` **somarem**, e a forma sai deslocada do
cursor exatamente pelo ponto onde o arrasto começou (Enio, 2026-07-09: *"offset bizarro
em relação ao mouse"*). Por isso `settle_origins` recebe a lista de paths em gesto e os
deixa em paz até o Up.

## Dois gaps que este ADR ABRE (verificados, não suspeitados)

**1. `vec_save` / `vec_load` (Ctrl+S / Ctrl+O com a tool ativa) perdem a pose.**
Eles serializam só o `VecScene` (`ph2d_vec_scene.postcard`). Depois de assentar, a
geometria de cada forma está centrada na origem **dela**, e a pose vive na entidade
ECS — que não é salva. Consequência exata: `save` → sair → `load` numa sessão nova
empilha todas as formas na origem do mundo. Um arquivo **antigo** (geometria em
world-space) ainda carrega certo, porque `settle_origins` o recentra. A quebra é *para
a frente*, não para trás.

O certo é o save de cena do ECS, não um segundo formato. Até lá, `vec_save` é uma
conveniência de dev e deve ser tratada como tal.

**2. Transformar uma forma pelo gizmo não é mais desfazível.** `ph2d_vec_edit::History`
tira snapshot do `VecScene`; um `Transform` não está lá, e o drag do gizmo de sprite
não empilha undo em lugar nenhum (nem para sprites). Antes do ADR-0111, o gizmo
vetorial próprio mutava geometria, então **era** desfazível. Ou seja: ficou consistente
com o resto do editor, e pior do que era. O conserto é um undo de `Transform` para
todos os objetos, não um remendo vetorial.

## Consequências

**Boas.** Um clique tem um dono. A caneta faz uma coisa. O pivô de uma forma é o centro
dela, como o de um sprite.

**Quebra de hábito.** A ferramenta Vector abre em **Select**, não na caneta. É a
convenção de todo editor vetorial, e o pill agora alterna (ADR-0111), então sair é um
clique.

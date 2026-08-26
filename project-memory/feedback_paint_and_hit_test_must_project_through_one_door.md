---
name: feedback_paint_and_hit_test_must_project_through_one_door
description: "Quando a tinta e o hit-test projectam o MESMO mundo por janelas diferentes, saem DOIS sintomas de uma causa — a alça desenhada não é a alça que existe; e uma lei escrita no doc de um módulo irmão não impede o erro, só um gate impede"
metadata:
  type: feedback
---

Construí o gizmo de canvas dos deformadores de quadrilátero e o Enio reportou **duas**
coisas: *"grade fora do lugar. drift"* e *"não consegui manipular pontos e alças no
canvas"*. Parecem dois defeitos e são **um**: o overlay projectava com a janela CHEIA e o
hit-test com a janela da CENA (o sub-retângulo do split). ⇒ a alça que se via não era a
alça que existia — o desenho saía deslocado **e** o clique errava, pelo mesmo motivo.

⚠️ **E a lei já estava escrita, no módulo irmão, com o precedente nomeado:**

> *"A vector shape projected with the FULL window drifts off them — shifted and shrunk —
> which is why a `motion.path`'s walkers sat on a displaced copy of the drawn curve."*

Eu **li** esse comentário — ele estava no ficheiro de onde copiei a estrutura — e mesmo
assim passei `surface.size()` ao overlay.

**Why:** um doc-comment protege quem o lê **no momento certo**, e o momento certo aqui é
o instante de escrever o argumento, não o de estudar o módulo. *Uma lei sem gate é uma
nota que se lê e não se executa* — a mesma família de
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]].

⚠️ **E o par de sintomas é a assinatura desta classe.** Quando um relato traz *"está
deslocado"* **e** *"não consigo clicar"*, a primeira hipótese não são dois bugs: é **uma**
projecção divergente entre quem pinta e quem agarra. Procurar dois culpados custa o dobro
e acha zero.

**How to apply:**
1. ⚠️ **Uma PORTA ÚNICA, e não dois chamadores disciplinados.** A cura não é lembrar-se de
   passar a janela certa — é o overlay receber o *split* e resolver a janela por dentro,
   de modo que o chamador **não tenha como** errar. É a lei
   [[feedback_the_representation_can_delete_the_special_case]] aplicada a um argumento.
2. **O gate fecha o CICLO pelo produto:** projecta cada alça para a tela com a porta da
   tinta, volta ao mundo com a porta do ponteiro, e exige que o agarre encontre aquela
   alça. Ele é puro (câmara + janela + alças), não precisa de app nenhum, e teria apanhado
   isto no minuto zero.
3. **E ele precisa do CONTROLE que reproduz o defeito** — pintar com uma janela e agarrar
   com outra tem de fazer o clique ERRAR a maioria das alças. Sem essa metade, o gate
   passaria por vacuidade no dia em que a projecção deixasse de importar.
4. ⛔ Quando um gizmo novo nasce copiando um irmão, **copie também os gates dele**, não só
   a estrutura. Foi a estrutura que eu copiei; a lei ficou no comentário.

*Irmã de `feedback_paint_and_dispatch_must_read_the_same_source` (a mesma doença com a
mesma cura, quando o que diverge é a FONTE em vez da PROJECÇÃO).*

---

## ⭐⭐⭐ Adenda 2026-08-26 — o QUARTO consumidor, e o único que não podia estar certo por acaso

A mesma doença voltou pela quinta vez, e desta vez **não era um gizmo novo a copiar um
irmão**: era um passe que já existia e **nunca teve o parâmetro na assinatura**.

Sob o split da tool Motion, três rotas desenham na cena. Duas recebiam o sub-retângulo (o
passe de sprites, por `uniform_for_subrect` + `set_viewport`; o Vello, por
`scene_camera_window`). A terceira — **o passe do Flip** — recebia `window_size` cru e
projetava a **janela CHEIA**: `H/floor(H·t) = 1/t ≈ 1,82×`.

⚠️ **E o sintoma não é um desalinhamento, é um MULTIPLICADOR do arrasto.** O pan converte o
deslocamento do rato em mundo pela altura **da cena**; quem projeta pela janela anda `1,82×`
o que o cursor anda. ⇒ parado, offset fixo; a arrastar, uma abertura que **cresce com a
distância percorrida e sem tecto**. O report do dono do produto foi *«a imagem de referência
sofre um drift no pan»*, e a assinatura *«sempre para o mesmo lado quando se arrasta muito
para o mesmo lado»* é exatamente a de uma diferença de ESCALA, nunca a de um offset.

**Why:** as três curas anteriores (o passe de sprites, o chrome/grade, as formas vectoriais
em 2026-07-25) foram cada uma feita **no sítio que reportou**. *Uma lei curada no sítio do
report volta pelo consumidor seguinte* — e o seguinte era o único a quem ninguém tinha dado
sequer a possibilidade de acertar.

**How to apply:**
1. ⭐⭐ **Enumere os CONSUMIDORES antes de curar o que reportou.** `grep` pelas duas formas —
   quem recebe o sub-retângulo e quem recebe a janela crua — e trate a lista inteira. Aqui,
   dois greps (`scene_viewport|scene_camera_window` contra `view_proj(|world_to_screen_affine(`)
   puseram as três rotas lado a lado em **uma** chamada.
2. ⭐⭐⭐ **A régua é o DESLOCAMENTO entre dois centros de câmera, não a posição num.** Um gate
   que compara posições absolutas num único quadro não distingue *drift* de *offset*, e é
   drift que o artista vê. O gate novo mede as três portas × dois centros e exige que os
   **deltas** coincidam.
3. ⚠️ **Nem toda cura desta família é um `set_viewport`.** Quando o passe compõe em alvos
   intermédios e termina num blit de alvo inteiro, recortar obrigaria a redimensionar a cadeia
   toda; a saída é projetar pelo sub-retângulo e **remapear o NDC** para a região que ele ocupa
   (`ndc' = a·ndc + b`) — que é o que a rota do Vello já fazia sem recorte nenhum. *Copie a
   rota que já está certa, não o mecanismo que se usou noutro passe.*
4. ⛔ **A grandeza derivada tem de seguir junto.** A espessura do traço (`px_per_world`) passa
   a ser a da CENA; o `viewport` do uniform continua a ser o do ALVO, porque o shader converte
   coordenada de fragmento do alvo cheio. Mudar uma e esquecer a outra é o mesmo defeito noutra
   unidade.

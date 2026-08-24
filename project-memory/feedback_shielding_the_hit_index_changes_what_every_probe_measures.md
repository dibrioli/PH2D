---
name: feedback_shielding_the_hit_index_changes_what_every_probe_measures
description: "Recortar o HitIndex pela banda do corpo é a cura certa — e ela reescreve o significado de toda sonda que media «o fundo do último retângulo registado»: essa grandeza passa a saturar na janela e a medir a JANELA em vez da coisa"
metadata:
  type: feedback
---

O painel de params rolava, e o `push_clip` recortava só o **desenho**: uma linha
rolada para cima continuava **registada** sob o título. A cura foi *uma banda,
dois consumidores* — `HitIndex::push_clip` com o mesmo `body_rect`. ⚠️ **A
ferramenta já existia** (o `section_header::body` usa-a desde que nasceu); o que
faltava era este painel chamá-la.

E então **três gates ficaram vermelhos**, todos por medirem a mesma coisa:
`max(r.y + r.h)` sobre os retângulos registados. Com a blindagem essa grandeza
**satura na altura do corpo**: o `motion.bezier_warp` passou a ler `802` de um
dock de `880` e o gate acusou-o de ter perdido params. Não perdeu — a sonda é que
deixou de medir o nó e passou a medir a janela.

**Why:** o fundo do último hit-rect responde *"até onde se pode APONTAR"*. Antes
da blindagem isso coincidia com *"até onde se DESENHOU"*; depois, são duas
perguntas diferentes, e uma sonda que queria a segunda tem de ler a altura de
conteúdo **publicada** (`panel_content_h`), que começa em zero e não é recortada.

⚠️ **E a mudança de régua desloca a barra junto:** o conteúdo publicado não
inclui a faixa do título, então compará-lo com a altura do DOCK diz que um nó
cabe quando ele já não cabe. A comparação certa é `content_h` contra
`visible_h` — o mesmo par que o `dispatch_wheel` usa.

**How to apply:**
1. Ao recortar um `HitIndex`, **grepe por quem lê registos como medida de
   tamanho**. Um gate que compara alturas por hit-rects é o primeiro a mentir.
2. Um número «nomeado» num gate (*este nó mede 1083 px*) é um **retrato tirado
   com uma régua**. Se a régua muda, o número muda sem que nada do produto se
   mexa — e a mensagem *"re-meça e mova o número, ou desfaça"* leva à conclusão
   errada. Escreva no gate **qual régua** tirou o retrato.
   [[feedback_a_ruler_anchored_in_the_world_measures_the_gesture_not_the_shape]]
3. O gate que pedia esta cura já existia e dizia-o em texto (*"o dia em que o
   teto subir é o dia em que a blindagem passa a ser necessária"*) — e ficou
   vermelho **um bloco inteiro** sem ninguém ver, porque o portão de fecho de
   cada bloco corria só as crates do diff. *O portão do fim da linha é sobre a
   workspace, e é para isto.*

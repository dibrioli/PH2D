# HANDOFF — `line/sculpt3d` · o Painter pinta a peça, etapa 1 (2026-09-24)

> Ordem do dono (24/09): *«Meu pedido inicial era a integração total do módulo
> Painter já existente para que consiga pintar com os mesmos features na malha
> 3d»* — e, sobre o plano de quatro etapas que lhe devolvi, *«sim. comece»*.
> Esta é a **etapa 1**: o modo `Digital` do Painter pinta a peça com os pincéis e
> as definições DELE.
>
> **Contadores:** `PROJECT_SCHEMA` 0 · os três registos 0 · `SCULPT_DOC_VERSION` 0 ·
> zero contrato (`Tool=12` e o `CanvasPaintTool` intocados — o `PainterTool` ganha
> métodos **inerentes**) · zero ADR · zero dependência externa. ⚠️ **Uma aresta
> nova no grafo:** `ph2d-app-sculpt3d → ph2d-tool-painter` (família → ferramenta,
> que a régua de camadas permite; o `architecture_no_dependency_climbs_a_layer`
> passou no portão).

## §1 — O que o artista faz agora

Com uma peça no ecrã, ligar `IMG` na barra de cima e escolher `PNTR` no rail: o
pincel passa a ser um **círculo no ecrã com o raio do pincel do Painter**, e
pintar põe a tinta **na peça** — com a cor, o tamanho, a força, a pressão e o
pincel que o painel do Painter tem. A tinta vê-se **enquanto** o traço corre, cai
na **tinta fina** quando o `Paint Detail` está armado (e na cor por vértice quando
não está), respeita a **máscara**, e **um `Ctrl+Z` desfaz a pincelada** como
desfaz uma do pincel de pintura da escultura.

## §2 — ⭐⭐⭐ O desenho: o Painter não sabe o que é uma malha, e não precisa

**O Painter pinta uma imagem TRANSPARENTE do tamanho da vista** — o motor dele
inteiro, sem uma linha nova de pintura — e **a escultura pousa essa imagem na
peça**, só onde a superfície está à vista. É a projecção de ecrã dos programas de
pintura 3D, e é o que faz *«os mesmos features»* sair de graça: cada pincel, cada
definição e cada modo do `Digital` que o Painter tem hoje pinta a tela, e a tela
chega à peça pelo mesmo caminho.

⭐⭐ **Cada amostra recalcula-se a partir da BASE de antes do traço**, nunca da cor
viva: `nova = base·(1 − a·k) + pm·k`, com `pm`/`a` a amostra bilinear
**pré-multiplicada** da tela e `k` a liberdade da máscara pela MESMA porta que o
carimbo e o `Fill` usam (`preenche::keep_da_amostra`). ⇒ **pousar duas vezes é
pousar uma** (idempotente), e é isso que permite re-projectar a tela inteira do
traço a **cada quadro** sem escurecer a tinta — a pincelada vê-se enquanto corre.
A base é a do congelamento do traço (`tinta_fina.base` · `stroke.base_color`), que
já existia para o desfazer.

## §3 — A lei: `ph2d_sculpt3d::tela_na_malha`

* **Visibilidade = de frente E desimpedida.** A face tem de apontar ao olho
  (`de_frente`), e um raio do olho à amostra, no espaço LOCAL da peça, tem de a
  alcançar (`h.t ≥ dist·(1 − FOLGA_DA_OCLUSAO)`, folga relativa `1e-3`) — ⇒ uma
  placa à frente tapa o que está atrás, e a face de costas não recebe tinta.
* **A visibilidade é LEMBRADA por amostra, por traço** (`visivel`): a re-projecção
  por quadro não re-lança um raio por amostra por quadro. Gate com contador de
  raios.
* **As faces são arrumadas numa grelha de ecrã de `32 px`** (`CELULA`), para a
  pousada visitar só as faces sob o rectângulo que a tela mudou.
* **A amostragem da tela:** centro do pixel em `i + 0,5`, leitura fora da tela
  **agarrada à borda**, e a pertença ao rectângulo sujo tem **margem de 1 px** —
  a pegada bilinear lê o pixel vizinho. ⚠️ As duas últimas foram escritas por
  gates vermelhos: sem elas os vértices na borda da tela liam transparente.
* **Dois caminhos, uma lei:** com a tinta fina armada escreve AMOSTRAS
  (`fina.tocou`/`fina.repinta`); sem ela, VÉRTICES (`tocou_vertice`/
  `repinta_vertice`, no `stroke_freeze.rs`). Os dois recalculam da base.

## §4 — As guardas no Painter: a tela entra como DOCUMENTO

A tela prende-se por `bind_document(SCREEN_CANVAS_DOC = u64::MAX, …)`, e **não**
por `set_source`: o `bind_document` GUARDA as camadas da sprite que estava ligada,
um `set_source` achatá-las-ia (gate `prender_a_tela_nao_achata_a_sprite_que_estava_ligada`).

⚠️ **A ponte da sprite corre a cada quadro e, sobre esta tela, faria quatro
defeitos** — drenar a pré-visualização (a escultura perdia o rectângulo sujo),
subir a pista GPU à textura de uma sprite, ligar a sprite escolhida por cima da
tela a meio de um traço, e o `Apply` assar a vista numa sprite. ⇒ as **quatro
portas que ela usa** perguntam `on_screen_canvas()`: `take_preview_arc`,
`take_preview_dirty`, `needs_document_bind`, `take_pending_commit`. ⭐ **A guarda
vive na FERRAMENTA**, que é quem sabe o que tem ligado; a escultura drena por uma
porta própria (`take_screen_canvas`, que usa o corpo renomeado `drain_preview_arc`).

Ciclo de vida: a tela **limpa-se a cada traço** (a peça guarda a tinta; a tela é
só o traço em voo — deixá-la cheia faria o traço seguinte recompor o anterior),
**segue o tamanho da vista**, e **solta-se** quando o barro sai do ecrã (a ponte
da sprite volta a poder ligar a sprite escolhida no quadro seguinte).

## §5 — A costura (`ph2d_app_sculpt3d::painter_na_malha`) e os três elos da shell

* `quadro(scene, painter)` — por quadro: com barro no ecrã **pousa primeiro** o
  que a tela mudou (a tinta em voo é da tela de antes de um redimensionamento),
  depois prende a tela ao tamanho da vista e publica o raio do pincel para o
  anel; sem barro fecha o traço e solta a tela; sem o Painter na mão, fecha.
* `entrega(scene, painter, x, y, pressão, fase)` — cada ponto do gesto, de janela
  para a tela; o pen-down fora da vista não é do Painter; o `Up` pousa, fecha o
  traço (`close_stroke`, a porta de todo traço ⇒ o desfazer) e limpa a tela.
* `painter_abre` mira a peça, **calcula a vista ANTES do empréstimo** do plano de
  tinta fina (um `return` depois dele deixava o plano preso num traço que ninguém
  fecha), começa o traço e empresta o plano pela porta do DONO.
* ⚠️ **Errar a peça não recusa o traço** — a lei do pincel de cor desta casa
  (21/09): a tinta aterra onde a peça estiver quando o traço lá chegar.

Os três elos da shell (+32 linhas; a catraca da shell fica em `196 283 ≤ 196 990`):
o `sculpt3d_pointer_down` **devolve o botão ESQUERDO** quando a tela está presa,
o `deliver_canvas_pointer` desvia para `entrega` antes do caminho da sprite, e a
`fase_painter_dispatch` chama `quadro` antes do `FrameGfx::of`. ⭐ Confirmado a
ler as rotas: o pen-up da escultura **não engole** o `Up` do Painter (sem `Drag`
aberto ele devolve `false`), e a ponte da sprite **só liga documento com uma
sprite escolhida**, logo nada esvazia a tela a cada quadro.

O **anel**: com o Painter na mão o `cursor_mark` desenha um círculo no ECRÃ com o
raio do pincel dele (`painter_raio_px`, publicado pelo `quadro`) — o anel deitado
na superfície descreveria o pincel da escultura.

## §6 — Gates

| onde | quantos | corre sem placa? | o que afirma |
|---|---|---|---|
| `ph2d-sculpt3d/tela_na_malha_tests.rs` | 10 | sim | tela opaca pinta ao bit · meia cobertura dá a mistura exacta · pousar duas vezes = uma, sem raios a mais · máscara · nada fora do rectângulo · oclusão · face de costas · na tinta fina uma aresta passa dentro de UMA face · **e as duas irmãs na tinta fina** (idempotência · máscara), escritas pelas duas sobreviventes |
| `ph2d-tool-painter/screen_canvas_tests.rs` | 5 | sim | a drenagem entrega o rectângulo · as 4 portas guardadas (com o CONTROLO de que a drenagem da escultura funciona) · limpar/soltar · segue o tamanho · não achata a sprite |
| `ph2d-app-sculpt3d/tinta_no_produto_painter.rs` | 3 `#[ignore]` + placa | não | um traço do Painter pela porta pública aterra na tinta fina e o `Ctrl+Z` devolve **ao bit** · sem tinta fina aterra nos vértices · sem cena a tela solta-se |
| `ph2d-app-sculpt3d/painter_fiacao_tests.rs` | 2 | sim | **o censo de TEXTO dos 8 elos** (família + os três da shell), com CONTAGEM por agulha e o CONTROLO da prosa |

⛔⛔ **O censo existe porque os gates de produto NÃO ALCANÇAM a shell:** eles
chamam `entrega` e `quadro` directamente, e são `#[ignore]` + placa — nem o CI nem
a suíte da família os correm. *Uma shell que deixasse de chamar a costura ficava
verde em todo o lado* — a forma que o censo irmão (`tinta_fiacao_tests.rs`) já
pagou cinco vezes, aqui escrita **antes** de uma mutação sobrevivente a pedir.

**Prova de mutação** (`docs/3D/ferramentas/muta_o_painter_na_peca.sh`, pré-voo
`24 de 24` âncoras; a POPULAÇÃO é por mutação e é de quem OBSERVA — a lei pelos
gates dela, as guardas pelos do Painter, a costura e a SHELL pelo censo):

### §6.1 — ⛔⛔ `21 de 23` à primeira, e as DUAS sobreviventes eram o MESMO buraco

`L4` (a máscara ignorada no caminho das AMOSTRAS) e `L7` (a amostra fina misturada
sobre a cor CORRENTE, logo a re-projecção por quadro escurecia a tinta) sobreviveram
— **os gates de máscara e de idempotência só mediam o caminho dos VÉRTICES**, e a
lei tem dois caminhos com duas escritas (`fina.repinta` · `repinta_vertice`). *Um
gate numa de duas cópias de uma lei afirma sobre metade dela.* ⇒ as duas irmãs na
tinta fina (`com_tinta_fina_pousar_duas_vezes_nao_engrossa_a_tinta` ·
`com_tinta_fina_a_mascara_protege_as_amostras`, cada uma com o CONTROLO de que a
pousada sem a condição pinta), e a re-corrida das duas mais o controlo:
**`2 de 2` sangram, controlo `1 de 1`** ⇒ o placar da wave é **`23 de 23`**.

⚠️ O arnês ganhou `MUTA_FILTRO=<regex>` para essa re-corrida, e o sumário **diz
que ela é PARCIAL** — *um placar parcial lido como completo é a forma mais barata
de um arnês mentir*. ⭐ E a costura da shell é mutada **por TEXTO** (`P2`/`P3`
renomeiam a chamada): o censo não compila a shell, lê-a, e por isso sangra sem
uma crate a mais na corrida.

## §7 — O portão apanhou DOIS vermelhos meus, os dois na shell

1. **Tecto de LOC (HR-18):** o ramo novo levou `painter_canvas_input.rs` a `611`
   contra `600`. Curado por **CORTE por responsabilidade**, nunca por isenção: as
   duas teclas do pincel (`[`/`]` e `E`) saíram para o irmão
   `painter_canvas_keys.rs` — aquele ficheiro é o do PONTEIRO e elas são teclado
   (`611 → 577`). ⚠️ O único gate que as nomeia lê a CHAMADA no teclado, não a
   definição.
2. **O censo dos consumidores do `pick`** (`a_stroke_belongs_to_the_piece_it_started_on`,
   `left: 5, right: 4`): o ramo do Painter no `cursor_mark` picava **uma segunda
   vez**. Ele é `&self` e somente-leitura, logo não era o perigo que o gate guarda
   — mas era um raio a mais por quadro sem razão. ⇒ **um pick só**, lido pelos
   dois ramos; o gate volta a `4` **sem ser tocado**.

Portão: `nextest-impacted` vs `main` **18 655** testes, os dois vermelhos acima e
mais nenhum; depois da cura, os dois + o censo + a suíte da família **310/310**;
clippy `-D warnings` zero nas quatro crates tocadas.

## §8 — ⚠️ Os limites da etapa 1, ditos (e o que vem)

* **Pinta-se o lado que se VÊ** — o de trás espera que o artista rode a peça.
* **A tela começa TRANSPARENTE:** os modos que misturam com o que JÁ está pintado
  (borrar, esfregar, multiplicar) precisam da cor da peça na tela — **etapa 2**,
  junto com liquify, balde e aquarela.
* **As teclas nuas da escultura** (`[`/`]` incluídas) continuam da escultura; o
  tamanho do pincel muda-se no painel do Painter.
* **Várias vistas:** pinta-se na vista activa.
* Etapa 3: tinta molhada e impasto · etapa 4: as camadas e efeitos do Painter na
  peça.

## §9 — Aberto

* ✅ **A foto da pincelada** (`diag_fotografa_a_pincelada_do_painter`, sonda
  versionada; `PH2D_SONDA_PNG=<ficheiro>`): três traços ondulados do Painter na bola
  da cena `=52`, a seguir a curva e a luz dela, com a borda macia do pincel. ⚠️ A
  1.ª tentativa foi RECUSADA pelo portão da placa (outra linha segurava-a há mais de
  `300 s`) — *nunca se força*; a 2.ª correu com os três gates de produto: `4/4`.
* O **smoke do dono** é o primeiro pedido desta etapa.

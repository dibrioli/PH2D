# HANDOFF — `line/sculpt3d` · o Painter pinta a peça, etapa 2 (2026-09-24)

> Ordem do dono: *«smoke OK. Siga»* — sobre a etapa 1
> ([handoff](HANDOFF_line_sculpt3d_O_PAINTER_NA_PECA_2026-09-24.md)). A etapa 2
> é a dos modos que **lêem a cor que já está debaixo do pincel**: borrar,
> esfumar, clonar, deformar (liquify), o balde, curar (inpaint), a aquarela, a
> tinta molhada e toda mistura que não seja o «over».
>
> **Contadores:** `PROJECT_SCHEMA` 0 · os três registos 0 · `SCULPT_DOC_VERSION` 0 ·
> zero contrato · zero ADR · zero dependência nova (a aresta
> `ph2d-app-sculpt3d → ph2d-tool-painter` é da etapa 1).

## §1 — O que o artista faz agora

Com o Painter na mão sobre a peça, os botões `SMEAR`, `BLUR`, `CLONE`,
`LIQFY` e `INPNT` do rail, o balde (arrastar do `C&F` para a peça), e os meios
`Watercolor` e `Wet Paint` **mexem na tinta que a peça já tem** — antes, sobre
a tela transparente da etapa 1, borravam o nada.

## §2 — ⭐⭐⭐ O desenho: a tela começa com o RETRATO da peça, e a lei passa a ser a DIFERENÇA

Um borrão lê a cor debaixo do pincel, e essa cor estava na PEÇA. ⇒ no pen-down
destes modos a tela do Painter começa com a peça desenhada nela
([`ph2d_sculpt3d::tela_semente::semente`]), e cada amostra recebe

```
nova = base + k · (c − s)
```

com `c` a cor que a tela FICOU e `s` a do RETRATO no mesmo ponto (as duas em
cor directa, desfeita a pré-multiplicação), e `k` a liberdade da máscara pela
porta de sempre.

⭐⭐ **O que o pincel não tocou anula-se POR CONSTRUÇÃO** — os mesmos bytes dos
dois lados dão `c − s = 0` exactamente, e o intacto sai **antes** do raio de
oclusão (gate: uma tela semeada intacta não muda uma amostra e lança **zero**
raios). É isso que deixa o retrato ter erros sem eles aparecerem: um erro dele
só se vê onde o pincel ARRASTA cor, e aí arrasta a cor que o artista via.

⭐ **A diferença leva a RELAÇÃO e não uma cor absoluta**, e é isso que serve os
modos de mistura: escurecer a tela escurece a peça (o `Multiply`, a aquarela).
⭐ **E o detalhe fino debaixo sobrevive**: a diferença soma-se à base da amostra,
não a substitui.

⚠️ **A pintura simples continua na lei da etapa 1** (o «over» sobre a tela
transparente, exacto): com a tela semeada uma tinta OPACA deixaria passar o
detalhe mais fino do que um píxel (`base − s`). A pergunta está na ferramenta
(`PainterTool::screen_canvas_reads_the_piece`), e é pela NEGATIVA: só `Paint` +
`Digital` + `Mix` (e a borracha) dispensam o retrato — *um modo novo nasce a ler
a peça, que é o lado seguro*.

## §3 — O retrato é um rasterizador de CPU

Faces de frente, um quad partido em dois triângulos, centros de píxel em
`i + 0,5`, profundidade `1/w` (linear no ecrã), baricêntricas **corrigidas pela
perspectiva**, e a cor pela porta que o renderizador e o oráculo partilham
(`Tinta::cor_tri`/`cor_quad`) — ou da cor por vértice sem plano. Fora da
silhueta, transparente.

Porque CPU e não uma leitura da placa: é **gateável sem adaptador**, a costura
**não precisa de um `wgpu::Device`**, e sai no mesmo espaço de cor das amostras.

**O preço, medido** (`diag_o_preco_do_retrato`, `--release`, `load 7`, vista
`1400×900`, `463 k` píxeis na peça): **`9,5`–`10,8 ms`** sem plano e
**`17,6`–`21,1 ms`** com o plano de tinta fina, praticamente independente dos
vértices (`32 k` e `130 k`). Uma vez por traço, e só nos modos que lêem ⇒ cerca
de um quadro no pen-down.

⚠️ **A drenagem do retrato deita-se FORA** logo a seguir a semear: semear marca a
tela inteira como mudada, e o retrato não é uma mudança — pousá-lo varreria a
peça no quadro seguinte para concluir que nada mudou.

## §4 — Os modos, medidos no motor do Painter (sondas versionadas)

`diag_cada_modo_sobre_a_tela_semeada` + `diag_clone_e_inpaint_sobre_a_tela_semeada`,
o mesmo traço sobre um retrato meio vermelho meio azul:

| modo | píxeis mudados |
|---|---|
| `smear` | 148 |
| `blur` | 242 |
| `liquify` | 907 |
| `fill` | 1 536 |
| `clone` (fonte armada + traço) | 500 |
| `inpaint` (sobre um defeito) | 216 |
| `Watercolor` | 358 |
| `Wet Paint` | 316 |

⚠️ Duas leituras a zero que NÃO eram defeito e ficam escritas para ninguém as
perseguir: o `clone` sem a fonte armada (o 1.º gesto só escolhe a fonte — no
produto, pelo botão `Set Source` do painel) e o `inpaint` sobre um retrato sem
defeito (a reconstrução devolve os mesmos píxeis).

⚠️ `diag_o_que_muda_depois_do_pen_up`: **zero** píxeis mudam na drenagem depois
do pen-up em `Digital`, `Watercolor` e `Wet Paint` — o traço na peça fecha no
pen-up e nada se perde. ⚠️ Medido no motor, **não** no app.

## §5 — Gates

| onde | novos | o que afirmam |
|---|---|---|
| `ph2d-sculpt3d/tela_semente_tests.rs` | 13 | o retrato: enche a vista · transparente fora · a superfície mais PERTO ganha (a ordem das faces não decide) · costas fora · resolução da TINTA · **perspectiva** (um plano inclinado: o meio do objecto lê a cor do meio) — e a lei da diferença: intacto = ao bit e zero raios · o mudado chega · escurecer escurece · apagado não arranca · idempotente · máscara · tinta fina |
| `ph2d-tool-painter/screen_canvas_tests.rs` | 3 | quem lê a peça (com o CONTROLO de volta ao pincel) · semear só com tela presa e tamanho certo · **um borrão do motor dele arrasta a cor do retrato** |
| `ph2d-app-sculpt3d/tinta_no_produto_painter.rs` | 1 `#[ignore]` + placa | um borrão do Painter sobre a peça leva o vermelho a amostras que não o tinham, e um `Ctrl+Z` desfaz SÓ o borrão, ao bit |
| `painter_fiacao_tests.rs` | +4 elos (12) | semear · a sessão recebe o retrato · a drenagem do retrato deita-se fora · os modificadores chegam ao Painter sobre a peça |

⚠️ O gate da perspectiva existe porque **nas fixturas ortográficas a correcção
é invisível** (`w` constante): sem ele a mutação que a apaga sobreviveria por
falta de corpus — e *uma sobrevivente por falta de corpus constrói-se*.

### §5.1 — Prova de mutação: `35 de 35` à PRIMEIRA corrida, e o controlo sobrevive

O arnês da etapa 1 (`docs/3D/ferramentas/muta_o_painter_na_peca.sh`) cresceu
de 24 para **36** âncoras (pré-voo `36 de 36`): a lei da diferença (`L10`–`L12`),
o retrato (`S1`–`S3`), a pergunta e a porta do Painter (`T7`, `T8`) e os quatro
elos novos da costura (`P9`–`P12`). As 23 da etapa 1 foram **re-corridas
inteiras** — o diff toca na lei que elas medem, e *um placar herdado é um placar
sobre outra árvore*. ⚠️ A âncora `L3` mudou de indentação (o «over» foi para
dentro de um `match`) e o pré-voo apanhou-a antes de correr um teste.

⭐ Diferente da etapa 1, nenhuma sobreviveu, e a razão é de DESENHO: as duas
sobreviventes de lá ensinaram que *um gate numa de duas cópias de uma lei afirma
sobre metade dela* ⇒ a diferença foi escrita em funções que os DOIS caminhos
chamam (`leitura` e `pousa`), e o que é por caminho (a máscara, a base) já tinha
as irmãs da etapa 1. ⚠️ Os gates da diferença correm quase todos no caminho dos
VÉRTICES e um no das amostras — o que os cobre nos dois é a lei ser uma só, não
os gates serem duplicados. E o da perspectiva nasceu antes de a `S3` o pedir.

### §5.2 — O gate de produto, o CONTROLO dele e a foto

⛔ **A 1.ª redacção do gate do borrão reprovou sobre um borrão CERTO:** ela pedia
o canal vermelho a SUBIR, e a peça da cena é BRANCA (`DEFAULT_COLOR = [1,1,1]`) —
sobre o branco o vermelho não tem para onde subir; avermelhar é o verde e o azul
DESCEREM. *Uma régua de cor escrita sem olhar a cor de fundo mede a própria
suposição.* ⭐ E o CONTROLO foi corrido na placa, uma vez: com a semeadura
desligada (`if false && …`) o gate reprova com a mensagem do avermelhar — ele
mede o retrato, não a sorte.

A sonda `diag_fotografa_a_pincelada_do_painter` ganhou três borrões a descer pelas
três ondas da etapa 1: na foto o vermelho é ARRASTADO para baixo, em três colunas,
ao longo da superfície.

## §6 — ⚠️ Limites desta etapa, ditos

* **Os modos agem à resolução do ECRÃ:** um borrão arrasta píxeis, e o detalhe
  fino mais pequeno do que um píxel fica onde estava. Para agir mais fino,
  aproxima-se a vista.
* **Outras peças não tapam** (a oclusão e o retrato são da peça activa) — o
  mesmo limite da etapa 1.
* **A borracha não faz nada na peça** (um píxel apagado não tem cor que se
  compare). O que «apagar» quer dizer numa peça pintada — voltar ao barro? — é
  pergunta do dono.
* **O balde:** o ajuste do limiar DEPOIS de largar (o modal) não chega à peça — o
  traço fecha no pen-up; o ajuste com o botão PRESO chega.
* **Etapa 3:** o relevo do impasto e o que a tinta molhada ainda faça depois do
  pen-up no app · **etapa 4:** camadas e efeitos.

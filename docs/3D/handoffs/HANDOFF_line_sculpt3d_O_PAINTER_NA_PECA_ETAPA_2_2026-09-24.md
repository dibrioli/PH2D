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

## §7 — O smoke da etapa 2 devolveu dois reports, e os dois eram da COSTURA

> *«Liquify: o gizmo do pincel tem o tamanho fixo apenas na aparência, mas
> funciona corretamente. Watercolor não fica molhado, corrija.»*

**Contadores:** `PROJECT_SCHEMA` 0 · os três registos 0 · `SCULPT_DOC_VERSION` 0 ·
zero contrato · zero ADR. ⚠️ Toca **uma** visibilidade na `ph2d-tool-painter`
(`wet_session_continues`: `pub(super)` → `pub(crate)`) e deriva `PartialEq` na
`Vista` da `ph2d-sculpt3d`.

### §7.1 — O anel do Liquify

O `Deform` tem um tamanho PRÓPRIO (`deform_size_px`), independente do pincel de
pintura, e a costura dava ao anel o `dab_footprint_px` — o raio do pincel de
PINTURA. O gesto usava o tamanho certo e o anel mentia. ⭐ A escolha já estava
escrita no anel da vista 2D (`painter_bridge_brush_ring`: *«Deform uses its OWN
(round) brush footprint»*) e a costura 3D não a herdou ⇒ porta
`PainterTool::screen_canvas_ring_px`, com o CONTROLO de que no pincel de pintura
o anel continua o do pincel e não se mexe com o tamanho do Liquify.

### §7.2 — ⭐⭐⭐ A aquarela: limpar a tela SECA o papel

O motor da aquarela guarda o papel molhado (`canvas_wet` + a sessão do wash) de
um traço para o seguinte; é isso que faz o segundo traço FUNDIR com o primeiro. A
costura limpava a tela no fim de cada traço, e limpar passa pelo `set_source`, que
chama `reset_transient_edit_state` → `dry_session_now`. O semear da etapa 2 tem a
MESMA consequência. ⇒ **cada traço nascia sobre papel seco**, e o dono via-o
exactamente como descreve.

A cura: **com o papel molhado no pen-up a tela FICA** (a pergunta é
`PainterTool::screen_canvas_is_wet`, que é a MESMA que o pen-down do Painter faz —
`wet_session_continues`, incluindo a guarda de que a tela é o `Arc` que o último
bake produziu), e o traço seguinte **reaproveita-a em vez de semear** se ela ainda
descreve a peça. ⭐ A semente do traço seguinte passa a ser **o que a peça
RECEBEU** (a última drenagem pousada, `painter_ultima`) — a lei da diferença pede
`c − s` com `s` = o que a peça já tem, e com o retrato antigo o traço anterior
somaria duas vezes.

A chave (`TelaMolhada`) é **a vista inteira** (câmera, tamanho, pose), **a peça**
e o **`edits`**. Qualquer diferença volta ao retrato fresco, que seca o papel:
⚠️ **rodar a vista seca a aquarela** — a humidade vive nos píxeis do ECRÃ, e
depois de rodar os píxeis são de outra parte da superfície. É um limite
DECLARADO, não um defeito.

⚠️ **A pintura simples começa numa tela TRANSPARENTE**, logo quando a tela ficou
molhada o pen-down dela limpa-a (e deita a drenagem fora, como a do retrato).

### §7.3 — ⛔ Um contador do HISTÓRICO foi construído e RETIRADO por mutação

A 1.ª redacção da chave levava um `historia: u64` subido em `record_for` e no
`step` do desfazer, «porque um traço de tinta fina não mexe num vértice». As duas
mutações que o apagavam **SOBREVIVERAM** ao gate de produto — e o gate tem as duas
metades para que ele existia (um `Ctrl+Z` entre os traços; um traço do pincel de
pintura da escultura na tinta fina entre os traços). Medido: todo braço do
desfazer que mexe na peça já sobe o `edits`, e o fecho de um traço de tinta fina
também (ele devolve a cor grossa). ⇒ **retirado**; *uma linha que a mutação não
consegue matar não é lei, é comentário com sintaxe de código.* A recusa está
escrita no doc do `TelaMolhada`.

⚠️ A cerca do `objeto` na chave **não tem fixtura** (seria uma cena de duas peças
com a mesma pose); fica como guarda declarada.

### §7.4 — Gates e prova

| onde | novos | o que afirmam |
|---|---|---|
| `screen_canvas_tests.rs` | 2 | o anel do Liquify segue o tamanho dele (com o controlo do pincel) · a aquarela fica molhada até a tela ser limpa ou semeada (com o controlo Digital) |
| `tinta_no_produto_painter.rs` | 1 `#[ignore]` + placa | o 2.º traço reaproveita a tela molhada · e NÃO a reaproveita depois de um `Ctrl+Z`, de um traço de outro pincel na tinta fina, nem de a vista rodar |
| `painter_fiacao_tests.rs` | +5 elos (17) | o anel · a pergunta «molhado?» duas vezes · guardar · a última drenagem · largar a molhada quando a tela renasce ou se solta; P6 e P11 passam a `2` (a limpeza da pintura simples) |

⭐ **O arnês versionado ganhou uma população de PRODUTO** (`MUTA_PRODUTO=1`, com
`PH2D_GPU=1` na porta) — e ABORTA se a saída disser que não há placa, porque o
`gpu_or_skip!` devolve cedo e o teste PASSA, o que se leria como «a mutação
sobreviveu». ⚠️ **E o pré-voo fez o trabalho dele:** P6, P10 e P11 casavam `2`, `0`
e `2` vezes depois desta mudança — apanhados em segundos, sem correr um teste.

**Mutação: 48 de 48 sangram, controlo a sobreviver** (duas fatias: 17 novas e
re-ancoradas com a placa, 32 das antigas re-corridas porque o diff toca nos
ficheiros que elas medem). Portão: `nextest-impacted` **18 677/18 677** · censos
da árvore COMBINADA **127/127** (controlo do filtro 12 de 12) · clippy
`-D warnings` zero · gates de produto do Painter na peça com placa **5/5**.

### §7.5 — ⏳ O que fica, dito

* O Wet Paint tem uma simulação que ANDA depois do pen-up (40 Hz); o que ela mudar
  com a sessão fechada é drenado e deitado fora, e entra na peça no traço
  seguinte (a semente é o que a peça recebeu). O dono não reportou isto; é a
  etapa 3.
* O ajuste do balde DEPOIS de largar continua sem chegar à peça na hora — com a
  tela molhada guardada ele passaria a chegar no traço seguinte; só com a
  aquarela, e o dono não o vai encontrar por acaso.

## §8 — ⭐⭐⭐⭐ «A resolução de 16x não chega para o Painter»: os degraus `32x`…`256x`, com o tecto do DISPOSITIVO e o preço MEDIDO

Report do dono (24/09, depois do smoke das duas correcções da §7): *«a resolução de
16x não chega para o painter. Demais implementações SMOKE OK.»*

### §8.1 — A necessidade, medida antes do número

Sonda `diag_pixeis_por_amostra_da_tinta_fina` (cena `=52`, vista de fábrica): a
`16x` a tinta fina tem **uma amostra a cada `1,9`–`3,6` píxeis de ecrã** — o traço
do Painter é mais fino que a tinta que o recebe. A `64x` ela é sub-píxel em todo o
lado. ⇒ o tecto antigo (`NIVEL_MAX = 4`) tinha sido medido **na peça errada**
(uma densa, onde `16x` já era sub-píxel) — §0.0: quem move o número reconfere a nota.

### §8.2 — O tecto é de DOIS recursos, e os dois são LEI

1. **O índice de 32 bits** (`ph2d-mesh-colors`): o plano endereça amostras em `u32`.
   `Topologia::nova` conta o total **antes** de alocar (`conta`, `u64`, a mesma
   fórmula que a alocação usa — gate `a_conta_prevista_e_a_conta_alocada`) e **desce**
   ao maior degrau que cabe; `regraduada` devolve `None` acima dele. A `Tinta` toma o
   nível que a topologia DEU, nunca o pedido.
2. **O buffer do dispositivo** (`ph2d-app-sculpt3d`): `12 bytes` por amostra num
   storage buffer ⇒ `orcamento_da_placa = min(max_storage_buffer_binding_size,
   max_buffer_size) / 12` (a RTX 5060 Ti desta máquina: `4 GiB` ⇒ ~`357 M` amostras).
   `degrau_que_cabe` desce ao maior degrau que cabe, reaproveitando a topologia que
   já existe (o 2.º quadro **não** reconstrói — gate
   `o_plano_descido_nao_e_refeito_no_quadro_seguinte`), e o slot **diz ao artista**
   (`app.sculpt3d.tinta_fina.nao_cabe_na_placa`) e **acerta a fileira** para o degrau
   que ficou. ⛔ *O caminho lento não define o produto*: o tecto é do dispositivo.

A fileira `Paint Detail` passa a `Mesh · 2x · 4x · 8x · 16x · 32x · 64x · 128x · 256x`
(`NIVEL_MAX = 8`).

### §8.3 — ⭐⭐⭐ O preço de pintar a `256x` era a OCLUSÃO, e a cura é o PÍXEL

Sonda `diag_o_preco_de_pintar_em_cada_degrau` (`--profile smoke`, `load ~4–5`),
pen-down / pior quadro de um traço:

| degrau | amostras | antes | + oclusão por píxel | + blocos (shipa) |
|---|---|---|---|---|
| `16x` | `188 418` | — | `2,6` / `1,2 ms` | `2,2` / `1,0 ms` |
| `64x` | `3,0 M` | — | `9,7` / `2,4` | `7,5` / `2,1` |
| `128x` | `12,1 M` | `62,8` / `12,1` | `12,7` / `6,8` | `8,6` / `3,5` |
| `256x` | `48,2 M` | **`247`** / **`45,7`** | `28,2` / `21,8` | **`15,0`** / **`9,1`** |

A sonda `diag_de_que_e_feito_o_pen_down` pô-lo no sítio: a 1.ª drenagem custava
`232,7 ms`, quase tudo **raios de oclusão por AMOSTRA** — a `256x` há `~70` amostras
por píxel de ecrã, e cada uma pagava o seu. Duas curas, as duas em
`ph2d-sculpt3d/tela_na_malha*`:

* **A oclusão decide-se por (face, PÍXEL)** (`TelaNaMalha::ve_se_no_pixel`, cache de
  mapeamento directo, uma entrada por píxel da vista). ⚠️ A chave inclui a **FACE**:
  dois lados de uma dobra no mesmo píxel têm vereditos opostos — uma colisão custa
  um raio, nunca um veredito errado (mutação L15 sangra no gate da placa).
* **Blocos de `16×16` da retícula de um QUAD que a caixa do traço não alcança nem se
  projectam**: um retalho bilinear fica no fecho convexo dos quatro cantos e a
  projecção perspectiva preserva-o para pontos à frente da câmera ⇒ a caixa dos
  quatro cantos projectados LIMITA o bloco. Os triângulos (pólos) não se partem.

⚠️ **Os blocos eram INOBSERVÁVEIS**: pintam o MESMO com e sem eles, e a mutação que os
desliga SOBREVIVEU ao gate de equivalência ⇒ sonda de custo `TelaNaMalha::projetadas`
e o gate `um_traco_pequeno_projecta_a_pegada_e_nao_a_face` (com o CONTROLO do
instrumento: `projetadas >= pintadas`, senão um contador mudo passava — L17).

⭐ **A regra «dentro da caixa» é UMA função** (`na_caixa`), lida pelo vértice e pela
amostra: com o corte ela tinha ficado escrita duas vezes.

⚠️ **LOC:** o `tela_na_malha.rs` foi a `748` — curado por **CORTE**: o DEPÓSITO
(a lei por amostra e o percurso da retícula) mudou-se para o filho
`tela_na_malha_pousa.rs` (`522` + `240`). ⏳ **`armar` a `256x` custa `~160 ms` uma
vez** (alocar `48 M` amostras, `579 MB` na CPU e na placa) — é um clique no chip, não
um quadro; não curado.

### §8.4 — Gates e prova

| onde | novos | o que afirmam |
|---|---|---|
| `ph2d-mesh-colors/topo_tests.rs` | 3 | a conta prevista é a alocada · uma malha grande desce ao degrau que cabe no índice (com o controlo de uma pequena) · regraduar acima do índice recusa |
| `tinta_da_peca_placa_tests.rs` | 4 | o degrau desce ao maior que cabe · sem o `2x` não há plano · o plano descido não é refeito · o orçamento é o MENOR tecto / 12 bytes |
| `tela_na_malha_pousa_tests.rs` | 4 | os blocos não mudam o que se pinta (conjunto EXACTO contra a contagem sem atalho) · a oclusão é partilhada no píxel · com tinta fina o escondido não se pinta · um traço pequeno projecta a pegada |
| `tinta_fiacao_tests.rs` | +3 elos (31) | o slot passa o orçamento da placa · diz o degrau que ficou · fala |

**Mutação — versionada nos três arneses:** `muta_o_painter_na_peca.sh` ganhou
**L13–L17** e re-ancorou **L3/L4/L5/L9/L10** no filho (o pré-voo apanhou-as a casar
`0`); `muta_a_metade_visivel.sh` ganhou **M43–M47** e re-ancorou **M2** (casava `2`
depois da segunda saída cedo do `garante`) e **M24** (o tecto é `8`);
`muta_a_cerca_do_plano.sh` ganhou **N10–N12**. Corridos: **11 de 11** · **2 de 2** ·
**5 de 5** · **3 de 3** sangram. Pré-voo dos três: `54/54` · `54/54` · `12/12`.

⚠️ **O portão apanhou um tecto de LOC meu** (`tinta_da_peca.rs` `718` de `700`),
curado por **CORTE**: o recurso do dispositivo (`orcamento_da_placa`,
`degrau_que_cabe`) mudou-se para o filho `tinta_da_peca_placa.rs` (`677` + `53`),
re-exportado pelo pai (o elo do censo da fiação continua a ler
`crate::tinta_da_peca::orcamento_da_placa`); M43/M44 re-ancoradas no filho e
re-corridas, **2 de 2**. ⚠️ **E o roteiro da `=52` ensinava o contrário** (o passo
(4-bis) chamava ao `16x` *«o último da fileira»*): hoje manda carregar `256x` e diz
que numa peça densa a fileira desce sozinha, com o aviso no topo.

**Portão:** `nextest-impacted` **18 688/18 688** · censos da árvore COMBINADA
**127/127** (controlo do filtro 12 de 12) · clippy `-D warnings` zero · gates da tinta
fina e do Painter na peça com placa **84/84** · pré-voo dos três arneses verde.

### §8.5 — ⏳ O que fica, dito

* `armar` o `256x` custa `~160 ms` uma vez (a alocação de `48 M` amostras).
* Os triângulos (os pólos) não se partem em blocos — são poucos; se uma peça de
  triângulos for pintada a `256x`, o custo volta a ser por face inteira.
* A oclusão por píxel pode errar uma amostra cuja borda de oclusor caia DENTRO de
  um píxel — abaixo do que a tela do Painter distingue, por construção.

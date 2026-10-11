# ARQUIVO — parte_3.md (história, 1338 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_3.md`](parte_3.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-472` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §9.14 — UM BLOCO: o tracejado por troço, as mutações nomeadas, o brilho no dispositivo e o `cs_varre` (2026-10-04, escrito ANTES de construir)

Ordem do dono (`CLAUDE.md` §0.10): os quatro itens abertos da família do passe de formas num só plano, um
lote de binários e UMA rodada de medição intercalada. Comparação: o binário da sonda de `4a1c99644`
(`base`). As recusas medidas do §9.4–§9.13 (herdar a bissectriz, costurar as correntes, estreitar a
célula, a variante esparsa) NÃO se reconstroem.

**O inventário (código lido; o (c) medido no app, tela virtual `1930 × 1040`, `PH2D_MOTION_ROUTE_LOG=1`):**

| item | causa | cura | régua |
|---|---|---|---|
| (a) tracejadas grandes esticadas: soma `1,17` ms, parede `1,57` contra `0,90` do Vello (iGPU) | o `cs_escreve` é UM fio por cópia; numa cópia tracejada grande o fio anda os `N` troços × `P` pedaços em série (`72` fios = `2` ondas na placa inteira) | `cs_escreve_grande`: um GRUPO de `64` por cópia tracejada — o arco de cada item em paralelo, o prefixo do arco por sub-caminho e o ajuste numa volta em memória de grupo (a MESMA ordem de soma do pixel a pixel ⇒ os mesmos bits), cada fio conta as arestas dos seus troços, prefixo no grupo, e escreve; a caixa reduz-se no grupo. As cópias vão a ele por um 4.º terço da `contagem` (a marca) com o prefixo no `cs_soma` e um despacho indirecto `[17, 20)` | sonda intercalada, a soma dos passes |
| (b) `M6` | o código que ela mutava (as duas correntes do contorno, `aresta_em`) SAIU em `5febba023` (§9.8); a única corrente que sobra é a dos blocos LOCAIS do preenchimento (`blocos.rs`, `area`), com gate próprio | fecha: o arnês marca-a obsoleta | `git log -S` |
| (b) `S6` | NÃO é «`0,1` px»: sem a cerca, um vértice LISO toma a esquadria INTEIRA, excesso `r·(1/cos(θ/2) − 1) ≈ r·tol/R` — até `~tol` (`0,25` px) quando a meia largura `r` chega ao raio `R` da curva; sobreviveu porque nenhuma fixtura tem `r ≈ R` | gate novo `o_vertice_liso_e_a_junta_redonda`: uma curva esticada com `r ≈ 0,9 R` no ecrã, contra a ÁREA VERDADEIRA (supersamostragem `32²` da união dos troços com discos nos vértices lisos — a lei da casa, «a redonda num ponto liso»), pelos pontos do PRÓPRIO eixo | o gate novo e o arnês da faixa |
| (b) `S8` | a folga da caixa da peça só existe no pixel a pixel (`peca_do_eixo`; o contorno calculado não tem caixa); sem ela a caixa encurta até `0,1` px onde a esquadria de um liso passa de `r` | mede-se no gate da `S6` (mesma fixtura, pixel a pixel) | a diferença máxima de cobertura |
| (c) `=114`, `=115`, `=120` | ⛔ **a premissa caiu:** as formas das QUATRO cenas já são desenhadas pela placa (`[formas] … pela PLACA`, rota da CPU). O que fica na CPU é o COZIMENTO: o contacto do colisor (doc 109, outra família) e uma cena sem estágio que despache (`=120`, por desenho) | fecha por medição | o `[formas]` do app |
| (c) `=70` e o brilho que lê o `pump` anterior | num quadro do dispositivo a bomba da CPU não corre (`motion_state.rs`, `gpu_live`), mas a camada do halo é montada das listas da CPU (`present_fx.rs`, `layer_instances(&pump.instances, &pump.vector_instances, …)`): as sprites são de um quadro VELHO e as formas não existem — por isso a `=70` recusa a placa (`RECUSA_FORMA_COM_BRILHO`). Alcançável hoje: um `fx.glow` com sprites e sem forma vai ao dispositivo e o halo fica congelado | **c1** as sprites do halo saem do buffer do dispositivo (o `draw_scratch` já aceita `gpu_extra`); **c2** as formas do halo saem da CAMADA do passe de formas (já `Rgba16Float`: o HDR do `tint` sobrevive, que era o porquê do tile), desenhada ANTES do passe do brilho e colada depois no mundo como hoje; sai a `RECUSA_FORMA_COM_BRILHO` | gate puro da fonte do halo + a `=70` no app |
| (d) `cs_varre` `0,24` ms nas densas da iGPU | um fio por pixel: `3` leituras atómicas, `5` passos do prefixo com `10` barreiras, as `4` palavras do registo; o `cs_zera` apaga `3` palavras por pixel | **D1** — as famílias PRESENTES na cópia (bits no registo): o `cs_varre` só lê essas e devolve-as a zero depois de as ler, e o `cs_zera` passa a apagar UMA palavra (a cobertura) e o registo; **D2** — `4` pixels por fio (`8` fios por célula): o prefixo de `4` em registos e `3` passos em memória de grupo | ablação `V0` (sem o prefixo) + D1 + D2 |

**Kill-criteria (DIRETIVA §5), um por item:**

- **(a)** iGPU, `load < 4`, `PH2D_FLUID_PROFILE=1`, a SOMA dos passes: nas `72` esticadas tracejadas
  `conta + escreve ≤ 0,22` ms (de `0,40`) e a soma `≤ 1,00` (de `1,17`); nenhum arranjo pior que `+5 %`
  (iGPU) / `+10 %` (RTX), tracejado e contínuo; a imagem igual (os `11` gates GPU da crate, os `5` do
  produto, `gpu-cook formas` `2/2`, as mutações do tracejado `21/21` e da acumulação `19/19`, e uma
  mutação nova do caminho de grupo a sangrar). O limiar de itens decide-se pela ablação `A0` (TODA cópia
  tracejada pelo grupo): se ela não perde para `A` nas densas, o limiar é zero (uma porta só). ⚠️
  **Previsão escrita antes:** o (a) sozinho NÃO põe a parede das tracejadas esticadas abaixo da do Vello
  — as células delas pedem `0,62` ms (as arestas dos traços), e o Vello corta o tracejado na CPU, fora do
  relógio. Se a parede ficar acima, a tabela diz quanto.
- **(b)** `S6`: o gate novo lê o pior desvio da área verdadeira `≤ 0,05` no binário de hoje, com o
  CONTROLO de que a fixtura tem esquadrias de lisos `≥ 0,15` px fora do arco, e a mutação `S6` lê
  `≥ 0,10` (sangra). `S8`: a diferença de cobertura com e sem a folga no pixel a pixel `≤ 1/255` ⇒
  EQUIVALENTE, recusa medida; acima, o gate mata-a. `M6`: fecha pelo `git log`.
- **(c)** a `=70` vai ao dispositivo (`[motion-route]`) com as formas pelo dispositivo (`[formas] … do
  dispositivo`) e o halo das formas e das sprites presente na foto (`PH2D_GLOW_DIAG=1`); um gate puro da
  fonte do halo vermelho antes (num quadro do dispositivo ela lê o `pump`); o censo de rota sobe `19 → 20`
  das `23` cenas com forma; a shell não cresce (`the_shell_only_shrinks`); sem `fx.glow` o quadro é o de
  sempre (o passe não corre).
- **(d)** densas iGPU: a soma `−5 %` (`≥ 0,05` ms de `1,01`); nenhum arranjo pior que `+5 %` (iGPU) /
  `+10 %` (RTX); a imagem igual (os gates e a mutação da acumulação) e, para o D1, uma mutação nova
  (o `cs_varre` que não devolve a zero) a sangrar no gate da cena que muda. Pedaço que não mexe o relógio
  sai (recusa medida aqui).

**A rodada (uma só):** os binários `base` · `A` · `A0` · `D1` · `D2` · `V0` · `F` (`A + D1 + D2`),
compilados de seguida e medidos intercalados por `mede_sonda_das_estrelas.sh` (`PERFIL=1`,
`PLACAS="igpu rtx"`, `CORRIDAS=2`) com `PH2D_SONDA_TRACEJADO=0` e `=1`; `registos_dos_shaders.sh` sobre
`base` e `F`; no app, `mede_formas_na_placa.sh` com o `F` (`=127` densa contínua e tracejada, as duas
placas). Os binários das ablações (`V0`) e dos pedaços sozinhos não vão a commit.

**O resultado da rodada (2026-10-04)** — `mede_sonda_das_estrelas.sh`, `PERFIL=1`, `2` corridas por célula
(lidas lado a lado, iguais ao `0,01` salvo nota), a SOMA dos passes da 2.ª janela do perfilador (o
regime), ms; a carga `ANTES` de cada corrida `≤ 4` (a maioria `0,2`–`3,5`). A rodada partiu-se em
três por dois motivos MEDIDOS: (1) a placa presa `25` min por um arnês de outra linha deixou células
VAZIAS e a 1.ª rodada seguiu com buracos — as duas ferramentas passaram a repetir a célula vazia
(`REPETE`, `556e2f261` · `97790b57f`); (2) as densas da iGPU decidiram o (d) e tiraram o D1/D2/F do
resto (a série que o resultado de um passo justifica, §0.10).

**(a) ⛔ RECUSADO — o passe de GRUPO por cópia tracejada** (`cs_escreve_grande`, construído e medido em
`90672dcb2`, retirado em `f76f0212b`):

| tracejado, soma | base | `A` (grupo, `≥ 24` itens) | `A0` (todas pelo grupo) |
|---|---:|---:|---:|
| iGPU `72` esticadas | `1,20` · `1,18` | **`1,27` · `1,27`** (`+6 %`) | `1,27` · `1,27` |
| — `conta + escreve` | `0,42` · `0,40` | **`0,49`** | `0,49` |
| iGPU `72` conformes | `0,55` · `0,55` | `0,56` · `0,56` | `0,56` |
| iGPU `1 225` densas | `1,33` · `1,33` | `1,33` (nenhuma chega ao limiar) | **`2,89`** |
| RTX `72` esticadas | `0,37` · `0,37` | **`0,19` · `0,19`** (`−49 %`) | `0,19` |
| RTX `72` conformes | `0,09` | `0,09` | `0,09` |
| RTX `1 225` densas | `0,12` · `0,12` | `0,14` · `0,14` | `0,17` |

A imagem era a MESMA (o gate do tracejado pelas duas escritas: as `6` famílias esticadas `40/40` pelo
grupo, alfa `≤ 1`, `0` px `> 1`). Mas o critério era o do proxy de telemóvel, e lá o grupo PERDE: a
escrita em série de `72` fios não é o que custa na iGPU — `64` fios por cópia, a contagem e a escrita
pela emissão duas vezes, a fase em série do fio `0` sobre a memória de grupo e `13 KB` de memória de
grupo por grupo (`128` VGPRs da variante completa) custam mais do que a série que tiram. Na RTX, onde
`72` fios deixavam a placa vazia, o mesmo código corta metade. ⇒ **a alavanca da escrita tracejada no
proxy de telemóvel NÃO é a topologia por troço**; quem voltar a isto mede primeiro, por ablação, a
emissão dupla contra a fase em série (as duas perguntas que esta rodada não separou), e não reconstrói
o grupo. Parede da sonda (iGPU, esticadas tracejadas): placa `1,58`–`1,76` contra Vello `0,90`–`1,15`
— a perda para o Vello com o tracejado já cortado fora do relógio continua; no produto a alternativa é
`5×` mais lenta (§9.13).

**(d) ⛔ RECUSADOS — D1 (as famílias presentes) e D2 (`4` pixels por fio)** (`90672dcb2`, retirados em
`d770fcfd9`); e a ablação que mede a alavanca (iGPU, contínuo):

| soma | base | D1 | D2 | F (`A + D1 + D2`) | `V0` (sem o prefixo) |
|---|---:|---:|---:|---:|---:|
| `72` esticadas | `0,88` · `0,90` | `0,92` · `0,93` | **`1,08` · `1,04`** | `1,07` · `1,06` | `0,88` · `0,87` |
| `72` conformes | `0,63` · `0,61` | **`0,75` · `0,68`** | `0,76` · `0,76` | `0,76` | `0,58` · `0,65` |
| `1 225` densas | `1,03` · `1,01` | `0,98` · `0,99` (`−4 %`) | `1,03` · `1,05` | `1,11` · `1,10` | **`0,93` · `0,93`** |

⇒ o prefixo do `cs_varre` vale no MÁXIMO `0,10` ms nas densas (`10 %`, o tecto `V0`) e nada nas
esticadas; o D1 tira `0,04` nas densas (o critério pedia `0,05`) e põe `+0,05`–`0,12` nas conformes; o D2
não ganha nas densas e perde `+0,13`–`0,18` nas outras (menos fios, mais trabalho em série por fio e a
mesma memória). O que sobra no `cs_varre` é a memória (as `3` leituras e a escrita por pixel), não o
prefixo. A alavanca restante, com tecto medido de `0,10` ms: o prefixo por operações de SUBGRUPO
(`Features::SUBGROUP`, que o `ph2d-gpu` não pede hoje) — não construída: `≤ 10 %` de um passe de `1 ms`
numa cena que já corre a `60 fps`.

**RTX, contínuo** (`A` só acrescenta o 4.º prefixo e um despacho vazio): esticadas `0,22` = `0,22`,
conformes `0,09`–`0,10` iguais, densas `0,10` = `0,10` (uma corrida do `A` leu `0,12` com a parede a
`2,45` ms — outlier de carga).

**(b) ✅ `M6` · `S6` · `S8` fechadas.** `M6`: o código que ela mutava saiu em `5febba023` (§9.8). `S6` e
`S8`: **equivalentes, com número** — gate de CPU
[`a_esquadria_de_um_vertice_liso_nunca_passa_da_flecha_do_nivel`](../../../../crates/ph2d-shape-gpu/src/eixo_tests.rs):
`126 870` vértices lisos (círculo, elipse, anel × `5` larguras × `45` afins, pelo nível que o shader
escolhe), ZERO esquadrias acima da cerca, a pior a `0,086` px (`0,853` da flecha do nível). A cerca do
recuo e o nível de detalhe já implicam a da folga; a `S8` corta no máximo esse excesso, e só no pixel a
pixel.

**(c) ✅ O brilho no dispositivo** (`5b88822fe`). As formas das `4` cenas já iam à placa (o censo media
o COZIMENTO); o defeito real era o halo: num quadro do dispositivo ele lia as listas da CPU, de um quadro
VELHO. `motion_glow_layer::halo_do_quadro` — as sprites do buffer do cozimento, as formas pela camada do
passe de formas REDESENHADA no RT do halo (`ShapePass::redesenha`, gate `o_redesenho_e_o_mesmo_desenho`:
`0` px diferentes em `3` quadros) — e sai a `RECUSA_FORMA_COM_BRILHO`. Censo de rota: a `=70` passa a
«híbrido» (dispositivo `116` de `128` cenas com saída). Gates puros: `a_device_frame_halo_never_reads_the_stale_cpu_lists`
e `shapes_drawn_by_the_shape_pass_leave_the_tile_path` (cada um com o CONTROLO da rota da CPU); costura da
shell `o_halo_recebe_o_buffer_da_placa_e_a_camada_das_formas` e a ordem «desenha antes do halo».

⛔→✅ **E a foto achou o defeito que nenhum gate via: com o passe de formas ligado o `fx.glow` NÃO
BRILHAVA, em rota nenhuma** (`2c16ead81`). Fotos da `=70` a `1930 × 2000` na tela virtual: com
`PH2D_FORMAS_NA_PLACA=0 PH2D_GPU_COOK=0` (a lei antiga inteira) a forma verde tem um halo largo; com a
placa ligada, NENHUM — pelo redesenho novo E pelo tile antigo (diagnóstico fora de commit que forçava o
tile: a mesma imagem sem halo). Um 2.º diagnóstico (o redesenho limpava o RT do halo a vermelho `4,0`)
provou que o halo era escrito e se perdia depois. **Mecanismo:** o passe de formas força o quadro por
FAIXAS (ADR-0154, `plan.banded |= placa.ativa()`), em que o `game_rt` começa TRANSPARENTE para a faixa
de sprites se colar sobre as de baixo; o `fs_composite` do brilho soma luz com alfa `0` e o tonemap
divide a cor pelo alfa (`tonemap.wgsl`, `straight = rgb / max(a, 1e-6)`, e volta a multiplicar) — a luz
fora das peças saía a ZERO. Sem faixas o `game_rt` é opaco e a luz sobrevivia: por isso o defeito só
existe desde que as formas foram à placa (W2), e é anterior a esta linha. **Cura:** o `fs_composite`
escreve a cobertura `a = min(máx(rgb), 1)` e o composite mistura o alfa `src·(1 − dst) + dst` — sobre o
opaco dá `1` (o quadro de sempre, byte a byte), sobre o transparente a divisão do tonemap devolve a
mesma luz, e a colagem pré-multiplicada da faixa compõe-na como *screen* (nunca escurece). Gate de placa
`the_halo_carries_its_coverage_over_a_transparent_target` (`alfa ≥ máx(rgb)` em todo pixel do halo, com
o CONTROLO do destino opaco: alfa `1` e a cor = o destino + o MESMO halo). O emissivo das sprites, que
usa o mesmo composite, ganha a mesma cura.

**Mutação `9` de `9`** ([arnês](../../ferramentas/mutacao_o_bloco_do_9_14_2026-10-04.py), pré-voo `9/9`, corridas
LIMPAS verdes dos quatro comandos): o redesenho mudo (R1, no gate do redesenho) · o halo sem cobertura,
o composite que guarda o alfa do destino, e o que cobre também o opaco (B1–B3, no gate do brilho — a B3
SOBREVIVEU à 1.ª corrida: com a intensidade de fábrica a cobertura dava `1` em todo o lado; a fixtura
passou a um halo FRACO, com o controlo de que há pixels abaixo de `1`) · o halo do dispositivo que lê as
listas da CPU e as formas da placa pelo tile (H1–H2, gates puros) · a camada que deixa de se desenhar
antes do halo, o halo sem o buffer do dispositivo e sem o redesenho (H3–H5, costura da shell). As
mutações do passe de grupo e do D1/D2 saíram com o código deles; os arneses da acumulação e do tracejado
voltaram às âncoras de antes (o código que mutam é o de `4a1c99644`).

### §9.15 — UM BLOCO: onde está o custo das tracejadas grandes esticadas no proxy de telemóvel (2026-10-04, escrito ANTES de construir)

Ordem do dono (`CLAUDE.md` §0.10): os quatro itens abertos da família — (a) o `cs_escreve` tracejado,
(b) as células das tracejadas, (c) a parede − soma nunca explicada, (d) o prefixo do `cs_varre` por
subgrupo — num plano, um lote de binários e UMA rodada intercalada. Base: `402591a88`. As recusas medidas
do §9.4–§9.14 (o GRUPO por cópia tracejada, o D1/D2 do `cs_varre`, a esparsa, estreitar a célula, herdar
a bissectriz, costurar as correntes) NÃO se reconstroem.

**Dois instrumentos SEM relógio antes do plano** (a sonda passa a imprimir as arestas `reservadas ·
escritas · do contorno` — `arestas_do_ultimo_quadro` — e o relógio de CPU de cada fase do quadro e a
parede de CADA quadro; binário de `402591a88` + os instrumentos, uma corrida por célula, iGPU):

| arranjo | arestas reservadas | escritas | do contorno | reservadas ÷ escritas |
|---|---:|---:|---:|---:|
| `72` esticadas contínuas | `192 960` | `42 624` | `33 984` | `4,5×` |
| `72` esticadas TRACEJADAS | **`561 600`** | **`36 288`** | `27 648` | **`15,5×`** |
| `1 225` densas contínuas | `166 600` | `78 400` | `58 800` | `2,1×` |
| `1 225` densas TRACEJADAS | `607 600` | `127 400` | `107 800` | `4,8×` |
| `72` conformes (as duas) | `18 432` · `15 552` | iguais | `0` | `1×` |

| iGPU, esticadas | parede (média) | parede (MEDIANA) | `gpu-busy(span)` | soma | 1.º quadro cronometrado | CPU `decide + desenha` |
|---|---:|---:|---:|---:|---:|---:|
| contínuas | `0,99` | `0,965` | `0,90` | `0,88` | `3,78` ms | `0,03` |
| TRACEJADAS | `1,59` | **`1,27`** | `1,20` | `1,18` | **`76,1` ms** | `0,04` |

(carga `6,5` na 2.ª tabela — a forma, não o terceiro algarismo; o `span` e a soma são os de sempre.)

**O inventário (código lido):**

| item | causa | cura proposta | régua | já RECUSADO por medição |
|---|---|---|---|---|
| (c) parede − soma `0,40` (tracejado) contra `0,10` | ✅ **medida, não suposta:** o `span` iguala a soma (`1,20` · `1,18`) e a CPU é `0,04` ms nos dois; a diferença é o **1.º quadro cronometrado** — ainda pixel a pixel pela variante COMPLETA (a capacidade medida chega dois quadros depois, `Contorno::colhe`): `76` ms ÷ `250` quadros = `0,30` ms. Na MEDIANA a parede é `span + 0,07` nos dois | **c1** a sonda e a tabela lêem a MEDIANA (a média continua impressa); **c2** o 1.º quadro de uma cena nova MEDE a capacidade bloqueando (a contagem, a leitura e o crescimento antes do quadro), em vez de dois quadros pixel a pixel | os primeiros quadros da sonda; a mediana | — |
| (b) células `0,63` contra `0,49` | ⛔ **a premissa caiu:** as tracejadas ESCREVEM MENOS arestas (`36 288` contra `42 624`); o que sobe é a RESERVA — o `cs_deposita` corre UM fio por aresta RESERVADA (`despacha(3u, acc)` com o total da contagem), e cada fio morto faz a busca binária da cópia e sai. A reserva do tracejado conta, POR PEÇA, a junta inteira (`max(4, leque)`) | **B1** o `cs_deposita` só sobre as arestas ESCRITAS: um prefixo das escritas por cópia (4.º terço da `contagem`, `cs_soma_escritas` de um grupo depois do `cs_escreve`, que escreve o despacho `[3, 6)`); **B2** a reserva com a junta UMA vez por troço (só a peça que passa do fim do troço liga ao seguinte — prova abaixo) — o buffer de arestas cresce à potência de dois da RESERVA, e na `=127` densa tracejada do app ninguém a mediu | a soma; as arestas reservadas (instrumento); a capacidade de arestas no app (o `[formas]` passa a dizê-la) | emitir menos arestas por pedaço (pontas partilhadas): as tracejadas já escrevem menos — sai como RECUSA medida se a ablação `P0` não tirar `≥ 0,05` ms |
| (a) `conta + escreve` `0,40` contra `0,24` | UM fio por cópia (`72` fios) anda o eixo TRÊS vezes na escrita: o `ajuste_do_tracejado` (todos os itens), o `sub_tracejado` de cada fechado (o total do sub-caminho, só para a emenda do 1.º traço) e o percurso que emite | **A1a** o ajuste sai da CONTAGEM, que já anda o eixo e já calcula o `arco` de cada troço (a mesma soma, na mesma ordem), guardado por cópia (5.º terço da `contagem`); **A1b** o total do fechado sai do PRÓPRIO percurso (o `s0` acumulado é a mesma soma, termo a termo) e o 1.º traço do fechado — o único que lê a emenda — emite-se no FIM do sub-caminho (a ordem das arestas não importa a ninguém, §9.8) | `conta + escreve` e a soma; ablações `E1` sem emitir pedaços · `E2` ajuste `= 1` · `E3` sem gravar arestas (só o cursor) · `C1` contagem sem o arco (a corda) | o GRUPO de `64` por cópia (iGPU `+6 %`) — não se reconstrói |
| (d) o prefixo do `cs_varre` | `5` passos com `10` barreiras em memória de grupo; tecto medido `0,10` ms nas densas da iGPU (`V0`, §9.14) | **D** o prefixo SEGMENTADO por operações de subgrupo (`subgroupInclusiveAdd` + a correção na fronteira da célula; um `workgroupBarrier` para subgrupos `< 32`), num módulo à parte só onde o dispositivo tem `Features::SUBGROUP` — o `ph2d-gpu` passa a pedi-la quando o adaptador a anuncia (aditivo); sem ela, o passe de sempre | a soma das densas; `V0` na MESMA janela | D1 (famílias presentes) e D2 (`4` px por fio) |

**A prova do B2** (o tecto da reserva continua um tecto): num troço, as peças `n ∈ [n0, n1]` têm traços
disjuntos `[n·per, n·per + tr]`; a junta só se emite em `emite_pedaco` com `liga1`, isto é, com o fim do
traço `b > fim` do troço. Se a peça `n` tem `b > fim`, qualquer `n' > n` começa em `n'·per ≥ (n+1)·per >
b > fim` e não é válida ⇒ **no máximo UMA peça por troço emite junta**. As pontas continuam duas por peça
(a de início sem `liga0`, a de fim sem `liga1`). ⇒ `pecas · (4 + 2·tampa) + max(4, leque)` por troço, em
vez de `pecas · (4 + 2·tampa + max(4, leque))`.

**Kill-criteria (DIRETIVA §5), um por pedaço; a régua é a SOMA dos passes na 2.ª janela do perfilador;
iGPU, `load < 4`; nenhum arranjo (contínuo e tracejado) pior que `+5 %` na iGPU nem `+10 %` na RTX:**

- **(a) A1a/A1b:** `conta + escreve` das `72` esticadas tracejadas `≤ 0,30` ms (de `0,40`) com os dois, e
  cada pedaço sozinho tem de tirar `≥ 0,03` ms ou sai; as imagens iguais (os gates GPU da crate, os do
  produto, `gpu-cook formas`, o arnês do tracejado contra o Vello) e uma mutação nova por pedaço a
  sangrar. ⚠️ **Previsão escrita antes:** o percurso que emite é o maior dos três (`E1` dirá quanto), e a
  parede das tracejadas esticadas continua acima da do Vello (`0,90`, que recebe o tracejado cortado fora
  do relógio). As ablações `E1`–`E3`/`C1` não têm critério — decompõem o que fica; se apontarem uma
  alavanca não construída, ela é o passo seguinte, com o número (série justificada, §0.10).
- **(b) B1:** células das esticadas tracejadas `≤ 0,52` ms (de `0,63`) e das densas tracejadas `≤ 0,90`
  (de `1,01`); as contínuas não pioram (previsão: esticadas `< 0,49`); a imagem é a MESMA por construção
  (as mesmas arestas somadas em inteiros) — os gates de sempre e uma mutação nova (o prefixo das escritas
  deslocado de um bloco) a sangrar. **B2:** sem relógio — as reservadas das esticadas tracejadas `≤ 300 000`
  (de `561 600`), nenhuma cópia recusada por falta de reserva em fixtura nenhuma (o gate do tracejado nas
  `7` famílias: `copias_com_contorno = n`), e a capacidade de arestas da `=127` densa tracejada no app
  medida antes e depois. `P0` (o `cs_deposita` sem as pontas de cada peça) é ablação: abaixo de `0,05` ms
  fecha (b) pela recusa das pontas partilhadas.
- **(c) c1:** a mediana da parede fica a `≤ 0,10` ms do `span` nas duas variantes (o que já se leu acima);
  **c2:** o 1.º quadro cronometrado da sonda tracejada na iGPU `≤ 5` ms (de `76`) e o regime igual
  (`± 2 %` na mediana); no app, os primeiros quadros da `=127` densa tracejada medidos antes e depois.
- **(d) D:** densas contínuas da iGPU `−5 %` na soma (`≥ 0,05` ms de `~1,01`; o tecto `V0` é `0,10`); a
  cobertura IGUAL byte a byte à do passe de sempre (gate de placa que corre os dois caminhos sobre as
  mesmas cenas, nas duas placas) e uma mutação nova (a correção da fronteira da célula) a sangrar.

**A rodada (uma só):** os binários `base` · `E1` · `E2` · `E3` · `C1` · `P0` · `V0` (ablações, fora de
commit) · `A1a` · `A1b` · `A1` · `B1` · `B2` · `D` · `F` (todos os pedaços), compilados de seguida a partir
da árvore com os pedaços atrás de constantes (um roteiro de troca com contagem por âncora e reposição),
medidos intercalados por `mede_sonda_das_estrelas.sh` (`PERFIL=1`, `PLACAS="igpu rtx"`, `CORRIDAS=2`,
`SEGUIDAS=2`, `TRACEJADOS="0 1"`) e lidos por `tabela_da_sonda.py`; `registos_dos_shaders.sh` sobre `base` e
`F`; no app, `mede_formas_na_placa.sh` com o `F` (`=127` densa contínua e tracejada, as duas placas).
Depois da tabela: as constantes dos pedaços recusados saem com o código deles, as dos aceites dobram-se.

**O resultado da rodada (2026-10-04) — PARCIAL, parada por ordem do dono às `19:26`.** Os `15` binários
compilados de seguida (`troca_os_pedacos_9_15.py`, `e00efb99d`; código em `e6daad1fa`, todos os pedaços
ligados = `F`), intercalados por `mede_sonda_das_estrelas.sh` (`PERFIL=1`, `2` corridas por célula, a soma
da 2.ª janela, ms; `tabela_da_sonda.py`). ⚠️ Das `16:45` às `19:00` três outras linhas (`components` `216`
comandos pesados, `vector` `148`, `sculpt3d` `96`, lidos no `journalctl`) seguraram a carga em `16`–`30`:
em `2 h 35` só `94` das `360` células. **Medido: a iGPU nas esticadas (contínuas e tracejadas) e nas
conformes contínuas. NÃO medido: as densas da iGPU, as conformes tracejadas (só `base`/`E1`–`E3`), a RTX
inteira, o app, os registos.** Onde uma das duas corridas saiu alta (carga) lê-se a outra, que repete o
`base` ao `0,01`.

| iGPU, soma | `base` | `A1a` | `A1b` | `A1` | `B1` | `B2` | `D` | `c2` | **`F`** | `V0` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| esticadas TRACEJADAS | `1,17`·`1,18` | `1,15` | `1,15` | `1,11`·`1,12` | `1,04`·`1,06` | `1,08`·`1,09` | `1,15` | `1,20` | **`0,95`** (`−19 %`) | `1,12` |
| — `conta + escreve` | `0,40` | `0,37` | `0,37` | `0,34` | `0,42` | `0,40` | `0,40` | `0,40` | `0,36` | `0,40` |
| — células | `0,62`·`0,63` | `0,63` | `0,63` | `0,62` | **`0,47`** | `0,53` | `0,60` | `0,64` | **`0,44`** | `0,57` |
| esticadas contínuas | `0,90` | `0,90` | `0,90` | `0,90` | `0,88` | `0,90` | `0,86` | `0,90` | **`0,85`** (`−6 %`) | `0,85` |
| conformes contínuas | `0,61` | `0,60` | `0,60` | `0,60` | `0,62` (`+2 %`) | `0,60` | `0,58` | `0,60` | `0,60` | `0,56` |

| iGPU, esticadas tracejadas | `base` | `c2` | `F` |
|---|---:|---:|---:|
| parede MEDIANA (Vello `0,90`–`0,95`) | `1,27`·`1,28` | `1,28` | **`1,03`** |
| 1.º quadro cronometrado | **`77,8`**·`78,5` | **`1,6`**·`1,7` | `1,2` |
| o quadro da cena nova (com a criação dos pipelines) | `93`–`95` | `16`–`22` | `22` |
| arestas reservadas · escritas | `561 600` · `36 288` | — | **`285 120`** · `36 288` (B2) |

**As ablações (sobre o `base`, iGPU, esticadas tracejadas):** `E1` sem emitir pedaços `conta + escreve
0,40 → 0,21` (soma `0,82`) — **a EMISSÃO é o custo da escrita**; `E2` ajuste `= 1` `→ 0,35`; `E3` sem gravar
arestas `→ 0,39` (a escrita na memória não custa nada); `C1` contagem pela corda `0,40` (o `arco` é grátis);
`P0` sem as pontas das peças: células `0,62 → 0,56`; `V0` (o tecto do (d)) `1,12`.

**Os veredictos, contra os critérios escritos antes:**

- **(c) c1 ✅** a mediana fica a `0,06`–`0,09` do `span` em todos os binários. **c2 ✅** o 1.º quadro
  cronometrado `77,8 → 1,6` ms (critério `≤ 5`); o regime igual (`1,26`–`1,28` contra `1,28`).
- **(b) B1 ✅** células das esticadas tracejadas `0,62 → 0,47` (critério `≤ 0,52`); contínuas não pioram
  (`0,51 → 0,47`); conformes `+2 %` na soma (dentro dos `5 %`, é o `cs_soma_escritas`). ⏳ o critério das
  densas (`≤ 0,90`) NÃO foi medido. **B2 ✅** reservadas `561 600 → 285 120` (critério `≤ 300 000`), as
  escritas iguais (nenhuma cópia recusada). ⛔ **As pontas partilhadas: RECUSADAS por construção** — `P0`
  vale `0,06` no `base` (acima dos `0,05`), mas a ponta é a aresta que FECHA cada traço: não há traço sem
  ela, e não se partilha com o vizinho, que fica do outro lado de um vão.
- **(a) A1 ⚠️ o critério combinado FALHOU:** `conta + escreve 0,40 → 0,34` (pedia `≤ 0,30`); cada pedaço
  sozinho tira `0,03` (o mínimo pedido). **A previsão confirmou-se e explica o falhanço:** `E1` mostra que a
  emissão é `0,19` dos `0,40`, e o A1 só tira as voltas. O A1 fica no código (o `F` medido) porque a imagem
  é a mesma e nada piora, e retirá-lo pedia outra corrida dos gates, que o dono mandou parar — ⏳ decisão
  registada no handoff §6.7. A alavanca que sobra é a EMISSÃO por peça (a geometria de `emite_pedaco`), não
  a topologia (o GRUPO por cópia continua recusado).
- **(d) D ⚠️ medido só fora das densas:** esticadas contínuas `0,90 → 0,86`, conformes `0,61 → 0,58`
  (`−5 %`), tracejadas `−0,02` — `60`–`80 %` do tecto `V0` em cada arranjo, a imagem igual byte a byte nas
  duas placas (gate). ⏳ o critério (densas `−5 %`) NÃO foi medido.
- **`F`:** a soma das esticadas tracejadas `1,17 → 0,95` (`−19 %`) e a parede mediana `1,27 → 1,03` contra o
  Vello `0,95` (que recebe o tracejado cortado fora do relógio) — quase empatados; contínuas `−6 %`.

### §9.16 — A SONDA INTERCALADA: a rodada do §9.15 em `37 s` em vez de `4`–`5 h` (2026-10-05)

Pedido do dono (05/10): *«não podemos esperar horas»*. **Onde estava o tempo da rodada do §9.15** (medido):
`mede_sonda_das_estrelas.sh` abria UM processo por célula (`1`–`2` s de dispositivo e pipelines para
`~0,3` s de régua), esperava `SEGUIDAS=2` leituras calmas com `20` s entre elas ANTES de cada célula — `360 ×
20 s = 2 h` de espera com a máquina ociosa —, ficou `2 h 35` à espera com três outras linhas a compilar
(`load 16`–`30`), e compilava um binário `release` por variante (`15 × 1,5 min`).

**A régua nova** — `sonda_intercalada` ([`motion_shape_placa_gpu_intercalada_tests.rs`](../../../../crates/ph2d-app-motion/src/motion_shape_placa_gpu_intercalada_tests.rs))
e [`ferramentas/mede_intercalado.sh`](../../ferramentas/mede_intercalado.sh):

- **as variantes são passes do MESMO processo:** os pedaços do §9.15 passaram de `const` a `override`
  (`ShapePass::com_constantes`, `VarianteDoPasse`: as constantes escolhem-se ao criar o pipeline; o subgrupo e
  a medida no início já eram portas de execução) ⇒ UMA compilação (perfil `smoke`, otimizado e incremental:
  o relógio é o da placa, e o nível de otimização do Rust só mexe na CPU);
- **intercaladas em blocos de `20` quadros por ordem rodada** (`7` rodadas): uma carga alheia cai em todas
  as variantes por igual;
- **o resumo é o MÍNIMO das rodadas** (uma interferência só soma tempo), com a mediana ao lado; a régua
  continua a SOMA dos passes do relógio da placa, agora entregue por bloco (`ph2d_gpu::pass_profiler::drain`,
  aditivo no foundational);
- **nenhuma espera de calma.** Medido com `load 20`–`29`: mínimo e mediana iguais ao `0,01`, e o `base`/`F` das
  esticadas tracejadas da iGPU a repetir a rodada de 04/10 (`1,18`/`0,94`–`0,95` contra `1,17`–`1,18`/`0,95`).

| | rodada do §9.15 | sonda intercalada |
|---|---:|---:|
| compilação | `15 ×` release (`~25 min`) | `1 ×` smoke (`~1 min` com mudança de código; `0 s` sem) |
| iGPU, `9` variantes × `6` cenas | — (parada em `94/360` células) | **`11 s`** |
| RTX, idem | — | **`26`–`28 s`** |
| espera de calma | `2 h` (ociosa) + `2 h 35` (carga) | `0` |

⛔→✅ **Na RTX o processo morria com SIGSEGV DEPOIS do `test result: ok`** (o core: uma thread que não a
principal a saltar para `0x170`): a thread do driver a gravar a cache de shaders (`54` passes × `~20`
pipelines) quando a biblioteca já fora descarregada. Com `__GL_SHADER_DISK_CACHE=0` sai limpo (a sonda
antiga, com `1` passe, sai limpa com e sem perfilador). O roteiro desliga a cache SÓ no processo da sonda
— ela só acelera o arranque seguinte, o relógio da placa não a vê.

**O que faltava do §9.15, medido agora** (soma mínima, ms; `F` = todos os pedaços, o produto):

| cena | iGPU `base → F` | RTX `base → F` | o pedaço que mais pesa |
|---|---:|---:|---|
| esticadas contínuas | `0,91 → 0,87` (`−4 %`) | `0,222 → 0,214` (`−4 %`) | `D`, `B1` |
| esticadas TRACEJADAS | `1,18 → 0,95` (`−20 %`) | `0,367 → 0,297` (`−19 %`) | `B1` `−12 %` · `B2` `−9 %` · `A1` `−6 %` (RTX `−15 %`) |
| conformes contínuas | `0,615 → 0,610` (`−1 %`) | `0,087 → 0,083` (`−5 %`) | `D` |
| conformes tracejadas | `0,557 → 0,545` (`−2 %`) | `0,083 → 0,078` (`−6 %`) | `D` |
| **densas contínuas** | **`1,02 → 0,85` (`−16 %`)** | `0,099 → 0,085` (`−14 %`) | `B1` `−11 %` · `D` `−5 %` (RTX `D` `−13 %`) |
| **densas TRACEJADAS** | **`1,32 → 0,95` (`−28 %`)** | `0,121 → 0,097` (`−20 %`) | `B1` `−21 %` · `B2` `−12 %` |

**Os critérios do §9.15 que ficaram por medir, agora fechados:** **B1 ✅** células das densas tracejadas
`1,00 → 0,71` (critério `≤ 0,90`); **D ✅** densas contínuas da iGPU `−5,3 %` (critério `−5 %`); **RTX ✅**
nenhum arranjo pior que `+10 %` — o pior pedaço sozinho é o `B1` nas conformes (`+6,4 %`, o
`cs_soma_escritas` num passe de `0,09` ms), e o `F` melhora em TODAS as cenas das duas placas. Nas conformes
o `B1` sozinho custa `+3 %` na iGPU e o `D` devolve-o: o `F` fica a `−1`/`−2 %`. ⏳ Sem medida ainda: o app
(`=127` e a memória das arestas que o `B2` corta) e os registos — a próxima onda corre-os já pela régua nova.


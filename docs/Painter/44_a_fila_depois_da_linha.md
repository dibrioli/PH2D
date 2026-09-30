# 44 — A fila depois da `line/PainterWatercolor` (aberta 2026-09-29)

> A linha fechou e foi integrada em 2026-09-25. O §10 do
> [handoff dela](handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_A_LINHA_2026-09-25.md) listava
> dez itens em aberto, cinco deles marcados como «decisão do dono». O dono disse (29/09) que *algumas
> dessas decisões já foram tomadas*. ⇒ **nenhum item se constrói antes de ser CONFERIDO**:
> procura-se a decisão no diário, nos commits e na memória, e o dono confirma.
>
> Estado por item: `a conferir` · `aberto (conferido)` · `decidido` · `feito` · `recusado`.

| # | item | de quem | estado |
|---|---|---|---|
| 1 | Wet Paint: passos 3–4 do transfer em série (soma `f64` · arrasto Gauss-Seidel) | dono | **decidido (29/09): (a), fica como está** — §1 |
| 2 | Aquarela: unificar a mistura seca (RYB) e a molhada (K–M) — a pista do *glaze*, diário §17.3 | linha (a ordem do dono já existia, 20/09) | **feito, smoke do dono OK (29/09)** — §2 |
| 3 | O `Mixing` no Impasto | dono | **decidido (30/09): manter** — §4.1 |
| 4 | O `Pigment` que mudou de sítio na aquarela (do cartão *Water* para o *Mixing*) | dono | **decidido (30/09): fica no *Mixing*** — §4.2 |
| 5 | Composite Brush: a pilha cheia numa tela grande (a alavanca é o RAIO) | dono | **fechado (30/09): premissa morta, medido** — §4.3 |
| 6 | Wet Paint: o *fork* do canvas no 1.º toque depois de soltar (`~9 ms`) — pede canvas em ladrilhos | linha | **fechado (30/09): medido, `~3 ms` de `~9–10`** — §5.1 |
| 7 | Wet Paint: o tile do papel do motor (`~12 ms`, em série por impressão digital) | linha | **feito (30/09): `7,41 → 0,013 ms`** — §5.2 |
| 8 | Composite Brush: o relevo fora da recomposição da pilha | linha | **8a feita (30/09): relevo, filme e Tiling iguais ao avulso ao bit · 8b aberta** — §6 |
| 9 | Composite Brush: o resíduo Blur+Smear (`12/255`) | linha | a conferir |
| 10 | Composite Brush: metade dos bytes dos intermédios da pilha | linha | a conferir |
| 11 | Aquarela: o **Smudge** não mexe na tinta MOLHADA da sessão (report do dono, 29/09) | linha | **feito, smoke do dono OK (29/09)** — §3.1 |
| 12 | Aquarela: o **Rewet** mexe pouco na tinta MOLHADA da sessão (report do dono, 29/09) | linha | **feito, smoke do dono OK (29/09)** — §3.2 |

---

## §1 — Item 1: os passos 3–4 do transfer do Wet Paint

**Onde:** [`trail/transfer.rs`](../../crates/ph2d-wet-paint/src/trail/transfer.rs), `transfer_paint_impl`.

**O que são:**
- **Passo 3:** a média da janela do bico sobre as células com pigmento (`sum_pig`, `sum_water`), somada em `f64` numa ordem fixa. Ela só é lida no *soft cap*, isto é, quando uma célula passa de `3000` (`shed_pig`) ou de `WaterCap` (`shed_water`).
- **Passos 4+5:** pousar a janela e o **arrasto** (`drag`). O arrasto lê `g.susp[si]`, `g.sett[si]`, `g.film[si]` e `g.wet[si]` na posição da âncora ANTERIOR. Esses são os mesmos arrays que o próprio laço escreve.

**Custo registado** (diário §39.3, 2026-09-24): `~1,9 ms` por transfer e `~2` transfers por quadro a raio 250. Isto é cerca de metade dos `~8 ms` do quadro a pintar.

**Conferência — não há decisão.** O que foi procurado:
- no diário da linha, de §39.3 até §43: o item fica aberto e não volta a ser citado;
- em `git log main --since=2026-09-18 --grep=transfer`: só o `8a05c45cf` (ADR-0175), que paraleliza o depósito e os passos **1–2** e deixa estes dois de fora de propósito;
- em `project-memory/`: nenhuma entrada.

**Leitura do código, ainda NÃO medida:**
- **Passo 3.** Somas parciais por linha, somadas numa ordem fixa, mudam o `f64` no último bit. Esse valor só chega à tinta através de `old + v − shed` guardado em `f32`, com `old ≥ 3000`, onde o ulp do `f32` é `2,4e-4`. Um erro de `~1e-13` no `shed` quase nunca muda o `f32`. ⇒ paralelizar o passo 3 é, provavelmente, uma **decisão técnica**: cabe à linha medir se sai ao bit.
- **Passo 5.** O arrasto é Gauss-Seidel **dependente da direcção do traço**. O laço varre da esquerda para a direita e de cima para baixo. Quando o traço anda para a direita ou para baixo, `si` (a janela anterior) fica ANTES de `i` no varrimento e já foi escrita neste laço. Para a esquerda ou para cima, ainda não foi. ⇒ hoje o arrasto provavelmente puxa a tinta de maneira diferente conforme o sentido do traço. Mudá-lo para uma leitura sem ordem (Jacobi, como fez o ADR-0147 com o solver) muda a tinta. Isso é **produto**.

**Medido (2026-09-29, sonda [`measure_drag_direction`](../../crates/ph2d-wet-paint/tests/it/measure_drag_direction.rs)):**

⛔ **O teste de ESPELHO foi a 1.ª redacção e está RECUSADO.** O mesmo traço espelhado com `Drag = 0`
já diverge `67 %` da mudança que o traço faz, porque o papel e as cerdas têm ruído com semente e não
são simétricos. O fundo afoga o sinal: a assimetria até DESCE com o arrasto (`0,67 → 0,59` a
`Drag 0,4`).

⭐ **A régua que decide é a própria lei trocada.** A sonda grava o grid final de um traço horizontal
em cada sentido. Uma cópia temporária do `transfer.rs` lê a origem do arrasto de um instantâneo
tirado antes do laço (Jacobi), a sonda corre outra vez, e o ficheiro é reposto (`git diff` vazio
conferido). A previsão é o que dá o controlo: **para a ESQUERDA as duas leis têm de coincidir**, porque
ali a origem ainda não foi escrita.

| passo/quadro | sentido | Σ\|GS−Jacobi\| / Σ\|mudança\| (susp · sett · film) | pior célula susp | células > 0,1 % | pior RGB | pior `wet` |
|---|---|---|---|---|---|---|
| 3 px | direita | `1,99 %` · `1,18 %` · `1,90 %` | `64,3` | `611` | `3,29` | `7` |
| 3 px | **esquerda (controlo)** | `0,00 %` | `0,0028` | **`0`** | `0` | `0` |
| 12 px | direita | `0,22 %` · `0,22 %` · `0,21 %` | `21,0` | `23` | `0,38` | `5` |
| 12 px | **esquerda (controlo)** | `0,00 %` | `0,0285` | **`0`** | `0` | `0` |

**Leitura:**
- a dependência do sentido **existe** e é **pequena**: `~2 %` da tinta que um traço LENTO move, `~0,2 %` num traço rápido;
- a esquerda é o Jacobi a menos do ruído de `f32` (o resíduo `≤ 0,03` de massa não passa em nenhuma célula), logo **a lei sem ordem é a que o traço para a esquerda já pinta hoje**;
- mudá-la muda a tinta **só nos traços para a direita e para baixo** e só nesta ordem de grandeza.

⏳ **Por medir antes de se construir:** o relógio. O diário dá `~1,9 ms` por transfer e `~2` transfers
por quadro a raio 250, contra `~8 ms` de quadro, e a leitura sem ordem paga uma cópia da janela de
origem por transfer.

⛔ **E o preço que a tabela não mostra:** o Wet Paint é um porte 1:1 com impressão digital
(ADR-0134/0147). Trocar a leitura re-baseia as impressões desse caminho, como o ADR-0147 fez ao
solver, que manteve a rota antiga PREGADA (`PINNED_GAUSS_SEIDEL`).

**Decisão do dono** (§0.8), com os números acima:
- (a) fica como está;
- (b) passa a sem-ordem: os quatro sentidos pintam igual, os dois passos podem dividir-se pelos núcleos, e a tinta muda `≤ 2 %` nos traços lentos para a direita ou para baixo.

**Decidido pelo dono (2026-09-29): (a).** A lei fica como está. A sonda e esta tabela ficam, porque são
a resposta a quem voltar a propor a troca.

---

## §2 — Item 2: a aquarela seca mistura pela lei do Wet Paint

**Conferência — a decisão JÁ EXISTIA.** A ordem do dono de 2026-09-20 é *«trocar as duas para a lei do
Wet Paint (os três meios passam a misturar igual)»* (diário §17, arquivado em
`docs/archive/docs-2026-09-24/painter/`). O Digital trocou, o depósito molhado da aquarela trocou em
24/09, e ficou por fazer o composite da aquarela sobre tinta SECA. A 1.ª tentativa foi revertida
(§17.3). ⇒ o item era trabalho, não decisão.

**§0.0 — a premissa da recusa foi reconferida e não caiu.** A presença de tinta (24/09) tirou o
papel da mistura do BOTÃO, mas o `watercolor_soak_…` continua a ler o MESMO pixel (`228,23,23`) com
o K–M. ⭐ **O mecanismo não é o papel: é o termo da ÁGUA** (`wet × tinta molhada`). O pigmento dele
é o que a própria água dissolveu da base, e no K–M o parceiro mais absorvente domina a mistura, o
que apaga o clarear do soak.

**A lei que fica** ([`alvo_sobre_seco`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_mistura.rs)):
- o **botão `Pigment`** (misturar duas tintas) passa ao K–M;
- a **água** continua RYB;
- o K–M manda só na parte que o botão pede **além** da água: `(botão − água)⁺ / botão`.

⛔ A 1.ª redacção partilhava por `botão/(botão+água)` e partiu o
`watercolor_wet_drives_the_paint_mix_without_pigment`, que guarda a lei do dono de 2026-07-06: *com
`wet = 1` o botão não muda nada*. A partilha pelo EXCESSO honra-a ao bit.

**Medido** (`diag_pigment_molhado_sobre_molhado`, amarelo sobre azul, `Pigment` ligado, o meio):

| | seco | molhado |
|---|---|---|
| antes | `119,209,228` (ciano) | `159,198,159` |
| depois | **`128,173,139`** (verde) | `159,198,159` |

**Gates:**
- `seco_e_molhado_dao_o_mesmo_tom` (barra `45`, do vale medido: `31` depois, `69` antes);
- `a_porta_das_duas_leis_tem_as_pontas_ao_bit`.

**Mutação:** 4 de 4 sangram (sempre-RYB · partilha pela soma · sempre-K–M · sem o ramo do RYB). Mais
duas equivalentes, nomeadas: um `.max(0)` redundante, que foi **apagado**, e `<=` por `<`, que dá o
RYB ao bit em `0`.

**Suítes:** `ph2d-tool-painter` + `ph2d-painter-brush` + `ph2d-pigment` **1 829/1 829**.

⏳ **Por smokar pelo dono.**

### §2.1 — O smoke: *«reduzindo Charge para < 1 não se percebe a mistura»* (2026-09-29)

**Medido** (`diag_pigment_com_charge`, o mesmo amarelo sobre azul, ao longo do `Charge`):
- **seco:** a mistura continua em todos os valores;
- **molhado:** com `Charge < 1` o meio lia o **MESMO pixel com o botão ligado e desligado** (`249,243,149` a `0,9`).

⛔ **A causa é a prioridade do mixer.** `Charge < 1` arma o pincel que apanha cor, e o depósito
escreve com peso `prio = pickup × carga`: só grava o que o pincel APANHOU. O mixer lê a base
CONGELADA, e a tinta molhada da própria sessão não está lá ⇒ ele não apanha nada ⇒ `prio = 0` ⇒
nenhum dos dois traços grava cor. A `deposita` do `Pigment` ficava sem os dois parceiros, e a cor
vinha inteira do DONO do texel.

⭐ **A cura:** no caminho do `Pigment` o peso é a tinta que a brocha LARGA (`depl`), sem a prioridade.
Fora do `Pigment` nada muda.

**Depois:** molhado `159,198,161` (`0,9`) · `160,199,164` (`0,75`) · `162,200,171` (`0,5`) ·
`168,203,181` (`0,25`), contra `159,198,159` a `1`.

**Gate:** `o_charge_abaixo_de_um_nao_desliga_o_pigment_molhado`. Mutação `1 de 1` (repor a
prioridade no peso sangra este gate e só ele).

⚠️ **Nomeado, e não é defeito:** sobre PAPEL com `Charge < 1` e o botão ligado o amarelo sozinho
muda até `7` num canal (`253,245,128 → 252,244,121`). Não é mistura com o papel. É o depósito a
passar a gravar a cor da brocha onde antes a cor vinha do dono SUAVIZADO na junção com o azul, e fica
igual ao que o `Charge 1` já fazia (`253,244,119`). O `o_pigment_nao_mistura_com_o_papel` corre a
`Charge 1`, onde continua ao bit.

⚠️ `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` reprovou uma vez na suíte a `load ~20`
e passou sozinho: é membro da família de flakes de carga (§5.0).

---

## §3 — Itens 11 e 12: o Smudge e o Rewet sobre tinta MOLHADA (report do dono, 2026-09-29)

> *«Aparentemente o Smudge e o Rewet não afetam a mancha de tinta quando a tinta está molhada.»*

**Medido** (`diag_smudge_e_rewet_sobre_molhado`: faixa azul vertical e traço amarelo horizontal por
cima, knob em `0` e em `1`, `Pigment` desligado):

| | texels mudados · soma \|Δ\| | o meio da faixa |
|---|---|---|
| Smudge 1, azul SECO | `728` · `84 258` | `190,202,138 → 253,245,140` |
| **Smudge 1, azul MOLHADO** | **`0` · `0`** | igual |
| Rewet 1, azul SECO | `3 588` · `152 402` | `190,202,138 → 214,229,253` (o azul sobe) |
| **Rewet 1, azul MOLHADO** | `3 901` · `59 007` | `253,245,140 → 253,247,167` (mal se vê) |

**Causa, lida no código (a confirmar pela cura):**
- o **Smudge** arrasta a BASE (a tinta seca de antes da sessão, `smear_wet_base`) e os NÍVEIS de
  reserva do traço vivo (`smear_level`, só com `Charge < 1`). A tinta molhada da sessão vive noutros
  planos — cobertura, cor, dono, água —, e **nenhum é arrastado**. Sobre ela o Smudge é inerte AO BIT;
- o **Rewet** levanta e dissolve a BASE da sessão (`build_rewet_fields`). A tinta molhada da própria
  sessão entra só pelos campos UNIÃO da água (anel, tinta, lift), e o efeito sai `2,6×` mais fraco.
  ⚠️ A cura óbvia — reassar a sessão na base — está **recusada por escrito** (Enio 2026-07-09: o
  vizinho assado re-renderizava como um RECTÂNGULO que clareia).

**Desenho proposto (por construir):** o Smudge arrasta também os planos da sessão, pela mesma lei do
`smear_dab`. O Rewet passa a ler a tinta da sessão como tinta levantável, via os campos UNIÃO e nunca
reassando a sessão.

### §3.1 — Item 11 feito: o Smudge arrasta a tinta molhada (2026-09-29)

**A lei** ([`watercolor_mistura_arrasto`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_mistura_arrasto.rs)):
a cada dab, ANTES do depósito, a tinta que a sessão tinha antes deste traço (o `antes` da mistura) é
arrastada do dab anterior para este pela MESMA lei de levantar e pesar do Smudge seco; o depósito
mistura a cor nova por cima dela com o peso do Smudge. Três escolhas, cada uma decidida por medição:

| escolha | o que acontecia sem ela (fixtura do gate, pincel de fábrica) |
|---|---|
| arrastar o `antes`, **nunca** o plano da cor | o depósito recompõe cada texel do `antes` e deitava o arrasto fora: fila às RISCAS, texels ao bit o amarelo entre os dabs |
| mistura em alfa **pré-multiplicado** (`smear_dab_premultiplicado`, `ph2d-painter-brush`) | o papel do plano é `0,0,0,0`: o rasto saía `152,152,129` (oliva sujo) contra `219,235,141` |
| fotografar o destino ANTES de o arrastar (flag `capturado` por texel) | com os dabs espaçados (`0,6`) cada texel é arrastado uma vez só, e o rasto abria texels a `0` |

**Resultado** (ganho de azul sobre o mesmo traço sem Smudge, em cada texel do rasto `x 108..150`):
`0` antes · `13`–`44` depois, sem falha; com os dabs espaçados, pior texel `14`.

**Duas coisas que a medição decidiu contra o primeiro desenho:**
- **a prioridade do mixer fica** no depósito do Smudge (sai só com o `Pigment`, §2.1): sem ela a
  poça do Wet Mix (dono, 2026-07-07) mudava com o Smudge ligado, e o
  `watercolor_color_change_junction_is_soft` reprovou;
- **não é preciso recompor os texels sem depósito:** sobre a tinta molhada o mixer apanha sempre, e a
  medição a Charge `1`, `0,5`, `0,326`, `0,2` e `0` não mudou um byte com ou sem essa recomposição.

⚠️ **Com o Smudge a `1` e a queda constante** o arrasto copia o papel do início do traço por cima
de tudo e o azul é APAGADO em vez de arrastado — a mesma lei do Smudge seco. O rasto vê-se com a
queda de fábrica (suave) ou com o Smudge abaixo de `1`.

⏳ **Aberto, medido e PRÉ-EXISTENTE:** com `Charge < 1` o mixer já carrega a cor molhada sozinho, e
o Smudge **tira** cor a esse rasto (soma do ganho a Charge `0`: `−2 279` sem o arrasto molhado,
`−1 899` com ele). Vem do arrasto da RESERVA do mixer (`smear_level`), que é anterior a esta fila.

**Gates:** [`watercolor_arrasto_molhado`](../../crates/ph2d-tool-painter/src/tool/paint/tests/watercolor_arrasto_molhado.rs)
(os do Smudge) + `premultiplied_smear_drags_transparency_without_darkening` (`ph2d-painter-brush`).
Prova de mutação: [`muta_o_arrasto_molhado.sh`](ferramentas/muta_o_arrasto_molhado.sh), **13 de 13 sangram**.
A captura da ORIGEM do arrasto foi escrita e **retirada**: com um taper de `3` diâmetros a saída
ficava ao bit a mesma sem ela (a origem é a pegada do dab anterior, já fotografada).

### §3.2 — Item 12 feito: o Rewet redissolve a tinta molhada (2026-09-29)

**Porque a cura não é no composite:** o Rewet de lá lê a base SECA da sessão de propósito — a tinta
molhada vizinha seria contada duas vezes (o rectângulo que clareia, dono 2026-07-09). A tinta
molhada vive no plano da cor da sessão, e é no DEPÓSITO deste traço que a água a encontra
([`watercolor_mistura_agua`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_mistura_agua.rs)):

- **dissolve** — o parceiro da mistura de cada texel passa a ser a tinta de ANTES do traço borrada
  pelo MESMO raio do Spread que o seco usa (`raio_da_agua`, uma porta com os dois leitores) e pela
  MESMA soma (`box_blur4`, presença e cores pesadas por ela), levada a `Rewet` do caminho, em
  pré-multiplicado. Um borrão por LOTE de dabs; o plano de antes não muda durante o traço, logo o
  resultado não depende da cadência;
- **mistura** — o `deposita` passa os dois pesos pela porta do composite seco (`alvo_sobre_seco`):
  o botão (Pigment, Smudge) pelo K–M, a água do Rewet pela lei da água.

**Resultado, com o SECO como régua** (o lado aprovado): o meio da faixa `253,245,140 → 188,226,174`
(verde: as duas tintas molhadas misturam); o azul espalha-se de `x = 63` a `x = 108`, a MESMA faixa do
Rewet seco, a menos de `8` por texel; o `B − R` longe da faixa lê `−61` contra `−62` do seco.
⚠️ A cor não é a do seco (lá o azul SOBE por uma película amarela fina: `214,229,253`), e é de
propósito: sobre molhado são duas tintas molhadas, e misturam.

**Gates:** `o_rewet_mistura_a_tinta_molhada` · `o_rewet_espalha_a_tinta_molhada_como_espalha_a_seca`
(barra por texel a `12` do seco) · `a_agua_mistura_pela_lei_da_agua_e_o_botao_pela_do_pigmento` ·
`diluir_com_papel_muda_o_alfa_e_nao_a_cor` · a porta nos três estados da sessão. Mutação: o mesmo
arnês, **20 de 20** com o Smudge. ⚠️ A mistura recta no parceiro SOBREVIVIA à fixtura do produto (ali
o papel é `0,0,0,0` e a faixa opaca, e as duas misturas coincidem) — só a orla de alfa parcial as
separa, e é essa a fixtura do gate de unidade.

## §4 — Itens 3, 4 e 5: conferidos (2026-09-29)

### §4.1 — Item 3: o cartão `Mixing` no Impasto

- **Proveniência.** Ninguém pediu. A decisão está escrita no diário arquivado
  ([§18.7](../archive/docs-2026-09-24/painter/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md)):
  *«não foi pedido … decisão do dono, e reverter é uma linha»*. O que existe é o `7489b8385`: a mistura
  de pigmento chega ao Digital e ao Impasto através de `PaintMedia::offers_pigment_mixing`.
- **Funciona.** Foi medido no diário (`|d| 142`).
- **Preço que o dono tem de saber.** Com o `Pigment` acima de zero, o carimbo sai do dispositivo,
  porque `stamp_device::eligible` exige `pigment_mix == 0`. O pincel volta ao caminho da CPU.
- **Decisão do dono (30/09): MANTER.** Retirar seria uma linha em `offers_pigment_mixing`.
- **Recomendação da linha:** manter.

### §4.2 — Item 4: o `Pigment` saiu do cartão *Water* para o *Mixing* (aquarela)

- **Proveniência.** Foi um censo que o forçou
  ([§19.5](../archive/docs-2026-09-24/painter/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md)).
  Não houve uma ordem do dono.
- **O que é hoje.** Nos três meios que o oferecem, o controlo vive no mesmo cartão.
- **Decisão do dono (30/09): o lugar fica** — o `Pigment` vive no cartão *Mixing*.
- **Recomendação da linha:** manter. Um controlo com o mesmo nome em cartões diferentes, conforme o
  meio, é a forma de «não encontro o botão».

### §4.3 — Item 5: a pilha cheia com pincel grande — a premissa MORREU

O §22.6-ter do diário dizia *«raio 96: 181–204 % de um quadro»*. Esse número foi medido antes de
duas curas da própria linha:
- o Blur da pilha passou a ser em caixa (`a586f21c2`);
- a pilha passou a compor **uma vez por quadro** (`set_compor_por_quadro`), em vez de uma vez por
  evento.

**Re-medido pela porta do produto.** Instrumento
[`mede_a_pilha`](../../crates/ph2d-tool-painter/examples/mede_a_pilha.rs). Condições: `--release`,
`load 3,4`, tela 1024², a pilha da foto do dono (7 camadas), raio `~83 px` (`size_norm 0,4`),
drenagem a cada 16 eventos.

| passo | ms do traço (720 px) | ms por quadro |
|---|---|---|
| 2 px | 74,1 | **3,37** |
| 8 px | 49,2 | **9,84** (59 % de 16,7) |

A decomposição no passo 8 foi obtida por ablação, retirando uma camada de cada vez. A pilha cheia
custa `47,5` por quadro (compor `7,7`, acumular `39,8`). As camadas mais caras são:

| camada retirada | Δ por quadro (ms) |
|---|---|
| Blur | 15,4 |
| Smear | 10,4 |
| Erase | 5,7 |

⇒ **a pilha cheia cabe num quadro com folga.** Hoje não há defeito a curar.

⛔ **Continua recusado:** qualquer corte silencioso por orçamento.

⏳ **A alavanca, se um dia for precisa:** o Blur é a camada mais cara.

**Decisão do dono (30/09): FECHADO.**


## §5 — Itens 6 e 7: o custo de o Wet Paint renascer (2026-09-30)

### §5.1 — Item 6: o *fork* da tela — fecha por medição

A sonda [`diag_o_fork_da_tela_a_4096`](../../crates/ph2d-tool-painter/src/plane_copy.rs) mede a cópia
da tela que o 1.º toque depois de soltar paga. Tela 4096², `load 45` (os números são tectos):

| cópia | ms |
|---|---|
| fria (páginas por tocar) | 3,26 |
| quente | 2,56 |

A cópia **já é paralela** e o que sobra dela é o 1.º toque nas páginas. São `~3 ms` de um pen-down
de `~9–10`. Descer daí pede **tela em ladrilhos** (copiar só o que o traço toca), que é arquitectura
e não afinação. Não há defeito a curar hoje.

### §5.2 — Item 7: o papel é feito uma vez por entrada

O tile do papel é função pura de `(preset, folha, knobs)`, e o gerador é **serial de propósito** (a
ordem do gerador aleatório é a impressão digital do motor; paralelizá-lo está recusado). Ele
repetia-se a cada vez que a sessão do Wet Paint renascia — cada Ctrl+Z, cada troca de modo — e
**duas** vezes com papel autorado (o `reconcile_facts` re-coze).

Cura: [`paper_memo::paper_tile`](../../crates/ph2d-wet-paint/src/paper_memo.rs), memória **exacta**:
- a chave são os **BITS** das entradas (`-0.0 ≠ 0.0`);
- o mais usado fica à frente e o tecto esquece o menos usado;
- `MEMO_CAP = 8`, e o recurso é a memória (1 MiB por tile);
- o gerador corre fora da trava.

Medido (`--release`, `load 1,5`):

| | ms |
|---|---|
| gerar | 7,41 |
| acerto na memória | 0,013 |

Mutação **4 de 4**, com controlo (6 testes no filtro limpo):
- tirar o *mover à frente* do acerto;
- o motor cozer pela porta crua;
- tirar o tecto;
- a chave pelo `==`.

⚠️ O contador de gerações vive no **gerador**. Com ele na memória, o gate da fiação era vácuo:
contava os pedidos à memória, e não se o gerador corria.

## §6 — Item 8: o relevo sob o pincel composto (2026-09-30)

### §6.1 — A conferência achou mais do que a nota dizia

A nota falava de *«relevo fora da recomposição»*. A sonda
[`diag_o_relevo_da_pilha`](../../crates/ph2d-tool-painter/src/tool/paint/diag_o_relevo_da_pilha.rs)
mediu o mesmo traço no Impasto, com e sem pilha. Com **duas camadas ou mais**:

| o que se mede | pincel avulso | pilha, antes |
|---|---|---|
| corpo depositado | `974,24` | **`0,00`** — o Impasto pintava chapado |
| tinta com `Draw To = Depth` | `0` | **`393 834`** — punha cor onde não devia |
| a camada Erase no relevo | morde (`0,84 → 0,63`) | **não morde** |
| a cor, no Impasto | o FILME | **a tinta cheia do Digital** (pior `178`) |
| Tiling: cor do outro lado da costura | `738` px | **`0`** px (também no Digital) |

As quatro primeiras têm a mesma causa: os planos da pilha forçavam `Draw To = Color`. A quinta é
outra: a região da pilha era medida com os dabs **sem** o Tiling.

### §6.2 — 8a: feita

A lei: **o relevo é da tela e a cor é do plano, uma porta cada.**
- Os planos ficam com o `Draw To` do artista (o filme volta).
- O depósito de altura não corre dentro de um plano (`acumulando_no_plano`).
- Cada camada deposita o relevo à parte
  ([`composite_relevo`](../../crates/ph2d-tool-painter/src/tool/paint/composite_relevo.rs)): Brush
  põe corpo; Erase de escopo `Tudo` morde como a borracha avulsa; `Traco`, Blur e Smear não lhe
  tocam, como os avulsos.
- A região da pilha conta as cópias do Tiling, e o depósito da pilha publica os grupos.

**Resultado:** cor e relevo da pilha **idênticos ao bit** aos do pincel avulso, no Digital e no
Impasto, para o Brush e para a borracha; e o Tiling atravessa a costura ao byte. `8` gates, mutação
**10 de 10** com controlo.

⚠️ Com grão ALEATÓRIO a pilha não é igual ao avulso, **mesmo longe da costura**: cada camada tem o
seu próprio fluxo aleatório (`rng_camada`), e isso é desenho da pilha. A régua dos grupos do Tiling é
outra: numa tela em Tiling, o mesmo traço deslocado de uma largura inteira pinta a mesma imagem.

### §6.3 — 8b: aberta, nomeada

Uma camada Erase **acima** de um Brush, no mesmo traço, apaga a **cor** que o Brush pôs e **não o
corpo**. O corpo do traço vive num envelope que só assenta ao soltar, e a borracha da pilha só morde o
relevo já assente.

A cura é compor o envelope pela lei dos planos de cor: a cobertura acumulada de cada borracha acima
multiplica a tinta do envelope, com a tinta crua guardada à parte. Isto toca o envelope do Impasto
(o commit e a luz leem-no), por isso é trabalho próprio.

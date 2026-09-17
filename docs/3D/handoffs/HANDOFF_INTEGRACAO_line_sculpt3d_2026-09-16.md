# HANDOFF de INTEGRAÇÃO — `line/sculpt3d`, 2026-09-16

> **Continuação de
> [`HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-15.md`](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-15.md)**
> (que fechou no §47, o Box Trim aprovado). A numeração **continua** (§48 em diante).
>
> **A jornada:** o dono aprovou o Box Trim e pediu o pincel que *«faz o trim esfregando o pincel
> como massinha»*; o smoke dele devolveu três relatos — *«sem undo/redo»*, *«o radius máximo
> permitido é pouco»* e *«resultado inferior ao blender para produzir superfícies planas»* — e a
> dica *«o melhor jeito de ver o efeito é com o pincel bem grande, do tamanho da peça»*.
>
> ⛔ **Nada integrado, nada enviado** (`CLAUDE.md` §0.7). Zero contadores partilhados: nem
> `PROJECT_SCHEMA`, nem `VEC_SCENE_SCHEMA`, nem `FLIP_SCHEMA`, nem `FIELD_DOC_VERSION`, nem os
> registos de componentes; zero contrato congelado; zero ADR; zero pacote externo.
> ⚠️ **UMA mudança foundational, aditiva:** o `ph2d-editor-core` ganha a **ligação curva**
> pista↔número (§50.3) — nenhuma assinatura pública muda, e a identidade é byte-idêntica.

---

## §48 — ⭐⭐⭐ O PINCEL DE PLANO (`Verb::Plane`, 34.º verbo) — clean-room do alvo, cena `=47`

**O que é:** esfregar apara o relevo contra o plano que a própria pegada ajusta — as cristas
descem, os vales ficam (tectos `1/0`), ou o contrário. Espec atestada:
[`SPEC_pincel_de_plano.md`](../cleanroom/SPEC_pincel_de_plano.md) (R-pré, 4.ª passagem), com as
cinco erratas curadas pela janela I antes da primeira linha de código (`9939f100d`).

### §48.1 — A lei, e o que a paridade mediu

| população | barra | pior medido |
|---|---|---|
| **19** fixturas de um dab efectivo (G-1) | `1e-6` | **`8,941e-08`** |
| **23** traços inteiros (G-1b — a dívida que a espec nomeava, **paga**) | `1e-6` | **`5,960e-08`** |

⭐ As nossas normais de vértice desviam `0,034°` na crista e `0,445°` nas bossas da malha do
oráculo e a saída **ainda** bate — a lei da §2.1 é robusta a isso por construção (média pesada).

### §48.2 — Quatro leis que só o corpus deu

- **A consulta tem de cobrir os raios de AMOSTRAGEM**, não só a pegada: `lei_area20` desviava
  `1,703e-1` até o factor passar a `max(1 + 0,75 + |deslocamento|, frac_normal, frac_área)`
  (`tectos_query_factor`).
- **O `from_live` fica PREGADO a `true`** (`brush_verb_defaults.rs`): nesta lei o interruptor de
  acumular decide **de que superfície o plano é lido** (porta `le_a_superficie_viva`), e mais
  nada. Preso à coluna, as cadeias de 4 e 8 dabs desviavam `1,634e-1`/`3,022e-1` com a de 2 certa
  — *o primeiro dab estava certo e o segundo já não*.
- **A força NÃO passa pelo perfil `S`** — o `RefMode::declares` é uma **lista negra**, e um verbo
  novo sem perfil caía no slider cru (força linear: `2×` a meio slider). O `Plane` entrou nela, e
  o censo novo `o_censo_das_referencias_que_ninguem_declara` lista os que ficam de fora com o
  nome: **Sharpen**, Cloth, Pose, Boundary, Density, Erase/Smear Displacement, Scene Project, Box
  Trim — ⚠️ o **Sharpen** é dívida nova nomeada (a força dele é linear).
- **O primeiro dab é INERTE** (falta a direcção do traço) e um bit trancado mantém um traço parado
  vivo; o arnês de fecho dava **um** dab a cada verbo e lia o pincel morto ⇒ porta
  `Verb::exige_esfregar` + `arma_o_traco` nos dois arneses.

### §48.3 — ⚠️ Divergência DECLARADA e dívidas

- **Inversão `Afastar`** (o sinal troca): `1,007e-1` contra o oráculo, com gate a medir o número
  (`o_afastar_e_uma_divergencia_declarada_com_numero`). A `TrocarTectos` é **exacta**.
- ⏳ Por escrever: G-5 (o discriminante), G-6, G-13; a pegada projectada; a máscara no arnês de
  bancada. (G-12 e G-14 fecharam com os estabilizadores, §51.)
- ⚠️ Desde o §51 o `Ctrl` de FÁBRICA é o `TrocarTectos` (o perfil *aparar* do alvo); o `Afastar`
  continua no chip.
- O verbo shipa **só com chip** (`CHIP_ONLY` no gate do teclado): `25` das `26` letras estão
  tomadas, medido.

**Smoke:** `PH2D_SCULPT3D_SMOKE=47` (bossas em `40 k` triângulos; o roteiro, reescrito no §51, tem 9 passos).

---

## §49 — ⛔⛔ *«sem undo/redo»*: a ferramenta VECTORIAL em mãos matava as teclas da escultura

⭐ **Não era do pincel.** O `~/.ph2d/layout.txt` do dono tem `active=vector`; a ferramenta
vectorial é reactivada no **quadro 1** (o `move` de omissão é o do quadro 0) e, com o barro na
tela, o **ponteiro** ia para a escultura (esfregar funcionava) enquanto **todas as teclas**
morriam — o `sculpt3d_keys_dead_reason` respondia *«a ferramenta Motion/Vector está EM MÃOS»*.
⚠️ Os dois gates do traço (pelas funções e pelas portas de ponteiro/teclado da família) passavam
**verdes com o defeito vivo**: ele vivia entre o evento do `winit` e a porta da família.

### §49.1 — O instrumento, e o que ele mediu na app real

`PH2D_SCULPT3D_UNDO_PROBE=1` com a `=47` — o roteiro do smoke (crase, chip, esfregar, arrastar a
pista, digitar num campo, esfregar, `Ctrl+Z`×2, `Ctrl+Shift+Z`×2) pelo teclado e ponteiro reais:

| | `ferramenta` | `teclas` | `Ctrl+Z` |
|---|---|---|---|
| antes | `vector` do quadro 1 em diante | **mortas** | `undo=2` fica `2`, a malha não mexe |
| depois | `vector` até ao 1.º esfregão, `move` a seguir | vivas | `2→1→0`, a malha volta ao valor **original ao dígito**; o refazer devolve os dois |

### §49.2 — A cura, e onde ela mora

- **4.ª linha da tabela do dono do canvas** (`ph2d_app_field3d::mode`): *um gesto da escultura
  no canvas devolve a ferramenta em mãos à de omissão*, partilhando o re-baseline com o modelador
  (`larga_para_a_neutra`); três gates da lei.
- A shell chama-a **só depois** de a família tomar o aperto e **só com o barro na tela**.
- ⭐ **O `Ctrl+Z` recusado DIZ porquê** — `keys_delete::queixa_do_desfazer`, pura e gateada; o
  teclado da família passou a receber a **razão** das teclas mortas em vez do `bool`.

### §49.3 — ⛔ As duas catracas que a cura acordou, curadas por MOVER

- `the_shell_only_shrinks` (+168): a sonda inteira na shell custava 187 linhas ⇒ o roteiro e a
  leitura vivem em `ph2d_app_sculpt3d::sonda_undo` (com gate de botão/tecla nunca presos), e a
  shell fica com o **executor de 42 linhas**. Os três gates de fonte do elo de shell vivem em
  `host_contract_tests::a_sculpt_gesture_releases_the_tool_in_hand` — o cabeçalho daquele
  ficheiro já mandava que vivessem lá. ⚠️ **A catraca conta também `shells/desktop/tests/`**: o
  gate novo lá dentro, sozinho, deixava a shell `5` linhas acima do tecto.
- `undo_tests.rs` `817/700`: os dois gates do plano partiram-se para `undo_plano_tests.rs`.

Prova de mutação **7 de 7**. ⚠️ **O arnês de mutação mentia antes**: um teste que falha imprime
`0 passed`, e o detector de «não correu nada» lia isso como vazio ⇒ ele passou a SOMAR
`passed + failed`.

---

## §50 — ⭐⭐⭐ *«o radius máximo permitido é pouco»*: o tecto é a DIAGONAL da vista

### §50.1 — O tecto antigo era uma opinião, e a medição do custo

O raio parava em `1/8` da altura da vista, e o doc dizia de que era: *«acima de meio modelo o
pincel é um deformador global, que é outra ferramenta»*. ⛔ **Não é um recurso.** Na vista de
omissão a peça ocupa `49 %` da altura ⇒ o tecto dava **metade do raio da peça** (`135` px a
1080 linhas contra `500` na pista do alvo). O custo, medido pela porta do produto
([`mede_o_raio_do_pincel`](../../../crates/ph2d-sculpt3d/tests/it/mede_o_raio_do_pincel.rs),
`--release`, `load 5`):

| triângulos | verbo | raio `0,25` | raio `1,0` | **peça inteira** (`2,5`) | `4,0` |
|---|---|---|---|---|---|
| `39 480` (a `=47`) | Draw | `0,16 ms` | `1,19` | **`2,76`** | `2,54` |
| `39 480` | Plane | `0,21` | `1,22` | **`1,40`** | `1,55` |
| `398 724` | Draw | `0,51` | `11,65` | **`27,8`** | `26,9` |
| `2 998 800` | Draw | `5,00` | `101,8` | **`256,8`** | `278,0` |

⭐ **~0,14 µs por vértice tocado, e SATURA na malha** (gate: a contagem acima do diâmetro não
cresce). E o espaçamento é proporcional ao raio ⇒ um pincel `8×` maior deposita `8×` menos dabs.
⇒ **o recurso que aperta é o ECRÃ**: com o cursor em qualquer ponto, um anel do tamanho da
diagonal contém a vista inteira, e acima dele só se acrescenta geometria que o artista não vê.
⚠️ O preço de uma malha pesada é o da **MALHA** (o filtro de malha inteira paga o mesmo), e a
saída dele é o K1 (`docs/3D/03.5`), não um tecto que esconda a peça.

### §50.2 — A cura

- `rulers::radius_ceiling_px(w, h)` = a diagonal (`RADIUS_MAX_FRAC_OF_DIAGONAL = 1`), com gate
  puro (`o_pincel_pode_ter_o_tamanho_da_peca`) e o elo na cena com GPU
  (`o_raio_da_cena_chega_a_diagonal_da_vista`).
- **O anel do cursor continua redondo:** os `48` segmentos fixos davam facetas de `~6 px` num
  anel de `2 779 px` ⇒ a contagem é derivada da **flecha** (`0,3 px` — a que os 48 davam no tecto
  antigo, o anel que o dono via redondo), entre `48` e `512`.
- A pista do painel vai a **`5 000` px** (≥ o digitável do alvo e ≥ a diagonal de um 4K).

### §50.3 — ⭐ A pista CÚBICA, e a extensão foundational que ela pediu

Uma pista **linear** até milhares punha o pincel de 50 px a `1 %` da trilha. ⇒ **ligação curva**
no `ph2d-editor-core` (`state/slider_curve.rs`): `número = offset + scale · pista^curva`.

- ⚠️ **Aditiva:** o `linked_slider_mapping` público continua `(scale, offset)` (sete crates o
  leem nos gates); a curva lê-se por `linked_slider_curve`, e `curva = 1` salta o `powf` —
  **byte-idêntico**. As projecções são UMA porta (`pista_para_fracao` / `fracao_para_pista`) que
  o store e o painel chamam, e a saturação do espelho continua a decidir-se na fracção LINEAR.
- **No painel a curva é DERIVADA da faixa** (`Row::curva`): piso positivo e três ordens de
  grandeza ⇒ cúbica; hoje só o raio (a razão seguinte é `200`). ⛔ Um campo por linha seria
  `58` literais a dizer «linear» e o `rows.rs` estava a `600/600`.
- Pincel de omissão a `~21 %` da pista, o tecto antigo a `~29 %`, a peça inteira a `~37 %`.

Prova de mutação **10 de 10** — ⚠️ a 1.ª ronda deu **9**: a curva sobre um mapa afim
**identidade** era descartada em silêncio, e o gate só usava a faixa do raio.

---

## §51 — ⭐⭐⭐ *«resultado inferior ao blender para produzir superfícies planas»*: a MEMÓRIA do plano

### §51.1 — A causa não estava na lei, e o próprio alvo a reproduziu

A lei por dab já batia o oráculo a `1e-8` (§48) — e o relato era verdadeiro. A 2.ª missão do E
(§14 da espec, **R-pré 5.ª passagem: verde**, `515e887ee`) mediu porquê: o corpus da 1.ª missão
escrevia os valores **à mão para a lei ser RECUPERÁVEL**, dava os dabs **por script** e punha o
cursor **fora** do relevo. O artista tem os valores **de fábrica**, um traço **arrastado** e o
cursor **na** superfície. ⭐⭐ **Os nossos valores de fábrica, no motor do alvo e com o traço
dele, deixam o relevo `1,114×` MAIS rugoso em oito passagens** — o relato do dono, reproduzido no
próprio alvo com os nossos números. A ablação (§14.7) nomeia a alavanca: **a firmeza da normal**
(`0,029` com ela, `1,380` sem ela), depois o acumular, depois a dureza.

### §51.2 — O que entrou

| peça | onde | o que é |
|---|---|---|
| **a memória do plano** (§6.1 + §6.2) | [`plano_memoria.rs`](../../../crates/ph2d-sculpt3d/src/plano_memoria.rs) | normal interpolada com a publicada e média numa memória circular (tecto `20`); centro puxado para o plano anterior, publicado com o desvio MÉDIO. **Uma por passe de simetria**, semeada no 1.º dab (o inerte, errata Q6), esquecida a cada traço |
| **o passo e a atenuação** (§14.4) | [`atenuacao_do_traco.rs`](../../../crates/ph2d-sculpt3d/src/atenuacao_do_traco.rs) | passo `raio × 7 / 50` só no plano; `(1 + a)/2 = 0,570` por dab arrastado (`0,5·a` no afastar invertido), calculado UMA vez por dab e guardado no plano |
| **o arrasto** | `Brush::traco_arrastado`, ligado **só** pelo `armed_brush_on` da família | a porta por script (bancada, oráculo) fica com o factor `1` |
| **os valores de fábrica do *aparar*** | `brush_default.rs` + `brush_verb_defaults.rs` | força `0,7`, dureza `0,6`, acumular **ligado**, extensão do centro `0,6`, firmeza da normal `1`, `Ctrl` a **trocar os tectos** |
| **o painel** | `rows_plano.rs` | `Hold Tilt` (BÁSICO — é a alavanca) e `Hold Height` (Pro, só com a da normal ligada — §6.3) |
| **o roteiro da `=47`** | `scenes_plano.rs` | pincel grande, ida e volta sem largar, e o passo que mostra o `Hold Tilt` a 0 |

⭐ **Um achado que a espec não fixava, e o oráculo decidiu:** a normal interpolada é
**NORMALIZADA** antes de entrar na memória — crua, as fixturas de firmeza `0,25`/`0,5` erram
`1,2e-4`/`6,6e-5`; normalizada, `3e-8`/`6e-8`. Com a firmeza a zero ela entra crua, e o caminho da
1.ª missão não muda um bit.

### §51.3 — O que se mediu

| gate | afirma | medido |
|---|---|---|
| memória (as `12` fixturas de `firmeza/`) + **G-12** | reprodução `≤ 1e-6`; o centro mudo nas bossas simétricas e vivo no degrau | pior `1,34e-7` |
| **G-14** | tecto da memória `20`, comprimento truncado | contagens exactas |
| **G-15** | um dab com o cursor NA superfície e os valores de fábrica | `≤ 1e-6` |
| **G-19** + 9 células | uma rampa e um degrau não se mexem; as outras 9 da §2.2 reproduzem | rampa `8,8e-8`, degrau `1,8e-7` (o alvo: `0`/`3e-8`) |
| **G-16** | o passo é `7 %` do diâmetro | `6/13/14/21` px ⇒ `0/1/2/3`; ⛔ **`7` px dá `0` contra `1` do alvo — divergência DECLARADA** (errata Q4: o `<=` do `walk` da casa tem gate e razão escrita; o dab fica para o evento seguinte, no mesmo sítio) |
| **G-17** | cada dab arrastado vale `(1 + a)/2` | lei `0,570`; o oráculo lê `0,563`; o produto, a razão exacta |
| ⭐⭐ **G-18** | a firmeza da normal é a alavanca | **fábrica `0,028`** (alvo `0,029`) · **sem firmeza `1,188`** (alvo `1,380`) · **os valores de antes `1,134`** (alvo `1,114`) |
| os valores de fábrica | nascem os do *aparar* | contra a fixtura de pincel nosso que os escreve |
| simetria e ciclo de vida | uma memória por passe; cada traço esquece | saída espelhada `≤ 1e-5`; B depois de A = B num traço novo, ao bit |

⚠️ **A planura do traço arrastado bate também a 4 passagens** (`0,092` contra `0,092`), e ⭐
**uma passagem é UMA travessia** — lida como ida-e-volta ela seria `0,028` a 4. ⚠️ Ponto a ponto o
arrasto desvia até `1,9e-2`: é a posição sub-píxel de cada dab (o arrasto do alvo nem é
determinístico ao bit), e a espec põe a afirmação na planura, não nas posições.
⛔ **Os gates usam só fixturas de pincel NOSSO** (`copia_*`, `ablacao_*`, `ctl_*`) — a nota de
proveniência do R-pré tira as `fab_*` (geradas com o perfil do alvo carregado).

Prova de mutação **12 de 12** (a memória desligada · a interpolada crua · o centro sem puxão ·
a memória partilhada entre passes · o traço que não esquece · o factor fora do alvo · a atenuação
a `1` · a app sem arrasto · o centro sem a normal · a dureza e o acumular de fábrica · o tecto a
`10`).

---

## §53 — ⭐⭐⭐ *«Próximo pincel: Draw Sharp»*: o VINCO, e ele não trouxe lei nova

**Ordem do dono (2026-09-16):** *«Próximo pincel: Draw Sharp. Veja no blender com a mesma
investigação que fez para Plane»*. Espec atestada à 2.ª passagem
(`docs/3D/cleanroom/SPEC_pincel_afiado.md`, `80` fixturas); o incidente **INC-I1** da janela I foi
classificado **RELANCE** por um R independente (`b7bdb3e48`).

### §53.1 — A resposta da PERGUNTA ZERO, e porque ela é o achado

⭐⭐ **O pincel afiado de fábrica é o nosso `Draw` com CINCO valores trocados e NENHUMA lei nova**
(espec §0.1) — três do pincel (a curva afiada, a direcção a afundar, a distância medida sempre do
pen-down) e dois do traço (o passo de `5 %` do diâmetro e a atenuação `a` por dab). ⇒ ele entra
como **verbo** (`Verb::DrawSharp`, o 35.º; a contagem é o `Verb::ALL`) porque é assim que esta casa
oferece uma ferramenta — o precedente é o par `Pinch`/`Magnify`, o mesmo kernel com um sinal e dois
chips —, e o `match` do alvo por-vértice ganha **um braço**, que difere do `Draw` só na NORMAL.

⛔⛔ **E a recusa de 2026-08-14 CAIU, com a medição ao lado:** a nota de planeamento tirava este
pincel da fila dizendo que *«o que o nome promete mora na CURVA, e a curva de fábrica está num
ficheiro binário»*. Os valores de fábrica **lêem-se correndo o programa** — foi o que a obra do
pincel de plano já fizera para a dela. ⚠️ *Quem move o número que tornava algo inalcançável tem de
reconferir a nota* (§0.0), e as três notas do nosso código com a premissa caída estão nomeadas na
§0.2 da espec.

### §53.2 — As cinco leis, e onde cada uma mora

| o que muda | onde | porquê ali |
|---|---|---|
| a distância mede-se do **pen-down**, sempre | `Verb::grip_law` prega `from_live = false` | é a **única** diferença de LEI (espec §2.1), e pendurá-la no interruptor faria o vinco ALARGAR ao aprofundar |
| a **normal da área** é a do alvo | `stroke_dab_core` pede o `n_gesto` e o braço do `stroke_target` lê-o | a lei já vivia em duas portas da casa (os gestos tangenciais, o pincel de plano) e o `Draw` não a usa |
| a direcção de fábrica **afunda** | `Verb::afunda_de_fabrica` + `Brush::reach` | a casa só sabia *«o Ctrl inverte»*, e o `invert` tem **um** escritor no produto inteiro |
| o passo de `5 %` e a atenuação **`a` crua** | `espacamento_do_verbo` + `Brush::factor_do_traco` | uma porta só para o número, lida pelo passo E pela atenuação |
| a curva **afiada** e a força ao quadrado | a lista negra do `S` em `RefMode::declares` | sem a entrada, o `S` declarava-o calado ⇒ força **linear** e curva suave |

⚠️ **A atenuação entra pelo [`Brush::reach`]**, que é onde o deslocamento de um dab é escrito e já
carregava o sinal; o pincel de plano lê a MESMA porta no sítio onde a lei dele pesa. *Uma lei, uma
porta, dois consumidores — e cada um multiplica-a na grandeza que a sua própria lei escala.*

### §53.3 — O que a bancada mede (14 gates, `crates/ph2d-sculpt3d/tests/it/oraculo_do_pincel_afiado*.rs`)

| gate | população | barra | medido |
|---|---|---|---|
| **G-1** a lei de um dab | `16` fixturas (a pegada projectada fica na catraca dos pendentes) | `2e-6` · `1e-5` nas bossas | **`5,96e-8`** |
| **G-2** as cadeias | `5` cadeias, **`36`** estados | `1e-5` | **`1,57e-6`** |
| **G-3a** o traço contínuo, vértice a vértice | `6` superfícies × fotos | `2e-3` na faixa do vinco | **`6,07e-4`** |
| **G-3b** os traços separados | `5` superfícies × fotos | `5e-3` | **`2,54e-3`** |
| **G-4** a RÉGUA do vinco (profundidade · largura · nitidez) | `8` células × `4` passagens | `6 %` · `2 %` (contínuo) | **`≤ 0,5 %`** |
| **G-5** o passo do traço | `13` saltos | contagem exacta + `7e-3` | **`4,61e-3`** |
| **G-5c** o dab do pen-down é atenuado | `2` | `1e-5` | **`1,96e-8`** (sem atenuação: `4,53e-2`) |
| **G-6** os valores de fábrica | os `5` que mudam | exacto | — |
| **G-7** a auto-limitação, forma fechada | `4` dabs + o controlo linear | `1e-6` | dentro |
| **G-8** a régua SEPARA o afiado do desenho comum | `2` saídas do alvo | `≥ 1,8` | **`2,29`** |
| **G-9** o 1.º dab é o do desenho comum | `2` | ao bit | **`0,0`** |
| **G-10** a direcção é simétrica | `2` | `2e-6` | dentro |
| **G-11** o recorte pela caixa NÃO se copia | `1` corrida nossa × `2` fixturas | `≤ 7e-3` e `≥ 1e-2` | **`2,9e-4`** e **`2,24e-2`** |
| — as duas leis de normal de vértice | `1` | `1e-5` | **`0,0`** |

**Prova de mutação: `10` mutações, `10` sangram** (a coluna do pen-down · a direcção de fábrica · a
atenuação trocada pela do plano · o espaçamento trocado · a normal da área · a curva de fábrica · a
consulta da pegada · a lista negra do `S` · o interruptor de acumular · a topologia dinâmica), com o
controlo da árvore limpa verde.

### §53.4 — ⛔ A DIVERGÊNCIA DECLARADA, e ela refuta um comentário que já shipava

O alvo deposita um dab num salto de **exactamente** um passo e o `walk` desta casa recusa-o
(`spacing.rs`, com gate próprio há mais tempo que esta wave). ⚠️⚠️ **O doc daquele `walk` afirma
que as duas fronteiras dão a MESMA lista de dabs — e isso é verdade a MEIO do traço e FALSO na
INVERSÃO:** ali o dab adiado nunca chega, porque a passagem seguinte anda para o outro lado. Medido
num vaivém simulado a `1` px por evento: perdemos **um** dab em cada ponta de cada passagem, o
vértice da ponta desvia **`1,03e-2`** na 1.ª passagem e a sombra chega a **`4,06e-2`** ao 8.º traço
separado, sobre `30` de `1 514` vértices.

⭐ **Por isso a barra que discrimina a LEI é a da FAIXA do vinco** (`|x| ≤ 0,25`), e as pontas
ficam com um **tecto declarado** (`5e-2`) que reprova se a sombra crescer. ⚠️ **O artista não a
atinge:** com eventos de rato irregulares, a distância cair exactamente sobre um múltiplo do passo
tem medida nula, e o fim de um traço real perde a fracção de passo que sobra **nas duas casas**.

### §53.5 — A costura de teste que a espec pediu, e porque ela é `test-support`

O caminho por script do oráculo **não refresca** as normais de vértice entre dabs e o nosso refresca
sempre ⇒ sem pregar as normais, a cadeia de oito dabs desvia `4,45e-2` **por uma razão que não é a
lei**. ⇒ `Mesh::pregar_normais_para_teste`, `#[cfg(any(test, feature = "test-support"))]`, com a
feature do tamanho do que ATRAVESSA a fronteira: **um** item. ⛔ No produto as normais seguem a
superfície — é isso que faz um traço arrastado ser o que é.

⚠️ **E a normal que a bancada usa é a NOSSA, não o bloco `n`** (errata do R-pré): o bloco é a normal
do REPOUSO do alvo (pesada pelo ângulo do canto) e a lei que reproduz o TRAÇO dele é a média **sem
peso**, que é a da casa. Há gate a afirmar que as duas leis são **duas**.

### §53.6 — Cena `=48`

Abre numa bola **lisa e densa**, e as duas metades são a lição: ao contrário do vizinho de plano
(que precisa de relevo para aparar), este **faz** o relevo; e o vinco mede `≈ 0,55 R` de largura, que
com menos de três arestas não é representável. O roteiro tem `6` passos e acaba no controlo — pegar
no `Draw` e arrastar do mesmo jeito, para ver o monte largo ao lado do vinco fino.

### §53.7 — O que fica ABERTO deste pincel

- **Dono (as três decisões da espec §15):** **P-2** o interruptor de acumular neste modo (hoje
  ESCONDIDO, com a fixtura da lei do alvo na catraca e o número que ela vale, `4,7e-4`) · **P-4** a
  pegada projectada (hoje não oferecida; ela empurra **de lado** e não faz o que o nome promete) ·
  **P-5** a opção da normal do pen-down (hoje não oferecida; inerte num plano).
- **Nossas, nomeadas:** a linha do `Radius` da normal tem tecto `1` no painel e o corpus tem uma
  fixtura a `2` (a consulta já a suporta; o tecto é de produto) · a divergência do §53.4 ·
  as `22` fixturas de `produto/` que são ablação e controlos ficam **fora** dos gates, com o papel
  de cada uma nomeado na espec §12.2.

---

## §54 — ⭐⭐⭐ *«do canto para o início da esfera, no canto fica meio pontilhado»*: o PASSO deixa de ser contado em píxeis

O report do dono (foto) sobre o pincel afiado: começando o traço na **silhueta** da bola, o vinco
sai **descontínuo** — uma fileira de crateras em vez de um sulco. No meio da peça ele sai perfeito.

### §54.1 — O mecanismo, e o alvo tem-no IGUAL

O passo do traço é contado em **píxeis de ECRÃ**. Um passo de `p` px sobre uma superfície cuja
normal faz `θ` com a vista percorre `(p/ppu)/cos θ` **de superfície** ⇒ junto à silhueta
(`cos θ → 0`) dois dabs consecutivos ficam a vários raios um do outro e o sulco parte-se.

Medido no nosso produto, por bandas de `u` (o cosseno do ângulo de incidência), sobre a fixtura de
fórmula da cúpula — `D/R` é a profundidade do vinco em raios de pincel e `r` a ondulação (excursão
pico-a-pico dentro de UM período de dab, sobre a profundidade média da janela, adimensional):

| banda | lei de ECRÃ `D/R` · `r` | a CURA `D/R` · `r` |
|---|---|---|
| 5–15° | `0,2011` · `0,002` | `0,2041` · `0,003` |
| 25–35° | `0,1856` · `0,006` | `0,2052` · `0,003` |
| 45–55° | `0,1494` · `0,023` | `0,2054` · `0,005` |
| 60–68° | `0,1090` · `0,085` | `0,2069` · `0,018` |
| 68–74° | `0,0832` · `0,187` | `0,2084` · `0,024` |
| 74–79° | `0,0607` · `0,314` | `0,2061` · `0,064` |
| 79–83° | `0,0347` · `0,569` | `0,1448` · `0,686` |
| 83–88° | `0,0402` · `0,974` | `0,0544` · `0,991` |

⚠️ **As duas últimas bandas são o ARRANQUE do traço, não o defeito** — ali o vinco ainda está a
nascer, e é por isso que os gates afirmam sobre as **seis** primeiras
([`BANDAS_DO_REGIME`](../../../crates/ph2d-sculpt3d/tests/it/oraculo_do_pincel_afiado_silhueta.rs),
uma const partilhada pelos dois gates que a usam).

⛔⛔ **O ALVO TEM O MESMO DEFEITO, e isso foi medido antes de se escrever uma linha de cura** — a
ondulação dele vai de `0,002` no meio a `0,977` junto à borda e a profundidade fica `5,19×` mais
rasa. ⇒ *aqui não há lado aprovado a copiar: a paridade e o produto apontam para lados opostos.*

⛔⛔⛔ **E a cura DELE está RECUSADA, com três defeitos medidos** (a lei que mede o passo contra a
superfície VIVA, que o próprio traço move): ela cura o pontilhado e compra `10×` de profundidade na
borda, `−30 %`/`−51 %` de dependência da **taxa de amostragem do rato** e `3,3×` de dependência do
**sentido do gesto**. *O traço deixa de ser um facto do CAMINHO*, que é a lei que esta casa pagou
seis vezes no Painter — e é o gate **G-19** que proíbe importá-la.

### §54.2 — A cura é NOSSA e tem DUAS metades, que NÃO são a mesma pergunta

1. **O passo mede-se sobre a superfície do PEN-DOWN**
   ([`ph2d_sculpt3d::CaminhoNoMundo`](../../../crates/ph2d-sculpt3d/src/passo_no_mundo.rs)): o
   caminho de ecrã é percorrido por candidatos, cada candidato é picado na superfície **congelada**,
   e o que se acumula é o **arco** entre candidatos. ⛔ Congelada e não viva — é isso que a separa
   da lei do alvo: a superfície que ela mede não se move enquanto o traço corre, logo não há
   realimentação, não há dependência da taxa de eventos e não há dependência do sentido.
   O resíduo **viaja** entre eventos (a mesma lei do carry do `walk`), e a granularidade dos
   candidatos é **adaptativa** — o passo de ecrã seguinte é previsto do arco que o último mediu, com
   `previsto` a viver na struct para atravessar os eventos.
2. **O centro do dab segue o BARRO**
   ([`levado_pela_deformacao`](../../../crates/ph2d-sculpt3d/src/passo_no_mundo.rs)): o raio pica a
   superfície congelada, e a face + baricêntricas do acerto são aplicadas às posições **vivas** — o
   ponto afunda com o vinco (a auto-limitação do pincel fica intacta) mas **não desliza** ao longo da
   superfície.

⚠️ **As duas armam a MESMA fotografia do pen-down e têm predicados SEPARADOS**
(`Verb::mede_o_passo_no_mundo` · `Verb::o_dab_segue_o_barro`), e juntá-las numa porta só faria a
próxima medição ter de as separar outra vez — que é exactamente o que aconteceu ao fechar esta wave.

### §54.3 — ⛔⛔⛔ A atribuição: o PASSO não custa paridade nenhuma; quem paga é o BARRO

Esta é a medição que decidiu a wave, e ela só existe porque a bancada do produto passou a correr as
**três** leis do cursor na mesma corrida
([`LeiDoCursor`](../../../crates/ph2d-sculpt3d/tests/it/oraculo_do_pincel_afiado_produto.rs)).
Corpus dos traços SEPARADOS (vaivém de 8 passagens, cada pen-down a refotografar), desvio
vértice-a-vértice na faixa do vinco:

| lei do cursor | p1 | p2 | p4 | p8 | pontas p8 |
|---|---|---|---|---|---|
| **ecrã** (o alvo) | `4,0e-4` | `2,0e-4` | `4,7e-4` | `1,1e-3` | `4,04e-2` |
| **só o PASSO** | `4,1e-4` | `2,7e-4` | `5,9e-4` | `1,2e-3` | `2,20e-2` |
| **passo + BARRO** | `4,0e-4` | `7,3e-4` | `1,9e-3` | **`1,71e-2`** | `1,91e-2` |

⭐⭐ **O passo sozinho fica dentro da barra apertada de sempre (`5e-3`) em todas as células e ainda
MELHORA a ponta**; e na bancada da silhueta ele entrega quase toda a uniformidade de profundidade
(`0,2018 → 0,1829` nas bandas `0`–`5`, contra `0,2011 → 0,0607` da lei de ecrã).
⭐⭐⭐ **Mas o pontilhado — que é o que o dono fotografou — é o BARRO que o cura:** sem a advecção a
ondulação da banda `74–79°` fica em **`0,251`** (contra `0,314` sem cura nenhuma, ou seja quase
nada); com ela cai para **`0,064`**. ⇒ *a metade cara é a necessária, e o preço dela está declarado.*

### §54.4 — ⛔ A DIVERGÊNCIA DECLARADA, e a sua própria catraca

[`TECTO_DO_ULTIMO_SEPARADO = 2,0e-2`], só para a **última** passagem do vaivém separado. O mecanismo
lê-se na tabela acima: a divergência **cresce com o número de passagens**, que é o mesmo que dizer
que cresce com o quanto a superfície já está **esculpida**. Na 1.ª passagem a peça é um plano de
frente para a vista e as três leis dão o mesmo número; da 2.ª em diante o cursor anda dentro de um
vinco, que é superfície CURVA — e é exactamente aí que as duas leis são desenhadas para discordar:
*o nosso cursor é um facto do BARRO e o do alvo é um facto do RAIO*. Ao nível do PRODUTO a troca é
`2,4 %`–`3,6 %` de profundidade e `1,4 %`–`4,9 %` de largura na 8.ª passagem (régua do §7.1).

⛔⛔ **O gate EXIGE que a divergência exista.** No dia em que alguém a fizer desaparecer ele reprova
e obriga a **apagar** o tecto, em vez de o deixar a cobrir uma regressão nova — a forma que uma
tolerância sem censo de obsolescência toma quando vira licença.

⭐ **E o `TECTO_DAS_PONTAS` DESCEU, `5e-2 → 2,5e-2`:** a divergência **D-1** era do `walk` (que
recusa um salto de exactamente um passo) e o afiado já **não passa por ali** — a fronteira do
`CaminhoNoMundo` é a do alvo, e a sombra da ponta cai de `4,04e-2` para `1,91e-2`. *Uma catraca que
a cura fez descer e ninguém desceu é uma catraca que virou tolerância.*
⚠️ A catraca é sobre o **PIOR da célula** e não sobre cada passagem, e a diferença foi medida: a cura
ganha nos extremos e na 2.ª passagem fica um cabelo atrás (`1,069e-2` contra `1,021e-2`).

### §54.5 — ⚠️ SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **A bancada do produto mudou de LEI, e é por isso que ela continua a afirmar paridade.** Se ela
   corresse o `walk` de ecrã enquanto o app corre o caminho no mundo, mediria um programa que já não
   existe. Ela pergunta ao **verbo** e só depois deixa a `LeiDoCursor` escolher entre o que ele
   oferece — *uma bancada que derivasse a lei só do parâmetro afirmaria a lei e não o produto, e a
   mutação que tira o afiado do predicado deixava o gate da cura VERDE (medido).*
2. **A fotografia do pen-down é RE-TIRADA em cada pen-down**, e o arnês tinha-a a ser tirada uma vez
   por corrida ⇒ o 8.º traço media-se contra a superfície do 1.º. Com isso a régua do vinco lia
   `1,6593` contra `1,5119` (`9,7 %`); com a fotografia certa, `0,0 %`.
3. **`PH2D_AFIADO_LEI_DE_ECRA` NÃO EXISTE.** Ela existiu durante a medição e **saiu**: `env VAR=`
   **define** a variável vazia, uma leitura por `is_err()` lia isso como armada, e o controlo correu
   a mesma lei que devia contradizer — as duas colunas saíram idênticas e quase passaram por prova.
   ⇒ a lei é **parâmetro**, e o controlo vive **dentro** do gate.
4. **O `MAX_DABS_POR_PASSO` nunca é atingido no corpus** — ele é um tecto de recurso (dabs por
   evento), e o gate que o mede é sintético de propósito.
5. **A `regua` do §16.1 não transfere para um traço TANGENCIAL**, e o instrumento que o mostra está
   guardado (`diag_o_percurso_ao_longo_da_borda_mal_esculpe`). Ali as estações atravessam o vinco em
   vez de o seguirem e a ondulação satura em `0,95`–`0,99` **com a cura ligada**.
6. **O `Verb::Plane` também declara espaçamento** e **não** entra nesta lei — `passo_no_mundo`
   responde por ele, mas `mede_o_passo_no_mundo` é `false`: *declarar um passo e medi-lo sobre a
   superfície são duas perguntas.*

### §54.6 — ⚠️ QUATRO premissas minhas que a medição derrubou

1. *«a divergência dos traços separados é a composição por dab»* — **falso**: é a superfície já
   esculpida, e a atribuição está na tabela do §54.3.
2. *«o centro congelado cura o pontilhado»* — **construído e REFUTADO**: cura a ondulação e **quebra
   a auto-limitação** (profundidade `0,24` e a crescer). O centro **advectado** guarda as duas.
3. *«a granularidade adaptativa é o que faz a cura chegar à silhueta»* — **não medível no corpus**:
   a mutação que a apagava sobrevivia ao gate da cura, ao da invariância à taxa de eventos e à suíte
   inteira. ⇒ ou se constrói a medição ou se apaga o mecanismo; construiu-se
   (`a_granularidade_adaptativa_nao_perde_dabs_numa_superficie_que_dispara`, por UNIDADE, com a lei
   chamada **um evento de cada vez** — uma corrida só sobre o caminho inteiro não afirma nada sobre
   o `previsto` ATRAVESSAR os eventos, que é metade do mecanismo).
4. *«o percurso tangencial mal esculpe»* — **falso, e o próprio instrumento me desmentiu**: ele
   esculpe `0,100` contra `0,205`, metade e não `3 %`. O que não serve é a **régua**.

### §54.7 — Provas de mutação: **13 de 13 sangram**

**A lei (8):** o predicado do passo · o predicado do barro · o resíduo do carry · o `previsto` a
atravessar os eventos · o tecto de dabs · a fronteira `< passo` · a fórmula da previsão · o
`esquece` do pen-up.
**A rota no app (5):** o braço do carimbo a perguntar ao verbo · a fotografia armada pelas duas
perguntas · o centro a perguntar ao verbo · o `esquece` no pen-up e no desfazer.

⚠️⚠️ **E o ARNÊS mentiu TRÊS vezes antes de dizer a verdade**, com as três formas que este repo já
tem escritas: um filtro que casou **zero** testes imprimiu `ok` e leu-se como *sobreviveu* (⇒ o
arnês CONTA quantos testes correram) · uma mutação que **não compila** lê-se como sangrar (⇒ as do
`esquece` apagam a chamada em vez de lhe trocar o nome) · e o `bc` não existe nesta máquina, o que
pôs o contador a devolver vazio — o guarda acusou-se a si próprio, que é para o que ele existe.

### §54.8 — Onde está

- Lei: [`passo_no_mundo.rs`](../../../crates/ph2d-sculpt3d/src/passo_no_mundo.rs) +
  [`passo_no_mundo_tests.rs`](../../../crates/ph2d-sculpt3d/src/passo_no_mundo_tests.rs).
- Predicados: `brush_verb_predicados.rs` (`mede_o_passo_no_mundo` · `o_dab_segue_o_barro`).
- Rota: `space.rs` (a fotografia e o `pick_do_dab`) · `input.rs` (`percorre_no_mundo`) ·
  `cena.rs`/`birth.rs`/`input_down.rs`/`history.rs` (o acumulador e o pen-up), com o gate da rota em
  [`caminho_no_mundo_tests.rs`](../../../crates/ph2d-app-sculpt3d/src/caminho_no_mundo_tests.rs).
- Bancadas: `oraculo_do_pincel_afiado_silhueta.rs` (a cura, `G-19`) ·
  `oraculo_do_pincel_afiado_produto.rs` (as três leis, a divergência declarada).

### §54.9 — ⏳ ABERTO deste §

- **Dono:** nada de novo — as três decisões da espec §15 continuam as do §53.7.
- **Nossas, nomeadas:** a divergência do §54.4 (declarada, com catraca) · a `regua` do §16.1 não
  serve um traço tangencial, e uma régua que o sirva é obra própria · o corpus do oráculo **não
  contém** um traço que comece na silhueta, logo a cura não tem lado aprovado a que se comparar —
  ela é medida contra a **lei** e contra o **controlo**, nunca contra o alvo.
- **Vassouras (as NOVE vivas, corridas sobre os 16 ficheiros desta wave):** seis fecham `exit 0`, e
  as **três** acusações são **todas PRÉ-EXISTENTES e medidas como tal** — `tip_roundness` (2 linhas
  do `CLAUDE.md` §5, que são precisamente as que já o declaram aberto), `NoError` (a API pública da
  dependência Apache-2.0 do Box Trim, já *«isento com nome»*) e `sculpt_gesture` (o NOME de um gate
  que esta casa escreveu, no §49 deste mesmo handoff). **Zero adições** de qualquer um dos três no
  diff desta wave e **zero** ocorrências nos ficheiros novos. ⛔ **A triagem é do R** — a janela I não
  pode ler o ledger para decidir se cada um é isenção registada ou dívida da linha dona.

### §54.10 — As duas decisões do dono da espec §16.13

- **P-6** (*«ficar fiel ao alvo, ou curar?»*) está **respondida pelo próprio report**: ele
  fotografou o pontilhado e chamou-lhe defeito ⇒ cura-se.
  ⚠️ **Mas a variante entregue NÃO é a recomendada.** A espec recomenda **(iii)** = passo em mundo
  **mais a atenuação a seguir o passo efectivo**, *«o único que mantém a profundidade constante»*.
  O que shipa é **(ii) + o centro levado pelo barro**, com a atenuação **intocada** — e a
  profundidade sai constante à mesma (`0,204`–`0,208` contra `0,201 → 0,061`), medida. ⇒ *a premissa
  do «único» da espec está refutada: a uniformidade veio de a densidade de dabs deixar de depender
  da inclinação, não de mexer na atenuação.* ⭐ E a variante entregue é a mais barata das duas — ela
  **não toca** no valor que os `80` traços do corpus fixam, logo a paridade por dab fica intacta por
  construção.
- **P-7** (*«oferecer o eixo “medido em” ao artista?»*) — **ESCONDIDO**, como a espec recomenda, e
  agora com a razão mais forte: a lei do alvo que aquele eixo ofereceria é a que o §54.1 mede a
  depender da **taxa de amostragem do rato** e do **sentido do gesto**. *Um knob cujo resultado
  depende de quão depressa a mão anda não é um controlo.*

### §54.11 — Portão de fecho

`nextest-impacted`: **15 368 testes, 15 368 passaram** (10 002 saltados), `exit 0` ·
`clippy --all-targets` nas duas crates: **zero** avisos · `cargo fmt --all` aplicado ·
tectos de LOC verdes (`workspace_src_files_under_loc_cap` + o censo de folgas obsoletas) ·
`ph2d-sculpt3d` **564** testes · `ph2d-app-sculpt3d` **206** · mutação **13 de 13**.

---

## §55 — ⭐⭐⭐ *«o spacing está alto e fica meio pontilhada. Reduza o spacing»*: o espaçamento sai do VERBO e passa para o PINCEL

Segundo report do dono sobre o mesmo traço, depois da cura do §54: melhorou, mas o sulco ainda se
lê pontilhado. **Ordem: reduzir o espaçamento.**

### §55.1 — A escada medida, e porque a troca é barata

| espaçamento | `D/R` no meio | `D/R` junto à borda | ondulação pior (bandas do regime) |
|---|---|---|---|
| `5 %` (fábrica do alvo) | `0,2041` | `0,2061` | `0,0643` |
| `3 %` | `0,2019` | `0,2061` | `0,0921` |
| **`2 %` (ship)** | `0,2007` | `0,2032` | **`0,0407`** |
| `1,5 %` | `0,2010` | `0,2065` | `0,0784` |
| `1 %` | `0,1992` | `0,2012` | `0,0402` |

⭐⭐ **A profundidade é praticamente INDEPENDENTE do espaçamento** (`0,199`–`0,208` sobre uma faixa
de `5×`), e a razão é a lei deste pincel: a queda mede-se das posições do **pen-down**, logo o
cursor afasta-se delas enquanto cava e a profundidade **converge**. ⇒ baixar o espaçamento compra
continuidade sem mexer no que o dono já aprovou.

⚠️ **A espec §5.3 media a MESMA coisa do outro lado** (`8 %` e `10 %` dão `−0,2 %` e `−0,6 %`) e
concluía que o espaçamento *«quase não é alavanca»* — **verdade sobre a PROFUNDIDADE e falsa sobre
a CONTINUIDADE do sulco**, que é o que o artista vê. *Uma recusa medida responde UMA pergunta.*

⚠️⚠️ **A coluna da ondulação não é monótona** (`3 %` lê pior que `5 %`), e a causa é da RÉGUA: a
ondulação do §16.1 normaliza por **um período de dab**, e o período segue o espaçamento ⇒ a janela
move-se com a variável. *Uma régua cuja janela segue o número que se está a variar não pode ser a
única testemunha da variação* — por isso o gate afirma pelo **passo** (aritmética, sem janela) e usa
a ondulação como coluna que corrobora.

### §55.2 — ⛔⛔ O espaçamento é um valor de FÁBRICA, e por isso mudou de dono

A 1.ª tentativa foi trocar a constante — e **partiu seis gates de paridade de uma vez**, com as
mensagens a acusar a LEI. O mecanismo: `espacamento_do_verbo` é lido pelo **passo** *e* pela
**atenuação**, e o corpus do oráculo foi gravado com o espaçamento **do alvo**. Com o nosso número,
a bancada arrastava a `2 %` contra uma saída gravada a `5 %` — *dois pincéis diferentes, e o desvio
lido como defeito de lei*.

⇒ **três mudanças, e a do meio é a que importa:**

1. `ESPACAMENTO_DO_AFIADO_DO_ALVO_PCT = 5,0` — o **FACTO** sobre o alvo, que as `80` fixturas fixam
   no cabeçalho.
2. **`Brush::espacamento_pct: Option<f32>`** — `None` é *«o que o verbo declara»*. É a porta pela
   qual a fixtura diz *«corre com ESTE espaçamento»*, e é o que mantém o oráculo vivo. ⛔ **Não é um
   knob de painel:** nenhum controlo o escreve.
3. `ESPACAMENTO_DO_AFIADO_PCT = 2,0` — o que **ship**, por ordem do dono.

⭐ E uma porta só (`espacamento_do_traco`) lida pelo passo de ecrã, pelo passo de mundo **e** pela
atenuação: *escrita duas vezes, um traço andaria com o espaçamento de um pincel e enfraqueceria com
o de outro* — que foi exactamente o modo de falha da 1.ª tentativa.

### §55.3 — ⚠️ Três gates tiveram a premissa MORTA, e a morte está visível no diff

1. *«o passo é `5 %` do diâmetro»* → hoje afirma as **duas** coisas: que o alvo vale `5` e que nós
   shipamos **menos**, com piso de `1 %` nomeado pelo recurso.
2. *«a atenuação de fábrica é `a = 0,24591`»* → hoje mede-a **com o espaçamento do alvo** (o facto) e
   exige que a **nossa** seja mais forte (a compensação que mantém a profundidade).
3. *«o passo de mundo é o espaçamento declarado»* → fixava `0,04` à mão e reprovou sobre produto
   correcto. Hoje afirma a **unidade** (`2·raio·pct/100`) e, de graça, separa **declarar** de
   **medir no mundo**: o afiado declara e mede · o plano declara e **não** mede · o `Draw` não
   declara. ⛔ `passo_no_mundo` passou a devolver `None` a quem não mede — *um número devolvido ao
   plano era uma segunda resposta a «este verbo mede no mundo?»*.

⭐ **E a ordem do dono é agora um ERRO DE COMPILAÇÃO** (`const _: () = assert!(nosso < do_alvo)`),
ao lado dos dois números: um `assert!` de teste sobre duas constantes é **dobrado pelo compilador**
antes de correr, e o clippy di-lo em voz alta (*«this assertion has a constant value»*).

### §55.4 — O gate que faltava, e o recurso

**G-21** (`o_nosso_espacamento_de_fabrica_deixa_o_sulco_mais_continuo`): todo o corpus arrasta com o
espaçamento do **alvo**, logo **nenhum gate media o que o produto ship** — reverter o número passaria
calado. Ele corre as duas colunas com a cura ligada e afirma: o passo é **`2,50×`** mais fino
(aritmética), a profundidade move-se **≤ 5 %** em todas as bandas do regime (medido: `1,7 %`), e a
ondulação pior **desce** (`0,0643 → 0,0407`).

⛔ **O RECURSO é o RELÓGIO**, medido (`--release`, mínimo de três, malha de `185 977` vértices, o
traço da silhueta a `1` px por evento): **`0,472 → 1,074 ms` por evento**, contra o *kill* de `8 ms`
(`13 %` do orçamento). ⚠️ **Ele não cresce só com os dabs:** a granularidade dos candidatos é
proporcional ao passo, logo um espaçamento `2,5×` mais fino pede também `2,5×` mais **raios** contra
a superfície congelada. Sonda: `diag_o_custo_do_traco_aos_dois_espacamentos`.

**Mutação: 4 de 4 sangram** — a porta a ignorar o pincel (`7` de `19` gates de paridade) · o nosso
número a voltar ao do alvo (G-21) · cada uma das **duas** bancadas a deixar de escrever o
espaçamento da fixtura no pincel (`3` de `19`, e `3` de `5`).

---

## §56 — 📦 PARA O AGENTE INTEGRADOR

> ⛔ **Esta linha NÃO integrou e NÃO enviou nada** (`CLAUDE.md` §0.7). O que segue é o que a
> integração precisa de saber, e **só isso** — o mecanismo de cada wave está nas secções acima.

### §56.1 — A linha

| | |
|---|---|
| ramo | `line/sculpt3d` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d` |
| merge-base | `1d43da737` |
| commits | **198** |
| diff | `976` ficheiros, `+75 653 / −1 743` (⚠️ **`634` são `docs/`**, a maioria fixturas de oráculo em `.gz`) |
| árvore | **limpa** — tudo comitado |

**Crates NOVAS (4), todas folhas:** [`ph2d-boundary`](../../../crates/ph2d-boundary/) ·
[`ph2d-pose`](../../../crates/ph2d-pose/) · [`ph2d-trim`](../../../crates/ph2d-trim/) ·
[`ph2d-mesh-bool`](../../../crates/ph2d-mesh-bool/). ⚠️ A última traz **uma dependência externa
nova** (motor de booleana de malha, **Apache-2.0**, Rust puro, feature `parallel` **desligada** de
propósito — §43).

### §56.2 — ⭐ O que NÃO se move (a parte fácil)

**Zero contadores partilhados.** Conferido por diff contra o merge-base: `PROJECT_SCHEMA` ·
`VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `FIELD_DOC_VERSION` · `DOC_VERSION` da timeline · os **três**
registos de componentes — **nenhum** ficheiro que os declara foi tocado.
**Zero contrato congelado** (§6), **zero ADR**, **zero** mudança no `AppHost`.

⚠️ **UMA mudança foundational, e ela é ADITIVA:** o `ph2d-editor-core` ganha a ligação **curva**
pista↔número (`link_slider_number_curved`, §50.3) — nenhuma assinatura pública muda e a identidade
é byte-idêntica no caso linear. *É o único sítio onde esta linha toca código de outra família.*

### §56.3 — ⚠️ O que a integração tem de RE-CORRER (e não pode herdar desta linha)

1. **Tectos de LOC**, que é a única grandeza deste repo que **SOMA entre linhas sem ninguém a
   contar** (§5.0). Esta linha fechou com todos verdes **sozinha**; duas linhas a somar 200 linhas
   cada não acordam gate nenhum até se juntarem. ⇒ `workspace_src_files_under_loc_cap` +
   `the_shell_only_shrinks` **na árvore combinada**, e a cura é **corte por responsabilidade**,
   nunca uma entrada nova no `FILE_OVERAGE_OK`.
2. **As vassouras clean-room**, **todas** as vivas, sobre a árvore combinada — *o sweep é
   propriedade do PAR (código, vassoura)*, e os agentes E estendem as vassouras a cada emenda.
   Instrumento: `bash scripts/cleanroom-sweep.sh <VASSOURA_*.txt> <paths…>`.
3. **Os gates de ARQUITECTURA**, que vivem em crates que esta linha não editou e que um fecho
   por-crate é cego a eles (a lição custou NOVE vermelhos numa integração anterior).
4. **`scripts/ship.sh`** inteiro antes do push, como sempre.

### §56.4 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **O `ESPACAMENTO_DO_AFIADO_PCT` mudou de `5` para `2` e isso NÃO é um desvio de paridade** — é
   ordem do dono (§55), e o corpus continua a medir-se com o número do alvo, que vive agora numa
   constante própria. Há `const _: () = assert!(…)` a prender a relação.
2. **O `TECTO_DAS_PONTAS` DESCEU** (`5e-2 → 2,5e-2`) — uma catraca a apertar, não a afrouxar.
3. **O `TECTO_DO_ULTIMO_SEPARADO` é uma divergência DECLARADA com o mecanismo** (§54.4), e o gate
   **exige que ela exista**: se alguém a fizer desaparecer, ele reprova e manda apagar o tecto.
4. **`passo_do_traco` e `passo_no_mundo` mudaram de assinatura** (`Verb` → `&Brush`). É mecânico;
   o que não é mecânico é a razão: o espaçamento passou a ser um valor do PINCEL.
5. **`passo_no_mundo` devolve `None` ao pincel de plano** — ele declara espaçamento e **não** mede
   no mundo; antes devolvia um número que ninguém devia usar.
6. **As `22` fixturas de `produto/` que são ablação e controlos ficam FORA dos gates**, de
   propósito, com o papel de cada uma na espec §12.2.

### §56.5 — Portão de fecho desta linha (o que já está medido)

`nextest-impacted` **15 368/15 368** verdes na wave do §54 e **15 369/15 369** na do §55 ·
`clippy --all-targets` nas crates da linha: **zero** avisos · `cargo fmt --all` aplicado ·
tectos de LOC verdes · `ph2d-sculpt3d` **565** testes · `ph2d-app-sculpt3d` **206** ·
mutação **17 de 17** nas duas waves (13 + 4) · as **nove** vassouras corridas, com as **três**
acusações medidas como **pré-existentes** (zero adições nesta linha; a triagem é do **R**).

⚠️ **Flake conhecida desta crate**, membro confirmado da família de fan-out do §5.0:
`the_frame_is_hoisted_out_of_the_vertex_loop` — re-rode-a **sozinha**, com o `/proc/loadavg`
impresso ao lado, antes de suspeitar do merge.

### §56.6 — ✅ O smoke do dono: APROVADO (2026-09-17)

⚠️ **A premissa desta secção MORREU e a morte fica visível aqui:** ela dizia *«o §55 não passou pelo
smoke dele»* e era verdade no dia em que foi escrita. O dono smokou a cena `=48` em **17/09** —
incluindo o passo **(6)**, o traço que começa **na beirada** da bola, que é o gesto dos dois reports
— e devolveu **«smoke ok»**.

⇒ **as duas waves desta jornada estão aprovadas pelo dono**: o §54 (o passo medido sobre a
superfície) e o §55 (o espaçamento a `2 %`). *Integrar continua a não ser aprovar* (§5.0) — o que
mudou é que a aprovação agora existe, e a integração já não leva uma mudança de valor de fábrica por
ver.

**Smoke:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=48 cargo run -p ph2d-host-desktop --profile smoke
```

O passo **(6)** do roteiro é o do report: começar o traço **na beirada** da bola e puxar para o meio.

---

## §52 — ⏳ O que fica ABERTO, e de quem é

- **Dono:** o **G-20** (tecto do raio *digitável* `≥ 5 000` px, errata Q2) — a pista vai a `5 000`
  e aceita-o, mas a cena prende o raio na **diagonal da vista** (`2 203` px a 1920×1080), que é o
  recurso nomeado no §50; o gate nasce com os tamanhos de vista nomeados, se o dono o quiser.
- **E:** regenerar as **14** `fab_*` com pincel nosso (nota de proveniência do R-pré); os `25`
  cabeçalhos com `o_que_ela_fixa` vazio e o salto de `20` px sem fixtura (errata Q8); a linha do
  G-1b na espec ainda diz «dívida» e a reprodução existe desde `1809f8b18` (Q9).
- **E/R:** a linha deste pincel no `docs/3D/cleanroom/README.md` ainda diz *«R-pré da §14
  PENDENTE»* — a janela I não lê esse ficheiro.
- **R-pré, ficou de fora dos portões dele:** a varredura achou citações antigas de nomes internos
  do alvo em **16 ficheiros do Painter** (registadas no ledger, para decisão do coordenador).
- **Nossas, nomeadas:** a extensão do centro que **segue a pressão** (o alvo a multiplica pela
  pressão; esta casa não tem pressão de caneta) · a **ordem do deslocamento** contra a memória (o
  deslocamento aplica-se depois da estabilização; o corpus só o exercita com a memória a zero) ·
  o **Sharpen** e os outros que caem no slider cru (§48.2) · os gates G-5, G-6, G-13 · a pegada
  projectada · a máscara no arnês de bancada · o gémeo Motion do defeito do §49 (o undo do grafo
  consome o `Ctrl+Z` antes da escultura) · os passos de PAINEL da sonda do undo são roteiro morto
  com `active=vector`.

**Smoke:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=47 cargo run -p ph2d-host-desktop --profile smoke
```

Diagnóstico do undo: `PH2D_SCULPT3D_UNDO_PROBE=1` com a mesma cena.

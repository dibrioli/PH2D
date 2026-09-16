# Auditoria do vaso — porque o render dele custa `76 ms`, e onde está a alavanca

> **Gatilho (Enio, 2026-09-15):** *«o render do vaso deveria ser mais rápido»*.
> **Resposta:** ele tem razão, e a causa não é a marcha, nem a placa, nem o material.
> **`68` dos `76 ms` são a TESSELAÇÃO DAS QUINAS ARREDONDADAS do contorno que ele desenhou** — e o
> botão *Resolution* dele **não pode ajudar**, porque já está no nível mais grosseiro.

Sondas: `preview::device_probes::audita_o_vaso` e `::audita_o_arredondamento_do_vaso`
(`#[ignore]`, pedem adaptador e máquina calma; correm pela porta com `PH2D_GPU=1`).

---

## §1 — Onde o quadro do vaso é gasto

⚠️ **Um quadro de `76 ms` pode ser `76` de marcha ou `70` de montagem e `6` de marcha, e a tabela
das cenas reais não distingue os dois.** A régua que separa é uma **varredura de resolução**: o
custo por pixel escala com a área, o custo fixo não. Com dois pontos, `fixo = (c₁·a₂ − c₂·a₁)/(a₂ − a₁)`,
e o terceiro ponto é o **controlo** que diz se o modelo de duas parcelas descreve a curva.

| cena | fita | vivos | guardados | `480×270` | `960×540` | `1920×1080` | FIXO | marcha | controlo |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`5` · o VASO** | `2 969` | `33` | `2 504` | `12,96` | `27,46` | **`77,13 ms`** | `8,68 ms` | `68,45 ms` (**89 %**) | erro `6,1 %` |
| `4` · cantoneira | `2 896` | `69` | `2 663` | `16,73` | `36,12` | `112,78` | `10,33` | `102,45` (`91 %`) | erro `0,5 %` |
| `2` · cubo | `28` | `6` | `21` | `2,16` | `2,84` | `5,69` | `1,93` | `3,77` (`66 %`) | erro `1,0 %` |

⇒ **a marcha é `89 %` do quadro do vaso.** O custo fixo (montar a fita, escalonar, emitir o WGSL) é
`8,7 ms` e escala com o tamanho da fita — o cubo paga `1,9`. Não é ele a alavanca.

⚠️ **E o material, o céu e as lâmpadas não aparecem nesta conta** — o pintor lê um G-buffer que a
marcha já produziu. *A última wave pôs o modo RENDER na placa e o tecto continua a ser o traçado.*

## §2 — A marcha é cara porque a FITA é grande, e a fita é grande por um motivo só

O WGSL emitido para o vaso tem **`2 504 lets` e `93 887 bytes`**: `select×555`, `max×280`,
`min×186`, aritmética `×1 849`, e apenas **`sqrt×2`**. ⛔ *Não são as transcendentais* — é volume.

E o volume tem uma origem: **o contorno de `12` pontos que o artista desenha vira `94` arestas.**
Medido, apagando as quinas uma a uma (a mesma peça, `1920×1080`):

| quinas redondas | arestas | fita (ops) | guardados | quadro | ms/aresta |
|---:|---:|---:|---:|---:|---:|
| `0` (quinas vivas) | **`12`** | `335` | `284` | **`8,67 ms`** | `0,723` |
| `2` | `30` | `911` | `770` | `20,24` | `0,675` |
| `5` | `54` | `1 690` | `1 424` | `36,15` | `0,669` |
| **`10` (o vaso real)** | **`94`** | `2 969` | `2 504` | **`76,49`** | `0,814` |

⭐⭐⭐ **`68` dos `76 ms` são o arredondamento das quinas.** Cada quina redonda custa **~`8,2`
arestas** e **~`260` operações de fita**, e o custo é **linear**: `0,67`–`0,81 ms` por aresta ao
longo de toda a faixa. *Isto concorda com a lei que a W56 já tinha medido noutro instrumento —
`0,95 ns` por ponto por aresta, linear perfeito.*

## §3 — ⛔ Por que o botão do artista não resolve

`ph2d_field::DEFAULT_PROFILE_RESOLUTION = 1`, e `tolerance_ratio_for(level) = TOLERANCE_RATIO/level`
⇒ **o nível `1` é o mais GROSSEIRO que existe**. O vaso já está lá. Subir o *Resolution*
**acrescenta** arestas; não há para onde descer.

⇒ *o artista não tem nenhum gesto que torne o vaso dele mais rápido.* É isso que faz disto um
defeito de produto e não uma preferência.

## §4 — ⭐⭐⭐ A alavanca: o perfil não sabe o que é um ARCO

```rust
pub struct Profile {
    contours: Vec<Vec<[f32; 2]>>,   // ⬅ polilinha PURA
    fill: FillRule,
    tolerance: f32,
}
```

Não existe primitiva de arco. Um raio de quina é **tesselado** em ~`8` segmentos rectos, e cada
segmento é ~`32` operações na fita que a marcha avalia **em cada passo de cada pixel**.

⭐ **Um arco exacto custa ~uma aresta e meia e é MAIS preciso que oito.** A distância 2D a um arco é
`|‖p−c‖ − r|` recortada pela cunha angular — da mesma ordem de grandeza da distância ponto-segmento,
e **sem erro de tesselação nenhum**.

Estimativa, a partir das contagens medidas (`32` ops por aresta recta; um arco ~`25` ops):

| | primitivas | fita (ops) | quadro estimado |
|---|---:|---:|---:|
| hoje (tesselado) | `94` | `2 969` | `76,5 ms` |
| com arco exacto | `12` rectas + `10` arcos | ~`630` | **~`15`–`25 ms`** |

⇒ **`3×` a `5×` no vaso, e a silhueta fica melhor, não pior.** ⚠️ **O intervalo é honesto:** o custo
exacto de uma primitiva de arco só se mede depois de ela existir; o que está medido é o preço do que
ela substitui.

⚠️ E o ganho **não é só do vaso**: toda peça desenhada com quinas vivas — a cantoneira da cena `4`
(`102` arestas, `112,78 ms`) — paga a mesma tesselação.

## §5 — ⛔ O que foi verificado e NÃO é a resposta

| hipótese | porque cai |
|---|---|
| «é o material / o céu / as lâmpadas» | o pintor lê um G-buffer pronto; a marcha é `89 %` |
| «são as transcendentais» | `sqrt×2` no vaso inteiro |
| «é o custo fixo de montar a fita» | `8,7` de `76,5 ms`, e escala com a fita (o cubo paga `1,9`) |
| «é a ocupação / os registos» | `vivos = 33` desde o escalonador (§43); o cubo tem `6` e a `4` tem `69` |
| «o *Resolution* resolve» | já está no nível mais grosseiro (§3) |
| ~~«a especialização por região já poda isto»~~ | ⛔ **esta linha mediu o motor errado** (corrigida na Parte III): as sondas cronometram a PLACA, que **nunca** especializa por região — a linearidade não diz nada sobre a especialização. Quem a usa é a CPU (o modo MODEL), e foi lá que as faixas sobreviveram |

## §6 — ⏳ O que fica

- ⏳ **A wave é o arco exacto no `Profile`** — um `enum` de primitiva (`Seg`/`Arc`) em vez de
  `Vec<[f32;2]>`, com `sd_profile_inner` a emitir a distância certa para cada uma. ⚠️ Ela atravessa
  o **cozimento** (`cook_path` tem de PRESERVAR o arco do Live Corner em vez de o achatar), a
  **regra de preenchimento** (o winding de um arco não é o de um segmento) e o **`ProfileIndex`**.
  Nada disto é contrato congelado;
- ⏳ **O `select×555` não foi auditado** — são ~`5,9` por aresta, e a hipótese é o teste de winding
  do preenchimento. Se metade dele for redundante com o `min` da distância, é uma segunda alavanca
  **na mesma fita**, e essa não precisa de modelo novo;
- ~~⏳ A especialização por região não está a morder neste caso~~ — ⛔ **premissa errada** (Parte
  III): a sonda cronometrava a placa, e a placa não especializa. A especialização é da CPU.

---

# PARTE II — A CURA, implementada (2026-09-16)

> **Ordem do dono:** *«implemente a cura. Não pare até finalizar, pronto para smoke»*.
> **Resultado medido, `1920×1080`, máquina a `95 %` ociosa, duas rondas concordantes:**

| cena | fita (ops) | antes | depois | ganho |
|---|---:|---:|---:|---:|
| **`5` · o VASO** | `2 969` → **`915`** | `77,13 ms` | **`25,6`–`26,2 ms`** | **`2,95×`** |
| `4` · cantoneira | `2 896` → **`519`** | `112,78 ms` | **`13,2 ms`** | **`8,5×`** |
| `2` · cubo (sem arcos) | `28` → `28` | `5,69` | `4,3`–`4,8` | *inalterado* |

⭐ **O cubo é o CONTROLO e a fita dele é byte-idêntica** (`28` operações, `21` guardados, os mesmos
`558` bytes de WGSL): o caminho sem arcos não foi tocado. ⚠️ O relógio dele mexeu-se `~12 %` entre
corridas — *é essa a dispersão da máquina, e é ela que diz que um ganho de `12 %` não seria
afirmável e um de `2,95×` é.*

## §8 — O que se construiu

1. **`ph2d_field::Profile` ganhou `arcs`** — por contorno, a lista de `(vértice, bulge)`. O *bulge* é
   a convenção do DXF (`tan(θ/4)`, com sinal), e dele mais a corda saem o centro e o raio sem
   guardar nenhum dos dois.
   ⚠️⚠️ **A `contours()` continua a ser A FIGURA.** A 1.ª tentativa pôs os bulges *paralelos* à
   polilinha e com isso a polilinha deixou de descrever a forma — **dois gates que já existiam
   apanharam-no na primeira corrida** (a tolerância do achatamento passou a medir a corda; um furo
   mediu `0,2828` em vez de `0,4`). *Vinte e quatro leitores tratam `contours()` como a figura, e
   estavam certos.* ⇒ o arco é uma vista **adicional**, e há gate a atar as duas.
2. **A cozedura reconhece o arco** (`bulge_do_cubico`) — pela **geometria**, nunca por proveniência:
   constrói o único círculo pelos dois extremos e pelo meio da cúbica, e depois **confere** que a
   cúbica inteira vive nele, com a tolerância de cozimento como barra. *Ajustar um círculo por
   mínimos quadrados aceitaria uma curva que passa perto de um círculo sem ser um.*
3. **O emissor da fita sabe arcos** — distância por cunha angular (`|‖p−c‖−r|` dentro dela, a ponta
   mais próxima fora), **sem uma única trigonométrica por amostra**: o centro, o raio, a bissectriz e
   o `cos(θ/2)` são constantes.
4. **O SINAL** — a corda entra no enrolamento como uma recta, mais a **correcção da meia-lua**: um
   ponto dentro do círculo e do lado do arco vale `∓1`. ⭐ **Provado por mutação:** apagar a correcção
   troca o sinal em **`652`** pontos de uma grelha de `68 121`.

## §9 — ⭐⭐⭐ E a auditoria achou um DEFEITO MAIOR pelo caminho

O reconhecimento falhou nas dez quinas do vaso, e a causa não era o reconhecedor: **as quinas não
eram arcos**. O `ph2d_vec_scene::corner_live::fillet_handles` emitia
`h = (4/3)·tan(α/4)·**s_in**` onde a lei do arco pede `·r`, e `s_in = r·tan(α/2)` — ou seja um factor
`tan(α/2)` a mais, que vale `1` **só a `90°`**.

| α | raio pedido | raio que saía | desvio | × tolerância |
|---:|---:|---:|---:|---:|
| `30°` | `0,05` | `0,04875` | `1,2e-3` | `4,2` |
| `70°` | `0,05` | `0,04729` | `2,7e-3` | `9,1` |
| **`90°`** | `0,05` | **`0,05000`** | `1,4e-5` | `0,0` ⬅ o único certo |
| `130°` | `0,05` | **`0,08304`** | `3,3e-2` | `110,7` |

⛔⛔ **O artista pedia `0,05` e recebia `0,083`** — e isto vale para **toda** quina arredondada do
app fora de `90°`, não só no modelador.

⚠️⚠️ **E o gate que devia tê-lo apanhado corre sobre um QUADRADO**
(`the_fillet_agrees_with_the_crates_canonical_corner_rounding`): quatro cantos a `90°`, que é
exactamente o ângulo em que o defeito é invisível. *Uma fixtura de um ângulo só não mede uma lei que
depende do ângulo* — a mesma família do «uma fixtura alinhada aos eixos não mede uma base».

⭐ Curado (uma divisão), e o gate novo (`o_filete_vivo_e_um_arco_em_todo_angulo`) varre `30°`–`150°`.
Depois da cura o raio sai **exacto** em toda a faixa. ⚠️ **A fidelidade tem um tecto que não é
desta wave:** UMA cúbica não representa um arco de `150°` dentro da tolerância (`1,04×` dela), e a
consequência é benigna — o reconhecedor simplesmente não aceita essa quina e ela fica tesselada.

⭐ **A `ph2d-vec-scene` passou `501/501`** com a cura dentro.

## §10 — ⛔ As duas RÉGUAS que mentiram primeiro

1. **Ponto→VÉRTICE em vez de ponto→SEGMENTO.** A régua que compara as duas vistas acusou uma
   decomposição **correcta** com `0,0028`–`0,0032` de erro, *uniformemente em todos os dez arcos*.
   Numa polilinha achatada a `9,7e-5` sobre um arco de raio `0,05` os vértices ficam a `~0,0062` um
   do outro ⇒ um ponto no meio de dois lê `~0,0031`. **Um desvio igual em todas as amostras é
   assinatura da régua, não do objecto** — e a lição já estava paga neste repo (a régua da ponta).
2. **O sentido do arco assumido em vez de derivado.** A 1.ª amostragem usou `θ = 4·atan(bulge)` com
   o sinal de uma convenção decorada; com `|bulge| < 1` o arco é o MENOR, logo a diferença de ângulos
   dobrada para `(−π, π]` **é** ele, e não há convenção para decorar.

## §11 — O formato

`FIELD_DOC_VERSION` **22 → 23** e `PROJECT_SCHEMA` **131 → 132**. O `Profile` viaja dentro de uma
`Primitive`, que viaja **posicionalmente** no blob do `FieldNode`: é a regra dos degraus 109/110
(campo novo numa struct já gravada), e não a dos `enum` que apendam. ⛔ Sem degrau de migração, pela
decisão do Enio de 26/08 — um v131 é recusado em voz alta. ⭐ Um documento velho abre com `arcs`
vazio e é avaliado pelo caminho de sempre, **ao bit**.

## §12 — ⏳ O que fica

- ~~⏳ **A especialização por REGIÃO não usa arcos** … não é defeito de correcção … é **ganho por
  colher**~~ — ⛔⛔ **ESTA NOTA ESTAVA ERRADA, e o smoke do dono desmentiu-a no dia seguinte** (Parte
  III): era defeito, e **visível**. O modo MODEL traça na CPU pela especialização por região, e a
  polilinha densa que ela lia tem normais que saltam a cada segmento — *«descrevem a mesma curva a
  menos da tolerância» é uma frase sobre VALORES, e a luz lê a NORMAL* (a lição da W54, repetida por
  mim). Curado na Parte III;
- ⏳ **O `select×555` do vaso** era `~5,9` por aresta e continua por auditar na fita nova;
- ⏳ **Uma quina acima de `~140°` fica tesselada**, porque uma cúbica não a representa dentro da
  tolerância. A saída publicada é partir a quina em duas cúbicas na emissão — wave da
  `ph2d-vec-scene`, não desta.

---

# PARTE III — as faixas que SOBREVIVERAM à cura (2026-09-16)

> **Smoke do dono, com foto:** *«arestas ainda visíveis»* — faixas horizontais de luz no ombro do
> vaso, no modo MODEL, com a cura da Parte II dentro do binário.
> Registo completo do bug: [`docs/3DModeling/BUGS_3dmodeling.md`](../3DModeling/BUGS_3dmodeling.md) #1.

## §13 — O mecanismo: dois motores, e a cura só tinha chegado a um

O modo **RENDER** traça na placa; o modo **MODEL** traça na **CPU**, e a CPU especializa a árvore por
ladrilho — a folha especializada lia o perfil por um `ProfileIndex` construído da **polilinha densa**.
A Parte II pôs os arcos só na árvore **global**, logo só a placa ficou lisa.

| motor (cena `5`, frente, `1920×1080`) | picos de faceta | maior pico |
|---|---:|---:|
| CPU, antes | **12 196** | **11,42°** |
| placa, antes | 0 | — |
| CPU com a especialização desligada (experiência) | 0 | — |
| **CPU, depois** | **0** | — |

## §14 — A cura: a lei do arco numa porta só

`ph2d_field_eval::profile_arc` — as constantes (`arco`), a distância (árvore e escalar) e a meia-lua
(para cada um dos dois caminhos do enrolamento) — lida pelos **três** leitores: a árvore global, a
árvore por região e o `ProfileIndex`, que passa a indexar a **decomposição exacta** quando ela existe.

⭐ **O corte espacial continua CONSERVADOR sem conhecer o arco**: todo ponto do arco está a menos da
flecha da corda, logo `d(região, arco) ≥ d(região, corda) − flecha` (minorante) e `d(p, arco) ≤ d(p,
corda) + flecha` (majorante, cujo máximo sobre a região continua num vértice porque a distância à
corda é convexa).

## §15 — ⛔⛔ Dois defeitos a mais, achados pelos gates novos

1. **O empate sobre a corda, na árvore global — meu, da Parte II.** Um ponto EXACTAMENTE sobre a corda
   lia o sinal trocado (`+0,0437` a `0,0437` de profundidade dentro da peça). O enrolamento pelo raio
   põe esse ponto do lado `−dir`; a meia-lua da Parte II usava um teste estrito. ⇒ cada caminho tem a
   sua regra de empate e a meia-lua copia-a (ver o cabeçalho do `profile_arc`).
   ⚠️ **O gate da Parte II nunca o viu**: a grelha dele nunca caiu numa corda. *Os pontos PÕEM-SE.*
2. **O canto de célula sobre uma aresta, no índice — antigo.** A partida do caminho do sinal era um
   canto da grelha, com o enrolamento tirado do RAIO e o caminho do SEMI-ABERTO; sobre uma aresta
   discordam. É o defeito da W56 um nível abaixo; as cordas redondas punham arestas nos cantos. Só a
   **paridade** o denunciou (`NonZero` lia `−2`, que também é «dentro»).

## §16 — As réguas, e as duas que caíram

- ⛔ **O salto MÁXIMO da normal** leu `82–85°` nos dois motores — o eixo e as oclusões. ⇒ picos
  isolados com continuidade no mundo.
- ⛔ **O primeiro vermelho do gate por região era pelo motivo errado** (`0,088` de discordância, muito
  acima da flecha de uma faceta) — o **controlo sem arcos** leu `0`, o que provou o gate são e apontou
  o defeito #1 acima.

## §17 — As provas

**8 mutações, 8 mortas** (a tabela está no registo do bug). ⚠️ **Duas sobreviveram à 1.ª ronda** —
esquecer a flecha no minorante ou no majorante do corte —, porque nenhuma região do corpus caía onde
a flecha decide; o gate `o_corte_por_regiao_guarda_a_primitiva_que_a_flecha_decide` tem as duas
regiões construídas à mão («cintura» e «pé»), com a conta ao lado.

⭐ **O oráculo é analítico**: a distância com sinal do *round box* julga os três leitores nos pontos
postos sobre as cordas, nas quatro combinações de sentido × preenchimento.

## §18 — ⏳ O que fica

- ~~⏳ **O `select×555` do vaso** continua por auditar na fita nova~~ — ✅ auditado no §23 (é o
  `compare`, três por cada um);
- ~~⏳ **Uma quina acima de `~140°` fica tesselada**~~ — ✅ **curado na Parte IV**, e a nota tinha o
  limiar errado: a quina fica tesselada acima de `~150°` no nível 1 e acima de **`~90°`** nos
  outros, porque a barra se dividia pelo nível;
- ~~⏳ **A lei do SEGMENTO continua escrita em dois sítios**~~ — ✅ numa porta só desde a Parte IV
  (`profile_arc::dist2_recta_tree`, §22).

---

# Parte IV — O arco que não sobrevivia ao botão (auditoria depois do smoke aprovado)

> O smoke da Parte III foi aprovado no nível de **omissão** do `Resolution`. A pergunta desta parte:
> *e com o botão noutro sítio?* Registo completo: [`BUGS_3dmodeling.md`](../3DModeling/BUGS_3dmodeling.md) #2.

## §19 — Três mecanismos, medidos

| `(arcos, primitivas)` que chegam ao TRAÇADOR | nível 1 | nível 4 | nível 16 | nível 64 |
|---|---|---|---|---|
| vaso, antes, cozedor | `(10, 22)` | `(8, 71)` | `(8, 122)` | `(6, 384)` |
| vaso, com as curas 1+2, **pela app parado** | `(12, 24)` | `(12, 24)` | **`(0, 329)`** | **`(0, 392)`** |
| vaso, com as curas 1+2, **pela app a mexer** | `(12, 24)` | **`(0, 168)`** | **`(0, 198)`** | **`(0, 241)`** |
| círculo `r = 0,5`, antes, cozedor | **`(0, 168)`** | `(0, 332)` | `(0, 664)` | `(0, 1 328)` |
| **depois das três curas, pela app** — vaso · círculo | `(12, 24)` · `(4, 4)` | igual | igual | igual |

1. **A barra do reconhecedor era a tolerância de achatamento** — e uma cúbica erra `2,7253e-4·r` de
   um círculo por construção. ⇒ `max(tol, ERRO_DO_QUARTO·r)`.
2. **Os arredondadores de quina escreviam arcos acima de `90°` numa cúbica** (a casa já tinha a lei
   dos `90°` escrita no `shapes::arc`). ⇒ `corners::circular_fillet`, duas metades, byte-idêntico até
   `90°`.
3. ⛔⛔ **O preview (`coarse_doc`, a mexer e parado) trocava arcos por polilinha** — comparava
   `segment_count`, e o custo da marcha é o `prim_count`. *Os gates do cozedor estavam certos sobre
   o cozedor; a app traça o que o preview lhe dá.*

⭐ E um defeito da Parte II: a porta dos arcos pedia **três** primitivas e recusava inteira a
meia-lua de dois pontos.

## §20 — O que a divisão custa, e onde

Uma quina aguda passa a ter **um vértice a mais** no desenho vectorial (a estrela de 5 pontas: `15 → 20`).
O 2D fica **mais fiel** ao raio pedido (uma ponta de `150°` errava `5,97e-3·r`, hoje `≤ 2,73e-4·r`), e
o 3D ganha o arco em todo nível. Portão dos impactados: `15 236` de `15 237`, e o vermelho era essa
contagem.

## §21 — As réguas que mentiram primeiro

- ⛔ **A imagem do vaso INTEIRO** não vê as facetas de um nível alto (menos de um pixel cada) — verde
  com as curas desfeitas.
- ⛔ **A imagem aproximada em PERSPECTIVA** pôs o olho dentro do vaso (`half_extent 0,05` ⇒ olho a
  `0,11` do eixo, parede a `0,33`) — verde outra vez. Lente **paralela**: `0` picos com a cura,
  **`10 020` (maior `6,23°`)** sem a divisão.
- ⛔ **A diferença entre duas imagens** leu `91 770` pixels em duas corridas IGUAIS — `acos` de
  normais idênticas com `|n| < 1` em `f32`.
- ⛔ **Um controlo que nunca aperta**: o arco de `150°` numa cúbica (erra `22×` a barra) deixou passar
  uma barra `10×` mais larga; o de `95°` (`1,38×`) mata-a.

**12 mutações, 12 mortas** (tabela no registo do bug).

## §22 — ⏳ O que fica

- ✅ **O `select×555` foi auditado (§23)** — e a pergunta mudou;
- ✅ **A lei do SEGMENTO numa porta só** (`profile_arc::dist2_recta_tree`, lida pelas duas árvores) —
  a fita do dispositivo sai **igual ao bit** (impressão do WGSL antes/depois: vaso, polilinha,
  cantoneira e vaso no nível 16), `76` gates de perfil/arco/região verdes, mutação `H1` morta
  (`5` gates). ⭐ E a mesma impressão mostrou de passagem que **o vaso no nível 16 dá a MESMA fita
  que no nível 1** — o botão já não desfaz os arcos;
- ✅ **O reconhecedor amostrava seis pontos** — e a nota que estava aqui (*«subestima `~8 %`»*) era
  ela própria uma leitura por amostras: o pico de um arco numa cúbica cai em `t = (3 − √3)/6`, e seis
  pontos leem-no **`5,35 %`** abaixo em todo ângulo. Hoje é uma grelha de `16` com secção áurea em
  cada máximo local, e a barra aplicada é a escrita (um arco de `90,03°` é recusado; nenhuma grelha
  fixa de até `32` o recusava). Preço: `~1,2 µs` por aresta que é arco
  ([`BUGS_3dmodeling.md`](../3DModeling/BUGS_3dmodeling.md), adenda ao #2);
- ✅ **O `coarsen` deitava fora os arcos de um contorno MISTO** — e era visível: um contorno com duas
  quinas e uma onda chegava ao traçador `(0, 93)` em vez de `(2, 237)` no nível 64 a mexer. Hoje a
  decomposição é decimada só nos troços tesselados, os arcos ficam iguais e a pré-visualização sai
  `(2, 22)` (Bug #6);
- ✅ **A suavização de quina** (`smooth_corner`) escrevia o arco curto numa cúbica só. Medido: só o
  `RoundRect` a alcança no produto (quinas de `90°`, arco `≤ 90°`), logo não era visível; a porta
  passa agora pela mesma `circular_fillet`, byte a byte igual até `90°`.

## §23 — O `select` da fita: é o `compare`, três por cada um

Medido pelo WGSL que o dispositivo recebe (`DeviceField::tape_wgsl`, CPU, sem placa), no nível 1:

| peça | `let` | bytes | `select(` | ` > ` | ` != ` | `min(` | `max(` | `sqrt(` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **vaso, com arcos** (`24` primitivas) | **`931`** | `35 588` | **`267`** | `89` | `178` | `46` | `118` | `14` |
| vaso, só a polilinha (`97` arestas) | `2 639` | `99 097` | `585` | `195` | `390` | `196` | `295` | `2` |
| cantoneira (cena `4`) | `500` | `17 977` | `126` | `42` | `84` | `29` | `77` | `14` |
| cubo (cena `2`, controlo) | `21` | `558` | `0` | `0` | `0` | `1` | `6` | `1` |

⭐ **`select = 3 × (>)` exactamente, nas três peças** — e é a lei do rebaixamento
(`ph2d_field_eval::wgsl::binary`): o `compare` da `fidget` (`partial_cmp` → `−1/0/1`, e `NaN` quando
não comparáveis) vira `select(select(select(0, 1, a > b), −1, a < b), NaN, a != a || b != b)`. ⇒ o
vaso tem **`89` comparações** (`~3,7` por primitiva: o enrolamento, a cunha do arco, a meia-lua) e
cada uma custa **8** operações de placa contra `~1,5` de um `let` comum — **`~36 %`** do shader.

⏳ **A alavanca está nomeada e NÃO foi aplicada** — pede uma medição que hoje não se pode fazer (a
placa com o smoke de outra linha, load `~12`):
- a guarda de `NaN` (`a != a || b != b` + um `select`) é metade do custo de cada comparação. Tirá-la
  muda a resposta **só** para entradas `NaN` — e na placa o `min`/`max` com `NaN` já não tem
  semântica garantida (WGSL), logo a paridade com a CPU nesse caso é hoje nominal. ⚠️ **Mas é uma
  mudança de semântica escrita de propósito** (o comentário do rebaixamento cita o `NaN`), e os gates
  de paridade CPU×placa são quem a julga;
- o padrão dominante nas leis do perfil é `compare(x, y).max(0)` — um degrau `0/1` — que caberia num
  `select(0, 1, x > y)`: **um** `select` e **uma** relação. Exige um *peephole* entre dois `let`.
⇒ o A/B tem de medir o **quadro** do vaso (o mínimo de N corridas, com a carga ao lado) antes e
depois, e só entra com o número. *A contagem de operações não é um perfil* (a lição da W147).

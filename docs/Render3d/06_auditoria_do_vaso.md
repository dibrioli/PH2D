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
| «a especialização por região já poda isto» | o custo é **linear** nas arestas de `12` a `94`; se a poda mordesse, a curva seria sublinear |

## §6 — ⏳ O que fica

- ⏳ **A wave é o arco exacto no `Profile`** — um `enum` de primitiva (`Seg`/`Arc`) em vez de
  `Vec<[f32;2]>`, com `sd_profile_inner` a emitir a distância certa para cada uma. ⚠️ Ela atravessa
  o **cozimento** (`cook_path` tem de PRESERVAR o arco do Live Corner em vez de o achatar), a
  **regra de preenchimento** (o winding de um arco não é o de um segmento) e o **`ProfileIndex`**.
  Nada disto é contrato congelado;
- ⏳ **O `select×555` não foi auditado** — são ~`5,9` por aresta, e a hipótese é o teste de winding
  do preenchimento. Se metade dele for redundante com o `min` da distância, é uma segunda alavanca
  **na mesma fita**, e essa não precisa de modelo novo;
- ⏳ **A especialização por região não está a morder neste caso** e ninguém mediu porquê. A
  suspeita é a simetria do torno: um ladrilho do ecrã mapeia para uma faixa larga de `(r, y)`.

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

- ⏳ **A especialização por REGIÃO não usa arcos.** O `ProfileIndex` é construído da polilinha densa,
  logo o caminho por ladrilho continua a pagar por aresta. Não é defeito de correcção (as duas
  descrevem a mesma curva a menos da tolerância, e as suítes passam), é **ganho por colher** — e é
  onde o vaso ainda tem `22 ms` de marcha;
- ⏳ **O `select×555` do vaso** era `~5,9` por aresta e continua por auditar na fita nova;
- ⏳ **Uma quina acima de `~140°` fica tesselada**, porque uma cúbica não a representa dentro da
  tolerância. A saída publicada é partir a quina em duas cúbicas na emissão — wave da
  `ph2d-vec-scene`, não desta.

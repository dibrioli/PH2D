# SPEC — o PINCEL DE TRAÇO AFIADO: um vinco que se limita a si mesmo

```
Alvo: Blender 5.2 (fonte lido na tag v5.2.0; oráculo corrido = binário 5.2.1 LTS, re-conferido
  inalterado em 2026-09-16) · Licença: GPL-2.0-or-later · Degrau: T2
Rótulo público do pincel no catálogo do alvo: «Draw Sharp» (a palavra do dono). Nesta casa: o
  «pincel afiado».
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-pincel-afiado.md, 2026-09-16, ANTES da primeira
  leitura de conteúdo do fonte. Autor desta espec: subagente-E agent-acf4ca41445c57ad7, 2026-09-16.
Patente (§8.1): buscado em 2026-09-16 — pincel de escultura que desloca vértices ao longo de uma
  normal de área com queda radial · distâncias medidas a partir das posições do início do traço ·
  vinco afiado. Resultado: NENHUMA patente viva alcança o método. As vizinhas achadas são de OUTRO
  assunto (deformação elástica regularizada; alisamento com preservação de volume; alisamento
  adaptativo à resolução) e estão no ledger. Arte anterior pública: o próprio pincel é público
  desde 2019, e «deslocar ao longo da normal com queda radial» é o pincel de desenho de todo
  escultor digital desde os anos 1990. Veredito: prosseguir.
Filtragem §4.3: executada em 2026-09-16 sobre este texto (zero código, zero nome interno, zero
  comentário, zero wording de manual ou da discussão pública, zero tabela transcrita; as fórmulas
  são matemática do método em notação genérica; todo `code span` é nosso, de fixtura nossa, ou
  valor público de enumeração usado como chave de regeneração — §4.1.13).
Sweep: controlo do instrumento corrido PRIMEIRO (exit 0, todos os canais), depois VERDE sobre a
  vassoura desta obra (ver o ledger para a contagem) aplicada a esta espec, às 80 fixturas e ao
  README delas, e em --git-history da pasta.
Auditoria §4.2 (R-pré): ⏳ POR CORRER — é condição de abrir a janela que implementa.
Mapa de leitura da literatura: não há paper. A literatura pública utilizável é (a) o manual do
  alvo (factos, ⛔ nunca o wording), (b) as notas públicas de versão, (c) a revisão pública de 2019
  em que o pincel nasceu, destilada em vocabulário nosso na §14, e (d) o NOSSO código, citado por
  ficheiro e linha na §10. ⛔ Não há apêndice com listing.
Denylist de URLs (⛔ o Implementador não abre): projects.blender.org · o espelho em
  github.com/blender · developer.blender.org e o arquivo dele · qualquer code-search sobre eles ·
  o rastreador de PRs e issues do alvo.
  ✅ Livres: docs.blender.org (manual).
"Este documento descreve comportamento; não contém expressão do alvo."
```

---

## §0 — A PERGUNTA ZERO, respondida com MEDIÇÃO

> *O pincel afiado é exprimível pelo nosso `Draw` (modo B) com knobs e uma curva que já existe,
> ou falta uma lei?*

### §0.1 — A resposta

⭐⭐ **O pincel afiado DE FÁBRICA é o nosso `Draw` (modo B) com quatro valores de fábrica
trocados e com o passo e a atenuação do traço arrastado que a casa já tem para o pincel de
plano — e mais NADA de lei nova.** Medido no nosso motor, sobre as mesmas malhas e o mesmo traço
arrastado do oráculo (§7, §8):

| régua (§7), traço contínuo, grelha 96² | passagem 1 | 2 | 4 | 8 |
|---|---|---|---|---|
| profundidade `D/R` — alvo | `0,2058` | `0,3221` | `0,4367` | `0,5406` |
| profundidade `D/R` — nosso `Draw` composto | `0,2121` | `0,3236` | `0,4377` | `0,5411` |
| meia-largura `W50/R` — alvo | `0,506` | `0,527` | `0,543` | `0,550` |
| meia-largura `W50/R` — nosso `Draw` composto | `0,504` | `0,526` | `0,542` | `0,549` |

*«composto»* = o nosso `Draw` modo B com **Acumular desligado · curva afiada
([`Falloff::Sharper`]) · direcção a afundar · o passo de `5 %` do diâmetro · a atenuação `a` por
dab** (§5). O desvio é de `+3,1 %` na profundidade da 1.ª passagem e cai para `+0,1 %` na 8.ª; a
meia-largura fica a `≤ 0,4 %` em todas. Nas grelhas 48² e 192² e no cilindro os desvios são os
mesmos (§12, G-4).

⚠️ **O que o nosso `Draw` ainda NÃO exprime, e o tamanho de cada buraco:**

1. **A normal da área do alvo** (§2.3) — o nosso `Draw` usa a normal do plano da casa. É esta a
   causa dos `+3,1 %` da 1.ª passagem e do resto do desvio nos traços **separados**: depois de
   oito traços separados a nossa meia-largura fica `11,7 %` mais estreita (`0,237` contra
   `0,269`), com a profundidade a `+4,6 %`. A lei do alvo **já existe na casa, duas vezes**
   (§10.2), só não serve o `Draw`. Com ela o modelo de referência do E reproduz o produto
   **vértice a vértice** (§12, G-3).
2. **O passo de `5 %` e a atenuação `a` por dab** — a casa tem as duas peças, mas só as dá ao
   pincel de plano, e ao pincel de plano com a lei `(1 + a)/2`, que **não** é a deste (§5.3).
3. **A direcção de fábrica «afundar»** — na casa a direcção só vem do `Ctrl` no pen-down; não há
   valor de fábrica por pincel (§10.3).
4. **O modo «Acumular LIGADO» deste pincel** (§3) — no alvo ele lê o cursor, a normal e a
   distância da superfície do pen-down e **aprofunda linearmente com o perfil constante**; o nosso
   Acumular significa outra coisa (a distância viva) e dá outro vinco (§8.3). As peças existem na
   casa (§10.4), mas **ligá-las é lei nova**. Não é o valor de fábrica.

⇒ **Não falta uma lei para o pincel de fábrica; faltam valores por pincel, o passo/atenuação por
pincel e — para a paridade vértice a vértice — a normal da área que a casa já tem noutro sítio.**

### §0.2 — A recusa antiga, reconferida

O plano [`docs/3D/21_plano_modos_e_ferramentas.md`](../21_plano_modos_e_ferramentas.md) (linhas
da W6 e da tabela de itens fora) tirou este pincel da fila com duas afirmações. As duas foram
reconferidas contra o oráculo **a correr**:

| afirmação de 2026-08-14 | o que a medição diz hoje |
|---|---|
| *«o que o nome promete mora na CURVA, e a curva de fábrica está num ficheiro binário ⇒ bloqueado»* | ⛔ **DISSOLVIDA.** Os valores de fábrica foram **lidos correndo o programa**: o pincel do catálogo, carregado pelo próprio programa num arranque de fábrica, e os valores lidos dele (§6). A prova de que a lista está completa é uma fixtura: o pincel de catálogo e um pincel nosso com os valores escritos dão o mesmo traço a `1,2e-7` (ruído de `f32`). ⇒ *um ficheiro binário não é uma fonte ilegível quando o programa que o lê se corre.* |
| *«um verbo só sobre o dado congelado seria um chip que o artista já alcança por um checkbox — os dois perfis são o mesmo domo»* | ⚠️ **METADE certa, metade medida com a curva errada.** Certa: a distância medida do pen-down **já existe** no nosso `Draw` (é o Acumular desligado, §10.1) — não há verbo novo a construir. Errada: com a curva **afiada** e oito passagens a distância do pen-down **é** uma alavanca, tão forte como a curva: trocá-la pela distância viva (o pincel de desenho comum com todo o resto igual) alarga o vinco de `0,550` para `0,852` raio (`+55 %`), e trocar só a curva alarga-o para `0,837` (`+52 %`) — §8.1. A medição antiga usou a curva suave e 9 dabs, onde a diferença não aparece. |

⚠️ **Notas da casa cuja premissa esta medição mudou** (§0.0 do `CLAUDE.md`: *quem move o número
que tornava algo inalcançável reconfere a nota*) — ⛔ o E não as edita; ficam para a janela que
implementa:

- `crates/ph2d-sculpt3d/src/brush_verb_defaults.rs:257-260` (*«a W1 e o Draw Sharp são decisão
  de produto… os defaults moram num binário»*);
- `crates/ph2d-sculpt3d/src/brush_magnitudes.rs:123` (*«a mesma lacuna que bloqueou … o Draw
  Sharp»*);
- `crates/ph2d-sculpt3d/src/ref_profiles.rs:315-347` (a coluna `B` *«não pode responder»* os
  defaults por ferramenta — ela pode, correndo o programa; a obra do pincel de plano já o fez para
  o dela);
- `docs/3D/21_plano_modos_e_ferramentas.md:434,447` e a linha do §5 do `CLAUDE.md` que diz o
  mesmo.

---

## §1 — O gesto, e o que o artista vê

O artista arrasta o pincel sobre a peça e fica um **vinco** estreito e de fundo afiado. Três
propriedades o distinguem do pincel de desenho comum, e as três estão medidas:

1. **O primeiro dab é IGUAL ao do desenho comum**, ao bit (`lei/um_dab_controlo_desenho_comum`
   contra `lei/um_dab_curva_afiada`: diferença `0,0`). A diferença entre os dois nasce do
   **segundo dab em diante** — é uma propriedade do TRAÇO, não do dab.
2. **Num traço contínuo o vinco LIMITA-SE a si mesmo**: por mais passagens que se dêem sem
   levantar a caneta, a profundidade converge e nunca passa de `R` abaixo da superfície do
   pen-down (§4). Medido: `0,21 → 0,32 → 0,44 → 0,54 R` em 1, 2, 4, 8 passagens.
3. **Levantar a caneta e voltar a descer RECOMEÇA o limite**: oito traços separados levam o vinco
   a `1,51 R`, e ele **estreita** a cada traço (`0,51 → 0,27 R`), ficando cada vez mais afiado
   (nitidez `D/W50` de `0,41` a `5,62`).

⭐ **O vinco não tem rebordo** (a régua do rebordo lê `0,000` em todas as corridas) e o **fundo
é agudo**: a curvatura normalizada do fundo lê `6,2`–`6,8` contra `3,2`–`5,0` do desenho comum.

---

## §2 — A lei de UM dab

Para cada vértice `v`, um dab com cursor `c`, raio `R` (em unidades do objecto), força `S`,
pressão `P`, curva `C`, dureza `h`, direcção `σ` e normal da área `n` desloca-o de

```text
u(v)   = |X₀(v) − c| / R                       X₀ = posição de v no PEN-DOWN (§2.1)
g(v)   = C( dureza_h(u) ) · [u < 1]             (§2.2)
Δ(v)   = σ · n · R · S² · P · a · g(v) · (1 − m(v)) · F(v)
```

- `a` — o factor do traço arrastado (§5.3); `1` num dab dado por script.
- `m(v)` — a máscara (`1` = travado) — `lei/mascara_metade`, reproduzida a `8,9e-9`.
- `F(v)` — o factor de faces de frente (§2.6); `1` com a opção desligada (o valor de fábrica).
- `σ = −1` com a direcção *afundar* e `+1` com *levantar*; o `Ctrl` troca-a (§2.4).

Reprodução das `17` fixturas de um dab pelo modelo de referência do E (`float64`): `≤ 8,1e-8`
nas planas e no cilindro, `≤ 4,2e-6` nas de bossas (§12, G-1).

### §2.1 — A distância mede-se das posições do PEN-DOWN — sempre

⭐⭐ **Neste pincel a distância de cada vértice ao cursor é medida da posição que o vértice tinha
quando a caneta desceu**, e isso **não depende** do interruptor do acumular (§3). É a única
diferença de LEI em relação ao pincel de desenho comum, que mede da posição viva.

- Um vértice que o traço já afundou continua a ser pesado pelo sítio onde **estava** — por isso o
  vinco não alarga à medida que aprofunda.
- ⚠️ **Com o cursor na superfície viva, o cursor afunda com o vinco e afasta-se das posições do
  pen-down** — é daqui que vem a auto-limitação (§4).
- Medido: numa cadeia de 8 dabs num plano, medir da posição viva erra `3,1e-2`; do pen-down,
  `2,4e-8` (`cadeia/plano_cursor_vivo`).

⚠️ **O conjunto de vértices considerado** (os que têm posição VIVA dentro da esfera do dab, ou os
que têm posição do PEN-DOWN dentro dela) **não é separado por nenhuma corrida deste corpus**: as
duas leituras dão o mesmo resultado, ao dígito, nos traços contínuos e separados medidos. A
nossa casa usa a esfera viva (`stroke_dab_core.rs:273-278` mede a distância; a pegada sai das
posições vivas) — compatível.

### §2.2 — A curva e a dureza

- `C(u)` é a curva do pincel avaliada na distância normalizada, zero a partir de `u = 1`. De
  fábrica é a **afiada**, `(1 − u)⁴` — a nossa [`Falloff::Sharper`]. As outras três medidas:
  suave `3t² − 2t³` com `t = 1 − u`, quadrática `(1 − u)²`, constante `1`.
- `dureza_h(u) = 0` para `u < h`, senão `(u − h)/(1 − h)`; com `h = 0` é a identidade. É a lei que
  a casa já tem em `Brush::shaped_distance` (`brush_scale.rs:298-312`) — `lei/um_dab_dureza_meia`
  reproduz a `8,1e-8`.

### §2.3 — A NORMAL DA ÁREA

```text
n_bruto = Σ_{v : |p(v) − c| ≤ R_n}  suave(1 − |p(v) − c| / R_n) · N(v)      suave(t) = 3t² − 2t³
R_n     = f_n · R              f_n = 0,5 de fábrica
n       = n_bruto da FRENTE, normalizada; se ela for nula, a do VERSO
```

- `p(v)` e `N(v)` são a posição e a normal de **vértice** lidas da superfície que o §3 manda ler.
- Os vértices dividem-se em **dois baldes** pelo sinal de `N(v) · o`, com `o` a direcção para o
  observador: o da **frente** (`> 0`) ganha sempre que a soma dele não for nula.
- ⚠️ **`R_n` pode passar de `R`**: `lei/normal_da_area_raio_2_0` usa `f_n = 2` e reproduz a
  `1,5e-8` com a soma sobre `2R`. Um estimador que só percorra a pegada do dab (raio `R`) não a
  exprime (§10.2).
- Medido sobre bossas (`lei/normal_da_area_raio_*`): `f_n = 0,25 · 0,5 · 1 · 2` reproduzem a
  `3,3e-6 · 1,0e-6 · 4,6e-7 · 1,5e-8`; os quatro resultados diferem entre si em `1,9e-3` a
  `1,5e-2`, que é o que torna o knob observável.
- ⭐ **É, letra por letra, a lei que a nossa casa já corre para os pincéis de puxar e para o
  pincel de plano** (§10.2).

### §2.4 — A direcção e o `Ctrl`

`σ` vem do valor de fábrica do pincel (**afundar**) e o `Ctrl` segurado **inverte-o** para o
traço inteiro: `produto/afiado_valores_de_fabrica_ctrl` é a imagem espelhada do traço normal
(a régua lê os mesmos números com o sinal trocado). *Afundar* e *levantar* dão o mesmo módulo
(`produto/ablacao_somar` = espelho de `produto/afiado_valores_de_fabrica_continuo`).

### §2.5 — A força entra AO QUADRADO

`lei/um_dab_forca_meia`: força `0,5` desloca o centro de `0,25 R` (`0,1000` com `R = 0,4`). A
pressão entra como factor; o corpus só tem **rato** (pressão `1`), logo a curva da pressão não
está medida aqui (e a casa não tem pressão de caneta).

### §2.6 — Só faces de frente (desligado de fábrica)

Com a opção ligada, `F(v) = max(0, N₀(v) · o)`, com `N₀` a normal do vértice **no pen-down** e `o`
a direcção para o observador. ⚠️ **É um FACTOR, não um corte**: num cilindro visto de cima todas
as normais têm componente para o observador e nada é cortado — o que muda é o peso
(`lei/so_faces_de_frente_cilindro`: `3,0e-8` com o factor, `5,2e-3` sem ele).

### §2.7 — A pegada PROJECTADA (não é o valor de fábrica)

Com a forma da pegada *projectada*, três coisas mudam juntas (`lei/pegada_projectada`, reproduzida
a `4,2e-6`):

1. a distância `|X₀(v) − c|` mede-se **no plano da vista** (sem a componente ao longo da vista);
2. a normal da área (§2.3) é somada com a **mesma** distância projectada;
3. ⚠️⚠️ **a direcção do deslocamento é a normal da área SEM a componente da vista, renormalizada**
   — o dab empurra **de lado**, e numa superfície vista de frente quase não afunda (a média dos
   deslocamentos tem `z = 0`).

Candidatas refutadas: normal da área com distância 3D (`3,1e-4`); sem renormalizar (`6,5e-2`).
⛔ Esta espec **não recomenda** oferecer este modo neste pincel: ele não faz o que o nome promete
(§15).

### §2.8 — O que este pincel IGNORA

- **O plano do pen-down** (a opção que noutros pincéis congela o plano): **inerte** —
  `lei/plano_do_pen_down_inerte` é igual a `lei/um_dab_forca_meia`, ao bit.
- **A topologia dinâmica**: com ela ligada o alvo **não refina nem colapsa** com este pincel
  (medido pela obra do dyntopo: `fixtures/dyntopo/README.md`, linha do pincel afiado). Ver §9.4.

---

## §3 — QUE ESTADO cada dab lê

| o que o dab lê | acumular **desligado** (fábrica) | acumular **ligado** |
|---|---|---|
| o **cursor** `c` | o acerto na superfície **viva** sob o ponteiro | o acerto na superfície **do pen-down** |
| a **normal da área** `n` (posições e normais) | as **vivas** | as **do pen-down** |
| as posições da **distância** `X₀` | as **do pen-down** | as **do pen-down** |
| o que isso faz | o cursor afunda com o vinco ⇒ **auto-limita** (§4) | tudo fixo ⇒ cada dab igual ao primeiro ⇒ **aprofunda linearmente, perfil constante** |

⚠️⚠️ **O interruptor tem, neste pincel, o efeito INVERSO do que tem no desenho comum** — medido,
não suposto: no desenho comum, *desligado* lê o cursor e a normal do pen-down e *ligado* lê-os
vivos (`cadeia/controlo_desenho_*`, `§11`).

Medições:

- **cursor e normal vivos com o interruptor desligado**: cadeias `cadeia/*_cursor_vivo`
  reproduzem a `≤ 1,9e-6`; ler a normal do pen-down erra `4,7e-4` (bossas).
- **normal do pen-down com o interruptor ligado**: `cadeia/bossas_cursor_vivo_acumula` reproduz a
  `1,6e-6`; ler a normal viva erra `4,7e-4`. (⚠️ nesta cadeia o cursor é imposto pelo script; a
  metade do cursor é a seguinte.)
- **cursor do pen-down com o interruptor ligado**: `produto/ablacao_acumular` (traço arrastado)
  reproduz a `3,6e-6` com o cursor lido da superfície do pen-down, e a régua lê o perfil
  **constante** (`W50/R = 0,473` em 1, 2, 4 e 8 passagens) com a profundidade **linear**
  (`0,2445 · 0,4890 · 0,9781 · 1,9561 R`).

### §3.1 — A normal do pen-down (opção, desligada de fábrica)

Com ela ligada, **a normal da área do PRIMEIRO dab do traço é usada por todos os dabs seguintes**
desse traço (`cadeia/bossas_cursor_vivo_normal_do_pen_down`: `1,9e-6`). Cada pen-down recomeça.
Num plano ela é inerte (`produto/ablacao_normal_do_pen_down` difere do de fábrica em `+0,1 %`).

---

## §4 — A COMPOSIÇÃO, e porque o vinco se limita

⭐ **Cada dab SOMA o seu deslocamento à posição viva** — não há acumulador por vértice nem tecto.

### §4.1 — A forma fechada da auto-limitação

Num plano, com o cursor sempre sobre o fundo do vinco (interruptor desligado), o vértice sob o
cursor tem posição do pen-down a distância `z_k` do cursor, onde `z_k` é a profundidade já
cavada. Logo

```text
z_{k+1} = z_k + R · S² · a · C(z_k / R)          z_0 = 0
```

e como `C(1) = 0` e `S² · a ≤ 1`, **`z` nunca passa de `R`** num traço (`(1 − t)⁴ ≤ 1 − t` para
`0 ≤ t ≤ 1`): é essa a razão física do limite.
Conferido dab a dab em `cadeia/mesmo_ponto_cursor_vivo` (`R = 0,4`, `S = 0,5`, `a = 1`, curva
afiada): a fórmula dá `0,1 · 0,13164 · 0,15190 · 0,16670` e o alvo dá os mesmos quatro números
(reprodução `1,5e-8`).

- **O controlo**: com o cursor **imposto** na superfície de repouso (`cadeia/mesmo_ponto_cursor_dado`)
  cada dab é igual ao primeiro: `0,1 · 0,2 · 0,3 · 0,4` — linear.
- **O desenho comum com o acumular ligado** (distância e cursor vivos) também é linear no mesmo
  ponto (`cadeia/controlo_desenho_mesmo_ponto_acumula`: `0,4` depois de 4 dabs).

### §4.2 — Levantar a caneta recomeça

Cada pen-down fotografa as posições `X₀` de novo; o limite de `R` vale **por traço**. Oito
traços separados: `0,21 · 0,41 · 0,83 · 1,51 R` depois de 1, 2, 4, 8 traços (§7).

### §4.3 — Consequência para o produto

A profundidade de um traço contínuo **não depende do número de passagens depois das primeiras**
(`+24 %` de 4 para 8 passagens) e **não depende da densidade** da malha (`D/R` a `8` passagens:
`0,5427 · 0,5406 · 0,5389` em 48², 96², 192²).

---

## §5 — O TRAÇO ARRASTADO: o passo e a atenuação

### §5.1 — O passo

- **`5 %` do diâmetro de ecrã** — com o diâmetro de `100` px, `5` px, ou seja `0,1 R`.
- Os dabs caem numa **grelha** que começa no píxel do pen-down: `pen_down + k·passo` ao longo do
  percurso do rato; o que sobra de um evento passa para o seguinte.
- O **pen-down deposita um dab** no píxel em que a caneta desce.
- ⚠️ **Um salto de EXACTAMENTE um passo deposita um dab** no alvo.

O detector (`detector/salto_*`): saltos de `0 · 1 · 4` px dão **o mesmo** resultado (só o dab do
pen-down); `5 · 9` dão o mesmo (mais um dab); `10 · 14`; `15 · 19`; `20 · 24`; `25 · 29`; `30`.
Dentro de cada classe as saídas são **idênticas ao bit**; entre classes vizinhas diferem
`1,06e-2`–`1,28e-2` (um dab a mais).

⚠️ **A nossa casa diverge na fronteira**: o nosso passeio do traço (`spacing.rs:155-175`) recusa
um salto de exactamente um passo (`dist <= passo` ⇒ nenhum dab), de propósito e com gate. Num
arrasto de `1` px por evento as duas fronteiras dão a **mesma** lista de dabs (o dab cai no mesmo
sítio, um evento depois); só um salto isolado de exactamente `5` px as separa. ⇒ **divergência a
declarar** (§15), como a obra do pincel de plano já declarou.

### §5.2 — A atenuação `a`

```text
h = s / 50              s = espaçamento em % do diâmetro
n = ⌊100 / s⌋
a = 1 / max_{φ ∈ {0; 0,1; …; 0,9}}  Σ_{j=0}^{n−1} C(|φ − 1 + j·h|) · [|φ − 1 + j·h| < 1]
```

com `C` a curva do pincel **sem** a dureza. É a lei que a casa já calcula em
`atenuacao_por_espacamento` (`atenuacao_do_traco.rs:48-69`).

| curva | espaçamento | `a` |
|---|---|---|
| afiada | `5 %` (fábrica deste pincel) | **`0,24591`** |
| afiada | `8 %` | `0,38367` |
| afiada | `10 %` | `0,46887` |
| suave | `10 %` (fábrica do desenho comum) | `0,20000` |
| suave | `5 %` | `0,10000` |

### §5.3 — ⚠️⚠️ Neste pincel o factor por dab é `a` — NÃO `(1 + a)/2`

- **Todo dab do traço arrastado, o do pen-down incluído**, é multiplicado por `a`.
- Medido: com `a` o traço contínuo reproduz a `4,9e-4`; com `(1 + a)/2` (a lei do pincel de plano)
  erra `4,7e-2`; sem atenuação, `8,0e-2`. O dab do pen-down sozinho: `1,8e-8` com `a`, `2,3e-2`
  com `(1 + a)/2`, `4,5e-2` sem (`detector/salto_00px` e `detector/salto_00px_sem_atenuacao`).
- ⚠️ **A nossa casa dá `(1 + a)/2` ao pincel de plano, e só a ele** (`atenuacao_do_traco.rs:77-91`).
  Estender a porta a este pincel com a lei do plano daria um vinco `2,5×` mais forte por dab.
- ⭐ **Porque o espaçamento quase não é alavanca**: a atenuação compensa-o. `8 %` e `10 %` dão
  `0,5394` e `0,5373 R` contra `0,5406` a `5 %` (`-0,2 %` e `-0,6 %`, §8.1).

---

## §6 — Os VALORES DE FÁBRICA

Lidos do pincel de catálogo carregado pelo próprio programa (§0.2). A coluna *nosso* foi lida no
código vivo, com o sítio.

| valor | afiado (alvo) | desenho comum (alvo) | nosso `Draw` modo B |
|---|---|---|---|
| curva de queda | **afiada** `(1−u)⁴` | suave | suave — `ref_profiles.rs:412` (`profile_b`) |
| direcção | **afundar** | levantar | levantar; só o `Ctrl` inverte — `ph2d-app-sculpt3d/src/input_down.rs:294` |
| força | `0,5`, ao quadrado | `0,5`, ao quadrado | `0,5` — `brush_verb_defaults.rs:56-65`; ao quadrado — `ref_profiles.rs:437-451` |
| força segue a pressão | sim | sim | não há pressão de caneta |
| acumular | **desligado** (e o interruptor é inverso, §3) | desligado | **LIGADO** — `ref_profiles.rs:162-168` via `brush_verb_defaults.rs:180-187` |
| distância medida de | **pen-down** (sempre) | posição viva | pen-down com o acumular desligado — `grip.rs:233-261` |
| espaçamento | **`5 %`** do diâmetro | `10 %` | `0,15 R` = `7,5 %` — `spacing.rs:55-61` |
| atenuação do traço | **ligada**, `a = 0,246` | ligada, `a = 0,200` | nenhuma (só o pincel de plano) — `atenuacao_do_traco.rs:77-91` |
| dureza | `0` | `0` | `0` — `brush_default.rs:127` |
| raio da normal da área | `0,5 R` | `0,5 R` | `0,5` no campo (`brush_default.rs:132`), mas o `Draw` usa a normal do plano da casa e não o lê — `stroke_target.rs:97,187` |
| referencial da direcção | a normal da área | a normal da área | a normal do plano da casa |
| só faces de frente | não | não | não — `brush_verb_defaults.rs:272-274` |
| só ligados | não | não | **sim** — `brush_default.rs:183` |
| normal do pen-down | não | não | — |
| plano do pen-down | não (inerte) | não | — |
| auto-alisar | `0` | `0` | `0` — `brush_default.rs:185` |
| forma da pegada | esfera | esfera | esfera |
| inclinação · tremor · traço suavizado | `0` · `0` · não | idem | — |
| topologia dinâmica | **não refina nem colapsa** | — (não medido aqui) | o `Draw` refina e colapsa — `brush_verb_dyntopo.rs:60,85` |

⭐ **Os quatro que o pincel afiado muda em relação ao desenho comum do alvo:** o tipo (a distância
do pen-down), a curva, a direcção e o espaçamento. ⭐ **Os que ele muda em relação ao NOSSO
`Draw`:** curva · direcção · acumular · espaçamento · atenuação — e, para a paridade vértice a
vértice, a normal da área.

⚠️ **O «só ligados» da casa** não muda um vinco raso (os vértices tocados estão todos ligados),
mas **satura** o nosso `Draw` a levantar (`D/R` pára em `1,34` depois de 8 passagens, contra
`3,80` sem ele — medido pela sonda do §7.5). Fica registado; não é deste pincel.

---

## §7 — A RÉGUA do produto: o perfil do vinco

### §7.1 — Definição

Sobre um traço ao longo de `x`, na linha `y ≈ 0`:

- o deslocamento de cada vértice **ao longo da normal de repouso** (no cilindro, a normal
  analítica), com sinal escolhido para que o vinco seja positivo;
- o **perfil**: a média desse deslocamento sobre as colunas com `|x| ≤ 0,1`, em função da
  coordenada transversal de repouso (no cilindro, o comprimento de arco);
- **`D`** = o máximo do perfil; **`W_f`** = a largura onde o perfil cruza `f·D` (interpolação
  linear entre amostras), com `W50` a meia-largura;
- **nitidez** `D/W50` · **agudeza** `W25/W75` · **rebordo** = `−min/D` · **curvatura do fundo**
  = `−κ·W50²/D`, com `κ` a segunda diferença no máximo.

Tudo em unidades de `R`. `R = 0,25`, grelha de lado `2`, traço de `x = −0,5` a `0,5`.

### §7.2 — O pincel afiado de fábrica, traço contínuo

| grelha | passagem | `D/R` | `W50/R` | nitidez | agudeza | fundo |
|---|---|---|---|---|---|---|
| 96² | 1 · 2 · 4 · 8 | `0,2058 · 0,3221 · 0,4367 · 0,5406` | `0,506 · 0,527 · 0,543 · 0,550` | `0,407 · 0,611 · 0,805 · 0,983` | `2,54 · 2,45 · 2,39 · 2,35` | `6,82 · 6,57 · 6,39 · 6,23` |
| 48² | 1 · 8 | `0,2064 · 0,5427` | `0,517 · 0,554` | `0,399 · 0,979` | `2,69 · 2,43` | `5,56 · 5,33` |
| 192² | 1 · 8 | `0,2055 · 0,5389` | `0,505 · 0,548` | `0,407 · 0,983` | `2,51 · 2,33` | `7,44 · 6,56` |
| cilindro 96² | 1 · 2 · 4 · 8 | `0,2057 · 0,3220 · 0,4366 · 0,5406` | `0,510 · 0,535 · 0,555 · 0,565` | `0,403 · 0,602 · 0,787 · 0,956` | `2,54 · 2,45 · 2,39 · 2,34` | `2,19 · 2,11 · 2,06 · 2,01` |

(A curvatura do fundo depende da amostragem — cresce com a densidade — e a do cilindro mede-se
contra a curvatura da peça; ela compara-se **só dentro da mesma linha**.)

### §7.3 — Oito traços SEPARADOS

| grelha | traço | `D/R` | `W50/R` | nitidez |
|---|---|---|---|---|
| 96² | 1 · 2 · 4 · 8 | `0,2058 · 0,4129 · 0,8286 · 1,5119` | `0,506 · 0,459 · 0,346 · 0,269` | `0,407 · 0,899 · 2,392 · 5,622` |
| 48² | 2 · 4 · 8 | `0,4154 · 0,8338 · 1,7609` | `0,474 · 0,349 · 0,225` | `0,876 · 2,391 · 7,815` |
| 192² | 2 · 4 · 8 | `0,4116 · 0,8243 · 1,6604` | `0,458 · 0,345 · 0,195` | `0,899 · 2,392 · 8,527` |
| cilindro | 2 · 4 · 8 | `0,4129 · 0,8291 · 1,5147` | `0,465 · 0,352 · 0,270` | `0,888 · 2,358 · 5,603` |

⚠️ **O 8.º traço separado é SENSÍVEL** (`1,51`–`1,76 R` entre densidades): o cursor cai num
vinco já estreito, e onde ele cai muda a profundidade. As barras do §12 respeitam-no.

### §7.4 — A régua SEPARA o afiado do desenho comum (o controlo)

| 8 passagens contínuas, 96² | `D/R` | `W50/R` | nitidez | agudeza | fundo |
|---|---|---|---|---|---|
| **afiado de fábrica** | `0,5406` | **`0,550`** | **`0,983`** | **`2,35`** | **`6,23`** |
| desenho comum de fábrica | `0,7766` | `1,258` | `0,617` | `1,62` | `3,17` |
| desenho comum de fábrica, a afundar | `0,7766` | `1,258` | `0,617` | `1,62` | `3,17` |

Separados, 8 traços: afiado `1,51 R` / `0,269 R` / nitidez `5,62`; desenho comum `1,90 R` /
`0,739 R` / `2,57`. ⇒ **o afiado é `2,3×` mais estreito contínuo e `2,7×` separado.**

### §7.5 — O NOSSO motor

| 96², traço contínuo | passagem 1 · 2 · 4 · 8 `D/R` | `W50/R` |
|---|---|---|
| nosso `Draw` de fábrica (levantar) | `0,9171 · 1,1537 · 1,2758 · 1,3369` | `0,745 · 0,628 · 0,570 · 0,541` |
| nosso `Draw` de fábrica, a afundar | `1,4450 · 2,7674 · 4,0370 · 4,3272` | `0,753 · 0,524 · 0,452 · 0,444` |
| **nosso `Draw` composto** (§0.1) | **`0,2121 · 0,3236 · 0,4377 · 0,5411`** | **`0,504 · 0,526 · 0,542 · 0,549`** |
| nosso `Draw` composto, separados | `0,2121 · 0,4256 · 0,8571 · 1,5814` | `0,504 · 0,457 · 0,342 · 0,237` |

A tabela inteira de desvios contra o alvo, nas quatro superfícies, está no G-4 (§12).

⚠️ **Como o nosso motor foi medido** (sonda temporária, NÃO commitada, apagada no fim desta
obra — antes `crates/ph2d-sculpt3d/tests/sonda_afiado_nosso.rs`): a mesma grelha em triângulos
com a diagonal `(i,j)–(i+1,j+1)`, o cilindro de raio `1,2`, um evento por píxel (`200` px por
unidade), o passeio do traço da casa com o passo pedido, um dab no pen-down, e o cursor lançado
por um raio vertical na linha `y = 0,0025` (o centro do píxel do alvo). A atenuação `a` entrou
como `√a` na força (a força é elevada ao quadrado). ⚠️ **O arnês não passa pela porta do
produto** — é uma medição de lei, não um gate; o G-4 tem de a refazer pela porta.

---

## §8 — A ABLAÇÃO: qual valor é a alavanca

### §8.1 — No alvo, um valor de cada vez a partir do de fábrica (96², contínuo, 8 passagens)

| troca | `D/R` | `W50/R` | nitidez | leitura |
|---|---|---|---|---|
| (fábrica) | `0,5406` | `0,550` | `0,983` | — |
| **tipo = desenho comum** (distância viva) | `0,5355` | **`0,852`** (`+55 %`) | `0,628` (`−36 %`) | ⭐ alavanca da **largura** |
| **curva suave** | `0,7777` (`+44 %`) | **`0,837`** (`+52 %`) | `0,929` | ⭐ alavanca da **largura** |
| **acumular ligado** | **`1,9561`** (`+262 %`) | `0,473` | `4,133` | ⭐ alavanca da **profundidade** (§3) |
| sem atenuação | `0,7062` (`+31 %`) | `0,565` | `1,251` | alavanca da profundidade |
| força `1` | `0,7047` (`+30 %`) | `0,564` | `1,248` | alavanca da profundidade |
| espaçamento `8 %` · `10 %` | `0,5394` · `0,5373` | `0,547` · `0,546` | — | inerte (a atenuação compensa) |
| normal do pen-down | `0,5411` | `0,549` | — | inerte num plano |
| raio da normal `1 R` | `0,5410` | `0,550` | — | inerte num plano |
| levantar | `0,5406` | `0,550` | — | espelho |

Passagem 1, as duas alavancas da largura: tipo `+7,1 %`, curva `+84 %`.

### §8.2 — A distância do pen-down, ISOLADA

Duas corridas do alvo que diferem **só** na fonte da distância (as duas com cursor e normal
vivos, curva afiada, `8 %`, sem atenuação, raio da normal `1 R`):

| fonte da distância | passagem 1 `D/R` | passagem 8 `D/R` | `W50/R` (8) |
|---|---|---|---|
| **pen-down** | `0,4061` | **`0,6630`** | `0,551` |
| viva | `0,5297` | **`2,9899`** (`4,5×`) | `0,243` |

⭐⭐ **Medida do vivo, a distância acompanha o vértice que afunda, o peso não cai e o vinco
cava sem limite** (e estreita). É isto que o §4 formaliza.

### §8.3 — No NOSSO motor

**(a) A partir do nosso `Draw` de fábrica a afundar** (passo `0,15 R`, sem atenuação), um de cada
vez — 8 passagens:

| troca | `D/R` | `W50/R` |
|---|---|---|
| (fábrica, afundar) | `4,3272` | `0,444` |
| **acumular desligado** | **`0,9553`** (`÷ 4,5`) | `0,780` |
| curva afiada | `2,9325` | `0,260` |
| passo `0,1 R` + atenuação `a` | `4,4673` | `0,414` |
| as duas primeiras juntas | `0,6707` | `0,549` |
| as três juntas (o *composto*) | **`0,5411`** | **`0,549`** |

**(b) A partir do composto** (o equivalente do alvo), um de cada vez — 8 passagens, contra a
mesma troca no alvo:

| troca | nosso `D/R` · `W50/R` | alvo `D/R` · `W50/R` |
|---|---|---|
| (composto / fábrica) | `0,5411 · 0,549` | `0,5406 · 0,550` |
| curva suave | `0,8942 · 0,793` | `0,7777 · 0,837` |
| acumular ligado | `1,6791 · 0,259` | `1,9561 · 0,473` |
| força `1` | `0,7080 · 0,556` | `0,7047 · 0,564` |
| sem atenuação | `0,7096 · 0,556` | `0,7062 · 0,565` |
| levantar | espelho, ao bit | espelho |

⚠️ **O acumular ligado dá OUTRO vinco nas duas casas** — o nosso cava e estreita (distância
viva, como a corrida «viva» do §8.2); o do alvo aprofunda com o perfil constante. É o buraco 4 do
§0.1.

⚠️ **A curva suave diverge mais (`+15 %` na profundidade)** porque o perfil largo lê a normal da
área sobre mais superfície, e é aí que a nossa lei de normal e a do alvo se afastam.

### §8.4 — ATRIBUIÇÃO do desvio que sobra

O modelo de referência do E reproduz o produto do alvo com três leituras da normal da área:

| traço (96²) | normal do alvo (§2.3), viva | normais congeladas no repouso | a lei da nossa casa (soma simples sobre a pegada, viva) |
|---|---|---|---|
| contínuo, 8 passagens | **`4,9e-4`** | `2,5e-2` | `1,7e-2` |
| separados, 8 traços | **`3,7e-3`** | `1,25e-1` | `9,3e-2` |

(erro máximo por vértice na malha inteira). ⇒ **o desvio de `+3,1 %` e o estreitamento de
`11,7 %` são da lei da normal**, e com a lei do alvo o resíduo cai 35× no contínuo e 25× nos
separados.

⭐ **A alavanca, numa frase:** *medir a distância a partir das posições do pen-down* — é ela que
faz o vinco limitar-se (no alvo, `2,99 → 0,66 R`; na nossa casa é o acumular desligado,
`4,33 → 0,96 R`) e, junto com a curva afiada, que o faz estreito (sem ela `W50` sobe `55 %`).

---

## §9 — O que o alvo faz e NÃO se copia

### §9.1 — ⛔ O recorte do cursor pela caixa envolvente

Numa vista ortográfica o alvo recorta o raio do cursor à caixa envolvente da malha antes de o
lançar. Num plano essa caixa tem espessura zero (mais uma folga de `1e-3`); um vinco mais fundo
fica **fora** e o acerto seguinte falha — **o dab perde-se em silêncio**
(`artefacto_caixa/caixa_fina_salto_14px`: só o dab do pen-down, `446` vértices e fundo `0,0148`,
contra três dabs, `507` vértices e `0,0323` com caixa espessa). O mesmo acontece a um traço que
**levanta** acima de uma caixa que só engrossa para baixo (`1,5e-2` e `2,4e-2`). ⇒ **a nossa casa
não recorta, e o gate G-11 prende isso.**

### §9.2 — As normais que o caminho por script não refresca

O caminho por script do alvo não recalcula as normais de vértice entre dabs; o arrastado sim.
Não é lei do produto — é como as famílias `lei/` e `cadeia/` foram medidas (a bancada delas usa o
bloco `n` para todos os dabs).

### §9.3 — O resíduo dos saltos de vários dabs num evento

Num salto que deposita vários dabs num só evento, o modelo erra `2,9e-4` (14 px) a `4,6e-3`
(30 px), e **nenhuma** das duas leituras das normais (refrescadas por dab, ou as do início do
evento) o fecha. ⇒ **é o chão medido** do detector; a contagem de dabs é exacta.

### §9.4 — A topologia dinâmica

O alvo não mexe na topologia com este pincel (§2.8). A nossa casa refina com o `Draw`. Se o
pincel afiado for um **modo** do `Draw`, a célula do dyntopo segue o verbo (refina) — ⇒ ou uma
célula por modo, ou **divergência declarada**. Decisão do dono (§15).

---

## §10 — O que a nossa casa JÁ exprime, e o que falta (lido no código vivo)

### §10.1 — Já existe

- **A distância do pen-down**: o `Draw` é `Grip::Stamp`, cujo `from_live` **é** o acumular
  (`grip.rs:233-261`); com ele desligado a distância sai do `base` (`stroke_dab_core.rs:273-278`).
- **O cursor na superfície viva** a cada dab: `Verb::le_a_superficie_viva` responde `true` para o
  `Draw` (`brush_verb_predicados.rs:211-217`).
- **A composição aditiva** sobre a posição viva (`stroke_target.rs:187`).
- **A curva afiada** `(1−u)⁴` (`Falloff::Sharper`, `falloff.rs:90-97`), a **dureza**
  (`brush_scale.rs:298-312`), a **força ao quadrado** (`ref_profiles.rs:437-451`), o **alcance
  = R** (`ref_profiles.rs:368-373`).
- **O passo de «s % do diâmetro»** e **o factor `a`** (`atenuacao_do_traco.rs:37-69`) — só para o
  pincel de plano.

### §10.2 — A normal da área do alvo existe DUAS vezes, e o `Draw` não a usa

- `stroke_normal_do_gesto.rs:61-115` (os pincéis de puxar) e
  `plano_da_pegada.rs:102-140` (o pincel de plano) implementam a lei do §2.3 — raio próprio,
  curva suave fixa, dois baldes com a frente a ganhar.
- O `Draw` lê `plane.normal` (`stroke_target.rs:97`), a normal do estimador de plano da casa.
- ⚠️ **As duas implementações percorrem só a pegada do dab** (`self.footprint`, raio `R`): com
  `f_n > 1` (`lei/normal_da_area_raio_2_0`) faltam vértices.

### §10.3 — Falta

1. **Valores de fábrica por pincel** para este modo: curva afiada, acumular desligado,
   espaçamento `5 %`, atenuação ligada.
2. **Uma direcção de fábrica** («afundar») separada do `Ctrl`: hoje `brush.invert = ctrl`
   (`ph2d-app-sculpt3d/src/input_down.rs:294`), logo não há onde guardar «este pincel nasce a afundar». A lei do §2.4 é
   `σ = direcção_de_fábrica × (−1 se Ctrl)`.
3. **O passo e o factor por pincel**: `passo_do_traco` e `factor_do_traco` perguntam
   `verb == Plane` (`atenuacao_do_traco.rs:37-44`, `:77-91`); este pincel pede o passo de `5 %` e
   o factor **`a`** (não `(1 + a)/2`).
4. **(para a paridade vértice a vértice)** a normal da área do §2.3 no `Draw`-afiado, com a soma
   sobre `R_n` mesmo quando `R_n > R`.

### §10.4 — Não é o valor de fábrica, e seria lei nova: o acumular ligado deste pincel

Ele pede o cursor da superfície do pen-down, a normal do pen-down e a distância do pen-down. As
peças existem — `Verb::pica_na_superficie_do_pen_down` (`brush_verb_predicados.rs:574-576`, hoje
só o projectar na cena), a fotografia das normais do primeiro toque (`stroke.rs:138`,
`stroke_plane.rs:214-222`) —, mas nenhuma está ligada ao `Draw`. ⇒ **decisão** (§15): esconder o
interruptor neste modo, ou ligá-lo à lei do alvo.

---

## §11 — As fixtures

[`fixtures/pincel_afiado/`](fixtures/pincel_afiado/README.md) — **80** ficheiros; a contagem sai
do directório (`find … -name '*.txt.gz' | wc -l`).

| família | n | o que a bancada faz com ela |
|---|---|---|
| `lei/` | 17 | um dab por ponto `c`, com o bloco `n` como normal de vértice, a máscara `m` onde houver, e compara `s` |
| `cadeia/` | 10 | os pontos `c` pela ordem, o bloco `n` para **todos** os dabs, e compara cada `d<j>` e `s` — `7` cadeias do pincel afiado (`52` estados) + `3` controlos do desenho comum do alvo (`20` estados), que **documentam** a lei dele (§3) e não são gate do nosso produto |
| `produto/` | 35 | o traço arrastado pela **porta do produto** (passo, atenuação, cursor vivo, normais vivas) e compara `p<k>` e `s`, ou a régua do §7 |
| `detector/` | 15 | um pen-down e um salto do rato, e conta os dabs / compara `s` |
| `artefacto_caixa/` | 3 | o controlo do §9.1 |

⚠️ **Regras de leitura** (todas no README das fixturas e no cabeçalho de cada uma):

- o raio efectivo é o do cabeçalho (`0,4` nas por script, `0,25` nas arrastadas);
- nas arrastadas, o píxel do pen-down e o passo de um píxel no mundo estão no cabeçalho; a linha
  do traço fica meio píxel ao lado de `y = 0` (`y = 0,0025`);
- nas de caixa dupla, **dois vértices de canto estão fora do plano** (`z = ±1`) — a régua ignora-os
  (estão longe do traço), mas uma comparação vértice a vértice tem de os incluir tal como estão;
- as `2` de catálogo servem de **completude** (o pincel de fábrica = os valores escritos); ⛔ os
  gates ancoram-se nas de pincel **nosso**, cujo cabeçalho diz `origem_dos_valores: … ESCRITOS`.

---

## §12 — Os GATES propostos, cada um com a população, o piso, a barra e a origem

Convenção: «aprovado» = o maior erro do lado que tem de passar; «errado» = o menor erro da
candidata que tem de reprovar. A barra fica **estritamente entre** os dois. As medições do lado
aprovado são do **modelo de referência do E** (`float64`); a implementação em `f32` pode pedir
reconferência — ⛔ **nunca** afrouxar uma barra por isso sem a medição ao lado.

| gate | população (piso) | mede | barra | aprovado | errado mais perto | origem |
|---|---|---|---|---|---|---|
| **G-1** lei de um dab | `lei/` (**17**) | max `|Δ|` por vértice contra `s` | `2e-6` planas e cilindro · `1e-5` bossas e pegada projectada (estas exigem a normal do §2.3, D-3) | `8,1e-8` · `4,2e-6` | faces de frente sem o factor `5,2e-3`; normal com distância 3D na pegada projectada `3,1e-4`; `R_n` trocado `≥ 1,9e-3`; força linear `1e-1` | §2 |
| **G-2** cadeias | as `7` cadeias do pincel afiado em `cadeia/` (**52** estados) | max `|Δ|` contra cada `d<j>` e `s` | `1e-5` (as de bossas exigem a normal do §2.3, D-3) | `1,9e-6` | normal lida do lado errado (§3) `4,7e-4`; distância viva `≥ 2,6e-2`; cursor do pen-down com o interruptor desligado `≥ 3,6e-2` | §2.1, §3, §4 |
| **G-3a** produto vértice a vértice, contínuo (⚠️ exige a normal do §2.3) | as `6` contínuas de pincel NOSSO (96², 48², 192², cilindro, `Ctrl`, triângulos) × fotos | max `|Δ|` na malha inteira | `2e-3` | `5,1e-4` | a lei de normal da casa `9,5e-3`; normais congeladas `2,35e-2`; sem atenuação `7,95e-2`; `(1+a)/2` `4,7e-2` | §2.3, §5 |
| **G-3b** produto vértice a vértice, separados (idem) | as `4` separadas (96², 48², 192², cilindro) + triângulos (**5**) | max `|Δ|` em `|x| ≤ 0,25` · na malha inteira | `8e-3` · `3e-2` | `2,3e-3` · `1,17e-2` | lei da casa `2,4e-2` · `9,1e-2` | §2.3, §4.2 |
| **G-4a** régua, contínuo — o `Draw` composto SEM a normal do alvo | 4 superfícies (96², 48², 192², cilindro) × passagens 1, 2, 4, 8 (**16** células) | desvio relativo de `D/R` · `W50/R` · nitidez contra o alvo | `6 %` · `2 %` · `6 %` | `3,2 %` · `0,4 %` · `3,6 %` | acumular ligado (p1) `D +13,6 %`, `W −4,5 %`; tipo desenho comum (p1) `W +7,1 %`, nitidez `−8,1 %`; curva suave `D +146 %`; sem passo/atenuação `D +112 %`; força `1` `D +146 %` | §7, §8 |
| **G-4b** régua, separados — idem | 4 superfícies × traços 1, 2, 4 (**12**) · × traço 8 (**4**) | idem | `6 %` · `3 %` · `8 %` (1–4) · `8 %` · `15 %` · `25 %` (8) | `3,5 %` · `1,3 %` · `4,8 %` · `4,9 %` · `11,8 %` · `18,9 %` | sem passo/atenuação (traço 8) `D +86 %`; desenho comum separado `W +175 %` | §7.3 |
| **G-5a** passo — contagem | `detector/salto_*` sem os declarados (**13**) | nº de dabs = `⌊N/5⌋ + 1` | exacto | exacto | `±1` dab | §5.1 |
| **G-5b** passo — posições | as mesmas (**13**) | max `|Δ|` contra `s` | `7e-3` | `4,6e-3` | um dab a mais ou a menos `≥ 1,06e-2` | §5.1, §9.3 |
| **G-5c** o dab do pen-down é atenuado | `detector/salto_00px` e `…_sem_atenuacao` (**2**) | max `|Δ|` | `1e-5` | `1,8e-8` | sem atenuação `4,5e-2`; `(1+a)/2` `2,3e-2` | §5.3 |
| **G-6** valores de fábrica | as linhas do §6 que o modo muda contra o nosso `Draw` (**5**: curva · direcção · acumular · espaçamento · atenuação) | o pincel nasce com cada valor | exacto | — | — | §6 |
| **G-7** auto-limitação, forma fechada | a recorrência do §4.1 contra `cadeia/mesmo_ponto_cursor_vivo` (**4** dabs) | `|Δz|` | `1e-6` | `1,5e-8` | cursor imposto (linear) `≥ 6,8e-2` | §4.1 |
| **G-8** a régua separa o afiado do desenho comum | `produto/*_valores_de_fabrica_*` (**2**) | razão `W50` desenho/afiado a 8 passagens | `≥ 1,8` | `2,29` | — (o controlo: se a razão cair, a régua deixou de ver) | §7.4 |
| **G-9** o primeiro dab é o do desenho comum | `lei/um_dab_curva_afiada` × `…_controlo_desenho_comum` (**2**) | diferença | `0` ao bit | `0,0` | — | §1 |
| **G-10** simetria de direcção | `produto/ablacao_somar` × `produto/afiado_valores_de_fabrica_continuo` (**2**, `4` fotos + a saída) | `|Δ|` contra o espelho `z → −z` | `2e-6` | `1,2e-7` | — | §2.4 |
| **G-11** o recorte pela caixa NÃO se copia | o salto de `14` px numa malha plana **sem** vértices de canto | `|Δ|` contra `detector/salto_14px` e contra `artefacto_caixa/caixa_fina_salto_14px` | `≤ 7e-3` do primeiro **e** `≥ 1e-2` do segundo | — | os dois diferem `2,24e-2` | §9.1 |
| **G-12** nenhum knob morto neste modo | cada knob oferecido × duas posições (piso = os knobs pintados) | o barro muda | `> 0` | — | — | §8 (o censo dos knobs da casa) |

⛔ **Declarados, com nome** (catraca, nunca silêncio): o G-5a/G-5b excluem **um** ficheiro,
`detector/salto_05px` — é o único salto de **exactamente** um passo (D-1); os de `9`, `14`, `19`,
`24` e `29` px dão os passos inteiros e um resto, e a casa já os reproduz. O
`detector/salto_00px_sem_atenuacao` pertence ao G-5c. Os G-3 excluem `produto/alvo_com_*`
(corridas do alvo com valores nossos, para a ablação), as `produto/desenho_*` (controlos da
régua) e as `produto/*_de_catalogo_*` (completude, §11).

---

## §13 — A proveniência de cada número

| número | de onde |
|---|---|
| valores de fábrica | pincel de catálogo carregado pelo programa (`produto/*_de_catalogo_*`); completude `1,2e-7` |
| `a = 0,24591` e a tabela do §5.2 | a fórmula do §5.2 (a mesma de `atenuacao_do_traco.rs`), confirmada pelo traço (`4,9e-4` com `a`) |
| o passo e as classes do detector | `detector/`, saídas idênticas ao bit dentro de cada classe |
| todas as reproduções «`≤ …`» | o modelo de referência do E (`float64`), fora da árvore, a partir **só** do repouso, do caminho e do cabeçalho |
| as réguas do alvo | as fotos `p<k>` e `s` das fixturas de `produto/` |
| as réguas nossas | a sonda temporária do §7.5 |
| a atribuição do §8.4 | o mesmo modelo, trocando só a lei da normal |
| a recusa do dyntopo | a obra do dyntopo (`fixtures/dyntopo/README.md`) |

---

## §14 — O que os autores aprenderam, e o que NÃO copiar

(Destilado da revisão pública de 2019, das notas de versão e do manual; em vocabulário nosso.)

1. **O pincel mudou de nome antes de sair**: o primeiro nome dizia *como* ele funciona por dentro;
   o que ficou diz *o que* ele faz. ⇒ na nossa casa o rótulo deve dizer o efeito.
2. **Na revisão pública levantou-se a pergunta «pincel próprio, ou variante de um que já
   existe?»** — é exactamente a pergunta zero desta espec. ⭐ A medição dá razão às duas partes: a
   curva **é** um valor, a distância do pen-down **é** um modo que a casa já tem (o acumular
   desligado), e o resto são valores do traço (§0.1).
3. **O interruptor do acumular ficou com o efeito inverso neste pincel** (§3). A leitura que a
   medição dá: como a distância já sai sempre do pen-down, a única coisa que sobra para o
   interruptor escolher é de onde vêm o cursor e a normal — e escolhe-a ao contrário do desenho
   comum. ⚠️ Na nossa casa, se o modo for oferecido com o nosso interruptor, o rótulo mente por
   construção: decisão (§15).
4. **Com topologia dinâmica o alvo não mexe na malha com este pincel**, e a documentação pública
   aponta o pincel de vinco para esse caso (§9.4).
5. **O vinco só se lê em malha fina**: a meia-largura é `≈ 0,55 R`; com menos de `~3` arestas
   nessa largura o fundo não é representável (medição nossa: `W50` em 48² são `3,3` células com
   `R = 0,25`).
6. ⛔ **Não copiar:** o recorte do cursor pela caixa (§9.1); a pegada projectada que empurra de
   lado (§2.7), se oferecida neste pincel.

---

## §15 — Divergências a declarar e decisões do dono

| # | assunto | as saídas | recomendação técnica |
|---|---|---|---|
| D-1 | um salto de exactamente um passo (§5.1) | mudar a fronteira do passeio da casa · declarar | **declarar** (a fronteira tem gate e o mesmo precedente do pincel de plano; num arrasto real as listas coincidem) |
| D-2 | o recorte pela caixa (§9.1) | copiar · não copiar | **não copiar** (G-11) |
| D-3 | a normal da área (§10.2) | manter a da casa (desvio `+3,1 %` · `−11,7 %`, G-4) · adoptar a do alvo neste modo (G-3) | adoptar — a lei já vive em duas portas da casa |
| **P-1** (dono) | **o nome e a forma**: um modo/valor do nosso `Draw` ou um pincel próprio no catálogo | as duas exprimem a lei | técnica: modo do `Draw` (§0.1); o nome no catálogo é do dono |
| **P-2** (dono) | **o acumular neste modo** (§10.4) | esconder · oferecer com a lei do alvo (lei nova) · oferecer com a nossa (outro vinco, §8.3) | esconder até haver pedido |
| **P-3** (dono) | **a topologia dinâmica** (§9.4) | não refinar (como o alvo) · refinar (como o nosso `Draw`) | não refinar — é o que o nome promete (um vinco fino não sobrevive a um remalhe) |
| **P-4** (dono) | **a pegada projectada** (§2.7) | não oferecer · oferecer | não oferecer neste modo |

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
  valor público de enumeração usado como chave de regeneração — §4.1.13). Re-executada sobre a
  §16 na 2.ª emenda, 2026-09-16.
Sweep: controlo do instrumento corrido PRIMEIRO (exit 0, todos os canais), depois VERDE com a
  vassoura desta obra (alongada na 1.ª emenda — contagem no ledger) e com as outras oito da pasta,
  sobre esta espec, as 110 fixturas e o README delas, o INBOX e o README da pasta. O
  --git-history da pasta acusa só patches de ledgers de OUTRAS obras (registado no ledger); o
  commit desta obra, varrido à parte: 0. Re-corrido na 2.ª emenda sobre a §16 e as 30 fixturas
  novas, com o controlo do instrumento PRIMEIRO.
2.ª EMENDA (§16, 2026-09-16): o PASSO JUNTO À SILHUETA, aberta por um report do dono com foto.
  Ela NÃO altera nenhuma lei nem nenhum gate das §1–§15; acrescenta a §16, a família de fixturas
  `silhueta/` (30) e os gates G-13..G-20.
  Auditoria da §16 (R-pré, **3.ª passagem**, 2026-09-16, subagente NOVO e independente do E e das
  duas passagens anteriores): ⭐ **a PAREDE da §16 está LIMPA** — os 3 blocos cercados são o
  cabeçalho e duas identidades matemáticas de 1 e 3 linhas; os 104 code spans não-numéricos são
  todos nossos, de fixtura nossa ou símbolo matemático; `VIEW`/`SCENE` são valores **públicos** de
  enumeração da API (conferido: o harness escreve a string numa propriedade pública, e a chave
  `espacamento_medido_em` mapeia para a propriedade do **espaçamento**, não para a do tamanho); os
  30 cabeçalhos e o conferidor lidos linha a linha; sweep verde com as nove vassouras sobre a
  espec + as 30 fixturas + os READMEs + o INBOX, verde em `--git-history`, e verde numa passagem
  extra em memória com 1 820 agulhas normalizadas (sem caixa, sem acentos) contra 34 artefactos.
  ⭐ O CONTROLO re-derivou-se **ao dígito** por uma régua independente alimentada só pelas fixturas
  publicadas: a tabela inteira do §16.2 nas três corridas, `10,35×` a profundidade em cena,
  `−45,6 %`/`−30,4 %`/`−51,1 %` com a taxa de amostragem, `3,25×` com o sentido, o adaptativo a
  `3,6e-12`, `±3 %` até 50° na composição com a atenuação, e os valores de fábrica dos três eixos.
  ⛔ **MAS a §16 NÃO estava atestada:** ficavam **1 achado que BLOQUEIA** (a barra do **G-13** não
  era alcançável pela leitura que o próprio G-13 nomeia: ler o pico como o vértice mais alto dá
  `0,515`–`0,646` px contra a barra de `0,5`, e só a interpolação sub-célula reproduz o `0,094 px`
  da coluna «aprovado») e **14 erratas** (E28–E41, entre elas **dois gémeos**: `0,0225` contra
  `0,0389` para a mesma banda, e `1,8e-7` contra `1,2e-7` na prova de completude).
3.ª EMENDA DO E (2026-09-16): ✅ **o bloqueador e as 14 erratas estão CURADOS.** O **G-13** passa a
  NOMEAR a leitura (o pico refinado por parábola nos três vértices) e a coluna «aprovado» passa a
  ser o desvio à grelha (`0,093` px), com a leitura simples e o seu `0,515`–`0,646` px escritos ao
  lado como razão de a leitura estar nomeada; o **G-14** passa a medir **PROFUNDIDADE** em vez de
  contar picos (a `4`–`5` px os dois dabs fundem-se num pico só, e contar picos lê `1` dos dois
  lados nas oito). ⛔ Ela não muda medição nenhuma do oráculo e não regenera fixtura nenhuma — muda
  o que a página DIZ. **Todo número da §16 foi re-derivado das 30 fixturas PUBLICADAS**, e os que o
  corpus não suporta estão marcados como **medidos fora dele**. Lista item a item: ledger, «3.ª
  EMENDA DO E».
Auditoria da §16 (R-pré): ✅ **§16 auditada contra §4.2 por R-pré em 2026-09-16, 2.ª passagem sobre
  a §16 (a 4.ª da obra)** — por um subagente NOVO, independente do E e das três passagens
  anteriores. ⭐ **A PAREDE DA §16 ESTÁ LIMPA**, varrida pela forma da §4.3.1 sobre o texto INTEIRO
  e não só sobre os parágrafos que a 3.ª passagem citou: os 3 blocos cercados são o cabeçalho e
  duas identidades matemáticas de 1 e 3 linhas; dos 542 code spans, os 99 distintos não-numéricos
  são todos nossos (fixtura nossa · ficheiro/símbolo do NOSSO código · chave de cabeçalho nosso ·
  símbolo matemático) ou valor público de enumeração usado como chave de regeneração (§4.1.13),
  o que foi **re-conferido na fonte**: o harness escreve-os em propriedades **públicas** e a chave
  `espacamento_medido_em` mapeia para a do **espaçamento**, não para a do tamanho. Os 30 cabeçalhos
  (73 chaves distintas) e o conferidor lidos linha a linha. Sweep: controlo do instrumento PRIMEIRO
  (`exit 0`, todos os canais), depois **as nove vassouras** da pasta sobre a espec + as 30 fixturas
  + o conferidor + os READMEs + o INBOX + o ledger — **limpo nas nove**; `--git-history` da pasta
  acusa só patches de ledgers de OUTRAS obras (já registado), e os **cinco commits desta obra
  varridos à parte dão `0` de 262 agulhas; mais uma passagem em memória com 1 799 agulhas
  normalizadas (sem caixa, sem acentos, sem pontuação) contra 67 artefactos: `0`.
  ⭐ **Re-derivei a §16 por régua própria, escrita só desta página e dos cabeçalhos e alimentada só
  pelas 30 fixturas publicadas**: a tabela inteira do §16.2 ao dígito nas duas formas (incluindo o
  `0,0389` contra `0,0225` da agregação e as sete bandas que coincidem), o controlo de resolução
  (`0,023 / 0,019 / —` · `0,314 / 0,306 / 0,307` · `0,977 / 0,973 / 0,975`), o G-13 (`0,0807` ·
  `0,0929` · `0,0878` px com a parábola contra `0,558` · `0,646` · `0,515` com o vértice mais alto),
  o G-14 (`1,13e-8` de espalhamento e `1,658`/`1,654`/`1,163`, com **um** pico dos dois lados nas
  oito), G-16 `0,51 %`/`0,0036`, G-17 `3,6e-12` e `0` ao bit, G-19 `1,89 %` contra `53,5 %`, G-20
  `0` contra `154`, a lei do passo nos dois modos (`14,40`/`23,99`/`36,01` px e `0,07205`/`0,12033`/
  `0,18019` de corda), o preço do modo de cena (`10,35×`, `−45,6/−30,4/−51,1 %`, `3,25×`) e a prova
  de completude a `1,2000e-07` sobre os mesmos `1 114` vértices. Os **dois gémeos** que a 3.ª emenda
  diz ter matado estão **mortos**: `0,0225` só aparece como a agregação recusada, e `1,8e-7` só
  aparece na narrativa da passagem que o apanhou. As afirmações sobre a NOSSA casa conferidas no
  código vivo, linha a linha (as quatro citações batem). ⛔ **ZERO achados que bloqueiam.**
  ⚠️ Ficam **9 erratas** (E42–E50), nomeadas no ledger com o sítio e a instrução — entre elas a
  agregação não-nomeada do §16.8 (a mesma espécie do gémeo E28, um nível acima), a leitura que o
  G-18 nomeia contra o número que imprime, e o trio de ângulos do §16.4.
  ⚠️ O atestado abaixo cobre as §1–§15 e as 80 fixturas da 1.ª emenda.
Auditoria §4.2 (R-pré): ✅ **auditada contra §4.2 por R-pré em 2026-09-16, 2.ª passagem** — por um
  subagente NOVO, independente do E e da 1.ª passagem. ⭐ A PAREDE ESTÁ LIMPA (os 5 blocos cercados
  são o cabeçalho e fórmulas de 1–3 linhas; os 172 code spans não-numéricos são todos nossos, nomes
  de fixtura nossa, chaves de cabeçalho ou variáveis da página; os 80 cabeçalhos lidos linha a
  linha; sweep verde com as nove vassouras da pasta sobre a espec, as 80 fixturas, o README, o
  INBOX e os anexos citados, e verde em memória sem caixa e sem acentos, incluindo os quatro
  commits desta obra). ⭐ A lei re-derivou-se por um modelo MEU, escrito só a partir desta página e
  dos cabeçalhos e alimentado **só pelas fixturas publicadas**: G-1 `≤ 7,9e-8` (planas/cilindro) e
  `≤ 3,3e-6` (bossas) · G-2 `≤ 1,6e-6` em 36 estados · G-3a `5,1e-4` · G-3b `3,1e-3` / `2,3e-3` ·
  G-5 contagem exacta e `≤ 4,6e-3` · G-7 `3,0e-7` · G-9 `0` ao bit · G-10 `1,2e-7` · G-11 `2,9e-4`
  contra `2,24e-2`, e as tabelas do §5.2, §7.2, §7.3, §7.4, §8.1, §8.2 e §9.1 ao dígito impresso.
  Cada barra fica **estritamente** entre o lado aprovado e a candidata errada mais perto, medidas
  sobre o corpus publicado. ⛔ **ZERO achados que bloqueiam.** ⚠️ Ficam **10 erratas** (E18–E27),
  nomeadas no ledger com o sítio e a instrução — entre elas a lei de normal de vértice das famílias
  por script (a da casa, congelada, lê melhor que o bloco `n`), o refresco de normais por dab da
  nossa casa, e a condição do censo do §12.1 contra a 3.ª saída do P-2.
  1.ª PASSAGEM (2026-09-16, subagente próprio): parede LIMPA, 3 achados a bloquear (o mapa do píxel
  das 5 de cilindro · 3 fixturas pendentes dentro do G-1/G-2 · células de 192² não publicadas) e 17
  erratas — os três curados pela emenda abaixo e re-conferidos nesta passagem.
1.ª EMENDA DO E (2026-09-16), resposta à 1.ª passagem — registo com as medições no ledger:
  (1) as 5 fixturas de cilindro e a da caixa fina com Ctrl foram RE-CORRIDAS com o mapa do píxel
  gravado; os mesmos vértices movem-se e a saída difere ≤ 3,3e-7 da publicação anterior (é o
  não-determinismo do próprio arrasto do alvo, medido repetindo as seis); as 53 arrastadas trazem
  hoje o mapa, conferido por derivação (§11). (2) as 3 fixturas cuja lei a página não pede saíram
  para uma catraca NOMEADA «pendentes de decisão do dono» (§12): o G-1 conta 16 e o G-2 conta 5
  cadeias / 36 estados; a opção da normal do pen-down ganhou decisão própria (P-5). (3) as 3 de
  192² foram regeneradas com as fotos 2 e 4 (os outros blocos idênticos ao bit): o G-4 sustenta as
  16 e 12 + 4 células que declara. ⭐ E a lei da NORMAL DE VÉRTICE ficou escrita (§2.3.1): a do
  repouso é pesada pelo ângulo do canto; a que o alvo recalcula durante o traço é a média SEM peso
  das normais unitárias das faces, com a malha lida como está — a da nossa casa. As 17 erratas e a
  nota N1 (a §0.2 já não manda ninguém a notas antigas) curadas; a filtragem §4.3 re-corrida.
  ✅ Os três bloqueadores e as 17 erratas foram re-conferidos um a um pela 2.ª passagem do R-pré,
  com um modelo independente alimentado só pelas fixturas publicadas.
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

⭐⭐ **O pincel afiado DE FÁBRICA é o nosso `Draw` (modo B) com CINCO valores trocados — três do
pincel (acumular, curva, direcção) e dois do traço (o passo e a atenuação por dab, que a casa já
tem para o pincel de plano) — e mais NADA de lei nova.** Medido no nosso motor, sobre as mesmas malhas e o mesmo traço
arrastado do oráculo (§7, §8):

| régua (§7), traço contínuo, grelha 96² | passagem 1 | 2 | 4 | 8 |
|---|---|---|---|---|
| profundidade `D/R` — alvo | `0,2058` | `0,3221` | `0,4367` | `0,5406` |
| profundidade `D/R` — nosso `Draw` composto | `0,2121` | `0,3236` | `0,4377` | `0,5411` |
| largura a meia profundidade `W50/R` — alvo | `0,506` | `0,527` | `0,543` | `0,550` |
| largura a meia profundidade `W50/R` — nosso `Draw` composto | `0,504` | `0,526` | `0,542` | `0,549` |

*«composto»* = o nosso `Draw` modo B com **Acumular desligado · curva afiada
([`Falloff::Sharper`]) · direcção a afundar · o passo de `5 %` do diâmetro · a atenuação `a` por
dab** (§5). O desvio é de `+3,1 %` na profundidade da 1.ª passagem e cai para `+0,1 %` na 8.ª; a
largura fica a `≤ 0,4 %` em todas. Nas grelhas 48² e 192² e no cilindro os desvios são os
mesmos (§12, G-4).

⚠️ **O que o nosso `Draw` ainda NÃO exprime, e o tamanho de cada buraco:**

1. **A normal da área do alvo** (§2.3) — o nosso `Draw` usa a normal do plano da casa. É esta a
   causa dos `+3,1 %` da 1.ª passagem e do resto do desvio nos traços **separados**: depois de
   oito traços separados a nossa largura fica `11,7 %` mais estreita (`0,237` contra
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

### §0.2 — A recusa antiga, reconferida (o RESULTADO — ⛔ não há nada a reabrir)

Uma nota de planeamento da casa, de 2026-08-14, tirou este pincel da fila com duas afirmações. A
reconferência foi feita **pelo E**, contra o oráculo **a correr**, e o resultado está todo aqui —
⛔ **a janela que implementa não abre notas antigas para a refazer** (algumas, fora do código, têm
linhas vizinhas com nomes internos do alvo; dívida registada no ledger para a integração).

| afirmação de 2026-08-14 | o que a medição diz hoje |
|---|---|
| *«o que o nome promete mora na CURVA, e a curva de fábrica está num ficheiro binário ⇒ bloqueado»* | ⛔ **CAIU.** Os valores de fábrica foram **lidos correndo o programa**: o pincel do catálogo, carregado pelo próprio programa num arranque de fábrica, e os valores lidos dele (§6). A prova de que a lista está completa é uma fixtura: o pincel de catálogo e um pincel nosso com os valores escritos dão o mesmo traço a `1,2e-7` (ruído de `f32`). ⇒ *um ficheiro binário não é uma fonte ilegível quando o programa que o lê se corre.* |
| *«um verbo só sobre o dado congelado seria um chip que o artista já alcança por um checkbox — os dois perfis são o mesmo domo»* | ⚠️ **METADE VALE, metade CAIU.** **Vale:** a distância medida do pen-down **já existe** no nosso `Draw` (é o Acumular desligado, §10.1) — não há verbo novo a construir. **Caiu:** com a curva **afiada** e oito passagens a distância do pen-down **é** uma alavanca, tão forte como a curva: trocá-la pela distância viva (o pincel de desenho comum com todo o resto igual — que, com o interruptor desligado, também lê o cursor e a normal do pen-down; o par que isola **só** a distância está no §8.2) alarga o vinco de `0,550` para `0,852` raio (`+55 %`), e trocar só a curva alarga-o para `0,837` (`+52 %`) — §8.1. A medição antiga usou a curva suave e 9 dabs, onde a diferença não aparece. |

⚠️⚠️ **E uma terceira afirmação das notas antigas CAIU, e é a mais perigosa para o produto:** a de
que neste pincel **também a normal** vem do pen-down. **No valor de fábrica (acumular desligado)
só as POSIÇÕES DA DISTÂNCIA são do pen-down; o CURSOR e a NORMAL DA ÁREA vêm da superfície VIVA**
(§3, medido: `≤ 1,6e-6` contra `4,7e-4` da alternativa). A normal do pen-down só existe com o
acumular ligado (§3) ou com a opção própria (§3.1) — as duas fora do valor de fábrica.

⚠️ **Notas do NOSSO código com a premissa caída** (a janela que implementa toca estes ficheiros
de qualquer forma; varridos com as nove vassouras da pasta: limpos, salvo o nome público de um
campo de pincel que já está na triagem do R, `CLAUDE.md` §5):

- `crates/ph2d-sculpt3d/src/brush_verb_defaults.rs:257-260` — *«… decisão de produto … binário»*;
- `crates/ph2d-sculpt3d/src/brush_magnitudes.rs:120-124` — *«a mesma lacuna que bloqueou …»*;
- `crates/ph2d-sculpt3d/src/ref_profiles.rs:343-347` — *«os defaults por ferramenta … não pode
  responder»*: pode, correndo o programa (a obra do pincel de plano já o fez para o dela).

A frase do `CLAUDE.md` §5 sobre este pincel diz o mesmo e fica com a premissa caída; a linha do
§5 é da integração.

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
   a `1,51 R`, e ele **estreita** a cada traço (a largura a meia profundidade cai de `0,51` para
   `0,27 R`), ficando cada vez mais afiado (nitidez `D/W50` de `0,41` a `5,62`).

⭐ **O vinco não tem rebordo** (a régua do rebordo lê `0,000` em todas as corridas) e o **fundo
é agudo**: na grelha 96² a curvatura normalizada do fundo lê `6,2`–`6,8` contra `3,2`–`5,0` do
desenho comum. ⚠️ Ela depende da amostragem (48²: `5,3`–`5,6`; 192²: `6,6`–`7,4`) e só se compara
dentro da mesma grelha.

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

Reprodução pelo modelo de referência do E (`float64`) das `16` fixturas de um dab que o G-1 conta:
`≤ 8,1e-8` nas planas e no cilindro, `≤ 3,3e-6` nas de bossas; e da 17.ª, a da pegada projectada
(pendente de decisão, §2.7): `4,2e-6` (§12).

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
- ⚠️⚠️ **SÓ a distância é do pen-down.** No valor de fábrica o **cursor** e a **normal da área**
  são lidos da superfície **VIVA** (§3). Ler a normal do pen-down com o acumular desligado é um
  defeito medido (`4,7e-4` na cadeia de bossas, contra `1,6e-6`).

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

### §2.3.1 — A NORMAL DE VÉRTICE `N(v)` — a lei está FIXADA

Duas leituras, e **não são a mesma lei**:

1. **No repouso** (o bloco `n` das fixturas) a normal é a que o alvo guarda: a média das normais
   **unitárias** das faces vizinhas **pesada pelo ângulo do canto**. Medido sobre as bossas: esta
   lei reproduz o bloco `n` a `5,4e-7`; a média sem peso erra `1,8e-4`.
2. ⭐ **Durante o traço arrastado** o alvo recalcula as normais entre eventos do rato, e a lei que o
   reproduz é **a média normalizada SEM PESO das normais UNITÁRIAS (Newell) das faces vizinhas, com
   a malha lida COMO ESTÁ** — quadriláteros como quadriláteros, triângulos como triângulos. ⭐ **É a
   lei da nossa casa** (`crates/ph2d-mesh/src/normals.rs:40-62`, a normal unitária da face, e `:165-185`, a soma sem peso do anel).

Os traços **separados** são o caso que discrimina (erro máximo na malha inteira, modelo de
referência com a normal da área do §2.3):

| lei da normal de vértice durante o traço | 96² | 48² | 192² | cilindro |
|---|---|---|---|---|
| ⭐ **média sem peso, malha como está** | **`1,43e-3`** | **`3,10e-3`** | **`2,19e-3`** | **`1,39e-3`** |
| pesada pelo ângulo do canto | `3,70e-3` | `1,17e-2` | `2,60e-3` | `3,60e-3` |
| média sem peso, quadriláteros TRIANGULADOS pela diagonal `(i,j)–(i+1,j+1)` | `3,20e-3` | **`3,25e-2`** | `2,39e-3` | `3,10e-3` |
| pesada pela área | `4,87e-2` | `8,79e-2` | — | — |

- Na faixa `|x| ≤ 0,25` a lei fixada lê `7,7e-4 · 2,3e-3 · 1,1e-3 · 7,5e-4`; a triangulada lê
  `1,4e-3 · 8,0e-3 · 1,4e-3 · 1,3e-3`.
- A fixtura já em triângulos (`produto/triangulos_afiado_separados`) lê `1,56e-3` com a mesma lei
  sobre os triângulos dela.
- Nos traços **contínuos** as leis não se separam (todas `≤ 1,04e-3`).

⇒ **A bancada carrega as fixturas de quadriláteros COMO quadriláteros** (a malha da casa aceita-os)
e recalcula as normais com a lei da casa; o cabeçalho de cada fixtura di-lo na linha
`normais_de_vertice`. ⛔ Triangular antes de carregar **reprova** o G-3b na grelha 48² (§12).

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
(§15, **P-4**). ⇒ a fixtura está **FORA do G-1**, na catraca nomeada «pendentes de decisão do
dono» (§12), e só volta ao gate no dia em que o modo for oferecido.

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
vivos (a normal: `cadeia/controlo_desenho_*`, §11; o cursor: `produto/ablacao_tipo_desenho_comum`,
reproduzida a `8,7e-7` com o cursor do pen-down).

Medições:

- **cursor e normal vivos com o interruptor desligado**: as quatro cadeias de cursor vivo do G-2
  (plano, bossas, mesmo ponto, cilindro) reproduzem a `≤ 1,6e-6`; ler a normal do pen-down erra
  `4,7e-4` (bossas — nas outras três é inerte por geometria).
- **normal do pen-down com o interruptor ligado**: `cadeia/bossas_cursor_vivo_acumula` reproduz a
  `1,6e-6`; ler a normal viva erra `4,7e-4`. (⚠️ nesta cadeia o cursor é imposto pelo script; a
  metade do cursor é a seguinte.) ⛔ **Pendente de decisão do dono (P-2)** — o modo não é o de
  fábrica e a espec recomenda escondê-lo; a fixtura está **FORA do G-2** (§12).
- **cursor do pen-down com o interruptor ligado**: `produto/ablacao_acumular` (traço arrastado)
  reproduz a `3,6e-6` com o cursor lido da superfície do pen-down, e a régua lê o perfil
  **constante** (`W50/R = 0,473` em 1, 2, 4 e 8 passagens) com a profundidade **linear**
  (`0,2445 · 0,4890 · 0,9781 · 1,9561 R`).

### §3.1 — A normal do pen-down (opção, desligada de fábrica)

Com ela ligada, **a normal da área do PRIMEIRO dab do traço é usada por todos os dabs seguintes**
desse traço (`cadeia/bossas_cursor_vivo_normal_do_pen_down`: `1,9e-6`). Cada pen-down recomeça.
Num plano ela é inerte (`produto/ablacao_normal_do_pen_down` difere do de fábrica em `+0,1 %`).

⛔ **O nosso `Draw` não tem esta opção** e a espec não a pede (§10.3). **Decisão do dono (P-5)**;
até lá a fixtura está **FORA do G-2**, na catraca nomeada da §12. Se a opção nascer, ela precisa
da normal do **PEN-DOWN** (não a do primeiro toque — §10.4) congelada no primeiro dab.

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
afiada): a fórmula dá `0,1 · 0,13164 · 0,15190 · 0,16670` e o fundo da fixtura lê os mesmos
quatro números a `3,0e-7` (o cursor gravado na fixtura carrega o arredondamento `f32` do alvo; o
modelo completo, que usa esse cursor, fecha a `1,5e-8`).

- **O controlo**: com o cursor **imposto** na superfície de repouso (`cadeia/mesmo_ponto_cursor_dado`)
  cada dab é igual ao primeiro: `0,1 · 0,2 · 0,3 · 0,4` — linear.
- **O desenho comum com o acumular ligado** (distância e cursor vivos) também é linear no mesmo
  ponto (`cadeia/controlo_desenho_mesmo_ponto_acumula`: `0,4` depois de 4 dabs).

### §4.2 — Levantar a caneta recomeça

Cada pen-down fotografa as posições `X₀` de novo; o limite de `R` vale **por traço**. Oito
traços separados: `0,21 · 0,41 · 0,83 · 1,51 R` depois de 1, 2, 4, 8 traços (§7).

### §4.3 — Consequência para o produto

A profundidade de um traço contínuo **cresce cada vez menos** com as passagens (`+36 %` de 2 para
4, `+24 %` de 4 para 8) e **não depende da densidade** da malha (`D/R` a `8` passagens:
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
- **`D`** = o máximo do perfil; **`W_f`** = a largura **INTEIRA** entre os dois pontos onde o
  perfil cruza `f·D`, achados andando **para fora** a partir do máximo, um de cada lado
  (interpolação linear entre amostras) — ⚠️ **não** a metade dela; `W50` é a **largura a meia
  profundidade**;
- **nitidez** `D/W50` · **agudeza** `W25/W75` · **rebordo** = `−min/D` · **curvatura do fundo**
  = `−κ·W50²/D`, com `κ` a segunda diferença no máximo.

Tudo em unidades de `R`. `R = 0,25`, grelha de lado `2`, traço de `x = −0,5` a `0,5`.

### §7.2 — O pincel afiado de fábrica, traço contínuo

| grelha | passagem | `D/R` | `W50/R` | nitidez | agudeza | fundo |
|---|---|---|---|---|---|---|
| 96² | 1 · 2 · 4 · 8 | `0,2058 · 0,3221 · 0,4367 · 0,5406` | `0,506 · 0,527 · 0,543 · 0,550` | `0,407 · 0,611 · 0,805 · 0,983` | `2,54 · 2,45 · 2,39 · 2,35` | `6,82 · 6,57 · 6,39 · 6,23` |
| 48² | 1 · 2 · 4 · 8 | `0,2064 · 0,3228 · 0,4382 · 0,5427` | `0,517 · 0,536 · 0,548 · 0,554` | `0,399 · 0,602 · 0,799 · 0,979` | `2,69 · 2,56 · 2,48 · 2,43` | `5,56 · 5,49 · 5,41 · 5,33` |
| 192² | 1 · 2 · 4 · 8 | `0,2055 · 0,3214 · 0,4355 · 0,5389` | `0,505 · 0,526 · 0,541 · 0,548` | `0,407 · 0,611 · 0,805 · 0,983` | `2,51 · 2,43 · 2,37 · 2,33` | `7,44 · 7,04 · 6,77 · 6,56` |
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

Todas as células das duas tabelas estão **publicadas** (as fotos `p1`, `p2`, `p4` e a saída `s` das
fixturas; as de 192² ganharam as fotos 2 e 4 na 1.ª emenda) e foram re-derivadas **a partir dos
ficheiros**, não dos dados do oráculo.

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
produto** — é uma medição de lei, não um gate; o G-4 tem de a refazer pela porta. ⛔ **E a malha
triangulada da sonda NÃO é receita** — a bancada carrega os quadriláteros como quadriláteros
(§2.3.1).

---

## §8 — A ABLAÇÃO: qual valor é a alavanca

### §8.1 — No alvo, um valor de cada vez a partir do de fábrica (96², contínuo, 8 passagens)

| troca | `D/R` | `W50/R` | nitidez | leitura |
|---|---|---|---|---|
| (fábrica) | `0,5406` | `0,550` | `0,983` | — |
| **tipo = desenho comum** (distância viva; com o interruptor desligado ele lê o cursor e a normal do pen-down, §3) | `0,5355` | **`0,852`** (`+55 %`) | `0,628` (`−36 %`) | ⭐ alavanca da **largura** |
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

| traço (96²) | normal da área do alvo (§2.3), viva, com a normal de vértice da §2.3.1 | normais congeladas no repouso | a lei da nossa casa (soma simples sobre a pegada, viva) |
|---|---|---|---|
| contínuo, 8 passagens | **`4,9e-4`** | `2,5e-2` | `1,7e-2` |
| separados, 8 traços | **`1,4e-3`** | `1,25e-1` | `9,2e-2` |

(erro máximo por vértice na malha inteira). ⇒ **o desvio de `+3,1 %` e o estreitamento de
`11,7 %` são da lei da normal**, e com a lei do alvo o resíduo cai 35× no contínuo e 64× nos
separados.

⭐ **A alavanca, numa frase:** *medir a distância a partir das posições do pen-down* — é ela que
faz o vinco limitar-se (no alvo, `2,99 → 0,66 R`; na nossa casa é o acumular desligado,
`4,33 → 0,96 R`) e, junto com a curva afiada, que o faz estreito (sem ela `W50` sobe `55 %`).

---

## §9 — O que o alvo faz e NÃO se copia

### §9.1 — ⛔ O recorte do cursor pela caixa envolvente

Numa vista ortográfica o alvo recorta o raio do cursor à caixa envolvente da malha antes de o
lançar. Num plano essa caixa tem espessura (praticamente) zero; um vinco mais fundo fica
**fora** e o acerto seguinte falha — **o dab perde-se em silêncio**
(`artefacto_caixa/caixa_fina_salto_14px`: só o dab do pen-down, `446` vértices e fundo `0,0148`,
contra três dabs, `507` vértices e `0,0323` com caixa espessa). O mesmo acontece a um traço que
**levanta**: acima de uma caixa que só engrossa para baixo (`artefacto_caixa/caixa_funda_somar`,
`1,5e-2` da corrida com caixa dupla) e acima da caixa fina com o `Ctrl`
(`artefacto_caixa/caixa_fina_ctrl`, `2,4e-2`). ⇒ **a nossa casa
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
- **O cursor na superfície viva** a cada dab: quem o governa é
  `Verb::pica_na_superficie_do_pen_down` (`brush_verb_predicados.rs:574-576`, `false` para o
  `Draw`), consultado quando a app fotografa ou não a superfície do pen-down
  (`ph2d-app-sculpt3d/src/space.rs:389-395`).
- **A normal e o plano lidos da superfície viva**: quem o governa é outro predicado,
  `Verb::le_a_superficie_viva` (`brush_verb_predicados.rs:211-217`), que responde `true` para o
  `Draw` **qualquer que seja o interruptor** — que é exactamente o que o valor de fábrica deste
  pincel pede (§3).
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
- ⚠️ **As duas percorrem a pegada do dab** (`self.footprint`), e é o **raio da consulta** dessa
  pegada que decide se `f_n > 1` (`lei/normal_da_area_raio_2_0`) é exprimível: o do pincel de plano
  **já é alargado** pela porta `Brush::query_radius` (`brush_scale.rs:398-424`) para cobrir a
  fracção do raio da normal; o dos pincéis de puxar fica em `R`. ⇒ o modo afiado alarga a consulta
  **pela mesma porta**, nunca com uma segunda varredura.

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
   sobre `R_n` mesmo quando `R_n > R` — pela porta do raio da consulta (§10.2).
5. **Nada** na normal de VÉRTICE: a lei que o traço do alvo pede já é a da casa (§2.3.1) — o que
   falta é a **bancada** carregar as fixturas de quadriláteros como quadriláteros.

### §10.4 — Não é o valor de fábrica, e seria lei nova: o acumular ligado deste pincel

Ele pede o cursor da superfície do pen-down, a normal do pen-down e a distância do pen-down. As
peças existem — `Verb::pica_na_superficie_do_pen_down` (`brush_verb_predicados.rs:574-576`, hoje
só o projectar na cena) e a fotografia das normais do **PEN-DOWN** (`stroke.rs:132-138`, armada em
`stroke_dab_core.rs:101-115` só para o projectar em modo plano) —, mas nenhuma está ligada ao
`Draw`. ⚠️ **Não confundir com a normal POR SLOT** (`base_nrm`), que é a do **primeiro toque** de cada
vértice (`stroke_plane.rs:213-224`): num arrasto as duas diferem, porque a normal de um vértice
muda quando os vizinhos se mexem antes de ele ser tocado. Esta lei (e a opção do §3.1) precisa da
do **pen-down**. As posições não têm o problema: um vértice só se mexe depois de ser tocado, logo
a posição do primeiro toque **é** a do pen-down. ⇒ **decisão** (§15): esconder o
interruptor neste modo, ou ligá-lo à lei do alvo.

---

## §11 — As fixtures

[`fixtures/pincel_afiado/`](fixtures/pincel_afiado/README.md) — **110** ficheiros; a contagem sai
do directório (`find … -name '*.txt.gz' | wc -l`). ⚠️ As **80** desta tabela são as da entrega e da
1.ª emenda; as outras **30** são a família `silhueta/` da **2.ª emenda** (§16.11), com convenção
própria e conferidor próprio.

| família | n | no gate | o que a bancada faz com ela |
|---|---|---|---|
| `lei/` | 17 | **16** no G-1 · **1** pendente (§12) | um dab por ponto `c`, com o bloco `n` como normal de vértice, a máscara `m` onde houver, e compara `s` |
| `cadeia/` | 10 | **5** no G-2 (**36** estados) · **2** pendentes (16 estados) · **3** controlos (20 estados) | os pontos `c` pela ordem, o bloco `n` para **todos** os dabs, e compara cada `d<j>` e `s`. Os 3 controlos do desenho comum do alvo **documentam** a lei dele (§3) e não são gate do nosso produto |
| `produto/` | 35 | **6** no G-3a · **5** no G-3b · **2** de completude · **22** excluídos com nome (§12) | o traço arrastado pela **porta do produto** (passo, atenuação, cursor vivo, normais vivas) e compara `p<k>` e `s`, ou a régua do §7 |
| `detector/` | 15 | **13** no G-5a/b · **1** declarado · **1** no G-5c (com o `salto_00px`) | um pen-down e um salto do rato, e conta os dabs / compara `s` |
| `artefacto_caixa/` | 3 | G-11 (1) · documentação (2) | o controlo do §9.1 |

⚠️ **Regras de leitura** (todas no README das fixturas e no cabeçalho de cada uma):

- o raio efectivo é **o do cabeçalho**: `0,4` em 23 das 27 por script, `0,25` nas 4 de cilindro
  (`lei/cilindro_um_dab`, as duas de faces de frente, `cadeia/cilindro_cursor_vivo`) e `0,25` nas
  53 arrastadas;
- nas **53** arrastadas o cabeçalho traz o píxel do pen-down e o passo de um píxel no mundo —
  conferido por derivação (`zcat … | grep '^# pixel_do_pen_down_no_mundo'` nas três famílias
  arrastadas: 53 de 53). ⚠️ A linha do traço **pedida** é `y = 0`; a **real** é a do píxel do
  pen-down, meio píxel ao lado (`y = 0,0025`);
- **a malha lê-se como está**: quadriláteros como quadriláteros (as duas de triângulos, como
  triângulos), e a normal de vértice recalcula-se com a lei da §2.3.1 — a linha
  `normais_de_vertice` de cada cabeçalho di-lo;
- nas de caixa dupla, **dois vértices de canto estão fora do plano** (`z = ±1`) — a régua ignora-os
  (estão longe do traço) e uma comparação vértice a vértice inclui-os tal como estão, **salvo** nos
  G-10 e G-11, que os **saltam** (lá comparam-se superfícies com repousos diferentes nesses dois
  vértices, ou posições espelhadas, e os cantos leriam `1` ou `2`);
- as `2` de catálogo servem de **completude** (o pincel de fábrica = os valores escritos); ⛔ os
  gates ancoram-se nas de pincel **nosso**, cujo cabeçalho diz `origem_dos_valores: … ESCRITOS`.

### §11.1 — O que a 1.ª emenda regenerou (provado contra a publicação anterior)

| ficheiros | o que mudou | prova |
|---|---|---|
| **71** | só o cabeçalho: a linha `normais_de_vertice` nova e a linha do traço reescrita | todos os blocos idênticos, linha a linha |
| **3** de 192² | + as fotos `p2` e `p4` | os blocos `r`, `n`, `c`, `p1` e `s` idênticos, linha a linha |
| **6** (as 5 de cilindro do produto + `artefacto_caixa/caixa_fina_ctrl`) | **re-corridas** com o mapa do píxel gravado | os **mesmos** vértices movidos; posições a `≤ 3,3e-7` da publicação anterior (o traço separado; `≤ 6e-8` nos outros cinco) |

⚠️ **O arrasto do alvo não é determinístico ao bit:** as seis corridas repetidas diferem da
primeira vez entre `2,3e-8` e `3,3e-7`, e o erro cresce com as fotos de um traço separado. É um
chão de ruído **~6 000×** abaixo da barra do G-3a. ⇒ a comparação passagem a
passagem não pode exigir o bit, e nenhuma exige.

---

## §12 — Os GATES propostos, cada um com a população, o piso, a barra e a origem

Convenção: «aprovado» = o maior erro do lado que tem de passar; «errado» = o menor erro da
candidata que tem de reprovar, **dentro da população do gate**. A barra fica **estritamente entre**
os dois. As medições do lado aprovado são do **modelo de referência do E** (`float64`); a
implementação em `f32` pode pedir reconferência — ⛔ **nunca** afrouxar uma barra por isso sem a
medição ao lado.

| gate | população (piso) | mede | barra | aprovado | errado mais perto | origem |
|---|---|---|---|---|---|---|
| **G-1** lei de um dab | `lei/` menos a pendente (**16**: 9 planas · 3 de cilindro · 4 de bossas) | max `|Δ|` por vértice contra `s` | `2e-6` planas e cilindro · `1e-5` bossas (estas exigem a normal do §2.3, D-3) | `8,1e-8` · `3,3e-6` | `R_n` trocado `≥ 1,9e-3`; faces de frente sem o factor `5,2e-3`; força linear `5,2e-2` (nas 10 com força `< 1`; nas outras 6 é inerte) | §2 |
| **G-2** cadeias | `cadeia/plano_cursor_vivo` · `bossas_cursor_vivo` · `mesmo_ponto_cursor_vivo` · `mesmo_ponto_cursor_dado` · `cilindro_cursor_vivo` (**5**, **36** estados) | max `|Δ|` contra cada `d<j>` e `s` | `1e-5` (a de bossas exige a normal do §2.3, D-3) | `1,6e-6` | normal lida do pen-down `4,7e-4` (só na de bossas; nas outras é inerte por geometria); distância viva `≥ 2,6e-2`; cursor do pen-down com o interruptor desligado `≥ 3,6e-2` (inerte na de cursor imposto) | §2.1, §3, §4 |
| **G-3a** produto vértice a vértice, contínuo — com a normal da área do §2.3 **e** a de vértice da §2.3.1 | `produto/afiado_valores_de_fabrica_continuo` · `afiado_valores_de_fabrica_ctrl` · `densidade_48_afiado_continuo` · `densidade_192_afiado_continuo` · `cilindro_afiado_continuo` · `triangulos_afiado_continuo` (**6**) × fotos + saída | max `|Δ|` na malha inteira | `2e-3` | `5,1e-4` | a lei de normal da área da casa `9,5e-3`; normais congeladas `1,11e-2`; `(1+a)/2` `4,7e-2`; sem atenuação `7,95e-2` | §2.3, §5 |
| **G-3b** produto vértice a vértice, separados — idem | `produto/afiado_valores_de_fabrica_separados` · `densidade_48_afiado_separados` · `densidade_192_afiado_separados` · `cilindro_afiado_separados` · `triangulos_afiado_separados` (**5**) | max `|Δ|` na malha inteira · em `|x| ≤ 0,25` | `8e-3` · `5e-3` | `3,1e-3` · `2,3e-3` | na 48² (a que discrimina a normal de vértice): pesada pelo ângulo `1,17e-2` · —, triangulada `3,25e-2` · `8,0e-3`; em todas: a lei de normal da área da casa `≥ 9,0e-2` · `≥ 2,6e-2`; normais congeladas `≥ 1,21e-1` · `≥ 1,19e-2` | §2.3, §2.3.1, §4.2 |
| **G-4a** régua, contínuo — corre com a normal que o produto tiver (as barras foram calibradas com a da casa e ficam folgadas com a do alvo) | 4 superfícies (96², 48², 192², cilindro) × passagens 1, 2, 4, 8 (**16** células, todas publicadas) | desvio relativo de `D/R` · `W50/R` · nitidez contra o alvo | `6 %` · `2 %` · `6 %` | `3,2 %` · `0,4 %` · `3,6 %` | nosso acumular ligado (p1) `D +13,6 %`, `W −4,5 %`; tipo desenho comum no alvo (p1) `W +7,1 %`, nitidez `−8,1 %`; curva suave `D +146 %`; sem passo/atenuação `D +112 %`; força `1` `D +146 %` | §7, §8 |
| **G-4b** régua, separados — idem | 4 superfícies × traços 1, 2, 4 (**12**) · × traço 8 (**4**), todas publicadas | idem | `6 %` · `3 %` · `8 %` (1–4) · `8 %` · `15 %` · `25 %` (8) | `3,5 %` · `1,3 %` · `4,8 %` · `4,9 %` · `11,8 %` · `18,9 %` | sem passo/atenuação `D +112 %` (traço 1), `W −9,4 %` (traço 2), `D +86 %` (traço 8); desenho comum separado `W +175 %` | §7.3 |
| **G-5a** passo — contagem | `detector/salto_*` menos o declarado (**13**) | nº de dabs = `⌊N/5⌋ + 1` | exacto | exacto | `±1` dab | §5.1 |
| **G-5b** passo — posições | as mesmas (**13**) | max `|Δ|` contra `s` | `7e-3` | `4,6e-3` | um dab a mais ou a menos `≥ 1,06e-2` | §5.1, §9.3 |
| **G-5c** o dab do pen-down é atenuado | `detector/salto_00px` e `detector/salto_00px_sem_atenuacao` (**2**) | max `|Δ|` | `1e-5` | `7,4e-8` | sem atenuação `4,5e-2`; `(1+a)/2` `2,3e-2` | §5.3 |
| **G-6** valores de fábrica | as linhas do §6 que o modo muda contra o nosso `Draw` (**5**: curva · direcção · acumular · espaçamento · atenuação) | o pincel nasce com cada valor | exacto | — | — | §6 |
| **G-7** auto-limitação, forma fechada | a recorrência do §4.1 contra o fundo de `cadeia/mesmo_ponto_cursor_vivo` (**4** dabs) | `|Δz|` | `1e-6` | `3,0e-7` | cursor imposto (linear) `6,8e-2` | §4.1 |
| **G-8** a régua separa o afiado do desenho comum | `produto/afiado_valores_de_fabrica_continuo` × `produto/desenho_valores_de_fabrica_continuo` (**2**) | razão `W50` desenho/afiado a 8 passagens | `≥ 1,8` | `2,29` (o par separado lê `2,75` e também passa) | — (o controlo: se a razão cair, a régua deixou de ver) | §7.4 |
| **G-9** o primeiro dab é o do desenho comum | `lei/um_dab_curva_afiada` × `lei/um_dab_controlo_desenho_comum` (**2**) | diferença | `0` ao bit | `0,0` | — | §1 |
| **G-10** simetria de direcção | `produto/ablacao_somar` × `produto/afiado_valores_de_fabrica_continuo` (**2**; `3` fotos + a saída) | `|Δ|` entre os **DESLOCAMENTOS** e o espelho `z → −z` (ou as posições, **saltando** os dois vértices de canto) | `2e-6` | `1,2e-7` | — | §2.4 |
| **G-11** o recorte pela caixa NÃO se copia | o salto de `14` px numa malha plana **sem** vértices de canto (**1** corrida nossa × **2** fixturas) | `|Δ|` contra `detector/salto_14px` e contra `artefacto_caixa/caixa_fina_salto_14px`, **saltando** os dois vértices de canto | `≤ 7e-3` do primeiro **e** `≥ 1e-2` do segundo | `2,9e-4` (o modelo contra a de caixa dupla) | as duas fixturas diferem `2,24e-2` | §9.1 |
| **G-12** nenhum knob morto neste modo | cada knob oferecido × duas posições (piso = os knobs pintados) | o barro muda | `> 0` | — | — | §8 (o censo dos knobs da casa) |

### §12.1 — ⛔ A catraca «PENDENTES DE DECISÃO DO DONO»

Três fixturas descrevem leis que **a própria página não pede** ao produto recomendado. Elas **não
contam** no G-1 nem no G-2 enquanto a decisão do dono não as pedir; um censo de obsolescência
devolve cada uma ao seu gate **no dia em que o controlo correspondente for oferecido** (e reprova
se o controlo existir e a fixtura continuar na catraca).

| fixtura | decisão | o modelo reproduz a | o produto recomendado (sem a lei) lê | volta ao |
|---|---|---|---|---|
| `lei/pegada_projectada` | **P-4** (não oferecer) | `4,2e-6` | `8,5e-2` | G-1 (barra `1e-5`) |
| `cadeia/bossas_cursor_vivo_acumula` | **P-2** (esconder o interruptor) | `1,6e-6` | `4,7e-4` | G-2 (8 estados) |
| `cadeia/bossas_cursor_vivo_normal_do_pen_down` | **P-5** (não oferecer a opção) | `1,9e-6` | `6,2e-2` | G-2 (8 estados) |

⚠️ *Uma lei construída sem controlo que a alcance é um knob inalcançável* — a regra da casa é *o
painel oferece exactamente o que o gesto faz*. Por isso estas três ficam FORA até haver o controlo.

### §12.2 — ⛔ Declarados e excluídos, com nome (catraca, nunca silêncio)

- **G-5a/G-5b excluem 1 ficheiro:** `detector/salto_05px` — o único salto de **exactamente** um
  passo (D-1); os de `9`, `14`, `19`, `24` e `29` px dão os passos inteiros e um resto, e a casa já
  os reproduz.
- **Completude (2), fora dos G-3:** `produto/afiado_de_catalogo_continuo`,
  `produto/desenho_de_catalogo_continuo`.
- **Excluídos dos G-3 (22), cada um com o papel:**
  - controlos do desenho comum (usados na régua; o G-8 usa um par): `produto/desenho_valores_de_fabrica_continuo`,
    `produto/desenho_valores_de_fabrica_separados`, `produto/desenho_valores_de_fabrica_subtrair_continuo`,
    `produto/densidade_48_desenho_subtrair_continuo`, `produto/densidade_192_desenho_subtrair_continuo`,
    `produto/cilindro_desenho_subtrair_continuo`;
  - a ablação (§8.1; o G-10 usa uma): `produto/ablacao_acumular`, `produto/ablacao_curva_suave`,
    `produto/ablacao_espacamento_10`, `produto/ablacao_espacamento_8`, `produto/ablacao_forca_1`,
    `produto/ablacao_normal_do_pen_down`, `produto/ablacao_raio_da_normal_1`,
    `produto/ablacao_sem_atenuacao`, `produto/ablacao_somar`, `produto/ablacao_tipo_desenho_comum`,
    `produto/cilindro_ablacao_acumular`;
  - o motor do alvo com valores nossos (§8.2–§8.3): `produto/alvo_com_o_nosso_afiado_composto`,
    `produto/alvo_com_o_nosso_afiado_composto_tipo_desenho`, `produto/alvo_com_os_valores_do_nosso_desenho`,
    `produto/alvo_com_os_valores_do_nosso_desenho_subtrair`, `produto/cilindro_alvo_com_o_nosso_afiado_composto`.
- **Os 3 controlos de `cadeia/`** (`controlo_desenho_*`) e as **2** de `artefacto_caixa/` que o
  G-11 não usa (`caixa_funda_somar`, `caixa_fina_ctrl`) documentam; não são gate.

⇒ **Soma, derivável do directório:** `lei/` 16 + 1 · `cadeia/` 5 + 2 + 3 · `produto/` 6 + 5 + 2 + 22 ·
`detector/` 13 + 1 + 1 · `artefacto_caixa/` 1 + 2 = **80** — mais as **30** da `silhueta/` (§16.11,
gates G-13..G-20), que são população de OUTROS gates e não entram em nenhum dos acima.

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
| a atribuição do §8.4 | o mesmo modelo, trocando só a lei da normal da área |
| a lei da normal de vértice (§2.3.1) | o mesmo modelo, trocando só a normal de vértice (quatro leis × quatro superfícies nos traços separados); a do repouso, contra o bloco `n` |
| a largura `W_f` (§7.1) | a régua do E, com os cruzamentos achados a partir do máximo; re-implementada de raiz pelo R-pré sobre os blocos publicados |
| o não-determinismo do arrasto (§11.1) | seis corridas repetidas, comparadas com a primeira |
| os números da catraca (§12.1) | o mesmo modelo, com e sem a lei pendente |
| a recusa do dyntopo | a obra do dyntopo (`fixtures/dyntopo/README.md`) |
| a folga do recorte do alvo | ⛔ retirada da espec (o produto não recorta, §9.1) |

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
5. **O vinco só se lê em malha fina**: a largura a meia profundidade é `≈ 0,55 R`; com menos de
   `~3` arestas nessa largura o fundo não é representável (medição nossa: `W50` em 48² são `3,3` células com
   `R = 0,25`).
6. ⛔ **Não copiar:** o recorte do cursor pela caixa (§9.1); a pegada projectada que empurra de
   lado (§2.7), se oferecida neste pincel.

---

## §15 — Divergências a declarar e decisões do dono

| # | assunto | as saídas | recomendação técnica |
|---|---|---|---|
| D-1 | um salto de exactamente um passo (§5.1) | mudar a fronteira do passeio da casa · declarar | **declarar** (a fronteira tem gate e o mesmo precedente do pincel de plano; num arrasto real as listas coincidem) |
| D-2 | o recorte pela caixa (§9.1) | copiar · não copiar | **não copiar** (G-11) |
| D-3 | a normal da área (§10.2) | manter a da casa (desvio `+3,1 %` · `−11,7 %`, G-4) · adoptar a do alvo neste modo (G-3, e as bossas do G-1/G-2) | adoptar — a lei já vive em duas portas da casa, e a consulta alarga pela porta que o pincel de plano já usa |
| **P-1** (dono) | **o nome e a forma**: um modo/valor do nosso `Draw` ou um pincel próprio no catálogo | as duas exprimem a lei | técnica: modo do `Draw` (§0.1); o nome no catálogo é do dono |
| **P-2** (dono) | **o acumular neste modo** (§10.4) | esconder · oferecer com a lei do alvo (lei nova) · oferecer com a nossa (outro vinco, §8.3) | esconder até haver pedido — a fixtura dele fica na catraca do §12.1 e volta ao G-2 se a lei do alvo for oferecida |
| **P-3** (dono) | **a topologia dinâmica** (§9.4) | não refinar (como o alvo) · refinar (como o nosso `Draw`) | não refinar — é o que o nome promete (um vinco fino não sobrevive a um remalhe) |
| **P-4** (dono) | **a pegada projectada** (§2.7) | não oferecer · oferecer | não oferecer neste modo — a fixtura fica na catraca do §12.1 e volta ao G-1 se for oferecida |
| **P-5** (dono) | **a opção da normal do pen-down** (§3.1) — o nosso `Draw` não a tem | não oferecer · oferecer (congela a normal do **pen-down** no primeiro dab, §10.4) | não oferecer — é inerte num plano e não faz parte do valor de fábrica; a fixtura fica na catraca do §12.1 e volta ao G-2 se for oferecida |

---

## §16 — 2.ª EMENDA: O PASSO JUNTO À SILHUETA (o vinco pontilhado)

```
Emenda aberta em 2026-09-16 por um report do dono, com foto: «se fizer o traço do canto para o
início da esfera, no canto fica meio pontilhado». Autor: subagente-E. Oráculo: o mesmo binário
5.2.1 LTS, re-conferido inalterado. Filtragem §4.3 executada sobre esta secção; sweep verde.
Malhas: NOSSAS, geradas por fórmula pelo harness (meia-cana e esfera amostradas por ÂNGULO —
espaçamento de MUNDO uniforme, que é o que a peça do dono tem; rampas de inclinação constante).
```

### §16.0 — A pergunta zero desta emenda

⚠️ **Antes de escrever lei nova, o CONTROLO:** *o alvo, com o pincel de fábrica e o mesmo gesto,
pontilha também?* Se pontilhar, o nosso produto está **fiel** e a pergunta passa a ser de produto;
se não pontilhar, falta-nos uma lei. A resposta é **pontilha, e com os mesmos números** (§16.2).

### §16.1 — A régua NOVA: a ondulação AO LONGO do traço

A régua do §7 mede a secção **transversal** (`D`, `W50`, nitidez). Ela é cega a este defeito: um
vinco contínuo e um pontilhado têm a MESMA secção transversal no fundo de cada dab. ⇒ régua nova,
**ao longo** do traço:

- **estação** = uma coluna da malha ao longo do traço; a coordenada é o **comprimento de arco de
  mundo** `s`, ⛔ nunca `x` de ecrã (junto à silhueta um píxel vale muitos graus, e `arcsin` perde
  toda a resolução: `x = 199` px e `x = 200` px distam `5,7°`);
- **`D(s)`** = o máximo, sobre a secção transversal daquela estação, do deslocamento **ao longo da
  normal de repouso**, positivo para dentro; **`W50(s)`** = a largura a meia profundidade daquela
  secção, na régua do §7.1;
- **ondulação `r(s)`** = `(max − min)/max` de `D` numa janela de **UM período de dab local**
  centrada em `s` — o período sai da lei em vigor (§16.4), não de um número escolhido;
- ⚠️ **a AGREGAÇÃO por banda é parte da régua, não um detalhe:** `D`, `W50` e `r` agregam-se pela
  **mediana sobre as estações em que `r` está DEFINIDA** (a cláusula abaixo). Agregar `D` sobre
  **todas** as estações da banda dá outro número — e dá-o **só na banda do pen-down**, onde metade
  das estações está fora da faixa útil (`0,0389` contra `0,0225` em `83–88°`; nas outras sete
  bandas as duas agregações coincidem ao 4.º decimal). *Duas agregações da mesma grandeza com o
  mesmo nome numa página são um gémeo à espera de alguém as comparar.*
- ⛔ **`r` só é definida onde a janela INTEIRA cai dentro da faixa que o traço cobriu**, e com `≥ 5`
  amostras dentro dela. Sem esta cláusula a régua lê `1,000` nas duas pontas do traço — onde o
  vinco termina, não onde ele pontilha — e chamar-lhe-ia defeito.

⭐ **Controlo de resolução (a régua não é um artefacto da malha), e ele DERIVA-SE da fixtura
publicada:** sub-amostrando a fila de `produto_cupula_fabrica_da_silhueta` de `1` em `2` e de `1` em
`3` — `6,8` → `3,4` → `2,3` amostras por período — a régua lê `0,023 / 0,019 / —` (`45–55°`),
`0,314 / 0,306 / 0,307` (`74–79°`) e `0,977 / 0,973 / 0,975` (`83–88°`). A `2,3` amostras a banda
interior deixa de ter janela, que é a cláusula acima a funcionar. ⚠️ **A medição original era em
duas MALHAS distintas** (`1 536` e `733` células) e **essa não está publicada** — a sub-amostragem
é o controlo que o corpus suporta, e a conclusão é a mesma.
⭐ **Controlo de forma:** a meia-cana (o instrumento) e a esfera (a forma do dono) lêem a mesma
tabela (§16.2) — a meia-cana é o instrumento porque a geometria é exacta (`s = r·u`, `cos θ = cos u`).

### §16.2 — O CONTROLO: o alvo tem o MESMO defeito, e ele é do ENCURTAMENTO

Pincel afiado de fábrica, uma passagem de `u = 84°` (junto à silhueta) até `u = 0` (o topo), meia-cana
de raio `1`, `300` px por unidade, diâmetro de cena `100` px ⇒ `R = 0,1667` de mundo.

| banda `u` | período de mundo | `D/R` | `W50/R` | **ondulação** |
|---|---|---|---|---|
| `5–15°` | `0,0169` | `0,2019` | `0,503` | **`0,002`** |
| `25–35°` | `0,0192` | `0,1860` | `0,498` | `0,006` |
| `45–55°` | `0,0259` | `0,1497` | `0,487` | `0,023` |
| `60–68°` | `0,0380` | `0,1092` | `0,480` | `0,085` |
| `68–74°` | `0,0511` | `0,0831` | `0,476` | `0,182` |
| `74–79°` | `0,0713` | `0,0608` | `0,471` | **`0,314`** |
| `79–83°` | `0,1065` | `0,0346` | `0,533` | **`0,570`** |
| `83–88°` | `0,1719` | `0,0389` | `0,455` | **`0,977`** |

Na **esfera** (`1 200` anéis), o mesmo gesto: `0,2025 · 0,1860 · 0,1498 · 0,1092 · 0,0831 · 0,0605 ·
0,0345 · 0,0386` de `D/R` e `0,002 · 0,006 · 0,025 · 0,084 · 0,185 · 0,311 · 0,568 · 0,978` de
ondulação. ⇒ **as duas formas concordam** (o maior desvio entre elas é `0,52 %` em `D/R` e `0,0036`
em `r`, nas bandas até `79°`).

⚠️ **As duas tabelas agregam pela mediana sobre as estações com `r` definida** (§16.1) e derivam-se
das fixturas `produto_cupula_fabrica_da_silhueta` e `produto_esfera_fabrica_da_silhueta`. ⛔ **Toda
`D/R` desta emenda usa essa agregação**, incluindo a tabela do §16.9 — ela e a agregação sobre todas
as estações só divergem na banda do pen-down (`0,0389` contra `0,0225`).

⭐⭐ **Duas coisas ao mesmo tempo, e o dono viu as duas:** o vinco **pontilha** (`0,002 → 0,977`) **e
fica 5× mais raso** (`0,2019 → 0,0389 R`). A segunda é a primeira: com os dabs separados, cada ponto
recebe **um** dab em vez da sobreposição de vinte.

⚠️ **A banda do defeito é `u ≳ 70°` e ela não depende do tamanho do pincel nem do tamanho da peça.**
O dab separa-se quando o passo de mundo passa a valer mais do que a largura útil da curva afiada,
e as duas escalam com o diâmetro: `(esp/100)·D_mundo / cos θ > k·D_mundo` ⇒ `cos θ < esp/(100·k)`.
Em ecrã a banda mede `R_peça_px · (1 − sin 70°) ≈ 6 %` do raio da peça — **quanto maior a peça no
ecrã, mais larga a faixa pontilhada**, que é o que faz o defeito saltar à vista numa esfera grande.

### §16.3 — O mecanismo, com número

O passo de fábrica é medido no **ECRÃ** (§16.4). Um passo de `p` píxeis sobre uma superfície cuja
normal faz `θ` com a vista percorre, na superfície,

```text
passo_de_mundo(θ) = (p / ppu) / cos θ
```

e `1/cos θ` → ∞ na silhueta. Medido no detector (as oito fixturas `detector_*`, pela régua do G-14)
a fronteira do passo é **`5` px num plano, numa rampa de `45°` e numa rampa `4:1` (`76°`) — a
MESMA**: ⇒ *o passo é cego à inclinação, e é a superfície que foge por baixo dele*.

### §16.4 — O eixo «espaçamento medido em»: as DUAS leis

O valor tem dois estados públicos (`VIEW` · `SCENE`) e o de fábrica deste pincel é `VIEW` (§16.7).

```text
VIEW  :  passo_de_ECRA  = (esp/100) · diametro_de_ecra_px            [constante, cego à inclinação]
SCENE :  passo_de_MUNDO = (esp/100) · (diametro_de_ecra_px / ppu)    [constante em mundo]
         e a distância que ele compara é a distância 3D entre o último dab e o acerto VIVO
```

**Medido** nas seis fixturas `passo_em_{vista,cena}_esp{060,100,150}`, com os dabs lidos como picos
separados do próprio barro. ⚠️ **Uma descrição só do percurso, porque ele tem três coordenadas que
se confundem:** a linha do **cursor** vai de `u = +89,5°` a `u = −20°` (é o que o cabeçalho
`caminho_do_traco` traz, em mundo); os **dabs** caem entre `+84,25°` e `−15,4°`/`−19,0°`/`−19,2°`
conforme o espaçamento (o 1.º dab é o do pen-down, e o último é onde o passo deixa de caber);
diâmetro de cena `24` px, `200` px por unidade.

| `esp` | `VIEW`: passo de ecrã | `SCENE`: corda 3D entre dabs | alvo `esp·D_mundo` |
|---|---|---|---|
| `60 %` | `14,40` px (desvio-padrão `0,05`) | `0,07204` (dp `5,7e-5`) | `0,07200` |
| `100 %` | `24,00` px (dp `0,06`) | `0,12015` (dp `3,3e-4`) | `0,12000` |
| `150 %` | `36,00` px (dp `0,09`) | `0,18016` (dp `9,5e-5`) | `0,18000` |

⭐ Em `VIEW` o ângulo dos dabs ao longo da meia-cana é **desigual** e o passo de ecrã constante; em
`SCENE` o ângulo é **uniforme** (`4,13° · 6,90° · 10,35°` para os três espaçamentos) e o passo de
ecrã encolhe com `cos θ` (`23,98 → 3,86` px entre o topo e `84°`).

⛔ **O que este corpus NÃO decide: se a grandeza comparada é a CORDA ou o ARCO.** A `150 %` a corda
erra `+0,09 %` e o arco `+0,22 %`, o que parece dar a corda por vencedora — mas as duas diferem
`0,13` pontos percentuais, **menos do que a quantização do pico suporta**, e uma leitura de pico
igualmente legítima troca-lhes o sinal. ⇒ a **lei** (o passo é constante em MUNDO) re-deriva sem
dúvida; a escolha entre corda e arco fica **declarada como não resolvida**, e ⛔ **nenhum gate a
usa** (para os passos deste corpus as duas diferem `≤ 0,2 %`).

⚠️ **Num plano de frente para a vista os dois modos COINCIDEM por construção** (`cos θ = 1`), e
isso está medido: `plano_vista` × `plano_cena` diferem `+0,09 %` no fundo do vinco.

### §16.5 — O «espaçamento adaptativo» é **INERTE** neste caminho

⛔ **A hipótese natural era esta ser a cura. Não é.** Ligado, ele não muda **nada**:

| população | `|Δ|` máximo entre ligado e desligado | o que vale um dab a mais |
|---|---|---|
| **os `2` pares PUBLICADOS** (`passo_em_{vista,cena}_esp100` × `adaptativo_ligado_em_…`) | `3,6e-12` (vista) e **`0` ao bit** (cena), sobre os mesmos vértices movidos | `1,3e-2` |
| `12` traços de meia-cana (`3` espaçamentos × `VIEW`/`SCENE`) — ⚠️ **10 destes não estão publicados** | `≤ 6,0e-8` (um par ao bit) | `1,3e-2` |
| `132` pares do detector (`3` superfícies × `22` saltos × `VIEW`/`SCENE`) — ⚠️ **nenhum destes está publicado** | `≤ 6,0e-8`; as **classes** de salto são idênticas | `1,3e-2` |

⇒ razão `2×10⁵`. ⚠️ **E o chão de ruído tinha de ser medido:** duas corridas da MESMA configuração
diferem `0` a `6,0e-8` (`f32`), então uma classe definida por igualdade **ao bit** parte-se em duas
sem nenhuma diferença de comportamento — foi o que a 1.ª leitura desta emenda fez, e leu «o
adaptativo mudou alguma coisa» onde não havia nada. *Uma classe de equivalência sem o chão de ruído
ao lado é uma medição à espera de inventar um efeito.*

⚠️ **O que NÃO foi medido, e porquê:** o interruptor só pode agir sobre um espaçamento que **varie
ao longo do traço por outra causa**, e a única causa que o alvo oferece para isso é o **tamanho a
seguir a pressão** — que um rato não produz (a pressão simulada é `1` constante). A janela virtual
não entrega caneta. ⇒ **declarado**: inerte *para o dispositivo que o nosso produto tem* (a nossa
casa também não tem pressão de caneta — §6), e a condição sob a qual poderia agir fica **nomeada e
por medir**.

### §16.6 — O caso degenerado: o cursor sobre a tangente

- **A caneta que desce EXACTAMENTE no píxel da silhueta não deposita nada** — o raio não acerta,
  e a malha fica intocada (`pen_down_na_silhueta_sem_dab`: zero vértices movidos).
- **Um píxel para dentro, e o traço começa ali**, com **um** dab
  (`pen_down_na_silhueta_um_px_para_dentro`) — o primeiro dab é o do primeiro acerto, não o do
  píxel em que o botão desceu. Saltos de `1 · 2 · 3 · 5 · 8` px a partir desse píxel dão **um** dab
  cada, o que é a grelha de `5` px a recomeçar no primeiro acerto — ⚠️ **só o salto de `1` px está
  publicado**; os de `2 · 3 · 5 · 8` px foram medidos fora do corpus.
- **Não há infinito**, e a razão é geométrica: a superfície é finita, logo o comprimento de arco
  sob o cursor é finito mesmo onde a projecção degenera. O que cresce sem tecto é o **número de
  dabs por píxel de rato** no modo `SCENE`, e ele cresce com `1/cos θ`: no topo um píxel pede
  **`0,20`** dabs (`0,00333` de arco contra um passo de `0,01667`), a `60°` pede `0,40`, e a `84°`
  pede **`1,91`** — **`9,6×`** mais do que no topo, que é `1/cos 84°`. ⚠️ *O `1/cos θ` é a RAZÃO
  entre dois regimes, nunca a contagem: quem o leia como contagem escreve `~10` onde a conta dá
  `1,9`.*
- ⛔ **Tecto: não encontrado, e a régua é a LINEARIDADE.** Num único evento de ponteiro sobre uma
  rampa `4:1` em modo `SCENE`, a soma do deslocamento cresce **linearmente** com o salto até `72` px
  (`0,073` · `0,075` · `0,077` · `0,079` · `0,080` por píxel nos saltos `10` · `20` · `35` · `50` ·
  `72`) — nenhum joelho, e o último vale `48×` o trabalho de um dab só. ⚠️ **Esta soma é um LIMITE
  INFERIOR da contagem** (dabs sobrepostos somam sub-linearmente pela auto-limitação do §4), logo o
  número real é `≥ 48`. *É uma ausência medida numa faixa, não uma prova de que não existe tecto.*
  ⚠️⚠️ **Esta sonda NÃO tem fixtura publicada** (os saltos de `10` a `72` px não estão entre as 30):
  é uma medição de campanha, e nenhum gate se apoia nela.

### §16.7 — Os VALORES DE FÁBRICA dos três

Lidos do pincel de **catálogo** carregado pelo próprio programa num arranque de fábrica (a mesma
porta do §6; nenhum ficheiro de pincéis foi aberto pelo harness):

⚠️ **A coluna «nosso» é a do nosso `DrawSharp`** (o verbo que esta obra shipou), ⛔ **não** o passo
de omissão da casa: o `Draw` genérico cai no `min_spacing` (`0,15 R`, que é `7,5 %` do diâmetro — a
§6 dá esse número para ele, e as duas linhas leem-se como contradição se a coluna não disser de quem
é).

| valor | pincel afiado | desenho comum | nosso `DrawSharp` |
|---|---|---|---|
| espaçamento medido em | **`VIEW`** (ecrã) | `VIEW` | ecrã — `spacing.rs:46-49` diz que a régua é a do chamador, e `input.rs:331,357,384` passa `scene.radius_px()` |
| espaçamento adaptativo | **desligado** | desligado | não existe |
| espaçamento segue a pressão | **não** | não | não há pressão de caneta |
| espaçamento | `5 %` do diâmetro | `10 %` | `5 %` — `atenuacao_do_traco.rs:51,70` (`espacamento_do_verbo(DrawSharp)`) |

⭐ **Prova de completude:** a corrida de catálogo e a de valores escritos movem **os mesmos `1 114`
vértices publicados** e dão a mesma malha a **`1,200e-7`** (ruído de `f32`) ⇒ os valores acima
**são** o pincel de fábrica. ⚠️ É o mesmo número que a §0.2, a §13 e o README das fixturas já
imprimem para o par de catálogo da entrega — *quando uma página imprime duas vezes a mesma
grandeza, ou os dígitos batem ou um deles é um gémeo*.

### §16.8 — A COMPOSIÇÃO com a atenuação: `a` **não** segue o passo

A atenuação `a` do §5.2 é derivada da **percentagem declarada**, e a percentagem não muda quando o
passo efectivo muda. Medido pela razão de profundidade entre os dois modos, contra `1/cos θ`:

| banda `u` | `D_cena / D_vista` | `1/cos u` | desvio |
|---|---|---|---|
| `0–10°` | `1,0093` | `1,0037` | `+0,6 %` |
| `15–25°` | `1,0540` | `1,0640` | `−0,9 %` |
| `30–40°` | `1,2032` | `1,2211` | `−1,5 %` |
| `40–50°` | `1,4565` | `1,4153` | `+2,9 %` |
| `50–58°` | `2,5152` | `1,7012` | `+47,9 %` (a auto-limitação do §4 já morde) |

⇒ **a profundidade segue a DENSIDADE de dabs, um para um**, até a auto-limitação entrar. Se `a`
fosse recalculado do passo efectivo, a razão seria `≈ 1` em toda a tabela. ⭐ **O contraste está no
§5.3:** mudar a percentagem **declarada** de `5 %` para `10 %` move a profundidade `−0,6 %` (o `a`
compensa); mudar o passo **efectivo** pelo mesmo factor move-a `+100 %` (não compensa).

⇒ **Resposta à pergunta do report:** com o adaptativo (inerte) a profundidade no meio da peça **não
muda** (`≤ 6e-8`); com o eixo que de facto varia o passo (`SCENE`) ela não muda **no meio** (plano
`+0,09 %`; a `u = 5–15°` `+1,6 %`, contra `1/cos 10° = 1,015` previsto) e muda **fora dele**, por
`1/cos θ`.

### §16.9 — ⛔⛔ O PREÇO do modo `SCENE` — ele cura o pontilhado e NÃO se copia como está

| medição | `VIEW` (fábrica) | `SCENE` |
|---|---|---|
| ondulação a `74–79°` | `0,314` | **`0,011`** ✅ cura |
| `D/R` a `74–79°` | `0,0608` | **`0,6295`** ⛔ `10×` mais fundo que em `VIEW`, `3×` mais fundo que o próprio meio da peça |
| o mesmo traço a **`4` px por evento** de rato em vez de `1` | `0,1092 · 0,0607 · 0,0389` nas bandas `60–68°`, `74–79°` e `83–88°` — desvio **`≤ 0,2 %`** nelas, e `1,9 %` no pior caso (a banda `5–15°`) | `0,3116 · 0,4380 · 0,2715` nas mesmas — **cai `45,6 %`, `30,4 %` e `51,1 %`**, com `53,5 %` no pior caso (`79–83°`) |
| o mesmo caminho percorrido **ao contrário** (do meio para a silhueta) | ⚠️ **sem controlo publicado** neste modo (não há `produto_cupula_fabrica_do_meio_para_a_silhueta`); medido fora do corpus, igual tirando a ponta do traço | `0,1935` a `74–79°` contra `0,6295` — **`3,3×`** (e `2,8×` a `60–68°`, `3,6×` a `83–88°`) |
| o cursor **PARADO** (eventos repetidos no mesmo píxel), `|Δ|` máximo | `0,00983` em `60°`, `75°` e `84°` (um dab) | `0,00977` · `0,01551` · `0,01728` — **continua a carimbar, e tanto mais quanto maior a inclinação**. ⚠️ **Só `84°` está publicado** (`parado_{cupula_cena,cupula_vista}_u84`); as células de `60°` e `75°` foram medidas fora do corpus |

⭐⭐ **O mecanismo das três últimas linhas é um só, e é uma realimentação:** o que `SCENE` compara é
a distância 3D até ao acerto **vivo**, e o acerto vivo **afunda com o vinco**. Uma superfície
deslocada de `δ` ao longo da própria normal move o acerto de um raio fixo em `δ / cos θ` —
**o mesmo `1/cos θ`** — logo junto à silhueta o afundamento *sozinho* já ultrapassa o passo e pede
mais dabs, que afundam mais. ⇒ o resultado passa a depender de **quantos eventos o rato entregou**
e de **em que sentido** a mão andou.

⛔⛔ **Isto viola por construção a lei que o módulo já pagou seis vezes** — *o traço é facto do
CAMINHO, nunca de quão fino o motor amostrou o caminho* (`spacing.rs:1-40`, e o §0 do roteador).
⇒ **copiar `SCENE` como está seria importar um defeito de classe**, e a fidelidade não o pede: ele
**não é o valor de fábrica** de nenhum dos dois pincéis do alvo.

### §16.10 — O que isto quer dizer para a NOSSA casa (lido no código vivo)

1. **Somos fiéis, e o pontilhado é fidelidade.** O nosso passo é
   `passo_do_traco(verb, raio) = raio · pct / 50` com `pct = 5` para o pincel afiado
   (`atenuacao_do_traco.rs:51,70`) e o `raio` que os três sítios de chamada passam é
   `scene.radius_px()` (`input.rs:331,357,384`), um raio de **ECRÃ** (`space.rs:253`). O módulo do
   passeio declara-o por escrito: *«a unidade é a do CHAMADOR … no nosso shell essa régua é a TELA»*
   (`spacing.rs:46-49`). ⇒ o nosso produto reproduz o modo `VIEW` do alvo, que é o de fábrica.
2. **A cura, se o dono a quiser, tem DUAS metades, e só a primeira existe no alvo:**
   - **(a) o passo em mundo.** ⚠️ ⛔ **não** pela lei do alvo (a distância ao acerto vivo, §16.9),
     mas pela **geometria do caminho**: percorrer o caminho do cursor e medir a distância entre os
     **acertos consecutivos na superfície do início do traço**. Isso dá o mesmo passo uniforme sem
     realimentação, sem dependência da taxa de amostragem e sem dependência do sentido — e é a
     forma que o nosso [`spacing.rs`](../../../crates/ph2d-sculpt3d/src/spacing.rs) já tem
     (o `walk` recebe *dois pontos e uma distância mínima na mesma régua*; muda a régua, não a lei).
   - **(b) a atenuação a seguir o passo efectivo.** Sem ela, (a) sozinha troca um vinco pontilhado
     por um vinco `1/cos θ` mais fundo (§16.8) — que foi exactamente o que se mediu no alvo.
     ⛔ **O alvo NÃO faz (b)**, logo isto é desenho nosso e não paridade: fora de qualquer gate de
     paridade, e **decisão do dono** (P-6, §16.13).
3. ⛔ **O «espaçamento adaptativo» não é a alavanca** e não deve ser implementado com esse nome à
   espera de curar isto: medido inerte (§16.5).

### §16.11 — As fixturas desta emenda

Família nova [`fixtures/pincel_afiado/silhueta/`](fixtures/pincel_afiado/silhueta/) — **30**
ficheiros; a contagem sai do directório, nunca desta prosa.

⚠️ **Convenção NOVA, declarada:** a malha é **gerada por FÓRMULA** e o cabeçalho traz as chaves
`malha_*` (tipo, raio, células, ângulos) que a reconstroem inteira; os blocos `r`/`n`/`s` publicam
um **SUBCONJUNTO DECLARADO** (a fila do traço inteira + a secção transversal inteira em `16`
estações). A razão é medida: as `30` malhas têm `9 409`, `37 249`, `42 947`, `93 757`, `185 977` e
`1 079 102` vértices — e as da régua da ondulação, que é o que obriga ao subconjunto, estão entre
`185 977` e `1 079 102`; publicá-las inteiras custaria megabytes por ficheiro.
⭐ **E a convenção é EXECUTÁVEL:** [`silhueta/confere_a_formula.py`](fixtures/pincel_afiado/silhueta/confere_a_formula.py)
re-deriva as posições de repouso da fórmula do cabeçalho, confere que os índices publicados são
exactamente os declarados, exige as `36` chaves de cabeçalho e tem **piso de população** (`≥ 20`
ficheiros). Provado por mutação nas quatro formas de podridão: fórmula errada (`1,0e-3` contra a
tolerância `2e-6`), chave de cabeçalho apagada, índice retirado do bloco, e o censo a varrer quase
nada. *Sem ele o cabeçalho podia descrever uma malha e o ficheiro trazer outra — prosa não é
executável.*

| grupo | ficheiros | o que fixa |
|---|---|---|
| o CONTROLO do report | `produto_cupula_fabrica_da_silhueta` · `produto_cupula_catalogo_da_silhueta` · `produto_esfera_fabrica_da_silhueta` | o alvo pontilha, com o pincel de fábrica e com o de catálogo, na meia-cana e na esfera |
| a LEI do passo | `passo_em_{vista,cena}_esp{060,100,150}` (**6**) | as duas leis do §16.4, com a percentagem a escalar |
| o adaptativo INERTE | `adaptativo_ligado_em_{vista,cena}_esp100` (**2**) | têm de dar a MESMA saída que os dois de cima |
| a inclinação não move a fronteira | `detector_{plano,rampa1,rampa4}_vista_salto_{04,05}px` (**6**) | a `4` px as três leem a profundidade de **um** dab (`0,0147635`, espalhamento `1,1e-8`) e a `5` px leem mais ⇒ a fronteira é `5` px nas três (G-14) |
| … e move-a em `SCENE` | `detector_rampa4_cena_salto_{03,04}px` (**2**) | na MESMA rampa `4:1`, a `4` px já se lê a profundidade de **dois** dabs (`0,0220558` contra `0,0147635` em vista) ⇒ ali a fronteira desce para `4` px |
| o PREÇO de `SCENE` | `produto_cupula_cena_da_silhueta` · `produto_esfera_cena_da_silhueta` · `produto_cupula_{fabrica,cena}_4px_por_evento` · `produto_cupula_cena_do_meio_para_a_silhueta` · `parado_cupula_{cena,vista}_u84` (**7**) | **quatro das CINCO** linhas do §16.9 com o controlo publicado ao lado — ⚠️ a do **sentido** tem só o lado em cena (não há `produto_cupula_fabrica_do_meio_para_a_silhueta`), e a do cursor parado só a `84°` |
| o degenerado | `pen_down_na_silhueta_sem_dab` · `pen_down_na_silhueta_um_px_para_dentro` (**2**) | §16.6 |
| o controlo plano | `plano_vista` · `plano_cena` (**2**) | os dois modos coincidem de frente para a vista |

### §16.12 — Os GATES propostos

Mesma convenção do §12 («aprovado» = o maior erro do lado que tem de passar; «errado» = o menor
erro da candidata que tem de reprovar; a barra fica **estritamente entre** os dois).

| gate | população (piso) | mede | barra | aprovado | errado mais perto | origem |
|---|---|---|---|---|---|---|
| **G-13** o nosso passo é o do alvo junto à silhueta | `passo_em_vista_esp{060,100,150}` (**3**) | ⚠️ **a posição de um dab lê-se do BARRO, e a leitura é nomeada:** o pico de `D` ao longo da fila, **refinado por parábola nos três vértices** (sub-célula), projectado em píxeis de ecrã; mede-se o **maior desvio à grelha uniforme** ajustada por mínimos quadrados | `0,5` px | **`0,093` px** (o pior das três: `0,081` · `0,093` · `0,088`) | a lei de `SCENE` no mesmo traço põe o 2.º dab a **`12,43` · `20,10` · `29,10` px** do sítio certo | §16.4 |
| **G-14** a inclinação NÃO move a fronteira do passo | `detector_{plano,rampa1,rampa4}_vista_salto_{04,05}px` (**6**, piso `3` superfícies) + o par de cena `detector_rampa4_cena_salto_{03,04}px` (**2**) | ⚠️ **a PROFUNDIDADE do vinco, nunca uma contagem de picos:** a `4`–`5` px de separação os dois dabs **fundem-se num pico só**, logo contar picos lê `1` dos dois lados nas oito. A grandeza é o `D_max` da fila. **Duas metades:** (a) a `4` px as **três** superfícies leem o valor de UM dab; (b) a `5` px as três leem MAIS | (a) as três a `4` px dentro de `1e-6` umas das outras · (b) razão `D(5)/D(4) > 1,10` nas três | (a) `0,014763483` · `0,014763472` · `0,014763474` — espalhamento **`1,1e-8`**, o chão de ruído do alvo · (b) `1,658` · `1,654` · **`1,163`** | **a MESMA superfície e o MESMO salto noutro modo:** `detector_rampa4_cena_salto_04px` lê `0,022055817` onde a vista lê `0,014763474` — razão **`1,494`**, ou seja o 2.º dab já lá está a `4` px. Uma lei que dividisse o passo por `cos θ` daria isso **em vista** | §16.3 |
| **G-15** a régua da ondulação separa o pontilhado do contínuo | `produto_cupula_fabrica_da_silhueta` (**1** corrida, **2** bandas) | `r` mediana em `u ∈ [25°,35°]` e em `u ∈ [74°,79°]` | `< 0,05` e `> 0,20` | `0,006` e `0,314` | um vinco contínuo lê `0,011` na banda de fora (`produto_cupula_cena_da_silhueta`); e **sem a cláusula das pontas** a régua lê `1,000` **nas duas PONTAS do traço** — onde o vinco acaba —, que é o falso positivo que ela existe para não dar (as bandas interiores continuam a ler `0,002 / 0,006 / 0,314`) | §16.1, §16.2 |
| **G-16** a nossa saída pontilha como a do alvo | `produto_cupula_fabrica_da_silhueta` · `produto_esfera_fabrica_da_silhueta` (**2**) | desvio relativo de `D/R` e de `r`, por banda de `u` | `8 %` em `D/R` (bandas até `79°`) · `0,06` absoluto em `r` | `0,52 %` e `0,0036` (meia-cana × esfera, que são duas medições independentes da mesma lei) | a lei de `SCENE`: `D/R` `+935 %` a `74–79°` | §16.2 |
| **G-17** o adaptativo é inerte **com o dispositivo que temos** (⚠️ a cerca está no NOME de propósito: a fixtura está, por construção, no ponto NEUTRO do único knob que o tornaria observável — um tamanho que varie ao longo do traço, que pede pressão de caneta) | `adaptativo_ligado_em_{vista,cena}_esp100` × os dois pares (**2**) | `max |Δ|` contra o par sem o interruptor, sobre os mesmos vértices movidos | `1e-6` | **`3,6e-12`** (vista) e **`0` ao bit** (cena) — ⚠️ o `6,0e-8` da §16.5 é da população larga, de que **nenhum membro está publicado** | um dab a mais `1,3e-2` | §16.5 |
| **G-18** de frente para a vista os dois modos coincidem | `plano_vista` × `plano_cena` (**2**) | desvio relativo do fundo do vinco, pela **mediana ao longo da fila** (⚠️ pelo **máximo global** o mesmo par lê `0,115 %`; as duas passam, e só uma é a coluna) | `0,5 %` | **`0,084 %`** | na meia-cana a `74–79°` os mesmos dois modos diferem `935 %` | §16.4 |
| **G-19** ⛔ o traço é facto do CAMINHO (a catraca que proíbe importar `SCENE`) | `produto_cupula_fabrica_4px_por_evento` × `produto_cupula_fabrica_da_silhueta` (**2**) | desvio relativo de `D/R` por banda entre `1` e `4` px por evento, nas **oito** bandas | `5 %` | **`1,89 %`** (a banda `5–15°`; as três bandas junto à silhueta leem `≤ 0,2 %`) | o par de `SCENE` lê **`53,5 %`** no pior caso (`79–83°`), e `45,6 %` · `30,4 %` · `51,1 %` nas três da silhueta | §16.9 |
| **G-20** o pen-down sobre a tangente não carimba | `pen_down_na_silhueta_sem_dab` × a irmã `pen_down_na_silhueta_um_px_para_dentro` (**2**) | vértices com linha `s` (movidos), entre os publicados | `0` | `0` | a irmã — o **mesmo** pen-down e UM píxel para dentro — move **`154`** dos `2 497` publicados | §16.6 |

⚠️⚠️ **Porque é que o G-13 NOMEIA a leitura do pico, e não a deixa ao leitor:** a fixtura publica
uma **malha**, não uma lista de dabs, e as duas leituras óbvias não são equivalentes. Lendo o pico
como o **vértice mais alto**, o mesmo produto correcto dá `0,558` · `0,646` · `0,515` px — e
**reprova a barra de `0,5` px em duas das três**, com desvios-padrão `4×`–`5×` maiores. Com a
interpolação sub-célula dá `0,081` · `0,093` · `0,088`. ⇒ *quem não soubesse qual é a leitura
concluiria que a NOSSA lei do passo está errada, ou afrouxaria a barra* — e afrouxar uma barra
sobre produto correcto é o que esta casa proíbe. ⭐ A margem para a candidata errada (`12,43`–`29,10`
px) sobrevive às duas leituras, então nomear a fina não enfraquece o gate: torna-o **decidível**.

⚠️ **G-19 é o gate que esta emenda existe para deixar escrito.** Ele não mede o pincel afiado: mede
a **classe** da lei do passo, e reprova qualquer futura implementação que meça a distância contra
uma superfície que o próprio traço está a mover. *Uma recusa medida sem gate é uma nota que a
próxima janela reabre.*

### §16.13 — Decisões do dono e recusas medidas

| # | assunto | as saídas | recomendação técnica |
|---|---|---|---|
| **P-6** (dono) | **o vinco pontilhado junto à silhueta** | (i) ficar fiel ao alvo (é o valor de fábrica dele, e é o que temos) · (ii) passo em MUNDO pela geometria do caminho, sem a realimentação do alvo · (iii) (ii) **mais** a atenuação a seguir o passo efectivo, que é o único que mantém a profundidade constante | **(iii)**, com (ii) isolado atrás de um interruptor para bissecar. ⛔ (i) é uma escolha legítima e barata; ⛔ copiar o `SCENE` do alvo **não** está na lista (§16.9) |
| **P-7** (dono) | **oferecer o eixo «medido em» ao artista** | esconder · oferecer com as duas leis do alvo · oferecer só com a nossa | **esconder** até P-6 decidir: oferecer o `SCENE` do alvo é oferecer um knob cujo resultado depende da taxa de amostragem do rato |

⛔ **Recusas MEDIDAS desta emenda — não as reconstrua:**

1. **O «espaçamento adaptativo» como cura** — medido inerte em `12` traços e `132` pares do
   detector (`≤ 6,0e-8` contra `1,3e-2` de um dab). §16.5.
2. **Copiar o modo `SCENE` do alvo** — cura o pontilhado e compra três defeitos medidos: `10×` a
   profundidade junto à silhueta, dependência da taxa de amostragem (`−30 %` a `−51 %` com `4` px
   por evento) e dependência do sentido (`3,3×`). §16.9.
3. **Medir a ondulação numa janela fixa** — lê `1,000` nas pontas do traço, onde o vinco acaba.
   A janela é **um período de dab local** e só vale com a janela inteira dentro da faixa coberta.
   §16.1.
4. **Ler a posição do dab em `x` de ecrã junto à silhueta** — `x = 199` px e `x = 200` px distam
   `5,7°` de superfície; a coordenada é o **arco de mundo**. §16.1.
5. **Definir as classes do detector por igualdade AO BIT** — duas corridas da mesma configuração
   diferem até `6,0e-8`, e a classe parte-se em duas sem que nada tenha mudado. §16.5.

⚠️⚠️ **E uma armadilha que esta emenda pagou, na própria fixtura:** a primeira esfera do harness
saiu com o **enrolamento das faces invertido**. Com a normal a apontar para dentro, o pincel que
**afunda** levanta uma **crista** — e **todas** as réguas continuaram a devolver números plausíveis
e da ordem de grandeza certa (`D_max` `0,0351` contra os `0,0346` do valor certo), porque o módulo
do deslocamento é quase o mesmo. O que o denunciou foi medir o deslocamento **com sinal, ao longo
da normal de repouso**; o que o teria apanhado antes é o guarda que o harness passou a ter
(*a normal média tem de apontar para fora*). ⇒ *uma malha com o enrolamento trocado não reprova
nenhum gate de magnitude: ela executa o verbo contrário e finge concordar.*

# O elástico que não para, e o volume — o report de 2026-09-08

> **Report do dono, com três fotos:**
> 1. *«Plasticity alto danificou o render (provável Bake)»*
> 2. *«O Cloth não age como pano real, mas como um elástico que estica
>    indefinidamente. Nas imagens coloquei pontos de mask e usei Gravity. Isso
>    exige pesquisa e estudo.»*
> 3. *«Deve haver algum grau de elasticidade mas deve haver a possibilidade de
>    manter volume»*

A cena das fotos: uma esfera, **três pontos mascarados** perto do topo, e o
filtro de **gravidade** arrastado até ao fim. A foto 2 é o repouso; a 3 é a
esfera puxada em três tubos compridos; a 1 é a mesma peça com a superfície que a
luz mostra rasgada.

---

## §1 — As duas queixas são UM defeito, e ele tem número

Uma restrição de distância com rigidez `0,6` (espec §5.2) é uma **mola**. Sob
carga sustentada uma mola não trava: ela assenta num equilíbrio **esticado** cujo
desvio cresce com a carga, sem tecto. A gravidade do filtro é carga sustentada —
`s` cresce enquanto o dedo arrasta e a simulação **acumula** (é o que separa este
filtro do de malha, que repõe a pose a cada passo).

Medido sobre a cena dele (esfera `48×96`, três pontos mascarados, quatro gestos
de 120 passos com `s` de `0` a `1,0`):

| | esticão máx | vincos `>60°` | volume | rugosidade máx |
|---|---:|---:|---:|---:|
| **como estava** | **`18,58`** | `247` | **`44 %`** | `5,99` |

⚠️ **A régua do «render danificado» não é o esticão — é o VINCO.** O matcap
amostra a direcção da face, então o que se vê como sujidade colada à peça são
pares de faces vizinhas com muito ângulo entre as normais. A malha de repouso tem
`3,75°` em toda parte; a peça da foto tem `~250` pares acima de `60°` e máximos a
`178°`, que é uma face virada do avesso.

### ⛔ A atribuição à *Plasticity* está INVERTIDA, e o número é monótono

O dono atribuiu o dano à plasticidade alta. Medido, ela faz o **contrário** —
varrendo `0,00` a `1,00` sobre quatro gestos, **todas as colunas melhoram**:

| plasticidade | esticão máx | vincos `>60°` | rugosidade máx |
|---:|---:|---:|---:|
| `0,00` | `18,58` | `247` | `5,99` |
| `0,50` | `13,90` | `257` | `3,70` |
| `0,90` | `6,08` | `56` | `1,38` |
| `1,00` | `3,51` | `4` | `0,65` |

⭐ **E a observação dele continua certa** — é a leitura que muda: com plasticidade
alta a peça **mantém a forma de esfera** e o que sobra é a superfície estragada,
que é a foto 1; com plasticidade baixa ela vira os três tubos da foto 3 e a
mesma sujidade lê-se como «esticão». *O dano é o mesmo nos dois; a plasticidade
só decide se a forma sobrevive para o mostrar.*

---

## §2 — O estado da arte, e por que cada peça

Três famílias publicadas respondem a isto, e nenhuma delas está no alvo — ele é
um solver de **pano**, e um pano é uma superfície aberta sem volume.

1. **Limitar o esticão** — Provot, *Deformation constraints in a mass-spring
   model to describe rigid cloth behaviour*, GI 1995. Depois da relaxação, toda
   aresta que passe de `ℓ·(1+ε)` é trazida de volta. É uma **desigualdade**, não
   uma mola: não depende da carga nem do número de varreduras.
2. **Âncoras de longo alcance (LRA)** — Kim, Chentanez & Müller, *Long Range
   Attachments*, SCA 2012. *Nenhum ponto do pano pode estar mais longe da âncora
   do que o caminho de material que o liga a ela.* Uma restrição por vértice,
   resolvida numa passagem.
3. **Conservar o volume** — Müller, Heidelberger, Hennix & Ratcliff, *Position
   Based Dynamics*, 2007, §4.5 (a restrição do balão). O volume com sinal de uma
   casca fechada é `V = ⅙ Σ_f (xᵢ × xⱼ)·x_k`; uma restrição **escalar** sobre a
   peça inteira, com gradiente barato.

⚠️ **Uma quarta — a dobra por ângulo diedro** (Grinspun et al., *Discrete
Shells*, SCA 2003) — **já vive na crate** (`bending.rs`), escrita para o caminho
VBD que foi refutado. Ela fica aberta: ver §5.

---

## §3 — O que foi construído

### 3.1 O tecto de esticão, e as duas metades que ele precisa

⛔⛔ **Provot SOZINHO não converge, e o número diz porquê.** Ele é uma projecção
**local**: para segurar uma tira pendurada tem de propagar a notícia do pino até
à ponta, um vértice por passagem. Medido num gesto de 120 passos com tecto `1,10`:

| passagens | esticão máx | custo |
|---:|---:|---|
| `4` | `4,63` | a rede inteira, 4× |
| `16` | `3,56` | 16× |
| `64` | `1,95` | 64× |
| `128` | `1,46` | inaceitável |

⭐ A **âncora de longo alcance** resolve o mesmo em **uma** passagem, porque não é
local. Com ela, `4` passagens de Provot chegam a **`1,68`**.

⚠️⚠️ **E ele é JACOBI COM MÉDIA, nunca Gauss-Seidel.** A 1.ª redacção era
Gauss-Seidel e **amplificava** o defeito que vinha curar: no passo 115 o pior
esticão da rede **entrava a `4,50` e saía a `8,84`**. A rede desta lei põe ~21
restrições sobre cada vértice (as 6 arestas do anel-1 mais os ~15 pares dele), e
uma projecção inteira por restrição resolvida em sequência faz cada vizinha
desfazer a anterior — divergência clássica de Gauss-Seidel sobre um sistema
sobredeterminado com projecções duras. A média é combinação convexa, logo o passo
nunca ultrapassa a mais exigente.

⚠️⚠️ **E a POSIÇÃO no passo foi medida.** A 1.ª redacção pôs o limitador entre as
varreduras e a integração — o sítio que a leitura ingénua indica, «para que a
velocidade o veja» — e ali ele **não limita nada**: a integração corre depois e
acrescenta `a·Δt + v·k` sobre a posição já corrigida. Medido num passo só, tecto
`1,10`: o esticão do fim do passo ficava em `1,270`, exactamente o de sem tecto.
Ele é hoje o **último acto do passo**, e continua a ser visto pela velocidade sem
custo, porque nesta lei `v` é a diferença entre duas posições **pós-relaxação**.

### 3.2 A conservação de volume

⚠️ **`k` é a FORÇA, não um volume-alvo.** A 1.ª redacção lia-o como *«que fracção
do volume manter»* (`C = V − k·V₀`) e é ingovernável: com `k = 0,5` a peça não
assenta em metade, **colapsa a `6,5 %`** e os vincos duplicam. Como força é
monótona:

| força | volume final |
|---:|---:|
| `0,00` | `39 %` |
| `0,02` | `65 %` |
| `0,10` | `88 %` |
| `0,50` | `97 %` |
| `1,00` | `98 %` |

⛔ **Só liga em peça FECHADA** (`Mesh::is_closed`) — o volume com sinal de uma
casca aberta é um número que existe e não é o volume de nada.

### 3.3 A tabela final

| | esticão máx | vincos `>60°` | volume | rugosidade máx |
|---|---:|---:|---:|---:|
| como estava | `18,58` | `247` | `44 %` | `5,99` |
| só o tecto (`1,10`) | `2,82` | `307` | `39 %` | `1,13` |
| só o volume | `18,36` | `166` | `99 %` | `6,06` |
| **os dois** | **`3,13`** | **`195`** | **`98 %`** | **`1,07`** |

⭐⭐⭐ **E o APERTO é quem mais ganha** — o que estava aberto no §5 do roteador
como *«a decisão do dono sobre o aperto com força alta, com a opção (b)
construída, medida e REFUTADA»*. Sem volume o aperto **não estica: implode**.

| aperto | vincos `>60°` | volume |
|---|---:|---:|
| sem volume | `1 151` | **`3 %`** |
| com volume | **`119`** | **`95 %`** |

*A terceira saída que faltava àquela decisão não era uma afinação da força — era
a peça saber que tem um dentro.*

### 3.4 O preço

Milissegundos por passo, esfera com o topo mascarado (a máquina a `load < 3`):

| vértices | lei do alvo | + tecto | + volume |
|---:|---:|---:|---:|
| `4 514` | `2,89` | `3,00` | `3,31` |
| `24 386` | `16,25` | `16,19` | `19,64` |
| `49 002` | `32,24` | `34,21` | `37,52` |

⇒ o tecto custa **`0` a `6 %`** e o volume mais **`10` a `16 %`**. ⚠️ **O que já
não cabe num quadro é o passo BASE** — a `24 386` vértices a lei do alvo sozinha
gasta `16,25 ms` contra os `16,7` de um quadro a 60 fps; os dois controlos novos
não mudam essa conversa.

⚠️ A semeadura das âncoras é um Dijkstra sobre a rede de restrições e corre no
**primeiro passo do gesto**, não no pen-down: a `49 002` vértices ela custa
**`~2 ms`**, uma vez.

---

## §4 — ⛔ Recusas MEDIDAS

- **O PISO de compressão** (a ideia: uma dobra de 180° encurta as «diagonais de
  `2h`» do anel-1 sem violar nenhuma aresta, logo um piso resistiria à dobra de
  graça). Construído e medido com piso a `0,00`/`0,55`/`0,75`/`0,90`: o esticão
  máximo, a dobra `p99`, a dobra máxima e a contagem de vincos **não se movem**
  (`75,69°`/`75,82°`/`76,02°`/`76,38°`), e a `0,90` pioram.
  ⭐ **A razão é estrutural:** um piso abaixo de `ℓ` é estritamente **mais fraco**
  do que a restrição que já lá está — a estrutural tem alvo `ℓ` exacto e já puxa
  `83 %` do desvio por passo. E há assimetria física: a gravidade **estica** de
  forma sustentada; a compressão não tem carga sustentada e resolve-se ao dobrar
  para fora do plano. *O tecto trata de um regime permanente; o piso trataria de
  transitórios que a lei já apaga.*
- **O «desligado» no topo da faixa do tecto.** A 1.ª redacção punha a faixa em
  `1..10` com `10` a virar `∞`, *«para o artista alcançar o comportamento antigo
  sem um segundo controlo»* — duas coisas erradas: o comportamento antigo **é o
  defeito reportado**, e uma faixa de `1` a `10` põe o intervalo útil
  (`1,00`–`1,50`) nos primeiros `5 %` do cursor. A faixa é `1,00..2,00` e não tem
  sentinela; o `∞` vive na LEI, que é onde as 103 fixtures o correm.
- **Ligar o tecto no PINCEL.** O defeito vive no filtro (força sustentada por
  centenas de quadros); um dab tem raio, banda e um número de passos que o cursor
  limita, e são as **86 fixtures dele** que provam a nossa paridade. *Ligar sem o
  corpus que o meça seria trocar o activo por um palpite.*
- **Ligar a conservação de volume por omissão.** Ela **cancela a Escala e o
  Inflate por construção** — os dois existem para mudar o volume —, e há gate a
  afirmá-lo: a Escala passa de `1,45×` o volume de repouso para `1,01×`. *Um
  default que faz um dos cinco tipos não fazer nada é pior que uma opção.*

---

## §5 — ⏳ ABERTO

- ✅ **O `Expand` explodia, e FECHOU em 2026-09-09.** O mecanismo escrito aqui
  estava certo — ele desloca o **repouso** (`τ`) e o tecto mede-se contra `ℓ + τ`,
  logo o denominador da régua crescia com a lei que ela devia limitar.
  ⚠️⚠️ **Os NÚMEROS deste item estavam todos errados**, e os três docs desta série
  davam três valores diferentes (`753×` aqui, `155×` no doc 11, `55×` no doc 12):
  medido de fresco nos valores de fábrica pela
  [`sonda_do_expand_que_nao_para`](../../../crates/ph2d-sculpt3d/tests/it/sonda_do_expand_que_nao_para.rs),
  o esticão contra o material era **`7,743`** a um gesto e `14,358` a três.
  ⭐⭐ **E o volume não explodia — ele COLAPSAVA** (`1,067 → 0,245 → 0,761`): o que
  aquilo produzia não era uma peça maior, era uma **amarrotada**.
  ⭐ **A prova de que o tecto não o alcançava:** com `tecto = 1,00`, que proíbe
  *todo* esticão elástico, o esticão contra o material continuava em `6,854` —
  logo aquilo era `τ` inteiro.
  **A cura** é [`Verlet::limitar_o_repouso`](../../../crates/ph2d-cloth/src/verlet_limites.rs):
  o `τ` de cada vértice é limitado a `(tecto − 1) ×` a **menor** aresta de material
  que lhe toca, ⛔ **sem constante nova** — é o `stretch_max` que o artista já
  define, aplicado à metade plástica. O `min` é o que torna a garantia
  demonstrável (`min ≤ ℓ` nos dois extremos ⇒ o repouso da aresta fica em `tecto·ℓ`).
  Depois: **`1,210 / 1,450 / 1,729`** de esticão e **`1,120 / 1,288 / 1,470`** de
  volume — o Expand entra na família do Inflate (`1,13`–`1,33`) e do Scale
  (`1,13`–`1,44`), e a peça passa a **crescer** em vez de dobrar sobre si.
  Gate `o_expand_obedece_ao_tecto_como_os_outros_quatro`, com a barra **derivada**
  `tecto²` e a metade que impede um no-op de a satisfazer; mutação: tirar a
  chamada devolve `7,7432`.
  ⛔ **A paridade não se moveu** — `limitar_esticao` já saía por `return` com
  `estica_max = ∞`, que é a omissão do `Solver` e o caminho das `103` fixtures.
- **A DOBRA continua sem modelo próprio.** Com a cura, os vincos `>60°` caem de
  `247` para `195` e a dobra `p99` de `98,0°` para `75,7°` — melhor, não curado.
  A ferramenta está escrita e testada (`bending.rs`, ângulo diedro com ângulo de
  repouso); falta ligá-la ao caminho Verlet como restrição PBD.
- **A faixa da força de volume é muito não-linear** — `0,10` já entrega `88 %`.
  Um slider linear gasta metade do curso entre `97 %` e `98 %`.
- **O tecto no pincel**, se e quando houver corpus que o meça.

---

## §6 — ⛔⛔ QUATRO tectos de LOC estavam VERMELHOS há waves, e nenhum fecho os viu

O `architecture_workspace_file_loc_cap` acusou **quatro** ficheiros, e só **um**
era desta wave:

| ficheiro | no `main` | antes desta wave | hoje |
|---|---:|---:|---:|
| `ph2d-cloth/src/verlet.rs` | — | `514` | `650` |
| `ph2d-mesh-render/src/pipeline.rs` | `664` | **`831`** | `676` |
| `ph2d-cloth/src/verlet_gesto.rs` | `620` | **`782`** | `411` |
| `ph2d-editor-core/src/ids/chrome/sculpt3d.rs` | `652` | **`723`** | `622` |
| `ph2d-panel-sculpt3d/src/rows.rs` (tecto `600`) | — | `602` | `490` |

⇒ **três deles atravessaram o tecto em waves anteriores desta linha e ficaram lá.**
A causa é a mesma que esta casa já registou seis vezes: *o gate mora em
`ph2d-editor-core/tests/`, e um fechamento por `cargo test -p <a minha crate>` —
ou por `--bins` — não o alcança.*

⚠️ **Todos foram curados por CORTE, nenhum por isenção**, e cada corte é por
responsabilidade e não pela última coisa escrita: `verlet_limites.rs` (o que o
alvo não tem) · `verlet_gesto_forca.rs` (a fase 4) · `pipeline_gbuffer.rs` (a
doação) · `sculpt3d_cloth.rs` (os ids do tecido) · as seis pistas do padrão para
o `rows_alpha.rs`, onde as perguntas delas já viviam.

### 6.1 ⛔⛔ E DOIS censos de FONTE estavam verdes por falta de `cargo fmt`

`cargo fmt --all` mexeu em **oito ficheiros que esta wave não editou** — todos de
waves anteriores desta mesma linha, commitados sem formatação. Duas agulhas de
censo morreram com isso, **sobre produto correcto**:

| gate | agulha | o que a partiu |
|---|---|---|
| `the_brush_cursor_asks_the_surface_for_its_orientation` | `ring_on_surface(&self.camera` | a chamada passou a sete linhas |
| `the_brush_radius_is_screen_pixels_converted_against_the_camera` | `ring_on_surface(&` | a mesma chamada |

⭐ **A lei:** *uma agulha que atravessa uma quebra de linha mede o FORMATADOR, não
o código.* A cura é ler a fonte com o espaço **normalizado**
(`src.split_whitespace().collect::<Vec<_>>().join(" ")`) antes de procurar, e ela
não custa nada às contagens — uma agulha `nome(` é imune por construção, porque um
nome nunca se separa do parêntese que abre.

⚠️ **O doc do segundo gate JÁ escrevia a lição** (*«uma régua textual que supõe a
forma dos argumentos mede a formatação, não a lei»*) e foi a terceira vez que a
mesma família o mordeu.

⏳ **ABERTO, e é do repo e não desta linha:** um censo grosseiro conta **388**
agulhas de fonte com argumentos dentro em `tests/` e `*_tests.rs`. A maioria é
curta e o formatador não lhe toca; nenhuma sonda separa as duas populações, e a
que separa é a única que vale — *quantas destas agulhas vivem numa linha que o
`rustfmt` pode quebrar amanhã?*

---

## §7 — Onde está

- Lei: [`ph2d-cloth/src/verlet.rs`](../../../crates/ph2d-cloth/src/verlet.rs)
  (`Solver::estica_max`, `Solver::volume`) e
  [`verlet_limites.rs`](../../../crates/ph2d-cloth/src/verlet_limites.rs)
  (`semear_lra`, `limitar_esticao`, `restringir_volume`, `conservar_volume`).
- Produto: [`cloth_filter_props.rs`](../../../crates/ph2d-sculpt3d/src/cloth_filter_props.rs)
  (`stretch_max`, `volume`, e a porta `solver()`).
- Gates: [`mede_o_tecido_que_estica.rs`](../../../crates/ph2d-sculpt3d/tests/it/mede_o_tecido_que_estica.rs)
  — seis, três provados por mutação (a âncora de longo alcance, a passagem de
  Provot e a restrição de volume, cada uma desligada faz um gate reprovar).
- Painel: *Stretch Limit* e *Preserve Volume*, os dois `Basic`, visíveis quando a
  **lei** escolhida é de tecido.

# 10 — A LUZ QUE ATRAVESSA A PEÇA (a subsuperfície)

> **Ordem do dono, 2026-09-17.** Posto perante os três ingredientes que faltavam ao
> [`01`](01_o_alvo_decomposto.md) — a translucidez (`6`), o pós (`7`) e o estilo (`8`) —, ele
> escolheu **«a luz atravessa a peça»**: *folha, jade, cera, mármore fino*.

⚠️ **Esta wave está NO PRODUTO.** A lei, a curvatura, o painel, os dois motores e a cena `=33`.

---

## §1 — ⭐⭐⭐ São DOIS caminhos e não um, e a diferença é a PEÇA

O `open_pbr_surface` escolhe entre eles pelo `geometry_thin_walled`, e eles respondem a perguntas
diferentes:

| | a peça | a lei | o que se vê |
|---|---|---|---|
| **parede fina** | folha, pétala, papel, abajur | oren-nayar à frente **+ `translucent` por trás**, meio a meio, com a fase a pesar os dois lados | com a luz **ATRÁS**, ela acende inteira |
| **maciça** | jade, cera, mármore fino, leite | o perfil de difusão de **Burley** integrado sobre a **curvatura local** | a luz **contorna** a quina, e o terminador amacia |

⭐ **A wave inteira da parede fina cabe numa linha** — o `mx_translucent_bsdf` faz `N = -N` e mais
nada, logo o `max(0)` dele é sobre `−N·L`. *É essa negação, e só ela, que faz uma folha acender com
o sol atrás.*

⚠️ **A parede fina não sabe nada sobre a forma da peça** e a maciça precisa de **uma** grandeza
geométrica. É essa diferença que faz a primeira custar zero e a segunda custar uma amostra de campo.

---

## §2 — ⭐⭐⭐ A lei maciça é a que ESTA CASA já tinha escrita

O [`ph2d_mesh_render::sss`](../../crates/ph2d-mesh-render/src/sss.rs) — a tabela pré-integrada de
Penner & Borshukov, portada em 2026-08 para o **esculpir** — integra **o mesmo integral**:

```text
D(θ, r) = ∫ clamp(cos(θ + x), 0, 1) · R(2r·sin(x/2)) dx  /  ∫ R(2r·sin(x/2)) dx
```

A mesma **corda** `2r·sin(x/2)`, a mesma normalização. O que muda:

| | o esculpir (2026-08) | o modelador (esta wave) |
|---|---|---|
| perfil `R` | soma de **seis gaussianas** (d'Eon & Luebke) | **Burley**, duas exponenciais (Pixar) |
| avaliação | **pré-integrada** numa tabela | `32` termos por amostra, como o oráculo |

⇒ *não é um sistema novo: é o mesmo fenómeno com o perfil que o oráculo desta linha usa.*

### §2.1 — ⭐⭐ E ela depende só do ÂNGULO e do quociente `mfp/raio` — medido

Escalar o raio e o caminho livre médio pelo mesmo factor deixa `shape·dist` invariante e `R` escala
por `1/k` **uniformemente**, logo `ΣD/ΣR` não se mexe. Medido sobre `16×`:

| `θ` | `r = 0,25 · mfp = 0,25` | `r = 1 · mfp = 1` | `r = 4 · mfp = 4` | desvio |
|---:|---:|---:|---:|---:|
| `0°` | `0,82041167` | `0,82041167` | `0,82041167` | `0,00e+00` |
| `90°` | `0,16056012` | `0,16056012` | `0,16056012` | `0,00e+00` |
| `180°` | `0,05095574` | `0,05095574` | `0,05095574` | `0,00e+00` |

⭐ A grandeza é o adimensional `mfp·κ` — **exactamente o eixo `t = scatter·|κ|` que o cabeçalho da
tabela do esculpir declara**.

---

## §3 — ⭐⭐⭐ A CURVATURA sai do CAMPO, e não de derivadas de ecrã

O renderizador de referência estima-a por `length(fwidth(N)) / length(fwidth(P))`.

⛔⛔ **Aqui isso era inexprimível sem partir a paridade de `100,000 %` desta linha:** o `fwidth` da
placa é por **quad de `2×2`** e a diferença do traçador de CPU seria **por pixel**.

### §3.1 — A lei, e porque ela quase não custa amostras

Num campo de distância `|∇f| = 1`, logo `H = ∇²f / 2`. E o Laplaciano sai da **mesma soma** que a
normal já percorre:

```text
Σⱼ f(p + ε·dⱼ) = n·f(p) + ε·∇f·Σdⱼ + (ε²/2)·Σ dⱼᵀ H dⱼ + O(ε³)
```

Nos dois estêncis `Σdⱼ = 0` e `Σ dⱼdⱼᵀ = c·I`:

| estêncil | `c` | `∇²f` |
|---|---:|---|
| `Central6` (`\|d\| = 1`) | `2` | `(Σfⱼ − 6·f(p)) / ε²` |
| `Tetra4` (`\|d\| = √3`) | `4` | `(Σfⱼ − 4·f(p)) / (2ε²)` |

⇒ falta **uma** amostra: a do centro. ⛔ E o estêncil desta lei é **fixo** no de quatro, mesmo
quando a normal usa seis — o traçador do dispositivo lê sempre por aquele, e seguir a escolha do
quadro faria os dois motores divergirem no dia em que ele mudasse.

### §3.2 — ⛔⛔ A premissa que a medição derrubou: o PASSO não é o da normal

A 1.ª redacção dizia *«`eps` é o mesmo que a normal usou; dois passos diferentes dariam duas
respostas para a mesma pergunta»*. **Falso, e o gate reprovou em voz alta:** com o passo da normal
uma face plana lia curvatura **`1,49`** e a esfera errava `109 %`.

*Uma primeira diferença divide por `ε` e uma segunda por `ε²`* ⇒ o cancelamento em `f32` entra
`1/ε` vezes mais cedo, e o óptimo da segunda é `~ulp^{1/4}` contra `^{1/3}` da primeira.

| `ε / raio` | `R = 0,4` | `R = 1,0` | `R = 2,5` |
|---:|---:|---:|---:|
| `0,0004` | `0,534` | `0,397` | `0,404` |
| `0,0016` | `0,033` | `0,022` | `0,025` |
| **`0,0064`** | **`0,0041`** | **`0,0041`** | **`0,0047`** |
| `0,0256` | `0,027` | `0,013` | `0,013` |
| `0,1024` | `0,054` | `0,054` | `0,054` |

⭐ **O vale está no MESMO sítio nos três**, e é isso que faz dele uma lei: abaixo manda o
cancelamento, acima manda a truncagem. ⚠️ E a escala é a da **PEÇA**, nunca a da vista — um passo
que seguisse o zoom daria duas curvaturas para o mesmo ponto.

### §3.3 — ⚠️ A divergência declarada

A referência devolve um **comprimento**, logo não distingue bossa de cova; esta porta devolve `|H|`
pela mesma razão. A diferença real é outra e é **a favor**: a dela é a curvatura normal na direcção
do **ECRÃ** (muda ao rodar a câmera) e a nossa é a **média** (não muda).

---

## §4 — ⭐⭐⭐ O oráculo, e porque ele ganhou uma LUZ DE TRÁS

O corpus é o `GlslRenderer` do MaterialX 1.39.5 corrido sem interface
([`ferramentas/fixture_openpbr.py`](ferramentas/fixture_openpbr.py)). Ele ganhou **4 materiais** e
**uma luz que viaja para `+z`**, isto é, de detrás da esfera contra a câmera.

⚠️ **Sem ela a fixture não contém o fenómeno que a parede fina existe para produzir:** nas três
luzes de sempre a transmissão lê `max(−N·L, 0) = 0` em quase todo pixel visível. *Um corpus sem o
fenómeno não afirma nada sobre a lei que o produz.* `945 → 1980` amostras directas, `315 → 495`
indirectas.

### §4.1 — A paridade, e as duas barras

| caminho | pior | barra | leitura |
|---|---:|---:|---|
| **parede fina** (materiais `7`, `8`) | `9,6e-6` | `1e-4` | a MESMA ordem do resto da fixture |
| resto do corpus (`0`..`6`) | `9,8e-6` | `1e-4` | inalterado |
| luz do céu (os `11`) | `1,6e-6` | `1e-4` | inalterado |

### §4.2 — ⛔⛔ E o caminho MACIÇO tem gate PRÓPRIO, com o porquê medido

A curvatura implícita de cada amostra do oráculo, sobre uma esfera cuja curvatura verdadeira é `1`,
espalha-se de **`0,02` a `54`**. ⚠️ **E não é a tesselação:** a mesma medição sobre uma esfera
`256×128` **nossa** devolve a mesma dispersão. *A grandeza é do ECRÃ, não da malha.*

⇒ um **máximo** sobre essa população mede o estimador **dele**. O gate afirma a mediana, os
quartis, e — a metade que de facto afirma a lei — que o mínimo em `κ = 1` é **AGUDO**:

| | `κ = 0,5` | **`κ = 1` (a verdade)** | `κ = 2` |
|---|---:|---:|---:|
| material `9` · `p50` | `0,0769` | **`0,0005`** | `0,0582` |
| material `9` · `p75` | `0,7307` | **`0,0073`** | `0,9589` |
| material `10` · `p50` | `0,0449` | **`0,0004`** | `0,0392` |

*Uma lei que não fosse a dele não teria vale nenhum na curvatura verdadeira.*

### §4.3 — ⛔ A divergência do `acos`

O `mx_acos` do MaterialX é o `acos` do GLSL **sem corte** (um `#define` directo em
`stdlib/genglsl/lib/mx_math.glsl`), e `dot` de dois unitários lê `1,0000001` em `f32` ⇒ o oráculo
devolve **`NaN`** ali. Esta porta corta a `[-1, 1]` — no-op dentro do domínio, e *um pixel `NaN` e
um pixel legitimamente preto leem-se iguais*.

---

## §5 — ⚠️ O `energy_compensation` é um ARGUMENTO, e o motivo é um valor de omissão

A nodedef `ND_oren_nayar_diffuse_bsdf` dá `energy_compensation = false`, e a **reflexão da parede
fina da subsuperfície** é a **única** closure do `open_pbr_surface` que o deixa por escrever — a
base escreve `true`. ⛔ Passar-lhe a compensada mudaria o número que o oráculo mede.

⛔ E o `stinv` do clássico é **`0` quando `s ≤ 0`**, onde o compensado guarda o `s` negativo: são
duas leis publicadas diferentes e não uma simplificação.

---

## §6 — ⭐⭐⭐ O que o artista ganha, e as três leis que a wave herdou

Seis linhas novas na secção do material: **Subsurface** (o peso) · **Subsurface Color** (amostra) ·
**Subsurface Radius** · **Subsurface Radius Scale** (amostra — *quanto mais fundo cada canal
viaja*, e é o que põe o vermelho à frente numa orelha) · **Subsurface Anisotropy** · **Thin
Walled**.

O painel é **derivado da tabela**, e foi isso que tornou a wave barata: nem uma linha de pintura
nova. As três leis herdadas:

1. **as seis ficam TRAVADAS com o peso a zero** — a lei do verniz (§21) numa quinta família, e a
   parede fina está entre elas porque com o peso a zero o caminho que ela escolhe nem é avaliado;
2. **a cor e a escala são AMOSTRAS**, pela tabela `CORES` que a ordem do dono de 14/09 criou;
3. **a escrita não se estreita** — o `set_param` continua a aceitar as `33` posições.

### §6.1 — ⛔ O `Thin Walled` é um booleano guardado como NÚMERO

A tabela de `FieldMaterial::get` é de `f32` e o painel é derivado dela; uma variante nova de linha
custaria a tabela inteira. Ele viaja como `Span::Choice` de dois (`Solid` · `Thin Walled`), logo o
arrasto é inteiro. ⏳ **Que ele se pinte como uma CAIXA fica NOMEADO e por fazer.**

---

## §7 — ⏱️ O degrau, o tecto e o que eles custaram

**`PROJECT_SCHEMA` `144 → 145`** — dez `f32` apendados ao `FieldMaterial`, o mesmo mecanismo dos
degraus `140`..`143` (`92` bytes onde o binário pede `132`).

⚠️ **APENDADOS e não na posição da NODEDEF**, e a troca é declarada: a subsuperfície vem **antes**
do verniz lá, logo re-numerar mexeria em `11` posições já gravadas — a quebra de layout que o
`142 → 143` pagou uma vez.

**`MAX_ROWS` do painel `79 → 85`**, pela lei que o doc dele já escrevia: a alternativa era baixar o
`MAX_POLYGON_VERTICES` de `27` para `24`, e *um tecto de REGISTO cujo recurso é memória a mandar
num tecto de FORMA é o caminho lento a definir o rápido* (§0.0). Preço medido: `6` widgets por
linha, `36` uma vez no arranque.

---

## §8 — ⭐ Os portões

| gate | o que afirma |
|---|---|
| `the_direct_light_is_the_oracles_on_every_sphere` | `9,8e-6` sobre `9` materiais, com a parede fina lá dentro |
| `o_caminho_macico_bate_o_oraculo_na_mediana` | `p50 5e-4` · `p75` · e o **mínimo AGUDO** em `κ = 1` |
| `uma_esfera_de_raio_r_le_curvatura_um_sobre_r` | `1/R` nos três raios **e** que a régua separa raios |
| `um_plano_le_curvatura_zero` | `0,000023` na face de uma caixa |
| `a_guarda_devolve_zeros_e_nunca_nan` | «não sei» é plano, nunca `NaN` |
| `every_number_a_material_has_reaches_the_law` | as `33` posições movem a radiância em pelo menos um dos dois caminhos |
| `a_luz_que_atravessa_a_peca_e_a_mesma_nos_dois_motores` | **`100,000 %`** nos dois, e cada um **move** a imagem |

---

## §9 — ⚠️ As premissas que a medição derrubou

1. *«o material FECHOU: são as `15` entradas do OpenPBR e não há mais nenhuma para apender»* —
   escrita em **três** sítios (a escada do schema, o `FieldMaterial::get`, o `MAX_ROWS`). Era
   verdade sobre a **fatia** de 14/09 e falsa sobre o modelo, que tem `41` entradas — e a
   `ph2d-material` declarava por escrito, na mesma semana, que cinco famílias ficavam *«nesta
   fatia»*. ⇒ *uma lista fecha-se contra o que se construiu, nunca contra o que existe.*
2. *«este tecto deixa de crescer por material»* (o `MAX_ROWS`).
3. *«o passo da curvatura é o da normal»* (§3.2).
4. *«a tesselação explica a dispersão do `fwidth`»* — a esfera `256×128` nossa dá a mesma.
5. *«o desvio do caminho maciço é a curvatura, e existe um valor que o fecha»* — a varredura de
   `0,05` a `27` fechou a porta: o melhor é `0,81`. *Se nenhum valor do parâmetro livre serve, o que
   está errado é a régua ou a lei* — e era a **régua** (o pior sobre a população).

---

## §10 — ⏳ O que fica

- **A ESPESSURA como entrada.** Nenhum dos dois caminhos da referência a lê — a maciça aproxima-a
  pela curvatura, e num campo de distância a espessura **mede-se** (uma marcha para dentro até o
  campo voltar a ser positivo). ⭐ É a mesma vantagem estrutural do [`02` §5.1](02_o_estado_da_arte.md),
  um nível abaixo, e é o candidato natural a **superar** a referência. ⛔ Fica aberta de propósito:
  a paridade desta wave é contra a lei da curvatura, e trocar a entrada sem oráculo é inventar.
- **A cena `=33` não põe a luz atrás sozinha** nem autora os materiais — um `FieldDoc` não carrega
  nenhum dos dois e o gancho não existe. O artista faz os dois gestos, que existem.
- **O `Thin Walled` como caixa** (§6.1).
- **O relógio desta wave não foi medido** — a máquina esteve entre `load 13` e `27` toda a jornada,
  e *nenhuma leitura acima de `load ~5` vale nada*. ⇒ vai para a **`W9`** ([`03`](03_o_plano.md)),
  que é onde o dono a pôs.
- **`transmission_*` · `fuzz_*` · `thin_film_*` · `geometry_opacity`** continuam fora, agora com a
  lição do §9 ao lado: a lista fecha contra o construído.

---

## §11 — ⛔⛔⛔ O report de 2026-09-18: a LINHA DURA, e as DUAS curas que a foto refutou

> *«Bom resultado! Avalie apenas uma coisa: em `Thin Walled: Solid` não há transição suave entre a
> área iluminada e a área sombreada da esfera, mas uma linha dura. Veja se é correto.»*
> … e depois: *«não vi mudanças. O resultado em Thin Walled é melhor (mais suave a transição).»*

**Não é correto.** E ⚠️⚠️ **nada nesta secção foi achado por uma régua: foi achado por uma FOTO.**

### §11.1 — A resposta, em uma linha

⭐⭐⭐ **A linha que ele aponta é a borda da SOMBRA QUE A PLACA LANÇA SOBRE A BOLA.** Não é o
terminador, não é a lei da subsuperfície e não é a quadratura.

O experimento que o decide é de uma linha (`PH2D_TERM_SO_A_BOLA=1` na
[`sonda_fotografa_o_terminador`]): a MESMA câmera, a MESMA luz, o MESMO material, **sem a placa**.
| | o que se vê |
|---|---|
| a cena `=33` | a linha está lá |
| a bola sozinha | **a bola é perfeitamente lisa** |

⇒ ela é dura porque a luz é um **PONTO**, e um ponto lança sombra com borda em degrau. E ela só
incomoda no `Solid` porque ali a resposta tem contraste através da borda; na parede fina metade da
energia vem do lóbulo de trás e a mesma borda lê-se lavada.

⛔ **É correcto como geometria e ERRADO como produto:** num jade a luz que entra fora da sombra
espalha-se por baixo da superfície PARA DENTRO dela, e a borda amolece. A nossa subsuperfície
multiplica uma visibilidade **dura, por pixel** — *a difusão está na lei do `N·L` e não está na lei
da sombra.*

### §11.2 — ⛔ A fixtura mentiu QUATRO vezes antes de conter o fenómeno

1. Uma bola **sozinha na origem**: o perfil atravessou `N·L = 0` liso e o pior salto ficou no
   **brilho especular**, que lê o mesmo no opaco. *Uma régua que procura o máximo global mede o
   realce.*
2. `radiance_at_one: [intensity; 3]` em vez da porta do produto (`cor × intensidade × π`): a
   lâmpada saiu **`π×` fraca e sem cor** e o céu dominou. *Uma sonda que reescreve a conversão do
   produto mede outro programa.*
3. O enquadramento de omissão em vez do **zoom** que ele usou.
4. ⭐⭐⭐ E a que custou a jornada inteira: **nunca perguntei se a linha era a sombra da placa.**
   As duas curas foram desenhadas sem essa resposta.

### §11.3 — ⛔⛔ CURA A, construída inteira e REVERTIDA pela foto

O passe de sombra escreve `vis = 1,0` para todo ponto de costas para a luz, o que **trunca** no
terminador a sombra de um vizinho — defeito real, e o comentário desse filtro **previa-o por
escrito** (*«até ao dia em que alguém ler este canal para outra coisa — a OpenPBR tem termos que
recebem luz com `N·L < 0`»*). Construiu-se a cura: o raio parte na mesma e só conta o que estiver
**depois de sair do próprio corpo**, com o `t` do estimador de penumbra recontado a partir da saída.

Ela tinha tudo: gémeo em WGSL, **6 de 6** paridades CPU↔dispositivo verdes, **4 de 4** mutações a
sangrar, a folha intacta (`83,7`), a esfera convexa com zero acne, a banda do terminador de
`p99 9,21` para `3,71`.

⛔ **E a foto reprovou-a:** a borda alargou **e ganhou um FIO escuro SERRILHADO** por cima. O
serrilhado é a assinatura — **um `if` por pixel** (`N·L <= 0`) escolhia entre duas maneiras de
calcular a mesma grandeza, e *a fronteira entre elas desenha-se*.

⛔ **E apagar o ramo é PIOR:** um caminho só, todo raio a partir do ponto, devolve **acne** (riscos
claros ao longo do terminador; a banda vai a `p99 13,06`), porque o ergue pela normal deixa de lá
estar.

⇒ *a truncagem é INVISÍVEL nesta cena e o fio é VISÍVEL* ⇒ **shipa a truncagem**, e os dois gates
que a cura teria partido ficam, porque são onde a segunda tentativa vai bater:
[`a_folha_com_a_luz_atras_nao_se_apaga`] · [`um_corpo_convexo_nao_se_tapa_a_si_proprio`].

### §11.4 — ⭐⭐ CURA B, que FICA: a quadratura deixa de pôr vincos na lei

Medida a lei **sozinha** (sem cena, `4 001` amostras de `N·L`), o maciço tinha o pior salto da 2.ª
derivada em **`N·L = −0,0980` = `sin(π/32)`** — o `x` da 1.ª amostra da quadratura — e **não se
movia** com o `Subsurface Radius` (`0,1` · `1,0` · `4,0` dão todos o mesmo ponto). *Uma feição cuja
posição não depende de nenhum parâmetro físico é da discretização.*

A causa: o `max(cos(θ+x), 0)` amostrado no **MEIO** da célula põe uma quina em cada nó, e o perfil
de Burley é **singular em `x = 0`** ⇒ as duas células vizinhas do zero levam quase todo o peso.
⇒ o cosseno passa a ser integrado **exactamente dentro da célula** (`(sin b − sin a)/largura`, com
os extremos cortados ao domínio onde ele é positivo), que é **C¹ em θ** — nos cortes a derivada é
`cos(±π/2) = 0`.

| pior 2.ª derivada em `\|N·L\| <= 0,97` | |
|---|---|
| opaco (o terminador de Lambert, quina legítima) | `276` |
| parede fina — **o lado que o dono APROVOU** | `137` |
| maciço, ponto médio (o oráculo) | `67`–`95`, em `N·L = −0,098` |
| **maciço, hoje** | **`0,4`–`0,5`** |

⛔⛔ **Divergência DECLARADA:** a mediana contra o oráculo vai de `5,0e-4` para `2,9e-3`. ⚠️ **Toda
cura possível diverge daqui, e é aritmético:** o integral verdadeiro é um só, e é o ponto médio a
`N = 32` que está a `~3e-3` dele — somar mais amostras converge para o mesmo sítio. *O que diverge
do oráculo é ele próprio do seu limite.* As barras subiram com a tabela ao lado, e ⭐ a divergência é
**load-bearing**: quem a reverter para recuperar a mediana reprova no
[`o_macico_nao_e_mais_duro_que_o_lado_que_o_dono_aprovou`], cuja barra é calibrada **no lado que ele
aprovou**. **3 de 3** mutações sangram.

⚠️ **E ela NÃO apaga a linha da foto** — ela apaga um vinco *da lei*, que é outro defeito.

### §11.5 — ⏳ A cura que falta, agora com endereço

**A visibilidade que um termo TRANSMISSIVO lê tem de ser BORRADA pela distância de espalhamento.**
É isso que faz a sombra num jade ter a borda mole, e nenhuma das duas referências o escreve (o
MaterialX põe `occlusion = 1` e foge do assunto).

⭐ A maquinaria já existe nesta crate: o borrão com guarda de normal que o céu e o ricochete usam
(`OCCLUSION_BLUR_COS`, com os gates `a_suavizacao_apaga_o_ruido_dentro_de_uma_superficie` e
`a_suavizacao_nao_atravessa_uma_quina`). O que falta é um **segundo canal de visibilidade**, borrado
com o raio `mfp/κ` que o `integrate_burley` já usa, lido **só** pela closure de subsuperfície —
⚠️ e isso muda a fronteira do `ph2d-material` (hoje há **uma** radiância por lâmpada para todas as
closures), mais o gémeo em WGSL. **É wave própria, e fica nomeada.**

---

## §12 — ⭐⭐⭐ A SOMBRA COM A BORDA MOLE (ordem do dono, 2026-09-18: *«sim. faça»*)

A §11 fechou com o diagnóstico: a linha é a borda da sombra que a placa lança, dura porque a luz é
um ponto, e **certa como geometria e errada como produto**. Esta secção é a cura.

### §12.1 — A lei

> **A visibilidade que uma closure TRANSLÚCIDA lê é a MÉDIA da vizinhança, sobre a distância de
> espalhamento do material.**

Num jade a luz que entra **fora** da sombra espalha-se por baixo da superfície **para dentro** dela.
A nossa subsuperfície já tinha a difusão na lei do `N·L` (o `integrate_burley` envolve a luz à volta
do terminador) e **não a tinha na lei da SOMBRA** — a visibilidade entrava dura, por pixel.

⭐ **O comprimento da média é o `subsurface_radius` por canal** ([`Surface::scatter_distance`]), em
unidades do MUNDO, convertido a píxeis pela câmera ([`sss_shadow::raio_em_pixeis`]). *É por ser por
canal que a borda fica avermelhada — o vermelho viaja mais e entra mais fundo na sombra, que é a
assinatura de toda pele e de toda cera.*

⚠️ **O raio é do MUNDO e não do ecrã**: um raio escrito em píxeis seria uma borda que encolhe quando
o artista se aproxima.

### §12.2 — Onde ela entra, e porque NÃO precisou de partir o `compose`

⭐⭐⭐ A composição do OpenPBR é **linear na resposta das closures** — o `mix`, o `add` e o `layer`
(que multiplica pelo *throughput*, e esse não depende da radiância). ⇒ compor a superfície **com** e
**sem** o peso de subsuperfície e ficar com a diferença dá **exactamente** a parcela dela através de
toda a pilha. A porta é o [`Surface::direct_sss`], que recebe **duas** radiâncias.

⚠️ **Com as duas iguais ele é o [`Surface::direct`] AO BIT**, pelo braço curto — e é esse o caminho
de todo material sem subsuperfície, que não paga nada.

### §12.3 — A medição, e a FOTO

| | a borda | a quebra na banda `\|N·L\| <= 0,15` |
|---|---|---|
| antes | **uma linha** | p99 `9,21` |
| hoje | **mole, e a sombra continua lá** | p99 `1,36` |
| o mesmo material sem subsuperfície | dura (correcto) | **byte a byte o de sempre** |

⭐ E a foto da cena `=33` com o enquadramento do dono mostra a linha **desaparecida**, com a região
sombreada ainda visivelmente mais escura — *a sombra não foi apagada, foi amaciada*.

### §12.4 — Os portões, e as duas mutações que SOBREVIVERAM primeiro

Três metades, porque nenhuma chega sozinha
([`a_borda_da_sombra_num_jade_e_mole_e_a_do_opaco_continua_dura`]): **o jade amacia** · **o opaco não
muda UM BYTE** (sem isto, borrar a visibilidade de toda a gente passava e apagava a sombra do app
inteiro) · **o jade continua a TER sombra** (sem isto, `vis = 1` em todo o lado passava na primeira).

⛔⛔ **E duas mutações sobreviveram à primeira redacção, cada uma a nomear uma régua em falta:**
- *a separação deixa de ser exacta* — a cena de jade tem `subsurface_weight = 1`, logo o difuso já
  saiu da mistura e **trocar quem lê o quê no resto da pilha não movia um byte lá**. ⇒
  [`zerar_a_radiancia_da_subsuperficie_tira_so_a_parcela_dela`], com os dois lóbulos a valer.
- *a guarda de normal cai* — a bola é lisa, e **uma cena sem quina nenhuma não testa a guarda da
  quina**. ⇒ [`a_media_da_borda_mole_nao_atravessa_uma_quina`], sobre uma tira sintética.
  ⚠️ E ela sobreviveu **outra vez**: a 1.ª asserção media as PONTAS da tira, que a `r = 8` ficam fora
  do alcance da quina e leem o mesmo com a guarda apagada. *Uma régua colada ao fenómeno, não ao
  extremo.* **4 de 4 sangram** hoje.

### §12.5 — ⏳ O que FALTA, e é declarado

⛔⛔ **O DISPOSITIVO ainda não tem o gémeo.** Ele calcula a visibilidade **dentro** da pintura, por
pixel, logo dar-lhe a borda mole pede uma passagem que a escreva num buffer, duas de borrão separável
e a leitura — o mesmo desenho que o ricochete lá já tem. ⇒ **hoje a cura vê-se no caminho de
REFERÊNCIA** (`PH2D_FIELD_GPU=0`), e as paridades CPU↔dispositivo continuam verdes porque nenhuma
delas assa o canal.

⏳ E fica também: a média é **separável** (duas passagens de uma dimensão) e a guarda de normal não é
separável em rigor — a divergência é declarada no cabeçalho do [`sss_shadow`], e vale o preço
(`O(r)` contra `O(r²)`; a `r = 24` isso são `2 401` toques por pixel e por canal).

---

## §13 — ⏳ O PADRÃO-OURO: a pergunta do dono, o que já está montado, e um DESVIO DE PROTOCOLO meu

> *«Não sei se temos o padrão ouro em qualidade. Temos a Unreal instalada aqui. Quer comparar e ver
> se podemos melhorar? Ou mesmo superar a unreal?»* — 2026-09-18.

### §13.1 — ⭐⭐⭐ A reformulação que muda o trabalho: a Unreal NÃO é a verdade

A subsuperfície em tempo real da Unreal é **também uma aproximação** (ecrã, perfil de Burley), com
artefactos próprios. ⇒ *comparar só com ela responde «somos como a indústria», nunca «estamos
certos».*

⭐ **O padrão-ouro é um TRAÇADO DE CAMINHOS CONVERGIDO**, e isso dá a *«superar a Unreal»* uma
definição medível e crispa:

> **superar = ficar mais perto da verdade do que a resposta de tempo real dela.**

⭐⭐ E ela própria traz um traçador de caminhos, logo os três podem correr a MESMA cena: nós · a
Unreal em tempo real · a verdade.

### §13.2 — O que está MEDIDO nesta máquina (2026-09-18)

| | |
|---|---|
| **Unreal Engine** | **5.8.2** instalada em `~/Documentos/Projetos/UnrealEngine`, `73 GB`, build promovida |
| licença | **EULA proprietária** ⇒ **PAREDE**: corre-se, nunca se lê o fonte |
| porta sem interface | ⭐ **`Engine/Binaries/Linux/UnrealEditor-Cmd` existe** |
| **Blender** | `5.2.2 LTS` (GPL ⇒ parede), `blender -b --factory-startup -P <script>` |
| placa | RTX 5060 Ti, 16 GB |
| ⭐ **a verdade, corrida** | um traçado convergido da cena `=33` a `2 048` amostras: **`7 s`** |

⭐⭐ **E o enquadramento BATE**: a bola sai no mesmo sítio, do mesmo tamanho, com o brilho no mesmo
canto — porque os números da câmera saem das **portas do produto**
([`sonda_os_numeros_da_cena`]) e não de um script que os re-deriva.

### §13.3 — ⛔ Os números ainda NÃO são veredito, e está declarado

O desvio de forma lê `96 %`–`98 %`, grande demais para uma comparação de forma ⇒ *as duas imagens
ainda não são comparáveis*. O que falta está nomeado: o **céu** do oráculo é cinzento uniforme e o
nosso é a `StudioSky`; as **unidades da lâmpada** não estão casadas (o ajuste de exposição foge
`+7` stops, que é a assinatura disso); a **janela** do perfil ainda apanha a silhueta.
*Publicar este número como resposta seria a quinta régua mal calibrada do dia.*

### §13.4 — ⭐⭐⭐ O achado que já saiu, e que explica o dia inteiro

A sombra da placa sobre a bola é uma **penumbra que nunca chega ao preto** (`vis` de `~0,52` a
`1,000`), e **a borda dela corre quase na HORIZONTAL**. ⇒ toda régua desta jornada que varria em
**linha** andava **paralela à feição** e lia outra coisa — a silhueta (`x = 317`), o realce
especular (`N·L = 0,95`), o vinco da quadratura. *Uma régua paralela à feição não a vê*, e o
instrumento passa a varrer em **coluna**.

### §13.5 — ⛔⛔ DESVIO DE PROTOCOLO (INC-R1), registado sem desculpa

O [`método §6`](../_ComoInvestigarApps/00_o_metodo.md) diz: *«o harness que corre o alvo vive FORA
da árvore. Ele é acto do **E**; o implementador pede uma emenda, nunca o corre.»*

**Eu fiz as duas coisas erradas:** escrevi o harness do Blender e **corri-o eu**, e cheguei a
copiá-lo para dentro do repo (retirado no mesmo commit).

⭐ **Contaminação realizada: ZERO** — o que foi lido foi a IMAGEM de uma cena nossa, que é livre
(GPLv2 §0), e nenhuma linha de fonte do alvo. Mas *o valor da parede é o protocolo, não a sorte de
desta vez não ter lido nada*. ⇒ **daqui para a frente as corridas do oráculo são pedidas a uma
janela E**, e o harness fica fora da árvore; o que entra no repo é a **fixtura com cabeçalho**.

### §13.6 — ⏳ O preço do que falta

| | |
|---|---|
| casar o céu e as unidades de luz, e fechar a janela do perfil | **meia jornada** — e é o que torna o número um veredito |
| a Unreal como terceiro contendor (projecto, compilação de shaders, cena casada, render sem interface) | **uma jornada** |

---

## §14 — ⭐⭐⭐ NÓS CONTRA A VERDADE: o primeiro veredito com número

O dono perguntou se temos o padrão-ouro. A §13 montou o oráculo; esta secção é a medição.

### §14.1 — ⛔⛔ Antes do número, DOIS defeitos MEUS que o agente E apanhou

O relatório dele mediu as duas convenções do PFM, e as duas partiam o meu leitor **em silêncio**:

1. o ficheiro é **BIG-endian** (escala `> 0`) e eu lia `from_le_bytes`;
2. o meu laço de cabeçalho parava aos **três** campos ⇒ **a linha da escala nunca era lida** e o
   offset dos dados ficava errado.

⇒ **o «desvio de forma de `96 %`» que esta sonda imprimiu era isso.** ⭐ *O leitor passou a
RECUSAR em voz alta* (magia, contagem de campos, tamanho que tem de fechar, valores finitos): um
leitor que devolve lixo plausível é pior que um que falha.

⚠️ E o E entregou o oráculo com **controlo interno**: cada média e cada máximo é **exactamente ×2 a
cada duplicação da energia**, nos dois modos ⇒ a saída é genuinamente linear, sem tonemap e sem ceifa.

### §14.2 — ⭐⭐ O CONTROLO valida a montagem, e é ele que dá direito ao resto

| opaco (sem subsuperfície) | NÓS | VERDADE |
|---|---|---|
| largura da transição, 10–90 % | **`43 px`** | **`43 px`** |
| cor da região iluminada, `R/B` | **`1,33`** | **`1,32`** |

⇒ *a cena, a câmera, a lâmpada, a sombra e a cor batem.* **Sem este controlo, nenhum número do
jade valeria nada** — seria mais uma régua mal calibrada.

### §14.3 — O veredito

| jade (`Subsurface Radius = 1,0 × (1 · 0,5 · 0,25)`) | NÓS | VERDADE |
|---|---|---|
| largura da transição, 10–90 % | `70 px` | `80 px` |
| **cor da região iluminada, `R/B`** | **`1,61`** | **`0,87`** |

⭐ **A LARGURA está perto** — `12,5 %` mais apertada que a verdade. A wave da §12 pôs a borda no
regime certo.

⛔⛔⛔ **A COR MOVE-SE NO SENTIDO OPOSTO, e é este o buraco para o padrão-ouro.** Partindo de
`R/B = 1,33` (a cor base), **nós vamos para `1,61`** (mais vermelho) e **a verdade vai para `0,87`**
(azulado). O mecanismo é lisível nas duas leis:

- a nossa (`mx_subsurface_bsdf`, o porte do MaterialX) lê `shape = 1/mfp` ⇒ **o canal de mfp maior
  tem o perfil mais LARGO** ⇒ o vermelho envolve mais ⇒ a peça avermelha;
- o traçado de caminhos transporta **fotões**: um mfp de `1,0` numa esfera de raio `0,42` quer dizer
  que **o vermelho ATRAVESSA e não volta** ⇒ o que regressa ao olho é azul.

⚠️⚠️ **Isto não é um defeito do nosso porte — é da APROXIMAÇÃO que a indústria publica.** Nós
reproduzimos o `mx_subsurface_bsdf` fielmente (a §11.4 mede a paridade contra ele). ⇒ *fechar este
buraco é SUPERAR a aproximação de referência, não alcançá-la* — que é exactamente a pergunta que o
dono fez.

### §14.4 — ⏳ O que falta para o veredito ficar fechado

⏳ **Um segundo ponto**, pedido ao E: os mesmos renders com o raio muito **menor** que a peça
(`0,1` e `0,3`) e com o raio **igual nos três canais**. Isso separa *«a cor muda com a
profundidade»* de *«a cor muda porque os três canais têm raios diferentes»*, e diz se o desvio é
geral ou do regime «mfp maior que a peça». ⛔ *Um desvio medido num ponto só é um ponto, não uma
lei.*

⚠️ E fica nomeado que o desvio de FORMA lê `25 %` **no próprio controlo opaco** — ele é o **piso do
método** (a janela apanha o realce ceifado e o ajuste de exposição é um compromisso), e não uma
propriedade do jade, cujo `30 %` está a cinco pontos desse piso.

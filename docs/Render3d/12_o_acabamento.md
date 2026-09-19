# 12 — O ACABAMENTO (a `W7`): o brilho, o anti-serrilhado e a profundidade de campo

> **Ordem do dono, 2026-09-19:** *«8 e depois do smoke o 7»* — e o smoke da `W8` foi aprovado no
> mesmo dia. Este é o **último ingrediente** dos oito do [`01`](01_o_alvo_decomposto.md).
>
> ⚠️ **Este documento é o PLANO, escrito antes da primeira linha de produto** (`/pd-feature`). O que
> ele contém de medido está medido; o que ele propõe está marcado como proposta.

## §1 — ⭐ A triagem de licença, e ela pára na PRIMEIRA porta aberta

`CLAUDE.md` §0.9 manda a triagem primeiro, e **pelo ARTEFACTO instalado, nunca pelo nome do
projecto**. Medido em 2026-09-19, perguntando ao próprio binário:

```
$ godot --version
4.7.2.stable.arch_linux.ed1daf0bf
$ Engine.get_license_text()   # 21 linhas
Copyright (c) 2014-present Godot Engine contributors … Permission is hereby granted, free of
charge, to any person obtaining a copy … without restriction …
```

⇒ **MIT**. ⭐ **Porta aberta: porta-se, com atribuição — sem parede, sem subagentes E/R, sem
vassoura** (é o que o [arsenal](../_ComoInvestigarApps/01_o_arsenal.md) já declara para este alvo).
⚠️ E o binário **não vem do `pacman`** (`/usr/bin/godot`, instalado à mão), que é exactamente porque
a licença se leu **no artefacto** e não num gestor de pacotes.

## §2 — O que o modo Render tem hoje: NADA, e o pipeline fecha POR PIXEL

Medido contra o código (`grep` sobre `ph2d-field-render`, `ph2d-field-gpu`, `ph2d-view-transform`):
**zero** ocorrências de bloom, DOF ou AA temporal. O que existe chama-se `antialias` e é a média
geométrica de borda do marchador ([`edges.rs`](../../crates/ph2d-field-render/src/edges.rs)), que é
outra coisa.

⛔⛔ **E o achado que decide o preço da wave:** os dois motores fecham o quadro **pixel a pixel** —

```rust
// shade_render.rs, o fim da função
let cena = pres.style.apply(…);      // a camada de estilo (W8)
pres.look.apply(cena)                 // a exposição + a vista (W1)
…
px[0] = ph2d_color::srgb::linear_to_srgb_byte(c[0]);   // e vira BYTE aqui
```

*Nada guarda a imagem em HDR.* O brilho tem de ler a imagem **antes** da exposição e do byte, e
hoje esse valor existe durante uma expressão e morre. ⇒ **a primeira peça da wave não é um efeito,
é um buffer** — e ela não muda um pixel do que se vê.

## §3 — O ORÁCULO, corrido sem interface

### §3.1 A porta, medida

`--headless` do Godot força o driver de render **dummy** (⇒ sem brilho), e esta máquina **não tem
Xvfb**. A porta que funciona é a **sessão KWin virtual** que a casa já paga para fotografar cenas
([`fotografa_cena.sh`](../Components/ferramentas/fotografa_cena.sh)):

```
kwin_wayland --virtual --xwayland --exit-with-session <sessao.sh>
  └─ godot --path <proj> --display-driver x11 --rendering-driver vulkan \
           --write-movie <saida.png> --quit-after <n>
```

Medido: `Vulkan 1.4.351 — Forward+ — NVIDIA GeForce RTX 5060 Ti`, `0,03 ms/quadro` de GPU, PNG
gravado. ⚠️ **Uma sessão virtual serve N corridas** — a placa é cara de segurar (`CLAUDE.md` §2: a
regra ali é *exclusão + prazo*), então a varredura abre a sessão **uma vez** e corre as células
dentro dela.

### §3.2 A superfície de botões, despejada da API (`--doctool`)

`Environment` expõe **18** botões de brilho, com estes valores de fábrica:

| botão | fábrica | | botão | fábrica |
|---|---:|---|---|---:|
| `glow_enabled` | `false` | | `glow_hdr_threshold` | `1.0` |
| `glow_intensity` | `0.3` | | `glow_hdr_scale` | `2.0` |
| `glow_strength` | `1.0` | | `glow_hdr_luminance_cap` | `12.0` |
| `glow_mix` | `0.05` | | `glow_bloom` | `0.0` |
| `glow_blend_mode` | `1` (*Screen*) | | `glow_normalized` | `false` |
| `glow_levels/1..7` | `0 · 0.8 · 0.4 · 0.1 · 0 · 0 · 0` | | `glow_map_strength` | `0.8` |

Os cinco modos de mistura: `0` Additive · `1` **Screen** · `2` Softlight · `3` Replace · `4` Mix.

⚠️ O `--doctool` despeja a **superfície** (nomes, tipos, omissões, enums) e **não** a prosa — e isso
é o lado certo: §0.9 diz que o alvo se **corre**, e a prosa responderia *como* onde um gate precisa
de *o quê*.

### §3.3 ⭐ A LEI DO LIMIAR — medida, com controlo

Um quadrado não iluminado de cor `(i,i,i)` sobre fundo preto, tonemapper **Linear**, exposição `1`,
tudo o resto de fábrica. O halo é a soma em **linear** de tudo o que está **fora** do quadrado:

| entrada | halo (soma linear) | pico do halo | raio até `1/255` |
|---:|---:|---:|---:|
| `0,5` | **`0,00`** | `0,00000` | `0` |
| `0,75` | **`0,00`** | `0,00000` | `0` |
| `1,0` | **`0,00`** | `0,00000` | `0` |
| `1,5` | `1 408,43` | `0,29177` | `89` |
| `2,0` | `4 028,09` | `0,79910` | `98` |
| `4,0` | `9 856,80` | `1,00000` | `106` |
| `8,0` | `10 655,88` | `1,00000` | `107` |
| **`4,0` com o brilho DESLIGADO** (controlo) | **`0,00`** | `0,00000` | `0` |

**Três leis saem desta tabela:**

1. ⚠️⚠️ **A `1,0` o halo é ZERO, não «pouco»** — e a 1.ª redacção desta linha dizia *«o limiar é
   DURO e ele é exactamente o `glow_hdr_threshold`»*. **A atribuição estava errada** e a §3.5
   desmente-a: esta varredura correu com o limiar no valor de FÁBRICA, logo ela mede a ENTRADA e
   não o botão. *Uma varredura de uma variável com as outras no default mede a variável, nunca a
   causa.*
2. ⭐ **O RAIO quase não depende da intensidade** (`89 → 107 px` sobre `5,3×` de entrada): a
   extensão do halo é uma propriedade da **cadeia de níveis**, não do brilho.
3. ⛔ **E a minha régua SATURA:** o pico do halo lê `1,00000` a partir de `4,0` porque a saída é de
   **8 bits** — a coluna da soma continua a subir e a do pico já não diz nada. *Uma régua que mede
   a saída do produto herda o tecto da saída do produto*; a coluna honesta para a intensidade é o
   **raio**, e para a forma é o perfil longe do quadrado.

### §3.4 ⭐⭐ A GEOMETRIA DA CADEIA — um nível de cada vez

O mesmo quadrado a `4,0`, com **um** `glow_levels/k` a `1,0` e os outros seis a `0`:

| nível | raio até `1/255` | **raio a meia altura** |
|---:|---:|---:|
| 1 | `7` | `3` |
| 2 | `25` | `9` |
| 3 | `58` | `20` |
| 4 | `119` | `38` |
| 5 | `192` ⚠️ | `55` |
| 6 | `192` ⚠️ | `151` |
| 7 | `192` ⚠️ | `192` |

⭐⭐⭐ **O raio a meia altura DUPLICA por nível** (`3 · 9 · 20 · 38 · 55`), que é a assinatura de uma
**cadeia de mip**: cada nível desce a resolução a metade e borra com o mesmo núcleo, logo o borrão
em pixéis de ecrã dobra. ⚠️ **Os níveis `5`, `6` e `7` batem no bordo de um quadro de `512`** — a
`192` a medição é a moldura e não o nível —, e é por isso que os valores de fábrica dele acendem
**`2`, `3` e `4`** e deixam os outros a zero: *a fábrica é a faixa que cabe no ecrã*.

### §3.5 — ⛔⛔⛔ E o CORTE do alvo não é uma lei limpa: três botões que se pisam

Variando **o botão** em vez da entrada (quadrado a `4,0`, halo lido a `60 px` da borda):

| botão | valores varridos | o que o halo faz |
|---|---|---|
| `glow_hdr_threshold` | `0 · 0,25 · 0,5 · 1 · 2 · 3` | **`0,0160 → 0,0152`** — `5 %`, que é a quantização de 8 bits |
| `glow_hdr_scale` | `0,5 · 1 · 2 · 4` | **nada**: as quatro células lêem o mesmo |
| `glow_hdr_luminance_cap` | `1 · 3 · 12` | `0,100 · 0,289 · 1,000` — **o único que manda** |

E a `entrada 2,0`, onde um limiar teria mais força, ele move `27 %` sobre uma faixa de `0 → 3`:
`0,1095 · 0,1046 · 0,0953 · 0,0802`. ⛔ **Com limiar `3` e entrada `2` ainda brilha `0,0802`** —
um corte a sério daria **zero**. E o `glow_bloom` faz brilhar uma entrada de `0,5`, que está
**abaixo** de qualquer limiar (`0 · 0,00273 · 0,00518`, linear nele).

⇒ ⭐⭐⭐ **A GEOMETRIA dele é limpa e porta-se; o CORTE dele não é uma lei — é três botões que se
pisam, um deles quase inerte e um limiar que não gateia.** Portar esse comportamento seria herdar
um painel em que *«Threshold»* não faz o que o nome diz.

**Decisão:** portamos a **cadeia** (§3.4, medida e limpa) e **escrevemos o nosso corte, declarado** —
um limiar que de facto gateia, com o gate a prová-lo. É o mesmo movimento que o pincel de POSE fez
ao bater o modelo de referência da própria espec: *a referência responde o que outro programa FAZ;
ela não obriga a repetir o que ele faz MAL.*

## §4 — ⛔⛔ A DECISÃO DE ARQUITECTURA: o brilho é um PASSE, e isso já estava escrito

A [`11` §11](11_a_camada_de_estilo.md) recusou pôr o contorno desenhado na `ph2d-style` com este
mecanismo, palavra por palavra:

> ⛔ **O CONTORNO desenhado fica de fora:** ele lê os VIZINHOS no ecrã, logo é um **passe** e não uma
> multiplicação — pô-lo na crate da lei obrigaria-a a receber um G-buffer e ela deixaria de ser a
> lei que os dois motores partilham.

**O brilho lê os vizinhos.** ⇒ ele **não** entra na `ph2d-style` nem na `ph2d-view-transform`; ele é
uma crate-folha nova de PASSE, com a lei em Rust e o gémeo em WGSL, como todo o resto desta pilha.

⚠️ **E a ordem no quadro é load-bearing:** `sombreamento → ESTILO → [brilho] → exposição/vista →
byte`. O brilho lê **depois** do estilo (senão um contorno pintado não brilha) e **antes** da
exposição (senão ele mede bytes já comprimidos, que é o defeito que o `01` §2 nomeia como *«sem o
`1`, o bloom mente»*).

## §5 — As waves propostas

> ⚠️⚠️ **As `W7a` e `W7b` FUNDEM-SE, e a razão foi medida:** um buffer de HDR sem consumidor é
> **código morto** — a lei da casa é que *cada canal só é assado se o consumidor DELE estiver vivo*
> ([`11` §11](11_a_camada_de_estilo.md)). Elas shipam juntas, e a prova de que o substrato é inerte
> continua a ser a mesma: **com o brilho desligado, a imagem é a de hoje ao bit.**
>
> ⭐ E a assinatura **não** muda nos `50` chamadores: `shade_render` passa a **delegar** numa porta
> que devolve o quadro inteiro (o molde do `move_character`/`move_character_from` do TOP-20 #13).
> Dos `50`, **um** é produto — o resto são gates e sondas.

### `W7a` — O QUADRO EM HDR (substrato, zero mudança visível)

O sombreamento passa a escrever um `Vec<[f32; 3]>` linear, e a conversão para byte passa a ser um
**passo separado**. ⭐ A prova de que não mudou nada é **byte a byte**: a imagem de hoje e a de
depois têm de ser idênticas nos dois motores.

⚠️ No dispositivo isto é um alvo de render em `rgba16float` em vez de escrever `u32` — e o tecto
dele é **memória de GPU**, a medir (`1920×1080×8 B ≈ 16,6 MB`).

### `W7b` — O BRILHO (o efeito que o dono nomeou)

A crate-folha nova (proposta: `ph2d-bloom`, zero dependências, o molde da `ph2d-style`), com a
cadeia de mip, o limiar duro e os pesos por nível. Botões no painel, na secção do Render.

⛔ **Os valores de fábrica saem do oráculo, não de gosto:** limiar `1,0`, `intensidade 0,3`,
`níveis 0 · 0,8 · 0,4 · 0,1 · 0 · 0 · 0`, mistura *Screen*.
⚠️ **E o número de níveis é DERIVADO da resolução**, não fixo em `7`: a `512` só cinco cabem
(§3.4). *Um nível cujo borrão é maior que o quadro mede a moldura.*

### `W7c` — O ANTI-SERRILHADO ao mexer ✅ **FECHADA (19/09)** → [§12](#12--a-borda-deixa-de-ferver-w7c-a-segunda-passagem-sai-da-bandeira)

O que o dono vê é a peça a «ferver» na borda enquanto orbita.

⭐⭐⭐ **A medição respondeu antes da escolha, e a resposta não era nenhuma das duas propostas:** a
borda re-amostrada **já existia nos dois motores** e estava desligada no quadro de movimento por uma
tabela de CPU de `640×360`. Medida no dispositivo e no caminho do pintor, ela custa `1,03×`–`1,09×`
— ⇒ *não havia uma lei nova para escrever, havia uma nota para reconferir* (`CLAUDE.md` §0.0).
⛔ Nem a acumulação temporal nem uma amostragem nova do marchador chegaram a ser precisas.

### `W7d` — A PROFUNDIDADE DE CAMPO

⚠️ **Recomendação: FICA DE FORA por omissão**, e a razão não é preço — é que num **modelador** ela
esconde a peça que se está a modelar. Ela é uma feature de **apresentação**, e o sítio dela é junto
do botão de exportar imagem, não no viewport de trabalho. ⛔ **Decisão do dono** (ele já disse que
a ouve: *«se preferir que a profundidade de campo fique de fora, diga»*).

## §6 — A UI

A secção **Bloom** no painel do Render, a seguir à **Style**, com as mesmas leis que a `W8` pagou:

- ⛔ **Toda linha que o modo não lê fica à vista, apagada, com a razão ao lado** (`ParamRow::inert`,
  decisão do dono de 18/09) — e aqui há quatro candidatas por construção: com o brilho desligado,
  as outras não fazem nada.
- ⛔ **Cada cor/número é um controlo SEU** — o defeito das cinco cores num controlo
  ([`11` §9](11_a_camada_de_estilo.md)) custou um espaço de nomes próprio, e este painel herda-o.
- ⚠️ **O tecto de cada slider vem de uma medição**, com a tabela ao lado (`CLAUDE.md` §0.0) — os
  três tectos que a auditoria da `W8` apanhou como palpite ([`11` §10.8](11_a_camada_de_estilo.md))
  são o precedente.

## §7 — Os gates que esta wave deve

| gate | o que afirma |
|---|---|
| `o_hdr_nao_muda_um_byte` | a `W7a` é byte-idêntica nos dois motores (a prova de que o substrato é inerte) |
| `o_que_nao_passa_do_limiar_nao_brilha` | a `§3.3` no caminho do produto: a `1,0` o halo é **zero** |
| `o_raio_de_cada_nivel_dobra` | a `§3.4`, medida no nosso motor contra a tabela do oráculo |
| `a_fabrica_e_a_identidade` | com `glow_enabled = false` a imagem é a de hoje **ao bit** |
| `o_brilho_le_depois_do_estilo` | um contorno pintado pelo estilo **brilha** (a ordem do §4) |
| `os_niveis_cabem_no_quadro` | o número de níveis é derivado da resolução, com piso de população |
| paridade CPU↔dispositivo | a mesma barra dos outros passes, e ⛔ **não afrouxada** |

## §8 — O smoke (para o dono)

A cena **`=36`** (a seguir à do estilo): a mesma peça, com uma **fonte de luz forte** e uma zona
escura ao lado — *sem as duas, o brilho não tem o que provar*. O roteiro nomeia os controlos pelo
nome que aparece no ecrã, e o gate `o_roteiro_da_cena_nomeia_controlos_que_existem` da `W8` já o
cobra automaticamente.

## §9 — ⚠️ Armadilhas MEDIDAS neste plano

- ⛔⛔ **Sete células do oráculo leram EXACTAMENTE o mesmo número, e eu imprimi uma tabela elegante
  sobre uma imagem em branco.** A cena do oráculo nunca foi construída (um erro de tipo no
  `_ready`), o quadro era a cor de fundo (`76` em todos os pixéis), e a minha régua somou-a com
  três casas decimais. ⇒ *toda varredura leva um **CONTROLO que tem de diferir*** (aqui,
  `glow_enabled = false`), e a primeira coisa a olhar num corpus é `min == max`.
- ⛔ **O `--headless` do Godot desliga o render** — ele é para script e servidor. Quem quiser a
  imagem corre na sessão virtual.
- ⛔ **A régua da intensidade satura a 8 bits** (§3.3): a coluna certa é o raio.
- ⚠️ **`kwin_wayland` precisa de `--exit-with-session`**; sem ele o comando não corre e não há log
  nenhum para ler — o modo de falha é **mudo**.

## §10 — ⛔⛔⛔ O report de 2026-09-19: o halo era CALCULADO, SOMADO, e NUNCA chegava ao ecrã

> *«Talvez devido a total falta de atmosfera não se possa perceber o efeito ao redor das esferas»*
> — o dono, com a foto da `=36` em Render, as três bolas brancas e **nenhum halo**.

### §10.1 — O que a medição disse, pelo caminho do produto

Com o app a correr (`=36`, `PH2D_FIELD_GPU=0`, Bloom ligado, `1898×916`):

| grandeza | medido |
|---|---:|
| pico da cena em linear (`campo_de_cena`) | `11,8381` |
| pico do halo (`ph2d_bloom::halo`) | `44,8896` |
| canais que o `soma_halo` MUDOU | **`2 738 165` de `5 215 704`** |
| maior salto de um canal | `255` bytes |
| o que se via na tela | **nada** |

⭐ O buffer que a thread de traçado produz foi despejado e **tem o halo**, magnífico. ⇒ *a lei
estava certa e o quadro estava certo; o que falhava era o que acontece DEPOIS dele.*

### §10.2 — A causa: luz com cobertura ZERO

O modelador traça com `BACKGROUND = [0, 0, 0, 0]` — **transparente de propósito**, para o canvas
do app (o cinzento, a grelha) aparecer por baixo; a decisão está escrita no próprio
`ph2d-app-field3d/src/smoke.rs` desde um smoke do dono de 19/08. E **um halo mora, por definição,
onde a peça não está** — ou seja, exactamente onde a cobertura é zero.

O `soma_halo` somava a luz e deixava o alfa como estava, com a frase *«luz acrescentada com alfa
inalterado é o que um compositor lê como luz»* ao lado. Essa frase é a álgebra do pré-multiplicado
e **é uma afirmação sobre o CONSUMIDOR** — e este consumidor compõe o quadro sobre o canvas e
multiplica a cor pela cobertura. *Luz com cobertura zero é luz multiplicada por nada.*

⭐ **A cura:** o halo é uma CAMADA de luz, e uma camada tem cobertura —
`c = max(r, g, b)` da luz que ela põe, composta como toda camada: `c + (1 − c)·a`. É **a mesma
conta** que a sombra do chão já fazia em `ground_shade::shadowed_background` (`(1 − f) + a·f`).
*A sombra já sabia que tapar o fundo custa cobertura; a luz é que não sabia.*

⚠️ **E o arredondamento é para CIMA.** A cor vai em **sRGB** e a cobertura em **linear**, e as duas
quantizam a ritmos muito diferentes: com arredondamento ao mais perto, `12 778` de `52 167` píxeis
acesos da cena ficavam com cobertura `0` (um linear de `0,0005` sai byte `6` na cor e `0` no alfa).
*Um pixel que recebeu luz nunca fica com cobertura nenhuma*, e o preço máximo é `1/255` de véu.

⚠️ **E a guarda `if c > 0.0` que a 1.ª redacção da cura tinha era provadamente MORTA** — a ida e
volta `byte → f32 → byte` é exacta nos **256** valores (medido). Ela saiu, e a propriedade que
alegava proteger virou gate (`o_alfa_atravessa_a_lei_sem_perder_um_byte`).

### §10.3 — ⛔⛔ Porque NENHUMA régua desta wave o via — são DUAS cegueiras somadas

1. **A fixtura dos gates do passe pintava sobre fundo OPACO** (`FUNDO = [0, 0, 0, 255]`). Ali o
   alfa é `255` em todo o quadro e **nunca chega a ser a grandeza que decide** — os sete gates
   mediam bytes de COR e passavam todos. *Uma fixtura com fundo opaco não pode conter este
   fenómeno.*
2. **A única coisa que alguém OLHOU foi a sonda `despeja_o_halo` — e ela escreve PPM**, um formato
   **sem canal alfa**. Foi com ela que eu verifiquei o halo antes de o entregar ao dono.
   ⭐⭐⭐ *Um despejo que deita fora um canal não pode auditar esse canal* — e o defeito vivia
   exactamente nesse canal.

⇒ Os gates novos pintam sobre **os dois** fundos, e o da crate do app corre a **cena a sério** com
o `BACKGROUND` **lido** do módulo (não repetido): se alguém o tornar opaco, é esse gate que diz que
a lei mudou de sujeito. Prova de mutação: **4 de 4 sangram** — e a 4.ª (`max` → `soma`) só passou a
sangrar quando ganhou uma fixtura com halo **TINGIDO**, porque *um corpus de uma cor só não testa
uma lei que fala de canais*.

### §10.4 — ⏳ O VIZINHO, nomeado e NÃO curado

`ground_shade::mais_luz` — a cor que a peça devolve ao chão (`docs/Render3d/09`) — tem a mesma
forma: soma luz e deixa o alfa. Pelo mesmo mecanismo, ela só chega ao ecrã onde a **sombra** já deu
cobertura (`a = 1 − f`), e evapora onde o chão está limpo. Ali isso é quase sempre invisível (a luz
devolvida é forte justamente dentro da sombra) e mudá-lo mexe na imagem de **toda** cena com chão
⇒ fica nomeado aqui, com o mecanismo, para quem lhe pegar não redescobrir a causa.

## §11 — ⭐⭐⭐ O BRILHO PASSA A CORRER NO DISPOSITIVO (o caminho de OMISSÃO)

> *«Bloom sem o comando especial»* — o dono, 2026-09-19, escolhendo entre as três frentes abertas.

### §11.1 — O que estava errado, e não era um defeito: era metade da feature

O modelador pinta no dispositivo desde a [`05` §39](05_o_modo_render_do_modelador.md) — a marcha e o
pintor correm lá e o que volta é a **imagem**. O brilho vivia só na cauda do sombreamento de CPU ⇒
com a placa a tomar o quadro, **as onze fileiras do painel nasciam APAGADAS**, o interruptor
incluído: *o artista não conseguia sequer LIGAR o efeito sem reabrir o app com `PH2D_FIELD_GPU=0`.*

⚠️ **E não havia atalho pelo relógio.** Medido (`--release`, melhor de 5, a sonda
`quanto_custa_a_cadeia` da `ph2d-bloom`), trazer a imagem de volta para a CPU custaria:

| tamanho | quadro | cadeia em CPU |
|---|---|---:|
| `445×305` | o de **MOVIMENTO** | **`24,1 ms`** |
| `1898×916` | o **ASSENTE** | **`347,2 ms`** |

O de movimento sozinho já não cabe num quadro de `16,7 ms`. ⚠️ **A razão é estrutural e declarada:**
a `ph2d-bloom` é uma FOLHA de **zero dependências** (a lei que os dois motores têm de responder
igual), logo não tem `rayon` — enquanto o resto do sombreador corre em `par_chunks_mut`. *O preço da
pureza da folha paga-se ali, e é por isso que o caminho de REFERÊNCIA fica lento com o brilho ligado.*

### §11.2 — A lei atravessa como TEXTO — a quarta a fazê-lo

[`ph2d_bloom::wgsl`] junta-se ao material, ao olhar e à lei do dono. ⚠️ **E o contrato tem uma
forma que as outras três não tinham: a porta de leitura entra NO MEIO da lei.** O WGSL exige
declaração antes do uso, e:

- o **corte** (`bl_corte`, `bl_cor`) não toca em dados ⇒ vem **primeiro**, e é isso que deixa o
  chamador chamá-lo de dentro da própria porta — que é como o tecto do *firefly* entra no primeiro
  degrau **sem um passe e sem um buffer do tamanho do quadro**;
- a **cadeia** (`bl_amostra_uv`, `bl_desce13`, `bl_sobe_tenda`) chama a porta ⇒ vem **depois**.

⇒ a folha exporta `fonte(bl_le: &str) -> String` em vez de uma const. *Uma const única obrigaria o
chamador a escolher entre não poder cortar e pagar uma cópia do quadro inteiro.*

⭐ **E a amostragem é bilinear escrita à mão, nunca um `textureSample`:** a referência de CPU faz a
sua própria bilinear em UV, e um *sampler* de hardware traz arredondamento próprio. *Duas leis
iguais amostradas por portas diferentes deixam de ser a mesma lei* — e é por isso que a paridade
abaixo fecha onde fecha.

### §11.3 — A paridade, medida

`crates/ph2d-app-field3d/src/brilho_parity_tests.rs`, pelo caminho do **produto** (o mesmo
`Presentation` nos dois motores, a mesma peça, o mesmo fundo transparente):

| bateria | acendeu (CPU) | acendeu (dispositivo) | canais fora de `1` byte | pior |
|---|---:|---:|---:|---:|
| fábrica | `58 124` | `58 124` | `0` | `1` |
| raio `8` | `59 340` | `59 340` | `0` | `1` |
| tingido (sat `0`) | `56 936` | `56 936` | `0` | `1` |
| tecto `2` | `53 391` | `53 391` | `0` | `1` |
| anamórfico (`3×`, `30°`) | `58 748` | `58 748` | `0` | `1` |

⚠️ **As duas colunas do meio são o CONTROLO**, e sem elas a tabela não afirma nada: uma cadeia que
não corresse em nenhum dos dois lados daria `0` fora da barra e leria-se como vitória.

**Custo no dispositivo**, contra a mesma cadeia em CPU:

| tamanho | no dispositivo | na CPU |
|---|---:|---:|
| `445×305` | **`0,35 ms`** | `24,1 ms` |
| `1898×916` | **`1,7`–`4,1 ms`** | `347,2 ms` |

⚠️ **O número do assente BALANÇA, e a razão é conhecida:** cada quadro cria os buffers da cadeia
(`~50 MB` a essa resolução) e deita-os fora ⇒ o relógio segue o alocador do driver. *É uma
optimização com endereço — uma cache por tamanho, como o traçador já tem — e não um defeito.*

### §11.4 — ⭐⭐ Três notas que esta wave APAGOU, e uma que ela criou

1. O cabeçalho do `brilho_painel.rs` **encomendava esta wave por escrito** (*«o `p_pinta` a escrever
   o cena-linear num segundo buffer, a cadeia de níveis em compute e a composição»*) — está feita, e
   a nota foi substituída pelo registo de que ela existiu.
2. A razão de inércia `field.inert.bloom_runs_on_the_reference_path` **saiu do i18n**: ela mandava o
   artista reabrir o app com uma variável de ambiente. ⚠️ Ficam as **três razões da LEI** (o brilho
   desligado · o joelho com o limiar em zero · o ângulo num halo redondo), que não dependem de motor.
3. O passo `(0)` do roteiro da `=36` (*«o comando já traz `PH2D_FIELD_GPU=0`»*) **saiu**.
4. ⏳ **E a que fica nomeada:** o caminho de REFERÊNCIA continua a pagar os `347 ms`. Ele é o que
   corre numa máquina sem placa, e a cura é paralelizar a cadeia — ⛔ o que a folha não pode fazer
   sem deixar de ser uma folha de zero dependências. *A saída provável é o chamador emprestar-lhe o
   paralelismo, e ela não foi medida.*

### §11.5 — ⛔ O que a prova de mutação apanhou

`7` mutações, `7` sangram — incluindo a que reproduz o report de 19/09 **do lado do dispositivo** (a
cobertura não sobe) e a que apaga a escrita do cena-linear no pintor. ⚠️ **E o gate que corre em
TODA máquina é outro:** o shader é montado de três fontes em tempo de execução, logo um erro de
sintaxe só apareceria quando o artista ligasse o Bloom — `ph2d-field-gpu/tests/brilho_wgsl_valido.rs`
parsa e valida os dois textos com o `naga`, sem placa nenhuma. *Os gates de paridade são
`#[ignore]`, logo o CI nunca os corre.*

## §12 — ⭐⭐⭐ A BORDA DEIXA DE FERVER (`W7c`): a segunda passagem sai da bandeira

> Report do dono, 2026-09-19: *«a peça ferve na borda enquanto orbito»*.

### §12.1 — A régua do fenómeno, ANTES de escolher a forma

O §5 desta wave deixou a `W7c` com a forma por decidir — *«a escolha é entre acumular quadros
(temporal) e amostrar melhor a borda no próprio marchador … medir antes de escolher»*. A primeira
coisa que se construiu foi a **régua**, e não a cura ([`borda_sondas::a_regua_do_fervilhar`],
`1920×1080`, no caminho do pintor):

| coluna | o que mede |
|---|---|
| `banda` | os pixels cuja vizinhança `3×3` atravessa a fronteira da peça |
| `parcial` | que fracção dela leva cobertura **entre** `0` e `255` |
| `pior` | o maior salto de um pixel quando a câmara roda `0,0005 rad` (**sub-pixel**) |

| cena | quadro | `parcial` | `pior` |
|---|---|---:|---:|
| `0` | mexia | **`0,0 %`** | `213,8` |
| `0` | assente | `28,3 %` | `137,0` |
| `11` | mexia | **`0,0 %`** | `211,7` |
| `11` | assente | `28,3 %` | `127,7` |
| `33` | mexia | **`0,0 %`** | `212,8` |
| `33` | assente | `28,7 %` | `128,6` |
| `36` | mexia | **`0,0 %`** | `221,7` |
| `36` | assente | `38,9 %` | `137,0` |
| `5` | mexia | **`0,0 %`** | `0,7` |
| `5` | assente | `27,7 %` | `24,9` |

⭐⭐⭐ **A silhueta que a mão arrasta não tinha UM ÚNICO pixel de cobertura parcial** — ela era uma
escada binária, e uma rotação de meio milésimo de radiano fazia um pixel saltar `212`–`222` de
`255`. *É essa a assinatura da fervura: o pixel não escurece um pouco, ele apaga-se.*

⚠️⚠️ **A cena `5` está na tabela e diz o contrário das outras — e é por isso que ela fica.**
A coluna `parcial` conta a mesma história (`0,0 %` → `27,7 %`), mas o salto é `0,7` de `255`, e a
causa é **geométrica e não um limite da régua**: a `5` é o TORNO — uma superfície de revolução em
torno de `Y` — e o passo desta régua é uma rotação em **yaw**, que é exactamente o eixo dela. *A
silhueta de um sólido de revolução é invariante à rotação em torno do próprio eixo*, logo aqui não
há fervura NENHUMA para medir — e o `24,9` da linha de baixo é o sombreado a mudar (a lâmpada da
sonda é posta em relação à câmara e roda com ela), não a cobertura.

⭐ *Uma tabela que só mostra as cenas em que a régua fala é uma tabela escolhida* — a `5` fica
porque uma linha que **não** mede o fenómeno, com a razão escrita, é o que separa uma medição de um
argumento.

⚠️ **O `d` é sub-pixel de propósito.** Com a câmara a rodar meio grau por quadro toda a banda é
outra e a diferença entre as duas leis desaparece na aritmética; é no regime em que a silhueta se
desloca uma **fracção** de pixel que uma imagem honesta muda pouco e uma escada dura vira pixels
inteiros.

⚠️⚠️ **E as três colunas do salto, nunca só o máximo:** a primeira redacção contava *«quantos pixels
saltam mais de `64`»* e a barra caía exactamente em cima do degrau de cobertura do padrão de quatro
amostras (`1/4` de `255`), logo ela lia ruído. *Um máximo é UM pixel e cai no ponto de maior
contraste da imagem; a média é o resumo honesto.*

### §12.2 — ⛔⛔ O OUTRO passageiro da mesma lei está INERTE, e isso mudou a wave

A régua leu `mexe` ≡ `assente` **ao bit em todas as colunas** assim que a segunda passagem entrou —
o que só pode ser verdade se o contorno **engrossado** do quadro de movimento não estiver a mudar
nada. Medido ([`borda_sondas::o_contorno_grosso_muda_alguma_coisa`]): o
[`preview::coarse_doc`](../../crates/ph2d-app-field3d/src/preview.rs) **não morde em NENHUMA** das
`23` cenas vivas do smoke.

O mecanismo está escrito no próprio ficheiro desde 2026-09-16: a decimação devolve uma **polilinha**
e os perfis das cenas são feitos de **arcos**; um arco já tem a normal exacta, logo
`thin.prim_count() < profile.prim_count()` é falso e o documento volta intacto. ⇒ *as duas metades
da lei «grosso a mexer» — o contorno e o anti-serrilhado — e só uma delas alguma vez mordeu.*

⚠️ **Não é código morto e não foi curado:** um perfil vindo de um DESENHO do artista (a `W53`) pode
ser uma polilinha com muitos segmentos, e ali a decimação ganha. O que está medido é que **nenhuma
cena do smoke** o alcança — e é isso que torna a igualdade do §12.4 possível.

### §12.3 — ⛔⛔⛔ O PREÇO: a nota que a tinha posto fora do quadro morreu com o dispositivo

A tabela que governa a decisão vive no [`ph2d_field_render::trace_cancellable`] e diz
`1,30×`–`1,40×` do quadro. Ela foi lida **na CPU, a `640×360`, antes de o quadro inteiro ir para a
placa**. *Quem move o número que tornava algo inalcançável tem de reconferir a nota*
(`CLAUDE.md` §0.0) — e foi essa mesma lei que atravessou o brilho para o dispositivo na §11.

Medida outra vez, no **caminho do pintor** e a `1920×1080`
([`borda_sondas::quanto_custa_a_borda_no_pintor`], mínimo de `5` **intercalado no mesmo processo**,
duas corridas a concordar, `ociosa 83 %`):

| cena | `hoje` | `+borda` | delta | razão | `assente` |
|---|---:|---:|---:|---:|---:|
| `0` | `11,14` | `11,52` | `+0,38` | `1,03×` | `22,31` |
| `5` | `29,82` | `32,48` | `+2,66` | `1,09×` | **`316,95`** |
| `11` | `16,91` | `18,47` | `+1,56` | `1,09×` | `35,14` |
| `33` | `6,08` | `6,26` | `+0,18` | `1,03×` | `12,30` |
| `36` | `4,58` | `4,83` | `+0,25` | `1,05×` | `10,98` |

⭐⭐⭐ **E o que a tirou da bandeira foi a COMPANHIA, não o preço dela.** A coluna `assente` é a mesma
bandeira com os outros passageiros ligados: na cena `5` ela custa **`+284 ms`**. *Uma bandeira que
junta passageiros com preços a duas ordens de grandeza de distância é exactamente como o barato fica
invisível* — quatro coisas atrás de uma palavra, e a palavra era o nome da mais barata.

⚠️ **A metade da medição que a carga da máquina NÃO alcança** é a ocupação: a passagem re-amostra
**`0,23 %`–`0,59 %`** dos pixels (`4 832`–`12 186` de `2 073 600`). *É ela que diz, sem relógio
nenhum, que não há hipótese de o preço ser grande* — e ela é a mesma numa máquina a `load 50` e
numa parada.

⛔ **Medir pelo [`gpu_frame::march`] não decide isto:** ali o G-buffer inteiro (`49,8 MB`) atravessa
o barramento e a diferença afoga-se no ruído (`0,93×`–`1,03×`, com `±6 ms` de dispersão). O caminho
do produto é o pintor, onde só a imagem volta (`8,3 MB`).

### §12.4 — A cura: ela deixa de ser uma DECISÃO do quadro

A segunda passagem sai da bandeira e passa a ser propriedade do caminho:

- [`preview::re_amostra_a_silhueta()`](../../crates/ph2d-app-field3d/src/preview.rs) é a **porta**,
  com a tabela ao lado, e tem **três leitores** — o pintor (pela `Sonda::default`), o recuo pelo
  `march` e o recuo pela CPU. ⚠️ *Uma lei escrita em três sítios viaja para os dois de que alguém se
  lembrou*: o censo `os_tres_caminhos_de_um_quadro_leem_a_mesma_porta` reprova quando um deles
  escreve o `true` à mão.
- [`gpu_frame::Sonda::bordas`](../../crates/ph2d-app-field3d/src/gpu_frame.rs) é a porta de
  **bissecção** e de re-medição — a mesma forma do `chao_recebe_cor`, e **não é caminho de produto**.
- A bandeira **muda de nome** (`antialias` → `assente`) nos sítios onde ainda decide alguma coisa: o
  contorno engrossado, a sombra directa da CPU, o ricochete e o campo do chão. *Um nome que descreve
  a bandeira pelo passageiro que já não viaja nela é o defeito seguinte à espera.*

⭐⭐ **O que o dono lê na tela:** o quadro que a mão arrasta passa a ter a **mesma silhueta** do
quadro de parar — e não parecida: o canal de cobertura é **igual ao byte**, com gate
(`a_borda_que_a_mao_arrasta_e_a_de_parar`), e as duas imagens continuam a diferir na COR, que é o
que prova que o gate não está a comparar dois quadros idênticos.

### §12.5 — ⚠️ O que esta wave NÃO faz

- ⛔ **Ela não afina o padrão de amostragem.** As quatro amostras do `ROOK` dão **cinco** níveis de
  cobertura, logo um degrau vale `64/255` e o pior salto medido cai de `~215` para `~130` — não para
  perto de zero. *O alvo desta wave é o quadro de movimento deixar de ser PIOR que o parado*, e essa
  igualdade está gateada; subir o número de amostras é outra wave, e teria de pagar o preço nos dois
  quadros.
- ⛔ **Ela não é adaptativa, de propósito.** Ligar a passagem só quando o quadro «cabe no orçamento»
  faria a silhueta alternar entre as duas leis conforme o relógio — *uma fervura nova, construída
  para curar a antiga*.
- ⚠️ **Duas cenas já estavam fora do orçamento antes desta wave** (a `5` a `29,8 ms` e a `11` a
  `16,9`, contra `16,7`), e continuam. *O que as põe lá é a marcha, não esta passagem* — é matéria
  da `W9`, que o dono pôs ao fim da fila.

### §12.6 — O smoke (para o dono), e porque ele são DUAS corridas

⛔⛔ **Uma cura que apaga o defeito não se pode demonstrar sozinha:** depois desta wave o dono abre o
app e a borda está boa — e uma borda boa é indistinguível de uma borda que nunca esteve má. ⇒ o
smoke é um **A/B**, e a porta que o permite é a `PH2D_FIELD_BORDA=0`, que é a mesma porta de
bissecção que um report futuro de *«piorou»* vai precisar.

1. `cd <worktree> && env PH2D_FIELD_SMOKE=36 cargo run -p ph2d-host-desktop --profile smoke` —
   arrastar com o botão esquerdo para rodar a peça, a olhar para o **contorno** das bolas.
2. a mesma coisa com `PH2D_FIELD_BORDA=0` à frente — a serrilha volta, e é ela que fervia.

**Deu errado se:** a borda estiver igual nas duas corridas (a porta não chega ao produto) · a peça
ficar visivelmente mais lenta a rodar (o preço medido é `+0,18`–`+2,66 ms`, e a `36` é `+0,25`) · a
borda mudar de aspecto **ao largar** o rato (a igualdade do §12.4 partiu-se).

⚠️ **Não há cena nova, e isso é a decisão certa:** a fervura existe em **toda** cena com silhueta, e
a `36` é aquela em que o dono acabou de reportar o defeito. *Uma cena inventada para demonstrar um
defeito que a cena dele já contém é uma fixtura a mais para manter.*

### §12.7 — ⛔⛔ O que o portão apanhou, e uma delas é um defeito de INSTRUMENTO

**(a) Um censo classificava código de teste pelo NOME do ficheiro, e acusou a sonda desta wave.**
O `the_export_never_goes_through_the_preview_coarsening` reprovou com a mensagem *«o `Resolution` do
artista deixa de ter efeito observável»* sobre um produto **correcto**: a sonda
[`borda_sondas`](../../crates/ph2d-app-field3d/src/borda_sondas.rs) é compilada só sob
`#[cfg(test)]` e não acaba em `_tests.rs`, logo foi lida como PRODUTO.

⚠️ **Eram TRÊS censos com o mesmo furo** no mesmo directório (`preview_tests`, `view_tests`,
`reload_tests`) — e o comentário de um deles **já escrevia a fronteira certa**, à letra: *«a
fronteira certa não é "este ficheiro": é "código que corre no app"»*, com a linha seguinte a
implementar a errada. ⛔ E [`device_probes`](../../crates/ph2d-app-field3d/src/device_probes.rs)
vivia no mesmo buraco desde que existe — ele só nunca tinha chamado nada que um censo procurasse.

⇒ [`censo_de_ficheiros::ficheiros_de_teste`](../../crates/ph2d-app-field3d/src/censo_de_ficheiros.rs):
*o que um ficheiro de teste declara por `#[path]` é código de teste*, transitivamente — a mesma cura
que a `line/sculpt3d` pagou em 15/09 (`CLAUDE.md` §5). Uma porta, **três** leitores, e controlo
próprio nos DOIS sentidos: excluir a MAIS deixa os três censos *verdes a medir nada*, que é a
direcção muda.

**(b) O arnês da prova de mutação mentiu DUAS vezes, e as duas estão escritas no `CLAUDE.md`.**
`-- --ignored` corre **só** os ignorados, logo o censo textual (que não é `#[ignore]`) casou **zero**
testes e leu-se como *sobreviveu*; e com duas corridas a disputar a placa o `cargo` saiu sem nunca
chegar a correr. ⭐ **As duas foram apanhadas porque o arnês conta o `running N tests`** — sem esse
controlo sobre o próprio filtro, o relatório teria dito *«3 de 5 sobreviveram»* sobre gates que
sangram. Com a placa livre e `--include-ignored`: **5 de 5**.

**(c) Uma flake de carga NOVA para promover à lista do `CLAUDE.md` §5.0:**
`materials::tests::dragging_a_colour_compiles_no_tape_at_all` (`ph2d-app-field3d`) — um **contador**
de fitas compiladas atrás de estado partilhado, que é a espécie que aquela lista já nomeia. Reprovou
uma vez a `load 52` (*«compilou 1 fita»* contra `0`), passa **3 de 3 sozinho** e a suíte inteira
fechou **verde duas corridas seguidas** com a máquina mais calma. ⚠️ *O conjunto de reprovadas mudou
entre corridas da mesma árvore*, que é a assinatura.

**(d) ⛔⛔ UMA SONDA DO CONJUNTO `#[ignore]` NÃO CABE NO PRAZO DA PORTA DE RECURSOS, e por isso a
corrida de fecho a SALTA — com o número ao lado.** O
`preview::device_probes::mede_o_preco_de_uma_aresta_de_perfil` varre perfis até `768` arestas;
medido nesta jornada, ele ultrapassa **`25 min` sozinho** e comeu duas corridas inteiras do conjunto
(`1800 s` e depois `> 2400 s`) sem chegar ao fim. ⚠️ O doc dele **já avisava** que o ponto de `1024`
tinha pendurado o driver por mais de meia hora; o que esta jornada acrescenta é que **o ponto de
`768` já torna o conjunto inteiro incorrível**.

⇒ o fecho corre `cargo test … -- --ignored --skip mede_o_preco_de_uma_aresta_de_perfil`. ⭐ *É uma
SONDA e não um gate* — ela imprime uma tabela e não afirma nada —, logo saltá-la não tira
verificação nenhuma; o que ela tira é uma tabela que ninguém pediu nesta wave. ⚠️ **E isto não é
uma isenção silenciosa:** quem quiser o número dela corre-a à parte, com o prazo levantado.
⛔ *Uma sonda que PODE pendurar a placa gasta a máquina de quem a corre e não avisa* — a frase é do
próprio doc dela, e vale também para o relógio.

**(e) ⭐⭐⭐ UM SEGUNDO GATE TINHA A PREMISSA MORTA, e ele reprovou por `191` níveis — a reprovar é
que ele estava a funcionar.** O `render_bounce_gpu_tests::o_quadro_de_movimento_nao_paga_o_ricochete`
compara a imagem do dispositivo com a que a CPU pinta **sem** ricochete, e conduzia os dois lados com
`false`. O comentário dele dizia, à letra: *«`false` nos DOIS lados — é a bandeira do quadro de
movimento, e ela **também desliga a re-amostragem da borda**: comparar um lado com ela e outro sem
mediria o anti-serrilhado»*. ⇒ depois desta wave o `march` governa a **borda** e o `paint` governa o
**ricochete**, logo o mesmo `false` pôs os dois lados em regimes diferentes e a comparação passou a
medir exactamente o que aquele comentário existia para impedir.

⭐ *A linha errada e a linha certa dizem a MESMA coisa* — «os dois lados no mesmo regime» —, e o que
mudou por baixo foi **qual argumento exprime esse regime**. Com o lado da CPU a ler a porta
([`preview::re_amostra_a_silhueta`](../../crates/ph2d-app-field3d/src/preview.rs)) o desvio vai de
`191` para **`0` níveis**: os dois motores pintam o quadro de movimento **byte a byte igual**, que é
melhor do que a barra de `2` que aquele gate sempre teve.

⚠️ **E nenhum dos gates NOVOS desta wave o podia ter apanhado**: eles medem o dispositivo contra si
próprio (a cobertura parcial, a igualdade de alfa entre os dois quadros). *Quem apanhou foi um gate
de PARIDADE entre motores, escrito por outra wave* — e é por isso que o fecho corre o conjunto
inteiro e não só os testes da wave.

## ⛔ Recusas MEDIDAS

| o que foi recusado | porquê, com o número |
|---|---|
| pôr o brilho na `ph2d-style` | ele lê os VIZINHOS ⇒ é um passe; a crate deixaria de ser a lei partilhada (§4, a mesma recusa da [`11` §11](11_a_camada_de_estilo.md)) |
| ler a lei do brilho no fonte do Godot | é legal (MIT) e é o método pior: o oráculo **corre-se**, e a §3.3/§3.4 saíram de `15` corridas (§0.9) |
| `--headless` para colher o oráculo | força o driver **dummy**: a saída é preta e lê-se como «não há brilho» |
| medir a intensidade pelo PICO do halo | satura em `1,00000` a partir de `entrada 4,0` — 8 bits (§3.3) |
| fixar a cadeia em `7` níveis | a `512` px os níveis `5`–`7` são maiores que o quadro e medem a moldura (§3.4) |
| portar o CORTE do alvo (os três botões de HDR) | dois estão inertes e o limiar não gateia: entrada `2` com limiar `3` ainda brilha `0,0802` (§3.5) |
| mudar a assinatura do `shade_render` | `50` chamadores, `1` de produto — a porta nova **delega** e o churn é zero (§5) |
| uma wave só de substrato (o buffer sem o brilho) | um canal sem consumidor é código morto, que é a lei que a `11` §11 já escreve (§5) |
| trazer a imagem do dispositivo para a CPU só para o brilho | a cadeia em CPU custa `24,1 ms` no quadro de MOVIMENTO e `347,2 ms` no assente (§11.1) |
| um `textureSample` na cadeia do dispositivo | a referência de CPU faz a própria bilinear em UV; um sampler traz arredondamento próprio e a paridade de `1` byte não fecharia (§11.2) |
| o `if c > 0.0` à volta da cobertura | a ida e volta `byte → f32 → byte` é exacta nos **256** valores ⇒ guarda provadamente morta (§10.2) |
| `soma` dos canais como cobertura | over-cobre um halo tingido: `[0,4 · 0,3 · 0,3]` sairia **opaco** e taparia a grelha; `max` é a MENOR cobertura válida (§10.3) |
| a profundidade de campo LIGADA por omissão | num modelador ela esconde a peça que se está a modelar — proposta, decisão do dono (§5) |
| ligar o anti-serrilhado do movimento só quando o quadro «cabe» | a silhueta alternaria entre as duas leis conforme o relógio — *uma fervura nova, construída para curar a antiga* (§12.5) |
| medir o preço da segunda passagem pelo `gpu_frame::march` | ali o G-buffer (`49,8 MB`) atravessa o barramento e a diferença afoga-se: `0,93×`–`1,03×` com `±6 ms` de dispersão (§12.3) |
| contar «quantos pixels saltam mais de `64`» como régua da fervura | a barra cai em cima do degrau de cobertura do padrão de 4 amostras (`1/4` de `255`) e lê ruído (§12.1) |
| subir o número de amostras do `ROOK` nesta wave | o alvo medido é o quadro de movimento deixar de ser PIOR que o parado, e isso já está gateado ao byte; mais amostras pagam-se nos DOIS quadros (§12.5) |
| curar o contorno engrossado inerte | ele não morde em nenhuma das `23` cenas porque os perfis são ARCOS — curá-lo seria tornar o quadro de movimento mais grosseiro, a direcção oposta a esta wave (§12.2) |

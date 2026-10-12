# ARQUIVO — 89_plano_conferencia_dos_nos.md (história, 201 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`89_plano_conferencia_dos_nos.md`](../../Motion%20Nodes/89_plano_conferencia_dos_nos.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-412,614-674` do original.
>
> ⚠️ **A única alteração ao corpo:** 2 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## §10.2 — A ORDENAÇÃO GLOBAL DAS WAVES (o produto do plano)

**A régua da §7 foi escrita antes dos resultados e não se mexe.** O que os resultados
acrescentam é uma coisa que a régua não previa: **~12 dos ~52 P0 são o MESMO gap**, visto de oito
famílias diferentes. Ordenar por família colocaria a mesma construção em oito waves.

⚠️ **Isto NÃO revoga a §8** (*uma wave por família*). A §8 existe porque uma família partilha
kernel, gates e cena de smoke — e o muro da §10.0 **não pertence a família nenhuma**: ele é o
substrato que todas consomem. Uma wave foundational antes das de família é a consequência da §8,
não a exceção dela.

### W0 — O LEITOR E O ESCRITOR DE COLUNA  ·  *o desbloqueador*

| | |
|---|---|
| **Mata** | ~12 P0 em **8 famílias** (1 · 4 · 5 · 6 · 8 · 9 · 10 · 15) |
| **Custo** | dois nós que **já existem** ganham simetria; zero contrato congelado, zero schema |
| **Metade A (LER)** | `value.attribute` passa a devolver componentes X/Y de `Vec2` e canais de `Vec4` — e **para de devolver zeros em silêncio** (é DEFEITO, não feature) |
| **Metade B (ESCREVER)** | `motion.drive` toma o canal por **NOME**, o espelho exato do `value.attribute` Custom — os cinco de hoje viram os cinco nomes de sempre |
| **Cai de graça** | `field.from_value` (a máscara é uma coluna) · `make_point` para `vel` · hue/sat/lightness · o pivô-centroide · `@P.x` na fórmula |
| **Default que reduz** | os cinco canais atuais, os dois modos atuais — bit a bit |

**Estado: A ✅ · B ✅ (a metade que desbloqueia) · B-genérico ⏳ (wave própria, com o preço medido).**

- **A — o leitor** (`53b54a773`): `value.attribute` lê **pista a pista** (`MODE_COMPONENT_BASE + k`),
  um degrau para Vec2/Vec3/Vec4 ⇒ o caso da COR cai junto. CPU **e** device, porque o
  `encode_project` replica a escada e shipar meia levaria à divergência que o T5 catalogou.
- **B — o escritor, pela porta barata:** a família FIELD nomeou **duas** formas do destravador —
  *"`field.from_value` **ou o canal `Falloff` no `motion.drive`**"* — e a segunda é **binding
  ESTÁTICO**, ou seja GPU-nativa pela máquina que já existe. `motion.drive` ganhou o 6º canal:
  um número computado vira **a máscara que cinco famílias leem**.
  - ⚠️ **Não auto-mascara, e não foi escolha:** todo outro variante binda `falloff` como `Read`
    para misturar, então o variante cujo ALVO é `falloff` o binda uma vez como `ReadWrite` e o
    read comum simplesmente não existe — a auto-máscara é **inexprimível na lista de bindings**,
    que é o tipo de recusa que sobrevive à próxima pessoa.
  - ⚠️ **Não clampa em `[0,1]`, de propósito:** um peso NEGATIVO inverte a força que o consome —
    a capacidade que a família FORCE achou ser nossa e não do C4D/Cavalry (que são `[0,1]` por
    construção). Clampar aqui a apagaria antes de alguém a usar.
  - ⚠️ **Identidade `1.0`, não `0.0`:** coluna ausente significa *efeito cheio* para todo leitor
    da biblioteca, e um escritor que começasse do zero discordaria de todos eles.
- **B-genérico — o que SOBRA e por que é wave própria:** escrever uma coluna **por NOME** (`vel`,
  `parent`, `len` — as famílias 13 e 16) precisa do análogo de ESCRITA do `StreamOp::Project`,
  porque um nome dinâmico não é binding estático. É maquinaria de sequenciador com pipes e
  paridade próprios, **não um apêndice**. O canal `Falloff` já mata a linha mais cara da tabela
  da §10.0; as outras três esperam essa wave.

⚠️ **A metade A vem primeiro e sozinha**, porque é conserto de uma resposta errada: hoje
`_ => vec![0.0; n]` entrega zeros com cara de dado. Um defeito não espera priorização de produto.

### W1..W7 — as famílias, pelo P0 que SOBRA depois do W0

| ordem | wave | P0 restantes | por que aqui |
|---|---|---|---|
| **W1** | **SIMULAÇÃO** (fam. 3) | 8 → **~7** | o maior bloco que o W0 não toca — e o item de cabeça é *os 3 geradores não consomem `accel`*, ou seja **a família `force.*` inteira não alcança simulação nenhuma**. É um segundo desbloqueador, barato (uma leitura de coluna) e com fan-out de 6 nós — **W1-A ✅**, ver abaixo |
| **W2** ✅ | **SOURCE** (fam. 14) | 4 → **0** | **FAMÍLIA FECHADA em 2026-08-14.** As 39 formas caíram em 2026-08-09 (o catálogo do editor virou o do grafo, 8 → 43); o `time_offset` e o traço fecharam em 12-13/08; e o **TEXTO** — a linha 1 da §3, *"metade do mograph do mundo é texto animado por caractere"* — fechou com a crate `ph2d-node-source-text`, **uma instância por GLIFO**. ⚠️ A frase da coluna vale para os quatro: **era FIAÇÃO, não geometria** — o módulo Vector já tinha o texto vetorial inteiro, e nada no grafo o alcançava. Cena `=39`. ⚠️ **E o ÁUDIO fechou em 2026-08-15** — ele não era P0 de folha nenhuma (a 14 §3 item 6 apenas o NOMEAVA *"para o consolidador não o perder na fronteira entre famílias"*, com o P0 vivendo no doc 63 §3), e por isso era o item que a varredura por folha teria perdido: **a nota que o salvou é a que o declarava fora do próprio escopo**. Cena `=40` |
| **W3** ✅ | **COR** (fam. 9) | 6 → **~3** → **6 FEITOS, FAMÍLIA FECHADA** | **os seis P0 fechados em 2026-08-09** ([§3 do 09_cor](../../Motion%20Nodes/89_conferencia/09_cor.md)): a alfa que não chegava ao device (**defeito**) · a máscara por campo · o canal "Opacity" do picker (que a tabela lia como *duas portas* e a medição mostrou ser **um FANTASMA** — ninguém escreve a coluna `opacity`) · o **canal do `motion.luminance`**, de 1 para 8 · e o **ESPAÇO de interpolação da rampa** (token `g3`, append-only; o motor sempre soube interpolar em HSV/HSL e o formato **não tinha onde guardar a escolha**, e o device herda de graça porque o LUT é assado na CPU). · e a **metade de ESCRITA do laço de cor** (os canais Hue/Saturation/Value do `motion.drive`, o espelho exato do `luminance` — e os modos `Add`/`Multiply` que já existiam JÁ eram o *Master Hue* / *Saturation* da referência). **Nenhum P0 restante nesta família** |
| **W4** ✅ | **TEMPO / ESTILÍSTICOS** (fam. 7) | 4 → **4 FEITOS, FAMÍLIA FECHADA** | **o DEFEITO caiu em 2026-08-09** ([§6 do 07_tempo](../../Motion%20Nodes/89_conferencia/07_tempo_estilisticos.md)): o `motion.morph` emitia **só `P`** e descartava `size`/`rot`/`tint`/`uv_rect`/`texture_id`/`geometry_id` — morfar dois `source.object` perdia a aparência e caía na tile 0. Agora as quantidades desvanecem e a identidade é carregada pelo **vizinho mais próximo** (lista BRANCA, cujo apodrecimento se VÊ). E a **máscara por campo do `trail`** (o único behaviour da família que não lia `falloff`): `falloff = 0` deixa a cabeça VIVA e tira o rastro, e o fantasma **herda** a máscara do ancestral porque o `gather` já carrega toda coluna. E o **canal do `delay`**, que só atrasava a POSIÇÃO: o prefixo `0..3` do enum compartilhado FICA com o significado que tem (o índice é o que o *documento* guarda, e seis crates carregam a cópia do mesmo `channel.rs`) e o nó estende com **`4` Position / `5` Colour**, as duas quantidades inteiras da referência. O filtro passou a trabalhar em `dim` floats por elemento ⇒ Position **byte-idêntica**; e duas coisas SABEM o que é o canal: um ângulo é um círculo (a linha guarda o **desenrolado**, senão o one-pole desanda 71,5° na emenda do `look_at`) e uma linha só é história da MESMA quantidade (Position e Size são ambos `Vec2`). E o **reset do `step`**, que não tinha volta (um tique monotônico que só sobe; baixar o `count_max` só re-dobra o mesmo tique, e nenhum nó escreve o `pre` de outro): uma **quarta porta APENDADA** ⇒ os três índices que um grafo salvo guarda não se movem e, não-ligada, ela cozinha para stream vazio — **byte-idêntica**, provado pelos nove gates que passam sem edição. ⚠️ **A contagem precisa da BORDA e o reset não** — *a borda importa onde o efeito acumula* —, e uma lei entrega o `While On` e o botão `Off to On` do TD sem param de condição. Os quatro P0 caíram |
| **W5** | **DISTRIBUIÇÃO + EMISSÃO** (fam. 1) | 4 → **~3** | o W0 mata a tangente; sobram a FORMA do emissor e o arco radial |
| **W6** | **PULSE** (fam. 12) | 2 | o *nível* é o gargalo único: com ele, gate/AND/OR/sequenciador/burst caem juntos |
| **W7** ✅ | **SIM.\*** (fam. 13) | 3 → **0** | o estágio de EVENTOS dependia do W6, que fechou em 2026-08-10 — e os **TRÊS P0 caíram na mesma janela**: o evento de MORTE (as saídas `died`/`pulse` do `sim.lifetime`, cena `=27`; `sim.replicate` acabou não sendo um nó, e sim uma fiação) · o **RAIO da partícula** (a inflação de Minkowski, cena `=28`; colidíamos um PONTO, então um sprite afundava até a metade no chão) · e o **plano NÃO-HORIZONTAL** (a forma de Hesse, cena `=29`). ⚠️ O terceiro **não era o mesmo tipo de item** — os dois primeiros eram omissões dentro da lei que o nó já tinha, e este pedia uma NORMAL onde havia o literal `vec2(0,1)` —, e é por isso que a cura foi a forma dar ORIENTAÇÃO a si mesma em vez de compor: as duas cadeias que a folha tentou seguem REFUTADAS (`motion.rotate` escreve só `rot`; uma cadeia de chãos é uma escada). **Uma parede e um teto vieram de graça** — a mesma meia-reta virada um quarto e meia volta. ⚠️ **E em 2026-08-11 a varredura seguiu para os P1 e fechou TRÊS de sete**: o **EVENTO DE CONTATO** (a coluna `hit` — *quão fundo a colisão deste tique empurrou*, `0` onde nada tocou; contato é `depth > 0` **por construção**, então um número carrega os dois fatos sem que possam discordar; acumula por `max` e a zona a estripa, como o `accel`; cena `=30`, **FULLY GPU, 8 estágios**) · os **COMPORTAMENTOS nomeados**, que **dissolveram** nele (*stick*/*slide* são os dois knobs que já estão na tela, e *die* virou a composição `hit → drive(Falloff) → cull` dentro da zona — um enum `behaviour` shiparia três nomes para dois números) · e o **BURST**, que **já estava construído** desde a porta `pulse` sem ninguém reconferir a nota. ⚠️ **Duas células envelheceram DENTRO desta linha** e foram corrigidas junto: a §0.1 afirmava *"`motion.drive` tem exatamente cinco canais"* — a W3 COR lhe deu mais quatro, e é o `Falloff` que torna o *die* exprimível. Restam **4 P1** (substeps · speed limit · probabilidade · teto de população) |

#### W1-A ✅ — os três geradores consomem `accel`  ·  *o segundo desbloqueador*

`motion.verlet_rope` · `motion.soft_body` · `motion.boids` liam **nenhuma** coluna. Agora leem
`accel`, e isso entrega **de uma vez** a família `force.*` inteira a uma corda, a uma gelatina e a
um bando: gravidade com DIREÇÃO, vento, curl (= o *wander* do Reynolds), atrator, vórtice, arrasto.
A fiação já existia, o vocabulário já existia; faltava a leitura.

- **Entra onde a aceleração já entrava**, então não há kernel novo: `p += a·dt²` no Verlet e na
  predição do shape matching, `v += a·dt` como quarto termo de steering no boids — antes do clamp
  de `max_speed`, que é o que faz o `force.curl` ler como *wander* e não como empurrão.
- **Consumido, nunca reemitido** (os três emitem o próprio estado e mais nada), então todo tique
  começa de aceleração zero — a disciplina que o `motion.integrate` já declara.
- **Zeros são a IDENTIDADE** ⇒ uma simulação que nenhuma força alcança é **byte-idêntica** à que
  shipou. ⚠️ E a associação importa: `p.gravity * dt * dt` é `(g·dt)·dt`, então re-agrupá-lo como
  `g·dt²` **move o ulp de toda corda que já pendurou** — o `dt2` só serve ao termo NOVO.
- **A metade do ADR-0155 viaja no mesmo commit**, e não é higiene: sem a `Coupling::Consumes`, o
  diagnose vê um `Produces("accel")` órfão e oferece **inserir um `motion.integrate`** — que é
  `Temporal`, carimba `sim_t = playhead` de passagem, e entregaria `dt = 0` ao gerador,
  **CONGELANDO** a simulação. Declarar o consumo e ensinar o diagnose a vê-lo são a mesma mudança.
- ⚠️ **O boids tem kernel de GPU, então a leitura é DUPLA** (`read_state_accel` no WGSL, binding
  `Consume` na porta de estado) — shipar só a CPU seria a divergência que o T5 catalogou.

**Duas coisas que a MEDIÇÃO corrigiu, e as duas ficam escritas:**

1. **A barra do gate da gelatina era um palpite meu.** O cisalhamento **satura** (o corpo cede até
   a restauração elástica equilibrar o vento): `5 → 0,073` · `20 → 0,282` · **`60 → 0,743`** ·
   `180 → 1,348`. A barra é `0,3` sobre a medição de `60`, com o modo de falha medindo zero.
2. ⚠️ **Os quatro gates de paridade do boids NÃO CONTINHAM O FENÔMENO.** Eles montam
   `boids → output` com o self-loop nu, então `accel` está **ausente nas duas rotas** e elas
   concordam sobre um termo que nenhuma avalia: apagar a linha do WGSL deixa os quatro **VERDES**,
   com a CPU levando o vento e a GPU não. Fixture nova com a força na cadeia (`2,4e-7` no device),
   e a mutação sangra **só** ela.
3. ⚠️ **E uma afirmação MINHA sobre `Consume` era FALSA** — eu escrevi que trocá-lo por `Read`
   seria pego por um gate. Não é: a supressão de ride-through corre pela porta BASE, e a porta 0
   do boids é `VALUE` contra um output `INST_VEC2` ⇒ `rides_base = false`, a saída nasce vazia e
   `Consume` remove uma coluna que nunca esteve lá. Medido, não suposto. A palavra fica (é a certa,
   e é a que o `motion.integrate` usa), **sem gate**, porque um gate que não pode falhar pelo
   motivo que alega é pior que nenhum. Um gate de dois passos foi escrito para pegá-lo e
   **descartado** quando a mutação sobreviveu a ele também.

#### W1-B ⏳ — o `spread` do `motion.collide`: o defeito era MAIOR, e a cura é OUTRA

A conferência classificou isto como *"entrada descartada"* (`spread_amount = vals.first()`
colapsa uma coluna por-instância) e prescreveu *"ler a coluna"*. A verificação achou uma coisa
pior e mudou a cura:

⚠️ **A GPU já lia por elemento** (`read_spread_v(i)`), e a CPU não — logo isto não era entrada
descartada, era **DIVERGÊNCIA CPU×GPU não gateada**. Medido com um `value.instance_field` em modo
Ramp na porta: a CPU devolve a grade **INTOCADA** (o `vals[0]` é `0` ⇒ raio `0` ⇒ a identidade
precoce dispara) enquanto o device empurra — **área 1,5625 contra 2,2039**, pior `|Δpos| = 1,65e-1`.
Não é ε: são dois desenhos. ⚠️ **E as fixtures irmãs não continham o fenômeno** (alimentam `spread`
de comprimento 1 ou nenhum), então os gates estavam todos verdes por cima.

⚠️ **O caminho por-elemento do device era ACIDENTE, não desenho:** a doc das bindings dele declara
*"broadcast (ausente ⇒ 1)"* — a *respiração* animável —, e `ColumnAccess::ReadBroadcast` só faz
broadcast com **um** valor; com N ele devolve `in[i]`. A lei que o acidente produzia é ainda
**assimétrica** (`min_dist = 2·r_i`, o raio de quem olha), então dois discos discordariam sobre
estarem se tocando.

**Fechado agora:** o device honra a intenção que ele mesmo declara (`read_spread_v(0u)`) ⇒ as duas
rotas são **bit-exatas** (`0e0`) e **nenhuma cena se move** (a CPU era a referência e não mudou).

**Aberto, com o desenho escrito — W1-B:** o raio **POR ELEMENTO** é capacidade real e desejada
(o `pscale` do Houdini POP Interact; o tamanho do clone no Push Apart do C4D), e a lei honesta é
**`r_i + r_j`** — simétrica, e **byte-idêntica à de hoje sob spread uniforme** (`r+r = 2r`). O que
a torna wave e não linha é o device: com raios variáveis o alcance da grade tem de ser limitado
pelo raio **MÁXIMO**, ou seja um `Max` reduce sobre a coluna — e **nenhum kernel do repo combina
`register_grid` com `reduces()` hoje**. É máquina nova, com paridade própria.

**O que SOBRA da família 3** (os ~7 P0): o teto de 2 000 do boids (o caminho da CPU definindo o do
device — §0.0) · `max_force` · `pressure` e os clusters do soft body · o `bend stiffness` da corda ·
e o raio por-elemento do `motion.collide` (**W1-B** acima — o defeito de divergência que ele
escondia está FECHADO).
⚠️ E **dois P0 mudaram de natureza com esta wave**: a *gravidade VETOR* da corda e da gelatina
passa a ser **exprimível por composição** (uma `force.wind` em qualquer ângulo, com `gravity = 0`)
— o que era gap virou ergonomia, e é isso que um desbloqueador faz com a tabela que o precede.

**Fora da fila, com motivo escrito:**
- **FX (fam. 11)** — P0 = 0, e a cerca do módulo de pós-produção já recusa a classe (**T7**).
- **RIG (fam. 16)** — deferida por decisão sua; a tabela existe para quando for retomada.
- **DEFORMERS · TRANSFORM · VALUE · STREAM · ANIMADORES · ZERO-PARAM** — o P0 delas **é** o W0.
  Depois dele, o que sobra é P1, e P1 entra por família na segunda volta.

**Os dois DEFEITOS (T5) não entram nesta fila** — eles são conserto e andam junto da wave que
tocar o arquivo: a alfa da rampa no W3, o `spread` do `collide` no W1.

## §11.0 — O PLACAR DE HOJE (derivado, 2026-08-16)

⚠️ **A tabela da §11 abaixo é o ACHADO ORIGINAL da conferência, não o estado.** Ela registra o
que as dezassete folhas encontraram no dia em que fecharam, e desde então doze grupos shiparam.
Lida como estado, ela manda construir o construído — que é exactamente o erro que este plano
existe para não repetir.

**O estado é DERIVADO das folhas, num comando:**

```
python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"
```

| # | Família | P0 | P1 | P2 | ⏸ | ✅ fechadas | ⛔ recusadas | linhas |
|---|---|---|---|---|---|---|---|---|
| 1 | DISTRIBUIÇÃO + EMISSÃO | 0 | **6** | 9 | — | 4 | 13 | 33 |
| 2 | FORCE | 0 | 5 | 6 | — | 1 | 10 | 22 |
| 3 | SIMULAÇÃO | 0 | 3 | 5 | — | 25 | 10 | 43 |
| 4 | DEFORMERS | 0 | **6** | 8 | — | 3 | 5 | 22 |
| 5 | TRANSFORM | 0 | 4 | 3 | — | 1 | 8 | 16 |
| 6 | ANIMADORES | **1** | **8** | 15 | — | 6 | 0 | 30 |
| 7 | TEMPO / ESTILÍSTICOS | 0 | 3 | 6 | — | 14 | 7 | 30 |
| 8 | STREAM / UTILIDADE | 0 | **8** | 10 | — | 5 | 8 | 32 |
| 9 | COR / APARÊNCIA | 0 | 3 | 3 | — | 7 | 6 | 19 |
| 10 | FIELD | 0 | **6** | 5 | — | 0 | 8 | 19 |
| 11 | FX (raster) | 0 | **6** | 7 | — | 0 | 4 | 17 |
| 12 | PULSE | 0 | 0 | 7 | — | 9 | 4 | 20 |
| 13 | SIM.\* (o stack) | 0 | 0 | 7 | — | 10 | 7 | 24 |
| 14 | SOURCE | 0 | **7** | 3 | — | 3 | 0 | 13 |
| 15 | VALUE | 0 | 2 | 18 | — | 31 | 19 | 72 |
| 16 | RIG *(deferida)* | 0 | 0 | 0 | 18 | 1 | 4 | 23 |
| 17 | ZERO-PARAM + DEBUG | 0 | 1 | 7 | — | 1 | 6 | 15 |
| | **TOTAL** | **1** | **68** | **119** | **18** | **121** | **119** | **450** |

⚠️ **Este placar nasceu do grupo M, e o que ele achou vale mais que os números:** sete das
dezassete folhas **não tinham contagem nenhuma**, a folha 13 dizia `3 P0 · 7 P1` com a própria
prosa três parágrafos abaixo a dizer *"não tem item aberto"*, e a folha 06 carregava um
`**P0/P1**` que o código já tinha fechado (as lanes `x`/`y` do `motion.expression`) — a **sexta**
célula desta conferência a envelhecer. *A cura não é reconferir com mais cuidado: é a contagem
deixar de ser escrita à mão.* A linha de `**Contagem:**` de cada folha passou a ser gerada por
esta ferramenta, e diz isso de si mesma.

⚠️ **A fila ACIONÁVEL de P0 está VAZIA, e a §10.0 FECHOU.** O único `**P0/P1**` que a tabela
ainda mostra é o `motion.wave` sem N produtores. A outra metade — o **`motion.drive` sem coluna
nomeada** — fechou no **grupo P** (2026-08-16, cena `=56`): o canal `Custom…` escreve a coluna
que o artista batiza, e o `value.attribute` a lê de volta pelo mesmo nome. ⚠️ **Ele RECUSA o
device**, e é o preço nomeado: uma `ColumnBinding` precisa do nome ANTES do cook, e um nome
digitado só existe DURANTE ele — o mesmo recuo da `Median`. *A W0-B-genérico deste plano previa
«o análogo de ESCRITA do `StreamOp::Project`, maquinaria de sequenciador com pipes e paridade
próprios»; medido, isso é o preço de o CUSTOM correr no DEVICE, e não o de ele existir.* Os 18
⏸ são a família RIG, deferida.

---


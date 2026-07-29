# A POLIA — plano de redesenho

> Report do Enio, 2026-07-28, com foto. Estado: **W0/W1/W2/W3 fechados e smokados**
> (o W3 é a cena 61; o smoke aprovou a simulação e achou o **tremor do gizmo**, que
> fechou — §W3, e era só-desenho).
> O tracker da linha é [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md); o mapa de
> waves é [`00_plano_waves.md`](00_plano_waves.md). Este doc é o **porquê** do redesenho.

## 1. O que o artista pediu

Oito pontos, numa sessão de smoke:

1. o número de Ratio não aparece na caixa;
2. criada pelo usuário, a polia **não funciona** (foto: as duas cordas convergindo
   num ponto longe dos corpos);
3. não há **diâmetro** de roldana, nem representação dela, nem rotação — *"melhor
   selecionar diâmetros e não Ratio"*;
4. não dá para escolher o **número** de roldanas, nem acrescentar depois;
5. a corda passa no **centro** da roldana e não na superfície externa;
6. não dá para **selecionar e posicionar** uma roldana — cada uma precisa de um
   ponto central (deslocar) e um ponto no raio (tamanho);
7. um algoritmo tem de descobrir, por roldana, se a corda passa **por cima ou por
   baixo**;
8. **motor** em cada roldana; **break force** no início e no fim da corda e em cada
   centro de roldana.

## 2. W0 — as quatro correções (FECHADO, smoke OK)

Os quatro defeitos da foto eram a mesma família: **o 8º tipo chegou e N
consumidores não foram ensinados**, cada um *enumerando ou inferindo* em vez de
perguntar a uma porta.

| # | defeito | causa | cura |
|---|---|---|---|
| B3 | "nada funciona" | o gesto de criação pelo canvas nasce `anchored: true`, e o semeio do RIG estava atrás do MESMO sentinela | o gesto estabelece a geometria autorada INTEIRA, pela porta única `pulley_rig` |
| B2 | o círculo gigante | o anel de comprimento perguntava `length.is_some()`; numa polia o `length` é a corda inteira, não um raio | `length_is_a_radius()` — a porta já existia, aplicada a UM dos dois consumidores |
| B1 | `0 / 0 N` permanente | a view nascia com `break_force: 0.0`, e o leitor decide por `is_finite()` | `∞` é o que "não parte" É; e o readout pergunta `can_break()` |
| B4 | Ratio sem valor | a row estava **morta**: faltavam registro, sync, rota, variante e campo | as cinco metades + o gate estrutural |

**Dois gates estruturais** ficaram, e são o que sobrevive ao redesenho:

- `every_number_row_the_section_paints_is_seeded_synced_and_routed` — a lista de
  rows **não é escrita à mão**: é a diferença entre o que o Inspector pinta com
  uma joint selecionada e sem nenhuma, menos os chips enumerados. Uma row nova
  entra na varredura sem ninguém lembrar dela.
- `each_kind_draws_only_the_annotations_it_uses` — exaustivo sobre
  `JointKind::ALL`, com oráculo por **diferença** (liga a entrada, conta os paths).

## 3. A física, e por que o `ratio` sai

⚠️ **`ratio` descreve uma corda que não existe.** Numa corda única sobre roldanas
livres a tensão é **uniforme**, logo os dois corpos sentem a MESMA força e a
vantagem mecânica é **1**, quaisquer que sejam os diâmetros. O `l1 + r·l2 = L0`
que o W-Pulley shipou é, na verdade, **uma talha diferencial com o eixo
invisível** — dois tambores de raios diferentes no mesmo eixo, sem os tambores.

Então a intuição do artista está certa e o motivo é mais forte que ergonomia. Com
roldanas de verdade a vantagem mecânica volta por onde ela vem no mundo:

- uma roldana **montada num corpo que se move** (a cadernal móvel de uma talha) —
  o corpo passa a ser sustentado por DOIS ramos de corda;
- um **tambor dirigido** (o guincho), onde `v = ω·r` e o diâmetro é o câmbio;
- e o `ratio` reaparece como **quociente de dois diâmetros** num eixo acoplado (a
  talha diferencial de Weston, `2R/(R−r)`) — que é exatamente o que o pedido (3)
  descreve.

## 4. A espinha: **uma roldana é uma ENTIDADE**

É o argumento do W3 desta mesma linha, palavra por palavra: *um joint guardado NO
corpo só pode ser um por corpo*. Uma roldana guardada **dentro** do
`PhysicsJoint` tem o teto **2** — e quatro dos oito pedidos caem junto com esse
teto:

| pedido | sai de graça da entidade |
|---|---|
| (4) nº de roldanas, em tempo real | é um `spawn` — Hierarquia, nome, delete, undo e save |
| (6) selecionar e posicionar | ela **tem `Transform`**: a posição é o gizmo que já existe |
| (8) motor por roldana | campo no componente dela, não um 9º campo do joint |
| (8) break por centro | idem |

```rust
pub struct PulleyWheel {
    pub rope: u64,      // stable_name_id do joint-corda
    pub order: u16,     // posição ao longo da corda, A → B
    pub radius: f32,    // metros
    pub side: WrapSide, // Auto | Over | Under
    // W2: motor_*, break_*
}
```

⚠️ **O `Transform.translation` É o centro** — nada de um segundo campo de posição
para discordar dele. `PhysicsJoint` continua `Copy` (nenhum `Vec` dentro) e
**perde** `wheel_a`/`wheel_b`/`ratio`.

## 5. A geometria — a corda na superfície (5) e o lado (7)

Rota: `âncora A → W1 → … → Wk → âncora B`. Entre nós consecutivos, a **tangente
comum** (ponto↔círculo e círculo↔círculo; externa se os dois lados coincidem,
interna se discordam) — tudo algébrico, um `sqrt` cada. O comprimento é
`Σ tangentes + Σ arcos`.

⚠️ **O Jacobiano NÃO ganha termo de arco.** Para uma corda enrolada num círculo,
`∂L/∂centro = −(u_entra + u_sai)` **exatamente** — a variação do arco cancela
contra o deslizamento dos pontos de tangência. É esse fato que torna o W3 (talha
real) barato **e** que dá a carga de ruptura no centro da roldana: **uma conta,
dois consumidores**. Um enlace de 180° carrega `2T`; um que quase não desvia a
corda carrega ~0 — e isso se **vê**.

**O lado** é ponto fixo: chute pela poligonal dos centros → tangentes → re-avalia
o sinal do produto vetorial → repete. Duas decisões:

- **resolvido em AUTORIA, congelado no play.** Uma corda real não troca de lado da
  polia no meio da corrida sem sair da canaleta — e um lado recomputado por frame
  **pisca** perto da configuração degenerada, e um pisco muda o comprimento, e a
  corda dá um puxão.
- **`Auto | Over | Under`** por roldana. O algoritmo erra; a lição que a linha do
  Flip pagou é que ele precisa do escape manual ao lado.

⚠️ **`libm::atan2f`, nunca `f32::atan2`** — o arco precisa de um ângulo e este
número alimenta o `physics_ecs_c9` (a lei 6, a mesma do `libm::sincosf` do
W-AreaFrame).

## 6. O motor (8): uma roldana dirigida é um GUINCHO

Uma linha no kernel: **o motor muda o comprimento de repouso a `ω·r`**. Recolher
encurta `L0` e ergue; pagar corda alonga e desce, com a corda ainda segurando (a
desigualdade `λ ≥ 0` fica intacta, então nada é empurrado). `max_force` é o teto
de `λ` = a tensão que aquele motor sustenta. E o diâmetro vira o câmbio,
visivelmente.

⚠️ Vários motores na mesma corda: as taxas **somam** (degenera certo para um só).

## 7. A ruptura (8)

Por ponto de amarração:

- **início e fim** (nos corpos): a carga é a tensão `T = λ/dt`;
- **centro de roldana**: a **resultante** `|T·(u_in + u_out)|` — a mesma conta do
  Jacobiano.

Romper em qualquer ponto **solta** o que estava preso: numa ponta, a corda
inteira; numa roldana, ela **sai da rota** ⇒ o caminho encurta ⇒ `C < 0` ⇒
**folga**, e a carga cai. **Sem estouro, por construção.**

## 8. A rotação (3)

`ω = v_corda / r`, com o sinal do lado — **a roldana grande gira mais devagar, e
isso se vê**. Desenhada com um **raio-guia**, pelo precedente do W2a (*"sem ele um
círculo rolando é idêntico a um parado"*).

⚠️ O ângulo é **estado vivo, mora na ponte** — nunca no componente (ângulo
serializado = um passo de undo por frame, a lei do W1); o replay do scrub o
reintegra sozinho.

## 9. As waves

| wave | entrega |
|---|---|
| **W0** ✅ | as quatro correções + os dois gates estruturais |
| **W1** ✅ | a roldana é entidade (com raio) · rota de N nós com tangentes e arcos · lado automático + override · kernel generalizado · desenho da roldana, da corda na superfície e do giro · "Add Wheel" · `ratio` aposentado · `PROJECT_SCHEMA` 40→41 |
| **W2-A** ✅ | **o motor** — a roldana dirigida é um guincho; `ω·r` encurta `L0`, o diâmetro é o câmbio |
| **W2-B** ✅ | **a ruptura** — a corda parte pela tensão, o eixo cede pela resultante, e o readout do overlay volta a dizer alguma coisa |
| **W3** ✅ | **a talha de verdade** — a roldana montada num corpo, e a vantagem mecânica de volta |
| **W4** ✅ | **o DIFERENCIAL** — uma roldana com DOIS raios, e a vantagem mecânica CONTÍNUA que cai do quociente deles |
| **W5** ✅ | **a COMPOSIÇÃO** — o tambor e a cadernal na mesma corda: as duas vantagens MULTIPLICAM (e a nota da Weston era falsa) |
| **W6** ✅ | **as ALÇAS que faltavam** — o segundo diâmetro vira agarrável, e re-colocar o eixo de uma roldana montada deixa de ser gesto morto |

⚠️ **Âncora de regressão do W1:** a polia de hoje é o caso especial *2 roldanas,
raio 0, estáticas* — os gates atuais têm de ficar **verdes**, e é isso que prova
que o geral não quebrou o particular.

### W4 — o TAMBOR DIFERENCIAL (FECHADO, pendente de smoke)

O `ratio` que o W1 aposentou descrevia *"uma talha diferencial com o eixo
invisível"* (§3): um número sem peça na cena. O W4 põe a peça.

⚠️ **A leitura ingênua é IMPOSSÍVEL, e é ela que decide o desenho todo.** Duas
roldanas **concêntricas** na rota não têm tangente comum — a existência exige
`|C₂−C₁| > |s₂r₂ − s₁r₁|`, que centros iguais nunca satisfazem — então a rota
inteira seria recusada e a corda **sumiria da tela**. Um eixo é UM nó. Logo:

> **um tambor diferencial é UMA roldana com DOIS raios** — o que a corda ENTRA e
> o que ela SAI.

Sem referência cruzada entre roldanas, sem topologia nova, sem caso especial.

**A lei é uma linha.** Girar o eixo de `dθ` recolhe `r_in·dθ` de um lado e paga
`r_out·dθ` do outro, então `r_out·Δl_in + r_in·Δl_out = 0`. Normalizado pelo lado
de entrada, o trecho de saída vale `gear = r_in/r_out` no orçamento da corda — e a
**vantagem mecânica É esse número**.

**O kernel quase não muda, e a razão é a do W3:** `End::k` é a forma quadrática
`vᵀM⁻¹v` e `End::rate` é uma projeção, então nenhuma das duas pede versor. O peso
cavalga no vetor (`dir_b * weight_b`); a `Tangent` carrega o peso acumulado; e o
`wheel_jacobian` — porta única que o impulso do eixo montado **e** a carga de
ruptura já usam — pesa os dois lados sozinho.

⚠️ **E o W4 FALSIFICA uma premissa escrita no próprio código.** O `break_force`
justifica ser um número só afirmando que *"a corda é inextensível, logo a tensão é
uniforme"* — verdade enquanto ela **desliza**, e o diferencial é exatamente onde
ela não desliza: os dois lados carregam `T` e `T·gear`. O limiar passou a comparar
contra o **pico** (`weight_max`), senão uma corda com engrenagem 4 aguentaria
quatro vezes o que o artista dimensionou, em silêncio. O **eixo** segue recebendo a
tensão BASE: o Jacobiano dele já carrega os pesos, e o pico contaria duas vezes.

**MEDIDO** pelo caminho do produto, contrapeso de 1 kg:

| r_saída | R/r | −20 % | previsto | +20 % |
|---|---|---|---|---|
| 0,500 | 1,00 | **+0,503** | −0,040 | −0,482 |
| 0,250 | 2,00 | **+0,260** | −0,089 | −0,393 |
| 0,125 | 4,00 | **+0,083** | −0,122 | −0,309 |
| 0,100 | 5,00 | **+0,041** | −0,130 | −0,285 |

O sinal vira na carga prevista em **toda** linha. Custo: **3,489 (comum) vs 3,532
ms/tique** para 50 sarilhos — a engrenagem é uma multiplicação, e o número diz isso.

⚠️ **A primeira sonda BISSECCIONAVA a carga de equilíbrio e a medição a
derrubou:** o sistema **não é monótono** na carga — muito acima do equilíbrio o
contrapeso leve é arremessado até o tambor e a rota degenera, então *desce* volta a
virar *sobe* lá em cima e a bissecção caminhava direto para o teto do intervalo
(40 kg em toda linha). A tabela pergunta no lugar certo.

**A autoria:** `PulleyWheel.radius_out` (`0` = roldana comum), row **Out Radius
(m)** na §13 — **sempre pintada**, porque ela é o único gesto que CRIA um
diferencial e um controle que só aparece depois da coisa existir não pode ser o que
a faz existir — e o **SEGUNDO ANEL** no overlay, sem o qual o número viveria só no
Inspector, que é a queixa que aposentou o `ratio`.

⚠️ **O gate estrutural da §13 pegou dois defeitos meus no minuto em que a row
nasceu:** a contagem de caixas (5 onde a lista dizia 4 — **respondido**, não
silenciado) e, na rodada seguinte, que a caixa era **write-only** (eu esqueci o
`sync_wheel_fields`, então digitar funcionava e re-selecionar mostrava `0`). É o
mesmo gap que a família de zonas pagou uma vez; aqui não sobreviveu a um commit.

`PROJECT_SCHEMA` **44→45** (campo apendado, postcard posicional) · c9 **94→96
corpos**, `7cb7728d44…` (debug ≡ release). ⚠️ A lane antiga ficou **comum** de
propósito: separá-las é o que a mantém provando que o W4 não mexeu no que já
existia — com só ela o hash fica em `52767c92f7…`, byte-idêntico ao do W3.

**11 gates, 9 mutações, 9 sangram.**

⚠️ **A cena 62 nasceu errada DUAS vezes, e as duas a medição derrubou:** o
`Transform` de uma corda é a **âncora em A**, não o lugar do tambor (pô-lo no
tambor amarra a corda a um mastro invisível: os dois corpos foram arremessados a
y=20,5 e y=−48,0, e as duas cargas SUBIRAM); e depois disso o contrapeso leve
**alcançava** o tambor — o degenerado do W1 —, a rota degenerava e a carga caía
livre pelo chão. Encurtar a queda limita a velocidade; subir o tambor só adia.

### Aberto no W4, nomeado

- ~~**O segundo diâmetro não tem alça no canvas**~~ — **FECHADO no W6** (`WheelRimOut`,
  id 971), e a conversa que ele esperava já tinha resposta: a 2ª alça de âncora do joint.
- ~~**A talha de WESTON (`2R/(R−r)`) sai por COMPOSIÇÃO**~~ — **FALSO, e o W5 mediu.**
  Sai uma composição, e ela vale **`2R/r`**. Ver §W5.
- **O arco do enlace conta pelo raio de ENTRADA** (a corda abraça o tambor em que
  chegou). Num diferencial ele se reparte entre os dois diâmetros; o que ele
  acrescenta é quase constante e o `L0` o absorve, então quem move a carga — os
  trechos livres — está pesado exatamente.

### W5 — a COMPOSIÇÃO (FECHADA, pendente de smoke)

O W3 e o W4 shiparam com gates fortes e **nenhuma fixture os pôs na mesma corda**.
O `wheel_jacobian` de um eixo montado passou a pesar os dois ramos pelo peso da
engrenagem, e todo eixo montado do repo vivia numa rota de peso `1`: aquela
multiplicação nunca rodou com outro número. **Uma multiplicação por um que ninguém
viu falhar é uma multiplicação não medida.**

**MEDIDO** (`measure_pulley_composition.rs`; contrapeso de 1 kg, tambor `R = 0,5`,
cadernal móvel na carga; cada linha testada a −20% e +20% do previsto):

| `r` de saída | engrenagem | previsto `2·R/r` | −20% | +20% |
|---|---|---|---|---|
| — (comum) | 1 | **2** | 1,6 kg sobe | 2,4 kg desce |
| 0,250 | 2 | **4** | 3,2 kg sobe | 4,8 kg desce |
| 0,125 | 4 | **8** | 6,4 kg sobe | 9,6 kg desce |
| 0,0625 | 8 | **16** | 12,8 kg sobe | 19,2 kg desce |

As duas vantagens **MULTIPLICAM**: a cadernal dá 2, o tambor dá `R/r`, e 1 kg
chega a segurar 16 kg sem que ninguém digite um "16".

⚠️ **A nota do W4 sobre a talha de WESTON estava ERRADA, e o modo de errar é
instrutivo.** A Weston vale `2R/(R−r)`; esta composição vale `2R/r`. As duas
**coincidem exatamente em `R = 2r`** — e o exemplo natural com que a nota foi
escrita (`0,5 → 0,25`) é justamente esse ponto. Uma fórmula conferida num único
exemplo é uma fórmula não conferida.

A Weston de verdade precisa que **o mesmo eixo seja atravessado DUAS vezes com a
cadernal no meio**: um contato de passagem (que transfere corda de um lado para o
outro) *mais* a outra ponta enrolada no eixo por um raio diferente. São **duas**
equações de não-escorregamento, e eliminar o `θ` delas deixa **duas** restrições
escalares. O nosso nó é um contato de passagem cujos dois lados são **adjacentes na
rota** por construção: duas equações, um `θ` eliminado, **uma** restrição — que é
exatamente o `l_entra + (R/r)·l_sai = L₀` do W4. Não é uma peça que falta; é uma
topologia diferente, e o gate mede a nossa numa fixture com `R = 4r`, onde as duas
fórmulas **não** podem ser confundidas.

⚠️ **Uma coluna da sonda previu errado e o produto estava certo:** eu esperava
`eixo / tensão = 2·(R/r)` e a medição deu **2,00 em toda engrenagem**. O motivo já
estava escrito no `apply` — `pulley_tension` publica o **PICO** (`base ·
weight_max`) e o eixo recebe a tensão **BASE**, porque o Jacobiano dele já carrega
os pesos. A razão entre os dois é o fator de **enlace sozinho**. O que multiplica
está no absoluto: em equilíbrio o eixo carrega **o peso do bloco que ele segura**
(78,9 N medidos contra 78,5 de `m·g`).

**2 gates + 3 na cena, 5 mutações, 5 sangram** — inclusive a que prova o
**CONTROLE**: com `gear()` devolvendo 4 para toda corda, as afirmações engrenadas
passam e só a linha comum morde.

⚠️ **A cena nasceu errada, e foi a MEDIÇÃO que a corrigiu, duas vezes.** (1) Eu
dimensionei o rig achando que o contrapeso sobe *metade* do que a carga cai — ele
sobe o **DOBRO** (a carga pende de dois ramos), então ele **passava do próprio
tambor** em 0,67 s e o rig inteiro entrava em oscilação. (2) Depois de curado, o
contrapeso ainda saltava acima do previsto, e a segunda medição separou as duas
explicações possíveis: subir o tambor **1 m inteiro** moveu o pico de 8,086 para
**8,100** — ou seja, o salto **não é degeneração de rota, é folga de corda**. Uma
corda só PUXA; quando a carga pousa, o contrapeso leve segue por inércia, e subir
**encurta** a perna dele. Corolário de projeto: *queda longa* e *contrapeso em
quadro* são requisitos que brigam, porque o salto é da ordem da subida presa.

**A política de UI (as quatro condições) é honrada sem uma linha de painel nova, e
isso é o ponto:** os dois controles já existem (`Out Radius` do W4 e `Mounted On`
do W3), os dois já pintam, registram e despacham, e a **metade VISÍVEL** também
está pronta — o tambor desenha seus dois anéis concêntricos e a cadernal segue o
corpo (a arena que o W3 refresca e o tremor de 28/07 curou). A quarta condição — *a
sequência leva a algum lugar* — é o que a cena 63 demonstra: **compor é uma
capacidade que já estava lá e ninguém tinha exercitado**.

Cena de smoke: **`PH2D_PHYSICS_SMOKE=63`** — dois sarilhos, MESMA carga (7 kg) e
MESMO contrapeso (1 kg), os DOIS com cadernal móvel; só o segundo diâmetro difere.
A carga engrenada **sobe 0,28 m** enquanto o contrapeso dela **desce 2,22 m**
(razão 8,05 — a corda é inextensível, e essa razão é o oráculo que não depende de
massa nenhuma); a carga comum **cai 0,75 m** até o chão.

### W6 — as ALÇAS que faltavam (FECHADA, pendente de smoke)

Duas dívidas de canvas que o plano nomeou e adiou, e que eram a mesma família:
**autorar uma roldana com o dedo em vez de digitar**. As duas já tinham
precedente — a 2ª alça de âncora de joint, e o sentinela `anchored` do
W-AnchorFollow —, então o que faltava era fazer.

**(A) O eixo de uma roldana MONTADA era um gesto morto, e o silêncio era o
defeito.** O centro de uma montada é **derivado** (`corpo · local`), e o
`sync_mounted_wheels` devolve esse número ao `Transform` a cada frame de repouso:
arrastar o dot escrevia num campo que o frame seguinte reescrevia — a alça andava
com o dedo e **voltava ao soltar**, sem erro e sem aviso. São **DOIS** sítios de
autoria (o dot de canvas · a row Position), e os dois passam agora pela mesma
porta nova, `reseat_mounted_axle`, que desarma o sentinela.

⚠️ **A exceção que o JOINT tem aqui NÃO se estende**, e a porta diz por quê: lá o
`anchored` re-deriva **DUAS** âncoras, então limpá-lo ao editar a ponta A jogaria
fora a ponta B que o artista acabou de posicionar (por isso o joint passa pela
porta de âncora). Uma roldana tem **UM** eixo — não há segunda metade a perder, e
o sentinela é a resposta certa.

**(B) O segundo diâmetro ganhou alça** (`PointHandleKind::WheelRimOut`, id **971**)
— e é a **única alça do app cujo arrasto muda quanta força a máquina faz** (`2R/r`,
o rig composto do W5). Até aqui a vantagem só era digitável, e *uma vantagem que se
digita é uma vantagem que não se descobre desenhando*.

⚠️ **Oferecida só quando existe**, e a regra tem consequência medida: numa roldana
comum ela cairia **exatamente sobre** o aro de entrada, e duas alças no mesmo pixel
são uma alça que às vezes faz outra coisa (qual delas o hit-test devolve vira
acidente de ordem de registro). A mutação que remove a regra imprime as duas na
MESMA coordenada. Ela sai do lado **OPOSTO** ao de entrada — dois raios próximos
poriam as duas a poucos pixels uma da outra — e o **piso dela é o do irmão, não
zero**: `radius_out = 0` é o sentinela de *"roldana comum"*, então deixar o arrasto
chegar lá **apagaria a própria alça sob o dedo**. Quem desliga um diferencial é a
row, digitando 0.

**4 mutações, 4 sangram — e a QUARTA produziu um gate.** O arch-gate afirmava que o
`open_drag` e o apply **CHAMAM** a porta do raio; trocar o **CORPO** dela por
`w.radius` deixava os três arch-gates verdes. *Um gate que pina a CHAMADA não pina a
RESPOSTA* ⇒ nasceu `each_radius_handle_measures_its_own_radius`, e sem ele agarrar o
aro de saída mediria o deslocamento contra o raio de ENTRADA — 0,375 m de salto no
instante do clique num tambor 0,5 → 0,125.

⚠️ **Higiene achada no caminho:** um comentário do `inspector_commits` nomeava
`reseat_joint_pivot_after_position_commit`, função que **não existe em lugar
nenhum** — o reseat é um bloco inline no `mod.rs`. Corrigido para apontar o que
existe.

**Nenhum schema, nenhum componente, nenhum contrato congelado** (`PROJECT_SCHEMA`
**45**, registro **21**); **c9 byte-idêntico** (`7cb7728d44…`, 96 corpos) — é
autoria, não solver. LOC: `joint_anchor_drag.rs` cruzou 600 e o corte é o que o
próprio arquivo já confessava num comentário (*"`joint` aqui é a RODA, não a
corda"*) — a metade da roldana saiu para o módulo **FILHO**
`joint_anchor_drag_wheel.rs` (537 + 110).

Smoke: a **mesma cena `PH2D_PHYSICS_SMOKE=63`**, cuja mensagem ganhou os dois
gestos — ela já tem um tambor diferencial E uma cadernal montada, que é
exatamente o palco das duas alças.

#### O smoke do W6 reprovou, e a causa não era a alça

*"Selecionar Geared Rope Drum não mostra três alças âmbar"* (Enio). **A medição
inocentou o publicador na primeira sonda:** com a cena 63 real e o tambor
selecionado ele devolve `["Centre", "Rim", "RimOut"]`, exatamente as três. A causa
estava a montante e era **ENQUADRAMENTO** — a câmera padrão mostra `y ∈ [−5, +5]`
(`center = (0,0)`, `height_world = 10`; o próprio `main.rs` o diz na definição do
`WORLD_HALF`) e o tambor da cena está em **y = 10**: as alças eram desenhadas
**cinco metros acima do topo da tela**.

⚠️ **E o defeito é do W5, meu:** eu subi o `DRUM_Y` de 7,0 para 10,0 para dar pista
ao contrapeso. Foi um número escolhido por motivo **FÍSICO**, e um número desses
**não sabe nada sobre o que cabe na tela** — as duas coisas têm de ser
reconciliadas por alguém, e até aqui não eram por ninguém.

⚠️ **A cena 62 tem a mesma doença e pior:** o tambor dela está em **y = 12**, e a
mensagem dela manda *selecionar o tambor e OLHAR o segundo anel aparecer*. A
instrução era **inverificável desde que foi escrita**, e ninguém notou porque o que
a cena demonstra (as cargas indo para lados opostos) acontece embaixo.

**A lei, escrita UMA vez** (`physics_smoke_pulley::outside_frame`): *uma cena de
smoke ENQUADRA o que ela pede que se olhe.* As duas cenas declaram o próprio quadro
em consts e o gate afirma que tudo que elas spawnam cabe nele — só o eixo **Y**,
porque a largura visível depende do aspecto da janela, que a cena não conhece.
**2 mutações, 2 sangram**, e cada uma reproduz o sintoma REPORTADO nomeando o
culpado (`"Plain Post" 10.0` na 63, `"Plain Rope Drum" 12.0` na 62).

⚠️ **O teto do enquadramento é COMPILE-TIME** (`const _: () = assert!(…)`):
*enquadrar* não pode virar *afastar a câmera até tudo caber*, e mexer na const para
além disso quebra o **BUILD** em vez de um teste que alguém pode filtrar.

⚠️ **Nomeado e NÃO corrigido:** as cenas 58-61 têm os tambores em `BOOM_Y = 7,0`,
também acima do topo padrão. Elas **não pedem** que se selecione uma roldana — o
que demonstram acontece embaixo —, e mexer no enquadramento de cenas já aprovadas
em smoke mudaria o que o artista já validou. A porta está pronta para quando uma
delas passar a pedir.

#### O segundo report: alça é rest-only (verdadeiro — mas ainda NÃO era a causa)

> ⚠️ **Esta seção dizia "a causa REAL" e estava errada.** O que ela descreve é um
> defeito verdadeiro, medido e corrigido — mas o sintoma sobreviveu a ele, e a
> causa está na seção seguinte. A frase ficou corrigida no lugar onde estava:
> *um doc que nomeia a causa errada custa a próxima investigação inteira.*

*"Nada visível ainda"* (Enio), depois do fix de enquadramento. **A hipótese da
câmera era um FATO medido, não a causa** — o tambor está mesmo a `y = 10` e a
câmera padrão mostra até `y = 5`, e isso precisava ser corrigido de qualquer
forma; mas não era o que escondia as alças.

**A causa:** as alças de ponto — a âncora de um joint, o centro e os aros de uma
roldana — são publicadas **rest-only** (`at_rest = !playhead.is_playing()`),
porque durante o play o overlay desenha a geometria do **SOLVER** e estas alças
autoram a **AUTORADA**. A cena 63 manda agarrar três alças e **nascia tocando**:
elas não podiam existir. ⚠️ E a mensagem dela **já se contradizia** — dizia
*"aperte B e depois PLAY"*, o que só faz sentido numa cena parada.

⚠️ **`PAUSED_SCENES` é uma ENUMERAÇÃO escrita à mão**, e as cenas de alça de joint
(43-47) estavam nela desde o começo. A minha não estava, e **nada disse nada** —
uma enumeração é exatamente o que a próxima cena nasce fora.

**E o gate de classe achou um SEGUNDO caso, pré-existente:** a **`=58`**, a cena
que *introduziu* as alças no W1. O passo 4 dela manda *"SELECIONE uma roldana…
aparecem DOIS pontos âmbar: o do CENTRO (arraste…) e o do ARO (arraste…)"* e ela
nasce tocando — ou seja **aquele passo, que é o pedido (6) do artista (*selecionar
e posicionar uma roldana*), nunca pôde ter sido smokado**.

⚠️ **Duas falhas do meu próprio proxy, as duas corrigidas antes de shipar** — e as
duas são a mesma doença, *um gate que acusa pelo motivo errado é tão inútil quanto
um que não acusa*:

1. **`"arraste"` sozinho acusou 26 cenas**, entre elas a `=52` (a MÃO), que manda
   arrastar um **CORPO** durante o play **de propósito**. Arrastar um corpo é gesto
   de play; arrastar uma alça é gesto de repouso — o par *verbo + substantivo* é o
   discriminador.
2. **`"alca"` como SUBSTRING casa dentro de `alcance`**, e acusou a `=53` por isso
   ⇒ comparação por **palavra inteira**.

**2 gates, 2 mutações, 2 sangram** — e os dois não são redundantes: o de **classe**
depende da REDAÇÃO da mensagem, o **específico** da 63 depende só da lista, então
reescrever a mensagem silencia um e não o outro.

⚠️ **Nomeado e NÃO corrigido:** seis cenas (**56, 57, 59, 60, 61, 62**) dizem
*"depois PLAY"* e nascem tocando. Nenhuma delas pede gesto de alça, então a
imprecisão é de **texto** e não quebra o que elas demonstram — e são cenas já
aprovadas em smoke.

#### E A CAUSA ERA O PINTOR: as três alças eram FILTRADAS (2026-07-29)

*"Nada ainda"* (Enio, terceira rodada). As duas correções acima eram **fatos
medidos** e nenhuma era a causa; o defeito estava no **terceiro estágio**, e o
diagnóstico veio de quatro lentes independentes sobre o caminho inteiro —
publicação · cena · seleção · desenho — das quais **duas convergiram sozinhas** no
mesmo ponto.

**`PAINT_ORDER` era um array de 5 kinds escrito à mão** sobre o qual o laço de
pintura iterava, e `WheelCentre`/`WheelRim`/`WheelRimOut` **caíam fora do filtro**:
não eram desenhados **nem registrados no hit index**. O braço de `match` que os
desenha era **código morto**, e o compilador não podia dizer uma palavra — um
`match` precisa ser exaustivo de qualquer forma, então acrescentar um variant o
satisfazia **ali** e deixava a lista intocada.

⚠️ **Havia gate dos dois lados e nenhum no meio:** `render_loop::point_gizmo`
provava que o publicador produz as três alças, `gizmo::point_tests` provava que o
pintor desenha âncoras e grips, e **ninguém afirmava que a saída de um chega à
entrada do outro** — a costura não-testada, outra vez. Corolário que o mesmo laço
escondia: como o `register` também era pulado, **o gesto de arrasto delas era
inalcançável pelo canvas**, e todo o `joint_anchor_drag_wheel.rs` estava morto por
essa via.

**A cura não é acrescentar três itens ao array — é APAGAR o array.** O rank sai de
um `match` **exaustivo** (`paint_rank`), então o próximo kind **não compila** até
dizer onde pinta; o último rank sai dos **dados**, então segue zero-alloc e um
passe por rank como sempre foi. Ordem **byte-idêntica** para os 5 kinds que já
existiam — os 11 gates antigos ficam verdes sem retoque.

**Precedência nova, cada metade com razão própria:** alça de roldana ganha da
âncora (só a roldana **selecionada** publica; toda âncora de joint publica sempre)
e o **centro** ganha do **aro** (eles só dividem pixel num zoom em que o raio é
sub-pixel, e ali *redimensionar* não quer dizer nada enquanto *mover* quer).

**E a segunda porta, que teria mordido logo depois:** `wheel_handles` exige
`show_overlay && at_rest`, e o gate de classe só cobria o relógio. `show_colliders`
nasce **`true`** e **`B` é um TOGGLE**, então *"aperte B para ver"* manda o artista
**DESLIGAR** o que ele quer ver. A forma segura é a **condicional** (*"aperte B **se**
não estiver ligado"*, cenas 44/45) ou a **declarativa**.

⚠️ **E o gate de classe tinha DOIS buracos próprios, os dois meus:**

1. **Agulha com ESPAÇO é agulha MORTA.** `HANDLE_WORDS` trazia `"aro "` e `"dot "`
   — a comparação é por **palavra inteira** e um token não tem espaço ⇒ as duas
   **não podiam casar com nada**; faltava o **plural** (`"alcas"`, que é o que as
   mensagens escrevem). Consequência medida: **a cena 63 nunca esteve na classe** —
   ela passava por já estar em `PAUSED_SCENES`, não por ser reconhecida. Com as
   agulhas corrigidas, **duas cenas novas apareceram com o mesmo defeito do
   report**: a **48** (passo 7 manda selecionar e arrastar alças de joint) e a
   **59** (passo 5 manda arrastar o aro *"com o motor ligado"* — que é literalmente
   o estado em que a alça não existe).
2. **O gate era NÃO-DETERMINÍSTICO.** A última `fn` de cada arquivo não tem uma
   `fn ` depois dela, então `body_of` engolia o começo do arquivo **seguinte** — e
   *"o seguinte"* é a ordem de `read_dir`, que o **sistema de arquivos** escolhe.
   Medido ao vivo: editar a mensagem de uma cena de roldana fez a cena **10**
   herdar a palavra que faltava e ser acusada de pedir uma alça que ela não pede.

**O invariante do gate mudou, e a versão anterior confundia uma das curas com a
regra:** não é *"a cena nasce parada"*, é ***"o artista está em REPOUSO quando o
passo da alça roda"***. Uma demo de **MOTOR** (48, 59) existe para ser vista
tocando — congelá-la ao nascer estragaria o que ela ensina para consertar um passo
só; ela satisfaz o invariante **mandando pausar naquele passo**.

**Gate novo `the_needles_can_match_something`** — o controle da própria busca
(nenhuma agulha com espaço + cada lista casa algo no corpus real). *Uma lista de
busca silenciosamente vazia é um gate que **não pode falhar**.*

**5 gates, 6 mutações, 6 sangram** — cada uma **só no seu gate**, o que é a prova
de que não são redundantes. ⚠️ **Uma sobreviveu na 1ª rodada** (pôr a instrução de
`B` de volta na 63) e foi ela que denunciou as agulhas mortas: *mutação que não
sangra acusa o gate, não o achado.*

⚠️ **Escopo NOMEADO:** ~19 cenas do módulo dizem *"aperte B para ver"* e só nas de
gesto de alça a frase é **load-bearing** (alça de ponto EXIGE o overlay; um
contorno de collider o artista vê sumir e reaperta). As outras ficam nomeadas aqui
em vez de varridas — mexer em cena de wave já aprovada, sem smoke, é churn com
risco e sem medição.

#### E O SMOKE APROVOU AS ALÇAS E DERRUBOU TRÊS COISAS (2026-07-29)

*"Apareceu!"* (Enio) — e com ele três reports, que a medição separou em três
defeitos independentes.

**(1) O TAMANHO.** *"Os círculos do gizmo estão muito grandes. Coloque no tamanho
padrão de todas as joints ou 1/4 do diâmetro atual."* A v1 desenhava no **DOBRO**
(`JOINT_ANCHOR_RING_PX * 2.0`, raio 30) sob o racional *"uma roldana é uma roda,
não um ponto de amarração"* — e essa cerca é decisão de **produto**, não de
geometria. Foi ao **padrão** — e **em duas rodadas**, porque a 1ª errou de marca: eu fui
ao **anel da âncora B** (`JOINT_ANCHOR_RING_PX` = 15) escolhendo a constante pelo
NOME (*"o anel padrão"*), e o padrão que ele aponta na 2ª rodada, com screenshot,
é o **ponto cheio da âncora A** (`JOINT_ANCHOR_DOT_PX` = 9, **diâmetro 18**):
*"veja o ponto amarelo. aquele é o diâmetro padrão dos gizmos"*. ⚠️ **Isso
reconcilia as duas opções que ele deu na 1ª rodada** — *"1/4 do diâmetro atual"*
dos 60 px originais são 15 px de diâmetro, e o padrão dá 18: o `RING` era o meio
do caminho. O desenho fica **oco** de propósito (o diâmetro é o padrão, mas alça
de roldana e âncora fazem coisas diferentes, e um disco cheio no mesmo tamanho as
tornaria a mesma marca).

⚠️ **E uma mutação SOBREVIVEU, com duas lições.** Crescer só o **desenho**,
deixando o alvo, passava pelo gate. (1) A **representação** passou a impedir a
divergência acidental: o braço de desenho lê o `half` da MESMA tabela de alvo,
então há **uma cópia do número** (o grip de parâmetro segue a exceção declarada —
desenha 6, pega 8). (2) A **mensagem do gate afirmava** que desenho e alvo andam
juntos, e ele **não pode** verificar isso — o raio desenhado não é observável por
este harness (`n_paths` conta paths, não mede um). A afirmação foi corrigida para
o que ele de fato mede: *vender cobertura que não existe é pior que a cobertura
faltar.*

**(2) OS SALTOS EXPLOSIVOS — a metade grave, e ela estava MEDIDA antes de eu
tocar em nada** (`tests/measure_pulley_radius.rs`). O `L0` da corda
(`PhysicsJoint::max_length`) é semeado UMA vez da rota que a montagem tem em
repouso e depois **congelado** por `anchored = true`. Crescer o raio **CRESCE a
rota** — o abraço é maior —, então a restrição `L(rota) ≤ L0` nascia **violada**:

| raio | rota | violação | maior salto num tick |
|---|---|---|---|
| 0,30 (controle) | 11,9650 | +0,0000 | 0,0817 m |
| 0,60 | 12,4851 | +0,5201 | 0,3097 m |
| 0,90 | 13,0581 | +1,0931 | **14,1247 m** |
| 1,50 | 14,3667 | +2,4018 | **50,4327 m** — a carga de 3 kg sai de +2 m para **+53 m** |

**É a MESMA doença que o W-AnchorFollow curou na âncora**, e a cura é a mesma lei:
***autorar re-deriva, o runtime congela***. Porta única
**`reseat_wheel_geometry`** — ela re-abre as DUAS coisas que a geometria de uma
roldana decide (o eixo de uma montada, que o `reseat_mounted_axle` já fazia, e o
`L0` da corda), porque as duas nascem do MESMO gesto: quem chamasse metade
deixaria o rig **estável e errado**. `rope_joint_of` responde *"de que corda é
esta roldana?"* uma vez, para os dois consumidores. Depois: violação **+0,0000 em
todo raio**, maior salto **0,0820** (r=0,90) e **0,0840** (r=1,50) contra 0,0817
do controle, e a carga volta a DESCER.

⚠️ **DUAS portas de autoria, as duas costuradas:** o arrasto da alça (raio e
centro) e a row **Radius/Out Radius** da §13 — esta escreve por **FILA** de
comandos, então a re-abertura vai **DEPOIS do flush**, e o `apply_wheel_edit`
devolve *"a rota mudou?"* para o chamador saber quando. ⚠️ **NÃO é "o componente
mudou"**, e a diferença é load-bearing: a cena `=59` autora o **MOTOR** com o
relógio ANDANDO de propósito, e re-derivar comprimento de corda ali prenderia a
restrição na configuração do instante.

⚠️ **MEDIDO E REFUTADO:** *"afasta a ponta da corda dos objetos"* **não** é o
desenho descolando — o vão entre a corda desenhada e a âncora é **`0.000000` em
todo raio**. O que se via era a **consequência** do arremesso: a carga saía de
quadro, e a corda ia atrás dela.

**(3) O NÚMERO QUE NÃO APARECIA.** *"Não mostra o tamanho real da corda."* A row
**existe** e o campo **é** sincado — mas sob `entity_changed`, o contrato certo
para um número que só o artista muda, e **é exactamente esse contrato que um
número derivado pelo PRODUTO quebra**: a seleção não muda quando a ponte semeia o
`L0`. Medido pela mutação que reinstala o defeito: **a row mostra 1,00 sobre uma
corda de 11,965 m**. ⚠️ **E a correção (2) PIORAVA isto se ficasse sozinha** — ela
re-deriva o `L0` a cada arrasto, então a row ficaria permanentemente velha; as
duas metades pertencem à mesma sessão. O guard é o **FOCO**, e ele é load-bearing
(caixa focada tem edição parcial que o componente ainda não viu).

**6 gates, 6 mutações, 6 sangram.** ⚠️ **A 1ª versão do arch-gate da §13
SOBREVIVEU** à mutação `if false && route_changed`, porque pinava a **PRESENÇA**
da chamada — a MESMA lição que o `wheel_radius_of` pagou nesta linha há poucas
horas, e eu a repeti: *um gate que pina a CHAMADA não pina a RESPOSTA*. Agora ele
pina a **FORMA** da guarda, e a metade comportamental (o predicado da rota) mora
ao lado do funil.

**`PROJECT_SCHEMA` fica 34**, registro fica **21**, **c9 BYTE-IDÊNTICO**
(`7cb7728d…`, 96 corpos) — a correção é de autoria e nenhuma cena do hash autora
raio.

**Re-smoke: `PH2D_PHYSICS_SMOKE=63` — APROVADO** (2026-07-29, *"Funciona muito
bem"*): alças no tamanho padrão, o aro de saída sem tranco, a row `Rope Length (m)`
acompanhando.

### O PISO — uma corda não pode ser mais curta que o caminho que ela enfia

A correção (2) acima instalou a cura numa **PORTA**, chamada pelos três gestos que
a conhecem. Mas o `L0` é derivado da rota, e *uma condição que enumera seus
leitores apodrece* — então a primeira coisa foi perguntar **quantos gestos mudam a
rota**, com o `L0` parado (sonda `tests/measure_pulley_route_gestures.rs`):

| gesto | violação | maior salto num tique |
|---|---|---|
| **controle** (ninguém tocou) | +0,0000 | 0,0817 m |
| **acrescentar** uma roldana (o botão Add Wheel) | **+2,8816** | **13,97 m** (raio 0,60: **55,45**) |
| **mover** o centro dela para o lado (commit de Position) | **+4,1908** | **25,27 m** |
| **digitar** `Rope Length = 5` numa rota de 11,97 | **+6,9650** | **46,58 m** |
| mover o centro para BAIXO | −1,3832 | 0,0813 m |
| **apagar** uma roldana | −2,1953 | 0,0785 m |

⚠️ **Três gestos não cobertos produziam a MESMA explosão** — e **um deles nunca
poderia passar por uma porta**: o delete da Hierarquia não sabe o que é uma corda,
e ensinar-lhe seria acoplar o delete genérico a este domínio.

⚠️ **A ASSIMETRIA é o desenho, e ela foi MEDIDA, não escolhida:** violação
POSITIVA explode; a negativa é **folga** e mede exatamente o salto do CONTROLE
(0,0785 contra 0,0817). Então a cura é um **PISO** — `L0 ≥ L(rota)` no estado
autorado —, nunca uma re-derivação: para baixo ela **clobbaria a row
`Rope Length (m)`**, que é editável numa polia. Quem quer a corda mais curta move a
geometria; encurtar abaixo do próprio caminho é **impossível**, não uma escolha.

**Ele mora onde a resposta já mora** — o `reconcile`, que já computa a rota para
semear: UMA derivação servindo as duas metades, em vez de um quarto chamador. E
**sem porta de relógio**, porque a rota é função do estado AUTORADO inteiro (as
âncoras saem de `world_from_local_at_pose(rest_a, …)`, os centros da pose de
repouso — o próprio código já dizia isso na linha acima), logo é constante durante
o play e a escrita é idempotente lá. Isso ainda fecha o caso *digitar-e-dar-Play-no-
mesmo-frame*, que uma porta rest-only reabriria.

**Resultado:** violação **+0,0000** nos três gestos explosivos, salto **0,0918 ·
0,0933 · 0,0817** contra 0,0817 do controle — e o caso do comprimento digitado
volta **exatamente** ao número do controle (0,0817), porque o piso devolve o `L0`
para a rota semeada: a cena fica indistinguível de uma que ninguém tocou.

⚠️ **As duas camadas NÃO são redundantes:** a porta dá o número EXATO nos dois
sentidos para os gestos que ela conhece (encolher um raio re-tensiona), o piso
garante o invariante para os gestos que ninguém enumerou. Gate cada.

**4 gates, 4 mutações, 3 sangram** — piso removido ⇒ os três números da sonda de
volta · piso vira re-derivação ⇒ a corda de 20 m clobbada para 11,97 **e** apagar
uma roldana reescrevendo o comprimento · a rota lendo a pose **VIVA** ⇒ *a corda
ESTICA durante a corrida* (14,846584 → 14,847183 no tique 2), pego **só** pelo gate
que roda o relógio. ⛔ **E uma mutação era INVÁLIDA, não um buraco:** uma margem de
segurança (`r * 1.001`) **não** caminha — o valor gravado já é maior que a rota,
então a condição fecha no dispatch seguinte. Ela ficou registrada no gate para
ninguém a repetir, e o gate foi **reescrito** por causa dela: ele vigiava
convergência (onde nada podia falhar) e agora vigia o play, onde a premissa do
desenho de fato vive.

⚠️ **Uma afirmação minha foi corrigida pela mutação:** o gate se chamava *"o piso
escreve no máximo uma vez"* e ele **não pode ver contagem de escrita** — observa o
VALOR. Renomeado, e o doc agora diz por que observar o valor é o certo: o diff do
undo também compara bytes, então uma escrita idempotente é invisível exactamente
onde ela importaria.

**`PROJECT_SCHEMA` fica 34**, registro fica **21**, **c9 BYTE-IDÊNTICO**
(`7cb7728d…`, 96 corpos) — as polias do hash são semeadas normalmente, então o piso
nunca dispara lá.

**Re-smoke: `PH2D_PHYSICS_SMOKE=63`** — clique **Add Wheel** com o rig montado, e
depois digite um `Rope Length` absurdamente curto: em nenhum dos dois o rig pode
dar tranco, e a row tem de mostrar o comprimento que a geometria de fato mede.

### A ROTA QUE NÃO RESOLVE — os guardas de degeneração, medidos

O item do §10 pedia *"cada um com decisão explícita em vez de `NaN` silencioso"*.
A medição (`tests/measure_pulley_degenerate.rs`) o corrigiu em **duas** metades:

**(1) O `NaN` já estava barrado.** `rope_route::tangent` recusa quando
`inner <= 0` e `route` pula a corda inteira — está escrito no cabeçalho do módulo,
com o porquê (*"recusar aqui é o que impede um `NaN` de chegar ao
`physics_ecs_c9`"*). Nada a construir; agora há gate pinando (o `NaN` num
`Transform` envenena o `GlobalTransform` da subárvore inteira).

**(2) Uma das três configurações NÃO degenera.** Duas roldanas sobrepostas **do
mesmo lado** têm tangente externa (`rr` é a DIFERENÇA dos raios), então a rota
existe e só encurta — medido, **9,9109 m** contra 11,9650. Quem recusa é a
sobreposição em lados OPOSTOS, onde `rr` é a soma. A lista tratava as três como
iguais.

| configuração | rota | carga (controle −0,460) | finito |
|---|---|---|---|
| âncora dentro da roldana | `None` | −1,237 | sim |
| roldanas sobrepostas | **9,9109** | −1,021 | sim |
| roldana dentro da outra | `None` | −2,910 | sim |

**E a VOLTA é limpa:** desfazer o gesto devolve a carga a **−0,460**, o número do
controle, nas três — a recusa é transitória e não deixa estado podre atrás dela.

⚠️ **O caso FEIO, e o piso o resgata:** uma cena que **nasce** degenerada (um
projeto salvo, um undo) sela o `L0` no default de 1 m com `anchored = true` — um
número derivado que ninguém pôde derivar. Medido **sem** o piso: consertar a
geometria depois **não conserta o rig** (o `L0` fica preso em 1,0 sobre uma rota de
11,965 e a carga acaba em **−6,528**, ou é arremessada a **+2,926**). Com o piso,
volta a −0,460. ⚠️ **O resíduo teórico** — um rig cuja rota seja MENOR que o default
— foi procurado e **não se materializa**: num elevador de bolso (0,8 m entre as
roldanas) a rota ainda mede **1,9621 m** e a folga é **+0,0000**.

**A metade VISÍVEL, que era o que de fato faltava:** uma corda que não roteia
**para de segurar** e o desenho era uma **reta ÂMBAR** — exactamente a figura de uma
corda que funciona. O único sinal na tela era a carga caindo. Agora ela veste o
**vermelho do não-segura**, que o vocabulário desta linha já define como *isto não
está segurando por acidente* (o comentário do `JOINT_STRAIN_RGBA`, palavra por
palavra) — e se distingue de um joint ROMPIDO pelo **estouro**, que só a ruptura
desenha, e pela corda sair reta em vez de roteada.

⚠️ **A fonte do fato é a rota derivada em REPOUSO, e isso foi MEDIDO em vez de
assumido:** a sim **não consegue** puxar uma âncora para dentro de uma roldana,
porque a corda puxa ao longo da **tangente**, que por construção fica fora do
círculo. Com um GUINCHO enrolando por 240 tiques — o único jeito de a corda
arrastar a âncora, já que subir só afrouxa —, ela chega a **0,6789 m** de um centro
de raio 0,5 e nunca entra. Logo a degeneração é alcançável **só por autoria**, que
acontece em repouso, onde a rota de repouso É a rota viva. **Gate próprio**, porque
se essa premissa cair a decisão de desenho tem de ser revisitada (o solver hoje
pula com um `continue` e **não publica** que pulou).

⚠️ **E uma afirmação do `views.rs` era FALSA:** *"toda view que existe descreve uma
corda que está segurando"* — o passe pula a corda degenerada e nada disso aparecia
no `active`. Corrigida no lugar onde estava.

**O fato sai da MESMA chamada que desenha** (`pulley_marks -> bool`): quem pinta e
quem colore leem a mesma resposta, então não podem discordar — e o `kind_marks`
passou a rodar ANTES da escolha de cor, com a ordem de EMPILHAMENTO preservada.
**6 gates, 2 mutações, 2 sangram** (o laço ignorando o fato ⇒ a corda degenerada
volta ao âmbar; a função mentindo que roteou ⇒ a 1ª metade cai) — as duas metades
são independentes de propósito, porque gatear só a fonte deixaria o fato correto e
a tela igual.

⚠️ **NÃO feito, e nomeado:** o **readout** da §12 de uma corda degenerada mostra
`0 N` em âmbar (a tensão É zero — honesto), sem dizer *por que*. Um texto ali quer
i18n e canal próprio; a cor já responde *"não está segurando"*.

**LOC:** `physics_overlay_joints.rs` bateu **612 > 600** com os comentários desta
wave ⇒ corte pela linha que o próprio arquivo articula — **anotação × gesto em
andamento**: `draw_band` (a banda elástica de criar) e `draw_grab` (a mola da mão)
são *"feedback de um gesto em voo, desenhado mesmo com o overlay DESLIGADO"* e
foram para o irmão **`physics_overlay_gesture.rs`** (553 + 77).

**`PROJECT_SCHEMA` fica 34**, registro **21**, **c9 BYTE-IDÊNTICO** (`7cb7728d…`,
96 corpos) — nada aqui toca o solver.

**Smoke: `PH2D_PHYSICS_SMOKE=63`** — arraste o CENTRO de uma roldana para cima da
âncora de uma ponta: a corda tem de ficar **vermelha** (ela parou de segurar), e
arrastar de volta tem de devolver o âmbar **e** a simulação.

### Aberto no W6, nomeado

- ~~**Nenhuma alça de roldana tem ÍMÃ**~~ — **FECHADO** (2026-07-29, seção
  [§O ÍMÃ DO EIXO](#o-ímã-do-eixo-de-uma-roldana-montada-w6) abaixo).

## O ÍMÃ do eixo de uma roldana MONTADA (W6)

O gesto de arrasto trazia uma isenção escrita à mão, **com o porquê**:

> *"Uma roldana sai antes do ÍMÃ, e não é atalho: o ímã cola a alça nos pontos do
> COLLIDER do corpo daquela ponta, e uma roldana não pertence a corpo nenhum — não
> há a que colar."*

⚠️ **Era verdade quando foi escrita, e o W3 a falsificou.** Uma roldana MONTADA
pertence a um corpo (`PulleyWheel::body`), o eixo dela é `corpo · local`, e aquele
corpo tem collider — logo tem os nove pontos. É a forma exata de
[[feedback_a_condition_that_enumerates_its_readers_rots]]: a frase enumerava os
donos de collider da época, e o dono novo nasceu fora da lista.

### O preço, medido ANTES de escrever código

Sonda `measure_pulley_wheel_snap` (bloco de meia-extensão `[0,60, 0,25]`):

| o que | número |
|---|---|
| candidatos de uma roldana **montada** | **9** (centro · 4 quinas · 4 meios de aresta) |
| candidatos de uma roldana de **cenário** (o controle) | **0** |
| erro de mão de 0,02 m ao mirar a quina | `local = [0,62, 0,27]`, desvio **0,0283 m** |
| … e depois de o bloco andar 3 m | desvio **0,0283 m** — ele **não decai** |
| alcance do ímã (14 px) | 0,052 m a `height_world` 4 · **0,207 m** a 16 |

⚠️ **O erro é congelado no `local`, logo permanente E invisível:** o eixo acompanha
o bloco corretamente, só não está na quina. Não há nada a jusante que o corrija, e
nada na tela que o denuncie. E o erro real da mão cai **dentro** do alcance do ímã,
que é o número que diz que a feature de fato pega o caso.

### O desenho

**Uma porta nova, uma colocação só.** `PhysicsBridge::wheel_snap_targets` é a irmã
exata de `joint_snap_targets`, e a razão de existirem duas é que a pergunta *"de
quem é o corpo?"* tem duas respostas — o joint a responde pela **ponta**
(`JointSide`), a roldana pelo **NOME** que ela cita (`PulleyWheel::body`, a mesma
chave do reconcile). A **COLOCAÇÃO** dos pontos (forma resolvida → pose de repouso →
offset do collider) é `body_snap_targets`, **uma** função: duas cópias colariam o
pino e o eixo em pontos diferentes do MESMO collider, e nada na tela diria qual dos
dois está errado.

⚠️ **A recusa da roldana de cenário passou de RAMO a ARITMÉTICA:** a porta devolve
zero e `nearest_within` de uma fatia vazia é `None`. O `if drag.kind.is_wheel()` que
pulava o ímã era justamente o que apodreceu — agora o braço `Grab::World` pergunta
UMA porta por rota e escreve o ponto colado nas duas.

### Gates

4 no kernel (`ph2d-physics-ecs::pulley_wheel_snap`) + 1 arch-gate de shell
(`pulley_wheel_handles::the_mounted_axle_snaps_and_the_snapped_point_is_what_lands`,
com controle positivo) + 1 na cena. **6 mutações, 6 sangram:**

| mutação | sangra |
|---|---|
| M1 a porta devolve 0 para roldana com corpo (a isenção antiga) | 3 gates do kernel |
| M2 o apply escreve `free` em vez de `target` | o arch-gate de shell |
| M3 a colocação compartilhada esquece o offset | o gate de offset **e** o do joint |
| M4 o ímã deixa de ser gateado no Ctrl | o arch-gate de shell |
| M5 a colocação esquece a pose do corpo | 2 gates do kernel |
| M6 a mensagem da cena cita 9 (o número da CAIXA) | o gate da cena |

⚠️ **A M2 é a que carrega o peso, e um gate só de CHAMADA não a pegaria:** computar
o candidato e escrever `free` de qualquer jeito é indistinguível de não haver ímã —
a marca do encaixe acende e a roda pousa fora dele.

⚠️ **A M3 expôs um buraco meu de FIXTURE:** a 1ª versão da suíte deixava offset e
escala em zero, então a mutação da colocação compartilhada **passava inteira** do
lado da roldana e só o gate irmão do joint a pegava. *Uma colocação partilhada
precisa de um gate independente em CADA lado* — senão o lado que não a mede vai
junto em silêncio no dia em que alguém lhe der uma cópia.

⚠️ **E a M6 é sobre a CENA mentir:** os gates do kernel medem uma **caixa** (nove
pontos) e o bloco da cena 61 é um **disco** (cinco — centro + cardinais, porque
inventar quinas ofereceria um ponto que *não está no corpo*). Uma mensagem escrita a
partir do gate errado mandaria o artista procurar quatro pontos que não existem.

**Smoke:** `PH2D_PHYSICS_SMOKE=61` — selecione `Tackle Rope Wheel 1` (a MONTADA) e
arraste o dot central **com CTRL**: ele cola nos 5 pontos do disco que a carrega. Sem
CTRL o arrasto é livre; na roldana do CENÁRIO o ímã não abre.

**Zero componente, zero id, zero schema** (`PROJECT_SCHEMA` 34, registro 21) e o
**c9 saiu byte-idêntico** (`7cb7728d…`, 96 corpos) — nada aqui alcança o solver.

### Aberto no W5, nomeado

- **A Weston não é expressável** hoje — e é topologia, não um número que falta. Ela
  pediria um nó cujos dois contatos são **separados na rota**, o que é uma segunda
  restrição por corda; não construída, e não deve ser confundida com afinação.
- **O salto balístico do contrapeso comum** é física honesta e fica **na tela**. A
  mensagem da cena o nomeia, para o artista não o ler como defeito.

## 10. O que MEDIR antes de escrever número

> **A lista FECHOU** (2026-07-29). Os quatro itens têm número; nenhum pediu mudança
> de código, e dois deles corrigiram uma NOTA em vez de um constante. Sondas:
> `measure_pulley_bias_radius.rs` · `measure_pulley_budget.rs` ·
> `measure_pulley_degenerate.rs`. Gates: `pulley.rs`
> (`the_bias_holds_its_accuracy_when_the_wheels_have_radius`,
> `the_route_cost_is_linear_in_the_wheel_count`) · `pulley_degenerate.rs`.

- ~~`PULLEY_BIAS` de novo (com raio, a geometria mudou)~~ — **MEDIDO: sobrevive.** As
  tabelas do `PULLEY_BIAS` foram medidas no modelo de PONTO (`radius: 0.0`), e com
  raio o comprimento passa a incluir os ARCOS (raio 0,3 acrescenta **0,9876 m** a uma
  corda de 6 m; raio 1,0 acrescenta 3,65) e os pontos de tangência DESLIZAM. O
  esticamento em regime saiu **idêntico a 4 decimais em raio 0,00 / 0,30 / 1,00, em
  todo β** — β=0,20 dá 0,0011 m nos três, abaixo da tolerância de repouso do rapier
  (1,3 mm). A previsão do teorema do envelope que o cabeçalho da rota afirmava
  (*a variação do arco CANCELA a do trecho*) está **confirmada por medição, não só
  escrita**. O único efeito novo do raio é um tremor minúsculo que CRESCE com ele e
  CAI com β: 0,00000 em raio 0 · 0,00003 em 0,3 · **0,00017 em 1,0** (β=0,2), 8×
  abaixo da tolerância. ⚠️ Gate com mutação dupla, e a segunda é a que importa:
  apontar o versor da perna para o CENTRO em vez da TANGÊNCIA deixa raio 0,00 e 0,30
  verdes e sangra **só em 1,00** — a linha do raio grande é que carrega o gate.
- ~~quantas iterações o ponto fixo do lado leva~~ **MEDIDO: 1**, em 18 montagens de 1 a 6 roldanas (o chute pela poligonal já É o ponto fixo);
- ~~custo por sub-passo × nº de roldanas, contra o HR-4~~ — **MEDIDO, e a resposta é
  que NENHUM cap se justifica.** O HR-4 dá **1,5 ms** a *Physics rígidos*. Uma
  roldana custa **~0,00001 ms/tique (10 ns)**, e o crescimento é LINEAR:

  | roldanas numa corda | ms/tique | % HR-4 |
  |---|---|---|
  | (sem corda) | 0,0033 | 0,22% |
  | 2 | 0,0035 | 0,24% |
  | 64 | 0,0039 | 0,26% |
  | 256 | 0,0052 | 0,35% |
  | **1024** | **0,0109** | **0,73%** |

  Para comer o orçamento inteiro com roldanas seriam **~150.000** — ninguém alcança
  isso clicando `Add Wheel`, e escrever um cap "por segurança" seria exactamente o
  palpite que o §0 do CLAUDE.md proíbe. **O que escala é o número de CORDAS** (isto
  é, de corpos): 64 cordas = 3,87% · 128 = 7,09% · **256 = 14,4%** — e a contagem de
  roldanas quase não entra (64 cordas de 8 roldanas custam 0,0552, menos que 64 de 2
  a 0,0580, dentro do ruído). ⚠️ O gate é uma **RAZÃO** (512× as roldanas por < 20×
  o custo), porque uma barra de wall-clock mediria o perfil de compilação; a mutação
  que a faz sangrar é a rota virar **quadrática** (202,7×).
  ⚠️ **Meu CONTROLE nasceu errado:** o bloco sem-corda reusava UM mundo entre as
  corridas enquanto os casos com corda reconstruíam a cena, e ele saiu **mais caro**
  que uma corda de 2 roldanas (0,0063 contra 0,0034), com a coluna de delta
  **negativa**. O controle atropelado pelo experimento, pela quinta vez nesta linha.
- ~~os **guardas de degeneração** — âncora dentro de uma roldana, roldanas
  sobrepostas, `|C₂−C₁| < |r₁±r₂|` — cada um com decisão explícita em vez de
  `NaN` silencioso~~ — **MEDIDO, e o item estava obsoleto em duas metades: o `NaN`
  já estava barrado, e uma das três configurações não degenera.** Ver §*A rota que
  não resolve* abaixo.

## 11. O que o W1 entregou, item por item

| pedido | onde ficou |
|---|---|
| 1. Ratio sem valor na caixa | **o campo saiu** — ver §3 |
| 2. criação pelo canvas não funciona | W0, `pulley_rig` como porta única |
| 3. diâmetro · representação · rotação | `PulleyWheel.radius` · o círculo do tamanho que tem · `ω = s/r` com raio-guia |
| 4. nº de roldanas em tempo real | botão **Add Wheel** na §12; tirar uma é apagar o objeto |
| 5. a corda passa no centro | a rota tangencia a SUPERFÍCIE (`rope_route`) |
| 6. selecionar e posicionar | a roda é entidade: dot de CENTRO + dot de ARO quando selecionada |
| 7. algoritmo de cima/baixo | ponto fixo automático (1 passada medida) + `WrapSide::{Auto,Over,Under}` |
| 8. motor e break por roldana | **W2** — o componente já tem onde |

(As três rows da roldana — raio numérico, ordem e lado — fecharam na W1-E,
abaixo.)

### W1-E — a §13 Pulley Wheel (FECHADA)

A roldana ganhou **seção própria** no Inspector: `Rope` (leitura), **Radius**,
**Order** e **Wrap** `Auto | Over | Under`.

⚠️ **Duas das três rows são o ÚNICO gesto que existe.** Até aqui, `Over`/`Under`
(o escape manual do pedido 7) e a `order` eram estado **autorado, serializado,
que muda a rota** — e que nenhum gesto do editor alcançava. O raio é o caso
oposto: a alça do aro já o editava, e as duas portas escrevem o MESMO campo.

Seção própria e não rows na §12 porque **uma roldana é uma entidade**: ela é o
objeto SELECIONADO quando estas rows importam, e a §12 só existe com a CORDA
selecionada. Uma seção que trocasse de assunto conforme a seleção teria um
estado de colapso descrevendo dois objetos.

Medido na cena 58 (`probe_wrap_58`): forçar `Under` na 2ª roldana do ziguezague
leva o lado resolvido de **−1 para +1**, e a corda salta para o outro lado da
roda.

### W2-A — o MOTOR (FECHADO)

`PulleyWheel.motor_speed` (rad/s, positivo recolhe) faz da roldana um **tambor**.
Uma linha no kernel: `L0` encolhe a `Σ ω·r`. A row **Motor (°/s)** entra na §13
ao lado de Radius/Order/Wrap (graus na fronteira, radianos no componente — a
convenção do motor do Pin).

⚠️ **Um alvo de VELOCIDADE não serviria, e a desigualdade é o motivo:** com
`λ ≥ 0` a corda só puxa, então um alvo que paga corda é clampado em zero e o
comprimento nunca muda — um guincho que sobe e não desce. Mexer em `L0` não tem
esse buraco, porque quem baixa a carga é a gravidade.

⚠️ **O recolhido é ESTADO VIVO** (a integral de uma taxa): mora no
`PhysicsWorld`, chaveado pelo nome estável da corda, e **entra no
`PhysicsCheckpoint`** — sem isso um scrub com acerto de ring veria o guincho de
agora dentro do mundo de então.

**Medido** (`measure_pulley.rs::sweep_the_winch`): a carga sobe a `ω·r` (razão
0,90–0,97 num segundo, a defasagem de partir do repouso) · **a massa é
irrelevante** (0,1 kg e 1000 kg sobem os mesmos 0,9624 m — a projeção tem massa
efetiva exata) · o diâmetro é o câmbio (raio dobrado ⇒ subida dobrada) · as taxas
de dois tambores **somam**, e sentidos opostos se anulam.

⚠️ **O guincho é ONIPOTENTE, e o que o limita é GEOMETRIA.** Recolhido até o
gancho alcançar a roldana, a carga orbita o eixo, `L(rota)` fica descontínuo e
`β·C/dt` a arremessa (medido: 7360 m/s). A cura é o **teto do termo de posição**
(`PULLEY_CORRECTION_LAG = 6` sub-passos de recolhimento — MEDIDO: 6 é o primeiro
valor em que o recolhimento normal fica exato, e 218× abaixo do sem-guarda). Uma
corda **sem tambor tem teto ∞**, o que mantém byte-idêntica toda cena anterior.

⛔ **Três guardas medidos e REJEITADOS** (a tabela completa vive no doc-comment
da constante): estolar por APERTO (dispara tarde e estrangula cedo) · estolar
quando o gancho encontra a roldana (*two-blocking*: o estol **piora**, 20,33
contra 30,66) · dar um collider à roldana **sozinho** (o impulso ilimitado
atravessa o contato). ✅ **Com o teto, o collider funciona** — a carga assenta na
roda, que é o que uma cadernal faz.

`PROJECT_SCHEMA` **41→42** (campo apendado a um componente = postcard posicional).
Cena de smoke: **`PH2D_PHYSICS_SMOKE=59`** (ergue · paga corda · e o par que só
difere no diâmetro, medido em **3,01×** contra os 3,00 que os raios prometem).

### W2-B — a RUPTURA (FECHADA)

O passe publica a própria tensão (`λ/dt`) e compara com um limiar em newtons,
exatamente como todo joint do rapier faz — o que revoga a exceção que o
`JointKind::can_break` carregava com a nota *"dá para acumular o λ do passe e
comparar, e isso é wave própria"*.

⚠️ **UM limiar para as DUAS pontas, e o §7 do plano pedia dois.** A corda é
inextensível ⇒ a tensão é **uniforme**, então as duas amarrações sentem a mesma
carga; dois números contra uma carga são um limite (o menor) e um controle
inerte ao lado dele. O que difere de ponto para ponto é o **EIXO de cada
roldana**, e esse ganhou limiar próprio (`PulleyWheel.break_enabled/break_force`).

**Medido** (`measure_pulley.rs::sweep_the_break`): uma carga pendurada faz a
corda ler o próprio peso com razão **0,9999** (o mesmo padrão-ouro do W-J7) · o
eixo carrega `T·|u_saída − u_entrada|`, medido em **1,99× num enlace de 180°**,
**1,43× num de 90°** e 1,12× num desvio pequeno.

**Romper é seguro por construção:** uma roldana que cede **sai da rota**, o
caminho ENCURTA ⇒ `C < 0` ⇒ folga ⇒ nenhum impulso. A carga cai; nada é
arremessado (gate com teto de velocidade, não só *"caiu"*).

⚠️ **Duas camadas, dois gates:** o passe filtra a roldana rompida (é o que mantém
o `PhysicsWorld` correto sozinho) **e** a ponte não a instala na arena — porque a
arena é a lista que o **DESENHO** lê, e sem essa metade o overlay desenharia a
corda passando por uma roldana que o solver já ignorou.

O readout do overlay volta a dizer alguma coisa: a view da polia carregava
`load: 0` e `break_force: ∞` porque nada media a reação dela — era o `0 / 0 N`
permanente que o W0 corrigiu pela metade honesta (`∞` em vez de `0`). Agora o
numerador é real. O guard `can_break()` **saiu do leitor** (uma pergunta sem
resposta NÃO é um guard que não pode disparar).

`PROJECT_SCHEMA` **42→43**. Cena de smoke: **`PH2D_PHYSICS_SMOKE=60`** — a corda
que segura, a que parte sob um TRANCO (**5202 N contra 29,4 N de peso: 177×**, a
física honesta de uma corda inextensível parando uma massa em movimento), e o
eixo que cede a **52,4 N** enquanto a corda nem sente.

### Aberto no W1, nomeado

- **A corda de uma roldana é escolhida só na CRIAÇÃO** — a §13 mostra o nome e
  não o re-escolhe. Uma roldana órfã (corda renomeada) é inerte e o diz em voz
  alta, mas a volta é apagar e refazer. É a mesma exposição de toda binding por
  nome do editor; um eyedropper de corda, se vier, é o irmão exato do da §12.
- **Um corpo que passa da própria roldana inverte o ramo** — a corda passa a
  puxar do outro lado e a cena dá um tranco. A cena de smoke evita o caso
  levantando as roldanas; o que uma polia REAL faz ali é a carga encostar na
  roda, que é contato, não corda.

### W3 — a TALHA (FECHADA e SMOKADA)

`RopeWheel` ganhou `body`/`local` e uma roldana pode estar montada num **corpo que
se move**: a *cadernal móvel*. O corpo passa a ser sustentado por DOIS ramos da
mesma corda, e é assim que a vantagem mecânica volta — **sem um número**.

⚠️ **O kernel custou uma linha, e o §5 já dizia por quê:** o arco não entra no
Jacobiano, então `∂L/∂C = u_entra − u_sai`. O eixo montado vira **mais uma ponta da
mesma restrição** — entra no `k`, no `Ċ` e no impulso, pelo MESMO `End` que as duas
amarrações usam (`End::k` é forma quadrática e `End::rate` é projeção: nenhum dos
dois pede versor). O "2" de um enlace de 180° é a magnitude daquele Jacobiano.

**MEDIDO** (`measure_pulley_tackle.rs`, bloco de 2 kg):

| contrapeso | talha (2 ramos) | 1:1 (um ramo) |
|---|---|---|
| 1 kg | **equilíbrio** (−0,009 m) | cai (−1,65 m) |
| 2 kg | **ergue** (+0,97 m) | **equilíbrio** (−0,019 m) |

Exatamente 2×. A resultante nos eixos mede **1,989–1,998 T** (o desvio dos 2,000 é
o ângulo dos ramos, que não são exatamente paralelos no rig).

⚠️ **A primeira fixture não tinha a cadernal FIXA e a medição a derrubou:** com a
ponta morta e a mão as duas ACIMA do bloco, descer o bloco alonga os dois ramos e a
mão desce `2d` junto — os dois lados liberam energia e **não existe equilíbrio**. A
tabela dizia *"desce"* em toda linha, para toda massa. A vantagem precisa da
cadernal fixa para INVERTER o sentido da mão: é isso que uma talha é.

⚠️ **`mounted_axle` é porta única.** A massa efetiva e o impulso percorrem a mesma
lista em DOIS momentos (`k` tem de estar completo antes de `λ` existir), e se cada
laço decidisse sozinho quem está montado o sistema ficaria **estável E errado**.
Medido: sem o `k` do eixo a corda **ARREMESSA** o bloco a +3,75 m (contra excursão
para cima de 0,0000 no produto) — o passe deixa de ser projeção e vira ganho.

⚠️ **O centro é refrescado na ARENA**, não numa cópia: a arena é a lista que o
DESENHO lê. E por SUB-PASSO, não por dispatch — geometria de um tique atrás puxaria
numa direção que já não é a da corda.

**A autoria** é a row **Mounted On** na §13 (conta-gotas que arma um pick de
canvas + lixeira que desmonta), com `local`/`mounted` seguindo a lei do
W-AnchorFollow: converte UMA vez contra a pose de REPOUSO, e daí em diante mover o
bloco **carrega** o eixo em vez de o deslizar por ele.

`PROJECT_SCHEMA` **43→44**; registro do `ph2d-ecs` intocado; **c9 byte-idêntico**
(`52767c92f7…`, 94 corpos) — nenhuma lane dele monta, e é isso que prova que os
campos novos são inertes sem montagem.

Cena de smoke: **`PH2D_PHYSICS_SMOKE=61`** — dois rigs com a MESMA carga (2 kg) e o
MESMO contrapeso (1 kg): a talha segura (**0,008 m** em 2 s), o controle 1:1 cai
**3,15 m**.

### O TREMOR DO GIZMO — fechado no smoke (2026-07-28)

O smoke da cena 61 aprovou a simulação e reprovou o desenho: *"o gizmo da polia
tremeu algumas vezes de forma incorreta mas sem afetar a simulação"* (Enio). Era
só-desenho, e a **forma** do defeito é a razão.

A arena de roldanas é **reinstalada por `prepare` a cada dispatch**, com o centro
de uma roldana montada derivado da pose de **REPOUSO** — o único que a colheita do
ECS conhece. O único lugar que a punha na pose **VIVA** era o laço de sub-passos,
**dentro do `step`**. Um quadro mais rápido que o tique — o caso normal a 60 Hz de
tique com o monitor à frente — não dá passo nenhum, e publicava a roldana **onde
ela foi autorada**. Medido num bloco que viaja: salto de **1,2683 m** entre um
quadro e o seguinte, crescendo com a distância percorrida.

⚠️ **O solver nunca leu esse número** — quem o lê é o pintor, e é exatamente isso
que se espera de uma lista que o solver refresca e o desenho consome.

**Fix:** `PhysicsWorld::refresh_mounted_wheels()` chamada uma vez no **fim** de
`dispatch_with_scene`, **incondicional**. Aqui, e não junto da instalação, porque
este é o único ponto por onde as **quatro** saídas passam (replay · laço de tiques
· `settle` pausado · o quadro que não deve tique nenhum): a arena publicada
descreve onde as roldanas **estão** sem ninguém ter de enumerar os ramos. Nos
ramos que dão passo é idempotente com o que o `step` já fez.

⚠️ **E ele fecha, de carona, a folga de um SUB-PASSO que este plano listava como
aberta.** O laço refresca **antes** de aplicar o passe (é o que o solver precisa),
então ao fim do `step` a arena descrevia a pose do **começo do último sub-passo**.
Medido: **4,8 mm a 22,7 mm**, crescendo com a velocidade → **0,00000 m**.

**c9 byte-idêntico** (`52767c92f7…`, 94 corpos, debug ≡ release) — a prova de que
é readout e não solver. Nenhum schema, nenhum id, nenhum contrato congelado.

3 gates (o salto entre quadros · o eixo é o do corpo ao fim do tique · arrastar
pausado leva o eixo no mesmo quadro) + 2 sondas. **3 mutações, 3 sangram** —
inclusive **mover a chamada para junto da instalação**, que cura o quadro sem
tique e reintroduz o atraso do sub-passo (0,0227 m).

### Aberto no W3, nomeado

- ~~**O eixo não tem alça própria no canvas.**~~ **FECHADO pelo W6** (2026-07-29,
  conferido por medição antes de qualquer código). A nota dizia que o dot de centro
  escreve num `Transform` que *"o próximo reconcile reescreve"*, e isso deixou de
  ser verdade quando o W6 fez o arrasto passar pela porta
  `reseat_wheel_geometry` → `reseat_mounted_axle`, que desarma o sentinela
  `mounted` e re-deriva o `local` do ponto novo. **8 gates provam** (`ph2d-physics-ecs::pulley_mount`),
  incluindo `re_authoring_a_mounted_axle_sticks` e o arrasto de corpo em repouso
  no MESMO frame.
  > ⚠️ **A nota sobreviveu ao fato por uma wave**, e é a terceira vez que esta §
  > custa isso: *uma lista de pendências velha não é ruído — ela faz a próxima LLM
  > propor construir o que existe.* O antídoto usado aqui: **rodar os gates do
  > assunto antes de acreditar na lista.**
- **Uma roldana montada num corpo KINEMATIC vira um guincho de graça** (o `end`
  não zera a velocidade do ponto, só a massa) — não medido, não gateado.

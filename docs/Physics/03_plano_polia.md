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
| **W4** | *nomeada, não escalonada*: o DIFERENCIAL — dois tambores acoplados num eixo ⇒ `ratio = r₂/r₁` emergente |

⚠️ **Âncora de regressão do W1:** a polia de hoje é o caso especial *2 roldanas,
raio 0, estáticas* — os gates atuais têm de ficar **verdes**, e é isso que prova
que o geral não quebrou o particular.

## 10. O que MEDIR antes de escrever número

- `PULLEY_BIAS` de novo (com raio, a geometria mudou);
- ~~quantas iterações o ponto fixo do lado leva~~ **MEDIDO: 1**, em 18 montagens de 1 a 6 roldanas (o chute pela poligonal já É o ponto fixo);
- custo por sub-passo × nº de roldanas, contra o HR-4;
- os **guardas de degeneração** — âncora dentro de uma roldana, roldanas
  sobrepostas, `|C₂−C₁| < |r₁±r₂|` — cada um com decisão explícita em vez de
  `NaN` silencioso.

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

- **O eixo não tem alça própria no canvas.** O dot de centro edita o `Transform` da
  roldana, que numa montada é DERIVADO — arrastá-lo é escrever num número que o
  próximo reconcile reescreve. A alça honesta editaria o `local`, e é a mesma
  conversa que a 2ª alça de âncora do joint teve.
- **Uma roldana montada num corpo KINEMATIC vira um guincho de graça** (o `end`
  não zera a velocidade do ponto, só a massa) — não medido, não gateado.

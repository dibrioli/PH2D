# A POLIA — plano de redesenho

> Report do Enio, 2026-07-28, com foto. Estado: **W0/W1/W2/W3 fechados e smokados**
> (o W3 é a cena 61; o smoke aprovou a simulação e achou o **tremor do gizmo**, que
> fechou — §W3, e era só-desenho).
> O tracker da linha é [`HANDOFF_line_physics.md`](handoffs/HANDOFF_line_physics.md); o mapa de
> waves é [`00_plano_waves.md`](00_plano_waves.md). Este doc é o **porquê** do redesenho.
>
> **O que está VIVO aqui (2026-08-18):** o pedido, a física, a espinha (*uma roldana é uma
> ENTIDADE*), a geometria, o motor, a ruptura, a rotação, a **tabela das waves** (§9), o que
> **MEDIR antes de escrever número** (§10), e as **⛔ recusas medidas** + os **abertos nomeados**
> que sobraram.
>
> **O que saiu:** o corpo de cada wave — **W1 · W2-A · W2-B · W3 · W4 · W5 · W6 · O ÍMÃ ·
> W-Weston**, todas ✅ FECHADAS —, movido **verbatim** para
> [`docs/archive/docs-2026-08-18/Physics/03_plano_polia.md`](../archive/docs-2026-08-18/Physics/03_plano_polia.md).
> É lá que se responde *"por que isto ficou assim?"*, e é lá que estão os números de cada uma.
> ⛔ Nada foi resumido — as duas metades remontam o original byte-a-byte (sha256).

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

⚠️ **As oito waves estão FECHADAS e o corpo delas foi para o
[arquivo](../archive/docs-2026-08-18/Physics/03_plano_polia.md)** — inclusive a **W-Weston** (a
talha diferencial, cena `=64`), que era a §11 deste doc. O que sobra abaixo é só o que **ainda
está aberto** ou o que foi **medido e REJEITADO**.

### Aberto no W4, nomeado

- ~~**O segundo diâmetro não tem alça no canvas**~~ — **FECHADO no W6** (`WheelRimOut`,
  id 971), e a conversa que ele esperava já tinha resposta: a 2ª alça de âncora do joint.
- ~~**A talha de WESTON (`2R/(R−r)`) sai por COMPOSIÇÃO**~~ — **FALSO, e o W5 mediu.**
  Sai uma composição, e ela vale **`2R/r`**. Ver §W5.
- **O arco do enlace conta pelo raio de ENTRADA** (a corda abraça o tambor em que
  chegou). Num diferencial ele se reparte entre os dois diâmetros; o que ele
  acrescenta é quase constante e o `L0` o absorve, então quem move a carga — os
  trechos livres — está pesado exatamente.

---

## ⛔ As recusas MEDIDAS, e os abertos que sobraram (recortados das waves arquivadas)

### W6 — a mutação que era INVÁLIDA, não um buraco

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

⚠️ **NÃO feito, e nomeado:** o **readout** da §12 de uma corda degenerada mostra
`0 N` em âmbar (a tensão É zero — honesto), sem dizer *por que*. Um texto ali quer
i18n e canal próprio; a cor já responde *"não está segurando"*.

### Aberto no W5, nomeado

- ~~**A Weston não é expressável**~~ — **CONSTRUÍDA (W-Weston, 2026-07-29). E a nota
  que este item carregava estava ERRADA na metade que decidia o custo.**

  A nota dizia: *"é topologia, e pediria uma SEGUNDA restrição por corda"*. A primeira
  metade era verdade — é topologia — e a segunda **não**: eliminar a rotação do eixo
  entre os DOIS contatos deixa **uma** restrição escalar, e ela é um orçamento
  **PESADO**, exatamente o tipo que a rota já soma. O peso é `R/(R−r)`, e com uma
  cadernal móvel abraçada entre os contatos a vantagem é `2R/(R−r)`.

  ⚠️ **E a objeção GEOMÉTRICA também caiu:** *"duas roldanas concêntricas são
  recusadas pela rota"* vale para pares **CONSECUTIVOS**, e num par de Weston eles
  nunca são consecutivos — a cadernal está no meio. A condição `|C₂−C₁| > |s₂r₂ −
  s₁r₁|` só é feita para pares consecutivos, então ela nunca é perguntada sobre o par.

  Detalhe da wave (a **W-Weston**, cena `=64`) no
  [arquivo](../archive/docs-2026-08-18/Physics/03_plano_polia.md).
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

## ⛔ Mais recusas MEDIDAS (recortadas da W2-A e da W-Weston)

### W2-A — os três guardas do guincho

⛔ **Três guardas medidos e REJEITADOS** (a tabela completa vive no doc-comment
da constante): estolar por APERTO (dispara tarde e estrangula cedo) · estolar
quando o gancho encontra a roldana (*two-blocking*: o estol **piora**, 20,33
contra 30,66) · dar um collider à roldana **sozinho** (o impulso ilimitado
atravessa o contato). ✅ **Com o teto, o collider funciona** — a carga assenta na
roda, que é o que uma cadernal faz.

### ⛔ NÃO há teto, e a medição é que decidiu

A mesma sonda varreu o peso até **131 072** (`r = 0,499996`) procurando o número que
um teto pudesse citar, e **não existe um**: nada explode, nada vira `NaN`, nada oscila
(o `axle_pair` já garante denominador estritamente positivo). O que acontece é que a
carga anda `1/peso` do que o esforço anda, e a partir de ~2 048 o movimento cai abaixo
da resolução de `f32` em `C = Σ w·l − L₀` — as duas colunas do bracket param de
discordar e a máquina deixa de ser dirigível.

| peso | `L₀` (m) | −20% | +20% |
|---|---|---|---|
| 32 | 308,7 | +0,0149 | −0,0149 |
| 512 | 4 907,6 | +0,0004 | −0,0014 |
| 2 048 | 19 624,0 | −0,0003 | −0,0008 |
| 131 072 | 1 255 808,4 | −0,0005 | −0,0005 |

**Isso é o que o DESENHO diz**, não um modo de falha: dois diâmetros a 6 µm um do
outro são um diferencial que não se vê mover. Capar o peso seria capar o desenho — o
§0 na forma exata (*nunca deixe o fallback definir o produto*) —, e o remédio é o
**readout `Gear` na §13**, não um número que contradiz as duas circunferências.

### Aberto na W-Weston (a talha diferencial), nomeado

**Aberto:** o `axle_pair` recusa **três ou mais** contatos no mesmo eixo (a eliminação
deixa de dar uma restrição) — dois diferenciais em série na mesma corda é topologia
própria, não um número · o eixo composto é **cenário** nesta v1 (montá-lo num corpo
que se move pede o Jacobiano do 2º contato no ledger, hoje um `max` entre os dois
contatos em vez da soma vetorial) · o **`radius_out` e o marcador são duas formas de
dizer "eixo composto"**, e a rota escolhe o marcador — a row impede a ambiguidade, mas
um dia isto quer ser um enum.

# A POLIA — plano de redesenho

> Report do Enio, 2026-07-28, com foto. Estado: **W0 fechado e smokado**; **W1 fechado,
> pendente de smoke**; W2 aberto.
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
| **W2** | motor por roldana · ruptura nas duas pontas e em cada centro · readouts |
| **W3** | a talha de verdade: roldana montada num corpo (a cadernal móvel) |
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

### Aberto no W1, nomeado

- **A seção de Inspector da RODA não existe** — o raio é autorável pela alça do
  aro e o `wrap` só pelo default `Auto`. As duas rows (Radius numérico, Wrap
  `Auto|Over|Under`) e a de `order` são a próxima fatia; os ids já estão cunhados
  (`INSP_WHEEL_RADIUS`/`_ORDER`/`_WRAP`).
- **Um corpo que passa da própria roldana inverte o ramo** — a corda passa a
  puxar do outro lado e a cena dá um tranco. A cena de smoke evita o caso
  levantando as roldanas; o que uma polia REAL faz ali é a carga encostar na
  roda, que é contato, não corda.

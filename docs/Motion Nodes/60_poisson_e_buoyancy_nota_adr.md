# 60 — Poisson-disc e Bóia (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-13). FILA 4 (parcial: 2 dos 4 nós).
> **Nós novos:** `motion.distribute_poisson` · `force.buoyancy`. Contrato congelado **intocado**.

## 1. `motion.distribute_poisson` — o raio é o knob, a contagem é a resposta

### 1.1 A pergunta que quase matou o nó: já não temos `motion.scatter`?

Temos, e ele **também** faz blue noise. A pesquisa começou aí, e a resposta é que os dois são a
**mesma família e perguntas opostas** — a diferença é *qual número você nomeia*:

| | você nomeia | você recebe | algoritmo |
|---|---|---|---|
| `motion.scatter` | a **contagem** | um espaçamento (o mais uniforme que N permite) | Mitchell 1991 best-candidate, `O(N²·K)` |
| `motion.distribute_poisson` | o **espaçamento** | uma contagem (quantos couberem) | **Bridson 2007** dart-throwing, `O(N)` |

Contagem exata é o que um designer quer para *doze pontos ao redor de um logo*. **Distância mínima
garantida** é o que uma *cena* quer: árvores que nunca se sobrepõem, sítios de spawn que nunca
coincidem, stipple que nunca aglomera — uma promessa que best-candidate **não pode fazer**, porque
quando você pede um ponto a mais do que cabe ele não tem onde pôr e põe perto. E Bridson é **linear**
onde best-candidate é quadrático, então este é também o que aguenta dez mil pontos.

O Blender traça a mesma linha (o *Distribute Points* tem modos Random e **Poisson Disk**, o segundo
com `distance_min`); o `scatter` do Houdini ganhou *relax iterations* pelo mesmo motivo.

### 1.2 Duas divergências deliberadas do paper

O sketch de Bridson diz *"escolha um ângulo aleatório"* — isso é `sin`/`cos`, **proibido** (HR-5).

1. **A direção do dardo é rejeitada da bola unitária, não polar.** Sortear no quadrado e normalizar
   **enviesaria** a direção para as diagonais (o canto está mais longe), então o sorteio no quadrado
   é *rejeitado* até cair dentro do disco unitário: direção uniforme, só aritmética e `sqrt`.
2. **O raio do dardo é uniforme por ÁREA, não uniforme em `[r, 2r]`.** O sketch sorteia o raio
   uniformemente, o que **amontoa** os dardos no anel interno (um anel fino de raio ρ tem área ∝ ρ).
   `ρ = √(r² + u·3r²)` é a correção aceita e gasta menos dardos na região que provavelmente já está
   ocupada.

### 1.3 O que substitui o `param_as_count`

Um nó **sem** param de contagem não tem `param_as_count` atrás do qual se esconder — quem vira o
vetor de alocação é o **raio**. Raio `0` divide por zero, e o cast `f32 as usize` **satura** (não
entra em pânico): viraria uma alocação de `usize::MAX` células.

O teto é a **grade**: uma célula de Bridson (`r/√2`) guarda **no máximo um ponto**, então *limitar as
células limita a memória E a contagem* — `r ≥ √(2·w·h / MAX_CELLS)`, com `MAX_CELLS = 2¹⁸`. Gate:
raio 0 / negativo / NaN / ∞ / 1e-30 → limitado, nunca fatal.

## 2. `force.buoyancy` — Arquimedes, e um mar pra boiar

### 2.1 A referência é a 2D, não a de fluido

O padrão-ouro aqui **não** é o FLIP do Houdini: é o que todo motor 2D convergiu — o
**`BuoyancyEffector2D` da Unity** (*surface level*, *density*, *linear drag*) — com a superfície
promovida de um **nível** para uma **onda viajante**, porque um mar plano não balança e balançar é a
razão inteira de existir do nó.

```text
surface(x, t) = level + amplitude · sin( (x − speed·t) / wavelength )   [fase em ciclos]
submersão     = clamp( (surface − y) / depth , 0 , 1 )        0 = seco, 1 = todo submerso
a  =  density · submersão · n            n = a NORMAL da superfície em x
   −  drag    · submersão · velocidade
```

É um nó **Pure-em-forma** (marcado `Temporal`: a onda lê o playhead) que **acumula na coluna
transiente `accel`** — a regra de Houdini que este módulo já fala: *microsolvers somam força, UM
integrador aplica*. Corrente **inteira** ligada, nada de contrato novo.

### 2.2 Três coisas caem de graça do modelo — e cada uma é um gate

- **Ele BOIA.** O empuxo cresce com o quão fundo a coisa está, então com a gravidade puxando `g` e
  isto empurrando `density`, o objeto **assenta onde os dois se cancelam** (`submersão = g/density`).
  Não é um piso em que se senta: afunde-o e ele volta; solte-o do alto e ele mergulha e sobe.
- **Ele balança de LADO também.** O empuxo é normal à **superfície**, não pra cima (pressão é normal
  à isóbara), então no flanco da onda ele tem componente horizontal apontando **ladeira abaixo** — o
  flutuante **cavalga a marola** em vez de bombear no lugar. A inclinação vem grátis: é o `cos` que o
  mesmo par de seno parabólico já calcula.
- **Água é grossa.** O `drag`, aplicado só à fração submersa, é o que impede o flutuante de oscilar
  pra sempre — e é o mesmo `−k·v` do `force.drag`, com a diferença de que este é **gateado pela
  submersão**: uma coisa no ar não é tocada por ele.

Corrente horizontal **não** é param: isso é `force.wind` com `angle = 0` — o mesmo argumento pelo
qual não existe um nó de gravidade separado.

### 2.3 A tolerância do gate da inclinação é MEDIDA, não frouxa

O seno parabólico erra ~0,09% em **valor**, mas a **derivada** dele é mais solta: o pior ponto do
ciclo fica **0,0812** longe do `2π·cos` verdadeiro (**1,29%** da inclinação de pico). É isso que o
gate permite (`0,085`) — e um par trocado/invertido erra por `2π`, setenta vezes mais do que isso
admite. ([[feedback_loose_oracle_hides_systematic_bias]]: tolerância frouxa esconde viés; a saída é
**medir** o erro da aproximação e cravar nele.)

## 3. A demo mudou: a neve agora cai NO MAR

O documento de boot é o da chuva (Enio: *"deixe só o grafo da chuva"*), então os dois nós entraram
**dentro dele**, não numa segunda cena:

- os **sítios de nascimento** viram Poisson (espaçamento garantido — nunca coincidem, e a linha deles
  é irregular do jeito que neve caindo é, em vez do **pente** que uma fileira de `motion.grid` era);
- o **mar** entra entre a gravidade e o integrador. Ele é **raso de propósito** (0,6 acima do leito):
  o floco chega com 1,45 s de gravidade atrás de si, **atravessa** a superfície, **encosta no leito**
  (`sim.collide`) e a água o traz de volta pra boiar e balançar enquanto derrete. **Os dois nós são
  portantes** — tire o mar e os flocos só pousam; tire o leito e um splash pesado cai do mundo.

Medido: queda 1,45 s → mergulho até **exatamente** o leito → **1,3 s boiando**, com a amplitude
decaindo de 0,36 (o assentamento) para a marola de 0,14. População estável em ~73.

### 3.1 O que a demo expôs, e que era um bug pré-existente

A faixa de nascimento era **mais larga que a região viva** na altura dela: o disco de kill tem raio
efetivo 3,23 em torno da origem, e a `y = 2,6` a meia-largura viva é `√(3,23² − 2,6²) = 1,92` — mas a
fileira do grid tinha 4,68 de largura (`x` até ±2,34). **Os sítios das pontas nasciam mortos.** A
faixa agora tem 3,2 de largura e todo floco nasce vivo.

## 4. As duas armadilhas que os gates pegaram (em MIM, não no código)

1. **O gate leu dois referenciais.** O sink é a cena **como DESENHADA** — o `motion.move` desloca a
   população por `RAIN_Y` antes do output — e as constantes do mar estão em espaço de **simulação**.
   Comparar um contra o outro mede um mar 2,4 unidades abaixo daquele em que os flocos caem. É
   [[feedback_derived_coordinate_seed_must_match_sample]] outra vez, e só peguei **olhando a
   trajetória** (o floco nascia em `y=+5,15`, não em `+2,6`) — nenhuma teoria ia me salvar.
2. **O gate do chão tinha um número mágico duplicando a constante da demo** (`-2.0 + 2.4` escrito à
   mão). Quando o leito se moveu, o literal seguiu apontando pra um trecho de água vazia — e o gate
   ficaria **verde sobre isso**. Agora ele importa a constante. E a afirmação dele (*"a neve assenta
   NO chão"*) deixou de ser verdade: ela **boia**, e o gate diz isso.

E o meu gate da inclinação da onda **assertava o flanco errado**: num seno, `x = −0,5` tem a **mesma**
inclinação de `x = +0,5` (a função é ímpar → a inclinação é *simétrica* em torno da origem). O flanco
espelhado é `x = +2`. O código estava certo; o teste é que estava.

## 5. Superfície (para o integrador)

- **Foundational tocado: NENHUM.** Duas drop-crates novas
  (`ph2d-node-motion-distribute-poisson`, `ph2d-node-force-buoyancy`), dependendo só de
  `ph2d-nodegraph` + `ph2d-node-registry`, com os leaves copiados (`hash.rs`, `trig.rs`, `accum.rs` —
  [[project_brush_along_path_satellite_not_node]]: copiar 40 linhas é melhor que criar foundational
  pra 1 consumidor).
- **`ph2d-node-registry-init` regenerado** (`cargo run -p ph2d-node-sync`) — **88 crates-nó**, era 86.
  É o conflito de merge esperado no rebase: **REGENERE, nunca resolva à mão.**
- **Shell:** `motion_demo_strobe.rs` (a demo) + `motion_state_tests.rs` (os gates). 4 constantes da
  demo viraram `pub(crate)` (`SEA_LEVEL`/`SEA_DRAFT`/`SEA_WAVE_AMP`/`SNOW_FLOOR_Y`/`RAIN_Y`) porque o
  gate **tem que ler a constante**, não redigitá-la (§4.2).
- **Contrato congelado:** `architecture_contract_surface` verde (8/2/1).
- **Aberto (FILA 4, a outra metade):** `motion.delay` (a família *History* do §1.3 — o ring É o valor
  do self-loop; note que `motion.time_remap` já resolve o atraso de uma sub-árvore **Pure** de graça,
  então o `delay` só se justifica pra atrasar o que **não é função de t**: uma simulação) ·
  `motion.path` (**precisa de DECISÃO do Enio** — o plano dizia *"integra vector.\*"*, mas o sistema
  vetorial de nós foi RETIRADO (ADR-0108) e a geometria mora em `ph2d-vec-scene`, que o cook **não
  alcança**: um nó só recebe params/inputs/playhead. Ler o documento vetorial de dentro de um nó
  exige um **canal novo** shell→cook (uma fonte "externa" nomeada), e isso é arquitetura, não
  fan-out).

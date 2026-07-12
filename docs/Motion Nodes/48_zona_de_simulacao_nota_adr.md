# 48 — **Zona de Simulação** (O4) — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **O4** (o horizonte do doc 03)
**Status:** implementado, testado (3 desenhos errados provados errados **pelos testes**), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1 — `EvalCtx` **não** é superfície congelada)
**Foundational tocado:** `ph2d-nodegraph::cook::EvalCtx::started()` (**aditivo**, campo novo em `EvalCtx`)

---

## 1. O que é

O doc 03 estudou quatro desenhos de re-entrância e escolheu **O1** (a porta `forces` do `motion.integrate`)
*"agora"*, deixando **O4 — a Zona de Simulação — como o horizonte, quando `cook_scoped` existir"*. Ele existe
(doc 11). A zona é o último degrau, e ela **não substitui** o O1: complementa por baixo (a lição que o próprio
campo do Blender documentou — *"a zona crua é low-level demais sem building blocks de alto nível"*).

Uma zona **segura um stream de estado vivo entre ticks**. A cadeia que o artista liga na porta `state` é o
**interior**: recebe o estado do tick passado, faz **qualquer coisa** com ele, e devolve.

```text
  grid ──→ init ┌──────────┐ out ──→ output
                │ sim.zone │
       state ←──┴──────────┘
         ↑                    ⊙  ← o interior recebe o estado do tick passado
  force.wind → sim.step → motion.falloff → motion.cull
```

## 2. Por que ela vale mais que a ramificação de forças que ela acompanha

O `forces` do integrate é uma zona de **fronteira implícita**: o interior só pode **ACUMULAR aceleração**, porque
o integrador é o dono do estado e tudo no ramo é `Pure`. O interior de uma zona **não é dono de nada** — e isso
transforma nós que a biblioteca **já tinha**:

- **`motion.cull` vira um KILL.** Fora da zona, cull derruba elementos *deste frame* e o frame seguinte os
  reconstrói. Dentro, o estado carrega os sobreviventes — **o que morre fica morto**. (O POP Kill do Houdini, de um
  filtro que já existia.)
- **`motion.combine` vira NASCIMENTO** — *mas só depois do doc 49*: fundir o `motion.emitter` (que é **stateless**)
  no estado a cada tick funde **as mesmas partículas de novo**. Quem calcula "quem nasceu NESTE tick" é o
  **`sim.spawn`** (doc 49); o `combine` funde. Esta linha foi escrita como fato antes de existir o nó que a torna
  verdadeira — ver a lição do doc 49.

**Nenhum maquinário novo comprou isso. A zona comprou.**

O motor por baixo é o que já estava lá: a porta `state` é um **feedback host** (a convenção do O1 — input chamado
`state`/`forces` do tipo do output 0), então o plumbing do editor é quem cria a aresta `pre` (self-loop quando a
porta está nua; na CABEÇA do interior quando há cadeia) e desenha como **badge de portal**. **Zero `Domain` novo,
zero contrato, zero UI nova.**

## 3. Os três desenhos errados (os testes mataram todos)

### 3.1 *"O estado está vazio?"* — **RESSUSCITA OS MORTOS**

Uma sim que matou o último elemento devolve um **stream VAZIO** — uma resposta de verdade, que por acaso não carrega
nada. Ler isso como *"não comecei"* faz a zona re-semear do `init`: **mate todas as partículas e a cena ressuscita,
um frame depois, para sempre.** Guarda: `a_zone_that_killed_everything_stays_dead`.

### 3.2 *"Chegou algum valor na porta `state`?"* — **SEMPRE CHEGOU**

Foi a minha 1ª correção, e o teste a derrubou em 30 segundos: o interior é ligado em `state` por uma aresta
**FORWARD**, então o cook o avalia **ANTES** da zona, e no tick 1 ele obedientemente devolve um stream vazio (o
valor ausente era o input *dele*: a saída anterior da zona, pela aresta `pre`).

**O estado de uma zona mora na saída anterior DELA MESMA** — então é isso que ela pergunta: *eu emiti algo no tick
passado?* (`EvalCtx::started()`, foundational aditivo).

### 3.3 *"A zona guarda tudo que voltou"* — **guarda o RASCUNHO junto**

O demo pegou este. A coluna **`falloff`** é uma **máscara** (o campo multiplicativo do §1.2: `force.wind`,
`motion.move`, `motion.scale` — todos multiplicam o efeito por ela). O kill da chuva é `motion.falloff` +
`motion.cull`. Guardando a máscara no estado, ela **volta pelo laço e mascara a própria gravidade que a criou** (a
franja do seed parava de cair) **e vaza para fora da zona**, escalando o `motion.move` que posiciona a cena —
esticando-a. Todos os sintomas de um estado que estava **lembrando rascunho**.

> **A zona guarda ESTADO, não RASCUNHO.** O que os elementos SÃO (`P`, `vel`, `id`, `size`, `tint`…) sobrevive; o
> que um tick escreveu para uso próprio (`accel`, `falloff`) **não**. Quem quiser máscara depois da zona, calcula
> uma — máscara é barata, e uma máscara velha não é máscara: é fantasma.

## 4. `sim.step` — e por que não é o `motion.integrate`

O integrate é **stateful**: ele guarda a sim, pareia cada elemento duma cadeia *rest* VIVA com a linha anterior
dele por `id`, e soma o deslocamento a uma posição que o resto do grafo re-autora. É o nó certo quando a sim é um
**desvio de uma animação**.

Dentro da zona **não existe cadeia rest: o stream É o estado**. Então o passo é **sem estado** — lê `vel`/`accel`,
escreve `P`/`vel` (o `Set Position` + `velocity × Delta Time` que o Blender faz você escrever à mão). Pôr o
`motion.integrate` dentro duma zona daria **duas memórias** à sim (a da zona e o self-loop `pre` dele), e elas
discordariam no instante em que um kill removesse uma linha. **Um estado, um dono: a zona.**

`dt` vem de um **relógio que o estado carrega** (`sim_t`), não de um contador de frames: elemento nunca pisado não
tem `sim_t`, logo `dt = 0` e ele simplesmente **começa**. Sem isso, uma zona solta num grafo com o playhead em 8 s
daria **um passo de oito segundos** no frame em que foi criada e atiraria a cena para o infinito. (Guarda:
`a_brand_new_element_starts_it_does_not_leap`.)

## 5. Blender: as duas surpresas, honradas por construção

1. **Zona nua CONGELA a entrada** (*"even just a simulation zone that does nothing freezes the mesh"*). É o que
   acontece aqui — e por isso a porta se chama **`init`**, não `in`: é o estado **INICIAL**, lido **uma vez**. O
   nome é a documentação. Guarda: `a_bare_zone_freezes_its_seed`.
2. **Atributos precisam "tunelar" pela zona** — **não aqui**: o interior é cozido por tick pelo cook pull-based, e
   qualquer nó dele pode ler um input EXTERNO (um LFO, um valor) normalmente. Só o **estado** dá a volta.

## 6. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| foundational | **`EvalCtx::started()`** + campo `started` (aditivo; `Cook` preenche de `prev_outputs`) |
| crates novas | **`ph2d-node-sim-zone`** (`sim.zone`) · **`ph2d-node-sim-step`** (`sim.step`) → **82 crates-nó** |
| shell | 5ª cena no documento de boot (`build_sim_zone`): a **chuva** que acelera e morre ao sair do disco |
| gates | `the_rain_accelerates_and_what_it_kills_stays_dead` (produto, no doc de boot real) |

## 7. A lição

**Três desenhos, todos "óbvios", todos errados — e nenhum deles falharia num teste que só perguntasse "a zona
roda?".** O que os matou foi sempre a mesma disciplina: escrever a guarda para a *pergunta do artista* (*matei
tudo: fica morto?* · *a chuva acelera?*), não para o código que eu acabara de escrever.

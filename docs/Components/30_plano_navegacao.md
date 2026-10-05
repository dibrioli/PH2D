# 30 — Plano: NAVEGAÇÃO (inimigos que acham o caminho sozinhos) (2026-10-01)

> Ordem do dono: *«inimigos que acham o caminho sozinhos (navegação). Comece com pesquisa … Depois faça
> planejamento detalhado.»* A pesquisa é o [doc 29](29_pesquisa_navegacao.md) (com o dossiê bruto em
> [`pesquisa/dossie_navegacao_2026-10-01.md`](pesquisa/dossie_navegacao_2026-10-01.md)); o oráculo é o
> **Godot 4.7.2 (MIT)** corrido sem interface ([`ferramentas/godot_nav_sonda/`](ferramentas/godot_nav_sonda/)).
>
> ⚠️ Este plano é o DESENHO e as PERGUNTAS. Cada wave abre com a medição que a pode desmentir, e o que ela
> desmentir é reescrito aqui **com a morte à vista** (§0.0 do roteador). Os números de contador abaixo são
> **estimativas de DELTA** contra o merge-base — ⛔ conte-os no código no dia (`PROJECT_SCHEMA` em
> `shells/desktop/src/project_schema.rs`, hoje `177`; registo da física `40`; `LIVE_SECTIONS` `43`).

---

## §1 — O que se constrói, numa frase por peça

| peça | mora | o que é |
|---|---|---|
| **`ph2d-navmesh`** | crate-folha nova; deps **só as que já estão no `Cargo.lock`**: `clipper2-rust 1.1.0` (BSL-1.0) e `spade 2.15.1` (MIT/Apache) | a LEI da construção: polígonos de entrada (contorno da região + obstáculos) → recuo pelo raio → união/diferença → triangulação com restrições → partição em convexos → `NavMesh` (polígonos convexos + adjacência por arestas-portal + ilhas) |
| **`ph2d-nav`** | crate-folha nova, **zero dependências** (o molde da `ph2d-health`/`ph2d-topdown`) | a LEI da procura e da condução: localização de ponto · projecção do que está fora · **Polyanya** (caminho óptimo em qualquer ângulo) · o **corredor** mantido · a política de recálculo · a aceitação por `velocidade × dt` · os estados com voz |
| **`ph2d-orca`** | crate-folha nova, **zero dependências** (W5) | a LEI do desvio entre agentes: ORCA (van den Berg et al. 2011) com as **arestas da navmesh como obstáculos estáticos** |
| **`NavRegion`** | CONFIG, registada (`ph2d-physics-ecs`) | onde se anda: a caixa da região (e depois um contorno desenhado com a caneta) · a máscara de camadas dos obstáculos · mostrar a navegação |
| **`NavAgent`** | CONFIG, registada (`ph2d-physics-ecs`) | quem persegue o quê: o modo de alvo (nome · a tag mais perto · ponto · patrulha) · o raio (**do corpo**, salvo se o artista o escrever) · as distâncias de chegada e de recálculo · o desvio · três nomes de sinal (*chegou · sem caminho · preso*) |
| **`NavAgentRuntime`** | **NÃO** registado (a cerca é o TIPO — o molde do `CounterRuntime`) | o corredor vivo, o ponto actual, o relógio do recálculo, o detector de «preso» — **no anel da física** |
| **a ponte** | `ph2d-physics-ecs/src/bridge/nav.rs`, **no tique da física** | reconstrói a malha quando a geometria estática muda · para cada agente: alvo → caminho → intenção → **escreve a intenção do `TopDownPlayer`** antes de ele correr · publica os sinais |

---

## §2 — ⭐⭐⭐ As decisões de arquitectura (e a medição por trás de cada uma)

### §2.1 — O agente DECIDE; quem anda é o `TopDownPlayer`

O desenho de três camadas da indústria tem um ponto em comum com a nossa casa: **a navegação não move o
corpo** — o Godot devolve o próximo ponto e uma velocidade segura, o `landmass` *«devolve a velocidade
desejada e nunca move o agente»* exactamente para conviver com uma física dona da pose (doc 29 §4). Na
nossa casa o executor **já existe e já é bom**: o `TopDownPlayer` acelera, trava, roda e **desliza na
parede à velocidade cheia** (o oráculo confirmou-o no #13), e com `default_controls = false` é *«um motor
PURO, obediente a quem lhe escrever a intenção»* (`bridge/player_channel.rs:79`).

⇒ o `NavAgent` **requer** o `TopDownPlayer` (no catálogo) e escreve-lhe a **intenção** `(drive, drive_y)` a
cada tique. ⛔ **Um quarto mover está recusado** (duplicaria a aceleração, o deslize e o anel do #13). ⭐ De
graça: mesmo que a navegação erre um canto, o corpo **não entra na parede real** — o `move_character_from`
desliza; é a primeira metade da cura da queixa Q1.

⚠️ **O modo de direcção importa e vai medido:** o `TopDownPlayer` nasce em **oito direcções**
(`ph2d-topdown/src/lib.rs:125`), que **encaixa** a intenção ⇒ um caminho em qualquer ângulo vira ziguezague.
A semente (o molde do `seed_kinematic_controller_body`) põe **`Free` + `default_controls = false`** quando o
`NavAgent` chega; se o artista voltar a pôr 8/4 direcções, o painel **diz** o preço (W4), com a medição do
ziguezague ao lado.

### §2.2 — Tudo DENTRO do tique da física (a fita obriga)

Medido no levantamento (doc 29 §5.4): a fita de entrada grava **UM** `PlayerInput` por tique e só o entrega
aos corpos lidos pelo teclado (`tape.rs:111`, :346-372) ⇒ uma intenção escrita **de fora** para um corpo
obediente **não é gravada** e um *scrub* não a repete. ⇒ a intenção do agente é **calculada dentro do
tique, a partir do estado do mundo**, pela porta única `drive_controllers`
(`bridge/controllers.rs:40`), **antes** do `drive_topdown`:

```text
drive_controllers:  drive_nav_agents → drive_players → drive_topdown → drive_projectiles   (os DOIS laços: frente e replay)
```

e o estado vivo do agente entra no **anel** com o par `record`/`seed` e a limpeza em `rebuild_from_rest`
(as três armadilhas que esta família já pagou três vezes — o censo `controllers_one_door` passa a cobrir o
quarto). ⭐ **Consequência que nenhum motor da pesquisa tem:** um *scrub* a meio de uma perseguição devolve o
agente, o caminho e o relógio do recálculo **exactos daquele tique**.

### §2.3 — A área andável é DERIVADA dos colisores; o raio é do CORPO

- **O obstáculo É o colisor** (o estado da arte 2D — `vleue_navigator`, Godot 4.3+): todo colisor **estático**,
  não-sensor, cuja camada está na máscara da região. Nada a sincronizar à mão; **nada gravado** (a malha é
  derivada como um cozido: não vai ao ficheiro nem ao `Ctrl+Z`).
- **O raio sai do colisor do PRÓPRIO agente** (`Ball` → raio; `Cuboid` → raio circunscrito; `Capsule` →
  raio + meio-segmento circunscrito), com sobreposição opcional escrita pelo artista. ⇒ a queixa Q2 (*«o raio
  não muda o caminho»*) **não se exprime**.
- **Uma malha por raio, derivada sozinha** (uma cache `BTreeMap<raio quantizado, NavMesh>`), ⇒ a queixa Q3
  (*«um mapa por tamanho»*) **não pede trabalho ao artista**. ⏳ A quantização do raio é **medida** na W1 (o
  recurso é a memória das malhas contra a fidelidade da folga).
- **Recuo REDONDO (soma de Minkowski com o disco), não em esquadria.** A sonda mostrou o Godot a deixar as
  quinas do furo **vivas** (`(140,90)–(260,210)` para um quadrado `(150,100)–(250,200)` a raio `10`) — a folga
  em esquadria é **mais** do que o disco precisa, e o caminho dá a volta mais larga. ⇒ **divergência
  DECLARADA**, com o ganho medido na W1 e gate a **exigir que ela exista** (com o recuo em esquadria a nossa
  lei tem de reproduzir o Godot; com o redondo, a área andável tem de ser **maior ou igual** e o caminho
  **menor ou igual**). ⚠️ O arco vira segmentos: o número de segmentos por quarto de volta é **medido** pela
  flecha contra o raio (a lei que o desenho vectorial desta casa já usa), nunca escolhido.

### §2.4 — A malha reconstrói-se no tique em que a geometria muda — síncrona, se a medição deixar

A queixa Q4 (*«caminho vazio no 1.º quadro»*) **foi medida no próprio oráculo**: com as iterações
assíncronas de fábrica, o Godot devolve caminhos vazios **sem erro** até ao quadro 4–5 (doc 29 §6.1). ⇒ o
nosso agente tem de ter caminho **no tique 1** — e isso só é honesto se a construção couber no tique. ⏳ **A
W0 mede o custo** da construção com `10`, `100` e `1000` obstáculos (em `--release`, com o `loadavg` ao
lado). Se não couber, a saída é **por mosaicos** (reconstruir só os tocados) — ⛔ **nunca** um corte silencioso
(o agente fica sem caminho e não diz porquê): o *«aceita e mente»* que esta casa já pagou três vezes.

**Quando a geometria «mudou»:** uma **assinatura** dos colisores estáticos (corpo, forma, pose em bits, camada)
num `BTreeMap` — o custo por tique de a calcular é medido na W0; a alternativa é a ponte marcar «sujo» nos
sítios onde já reconstrói corpos.

### §2.5 — A procura é Polyanya, com DOIS oráculos

- **Polyanya** (Cui, Harabor, Grastien, IJCAI 2017): o caminho **mais curto possível** em qualquer ângulo,
  sem pré-processamento, sobre a navmesh. O A\*+funil do Godot é óptimo **só dentro do corredor que o A\*
  escolheu** ⇒ o nosso caminho nunca pode ser mais longo que o dele.
- **Oráculo 1 — o Godot corrido:** *o nosso comprimento ≤ o dele + tolerância medida* em todo o corpus.
- **Oráculo 2 — o EXACTO, escrito por nós:** o caminho mais curto entre obstáculos poligonais é um teorema —
  ele passa pelos **vértices** dos obstáculos —, logo o **grafo de visibilidade + Dijkstra** dá a resposta
  **exacta** (cara, mas é teste). Gate: *Polyanya = o exacto* a meio ULP de comprimento. ⭐ É um oráculo de
  graça, independente de qualquer app.
- ⚠️ **Os custos por área (W7) quebram a optimalidade em qualquer ângulo do Polyanya** (ele supõe custo
  uniforme). ✅ A W7 abriu com essa medição e ela **DERRUBOU** a saída prevista (o A\* sobre polígonos + funil,
  o idioma do Detour/Godot, erra 27–43 % no p95): o produto é o **Polyanya que refracta** — §15.

### §2.6 — Determinismo (o `physics_ecs_c9` compara os três sistemas)

- `f32` com **só `sqrt`** (exacta em IEEE) nas leis do tique; ⛔ nada de `atan2`/`sin` sem `libm`.
- `BTreeMap`, nunca `HashMap`, na ponte e nas leis.
- ⚠️ **O `spade` usa `HashSet`/`HashMap`** (no `flood_fill_iterator.rs` e no `refinement.rs`; o
  `clipper2-rust` não tem nenhum fora dos testes) ⇒ **a W0 mede que a saída é bit-idêntica entre duas corridas
  em PROCESSOS diferentes** (as sementes de *hash* mudam por processo). ⛔ Se não for, a classificação das
  faces (dentro/fora) é **nossa** (ponto-no-polígono pelo resultado par-ímpar do Clipper), e o `spade` fica só
  com a triangulação; se nem a triangulação for, a partição é **nossa** (Hertel-Mehlhorn sobre uma triangulação
  própria) e o `spade` sai.
- O Clipper2 trabalha com **inteiros** (`Paths64`): o factor de escala metros→inteiros é um limite que diz de
  que recurso é (a precisão do `f32` na extensão do mundo contra o alcance do `i64`) — **medido** na W1, com a
  tabela ao lado.

### §2.7 — O desvio entre agentes respeita a malha (W5)

A queixa nº 1 da pesquisa (Q1) nasce de o desvio **ignorar a navmesh** no Godot e no RVO do Unreal. A nossa
cura tem **duas** metades: o ORCA recebe as **arestas de fronteira da malha** como obstáculos estáticos (o
RVO2 suporta obstáculos poligonais; o Godot não os liga à malha) **e** o corpo desliza na parede real. A Q7
(*ORCA preso nos cantos*, Sunshine-Hill) é um **banco de cenários automático** — cada um é um gate.

---

## §3 — O tique de um agente (a ordem é a da indústria, e o oráculo decide os números)

```text
(1) alvo        → por nome · a tag mais perto · ponto · o próximo ponto da patrulha
(2) malha       → a do raio deste agente (a cache; reconstrói se a geometria mudou)
(3) localizar   → o agente e o alvo na malha (fora dela: projecta para o ponto mais perto e DIZ)
(4) recalcular? → só se: alvo andou > distância de recálculo · o corredor ficou inválido · não há caminho
                  (⛔ nunca por tique — Q6; o relógio é escalonado por agente, declarado)
(5) caminho     → Polyanya → lista de cantos (o corredor)
(6) avançar     → consome cantos alcançados com aceitação ≥ velocidade × dt (Q5)
(7) intenção    → direcção para o próximo canto (travar perto do fim: distância de chegada)
(8) desvio      → (W5) ORCA sobre a velocidade desejada, com as fronteiras da malha
(9) entregar    → (drive, drive_y) do TopDownPlayer — e ele anda e desliza
(10) estados    → chegou · sem caminho · alvo fora da malha · preso (o progresso parou durante T)
                  — cada um é um sinal com NOME autorado (vazio = calado) e uma linha no painel
```

---

## §4 — As waves

| wave | entrega | abre com (a medição que a pode desmentir) | smoke |
|---|---|---|---|
| **W0** | o ORÁCULO e as medições: o corpus do Godot **com cabeçalho** (construção com raio · caminhos · ponto fora · desvio) e o oráculo EXACTO (grafo de visibilidade) | §5.0: a composição de hoje **não** persegue à volta de uma parede (o *homing* da arena, medido) · o custo da construção a 10/100/1000 obstáculos · o determinismo do `clipper2-rust`+`spade` entre processos · o custo da assinatura por tique | — |
| **W1** | `ph2d-navmesh`: a construção, com paridade contra o Godot (recuo em esquadria) e a divergência declarada (recuo redondo) | o factor de escala inteiro · os segmentos por arco · a quantização do raio | — |
| **W2** | `ph2d-nav`: localizar · projectar · **Polyanya** · ilhas (inalcançável ⇒ o ponto alcançável mais perto, **dito**) | o custo de uma consulta e de N consultas por tique (a tabela que decide o escalonamento do recálculo) · triângulos contra polígonos convexos fundidos | — |
| **W3** | **`NavRegion` + `NavAgent` + a ponte no tique + o anel + os sinais + o traço do caminho** | as queixas Q2, Q4, Q5, Q6, Q8, Q12 escritas como gates **antes** da ponte | **`=1` o labirinto**: o herói ao teclado, um perseguidor que **contorna** e, ao lado, o **CONTROLO** (o morcego de *homing* que bate na parede) · e um perseguidor **GRANDE** que não cabe na passagem estreita e dá a volta longa |
| **W4** | as secções **Nav Region** e **Nav Agent** do Inspector · *Show Navigation* (a malha e os caminhos no canvas) · a linha de estado com voz | o censo de que toda secção chega a pixel · o preço do 8-direcções medido | o artista monta um perseguidor **sem tabela nenhuma** |
| **W5** | `ph2d-orca`: desvio entre agentes com as fronteiras da malha · corpos móveis não-agentes como obstáculos dinâmicos | paridade com o RVO2 do Godot (velocidades por tique) · o custo a 10/100/1000 agentes (a vizinhança por grelha) | **`=2` a porta**: oito perseguidores a passar uma porta sem se entalarem, com o CONTROLO sem desvio ao lado |
| **W6** | o mundo que muda: reconstrução ao mudar a geometria · portas · verbos **`Start/Stop Navigation`** · alvos **a tag mais perto** e **patrulha por uma forma desenhada** · o exemplo patrulha→persegue com a `StateMachine` | o custo da reconstrução por mudança (síncrona ou por mosaicos) | **`=3`**: o guarda patrulha um caminho desenhado, vê o herói, persegue-o, e a porta fecha-se |
| **W7** | **custo por área** (`NavCostArea`: lama lenta, lava proibida) · **atalhos** (`NavLink`: teleporte, porta de um sentido) com o sinal *atravessou* | o Polyanya com custos (§2.5) · o custo da procura ponderada | o inimigo **evita a lava** e usa o teleporte |
| **W8** | a **arena** (`PH2D_VIDA_SMOKE=4`) com os morcegos a contornar as paredes · o **tutorial em PDF** `03_navegacao.pdf` + o gate dos rótulos | a foto da arena antes de a mandar | o dono monta uma perseguição do princípio ao fim |

⚠️ **Cada wave fecha com:** prova de mutação (com os quatro controlos e o pré-voo de âncoras), a **foto da
cena antes de a mandar** (`ferramentas/fotografa_cena.sh` — *uma cena que ensina o contrário é pior que uma
ausente*), e o portão da linha.

---

## §5 — ⭐ Onde superamos a referência (cada linha com a fonte da vantagem)

| | o que | contra | a fonte |
|---|---|---|---|
| S1 | **o caminho mais curto possível**, em qualquer ângulo | o A\*+funil do Godot, óptimo só no corredor | Polyanya (IJCAI 2017) + o oráculo exacto (§2.5) |
| S2 | **o raio sai do corpo** e uma malha por raio deriva-se sozinha | Godot/Unity/Unreal: um mapa ou *bake* por tamanho, à mão | doc 29 §2.4 Q2/Q3; *«ninguém entrega raio por agente numa malha só»* (doc 29 §4) |
| S3 | **a folga exacta de um disco** (recuo redondo) | o recuo em esquadria do Godot, medido nas quinas | §2.3, a sonda |
| S4 | **caminho no tique 1** | o Godot devolve vazio até ao quadro 4–5, medido | §2.4, a sonda |
| S5 | **o desvio não sai da área andável** | Godot #60354, aberto desde 2022 | §2.7 |
| S6 | **estados com voz** (*sem região · alvo fora · inalcançável · preso*) | nenhum motor expõe «preso»; caminhos vazios calados | doc 29 §2.4 Q12, §6.6 do dossiê |
| S7 | **o *scrub* devolve a perseguição exacta** | nenhum motor da pesquisa volta atrás no tempo | §2.2, o anel |
| S8 | **a mesma corrida nos três sistemas** | — | §2.6, o `physics_ecs_c9` |
| S9 | **a patrulha desenhada com a caneta** (W6) | os outros: pontos de passagem à mão | o `PathFollow`/`VecPath` desta casa |

---

## §6 — ⛔ Recusas antes de começar (com o motivo)

- **A grelha** (o modo do Construct/GDevelop): é do **projecto Tilling** (ordem do dono); o A\* do
  `ph2d-grid` continua no editor. ⚠️ E o doc dele (`square.rs:16-24`) promete um *callback* de custo que **não
  existe** — corrigir a frase é dívida da W0 (uma linha, nenhum produto).
- **Um quarto mover** (o agente mover o corpo ele próprio): duplicaria o `TopDownPlayer` (§2.1).
- **Recalcular o caminho a cada tique**: anti-padrão documentado (o *«dançar»* do Godot; o aviso do
  GDevelop) — Q6.
- **`vleue_navigator`** (arrasta `bevy` inteiro), **`oxidized_navigation`** (descontinuado), **`rerecast`**
  (voxel 3D para um problema 2D de geometria exacta), **FFI ao Recast/Detour em C++**.
- **`landmass` e `polyanya` como DEPENDÊNCIAS:** os dois são bons e MIT/Apache, mas trazem um segundo modelo do
  mundo (o *Archipelago* do `landmass` ao lado do ECS — duas fontes de verdade) e pacotes novos (`geo`, `rstar`,
  um `glam` a mais). ⇒ a lei é **nossa**, a partir dos artigos, sobre a geometria que **já** está no
  repositório; ⏳ os dois ficam como **oráculos adicionais** se a W2/W5 precisarem de um terceiro lado.
- **Plataformas com saltos** (grafo de plataformas, saltos simulados — Pignole, Surfacer): outro problema, **plano
  próprio** quando o dono o pedir.
- **Campos de fluxo** (hordas a partilhar o destino): **só** se a tabela de custo da W5 mostrar que N consultas
  não cabem no quadro.
- **Esculpir a malha com obstáculos MÓVEIS** (o *carve* do Unity): a regra da própria Unity é *«em movimento →
  desvio; parado → esculpir»*, e o nosso «parado» é a geometria estática, que já reconstrói (W6).
- **Ler o fonte do Godot, do RVO2 ou do Detour**, mesmo sendo legal: o oráculo corre-se (§0.9); a lei sai dos
  artigos.

---

## §7 — Os contadores (DELTA estimado — ⛔ conte no código no dia)

| contador | delta | quando |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (W3: `NavRegion`+`NavAgent`) · **+1** (W7: `NavCostArea`+`NavLink`) | o postcard é posicional |
| registo da **física** (`ph2d-physics-ecs`) | **+2** (W3) · **+2** (W7) | ⛔ os espelhos `ph2d-render`/`ph2d-script` **não** mexem (contam `ecs`) |
| `LIVE_SECTIONS` | **+2** (W4) · **+2** (W7) | |
| `SignalOrigin` | **+1** (`Navigation`, append-only) | W3 |
| `SignalVerb::ALL` | **+2** (`Start Navigation`/`Stop Navigation`, apendados — a posição **é** a tag) | W6 |
| `ComponentEdit` | **+2** (W4) · **+2** (W7) | |
| crates novas | **3** folhas (`ph2d-navmesh`, `ph2d-nav`, `ph2d-orca`) | W1/W2/W5 |
| pacotes externos novos | **0** (o `clipper2-rust` e o `spade` já estão resolvidos) | ⏳ a confirmar pelo `cargo deny`/`machete` na W1 |

---

## §8 — W0 em detalhe (o oráculo e as medições — nenhuma linha de produto)

### §8.1 — A sonda §5.0: a composição de hoje NÃO contorna

Um herói atrás de uma parede em `U` e um perseguidor feito com o que existe (o `ProjectileMotion` com
*homing*, como os morcegos da arena): medir a distância ao herói ao fim de `10 s`. ⏳ Previsão (a confirmar):
ela **não converge** (o perseguidor ricocheteia na parede). Esta sonda vira o **CONTROLO** da cena `=1`.

### §8.2 — O corpus do Godot, com cabeçalho

Em `crates/ph2d-navmesh/tests/fixtures/godot/` e `crates/ph2d-nav/tests/fixtures/godot/`, gerados por
`ferramentas/godot_nav_oraculo/*.gd` (a sonda de triagem passa a arnês), com
`region_set_use_async_iterations(reg, false)` antes do polígono (a armadilha do doc 29 §6.1):

| família | cenas | o que se grava |
|---|---|---|
| **F1 construção** | rectângulo+quadrado · dois quadrados que se tocam · L · obstáculo na borda · obstáculos sobrepostos · **passagem mais estreita que `2r`** · círculo e cápsula (como o Godot os converte em polígono) · obstáculo rodado | vértices, polígonos, área, furos, ilhas — a raios `0`, `5`, `10`, `25` |
| **F2 caminhos** | 8–12 pares início/fim por cena de F1, incluindo empates simétricos (o desempate do Godot é da decomposição — medido) | os cantos e o comprimento |
| **F3 fora da malha** | início e/ou alvo dentro de um obstáculo · fora da região · numa ilha separada | a projecção (o ponto mais perto) e o caminho parcial |
| **F4 desvio** (para a W5) | frente a frente (2) · cruzamento (4 cantos) · troca em círculo (8) · corredor · porta | posição e `velocity_computed` por quadro, a `--fixed-fps 60` |

⚠️ As tolerâncias saem do **ruído medido do oráculo** (a sonda leu `49.999996185` por `50`), nunca de um número
escolhido; e cada fixtura tem o gate *«o cabeçalho concorda com a tabela do teste»* (o molde da câmera).

### §8.3 — O oráculo EXACTO

Num módulo `test-support` da `ph2d-nav`: grafo de visibilidade sobre os vértices dos obstáculos já recuados +
Dijkstra (`BTreeMap`). Controlo positivo dele próprio: num rectângulo sem obstáculo o caminho é o segmento; com
um quadrado no meio, as duas voltas têm o mesmo comprimento.

### §8.4 — As medições de custo e de determinismo

| medição | régua | decide |
|---|---|---|
| construção a 10/100/1000 obstáculos | `--release`, mínimo de 5, `loadavg` ao lado (⛔ nada vale acima de `load ~5`) | síncrona no tique (§2.4) ou por mosaicos |
| a assinatura dos estáticos por tique | idem | assinatura ou «sujo» marcado pela ponte |
| `clipper2-rust` + `spade` bit-idênticos entre **dois processos** | `sha256` dos vértices e da topologia, 3 corridas | quem classifica as faces e quem triangula (§2.6) |

---

## §9 — W3 em detalhe (a primeira wave com produto)

### §9.1 — Os campos (valores de fábrica a confirmar pelo oráculo onde ele tem um)

**`NavRegion`:** `half_extents` (a caixa à volta do `Transform`) · `obstacle_layers` (máscara; de fábrica todas) ·
`show_navigation` (vista — ⚠️ se for só vista, vive no `WidgetStore` e **não** no componente; decidir na W4 pela
lei do *«interruptor de vista não é documento»* que o Sprite já pagou).

**`NavAgent`:** `target` = `Named(stable_name_id)` · `NearestTagged(tag)` · `Point(x, y)` · `None` (W3 entrega os
dois primeiros e o ponto; a patrulha é da W6) · `radius: Option<f32>` (`None` = o do corpo) ·
`arrive_distance` (Godot `target_desired_distance = 10` px ⇒ convertido a metros pelo
`pixels_per_meter`) · `repath_distance` (Godot `path_max_distance = 100` px, idem) · `stuck_after_s` ·
três nomes de sinal (`on_arrived`, `on_no_path`, `on_stuck`; vazio = calado) · `active: bool`.

### §9.2 — Os gates escritos ANTES da ponte (red-first)

| gate | queixa | régua |
|---|---|---|
| o caminho existe **no tique 1** | Q4 | o agente anda no 1.º tique depois de nascer (com a construção síncrona medida) |
| o raio sai do corpo · o grande não passa onde o pequeno passa | Q2/Q3 | dois agentes na mesma cena, a passagem `2r_pequeno < largura < 2r_grande` |
| nenhum canto é ultrapassado | Q5 | a distância ao corredor nunca passa de `velocidade × dt` + a folga |
| **não recalcula por tique** | Q6 | um contador `#[cfg(test)]` **por thread** (a lição do colisor do Motion) — com o alvo parado, `0` recálculos em `600` tiques |
| o agente não é obstáculo de si mesmo | Q8 | o colisor dele fora da construção da malha do raio dele, por construção |
| os quatro estados falam | Q12 | uma fixtura por estado, com o sinal **e** a linha do painel |
| o *scrub* devolve a perseguição exacta | S7 | gravar 120 tiques, rebobinar a 60, refazer: posições **ao bit** |
| a intenção nunca vem da fita | §2.2 | o censo da porta única `drive_controllers` cobre `drive_nav_agents` nos dois laços |
| um agente nascido de uma `Factory` persegue | — | a cópia transitória com `NavAgent` anda no tique seguinte ao nascimento |
| **o perseguidor contorna onde o *homing* bate** | §8.1 | a sonda da W0 com o `NavAgent`: a distância **converge** |

### §9.3 — A cena `PH2D_NAV_SMOKE=1` (o labirinto)

⭐ **O fenómeno com o uso real e o CONTROLO ao lado** (a lição de todas as cenas desta linha): um labirinto de
paredes sólidas, o herói ao teclado (as setas — ⛔ o `W` do WASD abre o painel de mundo e o espaço é o *play* da timeline, os dois medidos), **três** inimigos com o
mesmo `TopDownPlayer` e uma diferença cada — **(a)** o perseguidor com `NavAgent`, **(b)** o CONTROLO de *homing*
que bate na parede, **(c)** o perseguidor **grande** que não cabe na passagem estreita e dá a volta longa —, o
traço do caminho de cada agente desenhado, e o roteiro em passos numerados para o dono. ⚠️ A banda que sobra
com a timeline aberta **não é centrada na origem** (medido no FIM DE JOGO e na ARMA): o labirinto mede-se pela
**caixa**, com a foto a confirmar antes de mandar.

---

## §10 — Perigos conhecidos (as armadilhas que esta família já pagou)

1. **O laço que esquece o mover novo** (três vezes): `drive_nav_agents` entra **na porta única** e o censo tem de
   reprovar se só um laço o chamar.
2. **O anel**: o par `record`/`seed` **e** a limpeza em `rebuild_from_rest` — o tipo `PlayerStates` faz esquecer
   um deles não compilar; o `NavAgentRuntime` entra pelo mesmo tipo.
3. **O BVH vê o mundo do ÚLTIMO passo** (`cast.rs:10-24`): a construção e as consultas de linha de vista do tique
   veem a geometria do passo anterior — declarado e gateado.
4. **O corpo nascido `Dynamic` cai** (`y = −492 m` em dez segundos, medido no #25): a semente do `NavAgent` passa
   pela do controlador cinemático.
5. **Um evento lido como estado**: o painel lê o **estado** do agente (`a perseguir`, `preso`), nunca o sinal do
   tique.
6. **O custo**: 1 000 movers de vista de cima custam `263 %` de um quadro (medido no #13) — a navegação soma-se a
   isso; o tecto de agentes por cena é **medido** na W2/W5, nunca escolhido.
7. **Um `Hide` vira documento ao fim de dois quadros** (a lei do `settle`, medida no FIM DE JOGO): uma porta que
   abre e fecha na corrida é **condução**, e o que a corrida escreve a corrida desfaz (W6).
8. **A catraca da shell** (`the_shell_only_shrinks`): a fiação da cena nova vai para a crate da família; a shell
   leva só a linha do roteador.
9. **Um `#[cfg(target_os)]` muda no Linux** (a 3.ª espécie do §5.0): nenhum previsto; se aparecer, cruza-se para
   `aarch64-apple-darwin` antes de fechar.

---

## §11 — Decisões do DONO (produto), com a recomendação

1. ✅ **DECIDIDO pelo dono (02/10): SIM.** **O inimigo evita sozinho as zonas que ferem** (a lava da Vida e
   Dano a virar custo automático na W7), com uma caixa no `NavAgent` para o desligar (um inimigo imune ao
   fogo deve poder atravessá-la; a resistência do `Health` já sabe dizê-lo).
2. ✅ **DECIDIDO pelo dono (02/10): LIGADO.** **O desvio entre agentes nasce ligado** para quem tem
   `NavAgent` (o Godot nasce desligado), com o custo medido na W5 (§13.2) e a caixa *Avoid Others* para o
   desligar.
3. **Navegação em PLATAFORMAS (saltos)** — plano próprio, quando pedir.

---

## §12 — W4 FEITA (2026-10-01): o artista monta um perseguidor sem tabela nenhuma

**O que se consegue fazer agora:** *Add Component → Nav Agent* num objecto entrega um perseguidor
que funciona — a cascata do catálogo acrescenta o `TopDownPlayer` (e o corpo cinemático, pela semente
que o #25 pagou), e a semente do `NavAgent` desliga as setas do mover e põe-no em `Free`. O Inspector
ganha **duas secções** (`Nav Region` · `Nav Agent`) com todos os campos dos dois componentes, as
queixas da ponte ditas em voz alta, e a **leitura viva** do agente (*Moving · 3,25 m to go*,
*Can't reach it · going to the nearest point*, *Arrived.*). O canvas desenha a **área andável**: um
contorno por raio, e o do agente grande fecha a porta por onde o pequeno passa.

### §12.1 — As decisões, cada uma com a medição

| decisão | porquê (medido) |
|---|---|
| a leitura viva vem de um componente DERIVADO (`NavNow`, não registado) que a ponte publica no fim de todo `dispatch` | o precedente do `HealthNow`: zero canal novo na shell, e só escreve quando muda (sem passo de undo por quadro) |
| as queixas do painel são o ESPELHO das condições em que a ponte SALTA o agente (sem corpo · sem mover · mover a ler o teclado · `PlatformPlayer` · desligado · sem alvo / alvo perdido · fora de região) | um agente que a ponte salta e o painel não acusa é o *«parado sem razão»* (Q12 da pesquisa); a ordem é da mais específica para a mais geral, e o gate liga TODAS as gerais ao mesmo tempo |
| o modo `Objecto` tem DUAS faltas (`SemAlvo` vazio · `AlvoPerdido`) e o perdido vem primeiro | um nome que ninguém tem lê-se com o nome VAZIO (o painel não pode mostrar o texto de um hash); as curas são diferentes |
| o alvo grava `Named(0)` para um nome vazio, nunca o hash de `""` | a convenção de «ninguém» desta casa |
| a semente troca `EightWay` por `Free` **só** quando o modo é o de fábrica | o preço do 8-direcções, **medido** (`o_preco_do_oito_direccoes`): chega sempre, e paga `+3,3 %`–`+5,6 %` de caminho e `6`–`10` tiques; um `FourWay` escolhido pelo artista é respeitado |
| o contorno da área andável é desenhado pelo MESMO pintor dos sensores (`ProbeKind::NavEdge`, apendado), sem tiques e mais claro que o caminho | um segundo pintor seria a segunda resposta a *«como se desenha uma linha de física»*; com os tiques do raio cada vértice picotava o mapa |
| o contorno vive no overlay de física (`show_colliders`), **sem** interruptor próprio | ⚠️ **DIVERGÊNCIA do plano** (§4 dizia *Show Navigation*): o overlay de física já é a vista *«do que a física vê»*, os caminhos da W3 já moram lá, e um segundo interruptor seria a segunda resposta à mesma pergunta. A malha só existe para os raios que um agente pede — *a área andável é DE QUEM anda* |
| `PROJECT_SCHEMA` `177 → 178`; registo da física `40 → 42`; `LIVE_SECTIONS` `43 → 45`; `ComponentEdit` `+1` (`Nav`) | a registação entra no MESMO commit que as secções (a lei `every_registered_physics_component_has_a_ui_writer`) |

### §12.2 — O que a medição derrubou

- **A cadeia da SHELL não tinha régua**: apagar `nav_inspector::apply_all(sim, nav)` deixava a suíte
  inteira verde (os gates da ponte entram pela porta da família, ABAIXO da costura). ⇒ censo DERIVADO
  `toda_fila_do_inspector_chega_ao_seu_apply` (shell): as **40** filas do `DrainOut` são todas tiradas
  por um `take`, e os **19** parâmetros da `aplicar` do TOP-20 chegam todos a uma porta. Vale para
  todas as secções, não só a nova.
- **O raio AUTORADO não tinha régua** na ponte (a fixtura tinha `0`, o derivado): a metade foi escrita
  antes da prova de mutação, que a teria acusado.
- **A minha 1.ª régua do contorno tratava o recuo como cantos vivos** e acusou um ponto na quina
  arredondada (a `0,302` da quina real): a régua é a DISTÂNCIA ao rectângulo, com a folga da corda.
- **A catraca da shell** (`the_shell_only_shrinks`) cobrou `78` linhas: a cura foi **mudar a máquina de
  estados da shell para a `ph2d-app-components`** (`state_machine_tick` + `statemachine_inspector`,
  `556` linhas, os `11` testes contados antes e depois), com a isenção `State {n}` do HR-15 a viajar
  com o ficheiro e o arnês de mutação de 15/09 reapontado (as âncoras dele casavam ZERO nos caminhos
  velhos — a espécie MUDA).
- **A 1.ª foto mostrou a secção `Nav Agent` dobrada** (a política do Inspector fecha toda secção viva
  menos o Transform): a cena abre-a.

### §12.3 — A prova

Mutação **27 de 27** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w4_2026-10-01.py`](ferramentas/mutacao_navegacao_w4_2026-10-01.py), por grupos
`MUTA_G` porque a fatia tem prazo e a shell recompila). Gates novos: `a_ponte_publica_o_agente_agora` ·
`o_preco_do_oito_direccoes` · `a_area_andavel_desenha_um_contorno_por_raio` (com o CONTROLO de dois
raios) · `uma_parede_da_area_andavel_nao_leva_tiques` (com o CONTROLO do caminho) · `nav_edits_tests`
(4) · `nav_inspector_tests` (6) · `a_seccao_nav_esta_viva` (7, com clique real) ·
`escolher_um_agente_na_paleta_entrega_um_mover_que_o_ouve` · `toda_fila_do_inspector_chega_ao_seu_apply`
(2). As réguas de elisão e de altura do Inspector medem as duas secções ARMADAS no estado que pinta mais
linhas (`o_inspector_armado`).

### §12.4 — ⏳ O que fica para as waves seguintes

O desvio entre agentes (W5) · o mundo que muda, as portas, os verbos `Start/Stop Navigation`, o alvo
*a tag mais perto* e a patrulha (W6) · custo por área e atalhos (W7) · a arena e o tutorial (W8).

---

## §13 — W5 FEITA (2026-10-02): oito passam uma porta de dois sentidos sem se entalarem

**O que se consegue fazer agora:** todo `NavAgent` desvia dos outros corpos que andam — os outros
agentes, o herói, os corpos dinâmicos — sem sair da área andável, e a caixa **Avoid Others** do
Inspector desliga-o (ligada de fábrica: §11.2, ✅ decidido pelo dono em 02/10). A cena `PH2D_NAV_SMOKE=2` põe
oito vermelhos a cruzar UMA porta nos dois sentidos e, por baixo, os mesmos oito sem o desvio (o
CONTROLO): os vermelhos chegam todos (tique `500`), os cinzentos entalam-se e nenhum chega.

Commits: `d97a1fd99` (a lei + o oráculo) · `bc80e3e17` (a ponte, o componente, o Inspector) · o da cena.

### §13.1 — O oráculo: o Godot corrido em MALHA FECHADA, passo a passo

[`ferramentas/godot_nav_oraculo/desvio.gd`](ferramentas/godot_nav_oraculo/desvio.gd) — seis cenas
(frente a frente exacto e desviado · cruzamento de 4 · círculo de 8 · corredor · porta), 240 quadros. O
script é o dono do estado: a cada quadro entrega posição, velocidade de agora e velocidade preferida
(`IN`) e grava a segura que o servidor devolve (`OUT`) ⇒ cada passo re-resolve-se ISOLADO.

| medido | consequência |
|---|---|
| ⛔ com o desvio em várias linhas de execução (o de fábrica) o agente `1` lê o `0` ora antes, ora depois, ora a MEIO da actualização dele (quadros 52, 85, 169 da `frente`): **uma corrida de dados**, e duas corridas diferem | o projecto [`desvio_projeto/project.godot`](ferramentas/godot_nav_oraculo/desvio_projeto/project.godot) liga-o numa linha só ⇒ duas corridas iguais **byte a byte** |
| o Godot resolve **EM SEQUÊNCIA** pela ordem de criação (cada agente vê os anteriores já andados) | a paridade reproduz a ordem dele; com a fotografia comum o pior passo errava `0,74 px/s` no quadro 25 |
| paridade sobre `5 670` passos: **`0,00068 px/s`** longe do toque · `0,046 px/s` na faixa de toque (`0,5 %` de `R`, onde `√(d² − R²)` amplifica o `f32` dele) · os passos APERTADOS (o 3D) a `0,00012` | a lei (semi-planos + os três programas lineares) é a dele |
| ⛔ dois frente a frente no MESMO eixo **param a 25 px** e ficam (o empate Q7) | ver §13.2 |
| a ordem dos vértices de um obstáculo decide de que lado ele empurra (a 1.ª corrida pôs os quatro da `porta` DENTRO das paredes) | `Walls::from_polygons` exige área positiva, dito no doc |

### §13.2 — As decisões, cada uma com a medição

| decisão | porquê (medido) |
|---|---|
| a parede do desvio é a **da malha do raio do agente**, contra a qual ele é um PONTO (raio `0`) | a malha já está recuada pelo raio do corpo: a mesma folga não se conta duas vezes; é a cura da Q1 (o Godot desvia ignorando a malha — #60354) |
| **em sequência pela ordem das ENTIDADES**, não a fotografia comum do artigo | ⛔ a fotografia comum prende o círculo de 8 num anel à volta do centro com QUALQUER peso de lado (cada um encostado aos dois vizinhos, os empurrões anulam-se); em sequência as seis cenas do banco passam. A ordem das entidades é a mesma nos três sistemas e num replay |
| **preferência de lado** (`SIDE_BIAS = 0,25`): o pedido ganha uma componente à DIREITA do tamanho do que o aperto tirou | o empate só PRENDE no regime do CORTE do cone (longe: o semi-plano é perpendicular ao caminho e a resposta é travar); qualquer peso o desfaz; de `0,05` a `0,5` o desfecho do banco é plano (soma `1 149`–`1 158` quadros) e a `1` a porta paga `+70 %`. Precedente: o `weightSide` do DetourCrowd |
| **10 vizinhos** (`MAX_NEIGHBORS`, o do Godot) dentro do alcance SEM PERDA | ⚠️ é de TEMPO DE QUADRO: 1 000 agentes densos `9,5 ms` sem tecto (400 vizinhos cada) → `2,44 ms`; 100 agentes `0,067 ms`. O banco dá o mesmo desfecho a 6/10/16. O custo era a PROCURA de vizinhos (`9,1` de `12,3 ms`), não o programa linear |
| ninguém se desvia do PRÓPRIO alvo | desviar dele seria nunca lhe tocar: medido com a mutação, o perseguidor acaba a `2r + 0,006` contra `2r + 0,014` |
| um corpo SÓLIDO que anda e não é agente entra como obstáculo que não desvia (o agente faz o desvio inteiro) | parado, o mover já desliza à volta dele; com o desvio ele contorna ANTES de tocar (`0,39 m` fora do eixo a `1,5 m` do herói, contra `0`) |
| a intenção é a velocidade segura em FRACÇÃO da máxima | o `TopDownPlayer` em modo livre passa-a intacta (o comprimento incluído) ⇒ quem trava para dar passagem anda mesmo mais devagar |
| `NavAgent::avoidance` é o ÚLTIMO campo; `PROJECT_SCHEMA` `178 → 179`, sem migração | o postcard é posicional; zero componentes registados novos |
| sem estado novo no anel | o ORCA não tem memória: lê a velocidade do mover, que já vai no anel (gate `um_scrub_devolve_a_mesma_multidao`) |

### §13.3 — O que a medição derrubou

- **A cena de UM sentido não ensinava nada**: sem o desvio, oito corpos também passam uma porta no mesmo
  sentido (`349` tiques contra `343`, só mais roçados). É o FRENTE-A-FRENTE que entala ⇒ a cena `=2` é de
  dois sentidos.
- **A régua da parede tratou a quina como canto vivo** (a lição da W4 a repetir-se): a área recuada tem a
  quina REDONDA, e um centro a `0,311 m` da quina foi acusado. A régua é a distância ao rectângulo.
- **A 1.ª régua do perseguidor não distinguia nada** (chega no tique `78` contra `84` sem o «ignora o
  alvo»): a régua é o ENCOSTO.
- **A régua do lado direito**: a `2 m` o próprio ORCA escolhe um lado (regime das pernas) e o controlo
  saía do eixo sem peso; a `6 m` ninguém está em rota de colisão. A `4,5 m` (regime do corte) mede.
- **Contra um corpo PARADO o controlo não invade** (o mover desliza): a régua é contornar ANTES de tocar.
- **A prova de mutação achou DUAS leis sem régua** (e um mutante equivalente): as paredes da malha fora
  do desvio (M22) e um agente sem desvio contado como se fizesse metade (M24) passavam a suíte inteira.
  As réguas novas foram MEDIDAS com a mutação ao lado: `A` rente ao chão desce a `0,0060 m` sem as
  paredes contra `0,0275` com elas; no encontro misto os dois ENCOSTAM-SE (`2r + 0,0040`, o offset de
  fábrica do controlador) contra `2r + 0,0096` — o quanto `A` sai do eixo não distingue (`0,549`/`0,566`).

### §13.4-bis — A prova

Mutação **30 de 30** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w5_2026-10-02.py`](ferramentas/mutacao_navegacao_w5_2026-10-02.py), os quatro
controlos): 17 na lei, 7 na ponte, 3 na família (a aplicação, o retrato, o controlo da cena) e 3 no
painel. ⚠️ A 1.ª M10 (`verts[para]` → `verts[de]` em `from_walkable_walls`) é EQUIVALENTE — com o
`next`/`prev` a acompanhar dá as mesmas arestas invertidas — e foi trocada pela que vira a malha do avesso
na ponte. Gates novos: o oráculo do Godot passo a passo · o banco de cenários (+ o CONTROLO sem peso) ·
11 leis da folha · 8 de costura (`nav_desvio`) · `o_desvio_vai_e_volta` · `clicar_em_avoid_others_pede_o_contrario`
(clique real) · `nav_smoke_porta::a_cena_contem_o_fenomeno`. Fotos da `=2` aos `5,2 s` e aos `11,2 s`
(ecrã virtual), antes de a mandar.

### §13.4 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| a fotografia comum (Jacobi) | o círculo de 8 nunca chega, com qualquer peso de lado (0,05 a 1) |
| uma perturbação fixa por agente (o remédio dos exemplos do RVO2, sem acaso) | a `1e-6`–`1e-2` rad: o cruzamento de 4 prende-se |
| o alcance SEM PERDA sem tecto | `9,5 ms` a 1 000 agentes densos (57 % do quadro) |
| o Godot com as linhas de execução de fábrica como oráculo | uma corrida de dados: não é determinístico |

### §13.5 — ⏳ O que fica para as waves seguintes

- O custo que sobra a 1 000 agentes (`2,44 ms`) é a VARRIDA dos candidatos — a célula da grelha é o alcance
  sem perda; uma procura dos `k` mais perto por anéis de uma grelha fina tirá-lo-ia.
- A leitura viva não diz *«a dar passagem»*: um agente travado pela multidão lê-se `Moving`. Um estado com
  voz (S6) para o aperto é candidato.
- ✅ **Decisão do dono (§11.2, 02/10):** o desvio nasce LIGADO.
- O mundo que muda, as portas, `Start/Stop Navigation`, a patrulha (W6) · custo por área e atalhos (W7) ·
  a arena e o tutorial (W8).

---

## §14 — W6 FEITA (2026-10-02): o guarda patrulha, vê, persegue, e a porta fecha-se

✅ **Smoke do dono APROVADO (02/10)** — a cena `=3`, depois das curas do Rewind (§14.3).

**O que se consegue fazer agora:** uma porta que desliza e PÁRA vira parede para os agentes no tique em
que pára (e deixa de o ser quando volta a andar); os verbos **Start Navigation** (com o nome de quem
perseguir, ou vazio = o alvo autorado) e **Stop Navigation** na tabela de acções; dois alvos novos no
Inspector — **Tag** (o mais perto que pertence a uma tag, com a subárvore) e **Patrol** (os pontos de uma
forma desenhada com a caneta: fechada dá voltas, aberta vai e volta). A cena `PH2D_NAV_SMOKE=3` monta o
exemplo patrulha → persegue com a `StateMachine`, e a porta que se fecha na cara do guarda.

Commits: `47efb64d0` (mosaicos) · `b76d23c39` (a ponte e a porta) · `f35e4b440` (os verbos) · `c36582a72`
(os alvos e o Inspector) · `500e8a24d` (a cena `=3`) · `d9c99b5d4` (o custo medido na ponte) ·
`6c57e7405` (as réguas que faltavam).

### §14.1 — A medição que abriu a wave (e a que a fechou)

`examples/medir_mudanca.rs` (ph2d-navmesh) e `examples/medir_nav_tique.rs` (ph2d-physics-ecs), `--release`,
`loadavg < 5`, mínimo de 5 (ou de 10 janelas):

| construção INTEIRA, `100 × 100 m`, raio `0,4` | total | recuo | união | diferença | triangulação | fusão | `NavMesh` |
|---|---|---|---|---|---|---|---|
| 100 obstáculos | `4,2 ms` | `0,17` | `0,54` | `0,49` | `1,61` | `0,92` | `0,38` |
| 1 000 obstáculos | `61,4 ms` | `1,61` | **`32,0`** | `9,95` | `9,14` | `5,65` | `2,54` |

⇒ a construção síncrona **não cabe** no tique a 1 000 obstáculos (`61 ms` = quatro quadros), e metade é a
UNIÃO, que cresce mais depressa que a cena ⇒ **mosaicos** (§2.4).

| `TiledMesh`, 1 000 obstáculos | `T = 5` | `10` | **`15`** | `20` | `25` | `33,4` | `50` |
|---|---|---|---|---|---|---|---|
| a frio (ms) | `32,7` | `29,6` | `30,4` | `31,7` | `33,4` | `36,1` | `42,1` |
| uma porta (ms) | `4,1` | `4,2` | **`3,5`** | `4,0` | `10,7` | `6,7` | `42,5` |
| procura (µs; inteira `~636`), posição A | `1 123` | `1 050` | `665` | `597` | `1 189` | `764` | `819` |
| procura, posição B (a cena `+3,7 m`) | `680` | `774` | `584` | `624` | `598` | `910` | `657` |

⚠️ A procura paga as costuras conforme ONDE elas cortam a geometria (até `1,8×`, e o mesmo `T` muda de
posição para posição) — `15` e `20 m` ficaram `≤ 1,07×` nas duas; `15` paga menos por porta ⇒ `TILE_M = 15`.

| a PONTE, uma porta a mudar (o pior tique da janela) | 100 obst. | 1 000 obst. |
|---|---|---|
| a malha inteira (`TILE_M = 1 000`) | `4,0 ms` | `44,1 ms` |
| **mosaicos de 15 m** | **`0,81 ms`** | **`7,3 ms`** |
| o CONTROLO (a mesma janela, sem mexer) | `0,03 ms` | `0,50 ms` |
| o tique sem mudança (olhar e ver que está em dia) | `+12 µs` | `+0,2 ms` |

A 1 000 obstáculos, o que sobra da porta: **montar** `5,2 ms` (a montagem é O(malha): `~2,6` são a `NavMesh`)
+ **recalcular** o agente `2,1` (um caminho de `100 m`) — as paredes do desvio eram `3,0 ms` e passaram a
`0,12` (dois `BTreeMap` → vectores indexados pelo vértice). ⚠️ Uma porta muda a malha DUAS vezes: quando
começa a andar (deixa de recortar) e quando pára.

### §14.2 — As decisões, cada uma com a medição

| decisão | porquê (medido) |
|---|---|
| a malha de cada `(região, raio)` é uma grelha FIXA de mosaicos (ancorada na grelha inteira) com a assinatura de cada um; uma actualização refaz só os mosaicos cuja assinatura mudou | §14.1; incremental = a frio AO BIT (gate) |
| o corte é CANÓNICO: o ponto onde uma aresta cruza a costura sai da aresta ORIGINAL, com os extremos por ordem fixa e o arredondamento inteiro | os dois lados escrevem o mesmo ponto (gate de unidade); com o corte de Sutherland–Hodgman cru, o pedaço já cortado por outra linha arredondava outro ponto |
| a montagem REPARA as junções em T das costuras (todo vértice numa linha de costura entra nas arestas de costura que o atravessam) e só esses vértices passam pelo índice | a área por mosaicos fica numa faixa de UMA unidade da grelha nas costuras (medido `0,016` da faixa); o caminho muda `2e-10 m`; a procura sobre a malha montada = o EXACTO |
| o obstáculo é o colisor de um corpo que NÃO anda: estático (pose autorada) e **cinemático PARADO** (velocidade linear E angular do solver a zero, pose de agora) | é o *«carve only stationary»* do `NavMeshObstacle` do Unity, sem o relógio de espera dele (um número inventado); a velocidade vai no anel ⇒ a mesma num replay |
| ⛔ um cinemático com MOVER (`TopDownPlayer`/`PlatformPlayer`/`ProjectileMotion`) nunca é obstáculo | um herói parado punha o alvo de todos dentro de um furo, e um agente parado seria obstáculo de si mesmo (Q8) — gate com o CONTROLO da caixa sem mover |
| só os agentes da malha que MUDOU esquecem o caminho; uma malha que ninguém pede é esquecida | gate com duas regiões; o recálculo é o terço do custo de uma porta a 1 000 |
| os verbos ANUNCIAM e a ponte grava a ordem por tique (o idioma da vida); a ordem vive no anel e nunca toca no `NavAgent` | o que a corrida escreve a corrida desfaz (§10.7); o scrub refá-la ao bit (gate); tocar por cima depois de um scrub apaga a fita futura, como a da vida |
| `Start Navigation` lê o NOME de quem perseguir (vazio = o alvo autorado) e liga um agente autorado desligado | é o que torna o exemplo patrulha → persegue autorável SEM um verbo por alvo: perseguir é `Start «Hero»`, voltar à ronda é `Start «»` |
| a patrulha visita os pontos da forma COZIDA, em mundo; a curva parte-se ao meio até a flecha caber na `arrive_distance` do agente; o ponto avança a essa mesma distância | a régua é a do executor (detalhe mais fino ele não distingue) e o `On Arrived` não fala durante a ronda (gate) |
| *a tag mais perto* = em linha recta, com a subárvore, o próprio agente fora, empate pela identidade; a árvore chega por `set_tag_tree` na fase do passo, ANTES do tique | sem a árvore ninguém (falha fechado, gate); ⛔ a 1.ª redacção entregava-a na fase dos sinais (DEPOIS do tique): o 1.º tique depois de editar as tags e o replay dele viam árvores diferentes — apanhado na auditoria do fecho, gate de texto `a_arvore_das_tags_vai_a_ponte_antes_do_tique` |
| o `NavRoute` (os pontos da ronda) é DERIVADO e não registado, escrito pela família na passagem do seguidor de caminho | a geometria não entra no ECS (a doutrina do `VecPathRef`); a lei do `NavNow` |

### §14.3 — O que a medição derrubou

- **A folga do balde era o raio** (`1,01·r`): o canto em esquadria recua até `2r` e o polígono do disco
  circunscreve-o — um obstáculo ficava fora de um mosaico que tapava (`6e-4 m²` de área a mais). ⇒ `2r`.
- **A minha régua da área** (`1e-6 m²`) era um palpite: a costura move-se no máximo uma unidade da grelha,
  logo a régua é a faixa dessa largura ao longo de todas as costuras.
- **O mapa dizia que mudou sem mudar** (um obstáculo fora da região mudava a assinatura global): ⇒ só
  muda quando um mosaico se refez ou saiu.
- **A sonda da porta lia `0,43 ms` por `11 ms`**: o 1.º movimento punha a porta onde ela já estava e o
  mínimo escolhia esse caso. ⇒ o 1.º movimento é real e há o CONTROLO sem movimento.
- **A cena `=3`**: um ALARME de tempo fixo (`2,5 s`) deixava o guarda passar a porta; a ronda a `2,2 m`
  do vão também; a zona verde junto à porta disparava com o herói dentro do vão. As rondas nasceram
  POR BAIXO do chão (a `sync` dá à forma o 1.º lugar livre da pilha — apanhado na 1.ª foto). O relógio
  de fábrica ARRANCA sozinho e a porta abria a cena fechada.
- ⛔⛔ **O REWIND, report do dono (02/10): «milhões de mensagens e a porta não volta aberta».** (1) A
  porta do rebobinar corre a CADA quadro parado no início e renascia cada cérebro com `started =
  false` — o guarda anunciava «patrulhar» a cada quadro (29 em 30). ⇒ no rebobinar ela é idempotente
  (quem já está no inicial e já o anunciou fica). (2) A porta da cena era um cinemático que um TWEEN
  descia: o Rewind renasce o relógio, o tween larga e a pose fechada vira DOCUMENTO (a lei do
  `settle`); um tween de volta não corre parado no início. ⇒ a porta é uma FÁBRICA no vão (nasce uma
  parede estática), e o Rewind VARRE o que nasceu. O gate da cena joga agora 30 quadros parados.
- **O meu arnês da cena** deitava fora o que um cérebro emite: na shell, a emissão chega aos OUTROS
  cérebros no quadro seguinte (o barramento) e à tabela no mesmo.
- **Duas leis sem régua** antes da prova de mutação: a porta a RODAR no sítio (a angular conta para
  «parado») e o próprio guarda como *a tag mais perto*.

### §14.4 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| a construção síncrona inteira por mudança | `61 ms` a 1 000 obstáculos (`44 ms` na ponte, com as paredes rápidas) |
| o mosaico de `5`/`10`/`25 m` | a procura paga até `1,8×` conforme onde a costura corta; `25 m` paga `10,7 ms` por porta |
| um relógio de espera para «parado» (o `carve` do Unity) | um número inventado; a velocidade do solver diz-o ao tique e vai no anel |
| o alarme de tempo fixo na cena | o guarda (`2,2 m/s`) chega à porta em `~1,1 s` |
| a porta da cena como cinemático conduzido por um tween | o Rewind não a abria (a pose vira documento quando o tween larga) — o report do dono de 02/10 |
| o oráculo do Godot para a reconstrução | a lei que importa é *por mosaicos = inteira = o EXACTO* e *incremental = a frio ao bit*: um oráculo mais forte que o do Godot (que nem constrói por mosaicos na 2D), e gateado |

### §14.5 — A prova

Gates novos: `mosaicos` (3) + 4 de unidade do corte · `nav_mundo` (5: a porta fecha e abre ao bit, a
porta a andar e a RODAR não recortam, o personagem parado não é parede, só a malha que mudou) ·
`nav_ordens` (4) · `nav_alvos` (3) + 4 leis da ronda · a rota (3) · os verbos (o anúncio, a entrega, os
controlos à mão do `arg_kind`/tags) · o painel (cada modo a sua linha; a tag escolhida com o ponteiro
REAL) · a família (os modos vão e voltam; a forma perdida) · a queixa · a cena `=3` jogada inteira sem
ecrã. Fotos da `=3` aos `6 s` e `9 s` (ecrã virtual).

Mutação **45 de 45** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w6_2026-10-02.py`](ferramentas/mutacao_navegacao_w6_2026-10-02.py), os quatro
controlos, 8 grupos). A 1.ª corrida deu **37 de 45** e achou SETE leis sem régua, cada uma curada com a
régua MEDIDA ao lado da mutação: a ordem fixa dos extremos no corte era **equivalente** (os dois mosaicos
cortam o mesmo anel no mesmo sentido — saiu do código); o corte pela aresta ORIGINAL só decide quando
uma aresta atravessa um mosaico de ponta a ponta (o caso achado por busca: `x = −2` contra `−3`); a
reparação das junções em T (um losango com a ponta a meio de uma costura — a 1.ª fixtura punha-a no
canto de quatro mosaicos, onde o vizinho já tem o vértice); a recusa da malha sobreposta; a
continuação de menor índice das paredes do desvio; a FITA das ordens (os alvos do scrub saíam de
checkpoints que já tinham a ordem); e o recomeço do zero a esquecer as ordens. Uma mutação (um `if false`
num braço de `match`) não compilava e foi trocada.

### §14.6 — ⏳ O que fica

- **A montagem é O(malha)** (`~5 ms` a 1 000 obstáculos, `~2,6` são a `NavMesh`): uma `NavMesh` por
  mosaicos (a procura a atravessar mosaicos) tirá-la-ia — é a próxima alavanca do custo de uma porta.
- **Todos os agentes da malha que mudou recalculam no mesmo tique** (`~2 ms` cada a 1 000 obstáculos):
  um agente cujo corredor não toca nenhum mosaico refeito podia guardar o caminho QUANDO a porta fecha
  (abrir encurta caminhos de qualquer um).
- **Enquanto anda, uma porta é para o desvio um círculo** (o raio que a envolve) — uma porta comprida a
  deslizar desvia os agentes de longe.
- **O Inspector diz *«Switched off»*** de um agente autorado desligado que um `Start` pôs a andar (a
  queixa lê o autorado; a leitura viva diz *Moving*).
- As waves seguintes: W7 (custo por área — com a decisão do dono §11.1: o inimigo evita a lava, com caixa
  para desligar —, atalhos) · W8 (a arena, o tutorial `03_navegacao.pdf`).

---

## §15 — W7 FEITA (2026-10-03): o inimigo evita a lava sozinho, a lama custa, e o portal leva-o

✅ **Smoke do dono APROVADO (03/10)** — a cena `=4`.

**O que se consegue fazer agora:** *Add Component → Nav Cost Area* num objecto com colisor faz dele uma
**lama** (custo: o agente dá a volta se a volta custar menos) ou uma zona **proibida** (um furo para todos).
Uma **lava** da Vida e Dano (um `Damage` parado) é EVITADA sozinha por todo inimigo cuja vida a sente — a
decisão do dono (§11.1) —, e a caixa **Avoid Harm** do Nav Agent desliga-o; o imune ao fogo (a resistência
`0`) e o que não tem vida atravessam. *Nav Link* num objecto liga-o ao objecto com o nome escolhido: um
**teleporte** (o corpo salta) ou uma **porta de um sentido** (anda a direito; a porta é uma zona proibida
mais o atalho), com o sinal *atravessou* (`on_crossed`, do atalho, com o agente como o outro). A cena
`PH2D_NAV_SMOKE=4` monta o rio de lava com dois portais e o CONTROLO.

### §15.1 — A medição que abriu a wave (e a derrubou)

[`examples/medir_custo.rs`](../../crates/ph2d-navmesh/examples/medir_custo.rs) (ph2d-navmesh), `--release`,
8 cenas `30 × 20` m com 10 obstáculos e 4 lamas, 20 pares cada. A régua é o **oráculo ponderado**
(`ph2d_nav::oracle::WeightedOracle`: cantos + Steiner SÓ nas fronteiras, custo exacto por segmento), com a
convergência medida: a `0,1` m de passo erra no máximo `+0,23 %` contra `0,05` (mediana `0`).

| custo do caminho / oráculo (média · p95 · máx) | peso 1,5 | peso 4 | peso 10 |
|---|---|---|---|
| C0 — o Polyanya que ignora o custo | `1,022 · 1,147 · 1,377` | `1,197 · 2,029 · 3,282` | `1,578 · 4,091 · 7,854` |
| C1 — A\* pelo meio das arestas + funil (Detour/Godot) | `1,045 · 1,272 · 2,082` | `1,055 · 1,278 · 1,953` | `1,088 · 1,433 · 3,517` |
| **C2 — o Polyanya que refracta, grelha `0,25` m** | **`1,000 · 1,000 · 1,005`** | **`1,000 · 1,000 · 1,000`** | **`1,000 · 1,000 · 1,005`** |

| C2 por passo da grelha (peso 2 · peso 10): custo/oráculo máx · µs | `0,5` m | **`0,25` m** | `0,1` m |
|---|---|---|---|
| | `1,011 · 74` · `1,094 · 245` | **`1,011 · 100` · `1,005 · 351`** | `1,000 · 218` · `1,000 · 803` |

⇒ **`STEINER_M = 0,25`**: o mesmo caminho em média, ~1 % no pior, a 2,3× menos que `0,1` (a `0,5` a grelha já
escolhe o corredor errado). Abaixo de `1,0` é o oráculo a errar.

**A cena GRANDE** (`100 × 100` m, `1 000` obstáculos, `100` lamas a peso 4 — `~25 %` do chão; load alto,
só a ordem de grandeza): o uniforme `0,36 ms` de mediana e `5 650` nós expandidos por consulta; o
ponderado `~4,9 ms` e `64 500` (11×; eram `87 000` antes de sair o «dobrar» redundante, §15.4). Os mosaicos com as 100 lamas constroem a frio em `~71 ms` (39 sem),
e uma lama a mexer refaz em `~11 ms` (a montagem O(malha), o aberto da W6).

### §15.2 — As decisões, cada uma com a medição

| decisão | porquê (medido) |
|---|---|
| a malha guarda a ÁREA de cada polígono; o custo vai na CONSULTA (o modelo do Detour) | dois agentes com tabelas diferentes partilham a malha; mexer num custo não refaz nada |
| ⛔ **proibida = um FURO**, nunca um custo infinito | as ilhas, o ponto alcançável mais perto e as paredes do desvio leem a malha — com custo infinito todos mentiriam |
| a lava que MAGOA este agente é um furo só na malha dele: a chave da malha ganha a assinatura das zonas evitadas | `Damage::magoa` (a equipa, e o tipo que a vida dele sente) numa porta só; o imune ao fogo tem outra malha |
| a construção parte o chão em PEDAÇOS disjuntos pelo Clipper (o comum + uma peça por área, a mais cara manda), UMA triangulação, a paridade generalizada a «em que pedaço estou» | sem áreas a construção de sempre ao bit (gate); a fusão em convexos só dentro do pedaço |
| junções em T a menos de **2 unidades** da grelha entram na aresta | o Clipper arredonda o mesmo cruzamento de formas diferentes: medido um vértice a `1,016` unidades (o pior caso é `√2`) |
| onde a troca por dono é ambígua (`[0, 0, 2, 5]` — quatro pedaços num ponto), ponto-no-polígono EXACTO do baricentro | a troca por dono etiquetava um triângulo de fora |
| **o Polyanya que refracta**: dois movimentos numa aresta onde o custo muda — atravessar (raízes na grelha e nas pontas do intervalo; num vértice, o leque inteiro) e **deslizar** em cima da fronteira para o lado de TRÁS caro — e o polimento de Snell | cada movimento achado por uma cena que o oráculo resolvia e a procura não (abaixo) |
| as raízes de fronteira entram no heap como PROMESSAS (`g + w_min·|x − t|`) | 3,3× mais rápido, o mesmo custo (2 815 → 132 raízes materializadas num caso pesado) |
| **o atalho exacto**: se o caminho UNIFORME custa `w_min × comprimento`, ele É o óptimo ponderado; senão a resposta é o melhor dos dois | nenhum caminho custa menos que `w_min × comprimento` ≥ `w_min ×` o mais curto — prova de duas linhas |
| o atalho é um grafo pequeno sobre a procura: {partida, alvo, pontas}, arestas = procuras reais preguiçosas | o mais curto com atalhos é exacto (cada troço já é o óptimo da malha) |
| o teleporte põe o corpo na saída DENTRO do tique (`set_body_pose`) | o replay corre a mesma lei e salta no mesmo tique (gate do scrub depois do salto) |
| a meio de uma porta de um sentido não se replaneia | ali o agente está fora da malha (a porta é proibida); replanear levava-o para trás |

### §15.3 — O que a medição derrubou

- **A saída prevista (A\* + funil)**: o idioma da indústria erra 27–43 % no p95 (C1) e a peso 1,5 é PIOR
  que ignorar o custo.
- **O oráculo de Steiner em TODAS as arestas** convergia linearmente (`+9,8 %` a 8 pontos por aresta): a
  recta dentro de uma área uniforme virava ziguezague. ⇒ Steiner só nas fronteiras, custo exacto por
  segmento.
- **Os três movimentos que faltavam**, cada um pela sonda `DBG_CASO` (o caminho do oráculo troço a troço):
  (1) o caminho que contorna a lama dobra no vértice DELA, que não é canto de parede (a 1.ª cura —
  marcá-lo como canto — saiu depois: a mutação achou-a redundante com as raízes-vértice, ver §15.4); (2) dentro da lama o
  atalho «mesmo polígono ⇒ a direito» só é óptimo com custo uniforme (com custos é uma candidata no heap);
  (3) o óptimo SAI da lama, corre em cima da fronteira e volta a entrar — no ângulo crítico de Snell
  (`5 + 2d·√(w² − 1)`, gate analítico a `1e-9`).
- **A explosão**: com a poda estrita (`<`), duplicados de custo igual multiplicavam-se de canto em canto
  (35 milhões de nós numa cena de 200 polígonos); e num canto de custo o leque da volta dava a volta
  inteira (6,8 milhões de voltas). ⇒ igual também se poda (uma raiz de fronteira emite o polígono
  inteiro), e a fronteira de custo é a parede do leque.
- **A dominância com o menor dos dois lados** era falsa para o lado caro — exactamente o do deslize.
- **Os meus controlos dos gates** (duas vezes): «a peso 1,5 sair não compensa» e «a 1,02 também não» — as
  saídas a olho eram perpendiculares; a lei é a do ângulo crítico, e a procura acertava.

### §15.4 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| o A\* sobre polígonos + funil (Detour/Godot) | 27–43 % acima do óptimo no p95 |
| ignorar o custo (o Polyanya puro) | até 7,9× o óptimo a peso 10 |
| a proibida como custo infinito | as ilhas, o mais perto e as paredes do desvio mentiriam |
| o custo do caminho uniforme como TECTO da procura ponderada | piorava a precisão (máx `1,0189` contra `1,0000` a peso 4): o polimento traz para baixo do tecto o que a discretização punha acima |
| a grelha a `0,5` m | escolhe o corredor errado (9 % a peso 10) |
| deslizar para a FRENTE | redundante (desigualdade triangular), e não mudava o custo |
| dobrar no vértice de uma fronteira como num canto de parede | a mutação M8 sobreviveu; sem ela os mesmos custos ao dígito e menos 26 % de nós na cena grande |

### §15.6 — ⏳ O que fica

- **A procura ponderada numa cena com muita lama custa ~11× a uniforme** (`100` lamas e `1 000` obstáculos:
  `64 500` nós contra `5 650`): várias raízes na MESMA fronteira abrem frentes paralelas que só se podam
  nos cantos e nas fronteiras seguintes. A próxima alavanca: uma dominância entre as frentes. Só a paga
  quem tem lama (a lava é furo, a procura é a uniforme) e só quando o caminho uniforme a toca.
- A construção INTEIRA com áreas é lenta a escala (`1,2 s` a 100 lamas e 1 000 obstáculos); a ponte usa
  mosaicos (`71 ms` a frio). Os abertos da W6 continuam.

---

## §16 — W8 FEITA (2026-10-03): a arena navega, e o tutorial 03

**O que se consegue fazer agora:** na arena (`PH2D_VIDA_SMOKE=4`) os morcegos dão a volta a um **muro**
(só se passa por baixo dele) e, com o herói dentro da lava, **esperam na borda** — a lava queima-os. O
tutorial [`tutoriais/03_navegacao.pdf`](tutoriais/03_navegacao.pdf) leva o dono do mapa (`Nav Region`) ao
agente (`Nav Agent`) e acaba com ele a **montar uma perseguição**: *Add Component → Nav Agent* na
Salamandra, *Target → Object → Heroi* — ela dá a volta ao muro e, imune ao fogo, **entra na lava** atrás do
herói. Fonte em [`tutoriais/src/03_navegacao.html`](tutoriais/src/03_navegacao.html), gerado por
`scripts/tutorial-pdf.sh`.

### §16.1 — As decisões

| decisão | porquê |
|---|---|
| o morcego é `NavAgent` + `TopDownPlayer` (sai o `ProjectileMotion` teleguiado) | o §5.0 mediu que o teleguiado não contorna; o gate `um_morcego_da_a_volta_ao_muro` tem-no como CONTROLO (fica do lado de cá do muro, não morde em 10 s) |
| a chegada do morcego é a de fábrica (`0,1` m entre centros) | os corpos encostam a `0,525` m: ele empurra até MORDER. A chegada da cena `=1` (encostar com folga) pará-lo-ia ao lado do herói (mutação M7) |
| **a lava é EVITADA pelos morcegos, e é o que a cena ensina** | o `Damage::magoa` diz que ela os fere (sem equipa, fogo, sem resistência) — é a decisão do dono do §11.1; o tutorial mostra o outro lado com a Salamandra imune |
| o muro mora logo à direita da coluna dos avisos, do alto (`6 m`, fora da banda) até `0,5 m` | entre o ninho e o herói, sem tocar no que o roteiro manda ver; a passagem de baixo (`1,69 m` até ao `FUNDO`) deixa passar a Salamandra (meia diagonal `0,71`) — `const` asserts |
| a Salamandra é **cinemática**, não estática | a semente do mover **nunca rebaixa um `Static`** que o artista pôs (`physics_seed.rs`), logo com `Static` o gesto do tutorial pedia um terceiro passo (mudar o corpo); parada, a malha conta-a igual (o cinemático parado é obstáculo) |
| a região andável cobre o CANVAS (`REGIAO_FUNDO = −2,5`), não o `FUNDO` (`−1,19`) | a 1.ª foto da W8 mostrou chão visível abaixo da região onde nenhum morcego entrava |
| o gate do tutorial (`o_tutorial_da_navegacao_nomeia_rotulos_que_existem`) é o 3.º irmão, com as portas partilhadas do 01 | uma porta, três gates; 5 tabelas (a `inspector.rs` traz o `Add Component`) e 8 pintores (as secções da navegação, a do `Top-Down Player`, a da vida, o cabeçalho, os verbos) |

### §16.2 — O que a medição (e a foto) derrubou

- **A foto achou a região curta**: o `FUNDO` da família (medido noutra cena) não é o fundo do canvas desta
  foto (`1930×1040`, px `765` ↔ `−2,6 m`).
- **O gate `um_tiro_mata_um_morcego` punha o herói do lado de lá do muro** — a bala acertava no muro. O
  herói do caso passa para `x = 0,5` (lado do ninho).
- O 1.º texto do tutorial dizia «o muro também pára as balas» — afirmação sem régua; saiu.

### §16.3 — A prova

Gates novos ([`vida_arena_nav_tests.rs`](../../crates/ph2d-app-components/src/vida_arena_nav_tests.rs), pelo
arnês do quadro inteiro da arena): `um_morcego_da_a_volta_ao_muro` (passa por baixo, centro nunca a menos de
um raio do muro, morde; CONTROLO teleguiado) · `os_morcegos_esperam_na_borda_da_lava` (chegam a `≤ r + 0,3`
da borda e não a pisam — a régua é a DISTÂNCIA ao rectângulo; CONTROLO sem `Avoid Harm` entra e chega a
`< 1 m` do herói) · `a_salamandra_posta_a_perseguir_atravessa_a_lava` (o estado depois do gesto + o alvo
pela porta do Inspector `nav_inspector::apply_nav_edit`; entra na lava, zero dano). A cascata do gesto já
tinha gate na shell (`escolher_um_agente_na_paleta_entrega_um_mover_que_o_ouve`).
Mutação **10 / 10** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w8_2026-10-03.py`](ferramentas/mutacao_navegacao_w8_2026-10-03.py), quatro controlos,
grupos APPC · PANEL). ⛔ **A 1.ª corrida deu 9/10: a M10 (o botão `Object` renomeado) SOBREVIVEU** — o
`contains` da porta partilhada dos três gates de tutorial lia o fonte inteiro e `Object` casava dentro dos
identificadores `NavAlvoModo::Objecto` e `ArgKind::ObjectName`. Cura na PORTA (`literais_de` ·
`pintores_de`): o rótulo só vive num LITERAL de texto; os gates 01 e 02 passam por ela e continuam verdes.

### §16.4 — ⏳ O que fica

- O tutorial pede edições numa CÓPIA a correr (a Salamandra nascida da fábrica); o recomeço apaga-as — o
  tutorial di-lo. A foto não clica (o XTest é ignorado na Xwayland virtual): os passos 7–8 provam-se pelos
  gates, e o smoke do dono é a 1.ª corrida com rato.
- ✅ **Curados depois do fecho (03/10, ordem do dono):** o *«Switched off»* de quem um `Start` pôs a andar
  (o painel lê a ORDEM da ponte), o *«Giving way»* (só quando um VIZINHO cortou o pedido — só pela rapidez,
  o agente sozinho acusava `58/369` tiques nas quinas), a cena `=1` apertada (corpos e paredes escalados),
  a porta que anda (discos ao longo da forma: `1,55 m → 0,02 m` fora do caminho) e o defeito G da vida.
  Detalhe e gates: o handoff da W8 §2-bis.
- Os outros abertos da W5–W7 continuam (§15.6 e o handoff da W7 §4).

---

## §17 — W9 FEITA (2026-10-03): o custo à escala

✅ **Smoke do dono APROVADO (03/10)** — as cenas `=3`, `=4` e a arena (`PH2D_VIDA_SMOKE=4`).

**O que muda para quem usa:** nada que se veja numa cena pequena — e é esse o ponto. Numa cena grande
(`100 × 100 m`, `1 000` obstáculos) a procura com muita lama gasta menos de metade, uma porta que pára
refaz a malha num terço a menos, e uma porta que muda já não congela o jogo com todos os inimigos a
replanear no mesmo tique: eles entram numa FILA e são servidos aos poucos, os de caminho partido
primeiro. De passagem, a sonda com 200 agentes achou um **crash** do desvio (ORCA) quando um agente
fica entalado entre duas paredes — curado.

### §17.1 — As medições (`--release`, load `~2–3`; antes = o fecho da W8)

**A procura ponderada** (`medir_custo` §4, `100 × 100 m`, `1 000` obstáculos, `100` lamas a peso 4, 40
consultas; os nós não dependem da carga):

| | nós expandidos / consulta | mediana · p95 · máx | §3 a `0,25 m`, peso 4 · 10 |
|---|---|---|---|
| uniforme (a régua) | `5 650` | `0,32 · 1,7 · 4,9 ms` | — |
| ponderada, antes (o fecho da W8) | `64 508` | `4,7 · 38 · 66 ms` | `143 · 291 µs` |
| **ponderada, W9** (com a malha contígua) | **`24 661`** | **`3,5 · 21 · 30 ms`** | **`118 · 202 µs`** |
| CONTROLO: o mesmo binário, `SEM_DOMINANCIA=1` | `64 508` | `4,7 · 38 · 66 ms` | — |

Custo / oráculo (§3, `0,25 m`, os quatro pesos): **ao dígito o mesmo de antes** — `1,0000 · 1,0000 ·
1,0053` (1,5) · `1,0000 · 1,0000 · 1,0107` (2) · `0,9998 · 1,0000 · 1,0000` (4) · `0,9997 · 1,0000 ·
1,0045` (10) —, e a lista dos pares acima de `1,0001` é a MESMA com e sem a dominância
(`PIOR=1`).

**A montagem da malha** (`medir_custo` §4, o mínimo de 5, três corridas alternadas antes/depois na
mesma máquina):

| uma actualização | antes | **W9** |
|---|---|---|
| uma porta (sem lamas) | `3,96 ms` | **`2,65 ms`** |
| uma porta (com 100 lamas) | `5,66 ms` | **`3,76 ms`** |
| uma lama a mexer | `7,13 ms` | **`5,23 ms`** |
| a frio (com 100 lamas) | `58,6 ms` | `56,8 ms` |

**O replaneio em massa** (`examples/medir_replaneio.rs` da ponte, a porta a alternar; o pior tique da
janela que se segue, mediana de seis):

| orçamento de nós / tique | 10 · 50 · 200 agentes | a fila esvazia (200) | o último partido (200) |
|---|---|---|---|
| sem fila (antes) | `10,4 · 34,3 · 124,3 ms` | `1` tique | `1` |
| `40 000` | `9,3 · 18,5 · 52,6` | `30` | `5` |
| **`20 000` (o produto)** | **`6,9 · 16,5 · 51,7`** | **`58`** | **`9`** |
| `10 000` | `7,0 · 16,1 · 51,3` | `97` | `16` |

O CONTROLO (o mesmo tique sem a porta mexer) é `2,1 · 10,5 · 37,6 ms` — a 200 agentes o tique que muda
já está a `14 ms` do que não muda.

### §17.2 — As decisões

| decisão | porquê (medido) |
|---|---|
| **a dominância entre frentes** (`polyanya_dominancia.rs`): um nó numa aresta onde o custo NÃO muda é cortado nas pontas onde uma frente já EXPANDIDA na mesma aresta chega por `g + w·|ρ − y|` menor ou igual | `D(y)` tem no máximo um extremo ao longo de uma recta (a curva de nível é uma hipérbole) ⇒ o corte é exacto e só nas pontas (conservador); `64 508 → 24 661` nós |
| só na procura ponderada | a uniforme fica ao bit (gate); e ela já é a régua |
| só onde o custo não muda na aresta | onde muda, a refracção poda por ponto (`steiner_g`); cortar ali dava os mesmos nós (`24 615` contra `24 663`) e perdia, em teoria, as PONTAS de quem roça um canto — fica a escolha segura |
| `12` bissecções | o corte é conservador: menos passos só cortam menos. `60` → `347 ms`, `24` → `291`, `12` → `275` (os mesmos nós) |
| a gama de deslize de uma raiz de refracção é a ARESTA inteira (era «o que a raiz vê») | um par a peso 4 dava `1,0002` com a dominância: dois caminhos discretos de custo igual (`20,797590847` contra `…882`), e o que guarda um ponto a mais na quina arredondada da lama ficava preso — o polimento move um ponto de cada vez, e a gama curta não deixava a travessia ir para onde fica colinear. O polimento só aceita o que a caminhada real confirma |
| **a malha em listas CONTÍGUAS** (`ring_off`/`ring`/`nbrs`/`twin`); o `Poly` é uma vista | três listas por polígono eram `~43 000` alocações por construção (`0,62 ms` a criar e `0,28` a largar, medido à parte); a porta validada é uma só (`from_rings`) |
| a vizinhança e a sobreposição pelas arestas que SAEM de cada vértice | a mesma resposta (a 1.ª pela ordem dos polígonos), menos varrimento |
| **a fila do replaneio** (`ph2d_nav::refresh` + `bridge/nav_fila.rs`) | um caminho que ainda se anda não precisa de ser refeito no tique da mudança; um partido sim, e passa à frente |
| o orçamento em NÓS, com a estimativa = a última procura de cada um | determinístico (nada lê um relógio); o tempo varia com a máquina, a contagem não |
| a ordem: partidos, quem espera há mais tiques, a ordem das ENTIDADES (o `Ord` delas, o dos mapas da ponte) | sem o envelhecimento, uma malha que muda sem parar servia sempre os primeiros (gate) |
| o estado (`owed`, `broken`, `last_nodes`) no `AgentRuntime` | ele já vai no anel: um scrub para o meio da fila devolve a fila a meio (gate) |

### §17.3 — O que a medição derrubou

- **A 1.ª dominância custava mais do que poupava**: `2,6×` menos nós e o mesmo tempo — `0,4 µs` por nó
  contra `0,18`. A árvore por aresta saiu (uma lista por polígono), e as `60` bissecções desceram a `12`.
- **Repetir o corte até nada mudar** (uma frente pode cobrir a ponta depois de outra a cortar): `344 ms`
  contra `275`, e quase nenhum nó a menos. ⇒ As frentes que restam são LEGÍTIMAS: cada raiz da grelha de
  uma fronteira é a mais barata na sua fatia — o resto da distância à uniforme (`4,4×` nós) é a grelha.
- **A montagem incremental «reaproveitar os mosaicos que não mudaram»**: desenhada e medida antes de
  escrita — o que fica O(malha) depois das listas contíguas é a grelha (`0,46 ms`), as ilhas (`0,24`),
  o índice vértice→polígonos (`0,11`) e a ligação (`~0,8`); reaproveitar peças poupava `~0,5 ms`,
  porque inserir um mosaico desloca a numeração de todos os seguintes. A alavanca seguinte é outra
  (§17.6).
- **Um `!m.poly_count() == 0`**: a troca por script de `!m.polys().is_empty()` virou um NÃO bit-a-bit
  sobre o número — toda malha parecia vazia. Compila; apanharam-no os gates do agente (7 vermelhos).
- **A 1.ª cura da quina** («tirar cada raiz cujo caminho sem ela, polido de novo, não custa mais»): a
  mutação achou-a SOBREVIVENTE — sem ela a sonda dá os mesmos custos ao dígito, e a procura fica mais
  rápida (mediana `3,8 → 3,5 ms`). Saiu; a cura é só a gama.
- **A minha 1.ª fixtura do scrub da fila** movia a porta à mão (uma edição do mundo): o scrub reprovava
  também SEM a fila. A porta passou a ser uma curva da cena (`SceneAtTick`), como a timeline. E a 2.ª ia
  para o tique 9 — o anel guarda um âncora de 10 em 10, logo o scrub refazia tudo do zero e a mutação
  «o seed não devolve a dívida» SOBREVIVIA. Agora a porta muda no 8 e o scrub vai para o 13 (semeia do
  10, a meio da fila).
- **A ordem da fila pela consulta ao mundo** (a 1.ª redacção) servia os guardas ao contrário da ordem das
  entidades; e na fixtura as duas ordens coincidiam (a mutação sobrevivia) até metade dos guardas
  levar um componente de teste — a consulta percorre por tabelas.
- **A 1.ª fixtura de «o partido passa à frente»** punha o vigia em ÚLTIMO na ordem das entidades — o
  gate passava sem a prioridade. Agora a pré-condição está afirmada.

### §17.4 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| o corte repetido até ao ponto fixo | `344 ms` contra `275`, quase os mesmos nós |
| a dominância nas arestas onde o custo muda | os mesmos nós (`24 615` contra `24 663`); fica de fora pelo argumento das pontas |
| `60` (ou `24`) bissecções no corte | `347` / `291 ms` contra `275`, os mesmos nós |
| montar reaproveitando as peças dos mosaicos com a numeração de hoje | `~0,5 ms` de `2,0` — a numeração desloca-se; ver §17.6 |
| um orçamento abaixo de `20 000` nós | o tique não desce (`51,3` contra `51,7 ms` a 200) e a espera dobra (`97` tiques) |
| tirar os pontos a mais e polir de novo (a 1.ª cura da quina) | redundante com a gama da aresta inteira (os mesmos custos ao dígito) e mais lento |

### §17.5 — A prova

Gates novos: `a_dominancia_corta_nos_e_nunca_encarece_um_caminho` (CONTROLO sem a dominância: `26 493`
nós contra `43 543`, e nenhum caminho mais caro) · `a_quina_da_lama_nao_prende_o_polimento` (o par da
sonda, `≤ 1,0001` do oráculo) · `a_fila_serve_os_partidos_depois_os_mais_antigos_e_nunca_salta_a_frente`
· `a_porta_que_abre_um_atalho_serve_os_agentes_um_por_tique` (CONTROLO sem fila: os oito no mesmo tique)
· `o_caminho_partido_passa_a_frente_na_fila` · `um_scrub_para_o_meio_da_fila_devolve_a_mesma_corrida`
· `uma_malha_que_nao_para_de_mudar_serve_todos_a_vez` · `entalado_entre_duas_paredes_o_3d_nao_parte`
(visto VERMELHO, o mesmo panic, sem a cura). Os gates de sempre (incremental = a frio, por mosaicos = a
inteira, o oráculo exacto, o hash c9) passam sobre a malha contígua.
Mutação **15 / 15** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w9_2026-10-03.py`](ferramentas/mutacao_navegacao_w9_2026-10-03.py)); a 1.ª corrida deu
13/16 e as três sobreviventes estão no §17.3.

### §17.6 — ⏳ O que fica

- **Uma malha com ids FIXOS por mosaico** (o idioma do Detour: polígono = mosaico + índice local, com
  folga): a montagem passaria a ser proporcional ao mosaico tocado. Mexe no núcleo da procura (todo
  `u32` de polígono), por isso é uma wave própria. Hoje uma porta custa `2,65 ms` (`0,6` o mosaico,
  `2,0` a montagem).
- ✅ ~~O tique a 200 agentes SEM mudança nenhuma é `37,6 ms`~~ — **curado (§17.7)**: era o desvio a varrer
  as arestas da malha inteira por agente; com a grelha das paredes, `2,3 ms`.
- **A procura ponderada continua `~4×` os nós da uniforme** (a grelha das fronteiras, §17.3) e `~12×` o
  tempo na mediana; e o orçamento da fila em nós não é tempo uniforme entre as duas (`~110 ns` por nó na
  uniforme, `~330` na ponderada: os totais da §17.1 sobre os nós).
- A 1.ª procura de agentes que nascem juntos não passa pela fila (só a mudança de malha passa).

### §17.7 — (depois do smoke) O tique à escala: o desvio varria as paredes da malha inteira

✅ **Smoke do dono APROVADO (03/10)** — as cenas `=3` e `=4` com a grelha e os troços.

O CONTROLO da `medir_replaneio` (200 agentes, nada a mudar) era `37,6 ms`. Medido por fase (cronómetros
provisórios na ponte, `FASES=200`, load `~7` — as proporções valem): **o desvio era `97 %`** do tique
(`35,0` de `36,3 ms`); a condução `0,4`, os movers `0,2`, o passo da física `0,2`. E crescia LINEAR nos
agentes (`8,9 ms` a 50): por agente, `Walls::near` percorria TODAS as arestas de parede da malha
(milhares, com `1 000` obstáculos).

**A cura:** uma grelha das arestas dentro de `ph2d_orca::Walls` (montada uma vez, com as paredes, quando
a malha muda; o lado da célula sai da contagem — `~1` aresta por célula, a regra da grelha da
`NavMesh`). `near` lê só as células ao alcance e devolve, **ao bit**, o que a varredura inteira devolvia
(os mesmos índices pela mesma ordem — gate `a_grelha_das_paredes_da_o_mesmo_que_a_varredura_inteira`,
2 000 perguntas com alcances de zero a infinito e posições fora da caixa; a paridade com o Godot verde).

| tique, nada a mudar (`FASES=<n>`) | 10 agentes | 50 | 200 |
|---|---|---|---|
| antes (o CONTROLO da §17.1) | `2,1 ms` | `10,5` | `37,6` |
| **com a grelha** | **`0,38`** | **`0,82`** | **`2,29`** |

**E o tique em que a malha muda** (cronómetros por fase, 200 agentes, a porta a andar e a parar): a FILA
custava `12,4 ms` — `path_still_walkable` percorria o caminho inteiro (`~100 m`) de cada um dos 200 na
malha nova —, e a malha refeita `2,7–3,3 ms` (duas vezes: quando a porta começa a andar e quando pára).
⇒ **só se percorrem os troços que tocam os mosaicos REFEITOS** (`TiledMesh::changed_area`; fora deles a
geometria é a mesma — andava, anda). Gates `so_os_trocos_que_tocam_a_mudanca_se_percorrem` (o atalho
dispara: com a zona longe, um caminho partido nem se percorre) e `a_area_que_mudou_e_o_mosaico_da_pedra`.
De bónus, só é «partido» quem a mudança partiu: o último servido passa de `9` a `4` tiques.

| o pior tique depois de uma porta (fila a `20 000`), 10 · 50 · 200 agentes | |
|---|---|
| antes da W9 | `10,4 · 34,3 · 124,3 ms` |
| com a fila | `6,9 · 16,5 · 51,7` |
| + a grelha das paredes | `5,7 · 8,3 · 18,1` |
| **+ só os troços que tocam a mudança** | **`5,7 · 6,5 · 10,1`** (load `5–8`) |

Mutação: M17–M20 sangram — a prova da W9 fica em **19 / 19**.

## §18 — W10 (2026-10-03): a montagem proporcional ao mosaico tocado

### §18.1 — A medição que abre (`medir_custo` `SO_GRANDE=1`, `--release`, load `0,7`)

Uma porta (sem lamas) = **`2,66 ms`**, por fase (cronómetros provisórios, o mínimo de 5):

| o mosaico refeito | juntar (`monta`) | validar | vértice→polígonos | arestas que saem | sobreposição | vizinhança | ilhas | grelha |
|---|---|---|---|---|---|---|---|---|
| `0,68` | `0,32` | `0,18` | `0,09` | `0,09` | `0,36` | `0,32` | `0,15` | `0,44` |

A montagem é `1,95` dos `2,66 ms`, e TODA ela é O(malha). E a ponte, a cada mudança, refaz ainda a grelha das
paredes do desvio: **`0,53 ms`** (`Walls::from_walkable_walls`, `24 462` paredes, `13 671` polígonos).

### §18.2 — As perguntas que o briefing mandava fazer antes

| pergunta | resposta (medida / lida no código) |
|---|---|
| a porta mudar a malha DUAS vezes é a alavanca mais barata? | ⛔ não: é a LEI do §14.2 — um cinemático só é obstáculo PARADO (a velocidade do solver); andar e parar são duas geometrias diferentes, e o relógio de espera já está recusado |
| ids FIXOS por mosaico (o Detour: polígono = mosaico + índice local, com folga) servem a alguém? | ⛔ não: NINGUÉM guarda um id de polígono através de uma mudança — o agente guarda o caminho em PONTOS e esquece-o (`forget_path`), a dominância e a procura recomeçam por consulta. Os ids fixos só trariam BURACOS na numeração a todos os consumidores que varrem `0..poly_count` |

### §18.3 — O desenho: numeração densa e canónica, montada por BLOCOS com cache

A malha montada fica **a mesma de hoje, ao bit** (a mesma numeração de vértices — o 1.º mosaico pela ordem da
chave que tem o ponto —, os mesmos anéis cosidos, vizinhos, gémeos, cantos, ilhas, paredes); muda QUEM a
calcula. `ph2d_nav::MalhaPorBlocos` (a porta da malha por mosaicos, dentro da `ph2d-nav`, que valida tudo o que
recebe) guarda por mosaico três camadas:

| camada | depende de | o quê |
|---|---|---|
| L1 | só a peça do mosaico | a grelha de localização DO MOSAICO, a caixa, os vértices de cada lado |
| L2 | L1 dele e dos 8 vizinhos | o «dono» de cada ponto de costura, os anéis cosidos (as junções em T), a validação, a vizinhança interna, as componentes |
| L3 | L2 dos dois lados de uma costura | a vizinhança através da costura (e a sobreposição entre mosaicos) |

e a montagem final são só passagens lineares (numerar, traduzir, paredes, cantos, vértice→polígonos, ilhas por
união das componentes). A localização passa a uma grelha POR MOSAICO (a resposta de `locate_all` é a mesma — a
grelha nunca foi observável). O `monta` antigo fica como ORÁCULO dos gates: montagem por blocos = montagem
inteira, campo a campo.

### §18.4 — As medições (`--release`, antes/depois ALTERNADOS na mesma máquina, load `5–9`)

O «antes» é o fecho da W9 (`de2d23cae`) num worktree temporário com o seu `CARGO_TARGET_DIR`.

| `medir_custo` §4 (o mínimo de 5) | antes | **W10** |
|---|---|---|
| uma porta (sem lamas) | `2,66 · 2,67 ms` | **`1,14 · 1,20 ms`** |
| uma porta (com 100 lamas) | `3,77 · 3,78` | **`1,82 · 1,89`** |
| uma lama a mexer | `5,23 · 5,27` | **`3,62 · 3,73`** |
| a frio (com 100 lamas) | `57,1 · 57,2` | `59,4 · 61,5` (o cache das camadas; ruído da carga) |

Por fase (cronómetros provisórios, load `1,7`, uma porta sem lamas = `1,13 ms`): o mosaico refeito `0,70` ·
L2 (só o mosaico) `0,025` · L3 `0,005` · numerar `0,044` · anéis + paredes + cantos `0,25` · vértice→polígonos
`0,075` · ilhas `0,005` · grelhas `0,003`. ⇒ a montagem passou de **`1,95` a `0,41 ms`**; o que resta O(malha)
são passagens lineares de cópia e tradução.

`medir_replaneio` (a fila a `20 000`, o pior tique depois de uma porta, mediana de seis), 10 · 50 · 200 agentes:

| | antes | **W10** |
|---|---|---|
| pior tique | `5,17–5,35 · 5,92–5,96 · 8,44–8,47 ms` | **`3,63–3,73 · 4,39–4,44 · 6,95–7,12`** |
| procuras num tique · tiques até a fila esvaziar · até o último partido | `9 · 11 · 13` · `6 · 19 · 56` · `0 · 4 · 4` | **iguais** (a mesma malha ⇒ a mesma corrida) |

### §18.5 — As decisões

| decisão | porquê (medido) |
|---|---|
| a numeração densa e canónica de hoje, montada por blocos com cache (e NÃO ids fixos com folga) | ninguém guarda um id através de uma mudança (§18.2); a malha montada fica a MESMA ao bit, logo todos os consumidores, o hash e o replay ficam intactos — e o que sobra O(malha) são `0,37 ms` de cópia |
| um vizinho referido pela POSIÇÃO no lado (o `k`-ésimo ponto do lado), não pelo índice local | um mosaico refeito renumera os seus pontos; pelo índice, os 8 vizinhos recosiam-se SEMPRE (`0,27 ms`); pela posição, só quando os pontos do lado mudam (`0,025`) |
| as paredes e os cantos na passagem dos anéis | uma passagem a menos sobre a malha (`0,12 ms`) |
| a peça DENTRO do rectângulo é um `assert!` (era `debug_assert!`) | é o que torna impossível a mesma aresta orientada em dois blocos (num polígono anti-horário convexo, as arestas do lado direito sobem e as do vizinho descem) — e por isso a sobreposição só se confere DENTRO de cada bloco |
| o `monta` inteiro de antes fica como ORÁCULO dos gates | `a_montagem_por_blocos_e_a_montagem_inteira_ao_bit`: campo a campo + `locate_all`, a frio, incremental, com lamas e com a região a encolher |

### §18.6 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| a porta mudar a malha só UMA vez | é a lei do §14.2 (um cinemático só é obstáculo PARADO) |
| ids FIXOS por mosaico (Detour, com folga) | nenhum consumidor os usaria (§18.2); os buracos na numeração chegariam a todos os que varrem `0..poly_count` — e o ganho sobre a numeração densa seria só a cópia (`≤ 0,37 ms`) |
| recoser os 8 vizinhos a cada mosaico refeito | `0,27 ms` contra `0,025` (a referência pela posição no lado) |

### §18.7 — A prova

Gates novos: `a_montagem_por_blocos_e_a_montagem_inteira_ao_bit` (CONTROLOS de população: `115` vértices
cosidos — as junções em T só nascem de um obstáculo que ENCOSTA a uma costura de um lado só, e a fixtura
põe-nos de propósito —, `72` actualizações parciais, `12` regiões que encolhem, `16 078` pontos localizados) ·
na `ph2d-nav`: as recusas com o índice da malha montada (e a recusa que NÃO fica), o índice fora e as áreas,
a peça fora do rectângulo, o ponto de costura repetido, tirar um bloco, o vizinho com os mesmos pontos de
lado que deixa de ligar, dois pontos cosidos numa aresta que desce, o ponto a EPS da fronteira dos blocos.
Os de sempre (incremental = a frio, por mosaicos = inteira, a junção em T, o oráculo exacto, o hash e o
replay da ponte) verdes sobre a montagem nova.
Mutação **21 / 21** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w10_2026-10-03.py`](ferramentas/mutacao_navegacao_w10_2026-10-03.py)); a 1.ª corrida deu
17/22 — quatro ramos sem fixtura (tirar um bloco, o vizinho que deixa de ligar, a ordem descendente, a
fronteira exacta da grelha dos blocos), curados com as quatro fixturas acima, e uma EQUIVALENTE (a parede que
só marca o canto `u`: a ponta `v` é a `u` da parede seguinte), que saiu do arnês com o porquê.

### §18.8 — ⏳ O que fica

- **A grelha das paredes do desvio** (`ph2d_orca::Walls::from_walkable_walls`) refaz-se inteira a cada
  mudança: **`0,53 ms`** (`24 462` paredes) — agora UM TERÇO do custo de uma porta na ponte. A ordem das
  paredes desempata distâncias iguais (um canto partilhado) e entra no hash: uma versão incremental tem de dar
  os MESMOS índices — o molde é este (blocos + oráculo).
- **O mosaico refeito** (`0,70 ms`: o corte, a união, a triangulação de um mosaico com `~20` obstáculos) é
  agora o maior pedaço de uma porta.
- O que resta O(malha) na junção (`0,37 ms`: anéis, vértice→polígonos, numerar) — só desce com ids fixos,
  recusados acima enquanto ninguém os consumir.

## §19 — W11 (2026-10-04): as paredes do desvio proporcionais ao mosaico tocado

### §19.1 — A medição que abre (sonda `sonda_paredes_w11`, `--release`, load `4–5`, o mínimo de 20)

Cena: `100 × 100 m`, `1 000` círculos de raio `0,3–1,5` (LCG `77`), `TiledMesh::new(Params::default(),
TILE_M)` ⇒ `13 671` polígonos, `24 462` paredes, `24 489` vértices. `Walls::from_walkable_walls` real =
**`0,557 ms`**; uma CÓPIA do algoritmo, fase a fase:

| índices por vértice | `point` | `next`/`prev` | `dir` (sqrt) | `convex` | grelha: caixas | grelha: 2 passagens |
|---|---|---|---|---|---|---|
| `0,022` | `0,014` | `0,026` | `0,073` | `0,024` | **`0,203`** | **`0,147`** |

A grelha é `70 %`. A pergunta do briefing — **uma construção linear mais rápida basta?** — mediu-se: a mesma
grelha AO BIT (a célula por `as usize`, caixas em `u32`, a aresta de uma só célula sem laço) dá `0,285 → 0,236
ms`. ⛔ Não basta: o todo ficaria em `~0,4 ms`.

O PISO de um desenho por blocos (o que fica O(malha)): concatenar os cinco vectores finais em 49 blocos
**`0,013 ms`**; reler as coordenadas de TODAS as paredes e compará-las com as de antes **`0,020 ms`**.

### §19.2 — O kill-criterion (escrito ANTES do código)

1. **K1 — a mesma resposta, ao bit:** as paredes por blocos dão os MESMOS `point`/`next`/`prev`/`dir`/`convex`
   que `from_walkable_walls(m.verts(), m.walls())`, e `near` = a varredura inteira, sobre fixturas com
   população AFIRMADA (cadeias que atravessam costuras, junções em T, a borda da região, um vértice onde a
   fronteira se toca, mosaicos que entram e saem). Inegociável.
2. **K2 — o custo:** as paredes depois de uma porta (reler + comparar + refazer os blocos mudados + concatenar)
   **≤ `0,15 ms`** na cena acima (era `0,557`). Se passar de `0,25 ms` depois da 2.ª tentativa, o desenho por
   blocos NÃO existe nesta forma e fica a grelha linear rápida (`~0,4 ms`).
3. **K3 — a consulta não piora:** o pior tique de `medir_replaneio` a 200 agentes não sobe acima do ruído
   (alternado antes/depois), e as procuras e a fila ficam IGUAIS.

### §19.3 — As perguntas medidas antes do desenho

| pergunta | resposta (medida / lida no código) |
|---|---|
| o índice de uma parede é observável além da ORDEM? | não: o desvio só o usa como etiqueta (`point`/`next`/`prev`/`dir`/`convex` de `i`) e como desempate em `near` — logo as paredes podem viver por bloco, com o índice = base do bloco + local |
| dois vértices distintos da malha podem ter o mesmo ponto? | não (a triangulação funde por ponto; a junção dos blocos numera um ponto uma vez) ⇒ **um vértice é o seu ponto, ao bit**, e os blocos reconhecem-se pelas coordenadas — sem a numeração global, que muda a cada mudança |
| um ponto de dentro de um mosaico pode aparecer noutro? | não (a peça é um `assert!` dentro do rectângulo; os rectângulos só se tocam na borda, que coincide ao bit com `lo`/`hi`) ⇒ só os pontos da BORDA ligam blocos |
| a grelha das paredes é observável? | não: `near` = a varredura inteira (gate W9), qualquer grelha serve ⇒ uma grelha por bloco, com índices locais |

### §19.4 — O desenho: `ph2d_orca::ParedesPorBlocos`

A `ph2d-orca` continua com ZERO dependências: recebe, por bloco (a chave inteira da grelha regular, o
rectângulo, e as paredes `(de, para)` em COORDENADAS), o que a ponte lê de `TiledMesh::paredes_por_mosaico`
(uma `ph2d_nav::FaixaDeParedes` por bloco: `walls()[faixa]`, gravada pela junção da W10).

| camada | depende de | o quê |
|---|---|---|
| L1 | as paredes do bloco | o seguinte/anterior pelos pontos de DENTRO (duas listas ordenadas + duas junções lineares), `dir`/`convex` das entradas de dentro, os pontos da borda (a 1.ª que lá começa / acaba), a grelha do bloco |
| L2 | L1 dele e dos vizinhos que têm o ponto | só as entradas PENDENTES (o `de` ou o `para` na borda): a 1.ª entrada, pela ordem das chaves, entre os blocos cujo rectângulo tem o ponto |

`poe` compara as paredes com as de antes, ao bit: iguais = nada (é o que faz uma porta refazer 2 blocos e não
49). `monta` recose os blocos sujos (o mudado e os 8 vizinhos) e concatena. `Walls::near` lê os blocos cujo
rectângulo toca a caixa do alcance; uma construção inteira (`from_polygons`, `from_walkable_walls`) é UM bloco,
a caixa — a mesma porta de busca.

### §19.5 — As medições (`--release`, a sonda e `medir_replaneio`)

`sonda_paredes_w11` (a cena do §19.1, o mínimo de 10–20, load `3–5`):

| | antes | **W11** |
|---|---|---|
| as paredes depois de uma porta | `0,557 ms` (inteiras) | **`0,149 ms`** (pôr `0,098` · montar `0,050`; 2 blocos refeitos, 12 recosidos) |
| nada mudou (o piso O(malha): comparar + concatenar) | — | `0,056 ms` (`0,032` + `0,023`) |
| a frio (a 1.ª vez de uma malha) | `0,557` | `1,67 ms` — só quando a malha inteira nasce (`~57 ms`) |
| `near` × 20 000 | `26,3–27,7 ms` | `26,9–28,5 ms` (`+2–3 %`: as `partition_point` da grelha dos blocos) |

As três tentativas até ao K2: a 1.ª L2 recalculava `dir`/`convex` de TODAS as entradas dos 12 blocos recosidos
(`0,306 ms`); a 2.ª deixou à L2 só as pendentes (`0,229`); a 3.ª trocou, na L1, uma busca binária por entrada
por duas junções lineares (`0,149`).

`medir_replaneio` (antes = `cd7a87e94`, alternado 2×, load `4,5–6,6`): **as procuras e a fila IGUAIS nas 12
linhas**; o pior tique depois de uma porta a 200 agentes (fila a `20 000`) `7,23 · 7,73 → 7,01 · 7,04 ms`; o
CONTROLO (nada a mudar) sem subida.

### §19.6 — As decisões

| decisão | porquê (medido) |
|---|---|
| um vértice pelo PONTO ao bit (e não pelo índice da malha) | a numeração global muda a cada mudança; o ponto não — e é a mesma identidade (§19.3) |
| a chave do bloco = a do mosaico, os vizinhos pelas chaves `±1` | é a ordem das paredes da malha (a da junção da W10) e a do desempate |
| `poe` compara ao bit em vez de a malha dizer quem mudou | `0,032 ms` por toda a malha e nenhuma contabilidade partilhada entre as duas crates a errar |
| a grelha da busca por bloco também na construção inteira (um bloco) | uma só porta de `near`; o oráculo da W9 (`near` = varredura) guarda-a |
| a concatenação O(malha) fica | `0,023 ms`; tirá-la pediria índices por bloco no caminho quente do desvio |

### §19.7 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| só acelerar a construção inteira | a grelha AO BIT mais depressa dá `0,285 → 0,236 ms`; o todo ficaria em `~0,4` |
| a L2 refazer `dir`/`convex` de todas as entradas dos blocos recosidos | `0,116 ms` contra `0,050` (só as pendentes) |
| uma busca binária por entrada na L1 | `~70 µs` por bloco contra `~33` (duas junções lineares) |

### §19.8 — A prova

Gates novos:
- `as_paredes_por_blocos_sao_a_construcao_inteira_ao_bit` (`ph2d-orca`): blocos sintéticos numa grelha grossa,
  120 passos (blocos que mudam, saem, voltam, ou são postos iguais), campo a campo + `near` = varredura.
  CONTROLOS medidos: `663` entradas que seguem para outro bloco · `1 656` pontos onde começam várias · `1 908`
  sem seguinte · `36` postos iguais · `32` tirados.
- `so_se_refaz_o_bloco_que_mudou_e_os_vizinhos` (as `Contas`: igual = `0/0`; um do meio = `1/9`; um canto que
  sai = `0/3`) · a parede fora do rectângulo e o bloco fora da grelha são `panic`.
- No oráculo da W10 (`ph2d-navmesh`, as malhas reais com junções em T): as paredes por mosaicos que VIVEM entre
  actualizações = `from_walkable_walls`. ⚠️ O CONTROLO acusou **zero** pontos onde a fronteira se toca: a
  fixtura ganhou dois quadrados encostados por uma quina em cima de um canto de quatro mosaicos (`24`); e
  `3 200` entradas atravessam costuras.
- `as_paredes_da_ponte_por_mosaicos_sao_as_da_malha_inteira` (a ponte, pela porta que o desvio usa — a região
  encolhe a meio; `1 116` atravessam) · as faixas vazias numa malha recusada (`ph2d-nav`).

Mutação **31 / 31** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w11_2026-10-04.py`](ferramentas/mutacao_navegacao_w11_2026-10-04.py)). ⛔ A 1.ª corrida deu
**29/30**: apagar a invalidação das paredes montadas (`nav_malha.rs`) SOBREVIVIA — nenhum gate olhava as
paredes depois de a malha mudar. A cura foi no desenho, não numa fixtura: `TiledMesh::versao` sobe a cada
mudança e `ParedesDaMalha::paredes` compara-a, logo a frescura vive na porta que o gate da ponte exerce (e o
descarte noutro ficheiro deixou de existir). E o controlo (b) do arnês apanhou na 1.ª tentativa um filtro por
PREFIXO (`nav_desvio::`) que nunca casa um nome inteiro.

### §19.9 — ⏳ O que fica

- **O mosaico refeito** (`0,70 ms`: o corte, a união, a triangulação de um mosaico com `~20` obstáculos) — o
  maior pedaço de uma porta.
- O que resta O(malha) a cada mudança: a junção da malha (`0,37 ms`, §18.8) e, nas paredes, comparar
  (`0,032`) + concatenar (`0,023`).
- A construção a frio das paredes (`1,67 ms` contra `0,56`): só quando a malha inteira nasce.

## §20 — W12 (2026-10-04): o mosaico refeito mais depressa

### §20.1 — A medição que abre (sonda `sonda_mosaico_w12`, `--release`, load `1,6`, a mediana de 20 portas)

A cena GRANDE de `medir_custo` §4 (`100 × 100 m`, `1 000` obstáculos, LCG `77`; a porta = o obstáculo 3), `TILE_M`.
Uma porta refaz **1** mosaico. Por fase (cronómetros provisórios, ms):

| fase | sem lamas | com 100 lamas |
|---|---|---|
| **o mosaico refeito (`constroi`)** | **`0,230`** | **`0,521`** |
| — corte da região + inflar e cortar os obstáculos | `0,012` | `0,012` |
| — união dos furos (Clipper) | `0,034` | `0,037` |
| — diferença região − furos (Clipper) | `0,029` | `0,029` |
| — as lamas (interseção/diferença/união por área, Clipper) | `0` | `0,160` |
| — limpar os anéis, as junções em T, numerar, as restrições | `0,015` | `0,067` |
| — a triangulação com restrições (`spade`) | `0,073` | `0,102` |
| — dentro/fora (paridade) | `0,010` | `0,020` |
| — a fusão em convexos (Hertel–Mehlhorn) | `0,053` | `0,085` |
| a montagem (`MalhaPorBlocos::monta`) | `0,245` | `0,317` |
| as assinaturas e os baldes de TODOS os obstáculos | `0,074` | `0,087` |
| **uma porta** | **`0,569`** | **`0,951`** |

⚠️ **A premissa do briefing caiu em parte:** o «`0,70 ms` do mosaico refeito» (§18.4) foi medido a load `1,7–9`;
a frio de carga é **`0,23`**, e a montagem (`0,245`) já pesa o mesmo. O tamanho do mosaico NÃO é alavanca: é a
recusa medida do §14.1 (a procura paga as costuras até `1,8×` fora de `15`/`20 m`).

### §20.2 — O kill-criterion (escrito ANTES do código)

1. **A mesma malha, ao bit:** a impressão digital da sonda (`IMPRESSAO=1`: 3 cenas × 4 combinações de raio, lamas
   e fusão × 4 mudanças — vértices, anéis, áreas) é `71c5f70e3d312ee3` antes e depois; e as funções antigas ficam
   como ORÁCULO dos gates. A triangulação (`spade`) e o Clipper NÃO mudam: trocar um deles muda a malha.
2. **O custo:** o mosaico refeito **≤ `0,17 ms`** sem lamas e **≤ `0,40`** com lamas. Se depois da 2.ª tentativa
   ficar acima de `0,20` / `0,45`, o que não chegou fica escrito como o piso das duas bibliotecas.

### §20.3 — O que mudou (a mesma malha, ao bit)

| fase | antes | agora |
|---|---|---|
| a fusão em convexos | um `BTreeMap` das semi-arestas; `try_merge` copiava os dois anéis antes de decidir; um `Vec` por triângulo | as semi-arestas num vector ORDENADO (numa repetida vale a do último triângulo — o `insert` de antes); decide antes de copiar; um anel só ganha memória quando funde |
| a numeração e as restrições | um `BTreeMap<P, u32>` e um `BTreeMap<aresta, Vec<pedaço>>` | duas ordenações: cada ponto pela 1.ª aparição, cada restrição pela 1.ª aparição e com o sentido dela, e os donos de uma aresta por `partition_point` |
| as junções em T | um `BTreeMap` de listas de baldes, cada uma com `contains` | os pontos únicos numa grelha contígua (a ordem num balde não conta: os candidatos ordenam-se por `(t, p)`) |
| as assinaturas | FNV-1a byte a byte; a caixa de cada obstáculo com uma lista nova | por palavra, cada uma pelo finalizador do `splitmix64` (sem ele, dois sinais trocados anulavam-se — gate); a caixa sem lista |

### §20.4 — As medições (`--release`, o melhor de 8 alternado antes/depois, cronómetros provisórios)

| | antes | **W12** |
|---|---|---|
| o mosaico refeito, sem lamas | `0,217` | **`0,190 ms`** (`−12 %`) |
| o mosaico refeito, com 100 lamas | `0,503` | **`0,446`** (`−11 %`) |
| — a fusão | `0,051 · 0,084` | `0,031 · 0,053` |
| — limpar, junções em T, numerar | `0,015 · 0,067` | `0,011 · 0,041` |
| as assinaturas e os baldes | `0,072 · 0,086` | `0,046 · 0,057` |
| **uma porta** | `0,547 · 0,929` | **`0,486 · 0,824`** (`−11 %`) |

E sem cronómetros, `medir_custo` `SO_GRANDE=1` alternado (load `22–24`, os números inflados mas a razão justa):
uma porta `1,16–1,26 → 1,06–1,12 ms` sem lamas e `1,85–1,96 → 1,71–1,78` com lamas; a frio com lamas
`58,9–63,3 → 54,1–57,8`.

⛔ **O kill-criterion não foi atingido** (`≤ 0,17 · ≤ 0,40`), e ficou dentro da 2.ª banda (`≤ 0,20 · ≤ 0,45`). O
que resta do mosaico é o **PISO das duas bibliotecas**: sem lamas, `spade` `0,067` + Clipper `0,060` + o corte
`0,011` + a paridade `0,009` = `0,147` de `0,190`; com lamas, mais `0,156` do Clipper nas áreas. Trocar qualquer
delas muda a malha (o kill-criterion 1).

### §20.5 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| mosaicos mais pequenos | §14.1: a procura paga as costuras até `1,8×` fora de `15`/`20 m` |
| assinatura por palavra SEM mistura | `(1, 2)` e `(−1, −2)` davam a mesma (o bit do sinal anula-se) — o mosaico não se refazia (gate `a_assinatura_distingue_os_sinais_trocados`) |
| a união e a diferença numa só chamada do Clipper, ou cortar o chão à caixa de uma área antes de a intersectar | a geometria seria a mesma, a malha não (a ordem e o início dos anéis) — fora do kill-criterion 1 |

### §20.6 — A prova

Gates: `a_triangulacao_e_a_fusao_de_agora_sao_as_de_antes_ao_bit` (as funções de antes, verbatim, em
`triangulate_oraculo.rs`, sobre os pedaços REAIS de 36 cenas — caixas rodadas, círculos, cápsulas, três raios,
os dois cantos, lamas sobrepostas; CONTROLOS medidos: `30` cenas com vários pedaços, `10` pontos das junções em
T, `2 069` arestas com dois donos, `10 259` fusões de `23 781` triângulos) · `a_assinatura_distingue_os_sinais_trocados`.
A impressão digital da sonda (`71c5f70e3d312ee3`) igual antes, a meio e depois. Para isso a construção inteira
expõe `chao_e_areas` e `pedacos` (`pub(crate)`), a mesma porta que `build_with_areas` e `poligonos` usam.

Mutação **10 / 10** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w12_2026-10-04.py`](ferramentas/mutacao_navegacao_w12_2026-10-04.py)); duas EQUIVALENTES saíram
com o porquê: o par de uma semi-aresta sem conferir o oposto (o `try_merge` recusa o par errado), e a ordem
das restrições (o `spade` 2.15 não depende dela em nenhuma fixtura — a ordenação fica, pela ordem de antes).
⚠️ A 1.ª corrida do fecho acusou `no_std_transcendental_reaches_the_deterministic_hash`: a fixtura do oráculo
rodava as caixas com `cos`/`sin` do `std`, e a `ph2d-navmesh` inteira está no caminho do hash (a varredura lê o
FONTE, testes incluídos). A rotação passou a uma direcção ao calhas normalizada por `sqrt`; as populações foram
re-medidas e a mutação re-corrida sobre o código final (`10 / 10`).

### §20.7 — ⏳ O que fica

- **A montagem** (`MalhaPorBlocos::monta`, `0,24 ms` sem lamas) é agora o MAIOR pedaço de uma porta, e o que
  resta O(malha) nela só desce com ids fixos (recusados no §18.6 enquanto ninguém os consumir).
- O piso das bibliotecas no mosaico (`spade` + Clipper, `0,15 ms`; `+0,16` com lamas).

## §21 — W13 (2026-10-04): o comportamento que faltava (os abertos da W5 e da W9)

A ordem do dono (04/10): *«antes de juntar vamos fechar completamente a feature»*. O inventário de TODOS os
abertos do plano (§13.5 … §20.7) separou o que já estava fechado por uma wave seguinte (o *«a dar passagem»*,
a porta que anda como um círculo, todos a replanear no mesmo tique, a grelha das paredes), as recusas medidas
(ids fixos e a montagem O(malha), a porta que muda a malha duas vezes, o piso das bibliotecas, o tamanho do
mosaico), um plano à parte (plataformas) e o que estava MESMO por fazer. Esta wave fecha o comportamento:

| aberto | o defeito | a cura | o gate (e o que media sem a cura) |
|---|---|---|---|
| W5: o empurrão de um golpe | o desvio lia só o comando do mover de vista de cima; o corpo anda com o comando MAIS o empurrão | `velocidade_de` = a mesma soma do mover | `um_corpo_empurrado_por_um_golpe_e_visto_a_andar`: um herói atirado ATRAVÉS do caminho — `0,80 m` de desvio de quem já ia a sair, agora `0,0016` (= o CONTROLO) |
| W5: o corpo composto | o desvio via um corpo que anda só pelo colisor principal | os discos de cada peça no referencial do corpo; o disco de um ALVO envolve o corpo e as peças | `um_corpo_composto_desvia_se_pela_forma_inteira`: o carrinho com um braço atravessado — folga `0,006 → 0,80 m`; perseguido, `0,009 → 0,78 m` fora do eixo a 1 m do braço |
| W9: quem nasce junto | a 1.ª procura de quem não tem caminho não passava pela fila — cinquenta a nascer = cinquenta procuras num tique | é do orçamento do tique: cabe enquanto houver folga (a fila devolve o que prometeu; as outras contam pelo que gastam), sempre pelo menos uma; quem não cabe espera PARADO | `nav_nascer.rs` (um por tique pela ordem das entidades; quem espera fica parado e todos chegam; o scrub a meio) · `quem_nasce_com_a_fila_cheia_procura_um_e_o_outro_espera` |

⚠️ **A condução passa a correr pela ordem das ENTIDADES** (era a da consulta, a das tabelas): o 1.º gate do
scrub dos nascimentos DIVERGIA — o rebuild do scrub baralha as tabelas e servia outro agente. E a ordem das
entidades não é a de nascimento (o `Entity` desta versão do `bevy_ecs` ordena ao contrário): os gates calculam-na.

⚠️ **Achados de passagem:** (1) um obstáculo LARGO que anda contra o agente (ou à frente dele, na mesma direcção)
PRENDE-o — o desvio local (ORCA) escolhe a velocidade mais perto da pedida e nunca contorna; parado, o obstáculo
vira parede da malha e o caminho contorna-o (o gate do composto pára o carrinho por isso). É o limite conhecido
do ORCA, fica no §21.2. (2) O gate do caminho do hash apanhou um `hypot` do `std` no disco do alvo (curado com
`sqrt`). (3) O gate da fila que abre o atalho fixava o orçamento ANTES do nascimento; com a 1.ª procura no
orçamento, os oito nasciam um por tique ainda a meio da porta — a premissa passou a explícita (o orçamento do
teste vale a partir do tique `2`).

Mutação **8 / 8** a sangrar ([`mutacao_navegacao_w13_2026-10-04.py`](ferramentas/mutacao_navegacao_w13_2026-10-04.py));
a 1.ª corrida deu **5/8**: a K1 tirava só a componente x de um golpe vertical (o arnês), e a N2 e a N5 (o
«sempre pelo menos um» e o que a fila prometeu) não tinham fixtura que juntasse a fila CHEIA e quem nasce — o
gate da fila cheia nasceu disso.

### §21.2 — ⏳ O que fica (a velocidade, wave seguinte)

- **A fila conta NÓS, e um nó da procura ponderada custa `~3×` o da uniforme** (`~330` contra `~110 ns`): com
  muita lama a fila deixa passar mais tempo do que o orçamento diz.
- **O início de uma cena grande** (`100 × 100 m`, `1 000` obstáculos, `100` lamas): `~55–60 ms` a montar a malha a
  frio — um engasgo ao dar play.
- **O desvio a `1 000` agentes**: `2,4 ms` a varrer os candidatos.
- **A procura ponderada com muita lama**: `~4×` os nós e `~12×` o tempo da uniforme.
- **Um obstáculo largo que anda prende o agente** (o limite do ORCA — achado acima).
- O passo 7–8 do tutorial 03 prova-se no smoke do dono (a foto não clica).

## §22 — W14 (2026-10-04): a velocidade em cenas grandes

A lista é a do §21.2, por esta ordem. Cada item: a medição, o kill-criterion escrito ANTES do código, o
código, o gate com CONTROLO, a mutação. O que a medição mostrar não valer a pena fica como recusa medida.

### §22.1 — A fila contava NÓS; o tempo de um nó não é um só

**A medição que abre** (sonda provisória `sonda_custo_por_no_w14` da `ph2d-navmesh`: a malha por mosaicos
da cena grande, `100 × 100 m`, `1 000` obstáculos, com `30`, `100` e `300` lamas, pesos `1` · `1,5` · `4` ·
`10`, `240` pares — metade curtos —, o mínimo de 7 por consulta, `--release`; contadores provisórios na
procura). O tempo por nó EXPANDIDO, por grupo (load `20–40` — os tempos absolutos estão inflados; a forma
não):

| lamas | uniforme | peso 1,5 | peso 4 | peso 10 |
|---|---|---|---|---|
| `30` | `104 ns` | `181` | `185` | `184` |
| `100` | `110` | `225` | `248` | `261` |
| `300` | `117` | `268` | `338` | `376` |

⇒ não há UM factor para a ponderada (o «`~3×`» do §17.6 era a média de uma cena): o nó custa mais quanto
mais FRENTES há na mesma aresta (a dominância, `polyanya_dominancia.rs`, percorre a lista do polígono) e
quanto mais raízes de FRONTEIRA se materializam (`Kind::Pending`, que o `expanded` não conta). Ajuste por
mínimos quadrados do tempo de cada consulta sobre três contagens — nós expandidos `≈127 ns`, raízes
materializadas `≈590`, frentes comparadas `≈18` —: real/modelo `0,82–1,03` em todos os grupos (só os
expandidos: `0,30–1,08`, e a uniforme lia-se `3×` acima do que custa face à ponderada de `300` lamas).

Na ponte (`medir_replaneio`, agora com `LAMAS=<n>`; orçamento `20 000`, load `25–39`): sem lama o pior
tique depois da porta é `4,2 · 6,0 · 11,1 ms` (10 · 50 · 200 agentes); com `150` lamas a peso 4,
`55 · 97 · 112 ms` — o orçamento deixa passar o mesmo número de nós, e cada nó vale mais.

De passagem: com a tabela de custos SEM lama (`[1.0]`) cada procura varria os `~15 000` polígonos para
saber que é uniforme (`5–7 µs` por procura, `~50` nós).

**Kill-criterion (escrito antes do código):**
1. A unidade de custo é uma soma de contagens DETERMINÍSTICAS da procura (nada lê um relógio — o replay
   serve a mesma fila), com pesos MEDIDOS; e numa procura sem lama é EXACTAMENTE o número de nós de hoje
   (as cenas sem lama ficam com a mesma fila ao tique: os gates de hoje não mudam).
2. Real/modelo por grupo dentro de `0,75–1,25` em todos os grupos acima, re-medido a load `≤ 5`.
3. Na ponte, com lama, o pior tique depois da porta desce para perto do de sem lama mais UMA procura (o
   «sempre pelo menos um» fica — a procura cara sozinha é o item 4).
Se a soma não ficar em `0,75–1,25`, a unidade não entra e a fila continua em nós (recusa medida).

### §22.2 — O início de uma cena grande: a montagem a frio

**A medição que abre** (sonda provisória `sonda_frio_w14`, cronómetros provisórios por fase, a cena
grande `100 × 100 m`, `1 000` obstáculos, `49` mosaicos de `15 m`, o mínimo de 8, `--release`, load `~48`
— os tempos absolutos estão inflados, a partilha não):

| fase | sem lama | com `100` lamas | natureza |
|---|---|---|---|
| índice (que mosaico toca cada forma) | `0,06 ms` | `0,07` | série |
| recuo + corte canónico | `3,91` | `4,43` | por mosaico |
| união (Clipper) | `7,40` | `8,02` | por mosaico |
| diferença (Clipper) | `3,75` | `4,10` | por mosaico |
| anéis das áreas | — | `0,49` | por mosaico |
| pedaços (área a área, Clipper) | `0,03` | `19,00` | por mosaico |
| triangulação (`spade`) | `10,75` | `16,34` | por mosaico |
| fusão em convexos | `3,89` | `5,47` | por mosaico |
| a peça de cada mosaico | `0,68` | `0,90` | série |
| a montagem (`MalhaPorBlocos::monta`) | `2,17` | `2,65` | série |
| **total** | **`32,8`** | **`61,8`** | |

⇒ `~95 %` é trabalho POR MOSAICO, e os mosaicos são independentes por construção (cada um lê só as formas
que lhe tocam e escreve só o seu `Mosaico`; a costura é exacta pelo corte canónico, não pela ordem). O
piso das bibliotecas (§20.5: `spade` + Clipper) não se move; o que se move é o número de núcleos.

**A decisão (técnica, delegada):** os mosaicos a refazer constroem-se em PARALELO (`rayon`) e entram na
montagem pela mesma ordem de chave de antes. É uma exceção à regra «sem rayon» restrita a
`TiledMesh::update_with_areas` (ADR próprio, no molde dos cinco anteriores); na web o `rayon` cai sozinho
para uma thread (stack §11).

**Kill-criterion (escrito antes do código):**
1. A malha é a MESMA ao bit: a impressão digital da sonda (`IMPRESSAO=1`: 3 cenas × 3 raios × lamas
   sim/não × fusão sim/não × 5 mudanças incrementais — vértices, anéis, áreas, paredes, ilhas, área,
   falhas; `180` malhas) é `42f3d779b02e0329` antes e depois; os oráculos ao bit da `ph2d-navmesh` e o
   hash/replay da ponte não mudam.
2. A frio, a cena grande com `100` lamas desce a `≤ 16 ms` (um quadro a 60 Hz) a load `≤ 5`, na
   workstation; e uma porta (`1–4` mosaicos) não fica mais lenta que hoje (o paralelo só acorda com
   trabalho que o pague).
Se (2) não se cumprir, o paralelo não entra (recusa medida, com o número).

### §22.3 — O desvio a 1 000 agentes: a procura dos vizinhos

**A medição que abre** (`ph2d-orca/tests/it/custo.rs`, os três `#[ignore]`, `--release`, load `~85` — os
tempos estão inflados, a partilha não): a `1 000` agentes densos o tique é `3,2–4,0 ms`, e SÓ a vizinhança
`2,40 ms` (`~70 %`). A célula da grelha é o alcance SEM PERDA (`2r + 6sτ` = `12,6 m` a `2 m/s`), logo as
3 × 3 células à volta de cada agente apanham a multidão inteira (`~400` candidatos, cada um com a sua
distância) para ficar com os `10` mais perto.

**O desenho:** uma grelha FINA em listas contíguas sobre a caixa da fotografia, percorrida em ANÉIS
quadrados à volta da célula do agente; pára quando há pelo menos `n` candidatos ao alcance e a distância
ao 1.º anel por visitar é ESTRITAMENTE maior que a do `n`-ésimo (nenhum por visitar pode entrar, nem
empatar), ou quando o anel passa o maior alcance possível. Os candidatos, o alcance de cada par e a
ordem total (distância, índice) são os de hoje — a lista sai a MESMA. Só no modo do produto (os `n`
mais perto ao alcance sem perda); o da paridade com o Godot (`neighbor_dist`) fica a varrida.

**Kill-criterion (escrito antes do código):**
1. As listas de vizinhos são as MESMAS, elemento a elemento, que as da varrida de hoje (o oráculo fica
   verbatim nos testes) sobre multidões densas, esparsas, em grupos, com raios e velocidades mistos,
   empates de distância e `ignores`; o oráculo do Godot, o banco de cenários e
   `entalado_entre_duas_paredes_o_3d_nao_parte` passam sem mudança.
2. A `1 000` agentes densos a vizinhança desce de `2,4` para `≤ 0,6 ms` (load `≤ 5`), e a `100` não fica
   mais lenta. Senão a varrida fica (recusa medida).

### §22.4 — A procura ponderada com muita lama: ⛔ recusa MEDIDA

**Kill-criterion (escrito antes de qualquer código):** uma alavanca entra só se cortar o tempo da procura
ponderada na cena grande (`100` lamas, pesos `4` e `10`) a METADE ou menos, com o custo contra o oráculo
ao dígito de hoje (§17.1: `1,0000 · 1,0000 · 1,0045–1,0107` por peso, a `0,25 m`).

**As medições** (sonda provisória `sonda_steiner_w14`: a cena grande, `60` pares, o mínimo de 3, load
`~83` — só as razões valem; o custo contra o passo fino `0,1 m`):

| peso · passo da grelha | nós / consulta | trabalho / consulta | mediana | custo / ref. p95 · máx |
|---|---|---|---|---|
| 4 · **`0,25`** (o produto) | `25 692` | `48 135` | `4,0 ms` | `1,0001 · 1,0008` |
| 4 · `0,5` | `23 017` | `37 586` | `3,2` | `1,0001 · 1,0003` |
| 4 · `1,0` | `21 867` | `33 632` | `2,8` | `1,0002 · 1,0012` |
| 10 · **`0,25`** | `30 898` | `58 601` | `6,0` | `1,0001 · 1,0019` |
| 10 · `0,5` | `27 352` | `45 264` | `4,5` | `1,0007 · 1,0020` |
| 10 · `1,0` | `25 838` | `40 258` | `3,9` | `1,0019 · 1,0020` |

⇒ **a grelha das fronteiras NÃO é a fonte do excesso** (o «`4,4×` nós é a grelha» do §17.3 era a leitura
de uma cena): com `4×` menos raízes os nós descem só `15 %`. O que fica é a REGIÃO que uma procura exacta
tem de cobrir — o A\* expande todo o nó com `f < C*`, e a lama sobe o `C*` acima do comprimento (a elipse
`g + w_min·|x − t| < C*` alarga-se) e parte o chão em mais polígonos.

O perfil (`docs/Painter/ferramentas/amostra_gdb.py`, `1 331` amostras, peso 4, o produto): a dominância
`37,6 %` (o corte `25,4 %`), o heap `17,2 %`, a expansão e as raízes o resto. Mesmo a custo ZERO a
dominância dava `1,6×` — e sem ela os nós sobem `2,6×` (§17.1).

| recusado | medição |
|---|---|
| a grelha mais grossa (`0,5`–`1,0 m`) | `−20–35 %` de tempo na cena grande, mas a `0,5 m` a grelha escolhe o corredor errado nas cenas pequenas (`9 %` a peso 10, §15.1) — e não chega a metade |
| o custo do caminho uniforme como tecto (com folga) | o A\* nunca expande um nó acima de `C*`: o tecto só poupa INSERÇÕES (`gerados / expandidos ≈ 1,2`), e sem folga piorava a precisão (§15.4) |
| um heurístico mais forte que `w_min·|x − t|` (marcos ALT) | pede uma procura ponderada da malha inteira por marco a cada porta que muda e a cada tabela de custos — a malha muda em tempo de jogo |
| o A\* ponderado (heurístico inflacionado) | sub-óptimo por construção: parte o `1,0000` contra o oráculo, que é a régua da W7 |

O que a W14 já tirou a esta procura: a fila passa a contá-la pelo que custa (§22.1), e uma tabela sem lama
deixou de varrer a malha a cada procura.

### §22.5 — Um corpo LARGO que anda prende o agente

**A reprodução** (sonda `nav_desvio_largo::sonda_o_corpo_largo_que_anda`, `#[ignore]`: o agente de
`(−6, 0)` para `(6, 0)`, uma barreira cinemática `0,3 m` de espessura, a andar por `Transform`, 900
tiques):

| a barreira | resultado |
|---|---|
| de frente, `1,2 m`, a `0,3 m/s` | NÃO chega — fica colada à frente dela, em `y = 0,00`, e recua com ela |
| de frente, `3 m`, a `0,3 m/s` | igual, ao centímetro (os dois casos são o MESMO para o desvio) |
| de frente, `3 m`, a `1 m/s` | empurrado `6 m` para trás |
| à frente, `3 m`, no mesmo sentido, a `0,3 m/s` | chega no tique `210` (o CONTROLO parado: `200`) — não é armadilha |

O desvio vê a barreira como uma FILEIRA de discos de `0,21 m` (`discos()`, a cura da W6 contra o disco
único): os dois do meio estão simétricos em `y = ±0,15` e a fileira faz uma reentrância — a melhor
velocidade é travar, e o peso de lado (`SIDE_BIAS`) não tira ninguém de lá.

**O ORÁCULO CORRIDO** (`ferramentas/godot_nav_oraculo/largo.gd`, Godot 4.7.2, MIT, malha fechada, uma
linha de execução; o agente `R = 10 px`, `100 px/s`, a barreira `10 × 100 px` a `10 px/s`):

| a barreira no Godot | resultado |
|---|---|
| obstáculo de VÉRTICES que se muda de sítio, agente no eixo | preso (o empate simétrico — o Godot não tem peso de lado) |
| **obstáculo de VÉRTICES, agente `3 px` fora do eixo** | **contorna e chega (quadro `458`)** |
| obstáculo de vértices PARADO, no eixo | preso |
| um obstáculo de RAIO (o disco que a envolve) | preso, recua com ela |
| **dez discos ao longo dela (o que a ponte faz)** | **preso, mesmo fora do eixo** |

⇒ **a resposta da indústria é o POLÍGONO**: as arestas de um obstáculo são semi-planos por onde o agente
DESLIZA (o RVO2, o Godot); uma fileira de discos é uma serra. O peso de lado desfaz o empate do eixo.

**A decisão:** um corpo sólido que anda e não é agente (nem o alvo de quem o persegue — esse continua UM
disco, que o índice nomeia) entra no desvio como POLÍGONOS COM VELOCIDADE (`ph2d_orca::Movel`): as
linhas de cada polígono calculam-se no referencial dele (a velocidade relativa) e deslocam-se pela
velocidade do ponto mais perto do agente (a translação mais `ω ×` o braço — um torniquete roda). O disco
fica para a bola (é exacto). As linhas dos móveis são do agente, não das paredes: o 3D pode violá-las
(um corpo que empurra contra uma parede não torna o programa impossível).

**Kill-criterion (escrito antes do código):**
1. Os dois casos de frente (`1,2` e `3 m`) CHEGAM, sem tocar na barreira (folga `> 0`), em `≤ 2×` os
   tiques do CONTROLO parado; o caso «à frente» não piora.
2. Os gates do desvio passam sem mudar um número: o banco de cenários, o oráculo do Godot, `nav_desvio*`
   (a porta comprida, o torniquete, o corpo composto e o golpe), o scrub e o hash/replay.
3. Senão, a fileira de discos fica e o achado vira recusa medida.

**O que mudou e o que mediu** (`nav_desvio_largo::um_corpo_largo_que_vem_de_frente_e_contornado`):

| a barreira | antes (a fileira de discos) | **depois (polígono com velocidade + folga `|u|·τ`)** |
|---|---|---|
| de frente, `1,2 m`, `0,3 m/s` | nunca chega | **tique `213`**, folga `0,30 m` |
| de frente, `3 m`, `0,3 m/s` | nunca chega | **tique `578`**, folga `0,30 m` |
| de frente, `3 m`, `1 m/s` | empurrado `6 m` para trás | **tique `311`**, folga `1,00 m` |
| à frente, no mesmo sentido | `210` | `216` |
| CONTROLO parado (é parede da malha) | `200` | `199` |

- **A FOLGA** (`|u|·τ`, o que o corpo anda num horizonte): o ORCA desliza RENTE a uma parede — contra a da
  malha é o caminho mais curto, contra um corpo que anda é um roçar. Medido sem ela: `5 mm` ao braço do
  carrinho (o gate da W13 pede `> 0,2 m`); uma folga FIXA atrasava o CONTROLO parado (`199 → 322–335`
  tiques: lutava com o caminho rente à quina recortada); a da velocidade é zero parado.
- **O Godot tratado como parado** (o polígono sem a velocidade dele, à maneira do servidor do Godot) foi
  medido e é PIOR aqui: `705` tiques, e o caso rápido volta a falhar.
- **O oráculo, desfecho a desfecho** (`ph2d-orca/tests/it/oraculo_do_godot_largo.rs`, a cena do
  `largo.gd`): o polígono fora do eixo chega no quadro **`461`** (o Godot: **`458`**); a fileira de discos
  prende nos dois; no eixo exacto o Godot prende-se e o peso de lado tira-nos de lá (`456`).

⚠️ **O kill-criterion, contra o que mediu:** chegar e não tocar — cumprido nos três; «`≤ 2×` o CONTROLO»
— cumprido a `1,2 m` (`1,07×`) e a `1 m/s` (`1,56×`), **NÃO a `3 m` lenta (`2,9×`)**. A alternativa que o
critério previa (a fileira de discos) não chega em NENHUM — o polígono entra porque domina em todos os
casos medidos, e o resto fica aberto com o mecanismo (§22.6). Escrevi o critério contra «nada muda» e o
«nada» era a armadilha.

### §22.6 — ⏳ O que fica

- **A barreira larga e LENTA** (`3 m` a `0,3 m/s`): o agente contorna a `2,9×` o tempo do CONTROLO. O
  mecanismo, medido no traço: o caminho aponta para o alvo, no EIXO, através da barreira (ela anda, logo
  não é parede da malha), e puxa o agente de volta contra o peso de lado — equilibram-se quando o ângulo
  até ao alvo passa `asin(SIDE_BIAS) ≈ 14,5°`, e ele rasteja em `y ≈ −1,3` até a barreira o deixar
  passar. A cura é do CAMINHO (contornar o polígono pela tangente do lado escolhido), não do desvio.
- A dominância da procura ponderada (§22.4): o maior pedaço que sobra; nenhuma alavanca medida chega a
  metade.

### §22.7 — A prova

Gates novos: `a_ultima_procura_guarda_o_trabalho_e_nao_os_nos` · `o_trabalho_sem_lama_sao_os_nos_e_na_lama_pesa_o_que_custa`
(`16 837` de trabalho sobre `9 290` nós, `1,81×`) · `na_lama_a_vez_de_quem_nasce_conta_o_trabalho` (+ CONTROLO) ·
`os_mosaicos_feitos_em_paralelo_sao_os_de_uma_thread_ao_bit` (1 thread contra 8, quatro passos que refazem vários
mosaicos) · `os_vizinhos_por_aneis_sao_os_da_varrida` (o oráculo verbatim; `10 404` listas, `9 472` cortadas pelo
tecto, `2 089` empates na fronteira) · `o_corpo_largo_em_poligono_contorna_e_em_discos_prende_como_no_godot` ·
`um_corpo_que_anda_e_o_mesmo_parado_no_referencial_dele` (`785` de `2 000` cortados) ·
`um_corpo_largo_que_vem_de_frente_e_contornado` (+ CONTROLO sem desvio) ·
`uma_capsula_que_anda_e_um_torniquete_que_roda_sao_contornados_pela_forma` (a folga à forma EXACTA).

Mutação **17 / 17** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w14_2026-10-04.py`](ferramentas/mutacao_navegacao_w14_2026-10-04.py): o motor da W10/W13 com
UMA mudança declarada — cada grupo corre só os seus observadores). A 1.ª corrida deu **16 / 17**: a L2 (o agente
com a velocidade ABSOLUTA, não a relativa ao corpo) sobrevivia — chegar e a folga não mudam; o gate da
invariância de referencial nasceu disso. E antes da prova, a cápsula (o octógono) e a rotação (`ω ×` o braço) não
tinham régua nenhuma — os dois gates nasceram antes de a mutação os acusar.

### §22.8 — A re-medida CALMA do §22.1 (load `1,2–3,5`, no fecho)

**Os pesos do trabalho** (a mesma sonda, `2 880` consultas): o ajuste livre dá nó `129 ns` · raiz materializada
`464` · frente `13` (pesos `3,6` e `0,10` nós; o produto usa `37/8 = 4,6` e `1/8`). Com os pesos do PRODUTO, o
tempo por unidade de trabalho é `85–123 ns` em todos os grupos (`~114` de média ⇒ o orçamento de `20 000` vale
`~2,3 ms`); com os do ajuste calmo, real/modelo `0,71–1,07`, e com os do produto `0,74–1,07` — ficam os do
produto. Contar NÓS dava `85–321 ns` por nó. ⚠️ O canto que fica abaixo de `0,75` (`0,74`) é a procura UNIFORME
curta de `30` lamas: o trabalho sobrestima-a um pouco (é o lado seguro).

**O critério 3, na ponte** (`medir_replaneio`, orçamento `20 000`, o pior tique da janela depois da porta,
mediana de seis; 10 · 50 · 200 agentes): sem lama `3,1 · 3,9 · 6,8 ms`; com `150` lamas a peso 4 `38,7 · 46,9 ·
88,0 ms`; o CONTROLO com lama (a porta parada) `0,9 · 32,9 · 61,1 ms` com `1–7` procuras no tique. ⛔ **NÃO
cumprido**, e a medição separa os dois mecanismos — nenhum deles é a unidade da fila:
1. UMA procura nesta lama (`150` caixas de `2–8 m`, `~40 %` do chão) custa `~10 ms` ou mais, e a fila não parte
   uma procura ao meio — é o §22.4 (recusa medida).
2. Os replaneios que NÃO passam pela fila (o alvo que se mexeu, o corredor perdido, o preso): o CONTROLO, sem
   porta nenhuma, faz `1–7` procuras num tique. ⏳ Pô-los também no orçamento (quem já tem caminho continua a
   andá-lo enquanto espera, como na fila) é o aberto que fica — muda o momento de cada replaneio em todos os
   gates da condução, uma wave própria.

**Os itens 2 e 3, re-medidos calmos** (load `0,8`): a montagem a frio da cena grande `5,2 ms` sem lama e
**`7,5 ms`** com `100` (critério `≤ 16` ✓); uma porta `1,06` · `1,69 ms`; uma lama a mexer `2,24`. O desvio a
`1 000` agentes **`0,659 ms`** por tique, a vizinhança **`0,477 ms`** (critério `≤ 0,6` ✓); a `100`, `0,057` (era
`0,067` na W5 ✓).

## §23 — W15 (2026-10-05): o tique depois da porta, com lama — toda procura paga do orçamento, e a procura em FATIAS

O aberto do §22.8, nos dois mecanismos que a medição separou, e um defeito de determinismo da mesma família
achado ao desenhar a cura. Um bloco só (CLAUDE.md §0.10): os três decidem QUANDO e SOBRE QUE MALHA corre uma
procura.

### §23.1 — O que se mede e o que se achou

- **A medição que abre** é a do §22.8 (calma, load `1,2–3,5`): com `150` lamas a peso 4, o pior tique depois
  da porta `38,7 · 46,9 · 88,0 ms` (10 · 50 · 200 agentes) contra `3,1 · 3,9 · 6,8` sem lama; o CONTROLO com
  lama e sem porta `0,9 · 32,9 · 61,1 ms`, com `1–7` procuras num tique. Re-mede-se no fecho com o binário de
  antes e o de depois ALTERNADOS (`medir_replaneio`, que ganha a régua `falta` — o que falta andar em média a
  cada agente no fim da corrida —, a régua de que a cura não os atrasa).
- **⛔ Defeito achado (pré-existente, W9):** a fila decide «a malha mudou» pelo que o `TiledMesh` diz da última
  actualização — e depois de um scrub as malhas são as do FIM da corrida, não as do âncora. Uma porta que muda
  DEPOIS do âncora faz o replay ver uma mudança que a corrida não viu: a sonda
  `sonda_scrub_com_a_porta_noutro_sitio` (a porta abre no tique 25, scrub para o 13 a partir do 60) devolve
  `(procuras, dívida)` `[(1,4)×5, (2,0)×3]` contra `[(1,0)×8]` da corrida. O gate da W9 não o via: a porta
  dele muda ANTES do âncora.

### §23.2 — O desenho

1. **«Mudou» é o CONTEÚDO contra o tique anterior DESTA corrida:** a assinatura das entradas de cada malha
   (`TiledMesh::assinatura`) fica na ponte e entra no anel com a memória dos agentes; uma malha mudou quando a
   assinatura de agora difere da guardada. Depois de um scrub, a zona do que mudou (`changed_area`) é relativa
   a outra malha — essa chave percorre o caminho inteiro (`onde = None`).
2. **Toda procura paga do orçamento do tique** — a da fila, a de quem nasce E a de quem replaneia por motivo
   próprio (o alvo andou, saiu do corredor, preso). Quem não tem vez entra na fila (`owed`) e continua a andar o
   caminho que tem; quem não tem caminho espera parado.
3. **A procura em FATIAS** (a do Detour, `dtNavMeshQuery::updateSlicedFindPath`): uma procura que não cabe no
   que resta do tique PÁRA entre dois `pop` do heap e continua no tique seguinte, com prioridade na fila. O
   estado inteiro (heap, nós, raízes) fica na ponte, FORA do anel; no anel vai só a descrição
   (`AgentRuntime::a_meio`: de onde, para onde, o trabalho já feito e a assinatura das entradas). Depois de um
   scrub, a procura REFAZ-SE até ao mesmo trabalho — a mesma sequência de `pop`s, porque o trabalho cresce
   estritamente a cada `pop` e a pausa só acontece depois de um. Entradas que mudam (a malha, os custos, os
   atalhos) recomeçam-na; um recomeço deixa a seguinte correr inteira (vivacidade numa malha que nunca pára).
   Sem tecto (`u64::MAX`) a procura é a de hoje, ao bit.

### §23.3 — O kill-criterion (escrito ANTES do código)

1. **Ao bit onde cabe:** com o orçamento que hoje não se esgota (todas as cenas pequenas dos gates), a condução
   é a mesma — os gates da navegação passam sem mudar um número; os oráculos da `ph2d-nav` passam (a procura
   síncrona é a mesma).
2. **Determinismo:** um scrub para o MEIO de uma procura em fatias devolve a mesma corrida ao bit (gate), e a
   sonda do §23.1 passa a gate e fica verde.
3. **O tique:** com `150` lamas, o pior tique depois da porta fica `≤ 2×` o de sem lama em cada `N`, e o CONTROLO
   com lama `≤ 2×` o de sem lama (`--release`, load `≤ 5`, antes e depois alternados).
4. **Não atrasa:** a `falta` média com lama não sobe mais de `10 %` em nenhum `N`.
Se (3) não se cumprir, as fatias não entram (recusa medida); (2) é inegociável.

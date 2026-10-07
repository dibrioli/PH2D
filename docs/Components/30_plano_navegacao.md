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

### §23.4 — A 1.ª medição da cura, e o que ela mandou acrescentar (antes do código seguinte)

A régua é a da regra do dono de 05/10: as duas versões no MESMO processo (`set_nav_slices`: A = toda procura
inteira no tique, B = a vez e as fatias), 7 rodadas intercaladas com a ordem rodada, o MÍNIMO (a mediana ao lado),
uma compilação no perfil `smoke`, o loadavg anotado (`2,8–5,1`). `150` lamas a peso 4, orçamento `20 000`:

| agentes | A: pior tique depois da porta | **B** | A: CONTROLO sem porta | **B** | A: trabalho num tique | **B** |
|---|---|---|---|---|---|---|
| 10 | `45,3 ms` | **`6,5`** | `1,1` | `4,3` | `327 693` | **`20 013`** |
| 50 | `57,9` | **`7,8`** | `37,7` | **`4,7`** | `651 366` | **`20 010`** |
| 200 | `50,3` | **`7,4`** | `70,1` | **`5,5`** | `967 600` | **`20 012`** |

Dois defeitos achados pela régua no caminho: (1) quem tinha uma procura a meio ficava com a estimativa de quem
nunca procurou, e os que ainda não tinham começado passavam-lhe à frente e começavam outras — as procuras a meio
acumulavam-se e a porta seguinte recomeçava-as todas, cada uma INTEIRA ao 1.º recomeço (`47` procuras e `5,1 M`
de trabalho num tique). Cura: quem tem uma procura a meio vai primeiro na fila e leva o que sobra, e só corre
inteira ao 3.º recomeço SEGUIDO. (2) A régua `falta` contava `0` a quem ainda espera o 1.º caminho.

⛔ **O que fica por cumprir (o critério 4):** na cena de stress (`~40 %` do chão em lama, cada procura `~65 000`
de trabalho, `~7 ms`), o orçamento é a VAZÃO da procura — a `200` agentes, ao fim de `~8 s`, `157` ainda esperam o
1.º caminho (orçamento `40 000`: `107`; `80 000`: `14`, com o tique a `15 ms`). Antes, o «sempre pelo menos um»
pagava uma procura inteira por tique (o pico). Subir o orçamento troca um pelo outro.

**A alavanca é o hardware:** as procuras a meio são independentes (cada uma com os buffers dela) — correm em
PARALELO, cada uma com uma fatia de um orçamento; quem decide a fatia de cada uma é a fila, pela ordem, antes de as
correr; o resultado entra na condução pela ordem das entidades. ⇒ o resultado NÃO depende do número de núcleos (o
`rayon` com uma thread dá o mesmo, ao bit), só o relógio. `PROCURAS_EM_PARALELO` (as procuras a meio que avançam
juntas num tique) mede-se; na web o `rayon` cai para uma thread e o tique paga-as em série.

**Kill-criterion (escrito antes do código):**
5. O mesmo resultado com `1` e com `8` threads (gate: posições e memória dos agentes ao bit).
6. Na cena de stress a `200` agentes, os que esperam o 1.º caminho no fim da sonda caem a `≤ 10 %`, com o pior tique
   depois da porta `≤ 2×` o de sem lama (load anotado). Senão o paralelo não entra e o orçamento fica a vazão (recusa
   medida, com a tabela).

### §23.5 — O que mudou, o que se mediu, e o que a medição corrigiu no caminho

**O desenho final** (o §23.2 e o §23.4, com as correcções que a régua mandou):

1. **«Mudou» é o conteúdo** (`TiledMesh::assinatura`: os mosaicos e a assinatura de cada um — a assinatura das
   ENTRADAS mudava com um obstáculo fora da região) contra a do tique anterior desta corrida, que vai no anel
   (`ControllerMemory::nav_sinais`); depois de um scrub, uma malha cuja actualização partiu de outro conteúdo
   percorre o caminho inteiro (`sem_zona`).
2. **A procura abre-se no PEDIDO** (`Plano::begin`; as que não custam nada acabam logo) e paga do orçamento; sem
   a vez, a fila deve-lha e o agente anda o que tem.
3. **A pausa** acontece depois do `pop` em que o trabalho PASSOU o tecto (um tecto `W` paga uma procura de `W`; o
   scrub refaz com `W − 1`).
4. **O passo em paralelo** (`PROCURAS_EM_PARALELO = 16`, `rayon`, ADR-0180): antes da condução, as procuras a
   meio avançam juntas, a MAIS ADIANTADA primeiro, cada uma com uma fatia do orçamento; a fila e a condução, em
   série, ficam com o que a maior fatia não gastou — **o caminho crítico do tique é UM orçamento**.
5. **Uma procura a meio de outras entradas recomeça**; cada recomeço SEGUIDO com trabalho feito DOBRA a fatia da
   seguinte (`fatia_depois_de`: vivacidade numa malha que nunca pára, sem pico de uma vez).

**A medição final** (`medir_replaneio`, a régua do dono de 05/10: as versões no mesmo processo, 7 rodadas
intercaladas com a ordem rodada, o MÍNIMO e a mediana; perfil `smoke`; ⚠️ load `22–37` — a máquina estava cheia de
outras linhas, e as rodadas intercaladas são o que torna a comparação válida assim). A = toda procura inteira no
tique (`set_nav_slices(false)`); **B = o produto** (16 em paralelo):

| lamas | agentes | depois da porta A → **B** | CONTROLO sem porta A → **B** | trabalho crítico num tique A → **B** | `falta` A → B | sem caminho no fim (B) |
|---|---|---|---|---|---|---|
| 0 | 10 · 50 · 200 | `6,7 · 9,3 · 9,2` → **`5,4 · 6,0 · 8,2`** | `0,8 · 1,7 · 6,1` → `0,7 · 1,8 · 5,7` | `35 · 34 · 50 mil` → **`24 · 28 · 28 mil`** | `+0,1 · +0,3 · +0,8 %` | `0` |
| 150 | 10 | `48,0` → **`5,9`** | `1,2` → `6,4` | `328 mil` → **`40 mil`** | `+0,6 %` | `0` |
| 150 | 50 | `58,7` → **`7,5`** | `37,0` → **`6,0`** | `651 mil` → **`40 mil`** | `+1,2 %` | `0` |
| 150 | 200 | `65,9` → **`12,9`** | `88,4` → **`11,1`** | `968 mil` → **`40 mil`** | `+2,2 %` | `0` (em série: `189`) |

**Contra o kill-criterion:** (1) ✓ com a ressalva de que os gates da LEI DA FILA mudaram de observável (a
procura abre-se no pedido: «quem foi servido» é quem TEM o caminho) e correm numa faixa (`set_nav_parallel(0)`);
o «sempre pelo menos um» da W9 saiu (quem nasce com a fila cheia espera um tique). (2) ✓ (gates abaixo). (3) depois
da porta ✓ (`1,1 · 1,25 · 1,6×` o de sem lama); ⛔ o CONTROLO com lama a `10` agentes fica em `6,4 ms` contra `0,7`
sem lama — o critério comparava coisas diferentes: A faz todo o trabalho no tique da porta (`48 ms`) e fica vazio
depois, B reparte-o pelos tiques seguintes a um orçamento cada; o que se cumpre é «nenhum tique paga mais que um
orçamento (dois, num recomeço)». (4) ✓ (`≤ +2,2 %`). (5) ✓. (6) ✓ (`0` à espera a `200`). Com `8` em paralelo também `0` na medição final (`62–63` antes
da ordem pela mais adiantada), mas o pior tique a `50` agentes fica em `11,1 ms` (`7,5` com `16`) e o crítico chega a
`80 000` — ficam `16`.

**O que a régua corrigiu no caminho** (cada um foi um defeito do desenho, medido, e não uma afinação):

| achado | medido | cura |
|---|---|---|
| quem tem uma procura a meio ficava com a estimativa de quem nunca procurou; os que não tinham começado passavam-lhe à frente | `47` procuras e `5,1 M` de trabalho num tique (a porta recomeçava todas, e cada uma corria inteira ao 1.º recomeço) | a meio, primeiro; depois, o passo em paralelo |
| em série, o orçamento é a VAZÃO na cena de stress | a `200` agentes, `157` sem caminho ao fim de `8 s` (orçamento `40 000`: `107`; `80 000`: `14`, o tique a `15 ms`) | o passo em paralelo (`16`: `0`) |
| o paralelo e a série somavam-se | o caminho crítico em DOIS orçamentos (`9,6 ms` contra `4,6` sem lama a `10`) | a série fica com o que a maior fatia não gastou |
| a fila revezava as procuras a meio pela espera | nenhuma acabava antes de a porta seguinte as recomeçar; ao 3.º recomeço, inteira (`150 000` num tique) | a mais ADIANTADA primeiro |
| «corre inteira ao 3.º recomeço» | na cena de stress há procuras de `200–360 000` (`25–40 ms`), mais longas que o intervalo da porta da sonda | dobrar a fatia por recomeço (o crítico: `40 000` no pior) |
| um recomeço sem trabalho feito contava | dobrava a fatia de quem não perdeu nada | só conta o que tinha trabalho |

### §23.6 — ⛔ Recusas MEDIDAS

| recusado | porquê |
|---|---|
| subir o orçamento em vez do paralelo | `80 000`: o tique a `15 ms` e ainda `14` à espera a `200` agentes (§23.4) |
| guardar o estado da procura a meio no anel | megabytes por âncora (`256` âncoras); o anel leva a descrição e o scrub refá-la |
| continuar a procura numa malha nova (sem recomeçar) | o estado dela fala dos polígonos da malha velha; a numeração muda quando QUALQUER mosaico muda |
| guardar a malha velha (`Arc`) para a procura acabar nela | o anel reteria uma versão da malha por âncora com procura a meio (`~3 MB` cada; até `~80` versões na sonda) |
| `PROCURAS_EM_PARALELO` pelo número de núcleos da máquina | o resultado mudaria com a máquina (o hash do CI compara três) |

### §23.7 — A prova

Gates novos: `a_procura_em_fatias_e_a_procura_inteira_ao_bit` (`ph2d-navmesh`: fatias de `1 · 7 · 61`, com e sem
atalhos, a pesos 4 e 10, e refeita do zero até cada pausa — `140` de `144` procuras pararam a meio, `132` pausas
refeitas, `9` caminhos por um atalho) · `nav_fatias` (8): `uma_procura_que_nao_cabe_para_a_meio_e_acaba_no_mesmo_caminho`
(+ CONTROLO sem tecto) · `um_scrub_a_meio_de_uma_procura_em_fatias_devolve_a_mesma_corrida` (também com a procura
a meio NO MOMENTO do scrub) · `quem_segue_um_alvo_que_salta_espera_pela_vez` · `o_passo_em_paralelo_da_o_mesmo_com_uma_thread_e_com_oito`
· `uma_malha_que_nunca_para_nao_deixa_a_procura_sem_acabar` · `com_poucas_faixas_a_procura_mais_adiantada_acaba_primeiro`
· `uma_procura_que_nao_comecou_nao_conta_como_recomeco` · `uma_procura_a_meio_numa_malha_que_mudou_recomeca` ·
`um_scrub_numa_corrida_com_a_porta_a_alternar_devolve_a_mesma_corrida` (`nav_mundo`: a fixtura CONTÉM o fenómeno — o
fim com a porta fechada, cada âncora com ela aberta).

Mutação **18 / 19** ([`mutacao_navegacao_w15_2026-10-05.py`](ferramentas/mutacao_navegacao_w15_2026-10-05.py), o
motor da W14). A 1.ª corrida deu `13 / 22`: quatro mutações eram de código MORTO (a ordem das procuras a meio na fila
da série só valia sem o passo em paralelo, que não é um modo do produto — cortado); `F3` escapava porque a
referência «inteira» passa pelo mesmo código (observador novo: o gate da dominância); `S1/S2` escapavam porque o gate
do scrub tinha a fixtura VAZIA (o orçamento `1` deixava os guardas sem caminho no tique do replay — substituído);
`V6`, `P2` e a ordem do paralelo não tinham régua (três gates novos); `S3` perdeu a régua com o gate substituído e
ganhou-a no scrub com a procura a meio. ⏳ **`S5` sobrevive:** sem a guarda `sem_zona`, a zona do que mudou depois de
um scrub é a relativa à malha do FIM da corrida; a guarda percorre o caminho inteiro, o que dá a mesma resposta que a
verificação parcial da corrida (todo troço fora da zona que mudou anda-se, por construção) — uma fixtura que a
separe pede duas portas com mudanças em zonas diferentes entre o âncora, o agora e o fim.

### §23.8 — ⏳ O que fica

- `S5` (acima).
- A barreira larga e lenta (§22.6) e a dominância da ponderada (§22.4), como estavam.
- Com a fila cheia, as procuras novas de quem replaneia por motivo próprio esperam um tique pela vez — um
  perseguidor numa cena de stress segue o alvo com um tique de atraso a mais.

## §24 — Report do dono (05/10): dois inimigos iguais presos no mesmo portal

**O sintoma** (cena `=4`): *«dois inimigos configurados igual, ao tentarem usar o teletransporte, se impedem
e ficam parados — não tem a inteligência de ceder ou empurrar»*.

**A reprodução, pela cena real** (`nav_smoke::montar(…, 4)`, o cinzento com *Avoid Harm* ligado e a nascer em
`20` sítios à esquerda): `5` presos PARA SEMPRE, os dois encostados (`0,70 m` = os dois raios) um de cada lado do
portal, a `0,31` e a `0,47 m` dele. Igual com as fatias da W15 desligadas — o defeito é da W5 + W7.

**O mecanismo:** o teletransporte disparava quando o CENTRO do agente chegava ao ponto da entrada (a
`velocidade · dt`, `~4 cm`); para lá chegar, cada um teria de se sobrepor ao outro, e o desvio — simétrico —
não deixa nenhum. Nenhum gate tinha DOIS agentes no mesmo atalho (os da W7 e os da cena têm um).

**A cura** (`ph2d_nav::agent::alcance_de_atalho`): a entrada de um atalho alcança-se quando fica DENTRO do corpo
(a um raio); `AgentConfig::radius` (a ponte dá-lhe o raio do corpo). Um canto continua no passo do executor
(Q5). Medido sobre `135` nascimentos (`--release`):

| alcance | presos |
|---|---|
| o ponto (antes) | `44` |
| `0,5` raio | `1` |
| `0,75` raio | `0` |
| **`1` raio** | **`0`** |
| `2` raios | `0` |

Fica UM: o limiar com folga, e o que a palavra diz (o corpo está em cima do portal). ⛔ Recusado: dois raios —
o argumento geométrico que o sustentava não se aguentava, e a medição não o distingue de um.

**A prova:** gate `dois_inimigos_iguais_pelo_mesmo_portal_nao_se_prendem` (`nav_smoke_lava_tests.rs`: `40`
nascimentos, a fixtura exige os DOIS pelo portal em `≥ 8`), vermelho antes da cura; mutações à mão — o alcance no
ponto, a meio raio (o nascimento `(-2, -1,5)` prende), e a ponte sem o raio — sangram as três.

### §24.1 — O 2.º report (05/10): quem sai do portal em cima do herói prende-o

*«Se fizerem o teletransporte e caírem em cima do player, travam o player.»* **Medido:** o salto punha o
corpo no ponto da saída sem perguntar quem lá estava — com o herói parado na saída, o cinzento aterrava a
`2 cm` do centro dele (dois corpos sobrepostos). No arnês sem janela o herói ainda se soltava com as setas;
no app prendia — e sobrepor corpos é o defeito, seja qual for o efeito do mover.

**A cura** (a dos portais da indústria): quem chega à entrada só salta com a saída LIVRE para o corpo dele
(`ph2d_nav::agent::Vez::saida_livre`, respondida pela ponte, `nav_desvio::saida_livre`: nenhum corpo que não é
parede nem sensor a menos do raio, pela forma de cada um — o disco, ou o polígono do desvio); senão ESPERA em
cima da entrada, e esperar não é estar preso (o relógio do «preso» não anda). Gate
`ninguem_sai_do_portal_dentro_de_outro_corpo` (vermelho antes; o herói parado na saída `3 s`, os dois esperam
`≥ 30` tiques, ninguém diz «preso», e os dois passam quando ela fica livre) e
`a_distancia_a_uma_caixa_e_zero_dentro_e_a_do_canto_fora`. Mutações à mão (a lei sem a pergunta, a ponte sem
ela, o disco que não conta, o «dentro» ao contrário) sangram as quatro; o zerar do relógio do «preso» à espera
era redundante (sobreviveu) e saiu.

### §24.2 — O 2.º report (05/10): *«alguns morcegos encostam na lava e morrem»* (a arena)

**A investigação** (o arnês `Jogo` da arena, pelo laço inteiro; o dono: o herói estava DENTRO da lava, e os
morcegos não tinham levado tiro): em seis maneiras de jogar e nove sítios dentro da lava, **zero** morcegos
queimados — a folga do corpo deles à lava ficava em `0,7–2,2 mm` (passam rente, como todo caminho mais curto).
Duas causas medidas, nenhuma era a lava a matar:

1. **O desenho maior que o corpo:** o morcego desenhava-se como um QUADRADO de `0,45 m` com um corpo BOLA inscrito
   nele (raio `0,225`); os cantos saíam `41 %` para fora do corpo e entravam até `6 cm` na lava (`58–99`
   tiques-morcego em `40 s`) — *«encostam»*.
2. **O sumiço:** os que «morriam» (`6–12` por corrida) sumiam todos no RECOMEÇO, quando o herói morria na lava.

**A cura:** o corpo do morcego é o quadrado desenhado (`Cuboid`); a navegação dá-lhe o raio que o envolve
(`0,32 m`) e o desenho deixa de poder entrar na lava. ⚠️ **E ela destapou um 3.º defeito:** o gate
`um_tiro_mata_um_morcego` sangrou — a bala atravessava o morcego. Isolado (uma bala contra bola e caixa, com e
sem mover: as quatro acertam) e confirmado por ablação: o DESVIO tratava a bala como um corpo sólido que anda, e
o morcego **esquivava-se dos tiros** (com o corpo maior, sempre). Um projéctil não é um obstáculo a contornar, é
um golpe: os projécteis saem do desvio (`nav_desvio`, `projectile_state`).

**A prova:** gate `o_morcego_desenhado_nunca_entra_na_lava` (o herói parado dentro da lava junto à borda, `30 s`;
a fixtura exige morcegos rente a ela); `um_tiro_mata_um_morcego` como estava. Mutações à mão: o corpo de volta à
bola sangra o 1.º, as balas de volta ao desvio sangram o 2.º.

## §25 — W16 (2026-10-05): o aberto da W15 num ciclo só (CLAUDE.md §0.10)

Os seis itens do briefing de 05/10, um plano. ⚠️ **A ordem, dita sem enfeite:** os critérios de A, B e E são os do
briefing, sem mudança; o texto desta secção foi escrito DEPOIS do código de A, B e E (que se fez com eles à frente) e
ANTES de a 2.ª rodada de medição decidir C2 e D. **Série declarada:** C2 e D dependem da 1.ª rodada (ela mediu que
nenhum `PROCURAS_EM_PARALELO` menor serve — a cura de D tinha de ser outra), por isso há DUAS rodadas; A, B e E são
independentes e foram em paralelo.

### §25.1 — Os kill-criteria

| item | entra se | senão |
|---|---|---|
| **A** o desenho maior que o corpo | um CENSO de todas as cenas da família (`FAMILY.routers × 1..=max_level`) vermelho antes e verde depois, com população afirmada (cenas = a soma dos níveis; corpos ≥ o piso medido) e um CONTROLO; nenhum gate de trajecto muda | — |
| **B** a barreira larga e lenta | os três casos de `nav_desvio_largo` chegam em `≤ 2×` o CONTROLO parado, sem tocar (folga `> 0,05`) nem recuar; «à frente» não piora; o resto da crate sem mudar um número | a fileira de tangentes fica recusa medida |
| **C1** o tique com lama | o trabalho TOTAL por porta de B `≤` o da A (a vez não desperdiça) e o CONTROLO de B `≤` o da A na mesma cena ⇒ é a repartição, sem código | procurar o desperdício |
| **C2** o perseguidor com a fila cheia | o atraso (tiques seguidos com a procura pedida e por servir: máx · média) na cena de stress `≤` o da A, sem piorar as outras colunas mais de `10 %` | recusa medida |
| **D** a web (uma thread) | numa thread, o pior tique depois da porta `≤ 2×` o de todos os núcleos, com `≤ 10 %` sem caminho, a `falta` `≤ +10 %` e o atraso de C2 sem piorar — e o resultado igual com 1 e com N threads (o gate da W15) | o custo numa thread fica medido e escrito (o ADR-0180) |
| **E** a mutação `S5` | uma fixtura que a separa (vermelha sem a guarda `sem_zona`) | provar a guarda redundante e cortá-la |
| **F** higiene | `line-Components2` conferida; o ADR-0180 recontado no fecho | — |

### §25.2 — A 1.ª rodada (`target/prova/w16/medir_replaneio_rodada1.txt`)

A régua do dono de 05/10: as versões no MESMO processo (`set_nav_slices`, `set_nav_parallel`, e um pool `rayon` de
UMA thread ao lado do de todos — a web), 7 rodadas intercaladas com a ordem rodada, o mínimo (a mediana como
controlo), perfil `smoke`, load `4,0–8,0`. A sonda ganhou `10` perseguidores de uma PRESA que anda em círculo (a
régua de C2) e o trabalho total por porta (a de C1). `150` lamas, `200` agentes:

| versão | pior tique depois da porta | CONTROLO | trabalho por porta | atraso do perseguidor (máx · média) | sem caminho |
|---|---|---|---|---|---|
| A inteira | `47,6 ms` | `62,5` | `7,66 M` | `15 · 6,6` | `0` |
| B W15 (16 em paralelo) | `11,3` | `8,0` | `4,07 M` | **`52 · 8,7`** | `0` |
| B W15 · uma thread | **`40,8`** | `26,4` | igual | igual | `0` |
| B 8 em paralelo · uma thread | `29,2` | `29,9` | `4,48 M` | **`149 · 54,5`** | `0` |
| B 4 em paralelo | `12,4` | `6,9` | `2,25 M` | — | **`57`** |
| B 2 em paralelo | `13,0` | `5,9` | `1,15 M` | — | **`138`** |

Sem lama, a `200` agentes, o atraso do perseguidor é `15 · 4,7` (A) contra **`17 · 15,3`** (B) — o «um tique de
atraso a mais» do §23.8 era muito pior.

- **C1 fecha pela medição, sem código:** B gasta MENOS trabalho por porta que A em toda a cena (`0,92 M` contra
  `1,66 M` a `10` agentes; `4,07` contra `7,66` a `200`), e o CONTROLO de B fica abaixo do de A (`4,5 · 4,9 · 8,0 ms`
  contra `6,6 · 32,9 · 62,5` a `10 · 50 · 200`). O `6,4 ms` contra `0,7` da W15 comparava B COM lama com a cena SEM
  lama: é o trabalho da porta repartido pelos tiques seguintes, e é menos que o da A.
- **D:** o resultado é o mesmo com 1 e com 16 threads (as colunas de trabalho e de atraso são iguais ao bit), mas numa
  thread o passo paga as fatias todas (`40,8 ms`). Nenhum `PROCURAS_EM_PARALELO` menor serve: `8` atrasa quem persegue
  `149` tiques, `4` deixa `57` sem caminho. ⇒ a cura tem de limitar o TRABALHO do passo, não o número de procuras.
- **C2:** quem persegue espera porque a procura dele entra atrás de todas as outras (a mais adiantada primeiro).

### §25.3 — Os candidatos da 2.ª rodada (desenho, antes da medição que os decide)

- **C2 — o alvo À VISTA** (`ph2d_nav::agent::a_vista`): se a recta até ao alvo se anda dentro da malha com o custo do
  comprimento, nenhum custo da tabela é menor que `1` e não há atalhos, a recta É o caminho mais curto (todo caminho
  mede pelo menos a recta e custa pelo menos o que mede) — instala-se sem procura e sem a vez, e a procura a meio
  larga-se. É o que a procura devolveria; só o tique em que chega muda.
- **D — o tecto do trabalho TOTAL do passo em paralelo** (`Sonda::teto_paralelo`, em orçamentos): a fatia de cada
  procura a meio é `min(orçamento, tecto / quantas)`. Quem avança e quanto continua decidido antes de correr — o
  resultado não depende das threads; numa thread o passo paga no máximo o tecto. Medem-se `8` e `4` orçamentos.

### §25.4 — A 2.ª, a 3.ª e a 4.ª rodadas (`…rodada{2,3,4}.txt`) — em série, e porquê

- **A 2.ª** mediu os dois candidatos. O alvo à vista deu as colunas IGUAIS ao bit às da W15 na sonda — os
  perseguidores do campo de `1 000` caixas nunca têm a presa à vista a `15 m`; num campo aberto (o gate abaixo) os
  oito perseguidores de um herói que salta, com um orçamento de `1`, passam de `195` tiques-agente à espera a **`0`**.
  O tecto do passo em paralelo **não serve** (`150` lamas, `200` agentes):

  | tecto (orçamentos) | uma thread: pior tique depois da porta | todos os núcleos | atraso (régua da 1.ª) | sem caminho |
  |---|---|---|---|---|
  | nenhum (a W15) | `40,6 ms` | `10,5` | `52 · 8,7` | `0` |
  | `8` | `33,0` | `13,1` | **`151 · 51,9`** | `0` |
  | `4` | `22,9` | `14,6` | `167 · 38,9` | **`96`** |

  ⇒ a vazão que a cena de stress pede (`~4 M` de trabalho por porta, `~127 mil` por tique) é o que UMA thread paga;
  limitar o trabalho troca o tique pela espera. A 3.ª rodada nasceu daqui: o atraso vinha da ORDEM do passo em
  paralelo (a procura de quem persegue começa com trabalho `0`, atrás de todas) — mediu-se dar-lhe a vez primeiro.
- **A 3.ª** mediu a procura de quem persegue primeiro com a régua da 1.ª: `52 · 8,7 → 42 · 7,2` a `200`, e nada sem
  lama. A régua estava ERRADA: contava como espera a dívida da porta, em que o perseguidor anda um caminho que ainda
  serve. **A 4.ª** mediu com a régua certa — os tiques em que fica por acabar a procura pedida porque o alvo andou
  (`AMeio::persegue`) — com load `56–62` (outras linhas a compilar: o relógio desta rodada não vale; as colunas de
  trabalho e de espera não dependem dele):

  | `150` lamas | atraso do perseguidor (máx · média) a 10 · 50 · 200 | crítico (máx) a 200 | trabalho por porta a 200 |
  |---|---|---|---|
  | A inteira | `0 · 0` em todos | `789 mil` | `7,66 M` |
  | B W15 (= B vista, nenhum à vista) | `5 · 1,6 · 4 · 1,6 · 30 · 2,2` | `40 mil` | `4,07 M` |
  | B quem persegue primeiro | `5 · 1,6 · 4 · 1,6 · ` **`4 · 1,7`** | **`160 mil`** | `4,31 M` |

  Sem lama o atraso real é `0` a 10 e 50 agentes e `1 · 1,0` a `200` — o «um tique» do §23.8 estava certo; o
  `52 · 8,7` da 1.ª rodada era a régua.

### §25.5 — Contra os kill-criteria

| item | veredito |
|---|---|
| **A** | ✓ o censo `nenhuma_cena_de_smoke_desenha_fora_do_corpo` vermelho antes (`58` desenhos fora em `41` cenas, `130` corpos), verde depois (`42` cenas com a arma, `139` corpos, o piso afirmado), com o CONTROLO; nenhum corpo mudou ⇒ os `842` gates da crate passam sem mudar um número. A decisão, por caso, abaixo. |
| **B** | ✓ `194 · 200 · 207` tiques contra o CONTROLO parado `194 · 198` (`≤ 1,05×`); sem o contorno `213 · 578 · 311`; folga `0,30 · 0,30 · 1,00 m`, recuo `0`; «à frente» `216 → 205`. A crate inteira (`831`) passa sem mudar um gate. |
| **C1** | ✓ sem código: B gasta MENOS por porta que A (`0,92` contra `1,66 M` a `10`; `4,07` contra `7,66` a `200`) e o CONTROLO de B fica abaixo do de A. O `6,4 ms` contra `0,7` da W15 comparava B com lama com a cena sem lama. |
| **C2** | ⚠️ o critério como escrito («`≤` o da A») é inatingível com QUALQUER orçamento — a A paga o pico, e o pico é o que a W15 tirou. Medido: o atraso real é `≤ 1` tique sem lama, `≤ 2,2` em média com `150` lamas (máx `30` a `200` agentes). **Entra o alvo à vista** (é a resposta da procura — não pode piorar nada — e tira a espera onde o alvo se vê); **recusada** a procura de quem persegue primeiro (máx `30 → 4`, mas o crítico `40 → 160 mil`: o pico de volta). |
| **D** | ✗ nenhuma cura cumpre: numa thread o pior tique a `200` agentes no stress é `40,6 ms` (a A: `49,6`); com menos procuras em paralelo ou um tecto do trabalho, ou o tique, ou a espera, ou os sem caminho pioram. ⇒ o custo fica escrito no ADR-0180, com a exigência para a web: o `rayon` sobre Web Workers (o resultado não muda — o gate de 1 contra 8 threads). Nenhum build web existe hoje no repo. |
| **E** | ✓ a fixtura `um_scrub_com_a_zona_de_outra_porta_no_fim_devolve_a_mesma_corrida` sangra sem a guarda (`o scrub para 31`). |
| **F** | ✓ `line-Components2` já não existe (nem a worktree, nem o ramo); o ADR-0180 continua livre no `main` e nas seis worktrees (máximo `0179`). |

**A decisão de A, por caso** (a regra: um corpo que RODA no sítio para encarar o movimento fica bola — uma caixa
que roda entra na parede sem teste —, e quem não roda pode ser a forma desenhada):

| caso | cura | porquê |
|---|---|---|
| quadrado `2r` sobre bola `r` (os bonecos das cenas de navegação, os heróis da vista de cima, da paralaxe, do recomeço e do abanão, o alvo dos projécteis) | desenho = o DISCO do corpo (`DISC_TILE_KEY`, novo no atlas) | o desenho passa a ser o corpo, ao texel; nenhum trajecto, passagem estreita ou `const assert` muda |
| barra sobre bola, herói `ToMovement` (gatilho, golpe, vida `1–4`, tipos, arena) | o disco do corpo + o filho «Rumo» (dentro do disco, `0,98 r`) | roda: tem de ficar bola; a barra dizia para onde ele olha, o «Rumo» continua a dizê-lo |
| barra sobre bola, projéctil (`face_velocity`, por omissão) e os emissores das partículas | o disco do corpo | roda; e é um MOLDE de fábrica — um filho não viaja com a cópia |
| a moeda (`0,45` sobre a caixa `0,44`) | o desenho com a constante do corpo | `5 mm` fora: a mesma constante nos dois |
| o morcego (caixa desenhada, W15) | — | já era o corpo |

### §25.6 — ⛔ Recusas MEDIDAS

| recusado | medição |
|---|---|
| `PROCURAS_EM_PARALELO` menor (`8 · 4 · 2`) | `8`: perseguidores à espera até `149` tiques; `4`: `57` sem caminho; `2`: `138` (`150` lamas, `200` agentes) |
| um tecto do trabalho TOTAL do passo em paralelo (`8` · `4` orçamentos) | `8`: uma thread `33 ms` mas espera até `151`; `4`: `96` sem caminho |
| a procura de quem persegue primeiro | máx `30 → 4`, mas o crítico `40 → 160 mil` (cada recomeço de uma procura adiantada DOBRA a fatia) |
| o desenho dentro de uma bola encolhida (quadrado inscrito) | o que se via deixava de tocar a `0,29 r` das faces — a mentira ao contrário; o disco é o corpo exacto |
| um corpo-caixa para quem roda (herói, projéctil) | rodar no sítio não passa pelo teste de colisão: a caixa entraria na parede |

### §25.7 — A prova

Gates novos: `nenhuma_cena_de_smoke_desenha_fora_do_corpo` (o censo, `ph2d-app-components`) + o CONTROLO dele
`o_censo_ve_um_quadrado_sobre_uma_bola_e_aceita_o_disco` + `todo_roteador_que_a_shell_le_esta_na_familia` (`23`
roteadores) · `the_disc_tile_covers_the_inscribed_circle_and_nothing_outside_it` (`ph2d-render`; o teste da GPU
pede as peças reservadas juntas, e o branco sozinho deixou de ser público) · `o_caminho_contorna_um_corpo_pela_tangente_do_lado_mais_curto`
· `um_corpo_largo_que_vem_de_frente_e_contornado` (reescrito: `≤ 2×` o CONTROLO parado, o CONTROLO sem o contorno
a passar os `2×`, e os tiques em que o desvio corta o pedido `≤ 10`) · `o_alvo_a_vista_e_so_a_recta_que_nenhum_caminho_bate`
(`ph2d-nav`) · `quem_persegue_um_alvo_a_vista_nao_espera_a_vez` (+ CONTROLOS sem a vista e com a lama no meio) ·
`um_scrub_com_a_zona_de_outra_porta_no_fim_devolve_a_mesma_corrida`. Os gates da LEI DA FILA (`nav_nascer`) correm
sem a vista (todo alvo deles está à vista); o dos nascidos com a fila cheia (`nav_mundo`) põe o alvo atrás da parede.

Mutação **18 / 18** ([`mutacao_navegacao_w16_2026-10-05.py`](ferramentas/mutacao_navegacao_w16_2026-10-05.py), o
motor da W15), zero defeitos de arnês, árvore igual antes e depois. A 1.ª corrida deu `15 / 18`: `B7` e `V5` (o
contorno e a vista desligados por omissão) sobreviviam porque os gates LIGAVAM a alavanca à mão — agora correm o
produto por omissão e só o CONTROLO a desliga; `B2` (a tangente sem a folga do desvio) sobrevivia porque chegar
quase não muda (`207 → 214` tiques) — o que a folga compra é não apontar para onde o desvio proíbe: o gate conta os
tiques em que ele corta o pedido (`0 · 0 · 7` contra `51 · 58 · 95`).

### §25.8 — ⏳ O que fica

- **A web numa thread** paga as fatias do passo em paralelo (`40,6 ms` a `200` agentes no stress): a cura é da shell
  web que ainda não existe — o `rayon` sobre Web Workers (ADR-0180, adenda).
- **Quem persegue sem o alvo à vista**, na lama cerrada: `≤ 2,2` tiques em média, máx `30` a `200` agentes. A única
  alavanca medida que o encurta (a vez primeiro) traz o pico de volta pela DOBRA da fatia de um recomeço — uma cura
  teria de mudar essa dobra só para elas, e não foi medida.
- A dominância da procura ponderada (§22.4), como estava.

## §26 — W17 (2026-10-06): os três abertos do §25.8 num ciclo só (CLAUDE.md §0.10)

Os três itens do briefing de 06/10, um plano escrito ANTES de qualquer código (só a medição que o abre veio antes:
o perfil de C e o `cargo check` de B). **Série declarada:** nenhuma entre os itens — A (a fila da ponte e a condução),
B (só medir) e C (a procura ponderada da `ph2d-nav`) não partilham código nem régua; todos os candidatos de A e de C
entram no MESMO lote de binários como alavancas escolhidas em execução (`fila::Sonda` para A; `Polyanya` para C) e
medem-se na MESMA rodada intercalada. Série só DENTRO de A, e só se a rodada a pedir: se A1 falhar, a coluna de
diagnóstico (abaixo) diz se a dobra que traz o pico é a de quem persegue ou a das outras — e isso muda o candidato
seguinte; se for preciso, é uma 2.ª rodada e diz-se aqui porquê.

### §26.1 — A: quem persegue sem o alvo à vista, na lama cerrada

**O que se sabe (§25.4, 4.ª rodada):** a procura de quem persegue nasce com trabalho `0` e fica atrás das mais
adiantadas no passo em paralelo (`fila::procuras_a_meio` ordena pelo trabalho feito) — máx `30` tiques, média `2,2`
a `200` agentes com `150` lamas. Dar-lhe a vez primeiro, tal como está, dá máx `4` mas o crítico `40 → 160 mil`
(recusa §25.6). A régua é a da 4.ª rodada: os tiques SEGUIDOS com `AMeio::persegue` por acabar.

**Candidatos (alavancas da `Sonda`, todas no mesmo processo):**

- **A1 — a vez primeiro SEM a dobra para elas.** A procura de quem persegue vai à frente no passo em paralelo, e um
  recomeço dela não conta para `recomecos` (a fatia dela fica `pode`). Hipótese do briefing: o pico vem da dobra
  DELAS. ⚠️ Hipótese rival: o pico vem da dobra das OUTRAS — com `10` das `16` vagas tomadas, as adiantadas avançam
  menos, a porta apanha-as a meio mais vezes e a fatia delas dobra. **Coluna de diagnóstico** (nova na sonda): o maior
  `recomecos` visto numa procura a meio, separado em quem persegue e as outras. Ela decide entre as duas.
- **A2 — o FIM do caminho segue o alvo** (o `moveTargetPosition` do Detour): enquanto a procura de quem persegue
  está por acabar, se o troço do último canto do caminho até ao alvo NOVO se anda (`cost::segment_cost` é `Some`), o
  último ponto passa a ser o alvo novo — um caminho andável na hora; a procura completa continua atrás e, quando
  acaba, substitui-o. Nada muda onde o orçamento chega (a procura acaba no tique em que é pedida: não há janela).
  ⚠️ **A régua de A2 não pode ser a da 4.ª rodada**, que conta a procura por acabar — e A2 não a encurta, encurta o
  tempo em que o agente anda para onde o alvo JÁ NÃO está. Régua de A2 (nova, ao lado da da 4.ª, que continua
  impressa): os tiques seguidos com a procura de quem persegue por acabar E o fim do caminho a mais de
  `repath_distance` do alvo de agora (`0,5 m`, o de fábrica).
- **A3 — A1 e A2 juntos.**

**Kill-criterion (a `200` agentes, `150` lamas, a cena de stress do `medir_replaneio`):** entra o candidato com
**máx `≤ 5` tiques** (na régua dele, dita acima), **crítico `≤ 40 mil`**, e nenhuma outra coluna pior mais de `10 %`
(trabalho por porta, `falta`, sem caminho; o pior tique e o CONTROLO só valem a load `≤ 5` — acima disso valem as
colunas de trabalho e de espera). Se dois cumprirem, o mais simples. Se nenhum cumprir, recusa medida com a tabela.

### §26.2 — B: a web numa thread — o que desta linha compila para `wasm32`

Medido antes do plano (`target/prova/w17/wasm32_check.txt`, `cargo check --target wasm32-unknown-unknown`):
`ph2d-nav`, `ph2d-navmesh` e `ph2d-orca` compilam limpas; `ph2d-physics-ecs` NÃO — e o que falha não é dela: o codec
AVIF (`libavif-sys`, `libdav1d-sys`, `rav1e`: C e assembly), que chega por `ph2d-ecs → ph2d-asset →
ph2d-imageio-registry-init → ph2d-imageio-avif` (com `--keep-going` são só esses três). Vai para o ADR-0180 (adenda).
⛔ A shell web não se abre nesta linha: é decisão de PRODUTO do dono (perguntada no fecho).

### §26.3 — C: a dominância da procura ponderada (§22.4)

**O perfil novo** (`docs/Painter/ferramentas/amostra_gdb.py` sobre `medir_custo` `SO_GRANDE=1`, `release` com
símbolos; `target/prova/w17/perfil_dominancia.txt`), dentro de `find_path_costs`: `domina` `37 %`, dos quais o
`corte` (a geometria: `sqrt` e as bissecções) `32 %` e o percorrer da lista `~5 %`; o heap `~15 %`; o resto é a
expansão e as raízes. Confirma o de 04/10 (`37,6 · 25,4 · 17,2 %`). ⇒ indexar as frentes por aresta (o percorrer da
lista) vale pouco; o que pesa é a CONTA do `corte`.

**Candidatos (alavancas da `Polyanya`, escolhidas em execução):**

- **C1 — o `corte` barato, com a mesma resposta:** as rejeições baratas (a cobertura no parâmetro, sem `sqrt`) antes
  da desigualdade triangular, e o zero de `D(y)` em FORMA FECHADA (`g_A + w·|ρ_A − y| = g_B + w·|ρ_B − y|` sobre a
  recta da aresta é uma hipérbole cortada por uma recta: uma quadrática) no lugar das bissecções — e o corte continua
  conservador (fica o lado dominado).
- **C2 — outro heap:** a entrada de `24` para `16` bytes (o `seq` e o índice num `u64`) num heap de aridade 4 —
  menos falhas de cache por `pop`. A ordem dos `pop`s TEM de ser a mesma (a chave e o desempate não mudam).
- **C3 — menos polígonos na lama:** a fusão em convexos JÁ existe (`triangulate::merge_convex_labeled`,
  Hertel–Mehlhorn), mas só DENTRO do mesmo pedaço — e o pedaço é por ÁREA (`navmesh::pedacos`: um por
  `NavCostArea`, ids `1..`). Duas lamas do MESMO custo que se tocam ou sobrepõem são dois pedaços: a fronteira entre
  elas é uma aresta obrigatória e parte os polígonos dos dois lados. A alavanca: os pedaços pela CLASSE de custo e não
  pela área. ⚠️ Mexe num desenho da W7 (o custo vai na CONSULTA para que mudar a tabela não refaça malha) — por isso
  mede-se PRIMEIRO na sonda, sem código de produto: a mesma cena com as `100` lamas num id só (`costs = [1, w]`)
  contra os `100` ids. Só se essa medição, junta com C1 e C2, chegar a metade se desenha a porta do produto (série
  declarada: o desenho depende do número).

**Kill-criterion (o do §22.4, sem mudança):** entra só o que, sozinho ou junto, cortar o tempo da procura ponderada
na cena grande (`100` lamas, pesos `4` e `10`) a METADE ou menos — o mínimo de rodadas intercaladas no mesmo processo
—, com o custo contra o oráculo ao dígito de hoje (§17.1) e a procura em fatias = a inteira ao bit
(`a_procura_em_fatias_e_a_procura_inteira_ao_bit`). Senão, recusa medida com a tabela, e nenhum código de produto fica.

### §26.4 — A prova (planeada)

- **A medição:** UM lote de binários (`medir_replaneio` e `medir_custo`, perfil `smoke`, uma compilação), UMA rodada
  intercalada por sonda (as versões no mesmo processo, ordem rodada, o mínimo; a mediana só como controlo; loadavg
  anotado) — `target/prova/w17/`.
- **Os gates** de quem entrar: vermelho ANTES da cura, com CONTROLO e população afirmada; correm o produto por
  omissão e só o CONTROLO desliga a alavanca (a lição B7/V5 da W16). Quem não entrar deixa só a alavanca da sonda se
  ela for a régua de uma recusa — senão sai, e a sonda fica com a nota.
- **A mutação** (o motor da W16, `ferramentas/mutacao_navegacao_w16_2026-10-05.py`, copiado para a W17): cada lei
  nova; mutação que sobrevive = gate novo ou código morto cortado.

### §26.5 — A 1.ª rodada (`target/prova/w17/medir_{replaneio,custo}_w17.txt`, load `1,9–3,5`: o relógio vale)

**A** (`150` lamas, `200` agentes; o mínimo de 7 intercaladas, perfil `smoke`):

| versão | pior tique depois da porta | CONTROLO | crítico (máx) | trabalho por porta | atraso (régua da 4.ª: máx · média) | dobra máx (quem persegue · outras) |
|---|---|---|---|---|---|---|
| A inteira | `49,3 ms` | `64,4` | `789 mil` | `7,66 M` | `0 · 0` | — |
| **B produto** | **`11,5`** | **`8,7`** | **`40 mil`** | **`4,07 M`** | **`30 · 2,23`** | `1 · 1` |
| B produto · 1t | `42,0` | `22,2` | `40 mil` | igual | igual | igual |
| A1ctl primeiro COM a dobra (a recusa §25.6) | `11,7` | `8,4` | `160 mil` | `4,31 M` | `4 · 1,69` | `1 · 3` |
| A1 primeiro SEM a dobra delas | `12,5` | `7,7` | **`160 mil`** | `4,31 M` | `4 · 1,69` | `1 · 3` |
| A2 o fim do caminho segue o alvo | `11,7` | `8,0` | `40 mil` | `4,07 M` | `30 · 2,23` | `1 · 1` |
| A3 = A1 + A2 | `12,3` | `8,0` | `160 mil` | `4,31 M` | `4 · 1,69` | `1 · 3` |

- **A1 ✗ — e a coluna de diagnóstico diz porquê:** a dobra que traz o pico é a das OUTRAS (`3` recomeços seguidos =
  `8 ×` o orçamento = `160 mil`), não a de quem persegue (`1`). Com as procuras dele a tomar até `10` das `16` vagas,
  as adiantadas avançam menos, a porta apanha-as a meio mais vezes, e a fatia delas dobra. A hipótese do briefing
  (a dobra DELAS) estava errada; tirar-lhes a dobra não mexe um número.
- **A2 ✗:** o máximo fica em `30`. Na cena de stress (`1 000` caixas) o troço do último canto até à presa nova
  raramente se anda a direito; onde se anda, apaga só os episódios CURTOS (a média por episódio sobe: `2,21 → 3,38`).
- ⇒ **série declarada (a 2.ª rodada):** o candidato seguinte sai do diagnóstico — **A4, as procuras de quem persegue
  em vagas A MAIS**, à frente, sem tirar vaga às adiantadas (`16` continuam a ser delas). O crítico não sobe (nenhuma
  adiantada perde a vez); o preço possível é a coluna de UMA thread, que paga as vagas a mais.

**C** (a cena grande, `60` consultas, 7 rodadas intercaladas; o custo contra o oráculo IGUAL ao dígito em todas as
versões e pesos, `1,0000 · 1,0000 · 1,0053 / 1,0107 / 1,0000 / 1,0045` de máx a `1,5 · 2 · 4 · 10`):

| versão | peso 4: ms (mín) · ÷ hoje | peso 10 | nós expandidos (4 · 10) |
|---|---|---|---|
| hoje | `377,5` · `1,000` | `485,5` · `1,000` | `24 176 · 29 142` |
| C1 corte fechado | `358,4` · **`0,950`** | `459,0` · **`0,945`** | `24 178 · 29 144` |
| C2 heap de aridade 4 | `386,0` · `1,023` | `489,8` · `1,009` | igual |
| C1 + C2 | `0,967` | `0,957` | — |

- **C2 ✗ (recusa medida):** mais lento (`+1–2 %`). A entrada desce de `24` para `16` bytes sem mudar a ordem (o `seq`
  é sempre o índice mais um), mas o `BinaryHeap` da std (o *sift* até ao fundo, e depois para cima) já é o que um
  heap de aridade 4 escrito à mão não bate a estes tamanhos. Código cortado.
- **C1:** `5 %`. O `corte` era `32 %` do perfil, mas a maioria das chamadas sai antes das bissecções (a cobertura,
  `delta > k`, `D(0) > 0`) — o que pesa é o NÚMERO de comparações, não a conta de cada uma.
- **C3 não foi medido:** a malha com um id só tinha os mesmos `14 571` polígonos — os pedaços são por ÁREA e não por
  id (`navmesh::pedacos`). Defeito da SONDA, não do candidato. Cura (sem mudar o produto): áreas SEGUIDAS do mesmo id
  são um pedaço só — na ponte cada `NavCostArea` tem id próprio, logo nada muda ao bit; na sonda, com um id, as lamas
  juntam-se. Mede-se na 2.ª rodada (série declarada: a medição de C3 dependia de corrigir a régua).

### §26.6 — A 2.ª e a 3.ª rodadas (`…_rodada2.txt`, `…_rodada3.txt`)

**C, 2.ª rodada** (load `5–8`: as razões são de versões intercaladas no mesmo processo; o custo contra o oráculo igual
ao dígito em todas): com as áreas SEGUIDAS do mesmo id num pedaço, a malha da cena grande passa de `14 571` a
`14 381` polígonos (`−1,3 %`) — as lamas sobrepõem-se pouco e a fusão de Hertel–Mehlhorn já tinha juntado o que
podia dentro de cada uma.

| versão | peso 4: ÷ hoje (mín) | peso 10 | nós expandidos (4 · 10) |
|---|---|---|---|
| C1 corte fechado | `0,948` | `0,944` | `24 178 · 29 144` |
| C3 um id | `0,952` | `0,938` | `23 469 · 28 069` |
| **C1 + C3** | **`0,903`** | **`0,886`** | `23 471 · 28 069` |

**A, 2.ª rodada** (load `8–10`: só as colunas de trabalho e de espera) e **3.ª** (`LAMAS=150`, load `4,4–5,8`; as
colunas de `0` lamas da 2.ª deram A4 igual ao produto ao bit), `200` agentes:

| versão | atraso (máx · média) | crítico (máx) | trabalho por porta | pior tique · 1 thread | CONTROLO · 1 thread |
|---|---|---|---|---|---|
| B produto | `30 · 2,23` | `40 mil` | `4,07 M` | `11,0 · 41,8 ms` | `8,0 · 22,8` |
| **A4** vagas a mais | **`4 · 1,69`** | **`72 mil`** | `4,03 M` | `11,8 · 46,8` (`+12 %`) | `8,6 · 24,5` |
| A5 = A4 sem a dobra delas | `4 · 1,69` | `72 mil` | `4,03 M` | `11,8 · 46,4` | `8,3 · 25,4` |

⚠️ **O achado que muda a leitura do critério:** o PRODUTO já paga `80 mil` de crítico a `10` e a `50` agentes nesta
mesma cena (as três rodadas: `80 001 · 80 003`) — a lei da dobra (`fatia_depois_de`) põe o tecto em
`orçamento · 2^k`, não em dois orçamentos. O `40 mil` a `200` agentes (§25.4) era a corrida que calhou, não um tecto:
o A4 muda os trajectos e uma procura adiantada cai noutro recomeço (`72 mil` = uma fatia de `80 mil` que acabou antes).
O A5 dá o MESMO ⇒ não é a dobra de quem persegue.

### §26.7 — Contra os kill-criteria

| item | veredito |
|---|---|
| **A** | ✗ **nenhum candidato entra.** A1 (primeiro sem a dobra delas: crítico `160 mil`), A2 (o fim segue o alvo: máx `30`), A4 (máx `30 → 4`, mas crítico `40 → 72 mil` e uma thread `+12 %`), A5 (= A4). O critério (crítico `≤ 40 mil`) foi escrito ANTES e não se muda depois da medida — mesmo sabendo agora que o produto passa os `40 mil` a outros `N`. |
| **B** | ✓ medido e escrito (ADR-0180, adenda 2): `nav`, `navmesh` e `orca` compilam para `wasm32`; a `physics-ecs` pára no codec AVIF, e nada dela. A shell web é decisão do dono. |
| **C** | ✗ **recusa medida:** o melhor par (C1 + C3) corta `10–11 %`, longe de metade; C2 piora. Mesmo a custo zero a dominância dava `1,6×` (§22.4) — o que falta é FAZER MENOS nós, e nada barato os tira. |

⇒ **Nenhum código de produto fica** (o produto é o da W16 ao bit: `git diff 640499189 -- crates shells
':!crates/*/examples/*'` vazio). As alavancas medidas vivem no commit `4ccf96f2c`; a sonda `medir_replaneio`
guarda a régua da 4.ª rodada e o diagnóstico da dobra (quem persegue · as outras).

### §26.8 — ⛔ Recusas MEDIDAS (W17)

| recusado | medição |
|---|---|
| a procura de quem persegue primeiro SEM a dobra delas (A1) | crítico `160 mil` igual ao com a dobra: a dobra que sobe é a das OUTRAS (`3` recomeços seguidos), que perdem vagas |
| o fim do caminho a seguir o alvo (A2, o `moveTargetPosition`) | máx `30` igual: no campo de `1 000` caixas o troço do último canto à presa raramente se anda a direito |
| as de quem persegue em vagas A MAIS (A4) e sem a dobra (A5) | máx `30 → 4`, mas crítico `40 → 72 mil` e uma thread `41,8 → 46,8 ms` |
| o corte da dominância em forma fechada (C1) | `5 %`: a maioria das chamadas sai antes das bissecções |
| o heap de aridade 4 com entradas de `16` bytes (C2) | `+1–2 %` (mais lento que o `BinaryHeap` da std) |
| as lamas do mesmo custo num pedaço (C3) | `−1,3 %` de polígonos, `5–6 %` de tempo; e mexia no desenho da W7 (o custo na consulta) |

### §26.9 — ⏳ O que fica

- **Quem persegue sem o alvo à vista**, na lama cerrada: máx `30` tiques (média `2,2`) a `200` agentes. A alavanca que o
  encurta (A4: `4`) custa o crítico — e o crítico do produto já não é `2` orçamentos mas `orçamento · 2^k` (`80 mil` a
  `10` e `50` agentes). Uma cura de A passa por limitar a DOBRA das adiantadas (vivacidade com outro tecto), não pela
  vez de quem persegue.
- **A web numa thread**: a shell web não existe; o 1.º degrau é tirar o codec AVIF do caminho de `ph2d-ecs` (ADR-0180,
  adenda 2) — decisão do dono.
- **A dominância da procura ponderada** (§22.4): o que falta é fazer menos nós; as alavancas de custo por nó estão
  esgotadas (§26.8).

## §27 — W18 (aberta em 2026-10-06): fechar o aberto da §26.9 e MOSTRAR a lama (CLAUDE.md §0.10)

**Ordem do dono (06/10):** *«ainda não vi a lama em nenhum smoke. Mas vamos finalizar o que está em aberto»*. A web
fica ESTACIONADA (aprovada, «não agora»). Três itens, um plano; os critérios abaixo foram escritos ANTES do código.

### §27.1 — S: a cena da lama (`PH2D_NAV_SMOKE=5`) — o dono nunca viu a lama

**Medido:** nenhuma das `4` cenas de `nav_smoke` (nem outra da família) tem um `NavCostArea`; a lama (W7) só aparece
no Inspector. ⇒ a W7, a W9 (dominância), a W14, a W15 (fatias) e a C2 da W16 nunca tiveram uma cena — a lei
*«cena de smoke que ensina o contrário… pior que ausente»* vale também para a ausente.

**A cena:** FORMAS visíveis (as poças desenhadas do tamanho do corpo do sensor — o censo da W16 vale), duas lamas de
peso diferente (leve `2`, pesada `10`) entre o nascimento e o alvo, de modo que a escolha se VÊ: os agentes cortam a
leve quando o desvio é longo e contornam a pesada; um perseguidor atrás do herói na lama (o item A, visível).

**Kill-criterion:** um gate pela cena real (`nav_smoke::montar(…, 5)`) que mede a escolha — a fixtura AFIRMA agentes
que atravessam a leve (`≥ 1`) e nenhum que atravessa a pesada, com o CONTROLO (a mesma cena sem custos: o caminho
cruza as duas) —; o censo do desenho e o `todo_roteador_que_a_shell_le_esta_na_familia` verdes; `CENAS = 5`; a
cena fotografada (`docs/Components/ferramentas/fotografa_cena.sh`) antes de ir ao dono; o tutorial
`03_navegacao.pdf` ganha a página da lama.

### §27.2 — A: o tecto do caminho crítico, e depois quem persegue

**O que se sabe (§26.6):** a lei da dobra (`fatia_depois_de`) deixa o crítico chegar a `orçamento · 2^k` — o
produto paga `80 mil` a `10` e `50` agentes. As vagas a mais (A4) encurtam quem persegue (`30 → 4`) mas caem noutra
dobra (`72 mil`). ⇒ a ordem é **primeiro o tecto, depois o A4 por cima** (série declarada: o A4 só se re-mede com o
tecto escolhido).

**Candidatos para o tecto** (alavancas da `Sonda`, no mesmo processo; a vivacidade numa malha que nunca pára vem de
OUTRA coisa que não o tamanho da fatia):
- **T1** a fatia NÃO cresce; quem recomeçou `k` vezes seguidas vai à FRENTE no passo em paralelo (a vivacidade pela
  vez, não pelo tamanho: acaba se a procura couber em `orçamento × intervalo da porta`);
- **T2** a dobra só conta quando a procura gastou uma fatia INTEIRA desde o recomeço anterior (quem perdeu por falta
  de VEZ não dobra);
- **T3** a dobra com tecto em `2` (`fatia_depois_de` satura em `2·pode`).

**Kill-criterion:** crítico `≤ 2 ×` o orçamento (`40 mil`) a `10 · 50 · 200` agentes nas duas cenas da
`medir_replaneio`; `0` sem caminho; `falta` `≤ +10 %`; o gate `uma_malha_que_nunca_para_nao_deixa_a_procura_sem_acabar`
verde (a vivacidade) — e com o tecto escolhido, o A4 por cima: quem persegue máx `≤ 5`, uma thread `≤ +10 %` (load
`≤ 5`). Senão: o tecto entra sozinho se cumprir o seu critério, e o A4 fica recusa medida.

### §27.3 — C: menos NÓS na procura ponderada

**O que se sabe (§26.8):** o custo por nó está esgotado; a metade do tempo só vem de expandir menos. **1.º passo
(medir antes de propor):** onde estão os nós — a fracção dos expandidos com `f` perto de `C*` (o fundo que todo A\*
exacto paga) contra os longe dele, e dentro da lama contra o chão; quantos são frentes paralelas que a dominância corta
a meio (`trimmed`) e sobrevivem. Só depois os candidatos, cada um contra as recusas do §22.4 e da §26.8.

**Kill-criterion (o do §22.4, sem mudança):** metade do tempo na cena grande (`100` lamas, pesos `4` e `10`), o custo
contra o oráculo ao dígito, a procura em fatias = a inteira ao bit. ⚠️ Pode acabar outra vez em recusa medida — e
então diz-se ao dono em uma frase, sem a vender como feita.

### §27.4 — S feita: a cena 5, e o que a régua corrigiu nela (`nav_smoke_lama.rs`)

**A cena final:** duas pistas iguais, separadas por uma parede ao meio; em cada uma, uma faixa de lama de
`8,2 × 0,8 m` (sensor desenhado do tamanho do corpo) da parede do meio até `1,6 m` da parede de fora, e três
corredores VERMELHOS (`1,2 m/s`) que andam de baixo para a sua bandeira VERDE. Esquerda: `Light Mud`
(`Cost 2`); direita: `Heavy Mud` (`Cost 10`, a ESCOLHIDA). Os gates correm a cena real
(`nav_smoke::montar(…, 5)`):

| gate | medido |
|---|---|
| `a_leve_atravessa_se_a_pesada_contorna_se` | produto: atravessaram `3` a leve, `0` a pesada, chegaram `3 · 3`; CONTROLO (as duas a `1`): `3 · 3` atravessam |
| `com_o_cost_em_2_os_da_direita_cortam_pela_lama` | o `Cost` da pesada a `2` no tique `60` (todos abaixo da lama): cortam `2` de `3` (o 3.º, junto à passagem, segue a volta — ali ela é curta), chegam os `3` |
| `a_cena_tem_as_pecas_que_o_roteiro_nomeia` | os nomes e os dois custos |

**O que a régua corrigiu no caminho** (cada um medido antes da cura):

| achado | medido | cura |
|---|---|---|
| a passagem de `1,2 m` | três corpos de `0,35` empurravam-se nela; um raspava a quina (`1 cm` dentro) | `1,6 m`; e a régua de «atravessou» exige o centro a mais de um raio da PONTA da faixa |
| seis perseguidores do herói | cercavam-no e PRENDIAM-no (o herói parado com as setas premidas) | bandeiras, um alvo por corredor |
| um perseguidor do herói a cruzar a cena | ao passar pelas bandeiras empurrava um corredor parado para a borda da lama | saiu (o A, que ele mostraria, ficou recusa — §27.7) |
| na pista da direita, a ordem das bandeiras | o 1.º a sair da passagem estacionava no caminho dos outros | a ordem inverte-se: o 1.º vai à mais funda |
| ⭐⭐ **o `Cost` mexido com o agente a ANDAR** | um CONTROLO MORTO: `10 → 2` no tique `20` ou `60`, `0` procuras novas nos `5` tiques seguintes; só `→ 1` mudava algo (a lama sai da malha) | o 5.º motivo de replaneio (abaixo) |

**O 5.º motivo de replaneio** (o Q6 tinha quatro): `AgentRuntime::custos_do_caminho` guarda a assinatura dos
CUSTOS e dos ATALHOS com que o caminho em curso foi planeado (`Vez::custos`, o hash de base de
`entradas_das_procuras`, sem o conteúdo da malha — esse continua com a fila, senão cada porta replanearia
todos); mudou, quem anda pede uma procura, que paga do orçamento como as outras. O runtime vai no anel: o scrub
refaz o mesmo. Gate `mexer_no_custo_refaz_o_caminho_de_quem_anda` (`nav_custo`): a lama a `10` contornada; no
tique `20` passa a `1,05` — UMA procura nova, e atravessa; o CONTROLO sem a mudança: `0` procuras, não pisa.
Um `Nav Link` posto ou tirado com o agente a andar entra pela mesma porta.

### §27.5 — A: a rodada (`target/prova/w18/medir_replaneio_w18.txt`; load `8–17`: valem só as colunas de trabalho e de espera)

A vivacidade, pelo gate `uma_malha_que_nunca_para_nao_deixa_a_procura_sem_acabar` com cada alavanca (a pedra muda
a malha de dois em dois tiques; a procura precisa de `8` orçamentos): produto ✓ · T1 ✗ *«a procura nunca acabou»*
· T2 ✓ · T3 ✗. **A aritmética diz porquê antes da medida:** uma procura de trabalho `W` numa malha que muda de `K`
em `K` tiques só acaba se a fatia for `≥ W / K`; com o tecto em `2` orçamentos e `K = 2`, `W` tem de ser `≤ 4`
orçamentos — a fixtura pede `8`. Sem levar trabalho através de um recomeço (recusado, §23.6), **um tecto do
crítico e a vivacidade numa malha que nunca pára não cabem juntos.**

`150` lamas (com `0` lamas o crítico do produto já é `≤ 25 mil`):

| versão | crítico máx (10 · 50 · 200) | atraso de quem persegue (máx · média, 200) | `falta` (10 · 50 · 200) | vivacidade |
|---|---|---|---|---|
| B produto | `80 001 · 80 003 · 40 011` | `30 · 2,23` | `66,37 · 69,42 · 70,72` | ✓ |
| T1 a fatia não cresce, quem recomeçou vai à frente | **`20 017 · 20 015 · 20 017`** | `30 · 3,98` | `66,62 · 69,72 · 71,01` | ✗ |
| T2 só dobra quem pausou com o orçamento inteiro | `80 001 · 80 003 · 40 011` | `30 · 2,23` | = B | ✓ |
| T3 a dobra satura em `2` | **`40 004 · 40 008 · 40 011`** | `30 · 2,23` | = B | ✗ |
| T1 + A4 | `20 017 · 20 015 · 20 017` | `6 · 1,94` | `66,62 · 69,72 · 70,95` | ✗ |
| T2 + A4 | `80 001 · 80 003 · 72 307` | `4 · 1,70` | = B | ✓ |
| T3 + A4 | `40 004 · 40 008 · 40 011` | **`4 · 1,69`** | = B | ✗ |

`0` sem caminho em todas. ⚠️ O pico de `80 mil` do produto não se vê no relógio: o pior tique depois da porta a
`10` agentes fica em `6,5–6,8 ms` em TODAS as versões (só como razão, load alto).

### §27.6 — C: onde estão os nós (`target/prova/w18/diag_nos*.txt`, a cena grande, `60` consultas, `14 571` polígonos)

O registo de cada nó EXPANDIDO (o `f` com que saiu do heap, o custo da região, a aresta, a raiz e o intervalo),
classificado contra as outras frentes da MESMA aresta (polígono, entrada) em `17` amostras do intervalo. ⚠️ A 1.ª
leitura misturava a fase GERAL (a procura sem custos que corre antes da ponderada, sem dominância) com a
ponderada, e acusava `24 %` de nós dominados que a dominância «deixava»; o tecto amostrado (abaixo) desmentiu-a,
e separar as fases explicou-o.

| | peso 4 | peso 10 |
|---|---|---|
| expandidos / consulta (sem custo: `5 205`) | `24 176` | `29 142` |
| da fase GERAL | `21,5 %` | `17,9 %` |
| refractam (o custo muda na aresta) | `14,0 %` | `14,7 %` |
| 1.º na aresta | `11,7 %` | `12,0 %` |
| dominados inteiros pelas ANTERIORES do mesmo custo — **os que a dominância de hoje já corta** (`2 553 · 3 044` / consulta) | `10,6 %` | `10,5 %` |
| … contando as de outro custo | `0,0 %` | `0,0 %` |
| dominados só pelas POSTERIORES (a ordem do A*) | `9,9 %` | `11,9 %` |
| dominados em parte | `31,5 %` | `32,1 %` |
| não dominados | `0,7 %` | `1,0 %` |
| `f / C* < 0,8` (o que um heurístico mais apertado poderia cortar) | `6,1 %` | `15,8 %` |

Os tectos, medidos no mesmo processo (custo contra o de hoje igual ao bit em todos):

| alavanca | nós / consulta (peso 4 · 10) | ÷ hoje |
|---|---|---|
| C4 a dominância até ao ponto fixo (repetir a passagem até nada mudar) | `23 923 · 28 798` | `0,990 · 0,988` |
| **o tecto de QUALQUER dominância contra as anteriores** (amostrada, mesmo custo ou qualquer) | `24 167 · 29 131` | `0,9996` |
| a fase geral inteira (o tempo da procura sem custo nas mesmas consultas) | — | `~10 %` do tempo (`41–61` de `420–580 ms`) |

### §27.7 — Contra os kill-criteria

| item | veredito |
|---|---|
| **S** | ✓ a cena 5, os gates pela cena real com o CONTROLO, o censo e o roteador verdes, `CENAS = 5`, fotografada, a página no `03_navegacao.pdf` — e um controlo morto curado no caminho (o `Cost` ao vivo). |
| **A** | ✗ **recusa medida.** T1 e T3 cumprem o crítico e reprovam a vivacidade (a aritmética da §27.5); T2 a cumpre e não mexe no crítico. Sem tecto escolhido o A4 não se re-mede por cima dele, e fica recusa (W17) — mesmo com T3 + A4 a dar `4` no atraso. O critério foi escrito ANTES e não se muda. |
| **C** | ✗ **recusa medida.** A dominância de hoje já está no tecto do que frentes anteriores podem cortar (`0,04 %` a mais); o heurístico toca `6–16 %` dos nós; a fase geral `~10 %` do tempo. Nem somados chegam a metade: o resto são nós que todo A* deste desenho expande (1.º na aresta, refracções, frentes que só a ORDEM deixou passar, dominadas em parte). |

### §27.8 — ⛔ Recusas MEDIDAS (W18)

| recusado | medição |
|---|---|
| T1: a fatia não cresce, quem recomeçou vai à frente | crítico `20 mil`, mas a procura NUNCA acaba numa malha que muda de 2 em 2 tiques |
| T2: só dobra quem pausou com o orçamento inteiro | vivo, crítico igual ao do produto (`80 mil` a `10` e `50` agentes) |
| T3: a dobra satura em `2` | crítico `40 mil`, a procura nunca acaba na malha inquieta |
| T1/T2/T3 + A4 | sem tecto que entre, o A4 fica recusa (`4` no atraso só com T2/T3) |
| C4: a dominância até ao ponto fixo | `−1 %` de nós |
| qualquer dominância contra as frentes anteriores | o tecto amostrado corta `0,04 %` |
| um heurístico mais apertado | só `6–16 %` dos nós têm `f < 0,8 C*` |
| saltar a fase geral | `≤ 10 %` do tempo, e ela é a saída cedo quando a lama não toca o caminho |
| a cena 5 com herói e perseguidores | os seis cercavam e prendiam o herói; um perseguidor a cruzar empurrava os corredores parados para a lama |

### §27.9 — A prova

Gates novos: `a_leve_atravessa_se_a_pesada_contorna_se` · `com_o_cost_em_2_os_da_direita_cortam_pela_lama` ·
`a_cena_tem_as_pecas_que_o_roteiro_nomeia` (com a secção que a shell abre) · `o_cenas_conta_os_niveis_do_roteador`
· `mexer_no_custo_refaz_o_caminho_de_quem_anda`. Mutação **10 / 10**
([`mutacao_navegacao_w18_2026-10-06.py`](ferramentas/mutacao_navegacao_w18_2026-10-06.py), o motor da W15):
L1 a pesada a custar o mesmo · L2 a faixa sem a área · L3 o roteador sem a `5` · L4 a secção do agente aberta ·
L5 `CENAS = 4` (sobreviveria sem o gate do roteador, escrito para ela) · L6 a lama desenhada maior que o corpo ·
C1 sem o 5.º motivo · C2 o caminho instalado sem guardar os custos · C3/C4 a assinatura a zero. Gate batched:
`nextest-impacted` `15 782 / 15 783` (a falha, o tutorial a marcar `Cost 2` como rótulo de tela, curada e
re-corrida) · clippy limpo. As fotos: `target/prova/w18/foto/cena5_{a_meio,fim}.png`.

### §27.10 — ⏳ O que fica

- **O crítico do tique** chega a `orçamento · 2^k` (`80 mil` a `10` e `50` agentes na cena de stress) — é o preço
  da vivacidade numa malha que nunca pára, e não se vê no relógio. Fechado como escolha: não há tecto que não a
  parta (§27.5).
- **Quem persegue sem o alvo à vista**, na lama cerrada: máx `30` tiques a `200` agentes — o A4 que o leva a `4`
  só cabe com um tecto que a vivacidade não deixa.
- **A procura ponderada** fica no que é: a dominância no tecto, o resto é o A* deste desenho. Metade do tempo
  pediria OUTRO desenho (outra procura), não outra alavanca.

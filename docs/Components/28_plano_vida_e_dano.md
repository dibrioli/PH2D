# 28 — Plano: VIDA e DANO (2026-09-23)

> Ordem do dono: *«Busque o mais absoluto padrão ouro, sem olhar custos.»* A pesquisa é o
> [doc 27](27_pesquisa_vida_e_dano.md); o oráculo é o GDevelop 5.6.282 (MIT) corrido sem interface
> ([`ferramentas/gdevelop_health/`](ferramentas/gdevelop_health/)).
>
> ⚠️ Este plano é o DESENHO e as PERGUNTAS. Cada wave abre com a medição que a pode desmentir, e o
> que ela desmentir é reescrito aqui com a morte à vista (§0.0 do roteador).

---

## §1 — O que se constrói, numa frase por peça

| peça | mora | o que é |
|---|---|---|
| **`ph2d-health`** | crate-folha nova, **zero dependências** (o molde da `ph2d-shake`/`ph2d-topdown`) | a LEI pura: o pipeline de um golpe, a cura, a invencibilidade, a regeneração, o escudo — `(estado, pedido, dt) → (estado, eventos)` |
| **`Health`** (Vida) | CONFIG no ALVO, registada | máximo · inicial · invencibilidade após o golpe (s) · piscar · sobre-cura · regeneração (taxa, atraso) · escudo (máx., duração, regen, atraso, bloquear o excesso) · armadura (fixa, %) · esquiva · resistências por tipo · três nomes de sinal (levou dano · curado · morreu) |
| **`Damage`** (Dano) | CONFIG em QUEM BATE, registada | quantidade · tipo · equipa · **uma vez por contacto** (ou contínuo, por segundo, enquanto toca) · empurrão · o que acontece a quem bate (fica · some) |
| **`HealthRuntime`** | NÃO registado (a cerca é o TIPO — o molde do `CounterRuntime`) | o valor vivo, o escudo vivo, os relógios, quem já foi atingido por este contacto |
| **a ponte** | no **passo da física** | lê os `ContactEvent` (`Began`, com ponto, normal e impacto) e os `TriggerEvent`, cruza `Damage` × `Health`, chama a lei, publica os sinais |

---

## §2 — ⭐⭐⭐ A decisão de arquitectura: a vida vive no PASSO DA FÍSICA

**Medido antes de desenhar** (`ph2d-physics-ecs/src/bridge/contacts.rs:104-122`): o `ContactEvent`
é **diffado por TIQUE, contra a união dos sub-passos** — um toque rápido que começa e acaba dentro
de um tique **é reportado**, e um quadro que deve vários tiques reporta cada um. É o sítio exacto
onde as duas queixas nº 1 da pesquisa morrem por construção:

- **dano a dobrar num golpe** — um `Began` é UM por par e por toque; *«uma vez por contacto»* é o
  par `(dano, alvo)` numa lista de quem já foi atingido enquanto o toque durar;
- **golpe perdido em silêncio** — a união dos sub-passos apanha o toque que o Godot perde quando o
  hitbox nasce já sobreposto (⏳ a MEDIR na W2: um `Began` no 1.º tique de um corpo que nasce
  dentro de outro — é gate, não promessa).

⭐ **E o estado vai para o ANEL DE CHECKPOINTS da física** (o molde do `player_state`/`topdown_state`/
`projectile_state`, com a porta única `bridge::controllers` a servir o avanço **e** o replay):
⇒ **um scrub a meio da corrida devolve a vida EXACTA daquele tique**, não só o *«rebobinar é
renascer»* das irmãs. ⛔ A alternativa — a vida numa fase fora do passo, como o `CounterWatch` —
foi pesada e **recusada**: ela lê o que os tiques de um quadro deixaram, logo dois golpes em dois
tiques do mesmo quadro chegariam juntos e a invencibilidade entre eles não existiria.

⚠️ **O que isto custa, dito:** a vida passa a ser estado de SIMULAÇÃO — o `physics_ecs_c9` (o hash
determinístico que o CI compara nos três sistemas) ganha uma lane nova, e a lei tem de ser
**`f32` sem transcendentes** (ou com `libm`), senão os três sistemas discordam.

---

## §3 — O pipeline de um golpe (a ordem é a do oráculo, e o oráculo decide)

```text
pedido de dano → invencível? → esquiva → armadura fixa → armadura % → resistência do tipo → escudo → vida
```

- ⏳ **A ordem documentada pelo GDevelop é confirmada ou refutada pelas fixturas da W0**, passo a
  passo; a resistência por tipo é NOSSA (o GDevelop não tem tipos) e entra onde o RPG Maker a põe
  (multiplicativa, depois da armadura) — **divergência declarada**, com gate.
- **A vida nunca sai de `[0, máx]`** (ou `[0, máx·sobre-cura]`) e **há UMA porta de escrita** — a
  queixa do Roblox (dois caminhos, um que respeitava o limite e o escudo e outro que não).
- **Os três eventos duram o TIQUE em que nasceram** e são sinais, não condições — a queixa do
  GDevelop (*«Is just damaged» não dispara se a condição estiver acima do evento que causa o dano*)
  não se exprime numa tabela de sinais.

---

## §4 — As waves

| wave | entrega | abre com | smoke |
|---|---|---|---|
| **W0** | o ORÁCULO: o GDevelop corrido sem interface, as fixturas com cabeçalho (✅ **19** cenários — ver §7) | — | — |
| **W1** | a lei `ph2d-health`, com **paridade passo a passo** contra as fixturas | os gates das queixas (§2.2 do doc 27) escritos ANTES da lei | — |
| **W2** | `Health` + `Damage` + a ponte no passo da física, no anel; os três sinais; a morte pela porta do `Destroy`; verbos `Damage`/`Heal` na tabela | a medição do `Began` de um corpo que nasce sobreposto | um herói, três inimigos com vida diferente, uma arma |
| **W3** | a secção **Health** e **Damage** do Inspector | o censo de que toda secção chega a pixel | o artista monta um inimigo de 3 golpes sem tabela nenhuma |
| **W4** | **a barra de vida** — no HUD e **sobre a cabeça** (o `WorldSpaceWidget` do levantamento), com **rasto atrasado** | a medição de quanto custa uma barra por inimigo (N inimigos) | a barra que desce com o rasto branco |
| **W5** | **o impacto:** empurrão (a NORMAL do contacto), a pausa no golpe (*hitstop*), o piscar, os números de dano a subir | a medição dos três números da pesquisa (`~0,1 s` flash, `~0,05`/`0,15 s` hitstop) no nosso relógio | o golpe que PESA |
| **W6** | **tipos de dano e resistências** (fogo, veneno…) + **dano contínuo** (o veneno que dura) | — | um inimigo imune ao fogo e fraco ao gelo |
| **W7** | o **tutorial em PDF** + a cena final que junta tudo | — | o dono monta um jogo pequeno com vida do princípio ao fim |

---

## §5 — ⭐ Onde superamos a referência (cada linha com a fonte da vantagem)

- **O scrub exacto a meio da corrida** — nenhum dos motores da pesquisa volta atrás no tempo; o nosso
  devolve a vida do tique (§2).
- **A invencibilidade PISCA sozinha** — no GDevelop o `DamageCooldown` e o piscar são duas coisas que
  o artista liga à mão; aqui é um campo.
- **Dano a dobrar e golpe perdido morrem por CONSTRUÇÃO** (o `Began` por tique) — as duas queixas
  nº 1 do Godot e do Unity.
- **O empurrão sai da NORMAL real do contacto** — no Godot e no Unity cada tutorial a recalcula.
- **A barra com rasto atrasado vem de fábrica** — em todos os outros é um asset ou código.

---

## §6 — ⛔ Recusas antes de começar (com o motivo)

- **Um sistema de efeitos genérico à GAS** (atributos, tags, efeitos periódicos por fórmula) —
  a pesquisa mediu-o como *«overkill»* e C++ obrigatório para o público de um jogo simples; o nosso
  público é o do GDevelop. O dano contínuo (W6) é um campo, não uma linguagem.
- **O sinal passar a carregar NÚMEROS** — o contrato do `ph2d-runtime` é o NOME
  (`ph2d-runtime/src/lib.rs:61`); a quantidade viaja no componente de quem bate, que a ponte lê.
- **Reconstruir o que existe** — o flash é o `Tween` (canal `Silhueta`), o abanão é o `CameraShake`,
  a morte remove pela porta do `Destroy`, a contagem no HUD é o `Counter`.

---

## §7 — ✅ W1 FECHADA (2026-09-23): a lei `ph2d-health`, e o que a paridade achou

**Estado:** crate-folha [`ph2d-health`](../../crates/ph2d-health/) (**zero** dependências, sem um
transcendental — só `+ − × min max` em `f64`, logo ao bit nos três SO). A bancada
[`oraculo_do_gdevelop`](../../crates/ph2d-health/tests/it/oraculo_do_gdevelop.rs) corre a lei sob
`Regras::GDEVELOP` sobre as **19** fixturas e compara **três leituras por quadro** (`antes` ·
`depois` · `fim`) em todo campo público, nos internos e nos dois relógios, **por `to_bits`**, com os
sorteios da esquiva gravados **todos consumidos**. ⇒ **19 de 19, todos os quadros, ao bit.**

### §7.1 — O pipeline do §3 está CONFIRMADO pelo oráculo

`invencível? → esquiva → armadura plana → armadura % → escudo → vida`, e o `f3_armadura_ordem`
decide a metade que se podia trocar sem ninguém ver (`(25 − 5)·0,5 = 10` contra `25·0,5 − 5 = 7,5`).
⚠️ **O sorteio da esquiva é UM por golpe que passa a invencibilidade, mesmo com chance `0`** —
«optimizá-lo» desalinharia toda a sequência gravada, e há gate.

### §7.2 — O que o ALVO faz e a casa COPIA (medido, com o cenário)

- `ActivateShield` **SUBSTITUI** os pontos, não soma (`e1`), limitado pelo `MaxShield` se houver.
- Um golpe que **só toca o escudo** arma a invencibilidade (`e5`).
- A expiração do escudo zera **UMA vez** (é um `Once()`), e um escudo activado **sem renovar** com a
  duração vencida fica à espera em vez de morrer no quadro seguinte (`e3`).
- O relógio da duração do escudo **não existe** até alguém o repor ⇒ o `ShieldTimeRemaining` lê a
  duração INTEIRA com o escudo por activar — leitura enganadora do alvo, reproduzida e nomeada.
- Regenerar um escudo **a zero** é reactivá-lo com duração nova (`e4`).
- A regeneração **corta no máximo, com e sem sobre-cura**, e a do escudo no `MaxShield` (`d3`).

### §7.3 — ⛔ O que o alvo faz e a casa NÃO copia (`Regras::CASA`, divergências DECLARADAS)

Cada uma tem gate com as **duas metades** — a casa faz o que o produto quer **e** o controlo
`Regras::GDEVELOP` reproduz o que o oráculo mediu ([`lib_tests.rs`](../../crates/ph2d-health/src/lib_tests.rs)):

| o alvo | a casa | cenário |
|---|---|---|
| a vida desce a **negativo** (`100 − 130 = −30`) | pára em `0` | `a_dano_basico` |
| um morto **não é final**: uma cura ressuscita-o (`−30 + 50 = 20`) | só a porta `reviver` o tira da morte; a cura recusada não acende a marca; um golpe num morto não sorteia | `a_dano_basico` |
| pedidos **negativos** passam (`Heal(−20)` tira, `Hit(−10)` grava) | pedido negativo ou não-finito não faz **nada**, nem sorteia | `a_dano_basico` |
| ⭐ **com sobre-cura aplica a quantidade da cura ANTERIOR** (`Heal(50)` sobe `30`) — **defeito do alvo, achado pelo oráculo** | cura o que se pede | `c_cura_overheal`, passo 5 |

### §7.4 — ⚠️ Duas lições de INSTRUMENTO

- ⛔⛔ **O `serde_json` de omissão ERRA O ÚLTIMO BIT de um `f64`** (`0.19999999999999998` lido
  `0.2`): a 1.ª corrida leu **dez** fixturas a divergir por um ULP, todas nos relógios, e **a lei
  estava certa**. ⛔ A feature `float_roundtrip` **não** foi ligada — as features de dev-deps unificam
  na workspace e mudariam o `serde_json` de toda crate compilada ao lado —, logo a bancada tem um
  leitor exacto próprio ([`json_exacto.rs`](../../crates/ph2d-health/tests/it/json_exacto.rs)), com
  o número que o outro lê mal como controlo.
- ⛔⛔ **A 19.ª fixtura nasceu de uma mutação SOBREVIVENTE:** apagar o corte da regeneração passava a
  paridade inteira, porque o `d_regeneracao` sobe **1 ponto por quadro e cai EXACTAMENTE em 100** —
  *um corpus que nunca ultrapassa o limite não testa o limite*. O `d3_regeneracao_passa_do_maximo`
  (`45/s` = `0,75` por quadro, `99,75 → 100`) foi **corrido** para responder, e o piso da bancada
  subiu para `19` para que apagá-lo reprove.

**Prova de mutação: 17 de 17 sangram** ([arnês](ferramentas/mutacao_vida_2026-09-23.sh), com
controlo sobre o próprio filtro e modo `SECO=1` que confere só as âncoras).


---

## §8 — W2: o desenho, depois da sonda que a abre

A sonda [`mede_o_golpe_que_chega`](../../crates/ph2d-physics-ecs/tests/it/mede_o_golpe_que_chega.rs)
correu ANTES da primeira linha, e mudou o desenho em duas metades:

| caso | medido | consequência |
|---|---|---|
| A–C · dois corpos que **nascem** sobrepostos (sólido · sensor sobre cinemático · sensor cinemático sobre estático) | o toque chega no **1.º tique** | ✅ o *«golpe perdido quando o hitbox nasce sobreposto»* do Godot **não** existe nos nossos canais |
| D · um corpo que nasce **a meio** da corrida dentro de um sensor | visto no **dispatch em que nasce** | ✅ idem para as cópias de uma fábrica |
| E · **projéctil** (mover cinemático, sólido) contra um dinâmico, `6`–`120 m/s` | **zero** toques; o voo acaba (`Bounces`) e o alvo **não se mexe** (`0,0000`) | ⛔⛔ **a bala nunca ENCOSTA** — o mover pára rente ao obstáculo, logo o solver não tem o que reportar. Dano só por contacto **nunca** chegaria a uma bala |
| F · projéctil contra um estático | zero toques | idem |
| G · bala **só-sensor** | fica **parada onde nasceu** em todas as velocidades | ⛔ **defeito do #14:** o `move_character_from` devolve `none` quando o corpo não tem forma SÓLIDA — a bala-sensor, que é o padrão *hitbox que perfura*, é inexprimível hoje |

### §8.1 — ⭐⭐⭐ A fonte de um golpe são TRÊS canais, e a vida guarda a SUA memória

Um par `(quem bate, quem leva)` está a **tocar** num tique se aparecer em qualquer um de:

1. o **contacto sólido** do solver nesse tique (`tick_contacts`, a união dos sub-passos — quando há
   um dinâmico no par);
2. a **sobreposição de um sensor** depois do passo (`intersecting_collider_pairs`);
3. ⭐ **o que o próprio MOVER bateu** nesse tique — plataforma, vista de cima e projéctil (o
   `CharacterHit` do controlador), que é a metade que a sonda E obrigou.

⛔ **A vida NÃO lê o `contact_events` nem o `trigger_events`:** esses são canais de ECRÃ, e
re-baseiam em silêncio depois de um scrub (*«um scrub não é cem colisões»*). A vida tem de
reproduzir, num replay, **exactamente** os golpes da corrida — logo ela guarda a sua própria memória
de *«quem estava a tocar quem no tique anterior»* **dentro do estado que vai para o anel**. Um
`Began` é *«está a tocar agora e não estava no tique anterior»*, contra ESSA memória.

- ⭐ De graça: no tique 0 a memória está vazia, logo quem **nasce** sobreposto leva o golpe no
  1.º tique — a propriedade da sonda A–D passa a ser da VIDA e não de um acaso do canal de ecrã.
- ⚠️ Um mover encostado que **pára de empurrar** deixa de bater (o `CharacterHit` só existe quando
  ele se move contra o obstáculo): o toque «acaba», e o seguinte empurrão é outro golpe. A
  invencibilidade é o que impede isso de virar dano a dobrar — a mesma leitura do
  `get_slide_collision` do Godot.

### §8.2 — A ordem dentro do tique

`controladores (movem, e ANOTAM o que bateram) → passo do solver → A VIDA (lê os três canais) → anel`

A vida corre **depois** do passo e pela **porta dos dois laços** (a frente e o replay do
`rewind_to`), a lição do `bridge::controllers`: um tique de vida que só um dos laços chamasse seria
um scrub a devolver outra vida.

### §8.3 — Morte, sinais e o que a ponte NÃO faz

- A ponte **anuncia** e nunca apaga (o molde do `projectile_done`): a morte entra no dreno único da
  shell como `DeathCause::Killed`, e só quem `is_transient` sai da cena. ⚠️ Um inimigo de
  DOCUMENTO morre (deixa de levar golpes, grita o sinal) e **fica** — a mesma fronteira do `Destroy`.
- Os três sinais (levou dano · curado · morreu) saem pelo `signal_events` da física, com
  `source` = **quem tem a vida** e `other` = **quem bateu** — é isso que põe o `From Myself` do
  suplente #24 a funcionar sem uma linha nova.
- ⛔ **Num replay os sinais NÃO saem** (seria uma tempestade de cem golpes num scrub); o ESTADO anda.

### §8.4 — ⏳ O que fica para a W2b (fronteiras nomeadas)

- os verbos **`Damage`/`Heal`** da tabela de acções: eles chegam FORA do tique (a tabela corre no
  quadro), e para um scrub devolver a vida exacta o pedido tem de ser gravado **por tique** e
  re-aplicado no replay — o desenho de uma fita, não de uma fila;
- a bala **só-sensor** (defeito G), que pede que o mover deixe andar um corpo sem forma sólida.

### §8.5 — ✅ W2 FECHADA (2026-09-23): a ponte, a porta das mortes e a cena

- **A ponte** (`bridge::health`): a lei corre UMA vez por tique, depois do passo, pela porta
  `depois_do_passo` — chamada pelo laço da FRENTE (`publicar = true`) e pelo do REPLAY
  (`publicar = false`). O estado (`HealthState` = vida · gerador · memória do toque) vai no anel
  dentro do `ControllerMemory`, que é uma struct: esquecê-lo no `record`/`seed` não compila.
- **As três fontes, cada uma com gate próprio** (`tests/it/health.rs`): o contacto sólido (uma
  pedra que cai) · o sensor (um espinho, e quem nasce sobreposto leva o golpe no 1.º tique) · o
  canal do MOVER nas **três** pontes (a bala · a vista de cima · a plataforma cinemática).
  ⚠️⚠️ **Os gates das duas últimas nasceram da LEITURA do arnês de mutação:** o `extend` vive em
  três pontes e só a do projéctil tinha régua — *um canal ligado em um dos três ramos lê-se como
  ligado*. E o do contacto sólido pela mesma leitura: nenhum gate usava um corpo DINÂMICO.
- **A porta das mortes** (`bridge::mortes::mortes_anunciadas`) junta o voo acabado (`Spent`), a
  vida a zero (`Killed`) e a bala `Vanish` que bateu (`Spent`), filtradas por `is_transient` e sem
  repetidos. ⭐ Ela nasceu de um CORTE da shell (o bloco do projéctil saiu do
  `fase_fabrica_e_morte`, que encolheu `18` linhas), e ⛔ **não tinha gate nenhum** — a fase que a
  chama pede a `App`. Hoje: `so_sai_da_cena_quem_nasceu_numa_corrida`, com os dois CONTROLOS de
  documento (o inimigo morre e FICA; a bala pára).
- ⛔⛔ **O registo, o degrau de `PROJECT_SCHEMA` e o catálogo ESPERAM a W3**, e a razão é um gate
  que o portão apanhou: `every_registered_physics_component_has_a_ui_writer` e
  `every_registered_component_has_a_descriptor` reprovam um componente registado sem escritor de
  UI nem descritor — a saída que o RAIO (#21) já tomou. Consequência declarada: hoje uma `Health`
  **não viaja no `.ph2dproj`** e a paleta não a oferece; a cena monta-a por código. A W3 faz as
  quatro coisas no MESMO commit (registo +2 · degrau · catálogo · secção).
- **A cena** `PH2D_VIDA_SMOKE=1` (`ph2d-app-components::vida_smoke`): quatro alvos numa coluna com
  vidas `10`/`20`/`30` e o CONTROLO (um aliado da mesma equipa), e **zero** linhas de tabela — a
  morte é da vida. Os três gates dela correm a lei **com os números da cena** (lidos do mundo que o
  `montar` deixou), e o prólogo da shell passou a chamar a porta `liga_a_accao`, que apagou quatro
  cópias do bloco *«cria a acção e liga-lhe a tecla»*.
- **Prova de mutação: 23 de 23 sangram** (`ferramentas/mutacao_vida_w2_2026-09-23.sh`) —
  ⚠️ **22 de 23 na 1.ª corrida:** apagar o ramo do `damage_spent` na porta das mortes deixava o gate
  verde, porque a bala dele também acaba o VOO ao bater e o `projectile_done` anunciava-a pelo outro
  ramo. *Uma fixtura em que dois caminhos chegam ao mesmo resultado não testa nenhum deles* — o
  gate ganhou a pedra que cai (um `Vanish` que não voa), com o `Stay` como CONTROLO.
- **Portão:** `nextest-impacted` **17 835 / 17 835** · clippy `-D warnings` a zero · censos da
  árvore combinada **127 / 127**.

## §9 — ✅ W3 FECHADA (2026-09-24): as secções HEALTH e DAMAGE, e a CURA do smoke da W2

### §9.1 — ⛔⛔⛔ O smoke da W2 reprovou: *«ninguém sumiu ao levar muitos tiros»*

A causa não estava na lei nem na ponte: **a cópia profunda da fábrica leva só os componentes
REGISTADOS**, e a `Health`/`Damage` não eram. Os alvos nasciam SEM VIDA e as balas SEM DANO, em
silêncio. ⛔ **O gate da W2 `cada_alvo_morre_ao_tiro_que_a_vida_dele_diz` era cego a isto por
construção:** lia a RECEITA do mundo e montava o alvo À MÃO, sem passar pela fábrica. *Uma fixtura
que monta à mão o que o produto COPIA mede outro programa.*

- **A cura:** os dois componentes entram no registo da física (`39` registados) — e com isso o
  `PROJECT_SCHEMA` sobe **`168 → 169`** (a escada e a tripla), porque passam a viajar no ficheiro.
- **O gate que faltava:** `o_que_o_molde_tem_a_copia_tem` (a porta de cópia DO PRODUTO sobre todo
  `MasterRoot` da cena), e os helpers `receita`/`bala` dos gates da cena passaram a ler a CÓPIA
  (`copia()`), não a receita. Prova: sem o registo da vida **ou** do dano, `2` gates reprovam.

### §9.2 — As secções

- **`HealthNow`** é DERIVADO e NÃO registado: a ponte publica-o no fim de cada `dispatch`
  (`publica_vidas`, só quando muda) e o Inspector lê-o para `Now: X of Y`. *A vida de AGORA não é
  documento.*
- **Um vocabulário para as duas secções** (`ph2d_editor_core::vida_edits`, 26 edições): um inimigo
  que magoa ao toque mostra as duas, e duas listas pediriam à shell dois drenos para a mesma
  entidade.
- **A queixa vem antes dos números**, numa porta (`InspectorVidaInfo::queixa`): sem corpo · morto ·
  não fere, da mais específica para a mais geral.
- **Linhas que SOMEM:** o atraso da regeneração, as quatro do escudo, a semente e o *Overheal* só
  aparecem com o interruptor deles. O gate tem as duas metades (tudo ligado ⇒ tudo pintado; tudo
  desligado ⇒ nenhum condicional, com o CONTROLO de que a secção não sumiu inteira).
- **Os rótulos das seis caixas foram ENCURTADOS** (`Overheal` · `Absorb Rest` · `Per Second` ·
  `Pierce Shield` · `Pierce Armor` · `Vanish on Hit`): a varredura das elisões, com o Inspector
  armado com a vida, acusou os seis a `126 px` — *um nome perde a explicação antes de perder
  letras*, sem isenção nova na dívida.
- **A cena:** o Inspector vem à frente e o roteiro ganhou o passo (7) — clicar no roxo e ver
  `Now: 30 of 30` descer a cada tiro.

### §9.3 — Portão

`nextest-impacted` **17 854 / 17 854** · clippy `-D warnings` a zero · fmt · censos da árvore
combinada **127 / 127** · mutação **7 de 7** ([arnês](ferramentas/mutacao_vida_w3_2026-09-24.sh)) ·
a foto da cena monta os quatro alvos com o Inspector à frente. ⚠️ **O tiro real não se fotografa**
(o XTest é ignorado na Xwayland virtual): a corrente molde → cópia → golpe → morte está coberta
pelos gates da cena, que agora passam pela porta de cópia do produto.

---

## §10 — ✅ W2b FECHADA (2026-09-24): os verbos `Damage`/`Heal` da tabela, e a fita da vida

### §10.1 — O que se constrói

- **Dois verbos APENDADOS** ao `SignalVerb` (`Damage`, `Heal` — a posição é a tag do postcard ⇒
  **`PROJECT_SCHEMA` intocado**, e os três registos também). O argumento é **quanto**, e os dois
  entram no `uses_arg`. `ALL`, o array de ids do seletor (`INSP_ACTION_VERB`, `10 → 12`) e as
  chaves de i18n seguem-nos; os gates escritos à mão que os listam (`GRAVADAS`, os rótulos do
  dropdown) foram estendidos, e é isso que os torna **alcançáveis** pelo artista.
- **O verbo ANUNCIA, nunca aplica** (o idioma do `Destroy` e do `RestartRun`): o
  `ActionReport::pedidos_de_vida` leva `(alvo, PedidoDeVida)` e a shell entrega cada um à ponte por
  `PhysicsBridge::pede_vida`. ⚠️ O `ActionReport` perdeu o `Eq` — ele carrega um `f64`.
- **Inerte, contado e nunca inventado:** um argumento vazio, ilegível, `≤ 0` ou não finito, ou um
  alvo sem `Health`, não produz pedido. ⛔ Ler `"abc"` como `0` ou `"-3"` como uma cura seria o
  «aceita e mente». A recusa está nas DUAS portas (a da tabela e a da ponte), cada uma com gate.

### §10.2 — ⭐⭐⭐ A fita: o pedido é aplicado no PRÓXIMO tique e gravado POR TIQUE

A tabela corre por QUADRO e a vida anda por TIQUE dentro do anel. Um pedido aplicado «agora» seria
esquecido por um scrub (o replay não o sabia) ou re-aplicado por outro. ⇒ `fita_da_vida:
BTreeMap<tick, Vec<pedido>>`: o tique VIVO drena a fila e **grava por cima da própria entrada**
(remove-a se a fila está vazia); o tique de REPLAY lê a fita e **não publica**.

- ⭐⭐ **A regra é a da fita do dedo** (`InputTape::record`): *o artista que volta atrás e toca de
  novo está a autorar por cima*. Com a regra oposta — a fita a guardar a corrida velha — um tique vivo
  sem pedidos voltaria a ferir com o dano de uma corrida que já não existe (gate
  `um_tique_vivo_depois_do_scrub_grava_por_cima`).
- ⛔ **E nunca trunca o futuro**: a 1.ª redacção apagava a cauda da fita a cada tique vivo, e um
  salto para a FRENTE (que anda os tiques como um play) destruía os pedidos que o scrub seguinte
  devia refazer.
- ⚠️⚠️ **O gate do scrub reprovou primeiro sobre produto CERTO:** ele saltava `5 → 59` para a
  frente, e um salto para a frente é VIVO ⇒ grava por cima, que é a regra. O scrub que se compara à
  corrida é **só para trás, em ordem decrescente**, e atravessa os três pedidos dos dois lados
  (`41`/`40`, `21`/`20`, `11`/`10`).
- Um relógio **parado** guarda o pedido para o próximo tique: *um golpe da tabela não acontece fora
  do tempo da corrida*.

### §10.3 — O `On Heal` ganhou o primeiro produtor

`HealthEventKind::Healed { amount }` (o que a vida **ganhou**, nunca o que se pediu) e o braço
`Healed => on_heal` nos sinais. ⚠️ **A fonte de um facto de verbo é o PRÓPRIO alvo** — um verbo
não tem quem bata. ⛔ Um morto não é curado, e uma cura com a vida cheia **não produz facto nenhum**
(`depois > antes`), logo não grita.

### §10.4 — A cena: o `J` do veneno e a cura do roxo — e o que a FOTO apanhou

O roteiro ganhou o passo (8): com o roxo escolhido, o `J` publica `veneno` e ele responde com
`Damage 5` e arranca um relógio **dele próprio**, que um segundo depois publica `cura-lenta` — e ele
responde com `Heal 5` e grita `curou`. ⚠️ **O `J` foi medido** (uma das três letras sem braço no
teclado do editor; gate `a_tecla_do_veneno_nao_e_reclamada_pelo_editor`, com o `P` como controlo), e
a shell percorre uma lista (`vida_smoke::ACCOES`) em vez de duas chamadas soltas — a catraca da
shell desce.

- ⛔⛔ **A 1.ª redacção curava a cada segundo, sempre** (`autostart` + `repeat`), e a FOTO mostrou
  **cinco** avisos `cura-lenta` empilhados no topo do canvas aos `3 s`, a tapar `ai`/`morreu`/`curou`.
  A causa tem duas metades: o relógio vive no molde **e** na cópia, e **um molde reage a sinais e
  corre relógios** — lei desta casa que esta cena não muda. ⇒ o relógio passa a ser *one-shot* e
  **arrancado pelo veneno** (`StartTimer`); ele só fala depois de um toque, e arrancar um relógio
  que já anda recomeça-o (`timer::start`), logo vários toques seguidos dão uma cura só.
- ⚠️ **A cura ouve `From Myself`**: com `From Anyone` o relógio do molde curaria a cópia, e o de
  uma cópia curaria a outra.
- Gate `o_veneno_fere_o_roxo_e_so_o_relogio_dele_o_cura`, pela CÓPIA da fábrica, pela tabela
  (`resolve`), pela ponte da tabela (`apply`) e pela da vida: o veneno de outro objecto tira `5` e
  arranca o relógio · o sinal da cura vindo de OUTRO objecto não cura · vindo do roxo devolve `5` e
  acende `curou` · com a vida cheia nada. E `nenhum_alvo_precisa_de_tabela_para_morrer` foi
  reescrito com a premissa nova à vista: **nenhuma linha de alvo nenhum é um `Destroy`**, e só o
  roxo tem tabela, com exactamente as três linhas dele.

### §10.5 — ⛔⛔ A prova de mutação apanhou TRÊS gates meus a medir nada

A 1.ª corrida do [arnês](ferramentas/mutacao_vida_w2b_2026-09-24.sh) deixou **três VIVAS**:

- **O replay sem a fita e a fita sem gravar — os dois VERDES.** O anel guarda um retrato a cada
  `STRIDE = 10` tiques e um scrub semeia do mais novo e só REPLAYA o resto; o gate pedia nos tiques
  `10`/`20`/`40`, **exactamente sobre o passo**, logo cada scrub semeava de um retrato que já tinha o
  pedido aplicado e **nenhum replay atravessava um pedido**. *A fixtura não continha o fenómeno.*
  Hoje os pedidos caem em `13`/`27`/`45`, o gate **afirma antes de medir** que nenhum cai no passo,
  e os alvos ficam dos dois lados de cada um. O gate da regra de gravar por cima tinha a mesma
  cegueira (pedido no `10`) e ganhou a mutação que a mede (`F7`).
- **A porta da ponte a aceitar `≤ 0` — VERDE,** porque a lei já recusa o não finito e o negativo e
  o gate só olhava os PONTOS. ⛔ **Mas a recusa NÃO é redundante:** um golpe de **zero** passa a lei,
  **sorteia a esquiva** e, com ela certa, anuncia um `Dodged` — um toque da tabela que não feriu a
  gritar «esquivou». A régua passou a ser a ausência de FACTO sobre um alvo que esquiva sempre, com
  o CONTROLO de que o mesmo alvo esquiva um golpe válido.

E a ligação da shell (`fase_tabela_de_accoes` → `pede_vida`) **não tinha régua nenhuma** — a fase
pede a `App` —, e ganhou uma de TEXTO (`os_pedidos_de_vida_chegam_a_ponte`, por `include_str!`).

**Resultado final: 19 de 19 sangram**, com o controlo do filtro e o `RESTAURADO` verde nos cinco
grupos.

### §10.6 — Portão

`nextest-impacted` **17 818 / 17 818** (corrido sobre a W2b antes das três curas de régua acima; os
cinco grupos do arnês correram depois, verdes no `RESTAURADO`) · clippy `-D warnings` a zero nas
seis crates tocadas · fmt · censos da árvore combinada **127 / 127** · a foto da cena monta os
quatro alvos com o topo LIMPO. ⚠️ **O `J` real não se fotografa** (o XTest é ignorado na Xwayland
virtual): a corrente tecla → sinal → tabela → ponte → vida está coberta pelo gate da cena, pelo da
ligação da shell e pelo da tecla.

### §10.7 — ⏳ O que fica (fronteiras nomeadas)

- a bala **só-sensor** (defeito G), que pede que o mover deixe andar um corpo sem forma sólida —
  herdada da §8.4, ainda aberta;
- um verbo de vida com alvo por TAG fere cada membro, e cada um grava o seu pedido — **não medido**
  a N inimigos.

### §10.8 — O smoke da W2b: *«Now não desce 5»* — NÃO reproduzido, e o que a caça achou

⚠️ **O report não reproduz**, e a medição é pelo caminho INTEIRO: uma sonda temporária (retirada)
escolheu a CÓPIA do roxo como o dono faz e chamou `App::key_input` com `KeyCode::KeyJ` — o despacho
de teclado real, o mesmo que o `WindowEvent::KeyboardInput` chama sem guarda nenhuma. Com
`PH2D_SIGNAL_LOG=1`: `veneno <- a MAO do artista` → `4 efeito(s)` aplicados → `ai` → o `HealthNow`
da cópia `30 → 25` no tique seguinte → um segundo depois `cura-lenta` → `curou` → `30`. E a foto
numa tela de `3200` px de altura leu **`Now: 20 of 30`** no Inspector depois de três toques e uma
cura. ⇒ o motor, a tabela, a shell e o painel fazem o que o roteiro diz; a hipótese do ambiente do
dono fica aberta, com o diagnóstico nomeado (`PH2D_SIGNAL_LOG=1` diz se o `veneno` sai).

⚠️ **Nota de legibilidade, não de defeito:** a secção `Health` é a **última** do Inspector com um
alvo escolhido (a `2 900` px de altura numa tela alta) — na janela de `1 040` px ela exige rolar.

⭐ **E a foto achou um defeito REAL, pré-existente desde o #20:** o campo do parâmetro pintava
**sempre** *«timer name (empty = all)»* — na linha `Damage`, na `Heal` e no contador. O painel só
sabia SE o verbo lia o argumento e não O QUÊ. ⇒ `SignalVerb::arg_kind` (`TimerName` · `Count` ·
`Amount` · `None`, **sem `_`**: um verbo novo não compila sem dizer o que o campo dele é), o
`uses_arg` passa a derivar dele, a shell traduz para `ActionArgHint` (o painel não vê o
`ph2d-ecs`, ADR-0029) e a dica sai de `dica_do_parametro`. Três gates — o mapa à mão no motor, a
tradução na shell, as três dicas distintas e presentes na tabela de textos — e as duas mutações
das pontas novas sangram. Portão: `nextest-impacted` **17 821 / 17 821** · clippy · censos
**127 / 127**.

## §11 — ✅ W4 FECHADA (2026-09-24): a BARRA DE VIDA — sobre a cabeça, no placar, com rasto

### §11.1 — O que se constrói

- **A lei do rasto** ([`ph2d_hud::barra`](../../crates/ph2d-hud/src/barra.rs), pura): `Rasto
  { valor, ultimo, espera_s }` e `avanca` — um golpe recomeça a espera, depois da espera o rasto
  escorre a velocidade constante, uma cura puxa-o para cima, ele nunca fica abaixo da vida, e um
  `dt` inválido não o anda. `faixas` devolve **fundo → rasto → vida**, a crescer da esquerda.
- **O componente** `HealthBar` (`ph2d-physics-ecs`, **registado**): `target` (o NOME de outro
  objecto, **vazio = este**), tamanho, deslocamento, três cores, atraso e velocidade do rasto,
  `hide_when_full`. ⚠️ `PROJECT_SCHEMA` **169 → 170** e o registo da física **+1** (o gate da contagem
  foi a `40`, delta **+8** contra o `main`); ⛔ os espelhos não se mexem.
- **A ponte** ([`health_bar_bridge`](../../crates/ph2d-app-components/src/health_bar_bridge.rs)): o
  rasto de cada barra vive FORA do mundo (a lei das partículas: o que escorre numa corrida não é
  documento), e cada quadro produz três `RenderInstance` no ladrilho branco com `z_order = u32::MAX`,
  entregues no slot `extra` do passe de sprites (o **quinto** produtor). **Rebobinar é renascer**
  (`renascer_a_corrida`). A barra segue a **posição e a escala** de quem a carrega e **nunca a
  rotação** — uma barra torta não se lê.
- **Duas portas públicas com dois leitores cada:** `alvo_da_barra` e `vida_da_barra` — a ponte ao
  desenhar e o Inspector ao dizer o que a barra encontrou. *Escritas duas vezes, o painel diria
  «30 de 30» sobre uma barra que a tela desenha vazia.*
- **A secção `Health Bar`** do Inspector, **terceira da família** Health/Damage (o mesmo
  instantâneo, as mesmas edições, o mesmo dreno — ⇒ **zero** fiação nova na shell para o painel): a
  primeira linha diz **de quem** é a vida (`Shows 25 of 30` · *ninguém com esse nome* · *sem vida*),
  depois o alvo, os seis números, **três amostras de cor** (`register_picker_swatch`, sem braço de
  clique — a lei das amostras de script) e a caixa `Hide If Full`. ⚠️ Uma barra SOZINHA (o placar)
  **não se queixa de corpo**: só a vida e o dano precisam dele.

### §11.2 — ⭐⭐⭐ A medição que abria a wave: quanto custa uma barra por inimigo

A sonda `mede_o_custo_de_n_barras` (`--release`, `#[ignore]`, à mão) leu à **primeira** um custo
**super-linear** — `100 → 0,013 ms`, `1 000 → 0,387`, `10 000 → 22,9 ms` (`137 %` de um quadro): dez
vezes as barras custavam `59×` o tempo. ⛔ **Era meu, e eram DOIS:** a limpeza dos rastos fazia um
`any` linear dentro de um `retain` (`O(n²)`), e uma barra com `target` nomeado varria o mundo **por
barra**. Com o conjunto vivo num `BTreeSet` e um memo de nome **por quadro**:

| barras | ms / quadro | % de 16,67 ms |
|---|---|---|
| 100 | 0,009 | 0,1 % |
| 1 000 | 0,098 | 0,6 % |
| 10 000 | 1,956 | 11,7 % |

(`loadavg 14,87`, logo a coluna é tecto e não média.) ⇒ **nenhum `MAX_*`**: o número que a sonda dá
é o que diz se um tecto é preciso, e a `10 000` inimigos na tela a barra custa um oitavo de um quadro.

### §11.3 — A cena: uma barra por alvo e o PLACAR — e o que a FOTO apanhou

A cena da vida ganhou uma barra em cada receita de alvo (a cópia leva-a com o resto) e um **placar**
no alto: um objecto **sem vida** cuja barra mostra a do roxo **pelo nome** — a forma do HUD, pela
mesma porta. ⚠️ **O placar nomeia a CÓPIA** (`Alvo de 3 tiros (1)`): a porta de cópia dá a cada
cópia um nome livre e o molde já tem o nome sem sufixo — um gate prova que a cópia que a fábrica faz
é essa, senão o placar diria *«ninguém com esse nome»* com todos os outros gates verdes.

⛔ **A foto apanhou a barra do alvo de cima CORTADA pela borda da banda visível** (`0,62` com a altura
de fábrica acabava exactamente em `+4,09`) ⇒ `BARRA_Y = 0,57` e `BARRA_H = 0,1`, com gate nas três
cercas (o próprio alvo · o de cima · a borda). ⭐ E uma **variante temporária, nunca commitada**, da
cena (o relógio de arranque também envenena o roxo, e o rasto dele segura dez minutos) fotografou o
rasto: `25 de 30` a verde com o pedaço perdido a **branco**, e no placar o mesmo rasto já escorrido
até ao fundo. ⚠️ Os moldes do meio do ecrã **não** têm barra — e a ponte exclui-os pelas **duas**
marcas (`MasterPiece` e `MasterRoot`), porque o `MasterPiece` é derivado por um passe e antes dele
só a raiz existe.

### §11.4 — ⛔⛔ A prova de mutação apanhou TRÊS réguas minhas — e matou uma linha de produto

[`mutacao_vida_w4_2026-09-24.sh`](ferramentas/mutacao_vida_w4_2026-09-24.sh), **36 / 36 sangram**
depois das curas (a 1.ª corrida deixou três vivas e duas abortadas por agulhas que o `fmt` mudou):

- **L2 sobreviveu e o RAMO foi APAGADO:** o retorno cedo *«a cura cola o rasto»* não mudava um bit
  observável — o `max(agora)` do fim já o fazia. *Uma linha que a mutação não consegue matar não é
  lei, é comentário com sintaxe de código.*
- **B4 sobreviveu por FIXTURA:** o molde homónimo nascia DEPOIS do herói, logo nunca ganhava o
  empate da identidade e a cerca dele era invisível. ⚠️ E a 1.ª cura ainda falhava: sem `Transform`
  o molde nem recebe identidade.
- **I2 sobreviveu por fixtura no NEUTRO:** o dreno só via um deslocamento vertical positivo, e a
  mutação que o prende a `≥ 0` passava.

### §11.5 — Portão

`nextest-impacted` **17 910 / 17 912** com as duas reprovadas a serem a varredura das elisões no
degrau estreito — o rótulo novo `Hide When Full` era cortado; a cura é `Hide If Full` (*um nome perde
a explicação antes de perder letras*), e a varredura em âmbito de workspace fecha **12 / 12** ·
clippy `-D warnings` a zero nas nove crates tocadas · fmt · censos da árvore combinada **127 / 127**
· catraca da shell verde · gate de TEXTO novo na shell (as barras correm · são desenhadas · renascem).

### §11.6 — ⏳ O que fica (fronteiras nomeadas)

- uma barra filha de um `UiCanvas` segue a raiz conduzida pela propagação das poses — o placar da
  cena é um objecto de mundo, e **nenhum gate** mede a barra dentro de um canvas;
- a barra lê a vida de UM objecto; *«a vida de todos os inimigos»* seria outra grandeza;
- a bala **só-sensor** (defeito G) continua aberta.

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


# Handoff de integração — `line/physics` (2026-08-11)

**Status:** FECHADO 2026-08-11 · no `main` em `18c954bfe` (o commit que trouxe este arquivo).

> **A linha NÃO integra nem faz ship** (CLAUDE.md §0.7). Este documento é o que o
> integrador precisa para não colidir nem regredir. DIRETRIZ §1.5.9.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | **o tip de `line/physics`** ⚠️ ver abaixo |
| merge-base com `main` | `76788440adbabb0e5b12f8fdafecc6f1e1183e1a` |
| commits | **~42** |
| diff | 89 arquivos, +13.709 / −833 |

⚠️ **Todos são pós-integração de 2026-08-10** (a jornada `W-KinMove` /
modo cinemático, que já está no `main`). Nada aqui foi entregue antes.

⚠️ **O HEAD não é escrito aqui de propósito, e a razão é aritmética:** o commit
que o escreve MUDA o HEAD, então um sha nesta tabela é falso no instante em que
é commitado. O que identifica esta entrega é o **merge-base** acima mais *"o tip
da branch"* — que é o que um integrador usa de qualquer forma.

**O assunto é o PLAYER, em duas metades.** A primeira é o **catálogo** (o plano
08: nadar · a força de zona no cinemático · o sensor que varre o corpo). A
segunda — e é a que precisa de leitura — são **os SENSORES**: eles ficaram
visíveis, ficaram editáveis, e a perna deixou de ser um raio.

---

## 2. As waves, em uma linha cada

| wave | o quê | cena |
|---|---|---|
| `W-Swim` | nadar é um **REGIME**, e o limiar é a **linha de flutuação** (`buoyed ≥ 1`) | `=105` |
| `W-SwimLine` | o nadador parado **BOIA** — o repouso do nado é a linha | `=105` |
| `W-ZoneForce` | a **correnteza** leva um personagem cinemático | `=106` |
| `W-ShapeCast` | o sensor do agachar **VARRE o corpo** em vez de amostrar linhas | `=107` |
| `W-Probes` | os cinco sensores ficam **VISÍVEIS** (e seguem o corpo parado) | `=108` |
| `W-Probes2` | e ficam **EDITÁVEIS** — quatro números que eram `const` | `=108` |
| **`W-FootFan`** | **a perna é um LEQUE, não um raio** | **`=109`** |

---

## 3. Superfície de colisão

| item | valor | nota |
|---|---|---|
| `PROJECT_SCHEMA` | **70 → 73** | ⚠️ **três degraus, ver §4** |
| tripla do pin | `(73, 13, 14)` | `project_schema_tests.rs` |
| `physics_ecs_c9` | **`1699123f9ed2844f…`, 117 corpos** | debug ≡ release, medido no tip |
| registro `ph2d-physics-ecs` | **29, INTOCADO** | nenhum componente novo |
| registro `ph2d-ecs` + os 2 espelhos | **INTOCADOS** | |
| gizmo ids | **nenhum novo** (o último segue **973**, próximo livre **974**) | |
| ids novos | **10, todos `hash_node_id`** | ⇒ fora de todo gate de contagem |
| ADR | **nenhum** | ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **ZERO** | nenhuma crate nova, nenhuma dep nova |
| contrato congelado | **4/4** | rodado, não auto-relatado |
| `CLAUDE.md` | **não tocado** | a §5 desta linha é do integrador |

### Foundational / compartilhado tocado, e por quê

Fora de `crates/ph2d-phys*`, `crates/ph2d-platformer` e `docs/Physics/`:

| arquivo | o quê | aditivo? |
|---|---|---|
| `ph2d-editor-core/src/ids/inspector_player.rs` | 10 ids novos | **sim** |
| `ph2d-editor-core/src/screens/hero/inspector_model_player.rs` | campos do snapshot da §14 | **sim** |
| `ph2d-panel-inspector/src/{sections/player.rs,player_rows.rs}` | as rows novas | **sim** |
| `ph2d-panel-inspector/src/{event_player,populate_physics,sync_physics}.rs` | fiação das rows | **sim** |
| `ph2d-panel-inspector/tests/seam_player.rs` | seams + `PLAYER_ROW_COUNT` **40 → 42** | **sim** |
| `shells/desktop/src/main.rs` | 1 `mod` de cena | **sim** |
| `shells/desktop/src/physics_smoke.rs` | 1 braço do roteador (`"109"`) | **sim** |
| `shells/desktop/src/render_loop/physics_overlay*.rs` | o desenho dos sensores | **sim** |
| `shells/desktop/src/render_loop/mod.rs` | a leitura dos sensores no frame | **sim** |
| `shells/desktop/src/{project,project_schema_tests}.rs` | **o schema** | ⚠️ **não** |
| `shells/desktop/tests/the_overlay_reads_the_sensors_the_bridge_published.rs` | arch-gate novo | **sim** |

⚠️ **E `bridge/player.rs` foi PARTIDO** (734 > 700): a PERNA saiu para o irmão
`bridge/player_leg.rs` — *o que o tique FAZ com a resposta* × *como a resposta é
obtida*. **Pure code motion, provado**: o `c9` sai byte-idêntico ao commit
anterior. Uma linha que tenha tocado o bloco do cast de chão funde limpo contra
um arquivo de onde ele saiu.

---

## 4. `PROJECT_SCHEMA` — **CONTE, não copie**

A escada desta linha, contra o `main` em **70**:

| degrau | wave | o campo |
|---|---|---|
| **71** | `W-Swim` | os três do nado (`swim_speed` · `swim_accel` · `swim_enter`) |
| **72** | `W-Probes2` | os quatro dos sensores (`corner_samples` · `corner_lookahead` · `wall_samples` · `wall_spread`) |
| **73** | `W-FootFan` | os dois da perna (`foot_samples` · `foot_spread`) |

⚠️ **Se outra linha da janela bumpar, estes três valores são PROVISÓRIOS.** E
esta colisão **passa MUDA** quando as duas linhas escrevem o mesmo literal — o
`project.rs` não conflita e o git não sabe o que o número significa; quem
denuncia é o `project_schema_tests.rs` ao lado, e nem sempre. **Confira nos DOIS
arquivos.**

⚠️ **A escada do `project.rs` já traz as três entradas escritas** (a lição do
v69, que chegou ao `main` com a linha ausente).

---

## 5. O que muda de COMPORTAMENTO, nomeado

### 5.1 · A perna é um leque — e o default MUDA a física

⚠️ **É a única wave desta família em que isso acontece.** Os quatro números da
`W-Probes2` nasceram nas consts de sempre; aqui o valor antigo **é** o defeito.
Medido, parado sobre uma fenda que o corpo atravessa:

| fenda | corpo | queda | fração do `float_height` |
|---|---|---|---|
| 0,10 m | 0,40 m | **0,411 m** | **46%** |
| 0,40 m | 0,40 m | **113 m** | sai do mundo |

O `physics_ecs_c9` move-se por isso, e **a atribuição é por ablação**: com
`samples = 1` o hash volta **exatamente** ao do `main` (`fb27f676…`).

### 5.2 · Numa RAMPA o corpo cavalga mais alto

`meia-largura × spread × tan θ` — **7 cm a 20°, 18 cm a 45°** —, porque quem
vence a redução é o pé de CIMA. O piso da flutuação desce a mesma coisa.

⚠️ **`RideConfig::min_float_height` NÃO foi mudada, e a decisão está no gate
`the_fan_lowers_the_floor_by_exactly_the_uphill_foots_rise`:** ela descreve o
piso de uma perna de UM raio, então ficou **conservadora**. Nada regride (o valor
que ela dá hoje é o que dava ontem); o que ela custa é um personagem novo que
paira mais alto do que precisaria. Ensiná-la sobre o leque muda a assinatura e o
número que o artista lê em dois painéis ⇒ **wave própria**.

### 5.3 · Um degrau mais íngreme que o `max_slope` não é chão

Um pé de fora só conta se o chão dele for **caminhável** a partir do pé do meio.
Sem isto, um personagem que empurrava um caixote de 0,6 m passa a **SUBIR nele**
(deslocamento do caixote **7,27 → −0,02 m**).

### 5.4 · O pé fica DENTRO da pegada (`spread = 0.9`)

Um raio na borda EXATA é tangente a tudo o que o corpo encosta — o solver para a
cápsula a um contato da parede, e a borda coincide com a face. Com `1.0`, **seis**
gates do pulo de parede caem de uma vez (o personagem deixa de deslizar e passa a
cair **7,55 m em 1 s**).

⚠️ **A/B: com o recuo E sem o limite de degrau, o caixote volta a −0,02** ⇒ as
duas metades são load-bearing e nenhuma subsume a outra.

---

## 6. Os gates alheios que esta linha teve de corrigir

⚠️ **Todos diziam a verdade sobre a perna de ANTES** — é o preço de mover um
default, e nenhum foi afrouxado:

| gate | o que dizia | o que passou a dizer |
|---|---|---|
| `platform_idle::the_leg_still_holds_the_height_it_was_asked_for` | a folga sob o **CENTRO** | a folga sob o **pé de cima** — o raio que vence |
| `player_probe_view` (2) | *"uma perna, um raio"* | a contagem lida da **porta** do produto |
| `physics_smoke_probes_tests` (1) | idem | idem |
| `measure_float_floor::the_predicted_floor…` | media o leque contra uma fórmula que descreve **um raio** | mede **n=1**, que é o que ela descreve |

⚠️ **E a MENSAGEM da cena 108 dizia *"uma linha para BAIXO com um TIQUE"*** —
descrevia um produto que deixou de existir nesta wave. Corrigida para as três,
porque o **passo 1** dela é exatamente o que o artista olha.

---

## 7. Como o integrador confere

```bash
# a suíte, com --no-fail-fast: sem ele o primeiro binário vermelho ESCONDE o resto
cargo test --release -p ph2d-physics-ecs -p ph2d-platformer --no-fail-fast
cargo test --release -p ph2d-host-desktop --no-fail-fast

# o hash, nos dois perfis
cargo run --release --bin physics_ecs_c9 && cargo run --bin physics_ecs_c9
```

⚠️ **`--no-fail-fast` não é conforto:** foi ele que separou *"um gate caiu"* de
*"dez caíram"* nesta jornada, e a corrida sem ele deu o veredito errado.

---

## 8. Smokes

| cena | o quê |
|---|---|
| `PH2D_PHYSICS_SMOKE=105` | **nadar** |
| `=106` | a **correnteza** nos três modos |
| `=107` | as quatro **pedras** que não cabem num raio |
| `=108` | **o que ele vê** — os cinco sensores ⚠️ **re-smoke: a perna agora são TRÊS linhas** |
| **`=109`** | **A FENDA** — o leque, com o controle DENTRO do quadro |

⚠️ **A cena 109 em uma frase:** dois personagens iguais sobre fendas iguais, e a
única diferença é a contagem de raios. O da esquerda, com um raio só, **afunda** —
ele é a fotografia do mundo de antes. A terceira fenda, mais larga que o corpo,
engole os dois: **a perna não é levitação**.

**Próxima cena livre: `110`** (o `=84` não existe, de propósito).

---

## 9. Aberto, com o preço ao lado

- **`min_float_height` conservadora** (§5.2) — wave própria, e o gate novo é onde
  ela começa.
- **A metade *"um degrau íngreme não é chão"* não tem fixture de UNIDADE**, e a
  ausência é honesta: um degrau de 0,6 m ao lado do corpo é uma parede em que a
  cápsula não cabe, então o único jeito de a encostar é o solver a parar contra
  ela. Quem a segura é o `measure_push_spin`, que foi onde ela foi **descoberta**.
- **A fila do plano 08 segue:** `W-MultiJump` → `W-Ledge` → (`W-Glide`?) → o
  ajuste da entrada na água.
- ⚠️ **O `W-Ledge` acabou de ficar mais barato:** o `bevy_tnua` nomeia o sensor de
  raio único como o obstáculo do ledge grab, e ele deixou de existir aqui.

---

## 10. Mutações

**11 no total, 11 sangram.** As que valem ler:

1. ⚠️ **A redução *"fica o último"* SOBREVIVEU** aos quatro primeiros gates —
   sobre uma FENDA os dois pés de fora acham chão à MESMA distância, então
   qualquer regra de desempate dá o mesmo número. A propriedade só é observável
   sobre chão **DESIGUAL**, e o gate que nasceu disso usa um **degrau**.
2. **Sem o limite de degrau** ⇒ `measure_push_spin` (o caixote a −0,02 m).
3. **Limite = 0** ⇒ `measure_float_floor` (o leque inerte na rampa).
4. **Sem a guarda de não-encostado** na varredura do piso ⇒ ela cai na faixa de
   repouso e reporta **0,20 onde a resposta é 0,52** — o personagem **deitado na
   rampa**, imóvel por atrito, lido como *"flutua"*.

# HANDOFF DE INTEGRAÇÃO — `line/components`: A ARENA E O TUTORIAL (navegação W8) — 2026-10-03

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §16. Este documento é o que o
> integrador precisa.
>
> ⚠️⚠️ **A linha leva QUATRO waves por integrar**, todas juntas e sem reverter nada umas das outras: a W5
> ([`DESVIO_2026-10-02`](HANDOFF_INTEGRACAO_line_components_DESVIO_2026-10-02.md)), a W6
> ([`O_MUNDO_QUE_MUDA_2026-10-02`](HANDOFF_INTEGRACAO_line_components_O_MUNDO_QUE_MUDA_2026-10-02.md)), a W7
> ([`O_CUSTO_E_OS_ATALHOS_2026-10-03`](HANDOFF_INTEGRACAO_line_components_O_CUSTO_E_OS_ATALHOS_2026-10-03.md) —
> é lá que está a superfície foundational inteira, os contadores e o §3 «o que um merge pode partir») e
> esta W8. Smoke do dono APROVADO na W5, W6 e W7; o da W8 está por fazer (§5).

## §0 — O `--ff-only` deve passar limpo (medido)

- base `1ad60a1ce` (o `main`, que não andou durante a linha) · `git merge-base --is-ancestor main HEAD` ⇒
  verdade no fecho.
- Os commits da W8 (por cima de `782465553`, o fecho da W7): `2d3b1e696` a arena navega + gates + tutorial
  03 · `28b7d5484` o arnês de mutação · `723662e4f` a porta dos gates de tutorial só lê literais + plano §16 ·
  `459c79f55` fmt · e os de fecho (este handoff, o §5.1 do `CLAUDE.md`).

## §1 — Superfície de colisão da W8

A W8 **não mexe em nenhum número que soma entre linhas**: `PROJECT_SCHEMA` continua `180` (os degraus da W5
e da W7), registos `108`/`109`/`109` e o da física `44` iguais, `LIVE_SECTIONS` `47`, zero crate nova, zero
pacote externo, **zero linha em `shells/desktop`**, nenhum foundational de produto. A `collision-surface.sh`
no fecho dá o mesmo mapa da W7 (45 commits, 138 ficheiros; o único pacote novo da linha continua a ser o
`ph2d-orca` da W5).

| ficheiro | o quê |
|---|---|
| `crates/ph2d-app-components/src/vida_arena_smoke.rs` | o muro (`MURO_X/MEIO/BAIXO/CIMA`), a `NavRegion` (`REGIAO_FUNDO = −2,5`), o morcego `NavAgent`+`TopDownPlayer` (sai o `ProjectileMotion` e a const `MORCEGO_PERSEGUE`), a Salamandra `Kinematic`, `HEROI_XY` `2,45 → 2,85`; **694 / 700** LOC |
| `crates/ph2d-app-components/src/vida_arena_smoke_tests.rs` | `um_tiro_mata_um_morcego` põe o herói em `x = 0,5` (do lado do ninho — do outro a bala acertava no muro); `mod nav` |
| `crates/ph2d-app-components/src/vida_arena_nav_tests.rs` **NOVO** | 3 gates (abaixo) |
| `crates/ph2d-panel-inspector/tests/it/o_tutorial_nomeia_rotulos_que_existem.rs` | ⚠️ **porta partilhada** `literais_de` · `pintores_de`: os três gates de tutorial medem só os LITERAIS dos pintores (§2) |
| `crates/ph2d-panel-inspector/tests/it/{o_tutorial_da_vida…, o_tutorial_da_navegacao…(NOVO), main.rs}` | o 02 passa pela porta nova; o 03 é o irmão |
| `docs/Components/tutoriais/{src/03_navegacao.html, 03_navegacao.pdf}` **NOVOS** | o tutorial (gerado por `scripts/tutorial-pdf.sh`) |
| `docs/Components/ferramentas/mutacao_navegacao_w8_2026-10-03.py` **NOVO** | o arnês |
| `docs/Components/30_plano_navegacao.md` | §16 |

### ⚠️ O que um merge pode partir (só o da W8; o resto é o §3 da W7)

1. **Um gate de tutorial noutra linha** que leia os pintores pela mão (`desescapa(&sem_prosa(s))` + `contains`)
   continua a compilar, mas fica com a régua fraca — passe-o por `pintores_de`.
2. **Quem leia a arena por constantes**: `MORCEGO_PERSEGUE` saiu, `HEROI_XY` mudou, a Salamandra é
   `Kinematic`. `git grep -n 'vida_arena_smoke::'` no fecho: só `vida_smoke.rs` (a montagem e o roteiro).
3. O ficheiro da arena está a **6 linhas do tecto**: uma linha que lhe acrescente algo parte-o (cura: mover,
   nunca subir).

## §2 — O que a wave traz

O plano §16 tem tudo. Em resumo: os morcegos da arena contornam um muro e, porque a lava os fere
(`Damage::magoa`), **esperam na borda** quando o herói se refugia nela — **é o que a cena ensina, de
propósito** (a decisão do dono do plano 30 §11.1). O tutorial 03 acaba com o dono a pôr a Salamandra a
perseguir com UM gesto (*Add Component → Nav Agent*, *Target → Object → Heroi*); ela é imune ao fogo e
**atravessa** a lava. A Salamandra passou a `Kinematic` porque a semente do mover nunca rebaixa um `Static`
(o gesto pedia um passo a mais); parada, a malha conta-a igual.

**Achado pela prova de mutação, e curado na porta dos TRÊS gates de tutorial:** a M10 (renomear o botão
`Object`) sobreviveu — o `contains` lia o fonte inteiro e `Object` casava dentro de `NavAlvoModo::Objecto` e
`ArgKind::ObjectName`. ⇒ `literais_de` (o conteúdo dos `"…"`, sem `'"'`); os gates 01 e 02 continuam verdes
por ela.

**Achado pela foto:** a região andável acabava no `FUNDO` da família (`−1,19`, medido noutra cena) e via-se
chão abaixo dela onde nenhum morcego entrava ⇒ `REGIAO_FUNDO = −2,5` (o fundo do canvas na foto `1930×1040`).

## §2-bis — Três abertos de W3/W5/W6 CURADOS depois do fecho da W8 (ordem do dono, 03/10)

| defeito | cura | gate (visto VERMELHO na versão antiga) |
|---|---|---|
| o Inspector dizia *«Switched off»* de um agente desligado que um `Start` pôs a andar (W6) | `NavNow` `+ordem` `+alvo_da_ordem` (da ordem em vigor na ponte); `InspectorNavAgent` `+ordem` `+alvo_da_ordem`; a queixa lê a ordem — `Stop` ⇒ `AgentQueixa::ParadoPorAccao` (apendada), `Start` com nome ⇒ as faltas do alvo autorado não valem; a linha *«Started by an action [— after X]»* | `a_ordem_de_um_verbo_manda_na_queixa` (editor-core) · `o_painel_le_a_ordem_que_a_ponte_guarda` (pela ponte inteira) · `as_frases_da_ordem_e_da_passagem_aparecem_quando_valem` |
| a leitura viva não dizia *«a dar passagem»* (W5) | `ph2d_orca::Crowd::solve_all_why` (por agente: um semi-plano de VIZINHO exclui a velocidade pedida — `solve` inalterado); `NavNow` `+avanco` (a fracção da rapidez pelo caminho quando OUTRO cortou o pedido); `NavAgora` `+dando_passagem` (`avanco < AVANCO_DE_QUEM_DA_PASSAGEM = 0,5`, a tabela medida ao lado da const); *«Giving way · X m to go»*. ⚠️ Só pela rapidez o agente SOZINHO no labirinto acusava `58 / 369` tiques (a quina trava); com a pergunta do vizinho, `0` | `quem_da_passagem_na_porta_diz_que_da` (porta `> 0`, sozinho `= 0`) · o gate das frases |
| a porta que anda era um CÍRCULO para o desvio (W6) | `bridge/nav_desvio.rs::discos`: um corpo sólido que anda e não é agente entra no ORCA como discos ao longo da forma (`n = ⌈a/b⌉` células, cada disco o circunscrito da sua, `≤ b·√2`), cada um com a velocidade do SEU ponto (`v + ω × r`); o ALVO de alguém fica um disco só (o `ignores` nomeia um índice) | `uma_porta_comprida_a_andar_desvia_se_pela_forma` (`1,55 m` fora do caminho antes, `0,02 m` depois) |
| o defeito G — a bala SÓ-SENSOR ficava parada onde nascia (plano 28) | `ph2d-physics` `move_character_from`: um corpo só de sensores anda o que pediu (nada nele é parede); sem forma nenhuma continua parado. ⚠️ **foundational partilhado** (todo mover passa aqui) — só muda o caso que devolvia zero | `uma_hitbox_so_sensor_anda_livre_e_perfura` · `uma_bala_so_sensor_atravessa_a_parede_e_fere` (CONTROLO sólido; também a `240 m/s`) |
| a cena `=1` apertada (W3, o dono: *«não sei se intencionalmente»*) | corpos, porta e paredes escalados (paredes `0,5 → 0,24 m`, roxo `1,3 → 0,9 m`, os outros `0,7 → 0,5 m`, porta `1,0 → 0,7 m`); `LAB_RAIO_PEQUENO`/`LAB_RAIO_HEROI` novos, `RAIO_PEQUENO`/`RAIO_HEROI` ficam das cenas `=3`/`=4`; `perseguidor(…, raio_do_alvo, …)` | os gates da cena `=1` iguais; fotos antes/depois |

**Superfície nova destes três** (somar ao §1): `NavNow` +3 campos (literais fora da ponte não compilam —
só o `nav_inspector_tests`) · `NavAgora` +1 · `InspectorNavAgent` +2 (os 5 literais do repo atualizados) ·
`AgentQueixa::ParadoPorAccao` apendada (todo `match` exaustivo: só `sections/nav.rs`) · i18n +4 chaves
(`stopped_by_an_action`, `started_by_an_action`, `started_by_an_action_after_x`, `giving_way_x_m_to_go`) ·
`ph2d_orca::Crowd::solve_all_why` (pública) · `nav_smoke::perseguidor` +1 parâmetro (um par de raios) ·
`ph2d-physics/src/world/character.rs` (um braço novo do `match`) · teste novo `tests/it/bala_so_sensor.rs`.
Zero schema, zero registo, zero shell. O tutorial 03 ensina as frases novas (secções 6 e 10).

## §3 — ⏳ O que fica ABERTO

- Os passos 7–8 do tutorial (o gesto numa CÓPIA a correr) provam-se pelos gates — a foto não clica (o XTest
  é ignorado na Xwayland virtual); o smoke do dono é a 1.ª corrida com rato. As edições numa cópia somem no
  recomeço (o tutorial di-lo).
- Os abertos da W5–W7 que continuam (handoff da W7 §4): a procura ponderada `~11×` com muita lama, a
  construção inteira com áreas lenta a escala, a montagem O(malha), todos os agentes da malha que mudou
  recalculam no mesmo tique.
- **A família de navegação está COMPLETA no plano 30** (W0–W8). Plataformas (saltos) é plano próprio (§11.3).

## §4 — A prova de fecho

Tudo 1× sobre o diff acumulado, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha:

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | **`19 503 / 19 503`** no fecho da W8; **`19 507 / 19 507`** depois dos três defeitos (§2-bis), `138 s`; depois da porta e do defeito G **`19 509 / 19 510`** — o vermelho foi o gate do hash c9 (`no_std_transcendental_reaches_the_deterministic_hash`: um `sin_cos` do `std` no `nav_desvio.rs`), curado com `libm::sincos` e visto verde sozinho |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` (app-components, panel-inspector, physics-ecs, nav, navmesh, editor-core, i18n, host-desktop) | zero |
| clippy depois da porta e do G (+ `ph2d-physics`) | zero |
| clippy, de novo depois do §2-bis (orca, physics-ecs, editor-core, app-components, panel-inspector, panel-registry-init, i18n, host-desktop) | zero (a 1.ª corrida acusou `too_many_arguments` no `perseguidor`: os dois raios passaram a um par) |
| `cargo fmt --all --check` | verde (depois do commit de fmt) |
| `typos` · `cargo machete` | zero · zero |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` (32/398) | verdes |
| `censos-da-arvore-combinada.sh` | **`127 / 127`**, `12` de `12` censos |
| `doc-index.sh --check` | `20` índices em dia |
| `#[cfg(target_os` escrito/movido | nenhum |
| a máquina no fim | nada desta linha a correr; `/dev/dri` só Xwayland/plasmashell/code |

**Mutação 10 / 10** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w8_2026-10-03.py`](../ferramentas/mutacao_navegacao_w8_2026-10-03.py), quatro controlos,
grupos APPC · PANEL): morcego sem `NavAgent` · região sem paredes · morcego sem `avoid_harm` · Salamandra
estática · Salamandra sem imunidade ao fogo · muro sensor · chegada a `1 m` · lava sem queimadura · rótulo
`Avoid Harm` · rótulo `Object` (a sobrevivente da 1.ª corrida, §2).

**Auditoria (2 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **costura cena↔ponte** | o morcego que a FÁBRICA copia navega e morde | molde `receita_do_morcego` (`NavAgent`+`TopDownPlayer` registados) → `tick_factories`+`apply_births` → `PhysicsBridge::dispatch` (`drive_nav_agents` antes do `drive_topdown`) → `move_character_from` → `toques_do_mover` → `Damage` `Vanish` | `um_morcego_da_a_volta_ao_muro` (com o CONTROLO teleguiado) · `um_morcego_persegue_morde_e_some` · M1/M7 |
| **o que o tutorial afirma** | cada rótulo citado está num literal de um pintor, e o gesto do passo 7 dá o que o texto diz | `rotulos_citados_de` → `pintores_de` (literais) ∪ chave→texto; o gesto: cascata (gate da shell `escolher_um_agente_na_paleta_entrega_um_mover_que_o_ouve`) + `nav_inspector::apply_nav_edit` (alvo) → a ponte | `o_tutorial_da_navegacao_so_cita_rotulos_que_o_painel_pinta` (M9, M10) · `a_salamandra_posta_a_perseguir_atravessa_a_lava` (M4, M5) |
| **não-checado pela compilação** | a cena mostra o que o roteiro diz | 3 fotos no ecrã virtual (`target/prova/w8/`): o muro, o caminho que dobra na ponta, o contorno da lava, a região até ao fundo do canvas — a 1.ª achou a região curta | os gates acima; o clique real é o smoke |

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote novo na W8).

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                182   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                479 : 215   alvo: <= 1,0  razao 2.2x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (1500 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         487 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §5 — O smoke (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

O smoke da W8 é o tutorial: [`tutoriais/03_navegacao.pdf`](../tutoriais/03_navegacao.pdf), do princípio ao
fim (o dono monta a perseguição da Salamandra na secção 7). Errado = um morcego preso no muro ou a pisar a
lava com `Avoid Harm`; a Salamandra parada depois do nome, ou a esperar na borda da lava, ou a perder vida.

### O smoke compilado (a 2.ª corrida, colada)

Depois de TODOS os defeitos do §2-bis (o último: a porta e o G; incremental `6,5 G` + `370 M`) e de `rm -rf target/*/incremental` (29 G do `debug` + 2,3 G do `smoke`; na W8
tinham sido 15 G + 3,5 G), a 2.ª corrida de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

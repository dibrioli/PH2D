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

## §3 — ⏳ O que fica ABERTO

- Os passos 7–8 do tutorial (o gesto numa CÓPIA a correr) provam-se pelos gates — a foto não clica (o XTest
  é ignorado na Xwayland virtual); o smoke do dono é a 1.ª corrida com rato. As edições numa cópia somem no
  recomeço (o tutorial di-lo).
- Os abertos da W5–W7 continuam (handoff da W7 §4): a procura ponderada `~11×` com muita lama, a construção
  inteira com áreas lenta a escala, a montagem O(malha), todos os agentes da malha que mudou recalculam no
  mesmo tique, a porta que anda é um círculo para o desvio, o *«Switched off»* de um agente que um `Start`
  pôs a andar.
- **A família de navegação está COMPLETA no plano 30** (W0–W8). Plataformas (saltos) é plano próprio (§11.3).

## §4 — A prova de fecho

Tudo 1× sobre o diff acumulado, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha:

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | **`19 503 / 19 503`** verdes (`11 485` saltados, `136,7 s`; `load ~63` de outras linhas) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` (app-components, panel-inspector, physics-ecs, nav, navmesh, editor-core, i18n, host-desktop) | zero |
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

(preenchido no último passo)

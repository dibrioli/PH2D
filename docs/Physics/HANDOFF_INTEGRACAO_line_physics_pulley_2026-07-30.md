# HANDOFF DE INTEGRAÇÃO — `line/physics`, jornada da POLIA (2026-07-27 → 2026-07-30)

> **Para o agente INTEGRADOR.** A linha está FECHADA. Este documento é a única
> coisa que você precisa ler para integrar; o detalhe por-wave vive no tracker
> ([`HANDOFF_line_physics.md`](HANDOFF_line_physics.md)), o mapa em
> [`00_plano_waves.md`](00_plano_waves.md) e o desenho da polia em
> [`03_plano_polia.md`](03_plano_polia.md).

## §0 — GO / NO-GO

**AGUARDANDO ORDEM.** O Enio aprovou o smoke da última cena (`=64`, 2026-07-30:
*"Smoke OK"*) e mandou **seguir** — o que fechou a linha e produziu este
documento. **Ele não mandou integrar.** Integrar sem ordem explícita é violação
de protocolo (CLAUDE.md §0.7).

- Branch: **`line/physics`**, worktree em `Worktrees/line-physics`.
- Tip: **`a8e01ed32`** + o commit de docs que fecha a linha.
- Base: **`7ec917506`** — e o `main` **NÃO andou** desde o fork
  (`git rev-list --count $(git merge-base main HEAD)..main` = **0**).
- **62 commits**, **218 arquivos**, **+37.266 / −2.225**.

⚠️ Como o `main` não andou, a integração é `--ff-only` **limpa**. Se ela deixar
de ser (outra linha entrou primeiro), leia a §4 — é onde está tudo o que pode
colidir, e **os dois primeiros itens já colidiram entre estas linhas antes**.

## §1 — O que a linha entrega

Quatro frentes, na ordem em que foram construídas.

### (A) POSAR — o corpo obedece ao dedo

| wave | o quê | cena |
|---|---|---|
| W-IK | **posar arrastando a PONTA** — [ADR-0145](../architecture/decisions/0145-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md): a IK é uma **árvore de pose TRANSIENTE**, nunca uma segunda representação de joint | `=54` |
| W-FK + W-JointTools | a **cinemática DIRETA** (girar um elo em torno da própria âncora) + os cinco modos de joint numa seção própria | `=55` |

### (B) Os TIPOS que faltavam ao kit

| wave | o quê | cena |
|---|---|---|
| W-Rod | **a BARRA RÍGIDA** — o 6º tipo, e o único vínculo que este conjunto não sabia exprimir (distância FIXA, não teto) | `=56` |
| W-Wheel | **a RODA** — o cubo que gira **E** cavalga uma suspensão; o 1º tipo com dois graus de liberdade autorados | `=57` |

### (C) A POLIA — o plano 03 inteiro, W0..W6

O 7º tipo, e o primeiro vínculo do kit **cujo comprimento é um orçamento**, não
uma distância.

| wave | o quê | cena |
|---|---|---|
| W0 | as quatro correções da foto do smoke (criação pelo canvas nascia na ORIGEM · o anel de comprimento perguntava `length.is_some()` · o readout `0 / 0 N` permanente · a row Ratio **morta**) | `=58` |
| W1 | ⚠️ **o `ratio` SAIU por ser física errada** (numa corda única a tensão é uniforme ⇒ vantagem 1) e no lugar dele: **uma roldana é uma ENTIDADE com RAIO**, rota de N nós tangenciando a SUPERFÍCIE, arco no comprimento e não no Jacobiano, lado por ponto fixo, giro `ω = s/r`, "Add Wheel", a §13 | `=58` |
| W2 | **o MOTOR** (roldana dirigida = GUINCHO) e a **RUPTURA** (UM limiar; o que difere é o EIXO de cada roldana) | `=59` · `=60` |
| W3 | **a TALHA** — a roldana montada num corpo que se move; a vantagem mecânica volta **sem um número** (2 kg equilibram com 1,00 kg medido) | `=61` |
| W4 | **o TAMBOR DIFERENCIAL** — um eixo é UM nó, logo um tambor é UMA roldana com DOIS raios; vantagem `r_entra/r_sai`, o quociente de duas circunferências DESENHADAS | `=62` |
| W5 | **a COMPOSIÇÃO** — tambor e cadernal na MESMA corda: as vantagens MULTIPLICAM (1 kg segura 16 kg sem ninguém digitar um "16") | `=63` |
| W6 | **as ALÇAS** — re-colocar o eixo de uma roldana MONTADA (gesto morto e silencioso: o centro é derivado) e o **segundo diâmetro** agarrável | `=63` |
| — | **o PISO** (uma corda não pode ser mais curta que o caminho que enfia) · **a rota que não resolve PARA de segurar** · **o §10** (bias com raio + custo contra o HR-4) · **o ÍMÃ** do eixo montado · **a cadernal DIRIGIDA** (guincho de vantagem 2, medida) · **o conta-gotas de corda** (re-escolher a corda de uma roldana) | `=61` · `=63` |

### (D) W-WESTON — a talha DIFERENCIAL (ordem explícita do Enio)

O MESMO eixo atravessado **DUAS vezes**, com a cadernal ABRAÇADA entre os
contatos. Peso `R/(R−r)`, vantagem `2R/(R−r)`. Cena **`=64`**.

⚠️ **Ela derrubou DUAS notas do W5** que diziam que era caro: *"pediria uma
SEGUNDA restrição por corda"* (é **UMA** — eliminar a rotação entre os dois
contatos deixa um orçamento PESADO, o tipo que a rota já soma) e *"concêntricas
são recusadas pela rota"* (só para pares **CONSECUTIVOS**, e num par de Weston a
cadernal está no meio). O ramo depois do retorno leva peso **ZERO** — a corda
MORTA, dita por aritmética.

⚠️ **DOIS bugs PRÉ-EXISTENTES achados pelos gates dela**, e os dois entram no
`main` corrigidos:

1. **O teto do guincho era cego ao PESO da corda** — a taxa içada caía a **38% no
   peso 8**; *um guarda que clipa o regime virou um limite*. Latente desde o W4.
2. **Um rewind replayava SEM AS CORDAS** — o `rebuild_from_rest` trocava o
   `PhysicsWorld` sem reinstalar a tabela de polias, e o replay roda no MESMO
   chamado. Calado porque `target == 0` (o Reset) replaya zero passos.

## §2 — Números que se CONTAM (confira, não copie)

| pin | no fork | no tip | por quê |
|---|---|---|---|
| `PROJECT_SCHEMA` | **37** | **45** | oito bumps, todos por campo APENDADO (postcard é posicional). O 45 chegou no `76a6cd1e8` (W4-B) |
| registro `ph2d-physics-ecs` | **21** | **23** | `PulleyWheel` (W1) e `WestonAxle` (marcador, presença = booleano) |
| `JointKind` | 5 variants | **8** | `Rod` · `Wheel` · `Pulley` **APENDADOS** — o discriminante é posicional |
| `physics_ecs_c9` | — | **`7cb7728d…`, 96 corpos** | debug ≡ release; **byte-idêntico** desde o W6 (nenhuma das últimas seis waves alcança o solver) |
| ids de gizmo numéricos | ≤**968** | **969, 970, 971** | `GIZMO_WHEEL_CENTRE` · `GIZMO_WHEEL_RIM` · `GIZMO_WHEEL_RIM_OUT`. **Próximo livre: 972** |
| ADR | — | **0145** | o único novo |

⚠️ **O `PROJECT_SCHEMA` é o item nº 1 do seu checklist, e esta linha já perdeu
essa disputa DUAS vezes** — com a `line/FLIP` em 25/07 (o 30) e outra vez em
27/07 (o 32/33/34). Se outra linha bumpar na mesma janela, **o valor certo não
está em nenhum dos dois lados do conflito — CONTE**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]) e reescreva os
parágrafos `v3X`/`v4X` do `project.rs` na ordem em que os bumps de fato
empilharam. O antídoto é uma linha de shell, não uma lembrança:

```
grep -n "PROJECT_SCHEMA: u32" shells/desktop/src/project.rs
```

⚠️ **Os três ids de gizmo NOVOS são numéricos e sequenciais** — a única
superfície desta jornada que colide por VALOR e não por nome. Se outra linha
reivindicou 969-971, renumere OS DESTA (os nomes diferem, então o git funde
limpo e **nada avisa**); o gate `node_id_collisions` é quem pega.

⚠️ **Nenhum contrato congelado (CLAUDE.md §6) foi tocado** — conferido por grep,
não por auto-relato.

## §3 — O registro de smoke, honesto

| cena | o quê | estado |
|---|---|---|
| `=54` | W-IK | rodada, defeitos corrigidos; **sem nota de aprovação final** |
| `=55` | W-FK | rodada, defeitos corrigidos; **sem nota de aprovação final** |
| `=56` `=57` | W-Rod · W-Wheel | rodadas, defeitos corrigidos; **sem nota de aprovação final** |
| `=58`..`=60` | W-Pulley W0..W2 | **o smoke DELAS é a origem do plano 03** (oito pontos do Enio ⇒ o redesenho inteiro) |
| `=61` | a TALHA | **APROVADA** — ⚠️ e **DEPOIS** dela o ÍMÃ e o conta-gotas mexeram nesta cena |
| `=62` | o TAMBOR | rodada (o enquadramento saiu dela); **sem nota de aprovação final** |
| `=63` | a COMPOSIÇÃO | **APROVADA** — ⚠️ e **DEPOIS** dela o W6/PISO/degeneração mexeram nesta cena |
| `=64` | a WESTON | **APROVADA (2026-07-30)** |

⚠️ **Isto não é uma lista de pendências disfarçada, e a distinção entre as duas
colunas importa.** Toda cena rodou e todo defeito reportado foi fechado; o que
falta em algumas é uma nota de *aprovação final*, porque a jornada seguiu direto
para a wave seguinte — e em `=61`/`=63` a aprovação **é anterior** às waves que
voltaram a tocar aquela cena, o que não é a mesma coisa que "aprovada como está
hoje". **Se o Enio quiser re-smokar antes de integrar, a lista honesta é
`=54` `=55` `=56` `=57` `=61` `=62` `=63`**; se não, o que entra é o que os
gates provam, e a `=64` — a única com aprovação sobre o tip — é a que exercita a
rota inteira.

Rodar (todas `--release`):

```
env PH2D_PHYSICS_SMOKE=<54..64> cargo run -p ph2d-host-desktop --release
```

## §4 — O que pode colidir

Em ordem de probabilidade.

1. **`PROJECT_SCHEMA`** — §2. Já colidiu duas vezes com a `line/FLIP`.
2. **Os três ids de gizmo (969-971)** — colidem por VALOR, e o git funde limpo.
3. **`shells/desktop/src/physics_smoke.rs`** — a tabela de despacho de cenas
   ganhou onze linhas (`54`..`64`). Toda linha que acrescenta cena toca este
   `match`; o merge é textual e o conflito é óbvio, mas **o número da cena se
   CONTA como o schema**.
4. **`crates/ph2d-editor-core/src/ids/inspector_joint.rs`** — +100 linhas de ids
   de painel (nomes, não valores ⇒ colide por texto, não por semântica).
5. **`crates/ph2d-physics-ecs/Cargo.toml`** — a aresta `libm = "=0.2.16"`
   (mesmo pin de `ph2d-ecs`/`ph2d-physics`/`editor-core`/`wet-paint`). O
   `Cargo.lock` ganha só a aresta.
6. **`docs/Physics/00_plano_waves.md`** — lista compartilhada: só ADICIONE
   ([[feedback_a_shared_list_is_merged_against_todays_main]]).

**Nenhuma crate nova.** Nenhum `Cargo.toml` fora o de cima.

## §5 — O gate de fechamento

Rodado **1× sobre o diff acumulado**, na worktree:

- `cargo test` nas crates tocadas: **173 blocos ok** (`ph2d-physics`,
  `ph2d-physics-ecs`, `ph2d-panel-inspector`, `ph2d-editor-core`) + **55** na
  shell.
- `cargo clippy --all-targets` nas crates tocadas + shell: **0 warnings**.
- `cargo fmt --all`: limpo.
- **`architecture_workspace_file_loc_cap`** (crates, 700) e
  **`shells/desktop/tests/file_loc_caps.rs`** (shell, 600): verdes — ⚠️ os dois,
  porque o segundo **não roda** num `cargo test -p` filtrado e já foi
  vermelho-latente nesta linha três vezes.
- `architecture_panel_wiring_parity` · `node_id_collisions` ·
  `arch_safe_clamp_only` · `no_tofu_glyphs` · `handle_scenes_start_paused`:
  verdes.
- `physics_ecs_c9`: **`7cb7728d…`, 96 corpos**, idêntico em debug e release.

⚠️ **O que o `ship.sh` acrescenta e eu não rodei:** `machete`, `deny`, `audit`,
`typos` e a matriz de 3 OSes. A aresta `libm` é a única dep nova e é um pin já
presente no workspace, então `deny`/`audit` não deveriam ter o que dizer — mas
*"não deveria"* não é verde.

## §6 — Aberto, nomeado, NÃO construído

Nada aqui bloqueia a integração.

- **`axle_pair` recusa três ou mais contatos num eixo** — dois diferenciais em
  série é topologia própria.
- **O eixo composto da Weston é cenário na v1** — montá-lo num corpo que se move
  quer o Jacobiano do segundo contato no ledger (hoje é um `max` entre os dois
  contatos, não uma soma vetorial).
- **`radius_out` e o marcador `WestonAxle` são duas formas de dizer "eixo
  composto"** e um dia querem ser um enum.
- **O readout da §12 de uma corda degenerada mostra `0 N` em âmbar sem dizer por
  quê** — o texto quer i18n e canal próprio.
- **O salto balístico do contrapeso comum** — física honesta, fica na tela.
- **A nota do `0 N` no plano 02 aponta para o lugar errado** (o readout vive no
  overlay, e ele já foi corrigido no W0/W2-B) — quem souber qual das duas ela
  queria dizer, reescreve ou apaga.

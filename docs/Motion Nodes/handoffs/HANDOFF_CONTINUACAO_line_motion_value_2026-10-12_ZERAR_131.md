# CONTINUAÇÃO — `line/motion-value`: ZERAR, o último bloco (doc 131) — itens 2, 3, C, D, E FECHADOS (12/10)

> Ordem do Enio: *«vamos até zerar as pendências»*; itens 2, 3 e 4 do [B6](HANDOFF_CONTINUACAO_line_motion_value_2026-10-12_ZERAR_FAMILIA_B_B6.md)
> num ciclo (§0.10). Plano, leis, tabelas e recusas: [doc 131](../131_zerar_o_ultimo_bloco.md) (§7 = as medições).
> **Este ficheiro substitui o B6 como fila viva.** Regras R1–R9 do `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, **rebaseada sobre o `main` `9ea7821383`** no início (59 commits; só a
  memória conflitou — a família da régua unida e recontada). Commits desta janela: `bded383028` … `7baf43dcab`
  (`git log 9ea7821383..HEAD`). **Nada integrado nem enviado.**
- Os agentes de worktree (C.7, C.10, D.14) entraram por cherry-pick; as worktrees deles foram removidas.

## O que fechou (o número decisivo; o resto no doc 131 §7)

| item | lei / cura | medido |
|---|---|---|
| **2** `=70` | a cura `1a71c8f800` viva; gate do GIRO `as_estrelas_da_70_giram_em_volta_do_centro_delas`; roteiro «DE PLAY» | −2,47…−2,76 voltas/10 s |
| **3** `=129` | não era deriva nem o `transform`: o `GpuCook` guardava `tap_streams`/`debug_streams` ANTES do passe de separação; `a_saida_e_o_que_ela_desenha` (`saidas.rs`) | `2,74e-3 → 1,19e-7`, barra `1e-3` intacta |
| **D.13** cor | `grupo_de_cor`: um canal solto no MESMO canal liga o grupo, um desfazer | 1 → 4 canais por gesto |
| **D.13** dano | `vida → On Change (Fall)` = a lei do `Damage` ⇒ um objecto basta; cena `=136` | gate da sequência |
| **D.14** | `StreamOp::WriteNamed` (o irmão de escrita do `Project`): o `Drive` Custom vai à placa; `RECUSA_COLISOR` só para o nome que nenhum `StreamOp` nomeia; cena `=137` | 16 384 discos: CPU `7,02` × placa `1,46 ms` |
| **C.7** | ⛔ RECUSADOS: scan por subgrupos e cache de ligações (código em `104880a258`); a hipótese «barreiras do scan = piso» REFUTADA | cadeia 16·1024 `12,36` × `11,53 ms` |
| **C.8** | fecha por lei: o «piso» `~0,55` era o DESENHO + espera; o prefixo (`source.shape`) custa `0,001 ms` | cozimento híbrido `0,15 ms` |
| **C.9** | `opt-level = 2` na `ph2d-contact` em `dev` (`Cargo.toml`) | testes de contacto `7,05 → 3,96 s` |
| **C.10** | a vala entre células é `A + L + 1` (era `2·max + 1`) + saltar alvos vazios; `Celulas::escolhe(Variante)` | 4 096 formas `5,27 → 2,25 ms`, 4 → 2 páginas |
| **C.11** fita | gravar custava a HISTÓRIA (`35,07 → 0,027 ms`); a régua mentia 20× (dhat: declara = retém ±3 %); `FITA_BYTES` = 128 MiB REAIS | 50–73 B/mudança (era ~640) |
| **C.11** OKLCH/Bezier | fecha por lei (§0.0: a CPU é reserva; a placa custa `+0,05`/`+0,03 ms` a 32k — docs 127/126) | — |
| **C.12** | fecha por lei: `+1,18 ms` UMA vez por troca, o quadro fica a 8,6 % do orçamento | `=135` smoke |
| **E.15** | os 33 docs do Motion > 40 KB cortados por `doc-split.py` (sha256), cada um com `⛔ Recusas MEDIDAS`; arquivo em `docs/archive/motion-nodes-2026-10-12/` | 33 → 0 acima de 40 KB |

**Mutações (9/9 VERMELHAS)** desta janela: tomada e depuração da saída · o vórtice no mundo · força 0 · `grupo_de_cor`
nulo · a troca de canal · `direction` Both · o nome errado na `=137` · o cursor da fita. Os agentes provaram as
deles: D.14 5/5, C.10 2/2, C.11 2/2.

## Aberto (nomeado, com o porquê)

- **C.10, degrau de FORA** (Glow: alcance ≈ leitura `16`/`17`): a vala quase não encolhe (`35 → 34 px`).
- **C.10 (a) e (c)**: a sensibilidade da silhueta do `SpriteRenderer` (espera uma cena de sprites com filtro) e o U2
  (a unidade é do PROJECTO, não deste nó) — fora do Motion.
- **C.7, a pista**: as nuvens ganham `1,2–1,6×` com outro codegen do kernel de um grupo (o `corrige`, arrays de 9) —
  não medido; não é cura.
- **Fita**: 300 entradas a mudar em todo tique = 2,5 min de história a 128 MiB; para lá dela o scrub repete a base.
- Por lei (de antes): fora da camada isolada `Everything`/`Scene` ficam na CPU; uma IMAGEM num grupo fica na CPU.

## Smoke ao dono (fotografado)

`=70` (as estrelas giram) · `=136` (vida e dano; o gesto da cor) · `=137` (o colisor pelo nome). Fotos e passos no
relatório da janela; números de velocidade = as tabelas acima.
⚠️ **1.º report do dono (12/10)** — «smoke 2: tint não tem R» · «smoke 3: em drive não há colisor». Curado em
`8ee7620197`: o menu do fio solto lê os canais de um seletor de cor «Color R/G/B/A» (o `r` lia «Color», `g b a` a
chave crua) e um canal segue o seletor (no `Solid`, `g2 b2 a2` apareciam sem leitor); o nome da coluna do Drive
está no cartão (linha `Column`), legível só com o grafo APROXIMADO — os roteiros `=136`/`=137` dizem-no. O gate do
gesto tira o param da entrada «Color R» do MENU. ✅ `=137` OK no 2.º smoke.
⚠️ **2.º report** — «os 4 fios ligaram com só o R; as estrelas continuam brancas». Medido no app vivo (o intent na
fila do painel, fotografado): do `cor` a placa recebe verde; do `R` do cartão `vida` (Kind = Number) recebe
`(1, 1, 1, 1)` — o branco do report. Curado em `62e22985ce`: `saida_viva(kind, porta)` na crate do Input e a recusa com
aviso nas duas portas onde um fio nasce (`DriveParam`, `Connect`); o roteiro nomeia o cartão (o `cor` está SOZINHO à
direita). ⏳ Re-smoke `=136` passo 3.
⚠️ **3.º report** — a foto do aviso CORTADO («…only carry a colour on a…»): o aviso passa a 47 caracteres. Aberto
para a `line/UIUX`: o aviso (toast) corta sem reticências de sentido nem quebra de linha — outros avisos longos do
Motion (`can_t_drive_that_output_is_a_per_element_stream` + a cura) perdem a cura do mesmo modo.

## Fecho desta janela

Portão §1.5.9 (`verificador`, carga 16–31 de outras linhas): `nextest-impacted` **22 662/22 662** · `check
--workspace --all-targets` warnings=deny ✓ · clippy `-D warnings` (forma do `ship.sh`) ✓ depois de 1 cura (needless
borrow na sonda C) · fmt ✓ depois de `cargo fmt` (9 ficheiros) · typos ✓ · machete ✓ · `doc-index --check` ✓ ·
conferência 371 ✓ · arquitetura 111 ✓. GPU (`PH2D_GPU=1`): Motion 45/45 (todos os «placa», sem sondas) ·
`ph2d-gpu-cook --test it` 255/255 · `ph2d-fx-cells` 4/4 · `ph2d-shape-gpu` 15/15 · `ph2d-render` 155 + 1 flake de
carga (`the_pointwise_op_costs_much_less_than_a_blur`, 3/3 sozinho a load 28) + 4 `smoke_fixture_renderable::w2..w5`
que são `unimplemented!()` sob `#[ignore]` (goldens que não existem — não são gates). Mutações 9/9 VERMELHAS.
Fotos (`fotografa_cena.sh`, 1600×972): `=70`, `=136`, `=137` (+ `PH2D_MOTION_ROUTE_LOG=1`: `[motion-route] device:
HIBRIDO`). Worktrees dos agentes removidas; `target/*/incremental` reclamado.

binário do smoke (`cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida):
`Finished smoke profile [optimized] target(s) in 0.25s` (depois de `5ae5c93662`) — zero `Compiling`.

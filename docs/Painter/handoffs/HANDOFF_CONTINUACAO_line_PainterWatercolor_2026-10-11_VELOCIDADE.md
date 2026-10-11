# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, a onda da VELOCIDADE (2026-10-11, prompt 19)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../IntegracaoMultiAgente/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`: use
  `cargo check -p ph2d-tool-painter --test it`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`. Cura: (a) `opt-level = 2`; (b) fixture menor que
  ainda morda (prove com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`. ⛔ Lei nunca.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Base `960883e73` (= `main` `8f20c60de` + docs). Commits: `0ccadbfb3` (docs partidos) · `545254755` (perf + Tooth) ·
o deste handoff. Sem push.

## §2 — Antes × depois (medido nesta worktree; load ao lado, nunca esperei calma)

| | antes | depois |
|---|---|---|
| suíte `ph2d-tool-painter` (1574) | **377 s**, 9 TIMEOUT + 1 FAIL (load 70–130) | **174 s**, 1574/1574 (load 3–20) |
| `o_traco_nao_muda_de_tom_ao_secar` (A/B, mín. de 2) | 159 s | 23 s |
| `sem_celulas_soltas` | 252 s | 49 s |
| `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` | 28 s | 4 s |
| heavy edge · achatamento do S · opções ao byte · camadas · mistura nunca mais clara | TIMEOUT (> 180/360 s) | 28–125 s |
| `cada_preset_parece_a_sua_midia` (sozinho, mín. de 2) | 57 s (o2, em série) | 46 s |
| `a_largura_e_a_prometida` (sozinho) | 55 s (o2, em série) | 51 s |
| compilar a frio · remontar após 1 edição | 50 s · 3,9 / 3,3 s | 49 s · 3,5 / 3,2 s |
| gerador da fábrica (`--profile smoke`) | 545 s | 545 s (a variante paralela 577 s, recusada) |
| perfil do agente (10 sessões) | paral. 1,10 · test:check 2,4× · Edit 29 % · contexto 422 k | 1,23 · 3,2× · 75 % · 266 k |

**O que mudou:** (1) `[profile.dev.package.ph2d-wet-paint]` e `ph2d-pigment` `opt-level = 2` (fim da lista,
tabela no comentário). Os portões ao byte (`a_miniatura…`, `com_as_opcoes_desligadas…`, `smooth_edges_no_zero…`,
`os_outros_meios_nao_mudam`) passam a o0 e a o2 — nenhum bit mudou. Todas as dependências externas a o2: 0 %
(41 × 41 s; 49 × 48 s) — recusado. (2) Os dois portões da fábrica correm as cenas de uma vez (asserções iguais;
o controlo das mídias uma vez por LARGURA: 15 cenas em vez de 16). (3) `.config/nextest.toml`: o override de
6 × 60 s ganha os dois portões da fábrica, com a medida ao lado.

## §3 — Achado: o Tooth estava VERMELHO na base (e no `main`)

`tests::watercolor_papel_nos_vales::o_tooth_morde_no_wet_paint` falhava determinístico (631 / 552 / 592 com o
relógio fixo). Mecanismo: a largura prometida (W2) alarga o traço a raio 14 e o próprio corpo cai fora da faixa
`56..72` que o teste lê como «halo». Cura: os dois testes do Tooth pintam com `set_wet_largura_crua(1.0)` (a regra
dos 21 portões de física da W2) e o relógio fixo → 0 / 23 / 47 e 0 → 34. ⚠️ **Por que ninguém viu:** a bateria
da linha corria `--lib wetpaint` e este teste mora em `tool::paint::tests::*`. A bateria do fecho passa a ser a
lib INTEIRA do `ph2d-tool-painter`.

## §4 — Ficou acima de 10 s (o R9 por fazer — a próxima janela)

Sob a suíte, depois: camadas 159 · poças da aquarela `a_separacao_das_pocas…` 151 · achatamento do S 140 ·
`a_chave_muda_com_cada_parametro…` 113 · largura 105 · mídias 98 · heavy edge 93 · opções ao byte 91 ·
`sem_celulas_soltas` 67 · `a_borda_de_um_traco_sobre_tinta…` 41 · mistura nunca mais clara 31 · e 18 entre
10 e 28 s (previews, gotas, mistura, aquarela; lista em `depois_nextest` da janela, refaça-a com
`--status-level pass`). Já são lei com a o2; a cura agora é (b), uma a uma, com mutação.

- **Mídias — medido:** os 1 320 quadros depois dos 8 s da cena são ~87 % do custo (com o botão Dry canvas no
  lugar deles: 41 → 5,5 s). ⛔ **Mas o Dry muda as métricas** (Liner cobertura 0,73; o Express Oil reprova: o
  controlo Round dá borda 0,745 contra a foto 0,796 — a métrica deixa de distinguir) — a régua foi calibrada com
  a secagem natural. Próximo passo: medir o quadro a partir do qual a tela já não muda (ponto fixo) e parar aí,
  provando com M2/M8 que ainda morde; ou recalibrar a barra com o Dry (pergunta de régua, não de velocidade).
- **Largura:** a parede é o traço a raio 300 (tela 1000 × 1880, 480 quadros). Ideia por medir: a faixa da altura
  do traço; menos quadros quando a água já assentou.

## §5 — ⛔ Recusas MEDIDAS (não refaça)

| proposta | medida | veredicto |
|---|---|---|
| gerador: fila (preset, raio) paralela, do maior para o menor | 545 → 577 s; miniaturas iguais ao byte | o trabalho já enche os 16 núcleos da porta |
| todas as dependências externas a `opt-level = 2` | 41 × 41 s; 49 × 48 s | nada para ganhar |
| mídias com o botão Dry canvas no lugar dos 22 s | 41 → 5,5 s, mas reprova (Express Oil) | muda a régua |

## §6 — Fecho

**Mutações: 3 de 3 VERMELHAS** (agente `mutacao`, em série; cada portão verde antes; restauro por cópia +
`touch`, `cmp` igual, `git diff HEAD -- crates` vazio):

| mutação | sítio | portão | resultado |
|---|---|---|---|
| MT `dente` devolve `h` | `ph2d-wet-paint/src/paper.rs:295` | `o_tooth_morde_no_wet_paint` | 🔴 [23, 23, 23] |
| M1 sem a lei da largura | `tool/paint/dab_spec.rs:~57` | `a_largura_e_a_prometida` | 🔴 Round raio 26 → 30 px (pedido 52), 300 → 315 |
| M8 o grão ignorado | `wetpaint/dab_route.rs:~185` | `cada_preset_parece_a_sua_midia` | 🔴 Graphite textura 14,7 fora de 9,1; Charcoal cobertura 0,29 |

A fábrica ao byte: o gerador regravou os 12 presets (`PH2D_REGERA_PRESETS=1`) e `git status assets` ficou vazio;
as 12 FNV iguais às do gerador original.

**Bateria (agente `verificador`, load 10–106 o tempo todo):** cerca ✓ · `ph2d-wet-paint` 148/148 · `ph2d-tool-painter`
1573/1574 (a falha: `a_wet_move_costs…`, membro conhecido de FLAKES_DE_CARGA, 2/3 sozinho a load ~77) · censos 134 ·
237 · 17 ✓ · `ph2d-app-painter` 103/103 · fmt ✓ · clippy `-D warnings` ✓ depois de tirar dois `&` em
`midias_de_fabrica.rs` · tecto 700 ✓ (418 · 164 · 288) · `nextest-impacted.sh` (base `8f20c60de`) 17 636/17 641 —
as 5 passam sozinhas: as poças da aquarela (TIMEOUT a 180 s sob carga; 25 s sozinha), três da Luz 2D (sem
alterações nesta linha) e:

- ⚠️ **ABERTO — `ph2d-wet-paint::it paper_rows::the_paper_bake_by_rows_is_the_serial_bake`** falhou uma vez
  («o bake por linhas mudou o papel», `paper_rows.rs:31`) e passou 3/3 sozinha (load 24). É um portão de BITS
  serial × paralelo: falhar sob carga cheira a corrida no bake paralelo, não a relógio. Não visto antes da o2 —
  a o2 pode só ter mudado a janela da corrida. **Primeira tarefa da próxima janela:** repetir sob carga (N × com
  `stress`) a o0 e a o2, e ler `bake_paper_rows` (`Rows::Parallel`) contra as três condições do `par.rs`.

## §7 — O que NÃO foi tocado (a fila espera; §8 do handoff PRESETS_MIDIAS)

O relevo que some ao trocar de pincel · o óleo longe do Rebelle · a tinta/caneta (dono) · o Staining com K–M.

## §8 — Docs partidos (`doc-split.py`, sha256)

`NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md` 136 → 14 KB (índice; §4b em `05a`–`05d`, por data; entrada
nova vai para o `05d`). `BUGS_painter.md` 94 → 36 KB (os ABERTOS + #24/#25/#29 por ordem do dono; a tabela dos
fechados em `BUGS_painter_fechados_1_a_40.md` e `…_41_a_46.md`, este com o post-mortem do #15).

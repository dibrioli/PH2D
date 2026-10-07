# HANDOFF DE INTEGRAÇÃO — `line/components`: A LAMA (navegação W18) — 2026-10-06

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §27 (os kill-criteria §27.1–§27.3, a
> cena §27.4, as rodadas §27.5–§27.6, os vereditos §27.7, as recusas §27.8, o aberto §27.10). A linha ainda
> leva a W15, a W16 e a W17 inteiras, por integrar: os handoffs delas,
> [`O_TIQUE_DEPOIS_DA_PORTA`](HANDOFF_INTEGRACAO_line_components_O_TIQUE_DEPOIS_DA_PORTA_2026-10-05.md),
> [`O_DESENHO_E_O_CORPO`](HANDOFF_INTEGRACAO_line_components_O_DESENHO_E_O_CORPO_2026-10-05.md) e
> [`OS_TRES_ABERTOS_MEDIDOS`](HANDOFF_INTEGRACAO_line_components_OS_TRES_ABERTOS_MEDIDOS_2026-10-06.md),
> continuam válidos para a parte delas — este acrescenta a W18 por cima.

## §0 — O `--ff-only` deve passar limpo

- Base `main` `a46c4c200` (o `main` não andou; o `git rebase main` do início foi vazio). NADA integrado desde
  `5d596eaaf`: W15, W16, W17 e a W18 — `359dc42a3` (o plano §27, antes do código) · `702819e48` (o estado
  medido, com as alavancas) · `da21b95c6` (as alavancas saem) · `d8ef0527a` (a cena, o tutorial, o plano) ·
  `ea306c7ca` (o fecho) — e o deste handoff.
- Nenhum ADR novo (o máximo continua `0180`, da W16).

## §1 — Superfície de colisão (a W18)

| ficheiro | o quê |
|---|---|
| `ph2d-app-components/src/nav_smoke_lama.rs` + `_tests.rs` | NOVOS: a cena `=5` e os gates dela |
| `ph2d-app-components/src/nav_smoke.rs` (+ `_tests.rs`), `lib.rs` | o roteador: braço `5`, `CENAS = 5`, `Montada::lama`, `Montada::secao_do_roteiro`; `pub mod nav_smoke_lama` |
| `shells/desktop/src/components_scenes_suplentes.rs` | UMA linha trocada (`let sec = montada.secao_do_roteiro();`) — zero linhas a mais |
| `ph2d-nav/src/agent.rs` | `AgentRuntime::custos_do_caminho`, `Vez::custos`, o 5.º motivo de replaneio |
| `ph2d-physics-ecs/src/bridge/nav.rs` | a assinatura dos custos passa ao `Vez`; o `struct Vez` da ponte MUDOU-SE para `nav_fila.rs` (`694 → 684` linhas) |
| `ph2d-physics-ecs/src/bridge/nav_fila.rs` | `entradas_das_procuras` devolve também a assinatura dos custos e atalhos; o `Vez` da ponte |
| `ph2d-physics-ecs/tests/it/nav_custo.rs` | o gate do custo ao vivo |
| `docs/Components/30_plano_navegacao.md` · `tutoriais/src/03_navegacao.html` + `.pdf` · `CLAUDE.md` §5 | o plano §27, a página da lama, o roteador |

Zero crate, zero pacote no `Cargo.lock`, zero i18n, zero componente, `PROJECT_SCHEMA` intocado. O
`AgentRuntime` ganhou um campo — ele vai no anel (`ControllerMemory::nav`), não no ficheiro do projecto.

## §2 — O que a W18 traz

| item | veredito | o quê |
|---|---|---|
| **S** a cena da lama (`PH2D_NAV_SMOKE=5`) | ✓ | duas pistas iguais, `Light Mud` (`Cost 2`) e `Heavy Mud` (`Cost 10`, escolhida, a secção `Nav Cost Area` aberta); três corredores por pista: a leve atravessa-se, a pesada contorna-se. Gates pela cena real com o CONTROLO (as duas a `1`: cruzam as duas) |
| ⭐ **o `Cost` ao vivo** (achado pela cena) | ✓ curado | mexer no `Cost` com o agente a ANDAR era um controlo morto (`10 → 2`: `0` procuras novas; só `→ 1` mudava algo, a lama a sair da malha). O 5.º motivo de replaneio: a assinatura dos custos e dos atalhos mudou desde que o caminho foi planeado — paga do orçamento como as outras procuras; um `Nav Link` posto/tirado entra pela mesma porta |
| **A** o tecto do caminho crítico | ✗ recusa medida | T1 e T3 cumprem o crítico (`20` · `40 mil`) e REPROVAM a vivacidade (a procura nunca acaba numa malha que muda de 2 em 2 tiques — a aritmética: a fatia tem de ser `≥ W/K`); T2 é vivo e deixa o crítico igual (`80 mil`). Sem tecto, o A4 fica recusa |
| **C** menos nós na ponderada | ✗ recusa medida | a dominância de hoje JÁ está no tecto do que frentes anteriores cortam (o tecto amostrado: `0,04 %`); o heurístico toca `6–16 %` dos nós; a fase geral `~10 %` do tempo |

## §3 — ⏳ O que fica ABERTO

- **O crítico do tique** chega a `orçamento · 2^k` (`80 mil` a `10` e `50` agentes na cena de stress) — o preço
  da vivacidade numa malha que nunca pára; não se vê no relógio (o pior tique igual em todas as versões).
  Fechado como escolha medida, não como dívida.
- **Quem persegue sem o alvo à vista** na lama cerrada (máx `30` tiques a `200` agentes): o A4 só cabe com um
  tecto que a vivacidade não deixa.
- **A procura ponderada** está no tecto deste desenho; metade do tempo pediria OUTRA procura.
- (de antes, sem mudança) a versão web: aprovada pelo dono, ESTACIONADA.

## §4 — A prova de fecho

- **As medições** (a régua do dono de 05/10: as versões no MESMO processo, intercaladas, o mínimo, perfil
  `smoke`, loadavg anotado), `target/prova/w18/`: `medir_replaneio_w18.txt` (A, load `8–17`: valem as colunas
  de trabalho e de espera), `diag_nos.txt` · `diag_nos_c4.txt` · `diag_nos_tecto.txt` ·
  `diag_nos_tempo_geral.txt` (C), as fotos `foto/cena5_{a_meio,fim}.png`.
- **Gates novos:** `nav_smoke_lama::tests::{a_leve_atravessa_se_a_pesada_contorna_se,
  com_o_cost_em_2_os_da_direita_cortam_pela_lama, a_cena_tem_as_pecas_que_o_roteiro_nomeia}` ·
  `nav_smoke::tests::o_cenas_conta_os_niveis_do_roteador` · `nav_custo::mexer_no_custo_refaz_o_caminho_de_quem_anda`.
- **Mutação** ([`mutacao_navegacao_w18_2026-10-06.py`](../ferramentas/mutacao_navegacao_w18_2026-10-06.py), o
  motor da W15/W16): **`10 / 10`** sangram, `0` defeitos de arnês, checksums iguais —
  L1–L6 (a cena: o custo, a área, o roteador, a secção aberta, `CENAS`, o desenho) e C1–C4 (o 5.º motivo, o
  registo no caminho instalado, a assinatura, a ponte). `L5` (`CENAS = 4`) só sangra pelo gate novo do roteador.
- **Gate batched** (sobre o merge-base, W15–W18 juntas): `nextest-impacted` **`15 782 / 15 783`** (`92,8 s`, `0`
  lentos) — a única falha foi `o_tutorial_da_navegacao_so_cita_rotulos_que_o_painel_pinta` (o tutorial marcava
  `Cost 2` e os nomes das lamas como rótulos de tela); curada em `ea306c7ca` e o gate re-corrido verde ·
  clippy `--workspace --all-targets -D warnings` limpo · fmt limpo nos ficheiros da W18 —
  `target/prova/w18/gate_{nextest,clippy}.txt`. Load `5` antes e `18–20` durante.

## §5 — O smoke

⏳ **Smoke do dono: por fazer** (a cena `=5`, fotografada antes: `target/prova/w18/foto/cena5_a_meio.png` — à
esquerda os três DENTRO da lama clara a subir a direito, à direita os três a contornar pela passagem;
`cena5_fim.png` — os seis nas bandeiras, nenhum na lama escura). O roteiro é a página 10 do
`03_navegacao.pdf`:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. Só olhar: os três vermelhos da esquerda sobem a direito pela faixa castanha CLARA (`Light Mud`, `Cost 2`); os
   três da direita vão à passagem junto à parede e contornam a faixa ESCURA (`Heavy Mud`, `Cost 10`).
3. No Inspector (a `Heavy Mud` vem escolhida, a secção `Nav Cost Area` aberta): fechar e correr de novo, `Espaço`
   logo no início, `Cost` = `2`, `Espaço` — os da direita viram e cortam pela lama escura (o que nasceu junto à
   passagem pode seguir a volta).
4. Deu errado se: algum da direita pisa a escura com `Cost 10`; os da esquerda contornam a clara; ou, com `Cost 2`,
   nenhum da direita muda de caminho.

O binário compilado (a 2.ª corrida, colada), depois de `rm -rf target/*/incremental` (`3,6 G` do `debug`, `1,6 G`
do `smoke`):

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.09/passo   alvo: >= 1,5  (6% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                292   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                919 : 254   alvo: <= 1,0  razao 3.6x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1724 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         445 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

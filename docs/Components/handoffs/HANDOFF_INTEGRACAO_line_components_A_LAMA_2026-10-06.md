# HANDOFF DE INTEGRAÇÃO — `line/components`: A LAMA E A COTA (navegação W18 + W19) — 2026-10-06/07

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §27 (W18: os kill-criteria §27.1–§27.3, a
> cena §27.4, as rodadas §27.5–§27.6, os vereditos §27.7, as recusas §27.8, o aberto §27.10) e §28 (W19: os
> achados da auditoria §28.1–§28.5, os resultados §28.6–§28.8, o aberto §28.9). A linha ainda
> leva a W15, a W16 e a W17 inteiras, por integrar: os handoffs delas,
> [`O_TIQUE_DEPOIS_DA_PORTA`](HANDOFF_INTEGRACAO_line_components_O_TIQUE_DEPOIS_DA_PORTA_2026-10-05.md),
> [`O_DESENHO_E_O_CORPO`](HANDOFF_INTEGRACAO_line_components_O_DESENHO_E_O_CORPO_2026-10-05.md) e
> [`OS_TRES_ABERTOS_MEDIDOS`](HANDOFF_INTEGRACAO_line_components_OS_TRES_ABERTOS_MEDIDOS_2026-10-06.md),
> continuam válidos para a parte delas — este acrescenta a W18 e a W19 por cima.

## §0 — O `--ff-only` deve passar limpo

- Base `main` `a46c4c200` (o `main` não andou; o `git rebase main` do início foi vazio). NADA integrado desde
  `5d596eaaf`: W15, W16, W17 e a W18 — `359dc42a3` (o plano §27, antes do código) · `702819e48` (o estado
  medido, com as alavancas) · `da21b95c6` (as alavancas saem) · `d8ef0527a` (a cena, o tutorial, o plano) ·
  `ea306c7ca` (o fecho) · `a45a65b69`/`fb18873f8`/`a9fbdac97` (o canto, a cena 6, a área barata por dentro);
  e a W19 — `085d9d196` (o plano §28, antes do código) · `d1c142990` (a cota, a queixa, os textos) · `c1e9a4f70`
  (fmt) · `ecbfd512c` (a cena 7, os gates dirigidos, o arnês) · `eaad9c7d2` · `26053f9fc` (a sonda) · `63a1c394e`
  (o plano §28.6–§28.9) — e o deste handoff.
- Nenhum ADR novo (o máximo continua `0180`, da W16).

## §1 — Superfície de colisão (a W18)

| ficheiro | o quê |
|---|---|
| `ph2d-app-components/src/nav_smoke_lama.rs` + `_tests.rs` | NOVOS: a cena `=5` e os gates dela |
| `ph2d-app-components/src/nav_smoke_usos.rs` + `_tests.rs` | NOVOS: a cena `=6` (a lama nos jogos) e os gates dela |
| `ph2d-app-components/src/nav_smoke.rs` (+ `_tests.rs`), `lib.rs` | o roteador: braços `5` e `6`, `CENAS = 6`, `Montada::{lama, usos}`, `Montada::secao_do_roteiro`; `pub mod nav_smoke_{lama,usos}` |
| `shells/desktop/src/components_scenes_suplentes.rs` | UMA linha trocada (`let sec = montada.secao_do_roteiro();`) — zero linhas a mais |
| `ph2d-nav/src/agent.rs` | `AgentRuntime::custos_do_caminho`, `Vez::custos`, o 5.º motivo de replaneio; `ALCANCE_DO_CANTO` (a aceitação de um canto) |
| `ph2d-physics-ecs/src/bridge/nav.rs` | a assinatura dos custos passa ao `Vez`; o `struct Vez` da ponte MUDOU-SE para `nav_fila.rs` (`694 → 684` linhas) |
| `ph2d-physics-ecs/src/bridge/nav_fila.rs` | `entradas_das_procuras` devolve também a assinatura dos custos e atalhos; o `Vez` da ponte |
| `ph2d-physics-ecs/tests/it/nav_custo.rs` | o gate do custo ao vivo |
| `docs/Components/30_plano_navegacao.md` · `tutoriais/src/03_navegacao.html` + `.pdf` · `CLAUDE.md` §5 | o plano §27, a página da lama, o roteador |

Zero crate, zero pacote no `Cargo.lock`, zero i18n, zero componente, `PROJECT_SCHEMA` intocado. O
`AgentRuntime` ganhou um campo — ele vai no anel (`ControllerMemory::nav`), não no ficheiro do projecto.

## §1b — Superfície de colisão (a W19)

| ficheiro | o quê |
|---|---|
| `ph2d-nav/src/cota.rs` | NOVO: a cota inferior (`inferior`, `Cota`) — o heurístico, a saída cedo, a poda dos atalhos e o «à vista» |
| `ph2d-nav/src/mesh.rs`, `blocos_junta.rs` | `NavMesh::caixas` (a caixa de cada polígono de área, por área) e `caixas_da_area`; as duas portas de construção preenchem-no; `diferenca` compara-o |
| `ph2d-nav/src/polyanya.rs`, `polyanya_custo.rs`, `polyanya_fatias.rs`, `link.rs`, `agent.rs`, `lib.rs` | os quatro leitores da cota; `Root::d`; `Polyanya::set_cota_global` (o CONTROLO); `a_vista` sem a regra «toda a tabela `≥ 1`» |
| `ph2d-navmesh/src/inflate.rs`, `lib.rs`, `tiles.rs` | `inflate::some_na_malha`; `Params::lados_do_disco` (a porta que a construção e os mosaicos passaram a usar) |
| `ph2d-physics-ecs/src/components/nav.rs`, `components.rs`, `lib.rs` | `NavCostAreaNow` (DERIVADO, NÃO registado — o precedente do `NavNow`) |
| `ph2d-physics-ecs/src/bridge/nav_custo.rs`, `nav.rs` (`+1` linha, `685/700`) | `publica_areas_estreitas`, chamada do `publica_navegacao` |
| `ph2d-editor-core/src/nav_edits.rs`, `ph2d-app-components/src/nav_inspector.rs`, `ph2d-panel-inspector/src/sections/nav_custo.rs`, `ph2d-i18n/src/inspector_nav.rs` | `too_narrow_for`, `CostAreaQueixa::MaisEstreitaQueOCorpo`, a frase `panel.inspector.nav.area_narrower_than_body` (com o raio) |
| `ph2d-app-components/src/nav_smoke_estreita.rs` + `_tests.rs` (NOVOS), `nav_smoke.rs` (`CENAS = 7`, o braço `7`), `nav_smoke_usos.rs` (3 helpers `pub(crate)`), `lib.rs` | a cena `=7` |
| `ph2d-physics-ecs/src/components/nav.rs:135`, `bridge/nav_custo.rs:6` | os dois doc-comments que mentiam (§28.4) |
| testes: `ph2d-navmesh/tests/it/{cota.rs (NOVO), dominancia.rs, fatias.rs, main.rs}`, `ph2d-physics-ecs/tests/it/nav_custo.rs`, `ph2d-nav/src/agent_tests.rs`, `ph2d-editor-core/src/nav_edits_tests.rs`, `ph2d-panel-inspector/tests/it/a_seccao_nav_custo_esta_viva.rs`, `ph2d-panel-registry-init/tests/it/o_inspector_armado.rs`, `ph2d-app-components/src/nav_inspector_tests.rs` | os gates (§4b) |
| `fmt` só: `smoke_desenho*.rs`, `ph2d-render/src/atlas/{mod,tests}.rs`, `nav_desvio.rs`, `nav_tests.rs`, `tests/it/{nav_desvio_largo,nav_fatias,nav_mundo}.rs`, `examples/medir_replaneio.rs` | ficheiros da linha (W15–W18) que o `cargo fmt` reprovava |
| `docs/Components/30_plano_navegacao.md` §27.10–§27.11 (corrigidos) e §28 · `tutoriais/src/03_navegacao.html` + `.pdf` (secção 12 nova) · `ferramentas/mutacao_navegacao_w19_2026-10-07.py` · `ph2d-navmesh/examples/medir_cota_w19.rs` | o plano, o tutorial, a mutação, a sonda |

Zero crate, zero pacote no `Cargo.lock`, `PROJECT_SCHEMA` intocado (o `NavCostAreaNow` não é registado), UMA
chave i18n nova.

## §2 — O que a W18 traz

| item | veredito | o quê |
|---|---|---|
| **S** a cena da lama (`PH2D_NAV_SMOKE=5`) | ✓ | duas pistas iguais, `Light Mud` (`Cost 2`) e `Heavy Mud` (`Cost 10`, escolhida, a secção `Nav Cost Area` aberta); três corredores por pista: a leve atravessa-se, a pesada contorna-se. Gates pela cena real com o CONTROLO (as duas a `1`: cruzam as duas) |
| ⭐ **o `Cost` ao vivo** (achado pela cena) | ✓ curado | mexer no `Cost` com o agente a ANDAR era um controlo morto (`10 → 2`: `0` procuras novas; só `→ 1` mudava algo, a lama a sair da malha). O 5.º motivo de replaneio: a assinatura dos custos e dos atalhos mudou desde que o caminho foi planeado — paga do orçamento como as outras procuras; um `Nav Link` posto/tirado entra pela mesma porta |
| ⭐ **o canto alcançado** (report do dono: *«o R2 entra na quina da lama»*) | ✓ curado | um canto alcança-se também a `0,1` do raio (`ALCANCE_DO_CANTO`), não só no passo do executor (`1,3 cm` a `0,8 m/s`): o R2, empurrado `2–5 cm` na passagem, voltava atrás `109` tiques até o centro entrar na lama. Igual ao bit na cena de stress (lá o passo já passa a fracção). Plano §27.9b |
| ⭐ **a cena 6, a lama nos jogos** (pedido do dono) | ✓ | quatro usos lado a lado — pedras à volta de uma estrada (`Cost 4`), rio com ponte (`Cost 6`), canteiro `Forbidden`, a luz de um guarda (`Cost 8`) — cada um com o CONTROLO. A estrada como área BARATA foi medida e recusada para a cena (o corredor anda pela berma de fora: plano §27.11) |
| **A** o tecto do caminho crítico | ✗ recusa medida | T1 e T3 cumprem o crítico (`20` · `40 mil`) e REPROVAM a vivacidade (a procura nunca acaba numa malha que muda de 2 em 2 tiques — a aritmética: a fatia tem de ser `≥ W/K`); T2 é vivo e deixa o crítico igual (`80 mil`). Sem tecto, o A4 fica recusa |
| **C** menos nós na ponderada | ✗ recusa medida | a dominância de hoje JÁ está no tecto do que frentes anteriores cortam (o tecto amostrado: `0,04 %`); o heurístico toca `6–16 %` dos nós; a fase geral `~10 %` do tempo |

## §2b — O que a W19 traz (a auditoria de duas lentes, num ciclo)

| item | veredito | o quê |
|---|---|---|
| **H** o heurístico com uma área barata | ✓ | UMA estrada barata num canto pesava em TODA procura (a cota escalava pelo menor custo da tabela): `85 766 → 24 232` nós por consulta (`1,002×` o mundo sem estrada; kill `≤ 1,1`), `1 808,7 → 376,7 ms` nas `60` consultas da cena grande (`4,8×`, load `1,4`), custo igual ao dígito. A cota conta cada área barata pela DISTÂNCIA até ela (prova no doc-comment de `cota.rs`) |
| **V** o «alvo à vista» | ✓ | desligava-se no mundo inteiro com um custo `< 1` na tabela; agora só quando uma área barata pode encurtar a recta — a MESMA função `inferior` |
| **Q** a área barata mais estreita que o corpo | ✓ | a ponte publica `NavCostAreaNow` (a mesma erosão da malha), o Inspector diz *«Narrower than the body that walks it (radius 0.25 m)…»*; a cena `=7` mostra-o |
| **D** os textos que mentiam + fmt | ✓ | `nav.rs:135`, `nav_custo.rs:6`, plano §27.10–§27.11; fmt limpo nos ficheiros da linha |

## §3 — ⏳ O que fica ABERTO

- **O crítico do tique** chega a `orçamento · 2^k` (`80 mil` a `10` e `50` agentes na cena de stress) — o preço
  da vivacidade numa malha que nunca pára; não se vê no relógio (o pior tique igual em todas as versões).
  Fechado como escolha medida, não como dívida.
- **Quem persegue sem o alvo à vista** na lama cerrada (máx `30` tiques a `200` agentes): o A4 só cabe com um
  tecto que a vivacidade não deixa.
- ~~**Uma área mais barata que o chão**~~ — curada na W18 (`a9fbdac97`, `Area::dentro`); os achados da auditoria
  sobre ela fecharam na W19.
- (W19, §28.9) `d(x, K)` varre as caixas de todos os polígonos baratos por raiz: barato nas cenas de hoje; um
  mundo com milhares de polígonos baratos pediria uma grelha das caixas — por medir quando existir.
- **A procura ponderada** está no tecto deste desenho; metade do tempo pediria OUTRA procura.
- (de antes, sem mudança) a versão web: aprovada pelo dono, ESTACIONADA.

## §4 — A prova de fecho

- **As medições** (a régua do dono de 05/10: as versões no MESMO processo, intercaladas, o mínimo, perfil
  `smoke`, loadavg anotado), `target/prova/w18/`: `medir_replaneio_w18.txt` (A, load `8–17`: valem as colunas
  de trabalho e de espera), `diag_nos.txt` · `diag_nos_c4.txt` · `diag_nos_tecto.txt` ·
  `diag_nos_tempo_geral.txt` (C), as fotos `foto/cena5_{a_meio,fim}.png`.
- **Gates novos:** `nav_smoke_lama::tests::{a_leve_atravessa_se_a_pesada_contorna_se,
  com_o_cost_em_2_os_da_direita_cortam_pela_lama, a_cena_tem_as_pecas_que_o_roteiro_nomeia,
  nenhum_corredor_volta_atras_para_um_canto}` · `nav_smoke_usos::tests::{cada_uso_muda_o_caminho_e_o_controlo_corta_a_direito,
  a_cena_tem_as_pecas_que_o_roteiro_nomeia}` ·
  `nav_smoke::tests::o_cenas_conta_os_niveis_do_roteador` · `nav_custo::mexer_no_custo_refaz_o_caminho_de_quem_anda`.
- **Mutação** ([`mutacao_navegacao_w18_2026-10-06.py`](../ferramentas/mutacao_navegacao_w18_2026-10-06.py), o
  motor da W15/W16): **`16 / 16`** sangram, `0` defeitos de arnês, checksums iguais —
  L1–L6 (a cena: o custo, a área, o roteador, a secção aberta, `CENAS`, o desenho), C1–C4 (o 5.º motivo, o
  registo no caminho instalado, a assinatura, a ponte), K1 (o canto só no passo do executor) e U1–U5 (a
  cena 6: os custos a `1`, o canteiro sem `Forbidden`, o roteador). `L5` (`CENAS = 4`) só sangra pelo gate novo do roteador.
- **Gate batched** (sobre o merge-base, W15–W18 juntas, depois da cura do canto): `nextest-impacted`
  **`15 787 / 15 787`** (`92,3 s`, com a cena 6) — a 1.ª corrida deu
  `15 782 / 15 783`: `o_tutorial_da_navegacao_so_cita_rotulos_que_o_painel_pinta` (o tutorial marcava `Cost 2` e
  os nomes das lamas como rótulos de tela), curada em `ea306c7ca` ·
  clippy `--workspace --all-targets -D warnings` limpo · fmt limpo nos ficheiros da W18 —
  `target/prova/w18/gate_{nextest,clippy}.txt`. Load `5` antes e `18–20` durante.

## §4b — A prova de fecho da W19

- **As medições** (`target/prova/w19/`): `medir_cota_final.txt` (H, load `1,4`), `medir_cota_1.txt` (a 1.ª rodada,
  load `5–10`: só as colunas de trabalho), as fotos `foto/cena{5,6,7}_{a_meio,fim}.png`. ⚠️ As fotos da W18 da
  cena 6 eram da 1.ª versão (`Rough Ground`), anteriores a `a9fbdac97`; as da W19 são a cena de hoje (a `Road`).
  A cena 5 bate com a W18 (`0,2–0,4 %` dos pixels: o relógio da timeline).
- **Gates novos:** `cota::{uma_area_barata_longe_nao_pesa_na_procura, a_cota_nunca_encarece_um_caminho_e_o_oraculo_confirma,
  quando_a_vista_diz_sim_nenhum_caminho_e_mais_barato, com_atalhos_a_poda_pela_cota_nunca_perde_o_mais_barato,
  o_atalho_que_acaba_numa_estrada_barata_nao_e_podado, com_a_estrada_longe_a_saida_cedo_poupa_a_ponderada}` ·
  `fatias::a_procura_em_fatias_e_a_procura_inteira_ao_bit` (agora com áreas baratas) ·
  `nav_custo::{uma_area_barata_longe_nao_desliga_o_alvo_a_vista, a_ponte_publica_a_area_barata_mais_estreita_que_o_corpo}` ·
  `nav_edits::tests::a_area_barata_mais_estreita_que_o_corpo_queixa_se` ·
  `a_seccao_nav_custo_esta_viva::a_area_barata_mais_estreita_que_o_corpo_diz_se_no_painel` ·
  `nav_smoke_estreita::tests::{a_larga_anda_se_a_estreita_nao_faz_nada, a_estreita_queixa_se_e_a_larga_nao, a_cena_tem_as_pecas_que_o_roteiro_nomeia}`.
- **Mutação** ([`mutacao_navegacao_w19_2026-10-07.py`](../ferramentas/mutacao_navegacao_w19_2026-10-07.py)):
  **`15 / 15`** sangram, `0` defeitos de arnês, checksums iguais — H1–H5 (a cota), V1–V2 (o «à vista»), Q1–Q5 (a
  queixa), S1–S3 (a cena). A 1.ª corrida deu `12 / 15` (H1, H2, H5 sobreviveram — plano §28.8): a régua «sem a
  estrada» passou a ser o mundo sem estrada nenhuma, e dois gates dirigidos nasceram. `target/prova/w19/mutacao_w19{,_cota}.txt`.
- **Gate batched** (sobre o merge-base, W15–W19 juntas): `nextest-impacted` **`16 672 / 16 672`** (`90,9 s`, `9 706` saltados) ·
  clippy `--workspace --all-targets -D warnings` limpo · fmt limpo nos ficheiros da linha —
  `target/prova/w19/gate_{nextest,clippy}_2.txt`. A 1.ª corrida do clippy reprovou dois achados da W19, curados
  antes da 2.ª: `large_enum_variant` no `plano::Etapa` (o `Atalhos` ganhou o `Vec` das distâncias: `Box`) e um
  `assert!` constante no teste da cena 7 (o `const` do módulo já o garante). Load `9` antes e `28` depois (outras
  linhas na máquina): nenhum teste reprovou, logo nenhum a confirmar sozinho.

## §5 — O smoke

✓ **Smoke do dono:** a cena 5 aprovada (06/10, depois da cura do canto); a cena 7 (a W19, os passos abaixo)
aprovada a 07/10 (*«smoke ok»*); a 6 também (07/10, *«parece ok»*) — (a cena `=5`, fotografada antes: `target/prova/w18/foto/cena5_a_meio.png` — à
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

E a cena 6 (fotografada: `cena6_{a_meio,fim}.png`; a secção 11 do PDF):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke`
2. Só olhar os quatro: o 1 segue a estrada escura em U (a `Road`, `Cost 0.3`, com o corpo em cima dela); o 2 vai à ponte; o 3
   contorna o canteiro verde; o 4 contorna a luz amarela pela sombra da direita.
3. A `Road` comprida vem escolhida: `Espaço` logo no início, `Cost` = `1`, `Espaço` — o 1 corta a direito.
4. Deu errado se o 1 anda ao lado da estrada e não em cima dela, ou algum vermelho pisa a água, o canteiro ou a luz.

E a cena 7 (W19; fotografada: `target/prova/w19/foto/cena7_{a_meio,fim}.png`; a secção 12 do PDF):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Só olhar: à esquerda o vermelho segue a estrada escura LARGA em U; à direita sobe a direito, ignorando a fina.
3. O Inspector (a `Narrow Road` escolhida, `Nav Cost Area` aberta): em amarelo *«Narrower than the body that walks it (radius 0.25 m)…»*.
4. Deu errado se o da esquerda corta a direito, o da direita vai à fina, ou o aviso não aparece.

O binário compilado no fecho da W19 (a 2.ª corrida, colada), depois de `rm -rf target/*/incremental` (`12 G` do
`debug`, `2,7 G` do `smoke`):

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho da W19:

```
  ✗ paralelismo de ferramenta              1.11/passo   alvo: >= 1,5  (7% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                241   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                657 : 181   alvo: <= 1,0  razao 3.6x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (2254 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         505 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               61 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

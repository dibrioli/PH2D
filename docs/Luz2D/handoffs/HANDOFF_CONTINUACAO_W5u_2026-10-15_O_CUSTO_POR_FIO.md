# Handoff de continuação — Luz 2D, W5u: o custo é por fio (2026-10-15)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5q ([handoff](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md)), da W5r
> ([handoff](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md)), da W5s
> ([handoff](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md)) e da W5t
> ([handoff](HANDOFF_CONTINUACAO_W5t_2026-10-15_A_SUBIDA_PAGA_POR_FIO.md)), todas não integradas; os smokes da
> W5s e da W5t continuam pendentes do dono. Commits da W5u: `a1d5c64751` (fix: instrumento das etapas) ·
> `39704af9cf` (perf: variantes desligadas) · `9917440bd2` (test: Medium nas réguas, fatia por passe, cheia em
> regime, chaves W5U, régua do mesmo processo) · `fc96ca6d9f` (perf: 4 cones/4 pares por omissão; rgb fora) ·
> `e6dbd4b531` (test) · `42974a2e01` (perf: limiar 2^17, primeiro no Config) · `ebe79e46de` (test: grades no
> rótulo, chaves só do entorno) · `ff285fd73a` (fix: a decisão no UNIFORME `por_fio`, não nas constantes; + fmt).
> Nada integrado, nada enviado. Placa RTX 5060 Ti (448 GB/s). `c2` = ms de GPU do passe holográfico; «por
> processo» = um processo por medida (`medida_luz2d_w5o_lastro`).

## 0. A onda inteira, pelo produto

Inspector «custo sem o Automático» (`a_nitidez_da_luz::com_o_automatico_desligado_o_custo_e_o_do_nivel_escolhido`,
3 rodadas alternadas, um processo cada; um `ph2d-host-desktop` da `line-UIUX` estava na placa em todas: o A/B vale,
os absolutos estão sujos):

| High, ms | rodada 1 · 2 · 3 |
|---|---|
| antes (binário da W5t) | 11,39 · 11,42 · 11,29 |
| depois (W5u) | **9,93 · 9,07 · 10,12** |
| depois + `PH2D_LIGHT2D_HRC_POR_FIO=0` | 11,29 · 10,38 · 11,30 |

Low ~2,1 nos três braços (igual): a grade do Low do app a 1080p é 480×270 = 129 600 texels, logo abaixo do
limiar 2^17 = 131 072 — corre uma entrada por fio. O teste só mede Low e High (sem Medium).

Fotos (`fotografa_cena.sh`, `PH2D_LIGHT2D_SMOKE=5`, HOME isolado com cópia): High padrão contra chave=0 AE = 0; Low
padrão contra chave=0 AE = 0; controlo High contra Low 6 629 px (a cena muda) ⇒ a imagem é a mesma com e sem o corte.
⚠️ Prefs do dono hoje: `light_quality=low`, `light_auto=0` (não High, como a W5t dizia); não mexi.

Por processo, placa limpa (só kwin), binário final contra chaves desligadas (cones 1, pares 0), ms:

| | c2 antes → depois | vista total + entorno antes → depois |
|---|---|---|
| Low | 0,523 → 0,501 | 1,923+0,228 → 1,895+0,229 (≈2,15 → 2,13) |
| Medium | 1,819/1,822 → 1,595/1,592 | 3,263+0,453 → 3,046+0,442 (3,72 → 3,49, −6 %) |
| High | 7,291/7,290 → 6,018/6,018 | 8,835+1,404 → 7,570+1,367 (10,24 → 8,94, −13 %) |

## 1. Medir antes de cortar

**(a) O instrumento das etapas** (`PH2D_W5Q_FASES=1`) não terminava com `PH2D_W5S_NIVEL=low` (W5s §6b.5).
Mecanismo: sem refletor na cena o Low NÃO corre o rebote extra, mas o instrumento reservava timestamps para 2
gravações; resolver um timestamp nunca escrito faz a GPU esperar para sempre. Cura (`a1d5c64751`):
`resolver_fases(enc, feitas)` resolve/lê só as gravações feitas (`Cell feitas`). Item fechado.

**(b) Tabela dos três níveis** (por processo, vazia = cena 1, em regime). `c2`: Low 0,52 · Medium 1,81 · High
7,24. Fatia do quadro da vista (linha Q «mín por passe», vazia): Low total 1,91 + entorno 0,22 — miolo 0,70
(37 %) · c2 0,52 (27 %) · preenche 0,37 · fina 0,24: **no Low a holográfica NÃO é a maior**; miolo, preenchimento
e fina (iguais em todo nível, ~1,3 ms) são. Medium total 3,25 + 0,45: c2 1,81 (56 %). High total 8,79 + 1,40: c2
7,24 (82 %). Etapas HRC vazia (soma dos 4 quadrantes, mín), ms:

| | traça | sobe | desce | soma | saída |
|---|---|---|---|---|---|
| Low | 0,07 | 0,16 | 0,30 | 0,04 | 0,01 |
| Medium | 0,18 | 0,50 | 1,04 | 0,12 | 0,04 |
| High | 0,60 | 2,03 | 4,02 | 0,53 | 0,15 |

**(c) ⛔ A régua da «cheia» MENTIA:** o mínimo da `c2` era o 1.º quadro. Série por quadro, High cheia: 9,41 10,15
14,61 16,14 17,81 16,81 16,89 17,46 16,99 17,02 17,01 17,01 (o rebote acende a cena nos primeiros quadros); Low
cheia 1,42 e depois 1,49–1,50. Os «cheia 9,41/10,24/13,01» da W5t eram números do 1.º quadro: em regime a High
cheia é **~17,0 ms** (Low 1,49, Medium 2,88). Cura na régua: `a_c2_e_as_etapas` aquece 12 quadros
(`PH2D_W5U_AQUECE=0` = protocolo antigo). A vazia é estável (mín = med): as conclusões da W5t sobre a vazia valem.

**(d) Mecanismo.** Na cheia em regime, sobe/desce/soma custam 2–3× a vazia embora a aritmética não dependa dos
valores (sem ramos por valor): High cheia traça 2,92 · sobe 5,0 · desce 6,98 · soma 1,67. A desce move ~350 MB
únicos por nível em ~0,6 ms ≈ 580 GB/s (DRAM 448); a sobe ~234 MB em 0,56 ≈ 420 GB/s ⇒ limitadas pela DRAM com
valores incompressíveis; na vazia o mesmo tráfego vai 2–3× mais rápido ⇒ a placa comprime dado uniforme (vácuo).
Mecanismo candidato, coerente com o G4 da W5s («a placa cobra os VALORES»), NÃO provado direto. Consequência: na
vazia o custo é por fio (instruções/bytes comprimidos); na cheia são bytes.

**(e) Orçamento por fio da descida** (do shader). Por entrada (cone m da sonda (xi,v), nível n): xi ímpar 2
leituras de T + 2 de R + 2 arcos + 1 escrita; xi par 2 de T_{n+1} + 4 de R + 2 arcos + 1 escrita. Cones vizinhos
partilham a aresta T (o cone m e o m+1 partilham a aresta m+1). Subida: k par 2 leituras de T, ímpar 4; um par
(2c, 2c+1) precisa de 5 em vez de 6 e o B(c+1) passa ao par seguinte.

## 2. Os cortes

`Config::hrc_cones_por_fio`, `Config::hrc_pares_na_subida`; overrides `CONES_HRC`, `PARES_SOBE_HRC` no
`c2_hrc.wgsl`; só no 3D/colunas. `descer_cones` (C cones por fio, a aresta T lida uma vez, a MESMA ordem de soma de
`soma_da_descida`); `subir_pares` (P pares por fio, a mesma aritmética de `fusao_de_baixo`).

**Régua do mesmo processo** `medida_luz2d_w5u_por_fio` (os 16 casos da W5s × Low/Medium/High = 48): TODAS as
variantes (cones 2/4/8, pares 1/2/4, combos c4p1 c4p2 c8p2 c4p4, rgb) com **0 bytes diferentes em 48/48**; dois
controlos por texto do módulo mutado via `Config::modulo_rc` (cone m+1 com `t_ant` em vez de `t_prox`; o `menos`
com `b_prox`) mudaram 48/48 cada. ⚠️ Com 7 instâncias High no mesmo processo o regime lento cai nas NASCIDAS
primeiro (base 37 ms, as seguintes 8): ali só o BIT vale; o relógio é por processo.

**Por processo** (2×; um `ph2d-host-desktop` na placa na maioria ⇒ só o A/B vale), `c2` mín base → pares2 → c4p2 →
c4p4, ms:

| | base | pares2 | c4p2 | c4p4 |
|---|---|---|---|---|
| Low vazia | 0,523 | 0,501–0,505 | 0,508 | 0,500 |
| Medium vazia | 1,819 | 1,684 | 1,605 | 1,599 |
| High vazia | 7,30 | 6,60 | 6,10 | 6,02–6,07 |
| Low cheia (regime) | 1,486 | 1,452 | 1,454 | 1,449 |
| Medium cheia | 2,878 | 2,863 | 2,768 | 2,770 |
| High cheia | 17,93 | 17,4–18,0 | 17,16 | 17,19 |

Placa limpa: High cheia 17,01/17,02 → 17,02/17,00 (IGUAL); etapas sobe 5,04 → 5,17–5,21 (+3 %, limitada pela
memória), desce 6,98 → 6,95, soma 1,67 → 1,62. High vazia: sobe 2,03 → 1,24 (−39 %), desce 4,02 → 3,52 (−12 %).
O ganho é da VAZIA (por fio); a cheia, de bytes, não se move.

**Antes contra novo-com-chaves-desligadas** (binário de `71715a76b9` contra o novo, protocolo antigo, por processo,
2×): Low vazia 0,523/0,523 contra 0,521/0,522; High vazia 7,28/7,27 contra 7,26/7,27; cheia (1.º quadro) igual ⇒ o
código novo com as chaves desligadas = o antigo.

**O limiar.** O ENTORNO do Low (256×160) piorou com 4/4: 0,229 → 0,275. Medida só do entorno (por processo, 2×),
1/0 · 2/2 · 4/4, ms: 256×160 0,229 · 0,239 · 0,275; 512×320 0,451 · 0,436 · 0,443; 992×576 1,407 · 1,371 · 1,367;
mais a vista 640×384 0,522 · — · 0,500 e 2496×1472 7,32 · — · 6,04. Recurso = o paralelismo da placa (menos fios
numa grade pequena). Lei: `hrc::POR_FIO_DESDE = 1 << 17` texels; abaixo, uma entrada por fio. A decisão vive no
UNIFORME (`UniformeHrc::por_fio`, onde estava o preenchimento; tamanho continua 96 B) e no tamanho do despacho; as
constantes do pipeline ficam 4/4. A 1.ª versão pôs a decisão nos padrões do Config e
`redimensionar_a_janela_nao_recompila_a_luz` ficou vermelho (6 pipelines recompilados ao redimensionar: uma janela
que cruza o limiar mudava as constantes). App: `PH2D_LIGHT2D_HRC_POR_FIO=0` (vista E entorno) compara.

## 6b. ABERTO, na ordem

1. A cheia (em regime) é limitada pela DRAM com valores incompressíveis: High 17 ms — a próxima alavanca são os
   BYTES (formato = precisão ⇒ decisão de produto; o «R em 6 bytes» foi recusado pelo custo por fio da memória
   partilhada/barreiras na vazia). Um fio por par de cones escreve rg_m, rg_{m+1} e um u32 com b_m|b_{m+1} SEM
   memória partilhada: mecanismo novo, NÃO medido; só com o aval do dono, pois «R em 6 bytes» está na lista de
   recusados.
2. No Low o relógio é o miolo (0,70), o preenchimento (0,37) e a fina (0,24), não a holográfica (0,52). Outra
   família/onda.
3. A grade do Low do app (480×270 = 129 600) está logo abaixo do limiar; o intervalo entre 41 k (pior) e 164 k
   (melhor) não está medido — meça grades de 100–160 k antes de mexer no limiar.
4. A soma na cheia: 1,67 ms, limitada por bytes (acumulado RMW f32×4 por quadrante); a versão rgb (12 B) foi
   recusada (Recusas); uma soma única sobre os 4 quadrantes exigiria os 4 R0 vivos (+88 MB, armadilha do regime
   lento).
5. O pior caso do Low (W5s §2, decisão do dono) continua.
6. ⚠️ `docs/Components/ferramentas/fotografa_cena.sh`: crases dentro de um heredoc (linha ~108, `[env]`,
   `.cargo/config.toml`) imprimem «comando não encontrado» / «Permissão negada». Não é da luz: reportar ao dono do
   roteiro.

## ⛔ Recusas MEDIDAS e cuidados

| tentativa | medido |
|---|---|
| o acumulado em rgb (3 f32, 12 B por texel) | soma High vazia 0,53 → 0,94 por processo; c2 7,30 → 7,83; mesmo processo +3 a +40 % em todo nível (escalares de passo 3) |
| cones 8 / pares 1 / c8p2 | não melhores que c4p4 em nenhum nível (régua do mesmo processo) |
| a decisão por grade nas CONSTANTES do pipeline | redimensionar recompila 6 pipelines (portão `redimensionar_a_janela_nao_recompila_a_luz`) |
| o mínimo da c2 da cena cheia sem aquecer | lia o 1.º quadro (9,4 contra 17 em regime) |
| 7 instâncias do High no mesmo processo para o relógio | as primeiras caem no lento (base 37 ms) — só o bit vale |
| (da W5t/W5s/W5r/W5q, continuam) | [W5t](HANDOFF_CONTINUACAO_W5t_2026-10-15_A_SUBIDA_PAGA_POR_FIO.md) · [W5s](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md) · [W5r](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md) · [W5q](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md) |

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu` (+ o app `ph2d-app-light2d`). Campos PÚBLICOS novos
  `Config::hrc_cones_por_fio` (4) e `Config::hrc_pares_na_subida` (4); todo literal do workspace usa `..`.
  Overrides novos `CONES_HRC`, `PARES_SOBE_HRC`; `UniformeHrc._p[0]` virou `por_fio` (WGSL `_p0` → `por_fio`).
- O texto do `c2_hrc.wgsl` mudou: a 1.ª abertura depois da integração recompila a luz. Nenhum contrato congelado
  (§6).
- Testes: `tests/it/medida_luz2d_w5u.rs` (`#[ignore]`); chaves do `lastro`: `PH2D_W5S_NIVEL=low|medium`,
  `PH2D_W5U_CONES/PARES`, `PH2D_W5U_E_CONES/E_PARES`, `PH2D_W5U_AQUECE=0`, `PH2D_W5U_SERIE=1`;
  `a_c2_e_as_etapas` agora aquece 12 quadros e imprime a linha Q (fatia por passe).
- A tag local `w5t-recusados-esparso-r6` é só arquivo: NÃO empurrar.

## Gate e binário

Gate do fecho (verificador, régua = merge-base `0aab2c1cb6`), 2.ª corrida: fmt verde; clippy `-D warnings`
(forma do `ship.sh`, `PH2D_SUITE=fecho`) verde; check `--workspace --all-targets` deny verde; censos 117/117;
`nextest-impacted` (`PH2D_SUITE=fecho`, `PH2D_GPU=1`) **11 753 / 11 753** (load 4–47). A 1.ª corrida achou o fmt e
`redimensionar_a_janela_nao_recompila_a_luz` (6 pipelines no resize: o limiar nas constantes) — curados em `ff285fd73a`.
Por nome: `g3_` 8/8, `g4_`, `g5_`, `holografica` 16/16, `g9_`, `g10_`, `g12_`, `o_topo_passa_a_grade`, `a_nitidez_da_luz` 6/6.

Cache do dono (`fotografa_cena.sh`, `__GL_SHADER_DISK_CACHE_PATH=/home/enio/.cache/nvidia`, `PH2D_LIGHT2D_SMOKE=5`, HOME
isolado com cópia das prefs em High): 1.ª abertura **0,13 s** (68 pipelines; o cache já estava quente), 2.ª **0,05 s**;
`GLCache` 1 489 → 1 490 MB. `target/*/incremental` reclamado (7,2 GB). Binário do smoke (2.ª corrida, e de novo depois
das fotos):

```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.24s
```

Régua do loop (`agent-loop-profile.sh`): paralelismo 1,16 (✗ ≥ 1,5) · `test:check` 2,7× (✗) · contexto por passo
287 mil (✗ ≤ 250 mil) · edições sem conferir 0,9 % (✓).

## 7. ⛔ O número do DONO não é o da régua (smoke do dono, 15/10)

Cena 5, Low manual: Inspector do dono **7,6 ms** (AOC, 1920×1080) e **9,0** (LG, 2560×1080; no app 9,5–9,7, a fina 3,8–4,5) — contra ~2,1
do `a_nitidez_da_luz` e 0,5 de `c2` do `lastro`. Reproduzido no app INTEIRO (`fotografa_cena.sh`, HOME isolado, cena
5) com o instrumento novo `PH2D_LIGHT2D_CUSTO=1` (o Inspector por passe):

| cena 5, 1920×1080 | total | c2 | fina | vis | miolo | rebote | preenche | entorno |
|---|---|---|---|---|---|---|---|---|
| Low (grade 640×384) | 8,2–9,1 | 1,4–1,8 | **3,2** | 0,8–1,6 | 0,78 | 0,6 | 0,37 | 0,47 |
| High (2496×1408), W5u | **18,0–18,8** | 11,6–12,1 | 2,7 | — | 0,76 | 0,5 | 0,34 | 1,5–2,0 |
| High, `PH2D_LIGHT2D_HRC_POR_FIO=0` | 23,7–26,0 | 16,5–18,0 | 3,3–4,0 | — | 0,76 | 0,5 | 0,34 | 1,7–1,9 |

⇒ (1) No Low o relógio é a FINA (3,2 ms, o mesmo em todo nível) + a visibilidade + o miolo — não a holográfica.
(2) No app a `c2` do High é ~2× a da régua (11,6 contra 6,0): conteúdo + rebote (o mecanismo da cheia, §1d) e a placa
partilhada com o desenho da UI. O corte da W5u vale MAIS no app: −5 a −6 ms. (3) ⛔ O `a_nitidez_da_luz` e o `lastro`
(cena 1, quase vazia) NÃO medem o produto: a próxima onda mede a CENA 5 no app (este instrumento) antes de cortar.
⇒ o §6b começa pela fina.

## Smoke do dono

⏳ Pendente (o da W5s e o da W5t também).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_TEMPO=1 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. A cena de sempre: lâmpada no centro, oito formas coloridas com sombra.
3. As prefs estão hoje em Low manual. Clicar a **Lâmpada** → no Inspector, Quality → High. O Inspector diz
   «Quality: High (manual), NN ms of GPU per frame»: ~9–10 (antes da W5u ~11,3). Low continua ~2,1 (igual).
   Medium: ver o número, ~3,5 esperado com o entorno.
4. Comparar: a mesma linha com `PH2D_LIGHT2D_HRC_POR_FIO=0` à frente — a cena IGUAL, o número do High ~1,3 ms mais
   alto. Errado: imagem diferente entre os dois, ou o High acima de 11.
5. Honestamente: a barra de fps está presa ao teto de 60 da tela — o ganho não se sente na barra, só no número do
   Inspector e na folga para cenas maiores.

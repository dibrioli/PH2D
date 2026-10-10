# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-10 — F7c: O QUICKSHAPE COM ARCOS

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F7b, 09–10/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-09_F7b_QUICKSHAPE_SENSIBILIDADE.md)
> — citada, não reescrita. F0–F7: os handoffs dela.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `2a925859d` (o `main` não andou — sem rebase)
  · F0–F7b continuam por integrar e vão JUNTAS com esta jornada.
- Commits desta jornada: `d1e658973..HEAD`. Um deles (`wip(UIUX F7c): busca temporaria…`) é a busca
  das testemunhas da mutação, apagada no commit seguinte — inofensivo; o integrador pode deixá-lo.
- **O pedido (dono, 10/10, depois do smoke OK da F7b):** *«QuickShape com arcos»* — um arco de mão
  SEGURO ficava traço livre; no Procreate (R4 L165–169) vira um arco perfeito.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro (só o link; a frase e as cenas não mudam).

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **o arco** | `QuickShape::Arc(ArcShape)` — `ArcShape { centre, rx, ry, rot, start, sweep }` (círculo `rx == ry`, ou elipse — §1b; ângulos PARAMÉTRICOS em radianos; o SINAL de `sweep` é o sentido do desenho, `\|sweep\| < 2π`). Um traço ABERTO que a lei poligonal não explica é arco se a curvatura tem UM sinal (fracção da viragem contra o sentido, entre 6 cordas iguais, ≤ 0,25) e o círculo de **Taubin** (porte do `CircleFitByTaubin` de Chernov, 2010) o desenha a rms ≤ 0,05 × min(raio, diagonal); mais de uma volta não é arco | `ph2d-vec-edit/src/quickshape_arc.rs` (novo); o ramo em `recognise_with` |
| ⭐ **polilinha E arco** | ACHADO pelo gate de mão: um arco de 60° lia-se polilinha de 2 lados (cada metade com flecha no limite da régua do recto, 6,5 %). Quando o traço aberto é polilinha e o arco também passa, ganha a forma mais PERTO do traço (rms); a polilinha mede-se SEM o `regular`, para o `Shift` nunca trocar o tipo | `quickshape.rs` (`recognise_with`), `quickshape_arc::polyline_rms` |
| **o caminho** | `ArcShape::to_path`: troços de ≤ 90° (+5 % de folga: o meio arco de uma mão tem 181° e fica com 3 nós) com a alça `(4/3)·tan(α/4)·r` — a lei do `ph2d_vec_scene::arc` com início e sentido próprios; `Smooth` no meio, `Corner` nas pontas, `closed: false`; a elipse é o arco do círculo unitário levado pela afinidade dela (exacta para cúbicas) | `quickshape_arc.rs` |
| **o gesto** | o mesmo de toda forma: o pivô de uma forma aberta é o INÍCIO (`pivot`), arrastar escala e roda à volta dele (`transformed` anda o `start`), soltar comete o caminho com nós — gate `the_pencil_holds_an_arc_and_the_drag_turns_it_about_its_start` | `quickshape.rs` |
| **`regular`** (`Shift`/2.º dedo) | DECISÃO técnica (§2): o varrido em múltiplos de 15° e a corda início → fim a 15° (a lei da linha); o início fica onde a mão o pôs, o raio é o ajustado | `quickshape_arc::regularise` |
| **o oráculo** | `arco_meio` e `arco_quarto` (crus no Xournal++) passam a excepção do DONO com a geometria do gerador (`own()` 25 → 27) | `quickshape_tests.rs` |
| **as sondas** | `cases()`: «meio arco (não é forma)» passa a «meio arco» (tem de sair ARCO); `measure_open_gaps` conta o C de ¾ como arco (30/30); as três constantes novas na varredura de faixas | `quickshape_measure_tests.rs`, `quickshape_sensitivity_tests.rs`, `quickshape_arc_tests.rs` (novo) |
| **cena 12** | um 5.º traço SEGURO — um sorriso de 130° — antes do gelado livre (agora 6 traços, `HELD = 5`); foto [`f7c_01`](../vetor_nova_ui/fotos/f7c_01_cena12_um_arco_seguro_vira_arco_perfeito.png) | `ph2d-app-vec/src/vector_mode_quick.rs` |

### §1b — A 2.ª volta: o arco ASSIMÉTRICO (smoke do dono, 10/10)

*«Smoke OK. Arcos simétricos são desenhados com facilidade. Arcos assimétricos como na foto não
funcionam. Tente corrigir.»* — a foto: um gancho que abre largo e fecha apertado, que ficava LIVRE (o
círculo erra 8,8 % do raio). O traço da captura é `quickshape_arc::OWNERS_HOOK_PX` (pub, re-exportado
em `quickshape`), lido pela cena 12 e pelos gates — um sítio só.

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **a elipse** | o ajuste DIRECTO de Fitzgibbon–Pilu–Fisher (1999) na forma estável de Halíř–Flusser (1998), porte directo (cúbica de Cardano para os valores próprios 3×3) | `quickshape_conic.rs` (novo, geometria pura) |
| ⭐ **o biarco** | `QuickShape::Biarc([ArcShape; 2])` — dois arcos de círculo TANGENTES (Bolton, 1975), por Levenberg-Marquardt sobre `(x₀, y₀, θ₀, k₁, k₂, L₁, L₂)` a partir do círculo de Taubin, em fórmula fechada; recusado se as curvaturas têm sinais opostos (inflexão) ou passa uma volta. O caminho: os dois arcos, a junção `Smooth` | `quickshape_biarc.rs` (novo), `quickshape_arc::biarc_path` |
| **a escolha** | o CÍRCULO primeiro; o mais perto entre elipse e biarco quando o círculo não desenha o traço, ou quando o desenha mal (rms > `arc_circle_keep` 0,04 do menor de raio e tamanho) e o outro o desenha a ≤ `arc_asymmetric_gain` 0,2 do resíduo dele. `Shift` = sempre o arco de CÍRCULO | `quickshape_arc::fit` |
| **cena 12** | um 6.º traço SEGURO: o gancho do dono a meio tamanho (7 traços, `HELD = 6`); foto [`f7c_02`](../vetor_nova_ui/fotos/f7c_02_cena12_o_gancho_assimetrico_vira_biarco.png) | `vector_mode_quick.rs` |
| organização | as sondas do arco descem para `quickshape_arc_probe_tests.rs` (os gates em `quickshape_arc_tests.rs` passavam das 700 linhas) | `ph2d-vec-edit/src/` |

**Medido** ([§3g.2](../vetor_nova_ui/oraculo_quickshape/00_resultados.md)): o gancho sai biarco a 3 % do
tamanho (11 px num de 345 px; raios ~217 e ~39), em ½×, 1×, 2×, espelhado e com tremor até ±4 px; as
elipses fora do eixo saem elipse a ≤ 2,2 px; as espirais de 200–270° biarco a ≤ 4,5 px; um
reconhecimento custa 0,2–1,5 ms em release (o mínimo de 20; um por gesto). ⛔ **Recusas medidas:** a
espiral de Euler (34 px no gancho, e inverte a curvatura); trocar um círculo que JÁ serve pelo mais
perto — o arco SIMÉTRICO que respira (o da própria cena 12, ±3 %) saía biarco: a tabela guarda × ganho
sobre arcos que respiram ±2–6 % só tem uma linha a zero, a guarda 0,04 (ganho válido `0,1–0,25`).

## §2 — Medido e decidido (técnico, delegado)

Tabelas inteiras em [`00_resultados.md`](../vetor_nova_ui/oraculo_quickshape/00_resultados.md) §3g
(`cargo test --release -p ph2d-vec-edit --lib measure_arc -- --ignored --nocapture --test-threads=1`).

- **Custo nas formas antigas: ZERO** (`measure_arc_ab`, A × B no mesmo processo, 80 mãos, 24–400 px):
  rectângulo, triângulo, círculo, elipse, linha, zigue-zagues, L e S iguais ao décimo. Meio arco
  0 % → 100 / 99,7 / 74,7 % (±1 / ±2 / ±4 px).
- **Arcos de mão** (estabilizador, 40 mãos, os dois sentidos): 45°–315° a 100 % do raio 80 px para
  cima (±1 px); 120°–315° ≥ 98 % a todos os raios ≥ 40 com ±2. 30° é a fronteira com a LINHA (flecha
  6,5 % ≈ 30°). 330° FECHA em círculo (a folga de 13 % fecha a partir de ~318°).
- **Os «não é arco»** (S de dois meios círculos, S de seno, rabisco, volta e meia): 0 % lidos como arco.
- **As faixas** (`measure_arc_joint`, as três juntas — o resíduo e a viragem seguram o rabisco a
  meias): resíduo `0,05` em `0,04–0,06` (↓ o meio arco sem estabilizador ±2 cai a 86 %, ↑ rabisco 2,5 %);
  viragem para trás `0,25` em `0,15–0,35`; cordas `6` em `4–8`. As provisórias (0,15 / 8) estavam nas
  bordas — foram ao meio (a lição da F7b).
- **Taubin, não Kåsa:** com ±2 px Kåsa encolhe o raio de um arco de 45° (r 80) em −3,5 px, Taubin −0,06.
- **`regular`:** o Procreate não corre aqui (iPad); o manual só descreve o 2.º dedo para linha,
  elipse e polígonos. Escolhida a lei da LINHA aplicada ao arco (15° no varrido e na corda) — sem
  medição, dito.

**⛔ Recusas MEDIDAS** (não se reconstroem): o resíduo relativo só ao RAIO (um S raso tem raio enorme
e passava — o gate `each_guard_of_the_s_holds_alone` mostra-o com a viragem desligada); 12+ cordas (o
tremor de cada corda vira para trás); o ajuste de Kåsa (encolhe); a polilinha a ganhar sempre (o arco
de 60° saía dois lados).

## §3 — Foundational tocado e contratos

- **`ph2d-vec-edit`**: variantes PÚBLICAS novas `QuickShape::Arc(ArcShape)` e
  `QuickShape::Biarc([ArcShape; 2])`, o `pub struct ArcShape` e a `pub const OWNERS_HOOK_PX`
  (re-exportados em `quickshape`; nenhum `match` sobre `QuickShape` fora da crate — varrido com `grep`
  no workspace; lá fora só `to_path`/`pivot`/`transformed` e, no gate da cena 12, `ArcShape`); módulos
  privados `quickshape_arc`, `quickshape_conic`, `quickshape_biarc`; `Law` ganha `arc_rms_max`,
  `arc_bend_back`, `arc_chords`, `arc_asymmetric_gain`, `arc_circle_keep` (`pub(crate)`);
  `quickshape_pencil_tests::draw` passa a `pub(super)`.
- **`ph2d-app-vec`**: a cena 12 com 7 traços (6 seguros).
- **Contratos congelados (§6): nenhum.** Shell: **intocada** (`+0`).

## §4 — Superfície de colisão e contadores

Nada que some entre linhas: sem ids, chaves, cenas novas (a 12 só ganha um traço), schema. Shell `+0`.
`own()` 25 → 27 conta-se no próprio ficheiro (gate `every_own_law_exception_names_a_fixture_that_exists`).

## §5 — Gate final

Uma corrida do `verificador` (`BASE=2a925859d`, HEAD `bbf31dcac`): **fmt ✗** (19 quebras de linha em
três ficheiros de teste — curado com `cargo fmt` em `022068ae7`, `--check` limpo, testes re-corridos
35/35) · warnings-deny ✓ · clippy do ship ✓ · machete ✓ · standalone-optional ✓ · workflow-packages ✓ ·
censos 114/114 e 12/12 ✓ · `doc-index`/`adr-index` ✓ · `nextest-impacted` **17 030 / 17 030** (9 782
pulados) · catracas ✓ (`the_shell_only_shrinks`, LOC, `no_orphan_source_file`). Depois: o portão do
`CLAUDE.md` (`architecture_claude_md_cabe_no_orcamento`) 3/3 com o link novo.

**2.ª onda** (o arco assimétrico), uma corrida do `verificador` (HEAD `238e3e068`): fmt ✓ · warnings-deny
✓ · clippy do ship ✓ · machete ✓ · standalone-optional ✓ · workflow-packages ✓ · censos 114/114 e 12/12 ✓ ·
`doc-index`/`adr-index` ✓ · `nextest-impacted` **17 038 / 17 038** (9 784 pulados) — tudo verde à primeira.

### Mutação

Agente `mutacao` (verde visto antes, âncoras 1×, árvore limpa no fim): **12 de 17** à primeira — a
guarda do resíduo, a de uma volta, o sinal da viragem, o varrido e a corda do `regular`, o `Corner` das
pontas, o sinal da alça, a forma mais perto, o pivô, o `start` da rotação, o arco desligado, a cena.
Sobreviveram **5 buracos reais**, fechados com testes vistos VERMELHOS sob a mutação e verdes limpos:
a viragem para trás é a única guarda do S RASO (`each_guard_of_the_s_holds_alone`); o resíduo relativo
ao menor de raio e tamanho pára o S sozinho (idem, com a viragem desligada); o `Shift` nunca troca
polilinha por arco (`the_shift_never_turns_a_polyline_into_an_arc` — a polilinha de 3 lados com quinas
de 30° trocava); Taubin × Kåsa (`taubin_does_not_shrink_a_short_noisy_arc_as_kasa_does`, com Kåsa como
controlo); a folga de 5 % do troço (92° num troço). **17 / 17** sangram.

**2.ª onda (o arco assimétrico):** **12 de 19** à primeira (a guarda do círculo que já serve, a escolha
elipse × biarco, a régua da elipse, a junção `Smooth`, a rotação, a afinidade da elipse, o amortecimento
de Marquardt, o início do troço, a distância à elipse, a rotação da cónica, o pivô do biarco). Das 7
sobreviventes, **6 buracos reais** fechados com testemunhas ACHADAS por busca (base × mutação sobre
~980 traços): o gancho aguenta ±4 px (`the_owners_hook_survives_a_trembling_hand`); o arco que RESPIRA
±4–6 % fica círculo (`a_symmetric_arc_that_breathes_stays_a_circle`); o caracol de mais de uma volta não
é biarco; o taco nunca vira biarco com inflexão e o Π arredondado não é biarco; a alça de saída da junção;
o avanço de curvatura zero é a recta. **1 redundante retirada do código** (`cond > 0` no vector próprio:
a escolha pelo maior `cond` e a recusa da hipérbole já a fazem). **18 / 18** sangram.

## §6 — Smoke do dono

Fotos: [`f7c_01`](../vetor_nova_ui/fotos/f7c_01_cena12_um_arco_seguro_vira_arco_perfeito.png) — o
sorriso por baixo da linha é um arco de três nós; o gelado continua livre (smoke do dono: OK para os
simétricos). [`f7c_02`](../vetor_nova_ui/fotos/f7c_02_cena12_o_gancho_assimetrico_vira_biarco.png) — o
gancho do dono, em cima, é um biarco de cinco nós; o sorriso continua círculo.

## §7 — O que fica ABERTO

- **Arcos SIMÉTRICOS de elipse** (centrados no eixo) e espirais suaves (120°) continuam arco de CÍRCULO
  (até ~12 px num raio de 160): é o preço da guarda que mantém círculo o arco simétrico que respira.
- **Arcos minúsculos** (16–30 px de traço: raio 20 a 45–90°, raio 40 a 30°) podem sair polilinha (já
  saíam antes) ou biarco (antes: livres).
- O gancho do dono foi reconstruído dos nós VISÍVEIS da captura (23 pontos): o traço real dele é que diz
  se 3 % de desvio basta.
- O `regular` do arco é uma decisão sem medição (o Procreate não corre aqui).
- Do F7: setas; «Edit Shape».

## §8 — Binário de smoke (2.ª corrida)

Depois do `rm -rf target/*/incremental` (6,0 GB) — `bash scripts/ph2d-run.sh cargo build -p
ph2d-host-desktop --profile smoke`, 1.ª corrida 21,3 s; a 2.ª:

```
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

Varredura no fecho: zero processos meus (um `ph2d-host-desktop` vivo é da `line-PainterWatercolor`, na
tela real — não é desta linha).

## §9 — `agent-loop-profile.sh`

```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                262   alvo: <= 800
  ✗ cargo test : cargo check                734 : 284   alvo: <= 1,0  razao 2.6x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  36%   alvo: >= 80%
  ✗ contexto relido por passo (media)         443 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil
```
(As duas ondas — o arco e o arco assimétrico — correram na mesma janela; as sondas são `cargo test
--release` por natureza, e é delas a maior parte do `test`. As edições por script foram as de várias
âncoras com `assert` de contagem.)

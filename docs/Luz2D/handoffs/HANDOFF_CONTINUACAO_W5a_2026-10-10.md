# Handoff de continuação — Luz 2D, W5a: a fila do dono F1 · F2 · F3 (2026-10-10)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · merge-base `2a925859d` (sem rebase pendente).
- **HEAD:** o commit deste doc. Commits da onda: `81e28da5d` (gabarito F3) · `0f7467300` (placa + família) ·
  `5285e1b42` (fmt) · `63df65df1` (a semente pela grossa; sai a marca) · `5d3482cc7` (gates do interior) ·
  `8b5aa0e38` (o fundo pelo ponto de fronteira; os sobreviventes) · `5780ee03c` (tetos ao medido) ·
  `65030fadf` (o teto de lâmpadas) · este doc + fotos.
- **Estado:** W5a FECHADA (onda). F1–F3 riscados no [plano §7.1](../01_plano.md). Próxima: **W5 — janela NOVA**
  (rebote, HRC, governador acima de 1080p, Flip receber/bloquear e os ABERTOS abaixo).
- **Delta da shell: 0** (nada em `shells/`, `ph2d-render`, `ph2d-editor-core`). Foundational: nenhum.
  Contratos congelados: nenhum. `PROJECT_SCHEMA`: sem degrau.

## O que fechou

| # | o dono via | cura (medida) | gate |
|---|---|---|---|
| F1 | a sombra inteira deslocada (Low 3–4 px) | uma 2.ª C0 só dos OPACOS (cobertura + τ dentro), 8 × mais fina que a grossa no Low (`Degrau::captura_px = 0,5`); a luz direta marcha nela (o `meio` na escala da captura ligada — um `meio`, uma marcha); passo da luz direta 1 texel fino e **½ junto à borda** (onde a captura tem cobertura) | `a_sombra_nitida_nao_tem_escada`: média ≤ 1,05 px de TELA (era ≤ 0,95 texel GROSSO) |
| F2 | objeto de ~5 cm sem sombra no Low | a mesma captura + o refino pela **CURVATURA** das sondas (2.ª diferença > limiar: o vale de um oclusor mais estreito que a célula); a forma menor que o texel grosso entra na grossa como cobertura (dita no relatório) e semeia o campo | `o_oclusor_fino_faz_a_sombra_do_gabarito` (duas orientações × ponto e lâmpada pequena × 3 níveis) |
| F3 | linha escura de 1 px na face acesa | **preenchimento**: o texel fino DENTRO de um opaco calcula-se no ponto da FRONTEIRA mais próximo (janela 5×5 junto à borda; JFA do complemento em células de 4 na grade inteira para o fundo; bissecção do `meio`; visibilidade medida); a c0 marca no alfa as sondas de dentro e a C3 interpola só as de fora. **A mesma regra no gabarito** (`fora_dos_opacos`) | `a_face_acesa_da_parede_nao_tem_linha_escura` · `o_interior_de_um_opaco_e_o_exterior_mais_proximo` (faixa, fundo, parede grossa) · os 3 do gabarito |

Também: o teto de lâmpadas diretas **6 → 7** (medido, abaixo).

## Antes → depois (rodada `medida_luz2d_w5a`, RTX 5060 Ti, `smoke`, 1920×1080, vista de fábrica)

| régua | W4b | W5a |
|---|---|---|
| média da borda, contorno 50 % · Low · Medium · High | −4,21 · −2,13 · −1,21 px | **−0,79 · −0,73 · −0,72** |
| degrau rms 2 · 10 · 50 · 90 · 98 % (Low) | 0,163 · 0,105 · 0,085 · 0,139 · 0,182 | 0,169 · 0,115 · 0,053 · 0,105 · 0,168 |
| barra de 5 cm, pior erro massa · fundo (Low) | 1,000 · 1,000 (não fazia sombra) | **0,026 · 0,027** |
| face acesa da parede, pior placa ÷ gabarito (Low · Medium · High) | 0,446 · 0,456 · 0,446 | **0,983 · 0,992 · 0,992** |
| custo Low sem lâmpada · por lâmpada (`load` ~2) | 1,31 · 0,466 ms | **1,57 · 0,470** (captura fina 0,06 · preenchimento 0,17 · C3 +0,03) |
| Low com 6 · 7 · 8 lâmpadas diretas | 4,10 · 4,50 · 6,16 ms | 4,39 · **4,83** · 6,43 ⇒ teto **7** (era 6) |
| memória da luz Low | 40,8 MB | 105,7 MB (a captura ×8: 63 MB) |
| Vello da camada do vetor (20 caminhos, Low) | 0,23 ms | 0,50 ms (a 8 px por texel grosso) |

Fotos: `fotos_W5a/cena{1,2,4}_{low,medium,high}.png` (`fotografa_cena.sh`, `HOME` próprio com `light_quality`,
`light_auto=0`); recortes ampliados 6–7× antes (W4b, em cima) × depois: `fotos_W5a/cena4_canto_antes_depois_W5a.png`
(a sombra sai do canto exato) e `fotos_W5a/cena1_face_acesa_antes_depois_W5a.png` (a coluna escura sumiu).

## ⛔ Recusas MEDIDAS (W5a)

| tentativa | número | por quê |
|---|---|---|
| captura de 1 px (= a grade fina) | média −1,16 px; a lasca do ponto alarga 11 % | a de ½ px: −0,79 e 2,6 % |
| passo de 1 texel fino em tudo com a captura fina | degrau Low 0,237 · 0,194 (2 · 10 %) | o degrau é do PASSO (sonda `medida_causa_do_desvio`), não da interpolação |
| passo de ½ em tudo | a mesma escada a +75 % por lâmpada | ½ só junto à borda: +7 % |
| ¼ junto à borda | 0,164 · 0,103 a +28 % | sem ganho sobre o ½ |
| sem a curvatura | fundo da lasca 0,257 (Low) | o vale mais estreito que a célula tem os cantos iguais |
| a semente do campo a ler a captura fina (256 texels por célula num fio) | C1 0,03 → 0,71 ms | a forma menor que o texel entra na GROSSA como cobertura; captura ≤ 8 subamostras (`Config::validar`) |
| uma MARCA paralela da cobertura fina para a semente | 0,17 ms | redundante pelo mesmo motivo — saiu (`63df65df1`) |
| cobertura mínima ε nos shaders da C0 (para semear) | sprite × quad 0,0000 → 0,0027; vetor × polígono 0,0099 | semeava polígonos e sprites de forma diferente — quebrava duas leis exatas |
| preenchimento só na região | 0,05 ms (grade inteira 0,17) | a parede cortada pela borda da janela acharia a outra face e mudaria de cor no pan |
| células de 1 · 2 | face acesa igual à de 4 | 0,25 · 0,29 ms contra 0,17 |
| o texel exterior mais perto (em vez do ponto de FRONTEIRA mais perto) | interior 0,83 (penumbra) | no canto o eixo medial chega à borda e a face escolhida troca |
| só as sementes do JFA de células de 4 | interior 1,15 | a semente de uma célula pode ficar na face errada |

## Prova de mutação (agente `mutacao` + as três repetidas à mão)

Pré-voo: 17 âncoras casaram 1×; filtros povoados (placa 11 · família 7; as 1.ªs corridas de M1–M4 com 0
testes foram descartadas e refeitas); restauro por `checkout` + `touch` (+ `shaders.rs`); árvore = HEAD.

| # | mutação | resultado |
|---|---|---|
| M1 | `escala_da_captura` → 1 | **sangra** — borda −4,39 px; barra massa 0 |
| M2 | o `meio` sem a escala | **sangra** — 4 gates (escada, vetor, sprite, barra) |
| M3 | a forma menor que o texel fora da grossa | **sangra** — barra massa 0 (o campo salta-a) |
| M4 | a captura fina sem oclusores | **sangra** — 5 gates |
| M5 | a captura fina sem sprites | **sangra** — vetor 2,78 · sprite 3,03 · fenda 1,00 |
| M6 | a camada do vetor sem `× captura` | **sangra** — vetor 0,0433 |
| M7 | sem a curvatura | **sangra** — fundo da lasca 0,433 × 0,168 |
| M8 | sondas da região com folga 1 | **sangra** — texel (442, 101) muda |
| M9 | sem o passo junto à borda | **sangra** — escada 0,358 px |
| M10 | sem o preenchimento no texel | **sangra** — face 0,450; interior fundo 0,0146 |
| M11 | a c0 com as sondas de dentro | sobreviveu → **teto da grade fina ao medido** (céu e vidro 0,347; a mutação 0,433 > 0,382) — `5780ee03c` |
| M12 | `na_fronteira` → o centro do texel | **sangra** — fundo 0,0056 |
| M13 | sem a janela 5×5 | **sangra** — fundo 0,0050 |
| M14 | o salto do JFA parado | **sangra** — parede grossa 0,0524 |
| M15 | preenchimento só na região | **sangra** — texel (52, 106) muda |
| M16 | o remapeado sem a visibilidade medida | sobreviveu (a face de fora estava na sombra) → **uma lâmpada fora da região** acende-a: o texel (37, 108) muda — `8b5aa0e38` |
| C1 | `<` → `<=` na semente do preenchimento | sangrou (0,0035: o desempate decidia a face) → **a cura foi no algoritmo**: entre as 9 sementes, a de ponto de FRONTEIRA mais perto; agora equivalente (0,0014 · 0,0002 · 0,0006, dentro dos tetos) |

## Portão de fecho

`fmt --check` ✓ · clippy `-D warnings` (`ph2d-light2d-gpu`, `ph2d-app-light2d`, `ph2d-light2d-ref`, `ph2d-light2d-ecs`,
`ph2d-host-desktop`) ✓ · `machete` ✓ · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ ·
`nextest-impacted` **16 917 / 16 917** ✓ (sobre `0f7467300`/`5285e1b42`). 2.ª corrida, sobre `65030fadf` + `a993819b8` (um `mut` que sobrou, apanhado nela): fmt ✓ · clippy ✓ ·
machete ✓ · deny-check ✓ · `nextest-impacted` 16 916 / 16 917 — o vermelho é o G4 (K3), ver o ABERTO 3 (não é
desta onda, medido alternado). Suítes da família: placa 21 + 3 ignorados · família 39 + 8 · gabarito 20 + 1 ·
ECS 12. Smoke recompilado (regra I, depois do `rm -rf target/*/incremental`), 2.ª saída:
`Finished \`smoke\` profile [optimized] target(s) in 0.20s`.

## ⚠️ ABERTO (W5a → W5)

1. **A luz das CASCATAS sai baixa junto a um opaco** (céu, ambiente): na «céu e vidro», sem o preenchimento, o
   exterior vai de 0,17 a 0,01 em ~2 texels grossos onde o gabarito dá 0,25 (meio céu) — as sondas de cima de
   dentro da parede entram na fusão com 0. O preenchimento copia esse exterior para dentro. Território do HRC.
2. **O interior de uma parede que RECEBE luz** mostra a face mais próxima: a metade do lado aceso clara, a outra
   escura, com o corte no EIXO MEDIAL (onde «a face mais próxima» troca num salto; o gate do interior mede a
   faixa de 2 texels e o fundo pela média, não esse fio). As cenas 1 e 4 continuam com `NaoRecebeLuz` nas
   paredes — mudar é decisão de PRODUTO (pergunta ao dono no relatório).
3. **G4 (K3) na beira do limite — herdado, não desta onda.** Medido ALTERNADO no mesmo regime (`load` ~2,4; o
   HEAD da W4b numa worktree temporária com `target` próprio): **W4b 4,37 · 4,82 · 4,41 · 5,09 %** × **W5a
   3,91 · 3,59 · 3,94 · 4,16 %** (limite 5 %). O próprio HEAD da W4b reprova às vezes; sob carga, as duas.
   O que varia é o RC das 10 000 luzes (+0,2 ms: a entrada nos emissores). Decidir na W5: medir a causa
   (`marchar` ao entrar num emissor) ou rever a régua do K3 — nunca subir o número sem a medição.
4. A cena 2 (1 000 vaga-lumes) corre a 18 fps no roteiro da foto em todo nível — **igual na W4b** (não é da luz).
5. Os da W4b que continuam: as cascatas na grade inteira (81,5 % fora da janela), a região C3 pelo canvas, o K7
   que conta texturas inteiras (agora também as da captura fina e do preenchimento).

## O que a W5 precisa saber

- `Config::captura` (texels da captura dos opacos por texel grosso, ≤ 8) vem do `Degrau::captura_px` pela
  `captura_do_quadro`, como a `fina` vem do `fino_px`. A captura fina é `Solver`/`solver_fina.rs`; a C3 liga-a
  no grupo 0 e o `meio` lê-a na escala `ESCALA_DA_CAPTURA` (override; `COM_LUZ = false` na C3).
- `passo_direto` é em texels FINOS (nas voltas 1–2 da rodada contava-se em texels da captura: «½ px · passo 2»
  = o passo 1 de hoje). `passo_na_borda` só vale na C3 (as cascatas ficam no 1).
- O preenchimento: `c3_preenche.wgsl` (o JFA, grupo 3) e `onde_calcular`/`na_fronteira` em `c3_direta.wgsl`;
  `MASCARA_C0` (c2) põe o alfa da c0. A régua do interior é `comum::dentro_de_opaco_a_menos_de`.
- G3 e o gate da grade fina medem FORA dos opacos (dentro a lei é o preenchimento, com gate próprio).
- A rodada `medida_luz2d_w5a` (A custo com 0·1·4·6·7·8 e «6 junto à parede» · B escada · D face · E barra fina ·
  F Vello) é a régua da W5a; as variantes são configuração em execução. ⚠️ O custo por lâmpada NÃO é linear:
  a 7.ª custa 0,44 ms, a 8.ª (junto à parede) 1,6 — meça o teto com N lâmpadas, nunca pela média.
- A semente do campo de distância é a captura GROSSA: `Config::captura ≤ 8` (as subamostras da C0) garante
  que o que a fina possui a grossa vê; acima disso volta a ser preciso ler a fina (recusas acima).

## Smoke do dono (W5a) — o que mostrar

As cenas 1 e 4 (`PH2D_LIGHT2D_SMOKE=1` · `=4`) em Low · Medium · High (Settings ▸ 2D Light, Automatic
desligado): a sombra nítida sai do CANTO da parede (antes começava abaixo dele, mais no Low), sem degraus;
a face acesa da parede sem a linha escura. O F2 (objeto de 5 cm) não tem cena de smoke — a régua é o gate
`o_oclusor_fino`; para o dono ver, desenhar um retângulo fino com «Bloqueia luz» perto da lâmpada da cena 1.

## Smoke do dono (10/10): «Smoke OK. No Inspector o custo é igual para qualquer qualidade»

**Mecanismo:** o custo de placa que o Inspector mostra só se lia para o GOVERNADOR — com o «Automatic»
desligado (como manda o passo do smoke) o número ficava congelado no último nível automático. **Cura**
(`b8bf470c6`, `fb3f3532d`): `LuzDoQuadro::gravar` lê os carimbos sem esperar sempre que há preferência do
editor (as sondas, sem ela, continuam com a leitura bloqueante só delas); só o governador depende do
Automático; ao recriar a grade (outro nível ou janela) o custo da anterior esquece-se. Gate vermelho-primeiro
`com_o_automatico_desligado_o_custo_e_o_do_nivel_escolhido` (1080p: Low 2,3 · High 37,7 ms, sob carga) — as
duas metades sangram na mutação (sem o filtro antigo: «nunca foi lido»; sem o esquecer: «ainda mostra o custo
do anterior»). Portão: clippy · deny-check · família 40 + 8 ✓.

## Smoke do dono (10/10, 2.ª volta): «ao dar zoom e pan o aspecto da iluminação muda» — F4, para a W5

**Medido** (`a_luz_num_ponto_nao_muda_com_a_vista`, cena 4 sem a ambiente, três pontos à direita da parede
pequena, na sombra da lâmpada pequena): vistas de fábrica, 4 m e 2,5 m → 0 · 0 · 0 (o gabarito 0 · 0 · 0); vista
de 1,5 m (grade x ∈ [−5, 0]) → **0 · 2,45 · 2,38** (0,176 do máximo). **Mecanismo:** a lâmpada DIRETA (C3)
ilumina de qualquer distância, mas a captura só tem os opacos DENTRO da grade (vista + margem): o raio até a
lâmpada grande cruza o muro do meio num trecho fora da grade e passa. (As cascatas não têm o defeito: um
emissor fora da grade também não existe para elas — a margem é o desenho delas.) O gate está com `ignore`
(vermelho visto) e é o **F4** do plano §7.1, com três curas candidatas a medir. Não curado nesta janela:
é uma mudança de desenho (como a luz direta vê o mundo fora da grade) que pede rodada própria.

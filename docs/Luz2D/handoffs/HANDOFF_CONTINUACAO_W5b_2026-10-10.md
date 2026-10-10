# Handoff de continuação — Luz 2D, W5b: F4 — a luz não muda com a vista (2026-10-10)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · merge-base `2a925859d` (sem rebase pendente).
- **HEAD:** o commit deste doc. Commits da onda: `cab079373` (a cura) · `8a0c4c40e` (a bissecção do disco que
  escondia a lâmpada; a régua estendida) · `b2f186ac9` (carimbo das máscaras; G5) · `e164de0c3` (a C3 em par; a
  rodada) · `f3e29b1d4` (a régua só lê a janela; translúcidos; 2.ª máscara; testes de CPU) · `…` (os
  sobreviventes de CPU; A3; `fotos_do_f4`) · `0bcc38e53` (fotos) · `b33d78bad` (doc) · `9d2896cfa` (o portão: LOC e
  clippy) · este fecho.
- **Estado:** W5b FECHADA (onda). F4 riscado no [plano §7.1](../01_plano.md). Próxima: **W5 — janela NOVA**
  (rebote, HRC, governador acima de 1080p, Flip receber/bloquear, formas do Motion bloquearem e os ABERTOS).
- **Delta da shell: 0** (nada em `shells/`, `ph2d-render`, `ph2d-editor-core`). Foundational: nenhum. Contratos
  congelados: nenhum. `PROJECT_SCHEMA`: sem degrau.

## O que fechou

| o dono via | causa medida | cura | gate |
|---|---|---|---|
| zoom e pan mudam a luz (cena 4: à direita da parede pequena, preta numa vista e cinza na outra) | a luz direta (C3) ilumina de qualquer distância, mas a captura só tem os opacos DENTRO da grade: o raio até a lâmpada grande cruzava o muro FORA dela | **o mundo fora da grade** — abaixo | `a_luz_nao_muda_com_a_vista.rs` (6) · `g5_o_mundo_fora_da_grade_e_contado` · `distantes::testes` (5) |

**O desenho** (`ph2d-light2d-gpu`: `distantes.rs` CPU · `solver_distantes.rs` placa · `c3_mascara.wgsl` ·
`c3_direta.wgsl::fora_da_grade`/`cone_livre_fora`; `ph2d-app-light2d/src/vetor.rs::Distante`):
- por lâmpada direta cujo disco não cabe na grade, os opacos que tocam o FECHO CONVEXO (grade ∪ caixa do disco)
  e não cabem inteiros nela; a lista vai na própria `Lampada` (`lista`, 48 B);
- cada um na sua MOLDURA: o **polígono** da cena pelas arestas, exato (interior à esquerda; × τ a cada ENTRADA);
  o **sprite** pelo quad + uma MÁSCARA do alfa na moldura dele (`R8Unorm`, atlas de janelas, 4×4 subamostras; a
  lei da captura — bilinear ≥ ½ — à resolução do OBJETO; textura+UV iguais partilham a janela); o **caminho
  vetorial** chega como sprite (`CapturaDeSprites::distantes`) sobre um desenho do Vello, uma janela por caixa
  (`path_bounds_under`, a lei de limites do desenho avulso), só os que `tapa_fora_da_grade` diz;
- a C3 testa-os só no trecho do raio FORA da grade (`saida_da_grade`; dentro manda a captura) e o cone livre ganha o
  teste conservador (trapézio × paralelogramo, eixos separadores — tocar só obriga a medir);
- a C3 compila-se em **PAR** (`DISTANTES` sem/com); o quadro sem objeto distante corre a sem;
- as cascatas NÃO mudaram.

## Antes → depois (rodada `medida_luz2d_w5b`, RTX 5060 Ti, `smoke`, 1920×1080, ⚠️ `load` 8–15 na rodada final)

| régua | W5a | W5b |
|---|---|---|
| o caso do dono (cena 4, a vista de 1,5 m, pan de 9 vistas; da luz local) | 1,5246 (0,176 do máximo da cena) | **0,0000** |
| o gate (pan de 34 vistas, 2 alturas; pior ponto, da luz local): muro · parede da grande disco · ponto · fenda do sprite · vetor · translúcido | — | 0,0000 · **0,0032** · 0,0004 · 0,0044 · 0,0044 · 0,0003 |
| Low, vista de fábrica (nada distante), 2 · 8 lâmpadas (A2, 25 rodadas) | 3,761 · 8,682 ms | 3,759 · 8,510 |
| Low, vista do dono, 2 · 7 · 8 lâmpadas (sprites) | 4,47 · 6,97 · 6,71 | 4,04 · 5,79 · 5,87 |
| High, vista do dono, 7 lâmpadas | 27,13 (as sombras erradas) | 33,97 (a C3 fina mede onde a sombra agora existe) |
| máscaras (3 objetos, 1 máscara) · 256 · 512 | — | 0,008 · 0,016 ms |
| N caixotes fora da grade no caminho (A3, Low, dono): 0 · 10 · 50 · 200 → objetos 2 · 6 · 23 · 84 | — | 4,005 · 3,974 · 4,004 · 3,976 ms |
| memória da luz Low | 105,7 MB | 105,7 MB (+ 64 KB por máscara, + a lista) |

Fotos: `fotos_W5b/cena4_dono_antes_depois_W5b.png` (a luz recebida pelo chão, `fotos_do_f4`: o app não abre
noutra câmera) e `cena4_fabrica_{antes,depois}.png` (IGUAIS byte a byte); `cena{1,2,4}_{low,medium,high}.png`
(`fotografa_cena.sh`, `HOME` próprio com `light_quality`, `light_auto=0`; roteiro no scratchpad, reescrito).

## ⛔ Recusas MEDIDAS (W5b)

| tentativa | número | por quê |
|---|---|---|
| (a) mapa de sombra polar por lâmpada (o Godot) | o piso, com resolução INFINITA: 0,343 da luz local na penumbra da lâmpada de 0,6 m | vê do CENTRO: a lâmpada de disco vira um ponto |
| (b) 2.ª captura grossa e conservadora do caminho | o piso (o opaco dilatado ½–1 texel): 0,0031–0,0061 com a lâmpada a ~6 m (2048², 3,3 mm) · **0,023–0,046 a 50 m** | o texel é a CAIXA ÷ resolução: piora com a distância da lâmpada, e salta ao entrar na grade |
| a C3 só com `DISTANTES` (uma pipeline) | +2,5 % · +6,1 % no Low (2 · 8 lâmpadas), +1,3 · +2,3 % no High, com as listas VAZIAS | o código custa mesmo sem correr ⇒ o par |
| máscara de 16 · 32 · 64 · 128 · 256 · 512 | a fenda de 1/15: 0,041 · 0,0043 · 0,0049 · 0,0043 · 0,0043 · 0,0043 | 256 (o padrão) resolve um detalhe de 1/256 do sprite a 0,008 ms e 64 KB; 512 dobra o custo sem mudar a régua |

## Prova de mutação (agente `mutacao`; pré-voo das âncoras, filtro povoado 6 + 5, restauro `checkout` + `touch`)

18/18 **sangram** (a cura desligada · a τ das arestas · ENTRADA × saída · a τ da máscara · o limiar ½ · partir de
dentro · a saída da grade · o cone conservador · o `retain` do que cabe na grade · a janela da máscara · a τ
copiada · a lâmpada que atravessa a borda · o par `de(false)` · a bissecção com o contador · a lista zerada · o
`tapa_fora_da_grade` · a caixa do vetor · a largura do cone). Controlos equivalentes (`v ≤ 1`; passo da máscara
0,45) ficam verdes. Dois só sangravam na GPU (a τ copiada; `any` × `all` da lâmpada) → gates de CPU novos, vistos
a sangrar.

## Portão de fecho

1.ª corrida (sobre `0bcc38e53`): fmt ✓ · machete ✓ · deny-check ✓ · **clippy ✗** (`redundant_iter_cloned` em
`quadro.rs`) · **nextest ✗** (`workspace_src_files_under_loc_cap`: `solver.rs` 701 > 700 — curado MOVENDO o grupo das
lâmpadas para o `solver_distantes.rs`, 682). 2.ª corrida, sobre `9d2896cfa`: `fmt --check` ✓ · clippy `-D warnings`
(`ph2d-light2d-gpu`, `ph2d-app-light2d`, `ph2d-light2d-ref`, `ph2d-light2d-ecs`, `ph2d-host-desktop`) ✓ · `machete`
✓ · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ · `nextest-impacted` **16 930 / 16 930** ✓
(`load` 17–45). Suítes da família: placa 22 + 3 ignorados · lib 5 · família 46 + 12 · gabarito 20 + 1 · ECS 12.
Smoke recompilado (regra I, depois do `rm -rf target/*/incremental`), 2.ª saída:
`Finished \`smoke\` profile [optimized] target(s) in 0.26s`.

## ⚠️ ABERTO (W5b → W5)

1. **A3 não é o pior caso:** na cena da medida o muro tapa primeiro e o laço por objeto pára cedo. Uma cena com
   muitos sprites fora da grade e NENHUM a tapar por inteiro (o raio testa todos) não foi medida; se custar, a cura
   é uma divisão dos objetos por ângulo a partir da lâmpada (a lista por fatia), não um teto.
2. Dois opacos TRANSLÚCIDOS que se sobrepõem fora da grade contam uma entrada cada (na captura, dentro da grade,
   fundem-se numa só). Nenhum caso do dono; a lei do gabarito (por polígono) é a de fora.
3. O translúcido que ATRAVESSA a borda: a régua tem o muro a ½ nas duas alturas (o M3 «saída × entrada» sangra
   nele: 0,0806), mas não um caso desenhado só para isso.
4. O `a_nitidez_da_luz::com_o_automatico_desligado…` (High > 4 × Low) reprovou uma vez DENTRO da suíte (outros
   testes de placa no mesmo processo), passa sozinho — candidato a [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md).
5. Os da W5a que continuam: a luz das cascatas baixa junto a um opaco (HRC); o G4 (K3) na beira; a cena 2 a
   13–18 fps no roteiro da foto; as cascatas na grade inteira; o K7 que conta texturas inteiras.

## O que a W5 precisa saber

- `Config::distantes` (por omissão `true`) e `Config::mascara_px` (256) são configuração em execução — a rodada
  compara-os no mesmo processo. `RelatorioC0` diz `distantes`, `mascaras` e `lado_da_mascara` do quadro.
- Os recursos dos distantes nascem com o mínimo que a `PassList` declara e CRESCEM (são trocados: a contagem no
  alocador fica 1, mudam os bytes); `bytes_da_geometria_residente` conta só o que cresceu.
- A régua do F4 lê só a JANELA de cada vista (o resto da grade fina guarda o valor velho): um ponto fora dela lê
  lixo — e no caso opaco o zero velho «acertava». Cada caso confere que mediu cada ponto com o opaco fora.
- ⛔ WGSL deixa um laço interno ESCONDER um parâmetro com o mesmo nome (`for (var k …)` dentro de uma função de
  `k`): compila e lê a lâmpada errada. Custou a 1.ª volta (a penumbra +24 %).
- O teto `LAMPADAS_DIRETAS_MAX = 7` não foi re-medido: no Low a vista do dono com a cura custa MENOS que sem ela.

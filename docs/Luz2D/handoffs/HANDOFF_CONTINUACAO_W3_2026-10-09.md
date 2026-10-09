# Handoff de continuação — Luz 2D, W3 → W4 (2026-10-09)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · merge-base `2a925859d` (main de 07/10, sem
  rebase pendente) · **HEAD:** o commit deste doc (topo da branch).
- **Commits da W3:** `7cee7d124` luz_em no gabarito · `15434d869` receber (sprite + colagem do vetor, G7) ·
  `b6034e853` a família no quadro (G6, G11, paridade, NaoRecebeLuz) · `871b74f99` captura por texel ·
  `27523a8f6` o vetor que bloqueia + censo · `ce180d74c` o degrau medido · `33637b54e` o portão de
  fecho · `65c911634` o gate do passe isolado (da mutação) · este doc.
- **Estado:** W3 FECHADA (onda, não a linha). Próxima: **W4 — o artista** (menu Add, Inspector, gizmo,
  paleta, cenas 1–4 fotografadas, smoke do dono). **Janela nova.**

## Desenho — onde o solver entra no quadro e quem o possui (escrito ANTES do código)

Leitura do quadro de hoje (`shells/desktop/src/render_loop/`): `present.rs::present_world_sprites` →
[faixas de baixo] → `SpriteRenderer::render_with_streams` (`ph2d-render/src/renderer_draw.rs`) → Flip →
emissivo/glow (`present_fx.rs`, re-desenha sprites ISOLADAS pelo MESMO pipeline) → `present_chrome.rs`:
tonemap (hoje `BYPASS_LUT` = `clamp`) → faixas de cima → Vello (chrome **e** documento na MESMA cena, salvo
em faixas) → compositor. O documento só sai da cena do chrome no modo FAIXAS (ADR-0154).

| peça | onde | dono |
|---|---|---|
| coleta + guarda + grade ancorada | `fase_vector_bands` (a codificação) chama `codificar` (cena de smoke armada · `preparar` → `luz_acesa` · `desenhar_vetores`) | `ph2d-app-light2d::LuzDoQuadro` (crate nova) — a shell guarda UM campo (`AppGfx::luz2d`) e chama 3 portas: `codificar`, `gravar`, `ligar` |
| solver | `present_world_sprites`, ANTES do 1.º passe de sprite: `gravar` (encoder e submit próprios) e `ligar` | idem |
| `receber.wgsl` (`luz_em`, `luz_receber`) | `ph2d-render/src/shaders/` — concatenado à frente do sprite E da colagem | `ph2d-render` (foundational) |
| sprite recebe | `@group(3)` em TODAS as pipelines de sprite, ligado uma vez por passe (o idioma do grupo da PELE); NEUTRO (`ligada = 0`, ramo não tomado) sem luz | `SpriteRenderer::receber_luz` |
| vetor recebe | acesa ⇒ o quadro vai às FAIXAS (`doc_bands_of(.., forçar)` + `plan.banded |=`) e a colagem `BandSource::Vector`/`Formas` faz `linear(albedo) × luz_em → tonemap_mapear → sRGB`; a grade do chrome usa a fonte nova `Grade` (nunca acesa) | `BandBlit::receber_luz` |
| «não recebe luz» | bit 31 do `flip_uv` (tirado do índice de malha: 23 bits = 8 M malhas), posto na coleta pelo `SimRef`; a interface de fora (barras do HUD, tinta da máscara) marca-se no `present.rs` | `ph2d-render::sprite_collect` |
| glow/emissivo | o passe isolado liga o grupo NEUTRO (a emissão É luz) | `render_lifted_instances(.., com_luz)` |

**Guarda (G6) — decisão desta onda:** a cena acende **sse tem ao menos uma `Luz`** (Ponto ou Ambiente,
mesmo a zero — `CenaDeLuz::luzes`). O plano dizia «`Luz` **ou** emissor»: medido, `emissive_smoke.rs` e
todo projeto com `SpriteEmissive` ficariam ESCUROS no dia em que isto entrasse. A escuridão ambiente é a
`Luz` Ambiente (o `CanvasModulate` do Godot).

## O que fechou

- **Luz recebida** (`luz_em = F/2π`, o forno branco — plano §2.3): gabarito `ph2d-light2d-ref::luz_em`
  (3 fórmulas fechadas), shader `receber.wgsl`, o sprite multiplica antes do tonemap, o vetor na colagem
  com a MESMA conta (`tonemap_conta.wgsl` — o tonemap virou UMA função que o passe e a colagem usam, com o
  LUT ligado aos dois).
- **Captura por texel** (o gap da W1): `c0_dono.wgsl` (a lei do dono, um texto) + `c0_sprite.wgsl` — a
  caixa do sprite na grade, 8×8 subamostras da textura; `CenaDeLuz::sprites` aponta o quad que a placa
  troca; a família monta-os das instâncias do `PresentWorld` (o mesmo quad/UV/textura do passe).
- **O vetor que bloqueia**: `CenaDeLuz::vetores` (identidade do caminho); a família desenha os caminhos
  com `BloqueiaLuz` pelo Vello (`draw_path_standalone`, a porta do tile de uma forma) numa camada
  ALINHADA à grade, 1 px por texel, lida EXATA (`Amostragem::Exata`).
- **Ancoragem (G11)**: texel = potência de 2 em metros (só a oitava de zoom troca a rede); dimensões
  fixas por janela (o zoom não realoca); origem encaixada na rede da cascata de topo. `Solver::reposicionar`.
- **Ponto neutro (G6)** e **família**: `crates/ph2d-app-light2d` (registada por `ph2d-app-sync`).
- **Cena de smoke 1** (`PH2D_LIGHT2D_SMOKE=1`, «Uma luz»): o gate das famílias EXIGE um roteador
  alcançável (`every_registered_family_declares_a_reachable_router`) — a cena existe, encenada pela
  porta `codificar` uma vez por sessão, e o gate `smoke_cena_1` prova pelo gabarito o que ela ensina
  (luz no Verde atrás da parede 0,037 ≈ a ambiente 0,042; Vermelho 0,663; Azul 0,170). ⚠️ NÃO foi
  fotografada nem entregue ao dono: o roteiro, a foto e as cenas 2–4 são da W4.

## Gates (todos verdes; `PH2D_GPU=1`; RTX 5060 Ti, Vulkan)

| gate | medido | tolerância | controlo (tem de reprovar) |
|---|---|---|---|
| G6 sem `Luz` = quadro de sempre (sprite + tonemap + colagem do vetor) | 0 bytes | 0 | `Luz` a zero muda > 100 bytes |
| G7 sprite × vetor espelhado, luz uniforme · gradiente | 0 B · 1 B (vetor × CPU 0,54 B) | 1 B | colagem sem o caminho: > 20 B |
| G11 pan sub-texel, ponto estático | ancorada **0** (padrão) · 8,1e-15 (2 px) | 1e-9 | solta 4,9 · 2,6 |
| paridade: placa × conta da CPU | 1,21e-3 · 1,23e-3 (rel., pior pixel) | 1,36e-3 | — |
| paridade: placa × gabarito (RMSE rel., 1 786 pts) | Alto **0,0475** · padrão **0,0755** | +10 % | — |
| NaoRecebeLuz (cena e HUD) | o marcado intacto, o outro apaga | exato | — |
| captura: opaco rodado/espelhado × quad | 0,0000 | 1e-3 | — |
| captura: fenda × duas metades | 0,0131 | 0,0145 | o quad: 0,611 |
| captura: duas cores (direito e espelhado) | 0,0036 | 0,004 | — |
| vetor: caminho rodado × polígono (Alto) | 0,0000 | 1e-3 | sem a camada: 1,38 |
| censo WGSL partilhado (`luz_em`, `luz_receber`, `tonemap_mapear`, `c0_alvos`) | 1 ficheiro cada | — | — |
| o passe isolado (brilho) não recebe a luz | bytes iguais aos de um renderer sem luz | exato | `com_luz = true` difere |
| cena de smoke 1 (gabarito) | Verde 0,037 (ambiente 0,042) · Vermelho 0,663 · Azul 0,170 | Verde ≈ ambiente; os outros > 3–5× | — |
| W2 (G3 · G4 · G5 · G10 · G12 · interior) | os MESMOS números da W2 (G3 penumbra 0,0902…) | — | — |

⚠️ **Ordem:** o G7 foi escrito DEPOIS do código da colagem (o briefing pedia antes). O vermelho está
provado pelo controlo do próprio gate e pela mutação (abaixo).
⚠️ **«Byte a byte com o de HOJE»:** o G6 compara a família presente × ausente no MESMO binário; que o
shader novo com o grupo neutro é o de hoje prova-o a suíte de pixel do `ph2d-render` que já existia
(`blend_mode_regression`, `a_amostragem/a_mistura_do_device_chega_ao_pixel`, `tonemap_descent_gpu`
byte-exacta, `clip_children_regression`…): 269 verdes com o grupo neutro. Não comparei contra um binário
da base.

## A rodada (perfil `smoke`, `loadavg` 11–12; A, C e E pelo relógio da PLACA, B e D de parede)

Sonda: `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it
medida_luz2d_w3 -- --ignored --nocapture`. Variantes no mesmo processo, 15 rodadas intercaladas, MÍNIMO.

**A. Degrau a 1920×1080** (K7: ≤ 3 GB/s a 60 Hz = 50 MB/quadro):

| c0 px | margem | grade | ms | MB/quadro | GB/s | K7 |
|---|---|---|---|---|---|---|
| 2 | 0 · 0,125 · 0,25 · 0,5 | 992×576 … 1984×1152 | 4,85 · 8,82 · 11,10 · 16,44 | 115 · 228 · 319 · 530 | 6,9 · 13,7 · 19,2 · 31,8 | ✗ |
| 3 | 0 · 0,125 · 0,25 · 0,5 | 672×416 … 1344×832 | 3,36 · 4,16 · 4,86 · 9,88 | 56 · 86 · 115 · 259 | 3,4 · 5,1 · 6,9 · 15,6 | ✗ |
| **4** | 0 · **0,125** · 0,25 · 0,5 | 512×320 · **640×384** · 768×448 · 992×576 | 1,87 · **2,29** · 2,73 · 3,85 | 33 · **49** · 69 · 115 | 2,0 · **2,96** · 4,2 · 6,9 | ✔ ✔ ✗ ✗ |

⇒ **`DEGRAU_PADRAO` = texel 4 px, margem 12,5 %** (o recurso que limita é a BANDA; o palpite inicial de
2 px / 25 % media 19 GB/s e 11 ms). Em janelas maiores o K7 reprova — é o governador da W5.

**B. Quadro a 1080p** (300 sprites, 20 retângulos vetoriais): sem `Luz` **0,36 ms** · com `Luz` (30
bloqueiam, 5 brilham, 5 caminhos) **2,83 ms**. **C. Captura por texel** (C0): 0 · 100 · 1 000 sprites
64×64 = 0,007 · 0,016 · 0,070 ms. **D. Camada do vetor** (20 caminhos, lida exata): 1 · 2 · 4 px por
texel = erro 0,0012 · 0,0012 · 0,031, **0,27** · 0,29 · 0,69 ms (lida bilinear, a 1.ª redacção: 0,066 a
1 px). **E. Margem:** à zoom da rodada (100 px/m) a oitava já dá cobertura extra — a lâmpada a 0,5–5 m
fora entra mesmo com margem 0; a 9 m só com margem ≥ 12,5 %. A margem é o MÍNIMO garantido (no pior
ponto da oitava); a oitava dá até o dobro.

## Prova de mutação (agente `mutacao`)

Pré-voo: cada filtro verde e com população (`g6_` 2 · `g7_` 2 · `paridade_luz_recebida` 2 ·
`a_amostragem…` 1 · `blend_mode_regression` 5); cada âncora casou 1×; restauro com `checkout` + `touch`.

| # | mutação | resultado |
|---|---|---|
| M1 | sem a guarda (`luzes == 0` → `false`) | **G6 sangra** («uma cena sem `Luz` não acende») |
| M2 | o grupo aceso também no passe isolado | sobreviveu ⇒ **gate novo** `o_passe_isolado_do_brilho_nao_recebe_a_luz` (`65c911634`): agora sangra |
| M3 | o grupo NEUTRO com `ligada = 1` | G6 e G7 sobrevivem (os dois lados passam pelo mesmo grupo — o limite do G6); **sangram** `a_amostragem_do_device_chega_ao_pixel` e `blend_mode_regression` 4/5: são eles que fixam os bytes de hoje |
| M4 | a colagem sem o caminho do vetor | **G7 sangra** (30 B) |
| M5 | o vetor sem descodificar o sRGB | **G7 sangra** (49 B) |
| M6 | `tela_x.y·px.y → ·px.x` | sobreviveu — EQUIVALENTE (a câmera do editor não roda: `tela_x.y = 0`); a pergunta certa, `tela_y.y·px.y → ·px.x`, **sangra** (42 B) |
| M7 | `1/2π → 1/π` | **G7 sangra** (vetor × CPU 64 B) e **a paridade sangra** (1,00) |
| M8 | a colagem sem `tonemap_mapear` | sobreviveu: com `BYPASS_LUT` o tonemap é `clamp` e as cores < 1 não mudam — o G7 só verá o LUT quando ele acender |
| C1–C3 | `cor·E ↔ E·cor` · `1e-6 → 1e-7` · `== 0 → < 1` | sobreviveram (certo) |

## O que o portão de fecho obrigou (1.ª corrida: 1 lint + 6 vermelhos, todos determinísticos)

- `component_registry_for_tests` (`ph2d-app-components`): a W1 registou a luz no boot e não nas
  fixturas — nenhuma onda correra esta crate. Igualado (+ dependência `ph2d-light2d-ecs`).
- o roteador de smoke (acima); o censo da malha posada ganha `ph2d-app-light2d/src/captura.rs`
  («NENHUMA: só pergunta se há malha»); `solver.rs` 863 > 700 LOC ⇒ `solver_gravar.rs` (módulo-filho);
  `fase_vector_bands` 201 > 200 ⇒ a porta `codificar`; a agulha de `present_placa_tests` segue o
  `doc_bands_of(frame_order, placa || luz_acesa)`.
- Os gates `architecture_contract_surface` / `architecture_vector_contract_surface` que o briefing
  nomeia NÃO existem com esse nome na árvore (só `architecture_tool_contract_surface`, verde).

## ⚠️ ABERTO (nomeado; não fechou nesta onda)

1. **Flip não recebe nem bloqueia a luz.** O passe do Flip (`ph2d-app-flip::pass::render`) compõe direto
   no `game_rt` com o shader dele, e compõe TODOS os objetos. Receber = concatenar `receber.wgsl` na
   composição do Flip; bloquear = chamar `pass::render` com a câmera da grade num alvo grade×1 só para os
   objetos com `BloqueiaLuz` (falta um filtro por objeto na crate do Flip). As duas tocam a crate de
   outra família e a chamada vive na shell — W5, com a linha do Flip.
2. **Formas do Motion não bloqueiam** (recebem: `BandSource::Formas` acende). Não são entidades — não há
   onde pôr `BloqueiaLuz`. Pergunta de PRODUTO para a W4: como o artista marca uma forma do Motion?
3. **NaoRecebeLuz não vale para o VETOR** (a colagem acende a faixa inteira) nem para as peças da receita
   aberta no vidro do *Edit Prefab* (instâncias levantadas sem `SimRef`).
4. Fica com o QUAD (dito por `LuzDoQuadro::sprites_por_texel`): sprite com malha/pele, UV com ladrilho, textura
   cozida. A captura amostra bilinear mesmo num sprite com filtro `Nearest` (pixel art).
5. A recusa do solver (`LuzDoQuadro::erro`) e o `RelatorioC0` só chegam ao terminal — **a W4 leva-os à UI**.
6. O K7 em janelas > 1080p e o custo que depende da geometria (S3) — **W5** (governador).

## ⛔ Recusas MEDIDAS (não reconstruir)

| tentativa | número | por quê |
|---|---|---|
| guarda «`Luz` OU emissor» | projetos com `SpriteEmissive` escurecem | a `Luz` é o interruptor |
| degrau 2 px / margem 25 % | 19 GB/s, 11 ms a 1080p | K7 |
| camada do vetor lida BILINEAR | 0,066 (1 px) · 0,039 (2 px) | borra a silhueta; exata: 0,0012 |
| camada do vetor a 4 px por texel | 0,031 e 2,5× o custo | lida exata, 1 px já é a área |
| o tile por objeto do Motion (`ObjectBake`, 256 DPI) para o vetor | — | é de outra família, faz leitura de volta para a CPU, e a camada na grade é exata e mais barata |
| «um buraco no sprite deixa passar a luz» como gate | as 3 cenas davam a mesma fluência | em 2D um buraco fechado é uma cavidade; a pergunta é a fenda |

## Premissas do briefing que a medição derrubou

1. «sem `Luz` **nem emissor**» → a guarda é a `Luz` (acima).
2. «AgX» — o tonemap de hoje é `clamp` (`BYPASS_LUT`); «a MESMA conta» é a função partilhada, com o LUT.
3. «o compositor ganha o caminho do vetor» — o compositor não separa documento de chrome (mesma cena
   Vello); o vetor recebe na COLAGEM das faixas, e a luz acesa força as faixas.
4. «Vetor/Flip/Motion pelo tile de DPI fixo» — o vetor entra por uma camada alinhada à grade (exata, sem
   leitura de volta); Flip e Motion: ABERTO 1–2.
5. «margem: escolha medida» — a oitava de zoom já dá 0–100 % de cobertura extra; o K7 limita a margem a 12,5 %.

## Para a integração (somam entre linhas)

Foundational tocado (ADR-0107): **`ph2d-render`** — `receber.wgsl`, `tonemap_conta.wgsl`, `luz_recebida.rs`
(novos); `@group(3)` no layout de sprite (`pipeline.rs`), `SPRITE_WGSL`/`TONEMAP_WGSL`/`BAND_BLIT_WGSL`
concatenados; `RenderInstance::SEM_LUZ_BIT` = bit 31 (`MESH_MASK` passa a 8–30); `BandSource::Grade`
(índice **3**, apendado); `render_lifted_instances(.., com_luz)` (+1 parâmetro; 2 chamadores na shell);
`clip_pass::encode_*` (+`luz_bg`); `SpriteRenderer::{receber_luz, luz_acesa, vista_da_textura}`;
`Tonemap::lut`. `ph2d-light2d-*`: `CenaDeLuz` + `luzes`, `nao_recebem`, `sprites`, `vetores` (literais
de teste com `..Default::default()`); `Solver::{reposicionar, gravar_com_sprites}`; `Amostragem`,
`CapturaDeSprites`, `SpriteNaC0`, `C0_DONO_WGSL`. **Gerado:** `ph2d-app-registry-init` (`cargo run -p
ph2d-app-sync`, +`app-light2d`) — regenerar, nunca à mão. Shell: `AppGfx::luz2d`, `FrameGfx` (destructure
exaustivo), `init.rs`, `fase_vector_bands.rs`, `present.rs`, `present_bands.rs` (grade → `Grade`),
`present_frost.rs`, `present_fx.rs`, `Cargo.toml` (+`ph2d-app-light2d`). **Delta da shell:** **+31** linhas (185 537 → 185 568; tecto 189 041).
`PROJECT_SCHEMA` intocado (187); `crates/ph2d-vector-doc` intocado; nenhum contrato congelado.

## O que a W4 precisa saber (o artista)

1. As portas da `LuzDoQuadro` já estão na shell; a W4 só acrescenta o que o artista toca (menu Add →
   `Luz`/`Luz ambiente`, Inspector, gizmo, paleta) e as cenas 2–4 (`smoke.rs` já tem o roteador e a
   cena 1 — FOTOGRAFE-a e escreva o roteiro antes de a dar ao dono). ⚠️ `Luz` ainda não tem
   `Default`: a paleta `+` precisa dele.
2. **Intensidade:** `luz_em = I/(2π·r)` — uma `Luz` de `I = 2π` deixa branco um sprite branco a 1 m; a
   Ambiente de `I = 2π` é o forno branco (tudo como hoje). O padrão do Inspector sai daqui.
3. A cena só acende com uma `Luz` (mesmo a zero): o «Add → Luz» é o interruptor da sala escura.
4. `sprites_por_texel()`, `relatorio()` e `erro()` existem para a UI dizer o que ficou com o quad, fora
   da C0 ou recusado (recusa que só o terminal vê é botão mudo).
5. As cenas do smoke usam SPRITES e vetores (o Flip ainda não recebe — ABERTO 1).

## agent-loop-profile

```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                229   alvo: <= 800
  ✗ teste : check                           519 : 262   alvo: <= 1,0  razao 2.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  36%   alvo: >= 80%  (1010 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         412 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
(as 20 sessões mais recentes da máquina, não só esta janela; nesta, as edições foram quase todas por
script com contagem de âncora — o «36 %» é a régua a dizê-lo.)

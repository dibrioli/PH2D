# Handoff — `line/3DModeling`: O SOL E A SOMBRA (2026-10-03)

> **Wave, não integração.** Continua [O_CEU_DE_VERDADE](HANDOFF_line_3DModeling_O_CEU_DE_VERDADE_2026-10-03.md)
> — item 3 da ordem aprovada pelo dono ([O_RENDER_POR_MALHA §8](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md#8--a-próxima-onda-a-ordem-aprovada-pelo-dono-0210--qualidade-unrealfortnite-em-mobile)).
> Base da linha `1ad60a1ce` (main não andou). Commits da wave: `1e47d600c` a lei do sol (`ph2d-sky`) ·
> `d02fd055c` o sol no desenhista + força 0 = estúdio inteiro · `fb4224cf5` a sombra na direcção do
> sol, 3 níveis, Vogel, oráculo · `52f874fdc` faces de trás, raio do pixel, recorte da vista, quadro
> de perto · `df5954c1b` céu nublado sem sol + Key Light apagada · `bf726b1d1` sai o filtro de cone,
> o oráculo ganha o sol de `10°` · `917bf373a` clippy + fmt. **Smoke do dono: OK (03/10).**

## §1 — O que existe

**`ph2d-sky` — `sol.rs` (a lei, CPU)**

- `Panorama::separa_sol()` → `(céu sem sol, Option<Sol>)`. O componente do PICO (8-vizinhança, `x` dá a
  volta) acima de `LIMIAR = 16 ×` a luma média; cada texel dele é PRESO ao limiar com a cor dele e o
  EXCESSO vai para o sol ⇒ `céu sem sol + excesso = panorama`, texel a texel. Abaixo de
  `FRACAO_MIN = 1 %` da energia não há sol (o céu fica intacto: o pátio).
- `Sol { dir, raio, radiancia, energia, fracao }` + a tabela `RUGOSIDADES × ANGULOS` (`49 × 513`, os
  eixos do estúdio) da calote de radiância `1` sob o lóbulo GGX — o MESMO núcleo do atlas
  (`K = (r·l)·D(h)`, normalizado pelo hemisfério, `W(α)` em forma fechada). `raio` = o disco uniforme
  com o mesmo 2.º momento (`θ = √(2⟨ψ²⟩)`, com o `Δ²/12` do próprio texel). Bordo = 2 passos da
  tabela (`BORDO`, `0,45°`). Quadratura HÍBRIDA: amostragem do lóbulo perto do disco com `α` pequeno,
  quadratura sobre a calote no resto (a cauda).
- `irradiance(n)` = a linha `α = 1` (a lei do atlas). `Ceu::com_sol(p)`, `Ceu::sol()`; `Orientado.sol`
  = o peso do sol. WGSL: `sky_sol_tabela(α, cos ψ)`, `sky_sol_cos`; o chamador escreve `sky_sol_ler(i)`.

**`ph2d-mesh-forward`**

- A parte DA caixa sob o céu fotográfico é o SOL (`caixa_rad` / `caixa_irr` no `forward.wgsl`): a
  tabela na ligação `9` (`R32Float`), `Quadro.sol` (direcção no referencial do céu) e `sol_rad`
  (radiância × força × `Foto::caixa`). `Foto::caixa` = o peso do sol (`1` = o panorama como foi
  fotografado). Sem céu ou sem sol: a caixa de quem chama, como antes.
- `gpu_sombra.rs` (novo): `Chave` (estúdio = `+y` com a tangente da caixa; foto = o sol girado com o
  céu, `tan(raio)`) e `enquadra` → DOIS mapas: o de sombra ao longo da chave (objetos e as sombras que
  deitam no chão, fundo com a margem da penumbra) e a cobertura vista de cima (o céu que o chão vê;
  no estúdio também a caixa larga). `vista_da_camera`: o mapa de sombra enquadra só a parte da vista
  onde a cena está (com a cena toda à vista nada muda).
- 3 níveis do mapa de sombra (`2048 / 512 / 128`, o mesmo enquadramento redesenhado; ligações `4`,
  `10`, `11`); o passe de sombra guarda as faces de TRÁS (`cull Back`, sem viés na placa).
- PCSS (`visibilidade_da_caixa`): espiral de Vogel de 16; o raio do PRÓPRIO pixel conta primeiro na
  busca; nível contínuo com mistura (`PCSS_MIN 12 / PCSS_MAX 48` texels do nível); além do fundo do
  mapa, aceso. O chão sob o sol lê o mapa (`Quadro.ceu.w = 0`); no estúdio continua na cobertura.

**`ph2d-app-field3d`**: `ceu_foto::pronto` monta com `Ceu::com_sol`; `normalizacao` = a luz média do
estúdio INTEIRO (a foto substitui as duas partes); `ceu_foto::sem_sol(e)` (lê o cache);
`ceu_painel`: a Key Light apaga-se com `field.inert.sky_has_no_sun`. Tip da Key Light no i18n.

### Ids / variantes / consts novos (colisão na integração!)

| o quê | valor |
|---|---|
| ligações do `g0` do desenhista | `9` (tabela do sol), `10`, `11` (níveis do mapa de sombra) |
| `QUADRO` (floats do uniforme) | `+ 4 + 4 + 16 + 4` (`sol`, `sol_rad`, `ceu_vp`, `ceu`) |
| `ph2d_sky::sol::{LIMIAR, FRACAO_MIN, RUGOSIDADES, ANGULOS, TABELA, PASSO, BORDO}` | `16`, `0,01`, `49`, `513`, `25 137`, `2/512 rad`, `2·PASSO` |
| i18n | `field.inert.sky_has_no_sun` em `crates/ph2d-i18n/src/model3d_sky.rs` |
| fixtura | `crates/ph2d-mesh-forward/fixtures/oraculo_sombra_sol.csv` (`4 224` linhas) |
| instrumento | `docs/3DModeling/ferramentas/oraculo_sombra_sol_blender.py` |
| pipelines | inalterado (`7` / `12`) — só o `cull` e o viés do passe de sombra mudaram |

## §2 — Medições

**Os sóis dos embarcados** (`sol_tests::instrumento_sol`): cidade `47,8°` de altura, `0,40°`, `22 %` ·
nascer `8,1°`, `0,47°`, `56 %` · pôr do sol `3,3°`, `0,38°`, `2,3 %` · floresta `19,5°`, `1,10°`,
`6,2 %` · interior/noite/estúdio = a lâmpada mais forte (`0,4–2,9°`, `13–29 %`) · pátio `0,1 %` ⇒ sem sol.

**Conservação** contra a SOMA EXACTA (`o_ceu_sem_sol_mais_o_sol_e_o_ceu`): céu sem sol + sol erra
`1,5 %` (cidade) / `1,1 %` (nascer) no máximo — MENOS que o atlas do céu inteiro (`4,6 %` / `7,3 %`:
guarda o sol num texel de `3–4°`). Energia da tabela: `≤ 0,13 %` onde o disco ou o lóbulo cobre 8
passos; `≤ 5 %` num sol de `0,3°` quase espelho (o regime do texel de `0,35°` do atlas).

**Paridade placa × CPU** com o sol (pôr do sol e cidade ×1 e ×0,4, 3 materiais): p99 `1 B`, centro `0 B`.

**Oráculo Cycles** (sol de disco, esfera a flutuar + caixa pousada sobre shadow catcher, luz directa),
`|Δ|` médio / máximo do escurecimento do chão:

| caso | Poisson, 1 nível, faces da frente | agora |
|---|---|---|
| cena 40°/1° | `0,0031` / `0,20` | `0,0006` / `0,041` |
| cena 40°/4° | `0,0215` / `0,26` | `0,0095` / `0,083` |
| cena 15°/1° | `0,0065` / `0,24` | `0,0019` / `0,143` |
| cena 40°/10° (penumbra) | — | `0,0154` / `0,098` |
| perto 40°/1° (borda + miolo, `0,26 mm`/px) | `0,022` / `0,44` (miolo a `0,81` onde o Cycles dá `1`) | `0,0005` / `0,061` |

O miolo do sol de `10°` fica a `0,047` da conta do resto que a separação deixa no céu (`16·Ω/4π` =
`12 %` a `10°`; `0,02 %` num sol de `0,4°`): é a lei do limiar, não a sombra.

**Estúdio**: o chão igual; a sombra dos objetos nos objetos muda `0,08–0,15 %` dos pixels (pico
`0,14`, o nó um pouco mais liso). **Quadro** a `1080p`: `1,6–2,1 ms` de mediana a load `~30` (⚠️ não
vale — re-medir com `load < 5`; antes da wave `1,4–1,8 ms` à mesma carga).

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| Um nível de mapa com o tecto de `48` texels (faces da frente) | cortava a penumbra (sol `15°`/`1°` pede `~115`, a caixa do estúdio `~230`): a ponta saía `0` onde o Cycles dá `0,13–0,22` |
| Um nível só (sem níveis nem mistura), com faces de trás | sol de `4°`: máximo `0,083 → 0,278` (mutação vermelha) — os níveis ficam |
| Poisson de 16 pontos | as projecções dele são irregulares: perfis em degraus, `9–51` px com `|Δ| > 0,1` |
| Média de TODOS os bloqueadores do mapa de FRENTE | o topo da caixa (perto no mapa, longe no raio) puxava o raio: o miolo vazava `0,81` |
| O bloqueador mais fundo (máx.) | o pior pixel dobrava (`0,42`/`0,52`) |
| Ponto médio frente/trás | exacto em esferas, mas amolece `~5×` a borda dura do contacto; e pede dois mapas |
| Mapa de faces da FRENTE | a penumbra junto do contacto vinha da distância ao topo (`~1 cm` contra `~1 mm`) |
| Viés por inclinação na placa com faces de trás | empurrava a face rasante para lá do chão: aceso logo atrás da caixa |
| Filtro «só bloqueadores dentro do cone» | com as faces de trás ficou inútil (mutação sobrevivia) e cortava a ponta da penumbra de `10°` (`0` contra `0,40`) |
| Tabela do sol só por Hammersley | a cauda longe do disco com `~2,5` amostras: `+1,2 %` de energia |
| Régua = o atlas do céu inteiro | menos exacta que a separação (`4,6–7,3 %`) |
| Cascatas por fatias de profundidade | +2–4 passes por quadro; no modelador o mapa já enquadra o que se vê (texel `~1 mm` < pixel `~1,6 mm` com a cena toda à vista) — fica para cenas grandes ao ar livre (§5) |

## §4 — Gates

| Crate | Gates |
|---|---|
| `ph2d-sky` | `um_ceu_chapado_nao_tem_sol` · `o_sol_e_achado_e_so_ele` · `a_tabela_conserva_a_energia` · `as_duas_quadraturas_concordam_no_corte` · `o_ceu_sem_sol_mais_o_sol_e_o_ceu` · `o_patio_nublado_nao_tem_sol` · instrumento `instrumento_sol` |
| `ph2d-mesh-forward` | `a_sombra_do_sol_e_a_do_cycles` (GPU; cena ×4 + perto + o miolo de `10°` contra o resto + controlo luz-chave `0`) · `o_ceu_foto_e_a_lei_da_casa` com o sol · `os_ajudantes_sao_ccw_para_fora` |
| `ph2d-app-field3d` | `a_forca_zero_poe_o_ceu_a_luz_do_estudio` (com o sol, `0,00 %`) · `um_ceu_sem_sol_apaga_a_luz_chave` |

**Mutações** VERMELHAS: a chave ignorada (`+y`), sem a margem do fundo (pelo caso de `10°`), sem o raio
do pixel, mapa sem `cull`, sem o recorte da vista, UM nível só, o peso da luz-chave ignorado, a
radiância do sol sem `/Ω`, o sol que não sai do céu, o chão sob o sol pela cobertura. ⚠️ **Não
provadas por este gate**: o nível `2` sozinho e a mistura sozinha — a mistura é o princípio do «sem
degrau» entre pixels vizinhos; o nível `2` serve penumbras `> 192` texels (a caixa do estúdio sobre
objetos longe). ⛔ Lição: a 1.ª «experiência de um nível» forçou só `k0 = 0` e deixou a mistura com o
nível `1` — não era um nível; a conclusão «os níveis não importam» estava errada e foi a régua que a
apanhou.

**Suítes**: `ph2d-sky` 13 · `ph2d-mesh-forward` 15/15 (com a placa) · `ph2d-i18n` · `ph2d-panel-model3d`
66 · `field3d` lib 509 (com a costura do céu) · clippy `-D warnings` limpo nos 5 crates.

## §5 — ABERTO

- **A cauda de uma face de PERFIL para o sol** (o sol paralelo à face): nenhum mapa a guarda; medida e
  com tecto (`0,038` no oráculo de perto). Cura = retroprojeção ou dois mapas (frente + trás).
- **Lâmpadas grandes** (`≥ 4°`) sobre ocluders redondos: as faces de trás estreitam um pouco a penumbra
  interior (o caso de `4°` mede `0,083`); os sóis reais medem `≤ 1,1°`.
- **O resto do disco no céu num sol enorme** (`16·Ω/4π`): um limiar sobre a média SEM o sol curaria —
  não feito: nenhum HDRI real chega lá (o maior é `2,9°`, `1 %`).
- **Cascatas por fatias** para cenas grandes ao ar livre (quando houver): medir texel × pixel então.
- **Gates do Render TRAÇADO `preview::device_tests::w9::ceu_*`**: 3 de 12 reprovam já em `f6df19011`
  (antes desta wave) e o conjunto muda de corrida para corrida a load `30–40` — re-correr calmo.
- As sombras no chão sob um céu fotográfico saem fracas (o chão só escurece a foto atrás dele; na
  cidade o sol leva `28 %` da luz do chão) — é a física do shadow catcher; aprovado no smoke.
- Relógio com `load < 5`. `gpu.rs` a `683/700` linhas (o próximo crescimento move por responsabilidade).
- Herdados: §5 do [O_CEU_DE_VERDADE](HANDOFF_line_3DModeling_O_CEU_DE_VERDADE_2026-10-03.md#5--aberto).

## §6 — Smoke (aprovado pelo dono, 03/10)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=37 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**; secção **Sky** → linha **Sky** → **Sunrise**.
3. Deve acontecer: luz quente e baixa; a caixa verde recorta a sombra na bola amarela; sombras
   compridas e fracas no chão para longe do sol; **Rotation** gira o sol com as sombras; **Key Light**
   2 reforça / 0 apaga; **City** sobe o sol; **Courtyard** apaga a **Key Light** com a frase; **Studio**
   o de sempre.
4. Deu errado se: a sombra não gira com **Rotation**, aponta para o sol, há um bloco de bordas retas
   no chão, a **Key Light** não muda nada num céu com sol, ou engasga.

Nomes da tela = `crates/ph2d-i18n/src/model3d_sky.rs`. Foto do app:
`fotografa_cena.sh "PH2D_FIELD_SMOKE=37 PH2D_FIELD_SHADING=render PH2D_FIELD_SKY=sunrise" 1930 1040 <png> 14`.
Oráculo: `bash scripts/ph2d-run.sh blender -b -X --python docs/3DModeling/ferramentas/oraculo_sombra_sol_blender.py -- crates/ph2d-mesh-forward/fixtures/oraculo_sombra_sol.csv`.
Perfis do gate: `PH2D_SOL_PERFIL=<quadro>/<altura>/<raio>/<corte>` (ex. `cena/40/4/linha`). Sonda:
`PH2D_SONDA_CEU=sunrise` (ver o handoff do céu).

## §7 — Reports do dono

- ✅ Smoke OK (03/10).

## §8 — ⭐ A PRÓXIMA ONDA: item 4 — **Texturas triplanares + mapas de normal**

As malhas do campo não têm UV: projecção TRIPLANAR (as três vistas pesadas pela normal) para cor,
rugosidade e normal, no material da casa (`ph2d_material`) e com paridade placa × CPU como o céu.
Medir o custo por pixel (três leituras por mapa) contra o orçamento do celular. Depois: 5 — contacto
entre peças + chão em paridade com o Render traçado (o §5-d do O_RENDER_POR_MALHA).

## Estado ao fechar a janela

HEAD `917bf373a` (+ o commit deste handoff), árvore limpa, **NÃO integrado, NÃO enviado**,
`incremental/` NÃO reclamado (a linha continua). Binário `smoke` compilado nesta worktree — 2.ª corrida:

```
▸ linha line_3dmodeling · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

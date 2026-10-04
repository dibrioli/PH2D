# Handoff — `line/3DModeling`: A JUNTA DOS REFLEXOS (2026-10-04)

> **Continuação, não integração.** Report 3 do dono (§11 do
> [AS_CAPTURAS_DE_REFLEXO](HANDOFF_line_3DModeling_AS_CAPTURAS_DE_REFLEXO_2026-10-04.md)): *«quando dois reflexos
> estão próximos mesmo quando os objetos estão separados … é como se o reflexo fizesse operação booleana»*.
> Base `1ad60a1ce` (o main não andou). Commits: `5e4e6e1e6` a sonda move peças · `48a600fb1` red-first (oráculo
> `junta`, a régua da faixa, o gémeo da busca) · `0962d8470`/`7af09f5ef` WIP · `8875f7a2c` a cura · `46135d865` a guarda que a
> mutação mostrou redundante · o commit deste handoff (fmt, 2 avisos do clippy da linha, docs). **Smoke do dono: PENDENTE.**

## §1 — O que mudou

- **A captura guarda o 2.º momento da distância.** `fs_sonda` escreve `(d, 1, d², 1)` na textura da distância
  (o canal `b` estava a `0`). Ele atravessa TODA média — o resolve do MSAA das faces, as 4 amostras de `fs_octa`,
  `fs_desce`, o filtro linear da leitura — sem se perder, e normalizado pela cobertura (`b/g`) a borda contra o
  FUNDO não tem variância; só a borda entre DUAS vizinhas a distâncias diferentes a tem (o truque das sombras
  por variância).
- **`sonda_aresta(g)`** (`sonda_le.wgsl`): desvio `> ARESTA · média` (`ARESTA 0,1`, `gpu_sondas.rs`) ⇒ a
  distância lida é FANTASMA. Na busca (`sonda_marcha`), o cruzamento cujo lado de lá é aresta não é aceite nem
  dá franja (o raio continua).
- **`PASSAGEM 0,5`**: a busca passa ao ponto fixo só nos últimos `0,5` níveis abaixo de `LOD_BUSCA`
  (rugosidade `0,1 → 0,2`); abaixo disso (o cromo da cena 42, `0,05`) é só a busca, e o que ela falha é nada.
- **Sonda do app**: `PH2D_SONDA_MOVE=<folha>:x,y,z;…` (folha na ordem do documento) — a pose de um report.
- **Testes**: `tests_reflexo_junta` (gate) e `tests_sonda_junta` (sonda: a busca portada para a CPU sobre a
  captura lida de volta, por leitor e por limiar); `tests_sonda_cpu::texel` (o texel mais próximo);
  `tests_reflexo_perto::{Vista.alvo/meia, JUNTA, desenha_ate, vizinha_refletida}`.
- **Oráculo**: `oraculo_reflexo_perto_blender.py -- … junta` → `fixtures/oraculo_reflexo_junta.csv.gz`
  (`1,2 MB`): a caixa azul LONGE meio atrás da verde PERTO vistas do centro do cromo, com fundo entre elas visto
  de cada ponto do cromo; a vista amplia o par refletido (`meia 0,14`).

## §2 — A causa (medida, não suposta)

| pergunta | medição |
|---|---|
| reproduz pelo produto? | sim — cena 42, `PH2D_SONDA_MOVE='4:0.7,0.15,1.7' PH2D_SONDA_YAW=1.45 PH2D_SONDA_MEIA=0.33 PH2D_SONDA_ALVO=0,0.3,0`: uma ponte azulada translúcida liga o reflexo da verde ao da azul (as fotos do dono: `images/7.png`, `8.png` da sessão `13764661…`) |
| a régua vê? | a FAIXA entre os dois reflexos (px cujo raio, pela geometria, não acerta nada, a > 2 px de quem acerta, com uma vizinha DIFERENTE de cada lado na linha): `1 533 px`, `|Δ|` médio **`0,597`**, **`1 268`** grosseiros (nítido); `0,550 / 1 265` a `0,05` |
| é a interpolação da distância? (gémeo da CPU, a busca do produto sobre a captura lida) | com a azul, a busca ACEITA `1 233` px da faixa (bilinear); com o TEXEL mais próximo ainda `330` — a distância já vem misturada nos DADOS (MSAA + amostras do octaedro), não só na leitura; sem a azul, `0` |
| e o 0,05? | só a busca (temporário `busca = 1`): `0,0185 / 28`. O PONTO FIXO pesava `~37 %` no quase nítido (`(1 − busca) + busca · lod0` a `lod0 0,25`) e cai, pela imprecisão própria (níveis largos), em cima das vizinhas — não numa aresta (marcar o ponto fixo pela aresta mexeu `0,172 → 0,170`: recusado) |

## §3 — Números (A/B na mesma máquina, antes → depois)

| régua | antes | depois |
|---|---|---|
| junta nítido (faixa: `|Δ|` / grosseiros) | `0,597 / 1 268` | **`0,0142 / 29`** |
| junta `0,05` | `0,550 / 1 265` | **`0,0177 / 26`** |
| perto nítido (miolo / grosseiros) | `0,0126 / 919` | `0,0126 / 925` |
| perto `0,05` (miolo / contorno / grosseiros) | `0,0226 / 0,0938 / 820` | `0,0223 / 0,0982 / 858` — o contorno a `0,05` ficou tão nítido como o espelho (o ponto fixo borrava-o) |
| par nítido e `0,3` | `0,0069 · 0,0417` | iguais |
| gémeo: dos `42 711` px que DEVEM acertar, aceites | `41 875` | `41 155` (`−1,7 %`: 1–2 px da borda de uma vizinha junto da OUTRA) |
| **custo**, 1080p, release, A/B alternado `antes·depois·antes·depois`, load `4,1–4,8` | girar 16: `+0,46 / +0,41 ms` | `+0,47 / +0,40 ms` — nada além do ruído; arrastar 16: `+9,71 / +10,04` × `+10,03 / +9,77 ms` |

Suíte com placa da crate: **42/42** (`tests_reflexo`, `tests_reflexo_perto`, `tests_reflexo_junta`,
`tests_sonda_cpu`, `tests_sondas` — o cache ao byte e girar perto —, `tests_chao_tapa`, `tests_contacto`, …).
clippy `--all-targets` das duas crates limpo; fmt limpo.

## §4 — ⛔ Recusas MEDIDAS (não reconstrua)

| recusado | por quê |
|---|---|
| ler o texel MAIS PRÓXIMO na busca (sem filtro) | ainda `330` de `1 533` aceites: a mistura está nos dados |
| `ARESTA 0,2` / `0,05` | os três dão `0` aceites na faixa; `0,2` não marca pares de profundidade próxima (`d₂/d₁ < ~1,5`: desvio máximo `< 0,2`), `0,05` perde mais acertos legítimos (`41 069`); `0,1` marca `d₂/d₁ ≥ ~1,2` |
| na aresta, ler a distância «pura» uns texels adiante (`3` passos) | recupera `+278` acertos (`0,65 %`), a foto não muda e nenhum gate o veria — retirado |
| marcar o PONTO FIXO pela aresta | `0,172 → 0,170`: a ponte a `0,05` não vinha dali |
| o controlo «sem a azul» com a barra da média | lê `0,09`: é a FRANJA (abaixo), não ponte; o controlo «sem capturas» lê `0,107` (a máscara da zona da lei antiga escurece a faixa) ⇒ o controlo afirma só os GROSSEIROS (`35 ≤ 50`; a ponte dá `1 268`) |

## §5 — Gate e prova de mutação

`tests_reflexo_junta::entre_dois_reflexos_o_cromo_mostra_o_fundo` (nítido e `0,05`; barras `|Δ| < 0,03`,
grosseiros `≤ 50`; CONTROLO sem capturas: grosseiros `≤ 50`; foto `nossa | Cycles | faixa` com
`PH2D_REFLEXO_FOTOS`). **Mutações (agente `mutacao`, controlo verde `0,0142·29 / 0,0177·26`): 5 de 6
VERMELHAS** — sem o 2.º momento, `sonda_aresta → false`, `ARESTA 10`, `if (true)` na busca (todas `0,597·1 268 /
0,564·1 265`); `PASSAGEM 1` (só o `0,05`: `0,133·482`). **Sobreviveu** o `g.y > 0.5 &&` dentro de `sonda_aresta`:
o lado de lá de um cruzamento é coberto por construção ⇒ a guarda saiu (`46135d865`).

## §6 — ABERTO

- **A FRANJA alarga a borda das vizinhas refletidas** (pré-existente): sem a azul, a faixa lê `0,09` / `253`
  grosseiros — o raio que passa por trás da verde (vista do centro) até `ESPESSURA 5 %` conta como acerto e até
  `FRANJA 10 %` dá peso. Ampliado, a borda é MOLE e mais larga que a do Cycles (`~10 px` com o cromo a
  `~1 100 px`; `~3 px` na escala do dono). Uma franja do tamanho do PIXEL (a abertura do raio refletido) é o
  candidato; não medido. Onde há aresta atrás (a azul), a borda fica dura — o contraste entre os dois troços.
- **O lado escondido da vizinha** (a face que só o ponto do cromo vê) continua — o limite de uma captura.
- ⛔ **Arrastar 16 peças de METAL**, re-medido CALMO (load `< 5`): `+9,7–10,0 ms`, quadro `12,7 ms` a 1080p (o
  handoff anterior dizia `~+8 ms` com load `~15`). Alavancas medidas lá: amostras `64 → 16`, capturas só das
  peças brilhantes, LOD.
- Herdados: §5 e §9–§11 do [AS_CAPTURAS_DE_REFLEXO](HANDOFF_line_3DModeling_AS_CAPTURAS_DE_REFLEXO_2026-10-04.md).

## §7 — Para o integrador

`ph2d-mesh-forward`: consts novas `ARESTA`, `PASSAGEM` (WGSL `SONDA_ARESTA`, `SONDA_PASSAGEM`); a textura da
distância das capturas ganha o canal `b = d²·a` (mesmo formato `Rgba16Float`, nenhuma ligação nova). Fixtura
nova `~1,2 MB`. `ph2d-app-field3d`: só a sonda `#[ignore]` (`PH2D_SONDA_MOVE`). `PROJECT_SCHEMA` intacto; shell
`0` linhas; contratos congelados: nenhum; cenas: nenhuma nova (a próxima continua a 43).

## §8 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=42 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**. Arraste a caixa AZUL para perto da câmara, como nas suas fotos.
3. Deve acontecer: na bola de cromo, o reflexo da caixa verde e o da azul ficam SEPARADOS, com o fundo entre
   eles — nenhuma ponte nem cunha a colá-los, a girar a câmara e a arrastar a azul.
4. Deu errado se: aparecer uma ponte (mesmo translúcida) entre dois reflexos, ou um reflexo perder um pedaço.

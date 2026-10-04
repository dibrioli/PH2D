# Handoff — `line/3DModeling`: A BASE POUSADA (2026-10-04)

> **Continuação, não integração.** Report 5 do dono (§11 do
> [A_JUNTA_DOS_REFLEXOS](HANDOFF_line_3DModeling_A_JUNTA_DOS_REFLEXOS_2026-10-04.md)): *«no reflexo aparece a
> verde, um VAZIO, e só depois a azul»* — fotos da sessão `0bbd8ca2…`, `images/2.png`, `3.png`. A ordem era o
> reflexo pela tela (SSR); **a medição derrubou a premissa** e o dono redirigiu (§2). Base `1ad60a1ce` (o main
> não andou). Commits: `dcbaa2ca1` a sonda do app muda a rugosidade · `e143632b3` o oráculo em perspectiva e as
> poses do dono · `949f2428f` WIP · `0acd3bfc1` a cura · `6ffce09e3` a contagem de passos que a mutação mostrou redundante · o commit
> deste handoff. **Smoke do dono: PENDENTE.**

## §1 — O que mudou

- **`sonda_marcha` (`sonda_le.wgsl`): o CHÃO acaba o arco da busca e a última amostra é NELE.** Com chão e o
  raio a descer, o arco vai de `0` à direcção (vista do centro da captura) do ponto onde o raio toca o chão,
  `a = fim·k/n` (sem chão, como antes: `th·k/(n+1)` — bit a bit); o NÚMERO de passos é o de antes (do arco inteiro). Nada se acerta abaixo do chão, e a base fina
  de uma vizinha POUSADA (vista de raspão do centro, ~2 texels) já não cabe entre dois passos: o ponto do chão
  sob o raio fica atrás da face, visto do centro, e o cruzamento é apanhado.
- **Gémeo da CPU** (`tests_sonda_junta::marcha`) com o mesmo fim de arco (parâmetro `altura` sobre o chão).
- **Sonda do app**: `PH2D_SONDA_RUG=<folha>:<rugosidade>;…`, `PH2D_SONDA_NOME=<sufixo>`, e imprime a câmara
  (`alvo · meia · olho · base`) — para levar a pose de uma foto do dono ao oráculo.
- **Oráculo** `oraculo_reflexo_perto_blender.py`: `DIST` (câmara em PERSPECTIVA, a lente de fábrica do app);
  modos `vazio` (a pose do dono, o olho do app de perto) e `foto2` (a pose da 2.ª foto, olho de fábrica) →
  `fixtures/oraculo_reflexo_vazio.csv.gz` (`~0,96 MB`), `oraculo_reflexo_foto2.csv.gz`. Corre em `~35 s`.
- **Testes**: `Vista.dist` (perspectiva: `raio`, `vista_em`, `camera`); `VAZIO`, `FOTO2`;
  `tests_reflexo_tela.rs` (novo): gate `a_base_de_uma_vizinha_pousada_nao_se_salta`, sondas `sonda_do_vazio`,
  `sonda_do_vazio_a_vista` (CPU: o que o olho vê do vazio), `sonda_da_foto_do_dono`; `tests_sonda_junta`:
  `sonda_das_bordas_do_dono`, `sonda_da_escada`.

## §2 — ⛔ A premissa que a medição derrubou (o reflexo pela tela)

| pergunta | medição |
|---|---|
| o app em perspectiva? | **sim** — a lente de fábrica do `Orbit` (`atan(18/50)`); o oráculo e as réguas eram ORTOGRÁFICOS. Ortográfica, a pose do dono nem tinha vão |
| o vão da foto do dono é defeito? | **não**: na pose da 2.ª foto (`PH2D_SONDA_YAW=4.6`, azul em `−0,55; 0,15; 0,75`) o **Cycles mostra o MESMO vão** entre a verde e a azul (`|Δ|` `0,0073` nos px das vizinhas, `0,0047` no resto; foto [img/report5_o_vao_nosso_x_blender.png](img/report5_o_vao_nosso_x_blender.png)) |
| quanto do reflexo é «escondido do centro»? (`sonda_do_vazio_a_vista`, geometria, 21 poses) | `0–5 %` dos px que acertam vizinhas, quase sempre `< 1 %`; na pose da foto 2, **0** |
| e o reflexo pela tela consertaria? | só a parte que o OLHO vê e está na tela: na pose ampliada do dono, o crescente do cromo escondido pela verde (`362 px`, `|Δ|` `0,218`, a única zona vermelha) está FORA da tela |

⇒ **Decisão do dono (04/10)**: «limpar as bordas» em vez do reflexo pela tela. O SSR **não foi construído**;
o desenho dele (pré-passe no mesmo quadro, reaproveitando o pipeline das faces; uma textura de 2 camadas —
o grupo `0` já usa `15` das `16` texturas por estágio do WebGL2; confiança: falha CERTA quando o raio sai da
caixa da cena sem passar por trás de nada) fica aqui como ponto de partida, se um dia for pedido. O custo
estimado (uma passada de faces a mais, `~1/3` do quadro de 16 peças) não se mediu.

## §3 — A causa das bordas (medida)

| pergunta | medição |
|---|---|
| de onde vem a ESCADA e os DENTES da borda de baixo da azul (vista do dono, ampliada)? | `sonda_da_escada`: a geometria põe a borda numa diagonal regular (+2 px por 4 filas); a busca só ACEITAVA a azul 1–4 px para dentro, aos saltos; na faixa por aceitar aparecia a sombra escura do chão junto da caixa |
| porquê? | o ponto acertado nessa faixa está a `y = 0,000–0,011` (a BASE da caixa); o único cruzamento que a busca via era o contorno da verde, a `λ 1,5–7,7` — a base fina, de raspão do centro, cabia entre dois passos |
| a geometria dos outros acertos? | certa: peso `1`, `fora/λ ~0,001` (`sonda_das_bordas_do_dono`) |

## §4 — Números (antes → depois, mesma máquina)

| régua | antes | depois |
|---|---|---|
| **base** vista do dono, nítido · `0,05` (`|Δ|` / `> 0,1`) | `0,0266 / 363` · `0,0244 / 290` | **`0,0158 / 110` · `0,0191 / 125`** |
| base, perto da cena 42 | `0,0279 / 38` · `0,0227 / 31` | `0,0186 / 18` · `0,0157 / 11` |
| orla de DENTRO sobreposta, nítido · `0,05` | `0,0234 / 396` · `0,0265 / 394` | **`0,0161 / 249` · `0,0196 / 247`** |
| orla de dentro, junta | `0,0222 / 345` | `0,0140 / 203` |
| orla de fora sobreposta · junta | `0,0169 / 177` · `8` | `0,0170 / 179` · `9` |
| junta (faixa), perto (miolo), par | `0,0076 / 2` · `0,0113` · `0,0069` | iguais |

O resto da base (`110`) é a LUZ da face da azul virada para a verde (o Cycles escurece-a mais — a oclusão
entre peças), não salto: o erro é baixo e uniforme ao longo da faixa. Suíte com placa da crate (sem sondas nem
instrumentos): **28/28**; sem placa `6/6`; clippy `--all-targets` das duas crates e fmt limpos.

**Custo:** NÃO medido no relógio — a máquina esteve a load `28–40` (outra linha) o resto da sessão. Pela
construção: nenhuma leitura de textura a mais (o mesmo número de passos; só o fim do arco muda), um ramo e um
`atan2` por pixel da busca; nenhuma ligação, pipeline ou formato novo ⇒ o caminho GLES não muda (esta máquina
não tem GL: `as_capturas_cabem_no_gles` não corre aqui). A medir com load `< 5`: `instrumento_custo_sondas`.

## §5 — ⛔ Recusas MEDIDAS (não reconstrua)

| recusado | por quê |
|---|---|
| o reflexo pela tela para o report 5 | o vão é geometria (o Cycles mostra-o); a parte escondida do centro é `< 1 %` e, na vista do dono, fora da tela (§2) |
| o oráculo ortográfico para poses do app | o app é perspectiva; ortográfica a pose do dono não reproduzia nada |
| contar os passos pelo arco até ao chão | sobreviveu à mutação bit a bit (satura em `MARCHA_MAX`): saiu |
| faces da captura a `512` (contra a ondulação fina que sobra na ampliação extrema do dono) | o friso do topo da azul afina mas continua; a borda esquerda da verde não muda (não é das faces: é o octaedro / o limbo) — ganho parcial por `4×` o passe das faces |

## §6 — Gate e prova de mutação

`tests_reflexo_tela::a_base_de_uma_vizinha_pousada_nao_se_salta` — barras pela medição: vista do dono
`(0,022, 160)`, perto `(0,022, 25)`. **Mutações (agente `mutacao`, controlo verde `0,0158·110 / 0,0191·125 /
0,0186·18 / 0,0157·11` antes e depois): 3 de 4 VERMELHAS** — a cura desligada (`0,0266·363`, a de antes), a
última amostra fora do chão `a = fim·k/(n+1)` (`0,0787·1 412`: pior que sem cura — ela tem de estar NO chão), o
arco sem fim `fim = th` (`0,0234·316`); **sobreviveu** a contagem de passos pelo arco até ao chão ⇒ saiu (`6ffce09e3`).

## §7 — ABERTO

- **A luz da face entre peças**: a face da azul virada para a verde sai mais clara que no Cycles (a oclusão que
  uma vizinha faz a outra — o contacto); medido como o resto da régua da base.
- **Os dentes da foto 3 do dono** (triângulos na vinco onde a caixa azul ENTRA no cromo grande): é a extração da
  malha numa quina côncava da união (§5-b do O_RENDER_POR_MALHA), não o reflexo — por medir.
- **A ondulação fina na ampliação extrema** (o olho a `0,24` da superfície): o friso claro do topo da azul e a
  borda esquerda da verde ondulam com o passo de um texel do octaedro (`LADO 512`). Faces `512` não a curam (§5).
- **O limbo**: a borda esquerda da verde junto ao limbo do alumínio (`sonda_das_bordas_do_dono`, `+0,65–0,88`,
  ~40 px) — o raio de raspão extremo (já no §6 do A_JUNTA).
- **O escondido do centro** (`sonda_do_vazio`, `362 px`): cura candidata a 2.ª camada da captura (§11 do
  A_JUNTA), que também cobre o fora da tela.
- Herdados: §6 do [A_JUNTA](HANDOFF_line_3DModeling_A_JUNTA_DOS_REFLEXOS_2026-10-04.md).

## §8 — Para o integrador

`ph2d-mesh-forward`: só o WGSL da busca (nenhuma const, ligação, pipeline ou formato novo); fixturas novas
`~1,9 MB` (`oraculo_reflexo_vazio`, `oraculo_reflexo_foto2`); imagem `docs/3DModeling/handoffs/img/`.
`ph2d-app-field3d`: só a sonda `#[ignore]`. `PROJECT_SCHEMA` intacto; shell `0` linhas; contratos congelados:
nenhum; cenas: nenhuma nova (a próxima continua a 43).

## §9 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=42 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**. Ponha a bola de ALUMÍNIO (a da direita) com a rugosidade a `0`, como
   fez, arraste a caixa AZUL para a frente da verde, e aproxime a câmara do alumínio.
3. Deve acontecer: no reflexo, a borda de BAIXO das caixas é uma linha limpa — sem escada e sem dentes escuros
   por baixo. O vão entre o reflexo da verde e o da azul pode ficar: é verdadeiro (o Blender mostra o mesmo,
   [img/report5_o_vao_nosso_x_blender.png](img/report5_o_vao_nosso_x_blender.png)).
4. Deu errado se: a borda de baixo de uma caixa refletida voltar a ter degraus ou dentes escuros. (De MUITO perto
   ainda há uma ondulação fina no friso de cima da caixa e na borda da verde junto à beira da bola — aberta, §7.)

Build do smoke (2.ª corrida, a prova): `Finished smoke profile [optimized] target(s) in 0.31s`, zero `Compiling`.

## §10 — Perfil do loop do agente (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                211   alvo: <= 800
  ✗ cargo test : cargo check                636 : 229   alvo: <= 1,0  razao 2.8x
  ✗ edicoes pela ferramenta Edit                  34%   alvo: >= 80%
  ✗ contexto relido por passo (media)         491 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

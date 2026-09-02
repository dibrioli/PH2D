# HANDOFF DE TROCA — `line/quadextract` · **A AMPUTAÇÃO E O EMBOTAMENTO DAS PONTAS**

> **Data:** 2026-09-01 · **Branch:** `line/quadextract` · **HEAD:** `e333b01be` · **Base:** `066b4f92e`
> **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract`
> **Para:** o agente que assume esta linha numa janela nova. **Leia este ficheiro INTEIRO antes
> de tocar em código** — ele existe porque uma jornada inteira falhou, e o motivo da falha é a
> primeira secção.

---

## §0 — ⛔⛔⛔ A ÚNICA COISA QUE VOCÊ TEM DE ACREDITAR ANTES DE COMEÇAR

**As réguas desta linha dizem que o produto melhorou muito. O dono olha e diz
*«absolutamente nenhuma melhoria»*.**

Isto não é uma discordância de grau. É uma discordância de **sinal**, e ela repetiu-se
**quatro vezes num dia**:

| o que eu entreguei | o que a régua disse | o que o dono disse |
|---|---|---|
| a cerca de viagem do acabamento | ponta `2,39 → 0,67` quads | *«não é bom»* (foto) |
| a régua por-ponta + a chave no selector | `pior 4,14` deixa de poder vencer | *«veja. não é bom»* (foto) |
| a densidade no campo (`scale_by_density`) | desvio `0,47 → 0,22`, dobras `17 → 6` | **«absolutamente nenhuma melhoria»** |

⇒ ⭐⭐⭐ **A PRIMEIRA OBRA DESTA LINHA NÃO É ALGORITMO. É UMA RÉGUA QUE CONCORDE COM A FOTO.**
Enquanto não existir um número que fique **vermelho na foto que ele reprovou e verde numa
retopologia que ele aprovou**, toda medição desta linha é ruído com aparência de progresso — e
eu gastei um dia a provar isso.

⚠️ **Há material para a construir, e é o activo mais valioso aqui:** a pasta `~/Downloads` do
dono tem **as fotos e os ficheiros dos dois lados**:

- `Sculpt_Blender.obj` — uma retopologia que ele **APROVOU** (*«preserva as pontas»*)
- `_base_sculpt.obj` — a escultura de entrada
- `_remesh_sculpt.obj`, `sculpt_Depois.obj` — saídas nossas que ele **REPROVOU**

⇒ **Uma régua candidata tem de separar `Sculpt_Blender.obj` das nossas duas.** Se não separa,
não é a régua. ⛔ *Não avance para código de algoritmo sem esse discriminador.* Todas as réguas
que esta linha construiu falham este teste — nenhuma delas foi alguma vez corrida contra a
malha que ele aprovou.

---

## §1 — Como reproduzir, exactamente

### §1.1 ⛔ A FIXTURA — o botão vê a peça **RECENTRADA**

O importador (`sculpt3d_import::place`) faz `Mesh::recenter()` (subtrai o **centro da caixa**) e
mete escala/posição numa `Pose` que **só desenha e exporta**. ⇒ *um `.obj` exportado traz a pose
assada e NÃO é o que o botão vê.* Isto mordeu **quatro vezes em dois dias** e invalidou oito
células de uma varredura.

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract
python3 - <<'PY'
src="/home/enio/Downloads/_base_sculpt.obj"; dst="/tmp/base_recentrada.obj"
L=open(src).read().splitlines()
V=[tuple(map(float,l.split()[1:4])) for l in L if l.startswith("v ")]
c=[(min(p[k] for p in V)+max(p[k] for p in V))/2 for k in range(3)]
it=iter(V); out=[]
for l in L:
    if l.startswith("v "):
        p=next(it); out.append(f"v {p[0]-c[0]:.6f} {p[1]-c[1]:.6f} {p[2]-c[2]:.6f}")
    else: out.append(l)
open(dst,"w").write("\n".join(out)+"\n"); print("escrito", dst, len(V), "vertices")
PY
```

### §1.2 A SONDA — o botão inteiro, sem GPU

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract
cargo test -p ph2d-host-desktop --release --no-run          # 1×, e NÃO reconstrua durante uma medição
env PH2D_PIECE=/tmp/base_recentrada.obj PH2D_DETAIL=1.0 PH2D_ADAPT=1.0 \
    ./target/release/deps/ph2d_host_desktop-<hash> \
    the_artists_piece_through_the_button --ignored --nocapture --test-threads=1
```

Envs da sonda: `PH2D_PIECE` · `PH2D_DETAIL` · `PH2D_ADAPT` (o *Follow Curvature*, e ⚠️ **o dono
usa-o sempre no MÁXIMO**) · `PH2D_DUMP=<obj>` (a saída) · `PH2D_DUMP_F1=<obj>` (a fase zero) ·
`PH2D_PRESSES=<n>` · `PH2D_PROBE_LOCAL=1`.

Bissecção do produto: `PH2D_EXTRACT_FINISH=0` · `PH2D_EXTRACT_TRAVEL=<k>` ·
`PH2D_FIELD_DENSITY=<k>` · `PH2D_SIZING_RATIO=<n>` · `PH2D_SIZING_SMOOTH=<n>` ·
`PH2D_ISO_ADAPT=0` · `PH2D_ISO_LOG=1` · `PH2D_RETOPO_SERIAL=1` · `PH2D_EXTRACT_MIRROR=0` ·
`PH2D_RETOPO_EXTRACT=0`.

### §1.3 ⛔ Duas armadilhas de método que me custaram horas

1. **NUNCA reconstrua enquanto uma medição corre.** Apontei o `CARGO_TARGET_DIR` de uma
   bissecção ao `target/` da worktree principal: as reconstruções de commits antigos passaram a
   substituir o binário que as corridas «de agora» executavam. Concluí que o botão dependia da
   carga da máquina — **é falso**, ele é perfeitamente determinista.
2. **A cwd do Bash volta à árvore primária.** Um `python3` com caminho relativo correu no
   primário. Use sempre `cd /…/Worktrees/line-quadextract && …` na MESMA chamada.

---

## §2 — O que está MEDIDO e é sólido (não re-meça)

### §2.1 A localização: o defeito nasce **a jusante da fase zero**

Perfil de densidade por ponta (aresta média em anéis de **caminho sobre a superfície** a partir
de cada bico, `× a mediana da malha`):

| | pior bico |
|---|---|
| a escultura do dono | `2,40` (ela própria é grossa nos bicos) |
| ⭐ **a fase zero (F1)** | **`0,82`** — as seis pontas ficam finas |
| ⛔ a saída do botão | `1,55` |

⇒ **`100 %` da grossura da ponta nasce depois do F1.** O `ph2d-remesh-iso` está ilibado por
resultado.

### §2.2 O mecanismo: a grade **termina**, não converge

| ponta | quad no bico | vértices a `≤6q` | irregulares aí | valência do ápice |
|---|---|---|---|---|
| ⛔ má | `3,85×` | **`8`** | **`37,5 %`** | `3` |
| ⛔ má | `1,43×` | `26` | `11,5 %` | `3` |
| ✅ boa | `0,41×` | `246` | `1,2 %` | `4` |
| ✅ boa | `0,50×` | `183` | `1,6 %` | `4` |

As pontas boas têm **~30× mais vértices** na mesma vizinhança física. Nas más as linhas de grade
**acabam todas de uma vez, a meio do espinho**. ⚠️ A peça tem `0,23 %` de irregulares no total —
**classe do oráculo**: o defeito não é *quantos*, é *onde*.

### §2.3 O teorema que explica por que três curas falharam

Num mapa de grade inteira o factor de escala e o ângulo do campo são **conjugados**
(`∇σ = J∇θ`). ⇒ **a densidade realizável é ditada pelo CAMPO**, e o mínimo quadrado do G3
projecta fora o resto. *Pedir um passo mais fino AO MAPA é pedir algo que ele é obrigado a
recusar.*

---

## §3 — ⛔⛔ RECUSAS MEDIDAS — não as reconstrua

| o que foi tentado | o que a medição deu |
|---|---|
| **alisar mais/menos o pedido** (`SIZING_SMOOTH` 0·2·4·8·16) | `8` já é o óptimo; a faixa inteira é `1,28`–`1,46` |
| **subir o tecto da faixa** (`MAX_ADAPTIVE_RATIO` 4→8→16→32) | pedido `14 %` mais fino ⇒ saída move `3 %`; `>60` vai de `0` a `7`. ⭐ E o «rasga» que o doc do tecto declara **não foi reproduzido** |
| **`ADAPT_RATIO` da fase zero 16→64** | revertido por veredito do dono (*«piorou, amputou 2»*) |
| **canonicalizar a pose** | destrói (`−77 %`, `−105 %`) |
| **campo contínuo / plateau** | ponta `0/4 → 1/4`, uma célula a `−40,8 %` |
| **construir o campo uma vez da referência** | produto piora para `−48,6 %` |
| **puxar o vértice mais avançado** | aspecto `12,11`, enviesamento `85°` |
| **shrinkwrap da região da ponta** | malha destruída (aspecto `1,9·10⁸`) e a régua mal se move |
| **carregar no botão repetidamente** | **não** piora (1 clique `1,15×`, 2 cliques `0,89×`) |
| **`PH2D_F1_TARGET=1`** | `χ = 1`, `4` bordo, `123` dobras |
| **força da correcção conforme ≠ 1** | `1,5` leva o mapa a `105` dobras; `2` leva o enviesamento a `7,7°`; **negativa rasga** (`100` bordo) |

O mecanismo de cada uma está em `docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md`, Partes
IV–XI e §§82–90.

---

## §4 — O que esta jornada CONSTRUIU e deixou no `main` da linha

Quatro commits, todos com gate: `9a423062d` · `84492aac4` · `82822f14c` · `e333b01be`.

1. **`ph2d_quadfill::tip_deviation`** — distância da escultura à saída junto de cada ápice, em
   unidades do quad pedido. Barra `TIP_DEVIATION_MAX = 1`.
2. **`ph2d_quadfill::tip_density`** — o quad junto do bico, em unidades do quad pedido, por
   distância de **caminho**. Barra `TIP_DENSITY_MAX = 1,5`.
3. **`ph2d_quadfill::reach`** — centroide de **ÁREA** (o antigo era média de vértices e media a
   *amostragem*: lia `−6,5 %` onde a verdade era `−0,1 %`, **no caminho do produto**).
4. **A cerca de viagem do acabamento** (`EXTRACT_TRAVEL_RESCUE = 0,5`) — ela existia, o doc dela
   chamava-se *«a porta do produto»*, e **não tinha um chamador**: o produto passava
   `f32::INFINITY` com o laço a correr até `1 200` rondas.
5. **`ph2d_crossfield::Dual::scale_by_density`** — a correcção conforme `α = −∗ds` no transporte.
   Força `1` (a da teoria, sem curso). Entra como **candidata**, com uma guarda: *uma candidata
   com densidade que ampute MAIS pontas que a melhor sem correcção não é oferecida.*
6. **O selector** (`sculpt3d_retopo_decide.rs`) ganhou duas chaves — amputação por contagem+`p90`
   e densidade da ponta — e `still_broken` passou a armar socorro por elas.
7. **Três cortes de LOC por responsabilidade**: `sculpt3d_retopo_one.rs` (correr UMA candidata),
   `_decide.rs` (escolher entre duas), `_target_tests.rs`. E duas portas que apagaram duplicação
   real: `one::par` e `corrida`.

**Estado dos portões:** `cargo test -p ph2d-crossfield -p ph2d-quadflow -p ph2d-quadfill
-p ph2d-host-desktop --release` ⇒ **211 alvos verdes, 0 falhas** (as flakes de carga conhecidas
do §5 do `CLAUDE.md` — áudio e `flip_smooth::…::orcamento` — passam sozinhas).

⚠️ **Nada disto foi integrado nem pushado.** A linha fecha e espera ordem do Enio (§0.7).

---

## §5 — ⭐ POR ONDE EU COMEÇARIA, na sua posição

1. **A régua que concorda com a foto** (§0). Corra as réguas existentes sobre
   `Sculpt_Blender.obj` (aprovada) e sobre `sculpt_Depois.obj` (reprovada) e veja **qual, se
   alguma, as separa**. ⛔ Se nenhuma separa — que é a minha aposta —, a obra é inventar a
   grandeza que separa, e ela provavelmente **não é uma mediana nem um extremo**: as duas já
   falharam aqui, e o §84 mostra que o defeito é *onde* os irregulares estão, não quantos.
2. Só então: a **decomposição por fases** do §2.1 diz que o alvo é F2→F3→G3→extracção. A
   suspeita com mais suporte é a **distribuição de singularidades ao longo do cone** — a grade
   tem de perder linhas **escalonadamente** e perde-as todas de uma vez.
3. ⚠️ **Peça-lhe uma foto de perto de uma ponta BOA** (das que a régua diz `0,41×`). Eu nunca
   soube se o que ele chama de bom coincide com o que a régua chama de bom, e isso é uma
   pergunta de uma mensagem.

---

## §6 — Higiene desta linha

- ⛔ **Não integre, não pushe** (`CLAUDE.md` §0.7). 4 commits locais sobre `066b4f92e`.
- ⛔ **Clean-room activo** (ADR-0162/0167): nunca abra `ph2d-quadbench/oracle/`, `~/Referencias/`,
  nem fonte do alvo. Dentro de `docs/3D/cleanroom/` só os `SPEC_*`. Todo artefacto que cruza a
  parede passa `bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_quadwild.txt <paths>`.
- **Contratos congelados encostados:** nenhum.
- **Ids/consts novos** (colisão na integração): `TIP_DENSITY_MAX` · `EXTRACT_TRAVEL` ·
  `EXTRACT_TRAVEL_RESCUE` · `FIELD_DENSITY` · `TipDensity` · `Candidata` · `one::par` ·
  `decide::melhor` · `Dual::scale_by_density` · `ScaleField::adaptive_ranged`.
- **Foundational tocado:** `ph2d-crossfield` (método novo, aditivo) e `ph2d-quadflow`
  (`adaptive_ranged` novo; `adaptive_between` ganhou um parâmetro, privado).
- **Ficheiros no tecto de LOC** (`598`, `600` é o cap): `sculpt3d_history_retopo_extract.rs`.
  *A próxima linha que lhe acrescentar código corta primeiro.*

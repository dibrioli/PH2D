# ADR-0170 — O corte é uma **booleana de malha**, e desta vez a biblioteca **ENTRA** em vez de ficar oráculo

- **Status:** Aceito (2026-09-15)
- **Contexto:** `line/sculpt3d`. Ordem do dono em 2026-09-15: *«Creio que ainda não temos vários
  pincéis do blender: Vamos começar por TRIM. Vá estudar o blender para implementar aqui.»*
- **Sob:** [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) (drop-crate)
  · [ADR-0150](0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md) (o módulo de
  escultura) · a lei do gesto vive na
  [`SPEC_trim_gesture.md`](../../3D/cleanroom/SPEC_trim_gesture.md), atestada pelo R-pré.
- ⚠️ **Contraste deliberado com [ADR-0162](0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md)
  e [ADR-0167](0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)** —
  a *mesma* pergunta, e a resposta é **oposta**, porque a **licença** é outra.

---

## 1 — O problema

Cortar a escultura com uma forma desenhada no ecrã precisa de tirar volume de uma malha. Esta casa
tem **duas** rotas possíveis, e elas **não** levam ao mesmo sítio:

- a **volta por campo** (`ph2d_sdf::voxelize → flood_fill → surface nets`), que já ship no botão de
  remesh;
- uma **booleana de malha**, que **não existe aqui** (a `ph2d-vec-boolean` é 2D).

## 2 — A medição que decide (e ela foi feita dos DOIS lados, independentemente)

A sonda `ph2d_sdf::remesh::tests::diag_o_preco_da_volta_por_campo` mediu a nossa rota; a
[`SPEC_trim_gesture.md`](../../3D/cleanroom/SPEC_trim_gesture.md) §1.1–§1.2 mediu o oráculo. **Os
dois números concordam**, na mesma peça de `98 306` vértices:

| | booleana | volta por campo |
|---|---|---|
| relógio | ⭐ **`67,88 ms`** | `171,7 ms` (res 128) |
| vértices de entrada preservados **ao bit** | `57 489` (`58 %`) | ⛔ **`0`**, em 6 de 6 células |
| vértices **longe** do corte | ⭐ `26 533` de `26 533` intactos | ⛔ `0` |
| contagem de saída | segue a **ENTRADA** | ⛔ segue a **resolução da grelha** |

⇒ **um corte por campo não é um corte: é um corte MAIS um remalhamento da peça inteira.** Uma
escultura tem densidade **autorada** — fino onde o artista trabalhou, grosso onde não — e a volta
por campo devolve-a uniforme. ⭐ *A rota barata é `2,5×` mais lenta **e** destrói o que a outra
preserva* — normalmente há uma troca a fazer, e aqui não há.

⚠️ E o alvo trata cortar e remalhar como passos **separados e sequenciais**: colapsá-los retira do
artista a possibilidade de cortar **sem** remalhar, que é a razão de o passo existir.

## 3 — A decisão

1. **O corte é uma booleana de malha.** A rota por campo fica onde está (o botão de remesh), e
   ⛔ **não** é caminho de corte.
2. **A biblioteca permissiva ENTRA na árvore** — `manifold-rust 0.13.1`, **Apache-2.0**, **Rust
   puro**.
3. **Ela vive atrás de uma fronteira nossa** — a crate-folha
   [`ph2d-mesh-bool`](../../../crates/ph2d-mesh-bool/), com **cinco** coisas na superfície
   (converter · importar · **ler o estado da entrada** · operar · converter de volta). *Trocar o
   motor é reescrever **um ficheiro**.*

### 3.1 — Porque ela entra, quando as outras duas ficaram FORA

O ADR-0162 e o ADR-0167 puseram bibliotecas **fora** da árvore, como oráculos que se correm. A
diferença aqui **não é de gosto e não é de arquitectura — é de licença**:

| | ADR-0162 / 0167 | aqui |
|---|---|---|
| licença do motor | **GPL** / **MPL-2.0** | ⭐ **Apache-2.0** |
| consequência de portar | publicar ficheiros no subsistema mais valioso | nenhuma, além da atribuição |
| veredito | oráculo **fora** | **entra** |

⭐⭐ **E a triagem parte em DUAS metades, e só uma paga clean-room:** a **lei do gesto** (como o
desenho vira volume, o que decide a profundidade, o que se recusa) é **T2** e vive na espec sob a
parede; o **solucionador de omissão** é **T0**. ⚠️ E o T0 é **por solucionador**: o alvo documenta
três, e os outros dois são código dele sob copyleft — *escolher solucionador é também escolher
degrau*.

### 3.2 — Rust puro, e a razão é o CI

| rota | o que a build exige | relógio a frio |
|---|---|---|
| ligação a C++ | ⛔ `cmake` + `cc`, e o `build.rs` **clona por `git`** e compila | `1 min 39 s` |
| ⭐ **Rust puro** | **nada** — sem `cc`, sem `cmake`, **sem rede** | **`16,66 s`** (medido nesta janela) |

A matriz de CI é **linux + macOS + windows**: a rota de ligação põe um `git clone` e uma build de
CMake dentro de cada `cargo build`, nos três. ⚠️ A porta de fuga (apontar uma instalação nativa)
**move** o problema para os runners, não o resolve.

⚠️ **O risco da escolhida é a IDADE, e é o único** — e a mitigação é a fronteira da §3.3, não uma
promessa.

## 4 — As duas coisas que esta decisão OBRIGA

### 4.1 — ⛔⛔ O motor falha em SILÊNCIO, e a lei é nossa

Reproduzido nesta janela, fora do repo, com o motor cru:

```text
FECHADA   entrada status=NoError      → corte OK (16 V, 28 T)
ABERTA    entrada status=NotManifold
ABERTA    RESULTADO status=NoError  V=0  T=0   ← a armadilha
```

⇒ com peça **aberta** ele devolve malha **VAZIA** e o estado do **resultado** diz *«sem erro»*.
**Quem verificar o resultado entrega uma escultura APAGADA.** ⇒ a `ph2d-mesh-bool` pergunta o estado
da **ENTRADA antes de operar** e recusa **nomeando o lado**, com gate e prova de mutação — e a
mutação que tira a pergunta faz a porta recusar *«este corte apagaria a peça inteira»* sobre uma
peça que só tem um buraco: **a cura errada nomeada**.

⚠️ E o motor «robusto» **não** resgata malha aberta: ele aceita *soup fechada* (suja). *É a
distinção entre «suja» e «aberta».*

### 4.2 — A licença que o nosso próprio portão não tinha

Das `12` dependências transitivas, **onze** são MIT/Apache-2.0 e **uma** — `clipper2-rust` — é
**BSL-1.0**, que o [`deny.toml`](../../../deny.toml) **não** tem na lista geral (é concedida
por-crate ao clipboard). ⇒ entrada nova, **nomeando quem a puxa**, como as duas que já lá estavam.
⛔ Sem ela o `./scripts/ship.sh` reprova — e ele é o **último** portão antes do push. Medido nos dois
estados: com a entrada, `licenses ok`; sem ela, `licenses FAILED · rejected: license is not
explicitly allowed`.

## 5 — Consequências

- ⭐ O corte preserva a densidade autorada, e há **gate ao bit** a defendê-lo
  (`longe_do_corte_nenhum_vertice_se_move_um_bit`): *se alguém trocar o motor por um que re-tessela
  tudo, é este gate que reprova* — nenhuma contagem agregada o faria.
- A superfície de uso são cinco chamadas, logo a dependência é **substituível**.
- ⛔ **Uma peça aberta não se corta** — ou se fecha antes (o remesh tapa buracos), ou se recusa em
  voz alta. **Fechar a peça é trabalho nosso**, nunca da dependência.
- ⏳ Fica por medir se o `clipper2-rust` é evitável (ele é clipping **2D**, e o corte é 3D); hoje
  não há feature-flag que o corte.

## 6 — Alternativas medidas e recusadas

| alternativa | porque não |
|---|---|
| a volta por **campo** | §2 — `2,5×` mais lenta **e** destrói a densidade autorada |
| **ligação** a C++ | §3.2 — `cmake` + `git clone` dentro de cada build, nas três plataformas |
| **escrever** a operação | `45 270` linhas (`13 746` só no motor robusto), e o algoritmo de omissão **não tem paper** ⇒ «da literatura» seria lê-lo do código |
| uma 4.ª rota Rust | ⛔ **não existe no registo de pacotes**, e a licença dela tinha sido lida numa **página** e não num artefacto — o R-pré apanhou-o, e o nosso `deny.toml` proíbe dependência de `git` (`allow-git = []`) |

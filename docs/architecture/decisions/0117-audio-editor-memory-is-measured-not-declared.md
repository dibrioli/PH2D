# ADR-0117 — A memória do Audio Editor é MEDIDA, não declarada

- **Status:** ACCEPTED
- **Data:** 2026-07-12
- **Linha:** `line/audio` (Modo L)
- **Contexto:** fecha a dívida de memória levantada pela auditoria do W5 ([ADR-0122](0122-audio-spectral-fft-via-realfft.md))
- **Toca:** `ph2d-audio` (foundational — adição append-only), `ph2d-audio-edit`, `SKILL_Stack` §HR-13 + §12.1

---

## 1. Contexto

A auditoria do W5 apontou "a rack copia o buffer inteiro a cada op". Fui medir para escrever este
ADR e o que encontrei é **pior e diferente** do que eu tinha reportado.

### 1.1 As três medições

Todas reproduzíveis, em `crates/ph2d-audio-edit/tests/measure_*.rs` (dhat, `--release`):

| # | cenário | medido |
|---|---|---|
| **M1** | 64 edições num clipe estéreo de **3 min** (66 MB) | **pico de 4351 MB** — 66× o clipe |
| **M2** | editar **1 s** de um clipe de 180 s (0,56% do áudio) | **133,3 MB em 6 blocos** — 364× a seleção · **26,3 ms** |
| **M3** | um render do efeito mais barato da rack (`Saturate`) | **2 blocos, 2,0× o clipe** |

O teto de app **inteiro** da plataforma (`Platform::max_total_mb`, desktop) é **3500 MB**. M1
estoura o app sozinho, com um clipe e um usuário fazendo o seu trabalho.

### 1.2 O que eu tinha reportado, e por que estava errado

Eu disse ao Enio que "o budget de 30 MB do HR-13 já não descreve o Audio Editor". Errado em dois
níveis, e vale registrar porque a correção é o coração deste ADR:

1. **Os 30 MB são a coluna iPad.** Desktop são 80 MB. Eu estava citando o número de celular para
   uma ferramenta de desktop.
2. **E nada disso importa, porque o HR-13 nunca mediu isto.** `ph2d_core::budget::check_budget`
   **soma structs `MemoryBudget` declarados no boot** e compara com o teto da plataforma. Ele não
   observa **um único byte de fato alocado**, e não existe sequer um sítio de declaração para o
   editor — a linha "Audio buffers" descreve a residência de samples do **mixer em runtime, num
   jogo distribuído**, não o working set de um editor offline.

O erro não foi o número. Foi supor que a regra estava olhando.

> **Um budget que só é declarado não pode ser violado — só excedido em silêncio.**

Este é o achado que generaliza para além do áudio, e é o D4.

### 1.3 A causa-raiz, numa frase

**O editor trata toda edição como se produzisse um clipe novo, quando uma edição é um INTERVALO.**

`ops::in_range` esplica construindo `Vec::with_capacity(src.len())` — o clipe **inteiro** —,
copiando por dentro dele a cabeça e a cauda intocadas, e entrega esse `Vec` a
`SampleData::from_interleaved`, que aloca o clipe inteiro **outra vez** e faz memcpy. As amostras
fora da seleção são copiadas **duas vezes pelo privilégio de não serem tocadas**.

E `Arc::from(Vec)` **não pode** reaproveitar o buffer do `Vec`: um `Arc` guarda o refcount inline,
imediatamente antes dos dados, e a alocação de um `Vec` não tem espaço para ele. A segunda cópia
não é um descuido — é estrutural na escolha do tipo.

O histórico (`MAX_HISTORY = 64`) é o mesmo defeito em regime permanente. `SampleData` é
`Arc<[f32]>`, então `data.clone()` para o histórico é **bump de refcount, de graça** — a memória
não vem do clone. Vem de que **cada edição produz um buffer NOVO**, e o histórico retém todos.
`MAX_HISTORY = 64` não é um teto de bookkeeping: é um **multiplicador de 64× sobre o clipe**.

---

## 2. Decisão

### D1 — O histórico guarda DELTAS, e é capeado por BYTES

Uma edição muda um intervalo. O passo de undo guarda **as amostras antigas daquele intervalo** e
nada mais. Undo = esplicar de volta.

- **Edição de seleção** (o caso comum num clipe longo — você conserta um clique, você faz denoise
  de um trecho; ninguém satura 3 min de ambiência): o passo custa **proporcional à seleção**.
- **Edição de clipe inteiro:** o passo custa um clipe. Isso é **honesto e irredutível** — não se
  desfaz uma mudança de clipe inteiro sem guardar o clipe inteiro.

Por isso o cap vira **bytes, não contagem**. `MAX_HISTORY = 64` conta passos ignorando o tamanho
deles, que é precisamente o erro. Um orçamento em bytes descarta os passos mais antigos quando
estoura, e a **profundidade de undo passa a ser adaptativa**: um SFX curto ganha centenas de
passos; um clipe de 66 MB ganha os que couberem. O usuário nunca perde o app; perde undo antigo,
que é a troca certa e a que todo DAW faz.

### D2 — Os buffers são construídos UMA vez

`ph2d-audio` ganha um construtor de escrita-única (**adição append-only**, projetada para
isolamento conforme CLAUDE.md §0.2 — nenhuma mudança de layout, nenhum toque no hot path do mixer
RT):

```rust
/// Escreve cada amostra exatamente uma vez — sem `Vec` intermediário, sem memcpy.
pub fn from_fn(len: usize, format: AudioFormat, f: impl FnMut(usize) -> Sample) -> Self
```

Implementado como `(0..len).map(f).collect::<Arc<[Sample]>>()`. `Arc<[T]>: FromIterator<T>`
especializa em `TrustedLen`, e `Map<Range<usize>, F>` é `TrustedLen`: aloca o `ArcInner` uma vez e
escreve direto dentro dele.

**Isso é detalhe de implementação da std, então é MEDIDO, não comentado** —
`tests/measure_arc_build.rs` prova 2 blocos → **1 bloco**, pico pela metade. Se a especialização
mudar, o gate fica vermelho e este ADR precisa ser repensado, em vez de degradar em silêncio.

Ambas as crates mantêm `#![forbid(unsafe_code)]`. `Arc::new_uninit_slice` + `assume_init` resolveria
também, e exigiria `unsafe`; a rota `TrustedLen` entrega o mesmo 1-bloco sem gastar essa moeda.

### D3 — O teto do editor é um gate EXECUTÁVEL que MEDE

Não um número declarado. Os cenários M1–M3 viram gates dhat pinados no conjunto de aceite (§4).
É a única espécie de budget capaz de ficar **vermelha**.

### D4 — Emenda ao HR-13

> **HR-13 (emenda):** um subsistema que declara um `MemoryBudget` **possui também um gate
> executável que MEDE o consumo real** contra ele. Um budget só declarado é um desejo: `check_budget`
> soma intenções no boot e nunca observa uma alocação. Subsistema sem gate de medição não tem
> budget — tem uma opinião.

E `SKILL_Stack` §12.1 ganha a linha que faltava: **Audio Editor** (desktop) — o working set do
editor offline (clipe + histórico + preview), distinto de "Audio buffers", que é a residência do
mixer em runtime.

---

## 3. Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **Aumentar o número do HR-13** | Trata o sintoma e ratifica o desperdício. 364× o trabalho necessário é um defeito **em qualquer budget**. E não conserta a cegueira: o número continuaria não sendo medido. |
| **`SampleData` vira `Arc<Vec<f32>>`** | `Arc::new(vec)` de fato não copiaria. Mas acrescenta uma indireção em `frame_stereo`, que roda **por-frame na thread RT** — pagar no mixer de um jogo distribuído para consertar um editor offline é o blast radius errado. `Arc<[f32]>` fica intacto. |
| **`Arc::new_uninit_slice` + `assume_init`** | Resolve, e custa `unsafe` em duas crates que hoje o proíbem. A rota `TrustedLen` entrega o mesmo 1-bloco de graça. Gastar `unsafe` só quando não há alternativa — foi o critério do [ADR-0116](0116-audio-export-opus-isolated-unsafe-crate.md) e vale aqui. |
| **Histórico em disco (Audacity)** | Resolve o teto de vez, e traz I/O, tmpfiles e um formato para um problema que o delta + cap em bytes já bota dentro do orçamento. Se um dia clipes de 30 min entrarem em escopo, é a resposta certa — não hoje. |

---

## 4. Conjunto de aceite (CONGELADO antes de eu escrever uma linha de implementação)

As barras saem das medições de §1.1, não do que a implementação conseguir entregar.

| # | gate | hoje | barra |
|---|---|---|---|
| **A1** | 64 edições **de seleção** (1 s) num clipe de 180 s: pico | 4351 MB | ~~≤ 128 MB~~ → **≤ 2×clipe + 32 MB** (ver §4.1) |
| **A2** | 64 edições **de clipe inteiro** num clipe de 180 s: pico | 4351 MB | **≤ 512 MB** (o cap em bytes, não a sorte) |
| **A3** | render de 1 s dentro de 180 s: blocos / alocado | 6 / 133,3 MB | **≤ 3 blocos / ≤ 70 MB** |
| **A4** | render de clipe inteiro: blocos | 2 | **1** |
| **A5** | `collect::<Arc<_>>` aloca 1 bloco (a premissa do D2) | — | **verde, ou o ADR cai** |
| **A6** | **todo efeito da rack byte-idêntico** ao caminho pré-refactor | — | **39/39**, contra um oráculo `#[cfg(test)]` |
| **A7** | undo/redo restaura **byte-a-byte** (delta esplicado == snapshot) | — | verde |
| **A8** | hot path RT (`frame_stereo`, layout de `SampleData`) | `Arc<[f32]>` | **inalterado** |

**A6 é o gate que protege o invariante da rack.** O refactor toca o caminho por onde passam os 39
efeitos; a aritmética tem de sair idêntica. O oráculo é a implementação de hoje, preservada em
`#[cfg(test)]` — o mesmo padrão de `lpc.rs::solve` (contra Levinson) e `convolve.rs::direct`
(contra o FFT overlap-add) nesta mesma linha.

**A8 é a cerca do foundational.** `ph2d-audio` é o mixer RT (HR-3). Este ADR **adiciona** um
construtor e um handle CoW; não muda um byte do que a thread de áudio executa.

### 4.1 EMENDA — a barra do A1 estava aritmeticamente errada (registrada, não apagada)

Congelei o A1 em **≤ 128 MB**. A implementação mediu **156,3 MB**, e a culpa é da barra, não da
implementação. A conta:

```
 65,9 MB  o clipe (self.data)
 65,9 MB  o buffer NOVO sendo construído
 23,4 MB  os 64 deltas (1 s estéreo cada)
--------
155,2 MB  + o peak cache = 156,3 MB medidos
```

**Dois buffers cheios são irredutíveis.** Um buffer novo tem de existir antes que o antigo possa
ser solto — e no produto o caminho é sempre `render_effect` (preview, que o mixer TOCA antes de
você commitar) seguido de `commit_rendered`. `2 × 65,9 = 131,8 MB` **já estoura os 128 sozinho, com
histórico zero**. A barra era impossível de bater. Eu a escrevi antes de entender que o buffer de
preview é estrutural, e não um desperdício.

**O que eu NÃO fiz:** existe um caminho in-place (`SampleData::samples_mut`, o handle CoW) que faria
`apply_effect` escrever por cima do clipe e o gate passaria em ~90 MB. **O produto não usa esse
caminho** — ele renderiza preview. Passar o gate por ali seria maquiar o número, não consertar a
memória.

A barra vira **estrutural em vez de absoluta**, que é o que ela devia ter sido desde o início:

> **A1: pico ≤ 2 × clipe + 32 MB.**

Ela não depende do tamanho do clipe, e diz a coisa certa: *o editor segura o clipe, um buffer em
construção, e deltas — **não N clipes**.* É essa a propriedade que os 4351 MB violavam.

Medido: **156,3 MB** contra um teto de 163,8 MB. **28× melhor** que os 4351 MB.

---

## 5. Fora de escopo (declarado, não fingido)

**Preview de render verdadeiramente O(seleção).** Mesmo depois do D2, arrastar um knob num clipe de
3 min re-renderiza um buffer de preview **inteiro** por frame — porque o mixer toca esse buffer, e
tocar exige um `SampleData` contíguo. Chegar a O(seleção) no preview exige scratch com
double-buffer (o mixer segura um `Arc` enquanto escrevemos no outro), o que muda o contrato de
preview/playback.

**Isso NÃO é resolvido aqui, e o número fica registrado em vez de escondido:** o D2 leva o custo
por frame de 133 MB / 6 blocos para a barra A3, e o resto continua O(clipe). Se o Enio quiser
knobs fluidos em clipes longos, é o próximo ADR, e ele tem um alvo medido para mirar.

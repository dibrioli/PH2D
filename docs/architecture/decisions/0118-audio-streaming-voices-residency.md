# ADR-0118 — Vozes por STREAMING: o codec passa a valer para a RAM

- **Status:** ACCEPTED
- **Data:** 2026-07-12
- **Linha:** `line/audio` (Modo L)
- **Contexto:** o outro lado da cerca do [ADR-0117](0117-audio-editor-memory-is-measured-not-declared.md) — a memória do **runtime**, que é o que o HR-13 de fato governa
- **Toca:** `ph2d-audio` (foundational/RT — adição append-only), crate nova `ph2d-audio-stream`, shell

---

## 1. Contexto

O ADR-0117 emendou o HR-13: *quem declara budget possui um gate executável que **MEDE***. O Audio
Editor ganhou os seus. Escrevi então o mesmo gate para o outro lado — o **mixer que embarca dentro
de um jogo**, que é o que a linha "Audio buffers" da §12.1 (30 MB no iPad, 80 MB no desktop) sempre
descreveu.

Ele nasceu **vermelho** (`crates/ph2d-audio/tests/the_mixer_fits_its_budget.rs`):

```
one 3-minute stereo track, decoded: 65,9 MB
HR-13 'Audio buffers', iPad:        30 MB
over budget by:                     2,2x
```

**Uma única música estoura o orçamento inteiro de áudio do iPad em 2,2×, antes de um único efeito
sonoro.** E não é um asset exótico: é *uma canção*.

### 1.1 A causa

`Voice` guarda `Option<SampleData>` — **o clipe inteiro, decodificado em `f32`**. Não existe
caminho de streaming no mixer (`grep -i stream crates/ph2d-audio/src/` acha só a fila de
**comandos**). Todo asset que toca está residente, inteiro, enquanto tocar.

### 1.2 O corolário desconfortável

**O seletor de codec que a linha acabou de construir ([ADR-0113](0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md)/[ADR-0116](0116-audio-export-opus-isolated-unsafe-crate.md)) não economiza um byte de RAM.** Opus é
**6,4% do WAV16 em disco e 100% dele na memória**, porque a primeira coisa que o load faz é
expandir tudo de volta para `f32`.

O painel Delivery é **honesto** sobre isso (mostra a mesma cifra de RAM para qualquer codec). Mas
ser honesto sobre um buraco não é não ter o buraco. O codec só passa a significar alguma coisa para
a memória **quando o áudio não precisa estar todo residente para tocar** — que é este ADR.

---

## 2. Decisão

**Uma voz pode tocar a partir de um STREAM em vez de um buffer residente.** O áudio é decodificado
**à frente, aos pedaços**, numa thread produtora; a thread de áudio apenas **consome**.

### D1 — O contrato da thread RT: ela só POPA (HR-3)

A thread de áudio **não decodifica, não aloca, não libera, não trava**. Ela tira `Chunk`s de um
ring limitado, lê, e **devolve o chunk gasto por um segundo ring** — porque *soltar* um chunk na
thread de áudio seria um `free()`, e um `free()` é uma alocação de trás para frente.

Isso não é invenção: é o padrão que a crate **já usa** para as `SampleData` de vozes que terminam
(*"a finished voice's `SampleData` is shipped back to the control thread to be dropped off the RT
thread (HR-3)"*, `buffer.rs`). O streaming é o mesmo movimento, em regime.

```
produtor  --[ ring: chunks CHEIOS ]-->  thread de áudio
produtor  <--[ ring: chunks VAZIOS ]--  thread de áudio        (reciclagem; zero free no RT)
```

### D2 — Nenhum codec alcança o mixer RT

`ph2d-audio` ganha o **ring e a voz por stream**, e **nenhuma dependência de codec** — a razão de
`ph2d-audio-decode` existir separada desde sempre. Quem decodifica é a crate nova
**`ph2d-audio-stream`** (depende de `-decode` + `-opus`), que roda no produtor.

`ph2d-audio` expõe as duas metades e nada mais:
- `StreamHandle` — a metade da thread de áudio (popar frames).
- `StreamFeeder` — a metade do produtor (encher chunks).

Assim o foundational cresce por **módulo irmão** (`stream.rs`) e ponto de extensão, como manda o
CLAUDE.md §0.2, e o mixer segue `#![forbid(unsafe_code)]` e livre de codecs.

### D3 — Streaming soa EXATAMENTE igual a residente

O cursor de uma voz **só anda para a frente**. Logo acesso sequencial basta, e a interpolação linear
(`i0`, `i0+1`, `frac`) pode ser reproduzida **bit a bit** sobre uma janela deslizante de dois
frames. O loop cai de graça: o produtor emenda o começo do arquivo, e o "interpolar contra o frame
0" que a voz residente faz no wrap vira simplesmente *o próximo frame do stream*.

Isso vira o gate mais forte deste ADR (A2): **o mesmo clipe, residente e por stream, produz o mesmo
buffer de saída, byte a byte.** Se streaming soasse "quase igual", seria um bug que ninguém acharia.

### D4 — Underrun é silêncio contado, não um glitch

O produtor não chegou a tempo (disco lento, thread preemptada): a voz emite **silêncio naquele
frame** e **conta** o underrun. Ela não morre, não estala, e não trava esperando. O contador é
legível — um underrun invisível é um bug que só aparece no aparelho do jogador.

---

## 3. Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **Decodificar tudo, mas guardar comprimido e expandir sob demanda** | É streaming com outro nome, e pior: sem ring, a expansão cairia na thread de áudio. |
| **Baixar a taxa/bit-depth dos assets residentes** (i16 em vez de f32) | Metade da memória, e a conta continua perdida: 33 MB por música ainda estoura os 30 MB. Trata o sintoma. |
| **Ring de FRAMES em vez de chunks** (`ArrayQueue<[f32;2]>`) | Lock-free e zero-alloc, mas um push/pop atômico **por amostra** — o custo de sincronização domina o trabalho. Chunk amortiza. |
| **Deixar o produtor alocar chunks novos e o RT soltá-los** | Um `free()` na thread de áudio. Viola o HR-3 e é exatamente o que o ring de reciclagem existe para evitar. |
| **`mmap` do arquivo + decodificar no RT** | Decodificação na thread de áudio (não-determinística, aloca) e page-fault de disco no callback. Não. |

---

## 4. Conjunto de aceite (CONGELADO antes da implementação)

| # | gate | hoje | barra |
|---|---|---|---|
| **A1** | música de 3 min tocando: residência | 65,9 MB | **≤ 2 MB** (o ring, não o clipe) — e o gate do HR-13 fica **verde** |
| **A2** | mesmo clipe, residente vs stream: saída | — | **byte-idêntica** (D3) — inclusive com rate ≠ out_rate e em loop |
| **A3** | thread de áudio: alocações num render quente com voz por stream | — | **zero** (capacidade estável, como `no_alloc_render.rs`) |
| **A4** | underrun | — | silêncio + **contado**; a voz **sobrevive** e retoma |
| **A5** | fim do stream (EOF) | — | a voz termina **exatamente** onde a residente terminaria |
| **A6** | `ph2d-audio` não ganha dependência de codec | — | verde (gate de dependência) |

**A2 é o coração.** Um streaming que soa "quase igual" é indistinguível de um que soa igual, até o
dia em que não é.

---

## 5. Fora de escopo (declarado, não fingido)

- **Seek/scrub num stream.** O transporte do editor faz seek em vozes residentes; num stream isso
  exige o produtor re-posicionar o decoder e **descartar o ring**. Fica para quando houver um
  consumidor real (hoje o preview do editor é residente por construção — o clipe está aberto).
- **Pitch em tempo real num stream.** `advance` fracionário (44,1k → 48k) funciona e está no A2;
  varrer o pitch **ao vivo** muda a taxa de consumo do ring e é uma política de produtor, não de
  mixer.
- **A residência como escolha por-asset na UI.** O toggle "Streamed" no Delivery só faz sentido
  quando o *jogo* carrega assets — hoje o editor abre um clipe para editar, o que é residente por
  definição. **Botão que não faz nada é pior que botão que falta** (a razão pela qual ele não foi
  posto na jornada passada, e ela continua valendo).

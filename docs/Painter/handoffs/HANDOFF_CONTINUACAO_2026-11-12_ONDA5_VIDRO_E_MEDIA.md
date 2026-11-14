# Handoff de continuação — linha PainterWatercolor · 12/11 n.º 5: o vidro do Watercolor Basic e a régua da MÉDIA

> Não é handoff de integração (a linha continua; o link do CLAUDE.md §5.1 só muda no fecho). Leitor: a próxima LLM da linha.
> Anterior: [HANDOFF_INTEGRACAO_…_SELECAO_CAMADAS_E_RITMO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_SELECAO_CAMADAS_E_RITMO.md) (n.º 4).
> Backlog: [`../missao_4k/BACKLOG.md`](../missao_4k/BACKLOG.md) (linha B18.4, «Cont. 6»; B13.12 já actualizada em `c5ca8444cb`).

## 0. Superfície de colisão e estado da árvore

- **A linha FOI rebaseada em `main` a 12/11** (pelo comando de assumir): fica sobre `main` `4f46a3573c` (505 commits). **NÃO rebaseie de novo.**
  Cópia de segurança da cabeça de antes: ramo `backup/PainterWatercolor-antes-rebase-2026-11-12` (`8246bd897a`).
- Conflitos resolvidos no rebase:
  - `shells/desktop/src/main.rs`: `WindowEvent::Focused(false) => { self.on_focus_lost(); self.perde_a_mao(false); }` (o `on_focus_lost` do main já aplica o FocusLost; o `perde_a_mao` da linha fecha o traço).
  - `PROJECT_SCHEMA`: degraus da linha recontados por `python3 scripts/schema-recount.py`: B50.1 `231→232` virou `237→238`; B34.1 `232→233` virou `238→239`; const = **239**, tripla **(239,13,29)**; comentários corrigidos em `299399bf9b`.
  - `CLAUDE.md` §5.1 = texto do main + a entrada Painter da linha (4 vezes); `docs/archive/.../COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-10.md` por rerere, append-only verificado.
  - Mergiraf resolveu `janela_do_jogo.rs` (2×): o diff da linha contra o main é adição pura (`pede_o_quadro`), conferido.
  - ADR-0203: sem colisão (máximo do main = 0200). `cargo check -p ph2d-host-desktop` verde depois do rebase.
- **Risco para o integrador:** `cargo fmt --all` nesta árvore reformata ficheiros de OUTRAS linhas vindos do main (`ph2d-editor-core` `card_loose_ops.rs`, `tests/it/a_frame_request_has_a_listener.rs`, `ph2d-physics-outline` `tests/pontos/w5.rs`): o main não está fmt-clean e a linha não os tocou. Formate só os ficheiros da linha.
- Ficheiros da onda: `crates/ph2d-wet-paint/examples/sonda_estagios_4k.rs`, `…/advect_commit.rs` (`advect_jacobi_partes`, `#[doc(hidden)]`), `crates/ph2d-tool-painter/src/tool/paint/wetpaint/` (`watercolor_vidro.rs`, `watercolor_smudge.rs`, `watercolor_vidro_segue_tests.rs`, `sonda_primeiro_traco.rs`).
- Contadores (DELTA da onda): gates novos 2 (§2); `PROJECT_SCHEMA` 0 (o 239 é do rebase); ADR 0; cenas de smoke 0; tripla 0.

## 1. Item 1 — a sim a 6 traços (B13.12), commit `c5ca8444cb`

A régua era a MEDIANA; o relógio da sim é a MÉDIA (um passo lento nunca se recupera). A sonda `sonda_estagios_4k` imprime agora média por rodada (a melhor), p90, mín/med/máx por estágio e o advect por partes. 4K, 16 fios, load 4–9:

| traços | mediana | MÉDIA | sim-s/s | nota |
|---|---|---|---|---|
| 6 | 26–28 ms (0,90–0,96) | **32,3 ms** | **0,77** | p90 70 ms |
| 4 | — | 24,0 ms | 1,00 | |
| 2 | — | 13,8 ms | — | |

Os picos: estágio 1 (secagem, 1 em 6) 38–45 ms · estágio 2 (campo de fluxo, 1 em 4) até 11 · estágio 5 (project, 1 em 3) até 7,7 · estágio 0 até 3,5. Advect por partes (mín): momentum 3,4 · núcleo fundido 12,6 · commit 2,8 ms.

### ⛔ Recusas MEDIDAS (todas com hashes idênticos ao bit)

1. Momentum no bloco (rota m): média 32,3×32,7 (6), 24,0×24,2 (4), 13,8×14,4 (2) — sem ganho.
2. «Cura 1b» store-forwarding em `assenta` (`seco()` reescrito para escrever só `molhada`; o `perf annotate` mostrava 42 % do self de `assenta` em `movapd 0x150(%rsp)` depois de stores de 8 bytes do enum `Ecra`): `sonda_secagem_4k` 30 passos 32 fios, memo 38,85×38,33 ms, sem memo 62,2×61,7 — o ponto quente era *skid* da amostragem; revertida.
3. Tamanho da tabela do memo `MEMO_BITS` 12..16 intercalado: 70,4/72,3/69,1/68,0/74,2 ms — ruído; os 70 % de acertos (5,82 M hits × 2,46 M misses) são chaves novas, não colisões.
- Chão da secagem (32 fios): sem `assenta` 17,3 ms · copiar todos os planos 21,4 · memo 38,9 ⇒ ~21 ms é o Newton dos 2,5 M misses (~600 ciclos cada).
- **Conclusão:** os 6 traços em tempo real pela MÉDIA não saem de curas exactas ao bit por célula. O que sobra muda a aritmética da `assenta` (física — **decisão do dono**, fora do mandato) ou um commit por coerência de rascunho (recusado por desenho: 18 ficheiros escrevem `film/susp/susp_rgb` fora do advect — pincel, ferramentas, secagem, snapshot; um que escape muda um bit em silêncio). Aberto, não investigado: ~10 % de `crossbeam_epoch` em espera ativa (operários rayon à espera) no perf a 16 fios.

## 2. Item 2 — Watercolor Basic (WB) no 4K

Sonda `sonda_primeiro_traco.rs` estendida: faltas de página do processo por evento (`/proc/self/stat` campo 10, ×4 KiB = «MB novos»), `PH2D_SONDA_NOVA=1` (2 traços na base, «+», 2 traços na camada nova; com `PH2D_SONDA_PREPARO` espera 60×16 ms depois do «+»), `PH2D_SONDA_VIDRO_ANTIGO=1` (A×B). Comando (fila pesada; o subagente não o corre):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_SONDA_SO="Watercolor Basic" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 PH2D_SONDA_PARTES=1 PH2D_SONDA_NOVA=1 bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture
```

### 2.1 Achado (a) — TODO quadro de um traço WB tocava 58 MB de memória nova (`a0b5c360e6`)

Mecanismo (achado com instrumentação temporária de faltas, depois removida): o Smudge (ligado no WB) escreve a base da sessão no lugar (`watercolor_smudge.rs` `smear_wet_base` → `plane_fork::fork_plano_de_trabalho` → `Arc::make_mut`). Com o `Weak` do `VidroCongelado` vivo, o `make_mut` **MOVE o valor para um `Arc` novo** (mesmo buffer) e o `Weak` morre (strong 0) ⇒ `abre_o_vidro` (`watercolor_vidro.rs`) re-congelava a partir do plano JÁ escrito no quadro anterior e `par_clone`ava-o (n×7 B = 58 MB) em todo quadro. A 1080p o glibc reaproveita a memória libertada e o custo escondia-se; alocações ≥ 32 MB são `mmap`, então a 4K há falta de página em todo quadro.

Cura: `VidroCongelado::a_base_mudou_de_arc(antes, agora)` religa só o `Weak` que apontava exactamente para o `Arc` acabado de mover (captura-se `Arc::as_ptr` um instante antes; sem risco de reuso de endereço); chamada no fim de `smear_wet_base`. Interruptor A×B: `thread_local` `VIDRO_SEGUE_A_BASE` (`doc(hidden)`).

Medido, 4K, load 1–3, mesmo binário:

| | antes | depois |
|---|---|---|
| memória nova por quadro | 58,1 MB | **0** |
| tique | 7,4–8,3 ms | **2,6–3,5 ms** |
| quadro mediano do traço (1.º/2.º na camada nova) | 8,6 / 9,2 ms | **3,8 / 3,7 ms** |

Pixels do canvas idênticos ao bit nos dois caminhos (o plano do vidro só conta em papel colorido; passou a ser o vidro da base do início do traço, como já era sem Smudge — antes, com Smudge, o `vbase` derivava para o vidro do quadro anterior). Gate `o_vidro_segue_a_base_que_o_borrao_move_sem_copiar_o_plano` (`watercolor_vidro_segue_tests.rs`; controlo: o caminho velho recopia ≥ moves−2 vezes; mutação `strong_count==99` ⇒ vermelho). Verdes por nome: vidro 19, smudge 7, smear 17.

### 2.2 Achado (b) — o vidro que nasce zerado era copiado (`88263d549a`)

O plano do vidro recém-criado (`vec![[0;7];n]`, páginas preguiçosas) era `par_clone`ado logo porque o congelado o segura ⇒ 58 MB tocados no 1.º toque de TODA camada nova. Cura: se nasceu nesta chamada, a «cópia» é outro plano zerado (ao bit). Contador de teste `COPIAS_DO_VIDRO` (`thread_local`, conta `par_clone`s reais). Gate `o_primeiro_traco_da_camada_nao_copia_o_vidro_que_nasce_zerado` (controlo: o 2.º traço copia ≥ 1; mutação `nasce && false` ⇒ vermelho). Medido 4K load 8: 1.º toque da camada nova **206 → 91 MB** de memória nova; 1.º toque da base 384 → 269 MB; o tempo do clique continua 38–43 ms (ruído de carga).

Decomposição do pen-down WB 4K (instrumentação temporária, antes da cura b): `freeze_watercolor_ground` 7–13 ms (0–66 MB) · `stamp_stroke_dabs` 2–13 ms (9–109 MB) · 1.º `apply_watercolor` 19–29 ms (91–200 MB) · `paint_begin` inteiro 23–50 ms.

### 2.3 Ainda aberto no WB

- O **canvas fork (33 MB) + o COW do vidro (58 MB)** em todo traço depois do primeiro: o snapshot de undo (`stroke_undo.vidros` / canvas) segura o plano inteiro (copy-on-write inerente) ⇒ pede **undo em janela/tiles** — mudança de desenho (ADR primeiro). NÃO começado.
- `stroke_coverage` / `stroke_color` / `stroke_water` / `backdrop` de tela inteira são realocados por sessão nova (`clear()` + `vec!` depois): dá para reutilizar a capacidade e zerar só o sujo (ao bit). Não feito.

## 3. Item 3 — o nascer da sessão Wet (só medido)

Com a espera realista depois do «+» (preparo pronto), Wet Wash 4K camada nova: pen-down **3,9 ms · 27 MB** (sem a espera: 87,8 ms · 1,47 GB — o motor nasceu sem preparo). O **3.º evento do 1.º traço: 19,9 ms · 91 MB** (aberto: é onde o custo está de verdade). O «+» custa 67 ms de UI (os 1,16 GB do contador incluem a thread de fundo do preparo a montar o próximo motor 4K — contador do processo inteiro). Preparo usado: (2,2).

## 4. Prova de fecho da onda

- `clippy -D warnings` limpo em `ph2d-wet-paint` e `ph2d-tool-painter`.
- Binário smoke `target/smoke/ph2d-host-desktop`: 2.ª compilação `Finished smoke profile [optimized] target(s) in 0.22s`, zero `Compiling`.
- **Alarmes de pincel VERMELHOS com 77 diferenças** (`a_largura_de_fabrica_e_a_aprovada`: 2 larguras — Oil Wash, Smear Wet; `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`: miniaturas, incl. muitos Digital — Hard Round 66 px, Soft Airbrush 10033 px, Gouache Blender 16995 px…). A lista é **IDÊNTICA, com os `fnv`, com e sem a cura do Smudge** (corrida-base com a cura revertida) ⇒ não é desta onda. O n.º 4 listava só 3 miniaturas vermelhas (WB, Brush Pen, Mapping Pen) ⇒ o excedente veio muito provavelmente com o rebase (mudanças de motor de outras linhas) — **NÃO verificado na cabeça pré-rebase** (`backup/PainterWatercolor-antes-rebase-2026-11-12`). O relatório vai ao dono; só ele aprova regravar.

## 5. ABERTO, por ordem

1. Wet: o 3.º evento do 1.º traço na camada nova (19,9 ms · 91 MB) e o «+» de 67 ms.
2. WB, 1.º toque ~38–43 ms: reutilizar os buffers de tela inteira por sessão (coverage/color/water/backdrop), ao bit; o COW por traço do canvas+vidro pede undo em janela (ADR antes).
3. Re-medir a matriz do app com calma (load < 5): `bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel` (e com `NOVA=1`) para confirmar o ganho por quadro do WB no app.
4. B19.10 ruído, B20.6 a parte da placa, B23.2; a espera ativa de ~10 % do rayon no perf da sim.
5. Sim a 6 traços pela MÉDIA: só com decisão do dono (física) — diga-o a ele em palavras simples.
6. Verificar se os 77 vermelhos dos alarmes já existiam antes do rebase (corrida na cópia de segurança).

## 6. Regras aprendidas

- **A régua do tempo real é a MÉDIA, nunca a mediana:** o passo lento não se recupera.
- **Um `Weak` como chave de identidade morre quando alguém faz `make_mut` com ele vivo** (o valor MUDA de `Arc`): quem move tem de religar.
- **Alocação ≥ 32 MB é `mmap`:** libertada e pedida por quadro = faltas de página em todo quadro; a 1080p o glibc esconde-o.
- **`perf -e page-faults -c 1` acha a cópia (`memcpy` da libc), mas sem `-g` não diz quem:** instrumente por fases com `/proc/self/stat`.
- **`cargo fmt --all` nesta árvore formata ficheiros de outras linhas vindos do main:** reverta-os, formate só os da linha.

## 7. Smoke do dono (cartões)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`
2. Abra ou crie uma imagem de 3840×2160, escolha o pincel «Watercolor Basic» e pinte um traço comprido, devagar.
3. Tem de acontecer: o traço acompanha a caneta sem engasgar enquanto pinta (cada quadro pesava ~2× mais antes). Depois, em Layers, clique em «+» e pinte na camada nova: o primeiro toque não pode sentir-se pior que antes.
4. Deu errado se: o traço fica atrás da caneta durante o desenho, ou, em papel colorido, as cores fora da mancha mudam enquanto pinta.
5. Abra o [relatório dos pincéis](Worktrees/line-PainterWatercolor/target/relatorio-pinceis/index.html) e decida os 77 pincéis que mudaram (aprovar ou devolver) — não são desta onda.

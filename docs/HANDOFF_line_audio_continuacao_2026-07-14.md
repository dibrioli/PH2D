# Handoff de continuação — `line/audio` (pós-integração de 2026-07-14)

> **Você é a linha de áudio.** A jornada anterior (W3 + W4 + W6 + ADR-0120) **integrou**. Este
> documento é o seu briefing: onde a linha está, o que sobrou, e as minas que já explodiram uma vez.
>
> **Modo L** (worktree própria, ADR-0107). Você **NÃO integra e NÃO faz ship** — fecha, escreve o
> handoff de integração (DIRETRIZ §1.5.9) e **PARA**. Integração e ship são **ordem explícita do
> Enio**, via agente integrador.

---

## 1. Onde a linha está

| | |
|---|---|
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio` |
| Branch | `line/audio` (rebaseada sobre o `main` pós-integração) |
| Base | `4d203d48` — o `main` de depois das 6 linhas |
| Estado | **417/417** verdes na árvore combinada · worktree limpa |

**A rack tem 42 efeitos e 23 presets.** O que fechou na última jornada está no
[registro de integração](REGISTRO_integracao_jornada_2026-07-13.md) e no
[handoff de integração da linha](HANDOFF_INTEGRACAO_line_audio_2026-07-13.md) (apêndices §7–§9 —
leia antes de mexer no código).

---

## 2. A fila (triada — metade do que "sobrou" não é o que parece)

### 2.1 🔴 Bloqueado no Enio

| Item | Por quê |
|---|---|
| **W7 — AI/ML** (DeepFilterNet, Demucs via ONNX) | **Deps pesadas novas** ⇒ exige **ADR + autorização explícita**. **Zero linha de código antes disso.** O kill-criterion do [ADR-0122 §4](architecture/decisions/0122-audio-spectral-fft-via-realfft.md) não disparou — o denoise clássico (Ephraim-Malah) atendeu, então o W7 é *ganho incremental*, não necessidade |
| **Rename `"Gate"` → `"Gate / Expander"`** | O Expander **já existe** — é o `Effect::Gate` (o doc dele diz isso). O rename é puro ganho de descoberta, **mas os presets resolvem kind por NOME** (`kind_by_name`, gate `parse_is_keyed_by_name_not_index`): um preset dizendo `"Gate"` passaria a resolver `None` e **sumiria em silêncio**. Trade: descoberta × quebrar preset |

### 2.2 🟡 Pendente de smoke (a linha anterior integrou SEM isto)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
  PH2D_AUDIO_DELIVERY_SMOKE=1 cargo run --release -p ph2d-host-desktop   # W6: Export Set
```
> Clipe estéreo com **shimmer de 15 kHz** (que 24 kHz não representa) + loop + 2 markers. Clique
> **Export Set** → 3 arquivos. O Mobile perde o shimmer: **correto** — ele foi *filtrado*, não
> *dobrado* num tom fantasma de 9 kHz (que era o bug). O **ADR-0120** é invisível por construção
> (byte-idêntico); o que se sente é o knob fluido num clipe longo.

### 2.3 🟢 Aberto e acionável (ordem sugerida)

1. **Split do `fx.rs` — é PRÉ-REQUISITO, não higiene.** 689 linhas, **no teto do LOC cap**: o 43º
   efeito **não entra** sem isso. Candidato natural: mover os braços de `apply` para um módulo
   irmão, como o `warmup.rs` já fez. (`fx_presets.rs` está em 596 — o próximo.)
   ⚠️ `fmt` **re-expande** — formate ANTES de medir ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).
2. **Batch LUFS** — normalizar um lote de clipes para um alvo de loudness. O medidor já existe
   (true-peak oversampleado); falta o laço + a UI de lote.
3. **Preview O(seleção) para efeitos de CAUDA** (reverb/delay). O ADR-0120 **não os cobre**: eles
   mudam o **comprimento** do buffer, então não há "região a reescrever". É um problema diferente,
   e o ADR §5 diz isso explicitamente.

### 2.4 ⛔ NÃO construa — são cercas de Chesterton (o motivo ainda vale)

| Débito | Por que a cerca fica de pé |
|---|---|
| Seek/scrub num stream (ADR-0118 §5) | *"Fica para quando houver um consumidor real."* O editor **abre** um clipe — residente por definição. **Não há quem chame** |
| Pitch ao vivo num stream (ADR-0118 §5) | idem |
| Toggle "Streamed" no Delivery (ADR-0118 §5) | O próprio ADR: ***"botão que não faz nada é pior que botão que falta"***. Só faz sentido quando o **jogo** carrega assets |
| Múltiplas regiões de loop (ADR-0119) | O próprio ADR: jogos usam **uma** |

Construí-los hoje = **feature órfã**, que a DIRETIVA proíbe. Se você acha que a cerca caiu,
**mostre o consumidor** antes de escrever a primeira linha.

---

## 3. As minas (todas já explodiram — não pise de novo)

- **`clone()` de `SampleData` bumpa o `Arc`, NÃO copia.** No ADR-0120 isso transformaria o caminho
  rápido em **código morto que nunca roda**, com **todos os outros gates verdes**, e o único
  sintoma seria *"otimizei e não acelerou"*. Use `SampleData::map_in_place`. Existe um gate que
  **conta as vezes que o caminho rápido dispara** (8/8) — [[feedback_an_optimization_needs_a_gate_that_proves_it_fires]].
- **Não pinne wall-clock em teste do workspace.** O `ci-test` é `opt-level = 1`: o DSP fica ~30×
  mais lento e o memcpy (intrínseco da libc) não se mexe — uma barra afinada em release mede o
  **PERFIL**, não o código. Asserte **ratio**. (Os 7 `measure_*` rodam em 6,5 s sob `ci-test`.)
- **Ao adicionar efeito, confira que a mutação MORDE.** A 1ª mutação do Multiband era **cega**
  (removia a metade do allpass que já estava −96 dB abaixo) e o gate ficava verde com o bug dentro.
- **Cheque se o oráculo é ALCANÇÁVEL antes de escrever o gate.** O handoff anterior mandava provar
  que o crossover soma à identidade **byte a byte** — **nenhum crossover real faz isso** (soma a um
  allpass). Escrito às cegas, ele nasceria vermelho com o código CERTO, e o "fix" seria afrouxá-lo
  até não medir nada ([[feedback_check_the_oracle_is_achievable_before_writing_the_gate]]).
- **`MEMORY.md` / listas compartilhadas: só ADICIONE.** A linha passada "limpou" 4 linhas de índice
  que julgou alheias — **eram da main**, que as ganhou depois do fork; a deleção aplicou limpa e
  matou 4 memórias. Remover numa lista compartilhada é operação de **integração**, contra a main de
  HOJE ([[feedback_a_shared_list_is_merged_against_todays_main]]).
- **Escreva memória DIRETO na worktree, por caminho absoluto.** O symlink `~/.claude/.../memory`
  aponta para a árvore **primária** — copiar de lá pra cá importa o WIP das outras linhas.

---

## 4. O invariante da rack (o que torna "adicionar efeito" barato)

Todo efeito é **no-op byte-idêntico no seu ponto neutro** (`is_bypass` espelha os defaults) e o
**painel se auto-popula da tabela `KINDS`**. Adicionar efeito = variant `Effect` + braços
`apply`/`is_bypass`/`warmup` + **uma row na tabela**. **Zero mudança de painel.** 5 gates provam
isso por-efeito (neutro · o arm acorda · os outros knobs são inertes · layout · false-zero).

**`MAX_FX_PARAMS = 4`** é superfície dura do painel (`AEDIT_FX_PARAMS: [NodeId; 4]`), não um número
frouxo — um efeito com 5 knobs **não cabe** sem mexer no painel.

⚠️ **`KINDS: [FxKind; 42]` e `FACTORY: [Preset; 23]` são números que SOMAM entre linhas** — se
outra linha adicionar efeito, o valor certo não existe em nenhum lado do conflito: **conte**.

---

## 5. Aberto fora do áudio (não é seu — só para não tropeçar)

- **4 memórias órfãs pré-existentes** (sem linha de índice): `feedback_widget_is_done_when_a_test_clicks_it`,
  `project_brush_audit_2026_06_18`, `project_diretriz_v68_2026_05_22`,
  `project_painting_removed_layers_effects_kept`. O integrador não decidiu ("ou se indexam, ou se
  apagam") — é do Enio, e **não é do áudio**.

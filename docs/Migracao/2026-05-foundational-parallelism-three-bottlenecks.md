# Foundational lento sob multi-agente — três gargalos ortogonais

**Data:** 2026-05-21
**Status:** Proposta (aguardando aval do Enio para virar ADR + plano de waves)
**Autoria:** debate de 3 agentes, mediado pelo Enio; verificações de código por Claude (sessão Coordenador).
**Motivação:** apesar da migração tool-as-crate (ADR-0027) + Wave 9, o trabalho **foundational** continua inaceitavelmente lento sob operação multi-agente. Este doc isola *por que*, separa a dor em três gargalos independentes, e propõe a ordem de ataque custo-crescente.

---

## 0. TL;DR

> A dor "foundational lento" é **três problemas distintos e ortogonais** empacotados num nome só.
> Atacá-los na ordem errada (rearquitetar antes de medir/destravar) queima semanas.
>
> | # | Gargalo | Natureza | Fix | Custo |
> |---|---------|----------|-----|-------|
> | 1 | **Fila do compilador** — agentes paralelos compartilham `target/` e serializam no lock do Cargo | infra de execução | `CARGO_TARGET_DIR` por slot | horas |
> | 2 | **Colisão de Git** em arquivos de wiring (`widget/mod.rs`, `showcase/mod.rs`, `dispatch_all`) | mecânica de merge | codegen + sync-gate | dias |
> | 3 | **Ripple de build + migração falsa de tools + god-crate** `editor-core` (38k LOC) | arquitetura | extração física real das tools | semana(s) |
>
> **Ordem recomendada: 1 → (medir) → 3 ∥ 2 → (talvez) crate de contrato.**
> Faça o operacional barato primeiro; meça antes de operar; rearquiteture só o que o dado provar necessário.

---

## 1. Como chegamos aqui (o debate)

Três opiniões arquiteturais independentes, cada uma pressionando a anterior.

### Rodada 1 — proposta original
Extrair os 35 widgets de `editor-core` em crates próprios, em camadas (contrato estável embaixo ← widgets ← editor-core/painéis), com codegen do wiring. Alegava cortar o ciclo de validação de ~15min → ~30s.

### Rodada 2 — crítica (largely aceita)
1. **Topologia errada:** na proposta, `editor-core` dependia dos widget-crates → mexer num widget ainda recompila editor-core + os 8 downstream. O ripple **não** era cortado. Para cortar, widgets têm que ser **irmãos** de editor-core (ambos dependendo de um crate de contrato), com agregação num crate-folha que só o binário consome.
2. **Corolário do fan-out:** o ganho de build do split é **inversamente proporcional ao fan-out** do widget. `Button` (usado em todo painel) → split não economiza nada. Widget obscuro → economia real. Split ajuda **menos** nos widgets que mais se toca.
3. **Duas dores, soluções diferentes:** *colisão* morre só com codegen (sem mover arquivo); *ripple* não é resolvido pela camada proposta. Não bundlar as duas num ADR.
4. **35 crates é over-engineering:** uma vez que `mod.rs` é codegen'd, dois agentes editando `button.rs` e `slider.rs` no mesmo crate não colidem. Bastam ~6-8 bundle-crates por categoria, e só se o ripple provar real.
5. **linkme/inventory são errados:** ordem não-determinística (atrito com hard-rule de determinismo). Codegen dá lista explícita/diffável/determinística.
6. **Medir antes de migrar:** decompor os 15min (check-do-crate vs build-downstream vs runtime-dos-gates). História do projeto mostra que test-infra foi a alavanca gigante **sem** rearquitetar.
7. **PanelHost (trait) = 3 touches/200 commits** → serializar no Coordenador é aceitável; não construir abstração pra não-problema. Colisão real com shell é o render-loop/intent-path; princípio: extensão **aditiva** (append enum variant, add field) mergeia limpo, edição **invasiva** (match arm, corpo de fn) colide.

### Rodada 3 — o ângulo que os dois perderam
Os dois romantizaram colisões de **Git** e ignoraram a barreira **física** do ecossistema Rust:

- **Fila do compilador:** dois agentes na mesma máquina compartilhando `target/` serializam no lock do Cargo (`Blocking waiting for file lock on build directory...`). O paralelismo teórico cai a zero na CPU, independente do isolamento de Git.
- **Shim trap:** os tool-crates dependem de `ph2d-editor` (shim) → `ph2d-editor-core`. O grafo **não está invertido**.
- **Migração de fachada:** só o `ToolManifest` migrou pro crate periférico; o algoritmo pesado ficou no core.
- Bala de prata proposta: `CARGO_TARGET_DIR` por slot + `sccache`.

---

## 2. Dados medidos (não estimados)

Estado verificado em 2026-05-21 (`HEAD` = `c806bd2`).

### 2.1 Composição do god-crate `editor-core`
~38.000 LOC num só crate; **8-9 crates/shells dependem dele**.

| subdir | LOC | churn (200 commits, file-touches) |
|--------|-----|-----------------------------------|
| `widget/` | 12.892 (35 arquivos) | **101** |
| `screens/` | 5.784 (30 arquivos; 14 são chrome handlers) | **88** |
| `tools/` | 6.180 | **69** |
| `interaction/` | 7.643 | 28 |
| `panel/host.rs` (trait PanelHost) | 508 | 3 |
| `paint.rs` | — | 2 |

**Leituras:**
- O instinto "o contrato é estável" está **provado**: PanelHost 3, paint 2 em 200 commits.
- "Foundational lento" **não** é dominado por widgets. `screens + tools` (157) > `widgets` (101).
- `screens/` é **metade plugável**: 14/30 arquivos são chrome handlers (já file-per-handler; único toque central é o `dispatch_all`). O resto é composição da tela hero (shell genuíno).

### 2.2 Achados verificados sobre a "migração de tools" (ADR-0027)

A migração tool-as-crate está **falsa/parcial** — confirmado por inspeção:

| Verificação | Resultado |
|-------------|-----------|
| `crates/ph2d-tool-bgremoval/Cargo.toml` deps | depende de **`ph2d-editor`** (shim) → `ph2d-editor-core` (38k). Grafo **não invertido**. |
| `crates/ph2d-tool-bgremoval/src/` | **um único `lib.rs` de 105 linhas** (só manifesto). Fachada. |
| Algoritmo real de bgremoval | em `editor-core/src/tools/bgremoval/{algorithm/guided_filter,chroma,compose}.rs` + GrabCut, params, scratch. |
| dep `image 0.25` | está em **`editor-core/Cargo.toml`** ("imageops::resize for bgremoval thumbnail"). |

Mesma forma para `padding/`, `trim_transparency/`, `make_square/` — todos com corpo dentro de `editor-core/src/tools/`.

**Consequência:** um Implementador encarregado de evoluir bgremoval **edita o core**. A "pasta isolada" dele é uma mentira → invalida cache incremental do workspace e colide no fluxo global de Git. E o crate de UI carrega `image` (manipulação raster) que não precisaria só pra pintar Vello.

### 2.3 Restrição técnica irredutível (Rust/Cargo)
- Rust linka só o que está no grafo de deps → sempre há ≥1 aresta por unidade plugável.
- `build.rs` **não** pode adicionar deps. Glob em `workspace.members` não linka no binário.
- `linkme`/`inventory` coletam só de crates já linkados (ainda exigem a aresta) **e** têm ordem dependente de link-order (atrito com determinismo).
- "Uma linha num arquivo central" é alcançável; "zero acoplamento em compile-time" só com **carregamento dinâmico** (plugins `.so`/`.wasm`) — descartado para UI first-party (sem ABI Rust estável; UI cruzando boundary GPU/allocator/AccessKit; atrito com zero-alloc). A lane de extensibilidade sandboxed do projeto é o **Luau**.

---

## 3. Os três gargalos em detalhe

### Gargalo 1 — Fila do compilador (o nº1 prático)
Dois agentes na mesma máquina, mesmo `target/`, ambos rodando `cargo check`/`test` no ciclo rápido → o segundo **bloqueia** no lock do diretório de build. Todo o isolamento de Git vira teatro: os agentes competem sequencialmente pela fila de compilação local.

> **Nota:** já documentado em [`DIRETRIZ.md`](../IntegracaoMultiAgente/DIRETRIZ.md) §6.4 ("sintoma de cargo lock entre sessões") como problema **conhecido e não resolvido**. Este doc o promove a prioridade.

**Fix:** `CARGO_TARGET_DIR` por slot de agente.
```bash
# slot-1
export CARGO_TARGET_DIR="$PROJ/target/slot-1"
# slot-2
export CARGO_TARGET_DIR="$PROJ/target/slot-2"
```
Cada slot tem seu próprio cache incremental → zero contenção, incremental preservado por slot.

**Ressalvas (não ignorar):**
- **`sccache` NÃO é bala de prata aqui.** sccache **desabilita compilação incremental** (são incompatíveis). O inner loop real dos agentes é `cargo check -p <crate>` morno — onde o incremental é o que dá os ~30s. sccache ajuda build limpo/CI e deps de terceiros, mas pode **piorar** o loop iterativo. **Medir antes de adotar.** Alternativa que preserva incremental: target-dir por slot + `~/.cargo/registry` compartilhado (só os artefatos compilados duplicam — custo one-time por slot, mantido morno pelo incremental).
- **Disco/IO:** N target-dirs × dezenas de GB. DIRETRIZ §6.4 alerta "target/ em disco lento — mover pra SSD local". `MAC_EXTERNO` é drive externo — checar espaço + throughput antes.

### Gargalo 2 — Colisão de Git em arquivos de wiring
Widget novo → editar `widget/mod.rs` (102 linhas de puro `mod x; pub use x::{...}`) **e** o showcase central. Dois agentes colidem → serialização no Coordenador. Mesma coisa no `dispatch_all` do chrome.

**Fix:** codegen + sync-gate (mesma máquina do gate "enum order matches SVGs" dos ícones).
- `cargo xtask sync` varre `widget/*.rs`, `widget/showcase/*.rs`, `screens/hero/chrome/*.rs` → regenera os blocos `mod`/`pub use`/`dispatch_all` ordenados.
- Arch-test de **staleness**: falha CI se o output checado-in divergir da pasta.
- Resultado: agente larga o arquivo na pasta; nunca abre o central; lista determinística/diffável/greppável (vs ordem não-determinística de linkme/inventory).

**Cobre:** widget (churn 101) + metade de screens (chrome). **Ortogonal** ao Gargalo 3 — vale independente de mover tools. (Erro do agente 3 foi rebaixar o codegen confundindo-o com "fiar tools mal-posicionadas"; o alvo do codegen são **widgets**, que estão legitimamente em editor-core.)

### Gargalo 3 — Ripple + migração falsa de tools + god-crate
`editor-core` carrega corpo de algoritmos pesados (image processing) + a dep `image`, e os tool-crates "isolados" reapontam pro core via shim. Editar uma tool → toca o core → ripple nos 8 downstream + colisão de Git + cache invalidado.

**Fix: completar a extração física das tools de verdade.**
- Mover `editor-core/src/tools/<slug>/` (algoritmo + params + icon + state) para `crates/ph2d-tool-<slug>/src/`.
- Mover a dep `image` de `editor-core/Cargo.toml` para os tool-crates que a usam.
- Trocar a dep `ph2d-editor` (shim) dos tool-crates pelo **contrato enxuto** (ver §4).
- Ajustar `init.rs` do shell pra chamar a tool real do crate novo.

**Impacto:** encolhe o core, tira deps de raster de quem só pinta UI, e torna a "pasta isolada" do Implementador **verdadeira**. Para *ripple*, tools > widgets (tools carregam algoritmo pesado; widget split é morto pelo fan-out).

**Sinais para decidir caso a caso (deletar duplicação vs completar extração):**

| Sinal no código | Diagnóstico | Ação |
|-----------------|-------------|------|
| Crate periférico só tem manifesto/`shadow_handler` e delega via re-export do shim | **Fachada** (código real no core) | **Completar extração:** mover a pasta do core pro crate; remover shim |
| Ambos (crate e subpasta do core) têm o algoritmo idêntico | **Duplicação acidental** (branches paralelas) | **Deletar duplicação:** preservar o crate periférico; limpar a subpasta do core + remover dep `image` do core |
| Core importa structs internas da tool (ex. `BgRemovalScratch`) pra rodar preview no loop principal | **Acoplamento de pipeline** | **Decouple via intent-bus:** estado temporário de render vai pro shell; expor só contratos |

> Estado atual conhecido (§2.2): **bgremoval = fachada** → completar extração.

---

## 4. Crate de contrato (diferido, só se a medição mandar)

Se o ripple ainda doer **depois** de (3) encolher o core e (1)/(2) destravarem o fluxo, então — e só então — quebrar `editor-core` na costura de contrato:

```
ph2d-editor-contract   (paint trait, tokens-glue, a11y, interaction core)  ← estável
        ▲         ▲             ▲
     widgets   editor-core    painéis      ← todos dependem PARA BAIXO do contrato
        ▲_________ ▲ ____________▲
              ph2d-app-registry (folha, codegen'd)  ← só o binário depende
```

**Custo real (por isso é diferido):** promover `pub(crate)` → `pub` (vaza internos), risco de ciclos `screens↔widget` (exige traits/genéricos extras), semanas de trabalho serializado. Bundlar widgets em ~6-8 crates por categoria (espelhando o split que `showcase/` já tem), **não** 35×1.

---

## 5. Plano convergido (ordem custo-crescente)

1. **Agora (operacional, horas):** `CARGO_TARGET_DIR` por slot. Checar disco/IO do drive antes. **Não** ativar sccache no escuro — medir o trade-off de incremental separadamente.
2. **Medir os 15min:** decompor em `cargo check -p editor-core` (frontend) vs build dos 8 downstream vs runtime dos arch-gates. Se gate domina → escopar gate por arquivo mudado, **não** rearquitetar.
3. **Completar a extração física de tools** (§3, Gargalo 3). Top prioridade arquitetural — é dívida, é mentira de isolamento, e encolhe o core.
4. **Codegen widget/mod.rs + showcase + dispatch do chrome** (§3, Gargalo 2). Barato; mata a maior fonte de colisão (churn 101). Fazer **independente** de (3).
5. **Crate de contrato / inversão do grafo** (§4): **só** se (2) provar que o ripple ainda dói depois de (3).

> **Bônus irônico:** (3) e (4) são tão ortogonais que dois agentes poderiam fazê-los **em paralelo** — um teste real do isolamento por slot de (1).

---

## 6. Não-objetivos / explicitamente descartado

- **Carregamento dinâmico (WASM/dylib) para UI first-party** — sem ABI Rust estável; UI cruzando boundary mata o paint dispatch baseado em trait/genéricos; viola zero-alloc; AccessKit cross-boundary é sofrimento. A lane de extensibilidade sandboxed é o Luau (§2.3).
- **linkme/inventory para o registry** — ordem dependente de link-order, atrito com determinismo. Codegen vence (§3, Gargalo 2).
- **Abstração para PanelHost** — 3 touches/200 commits. Serializar no Coordenador é aceitável; não construir máquina pra não-problema.
- **35 crates 1-por-widget** — over-engineering; o codegen já mata a colisão. Bundle por categoria se e quando o ripple mandar.

---

## 7. Questões abertas (resolver com dado, não debate)

- **Q1:** os 15min são frontend, build-downstream, ou runtime-dos-gates? (passo 2) — decide se (5) é necessário.
- **Q2:** `padding`/`trim_transparency`/`make_square`/`real_size` são fachada como bgremoval, ou há duplicação? (inspeção por §3 tabela de sinais)
- **Q3:** `MAC_EXTERNO` aguenta N target-dirs (espaço + throughput)? Senão, target-dir por slot vai pra SSD local.
- **Q4:** o `dispatch_all` do chrome é puramente aditivo (codegen-friendly) ou tem lógica que resiste a geração? (inspeção antes do passo 4)

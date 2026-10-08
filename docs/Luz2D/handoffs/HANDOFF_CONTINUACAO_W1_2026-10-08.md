# Handoff de continuação — Luz 2D, W1 → W2 (2026-10-08)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · base `2a925859d` (main de 07/10, sem rebase pendente).
- **Commits da W1:** `456710bd0` (as duas crates + gates) · `a6039b1e9` (registo, catálogo, rótulos) · este doc.
- **Estado:** W1 FECHADA (onda, não a linha). Nada para o Enio ver ainda: sem GPU, sem UI, sem smoke.

## O que existe

| Crate | Conteúdo |
|---|---|
| `ph2d-light2d-ecs` | `Luz` (register, sem `Default`), `BloqueiaLuz` e `NaoRecebeLuz` (register_default); porta `register_light2d_components`; `coleta::cena_de_luz(&World) -> CenaDeLuz` |
| `ph2d-light2d-ref` | `fluencia(&CenaDeLuz, [f64;2]) -> [f64;3]` e `queda::queda(t, alcance)` |

A semântica fixada (intensidade = fluência a 1 m; opacidade por ENTRADA; emissores opacos; céu
do `Ambiente`; quad do sprite) está no [plano §2.3](../01_plano.md). **A W2 tem de responder à mesma
pergunta** — é isso que o G3 mede.

## O método do gabarito (decidido por G1/G2)

Varredura angular EXATA: eventos = vértices, tangentes aos discos, cruzamentos de fronteiras que
envolvem um emissor, cruzamentos com o círculo do alcance; um raio no meio de cada intervalo.
Com opacidade por entrada o integrando é constante por intervalo **também para α parcial**, logo
não há quadratura nem Monte Carlo. A quadratura (Gauss–Kronrod 7/15) só existe para alcance finito.
⚠️ A tolerância dela é absoluta e FIXA: proporcional ao pedaço, a singularidade `√` da tangente a
um disco fazia a árvore explodir (medido: 31 s num ponto; G10 nunca terminava).

## Números medidos (perfil `dev`, opt-level 1)

| Gate | Medido | Tolerância |
|---|---|---|
| G1 disco no vazio, 30 pontos `a ∈ [1e-3, 7,5]`, `r/a ∈ [1,0001, 1e3]` | 7,1e-16 rad = **0,80 ulp de 2π** | 4 ulp de 2π + 1e-14 rel |
| G2 contra o oráculo geométrico (α 1 e 0,5; 4 raios; 201 pontos) | **6,5e-14** de `2δL` | 1e-12 |
| G2 largura da penumbra por bissecção × `2D·a/√(d²−a²)` | **5,7e-13 m**; larguras 0,069 → 0,137 → 0,275 → 0,554 m | 1e-11 m |
| G10 768 valores, duas threads | bit-idênticos | — |
| G12 1 000 pontos × fórmula de Karis | **2,2e-16** | 4 ε |
| Tempo por ponto, cena cheia (4 emissores, 2 oclusores, alcance finito) | **80,6 µs** (mínimo de 7, `load ~20`) | medida `#[ignore]` |

⚠️ O erro do G1 é ABSOLUTO em ângulo: o piso de uma varredura em `f64` é ~1 ulp de 2π por evento.
Relativo, isso dá 1,7e-13 num disco com `δ = 1e-3`. A tolerância relativa original reprovou por isso.

**Mutação (`mutacao`):** 6 de 7 sangraram (tangente `asin→atan`, `0,999·δ`, transmissão ignorada,
sem vértices do polígono, `x²·x` na queda, raio do disco 1,01). Sobreviveu só o CONTROLO: avaliar
num ponto do intervalo que não é o meio dá o mesmo resultado, o que prova que o conjunto de eventos
está completo.

## Premissas do briefing que a medição derrubou

- «Suba os contadores do registo ECS e dos dois espelhos»: o `registry_tests` do ECS (105) e os
  espelhos render/script (106) contam SÓ os registadores deles e **não sobem**. Os que sobem são as
  duas listas da shell, o catálogo (categoria nova `Light`, `ALL` 16) e os rótulos.
- «A cor do sprite emissivo é a do próprio sprite»: os texels não estão no `World` (só o
  `AssetId` em `SpritePixels`). A coleta usa `tint∘self_tint`·α·k sobre o quad. Os texels são da C0.

## Para a integração (somam entre linhas)

Foundational tocado: `ph2d-component-desc` (variante `ComponentCategory::Light` no fim do enum,
`ALL` 15→16, `label_key`), `ph2d-app-components/component_palette.rs` (+1 braço), `ph2d-i18n`
(4 chaves), `shells/desktop` (+2 linhas no `init.rs`, +1 no censo, +1 dep). Shell: 185 537 →
185 540 linhas, contando `tests/` (tecto 189 041). `PROJECT_SCHEMA` intocado (187). Nenhum contrato congelado tocado.

## O que a W2 precisa saber (o solver na GPU)

1. A C0 lê a `CenaDeLuz` e mais nada. O `Ponto` (`tamanho_m = 0`) não cabe num texel: é o caminho
   analítico (C3), e o gabarito já o resolve exato (`I·queda(r)/r` com visibilidade).
2. Opacidade por entrada no RC = detetar a transição fora→dentro a cada passo, **também na
   fronteira de cada intervalo de cascata** (amostra-se o texel anterior ao início do intervalo).
3. A gêmea WGSL de `queda` mede-se contra a MESMA tabela do G12 (`tests/it/g12_queda.rs`).
4. Gaps nomeados, por fechar: a forma que a coleta vê é o QUAD (sem alfa por texel nem pele do
   esqueleto); objetos vetoriais/Flip/Motion só bloqueiam pela captura de tile da C0, porque o
   `World` só tem `VecPathRef`. O G3 deve usar cenas de discos e polígonos, onde o gabarito é exato.
5. Comandos: `bash scripts/cargo-test-narrow.sh ph2d-light2d-ref` (0,04 s);
   medida: `cargo test -p ph2d-light2d-ref --test it -- --ignored --nocapture medida_tempo`.

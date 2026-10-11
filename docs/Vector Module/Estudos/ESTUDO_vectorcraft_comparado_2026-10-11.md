---
name: estudo-vectorcraft-comparado
description: VectorCraft (storytold, MIT/Apache, Rust) contra o motor ph2d-vec-* — leitura dos dois códigos, corrida do oráculo por CLI e a prova lado a lado do offset na estrela do gate. Achado principal: linesweeper 0.5 zera os pânicos do nosso offset.
data: 2026-10-11
status: ESTUDO (nada mudou no código do PH2D)
---

# VectorCraft × motor vetorial do PH2D — 2026-10-11

Código: `~/Documentos/Recursos/Craft/vectorcraft` (commit `fe8af48f`), app 0.9.0 em `~/Apps/vectorcraft`
([memória](../../../project-memory/reference_craft_suite_installed.md)). Licença **MIT OU Apache-2.0** ⇒
porta aberta (ler e portar com atribuição). Scripts desta medição em [`ferramentas/vectorcraft_2026-10-11/`](../ferramentas/vectorcraft_2026-10-11/) (versionados pela linha `line/Vector`, regra J4).

## 1. Veredito

São produtos de natureza diferente, e cada um ganha no seu terreno.

| Eixo | VectorCraft | PH2D (`ph2d-vec-*`) |
|---|---|---|
| Natureza | clone do Illustrator: ilustração estática, impressão | vetor para animação/jogo: timeline, ossos, nodes |
| Tamanho | ~377 k LOC em 12 dias (1 974 commits, 1 074 co-autor Claude) | ~106 k LOC (`ph2d-vec-*`) + 67 k `ph2d-app-vec` |
| Ferramentas | 92 no catálogo (`crates/tools/src/catalog.rs:20-137`), 1 stub (Touch Type); 27/375 itens de menu stub | 16 modos, 47 formas, 10 efeitos de caminho |
| Formatos | SVG (usvg + writer próprio), PDF r/w (krilla/hayro), AI (lê), EPS (interp. PostScript próprio), DXF, EMF, Affinity (lê) | SVG import (usvg) / export; nada de PDF/AI/EPS |
| Render | **CPU** (`vello_cpu`), GPU só compõe; sem dirty regions; `bench` 10k paths = 254 ms/quadro a 4 threads (load ~64) | **GPU** (Vello) |
| Animação | nenhuma | timeline, morph, ossos/pele, opacidade animável |
| Texto | shaping com `harfrust` (kerning) | sem kerning (doc 42 §2.2) |
| Gradiente | linear, radial, freeform, **malha** (`render/src/live.rs:77`) | linear, radial (malha recusada: [13 §13.9](../13_fora_de_escopo.md)) |
| Undo | snapshot de `Arc<Document>` (O(1) dirty por ponteiro), limite 500 | diff do `ProjectState` |
| Testes | ~5 320 `#[test]`, 18 proptest, model-based; **sem CI Linux/macOS** nos PRs (só FreeBSD em push) | ~3 036 `#[test]`, mutação, paridade CPU↔GPU, censos |

**Para o PH2D, o nosso é o certo** (anima, GPU, integrado ao ECS). Como editor de ilustração estática o
VectorCraft é muito mais largo — mas largura escrita em 12 dias, sem CI que corra os testes em Linux.

## 2. Oráculo por CLI (`vectorcraft-cli`)

Subcomandos: `run --in F --cmd ID --params JSON --export F` (lote headless; JSON por passo) · `commands`
(732 comandos) · `convert` (25 leituras / 19 escritas) · `info` · `bench` · `perf` · `mcp`.

Corpus de 11 SVG próprios contra resvg 0.48.1 (load ~64, tempos inflados):
- Render no ruído rsvg×resvg (≤ 0,77 %), exceto gradiente com `stop-opacity`: 9,6 % — o VC interpola em
  alfa pré-multiplicado (como os browsers); escolha de modelo, não bug.
- Booleanas (`object.pathfinder.{unite,minusFront,intersect,exclude}`): 0–0,4 % contra a máscara
  booleana, inclusive tangentes, coincidentes, aresta partilhada; resultado vazio ⇒ exit 1 com mensagem.
- Robustez: zero pânico em malformados/1e30/1e308 (o resvg pânica em 1e308), mas **engole defeitos sem
  aviso** (path `NaN`, `use` cíclico, 1e308 some com `warnings: []`); coordenadas internas `f32`.
- Escala patológica: 1 path de 200 k segmentos ⇒ união/offset > 180 s e 5,1 GB, sem cancelamento.

⚠️ O VC importa com o mesmo usvg 0.48 do resvg — o resvg não é juiz independente da IMPORTAÇÃO.

## 3. A prova lado a lado — offset da estrela do gate

Fixture: a estrela de [`the_offset_never_takes_the_app_down.rs`](../../../crates/ph2d-vec-boolean/tests/it/the_offset_never_takes_the_app_down.rs)
(`cook(Star, [-3.6,-1.4], [-0.8,1.4], &[5, 0.45, 0])`, ×100 ⇒ R = 140, r = 63), 100 distâncias
`d = 4..400` por junção, para fora e para dentro. Verdade para fora = `fill ∪ stroke(2d, mesma junção,
miterlimit 4)` rasterizada no resvg, diferença em % de pixels. Nosso lado = `offset_path(.., OffsetSide::Outer)`
numa sonda fora do workspace (`sonda_offset_*`), em release, com target dir próprio.

| Junção, lado | VectorCraft | PH2D, linesweeper **0.4** (hoje) | PH2D, linesweeper **0.5** |
|---|---|---|---|
| Miter, fora | 100/100, ≤ 0,007 % | 100/100, ≤ 0,015 % | 100/100, ≤ 0,015 % |
| Round, fora | 100/100, ≤ 0,085 % | **87/100** (vazio em 76, 124–136, 160–200, 240, 244) | **100/100**, ≤ 0,068 % |
| Bevel, fora | 100/100 devolvidos, mas **≥ 172 a forma sai estilhaçada** (4–29 %) | 35/100 | 40/100 (vazio de 164 em diante) |
| Miter, dentro | some depois de 52 (certo: miter some no raio das RETAS, 53,76) | igual | igual |
| Round, dentro | some depois de 60 (certo: o disco inscrito tem raio 63, o vértice côncavo) | igual | igual |
| Bevel, dentro | some depois de 60 | **vazio em 44–52** (some cedo) | **vazio em 44–52** |
| Pânicos capturados pelo `catch_unwind` | — | **sim**, em `linesweeper-0.4.0/src/curve/mod.rs:364` | **0** |

Leituras:
1. **`linesweeper 0.5` compila sem mudar uma linha de `ph2d-vec-boolean`** e zera os pânicos da fixture;
   Round para fora sai de 87 → 100/100. O VC usa 0.5 (`Cargo.toml` do workspace dele). Não corri a suíte
   do PH2D sob 0.5 — a subida de 0.3→0.4 mudou a representação da saída (ver o comentário em
   `crates/ph2d-vec-boolean/Cargo.toml`); a linha que subir paga o gate inteiro.
2. **Bevel para fora com `d` ≥ ~164 (> raio da estrela) falha nos DOIS:** nós devolvemos vazio (a forma
   some sob o slider), o VC devolve triângulos soltos. Nenhum dos dois é oráculo aqui.
3. **Bevel para dentro em 44–52 é defeito só nosso** — o VC responde.

## 4. O que aprender (ordenado por valor/custo)

1. **Subir `linesweeper` 0.4 → 0.5** (medido acima). Depois dele, o `catch_unwind` de `expand.rs:458`
   e a tabela de `expand.rs:436-440` precisam de recontagem.
2. **Pós-processamento das booleanas do VC** (`crates/pathops/src/boolean.rs`): `snap_horizontals` (:160,
   contorna o mau ordenamento do sweep com arestas quase horizontais), `is_sliver` por 2·área/perímetro
   (:325), epsilon proporcional à magnitude das coordenadas (:123-136), `Tidy` refit dos pedaços (:233).
   Candidatos diretos às falhas de Bevel do §3.
3. **Mapa planar cisalhado do Live Paint** (`crates/pathops/src/planar.rs:14-30`): conta cruzamentos e não
   winding (path aberto não cancela) e varre num referencial cisalhado por potência de 2 (exato em float).
   Comparar com o nosso Build/Shape Builder (BUGS #12, #28).
4. **Kerning por `harfrust`** (`crates/text`), a lacuna do doc 42 §2.2.
5. **O VC como oráculo das nossas booleanas/offset** (§0.9): `vectorcraft-cli run` sobre entradas nossas,
   fixture com cabeçalho, gate. Ressalva: onde ele erra (§3, Bevel ≥ 172) a verdade é a analítica.
6. **Uma lista única de comandos** (`engine/src/cmd/mod.rs`, 714 `cmd!`) servida a menu, CLI e MCP — é o
   que torna o app dele testável por script.

⛔ Não copiar: hit-test linear sem índice espacial, render de tela inteira sem dirty regions, defeito de
entrada engolido sem aviso, coordenadas `f32`, booleanas sem prazo nem cancelamento.

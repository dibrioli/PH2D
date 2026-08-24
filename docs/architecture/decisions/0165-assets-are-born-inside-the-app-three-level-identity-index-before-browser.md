# ADR-0165 — O asset nasce DENTRO do app: identidade em 3 níveis, o ÍNDICE antes do navegador, catálogos por UUID — e o mestre É um asset

- **Status:** Accepted (aprovado pelo Enio em 2026-08-24, junto com o ADR-0164)
- **Data:** 2026-08-24
- **Linha:** `line/components` (fases **F6–F7** do [plano vivo](../../Components/05_plano_de_implementacao.md))
- **Depende de:** [ADR-0164](0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) — os mestres (F4) são o primeiro conteúdo do navegador
- **Toca:** crate nova `ph2d-asset-index` · `ph2d-asset` (variantes/metadado) · `shells/desktop` (painel + drag) · painéis consumidores (inspector/timeline/grafo) via `AssetRef<Kind>` como `ParamRow`
- **Não move:** HR-6 (`AssetId = blake3(bytes)` continua a identidade de CONTEÚDO) · o `ProjectFile` (a quebra em um-ficheiro-por-asset é decisão de DESTINO, wave própria, fora deste ADR)

## Contexto

O PH2D não tem navegador de assets — **zero código**, só o mockup de 2026-05
(`docs/design/screens/05-asset-browser.html`); o chip "Assets" da topbar imprime o próprio nome no
stdout. E a necessidade é maior que a da Godot, porque aqui o utilizador **produz** os assets dentro
do app: hoje o rack de áudio e os presets de pincel **não são salvos em lugar nenhum**, e cada
módulo pendura um campo próprio no `ProjectFile` (é o que levou o `PROJECT_SCHEMA` a 84 degraus).

A pesquisa ([doc 03](../../Components/03_pesquisa_sistema_de_assets.md)) comparou Godot (`uid://`,
4.4), Unreal (Asset Registry/redirectors/Reference Viewer), Unity (`.meta`/GUID), Blender (Asset
Browser, catálogos), USD (`ArResolver`) e Substance (`.sbs`×`.sbsar`), e destilou seis leis — as
três decisivas: **o índice vem antes do navegador** (o Content Browser só existe porque o Asset
Registry responde **sem carregar** nenhum asset); **identidade opaca funciona e tem de ser
versionada ao lado do conteúdo** (Godot e Unity pagaram o mesmo bug quando as duas se separam);
**a taxonomia que o artista vê não precisa de ser a árvore de ficheiros** (catálogos por UUID num
ficheiro de texto — Blender).

## Decisão

### 1. Identidade em TRÊS níveis com três nomes — o padrão do `Ar` 2.0, que o repo já pratica em UM tipo

| Nível | Tipo | Responde | Estado |
|---|---|---|---|
| **Conteúdo** | `AssetId = blake3(bytes)` | *"estes bytes exatos"* | ✅ HR-6, intocado |
| **Identidade lógica** | `LogicalId` opaco, versionado junto do conteúdo | *"esta COISA, através de versões e caminhos"* | generaliza o `LogicalTextureId` (hoje só textura) |
| **Caminho** | `PathBuf` | índice humano | continua metadado, nunca identidade |

Misturar os três era a raiz de metade dos bugs do `Ar` 1.0 (*"everything was just a string"*).
⚠️ A regra que Godot/Unity pagaram: **`LogicalId` vai para o VCS ao lado do conteúdo** — um
`.gitignore` que o separe quebra o projeto no clone seguinte, em silêncio.

### 2. O ÍNDICE é o subsistema; o painel é um cliente fino — e nem é o primeiro

`ph2d-asset-index` (crate-folha, sem UI, sem GPU): `query(AssetQuery)` (kind · catálogo · tag ·
nome · *usado-em*) · `deps(LogicalId) → {uses, used_by}` · `preview(LogicalId)` **assíncrono com
cache limitado** (o preview viaja com o asset, nunca em pasta paralela). O índice lê **cabeçalho,
nunca o corpo** — a lição do Asset Registry, e a cura da patologia medida no Godot (reimport
disparado por movimentação = a fonte nº 1 de travamento). O grafo de dependências não é ferramenta
de auditoria: é o **pré-requisito** de exportar, migrar e apagar com segurança (*portabilidade =
fecho transitivo das dependências*).

### 3. Catálogos por UUID, desacoplados da pasta — e "marcar", nunca "exportar"

Taxonomia = catálogos `{uuid, caminho-de-categoria, nome}` num **ficheiro de texto** (modelo
Blender): reorganizar a vista não toca nos ficheiros, renomear categoria não desliga assets.
**"Marcar como asset"** anuncia o dado onde ele vive — num app onde o asset nasce dentro de um
documento, *exportar é uma cópia que envelhece; marcar não é*. Vivo × congelado é **verbo**
(*Instanciar* × *Duplicar*/*Destacar* — ADR-0164), nunca checkbox; o padrão de todas as referências
é guardar o grafo e tratar o bake como cache.

### 4. O mestre É um asset — a unificação que dá conteúdo ao navegador no dia 1

*Criar componente* (ADR-0164) põe o mestre na biblioteca com `LogicalId`, preview e catálogo;
**arrastar do navegador para o canvas = Instanciar**. Um payload de arrasto único
(`DragAsset(AssetRef)`) serve os quatro consumidores — canvas, timeline, grafo, inspector — e
`AssetRef<Kind>` entra como **variante do `ParamRow`** (senão "escolher textura" volta a ser widget
artesanal). Depois dos mestres, entram pelo mesmo funil os órfãos de hoje: presets de pincel,
paletas, presets do rack de áudio, malhas.

### 5. Escopos NÃO se fundem

v1 = **documento** e **projeto**, mostrados lado a lado (nenhuma referência funde escopos e resolve
precedência — todas evitam o problema). Biblioteca do utilizador/instalada: depois, mesmo desenho.

## Consequências

- ⭐ O produto do artista (pincel, mestre, paleta, malha) ganha **endereço, preview e busca** —
  hoje parte dele não é sequer salvo.
- ⭐ Buscar por *"usado em"* e apagar com relatório de dependentes tornam-se possíveis (nenhuma
  coleta automática de órfãos — *"não referenciado" ≠ "não desejado"*; relatório, não colheita).
- **Preço:** um índice a manter coerente com o documento (dirigido pelos mesmos change ticks do
  ADR-0164); cor dominante entra barata (histograma OKLab numa passagem), **busca por similaridade
  (ML) fica fora** — mesma fronteira dura que o áudio já declarou.
- **Adiado com degrau nomeado:** um-ficheiro-por-asset (Git-friendly; hoje um traço de pincel
  reescreve o `ProjectFile` inteiro) é o **destino** declarado — o índice nasce compatível, a quebra
  do monólito é wave própria com ADR próprio.

## Alternativas medidas e recusadas

| alternativa | por que não |
|---|---|
| **Começar pelo painel** | o navegador que carrega assets para listar trava — a lição nº 1 das 4 referências; sem índice, cada filtro é um scan |
| **Caminho como identidade + redirectors** (Unreal) | funciona e acumula lápides que equipas limpam por rotina; o Godot acabou de fugir disto (`uid://`, 4.4) |
| **GUID sidecar por ficheiro** (Unity `.meta`) | a classe de bug mais documentada do modelo (meta perdido/conflitado = referências mortas em silêncio); e o PH2D nem tem um ficheiro por asset ainda |
| **Pastas como taxonomia** | mover vira evento de sistema (reimport/quebra); catálogo por UUID custa um ficheiro de texto |
| **Base de dados central de assets** | a biblioteca-é-o-filesystem do Blender prova que não precisa; um BD é um segundo formato a versionar |
| **Busca visual por similaridade na v1** | exige runtime de ML no editor — fronteira que o áudio já pagou para traçar (`audio-ml` OFF) |

## Referências

- **Pesquisa longa:** [doc 03 — o sistema de assets](../../Components/03_pesquisa_sistema_de_assets.md) (§1.8 as seis leis · §2 o eixo sem prior art) · decisão [doc 04 v2 §2.8, C8–C10](../../Components/04_decisao_arquitetura.md)
- **Plano:** [`05_plano_de_implementacao.md`](../../Components/05_plano_de_implementacao.md) fases F6–F7
- Fontes decisivas: Godot [UID changes 4.4](https://godotengine.org/article/uid-changes-coming-to-godot-4-4/) · Unreal [Asset Registry](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-registry-in-unreal-engine) · Blender [Asset Browser](https://docs.blender.org/manual/en/latest/editors/asset_browser.html) (GPL — comportamento apenas) · USD [Ar 2.0](https://openusd.org/dev/wp_ar2.html) · Adobe [SBSAR](https://substance3d.adobe.com/documentation/sddoc/publishing-substance-3d-asset-files-sbsar-200574380.html)
- HR-6 (blake3) · [ADR-0055](0055-cooked-texture-compression-pipeline.md) (`LogicalTextureId`, o precedente dos 3 níveis)

# Pesquisa — o sistema de assets: navegador, identidade, e o caso peculiar do PH2D (Fases B4–B5)

> **Pesquisa externa: 2026-08-21.** Mesma convenção do doc irmão: **[DOC]** doc oficial ·
> **[COM]** comunidade/artigo · **[INF]** inferência minha, sempre marcada.
> ⚠️ `docs.blender.org` **recusa fetch automatizado (403)**; as afirmações sobre o Asset Browser do
> Blender vêm de **resumo de busca sobre a página oficial**, marcadas **[DOC-indireto]**.
>
> **Irmãos:** o estado do PH2D é [`01_auditoria_modelo_de_objeto.md §5`](01_auditoria_modelo_de_objeto.md);
> composição e prefab são [`02_pesquisa_composicao_e_prefab.md`](02_pesquisa_composicao_e_prefab.md).
> A decisão é a [Fase D](04_decisao_arquitetura.md) — **este doc não escolhe nada.**

---

## 📍 Índice

| § | assunto |
|---:|---|
| **§0** | Higiene de licença |
| **§1** | **B4** — os cinco sistemas de asset comparados |
| **§1.8** | ⭐ As seis leis que a comparação destila |
| **§2** | **B5** — assets **gerados dentro da ferramenta**: o eixo sem prior art perfeito |
| **§2.1** | ⭐ O asset é o grafo, o bake, ou os dois? — Substance responde, e confessa o preço |
| **§3** | O que isto põe na mesa para o PH2D |
| **§4** | Fontes |

---

## §0 — Higiene de licença

| Fonte | Licença | O que li | Uso permitido |
|---|---|---|---|
| Godot | MIT | docs + issues/PRs oficiais | comportamento e código |
| Unreal, Unity, Substance/Adobe, Quixel | proprietárias | **só documentação pública** | comportamento, terminologia, modelo |
| Blender | **GPL** | ⚠️ **só manual e notas de release, e por resumo de busca** | comportamento apenas · ⛔ nenhum código lido |
| OpenUSD (`Ar`) | Apache-2.0 | docs + guia comunitário | tudo |
| Git / IPFS / DVC / git-annex | GPL/MIT/Apache (variados) | docs e artigos | ⚠️ **modelo conceitual apenas** — content-addressing é uma ideia, não código a copiar |

---

## §1 — B4: os cinco sistemas de asset

### §1.1 — A tabela de uma olhada

| | Identidade | Onde vive o metadado | Sobrevive a mover **fora** do editor | Grafo de dependências | Vem com navegador? |
|---|---|---|---|---|---|
| **Godot 4.4+** | `uid://` (id opaco) | **embutido** em `.tscn`/`.tres`/`.import`; **sidecar `.uid`** para script/shader | ✅ **é a razão de existir do sistema** | ✅ `[deps]` no `.import` + *Dependency editor* | ✅ dock FileSystem |
| **Unreal** | caminho de pacote + **redirector** ao mover | header do `.uasset` + **Asset Registry** em memória | ⚠️ não — mover fora do editor quebra; dentro, deixa redirector | ✅✅ **Reference Viewer + Size Map + Asset Audit** | ✅✅ Content Browser (o mais maduro) |
| **Unity** | **GUID** em `.meta` sidecar | `.meta` ao lado de cada ficheiro | ⚠️ só se o `.meta` for junto | ⚠️ existe, mas sem visualizador de 1ª classe | ✅ Project window |
| **Blender** | data-block dentro de um `.blend` | **catálogos por UUID** num `blender_assets.cats.txt` + metadados no próprio `.blend` | n/a (é ficheiro-biblioteca, não árvore de assets) | ⚠️ via link/override | ✅ Asset Browser |
| **OpenUSD** | **identificador de asset** resolvido por `ArResolver` | fora do formato — é o resolvedor que decide | ✅ **por construção** (o path nem precisa ser um path) | ✅ composição é o grafo | ❌ (é formato, não ferramenta) |
| **PH2D hoje** | `AssetId = blake3(bytes)` | `AssetDb` em memória, **nada em disco** | n/a | ❌ | ❌ |

### §1.2 — Godot: a engine que trocou de identidade em público **[DOC]**

**O problema que resolveram, na voz deles:** tornar o Godot *"better suited for large projects and
significantly more resilient to external filesystem changes"*, para que se possa reorganizar ficheiros
**fora** do editor.

**O mecanismo:**
- `uid://c82j4l3r4k4n2` é a referência; o `res://` é índice humano.
- Cenas (`.tscn`), recursos (`.tres`) e importados (via `.import`) **já embutiam** o UID.
- ⭐ 4.4 acrescentou **sidecar `.uid`** para os formatos de **texto puro sem header**: scripts (`.gd.uid`)
  e shaders. *"This avoids filename conflicts when a script and shader share the same base name."*

⚠️ **O footgun documentado, e é o mesmo do Unity:** *"as soon as you clone the project on another
device, the UID references will break"* se os `.uid` forem para o `.gitignore`. **A identidade tem de
ser versionada junto com o conteúdo.**
⚠️ **E há um caso que o UID não salva [DOC]:** *"two resources to be swapped around without either of
their paths changing"* — o fallback por caminho é ambíguo quando dois ficheiros trocam de lugar.

**O que o `.import` guarda [DOC/COM]:** é texto legível, com uma secção **`[deps]`** listando o ficheiro
fonte e os caminhos dos artefatos gerados. ⚠️ **Patologia medida e discutida no repo oficial:** *"every
time a file which requires import is moved in the filesystem, a reimport gets triggered, which can cause
potentially huge stalls"* ([proposals#8360](https://github.com/godotengine/godot-proposals/issues/8360));
mover uma dependência com a cena aberta chegou a **corromper a cena e a crashar a engine** (corrigido em
[#81657](https://github.com/godotengine/godot/pull/81657) e
[#84520](https://github.com/godotengine/godot/pull/84520)).

**[INF] A leitura:** o Godot é o experimento controlado mais útil da lista, porque **fez a migração em
público e documentou a dor**. Duas conclusões duras: (a) identidade opaca **funciona**, e (b) ela é
inútil se não estiver no VCS ao lado do conteúdo.

### §1.3 — Unreal: o mais maduro, e o que a maturidade custou **[DOC]**

**Quatro peças que trabalham juntas:**

1. **Asset Registry** — base de dados **em memória** com `FAssetData` por asset: caminho, classe, e
   **tags de registro** (propriedades marcadas `AssetRegistrySearchable` + `GetAssetRegistryTags()`).
   Construída lendo **os headers dos `.uasset` de forma assíncrona** enquanto o editor abre.
   ⭐ **É isto que permite o Content Browser listar e filtrar milhares de assets sem carregar nenhum.**
   ⚠️ Limitação declarada: *"This requires assets to be resaved before their properties will be
   discovered by the Asset Registry"* — e o registro *"may not have a complete list of all assets"*
   enquanto a varredura corre.
2. **Redirectors** — mover/renomear **dentro do editor** deixa um *redirector* no sítio antigo, para que
   pacotes não carregados ainda encontrem o asset. Limpa-se com *Fix Up Redirectors in Folder* ou o
   commandlet `ResavePackages -FixupRedirectors`.
   ⚠️ **[INF] Isto é o oposto exato da escolha do Godot:** o Unreal mantém o **caminho** como identidade e
   compensa com uma lápide; o Godot troca a identidade e dispensa a lápide. O preço do Unreal é um tipo
   de dívida que se acumula e que equipas precisam de limpar por rotina — há literatura inteira sobre
   *"unresolved redirectors"* **[COM]**.
3. **Reference Viewer / Size Map / Asset Audit** — grafo visual de *quem referencia* e *quem é
   referenciado*, com **profundidade configurável**, filtro por tipo, restrição a uma *Collection*, e
   *Show Reference Tree*. Serve para *"identify and debug high dependencies, slow load times, and high
   memory usage"*.
4. **Asset Manager** — divide tudo em **Primary** e **Secondary**: primários são endereçáveis por
   `FPrimaryAssetId` e governados por **regras** (`FPrimaryAssetRules`) que decidem cook e **chunking**;
   secundários *"are loaded automatically… in response to being referenced"*. Por default **só `UWorld`
   (níveis) é primário**.

⭐ **[INF] A ideia mais transferível do Unreal não é o Content Browser — é o Asset Registry.** O
navegador é a consequência: *só se consegue navegar 10 000 assets se houver um índice que responde sem
abrir nenhum deles*. Qualquer plano que comece pela UI e deixe o índice para depois constrói um
navegador que trava.

### §1.4 — Unity: GUID em sidecar, e as patologias que todo mundo cita **[DOC/COM]**

- Identidade = **GUID gerado aleatoriamente**, guardado num `.meta` ao lado do ficheiro. O `.meta`
  guarda também as **import settings**.
- **As três patologias, documentadas pela prática [COM]:**
  1. **Conflito de `.meta` em VCS** — dois utilizadores criam o mesmo asset em ramos diferentes → dois
     GUIDs → *"silently breaks references across the project"*. A mitigação oficial é `Force Text` +
     `Visible Meta Files` + `*.meta merge=unityyamlmerge` no `.gitattributes`.
  2. **`.meta` perdido ou corrompido** → Unity **regenera** o GUID → todas as referências apontam para o
     nada, **sem erro alto**.
  3. **Reimport lento** — o custo de mexer na árvore.
- **Addressables** existe como camada por cima para endereçamento lógico e carregamento sob demanda.

⚠️ **[INF] O contraste com o Godot é instrutivo:** os dois usam identidade opaca sidecar e sofrem
**exatamente a mesma classe de bug** — a identidade pode divergir do conteúdo. A diferença é que o Godot
**derivou** o problema e documentou o remédio *(commite o `.uid`)* de saída, e a Unity o descobriu em
produção ao longo de uma década.

### §1.5 — ⭐ Blender Asset Browser: o parente mais próximo do PH2D **[DOC-indireto]**

É o único da lista onde o utilizador **produz o asset dentro da própria ferramenta** — a mesma situação
do PH2D. As escolhas:

- **"Mark as Asset"** sobre um data-block (material, objeto, pose, node group…) — o asset **não muda de
  ficheiro**; ele passa a ser **anunciado**.
- **Asset library = um diretório de ficheiros `.blend`.** Não há base de dados central: a biblioteca é o
  sistema de ficheiros.
- ⭐ **Catálogos desacoplados da pasta.** Um ficheiro de texto `blender_assets.cats.txt` na pasta define
  os catálogos; **cada entrada é `UUID + caminho-de-categoria + nome de exibição`**. ⚠️ **Portanto a
  árvore que o artista vê NÃO é a árvore de pastas** — e o UUID é o que permite renomear/mover a
  categoria sem desligar os assets.
- **Metadados:** nome, preview, descrição, **tags**. ⚠️ **Editáveis apenas na biblioteca *Current File***
  — *"the only asset library that allows editing of asset metadata"*.
- **Preview:** gerado automaticamente ao marcar; substituível arrastando uma imagem.
- **Drag-and-drop** para o viewport instancia o asset (append ou link, conforme a configuração).

⭐ **[INF] As duas decisões que valem para o PH2D:**
1. **Catálogo por UUID separado da pasta** resolve *"quero reorganizar sem quebrar"* **sem** precisar de
   identidade por conteúdo em cada asset — a indireção está na taxonomia, não no dado.
2. **"Marcar" em vez de "exportar"** é a diferença entre um asset ser *um lugar* e ser *um estado*. Num
   app onde o utilizador cria o conteúdo, **exportar é uma cópia que envelhece**; marcar não é.

### §1.6 — Resolução por IDENTIDADE, não por caminho: `ArResolver` **[DOC]**

O USD leva a indireção ao limite: um *asset path* é uma **string opaca** que um `ArResolver`
plugável traduz. O `Ar` 2.0 introduziu o tipo explícito `ArResolvedPath` porque, na 1.0, *"everything
was just a string that could be either an Asset Path, an Asset Identifier or a Resolved Path"* — a
confusão entre os três era a fonte dos bugs.

**Por que existe:** *"the need to consume assets from multiple asset management systems became a
necessity"* — daí **Primary e URI Resolvers**. Um estúdio pode implementar `ArResolver` + `ArAsset` para
que identificadores próprios *"be used for references, payloads, and sublayer composition arcs throughout
USD"*, lendo de uma base de dados.

**[INF] A lição, e é barata de adotar sem adotar USD:** separar **três** conceitos que quase todo sistema
mistura em um: *o que o documento escreveu* (identifier), *o que isso significa agora* (resolved), e
*onde estão os bytes*. O PH2D já tem os três, mas **sem nomes distintos**: `SpriteSource::CookedTexture`
guarda um `LogicalTextureId` (identifier), o `LogicalTextureMap` resolve por tier (resolved), e o
`AssetDb` guarda bytes por `AssetId` (conteúdo). ⭐ **O padrão de 3 níveis já está lá, aplicado a UM tipo.**

### §1.7 — Fora do domínio criativo: versionar binários grandes **[COM, com fontes de fornecedor]**

| | Modelo | Ganha em | Perde em |
|---|---|---|---|
| **Git objects** | conteúdo endereçado por SHA, imutável | integridade, dedup | árvore inteira em cada clone |
| **Git LFS** | ponteiro no Git + blob num servidor | funciona com o fluxo Git | servidor obrigatório; caro em GB |
| **git-annex** | ponteiro + **múltiplos backends** (disco, rede, nuvem), com política por ficheiro | **offline**: *"users can continue their work with locally available files"* | complexidade operacional |
| **DVC** | cache local **content-addressed** + remoto | reprodutibilidade de pipeline | vocabulário de ML, não de arte |
| **Perforce Helix Core** | servidor central, **checkout exclusivo** | *"the only VCS powerful enough to handle many, massive files"*; **lock** resolve conflito de binário | servidor, custo, não-distribuído |

⭐ **[INF] O que isto diz ao PH2D, cujo alvo é o artista solo com Git:** o problema real de binário
grande não é armazenamento — é **conflito**. Perforce ganha porque **tranca**. Num app de artista solo
não há conflito a resolver, então **content-addressing (dedup + integridade) é a metade que compensa, e
o lock é a metade que não se aplica**. E o formato do PH2D é hoje **um ficheiro de projeto monolítico**,
que é o pior caso para Git: um blob que muda inteiro a cada save.

---

### §1.8 — ⭐ As seis leis que a comparação destila

1. **O índice vem antes do navegador.** O Content Browser do Unreal só existe porque o Asset Registry
   responde sem carregar. *(Unreal §1.3)*
2. **Identidade opaca funciona — e tem de ser versionada ao lado do conteúdo.** Godot e Unity sofrem o
   *mesmo* bug quando a identidade e o conteúdo se separam. *(§1.2, §1.4)*
3. **Ou você troca a identidade (Godot: `uid://`), ou você mantém o caminho e paga lápides
   (Unreal: redirectors).** Não há terceira opção estável, e as duas funcionam.
4. **A taxonomia que o artista vê não precisa de ser a árvore de ficheiros.** Catálogos por UUID
   (Blender) desacoplam as duas com um ficheiro de texto. *(§1.5)*
5. **Reimport/reindex disparado por movimentação é a fonte nº 1 de travamento.** Está medido no Godot
   e é o que a maturidade do Unreal evita lendo só headers. *(§1.2, §1.3)*
6. **Três conceitos, três nomes:** *identifier* (o que o documento escreveu) ≠ *resolved* (o que isso
   significa agora) ≠ *conteúdo* (os bytes). Misturá-los é a raiz de metade dos bugs do `Ar` 1.0. *(§1.6)*

---

## §2 — B5: assets **gerados dentro da ferramenta**

Aqui o prior art é escasso — mas não é nulo, e há **uma** fonte que respondeu a pergunta central com
números e uma limitação confessa.

### §2.1 — ⭐ O asset é o grafo, o bake, ou os dois? — Substance responde **[DOC — Adobe]**

Substance Designer tem **dois formatos, de propósito**:

| | `.sbs` | `.sbsar` |
|---|---|---|
| O que é | *"Substance Binary Source"* — **todos os nós, parâmetros e dependências** | *"Substance Archive"* — *"a compressed version… with **only the exposed parameters and outputs**"* |
| Editável | ✅ no Designer | ❌ *"not editable, but they can expose some parameters that allow users to tweak the material **without changing its structure**"* |
| Portátil | ❌ *"not compatible with other software"* | ✅ *"completely stand-alone: all resources required are embedded"* |
| Unidade | grafo | ⭐ **o PACOTE inteiro** — *"You always publish the whole package, not individual graphs"* |

⭐ **E a limitação confessa, que é a parte mais valiosa da secção inteira [DOC]:**

> *"some node properties are **static**, which means they cannot be computed on the fly once the graph is
> processed, and these properties and any parameters which contribute to their computation **will not be
> exposed** in a Substance 3D asset (SBSAR) published out of this graph."*

**[INF] Tradução para o PH2D:** *"publicar o grafo com parâmetros expostos"* **não é uma operação
completa** — há sempre um conjunto de parâmetros que só existe em tempo de autoria, porque mudá-los
mudaria a *estrutura*, não o *valor*. Um sistema honesto tem de **dizer ao autor quais parâmetros não
vão sobreviver à publicação**, no momento em que ele os expõe. Isto é uma exigência de UI, e é exatamente
a mesma família do *"um valor que não leva a lado nenhum não é OFERECIDO"* que o `vec_variants.rs` do
PH2D já pratica.

**As outras três respostas da indústria, para contraste:**

| Ferramenta | O asset é… | Como o utilizador escolhe |
|---|---|---|
| **Houdini HDA** | ⭐ **o grafo**, sempre — com interface **autorada** (§1.6 do doc irmão) | não escolhe: o HDA é vivo por construção; congelar é *"lock"*/cache explícito |
| **Substance** | **os dois formatos**, e o autor escolhe ao publicar | `.sbs` (vivo, interno) × `.sbsar` (parametrizável, selado) |
| **Blender** | **o data-block**, que pode ser malha **ou** node group (o grafo) | `link` (vivo, read-only) × `append` (cópia congelada) **[DOC-indireto]** |
| **Rive** | **o artboard**, com ViewModel; e desde 2025 a **fonte do artboard é trocável em runtime** | data binding |

⭐ **[INF] O padrão que atravessa todas:** ninguém escolhe *"grafo ou bake"* — **todas guardam o grafo e
tratam o bake como um cache**, e a escolha que expõem ao utilizador é outra: **vivo (segue a fonte) ×
congelado (não segue)**. `link` vs `append`, `.sbs` vs `.sbsar`, *instance* vs *detach*. **O PH2D já
tem esse verbo:** o `Detach` do componente vetorial.

### §2.2 — Vivo × congelado sem ambiguidade para o utilizador

As quatro ferramentas resolvem a ambiguidade do mesmo jeito, e **nenhuma** o faz por checkbox:

1. **Verbos distintos e irreversíveis-por-default** — `link`/`append`, *Place*/*Detach*, *instance*/*duplicate*.
2. **Sinal visual permanente na instância** — Unity (linha azul + badges), Figma (losango), Blender
   (ícone de biblioteca), Godot (ícone de cena instanciada).
3. **O congelado NÃO finge ser vivo** — depois do `append`/`Detach`, o vínculo some da UI.
4. ⭐ **flecs põe a escolha no TIPO** (`(OnInstantiate, Inherit|Override)`), antes de qualquer instância
   existir — a única fonte que a torna **não-ambígua por construção** em vez de por convenção de UI
   (§1.5 do doc irmão).

### §2.3 — Thumbnails: gerar previews sem travar a UI **[DOC/COM]**

O que a indústria faz, e o PH2D pode medir contra:

- **Unreal** — thumbnail fica **no header do `.uasset`** e o Content Browser *"displays them without
  loading any object"*. Renderizador de thumbnail por tipo, com cache. **[DOC/COM]**
- **Unity** — `AssetPreview` é **assíncrono**, com `SetPreviewTextureCacheSize` para limitar o cache;
  thumbnails estáticos a **128×128** guardados na pasta `Library/`. **[DOC/COM]**
- **Blender** — preview gerado ao *marcar* como asset, guardado **dentro do `.blend`**, e substituível à
  mão. **[DOC-indireto]**
- **Previews animados [COM]** — a prática relatada é: cachear em memória **só a pasta visível**, e usar
  **um frame de pico estático** ao filtrar/pesquisar.

⭐ **[INF] As três leis que isto destila:** (a) **o preview viaja com o asset**, não numa pasta paralela
que envelhece; (b) **assíncrono com cache limitado**, nunca sob demanda no frame; (c) **animado só para
o que está visível**. E o PH2D parte de uma posição privilegiada — ele **já renderiza tudo**, já tem
`render_texture_preview` para o pincel, e já tem `GameRt` (render-target offscreen com tonemap).

### §2.4 — Taxonomia: pastas (hierarquia rígida) × catálogos/tags (classificação múltipla)

| | Modelo | Quem usa | Consequência |
|---|---|---|---|
| **Pastas** | 1 asset = 1 lugar | Unity, Godot, Unreal | mover = evento de sistema (reimport, redirector, quebra) |
| **Catálogos por UUID** | taxonomia **separada** do disco | ⭐ **Blender** | reorganizar a vista **não toca** nos ficheiros |
| **Tags/coleções** | N classificações por asset | Unreal *Collections*, Blender *tags*, DAMs | busca por faceta; ⚠️ sem hierarquia, vira sopa |
| **Busca semântica/visual** | sem taxonomia | DAMs comerciais **[COM]** | ⚠️ requer modelo de ML e um índice; **fora de escopo declarado** para v1 |

⭐ **[INF] A escolha do Blender é a que melhor casa com o PH2D**, por um motivo estrutural: no PH2D o
asset **nasce dentro de um documento**, não como um ficheiro que alguém largou numa pasta. Uma taxonomia
que exige um lugar no disco obriga a inventar um ficheiro para cada coisa marcada.

### §2.5 — Grafo de dependências e coleta de órfãos

- **Unreal** é o único com ferramenta de 1ª classe: **Reference Viewer** (grafo com profundidade
  configurável, filtro por tipo, restrição a Collection, *Show Reference Tree*), **Size Map** e **Asset
  Audit**. **[DOC]**
- **Godot** guarda `[deps]` no `.import` e tem o *Dependency Editor* (o diálogo *"owners of"*). **[DOC]**
- **Unity** e **Blender** têm a informação mas não um visualizador comparável. **[INF]**
- ⚠️ **Coleta de lixo de órfãos:** nenhuma das quatro faz automaticamente, e a razão é boa — *"não
  referenciado" ≠ "não desejado"*: um asset pode ser referenciado por script, por nome, por conteúdo
  autorado, ou simplesmente ainda não usado. O que existe é **relatório**, não colheita. **[INF]**

### §2.6 — Escopos e sombreamento

| Escopo | Quem tem | Como resolve conflito |
|---|---|---|
| Documento | todos | — |
| Projeto | Unity/Godot/Unreal (a pasta do projeto) | é o único; não há sombreamento |
| **Biblioteca do utilizador** | ⭐ Blender (*asset libraries* configuráveis nas Preferences) | a UI mostra as bibliotecas **lado a lado**, não fundidas — **não há sombreamento porque não há fusão** |
| Instalada/partilhada | Unreal (plugins/*Engine content*), Quixel Bridge | namespace separado (`/Engine/…`, `/Game/…`) |

⭐ **[INF] O padrão vencedor é evitar o problema:** ninguém funde escopos e depois resolve precedência —
todos os **mostram separados** e o utilizador escolhe de onde puxa. É a decisão mais barata da secção, e
a que mais dor evita.

### §2.7 — Busca

- **Unreal** — filtros por tipo/classe + **Collections** (estáticas, dinâmicas por query) + filtros
  salvos + busca no Reference Viewer. **[DOC]**
- **Blender** — busca por nome + **tags** + catálogo. **[DOC-indireto]**
- **Godot/Unity** — busca por nome e tipo.
- **Busca visual / por cor / por similaridade** — existe e é **maduro em DAMs comerciais** (Bynder,
  Cloudinary, MuseDAM): cor por hex/RGB/HSL, similaridade por rede neural, tags automáticas **[COM]**.
  ⚠️ **Nenhuma engine de jogo tem isto.**
  **[INF] Viável e desejável para o PH2D?** *Viável*: cor dominante é trivial (o app já decodifica todo
  pixel; um histograma OKLab por asset custa uma passagem). *Similaridade por rede neural* não é —
  exigiria o `tract` (já presente para o denoise de áudio, **feature `audio-ml` OFF por default**) e um
  modelo, e cairia na mesma fronteira dura que o áudio já declarou. **A cor é barata; a similaridade é um
  projeto.**

### §2.8 — Import/export e portabilidade

- **Unreal** — *Migrate* copia o asset **com todas as dependências** para outro projeto, usando o Asset
  Registry para fechar o conjunto. **[DOC/COM]**
- **Substance** — `.sbsar` é *"completely stand-alone: all resources required are embedded"*. **[DOC]**
- **Blender** — um `.blend` **é** o pacote; `append` traz cópia, `link` mantém vínculo ao ficheiro
  original (⚠️ que precisa de continuar a existir). **[DOC-indireto]**
- **Godot** — não há *migrate*; copiar pastas à mão, e os `uid://` **seguem** se os sidecars forem junto.

⭐ **[INF] A regra que se repete:** *portabilidade = fecho transitivo do grafo de dependências*. Sem o
grafo (§2.5), *"exportar um asset"* é adivinhação. **Isto liga B4 e B5 num nó só: o índice de dependências
não é uma ferramenta de auditoria — é o pré-requisito de exportar, migrar e apagar com segurança.**

---

## §3 — O que isto põe na mesa para o PH2D (sem escolher)

1. **O PH2D já tem os 3 níveis do `Ar`** (identifier / resolved / conteúdo) — aplicados a **um** tipo
   (`LogicalTextureId` → `LogicalTextureMap` → `AssetDb`). Generalizá-los é reusar um padrão já provado
   na casa, não importar um novo.
2. **O índice vem antes do navegador** (§1.8 lei 1) — e hoje o PH2D **não tem índice nenhum**: o `AssetDb`
   é um mapa em memória de bytes, sem metadado, sem tipo, sem preview, sem dependência.
3. **Catálogo por UUID desacoplado da pasta** (Blender) é a taxonomia que casa com um app onde o asset
   nasce dentro de um documento — e é um **ficheiro de texto**, não uma base de dados.
4. **"Marcar como asset" ≠ "exportar"**, e a diferença decide se o asset é *um lugar* ou *um estado*.
5. **Ninguém escolhe entre grafo e bake:** guardam o grafo, tratam o bake como cache, e expõem
   **vivo × congelado**. O PH2D já tem o verbo (`Detach`) e já tem a lei (geometria derivada por frame).
6. ⚠️ **Substance confessa o limite de "expor parâmetros":** há parâmetros que mudam **estrutura**, não
   **valor**, e não sobrevivem à publicação. Um sistema honesto avisa **no momento de expor**.
7. **O ficheiro de projeto monolítico do PH2D é o pior caso para Git** (§1.7) — e é um fato de hoje, não
   uma consequência de nenhuma proposta.
8. ⚠️ **Duas patologias que a concorrência pagou e que o PH2D pode evitar de graça:** (a) a identidade
   deve ser versionada ao lado do conteúdo (Godot/Unity); (b) reindex disparado por movimentação trava
   (Godot).

---

## §4 — Fontes

**Godot** (MIT)
- ⭐ [UID changes coming to Godot 4.4](https://godotengine.org/article/uid-changes-coming-to-godot-4-4/) · [Discussão de feedback sobre UIDs generalizados](https://github.com/godotengine/godot-proposals/discussions/11574) · [Import process](https://docs.godotengine.org/en/3.1/getting_started/workflow/assets/import_process.html)
- ⚠️ [proposals#8360 — evitar reimport ao mover](https://github.com/godotengine/godot-proposals/issues/8360) · [#29607 — mover ficheiros quebra referências](https://github.com/godotengine/godot/issues/29607) · [PR #81657](https://github.com/godotengine/godot/pull/81657) · [PR #84520](https://github.com/godotengine/godot/pull/84520) · [#114493 — sobrescrever ficheiro pode mudar UID](https://github.com/godotengine/godot/issues/114493)

**Unreal** (proprietária — só documentação)
- ⭐ [Asset Registry](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-registry-in-unreal-engine) · [Asset Redirectors](https://dev.epicgames.com/documentation/unreal-engine/asset-redirectors-in-unreal-engine) · [Reference Viewer](https://dev.epicgames.com/documentation/en-us/unreal-engine/reference-viewer-in-unreal-engine) · [Asset Management](https://dev.epicgames.com/documentation/unreal-engine/asset-management-in-unreal-engine) · [Cooking and Chunking](https://dev.epicgames.com/documentation/unreal-engine/cooking-content-and-creating-chunks-in-unreal-engine) · [Assets and Packages](https://dev.epicgames.com/documentation/en-us/unreal-engine/assets-and-packages)

**Unity** (proprietária — só documentação; patologias por relato de comunidade)
- [`AssetPreview`](https://docs.unity3d.com/ScriptReference/AssetPreview.html) · [How to Git with Unity (thoughtbot)](https://thoughtbot.com/blog/how-to-git-with-unity) · [Mismatched GUID em VCS (Unity Discussions)](https://discussions.unity.com/t/mismatched-guid-issue-between-different-users-in-version-control/798529)

**Blender** (GPL — ⚠️ **só manual/notas, por resumo de busca; página 403 ao fetch**)
- [Asset Browser (manual)](https://docs.blender.org/manual/en/latest/editors/asset_browser.html) · [Asset Browser (release notes 2.92)](https://developer.blender.org/docs/release_notes/2.92/asset_browser/) · [Asset Browser Project Update](https://code.blender.org/2021/06/asset-browser-project-update/) · [Asset Browser Workshop Outcomes](https://code.blender.org/2021/06/asset-browser-workshop-outcomes/)

**OpenUSD** (Apache-2.0 modificada)
- ⭐ [Asset Resolution (Ar) 2.0 white paper](https://openusd.org/dev/wp_ar2.html) · [`ArResolver` API](https://openusd.org/dev/api/class_ar_resolver.html) · [Ar: Asset Resolution](https://openusd.org/dev/api/ar_page_front.html) · [Asset Resolver — USD Survival Guide](https://lucascheller.github.io/VFX-UsdSurvivalGuide/pages/core/plugins/assetresolver.html)

**Substance 3D Designer** (proprietária — só documentação)
- ⭐ [Publishing Substance 3D asset files (SBSAR)](https://substance3d.adobe.com/documentation/sddoc/publishing-substance-3d-asset-files-sbsar-200574380.html)

**Versionamento de binários** (fornecedores + comunidade)
- [Perforce — Version Control for Binary Files](https://www.perforce.com/blog/vcs/version-control-for-binary-files) · [Git LFS e alternativas comparadas a 10/100/1000 GB](https://codenote.net/en/posts/github-large-binaries-git-lfs-alternatives/) · [Git LFS e DVC](https://medium.com/@pablojusue/git-lfs-and-dvc-the-ultimate-guide-to-managing-large-artifacts-in-mlops-c1c926e6c5f4)

**Busca visual em DAM** (fornecedores — **[COM]**, para calibrar viabilidade, não para adotar)
- [Cloudinary — DAM Visual Search](https://cloudinary.com/documentation/dam_visual_search) · [Bynder — visual & semantic search](https://www.bynder.com/en/blog/mastering-bynders-advanced-content-retrieval/)

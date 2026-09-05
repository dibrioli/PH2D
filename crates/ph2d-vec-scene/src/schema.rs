//! ⭐ **A VERSÃO DO WIRE-FORMAT, e a ESCADA que a explica** — módulo irmão do [`crate`] pelo tecto
//! de LOC, e o corte é por RESPONSABILIDADE: ali mora o MODELO do documento; aqui, o número que
//! diz **que bytes** ele escreve, com a história de cada degrau ao lado dele.
//!
//! ⚠️ **A escada fica COLADA ao número de propósito.** Ela é o que impede o próximo bump de ser
//! escolhido em vez de contado: quem sobe o número lê, no mesmo ecrã, o que cada degrau anterior
//! mudou e por que quebrou. Guardá-la noutro ficheiro é o que faz uma escada envelhecer.

/// Versão do wire-format de save (postcard é posicional → bump a cada mudança de
/// schema). v2: `VertexKind` ganhou `Symmetric`. v3: `stroke` virou
/// [`StrokeSpec`] (cap/join/dash). v4: `fill` virou [`Paint`] (sólido + gradientes
/// Linear/Radial/MultiPoint). v5: [`GradientPoint`] ganhou `jitter`. v6: `VecPath`
/// ganhou `subpaths` + `fill_rule` (compound paths). v8: [`VecVertex`] ganhou
/// `corner_radius` (Live Corners — [`crate::corner_live`]). v9: [`VecPath`] ganhou
/// `effects`, a pilha de Live Path Effects ([`crate::effect`], ADR-0132). v10: a entrada da
/// pilha virou [`effect::FxEntry`] (o efeito + se está LIGADO) — desarmar sem perder os
/// parâmetros. v11: [`effect::PathEffect`] ganhou `Repeat`/`Twist`/`Bloat` ([`crate::fx_repeat`],
/// [`crate::fx_warp`]). v12: o `Twist` foi CORTADO e os índices fecharam-se atrás dele — a
/// v11 nunca chegou a existir num save, e a razão do corte está no `fx_warp`.
///
/// ⚠️ **Apender um variant obriga a bump, embora os saves antigos continuem a ler CERTO.** Os
/// índices anteriores não se mexem, então v10 lido por v11 está correto — o que quebra é o
/// sentido inverso: um save v11 com um Repeater, lido por um binário v10, encontra um índice de
/// variant que não conhece. O bump é o que transforma isso num erro de versão em vez de um
/// postcard a falhar longe da causa. (Migração robusta = cutover, Fase R.)
/// v14: [`StrokeSpec`] ganhou [`StrokeAlign`] (Centre/Inner/Outer). Campo APENDADO, e o bump é
/// obrigatório nos DOIS sentidos — o postcard não sinaliza ausência, então um save v13 lido por
/// v14 chega ao fim dos bytes no campo novo (`Hit the end of buffer`, medido em 2026-08-01) e um
/// save v14 lido por v13 traz um byte a mais. O número é o que transforma os dois casos num erro
/// de versão em vez de num postcard a falhar longe da causa.
/// v15: [`Paint`] ganhou `Pattern(Box<PatternFill>)` — o *Texture Pattern* (plano 33, W3). Variante
/// **apendada**, então um save v14 lido por v15 está correcto; o que quebra é o sentido inverso (um
/// v15 com um padrão, lido por um binário v14, encontra um índice de variante que não conhece), e o
/// bump é o que transforma isso num erro de versão. ⚠️ O `Box` **não** aparece no wire: o postcard
/// serializa através dele.
/// v16: o [`StrokeSpec`] deixou de ter `color: Rgba8` e passou a ter `paint: StrokePaint`
/// (`Solid | Pattern`) — o *padrão no traço* (plano 35, wave A). ⚠️⚠️ **DESTRUTIVO nos dois
/// sentidos**, ao contrário de todos os degraus acima: um campo **mudou de tipo no meio da
/// estrutura**, então onde um v15 tem os 4 bytes de um `Rgba8` um leitor v16 espera o
/// **discriminante** de um enum. Os bytes não desaparecem — passam a significar outra coisa, que é
/// o pior modo de falha que há (⛔ *ler torto sem erro nenhum*), e é o número que o transforma num
/// erro de versão. ⚠️ **Este degrau ficou por escrever quando a wave A o subiu** — a escada parou no
/// v15 e o `project_schema.rs` documentou-o sozinho; *uma escada com um degrau a menos manda a
/// próxima janela procurar o que mudou no diff*.
/// v17: o [`StrokePaint`] ganhou `Brush(Box<BrushStroke>)` — o **pincel de contorno** (plano 36,
/// W1). Variante **APENDADA**, do lado aditivo da regra: um save v16 lido por v17 está correcto, e
/// o que quebra é o inverso (um v17 com pincel encontra, num binário v16, um índice de variante que
/// ele não conhece). ⭐ É exactamente o degrau barato que a nota do `PROJECT_SCHEMA` 101 previu ao
/// desenhar o `StrokePaint` como enum.
/// v19: [`VecPath`] ganhou `opacity` e `blend` — a **opacidade e o modo de mistura do OBJECTO**
/// (estudo 42 item 2). Dois campos **apendados ao fim**, do lado destrutivo da regra nos dois
/// sentidos: um save v18 lido por este layout fica **sem bytes** nos dois últimos campos
/// (`Hit the end of buffer`) e um v19 lido por um binário v18 traz bytes a mais. ⚠️ O `Opacity` é
/// um newtype de `f32` e o postcard serializa **através** dele (4 bytes), e o `BlendMode` é um
/// enum de 22 variantes ⇒ **1 byte** de varint. ⭐ O default é o neutro nos dois (`1.0` / `Normal`),
/// então uma cena que nunca lhes toque desenha byte a byte o que desenhava.
/// v20: [`VecPath`] ganhou `paints` — a **PILHA DE APARÊNCIA** (estudo 42 item 4), N
/// preenchimentos e N contornos numa forma. Um `Vec` **apendado ao fim**, pela mesma escada do
/// `effects`: vazio é o neutro, e uma cena que nunca lhe toque desenha byte a byte o que desenhava
/// (o postcard escreve **1 byte** de comprimento zero). ⚠️ O degrau é destrutivo nos dois sentidos
/// pela razão de sempre — um save v19 lido por este layout fica sem bytes no último campo.
pub const VEC_SCENE_SCHEMA_VERSION: u32 = 20;

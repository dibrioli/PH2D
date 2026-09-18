//! ⭐⭐⭐ **O IDIOMA DE TESTE** — a lei que deforma cada palavra que sai da tabela.
//!
//! O porquê e o que ele não prova vivem no irmão [`crate::idioma`]. Aqui está a lei, e as **três**
//! decisões dela, cada uma com a medição ou o mecanismo ao lado.
//!
//! # ⭐ 1. O ALONGAMENTO É MEDIDO, e a medição MUDOU o desenho
//!
//! A 1.ª redacção levava uma constante (*«+35 %, que é quanto uma língua cresce»*) — um número sem
//! recurso nomeado, que é o que o `CLAUDE.md` §0.0 proíbe. O oráculo existia instalado: **quatro
//! editores da nossa classe** (Krita · Inkscape · GIMP · Synfig Studio) trazem os catálogos `.mo`
//! deles em `/usr/share/locale/`, e o que se lê ali é **saída**, nunca fonte (§0.9). Medido sobre
//! **89 832** pares rótulo↔tradução em **5** línguas (pt-BR · de · fr · es · it), excluindo o que
//! não é rótulo (multilinha, `%` de formato, sem letras):
//!
//! | original (caracteres) | n | p50 | p75 | **p90** | p95 |
//! |---|---:|---:|---:|---:|---:|
//! | 1..6 | 7 981 | 1,17 | 1,50 | **2,00** | 2,40 |
//! | 7..12 | 20 375 | 1,20 | 1,50 | **1,83** | 2,08 |
//! | 13..20 | 23 831 | 1,25 | 1,47 | **1,70** | 1,87 |
//! | 21..35 | 18 806 | 1,24 | 1,42 | **1,61** | 1,73 |
//! | 36..70 | 13 171 | 1,19 | 1,33 | **1,48** | 1,58 |
//! | 71+ | 5 668 | 1,16 | 1,26 | **1,37** | 1,44 |
//! | TODOS | 89 832 | 1,21 | 1,42 | 1,67 | 1,88 |
//!
//! ⭐⭐ **O achado é que o alongamento NÃO é uma constante: ele cresce quando o rótulo encolhe.**
//! Um rótulo de até 6 caracteres **dobra** no percentil 90, e um parágrafo cresce `1,37`. ⇒ um
//! número único estaria `1,65×` abaixo do necessário exactamente nos rótulos curtos, que são os que
//! vivem nas colunas apertadas deste app — *a tensão faltaria onde a interface parte*.
//!
//! ⚠️ **A barra é o p90 e isso é uma escolha declarada:** nove em cada dez traduções reais cabem
//! neste alongamento. As outras não, e um rótulo que sobreviva ao idioma de teste **ainda pode**
//! cortar numa língua real. *Isto é uma tensão, não uma prova de suficiência.*
//!
//! ⚠️ **`11,1 %` dos pares MEDIDOS encolhem** (`< 1,00`). O idioma de teste nunca encolhe, e não
//! precisa: uma palavra mais curta não parte uma coluna. Ponto cego declarado.
//!
//! ⚠️ **A régua é o CARACTERE e o que corta é o PIXEL.** As letras acentuadas que se usam aqui são
//! do mesmo bloco latino das originais e medem praticamente o mesmo, então as duas grandezas andam
//! juntas — mas não são a mesma, e quem quiser a medida em pixels usa o
//! `cada_nome_deste_painel_cabe_na_coluna_da_seccao` com o idioma de teste ligado.
//!
//! # ⛔ 2. UM MARCADOR SAI VERBATIM, e sem isso o [`crate::tr_with`] parte
//!
//! `"{nodes}: {count} · {cost} {ms} ms"` é um MODELO: o `tr_with` procura `{nome}` e substitui.
//! Deformar o `{nodes}` faria a procura falhar e a frase sair com o marcador escrito na tela.
//! ⇒ tudo entre `{` e `}` é copiado letra a letra, **e não conta para o alongamento** — o que
//! cresce é o texto que uma tradução traduziria, não o valor que o código entrega.
//!
//! # ⛔ 3. A CHAVE DESCONHECIDA FICA CRUA
//!
//! O [`crate::tr`] devolve a própria chave quando não a conhece (*missing-key passthrough*), e meio
//! repo pergunta *«esta chave existe?»* com `tr(k) != k` — o [`crate::TextKey::key`] tem isso
//! escrito. Deformar o caminho da falha partiria **todos** esses censos de uma vez, e trocaria um
//! identificador cru (feio de propósito, reconhecível) por um identificador cru acentuado.
//! ⇒ a deformação só toca no que a tabela **respondeu**.
//!
//! # ⚠️ Os parênteses são `[` e `]`, e a razão é um vermelho que este repo já pagou
//!
//! Um parêntese matemático (`⟦`/`⟧`) diria melhor *«isto é um invólucro»* e arrisca **tofu** se a
//! fonte não o tiver — e um tofu foi um dos vermelhos do portão de 01/09. O ASCII está em toda
//! fonte, e o interior acentuado já torna a palavra inconfundível. ⭐ **O `]` é o detector de
//! corte:** se ele não aparece no ecrã, aquele rótulo foi elidido.

use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

/// O alongamento alvo em **por cento**, por comprimento do original — a coluna `p90` da tabela do
/// cabeçalho. Cada par é `(comprimento máximo do balde, alvo)`, do mais curto ao mais longo.
///
/// ⚠️ Inteiros e não `f32`: uma razão medida não é um número de UI, e a aritmética inteira aqui é
/// exacta (`base * pct / 100`, arredondado para cima).
const ALONGAMENTO: &[(usize, usize)] = &[
    (6, 200),
    (12, 183),
    (20, 170),
    (35, 161),
    (70, 148),
    (usize::MAX, 137),
];

/// O caractere de enchimento — `·` (U+00B7), do mesmo bloco latino das letras acentuadas, logo
/// presente em toda fonte que já desenhe um `á`.
const ENCHIMENTO: char = '\u{b7}';

/// A letra deformada, ou a própria se não for uma letra ASCII.
///
/// ⚠️ **Legível de propósito**: o alvo é o dono conseguir ler `[Áppĺý··]` e reconhecer *Apply*. Uma
/// deformação ilegível tornaria o smoke inútil — ele precisa de dizer *«esta palavra veio da
/// tabela»*, não de esconder qual palavra é.
fn acentua(c: char) -> char {
    match c {
        'a' => 'á',
        'b' => 'ƀ',
        'c' => 'ç',
        'd' => 'ð',
        'e' => 'é',
        'f' => 'ƒ',
        'g' => 'ĝ',
        'h' => 'ĥ',
        'i' => 'í',
        'j' => 'ĵ',
        'k' => 'ķ',
        'l' => 'ĺ',
        'm' => 'ɱ',
        'n' => 'ñ',
        'o' => 'ó',
        'p' => 'þ',
        'q' => 'ɋ',
        'r' => 'ŕ',
        's' => 'š',
        't' => 'ŧ',
        'u' => 'ú',
        'v' => 'ṽ',
        'w' => 'ŵ',
        'x' => 'ẋ',
        'y' => 'ý',
        'z' => 'ž',
        'A' => 'Á',
        'B' => 'Ɓ',
        'C' => 'Ç',
        'D' => 'Ð',
        'E' => 'É',
        'F' => 'Ƒ',
        'G' => 'Ĝ',
        'H' => 'Ĥ',
        'I' => 'Í',
        'J' => 'Ĵ',
        'K' => 'Ķ',
        'L' => 'Ĺ',
        'M' => 'Ɱ',
        'N' => 'Ñ',
        'O' => 'Ó',
        'P' => 'Þ',
        'Q' => 'Ɋ',
        'R' => 'Ŕ',
        'S' => 'Š',
        'T' => 'Ŧ',
        'U' => 'Ú',
        'V' => 'Ṽ',
        'W' => 'Ŵ',
        'X' => 'Ẋ',
        'Y' => 'Ý',
        'Z' => 'Ž',
        outro => outro,
    }
}

/// O alvo em por cento para um original de `n` caracteres traduzíveis.
fn alvo_pct(n: usize) -> usize {
    ALONGAMENTO
        .iter()
        .find(|(ate, _)| n <= *ate)
        .map_or(137, |(_, pct)| *pct)
}

/// ⭐ **A LEI, pura** — `"Apply"` → `"[Áþþĺý·]"`, `"{n} items"` → `"[{n} íŧéɱš··]"`.
#[must_use]
pub fn deforma(texto: &str) -> String {
    let mut corpo = String::with_capacity(texto.len() * 2);
    let mut base = 0_usize;
    let mut resto = texto;
    while let Some(abre) = resto.find('{') {
        for c in resto[..abre].chars() {
            corpo.push(acentua(c));
            base += 1;
        }
        let depois = &resto[abre + 1..];
        match depois.find('}') {
            // O marcador inteiro, verbatim, e fora da contagem.
            Some(fecha) => {
                corpo.push('{');
                corpo.push_str(&depois[..=fecha]);
                resto = &depois[fecha + 1..];
            }
            // ⚠️ Um `{` sem par é texto, e é a MESMA tolerância do `tr_with` — as duas leis lêem
            // o mesmo modelo, e discordar aqui deixaria uma frase com marcador aberto a contar
            // de uma maneira num sítio e de outra no outro.
            None => {
                corpo.push('{');
                base += 1;
                resto = depois;
            }
        }
    }
    for c in resto.chars() {
        corpo.push(acentua(c));
        base += 1;
    }

    let alvo = (base * alvo_pct(base)).div_ceil(100);
    // Os dois parênteses já são dois caracteres do alongamento.
    let enche = alvo.saturating_sub(base).saturating_sub(2);
    let mut out = String::with_capacity(corpo.len() + enche + 2);
    out.push('[');
    out.push_str(&corpo);
    for _ in 0..enche {
        out.push(ENCHIMENTO);
    }
    out.push(']');
    out
}

/// A memória do que já foi deformado, para o `tr` continuar a devolver `&'static str`.
///
/// ⚠️ **Deformar por chamada seria um `Box::leak` POR QUADRO**, que é precisamente o defeito que o
/// doc do [`crate::tr`] já nomeia sobre a chave desconhecida. A tabela é finita (~5 000 entradas),
/// logo vazar uma vez cada é um custo com tecto.
///
/// ⚠️ A chave é o **ponteiro** do `&'static str` inglês — estável e único por literal. Duas chaves
/// de i18n cujo valor o compilador tenha fundido partilham a entrada, e isso está certo: é a mesma
/// palavra. ⚠️ Duas threads podem deformar a mesma entrada ao mesmo tempo e vazar duas cópias; a
/// segunda escrita ganha e a primeira fica órfã — **uma vez**, e por isso não se paga um `Mutex` no
/// caminho de leitura.
///
/// ⛔ **`BTreeMap` e não `HashMap`**, que é tipo proibido neste repo por um lint estrutural — a
/// espinha do determinismo. Aqui a ordem não decide nada e o tipo entra na mesma: *uma excepção a
/// uma lei estrutural custa mais do que os `log n` que ela poupa*, e sobre ~5 000 entradas o `log n`
/// é `12` comparações de um inteiro.
static MEMORIA: OnceLock<RwLock<BTreeMap<usize, &'static str>>> = OnceLock::new();

/// O texto no idioma de teste, dado o que a tabela respondeu em inglês.
pub(crate) fn traduz(key: &str, ingles: &'static str) -> &'static str {
    // ⛔ A chave desconhecida volta CRUA — ver §3 do cabeçalho.
    if ingles == key {
        return ingles;
    }
    let memoria = MEMORIA.get_or_init(|| RwLock::new(BTreeMap::new()));
    let id = ingles.as_ptr() as usize;
    if let Ok(m) = memoria.read()
        && let Some(v) = m.get(&id)
    {
        return v;
    }
    let novo: &'static str = Box::leak(deforma(ingles).into_boxed_str());
    if let Ok(mut m) = memoria.write() {
        m.insert(id, novo);
    }
    novo
}

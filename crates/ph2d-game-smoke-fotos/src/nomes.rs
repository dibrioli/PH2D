//! **Os nomes do smoke do jogo**: cada leitor de `PH2D_GAME_SMOKE_FOTO` declara os seus (`NOMES`), e o arranque
//! recusa ALTO o nome que nenhum aceita — antes, um nome errado deixava a cena por preparar, calada.
use ph2d_physics_scene::{cena_projectile, cena_topdown, fotos_dos_cartoes};

/// Um leitor: os nomes que declara (separados por espaço) e a porta que diz se aceita um nome.
pub type Leitor = (&'static str, fn(&str) -> bool);

/// Os cinco leitores de `PH2D_GAME_SMOKE_FOTO`: a lista de cada um e a sua porta.
pub const LEITORES: [Leitor; 5] = [
    (fotos_dos_cartoes::NOMES, |n| {
        fotos_dos_cartoes::caso(n).is_some()
    }),
    (crate::foto::NOMES, |n| crate::foto::foto_de(n).is_some()),
    (cena_topdown::NOMES, |n| {
        cena_topdown::foto(Some(n)).is_some()
    }),
    (ph2d_sprite_anim_scene::NOMES, |n| {
        ph2d_sprite_anim_scene::foto_de(n).is_some()
    }),
    (cena_projectile::NOMES, |n| {
        cena_projectile::foto(Some(n)).is_some()
    }),
];

/// O nome de foto que algum leitor aceita.
#[must_use]
pub fn aceita_foto(nome: &str) -> bool {
    LEITORES.iter().any(|(_, aceita)| aceita(nome.trim()))
}

/// A recusa de `var=pedido` quando `aceita` o nega: o nome, a sugestão mais próxima e a lista dos válidos.
#[must_use]
pub fn recusa(
    var: &str,
    pedido: &str,
    validos: &str,
    aceita: impl Fn(&str) -> bool,
) -> Option<String> {
    let p = pedido.trim();
    let perto = validos
        .split_whitespace()
        .min_by_key(|v| (!v.contains(p) || p.is_empty(), distancia(p, v)));
    let sugere = perto.map_or_else(String::new, |v| format!(" Quis dizer «{v}»?"));
    (!aceita(p)).then(|| format!(
        "[game-smoke] {var}=«{pedido}» não é nome que algum leitor aceite.{sugere}\n  válidos: {validos}"
    ))
}

/// A distância de edição (Levenshtein) entre `a` e `b`.
fn distancia(a: &str, b: &str) -> usize {
    let mut linha: Vec<usize> = (0..=b.chars().count()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diag = linha[0];
        linha[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cima = linha[j + 1];
            linha[j + 1] = (diag + usize::from(ca != cb))
                .min(cima + 1)
                .min(linha[j] + 1);
            diag = cima;
        }
    }
    linha[linha.len() - 1]
}

/// Todos os nomes de foto válidos, separados por espaço.
#[must_use]
pub fn nomes_da_foto() -> String {
    LEITORES
        .iter()
        .map(|(n, _)| *n)
        .collect::<Vec<_>>()
        .join(" ")
}

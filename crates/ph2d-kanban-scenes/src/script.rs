//! O script do `PH2D_CODE_SMOKE=10` (o editor de scripts): mexe no Kanban da cena 1. Mora aqui, ao
//! lado do Kanban que ele lê, e um teste corre-o sobre ele.

pub const KANBAN_NAME: &str = "kanban.luau";

/// ⭐ **O script da cena 10.** As agulhas do condutor das fotos
/// (`ph2d_app_code::smoke_drive::STEPS_10`) vivem neste texto (conferido no teste do `ph2d-app-code`).
pub const KANBAN_LUAU: &str = r#"-- kanban.luau — este script mexe no KANBAN «Short film — production» do projeto, não na cena.
--
-- 1) Na linha marcada com 1), escreva  card:  — a lista mostra o que um cartão sabe fazer
--    (move, comment, set, check…). Escolha um: a ajuda diz o que ele pede.

local k = ph2d.kanban.get("Short film — production")
assert(k, "o Kanban da cena")
print(k.name, #k:lists(), "listas")

-- os cartões atrasados do projeto inteiro
for _, c in ph2d.kanban.search("due:overdue") do
  print("atrasado:", c.title, "na", c.list.name)
end

-- comentar e mover um cartão
local card = k:cards("Animate shot 12")[1]
card:comment("Blocking pronto — @Rui, pode rever?")
card:move("Review")
-- 1) escreva aqui, numa linha nova
"#;

/// A linha onde o condutor das fotos escreve `card:` (o fim dela, e Enter).
pub const NEEDLE: &str = "-- 1) escreva aqui";

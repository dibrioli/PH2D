//! **AS EXPLICAÇÕES do editor de scripts** (CodeEditor Onda 2, I8): uma por classe de mensagem do
//! corpus do I0c, escrita à mão. `code.explain.<id>` é o título (uma linha, com os nomes reais) e
//! `code.explain.<id>.why` o porquê e o que fazer. Quem escolhe a classe é `ph2d-code-explain`; o
//! censo dele reprova uma classe sem as duas chaves.

/// A tradução de uma chave `code.explain.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── Sintaxe (o Luau parou de ler o script) ──
        "code.explain.incomplete_statement" => "This line does nothing on its own",
        "code.explain.incomplete_statement.why" => {
            "A line must give a value to something (x = 1) or call a function (print(x)). Look for a missing = or ( on this line."
        }
        "code.explain.expected_expression" => "A value is missing before “{got}”",
        "code.explain.expected_expression.why" => {
            "Luau wanted a value here (a number, a name, a string…) and found “{got}”. Look for a doubled sign, like = =, or a gap before it."
        }
        "code.explain.expected_expression_eof" => "The script ends where a value should be",
        "code.explain.expected_expression_eof.why" => {
            "The last line stops in the middle, right after a sign like = or +. Finish the line with a value."
        }
        "code.explain.variable_name" => "A name is missing here",
        "code.explain.variable_name.why" => {
            "local and for need a new name next: local speed = 2. A name can’t be a Luau word (end, then, local…) or start with a digit."
        }
        "code.explain.unclosed" => "The “{open}” on line {line} is never closed",
        "code.explain.unclosed.why" => {
            "Every “{open}” needs a “{close}”. Add “{close}” where this part ends — or remove the extra “{open}”."
        }
        "code.explain.unclosed_here" => "A “{close}” is missing",
        "code.explain.unclosed_here.why" => {
            "Something opened before this point needs a “{close}”. Add it where that part ends."
        }
        "code.explain.expected_then" => "if needs then",
        "code.explain.expected_then.why" => {
            "Write the condition, then the word then: if x > 0 then … end. To compare, use == (a single = gives a value)."
        }
        "code.explain.expected_do" => "for needs do",
        "code.explain.expected_do.why" => {
            "A loop reads for i = 1, 10 do … end. Check the commas and the word do."
        }
        "code.explain.unclosed_block" => "The “{kw}” on line {line} has no end",
        "code.explain.unclosed_block.why" => {
            "Every function, if, for and while closes with end. Add the missing end where that block finishes."
        }
        "code.explain.extra_token" => "“{got}” is left over",
        "code.explain.extra_token.why" => {
            "The script was already complete before this point. Usually it is an extra end, or a word left behind — remove it."
        }
        "code.explain.malformed_string" => "This text is never closed",
        "code.explain.malformed_string.why" => {
            "A text starts and ends with the same quote, on the same line: \"hello\" or 'hello'. Add the closing quote."
        }
        "code.explain.table_comma" => "A comma is missing in this table",
        "code.explain.table_comma.why" => {
            "The items inside { } are separated by commas: { speed = 2, size = 4 }."
        }
        "code.explain.call_args" => "This call is missing its (",
        "code.explain.call_args.why" => {
            "After the name of a function, the values go between parentheses: print(x). An empty call is print()."
        }
        "code.explain.c_operator" => "Luau writes “{good}”, not “{bad}”",
        "code.explain.c_operator.why" => {
            "Luau uses words for logic — and, or, not — and ~= for “not equal”."
        }
        "code.explain.function_paren" => "function needs ( after the name",
        "code.explain.function_paren.why" => {
            "A function reads function name(values) … end. The name can’t be a Luau word."
        }
        "code.explain.expected_equals" => "= is missing",
        "code.explain.expected_equals.why" => {
            "To give values to several names at once, write a, b = 1, 2."
        }
        // ── Tipos (o assistente vê que o script vai falhar) ──
        "code.explain.unknown_global" => "“{name}” doesn’t exist",
        "code.explain.unknown_global.why" => {
            "Luau knows no “{name}” here. It may be misspelled (Alt+Enter suggests the closest name), or it needs local {name} = … above."
        }
        "code.explain.type_mismatch" => "This should be {expected}, but it is {got}",
        "code.explain.type_mismatch.why" => {
            "The value here doesn’t fit what this place takes. Check what you passed, or convert it (tonumber, tostring)."
        }
        "code.explain.unknown_key" => "“{table}” has no “{key}”",
        "code.explain.unknown_key.why" => {
            "Check the spelling — Alt+Enter suggests the closest name. After the dot, the list shows every name that exists."
        }
        "code.explain.unknown_key_here" => "There is no “{key}” here",
        "code.explain.unknown_key_here.why" => {
            "This table has no “{key}”. Check the spelling — Alt+Enter suggests the closest name, and after the dot the list shows every name that exists."
        }
        "code.explain.too_few_args" => {
            "Too few values for “{name}”: it takes {expects}, and gets {given}"
        }
        "code.explain.too_few_args.why" => {
            "Add the missing values between the parentheses. Typing ( shows what each one is."
        }
        "code.explain.too_many_args" => {
            "Too many values for “{name}”: it takes {expects}, and gets {given}"
        }
        "code.explain.too_many_args.why" => {
            "Remove the extra values. Typing ( shows what each one is."
        }
        "code.explain.too_few_args_anon" => {
            "Too few values: this function takes {expects}, and gets {given}"
        }
        "code.explain.too_few_args_anon.why" => "Add the missing values between the parentheses.",
        "code.explain.compare_types" => "{a} can’t be compared with {b}",
        "code.explain.compare_types.why" => {
            "These two can never be equal, so the comparison always gives the same answer. Check both sides: often a misspelled text, or a number compared with a text (tonumber, tostring)."
        }
        "code.explain.missing_return" => "Some paths of this function give back nothing",
        "code.explain.missing_return.why" => {
            "One branch ends with return and another reaches end without it. Add a return at the end."
        }
        "code.explain.call_nil_type" => "This calls something that is nil",
        "code.explain.call_nil_type.why" => {
            "The name before ( holds no function. Check the spelling, or that it was given a function above."
        }
        "code.explain.needs_colon" => "Call this with : instead of .",
        "code.explain.needs_colon.why" => {
            "A function that uses self is called with a colon: object:method()."
        }
        "code.explain.bad_cast" => "{from} can’t be turned into {to}",
        "code.explain.bad_cast.why" => {
            ":: only changes the type between types that fit each other. Convert the value instead."
        }
        // ── Avisos (o script corre, mas parece engano) ──
        "code.explain.same_line" => "Two statements on one line",
        "code.explain.same_line.why" => {
            "A new statement starts right after another. Put it on its own line, or look for a missing sign between them."
        }
        "code.explain.shadow_global" => "“{name}” hides a global with the same name",
        "code.explain.shadow_global.why" => {
            "From here on “{name}” means this local, not the global. Rename it to keep both."
        }
        "code.explain.shadow_local" => "“{name}” is declared again",
        "code.explain.shadow_local.why" => {
            "This second local {name} hides the first one. Reuse the first (without local), or pick another name."
        }
        "code.explain.local_unused" => "“{name}” is never used",
        "code.explain.local_unused.why" => "Remove it, or name it _{name} if it is on purpose.",
        "code.explain.function_unused" => "Function “{name}” is never called",
        "code.explain.function_unused.why" => "Remove it, or name it _{name} if it is on purpose.",
        "code.explain.global_as_local" => "“{name}” should be local",
        "code.explain.global_as_local.why" => {
            "Only one function uses it. Declare it with local inside that function, so nothing else changes it by accident."
        }
        "code.explain.uninitialized" => "“{name}” is read before it gets a value",
        "code.explain.uninitialized.why" => {
            "Give it a value where it is declared: local {name} = 0."
        }
        "code.explain.implicit_return" => "“{name}” sometimes gives back nothing",
        "code.explain.implicit_return.why" => {
            "Some paths reach end without return. Add a return at the end of the function."
        }
        "code.explain.unreachable" => "This code never runs",
        "code.explain.unreachable.why" => {
            "A return, break or error above it always ends the block first. Move it up, or remove it."
        }
        "code.explain.too_many_values" => "More values than names",
        "code.explain.too_many_values.why" => {
            "Left of = there are {vars}, right of it {values}: the extra values are dropped."
        }
        "code.explain.too_many_vars" => "More names than values",
        "code.explain.too_many_vars.why" => {
            "Left of = there are {vars}, right of it {values}: the extra names stay nil."
        }
        "code.explain.multiline" => "This statement goes on to the next line",
        "code.explain.multiline.why" => {
            "Indent the next line, so it reads as part of this one — or look for a missing sign."
        }
        // Um tipo enorme (a tabela inteira do `ph2d`) dito numa palavra.
        "code.explain.type_table" => "a table",
        "code.explain.type_function" => "a function",
        // ── Ao correr (o motor parou o script) ──
        "code.explain.index_nil" => "Tried to read “{key}” from nil",
        "code.explain.index_nil.why" => {
            "The value before .{key} has nothing in it: it was never set, or the name is misspelled."
        }
        "code.explain.index_nil_number" => "Tried to read an item from nil",
        "code.explain.index_nil_number.why" => {
            "The value before [ ] has nothing in it: it was never set, or the name is misspelled."
        }
        "code.explain.arith" => "Tried to do math with {what}",
        "code.explain.arith.why" => {
            "One side of + - * / is {what}, not a number. Check that it was set, or convert it with tonumber."
        }
        "code.explain.call_nil_value" => "Tried to call something that is nil",
        "code.explain.call_nil_value.why" => {
            "The name before ( holds no function: check the spelling, or that the function is defined above this line."
        }
        "code.explain.convert_argument" => "Value {n} should be a {want}, and it is {got}",
        "code.explain.convert_argument.why" => {
            "The function got the wrong kind of value in place {n}. Check what you passed there."
        }
        "code.explain.missing_argument" => "“{name}” is missing value {n}",
        "code.explain.missing_argument.why" => "“{name}” needs a {want} in place {n}.",
        "code.explain.invalid_argument" => {
            "Value {n} of “{name}” should be a {want}, and it is {got}"
        }
        "code.explain.invalid_argument.why" => "Check what you passed in place {n} of “{name}”.",
        "code.explain.table_index_nil" => "A table key is nil",
        "code.explain.table_index_nil.why" => {
            "t[key] = … needs a key, and the key here has nothing in it. Check that it was set."
        }
        "code.explain.compare" => "Can’t compare {a} with {b}",
        "code.explain.compare.why" => {
            "Both values of a comparison must be set and of the same kind. Check the one that is {b}."
        }
        "code.explain.concat" => "Tried to join {a} with {b}",
        "code.explain.concat.why" => {
            ".. joins texts and numbers only. Check that the value was set, or wrap it in tostring."
        }
        // ── As propriedades (`ph2d.property`) ──
        "code.explain.bad_default" => "Property “{name}” needs a starting value",
        "code.explain.bad_default.why" => {
            "ph2d.property(\"{name}\", value): the value is a number, true or false, a text, ph2d.vec2 or ph2d.color."
        }
        "code.explain.bad_name" => "“{name}” can’t be a property name",
        "code.explain.bad_name.why" => {
            "A property name uses letters, digits and _, and doesn’t start with a digit."
        }
        "code.explain.bad_hint" => "The min, max or step of “{name}” don’t fit",
        "code.explain.bad_hint.why" => {
            "They need a number (or ph2d.vec2) starting value, a step above 0, and min no bigger than max."
        }
        // ── As nossas (o lint do `self`, I7) ──
        "code.explain.unknown_field" => "self.{name} is never set",
        "code.explain.unknown_field.why" => {
            "Nothing in this script writes self.{name} = …, and no ph2d.property declares it, so it is always nil."
        }
        "code.explain.unknown_field_near" => "self.{name} is never set — did you mean {near}?",
        "code.explain.unknown_field_near.why" => {
            "Nothing in this script writes self.{name} = …, and no ph2d.property declares it, so it is always nil. Alt+Enter changes it to {near}."
        }
        _ => return None,
    })
}

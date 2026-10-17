
### estatico: 1095 mutantes, 54 classes, 1661 pares (classe × sítio); calados: 180

| # | classe | código | grav. | sítios | % | % acum. | 1.ª | % ⚖️ op. | exemplo cru | operadores |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `TypeError: Unknown global <x>; consider assigning to it first` | 1001 | erro | 317 | 19.1 | 19.1 | 234 | 7.6 | `TypeError: Unknown global 'p2hd'; consider assigning to it first` | identificador_trocado 265, palavra_chave_trocada 35, dois_pontos_por_ponto 6, sem_fecha_aspas 4, sem_self_no_parametro 4, sem_fecha_chave 3 |
| 2 | `SameLineStatement: A new statement is on the same line; add semi-colon on previous statement to silence` | 5 | aviso | 232 | 14.0 | 33.1 | 0 | 16.2 | `SameLineStatement: A new statement is on the same line; add semi-colon on previous statement to silence` | palavra_chave_trocada 103, atribuicao_por_igual 47, sem_fecha_parenteses 27, sem_fecha_aspas 22, igual_por_atribuicao 16, sem_end 9, sem_fecha_chave 4, virgula_em_falta 2, comparacao_invertida 1, mais_mais 1 |
| 3 | `SyntaxError: Incomplete statement: expected assignment or a function call` | 1014 | erro | 151 | 9.1 | 42.1 | 120 | 6.7 | `SyntaxError: Incomplete statement: expected assignment or a function call` | palavra_chave_trocada 115, atribuicao_por_igual 33, sem_fecha_aspas 2, mais_mais 1 |
| 4 | `SyntaxError: Expected identifier when parsing expression, got <tok>` | 1014 | erro | 112 | 6.7 | 48.9 | 23 | 12.4 | `SyntaxError: Expected identifier when parsing expression, got '=='` | atribuicao_por_igual 47, palavra_chave_trocada 39, igual_por_atribuicao 15, sem_fecha_aspas 3, sem_fecha_chave 3, virgula_em_falta 2, comparacao_invertida 1, sem_end 1, mais_mais 1 |
| 5 | `SyntaxError: Expected ')' (to close '(' at <pos>), got <tok>` | 1014 | erro | 108 | 6.5 | 55.4 | 76 | 4.3 | `SyntaxError: Expected ')' (to close '(' at line 9), got 'ph2d'` | sem_fecha_parenteses 74, sem_fecha_aspas 31, palavra_chave_trocada 2, sem_end 1 |
| 6 | `SyntaxError: Expected 'then' when parsing if statement, got <tok>` | 1014 | erro | 81 | 4.9 | 60.3 | 69 | 6.4 | `SyntaxError: Expected 'then' when parsing if statement, got '='` | palavra_chave_trocada 30, sem_then 24, igual_por_atribuicao 15, sem_fecha_aspas 12 |
| 7 | `SyntaxError: Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | 1014 | erro | 67 | 4.0 | 64.3 | 29 | 4.9 | `SyntaxError: Expected 'end' (to close 'function' at line 31), got <eof>; did you forget to close 'then' at line 49?` | sem_end 30, palavra_chave_trocada 29, else_if 7, sem_fecha_parenteses 1 |
| 8 | `LocalShadow: Variable <x> shadows a global variable used at <pos>` | 4 | aviso | 59 | 3.6 | 67.9 | 37 | 4.8 | `LocalShadow: Variable 'self' shadows a global variable used at line 25` | palavra_chave_trocada 24, identificador_trocado 22, dois_pontos_por_ponto 6, sem_self_no_parametro 4, sem_fecha_chave 3 |
| 9 | `TypeError: Expected this to be <x>, but got <tok>` | 1000 | erro | 57 | 3.4 | 71.3 | 55 | 6.8 | `TypeError: Expected this to be 'string', but got 'number'` | ponto_por_dois_pontos 39, argumentos_trocados 15, sem_fecha_parenteses 2, concat_com_mais 1 |
| 10 | `TypeError: Key <x> not found in table <x>` | 1002 | erro | 51 | 3.1 | 74.4 | 42 | 1.0 | `TypeError: Key 'proeprty' not found in table '{ attach_script: (number, string) -> (), color: (number, number, number, number?) -> Color, de…` | membro_trocado 42, palavra_chave_trocada 7, identificador_trocado 1, sem_fecha_chave 1 |
| 11 | `SyntaxError: Malformed string; did you forget to finish it?` | 1014 | erro | 49 | 3.0 | 77.3 | 49 | 2.5 | `SyntaxError: Malformed string; did you forget to finish it?` | sem_fecha_aspas 49 |
| 12 | `TypeError: Argument count mismatch. Function <x> expects <n> arguments, but only <n> are specified` | 1008 | erro | 42 | 2.5 | 79.8 | 32 | 2.2 | `TypeError: Argument count mismatch. Function 'ph2d.property' expects 2 to 3 arguments, but only 1 is specified` | argumento_a_menos 32, sem_fecha_aspas 9, sem_fecha_chave 1 |
| 13 | `TypeError: Argument count mismatch. Function <x> expects <n> arguments, but <n> are specified` | 1008 | erro | 32 | 1.9 | 81.8 | 2 | 1.9 | `TypeError: Argument count mismatch. Function 'ph2d.property' expects 2 to 3 arguments, but 4 are specified` | ponto_por_dois_pontos 31, sem_fecha_parenteses 1 |
| 14 | `LocalUnused: Variable <x> is never used; prefix with <x> to silence` | 7 | aviso | 31 | 1.9 | 83.6 | 23 | 0.3 | `LocalUnused: Variable 'ss' is never used; prefix with '_' to silence` | identificador_trocado 25, palavra_chave_trocada 6 |
| 15 | `SyntaxError: Expected <eof>, got <tok>` | 1014 | erro | 31 | 1.9 | 85.5 | 0 | 0.7 | `SyntaxError: Expected <eof>, got 'end'` | palavra_chave_trocada 30, igual_por_atribuicao 1 |
| 16 | `SyntaxError: Expected 'end' (to close <kw> at <pos>), got <tok>` | 1014 | erro | 24 | 1.4 | 86.9 | 18 | 1.2 | `SyntaxError: Expected 'end' (to close 'then' at line 32), got 'self'` | palavra_chave_trocada 15, sem_end 7, else_if 1, igual_por_atribuicao 1 |
| 17 | `SyntaxError: Expected '}' (to close '{' at <pos>), got <tok>` | 1014 | erro | 24 | 1.4 | 88.4 | 13 | 2.5 | `SyntaxError: Expected '}' (to close '{' at column 43), got malformed string` | sem_fecha_chave 15, sem_fecha_aspas 5, virgula_em_falta 4 |
| 18 | `FunctionUnused: Function <x> is never used; prefix with <x> to silence` | 8 | aviso | 22 | 1.3 | 89.7 | 21 | 0.3 | `FunctionUnused: Function 'iint' is never used; prefix with '_' to silence` | identificador_trocado 15, palavra_chave_trocada 6, argumento_a_menos 1 |
| 19 | `GlobalUsedAsLocal: Global <x> is only used in the enclosing function <x>; consider changing it to local` | 3 | aviso | 20 | 1.2 | 90.9 | 4 | 0.4 | `GlobalUsedAsLocal: Global 's' is only used in the enclosing function 'update'; consider changing it to local` | palavra_chave_trocada 15, identificador_trocado 4, sem_end 1 |
| 20 | `SyntaxError: Expected ',' after table constructor element` | 1014 | erro | 20 | 1.2 | 92.1 | 19 | 2.6 | `SyntaxError: Expected ',' after table constructor element` | virgula_em_falta 15, sem_fecha_aspas 3, sem_fecha_chave 2 |
| 21 | `UninitializedLocal: Variable <x> defined at <pos> is never initialized or assigned; initialize with 'nil' to silence` | 20 | aviso | 18 | 1.1 | 93.2 | 0 | 1.1 | `UninitializedLocal: Variable 'brilho' defined at line 44 is never initialized or assigned; initialize with 'nil' to silence` | atribuicao_por_igual 18 |
| 22 | `SyntaxError: Expected '(', '{' or <string> when parsing function call, got <tok>` | 1014 | erro | 16 | 1.0 | 94.2 | 1 | 0.3 | `SyntaxError: Expected '(', '{' or <string> when parsing function call, got ','` | palavra_chave_trocada 16 |
| 23 | `SyntaxError: Expected 'end' (to close <kw> at <pos>), got <tok>; did you forget to close <kw> at <pos>?` | 1014 | erro | 9 | 0.5 | 94.7 | 5 | 0.2 | `SyntaxError: Expected 'end' (to close 'function' at line 21), got 'elseif'; did you forget to close 'then' at line 26?` | palavra_chave_trocada 7, sem_end 2 |
| 24 | `ImplicitReturn: Function <x> can implicitly return no values even though there's an explicit return at <pos>; add explicit return to silence` | 16 | aviso | 8 | 0.5 | 95.2 | 1 | 0.2 | `ImplicitReturn: Function 'update' can implicitly return no values even though there's an explicit return at line 32; add explicit return to …` | palavra_chave_trocada 6, sem_end 2 |
| 25 | `SyntaxError: Expected 'do' when parsing for loop, got <tok>` | 1014 | erro | 8 | 0.5 | 95.7 | 8 | 2.8 | `SyntaxError: Expected 'do' when parsing for loop, got 'od'` | palavra_chave_trocada 4, sem_do 4 |
| 26 | `UnreachableCode: Unreachable code (previous statement always returns)` | 12 | aviso | 7 | 0.4 | 96.1 | 3 | 0.5 | `UnreachableCode: Unreachable code (previous statement always returns)` | palavra_chave_trocada 4, sem_end 2, else_if 1 |
| 27 | `SyntaxError: Expected ']' (to close '[' at <pos>), got <tok>` | 1014 | erro | 6 | 0.4 | 96.4 | 5 | 0.7 | `SyntaxError: Expected ']' (to close '[' at column 15), got '='` | sem_fecha_chave 6 |
| 28 | `TypeError: Expected this to be <x>, but got <tok> caused by: None of the union options are compatible. For example: Expected this to be <x>, but got <tok>` | 1000 | erro | 5 | 0.3 | 96.7 | 0 | 0.3 | `TypeError: Expected this to be 'PropertyOptions?', but got 'number' caused by: None of the union options are compatible. For example: Expect…` | ponto_por_dois_pontos 5 |
| 29 | `SyntaxError: Unexpected '&&'; did you mean 'and'?` | 1014 | erro | 4 | 0.2 | 97.0 | 4 | 1.4 | `SyntaxError: Unexpected '&&'; did you mean 'and'?` | logico_estilo_c 4 |
| 30 | `TypeError: Type <x> cannot be compared with <x>` | 1018 | erro | 4 | 0.2 | 97.2 | 2 | 0.2 | `TypeError: Type "idle" cannot be compared with "walk"` | palavra_chave_trocada 2, sem_end 2 |
| 31 | `LocalShadow: Variable <x> shadows previous declaration at <pos>` | 4 | aviso | 4 | 0.2 | 97.5 | 1 | 0.2 | `LocalShadow: Variable 'speed' shadows previous declaration at line 30` | palavra_chave_trocada 2, ponto_por_dois_pontos 1, sem_end 1 |
| 32 | `UnbalancedAssignment: Assigning <n> values to <n> variables leaves some values unused` | 15 | aviso | 4 | 0.2 | 97.7 | 0 | 0.1 | `UnbalancedAssignment: Assigning 2 values to 1 variables leaves some values unused` | palavra_chave_trocada 4 |
| 33 | `SyntaxError: Expected 'end' (to close <kw> at <pos>), got <eof>` | 1014 | erro | 4 | 0.2 | 98.0 | 2 | 0.2 | `SyntaxError: Expected 'end' (to close 'function' at line 7), got <eof>` | palavra_chave_trocada 2, sem_end 2 |
| 34 | `SyntaxError: Expected function call arguments after '('` | 1014 | erro | 3 | 0.2 | 98.1 | 0 | 0.0 | `SyntaxError: Expected function call arguments after '('` | palavra_chave_trocada 3 |
| 35 | `TypeError: Not all codepaths in this function return <x>.` | 1027 | erro | 3 | 0.2 | 98.3 | 0 | 0.0 | `TypeError: Not all codepaths in this function return 'boolean'.` | palavra_chave_trocada 3 |
| 36 | `SyntaxError: Unexpected '!'; did you mean 'not'?` | 1014 | erro | 2 | 0.1 | 98.4 | 2 | 0.7 | `SyntaxError: Unexpected '!'; did you mean 'not'?` | logico_estilo_c 2 |
| 37 | `SyntaxError: Unexpected '\|\|'; did you mean 'or'?` | 1014 | erro | 2 | 0.1 | 98.6 | 2 | 0.7 | `SyntaxError: Unexpected '\|\|'; did you mean 'or'?` | logico_estilo_c 2 |
| 38 | `TypeError: Expected this to be <x>, but got <tok> caused by: None of the union options are compatible. For example: Table type <x> not compatible with type <x> because the former is missing fields <x>, <x>, <x>, and <x>` | 1000 | erro | 2 | 0.1 | 98.7 | 0 | 0.1 | `TypeError: Expected this to be 'PropertyOptions?', but got 'Vec2' caused by: None of the union options are compatible. For example: Table ty…` | ponto_por_dois_pontos 2 |
| 39 | `UnbalancedAssignment: Assigning <n> values to <n> variables initializes extra variables with nil; add 'nil' to value list to silence` | 15 | aviso | 2 | 0.1 | 98.8 | 0 | 0.3 | `UnbalancedAssignment: Assigning 1 values to 2 variables initializes extra variables with nil; add 'nil' to value list to silence` | virgula_em_falta 2 |
| 40 | `SyntaxError: Expected '(' when parsing function, got <tok>` | 1014 | erro | 2 | 0.1 | 98.9 | 2 | 0.2 | `SyntaxError: Expected '(' when parsing function, got 'Dialogue'` | sem_fecha_chave 2 |
| 41 | `SyntaxError: Expected ')' (to close identifier at <pos>), got <tok>` | 1014 | erro | 2 | 0.1 | 99.0 | 0 | 0.2 | `SyntaxError: Expected ')' (to close identifier at column 10), got '.'` | sem_fecha_chave 2 |
| 42 | `SyntaxError: Expected identifier when parsing expression, got <eof>` | 1014 | erro | 2 | 0.1 | 99.2 | 2 | 0.2 | `SyntaxError: Expected identifier when parsing expression, got <eof>` | sem_fecha_chave 2 |
| 43 | `SyntaxError: Expected identifier when parsing variable name, got <tok>` | 1014 | erro | 2 | 0.1 | 99.3 | 2 | 0.1 | `SyntaxError: Expected identifier when parsing variable name, got 'for'` | sem_fecha_parenteses 2 |
| 44 | `SyntaxError: Expected ')' (to close '(' at <pos>), got <eof>` | 1014 | erro | 2 | 0.1 | 99.4 | 1 | 0.1 | `SyntaxError: Expected ')' (to close '(' at line 2), got <eof>` | sem_fecha_aspas 1, sem_fecha_parenteses 1 |
| 45 | `SyntaxError: Unexpected '!='; did you mean '~='?` | 1014 | erro | 1 | 0.1 | 99.5 | 1 | 2.8 | `SyntaxError: Unexpected '!='; did you mean '~='?` | diferente_estilo_c 1 |
| 46 | `MultiLineStatement: Statement spans multiple lines; use indentation to silence` | 6 | aviso | 1 | 0.1 | 99.5 | 0 | 0.1 | `MultiLineStatement: Statement spans multiple lines; use indentation to silence` | sem_end 1 |
| 47 | `SyntaxError: Expected '=' when parsing assignment, got <tok>` | 1014 | erro | 1 | 0.1 | 99.6 | 0 | 0.1 | `SyntaxError: Expected '=' when parsing assignment, got '}'` | virgula_em_falta 1 |
| 48 | `TypeError: Cannot call a value of type nil` | 1021 | erro | 1 | 0.1 | 99.6 | 0 | 0.1 | `TypeError: Cannot call a value of type nil` | virgula_em_falta 1 |
| 49 | `TypeError: This function must be called with self. Did you mean to use a colon instead of a dot?` | 1010 | erro | 1 | 0.1 | 99.7 | 1 | 0.1 | `TypeError: This function must be called with self. Did you mean to use a colon instead of a dot?` | ponto_por_dois_pontos 1 |
| 50 | `TypeError: Type string cannot be compared with true` | 1018 | erro | 1 | 0.1 | 99.8 | 1 | 0.1 | `TypeError: Type string cannot be compared with true` | sem_fecha_chave 1 |
| 51 | `TypeError: Expected this to be <x> but got <tok>` | 1000 | erro | 1 | 0.1 | 99.8 | 1 | 0.1 | `TypeError: Expected this to be '(a...) -> (b...)' but got 'typeof(coroutine)'` | ponto_por_dois_pontos 1 |
| 52 | `TypeError: Argument count mismatch. Function expects <n> arguments, but only <n> are specified` | 1008 | erro | 1 | 0.1 | 99.9 | 1 | 0.1 | `TypeError: Argument count mismatch. Function expects 3 arguments, but only 2 are specified` | argumento_a_menos 1 |
| 53 | `TypeError: Cannot cast <x> into <x> because the types are unrelated` | 1035 | erro | 1 | 0.1 | 99.9 | 1 | 0.0 | `TypeError: Cannot cast 'string' into 'number?' because the types are unrelated` | sem_fecha_parenteses 1 |
| 54 | `TypeError: Expected this to be <x>, but got <tok> caused by: Not all union options are compatible. Expected this to be <x>, but got <tok>` | 1000 | erro | 1 | 0.1 | 100.0 | 0 | 0.0 | `TypeError: Expected this to be 'string', but got 'number?' caused by: Not all union options are compatible. Expected this to be 'string', bu…` | sem_fecha_parenteses 1 |

Calados (mutantes SEM nenhuma mensagem nova), por operador: membro_trocado 108, inicializacao_esquecida 22, argumento_a_menos 21, argumentos_trocados 11, dois_pontos_por_ponto 9, indice_zero 4, identificador_trocado 3, ponto_por_dois_pontos 2

### runtime: 1095 mutantes, 47 classes, 669 pares (classe × sítio); calados: 426

| # | classe | código | grav. | sítios | % | % acum. | 1.ª | % ⚖️ op. | exemplo cru | operadores |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `Incomplete statement: expected assignment or a function call` | Syntax | erro | 140 | 20.9 | 20.9 | 140 | 12.4 | `Incomplete statement: expected assignment or a function call` | palavra_chave_trocada 110, atribuicao_por_igual 29, mais_mais 1 |
| 2 | `Expected ')' (to close '(' at <pos>), got <tok>` | Syntax | erro | 79 | 11.8 | 32.7 | 79 | 5.7 | `Expected ')' (to close '(' at column 14), got malformed string` | sem_fecha_parenteses 74, sem_fecha_aspas 4, palavra_chave_trocada 1 |
| 3 | `Expected 'then' when parsing if statement, got <tok>` | Syntax | erro | 69 | 10.3 | 43.0 | 69 | 11.6 | `Expected 'then' when parsing if statement, got '='` | palavra_chave_trocada 30, sem_then 24, igual_por_atribuicao 15 |
| 4 | `attempt to index nil with <x>` | Runtime | erro | 65 | 9.7 | 52.8 | 65 | 4.1 | `attempt to index nil with 'property'` | identificador_trocado 58, membro_trocado 5, sem_self_no_parametro 2 |
| 5 | `Malformed string; did you forget to finish it?` | Syntax | erro | 49 | 7.3 | 60.1 | 49 | 4.9 | `Malformed string; did you forget to finish it?` | sem_fecha_aspas 49 |
| 6 | `Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | Syntax | erro | 36 | 5.4 | 65.5 | 36 | 8.7 | `Expected 'end' (to close 'function' at line 31), got <eof>; did you forget to close 'then' at line 49?` | sem_end 29, else_if 7 |
| 7 | `attempt to perform arithmetic (<op>) on nil and number` | Runtime | erro | 33 | 4.9 | 70.4 | 33 | 1.6 | `attempt to perform arithmetic (add) on nil and number` | identificador_trocado 16, membro_trocado 14, inicializacao_esquecida 3 |
| 8 | `Expected identifier when parsing expression, got <tok>` | Syntax | erro | 23 | 3.4 | 73.8 | 23 | 8.4 | `Expected identifier when parsing expression, got '=='` | atribuicao_por_igual 18, sem_fecha_chave 3, comparacao_invertida 1, sem_end 1 |
| 9 | `attempt to call a nil value` | Runtime | erro | 22 | 3.3 | 77.1 | 22 | 0.8 | `attempt to call a nil value` | membro_trocado 18, palavra_chave_trocada 2, identificador_trocado 2 |
| 10 | `Expected 'end' (to close <kw> at <pos>), got <tok>` | Syntax | erro | 19 | 2.8 | 80.0 | 19 | 2.3 | `Expected 'end' (to close 'then' at line 32), got 'self'` | palavra_chave_trocada 10, sem_end 7, else_if 1, igual_por_atribuicao 1 |
| 11 | `Expected ',' after table constructor element` | Syntax | erro | 19 | 2.8 | 82.8 | 19 | 5.0 | `Expected ',' after table constructor element` | virgula_em_falta 15, sem_fecha_aspas 2, sem_fecha_chave 2 |
| 12 | `Expected '}' (to close '{' at <pos>), got <tok>` | Syntax | erro | 14 | 2.1 | 84.9 | 14 | 3.3 | `Expected '}' (to close '{' at column 33), got ')'` | sem_fecha_chave 10, virgula_em_falta 4 |
| 13 | `bad argument #<n>: error converting Lua table to String (expected string or number)` | Runtime | erro | 9 | 1.3 | 86.2 | 9 | 1.3 | `bad argument #1: error converting Lua table to String (expected string or number)` | ponto_por_dois_pontos 7, argumentos_trocados 2 |
| 14 | `bad argument #<n>: error converting Lua nil to <número> (expected number or string coercible to number)` | Runtime | erro | 8 | 1.2 | 87.4 | 8 | 0.6 | `bad argument #2: error converting Lua nil to f64 (expected number or string coercible to number)` | argumento_a_menos 4, membro_trocado 4 |
| 15 | `Expected 'do' when parsing for loop, got <tok>` | Syntax | erro | 8 | 1.2 | 88.6 | 8 | 5.6 | `Expected 'do' when parsing for loop, got 'od'` | palavra_chave_trocada 4, sem_do 4 |
| 16 | `bad argument #<n>: error converting Lua table to <número> (expected number or string coercible to number)` | Runtime | erro | 6 | 0.9 | 89.5 | 6 | 0.7 | `bad argument #1: error converting Lua table to f64 (expected number or string coercible to number)` | ponto_por_dois_pontos 6 |
| 17 | `Expected ']' (to close '[' at <pos>), got <tok>` | Syntax | erro | 6 | 0.9 | 90.4 | 6 | 1.3 | `Expected ']' (to close '[' at column 15), got '='` | sem_fecha_chave 6 |
| 18 | `Expected 'end' (to close <kw> at <pos>), got <tok>; did you forget to close <kw> at <pos>?` | Syntax | erro | 5 | 0.7 | 91.2 | 5 | 0.4 | `Expected 'end' (to close 'function' at line 69), got 'ned'; did you forget to close 'then' at line 77?` | palavra_chave_trocada 3, sem_end 2 |
| 19 | `attempt to index nil with number` | Runtime | erro | 5 | 0.7 | 91.9 | 5 | 0.1 | `attempt to index nil with number` | identificador_trocado 5 |
| 20 | `bad argument #<n>: error converting Lua string to <número> (expected number or string coercible to number)` | Runtime | erro | 4 | 0.6 | 92.5 | 4 | 0.8 | `bad argument #1: error converting Lua string to u64 (expected number or string coercible to number)` | argumentos_trocados 4 |
| 21 | `Unexpected '&&'; did you mean 'and'?` | Syntax | erro | 4 | 0.6 | 93.1 | 4 | 2.7 | `Unexpected '&&'; did you mean 'and'?` | logico_estilo_c 4 |
| 22 | `Expected '(', '{' or <string> when parsing function call, got <tok>` | Syntax | erro | 4 | 0.6 | 93.7 | 4 | 0.1 | `Expected '(', '{' or <string> when parsing function call, got ','` | palavra_chave_trocada 4 |
| 23 | `property <x> needs a number, boolean, string, ph2d.vec2 or ph2d.color default` | Decl:BadDefault | erro | 3 | 0.4 | 94.2 | 3 | 0.3 | `property 'active' needs a number, boolean, string, ph2d.vec2 or ph2d.color default` | argumento_a_menos 3 |
| 24 | `missing argument #<n> to <x> (number expected)` | Runtime | erro | 3 | 0.4 | 94.6 | 3 | 0.3 | `missing argument #1 to 'sin' (number expected)` | argumento_a_menos 3 |
| 25 | `<x> is not a valid property name` | Decl:BadName | erro | 3 | 0.4 | 95.1 | 3 | 0.6 | `'1' is not a valid property name` | argumentos_trocados 3 |
| 26 | `invalid argument #<n> to <x> (number expected, got table)` | Runtime | erro | 3 | 0.4 | 95.5 | 3 | 0.4 | `invalid argument #1 to 'sin' (number expected, got table)` | ponto_por_dois_pontos 3 |
| 27 | `table index is nil` | Runtime | erro | 3 | 0.4 | 96.0 | 3 | 0.1 | `table index is nil` | identificador_trocado 3 |
| 28 | `bad argument #<n>: error converting Lua nil to String (expected string or number)` | Runtime | erro | 2 | 0.3 | 96.3 | 2 | 0.2 | `bad argument #2: error converting Lua nil to String (expected string or number)` | argumento_a_menos 2 |
| 29 | `Unexpected '!'; did you mean 'not'?` | Syntax | erro | 2 | 0.3 | 96.6 | 2 | 1.4 | `Unexpected '!'; did you mean 'not'?` | logico_estilo_c 2 |
| 30 | `Unexpected '\|\|'; did you mean 'or'?` | Syntax | erro | 2 | 0.3 | 96.9 | 2 | 1.4 | `Unexpected '\|\|'; did you mean 'or'?` | logico_estilo_c 2 |
| 31 | `Expected '(' when parsing function, got <tok>` | Syntax | erro | 2 | 0.3 | 97.2 | 2 | 0.4 | `Expected '(' when parsing function, got 'Dialogue'` | sem_fecha_chave 2 |
| 32 | `Expected identifier when parsing expression, got <eof>` | Syntax | erro | 2 | 0.3 | 97.5 | 2 | 0.4 | `Expected identifier when parsing expression, got <eof>` | sem_fecha_chave 2 |
| 33 | `Expected identifier when parsing variable name, got <tok>` | Syntax | erro | 2 | 0.3 | 97.8 | 2 | 0.1 | `Expected identifier when parsing variable name, got 'for'` | sem_fecha_parenteses 2 |
| 34 | `Expected 'end' (to close <kw> at <pos>), got <eof>` | Syntax | erro | 2 | 0.3 | 98.1 | 2 | 0.3 | `Expected 'end' (to close 'function' at line 7), got <eof>` | sem_end 2 |
| 35 | `bad argument #<n>: error converting Lua boolean to String (expected string or number)` | Runtime | erro | 1 | 0.1 | 98.2 | 1 | 0.2 | `bad argument #1: error converting Lua boolean to String (expected string or number)` | argumentos_trocados 1 |
| 36 | `property <x>: min/max/step need a number or ph2d.vec2 default, step > <n> and min <= max` | Decl:BadHint | erro | 1 | 0.1 | 98.4 | 1 | 0.2 | `property 'sine': min/max/step need a number or ph2d.vec2 default, step > 0 and min <= max` | argumentos_trocados 1 |
| 37 | `Unexpected '!='; did you mean '~='?` | Syntax | erro | 1 | 0.1 | 98.5 | 1 | 5.5 | `Unexpected '!='; did you mean '~='?` | diferente_estilo_c 1 |
| 38 | `attempt to compare number < nil` | Runtime | erro | 1 | 0.1 | 98.7 | 1 | 0.0 | `attempt to compare number < nil` | identificador_trocado 1 |
| 39 | `Expected <eof>, got <tok>` | Syntax | erro | 1 | 0.1 | 98.8 | 1 | 0.0 | `Expected <eof>, got 'end'` | palavra_chave_trocada 1 |
| 40 | `missing argument #<n> to <x> (function expected)` | Runtime | erro | 1 | 0.1 | 99.0 | 1 | 0.1 | `missing argument #1 to 'create' (function expected)` | argumento_a_menos 1 |
| 41 | `missing argument #<n> to <x> (thread expected)` | Runtime | erro | 1 | 0.1 | 99.1 | 1 | 0.1 | `missing argument #1 to 'resume' (thread expected)` | argumento_a_menos 1 |
| 42 | `attempt to perform arithmetic (<op>) on string and number` | Runtime | erro | 1 | 0.1 | 99.3 | 1 | 5.5 | `attempt to perform arithmetic (add) on string and number` | concat_com_mais 1 |
| 43 | `attempt to concatenate string with nil` | Runtime | erro | 1 | 0.1 | 99.4 | 1 | 0.0 | `attempt to concatenate string with nil` | identificador_trocado 1 |
| 44 | `invalid argument #<n> to <x> (thread expected, got nil)` | Runtime | erro | 1 | 0.1 | 99.6 | 1 | 0.0 | `invalid argument #1 to 'resume' (thread expected, got nil)` | identificador_trocado 1 |
| 45 | `invalid argument #<n> to <x> (function expected, got table)` | Runtime | erro | 1 | 0.1 | 99.7 | 1 | 0.1 | `invalid argument #1 to 'create' (function expected, got table)` | ponto_por_dois_pontos 1 |
| 46 | `invalid argument #<n> to <x> (thread expected, got table)` | Runtime | erro | 1 | 0.1 | 99.9 | 1 | 0.1 | `invalid argument #1 to 'resume' (thread expected, got table)` | ponto_por_dois_pontos 1 |
| 47 | `Expected ')' (to close '(' at <pos>), got <eof>` | Syntax | erro | 1 | 0.1 | 100.0 | 1 | 0.1 | `Expected ')' (to close '(' at line 2), got <eof>` | sem_fecha_parenteses 1 |

Calados (mutantes SEM nenhuma mensagem nova), por operador: identificador_trocado 190, membro_trocado 109, argumento_a_menos 40, ponto_por_dois_pontos 27, inicializacao_esquecida 19, argumentos_trocados 15, dois_pontos_por_ponto 15, palavra_chave_trocada 5, indice_zero 4, sem_self_no_parametro 2

### Mapa entre as vias (1.ª mensagem estática → 1.ª do motor)

| operador | mutantes | estática (1.ª) | motor (1.ª) | n |
|---|---|---|---|---|
| igual_por_atribuicao | 16 | `1014 · SyntaxError: Expected 'then' when parsing if statement, got <tok>` | `Syntax · Expected 'then' when parsing if statement, got <tok>` | 15 |
| igual_por_atribuicao | 16 | `1014 · SyntaxError: Expected 'end' (to close <kw> at <pos>), got <tok>` | `Syntax · Expected 'end' (to close <kw> at <pos>), got <tok>` | 1 |
| atribuicao_por_igual | 47 | `1014 · SyntaxError: Incomplete statement: expected assignment or a function call` | `Syntax · Incomplete statement: expected assignment or a function call` | 29 |
| atribuicao_por_igual | 47 | `1014 · SyntaxError: Expected identifier when parsing expression, got <tok>` | `Syntax · Expected identifier when parsing expression, got <tok>` | 18 |
| diferente_estilo_c | 1 | `1014 · SyntaxError: Unexpected '!='; did you mean '~='?` | `Syntax · Unexpected '!='; did you mean '~='?` | 1 |
| concat_com_mais | 1 | `1000 · TypeError: Expected this to be <x>, but got <tok>` | `Runtime · attempt to perform arithmetic (<op>) on string and number` | 1 |
| logico_estilo_c | 8 | `1014 · SyntaxError: Unexpected '&&'; did you mean 'and'?` | `Syntax · Unexpected '&&'; did you mean 'and'?` | 4 |
| logico_estilo_c | 8 | `1014 · SyntaxError: Unexpected '!'; did you mean 'not'?` | `Syntax · Unexpected '!'; did you mean 'not'?` | 2 |
| logico_estilo_c | 8 | `1014 · SyntaxError: Unexpected '\|\|'; did you mean 'or'?` | `Syntax · Unexpected '\|\|'; did you mean 'or'?` | 2 |
| comparacao_invertida | 1 | `1014 · SyntaxError: Expected identifier when parsing expression, got <tok>` | `Syntax · Expected identifier when parsing expression, got <tok>` | 1 |
| sem_end | 41 | `1014 · SyntaxError: Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | `Syntax · Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | 23 |
| sem_end | 41 | `1014 · SyntaxError: Expected 'end' (to close <kw> at <pos>), got <tok>` | `Syntax · Expected 'end' (to close <kw> at <pos>), got <tok>` | 7 |
| sem_end | 41 | `1018 · TypeError: Type <x> cannot be compared with <x>` | `Syntax · Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | 2 |
| sem_then | 24 | `1014 · SyntaxError: Expected 'then' when parsing if statement, got <tok>` | `Syntax · Expected 'then' when parsing if statement, got <tok>` | 24 |
| sem_do | 4 | `1014 · SyntaxError: Expected 'do' when parsing for loop, got <tok>` | `Syntax · Expected 'do' when parsing for loop, got <tok>` | 4 |
| sem_fecha_parenteses | 77 | `1014 · SyntaxError: Expected ')' (to close '(' at <pos>), got <tok>` | `Syntax · Expected ')' (to close '(' at <pos>), got <tok>` | 72 |
| sem_fecha_parenteses | 77 | `1014 · SyntaxError: Expected identifier when parsing variable name, got <tok>` | `Syntax · Expected identifier when parsing variable name, got <tok>` | 2 |
| sem_fecha_parenteses | 77 | `1008 · TypeError: Argument count mismatch. Function <x> expects <n> arguments, but <n> are specified` | `Syntax · Expected ')' (to close '(' at <pos>), got <tok>` | 1 |
| sem_fecha_chave | 25 | `1014 · SyntaxError: Expected '}' (to close '{' at <pos>), got <tok>` | `Syntax · Expected '}' (to close '{' at <pos>), got <tok>` | 9 |
| sem_fecha_chave | 25 | `1014 · SyntaxError: Expected ']' (to close '[' at <pos>), got <tok>` | `Syntax · Expected ']' (to close '[' at <pos>), got <tok>` | 5 |
| sem_fecha_chave | 25 | `1014 · SyntaxError: Expected identifier when parsing expression, got <tok>` | `Syntax · Expected identifier when parsing expression, got <tok>` | 3 |
| sem_fecha_aspas | 55 | `1014 · SyntaxError: Malformed string; did you forget to finish it?` | `Syntax · Malformed string; did you forget to finish it?` | 49 |
| sem_fecha_aspas | 55 | `1014 · SyntaxError: Expected ')' (to close '(' at <pos>), got <tok>` | `Syntax · Expected ')' (to close '(' at <pos>), got <tok>` | 4 |
| sem_fecha_aspas | 55 | `1014 · SyntaxError: Expected ',' after table constructor element` | `Syntax · Expected ',' after table constructor element` | 2 |
| else_if | 8 | `1014 · SyntaxError: Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | `Syntax · Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | 6 |
| else_if | 8 | `1014 · SyntaxError: Expected 'end' (to close <kw> at <pos>), got <tok>` | `Syntax · Expected 'end' (to close <kw> at <pos>), got <tok>` | 1 |
| else_if | 8 | `12 · UnreachableCode: Unreachable code (previous statement always returns)` | `Syntax · Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | 1 |
| palavra_chave_trocada | 170 | `1014 · SyntaxError: Incomplete statement: expected assignment or a function call` | `Syntax · Incomplete statement: expected assignment or a function call` | 90 |
| palavra_chave_trocada | 170 | `1014 · SyntaxError: Expected 'then' when parsing if statement, got <tok>` | `Syntax · Expected 'then' when parsing if statement, got <tok>` | 30 |
| palavra_chave_trocada | 170 | `4 · LocalShadow: Variable <x> shadows a global variable used at <pos>` | `Syntax · Incomplete statement: expected assignment or a function call` | 15 |
| identificador_trocado | 277 | `1001 · TypeError: Unknown global <x>; consider assigning to it first` | `(nada)` | 150 |
| identificador_trocado | 277 | `1001 · TypeError: Unknown global <x>; consider assigning to it first` | `Runtime · attempt to index nil with <x>` | 54 |
| identificador_trocado | 277 | `8 · FunctionUnused: Function <x> is never used; prefix with <x> to silence` | `(nada)` | 14 |
| membro_trocado | 150 | `(nada)` | `(nada)` | 85 |
| membro_trocado | 150 | `1002 · TypeError: Key <x> not found in table <x>` | `(nada)` | 24 |
| membro_trocado | 150 | `1002 · TypeError: Key <x> not found in table <x>` | `Runtime · attempt to call a nil value` | 18 |
| dois_pontos_por_ponto | 15 | `(nada)` | `(nada)` | 9 |
| dois_pontos_por_ponto | 15 | `4 · LocalShadow: Variable <x> shadows a global variable used at <pos>` | `(nada)` | 6 |
| ponto_por_dois_pontos | 45 | `1000 · TypeError: Expected this to be <x>, but got <tok>` | `(nada)` | 22 |
| ponto_por_dois_pontos | 45 | `1000 · TypeError: Expected this to be <x>, but got <tok>` | `Runtime · bad argument #<n>: error converting Lua table to String (expected string or number)` | 7 |
| ponto_por_dois_pontos | 45 | `1000 · TypeError: Expected this to be <x>, but got <tok>` | `Runtime · bad argument #<n>: error converting Lua table to <número> (expected number or string coercible to number)` | 6 |
| sem_self_no_parametro | 4 | `1001 · TypeError: Unknown global <x>; consider assigning to it first` | `Runtime · attempt to index nil with <x>` | 1 |
| sem_self_no_parametro | 4 | `4 · LocalShadow: Variable <x> shadows a global variable used at <pos>` | `Runtime · attempt to index nil with <x>` | 1 |
| sem_self_no_parametro | 4 | `1001 · TypeError: Unknown global <x>; consider assigning to it first` | `(nada)` | 1 |
| argumento_a_menos | 54 | `(nada)` | `(nada)` | 21 |
| argumento_a_menos | 54 | `1008 · TypeError: Argument count mismatch. Function <x> expects <n> arguments, but only <n> are specified` | `(nada)` | 18 |
| argumento_a_menos | 54 | `1008 · TypeError: Argument count mismatch. Function <x> expects <n> arguments, but only <n> are specified` | `Runtime · bad argument #<n>: error converting Lua nil to <número> (expected number or string coercible to number)` | 4 |
| argumentos_trocados | 26 | `(nada)` | `(nada)` | 9 |
| argumentos_trocados | 26 | `1000 · TypeError: Expected this to be <x>, but got <tok>` | `(nada)` | 6 |
| argumentos_trocados | 26 | `1000 · TypeError: Expected this to be <x>, but got <tok>` | `Runtime · bad argument #<n>: error converting Lua string to <número> (expected number or string coercible to number)` | 4 |
| indice_zero | 4 | `(nada)` | `(nada)` | 4 |
| virgula_em_falta | 19 | `1014 · SyntaxError: Expected ',' after table constructor element` | `Syntax · Expected ',' after table constructor element` | 15 |
| virgula_em_falta | 19 | `1014 · SyntaxError: Expected '}' (to close '{' at <pos>), got <tok>` | `Syntax · Expected '}' (to close '{' at <pos>), got <tok>` | 4 |
| inicializacao_esquecida | 22 | `(nada)` | `(nada)` | 19 |
| inicializacao_esquecida | 22 | `(nada)` | `Runtime · attempt to perform arithmetic (<op>) on nil and number` | 3 |
| mais_mais | 1 | `1014 · SyntaxError: Incomplete statement: expected assignment or a function call` | `Syntax · Incomplete statement: expected assignment or a function call` | 1 |

### O que os ORIGINAIS já dizem (subtraído de cada mutante)

- estático · `coro_timing` linha 7: 8 `FunctionUnused: Function 'task_body' is never used; prefix with '_' to silence`
- estático · `gc_stress` linha 21: 8 `FunctionUnused: Function 'tick_alloc' is never used; prefix with '_' to silence`

### Sítios por operador × ficheiro

| operador | bob | c7_typical_script | case_2_coroutine | coro_timing | gc_stress | hello | ph2d_script_attach | script_boot | total |
|---|---|---|---|---|---|---|---|---|---|
| igual_por_atribuicao | 2 | 11 | 0 | 0 | 0 | 1 | 2 | 0 | 16 |
| atribuicao_por_igual | 12 | 25 | 0 | 0 | 6 | 1 | 3 | 0 | 47 |
| diferente_estilo_c | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| concat_com_mais | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 |
| logico_estilo_c | 6 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 8 |
| comparacao_invertida | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| sem_end | 5 | 20 | 2 | 1 | 5 | 2 | 6 | 0 | 41 |
| sem_then | 4 | 16 | 0 | 0 | 0 | 1 | 3 | 0 | 24 |
| sem_do | 0 | 0 | 1 | 0 | 3 | 0 | 0 | 0 | 4 |
| sem_fecha_parenteses | 18 | 33 | 2 | 2 | 7 | 3 | 11 | 1 | 77 |
| sem_fecha_chave | 5 | 11 | 0 | 0 | 8 | 0 | 1 | 0 | 25 |
| sem_fecha_aspas | 19 | 22 | 0 | 0 | 2 | 2 | 9 | 1 | 55 |
| else_if | 1 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 8 |
| palavra_chave_trocada | 27 | 87 | 7 | 2 | 17 | 6 | 24 | 0 | 170 |
| identificador_trocado | 65 | 135 | 4 | 4 | 26 | 10 | 32 | 1 | 277 |
| membro_trocado | 50 | 84 | 1 | 1 | 5 | 1 | 8 | 0 | 150 |
| dois_pontos_por_ponto | 0 | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 15 |
| ponto_por_dois_pontos | 16 | 13 | 1 | 1 | 5 | 1 | 8 | 0 | 45 |
| sem_self_no_parametro | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 4 |
| argumento_a_menos | 16 | 19 | 1 | 1 | 5 | 3 | 8 | 1 | 54 |
| argumentos_trocados | 12 | 8 | 0 | 0 | 0 | 1 | 5 | 0 | 26 |
| chamada_sem_parenteses | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| indice_zero | 0 | 0 | 1 | 0 | 3 | 0 | 0 | 0 | 4 |
| virgula_em_falta | 5 | 5 | 0 | 0 | 7 | 0 | 2 | 0 | 19 |
| inicializacao_esquecida | 6 | 16 | 0 | 0 | 0 | 0 | 0 | 0 | 22 |
| mais_mais | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 |

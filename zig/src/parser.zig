const std = @import("std");
const Allocator = std.mem.Allocator;
const Token = @import("token.zig").Token;
const Lexer = @import("lexer.zig").Lexer;
const ast = @import("ast.zig");

pub const ParseError = error{
    UnexpectedToken,
    NotANumber,
    NotAnExpression,
} || Allocator.Error;

pub fn parse(input: []const u8, allocator: Allocator) ParseError!ast.Program {
    var lex = Lexer.init(input);
    var parser = Parser.init(&lex, allocator);

    return parser.parseProgram();
}

const Precedence = enum(u8) {
    lowest,
    equals,
    less_greater,
    sum,
    product,
    prefix,
    call,

    fn fromToken(token: Token) Precedence {
        return switch (token) {
            .eq, .not_eq => .equals,
            .lt, .gt => .less_greater,
            .plus, .minus => .sum,
            .slash, .asterisk => .product,
            .lparen => .call,
            else => .lowest,
        };
    }

    fn isLower(self: Precedence, other: Precedence) bool {
        return @intFromEnum(self) < @intFromEnum(other);
    }
};

const Parser = struct {
    const Self = @This();

    lexer: *Lexer,
    current_token: Token,
    peek_token: Token,

    allocator: Allocator,

    fn init(lexer: *Lexer, allocator: Allocator) Self {
        return Self{
            .lexer = lexer,
            .current_token = lexer.nextToken(),
            .peek_token = lexer.nextToken(),
            .allocator = allocator,
        };
    }

    fn step(self: *Self) void {
        self.current_token = self.peek_token;
        self.peek_token = self.lexer.nextToken();
    }

    fn peekPrecedence(self: *Self) Precedence {
        return Precedence.fromToken(self.peek_token);
    }

    fn parseProgram(self: *Self) ParseError!ast.Program {
        var program = ast.Program{
            .statements = std.ArrayList(ast.Statement).init(self.allocator),
        };
        errdefer program.deinit(self.allocator);

        while (self.current_token != .eof) {
            const stmt = try self.parseStatement();
            try program.statements.append(stmt);

            self.step();
        }

        return program;
    }

    fn parseStatement(self: *Self) ParseError!ast.Statement {
        return switch (self.current_token) {
            .let => self.parseLetStatement(),
            .return_token => self.parseReturnStatement(),
            else => return self.parseExpressionStatement(),
        };
    }

    fn parseLetStatement(self: *Self) ParseError!ast.Statement {
        const name_ref = switch (self.peek_token) {
            .ident => |name| name,
            else => return ParseError.UnexpectedToken,
        };
        self.step();

        if (self.peek_token != .assign) {
            return ParseError.UnexpectedToken;
        }
        self.step();
        self.step();

        const value = try self.parseExpression(.lowest);

        if (self.peek_token == .semicolon) {
            self.step();
        }

        const name = try self.allocator.alloc(u8, name_ref.len);
        @memcpy(name, name_ref);

        return .{
            .let_stmt = .{
                .name = name,
                .value = value,
            },
        };
    }

    fn parseReturnStatement(self: *Self) ParseError!ast.Statement {
        self.step();

        const value = try self.parseExpression(.lowest);

        if (self.peek_token == .semicolon) {
            self.step();
        }

        return .{
            .return_stmt = .{
                .value = value,
            },
        };
    }

    fn parseExpressionStatement(self: *Self) ParseError!ast.Statement {
        const exp = try self.parseExpression(.lowest);

        if (self.peek_token == .semicolon) {
            self.step();
        }

        return .{
            .expression_stmt = exp,
        };
    }

    fn parseBlockStatement(self: *Self) ParseError!ast.BlockStatement {
        self.step();

        var statements = std.ArrayList(ast.Statement).init(self.allocator);
        errdefer {
            for (statements.items) |stmt| {
                stmt.deinit(self.allocator);
            }
            statements.deinit();
        }

        while (self.current_token != .rsquigly and self.current_token != .eof) {
            const stmt = try self.parseStatement();
            try statements.append(stmt);

            self.step();
        }

        return .{
            .statements = statements,
        };
    }

    fn parseExpression(self: *Self, precedence: Precedence) ParseError!ast.Expression {
        var left = try self.parsePrefix(precedence);

        while (self.peek_token != .semicolon and precedence.isLower(self.peekPrecedence())) {
            if (!self.peek_token.isInfix()) {
                return left;
            }

            self.step();
            left = try self.parseInfix(left);
        }

        return left;
    }

    fn parsePrefix(self: *Self, _: Precedence) ParseError!ast.Expression {
        switch (self.current_token) {
            .int => |value| {
                const number = std.fmt.parseInt(i64, value, 0) catch return ParseError.NotANumber;
                return .{ .integer_literal = number };
            },
            .ident => |name_ref| {
                const name = try self.allocator.alloc(u8, name_ref.len);
                @memcpy(name, name_ref);
                return .{ .identifier = name };
            },
            .true_token => return .{ .boolean_literal = true },
            .false_token => return .{ .boolean_literal = false },
            .bang, .minus => {
                const operator_kind = try prefixOperatorKindFromToken(self.current_token);
                self.step();

                const right = try self.parseExpression(.prefix);
                const exp = ast.Expression{
                    .prefix_operator = .{
                        .operator = operator_kind,
                        .right = try self.allocator.create(ast.Expression),
                    },
                };
                exp.prefix_operator.right.* = right;

                return exp;
            },
            .lparen => return self.parseGrouped(),
            .if_token => return self.parseIfExpression(),
            .function => return self.parseFunctionLiteral(),
            else => return ParseError.NotAnExpression,
        }
    }

    fn parseInfix(self: *Self, left: ast.Expression) ParseError!ast.Expression {
        switch (self.current_token) {
            .plus, .minus, .slash, .asterisk, .eq, .not_eq, .lt, .gt => {
                const operator = try infixOperatorKindFromToken(self.current_token);
                const precedence = Precedence.fromToken(self.current_token);

                self.step();

                const right = try self.parseExpression(precedence);

                const exp = ast.Expression{
                    .infix_operator = .{
                        .operator = operator,
                        .left = try self.allocator.create(ast.Expression),
                        .right = try self.allocator.create(ast.Expression),
                    },
                };
                exp.infix_operator.left.* = left;
                exp.infix_operator.right.* = right;
                return exp;
            },
            .lparen => {
                const arguments = try self.parseExpressionList(.rparen);

                const exp = ast.Expression{ .function_call = .{
                    .function = try self.allocator.create(ast.Expression),
                    .arguments = arguments,
                } };
                exp.function_call.function.* = left;

                return exp;
            },
            else => return ParseError.UnexpectedToken,
        }
    }

    fn parseExpressionList(self: *Self, end: Token) ParseError!std.ArrayList(ast.Expression) {
        var list = std.ArrayList(ast.Expression).init(self.allocator);
        errdefer {
            for (list.items) |exp| {
                exp.deinit(self.allocator);
            }
            list.deinit();
        }

        self.step();
        if (@intFromEnum(self.current_token) == @intFromEnum(end)) {
            return list;
        }

        try list.append(try self.parseExpression(.lowest));

        while (self.peek_token == .comma) {
            self.step();
            self.step();
            try list.append(try self.parseExpression(.lowest));
        }

        if (@intFromEnum(self.peek_token) != @intFromEnum(end)) {
            return ParseError.UnexpectedToken;
        }
        self.step();

        return list;
    }

    fn parseGrouped(self: *Self) ParseError!ast.Expression {
        self.step();

        const exp = try self.parseExpression(.lowest);

        if (self.peek_token != .rparen) {
            return ParseError.UnexpectedToken;
        }

        self.step();
        return exp;
    }

    fn parseIfExpression(self: *Self) ParseError!ast.Expression {
        // Parse condition
        if (self.peek_token != .lparen) {
            return ParseError.UnexpectedToken;
        }
        self.step();
        self.step();

        const condition = try self.parseExpression(.lowest);

        if (self.peek_token != .rparen) {
            return ParseError.UnexpectedToken;
        }
        self.step();

        // Parse consequence
        if (self.peek_token != .lsquigly) {
            return ParseError.UnexpectedToken;
        }
        self.step();

        const consequence = try self.parseBlockStatement();

        if (self.current_token != .rsquigly) {
            return ParseError.UnexpectedToken;
        }

        // Parse alternative
        var alternative: ast.BlockStatement = undefined;
        if (self.peek_token == .else_token) {
            self.step();

            if (self.peek_token != .lsquigly) {
                return ParseError.UnexpectedToken;
            }
            self.step();

            alternative = try self.parseBlockStatement();

            if (self.current_token != .rsquigly) {
                return ParseError.UnexpectedToken;
            }
        } else {
            alternative = ast.BlockStatement{
                .statements = std.ArrayList(ast.Statement).init(self.allocator),
            };
        }

        const if_expr = ast.IfExpression{
            .condition = try self.allocator.create(ast.Expression),
            .consequence = consequence,
            .alternative = alternative,
        };
        if_expr.condition.* = condition;

        return ast.Expression{ .if_expression = if_expr };
    }

    fn parseFunctionParameters(self: *Self) ParseError!std.ArrayList([]const u8) {
        var parameters = std.ArrayList([]const u8).init(self.allocator);
        errdefer {
            for (parameters.items) |param| {
                self.allocator.free(param);
            }
            parameters.deinit();
        }

        self.step();
        if (self.current_token == .rparen) {
            return parameters;
        }

        switch (self.current_token) {
            .ident => |name_ref| {
                const name = try self.allocator.alloc(u8, name_ref.len);
                @memcpy(name, name_ref);
                try parameters.append(name);
            },
            else => return ParseError.UnexpectedToken,
        }

        while (self.peek_token == .comma) {
            self.step();
            self.step();

            switch (self.current_token) {
                .ident => |name_ref| {
                    const name = try self.allocator.alloc(u8, name_ref.len);
                    @memcpy(name, name_ref);
                    try parameters.append(name);
                },
                else => return ParseError.UnexpectedToken,
            }
        }

        if (self.peek_token != .rparen) {
            return ParseError.UnexpectedToken;
        }
        self.step();

        return parameters;
    }

    fn parseFunctionLiteral(self: *Self) ParseError!ast.Expression {
        if (self.peek_token != .lparen) {
            return ParseError.UnexpectedToken;
        }
        self.step();

        const parameters = try self.parseFunctionParameters();
        errdefer {
            for (parameters.items) |param| {
                self.allocator.free(param);
            }
            parameters.deinit();
        }

        if (self.peek_token != .lsquigly) {
            return ParseError.UnexpectedToken;
        }
        self.step();

        const body = try self.parseBlockStatement();

        return ast.Expression{ .function_literal = .{
            .parameters = parameters,
            .body = body,
        } };
    }
};

fn prefixOperatorKindFromToken(token: Token) ParseError!ast.PrefixOperatorKind {
    return switch (token) {
        .bang => .not,
        .minus => .negative,
        else => return ParseError.UnexpectedToken,
    };
}

fn infixOperatorKindFromToken(token: Token) ParseError!ast.InfixOperatorKind {
    return switch (token) {
        .plus => .add,
        .minus => .subtract,
        .asterisk => .multiply,
        .slash => .divide,
        .eq => .equal,
        .not_eq => .not_equal,
        .lt => .less_than,
        .gt => .greater_than,
        else => ParseError.UnexpectedToken,
    };
}

test "Parser: let statements" {
    const testing = std.testing;

    const Test = struct {
        input: []const u8,
        expected_identifier: []const u8,
        expected_value: ast.Expression,
    };

    const input = [_]Test{ .{
        .input = "let x = 5;",
        .expected_identifier = "x",
        .expected_value = .{ .integer_literal = 5 },
    }, .{
        .input = "let y = 10;",
        .expected_identifier = "y",
        .expected_value = .{ .integer_literal = 10 },
    }, .{
        .input = "let foobar = y",
        .expected_identifier = "foobar",
        .expected_value = .{ .identifier = "y" },
    } };

    for (input) |t| {
        const program = try parse(t.input, testing.allocator);
        defer program.deinit(testing.allocator);

        try testing.expectEqual(1, program.statements.items.len);
        try testing.expectEqualDeep(ast.Statement{ .let_stmt = .{
            .name = t.expected_identifier,
            .value = t.expected_value,
        } }, program.statements.items[0]);
    }
}

test "Parser: invalid let statement" {
    const Test = struct {
        input: []const u8,
        expected_error: ParseError,
    };

    const inputs = [_]Test{
        .{
            .input = "let x 5;",
            .expected_error = ParseError.UnexpectedToken,
        },
        .{
            .input = "let = 10;",
            .expected_error = ParseError.UnexpectedToken,
        },
        .{
            .input = "let 838383;",
            .expected_error = ParseError.UnexpectedToken,
        },
    };

    for (inputs) |input| {
        const program = parse(input.input, std.testing.allocator);
        try std.testing.expectError(input.expected_error, program);
    }
}

test "Parser: identifier expression" {
    const testing = std.testing;
    const program = try parse("foobar;", testing.allocator);
    defer program.deinit(testing.allocator);

    try testing.expectEqual(1, program.statements.items.len);
    try testing.expectEqualDeep(ast.Statement{ .expression_stmt = .{
        .identifier = "foobar",
    } }, program.statements.items[0]);
}

test "Parser: integer literal expression" {
    const testing = std.testing;
    const program = try parse("5;", testing.allocator);
    defer program.deinit(testing.allocator);

    try testing.expectEqual(1, program.statements.items.len);
    try testing.expectEqualDeep(ast.Statement{ .expression_stmt = .{
        .integer_literal = 5,
    } }, program.statements.items[0]);
}

test "Parser: boolean (true) literal expression" {
    const testing = std.testing;
    const program = try parse("true;", testing.allocator);
    defer program.deinit(testing.allocator);

    try testing.expectEqual(1, program.statements.items.len);
    try testing.expectEqualDeep(ast.Statement{ .expression_stmt = .{
        .boolean_literal = true,
    } }, program.statements.items[0]);
}

test "Parser: boolean (false) literal expression" {
    const testing = std.testing;
    const program = try parse("false;", testing.allocator);
    defer program.deinit(testing.allocator);

    try testing.expectEqual(1, program.statements.items.len);
    try testing.expectEqualDeep(ast.Statement{ .expression_stmt = .{
        .boolean_literal = false,
    } }, program.statements.items[0]);
}

test "Parser: prefix expressions" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected_operator: ast.PrefixOperatorKind,
        expected_right: ast.Expression,
    };

    const tests = [_]Test{
        .{
            .input = "!5;",
            .expected_operator = .not,
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "-15;",
            .expected_operator = .negative,
            .expected_right = .{ .integer_literal = 15 },
        },
        .{
            .input = "!true",
            .expected_operator = .not,
            .expected_right = .{ .boolean_literal = true },
        },

        .{
            .input = "!false",
            .expected_operator = .not,
            .expected_right = .{ .boolean_literal = false },
        },
    };

    for (tests) |tt| {
        const program = try parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var exp = tt.expected_right;

        try t.expectEqual(1, program.statements.items.len);
        try t.expectEqualDeep(ast.Statement{ .expression_stmt = .{
            .prefix_operator = .{
                .operator = tt.expected_operator,
                .right = &exp,
            },
        } }, program.statements.items[0]);
    }
}

test "Parser: infix expressions" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected_operator: ast.InfixOperatorKind,
        expected_left: ast.Expression,
        expected_right: ast.Expression,
    };

    const tests = [_]Test{
        .{
            .input = "5 + 5;",
            .expected_operator = .add,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "5 - 5;",
            .expected_operator = .subtract,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "5 * 5;",
            .expected_operator = .multiply,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "5 / 5;",
            .expected_operator = .divide,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "5 > 5;",
            .expected_operator = .greater_than,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "5 < 5;",
            .expected_operator = .less_than,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "5 == 5;",
            .expected_operator = .equal,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
        .{
            .input = "5 != 5;",
            .expected_operator = .not_equal,
            .expected_left = .{ .integer_literal = 5 },
            .expected_right = .{ .integer_literal = 5 },
        },
    };

    for (tests) |tt| {
        const program = try parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var left = tt.expected_left;
        var right = tt.expected_right;

        try t.expectEqual(1, program.statements.items.len);
        try t.expectEqualDeep(ast.Statement{
            .expression_stmt = .{
                .infix_operator = .{
                    .operator = tt.expected_operator,
                    .left = &left,
                    .right = &right,
                },
            },
        }, program.statements.items[0]);
    }
}

test "Parser: if expression" {
    const t = std.testing;
    const input = "if (x < y) { x }";

    const program = try parse(input, t.allocator);
    defer program.deinit(t.allocator);

    try t.expectEqual(1, program.statements.items.len);

    var left = ast.Expression{ .identifier = "x" };
    var right = ast.Expression{ .identifier = "y" };
    var condition = ast.Expression{ .infix_operator = .{
        .operator = .less_than,
        .left = &left,
        .right = &right,
    } };

    var cons_statements = std.ArrayList(ast.Statement).init(t.allocator);
    defer cons_statements.deinit();
    try cons_statements.append(.{ .expression_stmt = .{ .identifier = "x" } });

    try t.expectEqualDeep(ast.Statement{ .expression_stmt = .{
        .if_expression = .{
            .condition = &condition,
            .consequence = .{
                .statements = cons_statements,
            },
            .alternative = .{
                .statements = std.ArrayList(ast.Statement).init(t.allocator),
            },
        },
    } }, program.statements.items[0]);
}

test "Parser: if else expressions" {
    const t = std.testing;
    const input = "if (x < y) { x } else {y}";

    const program = try parse(input, t.allocator);
    defer program.deinit(t.allocator);

    try t.expectEqual(1, program.statements.items.len);

    var left = ast.Expression{ .identifier = "x" };
    var right = ast.Expression{ .identifier = "y" };
    var condition = ast.Expression{ .infix_operator = .{
        .operator = .less_than,
        .left = &left,
        .right = &right,
    } };

    var cons_statements = std.ArrayList(ast.Statement).init(t.allocator);
    defer cons_statements.deinit();
    try cons_statements.append(.{ .expression_stmt = .{ .identifier = "x" } });

    var alt_statements = std.ArrayList(ast.Statement).init(t.allocator);
    defer alt_statements.deinit();
    try alt_statements.append(.{ .expression_stmt = .{ .identifier = "y" } });

    try t.expectEqualDeep(ast.Statement{ .expression_stmt = .{
        .if_expression = .{
            .condition = &condition,
            .consequence = .{
                .statements = cons_statements,
            },
            .alternative = .{
                .statements = alt_statements,
            },
        },
    } }, program.statements.items[0]);
}

test "Parser: function literal" {
    const t = std.testing;
    const input = "fn(x, y) { x + y; }";

    const program = try parse(input, t.allocator);
    defer program.deinit(t.allocator);

    try t.expectEqual(1, program.statements.items.len);
    const if_expr = program.statements.items[0].expression_stmt.function_literal;

    try t.expectEqual(2, if_expr.parameters.items.len);
    try t.expectEqualStrings("x", if_expr.parameters.items[0]);
    try t.expectEqualStrings("y", if_expr.parameters.items[1]);

    try t.expectEqual(1, if_expr.body.statements.items.len);

    var left = ast.Expression{ .identifier = "x" };
    var right = ast.Expression{ .identifier = "y" };
    try t.expectEqualDeep(ast.Statement{ .expression_stmt = .{
        .infix_operator = ast.InfixOperator{
            .operator = .add,
            .left = &left,
            .right = &right,
        },
    } }, if_expr.body.statements.items[0]);
}

test "Parser: function parameters" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected_parameters: []const []const u8,
    };

    const tests = [_]Test{ .{
        .input = "fn() {}",
        .expected_parameters = &[_][]const u8{},
    }, .{
        .input = "fn(x) {}",
        .expected_parameters = &[_][]const u8{"x"},
    }, .{
        .input = "fn(x, y, z) {}",
        .expected_parameters = &[_][]const u8{ "x", "y", "z" },
    } };

    for (tests) |tt| {
        const program = try parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        try t.expectEqual(1, program.statements.items.len);
        const if_expr = program.statements.items[0].expression_stmt.function_literal;

        try t.expectEqual(tt.expected_parameters.len, if_expr.parameters.items.len);
        for (tt.expected_parameters, if_expr.parameters.items) |expected, got| {
            try t.expectEqualStrings(expected, got);
        }
    }
}

test "Parser: function call expression" {
    const t = std.testing;
    const input = "add(1, 2*3, 4 + 5);";

    const program = try parse(input, t.allocator);
    defer program.deinit(t.allocator);

    try t.expectEqual(1, program.statements.items.len);
    const fn_call = program.statements.items[0].expression_stmt.function_call;

    try t.expectEqualStrings("add", fn_call.function.identifier);

    try t.expectEqual(3, fn_call.arguments.items.len);

    var debug_str = std.ArrayList(u8).init(t.allocator);
    defer debug_str.deinit();

    try fn_call.arguments.items[0].string(debug_str.writer());
    try t.expectEqualStrings("1", debug_str.items);

    debug_str.clearAndFree();
    try fn_call.arguments.items[1].string(debug_str.writer());
    try t.expectEqualStrings("(2 * 3)", debug_str.items);

    debug_str.clearAndFree();
    try fn_call.arguments.items[2].string(debug_str.writer());
    try t.expectEqualStrings("(4 + 5)", debug_str.items);
}

test "Parser: function call arguments" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected_arguments: []const []const u8,
    };

    const tests = [_]Test{
        .{
            .input = "add();",
            .expected_arguments = &[_][]const u8{},
        },
        .{
            .input = "add(1);",
            .expected_arguments = &[_][]const u8{"1"},
        },
        .{
            .input = "add(1, 2 * 3, 4 + 5);",
            .expected_arguments = &[_][]const u8{ "1", "(2 * 3)", "(4 + 5)" },
        },
    };

    for (tests) |tt| {
        const program = try parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        try t.expectEqual(1, program.statements.items.len);
        const fn_call = program.statements.items[0].expression_stmt.function_call;

        try t.expectEqual(tt.expected_arguments.len, fn_call.arguments.items.len);
        for (tt.expected_arguments, fn_call.arguments.items) |expected, got| {
            var debug_str = std.ArrayList(u8).init(t.allocator);
            defer debug_str.deinit();

            try got.string(debug_str.writer());
            try t.expectEqualStrings(expected, debug_str.items);
        }
    }
}

test "Parser: operator precedence" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected: []const u8,
    };

    const tests = [_]Test{
        .{ .input = "-a * b", .expected = "((-a) * b);" },
        .{ .input = "!-a", .expected = "(!(-a));" },
        .{ .input = "a + b + c", .expected = "((a + b) + c);" },
        .{ .input = "a + b - c", .expected = "((a + b) - c);" },
        .{ .input = "a * b * c", .expected = "((a * b) * c);" },
        .{ .input = "a * b / c", .expected = "((a * b) / c);" },
        .{ .input = "a + b / c", .expected = "(a + (b / c));" },
        .{ .input = "a + b * c + d / e -f", .expected = "(((a + (b * c)) + (d / e)) - f);" },
        .{ .input = "3 + 4; -5 * 5", .expected = "(3 + 4);((-5) * 5);" },
        .{ .input = "5 > 4 == 3 < 4", .expected = "((5 > 4) == (3 < 4));" },
        .{ .input = "5 < 4 != 3 > 4", .expected = "((5 < 4) != (3 > 4));" },
        .{
            .input = "3 + 4 * 5 == 3 * 1 + 4 * 5;",
            .expected = "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)));",
        },
        .{ .input = "true", .expected = "true;" },
        .{ .input = "false", .expected = "false;" },
        .{ .input = "3 > 5 == false", .expected = "((3 > 5) == false);" },
        .{ .input = "3 < 5 == true", .expected = "((3 < 5) == true);" },
        .{ .input = "1 + (2 + 3) + 4", .expected = "((1 + (2 + 3)) + 4);" },
        .{ .input = "(5 + 5) * 2", .expected = "((5 + 5) * 2);" },
        .{ .input = "2 / (5 + 5)", .expected = "(2 / (5 + 5));" },
        .{ .input = "-(5 + 5)", .expected = "(-(5 + 5));" },
        .{ .input = "!(true == true)", .expected = "(!(true == true));" },
        .{ .input = "a + add(b*c) + d", .expected = "((a + add((b * c))) + d);" },
        .{
            .input = "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
            .expected = "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)));",
        },
        .{
            .input = "add(a + b + c * d / f + g)",
            .expected = "add((((a + b) + ((c * d) / f)) + g));",
        },
    };

    for (tests) |tt| {
        const program = try parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var out = std.ArrayList(u8).init(t.allocator);
        defer out.deinit();

        try program.string(out.writer());
        try t.expectEqualStrings(tt.expected, out.items);
    }
}

test "Parser: cleanup on error" {
    const t = std.testing;

    const res = parse("fn(a, b, c){let b c}", t.allocator);
    try t.expectError(ParseError.UnexpectedToken, res);
}

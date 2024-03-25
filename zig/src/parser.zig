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

        while (self.current_token != .eof) {
            const stmt = try self.parseStatement(.lowest);
            try program.statements.append(stmt);

            self.step();
        }

        return program;
    }

    fn parseStatement(self: *Self, precedence: Precedence) ParseError!ast.Statement {
        return switch (self.current_token) {
            .let => self.parseLetStatement(precedence),
            .return_token => self.parseReturnStatement(precedence),
            else => return self.parseExpressionStatement(precedence),
        };
    }

    fn parseLetStatement(self: *Self, precedence: Precedence) ParseError!ast.Statement {
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

        const value = try self.parseExpression(precedence);

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

    fn parseReturnStatement(self: *Self, precedence: Precedence) ParseError!ast.Statement {
        self.step();

        const value = try self.parseExpression(precedence);

        if (self.peek_token == .semicolon) {
            self.step();
        }

        return .{
            .return_stmt = .{
                .value = value,
            },
        };
    }

    fn parseExpressionStatement(self: *Self, precedence: Precedence) ParseError!ast.Statement {
        const exp = try self.parseExpression(precedence);

        if (self.peek_token == .semicolon) {
            self.step();
        }

        return .{
            .expression_stmt = exp,
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
            else => return ParseError.UnexpectedToken,
        }
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
        // .{ .input = "a + add(b*c) + d", .expected = "((a + add((b * c))) + d)" },
        // .{
        //     .input = "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
        //     .expected = "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
        // },
        // .{
        //     .input = "add(a + b + c * d / f + g)",
        //     .expected = "add((((a + b) + ((c * d) / f)) + g))",
        // },
        // .{
        //     .input = "a * [1, 2, 3, 4][b * c] * d",
        //     .expected = "((a * ([1, 2, 3, 4][(b * c)])) * d)",
        // },
        // .{
        //     .input = "add(a * b[2], b[1], 2 * [1, 2][1])",
        //     .expected = "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
        // },
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

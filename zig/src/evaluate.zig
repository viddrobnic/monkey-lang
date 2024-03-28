const std = @import("std");
const Allocator = std.mem.Allocator;
const ast = @import("ast.zig");
const env = @import("environment.zig");
const parser = @import("parser.zig");
const Rc = @import("rc.zig").Rc;
const obj = @import("object.zig");

pub const EvaluateError = error{ UnknownOperator, TypeMismatch } || Allocator.Error;

pub fn evaluate(program: ast.Program, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    var result = try Rc(obj.Object).init(allocator, .null_obj);
    errdefer obj.releaseObject(&result, allocator);

    for (program.statements.items) |statement| {
        const new_result = try evaluateStatement(statement, environment, allocator);
        obj.releaseObject(&result, allocator);
        result = new_result;

        switch (result.value.*) {
            .return_obj => |*inner| {
                const inner_res = inner.retain();
                obj.releaseObject(&result, allocator);
                return inner_res;
            },
            else => {},
        }
    }

    return result;
}

fn evaluateStatement(statement: ast.Statement, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    return switch (statement) {
        .let_stmt => |stmt| try evaluateLetStatement(stmt, environment, allocator),
        .return_stmt => |stmt| blk: {
            var value = try evaluateExpression(stmt.value, environment, allocator);
            errdefer obj.releaseObject(&value, allocator);

            const object = obj.Object{ .return_obj = value };
            break :blk try Rc(obj.Object).init(allocator, object);
        },
        .expression_stmt => |stmt| try evaluateExpression(stmt, environment, allocator),
    };
}

fn evaluateLetStatement(statement: ast.LetStatement, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    const value = try evaluateExpression(statement.value, environment, allocator);
    const name = statement.name;
    try environment.value.put(name, value, allocator);

    return try Rc(obj.Object).init(allocator, .null_obj);
}

fn evaluateExpression(expression: ast.Expression, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    switch (expression) {
        .identifier => |name| {
            var res = environment.value.get(name);
            if (res == null) {
                return try Rc(obj.Object).init(allocator, .null_obj);
            } else {
                return res.?.retain();
            }
        },
        .integer_literal => |value| return try Rc(obj.Object).init(allocator, obj.Object{ .integer = value }),
        .boolean_literal => |value| return try Rc(obj.Object).init(allocator, obj.Object{ .boolean = value }),
        .prefix_operator => |operator| return evaluatePrefixExpression(operator, environment, allocator),
        .infix_operator => |operator| return evaluateInfixExpression(operator, environment, allocator),
        .if_expression => |if_expr| return evaluateIfExpression(if_expr, environment, allocator),
        else => unreachable,
    }
}

fn evaluatePrefixExpression(operator: ast.PrefixOperator, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    var right = try evaluateExpression(operator.right.*, environment, allocator);
    defer obj.releaseObject(&right, allocator);

    switch (operator.operator) {
        .not => {
            switch (right.value.*) {
                .boolean => |value| {
                    return try Rc(obj.Object).init(allocator, obj.Object{ .boolean = !value });
                },
                .null_obj => {
                    return try Rc(obj.Object).init(allocator, obj.Object{ .boolean = true });
                },
                else => {
                    return try Rc(obj.Object).init(allocator, obj.Object{ .boolean = false });
                },
            }
        },
        .negative => {
            switch (right.value.*) {
                .integer => |value| {
                    return try Rc(obj.Object).init(allocator, obj.Object{ .integer = -value });
                },
                else => {
                    return EvaluateError.UnknownOperator;
                },
            }
        },
    }
}

fn evaluateInfixExpression(operator: ast.InfixOperator, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    var left = try evaluateExpression(operator.left.*, environment, allocator);
    defer obj.releaseObject(&left, allocator);

    var right = try evaluateExpression(operator.right.*, environment, allocator);
    defer obj.releaseObject(&right, allocator);

    if (@as(obj.ObjectTag, left.value.*) == obj.ObjectTag.integer and @as(obj.ObjectTag, right.value.*) == obj.ObjectTag.integer) {
        const res: obj.Object = switch (operator.operator) {
            .add => .{ .integer = left.value.integer + right.value.integer },
            .subtract => .{ .integer = left.value.integer - right.value.integer },
            .multiply => .{ .integer = left.value.integer * right.value.integer },
            .divide => .{ .integer = @divTrunc(left.value.integer, right.value.integer) },
            .equal => .{ .boolean = left.value.integer == right.value.integer },
            .not_equal => .{ .boolean = left.value.integer != right.value.integer },
            .less_than => .{ .boolean = left.value.integer < right.value.integer },
            .greater_than => .{ .boolean = left.value.integer > right.value.integer },
        };
        return try Rc(obj.Object).init(allocator, res);
    }

    if (@as(obj.ObjectTag, left.value.*) == obj.ObjectTag.boolean and @as(obj.ObjectTag, right.value.*) == obj.ObjectTag.boolean) {
        const res: obj.Object = switch (operator.operator) {
            .equal => .{ .boolean = left.value.boolean == right.value.boolean },
            .not_equal => .{ .boolean = left.value.boolean != right.value.boolean },
            else => return EvaluateError.UnknownOperator,
        };

        return try Rc(obj.Object).init(allocator, res);
    }

    if (@intFromEnum(left.value.*) != @intFromEnum(right.value.*)) {
        return EvaluateError.TypeMismatch;
    }

    return EvaluateError.UnknownOperator;
}

fn evaluateIfExpression(if_expr: ast.IfExpression, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    var condition = try evaluateExpression(if_expr.condition.*, environment, allocator);
    defer obj.releaseObject(&condition, allocator);

    const is_truthy = switch (condition.value.*) {
        .boolean => condition.value.boolean,
        .null_obj => false,
        else => true,
    };

    if (is_truthy) {
        return evaluateBlockStatement(if_expr.consequence, environment, allocator);
    } else {
        return evaluateBlockStatement(if_expr.alternative, environment, allocator);
    }
}

fn evaluateBlockStatement(block: ast.BlockStatement, environment: Rc(env.Environment), allocator: Allocator) EvaluateError!Rc(obj.Object) {
    var result = try Rc(obj.Object).init(allocator, .null_obj);
    errdefer obj.releaseObject(&result, allocator);

    for (block.statements.items) |statement| {
        const new_result = try evaluateStatement(statement, environment, allocator);
        obj.releaseObject(&result, allocator);
        result = new_result;

        switch (result.value.*) {
            .return_obj => return result,
            else => {},
        }
    }

    return result;
}

test "Eval: integer" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected: i64,
    };
    const tests = [_]Test{
        .{
            .input = "5",
            .expected = 5,
        },
        .{
            .input = "10",
            .expected = 10,
        },
        .{
            .input = "-5",
            .expected = -5,
        },
        .{
            .input = "-10",
            .expected = -10,
        },
        .{
            .input = "5 + 5 + 5 + 5 - 10",
            .expected = 10,
        },
        .{
            .input = "2 * 2 * 2 * 2 * 2",
            .expected = 32,
        },
        .{
            .input = "-50 + 100 + -50",
            .expected = 0,
        },
        .{
            .input = "5 * 2 + 10",
            .expected = 20,
        },
        .{
            .input = "5 + 2 * 10",
            .expected = 25,
        },
        .{
            .input = "20 + 2 * -10",
            .expected = 0,
        },
        .{
            .input = "50 / 2 * 2 + 10",
            .expected = 60,
        },
        .{
            .input = "2 * (5 + 10)",
            .expected = 30,
        },
        .{
            .input = "3 * 3 * 3 + 10",
            .expected = 37,
        },
        .{
            .input = "3 * (3 * 3) + 10",
            .expected = 37,
        },
        .{
            .input = "(5 + 10 * 2 + 15 / 3) * 2 + -10",
            .expected = 50,
        },
    };

    for (tests) |tt| {
        const program = try parser.parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var environment = try env.Environment.init(t.allocator);
        defer env.releaseEnvironment(&environment, t.allocator);

        const res = try evaluate(program, environment, t.allocator);
        defer res.release();

        try t.expectEqual(obj.Object{ .integer = tt.expected }, res.value.*);
    }
}

test "Eval: bool" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected: bool,
    };
    const tests = [_]Test{
        .{
            .input = "true",
            .expected = true,
        },
        .{
            .input = "false",
            .expected = false,
        },
        .{
            .input = "1 < 2",
            .expected = true,
        },
        .{
            .input = "1 > 2",
            .expected = false,
        },
        .{
            .input = "1 < 1",
            .expected = false,
        },
        .{
            .input = "1 > 1",
            .expected = false,
        },
        .{
            .input = "1 == 1",
            .expected = true,
        },
        .{
            .input = "1 != 1",
            .expected = false,
        },
        .{
            .input = "1 == 2",
            .expected = false,
        },
        .{
            .input = "1 != 2",
            .expected = true,
        },
        .{
            .input = "true == true",
            .expected = true,
        },
        .{
            .input = "false == false",
            .expected = true,
        },
        .{
            .input = "true == false",
            .expected = false,
        },
        .{
            .input = "true != false",
            .expected = true,
        },
        .{
            .input = "false != true",
            .expected = true,
        },
        .{
            .input = "1 < 2 == true",
            .expected = true,
        },
        .{
            .input = "1 < 2 == false",
            .expected = false,
        },
        .{
            .input = "1 > 2 == true",
            .expected = false,
        },
        .{
            .input = "1 > 2 == false",
            .expected = true,
        },
    };

    for (tests) |tt| {
        const program = try parser.parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var environment = try env.Environment.init(t.allocator);
        defer env.releaseEnvironment(&environment, t.allocator);

        const res = try evaluate(program, environment, t.allocator);
        defer res.release();

        try t.expectEqual(obj.Object{ .boolean = tt.expected }, res.value.*);
    }
}

test "Eval: bang operator" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected: bool,
    };
    const tests = [_]Test{
        .{
            .input = "!true",
            .expected = false,
        },
        .{
            .input = "!false",
            .expected = true,
        },
        .{
            .input = "!5",
            .expected = false,
        },
        .{
            .input = "!!true",
            .expected = true,
        },
        .{
            .input = "!!false",
            .expected = false,
        },
        .{
            .input = "!!5",
            .expected = true,
        },
    };

    for (tests) |tt| {
        const program = try parser.parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var environment = try env.Environment.init(t.allocator);
        defer env.releaseEnvironment(&environment, t.allocator);

        const res = try evaluate(program, environment, t.allocator);
        defer res.release();

        try t.expectEqual(obj.Object{ .boolean = tt.expected }, res.value.*);
    }
}

test "Eval: if else expression" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected: obj.Object,
    };
    const tests = [_]Test{
        .{
            .input = "if (true) { 10 }",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "if (false) { 10 }",
            .expected = .null_obj,
        },
        .{
            .input = "if (1) { 10 }",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "if (1 < 2) { 10 }",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "if (1 > 2) { 10 }",
            .expected = .null_obj,
        },
        .{
            .input = "if (1 > 2) { 10 } else { 20 }",
            .expected = obj.Object{ .integer = 20 },
        },
        .{
            .input = "if (1 < 2) { 10 } else { 20 }",
            .expected = obj.Object{ .integer = 10 },
        },
    };

    for (tests) |tt| {
        const program = try parser.parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var environment = try env.Environment.init(t.allocator);
        defer env.releaseEnvironment(&environment, t.allocator);

        var res = try evaluate(program, environment, t.allocator);
        defer obj.releaseObject(&res, t.allocator);

        try t.expectEqual(tt.expected, res.value.*);
    }
}

test "Eval: return" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected: obj.Object,
    };
    const tests = [_]Test{
        .{
            .input = "return 10;",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "return 10; 9;",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "return 2 * 5; 9;",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "9; return 2 * 5; 9;",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
            .expected = obj.Object{ .integer = 10 },
        },
    };

    for (tests) |tt| {
        const program = try parser.parse(tt.input, t.allocator);
        defer program.deinit(t.allocator);

        var environment = try env.Environment.init(t.allocator);
        defer env.releaseEnvironment(&environment, t.allocator);

        var res = try evaluate(program, environment, t.allocator);
        defer obj.releaseObject(&res, t.allocator);

        try t.expectEqual(tt.expected, res.value.*);
    }
}

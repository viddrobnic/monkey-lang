const std = @import("std");
const Allocator = std.mem.Allocator;

const ast = @import("ast.zig");
const env = @import("environment.zig");
const Environment = env.Environment;
const parser = @import("parser.zig");
const obj = @import("object.zig");
const Object = obj.Object;

pub const EvaluateError = error{
    UnknownOperator,
    TypeMismatch,
    NotAFunction,
    FunctionArgumentMismatch,
} || Allocator.Error;

const AllocatedObjectType = enum {
    environment,
    object,
    function_object,
};

pub const Evaluator = struct {
    const Self = @This();

    allocator: Allocator,
    allocatedObjects: std.AutoHashMap(usize, AllocatedObjectType),
    environment: *env.Environment,

    pub fn init(allocator: Allocator) Allocator.Error!Self {
        var eval = Self{
            .allocator = allocator,
            .allocatedObjects = std.AutoHashMap(usize, AllocatedObjectType).init(allocator),
            .environment = try allocator.create(Environment),
        };
        errdefer allocator.destroy(eval.environment);

        eval.environment.* = Environment.init(allocator, null);
        try eval.allocatedObjects.put(@intFromPtr(eval.environment), .environment);

        return eval;
    }

    pub fn evaluate(self: *Self, program: ast.Program) EvaluateError!obj.Object {
        var result: Object = .null_obj;

        for (program.statements.items) |statement| {
            result = try self.evaluateStatement(statement, self.environment);

            switch (result) {
                .return_obj => |inner| {
                    const res = inner.*;

                    try self.collectGarbage();
                    return res;
                },
                else => {},
            }
        }

        try self.collectGarbage();
        return result;
    }

    pub fn deinit(self: *Self) void {
        var iterator = self.allocatedObjects.iterator();
        while (iterator.next()) |item| {
            deallocateObject(self.allocator, item);
        }

        self.allocatedObjects.deinit();
    }

    fn collectGarbage(self: *Self) Allocator.Error!void {
        var accessible_objects = std.AutoHashMap(usize, void).init(self.allocator);
        defer accessible_objects.deinit();

        try getAccessibleObjectsFromEnv(self.environment, &accessible_objects);

        var to_remove = try std.ArrayList(usize).initCapacity(self.allocator, self.allocatedObjects.count());
        defer to_remove.deinit();

        var iterator = self.allocatedObjects.iterator();
        while (iterator.next()) |item| {
            if (accessible_objects.get(item.key_ptr.*) != null) {
                continue;
            }

            deallocateObject(self.allocator, item);

            try to_remove.append(item.key_ptr.*);
        }

        for (to_remove.items) |key| {
            _ = self.allocatedObjects.remove(key);
        }
    }

    fn allocateObject(self: *Self) Allocator.Error!*Object {
        const object = try self.allocator.create(Object);
        errdefer self.allocator.destroy(object);

        try self.allocatedObjects.put(@intFromPtr(object), .object);

        return object;
    }

    fn allocateFunctionObject(self: *Self) Allocator.Error!*obj.FunctionObject {
        const fn_obj = try self.allocator.create(obj.FunctionObject);
        errdefer self.allocator.destroy(fn_obj);

        try self.allocatedObjects.put(@intFromPtr(fn_obj), .function_object);

        return fn_obj;
    }

    fn allocateEnvironment(self: *Self) Allocator.Error!*Environment {
        const environment = try self.allocator.create(Environment);
        errdefer self.allocator.destroy(environment);

        try self.allocatedObjects.put(@intFromPtr(environment), .environment);

        return environment;
    }

    fn evaluateStatement(self: *Self, statement: ast.Statement, environment: *Environment) EvaluateError!Object {
        return switch (statement) {
            .let_stmt => |stmt| try self.evaluateLetStatement(stmt, environment),
            .return_stmt => |stmt| blk: {
                const inner_obj = try self.allocateObject();
                inner_obj.* = try self.evaluateExpression(stmt.value, environment);

                break :blk Object{ .return_obj = inner_obj };
            },
            .expression_stmt => |stmt| try self.evaluateExpression(stmt, environment),
        };
    }

    fn evaluateLetStatement(self: *Self, statement: ast.LetStatement, environment: *Environment) EvaluateError!Object {
        const value = try self.evaluateExpression(statement.value, environment);
        try environment.put(statement.name, value);

        return .null_obj;
    }

    fn evaluateExpression(self: *Self, expression: ast.Expression, environment: *Environment) EvaluateError!Object {
        switch (expression) {
            .identifier => |ident| {
                const res = environment.get(ident.name);
                if (res) |object| {
                    return object;
                } else {
                    return .null_obj;
                }
            },
            .integer_literal => |value| return Object{ .integer = value },
            .boolean_literal => |value| return Object{ .boolean = value },
            .prefix_operator => |operator| return self.evaluatePrefixExpression(operator, environment),
            .infix_operator => |operator| return self.evaluateInfixExpression(operator, environment),
            .if_expression => |if_expr| return self.evaluateIfExpression(if_expr, environment),
            .function_literal => |function| {
                const fn_obj = try self.allocateFunctionObject();
                fn_obj.* = try obj.FunctionObject.init(
                    self.allocator,
                    function.parameters.items,
                    function.body,
                    environment,
                );

                return Object{ .function_obj = fn_obj };
            },
            .function_call => |fn_call| return self.evaluateFunctionCall(fn_call, environment),
        }
    }

    fn evaluatePrefixExpression(self: *Self, operator: ast.PrefixOperator, environment: *Environment) EvaluateError!Object {
        const right = try self.evaluateExpression(operator.right.*, environment);

        switch (operator.operator) {
            .not => {
                switch (right) {
                    .boolean => |value| {
                        return Object{ .boolean = !value };
                    },
                    .null_obj => {
                        return Object{ .boolean = true };
                    },
                    else => {
                        return Object{ .boolean = false };
                    },
                }
            },
            .negative => {
                switch (right) {
                    .integer => |value| {
                        return Object{ .integer = -value };
                    },
                    else => {
                        return EvaluateError.UnknownOperator;
                    },
                }
            },
        }
    }

    fn evaluateInfixExpression(self: *Self, operator: ast.InfixOperator, environment: *Environment) EvaluateError!Object {
        const left = try self.evaluateExpression(operator.left.*, environment);
        const right = try self.evaluateExpression(operator.right.*, environment);

        if (@as(obj.ObjectTag, left) == obj.ObjectTag.integer and @as(obj.ObjectTag, right) == obj.ObjectTag.integer) {
            return switch (operator.operator) {
                .add => .{ .integer = left.integer + right.integer },
                .subtract => .{ .integer = left.integer - right.integer },
                .multiply => .{ .integer = left.integer * right.integer },
                .divide => .{ .integer = @divTrunc(left.integer, right.integer) },
                .equal => .{ .boolean = left.integer == right.integer },
                .not_equal => .{ .boolean = left.integer != right.integer },
                .less_than => .{ .boolean = left.integer < right.integer },
                .greater_than => .{ .boolean = left.integer > right.integer },
            };
        }

        if (@as(obj.ObjectTag, left) == obj.ObjectTag.boolean and @as(obj.ObjectTag, right) == obj.ObjectTag.boolean) {
            return switch (operator.operator) {
                .equal => .{ .boolean = left.boolean == right.boolean },
                .not_equal => .{ .boolean = left.boolean != right.boolean },
                else => return EvaluateError.UnknownOperator,
            };
        }

        if (@intFromEnum(left) != @intFromEnum(right)) {
            return EvaluateError.TypeMismatch;
        }

        return EvaluateError.UnknownOperator;
    }

    fn evaluateIfExpression(self: *Self, if_expr: ast.IfExpression, environment: *Environment) EvaluateError!Object {
        const condition = try self.evaluateExpression(if_expr.condition.*, environment);

        const is_truthy = switch (condition) {
            .boolean => condition.boolean,
            .null_obj => false,
            else => true,
        };

        if (is_truthy) {
            return self.evaluateBlockStatement(if_expr.consequence, environment);
        } else {
            return self.evaluateBlockStatement(if_expr.alternative, environment);
        }
    }

    fn evaluateBlockStatement(self: *Self, block: ast.BlockStatement, environment: *Environment) EvaluateError!Object {
        var result: Object = .null_obj;

        for (block.statements.items) |statement| {
            result = try self.evaluateStatement(statement, environment);

            switch (result) {
                .return_obj => return result,
                else => {},
            }
        }

        return result;
    }

    fn evaluateFunctionCall(self: *Self, fn_call: ast.FunctionCall, environment: *Environment) EvaluateError!Object {
        const function = try self.evaluateExpression(fn_call.function.*, environment);
        switch (function) {
            .function_obj => |fn_obj| {
                var extended_env = try self.allocateEnvironment();
                extended_env.* = Environment.init(self.allocator, fn_obj.environment);

                if (fn_call.arguments.items.len != fn_obj.parameters.items.len) {
                    return EvaluateError.FunctionArgumentMismatch;
                }

                for (fn_call.arguments.items, fn_obj.parameters.items) |arg, param| {
                    const value = try self.evaluateExpression(arg, environment);
                    try extended_env.put(param, value);
                }

                const evaluated = try self.evaluateBlockStatement(fn_obj.body, extended_env);
                switch (evaluated) {
                    .return_obj => |inner| return inner.*,
                    else => return evaluated,
                }
            },
            else => return EvaluateError.NotAFunction,
        }
    }
};

fn deallocateObject(allocator: Allocator, item: std.AutoHashMap(usize, AllocatedObjectType).Entry) void {
    switch (item.value_ptr.*) {
        .environment => {
            var environment: *Environment = @ptrFromInt(item.key_ptr.*);

            environment.deinit();
            allocator.destroy(environment);
        },
        .object => {
            const object: *Object = @ptrFromInt(item.key_ptr.*);
            allocator.destroy(object);
        },
        .function_object => {
            const fn_obj: *obj.FunctionObject = @ptrFromInt(item.key_ptr.*);
            fn_obj.deinit();

            allocator.destroy(fn_obj);
        },
    }
}

fn getAccessibleObjectsFromEnv(environment: *Environment, accessible_objects: *std.AutoHashMap(usize, void)) Allocator.Error!void {
    if (accessible_objects.get(@intFromPtr(environment)) != null) {
        return;
    }

    try accessible_objects.put(@intFromPtr(environment), {});

    if (environment.outer) |outer| {
        try getAccessibleObjectsFromEnv(outer, accessible_objects);
    }

    var iterator = environment.store.valueIterator();
    while (iterator.next()) |item| {
        try getAccessibleObjectsFromObj(item.*, accessible_objects);
    }
}

fn getAccessibleObjectsFromObj(object: Object, accessible_objects: *std.AutoHashMap(usize, void)) Allocator.Error!void {
    switch (object) {
        .return_obj => |inner| {
            try accessible_objects.put(@intFromPtr(inner), {});
            try getAccessibleObjectsFromObj(inner.*, accessible_objects);
        },
        .function_obj => |fn_obj| {
            try accessible_objects.put(@intFromPtr(fn_obj), {});
            try getAccessibleObjectsFromEnv(fn_obj.environment, accessible_objects);
        },
        else => {},
    }
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
        defer program.deinit();

        var evaluator = try Evaluator.init(t.allocator);
        defer evaluator.deinit();

        const res = try evaluator.evaluate(program);

        try t.expectEqual(obj.Object{ .integer = tt.expected }, res);
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
        defer program.deinit();

        var evaluator = try Evaluator.init(t.allocator);
        defer evaluator.deinit();

        const res = try evaluator.evaluate(program);

        try t.expectEqual(obj.Object{ .boolean = tt.expected }, res);
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
        defer program.deinit();

        var evaluator = try Evaluator.init(t.allocator);
        defer evaluator.deinit();

        const res = try evaluator.evaluate(program);

        try t.expectEqual(obj.Object{ .boolean = tt.expected }, res);
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
        defer program.deinit();

        var evaluator = try Evaluator.init(t.allocator);
        defer evaluator.deinit();

        const res = try evaluator.evaluate(program);
        try t.expectEqual(tt.expected, res);
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
        defer program.deinit();

        var evaluator = try Evaluator.init(t.allocator);
        defer evaluator.deinit();

        const res = try evaluator.evaluate(program);
        try t.expectEqual(tt.expected, res);
    }
}

test "Eval: let statements" {
    const t = std.testing;
    const Test = struct {
        input: []const u8,
        expected: obj.Object,
    };
    const tests = [_]Test{
        .{
            .input = "let a = 5; a;",
            .expected = obj.Object{ .integer = 5 },
        },
        .{
            .input = "let a = 5 * 5; a;",
            .expected = obj.Object{ .integer = 25 },
        },
        .{
            .input = "let a = 5; let b = a; b;",
            .expected = obj.Object{ .integer = 5 },
        },
        .{
            .input = "let a = 5; let b = a; let c = a + b + 5; c;",
            .expected = obj.Object{ .integer = 15 },
        },
    };

    for (tests) |tt| {
        const program = try parser.parse(tt.input, t.allocator);
        defer program.deinit();

        var evaluator = try Evaluator.init(t.allocator);
        defer evaluator.deinit();

        const res = try evaluator.evaluate(program);
        try t.expectEqual(tt.expected, res);
    }
}

test "Eval: function application" {
    const t = std.testing;

    const Test = struct {
        input: []const u8,
        expected: obj.Object,
    };
    const tests = [_]Test{
        .{
            .input = "let identity = fn(x) { x; }; identity(5);",
            .expected = obj.Object{ .integer = 5 },
        },
        .{
            .input = "let identity = fn(x) { return x; }; identity(10);",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "let double = fn(x) { x * 2; }; double(5);",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "let add = fn(x, y) { x + y; }; add(5, 5);",
            .expected = obj.Object{ .integer = 10 },
        },
        .{
            .input = "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
            .expected = obj.Object{ .integer = 20 },
        },
        .{
            .input = "fn(x) { x; }(5);",
            .expected = obj.Object{ .integer = 5 },
        },
    };

    for (tests) |tt| {
        const program = try parser.parse(tt.input, t.allocator);
        defer program.deinit();

        var evaluator = try Evaluator.init(t.allocator);
        defer evaluator.deinit();

        const res = try evaluator.evaluate(program);
        try t.expectEqual(tt.expected, res);
    }
}

test "Eval: closures" {
    const t = std.testing;

    const input =
        \\ let newAdder = fn(x) {
        \\   fn(y) { x + y; };
        \\ };
        \\
        \\ let addTwo = newAdder(2);
        \\ addTwo(3)
    ;

    const program = try parser.parse(input, t.allocator);
    defer program.deinit();

    var evaluator = try Evaluator.init(t.allocator);
    defer evaluator.deinit();

    const res = try evaluator.evaluate(program);
    try t.expectEqual(obj.Object{ .integer = 5 }, res);
}

test "Eval: recursion" {
    const t = std.testing;

    const input =
        \\ let fibonacci = fn(x) {
        \\   if (x < 3) {
        \\     return 1;
        \\   } else {
        \\     return fibonacci(x - 1) + fibonacci(x - 2);
        \\   }
        \\ };
        \\
        \\ fibonacci(5);
    ;

    const program = try parser.parse(input, t.allocator);
    defer program.deinit();

    var evaluator = try Evaluator.init(t.allocator);
    defer evaluator.deinit();

    const res = try evaluator.evaluate(program);
    try t.expectEqual(obj.Object{ .integer = 5 }, res);
}

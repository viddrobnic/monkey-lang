const std = @import("std");
const Allocator = std.mem.Allocator;

pub const Program = struct {
    const Self = @This();

    statements: std.ArrayList(Statement),
    allocator: Allocator,

    pub fn init(allocator: Allocator) Self {
        return Self{
            .statements = std.ArrayList(Statement).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        for (self.statements.items) |item| {
            try item.string(writer);
        }
    }

    pub fn deinit(self: Self) void {
        for (self.statements.items) |item| {
            item.deinit();
        }

        self.statements.deinit();
    }
};

pub const Statement = union(enum) {
    const Self = @This();

    let_stmt: LetStatement,
    return_stmt: ReturnStatement,
    expression_stmt: Expression,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        return switch (self) {
            .let_stmt => self.let_stmt.string(writer),
            .return_stmt => self.return_stmt.string(writer),
            .expression_stmt => {
                try self.expression_stmt.string(writer);
                try writer.writeAll(";");
            },
        };
    }

    pub fn deinit(self: Self) void {
        switch (self) {
            .let_stmt => self.let_stmt.deinit(),
            .return_stmt => self.return_stmt.deinit(),
            .expression_stmt => self.expression_stmt.deinit(),
        }
    }

    pub fn clone(self: Self) Allocator.Error!Statement {
        return switch (self) {
            .let_stmt => |stmt| Self{ .let_stmt = try stmt.clone() },
            .return_stmt => |stmt| Self{ .return_stmt = try stmt.clone() },
            .expression_stmt => |stmt| Self{ .expression_stmt = try stmt.clone() },
        };
    }
};

pub const LetStatement = struct {
    const Self = @This();

    allocator: Allocator,

    name: []const u8,
    value: Expression,

    pub fn init(allocator: Allocator, name: []const u8, value: Expression) Allocator.Error!Self {
        const name_copy = try allocator.alloc(u8, name.len);
        @memcpy(name_copy, name);

        return Self{
            .allocator = allocator,
            .name = name_copy,
            .value = value,
        };
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try std.fmt.format(writer, "let {s} = ", .{self.name});
        try self.value.string(writer);
        try writer.writeAll(";");
    }

    pub fn deinit(self: Self) void {
        self.allocator.free(self.name);
        self.value.deinit();
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        const name = try self.allocator.alloc(u8, self.name.len);
        errdefer self.allocator.free(name);

        @memcpy(name, self.name);

        return Self{
            .allocator = self.allocator,
            .name = name,
            .value = try self.value.clone(),
        };
    }
};

pub const ReturnStatement = struct {
    const Self = @This();

    allocator: Allocator,

    value: Expression,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("return ");
        try self.value.string(writer);
        try writer.writeAll(";");
    }

    pub fn deinit(self: Self) void {
        self.value.deinit();
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        return Self{
            .allocator = self.allocator,
            .value = try self.value.clone(),
        };
    }
};

pub const BlockStatement = struct {
    const Self = @This();

    allocator: Allocator,

    statements: std.ArrayList(Statement),

    pub fn init(allocator: Allocator) Self {
        return Self{
            .allocator = allocator,
            .statements = std.ArrayList(Statement).init(allocator),
        };
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        for (self.statements.items) |item| {
            try item.string(writer);
        }
    }

    pub fn deinit(self: Self) void {
        for (self.statements.items) |item| {
            item.deinit();
        }
        self.statements.deinit();
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        var res = Self{
            .allocator = self.allocator,
            .statements = try std.ArrayList(Statement).initCapacity(
                self.allocator,
                self.statements.items.len,
            ),
        };
        errdefer res.deinit();

        for (self.statements.items) |stmt| {
            try res.statements.append(try stmt.clone());
        }

        return res;
    }
};

pub const Expression = union(enum) {
    const Self = @This();

    identifier: Identifier,
    integer_literal: i64,
    boolean_literal: bool,
    prefix_operator: PrefixOperator,
    infix_operator: InfixOperator,
    if_expression: IfExpression,
    function_literal: FunctionLiteral,
    function_call: FunctionCall,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        return switch (self) {
            .identifier => writer.writeAll(self.identifier.name),
            .integer_literal => writer.print("{d}", .{self.integer_literal}),
            .boolean_literal => writer.print("{any}", .{self.boolean_literal}),
            .prefix_operator => self.prefix_operator.string(writer),
            .infix_operator => self.infix_operator.string(writer),
            .if_expression => self.if_expression.string(writer),
            .function_literal => self.function_literal.string(writer),
            .function_call => self.function_call.string(writer),
        };
    }

    pub fn deinit(self: Self) void {
        switch (self) {
            .identifier => self.identifier.deinit(),
            .prefix_operator => self.prefix_operator.deinit(),
            .infix_operator => self.infix_operator.deinit(),
            .if_expression => self.if_expression.deinit(),
            .function_literal => self.function_literal.deinit(),
            .function_call => self.function_call.deinit(),
            else => {},
        }
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        return switch (self) {
            .identifier => |val| .{ .identifier = try val.clone() },
            .integer_literal => |val| .{ .integer_literal = val },
            .boolean_literal => |val| .{ .boolean_literal = val },
            .prefix_operator => |val| .{ .prefix_operator = try val.clone() },
            .infix_operator => |val| .{ .infix_operator = try val.clone() },
            .if_expression => |val| .{ .if_expression = try val.clone() },
            .function_literal => |val| .{ .function_literal = try val.clone() },
            .function_call => |val| .{ .function_call = try val.clone() },
        };
    }
};

pub const Identifier = struct {
    const Self = @This();

    allocator: Allocator,
    name: []const u8,

    /// Initializes identifier. Name is copied.
    pub fn init(allocator: Allocator, name: []const u8) Allocator.Error!Self {
        const new_name = try allocator.alloc(u8, name.len);
        @memcpy(new_name, name);

        return Self{
            .allocator = allocator,
            .name = new_name,
        };
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        return Self.init(self.allocator, self.name);
    }

    pub fn deinit(self: Self) void {
        self.allocator.free(self.name);
    }
};

pub const PrefixOperatorKind = enum {
    not,
    negative,
};

pub const PrefixOperator = struct {
    const Self = @This();

    allocator: Allocator,

    operator: PrefixOperatorKind,
    right: *Expression,

    pub fn init(allocator: Allocator, operator: PrefixOperatorKind, right: Expression) Allocator.Error!Self {
        const expression = try allocator.create(Expression);
        expression.* = right;

        return Self{
            .allocator = allocator,
            .operator = operator,
            .right = expression,
        };
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("(");

        switch (self.operator) {
            .not => try writer.writeAll("!"),
            .negative => try writer.writeAll("-"),
        }

        try self.right.string(writer);
        try writer.writeAll(")");
    }

    pub fn deinit(self: Self) void {
        self.right.deinit();
        self.allocator.destroy(self.right);
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        const right = try self.allocator.create(Expression);
        errdefer self.allocator.destroy(right);

        right.* = try self.right.clone();

        return Self{
            .allocator = self.allocator,
            .operator = self.operator,
            .right = right,
        };
    }
};

pub const InfixOperatorKind = enum {
    add,
    subtract,
    multiply,
    divide,
    equal,
    not_equal,
    less_than,
    greater_than,
};

pub const InfixOperator = struct {
    const Self = @This();

    allocator: Allocator,

    operator: InfixOperatorKind,
    left: *Expression,
    right: *Expression,

    pub fn init(allocator: Allocator, operator: InfixOperatorKind, left: Expression, right: Expression) Allocator.Error!Self {
        const left_ptr = try allocator.create(Expression);
        errdefer allocator.destroy(left_ptr);

        const right_ptr = try allocator.create(Expression);

        left_ptr.* = left;
        right_ptr.* = right;

        return Self{
            .allocator = allocator,
            .operator = operator,
            .left = left_ptr,
            .right = right_ptr,
        };
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("(");
        try self.left.string(writer);

        switch (self.operator) {
            .add => try writer.writeAll(" + "),
            .subtract => try writer.writeAll(" - "),
            .multiply => try writer.writeAll(" * "),
            .divide => try writer.writeAll(" / "),
            .equal => try writer.writeAll(" == "),
            .not_equal => try writer.writeAll(" != "),
            .less_than => try writer.writeAll(" < "),
            .greater_than => try writer.writeAll(" > "),
        }

        try self.right.string(writer);
        try writer.writeAll(")");
    }

    pub fn deinit(self: Self) void {
        self.left.deinit();
        self.allocator.destroy(self.left);

        self.right.deinit();
        self.allocator.destroy(self.right);
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        const left = try self.allocator.create(Expression);
        errdefer self.allocator.destroy(left);

        left.* = try self.left.clone();
        errdefer left.deinit();

        const right = try self.allocator.create(Expression);
        errdefer self.allocator.destroy(right);

        right.* = try self.right.clone();

        return Self{
            .allocator = self.allocator,
            .operator = self.operator,
            .left = left,
            .right = right,
        };
    }
};

pub const IfExpression = struct {
    const Self = @This();

    allocator: Allocator,

    condition: *Expression,
    consequence: BlockStatement,
    alternative: BlockStatement,

    pub fn init(allocator: Allocator, condition: Expression, consequence: BlockStatement, alternative: BlockStatement) Allocator.Error!Self {
        const cond_ptr = try allocator.create(Expression);
        cond_ptr.* = condition;

        return Self{
            .allocator = allocator,
            .condition = cond_ptr,
            .consequence = consequence,
            .alternative = alternative,
        };
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("if ");
        try self.condition.string(writer);
        try writer.writeAll(" ");
        try self.consequence.string(writer);
        try writer.writeAll(" else ");
        try self.alternative.string(writer);
    }

    pub fn deinit(self: Self) void {
        self.condition.deinit();
        self.allocator.destroy(self.condition);

        self.consequence.deinit();
        self.alternative.deinit();
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        const condition = try self.allocator.create(Expression);
        errdefer self.allocator.destroy(condition);

        condition.* = try self.condition.clone();
        errdefer condition.deinit();

        return Self{
            .allocator = self.allocator,
            .condition = condition,
            .consequence = try self.consequence.clone(),
            .alternative = try self.alternative.clone(),
        };
    }
};

pub const FunctionLiteral = struct {
    const Self = @This();

    allocator: Allocator,

    parameters: std.ArrayList([]const u8),
    body: BlockStatement,

    pub fn init(allocator: Allocator, parameters: std.ArrayList([]const u8), body: BlockStatement) Self {
        return Self{
            .allocator = allocator,
            .parameters = parameters,
            .body = body,
        };
    }

    pub fn addParameter(self: *Self, param: []const u8) Allocator.Error!void {
        const p = try self.allocator.alloc(u8, param.len);
        errdefer self.allocator.free(p);

        @memcpy(p, param);

        try self.parameters.append(p);
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("fn(");

        for (self.parameters.items, 0..) |param, idx| {
            if (idx != 0) {
                try writer.writeAll(", ");
            }
            try writer.writeAll(param);
        }

        try writer.writeAll(" {");
        try self.body.string(writer);
        try writer.writeAll("}");
    }

    pub fn deinit(self: Self) void {
        for (self.parameters.items) |param| {
            self.allocator.free(param);
        }
        self.parameters.deinit();

        self.body.deinit();
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        var params = try std.ArrayList([]const u8).initCapacity(self.allocator, self.parameters.items.len);
        errdefer {
            for (params.items) |param| {
                self.allocator.free(param);
            }
            params.deinit();
        }

        for (self.parameters.items) |param| {
            const p = try self.allocator.alloc(u8, param.len);
            errdefer self.allocator.free(p);

            @memcpy(p, param);

            try params.append(p);
        }

        return Self{
            .allocator = self.allocator,
            .parameters = params,
            .body = try self.body.clone(),
        };
    }
};

pub const FunctionCall = struct {
    const Self = @This();

    allocator: Allocator,

    function: *Expression,
    arguments: std.ArrayList(Expression),

    pub fn init(allocator: Allocator, function: Expression) Allocator.Error!Self {
        const fun = try allocator.create(Expression);
        fun.* = function;

        return Self{
            .allocator = allocator,
            .function = fun,
            .arguments = std.ArrayList(Expression).init(allocator),
        };
    }

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try self.function.string(writer);
        try writer.writeAll("(");

        for (self.arguments.items, 0..) |arg, idx| {
            if (idx != 0) {
                try writer.writeAll(", ");
            }
            try arg.string(writer);
        }

        try writer.writeAll(")");
    }

    pub fn deinit(self: Self) void {
        self.function.deinit();
        self.allocator.destroy(self.function);

        for (self.arguments.items) |arg| {
            arg.deinit();
        }
        self.arguments.deinit();
    }

    pub fn clone(self: Self) Allocator.Error!Self {
        const function = try self.allocator.create(Expression);
        errdefer self.allocator.destroy(function);

        function.* = try self.function.clone();
        errdefer function.deinit();

        var args = try std.ArrayList(Expression).initCapacity(self.allocator, self.arguments.items.len);
        errdefer {
            for (args.items) |arg| {
                arg.deinit();
            }
            args.deinit();
        }

        for (self.arguments.items) |arg| {
            try args.append(try arg.clone());
        }

        return Self{
            .allocator = self.allocator,
            .function = function,
            .arguments = args,
        };
    }
};

test "program to string" {
    const t = std.testing;

    var program = Program.init(t.allocator);
    defer program.deinit();

    try program.statements.append(.{
        .let_stmt = try LetStatement.init(
            t.allocator,
            "myVar",
            Expression{
                .identifier = try Identifier.init(t.allocator, "anotherVar"),
            },
        ),
    });

    var res = std.ArrayList(u8).init(t.allocator);
    defer res.deinit();

    _ = try program.string(res.writer());

    try t.expectEqualStrings("let myVar = anotherVar;", res.items);
}

test "bool expression to string" {
    const t = std.testing;
    const exp = Expression{ .boolean_literal = true };

    var res = std.ArrayList(u8).init(t.allocator);
    defer res.deinit();

    _ = try exp.string(res.writer());
    try t.expectEqualStrings("true", res.items);
}

test "clone test" {
    const t = std.testing;
    const stmt = Statement{
        .let_stmt = try LetStatement.init(
            t.allocator,
            "x",
            .{ .integer_literal = 42 },
        ),
    };
    defer stmt.deinit();

    const cloned_stmt = try stmt.clone();
    defer cloned_stmt.deinit();

    try t.expectEqualDeep(stmt, cloned_stmt);
}

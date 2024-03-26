const std = @import("std");
const Allocator = std.mem.Allocator;

pub const Program = struct {
    const Self = @This();

    statements: std.ArrayList(Statement),

    pub fn string(self: Self, writer: anytype) anyerror!void {
        for (self.statements.items) |item| {
            try item.string(writer);
        }
    }

    /// Deinitializes the program. Given allocator must be the same allocator
    /// used to initialize the program tree.
    ///
    /// This method works under the assumption, that the program tree was allocated
    /// with the given allocator, including the identifiers and other strings.
    ///
    /// It should mainly be used to deallocate the program tree after it was
    /// constructed with the parser. If the tree was constructed some other way,
    /// it should probably be cleaned up in a different way.
    pub fn deinit(self: *const Self, allocator: Allocator) void {
        for (self.statements.items) |item| {
            item.deinit(allocator);
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

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        switch (self.*) {
            .let_stmt => self.let_stmt.deinit(allocator),
            .return_stmt => self.return_stmt.deinit(allocator),
            .expression_stmt => self.expression_stmt.deinit(allocator),
        }
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Statement {
        return switch (self) {
            .let_stmt => |stmt| Self{ .let_stmt = try stmt.clone(allocator) },
            .return_stmt => |stmt| Self{ .return_stmt = try stmt.clone(allocator) },
            .expression_stmt => |stmt| Self{ .expression_stmt = try stmt.clone(allocator) },
        };
    }
};

pub const LetStatement = struct {
    const Self = @This();

    name: []const u8,
    value: Expression,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try std.fmt.format(writer, "let {s} = ", .{self.name});
        try self.value.string(writer);
        try writer.writeAll(";");
    }

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        allocator.free(self.name);
        self.value.deinit(allocator);
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        const name = try allocator.alloc(u8, self.name.len);
        errdefer allocator.free(name);

        @memcpy(name, self.name);

        return Self{
            .name = name,
            .value = try self.value.clone(allocator),
        };
    }
};

pub const ReturnStatement = struct {
    const Self = @This();

    value: Expression,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("return ");
        try self.value.string(writer);
        try writer.writeAll(";");
    }

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        self.value.deinit(allocator);
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        return Self{ .value = try self.value.clone(allocator) };
    }
};

pub const BlockStatement = struct {
    const Self = @This();

    statements: std.ArrayList(Statement),

    pub fn string(self: Self, writer: anytype) anyerror!void {
        for (self.statements.items) |item| {
            try item.string(writer);
        }
    }

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        for (self.statements.items) |item| {
            item.deinit(allocator);
        }
        self.statements.deinit();
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        var res = Self{ .statements = try std.ArrayList(Statement).initCapacity(allocator, self.statements.items.len) };
        errdefer res.deinit(allocator);

        for (self.statements.items) |stmt| {
            try res.statements.append(try stmt.clone(allocator));
        }

        return res;
    }
};

pub const Expression = union(enum) {
    const Self = @This();

    identifier: []const u8,
    integer_literal: i64,
    boolean_literal: bool,
    prefix_operator: PrefixOperator,
    infix_operator: InfixOperator,
    if_expression: IfExpression,
    function_literal: FunctionLiteral,
    function_call: FunctionCall,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        return switch (self) {
            .identifier => writer.writeAll(self.identifier),
            .integer_literal => writer.print("{d}", .{self.integer_literal}),
            .boolean_literal => writer.print("{any}", .{self.boolean_literal}),
            .prefix_operator => self.prefix_operator.string(writer),
            .infix_operator => self.infix_operator.string(writer),
            .if_expression => self.if_expression.string(writer),
            .function_literal => self.function_literal.string(writer),
            .function_call => self.function_call.string(writer),
        };
    }

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        switch (self.*) {
            .identifier => allocator.free(self.identifier),
            .prefix_operator => self.prefix_operator.deinit(allocator),
            .infix_operator => self.infix_operator.deinit(allocator),
            .if_expression => self.if_expression.deinit(allocator),
            .function_literal => self.function_literal.deinit(allocator),
            .function_call => self.function_call.deinit(allocator),
            else => {},
        }
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        return switch (self) {
            .identifier => |val| blk: {
                const ident = try allocator.alloc(u8, val.len);
                @memcpy(ident, val);

                break :blk Self{ .identifier = ident };
            },
            .integer_literal => |val| .{ .integer_literal = val },
            .boolean_literal => |val| .{ .boolean_literal = val },
            .prefix_operator => |val| .{ .prefix_operator = try val.clone(allocator) },
            .infix_operator => |val| .{ .infix_operator = try val.clone(allocator) },
            .if_expression => |val| .{ .if_expression = try val.clone(allocator) },
            .function_literal => |val| .{ .function_literal = try val.clone(allocator) },
            .function_call => |val| .{ .function_call = try val.clone(allocator) },
        };
    }
};

pub const PrefixOperatorKind = enum {
    not,
    negative,
};

pub const PrefixOperator = struct {
    const Self = @This();

    operator: PrefixOperatorKind,
    right: *Expression,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("(");

        switch (self.operator) {
            .not => try writer.writeAll("!"),
            .negative => try writer.writeAll("-"),
        }

        try self.right.string(writer);
        try writer.writeAll(")");
    }

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        self.right.deinit(allocator);
        allocator.destroy(self.right);
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        const right = try allocator.create(Expression);
        errdefer allocator.destroy(right);

        right.* = try self.right.clone(allocator);

        return Self{
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

    operator: InfixOperatorKind,
    left: *Expression,
    right: *Expression,

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

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        self.left.deinit(allocator);
        allocator.destroy(self.left);

        self.right.deinit(allocator);
        allocator.destroy(self.right);
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        const left = try allocator.create(Expression);
        errdefer allocator.destroy(left);

        left.* = try self.left.clone(allocator);
        errdefer left.deinit(allocator);

        const right = try allocator.create(Expression);
        errdefer allocator.destroy(right);

        right.* = try self.right.clone(allocator);

        return Self{
            .operator = self.operator,
            .left = left,
            .right = right,
        };
    }
};

pub const IfExpression = struct {
    const Self = @This();

    condition: *Expression,
    consequence: BlockStatement,
    alternative: BlockStatement,

    pub fn string(self: Self, writer: anytype) anyerror!void {
        try writer.writeAll("if ");
        try self.condition.string(writer);
        try writer.writeAll(" ");
        try self.consequence.string(writer);
        try writer.writeAll(" else ");
        try self.alternative.string(writer);
    }

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        self.condition.deinit(allocator);
        allocator.destroy(self.condition);

        self.consequence.deinit(allocator);
        self.alternative.deinit(allocator);
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        const condition = try allocator.create(Expression);
        errdefer allocator.destroy(condition);

        condition.* = try self.condition.clone(allocator);
        errdefer condition.deinit(allocator);

        return Self{
            .condition = condition,
            .consequence = try self.consequence.clone(allocator),
            .alternative = try self.alternative.clone(allocator),
        };
    }
};

pub const FunctionLiteral = struct {
    const Self = @This();

    parameters: std.ArrayList([]const u8),
    body: BlockStatement,

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

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        for (self.parameters.items) |param| {
            allocator.free(param);
        }
        self.parameters.deinit();

        self.body.deinit(allocator);
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        var params = try std.ArrayList([]const u8).initCapacity(allocator, self.parameters.items.len);
        errdefer {
            for (params.items) |param| {
                allocator.free(param);
            }
            params.deinit();
        }

        for (self.parameters.items) |param| {
            const p = try allocator.alloc(u8, param.len);
            @memcpy(p, param);
            try params.append(p);
        }

        return Self{
            .parameters = params,
            .body = try self.body.clone(allocator),
        };
    }
};

pub const FunctionCall = struct {
    const Self = @This();

    function: *Expression,
    arguments: std.ArrayList(Expression),

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

    pub fn deinit(self: *const Self, allocator: Allocator) void {
        self.function.deinit(allocator);
        allocator.destroy(self.function);

        for (self.arguments.items) |arg| {
            arg.deinit(allocator);
        }
        self.arguments.deinit();
    }

    pub fn clone(self: Self, allocator: Allocator) Allocator.Error!Self {
        const function = try allocator.create(Expression);
        errdefer allocator.destroy(function);

        function.* = try self.function.clone(allocator);
        errdefer function.deinit(allocator);

        var args = try std.ArrayList(Expression).initCapacity(allocator, self.arguments.items.len);
        errdefer {
            for (args.items) |arg| {
                arg.deinit(allocator);
            }
            args.deinit();
        }

        for (self.arguments.items) |arg| {
            try args.append(try arg.clone(allocator));
        }

        return Self{
            .function = function,
            .arguments = args,
        };
    }
};

test "program to string" {
    const t = std.testing;

    var program = Program{
        .statements = std.ArrayList(Statement).init(t.allocator),
    };
    defer program.statements.deinit();

    try program.statements.append(.{
        .let_stmt = LetStatement{
            .name = "myVar",
            .value = .{
                .identifier = "anotherVar",
            },
        },
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
    const stmt = Statement{ .let_stmt = .{ .name = "x", .value = .{ .integer_literal = 42 } } };
    const cloned_stmt = try stmt.clone(t.allocator);
    defer cloned_stmt.deinit(t.allocator);

    try t.expectEqualDeep(stmt, cloned_stmt);
}

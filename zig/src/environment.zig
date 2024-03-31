const std = @import("std");
const Allocator = std.mem.Allocator;
const Rc = @import("rc.zig").Rc;

const Object = @import("object.zig").Object;

pub const Environment = struct {
    const Self = @This();

    allocator: Allocator,

    store: std.StringHashMap(Object),
    outer: ?*Environment,

    pub fn init(allocator: Allocator, outer: ?*Self) Self {
        return Self{
            .allocator = allocator,
            .store = std.StringHashMap(Object).init(allocator),
            .outer = outer,
        };
    }

    pub fn get(self: Self, key: []const u8) ?Object {
        const value = self.store.get(key);
        if (value) |val| {
            return val;
        }

        if (self.outer) |outer| {
            return outer.get(key);
        }

        return null;
    }

    pub fn put(self: *Self, key: []const u8, value: Object) Allocator.Error!void {
        const key_copy = try self.allocator.alloc(u8, key.len);
        errdefer self.allocator.free(key_copy);

        @memcpy(key_copy, key);

        try self.store.put(key_copy, value);
    }

    // Deallocate all keys in the store and call deinit on the store.
    // Outer environment is not affected.
    pub fn deinit(self: *Self) void {
        // Deallocate keys
        var key_iter = self.store.keyIterator();
        while (key_iter.next()) |key| {
            self.allocator.free(key.*);
        }

        self.store.deinit();
    }
};

test "Environment: extend" {
    const t = std.testing;

    var env = Environment.init(t.allocator, null);
    defer env.deinit();

    const obj1 = Object{ .integer = 1 };
    try env.put("key", obj1);

    var env2 = Environment.init(t.allocator, &env);
    defer env2.deinit();

    const obj2 = Object{ .integer = 2 };
    try env2.put("key", obj2);

    const got = env2.get("key").?;
    try t.expectEqualDeep(Object{ .integer = 2 }, got);

    const got2 = env.get("key").?;
    try t.expectEqualDeep(Object{ .integer = 1 }, got2);
}

test "Environment: function object" {
    const ast = @import("ast.zig");
    const obj = @import("object.zig");
    const t = std.testing;

    var env = Environment.init(t.allocator, null);
    defer env.deinit();

    var params = [_][]const u8{"x"};

    var body = ast.BlockStatement.init(t.allocator);
    defer body.deinit();
    try body.statements.append(ast.Statement{ .expression_stmt = ast.Expression{ .integer_literal = 42 } });

    const fn_obj = try t.allocator.create(obj.FunctionObject);
    defer t.allocator.destroy(fn_obj);
    fn_obj.* = try obj.FunctionObject.init(
        t.allocator,
        &params,
        body,
        &env,
    );
    defer fn_obj.deinit();

    const object = Object{ .function_obj = fn_obj };

    try env.put("f", object);
}

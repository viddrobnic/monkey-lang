const std = @import("std");
const Allocator = std.mem.Allocator;
const Rc = @import("rc.zig").Rc;
const object = @import("object.zig");
const Object = object.Object;

pub const Environment = struct {
    const Self = @This();

    store: std.StringHashMap(Rc(Object)),
    outer: ?Rc(Environment),

    pub fn init(allocator: std.mem.Allocator) Allocator.Error!Rc(Self) {
        const env = Self{
            .store = std.StringHashMap(Rc(Object)).init(allocator),
            .outer = null,
        };
        return try Rc(Self).init(allocator, env);
    }

    /// Create a new environment that extends the given environment.
    /// The new environment will retain the given environment Rc.
    /// Therefore the caller of the method must only release the new environment.
    pub fn extend(outer: *Rc(Environment), allocator: Allocator) Allocator.Error!Rc(Self) {
        const new_outer = outer.retain();
        const env = Self{
            .store = std.StringHashMap(Rc(Object)).init(allocator),
            .outer = new_outer,
        };

        return try Rc(Self).init(allocator, env);
    }

    /// Get the value associated with the given key.
    /// The resulting Rc will be retained, so the caller must only release it.
    pub fn get(self: Self, key: []const u8) ?Rc(Object) {
        var value = self.store.get(key);
        if (value) |*val| {
            return val.retain();
        }

        if (self.outer) |outer| {
            return outer.value.get(key);
        }

        return null;
    }

    pub fn put(self: *Self, key: []const u8, value: Rc(Object), allocator: Allocator) Allocator.Error!void {
        if (self.store.getPtr(key)) |existing| {
            object.releaseObject(existing, allocator);
        }

        try self.store.put(key, value);
    }
};

pub fn releaseEnvironment(env: *Rc(Environment), allocator: Allocator) void {
    if (env.strongCount() == 1) {
        // After we call env.release(), the strong count will be 0 and the environment will be deallocated.
        // We need to make sure to deallocate the environment inside Rc before we release it.
        if (env.value.outer) |*outer| {
            releaseEnvironment(outer, allocator);
        }

        var iter = env.value.store.valueIterator();
        while (iter.next()) |entry| {
            object.releaseObject(entry, allocator);
        }
        env.value.store.deinit();
    }

    env.release();
}

test "Environment: double put" {
    const t = std.testing;

    var env = try Environment.init(t.allocator);
    defer releaseEnvironment(&env, t.allocator);

    const obj1 = try Rc(Object).init(t.allocator, Object{ .integer = 1 });
    const obj2 = try Rc(Object).init(t.allocator, Object{ .integer = 2 });

    try env.value.put("key", obj1, t.allocator);
    try env.value.put("key", obj2, t.allocator);

    var got = env.value.get("key").?;
    defer object.releaseObject(&got, t.allocator);
    try t.expectEqualDeep(Object{ .integer = 2 }, got.value.*);
}

test "Environment: extend" {
    const t = std.testing;

    var env = try Environment.init(t.allocator);
    defer releaseEnvironment(&env, t.allocator);

    const obj1 = try Rc(Object).init(t.allocator, Object{ .integer = 1 });
    try env.value.put("key", obj1, t.allocator);

    var env2 = try Environment.extend(&env, t.allocator);
    defer releaseEnvironment(&env2, t.allocator);

    const obj2 = try Rc(Object).init(t.allocator, Object{ .integer = 2 });
    try env2.value.put("key", obj2, t.allocator);

    var got = env2.value.get("key").?;
    defer object.releaseObject(&got, t.allocator);
    try t.expectEqualDeep(Object{ .integer = 2 }, got.value.*);

    var got2 = env.value.get("key").?;
    defer object.releaseObject(&got2, t.allocator);
    try t.expectEqualDeep(Object{ .integer = 1 }, got2.value.*);
}

test "Environment: cycle" {
    const ast = @import("ast.zig");
    const t = std.testing;

    var env = try Environment.init(t.allocator);
    defer releaseEnvironment(&env, t.allocator);

    const fn_obj = object.FunctionObject{
        .parameters = std.ArrayList([]const u8).init(t.allocator),
        .body = ast.BlockStatement{
            .statements = std.ArrayList(ast.Statement).init(t.allocator),
        },
        .environment = env.downgrade(),
    };

    const obj = Object{ .function_obj = fn_obj };
    try env.value.put("f", try Rc(Object).init(t.allocator, obj), t.allocator);
}

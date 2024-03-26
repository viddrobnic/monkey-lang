const std = @import("std");
const rc = @import("rc.zig");
const ast = @import("ast.zig");
const env = @import("environment.zig");

pub const Object = union(enum) {
    integer: i64,
    boolean: bool,
    return_obj: rc.Rc(Object),
    function_obj: FunctionObject,
    null_obj,
};

pub const FunctionObject = struct {
    parameters: std.ArrayList([]const u8),
    body: ast.BlockStatement,
    environment: rc.Rc(env.Environment).Weak,
};

pub fn releaseObject(obj: *rc.Rc(Object), allocator: std.mem.Allocator) void {
    if (obj.strongCount() == 1) {
        // After we call obj.release(), the strong count will be 0 and the object will be deallocated.
        // We need to deallocate the inner objects before we release the outer object.
        switch (obj.value.*) {
            .return_obj => |*inner| releaseObject(inner, allocator),
            .function_obj => |*inner| {
                for (inner.parameters.items) |param| {
                    allocator.free(param);
                }
                inner.parameters.deinit();

                inner.body.deinit(allocator);

                inner.environment.release();
            },
            else => {},
        }
    }

    obj.release();
}

test "Object: return" {
    const t = std.testing;
    const rc_obj = try rc.Rc(Object).init(t.allocator, Object{ .integer = 42 });

    var return_obj = try rc.Rc(Object).init(t.allocator, Object{ .return_obj = rc_obj });

    try t.expectEqual(42, return_obj.value.return_obj.value.integer);

    releaseObject(&return_obj, t.allocator);
}

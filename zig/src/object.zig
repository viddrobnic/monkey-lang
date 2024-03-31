const std = @import("std");
const Allocator = std.mem.Allocator;

const ast = @import("ast.zig");
const env = @import("environment.zig");

pub const ObjectTag = enum {
    integer,
    boolean,
    return_obj,
    function_obj,
    null_obj,
};

pub const Object = union(ObjectTag) {
    const Self = @This();

    integer: i64,
    boolean: bool,
    return_obj: *Object,
    function_obj: *FunctionObject,
    null_obj: void,
};

pub const FunctionObject = struct {
    const Self = @This();

    allocator: Allocator,

    parameters: std.ArrayList([]const u8),
    body: ast.BlockStatement,
    environment: *env.Environment,

    pub fn init(allocator: Allocator, parameters: [][]const u8, body: ast.BlockStatement, environment: *env.Environment) Allocator.Error!Self {
        var params = try std.ArrayList([]const u8).initCapacity(allocator, parameters.len);
        errdefer {
            for (params.items) |param| {
                allocator.free(param);
            }
            params.deinit();
        }

        for (parameters) |param| {
            const param_copy = try allocator.alloc(u8, param.len);
            errdefer allocator.free(param_copy);

            @memcpy(param_copy, param);

            try params.append(param_copy);
        }

        return Self{
            .allocator = allocator,
            .parameters = params,
            .body = try body.clone(),
            .environment = environment,
        };
    }

    pub fn deinit(self: Self) void {
        for (self.parameters.items) |param| {
            self.allocator.free(param);
        }
        self.parameters.deinit();

        self.body.deinit();
    }
};

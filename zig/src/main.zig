const std = @import("std");
const parser = @import("parser.zig");
const eval = @import("evaluate.zig");

pub fn main() !void {
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

    const allocator = std.heap.c_allocator;

    const program = try parser.parse(input, allocator);
    defer program.deinit();

    var evaluator = try eval.Evaluator.init(allocator);
    defer evaluator.deinit();

    const res = try evaluator.evaluate(program);
    std.debug.print("{}\n", .{res});
}

test {
    _ = @import("token.zig");
    _ = @import("lexer.zig");
    _ = @import("ast.zig");
    _ = @import("parser.zig");
    _ = @import("object.zig");
    _ = @import("evaluate.zig");
}

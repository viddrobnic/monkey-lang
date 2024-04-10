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
        \\ fibonacci(20);
    ;

    const allocator = std.heap.c_allocator;

    var total_ns: u64 = 0;

    for (0..100) |_| {
        var t = try std.time.Timer.start();

        const program = try parser.parse(input, allocator);
        defer program.deinit();

        var evaluator = try eval.Evaluator.init(allocator);
        defer evaluator.deinit();

        _ = try evaluator.evaluate(program);

        total_ns += t.read();
    }

    const average = total_ns / 100;
    std.debug.print("Average: {}\n", .{std.fmt.fmtDuration(average)});
}

test {
    _ = @import("token.zig");
    _ = @import("lexer.zig");
    _ = @import("ast.zig");
    _ = @import("parser.zig");
    _ = @import("object.zig");
    _ = @import("evaluate.zig");
}

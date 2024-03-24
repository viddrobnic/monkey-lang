const std = @import("std");

pub const Token = union(enum) {
    illegal: u8,
    eof,
    // Identifiers + literals
    ident: []const u8, // add, foobar, x, y, ...
    int: []const u8, // 1343456
    // Operators
    assign,
    plus,
    minus,
    bang,
    asterisk,
    slash,
    lt,
    gt,
    eq,
    not_eq,
    // Delimiters
    comma,
    semicolon,
    lparen,
    rparen,
    lsquigly,
    rsquigly,
    // Keywords
    function,
    let,
    true_token,
    false_token,
    if_token,
    else_token,
    return_token,

    pub fn keywoard(ident: []const u8) ?Token {
        const map = std.ComptimeStringMap(Token, .{
            .{ "let", .let },
            .{ "fn", .function },
            .{ "if", .if_token },
            .{ "true", .true_token },
            .{ "false", .false_token },
            .{ "return", .return_token },
            .{ "else", .else_token },
        });
        return map.get(ident);
    }
};

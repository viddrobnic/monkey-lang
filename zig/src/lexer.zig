const std = @import("std");
const Token = @import("token.zig").Token;

pub const Lexer = struct {
    const Self = Lexer;

    input: []const u8,
    position: usize = 0,
    read_position: usize = 0,
    ch: u8 = 0,

    pub fn init(input: []const u8) Lexer {
        var lex = Self{
            .input = input,
        };
        lex.readChar();

        return lex;
    }

    fn readChar(self: *Self) void {
        if (self.read_position >= self.input.len) {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn skipWhitespace(self: *Self) void {
        while (std.ascii.isWhitespace(self.ch)) {
            self.readChar();
        }
    }

    fn peekChar(self: *Self) u8 {
        if (self.read_position >= self.input.len) {
            return 0;
        } else {
            return self.input[self.read_position];
        }
    }

    fn readIdentifier(self: *Self) []const u8 {
        const start_position = self.position;
        while (isLetter(self.ch)) {
            self.readChar();
        }
        return self.input[start_position..self.position];
    }

    fn readNumber(self: *Self) []const u8 {
        const start_position = self.position;
        while (std.ascii.isDigit(self.ch)) {
            self.readChar();
        }
        return self.input[start_position..self.position];
    }

    pub fn nextToken(self: *Self) Token {
        self.skipWhitespace();

        const tok: Token = switch (self.ch) {
            '=' => blk: {
                if (self.peekChar() == '=') {
                    self.readChar();
                    break :blk .eq;
                } else {
                    break :blk .assign;
                }
            },
            ';' => .semicolon,
            '(' => .lparen,
            ')' => .rparen,
            ',' => .comma,
            '+' => .plus,
            '-' => .minus,
            '{' => .lsquigly,
            '}' => .rsquigly,
            '!' => blk: {
                if (self.peekChar() == '=') {
                    self.readChar();
                    break :blk .not_eq;
                } else {
                    break :blk .bang;
                }
            },
            '/' => .slash,
            '*' => .asterisk,
            '<' => .lt,
            '>' => .gt,
            0 => .eof,
            else => blk: {
                if (isLetter(self.ch)) {
                    const ident = self.readIdentifier();
                    if (Token.keywoard(ident)) |token| {
                        return token;
                    } else {
                        return .{ .ident = ident };
                    }
                } else if (std.ascii.isDigit(self.ch)) {
                    const number = self.readNumber();
                    return .{ .int = number };
                } else {
                    break :blk .{ .illegal = self.ch };
                }
            },
        };

        self.readChar();
        return tok;
    }
};

fn isLetter(ch: u8) bool {
    return std.ascii.isAlphabetic(ch) or ch == '_';
}

test "lexer - init" {
    const testing = @import("std").testing;

    const lexer = Lexer.init("asdf");
    try testing.expectEqual(lexer.ch, 'a');
    try testing.expectEqual(lexer.position, 0);
    try testing.expectEqual(lexer.read_position, 1);
}

test "lexer - nextToken" {
    const testing = @import("std").testing;

    const input =
        \\let five = 5;
        \\let ten = 10;
        \\
        \\let add = fn(x, y) {
        \\    x + y;
        \\};
        \\
        \\let result = add(five, ten);
        \\!-/*5;
        \\5 < 10 > 5;
        \\
        \\if (5 < 10) {
        \\    return true;
        \\} else {
        \\    return false;
        \\}
        \\
        \\10 == 10;
        \\10 != 9;
    ;

    var lex = Lexer.init(input);
    const expected = [_]Token{
        .let,
        .{ .ident = "five" },
        .assign,
        .{ .int = "5" },
        .semicolon,
        .let,
        .{ .ident = "ten" },
        .assign,
        .{ .int = "10" },
        .semicolon,
        .let,
        .{ .ident = "add" },
        .assign,
        .function,
        .lparen,
        .{ .ident = "x" },
        .comma,
        .{ .ident = "y" },
        .rparen,
        .lsquigly,
        .{ .ident = "x" },
        .plus,
        .{ .ident = "y" },
        .semicolon,
        .rsquigly,
        .semicolon,
        .let,
        .{ .ident = "result" },
        .assign,
        .{ .ident = "add" },
        .lparen,
        .{ .ident = "five" },
        .comma,
        .{ .ident = "ten" },
        .rparen,
        .semicolon,
        .bang,
        .minus,
        .slash,
        .asterisk,
        .{ .int = "5" },
        .semicolon,
        .{ .int = "5" },
        .lt,
        .{ .int = "10" },
        .gt,
        .{ .int = "5" },
        .semicolon,
        .if_token,
        .lparen,
        .{ .int = "5" },
        .lt,
        .{ .int = "10" },
        .rparen,
        .lsquigly,
        .return_token,
        .true_token,
        .semicolon,
        .rsquigly,
        .else_token,
        .lsquigly,
        .return_token,
        .false_token,
        .semicolon,
        .rsquigly,
        .{ .int = "10" },
        .eq,
        .{ .int = "10" },
        .semicolon,
        .{ .int = "10" },
        .not_eq,
        .{ .int = "9" },
        .semicolon,
        .eof,
    };

    for (expected) |expected_token| {
        const tok = lex.nextToken();
        try testing.expectEqualDeep(expected_token, tok);
    }
}

const std = @import("std");

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len != 2) {
        std.log.err("Usage: {} <source_file>", .{args[0]}) catch {};
        return;
    }

    const source_file = args[1];
    const assembly_file = try createAssemblyFileName(source_file);

    const source_re = "int main\\s*\\(\\s*\\)\\s*{\\s*return\\s+(?<ret>[0-9]+)\\s*;\\s*}";
    const assembly_format = \\
        .globl _main
_main:
    movl    ${}, %eax
    ret
\\;

    var infile = try std.fs.cwd().openFile(source_file, .{ .read = true });
    defer infile.close();

    const source_data = try infile.readAllAlloc(allocator, null);
    defer allocator.free(source_data);

    const source = std.mem.trim(u8, source_data, " \t\n\r");

    var regex = try std.regex.Regex.init(allocator, source_re, .{});
    defer regex.deinit();

    var it = regex.iterate(source);
    if (!it.next()) {
        std.log.err("No match found in source file", .{}) catch {};
        return;
    }

    const m = it.match();
    const ret_val = m.group("ret") orelse {
        std.log.err("Failed to extract return value", .{}) catch {};
        return;
    };

    const ret_val_trimmed = std.mem.trim(u8, ret_val, " \t\n\r");
    const final_val = try std.fmt.parseInt(i32, ret_val_trimmed, 10);

    var outfile = try std.fs.cwd().createFile(assembly_file, .{ .write = true, .create = true });
    defer outfile.close();

    try outfile.writeAll(std.fmt.format(assembly_format, .{final_val}));
}

fn createAssemblyFileName(source_file: []const u8) ![]u8 {
    const allocator = std.heap.page_allocator;
    const index = std.mem.lastIndexOf(u8, source_file, '.') orelse return error.InvalidFileName;

    const base_name = source_file[0..index];
    return std.mem.concat(u8, &[_][]const u8{ base_name, ".s" }, allocator);
}


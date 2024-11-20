
const std = @import("std");

pub fn main() !void {
    const args = try std.process.argsAlloc(std.heap.page_allocator);
    defer std.process.argsFree(std.heap.page_allocator, args);

    if (args.len != 2) {
        std.log.err("Usage: {} <source_file>", .{args[0]}) catch {};
        return;
    }

    const source_file = args[1];
    const assembly_file = try createAssemblyFileName(source_file);

    const source_re = "int main\\s*\\(\\s*\\)\\s*{\\s*return\\s+(?P<ret>[0-9]+)\\s*;\\s*}";
    const assembly_format = \\    
        \\    .globl _main
        \\_main:
        \\    movl    ${}, %eax
        \\    ret
    \\;

    var infile = try std.fs.cwd().openFile(source_file, .{ .read = true });
    defer infile.close();

    const source_data = try infile.readAllAlloc(std.heap.page_allocator, null);
    defer std.heap.page_allocator.free(source_data);

    const source = std.mem.trim(source_data, " \t\n\r");
    const match = try std.regex.Matcher.init(source_re, source, std.heap.page_allocator);

    defer match.deinit();

    if (!match.findNext()) {
        std.log.err("No match found in source file", .{}) catch {};
        return;
    }

    const ret_val = match.getNamedGroup("ret") orelse |err| {
        std.log.err("Failed to extract return value: {}", .{err}) catch {};
        return;
    };

    const ret_val_trimmed = std.mem.trim(ret_val, " \t\n\r");
    const final_val = try std.fmt.parseInt(i32, ret_val_trimmed, 10);

    var outfile = try std.fs.cwd().createFile(assembly_file, .{ .write = true, .create = true });
    defer outfile.close();

    try outfile.writeAll(std.fmt.format(assembly_format, .{final_val}));
}

fn createAssemblyFileName(source_file: []const u8) ![]u8 {
    const index = std.mem.lastIndexOf(source_file, '.');
    if (index == null) return error.InvalidFileName;

    const base_name = source_file[0..index.?];
    return std.mem.concat(u8, base_name, ".s", std.heap.page_allocator);
}

import glob
import sys
import re

def verify_file(filepath):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    stack = []
    lines = content.splitlines()
    in_raw_string = False
    raw_hash_count = 0
    in_string = False
    in_char = False
    in_line_comment = False
    in_block_comment = 0  # Rust supports nested block comments!
    escape = False

    for line_idx, line in enumerate(lines, 1):
        idx = 0
        in_line_comment = False
        while idx < len(line):
            ch = line[idx]

            if in_line_comment:
                break

            # Handle block comments (including nested)
            if in_block_comment > 0:
                if ch == "/" and idx + 1 < len(line) and line[idx+1] == "*":
                    in_block_comment += 1
                    idx += 2
                    continue
                elif ch == "*" and idx + 1 < len(line) and line[idx+1] == "/":
                    in_block_comment -= 1
                    idx += 2
                    continue
                idx += 1
                continue

            # Handle raw strings
            if in_raw_string:
                if ch == '"':
                    # check if followed by raw_hash_count hashes
                    end_hashes = 0
                    for h_idx in range(idx + 1, len(line)):
                        if line[h_idx] == '#':
                            end_hashes += 1
                        else:
                            break
                    if end_hashes >= raw_hash_count:
                        in_raw_string = False
                        idx += 1 + raw_hash_count
                        continue
                idx += 1
                continue

            # Handle normal strings
            if in_string:
                if escape:
                    escape = False
                elif ch == "\\":
                    escape = True
                elif ch == '"':
                    in_string = False
                idx += 1
                continue

            # Handle chars
            if in_char:
                if escape:
                    escape = False
                elif ch == "\\":
                    escape = True
                elif ch == "'":
                    in_char = False
                idx += 1
                continue

            # Comments
            if ch == "/" and idx + 1 < len(line):
                if line[idx+1] == "/":
                    in_line_comment = True
                    break
                elif line[idx+1] == "*":
                    in_block_comment += 1
                    idx += 2
                    continue

            # Raw string start: r#" or r##" or br#"
            if (ch == 'r' or (ch == 'b' and idx + 1 < len(line) and line[idx+1] == 'r')) and idx < len(line):
                r_pos = idx if ch == 'r' else idx + 1
                hashes = 0
                for h_pos in range(r_pos + 1, len(line)):
                    if line[h_pos] == '#':
                        hashes += 1
                    elif line[h_pos] == '"':
                        in_raw_string = True
                        raw_hash_count = hashes
                        idx = h_pos + 1
                        break
                    else:
                        break
                if in_raw_string:
                    continue

            if ch == '"':
                in_string = True
                idx += 1
                continue

            if ch == "'" and (idx == 0 or line[idx-1] in " ([{,;:=!&|<>"):
                # Potential char literal vs lifetime
                if idx + 2 < len(line) and line[idx+2] == "'":
                    in_char = True
                    idx += 1
                    continue
                elif idx + 3 < len(line) and line[idx+1] == "\\" and line[idx+3] == "'":
                    in_char = True
                    idx += 1
                    continue

            if ch in "{[(":
                stack.append((ch, line_idx, idx + 1))
            elif ch in "}])":
                matching = {"}": "{", "]": "[", ")": "("}[ch]
                if not stack:
                    print(f"Error in {filepath}:{line_idx}:{idx+1} - Unexpected closing '{ch}'")
                    return False
                last, l_num, c_num = stack.pop()
                if last != matching:
                    print(f"Error in {filepath}:{line_idx}:{idx+1} - Mismatched '{last}' (from line {l_num}) with '{ch}'")
                    return False
            idx += 1

    if stack:
        print(f"Error in {filepath}: Unclosed delimiters: {stack}")
        return False

    return True

def main():
    files = sorted(glob.glob("crates/**/*.rs", recursive=True))
    all_ok = True
    print(f"Verifying {len(files)} Rust files...")
    for f in files:
        if not verify_file(f):
            all_ok = False

    if all_ok:
        print(f"All {len(files)} Rust files verified successfully! 100% bracket balance.")
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()

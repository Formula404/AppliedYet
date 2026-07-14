"""Replace hex color values with CSS variables, skipping :root and [data-theme] blocks."""
import sys

path = r"F:\Postgraduate\AppliedYet\apps\desktop\src\styles\index.css"

with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()

hex_to_var = {
    "#f6f8fb": "var(--bg)",
    "#667085": "var(--muted)",
    "#98a2b3": "var(--text-placeholder)",
    "#344054": "var(--text-secondary)",
    "#dce1e9": "var(--border-input)",
    "#eef0f4": "var(--border-light)",
    "#f2f4f7": "var(--surface-hover)",
    "#f9fafc": "var(--soft)",
}

result = []
in_block = False

for line in lines:
    stripped = line.strip()

    if stripped.startswith(":root {") or stripped.startswith("[data-theme"):
        in_block = True
    elif in_block and stripped == "}":
        in_block = False

    if not in_block:
        for hex_val, var_name in hex_to_var.items():
            line = line.replace(hex_val, var_name)

    result.append(line)

with open(path, "w", encoding="utf-8") as f:
    f.writelines(result)

print("OK: replaced color hexes outside definition blocks")

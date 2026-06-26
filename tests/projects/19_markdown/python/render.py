# Renderer for mini-Markdown. Turns typed blocks into HTML lines. Handles
# inline spans: HTML escaping, bold (**...**) and inline code (`...`).

from parser import count_kind, count_list_items


def find_from(s, target, start):
    i = start
    limit = len(s) - len(target)
    while i <= limit:
        if s[i:i + len(target)] == target:
            return i
        i = i + 1
    return -1


def escape_html(s):
    out = s.replace("&", "&amp;")
    out = out.replace("<", "&lt;")
    out = out.replace(">", "&gt;")
    return out


def wrap_pairs(s, marker, open_tag, close_tag):
    out = ""
    i = 0
    open_next = True
    n = len(s)
    while i < n:
        if i + len(marker) <= n and s[i:i + len(marker)] == marker:
            if open_next:
                out = out + open_tag
            else:
                out = out + close_tag
            open_next = not open_next
            i = i + len(marker)
        else:
            out = out + s[i:i + 1]
            i = i + 1
    return out


def render_inline(s):
    escaped = escape_html(s)
    bolded = wrap_pairs(escaped, "**", "<strong>", "</strong>")
    coded = wrap_pairs(bolded, "`", "<code>", "</code>")
    return coded


def count_bold_spans(s):
    count = 0
    i = 0
    j = find_from(s, "**", i)
    while j >= 0:
        count = count + 1
        i = j + 2
        j = find_from(s, "**", i)
    return count // 2


def render_heading(block):
    tag = "h" + str(block.level)
    inner = render_inline(block.lines[0])
    return "<" + tag + ">" + inner + "</" + tag + ">"


def render_list(block):
    lines = []
    lines.append("<ul>")
    for item in block.lines:
        lines.append("  <li>" + render_inline(item) + "</li>")
    lines.append("</ul>")
    return lines


def render_para(block):
    joined = " ".join(block.lines)
    return "<p>" + render_inline(joined) + "</p>"


def render_blocks(blocks):
    out = []
    for b in blocks:
        if b.kind == "heading":
            out.append(render_heading(b))
        elif b.kind == "list":
            for line in render_list(b):
                out.append(line)
        else:
            out.append(render_para(b))
    return out

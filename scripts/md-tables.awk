# just md-tables runs it on every tracked Markdown file, one file at a time (POSIX awk: gawk or mawk).
# A Markdown table row lives in a table: each block of `|` lines (outside code fences) starts with a
# header row and a separator row (| :-- | --: |). Prints file:line of each orphan; exit 1 if any.
function orphan(msg) { print FILENAME ":" start ": " msg; bad = 1 }
/^[ ]*(```|~~~)/ { fence = !fence; n = 0; next }
fence { next }
/^\|/ {
  if (++n == 1) start = FNR
  if (n == 2 && $0 !~ /^\|[-:| ]+[ ]*$/) orphan("table rows with no separator row")
  next
}
{ if (n == 1) orphan("a table row outside a table"); n = 0 }
END { if (n == 1) orphan("a table row outside a table"); exit bad }

=== start ===
Elige una opción:
-> menu

=== menu ===
* [generate 1 line] -> generate_1
* [generate 2 lines] -> generate_2
* [generate 5 lines] -> generate_5
* [generate 10 lines] -> generate_10
* [generate 50 lines] -> generate_50
* [Terminar] -> END

=== generate_1 ===
{ generate_lines(1) }
-> menu

=== generate_2 ===
{ generate_lines(2) }
-> menu

=== generate_5 ===
{ generate_lines(5) }
-> menu

=== generate_10 ===
{ generate_lines(10) }
-> menu

=== generate_50 ===
{ generate_lines(50) }
-> menu

=== function generate_lines(line_count) ===
{~ temp current_line = 1}
-> generate_lines_loop

=== generate_lines_loop ===
Línea generada {current_line}.
{~ current_line = current_line + 1}
{ current_line <= line_count:
    -> generate_lines_loop
- else:
    ->->
}

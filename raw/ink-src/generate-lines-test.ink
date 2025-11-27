
-> menu

=== menu ===
+ [generate 1 line] ->generate_lines(1)->menu
+ [generate 2 lines] -> generate_lines(2)->menu
+ [generate 5 lines] -> generate_lines(5)->menu
+ [generate 10 lines] -> generate_lines(10)->menu
+ [generate 50 lines] -> generate_lines(50)->menu
+ [generate long lines] -> menu2
+ [End] -> END


=== menu2 ===
+ [generate 1 long line] ->generate_long_lines(1)->menu2
+ [generate 2 long lines] -> generate_long_lines(2)->menu2
+ [generate 5 long lines] -> generate_long_lines(5)->menu2
+ [generate 10 long lines] -> generate_long_lines(10)->menu2
+ [generate 50 long lines] -> generate_long_lines(50)->menu2
+ [Back] -> menu

=== generate_lines(line_count) ===

~ temp current_line = 1
- (generate_lines_loop)

Línea generada {current_line}. Especial chars: áéíóúñüÁÉÍÓÚÑÜ¡¿

~ current_line = current_line + 1
{ current_line <= line_count:
    -> generate_lines_loop
}

->->

=== generate_long_lines(line_count) ===

~ temp current_line = 1
- (generate_lines_loop)

{current_line}. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip at ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

~ current_line = current_line + 1
{ current_line <= line_count:
    -> generate_lines_loop
}

->->


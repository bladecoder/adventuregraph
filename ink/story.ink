->start

=== start ===
This line has no modifier and should use the default color.
This line uses the named color red. #color:red
This line uses a hexadecimal color with a space after the colon. #color: ff0000
>cls
After the clear, this line should be the only story text still on screen.
This line switches to blue. #color:blue
>cls
This line remains after the second clear.

>set defaultcolor=red
This untagged line inherits red text.
This line overrides the text color for one line. #color:yellow
This untagged line is red again.
>set defaultbgcolor=green
This line inherits red text on a green background.
This line overrides only the background. #bgcolor: red
This line overrides both colors. #color:white #bgcolor:0000ff
>set defaultcolor=yellow defaultbgcolor=blue
Both defaults can be changed with attributes in one command.
>cls
Clear preserves the default yellow text and blue background.
>set defaultcolor=reset defaultbgcolor=reset
This line restores the terminal's default text and background colors.

This line is centered. #align:center
This line is right-aligned. #align: right
This line is explicitly left-aligned. #align:left
This untagged line uses left alignment again.
Centered text can also have colors. #align:center #color:yellow #bgcolor:blue

* [Replay the example] -> start
* [Finish] -> END

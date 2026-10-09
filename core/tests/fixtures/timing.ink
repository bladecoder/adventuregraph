>wait time=1
á🙂
>set text-animation=false defaultcolor=red
instant #align:center
>wait time=2
>cls
>wait time=0
after
>set text-animation=true defaultbgcolor=green
slow
>set text-animation=false
last
>wait time=1
* [Continue]
  >wait time=1
  continued
  -> END
* [Invalid boolean]
  >set text-animation=false
  hidden
  >set text-animation=maybe
  -> END
* [Invalid wait]
  >set text-animation=false
  hidden
  >wait time=-1
  -> END

>set text-animation=false
>buzzer frequency=1200 duration=100
The terminal accepts the card.
>wait time=1
>buzzer pattern=phone
* [Stop]
    >buzzer stop
    -> END
* [Invalid command]
    >buzzer stop
    >buzzer frequency=0 duration=100
    -> END

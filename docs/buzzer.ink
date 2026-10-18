// Standalone command example. Include visible descriptions for silent play.
>set text-animation=false
>buzzer frequency=1200 duration=100
The terminal accepts the card.
>wait time=1

>buzzer pattern=phone
The phone rings twice.
>wait time=2

* [Answer the phone]
    >buzzer stop
    The line goes quiet.
    -> END
* [Enter an incorrect code]
    >buzzer pattern=error
    The terminal rejects the code.
    >wait time=1
    -> END
* [Trigger the alarm]
    >buzzer pattern=alarm
    The alarm sounds.
    >wait time=2
    >buzzer stop
    -> END

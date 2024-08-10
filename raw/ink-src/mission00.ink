// INTRO: Agent 1 has to wake up Agent 2

=== mission00 ===

= agent1

-(loop)

+ Agent1 option
+ Terminar agent1
    ->END

- ->loop

= agent2

-(loop)

+ Agent2 option
+ Terminar agent2
    ->END

- ->loop

->END
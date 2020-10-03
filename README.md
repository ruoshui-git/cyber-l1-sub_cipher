# Things to know

Decoder will try caesar cipher first. If that doesn't work, it will try breaking it as a simple substitution cipher.

Simple sub is tested by hill climbing. Default num of hills = 500; provide optional argument to specify num hills

For sample3, nhill=1500 is good

For sample6, nhill=2500 is pretty good

Good means message is almost readable, requiring only a few substitution that a person can probably do in mind

But those are still just based on chance. You might get lucky.
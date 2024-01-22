# Questions

## Why borrowing?
In this lab, we added almost no new capabilities to our program, but instead just moved things around and changed some types.
What benefits does this lab bring?
Consider factors like heap allocations and copying data around.

### Response
This lab is mainly a performance optimization. The main difference that we created by adding the guard struct into our progam
was that now, we are not allocating frames on the heap every time we want to parse them from our big array of bytes.
Additionally, we are not copying the data from the big array of bytes and having to put it on the heap. 
Copying and heap allocating both slow down our runtime, so by removing these two steps, we are saving a lot
of time when we run our program.

## What is a guard?
In this lab, we introduced the concept of _guards_.
In your own words, what do guards allow us to do?
More importantly, what do they (helpfully) prevent us from doing?

### Response
Guards allow us to parse a frame from our big array of bytes in the system and mutate it without copying the frame
and allocating space for it on the heap in way that absolves us of the potential danger of reading too many frames 
at once and mutating the data that we are currently referencing. The guard type acts as a sort of lock on the specific section
of bytes that then is cleaned up once the guard is dropped. The guard won't allow us to overwrite/read the data that it
is referencing while it is in place. 
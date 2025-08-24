# stm32f1-hal
Because the design of [stm32f1xx-hal](https://github.com/stm32-rs/stm32f1xx-hal) is unsuitable for my needs and [stm32-hal](https://github.com/David-OConnor/stm32-hal) doesn't support the F1 series, I decided to write a new crate.

Many codes come from [stm32f1xx-hal](https://github.com/stm32-rs/stm32f1xx-hal).

## Design
Below are the design principles.
1. Readability is the most important.
    - We only write code a few times, but we read it countless times. Moreover, understanding the code is a necessary condition for maintaining it.
2. Conciseness does not equal simplicity.
    - Fewer lines of code mean conciseness, but being easy to understand is simplicity.

Therefore, if a module is quite complex, I would not use a macro + generic approach, as it is too difficult to read.

Instead, I use a script to generate code for GPIO alternate function remapping and a synchronization script to manage duplicate code across peripherals.

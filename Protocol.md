# P2P Protocol

## Communication

All communications start with 4 big endian bytes communicating the size of the message in bytes. The size does not include the size of itself, so a the message `00 00 00 01 01` is only listed as 1 byte.
After the first 4 bytes, there is another 'instruction' byte that communicates the action.
Any remaining bytes are 'arguments' of the action


| Name | Instruction | Arguments | Description |
|--|--|--|--|
Server Information | 0x00 | u32 (screen size x) u32 (screen size y) | Basic information about the server for the client to function
Move Mouse | 0x10 | u32 (x position); u32 (y position) | Indicates where to move the mouse to
Click | 0x11 | u8 (button | isClicked)| the 2^1 bit determines the mouse button (0 = left, 1 = right) and the 2^0 bit determines the state (0 = released, 1 = pressed)
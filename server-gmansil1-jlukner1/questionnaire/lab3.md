# Questions

## Error Variants
Tell us about the different variants you added to `ParseError`.
What is the purpose of each, and in what situations do each of them occur?
Since we were intentionally vague about what variants you might need to add, we want your insight here since we don't necessarily anticipate it'll line up exactly with what we have.

### Response
The variants which we added were UTF8 error which we used in order to make sure a simple or error were valid strings. The mismatchsize error is used when the size of our bulkstring does not match the size of the passed in size.

-------------------------------------------------------------------------------

## The Type System
Prior to this lab, all functions that don't return anything (`Cookie::eat`, `main`, .etc) have no final expression, which is implicitly interpreted as returning `()`.

For example, you could write the following `main` function:

```rust
fn main() -> () {
    ()
}
```

If we're not returning anything useful on the success case of `Frame::check`, how come we need to still explicitly write `Ok(())`?
A sentence or two is fine.

### Response
We use the Ok(()) because the function returns a result which has a case for error and a case for not an error.

-------------------------------------------------------------------------------

## How are enums represented?
In class, we mentioned that enums are always stored on the stack (unless we explicitly ask for it on the heap), even though the variants might be storing completely different pieces of data.

To think about how this might be possible, consider C unions.
Here's an example of what a `union` might look like in C:
```c
union NameOrWord {
    char name[8];
    long word;
}
```
If this were a `struct`, it would probably take 16 bytes to hold both the `char[8]` and `long`.
However, a `union` is only big enough to store one of its variants, and it does this by being the size of the largest field.
In this case, `NameOrWord` would only be 8 bytes in memory, since 8 bytes is sufficient to store either a `long` or a `char[8]`.

This means that our `NameOrWord` type can only hold one field at a time, because the same memory is used for all variants and if data to one variant is written, any other data for the other variants will be overwritten.

If we write:
```c
NameOrWord x;
x.name = "William";
```
Then `x` would look like this in memory:
```
| 'W' | 'i' | 'l' | 'l' | 'i' | 'a' | 'm' | '\0' |
```
But if we assigned to its `word` field with `x.word = 0`, then it would overwrite the name we just wrote:

```
|  0  |  0  |  0  |  0  |  0  |  0  |  0  |  0  |
```

The problem with C unions is that given an instance of one, there's no easy way to check which variant the data is supposed to represent.
Now, consider another term for Rust's enums: _tagged_ unions.

The question is, how do you think Rust enums are represented under the hood?

### Response
In Rust, enums are generally implemented through a pair of a union and a number(tag) to match with. The field from the union will be used depending on the number passed in.The union lets us be memory efficient by allowing our union to be saved in memory by using the size of the largest field.

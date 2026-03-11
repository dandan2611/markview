---
title: md-tui Test File
author: Test
date: 2026-03-11
---

# Heading 1

## Heading 2

### Heading 3

#### Heading 4

##### Heading 5

###### Heading 6

This is a paragraph with **bold**, *italic*, and ~~strikethrough~~ text.

Here is some `inline code` in a sentence.

> This is a blockquote
> with multiple lines
>
> > And a nested blockquote

---

## Lists

- Item 1
- Item 2
  - Nested item A
  - Nested item B
- Item 3

1. First
2. Second
3. Third

## Task List

- [x] Completed task
- [ ] Incomplete task
- [x] Another done task

## Code Block

```rust
fn main() {
    println!("Hello, world!");
    let x = 42;
    for i in 0..x {
        println!("{}", i);
    }
}
```

```python
def greet(name):
    print(f"Hello, {name}!")

greet("World")
```

## Links

Check out [Rust](https://www.rust-lang.org) and [Ratatui](https://ratatui.rs).

## Images

![Alt text](https://example.com/image.png)

## Table

| Name | Age | City |
|------|----:|:----:|
| Alice | 30 | New York |
| Bob | 25 | San Francisco |
| Charlie | 35 | London |

## Another Table

| Feature | Status | Notes |
|---------|--------|-------|
| Headings | Done | All levels |
| Code blocks | Done | With syntax highlighting |
| Tables | Done | With alignment |
| Search | Done | `/` to search |

## Footnotes

This sentence has a footnote[^1] and another one[^note].

A third reference to the first footnote[^1] again.

[^1]: This is the first footnote definition.
[^note]: This is a named footnote with more detail.

## Math

Euler's identity: $e^{i\pi} + 1 = 0$ is elegant.

The quadratic formula is also inline: $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$.

A display math block:

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

## Smart Punctuation

"Curly quotes" and 'single quotes' --- em-dashes -- en-dashes... and ellipsis.

## Heading with Attributes {#custom-id}

This heading above uses the `{#custom-id}` attribute syntax.

That's all folks!

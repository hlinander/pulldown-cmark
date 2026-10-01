use pulldown_cmark::{Event, Options, Parser};

const MATHJAX: Options = Options::ENABLE_MATHJAX_MATH;

fn math(source: &str, options: Options) -> Vec<(bool, String)> {
    Parser::new_ext(source, options)
        .filter_map(|event| match event {
            Event::InlineMath(s) => Some((true, s.into_string())),
            Event::DisplayMath(s) => Some((false, s.into_string())),
            _ => None,
        })
        .collect()
}

#[test]
fn explicit_delimiters_and_literal_currency() {
    let source = r"$3–$9, $x$, \( x_1 + \frac{a}{b} \), \[x^2\], $$y^2$$";
    assert_eq!(
        math(source, MATHJAX),
        [
            (true, r" x_1 + \frac{a}{b} ".into()),
            (false, "x^2".into()),
            (false, "y^2".into())
        ]
    );
    let text: String = Parser::new_ext(source, MATHJAX)
        .filter_map(|event| {
            if let Event::Text(text) = event {
                Some(text.into_string())
            } else {
                None
            }
        })
        .collect();
    assert!(text.contains("$3–$9, $x$, "), "{text}");
}

#[test]
fn existing_math_option_keeps_dollar_syntax() {
    assert_eq!(
        math(r"$x$ $$y$$ \(z\) \[w\]", Options::ENABLE_MATH),
        [(true, "x".into()), (false, "y".into())]
    );
    assert!(math(r"$x$ $$y$$ \(z\) \[w\]", Options::empty()).is_empty());
}

#[test]
fn preserves_tex_and_source_offsets() {
    let source = r"π \(a_b * c_d + \text{\$3}\) \[\begin{matrix}a&b\\c&d\end{matrix}\]";
    let formulas: Vec<_> = Parser::new_ext(source, MATHJAX)
        .into_offset_iter()
        .filter_map(|(event, span)| match event {
            Event::InlineMath(s) | Event::DisplayMath(s) => Some((s.into_string(), &source[span])),
            _ => None,
        })
        .collect();
    assert_eq!(
        formulas,
        [
            (
                r"a_b * c_d + \text{\$3}".into(),
                r"\(a_b * c_d + \text{\$3}\)"
            ),
            (
                r"\begin{matrix}a&b\\c&d\end{matrix}".into(),
                r"\[\begin{matrix}a&b\\c&d\end{matrix}\]"
            )
        ]
    );
}

#[test]
fn code_html_and_urls_are_not_math() {
    let source = r###"`\(x\) $$y$$` <https://a/$b$c>

```tex
\(x\) $$y$$
```

    \[x\]

<div title="\(x\)">$$y$$</div>

[plain](https://a/$b$c "\(x\)")

[ref]: https://a/$b$c "\(x\)"

[ref]
"###;
    assert!(math(source, MATHJAX).is_empty());
    let links: Vec<_> = Parser::new_ext(source, MATHJAX)
        .filter_map(|event| {
            if let Event::Start(pulldown_cmark::Tag::Link { dest_url, .. }) = event {
                Some(dest_url.into_string())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        links,
        ["https://a/$b$c", "https://a/$b$c", "https://a/$b$c"]
    );
    assert_eq!(
        math(r"[\(x\)](https://a/$b$c)", MATHJAX),
        [(true, "x".into())]
    );
}

#[test]
fn escaped_unmatched_and_mismatched_delimiters() {
    for source in [
        r"\\(x\\)",
        r"\\[x\\]",
        r"\$\$x\$\$",
        r"\(x",
        r"\[x",
        r"$$x",
        r"\(x\]",
        r"\[x$$",
    ] {
        assert!(math(source, MATHJAX).is_empty(), "{source}");
    }
    assert_eq!(
        math(r"\(a \] b\) $$c \) d$$", MATHJAX),
        [(true, r"a \] b".into()), (false, r"c \) d".into())]
    );
    assert_eq!(math("\\(x\n\ny\\)", MATHJAX), []);
}

#[test]
fn containers_and_table_cells_preserve_math() {
    let source = "> \\[\n> x_1 + x_2\n> \\]\n\n- \\(y\\)\n\n| Math |\n|---|\n| \\(a\\|b\\) |\n";
    assert_eq!(
        math(source, MATHJAX | Options::ENABLE_TABLES),
        [
            (false, "\nx_1 + x_2\n".into()),
            (true, "y".into()),
            (true, "a|b".into())
        ]
    );
}

#[test]
fn repeated_unmatched_openers() {
    let source = r"\( ".repeat(20_000);
    assert!(math(&source, MATHJAX).is_empty());
}

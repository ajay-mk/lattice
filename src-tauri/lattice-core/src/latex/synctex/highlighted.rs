//! The source line of code a package typeset from a file it generated.
//!
//! minted has Pygments write each code block, highlighted, to a file in its
//! cache folder (`_minted/<hash>.highlight.minted`, minted 2.x's
//! `_minted-<job>/<hash>.pygtex`) and inputs that file in place of the block.
//! SyncTeX records the code there, so `synctex edit` on a code line names the
//! generated file, and nothing the writer can edit: TeX recorded no line of
//! the block itself. The block is found again by its text instead.

/// Every place in `sources` (path and text) that line `line` of
/// `highlighted`, a minted cache file, may have been highlighted from: each
/// run of lines holding the same code as the whole block, and the clicked
/// line's place in it, marked when the run opens an environment (the nearest
/// line with text above it has a `\begin{`), as a code block does. Empty
/// when no source holds that code (an `\inputminted` file, or code `gobble`
/// changed). minted names the file by the code, so blocks holding the same
/// code share it, and only where the click was tells them apart.
pub(super) fn source_lines<P: Clone>(
    highlighted: &str, line: u32, sources: impl IntoIterator<Item = (P, String)>,
) -> Vec<(P, u32, bool)> {
    let lines: Vec<&str> = highlighted.lines().collect();
    // Pygments wraps the block in a verbatim environment (`Verbatim`, minted
    // 3's `MintedVerbatim`) on lines of their own.
    let begin = usize::from(lines.first().is_some_and(|text| text.starts_with("\\begin{")));
    let end =
        lines.len() - usize::from(lines.last().is_some_and(|text| text.starts_with("\\end{")));
    let Some(block) = lines.get(begin..end.max(begin)) else { return Vec::new() };
    let block: Vec<String> = block.iter().map(|text| highlighted_text(text)).collect();
    // Code with no letter or digit (braces alone) would match anywhere.
    if block.iter().all(String::is_empty) {
        return Vec::new();
    }
    let offset = (line as usize).saturating_sub(begin + 1).min(block.len() - 1);
    let mut found = Vec::new();
    for (path, source) in sources {
        let source: Vec<&str> = source.lines().collect();
        let code: Vec<String> = source.iter().map(|text| code_text(text)).collect();
        for (start, run) in code.windows(block.len()).enumerate() {
            if run != block.as_slice() {
                continue;
            }
            let opened = source[..start]
                .iter()
                .rev()
                .find(|text| !text.trim().is_empty())
                .is_some_and(|text| text.contains("\\begin{"));
            found.push((path.clone(), (start + offset + 1) as u32, opened));
        }
    }
    found
}

/// The letters and digits of a line of source: what survives highlighting
/// unchanged. Pygments escapes punctuation and spaces (`\PYGZob{}` for `{`).
fn code_text(line: &str) -> String {
    line.chars().filter(|character| character.is_alphanumeric()).collect()
}

/// The letters and digits of a line Pygments highlighted, without its
/// markup: `\PYG{k}{def}` (a token and its style) is `def`, and an escape
/// such as `\PYGZbs{}` (a backslash) has none. Any other command came from
/// the code itself (an `escapeinside` escape), so it counts as it is written.
fn highlighted_text(line: &str) -> String {
    let mut text = String::new();
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\\' {
            if character.is_alphanumeric() {
                text.push(character);
            }
            continue;
        }
        let mut name = String::new();
        while let Some(letter) = characters.next_if(char::is_ascii_alphabetic) {
            name.push(letter);
        }
        if !name.starts_with("PYG") {
            text.push_str(&name);
        } else if !name.starts_with("PYGZ") && characters.next_if_eq(&'{').is_some() {
            // The token's style (`k`, `n+nf`) is markup, not code.
            characters.by_ref().find(|character| *character == '}');
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIGHLIGHTED: &str = "\\begin{Verbatim}[commandchars=\\\\\\{\\}]\n\
\\PYG{k}{def}\\PYG{+w}{ }\\PYG{n+nf}{f}\\PYG{p}{(}\\PYG{n}{x}\\PYG{p}{):}\n\
    \\PYG{k}{return} \\PYG{n}{x} \\PYG{o}{+} \\PYG{l+m+mi}{1}\n\
\n\
\\PYG{n+nb}{print}\\PYG{p}{(}\\PYG{l+s+s2}{\\PYGZdq{}}\\PYG{l+s+se}{\\PYGZbs{}n}\\PYG{l+s+s2}{\\PYGZdq{}}\\PYG{p}{)}\n\
\\end{Verbatim}\n";

    const MAIN: &str = "\\documentclass{article}\n\\usepackage{minted}\n\\begin{document}\n\
Before.\n\n\\begin{minted}{python}\ndef f(x):\n    return x + 1\n\nprint(\"\\n\")\n\
\\end{minted}\n\nAfter.\n\\end{document}\n";

    fn find(line: u32, sources: &[(&'static str, &str)]) -> Vec<(&'static str, u32, bool)> {
        source_lines(
            HIGHLIGHTED,
            line,
            sources.iter().map(|(path, text)| (*path, text.to_string())),
        )
    }

    #[test]
    fn a_highlighted_code_line_lands_on_its_line_in_the_block() {
        // Cache line 2 is the block's first line of code, source line 7.
        assert_eq!(find(2, &[("main.tex", MAIN)]), [("main.tex", 7, true)]);
        assert_eq!(find(3, &[("main.tex", MAIN)]), [("main.tex", 8, true)]);
        assert_eq!(find(5, &[("main.tex", MAIN)]), [("main.tex", 10, true)]);
        // The environment's own lines land on the code's first and last line.
        assert_eq!(find(1, &[("main.tex", MAIN)]), [("main.tex", 7, true)]);
        assert_eq!(find(6, &[("main.tex", MAIN)]), [("main.tex", 10, true)]);
    }

    #[test]
    fn finds_the_block_wherever_a_source_holds_its_code() {
        let chapter = MAIN.replace("x + 1", "x + 2");
        let sources = [("chapter.tex", chapter.as_str()), ("main.tex", MAIN)];
        assert_eq!(find(3, &sources), [("main.tex", 8, true)]);
        // Code no source holds is not guessed at.
        assert_eq!(find(3, &[("chapter.tex", chapter.as_str())]), []);
        // The same code twice is found twice; outside a block, it is marked so.
        let twice = format!("{MAIN}% def f(x):\n%    return x + 1\n%\n% print(\"\\n\")\n");
        assert_eq!(
            find(3, &[("main.tex", &twice)]),
            [("main.tex", 8, true), ("main.tex", 16, false)]
        );
    }

    #[test]
    fn strips_pygments_markup_and_escapes() {
        assert_eq!(highlighted_text("\\PYG{n+nf}{f}\\PYG{p}{(}\\PYG{n}{x}"), "fx");
        assert_eq!(highlighted_text("\\PYG{l+s+se}{\\PYGZbs{}n}\\PYGZob{}"), "n");
        // A command the code itself holds is part of the code.
        assert_eq!(highlighted_text("\\PYG{n}{a} \\textbf{b}"), "atextbfb");
        assert_eq!(code_text("print(\"\\n\") # {x}"), "printnx");
    }
}

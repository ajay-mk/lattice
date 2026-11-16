import { describe, expect, it } from "vitest";
import { isBuildOutput } from "./build-inputs";

describe("isBuildOutput", () => {
  it("knows what a LaTeX run writes beside its sources", () => {
    for (const path of [
      "main.pdf", "main.aux", "main.log", "main.fls", "main.fdb_latexmk", "main.synctex.gz", "main.synctex.gz(busy)",
      "main.out", "main.toc", "main.bbl", "main.blg", "main-blx.bib", "acs-main.bib", "paper/acs-thesis.bib", "main.run.xml", "chapters/intro.aux",
      "_minted-main/default.pygstyle", "_minted/5F2A1B.highlight.minted", "chapters/_minted-intro/0A1B2C.pygtex",
      "figures/svg-inkscape/plot_svg-tex.pdf", "_minted-main", ".git/index", "paper/thesis.pdf",
      ".lattice-0b7e4c1a-9f3d-4e2b-8a6c-5d1f2e3a4b5c.tmp", "chapters/.lattice-0b7e4c1a-9f3d-4e2b-8a6c-5d1f2e3a4b5c.tmp",
      ".lattice-0b7e4c1a-9f3d-4e2b-8a6c-5d1f2e3a4b5c.tmp.bib",
    ]) {
      expect(isBuildOutput(path, ["main.tex", "paper/thesis.tex"]), path).toBe(true);
    }
  });

  it("treats every source, and anything unfamiliar, as an input", () => {
    for (const path of [
      "main.tex", "main.bib", "references.bib", "acs-refs.bib", "acs-thesis.bib", "chapters/intro.tex", "figures/plot.pdf", "macros.sty",
      "data/table.csv", "notes.md", "thesis.pdf", "kept.bbl", "chapters", "../outside.aux", "draft.tmp", "lattice-notes.tmp",
      "_minted_snippets/hello.py", "svg-inkscape-notes/readme.md",
    ]) {
      expect(isBuildOutput(path, ["main.tex", "paper/thesis.tex"]), path).toBe(false);
    }
  });
});

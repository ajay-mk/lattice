//! Folders a package fills while compiling (minted's cache, the svg
//! package's conversions): build output, wherever they land in a project.

/// A folder a package fills while compiling, through a program it runs with
/// shell escape (Pygments, Inkscape). TeX's `-output-directory` does not
/// reach that program, so the folder lands beside the sources even with
/// Settings › Hide auxiliary files on (`latex/build/beside.rs`), and
/// Overleaf, which compiles in an output folder of its own, keeps its copy
/// out of the project. It is build output: the tree hides it, Overleaf sync
/// leaves it out in both directions (`is_excluded` in `overleaf/files.rs`),
/// and GPUI's watcher does not build again for it (`build/inputs.rs`). The
/// Tauri app's watcher keeps a copy of this table, aligned by hand:
/// `GENERATED_FOLDER` in `src/build/build-inputs.ts`.
///
/// Never deleted as output, though: a project may keep the cache on purpose
/// (minted's `frozencache`, svg conversions made where Inkscape was
/// installed), and nothing here can write it again.
struct PackageCache {
    /// The folder's name, also matched as `<folder>-<jobname>` when
    /// `per_job`. Matched exactly otherwise: a writer's `_minted_snippets/`,
    /// read by `\inputminted`, is a source.
    folder: &'static str,
    per_job: bool,
}

const PACKAGE_CACHES: &[PackageCache] = &[
    // minted's Pygments output: `_minted-<jobname>/` (2.x), `_minted/` (3.x).
    PackageCache { folder: "_minted", per_job: true },
    // The svg package's Inkscape conversions (`plot_svg-tex.pdf`).
    PackageCache { folder: "svg-inkscape", per_job: false },
];

fn is_package_cache_folder(name: &str) -> bool {
    PACKAGE_CACHES.iter().any(|cache| {
        name.strip_prefix(cache.folder).is_some_and(|rest| {
            rest.is_empty() || (cache.per_job && rest.len() > 1 && rest.starts_with('-'))
        })
    })
}

/// Whether `relative` (forward slashes) is a package's cache folder
/// ([`PACKAGE_CACHES`]) or lies inside one, at any depth. `is_folder` says
/// whether its last name is a folder's: a writer's `_minted` file is not.
pub fn in_package_cache_folder(relative: &str, is_folder: bool) -> bool {
    let folders = if is_folder {
        Some(relative)
    } else {
        relative.rsplit_once('/').map(|(folder, _)| folder)
    };
    folders.is_some_and(|folders| folders.split('/').any(is_package_cache_folder))
}

#[cfg(test)]
mod tests {
    use crate::project::test_support::Fixture;
    use crate::project::{project_tree_path_visible, scan_tree, tree_files, TreeView};
    use std::path::Path;

    #[test]
    fn package_caches_are_build_output_at_any_depth() {
        let fixture = Fixture::empty("minted-cache");
        let root = &fixture.root;
        for path in [
            "main.tex",
            "_minted-main/default.pygstyle",
            "_minted/5F2A1B.highlight.minted",
            "chapters/_minted-intro/0A1B2C.pygtex",
            "chapters/intro.tex",
            "_minted_listing.tex",
            "code/minted/example.py",
            "_minted_snippets/hello.py",
            "_minted-/stray.tex",
            "figures/svg-inkscape/plot_svg-tex.pdf",
            "svg-inkscape-notes/readme.md",
        ] {
            fixture.write(path, b"content");
        }
        let listed = |view| -> Vec<String> {
            let nodes = scan_tree(root, view).unwrap();
            tree_files(&nodes).into_iter().map(|file| file.path.clone()).collect()
        };
        // Tree order: folders first.
        let sources = [
            "_minted-/stray.tex",
            "_minted_snippets/hello.py",
            "chapters/intro.tex",
            "code/minted/example.py",
            "svg-inkscape-notes/readme.md",
            "_minted_listing.tex",
            "main.tex",
        ];
        assert_eq!(listed(TreeView::Inventory), sources);
        assert_eq!(listed(TreeView::Project), sources);
        // The expanded view shows it with the rest of the build output.
        assert!(
            listed(TreeView::ProjectWithHidden).contains(&"_minted-main/default.pygstyle".into())
        );
        assert!(!project_tree_path_visible(
            root,
            Path::new("chapters/_minted-intro/0A1B2C.pygtex")
        ));
        assert!(!project_tree_path_visible(root, Path::new("_minted-main")));
        assert!(project_tree_path_visible(root, Path::new("_minted_listing.tex")));
        assert!(!project_tree_path_visible(root, Path::new("figures/svg-inkscape")));
    }
}

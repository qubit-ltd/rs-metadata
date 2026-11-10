#!/usr/bin/env python3
"""Focused tests for independent user-guide consumer verification."""

import importlib.util
import pathlib
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).with_name("check-user-guide-consumers.py")
SPEC = importlib.util.spec_from_file_location("user_guide_consumers", SCRIPT)
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


class FencedBlockTests(unittest.TestCase):
    def test_extracts_one_complete_block(self):
        self.assertEqual(
            CHECKER.fenced_block("<!-- sample:start -->\n```rust\nfn main() {}\n```\n<!-- sample:end -->", "sample", "rust"),
            "fn main() {}\n",
        )

    def test_rejects_missing_duplicate_reverse_and_multiple_blocks(self):
        malformed = {
            "missing": "text",
            "duplicate": "<!-- sample:start -->\n```rust\na\n```\n<!-- sample:end -->\n<!-- sample:start -->\n```rust\nb\n```\n<!-- sample:end -->",
            "reverse": "<!-- sample:end -->\n```rust\na\n```\n<!-- sample:start -->",
            "multiple": "<!-- sample:start -->\n```rust\na\n```\n```rust\nb\n```\n<!-- sample:end -->",
        }
        for label, document in malformed.items():
            with self.subTest(label=label), self.assertRaisesRegex(ValueError, "sample"):
                CHECKER.fenced_block(document, "sample", "rust")

    def test_execute_adds_document_name_to_marker_errors(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            (root / "README.md").write_text("", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, r"README\.md: expected exactly one ordered readme-json-roundtrip-example region"):
                CHECKER.execute(root, [], run=lambda *_args, **_kwargs: None)


class ConsumerMatrixTests(unittest.TestCase):
    def test_has_nine_paired_cases_bound_to_declared_documents(self):
        cases = CHECKER.CASES
        self.assertEqual(len(cases), 18)
        self.assertEqual(
            {case[1] for case in cases},
            {
                "readme-json-roundtrip-example", "core-read-example", "chunk-workflow-example",
                "empty-set-example", "query-translation-example", "bounded-json-decoding-example",
                "directional-json-limits-example", "bounded-json-encoding-example",
                "bounded-filter-json-encoding-example",
            },
        )
        self.assertEqual({case[0] for case in cases}, {
            "README.md", "README.zh_CN.md", "doc/user_guide.md", "doc/user_guide.zh_CN.md"
        })
        for code_marker in {case[1] for case in cases}:
            with self.subTest(code_marker=code_marker):
                self.assertEqual(sum(case[1] == code_marker for case in cases), 2)
                documents = {case[0] for case in cases if case[1] == code_marker}
                expected_documents = (
                    {"README.md", "README.zh_CN.md"}
                    if code_marker == "readme-json-roundtrip-example"
                    else {"doc/user_guide.md", "doc/user_guide.zh_CN.md"}
                )
                self.assertEqual(documents, expected_documents)

    def test_real_document_matrix_has_one_rust_and_toml_block_per_case(self):
        repository = SCRIPT.parents[2]
        for document_name, code_marker, dependency_marker in CHECKER.CASES:
            with self.subTest(document=document_name, code=code_marker, dependencies=dependency_marker):
                document = (repository / document_name).read_text(encoding="utf-8")
                self.assertTrue(CHECKER.fenced_block(document, code_marker, "rust"))
                self.assertIn("[dependencies]", CHECKER.fenced_block(document, dependency_marker, "toml"))

    def test_each_consumer_uses_document_content_and_is_independent(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            (root / "doc").mkdir()
            for name in ("README.md", "README.zh_CN.md", "doc/user_guide.md", "doc/user_guide.zh_CN.md"):
                (root / name).write_text("placeholder", encoding="utf-8")
            documents = {
                "README.md": "readme-json-roundtrip-example readme-json-dependencies",
                "README.zh_CN.md": "readme-json-roundtrip-example readme-json-dependencies",
                "doc/user_guide.md": "core-read-example core-dependencies chunk-workflow-example installation-dependencies empty-set-example query-translation-example filter-dependencies bounded-json-decoding-example bounded-json-decode-dependencies directional-json-limits-example budget-json-dependencies bounded-json-encoding-example bounded-filter-json-encoding-example filter-json-dependencies",
                "doc/user_guide.zh_CN.md": "core-read-example core-dependencies chunk-workflow-example installation-dependencies empty-set-example query-translation-example filter-dependencies bounded-json-decoding-example bounded-json-decode-dependencies directional-json-limits-example budget-json-dependencies bounded-json-encoding-example bounded-filter-json-encoding-example filter-json-dependencies",
            }
            code = "fn main() { println!(\"from-doc\"); }"
            deps = '[dependencies]\nqubit-metadata = { version = "0.7", features = ["json"] }\n'
            for name in documents:
                text = []
                for marker in documents[name].split():
                    if marker.endswith("dependencies"):
                        text.append(f"<!-- {marker}:start -->\n```toml\n{deps}```\n<!-- {marker}:end -->")
                    else:
                        text.append(f"<!-- {marker}:start -->\n```rust\n{code}\n```\n<!-- {marker}:end -->")
                (root / name).write_text("\n".join(text), encoding="utf-8")
            calls = []
            def fake_run(args, **kwargs):
                manifest = pathlib.Path(args[args.index("--manifest-path") + 1])
                calls.append((args, kwargs, manifest.read_text(encoding="utf-8"), (manifest.parent / "src/main.rs").read_text(encoding="utf-8")))
            CHECKER.execute(root, ["--locked"], run=fake_run)
            self.assertEqual(len(calls), 18)
            manifests = [pathlib.Path(args[args.index("--manifest-path") + 1]) for args, _, _, _ in calls]
            self.assertEqual(len(set(manifest.parent for manifest in manifests)), 18)
            for args, kwargs, content, main_rs in calls:
                self.assertEqual(content.split("\n\n", 1)[1], deps)
                self.assertNotIn("[dev-dependencies]", content)
                self.assertNotIn("--all-features", args)
                self.assertIn("from-doc", main_rs)
                self.assertTrue(kwargs["check"])
                self.assertIn("CARGO_TARGET_DIR", kwargs["env"])

    def test_preserves_cli_args_path_patch_and_propagates_cargo_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            for name in ("README.md", "README.zh_CN.md"):
                (root / name).write_text(
                    '<!-- readme-json-dependencies:start -->\n```toml\n[dependencies]\nqubit-metadata = "0.7"\n```\n<!-- readme-json-dependencies:end -->\n'
                    '<!-- readme-json-roundtrip-example:start -->\n```rust\nfn main() {}\n```\n<!-- readme-json-roundtrip-example:end -->',
                    encoding="utf-8",
                )
            calls = []
            def fail(args, **kwargs):
                calls.append(args)
                raise __import__("subprocess").CalledProcessError(9, args)
            with self.assertRaises(__import__("subprocess").CalledProcessError):
                CHECKER.execute(root, ["--offline", "--locked"], run=fail)
            args = calls[0]
            self.assertEqual(args[1:3], ["--offline", "--locked"])
            self.assertIn("--config", args)
            self.assertTrue(any("patch.crates-io.qubit-metadata.path=" in arg for arg in args))


if __name__ == "__main__":
    unittest.main()

# Coding style and best practices

## Basic rules:
- Use full, descriptive names for variables, functions, types, and modules. NEVER shorten the names, as it makes it extremely hard to read and review.
- Functions should be small and avoid deep nesting, well-structured and documented. The new developer onboarding to the project should be able to easily grasp what the code does and why.
- Group all `use` imports at the top of the file (no local import islands). Prefer ordering: std → third-party → workspace crates.
- Do not define structs/enums/traits inside function bodies. Keep types at module/file scope unless there is a compelling, strictly local reason.
- Use of `unwrap` is discouraged except in tests or in cases where failure is impossible (e.g., after checking `Option::is_some`). Prefer using `?` to propagate errors or handle them gracefully. Log errors using tracing crate macros (e.g., `tracing::error!`, `tracing::warn!`) instead of panicking, unless it's a critical, unrecoverable error.
- When fixing a bug, first write a test that reproduces the bug, then fix the bug, and finally ensure the test passes.
- Avoid dynamic dispatch wherever possible. Use generics and `impl Trait` instead of `Box<dyn Trait>`/`Arc<dyn Trait>`.
- All e2e tests must be as watertight as possible.
- No global state whatsoever. It makes reliable and quick testing a very hard task.
- To compile, lint, and run tests run `./scripts/check.sh`. CI is likely to fail if this script fails. You MUST run this script. PRs that fail this check will not be accepted.
- Avoid pyramids of doom and deep nesting, break down things into small, readable functions.
- Make sure comments are short and descriptive, don't contain the thought process or plans - they should strictly clarify parts that might be unobvious from surrounding code, nothing more.
- Use `thiserror` and derive `thiserror::Error` for error types.
- Try to keep modules small and focused. If you notice that your module starts to grow over a thousand lines, consider 
splitting it into smaller modules. Don't overdo it though. Don't break up things that can't be broken up.

## Performance

### Collections
- Try to avoid unnecessary allocations as much as possible. Keep reusable collections on long-lived structs and
clear them once they are no longer needed.
- Don't use `mem::take` on reusable collections.
- If keeping a reusable collection is not possible, and you certain that you need a small collection on the hot path,
use `SmallVec` instead of `Vec`.

## Testing
- Group tests by functionality. Don't put all tests in one file.

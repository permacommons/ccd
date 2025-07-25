# Guidelines for Human and AI Contributors

## Welcome! 🤖👋

Contributions from well-behaved humans and AI models are welcome. :)

## In a nutshell

`ccd-pick` uses `locate` to find directories on the filesystem. In interactive mode, it also logs picks in `~/.ccd_frequency`. Those are used in the search suggestions, and in an alternative "Frequently used" mode.

Because a subprocess can't change the shell working directory, we need to wrap `ccd-pick` in a shell wrapper.

Look at `src/main.rs` for the main Rust source, `ccd.sh` for the shell wrapper function, and `README.md` for more.

## Operating

As an agent, rely on the human operator to test interactive mode. By all means, run `cargo build`, though!

## Core Principles

### 1. Usability and Accessibility First
Even though this is a CLI tool, we consider usability and accessibility carefully. Every interaction should be intuitive and inclusive.

### 2. Test-Driven Development
To aid the maintainability of the codebase, for every change, consider how it could be tested automatically. Update tests alongside your changes.

### 3. Documentation Consistency
Make sure `README.md`, code comments and CLI help are still accurate after your change. Documentation should reflect the current state of the code.

## Contribution Guidelines

- Write clear, maintainable code
- Include appropriate tests for new functionality
- Update documentation when making changes
- Follow existing code patterns and conventions
- Consider the user experience in all changes

Thank you for helping make this project better!
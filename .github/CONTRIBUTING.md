# Contributing to Kokoro Engine / 贡献指南

Thanks for your interest in contributing! / 感谢你对贡献的兴趣！

## Getting Started / 开始

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/<your-username>/Kokoro-Engine.git
   cd Kokoro-Engine
   ```
3. Install dependencies:
   ```bash
   npm install
   ```
4. Start the dev environment:
   ```bash
   npm run tauri dev
   ```

## Development / 开发

### Prerequisites / 前置要求

- **Node.js** >= 18
- **Rust** >= 1.75 (with `cargo`)
- **Tauri CLI** (`npm install -g @tauri-apps/cli`)

### Project Structure / 项目结构

- `src/` — Frontend (React + TypeScript)
- `src-tauri/` — Backend (Rust / Tauri)
- `mods/` — MOD system modules

### Before Submitting / 提交前检查

```bash
# Frontend type check / 前端类型检查
npx tsc --noEmit

# Rust compile check / Rust 编译检查
cd src-tauri && cargo check

# Rust lint
cd src-tauri && cargo clippy
```

## Commit Convention / 提交规范

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type: description
```

| Type | Usage |
|------|-------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code refactoring (no behavior change) |
| `docs` | Documentation only |
| `style` | UI/CSS changes |
| `test` | Adding or updating tests |
| `chore` | Build, tooling, dependencies |

Commit messages should be in **English**.

## Pull Requests / 拉取请求

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feat/my-feature
   ```
2. Make your changes with clear, focused commits
3. Ensure all checks pass (TypeScript, Cargo)
4. Open a PR against `main` using the PR template
5. Describe your changes and link related issues

## Issues

- Use the **Bug Report** template for bugs
- Use the **Feature Request** template for suggestions
- You can write in English or Chinese / 可以使用英文或中文

## Code Style / 代码风格

- **TypeScript**: Follow existing patterns, use typed IPC via `kokoro-bridge.ts`
- **Rust**: `cargo clippy` clean, use `Arc<RwLock<T>>` for shared state
- **CSS**: Use CSS variables from the theme system (`var(--color-*)`)

## License / 许可

By contributing, you agree that your contributions will be licensed under the same license as the project.

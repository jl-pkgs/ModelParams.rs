#import "@preview/modern-cug-report:0.1.3": *
#show: doc => template(doc, footer: "CUG水文气象学2025", header: "", size:11pt)

#set par(leading: 1em, spacing: 1em)

= 1 Rust Workspace 多项目管理

Cargo *workspace* 是 Rust 管理多个相关 crate 的机制。
一个 workspace 下所有成员共享同一个 `Cargo.lock` 和 `target/` 编译目录，
避免重复编译依赖，同时保持各 crate 独立发布。

== 1.1 目录结构

以本项目为例：

```
rust/                          ← workspace 根目录
├── Cargo.toml                 ← workspace 配置（不是 crate）
├── Cargo.lock                 ← 全局锁文件（只有一份）
├── target/                    ← 所有成员共享的编译输出
│
├── modelparams/               ← 主库 crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── gof.rs
│       └── optim/
│           ├── mod.rs
│           ├── sceua.rs
│           └── cceua.rs
│
└── modelparams-derive/        ← proc macro crate（独立编译）
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

== 1.2 Workspace 根 `Cargo.toml`

根目录的 `Cargo.toml` 只声明成员，本身不是一个 crate：

```toml
[workspace]
members = [
    "modelparams",
    "modelparams-derive",
]
resolver = "2"          # 推荐：独立解析 feature 标志
```

`members` 是相对于根目录的路径列表，每项对应一个含有 `Cargo.toml` 的子目录。

== 1.3 成员 Crate 的 `Cargo.toml`

每个成员有自己的 `Cargo.toml`，声明自己的名称、版本、依赖：

```toml
# modelparams/Cargo.toml
[package]
name    = "modelparams"
version = "0.1.0"
edition = "2021"

[dependencies]
# 引用同 workspace 内的另一个成员用 path 依赖
modelparams-derive = { path = "../modelparams-derive" }
rayon = "1"
rand  = { version = "0.8", features = ["small_rng"] }
```

```toml
# modelparams-derive/Cargo.toml
[package]
name    = "modelparams-derive"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true      # 声明为过程宏 crate，必须独立编译

[dependencies]
syn        = { version = "2", features = ["full"] }
quote      = "1"
proc-macro2 = "1"
```

=== 1.3.1 Path 依赖 vs 版本依赖

#table(
  columns: (auto, 1fr, 1fr),
  inset: 8pt,
  align: (left, left, left),
  stroke: 0.5pt,
  [*写法*], [*适用场景*], [*发布行为*],
  [`{ path = "../foo" }`],   [同 workspace 内成员互相引用], [发布时需替换为版本号],
  [`{ version = "1.0" }`],   [引用 crates.io 上的外部库],   [直接使用],
  [`{ git = "..." }`],        [引用 GitHub 上未发布的库],     [锁定到具体 commit],
)

== 1.4 常用命令

所有命令在 workspace 根目录运行，用 `-p` 指定具体成员：

```bash
# 编译所有成员
cargo build

# 只编译某个成员
cargo build -p modelparams

# 运行某个成员的示例
cargo run -p modelparams-examples --example soil_model

# 运行所有测试
cargo test

# 只测试某个成员
cargo test -p modelparams

# release 模式（优化，去调试符号）
cargo build --release -p modelparams-examples --example van_genuchten

# 检查代码（不编译，速度快）
cargo check

# 格式化
cargo fmt

# Lint 检查
cargo clippy
```

== 1.5 Proc Macro Crate 的特殊性

`modelparams-derive` 声明了 `proc-macro = true`，这类 crate 有特殊限制：

- *必须独立编译*：它运行在编译器内部（宿主机器），不能与普通 crate 混合链接
- *只能导出过程宏*：不能导出普通函数或类型供运行时使用
- *编译时执行*：`#[derive(ModelParams)]` 在 `cargo build` 时展开，生成代码注入到调用方

```
编译时：
  modelparams-derive  →  编译为宿主工具
        ↓ 展开 #[derive(ModelParams)]
  modelparams         →  编译为目标库
```

== 1.6 共享依赖版本（workspace 继承）

Rust 1.64+ 支持在根 `Cargo.toml` 统一声明依赖版本，成员直接继承，避免版本漂移：

```toml
# workspace Cargo.toml
[workspace.dependencies]
rayon = "1"
rand  = { version = "0.8", features = ["small_rng"] }
syn   = { version = "2", features = ["full"] }
```

```toml
# 成员 Cargo.toml 中继承
[dependencies]
rayon = { workspace = true }
rand  = { workspace = true }
```

== 1.7 编译产物位置

所有编译产物统一输出到 workspace 根的 `target/`：

```
target/
├── debug/
│   ├── libmodelparams.rlib          ← 开发库
│   ├── modelparams_derive.dll       ← proc macro（Windows）
│   └── examples/
│       ├── soil_model.exe
│       └── van_genuchten.exe
└── release/
    └── examples/
        ├── soil_model.exe           ← 1.2 MB，无外部依赖
        └── van_genuchten.exe
```

`debug/` 含调试符号，编译快；`release/` 开启优化（`-O3` + LTO），体积小、运行快。

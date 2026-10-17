# wslc-runtime

WSL Containers の SDK・セッション・コンテナ実行を扱う Rust ライブラリと、ホストのコマンドへセッションを貸し出す `wslc-session` です。GCC、ソースの形式、特定のアプリの設定には依存しません。

## 必要な環境とインストール

WSLC の実行には x64 Windows と WSL 2.9.3 以上が必要です。Rust のビルドには固定した公式 1.99.0 ツールチェーンを使います。初回は Microsoft.WSL.Containers 3.0.1 の SDK を取得し、SHA-256 を検証します。

```sh
cargo install --path crates/wslc-runtime --locked
wslc-session --help
```

`process` モジュールは Windows と Unix で利用できます。SDK・セッション・コンテナのモジュールは Windows で提供します。

## 所有と接続

`with_session` はセッションを作成し、コールバック終了時に終了・解放します。`Context::connect` は既存の名前へ接続するだけで、セッションを作成・終了しません。`with_connection` は `WSLC_SESSION` が指定されていれば接続し、省略時は自分で所有します。

```rust,no_run
use std::process::Command;
use wslc_runtime::container::{SessionOptions, with_session};
use wslc_runtime::process::{Runner, CONTAINER_TIMEOUT};

let root = std::env::current_dir().unwrap();
let options = SessionOptions::new(root.join(".tools"));
with_session(&root, &options, |session| {
    let mut command = Command::new("your-build-tool");
    session.configure_child(&mut command);
    Runner { root: &root }.run_with_input(command, CONTAINER_TIMEOUT, false, true)?;
    Ok(())
}).unwrap();
```

`configure_child` / `child_environment` は子の `WSLC_SESSION` と `WSLC_EXECUTABLE` を設定します。プロセス全体の環境変数は変更しません。親は子の完了までセッションを保持する必要があります。接続先の存続・アクセス権は呼び出し側の契約です。

SDK キャッシュの更新は短いロックで保護し、実行中はセッション保存先を別のロックで保護します。同じ保存先を所有しようとする呼び出しは待機します。既存セッションへの接続は保存先のロックを取り直さないため、同じセッション内の複数のコンテナを並列実行できます。独立した所有者を同時実行する場合は `SessionOptions.storage` も分けます。

## コマンドへ貸し出す

```sh
wslc-session -- your-build-tool
```

指定したホストのコマンドへ接続情報を渡し、その終了まで保持します。ネストした呼び出しは既存セッションを利用します。子の入出力と終了コードを中継し、通常の開始・終了ログは出しません。`wslc-runtime` 自体は、貸し出し先のコマンドの種類を知りません。

| 環境変数 | 意味 | 既定値 |
| --- | --- | --- |
| `WSLC_SESSION` | 接続する既存セッション | 省略時は所有 |
| `WSLC_EXECUTABLE` | `wslc.exe` | PATH、次に Program Files |
| `WSLC_CACHE` | SDK のキャッシュ | LocalAppData/wslc-runtime |
| `WSLC_STORAGE` | 所有するセッションの保存先 | キャッシュ/session-storage |
| `WSLC_MEMORY_MB` | 所有するセッションのメモリ | 2048 |
| `WSLC_SDK_VERSION` / `WSLC_SDK_SHA256` | SDK の版と検証値 | 対応する 3.0.1 の値 |
| `WSLC_COMMAND_TIMEOUT` | ランチャーの子の制限秒数 | 1800 |

Rust API はキャッシュ・保存先・メモリ・SDK を `SessionOptions` でも指定できます。CPU 数はホストの利用可能な CPU 数に合わせます。対応する SDK の C ABI は 3.0.1 に固定しています。

## 実行と解放

`RunOptions` はイメージ、マウント、作業ディレクトリ、エントリポイント、引数を別々に保持します。引数をシェルのコードに埋め込みません。ホストのマウント元はローカル Windows ドライブ上、コンテナの作業ディレクトリは絶対パスにします。UNC パスは対象外です。

プロセスは Windows の Job Object / Unix のプロセスグループで所有し、タイムアウト時は子孫も終了します。`Runner` の文字列取得 API は UTF-8 不正列を置換します。バイトをそのまま中継する CLI は標準ハンドルを継承して利用します。Windows の NUL 入力を WSLC の対話入力へ接続しないため、`stdin_available` を提供します。

通常の終了・処理エラー・プロセスのタイムアウト時には所有セッションを終了・解放します。所有者自体の強制終了後の SDK による解放は保証しません。接続しただけのセッションや既定セッションは終了しません。

## 検証

```sh
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
```

テストはアプリに依存しない合成プロセス、キャッシュの破損、所有権、タイムアウトを使います。端末継承のテストは Windows の端末で明示的に実行します。

```sh
cargo test --locked -p wslc-runtime --test process -- --ignored --exact streaming_children_keep_the_attached_console --nocapture
```

実際の GCC との連携は、独立した `wslc-gcc` リポジトリの統合テストで検証します。ライセンスは [MIT](LICENSE) です。

## Install from Git

```sh
cargo install --git https://github.com/uni-kakurenbo/wslc-runtime --locked wslc-runtime
```

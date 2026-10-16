# テスト

通常の`cargo test --locked`は、台帳・stale伝播・網羅性・再試行・SCC順序・ロック・revision conflictの回帰、実際のCargo/rust-analyzerを使うCLIの回帰、実際のKani 0.68.0のJSON出力を使う結果解析の回帰を実行します。Unixのプロセス停止テストは別process groupの子も含めてtimeoutを確認します。CLIテストにはPython 3が必要です。

`tests/support/codex.py`はstdio app-serverを模擬し、実際のcargo-elicit agent CLIを呼びます。正常な完了通知だけで成果を出さないケース、failed status、無人承認、本文の変更、中断とthread再開を再現します。`tests/support/cargo.py`は検証器コマンドだけを置き換え、metadataなど通常のCargo操作は実物に委譲します。製品にtest-onlyの環境変数や偽成功経路はありません。

nativeテストは既定でignoreです。製品用と同じラッパーで実際の検証器を実行します。Codex側は決定的なprotocol peerを使い、AI生成の品質やlive AI turnの確認は対象にしません。

```sh
cargo test --release --locked --test native -- --ignored --test-threads=1
```

各テストはfixture package全体を一時ディレクトリへコピーし、元の関数を検証します。検証用に実装関数のコピーを別crateへ作ってはいません。中断・再開、仕様化モード（証明探索なし）、検証必須モード、cache削除後のinitと証明結果再利用を確認します。失敗時の一時ディレクトリは診断のため保持します。

再構築時にはLinux x86_64で通常テスト39件とnativeテスト2件が成功し、fmt・Clippy・Rust 1.86のビルド確認も通過しています。

再構築時のツール構成:

- Rust stable 1.99.1 / MSRV 1.86.0
- Codex app-server schema 0.159.2
- Kani 0.68.0 / CBMC 6.11.0 / nightly-2026-08-21
- Creusot 0.14.0 / nightly-2026-08-03
- Why3 `05bf4ddf55cb3042f34c3bc0ed3aff56a226978a`（1.8.2+git）
- Why3find `0f054b93c86ac7ba20bca7df5d1563a9a4434a30`（1.3.0+dev）
- Alt-Ergo 2.6.4

CIはfmt、clippy、通常のtest、MSRV checkを実行します。native toolchainのインストールやlive AI turnはCIに含みません。実行済み結果はリポジトリのCI状態とコミットの検証記録で確認してください。

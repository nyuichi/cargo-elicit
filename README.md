# cargo-elicit

既存のRustクレートのcontract・型不変条件・証明コードを、Codexに継続的に整備させるCargoサブコマンドです。対象選択、進捗の永続化、仕様変更の影響追跡、検証結果の再利用を担当します。

## インストール

Rust 1.86以降が必要です。

```sh
cargo install --git https://github.com/nyuichi/cargo-elicit --locked
rustup component add rust-analyzer rust-src
```

別途 [Codex CLI](https://developers.openai.com/codex/cli/) 0.159.xと、[Creusot](https://github.com/creusot-rs/creusot) 0.14.x または [Kani](https://github.com/model-checking/kani) 0.68.xを、その検証器の手順に従ってインストールしてください。Creusotは同梱のWhy3・Why3find・標準モデルとSMT solverの設定も必要です。モデル、推論努力量、認証、sandbox、承認方針にはCodex自身の設定を使います。

## 使い方

対象クレートのディレクトリで実行します。

```sh
cargo elicit init --tool kani
cargo elicit run
cargo elicit status
cargo elicit list --json
```

Creusotなら `init --tool creusot` を使用します。設定は次の3項目です。

```toml
[agent]
provider = "codex"

[verification]
tool = "kani" # または "creusot"
required = false
```

`required = false` は仕様化と検証器による構文・型確認までを目標にします。`true` に変更すると既存の仕様を再利用し、contract全体の証明を追加の完了条件にします。型・traitには独立した「verified」を表示しません。

複数パッケージのworkspaceでは明示的に一つを選びます。

```sh
cargo elicit init --tool creusot --package my-crate --features feature-a,feature-b
cargo elicit status --json
cargo elicit run --retry --max-tasks 10 --max-seconds 1200
```

初期化時のfeatures・targetと、解決されたfeatures・host targetを保存・表示します。Kani 0.68のアダプターはhost targetだけに対応します。保存後の検証器・ビルド範囲の切り替えは拒否し、既存台帳を保持します。別の構成には別の作業ディレクトリを使用してください。

既定のrun上限は32タスク、合計3600秒、1タスク600秒、検証600秒、タスクごとの累積試行3回、連続無進捗2回です。CLIの `--help` に表示し、引数で変更できます。同じ失敗を同じ入力で繰り返しません。方針を見直して再試行する場合は `run --retry` を使います。

終了コードはrunの目標完了が0、未解決が1、設定・環境エラーが2、中断が130です。`status`・`list` は読み取りに成功すれば未解決があっても0です。

## 何を信頼できるか

- contract・証明・モデルは元のソースツリーに保存します。検証用に関数を複製しません。
- 正式な証明結果はcargo-elicitのラッパーだけが記録します。Codexの自己申告やツールの終了コード0だけではverifiedにしません。
- 型・入力範囲が限定されたハーネス、未証明helper、trusted、義務未実行、検証中に入力が変化した結果を全APIの証明に昇格させません。
- 仕様の再確認と証明の現行性は別に追跡します。直接依存の仕様を変更せず再確認できた場合、その先への仕様再生成を止めます。
- `status`・`list` も現在のソースと照合します。未申告の仕様依存があっても、パッケージ・ローカル依存・設定・ツール・標準モデル等の広いfingerprintで古い証明の再利用を防ぎます。

**初版の全contract完了判定は、bool・char・整数・unitを使う独立したfree functionに限定しています。** Kaniでは同じファイルの型付き任意入力ハーネス、Creusotでは元関数に対応する新しいComa出力と完全なWhy3証明木を照合します。ジェネリック、method、trait適合、helperや独自モデルに依存する証明は、実行結果を保存しても範囲不明／部分結果として残します。型不変条件やtrait lawを表現・処理できない場合も未解決として報告させます。詳細は [実装と制限](docs/implementation.md) を参照してください。

仕様の有用性や利用者の意図との一致は自動保証しません。Codexの生成した変更はレビューしてください。

## 保存と再開

```text
elicit.toml
src/                         # 元の実装・contract・proof
.elicit/state.toml            # schema付き単一台帳、結果要約、再試行・再開情報
.elicit/runtime/              # run / state / worker / verifier locks
.elicit/cache/                # 再生成可能な解析・検証出力と専用build cache
```

台帳はGit管理できます。`.elicit/runtime/`、`.elicit/cache/`、Creusotの`.why3find/`はGitignoreに追加してください。キャッシュを消しても仕様化・成功記録の要約を失いません。

Ctrl-C時はworkerと可能な範囲の子プロセスを停止し、未確定タスクをチェックポイントとして保持します。次回はソース照合から再開します。`init`の再実行も既存のコード・台帳・結果を保持します。対象クレートの未コミット変更をcommit・reset・cleanしません。

エージェントによる実装本文の変更は完了をブロックします。人間が差分を確認して意図した変更と判断した場合だけ、次のコマンドで基準を更新できます。更新後も仕様の再確認が必要です。

```sh
cargo elicit accept-implementation 'TARGET_ID' --reason '変更を承認した理由'
```

無人実行で承認が必要になった場合は、自動承認せず停止します。対話端末ではCodexのcommand/file承認要求を表示します。未対応の対話要求は理由を返して停止します。

## 開発・テスト

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked -- --test-threads=2
cargo +1.86 check --locked
# 実際の両検証器とツールチェーンのインストール後
cargo test --release --locked --test native -- --ignored --test-threads=1
```

通常のCLIテストは実際のCargo・rust-analyzerと、決定的なCodex／検証器のprotocol fixtureを使用します。nativeテストは実際の検証器で両モード、中断・再開、キャッシュ削除後の再利用を確認します。AIのlive turnを実行するテストではありません。[テストの説明](docs/testing.md)。

仕様書: [v0.6](docs/spec-v0.6.md)。ライセンス: [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)。

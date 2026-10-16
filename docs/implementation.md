# 実装と現在の対応範囲

## 構成

`analysis` がCargo metadataとrust-analyzer stdio LSPを管理し、`syn`で元ソースを構造化します。選択したパッケージ・features・target・検証器cfgをrust-analyzerに渡し、workspaceの読み込み完了を待って定義・呼び出し・明示的再エクスポートを照会します。run中は同じLSP接続を再利用し、入力が変わったときに解析を更新します。

IDはpackage、module path、定義種別、名前、impl headerのhashから構成します。impl self typeとtraitを区別し、行番号は照合位置としてのみ使います。対応が不明な削除・改名は旧対象の成果を残して要確認とします。旧対象を補助定義へ変えて進捗の分母を縮めることも拒否します。

解析は完全なrustc HIRの抽出ではありません。未解決のローカル呼び出し、マクロによる定義、glob re-export、build.rsの生成物は解析不完全として表示し、全体完了を防ぎます。cfgは選択target/featuresと検証器cfgを適用しますが、任意の属性マクロやbuild scriptの独自cfgの完全な再現は未対応です。公開名の同一性が曖昧な場合も完了を引き継ぎません。

`model` は対象と正式な検証結果を分離します。verifiedは、仕様が現行で、既知の仕様依存が再確認済みで、同じ広い入力fingerprintに対して、全required unitが制限・未証明仮定なしに証明済みかどうかから導出します。型・traitに独立したverifiedはありません。循環した未確認の仕様依存を証明根拠にできません。

`store` は単一のschema 1の`state.toml`をロック・fsync・原子的置換で更新します。変更はrevisionで検査します。run、worker、正式検証には別のロックがあります。正式検証中はworker turnを開始せず、前後の入力照合で外部編集も検出します。これは同じOSアカウントでファイルを書き換えられるworkerに対する安全境界ではありません。

`scheduler` は再確認を優先し、通常は型・traitから仕様化し、呼び出しグラフのSCCをcallee-first・安定ID順に処理します。検証必須時は仕様化済みグループの検証を次の仕様化より優先します。失敗したタスクは同一入力で抑制し、入力変更があっても累積の試行上限・無進捗上限はリセットしません。明示的な`--retry`でのみ累積上限をリセットします。

`agent` はCodex 0.159.xのstdio app-serverプロトコルを実装しています。initialize、thread start/resume、turn start/completed/interrupt、token usageを扱います。モデル・推論努力・権限・認証を上書きしません。長い共通指示はthreadのdeveloper instructionsに置き、各turnにはタスク・対象・関係仕様・revisionを渡します。thread喪失時は新しいthreadで台帳から継続します。Prompt Cacheのhitは保証しません。

## 検証アダプター

| 項目 | Kani 0.68.x | Creusot 0.14.x |
|---|---|---|
| 仕様処理 | `--only-codegen`、function-contracts | `--only coma`、erasure check |
| 正式検証 | cargo kani、fresh `--export-json` | fresh Coma生成、why3find prove |
| 結果 | schema 1.0のharness metadata、実行数、個別checks | 元ソース位置・関数名と対応するComa、`proof.json`のVCと証明木 |
| 全contract完了の範囲 | 同じファイルの独立したプリミティブfree function、明示型`::kani::any()`の入力、一回の元関数呼び出し | 独立したプリミティブfree function、閉じたcontract、標準creusotモデルと標準変換のみ |
| 未対応として保持 | method、generics、独自モデル、helper、入力制限、stub、loop contracts、外部依存等 | method、generics、独自モデル、helper、project axiom/assume、custom driver等 |

全contract判定には少なくとも一つのensuresと実際の証明義務の対応が必要です。空のcontract、単なるプロセス成功、義務0件、trustedによる本文省略を証明成功にしません。型付きハーネスでも元APIがgenericなら、その一つの型の成功を全APIの成功にしません。Kaniの`stub_verified`も名前で証明済みにはしません。

Creusotはcargo-elicit専用build cache内の対象パッケージだけを、Creusot指定のRust toolchainでcleanしてからComaを生成します。ユーザーのtarget directoryをcleanしません。Why3find/Why3はCreusot標準のインストールを使います。backend、compiler、標準モデル、solverを信頼基盤として記録し、その追跡できたファイルをfingerprintに含めます。Kaniのインストールを特定できない場合は成功を再利用可能としません。

任意の型の不変条件やtrait lawを自動的に全実装へ展開する仕組みは未実装です。workerは元ソースにモデル・法則・保存ハーネス等を追加できますが、未対応の性質は問題として残す必要があります。構文・型確認で表現の妥当性を処理できても、その性質を実装が満たす証明とは区別します。

## 変更と再利用

自身の仕様hashが変わると、その仕様と直接依存する仕様を再確認に戻します。BがAを参照し、CがBを参照する場合、Bの仕様内容を変更せずAの参照hashだけ更新すれば、Cの仕様を再生成しません。Bが未確認の間はCの証明完了も保留します。依存が回復し入力も同一なら、既存の証明結果は再利用できます。

未申告依存の変更を見逃さないため、検証fingerprintはソース、proof/model、packageとlocal dependencyのファイル、workspace/ancestor manifestとlockfile、Cargo設定、選択構成、compiler flags、toolchain・検証器・標準モデル等を広く含めます。台帳、runtime、cache、`.git`、Why3findのcache、Cargo metadataが返す実際のtarget directoryは除外します。メモだけの更新では入力を変えません。ソース入力のsymlinkは追跡できない入力としてエラーにします。

build scriptや外部モデルなどの追跡が十分でない結果は、全contractの完了として再利用しません。全features・全targetの網羅を示すものではありません。

実行時本文とシグネチャの変更を検出し、worker実行中の未承認の変更をブロックします。外側の既知のcontract属性は区別します。本文内のghost/loop注釈などは保守的な差分検査で実装変更と判定される場合があり、その場合は人間のレビューが必要です。任意のRustコードの意味上の同値性を判定する仕組みではありません。

## worker用CLI

`agent show [ID]`が現行revisionを返します。ソース編集後はこれを再取得してください。

- `agent submit ID --revision N --note REASON [--depends ID] [--unit UNIT]`: 構文・型処理後に仕様化／再確認を提出します。既知の依存を保持し、参照hashを更新します。contract unitを削除して分母を減らせません。
- `agent depend ID DEPENDENCY --revision N`: 既知の仕様依存を追加し、対象を再確認に戻します。
- `agent issue ID --revision N --reason REASON`: 修正前でも仕様を要再確認に戻します。過去の結果は保持します。
- `agent verify [ID ...] --revision N`: workerがidleの場合だけ正式検証ラッパーを実行します。turn内では準備を終えてschedulerに渡します。

生の会話、stdout、stderrは台帳に記録しません。エラーの要約とtypedな検証情報を保存します。これにより認証情報が含まれうるログの無条件保存を避けています。失敗の詳しい診断は検証器を直接実行して確認してください。native検証器自身の生成物はcacheに保持します。
